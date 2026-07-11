use super::bundle_lane_impl::{
    calc_prev_root, demo_spent_key, Checkpoint, DemoCheckpoint, FragTx, MadeEnt, PrepIdx,
};
use super::demo_checkpoint_agg::{build_demo_cp, build_demo_frag};
use super::fragment_construction::{build_target_frag, hash_leaf};
use std::collections::HashSet;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use z00z_core::{assets::AssetPkgWire, genesis::asset_std::asset_from_dev_class, AssetClass};
use z00z_crypto::expert::encoding::to_hex;
use z00z_storage::{
    settlement::{
        proof_blob_rebind_root, CheckRoot, DefinitionId, SerialId, SettlementPath,
        SettlementStateRoot, SettlementStore, SnapItem, StoreItem, TerminalId,
    },
    snapshot::{build_snapshot, PrepFsStore, PrepReplayEntry, PrepSnapshot, PrepSnapshotStore},
};
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_wallets::tx::{InputResolver, StateError, TxInputWire, TxOutRole, TxOutputWire};

static NEXT_DIR: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy)]
struct TestPrepSpec {
    asset_id: [u8; 32],
    definition_id: [u8; 32],
    serial_id: u32,
}

fn prep_spec(asset_id: [u8; 32], definition_id: [u8; 32], serial_id: u32) -> TestPrepSpec {
    TestPrepSpec {
        asset_id,
        definition_id,
        serial_id,
    }
}

fn prep_path(spec: TestPrepSpec) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(spec.definition_id),
        SerialId::new(spec.serial_id),
        TerminalId::new(spec.asset_id),
    )
}

fn prep_leaf(spec: TestPrepSpec) -> z00z_core::assets::AssetLeaf {
    z00z_core::assets::AssetLeaf {
        asset_id: spec.asset_id,
        serial_id: spec.serial_id,
        ..z00z_core::assets::AssetLeaf::default()
    }
}

fn blob_root_with_rebound_root(wit: &[u8], root: [u8; 32]) -> Vec<u8> {
    proof_blob_rebind_root(wit, SettlementStateRoot::settlement_v1(root)).expect("blob root rebind")
}

fn prep_snapshot(specs: &[TestPrepSpec]) -> PrepSnapshot {
    let mut store = SettlementStore::new();
    let rows = specs
        .iter()
        .map(|spec| {
            let path = prep_path(*spec);
            let leaf = prep_leaf(*spec);
            store
                .put_settlement_item(
                    StoreItem::new(
                        path,
                        z00z_storage::settlement::TerminalLeaf::from(leaf.clone()),
                    )
                    .expect("store item"),
                )
                .expect("put item");
            (path, leaf)
        })
        .collect::<Vec<_>>();
    let prev_root = CheckRoot::from(store.settlement_root().expect("prep root"));
    let entries = rows
        .into_iter()
        .map(|(path, leaf)| {
            let wit = store
                .settlement_proof_blob(&path)
                .expect("proof blob")
                .encode()
                .expect("proof blob encode");
            SnapItem::new(
                path,
                z00z_storage::settlement::TerminalLeaf::from(leaf),
                wit,
            )
            .expect("snap item")
        })
        .collect::<Vec<_>>();
    build_snapshot(prev_root, entries).expect("snapshot").0
}

fn replay_entries(snapshot: &PrepSnapshot) -> Vec<PrepReplayEntry> {
    PrepFsStore::new(fixture_dir("replay"))
        .replay_entries(snapshot)
        .expect("replay")
}

fn fixture_dir(tag: &str) -> PathBuf {
    let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target/test-prep/stage6")
        .join(format!("{tag}_{id}"));
    z00z_utils::io::create_dir_all(&dir).expect("create test dir");
    dir
}

fn out_wire(serial_id: u32, amount: u64) -> TxOutputWire {
    let asset = asset_from_dev_class(AssetClass::Coin, serial_id, amount).expect("asset");
    TxOutputWire {
        role: TxOutRole::Recipient,
        asset_wire: AssetPkgWire::from_asset(&asset),
    }
}

fn in_ref(asset_id_hex: String, serial_id: u32) -> TxInputWire {
    TxInputWire {
        asset_id_hex,
        serial_id,
    }
}

fn fixture() -> (String, Vec<FragTx>) {
    let prev_root_hex = to_hex(&calc_prev_root(6));
    let frags = vec![
        build_demo_frag(6, 1, 100, &prev_root_hex),
        build_demo_frag(6, 2, 150, &prev_root_hex),
    ];
    (prev_root_hex, frags)
}

