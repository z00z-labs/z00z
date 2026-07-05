#[path = "support/test_theorem_fixture.rs"]
mod theorem_fixture;

use z00z_aggregators::{BatchId, PublicationRequest};
use z00z_rollup_node::{DaAdapter, DaError, LocalDaAdapter, LocalResolveState};

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

fn request_fixture(batch_bytes: [u8; 32], replay_id: &str) -> PublicationRequest {
    let request = theorem_fixture::publication_request(batch_bytes, replay_id);
    assert_eq!(request.batch_id, BatchId::from_bytes(batch_bytes));
    request
}
