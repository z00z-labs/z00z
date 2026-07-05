#[path = "test_common.rs"]
mod test_common;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;

use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, AggregatorId, BatchPlanner,
    BatchRoute, CommitSubject, DeterministicLocalVoteSigner, JournalCandidate, RouteRangeRule,
    SecondaryState, ShardId, ShardRouteTable, ShardVote, ShardVoteKind, ShardVoteRole,
    VoteSignature, VoteSignatureScheme,
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
fn test_signature_trait_signs_canonical_vote_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = signature_fixture()?;
    let signer = DeterministicLocalVoteSigner;
    let vote = ShardVote::new(
        fixture.primary,
        ShardVoteRole::Primary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        fixture.subject.digest(),
        ShardVoteKind::LocalCommit,
        &signer,
    );

    assert_eq!(
        vote.signature.scheme,
        VoteSignatureScheme::DeterministicLocal
    );
    assert!(vote.has_valid_signature());
    assert_eq!(vote.signature_scheme().as_str(), "deterministic_local");

    Ok(())
}

#[test]
fn test_signature_rejects_wrong_signer_and_digest_bindings(
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = signature_fixture()?;
    let vote = ShardVote::new_local(
        fixture.primary,
        ShardVoteRole::Primary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        fixture.subject.digest(),
        ShardVoteKind::LocalCommit,
    );

    let mut wrong_voter = vote.clone();
    wrong_voter.voter_id = fixture.secondaries[0].aggregator_id;
    assert!(!wrong_voter.has_valid_signature());

    let mut wrong_membership = vote.clone();
    wrong_membership.membership_digest = membership_digest_for_voters(
        fixture.subject.route(),
        fixture.primary,
        [fixture.secondaries[0].aggregator_id],
    );
    assert!(!wrong_membership.has_valid_signature());

    let mut wrong_subject = vote.clone();
    wrong_subject.subject_digest = [0x55; 32];
    assert!(!wrong_subject.has_valid_signature());

    Ok(())
}

#[test]
fn test_signature_rejects_tampered_signature_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = signature_fixture()?;
    let mut vote = ShardVote::new_local(
        fixture.primary,
        ShardVoteRole::Primary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        fixture.subject.digest(),
        ShardVoteKind::LocalCommit,
    );
    vote.signature = VoteSignature::new(VoteSignatureScheme::DeterministicLocal, vec![0u8; 32]);

    assert!(!vote.has_valid_signature());

    Ok(())
}

struct SignatureFixture {
    subject: CommitSubject,
    primary: AggregatorId,
    secondaries: Vec<SecondaryState>,
}

fn signature_fixture() -> Result<SignatureFixture, Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: ShardId::new(5),
        routing_generation: 12,
    };
    let primary = AggregatorId::new(21);
    let secondaries = vec![
        SecondaryState::ready(AggregatorId::new(22)),
        SecondaryState::ready(AggregatorId::new(23)),
    ];
    let batch = planner(route)
        .make_batch(batch_id("signature-fixture"), &[tx_item("signature")])
        .expect("planned batch");
    let recovery = route_bound_recovery_state(
        0x81,
        batch.batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let record = recovery_record(
        "signature-fixture",
        route,
        primary,
        secondaries.clone(),
        recovery,
    );
    let candidate = JournalCandidate::from_record(&record).expect("recovery candidate");
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
            secondaries.iter().map(|secondary| secondary.aggregator_id),
        ),
        &batch,
        &candidate,
        &publication,
        publication.pub_in_digest(),
        None,
    )
    .expect("commit subject");
    Ok(SignatureFixture {
        subject,
        primary,
        secondaries,
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
