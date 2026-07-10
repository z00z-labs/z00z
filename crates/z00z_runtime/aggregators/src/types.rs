#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use z00z_storage::{
    checkpoint::{
        CheckpointDaReferenceV1, CheckpointExecTx, CheckpointLifecycleStatus,
        CheckpointLifecycleV1, CheckpointPublicationEvidenceV1,
    },
    settlement::{PublicationRouteSnapshotV1, SettlementExecHandoff, SettlementRouteCtx, StoreOp},
};
use z00z_storage::{
    checkpoint::{
        CheckpointDraft, CheckpointDraftId, CheckpointExecInput, CheckpointId, CheckpointLink,
        CheckpointPubIn,
    },
    settlement::{ClaimNullifier, ClaimSourceRoot, SettlementLeaf, SettlementStateRoot},
};
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_wallets::tx::{ClaimTxPackage, TxPackage};

use crate::{CommitSubject, ShardQuorumCertificate};

pub use z00z_storage::settlement::{
    ObjectWitnessBundleV1, RightWitnessRefV1, RightWitnessStateV1, RuntimeObjectPackageV1,
};

const WORK_ITEM_OBJECT_BINDING_LABEL: &[u8] = b"z00z.runtime.work-item.object-binding.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BatchId {
    pub draft_id: CheckpointDraftId,
}

impl BatchId {
    #[must_use]
    pub const fn new(draft_id: CheckpointDraftId) -> Self {
        Self { draft_id }
    }

    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self::new(CheckpointDraftId::new(bytes))
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.draft_id.into_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntakeId {
    digest_hex: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlannerMode {
    Central,
    PerAgg,
}

impl PlannerMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Central => "central",
            Self::PerAgg => "per_agg",
        }
    }
}

impl IntakeId {
    #[must_use]
    pub(crate) fn new(digest_hex: String) -> Self {
        Self { digest_hex }
    }

