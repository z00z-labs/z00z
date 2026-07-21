//! File I/O operations with codec support

use super::error::IoError;
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tempfile::NamedTempFile;

#[path = "atomic_write.rs"]
mod atomic_write;
#[path = "bincode_io.rs"]
mod bincode_io;
#[path = "file_read.rs"]
mod file_read;
#[path = "fs_codec.rs"]
mod fs_codec;
#[path = "json_io.rs"]
mod json_io;
#[path = "secure.rs"]
mod secure;
#[cfg(test)]
#[path = "test_fs_io_suite.rs"]
mod test_io;
#[cfg(test)]
#[path = "test_fs_suite.rs"]
mod tests;
#[path = "yaml_io.rs"]
mod yaml_io;

pub use self::{
    atomic_write::{
        atomic_write_file_private, atomic_write_file_streaming, write_file, write_file_private_new,
    },
    bincode_io::{load_bincode, load_bincode_bounded, save_bincode},
    file_read::{
        create_dir_all, file_len, open_lock_file, path_exists, path_exists_no_follow, read_dir,
        read_dir_bounded, read_file, read_link, read_to_string, remove_dir_all, remove_file,
        rename_file, set_file_mode, set_permissions_mode, symlink_metadata, sync_directory,
    },
    fs_codec::{load_with_codec, read_file_bounded, save_with_codec},
    json_io::{load_json, load_json_bounded, save_json},
    secure::SecureDir,
    yaml_io::{load_yaml, load_yaml_bounded, save_yaml},
};

const PERM_COPY_FAIL_MARKER: &str = ".perm-copy-fail";
const ROOT_MARK_FILE: &str = ".managed-root-fingerprint";

// This test-only seam is intentionally scoped to the test_io_integration
// binary plus a hidden sibling marker file so production callers cannot trip
// deterministic permission-copy failures through normal destination names.
fn is_test_io_process() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_owned)
        })
        .map(|stem| stem == "test_io_integration" || stem.starts_with("test_io_integration-"))
        .unwrap_or(false)
}

fn permission_copy_seam_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("permission-copy-target");
    path.with_file_name(format!(".{file_name}{PERM_COPY_FAIL_MARKER}"))
}

fn is_permission_copy_failure_forced(path: &Path) -> bool {
    is_test_io_process() && permission_copy_seam_path(path).exists()
}

fn copy_existing_permissions(temp_file: &std::fs::File, path: &Path) -> Result<(), IoError> {
    if let Ok(meta) = std::fs::metadata(path) {
        if is_permission_copy_failure_forced(path) {
            return Err(IoError::Io(std::io::Error::new(
                ErrorKind::PermissionDenied,
                format!(
                    "forced permission-copy failure for test seam via {}",
                    path.display()
                ),
            )));
        }

        temp_file.set_permissions(meta.permissions())?;
    }

    Ok(())
}

/// Helper function for atomic file writing with error context
fn atomic_write_with_context<T: Serialize>(
    path: &Path,
    value: &T,
    serialize_fn: impl FnOnce(&T) -> Result<Vec<u8>, String>,
    codec_name: &str,
) -> Result<(), IoError> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Serialize the value with context
    let bytes = serialize_fn(value).map_err(|e| {
        IoError::Serialization(format!(
            "Failed to serialize {} to {} using {}: {}",
            std::any::type_name::<T>(),
            path.display(),
            codec_name,
            e
        ))
    })?;

    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    // Create a temp file with a random name in the same directory.
    // This avoids predictable temp paths, but parent-directory trust still applies.
    let mut temp = NamedTempFile::new_in(parent)?;

    // Preserve existing destination permissions when overwriting.
    copy_existing_permissions(temp.as_file(), path)?;

    temp.write_all(&bytes)?;
    temp.flush()?;

    temp.persist(path)
        .map(|_| ())
        .map_err(|e| IoError::Io(e.error))?;

    Ok(())
}

