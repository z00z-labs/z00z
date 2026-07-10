use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use crate::{
    config::{ScenarioCfg, Stage6ProofMode},
    scenario_1::{stage_10, stage_11, stage_12, stage_7, stage_8, stage_9},
    StageResult,
};
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::write_file,
};

use super::{fixture_cache, scenario_support, stage_runner_support};

const STAGE10_MUTATION_RELS: &[&str] = &[
    "claim_publish",
    "stage_4_snapshot.json",
    "transactions",
    "wallets",
];
const STAGE12_MUTATION_RELS: &[&str] = &[
    "storage/post_tx",
    "transactions/checkpoint_s7.json",
    "transactions/checkpoint_s8.json",
];

fn good_s4(cfg: &mut ScenarioCfg) {
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
    stage4.transaction.class = "Coin".to_string();
    stage4.transaction.symbol = "Z00Z".to_string();
    stage4.transaction.mode = "fraction".to_string();
    stage4.transaction.fraction = Some(0.1);
    stage4.transaction.amount = None;
}

fn with_opaque_mode(cfg: &mut ScenarioCfg) {
    good_s4(cfg);
    cfg.stage6_bundle
        .get_or_insert_with(Default::default)
        .proof_mode = Stage6ProofMode::OpaqueTest;
}

fn make_case_cfg_in(
    base: &Path,
    edit_cfg: impl FnOnce(&mut ScenarioCfg),
) -> (PathBuf, PathBuf, PathBuf) {
    let out = base.join("outputs/scenario_1");
    let mut cfg = ScenarioCfg::from_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_config.yaml"),
    )
    .expect("load scenario config");
    cfg.stage1_genesis
        .get_or_insert_with(Default::default)
        .genesis_config = z00z_core::config_paths::devnet_genesis_path()
        .to_string_lossy()
        .to_string();
    cfg.outputs.dir = out.to_string_lossy().to_string();
    cfg.stage5_transfer
        .as_mut()
        .expect("stage5 cfg")
        .recipient_output_index = 0;
    edit_cfg(&mut cfg);

    let cfg_path = base.join("scenario_config.yaml");
    let cfg_bytes = YamlCodec.serialize(&cfg).expect("serialize cfg");
    write_file(&cfg_path, &cfg_bytes).expect("write cfg");

    let design_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_design.yaml");
    (cfg_path, design_path, out)
}

fn run_stage_ids(ctx: &mut crate::SimContext, design_path: &Path, stage_ids: &[u32]) {
    for &stage_id in stage_ids {
        let stage = stage_runner_support::stage_by_id(design_path, stage_id);
        let res = match stage_id {
            7 => stage_7::run_transfer_receive(ctx, &stage),
            8 => stage_8::run_transfer_claim(ctx, &stage),
            9 => stage_9::run_bundle_build(ctx, &stage),
            10 => stage_10::run_bundle_publish(ctx, &stage),
            11 => stage_11::run_apply(ctx, &stage),
            12 => stage_12::run_finalize(ctx, &stage),
            _ => unreachable!(),
        };
        assert!(
            matches!(res, StageResult::Ok),
            "stage {stage_id} must succeed: {res:?}"
        );
    }
}

fn build_out(case_name: &str, stage_ids: &[u32], validate_out: impl FnOnce(&Path)) -> PathBuf {
    let root = fixture_cache::ensure_shared_case_precise(case_name, |base| {
        let (cfg_path, design_path, out) = make_case_cfg_in(base, with_opaque_mode);
        let mut ctx = stage_runner_support::run_stage4_session(&cfg_path, &design_path);
        run_stage_ids(&mut ctx, &design_path, stage_ids);
        validate_out(&out);
    });
    root.join("outputs/scenario_1")
}

pub fn stage10_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_out(
            "checkpoint_acceptance_stage10_shared_v2",
            &[7_u32, 8, 9, 10],
            |out| {
                assert!(
                    out.join("transactions/tx_alice_to_bob_pkg.json").exists(),
                    "stage10 shared case missing tx package"
                );
                assert!(
                    out.join("claim_publish/audit_log.json").exists(),
                    "stage10 shared case missing claim publish audit"
                );
                assert!(
                    !out.join("transactions/checkpoint_s7.json").exists(),
                    "stage10 shared case must stop before stage11 apply"
                );
            },
        )
    })
    .clone()
}

pub fn stage12_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_out(
            "checkpoint_acceptance_stage12_shared_v3",
            &[7_u32, 8, 9, 10, 11, 12],
            |out| {
                assert!(
                    out.join("transactions/checkpoint_s7.json").exists(),
                    "stage12 shared case missing checkpoint_s7"
                );
                assert!(
                    out.join("transactions/checkpoint_s8.json").exists(),
                    "stage12 shared case missing checkpoint_s8"
                );
                assert!(
                    out.join("storage/post_tx/artifacts/checkpoints").exists(),
                    "stage12 shared case missing post-tx checkpoint store"
                );
            },
        )
    })
    .clone()
}

