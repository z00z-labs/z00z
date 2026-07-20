#![forbid(unsafe_code)]
//! Local data-availability adapter surfaces for checkpoint publication.
//!
//! Publication readiness is availability evidence only; it does not prove state validity or
//! replace storage-owned checkpoint authority.
//! Validators consume storage-owned checkpoint artifacts plus publication bindings; watchers
//! remain advisory observers of readiness and gap evidence only.
//! Provider SDK or IPFS/Kubo wiring must stay behind adapter boundaries rather than turning
//! provider-native types into checkpoint theorem ownership.

use std::collections::{HashMap, HashSet};

use sha2::{Digest, Sha256};
use thiserror::Error;
use z00z_aggregators::{
    bind_publication_contract, BatchId, PublicationBinding, PublicationRequest, PublishedBatch,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, derive_exec_id, encode_exec_bin, ArchiveManifestVersion,
        CheckpointArchiveEncodingKindV1, CheckpointArchiveEntryKindV1, CheckpointArchiveEntryV1,
        CheckpointArchiveEntryVersion, CheckpointArchiveManifestV1,
        CheckpointArchiveRetentionClassV1, CheckpointArtifact, CheckpointDaLocatorKind,
        CheckpointDaProviderFamily, CheckpointDaReferenceV1, CheckpointDraft, CheckpointExecInput,
        CheckpointFsStore, CheckpointId, CheckpointLifecycleV1, CheckpointPublicationEvidenceV1,
        CheckpointPublicationEvidenceVersion, CheckpointPublicationState, CheckpointStore,
        CheckpointTransitionStatementCoreV1,
    },
    settlement::{check_route_binding_v1, ClaimNullifier, PublicationRouteSnapshotV1},
};
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_validators::{ResolvedBatch, SettlementTheoremBundle};
use z00z_wallets::tx::TxPackage;

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
    #[error("data-availability blob bytes drifted from the published checkpoint contract")]
    BlobBytesMismatch,
    #[error(
        "data-availability inclusion reference drifted from the published checkpoint contract"
    )]
    InclusionReferenceMismatch,
    #[error("data-availability blob retention window expired before retrieval")]
    BlobRetentionExpired,
    #[error("data-availability publication readiness contract failed")]
    ReadinessFailed,
}

pub trait DaAdapter {
    fn publish(&mut self, request: PublicationRequest) -> Result<PublishedBatch, DaError>;

    fn resolve(&mut self, batch: &PublishedBatch) -> Result<ResolvedBatch, DaError>;

    fn publication_ready(
        &mut self,
        batch: &PublishedBatch,
        input: &PublicationReadyInput,
        store: &mut CheckpointFsStore,
    ) -> Result<PublicationReadyRecord, DaError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationReadyInput {
    pub archive_manifest: Option<CheckpointArchiveManifestV1>,
    pub observations_root: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationReadyRecord {
    pub batch_id: BatchId,
    pub checkpoint_id: CheckpointId,
    pub da_reference: CheckpointDaReferenceV1,
    pub publication_evidence: CheckpointPublicationEvidenceV1,
    pub lifecycle: CheckpointLifecycleV1,
}

#[derive(Debug, Clone)]
pub struct PreSealPublicationContract {
    pub payload_commitment: [u8; 32],
    pub publication_height: u64,
    pub statement_core: CheckpointTransitionStatementCoreV1,
    pub archive_manifest: CheckpointArchiveManifestV1,
    pub da_reference: CheckpointDaReferenceV1,
    pub artifact: CheckpointArtifact,
    pub checkpoint_id: CheckpointId,
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
        let preview =
            preview_publication_contract(request, CheckpointDaProviderFamily::LocalArchive)?;
        let artifact = preview.artifact.clone();
        let theorem = SettlementTheoremBundle::new(
            request.tx_package.clone(),
            artifact.clone(),
            request.exec_input.clone(),
            request.link.clone(),
        )
        .map_err(|_| DaError::PublishFailed)?;
        let theorem_digest = theorem.theorem_digest();
        let checkpoint_id = preview.checkpoint_id;
        if request.link.checkpoint_id() != checkpoint_id
            || request.link.prep_snapshot_id() != request.exec_input.prep_snapshot_id()
        {
            return Err(DaError::PublishFailed);
        }
        let publication_route = request.publication_route.clone();
        let route_table_digest = publication_route.route_table_digest;
        let publication_checkpoint = preview.publication_height;
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
            None,
            item.request.ordered_batch.clone(),
            theorem,
            Some(item.request.subject.clone()),
            Some(item.request.certificate.clone()),
            item.request.nullifiers.clone(),
            None,
            None,
        ))
    }

    fn publication_ready(
        &mut self,
        batch: &PublishedBatch,
        input: &PublicationReadyInput,
        store: &mut CheckpointFsStore,
    ) -> Result<PublicationReadyRecord, DaError> {
        self.resolve(batch)?;
        let record = self
            .record(batch.batch_id)
            .ok_or(DaError::MissingResolveResult)?
            .clone();
        persist_publication_ready(
            store,
            batch,
            input,
            CheckpointDaProviderFamily::LocalArchive,
            &batch.blob_ref,
            record.payload_digest,
            batch.publication_checkpoint,
        )
    }
}

