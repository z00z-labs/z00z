use thiserror::Error;
use z00z_crypto::CheckpointSha256Error;
use z00z_utils::{codec::CodecError, io::IoError};

use crate::{
    checkpoint::recursive_reject::RecursiveCheckpointRejectReasonV2,
    settlement::SettlementStateRoot,
};

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
    /// Stable recursive V2 admission rejection exposed on the wire.
    #[error("recursive checkpoint V2 rejected: {0}")]
    RecursiveRejected(RecursiveCheckpointRejectReasonV2),
    /// Stable reject-reason bytes were malformed or carried an unknown code.
    #[error("recursive checkpoint V2 reject-reason codec mismatch")]
    RecursiveRejectCodec,
    /// Recursive V2 operational version mismatch.
    #[error("unsupported recursive V2 version")]
    Version,
    /// Recursive V2 operational resource limit was exceeded.
    #[error("recursive V2 resource limit exceeded")]
    Limit,
    /// Recursive V2 Nova resource preflight rejected the requested work.
    #[error("recursive V2 Nova resource preflight rejected")]
    Resource,
    /// Recursive V2 work was cancelled before receipt issuance.
    #[error("recursive V2 operation cancelled")]
    RecursiveCancelled,
    /// Recursive V2 work exceeded its authority-pinned deadline.
    #[error("recursive V2 operation deadline exceeded")]
    RecursiveDeadline,
    /// Recursive V2 checked arithmetic overflowed.
    #[error("recursive V2 checked arithmetic overflow")]
    Overflow,
    /// An internal recursive V2 invariant failed.
    #[error("recursive V2 invariant mismatch")]
    Invariant,
    /// The selected recursive V2 backend failed cryptographic verification.
    #[error("recursive V2 backend verification failed")]
    BackendVerificationFailed,
    /// Recursive V2 operational bytes were noncanonical.
    #[error("recursive V2 canonical encoding mismatch")]
    Canonical,
    /// Recursive V2 trace state was inconsistent.
    #[error("recursive V2 trace state mismatch")]
    TraceState,
    /// Recursive V2 snapshot identity changed during one operation.
    #[error("recursive V2 snapshot handle changed")]
    SnapshotChanged,
    /// Recursive V2 events arrived out of order.
    #[error("recursive V2 event order mismatch")]
    EventOrder,
    /// Recursive V2 input contained a duplicate identifier.
    #[error("recursive V2 duplicate identifier")]
    DuplicateIdentifier,
    /// Recursive V2 cutover authority was inconsistent.
    #[error("recursive V2 cutover authority mismatch")]
    CutoverAuthority,
    /// Recursive V2 authority resolution was inconsistent.
    #[error("recursive V2 authority resolution mismatch")]
    Authority,
    /// Recursive V2 cutover installation was inconsistent.
    #[error("recursive V2 cutover install mismatch")]
    CutoverInstall,
    /// Recursive V2 root derivation or linkage was inconsistent.
    #[error("recursive V2 root mismatch")]
    Root,
    /// Recursive V2 storage transition was rejected operationally.
    #[error("recursive V2 storage transition rejected")]
    Storage,
    /// Storage committed but the corresponding proving source became unavailable.
    #[error(
        "recursive V2 storage commit completed but its proving source is unavailable at generation {generation}"
    )]
    CommittedWithoutSource {
        root: SettlementStateRoot,
        generation: u64,
    },
    /// Recursive V2 spool I/O failed.
    #[error("recursive V2 spool I/O failure: {0}")]
    Spool(#[from] IoError),
    /// Recursive V2 SHA-256 input framing failed.
    #[error("recursive V2 SHA-256 input failure: {0}")]
    Sha256(#[from] CheckpointSha256Error),
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
