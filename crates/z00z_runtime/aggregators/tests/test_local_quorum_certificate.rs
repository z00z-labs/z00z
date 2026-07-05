#[path = "test_common.rs"]
mod test_common;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;

use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, AggregatorId, BatchPlanner,
    BatchRoute, CommitSubject, ConsensusAdapter, MembershipChange, OrderedBatch, RouteRangeRule,
    SecondaryState, ShardId, ShardPlacement, ShardRouteTable, ShardVote, ShardVoteKind,
    ShardVoteRole,
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

struct LocalCommitFixture {
    route: BatchRoute,
    batch: OrderedBatch,
    candidate: z00z_aggregators::JournalCandidate,
    placement: ShardPlacement,
    primary: AggregatorId,
    ready_secondaries: [SecondaryState; 2],
    pending_secondary: SecondaryState,
}

#[test]
fn test_local_commit_returns_live_certificate() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = local_commit_fixture()?;
    let mut adapter =
        ConsensusAdapter::from_placement(&fixture.placement).expect("consensus adapter");
    let subject = subject_for_members(17, &fixture, &fixture.ready_secondaries)
        .expect("subject must bind ready-membership route");
    let votes = votes_for_subject(
        &subject,
        &[
            (
                fixture.ready_secondaries[1].aggregator_id,
                ShardVoteRole::Secondary,
            ),
            (fixture.primary, ShardVoteRole::Primary),
        ],
    );

    let commit = adapter
        .commit(&subject, &votes)
        .expect("ready local quorum must commit");

    assert_eq!(commit.term, subject.term);
    assert_eq!(commit.batch_id, subject.batch_id);
    assert_eq!(commit.route, fixture.route);
    assert_eq!(commit.subject, subject);
    assert_eq!(commit.state_root, subject.new_state_root);
    assert_eq!(commit.journal_lineage, fixture.candidate.journal_lineage);
    assert_eq!(commit.certificate.shard_id, fixture.route.shard_id);
    assert_eq!(
        commit.certificate.routing_generation,
        fixture.route.routing_generation
    );
    assert_eq!(
        commit.certificate.route_table_digest,
        fixture.batch.planned.route_table_digest.into_bytes()
    );
    assert_eq!(
        commit.certificate.membership_digest,
        adapter.membership_digest()
    );
    assert_eq!(commit.certificate.subject_digest, commit.subject.digest());
    assert_eq!(commit.certificate.votes.len(), 2);
    assert_eq!(
        commit.certificate.votes[0].voter_id, fixture.primary,
        "votes must be canonically sorted by voter id",
    );
    assert_eq!(
        commit.certificate.votes[1].voter_id,
        fixture.ready_secondaries[1].aggregator_id
    );
    assert_eq!(adapter.committed(), Some(&commit));

    Ok(())
}

#[test]
fn test_local_commit_rejects_unready_and_removed_voters() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = local_commit_fixture()?;
    let mut adapter =
        ConsensusAdapter::from_placement(&fixture.placement).expect("consensus adapter");
    let subject = subject_for_members(17, &fixture, &fixture.ready_secondaries)
        .expect("subject must bind ready-membership route");
    let unready_votes = votes_for_subject(
        &subject,
        &[
            (fixture.primary, ShardVoteRole::Primary),
            (
                fixture.pending_secondary.aggregator_id,
                ShardVoteRole::Secondary,
            ),
        ],
    );
    let err = adapter
        .commit(&subject, &unready_votes)
        .expect_err("pending secondary must not count toward local quorum");
    assert!(err.detail.contains("inactive voter ids"));

    adapter
        .apply_change(
            MembershipChange::Leave,
            fixture.ready_secondaries[1].aggregator_id,
            fixture.route.routing_generation,
        )
        .expect("leave must update active membership");

    let subject_after_leave = subject_for_members(18, &fixture, &fixture.ready_secondaries[..1])
        .expect("subject must rebind after member leave");
    let removed_votes = votes_for_subject(
        &subject_after_leave,
        &[
            (fixture.primary, ShardVoteRole::Primary),
            (
                fixture.ready_secondaries[1].aggregator_id,
                ShardVoteRole::Secondary,
            ),
        ],
    );
    let err = adapter
        .commit(&subject_after_leave, &removed_votes)
        .expect_err("removed secondary must reject before commit");
    assert!(err.detail.contains("inactive voter ids"));

    Ok(())
}

