#[path = "test_common.rs"]
mod test_common;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;

use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, AggregatorId, ArtifactKind,
    BatchPlanner, BatchRoute, EquivocationEvidence, EvidenceKind, EvidenceRecord,
    MissingBlobEvidence, PayloadWithholdingEvidence, RouteRangeRule, SecondaryState, ShardId,
    ShardRouteTable, ShardVote, ShardVoteKind, ShardVoteRole, SplitBrainEvidence,
    StaleMemberEvidence, TransportFaultEvidence, TransportFaultEvidenceKind, VoteTransportEnvelope,
    WrongRootEvidence, WrongRouteDigestEvidence,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, CheckpointDraft, CheckpointExecInputId, CheckpointVersion,
        CreatedEnt, SpentEnt,
    },
    settlement::{CheckRoot, SettlementStateRoot},
    snapshot::PrepSnapshotId,
};
use z00z_utils::codec::{Codec, JsonCodec};

use self::{
    test_common::{batch_id, tx_item, HASH_MAX, HASH_MIN},
    test_recovery_common::{recovery_record, route_bound_recovery_state},
};

#[test]
fn test_structured_evidence_registry_covers_gate14() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = evidence_fixture()?;
    let first_vote = ShardVote::new_local(
        fixture.secondary.aggregator_id,
        ShardVoteRole::Secondary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        fixture.subject.digest(),
        ShardVoteKind::LocalCommit,
    );
    let mut conflicting_subject = fixture.subject.clone();
    conflicting_subject.theorem_or_settlement_digest = [0x91; 32];
    let second_vote = ShardVote::new_local(
        fixture.secondary.aggregator_id,
        ShardVoteRole::Secondary,
        conflicting_subject.shard_id,
        conflicting_subject.term,
        conflicting_subject.membership_digest,
        conflicting_subject.digest(),
        ShardVoteKind::LocalCommit,
    );

    let required = vec![
        EvidenceRecord::Equivocation(
            EquivocationEvidence::new(first_vote, second_vote)
                .expect("equivocation evidence fixture"),
        ),
        EvidenceRecord::PayloadWithholding(
            PayloadWithholdingEvidence::new(
                fixture.secondary.aggregator_id,
                fixture.primary,
                fixture.subject.shard_id,
                fixture.subject.term,
                fixture.subject.membership_digest,
                fixture.subject.digest(),
                fixture.subject.payload_digest,
                "payload missing before replay",
            )
            .expect("payload withholding fixture"),
        ),
        EvidenceRecord::MissingBlob(
            MissingBlobEvidence::new(
                fixture.secondary.aggregator_id,
                fixture.subject.shard_id,
                fixture.subject.term,
                fixture.subject.membership_digest,
                fixture.subject.digest(),
                [0x11; 8],
                [0x22; 32],
                [0x33; 32],
                "blob bytes were unavailable during resolution",
            )
            .expect("missing blob fixture"),
        ),
        EvidenceRecord::WrongRoot(
            WrongRootEvidence::new(
                fixture.secondary.aggregator_id,
                fixture.subject.shard_id,
                fixture.subject.term,
                fixture.subject.digest(),
                [0x44; 32],
                [0x55; 32],
                "wrong root drifted from replayed execution",
            )
            .expect("wrong root fixture"),
        ),
        EvidenceRecord::WrongRouteDigest(
            WrongRouteDigestEvidence::new(
                fixture.secondary.aggregator_id,
                fixture.subject.shard_id,
                fixture.subject.term,
                fixture.subject.digest(),
                [0x66; 32],
                [0x77; 32],
                "wrong route digest drifted from the live table",
            )
            .expect("wrong route fixture"),
        ),
        EvidenceRecord::StaleMember(
            StaleMemberEvidence::new(
                fixture.secondary.aggregator_id,
                fixture.subject.shard_id,
                fixture.subject.term,
                fixture.subject.membership_digest,
                [0x88; 32],
                "stale member replay drifted from the live membership",
            )
            .expect("stale member fixture"),
        ),
        EvidenceRecord::SplitBrain(
            SplitBrainEvidence::new(
                fixture.primary,
                fixture.subject.shard_id,
                fixture.subject.term,
                fixture.subject.membership_digest,
                fixture.subject.digest(),
                conflicting_subject.digest(),
                "same-term divergent root froze the committee",
            )
            .expect("split brain fixture"),
        ),
    ];

    let kinds = required
        .iter()
        .map(|entry| entry.kind())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&EvidenceKind::Equivocation));
    assert!(kinds.contains(&EvidenceKind::PayloadWithholding));
    assert!(kinds.contains(&EvidenceKind::MissingBlob));
    assert!(kinds.contains(&EvidenceKind::WrongRoot));
    assert!(kinds.contains(&EvidenceKind::WrongRouteDigest));
    assert!(kinds.contains(&EvidenceKind::StaleMember));
    assert!(kinds.contains(&EvidenceKind::SplitBrain));
    assert!(required
        .iter()
        .all(|entry| !entry.artifact_refs().is_empty()));
    assert!(required.iter().all(|entry| entry.digest() != [0u8; 32]));

    let transport = EvidenceRecord::TransportFault(TransportFaultEvidence::for_envelope(
        TransportFaultEvidenceKind::Replay,
        7,
        &VoteTransportEnvelope::available(
            fixture.primary,
            fixture.secondary.aggregator_id,
            fixture.subject.clone(),
            ShardVoteKind::LocalCommit,
        ),
        "transport replayed the vote envelope",
    ));
    assert_eq!(transport.kind(), EvidenceKind::TransportFault);
    assert!(transport
        .artifact_refs()
        .iter()
        .any(|artifact| artifact.kind == ArtifactKind::TransportMessage));

    let codec = JsonCodec;
    let encoded = codec.serialize(&required)?;
    let decoded: Vec<EvidenceRecord> = codec.deserialize(&encoded)?;
    assert_eq!(decoded, required);

    Ok(())
}

