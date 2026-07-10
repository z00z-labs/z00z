#[path = "support/test_theorem_fixture.rs"]
mod theorem_fixture;

use std::path::Path;

use tempfile::tempdir;
use z00z_aggregators::{BatchId, PublicationRequest};
use z00z_rollup_node::{
    preview_publication_contract, DaAdapter, DaError, LocalDaAdapter, LocalResolveState,
    PublicationReadyInput,
};
use z00z_storage::{
    checkpoint::{
        CheckpointArchiveManifestV1, CheckpointDaProviderFamily, CheckpointFsStore,
        CheckpointLifecycleStatus, CheckpointPublicationState, CheckpointStore,
    },
    snapshot::{PrepFsStore, PrepSnapshot, PrepSnapshotStore},
};

#[test]
fn local_adapter_publish_resolve_contract() {
    let request = request_fixture([0x21; 32], "external-input-1");
    let expected_nullifiers = request.nullifiers.clone();
    let expected_subject_digest = request.subject.digest();
    let expected_certificate_digest = request.certificate.digest();
    let expected_ordered = request.ordered_batch.clone();
    let mut adapter = LocalDaAdapter::new("local-bridge");

    let published = adapter.publish(request.clone()).expect("publish");
    let record = adapter.record(published.batch_id).expect("record");

    assert_eq!(record.batch_id, published.batch_id);
    assert_eq!(record.source_label, "local-bridge");
    assert_eq!(record.resolve_result, LocalResolveState::Ready);
    assert_eq!(record.replay_id, request.idempotency_key);
    assert_ne!(record.payload_digest, [0u8; 32]);
    assert_ne!(record.publication_digest, [0u8; 32]);
    assert_eq!(record.subject_digest, expected_subject_digest);
    assert_eq!(record.certificate_digest, expected_certificate_digest);
    assert_eq!(published.subject_digest, Some(expected_subject_digest));
    assert_eq!(
        published.certificate_digest,
        Some(expected_certificate_digest)
    );
    assert_eq!(published.theorem_digest, Some(record.theorem_digest));
    assert_eq!(published.batch_id, request.batch_id);
    assert_eq!(published.da_provider, "local-bridge");
    assert!(published.blob_ref.starts_with("local-da://local-bridge/"));
    assert_eq!(published.publication_route, request.publication_route);
    assert!(published.publication_checkpoint >= request.publication_route.activation_checkpoint);

    let resolved = adapter.resolve(&published).expect("resolve");

    assert_eq!(resolved.published, published);
    assert_eq!(resolved.ordered, expected_ordered);
    assert_eq!(resolved.subject.as_ref(), Some(&request.subject));
    assert_eq!(resolved.certificate.as_ref(), Some(&request.certificate));
    assert_eq!(resolved.nullifiers, expected_nullifiers);
    assert_eq!(resolved.link(), &request.link);
    assert_eq!(resolved.exec_input(), &request.exec_input);
    assert_eq!(
        resolved.ordered.planned.route_table_digest.into_bytes(),
        published.publication_route.route_table_digest
    );
}

#[test]
fn local_adapter_rejects_forged_source() {
    let mut adapter = LocalDaAdapter::new("local-bridge");
    let published = adapter
        .publish(request_fixture([0x22; 32], "external-input-2"))
        .expect("publish");
    assert!(adapter.forge_source_label(published.batch_id, "forged-bridge"));

    let err = adapter
        .resolve(&published)
        .expect_err("forged source label must reject");

    assert_eq!(err, DaError::MetadataMismatch);
}

#[test]
fn local_adapter_rejects_forged_digest() {
    let mut adapter = LocalDaAdapter::new("local-bridge");
    let published = adapter
        .publish(request_fixture([0x23; 32], "external-input-3"))
        .expect("publish");
    assert!(adapter.forge_publication_digest(published.batch_id, [0xDD; 32]));

    let err = adapter
        .resolve(&published)
        .expect_err("wrong digest must reject");

    assert_eq!(err, DaError::MetadataMismatch);
}