    #[must_use]
    pub fn digest_hex(&self) -> &str {
        &self.digest_hex
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkPayload {
    Tx(Box<TxPackage>),
    Claim(Box<ClaimTxPackage>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CanonicalDigest {
    hex: String,
    bytes: [u8; 32],
}

impl CanonicalDigest {
    #[must_use]
    pub(crate) fn new(hex: String, bytes: [u8; 32]) -> Self {
        Self { hex, bytes }
    }

    #[must_use]
    pub(crate) fn hex(&self) -> &str {
        &self.hex
    }

    #[must_use]
    pub(crate) const fn bytes(&self) -> [u8; 32] {
        self.bytes
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkItem {
    intake_id: IntakeId,
    payload_digest: CanonicalDigest,
    admission_digest: CanonicalDigest,
    payload: WorkPayload,
    object_package: Option<RuntimeObjectPackageV1>,
}

impl WorkItem {
    #[must_use]
    pub(crate) fn new(payload: WorkPayload, digest: CanonicalDigest) -> Self {
        let payload_digest = digest.clone();
        let admission_digest = digest;
        let intake_id = IntakeId::new(admission_digest.hex().to_string());
        Self {
            intake_id,
            payload_digest,
            admission_digest,
            payload,
            object_package: None,
        }
    }

    #[must_use]
    pub fn with_object_package(mut self, object_package: RuntimeObjectPackageV1) -> Self {
        self.admission_digest = bind_object_package_digest(&self.payload_digest, &object_package);
        self.intake_id = IntakeId::new(self.admission_digest.hex().to_string());
        self.object_package = Some(object_package);
        self
    }

    #[must_use]
    pub fn intake_id(&self) -> &IntakeId {
        &self.intake_id
    }

    #[must_use]
    pub fn digest_hex(&self) -> &str {
        self.intake_id.digest_hex()
    }

    #[must_use]
    pub(crate) const fn route_key(&self) -> [u8; 32] {
        self.payload_digest.bytes()
    }

    #[must_use]
    pub(crate) const fn admission_digest_bytes(&self) -> [u8; 32] {
        self.admission_digest.bytes()
    }

    #[must_use]
    pub fn payload(&self) -> &WorkPayload {
        &self.payload
    }

    #[must_use]
    pub fn object_package(&self) -> Option<&RuntimeObjectPackageV1> {
        self.object_package.as_ref()
    }

    #[must_use]
    pub fn kind_tag(&self) -> u8 {
        match &self.payload {
            WorkPayload::Tx(_) => 1,
            WorkPayload::Claim(_) => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ShardId(u16);

impl ShardId {
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.0
    }

    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0 as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlanDigest([u8; 32]);

impl PlanDigest {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BatchRoute {
    pub shard_id: ShardId,
    pub routing_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchPlanned {
    pub batch_id: BatchId,
    pub route: BatchRoute,
    pub route_table_digest: PlanDigest,
    pub intake_ids: Vec<IntakeId>,
    pub op_count: usize,
    pub plan_digest: PlanDigest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderedBatch {
    pub batch_id: BatchId,
    pub items: Vec<WorkItem>,
    pub created_leaves: Vec<SettlementLeaf>,
    pub planned: BatchPlanned,
}

impl OrderedBatch {
    pub fn object_packages(&self) -> impl Iterator<Item = &RuntimeObjectPackageV1> {
        self.items.iter().filter_map(WorkItem::object_package)
    }

    #[must_use]
    pub fn exec_handoff(
        &self,
        ops: Vec<StoreOp>,
        txs: Vec<CheckpointExecTx>,
    ) -> SettlementExecHandoff {
        debug_assert_eq!(self.batch_id, self.planned.batch_id);
        SettlementExecHandoff::new(
            SettlementRouteCtx::new(
                self.planned.batch_id.into_bytes(),
                self.planned.route.shard_id.as_u32(),
                self.planned.route.routing_generation,
                self.planned.route_table_digest.into_bytes(),
            ),
            ops,
            txs,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublicationRequest {
    pub batch_id: BatchId,
    pub ordered_batch: OrderedBatch,
    pub publication_route: PublicationRouteSnapshotV1,
    pub draft: CheckpointDraft,
    pub subject: CommitSubject,
    pub certificate: ShardQuorumCertificate,
    pub tx_package: TxPackage,
    pub exec_input: CheckpointExecInput,
    pub link: CheckpointLink,
    pub nullifiers: Vec<ClaimNullifier>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedBatch {
    pub batch_id: BatchId,
    pub checkpoint_id: CheckpointId,
    pub publication_checkpoint: u64,
    pub publication_route: PublicationRouteSnapshotV1,
    pub pub_in: CheckpointPubIn,
    pub subject_digest: Option<[u8; 32]>,
    pub certificate_digest: Option<[u8; 32]>,
    pub theorem_digest: Option<[u8; 32]>,
    pub da_provider: String,
    pub blob_ref: String,
}

impl PublishedBatch {
    #[must_use]
    pub fn quorum_binding_enabled(&self) -> bool {
        self.subject_digest.is_some()
            || self.certificate_digest.is_some()
            || self.theorem_digest.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationBinding {
    batch_id: BatchId,
    checkpoint_id: CheckpointId,
    route_table_digest: [u8; 32],
    prev_settlement_root: SettlementStateRoot,
    new_settlement_root: SettlementStateRoot,
    claim_root: Option<ClaimSourceRoot>,
    spent_count: usize,
    created_count: usize,
    pub_in_digest: [u8; 32],
    binding_digest: [u8; 32],
}

impl PublicationBinding {
    pub(crate) fn new(
        batch_id: BatchId,
        checkpoint_id: CheckpointId,
        route_table_digest: [u8; 32],
        pub_in: &CheckpointPubIn,
    ) -> Self {
        let pub_in_digest = digest_pub_in(pub_in);
        let binding_digest =
            digest_binding(batch_id, checkpoint_id, route_table_digest, pub_in_digest);
        Self {
            batch_id,
            checkpoint_id,
            route_table_digest,
            prev_settlement_root: pub_in.prev_settlement_root(),
            new_settlement_root: pub_in.new_settlement_root(),
            claim_root: pub_in.claim_root(),
            spent_count: pub_in.spent_delta().len(),
            created_count: pub_in.created_delta().len(),
            pub_in_digest,
            binding_digest,
        }
    }

    #[must_use]
    pub const fn batch_id(&self) -> BatchId {
        self.batch_id
    }

    #[must_use]
    pub const fn checkpoint_id(&self) -> CheckpointId {
        self.checkpoint_id
    }

    #[must_use]
    pub const fn route_table_digest(&self) -> [u8; 32] {
        self.route_table_digest
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
    pub const fn spent_count(&self) -> usize {
        self.spent_count
    }

    #[must_use]
    pub const fn created_count(&self) -> usize {
        self.created_count
    }

    #[must_use]
    pub const fn binding_digest(&self) -> [u8; 32] {
        self.binding_digest
    }

    #[must_use]
    pub const fn pub_in_digest(&self) -> [u8; 32] {
        self.pub_in_digest
    }

    #[must_use]
    pub fn matches_pub_in(&self, pub_in: &CheckpointPubIn) -> bool {
        self.prev_settlement_root == pub_in.prev_settlement_root()
            && self.new_settlement_root == pub_in.new_settlement_root()
            && self.claim_root == pub_in.claim_root()
            && self.spent_count == pub_in.spent_delta().len()
            && self.created_count == pub_in.created_delta().len()
            && self.pub_in_digest == digest_pub_in(pub_in)
    }

    #[must_use]
    pub fn matches_route_table_digest(&self, route_table_digest: [u8; 32]) -> bool {
        self.route_table_digest == route_table_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationRecord {
    pub batch_id: BatchId,
    pub checkpoint_id: Option<CheckpointId>,
    pub state: PublicationState,
    pub da_reference: Option<CheckpointDaReferenceV1>,
    pub publication_evidence: Option<CheckpointPublicationEvidenceV1>,
    pub lifecycle: Option<CheckpointLifecycleV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationReadinessErr {
    CheckpointMismatch,
    PartialBundle,
    LifecycleStateMismatch,
    StatementCoreMismatch,
    EvidenceRootMismatch,
    ArchiveManifestMismatch,
    DaReferenceMismatch,
    PayloadCommitmentMismatch,
    ProviderMismatch,
    HeightMismatch,
}

impl PublicationRecord {
    pub fn validate_readiness_bundle(
        &self,
        checkpoint_id: CheckpointId,
    ) -> Result<(), PublicationReadinessErr> {
        if self.checkpoint_id != Some(checkpoint_id) {
            return Err(PublicationReadinessErr::CheckpointMismatch);
        }

        match (
            self.da_reference.as_ref(),
            self.publication_evidence.as_ref(),
            self.lifecycle.as_ref(),
        ) {
            (None, None, None) => Ok(()),
            (None, None, Some(lifecycle)) => {
                if lifecycle.checkpoint_id() != checkpoint_id {
                    return Err(PublicationReadinessErr::CheckpointMismatch);
                }
                match lifecycle.status() {
                    CheckpointLifecycleStatus::Sealed | CheckpointLifecycleStatus::Linked => Ok(()),
                    CheckpointLifecycleStatus::PublicationReady
                    | CheckpointLifecycleStatus::ChallengeOpen
                    | CheckpointLifecycleStatus::Finalized
                    | CheckpointLifecycleStatus::Disputed
                    | CheckpointLifecycleStatus::Rejected => {
                        Err(PublicationReadinessErr::PartialBundle)
                    }
                }
            }
            (Some(da_reference), Some(evidence), Some(lifecycle)) => {
                if lifecycle.checkpoint_id() != checkpoint_id {
                    return Err(PublicationReadinessErr::CheckpointMismatch);
                }
                match lifecycle.status() {
                    CheckpointLifecycleStatus::PublicationReady
                    | CheckpointLifecycleStatus::ChallengeOpen
                    | CheckpointLifecycleStatus::Finalized
                    | CheckpointLifecycleStatus::Disputed
                    | CheckpointLifecycleStatus::Rejected => {}
                    CheckpointLifecycleStatus::Sealed | CheckpointLifecycleStatus::Linked => {
                        return Err(PublicationReadinessErr::LifecycleStateMismatch);
                    }
                }
                if lifecycle.statement_core_digest() != Some(evidence.statement_core_digest())
                    || lifecycle.statement_core_digest()
                        != Some(da_reference.statement_core_digest())
                {
                    return Err(PublicationReadinessErr::StatementCoreMismatch);
                }
                if lifecycle.publication_evidence_root()
                    != Some(evidence.publication_evidence_root())
                {
                    return Err(PublicationReadinessErr::EvidenceRootMismatch);
                }
                if lifecycle.challenge_window_start_height()
                    != Some(evidence.challenge_window_start_height())
                {
                    return Err(PublicationReadinessErr::HeightMismatch);
                }
                if evidence.da_ref() != da_reference.da_ref() {
                    return Err(PublicationReadinessErr::DaReferenceMismatch);
                }
                if evidence.archive_manifest_root() != da_reference.archive_manifest_root() {
                    return Err(PublicationReadinessErr::ArchiveManifestMismatch);
                }
                if evidence.payload_commitment() != da_reference.payload_commitment() {
                    return Err(PublicationReadinessErr::PayloadCommitmentMismatch);
                }
                if evidence.provider_family() != da_reference.provider_family() {
                    return Err(PublicationReadinessErr::ProviderMismatch);
                }
                if evidence.readiness_height() != da_reference.published_height() {
                    return Err(PublicationReadinessErr::HeightMismatch);
                }
                Ok(())
            }
            _ => Err(PublicationReadinessErr::PartialBundle),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftConfirmation {
    pub intake_id: IntakeId,
    pub batch_id: BatchId,
    pub pub_in: CheckpointPubIn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectRecord {
    pub intake_id: Option<IntakeId>,
    pub class: RejectClass,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationState {
    Received,
    Admitted,
    Ordered,
    Scheduled,
    Built,
    ProofReady,
    HandedOff,
    Posted,
    Seen,
    Accepted,
    Rejected,
    RetryPending,
    Failed,
    Finalized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectClass {
    ParseInvalid,
    AuthInvalid,
    ShapeInvalid,
    ReplayLocal,
    PolicyReject,
    DeferredRetry,
}

pub(crate) fn decode_hex32(raw: &str) -> Result<[u8; 32], String> {
    if raw.len() != 64 {
        return Err("digest must be 64 lowercase hex chars".to_string());
    }

    let mut out = [0u8; 32];
    for (index, chunk) in raw.as_bytes().chunks_exact(2).enumerate() {
        let hi =
            decode_nibble(chunk[0]).ok_or_else(|| "digest must be lowercase hex".to_string())?;
        let lo =
            decode_nibble(chunk[1]).ok_or_else(|| "digest must be lowercase hex".to_string())?;
        out[index] = (hi << 4) | lo;
    }
    Ok(out)
}

fn encode_hex32(bytes: [u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn bind_object_package_digest(
    payload_digest: &CanonicalDigest,
    object_package: &RuntimeObjectPackageV1,
) -> CanonicalDigest {
    let object_bytes = JsonCodec
        .serialize(object_package)
        .expect("runtime object package serialization must succeed");
    let mut hasher = Sha256::new();
    hasher.update(WORK_ITEM_OBJECT_BINDING_LABEL);
    hasher.update(payload_digest.bytes());
    hasher.update((object_bytes.len() as u64).to_be_bytes());
    hasher.update(&object_bytes);
    let bytes: [u8; 32] = hasher.finalize().into();
    CanonicalDigest::new(encode_hex32(bytes), bytes)
}

fn digest_pub_in(pub_in: &CheckpointPubIn) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"phase057_pub_in_binding_v1");
    hasher.update(pub_in.prev_settlement_root().as_bytes());
    hasher.update(pub_in.new_settlement_root().as_bytes());
    match pub_in.claim_root() {
        Some(root) => {
            hasher.update([1]);
            hasher.update(root.as_bytes());
        }
        None => hasher.update([0]),
    }
    hasher.update((pub_in.spent_delta().len() as u64).to_be_bytes());
    for entry in pub_in.spent_delta() {
        hasher.update(entry.terminal_id().as_bytes());
    }
    hasher.update((pub_in.created_delta().len() as u64).to_be_bytes());
    for entry in pub_in.created_delta() {
        hasher.update(entry.terminal_id().as_bytes());
        hasher.update(entry.leaf_hash());
    }
    hasher.finalize().into()
}

fn digest_binding(
    batch_id: BatchId,
    checkpoint_id: CheckpointId,
    route_table_digest: [u8; 32],
    pub_in_digest: [u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"phase057_publication_binding_v1");
    hasher.update(batch_id.into_bytes());
    hasher.update(checkpoint_id.as_bytes());
    hasher.update(route_table_digest);
    hasher.update(pub_in_digest);
    hasher.finalize().into()
}

fn decode_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use z00z_core::ObjectFamily;
    use z00z_storage::{
        checkpoint::{CheckpointDraftId, CheckpointExecOut, CheckpointExecTx, CheckpointInRef},
        settlement::{
            DefinitionId, ObjectDeltaSetV1, SerialId, SettlementActionV1, SettlementPath,
            SettlementStateRoot, StoreItem, TerminalId, TerminalLeaf, VoucherAction,
        },
    };

    use super::{
        BatchId, BatchPlanned, BatchRoute, IntakeId, ObjectWitnessBundleV1, OrderedBatch,
        PlanDigest, RightWitnessRefV1, RightWitnessStateV1, RuntimeObjectPackageV1, ShardId,
        WorkItem, WorkPayload,
    };

    #[test]
    fn exec_handoff_copies_route_ctx() {
        let batch = OrderedBatch {
            batch_id: BatchId::new(CheckpointDraftId::new([1u8; 32])),
            items: vec![dummy_item()],
            created_leaves: Vec::new(),
            planned: BatchPlanned {
                batch_id: BatchId::new(CheckpointDraftId::new([1u8; 32])),
                route: BatchRoute {
                    shard_id: ShardId::new(7),
                    routing_generation: 11,
                },
                route_table_digest: PlanDigest::new([2u8; 32]),
                intake_ids: vec![IntakeId::new("11".repeat(32))],
                op_count: 1,
                plan_digest: PlanDigest::new([3u8; 32]),
            },
        };
        let path = SettlementPath::new(
            DefinitionId::new([4u8; 32]),
            SerialId::new(9),
            TerminalId::new([5u8; 32]),
        );
        let ops = vec![z00z_storage::settlement::StoreOp::Put(Box::new(
            StoreItem::new(path, term_leaf(path)).expect("store item"),
        ))];
        let txs =
            vec![
                CheckpointExecTx::new(
                    vec![CheckpointInRef::new([6u8; 32], SerialId::new(8))],
                    vec![CheckpointExecOut::new(path.definition_id, term_leaf(path))
                        .expect("exec out")],
                    vec![9u8],
                )
                .expect("exec tx"),
            ];

        let handoff = batch.exec_handoff(ops.clone(), txs.clone());

        assert_eq!(handoff.route().batch_id(), [1u8; 32]);
        assert_eq!(handoff.route().shard_id(), 7);
        assert_eq!(handoff.route().routing_generation(), 11);
        assert_eq!(handoff.route().route_table_digest(), [2u8; 32]);
        assert_eq!(handoff.ops(), ops.as_slice());
        assert_eq!(handoff.txs(), txs.as_slice());
    }

    #[test]
    fn test_rebinds_intake_digest() {
        let item = dummy_item();
        let rebound = item.clone().with_object_package(dummy_object_package(7));

        assert_eq!(rebound.route_key(), item.route_key());
        assert_ne!(rebound.digest_hex(), item.digest_hex());
        assert!(rebound.object_package().is_some());
    }

    #[test]
    fn test_packages_change_admission() {
        let left = dummy_item().with_object_package(dummy_object_package(7));
        let right = dummy_item().with_object_package(dummy_object_package(8));

        assert_eq!(left.route_key(), right.route_key());
        assert_ne!(left.digest_hex(), right.digest_hex());
    }

    fn dummy_item() -> WorkItem {
        WorkItem::new(
            WorkPayload::Tx(Box::new(z00z_wallets::tx::TxPackage {
                kind: "TxPackage".to_string(),
                package_type: "regular_tx".to_string(),
                version: 1,
                chain_id: 1,
                chain_type: "devnet".to_string(),
                chain_name: "z00z-devnet".to_string(),
                tx: z00z_wallets::tx::TxWire {
                    tx_type: "regular_tx".to_string(),
                    inputs: Vec::new(),
                    outputs: Vec::new(),
                    fee: 0,
                    nonce: 0,
                    context: z00z_wallets::tx::TxContextWire::default(),
                    proof: z00z_wallets::tx::TxProofWire::default(),
                    auth: z00z_wallets::tx::TxAuthWire::default(),
                },
                tx_digest_hex: "11".repeat(32),
                status: "received".to_string(),
            })),
            super::CanonicalDigest::new("11".repeat(32), [0x11; 32]),
        )
    }

    fn term_leaf(path: SettlementPath) -> TerminalLeaf {
        let mut leaf = TerminalLeaf::dummy_for_scan(path.serial_id.get());
        leaf.asset_id = path.terminal_id().into_bytes();
        leaf.serial_id = path.serial_id.get();
        leaf
    }

    fn dummy_object_package(seed: u8) -> RuntimeObjectPackageV1 {
        let prior_root = SettlementStateRoot::settlement_v1([seed; 32]);
        let expected_new_root = SettlementStateRoot::settlement_v1([seed.wrapping_add(1); 32]);
        RuntimeObjectPackageV1 {
            primary_family: ObjectFamily::Voucher,
            selected_action: SettlementActionV1::Voucher(VoucherAction::RedeemFull),
            selected_action_id: [seed; 32],
            policy_descriptor_hash: [seed.wrapping_add(1); 32],
            action_pool_id: [seed.wrapping_add(2); 32],
            required_rights: vec![RightWitnessRefV1 {
                right_policy: format!("kyc_{seed}"),
                witness_state: RightWitnessStateV1::Present,
            }],
            object_witnesses: ObjectWitnessBundleV1 {
                signatures: BTreeSet::new(),
                attestation_labels: BTreeSet::new(),
                has_acceptance_proof: false,
                has_replay_nonce: true,
                has_prior_root_binding: true,
                has_disclosure_commitment: false,
            },
            delta_set: ObjectDeltaSetV1::new(
                SettlementActionV1::Voucher(VoucherAction::RedeemFull),
                [seed.wrapping_add(1); 32],
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                None,
                prior_root,
                expected_new_root,
            ),
            fee_support_ref: None,
            prior_root,
            expected_new_root,
        }
    }
}
