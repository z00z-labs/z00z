use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use tempfile::TempDir;
use z00z_simulator::{
    config::{
        Stage6ProofMode, STAGE12_FINAL_PUBLIC_EVIDENCE_CLASS, STAGE12_PRIVATE_DRAFT_EVIDENCE_CLASS,
    },
    scenario_1::{runner, stage_12},
    StageResult,
};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{read_file, write_file},
};
use z00z_wallets::tx::{verify_full_tx_package, TxPackage};

use z00z_simulator::scenario_1::{
    support::checkpoint_shared_cases, support::fixture_cache, support::scenario_support,
    support::stage_runner_support,
};

struct RunCase {
    out: PathBuf,
}

const STORAGE_VIEW_SRC: &str = include_str!("../../src/scenario_1/stage_4/storage_view.rs");
const STAGE12_SRC: &str = include_str!("../../src/scenario_1/stage_12/mod.rs");
const STORAGE_ERROR_SRC: &str = include_str!("../../../z00z_storage/src/error.rs");

fn good_s4(cfg: &mut z00z_simulator::config::ScenarioCfg) {
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

fn with_opaque_mode(cfg: &mut z00z_simulator::config::ScenarioCfg) {
    good_s4(cfg);
    cfg.stage6_bundle
        .get_or_insert_with(Default::default)
        .proof_mode = Stage6ProofMode::OpaqueTest;
}

fn with_draft_mode(cfg: &mut z00z_simulator::config::ScenarioCfg) {
    good_s4(cfg);
    cfg.stage6_bundle
        .get_or_insert_with(Default::default)
        .proof_mode = Stage6ProofMode::DraftOnly;
}

fn tx_file(out: &Path) -> PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn s8_file(out: &Path) -> PathBuf {
    out.join("transactions/checkpoint_s8.json")
}

fn load_tx(path: &Path) -> TxPackage {
    JsonCodec
        .deserialize(&read_file(path).expect("read tx json"))
        .expect("decode tx package")
}

fn load_json(path: &Path) -> serde_json::Value {
    serde_json::from_slice(&read_file(path).expect("read json")).expect("decode json")
}

fn save_json(path: &Path, value: &serde_json::Value) {
    write_file(
        path,
        serde_json::to_vec_pretty(value)
            .expect("encode json")
            .as_slice(),
    )
    .expect("write json");
}

fn assert_tx_digest_placeholder_rejection(msg: &str) {
    assert!(
        msg.contains("stage4 tx package verification failed")
            || msg.contains("tx digest must be 32-byte hex"),
        "unexpected tx-digest placeholder failure: {msg}"
    );
}

fn full_draft_case() -> &'static RunCase {
    static CASE: OnceLock<RunCase> = OnceLock::new();
    CASE.get_or_init(|| RunCase {
        out: checkpoint_shared_cases::draft_stage12_out(),
    })
}

fn full_opaque_case() -> &'static RunCase {
    static CASE: OnceLock<RunCase> = OnceLock::new();
    CASE.get_or_init(|| RunCase {
        out: checkpoint_shared_cases::opaque_stage12_out(),
    })
}

fn opaque_stage11_case() -> &'static RunCase {
    static CASE: OnceLock<RunCase> = OnceLock::new();
    CASE.get_or_init(|| RunCase {
        out: checkpoint_shared_cases::opaque_stage11_out(),
    })
}

fn clone_case(
    edit_cfg: impl FnOnce(&mut z00z_simulator::config::ScenarioCfg),
    src_out: &Path,
) -> (TempDir, PathBuf, PathBuf, PathBuf) {
    let temp = TempDir::new().expect("temp dir");
    let (cfg_path, design_path, out) = scenario_support::make_cfg_in(temp.path(), edit_cfg);
    fixture_cache::copy_tree(src_out, &out);
    (temp, cfg_path, design_path, out)
}

#[test]
fn test_truth_stays_noncanonical_artifacts() {
    assert!(
        STORAGE_VIEW_SRC.contains("noncanonical")
            && STORAGE_VIEW_SRC.contains("draft/final checkpoint class boundary"),
        "storage_view must describe prior-final artifacts as noncanonical"
    );
    assert!(
        STAGE12_SRC.contains("noncanonical")
            && STAGE12_SRC.contains("draft/final checkpoint class boundary"),
        "stage 12 finalize flow must keep noncanonical artifacts separate from the canonical final lane"
    );
    assert!(
        STORAGE_ERROR_SRC.contains("noncanonical")
            && STORAGE_ERROR_SRC.contains("checkpoint artifact noncanonical mismatch"),
        "checkpoint error taxonomy must keep noncanonical mismatches explicit"
    );
}

