#[path = "test_theorem_support.rs"]
mod theorem_support;

use z00z_aggregators::{
    AggregatorId, BatchId, BatchPlanned, BatchRoute, OrderedBatch, PlanDigest, PublicationRecord,
    PublicationState, PublishedBatch, ShardExecState, ShardExecTicket, ShardId, ShardPlacementView,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, CheckpointArtifact, CheckpointDaLocatorKind,
        CheckpointDaProviderFamily, CheckpointDaReferenceV1, CheckpointDaReferenceVersion,
        CheckpointId, CheckpointLifecycleV1, CheckpointLink, CheckpointLinkVersion,
        CheckpointPubIn, CheckpointPublicationEvidenceV1, CheckpointPublicationEvidenceVersion,
        CheckpointPublicationState,
    },
    settlement::{ObjectPolicyRegistryV1, PublicationRouteSnapshotV1, SettlementStateRoot},
};
use z00z_validators::{
    CheckpointFlow, RejectClass, ResolvedBatch, SettlementError, SettlementTheoremBundle,
    ValidatorBoundary, Verdict, VerdictKind,
};

#[test]
fn batch_prefers_exec_placement() {
    let batch_id = BatchId::from_bytes([0x31; 32]);
    let fallback = placement_view(1, 7, 3);
    let exec = exec_ticket(batch_id, 1, 7, 4, ShardExecState::Running);
    let resolved = resolved_batch(batch_id, Some(fallback), Some(exec.clone()));
    let boundary = ValidatorBoundary;
    let flow = boundary
        .checkpoint_flow(&resolved)
        .expect("checkpoint flow");

    assert_eq!(resolved.runtime_placement(), Some(&exec.placement));
    assert_eq!(resolved.runtime_exec(), Some(&exec));
    assert_eq!(boundary.placement_view(&resolved), Some(&exec.placement));
    assert_eq!(boundary.exec_ticket(&resolved), Some(&exec));
    assert_eq!(flow.ordered_route, resolved.ordered.planned.route);
    assert_eq!(flow.runtime_route, Some(exec.placement.route));
    assert_eq!(flow.publication_route, resolved.published.publication_route);
    assert!(flow
        .publication
        .matches_route_table_digest(resolved.ordered.planned.route_table_digest.into_bytes()));
    assert!(flow.publication.matches_pub_in(&resolved.published.pub_in));
    assert_eq!(
        resolved.published.checkpoint_id,
        derive_checkpoint_id(resolved.artifact()).expect("checkpoint id")
    );
}

#[test]
fn verdict_keeps_same_publication_checkpoint() {
    let batch_id = BatchId::from_bytes([0x41; 32]);
    let fallback = placement_view(1, 7, 6);
    let resolved = resolved_batch(batch_id, Some(fallback.clone()), None);
    let boundary = ValidatorBoundary;
    let flow = boundary
        .checkpoint_flow(&resolved)
        .expect("checkpoint flow");
    let verdict = Verdict {
        batch_id,
        checkpoint_id: Some(resolved.published.checkpoint_id),
        publication: Some(flow.publication.clone()),
        kind: VerdictKind::Accepted,
        reject: None,
        object_verdicts: Vec::new(),
    };

    assert_eq!(resolved.runtime_placement(), Some(&fallback));
    assert!(resolved.runtime_exec().is_none());
    assert_eq!(boundary.placement_view(&resolved), Some(&fallback));
    assert!(boundary.exec_ticket(&resolved).is_none());
    assert_eq!(verdict.batch_id, resolved.published.batch_id);
    assert_eq!(
        verdict.checkpoint_id,
        Some(resolved.published.checkpoint_id)
    );
    assert_eq!(
        verdict
            .publication
            .as_ref()
            .expect("publication")
            .binding_digest(),
        flow.binding_digest()
    );
    assert_eq!(resolved.published.pub_in, resolved.artifact().pub_in());
}

