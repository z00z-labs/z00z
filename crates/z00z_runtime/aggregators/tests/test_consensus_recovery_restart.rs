#[path = "test_common.rs"]
mod test_common;
mod test_recovery_common;

use tempfile::tempdir;
use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, persist_consensus_commit,
    persist_consensus_publication, persist_validator_decision, publication_record_for_published,
    validator_decision_snapshot, AggregatorId, BatchPlanner, BatchRoute, CommitSubject,
    ConsensusStore, JournalCandidate, PublicationState, RecoveryBoundary, RecoveryIntent,
    RouteRangeRule, SecondaryState, ShardQuorumCertificate, ShardRouteTable, ShardVote,
    ShardVoteKind, ShardVoteRole,
};
use z00z_storage::{
    checkpoint::{CheckpointId, CheckpointPubIn},
    settlement::{PublicationRouteSnapshotV1, SettlementRecoveryState, SettlementStateRoot},
};
use z00z_utils::io::save_json;

use self::test_common::{batch_id, tx_item, HASH_MAX, HASH_MIN};
use self::test_recovery_common::{placement_table, recovery_record, route_bound_recovery_state};

struct RestartFixture {
    route: BatchRoute,
    takeover: AggregatorId,
    batch_id: z00z_aggregators::BatchId,
    current: SettlementRecoveryState,
    live_table: z00z_aggregators::ShardPlacementTable,
    recovery_record: z00z_aggregators::ShardRecoveryRecord,
    subject: CommitSubject,
    votes: Vec<ShardVote>,
    certificate: ShardQuorumCertificate,
    publication_binding: z00z_aggregators::PublicationBinding,
    publication_record: z00z_aggregators::PublicationRecord,
    published: z00z_aggregators::PublishedBatch,
    validator_decision: z00z_aggregators::ConsensusValidatorDecision,
}

#[test]
fn test_restart_resumes_exact_certificate_before_publication(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fixture = fixture()?;
    let store = ConsensusStore::open(temp.path())?;

    persist_consensus_commit(
        &store,
        &fixture.recovery_record,
        &fixture.subject,
        &fixture.votes,
        &fixture.certificate,
    )
    .map_err(reject_to_io)?;

    let resumed = RecoveryBoundary
        .resume_from_store(
            fixture.takeover,
            &fixture.live_table,
            &fixture.current,
            &store,
            fixture.route,
            RecoveryIntent::TakeoverSecondary,
        )
        .map_err(reject_to_io)?;
    assert_eq!(resumed.ticket.batch_id, fixture.batch_id);
    assert_eq!(resumed.ticket.placement.primary_id, fixture.takeover);
    assert_eq!(resumed.record.header.digest(), fixture.subject.digest());
    assert_eq!(
        resumed.record.certificate.digest(),
        fixture.certificate.digest()
    );

    persist_consensus_publication(
        &store,
        fixture.batch_id,
        fixture.publication_record.clone(),
        &fixture.publication_binding,
        &fixture.published,
    )
    .map_err(reject_to_io)?;
    let after_publication = store.load_route(fixture.route)?;
    assert_eq!(
        after_publication.certificate.digest(),
        fixture.certificate.digest()
    );
    assert_eq!(
        after_publication
            .publication
            .as_ref()
            .expect("publication")
            .binding
            .binding_digest,
        fixture.publication_binding.binding_digest(),
    );

    Ok(())
}

#[test]
fn test_restart_recovers_publication_and_validator_binding(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fixture = fixture()?;
    let store = ConsensusStore::open(temp.path())?;

    persist_consensus_commit(
        &store,
        &fixture.recovery_record,
        &fixture.subject,
        &fixture.votes,
        &fixture.certificate,
    )
    .map_err(reject_to_io)?;
    persist_consensus_publication(
        &store,
        fixture.batch_id,
        fixture.publication_record.clone(),
        &fixture.publication_binding,
        &fixture.published,
    )
    .map_err(reject_to_io)?;
    persist_validator_decision(&store, fixture.batch_id, fixture.validator_decision.clone())
        .map_err(reject_to_io)?;

    let reopened = ConsensusStore::open(temp.path())?;
    let resumed = RecoveryBoundary
        .resume_from_store(
            fixture.takeover,
            &fixture.live_table,
            &fixture.current,
            &reopened,
            fixture.route,
            RecoveryIntent::TakeoverSecondary,
        )
        .map_err(reject_to_io)?;
    let validator = resumed
        .record
        .validator_decision
        .as_ref()
        .expect("validator decision");
    assert_eq!(validator.verdict_kind, "accepted");
    assert_eq!(
        validator.publication_binding_digest,
        Some(fixture.publication_binding.binding_digest())
    );
    assert_eq!(
        validator.checkpoint_id,
        Some(fixture.published.checkpoint_id)
    );

    Ok(())
}

