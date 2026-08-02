use super::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;

fn assert_split_file(name: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/io")
        .join(name);
    assert!(path.is_file(), "missing split io seam: {}", path.display());
}

fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().expect("io env lock")
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestData {
    id: u32,
    name: String,
    values: Vec<i32>,
}

#[test]
fn test_io_split_files_exist() {
    assert_split_file("atomic_write.rs");
    assert_split_file("file_read.rs");
    assert_split_file("json_io.rs");
    assert_split_file("yaml_io.rs");
    assert_split_file("bincode_io.rs");
    assert_split_file("fs_codec.rs");
}

#[test]
fn test_save_load_json() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.json");

    let original = TestData {
        id: 42,
        name: "test".to_string(),
        values: vec![1, 2, 3],
    };

    save_json(&path, &original).unwrap();
    let loaded: TestData = load_json(&path).unwrap();
    assert_eq!(original, loaded);
}

#[test]
fn test_save_load_yaml() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.yaml");

    let original = TestData {
        id: 99,
        name: "yaml-test".to_string(),
        values: vec![4, 5, 6],
    };

    save_yaml(&path, &original).unwrap();
    let loaded: TestData = load_yaml(&path).unwrap();
    assert_eq!(original, loaded);
}

#[test]
fn test_save_load_bincode() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.bin");

    let original = TestData {
        id: 7,
        name: "bincode-test".to_string(),
        values: vec![10, 20, 30],
    };

    save_bincode(&path, &original).unwrap();
    let loaded: TestData = load_bincode(&path).unwrap();
    assert_eq!(original, loaded);
}

#[test]
fn test_create_parent_directories() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nested/deep/dir/file.json");

    let data = TestData {
        id: 1,
        name: "nested".to_string(),
        values: vec![],
    };

    save_json(&path, &data).unwrap();
    assert!(path.exists());

    let loaded: TestData = load_json(&path).unwrap();
    assert_eq!(data, loaded);
}

#[test]
fn test_atomic_write() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("atomic.json");

    let data1 = TestData {
        id: 1,
        name: "first".to_string(),
        values: vec![1],
    };

    let data2 = TestData {
        id: 2,
        name: "second".to_string(),
        values: vec![2],
    };

    save_json(&path, &data1).unwrap();
    save_json(&path, &data2).unwrap();

    let loaded: TestData = load_json(&path).unwrap();
    assert_eq!(data2, loaded);
}

#[test]
fn test_atomic_write_parallel() {
    use std::thread;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("parallel.json");

    let handles: Vec<_> = (0..10)
        .map(|index| {
            let path = path.clone();
            thread::spawn(move || save_json(&path, &index).unwrap())
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let loaded: i32 = load_json(&path).unwrap();
    assert!((0..10).contains(&loaded));
}

#[test]
fn test_prepare_root_stale() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("managed");

    create_dir_all(&root).unwrap();
    write_file(root.join(".managed-root-fingerprint"), b"stale").unwrap();
    write_file(root.join("old.txt"), b"old").unwrap();

    let cleared = prepare_managed_root(&root, "fresh").unwrap();

    assert!(cleared);
    assert!(!root.join("old.txt").exists());
    assert_eq!(
        read_to_string(root.join(".managed-root-fingerprint")).unwrap(),
        "fresh"
    );
}

#[test]
fn test_prepare_root_reuse() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("managed");

    let first = prepare_managed_root(&root, "same").unwrap();
    assert!(first);
    write_file(root.join("keep.txt"), b"ok").unwrap();

    let second = prepare_managed_root(&root, "same").unwrap();

    assert!(!second);
    assert_eq!(read_to_string(root.join("keep.txt")).unwrap(), "ok");
}

#[test]
fn test_root_reuse_stale_payload() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("managed");

    reset_managed_root(&root, "same", &[], None).unwrap();
    write_file(root.join("stale.txt"), b"stale").unwrap();

    reset_managed_root(&root, "same", &[], None).unwrap();

    assert!(!root.join("stale.txt").exists());
    assert_eq!(
        read_to_string(root.join(".managed-root-fingerprint")).unwrap(),
        "same"
    );
}

#[test]
fn test_reset_root_selected_prefixes() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("managed");

    create_dir_all(root.join("keep/nested")).unwrap();
    create_dir_all(root.join("drop")).unwrap();
    write_file(root.join("keep/nested/marker.txt"), b"keep").unwrap();
    write_file(root.join("drop/stale.txt"), b"drop").unwrap();

    reset_managed_root(&root, "fresh", &["keep"], None).unwrap();

    assert_eq!(
        read_to_string(root.join("keep/nested/marker.txt")).unwrap(),
        "keep"
    );
    assert!(!root.join("drop").exists());
}

#[cfg(unix)]
#[test]
fn test_reset_root_does_not_follow_preserved_symlink() {
    use std::os::unix::fs::symlink;

    let dir = TempDir::new().unwrap();
    let root = dir.path().join("managed");
    let outside = dir.path().join("outside");

    create_dir_all(&root).unwrap();
    create_dir_all(outside.join("keep")).unwrap();
    write_file(outside.join("keep/marker.txt"), b"keep").unwrap();
    write_file(outside.join("victim.txt"), b"outside").unwrap();
    symlink(&outside, root.join("linked")).unwrap();

    reset_managed_root(&root, "fresh", &["linked/keep"], None).unwrap();

    assert!(!path_exists_no_follow(root.join("linked")).unwrap());
    assert_eq!(
        read_to_string(outside.join("keep/marker.txt")).unwrap(),
        "keep"
    );
    assert_eq!(
        read_to_string(outside.join("victim.txt")).unwrap(),
        "outside"
    );
}