/// Helper function for file loading with error context
fn load_with_context<T: DeserializeOwned>(
    path: &Path,
    deserialize_fn: impl FnOnce(&[u8]) -> Result<T, String>,
    codec_name: &str,
) -> Result<T, IoError> {
    load_with_context_bounded(
        path,
        fs_codec::default_max_file_size(),
        deserialize_fn,
        codec_name,
    )
}

fn load_with_context_bounded<T: DeserializeOwned>(
    path: &Path,
    max_file_size: u64,
    deserialize_fn: impl FnOnce(&[u8]) -> Result<T, String>,
    codec_name: &str,
) -> Result<T, IoError> {
    let bytes = read_file_bounded(path, max_file_size)?;
    deserialize_fn(&bytes).map_err(|e| {
        IoError::Deserialization(format!(
            "Failed to deserialize {} from {} using {}: {}",
            std::any::type_name::<T>(),
            path.display(),
            codec_name,
            e
        ))
    })
}

/// Recreate a managed artifact root when its stored fingerprint drifts.
///
/// Returns `true` when the root was cleared and rebuilt for the supplied
/// fingerprint, or `false` when the existing root already matched.
pub fn prepare_managed_root(path: impl AsRef<Path>, fingerprint: &str) -> Result<bool, IoError> {
    let path = path.as_ref();
    let mark = root_mark_path(path);
    let stale = match std::fs::read_to_string(&mark) {
        Ok(current) => current != fingerprint,
        Err(_) => true,
    };

    if stale && path_exists(path)? {
        remove_dir_all(path)?;
    }

    create_dir_all(path)?;
    if stale {
        write_file(&mark, fingerprint.as_bytes())?;
    }
    Ok(stale)
}

/// Recreate a managed artifact root on the first use in this process and
/// reuse it afterward unless the fingerprint drifts.
///
/// The reset removes all entries under `path` except the optional preserved
/// relative prefixes supplied directly or through `preserve_env`.
pub fn reset_managed_root_once(
    path: impl AsRef<Path>,
    fingerprint: &str,
    preserve_prefixes: &[&str],
    preserve_env: Option<&str>,
) -> Result<bool, IoError> {
    let path = path.as_ref();
    let key = path.to_path_buf();
    let first_use = {
        let mut prepared = managed_root_once_registry()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        prepared.insert(key)
    };

    if first_use {
        reset_managed_root(path, fingerprint, preserve_prefixes, preserve_env)?;
        Ok(true)
    } else {
        prepare_managed_root(path, fingerprint)
    }
}

/// Recreate a managed artifact root immediately while preserving the selected
/// relative child prefixes.
pub fn reset_managed_root(
    path: impl AsRef<Path>,
    fingerprint: &str,
    preserve_prefixes: &[&str],
    preserve_env: Option<&str>,
) -> Result<(), IoError> {
    let path = path.as_ref();
    let keep = managed_root_keep_prefixes(preserve_prefixes, preserve_env)?;

    if path_exists(path)? {
        clear_managed_root_contents(path, Path::new(""), &keep)?;
    }

    create_dir_all(path)?;
    write_file(root_mark_path(path), fingerprint.as_bytes())?;
    Ok(())
}

/// Remove sibling scope directories whose Cargo hash suffix normalizes to the
/// active scope name.
pub fn prune_scope_alias_dirs(
    parent: impl AsRef<Path>,
    scope_name: &str,
) -> Result<usize, IoError> {
    let parent = parent.as_ref();
    if !path_exists(parent)? {
        return Ok(0);
    }

    let mut removed = 0usize;
    for entry in read_dir(parent)? {
        let meta = match std::fs::metadata(&entry) {
            Ok(meta) => meta,
            Err(err) if err.kind() == ErrorKind::NotFound => continue,
            Err(err) => return Err(IoError::Io(err)),
        };
        if !meta.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name == scope_name {
            continue;
        }
        if normalize_exe_scope_name(name) == scope_name {
            match remove_dir_all(&entry) {
                Ok(()) => removed += 1,
                Err(IoError::Io(err)) if err.kind() == ErrorKind::NotFound => {}
                Err(err) => return Err(err),
            }
        }
    }

    Ok(removed)
}

