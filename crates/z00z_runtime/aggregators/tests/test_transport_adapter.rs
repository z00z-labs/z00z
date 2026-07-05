#[path = "test_common.rs"]
mod test_common;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;

use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, AggregatorId, BatchPlanner,
    BatchRoute, CommitSubject, InMemoryVoteTransport, ReplayVerifiedVoteService, RouteRangeRule,
    SecondaryReplayRequest, SecondaryState, ShardId, ShardPlacement, ShardPlacementTable,
    ShardRouteTable, ShardVoteKind, ShardVoteRole, TransportPayloadStatus, VoteEvidence,
    VoteExchangeContext, VoteExchangeOutcome, VoteTransport, VoteTransportEnvelope, WorkItem,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, CheckpointDraft, CheckpointExecInputId, CheckpointVersion,
        CreatedEnt, SpentEnt,
    },
    settlement::{CheckRoot, SettlementRecoveryState, SettlementStateRoot},
    snapshot::PrepSnapshotId,
};

use self::{
    test_common::{batch_id, tx_item, HASH_MAX, HASH_MIN},
    test_recovery_common::{recovery_record, route_bound_recovery_state},
};

#[test]
fn test_in_memory_transport_preserves_identity_delay_and_requeue(
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = transport_fixture()?;
    let mut transport = InMemoryVoteTransport::default();
    let delayed = VoteTransportEnvelope::available(
        fixture.primary,
        fixture.ready_secondaries[0].aggregator_id,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
    );
    let immediate = VoteTransportEnvelope::available(
        fixture.primary,
        fixture.ready_secondaries[1].aggregator_id,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
    );
    transport.enqueue_delayed(delayed.clone(), 2);
    transport.enqueue_front(immediate.clone());

    let first = transport.step();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].message_id, immediate.message_id);

    transport.requeue(immediate.clone(), 2);
    let second = transport.step();
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].message_id, delayed.message_id);

    let third = transport.step();
    assert_eq!(third.len(), 1);
    assert_eq!(third[0].message_id, immediate.message_id);

    Ok(())
}

#[test]
fn test_transport_delivery_requires_replay_verification() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = transport_fixture()?;
    let mut service = ReplayVerifiedVoteService::local();
    let envelope = VoteTransportEnvelope::available(
        fixture.primary,
        fixture.ready_secondaries[0].aggregator_id,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
    );
    let accepted = service.process_envelope(
        &envelope,
        vote_exchange_context(
            &fixture,
            fixture.ready_secondaries[0].aggregator_id,
            &fixture.subject,
        ),
    );
    match accepted.outcome {
        VoteExchangeOutcome::Vote(vote) => {
            assert!(vote.has_valid_signature());
            assert_eq!(vote.voter_id, fixture.ready_secondaries[0].aggregator_id);
        }
        other => panic!("expected vote, got {other:?}"),
    }

    let duplicate = service.process_envelope(
        &envelope,
        vote_exchange_context(
            &fixture,
            fixture.ready_secondaries[0].aggregator_id,
            &fixture.subject,
        ),
    );
    assert_eq!(duplicate.outcome, VoteExchangeOutcome::DuplicateMessage);

    let mut drifted_subject = fixture.subject.clone();
    drifted_subject.theorem_or_settlement_digest = [0x77; 32];
    let drifted_envelope = VoteTransportEnvelope::available(
        fixture.primary,
        fixture.ready_secondaries[1].aggregator_id,
        drifted_subject,
        ShardVoteKind::LocalCommit,
    );
    let rejected = service.process_envelope(
        &drifted_envelope,
        vote_exchange_context(
            &fixture,
            fixture.ready_secondaries[1].aggregator_id,
            &fixture.subject,
        ),
    );
    match rejected.outcome {
        VoteExchangeOutcome::ReplayRejected(reject) => {
            assert!(matches!(
                reject.code,
                z00z_aggregators::SecondaryReplayRejectCode::WrongTheoremDigest
            ));
        }
        other => panic!("expected replay rejection, got {other:?}"),
    }

    Ok(())
}

