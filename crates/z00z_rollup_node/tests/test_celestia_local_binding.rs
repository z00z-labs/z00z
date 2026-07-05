#[path = "support/test_theorem_fixture.rs"]
mod theorem_fixture;

use z00z_aggregators::{AggregatorId, BatchRoute, SecondaryState, ShardPlacementView};
use z00z_rollup_node::{CelestiaLocalAdapter, DaAdapter, DaError};
use z00z_storage::checkpoint::CheckpointPubIn;
use z00z_storage::settlement::{ObjectPolicyRegistryV1, SettlementStateRoot};
use z00z_validators::{CheckpointFlow, RejectClass, ResolvedBatch, ValidatorBoundary, VerdictKind};

#[test]
fn test_celestia_roundtrip() {
    let request = theorem_fixture::publication_request([0x41; 32], "celestia-1");
    let mut adapter = CelestiaLocalAdapter::new("local-celestia");

    let published = adapter.publish(request.clone()).expect("publish");
    let resolved = adapter.resolve(&published).expect("resolve");

    assert!(published.blob_ref.starts_with("celestia-local://"));
    assert!(published.da_provider.starts_with("celestia-local/"));
    assert_eq!(published.subject_digest, Some(request.subject.digest()));
    assert_eq!(
        published.certificate_digest,
        Some(request.certificate.digest())
    );
    assert_eq!(published.theorem_digest, Some(resolved.theorem_digest()));
    assert_eq!(resolved.subject, Some(request.subject));
    assert_eq!(resolved.certificate, Some(request.certificate));
}

#[test]
fn test_celestia_namespace_drift() {
    let request = theorem_fixture::publication_request([0x42; 32], "celestia-2");
    let mut adapter = CelestiaLocalAdapter::new("local-celestia");
    let published = adapter.publish(request).expect("publish");
    assert!(adapter.forge_namespace(published.batch_id, "deadbeefdeadbeef"));

    let err = adapter
        .resolve(&published)
        .expect_err("wrong namespace must reject");

    assert_eq!(err, DaError::NamespaceMismatch);
}

#[test]
fn test_celestia_commitment_drift() {
    let request = theorem_fixture::publication_request([0x43; 32], "celestia-3");
    let mut adapter = CelestiaLocalAdapter::new("local-celestia");
    let published = adapter.publish(request).expect("publish");
    assert!(adapter.forge_blob_commitment(published.batch_id, [0xCC; 32]));

    let err = adapter
        .resolve(&published)
        .expect_err("wrong blob commitment must reject");

    assert_eq!(err, DaError::BlobCommitmentMismatch);
}

#[test]
fn test_celestia_missing_payload() {
    let request = theorem_fixture::publication_request([0x44; 32], "celestia-4");
    let mut adapter = CelestiaLocalAdapter::new("local-celestia");
    let published = adapter.publish(request).expect("publish");
    assert!(adapter.mark_payload_missing(published.batch_id));

    let err = adapter
        .resolve(&published)
        .expect_err("missing payload must reject during the challenge window");

    assert_eq!(err, DaError::MissingPayload);
}

#[test]
fn test_celestia_stale_anchor() {
    let request = theorem_fixture::publication_request([0x45; 32], "celestia-5");
    let mut adapter = CelestiaLocalAdapter::new("local-celestia");
    let published = adapter.publish(request).expect("publish");
    let blob_height = adapter
        .record(published.batch_id)
        .expect("record")
        .blob_height;
    assert!(adapter.forge_anchor_height(published.batch_id, Some(blob_height - 1)));

    let err = adapter
        .resolve(&published)
        .expect_err("stale anchor must reject");

    assert_eq!(err, DaError::StaleAnchor);
}

#[test]
fn test_celestia_cert_drift() {
    let request = theorem_fixture::publication_request([0x46; 32], "celestia-6");
    let mut adapter = CelestiaLocalAdapter::new("local-celestia");
    let published = adapter.publish(request).expect("publish");
    assert!(adapter.forge_certificate_digest(published.batch_id, [0xDD; 32]));

    let err = adapter
        .resolve(&published)
        .expect_err("mismatched certificate digest must reject");

    assert_eq!(err, DaError::CertificateMismatch);
}

#[test]
fn test_celestia_unanchored_limit() {
    let request = theorem_fixture::publication_request([0x47; 32], "celestia-7");
    let mut adapter = CelestiaLocalAdapter::new("local-celestia");
    let published = adapter.publish(request).expect("publish");
    let record = adapter.record(published.batch_id).expect("record").clone();
    assert!(adapter.clear_anchor(published.batch_id));
    adapter.set_current_height(record.blob_height + record.unanchored_limit + 1);

    let err = adapter
        .resolve(&published)
        .expect_err("unanchored height limit must reject");

    assert_eq!(err, DaError::UnanchoredHeightExceeded);
}

#[test]
fn test_celestia_validator_rejects() {
    let request = theorem_fixture::publication_request([0x48; 32], "celestia-8");
    let mut adapter = CelestiaLocalAdapter::new("local-celestia");
    let published = adapter.publish(request.clone()).expect("publish");
    let resolved = adapter.resolve(&published).expect("resolve");
    let resolved = resolved_with_placement(&request, resolved);

    let verdict =
        ValidatorBoundary.verdict_for_batch(&resolved, &ObjectPolicyRegistryV1::default());
    assert_eq!(verdict.kind, VerdictKind::Accepted);

    let mut drifted = resolved.clone();
    drifted.published.pub_in = tampered_pub_in(&drifted.published.pub_in);

    let err = CheckpointFlow::try_from_resolved(&drifted)
        .expect_err("validator must reject blob content drift");

    assert_eq!(err, RejectClass::StateRootMismatch);
}

fn resolved_with_placement(
    request: &z00z_aggregators::PublicationRequest,
    resolved: ResolvedBatch,
) -> ResolvedBatch {
    ResolvedBatch::new(
        resolved.published.clone(),
        request.ordered_batch.clone(),
        resolved.theorem.clone(),
        resolved.subject.clone(),
        resolved.certificate.clone(),
        resolved.nullifiers.clone(),
        Some(fixture_placement(request.ordered_batch.planned.route)),
        None,
    )
}

fn fixture_placement(route: BatchRoute) -> ShardPlacementView {
    ShardPlacementView {
        route,
        primary_id: AggregatorId::new(3),
        secondaries: vec![
            SecondaryState::ready(AggregatorId::new(4)),
            SecondaryState::ready(AggregatorId::new(5)),
        ],
        expected_journal_lineage: [0x62; 32],
    }
}

fn tampered_pub_in(pub_in: &CheckpointPubIn) -> CheckpointPubIn {
    let mut tampered = CheckpointPubIn::new_settlement(
        pub_in.prev_settlement_root(),
        SettlementStateRoot::settlement_v1([0xFA; 32]),
        pub_in.spent_delta().to_vec(),
        pub_in.created_delta().to_vec(),
    );
    if let Some(claim_root) = pub_in.claim_root() {
        tampered = tampered.with_claim_root(claim_root);
    }
    tampered
}
