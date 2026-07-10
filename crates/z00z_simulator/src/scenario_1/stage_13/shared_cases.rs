use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    thread,
    time::Duration,
};

use crate::{config::ScenarioCfg, scenario_1::runner, StageResult};
use z00z_utils::{
    codec::{Codec, JsonCodec, Value},
    io::{
        create_dir_all, path_exists, read_file, read_to_string, remove_dir_all, remove_file,
        write_file,
    },
};

use crate::scenario_1::support::{fixture_cache, scenario_support};

const FULL_STAGE13_CASE: &str = "scenario1_full_stage13_shared_v4";
const FULL_STAGE13_IDS: &[u32] = &[1_u32, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
const FULL_STAGE13_BUILD_RETRIES: usize = 5;
const STAGE13_STABLE_MARKER_FILE: &str = ".stage13-stable";
const STAGE13_STABLE_SCHEMA: &str = "stage13-shared-stable-v2";
const STAGE13_LOCAL_CASE_PREFIX: &str = "scenario1_full_stage13_localized_v2";
const STAGE13_ARTIFACT_READY_RETRIES: usize = 100;
const STAGE13_ARTIFACT_READY_WAIT_MS: u64 = 100;

fn validate_stage13_run(run: &crate::ScenarioResult) -> Result<(), String> {
    for &stage_id in FULL_STAGE13_IDS {
        let state = run
            .stages
            .iter()
            .find(|item| item.stage == stage_id)
            .ok_or_else(|| format!("shared stage13 missing stage {stage_id}"))?;
        match &state.result {
            StageResult::Ok => {}
            StageResult::Fail(message) => {
                return Err(format!(
                    "shared stage13 stage {stage_id} failed: {}",
                    message
                ));
            }
            StageResult::Warn(message) => {
                return Err(format!(
                    "shared stage13 stage {stage_id} warned: {}",
                    message
                ));
            }
        }
    }

    Ok(())
}

fn stabilize_stage13_root(root: &Path) -> Result<(), String> {
    let _case_lock = fixture_cache::acquire_case_lock(root)?;
    stabilize_stage13_root_locked(root)
}

fn shared_stage13_validation_paths(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let cfg_path = root.join("scenario_config.yaml");
    let design_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_design.yaml");
    let out = root.join("outputs/scenario_1");
    (cfg_path, design_path, out)
}

fn stabilize_stage13_root_locked(root: &Path) -> Result<(), String> {
    let (cfg_path, design_path, out) = shared_stage13_validation_paths(root);

    rewrite_stage13_trace_cfg_paths(root)?;
    validate_stage13_cached_root(&cfg_path, &design_path, &out)?;
    mark_stage13_root_stable(root)?;
    fixture_cache::refresh_case_content_fingerprint(root)
        .map_err(|err| format!("shared stage13 cache fingerprint refresh failed: {err}"))?;

    Ok(())
}

fn reset_stage13_build_root(root: &Path) {
    if path_exists(root).unwrap_or(false) {
        remove_dir_all(root).expect("remove stale shared stage13 build root");
    }
    create_dir_all(root).expect("create shared stage13 build root");
}

fn build_shared_stage13_root(base: &Path) -> Result<(), String> {
    create_dir_all(base).map_err(|err| {
        format!(
            "create shared stage13 base root {} failed: {err}",
            base.display()
        )
    })?;

    let mut last_attempt_err = None;
    for attempt in 1..=FULL_STAGE13_BUILD_RETRIES {
        reset_stage13_build_root(base);
        let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, |_| {});
        match runner::run_with_paths(&cfg_path, &design_path) {
            Ok(run) => match validate_and_stabilize_stage13_attempt(
                base,
                &cfg_path,
                &design_path,
                &out,
                &run,
            ) {
                Ok(()) => {
                    stabilize_stage13_root_locked(base)?;
                    return Ok(());
                }
                Err(message) if attempt < FULL_STAGE13_BUILD_RETRIES => {
                    last_attempt_err = Some(format!("attempt {attempt}: {message}"));
                    continue;
                }
                Err(message) => return Err(message),
            },
            Err(err) => {
                let message = format!("shared stage13 scenario run must succeed: {err}");
                if attempt < FULL_STAGE13_BUILD_RETRIES {
                    last_attempt_err = Some(format!("attempt {attempt}: {message}"));
                    continue;
                }
                return Err(message);
            }
        }
    }

    Err(format!(
        "shared stage13 build exhausted {} retry attempts{}",
        FULL_STAGE13_BUILD_RETRIES,
        last_attempt_err
            .map(|message| format!("; last error: {message}"))
            .unwrap_or_default()
    ))
}

