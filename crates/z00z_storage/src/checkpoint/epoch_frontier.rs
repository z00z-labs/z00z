//! Crash-durable frontier for actual-verifier-admitted Plonky3 trace chunks.
//!
//! Proof bodies are local shadow evidence. Compact chunk bindings survive until
//! epoch seal; child proof bodies are removed only after a verified parent and
//! retirement journal entry are both durable.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::ErrorKind,
    sync::atomic::{AtomicU64, Ordering},
};

use fs2::FileExt;
use z00z_crypto::sha256_256;
use z00z_utils::io::{IoError, SecureDir, Write};

#[cfg(test)]
use super::epoch_prover::{
    epoch_stream_initial_accumulator, epoch_stream_step_accumulator,
    EpochProofWorkManifestInputsV2, EpochTraceChunkInputsV2, EpochTransitionInputsV2,
    EPOCH_TRANSITION_SLICE_DOMAIN_V2,
};
use super::{
    contract_config_v3::{ActiveCheckpointConfigIdentityV3, ConfigV3ActivationStore},
    epoch_prover::{
        EpochAirTableV2, EpochProofWorkManifestV2, EpochTraceChunkV2, EpochTransitionBindingV2,
        EPOCH_CHUNK_BYTES_V2, EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2,
        EPOCH_TRANSITION_BINDING_BYTES_V2,
    },
    epoch_range::{
        epoch_ordered_digest_root_v2, epoch_ordered_digest_span_root_v2,
        epoch_verified_trace_chunk_span_digest_v2, EpochCadenceClassV2, EpochCodecReaderV2,
        EpochRangeInputsV2, EpochRangeStatementV2, EPOCH_ARTIFACT_ROOT_DOMAIN_V2,
        EPOCH_CHALLENGE_ROOT_DOMAIN_V2, EPOCH_DA_ROOT_DOMAIN_V2, EPOCH_DELTA_ROOT_DOMAIN_V2,
        EPOCH_LINK_ROOT_DOMAIN_V2, EPOCH_STATEMENT_ROOT_DOMAIN_V2,
        EPOCH_VERIFIED_TRACE_CHUNK_ROOT_DOMAIN_V2, EPOCH_WITNESS_ROOT_DOMAIN_V2,
    },
    plonky3::{
        Plonky3EpochChunkProofV2, Plonky3HistoryAuthorityResolverV2,
        ResolvedPlonky3HistoryAuthorityV2, VerifiedEpochTraceChunkAdmissionV2,
    },
    recursive_reject::RecursiveCheckpointRejectReasonV2,
    version_registry::{PLONKY3_PUBLISH_BYTES_V2, RECURSIVE_INGRESS_BYTES_V2},
};
use crate::CheckpointError;

const FRONTIER_AUTHORITY_MAGIC_V2: [u8; 8] = *b"Z00ZEFA5";
const FRONTIER_CHUNK_MAGIC_V2: [u8; 8] = *b"Z00ZEFC5";
const FRONTIER_NODE_MAGIC_V2: [u8; 8] = *b"Z00ZEFN5";
const FRONTIER_JOURNAL_MAGIC_V2: [u8; 8] = *b"Z00ZEFJ5";
const FRONTIER_WIRE_VERSION_V2: u16 = 5;
// Generation 9 additionally binds the actual proof and verification receipt
// into durable chunk admission identity. There is no decoder or migration path
// from predecessor frontier generations.
const FRONTIER_TREE_GENERATION_V2: u8 = 10;
const FRONTIER_AUTHORITY_DIGEST_COUNT_V2: usize = 10;
const FRONTIER_AUTHORITY_BYTES_V2: usize =
    8 + 2 + 1 + 1 + 8 * 4 + 4 * 2 + 2 + 8 * 4 + 4 + FRONTIER_AUTHORITY_DIGEST_COUNT_V2 * 32 + 32;
const FRONTIER_CHUNK_FIXED_BYTES_V2: usize = 8 + 2 + 1 + 4 * 4 + EPOCH_CHUNK_BYTES_V2 + 32 * 3;
const FRONTIER_CHUNK_MAX_BYTES_V2: usize = FRONTIER_CHUNK_FIXED_BYTES_V2
    + EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 as usize * EPOCH_TRANSITION_BINDING_BYTES_V2;
const FRONTIER_NODE_DIGEST_COUNT_V2: usize = 12;
const FRONTIER_NODE_PREFIX_BYTES_V2: usize =
    8 + 2 + 1 + 1 + 8 * 3 + 4 + 8 * 2 + 4 + 2 + FRONTIER_NODE_DIGEST_COUNT_V2 * 32 + 4;
const FRONTIER_JOURNAL_BYTES_V2: usize = 8 + 2 + 1 + 8 * 3 + 4 + 32 * 4;
const FRONTIER_MAX_JOURNAL_ENTRIES_V2: usize = 32_768;
const FRONTIER_MAX_CHUNK_FILES_V2: usize = 4_096;
const FRONTIER_MAX_NODE_FILES_V2: usize = 8_192;
const FRONTIER_MAX_ROOT_ENTRIES_V2: usize = 8;
const FRONTIER_NODE_MAX_BYTES_V2: usize =
    RECURSIVE_INGRESS_BYTES_V2 + FRONTIER_NODE_PREFIX_BYTES_V2 + 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochFrontierAuthorityInputsV2 {
    pub cadence_class: EpochCadenceClassV2,
    pub epoch_index: u64,
    pub cadence_blocks: u64,
    pub start_root: [u8; 32],
    pub chain_context_digest: [u8; 32],
    pub predicate_digest: [u8; 32],
    pub parameter_digest: [u8; 32],
    pub verifier_bundle_digest: [u8; 32],
}

/// Static authority known at open-epoch time. Close-only roots are deliberately
/// absent and are checked later against compact trace-chunk records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochFrontierAuthorityV2 {
    cadence_class: EpochCadenceClassV2,
    epoch_index: u64,
    start_height: u64,
    end_height: u64,
    cadence_blocks: u64,
    transition_count: u32,
    chunk_count: u32,
    runtime_profile_generation: u16,
    config_generation: u64,
    authority_generation: u64,
    activation_height: u64,
    rollback_floor: u64,
    parameter_generation: u32,
    start_root: [u8; 32],
    chain_context_digest: [u8; 32],
    predicate_digest: [u8; 32],
    parameter_digest: [u8; 32],
    verifier_bundle_digest: [u8; 32],
    security_budget_digest: [u8; 32],
    config_digest: [u8; 32],
    registry_digest: [u8; 32],
    runtime_profile_manifest_digest: [u8; 32],
    history_authority_bundle_digest: [u8; 32],
    digest: [u8; 32],
}

impl EpochFrontierAuthorityV2 {
    pub fn new(inputs: EpochFrontierAuthorityInputsV2) -> Result<Self, CheckpointError> {
        let resolved = Plonky3HistoryAuthorityResolverV2::resolve_active()?;
        Self::new_with_authority(inputs, &resolved)
    }

    /// Reconstruct an unfinished frontier only from an exact trusted
    /// generation identity and its immutable installed authority bundle.
    ///
    /// Neither persisted frontier bytes nor a generation number select the
    /// historical authority, and there is no fallback to the current config.
    pub fn resolve_installed(
        inputs: EpochFrontierAuthorityInputsV2,
        store: &ConfigV3ActivationStore,
        expected: ActiveCheckpointConfigIdentityV3,
    ) -> Result<Self, CheckpointError> {
        let resolved = Plonky3HistoryAuthorityResolverV2::resolve_installed(store, expected)?;
        Self::new_with_authority(inputs, &resolved)
    }

