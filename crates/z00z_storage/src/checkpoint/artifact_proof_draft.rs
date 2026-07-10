use crate::{
    checkpoint::CheckpointExecInputId,
    error::CheckpointError,
    settlement::{CheckRoot, ClaimSourceRoot, SettlementStateRoot},
    snapshot::PrepSnapshotId,
};

use super::{
    artifact_final::{check_attest_sys, check_ver},
    CheckpointArtifact, CheckpointProofSystem, CheckpointPubIn, CheckpointTransitionStatementV1,
    CheckpointVersion, CreatedEnt, SpentEnt, WalletDraft,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointProof {
    proof_sys: CheckpointProofSystem,
    pub_in: CheckpointPubIn,
    stmt: CheckpointTransitionStatementV1,
    attest_payload_bytes: Vec<u8>,
}

impl CheckpointProof {
    pub fn new_attest(
        stmt: CheckpointTransitionStatementV1,
        attest_payload_bytes: Vec<u8>,
    ) -> Result<Self, CheckpointError> {
        check_ver(stmt.checkpoint_version())?;
        check_attest_sys(CheckpointProofSystem::OPAQUE_ATTEST)?;
        if attest_payload_bytes.is_empty() {
            return Err(CheckpointError::ProoflessFinal);
        }
        if attest_payload_bytes != stmt.backend_payload() {
            return Err(CheckpointError::ProofMix);
        }

        Ok(Self {
            proof_sys: CheckpointProofSystem::OPAQUE_ATTEST,
            pub_in: stmt.pub_in(),
            stmt,
            attest_payload_bytes,
        })
    }

    #[must_use]
    pub const fn proof_sys(&self) -> CheckpointProofSystem {
        self.proof_sys
    }

    #[must_use]
    /// Return the canonical attestation statement when this proof is statement-bound.
    pub fn statement(&self) -> &CheckpointTransitionStatementV1 {
        &self.stmt
    }

    #[must_use]
    pub fn pub_in(&self) -> CheckpointPubIn {
        self.pub_in.clone()
    }

    #[must_use]
    /// Return the verifier-bound proof payload bytes.
    ///
    /// These bytes are not the canonical replay-evidence binding and do not
    /// define checkpoint identity without the attested statement.
    pub fn attest_payload_bytes(&self) -> &[u8] {
        &self.attest_payload_bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointDraft {
    version: CheckpointVersion,
    height: u64,
    prev_root: CheckRoot,
    new_root: CheckRoot,
    prev_settlement_root: SettlementStateRoot,
    new_settlement_root: SettlementStateRoot,
    #[serde(default)]
    claim_root: Option<ClaimSourceRoot>,
    spent_delta: Vec<SpentEnt>,
    created_delta: Vec<CreatedEnt>,
}

impl CheckpointDraft {
    #[must_use]
    pub fn new(
        version: CheckpointVersion,
        height: u64,
        prev_root: CheckRoot,
        new_root: CheckRoot,
        spent_delta: Vec<SpentEnt>,
        created_delta: Vec<CreatedEnt>,
    ) -> Self {
        Self::new_settlement(
            version,
            height,
            SettlementStateRoot::settlement_v1(prev_root.into_bytes()),
            SettlementStateRoot::settlement_v1(new_root.into_bytes()),
            spent_delta,
            created_delta,
        )
    }

    #[must_use]
    pub fn new_settlement(
        version: CheckpointVersion,
        height: u64,
        prev_settlement_root: SettlementStateRoot,
        new_settlement_root: SettlementStateRoot,
        spent_delta: Vec<SpentEnt>,
        created_delta: Vec<CreatedEnt>,
    ) -> Self {
        Self {
            version,
            height,
            prev_root: CheckRoot::from(prev_settlement_root),
            new_root: CheckRoot::from(new_settlement_root),
            prev_settlement_root,
            new_settlement_root,
            claim_root: None,
            spent_delta,
            created_delta,
        }
    }

    #[must_use]
    pub fn with_claim_root(mut self, claim_root: ClaimSourceRoot) -> Self {
        self.claim_root = Some(claim_root);
        self
    }

    #[must_use]
    pub fn from_src(src: &impl WalletDraft) -> Self {
        let mut draft = Self::new_settlement(
            CheckpointVersion::CURRENT,
            src.draft_height(),
            src.draft_prev_settlement_root(),
            src.draft_new_settlement_root(),
            src.draft_spent(),
            src.draft_created(),
        );
        if let Some(claim_root) = src.draft_claim_root() {
            draft = draft.with_claim_root(claim_root);
        }
        draft
    }

    #[must_use]
    pub const fn version(&self) -> CheckpointVersion {
        self.version
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub const fn prev_root(&self) -> CheckRoot {
        self.prev_root
    }

    #[must_use]
    pub const fn new_root(&self) -> CheckRoot {
        self.new_root
    }

    #[must_use]
    pub const fn prev_settlement_root(&self) -> SettlementStateRoot {
        self.prev_settlement_root
    }

    #[must_use]
    pub const fn new_settlement_root(&self) -> SettlementStateRoot {
        self.new_settlement_root
    }

    #[must_use]
    pub fn spent_delta(&self) -> &[SpentEnt] {
        &self.spent_delta
    }

    #[must_use]
    pub fn created_delta(&self) -> &[CreatedEnt] {
        &self.created_delta
    }

    #[must_use]
    pub const fn claim_root(&self) -> Option<ClaimSourceRoot> {
        self.claim_root
    }

    #[must_use]
    pub fn attest_stmt(
        &self,
        prep_snapshot_id: PrepSnapshotId,
        exec_input_id: CheckpointExecInputId,
    ) -> CheckpointTransitionStatementV1 {
        CheckpointTransitionStatementV1::from_draft(self, prep_snapshot_id, exec_input_id)
    }

    pub fn attest_proof(
        &self,
        prep_snapshot_id: PrepSnapshotId,
        exec_input_id: CheckpointExecInputId,
    ) -> Result<CheckpointProof, CheckpointError> {
        let stmt = self.attest_stmt(prep_snapshot_id, exec_input_id);
        CheckpointProof::new_attest(stmt.clone(), stmt.backend_payload())
    }

    #[must_use]
    pub fn pub_in(&self) -> CheckpointPubIn {
        let mut pub_in = CheckpointPubIn::new_settlement(
            self.prev_settlement_root,
            self.new_settlement_root,
            self.spent_delta.clone(),
            self.created_delta.clone(),
        );
        if let Some(claim_root) = self.claim_root {
            pub_in = pub_in.with_claim_root(claim_root);
        }
        pub_in
    }

    pub fn finalize(&self, proof: CheckpointProof) -> Result<CheckpointArtifact, CheckpointError> {
        let CheckpointProof {
            proof_sys: _,
            pub_in,
            stmt,
            attest_payload_bytes,
        } = proof;

        if pub_in != self.pub_in() {
            return Err(CheckpointError::ProofMix);
        }

        if !stmt.matches_draft(self) {
            return Err(CheckpointError::ProofMix);
        }
        if attest_payload_bytes != stmt.backend_payload() {
            return Err(CheckpointError::ProofMix);
        }

        CheckpointArtifact::new_attest(stmt, attest_payload_bytes)
    }
}
