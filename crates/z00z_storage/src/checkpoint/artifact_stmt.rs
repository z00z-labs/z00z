use z00z_crypto::{expert::hash_domain, frame_bytes, frame_str, hash_zk::hash_zk};

use crate::{
    checkpoint::{CheckpointExecInput, CheckpointExecInputId},
    settlement::{CheckRoot, ClaimSourceRoot, SettlementStateRoot},
    snapshot::PrepSnapshotId,
};

use super::{CheckpointDraft, CheckpointPubIn, CheckpointVersion, CreatedEnt, SpentEnt};

hash_domain!(
    StorCheckpointStatementDom,
    "z00z.storage.checkpoint.statement",
    1
);

const CHECKPOINT_TRANSITION_STATEMENT_V1_VERSION: u8 = 1;
const CHECKPOINT_TRANSITION_STATEMENT_V1_CORE_LABEL: &str =
    "checkpoint_transition_statement_core_v1";
const CHECKPOINT_TRANSITION_STATEMENT_V1_FINAL_LABEL: &str = "checkpoint_transition_statement_v1";
const CHECKPOINT_TRANSITION_STATEMENT_V1_PROOF_FAMILY: &str = "checkpoint_transition_shared_v1";

pub const CHECKPOINT_TRANSITION_STATEMENT_V1_DOMAIN: &str = "z00z.checkpoint.transition.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointTransitionStatementCoreV1 {
    tx_data_root: [u8; 32],
    delta_root: [u8; 32],
    witness_root: [u8; 32],
    journal_digest: [u8; 32],
    #[serde(default)]
    prior_recursive_output_root: Option<[u8; 32]>,
}

impl CheckpointTransitionStatementCoreV1 {
    #[must_use]
    pub fn new(
        tx_data_root: [u8; 32],
        delta_root: [u8; 32],
        witness_root: [u8; 32],
        journal_digest: [u8; 32],
    ) -> Self {
        Self {
            tx_data_root,
            delta_root,
            witness_root,
            journal_digest,
            prior_recursive_output_root: None,
        }
    }

    #[must_use]
    /// Build one statement-core input directly from the canonical execution-input root.
    pub fn from_exec(
        exec: &CheckpointExecInput,
        delta_root: [u8; 32],
        witness_root: [u8; 32],
        journal_digest: [u8; 32],
    ) -> Self {
        Self::new(
            exec.tx_data_root(),
            delta_root,
            witness_root,
            journal_digest,
        )
    }

    #[must_use]
    pub fn with_prior_recursive_output_root(
        mut self,
        prior_recursive_output_root: [u8; 32],
    ) -> Self {
        self.prior_recursive_output_root = Some(prior_recursive_output_root);
        self
    }

    #[must_use]
    pub const fn tx_data_root(&self) -> [u8; 32] {
        self.tx_data_root
    }

    #[must_use]
    pub const fn delta_root(&self) -> [u8; 32] {
        self.delta_root
    }

    #[must_use]
    pub const fn witness_root(&self) -> [u8; 32] {
        self.witness_root
    }

    #[must_use]
    pub const fn journal_digest(&self) -> [u8; 32] {
        self.journal_digest
    }

