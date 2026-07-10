use z00z_storage::fixture_support::snapshot_fix::{bin_path, bytes, save, snap, temp_store};
use z00z_storage::snapshot::{PrepSnapshotError, PrepSnapshotId, PrepSnapshotStore};
use z00z_utils::io::{create_dir_all, write_file};

#[test]
fn test_persist_ok() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_dir, store, snap_id) = save(&snapshot);

    let loaded = store.load_snapshot(&snap_id).expect("load snapshot");

    assert_eq!(loaded, snapshot);
}

#[test]
fn test_persist_key_mix() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (dir, store) = temp_store();
    let wrong_id = PrepSnapshotId::new([2u8; 32]);

    create_dir_all(store.snapshot_dir()).expect("create snapshot dir");
    write_file(bin_path(&dir, &wrong_id), &bytes(&snapshot)).expect("write wrong-key snapshot");

    let err = store.load_snapshot(&wrong_id).expect_err("key mix");

    assert!(matches!(err, PrepSnapshotError::IdMix));
}

#[test]
fn test_persist_corrupt() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (dir, _, snap_id) = save(&snapshot);
    let store = z00z_storage::snapshot::PrepFsStore::new(dir.path());

    write_file(bin_path(&dir, &snap_id), &[1u8, 2, 3]).expect("write corrupt snapshot");

    let err = store.load_snapshot(&snap_id).expect_err("corrupt load");

    assert!(matches!(err, PrepSnapshotError::Codec(_)));
}

#[test]
fn test_reject_json_wrap() {
    let (dir, store) = temp_store();
    let snap_id = PrepSnapshotId::new([7u8; 32]);

    create_dir_all(store.snapshot_dir()).expect("create snapshot dir");
    write_file(
        bin_path(&dir, &snap_id),
        br#"{"prev_root_hex":"00","rows":[]}"# as &[u8],
    )
    .expect("write json wrapper");

    let err = store
        .load_snapshot(&snap_id)
        .expect_err("json wrapper reject");

    assert!(matches!(err, PrepSnapshotError::Codec(_)));
}
