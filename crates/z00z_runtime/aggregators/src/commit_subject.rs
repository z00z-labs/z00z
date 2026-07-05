#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use z00z_crypto::{
    domains::CommitSubjectDomain, expert::traits::DomainSeparation, DomainHasher256,
};
use z00z_storage::settlement::SettlementStateRoot;

use crate::{
    recovery::ShardRecoveryRecord,
    types::{
        decode_hex32, BatchId, BatchRoute, OrderedBatch, PublicationBinding, RejectClass,
        RejectRecord, ShardId,
    },
};

const JOURNAL_CANDIDATE_TAG: &[u8] = b"z00z.journal_candidate";
const COMMIT_SUBJECT_TAG: &[u8] = b"z00z.commit_subject";
pub(crate) const COMMIT_SUBJECT_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalCandidate {
    pub batch_id: BatchId,
    pub route: BatchRoute,
    pub state_root: SettlementStateRoot,
    pub journal_lineage: [u8; 32],
    pub version: u64,
    pub root_generation: u8,
    pub proof_version: u16,
    pub bucket_policy_generation: u32,
    pub bucket_policy_id: [u8; 32],
}

impl JournalCandidate {
    pub fn from_record(record: &ShardRecoveryRecord) -> Result<Self, RejectRecord> {
        let recovery = &record.recovery;
        if recovery.version != 0 {
            let Some(route) = recovery.route else {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "candidate recovery route is missing",
                ));
            };
            if route.shard_id() != record.placement.route.shard_id.as_u32()
                || route.routing_generation() != record.placement.route.routing_generation
            {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "candidate recovery route drifted from shard placement",
                ));
            }
            if route.batch_id() != record.batch_id.into_bytes() {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "candidate recovery batch id drifted from the recovery record",
                ));
            }
        }

        Ok(Self {
            batch_id: record.batch_id,
            route: record.placement.route,
            state_root: recovery.state_root,
            journal_lineage: recovery.journal_lineage,
            version: recovery.version,
            root_generation: recovery.root_generation,
            proof_version: recovery.proof_version,
            bucket_policy_generation: recovery.bucket_policy_generation,
            bucket_policy_id: recovery.bucket_policy_id,
        })
    }

    #[must_use]
    pub fn conflicts_with(&self, other: &Self) -> bool {
        self.route == other.route
            && (self.batch_id != other.batch_id
                || self.state_root != other.state_root
                || self.journal_lineage != other.journal_lineage
                || self.version != other.version
                || self.root_generation != other.root_generation
                || self.proof_version != other.proof_version
                || self.bucket_policy_generation != other.bucket_policy_generation
                || self.bucket_policy_id != other.bucket_policy_id)
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(128);
        out.extend_from_slice(JOURNAL_CANDIDATE_TAG);
        push_u8(&mut out, COMMIT_SUBJECT_VERSION);
        push_batch_route(&mut out, self.route);
        push_bytes32(&mut out, self.batch_id.into_bytes());
        push_root(&mut out, self.state_root);
        push_bytes32(&mut out, self.journal_lineage);
        push_u64(&mut out, self.version);
        push_u8(&mut out, self.root_generation);
        push_u16(&mut out, self.proof_version);
        push_u32(&mut out, self.bucket_policy_generation);
        push_bytes32(&mut out, self.bucket_policy_id);
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitSubject {
    pub version: u8,
    pub shard_id: ShardId,
    pub routing_generation: u64,
    pub route_table_digest: [u8; 32],
    pub membership_digest: [u8; 32],
    pub term: u64,
    pub batch_id: BatchId,
    pub plan_digest: [u8; 32],
    pub ordered_batch_digest: [u8; 32],
    pub payload_digest: [u8; 32],
    pub previous_state_root: SettlementStateRoot,
    pub new_state_root: SettlementStateRoot,
    pub journal_lineage: [u8; 32],
    pub recovery_version: u64,
    pub root_generation: u8,
    pub proof_version: u16,
    pub bucket_policy_generation: u32,
    pub bucket_policy_id: [u8; 32],
    pub publication_binding_digest: [u8; 32],
    pub theorem_or_settlement_digest: [u8; 32],
    pub da_availability_digest: Option<[u8; 32]>,
}

impl CommitSubject {
    pub fn from_runtime(
        term: u64,
        membership_digest: [u8; 32],
        batch: &OrderedBatch,
        candidate: &JournalCandidate,
        publication_binding: &PublicationBinding,
        theorem_or_settlement_digest: [u8; 32],
        da_availability_digest: Option<[u8; 32]>,
    ) -> Result<Self, RejectRecord> {
        if batch.items.is_empty() {
            return Err(reject(
                RejectClass::ShapeInvalid,
                "commit subject cannot bind an empty ordered batch",
            ));
        }
        if batch.batch_id != batch.planned.batch_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject batch id drifted from batch plan metadata",
            ));
        }
        if batch.planned.op_count != batch.items.len() {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject op_count drifted from the ordered batch items",
            ));
        }
        if batch.planned.intake_ids.len() != batch.items.len() {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject intake ids drifted from the ordered batch items",
            ));
        }
        if candidate.batch_id != batch.batch_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject candidate batch id drifted from the ordered batch",
            ));
        }
        if candidate.route != batch.planned.route {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject route drifted between planning and recovery",
            ));
        }
        if publication_binding.batch_id() != batch.batch_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject publication binding batch id drifted from the ordered batch",
            ));
        }
        if !publication_binding
            .matches_route_table_digest(batch.planned.route_table_digest.into_bytes())
        {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject publication route-table digest drifted from the ordered batch",
            ));
        }
        if publication_binding.new_settlement_root() != candidate.state_root {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject new settlement root drifted from the recovery candidate",
            ));
        }

        let payload_digest = payload_digest(batch)?;
        let ordered_batch_digest = ordered_batch_digest(batch, payload_digest)?;

        Ok(Self {
            version: COMMIT_SUBJECT_VERSION,
            shard_id: batch.planned.route.shard_id,
            routing_generation: batch.planned.route.routing_generation,
            route_table_digest: batch.planned.route_table_digest.into_bytes(),
            membership_digest,
            term,
            batch_id: batch.batch_id,
            plan_digest: batch.planned.plan_digest.into_bytes(),
            ordered_batch_digest,
            payload_digest,
            previous_state_root: publication_binding.prev_settlement_root(),
            new_state_root: publication_binding.new_settlement_root(),
            journal_lineage: candidate.journal_lineage,
            recovery_version: candidate.version,
            root_generation: candidate.root_generation,
            proof_version: candidate.proof_version,
            bucket_policy_generation: candidate.bucket_policy_generation,
            bucket_policy_id: candidate.bucket_policy_id,
            publication_binding_digest: publication_binding.binding_digest(),
            theorem_or_settlement_digest,
            da_availability_digest,
        })
    }

    #[must_use]
    pub const fn route(&self) -> BatchRoute {
        BatchRoute {
            shard_id: self.shard_id,
            routing_generation: self.routing_generation,
        }
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(384);
        out.extend_from_slice(COMMIT_SUBJECT_TAG);
        push_u8(&mut out, self.version);
        push_shard_id(&mut out, self.shard_id);
        push_u64(&mut out, self.routing_generation);
        push_bytes32(&mut out, self.route_table_digest);
        push_bytes32(&mut out, self.membership_digest);
        push_u64(&mut out, self.term);
        push_bytes32(&mut out, self.batch_id.into_bytes());
        push_bytes32(&mut out, self.plan_digest);
        push_bytes32(&mut out, self.ordered_batch_digest);
        push_bytes32(&mut out, self.payload_digest);
        push_root(&mut out, self.previous_state_root);
        push_root(&mut out, self.new_state_root);
        push_bytes32(&mut out, self.journal_lineage);
        push_u64(&mut out, self.recovery_version);
        push_u8(&mut out, self.root_generation);
        push_u16(&mut out, self.proof_version);
        push_u32(&mut out, self.bucket_policy_generation);
        push_bytes32(&mut out, self.bucket_policy_id);
        push_bytes32(&mut out, self.publication_binding_digest);
        push_bytes32(&mut out, self.theorem_or_settlement_digest);
        push_opt_bytes32(&mut out, self.da_availability_digest);
        out
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        digest_bytes::<CommitSubjectDomain>("digest", &self.encode())
    }

    pub fn verify_binding(
        &self,
        batch: &OrderedBatch,
        publication_binding: &PublicationBinding,
        theorem_or_settlement_digest: [u8; 32],
    ) -> Result<(), RejectRecord> {
        if self.version != COMMIT_SUBJECT_VERSION {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject version drifted from the live contract",
            ));
        }
        if batch.items.is_empty() {
            return Err(reject(
                RejectClass::ShapeInvalid,
                "commit subject cannot bind an empty ordered batch",
            ));
        }
        if batch.batch_id != batch.planned.batch_id || self.batch_id != batch.batch_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject batch id drifted from the ordered batch",
            ));
        }
        if self.route() != batch.planned.route {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject route drifted from the ordered batch",
            ));
        }
        if self.route_table_digest != batch.planned.route_table_digest.into_bytes() {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject route-table digest drifted from the ordered batch",
            ));
        }
        if self.plan_digest != batch.planned.plan_digest.into_bytes() {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject plan digest drifted from the ordered batch",
            ));
        }
        if batch.planned.op_count != batch.items.len() {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject op_count drifted from the ordered batch items",
            ));
        }
        if batch.planned.intake_ids.len() != batch.items.len() {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject intake ids drifted from the ordered batch items",
            ));
        }
        if publication_binding.batch_id() != batch.batch_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject publication binding batch id drifted from the ordered batch",
            ));
        }
        if !publication_binding
            .matches_route_table_digest(batch.planned.route_table_digest.into_bytes())
        {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject publication route-table digest drifted from the ordered batch",
            ));
        }
        if self.previous_state_root != publication_binding.prev_settlement_root() {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject previous settlement root drifted from publication binding",
            ));
        }
        if self.new_state_root != publication_binding.new_settlement_root() {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject new settlement root drifted from publication binding",
            ));
        }
        if self.publication_binding_digest != publication_binding.binding_digest() {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject publication binding digest drifted from publication binding",
            ));
        }
        if self.theorem_or_settlement_digest != theorem_or_settlement_digest {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject theorem digest drifted from the theorem bundle",
            ));
        }

        let payload_digest = payload_digest(batch)?;
        if self.payload_digest != payload_digest {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject payload digest drifted from the ordered batch",
            ));
        }
        let ordered_batch_digest = ordered_batch_digest(batch, payload_digest)?;
        if self.ordered_batch_digest != ordered_batch_digest {
            return Err(reject(
                RejectClass::PolicyReject,
                "commit subject ordered-batch digest drifted from the ordered batch",
            ));
        }

        Ok(())
    }
}

