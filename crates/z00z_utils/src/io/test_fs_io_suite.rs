use super::*;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestData {
    id: u32,
    name: String,
    values: Vec<i32>,
}

#[cfg(unix)]
fn secure_root(dir: &TempDir) -> std::path::PathBuf {
    let root = dir.path().join("secure");
    std::fs::create_dir(&root).expect("create secure root");
    set_permissions_mode(&root, 0o700).expect("set secure root mode");
    root
}

#[cfg(unix)]
fn write_secure(dir: &SecureDir, name: &str, bytes: &[u8]) {
    use std::io::Write as _;

    let mut file = dir.create_file(name).expect("create secure file");
    file.write_all(bytes).expect("write secure file");
    file.sync_all().expect("sync secure file");
}

#[test]
#[cfg(unix)]
fn test_secure_lock_rejects_symlink() {
    let temp = TempDir::new().expect("create temp dir");
    let root = secure_root(&temp);
    let victim = temp.path().join("victim");
    std::fs::write(&victim, b"keep").expect("write victim");
    std::os::unix::fs::symlink(&victim, root.join("lock")).expect("create symlink");
    let dir = SecureDir::open(&root).expect("open secure root");

    assert!(dir.open_lock("lock").is_err());
    assert_eq!(std::fs::read(victim).expect("read victim"), b"keep");
}

#[test]
#[cfg(unix)]
fn test_secure_lock_mode() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().expect("create temp dir");
    let root = secure_root(&temp);
    std::fs::write(root.join("lock"), b"state").expect("seed lock");
    set_permissions_mode(root.join("lock"), 0o644).expect("set broad mode");
    let dir = SecureDir::open(root).expect("open secure root");

    let file = dir.open_lock("lock").expect("open lock");
    let mode = file.metadata().expect("lock fstat").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
#[cfg(unix)]
fn test_secure_child_rejects_symlink() {
    let temp = TempDir::new().expect("create temp dir");
    let root = secure_root(&temp);
    let target = temp.path().join("target");
    std::fs::create_dir(&target).expect("create child target");
    set_permissions_mode(&target, 0o700).expect("set target mode");
    std::os::unix::fs::symlink(&target, root.join("child")).expect("create child symlink");
    let dir = SecureDir::open(root).expect("open secure root");

    assert!(dir.open_child("child").is_err());
    assert!(dir.ensure_dir("child").is_err());
}

#[test]
#[cfg(unix)]
fn test_secure_child_mode() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().expect("create temp dir");
    let root = secure_root(&temp);
    std::fs::create_dir(root.join("child")).expect("create child");
    set_permissions_mode(root.join("child"), 0o755).expect("set broad child mode");
    let dir = SecureDir::open(root).expect("open secure root");

    assert!(dir.open_child("child").is_err());
    let child = dir.ensure_dir("child").expect("ensure private child");
    write_secure(&child, "entry", b"anchored");
    let mode = std::fs::metadata(temp.path().join("secure/child"))
        .expect("child metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o700);
    assert_eq!(
        child.read_file_bounded("entry", 64).expect("read child"),
        b"anchored"
    );
}

#[test]
#[cfg(unix)]
fn test_secure_read_rejects_symlink() {
    let temp = TempDir::new().expect("create temp dir");
    let root = secure_root(&temp);
    let victim = temp.path().join("victim");
    std::fs::write(&victim, b"secret").expect("write victim");
    std::os::unix::fs::symlink(&victim, root.join("entry")).expect("create symlink");
    let dir = SecureDir::open(root).expect("open secure root");

    assert!(dir.read_file_bounded("entry", 64).is_err());
}

#[test]
#[cfg(unix)]
fn test_secure_dir_stays_anchored() {
    let temp = TempDir::new().expect("create temp dir");
    let root = secure_root(&temp);
    let dir = SecureDir::open(&root).expect("open secure root");
    write_secure(&dir, "entry", b"original");

    let moved = temp.path().join("moved");
    std::fs::rename(&root, &moved).expect("move opened root");
    std::fs::create_dir(&root).expect("replace path root");
    set_permissions_mode(&root, 0o700).expect("set replacement root mode");
    std::fs::write(root.join("entry"), b"replacement").expect("write replacement");
    set_permissions_mode(root.join("entry"), 0o600).expect("set replacement mode");

    assert_eq!(
        dir.read_file_bounded("entry", 64).expect("read anchored"),
        b"original"
    );
}

#[test]
#[cfg(unix)]
fn test_secure_rename_no_clobber() {
    let temp = TempDir::new().expect("create temp dir");
    let root = secure_root(&temp);
    let dir = SecureDir::open(root).expect("open secure root");
    write_secure(&dir, "source", b"source");
    write_secure(&dir, "target", b"target");

    assert!(dir.rename_no_clobber("source", "target").is_err());
    assert_eq!(
        dir.read_file_bounded("source", 64).expect("read source"),
        b"source"
    );
    assert_eq!(
        dir.read_file_bounded("target", 64).expect("read target"),
        b"target"
    );
}

