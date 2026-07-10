#[path = "test_common.rs"]
mod test_common;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;
#[path = "../../../z00z_rollup_node/tests/support/test_theorem_fixture.rs"]
mod theorem_fixture;

use z00z_aggregators::{
    bind_publication_contract, AggregatorId, BatchPlanner, BatchRoute, CommitSubject,
    ConsensusAdapter, IngressBoundary, JournalCandidate, PublicationBinding, RejectClass,
    RouteRangeRule, SecondaryReplayRejectCode, SecondaryReplayRequest, SecondaryReplayVerdict,
    SecondaryReplayVerifier, SecondaryState, ShardId, ShardPlacement, ShardPlacementTable,
    ShardRecoveryRecord, ShardRouteTable, WorkItem, WorkPayload,
};
use z00z_rollup_node::SettlementTheoremBundle;
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, CheckpointDaProviderFamily, CheckpointDraft, CheckpointExecInputId,
        CheckpointVersion, CreatedEnt, SpentEnt,
    },
    settlement::{CheckRoot, SettlementRecoveryState, SettlementStateRoot},
    snapshot::PrepSnapshotId,
};

use self::{
    test_common::{tx_item, HASH_MAX, HASH_MIN},
    test_recovery_common::{
        batch_id, placement_table, recovery_record, route_bound_recovery_state,
    },
};

struct ReplayFixture {
    term: u64,
    route: BatchRoute,
    batch_id: z00z_aggregators::BatchId,
    primary: AggregatorId,
    voter_id: AggregatorId,
    secondaries: Vec<SecondaryState>,
    items: Vec<WorkItem>,
    planner: BatchPlanner,
    placement_table: ShardPlacementTable,
    record: ShardRecoveryRecord,
    current: SettlementRecoveryState,
    publication_binding: PublicationBinding,
    theorem_digest: [u8; 32],
    da_availability_digest: Option<[u8; 32]>,
    subject: CommitSubject,
    publication_prev_root: SettlementStateRoot,
}

impl ReplayFixture {
    fn request<'a>(
        &'a self,
        items: &'a [WorkItem],
        planner: &'a BatchPlanner,
        placement_table: &'a ShardPlacementTable,
        record: &'a ShardRecoveryRecord,
        current: &'a SettlementRecoveryState,
        publication_binding: &'a PublicationBinding,
        theorem_digest: [u8; 32],
    ) -> SecondaryReplayRequest<'a> {
        SecondaryReplayRequest {
            voter_id: self.voter_id,
            term: self.term,
            items,
            planner,
            placement_table,
            recovery_record: record,
            local_recovery: current,
            publication_binding,
            theorem_or_settlement_digest: theorem_digest,
            da_availability_digest: self.da_availability_digest,
        }
    }
}

#[test]
fn test_exact_primary_subject_is_replayed_and_accepted() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = replay_fixture()?;
    let verifier = SecondaryReplayVerifier;
    let replayed = verifier
        .replay_subject(&fixture.request(
            &fixture.items,
            &fixture.planner,
            &fixture.placement_table,
            &fixture.record,
            &fixture.current,
            &fixture.publication_binding,
            fixture.theorem_digest,
        ))
        .expect("exact live inputs must replay");
    assert_eq!(replayed, fixture.subject);

    let verdict = verifier.verify(
        &fixture.subject,
        &fixture.request(
            &fixture.items,
            &fixture.planner,
            &fixture.placement_table,
            &fixture.record,
            &fixture.current,
            &fixture.publication_binding,
            fixture.theorem_digest,
        ),
    );
    match verdict {
        SecondaryReplayVerdict::Accept(accept) => assert_eq!(accept.subject, fixture.subject),
        SecondaryReplayVerdict::Reject(reject) => panic!("unexpected reject: {reject:?}"),
    }

    Ok(())
}

