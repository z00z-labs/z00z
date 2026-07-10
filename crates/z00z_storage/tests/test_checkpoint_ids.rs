use tempfile::TempDir;
use z00z_storage::{
    checkpoint::{
        check_art_key, check_exec_key, derive_checkpoint_id, derive_draft_id, derive_exec_id,
        encode_art_bin, reject_draft_for_checkpoint_id, CheckpointArtifact, CheckpointDraft,
        CheckpointExecInputId, CheckpointId, CheckpointProof, CheckpointProofSystem,
        CheckpointTransitionStatementV1, CheckpointVersion, CreatedEnt, SpentEnt,
    },
    settlement::{CheckRoot, SettlementStateRoot},
    snapshot::PrepSnapshotId,
    CheckpointError,
};
use z00z_utils::{
    codec::{BincodeCodec, Codec},
    io::{read_file, write_file},
};

#[derive(serde::Serialize)]
struct ArtWire {
    version: CheckpointVersion,
    height: u64,
    prev_root: CheckRoot,
    new_root: CheckRoot,
    prev_settlement_root: SettlementStateRoot,
    new_settlement_root: SettlementStateRoot,
    claim_root: Option<z00z_storage::settlement::ClaimSourceRoot>,
    spent_delta: Vec<SpentEnt>,
    created_delta: Vec<CreatedEnt>,
    prep_snapshot_id: Option<z00z_storage::snapshot::PrepSnapshotId>,
    exec_input_id: Option<CheckpointExecInputId>,
    proof_sys: CheckpointProofSystem,
    cp_proof: Vec<u8>,
}

fn temp_dir() -> TempDir {
    TempDir::new().expect("temp dir")
}

fn draft() -> CheckpointDraft {
    CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        22,
        CheckRoot::new([1u8; 32]),
        CheckRoot::new([2u8; 32]),
        vec![SpentEnt::new([3u8; 32])],
        vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
    )
}

fn art() -> CheckpointArtifact {
    let draft = draft();
    let proof = draft
        .attest_proof(
            PrepSnapshotId::new([7u8; 32]),
            CheckpointExecInputId::new([8u8; 32]),
        )
        .expect("proof");
    draft.finalize(proof).expect("checkpoint artifact")
}

#[test]
fn test_art_stable_same_bytes() {
    let dir = temp_dir();
    let one = dir.path().join("checkpoint-final.bin");
    let two = dir.path().join("report-copy.bin");
    let bytes = BincodeCodec.serialize(&art()).expect("encode artifact");
    write_file(&one, &bytes).expect("write one");
    write_file(&two, &bytes).expect("write two");

    let art_one: CheckpointArtifact = BincodeCodec
        .deserialize(&read_file(&one).expect("read one"))
        .expect("decode one");
    let art_two: CheckpointArtifact = BincodeCodec
        .deserialize(&read_file(&two).expect("read two"))
        .expect("decode two");

    assert_eq!(
        derive_checkpoint_id(&art_one).expect("id one"),
        derive_checkpoint_id(&art_two).expect("id two")
    );
}

#[test]
fn test_draft_class_art_id() {
    let err = reject_draft_for_checkpoint_id(&draft()).expect_err("draft id must reject");

    assert!(matches!(err, CheckpointError::WrongClass));
}

#[test]
fn test_unsupported_rejects_art_id() {
    let wire = ArtWire {
        version: CheckpointVersion::new(9),
        height: 22,
        prev_root: CheckRoot::new([1u8; 32]),
        new_root: CheckRoot::new([2u8; 32]),
        prev_settlement_root: SettlementStateRoot::settlement_v1([1u8; 32]),
        new_settlement_root: SettlementStateRoot::settlement_v1([2u8; 32]),
        claim_root: None,
        spent_delta: vec![SpentEnt::new([3u8; 32])],
        created_delta: vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
        prep_snapshot_id: None,
        exec_input_id: None,
        proof_sys: CheckpointProofSystem::OPAQUE_ATTEST,
        cp_proof: vec![8u8],
    };
    let bytes = BincodeCodec.serialize(&wire).expect("encode wire");
    let bad: CheckpointArtifact = BincodeCodec.deserialize(&bytes).expect("artifact shell");

    let err = derive_checkpoint_id(&bad).expect_err("unsupported version must reject");

    assert!(matches!(err, CheckpointError::VersionMix));
}