#[test]
fn test_transport_missing_payload_emits_evidence_without_vote(
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = transport_fixture()?;
    let mut service = ReplayVerifiedVoteService::local();
    let envelope = VoteTransportEnvelope::missing_payload(
        fixture.primary,
        fixture.ready_secondaries[0].aggregator_id,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
        "payload missing before replay",
    );
    assert!(matches!(
        envelope.payload_status,
        TransportPayloadStatus::Missing { .. }
    ));

    let result = service.process_envelope(
        &envelope,
        vote_exchange_context(
            &fixture,
            fixture.ready_secondaries[0].aggregator_id,
            &fixture.subject,
        ),
    );
    match result.outcome {
        VoteExchangeOutcome::Evidence(VoteEvidence::PayloadWithholding(evidence)) => {
            assert_eq!(evidence.accused_id, fixture.primary);
            assert_eq!(
                evidence.reporter_id,
                fixture.ready_secondaries[0].aggregator_id
            );
            assert_eq!(evidence.subject_digest, fixture.subject.digest());
            assert_eq!(evidence.payload_digest, fixture.subject.payload_digest);
        }
        other => panic!("expected payload-withholding evidence, got {other:?}"),
    }
    assert_eq!(service.evidence_records().len(), 1);

    Ok(())
}

struct TransportFixture {
    subject: CommitSubject,
    items: Vec<WorkItem>,
    planner: BatchPlanner,
    placement_table: ShardPlacementTable,
    record: z00z_aggregators::ShardRecoveryRecord,
    recovery: SettlementRecoveryState,
    publication: z00z_aggregators::PublicationBinding,
    theorem_digest: [u8; 32],
    primary: AggregatorId,
    ready_secondaries: [SecondaryState; 2],
}

fn transport_fixture() -> Result<TransportFixture, Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: ShardId::new(6),
        routing_generation: 12,
    };
    let primary = AggregatorId::new(31);
    let ready_secondaries = [
        SecondaryState::ready(AggregatorId::new(32)),
        SecondaryState::ready(AggregatorId::new(33)),
    ];
    let planner = planner(route);
    let items = vec![tx_item("transport-adapter")];
    let batch = planner
        .make_batch(batch_id("transport-adapter"), &items)
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
        vec![ready_secondaries[0], ready_secondaries[1]],
        recovery.journal_lineage,
    );
    let mut placement_table = ShardPlacementTable::default();
    placement_table.insert(placement.clone());
    let record = recovery_record(
        "transport-adapter",
        route,
        primary,
        placement.secondaries.clone(),
        recovery.clone(),
    );
    let candidate = z00z_aggregators::JournalCandidate::from_record(&record).expect("candidate");
    let publication = publication_binding(
        &batch,
        SettlementStateRoot::settlement_v1([0x11; 32]),
        candidate.state_root,
    );
    let theorem_digest = publication.pub_in_digest();
    let subject = CommitSubject::from_runtime(
        17,
        membership_digest_for_voters(
            route,
            primary,
            ready_secondaries
                .iter()
                .map(|secondary| secondary.aggregator_id),
        ),
        &batch,
        &candidate,
        &publication,
        theorem_digest,
        None,
    )
    .expect("commit subject");
    Ok(TransportFixture {
        subject,
        items,
        planner,
        placement_table,
        record,
        recovery,
        publication,
        theorem_digest,
        primary,
        ready_secondaries,
    })
}

fn vote_exchange_context<'a>(
    fixture: &'a TransportFixture,
    voter_id: AggregatorId,
    subject: &'a CommitSubject,
) -> VoteExchangeContext<'a> {
    VoteExchangeContext {
        voter_role: ShardVoteRole::Secondary,
        replay_request: SecondaryReplayRequest {
            voter_id,
            term: subject.term,
            items: &fixture.items,
            planner: &fixture.planner,
            placement_table: &fixture.placement_table,
            recovery_record: &fixture.record,
            local_recovery: &fixture.recovery,
            publication_binding: &fixture.publication,
            theorem_or_settlement_digest: fixture.theorem_digest,
            da_availability_digest: None,
        },
    }
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