pub(crate) fn digest_bytes<D: DomainSeparation>(label: &'static str, bytes: &[u8]) -> [u8; 32] {
    let digest = DomainHasher256::<D>::new_with_label(label)
        .chain(bytes)
        .finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(digest.as_ref());
    out
}

pub(crate) fn push_u8(out: &mut Vec<u8>, value: u8) {
    out.push(value);
}

pub(crate) fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_be_bytes());
}

pub(crate) fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_be_bytes());
}

pub(crate) fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}

pub(crate) fn push_usize(out: &mut Vec<u8>, value: usize) {
    push_u64(out, value as u64);
}

pub(crate) fn push_len_prefixed(out: &mut Vec<u8>, bytes: &[u8]) {
    push_usize(out, bytes.len());
    out.extend_from_slice(bytes);
}

pub(crate) fn push_bytes32(out: &mut Vec<u8>, bytes: [u8; 32]) {
    out.extend_from_slice(&bytes);
}

pub(crate) fn push_root(out: &mut Vec<u8>, root: SettlementStateRoot) {
    out.extend_from_slice(root.as_bytes());
}

pub(crate) fn push_shard_id(out: &mut Vec<u8>, shard_id: ShardId) {
    push_u16(out, shard_id.as_u16());
}

pub(crate) fn push_batch_route(out: &mut Vec<u8>, route: BatchRoute) {
    push_shard_id(out, route.shard_id);
    push_u64(out, route.routing_generation);
}

