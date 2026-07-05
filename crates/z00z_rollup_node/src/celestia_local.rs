#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};

use z00z_aggregators::{bind_publication_contract, BatchId, PublicationRequest, PublishedBatch};
use z00z_validators::{ResolvedBatch, SettlementTheoremBundle};

use crate::da::{
    artifact_for_request, hash_parts, request_payload_digest, verify_request_quorum_binding,
    DaAdapter, DaError,
};

const NAMESPACE_LABEL: &[u8] = b"z00z.rollup.celestia-local.namespace.v1";
const COMMITMENT_LABEL: &[u8] = b"z00z.rollup.celestia-local.commitment.v1";

/// Deterministic local metadata for one Celestia-compatible blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CelestiaLocalRecord {
    pub batch_id: BatchId,
    pub source_label: String,
    pub namespace_hex: String,
    pub blob_commitment: [u8; 32],
    pub payload_digest: [u8; 32],
    pub publication_digest: [u8; 32],
    pub subject_digest: [u8; 32],
    pub certificate_digest: [u8; 32],
    pub theorem_digest: [u8; 32],
    pub blob_height: u64,
    pub anchor_height: Option<u64>,
    pub challenge_window: u64,
    pub unanchored_limit: u64,
    pub payload_available: bool,
    pub replay_id: String,
}

#[derive(Debug, Clone)]
struct CelestiaStoredBatch {
    request: PublicationRequest,
    artifact: z00z_storage::checkpoint::CheckpointArtifact,
    published: PublishedBatch,
    record: CelestiaLocalRecord,
}

/// Local Celestia-compatible adapter that keeps the live theorem and QC contract.
#[derive(Debug, Clone)]
pub struct CelestiaLocalAdapter {
    source_label: String,
    replay_ids: HashSet<String>,
    batches: HashMap<BatchId, CelestiaStoredBatch>,
    current_height: u64,
    challenge_window: u64,
    unanchored_limit: u64,
}

impl Default for CelestiaLocalAdapter {
    fn default() -> Self {
        Self::new("local")
    }
}

impl CelestiaLocalAdapter {
    /// Build one deterministic local adapter.
    #[must_use]
    pub fn new(source_label: impl Into<String>) -> Self {
        Self {
            source_label: source_label.into(),
            replay_ids: HashSet::new(),
            batches: HashMap::new(),
            current_height: 0,
            challenge_window: 4,
            unanchored_limit: 8,
        }
    }

    /// Return the recorded metadata for one batch.
    #[must_use]
    pub fn record(&self, batch_id: BatchId) -> Option<&CelestiaLocalRecord> {
        self.batches.get(&batch_id).map(|item| &item.record)
    }

    /// Set the simulated local chain height.
    pub fn set_current_height(&mut self, current_height: u64) {
        self.current_height = current_height;
    }

    /// Advance the simulated local chain height.
    pub fn advance_height(&mut self, delta: u64) {
        self.current_height = self.current_height.saturating_add(delta);
    }

