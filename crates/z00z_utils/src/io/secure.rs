//! Descriptor-anchored private filesystem operations.

use super::IoError;
use std::{ffi::OsString, fs::File, path::Path};

/// A private directory handle used for race-resistant evidence and cache I/O.
///
/// On Unix, the root is opened component by component without following
/// symbolic links. All entry operations are then relative to the retained
/// directory descriptor and accept one basename only. Unsupported platforms
/// fail closed instead of falling back to path-based operations.
#[derive(Debug)]
pub struct SecureDir {
    #[cfg(unix)]
    dir: File,
    #[cfg(not(unix))]
    _unsupported: (),
}

impl SecureDir {
    /// Open or atomically create the final component of a private directory
    /// path without following any component symlink. Existing parents are
    /// traversed read-only and the final component must be mode `0700`.
    pub fn ensure_private(path: impl AsRef<Path>) -> Result<Self, IoError> {
        #[cfg(unix)]
        {
            unix::ensure_private_dir(path.as_ref())
        }

        #[cfg(not(unix))]
        {
            let _ = path;
            Err(unsupported_error())
        }
    }

    /// Open an existing private directory without following symbolic links.
    ///
    /// The final directory must have Unix mode `0700`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, IoError> {
        #[cfg(unix)]
        {
            unix::open_dir(path.as_ref())
        }

        #[cfg(not(unix))]
        {
            let _ = path;
            Err(unsupported_error())
        }
    }

    /// Open or create a private lock file without truncating it.
    ///
    /// The entry name must be one basename. On Unix, symbolic links and
    /// non-regular entries are rejected, and the returned descriptor is set
    /// to and verified as mode `0600`.
    pub fn open_lock(&self, name: impl AsRef<Path>) -> Result<File, IoError> {
        #[cfg(unix)]
        {
            unix::open_lock(self, name.as_ref())
        }

        #[cfg(not(unix))]
        {
            let _ = (self, name);
            Err(unsupported_error())
        }
    }

    /// Open one existing private child directory without following links.
    ///
    /// The entry name must be one basename and the returned directory
    /// descriptor is verified as mode `0700`.
    pub fn open_child(&self, name: impl AsRef<Path>) -> Result<Self, IoError> {
        #[cfg(unix)]
        {
            unix::open_child(self, name.as_ref())
        }

        #[cfg(not(unix))]
        {
            let _ = (self, name);
            Err(unsupported_error())
        }
    }

    /// Create or open one private child directory without following links.
    ///
    /// The returned descriptor is set to and verified as mode `0700`.
    pub fn ensure_dir(&self, name: impl AsRef<Path>) -> Result<Self, IoError> {
        #[cfg(unix)]
        {
            unix::ensure_dir(self, name.as_ref())
        }

        #[cfg(not(unix))]
        {
            let _ = (self, name);
            Err(unsupported_error())
        }
    }

    /// Create a new private regular file without replacing any entry.
    ///
    /// The returned descriptor is set to and verified as mode `0600`.
    pub fn create_file(&self, name: impl AsRef<Path>) -> Result<File, IoError> {
        #[cfg(unix)]
        {
            unix::create_file(self, name.as_ref())
        }

        #[cfg(not(unix))]
        {
            let _ = (self, name);
            Err(unsupported_error())
        }
    }

    /// Read one private regular file while enforcing a byte limit.
    ///
    /// The file is opened without following links, then its returned
    /// descriptor is checked for regular-file type, mode `0600`, and size.
    pub fn read_file_bounded(
        &self,
        name: impl AsRef<Path>,
        max_bytes: u64,
    ) -> Result<Vec<u8>, IoError> {
        #[cfg(unix)]
        {
            unix::read_file_bounded(self, name.as_ref(), max_bytes)
        }

        #[cfg(not(unix))]
        {
            let _ = (self, name, max_bytes);
            Err(unsupported_error())
        }
    }

    /// Read sorted entry basenames while enforcing an entry-count limit.
    ///
    /// At most `max_entries + 1` names are collected. Dot entries are omitted.
    pub fn read_dir_bounded(&self, max_entries: usize) -> Result<Vec<OsString>, IoError> {
        #[cfg(unix)]
        {
            unix::read_dir_bounded(self, max_entries)
        }

        #[cfg(not(unix))]
        {
            let _ = (self, max_entries);
            Err(unsupported_error())
        }
    }

    /// Remove one entry without following it.
    pub fn remove_file(&self, name: impl AsRef<Path>) -> Result<(), IoError> {
        #[cfg(unix)]
        {
            unix::remove_file(self, name.as_ref())
        }

        #[cfg(not(unix))]
        {
            let _ = (self, name);
            Err(unsupported_error())
        }
    }

    /// Rename one entry without replacing an existing destination.
    pub fn rename_no_clobber(
        &self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> Result<(), IoError> {
        self.rename_to_no_clobber(from, self, to)
    }

    /// Rename one entry to another secure directory without replacement.
    pub fn rename_to_no_clobber(
        &self,
        from: impl AsRef<Path>,
        target: &Self,
        to: impl AsRef<Path>,
    ) -> Result<(), IoError> {
        #[cfg(unix)]
        {
            unix::rename_no_clobber(self, from.as_ref(), target, to.as_ref())
        }

        #[cfg(not(unix))]
        {
            let _ = (self, from, target, to);
            Err(unsupported_error())
        }
    }

    /// Flush directory entry changes to stable storage.
    pub fn sync(&self) -> Result<(), IoError> {
        #[cfg(unix)]
        {
            self.dir.sync_all()?;
            Ok(())
        }

        #[cfg(not(unix))]
        {
            let _ = self;
            Err(unsupported_error())
        }
    }
}

