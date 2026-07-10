//! Scenario 1 runner entry (library side).

use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::Write as _,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

use crate::{
    config::ScenarioCfgErr, scenario_1::stage_1, scenario_1::stage_10, scenario_1::stage_11,
    scenario_1::stage_12, scenario_1::stage_13, scenario_1::stage_2, scenario_1::stage_3,
    scenario_1::stage_4, scenario_1::stage_5, scenario_1::stage_6, scenario_1::stage_7,
    scenario_1::stage_8, scenario_1::stage_9, DesignDoc, DesignErr, DesignStage, ScenarioCfg,
    ScenarioResult, SimContext, StageResult, StageState,
};
use thiserror::Error;
use z00z_core::{AssetDefinitionRegistry, ChainType};
#[cfg(test)]
use z00z_utils::io::prepare_managed_root;
use z00z_utils::{
    config::{ConfigSource, EnvConfig},
    io::{
        self, current_exe_run_root, hash_root_inputs, reset_managed_root_once,
        stable_current_exe_scope,
    },
    logger::{Logger, NoopLogger, StdoutLogger},
    metrics::NoopMetrics,
    time::SystemTimeProvider,
};
use z00z_wallets::claim::registry as claim_registry;

use super::runner_contract::load_design;
use super::runner_verify::{check_stage, log_stage};
use super::runtime_observability;
use super::support::{
    fixture_cache,
    path_roots::{
        normalize_path, resolve_workspace_path, simulator_root, workspace_root,
        workspace_target_root,
    },
};

// Stage 5 and stage 6 continue to route through stage_4 via their facades.
const CFG_PATH: &str = "crates/z00z_simulator/src/scenario_1/scenario_config.yaml";
const DESIGN_PATH: &str = "crates/z00z_simulator/src/scenario_1/scenario_design.yaml";
const CARGO_TARGET_DIR_ENV: &str = "CARGO_TARGET_DIR";
const RUNTIME_CWD_ROOT_ENV: &str = "Z00Z_RUNTIME_CWD_ROOT";
const STORAGE_ROOT_BASE_ENV: &str = "Z00Z_STORAGE_REDB_ROOT_BASE";
const SCENARIO_RUN_LOCK_WAIT_MS: u64 = 100;
const SCENARIO_CACHE_ROOT_ENV: &str = "Z00Z_SIMULATOR_CACHE_ROOT";
const SCENARIO_STORAGE_ROOT_ENV: &str = "Z00Z_SIMULATOR_STORAGE_ROOT";
const VERIFICATION_RUN_ROOT_ENV: &str = "Z00Z_VERIFICATION_RUN_ROOT";

type StageFn = fn(&mut SimContext, &DesignStage) -> StageResult;

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

struct ScenarioRunGuard {
    lock_path: PathBuf,
}

pub(crate) struct StageIoGuards {
    _run_file_guard: ScenarioRunGuard,
    _storage_root_guard: EnvVarGuard,
}

impl Drop for ScenarioRunGuard {
    fn drop(&mut self) {
        let _ = io::remove_file(&self.lock_path);
    }
}