#[test]
fn local_adapter_rejects_payload_drift() {
    let mut adapter = LocalDaAdapter::new("local-bridge");
    let published = adapter
        .publish(request_fixture([0x24; 32], "external-input-4"))
        .expect("publish");
    assert!(adapter.forge_payload_digest(published.batch_id, [0xEE; 32]));

    let err = adapter
        .resolve(&published)
        .expect_err("forged payload digest must reject");

    assert_eq!(err, DaError::MetadataMismatch);
}

#[test]
fn local_adapter_rejects_subject_digest_drift() {
    let mut adapter = LocalDaAdapter::new("local-bridge");
    let published = adapter
        .publish(request_fixture([0x2A; 32], "external-input-4b"))
        .expect("publish");
    assert!(adapter.forge_subject_digest(published.batch_id, [0xBC; 32]));

    let err = adapter
        .resolve(&published)
        .expect_err("forged subject digest must reject");

    assert_eq!(err, DaError::MetadataMismatch);
}

#[test]
fn local_adapter_rejects_missing_resolve() {
    let mut adapter = LocalDaAdapter::new("local-bridge");
    let published = adapter
        .publish(request_fixture([0x25; 32], "external-input-5"))
        .expect("publish");
    assert!(adapter.mark_resolve_missing(published.batch_id));

    let err = adapter
        .resolve(&published)
        .expect_err("missing resolve result must reject");

    assert_eq!(err, DaError::MissingResolveResult);
}

#[test]
fn local_adapter_rejects_replayed_input() {
    let mut adapter = LocalDaAdapter::new("local-bridge");
    adapter
        .publish(request_fixture([0x26; 32], "external-input-6"))
        .expect("first publish");

    let err = adapter
        .publish(request_fixture([0x27; 32], "external-input-6"))
        .expect_err("replayed external input id must reject");

    assert_eq!(err, DaError::ReplayDetected);
}

#[test]
fn local_adapter_publication_ready_persists_verified_evidence() {
    let (request, snapshot) =
        theorem_fixture::publication_case([0x27; 32], "external-input-ready-1");
    let temp = tempdir().expect("tempdir");
    let mut store = sealed_store(temp.path(), &request, &snapshot);
    let mut adapter = LocalDaAdapter::new("local-bridge");
    let published = adapter.publish(request.clone()).expect("publish");
    let preview = preview_publication_contract(&request, CheckpointDaProviderFamily::LocalArchive)
        .expect("preview");

    assert_eq!(
        store
            .load_archive_manifest(&published.checkpoint_id)
            .expect("staged manifest promoted at seal"),
        preview.archive_manifest
    );
    assert_eq!(
        store
            .load_da_reference(&published.checkpoint_id)
            .expect("staged da reference promoted at seal"),
        preview.da_reference
    );
    assert!(store
        .load_publication_evidence(&published.checkpoint_id)
        .is_err());
    assert!(store.load_lifecycle(&published.checkpoint_id).is_err());

    let manifest = preview.archive_manifest.clone();
    let ready = adapter
        .publication_ready(
            &published,
            &PublicationReadyInput {
                archive_manifest: Some(manifest.clone()),
                observations_root: [0xAA; 32],
            },
            &mut store,
        )
        .expect("publication readiness");

    assert_eq!(
        ready.publication_evidence.publication_state(),
        CheckpointPublicationState::DaPublicationReady
    );
    assert_eq!(
        ready.lifecycle.status(),
        CheckpointLifecycleStatus::PublicationReady
    );
    assert_eq!(
        ready.lifecycle.statement_core_digest(),
        Some(manifest.statement_core_digest())
    );
    assert_eq!(
        ready.lifecycle.challenge_window_start_height(),
        Some(published.publication_checkpoint)
    );
    assert_eq!(
        ready.publication_evidence.payload_commitment(),
        preview.payload_commitment
    );

    assert_eq!(
        store
            .load_archive_manifest(&published.checkpoint_id)
            .expect("stored manifest"),
        manifest
    );
    assert_eq!(
        store
            .load_da_reference(&published.checkpoint_id)
            .expect("stored da reference"),
        ready.da_reference
    );
    assert_eq!(
        store
            .load_publication_evidence(&published.checkpoint_id)
            .expect("stored evidence"),
        ready.publication_evidence
    );
    assert_eq!(
        store
            .load_lifecycle(&published.checkpoint_id)
            .expect("stored lifecycle"),
        ready.lifecycle
    );
}