    /// Mark one payload as missing.
    pub fn mark_payload_missing(&mut self, batch_id: BatchId) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.payload_available = false;
        true
    }

    /// Remove the local anchor for one blob.
    pub fn clear_anchor(&mut self, batch_id: BatchId) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.anchor_height = None;
        true
    }

    /// Replace the recorded namespace for one blob.
    pub fn forge_namespace(&mut self, batch_id: BatchId, namespace_hex: impl Into<String>) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.namespace_hex = namespace_hex.into();
        true
    }

    /// Replace the recorded blob commitment for one blob.
    pub fn forge_blob_commitment(&mut self, batch_id: BatchId, blob_commitment: [u8; 32]) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.blob_commitment = blob_commitment;
        true
    }

    /// Replace the recorded certificate digest for one blob.
    pub fn forge_certificate_digest(
        &mut self,
        batch_id: BatchId,
        certificate_digest: [u8; 32],
    ) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.certificate_digest = certificate_digest;
        true
    }

    /// Replace the recorded theorem digest for one blob.
    pub fn forge_theorem_digest(&mut self, batch_id: BatchId, theorem_digest: [u8; 32]) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.theorem_digest = theorem_digest;
        true
    }

    /// Replace the recorded anchor height for one blob.
    pub fn forge_anchor_height(&mut self, batch_id: BatchId, anchor_height: Option<u64>) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.anchor_height = anchor_height;
        true
    }

    fn publish_checkpoint(
        &mut self,
        request: &PublicationRequest,
        payload_digest: [u8; 32],
    ) -> Result<
        (
            z00z_storage::checkpoint::CheckpointArtifact,
            PublishedBatch,
            CelestiaLocalRecord,
        ),
        DaError,
    > {
        let artifact = artifact_for_request(&request.draft, &request.exec_input)?;
        let theorem = SettlementTheoremBundle::new(
            request.tx_package.clone(),
            artifact.clone(),
            request.exec_input.clone(),
            request.link.clone(),
        )
        .map_err(|_| DaError::PublishFailed)?;
        let theorem_digest = theorem.theorem_digest();
        let checkpoint_id = z00z_storage::checkpoint::derive_checkpoint_id(&artifact)
            .map_err(|_| DaError::PublishFailed)?;
        let publication_route = request.publication_route.clone();
        let route_table_digest = publication_route.route_table_digest;
        let publication_checkpoint = request
            .draft
            .height()
            .max(publication_route.activation_checkpoint)
            .max(1);
        let pub_in = artifact.pub_in();
        let binding =
            bind_publication_contract(request.batch_id, checkpoint_id, route_table_digest, &pub_in);
        verify_request_quorum_binding(request, &binding, theorem_digest)?;

        let namespace_hex = namespace_hex(request);
        let blob_commitment = blob_commitment(
            &namespace_hex,
            payload_digest,
            binding.binding_digest(),
            request.subject.digest(),
            request.certificate.digest(),
            theorem_digest,
        );
        let blob_height = publication_checkpoint.max(self.current_height.saturating_add(1));
        self.current_height = blob_height;

        let published = PublishedBatch {
            batch_id: request.batch_id,
            checkpoint_id,
            publication_checkpoint,
            publication_route,
            pub_in,
            subject_digest: Some(request.subject.digest()),
            certificate_digest: Some(request.certificate.digest()),
            theorem_digest: Some(theorem_digest),
            da_provider: format!("celestia-local/{}", self.source_label),
            blob_ref: format!(
                "celestia-local://{}/{}/{}",
                self.source_label,
                namespace_hex,
                hex::encode(blob_commitment)
            ),
        };
        let record = CelestiaLocalRecord {
            batch_id: request.batch_id,
            source_label: self.source_label.clone(),
            namespace_hex,
            blob_commitment,
            payload_digest,
            publication_digest: binding.binding_digest(),
            subject_digest: request.subject.digest(),
            certificate_digest: request.certificate.digest(),
            theorem_digest,
            blob_height,
            anchor_height: Some(blob_height),
            challenge_window: self.challenge_window,
            unanchored_limit: self.unanchored_limit,
            payload_available: true,
            replay_id: request.idempotency_key.clone(),
        };
        Ok((artifact, published, record))
    }
}

impl DaAdapter for CelestiaLocalAdapter {
    fn publish(&mut self, request: PublicationRequest) -> Result<PublishedBatch, DaError> {
        if !self.replay_ids.insert(request.idempotency_key.clone()) {
            return Err(DaError::ReplayDetected);
        }
        if self.batches.contains_key(&request.batch_id) {
            return Err(DaError::ReplayDetected);
        }

        let payload_digest = request_payload_digest(&request)?;
        let (artifact, published, record) = self.publish_checkpoint(&request, payload_digest)?;
        self.batches.insert(
            request.batch_id,
            CelestiaStoredBatch {
                request,
                artifact,
                published: published.clone(),
                record,
            },
        );
        Ok(published)
    }