    fn new_with_authority(
        inputs: EpochFrontierAuthorityInputsV2,
        resolved: &ResolvedPlonky3HistoryAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        let identity = resolved.identity();
        let registry = resolved.registry();
        let security = resolved.security();
        let production_cadence = resolved.cadence_blocks();
        let cadence_blocks = match inputs.cadence_class {
            EpochCadenceClassV2::Production if inputs.cadence_blocks == production_cadence => {
                production_cadence
            }
            EpochCadenceClassV2::BoundedSimulation
                if inputs.cadence_blocks > 0 && inputs.cadence_blocks < production_cadence =>
            {
                inputs.cadence_blocks
            }
            EpochCadenceClassV2::Production | EpochCadenceClassV2::BoundedSimulation => {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::HybridCadenceMismatch,
                ));
            }
        };
        let start_height = inputs
            .epoch_index
            .checked_mul(cadence_blocks)
            .and_then(|height| height.checked_add(1))
            .ok_or(CheckpointError::Overflow)?;
        let end_height = start_height
            .checked_add(cadence_blocks)
            .and_then(|height| height.checked_sub(1))
            .ok_or(CheckpointError::Overflow)?;
        let transition_count = u32::try_from(cadence_blocks).map_err(|_| CheckpointError::Limit)?;
        let chunk_count = transition_count
            .checked_add(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 - 1)
            .and_then(|count| count.checked_div(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2))
            .filter(|count| *count != 0)
            .ok_or(CheckpointError::Overflow)?;
        if [
            inputs.start_root,
            inputs.chain_context_digest,
            inputs.predicate_digest,
            inputs.parameter_digest,
            inputs.verifier_bundle_digest,
            identity.security_budget_digest,
            identity.config_digest,
            identity.registry_digest,
            identity.runtime_profile_manifest_digest,
            identity.authority_bundle_digest,
        ]
        .contains(&[0; 32])
            || inputs.parameter_digest != identity.verifier_parameter_digest
            || inputs.verifier_bundle_digest != identity.verifier_bundle_digest
            || security.digest() != identity.security_budget_digest
            || registry.digest() != identity.registry_digest
        {
            return Err(CheckpointError::Authority);
        }
        let mut authority = Self {
            cadence_class: inputs.cadence_class,
            epoch_index: inputs.epoch_index,
            start_height,
            end_height,
            cadence_blocks,
            transition_count,
            chunk_count,
            runtime_profile_generation: identity.runtime_profile_generation,
            config_generation: identity.config_generation,
            authority_generation: identity.authority_generation,
            activation_height: identity.activation_height,
            rollback_floor: identity.rollback_floor,
            parameter_generation: identity.parameter_generation,
            start_root: inputs.start_root,
            chain_context_digest: inputs.chain_context_digest,
            predicate_digest: inputs.predicate_digest,
            parameter_digest: inputs.parameter_digest,
            verifier_bundle_digest: inputs.verifier_bundle_digest,
            security_budget_digest: security.digest(),
            config_digest: identity.config_digest,
            registry_digest: registry.digest(),
            runtime_profile_manifest_digest: identity.runtime_profile_manifest_digest,
            history_authority_bundle_digest: identity.authority_bundle_digest,
            digest: [0; 32],
        };
        authority.digest = authority_digest(&authority);
        Ok(authority)
    }

    #[must_use]
    pub const fn cadence_class(&self) -> EpochCadenceClassV2 {
        self.cadence_class
    }

    #[must_use]
    pub const fn epoch_index(&self) -> u64 {
        self.epoch_index
    }

    #[must_use]
    pub const fn start_height(&self) -> u64 {
        self.start_height
    }

    #[must_use]
    pub const fn end_height(&self) -> u64 {
        self.end_height
    }

    #[must_use]
    pub const fn transition_count(&self) -> u32 {
        self.transition_count
    }

    #[must_use]
    pub const fn chunk_count(&self) -> u32 {
        self.chunk_count
    }

    #[must_use]
    pub const fn cadence_blocks(&self) -> u64 {
        self.cadence_blocks
    }

    #[must_use]
    pub const fn start_root(&self) -> [u8; 32] {
        self.start_root
    }

    #[must_use]
    pub const fn parameter_generation(&self) -> u32 {
        self.parameter_generation
    }

    #[must_use]
    pub(super) const fn runtime_profile_generation(&self) -> u16 {
        self.runtime_profile_generation
    }

    #[must_use]
    pub(super) const fn config_generation(&self) -> u64 {
        self.config_generation
    }

    #[must_use]
    pub(super) const fn authority_generation(&self) -> u64 {
        self.authority_generation
    }

    #[must_use]
    pub const fn chain_context_digest(&self) -> [u8; 32] {
        self.chain_context_digest
    }

    #[must_use]
    pub const fn predicate_digest(&self) -> [u8; 32] {
        self.predicate_digest
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub const fn parameter_digest(&self) -> [u8; 32] {
        self.parameter_digest
    }

    #[must_use]
    pub const fn security_budget_digest(&self) -> [u8; 32] {
        self.security_budget_digest
    }

    #[must_use]
    pub const fn verifier_bundle_digest(&self) -> [u8; 32] {
        self.verifier_bundle_digest
    }

    #[must_use]
    pub(super) const fn config_digest(&self) -> [u8; 32] {
        self.config_digest
    }

    #[must_use]
    pub(super) const fn registry_digest(&self) -> [u8; 32] {
        self.registry_digest
    }

    #[must_use]
    pub(super) const fn runtime_profile_manifest_digest(&self) -> [u8; 32] {
        self.runtime_profile_manifest_digest
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochRangeRootsV2 {
    pub start_root: [u8; 32],
    pub end_root: [u8; 32],
    pub statement_digest_root: [u8; 32],
    pub checkpoint_artifact_root: [u8; 32],
    pub checkpoint_link_root: [u8; 32],
    pub delta_root: [u8; 32],
    pub witness_root: [u8; 32],
    pub challenge_content_root: [u8; 32],
    pub da_payload_commitment: [u8; 32],
    pub verified_trace_chunk_root: [u8; 32],
}

/// Bounded, non-secret restart status for one exact epoch frontier.
///
/// This contains only counters and the next canonical chunk requiring an
/// actual-verified direct proof. It deliberately exposes neither proof nor
/// witness bytes and is derived from the journal plus fixed chunk-record names
/// than directory enumeration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochFrontierProgressV2 {
    verified_chunk_count: u32,
    total_chunk_count: u32,
    active_range_count: u32,
    next_missing_chunk: Option<u32>,
}

impl EpochFrontierProgressV2 {
    #[must_use]
    pub const fn verified_chunk_count(self) -> u32 {
        self.verified_chunk_count
    }

    #[must_use]
    pub const fn total_chunk_count(self) -> u32 {
        self.total_chunk_count
    }

    #[must_use]
    pub const fn active_range_count(self) -> u32 {
        self.active_range_count
    }

    #[must_use]
    pub const fn next_missing_chunk(self) -> Option<u32> {
        self.next_missing_chunk
    }

    #[must_use]
    pub const fn all_chunks_verified(self) -> bool {
        self.verified_chunk_count == self.total_chunk_count && self.next_missing_chunk.is_none()
    }
}

impl EpochRangeRootsV2 {
    #[allow(clippy::too_many_arguments)]
    pub fn statement_inputs(
        self,
        authority: &EpochFrontierAuthorityV2,
        manifest: &EpochProofWorkManifestV2,
        recursive_epoch_commitment: [u8; 32],
    ) -> Result<EpochRangeInputsV2, CheckpointError> {
        if manifest.frontier_authority_digest() != authority.digest
            || manifest.cadence_class() != authority.cadence_class
            || manifest.epoch_index() != authority.epoch_index
            || manifest.start_height() != authority.start_height
            || manifest.end_height() != authority.end_height
            || manifest.transition_count() != authority.transition_count
            || manifest.start_root() != authority.start_root
            || manifest.epoch_close_anchor_digest() == [0; 32]
            || recursive_epoch_commitment == [0; 32]
            || manifest.nova_chain_root() == Some([0; 32])
        {
            return Err(CheckpointError::Canonical);
        }
        Ok(EpochRangeInputsV2 {
            cadence_class: authority.cadence_class,
            epoch_index: authority.epoch_index,
            start_height: authority.start_height,
            end_height: authority.end_height,
            cadence_blocks: authority.cadence_blocks,
            transition_count: authority.transition_count,
            parameter_generation: authority.parameter_generation,
            chain_context_digest: authority.chain_context_digest,
            predicate_digest: authority.predicate_digest,
            parameter_digest: authority.parameter_digest,
            verifier_bundle_digest: authority.verifier_bundle_digest,
            security_budget_digest: authority.security_budget_digest,
            config_digest: authority.config_digest,
            registry_digest: authority.registry_digest,
            runtime_profile_manifest_digest: authority.runtime_profile_manifest_digest,
            frontier_authority_digest: authority.digest,
            epoch_work_manifest_digest: manifest.digest(),
            epoch_close_anchor_digest: manifest.epoch_close_anchor_digest(),
            start_root: self.start_root,
            end_root: self.end_root,
            statement_digest_root: self.statement_digest_root,
            checkpoint_artifact_root: self.checkpoint_artifact_root,
            checkpoint_link_root: self.checkpoint_link_root,
            delta_root: self.delta_root,
            witness_root: self.witness_root,
            challenge_content_root: self.challenge_content_root,
            da_payload_commitment: self.da_payload_commitment,
            verified_trace_chunk_root: self.verified_trace_chunk_root,
            recursive_epoch_commitment,
            nova_chain_root: manifest.nova_chain_root(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum FrontierNodeKindV2 {
    Chunk = 1,
    Parent = 2,
}

#[derive(Clone)]
struct FrontierNodeV2 {
    kind: FrontierNodeKindV2,
    tree_level: u8,
    start_height: u64,
    end_height: u64,
    transition_count: u32,
    config_generation: u64,
    authority_generation: u64,
    parameter_generation: u32,
    runtime_profile_generation: u16,
    epoch_authority_digest: [u8; 32],
    chain_context_digest: [u8; 32],
    config_digest: [u8; 32],
    registry_digest: [u8; 32],
    runtime_profile_manifest_digest: [u8; 32],
    parameter_digest: [u8; 32],
    security_budget_digest: [u8; 32],
    range_binding_digest: [u8; 32],
    proof_digest: [u8; 32],
    verification_receipt_digest: [u8; 32],
    left_dependency_digest: [u8; 32],
    right_dependency_digest: [u8; 32],
    proof_bytes: Vec<u8>,
    node_digest: [u8; 32],
}

impl core::fmt::Debug for FrontierNodeV2 {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("FrontierNodeV2")
            .field("kind", &self.kind)
            .field("tree_level", &self.tree_level)
            .field("start_height", &self.start_height)
            .field("end_height", &self.end_height)
            .field("transition_count", &self.transition_count)
            .field("proof_digest", &self.proof_digest)
            .field("proof_bytes_len", &self.proof_bytes.len())
            .field("node_digest", &self.node_digest)
            .finish()
    }
}

pub(super) struct EpochMergeJobV2 {
    authority: EpochFrontierAuthorityV2,
    left: FrontierNodeV2,
    right: FrontierNodeV2,
    chunk_pair: Option<EpochChunkPairInputsV2>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EpochTraceChunkProofInputsV2 {
    pub(super) chunk_ordinal: u32,
    pub(super) statement: EpochTraceChunkV2,
    pub(super) bindings: Vec<EpochTransitionBindingV2>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EpochChunkPairInputsV2 {
    pub(super) left: EpochTraceChunkProofInputsV2,
    pub(super) right: EpochTraceChunkProofInputsV2,
    pub(super) total_transition_count: u32,
    pub(super) total_chunk_count: u32,
    pub(super) epoch_index: u64,
    pub(super) cadence_blocks: u64,
    pub(super) parameter_generation: u32,
    pub(super) runtime_profile_generation: u16,
}

impl EpochMergeJobV2 {
    pub(super) fn authority(&self) -> EpochFrontierAuthorityV2 {
        self.authority
    }

    pub(super) fn left_proof_bytes(&self) -> &[u8] {
        &self.left.proof_bytes
    }

    pub(super) fn right_proof_bytes(&self) -> &[u8] {
        &self.right.proof_bytes
    }

    pub(super) fn left_node_digest(&self) -> [u8; 32] {
        self.left.node_digest
    }

    pub(super) fn right_node_digest(&self) -> [u8; 32] {
        self.right.node_digest
    }

    pub(super) fn is_chunk_pair(&self) -> bool {
        self.left.kind == FrontierNodeKindV2::Chunk && self.right.kind == FrontierNodeKindV2::Chunk
    }

    pub(super) fn chunk_pair_inputs(&self) -> Option<EpochChunkPairInputsV2> {
        self.chunk_pair.clone()
    }

    pub(super) fn left_range(&self) -> (u64, u64, u32, u8) {
        (
            self.left.start_height,
            self.left.end_height,
            self.left.transition_count,
            self.left.tree_level,
        )
    }

    pub(super) fn right_range(&self) -> (u64, u64, u32, u8) {
        (
            self.right.start_height,
            self.right.end_height,
            self.right.transition_count,
            self.right.tree_level,
        )
    }

    pub(super) fn left_proof_digest(&self) -> [u8; 32] {
        self.left.proof_digest
    }

    pub(super) fn right_proof_digest(&self) -> [u8; 32] {
        self.right.proof_digest
    }

    pub(super) fn start_height(&self) -> u64 {
        self.left.start_height
    }

    pub(super) fn end_height(&self) -> u64 {
        self.right.end_height
    }

    pub(super) fn transition_count(&self) -> Result<u32, CheckpointError> {
        self.left
            .transition_count
            .checked_add(self.right.transition_count)
            .ok_or(CheckpointError::Overflow)
    }

    pub(super) fn tree_level(&self) -> Result<u8, CheckpointError> {
        self.left
            .tree_level
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)
    }
}

/// Capability constructible only by the private Plonky3 owner after its
/// generated parent has passed the actual pinned verifier.
pub(super) struct VerifiedEpochParentV2 {
    pub(super) left_node_digest: [u8; 32],
    pub(super) right_node_digest: [u8; 32],
    pub(super) start_height: u64,
    pub(super) end_height: u64,
    pub(super) transition_count: u32,
    pub(super) tree_level: u8,
    pub(super) range_binding_digest: [u8; 32],
    pub(super) proof_digest: [u8; 32],
    pub(super) verification_receipt_digest: [u8; 32],
    pub(super) proof_bytes: Vec<u8>,
}

pub(super) struct EpochFinalizationNodeV2 {
    pub(super) start_height: u64,
    pub(super) end_height: u64,
    pub(super) transition_count: u32,
    pub(super) tree_level: u8,
    pub(super) proof_digest: [u8; 32],
    pub(super) proof_bytes: Vec<u8>,
}

pub struct EpochProofFrontierV2 {
    authority: EpochFrontierAuthorityV2,
    nodes: SecureDir,
    chunks: SecureDir,
    journal: SecureDir,
    process_lock: File,
}

impl EpochProofFrontierV2 {
    pub(super) const fn authority(&self) -> EpochFrontierAuthorityV2 {
        self.authority
    }

    pub fn open(
        root: impl AsRef<std::path::Path>,
        authority: EpochFrontierAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        let root = SecureDir::ensure_private(root).map_err(|_| CheckpointError::Storage)?;
        let nodes = root
            .ensure_dir("nodes")
            .map_err(|_| CheckpointError::Storage)?;
        let chunks = root
            .ensure_dir("chunks")
            .map_err(|_| CheckpointError::Storage)?;
        let journal = root
            .ensure_dir("journal")
            .map_err(|_| CheckpointError::Storage)?;
        let process_lock = root
            .open_lock(".frontier.lock")
            .map_err(|_| CheckpointError::Storage)?;
        let frontier = Self {
            authority,
            nodes,
            chunks,
            journal,
            process_lock,
        };
        frontier
            .process_lock
            .lock_exclusive()
            .map_err(|_| CheckpointError::Storage)?;
        let result = (|| {
            scavenge_temporary_files(&root, FRONTIER_MAX_ROOT_ENTRIES_V2)?;
            scavenge_temporary_files(&frontier.nodes, FRONTIER_MAX_NODE_FILES_V2)?;
            scavenge_temporary_files(&frontier.chunks, FRONTIER_MAX_CHUNK_FILES_V2)?;
            scavenge_temporary_files(&frontier.journal, FRONTIER_MAX_JOURNAL_ENTRIES_V2)?;
            frontier.install_or_validate_authority(&root)?;
            frontier.reconcile_incomplete_parent_retirements()?;
            frontier.clean_retired_node_bodies()?;
            frontier.clean_orphan_records()
        })();
        FileExt::unlock(&frontier.process_lock).map_err(|_| CheckpointError::Storage)?;
        result?;
        Ok(frontier)
    }

    pub fn admit_verified_chunk(
        &self,
        proof: &Plonky3EpochChunkProofV2,
    ) -> Result<(), CheckpointError> {
        self.process_lock
            .lock_exclusive()
            .map_err(|_| CheckpointError::Storage)?;
        let result = self.admit_verified_chunk_locked(proof);
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    fn admit_verified_chunk_locked(
        &self,
        proof: &Plonky3EpochChunkProofV2,
    ) -> Result<(), CheckpointError> {
        let admission = proof.verified_frontier_admission()?;
        self.validate_chunk_admission(&admission)?;
        let ordinal = admission.transition_statement.inputs().chunk_ordinal;
        let chunk_record = FrontierChunkRecordV2::new(&admission)?;
        let chunk_name = chunk_file_name(ordinal);
        if self
            .chunks
            .read_file_bounded(&chunk_name, FRONTIER_CHUNK_MAX_BYTES_V2 as u64)
            .is_ok()
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StepRepeated,
            ));
        }
        let node = FrontierNodeV2::chunk(&self.authority, &chunk_record, &admission)?;
        let state = self.journal_state()?;
        if state.overlaps(node.start_height, node.end_height) {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StepRepeated,
            ));
        }
        write_once(&self.chunks, &chunk_name, &chunk_record.encode())?;
        self.write_node(&node)?;
        self.append_journal(FrontierJournalRecordV2::node_installed(
            state.next_sequence,
            &node,
        )?)?;
        Ok(())
    }

    pub(super) fn next_merge_job(&self) -> Result<Option<EpochMergeJobV2>, CheckpointError> {
        self.process_lock
            .lock_exclusive()
            .map_err(|_| CheckpointError::Storage)?;
        let result = self.next_merge_job_locked();
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    fn next_merge_job_locked(&self) -> Result<Option<EpochMergeJobV2>, CheckpointError> {
        let state = self.journal_state()?;
        let mut nodes = self.load_active_nodes(&state)?;
        nodes.sort_by_key(|node| node.start_height);
        for pair in nodes.windows(2) {
            let left = &pair[0];
            let right = &pair[1];
            if left.tree_level == right.tree_level
                && left.transition_count == right.transition_count
                && left.end_height.checked_add(1) == Some(right.start_height)
            {
                let key = (left.node_digest, right.node_digest);
                if !state.scheduled.contains(&key) {
                    self.append_journal(FrontierJournalRecordV2::merge_scheduled(
                        state.next_sequence,
                        left,
                        right,
                    )?)?;
                }
                return Ok(Some(EpochMergeJobV2 {
                    authority: self.authority,
                    left: left.clone(),
                    right: right.clone(),
                    chunk_pair: if left.kind == FrontierNodeKindV2::Chunk
                        && right.kind == FrontierNodeKindV2::Chunk
                    {
                        Some(self.load_chunk_pair_inputs(left, right)?)
                    } else {
                        None
                    },
                }));
            }
        }
        Ok(None)
    }

    fn load_chunk_pair_inputs(
        &self,
        left: &FrontierNodeV2,
        right: &FrontierNodeV2,
    ) -> Result<EpochChunkPairInputsV2, CheckpointError> {
        let load =
            |node: &FrontierNodeV2| -> Result<EpochTraceChunkProofInputsV2, CheckpointError> {
                if node.kind != FrontierNodeKindV2::Chunk
                    || node.transition_count == 0
                    || node.transition_count > EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2
                    || node.tree_level != 0
                {
                    return Err(CheckpointError::Canonical);
                }
                let first_transition = u32::try_from(
                    node.start_height
                        .checked_sub(self.authority.start_height)
                        .ok_or(CheckpointError::Overflow)?,
                )
                .map_err(|_| CheckpointError::Limit)?;
                let ordinal = first_transition
                    .checked_div(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2)
                    .ok_or(CheckpointError::Overflow)?;
                let bytes = self
                    .chunks
                    .read_file_bounded(
                        &chunk_file_name(ordinal),
                        FRONTIER_CHUNK_MAX_BYTES_V2 as u64,
                    )
                    .map_err(|_| CheckpointError::Storage)?;
                let record = FrontierChunkRecordV2::decode(&self.authority, &bytes)?;
                if record.chunk_ordinal != ordinal
                    || record.start_height()? != node.start_height
                    || record.end_height()? != node.end_height
                    || record.transition_count()? != node.transition_count
                    || record.proof_digest != node.proof_digest
                    || record.verified_trace_chunk_binding_digest()? != node.range_binding_digest
                {
                    return Err(CheckpointError::Canonical);
                }
                Ok(EpochTraceChunkProofInputsV2 {
                    chunk_ordinal: ordinal,
                    statement: record.statement,
                    bindings: record.bindings,
                })
            };
        let left = load(left)?;
        let right = load(right)?;
        if left.statement.inputs().output_accumulator != right.statement.inputs().input_accumulator
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
            ));
        }
        Ok(EpochChunkPairInputsV2 {
            left,
            right,
            total_transition_count: self.authority.transition_count,
            total_chunk_count: self.authority.chunk_count,
            epoch_index: self.authority.epoch_index,
            cadence_blocks: self.authority.cadence_blocks,
            parameter_generation: self.authority.parameter_generation,
            runtime_profile_generation: self.authority.runtime_profile_generation,
        })
    }

    pub(super) fn install_verified_parent(
        &self,
        verified: VerifiedEpochParentV2,
    ) -> Result<(), CheckpointError> {
        self.process_lock
            .lock_exclusive()
            .map_err(|_| CheckpointError::Storage)?;
        let result = self.install_verified_parent_locked(verified);
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    fn install_verified_parent_locked(
        &self,
        verified: VerifiedEpochParentV2,
    ) -> Result<(), CheckpointError> {
        let mut state = self.journal_state()?;
        let left = self.load_active_node(&state, verified.left_node_digest)?;
        let right = self.load_active_node(&state, verified.right_node_digest)?;
        if !state
            .scheduled
            .contains(&(left.node_digest, right.node_digest))
            || left.tree_level != right.tree_level
            || left.transition_count != right.transition_count
            || left.end_height.checked_add(1) != Some(right.start_height)
            || verified.start_height != left.start_height
            || verified.end_height != right.end_height
            || verified.transition_count
                != left
                    .transition_count
                    .checked_add(right.transition_count)
                    .ok_or(CheckpointError::Overflow)?
            || verified.tree_level
                != left
                    .tree_level
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?
            || verified.proof_bytes.is_empty()
            || verified.proof_bytes.len() > PLONKY3_PUBLISH_BYTES_V2
            || [
                verified.range_binding_digest,
                verified.proof_digest,
                verified.verification_receipt_digest,
            ]
            .contains(&[0; 32])
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        let node = FrontierNodeV2::parent(&self.authority, verified)?;
        self.write_node(&node)?;
        self.append_journal(FrontierJournalRecordV2::parent_installed(
            state.next_sequence,
            &node,
        )?)?;
        state.next_sequence = state
            .next_sequence
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        self.append_journal(FrontierJournalRecordV2::children_retired(
            state.next_sequence,
            &node,
        )?)?;
        self.remove_node_body(left.node_digest)?;
        self.remove_node_body(right.node_digest)?;
        Ok(())
    }

    /// Compact roots become available only after every exact chunk was admitted
    /// and all equal-height merges were consumed.
    pub fn range_roots(&self) -> Result<EpochRangeRootsV2, CheckpointError> {
        self.process_lock
            .lock_shared()
            .map_err(|_| CheckpointError::Storage)?;
        let result = self.range_roots_locked();
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    fn range_roots_locked(&self) -> Result<EpochRangeRootsV2, CheckpointError> {
        let state = self.journal_state()?;
        let mut active = self.load_active_nodes(&state)?;
        active.sort_by_key(|node| node.start_height);
        validate_complete_segment_cover(&self.authority, &active)?;
        if active.windows(2).any(|pair| {
            pair[0].tree_level == pair[1].tree_level
                && pair[0].transition_count == pair[1].transition_count
        }) {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
            ));
        }
        let max_segments = usize::try_from(self.authority.chunk_count.ilog2() + 1)
            .map_err(|_| CheckpointError::Limit)?;
        if active.len() > max_segments {
            return Err(CheckpointError::Limit);
        }
        let mut chunks: Vec<FrontierChunkRecordV2> = Vec::with_capacity(
            usize::try_from(self.authority.chunk_count).map_err(|_| CheckpointError::Limit)?,
        );
        for ordinal in 0..self.authority.chunk_count {
            let bytes = self
                .chunks
                .read_file_bounded(
                    &chunk_file_name(ordinal),
                    FRONTIER_CHUNK_MAX_BYTES_V2 as u64,
                )
                .map_err(|_| {
                    CheckpointError::RecursiveRejected(
                        RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
                    )
                })?;
            let chunk = FrontierChunkRecordV2::decode(&self.authority, &bytes)?;
            let expected_first = ordinal
                .checked_mul(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2)
                .ok_or(CheckpointError::Overflow)?;
            let expected_last = expected_first
                .checked_add(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 - 1)
                .map(|last| last.min(self.authority.transition_count - 1))
                .ok_or(CheckpointError::Overflow)?;
            if chunk.chunk_ordinal != ordinal
                || chunk.first_transition != expected_first
                || chunk.last_transition != expected_last
            {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::StepReordered,
                ));
            }
            if let Some(previous) = chunks.last() {
                let previous = previous
                    .bindings
                    .last()
                    .ok_or(CheckpointError::Canonical)?
                    .inputs();
                let next = chunk
                    .bindings
                    .first()
                    .ok_or(CheckpointError::Canonical)?
                    .inputs();
                if previous.post_settlement_root != next.pre_settlement_root
                    || next.predecessor != Some(previous.checkpoint_id)
                    || chunks
                        .last()
                        .ok_or(CheckpointError::Invariant)?
                        .statement
                        .inputs()
                        .output_accumulator
                        != chunk.statement.inputs().input_accumulator
                {
                    return Err(CheckpointError::RecursiveRejected(
                        RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
                    ));
                }
            }
            chunks.push(chunk);
        }
        let transitions = chunks
            .iter()
            .flat_map(|chunk| chunk.bindings.iter())
            .collect::<Vec<_>>();
        if transitions.len()
            != usize::try_from(self.authority.transition_count)
                .map_err(|_| CheckpointError::Limit)?
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
            ));
        }
        let roots = |domain: &str,
                     select: fn(&EpochTransitionBindingV2) -> [u8; 32]|
         -> Result<[u8; 32], CheckpointError> {
            let values = transitions
                .iter()
                .map(|binding| select(binding))
                .collect::<Vec<_>>();
            epoch_ordered_digest_root_v2(domain, &values)
        };
        let verified_trace_chunks = chunks
            .iter()
            .map(|chunk| -> Result<(u64, u64, [u8; 32]), CheckpointError> {
                Ok((
                    u64::from(chunk.first_transition),
                    u64::from(chunk.transition_count()?),
                    chunk.verified_trace_chunk_binding_digest()?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(EpochRangeRootsV2 {
            start_root: transitions
                .first()
                .ok_or(CheckpointError::Canonical)?
                .inputs()
                .pre_settlement_root,
            end_root: transitions
                .last()
                .ok_or(CheckpointError::Canonical)?
                .inputs()
                .post_settlement_root,
            statement_digest_root: roots(EPOCH_STATEMENT_ROOT_DOMAIN_V2, |binding| {
                binding.inputs().checkpoint_statement_digest
            })?,
            checkpoint_artifact_root: roots(EPOCH_ARTIFACT_ROOT_DOMAIN_V2, |binding| {
                binding.inputs().checkpoint_artifact_digest
            })?,
            checkpoint_link_root: roots(EPOCH_LINK_ROOT_DOMAIN_V2, |binding| {
                binding.inputs().checkpoint_link_digest
            })?,
            delta_root: roots(EPOCH_DELTA_ROOT_DOMAIN_V2, |binding| {
                binding.inputs().delta_root
            })?,
            witness_root: roots(EPOCH_WITNESS_ROOT_DOMAIN_V2, |binding| {
                binding.inputs().witness_root
            })?,
            challenge_content_root: roots(EPOCH_CHALLENGE_ROOT_DOMAIN_V2, |binding| {
                binding.inputs().challenge_content_digest
            })?,
            da_payload_commitment: roots(EPOCH_DA_ROOT_DOMAIN_V2, |binding| {
                binding.inputs().da_payload_commitment
            })?,
            verified_trace_chunk_root: epoch_ordered_digest_span_root_v2(
                EPOCH_VERIFIED_TRACE_CHUNK_ROOT_DOMAIN_V2,
                u64::from(self.authority.transition_count),
                &verified_trace_chunks,
            )?,
        })
    }

    pub(super) fn finalization_nodes(
        &self,
    ) -> Result<Vec<EpochFinalizationNodeV2>, CheckpointError> {
        self.process_lock
            .lock_shared()
            .map_err(|_| CheckpointError::Storage)?;
        let result = self.finalization_nodes_locked();
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    fn finalization_nodes_locked(&self) -> Result<Vec<EpochFinalizationNodeV2>, CheckpointError> {
        let state = self.journal_state()?;
        let mut active = self.load_active_nodes(&state)?;
        active.sort_by_key(|node| node.start_height);
        validate_complete_segment_cover(&self.authority, &active)?;
        if active
            .iter()
            .any(|node| node.kind != FrontierNodeKindV2::Parent)
            || active.windows(2).any(|pair| {
                pair[0].tree_level == pair[1].tree_level
                    && pair[0].transition_count == pair[1].transition_count
            })
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
            ));
        }
        let max_segments = usize::try_from(self.authority.chunk_count.ilog2() + 1)
            .map_err(|_| CheckpointError::Limit)?;
        if active.len() > max_segments {
            return Err(CheckpointError::Limit);
        }
        Ok(active
            .into_iter()
            .map(|node| EpochFinalizationNodeV2 {
                start_height: node.start_height,
                end_height: node.end_height,
                transition_count: node.transition_count,
                tree_level: node.tree_level,
                proof_digest: node.proof_digest,
                proof_bytes: node.proof_bytes,
            })
            .collect())
    }

    pub fn validate_closed_manifest(
        &self,
        manifest: &EpochProofWorkManifestV2,
    ) -> Result<(), CheckpointError> {
        self.process_lock
            .lock_shared()
            .map_err(|_| CheckpointError::Storage)?;
        let result = self.validate_closed_manifest_locked(manifest).map(|_| ());
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    fn validate_closed_manifest_locked(
        &self,
        manifest: &EpochProofWorkManifestV2,
    ) -> Result<EpochRangeRootsV2, CheckpointError> {
        if manifest.cadence_class() != self.authority.cadence_class
            || manifest.epoch_index() != self.authority.epoch_index
            || manifest.start_height() != self.authority.start_height
            || manifest.end_height() != self.authority.end_height
            || manifest.transition_count() != self.authority.transition_count
            || manifest.frontier_authority_digest() != self.authority.digest
            || manifest.start_root() != self.authority.start_root
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
            ));
        }
        for ordinal in 0..self.authority.chunk_count {
            let bytes = self
                .chunks
                .read_file_bounded(
                    &chunk_file_name(ordinal),
                    FRONTIER_CHUNK_MAX_BYTES_V2 as u64,
                )
                .map_err(|_| {
                    CheckpointError::RecursiveRejected(
                        RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
                    )
                })?;
            let chunk = FrontierChunkRecordV2::decode(&self.authority, &bytes)?;
            if chunk.chunk_ordinal != ordinal {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::StepReordered,
                ));
            }
            manifest.validate_closed_chunk(&chunk.statement)?;
        }
        let roots = self.range_roots_locked()?;
        if roots.start_root != manifest.start_root() || roots.end_root != manifest.end_root() {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
            ));
        }
        Ok(roots)
    }

    pub fn verify_sealed_statement(
        &self,
        statement: &EpochRangeStatementV2,
        manifest: &EpochProofWorkManifestV2,
    ) -> Result<(), CheckpointError> {
        self.process_lock
            .lock_shared()
            .map_err(|_| CheckpointError::Storage)?;
        let result = (|| {
            let roots = self.validate_closed_manifest_locked(manifest)?;
            let inputs = statement.inputs();
            if statement.cadence_class() != self.authority.cadence_class
                || statement.epoch_index() != self.authority.epoch_index
                || statement.start_height() != self.authority.start_height
                || statement.end_height() != self.authority.end_height
                || statement.transition_count() != self.authority.transition_count
                || inputs.chain_context_digest != self.authority.chain_context_digest
                || inputs.predicate_digest != self.authority.predicate_digest
                || inputs.parameter_digest != self.authority.parameter_digest
                || inputs.verifier_bundle_digest != self.authority.verifier_bundle_digest
                || inputs.security_budget_digest != self.authority.security_budget_digest
                || inputs.config_digest != self.authority.config_digest
                || inputs.registry_digest != self.authority.registry_digest
                || inputs.runtime_profile_manifest_digest
                    != self.authority.runtime_profile_manifest_digest
                || inputs.frontier_authority_digest != self.authority.digest
                || inputs.epoch_work_manifest_digest != manifest.digest()
                || inputs.epoch_close_anchor_digest != manifest.epoch_close_anchor_digest()
                || inputs.nova_chain_root != manifest.nova_chain_root()
                || inputs.start_root != roots.start_root
                || inputs.end_root != roots.end_root
                || inputs.statement_digest_root != roots.statement_digest_root
                || inputs.checkpoint_artifact_root != roots.checkpoint_artifact_root
                || inputs.checkpoint_link_root != roots.checkpoint_link_root
                || inputs.delta_root != roots.delta_root
                || inputs.witness_root != roots.witness_root
                || inputs.challenge_content_root != roots.challenge_content_root
                || inputs.da_payload_commitment != roots.da_payload_commitment
                || inputs.verified_trace_chunk_root != roots.verified_trace_chunk_root
            {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
                ));
            }
            Ok(())
        })();
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    pub fn active_range_count(&self) -> Result<usize, CheckpointError> {
        self.process_lock
            .lock_shared()
            .map_err(|_| CheckpointError::Storage)?;
        let result = self.journal_state().map(|state| state.active_nodes.len());
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    /// Return deterministic restart progress without reading proof bodies.
    ///
    /// Chunk authority is the exact ordinal sequence `0..chunk_count`; missing
    /// files are progress, while malformed, substituted, or misplaced records
    /// fail closed.
    pub fn progress(&self) -> Result<EpochFrontierProgressV2, CheckpointError> {
        self.process_lock
            .lock_shared()
            .map_err(|_| CheckpointError::Storage)?;
        let result = self.progress_locked();
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    /// Return the exact validated chunk ordinals still absent from the durable
    /// frontier. This is the single restart/scheduling view used by bounded
    /// local workers; malformed persisted records fail closed.
    pub fn missing_chunk_ordinals(&self) -> Result<Vec<u32>, CheckpointError> {
        self.process_lock
            .lock_shared()
            .map_err(|_| CheckpointError::Storage)?;
        let result = (|| {
            let state = self.journal_state()?;
            self.validated_chunk_presence_locked(&state)?
                .into_iter()
                .enumerate()
                .filter_map(|(ordinal, present)| {
                    (!present).then(|| u32::try_from(ordinal).map_err(|_| CheckpointError::Limit))
                })
                .collect()
        })();
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    fn progress_locked(&self) -> Result<EpochFrontierProgressV2, CheckpointError> {
        let state = self.journal_state()?;
        let active_range_count =
            u32::try_from(state.active_nodes.len()).map_err(|_| CheckpointError::Limit)?;
        let presence = self.validated_chunk_presence_locked(&state)?;
        let verified_chunk_count =
            u32::try_from(presence.iter().filter(|present| **present).count())
                .map_err(|_| CheckpointError::Limit)?;
        let next_missing_chunk = presence
            .iter()
            .position(|present| !present)
            .map(u32::try_from)
            .transpose()
            .map_err(|_| CheckpointError::Limit)?;
        Ok(EpochFrontierProgressV2 {
            verified_chunk_count,
            total_chunk_count: self.authority.chunk_count,
            active_range_count,
            next_missing_chunk,
        })
    }

    fn validated_chunk_presence_locked(
        &self,
        state: &FrontierJournalStateV2,
    ) -> Result<Vec<bool>, CheckpointError> {
        let mut presence = Vec::with_capacity(
            usize::try_from(self.authority.chunk_count).map_err(|_| CheckpointError::Limit)?,
        );
        for ordinal in 0..self.authority.chunk_count {
            let first_transition = ordinal
                .checked_mul(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2)
                .ok_or(CheckpointError::Overflow)?;
            let start_height = self
                .authority
                .start_height
                .checked_add(u64::from(first_transition))
                .ok_or(CheckpointError::Overflow)?;
            let name = chunk_file_name(ordinal);
            match self
                .chunks
                .read_file_bounded(&name, FRONTIER_CHUNK_MAX_BYTES_V2 as u64)
            {
                Ok(bytes) => {
                    let chunk = FrontierChunkRecordV2::decode(&self.authority, &bytes)?;
                    if chunk.chunk_ordinal != ordinal
                        || chunk.start_height()? != start_height
                        || state.verified_chunks.get(&start_height)
                            != Some(&(
                                chunk.verified_trace_chunk_binding_digest()?,
                                chunk.verified_admission_identity_digest()?,
                            ))
                    {
                        return Err(CheckpointError::Canonical);
                    }
                    presence.push(true);
                }
                Err(IoError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                    presence.push(false);
                }
                Err(_) => return Err(CheckpointError::Storage),
            }
        }
        Ok(presence)
    }

    fn validate_chunk_admission(
        &self,
        admission: &VerifiedEpochTraceChunkAdmissionV2,
    ) -> Result<(), CheckpointError> {
        let inputs = admission.transition_statement.inputs();
        let expected_first = inputs
            .chunk_ordinal
            .checked_mul(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2)
            .ok_or(CheckpointError::Overflow)?;
        let expected_last = expected_first
            .checked_add(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 - 1)
            .map(|last| last.min(self.authority.transition_count - 1))
            .ok_or(CheckpointError::Overflow)?;
        let expected_count = expected_last
            .checked_sub(expected_first)
            .and_then(|count| count.checked_add(1))
            .ok_or(CheckpointError::Overflow)?;
        if inputs.table != EpochAirTableV2::Transition
            || inputs.replica != 0
            || inputs.chunk_count != self.authority.chunk_count
            || inputs.transition_count != self.authority.transition_count
            || inputs.chunk_ordinal >= self.authority.chunk_count
            || inputs.first_transition != expected_first
            || inputs.last_transition != expected_last
            || admission.bindings.len()
                != usize::try_from(expected_count).map_err(|_| CheckpointError::Limit)?
            || admission
                .bindings
                .first()
                .map(EpochTransitionBindingV2::height)
                != self
                    .authority
                    .start_height
                    .checked_add(u64::from(expected_first))
            || admission
                .bindings
                .last()
                .map(EpochTransitionBindingV2::height)
                != self
                    .authority
                    .start_height
                    .checked_add(u64::from(expected_last))
            || inputs.frontier_authority_digest != self.authority.digest
            || inputs.parameter_digest != self.authority.parameter_digest
            || inputs.verifier_bundle_digest != self.authority.verifier_bundle_digest
            || inputs.security_budget_digest != self.authority.security_budget_digest
            || admission.proof_digest == [0; 32]
            || admission.verification_receipt_digest == [0; 32]
            || admission.proof_bytes.is_empty()
            || admission.proof_bytes.len() > RECURSIVE_INGRESS_BYTES_V2
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
            ));
        }
        Ok(())
    }

    fn install_or_validate_authority(&self, root: &SecureDir) -> Result<(), CheckpointError> {
        let bytes = encode_authority(&self.authority);
        if let Ok(existing) =
            root.read_file_bounded("frontier.authority", FRONTIER_AUTHORITY_BYTES_V2 as u64)
        {
            let decoded = decode_authority(&existing)?;
            if existing != bytes || decoded != self.authority {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::MixedEra,
                ));
            }
            return Ok(());
        }
        write_once(root, "frontier.authority", &bytes)
    }

    fn write_node(&self, node: &FrontierNodeV2) -> Result<(), CheckpointError> {
        write_once(
            &self.nodes,
            &node_file_name(node.node_digest),
            &encode_node(node)?,
        )
    }

    fn load_node(&self, digest: [u8; 32]) -> Result<FrontierNodeV2, CheckpointError> {
        let bytes = self
            .nodes
            .read_file_bounded(&node_file_name(digest), FRONTIER_NODE_MAX_BYTES_V2 as u64)
            .map_err(|_| CheckpointError::Storage)?;
        let node = decode_node(&bytes)?;
        if node.node_digest != digest || node.epoch_authority_digest != self.authority.digest {
            return Err(CheckpointError::Canonical);
        }
        Ok(node)
    }

    fn load_active_node(
        &self,
        state: &FrontierJournalStateV2,
        digest: [u8; 32],
    ) -> Result<FrontierNodeV2, CheckpointError> {
        let expected = state
            .active_nodes
            .get(&digest)
            .ok_or(CheckpointError::Canonical)?;
        let node = self.load_node(digest)?;
        if (node.start_height, node.end_height, node.transition_count) != *expected {
            return Err(CheckpointError::Canonical);
        }
        Ok(node)
    }

    fn load_active_nodes(
        &self,
        state: &FrontierJournalStateV2,
    ) -> Result<Vec<FrontierNodeV2>, CheckpointError> {
        state
            .active_nodes
            .keys()
            .copied()
            .map(|digest| self.load_node(digest))
            .collect()
    }

    fn remove_node_body(&self, digest: [u8; 32]) -> Result<(), CheckpointError> {
        let name = node_file_name(digest);
        match self.nodes.remove_file(&name) {
            Ok(()) => self.nodes.sync().map_err(|_| CheckpointError::Storage),
            Err(IoError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(_) => Err(CheckpointError::Storage),
        }
    }

    fn append_journal(&self, record: FrontierJournalRecordV2) -> Result<(), CheckpointError> {
        let state = self.journal_state()?;
        if state.next_sequence != record.sequence
            || usize::try_from(record.sequence).map_err(|_| CheckpointError::Limit)?
                >= FRONTIER_MAX_JOURNAL_ENTRIES_V2
        {
            return Err(CheckpointError::Limit);
        }
        write_once(
            &self.journal,
            &journal_file_name(record.sequence, record.digest),
            &record.encode(),
        )
    }

    fn journal_state(&self) -> Result<FrontierJournalStateV2, CheckpointError> {
        let names = self
            .journal
            .read_dir_bounded(FRONTIER_MAX_JOURNAL_ENTRIES_V2)
            .map_err(|_| CheckpointError::Storage)?;
        let mut ordered = BTreeMap::new();
        for name in names {
            let name = name.into_string().map_err(|_| CheckpointError::Canonical)?;
            let bytes = self
                .journal
                .read_file_bounded(&name, FRONTIER_JOURNAL_BYTES_V2 as u64)
                .map_err(|_| CheckpointError::Storage)?;
            let record = FrontierJournalRecordV2::decode(&bytes)?;
            if name != journal_file_name(record.sequence, record.digest)
                || ordered.insert(record.sequence, record).is_some()
            {
                return Err(CheckpointError::Canonical);
            }
        }
        let mut state = FrontierJournalStateV2::default();
        for (expected, record) in ordered.into_values().enumerate() {
            if record.sequence != expected as u64 {
                return Err(CheckpointError::Canonical);
            }
            state.apply(record)?;
        }
        Ok(state)
    }

    fn reconcile_incomplete_parent_retirements(&self) -> Result<(), CheckpointError> {
        loop {
            let state = self.journal_state()?;
            let Some(parent_digest) = state.unretired_parents.iter().next().copied() else {
                return Ok(());
            };
            let parent = self.load_node(parent_digest)?;
            self.append_journal(FrontierJournalRecordV2::children_retired(
                state.next_sequence,
                &parent,
            )?)?;
        }
    }

    fn clean_retired_node_bodies(&self) -> Result<(), CheckpointError> {
        let state = self.journal_state()?;
        for digest in state.retired_nodes {
            self.remove_node_body(digest)?;
        }
        Ok(())
    }

    fn clean_orphan_records(&self) -> Result<(), CheckpointError> {
        let state = self.journal_state()?;
        let mut seen_chunks = BTreeSet::new();
        let mut chunks_changed = false;
        for name in self
            .chunks
            .read_dir_bounded(FRONTIER_MAX_CHUNK_FILES_V2)
            .map_err(|_| CheckpointError::Storage)?
        {
            let name = name.into_string().map_err(|_| CheckpointError::Canonical)?;
            let bytes = self
                .chunks
                .read_file_bounded(&name, FRONTIER_CHUNK_MAX_BYTES_V2 as u64)
                .map_err(|_| CheckpointError::Storage)?;
            let chunk = FrontierChunkRecordV2::decode(&self.authority, &bytes)?;
            if name != chunk_file_name(chunk.chunk_ordinal)
                || chunk.chunk_ordinal >= self.authority.chunk_count
            {
                return Err(CheckpointError::Canonical);
            }
            let start_height = chunk.start_height()?;
            match state.verified_chunks.get(&start_height) {
                Some((binding, identity))
                    if *binding == chunk.verified_trace_chunk_binding_digest()?
                        && *identity == chunk.verified_admission_identity_digest()? =>
                {
                    if !seen_chunks.insert(start_height) {
                        return Err(CheckpointError::Canonical);
                    }
                }
                Some(_) => return Err(CheckpointError::Canonical),
                None => {
                    self.chunks
                        .remove_file(&name)
                        .map_err(|_| CheckpointError::Storage)?;
                    chunks_changed = true;
                }
            }
        }
        if seen_chunks.len() != state.verified_chunks.len() {
            return Err(CheckpointError::Canonical);
        }
        if chunks_changed {
            self.chunks.sync().map_err(|_| CheckpointError::Storage)?;
        }

        let mut seen_nodes = BTreeSet::new();
        let mut nodes_changed = false;
        for name in self
            .nodes
            .read_dir_bounded(FRONTIER_MAX_NODE_FILES_V2)
            .map_err(|_| CheckpointError::Storage)?
        {
            let name = name.into_string().map_err(|_| CheckpointError::Canonical)?;
            let bytes = self
                .nodes
                .read_file_bounded(&name, FRONTIER_NODE_MAX_BYTES_V2 as u64)
                .map_err(|_| CheckpointError::Storage)?;
            let node = decode_node(&bytes)?;
            if name != node_file_name(node.node_digest) {
                return Err(CheckpointError::Canonical);
            }
            if state.active_nodes.contains_key(&node.node_digest) {
                node.validate(&self.authority)?;
                if !seen_nodes.insert(node.node_digest) {
                    return Err(CheckpointError::Canonical);
                }
            } else {
                self.nodes
                    .remove_file(&name)
                    .map_err(|_| CheckpointError::Storage)?;
                nodes_changed = true;
            }
        }
        if seen_nodes.len() != state.active_nodes.len() {
            return Err(CheckpointError::Canonical);
        }
        if nodes_changed {
            self.nodes.sync().map_err(|_| CheckpointError::Storage)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct FrontierChunkRecordV2 {
    chunk_ordinal: u32,
    first_transition: u32,
    last_transition: u32,
    statement: EpochTraceChunkV2,
    bindings: Vec<EpochTransitionBindingV2>,
    proof_digest: [u8; 32],
    verification_receipt_digest: [u8; 32],
    record_digest: [u8; 32],
}

impl FrontierChunkRecordV2 {
    fn new(admission: &VerifiedEpochTraceChunkAdmissionV2) -> Result<Self, CheckpointError> {
        let inputs = admission.transition_statement.inputs();
        if admission.bindings.is_empty()
            || admission.bindings.len()
                > usize::try_from(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2)
                    .map_err(|_| CheckpointError::Limit)?
            || admission.proof_digest == [0; 32]
            || admission.verification_receipt_digest == [0; 32]
        {
            return Err(CheckpointError::Canonical);
        }
        let mut record = Self {
            chunk_ordinal: inputs.chunk_ordinal,
            first_transition: inputs.first_transition,
            last_transition: inputs.last_transition,
            statement: admission.transition_statement.clone(),
            bindings: admission.bindings.clone(),
            proof_digest: admission.proof_digest,
            verification_receipt_digest: admission.verification_receipt_digest,
            record_digest: [0; 32],
        };
        record.validate()?;
        record.record_digest = sha256_256(
            "z00z.storage.checkpoint.epoch-frontier-trace-chunk.v2",
            "record",
            &[&record.prefix()],
        );
        Ok(record)
    }

    fn validate(&self) -> Result<(), CheckpointError> {
        let inputs = self.statement.inputs();
        let expected_count = self
            .last_transition
            .checked_sub(self.first_transition)
            .and_then(|span| span.checked_add(1))
            .ok_or(CheckpointError::Overflow)?;
        if self.chunk_ordinal != inputs.chunk_ordinal
            || self.first_transition != inputs.first_transition
            || self.last_transition != inputs.last_transition
            || self.bindings.len()
                != usize::try_from(expected_count).map_err(|_| CheckpointError::Limit)?
            || self.bindings.first().map(EpochTransitionBindingV2::ordinal)
                != Some(self.first_transition)
            || self.bindings.last().map(EpochTransitionBindingV2::ordinal)
                != Some(self.last_transition)
            || self
                .bindings
                .windows(2)
                .any(|pair| pair[0].ordinal().checked_add(1) != Some(pair[1].ordinal()))
            || [self.proof_digest, self.verification_receipt_digest].contains(&[0; 32])
        {
            return Err(CheckpointError::Canonical);
        }
        Ok(())
    }

    fn start_height(&self) -> Result<u64, CheckpointError> {
        self.bindings
            .first()
            .map(EpochTransitionBindingV2::height)
            .ok_or(CheckpointError::Canonical)
    }

    fn end_height(&self) -> Result<u64, CheckpointError> {
        self.bindings
            .last()
            .map(EpochTransitionBindingV2::height)
            .ok_or(CheckpointError::Canonical)
    }

    fn transition_count(&self) -> Result<u32, CheckpointError> {
        u32::try_from(self.bindings.len()).map_err(|_| CheckpointError::Limit)
    }

    fn prefix(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(
            FRONTIER_CHUNK_FIXED_BYTES_V2
                .checked_add(
                    self.bindings
                        .len()
                        .checked_mul(EPOCH_TRANSITION_BINDING_BYTES_V2)
                        .expect("bounded chunk binding bytes"),
                )
                .expect("bounded chunk record bytes")
                - 32,
        );
        bytes.extend_from_slice(&FRONTIER_CHUNK_MAGIC_V2);
        bytes.extend_from_slice(&FRONTIER_WIRE_VERSION_V2.to_le_bytes());
        bytes.push(FRONTIER_TREE_GENERATION_V2);
        bytes.extend_from_slice(&self.chunk_ordinal.to_le_bytes());
        bytes.extend_from_slice(&self.first_transition.to_le_bytes());
        bytes.extend_from_slice(&self.last_transition.to_le_bytes());
        bytes.extend_from_slice(
            &u32::try_from(self.bindings.len())
                .expect("bounded chunk binding count")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(self.statement.canonical_bytes());
        for binding in &self.bindings {
            bytes.extend_from_slice(&binding.encode_canonical());
        }
        bytes.extend_from_slice(&self.proof_digest);
        bytes.extend_from_slice(&self.verification_receipt_digest);
        bytes
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = self.prefix();
        bytes.extend_from_slice(&self.record_digest);
        bytes
    }

    fn decode(authority: &EpochFrontierAuthorityV2, bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() < FRONTIER_CHUNK_FIXED_BYTES_V2 || bytes.len() > FRONTIER_CHUNK_MAX_BYTES_V2
        {
            return Err(CheckpointError::Canonical);
        }
        let mut reader = EpochCodecReaderV2::new(bytes);
        if reader.array::<8>()? != FRONTIER_CHUNK_MAGIC_V2
            || reader.u16()? != FRONTIER_WIRE_VERSION_V2
            || reader.u8()? != FRONTIER_TREE_GENERATION_V2
        {
            return Err(CheckpointError::Canonical);
        }
        let chunk_ordinal = reader.u32()?;
        let first_transition = reader.u32()?;
        let last_transition = reader.u32()?;
        let binding_count = usize::try_from(reader.u32()?).map_err(|_| CheckpointError::Limit)?;
        if binding_count == 0
            || binding_count
                > usize::try_from(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2)
                    .map_err(|_| CheckpointError::Limit)?
        {
            return Err(CheckpointError::Canonical);
        }
        let statement_bytes = reader.take(EPOCH_CHUNK_BYTES_V2)?.to_vec();
        let mut bindings = Vec::with_capacity(binding_count);
        for _ in 0..binding_count {
            bindings.push(EpochTransitionBindingV2::decode_canonical(
                reader.take(EPOCH_TRANSITION_BINDING_BYTES_V2)?,
            )?);
        }
        let statement =
            EpochTraceChunkV2::decode_canonical(authority, &bindings, &statement_bytes)?;
        let proof_digest = reader.array()?;
        let verification_receipt_digest = reader.array()?;
        let record_digest = reader.array()?;
        if !reader.is_done() {
            return Err(CheckpointError::Canonical);
        }
        let mut record = Self {
            chunk_ordinal,
            first_transition,
            last_transition,
            statement,
            bindings,
            proof_digest,
            verification_receipt_digest,
            record_digest: [0; 32],
        };
        record.validate()?;
        record.record_digest = sha256_256(
            "z00z.storage.checkpoint.epoch-frontier-trace-chunk.v2",
            "record",
            &[&record.prefix()],
        );
        if record.record_digest != record_digest || record.encode() != bytes {
            return Err(CheckpointError::Canonical);
        }
        Ok(record)
    }

    fn verified_trace_chunk_binding_digest(&self) -> Result<[u8; 32], CheckpointError> {
        let inputs = self.statement.inputs();
        epoch_verified_trace_chunk_span_digest_v2(
            inputs.transition_count,
            self.chunk_ordinal,
            inputs.chunk_count,
            self.first_transition,
            self.transition_count()?,
            self.statement.digest(),
        )
    }

    fn verified_admission_identity_digest(&self) -> Result<[u8; 32], CheckpointError> {
        verified_admission_identity_digest(
            self.verified_trace_chunk_binding_digest()?,
            self.proof_digest,
            self.verification_receipt_digest,
        )
    }
}

fn verified_admission_identity_digest(
    range_binding_digest: [u8; 32],
    proof_digest: [u8; 32],
    verification_receipt_digest: [u8; 32],
) -> Result<[u8; 32], CheckpointError> {
    if [
        range_binding_digest,
        proof_digest,
        verification_receipt_digest,
    ]
    .contains(&[0; 32])
    {
        return Err(CheckpointError::Canonical);
    }
    Ok(sha256_256(
        "z00z.storage.checkpoint.epoch-frontier-admission.v2",
        "actual_verified_chunk",
        &[
            &range_binding_digest,
            &proof_digest,
            &verification_receipt_digest,
        ],
    ))
}

impl FrontierNodeV2 {
    fn chunk(
        authority: &EpochFrontierAuthorityV2,
        record: &FrontierChunkRecordV2,
        admission: &VerifiedEpochTraceChunkAdmissionV2,
    ) -> Result<Self, CheckpointError> {
        let range_binding_digest = record.verified_trace_chunk_binding_digest()?;
        let mut node = Self {
            kind: FrontierNodeKindV2::Chunk,
            tree_level: 0,
            start_height: record.start_height()?,
            end_height: record.end_height()?,
            transition_count: record.transition_count()?,
            config_generation: authority.config_generation,
            authority_generation: authority.authority_generation,
            parameter_generation: authority.parameter_generation,
            runtime_profile_generation: authority.runtime_profile_generation,
            epoch_authority_digest: authority.digest,
            chain_context_digest: authority.chain_context_digest,
            config_digest: authority.config_digest,
            registry_digest: authority.registry_digest,
            runtime_profile_manifest_digest: authority.runtime_profile_manifest_digest,
            parameter_digest: authority.parameter_digest,
            security_budget_digest: authority.security_budget_digest,
            range_binding_digest,
            proof_digest: admission.proof_digest,
            verification_receipt_digest: admission.verification_receipt_digest,
            left_dependency_digest: [0; 32],
            right_dependency_digest: [0; 32],
            proof_bytes: admission.proof_bytes.clone(),
            node_digest: [0; 32],
        };
        node.validate(authority)?;
        node.node_digest = node_digest(&node)?;
        Ok(node)
    }

    fn parent(
        authority: &EpochFrontierAuthorityV2,
        verified: VerifiedEpochParentV2,
    ) -> Result<Self, CheckpointError> {
        let mut node = Self {
            kind: FrontierNodeKindV2::Parent,
            tree_level: verified.tree_level,
            start_height: verified.start_height,
            end_height: verified.end_height,
            transition_count: verified.transition_count,
            config_generation: authority.config_generation,
            authority_generation: authority.authority_generation,
            parameter_generation: authority.parameter_generation,
            runtime_profile_generation: authority.runtime_profile_generation,
            epoch_authority_digest: authority.digest,
            chain_context_digest: authority.chain_context_digest,
            config_digest: authority.config_digest,
            registry_digest: authority.registry_digest,
            runtime_profile_manifest_digest: authority.runtime_profile_manifest_digest,
            parameter_digest: authority.parameter_digest,
            security_budget_digest: authority.security_budget_digest,
            range_binding_digest: verified.range_binding_digest,
            proof_digest: verified.proof_digest,
            verification_receipt_digest: verified.verification_receipt_digest,
            left_dependency_digest: verified.left_node_digest,
            right_dependency_digest: verified.right_node_digest,
            proof_bytes: verified.proof_bytes,
            node_digest: [0; 32],
        };
        node.validate(authority)?;
        node.node_digest = node_digest(&node)?;
        Ok(node)
    }

    fn validate(&self, authority: &EpochFrontierAuthorityV2) -> Result<(), CheckpointError> {
        let expected_count = self
            .end_height
            .checked_sub(self.start_height)
            .and_then(|span| span.checked_add(1));
        if self.start_height < authority.start_height
            || self.end_height > authority.end_height
            || expected_count != Some(u64::from(self.transition_count))
            || self.proof_bytes.is_empty()
            || self.proof_bytes.len()
                > match self.kind {
                    FrontierNodeKindV2::Chunk => RECURSIVE_INGRESS_BYTES_V2,
                    FrontierNodeKindV2::Parent => PLONKY3_PUBLISH_BYTES_V2,
                }
            || self.epoch_authority_digest != authority.digest
            || self.chain_context_digest != authority.chain_context_digest
            || self.config_digest != authority.config_digest
            || self.registry_digest != authority.registry_digest
            || self.runtime_profile_manifest_digest != authority.runtime_profile_manifest_digest
            || self.parameter_digest != authority.parameter_digest
            || self.security_budget_digest != authority.security_budget_digest
            || self.config_generation != authority.config_generation
            || self.authority_generation != authority.authority_generation
            || self.parameter_generation != authority.parameter_generation
            || self.runtime_profile_generation != authority.runtime_profile_generation
            || [
                self.range_binding_digest,
                self.proof_digest,
                self.verification_receipt_digest,
            ]
            .contains(&[0; 32])
        {
            return Err(CheckpointError::Canonical);
        }
        match self.kind {
            FrontierNodeKindV2::Chunk
                if self.tree_level != 0
                    || self.transition_count == 0
                    || self.transition_count > EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2
                    || self.left_dependency_digest != [0; 32]
                    || self.right_dependency_digest != [0; 32] =>
            {
                Err(CheckpointError::Canonical)
            }
            FrontierNodeKindV2::Parent
                if self.tree_level == 0
                    || EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2
                        .checked_shl(u32::from(self.tree_level))
                        != Some(self.transition_count)
                    || self.left_dependency_digest == [0; 32]
                    || self.right_dependency_digest == [0; 32]
                    || self.left_dependency_digest == self.right_dependency_digest =>
            {
                Err(CheckpointError::Canonical)
            }
            FrontierNodeKindV2::Chunk | FrontierNodeKindV2::Parent => Ok(()),
        }
    }
}

fn encode_node(node: &FrontierNodeV2) -> Result<Vec<u8>, CheckpointError> {
    let mut bytes = Vec::with_capacity(
        FRONTIER_NODE_PREFIX_BYTES_V2
            .checked_add(node.proof_bytes.len())
            .and_then(|len| len.checked_add(32))
            .ok_or(CheckpointError::Overflow)?,
    );
    bytes.extend_from_slice(&FRONTIER_NODE_MAGIC_V2);
    bytes.extend_from_slice(&FRONTIER_WIRE_VERSION_V2.to_le_bytes());
    bytes.push(node.kind as u8);
    bytes.push(node.tree_level);
    bytes.extend_from_slice(&node.start_height.to_le_bytes());
    bytes.extend_from_slice(&node.end_height.to_le_bytes());
    bytes.extend_from_slice(&u64::from(node.transition_count).to_le_bytes());
    bytes.extend_from_slice(&node.transition_count.to_le_bytes());
    bytes.extend_from_slice(&node.config_generation.to_le_bytes());
    bytes.extend_from_slice(&node.authority_generation.to_le_bytes());
    bytes.extend_from_slice(&node.parameter_generation.to_le_bytes());
    bytes.extend_from_slice(&node.runtime_profile_generation.to_le_bytes());
    for digest in [
        node.epoch_authority_digest,
        node.chain_context_digest,
        node.config_digest,
        node.registry_digest,
        node.runtime_profile_manifest_digest,
        node.parameter_digest,
        node.security_budget_digest,
        node.range_binding_digest,
        node.proof_digest,
        node.verification_receipt_digest,
        node.left_dependency_digest,
        node.right_dependency_digest,
    ] {
        bytes.extend_from_slice(&digest);
    }
    bytes.extend_from_slice(
        &u32::try_from(node.proof_bytes.len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&node.proof_bytes);
    bytes.extend_from_slice(&node.node_digest);
    Ok(bytes)
}

fn decode_node(bytes: &[u8]) -> Result<FrontierNodeV2, CheckpointError> {
    if bytes.len() < FRONTIER_NODE_PREFIX_BYTES_V2 + 32 || bytes.len() > FRONTIER_NODE_MAX_BYTES_V2
    {
        return Err(CheckpointError::Canonical);
    }
    let mut reader = EpochCodecReaderV2::new(bytes);
    if reader.array::<8>()? != FRONTIER_NODE_MAGIC_V2 || reader.u16()? != FRONTIER_WIRE_VERSION_V2 {
        return Err(CheckpointError::Canonical);
    }
    let kind = match reader.u8()? {
        1 => FrontierNodeKindV2::Chunk,
        2 => FrontierNodeKindV2::Parent,
        _ => return Err(CheckpointError::Canonical),
    };
    let tree_level = reader.u8()?;
    let start_height = reader.u64()?;
    let end_height = reader.u64()?;
    let redundant_count = reader.u64()?;
    let transition_count = reader.u32()?;
    if redundant_count != u64::from(transition_count) {
        return Err(CheckpointError::Canonical);
    }
    let config_generation = reader.u64()?;
    let authority_generation = reader.u64()?;
    let parameter_generation = reader.u32()?;
    let runtime_profile_generation = reader.u16()?;
    let mut node = FrontierNodeV2 {
        kind,
        tree_level,
        start_height,
        end_height,
        transition_count,
        config_generation,
        authority_generation,
        parameter_generation,
        runtime_profile_generation,
        epoch_authority_digest: reader.array()?,
        chain_context_digest: reader.array()?,
        config_digest: reader.array()?,
        registry_digest: reader.array()?,
        runtime_profile_manifest_digest: reader.array()?,
        parameter_digest: reader.array()?,
        security_budget_digest: reader.array()?,
        range_binding_digest: reader.array()?,
        proof_digest: reader.array()?,
        verification_receipt_digest: reader.array()?,
        left_dependency_digest: reader.array()?,
        right_dependency_digest: reader.array()?,
        proof_bytes: {
            let len = usize::try_from(reader.u32()?).map_err(|_| CheckpointError::Limit)?;
            let proof_limit = match kind {
                FrontierNodeKindV2::Chunk => RECURSIVE_INGRESS_BYTES_V2,
                FrontierNodeKindV2::Parent => PLONKY3_PUBLISH_BYTES_V2,
            };
            if len == 0 || len > proof_limit {
                return Err(CheckpointError::Canonical);
            }
            reader.take(len)?.to_vec()
        },
        node_digest: reader.array()?,
    };
    if !reader.is_done() {
        return Err(CheckpointError::Canonical);
    }
    let encoded_digest = node.node_digest;
    node.node_digest = [0; 32];
    node.node_digest = node_digest(&node)?;
    if node.node_digest != encoded_digest || encode_node(&node)? != bytes {
        return Err(CheckpointError::Canonical);
    }
    Ok(node)
}

fn node_digest(node: &FrontierNodeV2) -> Result<[u8; 32], CheckpointError> {
    let mut zeroed = node.clone();
    zeroed.node_digest = [0; 32];
    let bytes = encode_node(&zeroed)?;
    Ok(sha256_256(
        "z00z.storage.checkpoint.epoch-frontier-node.v2",
        "node",
        &[&bytes],
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum FrontierJournalKindV2 {
    NodeInstalled = 1,
    MergeScheduled = 2,
    ParentInstalled = 3,
    ChildrenRetired = 4,
}

#[derive(Clone, Copy)]
struct FrontierJournalRecordV2 {
    kind: FrontierJournalKindV2,
    sequence: u64,
    start_height: u64,
    end_height: u64,
    transition_count: u32,
    node_digest: [u8; 32],
    left_digest: [u8; 32],
    right_digest: [u8; 32],
    digest: [u8; 32],
}

impl FrontierJournalRecordV2 {
    fn node_installed(sequence: u64, node: &FrontierNodeV2) -> Result<Self, CheckpointError> {
        Self::new(
            FrontierJournalKindV2::NodeInstalled,
            sequence,
            node,
            node.range_binding_digest,
            verified_admission_identity_digest(
                node.range_binding_digest,
                node.proof_digest,
                node.verification_receipt_digest,
            )?,
        )
    }

    fn merge_scheduled(
        sequence: u64,
        left: &FrontierNodeV2,
        right: &FrontierNodeV2,
    ) -> Result<Self, CheckpointError> {
        let shell = FrontierNodeV2 {
            kind: FrontierNodeKindV2::Parent,
            tree_level: left.tree_level + 1,
            start_height: left.start_height,
            end_height: right.end_height,
            transition_count: left
                .transition_count
                .checked_add(right.transition_count)
                .ok_or(CheckpointError::Overflow)?,
            config_generation: left.config_generation,
            authority_generation: left.authority_generation,
            parameter_generation: left.parameter_generation,
            runtime_profile_generation: left.runtime_profile_generation,
            epoch_authority_digest: left.epoch_authority_digest,
            chain_context_digest: left.chain_context_digest,
            config_digest: left.config_digest,
            registry_digest: left.registry_digest,
            runtime_profile_manifest_digest: left.runtime_profile_manifest_digest,
            parameter_digest: left.parameter_digest,
            security_budget_digest: left.security_budget_digest,
            range_binding_digest: [0; 32],
            proof_digest: [0; 32],
            verification_receipt_digest: [0; 32],
            left_dependency_digest: left.node_digest,
            right_dependency_digest: right.node_digest,
            proof_bytes: Vec::new(),
            node_digest: [0; 32],
        };
        Self::new(
            FrontierJournalKindV2::MergeScheduled,
            sequence,
            &shell,
            left.node_digest,
            right.node_digest,
        )
    }

    fn parent_installed(sequence: u64, node: &FrontierNodeV2) -> Result<Self, CheckpointError> {
        Self::new(
            FrontierJournalKindV2::ParentInstalled,
            sequence,
            node,
            node.left_dependency_digest,
            node.right_dependency_digest,
        )
    }

    fn children_retired(sequence: u64, node: &FrontierNodeV2) -> Result<Self, CheckpointError> {
        Self::new(
            FrontierJournalKindV2::ChildrenRetired,
            sequence,
            node,
            node.left_dependency_digest,
            node.right_dependency_digest,
        )
    }

    fn new(
        kind: FrontierJournalKindV2,
        sequence: u64,
        node: &FrontierNodeV2,
        left_digest: [u8; 32],
        right_digest: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        let node_digest = if kind == FrontierJournalKindV2::MergeScheduled {
            [0; 32]
        } else {
            node.node_digest
        };
        let mut record = Self {
            kind,
            sequence,
            start_height: node.start_height,
            end_height: node.end_height,
            transition_count: node.transition_count,
            node_digest,
            left_digest,
            right_digest,
            digest: [0; 32],
        };
        record.digest = sha256_256(
            "z00z.storage.checkpoint.epoch-frontier-journal.v2",
            "record",
            &[&record.prefix()],
        );
        Ok(record)
    }

    fn prefix(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FRONTIER_JOURNAL_BYTES_V2 - 32);
        bytes.extend_from_slice(&FRONTIER_JOURNAL_MAGIC_V2);
        bytes.extend_from_slice(&FRONTIER_WIRE_VERSION_V2.to_le_bytes());
        bytes.push(self.kind as u8);
        bytes.extend_from_slice(&self.sequence.to_le_bytes());
        bytes.extend_from_slice(&self.start_height.to_le_bytes());
        bytes.extend_from_slice(&self.end_height.to_le_bytes());
        bytes.extend_from_slice(&self.transition_count.to_le_bytes());
        bytes.extend_from_slice(&self.node_digest);
        bytes.extend_from_slice(&self.left_digest);
        bytes.extend_from_slice(&self.right_digest);
        bytes
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = self.prefix();
        bytes.extend_from_slice(&self.digest);
        bytes
    }

    fn decode(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() != FRONTIER_JOURNAL_BYTES_V2 {
            return Err(CheckpointError::Canonical);
        }
        let mut reader = EpochCodecReaderV2::new(bytes);
        if reader.array::<8>()? != FRONTIER_JOURNAL_MAGIC_V2
            || reader.u16()? != FRONTIER_WIRE_VERSION_V2
        {
            return Err(CheckpointError::Canonical);
        }
        let kind = match reader.u8()? {
            1 => FrontierJournalKindV2::NodeInstalled,
            2 => FrontierJournalKindV2::MergeScheduled,
            3 => FrontierJournalKindV2::ParentInstalled,
            4 => FrontierJournalKindV2::ChildrenRetired,
            _ => return Err(CheckpointError::Canonical),
        };
        let record = Self {
            kind,
            sequence: reader.u64()?,
            start_height: reader.u64()?,
            end_height: reader.u64()?,
            transition_count: reader.u32()?,
            node_digest: reader.array()?,
            left_digest: reader.array()?,
            right_digest: reader.array()?,
            digest: reader.array()?,
        };
        if !reader.is_done()
            || sha256_256(
                "z00z.storage.checkpoint.epoch-frontier-journal.v2",
                "record",
                &[&record.prefix()],
            ) != record.digest
            || record.encode() != bytes
        {
            return Err(CheckpointError::Canonical);
        }
        Ok(record)
    }
}

#[derive(Default)]
struct FrontierJournalStateV2 {
    next_sequence: u64,
    active_nodes: BTreeMap<[u8; 32], (u64, u64, u32)>,
    verified_chunks: BTreeMap<u64, ([u8; 32], [u8; 32])>,
    retired_nodes: BTreeSet<[u8; 32]>,
    scheduled: BTreeSet<([u8; 32], [u8; 32])>,
    unretired_parents: BTreeSet<[u8; 32]>,
    parent_dependencies: BTreeMap<[u8; 32], ([u8; 32], [u8; 32])>,
}

impl FrontierJournalStateV2 {
    fn apply(&mut self, record: FrontierJournalRecordV2) -> Result<(), CheckpointError> {
        self.next_sequence = record
            .sequence
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        match record.kind {
            FrontierJournalKindV2::NodeInstalled => {
                if record.node_digest == [0; 32]
                    || record.left_digest == [0; 32]
                    || record.right_digest == [0; 32]
                    || record.transition_count == 0
                    || record.transition_count > EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2
                    || record
                        .end_height
                        .checked_sub(record.start_height)
                        .and_then(|span| span.checked_add(1))
                        != Some(u64::from(record.transition_count))
                    || self
                        .verified_chunks
                        .insert(
                            record.start_height,
                            (record.left_digest, record.right_digest),
                        )
                        .is_some()
                    || self
                        .active_nodes
                        .insert(
                            record.node_digest,
                            (
                                record.start_height,
                                record.end_height,
                                record.transition_count,
                            ),
                        )
                        .is_some()
                {
                    return Err(CheckpointError::Canonical);
                }
            }
            FrontierJournalKindV2::MergeScheduled => {
                let left = self
                    .active_nodes
                    .get(&record.left_digest)
                    .copied()
                    .ok_or(CheckpointError::Canonical)?;
                let right = self
                    .active_nodes
                    .get(&record.right_digest)
                    .copied()
                    .ok_or(CheckpointError::Canonical)?;
                let expected_count = left
                    .2
                    .checked_add(right.2)
                    .ok_or(CheckpointError::Overflow)?;
                if record.node_digest != [0; 32]
                    || record.left_digest == [0; 32]
                    || record.right_digest == [0; 32]
                    || record.left_digest == record.right_digest
                    || left.1.checked_add(1) != Some(right.0)
                    || left.2 != right.2
                    || (
                        record.start_height,
                        record.end_height,
                        record.transition_count,
                    ) != (left.0, right.1, expected_count)
                    || !self
                        .scheduled
                        .insert((record.left_digest, record.right_digest))
                {
                    return Err(CheckpointError::Canonical);
                }
            }
            FrontierJournalKindV2::ParentInstalled => {
                let left = self
                    .active_nodes
                    .get(&record.left_digest)
                    .copied()
                    .ok_or(CheckpointError::Canonical)?;
                let right = self
                    .active_nodes
                    .get(&record.right_digest)
                    .copied()
                    .ok_or(CheckpointError::Canonical)?;
                let expected_count = left
                    .2
                    .checked_add(right.2)
                    .ok_or(CheckpointError::Overflow)?;
                if record.node_digest == [0; 32]
                    || record.node_digest == record.left_digest
                    || record.node_digest == record.right_digest
                    || !self
                        .scheduled
                        .contains(&(record.left_digest, record.right_digest))
                    || left.1.checked_add(1) != Some(right.0)
                    || left.2 != right.2
                    || (
                        record.start_height,
                        record.end_height,
                        record.transition_count,
                    ) != (left.0, right.1, expected_count)
                    || self
                        .active_nodes
                        .insert(
                            record.node_digest,
                            (
                                record.start_height,
                                record.end_height,
                                record.transition_count,
                            ),
                        )
                        .is_some()
                {
                    return Err(CheckpointError::Canonical);
                }
                if !self.unretired_parents.insert(record.node_digest)
                    || self
                        .parent_dependencies
                        .insert(
                            record.node_digest,
                            (record.left_digest, record.right_digest),
                        )
                        .is_some()
                {
                    return Err(CheckpointError::Canonical);
                }
            }
            FrontierJournalKindV2::ChildrenRetired => {
                let parent = self
                    .active_nodes
                    .get(&record.node_digest)
                    .copied()
                    .ok_or(CheckpointError::Canonical)?;
                let left = self
                    .active_nodes
                    .get(&record.left_digest)
                    .copied()
                    .ok_or(CheckpointError::Canonical)?;
                let right = self
                    .active_nodes
                    .get(&record.right_digest)
                    .copied()
                    .ok_or(CheckpointError::Canonical)?;
                if !self.unretired_parents.contains(&record.node_digest)
                    || self.parent_dependencies.get(&record.node_digest).copied()
                        != Some((record.left_digest, record.right_digest))
                    || !self
                        .scheduled
                        .contains(&(record.left_digest, record.right_digest))
                    || left.1.checked_add(1) != Some(right.0)
                    || left.2 != right.2
                    || (
                        record.start_height,
                        record.end_height,
                        record.transition_count,
                    ) != parent
                    || parent
                        != (
                            left.0,
                            right.1,
                            left.2
                                .checked_add(right.2)
                                .ok_or(CheckpointError::Overflow)?,
                        )
                {
                    return Err(CheckpointError::Canonical);
                }
                self.unretired_parents.remove(&record.node_digest);
                self.parent_dependencies.remove(&record.node_digest);
                self.active_nodes.remove(&record.left_digest);
                self.active_nodes.remove(&record.right_digest);
                self.retired_nodes.insert(record.left_digest);
                self.retired_nodes.insert(record.right_digest);
                self.scheduled
                    .remove(&(record.left_digest, record.right_digest));
            }
        }
        Ok(())
    }

    fn overlaps(&self, start: u64, end: u64) -> bool {
        self.active_nodes
            .values()
            .any(|(existing_start, existing_end, _)| {
                start <= *existing_end && *existing_start <= end
            })
    }
}

fn validate_complete_segment_cover(
    authority: &EpochFrontierAuthorityV2,
    nodes: &[FrontierNodeV2],
) -> Result<(), CheckpointError> {
    if nodes.is_empty() {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
        ));
    }
    let mut expected = authority.start_height;
    let mut count = 0_u64;
    for node in nodes {
        if node.start_height != expected {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
            ));
        }
        expected = node
            .end_height
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        count = count
            .checked_add(u64::from(node.transition_count))
            .ok_or(CheckpointError::Overflow)?;
    }
    if expected
        != authority
            .end_height
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?
        || count != u64::from(authority.transition_count)
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
        ));
    }
    Ok(())
}

