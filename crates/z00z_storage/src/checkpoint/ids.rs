use z00z_crypto::expert::hash_domain;
use z00z_crypto::hash_zk::hash_zk;
use z00z_utils::codec::{BincodeCodec, Codec};

use crate::error::CheckpointError;

use super::{
    artifact_final::{check_proof_sys, check_ver},
    artifact_stmt::{CheckpointStatement, CheckpointTransitionStatementV1},
    artifact_types::CheckpointProofSystem,
    codec::{check_artifact_contract, encode_draft_bin},
    CheckpointArtifact, CheckpointDraft,
};

hash_domain!(StorCheckpointIdDom, "z00z.storage.checkpoint.id", 1);

const DRAFT_ID_LABEL: &str = "checkpoint_draft_id";
const CHECKPOINT_ID_LABEL: &str = "checkpoint_final_id";
const EXEC_ID_LABEL: &str = "checkpoint_exec_id";
const ID_CLASS_DRAFT: &[u8] = b"draft";
const ID_CLASS_CHECKPOINT: &[u8] = b"checkpoint";
const ID_CLASS_EXEC: &[u8] = b"exec_input";

#[derive(serde::Serialize)]
enum CheckpointIdInput {
    V1Opaque {
        proof_sys: CheckpointProofSystem,
        stmt: CheckpointTransitionStatementV1,
    },
    V1Canonical {
        proof_sys: CheckpointProofSystem,
        statement_digest: [u8; 32],
    },
}

/// External content-addressed identifier for one canonical checkpoint draft.
///
/// # Examples
///
/// ```
/// use z00z_storage::checkpoint::CheckpointDraftId;
///
/// let draft_id = CheckpointDraftId::new([8u8; 32]);
/// assert_eq!(draft_id.as_bytes(), &[8u8; 32]);
/// ```
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckpointDraftId([u8; 32]);

impl CheckpointDraftId {
    /// Build one external draft identifier from canonical draft bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Consume the identifier and return its raw bytes.
    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    /// Borrow the raw identifier bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for CheckpointDraftId {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

/// External content-addressed identifier for one canonical final checkpoint artifact.
///
/// # Examples
///
/// ```
/// use z00z_storage::checkpoint::CheckpointId;
///
/// let checkpoint_id = CheckpointId::new([8u8; 32]);
/// assert_eq!(checkpoint_id.as_bytes(), &[8u8; 32]);
/// ```
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckpointId([u8; 32]);

impl CheckpointId {
    /// Build one external checkpoint identifier from canonical final bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Consume the identifier and return its raw bytes.
    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    /// Borrow the raw identifier bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for CheckpointId {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

/// External content-addressed identifier for one canonical checkpoint execution-input artifact.
///
/// # Examples
///
/// ```
/// use z00z_storage::checkpoint::{derive_exec_id, CheckpointExecInputId};
///
/// let want = derive_exec_id(b"canonical-exec-input");
/// let got = CheckpointExecInputId::new(want.into_bytes());
/// assert_eq!(want, got);
/// ```
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckpointExecInputId([u8; 32]);

impl CheckpointExecInputId {
    /// Build one external execution-input identifier from canonical bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Consume the identifier and return its raw bytes.
    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    /// Borrow the raw identifier bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for CheckpointExecInputId {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

/// Derive one checkpoint draft id from canonical draft bytes only.
pub fn derive_draft_id(draft: &CheckpointDraft) -> Result<CheckpointDraftId, CheckpointError> {
    let bytes = encode_draft_bin(draft)?;
    Ok(CheckpointDraftId::new(typed_id(
        DRAFT_ID_LABEL,
        ID_CLASS_DRAFT,
        &bytes,
    )))
}

/// Derive one checkpoint id from canonical final artifact bytes only.
///
/// Forbidden identity inputs: filenames, report paths, row numbers,
/// `fragment_ids`, and `prep_snapshot_id` taken without canonical final bytes.
pub fn derive_checkpoint_id(
    artifact: &CheckpointArtifact,
) -> Result<CheckpointId, CheckpointError> {
    check_ver(artifact.version())?;
    check_proof_sys(artifact.proof_sys())?;
    check_artifact_contract(artifact)?;
    let bytes = checkpoint_id_input_bytes(artifact)?;
    Ok(CheckpointId::new(typed_id(
        CHECKPOINT_ID_LABEL,
        ID_CLASS_CHECKPOINT,
        &bytes,
    )))
}

/// Reject draft artifacts on the final-artifact id path.
pub fn reject_draft_for_checkpoint_id(
    _draft: &CheckpointDraft,
) -> Result<CheckpointId, CheckpointError> {
    Err(CheckpointError::WrongClass)
}

/// Derive one execution-input id from canonical execution-input bytes only.
///
/// Forbidden identity inputs: filenames, report paths, row numbers,
/// `fragment_ids`, and `prep_snapshot_id` taken without canonical execution-input bytes.
#[must_use]
pub fn derive_exec_id(bytes: &[u8]) -> CheckpointExecInputId {
    CheckpointExecInputId::new(typed_id(EXEC_ID_LABEL, ID_CLASS_EXEC, bytes))
}

fn checkpoint_id_input_bytes(artifact: &CheckpointArtifact) -> Result<Vec<u8>, CheckpointError> {
    let codec = BincodeCodec;
    let input = if let Some(statement_digest) = artifact.statement_digest_v1() {
        CheckpointIdInput::V1Canonical {
            proof_sys: artifact.proof_sys(),
            statement_digest,
        }
    } else {
        match artifact.statement() {
            CheckpointStatement::Detached => return Err(CheckpointError::ArtifactCompatMix),
            CheckpointStatement::V1(stmt) => CheckpointIdInput::V1Opaque {
                proof_sys: artifact.proof_sys(),
                stmt: *stmt,
            },
        }
    };
    codec.serialize(&input).map_err(CheckpointError::from)
}

fn typed_id(label: &'static str, class: &[u8], payload: &[u8]) -> [u8; 32] {
    hash_zk::<StorCheckpointIdDom>(label, &[class, payload])
}

#[cfg(test)]
mod tests {
    use z00z_utils::codec::{BincodeCodec, Codec};

