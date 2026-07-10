use crate::settlement::CheckRoot;
use crate::CheckpointError;

use super::{
    artifact_final::check_proof_sys, CheckpointDraft, CheckpointProof, CheckpointProofSystem,
    CheckpointPubIn, CheckpointTransitionStatementV1, CheckpointVersion, CreatedEnt, SpentEnt,
    WalletDraft,
};
use crate::checkpoint::CheckpointExecInputId;
use crate::snapshot::PrepSnapshotId;

struct DemoDraft;

impl WalletDraft for DemoDraft {
    fn draft_height(&self) -> u64 {
        17
    }

    fn draft_prev_root(&self) -> CheckRoot {
        CheckRoot::new([1u8; 32])
    }

    fn draft_new_root(&self) -> CheckRoot {
        CheckRoot::new([2u8; 32])
    }

    fn draft_spent(&self) -> Vec<SpentEnt> {
        vec![SpentEnt::new([3u8; 32])]
    }

    fn draft_created(&self) -> Vec<CreatedEnt> {
        vec![CreatedEnt::new([4u8; 32], [5u8; 32])]
    }
}

#[test]
fn test_draft_src_keeps_semantics() {
    let draft = CheckpointDraft::from_src(&DemoDraft);

    assert_eq!(draft.version(), CheckpointVersion::CURRENT);
    assert_eq!(draft.height(), 17);
    assert_eq!(draft.prev_root(), CheckRoot::new([1u8; 32]));
    assert_eq!(draft.new_root(), CheckRoot::new([2u8; 32]));
    assert_eq!(draft.spent_delta().len(), 1);
    assert_eq!(draft.created_delta().len(), 1);
}

#[test]
fn test_draft_is_proofless_type() {
    let draft = CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        7,
        CheckRoot::new([1u8; 32]),
        CheckRoot::new([2u8; 32]),
        vec![SpentEnt::new([3u8; 32])],
        vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
    );

    assert_eq!(draft.spent_delta()[0].terminal_id().into_bytes(), [3u8; 32]);
    assert_eq!(draft.created_delta()[0].leaf_hash(), &[5u8; 32]);
}

#[test]
fn test_final_rejects_empty_proof() {
    let draft = CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        7,
        CheckRoot::new([1u8; 32]),
        CheckRoot::new([2u8; 32]),
        vec![SpentEnt::new([3u8; 32])],
        vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
    );
    let stmt = CheckpointTransitionStatementV1::from_draft(
        &draft,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let err = CheckpointProof::new_attest(stmt, Vec::new()).expect_err("empty proof must reject");

    assert!(matches!(err, CheckpointError::ProoflessFinal));
}

#[test]
fn test_finalize_rejects_pub_mix() {
    let draft = CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        7,
        CheckRoot::new([1u8; 32]),
        CheckRoot::new([2u8; 32]),
        vec![SpentEnt::new([3u8; 32])],
        vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
    );
    let bad_pub = CheckpointPubIn::new(
        CheckRoot::new([9u8; 32]),
        CheckRoot::new([2u8; 32]),
        vec![SpentEnt::new([3u8; 32])],
        vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
    );
    let bad_stmt = CheckpointTransitionStatementV1::new(
        CheckpointVersion::CURRENT,
        draft.height(),
        bad_pub,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let proof =
        CheckpointProof::new_attest(bad_stmt.clone(), bad_stmt.backend_payload()).expect("proof");

    let err = draft.finalize(proof).expect_err("pub-in mix must reject");

    assert!(matches!(err, CheckpointError::ProofMix));
}

#[test]
fn test_unsupported_proof_sys_rejects() {
    let err = check_proof_sys(CheckpointProofSystem::new(9)).expect_err("bad proof sys");

    assert!(matches!(err, CheckpointError::ProofSysMix));
}

#[test]
fn test_verified_proof_sys_remains_reserved() {
    let err =
        check_proof_sys(CheckpointProofSystem::VERIFIED).expect_err("verified proof sys reserved");

    assert!(matches!(err, CheckpointError::ProofSysMix));
}