    #[must_use]
    pub const fn prior_recursive_output_root(&self) -> Option<[u8; 32]> {
        self.prior_recursive_output_root
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointTransitionStatementFinalV1 {
    da_ref: [u8; 32],
    #[serde(default)]
    pq_anchor_root: Option<[u8; 32]>,
}

impl CheckpointTransitionStatementFinalV1 {
    #[must_use]
    pub const fn new(da_ref: [u8; 32]) -> Self {
        Self {
            da_ref,
            pq_anchor_root: None,
        }
    }

    #[must_use]
    pub fn with_pq_anchor_root(mut self, pq_anchor_root: [u8; 32]) -> Self {
        self.pq_anchor_root = Some(pq_anchor_root);
        self
    }

    #[must_use]
    pub const fn da_ref(&self) -> [u8; 32] {
        self.da_ref
    }

    #[must_use]
    pub const fn pq_anchor_root(&self) -> Option<[u8; 32]> {
        self.pq_anchor_root
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointTransitionStatementV1 {
    checkpoint_version: CheckpointVersion,
    height: u64,
    prev_root: CheckRoot,
    new_root: CheckRoot,
    prev_settlement_root: SettlementStateRoot,
    new_settlement_root: SettlementStateRoot,
    claim_root: Option<ClaimSourceRoot>,
    spent_delta: Vec<SpentEnt>,
    created_delta: Vec<CreatedEnt>,
    prep_snapshot_id: PrepSnapshotId,
    exec_input_id: CheckpointExecInputId,
}

impl CheckpointTransitionStatementV1 {
    #[must_use]
    pub fn new(
        checkpoint_version: CheckpointVersion,
        height: u64,
        pub_in: CheckpointPubIn,
        prep_snapshot_id: PrepSnapshotId,
        exec_input_id: CheckpointExecInputId,
    ) -> Self {
        Self {
            checkpoint_version,
            height,
            prev_root: pub_in.prev_root(),
            new_root: pub_in.new_root(),
            prev_settlement_root: pub_in.prev_settlement_root(),
            new_settlement_root: pub_in.new_settlement_root(),
            claim_root: pub_in.claim_root(),
            spent_delta: pub_in.spent_delta().to_vec(),
            created_delta: pub_in.created_delta().to_vec(),
            prep_snapshot_id,
            exec_input_id,
        }
    }

    #[must_use]
    pub fn from_draft(
        draft: &CheckpointDraft,
        prep_snapshot_id: PrepSnapshotId,
        exec_input_id: CheckpointExecInputId,
    ) -> Self {
        Self::new(
            draft.version(),
            draft.height(),
            draft.pub_in(),
            prep_snapshot_id,
            exec_input_id,
        )
    }

    #[must_use]
    pub const fn statement_version() -> u8 {
        CHECKPOINT_TRANSITION_STATEMENT_V1_VERSION
    }

    #[must_use]
    pub const fn statement_domain() -> &'static str {
        CHECKPOINT_TRANSITION_STATEMENT_V1_DOMAIN
    }

    #[must_use]
    pub const fn proof_system_family_v1() -> &'static str {
        CHECKPOINT_TRANSITION_STATEMENT_V1_PROOF_FAMILY
    }

    #[must_use]
    pub const fn checkpoint_version(&self) -> CheckpointVersion {
        self.checkpoint_version
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
    pub fn spent_delta(&self) -> &[SpentEnt] {
        &self.spent_delta
    }

    #[must_use]
    pub fn created_delta(&self) -> &[CreatedEnt] {
        &self.created_delta
    }

    #[must_use]
    pub const fn prep_snapshot_id(&self) -> PrepSnapshotId {
        self.prep_snapshot_id
    }

    #[must_use]
    pub const fn exec_input_id(&self) -> CheckpointExecInputId {
        self.exec_input_id
    }

    #[must_use]
    pub fn canonical_bytes_v1(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_framed_field(
            &mut bytes,
            "statement_version",
            &[Self::statement_version()],
        );
        push_framed_field(
            &mut bytes,
            "statement_domain",
            Self::statement_domain().as_bytes(),
        );
        push_framed_field(
            &mut bytes,
            "proof_system_family",
            Self::proof_system_family_v1().as_bytes(),
        );
        push_framed_field(
            &mut bytes,
            "checkpoint_version",
            &[self.checkpoint_version.as_u8()],
        );
        push_framed_field(&mut bytes, "height", &self.height.to_le_bytes());
        push_framed_field(&mut bytes, "prev_root", self.prev_root.as_bytes());
        push_framed_field(&mut bytes, "new_root", self.new_root.as_bytes());
        push_framed_field(
            &mut bytes,
            "prev_settlement_root",
            &encode_settlement_root(self.prev_settlement_root),
        );
        push_framed_field(
            &mut bytes,
            "new_settlement_root",
            &encode_settlement_root(self.new_settlement_root),
        );
        push_framed_field(
            &mut bytes,
            "claim_root",
            &encode_optional_claim_root(self.claim_root),
        );
        push_framed_field(
            &mut bytes,
            "spent_delta",
            &encode_spent_delta(&self.spent_delta),
        );
        push_framed_field(
            &mut bytes,
            "created_delta",
            &encode_created_delta(&self.created_delta),
        );
        push_framed_field(
            &mut bytes,
            "prep_snapshot_id",
            self.prep_snapshot_id.as_bytes(),
        );
        push_framed_field(&mut bytes, "exec_input_id", self.exec_input_id.as_bytes());
        bytes
    }

    #[must_use]
    pub fn statement_core_preimage_v1(
        &self,
        core: &CheckpointTransitionStatementCoreV1,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_framed_field(
            &mut bytes,
            "statement_version",
            &[Self::statement_version()],
        );
        push_framed_field(
            &mut bytes,
            "statement_domain",
            Self::statement_domain().as_bytes(),
        );
        push_framed_field(
            &mut bytes,
            "proof_system_family",
            Self::proof_system_family_v1().as_bytes(),
        );
        push_framed_field(&mut bytes, "height", &self.height.to_le_bytes());
        push_framed_field(&mut bytes, "prev_root", self.prev_root.as_bytes());
        push_framed_field(
            &mut bytes,
            "prev_settlement_root",
            &encode_settlement_root(self.prev_settlement_root),
        );
        push_framed_field(
            &mut bytes,
            "checkpoint_exec_input_id",
            self.exec_input_id.as_bytes(),
        );
        push_framed_field(
            &mut bytes,
            "prep_snapshot_id",
            self.prep_snapshot_id.as_bytes(),
        );
        push_framed_field(&mut bytes, "tx_data_root", &core.tx_data_root);
        push_framed_field(&mut bytes, "delta_root", &core.delta_root);
        push_framed_field(&mut bytes, "witness_root", &core.witness_root);
        push_framed_field(&mut bytes, "journal_digest", &core.journal_digest);
        push_framed_field(
            &mut bytes,
            "claim_root",
            &encode_optional_claim_root(self.claim_root),
        );
        push_framed_field(
            &mut bytes,
            "prior_recursive_output_root",
            &encode_optional_digest(core.prior_recursive_output_root),
        );
        push_framed_field(&mut bytes, "new_root", self.new_root.as_bytes());
        push_framed_field(
            &mut bytes,
            "new_settlement_root",
            &encode_settlement_root(self.new_settlement_root),
        );
        bytes
    }

    #[must_use]
    pub fn statement_core_digest_v1(&self, core: &CheckpointTransitionStatementCoreV1) -> [u8; 32] {
        let bytes = self.statement_core_preimage_v1(core);
        hash_zk::<StorCheckpointStatementDom>(
            CHECKPOINT_TRANSITION_STATEMENT_V1_CORE_LABEL,
            &[bytes.as_slice()],
        )
    }

    #[must_use]
    pub fn final_statement_preimage_v1(
        core_digest: [u8; 32],
        final_bind: &CheckpointTransitionStatementFinalV1,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_framed_field(&mut bytes, "statement_core_digest", &core_digest);
        push_framed_field(&mut bytes, "da_ref", &final_bind.da_ref);
        push_framed_field(
            &mut bytes,
            "pq_anchor_root",
            &encode_optional_digest(final_bind.pq_anchor_root),
        );
        bytes
    }

    #[must_use]
    pub fn statement_digest_v1(
        core_digest: [u8; 32],
        final_bind: &CheckpointTransitionStatementFinalV1,
    ) -> [u8; 32] {
        let bytes = Self::final_statement_preimage_v1(core_digest, final_bind);
        hash_zk::<StorCheckpointStatementDom>(
            CHECKPOINT_TRANSITION_STATEMENT_V1_FINAL_LABEL,
            &[bytes.as_slice()],
        )
    }

    #[must_use]
    pub fn final_statement_digest_v1(
        &self,
        core: &CheckpointTransitionStatementCoreV1,
        final_bind: &CheckpointTransitionStatementFinalV1,
    ) -> [u8; 32] {
        Self::statement_digest_v1(self.statement_core_digest_v1(core), final_bind)
    }

    #[must_use]
    pub fn backend_payload(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(164);
        bytes.push(self.prev_settlement_root.generation_version());
        bytes.extend_from_slice(self.prev_settlement_root.as_bytes());
        bytes.push(self.new_settlement_root.generation_version());
        bytes.extend_from_slice(self.new_settlement_root.as_bytes());
        match self.claim_root {
            Some(claim_root) => {
                bytes.push(1);
                bytes.push(claim_root.root_version());
                bytes.extend_from_slice(claim_root.as_bytes());
            }
            None => {
                bytes.push(0);
                bytes.push(0);
                bytes.extend_from_slice(&[0u8; 32]);
            }
        }
        bytes.extend_from_slice(self.exec_input_id.as_bytes());
        bytes.extend_from_slice(self.new_root.as_bytes());
        bytes
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

    #[must_use]
    pub(super) fn matches_draft(&self, draft: &CheckpointDraft) -> bool {
        self.checkpoint_version == draft.version()
            && self.height == draft.height()
            && self.prev_root == draft.prev_root()
            && self.new_root == draft.new_root()
            && self.prev_settlement_root == draft.prev_settlement_root()
            && self.new_settlement_root == draft.new_settlement_root()
            && self.claim_root == draft.claim_root()
            && self.spent_delta == draft.spent_delta()
            && self.created_delta == draft.created_delta()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckpointStatement {
    Detached,
    V1(Box<CheckpointTransitionStatementV1>),
}

pub trait WalletDraft {
    fn draft_height(&self) -> u64;
    fn draft_prev_root(&self) -> CheckRoot;
    fn draft_new_root(&self) -> CheckRoot;
    fn draft_spent(&self) -> Vec<SpentEnt>;
    fn draft_created(&self) -> Vec<CreatedEnt>;

    fn draft_claim_root(&self) -> Option<ClaimSourceRoot> {
        None
    }

    fn draft_prev_settlement_root(&self) -> SettlementStateRoot {
        SettlementStateRoot::settlement_v1(self.draft_prev_root().into_bytes())
    }

    fn draft_new_settlement_root(&self) -> SettlementStateRoot {
        SettlementStateRoot::settlement_v1(self.draft_new_root().into_bytes())
    }
}

fn push_framed_field(out: &mut Vec<u8>, name: &str, value: &[u8]) {
    out.extend_from_slice(&frame_str(name));
    out.extend_from_slice(&frame_bytes(value));
}

fn encode_settlement_root(root: SettlementStateRoot) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(33);
    bytes.push(root.generation_version());
    bytes.extend_from_slice(root.as_bytes());
    bytes
}

fn encode_optional_claim_root(claim_root: Option<ClaimSourceRoot>) -> Vec<u8> {
    match claim_root {
        Some(claim_root) => {
            let mut bytes = Vec::with_capacity(34);
            bytes.push(1);
            bytes.push(claim_root.root_version());
            bytes.extend_from_slice(claim_root.as_bytes());
            bytes
        }
        None => vec![0],
    }
}

fn encode_optional_digest(digest: Option<[u8; 32]>) -> Vec<u8> {
    match digest {
        Some(digest) => {
            let mut bytes = Vec::with_capacity(33);
            bytes.push(1);
            bytes.extend_from_slice(&digest);
            bytes
        }
        None => vec![0],
    }
}

fn encode_spent_delta(spent_delta: &[SpentEnt]) -> Vec<u8> {
    let mut bytes = frame_bytes(&(spent_delta.len() as u32).to_le_bytes());
    for spent in spent_delta {
        bytes.extend_from_slice(&frame_bytes(spent.terminal_id().as_bytes()));
    }
    bytes
}

fn encode_created_delta(created_delta: &[CreatedEnt]) -> Vec<u8> {
    let mut bytes = frame_bytes(&(created_delta.len() as u32).to_le_bytes());
    for created in created_delta {
        bytes.extend_from_slice(&frame_bytes(created.terminal_id().as_bytes()));
        bytes.extend_from_slice(&frame_bytes(created.leaf_hash()));
    }
    bytes
}
