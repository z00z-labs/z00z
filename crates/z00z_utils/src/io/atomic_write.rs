use super::{copy_existing_permissions, IoError, NamedTempFile, Path, Write};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Write raw bytes with the standard atomic temp-file replacement path.
pub fn write_file(path: impl AsRef<Path>, data: &[u8]) -> Result<(), IoError> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    let mut temp = NamedTempFile::new_in(parent)?;
    copy_existing_permissions(temp.as_file(), path)?;
    temp.write_all(data)?;
    temp.flush()?;

    temp.persist(path)
        .map(|_| ())
        .map_err(|e| IoError::Io(e.error))?;
    Ok(())
}

/// Write sensitive bytes atomically and enforce private Unix permissions.
pub fn atomic_write_file_private(path: impl AsRef<Path>, data: &[u8]) -> Result<(), IoError> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    #[cfg(unix)]
    {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut temp = NamedTempFile::new_in(parent)?;

        {
            let permissions = std::fs::Permissions::from_mode(0o600);
            temp.as_file().set_permissions(permissions)?;
        }

        let file = temp.as_file_mut();
        file.write_all(data)?;
        file.sync_all()?;

        temp.persist(path)
            .map(|_| ())
            .map_err(|e| IoError::Io(e.error))?;

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                let dir = std::fs::File::open(parent)?;
                dir.sync_all()?;
            }
        }

        Ok(())
    }

    #[cfg(not(unix))]
    {
        write_file(path, data)
    }
}

/// Create and sync a private file without replacing an existing path entry.
pub fn write_file_private_new(path: impl AsRef<Path>, data: &[u8]) -> Result<(), IoError> {
    let path = path.as_ref();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    #[cfg(unix)]
    file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    file.write_all(data)?;
    file.sync_all()?;
    if let Some(parent) = path.parent().filter(|value| !value.as_os_str().is_empty()) {
        std::fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

/// Stream bytes into an atomic temp-file replacement path.
pub fn atomic_write_file_streaming(
    path: impl AsRef<Path>,
    write_fn: impl FnOnce(&mut std::fs::File) -> Result<(), IoError>,
) -> Result<(), IoError> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    #[cfg(unix)]
    {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut temp = NamedTempFile::new_in(parent)?;

        {
            let permissions = std::fs::Permissions::from_mode(0o600);
            temp.as_file().set_permissions(permissions)?;
        }

        let file = temp.as_file_mut();
        write_fn(file)?;
        file.sync_all()?;

        temp.persist(path)
            .map(|_| ())
            .map_err(|e| IoError::Io(e.error))?;

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                let dir = std::fs::File::open(parent)?;
                dir.sync_all()?;
            }
        }

        Ok(())
    }

    #[cfg(not(unix))]
    {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut temp = NamedTempFile::new_in(parent)?;
        let file = temp.as_file_mut();

        write_fn(file)?;
        file.flush()?;

        temp.persist(path)
            .map(|_| ())
            .map_err(|e| IoError::Io(e.error))
    }
}