#[test]
#[cfg(unix)]
fn test_secure_remove_unlinks_symlink() {
    let temp = TempDir::new().expect("create temp dir");
    let root = secure_root(&temp);
    let victim = temp.path().join("victim");
    std::fs::write(&victim, b"keep").expect("write victim");
    std::os::unix::fs::symlink(&victim, root.join("entry")).expect("create symlink");
    let dir = SecureDir::open(root).expect("open secure root");

    dir.remove_file("entry").expect("unlink symlink entry");
    assert_eq!(std::fs::read(victim).expect("read victim"), b"keep");
}

#[test]
#[cfg(unix)]
fn test_dir_limit_one_overflow() {
    let temp = TempDir::new().expect("create temp dir");
    let root = secure_root(&temp);
    let dir = SecureDir::open(&root).expect("open secure root");
    write_secure(&dir, "a", b"a");
    write_secure(&dir, "b", b"b");
    write_secure(&dir, "c", b"c");

    assert_eq!(dir.read_dir_bounded(3).expect("read exact limit").len(), 3);
    assert!(matches!(
        dir.read_dir_bounded(2),
        Err(IoError::DirectoryTooLarge { max: 2 })
    ));
    assert!(matches!(
        read_dir_bounded(&root, 2),
        Err(IoError::DirectoryTooLarge { max: 2 })
    ));
}

#[test]
#[cfg(unix)]
fn test_secure_names_reject_paths() {
    let temp = TempDir::new().expect("create temp dir");
    let root = secure_root(&temp);
    let dir = SecureDir::open(root).expect("open secure root");

    for name in ["", ".", "..", "nested/name", "name/"] {
        assert!(dir.create_file(name).is_err(), "accepted {name:?}");
    }
}

#[test]
#[cfg(unix)]
fn test_atomic_write_symlink() {
    let dir = TempDir::new().unwrap();
    let victim = dir.path().join("victim.txt");
    let link = dir.path().join("link.txt");

    write_file(&victim, b"keep").expect("setup victim");
    std::os::unix::fs::symlink(&victim, &link).expect("setup symlink");

    atomic_write_file_private(&link, b"new").expect("write via atomic private API");

    let victim_bytes = read_file(&victim).expect("read victim");
    assert_eq!(victim_bytes, b"keep");

    let link_meta = std::fs::symlink_metadata(&link).expect("metadata");
    assert!(link_meta.file_type().is_file());

    let link_bytes = read_file(&link).expect("read link");
    assert_eq!(link_bytes, b"new");
}

#[test]
#[cfg(unix)]
fn test_path_exists_no_follow() {
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join("missing");
    let link = dir.path().join("dangling");
    std::os::unix::fs::symlink(&missing, &link).expect("create dangling symlink");

    assert!(!path_exists(&link).expect("followed existence"));
    assert!(path_exists_no_follow(&link).expect("entry existence"));
    assert!(symlink_metadata(&link)
        .expect("entry metadata")
        .file_type()
        .is_symlink());
    sync_directory(dir.path()).expect("directory sync");
}

#[test]
fn test_json_file_format() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("format.json");

    let data = TestData {
        id: 42,
        name: "format-test".to_string(),
        values: vec![1, 2, 3],
    };

    save_json(&path, &data).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content.contains('\n'));
    assert!(content.contains("\"id\""));
    assert!(content.contains("\"name\""));
}

#[test]
fn test_load_error_not_found() {
    let result: Result<TestData, _> = load_json("/nonexistent/path/file.json");
    assert!(result.is_err());
}

#[test]
fn test_load_error_invalid_json() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("invalid.json");

    std::fs::write(&path, b"not valid json {").unwrap();

    let result: Result<TestData, _> = load_json(&path);
    assert!(result.is_err());
}

#[test]
fn test_write_read_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test_bytes.bin");

    let data = b"Hello, world!";
    write_file(&path, data).unwrap();

    let loaded = read_file(&path).unwrap();
    assert_eq!(data, loaded.as_slice());
}

#[test]
fn test_read_to_string_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test_string.txt");

    let text = "Hello, UTF-8 мир!";
    write_file(&path, text.as_bytes()).unwrap();

    let loaded = read_to_string(&path).unwrap();
    assert_eq!(text, loaded);
}

#[test]
fn test_file_len_operation() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("length.bin");

    let data = b"length-check";
    write_file(&path, data).unwrap();

    let len = file_len(&path).unwrap();
    assert_eq!(len, data.len() as u64);
}

#[test]
fn test_file_len_missing() {
    let result = file_len("/nonexistent/path/file.bin");
    assert!(result.is_err());
}

