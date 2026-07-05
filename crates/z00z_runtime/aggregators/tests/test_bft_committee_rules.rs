#[path = "test_common.rs"]
mod test_common;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;

use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, AggregatorId, BatchPlanner,
    BatchRoute, BftCommittee, BftEngine, BftThresholds, CommitSubject, JournalCandidate,
    OrderedBatch, QuorumRule, RouteRangeRule, SecondaryState, ShardId, ShardPlacement,
    ShardRouteTable, ShardVote, ShardVoteKind, ShardVoteRole,
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

struct BftFixture {
    route: BatchRoute,
    batch: OrderedBatch,
    candidate: JournalCandidate,
    placement: ShardPlacement,
}

#[test]
fn test_bft_7_10_13() {
    let seven = BftThresholds::new(7).expect("7 members are BFT-valid");
    let ten = BftThresholds::new(10).expect("10 members are BFT-valid");
    let thirteen = BftThresholds::new(13).expect("13 members are BFT-valid");

    assert_eq!(seven.max_faulty, 2);
    assert_eq!(seven.quorum_threshold, 5);
    assert_eq!(ten.max_faulty, 3);
    assert_eq!(ten.quorum_threshold, 7);
    assert_eq!(thirteen.max_faulty, 4);
    assert_eq!(thirteen.quorum_threshold, 9);
}

#[test]
fn test_bft_bad_counts() {
    let too_small = BftThresholds::new(3).expect_err("3 members are only CFT-valid");
    assert!(too_small.detail.contains("3f+1"));

    let non_exact = BftThresholds::new(5).expect_err("5 members are not one exact 3f+1 set");
    assert!(non_exact.detail.contains("3f+1"));
}

#[test]
fn test_committee_ready_set() -> Result<(), Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: ShardId::new(4),
        routing_generation: 19,
    };
    let placement = ShardPlacement::new(
        route,
        AggregatorId::new(20),
        vec![
            SecondaryState::ready(AggregatorId::new(21)),
            SecondaryState::ready(AggregatorId::new(22)),
            SecondaryState::ready(AggregatorId::new(23)),
            SecondaryState::ready(AggregatorId::new(24)),
            SecondaryState::ready(AggregatorId::new(25)),
            SecondaryState::ready(AggregatorId::new(26)),
            SecondaryState::pending(AggregatorId::new(27)),
        ],
        [0x11; 32],
    );

    let committee = BftCommittee::from_placement(&placement).map_err(box_reject_record)?;

    assert_eq!(committee.member_count(), 7);
    assert_eq!(committee.max_faulty(), 2);
    assert_eq!(committee.quorum_threshold(), 5);
    assert_eq!(
        committee.membership_digest(route),
        membership_digest_for_voters(
            route,
            placement.primary_id,
            placement
                .secondaries
                .iter()
                .filter(|secondary| secondary.is_ready)
                .map(|secondary| secondary.aggregator_id),
        )
    );

    Ok(())
}

#[test]
fn test_engine_accepts_5_votes() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = bft_fixture(7)?;
    let subject = subject_for_members(29, &fixture).map_err(box_reject_record)?;
    let votes = votes_for_subject(
        &subject,
        &[
            (AggregatorId::new(40), ShardVoteRole::Primary),
            (AggregatorId::new(41), ShardVoteRole::Secondary),
            (AggregatorId::new(42), ShardVoteRole::Secondary),
            (AggregatorId::new(43), ShardVoteRole::Secondary),
            (AggregatorId::new(44), ShardVoteRole::Secondary),
        ],
    );
    let mut engine = BftEngine::from_placement(&fixture.placement).map_err(box_reject_record)?;

    let commit = engine
        .commit(&subject, &votes)
        .expect("2f+1 votes must commit");

    assert_eq!(engine.committee().member_count(), 7);
    assert_eq!(engine.committee().quorum_threshold(), 5);
    assert_eq!(commit.subject, subject);
    assert_eq!(commit.certificate.quorum_rule, QuorumRule::BftTwoFPlusOne);
    assert_eq!(commit.certificate.votes.len(), 5);
    assert_eq!(commit.certificate.votes[0].voter_id, AggregatorId::new(40));
    assert_eq!(commit.certificate.votes[4].voter_id, AggregatorId::new(44));

    Ok(())
}