fn encode_authority(authority: &EpochFrontierAuthorityV2) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(FRONTIER_AUTHORITY_BYTES_V2);
    bytes.extend_from_slice(&FRONTIER_AUTHORITY_MAGIC_V2);
    bytes.extend_from_slice(&FRONTIER_WIRE_VERSION_V2.to_le_bytes());
    bytes.push(authority.cadence_class as u8);
    bytes.push(FRONTIER_TREE_GENERATION_V2);
    bytes.extend_from_slice(&authority.epoch_index.to_le_bytes());
    bytes.extend_from_slice(&authority.start_height.to_le_bytes());
    bytes.extend_from_slice(&authority.end_height.to_le_bytes());
    bytes.extend_from_slice(&authority.cadence_blocks.to_le_bytes());
    bytes.extend_from_slice(&authority.transition_count.to_le_bytes());
    bytes.extend_from_slice(&authority.chunk_count.to_le_bytes());
    bytes.extend_from_slice(&authority.runtime_profile_generation.to_le_bytes());
    bytes.extend_from_slice(&authority.config_generation.to_le_bytes());
    bytes.extend_from_slice(&authority.authority_generation.to_le_bytes());
    bytes.extend_from_slice(&authority.activation_height.to_le_bytes());
    bytes.extend_from_slice(&authority.rollback_floor.to_le_bytes());
    bytes.extend_from_slice(&authority.parameter_generation.to_le_bytes());
    for digest in [
        authority.start_root,
        authority.chain_context_digest,
        authority.predicate_digest,
        authority.parameter_digest,
        authority.verifier_bundle_digest,
        authority.security_budget_digest,
        authority.config_digest,
        authority.registry_digest,
        authority.runtime_profile_manifest_digest,
        authority.history_authority_bundle_digest,
    ] {
        bytes.extend_from_slice(&digest);
    }
    bytes.extend_from_slice(&authority.digest);
    bytes
}