#[cfg(not(unix))]
fn unsupported_error() -> IoError {
    IoError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "secure descriptor-anchored filesystem operations are unavailable",
    ))
}

#[cfg(unix)]
mod unix {
    use super::{File, IoError, OsString, Path, SecureDir};
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    use rustix::fs::RenameFlags;
    use rustix::fs::{
        fchmod, fstat, mkdirat, open, openat, unlinkat, AtFlags, Dir, FileType, Mode, OFlags,
    };
    use std::{
        ffi::OsStr,
        io::{Error, ErrorKind, Read},
        os::unix::ffi::OsStrExt,
        path::Component,
    };

    const PRIVATE_DIR_MODE: Mode = Mode::RWXU;
    const PRIVATE_FILE_MODE: Mode = Mode::RUSR.union(Mode::WUSR);

    pub(super) fn open_dir(path: &Path) -> Result<SecureDir, IoError> {
        let dir = open_dir_no_follow(path)?;
        let stat = fstat(&dir).map_err(Error::from)?;
        if FileType::from_raw_mode(stat.st_mode) != FileType::Directory {
            return Err(invalid_data("secure root is not a directory"));
        }
        if Mode::from_raw_mode(stat.st_mode) != PRIVATE_DIR_MODE {
            return Err(permission_denied("secure root mode must be 0700"));
        }
        Ok(SecureDir { dir })
    }

    pub(super) fn ensure_private_dir(path: &Path) -> Result<SecureDir, IoError> {
        if path.as_os_str().is_empty() {
            return Err(invalid_input("secure root path must not be empty"));
        }
        let flags = OFlags::RDONLY
            | OFlags::DIRECTORY
            | OFlags::NOFOLLOW
            | OFlags::CLOEXEC
            | OFlags::NONBLOCK;
        let start = if path.is_absolute() {
            Path::new("/")
        } else {
            Path::new(".")
        };
        let mut dir: File = open(start, flags, Mode::empty())
            .map_err(Error::from)?
            .into();
        let mut components = path.components().peekable();
        let mut saw_final = false;
        while let Some(component) = components.next() {
            match component {
                Component::RootDir | Component::CurDir => {}
                Component::Normal(name) if components.peek().is_none() => {
                    let created = match mkdirat(&dir, name, PRIVATE_DIR_MODE) {
                        Ok(()) => true,
                        Err(rustix::io::Errno::EXIST) => false,
                        Err(error) => return Err(IoError::Io(Error::from(error))),
                    };
                    dir = openat(&dir, name, flags, Mode::empty())
                        .map_err(Error::from)?
                        .into();
                    let stat = fstat(&dir).map_err(Error::from)?;
                    if FileType::from_raw_mode(stat.st_mode) != FileType::Directory {
                        return Err(invalid_data("secure root is not a directory"));
                    }
                    if created {
                        fchmod(&dir, PRIVATE_DIR_MODE).map_err(Error::from)?;
                    }
                    let stat = fstat(&dir).map_err(Error::from)?;
                    if Mode::from_raw_mode(stat.st_mode) != PRIVATE_DIR_MODE {
                        return Err(permission_denied("secure root mode must be 0700"));
                    }
                    saw_final = true;
                }
                Component::Normal(name) => {
                    dir = openat(&dir, name, flags, Mode::empty())
                        .map_err(Error::from)?
                        .into();
                }
                Component::ParentDir | Component::Prefix(_) => {
                    return Err(invalid_input(
                        "secure root path must not contain parent or prefix components",
                    ));
                }
            }
        }
        if !saw_final {
            return Err(invalid_input(
                "secure root path must name a non-root final component",
            ));
        }
        Ok(SecureDir { dir })
    }