    fn resolve(&mut self, batch: &PublishedBatch) -> Result<ResolvedBatch, DaError> {
        let Some(item) = self.batches.get(&batch.batch_id) else {
            return Err(DaError::MissingResolveResult);
        };
        if &item.published != batch {
            return Err(DaError::ResolveFailed);
        }
        if !item.record.payload_available {
            return Err(DaError::MissingPayload);
        }
        if item
            .record
            .anchor_height
            .is_some_and(|anchor_height| anchor_height < item.record.blob_height)
        {
            return Err(DaError::StaleAnchor);
        }
        if item.record.anchor_height.is_none()
            && self.current_height.saturating_sub(item.record.blob_height)
                > item.record.unanchored_limit
        {
            return Err(DaError::UnanchoredHeightExceeded);
        }

        let expected_payload = request_payload_digest(&item.request)?;
        let theorem = SettlementTheoremBundle::new(
            item.request.tx_package.clone(),
            item.artifact.clone(),
            item.request.exec_input.clone(),
            item.request.link.clone(),
        )
        .map_err(|_| DaError::ResolveFailed)?;
        let binding = bind_publication_contract(
            item.published.batch_id,
            item.published.checkpoint_id,
            item.published.publication_route.route_table_digest,
            &item.published.pub_in,
        );
        let expected_namespace = namespace_hex(&item.request);
        if item.record.namespace_hex != expected_namespace {
            return Err(DaError::NamespaceMismatch);
        }
        let expected_blob_commitment = blob_commitment(
            &expected_namespace,
            expected_payload,
            binding.binding_digest(),
            item.request.subject.digest(),
            item.request.certificate.digest(),
            theorem.theorem_digest(),
        );
        if item.record.blob_commitment != expected_blob_commitment {
            return Err(DaError::BlobCommitmentMismatch);
        }
        let expected_certificate_digest = item
            .published
            .certificate_digest
            .ok_or(DaError::ResolveFailed)?;
        if item.record.certificate_digest != expected_certificate_digest {
            return Err(DaError::CertificateMismatch);
        }
        if item.record.payload_digest != expected_payload
            || item.record.publication_digest != binding.binding_digest()
            || item.record.subject_digest
                != item
                    .published
                    .subject_digest
                    .ok_or(DaError::ResolveFailed)?
            || item.record.theorem_digest != theorem.theorem_digest()
        {
            return Err(DaError::MetadataMismatch);
        }
        verify_request_quorum_binding(&item.request, &binding, theorem.theorem_digest())
            .map_err(|_| DaError::ResolveFailed)?;

        Ok(ResolvedBatch::new(
            item.published.clone(),
            item.request.ordered_batch.clone(),
            theorem,
            Some(item.request.subject.clone()),
            Some(item.request.certificate.clone()),
            item.request.nullifiers.clone(),
            None,
            None,
        ))
    }
}

fn namespace_hex(request: &PublicationRequest) -> String {
    let route_digest = request
        .ordered_batch
        .planned
        .route_table_digest
        .into_bytes();
    let digest = hash_parts(
        NAMESPACE_LABEL,
        &[
            &request.batch_id.into_bytes(),
            &route_digest,
            &request.subject.digest(),
        ],
    );
    hex::encode(&digest[..8])
}

fn blob_commitment(
    namespace_hex: &str,
    payload_digest: [u8; 32],
    publication_digest: [u8; 32],
    subject_digest: [u8; 32],
    certificate_digest: [u8; 32],
    theorem_digest: [u8; 32],
) -> [u8; 32] {
    hash_parts(
        COMMITMENT_LABEL,
        &[
            namespace_hex.as_bytes(),
            &payload_digest,
            &publication_digest,
            &subject_digest,
            &certificate_digest,
            &theorem_digest,
        ],
    )
}
