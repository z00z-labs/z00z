use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use tempfile::TempDir;
use z00z_simulator::{scenario_1::stage_11, StageResult};
use z00z_storage::{
    checkpoint::{
        derive_exec_id, encode_exec_bin, CheckpointExecInput, CheckpointExecInputId,
        CheckpointExecTx, CheckpointFsStore, CheckpointId, CheckpointProofSystem, CheckpointStore,
        CheckpointTransitionStatementCoreV1, CheckpointVersion, CreatedEnt, SpentEnt,
    },
    settlement::{CheckRoot, ClaimSourceRoot, SettlementStateRoot},
    snapshot::PrepSnapshotId,
    CheckpointError,
};
use z00z_utils::{
    codec::{BincodeCodec, Codec},
    io::{read_file, save_json, write_file},
};
use z00z_wallets::tx::{verify_full_tx_package, TxPackage};

const STAGE4_SRC: &str = include_str!("../../src/scenario_1/stage_4/mod.rs");
const SPEND_VERIFICATION_SRC: &str =
    include_str!("../../../z00z_wallets/src/tx/spend_verification.rs");

use z00z_simulator::scenario_1::{support::checkpoint_shared_cases, support::stage_runner_support};

use stage_runner_support::stage_by_id;

struct RunCase {
    out: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ArtifactWire {
    version: CheckpointVersion,
    height: u64,
    prev_root: CheckRoot,
    new_root: CheckRoot,
    prev_settlement_root: SettlementStateRoot,
    new_settlement_root: SettlementStateRoot,
    claim_root: Option<ClaimSourceRoot>,
    spent_delta: Vec<SpentEnt>,
    created_delta: Vec<CreatedEnt>,
    prep_snapshot_id: Option<PrepSnapshotId>,
    exec_input_id: Option<CheckpointExecInputId>,
    statement_core: Option<CheckpointTransitionStatementCoreV1>,
    da_ref: Option<[u8; 32]>,
    proof_sys: CheckpointProofSystem,
    cp_proof: Vec<u8>,
}

fn load_json(path: &Path) -> serde_json::Value {
    serde_json::from_slice(&read_file(path).expect("read json")).expect("decode json")
}

fn parse_hex32(value: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    hex::decode_to_slice(value, &mut out).expect("decode hex32");
    out
}

fn transactions_store_root(out: &Path) -> PathBuf {
    out.join("transactions")
}

fn post_tx_store_root(out: &Path) -> PathBuf {
    out.join("storage").join("post_tx")
}

fn checkpoint_id_from_s8(out: &Path) -> CheckpointId {
    let s8 = load_json(&out.join("transactions/checkpoint_s8.json"));
    CheckpointId::new(parse_hex32(
        s8["checkpoint_id_hex"].as_str().expect("checkpoint id hex"),
    ))
}

fn read_stage7_input_id(out: &Path) -> CheckpointExecInputId {
    let s7 = load_json(&out.join("transactions/checkpoint_s7.json"));
    CheckpointExecInputId::new(parse_hex32(
        s7["exec_input_id_hex"].as_str().expect("exec input id hex"),
    ))
}

fn exec_path(root: &Path, exec_id: CheckpointExecInputId) -> PathBuf {
    CheckpointFsStore::new(root)
        .exec_dir()
        .join(format!("{}.bin", hex::encode(exec_id.as_bytes())))
}

fn artifact_path(root: &Path, checkpoint_id: CheckpointId) -> PathBuf {
    CheckpointFsStore::new(root)
        .artifact_dir()
        .join(format!("{}.bin", hex::encode(checkpoint_id.as_bytes())))
}

fn load_artifact_wire(path: &Path) -> ArtifactWire {
    BincodeCodec
        .deserialize(&read_file(path).expect("read artifact"))
        .expect("decode artifact wire")
}

fn write_artifact_wire(path: &Path, artifact: &ArtifactWire) {
    let bytes = BincodeCodec
        .serialize(artifact)
        .expect("encode artifact wire");
    write_file(path, &bytes).expect("write artifact wire");
}

fn tampered_claim_root(claim_root: ClaimSourceRoot) -> ClaimSourceRoot {
    let mut bytes = claim_root.into_bytes();
    bytes[0] ^= 0xA5;
    ClaimSourceRoot::new(
        claim_root.root_version(),
        SettlementStateRoot::settlement_v1(bytes),
    )
}

fn assert_redacted_checkpoint_err(err: &CheckpointError, checkpoint_id: CheckpointId, want: &str) {
    let text = err.to_string();
    assert_eq!(text, want);
    assert!(!text.contains(&hex::encode(checkpoint_id.as_bytes())));
}

fn tampered_exec(exec: &CheckpointExecInput) -> CheckpointExecInput {
    let txs = exec
        .txs()
        .iter()
        .enumerate()
        .map(|(index, tx)| {
            let mut proof = tx.tx_proof().to_vec();
            if index == 0 {
                proof.push(0xA5);
            }
            CheckpointExecTx::new(tx.input_refs().to_vec(), tx.outputs().to_vec(), proof)
                .expect("rebuild exec tx")
        })
        .collect::<Vec<_>>();
    CheckpointExecInput::new(
        exec.version(),
        exec.prep_snapshot_id(),
        exec.prev_root(),
        txs,
    )
    .expect("rebuild exec input")
}

fn tampered_exec_root(exec: &CheckpointExecInput, prev_root: [u8; 32]) -> CheckpointExecInput {
    CheckpointExecInput::new(
        exec.version(),
        exec.prep_snapshot_id(),
        prev_root.into(),
        exec.txs().to_vec(),
    )
    .expect("rebuild exec input with tampered root")
}

fn full_case() -> &'static RunCase {
    static CASE: OnceLock<RunCase> = OnceLock::new();
    CASE.get_or_init(|| RunCase {
        out: checkpoint_shared_cases::stage12_out(),
    })
}

