#![allow(clippy::duplicate_mod)]

use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};

use super::path_roots::{resolve_workspace_path, workspace_root};
use crate::{config::ScenarioCfg, ScenarioResult, StageResult};
use z00z_utils::{
    codec::{Codec, JsonCodec, Value, YamlCodec},
    io::{
        create_dir_all, current_exe_run_root, hash_root_inputs, path_exists,
        prune_scope_alias_dirs, read_dir, read_to_string, remove_dir_all, reset_managed_root_once,
        stable_current_exe_scope, write_file,
    },
};

const ALLOW_DEBUG_RANGE_PROOF: &str = "Z00Z_ALLOW_DEBUG_RANGE_PROOF";
const CARGO_TARGET_DIR_ENV: &str = "CARGO_TARGET_DIR";
const RUNTIME_CWD_ROOT_ENV: &str = "Z00Z_RUNTIME_CWD_ROOT";
const STORAGE_ROOT_BASE_ENV: &str = "Z00Z_STORAGE_REDB_ROOT_BASE";
const SCENARIO_CACHE_ROOT_ENV: &str = "Z00Z_SIMULATOR_CACHE_ROOT";
const SCENARIO_STORAGE_ROOT_ENV: &str = "Z00Z_SIMULATOR_STORAGE_ROOT";
const VERIFICATION_RUN_ROOT_ENV: &str = "Z00Z_VERIFICATION_RUN_ROOT";

pub fn repo_root() -> PathBuf {
    workspace_root()
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
        .unwrap_or_else(|| repo_root().join(".cache").join("scenario_1"))
}

fn scenario_storage_root() -> PathBuf {
    std::env::var_os(SCENARIO_STORAGE_ROOT_ENV)
        .map(PathBuf::from)
        .map(resolve_workspace_path)
        .or_else(|| runtime_cache_root_base().map(|root| root.join("storage").join("scenario_1")))
        .unwrap_or_else(|| {
            repo_root()
                .join(".cache")
                .join("storage")
                .join("scenario_1")
        })
}

pub fn make_cfg(edit_cfg: impl FnOnce(&mut ScenarioCfg)) -> (PathBuf, PathBuf, PathBuf) {
    let base = fresh_ad_hoc_case_root();
    make_cfg_in(&base, edit_cfg)
}

pub fn make_cfg_in(
    base: &Path,
    edit_cfg: impl FnOnce(&mut ScenarioCfg),
) -> (PathBuf, PathBuf, PathBuf) {
    // Scenario integration tests exercise the simulator tx lane in debug mode.
    // Keep the guard local to test support rather than weakening production code.
    std::env::set_var(ALLOW_DEBUG_RANGE_PROOF, "1");
    install_storage_override_once();

    let base = resolve_workspace_path(base);
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

    cfg.simulation.use_mock_rng = true;
    cfg.simulation.mock_rng_seed = Some(42);
    for asset in &mut cfg.genesis_assets {
        asset.serials = asset.serials.min(6);
    }

    if let Some(stage3) = cfg.stage3_claim.as_mut() {
        stage3.consume_bins = Some(false);
        stage3.rng_seed = Some(42);
    }

    if let Some(stage4) = cfg.stage4_tx_prepare.as_mut() {
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
            .distinct_serial_ids_max = 4;
        stage4.transaction.outputs.bob_outputs_count = 4;
        stage4.transaction.class = "Coin".to_string();
        stage4.transaction.symbol = "Z00Z".to_string();
        stage4.transaction.mode = "fraction".to_string();
        stage4.transaction.fraction = Some(0.1);
        stage4.transaction.amount = None;
    }

    edit_cfg(&mut cfg);

    let cfg_path = base.join("scenario_config.yaml");
    let cfg_bytes = YamlCodec.serialize(&cfg).expect("cfg bytes");
    write_file(&cfg_path, &cfg_bytes).expect("write cfg");

    let design_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_design.yaml");
    (cfg_path, design_path, out)
}

fn install_storage_override_once() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        let scope_name = process_scope_name();
        let storage_root_base = scenario_storage_root().join("test_bins").join(&scope_name);
        prune_test_storage_scope(&storage_root_base);
        reset_managed_root_once(&storage_root_base, &storage_test_fingerprint(), &[], None)
            .expect("reset managed test storage root");
        std::env::set_var(STORAGE_ROOT_BASE_ENV, &storage_root_base);
    });
}

fn exe_scope() -> String {
    stable_current_exe_scope("unknown_test_binary")
}

fn process_scope_name() -> String {
    format!("{}-pid-{}", exe_scope(), std::process::id())
}

