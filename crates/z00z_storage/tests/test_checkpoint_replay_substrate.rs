use serde::Serialize;
use z00z_crypto::{expert::hash_domain, frame_bytes, hash_zk::hash_zk};
use z00z_storage::fixture_support::snapshot_fix;

use z00z_core::assets::AssetLeaf;
use z00z_storage::{
    checkpoint::{
        build_cp_draft, build_stmt_core_v1, derive_delta_root_v1, derive_exec_id, encode_exec_bin,
        CheckpointDraft, CheckpointExecInput, CheckpointExecOut, CheckpointExecTx,
        CheckpointExecVersion, CheckpointInRef, CheckpointLink, CheckpointLinkVersion, CreatedEnt,
        SpentEnt, SpentIndex, SpentIndexError, StateError, TxPkgSum, TxProofError, TxProofVerifier,
    },
    settlement::{
        derive_journal_digest_v1, derive_witness_root_v1, BatchProofBlobV1, CheckRoot,
        DefinitionId, SerialId, SettlementPath, SettlementStateRoot, SettlementStore, StoreItem,
        StoreOp, TerminalLeaf,
    },
    snapshot::{PrepSnapshot, PrepSnapshotStore},
};

hash_domain!(TestCheckpointReplayDom, "z00z.storage.checkpoint.replay", 1);

const DELTA_RECORD_LABEL: &str = "checkpoint_delta_record_v1";
const DELTA_ROOT_LABEL: &str = "checkpoint_delta_root_v1";
const DELTA_SPENT_LABEL: &str = "checkpoint_delta_spent_v1";
const DELTA_CREATED_LABEL: &str = "checkpoint_delta_created_v1";

struct NoSpent;

impl SpentIndex for NoSpent {
    fn is_spent(
        &self,
        _prev: CheckRoot,
        _curr: CheckRoot,
        _id: &z00z_storage::settlement::TerminalId,
    ) -> Result<bool, SpentIndexError> {
        Ok(false)
    }
}

struct PassProof;

impl TxProofVerifier for PassProof {
    fn verify_tx(&self, tx: &TxPkgSum) -> Result<(), TxProofError> {
        if tx.tx_proof.is_empty() {
            return Err(TxProofError::Invalid);
        }
        Ok(())
    }
}

fn exec(
    snapshot_id: z00z_storage::snapshot::PrepSnapshotId,
    prev_root: CheckRoot,
) -> CheckpointExecInput {
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        snapshot_id,
        prev_root,
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([1u8; 32], SerialId::new(1))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([7u8; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(11)),
            )
            .expect("exec out")],
            vec![9u8, 7u8, 5u8],
        )
        .expect("exec tx")],
    )
    .expect("exec input")
}

#[derive(Serialize)]
struct CheckpointDeltaRecordV1 {
    version: u8,
    tx_ordinal: u32,
    item_ordinal: u32,
    delta_kind: u8,
    terminal_id: z00z_storage::settlement::TerminalId,
    payload_digest: [u8; 32],
}

impl CheckpointDeltaRecordV1 {
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&frame_bytes(&[self.version]));
        bytes.extend_from_slice(&frame_bytes(&self.tx_ordinal.to_le_bytes()));
        bytes.extend_from_slice(&frame_bytes(&self.item_ordinal.to_le_bytes()));
        bytes.extend_from_slice(&frame_bytes(&[self.delta_kind]));
        bytes.extend_from_slice(&frame_bytes(self.terminal_id.as_bytes()));
        bytes.extend_from_slice(&frame_bytes(&self.payload_digest));
        bytes
    }
}

fn spent_payload_bytes(spent: &SpentEnt) -> Vec<u8> {
    frame_bytes(spent.terminal_id().as_bytes())
}

fn created_payload_bytes(created: &CreatedEnt) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&frame_bytes(created.terminal_id().as_bytes()));
    bytes.extend_from_slice(&frame_bytes(created.leaf_hash()));
    bytes
}

fn seed_store(snapshot: &PrepSnapshot) -> SettlementStore {
    let mut store = SettlementStore::new();
    for entry in &snapshot.entries {
        let Ok(leaf) = entry.terminal_leaf() else {
            continue;
        };
        let item = StoreItem::new(entry.path(), leaf.clone()).expect("store item");
        store.put_settlement_item(item).expect("put snapshot item");
    }
    store
}

fn output_path(output: &CheckpointExecOut) -> SettlementPath {
    SettlementPath::new(
        output.definition_id(),
        SerialId::new(output.leaf().serial_id),
        output.leaf().terminal_id(),
    )
}