fn refresh_shared_stage13_root(root: &Path) -> Result<(), String> {
    let _case_lock = fixture_cache::acquire_case_lock(root)?;
    let (cfg_path, design_path, out) = shared_stage13_validation_paths(root);

    if stage13_root_is_stable(root) {
        let current = rewrite_stage13_trace_cfg_paths(root)
            .and_then(|_| validate_stage13_cached_root(&cfg_path, &design_path, &out))
            .and_then(|_| mark_stage13_root_stable(root))
            .and_then(|_| {
                fixture_cache::refresh_case_content_fingerprint(root).map_err(|err| {
                    format!("shared stage13 cache fingerprint refresh failed: {err}")
                })
            });
        if current.is_ok() {
            return Ok(());
        }
    }

    clear_stage13_root_stable(root);
    build_shared_stage13_root(root)
}

fn mark_stage13_root_stable(root: &Path) -> Result<(), String> {
    write_file(
        root.join(STAGE13_STABLE_MARKER_FILE),
        STAGE13_STABLE_SCHEMA.as_bytes(),
    )
    .map_err(|err| format!("write shared stage13 stable marker failed: {err}"))
}

fn clear_stage13_root_stable(root: &Path) {
    let _ = remove_file(root.join(STAGE13_STABLE_MARKER_FILE));
}

fn stage13_root_is_stable(root: &Path) -> bool {
    read_to_string(root.join(STAGE13_STABLE_MARKER_FILE))
        .ok()
        .is_some_and(|text| text.trim() == STAGE13_STABLE_SCHEMA)
}

fn ensure_stage13_root_stable(root: &Path) -> Result<(), String> {
    if stage13_root_is_stable(root) {
        return Ok(());
    }

    let _case_lock = fixture_cache::acquire_case_lock(root)?;
    if stage13_root_is_stable(root) {
        return Ok(());
    }

    stabilize_stage13_root_locked(root)
}

fn localized_stage13_case_name(shared_root: &Path) -> Result<String, String> {
    let fingerprint = read_to_string(shared_root.join(".content-fingerprint")).map_err(|err| {
        format!(
            "read shared stage13 content fingerprint {} failed: {err}",
            shared_root.display()
        )
    })?;
    let short = fingerprint
        .trim()
        .get(..16)
        .ok_or_else(|| "shared stage13 content fingerprint too short".to_string())?;
    Ok(format!("{STAGE13_LOCAL_CASE_PREFIX}_{short}"))
}

fn sanitize_case_suffix(case_suffix: &str) -> Result<String, String> {
    let trimmed = case_suffix.trim();
    if trimmed.is_empty() {
        return Err("localized stage13 case suffix must not be empty".to_string());
    }

    let sanitized: String = trimmed
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect();
    if sanitized.chars().all(|ch| ch == '_') {
        return Err(
            "localized stage13 case suffix must contain alphanumeric characters".to_string(),
        );
    }

    Ok(sanitized)
}

fn stage13_case_name(shared_root: &Path, case_suffix: &str) -> Result<String, String> {
    let base = localized_stage13_case_name(shared_root)?;
    let suffix = sanitize_case_suffix(case_suffix)?;
    Ok(format!("{base}_{suffix}"))
}

fn rewrite_stage13_trace_cfg_paths(root: &Path) -> Result<(), String> {
    let cfg_path = root.join("scenario_config.yaml");
    let out = root.join("outputs/scenario_1");
    wait_for_stage13_artifacts(&out)?;
    let cfg = ScenarioCfg::from_file(&cfg_path)
        .map_err(|err| format!("shared stage13 config must load: {err}"))?;
    let observability = cfg
        .runtime_observability_ref()
        .ok_or_else(|| "shared stage13 runtime_observability config missing".to_string())?;
    let traces = observability.traces.clone();
    let packet = observability.packet.clone();
    let cfg_path_text = cfg_path.to_string_lossy().to_string();
    let prior_out = normalize_path(std::path::Path::new(&cfg.outputs.dir));
    let stable_out = normalize_path(&out);

    let mut rels = vec![
        traces.cfg_flow_file,
        traces.tx_flow_file,
        traces.route_flow_file,
        traces.plan_flow_file,
        traces.journal_flow_file,
        traces.scope_flow_file,
        traces.proc_flow_file,
        traces.recovery_flow_file,
        traces.leaf_flow_file,
        traces.proof_flow_file,
        traces.pub_flow_file,
        traces.val_flow_file,
        traces.watch_flow_file,
    ];
    rels.extend(packet.emitted_public_files);
    rels.sort();
    rels.dedup();

    for rel in rels {
        let path = out.join(&rel);
        let bytes = read_stage13_file_with_retry(&path, "trace")?;
        let mut value: Value = JsonCodec.deserialize(&bytes).map_err(|err| {
            format!(
                "decode shared stage13 trace {} failed: {err}",
                path.display()
            )
        })?;
        value["scenario_config_path"] = Value::String(cfg_path_text.clone());
        if let Some(records) = value["config_digests"].as_array_mut() {
            for record in records {
                if record["label"].as_str() == Some("scenario-config") {
                    record["path"] = Value::String(cfg_path_text.clone());
                }
            }
        }
        for field in [
            "tx_package_path",
            "transfer_leaf_path",
            "bundle_frag1_path",
            "bundle_frag2_path",
            "bundle_bridge_path",
            "checkpoint_apply_path",
            "checkpoint_finalize_path",
            "hjmt_examples_report_path",
        ] {
            if let Some(path_text) = value[field].as_str() {
                if let Some(rebased) = rebase_trace_output_path(path_text, &prior_out, &stable_out)
                {
                    value[field] = Value::String(rebased);
                }
            }
        }
        let body = JsonCodec.serialize(&value).map_err(|err| {
            format!(
                "encode shared stage13 trace {} failed: {err}",
                path.display()
            )
        })?;
        write_file(&path, &body).map_err(|err| {
            format!(
                "rewrite shared stage13 trace {} failed: {err}",
                path.display()
            )
        })?;
    }

    Ok(())
}