#[test]
#[cfg(unix)]
fn test_read_rejects_large_device() {
    let result = read_file_bounded("/dev/zero", 1024);
    assert!(matches!(result, Err(IoError::FileTooLarge { .. })));
}

#[test]
fn test_read_rejects_large_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("big.bin");

    let limit = 10 * 1024 * 1024;
    let data = vec![0u8; (limit + 1024) as usize];
    write_file(&path, &data).unwrap();

    let result = read_file_bounded(&path, limit);
    assert!(matches!(result, Err(IoError::FileTooLarge { .. })));
}

#[test]
fn test_remove_file_operation() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("to_delete.txt");

    write_file(&path, b"delete me").unwrap();
    assert!(path.exists());

    remove_file(&path).unwrap();
    assert!(!path.exists());
}

#[test]
fn test_rename_file_operation() {
    let dir = TempDir::new().unwrap();
    let old_path = dir.path().join("old_name.txt");
    let new_path = dir.path().join("new_name.txt");

    write_file(&old_path, b"content").unwrap();
    assert!(old_path.exists());

    rename_file(&old_path, &new_path).unwrap();
    assert!(!old_path.exists());
    assert!(new_path.exists());

    let content = read_file(&new_path).unwrap();
    assert_eq!(b"content", content.as_slice());
}

#[test]
fn test_create_dir_all_operation() {
    let dir = TempDir::new().unwrap();
    let nested_path = dir.path().join("a/b/c/d");

    assert!(!nested_path.exists());
    create_dir_all(&nested_path).unwrap();
    assert!(nested_path.exists());
    assert!(nested_path.is_dir());
}

#[test]
fn test_write_file_creates_parent() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nested/path/file.txt");

    write_file(&path, b"content").unwrap();
    assert!(path.exists());

    let content = read_file(&path).unwrap();
    assert_eq!(b"content", content.as_slice());
}

#[test]
fn test_streaming_write_works() {
    use std::io::Write;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("stream.txt");

    atomic_write_file_streaming(&path, |file| {
        file.write_all(b"streamed content")?;
        Ok(())
    })
    .unwrap();

    let content = read_file(&path).unwrap();
    assert_eq!(content, b"streamed content");
}

#[test]
#[cfg(unix)]
fn test_temp_name_random() {
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    let dir = TempDir::new().unwrap();
    let dir_path = dir.path().to_path_buf();
    let path = dir.path().join("out.txt");

    let mut names = Vec::new();

    for _ in 0..2 {
        let seen = Arc::new(Mutex::new(None::<String>));
        let seen2 = Arc::clone(&seen);
        let dir_path = dir_path.clone();
        let dst = path.clone();
        let dst2 = dst.clone();

        atomic_write_file_streaming(&dst, move |file| {
            file.write_all(b"data")?;

            let mut tmp_files = Vec::new();
            for entry in std::fs::read_dir(&dir_path)? {
                let path = entry?.path();
                if path != dst2 {
                    tmp_files.push(path);
                }
            }

            assert_eq!(tmp_files.len(), 1, "expected exactly one temp file");
            let name = tmp_files[0]
                .file_name()
                .and_then(|item| item.to_str())
                .unwrap_or("")
                .to_string();
            *seen2.lock().unwrap() = Some(name);

            Ok(())
        })
        .unwrap();

        names.push(seen.lock().unwrap().clone().unwrap());
    }

    assert_ne!(names[0], names[1], "temp file names must differ");
}

#[test]
#[cfg(not(unix))]
fn test_streaming_fallback_works() {
    use std::io::Write;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("fallback.txt");

    atomic_write_file_streaming(&path, |file| {
        file.write_all(b"fallback works")?;
        Ok(())
    })
    .unwrap();

    let content = read_file(&path).unwrap();
    assert_eq!(content, b"fallback works");
}

#[test]
#[cfg(not(unix))]
fn test_perms_mode_noop() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("noop.txt");
    write_file(&path, b"test").unwrap();

    set_permissions_mode(&path, 0o600).unwrap();
}

#[test]
#[cfg(unix)]
fn test_perms_mode_enforced() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("enforced.txt");
    write_file(&path, b"test").unwrap();

    set_permissions_mode(&path, 0o600).unwrap();
    let perms = std::fs::metadata(&path).unwrap().permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
}

#[test]
#[cfg(unix)]
fn test_private_file_unix_mode() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("private.txt");

    atomic_write_file_private(&path, b"secret").unwrap();
    let perms = std::fs::metadata(&path).unwrap().permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
}

#[test]
#[cfg(not(unix))]
fn test_private_file_fallback() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("private.txt");

    atomic_write_file_private(&path, b"secret").unwrap();
    let content = read_file(&path).unwrap();
    assert_eq!(content, b"secret");
}