#[test]
fn test_proof_sys_art_id() {
    let wire = ArtWire {
        version: CheckpointVersion::CURRENT,
        height: 22,
        prev_root: CheckRoot::new([1u8; 32]),
        new_root: CheckRoot::new([2u8; 32]),
        prev_settlement_root: SettlementStateRoot::settlement_v1([1u8; 32]),
        new_settlement_root: SettlementStateRoot::settlement_v1([2u8; 32]),
        claim_root: None,
        spent_delta: vec![SpentEnt::new([3u8; 32])],
        created_delta: vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
        prep_snapshot_id: None,
        exec_input_id: None,
        proof_sys: CheckpointProofSystem::new(9),
        cp_proof: vec![8u8],
    };
    let bytes = BincodeCodec.serialize(&wire).expect("encode wire");
    let bad: CheckpointArtifact = BincodeCodec.deserialize(&bytes).expect("artifact shell");

    let err = derive_checkpoint_id(&bad).expect_err("unsupported proof system must reject");

    assert!(matches!(err, CheckpointError::ProofSysMix));
}

#[test]
fn test_verified_proof_sys_rejects_art_id_and_encode() {
    let draft = draft();
    let stmt = CheckpointTransitionStatementV1::from_draft(
        &draft,
        PrepSnapshotId::new([7u8; 32]),
        CheckpointExecInputId::new([8u8; 32]),
    );
    let wire = ArtWire {
        version: CheckpointVersion::CURRENT,
        height: draft.height(),
        prev_root: draft.prev_root(),
        new_root: draft.new_root(),
        prev_settlement_root: draft.prev_settlement_root(),
        new_settlement_root: draft.new_settlement_root(),
        claim_root: draft.claim_root(),
        spent_delta: draft.spent_delta().to_vec(),
        created_delta: draft.created_delta().to_vec(),
        prep_snapshot_id: Some(stmt.prep_snapshot_id()),
        exec_input_id: Some(stmt.exec_input_id()),
        proof_sys: CheckpointProofSystem::VERIFIED,
        cp_proof: stmt.backend_payload(),
    };
    let bytes = BincodeCodec.serialize(&wire).expect("encode wire");
    let bad: CheckpointArtifact = BincodeCodec.deserialize(&bytes).expect("artifact shell");

    let id_err = derive_checkpoint_id(&bad).expect_err("verified proof sys must reject id");
    let encode_err = encode_art_bin(&bad).expect_err("verified proof sys must reject encode");

    assert!(matches!(id_err, CheckpointError::ProofSysMix));
    assert!(matches!(encode_err, CheckpointError::ProofSysMix));
}

#[test]
fn test_attest_rejects_id_encode() {
    let wire = ArtWire {
        version: CheckpointVersion::CURRENT,
        height: 22,
        prev_root: CheckRoot::new([1u8; 32]),
        new_root: CheckRoot::new([2u8; 32]),
        prev_settlement_root: SettlementStateRoot::settlement_v1([1u8; 32]),
        new_settlement_root: SettlementStateRoot::settlement_v1([2u8; 32]),
        claim_root: None,
        spent_delta: vec![SpentEnt::new([3u8; 32])],
        created_delta: vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
        prep_snapshot_id: None,
        exec_input_id: None,
        proof_sys: CheckpointProofSystem::OPAQUE_ATTEST,
        cp_proof: vec![8u8],
    };
    let bytes = BincodeCodec.serialize(&wire).expect("encode wire");
    let bad: CheckpointArtifact = BincodeCodec.deserialize(&bytes).expect("artifact shell");

    let id_err = derive_checkpoint_id(&bad).expect_err("attest shell must reject id derivation");
    let encode_err = encode_art_bin(&bad).expect_err("attest shell must reject encode");

    assert!(matches!(id_err, CheckpointError::ArtifactCompatMix));
    assert!(matches!(encode_err, CheckpointError::ArtifactCompatMix));
}

