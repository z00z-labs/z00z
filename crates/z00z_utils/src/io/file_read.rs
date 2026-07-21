use super::{read_file_bounded, ErrorKind, IoError, Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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

/// Read directory entries into a deterministically sorted path list.
pub fn read_dir(path: impl AsRef<Path>) -> Result<Vec<PathBuf>, IoError> {
    let path = path.as_ref();
    let mut entries = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        entries.push(entry.path());
    }

    entries.sort();
    Ok(entries)
}
