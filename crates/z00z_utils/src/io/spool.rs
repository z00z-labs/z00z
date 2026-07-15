//! Private bounded spool-file support for rewindable verification streams.

use super::{IoError, Read, Seek, Write};
use std::io::{Error, ErrorKind, SeekFrom};
use std::path::Path;
use tempfile::NamedTempFile;

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

/// Descriptor identity captured when the exclusively-created spool is opened.
///
/// The path is deliberately never exposed.  The identity and metadata checks
/// below still make replacement, linking, truncation, append, or permission
/// drift fail closed before a trace pass consumes bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PrivateSpoolIdentity {
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}

/// A private, bounded, rewindable temporary spool file.
///
/// The temporary path never escapes this type.  Writes are allowed only before
/// the first rewind, so event replay cannot alter precommitted spool bytes.
pub struct PrivateSpoolFile {
    file: NamedTempFile,
    identity: PrivateSpoolIdentity,
    max_bytes: u64,
    written: u64,
    rewound: bool,
}

impl PrivateSpoolFile {
    /// Create an exclusively owned private spool under `dir`.
    pub fn create_in(dir: impl AsRef<Path>, max_bytes: u64) -> Result<Self, IoError> {
        if max_bytes == 0 {
            return Err(IoError::Io(Error::new(
                ErrorKind::InvalidInput,
                "private spool byte cap must be nonzero",
            )));
        }

        let mut file = tempfile::Builder::new()
            .prefix("z00z-private-spool-")
            .tempfile_in(dir)?;

        #[cfg(unix)]
        file.as_file_mut()
            .set_permissions(std::fs::Permissions::from_mode(0o600))?;

        let identity = Self::capture_identity(file.as_file())?;

        let spool = Self {
            file,
            identity,
            max_bytes,
            written: 0,
            rewound: false,
        };
        spool.verify_integrity()?;
        Ok(spool)
    }

    /// Append bytes while enforcing the configured aggregate cap.
    pub fn write_bounded(&mut self, bytes: &[u8]) -> Result<(), IoError> {
        if self.rewound {
            return Err(IoError::Io(Error::new(
                ErrorKind::PermissionDenied,
                "private spool is sealed after rewind",
            )));
        }
        self.verify_integrity()?;

        let incoming = u64::try_from(bytes.len()).map_err(|_| {
            IoError::Io(Error::new(
                ErrorKind::InvalidInput,
                "private spool input length does not fit u64",
            ))
        })?;
        let next = self.written.checked_add(incoming).ok_or_else(|| {
            IoError::Io(Error::new(
                ErrorKind::InvalidInput,
                "private spool byte counter overflow",
            ))
        })?;
        if next > self.max_bytes {
            return Err(IoError::FileTooLarge {
                size: next,
                max: self.max_bytes,
            });
        }

        self.file.as_file_mut().write_all(bytes)?;
        self.written = next;
        self.verify_integrity()?;
        Ok(())
    }

    /// Flush and seek to the immutable beginning of the committed spool.
    pub fn rewind(&mut self) -> Result<(), IoError> {
        self.verify_integrity()?;
        self.file.as_file_mut().sync_all()?;
        self.file.as_file_mut().seek(SeekFrom::Start(0))?;
        self.rewound = true;
        self.verify_integrity()?;
        Ok(())
    }

    /// Persist the current bytes to stable storage without exposing the path.
    pub fn sync(&mut self) -> Result<(), IoError> {
        self.verify_integrity()?;
        self.file.as_file_mut().sync_all()?;
        self.verify_integrity()?;
        Ok(())
    }

    /// Read committed bytes into a caller-provided bounded buffer.
    pub fn read_chunk(&mut self, buffer: &mut [u8]) -> Result<usize, IoError> {
        if !self.rewound {
            return Err(IoError::Io(Error::new(
                ErrorKind::InvalidInput,
                "private spool must be rewound before reading",
            )));
        }
        self.verify_integrity()?;
        let read = self.file.as_file_mut().read(buffer)?;
        self.verify_integrity()?;
        Ok(read)
    }

    /// Confirm that the sealed descriptor still names its original private
    /// regular file and exactly the bytes admitted by the bounded writer.
    ///
    /// This reveals neither a path nor a file handle.  Callers use it at
    /// protocol boundaries in addition to the checks built into read/write.
    pub fn verify_integrity(&self) -> Result<(), IoError> {
        let metadata = self.file.as_file().metadata()?;
        if !metadata.file_type().is_file() || metadata.len() != self.written {
            return Err(IoError::Io(Error::new(
                ErrorKind::InvalidData,
                "private spool identity or length changed",
            )));
        }

        #[cfg(unix)]
        {
            let identity = Self::capture_identity(self.file.as_file())?;
            if identity != self.identity
                || metadata.nlink() != 1
                || metadata.mode() & 0o777 != 0o600
            {
                return Err(IoError::Io(Error::new(
                    ErrorKind::PermissionDenied,
                    "private spool identity, link count, or mode changed",
                )));
            }
        }
        Ok(())
    }

    fn capture_identity(file: &std::fs::File) -> Result<PrivateSpoolIdentity, IoError> {
        let metadata = file.metadata()?;
        if !metadata.file_type().is_file() {
            return Err(IoError::Io(Error::new(
                ErrorKind::InvalidData,
                "private spool is not a regular file",
            )));
        }
        Ok(PrivateSpoolIdentity {
            #[cfg(unix)]
            device: metadata.dev(),
            #[cfg(unix)]
            inode: metadata.ino(),
        })
    }

    /// Return the immutable byte length recorded by the spool writer.
    #[must_use]
    pub const fn len(&self) -> u64 {
        self.written
    }

    /// Return whether no bytes have been recorded.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.written == 0
    }
}

#[cfg(test)]
mod tests {
    use super::PrivateSpoolFile;
    use crate::io::IoError;
    use tempfile::TempDir;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn spool_enforces_cap_and_seals_after_rewind() {
        let dir = TempDir::new().expect("temp dir");
        let mut spool = PrivateSpoolFile::create_in(dir.path(), 3).expect("spool");
        spool.write_bounded(b"abc").expect("bounded write");
        assert!(matches!(
            spool.write_bounded(b"d"),
            Err(IoError::FileTooLarge { size: 4, max: 3 })
        ));

        spool.rewind().expect("rewind");
        let mut out = [0u8; 3];
        assert_eq!(spool.read_chunk(&mut out).expect("read"), 3);
        assert_eq!(&out, b"abc");
        assert!(spool.write_bounded(b"x").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn spool_rejects_link_and_permission_identity_drift() {
        let dir = TempDir::new().expect("temp dir");
        let mut spool = PrivateSpoolFile::create_in(dir.path(), 8).expect("spool");
        spool.write_bounded(b"data").expect("bounded write");

        let link = dir.path().join("unexpected-private-spool-link");
        std::fs::hard_link(spool.file.path(), &link).expect("test hard link");
        assert!(spool.verify_integrity().is_err());
        std::fs::remove_file(link).expect("remove test link");
        spool.verify_integrity().expect("identity restored");

        spool
            .file
            .as_file_mut()
            .set_permissions(std::fs::Permissions::from_mode(0o644))
            .expect("mutate mode");
        assert!(spool.verify_integrity().is_err());
    }
}
