#[path = "support/test_theorem_fixture.rs"]
mod theorem_fixture;

use z00z_aggregators::{AggregatorId, ShardQuorumCertificate};
use z00z_rollup_node::{DaAdapter, DaError, LocalDaAdapter};

#[test]
fn local_da_preserves_live_quorum_binding() {
    let request = theorem_fixture::publication_request([0x31; 32], "qc-bind-1");
    let mut adapter = LocalDaAdapter::new("local-quorum");

    let published = adapter.publish(request.clone()).expect("publish");
    let resolved = adapter.resolve(&published).expect("resolve");

    assert_eq!(published.subject_digest, Some(request.subject.digest()));
    assert_eq!(
        published.certificate_digest,
        Some(request.certificate.digest())
    );
    assert_eq!(published.theorem_digest, Some(resolved.theorem_digest()));
    assert_eq!(resolved.ordered, request.ordered_batch);
    assert_eq!(resolved.subject, Some(request.subject));
    assert_eq!(resolved.certificate, Some(request.certificate));
}

#[test]
fn majority_certificates_intersect_exhaustively() {
    let request = theorem_fixture::publication_request([0x32; 32], "qc-bind-2");
    let votes = theorem_fixture::quorum_votes(&request.subject);
    let member_pairs = [[0usize, 1usize], [0, 2], [1, 2]];
    let certificates = member_pairs
        .iter()
        .map(|pair| {
            ShardQuorumCertificate::new(
                &request.subject,
                AggregatorId::new(3),
                [AggregatorId::new(4), AggregatorId::new(5)],
                &[votes[pair[0]].clone(), votes[pair[1]].clone()],
            )
            .expect("quorum certificate")
        })
        .collect::<Vec<_>>();

    for (index, left) in certificates.iter().enumerate() {
        for right in certificates.iter().skip(index + 1) {
            let intersects = left.votes.iter().any(|left_vote| {
                right
                    .votes
                    .iter()
                    .any(|right_vote| right_vote.voter_id == left_vote.voter_id)
            });
            assert!(intersects, "majority vote sets must intersect");
        }
    }
}

#[test]
fn local_da_rejects_detached_certificate_digest() {
    let mut adapter = LocalDaAdapter::new("local-quorum");
    let published = adapter
        .publish(theorem_fixture::publication_request(
            [0x33; 32],
            "qc-bind-3",
        ))
        .expect("publish");
    assert!(adapter.forge_certificate_digest(published.batch_id, [0xCD; 32]));

    let err = adapter
        .resolve(&published)
        .expect_err("detached certificate digest must reject");

    assert_eq!(err, DaError::MetadataMismatch);
}