fn clone_stage10_case() -> (TempDir, PathBuf, PathBuf, PathBuf) {
    let temp = TempDir::new().expect("temp dir");
    let (cfg_path, design_path, out) = checkpoint_shared_cases::clone_stage10_case(temp.path());
    (temp, cfg_path, design_path, out)
}

fn clone_stage12_case() -> (TempDir, PathBuf, PathBuf, PathBuf) {
    let temp = TempDir::new().expect("temp dir");
    let (cfg_path, design_path, out) = checkpoint_shared_cases::clone_stage12_case(temp.path());
    (temp, cfg_path, design_path, out)
}

#[test]
fn test_checkpoint_matches_exec_proof() {
    let out = &full_case().out;
    let checkpoint_id = checkpoint_id_from_s8(out);
    let exec_input_id = read_stage7_input_id(out);
    let tx_store = CheckpointFsStore::new(transactions_store_root(out));
    let post_store = CheckpointFsStore::new(post_tx_store_root(out));

    let tx_exec = tx_store
        .load_exec_input(&exec_input_id)
        .expect("tx exec roundtrip");
    let tx_art = tx_store
        .load_artifact(&checkpoint_id)
        .expect("tx artifact roundtrip");
    let tx_link = tx_store
        .load_link(&checkpoint_id)
        .expect("tx link roundtrip");
    let tx_audit = tx_store
        .load_audit(&checkpoint_id)
        .expect("tx audit roundtrip");

    let post_exec = post_store
        .load_exec_input(&exec_input_id)
        .expect("post-tx exec roundtrip");
    let post_art = post_store
        .load_noncanonical_artifact(&checkpoint_id)
        .expect("post-tx artifact roundtrip");
    let post_link = post_store
        .load_noncanonical_link(&checkpoint_id)
        .expect("post-tx link roundtrip");
    let post_audit = post_store
        .load_noncanonical_audit(&checkpoint_id)
        .expect("post-tx audit roundtrip");

    assert_eq!(tx_exec, post_exec);
    assert_eq!(tx_art, post_art);
    assert_eq!(tx_link, post_link);
    assert_eq!(tx_audit, post_audit);
    assert_eq!(tx_exec.txs().len(), 1);
    assert_eq!(
        tx_art.cp_proof(),
        z00z_storage::checkpoint::CheckpointTransitionStatementV1::new(
            tx_art.version(),
            tx_art.height(),
            tx_art.pub_in(),
            tx_link.prep_snapshot_id(),
            tx_link.exec_input_id(),
        )
        .backend_payload()
        .as_slice()
    );
    assert_eq!(tx_link.checkpoint_id(), checkpoint_id);
    assert_eq!(tx_audit.checkpoint_id(), checkpoint_id);
}