fn rebase_trace_output_path(
    path_text: &str,
    prior_out: &Path,
    stable_out: &Path,
) -> Option<String> {
    let normalized = normalize_path(Path::new(path_text));
    if let Ok(rel) = normalized.strip_prefix(prior_out) {
        return Some(stable_out.join(rel).to_string_lossy().to_string());
    }

    let rel = strip_scenario_output_prefix(&normalized)?;
    Some(stable_out.join(rel).to_string_lossy().to_string())
}

fn strip_scenario_output_prefix(path: &Path) -> Option<PathBuf> {
    let components: Vec<_> = path.components().collect();
    for index in 0..components.len().saturating_sub(1) {
        if components[index].as_os_str() == OsStr::new("outputs")
            && components[index + 1].as_os_str() == OsStr::new("scenario_1")
        {
            let mut rel = PathBuf::new();
            for component in &components[index + 2..] {
                rel.push(component.as_os_str());
            }
            return Some(rel);
        }
    }
    None
}

fn normalize_path(path: &Path) -> std::path::PathBuf {
    let mut normalized = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn validate_stage13_cached_root(
    cfg_path: &Path,
    design_path: &Path,
    out: &Path,
) -> Result<(), String> {
    runner::validate_runtime_observability_artifacts(cfg_path, design_path, out)
        .map_err(|err| format!("stage13 shared case runtime trace pack must validate: {err}"))
}

fn stage13_out_needs_repair(out: &Path) -> bool {
    !out.join("hjmt/hjmt_settlement_examples.json").exists()
        || !out.join("transactions/tx_alice_to_bob_pkg.json").exists()
        || !out.join("stage_4_snapshot.json").exists()
        || !out.join("asset_flow.json").exists()
        || !out.join("voucher_flow.json").exists()
        || !out.join("right_flow.json").exists()
}

fn wait_for_stage13_artifacts(out: &Path) -> Result<(), String> {
    for attempt in 1..=STAGE13_ARTIFACT_READY_RETRIES {
        if !stage13_out_needs_repair(out) {
            return Ok(());
        }
        if attempt < STAGE13_ARTIFACT_READY_RETRIES {
            thread::sleep(Duration::from_millis(STAGE13_ARTIFACT_READY_WAIT_MS));
            continue;
        }
    }

    Err(format!(
        "shared stage13 artifacts not ready under {} after {} waits",
        out.display(),
        STAGE13_ARTIFACT_READY_RETRIES
    ))
}

fn read_stage13_file_with_retry(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    let mut last_err = None;
    for attempt in 1..=STAGE13_ARTIFACT_READY_RETRIES {
        match read_file(path) {
            Ok(bytes) => return Ok(bytes),
            Err(err) => {
                last_err = Some(err);
                if attempt < STAGE13_ARTIFACT_READY_RETRIES {
                    thread::sleep(Duration::from_millis(STAGE13_ARTIFACT_READY_WAIT_MS));
                    continue;
                }
            }
        }
    }

    Err(format!(
        "read shared stage13 {label} {} failed: {}",
        path.display(),
        last_err
            .map(|err| err.to_string())
            .unwrap_or_else(|| "unknown read error".to_string())
    ))
}

fn validate_and_stabilize_stage13_attempt(
    _root: &Path,
    _cfg_path: &Path,
    _design_path: &Path,
    _out: &Path,
    run: &crate::ScenarioResult,
) -> Result<(), String> {
    validate_stage13_run(run)?;
    // `runner::run_with_paths(...)` already emits and strict-validates the
    // fresh runtime packet before it returns success. Shared-case promotion must
    // not re-run that same packet walk on the attempt root; the canonical
    // path-rebased validation belongs only to `stabilize_stage13_root_locked(...)`
    // after promotion or localization changes the packet anchor.
    Ok(())
}

fn prepare_localized_stage13_root(shared_root: &Path, case_name: &str) -> Result<PathBuf, String> {
    let shared_root = shared_root.to_path_buf();
    let local_root = fixture_cache::ensure_case(case_name, |base| {
        // Serialize localization against shared-root refresh/promote so parallel
        // test binaries never snapshot a partially rebuilt Stage 13 packet.
        let _shared_case_lock = fixture_cache::acquire_case_lock(&shared_root)
            .unwrap_or_else(|err| panic!("acquire shared stage13 cache lock failed: {err}"));
        fixture_cache::copy_tree(&shared_root, base);
    });
    clear_stage13_root_stable(&local_root);
    if let Err(first_err) = stabilize_stage13_root(&local_root) {
        let _shared_case_lock = fixture_cache::acquire_case_lock(&shared_root)
            .unwrap_or_else(|err| panic!("acquire shared stage13 cache lock failed: {err}"));
        fixture_cache::copy_tree(&shared_root, &local_root);
        clear_stage13_root_stable(&local_root);
        stabilize_stage13_root(&local_root).map_err(|second_err| {
            format!(
                "localized stage13 root repair failed for {}: first={first_err}; second={second_err}",
                local_root.display()
            )
        })?;
    }
    Ok(local_root)
}

fn ensure_shared_stage13_root() -> PathBuf {
    let shared_root = fixture_cache::ensure_shared_case_precise(FULL_STAGE13_CASE, |base| {
        build_shared_stage13_root(base).unwrap_or_else(|message| panic!("{message}"));
    });
    refresh_shared_stage13_root(&shared_root)
        .expect("refresh promoted shared stage13 root remains stable");
    ensure_stage13_root_stable(&shared_root)
        .expect("ensure promoted shared stage13 root remains stable");
    shared_root
}

fn build_full_stage13_out() -> PathBuf {
    let shared_root = ensure_shared_stage13_root();
    let local_case_name =
        localized_stage13_case_name(&shared_root).expect("derive localized shared stage13 case");
    let local_root = prepare_localized_stage13_root(&shared_root, &local_case_name)
        .expect("prepare localized stage13 root");
    local_root.join("outputs/scenario_1")
}

pub fn full_stage13_out() -> PathBuf {
    static OUT: OnceLock<Mutex<PathBuf>> = OnceLock::new();
    let out = OUT.get_or_init(|| Mutex::new(build_full_stage13_out()));
    let mut guard = out.lock().unwrap_or_else(|poison| poison.into_inner());
    if stage13_out_needs_repair(&guard) {
        *guard = build_full_stage13_out();
    }
    guard.clone()
}

pub fn stage13_out(case_suffix: &str) -> PathBuf {
    let shared_root = ensure_shared_stage13_root();
    let local_case_name = stage13_case_name(&shared_root, case_suffix)
        .expect("derive localized stage13 case with suffix");
    let local_root = prepare_localized_stage13_root(&shared_root, &local_case_name)
        .expect("prepare localized stage13 root with suffix");
    let out = local_root.join("outputs/scenario_1");
    if stage13_out_needs_repair(&out) {
        let local_root = prepare_localized_stage13_root(&shared_root, &local_case_name)
            .expect("repair localized stage13 root with suffix");
        return local_root.join("outputs/scenario_1");
    }
    out
}

pub fn shared_full_stage13_out() -> PathBuf {
    ensure_shared_stage13_root().join("outputs/scenario_1")
}

#[cfg(test)]
mod tests {
    use super::reset_stage13_build_root;
    use std::path::PathBuf;
    use z00z_utils::io::{create_dir_all, path_exists, remove_dir_all, write_file};
    use z00z_utils::time::{SystemTimeProvider, TimeProvider};

    fn unique_attempt_root() -> PathBuf {
        let time = SystemTimeProvider;
        let nonce = time
            .try_unix_timestamp_micros()
            .expect("system clock before unix epoch");
        std::env::temp_dir().join(format!(
            "z00z_stage13_attempt_root_{}_{}",
            std::process::id(),
            nonce
        ))
    }

    #[test]
    fn reset_stage13_removes_outputs() {
        let root = unique_attempt_root();
        let stale = root.join("outputs/scenario_1/hjmt/stale.json");
        create_dir_all(stale.parent().expect("stale parent")).expect("create stale parent");
        write_file(&stale, b"stale").expect("write stale artifact");

        reset_stage13_build_root(&root);

        assert!(path_exists(&root).expect("check root exists"));
        assert!(
            !path_exists(&stale).expect("check stale removed"),
            "retry root reset must remove stale outputs before the next build attempt"
        );

        if path_exists(&root).expect("check root cleanup") {
            remove_dir_all(&root).expect("remove test attempt root");
        }
    }
}