#[test]
fn checkpoint_rejects_pub_drift() {
    let batch_id = BatchId::from_bytes([0x51; 32]);
    let mut resolved = resolved_batch(batch_id, Some(placement_view(1, 7, 3)), None);
    resolved.published.pub_in = tampered_pub_in(&resolved.published.pub_in);

    let err = CheckpointFlow::try_from_resolved(&resolved).expect_err("pub_in drift must reject");

    assert_eq!(err, RejectClass::StateRootMismatch);
}

#[test]
fn checkpoint_rejects_route_drift() {
    let batch_id = BatchId::from_bytes([0x61; 32]);
    let resolved = resolved_batch(
        batch_id,
        Some(placement_view(1, 7, 3)),
        Some(exec_ticket(batch_id, 2, 8, 4, ShardExecState::Running)),
    );

    let err = CheckpointFlow::try_from_resolved(&resolved).expect_err("route drift must reject");

    assert_eq!(err, RejectClass::ReconcileInvalid);
}

#[test]
fn checkpoint_rejects_route_digest_drift() {
    let batch_id = BatchId::from_bytes([0x69; 32]);
    let mut resolved = resolved_batch(batch_id, Some(placement_view(1, 7, 3)), None);
    resolved.published.publication_route.route_table_digest = [0x99; 32];

    let err =
        CheckpointFlow::try_from_resolved(&resolved).expect_err("route digest drift must reject");

    assert_eq!(err, RejectClass::ReconcileInvalid);
}

#[test]
fn checkpoint_rejects_stale_route_activation() {
    let batch_id = BatchId::from_bytes([0x6A; 32]);
    let mut resolved = resolved_batch(batch_id, Some(placement_view(1, 7, 3)), None);
    resolved.published.publication_checkpoint =
        resolved.published.publication_route.activation_checkpoint - 1;

    let err = CheckpointFlow::try_from_resolved(&resolved)
        .expect_err("stale publication checkpoint must reject");

    assert_eq!(err, RejectClass::ReconcileInvalid);
}

#[test]
fn checkpoint_rejects_checkpoint_id_drift() {
    let batch_id = BatchId::from_bytes([0x71; 32]);
    let mut resolved = resolved_batch(batch_id, Some(placement_view(1, 7, 3)), None);
    resolved.published.checkpoint_id = CheckpointId::new([0xAB; 32]);

    let err =
        CheckpointFlow::try_from_resolved(&resolved).expect_err("checkpoint id drift must reject");

    assert_eq!(err, RejectClass::ReconcileInvalid);
}

#[test]
fn checkpoint_rejects_exec_batch_drift() {
    let batch_id = BatchId::from_bytes([0x81; 32]);
    let resolved = resolved_batch(
        batch_id,
        Some(placement_view(1, 7, 3)),
        Some(exec_ticket(
            BatchId::from_bytes([0x82; 32]),
            1,
            7,
            4,
            ShardExecState::Running,
        )),
    );

    let err =
        CheckpointFlow::try_from_resolved(&resolved).expect_err("exec batch drift must reject");

    assert_eq!(err, RejectClass::ReconcileInvalid);
}

#[test]
fn verdict_retry_incomplete() {
    let batch_id = BatchId::from_bytes([0x83; 32]);
    let resolved = resolved_batch(
        batch_id,
        Some(placement_view(1, 7, 3)),
        Some(exec_ticket(batch_id, 1, 7, 4, ShardExecState::RetryPending)),
    );

    let verdict =
        ValidatorBoundary.verdict_for_batch(&resolved, &ObjectPolicyRegistryV1::default());

    assert_eq!(verdict.kind, VerdictKind::Incomplete);
    assert!(verdict.reject.is_none());
    assert!(verdict.publication.is_some());
}

#[test]
fn verdict_blob_gap_incomplete() {
    let batch_id = BatchId::from_bytes([0x84; 32]);
    let mut resolved = resolved_batch(batch_id, Some(placement_view(1, 7, 3)), None);
    resolved.published.blob_ref.clear();

    let verdict =
        ValidatorBoundary.verdict_for_batch(&resolved, &ObjectPolicyRegistryV1::default());

    assert_eq!(verdict.kind, VerdictKind::Incomplete);
    assert!(verdict.reject.is_none());
    assert!(verdict.publication.is_some());
}

