//! Temporary directory ownership behind the Z00Z I/O facade.

use std::path::Path;

use super::{set_permissions_mode, IoError};

/// A uniquely named temporary directory removed when its owner is dropped.
pub struct TemporaryDirectory {
    inner: tempfile::TempDir,
}

impl TemporaryDirectory {
    /// Create a new temporary directory using the platform temp root.
    pub fn new() -> Result<Self, IoError> {
        let inner = tempfile::tempdir()?;
        set_permissions_mode(inner.path(), 0o700)?;
        Ok(Self { inner })
    }

    /// Return the owned directory path.
    pub fn path(&self) -> &Path {
        self.inner.path()
    }
}

#[cfg(test)]
mod tests {
    use super::TemporaryDirectory;

    #[test]
    fn owns_existing_directory() {
        let directory = TemporaryDirectory::new().expect("temporary directory");
        assert!(directory.path().is_dir());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = directory
                .path()
                .metadata()
                .expect("temporary directory metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o700);
        }
    }
}