    use super::{derive_checkpoint_id, derive_draft_id, derive_exec_id};
    use crate::{
        checkpoint::{
            CheckpointArtifact, CheckpointDraft, CheckpointExecInputId, CheckpointProof,
            CheckpointProofSystem, CheckpointTransitionStatementCoreV1,
            CheckpointTransitionStatementV1, CheckpointVersion, CreatedEnt, SpentEnt,
        },
        settlement::{CheckRoot, SettlementStateRoot},
        CheckpointError,
    };

    fn draft() -> CheckpointDraft {
        CheckpointDraft::new(
            CheckpointVersion::CURRENT,
            9,
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
                crate::snapshot::PrepSnapshotId::new([7u8; 32]),
                CheckpointExecInputId::new([8u8; 32]),
            )
            .expect("proof");
        draft.finalize(proof).expect("checkpoint artifact")
    }

    #[test]
    fn test_draft_stable_across_reencode() {
        let first = derive_draft_id(&draft()).expect("first draft id");
        let bytes = BincodeCodec.serialize(&draft()).expect("encode draft");
        let decoded: CheckpointDraft = BincodeCodec.deserialize(&bytes).expect("decode draft");
        let second = derive_draft_id(&decoded).expect("second draft id");

        assert_eq!(first, second);
    }

    #[test]
    fn test_art_stable_across_reencode() {
        let first = derive_checkpoint_id(&art()).expect("first id");
        let bytes = BincodeCodec.serialize(&art()).expect("encode artifact");
        let decoded: CheckpointArtifact =
            BincodeCodec.deserialize(&bytes).expect("decode artifact");
        let second = derive_checkpoint_id(&decoded).expect("second id");

        assert_eq!(first, second);
    }

    #[test]
    fn test_unsupported_rejects_art_id() {
        #[derive(serde::Serialize)]
        struct UnsupportedVersionArtWire {
            version: CheckpointVersion,
            height: u64,
            prev_root: CheckRoot,
            new_root: CheckRoot,
            prev_settlement_root: SettlementStateRoot,
            new_settlement_root: SettlementStateRoot,
            claim_root: Option<crate::settlement::ClaimSourceRoot>,
            spent_delta: Vec<SpentEnt>,
            created_delta: Vec<CreatedEnt>,
            prep_snapshot_id: Option<crate::snapshot::PrepSnapshotId>,
            exec_input_id: Option<CheckpointExecInputId>,
            statement_core: Option<CheckpointTransitionStatementCoreV1>,
            da_ref: Option<[u8; 32]>,
            proof_sys: CheckpointProofSystem,
            cp_proof: Vec<u8>,
        }

        let wire = UnsupportedVersionArtWire {
            version: CheckpointVersion::new(9),
            height: 9,
            prev_root: CheckRoot::new([1u8; 32]),
            new_root: CheckRoot::new([2u8; 32]),
            prev_settlement_root: SettlementStateRoot::settlement_v1([1u8; 32]),
            new_settlement_root: SettlementStateRoot::settlement_v1([2u8; 32]),
            claim_root: None,
            spent_delta: vec![SpentEnt::new([3u8; 32])],
            created_delta: vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
            prep_snapshot_id: None,
            exec_input_id: None,
            statement_core: None,
            da_ref: None,
            proof_sys: CheckpointProofSystem::OPAQUE_ATTEST,
            cp_proof: vec![8u8],
        };
        let bytes = BincodeCodec.serialize(&wire).expect("encode wire");
        let bad: CheckpointArtifact = BincodeCodec.deserialize(&bytes).expect("artifact shell");

        let err = derive_checkpoint_id(&bad).expect_err("unsupported version must reject");

        assert!(matches!(err, CheckpointError::VersionMix));
    }

    #[test]
    fn test_exec_stable_same_bytes() {
        let first = derive_exec_id(b"exec-input-v1");
        let second = derive_exec_id(b"exec-input-v1");

        assert_eq!(first, second);
    }

    #[test]
    fn test_art_id_proof_bytes() {
        let draft = draft();
        let stmt = CheckpointTransitionStatementV1::from_draft(
            &draft,
            crate::snapshot::PrepSnapshotId::new([7u8; 32]),
            CheckpointExecInputId::new([8u8; 32]),
        );
        let one = draft
            .clone()
            .finalize(
                CheckpointProof::new_attest(stmt.clone(), stmt.backend_payload())
                    .expect("proof one"),
            )
            .expect("artifact one");
        let err = CheckpointProof::new_attest(stmt, vec![10u8, 11u8])
            .expect_err("non-backend payload must reject");

        assert_eq!(
            derive_checkpoint_id(&one).expect("id one"),
            derive_checkpoint_id(&one).expect("id two")
        );
        assert!(matches!(err, CheckpointError::ProofMix));
    }
}