#[test]
fn verdict_does_not_open_challenge_without_publication_readiness() {
    let batch_id = BatchId::from_bytes([0x84; 32]);
    let resolved = resolved_batch(batch_id, Some(placement_view(1, 7, 3)), None);
    let verdict =
        ValidatorBoundary.verdict_for_batch(&resolved, &ObjectPolicyRegistryV1::default());
    let lifecycle = CheckpointLifecycleV1::from_artifact(resolved.artifact())
        .expect("sealed lifecycle")
        .link(resolved.artifact(), resolved.link(), None, [0xA4; 32])
        .expect("linked lifecycle");

    assert_eq!(verdict.kind, VerdictKind::Accepted);
    assert!(verdict.publication.is_some());
    assert_eq!(
        lifecycle.status(),
        z00z_storage::checkpoint::CheckpointLifecycleStatus::Linked
    );
    assert!(matches!(
        lifecycle.challenge_open(11),
        Err(z00z_storage::CheckpointError::LifecycleMix)
    ));
}

#[test]
fn validator_accepts_publication_readiness_bundle() {
    let batch_id = BatchId::from_bytes([0x8A; 32]);
    let mut resolved = resolved_batch(batch_id, Some(placement_view(1, 7, 3)), None);
    resolved.publication_record = Some(ready_publication_record(&resolved));

    let flow = CheckpointFlow::try_from_resolved(&resolved).expect("ready bundle must validate");
    let verdict =
        ValidatorBoundary.verdict_for_batch(&resolved, &ObjectPolicyRegistryV1::default());

    assert_eq!(flow.publication.batch_id(), batch_id);
    assert_eq!(verdict.kind, VerdictKind::Accepted);
    assert!(verdict.reject.is_none());
}

#[test]
fn validator_rejects_missing_publication_evidence_when_ready() {
    let batch_id = BatchId::from_bytes([0x8B; 32]);
    let mut resolved = resolved_batch(batch_id, Some(placement_view(1, 7, 3)), None);
    let mut publication = ready_publication_record(&resolved);
    publication.publication_evidence = None;
    resolved.publication_record = Some(publication);

    let err = CheckpointFlow::try_from_resolved(&resolved)
        .expect_err("ready lifecycle without publication evidence must reject");

    assert_eq!(err, RejectClass::ReconcileInvalid);
}

#[test]
fn validator_rejects_detached_da_ref_when_ready() {
    let batch_id = BatchId::from_bytes([0x8C; 32]);
    let mut resolved = resolved_batch(batch_id, Some(placement_view(1, 7, 3)), None);
    let mut publication = ready_publication_record(&resolved);
    let evidence = publication
        .publication_evidence
        .as_ref()
        .expect("publication evidence")
        .clone();
    publication.da_reference = Some(
        CheckpointDaReferenceV1::new(
            CheckpointDaReferenceVersion::CURRENT,
            evidence.provider_family(),
            CheckpointDaLocatorKind::OpaqueProviderRef,
            "local-da://validator-detached",
            evidence.payload_commitment(),
            evidence.statement_core_digest(),
            evidence.archive_manifest_root(),
            evidence.readiness_height(),
        )
        .expect("detached da ref"),
    );
    resolved.publication_record = Some(publication);

    let err =
        CheckpointFlow::try_from_resolved(&resolved).expect_err("detached da ref must reject");

    assert_eq!(err, RejectClass::ReconcileInvalid);
}

#[test]
fn test_keeps_adapter_advisory() {
    let batch_id = BatchId::from_bytes([0x85; 32]);
    let mut resolved = resolved_batch(batch_id, Some(placement_view(1, 7, 3)), None);
    resolved.published.da_provider = "forged-local-bridge".to_string();
    resolved.published.blob_ref = "local-da://forged-local-bridge/deadbeef".to_string();

    let flow = CheckpointFlow::try_from_resolved(&resolved)
        .expect("adapter metadata must stay advisory before binding drift");

    assert_eq!(flow.publication.batch_id(), batch_id);
    assert_eq!(
        flow.publication.checkpoint_id(),
        resolved.published.checkpoint_id
    );

    resolved.published.pub_in = tampered_pub_in(&resolved.published.pub_in);

    let err = CheckpointFlow::try_from_resolved(&resolved)
        .expect_err("binding drift must still reject even with local adapter metadata");

    assert_eq!(err, RejectClass::StateRootMismatch);
}

