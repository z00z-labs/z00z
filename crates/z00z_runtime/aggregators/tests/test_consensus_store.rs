#[path = "test_common.rs"]
mod test_common;
mod test_recovery_common;

use tempfile::tempdir;
use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, persist_consensus_commit,
    persist_consensus_publication, persist_validator_decision, publication_record_for_published,
    validator_decision_snapshot, AggregatorId, BatchPlanner, BatchRoute, CommitSubject,
    ConsensusStore, JournalCandidate, PublicationState, RouteRangeRule, SecondaryState,
    ShardQuorumCertificate, ShardRouteTable, ShardVote, ShardVoteKind, ShardVoteRole,
};
use z00z_storage::{
    checkpoint::{CheckpointId, CheckpointPubIn},
    settlement::{PublicationRouteSnapshotV1, SettlementStateRoot},
};
use z00z_utils::io::save_json;

use self::test_common::{batch_id, tx_item, HASH_MAX, HASH_MIN};
use self::test_recovery_common::{recovery_record, route_bound_recovery_state};

struct ConsensusFixture {
    route: BatchRoute,
    batch_id: z00z_aggregators::BatchId,
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
fn test_consensus_store_round_trip_and_canonical_paths() -> Result<(), Box<dyn std::error::Error>> {
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

    assert!(store.batch_path(fixture.batch_id).exists());
    assert!(store.route_path(fixture.route).exists());

    let reopened = ConsensusStore::open(temp.path())?;
    let loaded = reopened.load_route(fixture.route)?;
    assert_eq!(loaded.batch_id, fixture.batch_id);
    assert_eq!(loaded.header.digest(), fixture.subject.digest());
    assert_eq!(loaded.certificate.digest(), fixture.certificate.digest());
    assert_eq!(loaded.votes.len(), fixture.votes.len());
    assert_eq!(
        loaded
            .publication
            .as_ref()
            .expect("publication")
            .binding
            .binding_digest,
        fixture.publication_binding.binding_digest(),
    );
    assert_eq!(
        loaded
            .validator_decision
            .as_ref()
            .expect("validator decision")
            .verdict_kind,
        "accepted"
    );

    Ok(())
}

#[test]
fn test_consensus_store_rejects_subject_vote_and_anchor_drift(
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

    let mut wrong_header = store.load_batch(fixture.batch_id)?;
    wrong_header.header.batch_id = batch_id("drifted-header");
    save_json(store.batch_path(fixture.batch_id), &wrong_header)?;
    let err = store
        .load_route(fixture.route)
        .expect_err("wrong header must reject");
    assert!(err
        .to_string()
        .contains("stored consensus header batch id drifted"));

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

    let mut missing_vote = store.load_batch(fixture.batch_id)?;
    missing_vote.votes.truncate(1);
    save_json(store.batch_path(fixture.batch_id), &missing_vote)?;
    let err = store
        .load_route(fixture.route)
        .expect_err("missing vote must reject");
    assert!(err
        .to_string()
        .contains("stored consensus vote material drifted"));

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

    let mut wrong_anchor = store.load_batch(fixture.batch_id)?;
    wrong_anchor
        .publication
        .as_mut()
        .expect("publication")
        .published
        .certificate_digest = Some([0x99; 32]);
    save_json(store.batch_path(fixture.batch_id), &wrong_anchor)?;
    let err = store
        .load_route(fixture.route)
        .expect_err("anchor drift must reject");
    assert!(err
        .to_string()
        .contains("publication anchor is missing the persisted certificate digest"));

    Ok(())
}

#[test]
fn test_consensus_store_rejects_corrupt_json() -> Result<(), Box<dyn std::error::Error>> {
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
    std::fs::write(store.batch_path(fixture.batch_id), b"{not-json")?;

    let err = store
        .load_route(fixture.route)
        .expect_err("corrupt store must reject");
    assert!(err
        .to_string()
        .contains("failed to load durable consensus batch record"));

    Ok(())
}

fn fixture() -> Result<ConsensusFixture, Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: z00z_aggregators::ShardId::new(5),
        routing_generation: 12,
    };
    let primary = AggregatorId::new(21);
    let secondaries = vec![
        SecondaryState::ready(AggregatorId::new(22)),
        SecondaryState::ready(AggregatorId::new(23)),
    ];
    let batch_id = batch_id("consensus-store");
    let planner = planner(route);
    let item = tx_item("consensus-store");
    let batch = planner
        .make_batch(batch_id, std::slice::from_ref(&item))
        .map_err(reject_to_io)?;
    let recovery = route_bound_recovery_state(
        0x91,
        batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let recovery_record = recovery_record(
        "consensus-store",
        route,
        primary,
        secondaries.clone(),
        recovery.clone(),
    );
    let publication_route = PublicationRouteSnapshotV1::new(
        route.routing_generation,
        batch.planned.route_table_digest.into_bytes(),
        42,
        vec![route.shard_id.as_u32()],
    );
    let checkpoint_id = CheckpointId::new([0x44; 32]);
    let pub_in = CheckpointPubIn::new_settlement(
        SettlementStateRoot::settlement_v1([0x11; 32]),
        recovery.state_root,
        Vec::new(),
        Vec::new(),
    );
    let publication_binding = bind_publication_contract(
        batch_id,
        checkpoint_id,
        batch.planned.route_table_digest.into_bytes(),
        &pub_in,
    );
    let theorem_digest = [0x33; 32];
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
            secondaries[0].aggregator_id,
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
        blob_ref: "blob://consensus-store".to_string(),
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

    Ok(ConsensusFixture {
        route,
        batch_id,
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
