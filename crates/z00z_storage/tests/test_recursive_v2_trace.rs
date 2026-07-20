use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::ZkPackEncrypted;
use z00z_storage::{
    checkpoint::recursive_v2::{
        CanonicalCheckpointTransitionV2, RecursiveCircuitProfileV2, RecursiveTraceOpcodeV2,
    },
    checkpoint::{
        derive_exec_tx_root, CheckpointDraft, CheckpointExecInput, CheckpointExecOut,
        CheckpointExecTx, CheckpointExecVersion, CheckpointFsStore, CheckpointId, CheckpointInRef,
        CheckpointStore, CheckpointVersion, CreatedEnt, SpentEnt,
    },
    fixture_support::checkpoint_fixtures,
    settlement::{
        DefinitionId, SerialId, SettlementExecHandoff, SettlementPath, SettlementRouteCtx,
        SettlementStateRoot, SettlementStore, StoreItem, StoreOp, TerminalId, TerminalLeaf,
    },
    snapshot::{build_snapshot_v2, PrepFsStore, PrepSnapshotStore},
};

fn profile() -> RecursiveCircuitProfileV2 {
    RecursiveCircuitProfileV2::repository_fixture()
}

fn path(definition: u8, serial: u32, terminal: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([definition; 32]),
        SerialId::new(serial),
        TerminalId::new([terminal; 32]),
    )
}

fn leaf(path: SettlementPath, value: u64) -> TerminalLeaf {
    let payload = AssetPackPlain {
        value,
        blinding: [3; 32],
        s_out: [4; 32],
    }
    .to_bytes();
    AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        r_pub: [1; 32],
        owner_tag: [2; 32],
        c_amount: [5; 32],
        enc_pack: ZkPackEncrypted {
            version: 1,
            ciphertext: payload,
            tag: [0; 16],
        },
        range_proof: vec![9; 4],
        tag16: 11,
    }
    .into()
}

fn item(path: SettlementPath, value: u64) -> StoreItem {
    StoreItem::new(path, leaf(path, value)).expect("terminal item")
}

fn handoff(input: SettlementPath, output: StoreItem) -> SettlementExecHandoff {
    let tx = CheckpointExecTx::new(
        vec![CheckpointInRef::new(input.terminal_id(), input.serial_id)],
        vec![CheckpointExecOut::new(
            output.path().definition_id,
            output.terminal_leaf().unwrap().clone(),
        )
        .expect("canonical output")],
        vec![8],
    )
    .expect("canonical transaction row");
    SettlementExecHandoff::new(
        SettlementRouteCtx::new([9; 32], 1, 1, [10; 32]),
        vec![StoreOp::Delete(input), StoreOp::Put(Box::new(output))],
        vec![tx],
    )
}

fn canonical_checkpoint(
    root: &std::path::Path,
    pre_settlement_root: SettlementStateRoot,
    post_settlement_root: SettlementStateRoot,
    handoff: &SettlementExecHandoff,
) -> (CheckpointFsStore, PrepFsStore, CheckpointId) {
    let draft = CheckpointDraft::new_settlement(
        CheckpointVersion::CURRENT,
        1,
        pre_settlement_root,
        post_settlement_root,
        vec![SpentEnt::new([0x51; 32])],
        vec![CreatedEnt::new([0x52; 32], [0x53; 32])],
    );
    let (snapshot, snapshot_id) =
        build_snapshot_v2(pre_settlement_root, Vec::new()).expect("prep snapshot");
    let mut prep_store = PrepFsStore::new(root);
    assert_eq!(
        prep_store
            .save_snapshot(&snapshot)
            .expect("persist prep snapshot"),
        snapshot_id
    );
    let exec = CheckpointExecInput::new_settlement(
        CheckpointExecVersion::CURRENT,
        snapshot_id,
        pre_settlement_root,
        handoff.txs().to_vec(),
    )
    .expect("canonical execution input");
    let mut checkpoint_store = CheckpointFsStore::new(root);
    let exec_id = checkpoint_store
        .save_exec_input(&exec)
        .expect("persist execution input");
    let manifest = checkpoint_fixtures::archive_manifest(&draft, &exec, exec_id);
    let da_reference = checkpoint_fixtures::da_reference(&manifest);
    let statement_core = checkpoint_fixtures::statement_core(&exec);
    checkpoint_store
        .stage_publication_contract(exec_id, &statement_core, &manifest, &da_reference)
        .expect("stage canonical checkpoint evidence");
    let link = checkpoint_store
        .seal_artifact(
            &draft,
            draft
                .attest_proof(snapshot_id, exec_id)
                .expect("attested checkpoint proof"),
            snapshot_id,
            exec_id,
        )
        .expect("persist canonical checkpoint artifact and link");
    (checkpoint_store, prep_store, link.checkpoint_id())
}

