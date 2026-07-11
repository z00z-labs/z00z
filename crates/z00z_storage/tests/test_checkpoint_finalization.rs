use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, encode_art_bin, load_artifact, CheckpointDraft,
        CheckpointExecInputId, CheckpointProof, CheckpointProofSystem, CheckpointPubIn,
        CheckpointTransitionStatementV1, CheckpointVersion, CreatedEnt, SpentEnt,
    },
    settlement::{CheckRoot, ClaimSourceRoot, SettlementStateRoot},
    snapshot::PrepSnapshotId,
    CheckpointError,
};
use z00z_utils::codec::{BincodeCodec, Codec};

const DRAFT_PROOF_SRC: &str = include_str!("../src/checkpoint/artifact_proof_draft.rs");
const FINAL_ARTIFACT_SRC: &str = include_str!("../src/checkpoint/artifact_final.rs");

fn draft() -> CheckpointDraft {
    CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        61,
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

#[derive(serde::Serialize)]
struct StatementlessArtWire {
    version: CheckpointVersion,
    height: u64,
    prev_root: CheckRoot,
    new_root: CheckRoot,
    prev_settlement_root: SettlementStateRoot,
    new_settlement_root: SettlementStateRoot,
    claim_root: Option<z00z_storage::settlement::ClaimSourceRoot>,
    spent_delta: Vec<SpentEnt>,
    created_delta: Vec<CreatedEnt>,
    statement_core: Option<z00z_storage::checkpoint::CheckpointTransitionStatementCoreV1>,
    da_ref: Option<[u8; 32]>,
    proof_sys: CheckpointProofSystem,
    cp_proof: Vec<u8>,
}

#[derive(serde::Serialize)]
struct UnsupportedArtWire {
    version: CheckpointVersion,
    height: u64,
    prev_root: CheckRoot,
    new_root: CheckRoot,
    prev_settlement_root: SettlementStateRoot,
    new_settlement_root: SettlementStateRoot,
    claim_root: Option<z00z_storage::settlement::ClaimSourceRoot>,
    spent_delta: Vec<SpentEnt>,
    created_delta: Vec<CreatedEnt>,
    prep_snapshot_id: Option<PrepSnapshotId>,
    exec_input_id: Option<CheckpointExecInputId>,
    statement_core: Option<z00z_storage::checkpoint::CheckpointTransitionStatementCoreV1>,
    da_ref: Option<[u8; 32]>,
    proof_sys: CheckpointProofSystem,
    cp_proof: Vec<u8>,
}

#[test]
fn test_empty_proof_rejects() {
    let draft = draft();
    let stmt = CheckpointTransitionStatementV1::from_draft(
        &draft,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let err =
        CheckpointProof::new_attest(stmt, Vec::new()).expect_err("empty attestation must reject");

    assert!(matches!(err, CheckpointError::ProoflessFinal));
}

#[test]
fn test_tampered_pub_in_rejects() {
    let draft = draft();
    let bad_stmt = CheckpointTransitionStatementV1::new(
        CheckpointVersion::CURRENT,
        draft.height(),
        CheckpointPubIn::new(
            CheckRoot::new([9u8; 32]),
            draft.new_root(),
            draft.spent_delta().to_vec(),
            draft.created_delta().to_vec(),
        ),
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let proof =
        CheckpointProof::new_attest(bad_stmt.clone(), bad_stmt.backend_payload()).expect("proof");

    let err = draft
        .finalize(proof)
        .expect_err("tampered pub-in must reject");

    assert!(matches!(err, CheckpointError::ProofMix));
}

#[test]
fn test_claim_root_changes_payload() {
    let claim_root =
        ClaimSourceRoot::new_settlement(1, SettlementStateRoot::settlement_v1([0xA1; 32]));
    let other_claim_root =
        ClaimSourceRoot::new_settlement(1, SettlementStateRoot::settlement_v1([0xB2; 32]));
    let draft = draft().with_claim_root(claim_root);
    let stmt = CheckpointTransitionStatementV1::from_draft(
        &draft,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let other_stmt = CheckpointTransitionStatementV1::new(
        CheckpointVersion::CURRENT,
        draft.height(),
        CheckpointPubIn::new(
            draft.prev_root(),
            draft.new_root(),
            draft.spent_delta().to_vec(),
            draft.created_delta().to_vec(),
        )
        .with_claim_root(other_claim_root),
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );

    assert_ne!(stmt.backend_payload(), other_stmt.backend_payload());

    let err = CheckpointProof::new_attest(stmt, other_stmt.backend_payload())
        .expect_err("claim_root-mixed payload must reject");

    assert!(matches!(err, CheckpointError::ProofMix));
}

#[test]
fn test_finalization_is_deterministic() {
    let draft = draft();
    let art_one = draft.finalize(proof(&draft, vec![9u8])).expect("art one");
    let art_two = draft.finalize(proof(&draft, vec![9u8])).expect("art two");

    assert_eq!(art_one, art_two);
    assert_eq!(
        encode_art_bin(&art_one).expect("bytes one"),
        encode_art_bin(&art_two).expect("bytes two")
    );
    assert_eq!(
        derive_checkpoint_id(&art_one).expect("id one"),
        derive_checkpoint_id(&art_two).expect("id two")
    );
}

#[test]
fn test_attested_proof_backend_payload() {
    let draft = draft();
    let stmt = CheckpointTransitionStatementV1::from_draft(
        &draft,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let err = CheckpointProof::new_attest(stmt, vec![8u8]).expect_err("bad payload must reject");

    assert!(matches!(err, CheckpointError::ProofMix));
}

#[test]
fn test_load_artifact_proof_system() {
    let draft = draft();
    let stmt = CheckpointTransitionStatementV1::from_draft(
        &draft,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let bytes = BincodeCodec
        .serialize(&UnsupportedArtWire {
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
            statement_core: None,
            da_ref: None,
            proof_sys: CheckpointProofSystem::new(9),
            cp_proof: stmt.backend_payload(),
        })
        .expect("unsupported proof-system bytes");

    let err = load_artifact(&bytes).expect_err("unsupported proof system must reject load surface");

    assert!(matches!(err, CheckpointError::ProofSysMix));
}

#[test]
fn test_load_artifact_verified_proof_system_rejects() {
    let draft = draft();
    let stmt = CheckpointTransitionStatementV1::from_draft(
        &draft,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let bytes = BincodeCodec
        .serialize(&UnsupportedArtWire {
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
            statement_core: None,
            da_ref: None,
            proof_sys: CheckpointProofSystem::VERIFIED,
            cp_proof: stmt.backend_payload(),
        })
        .expect("verified proof-system bytes");

    let err = load_artifact(&bytes).expect_err("verified proof system must stay reserved");

    assert!(matches!(err, CheckpointError::ProofSysMix));
}

#[test]
fn test_statementless_artifact_load_surface() {
    let draft = draft();
    let bytes = BincodeCodec
        .serialize(&StatementlessArtWire {
            version: CheckpointVersion::CURRENT,
            height: draft.height(),
            prev_root: draft.prev_root(),
            new_root: draft.new_root(),
            prev_settlement_root: draft.prev_settlement_root(),
            new_settlement_root: draft.new_settlement_root(),
            claim_root: draft.claim_root(),
            spent_delta: draft.spent_delta().to_vec(),
            created_delta: draft.created_delta().to_vec(),
            statement_core: None,
            da_ref: None,
            proof_sys: CheckpointProofSystem::OPAQUE_ATTEST,
            cp_proof: vec![9u8],
        })
        .expect("statementless bytes");
    let err = load_artifact(&bytes).expect_err("statementless artifact must reject load surface");

    assert!(matches!(
        err,
        CheckpointError::ArtifactCompatMix | CheckpointError::Codec(_)
    ));
}

#[test]
fn test_attest_payload_naming_stays_split() {
    assert!(DRAFT_PROOF_SRC.contains("attest_payload_bytes"));
    assert!(!DRAFT_PROOF_SRC.contains("pub fn cp_proof(&self)"));
    assert!(FINAL_ARTIFACT_SRC.contains("pub fn cp_proof(&self)"));
}
