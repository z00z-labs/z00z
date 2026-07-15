use tempfile::TempDir;
use z00z_core::assets::AssetLeaf;

use super::{PrepFsStore, PrepReplayEntry, PrepSnapshotError, PrepSnapshotStore};
use crate::settlement::{
    CheckRoot, DefinitionId, DefinitionRootLeaf, HjmtProofFamily, ProofBlob, ProofItem, SerialId,
    SerialRootLeaf, SettlementPath, SettlementStateRoot, SettlementStore, SnapItem, StoreItem,
    TerminalId, TerminalLeaf,
};
use crate::snapshot::{PrepSnapshot, PrepSnapshotId, PrepSnapshotVersion};

fn path(def_mark: u8, serial_id: u32, asset_mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([def_mark; 32]),
        SerialId::new(serial_id),
        TerminalId::new([asset_mark; 32]),
    )
}

fn leaf(path: &SettlementPath, mark: u8) -> TerminalLeaf {
    let mut leaf = AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        ..AssetLeaf::default()
    };
    leaf.owner_tag[0] = mark;
    leaf.c_amount[0] = mark;
    TerminalLeaf::from(leaf)
}

fn proof_item(path: SettlementPath, leaf: TerminalLeaf) -> ProofItem {
    ProofItem::new_settlement(
        SettlementStateRoot::settlement_v1([9u8; 32]),
        path,
        DefinitionRootLeaf {
            definition_id: path.definition_id,
            definition_root: [3u8; 32],
        },
        SerialRootLeaf {
            definition_id: path.definition_id,
            serial_id: path.serial_id,
            serial_root: [4u8; 32],
        },
        leaf,
    )
    .expect("proof item")
}