pub fn publication_height_for_request(
    draft: &CheckpointDraft,
    publication_route: &PublicationRouteSnapshotV1,
) -> u64 {
    draft
        .height()
        .max(publication_route.activation_checkpoint)
        .max(1)
}

pub fn preview_publication_contract(
    request: &PublicationRequest,
    provider_family: CheckpointDaProviderFamily,
) -> Result<PreSealPublicationContract, DaError> {
    preview_publication_contract_parts(
        request.batch_id,
        &request.idempotency_key,
        &request.publication_route,
        &request.draft,
        &request.tx_package,
        &request.exec_input,
        &request.nullifiers,
        provider_family,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn preview_publication_contract_parts(
    batch_id: BatchId,
    replay_id: &str,
    publication_route: &PublicationRouteSnapshotV1,
    draft: &CheckpointDraft,
    tx_package: &TxPackage,
    exec_input: &CheckpointExecInput,
    nullifiers: &[ClaimNullifier],
    provider_family: CheckpointDaProviderFamily,
) -> Result<PreSealPublicationContract, DaError> {
    let payload_commitment = payload_commitment_from_parts(
        batch_id,
        replay_id,
        publication_route,
        draft,
        tx_package,
        exec_input,
        nullifiers,
    )?;
    let publication_height = publication_height_for_request(draft, publication_route);
    let statement_core = statement_core_from_parts(
        batch_id,
        publication_route,
        tx_package,
        exec_input,
        nullifiers,
    )?;
    let exec_bytes = encode_exec_bin(exec_input).map_err(|_| DaError::PublishFailed)?;
    let exec_input_id = derive_exec_id(&exec_bytes);
    let archive_manifest = archive_manifest_from_parts(
        batch_id,
        publication_route,
        draft,
        tx_package,
        exec_input,
        exec_input_id,
        payload_commitment,
        statement_core,
    )?;
    let da_reference = CheckpointDaReferenceV1::new(
        z00z_storage::checkpoint::CheckpointDaReferenceVersion::CURRENT,
        provider_family,
        CheckpointDaLocatorKind::OpaqueProviderRef,
        canonical_locator_value(
            batch_id,
            provider_family,
            publication_height,
            payload_commitment,
        ),
        payload_commitment,
        archive_manifest.statement_core_digest(),
        archive_manifest.archive_manifest_root(),
        publication_height,
    )
    .map_err(|_| DaError::PublishFailed)?;
    let proof = draft
        .attest_proof(exec_input.prep_snapshot_id(), exec_input_id)
        .map_err(|_| DaError::PublishFailed)?;
    let artifact = draft
        .finalize(proof)
        .map_err(|_| DaError::PublishFailed)?
        .bind_canonical_v1(
            statement_core,
            z00z_storage::checkpoint::CheckpointTransitionStatementFinalV1::new(
                da_reference.da_ref(),
            ),
        )
        .map_err(|_| DaError::PublishFailed)?;
    let checkpoint_id = derive_checkpoint_id(&artifact).map_err(|_| DaError::PublishFailed)?;
    Ok(PreSealPublicationContract {
        payload_commitment,
        publication_height,
        statement_core,
        archive_manifest,
        da_reference,
        artifact,
        checkpoint_id,
    })
}

pub(crate) fn request_payload_bytes(request: &PublicationRequest) -> Result<Vec<u8>, DaError> {
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
    let mut payload = Vec::new();
    payload.extend_from_slice(b"z00z.rollup.celestia-local.payload.v1");
    append_payload_part(&mut payload, &request.batch_id.into_bytes());
    append_payload_part(&mut payload, request.idempotency_key.as_bytes());
    append_payload_part(&mut payload, &publication_route_bytes);
    append_payload_part(&mut payload, &pub_in_bytes);
    append_payload_part(&mut payload, &tx_package_bytes);
    append_payload_part(&mut payload, &exec_input_bytes);
    append_payload_part(&mut payload, &nullifier_bytes);
    Ok(payload)
}

pub(crate) fn request_payload_digest(request: &PublicationRequest) -> Result<[u8; 32], DaError> {
    payload_commitment_from_parts(
        request.batch_id,
        &request.idempotency_key,
        &request.publication_route,
        &request.draft,
        &request.tx_package,
        &request.exec_input,
        &request.nullifiers,
    )
}

#[allow(clippy::too_many_arguments)]
fn payload_commitment_from_parts(
    batch_id: BatchId,
    replay_id: &str,
    publication_route: &PublicationRouteSnapshotV1,
    draft: &CheckpointDraft,
    tx_package: &TxPackage,
    exec_input: &CheckpointExecInput,
    nullifiers: &[ClaimNullifier],
) -> Result<[u8; 32], DaError> {
    let pub_in = draft.pub_in();
    let pub_in_bytes = JsonCodec
        .serialize(&pub_in)
        .map_err(|_| DaError::PublishFailed)?;
    let publication_route_bytes = JsonCodec
        .serialize(publication_route)
        .map_err(|_| DaError::PublishFailed)?;
    let nullifier_bytes = nullifier_bytes(nullifiers);
    let tx_package_bytes = JsonCodec
        .serialize(tx_package)
        .map_err(|_| DaError::PublishFailed)?;
    let exec_input_bytes = encode_exec_bin(exec_input).map_err(|_| DaError::PublishFailed)?;
    Ok(hash_parts(
        b"z00z.rollup.local-da.payload.v1",
        &[
            &batch_id.into_bytes(),
            replay_id.as_bytes(),
            &publication_route_bytes,
            &pub_in_bytes,
            &tx_package_bytes,
            &exec_input_bytes,
            &nullifier_bytes,
        ],
    ))
}

fn statement_core_from_parts(
    batch_id: BatchId,
    publication_route: &PublicationRouteSnapshotV1,
    tx_package: &TxPackage,
    exec_input: &CheckpointExecInput,
    nullifiers: &[ClaimNullifier],
) -> Result<CheckpointTransitionStatementCoreV1, DaError> {
    let exec_bytes = encode_exec_bin(exec_input).map_err(|_| DaError::PublishFailed)?;
    let route_bytes = JsonCodec
        .serialize(publication_route)
        .map_err(|_| DaError::PublishFailed)?;
    let tx_package_bytes = JsonCodec
        .serialize(tx_package)
        .map_err(|_| DaError::PublishFailed)?;
    let nullifier_bytes = nullifier_bytes(nullifiers);
    let delta_root = hash_parts(
        b"z00z.rollup.checkpoint.delta-root.v1",
        &[&batch_id.into_bytes(), &route_bytes, &exec_bytes],
    );
    let witness_root = hash_parts(
        b"z00z.rollup.checkpoint.witness-root.v1",
        &[&exec_bytes, &tx_package_bytes],
    );
    let journal_digest = hash_parts(
        b"z00z.rollup.checkpoint.journal-digest.v1",
        &[&batch_id.into_bytes(), &nullifier_bytes, &route_bytes],
    );
    Ok(CheckpointTransitionStatementCoreV1::from_exec(
        exec_input,
        delta_root,
        witness_root,
        journal_digest,
    ))
}

#[allow(clippy::too_many_arguments)]
fn archive_manifest_from_parts(
    batch_id: BatchId,
    publication_route: &PublicationRouteSnapshotV1,
    draft: &CheckpointDraft,
    tx_package: &TxPackage,
    exec_input: &CheckpointExecInput,
    exec_input_id: z00z_storage::checkpoint::CheckpointExecInputId,
    payload_commitment: [u8; 32],
    statement_core: CheckpointTransitionStatementCoreV1,
) -> Result<CheckpointArchiveManifestV1, DaError> {
    let route_bytes = JsonCodec
        .serialize(publication_route)
        .map_err(|_| DaError::PublishFailed)?;
    let tx_package_bytes = JsonCodec
        .serialize(tx_package)
        .map_err(|_| DaError::PublishFailed)?;
    let exec_bytes = encode_exec_bin(exec_input).map_err(|_| DaError::PublishFailed)?;
    let proof_bytes = exact_tx_proof_bytes(exec_input);
    let statement_core_digest =
        z00z_storage::checkpoint::CheckpointTransitionStatementV1::from_draft(
            draft,
            exec_input.prep_snapshot_id(),
            exec_input_id,
        )
        .statement_core_digest_v1(&statement_core);
    let epoch_manifest_root = hash_parts(
        b"z00z.rollup.checkpoint.epoch-manifest-root.v1",
        &[&batch_id.into_bytes(), &route_bytes, &payload_commitment],
    );
    let raw_tx_package_root = hash_parts(
        b"z00z.rollup.checkpoint.raw-tx-package-root.v1",
        &[&tx_package_bytes],
    );
    let exact_tx_proof_bytes_root = hash_parts(
        b"z00z.rollup.checkpoint.exact-tx-proof-root.v1",
        &[&proof_bytes],
    );
    let witness_archive_root = hash_parts(
        b"z00z.rollup.checkpoint.witness-archive-root.v1",
        &[&statement_core.witness_root(), &exec_bytes],
    );
    let delta_journal_root = hash_parts(
        b"z00z.rollup.checkpoint.delta-journal-root.v1",
        &[
            &statement_core.delta_root(),
            &statement_core.journal_digest(),
        ],
    );
    let archive_provider_receipt_root = hash_parts(
        b"z00z.rollup.checkpoint.archive-provider-receipt-root.v1",
        &[&payload_commitment, &route_bytes],
    );
    let retrieval_audit_root = hash_parts(
        b"z00z.rollup.checkpoint.retrieval-audit-root.v1",
        &[&payload_commitment, &batch_id.into_bytes()],
    );
    let content_address_root = hash_parts(
        b"z00z.rollup.checkpoint.content-address-root.v1",
        &[&payload_commitment, &tx_package_bytes],
    );
    let entries = vec![
        archive_entry(
            CheckpointArchiveEntryKindV1::RawTxPackage,
            0,
            raw_tx_package_root,
            tx_package_bytes.len() as u64,
            CheckpointArchiveRetentionClassV1::ArchiveRequired,
            CheckpointArchiveEncodingKindV1::CanonicalJsonV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::ExactTxProofBytes,
            1,
            exact_tx_proof_bytes_root,
            proof_bytes.len() as u64,
            CheckpointArchiveRetentionClassV1::DisputeRequired,
            CheckpointArchiveEncodingKindV1::CanonicalBinV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::WitnessArchive,
            2,
            witness_archive_root,
            exec_bytes.len() as u64,
            CheckpointArchiveRetentionClassV1::AuditRequired,
            CheckpointArchiveEncodingKindV1::CanonicalBinV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::DeltaJournal,
            3,
            delta_journal_root,
            route_bytes.len() as u64,
            CheckpointArchiveRetentionClassV1::DisputeRequired,
            CheckpointArchiveEncodingKindV1::CanonicalJsonV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::ArchiveProviderReceipt,
            4,
            archive_provider_receipt_root,
            payload_commitment.len() as u64,
            CheckpointArchiveRetentionClassV1::AuditRequired,
            CheckpointArchiveEncodingKindV1::ProviderPayloadV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::RetrievalAudit,
            5,
            retrieval_audit_root,
            payload_commitment.len() as u64,
            CheckpointArchiveRetentionClassV1::AuditRequired,
            CheckpointArchiveEncodingKindV1::ProviderPayloadV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::ContentAddressIndex,
            6,
            content_address_root,
            payload_commitment.len() as u64,
            CheckpointArchiveRetentionClassV1::ArchiveRequired,
            CheckpointArchiveEncodingKindV1::ProviderPayloadV1,
        )?,
    ];
    CheckpointArchiveManifestV1::new(
        ArchiveManifestVersion::CURRENT,
        statement_core_digest,
        exec_input_id,
        exec_input.prep_snapshot_id(),
        statement_core.tx_data_root(),
        statement_core.delta_root(),
        statement_core.witness_root(),
        statement_core.journal_digest(),
        epoch_manifest_root,
        raw_tx_package_root,
        exact_tx_proof_bytes_root,
        witness_archive_root,
        delta_journal_root,
        payload_commitment,
        archive_provider_receipt_root,
        retrieval_audit_root,
        content_address_root,
        entries,
        3,
    )
    .map_err(|_| DaError::PublishFailed)
}

fn archive_entry(
    entry_kind: CheckpointArchiveEntryKindV1,
    ordinal: u32,
    content_digest: [u8; 32],
    byte_length: u64,
    retention_class: CheckpointArchiveRetentionClassV1,
    encoding_kind: CheckpointArchiveEncodingKindV1,
) -> Result<CheckpointArchiveEntryV1, DaError> {
    CheckpointArchiveEntryV1::new(
        CheckpointArchiveEntryVersion::CURRENT,
        entry_kind,
        ordinal,
        content_digest,
        byte_length,
        retention_class,
        encoding_kind,
    )
    .map_err(|_| DaError::PublishFailed)
}

fn exact_tx_proof_bytes(exec_input: &CheckpointExecInput) -> Vec<u8> {
    let mut out = Vec::new();
    for tx in exec_input.txs() {
        out.extend_from_slice(&(tx.tx_proof().len() as u64).to_le_bytes());
        out.extend_from_slice(tx.tx_proof());
    }
    out
}

fn canonical_locator_value(
    batch_id: BatchId,
    provider_family: CheckpointDaProviderFamily,
    publication_height: u64,
    payload_commitment: [u8; 32],
) -> String {
    format!(
        "checkpoint-da://{}/{}/{}/{}",
        provider_family.as_str(),
        hex::encode(batch_id.into_bytes()),
        publication_height,
        hex::encode(payload_commitment)
    )
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

fn append_payload_part(payload: &mut Vec<u8>, part: &[u8]) {
    payload.extend_from_slice(&(part.len() as u64).to_le_bytes());
    payload.extend_from_slice(part);
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

pub(crate) fn persist_publication_ready(
    store: &mut CheckpointFsStore,
    batch: &PublishedBatch,
    input: &PublicationReadyInput,
    provider_family: CheckpointDaProviderFamily,
    _locator_value: &str,
    payload_commitment: [u8; 32],
    readiness_height: u64,
) -> Result<PublicationReadyRecord, DaError> {
    let manifest = input
        .archive_manifest
        .as_ref()
        .ok_or(DaError::ReadinessFailed)?;
    if input.observations_root == [0u8; 32]
        || manifest.da_payload_commitment() != payload_commitment
        || readiness_height == 0
    {
        return Err(DaError::ReadinessFailed);
    }

    let da_reference = CheckpointDaReferenceV1::new(
        z00z_storage::checkpoint::CheckpointDaReferenceVersion::CURRENT,
        provider_family,
        CheckpointDaLocatorKind::OpaqueProviderRef,
        canonical_locator_value(
            batch.batch_id,
            provider_family,
            readiness_height,
            payload_commitment,
        ),
        payload_commitment,
        manifest.statement_core_digest(),
        manifest.archive_manifest_root(),
        readiness_height,
    )
    .map_err(|_| DaError::ReadinessFailed)?;
    let evidence = CheckpointPublicationEvidenceV1::new(
        CheckpointPublicationEvidenceVersion::CURRENT,
        manifest.statement_core_digest(),
        da_reference.da_ref(),
        manifest.archive_manifest_root(),
        payload_commitment,
        CheckpointPublicationState::DaPublicationReady,
        provider_family,
        readiness_height,
        readiness_height,
        input.observations_root,
    )
    .map_err(|_| DaError::ReadinessFailed)?;

    let artifact = store
        .load_artifact(&batch.checkpoint_id)
        .map_err(|_| DaError::ReadinessFailed)?;
    let link = store
        .load_link(&batch.checkpoint_id)
        .map_err(|_| DaError::ReadinessFailed)?;
    let predecessor = match link.prev_checkpoint_id() {
        Some(checkpoint_id) => Some(
            store
                .load_artifact(&checkpoint_id)
                .map_err(|_| DaError::ReadinessFailed)?,
        ),
        None => None,
    };
    let lifecycle = CheckpointLifecycleV1::from_artifact(&artifact)
        .map_err(|_| DaError::ReadinessFailed)?
        .link(
            &artifact,
            &link,
            predecessor.as_ref(),
            manifest.statement_core_digest(),
        )
        .map_err(|_| DaError::ReadinessFailed)?
        .publication_ready(&evidence)
        .map_err(|_| DaError::ReadinessFailed)?;

    store
        .save_archive_manifest(batch.checkpoint_id, manifest)
        .map_err(|_| DaError::ReadinessFailed)?;
    store
        .save_da_reference(batch.checkpoint_id, &da_reference)
        .map_err(|_| DaError::ReadinessFailed)?;
    store
        .save_publication_evidence(batch.checkpoint_id, &evidence)
        .map_err(|_| DaError::ReadinessFailed)?;
    store
        .save_lifecycle(&lifecycle)
        .map_err(|_| DaError::ReadinessFailed)?;

    Ok(PublicationReadyRecord {
        batch_id: batch.batch_id,
        checkpoint_id: batch.checkpoint_id,
        da_reference,
        publication_evidence: evidence,
        lifecycle,
    })
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
