use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Arc,
};

use super::scenario_support;
use crate::{
    config::ScenarioCfg,
    design::DesignDoc,
    scenario_1::runner,
    scenario_1::{
        stage_1, stage_10, stage_11, stage_12, stage_13, stage_2, stage_3, stage_4, stage_5,
        stage_6, stage_7, stage_8, stage_9,
    },
    DesignStage, ScenarioResult, SimContext, StageResult, StageState,
};
use z00z_core::{AssetDefinitionRegistry, ChainType};
use z00z_utils::{logger::NoopLogger, metrics::NoopMetrics, time::SystemTimeProvider};
use z00z_wallets::claim::registry as claim_registry;

const STAGE_SETUP_RETRIES: usize = 3;

pub struct StageSession {
    pub ctx: SimContext,
    _process_guard: crate::scenario_1::ScenarioProcessGuard<'static>,
    _io_guards: runner::StageIoGuards,
}

pub struct ProcessLock {
    _guard: crate::scenario_1::ScenarioProcessGuard<'static>,
}

impl Deref for StageSession {
    type Target = SimContext;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl DerefMut for StageSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

pub fn acquire_process_lock() -> ProcessLock {
    ProcessLock {
        _guard: crate::scenario_1::acquire_scenario_process_guard(),
    }
}

pub fn make_cfg(edit_cfg: impl FnOnce(&mut ScenarioCfg)) -> (PathBuf, PathBuf, PathBuf) {
    scenario_support::make_cfg(|cfg| {
        apply_stage5_defaults(cfg);
        edit_cfg(cfg);
    })
}

pub fn make_cfg_in(
    base: &Path,
    edit_cfg: impl FnOnce(&mut ScenarioCfg),
) -> (PathBuf, PathBuf, PathBuf) {
    scenario_support::make_cfg_in(base, |cfg| {
        apply_stage5_defaults(cfg);
        edit_cfg(cfg);
    })
}

fn apply_stage5_defaults(cfg: &mut ScenarioCfg) {
    let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_min = 4;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_target = 4;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_max = 10;
    stage4.transaction.outputs.bob_outputs_count = 4;
    stage4.transaction.fraction = Some(0.1);

    let stage5 = cfg.stage5_transfer.as_mut().expect("stage5 cfg");
    stage5.recipient_output_index = 0;
}

pub fn stage_by_id(design_path: &Path, stage_id: u32) -> DesignStage {
    let doc = DesignDoc::from_file(design_path).expect("load design");
    doc.stages
        .iter()
        .find(|item| item.stage == stage_id)
        .cloned()
        .expect("stage exists")
}

pub fn tx_pkg_path(out: &Path) -> PathBuf {
    out.join("transactions").join("tx_alice_to_bob_pkg.json")
}

pub fn mk_ctx(cfg_path: &Path) -> SimContext {
    let cfg = ScenarioCfg::from_file(cfg_path).expect("load scenario cfg");
    let chain = cfg
        .chain
        .parse::<ChainType>()
        .unwrap_or_else(|err| panic!("invalid simulator chain '{}': {err}", cfg.chain));
    let out_dir = PathBuf::from(cfg.outputs.dir.clone());
    let reg = AssetDefinitionRegistry::new(
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );

    SimContext {
        config: cfg,
        chain_type: chain,
        registry: reg,
        assets: Vec::new(),
        genesis_rights: Vec::new(),
        actors: Vec::new(),
        leaves: Vec::new(),
        block_height: 0,
        outputs_dir: out_dir,
        logger: Arc::new(NoopLogger),
        wallet_service: None,
    }
}

pub fn run_stage_once(ctx: &mut SimContext, design_path: &Path, stage_id: u32) -> StageResult {
    let stage = stage_by_id(design_path, stage_id);
    match stage_id {
        1 => stage_1::run(ctx, &stage),
        2 => stage_2::run(ctx, &stage),
        3 => stage_3::run_claim_genesis(ctx, &stage),
        4 => stage_4::run_claim_publish(ctx, &stage),
        5 => stage_5::run_tx_plan(ctx, &stage),
        6 => stage_6::run_tx_prepare(ctx, &stage),
        7 => stage_7::run_transfer_receive(ctx, &stage),
        8 => stage_8::run_transfer_claim(ctx, &stage),
        9 => stage_9::run_bundle_build(ctx, &stage),
        10 => stage_10::run_bundle_publish(ctx, &stage),
        11 => stage_11::run_apply(ctx, &stage),
        12 => stage_12::run_finalize(ctx, &stage),
        13 => stage_13::run_hjmt_examples(ctx, &stage),
        _ => unreachable!(),
    }
}

pub fn run_stage_plan_subset(
    cfg_path: &Path,
    design_path: &Path,
    stage_ids: &[u32],
) -> ScenarioResult {
    let mut session = if stage_ids.first() == Some(&1_u32) {
        begin_stage_session(cfg_path, true)
    } else {
        begin_stage_session(cfg_path, false)
    };
    let mut out = ScenarioResult::new(session.config.scenario.id);

    for &stage_id in stage_ids {
        let stage = stage_by_id(design_path, stage_id);
        let result = run_stage_once(&mut session, design_path, stage_id);
        let is_fail = matches!(result, StageResult::Fail(_));
        out.stages.push(StageState {
            stage: stage_id,
            name: stage.name,
            result,
        });
        if is_fail && session.config.simulation.abort_on_fail {
            out.is_aborted = true;
            break;
        }
    }

    out
}

fn begin_stage_session(cfg_path: &Path, reset_outputs: bool) -> StageSession {
    let process_guard = crate::scenario_1::acquire_scenario_process_guard();
    // Stage-by-stage runs emulate independent scenario entries and must not
    // inherit process-global claim membership from prior test cases.
    claim_registry::clear_rows();
    let ctx = mk_ctx(cfg_path);
    // Keep stage-by-stage tests on the same IO isolation path as the canonical runner.
    let io_guards = if reset_outputs {
        runner::prepare_stage_io_for_cfg(cfg_path, &ctx.config.outputs.dir)
    } else {
        runner::prepare_existing_stage_io(cfg_path, &ctx.config.outputs.dir)
    }
    .unwrap_or_else(|err| panic!("prepare stage io failed: {err}"));
    StageSession {
        ctx,
        _process_guard: process_guard,
        _io_guards: io_guards,
    }
}

pub fn resume_stage_session(cfg_path: &Path) -> StageSession {
    begin_stage_session(cfg_path, false)
}

pub fn run_stage_setup_session(
    cfg_path: &Path,
    design_path: &Path,
    stage_ids: &[u32],
) -> StageSession {
    try_run_stage_setup_session(cfg_path, design_path, stage_ids)
        .unwrap_or_else(|err| panic!("{err}"))
}

pub fn try_run_stage_setup_session(
    cfg_path: &Path,
    design_path: &Path,
    stage_ids: &[u32],
) -> Result<StageSession, String> {
    let mut last_err = None;

    for attempt in 1..=STAGE_SETUP_RETRIES {
        let mut session = begin_stage_session(cfg_path, true);
        let mut failed = None;
        for &stage_id in stage_ids {
            let result = run_stage_once(&mut session, design_path, stage_id);
            if !matches!(result, StageResult::Ok) {
                failed = Some(format!("stage {stage_id} failed: {result:?}"));
                break;
            }
        }
        if let Some(err) = failed {
            last_err = Some(format!("attempt {attempt}: {err}"));
            continue;
        }
        return Ok(session);
    }

    Err(format!(
        "stage setup failed after {STAGE_SETUP_RETRIES} attempts: {}",
        last_err.unwrap_or_else(|| "unknown stage setup error".to_string())
    ))
}

pub fn run_stage_setup(cfg_path: &Path, design_path: &Path, stage_ids: &[u32]) -> SimContext {
    run_stage_setup_session(cfg_path, design_path, stage_ids).ctx
}

pub fn run_stage5_setup(cfg_path: &Path, design_path: &Path) -> SimContext {
    run_stage_setup(cfg_path, design_path, &[1_u32, 2, 3, 4, 5])
}

pub fn run_stage4_setup(cfg_path: &Path, design_path: &Path) -> SimContext {
    run_stage_setup(cfg_path, design_path, &[1_u32, 2, 3, 4, 5, 6])
}

pub fn run_stage5_session(cfg_path: &Path, design_path: &Path) -> StageSession {
    run_stage_setup_session(cfg_path, design_path, &[1_u32, 2, 3, 4, 5])
}

pub fn run_stage4_session(cfg_path: &Path, design_path: &Path) -> StageSession {
    run_stage_setup_session(cfg_path, design_path, &[1_u32, 2, 3, 4, 5, 6])
}

pub fn try_run_stage4_session(cfg_path: &Path, design_path: &Path) -> Result<StageSession, String> {
    try_run_stage_setup_session(cfg_path, design_path, &[1_u32, 2, 3, 4, 5, 6])
}

pub fn run_stage5_edit(
    cfg_path: &Path,
    design_path: &Path,
    out: &Path,
    edit: impl FnOnce(&Path),
) -> StageResult {
    let mut ctx = run_stage4_session(cfg_path, design_path);
    let tx_path = tx_pkg_path(out);
    edit(&tx_path);
    let stage = stage_by_id(design_path, 7);
    stage_7::run_transfer_receive(&mut ctx, &stage)
}

pub fn fail_text(res: StageResult) -> String {
    match res {
        StageResult::Fail(msg) => msg,
        other => panic!("expected StageResult::Fail, got {other:?}"),
    }
}

pub fn stage_res(run: &ScenarioResult, stage_id: u32) -> &StageResult {
    scenario_support::stage_res(run, stage_id)
}
