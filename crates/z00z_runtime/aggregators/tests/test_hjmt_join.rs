use tempfile::tempdir;
use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, AggregatorId, BatchPlanner,
    BatchRoute, CommitSubject, ConsensusAdapter, JournalCandidate, OrderedBatch, RouteRangeRule,
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

#[path = "test_common.rs"]
mod test_common;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;

use test_common::{batch_id, tx_item, HASH_MAX, HASH_MIN};
use test_recovery_common::{recovery_record, route_bound_recovery_state};

#[path = "test_hjmt_topology_support.rs"]
mod hjmt_topology_support;

use hjmt_topology_support::{
    bind_previous_generation, canonical_five_by_seven, load_cfg, owner_join_six_by_seven,
    placement_row, primary_id, read_route_table, secondary_join_six_by_seven,
    set_activation_checkpoint, write_home,
};

#[test]
fn test_join_keeps_route() {
    let temp = tempdir().expect("tempdir");
    let old_home = temp.path().join("old_5a7s");
    let new_home = temp.path().join("new_6a7s_secondary");
    write_home(&old_home, 1, &canonical_five_by_seven(7700));
    write_home(&new_home, 1, &secondary_join_six_by_seven(7800));
    set_activation_checkpoint(&old_home, 11);
    set_activation_checkpoint(&new_home, 11);

    let old_cfg = load_cfg(&old_home);
    let new_cfg = load_cfg(&new_home);
    let old_table = read_route_table(&old_home);
    let new_table = read_route_table(&new_home);
    let new_row = placement_row(&new_cfg, 0, 1);

    assert_eq!(old_cfg.node_stat().expect("old stat").agg_count, 5);
    assert_eq!(new_cfg.node_stat().expect("new stat").agg_count, 6);
    assert_eq!(new_cfg.node_stat().expect("new stat").shard_count, 7);
    assert_eq!(primary_id(&old_cfg, 0, 1), AggregatorId::new(0));
    assert_eq!(new_row.primary_id, AggregatorId::new(0));
    assert!(new_row
        .secondaries
        .iter()
        .any(|secondary| secondary.aggregator_id == AggregatorId::new(5)));
    assert_eq!(old_table.canonical_bytes(), new_table.canonical_bytes());
}

#[test]
fn test_join_advances_generation() {
    let temp = tempdir().expect("tempdir");
    let old_home = temp.path().join("old_5a7s");
    let new_home = temp.path().join("new_6a7s_owner");
    write_home(&old_home, 1, &canonical_five_by_seven(7900));
    write_home(&new_home, 2, &owner_join_six_by_seven(8000));
    set_activation_checkpoint(&old_home, 11);
    bind_previous_generation(&new_home, &read_route_table(&old_home));
    set_activation_checkpoint(&new_home, 42);

    let old_cfg = load_cfg(&old_home);
    let new_cfg = load_cfg(&new_home);
    let old_table = read_route_table(&old_home);
    let new_table = read_route_table(&new_home);
    let new_row = placement_row(&new_cfg, 0, 2);

    new_table
        .validate_migration(&old_table)
        .expect("owner activation must bind to generation advance");

    assert_eq!(primary_id(&old_cfg, 0, 1), AggregatorId::new(0));
    assert_eq!(new_row.primary_id, AggregatorId::new(5));
    assert!(new_row
        .secondaries
        .iter()
        .any(|secondary| secondary.aggregator_id == AggregatorId::new(0)));
    assert_eq!(old_cfg.node_stat().expect("old stat").routing_generation, 1);
    assert_eq!(new_cfg.node_stat().expect("new stat").routing_generation, 2);
    assert_eq!(old_table.activation_checkpoint, 11);
    assert_eq!(new_table.activation_checkpoint, 42);
}

#[test]
fn owner_join_rejects_checkpoint_rollback() {
    let temp = tempdir().expect("tempdir");
    let old_home = temp.path().join("old_5a7s");
    let new_home = temp.path().join("new_6a7s_owner");
    write_home(&old_home, 1, &canonical_five_by_seven(8100));
    write_home(&new_home, 2, &owner_join_six_by_seven(8200));
    set_activation_checkpoint(&old_home, 11);
    bind_previous_generation(&new_home, &read_route_table(&old_home));
    set_activation_checkpoint(&new_home, 10);

    let old_table = read_route_table(&old_home);
    let new_table = read_route_table(&new_home);

    let err = new_table
        .validate_migration(&old_table)
        .expect_err("owner activation must reject before the activation checkpoint advances");

    assert_eq!(format!("{err:?}"), "BadPrevGen");
}

#[test]
fn pending_observer_blocks_voting() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = join_lifecycle_fixture()?;
    let pending_subject =
        subject_for_placement(17, &fixture.batch, &fixture.candidate, &fixture.placement)
            .expect("pending placement subject");

    let err = ConsensusAdapter::from_placement(&fixture.placement)
        .expect("pending placement adapter")
        .commit(
            &pending_subject,
            &[
                vote_for_subject(&pending_subject, fixture.primary, ShardVoteRole::Primary),
                vote_for_subject(
                    &pending_subject,
                    fixture.pending_secondary.aggregator_id,
                    ShardVoteRole::Secondary,
                ),
            ],
        )
        .expect_err("pending observer must not vote");
    assert!(err.detail.contains("inactive voter ids"));

    let ready_observer = SecondaryState::ready(fixture.pending_secondary.aggregator_id);
    let ready_placement = ShardPlacement::new(
        fixture.route,
        fixture.primary,
        vec![
            fixture.ready_secondaries[0],
            fixture.ready_secondaries[1],
            ready_observer,
        ],
        fixture.placement.expected_journal_lineage,
    );
    let ready_subject =
        subject_for_placement(18, &fixture.batch, &fixture.candidate, &ready_placement)
            .expect("ready placement subject");
    let commit = ConsensusAdapter::from_placement(&ready_placement)
        .expect("ready placement adapter")
        .commit(
            &ready_subject,
            &[
                vote_for_subject(&ready_subject, fixture.primary, ShardVoteRole::Primary),
                vote_for_subject(
                    &ready_subject,
                    fixture.ready_secondaries[0].aggregator_id,
                    ShardVoteRole::Secondary,
                ),
                vote_for_subject(
                    &ready_subject,
                    ready_observer.aggregator_id,
                    ShardVoteRole::Secondary,
                ),
            ],
        )
        .expect("ready observer must become vote-eligible");

    assert!(commit
        .certificate
        .votes
        .iter()
        .any(|vote| vote.voter_id == ready_observer.aggregator_id));

    Ok(())
}

