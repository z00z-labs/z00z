use z00z_storage::fixture_support::snapshot_fix;
use z00z_utils::codec::{Codec, JsonCodec};

use serde_json::Value;
use z00z_core::assets::AssetLeaf;
use z00z_storage::{
    checkpoint::{
        check_exec_root, check_link_ids, decode_exec_bin, decode_exec_json, derive_exec_id,
        derive_exec_tx_root, encode_exec_bin, encode_exec_json, CheckpointExecInput,
        CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion, CheckpointInRef,
        CheckpointLink, CheckpointLinkVersion,
    },
    settlement::{CheckRoot, DefinitionId, SerialId, TerminalLeaf},
    snapshot::{PrepSnapshotId, PrepSnapshotStore},
    CheckpointError,
};

fn exec_with_proof(
    snapshot_id: PrepSnapshotId,
    prev_root: CheckRoot,
    tx_proof: Vec<u8>,
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
            tx_proof,
        )
        .expect("exec tx")],
    )
    .expect("exec input")
}

fn exec(snapshot_id: PrepSnapshotId, prev_root: CheckRoot) -> CheckpointExecInput {
    exec_with_proof(snapshot_id, prev_root, vec![9u8, 7u8, 5u8])
}

fn exec_pair(snapshot_id: PrepSnapshotId, prev_root: CheckRoot) -> CheckpointExecInput {
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        snapshot_id,
        prev_root,
        vec![
            CheckpointExecTx::new(
                vec![CheckpointInRef::new([1u8; 32], SerialId::new(1))],
                vec![CheckpointExecOut::new(
                    DefinitionId::new([7u8; 32]),
                    TerminalLeaf::from(AssetLeaf::dummy_for_scan(11)),
                )
                .expect("exec out a")],
                vec![1u8, 2u8, 3u8],
            )
            .expect("exec tx a"),
            CheckpointExecTx::new(
                vec![CheckpointInRef::new([2u8; 32], SerialId::new(2))],
                vec![CheckpointExecOut::new(
                    DefinitionId::new([8u8; 32]),
                    TerminalLeaf::from(AssetLeaf::dummy_for_scan(12)),
                )
                .expect("exec out b")],
                vec![4u8, 5u8, 6u8],
            )
            .expect("exec tx b"),
        ],
    )
    .expect("exec input pair")
}

#[test]
fn test_preserves_exact_proof_bytes() {
    let tx_proof = b"verified-proof-v1".to_vec();
    let exec = exec_with_proof(
        PrepSnapshotId::new([2u8; 32]),
        CheckRoot::new([3u8; 32]),
        tx_proof.clone(),
    );

    let decoded = decode_exec_bin(&encode_exec_bin(&exec).expect("exec bytes")).expect("exec");

    assert_eq!(decoded.txs()[0].tx_proof(), tx_proof.as_slice());
    assert_eq!(decoded.tx_data_root(), exec.tx_data_root());
    assert_eq!(
        decoded.tx_data_root(),
        derive_exec_tx_root(decoded.txs()).expect("derived root")
    );
}

#[test]
fn test_tx_data_root_mismatch_rejects() {
    let exec = exec(PrepSnapshotId::new([2u8; 32]), CheckRoot::new([3u8; 32]));
    let codec = JsonCodec;
    let mut value = codec
        .deserialize::<Value>(&encode_exec_json(&exec).expect("exec json"))
        .expect("json value");
    value["tx_data_root"] = Value::Array((0..32).map(|_| Value::from(0)).collect());

    let err = decode_exec_json(&codec.serialize_pretty(&value).expect("mutated json"))
        .expect_err("wrong tx_data_root must reject");

    assert!(matches!(err, CheckpointError::ReplayMix));
}

#[test]
fn test_tx_row_reorder_rejects() {
    let exec = exec_pair(PrepSnapshotId::new([2u8; 32]), CheckRoot::new([3u8; 32]));
    let codec = JsonCodec;
    let mut value = codec
        .deserialize::<Value>(&encode_exec_json(&exec).expect("exec json"))
        .expect("json value");
    let txs = value["txs"].as_array_mut().expect("tx rows");
    txs.swap(0, 1);

    let err = decode_exec_json(&codec.serialize_pretty(&value).expect("mutated json"))
        .expect_err("reordered tx rows must reject");

    assert!(matches!(err, CheckpointError::ReplayMix));
}

#[test]
fn test_proof_byte_tamper_rejects() {
    let exec = exec(PrepSnapshotId::new([2u8; 32]), CheckRoot::new([3u8; 32]));
    let codec = JsonCodec;
    let mut value = codec
        .deserialize::<Value>(&encode_exec_json(&exec).expect("exec json"))
        .expect("json value");
    value["txs"][0]["tx_proof"] =
        Value::Array(vec![Value::from(0), Value::from(1), Value::from(2)]);

    let err = decode_exec_json(&codec.serialize_pretty(&value).expect("mutated json"))
        .expect_err("tampered proof bytes must reject");

    assert!(matches!(err, CheckpointError::ReplayMix));
}

#[test]
fn test_snap_id_mismatch_rejects() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let exec = exec(PrepSnapshotId::new([8u8; 32]), loaded.prev_root);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes")),
    )
    .expect("link");

    let err = check_link_ids(snap_id, &link, &exec).expect_err("snap id mismatch");

    assert!(matches!(err, CheckpointError::LinkMix));
}

#[test]
fn test_exec_id_mismatch_rejects() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let exec = exec(snap_id, loaded.prev_root);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        z00z_storage::checkpoint::CheckpointExecInputId::new([0u8; 32]),
    )
    .expect("link");

    let err = check_link_ids(snap_id, &link, &exec).expect_err("exec id mismatch");

    assert!(matches!(err, CheckpointError::ReplayMix));
}

#[test]
fn test_prev_root_mismatch_rejects() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let exec = exec(snap_id, CheckRoot::new([9u8; 32]));

    let err = check_exec_root(&loaded, &exec).expect_err("root mismatch");

    assert!(matches!(err, CheckpointError::RootMix));
}
