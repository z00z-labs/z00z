use std::path::PathBuf;

use crate::{
    settlement::{
        CheckRoot, DefinitionId, ProofBlob, ProofItem, SerialId, SettlementPath,
        SettlementStateRoot, SettlementStore, SnapItem, StoreItem, TerminalId, TerminalLeaf,
    },
    snapshot::{PrepFsStore, PrepSnapshot, PrepSnapshotId, PrepSnapshotStore, PrepSnapshotVersion},
};
use tempfile::TempDir;
use z00z_core::assets::AssetLeaf;
use z00z_utils::codec::{BincodeCodec, Codec};

pub fn test_path(def_mark: u8, serial_num: u32, asset_mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([def_mark; 32]),
        SerialId::new(serial_num),
        TerminalId::new([asset_mark; 32]),
    )
}

pub fn test_leaf(path: SettlementPath, mark: u8) -> TerminalLeaf {
    let mut leaf = AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        ..AssetLeaf::default()
    };
    leaf.owner_tag[0] = mark;
    leaf.c_amount[0] = mark;
    TerminalLeaf::from(leaf)
}

pub fn snap(specs: &[(u8, u32, u8)]) -> PrepSnapshot {
    let mut store = SettlementStore::new();
    let mut rows = Vec::with_capacity(specs.len());

    for &(def_mark, serial_num, asset_mark) in specs {
        let path = test_path(def_mark, serial_num, asset_mark);
        let leaf = test_leaf(path, asset_mark);
        let item = StoreItem::new(path, leaf.clone()).expect("store item");
        store.put_settlement_item(item).expect("put item");
        rows.push((path, leaf));
    }

    let root = CheckRoot::from(store.settlement_root().expect("check root"));
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

pub fn dup_item(
    root: SettlementStateRoot,
    def_mark: u8,
    serial_num: u32,
    asset_mark: u8,
) -> SnapItem {
    let mut store = SettlementStore::new();
    let path = test_path(def_mark, serial_num, asset_mark);
    let leaf = test_leaf(path, asset_mark);

    store
        .put_settlement_item(StoreItem::new(path, leaf.clone()).expect("store item"))
        .expect("put item");

    let blob = store.settlement_proof_blob(&path).expect("proof blob");
    let item = ProofItem::new_settlement(
        root,
        path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        leaf.clone(),
    )
    .expect("proof item");
    let wit = blob.rebind(item).encode().expect("encode proof blob");

    SnapItem::new(path, leaf, wit).expect("snap item")
}

pub fn hash_mix(item: &SnapItem) -> SnapItem {
    let blob = ProofBlob::decode(item.wit()).expect("decode blob");
    let wit = blob
        .with_terminal_leaf_hash([0u8; 32])
        .encode()
        .expect("encode bad hash blob");

    SnapItem::new(item.path(), item.leaf().clone(), wit).expect("snap item")
}

pub fn bytes(snapshot: &PrepSnapshot) -> Vec<u8> {
    BincodeCodec.serialize(snapshot).expect("encode snapshot")
}

pub fn temp_store() -> (TempDir, PrepFsStore) {
    let dir = TempDir::new().expect("temp dir");
    let store = PrepFsStore::new(dir.path());
    (dir, store)
}

pub fn save(snapshot: &PrepSnapshot) -> (TempDir, PrepFsStore, PrepSnapshotId) {
    let (dir, mut store) = temp_store();
    let snap_id = store.save_snapshot(snapshot).expect("save snapshot");
    (dir, store, snap_id)
}

pub fn bin_path(dir: &TempDir, snap_id: &PrepSnapshotId) -> PathBuf {
    PrepFsStore::new(dir.path())
        .snapshot_dir()
        .join(format!("{}.bin", hex_id(snap_id)))
}

pub fn hex_id(snap_id: &PrepSnapshotId) -> String {
    let mut out = String::with_capacity(64);
    for byte in snap_id.as_bytes() {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_anchor() {
        let _ = test_path as fn(u8, u32, u8) -> SettlementPath;
        let _ = test_leaf as fn(SettlementPath, u8) -> TerminalLeaf;
        let _ = snap as fn(&[(u8, u32, u8)]) -> PrepSnapshot;
        let _ = dup_item as fn(SettlementStateRoot, u8, u32, u8) -> SnapItem;
        let _ = hash_mix as fn(&SnapItem) -> SnapItem;
        let _ = bytes as fn(&PrepSnapshot) -> Vec<u8>;
        let _ = temp_store as fn() -> (TempDir, PrepFsStore);
        let _ = save as fn(&PrepSnapshot) -> (TempDir, PrepFsStore, PrepSnapshotId);
        let _ = bin_path as fn(&TempDir, &PrepSnapshotId) -> PathBuf;
        let _ = hex_id as fn(&PrepSnapshotId) -> String;
    }
}