impl EnvVarGuard {
    fn set_path(key: &'static str, value: &Path) -> Self {
        let env = EnvConfig;
        let previous = env.get(key).ok().flatten();
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(value) = &self.previous {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

/// Scenario 1 run-time errors.
#[derive(Debug, Error)]
pub enum Scenario1Err {
    /// Filesystem access failed.
    #[error("scenario_1 io failed: {0}")]
    Io(#[from] z00z_utils::io::IoError),
    /// Failed to load config yaml.
    #[error("scenario_1 config load failed: {0}")]
    Config(#[from] ScenarioCfgErr),
    /// Failed to load design yaml.
    #[error("scenario_1 design load failed: {0}")]
    Design(#[from] DesignErr),
    /// Missing required design yaml.
    #[error("scenario_1 design file is missing: {0}")]
    MissingDesign(String),
    /// Output directory escapes the approved simulator sandbox.
    #[error("scenario_1 output directory is outside the approved sandbox: {0}")]
    InvalidOutputSandbox(String),
    /// Scenario chain is invalid.
    #[error("scenario_1 invalid simulator chain '{0}'")]
    InvalidChain(String),
    /// Runtime-observability contract is missing or drifted.
    #[error("scenario_1 runtime evidence failed: {0}")]
    Evidence(String),
}

/// Runs scenario 1 with default config/design paths.
pub fn run() -> Result<ScenarioResult, Scenario1Err> {
    run_with_paths(CFG_PATH, DESIGN_PATH)
}

/// Runs scenario 1 with explicit config/design paths.
pub fn run_with_paths(
    cfg_path: impl AsRef<Path>,
    design_path: impl AsRef<Path>,
) -> Result<ScenarioResult, Scenario1Err> {
    // The simulator runner uses process-global env overrides and shared runtime/cache roots.
    // Keep one canonical in-process execution path so parallel integration tests cannot
    // stomp each other's storage-root, verification-root, or trace-pack setup.
    let _process_guard = super::acquire_scenario_process_guard();
    // Scenario runs must start from a fresh process-global claim membership state.
    claim_registry::clear_rows();
    let logger = StdoutLogger;

    let cfg_path_ref = cfg_path.as_ref();
    let design_path_ref = design_path.as_ref();

    logger.info(&format!(
        "scenario_1.start: config={}, design={}",
        cfg_path_ref.display(),
        design_path_ref.display()
    ));

    if !io::path_exists(design_path_ref)? {
        return Err(Scenario1Err::MissingDesign(
            design_path_ref.display().to_string(),
        ));
    }

    let cfg = ScenarioCfg::from_file(cfg_path_ref)?;
    let runtime_trace_spec = runtime_observability::prepare(cfg_path_ref, design_path_ref, &cfg)?;
    let out_dir = validate_output_sandbox_for_run(&cfg.outputs.dir, cfg_path_ref)?;
    let chain_type = parse_chain_type(&cfg.chain)?;
    let _io_guards = prepare_stage_io_for_cfg(cfg_path_ref, &cfg.outputs.dir)?;
    let design = load_design(design_path_ref)?;
    let stage_map = build_stage_map();
    let mut ctx = build_ctx(cfg, out_dir.clone(), chain_type);

    let scenario_started = Instant::now();
    let result = run_stage_plan(&logger, &mut ctx, &design, &stage_map);
    logger.info(&format!(
        "scenario.profile_total: stage_elapsed_ms={}",
        scenario_started.elapsed().as_millis()
    ));

    logger.info(&format!(
        "scenario_1.done: scenario_id={}, stage_count={}",
        result.scenario_id,
        result.stages.len()
    ));

    if result.is_ok() {
        runtime_observability::emit(
            &runtime_trace_spec,
            cfg_path_ref,
            design_path_ref,
            &out_dir,
            &ctx.config,
            &design,
            &result,
        )?;
        runtime_observability::validate_strict(cfg_path_ref, design_path_ref, &out_dir)?;
    }

    if let Err(err) = teardown_wallet_runtime(&mut ctx) {
        logger.warn(&format!("scenario_1.wallet_runtime_teardown_warn: {err}"));
    }

    Ok(result)
}

pub fn validate_runtime_observability_artifacts(
    cfg_path: impl AsRef<Path>,
    design_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<(), Scenario1Err> {
    runtime_observability::validate(cfg_path.as_ref(), design_path.as_ref(), out_dir.as_ref())
}

pub fn validate_design_file(path: impl AsRef<Path>) -> Result<DesignDoc, Scenario1Err> {
    let path_ref = path.as_ref();
    if !io::path_exists(path_ref)? {
        return Err(Scenario1Err::MissingDesign(path_ref.display().to_string()));
    }
    load_design(path_ref)
}

pub fn reset_outputs_dir(dir: &str) -> Result<(), Scenario1Err> {
    let out_dir = validate_output_sandbox(dir)?;
    recreate_outputs_dir(&out_dir)
}

pub(crate) fn prepare_stage_io_for_cfg(
    cfg_path: &Path,
    dir: &str,
) -> Result<StageIoGuards, Scenario1Err> {
    let out_dir = validate_output_sandbox_for_run(dir, cfg_path)?;
    prepare_stage_io_dir(&out_dir, true)
}

pub(crate) fn prepare_existing_stage_io(
    cfg_path: &Path,
    dir: &str,
) -> Result<StageIoGuards, Scenario1Err> {
    let out_dir = validate_output_sandbox_for_run(dir, cfg_path)?;
    prepare_stage_io_dir(&out_dir, false)
}

fn prepare_stage_io_dir(
    out_dir: &Path,
    reset_outputs: bool,
) -> Result<StageIoGuards, Scenario1Err> {
    let run_file_guard = acquire_scenario_run_guard(out_dir)?;
    if reset_outputs {
        recreate_outputs_dir(out_dir)?;
    }
    let storage_root_guard = install_storage_override(out_dir)?;
    Ok(StageIoGuards {
        _run_file_guard: run_file_guard,
        _storage_root_guard: storage_root_guard,
    })
}

fn recreate_outputs_dir(out_dir: &Path) -> Result<(), Scenario1Err> {
    if io::path_exists(out_dir)? {
        io::remove_dir_all(out_dir)?;
    }
    io::create_dir_all(out_dir)?;
    Ok(())
}

fn install_storage_override(out_dir: &Path) -> Result<EnvVarGuard, Scenario1Err> {
    let storage_root_root = scenario_storage_root();
    let process_prefix = format!("{}-pid-", exe_scope());
    prune_dead_process_scope_dirs(&storage_root_root, &process_prefix)?;
    let storage_scope_root = storage_root_root.join(process_scope_name());
    prepare_storage_scope(&storage_scope_root)?;
    let lane_hash = hex::encode(z00z_crypto::blake2b_hash(
        b"z00z.scenario.storage.lane.v1",
        &[out_dir.to_string_lossy().as_bytes()],
    ));
    let lane = &lane_hash[..16];
    let storage_root_base = storage_scope_root.join(lane);
    if io::path_exists(&storage_root_base)? {
        io::remove_dir_all(&storage_root_base)?;
    }
    io::create_dir_all(&storage_root_base)?;
    Ok(EnvVarGuard::set_path(
        STORAGE_ROOT_BASE_ENV,
        &storage_root_base,
    ))
}

fn prepare_storage_scope(scope_root: &Path) -> Result<(), Scenario1Err> {
    let _guard = storage_scope_lock()
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    prune_storage_root(scope_root)?;
    reset_managed_root_once(scope_root, &storage_runtime_fingerprint(), &[], None)?;
    Ok(())
}

#[cfg(test)]
fn recreate_storage_scope(scope_root: &Path) -> Result<(), Scenario1Err> {
    let _guard = storage_scope_lock()
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    if io::path_exists(scope_root)? {
        io::remove_dir_all(scope_root)?;
    }
    prune_storage_root(scope_root)?;
    prepare_managed_root(scope_root, &storage_runtime_fingerprint())?;
    Ok(())
}

fn prune_storage_root(scope_root: &Path) -> Result<(), Scenario1Err> {
    let scope_parent = scope_root.parent().ok_or_else(|| {
        Scenario1Err::Evidence(format!(
            "storage scope parent missing for {}",
            scope_root.display()
        ))
    })?;
    let scope_name = scope_root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            Scenario1Err::Evidence(format!(
                "storage scope name missing for {}",
                scope_root.display()
            ))
        })?;

    io::prune_scope_alias_dirs(scope_parent, scope_name)?;
    io::prune_hex_dirs(scope_parent, 16)?;
    Ok(())
}

fn storage_scope_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn acquire_scenario_run_guard(out_dir: &Path) -> Result<ScenarioRunGuard, Scenario1Err> {
    let lock_dir = scenario_cache_root().join("run_locks");
    io::create_dir_all(&lock_dir)?;
    let lock_name = format!("{}.lock", run_lock_key(out_dir));
    let lock_path = lock_dir.join(lock_name);

    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                let owner = format!("{}\n{}\n", std::process::id(), out_dir.display());
                file.write_all(owner.as_bytes())
                    .map_err(|err| Scenario1Err::Io(err.into()))?;
                return Ok(ScenarioRunGuard { lock_path });
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                if clear_stale_scenario_run_lock(&lock_path)? {
                    continue;
                }
                thread::sleep(Duration::from_millis(SCENARIO_RUN_LOCK_WAIT_MS));
            }
            Err(err) => return Err(Scenario1Err::Io(err.into())),
        }
    }
}

fn clear_stale_scenario_run_lock(lock_path: &Path) -> Result<bool, Scenario1Err> {
    let text = match io::read_to_string(lock_path) {
        Ok(text) => text,
        Err(z00z_utils::io::IoError::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => {
            // Another process may drop the lock after we observed AlreadyExists.
            // Treat that race as cleared so the caller retries immediately.
            return Ok(true);
        }
        Err(err) => return Err(Scenario1Err::Io(err)),
    };
    let Some(first_line) = text.lines().next() else {
        return Ok(false);
    };
    let Ok(pid) = first_line.trim().parse::<u32>() else {
        return Ok(false);
    };
    if fixture_cache::process_alive(pid) {
        return Ok(false);
    }
    match io::remove_file(lock_path) {
        Ok(()) => Ok(true),
        Err(z00z_utils::io::IoError::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => {
            Ok(true)
        }
        Err(err) => Err(Scenario1Err::Io(err)),
    }
}

fn run_lock_key(out_dir: &Path) -> String {
    hex::encode(z00z_crypto::blake2b_hash(
        b"z00z.scenario.run.lock.v1",
        &[out_dir.to_string_lossy().as_bytes()],
    ))
    .chars()
    .take(16)
    .collect()
}

fn exe_scope() -> String {
    stable_current_exe_scope("unknown_scenario_bin")
}

fn process_scope_name() -> String {
    format!("{}-pid-{}", exe_scope(), std::process::id())
}

fn runtime_cache_root_base() -> Option<PathBuf> {
    std::env::var_os(RUNTIME_CWD_ROOT_ENV)
        .map(PathBuf::from)
        .map(|root| resolve_workspace_path(&root).join("cache"))
        .or_else(|| {
            std::env::var_os(VERIFICATION_RUN_ROOT_ENV)
                .map(PathBuf::from)
                .map(|root| resolve_workspace_path(&root).join("cache"))
        })
        .or_else(|| current_exe_run_root().map(|root| resolve_workspace_path(&root).join("cache")))
        .or_else(|| {
            std::env::var_os(CARGO_TARGET_DIR_ENV)
                .map(PathBuf::from)
                .map(|target| resolve_workspace_path(&target).join("z00z-simulator-cache"))
        })
}

fn scenario_cache_root() -> PathBuf {
    std::env::var_os(SCENARIO_CACHE_ROOT_ENV)
        .map(PathBuf::from)
        .map(resolve_workspace_path)
        .or_else(|| runtime_cache_root_base().map(|root| root.join("scenario_1")))
        .unwrap_or_else(|| workspace_root().join(".cache").join("scenario_1"))
}

fn scenario_storage_root() -> PathBuf {
    std::env::var_os(SCENARIO_STORAGE_ROOT_ENV)
        .map(PathBuf::from)
        .map(resolve_workspace_path)
        .or_else(|| runtime_cache_root_base().map(|root| root.join("storage").join("scenario_1")))
        .unwrap_or_else(|| {
            workspace_root()
                .join(".cache")
                .join("storage")
                .join("scenario_1")
        })
}

fn prune_dead_process_scope_dirs(parent: &Path, prefix: &str) -> Result<(), Scenario1Err> {
    if !io::path_exists(parent)? {
        return Ok(());
    }

    let current_pid = std::process::id();
    for entry in io::read_dir(parent)? {
        if !entry.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(pid_text) = name.strip_prefix(prefix) else {
            continue;
        };
        let Ok(pid) = pid_text.parse::<u32>() else {
            continue;
        };
        if pid == current_pid {
            continue;
        }
        let proc_path = PathBuf::from(format!("/proc/{pid}"));
        if io::path_exists(&proc_path)? {
            continue;
        }
        io::remove_dir_all(&entry)?;
    }

    Ok(())
}

fn validate_output_sandbox(dir: &str) -> Result<PathBuf, Scenario1Err> {
    let simulator_root = simulator_root();
    let approved_roots = [
        normalize_path(&simulator_root.join("outputs/scenario_1")),
        workspace_target_root(),
    ];
    let candidate = resolve_output_candidate(dir);
    if approved_roots
        .iter()
        .any(|root| candidate.starts_with(root))
    {
        Ok(candidate)
    } else {
        Err(Scenario1Err::InvalidOutputSandbox(
            candidate.display().to_string(),
        ))
    }
}

fn validate_output_sandbox_for_run(dir: &str, cfg_path: &Path) -> Result<PathBuf, Scenario1Err> {
    let simulator_root = simulator_root();
    let candidate = resolve_output_candidate(dir);
    let approved_roots = [
        normalize_path(&simulator_root.join("outputs/scenario_1")),
        workspace_target_root(),
    ];
    if approved_roots
        .iter()
        .any(|root| candidate.starts_with(root))
    {
        return Ok(candidate);
    }

    if let Some(fixture_root) = approved_fixture_root(cfg_path) {
        let fixture_outputs = normalize_path(&fixture_root.join("outputs/scenario_1"));
        if candidate.starts_with(&fixture_outputs) {
            return Ok(candidate);
        }
    }

    Err(Scenario1Err::InvalidOutputSandbox(
        candidate.display().to_string(),
    ))
}

fn resolve_output_candidate(dir: &str) -> PathBuf {
    let raw = PathBuf::from(dir);
    if raw.is_absolute() {
        normalize_path(&raw)
    } else {
        normalize_path(&workspace_root().join(raw))
    }
}

fn approved_fixture_root(cfg_path: &Path) -> Option<PathBuf> {
    let fixture_root = resolve_workspace_path(cfg_path.parent()?);
    let workspace_target = workspace_target_root();
    let workspace_fixture_cache = scenario_cache_root();
    let is_temp_fixture = fixture_root
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(".tmp") || name.contains(".tmp."));

    if fixture_root.starts_with(&workspace_target)
        || fixture_root.starts_with(&workspace_fixture_cache)
        || is_temp_fixture
    {
        Some(fixture_root)
    } else {
        None
    }
}

pub(crate) fn storage_runtime_fingerprint() -> String {
    static VALUE: OnceLock<String> = OnceLock::new();
    VALUE
        .get_or_init(|| {
            let workspace_root = workspace_root();
            // INVARIANT: fingerprint inputs are checked-in workspace paths resolved from CARGO_MANIFEST_DIR.
            hash_root_inputs(
                "scenario-runtime-artifacts-v1",
                &[
                    workspace_root.join("Cargo.toml"),
                    workspace_root.join("Cargo.lock"),
                    workspace_root.join(".cargo/config.toml"),
                    workspace_root.join("crates/z00z_core/Cargo.toml"),
                    workspace_root.join("crates/z00z_crypto/Cargo.toml"),
                    workspace_root.join("crates/z00z_networks/rpc/Cargo.toml"),
                    workspace_root.join("crates/z00z_simulator/Cargo.toml"),
                    workspace_root.join("crates/z00z_storage/Cargo.toml"),
                    workspace_root.join("crates/z00z_utils/Cargo.toml"),
                    workspace_root.join("crates/z00z_wallets/Cargo.toml"),
                ],
                &[
                    workspace_root.join("crates/z00z_core/src"),
                    workspace_root.join("crates/z00z_crypto/src"),
                    workspace_root.join("crates/z00z_networks/rpc/src"),
                    workspace_root.join("crates/z00z_simulator/src"),
                    workspace_root.join("crates/z00z_storage/src"),
                    workspace_root.join("crates/z00z_utils/src"),
                    workspace_root.join("crates/z00z_wallets/src"),
                ],
            )
            .expect("BUG: scenario runtime artifacts must hash")
        })
        .clone()
}

fn build_stage_map() -> BTreeMap<u32, StageFn> {
    let mut stage_map = BTreeMap::new();
    stage_map.insert(1, stage_1::run as _);
    stage_map.insert(2, stage_2::run as _);
    stage_map.insert(3, stage_3::run_claim_prepare as _);
    stage_map.insert(4, stage_4::run_claim_publish as _);
    stage_map.insert(5, stage_5::run_tx_plan as _);
    stage_map.insert(6, stage_6::run_tx_prepare as _);
    stage_map.insert(7, stage_7::run_transfer_receive as _);
    stage_map.insert(8, stage_8::run_transfer_claim as _);
    stage_map.insert(9, stage_9::run_bundle_build as _);
    stage_map.insert(10, stage_10::run_bundle_publish as _);
    stage_map.insert(11, stage_11::run_apply as _);
    stage_map.insert(12, stage_12::run_finalize as _);
    stage_map.insert(13, stage_13::run_hjmt_examples as _);
    stage_map
}

fn build_ctx(cfg: ScenarioCfg, out_dir: PathBuf, chain_type: ChainType) -> SimContext {
    let reg = AssetDefinitionRegistry::new(
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    SimContext {
        config: cfg,
        chain_type,
        registry: reg,
        assets: Vec::new(),
        genesis_rights: Vec::new(),
        actors: Vec::new(),
        leaves: Vec::new(),
        block_height: 0,
        outputs_dir: out_dir,
        logger: Arc::new(StdoutLogger),
        wallet_service: None,
    }
}

fn parse_chain_type(chain: &str) -> Result<ChainType, Scenario1Err> {
    chain
        .parse::<ChainType>()
        .map_err(|_| Scenario1Err::InvalidChain(chain.to_string()))
}

fn run_stage_plan(
    logger: &impl Logger,
    ctx: &mut SimContext,
    design: &DesignDoc,
    stage_map: &BTreeMap<u32, StageFn>,
) -> ScenarioResult {
    let mut out = ScenarioResult::new(ctx.config.scenario.id);

    for stage in &design.stages {
        let stage_started = Instant::now();
        let result = run_stage(ctx, stage, stage_map);
        let stage_elapsed_ms = stage_started.elapsed().as_millis();

        log_stage(logger, stage, &result);
        logger.info(&format!(
            "stage.profile: id={}, name={}, elapsed_ms={}, result={}",
            stage.stage,
            stage.name,
            stage_elapsed_ms,
            stage_result_tag(&result)
        ));

        let is_fail = matches!(result, StageResult::Fail(_));
        out.stages.push(StageState {
            stage: stage.stage,
            name: stage.name.clone(),
            result,
        });

        if is_fail && ctx.config.simulation.abort_on_fail {
            out.is_aborted = true;
            break;
        }
    }

    out
}

fn run_stage(
    ctx: &mut SimContext,
    stage: &DesignStage,
    stage_map: &BTreeMap<u32, StageFn>,
) -> StageResult {
    let result = match stage_map.get(&stage.stage) {
        Some(stage_fn) => stage_fn(ctx, stage),
        None => missing_stage(stage),
    };

    if !matches!(result, StageResult::Ok) {
        return result;
    }

    check_stage(ctx, stage)
}

fn missing_stage(stage: &DesignStage) -> StageResult {
    StageResult::Fail(format!(
        "stage {} ({}) has no implementation",
        stage.stage, stage.name
    ))
}

fn stage_result_tag(result: &StageResult) -> &'static str {
    match result {
        StageResult::Ok => "ok",
        StageResult::Warn(_) => "warn",
        StageResult::Fail(_) => "fail",
    }
}

fn teardown_wallet_runtime(ctx: &mut SimContext) -> Result<(), String> {
    if ctx.wallet_service.is_none() {
        for actor in &mut ctx.actors {
            actor.session = None;
        }
        return Ok(());
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("tokio runtime: {e}"))?;
    rt.block_on(async {
        crate::scenario_1::stage_2::lock_existing_wallet_sessions(ctx)
            .await
            .map_err(|e| format!("lock existing wallet sessions: {e}"))
    })?;

    ctx.wallet_service = None;
    for actor in &mut ctx.actors {
        actor.session = None;
    }

    Ok(())
}

/// Entrypoint used by scenario-specific bin wrapper.
pub fn main() -> Result<(), Scenario1Err> {
    let logger = StdoutLogger;
    let result = run()?;

    if result.is_ok() {
        logger.info(&format!(
            "scenario_1.result: success, scenario_id={}",
            result.scenario_id
        ));
    } else {
        logger.warn(&format!(
            "scenario_1.result: warnings_or_failures, scenario_id={}, stages={}",
            result.scenario_id,
            result.stages.len()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};
    use std::sync::{Mutex, OnceLock};
    use z00z_utils::io::{create_dir_all, remove_dir_all, write_file};

    fn runner_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn missing_stage_fails_closed() {
        let stage = DesignStage {
            stage: 13,
            name: "hjmt_settlement_examples".to_string(),
            description: Some("stage 13 scaffold".to_string()),
            rust_entry: Some("stage_13::run_hjmt_examples(ctx, stage)".to_string()),
            config_source: Some(
                "scenario_config.yaml::stage13_hjmt_settlement_examples".to_string(),
            ),
            steps: Vec::new(),
        };

        assert!(matches!(
            missing_stage(&stage),
            StageResult::Fail(msg) if msg.contains("has no implementation")
        ));
    }

    #[test]
    fn reset_outputs_clears_stale_tree() {
        let _guard = runner_test_lock()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("outputs/scenario_1/tests/reset_root_contract");
        if root.exists() {
            remove_dir_all(&root).expect("clear prior test root");
        }
        create_dir_all(root.join("nested")).expect("create nested dir");
        write_file(root.join("nested/stale.txt"), b"stale").expect("write stale file");

        reset_outputs_dir(&root.to_string_lossy()).expect("reset outputs dir");

        assert!(root.exists(), "output root must still exist");
        assert!(
            !root.join("nested/stale.txt").exists(),
            "reset must remove stale output payload"
        );

        remove_dir_all(&root).expect("cleanup reset root");
    }

    #[test]
    fn recreate_scope_clears_stale_tree() {
        let _guard = runner_test_lock()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("outputs/scenario_1/tests/storage_scope_contract");
        if root.exists() {
            remove_dir_all(&root).expect("clear prior storage scope root");
        }
        create_dir_all(root.join("nested")).expect("create nested storage scope dir");
        write_file(root.join("nested/stale.txt"), b"stale").expect("write stale storage scope");

        recreate_storage_scope(&root).expect("recreate storage scope");

        assert!(root.exists(), "storage scope root must still exist");
        assert!(
            !root.join("nested/stale.txt").exists(),
            "storage scope reset must remove stale payload"
        );

        remove_dir_all(&root).expect("cleanup storage scope root");
    }

    #[test]
    fn recreate_scope_prunes_alias_legacy() {
        let _guard = runner_test_lock()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("outputs/scenario_1/tests/storage_scope_alias_contract");
        let root = parent.join("test_scope");
        let alias = parent.join("test_scope-9d4bd19d594c355f");
        let legacy = parent.join("0123456789abcdef");
        if parent.exists() {
            remove_dir_all(&parent).expect("clear prior storage scope alias root");
        }
        create_dir_all(alias.join("nested")).expect("create stale alias dir");
        create_dir_all(legacy.join("nested")).expect("create stale legacy hash dir");
        write_file(alias.join("nested/stale.txt"), b"stale").expect("write stale alias marker");
        write_file(legacy.join("nested/stale.txt"), b"stale").expect("write stale legacy marker");

        recreate_storage_scope(&root).expect("recreate storage scope with alias cleanup");

        assert!(root.exists(), "storage scope root must still exist");
        assert!(
            !alias.exists(),
            "storage scope rebuild must drop stale hash-suffixed alias dirs"
        );
        assert!(
            !legacy.exists(),
            "storage scope rebuild must drop legacy top-level hash dirs"
        );

        remove_dir_all(&parent).expect("cleanup storage scope alias root");
    }

    #[test]
    fn clear_foreign_live_lock() {
        let _guard = runner_test_lock()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("outputs/scenario_1/tests/run_lock_foreign_pid_contract");
        if parent.exists() {
            remove_dir_all(&parent).expect("clear prior run lock contract root");
        }
        create_dir_all(&parent).expect("create run lock contract root");
        let lock_path = parent.join("foreign.lock");

        let mut child = Command::new("sleep")
            .arg("30")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn foreign live process");
        write_file(
            &lock_path,
            format!("{}\n/tmp/foreign/outputs/scenario_1\n", child.id()).as_bytes(),
        )
        .expect("write synthetic foreign run lock");

        let cleared =
            clear_stale_scenario_run_lock(&lock_path).expect("clear foreign live pid run lock");

        assert!(cleared, "foreign live pid run lock must be cleared");
        assert!(
            !lock_path.exists(),
            "foreign live pid run lock file must be removed"
        );

        let _ = child.kill();
        let _ = child.wait();
        if parent.exists() {
            remove_dir_all(&parent).expect("cleanup run lock contract root");
        }
    }

    #[test]
    fn clear_missing_lock_reports_retry() {
        let _guard = runner_test_lock()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("outputs/scenario_1/tests/run_lock_missing_contract");
        if parent.exists() {
            remove_dir_all(&parent).expect("clear prior missing run lock contract root");
        }
        create_dir_all(&parent).expect("create missing run lock contract root");
        let lock_path = parent.join("missing.lock");

        let cleared = clear_stale_scenario_run_lock(&lock_path)
            .expect("missing run lock must report cleared for retry");

        assert!(cleared, "missing run lock must be treated as retryable");

        if parent.exists() {
            remove_dir_all(&parent).expect("cleanup missing run lock contract root");
        }
    }

    #[test]
    fn workspace_target_output_is_allowed() {
        let out = "target/scenario_1/tests/workspace_target_contract";
        let resolved = validate_output_sandbox(out).expect("workspace target must be allowed");
        assert_eq!(
            resolved,
            workspace_target_root().join("scenario_1/tests/workspace_target_contract")
        );
    }

    #[test]
    fn simulator_local_target_output_is_rejected() {
        let err = validate_output_sandbox("crates/z00z_simulator/target/local_target_contract")
            .expect_err("crate-local simulator target must be rejected");

        assert!(matches!(
            err,
            Scenario1Err::InvalidOutputSandbox(path)
                if path.ends_with("crates/z00z_simulator/target/local_target_contract")
        ));
    }
}