fn post_state_batch(snapshot: &PrepSnapshot, exec: &CheckpointExecInput) -> BatchProofBlobV1 {
    let mut store = seed_store(snapshot);
    let mut ops = Vec::new();
    let mut output_paths = Vec::new();

    for tx in exec.txs() {
        for input_ref in tx.input_refs() {
            let path = snapshot
                .entries
                .iter()
                .find(|entry| {
                    entry.path().terminal_id() == input_ref.terminal_id()
                        && entry.path().serial_id == input_ref.serial_id()
                })
                .map(|entry| entry.path())
                .expect("input path must exist in snapshot");
            ops.push(StoreOp::Delete(path));
        }
        for output in tx.outputs() {
            let path = output_path(output);
            output_paths.push(path);
            ops.push(StoreOp::Put(Box::new(
                StoreItem::new(path, output.leaf().clone()).expect("output item"),
            )));
        }
    }

    store.apply_settlement_ops(ops).expect("apply exec ops");
    store
        .settlement_inclusion_batch_v1(&output_paths)
        .expect("live inclusion batch")
}

fn manual_delta_root(exec: &CheckpointExecInput, draft: &CheckpointDraft) -> [u8; 32] {
    let record_count = draft.spent_delta().len() + draft.created_delta().len();
    let mut root_bytes = frame_bytes(&(record_count as u32).to_le_bytes());
    let mut spent_idx = 0usize;
    let mut created_idx = 0usize;

    for (tx_ordinal, tx) in exec.txs().iter().enumerate() {
        for (item_ordinal, _input_ref) in tx.input_refs().iter().enumerate() {
            let spent: &SpentEnt = &draft.spent_delta()[spent_idx];
            let spent_bytes = spent_payload_bytes(spent);
            let payload_digest =
                hash_zk::<TestCheckpointReplayDom>(DELTA_SPENT_LABEL, &[spent_bytes.as_slice()]);
            let record = CheckpointDeltaRecordV1 {
                version: 1,
                tx_ordinal: tx_ordinal as u32,
                item_ordinal: item_ordinal as u32,
                delta_kind: 1,
                terminal_id: spent.terminal_id(),
                payload_digest,
            };
            let record_bytes = record.canonical_bytes();
            let record_hash =
                hash_zk::<TestCheckpointReplayDom>(DELTA_RECORD_LABEL, &[record_bytes.as_slice()]);
            root_bytes.extend_from_slice(&frame_bytes(&record_hash));
            spent_idx += 1;
        }

        for (item_ordinal, _output) in tx.outputs().iter().enumerate() {
            let created: &CreatedEnt = &draft.created_delta()[created_idx];
            let created_bytes = created_payload_bytes(created);
            let payload_digest = hash_zk::<TestCheckpointReplayDom>(
                DELTA_CREATED_LABEL,
                &[created_bytes.as_slice()],
            );
            let record = CheckpointDeltaRecordV1 {
                version: 1,
                tx_ordinal: tx_ordinal as u32,
                item_ordinal: item_ordinal as u32,
                delta_kind: 2,
                terminal_id: created.terminal_id(),
                payload_digest,
            };
            let record_bytes = record.canonical_bytes();
            let record_hash =
                hash_zk::<TestCheckpointReplayDom>(DELTA_RECORD_LABEL, &[record_bytes.as_slice()]);
            root_bytes.extend_from_slice(&frame_bytes(&record_hash));
            created_idx += 1;
        }
    }

    hash_zk::<TestCheckpointReplayDom>(DELTA_ROOT_LABEL, &[root_bytes.as_slice()])
}

#[test]
fn test_build_cp_draft_preserves_replay_substrate_roots_and_deltas() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let replay = store.replay_entries(&loaded).expect("replay");
    let exec = exec(snap_id, loaded.prev_root);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes")),
    )
    .expect("link");

    let draft = build_cp_draft(
        7, snap_id, &loaded, &replay, &link, &exec, &PassProof, &NoSpent,
    )
    .expect("draft");

    assert_eq!(draft.prev_root(), loaded.prev_root);
    assert_eq!(
        draft.prev_settlement_root(),
        SettlementStateRoot::settlement_v1(loaded.prev_root.into_bytes())
    );
    assert_ne!(draft.new_root(), draft.prev_root());
    assert_eq!(draft.spent_delta().len(), 1);
    assert_eq!(draft.created_delta().len(), 1);
}