fn decode_authority(bytes: &[u8]) -> Result<EpochFrontierAuthorityV2, CheckpointError> {
    if bytes.len() != FRONTIER_AUTHORITY_BYTES_V2 {
        return Err(CheckpointError::Canonical);
    }
    let mut reader = EpochCodecReaderV2::new(bytes);
    if reader.array::<8>()? != FRONTIER_AUTHORITY_MAGIC_V2
        || reader.u16()? != FRONTIER_WIRE_VERSION_V2
    {
        return Err(CheckpointError::Canonical);
    }
    let cadence_class = match reader.u8()? {
        1 => EpochCadenceClassV2::Production,
        2 => EpochCadenceClassV2::BoundedSimulation,
        _ => return Err(CheckpointError::Canonical),
    };
    if reader.u8()? != FRONTIER_TREE_GENERATION_V2 {
        return Err(CheckpointError::Canonical);
    }
    let authority = EpochFrontierAuthorityV2 {
        cadence_class,
        epoch_index: reader.u64()?,
        start_height: reader.u64()?,
        end_height: reader.u64()?,
        cadence_blocks: reader.u64()?,
        transition_count: reader.u32()?,
        chunk_count: reader.u32()?,
        runtime_profile_generation: reader.u16()?,
        config_generation: reader.u64()?,
        authority_generation: reader.u64()?,
        activation_height: reader.u64()?,
        rollback_floor: reader.u64()?,
        parameter_generation: reader.u32()?,
        start_root: reader.array()?,
        chain_context_digest: reader.array()?,
        predicate_digest: reader.array()?,
        parameter_digest: reader.array()?,
        verifier_bundle_digest: reader.array()?,
        security_budget_digest: reader.array()?,
        config_digest: reader.array()?,
        registry_digest: reader.array()?,
        runtime_profile_manifest_digest: reader.array()?,
        history_authority_bundle_digest: reader.array()?,
        digest: reader.array()?,
    };
    if !reader.is_done()
        || authority.runtime_profile_generation == 0
        || authority.config_generation == 0
        || authority.authority_generation == 0
        || authority.parameter_generation == 0
        || authority.chunk_count == 0
        || authority
            .transition_count
            .checked_add(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 - 1)
            .and_then(|count| count.checked_div(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2))
            != Some(authority.chunk_count)
        || [
            authority.start_root,
            authority.chain_context_digest,
            authority.predicate_digest,
            authority.parameter_digest,
            authority.verifier_bundle_digest,
            authority.security_budget_digest,
            authority.config_digest,
            authority.registry_digest,
            authority.runtime_profile_manifest_digest,
            authority.history_authority_bundle_digest,
        ]
        .contains(&[0; 32])
        || authority.digest != authority_digest(&authority)
        || encode_authority(&authority) != bytes
    {
        return Err(CheckpointError::Canonical);
    }
    Ok(authority)
}

