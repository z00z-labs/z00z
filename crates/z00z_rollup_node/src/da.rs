#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};

use sha2::{Digest, Sha256};
use thiserror::Error;
use z00z_aggregators::{
    bind_publication_contract, BatchId, PublicationBinding, PublicationRequest, PublishedBatch,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, derive_exec_id, encode_exec_bin, CheckpointArtifact, CheckpointDraft,
        CheckpointExecInput,
    },
    settlement::{check_route_binding_v1, ClaimNullifier},
};
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_validators::{ResolvedBatch, SettlementTheoremBundle};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DaError {
    #[error("data-availability adapter is not configured")]
    NotConfigured,
    #[error("data-availability publish failed")]
    PublishFailed,
    #[error("data-availability resolve failed")]
    ResolveFailed,
    #[error("data-availability metadata drifted from the published checkpoint contract")]
    MetadataMismatch,
    #[error("data-availability resolve result is missing")]
    MissingResolveResult,
    #[error("data-availability replayed the same external input id")]
    ReplayDetected,
    #[error("data-availability blob namespace drifted from the published checkpoint contract")]
    NamespaceMismatch,
    #[error("data-availability blob commitment drifted from the published checkpoint contract")]
    BlobCommitmentMismatch,
    #[error("data-availability payload is missing during the local challenge window")]
    MissingPayload,
    #[error("data-availability blob anchor drifted behind the published local height")]
    StaleAnchor,
    #[error("data-availability certificate digest drifted from the published checkpoint contract")]
    CertificateMismatch,
    #[error("data-availability unanchored height exceeded the local safety limit")]
    UnanchoredHeightExceeded,
}

pub trait DaAdapter {
    fn publish(&mut self, request: PublicationRequest) -> Result<PublishedBatch, DaError>;

    fn resolve(&mut self, batch: &PublishedBatch) -> Result<ResolvedBatch, DaError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalResolveState {
    Ready,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalAdapterRecord {
    pub batch_id: BatchId,
    pub source_label: String,
    pub payload_digest: [u8; 32],
    pub publication_digest: [u8; 32],
    pub subject_digest: [u8; 32],
    pub certificate_digest: [u8; 32],
    pub theorem_digest: [u8; 32],
    pub resolve_result: LocalResolveState,
    pub replay_id: String,
}

#[derive(Debug, Clone)]
struct LocalStoredBatch {
    request: PublicationRequest,
    artifact: CheckpointArtifact,
    published: PublishedBatch,
    record: LocalAdapterRecord,
}

#[derive(Debug, Clone, Default)]
pub struct LocalDaAdapter {
    source_label: String,
    replay_ids: HashSet<String>,
    batches: HashMap<BatchId, LocalStoredBatch>,
}

impl LocalDaAdapter {
    #[must_use]
    pub fn new(source_label: impl Into<String>) -> Self {
        Self {
            source_label: source_label.into(),
            replay_ids: HashSet::new(),
            batches: HashMap::new(),
        }
    }

    #[must_use]
    pub fn record(&self, batch_id: BatchId) -> Option<&LocalAdapterRecord> {
        self.batches.get(&batch_id).map(|item| &item.record)
    }

    pub fn mark_resolve_missing(&mut self, batch_id: BatchId) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.resolve_result = LocalResolveState::Missing;
        true
    }

    pub fn forge_source_label(
        &mut self,
        batch_id: BatchId,
        source_label: impl Into<String>,
    ) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.source_label = source_label.into();
        true
    }