#[test]
fn test_named_drift_axes_reject_with_stable_codes() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = replay_fixture()?;
    let verifier = SecondaryReplayVerifier;

    let drifted_route = BatchRoute {
        shard_id: fixture.route.shard_id,
        routing_generation: fixture.route.routing_generation + 1,
    };
    let drifted_route_planner = planner(drifted_route);
    assert_reject(
        verifier.verify(
            &fixture.subject,
            &fixture.request(
                &fixture.items,
                &drifted_route_planner,
                &fixture.placement_table,
                &fixture.record,
                &fixture.current,
                &fixture.publication_binding,
                fixture.theorem_digest,
            ),
        ),
        SecondaryReplayRejectCode::WrongRoute,
        RejectClass::PolicyReject,
        "route",
    );

    let drifted_plan_items = vec![
        fixture.items[0].clone(),
        tx_item("secondary-replay-plan-drift"),
    ];
    assert_reject(
        verifier.verify(
            &fixture.subject,
            &fixture.request(
                &drifted_plan_items,
                &fixture.planner,
                &fixture.placement_table,
                &fixture.record,
                &fixture.current,
                &fixture.publication_binding,
                fixture.theorem_digest,
            ),
        ),
        SecondaryReplayRejectCode::WrongPlanDigest,
        RejectClass::PolicyReject,
        "planner digest drift",
    );

    let mut root_recovery = fixture.current.clone();
    root_recovery.state_root = SettlementStateRoot::settlement_v1([0x77; 32]);
    let root_record = record_with_recovery(
        &fixture,
        root_recovery.clone(),
        root_recovery.journal_lineage,
    );
    let root_table = placement_table(
        fixture.route,
        fixture.primary,
        fixture.secondaries.clone(),
        root_recovery.journal_lineage,
    );
    let root_publication = publication_binding(
        fixture.batch_id,
        fixture.subject.route_table_digest,
        fixture.publication_prev_root,
        root_recovery.state_root,
    );
    assert_reject(
        verifier.verify(
            &fixture.subject,
            &fixture.request(
                &fixture.items,
                &fixture.planner,
                &root_table,
                &root_record,
                &root_recovery,
                &root_publication,
                fixture.theorem_digest,
            ),
        ),
        SecondaryReplayRejectCode::WrongRoot,
        RejectClass::PolicyReject,
        "wrong root",
    );

    let mut lineage_recovery = fixture.current.clone();
    lineage_recovery.journal_lineage = [0x66; 32];
    let lineage_record = record_with_recovery(
        &fixture,
        lineage_recovery.clone(),
        lineage_recovery.journal_lineage,
    );
    let lineage_table = placement_table(
        fixture.route,
        fixture.primary,
        fixture.secondaries.clone(),
        lineage_recovery.journal_lineage,
    );
    assert_reject(
        verifier.verify(
            &fixture.subject,
            &fixture.request(
                &fixture.items,
                &fixture.planner,
                &lineage_table,
                &lineage_record,
                &lineage_recovery,
                &fixture.publication_binding,
                fixture.theorem_digest,
            ),
        ),
        SecondaryReplayRejectCode::WrongLineage,
        RejectClass::PolicyReject,
        "wrong lineage",
    );

    let mut proof_recovery = fixture.current.clone();
    proof_recovery.proof_version = proof_recovery.proof_version.saturating_add(1);
    let proof_record = record_with_recovery(
        &fixture,
        proof_recovery.clone(),
        proof_recovery.journal_lineage,
    );
    let proof_table = placement_table(
        fixture.route,
        fixture.primary,
        fixture.secondaries.clone(),
        proof_recovery.journal_lineage,
    );
    assert_reject(
        verifier.verify(
            &fixture.subject,
            &fixture.request(
                &fixture.items,
                &fixture.planner,
                &proof_table,
                &proof_record,
                &proof_recovery,
                &fixture.publication_binding,
                fixture.theorem_digest,
            ),
        ),
        SecondaryReplayRejectCode::WrongProofVersion,
        RejectClass::PolicyReject,
        "wrong proof version",
    );

    let mut policy_recovery = fixture.current.clone();
    policy_recovery.bucket_policy_generation =
        policy_recovery.bucket_policy_generation.saturating_add(1);
    policy_recovery.bucket_policy_id = [0x55; 32];
    let policy_record = record_with_recovery(
        &fixture,
        policy_recovery.clone(),
        policy_recovery.journal_lineage,
    );
    let policy_table = placement_table(
        fixture.route,
        fixture.primary,
        fixture.secondaries.clone(),
        policy_recovery.journal_lineage,
    );
    assert_reject(
        verifier.verify(
            &fixture.subject,
            &fixture.request(
                &fixture.items,
                &fixture.planner,
                &policy_table,
                &policy_record,
                &policy_recovery,
                &fixture.publication_binding,
                fixture.theorem_digest,
            ),
        ),
        SecondaryReplayRejectCode::WrongPolicyGeneration,
        RejectClass::PolicyReject,
        "wrong policy generation",
    );

    let publication_drift = publication_binding(
        fixture.batch_id,
        [0x44; 32],
        fixture.publication_prev_root,
        fixture.current.state_root,
    );
    assert_reject(
        verifier.verify(
            &fixture.subject,
            &fixture.request(
                &fixture.items,
                &fixture.planner,
                &fixture.placement_table,
                &fixture.record,
                &fixture.current,
                &publication_drift,
                fixture.theorem_digest,
            ),
        ),
        SecondaryReplayRejectCode::WrongPublicationBinding,
        RejectClass::PolicyReject,
        "publication",
    );

    assert_reject(
        verifier.verify(
            &fixture.subject,
            &fixture.request(
                &fixture.items,
                &fixture.planner,
                &fixture.placement_table,
                &fixture.record,
                &fixture.current,
                &fixture.publication_binding,
                [0x91; 32],
            ),
        ),
        SecondaryReplayRejectCode::WrongTheoremDigest,
        RejectClass::PolicyReject,
        "theorem",
    );

    let membership_table = placement_table(
        fixture.route,
        fixture.primary,
        vec![SecondaryState::ready(fixture.voter_id)],
        fixture.current.journal_lineage,
    );
    assert_reject(
        verifier.verify(
            &fixture.subject,
            &fixture.request(
                &fixture.items,
                &fixture.planner,
                &membership_table,
                &fixture.record,
                &fixture.current,
                &fixture.publication_binding,
                fixture.theorem_digest,
            ),
        ),
        SecondaryReplayRejectCode::MembershipDrift,
        RejectClass::PolicyReject,
        "membership drift",
    );

    let mut availability_request = fixture.request(
        &fixture.items,
        &fixture.planner,
        &fixture.placement_table,
        &fixture.record,
        &fixture.current,
        &fixture.publication_binding,
        fixture.theorem_digest,
    );
    availability_request.da_availability_digest = Some([0x92; 32]);
    assert_reject(
        verifier.verify(&fixture.subject, &availability_request),
        SecondaryReplayRejectCode::WrongDaAvailability,
        RejectClass::PolicyReject,
        "data-availability",
    );

    let mut term_request = fixture.request(
        &fixture.items,
        &fixture.planner,
        &fixture.placement_table,
        &fixture.record,
        &fixture.current,
        &fixture.publication_binding,
        fixture.theorem_digest,
    );
    term_request.term = fixture.term.saturating_add(1);
    assert_reject(
        verifier.verify(&fixture.subject, &term_request),
        SecondaryReplayRejectCode::WrongTerm,
        RejectClass::PolicyReject,
        "wrong term",
    );

    Ok(())
}

