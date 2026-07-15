//! Stable rejection taxonomy for the sole recursive checkpoint V2 surface.

use thiserror::Error;
use z00z_crypto::CheckpointSha256Error;
use z00z_utils::io::IoError;

use crate::settlement::SettlementStateRoot;

/// Typed non-success outcomes for recursive checkpoint V2 construction.
#[derive(Debug, Error)]
pub enum RecursiveV2Error {
    #[error("unsupported recursive V2 version")]
    Version,
    #[error("recursive V2 resource limit exceeded")]
    Limit,
    #[error("recursive V2 Nova resource preflight rejected")]
    Resource,
    #[error("recursive V2 checked arithmetic overflow")]
    Overflow,
    #[error("recursive V2 invariant mismatch")]
    Invariant,
    #[error("recursive V2 canonical encoding mismatch")]
    Canonical,
    #[error("recursive V2 trace state mismatch")]
    TraceState,
    #[error("recursive V2 snapshot handle changed")]
    SnapshotChanged,
    #[error("recursive V2 event order mismatch")]
    EventOrder,
    #[error("recursive V2 duplicate identifier")]
    DuplicateIdentifier,
    #[error("recursive V2 cutover authority mismatch")]
    CutoverAuthority,
    #[error("recursive V2 authority resolution mismatch")]
    Authority,
    #[error("recursive V2 cutover install mismatch")]
    CutoverInstall,
    #[error("recursive V2 root mismatch")]
    Root,
    #[error("recursive V2 storage transition rejected")]
    Storage,
    #[error(
        "recursive V2 storage commit completed but its proving source is unavailable at generation {generation}"
    )]
    CommittedWithoutSource {
        root: SettlementStateRoot,
        generation: u64,
    },
    #[error("recursive V2 spool I/O failure: {0}")]
    Spool(#[from] IoError),
    #[error("recursive V2 SHA-256 input failure: {0}")]
    Sha256(#[from] CheckpointSha256Error),
}