fn authority_digest(authority: &EpochFrontierAuthorityV2) -> [u8; 32] {
    let mut copy = *authority;
    copy.digest = [0; 32];
    sha256_256(
        "z00z.storage.checkpoint.epoch-frontier-authority.v2",
        "authority",
        &[&encode_authority(&copy)],
    )
}

fn chunk_file_name(ordinal: u32) -> String {
    format!("{ordinal:010}.chunk")
}

fn node_file_name(digest: [u8; 32]) -> String {
    format!("{}.node", lowercase_hex(digest))
}

fn journal_file_name(sequence: u64, digest: [u8; 32]) -> String {
    format!("{sequence:020}-{}.journal", lowercase_hex(digest))
}

fn scavenge_temporary_files(
    directory: &SecureDir,
    max_entries: usize,
) -> Result<(), CheckpointError> {
    let mut changed = false;
    for name in directory
        .read_dir_bounded(max_entries)
        .map_err(|_| CheckpointError::Storage)?
    {
        let name = name.into_string().map_err(|_| CheckpointError::Canonical)?;
        if name.starts_with(".tmp-") {
            directory
                .remove_file(&name)
                .map_err(|_| CheckpointError::Storage)?;
            changed = true;
        }
    }
    if changed {
        directory.sync().map_err(|_| CheckpointError::Storage)?;
    }
    Ok(())
}

