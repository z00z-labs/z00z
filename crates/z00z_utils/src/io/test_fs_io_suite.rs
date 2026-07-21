use super::*;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestData {
    id: u32,
    name: String,
    values: Vec<i32>,
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