pub fn clone_stage10_case(base: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let (cfg_path, design_path, out) = make_case_cfg_in(base, with_opaque_mode);
    fixture_cache::copy_selected(&stage10_out(), &out, STAGE10_MUTATION_RELS);
    (cfg_path, design_path, out)
}

pub fn clone_stage12_case(base: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let (cfg_path, design_path, out) = make_case_cfg_in(base, with_opaque_mode);
    fixture_cache::copy_selected(&stage12_out(), &out, STAGE12_MUTATION_RELS);
    (cfg_path, design_path, out)
}

fn checkpoint_bridge_cfg(cfg: &mut ScenarioCfg) {
    let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
    cfg.genesis_assets.truncate(1);
    for asset in &mut cfg.genesis_assets {
        asset.serials = asset.serials.min(4);
        asset.nominal = asset.nominal.max(50_000);
    }
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_min = 3;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_target = 3;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_max = 3;
    stage4.transaction.outputs.bob_outputs_count = 3;
    stage4.transaction.class = "Coin".to_string();
    stage4.transaction.symbol = "Z00Z".to_string();
    stage4.transaction.mode = "fraction".to_string();
    stage4.transaction.fraction = Some(0.1);
    stage4.transaction.amount = None;
}

fn checkpoint_bridge_draft_cfg(cfg: &mut ScenarioCfg) {
    checkpoint_bridge_cfg(cfg);
    cfg.stage6_bundle
        .get_or_insert_with(Default::default)
        .proof_mode = Stage6ProofMode::DraftOnly;
}

fn checkpoint_bridge_opaque_cfg(cfg: &mut ScenarioCfg) {
    checkpoint_bridge_cfg(cfg);
    cfg.stage6_bundle
        .get_or_insert_with(Default::default)
        .proof_mode = Stage6ProofMode::OpaqueTest;
}

fn run_checkpoint_stage_ids(ctx: &mut crate::SimContext, design_path: &Path, stage_ids: &[u32]) {
    for &stage_id in stage_ids {
        let stage = stage_runner_support::stage_by_id(design_path, stage_id);
        let res = match stage_id {
            9 => stage_9::run_bundle_build(ctx, &stage),
            11 => stage_11::run_apply(ctx, &stage),
            12 => stage_12::run_finalize(ctx, &stage),
            _ => unreachable!(),
        };
        assert!(
            matches!(res, StageResult::Ok),
            "stage {stage_id} must succeed: {res:?}"
        );
    }
}

fn build_checkpoint_out(
    case_name: &str,
    edit_cfg: impl FnOnce(&mut ScenarioCfg),
    stage_ids: &[u32],
    validate_out: impl FnOnce(&Path),
) -> PathBuf {
    let root = fixture_cache::ensure_shared_case_precise(case_name, |base| {
        let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, edit_cfg);
        let mut ctx = stage_runner_support::run_stage4_session(&cfg_path, &design_path);
        run_checkpoint_stage_ids(&mut ctx, &design_path, stage_ids);
        validate_out(&out);
    });
    root.join("outputs/scenario_1")
}

pub fn default_stage11_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_checkpoint_out(
            "checkpoint_stage11_shared_v2",
            checkpoint_bridge_cfg,
            &[9_u32, 11],
            |out| {
                assert!(out.exists(), "cached checkpoint output must exist");
            },
        )
    })
    .clone()
}

pub fn bridge_stage9_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_checkpoint_out(
            "checkpoint_stage9_bridge_shared_v2",
            checkpoint_bridge_cfg,
            &[9_u32],
            |out| {
                assert!(
                    !out.join("transactions/artifacts/checkpoints/draft")
                        .exists(),
                    "stage9-only bridge baseline must stop before draft output"
                );
            },
        )
    })
    .clone()
}

pub fn draft_stage12_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_checkpoint_out(
            "checkpoint_stage12_draft_shared_v2",
            checkpoint_bridge_draft_cfg,
            &[9_u32, 11, 12],
            |out| {
                assert!(out.exists(), "cached checkpoint draft output must exist");
            },
        )
    })
    .clone()
}

pub fn opaque_stage12_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_checkpoint_out(
            "checkpoint_stage12_opaque_shared_v3",
            checkpoint_bridge_opaque_cfg,
            &[9_u32, 11, 12],
            |out| {
                assert!(out.exists(), "cached opaque checkpoint output must exist");
            },
        )
    })
    .clone()
}

pub fn opaque_stage11_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_checkpoint_out(
            "checkpoint_stage11_opaque_shared_v2",
            checkpoint_bridge_opaque_cfg,
            &[9_u32, 11],
            |out| {
                assert!(
                    !out.join("transactions/artifacts/checkpoints/final")
                        .exists(),
                    "stage11 baseline must stop before stage12 final artifact emission"
                );
            },
        )
    })
    .clone()
}
