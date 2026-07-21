use super::{atomic_write_with_context, load_with_context, IoError};
use crate::codec::Codec;
use serde::{de::DeserializeOwned, Serialize};
use std::io::Read;
use std::path::Path;

const DEFAULT_MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Read raw bytes from a file, enforcing a maximum size.
///
/// # Security
///
/// This function prevents unbounded reads that can cause memory exhaustion when
/// parsing untrusted files or special devices.
pub fn read_file_bounded(path: impl AsRef<Path>, max_bytes: u64) -> Result<Vec<u8>, IoError> {
    let path = path.as_ref();
    let file = std::fs::File::open(path)?;
    let take_limit = max_bytes.checked_add(1).ok_or_else(|| {
        IoError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "file byte limit must allow a one-byte overflow probe",
        ))
    })?;

    // Descriptor metadata is an fstat on Unix; failure must not be ignored.
    let metadata = file.metadata()?;
    if metadata.is_file() {
        let size = metadata.len();
        if size > max_bytes {
            return Err(IoError::FileTooLarge {
                size,
                max: max_bytes,
            });
        }
    }

    let mut bytes = Vec::new();
    file.take(take_limit).read_to_end(&mut bytes)?;

    if (bytes.len() as u64) > max_bytes {
        return Err(IoError::FileTooLarge {
            size: bytes.len() as u64,
            max: max_bytes,
        });
    }

    Ok(bytes)
}

/// Save a value to a file using a codec.
pub fn save_with_codec<T: Serialize, C: Codec>(
    path: impl AsRef<Path>,
    value: &T,
    codec: C,
) -> Result<(), IoError>
where
    C::Error: Into<IoError>,
{
    let path = path.as_ref();
    let codec_name = codec.name();

    atomic_write_with_context(
        path,
        value,
        |item| codec.serialize(item).map_err(|err| err.to_string()),
        codec_name,
    )
}

/// Load a value from a file using a codec.
pub fn load_with_codec<T: DeserializeOwned, C: Codec>(
    path: impl AsRef<Path>,
    codec: C,
) -> Result<T, IoError>
where
    C::Error: Into<IoError>,
{
    let path = path.as_ref();
    let codec_name = codec.name();

    load_with_context(
        path,
        |bytes| codec.deserialize(bytes).map_err(|err| err.to_string()),
        codec_name,
    )
}

pub(super) fn default_max_file_size() -> u64 {
    DEFAULT_MAX_FILE_SIZE
}