#[test]
fn test_cp_ok() {
    let (prev_root_hex, frags) = fixture();
    let cp = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).expect("cp");
    assert_eq!(cp.fragment_ids.len(), 2);
    assert_eq!(cp.spent_delta.len(), 2);
    assert_eq!(cp.created_delta.len(), 2);
    assert_eq!(cp.spent_delta[0], frags[0].inputs[0].asset_id_hex);
    assert_eq!(cp.spent_delta[1], frags[1].inputs[0].asset_id_hex);
    assert!(cp.spent_delta.iter().all(|item| !item.contains(':')));
    assert_eq!(
        cp.created_delta[0].asset_id_hex,
        frags[0].outputs[0].asset_id_hex
    );
    assert_eq!(
        cp.created_delta[1].asset_id_hex,
        frags[1].outputs[0].asset_id_hex
    );
    assert!(!cp.demo_digest_hex.is_empty());
}

#[test]
fn test_reject_stale_root() {
    let (prev_root_hex, mut frags) = fixture();
    frags[0].prev_root_hex = "deadbeef".to_string();
    let err = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).unwrap_err();
    assert!(err.contains("stale root"));
}

#[test]
fn test_reject_bad_member() {
    let (prev_root_hex, mut frags) = fixture();
    frags[1].inputs[0].member_ok = false;
    let err = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).unwrap_err();
    assert!(err.contains("membership witness failed"));
}

#[test]
fn test_demo_dup_ref() {
    let (prev_root_hex, mut frags) = fixture();
    frags[1].inputs[0].asset_id_hex = frags[0].inputs[0].asset_id_hex.clone();
    frags[1].inputs[0].serial_id = frags[0].inputs[0].serial_id;
    let err = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).unwrap_err();
    assert!(err.contains("double spend"));
}

#[test]
fn test_demo_prev_spent() {
    let (prev_root_hex, frags) = fixture();
    let mut spent_prev = HashSet::new();
    spent_prev.insert(demo_spent_key(&frags[0].inputs[0]));

    let err = build_demo_cp(&prev_root_hex, &frags, &spent_prev).unwrap_err();
    assert!(err.contains("double spend"));
}

#[test]
fn test_demo_dup_asset() {
    let (prev_root_hex, mut frags) = fixture();
    frags[1].inputs[0].asset_id_hex = frags[0].inputs[0].asset_id_hex.clone();
    frags[1].inputs[0].serial_id = frags[0].inputs[0].serial_id.saturating_add(1);

    let err = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).unwrap_err();
    assert!(err.contains("double spend"));
}

#[test]
fn test_reject_dup_frag() {
    let (prev_root_hex, mut frags) = fixture();
    frags[1].id = frags[0].id.clone();
    let err = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).unwrap_err();
    assert!(err.contains("duplicate fragment id"));
}

#[test]
fn test_reject_bad_input() {
    let (prev_root_hex, mut frags) = fixture();
    frags[0].inputs[0].asset_id_hex = "gg".to_string();
    let err = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).unwrap_err();
    assert!(err.contains("invalid input asset hex"));
}

#[test]
fn test_reject_no_inputs() {
    let (prev_root_hex, mut frags) = fixture();
    frags[0].inputs.clear();
    let err = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).unwrap_err();
    assert!(err.contains("no inputs"));
}

#[test]
fn test_replay_same_order() {
    let (prev_root_hex, frags) = fixture();
    let cp1: DemoCheckpoint = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).expect("cp1");
    let cp2: DemoCheckpoint = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).expect("cp2");
    assert_eq!(cp1.demo_digest_hex, cp2.demo_digest_hex);
}

#[test]
fn test_demo_expected_digest() {
    let (prev_root_hex, frags) = fixture();
    let cp = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).expect("cp");
    assert_eq!(
        cp.demo_digest_hex,
        "75dfd626092256924af188b3811c7de2c327f18c6d66a903c3a0271acd220564"
    );
}

#[test]
fn test_batch_digest_stable() {
    let prev_root_hex = to_hex(&calc_prev_root(6));
    let mut frags = Vec::new();
    for idx in 1..=16u32 {
        frags.push(build_demo_frag(6, idx, 10 + idx as u64, &prev_root_hex));
    }

    let cp1 = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).expect("cp1");
    let cp2 = build_demo_cp(&prev_root_hex, &frags, &HashSet::new()).expect("cp2");

    assert_eq!(cp1.demo_digest_hex, cp2.demo_digest_hex);
    assert_eq!(cp1.spent_delta.len(), 16);
    assert_eq!(cp1.created_delta.len(), 16);
}

