use std::{
    fs,
    panic::{catch_unwind, AssertUnwindSafe},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use serde_json::Value;
use z00z_simulator::scenario_1::support::checkpoint_shared_cases;
use z00z_storage::checkpoint::{decode_draft_bin, decode_exec_bin};
use z00z_utils::io::read_file;

struct FullRunCase {
    out: PathBuf,
}

struct Stage9Case {
    out: PathBuf,
}

const CHECKPOINT_CASE_RETRIES: usize = 3;

fn load_json(path: &Path) -> Value {
    serde_json::from_slice(&read_file(path).expect("read json")).expect("decode json")
}

fn only_bin(dir: &Path) -> PathBuf {
    let items = fs::read_dir(dir)
        .expect("read dir")
        .map(|row| row.expect("dir entry").path())
        .filter(|path| path.extension().and_then(|item| item.to_str()) == Some("bin"))
        .collect::<Vec<_>>();
    assert_eq!(
        items.len(),
        1,
        "expected one .bin file in {}",
        dir.display()
    );
    items.into_iter().next().expect("bin path")
}

fn draft_file(out: &Path) -> PathBuf {
    only_bin(&out.join("transactions/artifacts/checkpoints/draft"))
}

fn exec_file(out: &Path) -> PathBuf {
    only_bin(&out.join("transactions/artifacts/checkpoints/exec_input"))
}

fn cp_file(out: &Path) -> PathBuf {
    out.join("transactions/checkpoint_s7.json")
}

fn bridge_file(out: &Path) -> PathBuf {
    out.join("transactions/checkpoint_bridge_s6.json")
}

fn full_run_case() -> &'static FullRunCase {
    static CASE: OnceLock<FullRunCase> = OnceLock::new();
    CASE.get_or_init(|| FullRunCase {
        out: retry_checkpoint_case(
            "stage11_full_run",
            checkpoint_shared_cases::default_stage11_out,
        ),
    })
}

fn stage9_case() -> &'static Stage9Case {
    static CASE: OnceLock<Stage9Case> = OnceLock::new();
    CASE.get_or_init(|| Stage9Case {
        out: retry_checkpoint_case("stage9_bridge", checkpoint_shared_cases::bridge_stage9_out),
    })
}

fn stage6_case_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

fn retry_checkpoint_case(label: &str, build: impl Fn() -> PathBuf) -> PathBuf {
    let mut last_err = None;
    for attempt in 1..=CHECKPOINT_CASE_RETRIES {
        match catch_unwind(AssertUnwindSafe(&build)) {
            Ok(path) => return path,
            Err(payload) => {
                last_err = Some(format!(
                    "attempt {attempt}: {}",
                    panic_payload_text(payload)
                ));
            }
        }
    }

    panic!(
        "checkpoint shared case {label} failed after {CHECKPOINT_CASE_RETRIES} attempts: {}",
        last_err.unwrap_or_else(|| "unknown panic".to_string())
    );
}

fn panic_payload_text(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

#[test]
fn test_stage6_writes_exec_input() {
    let _guard = stage6_case_lock();
    let out = &stage9_case().out;
    let bridge = load_json(&bridge_file(out));

    assert_eq!(bridge["stage"].as_u64(), Some(9));
    assert_eq!(bridge["status"].as_str(), Some("ok"));
    assert_eq!(bridge["exec_input_id_hex"].as_str().map(str::len), Some(64));
    assert!(exec_file(out).exists(), "exec_input file must exist");
    assert!(bridge_file(out).exists(), "bridge file must exist");
    assert!(
        !out.join("transactions/artifacts/checkpoints/draft")
            .exists(),
        "stage6 bridge must not write draft output on its own"
    );
}

#[test]
fn test_stage7_uses_draft_path() {
    let _guard = stage6_case_lock();
    let out = &full_run_case().out;

    let draft =
        decode_draft_bin(&read_file(draft_file(out)).expect("read draft")).expect("decode draft");
    let exec =
        decode_exec_bin(&read_file(exec_file(out)).expect("read exec")).expect("decode exec");
    let cp = load_json(&cp_file(out));
    let bridge = load_json(&bridge_file(out));
    let prev_root_hex = hex::encode(draft.prev_root().as_bytes());
    let new_root_hex = hex::encode(draft.new_root().as_bytes());

    assert_eq!(exec.prev_root(), draft.prev_root());
    assert_eq!(exec.prep_snapshot_id().as_bytes().len(), 32);
    assert_eq!(draft.height(), 11);
    assert_eq!(cp["prev_root_hex"].as_str(), Some(prev_root_hex.as_str()));
    assert_eq!(cp["new_root_hex"].as_str(), Some(new_root_hex.as_str()));
    assert_eq!(cp["exec_input_id_hex"], bridge["exec_input_id_hex"]);
    assert_eq!(
        cp["spent_delta"].as_array().map(|item| item.len()),
        Some(draft.spent_delta().len())
    );
    assert_eq!(
        cp["created_delta"].as_array().map(|item| item.len()),
        Some(draft.created_delta().len())
    );
}

#[test]
fn test_stage6_keeps_surfaces_off() {
    let _guard = stage6_case_lock();
    let out = &full_run_case().out;
    assert!(draft_file(out).exists(), "draft file must exist");
    assert!(exec_file(out).exists(), "exec_input file must exist");
    assert!(
        !out.join("transactions/artifacts/checkpoints/final")
            .exists(),
        "draft-only mode must not persist final artifacts"
    );
    assert!(
        !out.join("transactions/artifacts/checkpoints/links")
            .exists(),
        "draft-only mode must not persist links"
    );
    assert!(
        !out.join("transactions/artifacts/checkpoints/audit")
            .exists(),
        "draft-only mode must not persist audit files without a checkpoint id"
    );
    assert!(
        !out.join("storage/post_tx/artifacts/checkpoints/final_lane.marker")
            .exists(),
        "draft-only mode must not publish any final-lane marker"
    );
}