#[test]
fn rotated_primary_rejects_old_role() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = join_lifecycle_fixture()?;
    let old_primary = fixture.primary;
    let new_primary = fixture.ready_secondaries[1].aggregator_id;
    let carried_secondary = fixture.ready_secondaries[0];
    let rotated_route = BatchRoute {
        shard_id: fixture.route.shard_id,
        routing_generation: fixture.route.routing_generation + 1,
    };
    let rotated_batch = planner(rotated_route)
        .make_batch(batch_id("rotated-primary"), &[tx_item("rotated-primary")])
        .expect("rotated batch");
    let rotated_recovery = route_bound_recovery_state(
        0x92,
        rotated_batch.batch_id,
        rotated_route,
        rotated_batch.planned.route_table_digest.into_bytes(),
    )?;
    let rotated_placement = ShardPlacement::new(
        rotated_route,
        new_primary,
        vec![SecondaryState::ready(old_primary), carried_secondary],
        rotated_recovery.journal_lineage,
    );
    let rotated_record = recovery_record(
        "rotated-primary",
        rotated_route,
        new_primary,
        rotated_placement.secondaries.clone(),
        rotated_recovery,
    );
    let rotated_candidate =
        JournalCandidate::from_record(&rotated_record).expect("rotated recovery candidate");
    let rotated_subject =
        subject_for_placement(19, &rotated_batch, &rotated_candidate, &rotated_placement)
            .expect("rotated subject");

    let err = ConsensusAdapter::from_placement(&rotated_placement)
        .expect("rotated placement adapter")
        .commit(
            &rotated_subject,
            &[
                vote_for_subject(&rotated_subject, old_primary, ShardVoteRole::Primary),
                vote_for_subject(&rotated_subject, new_primary, ShardVoteRole::Secondary),
                vote_for_subject(
                    &rotated_subject,
                    carried_secondary.aggregator_id,
                    ShardVoteRole::Secondary,
                ),
            ],
        )
        .expect_err("old primary must not keep the primary vote role after rotation");
    assert!(err.detail.contains("wrong voter role"));

    let commit = ConsensusAdapter::from_placement(&rotated_placement)
        .expect("rotated placement adapter")
        .commit(
            &rotated_subject,
            &[
                vote_for_subject(&rotated_subject, new_primary, ShardVoteRole::Primary),
                vote_for_subject(&rotated_subject, old_primary, ShardVoteRole::Secondary),
                vote_for_subject(
                    &rotated_subject,
                    carried_secondary.aggregator_id,
                    ShardVoteRole::Secondary,
                ),
            ],
        )
        .expect("lawful rotated committee must still commit");

    assert!(commit
        .certificate
        .votes
        .iter()
        .any(|vote| vote.voter_id == old_primary && vote.voter_role == ShardVoteRole::Secondary));

    Ok(())
}

struct JoinLifecycleFixture {
    route: BatchRoute,
    batch: OrderedBatch,
    candidate: JournalCandidate,
    placement: ShardPlacement,
    primary: AggregatorId,
    ready_secondaries: [SecondaryState; 2],
    pending_secondary: SecondaryState,
}

fn join_lifecycle_fixture() -> Result<JoinLifecycleFixture, Box<dyn std::error::Error>> {
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
        .make_batch(batch_id("join-lifecycle"), &[tx_item("join-lifecycle")])
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
        "join-lifecycle",
        route,
        primary,
        placement.secondaries.clone(),
        recovery,
    );
    let candidate = JournalCandidate::from_record(&record).expect("recovery candidate");

    Ok(JoinLifecycleFixture {
        route,
        batch,
        candidate,
        placement,
        primary,
        ready_secondaries,
        pending_secondary,
    })
}

fn subject_for_placement(
    term: u64,
    batch: &OrderedBatch,
    candidate: &JournalCandidate,
    placement: &ShardPlacement,
) -> Result<CommitSubject, z00z_aggregators::RejectRecord> {
    let publication = publication_binding(
        batch,
        SettlementStateRoot::settlement_v1([0x41; 32]),
        candidate.state_root,
    );
    CommitSubject::from_runtime(
        term,
        membership_digest_for_voters(
            placement.route,
            placement.primary_id,
            placement
                .secondaries
                .iter()
                .filter(|secondary| secondary.is_ready)
                .map(|secondary| secondary.aggregator_id),
        ),
        batch,
        candidate,
        &publication,
        publication.pub_in_digest(),
        None,
    )
}

fn vote_for_subject(
    subject: &CommitSubject,
    voter_id: AggregatorId,
    voter_role: ShardVoteRole,
) -> ShardVote {
    ShardVote::new_local(
        voter_id,
        voter_role,
        subject.shard_id,
        subject.term,
        subject.membership_digest,
        subject.digest(),
        ShardVoteKind::LocalCommit,
    )
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