fn canonical_noop_checkpoint(
    root: &std::path::Path,
    settlement_root: SettlementStateRoot,
) -> (CheckpointFsStore, PrepFsStore, CheckpointId) {
    let draft = CheckpointDraft::new_settlement(
        CheckpointVersion::CURRENT,
        1,
        settlement_root,
        settlement_root,
        Vec::new(),
        Vec::new(),
    );
    let (snapshot, snapshot_id) =
        build_snapshot_v2(settlement_root, Vec::new()).expect("prep snapshot");
    let mut prep_store = PrepFsStore::new(root);
    assert_eq!(
        prep_store
            .save_snapshot(&snapshot)
            .expect("persist prep snapshot"),
        snapshot_id
    );
    let exec = CheckpointExecInput::new_recursive_v2_noop(snapshot_id, settlement_root)
        .expect("typed recursive V2 noop input");
    let mut checkpoint_store = CheckpointFsStore::new(root);
    let exec_id = checkpoint_store
        .save_exec_input(&exec)
        .expect("persist typed noop input");
    let manifest = checkpoint_fixtures::archive_manifest(&draft, &exec, exec_id);
    let da_reference = checkpoint_fixtures::da_reference(&manifest);
    let statement_core = checkpoint_fixtures::statement_core(&exec);
    checkpoint_store
        .stage_publication_contract(exec_id, &statement_core, &manifest, &da_reference)
        .expect("stage canonical checkpoint evidence");
    let link = checkpoint_store
        .seal_artifact(
            &draft,
            draft
                .attest_proof(snapshot_id, exec_id)
                .expect("attested checkpoint proof"),
            snapshot_id,
            exec_id,
        )
        .expect("persist canonical noop artifact and link");
    (checkpoint_store, prep_store, link.checkpoint_id())
}

fn expected_post_root(input: SettlementPath, output: StoreItem) -> SettlementStateRoot {
    let mut expected = SettlementStore::new();
    expected
        .put_settlement_item(item(input, 10))
        .expect("seed expected pre-state");
    expected
        .apply_exec_handoff(handoff(input, output))
        .expect("apply canonical expected handoff");
    expected
        .settlement_root_v2(7)
        .expect("expected V2 post-state root")
}

#[test]
fn recursive_v2_trace_replays_one_real_hjmt_commit_and_binds_the_result() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let mut store = SettlementStore::new();
    let input = path(1, 1, 1);
    store
        .put_settlement_item(item(input, 10))
        .expect("seed pre-state");
    let pre_settlement_root = store.settlement_root_v2(7).expect("V2 pre-state root");
    let next = item(path(2, 2, 2), 20);
    let post_settlement_root = expected_post_root(input, next.clone());
    let expected_exec_root =
        derive_exec_tx_root(handoff(input, next.clone()).txs()).expect("canonical execution root");
    let (checkpoint_store, prep_snapshot_store, checkpoint_id) = canonical_checkpoint(
        temp.path(),
        pre_settlement_root,
        post_settlement_root,
        &handoff(input, next.clone()),
    );
    let mut transition = CanonicalCheckpointTransitionV2::from_exec(
        temp.path(),
        profile(),
        &checkpoint_store,
        &prep_snapshot_store,
        checkpoint_id,
        &mut store,
        handoff(input, next),
    )
    .expect("canonical V2 transition");

    let precommit = transition.precommit();
    assert_ne!(transition.update_trace_digest(), [0; 32]);
    assert!(transition.update_trace_count() > 0);
    let evaluated = transition.evaluate(&store).expect("independent evaluation");
    assert_eq!(evaluated.trace(), precommit);
    assert_eq!(
        evaluated.settlement_root(),
        transition.post_settlement_root()
    );
    assert_eq!(
        evaluated.statement().pre_settlement_root(),
        pre_settlement_root,
        "the typed statement must bind the immutable V2 pre-state root"
    );
    assert_eq!(
        evaluated.statement().post_settlement_root(),
        transition.post_settlement_root(),
        "the typed statement must bind the independently evaluated V2 post-state root"
    );
    assert_ne!(
        evaluated.statement().pre_definition_root(),
        evaluated.statement().post_definition_root(),
        "the typed statement must retain the verified definition-tree transition, not only derived settlement endpoints"
    );
    assert_eq!(evaluated.statement().height(), 1);
    assert_eq!(evaluated.statement().predecessor(), None);
    assert_eq!(evaluated.statement().checkpoint_id(), checkpoint_id);
    assert_eq!(
        evaluated.statement().checkpoint_exec_tx_root(),
        expected_exec_root
    );
    assert_eq!(evaluated.statement().checkpoint_exec_tx_count(), 1);
    assert_ne!(evaluated.statement().checkpoint_statement_digest(), [0; 32]);
    assert_ne!(
        evaluated.statement().checkpoint_statement_core_digest(),
        [0; 32]
    );
    assert_ne!(evaluated.statement().checkpoint_link_digest(), [0; 32]);
    assert_eq!(
        evaluated.statement().trace_digest(),
        precommit.trace_digest(),
        "the typed statement must bind the exact precommitted source trace"
    );
    let declared_counts = evaluated.statement().declared_event_counts();
    assert_eq!(
        declared_counts,
        evaluated.statement().consumed_event_counts(),
        "the independent evaluator must consume exactly the source-declared fixed schedule"
    );
    assert_eq!(
        declared_counts
            .source_record_count()
            .expect("source count is representable"),
        precommit.event_count(),
        "the per-opcode source rows must equal the sealed source-record count"
    );
    assert_eq!(declared_counts.count(RecursiveTraceOpcodeV2::BeginBlock), 1);
    assert_eq!(
        declared_counts.count(RecursiveTraceOpcodeV2::ReplayInput),
        1
    );
    assert_eq!(
        declared_counts.count(RecursiveTraceOpcodeV2::ReplayOutput),
        1
    );
    assert_eq!(
        declared_counts.count(RecursiveTraceOpcodeV2::CommitTypedEvent),
        4,
        "the sole typed-commit lane must contain the four ordered checkpoint-core commitments"
    );
    assert_eq!(
        declared_counts.count(RecursiveTraceOpcodeV2::UniquenessSorted),
        8,
        "each replay identifier must appear as original/sorted in commit and product passes"
    );
    assert_ne!(evaluated.statement().declared_work_digest(), [0; 32]);
    assert_ne!(
        evaluated.statement().pre_uniqueness_context_digest(),
        [0; 32]
    );
    assert!(declared_counts.count(RecursiveTraceOpcodeV2::JmtUpdate) > 0);
    assert!(declared_counts.count(RecursiveTraceOpcodeV2::TraceChunk) > 0);
    assert_eq!(
        evaluated.statement().update_trace_digest(),
        transition.update_trace_digest(),
        "the typed statement must bind the one real JMT update envelope"
    );
    assert_eq!(
        transition.finish(&store).expect("same capability"),
        precommit
    );
}