fn blob_bytes(item: &ProofItem) -> Vec<u8> {
    ProofBlob::new(
        item.clone(),
        [7u8; 32],
        [8u8; 32],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .encode()
    .expect("proof blob")
}

fn temp_root() -> TempDir {
    TempDir::new().expect("temp dir")
}

fn valid_item(def_mark: u8, serial_num: u32, asset_mark: u8) -> (CheckRoot, SnapItem) {
    let mut store = SettlementStore::test_hjmt_store();
    let path = path(def_mark, serial_num, asset_mark);
    let leaf = leaf(&path, asset_mark);

    store
        .put_settlement_item(StoreItem::new(path, leaf.clone()).expect("store item"))
        .expect("put item");

    let wit = store
        .settlement_proof_blob(&path)
        .expect("proof blob")
        .encode()
        .expect("encode proof blob");
    let root = store.check_root().expect("check root");
    let item = SnapItem::new(path, leaf, wit).expect("snap item");

    (root, item)
}

fn valid_snapshot() -> PrepSnapshot {
    let (root, item) = valid_item(1, 7, 9);
    PrepSnapshot::new(PrepSnapshotVersion::CURRENT, root, vec![item])
}

#[test]
fn test_build_snapshot_returns_id() {
    let snapshot = valid_snapshot();
    let (built, snapshot_id) = super::build_snapshot(snapshot.prev_root, snapshot.entries.clone())
        .expect("build snapshot");

    assert_eq!(built.version, PrepSnapshotVersion::CURRENT);
    assert_eq!(built.prev_root, snapshot.prev_root);
    assert_eq!(built.entries, snapshot.entries);
    assert_ne!(snapshot_id.as_bytes(), &[0u8; 32]);
}

#[test]
fn test_build_snapshot_v2_preserves_the_typed_root_binding() {
    let store = SettlementStore::new();
    let root = store
        .settlement_root_v2(7)
        .expect("derive empty typed V2 root");

    let (snapshot, snapshot_id) =
        super::build_snapshot_v2(root, Vec::new()).expect("build V2-bound snapshot");

    assert_eq!(snapshot.settlement_root_v2(), Some(root));
    assert_eq!(snapshot.prev_root, CheckRoot::from(root));
    assert_ne!(snapshot_id.as_bytes(), &[0u8; 32]);
}

fn snapshot_from_specs(specs: &[(u8, u32, u8)]) -> PrepSnapshot {
    let mut store = SettlementStore::test_hjmt_store();
    let mut rows = Vec::with_capacity(specs.len());

    for &(def_mark, serial_num, asset_mark) in specs {
        let path = path(def_mark, serial_num, asset_mark);
        let leaf = leaf(&path, asset_mark);

        store
            .put_settlement_item(StoreItem::new(path, leaf.clone()).expect("store item"))
            .expect("put item");
        rows.push((path, leaf));
    }

    let root = store.check_root().expect("check root");
    let entries = rows
        .into_iter()
        .map(|(path, leaf)| {
            let wit = store
                .settlement_proof_blob(&path)
                .expect("proof blob")
                .encode()
                .expect("encode proof blob");
            SnapItem::new(path, leaf, wit).expect("snap item")
        })
        .collect();

    PrepSnapshot::new(PrepSnapshotVersion::CURRENT, root, entries)
}

fn rebound_item(
    root: SettlementStateRoot,
    def_mark: u8,
    serial_num: u32,
    asset_mark: u8,
) -> SnapItem {
    let mut store = SettlementStore::test_hjmt_store();
    let path = path(def_mark, serial_num, asset_mark);
    let leaf = leaf(&path, asset_mark);

    store
        .put_settlement_item(StoreItem::new(path, leaf.clone()).expect("store item"))
        .expect("put item");

    let blob = store.settlement_proof_blob(&path).expect("proof blob");
    let rebound = ProofItem::new_settlement(
        root,
        path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        leaf.clone(),
    )
    .expect("rebound proof item");
    let wit = blob
        .rebind(rebound)
        .encode()
        .expect("encode rebound proof blob");

    SnapItem::new(path, leaf, wit).expect("snap item")
}

#[test]
fn test_replay_entry_path_mix() {
    let path_a = path(1, 7, 9);
    let path_b = path(2, 7, 9);
    let leaf_a = leaf(&path_a, 1);
    let leaf_b = leaf(&path_b, 1);
    let proof_item = proof_item(path_b, leaf_b);
    let item = SnapItem::new(path_a, leaf_a, blob_bytes(&proof_item)).expect("snap item");

    let err = PrepReplayEntry::from_blob(item).unwrap_err();

    assert!(matches!(err, PrepSnapshotError::ReplayPathMix));
}

#[test]
fn test_replay_entry_leaf_mix() {
    let path = path(1, 7, 9);
    let leaf_a = leaf(&path, 1);
    let leaf_b = leaf(&path, 2);
    let proof_item = proof_item(path, leaf_b);
    let item = SnapItem::new(path, leaf_a, blob_bytes(&proof_item)).expect("snap item");

    let err = PrepReplayEntry::from_blob(item).unwrap_err();

    assert!(matches!(err, PrepSnapshotError::ReplayLeafMix));
}

#[test]
fn test_validate_snapshot_ok() {
    let root = temp_root();
    let store = PrepFsStore::new(root.path());

    store
        .validate_snapshot(&valid_snapshot())
        .expect("validate snapshot");
}

#[test]
fn test_accept_storage_witness_fixture() {
    let root = temp_root();
    let store = PrepFsStore::new(root.path());
    let snapshot = snapshot_from_specs(&[(1, 7, 9), (2, 8, 10)]);

    let replay = store.replay_entries(&snapshot).expect("replay entries");

    assert_eq!(replay.len(), snapshot.entries.len());

    for (entry, replay_entry) in snapshot.entries.iter().zip(replay.iter()) {
        assert_eq!(replay_entry.item(), entry);
        assert_eq!(
            CheckRoot::from(replay_entry.proof_item().root()),
            snapshot.prev_root
        );
        assert_eq!(replay_entry.proof_item().path(), entry.path());
        assert_eq!(replay_entry.proof_item().leaf(), entry.leaf());
    }
}

#[test]
fn test_validate_snapshot_root_mix() {
    let root = temp_root();
    let store = PrepFsStore::new(root.path());
    let mut snapshot = valid_snapshot();
    snapshot.prev_root = CheckRoot::new([0u8; 32]);

    let err = store.validate_snapshot(&snapshot).unwrap_err();

    assert!(matches!(err, PrepSnapshotError::RootMix));
}

#[test]
fn test_validate_snapshot_path_mix() {
    let root = temp_root();
    let store = PrepFsStore::new(root.path());
    let (root, item) = valid_item(1, 7, 9);
    let mut diff_path = item.path();
    diff_path.definition_id = DefinitionId::new([8u8; 32]);
    let bad =
        SnapItem::new(diff_path, item.leaf().clone(), item.wit().to_vec()).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, root, vec![bad]);

    let err = store.validate_snapshot(&snapshot).unwrap_err();

    assert!(matches!(err, PrepSnapshotError::PathMix));
}

#[test]
fn test_validate_snapshot_serial_mix() {
    let root = temp_root();
    let store = PrepFsStore::new(root.path());
    let (root, item) = valid_item(1, 7, 9);
    let path = SettlementPath::new(
        item.path().definition_id,
        SerialId::new(99),
        item.path().terminal_id(),
    );
    let bad_leaf = leaf(&path, 9);
    let bad = SnapItem::new(path, bad_leaf, item.wit().to_vec()).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, root, vec![bad]);

    let err = store.validate_snapshot(&snapshot).unwrap_err();

    assert!(matches!(err, PrepSnapshotError::SerialMix));
}

#[test]
fn test_validate_asset_id_mix() {
    let root = temp_root();
    let store = PrepFsStore::new(root.path());
    let (root, item) = valid_item(1, 7, 9);
    let path = SettlementPath::new(
        item.path().definition_id,
        item.path().serial_id,
        TerminalId::new([3u8; 32]),
    );
    let bad_leaf = leaf(&path, 3);
    let bad = SnapItem::new(path, bad_leaf, item.wit().to_vec()).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, root, vec![bad]);

    let err = store.validate_snapshot(&snapshot).unwrap_err();

    assert!(matches!(err, PrepSnapshotError::TerminalIdMix));
}