#[test]
fn test_frag_uses_input() {
    let spec = prep_spec([0x11; 32], [0x21; 32], 7);
    let snapshot = prep_snapshot(&[spec]);
    let prev_root_hex = to_hex(snapshot.prev_root.as_bytes());
    let input = in_ref("11".repeat(32), 7);
    let output = out_wire(12, 55);

    let frag =
        build_target_frag(1, &prev_root_hex, &snapshot.entries, &input, &output).expect("frag");

    assert_eq!(frag.inputs[0].asset_id_hex, "11".repeat(32));
    assert_eq!(frag.inputs[0].serial_id, 7);
    assert_eq!(
        frag.inputs[0].leaf_hash_hex,
        to_hex(&hash_leaf(&prep_leaf(spec).into()))
    );
    assert_eq!(frag.outputs[0].asset_id_hex.len(), 64);
    assert_eq!(frag.outputs[0].amount, 55);
}

#[test]
fn test_reject_leaf_miss() {
    let snapshot = prep_snapshot(&[prep_spec([0x11; 32], [0x21; 32], 7)]);
    let prev_root_hex = to_hex(snapshot.prev_root.as_bytes());
    let input = in_ref("11".repeat(32), 8);
    let output = out_wire(12, 55);

    let err = build_target_frag(1, &prev_root_hex, &snapshot.entries, &input, &output).unwrap_err();
    assert!(err.contains("leaf_match mismatch"));
}

#[test]
fn test_frag_hex_case() {
    let snapshot = prep_snapshot(&[prep_spec([0x11; 32], [0x21; 32], 7)]);
    let prev_root_hex = to_hex(snapshot.prev_root.as_bytes());
    let input = in_ref("11".repeat(32).to_uppercase(), 7);
    let output = out_wire(12, 55);

    let frag =
        build_target_frag(1, &prev_root_hex, &snapshot.entries, &input, &output).expect("frag");

    assert_eq!(frag.inputs[0].serial_id, 7);
}

#[test]
fn test_cp_roundtrip() {
    let cp = Checkpoint {
        prev_root_hex: "11".repeat(32),
        new_root_hex: "22".repeat(32),
        spent_delta: vec!["33".repeat(32)],
        created_delta: vec![MadeEnt {
            asset_id_hex: "44".repeat(32),
            leaf_hash_hex: "55".repeat(32),
        }],
        fragment_ids: vec!["frag_1".to_string()],
    };

    let json = JsonCodec.serialize(&cp).expect("json");
    let back: Checkpoint = JsonCodec.deserialize(&json).expect("back");

    assert_eq!(back, cp);
}

#[test]
fn test_snapshot_rejects_root_mix() {
    let mut snapshot = prep_snapshot(&[prep_spec([0x31; 32], [0x12; 32], 9)]);
    let item = snapshot.entries[0].clone();
    let bad_wit = blob_root_with_rebound_root(item.wit(), [0x55; 32]);
    snapshot.entries[0] =
        SnapItem::new(item.path(), item.leaf().clone(), bad_wit).expect("snap item");

    let err = build_snapshot(snapshot.prev_root, snapshot.entries.clone()).expect_err("root mix");
    assert!(err.to_string().contains("root"), "unexpected error: {err}");
}

#[test]
fn test_replay_rejects_path_mix() {
    let mut snapshot = prep_snapshot(&[prep_spec([0x32; 32], [0x13; 32], 10)]);
    let item = snapshot.entries[0].clone();
    let bad_path = SettlementPath::new(
        DefinitionId::new([0x77; 32]),
        item.path().serial_id,
        item.path().terminal_id,
    );
    snapshot.entries[0] =
        SnapItem::new(bad_path, item.leaf().clone(), item.wit().to_vec()).expect("snap item");

    let err = PrepFsStore::new(fixture_dir("path_mix"))
        .replay_entries(&snapshot)
        .expect_err("path mix");
    assert!(err.to_string().contains("path"), "unexpected error: {err}");
}

#[test]
fn test_prep_idx_lookup_root() {
    let spec = prep_spec([0x33; 32], [0x14; 32], 11);
    let snapshot = prep_snapshot(&[spec]);
    let replay = replay_entries(&snapshot);
    let idx = PrepIdx::new(snapshot.prev_root, &replay).expect("prep idx");
    let wrong_root = CheckRoot::new([0x99; 32]);

    let err = idx
        .resolve(wrong_root, spec.asset_id.into(), 11)
        .expect_err("root mismatch");
    assert_eq!(err, StateError::PrevRoot);
}