#[test]
fn checkpoint_accepts_live_quorum_binding() {
    let batch_id = BatchId::from_bytes([0x86; 32]);
    let resolved = resolved_batch_with_quorum(batch_id);

    let flow = CheckpointFlow::try_from_resolved(&resolved).expect("quorum-bound flow");

    assert_eq!(
        flow.binding_digest(),
        resolved
            .subject
            .as_ref()
            .expect("subject")
            .publication_binding_digest
    );
}

#[test]
fn checkpoint_rejects_missing_certificate_when_gate_enabled() {
    let batch_id = BatchId::from_bytes([0x87; 32]);
    let mut resolved = resolved_batch_with_quorum(batch_id);
    resolved.certificate = None;

    let err = CheckpointFlow::try_from_resolved(&resolved)
        .expect_err("missing certificate must reject when gate is enabled");

    assert_eq!(err, RejectClass::AuthInvalid);
}

#[test]
fn checkpoint_rejects_theorem_drift_when_gate_enabled() {
    let batch_id = BatchId::from_bytes([0x88; 32]);
    let mut resolved = resolved_batch_with_quorum(batch_id);
    resolved.published.theorem_digest = Some([0xAA; 32]);

    let err = CheckpointFlow::try_from_resolved(&resolved)
        .expect_err("theorem drift must reject when gate is enabled");

    assert_eq!(err, RejectClass::ProofInvalid);
}

#[test]
fn checkpoint_rejects_stale_membership_when_gate_enabled() {
    let batch_id = BatchId::from_bytes([0x89; 32]);
    let mut resolved = resolved_batch_with_quorum(batch_id);
    if let Some(placement) = resolved.placement.as_mut() {
        placement.secondaries.truncate(1);
    }

    let err = CheckpointFlow::try_from_resolved(&resolved)
        .expect_err("stale placement membership must reject");

    assert_eq!(err, RejectClass::ReconcileInvalid);
}

#[test]
fn resolved_batch_rejects_bad_theorem_link() {
    let (tx_package, exec_input, artifact, _link) = theorem_support::settlement_fixture();
    let bad_link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        CheckpointId::new([0xA5; 32]),
        exec_input.prep_snapshot_id(),
        z00z_storage::checkpoint::derive_exec_id(
            &z00z_storage::checkpoint::encode_exec_bin(&exec_input).expect("exec encode"),
        ),
    )
    .expect("bad link");

    let err = SettlementTheoremBundle::new(tx_package, artifact, exec_input, bad_link)
        .expect_err("bad theorem link must reject");

    assert!(matches!(err, SettlementError::CheckpointLink));
}

fn resolved_batch(
    batch_id: BatchId,
    placement: Option<ShardPlacementView>,
    exec_ticket: Option<ShardExecTicket>,
) -> ResolvedBatch {
    let theorem = theorem_support::theorem_bundle();
    let published = published_batch(batch_id, theorem.artifact());
    ResolvedBatch::new(
        published,
        None,
        ordered_batch(batch_id),
        theorem,
        None,
        None,
        Vec::new(),
        placement,
        exec_ticket,
    )
}

fn resolved_batch_with_quorum(batch_id: BatchId) -> ResolvedBatch {
    let fixture = theorem_support::quorum_fixture(batch_id);
    ResolvedBatch::new(
        fixture.published,
        None,
        fixture.ordered,
        fixture.theorem,
        Some(fixture.subject),
        Some(fixture.certificate),
        Vec::new(),
        Some(fixture.placement),
        None,
    )
}

fn published_batch(batch_id: BatchId, artifact: &CheckpointArtifact) -> PublishedBatch {
    let checkpoint_id = derive_checkpoint_id(artifact).expect("checkpoint id");
    PublishedBatch {
        batch_id,
        checkpoint_id,
        publication_checkpoint: 11,
        publication_route: publication_route(),
        pub_in: artifact.pub_in(),
        subject_digest: None,
        certificate_digest: None,
        theorem_digest: None,
        da_provider: "local-da".to_string(),
        blob_ref: "blob://hjmt-publication".to_string(),
    }
}