    pub(super) fn open_lock(dir: &SecureDir, name: &Path) -> Result<File, IoError> {
        let name = entry_name(name)?;
        let flags =
            OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK;
        let fd = openat(&dir.dir, name, flags, PRIVATE_FILE_MODE).map_err(Error::from)?;
        private_file(fd.into(), true)
    }

    pub(super) fn open_child(dir: &SecureDir, name: &Path) -> Result<SecureDir, IoError> {
        child_dir(dir, name, false)
    }

    pub(super) fn ensure_dir(dir: &SecureDir, name: &Path) -> Result<SecureDir, IoError> {
        let name = entry_name(name)?;
        match mkdirat(&dir.dir, name, PRIVATE_DIR_MODE) {
            Ok(()) | Err(rustix::io::Errno::EXIST) => child_dir(dir, Path::new(name), true),
            Err(err) => Err(IoError::Io(Error::from(err))),
        }
    }

    pub(super) fn create_file(dir: &SecureDir, name: &Path) -> Result<File, IoError> {
        let name = entry_name(name)?;
        let flags = OFlags::WRONLY
            | OFlags::CREATE
            | OFlags::EXCL
            | OFlags::NOFOLLOW
            | OFlags::CLOEXEC
            | OFlags::NONBLOCK;
        let fd = openat(&dir.dir, name, flags, PRIVATE_FILE_MODE).map_err(Error::from)?;
        private_file(fd.into(), true)
    }

    pub(super) fn read_file_bounded(
        dir: &SecureDir,
        name: &Path,
        max_bytes: u64,
    ) -> Result<Vec<u8>, IoError> {
        let read_limit = max_bytes.checked_add(1).ok_or_else(|| {
            invalid_input("secure file byte limit must allow a one-byte overflow probe")
        })?;
        let name = entry_name(name)?;
        let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK;
        let fd = openat(&dir.dir, name, flags, Mode::empty()).map_err(Error::from)?;
        let file: File = fd.into();
        let stat = regular_stat(&file)?;
        if Mode::from_raw_mode(stat.st_mode) != PRIVATE_FILE_MODE {
            return Err(permission_denied("secure file mode must be 0600"));
        }
        let size = u64::try_from(stat.st_size)
            .map_err(|_| invalid_data("secure file reported a negative size"))?;
        if size > max_bytes {
            return Err(IoError::FileTooLarge {
                size,
                max: max_bytes,
            });
        }

        let mut bytes = Vec::new();
        file.take(read_limit).read_to_end(&mut bytes)?;
        if (bytes.len() as u64) > max_bytes {
            return Err(IoError::FileTooLarge {
                size: bytes.len() as u64,
                max: max_bytes,
            });
        }
        Ok(bytes)
    }

    pub(super) fn read_dir_bounded(
        dir: &SecureDir,
        max_entries: usize,
    ) -> Result<Vec<OsString>, IoError> {
        let probe_limit = max_entries.checked_add(1).ok_or_else(|| {
            invalid_input("directory entry limit must allow a one-entry overflow probe")
        })?;
        let mut stream = Dir::read_from(&dir.dir).map_err(Error::from)?;
        let mut names = Vec::with_capacity(probe_limit.min(256));

        while let Some(entry) = stream.read() {
            let entry = entry.map_err(Error::from)?;
            let bytes = entry.file_name().to_bytes();
            if bytes == b"." || bytes == b".." {
                continue;
            }
            names.push(OsStr::from_bytes(bytes).to_os_string());
            if names.len() == probe_limit {
                return Err(IoError::DirectoryTooLarge { max: max_entries });
            }
        }

        names.sort();
        Ok(names)
    }

    pub(super) fn remove_file(dir: &SecureDir, name: &Path) -> Result<(), IoError> {
        let name = entry_name(name)?;
        unlinkat(&dir.dir, name, AtFlags::empty()).map_err(Error::from)?;
        Ok(())
    }