/// Remove direct child directories whose names are fixed-width lowercase or
/// uppercase hex digests.
pub fn prune_hex_dirs(parent: impl AsRef<Path>, width: usize) -> Result<usize, IoError> {
    if width == 0 {
        return Err(IoError::Io(std::io::Error::new(
            ErrorKind::InvalidInput,
            "hex dir width must be > 0",
        )));
    }

    let parent = parent.as_ref();
    if !path_exists(parent)? {
        return Ok(0);
    }

    let mut removed = 0usize;
    for entry in read_dir(parent)? {
        let meta = match std::fs::metadata(&entry) {
            Ok(meta) => meta,
            Err(err) if err.kind() == ErrorKind::NotFound => continue,
            Err(err) => return Err(IoError::Io(err)),
        };
        if !meta.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.len() == width && name.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            match remove_dir_all(&entry) {
                Ok(()) => removed += 1,
                Err(IoError::Io(err)) if err.kind() == ErrorKind::NotFound => {}
                Err(err) => return Err(err),
            }
        }
    }

    Ok(removed)
}

/// Return the current executable scope name with Cargo hash suffixes removed.
pub fn stable_current_exe_scope(fallback: &str) -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(normalize_exe_scope_name)
        })
        .filter(|scope| !scope.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

/// Infer the active Z00Z verification run root from the current executable path.
///
/// This is needed because Cargo does not reliably pass `CARGO_TARGET_DIR` or
/// caller-specific runtime env vars into spawned test binaries. When a test or
/// helper binary itself lives under
/// `reports/z00z-verification-orchestrator-<timestamp>/target/...`, this helper
/// recovers the enclosing run root so runtime caches can stay inside that
/// report tree instead of falling back to the repository root.
pub fn current_exe_run_root() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|path| {
        path.ancestors().find_map(|ancestor| {
            let name = ancestor.file_name()?.to_str()?;
            if !name.starts_with("z00z-verification-orchestrator-") {
                return None;
            }
            let parent = ancestor.parent()?;
            let parent_name = parent.file_name()?.to_str()?;
            if parent_name != "reports" {
                return None;
            }
            Some(ancestor.to_path_buf())
        })
    })
}

/// Build a deterministic digest for a managed-artifact input set.
///
/// The caller supplies a schema tag, an ordered file list, and an ordered
/// directory list. Directory trees are traversed recursively with stable
/// sorting so any input drift changes the returned hex digest.
pub fn hash_root_inputs(
    schema: &str,
    files: &[PathBuf],
    dirs: &[PathBuf],
) -> Result<String, IoError> {
    let mut hasher = Sha256::new();
    hasher.update(schema.as_bytes());
    hasher.update(b"\n");

    for file in files {
        hash_file(&mut hasher, file)?;
    }
    for dir in dirs {
        hash_tree(&mut hasher, dir)?;
    }

    Ok(hex_digest(&hasher.finalize()))
}

fn managed_root_once_registry() -> &'static Mutex<std::collections::BTreeSet<PathBuf>> {
    static PREPARED: OnceLock<Mutex<std::collections::BTreeSet<PathBuf>>> = OnceLock::new();
    PREPARED.get_or_init(|| Mutex::new(std::collections::BTreeSet::new()))
}

fn root_mark_path(path: &Path) -> PathBuf {
    path.join(ROOT_MARK_FILE)
}

fn managed_root_keep_prefixes(
    preserve_prefixes: &[&str],
    preserve_env: Option<&str>,
) -> Result<Vec<PathBuf>, IoError> {
    let mut keep = Vec::new();

    for prefix in preserve_prefixes {
        keep.push(parse_preserve_prefix(prefix)?);
    }

    if let Some(env_key) = preserve_env {
        if let Some(raw) = std::env::var_os(env_key) {
            let raw = raw.to_string_lossy();
            for prefix in raw.split([';', '\n']) {
                let trimmed = prefix.trim();
                if trimmed.is_empty() {
                    continue;
                }
                keep.push(parse_preserve_prefix(trimmed)?);
            }
        }
    }

    keep.sort();
    keep.dedup();
    Ok(keep)
}