#[test]
fn test_proofless_rejects_id_encode() {
    let wire = ArtWire {
        version: CheckpointVersion::CURRENT,
        height: 22,
        prev_root: CheckRoot::new([1u8; 32]),
        new_root: CheckRoot::new([2u8; 32]),
        prev_settlement_root: SettlementStateRoot::settlement_v1([1u8; 32]),
        new_settlement_root: SettlementStateRoot::settlement_v1([2u8; 32]),
        claim_root: None,
        spent_delta: vec![SpentEnt::new([3u8; 32])],
        created_delta: vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
        prep_snapshot_id: None,
        exec_input_id: None,
        proof_sys: CheckpointProofSystem::OPAQUE_ATTEST,
        cp_proof: Vec::new(),
    };
    let bytes = BincodeCodec.serialize(&wire).expect("encode wire");
    let bad: CheckpointArtifact = BincodeCodec.deserialize(&bytes).expect("artifact shell");

    let id_err = derive_checkpoint_id(&bad).expect_err("proofless shell must reject id derivation");
    let encode_err = encode_art_bin(&bad).expect_err("proofless shell must reject encode");

    assert!(matches!(id_err, CheckpointError::ProoflessFinal));
    assert!(matches!(encode_err, CheckpointError::ProoflessFinal));
}

#[test]
fn test_partial_rejects_id_encode() {
    let wire = ArtWire {
        version: CheckpointVersion::CURRENT,
        height: 22,
        prev_root: CheckRoot::new([1u8; 32]),
        new_root: CheckRoot::new([2u8; 32]),
        prev_settlement_root: SettlementStateRoot::settlement_v1([1u8; 32]),
        new_settlement_root: SettlementStateRoot::settlement_v1([2u8; 32]),
        claim_root: None,
        spent_delta: vec![SpentEnt::new([3u8; 32])],
        created_delta: vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
        prep_snapshot_id: Some(PrepSnapshotId::new([6u8; 32])),
        exec_input_id: None,
        proof_sys: CheckpointProofSystem::OPAQUE_ATTEST,
        cp_proof: vec![8u8],
    };
    let bytes = BincodeCodec.serialize(&wire).expect("encode wire");
    let bad: CheckpointArtifact = BincodeCodec.deserialize(&bytes).expect("artifact shell");

    let id_err =
        derive_checkpoint_id(&bad).expect_err("partial stmt shell must reject id derivation");
    let encode_err = encode_art_bin(&bad).expect_err("partial stmt shell must reject encode");

    assert!(matches!(id_err, CheckpointError::ArtifactCompatMix));
    assert!(matches!(encode_err, CheckpointError::ArtifactCompatMix));
}

#[test]
fn test_exec_ignores_file_name() {
    let dir = temp_dir();
    let one = dir.path().join("stage_6_exec.bin");
    let two = dir.path().join("checkpoint_exec_copy.bin");
    let raw = b"canonical-exec-input";
    write_file(&one, raw).expect("write one");
    write_file(&two, raw).expect("write two");

    assert_eq!(
        derive_exec_id(&read_file(&one).expect("read one")),
        derive_exec_id(&read_file(&two).expect("read two"))
    );
}

#[test]
fn test_backend_mismatch_is_precise() {
    let art_err = check_art_key(CheckpointId::new([1u8; 32]), CheckpointId::new([2u8; 32]))
        .expect_err("artifact key mismatch must reject");
    let exec_err = check_exec_key(
        CheckpointExecInputId::new([1u8; 32]),
        CheckpointExecInputId::new([2u8; 32]),
    )
    .expect_err("exec key mismatch must reject");

    assert!(matches!(art_err, CheckpointError::KeyMix));
    assert!(matches!(exec_err, CheckpointError::KeyMix));
}

#[test]
fn test_separated_ids_differ_payload() {
    let draft = draft();
    let draft_bytes = BincodeCodec.serialize(&draft).expect("encode draft");

    assert_ne!(
        derive_draft_id(&draft).expect("draft id").into_bytes(),
        derive_exec_id(&draft_bytes).into_bytes()
    );
}

#[test]
fn test_checkpoint_id_proof_bytes() {
    let draft = draft();
    let stmt = CheckpointTransitionStatementV1::from_draft(
        &draft,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let one = draft
        .clone()
        .finalize(
            CheckpointProof::new_attest(stmt.clone(), stmt.backend_payload()).expect("proof one"),
        )
        .expect("artifact one");
    let err = CheckpointProof::new_attest(stmt, vec![8u8, 9u8])
        .expect_err("non-backend payload must reject");

    assert_eq!(
        derive_checkpoint_id(&one).expect("id one"),
        derive_checkpoint_id(&one).expect("id two")
    );
    assert!(matches!(err, CheckpointError::ProofMix));
}