#[test]
fn test_draft_cannot_final_artifact() {
    let out = &full_draft_case().out;
    assert!(
        !out.join("transactions/artifacts/checkpoints/final")
            .exists(),
        "draft-only mode must block final artifact emission"
    );
    assert!(
        !out.join("transactions/artifacts/checkpoints/links")
            .exists(),
        "draft-only mode must block link emission"
    );
    let s8 = load_json(&s8_file(out));
    assert_eq!(s8["status"].as_str(), Some("draft_only"));
    assert_eq!(
        s8["evidence_class"].as_str(),
        Some(STAGE12_PRIVATE_DRAFT_EVIDENCE_CLASS)
    );
    assert!(s8["checkpoint_id_hex"].is_null());
    assert!(s8["artifact_path"].is_null());
    assert!(s8["link_path"].is_null());
    assert!(s8["audit_path"].is_null());
}

#[test]
fn test_opaque_mode_final_artifact() {
    let out = &full_opaque_case().out;
    let s8 = load_json(&s8_file(out));
    assert_eq!(s8["status"].as_str(), Some("ok"));
    assert_eq!(
        s8["evidence_class"].as_str(),
        Some(STAGE12_FINAL_PUBLIC_EVIDENCE_CLASS)
    );
    assert_eq!(s8["checkpoint_id_hex"].as_str().map(str::len), Some(64));
    assert_eq!(
        s8["artifact_path"].as_str(),
        Some("transactions/artifacts/checkpoints/final")
    );
    assert_eq!(
        s8["link_path"].as_str(),
        Some("transactions/artifacts/checkpoints/links")
    );
    assert_eq!(
        s8["audit_path"].as_str(),
        Some("transactions/artifacts/checkpoints/audit")
    );
    assert!(out
        .join("transactions/artifacts/checkpoints/final")
        .exists());
    assert!(out
        .join("transactions/artifacts/checkpoints/links")
        .exists());
    assert!(out
        .join("transactions/artifacts/checkpoints/audit")
        .exists());
}

#[test]
fn test_opaque_rejects_digest_placeholder() {
    let mut tx = load_tx(&tx_file(&full_opaque_case().out));
    tx.tx_digest_hex = "not-a-proof-source".to_string();
    let tx_bytes = JsonCodec.serialize(&tx).expect("encode tx package");
    let verdict = verify_full_tx_package(&tx_bytes)
        .expect("broken tx_digest_hex should produce an invalid verdict, not panic");
    let msg = verdict.errors.join("; ");
    assert_tx_digest_placeholder_rejection(&msg);
}

#[test]
fn test_draft_publication_rejected() {
    let temp = TempDir::new().expect("temp dir");
    let (cfg_path, design_path, out) = scenario_support::make_cfg_in(temp.path(), with_draft_mode);
    let err = runner::run_with_paths(&cfg_path, &design_path)
        .expect_err("draft-only runs must not emit public publication evidence");
    let msg = err.to_string();
    assert!(
        msg.contains("synthetic checkpoint ids are forbidden")
            || msg.contains("must stay off public publication evidence"),
        "unexpected draft public-lane rejection: {msg}"
    );
    assert!(
        !out.join("pub_flow.json").exists(),
        "draft-only public lane must not emit pub_flow.json"
    );
    assert!(
        !out.join("val_flow.json").exists(),
        "draft-only public lane must not emit val_flow.json"
    );
    assert!(
        !out.join("watch_flow.json").exists(),
        "draft-only public lane must not emit watch_flow.json"
    );
}

#[test]
fn test_draft_rejects_digest_placeholder() {
    let mut tx = load_tx(&tx_file(&full_draft_case().out));
    tx.tx_digest_hex = "broken-but-unused".to_string();
    let tx_bytes = JsonCodec.serialize(&tx).expect("encode tx package");
    let verdict = verify_full_tx_package(&tx_bytes)
        .expect("broken tx_digest_hex should produce an invalid verdict, not panic");
    let msg = verdict.errors.join("; ");
    assert_tx_digest_placeholder_rejection(&msg);
}