fn parse_preserve_prefix(raw: &str) -> Result<PathBuf, IoError> {
    let mut out = PathBuf::new();
    for component in Path::new(raw).components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(part) => out.push(part),
            _ => {
                return Err(IoError::Io(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("managed-root preserve prefix must stay relative: {raw}"),
                )));
            }
        }
    }

    if out.as_os_str().is_empty() {
        return Err(IoError::Io(std::io::Error::new(
            ErrorKind::InvalidInput,
            "managed-root preserve prefix must not be empty",
        )));
    }

    Ok(out)
}

fn clear_managed_root_contents(
    dir: &Path,
    rel_dir: &Path,
    keep: &[PathBuf],
) -> Result<(), IoError> {
    if !path_exists(dir)? {
        return Ok(());
    }

    for entry in read_dir(dir)? {
        let name = entry.file_name().map(PathBuf::from).ok_or_else(|| {
            IoError::Io(std::io::Error::other(format!(
                "managed-root entry missing file name: {}",
                entry.display()
            )))
        })?;
        let rel_path = if rel_dir.as_os_str().is_empty() {
            name
        } else {
            rel_dir.join(name)
        };
        let meta = std::fs::metadata(&entry)?;

        if keep.iter().any(|prefix| rel_path.starts_with(prefix)) {
            continue;
        }

        if meta.is_dir() && keep.iter().any(|prefix| prefix.starts_with(&rel_path)) {
            clear_managed_root_contents(&entry, &rel_path, keep)?;
            if read_dir(&entry)?.is_empty() {
                remove_dir_all(&entry)?;
            }
            continue;
        }

        if meta.is_dir() {
            remove_dir_all(&entry)?;
        } else {
            remove_file(&entry)?;
        }
    }

    Ok(())
}

fn normalize_exe_scope_name(raw: &str) -> String {
    let Some((prefix, suffix)) = raw.rsplit_once('-') else {
        return raw.to_string();
    };

    if suffix.len() >= 8 && suffix.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        prefix.to_string()
    } else {
        raw.to_string()
    }
}

fn hash_tree(hasher: &mut Sha256, root: &Path) -> Result<(), IoError> {
    hasher.update(b"tree:\n");
    hasher.update(root.as_os_str().as_encoded_bytes());
    hasher.update(b"\n");

    if !path_exists(root)? {
        hasher.update(b"missing-tree\n");
        return Ok(());
    }

    let mut files = Vec::new();
    collect_tree_files(root, &mut files)?;
    for file in files {
        hash_file(hasher, &file)?;
    }
    Ok(())
}

fn collect_tree_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), IoError> {
    let mut entries = read_dir(root)?;
    entries.sort();

    for entry in entries {
        if entry
            .file_name()
            .is_some_and(|name| name == ".git" || name == "target")
        {
            continue;
        }
        if std::fs::metadata(&entry)?.is_dir() {
            collect_tree_files(&entry, files)?;
        } else {
            files.push(entry);
        }
    }

    Ok(())
}

fn hash_file(hasher: &mut Sha256, path: &Path) -> Result<(), IoError> {
    hasher.update(b"file:\n");
    hasher.update(path.as_os_str().as_encoded_bytes());
    hasher.update(b"\n");

    match read_file(path) {
        Ok(bytes) => {
            hasher.update(bytes.len().to_string().as_bytes());
            hasher.update(b"\n");
            hasher.update(&bytes);
            hasher.update(b"\n");
            Ok(())
        }
        Err(IoError::Io(err)) if err.kind() == ErrorKind::NotFound => {
            hasher.update(b"missing-file\n");
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(hex_char(byte >> 4));
        out.push(hex_char(byte & 0x0f));
    }
    out
}

fn hex_char(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        10..=15 => char::from(b'a' + (nibble - 10)),
        _ => unreachable!("hex nibble out of range"),
    }
}