#[test]
fn test_post_tx_export_rejects_canonical_loads() {
    let out = &full_case().out;
    let checkpoint_id = checkpoint_id_from_s8(out);
    let post_store = CheckpointFsStore::new(post_tx_store_root(out));

    let art_err = post_store
        .load_artifact(&checkpoint_id)
        .expect_err("canonical artifact load must reject post-tx export lane");
    let link_err = post_store
        .load_link(&checkpoint_id)
        .expect_err("canonical link load must reject post-tx export lane");
    let audit_err = post_store
        .load_audit(&checkpoint_id)
        .expect_err("canonical audit load must reject post-tx export lane");

    assert!(matches!(art_err, CheckpointError::ArtifactCompatMix));
    assert!(matches!(link_err, CheckpointError::ArtifactCompatMix));
    assert!(matches!(audit_err, CheckpointError::ArtifactCompatMix));
}

#[test]
fn test_stage4_artifacts_pre_acceptance() {
    assert!(
        STAGE4_SRC.contains("structurally useful")
            && STAGE4_SRC.contains("weaker than later")
            && STAGE4_SRC.contains("spend/checkpoint semantic acceptance")
            && STAGE4_SRC.contains("non-authoritative"),
        "stage 4 must document that its early files are non-authoritative"
    );
    assert!(
        SPEND_VERIFICATION_SRC.contains("Semantically incomplete")
            && SPEND_VERIFICATION_SRC.contains("structurally plausible")
            && SPEND_VERIFICATION_SRC.contains("before checkpoint/state mutation"),
        "public spend verifier must state that structural plausibility stays weaker than semantic acceptance"
    );

    let (_temp, cfg_path, design_path, out) = clone_stage10_case();
    let mut ctx = stage_runner_support::resume_stage_session(&cfg_path);

    assert!(
        out.join("stage_4_snapshot.json").exists(),
        "stage 4 snapshot should exist before semantic acceptance"
    );
    assert!(
        out.join("claim_publish/audit_log.json").exists(),
        "stage 4 publish audit should exist before semantic acceptance"
    );

    let tx_pkg_path = out.join("transactions").join("tx_alice_to_bob_pkg.json");
    let mut pkg: TxPackage = z00z_utils::io::load_json(&tx_pkg_path).expect("load tx package");
    pkg.chain_name = "forged-devnet".to_string();
    save_json(&tx_pkg_path, &pkg).expect("rewrite digest-tampered tx package");

    let stage11 = stage_by_id(&design_path, 11);
    let res = stage_11::run_apply(&mut ctx, &stage11);
    let msg = match res {
        StageResult::Fail(msg) => msg,
        other => panic!("stage 11 must fail after tx package digest tamper, got {other:?}"),
    };

    assert!(
        msg.contains("stage4 tx package verification failed")
            || msg.contains("tx_digest_hex does not match payload"),
        "unexpected stage 11 failure: {msg}"
    );
    assert!(
        out.join("stage_4_snapshot.json").exists(),
        "stage 4 snapshot must remain a weaker structural artifact after semantic failure"
    );
    assert!(
        out.join("claim_publish/audit_log.json").exists(),
        "stage 4 publish audit must remain a weaker structural artifact after semantic failure"
    );
    assert!(
        !out.join("transactions/checkpoint_s7.json").exists(),
        "semantic failure must block authoritative checkpoint summary emission"
    );
    assert!(
        !post_tx_store_root(&out)
            .join("artifacts/checkpoints/draft")
            .exists(),
        "semantic failure must block post-tx checkpoint draft state mutation"
    );
}