#[test]
fn test_validate_snapshot_leaf_mix() {
    let root = temp_root();
    let store = PrepFsStore::new(root.path());
    let (root, item) = valid_item(1, 7, 9);
    let mut bad_leaf = item.terminal_leaf().expect("asset leaf").clone();
    bad_leaf.owner_tag[0] ^= 1;
    let bad = SnapItem::new(item.path(), bad_leaf, item.wit().to_vec()).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, root, vec![bad]);

    let err = store.validate_snapshot(&snapshot).unwrap_err();

    assert!(matches!(err, PrepSnapshotError::LeafMix));
}

#[test]
fn test_validate_snapshot_decode_mix() {
    let root = temp_root();
    let store = PrepFsStore::new(root.path());
    let (root, item) = valid_item(1, 7, 9);
    let bad = SnapItem::new(item.path(), item.leaf().clone(), vec![1u8, 2, 3]).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, root, vec![bad]);

    let err = store.validate_snapshot(&snapshot).unwrap_err();

    assert!(matches!(err, PrepSnapshotError::WitDecode(_)));
}

#[test]
fn test_rejects_non_incl_witness() {
    let root = temp_root();
    let store = PrepFsStore::new(root.path());
    let (prev_root, item) = valid_item(1, 7, 9);
    let blob = ProofBlob::decode(item.wit()).expect("decode proof blob");
    let bad = SnapItem::new(
        item.path(),
        item.leaf().clone(),
        blob.with_hjmt_proof_family(HjmtProofFamily::Deletion)
            .encode()
            .expect("encode proof blob"),
    )
    .expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, prev_root, vec![bad]);

    let err = store.validate_snapshot(&snapshot).unwrap_err();

    assert!(matches!(err, PrepSnapshotError::WitMix));
}

#[test]
fn test_validate_snapshot_dup_path() {
    let root = temp_root();
    let store = PrepFsStore::new(root.path());
    let snapshot = {
        let (root, item) = valid_item(1, 7, 9);
        PrepSnapshot::new(PrepSnapshotVersion::CURRENT, root, vec![item.clone(), item])
    };

    let err = store.validate_snapshot(&snapshot).unwrap_err();

    assert!(matches!(err, PrepSnapshotError::DupPath));
}

#[test]
fn test_validate_dup_asset_id() {
    let root = temp_root();
    let store = PrepFsStore::new(root.path());
    let asset_root = SettlementStateRoot::settlement_v1([9u8; 32]);
    let snapshot = PrepSnapshot::new(
        PrepSnapshotVersion::CURRENT,
        CheckRoot::from(asset_root),
        vec![
            rebound_item(asset_root, 1, 7, 9),
            rebound_item(asset_root, 2, 8, 9),
        ],
    );

    let err = store.validate_snapshot(&snapshot).unwrap_err();

    assert!(matches!(err, PrepSnapshotError::DupTerminalId));
}

#[test]
fn test_save_load_roundtrip() {
    let root = temp_root();
    let mut store = PrepFsStore::new(root.path());
    let snapshot = valid_snapshot();

    let snapshot_id = store.save_snapshot(&snapshot).expect("save snapshot");
    let loaded = store.load_snapshot(&snapshot_id).expect("load snapshot");

    assert_eq!(loaded, snapshot);
}

#[test]
fn test_load_rejects_key_mix() {
    let root = temp_root();
    let store = PrepFsStore::new(root.path());
    let snapshot = valid_snapshot();
    let bytes = crate::snapshot::codec::encode_snap(&snapshot).expect("encode snapshot");
    let wrong_id = PrepSnapshotId::new([2u8; 32]);

    z00z_utils::io::create_dir_all(store.snapshot_dir()).expect("create snapshot dir");
    z00z_utils::io::write_file(store.snap_path(&wrong_id), &bytes)
        .expect("write wrong-key snapshot");

    let err = store.load_snapshot(&wrong_id).unwrap_err();

    assert!(matches!(err, PrepSnapshotError::IdMix));
}

#[test]
fn test_ordering_identical_input_sets() {
    let specs = [(1, 7, 9), (2, 8, 10)];
    let first = snapshot_from_specs(&specs);
    let second = snapshot_from_specs(&specs);
    let store = PrepFsStore::new(temp_root().path());

    let first_bytes = crate::snapshot::codec::encode_snap(&first).expect("encode first");
    let second_bytes = crate::snapshot::codec::encode_snap(&second).expect("encode second");
    let first_id = store.derive_snapshot_id(&first).expect("first id");
    let second_id = store.derive_snapshot_id(&second).expect("second id");

    assert_eq!(first.entries, second.entries);
    assert_eq!(first_bytes, second_bytes);
    assert_eq!(first_id, second_id);
}

#[test]
fn test_load_rejects_unsupported_version() {
    let snapshot = PrepSnapshot::new(
        PrepSnapshotVersion::new(9),
        CheckRoot::new([5u8; 32]),
        Vec::new(),
    );
    let bytes = crate::snapshot::codec::encode_snap(&snapshot).unwrap_err();

    assert!(matches!(bytes, PrepSnapshotError::VersionMix));
}
