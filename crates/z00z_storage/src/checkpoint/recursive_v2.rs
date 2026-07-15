//! Sole public path for the live recursive checkpoint V2 contract.

// The Nova implementation is deliberately private to the storage owner.  In
// particular, no dependency-specific proof type crosses this facade.
pub(crate) mod nova;

pub use super::{
    canonical_transition::{
        CanonicalCheckpointTransitionV2, SettlementRootCutoverModeV2,
        SettlementRootGenerationCutoverV2,
    },
    recursive_circuit::{
        RecursiveCircuitProfileV2, RecursiveCircuitSpecV2, RECURSIVE_CIRCUIT_PROFILE_VERSION_V2,
        RECURSIVE_CIRCUIT_SPEC_VERSION_V2, RECURSIVE_V2_MAX_CONTENT_BYTES,
    },
    recursive_context::RecursiveAuthoritySnapshotV2,
    recursive_predicate::EvaluatedCheckpointTransitionV2,
    recursive_reject::RecursiveV2Error,
    recursive_statement::RecursiveTransitionStatementV2,
    recursive_trace::RecursiveTracePrecommitV2,
};