#[test]
fn test_local_commit_rejects_duplicate_and_mixed_votes() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = local_commit_fixture()?;
    let subject = subject_for_members(17, &fixture, &fixture.ready_secondaries)
        .expect("subject must bind ready-membership route");
    let primary_vote = ShardVote::new_local(
        fixture.primary,
        ShardVoteRole::Primary,
        subject.shard_id,
        subject.term,
        subject.membership_digest,
        subject.digest(),
        ShardVoteKind::LocalCommit,
    );
    let secondary_vote = ShardVote::new_local(
        fixture.ready_secondaries[0].aggregator_id,
        ShardVoteRole::Secondary,
        subject.shard_id,
        subject.term,
        subject.membership_digest,
        subject.digest(),
        ShardVoteKind::LocalCommit,
    );

    let duplicate_err = commit_err(&fixture, &subject, &[primary_vote.clone(), primary_vote])?;
    assert!(duplicate_err.detail.contains("duplicate voter ids"));

    let below_quorum_err = commit_err(&fixture, &subject, &[secondary_vote.clone()])?;
    assert!(below_quorum_err.detail.contains("below quorum"));

    let mixed_term = ShardVote::new_local(
        fixture.ready_secondaries[1].aggregator_id,
        ShardVoteRole::Secondary,
        subject.shard_id,
        subject.term + 1,
        subject.membership_digest,
        subject.digest(),
        ShardVoteKind::LocalCommit,
    );
    let mixed_term_err = commit_err(&fixture, &subject, &[secondary_vote.clone(), mixed_term])?;
    assert!(mixed_term_err.detail.contains("mixed terms"));

    let mixed_membership = ShardVote::new_local(
        fixture.ready_secondaries[1].aggregator_id,
        ShardVoteRole::Secondary,
        subject.shard_id,
        subject.term,
        membership_digest_for_voters(
            fixture.route,
            fixture.primary,
            [fixture.ready_secondaries[1].aggregator_id],
        ),
        subject.digest(),
        ShardVoteKind::LocalCommit,
    );
    let mixed_membership_err = commit_err(&fixture, &subject, &[secondary_vote, mixed_membership])?;
    assert!(mixed_membership_err
        .detail
        .contains("mixed membership digests"));

    Ok(())
}

fn local_commit_fixture() -> Result<LocalCommitFixture, Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: ShardId::new(6),
        routing_generation: 12,
    };
    let primary = AggregatorId::new(31);
    let ready_secondaries = [
        SecondaryState::ready(AggregatorId::new(32)),
        SecondaryState::ready(AggregatorId::new(33)),
    ];
    let pending_secondary = SecondaryState::pending(AggregatorId::new(34));
    let batch = planner(route)
        .make_batch(batch_id("local-quorum-certificate"), &[tx_item("local-qc")])
        .expect("planned batch");
    let recovery = route_bound_recovery_state(
        0x91,
        batch.batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let placement = ShardPlacement::new(
        route,
        primary,
        vec![
            ready_secondaries[0],
            ready_secondaries[1],
            pending_secondary,
        ],
        recovery.journal_lineage,
    );
    let record = recovery_record(
        "local-quorum-certificate",
        route,
        primary,
        placement.secondaries.clone(),
        recovery,
    );
    let candidate =
        z00z_aggregators::JournalCandidate::from_record(&record).expect("recovery candidate");
    Ok(LocalCommitFixture {
        route,
        batch,
        candidate,
        placement,
        primary,
        ready_secondaries,
        pending_secondary,
    })
}

fn commit_err(
    fixture: &LocalCommitFixture,
    subject: &CommitSubject,
    votes: &[ShardVote],
) -> Result<z00z_aggregators::RejectRecord, Box<dyn std::error::Error>> {
    let mut adapter =
        ConsensusAdapter::from_placement(&fixture.placement).expect("consensus adapter");
    Ok(adapter
        .commit(subject, votes)
        .expect_err("vote drift must reject before commit"))
}

fn subject_for_members(
    term: u64,
    fixture: &LocalCommitFixture,
    active_secondaries: &[SecondaryState],
) -> Result<CommitSubject, z00z_aggregators::RejectRecord> {
    let publication = publication_binding(
        &fixture.batch,
        SettlementStateRoot::settlement_v1([0x41; 32]),
        fixture.candidate.state_root,
    );
    CommitSubject::from_runtime(
        term,
        membership_digest_for_voters(
            fixture.route,
            fixture.primary,
            active_secondaries
                .iter()
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
