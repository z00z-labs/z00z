use crate::{
    checkpoint::CheckpointExecInputId,
    error::CheckpointError,
    settlement::{CheckRoot, ClaimSourceRoot, SettlementStateRoot},
    snapshot::PrepSnapshotId,
};

use super::{
    CheckpointProofSystem, CheckpointPubIn, CheckpointStatement,
    CheckpointTransitionStatementCoreV1, CheckpointTransitionStatementFinalV1,
    CheckpointTransitionStatementV1, CheckpointVersion, CreatedEnt, SpentEnt,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointArtifact {
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
    #[serde(default)]
    prep_snapshot_id: Option<PrepSnapshotId>,
    #[serde(default)]
    exec_input_id: Option<CheckpointExecInputId>,
    #[serde(default)]
    statement_core: Option<CheckpointTransitionStatementCoreV1>,
    #[serde(default)]
    da_ref: Option<[u8; 32]>,
    proof_sys: CheckpointProofSystem,
    cp_proof: Vec<u8>,
}

impl CheckpointArtifact {
    pub(crate) fn new_attest(
        stmt: CheckpointTransitionStatementV1,
        cp_proof: Vec<u8>,
    ) -> Result<Self, CheckpointError> {
        check_ver(stmt.checkpoint_version())?;
        check_attest_sys(CheckpointProofSystem::OPAQUE_ATTEST)?;
        if cp_proof.is_empty() {
            return Err(CheckpointError::ProoflessFinal);
        }
        if cp_proof != stmt.backend_payload() {
            return Err(CheckpointError::ProofMix);
        }

        Ok(Self {
            version: stmt.checkpoint_version(),
            height: stmt.height(),
            prev_root: stmt.prev_root(),
            new_root: stmt.new_root(),
            prev_settlement_root: stmt.prev_settlement_root(),
            new_settlement_root: stmt.new_settlement_root(),
            claim_root: stmt.claim_root(),
            spent_delta: stmt.spent_delta().to_vec(),
            created_delta: stmt.created_delta().to_vec(),
            prep_snapshot_id: Some(stmt.prep_snapshot_id()),
            exec_input_id: Some(stmt.exec_input_id()),
            statement_core: None,
            da_ref: None,
            proof_sys: CheckpointProofSystem::OPAQUE_ATTEST,
            cp_proof,
        })
    }

    #[must_use]
    /// Return the canonical checkpoint statement that binds artifact identity.
    ///
    /// The statement carries the canonical roots and replay references. The
    /// proof bytes remain verifier payload only.
    pub fn statement(&self) -> CheckpointStatement {
        match (self.prep_snapshot_id, self.exec_input_id) {
            (Some(prep_snapshot_id), Some(exec_input_id)) => {
                CheckpointStatement::V1(Box::new(CheckpointTransitionStatementV1::new(
                    self.version,
                    self.height,
                    self.pub_in(),
                    prep_snapshot_id,
                    exec_input_id,
                )))
            }
            _ => CheckpointStatement::Detached,
        }
    }

    pub(crate) const fn has_partial_stmt_ids(&self) -> bool {
        self.prep_snapshot_id.is_some() ^ self.exec_input_id.is_some()
    }

    pub(crate) const fn has_partial_canonical_binding(&self) -> bool {
        self.statement_core.is_some() ^ self.da_ref.is_some()
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
    pub const fn claim_root(&self) -> Option<ClaimSourceRoot> {
        self.claim_root
    }

    #[must_use]
    pub const fn statement_core(&self) -> Option<CheckpointTransitionStatementCoreV1> {
        self.statement_core
    }

    #[must_use]
    pub const fn da_ref(&self) -> Option<[u8; 32]> {
        self.da_ref
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
    pub const fn proof_sys(&self) -> CheckpointProofSystem {
        self.proof_sys
    }

    #[must_use]
    /// Return the verifier-bound proof payload bytes.
    ///
    /// These bytes do not replace the canonical statement-owned binding or the
    /// persisted replay evidence referenced by attested artifacts.
    pub fn cp_proof(&self) -> &[u8] {
        &self.cp_proof
    }

    pub fn bind_canonical_v1(
        mut self,
        statement_core: CheckpointTransitionStatementCoreV1,
        final_bind: CheckpointTransitionStatementFinalV1,
    ) -> Result<Self, CheckpointError> {
        if final_bind.pq_anchor_root().is_some() || final_bind.da_ref() == [0u8; 32] {
            return Err(CheckpointError::ArtifactCompatMix);
        }
        self.statement_core = Some(statement_core);
        self.da_ref = Some(final_bind.da_ref());
        Ok(self)
    }

    #[must_use]
    pub fn statement_digest_v1(&self) -> Option<[u8; 32]> {
        match (self.statement(), self.statement_core, self.da_ref) {
            (CheckpointStatement::V1(stmt), Some(statement_core), Some(da_ref)) => {
                Some(stmt.final_statement_digest_v1(
                    &statement_core,
                    &CheckpointTransitionStatementFinalV1::new(da_ref),
                ))
            }
            _ => None,
        }
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
}

pub(crate) fn check_ver(version: CheckpointVersion) -> Result<(), CheckpointError> {
    if version == CheckpointVersion::CURRENT {
        return Ok(());
    }

    Err(CheckpointError::VersionMix)
}

pub(crate) fn check_proof_sys(proof_sys: CheckpointProofSystem) -> Result<(), CheckpointError> {
    if proof_sys.is_opaque_attest() {
        return Ok(());
    }

    Err(CheckpointError::ProofSysMix)
}

pub(super) fn check_attest_sys(proof_sys: CheckpointProofSystem) -> Result<(), CheckpointError> {
    if proof_sys.is_opaque_attest() {
        return Ok(());
    }

    if proof_sys.claims_verified() {
        return Err(CheckpointError::ProofSysMix);
    }

    Err(CheckpointError::ArtifactCompatMix)
}