#[test]
fn test_stage11_rejects_input_row() {
    let (_temp, cfg_path, design_path, out) = clone_stage10_case();
    let mut ctx = stage_runner_support::resume_stage_session(&cfg_path);

    let tx_root = transactions_store_root(&out);
    let tx_store = CheckpointFsStore::new(&tx_root);
    let exec_input_id = {
        let bridge = load_json(&tx_root.join("checkpoint_bridge_s6.json"));
        CheckpointExecInputId::new(parse_hex32(
            bridge["exec_input_id_hex"]
                .as_str()
                .expect("bridge exec input id"),
        ))
    };
    let exec = tx_store
        .load_exec_input(&exec_input_id)
        .expect("load canonical exec");
    let tampered = tampered_exec(&exec);
    let tampered_bytes = encode_exec_bin(&tampered).expect("encode tampered exec");
    write_file(exec_path(&tx_root, exec_input_id), &tampered_bytes).expect("write tampered exec");

    let stage11 = stage_by_id(&design_path, 11);
    let res = stage_11::run_apply(&mut ctx, &stage11);
    let msg = match res {
        StageResult::Fail(msg) => msg,
        other => panic!("stage 11 must fail after exec tamper, got {other:?}"),
    };

    assert!(
        msg.contains("exec_input") || msg.contains("bridge exec_input_id mismatch"),
        "unexpected stage 11 failure: {msg}"
    );
    assert!(
        !out.join("transactions/checkpoint_s7.json").exists(),
        "stage 11 must not emit checkpoint_s7.json after authoritative exec mismatch"
    );
    assert!(
        !post_tx_store_root(&out)
            .join("artifacts/checkpoints/draft")
            .exists(),
        "stage 11 must not persist post-tx checkpoint draft after authoritative exec mismatch"
    );
}

#[test]
fn test_stage11_checkpoint_second_seam() {
    let (_temp, cfg_path, design_path, out) = clone_stage10_case();
    let mut ctx = stage_runner_support::resume_stage_session(&cfg_path);

    let tx_pkg_path = out.join("transactions").join("tx_alice_to_bob_pkg.json");
    let verdict = verify_full_tx_package(&read_file(&tx_pkg_path).expect("read tx package"))
        .expect("full verifier must run on the persisted stage4 package");
    assert!(
        verdict.valid,
        "stage4 package must stay admissible before the checkpoint-apply seam: {:?}",
        verdict.errors
    );

    let tx_root = transactions_store_root(&out);
    let tx_store = CheckpointFsStore::new(&tx_root);
    let exec_input_id = {
        let bridge = load_json(&tx_root.join("checkpoint_bridge_s6.json"));
        CheckpointExecInputId::new(parse_hex32(
            bridge["exec_input_id_hex"]
                .as_str()
                .expect("bridge exec input id"),
        ))
    };
    let exec = tx_store
        .load_exec_input(&exec_input_id)
        .expect("load canonical exec");
    let tampered = tampered_exec(&exec);
    let tampered_bytes = encode_exec_bin(&tampered).expect("encode tampered exec");
    write_file(exec_path(&tx_root, exec_input_id), &tampered_bytes).expect("write tampered exec");

    let stage11 = stage_by_id(&design_path, 11);
    let res = stage_11::run_apply(&mut ctx, &stage11);
    let msg = match res {
        StageResult::Fail(msg) => msg,
        other => panic!(
            "stage 11 must fail after exec tamper even when package admission already passed, got {other:?}"
        ),
    };

    assert!(
        msg.contains("exec_input") || msg.contains("bridge exec_input_id mismatch"),
        "unexpected stage 11 failure: {msg}"
    );
    assert!(
        !out.join("transactions/checkpoint_s7.json").exists(),
        "checkpoint apply must stay the second explicit seam after package admission"
    );
    assert!(
        !post_tx_store_root(&out)
            .join("artifacts/checkpoints/draft")
            .exists(),
        "failed checkpoint apply must not persist post-tx checkpoint draft state"
    );
}

