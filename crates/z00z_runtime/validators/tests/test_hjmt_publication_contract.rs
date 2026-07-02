#[path = "test_theorem_support.rs"]
mod theorem_support;

use z00z_aggregators::{
    AggregatorId, BatchId, BatchPlanned, BatchRoute, OrderedBatch, PlanDigest, PublishedBatch,
    ShardExecState, ShardExecTicket, ShardId, ShardPlacementView,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, CheckpointArtifact, CheckpointId, CheckpointLink,
        CheckpointLinkVersion, CheckpointPubIn,
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
        ordered_batch(batch_id),
        theorem,
        Vec::new(),
        placement,
        exec_ticket,
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
        standby: Vec::new(),
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