#[test]
fn test_structured_evidence_registry_rejects_malformed_records() {
    let shard_id = ShardId::new(9);
    let reporter = AggregatorId::new(41);

    let missing_blob_err = MissingBlobEvidence::new(
        reporter,
        shard_id,
        3,
        [0x11; 32],
        [0x22; 32],
        [0u8; 8],
        [0x33; 32],
        [0x44; 32],
        "missing namespace",
    )
    .expect_err("namespace is mandatory");
    assert!(missing_blob_err.detail.contains("namespace"));

    let wrong_root_err = WrongRootEvidence::new(
        reporter,
        shard_id,
        3,
        [0x55; 32],
        [0x66; 32],
        [0x66; 32],
        "identical roots",
    )
    .expect_err("root digests must conflict");
    assert!(wrong_root_err.detail.contains("conflicting root digests"));

    let stale_member_err = StaleMemberEvidence::new(
        reporter,
        shard_id,
        3,
        [0x77; 32],
        [0x77; 32],
        "identical membership",
    )
    .expect_err("membership digests must conflict");
    assert!(stale_member_err
        .detail
        .contains("conflicting membership digests"));

    let split_brain_err = SplitBrainEvidence::new(
        reporter,
        shard_id,
        3,
        [0x88; 32],
        [0x99; 32],
        [0x99; 32],
        "same subject twice",
    )
    .expect_err("split brain requires two subjects");
    assert!(split_brain_err
        .detail
        .contains("conflicting subject digests"));
}

struct EvidenceFixture {
    primary: AggregatorId,
    subject: z00z_aggregators::CommitSubject,
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
        .make_batch(
            batch_id("evidence-registry-fixture"),
            &[tx_item("registry")],
        )
        .expect("planned batch");
    let recovery = route_bound_recovery_state(
        0x81,
        batch.batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let record = recovery_record(
        "evidence-registry-fixture",
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
    let subject = z00z_aggregators::CommitSubject::from_runtime(
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
    Ok(EvidenceFixture {
        primary,
        subject,
        secondary,
    })
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