#[test]
fn test_stage11_rejects_root_drift() {
    let (_temp, cfg_path, design_path, out) = clone_stage10_case();
    let mut ctx = stage_runner_support::resume_stage_session(&cfg_path);

    let tx_pkg_path = out.join("transactions").join("tx_alice_to_bob_pkg.json");
    let verdict = verify_full_tx_package(&read_file(&tx_pkg_path).expect("read tx package"))
        .expect("full verifier must run on the persisted stage4 package");
    assert!(
        verdict.valid,
        "stage4 package must stay admissible before the checkpoint-apply seam: {:?}",
        verdict.errors
    );

    let tx_root = transactions_store_root(&out);
    let tx_store = CheckpointFsStore::new(&tx_root);
    let exec_input_id = {
        let bridge = load_json(&tx_root.join("checkpoint_bridge_s6.json"));
        CheckpointExecInputId::new(parse_hex32(
            bridge["exec_input_id_hex"]
                .as_str()
                .expect("bridge exec input id"),
        ))
    };
    let exec = tx_store
        .load_exec_input(&exec_input_id)
        .expect("load canonical exec");
    let tampered = tampered_exec_root(&exec, [0xA5; 32]);
    let tampered_bytes = encode_exec_bin(&tampered).expect("encode tampered exec");
    write_file(exec_path(&tx_root, exec_input_id), &tampered_bytes).expect("write tampered exec");

    let mut bridge = load_json(&tx_root.join("checkpoint_bridge_s6.json"));
    bridge["exec_input_id_hex"] =
        serde_json::Value::String(hex::encode(derive_exec_id(&tampered_bytes).as_bytes()));
    bridge["prev_root_hex"] = serde_json::Value::String(hex::encode([0xA5; 32]));
    save_json(tx_root.join("checkpoint_bridge_s6.json"), &bridge).expect("rewrite bridge root");

    let stage11 = stage_by_id(&design_path, 11);
    let res = stage_11::run_apply(&mut ctx, &stage11);
    let msg = match res {
        StageResult::Fail(msg) => msg,
        other => panic!("stage 11 must fail after exec root drift, got {other:?}"),
    };

    assert!(
        msg.contains("exec prev_root mismatch with stage4 spend proof")
            || msg.contains("tx proof mismatch with stage4 package"),
        "unexpected stage 11 failure: {msg}"
    );
    assert!(
        !out.join("transactions/checkpoint_s7.json").exists(),
        "stage 11 must not emit checkpoint_s7.json after exec root drift"
    );
    assert!(
        !post_tx_store_root(&out)
            .join("artifacts/checkpoints/draft")
            .exists(),
        "stage 11 must not persist post-tx checkpoint draft after exec root drift"
    );
}

#[test]
fn test_stage11_rejects_proof_tamper() {
    let (_temp, cfg_path, design_path, out) = clone_stage10_case();
    let mut ctx = stage_runner_support::resume_stage_session(&cfg_path);

    let tx_pkg_path = out.join("transactions").join("tx_alice_to_bob_pkg.json");
    let mut pkg: TxPackage = z00z_utils::io::load_json(&tx_pkg_path).expect("load tx package");
    let spend = pkg.tx.proof.spend.as_mut().expect("stage4 spend proof");
    spend.prev_root_hex = hex::encode([0xA5u8; 32]);
    save_json(&tx_pkg_path, &pkg).expect("rewrite tampered tx package");

    let stage11 = stage_by_id(&design_path, 11);
    let res = stage_11::run_apply(&mut ctx, &stage11);
    let msg = match res {
        StageResult::Fail(msg) => msg,
        other => panic!("stage 11 must fail after tx package proof tamper, got {other:?}"),
    };

    assert!(
        msg.contains("exec tx proof mismatch with stage4 package")
            || msg.contains("stage4 tx package verification failed")
            || msg.contains("tx_digest_hex does not match payload")
            || msg.contains("stage4 public spend contract failed")
            || msg.contains("current-stack tx public spend verifier failed"),
        "unexpected stage 11 failure: {msg}"
    );
    assert!(
        !out.join("transactions/checkpoint_s7.json").exists(),
        "stage 11 must not emit checkpoint_s7.json after tx package proof tamper"
    );
}

#[test]
fn test_stage11_rejects_digest_tamper() {
    let (_temp, cfg_path, design_path, out) = clone_stage10_case();
    let mut ctx = stage_runner_support::resume_stage_session(&cfg_path);

    let tx_pkg_path = out.join("transactions").join("tx_alice_to_bob_pkg.json");
    let mut pkg: TxPackage = z00z_utils::io::load_json(&tx_pkg_path).expect("load tx package");
    pkg.chain_name = "forged-devnet".to_string();
    save_json(&tx_pkg_path, &pkg).expect("rewrite digest-tampered tx package");

    let stage11 = stage_by_id(&design_path, 11);
    let res = stage_11::run_apply(&mut ctx, &stage11);
    let msg = match res {
        StageResult::Fail(msg) => msg,
        other => panic!("stage 11 must fail after tx package digest tamper, got {other:?}"),
    };

    assert!(
        msg.contains("stage4 tx package verification failed")
            || msg.contains("tx_digest_hex does not match payload"),
        "unexpected stage 11 failure: {msg}"
    );
    assert!(
        !out.join("transactions/checkpoint_s7.json").exists(),
        "stage 11 must not emit checkpoint_s7.json after tx package digest tamper"
    );
}