    pub(super) fn rename_no_clobber(
        source: &SecureDir,
        from: &Path,
        target: &SecureDir,
        to: &Path,
    ) -> Result<(), IoError> {
        let from = entry_name(from)?;
        let to = entry_name(to)?;

        #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
        {
            rustix::fs::renameat_with(&source.dir, from, &target.dir, to, RenameFlags::NOREPLACE)
                .map_err(Error::from)?;
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
        {
            let _ = (source, from, target, to);
            Err(IoError::Io(Error::new(
                ErrorKind::Unsupported,
                "atomic no-clobber rename is unavailable",
            )))
        }
    }

    fn open_dir_no_follow(path: &Path) -> Result<File, IoError> {
        if path.as_os_str().is_empty() {
            return Err(invalid_input("secure root path must not be empty"));
        }

        let flags = OFlags::RDONLY
            | OFlags::DIRECTORY
            | OFlags::NOFOLLOW
            | OFlags::CLOEXEC
            | OFlags::NONBLOCK;
        let start = if path.is_absolute() {
            Path::new("/")
        } else {
            Path::new(".")
        };
        let mut dir: File = open(start, flags, Mode::empty())
            .map_err(Error::from)?
            .into();

        for component in path.components() {
            match component {
                Component::RootDir | Component::CurDir => {}
                Component::Normal(name) => {
                    dir = openat(&dir, name, flags, Mode::empty())
                        .map_err(Error::from)?
                        .into();
                }
                Component::ParentDir | Component::Prefix(_) => {
                    return Err(invalid_input(
                        "secure root path must not contain parent or prefix components",
                    ));
                }
            }
        }
        Ok(dir)
    }

    fn child_dir(parent: &SecureDir, name: &Path, set_mode: bool) -> Result<SecureDir, IoError> {
        let name = entry_name(name)?;
        let flags = OFlags::RDONLY
            | OFlags::DIRECTORY
            | OFlags::NOFOLLOW
            | OFlags::CLOEXEC
            | OFlags::NONBLOCK;
        let fd = openat(&parent.dir, name, flags, Mode::empty()).map_err(Error::from)?;
        let dir: File = fd.into();
        let stat = fstat(&dir).map_err(Error::from)?;
        if FileType::from_raw_mode(stat.st_mode) != FileType::Directory {
            return Err(invalid_data("secure child is not a directory"));
        }
        if set_mode {
            fchmod(&dir, PRIVATE_DIR_MODE).map_err(Error::from)?;
        }
        let stat = fstat(&dir).map_err(Error::from)?;
        if Mode::from_raw_mode(stat.st_mode) != PRIVATE_DIR_MODE {
            return Err(permission_denied("secure child mode must be 0700"));
        }
        Ok(SecureDir { dir })
    }

    fn entry_name(path: &Path) -> Result<&OsStr, IoError> {
        let bytes = path.as_os_str().as_bytes();
        if bytes.is_empty() || bytes == b"." || bytes == b".." || bytes.contains(&b'/') {
            return Err(invalid_input(
                "secure entry name must be one non-dot basename",
            ));
        }
        Ok(path.as_os_str())
    }

    fn private_file(file: File, set_mode: bool) -> Result<File, IoError> {
        regular_stat(&file)?;
        if set_mode {
            fchmod(&file, PRIVATE_FILE_MODE).map_err(Error::from)?;
        }
        let stat = regular_stat(&file)?;
        if Mode::from_raw_mode(stat.st_mode) != PRIVATE_FILE_MODE {
            return Err(permission_denied("secure file mode must be 0600"));
        }
        Ok(file)
    }

    fn regular_stat(file: &File) -> Result<rustix::fs::Stat, IoError> {
        let stat = fstat(file).map_err(Error::from)?;
        if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile {
            return Err(invalid_data("secure entry is not a regular file"));
        }
        Ok(stat)
    }

    fn invalid_input(message: &'static str) -> IoError {
        IoError::Io(Error::new(ErrorKind::InvalidInput, message))
    }

    fn invalid_data(message: &'static str) -> IoError {
        IoError::Io(Error::new(ErrorKind::InvalidData, message))
    }

    fn permission_denied(message: &'static str) -> IoError {
        IoError::Io(Error::new(ErrorKind::PermissionDenied, message))
    }
}