#[test]
fn test_keep_env_relative_prefix() {
    let _guard = env_lock();
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("managed");

    create_dir_all(root.join("env-keep")).unwrap();
    create_dir_all(root.join("drop")).unwrap();
    write_file(root.join("env-keep/marker.txt"), b"keep").unwrap();
    write_file(root.join("drop/stale.txt"), b"drop").unwrap();
    std::env::set_var("Z00Z_TEST_KEEP_PREFIXES", "env-keep");

    reset_managed_root(&root, "fresh", &[], Some("Z00Z_TEST_KEEP_PREFIXES")).unwrap();

    std::env::remove_var("Z00Z_TEST_KEEP_PREFIXES");
    assert_eq!(
        read_to_string(root.join("env-keep/marker.txt")).unwrap(),
        "keep"
    );
    assert!(!root.join("drop").exists());
}

#[test]
fn test_root_once_reuses_scope() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("managed-once");

    let first = reset_managed_root_once(&root, "same", &[], None).unwrap();
    assert!(first);
    write_file(root.join("keep.txt"), b"keep").unwrap();

    let second = reset_managed_root_once(&root, "same", &[], None).unwrap();

    assert!(!second);
    assert_eq!(read_to_string(root.join("keep.txt")).unwrap(), "keep");
}

#[test]
fn test_strips_cargo_hash_suffix() {
    assert_eq!(
        normalize_exe_scope_name("test_checkpoint_acceptance-8e65c4c0bf7f47fb"),
        "test_checkpoint_acceptance"
    );
    assert_eq!(
        normalize_exe_scope_name("scenario_1"),
        "scenario_1".to_string()
    );
    assert_eq!(
        normalize_exe_scope_name("assets-generation"),
        "assets-generation".to_string()
    );
}

#[test]
fn test_dirs_hashed_scope_siblings() {
    let dir = TempDir::new().unwrap();
    let parent = dir.path().join("scopes");

    create_dir_all(parent.join("test_stage4_card_gate")).unwrap();
    create_dir_all(parent.join("test_stage4_card_gate-9d4bd19d594c355f")).unwrap();
    create_dir_all(parent.join("test_stage4_card_gate-v1")).unwrap();
    write_file(
        parent.join("test_stage4_card_gate-9d4bd19d594c355f/marker.txt"),
        b"stale",
    )
    .unwrap();

    let removed = prune_scope_alias_dirs(&parent, "test_stage4_card_gate").unwrap();

    assert_eq!(removed, 1);
    assert!(parent.join("test_stage4_card_gate").exists());
    assert!(!parent
        .join("test_stage4_card_gate-9d4bd19d594c355f")
        .exists());
    assert!(parent.join("test_stage4_card_gate-v1").exists());
}

#[cfg(unix)]
#[test]
fn test_dirs_ignore_metadata() {
    use std::os::unix::fs::symlink;

    let dir = TempDir::new().unwrap();
    let parent = dir.path().join("scopes");

    create_dir_all(parent.join("test_stage4_card_gate")).unwrap();
    create_dir_all(parent.join("test_stage4_card_gate-9d4bd19d594c355f")).unwrap();
    symlink(parent.join("missing-target"), parent.join("broken-entry")).unwrap();

    let removed = prune_scope_alias_dirs(&parent, "test_stage4_card_gate").unwrap();

    assert_eq!(removed, 1);
    assert!(parent.join("test_stage4_card_gate").exists());
    assert!(!parent
        .join("test_stage4_card_gate-9d4bd19d594c355f")
        .exists());
}

#[test]
fn test_hex_dirs_hex_children() {
    let dir = TempDir::new().unwrap();
    let parent = dir.path().join("hex");

    create_dir_all(parent.join("0123456789abcdef")).unwrap();
    create_dir_all(parent.join("not-a-hash")).unwrap();
    create_dir_all(parent.join("deadbeef")).unwrap();

    let removed = prune_hex_dirs(&parent, 16).unwrap();

    assert_eq!(removed, 1);
    assert!(!parent.join("0123456789abcdef").exists());
    assert!(parent.join("not-a-hash").exists());
    assert!(parent.join("deadbeef").exists());
}

#[cfg(unix)]
#[test]
fn test_hex_dirs_ignore_metadata() {
    use std::os::unix::fs::symlink;

    let dir = TempDir::new().unwrap();
    let parent = dir.path().join("hex");

    create_dir_all(parent.join("0123456789abcdef")).unwrap();
    create_dir_all(parent.join("not-a-hash")).unwrap();
    symlink(parent.join("missing-target"), parent.join("broken-entry")).unwrap();

    let removed = prune_hex_dirs(&parent, 16).unwrap();

    assert_eq!(removed, 1);
    assert!(!parent.join("0123456789abcdef").exists());
    assert!(parent.join("not-a-hash").exists());
}

#[test]
fn test_root_hash_change() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("input.txt");
    let tree = dir.path().join("tree");

    write_file(&file, b"one").unwrap();
    create_dir_all(&tree).unwrap();
    write_file(tree.join("nested.txt"), b"alpha").unwrap();

    let before = hash_root_inputs("root-hash-v1", &[file.clone()], &[tree.clone()]).unwrap();
    write_file(tree.join("nested.txt"), b"beta").unwrap();
    let after = hash_root_inputs("root-hash-v1", &[file], &[tree]).unwrap();

    assert_ne!(before, after);
}