    pub fn forge_payload_digest(&mut self, batch_id: BatchId, payload_digest: [u8; 32]) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.payload_digest = payload_digest;
        true
    }

    pub fn forge_publication_digest(
        &mut self,
        batch_id: BatchId,
        publication_digest: [u8; 32],
    ) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.publication_digest = publication_digest;
        true
    }

    pub fn forge_subject_digest(&mut self, batch_id: BatchId, subject_digest: [u8; 32]) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.subject_digest = subject_digest;
        true
    }

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

    pub fn forge_theorem_digest(&mut self, batch_id: BatchId, theorem_digest: [u8; 32]) -> bool {
        let Some(item) = self.batches.get_mut(&batch_id) else {
            return false;
        };
        item.record.theorem_digest = theorem_digest;
        true
    }

    fn publish_checkpoint(
        &self,
        request: &PublicationRequest,
        payload_digest: [u8; 32],
    ) -> Result<(CheckpointArtifact, PublishedBatch, LocalAdapterRecord), DaError> {
        let artifact = artifact_for_request(&request.draft, &request.exec_input)?;
        let theorem = SettlementTheoremBundle::new(
            request.tx_package.clone(),
            artifact.clone(),
            request.exec_input.clone(),
            request.link.clone(),
        )
        .map_err(|_| DaError::PublishFailed)?;
        let theorem_digest = theorem.theorem_digest();
        let checkpoint_id = derive_checkpoint_id(&artifact).map_err(|_| DaError::PublishFailed)?;
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
        let published = PublishedBatch {
            batch_id: request.batch_id,
            checkpoint_id,
            publication_checkpoint,
            publication_route,
            pub_in,
            subject_digest: Some(request.subject.digest()),
            certificate_digest: Some(request.certificate.digest()),
            theorem_digest: Some(theorem_digest),
            da_provider: self.source_label.clone(),
            blob_ref: format!(
                "local-da://{}/{}",
                self.source_label,
                hex::encode(binding.binding_digest())
            ),
        };
        let record = LocalAdapterRecord {
            batch_id: request.batch_id,
            source_label: self.source_label.clone(),
            payload_digest,
            publication_digest: binding.binding_digest(),
            subject_digest: request.subject.digest(),
            certificate_digest: request.certificate.digest(),
            theorem_digest,
            resolve_result: LocalResolveState::Ready,
            replay_id: request.idempotency_key.clone(),
        };
        Ok((artifact, published, record))
    }
}