#[test]
fn test_engine_rejects_4_votes() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = bft_fixture(7)?;
    let subject = subject_for_members(31, &fixture).map_err(box_reject_record)?;
    let votes = votes_for_subject(
        &subject,
        &[
            (AggregatorId::new(40), ShardVoteRole::Primary),
            (AggregatorId::new(41), ShardVoteRole::Secondary),
            (AggregatorId::new(42), ShardVoteRole::Secondary),
            (AggregatorId::new(43), ShardVoteRole::Secondary),
        ],
    );
    let mut engine = BftEngine::from_placement(&fixture.placement).map_err(box_reject_record)?;

    let err = engine
        .commit(&subject, &votes)
        .expect_err("4 votes are below 2f+1 for 7 members");

    assert!(err.detail.contains("2f+1"));

    Ok(())
}

#[test]
fn test_engine_rejects_cft() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = bft_fixture(3)?;
    let err = BftEngine::from_placement(&fixture.placement)
        .expect_err("3-member committee must stay on the CFT path");

    assert!(err.detail.contains("3f+1"));

    Ok(())
}

fn bft_fixture(member_count: usize) -> Result<BftFixture, Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: ShardId::new(6),
        routing_generation: 21,
    };
    let batch = planner(route)
        .make_batch(batch_id("bft-committee"), &[tx_item("bft-committee")])
        .expect("planned batch");
    let recovery = route_bound_recovery_state(
        0xA1,
        batch.batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let secondaries = (1..member_count)
        .map(|index| SecondaryState::ready(AggregatorId::new(40 + index as u16)))
        .collect::<Vec<_>>();
    let placement = ShardPlacement::new(
        route,
        AggregatorId::new(40),
        secondaries,
        recovery.journal_lineage,
    );
    let record = recovery_record(
        "bft-committee",
        route,
        placement.primary_id,
        placement.secondaries.clone(),
        recovery,
    );
    let candidate = JournalCandidate::from_record(&record).expect("recovery candidate");
    Ok(BftFixture {
        route,
        batch,
        candidate,
        placement,
    })
}

fn subject_for_members(
    term: u64,
    fixture: &BftFixture,
) -> Result<CommitSubject, z00z_aggregators::RejectRecord> {
    let publication = publication_binding(
        &fixture.batch,
        SettlementStateRoot::settlement_v1([0x51; 32]),
        fixture.candidate.state_root,
    );
    CommitSubject::from_runtime(
        term,
        membership_digest_for_voters(
            fixture.route,
            fixture.placement.primary_id,
            fixture
                .placement
                .secondaries
                .iter()
                .filter(|secondary| secondary.is_ready)
                .map(|secondary| secondary.aggregator_id),
        ),
        &fixture.batch,
        &fixture.candidate,
        &publication,
        publication.pub_in_digest(),
        None,
    )
}

fn votes_for_subject(
    subject: &CommitSubject,
    voters: &[(AggregatorId, ShardVoteRole)],
) -> Vec<ShardVote> {
    let subject_digest = subject.digest();
    voters
        .iter()
        .map(|(voter_id, voter_role)| {
            ShardVote::new_local(
                *voter_id,
                *voter_role,
                subject.shard_id,
                subject.term,
                subject.membership_digest,
                subject_digest,
                ShardVoteKind::LocalCommit,
            )
        })
        .collect()
}

fn publication_binding(
    batch: &OrderedBatch,
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

fn box_reject_record(reject: z00z_aggregators::RejectRecord) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::other(reject.detail))
}
