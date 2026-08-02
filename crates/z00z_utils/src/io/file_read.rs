use super::{read_file_bounded, ErrorKind, IoError, Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

const DEFAULT_MAX_DIR_ENTRIES: usize = 100_000;

/// Read raw bytes from a file using the default bounded size limit.
pub fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>, IoError> {
    let path = path.as_ref();
    read_file_bounded(path, super::fs_codec::default_max_file_size())
}

/// Read a UTF-8 string from a file using the default bounded size limit.
pub fn read_to_string(path: impl AsRef<Path>) -> Result<String, IoError> {
    let path = path.as_ref();
    let bytes = read_file_bounded(path, super::fs_codec::default_max_file_size())?;
    String::from_utf8(bytes)
        .map_err(|e| IoError::Io(std::io::Error::new(ErrorKind::InvalidData, e)))
}

/// Read a symbolic-link target path.
pub fn read_link(path: impl AsRef<Path>) -> Result<PathBuf, IoError> {
    let path = path.as_ref();
    Ok(std::fs::read_link(path)?)
}

/// Return the current file length in bytes.
pub fn file_len(path: impl AsRef<Path>) -> Result<u64, IoError> {
    let path = path.as_ref();
    Ok(std::fs::metadata(path)?.len())
}

/// Remove a single file from the filesystem.
pub fn remove_file(path: impl AsRef<Path>) -> Result<(), IoError> {
    let path = path.as_ref();
    Ok(std::fs::remove_file(path)?)
}

/// Remove a directory tree and treat missing paths as success.
pub fn remove_dir_all(path: impl AsRef<Path>) -> Result<(), IoError> {
    let path = path.as_ref();
    match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Rename a file or directory.
pub fn rename_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<(), IoError> {
    let from = from.as_ref();
    let to = to.as_ref();
    Ok(std::fs::rename(from, to)?)
}

/// Check whether a filesystem path currently exists.
pub fn path_exists(path: impl AsRef<Path>) -> Result<bool, IoError> {
    let path = path.as_ref();
    match std::fs::metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

/// Check whether a path entry exists without following symbolic links.
pub fn path_exists_no_follow(path: impl AsRef<Path>) -> Result<bool, IoError> {
    let path = path.as_ref();
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

/// Read metadata for a path entry without following symbolic links.
pub fn symlink_metadata(path: impl AsRef<Path>) -> Result<std::fs::Metadata, IoError> {
    Ok(std::fs::symlink_metadata(path)?)
}

/// Return whether two file destinations may address the same entry.
///
/// Existing parent directories are compared by filesystem identity after
/// resolving parent aliases. The final component is inspected without
/// following symbolic links, and existing hard links are compared by file
/// identity. A final symbolic link is rejected conservatively.
pub fn destinations_alias_no_follow(
    left: impl AsRef<Path>,
    right: impl AsRef<Path>,
) -> Result<bool, IoError> {
    let left = left.as_ref();
    let right = right.as_ref();
    let left_name = left.file_name().ok_or_else(invalid_destination)?;
    let right_name = right.file_name().ok_or_else(invalid_destination)?;
    let left_parent = left.parent().unwrap_or_else(|| Path::new("."));
    let right_parent = right.parent().unwrap_or_else(|| Path::new("."));
    let left_parent_metadata = metadata_if_present(left_parent)?;
    let right_parent_metadata = metadata_if_present(right_parent)?;
    let same_parent = match (&left_parent_metadata, &right_parent_metadata) {
        (Some(left_metadata), Some(right_metadata)) => {
            if !left_metadata.is_dir() || !right_metadata.is_dir() {
                return Err(invalid_destination().into());
            }
            metadata_identity_equal(left_metadata, right_metadata)
                || resolved_parent_path(left_parent)? == resolved_parent_path(right_parent)?
        }
        _ => resolved_parent_path(left_parent)? == resolved_parent_path(right_parent)?,
    };
    if same_parent && left_name == right_name {
        return Ok(true);
    }

    let left_metadata = symlink_metadata_if_present(left)?;
    let right_metadata = symlink_metadata_if_present(right)?;
    if left_metadata
        .as_ref()
        .is_some_and(|metadata| metadata.file_type().is_symlink())
        || right_metadata
            .as_ref()
            .is_some_and(|metadata| metadata.file_type().is_symlink())
    {
        return Ok(true);
    }
    Ok(match (left_metadata, right_metadata) {
        (Some(left_metadata), Some(right_metadata)) => {
            metadata_identity_equal(&left_metadata, &right_metadata)
                || std::fs::canonicalize(left)? == std::fs::canonicalize(right)?
        }
        _ => false,
    })
}

fn invalid_destination() -> std::io::Error {
    std::io::Error::new(ErrorKind::InvalidInput, "destination must name a file")
}

fn metadata_if_present(path: &Path) -> Result<Option<std::fs::Metadata>, IoError> {
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn symlink_metadata_if_present(path: &Path) -> Result<Option<std::fs::Metadata>, IoError> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn resolved_parent_path(path: &Path) -> Result<PathBuf, IoError> {
    let mut cursor = path.to_path_buf();
    let mut missing = Vec::new();
    loop {
        match std::fs::canonicalize(&cursor) {
            Ok(mut resolved) => {
                for component in missing.iter().rev() {
                    resolved.push(component);
                }
                return Ok(resolved);
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let component = cursor.file_name().ok_or(error)?;
                missing.push(component.to_os_string());
                if !cursor.pop() {
                    return Err(invalid_destination().into());
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(unix)]
fn metadata_identity_equal(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(not(unix))]
fn metadata_identity_equal(_left: &std::fs::Metadata, _right: &std::fs::Metadata) -> bool {
    false
}

/// Flush a directory entry set to stable storage.
pub fn sync_directory(path: impl AsRef<Path>) -> Result<(), IoError> {
    std::fs::File::open(path)?.sync_all()?;
    Ok(())
}

/// Open a filesystem lock file without truncating existing bytes.
pub fn open_lock_file(path: impl AsRef<Path>) -> Result<std::fs::File, IoError> {
    Ok(std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)?)
}

/// Create a directory path recursively.
pub fn create_dir_all(path: impl AsRef<Path>) -> Result<(), IoError> {
    let path = path.as_ref();
    Ok(std::fs::create_dir_all(path)?)
}

/// Apply a numeric Unix mode, or no-op outside Unix.
pub fn set_permissions_mode(path: impl AsRef<Path>, mode: u32) -> Result<(), IoError> {
    #[cfg(unix)]
    {
        let path = path.as_ref();
        let perms = std::fs::Permissions::from_mode(mode);
        std::fs::set_permissions(path, perms)?;
        Ok(())
    }

    #[cfg(not(unix))]
    {
        let _ = path;
        let _ = mode;
        Ok(())
    }
}

/// Apply a numeric Unix mode to an open file, or no-op outside Unix.
pub fn set_file_mode(file: &std::fs::File, mode: u32) -> Result<(), IoError> {
    #[cfg(unix)]
    {
        file.set_permissions(std::fs::Permissions::from_mode(mode))?;
    }

    #[cfg(not(unix))]
    {
        let _ = file;
        let _ = mode;
    }
    Ok(())
}

/// Mark a file read-only using the platform-native permission representation.
pub fn set_file_readonly(path: impl AsRef<Path>) -> Result<(), IoError> {
    let path = path.as_ref();
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions)?;
    std::fs::File::open(path)?.sync_all()?;
    Ok(())
}

/// Read directory entries into a bounded, deterministically sorted path list.
pub fn read_dir(path: impl AsRef<Path>) -> Result<Vec<PathBuf>, IoError> {
    read_dir_bounded(path, DEFAULT_MAX_DIR_ENTRIES)
}

/// Read sorted directory paths while enforcing an entry-count limit.
///
/// At most `max_entries + 1` paths are collected so oversized directories fail
/// without first allocating or traversing the full directory.
pub fn read_dir_bounded(
    path: impl AsRef<Path>,
    max_entries: usize,
) -> Result<Vec<PathBuf>, IoError> {
    let path = path.as_ref();
    let probe_limit = max_entries.checked_add(1).ok_or_else(|| {
        IoError::Io(std::io::Error::new(
            ErrorKind::InvalidInput,
            "directory entry limit must allow a one-entry overflow probe",
        ))
    })?;
    let mut entries = Vec::with_capacity(probe_limit.min(256));

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        entries.push(entry.path());
        if entries.len() == probe_limit {
            return Err(IoError::DirectoryTooLarge { max: max_entries });
        }
    }

    entries.sort();
    Ok(entries)
}