fn write_once(directory: &SecureDir, name: &str, bytes: &[u8]) -> Result<(), CheckpointError> {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    if let Ok(existing) = directory.read_file_bounded(name, bytes.len() as u64) {
        return if existing == bytes {
            Ok(())
        } else {
            Err(CheckpointError::Canonical)
        };
    }
    let mut temporary = None;
    for _ in 0..8 {
        let candidate = format!(
            ".tmp-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        if let Ok(file) = directory.create_file(&candidate) {
            temporary = Some((candidate, file));
            break;
        }
    }
    let (temporary_name, mut file) = temporary.ok_or(CheckpointError::Storage)?;
    if file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .is_err()
    {
        let _ = directory.remove_file(&temporary_name);
        return Err(CheckpointError::Storage);
    }
    drop(file);
    if directory.rename_no_clobber(&temporary_name, name).is_err() {
        let _ = directory.remove_file(&temporary_name);
        return Err(CheckpointError::Storage);
    }
    directory.sync().map_err(|_| CheckpointError::Storage)
}

fn lowercase_hex(digest: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(mark: u8) -> [u8; 32] {
        [mark; 32]
    }

    fn authority() -> EpochFrontierAuthorityV2 {
        let identity = Plonky3HistoryAuthorityResolverV2::resolve_active()
            .expect("history authority")
            .identity();
        EpochFrontierAuthorityV2::new(EpochFrontierAuthorityInputsV2 {
            cadence_class: EpochCadenceClassV2::BoundedSimulation,
            epoch_index: 0,
            cadence_blocks: 16,
            start_root: digest(20),
            chain_context_digest: digest(1),
            predicate_digest: digest(2),
            parameter_digest: identity.verifier_parameter_digest,
            verifier_bundle_digest: identity.verifier_bundle_digest,
        })
        .expect("frontier authority")
    }

    #[test]
    fn test_authority_mismatch_rejects() {
        let identity = Plonky3HistoryAuthorityResolverV2::resolve_active()
            .expect("history authority")
            .identity();
        let mut inputs = EpochFrontierAuthorityInputsV2 {
            cadence_class: EpochCadenceClassV2::BoundedSimulation,
            epoch_index: 0,
            cadence_blocks: 16,
            start_root: digest(20),
            chain_context_digest: digest(1),
            predicate_digest: digest(2),
            parameter_digest: identity.verifier_parameter_digest,
            verifier_bundle_digest: identity.verifier_bundle_digest,
        };
        inputs.parameter_digest[0] ^= 1;
        assert!(matches!(
            EpochFrontierAuthorityV2::new(inputs),
            Err(CheckpointError::Authority)
        ));
    }

    fn chunk_admission(
        authority: EpochFrontierAuthorityV2,
        chunk_ordinal: u32,
        mark: u8,
    ) -> VerifiedEpochTraceChunkAdmissionV2 {
        let first_transition = chunk_ordinal
            .checked_mul(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2)
            .expect("bounded first transition");
        let last_transition = first_transition
            .checked_add(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 - 1)
            .map(|last| last.min(authority.transition_count() - 1))
            .expect("bounded last transition");
        let bindings = (first_transition..=last_transition)
            .map(|ordinal| {
                let ordinal_mark = u8::try_from(ordinal).expect("bounded test ordinal");
                EpochTransitionBindingV2::new(EpochTransitionInputsV2 {
                    ordinal,
                    height: authority.start_height() + u64::from(ordinal),
                    checkpoint_id: digest(50_u8.wrapping_add(ordinal_mark)),
                    predecessor: (ordinal != 0).then(|| digest(49_u8.wrapping_add(ordinal_mark))),
                    recursive_transition_statement_digest: digest(11_u8.wrapping_add(ordinal_mark)),
                    checkpoint_exec_tx_root: digest(12_u8.wrapping_add(ordinal_mark)),
                    checkpoint_exec_tx_count: 1,
                    checkpoint_statement_digest: digest(1_u8.wrapping_add(ordinal_mark)),
                    checkpoint_statement_core_digest: digest(2_u8.wrapping_add(ordinal_mark)),
                    checkpoint_link_digest: digest(3_u8.wrapping_add(ordinal_mark)),
                    checkpoint_artifact_digest: digest(4_u8.wrapping_add(ordinal_mark)),
                    delta_root: digest(5_u8.wrapping_add(ordinal_mark)),
                    witness_root: digest(6_u8.wrapping_add(ordinal_mark)),
                    journal_digest: digest(7_u8.wrapping_add(ordinal_mark)),
                    challenge_content_digest: digest(8_u8.wrapping_add(ordinal_mark)),
                    da_payload_commitment: digest(9_u8.wrapping_add(ordinal_mark)),
                    prior_recursive_output_root: (ordinal != 0)
                        .then(|| digest(13_u8.wrapping_add(ordinal_mark))),
                    pre_settlement_root: if ordinal == 0 {
                        authority.start_root()
                    } else {
                        digest(100_u8.wrapping_add(ordinal_mark))
                    },
                    post_settlement_root: digest(101_u8.wrapping_add(ordinal_mark)),
                    pre_definition_root: digest(14_u8.wrapping_add(ordinal_mark)),
                    post_definition_root: digest(15_u8.wrapping_add(ordinal_mark)),
                    trace_digest: digest(16_u8.wrapping_add(ordinal_mark)),
                    update_trace_digest: digest(17_u8.wrapping_add(ordinal_mark)),
                    declared_work_digest: digest(18_u8.wrapping_add(ordinal_mark)),
                    pre_uniqueness_context_digest: digest(19_u8.wrapping_add(ordinal_mark)),
                    spent_uniqueness_precommit: digest(20_u8.wrapping_add(ordinal_mark)),
                    output_uniqueness_precommit: digest(21_u8.wrapping_add(ordinal_mark)),
                    event_vector_digest: digest(10_u8.wrapping_add(ordinal_mark)),
                    event_count: 3,
                    event_bytes: 1,
                    uniqueness_row_count: 1,
                    jmt_record_count: 1,
                    jmt_envelope_count: 1,
                    jmt_update_count: 1,
                })
                .expect("canonical transition")
            })
            .collect::<Vec<_>>();
        let transition_digests = bindings
            .iter()
            .map(EpochTransitionBindingV2::digest)
            .collect::<Vec<_>>();
        let input_slice_commitment =
            epoch_ordered_digest_root_v2(EPOCH_TRANSITION_SLICE_DOMAIN_V2, &transition_digests)
                .expect("transition slice commitment");
        let first = bindings.first().expect("first binding").inputs();
        let last = bindings.last().expect("last binding").inputs();
        let prior_bindings = (0..first_transition)
            .map(|ordinal| {
                let ordinal_mark = u8::try_from(ordinal).expect("bounded test ordinal");
                EpochTransitionBindingV2::new(EpochTransitionInputsV2 {
                    ordinal,
                    height: authority.start_height() + u64::from(ordinal),
                    checkpoint_id: digest(50_u8.wrapping_add(ordinal_mark)),
                    predecessor: (ordinal != 0).then(|| digest(49_u8.wrapping_add(ordinal_mark))),
                    recursive_transition_statement_digest: digest(11_u8.wrapping_add(ordinal_mark)),
                    checkpoint_exec_tx_root: digest(12_u8.wrapping_add(ordinal_mark)),
                    checkpoint_exec_tx_count: 1,
                    checkpoint_statement_digest: digest(1_u8.wrapping_add(ordinal_mark)),
                    checkpoint_statement_core_digest: digest(2_u8.wrapping_add(ordinal_mark)),
                    checkpoint_link_digest: digest(3_u8.wrapping_add(ordinal_mark)),
                    checkpoint_artifact_digest: digest(4_u8.wrapping_add(ordinal_mark)),
                    delta_root: digest(5_u8.wrapping_add(ordinal_mark)),
                    witness_root: digest(6_u8.wrapping_add(ordinal_mark)),
                    journal_digest: digest(7_u8.wrapping_add(ordinal_mark)),
                    challenge_content_digest: digest(8_u8.wrapping_add(ordinal_mark)),
                    da_payload_commitment: digest(9_u8.wrapping_add(ordinal_mark)),
                    prior_recursive_output_root: (ordinal != 0)
                        .then(|| digest(13_u8.wrapping_add(ordinal_mark))),
                    pre_settlement_root: if ordinal == 0 {
                        authority.start_root()
                    } else {
                        digest(100_u8.wrapping_add(ordinal_mark))
                    },
                    post_settlement_root: digest(101_u8.wrapping_add(ordinal_mark)),
                    pre_definition_root: digest(14_u8.wrapping_add(ordinal_mark)),
                    post_definition_root: digest(15_u8.wrapping_add(ordinal_mark)),
                    trace_digest: digest(16_u8.wrapping_add(ordinal_mark)),
                    update_trace_digest: digest(17_u8.wrapping_add(ordinal_mark)),
                    declared_work_digest: digest(18_u8.wrapping_add(ordinal_mark)),
                    pre_uniqueness_context_digest: digest(19_u8.wrapping_add(ordinal_mark)),
                    spent_uniqueness_precommit: digest(20_u8.wrapping_add(ordinal_mark)),
                    output_uniqueness_precommit: digest(21_u8.wrapping_add(ordinal_mark)),
                    event_vector_digest: digest(10_u8.wrapping_add(ordinal_mark)),
                    event_count: 3,
                    event_bytes: 1,
                    uniqueness_row_count: 1,
                    jmt_record_count: 1,
                    jmt_envelope_count: 1,
                    jmt_update_count: 1,
                })
                .expect("canonical prior transition")
            })
            .collect::<Vec<_>>();
        let input_accumulator = prior_bindings.iter().copied().fold(
            epoch_stream_initial_accumulator(authority),
            |accumulator, transition| {
                epoch_stream_step_accumulator(authority, accumulator, transition)
            },
        );
        let output_accumulator =
            bindings
                .iter()
                .copied()
                .fold(input_accumulator, |accumulator, transition| {
                    epoch_stream_step_accumulator(authority, accumulator, transition)
                });
        let transition_statement = EpochTraceChunkV2::new(
            &authority,
            &bindings,
            EpochTraceChunkInputsV2 {
                table: EpochAirTableV2::Transition,
                replica: 0,
                chunk_ordinal,
                chunk_count: authority.chunk_count(),
                first_transition,
                last_transition,
                transition_count: authority.transition_count(),
                row_start: u64::from(first_transition),
                row_count: u64::from(last_transition - first_transition + 1),
                event_start: u64::from(first_transition) * 3,
                event_count: u64::from(last_transition - first_transition + 1) * 3,
                frontier_authority_digest: authority.digest(),
                chain_context_digest: authority.chain_context_digest(),
                predicate_digest: authority.predicate_digest(),
                input_state_root: first.pre_settlement_root,
                output_state_root: last.post_settlement_root,
                input_accumulator,
                output_accumulator,
                input_slice_commitment,
                parameter_digest: authority.parameter_digest(),
                verifier_bundle_digest: authority.verifier_bundle_digest(),
                security_budget_digest: authority.security_budget_digest(),
            },
        )
        .expect("canonical trace-chunk statement");
        VerifiedEpochTraceChunkAdmissionV2 {
            transition_statement,
            bindings,
            proof_digest: digest(mark.wrapping_add(13)),
            verification_receipt_digest: digest(mark.wrapping_add(14)),
            proof_bytes: vec![mark.wrapping_add(15)],
        }
    }

    fn chunk_record(
        authority: EpochFrontierAuthorityV2,
        chunk_ordinal: u32,
        mark: u8,
    ) -> (FrontierChunkRecordV2, FrontierNodeV2) {
        let admission = chunk_admission(authority, chunk_ordinal, mark);
        let record = FrontierChunkRecordV2::new(&admission).expect("chunk record");
        let node =
            FrontierNodeV2::chunk(&authority, &record, &admission).expect("trace-chunk node");
        (record, node)
    }

    fn closed_manifest(
        authority: EpochFrontierAuthorityV2,
        records: &[&FrontierChunkRecordV2],
    ) -> EpochProofWorkManifestV2 {
        let transitions = records
            .iter()
            .flat_map(|record| record.bindings.iter().copied())
            .collect::<Vec<_>>();
        let end_root = transitions
            .last()
            .expect("closed manifest transition")
            .inputs()
            .post_settlement_root;
        EpochProofWorkManifestV2::new(EpochProofWorkManifestInputsV2 {
            cadence_class: authority.cadence_class,
            epoch_index: authority.epoch_index,
            start_height: authority.start_height,
            end_height: authority.end_height,
            transition_count: authority.transition_count,
            parameter_generation: authority.parameter_generation,
            runtime_profile_generation: authority.runtime_profile_generation,
            config_generation: authority.config_generation,
            authority_generation: authority.authority_generation,
            chain_context_digest: authority.chain_context_digest,
            predicate_digest: authority.predicate_digest,
            parameter_digest: authority.parameter_digest,
            verifier_bundle_digest: authority.verifier_bundle_digest,
            security_budget_digest: authority.security_budget_digest,
            config_digest: authority.config_digest,
            registry_digest: authority.registry_digest,
            runtime_profile_manifest_digest: authority.runtime_profile_manifest_digest,
            frontier_authority_digest: authority.digest,
            epoch_close_anchor_digest: digest(107),
            nova_chain_root: None,
            start_root: authority.start_root,
            end_root,
            transitions,
        })
        .expect("closed work manifest")
    }

    #[test]
    fn test_verified_chunk_is_the_only_canonical_frontier_source() {
        let authority = authority();
        let admission = chunk_admission(authority, 0, 21);
        let record = FrontierChunkRecordV2::new(&admission).expect("chunk record");

        assert_eq!(record.chunk_ordinal, 0);
        assert_eq!(record.first_transition, 0);
        assert_eq!(record.last_transition, 7);
        assert_eq!(record.statement, admission.transition_statement);
        assert_eq!(record.bindings, admission.bindings);
        assert_eq!(record.proof_digest, admission.proof_digest);
        assert_eq!(
            record.verification_receipt_digest,
            admission.verification_receipt_digest
        );
    }

    #[test]
    fn test_predecessor_frontier_generation_rejects() {
        let authority = authority();
        let (record, node) = chunk_record(authority, 0, 21);

        let mut authority_bytes = encode_authority(&authority);
        authority_bytes[7] = b'3';
        assert!(decode_authority(&authority_bytes).is_err());
        let mut authority_generation = encode_authority(&authority);
        authority_generation[11] = FRONTIER_TREE_GENERATION_V2 - 1;
        assert!(decode_authority(&authority_generation).is_err());

        let mut chunk_bytes = record.encode();
        chunk_bytes[7] = b'3';
        assert!(FrontierChunkRecordV2::decode(&authority, &chunk_bytes).is_err());
        let mut chunk_generation = record.encode();
        chunk_generation[10] = FRONTIER_TREE_GENERATION_V2 - 1;
        assert!(FrontierChunkRecordV2::decode(&authority, &chunk_generation).is_err());

        let mut node_bytes = encode_node(&node).expect("canonical node");
        node_bytes[7] = b'3';
        assert!(decode_node(&node_bytes).is_err());

        let journal = FrontierJournalRecordV2::node_installed(0, &node).expect("canonical journal");
        let mut journal_bytes = journal.encode();
        journal_bytes[7] = b'3';
        assert!(FrontierJournalRecordV2::decode(&journal_bytes).is_err());
    }

    #[test]
    fn test_chunk_bundle_uses_ingress_cap_only() {
        let authority = authority();
        let mut admission = chunk_admission(authority, 0, 21);
        admission.proof_bytes = vec![0x5a; PLONKY3_PUBLISH_BYTES_V2 + 1];
        let record = FrontierChunkRecordV2::new(&admission).expect("chunk record");
        let chunk = FrontierNodeV2::chunk(&authority, &record, &admission)
            .expect("internal chunk bundle may exceed publish cap");
        assert_eq!(chunk.proof_bytes.len(), PLONKY3_PUBLISH_BYTES_V2 + 1);
        assert!(decode_node(&encode_node(&chunk).expect("encode chunk node")).is_ok());

        assert!(FrontierNodeV2::parent(
            &authority,
            VerifiedEpochParentV2 {
                left_node_digest: digest(71),
                right_node_digest: digest(72),
                start_height: authority.start_height(),
                end_height: authority.start_height()
                    + u64::from(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 * 2)
                    - 1,
                transition_count: EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 * 2,
                tree_level: 1,
                range_binding_digest: digest(73),
                proof_digest: digest(74),
                verification_receipt_digest: digest(75),
                proof_bytes: vec![0x5a; PLONKY3_PUBLISH_BYTES_V2 + 1],
            },
        )
        .is_err());
    }

    fn write_chunk(
        frontier: &EpochProofFrontierV2,
        record: &FrontierChunkRecordV2,
        node: &FrontierNodeV2,
    ) {
        write_once(
            &frontier.chunks,
            &chunk_file_name(record.chunk_ordinal),
            &record.encode(),
        )
        .expect("chunk write");
        frontier.write_node(node).expect("node write");
    }

    fn commit_chunk(
        frontier: &EpochProofFrontierV2,
        record: &FrontierChunkRecordV2,
        node: &FrontierNodeV2,
        sequence: u64,
    ) {
        write_chunk(frontier, record, node);
        frontier
            .append_journal(
                FrontierJournalRecordV2::node_installed(sequence, node).expect("chunk journal"),
            )
            .expect("chunk commit");
    }

    fn parent_for_job(
        authority: EpochFrontierAuthorityV2,
        job: EpochMergeJobV2,
        mark: u8,
    ) -> FrontierNodeV2 {
        FrontierNodeV2::parent(
            &authority,
            VerifiedEpochParentV2 {
                left_node_digest: job.left_node_digest(),
                right_node_digest: job.right_node_digest(),
                start_height: job.start_height(),
                end_height: job.end_height(),
                transition_count: job.transition_count().expect("parent count"),
                tree_level: job.tree_level().expect("parent level"),
                range_binding_digest: digest(mark),
                proof_digest: digest(mark.wrapping_add(1)),
                verification_receipt_digest: digest(mark.wrapping_add(2)),
                proof_bytes: vec![mark.wrapping_add(3)],
            },
        )
        .expect("parent node")
    }

    #[test]
    fn test_recovery_cleans_files() {
        let temp = tempfile::tempdir().expect("frontier root");
        let root = temp.path().join("frontier");
        let authority = authority();
        let (record, node) = chunk_record(authority, 0, 31);
        {
            let frontier = EpochProofFrontierV2::open(&root, authority).expect("open frontier");
            write_chunk(&frontier, &record, &node);
            let mut stale = frontier
                .journal
                .create_file(".tmp-stale")
                .expect("temporary journal");
            stale.write_all(b"partial").expect("temporary bytes");
            stale.sync_all().expect("temporary sync");
        }

        let reopened = EpochProofFrontierV2::open(&root, authority).expect("recover");
        assert_eq!(reopened.active_range_count().expect("range count"), 0);
        assert!(reopened
            .chunks
            .read_file_bounded(
                &chunk_file_name(record.chunk_ordinal),
                FRONTIER_CHUNK_MAX_BYTES_V2 as u64
            )
            .is_err());
        assert!(reopened
            .nodes
            .read_file_bounded(
                &node_file_name(node.node_digest),
                FRONTIER_NODE_MAX_BYTES_V2 as u64
            )
            .is_err());
    }

    #[test]
    fn test_progress_uses_canonical_ordinals() {
        let temp = tempfile::tempdir().expect("frontier root");
        let authority = authority();
        let frontier =
            EpochProofFrontierV2::open(temp.path().join("frontier"), authority).expect("frontier");
        let empty = frontier.progress().expect("empty progress");
        assert_eq!(empty.verified_chunk_count(), 0);
        assert_eq!(empty.total_chunk_count(), 2);
        assert_eq!(empty.active_range_count(), 0);
        assert_eq!(empty.next_missing_chunk(), Some(0));
        assert_eq!(
            frontier
                .missing_chunk_ordinals()
                .expect("empty missing chunks"),
            vec![0, 1]
        );
        assert!(!empty.all_chunks_verified());

        let (orphan_record, _) = chunk_record(authority, 0, 31);
        write_once(
            &frontier.chunks,
            &chunk_file_name(orphan_record.chunk_ordinal),
            &orphan_record.encode(),
        )
        .expect("write unjournaled chunk");
        assert!(matches!(
            frontier.progress(),
            Err(CheckpointError::Canonical)
        ));
        frontier
            .chunks
            .remove_file(&chunk_file_name(orphan_record.chunk_ordinal))
            .expect("remove unjournaled chunk");

        let (right_record, right) = chunk_record(authority, 1, 33);
        commit_chunk(&frontier, &right_record, &right, 0);
        let out_of_order = frontier.progress().expect("out-of-order progress");
        assert_eq!(out_of_order.verified_chunk_count(), 1);
        assert_eq!(out_of_order.next_missing_chunk(), Some(0));
        assert_eq!(
            frontier
                .missing_chunk_ordinals()
                .expect("out-of-order missing chunks"),
            vec![0]
        );

        let (left_record, left) = chunk_record(authority, 0, 32);
        commit_chunk(&frontier, &left_record, &left, 1);
        let complete = frontier.progress().expect("complete progress");
        assert_eq!(complete.verified_chunk_count(), 2);
        assert_eq!(complete.total_chunk_count(), 2);
        assert_eq!(complete.active_range_count(), 2);
        assert_eq!(complete.next_missing_chunk(), None);
        assert!(frontier
            .missing_chunk_ordinals()
            .expect("complete missing chunks")
            .is_empty());
        assert!(complete.all_chunks_verified());
    }

    #[test]
    fn test_recovery_rejects_substitution() {
        let temp = tempfile::tempdir().expect("frontier root");
        let root = temp.path().join("frontier");
        let authority = authority();
        let (record, node) = chunk_record(authority, 0, 41);
        {
            let frontier = EpochProofFrontierV2::open(&root, authority).expect("open frontier");
            write_chunk(&frontier, &record, &node);
            frontier
                .append_journal(
                    FrontierJournalRecordV2::node_installed(0, &node).expect("journal record"),
                )
                .expect("journal write");
            frontier
                .chunks
                .remove_file(&chunk_file_name(record.chunk_ordinal))
                .expect("remove chunk");
            let (replacement, _) = chunk_record(authority, 0, 42);
            write_once(
                &frontier.chunks,
                &chunk_file_name(replacement.chunk_ordinal),
                &replacement.encode(),
            )
            .expect("replacement chunk");
        }

        assert!(matches!(
            EpochProofFrontierV2::open(&root, authority),
            Err(CheckpointError::Canonical)
        ));
    }

    #[test]
    fn test_recovery_retires_children() {
        let temp = tempfile::tempdir().expect("frontier root");
        let root = temp.path().join("frontier");
        let authority = authority();
        let (left_record, left) = chunk_record(authority, 0, 51);
        let (right_record, right) = chunk_record(authority, 1, 52);
        let parent_digest;
        {
            let frontier = EpochProofFrontierV2::open(&root, authority).expect("open frontier");
            commit_chunk(&frontier, &left_record, &left, 0);
            commit_chunk(&frontier, &right_record, &right, 1);
            let job = frontier
                .next_merge_job()
                .expect("merge search")
                .expect("merge job");
            let parent = parent_for_job(authority, job, 61);
            parent_digest = parent.node_digest;
            frontier.write_node(&parent).expect("parent body");
            frontier
                .append_journal(
                    FrontierJournalRecordV2::parent_installed(3, &parent).expect("parent journal"),
                )
                .expect("parent commit");
        }

        let reopened = EpochProofFrontierV2::open(&root, authority).expect("recover");
        assert_eq!(reopened.active_range_count().expect("range count"), 1);
        assert!(reopened.load_node(parent_digest).is_ok());
        assert!(reopened.load_node(left.node_digest).is_err());
        assert!(reopened.load_node(right.node_digest).is_err());
    }

    #[test]
    fn test_parent_shape_rejects() {
        let authority = authority();
        assert!(matches!(
            FrontierNodeV2::parent(
                &authority,
                VerifiedEpochParentV2 {
                    left_node_digest: digest(71),
                    right_node_digest: digest(72),
                    start_height: authority.start_height,
                    end_height: authority.start_height + 1,
                    transition_count: 2,
                    tree_level: 2,
                    range_binding_digest: digest(73),
                    proof_digest: digest(74),
                    verification_receipt_digest: digest(75),
                    proof_bytes: vec![76],
                },
            ),
            Err(CheckpointError::Canonical)
        ));
    }

    #[test]
    fn test_journal_rejects_parent_range() {
        let temp = tempfile::tempdir().expect("frontier root");
        let root = temp.path().join("frontier");
        let authority = authority();
        let (left_record, left) = chunk_record(authority, 0, 81);
        let (right_record, right) = chunk_record(authority, 1, 82);
        {
            let frontier = EpochProofFrontierV2::open(&root, authority).expect("open frontier");
            commit_chunk(&frontier, &left_record, &left, 0);
            commit_chunk(&frontier, &right_record, &right, 1);
            let job = frontier
                .next_merge_job()
                .expect("merge search")
                .expect("merge job");
            let parent = parent_for_job(authority, job, 83);
            frontier.write_node(&parent).expect("parent body");
            let mut substituted =
                FrontierJournalRecordV2::parent_installed(3, &parent).expect("parent journal");
            substituted.end_height = substituted
                .end_height
                .checked_add(1)
                .expect("bounded mutation");
            substituted.digest = sha256_256(
                "z00z.storage.checkpoint.epoch-frontier-journal.v2",
                "record",
                &[&substituted.prefix()],
            );
            frontier
                .append_journal(substituted)
                .expect("write semantic substitution");
        }

        assert!(matches!(
            EpochProofFrontierV2::open(&root, authority),
            Err(CheckpointError::Canonical)
        ));
    }

    #[test]
    fn test_sealed_statement_binds_every_frontier_root() {
        let temp = tempfile::tempdir().expect("frontier root");
        let authority = authority();
        let frontier =
            EpochProofFrontierV2::open(temp.path().join("frontier"), authority).expect("frontier");
        let (left_record, left) = chunk_record(authority, 0, 101);
        let (right_record, right) = chunk_record(authority, 1, 102);
        commit_chunk(&frontier, &left_record, &left, 0);
        commit_chunk(&frontier, &right_record, &right, 1);
        let job = frontier
            .next_merge_job()
            .expect("merge search")
            .expect("merge job");
        frontier
            .install_verified_parent(VerifiedEpochParentV2 {
                left_node_digest: job.left_node_digest(),
                right_node_digest: job.right_node_digest(),
                start_height: job.start_height(),
                end_height: job.end_height(),
                transition_count: job.transition_count().expect("parent count"),
                tree_level: job.tree_level().expect("parent level"),
                range_binding_digest: digest(103),
                proof_digest: digest(104),
                verification_receipt_digest: digest(105),
                proof_bytes: vec![106],
            })
            .expect("install parent");

        let manifest = closed_manifest(authority, &[&left_record, &right_record]);
        let roots = frontier.range_roots().expect("range roots");
        let inputs = roots
            .statement_inputs(&authority, &manifest, digest(108))
            .expect("statement inputs");
        let statement = EpochRangeStatementV2::new(inputs).expect("statement");
        frontier
            .verify_sealed_statement(&statement, &manifest)
            .expect("exact frontier statement");

        let mut substituted = inputs;
        substituted.checkpoint_artifact_root[0] ^= 1;
        let substituted =
            EpochRangeStatementV2::new(substituted).expect("well-formed substituted statement");
        assert!(matches!(
            frontier.verify_sealed_statement(&substituted, &manifest),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing
            ))
        ));

        let mut substituted = inputs;
        substituted.epoch_work_manifest_digest[0] ^= 1;
        let substituted =
            EpochRangeStatementV2::new(substituted).expect("well-formed manifest substitution");
        assert!(matches!(
            frontier.verify_sealed_statement(&substituted, &manifest),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing
            ))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn test_retirement_unlinks_symlink() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().expect("frontier root");
        let root = temp.path().join("frontier");
        let authority = authority();
        let (left_record, left) = chunk_record(authority, 0, 91);
        let (right_record, right) = chunk_record(authority, 1, 92);
        let parent_digest;
        let left_path = root.join("nodes").join(node_file_name(left.node_digest));
        {
            let frontier = EpochProofFrontierV2::open(&root, authority).expect("open frontier");
            commit_chunk(&frontier, &left_record, &left, 0);
            commit_chunk(&frontier, &right_record, &right, 1);
            let job = frontier
                .next_merge_job()
                .expect("merge search")
                .expect("merge job");
            let parent = parent_for_job(authority, job, 93);
            parent_digest = parent.node_digest;
            frontier.write_node(&parent).expect("parent body");
            frontier
                .append_journal(
                    FrontierJournalRecordV2::parent_installed(3, &parent).expect("parent journal"),
                )
                .expect("parent commit");
            frontier
                .append_journal(
                    FrontierJournalRecordV2::children_retired(4, &parent)
                        .expect("retirement journal"),
                )
                .expect("retirement commit");
            frontier
                .nodes
                .remove_file(&node_file_name(left.node_digest))
                .expect("remove left body");
            frontier.nodes.sync().expect("sync left removal");
            symlink("missing-retired-body", &left_path).expect("install retired-body symlink");
        }

        let reopened = EpochProofFrontierV2::open(&root, authority).expect("recover");
        assert_eq!(reopened.active_range_count().expect("range count"), 1);
        assert!(reopened.load_node(parent_digest).is_ok());
        assert!(std::fs::symlink_metadata(left_path).is_err());
        assert!(reopened.load_node(right.node_digest).is_err());
    }
}