#[test]
fn test_stage8_rejects_exec_ref() {
    let (_temp, cfg_path, design_path, out) =
        clone_case(with_opaque_mode, &opaque_stage11_case().out);
    let mut ctx = stage_runner_support::resume_stage_session(&cfg_path);

    let s7_path = out.join("transactions/checkpoint_s7.json");
    let mut s7 = load_json(&s7_path);
    s7["exec_input_id_hex"] = serde_json::Value::String("11".repeat(32));
    save_json(&s7_path, &s7);

    let stage12 = stage_runner_support::stage_by_id(&design_path, 12);
    let res = stage_12::run_finalize(&mut ctx, &stage12);
    let msg = match res {
        StageResult::Fail(msg) => msg,
        other => panic!("stage 12 must fail after exec ref tamper, got {other:?}"),
    };

    assert!(msg.contains("exec_input") || msg.contains("link binding") || msg.contains("refs"));
    assert!(
        !out.join("transactions/artifacts/checkpoints/final")
            .exists(),
        "tampered exec ref must block artifact emission"
    );
    assert!(
        !out.join("transactions/artifacts/checkpoints/links")
            .exists(),
        "tampered exec ref must block link emission"
    );
    assert!(
        !out.join("transactions/artifacts/checkpoints/audit")
            .exists(),
        "tampered exec ref must block audit emission"
    );
}

#[test]
fn test_stage8_rejects_snapshot_ref() {
    let (_temp, cfg_path, design_path, out) =
        clone_case(with_opaque_mode, &opaque_stage11_case().out);
    let mut ctx = stage_runner_support::resume_stage_session(&cfg_path);

    let s7_path = out.join("transactions/checkpoint_s7.json");
    let mut s7 = load_json(&s7_path);
    s7["snapshot_id_hex"] = serde_json::Value::String("22".repeat(32));
    save_json(&s7_path, &s7);

    let stage12 = stage_runner_support::stage_by_id(&design_path, 12);
    let res = stage_12::run_finalize(&mut ctx, &stage12);
    let msg = match res {
        StageResult::Fail(msg) => msg,
        other => panic!("stage 12 must fail after snapshot ref tamper, got {other:?}"),
    };

    assert!(
        msg.contains("snapshot_id mismatch") || msg.contains("exec_input snapshot_id mismatch")
    );
    assert!(
        !out.join("transactions/artifacts/checkpoints/final")
            .exists(),
        "tampered snapshot ref must block artifact emission"
    );
    assert!(
        !out.join("transactions/artifacts/checkpoints/links")
            .exists(),
        "tampered snapshot ref must block link emission"
    );
    assert!(
        !out.join("transactions/artifacts/checkpoints/audit")
            .exists(),
        "tampered snapshot ref must block audit emission"
    );
}

#[test]
fn test_stage8_rejects_fragment_ids() {
    let (_temp, cfg_path, design_path, out) =
        clone_case(with_opaque_mode, &opaque_stage11_case().out);
    let mut ctx = stage_runner_support::resume_stage_session(&cfg_path);

    let s7_path = out.join("transactions/checkpoint_s7.json");
    let mut s7 = load_json(&s7_path);
    s7["fragment_ids"] = serde_json::json!(["frag_x", "frag_y"]);
    save_json(&s7_path, &s7);

    let stage12 = stage_runner_support::stage_by_id(&design_path, 12);
    let res = stage_12::run_finalize(&mut ctx, &stage12);
    let msg = match res {
        StageResult::Fail(msg) => msg,
        other => panic!("stage 12 must fail after fragment_ids tamper, got {other:?}"),
    };

    assert!(msg.contains("fragment_ids mismatch"));
    assert!(
        !out.join("transactions/artifacts/checkpoints/final")
            .exists(),
        "tampered fragment_ids must block artifact emission"
    );
    assert!(
        !out.join("transactions/artifacts/checkpoints/links")
            .exists(),
        "tampered fragment_ids must block link emission"
    );
    assert!(
        !out.join("transactions/artifacts/checkpoints/audit")
            .exists(),
        "tampered fragment_ids must block audit emission"
    );
}