#[test]
fn recursive_v2_accepts_only_the_authority_defined_empty_noop_transition() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let mut store = SettlementStore::new();
    let pre_root = store.settlement_root_v2(7).expect("V2 pre-state root");
    let generation = store.recursive_v2_storage_generation();
    let (checkpoint_store, prep_snapshot_store, checkpoint_id) =
        canonical_noop_checkpoint(temp.path(), pre_root);

    let mut transition = CanonicalCheckpointTransitionV2::from_exec(
        temp.path(),
        profile(),
        &checkpoint_store,
        &prep_snapshot_store,
        checkpoint_id,
        &mut store,
        SettlementExecHandoff::recursive_v2_noop(),
    )
    .expect("authority-defined noop transition");

    assert_eq!(transition.post_settlement_root(), pre_root);
    assert_eq!(store.recursive_v2_storage_generation(), generation);
    assert_eq!(transition.update_trace_count(), 0);
    let evaluated = transition
        .evaluate(&store)
        .expect("independent noop evaluation");
    assert_eq!(evaluated.settlement_root(), pre_root);
    assert_eq!(evaluated.statement().checkpoint_exec_tx_count(), 0);
    assert_eq!(
        transition.finish(&store).expect("same noop capability"),
        transition.precommit()
    );

    let mut rejected_store = SettlementStore::new();
    let invalid_handoff = SettlementExecHandoff::new(
        SettlementRouteCtx::new([9; 32], 1, 1, [10; 32]),
        Vec::new(),
        Vec::new(),
    );
    assert!(CanonicalCheckpointTransitionV2::from_exec(
        temp.path(),
        profile(),
        &checkpoint_store,
        &prep_snapshot_store,
        checkpoint_id,
        &mut rejected_store,
        invalid_handoff,
    )
    .is_err());
    assert_eq!(
        rejected_store
            .settlement_root_v2(7)
            .expect("unchanged root"),
        pre_root
    );
    assert_eq!(rejected_store.recursive_v2_storage_generation(), generation);
}