fn prune_dead_process_scope_dirs(parent: &Path, prefix: &str) {
    if !path_exists(parent).unwrap_or(false) {
        return;
    }

    let current_pid = std::process::id();
    for entry in read_dir(parent).expect("read process scope dir") {
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
        if proc_path.exists() {
            continue;
        }
        remove_dir_all(&entry).expect("remove stale process scope dir");
    }
}

fn prune_test_storage_scope(scope_root: &Path) {
    let storage_scope_root = scope_root
        .parent()
        .expect("test storage scope parent missing");
    let exe_scope = exe_scope();
    let process_prefix = format!("{exe_scope}-pid-");
    prune_dead_process_scope_dirs(storage_scope_root, &process_prefix);
    let storage_scope = scope_root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("test storage scope name");
    prune_scope_alias_dirs(storage_scope_root, storage_scope)
        .expect("prune stale test storage scope aliases");
}

fn fresh_ad_hoc_case_root() -> PathBuf {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    static INIT: OnceLock<()> = OnceLock::new();

    let base = scenario_cache_root()
        .join("adhoc")
        .join(process_scope_name());
    INIT.get_or_init(|| {
        let parent = base.parent().expect("ad hoc fixture parent");
        let exe_scope = exe_scope();
        let process_prefix = format!("{exe_scope}-pid-");
        prune_dead_process_scope_dirs(parent, &process_prefix);
        if path_exists(&base).unwrap_or(false) {
            remove_dir_all(&base).expect("remove stale ad hoc fixture root");
        }
        create_dir_all(&base).expect("create ad hoc fixture root");
    });

    let pid = std::process::id();
    for _ in 0..1_000 {
        let next_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let candidate = base.join(format!("case-{pid}-{next_id}"));
        if !path_exists(&candidate).unwrap_or(false) {
            create_dir_all(&candidate).expect("create ad hoc fixture case");
            return candidate;
        }
    }

    panic!("allocate ad hoc fixture root under {}", base.display());
}

pub(crate) fn storage_test_fingerprint() -> String {
    static VALUE: OnceLock<String> = OnceLock::new();
    VALUE
        .get_or_init(|| {
            let root = repo_root();
            hash_root_inputs(
                "scenario-test-artifacts-v1",
                &[
                    root.join("Cargo.toml"),
                    root.join("Cargo.lock"),
                    root.join(".cargo/config.toml"),
                    root.join("crates/z00z_simulator/Cargo.toml"),
                    root.join("crates/z00z_storage/Cargo.toml"),
                    root.join("crates/z00z_wallets/Cargo.toml"),
                    root.join("crates/z00z_utils/Cargo.toml"),
                ],
                &[
                    root.join("crates/z00z_simulator/src"),
                    root.join("crates/z00z_simulator/tests"),
                    root.join("crates/z00z_storage/src"),
                    root.join("crates/z00z_wallets/src"),
                    root.join("crates/z00z_utils/src"),
                ],
            )
            .expect("hash scenario test artifacts")
        })
        .clone()
}

pub fn stage_res(run: &ScenarioResult, stage_id: u32) -> &StageResult {
    &run.stages
        .iter()
        .find(|item| item.stage == stage_id)
        .expect("stage present")
        .result
}

fn parse_rpc_log_line(line: &str) -> Option<Value> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(value) = JsonCodec.deserialize(trimmed.as_bytes()) {
        return Some(value);
    }

    let json_start = trimmed.find('{')?;
    JsonCodec
        .deserialize(&trimmed.as_bytes()[json_start..])
        .ok()
}

pub fn read_rpc_req_rows(out: &Path) -> Vec<Value> {
    read_to_string(out.join("logs/rpc_logger.json"))
        .expect("read rpc log")
        .lines()
        .filter_map(parse_rpc_log_line)
        .filter(|row| row["event"].as_str() == Some("rpc.request"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prune_scope_removes_hash_aliases() {
        let root = scenario_storage_root()
            .join("test_bins")
            .join("scenario_support_scope_contract");
        let parent = root.parent().expect("scope parent").to_path_buf();
        let alias = parent.join("scenario_support_scope_contract-9d4bd19d594c355f");
        if root.exists() {
            remove_dir_all(&root).expect("clear scenario support contract root");
        }
        if alias.exists() {
            remove_dir_all(&alias).expect("clear scenario support contract alias");
        }
        create_dir_all(alias.join("nested")).expect("create scenario support alias");
        write_file(alias.join("nested/stale.txt"), b"stale").expect("write scenario support alias");

        prune_test_storage_scope(&root);

        assert!(
            !alias.exists(),
            "test storage cleanup must remove stale hash-suffixed scope aliases"
        );

        if root.exists() {
            remove_dir_all(&root).expect("cleanup scenario support contract root");
        }
    }
}