pub(crate) fn push_opt_bytes32(out: &mut Vec<u8>, bytes: Option<[u8; 32]>) {
    match bytes {
        Some(bytes) => {
            push_u8(out, 1);
            push_bytes32(out, bytes);
        }
        None => push_u8(out, 0),
    }
}

fn payload_digest(batch: &OrderedBatch) -> Result<[u8; 32], RejectRecord> {
    let mut payload = Vec::new();
    payload.extend_from_slice(b"z00z.ordered_batch_payload");
    push_u8(&mut payload, COMMIT_SUBJECT_VERSION);
    push_usize(&mut payload, batch.items.len());
    for item in &batch.items {
        push_u8(&mut payload, item.kind_tag());
        push_bytes32(&mut payload, item.route_key());
        push_bytes32(&mut payload, item.admission_digest_bytes());
    }
    Ok(digest_bytes::<CommitSubjectDomain>(
        "payload_digest",
        &payload,
    ))
}

fn ordered_batch_digest(
    batch: &OrderedBatch,
    payload_digest: [u8; 32],
) -> Result<[u8; 32], RejectRecord> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"z00z.ordered_batch");
    push_u8(&mut bytes, COMMIT_SUBJECT_VERSION);
    push_bytes32(&mut bytes, batch.batch_id.into_bytes());
    push_batch_route(&mut bytes, batch.planned.route);
    push_bytes32(&mut bytes, batch.planned.route_table_digest.into_bytes());
    push_bytes32(&mut bytes, batch.planned.plan_digest.into_bytes());
    push_usize(&mut bytes, batch.planned.op_count);
    push_usize(&mut bytes, batch.planned.intake_ids.len());
    for intake_id in &batch.planned.intake_ids {
        let intake_bytes = decode_hex32(intake_id.digest_hex()).map_err(|detail| {
            reject(
                RejectClass::PolicyReject,
                &format!("commit subject intake id is not canonical hex: {detail}"),
            )
        })?;
        push_bytes32(&mut bytes, intake_bytes);
    }
    push_bytes32(&mut bytes, payload_digest);
    push_usize(&mut bytes, batch.created_leaves.len());
    for leaf in &batch.created_leaves {
        let encoded = leaf.encode().map_err(|err| {
            reject(
                RejectClass::PolicyReject,
                &format!("commit subject leaf encoding failed: {err}"),
            )
        })?;
        push_len_prefixed(&mut bytes, &encoded);
    }
    Ok(digest_bytes::<CommitSubjectDomain>(
        "ordered_batch_digest",
        &bytes,
    ))
}

fn reject(class: RejectClass, detail: &str) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.to_string(),
    }
}