#[test]
fn test_stale_secondary_state_rejects_before_vote_creation(
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = replay_fixture()?;
    let verifier = SecondaryReplayVerifier;
    let mut stale_current = fixture.current.clone();
    stale_current.state_root = SettlementStateRoot::settlement_v1([0x33; 32]);

    assert_reject(
        verifier.verify(
            &fixture.subject,
            &fixture.request(
                &fixture.items,
                &fixture.planner,
                &fixture.placement_table,
                &fixture.record,
                &stale_current,
                &fixture.publication_binding,
                fixture.theorem_digest,
            ),
        ),
        SecondaryReplayRejectCode::StaleSecondaryState,
        RejectClass::PolicyReject,
        "stale local root",
    );

    Ok(())
}

fn replay_fixture() -> Result<ReplayFixture, Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: ShardId::new(5),
        routing_generation: 12,
    };
    let primary = AggregatorId::new(21);
    let secondaries = vec![
        SecondaryState::ready(AggregatorId::new(22)),
        SecondaryState::ready(AggregatorId::new(23)),
    ];
    let voter_id = secondaries[0].aggregator_id;
    let term = 17;
    let batch_id = batch_id("secondary-replay");
    let request = theorem_fixture::publication_request(batch_id.into_bytes(), "secondary-replay-1");
    let item = IngressBoundary
        .normalize(WorkPayload::Tx(Box::new(request.tx_package.clone())))
        .expect("normalized theorem tx");
    let items = vec![item];
    let planner = planner(route);
    let batch = planner
        .make_batch(batch_id, &items)
        .expect("route-local planned batch");
    let current = route_bound_recovery_state(
        0x81,
        batch.batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let record = recovery_record(
        "secondary-replay",
        route,
        primary,
        secondaries.clone(),
        current.clone(),
    );
    let candidate = JournalCandidate::from_record(&record).expect("journal candidate");
    let placement =
        ShardPlacement::new(route, primary, secondaries.clone(), current.journal_lineage);
    let membership_digest = ConsensusAdapter::from_placement(&placement)
        .expect("consensus adapter")
        .membership_digest();
    let publication_prev_root = SettlementStateRoot::settlement_v1([0x11; 32]);
    let publication_binding = publication_binding(
        batch.batch_id,
        batch.planned.route_table_digest.into_bytes(),
        publication_prev_root,
        current.state_root,
    );
    let theorem_digest = theorem_digest(&request)?;
    let da_availability_digest = Some([0xAB; 32]);
    let subject = CommitSubject::from_runtime(
        term,
        membership_digest,
        &batch,
        &candidate,
        &publication_binding,
        theorem_digest,
        da_availability_digest,
    )
    .expect("commit subject");

    Ok(ReplayFixture {
        term,
        route,
        batch_id,
        primary,
        voter_id,
        secondaries: secondaries.clone(),
        items,
        planner,
        placement_table: placement_table(route, primary, secondaries, current.journal_lineage),
        record,
        current,
        publication_binding,
        theorem_digest,
        da_availability_digest,
        subject,
        publication_prev_root,
    })
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

fn record_with_recovery(
    fixture: &ReplayFixture,
    recovery: SettlementRecoveryState,
    expected_lineage: [u8; 32],
) -> ShardRecoveryRecord {
    let mut record = fixture.record.clone();
    record.recovery = recovery;
    record.placement.expected_journal_lineage = expected_lineage;
    record
}

fn publication_binding(
    batch_id: z00z_aggregators::BatchId,
    route_table_digest: [u8; 32],
    prev_root: SettlementStateRoot,
    new_root: SettlementStateRoot,
) -> PublicationBinding {
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
        batch_id,
        checkpoint_id,
        route_table_digest,
        &artifact.pub_in(),
    )
}

fn theorem_digest(
    request: &z00z_aggregators::PublicationRequest,
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let artifact = theorem_fixture::canonical_artifact_for_request(
        request,
        CheckpointDaProviderFamily::LocalArchive,
    );
    let theorem = SettlementTheoremBundle::new(
        request.tx_package.clone(),
        artifact,
        request.exec_input.clone(),
        request.link.clone(),
    )?;
    Ok(theorem.theorem_digest())
}

fn assert_reject(
    verdict: SecondaryReplayVerdict,
    code: SecondaryReplayRejectCode,
    class: RejectClass,
    detail_fragment: &str,
) {
    match verdict {
        SecondaryReplayVerdict::Accept(accept) => {
            panic!("unexpected accept: {:?}", accept.subject.digest())
        }
        SecondaryReplayVerdict::Reject(reject) => {
            assert_eq!(reject.code, code);
            assert_eq!(reject.class, class);
            assert!(
                reject.detail.contains(detail_fragment),
                "detail `{}` missing `{detail_fragment}`",
                reject.detail
            );
        }
    }
}
