//! ConfigSource trait and error types

use crate::io::IoError;
use std::str::FromStr;
use thiserror::Error;

fn redact_config_value(value: &str) -> String {
    if value.is_empty() {
        return "<redacted empty value>".to_string();
    }

    let char_count = value.chars().count();
    format!("<redacted len={char_count}>")
}

/// Configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration key not found
    #[error("config key not found: {key}")]
    NotFound {
        /// The configuration key that was not found
        key: String,
    },

    /// Failed to parse config value
    #[error("failed to parse config '{key}' = '{value}': {error}")]
    Parse {
        /// The configuration key
        key: String,
        /// The configuration value
        value: String,
        /// The error message
        error: String,
    },

    /// YAML parsing error
    #[error("YAML parse error: {0}")]
    Yaml(String),

    /// YAML config file exceeds the supported size limit.
    #[error("config file too large: {size} bytes exceeds limit {max} bytes")]
    FileTooLarge {
        /// Actual file size.
        size: u64,
        /// Maximum supported file size.
        max: u64,
    },

    /// Configuration directory exceeds the supported entry-count limit.
    #[error("config directory contains more entries than limit {max}")]
    DirectoryTooLarge {
        /// Maximum supported number of directory entries.
        max: usize,
    },

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<IoError> for ConfigError {
    fn from(err: IoError) -> Self {
        match err {
            IoError::Io(err) => Self::Io(err),
            IoError::FileTooLarge { size, max } => Self::FileTooLarge { size, max },
            IoError::DirectoryTooLarge { max } => Self::DirectoryTooLarge { max },
            IoError::Serialization(err) | IoError::Deserialization(err) => Self::Yaml(err),
        }
    }
}

impl ConfigError {
    pub(crate) fn parse_value(key: &str, value: &str, error: impl Into<String>) -> Self {
        Self::Parse {
            key: key.to_string(),
            value: redact_config_value(value),
            error: error.into(),
        }
    }
}

/// Configuration source abstraction
///
/// Implementations of this trait provide configuration values from various sources
/// (environment variables, YAML files, etc.).
pub trait ConfigSource {
    /// Error type for this configuration source
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get config value as string (if exists)
    fn get(&self, key: &str) -> Result<Option<String>, Self::Error>;

    /// Get config value with type conversion
    ///
    /// Returns `Ok(None)` if the key is not found, and `Err` if the key exists
    /// but the value cannot be parsed into the target type. The error will include
    /// the original parse error details for debugging.
    fn get_typed<T>(&self, key: &str) -> Result<Option<T>, Self::Error>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
        Self::Error: From<ConfigError>,
    {
        match self.get(key)? {
            Some(s) => {
                let value = s
                    .parse::<T>()
                    .map_err(|e| ConfigError::parse_value(key, &s, e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}
