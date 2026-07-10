use z00z_aggregators::{
    AggregatorIngress, AggregatorOrdering, AggregatorRecovery, AggregatorService, BatchId,
    BatchPlanned, BatchRoute, OrderedBatch, PlanDigest, PublicationRecord, PublicationRequest,
    PublicationState, PublishedBatch, RecoveryBoundary, RejectRecord, RouteRangeRule,
    ShardExecState, ShardExecTicket, ShardId, ShardRouteTable, WorkItem, WorkPayload,
};
use z00z_storage::{
    checkpoint::{CheckpointDraftId, CheckpointExecOut, CheckpointExecTx, CheckpointInRef},
    settlement::{
        check_publication_route_v1, CheckpointPublicationV1, DefinitionId, PolicySetCommitmentV1,
        PublicationModeTagV1, PublicationRouteSnapshotV1, RootGenerationTagV1, SerialId,
        SettlementLeaf, SettlementPath, SettlementRecoveryState, ShardRootLeafV1, StoreItem,
        StoreOp, TerminalId, TerminalLeaf,
    },
};

struct PassthroughService;

impl AggregatorIngress for PassthroughService {
    fn admit(&mut self, _item: WorkPayload) -> Result<WorkItem, RejectRecord> {
        unreachable!("bind_exec_handoff test does not exercise ingress")
    }
}

impl AggregatorOrdering for PassthroughService {
    fn order(&mut self, _items: &[WorkItem]) -> Result<OrderedBatch, RejectRecord> {
        unreachable!("bind_exec_handoff test does not exercise ordering")
    }
}

impl AggregatorRecovery for PassthroughService {
    fn build_publication(&mut self, _batch: OrderedBatch) -> PublicationRequest {
        unreachable!("bind_exec_handoff test does not exercise publication assembly")
    }

    fn record_publication(&mut self, _batch: PublishedBatch) -> PublicationRecord {
        unreachable!("bind_exec_handoff test does not exercise publication recording")
    }
}

impl AggregatorService for PassthroughService {
    fn emit_soft_confirmation(
        &self,
        _intake_id: &z00z_aggregators::IntakeId,
        _batch_id: &BatchId,
    ) -> z00z_aggregators::SoftConfirmation {
        unreachable!("bind_exec_handoff test does not exercise confirmations")
    }
}

#[test]
fn test_bind_keeps_route_metadata() {
    let service = PassthroughService;
    let batch = ordered_batch();
    let route_table = route_table();
    let path = SettlementPath::new(
        DefinitionId::new([4u8; 32]),
        SerialId::new(9),
        TerminalId::new([5u8; 32]),
    );
    let ops = vec![StoreOp::Put(Box::new(
        StoreItem::new(path, term_leaf(path)).expect("store item"),
    ))];
    let txs = vec![CheckpointExecTx::new(
        vec![CheckpointInRef::new([6u8; 32], SerialId::new(8))],
        vec![CheckpointExecOut::new(path.definition_id, term_leaf(path)).expect("exec out")],
        vec![9u8],
    )
    .expect("exec tx")];

    let handoff = service.bind_exec_handoff(&batch, ops.clone(), txs.clone());

    assert_eq!(handoff.route().batch_id(), [0x21; 32]);
    assert_eq!(handoff.route().shard_id(), 7);
    assert_eq!(handoff.route().routing_generation(), 13);
    assert_eq!(
        handoff.route().route_table_digest(),
        route_table.digest().into_bytes()
    );
    assert_eq!(handoff.ops(), ops.as_slice());
    assert_eq!(handoff.txs(), txs.as_slice());
}

#[test]
fn test_handoff_stays_checkpoint_free() {
    let record = RecoveryBoundary.mark_handed_off(BatchId::new(CheckpointDraftId::new([0x31; 32])));

    assert_eq!(
        record.batch_id,
        BatchId::new(CheckpointDraftId::new([0x31; 32]))
    );
    assert_eq!(record.checkpoint_id, None);
    assert_eq!(record.state, PublicationState::HandedOff);
}