impl DaAdapter for LocalDaAdapter {
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
            LocalStoredBatch {
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
        if item.record.resolve_result != LocalResolveState::Ready {
            return Err(DaError::MissingResolveResult);
        }
        if &item.published != batch {
            return Err(DaError::ResolveFailed);
        }

        let expected_payload = request_payload_digest(&item.request)?;
        let expected_record = LocalAdapterRecord {
            batch_id: item.request.batch_id,
            source_label: self.source_label.clone(),
            payload_digest: expected_payload,
            publication_digest: bind_publication_contract(
                item.published.batch_id,
                item.published.checkpoint_id,
                item.published.publication_route.route_table_digest,
                &item.published.pub_in,
            )
            .binding_digest(),
            subject_digest: item
                .published
                .subject_digest
                .ok_or(DaError::ResolveFailed)?,
            certificate_digest: item
                .published
                .certificate_digest
                .ok_or(DaError::ResolveFailed)?,
            theorem_digest: item
                .published
                .theorem_digest
                .ok_or(DaError::ResolveFailed)?,
            resolve_result: LocalResolveState::Ready,
            replay_id: item.request.idempotency_key.clone(),
        };
        if item.record != expected_record {
            return Err(DaError::MetadataMismatch);
        }

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

pub(crate) fn artifact_for_request(
    draft: &CheckpointDraft,
    exec_input: &CheckpointExecInput,
) -> Result<CheckpointArtifact, DaError> {
    let exec_bytes = encode_exec_bin(exec_input).map_err(|_| DaError::PublishFailed)?;
    let exec_input_id = derive_exec_id(&exec_bytes);
    let proof = draft
        .attest_proof(exec_input.prep_snapshot_id(), exec_input_id)
        .map_err(|_| DaError::PublishFailed)?;
    draft.finalize(proof).map_err(|_| DaError::PublishFailed)
}

pub(crate) fn request_payload_digest(request: &PublicationRequest) -> Result<[u8; 32], DaError> {
    let pub_in = request.draft.pub_in();
    let pub_in_bytes = JsonCodec
        .serialize(&pub_in)
        .map_err(|_| DaError::PublishFailed)?;
    let publication_route_bytes = JsonCodec
        .serialize(&request.publication_route)
        .map_err(|_| DaError::PublishFailed)?;
    let nullifier_bytes = nullifier_bytes(&request.nullifiers);
    let tx_package_bytes = JsonCodec
        .serialize(&request.tx_package)
        .map_err(|_| DaError::PublishFailed)?;
    let exec_input_bytes =
        encode_exec_bin(&request.exec_input).map_err(|_| DaError::PublishFailed)?;
    let link_bytes = JsonCodec
        .serialize(&request.link)
        .map_err(|_| DaError::PublishFailed)?;
    let subject_bytes = request.subject.encode();
    let certificate_bytes = request.certificate.encode();
    Ok(hash_parts(
        b"z00z.rollup.local-da.payload.v1",
        &[
            &request.batch_id.into_bytes(),
            request.idempotency_key.as_bytes(),
            &publication_route_bytes,
            &pub_in_bytes,
            &tx_package_bytes,
            &exec_input_bytes,
            &link_bytes,
            &nullifier_bytes,
            &subject_bytes,
            &certificate_bytes,
        ],
    ))
}

pub(crate) fn verify_request_quorum_binding(
    request: &PublicationRequest,
    binding: &PublicationBinding,
    theorem_digest: [u8; 32],
) -> Result<(), DaError> {
    let publication_checkpoint = request
        .draft
        .height()
        .max(request.publication_route.activation_checkpoint)
        .max(1);
    check_route_binding_v1(
        &request.publication_route,
        request
            .ordered_batch
            .planned
            .route_table_digest
            .into_bytes(),
        Some(publication_checkpoint),
        Some((
            request.ordered_batch.planned.route.shard_id.as_u32(),
            request.ordered_batch.planned.route.routing_generation,
        )),
    )
    .map_err(|_| DaError::PublishFailed)?;
    request
        .subject
        .verify_binding(&request.ordered_batch, binding, theorem_digest)
        .map_err(|_| DaError::PublishFailed)?;
    request
        .certificate
        .verify_subject(&request.subject)
        .map_err(|_| DaError::PublishFailed)?;
    Ok(())
}

fn nullifier_bytes(nullifiers: &[ClaimNullifier]) -> Vec<u8> {
    let mut out = Vec::with_capacity(nullifiers.len() * 32);
    for nullifier in nullifiers {
        out.extend_from_slice(nullifier.as_bytes());
    }
    out
}

pub(crate) fn hash_parts(label: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(label);
    for part in parts {
        hasher.update((*part).len().to_le_bytes());
        hasher.update(part);
    }
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    mod theorem_fixture {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/test_theorem_fixture.rs"
        ));
    }

    use z00z_aggregators::PublicationRequest;

    use super::{DaAdapter, DaError, LocalDaAdapter, LocalResolveState};

    #[test]
    fn test_local_adapter_roundtrip() {
        let request = request_fixture();

        let mut left = LocalDaAdapter::new("local-ext");
        let mut right = LocalDaAdapter::new("local-ext");

        let left_published = left.publish(request.clone()).expect("left publish");
        let right_published = right.publish(request.clone()).expect("right publish");
        let left_record = left
            .record(left_published.batch_id)
            .expect("left record")
            .clone();
        let right_record = right
            .record(right_published.batch_id)
            .expect("right record")
            .clone();

        assert_eq!(left_published, right_published);
        assert_eq!(left_record, right_record);
        assert_eq!(left_record.resolve_result, LocalResolveState::Ready);
        let resolved = left.resolve(&left_published).expect("resolve");
        assert_eq!(resolved.published, left_published);
        assert_eq!(resolved.link(), &request.link);
        assert_eq!(resolved.exec_input(), &request.exec_input);
    }

    #[test]
    fn local_adapter_rejects_metadata_drift() {
        let request = request_fixture();
        let mut adapter = LocalDaAdapter::new("local-ext");
        let published = adapter.publish(request).expect("publish");
        assert!(adapter.forge_payload_digest(published.batch_id, [0xEE; 32]));

        let err = adapter
            .resolve(&published)
            .expect_err("payload digest drift must reject");

        assert_eq!(err, DaError::MetadataMismatch);
    }

    fn request_fixture() -> PublicationRequest {
        theorem_fixture::publication_request([0x51; 32], "external-input-1")
    }
}