#[test]
fn recursive_v2_transition_rejects_handoff_rows_outside_the_checkpoint_exec() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let mut store = SettlementStore::new();
    let input = path(1, 1, 1);
    store
        .put_settlement_item(item(input, 10))
        .expect("seed pre-state");
    let pre_settlement_root = store.settlement_root_v2(7).expect("pre-state root");
    let generation_before = store.recursive_v2_storage_generation();
    let next = item(path(2, 2, 2), 20);
    let post_settlement_root = expected_post_root(input, next.clone());
    let canonical_handoff = handoff(input, next.clone());
    let (checkpoint_store, prep_snapshot_store, checkpoint_id) = canonical_checkpoint(
        temp.path(),
        pre_settlement_root,
        post_settlement_root,
        &canonical_handoff,
    );
    let (route, ops, txs) = handoff(input, next).into_parts();
    let tx = &txs[0];
    let altered_handoff = SettlementExecHandoff::new(
        route,
        ops,
        vec![
            CheckpointExecTx::new(tx.input_refs().to_vec(), tx.outputs().to_vec(), vec![0x99])
                .expect("different but structurally valid execution row"),
        ],
    );

    assert!(CanonicalCheckpointTransitionV2::from_exec(
        temp.path(),
        profile(),
        &checkpoint_store,
        &prep_snapshot_store,
        checkpoint_id,
        &mut store,
        altered_handoff,
    )
    .is_err());
    assert_eq!(
        store.settlement_root_v2(7).expect("root remains pre-state"),
        pre_settlement_root
    );
    assert_eq!(store.recursive_v2_storage_generation(), generation_before);
}

#[test]
fn recursive_v2_transition_rejects_a_stale_pre_state_handle_before_commit() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let mut store = SettlementStore::new();
    let input = path(1, 1, 1);
    store
        .put_settlement_item(item(input, 10))
        .expect("seed pre-state");
    let pre_settlement_root = store.settlement_root_v2(7).expect("pre-state root");
    let next = item(path(2, 2, 2), 20);
    let post_settlement_root = expected_post_root(input, next.clone());
    let (checkpoint_store, prep_snapshot_store, checkpoint_id) = canonical_checkpoint(
        temp.path(),
        pre_settlement_root,
        post_settlement_root,
        &handoff(input, next.clone()),
    );
    store
        .put_settlement_item(item(path(9, 9, 9), 99))
        .expect("advance live storage after capture");

    assert!(CanonicalCheckpointTransitionV2::from_exec(
        temp.path(),
        profile(),
        &checkpoint_store,
        &prep_snapshot_store,
        checkpoint_id,
        &mut store,
        handoff(input, next),
    )
    .is_err());
}

#[test]
fn recursive_v2_preflight_failure_cannot_advance_the_live_store() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let mut store = SettlementStore::new();
    let input = path(1, 1, 1);
    store
        .put_settlement_item(item(input, 10))
        .expect("seed pre-state");
    let root_before = store
        .settlement_root_v2(7)
        .expect("preflight root before rejection");
    let generation_before = store.recursive_v2_storage_generation();
    let next = item(path(2, 2, 2), 20);
    let post_settlement_root = expected_post_root(input, next.clone());
    let (checkpoint_store, prep_snapshot_store, checkpoint_id) = canonical_checkpoint(
        temp.path(),
        root_before,
        post_settlement_root,
        &handoff(input, next.clone()),
    );
    let unavailable_dir = temp.path().join("missing-spool-directory");

    assert!(CanonicalCheckpointTransitionV2::from_exec(
        unavailable_dir,
        profile(),
        &checkpoint_store,
        &prep_snapshot_store,
        checkpoint_id,
        &mut store,
        handoff(input, next),
    )
    .is_err());

    assert_eq!(
        store
            .settlement_root_v2(7)
            .expect("root after rejected preflight"),
        root_before,
        "source construction must fail before the live HJMT is admitted"
    );
    assert_eq!(
        store.recursive_v2_storage_generation(),
        generation_before,
        "a rejected source preflight must not advance durable generation"
    );
}

#[test]
fn recursive_v2_transition_rejects_a_post_commit_generation_change() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let mut store = SettlementStore::new();
    let input = path(1, 1, 1);
    store
        .put_settlement_item(item(input, 10))
        .expect("seed pre-state");
    let pre_settlement_root = store.settlement_root_v2(7).expect("pre-state root");
    let next = item(path(2, 2, 2), 20);
    let post_settlement_root = expected_post_root(input, next.clone());
    let (checkpoint_store, prep_snapshot_store, checkpoint_id) = canonical_checkpoint(
        temp.path(),
        pre_settlement_root,
        post_settlement_root,
        &handoff(input, next.clone()),
    );
    let mut transition = CanonicalCheckpointTransitionV2::from_exec(
        temp.path(),
        profile(),
        &checkpoint_store,
        &prep_snapshot_store,
        checkpoint_id,
        &mut store,
        handoff(input, next),
    )
    .expect("canonical V2 transition");

    store
        .put_settlement_item(item(path(3, 3, 3), 30))
        .expect("concurrent replacement mutation");
    assert!(transition.evaluate(&store).is_err());
    assert!(transition.finish(&store).is_err());
}