#[test]
fn capture_rejects_mismatched_publication_batch() {
    let route = BatchRoute {
        shard_id: ShardId::new(4),
        routing_generation: 15,
    };
    let placement = z00z_aggregators::ShardPlacement::new(
        route,
        z00z_aggregators::AggregatorId::new(19),
        vec![z00z_aggregators::SecondaryState::ready(
            z00z_aggregators::AggregatorId::new(20),
        )],
        [0x44; 32],
    );
    let ticket = ShardExecTicket {
        batch_id: BatchId::new(CheckpointDraftId::new([0x41; 32])),
        placement: placement.view(),
        state: ShardExecState::Routed,
    };
    let publication = PublicationRecord {
        batch_id: BatchId::new(CheckpointDraftId::new([0x42; 32])),
        checkpoint_id: None,
        state: PublicationState::HandedOff,
        da_reference: None,
        publication_evidence: None,
        lifecycle: None,
    };

    let err = RecoveryBoundary
        .capture(&ticket, &publication, recovery_state())
        .expect_err("mismatched publication batch must reject");

    assert_eq!(err.class, z00z_aggregators::RejectClass::PolicyReject);
    assert!(err.detail.contains("batch id"));
}

#[test]
fn test_binds_route_metadata() {
    let batch = ordered_batch();
    let route_table = route_table();
    let recovery = recovery_state();
    let policy_set = PolicySetCommitmentV1::singleton_live(
        u64::from(recovery.bucket_policy_generation),
        recovery.bucket_policy_id,
        recovery.version,
    );
    let leaf = ShardRootLeafV1::new(
        batch.planned.route.shard_id.as_u32(),
        recovery.state_root.into_bytes(),
        21,
        batch.planned.route.routing_generation,
        batch.planned.route_table_digest.into_bytes(),
        policy_set.digest().expect("policy-set digest"),
        recovery.version,
        44,
        0,
    );
    let publication = CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        recovery.version,
        batch.planned.route_table_digest.into_bytes(),
        recovery.state_root,
        vec![leaf],
    );

    publication
        .check_prior_root_v1(recovery.state_root)
        .expect("prior root binding");
    route_table.validate().expect("route table contract");
    assert!(route_table.activation_checkpoint <= publication.publication_checkpoint);
    assert_eq!(publication.shard_leaves[0].shard_id, 7);
    assert_eq!(publication.shard_leaves[0].routing_generation, 13);
    assert_eq!(
        publication.route_table_digest,
        route_table.digest().into_bytes()
    );
    check_publication_route_v1(
        &publication,
        &PublicationRouteSnapshotV1::new(
            route_table.routing_generation,
            route_table.digest().into_bytes(),
            route_table.activation_checkpoint,
            route_table
                .shard_set
                .iter()
                .map(|shard_id| shard_id.as_u32())
                .collect(),
        ),
    )
    .expect("route snapshot contract");
}

fn ordered_batch() -> OrderedBatch {
    let route_table = route_table();
    OrderedBatch {
        batch_id: BatchId::new(CheckpointDraftId::new([0x21; 32])),
        items: Vec::new(),
        created_leaves: Vec::<SettlementLeaf>::new(),
        planned: BatchPlanned {
            batch_id: BatchId::new(CheckpointDraftId::new([0x21; 32])),
            route: BatchRoute {
                shard_id: ShardId::new(7),
                routing_generation: 13,
            },
            route_table_digest: route_table.digest(),
            intake_ids: Vec::new(),
            op_count: 1,
            plan_digest: PlanDigest::new([0x23; 32]),
        },
    }
}

fn route_table() -> ShardRouteTable {
    let prev = ShardRouteTable::default();
    ShardRouteTable {
        routing_generation: 13,
        shard_set: vec![ShardId::new(7)],
        rules: vec![RouteRangeRule::new([0u8; 32], [0xff; 32], ShardId::new(7))],
        previous_generation_digest: Some(prev.digest()),
        activation_checkpoint: 9,
    }
}

fn recovery_state() -> SettlementRecoveryState {
    SettlementRecoveryState::new(
        9,
        z00z_storage::settlement::SettlementStateRoot::settlement_v1([0x45; 32]),
        1,
        1,
        2,
        [0x46; 32],
        [0x44; 32],
    )
}

fn term_leaf(path: SettlementPath) -> TerminalLeaf {
    let mut leaf = TerminalLeaf::dummy_for_scan(path.serial_id.get());
    leaf.asset_id = path.terminal_id().into_bytes();
    leaf.serial_id = path.serial_id.get();
    leaf
}
