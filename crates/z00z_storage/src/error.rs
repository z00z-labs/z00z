use thiserror::Error;
use z00z_utils::{codec::CodecError, io::IoError};

/// Errors for checkpoint type validation, identity derivation, and key checks.
///
/// # Examples
///
/// ```
/// use z00z_storage::CheckpointError;
///
/// let err = CheckpointError::VersionMix;
/// assert_eq!(err.to_string(), "unsupported checkpoint version");
/// ```
#[derive(Debug, Error)]
pub enum CheckpointError {
    /// Final artifacts must carry non-empty proof bytes.
    #[error("proofless final artifact")]
    ProoflessFinal,
    /// Final proof inputs do not match the checkpoint draft boundary.
    #[error("checkpoint proof mismatch")]
    ProofMix,
    /// Final artifacts must declare one supported proof system explicitly.
    #[error("unsupported checkpoint proof system")]
    ProofSysMix,
    /// noncanonical prior-final artifact bytes or write paths crossed the
    /// canonical draft/final checkpoint class boundary.
    #[error("checkpoint artifact noncanonical mismatch")]
    ArtifactCompatMix,
    /// One checkpoint-link edge is inconsistent.
    #[error("checkpoint link mismatch")]
    LinkMix,
    /// One checkpoint lifecycle transition is inconsistent.
    #[error("checkpoint lifecycle mismatch")]
    LifecycleMix,
    /// One execution-input binding is inconsistent.
    #[error("checkpoint replay-input mismatch")]
    ReplayMix,
    /// One archive-retention object is inconsistent.
    #[error("checkpoint archive-retention mismatch")]
    ArchiveMix,
    /// One checkpoint snapshot object is inconsistent.
    #[error("checkpoint snapshot mismatch")]
    SnapshotMix,
    /// One checkpoint pruning decision is unsafe.
    #[error("checkpoint pruning mismatch")]
    PruningMix,
    /// One root boundary is inconsistent.
    #[error("checkpoint root mismatch")]
    RootMix,
    /// Artifact class is not accepted by the requested path.
    #[error("wrong checkpoint artifact class")]
    WrongClass,
    /// Schema version is not supported.
    #[error("unsupported checkpoint version")]
    VersionMix,
    /// Serialization or deserialization failed.
    #[error("checkpoint codec error: {0}")]
    Codec(#[from] CodecError),
    /// Persisted backend key does not match the expected external id.
    #[error("checkpoint backend key mismatch")]
    KeyMix,
    /// Backend-specific storage failure.
    #[error("checkpoint backend failure: {0}")]
    Backend(String),
    /// Checkpoint contract configuration is missing, malformed, or unsafe.
    #[error("checkpoint contract config error: {0}")]
    ContractConfig(String),
    /// Nova is a classical non-authoritative branch and cannot claim PQ authority.
    #[error("Nova PQ authority unsupported")]
    NovaPqAuthorityUnsupported,
}

/// Errors for deterministic JMT serialization, reconstruction, and rendering.
///
/// # Examples
///
/// ```
/// use z00z_storage::SerializationError;
///
/// let err = SerializationError::VersionMix;
/// assert_eq!(err.to_string(), "unsupported serialization version");
/// ```
#[derive(Debug, Error)]
pub enum SerializationError {
    /// Serialization or deserialization failed.
    #[error("serialization codec error: {0}")]
    Codec(#[from] CodecError),
    /// Storage I/O failed.
    #[error("serialization io error: {0}")]
    Io(#[from] IoError),
    /// Schema version is not supported.
    #[error("unsupported serialization version")]
    VersionMix,
    /// One serialized node shape is not supported by the storage-owned contract.
    #[error("unsupported serialized node kind")]
    NodeKindMix,
    /// One serialized root boundary is inconsistent.
    #[error("serialization root mismatch")]
    RootMix,
    /// Reconstructed topology diverges from serialized structure.
    #[error("serialization reconstruction mismatch")]
    RebuildMix,
    /// Human-readable rendering failed or diverged from the typed artifact.
    #[error("serialization visualization mismatch")]
    ViewMix,
    /// Backend-specific persistence failure.
    #[error("serialization backend failure: {0}")]
    Backend(String),
}
