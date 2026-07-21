//! I/O error types

use thiserror::Error;

/// Errors that can occur during I/O operations
#[derive(Debug, Error)]
pub enum IoError {
    /// Standard I/O error (file not found, permission denied, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// File too large for the configured limit.
    #[error("File too large: {size} bytes exceeds limit {max} bytes")]
    FileTooLarge {
        /// Actual number of bytes observed or read.
        size: u64,
        /// Maximum allowed number of bytes.
        max: u64,
    },

    /// Directory contains more entries than the configured limit.
    #[error("Directory contains more entries than limit {max}")]
    DirectoryTooLarge {
        /// Maximum allowed number of directory entries.
        max: usize,
    },

    /// Serialization error
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("deserialization error: {0}")]
    Deserialization(String),
}

#[cfg(not(target_arch = "wasm32"))]
impl From<tempfile::PersistError> for IoError {
    fn from(err: tempfile::PersistError) -> Self {
        IoError::Io(err.error)
    }
}
