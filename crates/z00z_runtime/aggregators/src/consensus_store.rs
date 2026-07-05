#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use z00z_storage::{
    checkpoint::CheckpointId,
    settlement::{ClaimSourceRoot, PublicationRouteSnapshotV1, SettlementStateRoot},
};
use z00z_utils::io::{create_dir_all, load_json, save_json};

use crate::{BatchId, BatchRoute};
use crate::{
    CommitSubject, JournalCandidate, PublicationBinding, PublicationRecord, PublishedBatch,
    RejectClass, RejectRecord, ShardQuorumCertificate, ShardRecoveryRecord, ShardVote,
};

const CONSENSUS_STORE_ROUTE_TAG: &[u8] = b"z00z.consensus_store.route.v1";

pub const CONSENSUS_STORE_SCHEMA_VERSION: u32 = 1;
pub const CONSENSUS_STORE_BACKEND: &str = "json_directory_v1";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConsensusStoreError {
    #[error("{0}")]
    Message(String),
}

impl ConsensusStoreError {
    fn new(detail: impl Into<String>) -> Self {
        Self::Message(detail.into())
    }

    #[must_use]
    pub fn to_reject(&self) -> RejectRecord {
        RejectRecord {
            intake_id: None,
            class: RejectClass::PolicyReject,
            detail: self.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusStore {
    root: PathBuf,
}

impl ConsensusStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ConsensusStoreError> {
        let root = root.as_ref().to_path_buf();
        create_dir_all(root.join("batches")).map_err(|err| {
            ConsensusStoreError::new(format!("failed to create batch directory: {err}"))
        })?;
        create_dir_all(root.join("routes")).map_err(|err| {
            ConsensusStoreError::new(format!("failed to create route directory: {err}"))
        })?;
        Ok(Self { root })
    }

    #[must_use]
    pub fn batch_path(&self, batch_id: BatchId) -> PathBuf {
        self.root
            .join("batches")
            .join(format!("{}.json", encode_hex32(batch_id.into_bytes())))
    }

    #[must_use]
    pub fn route_path(&self, route: BatchRoute) -> PathBuf {
        self.root
            .join("routes")
            .join(format!("{}.json", encode_hex32(route_key(route))))
    }

    pub fn persist_commit(
        &self,
        recovery_record: &ShardRecoveryRecord,
        header: &CommitSubject,
        votes: &[ShardVote],
        certificate: &ShardQuorumCertificate,
    ) -> Result<ConsensusStoreRecord, ConsensusStoreError> {
        let record = ConsensusStoreRecord::new(
            recovery_record.clone(),
            header.clone(),
            votes.to_vec(),
            certificate.clone(),
        )?;
        self.save_record(&record)?;
        self.save_cursor(&record.route_cursor())?;
        Ok(record)
    }

    pub fn persist_publication(
        &self,
        batch_id: BatchId,
        publication_record: PublicationRecord,
        binding: &PublicationBinding,
        published: &PublishedBatch,
    ) -> Result<ConsensusStoreRecord, ConsensusStoreError> {
        let mut record = self.load_batch(batch_id)?;
        record.publication = Some(ConsensusStorePublication::new(
            publication_record,
            binding,
            published,
        ));
        record.verify().map_err(ConsensusStoreError::from)?;
        self.save_record(&record)?;
        Ok(record)
    }

    pub fn persist_validator_decision(
        &self,
        batch_id: BatchId,
        decision: ConsensusValidatorDecision,
    ) -> Result<ConsensusStoreRecord, ConsensusStoreError> {
        let mut record = self.load_batch(batch_id)?;
        record.validator_decision = Some(decision);
        record.verify().map_err(ConsensusStoreError::from)?;
        self.save_record(&record)?;
        Ok(record)
    }

    pub fn load_batch(
        &self,
        batch_id: BatchId,
    ) -> Result<ConsensusStoreRecord, ConsensusStoreError> {
        let path = self.batch_path(batch_id);
        let record: ConsensusStoreRecord = load_json(&path).map_err(|err| {
            ConsensusStoreError::new(format!(
                "failed to load durable consensus batch record {}: {err}",
                path.display()
            ))
        })?;
        record.verify().map_err(ConsensusStoreError::from)?;
        Ok(record)
    }

    pub fn load_route(
        &self,
        route: BatchRoute,
    ) -> Result<ConsensusStoreRecord, ConsensusStoreError> {
        let path = self.route_path(route);
        let cursor: ConsensusStoreRouteCursor = load_json(&path).map_err(|err| {
            ConsensusStoreError::new(format!(
                "failed to load durable consensus route cursor {}: {err}",
                path.display()
            ))
        })?;
        cursor.verify(route).map_err(ConsensusStoreError::from)?;
        let record = self.load_batch(cursor.batch_id)?;
        if record.route != route {
            return Err(ConsensusStoreError::new(
                "stored consensus route cursor drifted from the persisted batch record",
            ));
        }
        Ok(record)
    }

    fn save_record(&self, record: &ConsensusStoreRecord) -> Result<(), ConsensusStoreError> {
        save_json(self.batch_path(record.batch_id), record).map_err(|err| {
            ConsensusStoreError::new(format!(
                "failed to save durable consensus batch record: {err}"
            ))
        })
    }

    fn save_cursor(&self, cursor: &ConsensusStoreRouteCursor) -> Result<(), ConsensusStoreError> {
        save_json(self.route_path(cursor.route), cursor).map_err(|err| {
            ConsensusStoreError::new(format!(
                "failed to save durable consensus route cursor: {err}"
            ))
        })
    }
}

impl From<RejectRecord> for ConsensusStoreError {
    fn from(value: RejectRecord) -> Self {
        Self::new(value.detail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsensusStoreRouteCursor {
    pub schema_version: u32,
    pub backend: String,
    pub route: BatchRoute,
    pub route_key_hex: String,
    pub batch_id: BatchId,
}

impl ConsensusStoreRouteCursor {
    fn new(route: BatchRoute, batch_id: BatchId) -> Self {
        Self {
            schema_version: CONSENSUS_STORE_SCHEMA_VERSION,
            backend: CONSENSUS_STORE_BACKEND.to_string(),
            route,
            route_key_hex: encode_hex32(route_key(route)),
            batch_id,
        }
    }

    fn verify(&self, expected_route: BatchRoute) -> Result<(), RejectRecord> {
        if self.schema_version != CONSENSUS_STORE_SCHEMA_VERSION {
            return Err(reject(
                "durable consensus route cursor schema version drifted from the live contract",
            ));
        }
        if self.backend != CONSENSUS_STORE_BACKEND {
            return Err(reject(
                "durable consensus route cursor backend drifted from the live contract",
            ));
        }
        if self.route != expected_route {
            return Err(reject(
                "durable consensus route cursor drifted from the requested route",
            ));
        }
        if self.route_key_hex != encode_hex32(route_key(self.route)) {
            return Err(reject(
                "durable consensus route cursor key drifted from the canonical route path",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsensusStoreRecord {
    pub schema_version: u32,
    pub backend: String,
    pub route: BatchRoute,
    pub route_key_hex: String,
    pub batch_id: BatchId,
    pub recovery_record: ShardRecoveryRecord,
    pub header: CommitSubject,
    pub votes: Vec<ShardVote>,
    pub certificate: ShardQuorumCertificate,
    pub publication: Option<ConsensusStorePublication>,
    pub validator_decision: Option<ConsensusValidatorDecision>,
}

impl ConsensusStoreRecord {
    pub fn new(
        recovery_record: ShardRecoveryRecord,
        header: CommitSubject,
        votes: Vec<ShardVote>,
        certificate: ShardQuorumCertificate,
    ) -> Result<Self, RejectRecord> {
        let route = recovery_record.placement.route;
        let batch_id = recovery_record.batch_id;
        let record = Self {
            schema_version: CONSENSUS_STORE_SCHEMA_VERSION,
            backend: CONSENSUS_STORE_BACKEND.to_string(),
            route,
            route_key_hex: encode_hex32(route_key(route)),
            batch_id,
            recovery_record,
            header,
            votes,
            certificate,
            publication: None,
            validator_decision: None,
        };
        record.verify()?;
        Ok(record)
    }

    #[must_use]
    pub fn route_cursor(&self) -> ConsensusStoreRouteCursor {
        ConsensusStoreRouteCursor::new(self.route, self.batch_id)
    }

    pub fn verify(&self) -> Result<(), RejectRecord> {
        if self.schema_version != CONSENSUS_STORE_SCHEMA_VERSION {
            return Err(reject(
                "durable consensus record schema version drifted from the live contract",
            ));
        }
        if self.backend != CONSENSUS_STORE_BACKEND {
            return Err(reject(
                "durable consensus backend drifted from the live contract",
            ));
        }
        if self.route_key_hex != encode_hex32(route_key(self.route)) {
            return Err(reject(
                "durable consensus route key drifted from the canonical route path",
            ));
        }
        if self.batch_id != self.recovery_record.batch_id {
            return Err(reject(
                "durable consensus batch id drifted from the persisted recovery record",
            ));
        }
        if self.route != self.recovery_record.placement.route {
            return Err(reject(
                "durable consensus route drifted from the persisted recovery record",
            ));
        }
        if self.batch_id != self.header.batch_id {
            return Err(reject(
                "stored consensus header batch id drifted from the persisted recovery record",
            ));
        }
        if self.route != self.header.route() {
            return Err(reject(
                "stored consensus header route drifted from the persisted recovery record",
            ));
        }

        let candidate = JournalCandidate::from_record(&self.recovery_record)?;
        if candidate.state_root != self.header.new_state_root
            || candidate.journal_lineage != self.header.journal_lineage
            || candidate.version != self.header.recovery_version
            || candidate.root_generation != self.header.root_generation
            || candidate.proof_version != self.header.proof_version
            || candidate.bucket_policy_generation != self.header.bucket_policy_generation
            || candidate.bucket_policy_id != self.header.bucket_policy_id
        {
            return Err(reject(
                "stored consensus header recovery metadata drifted from the persisted recovery record",
            ));
        }

        if self.votes.is_empty() {
            return Err(reject(
                "stored consensus vote material is missing for the persisted quorum certificate",
            ));
        }
        self.certificate.verify_subject(&self.header)?;
        if vote_digest_set(&self.votes) != vote_digest_set(&self.certificate.votes) {
            return Err(reject(
                "stored consensus vote material drifted from the persisted quorum certificate",
            ));
        }

        if let Some(publication) = &self.publication {
            publication.verify(self.batch_id, &self.header, &self.certificate)?;
        }
        if let Some(decision) = &self.validator_decision {
            decision.verify(
                self.batch_id,
                &self.header,
                &self.certificate,
                self.publication.as_ref(),
            )?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsensusStorePublication {
    pub publication_record: PublicationRecord,
    pub binding: ConsensusStoreBinding,
    pub published: ConsensusStorePublishedBatch,
}

impl ConsensusStorePublication {
    #[must_use]
    pub fn new(
        publication_record: PublicationRecord,
        binding: &PublicationBinding,
        published: &PublishedBatch,
    ) -> Self {
        Self {
            publication_record,
            binding: ConsensusStoreBinding::from(binding),
            published: ConsensusStorePublishedBatch::from(published),
        }
    }

    fn verify(
        &self,
        batch_id: BatchId,
        header: &CommitSubject,
        certificate: &ShardQuorumCertificate,
    ) -> Result<(), RejectRecord> {
        if self.publication_record.batch_id != batch_id
            || self.binding.batch_id != batch_id
            || self.published.batch_id != batch_id
        {
            return Err(reject(
                "durable consensus publication batch id drifted from the persisted quorum evidence",
            ));
        }
        if self.publication_record.checkpoint_id != Some(self.published.checkpoint_id) {
            return Err(reject(
                "durable consensus publication checkpoint drifted from the published anchor",
            ));
        }
        if self.binding.route_table_digest != header.route_table_digest
            || self.binding.prev_settlement_root != header.previous_state_root
            || self.binding.new_settlement_root != header.new_state_root
        {
            return Err(reject(
                "durable consensus publication binding drifted from the persisted subject header",
            ));
        }
        if self.binding.binding_digest != header.publication_binding_digest {
            return Err(reject(
                "durable consensus publication binding digest drifted from the persisted subject header",
            ));
        }
        if self.published.subject_digest != Some(header.digest()) {
            return Err(reject(
                "durable consensus publication anchor is missing the persisted subject digest",
            ));
        }
        if self.published.certificate_digest != Some(certificate.digest()) {
            return Err(reject(
                "durable consensus publication anchor is missing the persisted certificate digest",
            ));
        }
        if self.published.theorem_digest != Some(header.theorem_or_settlement_digest) {
            return Err(reject(
                "durable consensus publication anchor theorem digest drifted from the persisted subject header",
            ));
        }
        if self.published.publication_route.routing_generation != header.routing_generation
            || self.published.publication_route.route_table_digest != header.route_table_digest
            || !self
                .published
                .publication_route
                .shard_ids
                .contains(&header.shard_id.as_u32())
        {
            return Err(reject(
                "durable consensus publication route drifted from the persisted subject header",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsensusStoreBinding {
    pub batch_id: BatchId,
    pub checkpoint_id: CheckpointId,
    pub route_table_digest: [u8; 32],
    pub prev_settlement_root: SettlementStateRoot,
    pub new_settlement_root: SettlementStateRoot,
    pub claim_root: Option<ClaimSourceRoot>,
    pub spent_count: usize,
    pub created_count: usize,
    pub pub_in_digest: [u8; 32],
    pub binding_digest: [u8; 32],
}

impl From<&PublicationBinding> for ConsensusStoreBinding {
    fn from(value: &PublicationBinding) -> Self {
        Self {
            batch_id: value.batch_id(),
            checkpoint_id: value.checkpoint_id(),
            route_table_digest: value.route_table_digest(),
            prev_settlement_root: value.prev_settlement_root(),
            new_settlement_root: value.new_settlement_root(),
            claim_root: value.claim_root(),
            spent_count: value.spent_count(),
            created_count: value.created_count(),
            pub_in_digest: value.pub_in_digest(),
            binding_digest: value.binding_digest(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsensusStorePublishedBatch {
    pub batch_id: BatchId,
    pub checkpoint_id: CheckpointId,
    pub publication_checkpoint: u64,
    pub publication_route: PublicationRouteSnapshotV1,
    pub subject_digest: Option<[u8; 32]>,
    pub certificate_digest: Option<[u8; 32]>,
    pub theorem_digest: Option<[u8; 32]>,
    pub da_provider: String,
    pub blob_ref: String,
}

impl From<&PublishedBatch> for ConsensusStorePublishedBatch {
    fn from(value: &PublishedBatch) -> Self {
        Self {
            batch_id: value.batch_id,
            checkpoint_id: value.checkpoint_id,
            publication_checkpoint: value.publication_checkpoint,
            publication_route: value.publication_route.clone(),
            subject_digest: value.subject_digest,
            certificate_digest: value.certificate_digest,
            theorem_digest: value.theorem_digest,
            da_provider: value.da_provider.clone(),
            blob_ref: value.blob_ref.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsensusValidatorDecision {
    pub verdict_kind: String,
    pub reject_class: Option<String>,
    pub checkpoint_id: Option<CheckpointId>,
    pub publication_binding_digest: Option<[u8; 32]>,
    pub theorem_digest: [u8; 32],
    pub batch_id: BatchId,
    pub subject_digest: [u8; 32],
    pub certificate_digest: [u8; 32],
}

impl ConsensusValidatorDecision {
    #[must_use]
    pub fn new(
        verdict_kind: impl Into<String>,
        reject_class: Option<String>,
        checkpoint_id: Option<CheckpointId>,
        publication_binding_digest: Option<[u8; 32]>,
        theorem_digest: [u8; 32],
        batch_id: BatchId,
        subject_digest: [u8; 32],
        certificate_digest: [u8; 32],
    ) -> Self {
        Self {
            verdict_kind: verdict_kind.into(),
            reject_class,
            checkpoint_id,
            publication_binding_digest,
            theorem_digest,
            batch_id,
            subject_digest,
            certificate_digest,
        }
    }

    fn verify(
        &self,
        batch_id: BatchId,
        header: &CommitSubject,
        certificate: &ShardQuorumCertificate,
        publication: Option<&ConsensusStorePublication>,
    ) -> Result<(), RejectRecord> {
        if self.batch_id != batch_id {
            return Err(reject(
                "durable validator decision batch id drifted from the persisted quorum evidence",
            ));
        }
        if self.subject_digest != header.digest() {
            return Err(reject(
                "durable validator decision subject digest drifted from the persisted subject header",
            ));
        }
        if self.certificate_digest != certificate.digest() {
            return Err(reject(
                "durable validator decision certificate digest drifted from the persisted quorum certificate",
            ));
        }
        if self.theorem_digest != header.theorem_or_settlement_digest {
            return Err(reject(
                "durable validator decision theorem digest drifted from the persisted subject header",
            ));
        }
        let Some(publication) = publication else {
            return Err(reject(
                "durable validator decision exists without a matching persisted publication anchor",
            ));
        };
        if self.checkpoint_id != Some(publication.published.checkpoint_id) {
            return Err(reject(
                "durable validator decision checkpoint drifted from the persisted publication anchor",
            ));
        }
        if self.publication_binding_digest != Some(publication.binding.binding_digest) {
            return Err(reject(
                "durable validator decision publication binding drifted from the persisted publication anchor",
            ));
        }
        Ok(())
    }
}

fn route_key(route: BatchRoute) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(CONSENSUS_STORE_ROUTE_TAG);
    hasher.update(route.shard_id.as_u16().to_be_bytes());
    hasher.update(route.routing_generation.to_be_bytes());
    hasher.finalize().into()
}

fn vote_digest_set(votes: &[ShardVote]) -> Vec<[u8; 32]> {
    let mut digests = votes.iter().map(ShardVote::digest).collect::<Vec<_>>();
    digests.sort();
    digests
}

fn encode_hex32(bytes: [u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for byte in bytes {
        out.push(nibble(byte >> 4));
        out.push(nibble(byte & 0x0f));
    }
    out
}

const fn nibble(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + (value - 10)) as char,
    }
}

fn reject(detail: &str) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class: RejectClass::PolicyReject,
        detail: detail.to_string(),
    }
}