#[test]
fn local_adapter_publication_ready_rejects_missing_manifest() {
    let (request, snapshot) =
        theorem_fixture::publication_case([0x28; 32], "external-input-ready-2");
    let temp = tempdir().expect("tempdir");
    let mut store = sealed_store(temp.path(), &request, &snapshot);
    let mut adapter = LocalDaAdapter::new("local-bridge");
    let published = adapter.publish(request).expect("publish");

    let err = adapter
        .publication_ready(
            &published,
            &PublicationReadyInput {
                archive_manifest: None,
                observations_root: [0xAB; 32],
            },
            &mut store,
        )
        .expect_err("missing manifest must reject");

    assert_eq!(err, DaError::ReadinessFailed);
    assert!(store
        .load_archive_manifest(&published.checkpoint_id)
        .is_ok());
    assert!(store
        .load_publication_evidence(&published.checkpoint_id)
        .is_err());
}

#[test]
fn local_adapter_publication_ready_rejects_unsealed_store() {
    let (request, _snapshot) =
        theorem_fixture::publication_case([0x29; 32], "external-input-ready-3");
    let temp = tempdir().expect("tempdir");
    let mut store = CheckpointFsStore::new(temp.path());
    let mut adapter = LocalDaAdapter::new("local-bridge");
    let published = adapter.publish(request.clone()).expect("publish");
    let manifest = preview_publication_contract(&request, CheckpointDaProviderFamily::LocalArchive)
        .expect("preview")
        .archive_manifest;

    let err = adapter
        .publication_ready(
            &published,
            &PublicationReadyInput {
                archive_manifest: Some(manifest),
                observations_root: [0xAC; 32],
            },
            &mut store,
        )
        .expect_err("unsealed store must reject");

    assert_eq!(err, DaError::ReadinessFailed);
    assert!(store
        .load_archive_manifest(&published.checkpoint_id)
        .is_err());
    assert!(store
        .load_publication_evidence(&published.checkpoint_id)
        .is_err());
}

#[test]
fn local_adapter_publication_ready_rejects_payload_commitment_drift() {
    let (request, snapshot) =
        theorem_fixture::publication_case([0x2B; 32], "external-input-ready-4");
    let temp = tempdir().expect("tempdir");
    let mut store = sealed_store(temp.path(), &request, &snapshot);
    let mut adapter = LocalDaAdapter::new("local-bridge");
    let preview = preview_publication_contract(&request, CheckpointDaProviderFamily::LocalArchive)
        .expect("preview");
    let published = adapter.publish(request).expect("publish");
    let mut manifest = preview.archive_manifest.clone();
    let tampered = CheckpointArchiveManifestV1::new(
        manifest.version(),
        manifest.statement_core_digest(),
        manifest.checkpoint_exec_input_id(),
        manifest.prep_snapshot_id(),
        manifest.tx_data_root(),
        manifest.delta_root(),
        manifest.witness_root(),
        manifest.journal_digest(),
        manifest.epoch_manifest_root(),
        manifest.raw_tx_package_root(),
        manifest.exact_tx_proof_bytes_root(),
        manifest.witness_archive_root(),
        manifest.delta_journal_root(),
        [0xDD; 32],
        manifest.archive_provider_receipt_root(),
        manifest.retrieval_audit_root(),
        manifest.content_address_root(),
        manifest.entries().to_vec(),
        manifest.min_archive_replicas(),
    )
    .expect("tampered manifest");
    manifest = tampered;

    let err = adapter
        .publication_ready(
            &published,
            &PublicationReadyInput {
                archive_manifest: Some(manifest),
                observations_root: [0xAD; 32],
            },
            &mut store,
        )
        .expect_err("payload drift must reject");

    assert_eq!(err, DaError::ReadinessFailed);
    assert!(store
        .load_archive_manifest(&published.checkpoint_id)
        .is_ok_and(|stored| stored == preview.archive_manifest));
    assert!(store
        .load_publication_evidence(&published.checkpoint_id)
        .is_err());
    assert!(store.load_lifecycle(&published.checkpoint_id).is_err());
}