#[test]
fn test_build_cp_draft_rejects_serial_mismatch_in_replay_substrate() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let replay = store.replay_entries(&loaded).expect("replay");
    let exec = CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        snap_id,
        loaded.prev_root,
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([1u8; 32], SerialId::new(9))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([7u8; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(11)),
            )
            .expect("exec out")],
            vec![9u8, 7u8, 5u8],
        )
        .expect("exec tx")],
    )
    .expect("exec input");
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes")),
    )
    .expect("link");

    let err = build_cp_draft(
        7, snap_id, &loaded, &replay, &link, &exec, &PassProof, &NoSpent,
    )
    .expect_err("serial mismatch must reject");

    assert!(matches!(err, StateError::LeafMatch));
}

#[test]
fn test_build_cp_draft_rejects_missing_replay_input() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let replay = store.replay_entries(&loaded).expect("replay");
    let exec = CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        snap_id,
        loaded.prev_root,
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([0xAA; 32], SerialId::new(1))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([7u8; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(11)),
            )
            .expect("exec out")],
            vec![9u8, 7u8, 5u8],
        )
        .expect("exec tx")],
    )
    .expect("exec input");
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes")),
    )
    .expect("link");

    let err = build_cp_draft(
        7, snap_id, &loaded, &replay, &link, &exec, &PassProof, &NoSpent,
    )
    .expect_err("missing input must reject");

    assert!(matches!(err, StateError::MissingInput));
}

#[test]
fn test_build_stmt_core_v1_binds_live_delta_witness_and_journal() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let replay = store.replay_entries(&loaded).expect("replay");
    let exec = exec(snap_id, loaded.prev_root);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes")),
    )
    .expect("link");
    let draft = build_cp_draft(
        7, snap_id, &loaded, &replay, &link, &exec, &PassProof, &NoSpent,
    )
    .expect("draft");
    let batch = post_state_batch(&snapshot, &exec);

    let core = build_stmt_core_v1(&draft, &exec, std::slice::from_ref(&batch)).expect("core");

    assert_eq!(core.tx_data_root(), exec.tx_data_root());
    assert_eq!(core.delta_root(), manual_delta_root(&exec, &draft));
    assert_eq!(
        core.delta_root(),
        derive_delta_root_v1(&exec, &draft).expect("delta root")
    );
    assert_eq!(
        core.witness_root(),
        derive_witness_root_v1(std::slice::from_ref(&batch)).expect("witness root")
    );
    assert_eq!(
        core.journal_digest(),
        derive_journal_digest_v1(std::slice::from_ref(&batch)).expect("journal digest")
    );
}

#[test]
fn test_build_stmt_core_v1_rejects_root_mismatch_between_draft_and_batch() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let replay = store.replay_entries(&loaded).expect("replay");
    let exec = exec(snap_id, loaded.prev_root);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes")),
    )
    .expect("link");
    let draft = build_cp_draft(
        7, snap_id, &loaded, &replay, &link, &exec, &PassProof, &NoSpent,
    )
    .expect("draft");
    let prestate_store = seed_store(&snapshot);
    let prestate_batch = prestate_store
        .settlement_inclusion_batch_v1(&[snapshot_fix::test_path(1, 1, 1)])
        .expect("prestate batch");

    let err = build_stmt_core_v1(&draft, &exec, std::slice::from_ref(&prestate_batch))
        .expect_err("mismatched settlement root must reject");

    assert!(matches!(err, z00z_storage::CheckpointError::RootMix));
}

#[test]
fn test_build_stmt_core_v1_rejects_delta_drift() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let replay = store.replay_entries(&loaded).expect("replay");
    let exec = exec(snap_id, loaded.prev_root);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes")),
    )
    .expect("link");
    let draft = build_cp_draft(
        7, snap_id, &loaded, &replay, &link, &exec, &PassProof, &NoSpent,
    )
    .expect("draft");
    let batch = post_state_batch(&snapshot, &exec);
    let drifted = CheckpointDraft::new_settlement(
        draft.version(),
        draft.height(),
        draft.prev_settlement_root(),
        draft.new_settlement_root(),
        vec![SpentEnt::new([0xAA; 32])],
        draft.created_delta().to_vec(),
    );

    let err = build_stmt_core_v1(&drifted, &exec, std::slice::from_ref(&batch))
        .expect_err("delta drift must reject");

    assert!(matches!(err, z00z_storage::CheckpointError::ReplayMix));
}