#[test]
fn test_restart_fails_closed_for_partial_store_and_stale_root(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fixture = fixture()?;
    let store = ConsensusStore::open(temp.path())?;

    persist_consensus_commit(
        &store,
        &fixture.recovery_record,
        &fixture.subject,
        &fixture.votes,
        &fixture.certificate,
    )
    .map_err(reject_to_io)?;

    let mut partial = store.load_batch(fixture.batch_id)?;
    partial.votes.truncate(1);
    save_json(store.batch_path(fixture.batch_id), &partial)?;
    let err = RecoveryBoundary
        .resume_from_store(
            fixture.takeover,
            &fixture.live_table,
            &fixture.current,
            &store,
            fixture.route,
            RecoveryIntent::TakeoverSecondary,
        )
        .expect_err("partial vote material must reject");
    assert!(err
        .detail
        .contains("stored consensus vote material drifted"));

    persist_consensus_commit(
        &store,
        &fixture.recovery_record,
        &fixture.subject,
        &fixture.votes,
        &fixture.certificate,
    )
    .map_err(reject_to_io)?;

    let stale_current = SettlementRecoveryState::new(
        fixture.current.version,
        SettlementStateRoot::settlement_v1([0xaa; 32]),
        fixture.current.root_generation,
        fixture.current.proof_version,
        fixture.current.bucket_policy_generation,
        fixture.current.bucket_policy_id,
        fixture.current.journal_lineage,
    )
    .with_route(fixture.current.route.expect("route-bound current"));
    let err = RecoveryBoundary
        .resume_from_store(
            fixture.takeover,
            &fixture.live_table,
            &stale_current,
            &store,
            fixture.route,
            RecoveryIntent::TakeoverSecondary,
        )
        .expect_err("stale root must reject");
    assert!(err.detail.contains("stale local root"));

    Ok(())
}

fn fixture() -> Result<RestartFixture, Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: z00z_aggregators::ShardId::new(5),
        routing_generation: 12,
    };
    let primary = AggregatorId::new(21);
    let secondaries = vec![
        SecondaryState::ready(AggregatorId::new(22)),
        SecondaryState::ready(AggregatorId::new(23)),
    ];
    let takeover = secondaries[0].aggregator_id;
    let batch_id = batch_id("consensus-restart");
    let planner = planner(route);
    let item = tx_item("consensus-restart");
    let batch = planner
        .make_batch(batch_id, std::slice::from_ref(&item))
        .map_err(reject_to_io)?;
    let current = route_bound_recovery_state(
        0x92,
        batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let recovery_record = recovery_record(
        "consensus-restart",
        route,
        primary,
        secondaries.clone(),
        current.clone(),
    );
    let publication_route = PublicationRouteSnapshotV1::new(
        route.routing_generation,
        batch.planned.route_table_digest.into_bytes(),
        42,
        vec![route.shard_id.as_u32()],
    );
    let checkpoint_id = CheckpointId::new([0x55; 32]);
    let pub_in = CheckpointPubIn::new_settlement(
        SettlementStateRoot::settlement_v1([0x21; 32]),
        current.state_root,
        Vec::new(),
        Vec::new(),
    );
    let publication_binding = bind_publication_contract(
        batch_id,
        checkpoint_id,
        batch.planned.route_table_digest.into_bytes(),
        &pub_in,
    );
    let theorem_digest = [0x61; 32];
    let subject = CommitSubject::from_runtime(
        7,
        membership_digest_for_voters(
            route,
            primary,
            secondaries.iter().map(|secondary| secondary.aggregator_id),
        ),
        &batch,
        &JournalCandidate::from_record(&recovery_record).map_err(reject_to_io)?,
        &publication_binding,
        theorem_digest,
        None,
    )
    .map_err(reject_to_io)?;
    let votes = vec![
        ShardVote::new_local(
            primary,
            ShardVoteRole::Primary,
            subject.shard_id,
            subject.term,
            subject.membership_digest,
            subject.digest(),
            ShardVoteKind::LocalCommit,
        ),
        ShardVote::new_local(
            takeover,
            ShardVoteRole::Secondary,
            subject.shard_id,
            subject.term,
            subject.membership_digest,
            subject.digest(),
            ShardVoteKind::LocalCommit,
        ),
    ];
    let certificate = ShardQuorumCertificate::new(
        &subject,
        primary,
        secondaries.iter().map(|secondary| secondary.aggregator_id),
        &votes,
    )
    .map_err(reject_to_io)?;
    let published = z00z_aggregators::PublishedBatch {
        batch_id,
        checkpoint_id,
        publication_checkpoint: 42,
        publication_route,
        pub_in,
        subject_digest: Some(subject.digest()),
        certificate_digest: Some(certificate.digest()),
        theorem_digest: Some(theorem_digest),
        da_provider: "scenario11-local".to_string(),
        blob_ref: "blob://consensus-restart".to_string(),
    };
    let publication_record = publication_record_for_published(&published, PublicationState::Posted);
    let validator_decision = validator_decision_snapshot(
        "accepted",
        None,
        batch_id,
        &subject,
        &certificate,
        theorem_digest,
        Some(checkpoint_id),
        Some(&publication_binding),
    );
    let live_table = placement_table(route, primary, secondaries, current.journal_lineage);

    Ok(RestartFixture {
        route,
        takeover,
        batch_id,
        current,
        live_table,
        recovery_record,
        subject,
        votes,
        certificate,
        publication_binding,
        publication_record,
        published,
        validator_decision,
    })
}

fn planner(route: BatchRoute) -> BatchPlanner {
    BatchPlanner::new(ShardRouteTable {
        routing_generation: route.routing_generation,
        shard_set: vec![route.shard_id],
        rules: vec![RouteRangeRule::new(HASH_MIN, HASH_MAX, route.shard_id)],
        previous_generation_digest: (route.routing_generation != 0)
            .then_some(z00z_aggregators::PlanDigest::new([0x11; 32])),
        activation_checkpoint: 21,
    })
}

fn reject_to_io(err: z00z_aggregators::RejectRecord) -> std::io::Error {
    std::io::Error::other(err.detail)
}