#[test]
fn local_adapter_publication_ready_rejects_zero_observations_root() {
    let (request, snapshot) =
        theorem_fixture::publication_case([0x2C; 32], "external-input-ready-5");
    let temp = tempdir().expect("tempdir");
    let mut store = sealed_store(temp.path(), &request, &snapshot);
    let mut adapter = LocalDaAdapter::new("local-bridge");
    let published = adapter.publish(request.clone()).expect("publish");
    let manifest = preview_publication_contract(&request, CheckpointDaProviderFamily::LocalArchive)
        .expect("preview")
        .archive_manifest;

    let err = adapter
        .publication_ready(
            &published,
            &PublicationReadyInput {
                archive_manifest: Some(manifest.clone()),
                observations_root: [0u8; 32],
            },
            &mut store,
        )
        .expect_err("zero observations must reject");

    assert_eq!(err, DaError::ReadinessFailed);
    assert!(store
        .load_archive_manifest(&published.checkpoint_id)
        .is_ok_and(|stored| stored == manifest));
    assert!(store
        .load_publication_evidence(&published.checkpoint_id)
        .is_err());
    assert!(store.load_lifecycle(&published.checkpoint_id).is_err());
}

#[test]
fn local_adapter_publication_ready_requires_resolve_path() {
    let (request, snapshot) =
        theorem_fixture::publication_case([0x2D; 32], "external-input-ready-6");
    let temp = tempdir().expect("tempdir");
    let mut store = sealed_store(temp.path(), &request, &snapshot);
    let mut adapter = LocalDaAdapter::new("local-bridge");
    let published = adapter.publish(request.clone()).expect("publish");
    let manifest = preview_publication_contract(&request, CheckpointDaProviderFamily::LocalArchive)
        .expect("preview")
        .archive_manifest;
    assert!(adapter.mark_resolve_missing(published.batch_id));

    let err = adapter
        .publication_ready(
            &published,
            &PublicationReadyInput {
                archive_manifest: Some(manifest.clone()),
                observations_root: [0xAE; 32],
            },
            &mut store,
        )
        .expect_err("missing resolve path must reject");

    assert_eq!(err, DaError::MissingResolveResult);
    assert!(store
        .load_archive_manifest(&published.checkpoint_id)
        .is_ok_and(|stored| stored == manifest));
    assert!(store
        .load_publication_evidence(&published.checkpoint_id)
        .is_err());
    assert!(store.load_lifecycle(&published.checkpoint_id).is_err());
}

fn request_fixture(batch_bytes: [u8; 32], replay_id: &str) -> PublicationRequest {
    let request = theorem_fixture::publication_request(batch_bytes, replay_id);
    assert_eq!(request.batch_id, BatchId::from_bytes(batch_bytes));
    request
}

fn sealed_store(
    root: &Path,
    request: &PublicationRequest,
    snapshot: &PrepSnapshot,
) -> CheckpointFsStore {
    let snapshot_id = request.exec_input.prep_snapshot_id();
    assert_eq!(snapshot_id, request.exec_input.prep_snapshot_id());

    let mut snap_store = PrepFsStore::new(root);
    let saved_id = snap_store.save_snapshot(snapshot).expect("save snapshot");
    assert_eq!(saved_id, snapshot_id);

    let mut store = CheckpointFsStore::new(root);
    let exec_id = store
        .save_exec_input(&request.exec_input)
        .expect("save exec input");
    assert_eq!(exec_id, request.link.exec_input_id());
    let preview = preview_publication_contract(request, CheckpointDaProviderFamily::LocalArchive)
        .expect("preview");
    store
        .stage_publication_contract(exec_id, &preview.archive_manifest, &preview.da_reference)
        .expect("stage publication contract");

    let proof = request
        .draft
        .attest_proof(snapshot_id, exec_id)
        .expect("checkpoint proof");
    let link = store
        .seal_artifact(&request.draft, proof, snapshot_id, exec_id)
        .expect("seal artifact");
    assert_eq!(link.checkpoint_id(), request.link.checkpoint_id());
    assert_eq!(link.prep_snapshot_id(), request.link.prep_snapshot_id());
    assert_eq!(link.exec_input_id(), request.link.exec_input_id());
    store
}
