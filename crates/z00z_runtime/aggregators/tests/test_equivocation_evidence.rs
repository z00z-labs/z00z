#[path = "test_common.rs"]
mod test_common;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;

use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, AggregatorId, BatchPlanner,
    BatchRoute, CommitSubject, EquivocationEvidence, RouteRangeRule, SecondaryState, ShardId,
    ShardRouteTable, ShardVote, ShardVoteKind, ShardVoteRole, VoteEvidence, VoteEvidenceTracker,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, CheckpointDraft, CheckpointExecInputId, CheckpointVersion,
        CreatedEnt, SpentEnt,
    },
    settlement::{CheckRoot, SettlementStateRoot},
    snapshot::PrepSnapshotId,
};

use self::{
    test_common::{batch_id, tx_item, HASH_MAX, HASH_MIN},
    test_recovery_common::{recovery_record, route_bound_recovery_state},
};

#[test]
fn test_equivocation_evidence_emits_for_conflicting_votes() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = evidence_fixture()?;
    let first = ShardVote::new_local(
        fixture.secondary.aggregator_id,
        ShardVoteRole::Secondary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        fixture.subject.digest(),
        ShardVoteKind::LocalCommit,
    );
    let mut conflicting_subject = fixture.subject.clone();
    conflicting_subject.theorem_or_settlement_digest = [0x88; 32];
    let second = ShardVote::new_local(
        fixture.secondary.aggregator_id,
        ShardVoteRole::Secondary,
        conflicting_subject.shard_id,
        conflicting_subject.term,
        conflicting_subject.membership_digest,
        conflicting_subject.digest(),
        ShardVoteKind::LocalCommit,
    );

    let forward_evidence =
        EquivocationEvidence::new(first.clone(), second.clone()).expect("forward evidence");
    let reverse_evidence =
        EquivocationEvidence::new(second.clone(), first.clone()).expect("reverse evidence");
    assert_eq!(
        forward_evidence.evidence_digest,
        reverse_evidence.evidence_digest
    );

    let mut tracker = VoteEvidenceTracker::default();
    tracker.record_vote(first).expect("first vote accepted");
    let evidence = tracker
        .record_vote(second)
        .expect_err("conflicting vote must emit evidence");
    match evidence {
        VoteEvidence::Equivocation(evidence) => {
            assert_eq!(evidence.voter_id, fixture.secondary.aggregator_id);
            assert_eq!(evidence.first_vote.voter_id, evidence.second_vote.voter_id);
            assert_ne!(
                evidence.first_vote.subject_digest,
                evidence.second_vote.subject_digest
            );
            assert_eq!(tracker.records().len(), 1);
        }
        other => panic!("expected equivocation evidence, got {other:?}"),
    }

    Ok(())
}

#[test]
fn test_equivocation_evidence_is_idempotent_for_same_subject(
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = evidence_fixture()?;
    let vote = ShardVote::new_local(
        fixture.secondary.aggregator_id,
        ShardVoteRole::Secondary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        fixture.subject.digest(),
        ShardVoteKind::LocalCommit,
    );

    let mut tracker = VoteEvidenceTracker::default();
    tracker.record_vote(vote.clone()).expect("first vote");
    tracker.record_vote(vote).expect("same vote is idempotent");
    assert!(tracker.records().is_empty());

    Ok(())
}

struct EvidenceFixture {
    subject: CommitSubject,
    secondary: SecondaryState,
}

fn evidence_fixture() -> Result<EvidenceFixture, Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: ShardId::new(5),
        routing_generation: 12,
    };
    let primary = AggregatorId::new(21);
    let secondary = SecondaryState::ready(AggregatorId::new(22));
    let companion = SecondaryState::ready(AggregatorId::new(23));
    let batch = planner(route)
        .make_batch(batch_id("equivocation-fixture"), &[tx_item("equivocation")])
        .expect("planned batch");
    let recovery = route_bound_recovery_state(
        0x81,
        batch.batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let record = recovery_record(
        "equivocation-fixture",
        route,
        primary,
        vec![secondary, companion],
        recovery,
    );
    let candidate = z00z_aggregators::JournalCandidate::from_record(&record).expect("candidate");
    let publication = publication_binding(
        &batch,
        SettlementStateRoot::settlement_v1([0x11; 32]),
        candidate.state_root,
    );
    let subject = CommitSubject::from_runtime(
        17,
        membership_digest_for_voters(
            route,
            primary,
            [secondary.aggregator_id, companion.aggregator_id],
        ),
        &batch,
        &candidate,
        &publication,
        publication.pub_in_digest(),
        None,
    )
    .expect("commit subject");
    Ok(EvidenceFixture { subject, secondary })
}

fn publication_binding(
    batch: &z00z_aggregators::OrderedBatch,
    prev_root: SettlementStateRoot,
    new_root: SettlementStateRoot,
) -> z00z_aggregators::PublicationBinding {
    let draft = CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        52,
        CheckRoot::new(prev_root.into_bytes()),
        CheckRoot::new(new_root.into_bytes()),
        vec![SpentEnt::new([0x31; 32]), SpentEnt::new([0x32; 32])],
        vec![CreatedEnt::new([0x41; 32], [0x51; 32])],
    );
    let proof = draft
        .attest_proof(
            PrepSnapshotId::new([0x61; 32]),
            CheckpointExecInputId::new([0x71; 32]),
        )
        .expect("checkpoint proof");
    let artifact = draft.finalize(proof).expect("checkpoint artifact");
    let checkpoint_id = derive_checkpoint_id(&artifact).expect("checkpoint id");
    bind_publication_contract(
        batch.batch_id,
        checkpoint_id,
        batch.planned.route_table_digest.into_bytes(),
        &artifact.pub_in(),
    )
}

fn planner(route: BatchRoute) -> BatchPlanner {
    BatchPlanner::new(ShardRouteTable {
        routing_generation: route.routing_generation,
        shard_set: vec![route.shard_id],
        rules: vec![RouteRangeRule::new(HASH_MIN, HASH_MAX, route.shard_id)],
        previous_generation_digest: (route.routing_generation > 0)
            .then_some(ShardRouteTable::default().digest()),
        activation_checkpoint: 11,
    })
}