#[test]
fn test_post_rejects_input_row() {
    let (_temp, _cfg_path, _design_path, out) = clone_stage12_case();
    let checkpoint_id = checkpoint_id_from_s8(&out);
    let exec_input_id = read_stage7_input_id(&out);
    let post_root = post_tx_store_root(&out);
    let post_store = CheckpointFsStore::new(&post_root);
    let exec = post_store
        .load_exec_input(&exec_input_id)
        .expect("load post-tx exec");
    let tampered = tampered_exec(&exec);
    let tampered_bytes = encode_exec_bin(&tampered).expect("encode tampered exec");
    write_file(exec_path(&post_root, exec_input_id), &tampered_bytes)
        .expect("write replay-style exec row");

    let err = post_store
        .load_noncanonical_link(&checkpoint_id)
        .expect_err("tampered exec row must reject post-tx link reload");

    assert!(matches!(err, CheckpointError::ReplayMix));
}

#[test]
fn test_claim_tamper_rejects() {
    let (_temp, _cfg_path, _design_path, out) = clone_stage12_case();
    let checkpoint_id = checkpoint_id_from_s8(&out);
    let post_root = post_tx_store_root(&out);
    let post_store = CheckpointFsStore::new(&post_root);
    let baseline_link = post_store
        .load_noncanonical_link(&checkpoint_id)
        .expect("baseline post-tx link");
    let art_path = artifact_path(&post_root, checkpoint_id);
    let mut artifact = load_artifact_wire(&art_path);
    let claim_root = artifact
        .claim_root
        .expect("stage12 checkpoint artifact must keep claim_root");
    artifact.claim_root = Some(tampered_claim_root(claim_root));
    write_artifact_wire(&art_path, &artifact);

    let load_err = post_store
        .load_noncanonical_link(&checkpoint_id)
        .expect_err("tampered claim_root must reject post-tx link reload");
    assert!(matches!(load_err, CheckpointError::ProofMix));
    assert_redacted_checkpoint_err(&load_err, checkpoint_id, "checkpoint proof mismatch");

    let art_err = post_store
        .load_noncanonical_artifact(&baseline_link.checkpoint_id())
        .expect_err("tampered claim_root must reject noncanonical artifact load");
    assert!(matches!(art_err, CheckpointError::ProofMix));
    assert_redacted_checkpoint_err(&art_err, checkpoint_id, "checkpoint proof mismatch");
}

#[test]
fn test_proof_tamper_rejects() {
    let (_temp, _cfg_path, _design_path, out) = clone_stage12_case();
    let checkpoint_id = checkpoint_id_from_s8(&out);
    let post_root = post_tx_store_root(&out);
    let post_store = CheckpointFsStore::new(&post_root);
    let baseline_link = post_store
        .load_noncanonical_link(&checkpoint_id)
        .expect("baseline post-tx link");
    let art_path = artifact_path(&post_root, checkpoint_id);
    let mut artifact = load_artifact_wire(&art_path);
    assert!(
        !artifact.cp_proof.is_empty(),
        "stage12 checkpoint artifact must keep cp_proof bytes"
    );
    artifact.cp_proof[0] ^= 0xA5;
    write_artifact_wire(&art_path, &artifact);

    let load_err = post_store
        .load_noncanonical_link(&checkpoint_id)
        .expect_err("tampered cp_proof must reject post-tx link reload");
    assert!(matches!(load_err, CheckpointError::ProofMix));
    assert_redacted_checkpoint_err(&load_err, checkpoint_id, "checkpoint proof mismatch");

    let art_err = post_store
        .load_noncanonical_artifact(&baseline_link.checkpoint_id())
        .expect_err("tampered cp_proof must reject noncanonical artifact load");
    assert!(matches!(art_err, CheckpointError::ProofMix));
    assert_redacted_checkpoint_err(&art_err, checkpoint_id, "checkpoint proof mismatch");
}