fn ordered_batch(batch_id: BatchId) -> OrderedBatch {
    OrderedBatch {
        batch_id,
        items: Vec::new(),
        created_leaves: Vec::new(),
        planned: BatchPlanned {
            batch_id,
            route: BatchRoute {
                shard_id: ShardId::new(1),
                routing_generation: 7,
            },
            route_table_digest: PlanDigest::new([0x51; 32]),
            intake_ids: Vec::new(),
            op_count: 0,
            plan_digest: PlanDigest::new([0x52; 32]),
        },
    }
}

fn publication_route() -> PublicationRouteSnapshotV1 {
    PublicationRouteSnapshotV1::new(7, [0x51; 32], 10, vec![1])
}

fn placement_view(shard_id: u16, generation: u64, aggregator_id: u16) -> ShardPlacementView {
    ShardPlacementView {
        route: BatchRoute {
            shard_id: ShardId::new(shard_id),
            routing_generation: generation,
        },
        primary_id: AggregatorId::new(aggregator_id),
        secondaries: Vec::new(),
        expected_journal_lineage: [0x61; 32],
    }
}

fn exec_ticket(
    batch_id: BatchId,
    shard_id: u16,
    generation: u64,
    aggregator_id: u16,
    state: ShardExecState,
) -> ShardExecTicket {
    ShardExecTicket {
        batch_id,
        placement: placement_view(shard_id, generation, aggregator_id),
        state,
    }
}

fn tampered_pub_in(pub_in: &CheckpointPubIn) -> CheckpointPubIn {
    let mut tampered = CheckpointPubIn::new_settlement(
        pub_in.prev_settlement_root(),
        SettlementStateRoot::settlement_v1([0xAA; 32]),
        pub_in.spent_delta().to_vec(),
        pub_in.created_delta().to_vec(),
    );
    if let Some(claim_root) = pub_in.claim_root() {
        tampered = tampered.with_claim_root(claim_root);
    }
    tampered
}

fn ready_publication_record(resolved: &ResolvedBatch) -> PublicationRecord {
    let statement_core_digest = [0xB4; 32];
    let archive_manifest_root = [0xB5; 32];
    let payload_commitment = [0xB6; 32];
    let observations_root = [0xB7; 32];
    let readiness_height = resolved.published.publication_checkpoint;
    let da_reference = CheckpointDaReferenceV1::new(
        CheckpointDaReferenceVersion::CURRENT,
        CheckpointDaProviderFamily::LocalArchive,
        CheckpointDaLocatorKind::OpaqueProviderRef,
        "local-da://validator-ready",
        payload_commitment,
        statement_core_digest,
        archive_manifest_root,
        readiness_height,
    )
    .expect("da reference");
    let publication_evidence = CheckpointPublicationEvidenceV1::new(
        CheckpointPublicationEvidenceVersion::CURRENT,
        statement_core_digest,
        da_reference.da_ref(),
        archive_manifest_root,
        payload_commitment,
        CheckpointPublicationState::DaPublicationReady,
        CheckpointDaProviderFamily::LocalArchive,
        readiness_height,
        readiness_height,
        observations_root,
    )
    .expect("publication evidence");
    let lifecycle = CheckpointLifecycleV1::from_artifact(resolved.artifact())
        .expect("sealed lifecycle")
        .link(
            resolved.artifact(),
            resolved.link(),
            None,
            statement_core_digest,
        )
        .expect("linked lifecycle")
        .publication_ready(&publication_evidence)
        .expect("publication-ready lifecycle");

    PublicationRecord {
        batch_id: resolved.published.batch_id,
        checkpoint_id: Some(resolved.published.checkpoint_id),
        state: PublicationState::Accepted,
        da_reference: Some(da_reference),
        publication_evidence: Some(publication_evidence),
        lifecycle: Some(lifecycle),
    }
}
