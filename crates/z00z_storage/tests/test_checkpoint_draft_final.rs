use tempfile::TempDir;
use z00z_storage::{
    checkpoint::{
        load_artifact, load_draft, CheckpointDraft, CheckpointExecInputId, CheckpointProof,
        CheckpointTransitionStatementV1, CheckpointVersion, CreatedEnt, SpentEnt,
    },
    settlement::CheckRoot,
    snapshot::PrepSnapshotId,
    CheckpointError,
};
use z00z_utils::{
    codec::{BincodeCodec, Codec},
    io::{read_file, write_file},
};

fn temp_dir() -> TempDir {
    TempDir::new().expect("temp dir")
}

fn draft() -> CheckpointDraft {
    CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        21,
        CheckRoot::new([1u8; 32]),
        CheckRoot::new([2u8; 32]),
        vec![SpentEnt::new([3u8; 32])],
        vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
    )
}

fn proof(draft: &CheckpointDraft, bytes: Vec<u8>) -> CheckpointProof {
    let stmt = CheckpointTransitionStatementV1::from_draft(
        draft,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let cp_proof = if bytes.is_empty() {
        bytes
    } else {
        stmt.backend_payload()
    };
    CheckpointProof::new_attest(stmt, cp_proof).expect("proof")
}

#[test]
fn test_persist_rejects_final_load() {
    let dir = temp_dir();
    let file = dir.path().join("checkpoint_draft.bin");
    let bytes = BincodeCodec.serialize(&draft()).expect("encode draft");
    write_file(&file, &bytes).expect("write draft file");

    let raw = read_file(&file).expect("read draft file");
    let err = load_artifact(&raw).expect_err("draft bytes must not load as final artifact");

    assert!(matches!(err, CheckpointError::WrongClass));
}

#[test]
fn test_persist_rejects_draft_load() {
    let dir = temp_dir();
    let file = dir.path().join("checkpoint_final.bin");
    let draft = draft();
    let art = draft
        .finalize(proof(&draft, vec![9u8]))
        .expect("seal draft");
    let bytes = BincodeCodec.serialize(&art).expect("encode final artifact");
    write_file(&file, &bytes).expect("write final file");

    let raw = read_file(&file).expect("read final file");
    let err = load_draft(&raw).expect_err("final bytes must not load as draft artifact");

    assert!(matches!(err, CheckpointError::WrongClass));
}

#[test]
fn test_proofless_final_rejects() {
    let draft = draft();
    let stmt = CheckpointTransitionStatementV1::from_draft(
        &draft,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let err = CheckpointProof::new_attest(stmt, Vec::new())
        .expect_err("proofless attestation must reject");

    assert!(matches!(err, CheckpointError::ProoflessFinal));
}

#[test]
fn test_draft_final_stay_distinct() {
    let draft = draft();
    let art = draft
        .finalize(proof(&draft, vec![9u8]))
        .expect("seal draft");

    assert_eq!(draft.height(), art.height());
    assert_eq!(draft.prev_root(), art.prev_root());
    assert_eq!(draft.new_root(), art.new_root());
    assert_eq!(draft.pub_in(), art.pub_in());
    assert_eq!(
        art.cp_proof(),
        proof(&draft, vec![9u8]).attest_payload_bytes()
    );
}
