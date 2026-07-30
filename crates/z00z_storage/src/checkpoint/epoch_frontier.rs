//! Crash-durable frontier for actual-verifier-admitted Plonky3 epoch leaves.
//!
//! Proof bodies are local shadow evidence. Compact leaf bindings survive until
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

use super::{
    contract_config_v3::{ActiveCheckpointConfigIdentityV3, ConfigV3ActivationStore},
    epoch_range::{
        epoch_ordered_digest_root_v2, epoch_verified_base_statement_digest_v2, EpochCadenceClassV2,
        EpochCodecReaderV2, EpochRangeInputsV2, EpochRangeStatementV2,
        EPOCH_ARTIFACT_ROOT_DOMAIN_V2, EPOCH_CHALLENGE_ROOT_DOMAIN_V2, EPOCH_DA_ROOT_DOMAIN_V2,
        EPOCH_DELTA_ROOT_DOMAIN_V2, EPOCH_LINK_ROOT_DOMAIN_V2, EPOCH_STATEMENT_ROOT_DOMAIN_V2,
        EPOCH_VERIFIED_BASE_ROOT_DOMAIN_V2, EPOCH_WITNESS_ROOT_DOMAIN_V2,
    },
    plonky3::{
        Plonky3BaseProofV2, Plonky3BaseRangeBindingV2, Plonky3HistoryAuthorityResolverV2,
        ResolvedPlonky3HistoryAuthorityV2,
    },
    receipt::{Plonky3BaseVerificationReceiptV2, VerifiedPlonky3BaseAdmissionV2},
    recursive_reject::RecursiveCheckpointRejectReasonV2,
    version_registry::PLONKY3_PUBLISH_BYTES_V2,
};
use crate::CheckpointError;

const FRONTIER_AUTHORITY_MAGIC_V2: [u8; 8] = *b"Z00ZEFA2";
const FRONTIER_LEAF_MAGIC_V2: [u8; 8] = *b"Z00ZEFL2";
const FRONTIER_NODE_MAGIC_V2: [u8; 8] = *b"Z00ZEFN2";
const FRONTIER_JOURNAL_MAGIC_V2: [u8; 8] = *b"Z00ZEFJ2";
const FRONTIER_WIRE_VERSION_V2: u16 = 2;
// Generation 5 makes the sparse fixed-word Merkle-multiproof body codec the
// sole restart-authorized frontier path.
const FRONTIER_TREE_GENERATION_V2: u8 = 6;
const FRONTIER_AUTHORITY_DIGEST_COUNT_V2: usize = 10;
const FRONTIER_AUTHORITY_BYTES_V2: usize =
    8 + 2 + 1 + 1 + 8 * 4 + 4 + 2 + 8 * 4 + 4 + FRONTIER_AUTHORITY_DIGEST_COUNT_V2 * 32 + 32;
const FRONTIER_LEAF_DIGEST_COUNT_V2: usize = 13;
const FRONTIER_LEAF_BYTES_V2: usize =
    8 + 2 + 4 + 8 + 32 + 1 + 32 + FRONTIER_LEAF_DIGEST_COUNT_V2 * 32 + 32;
const FRONTIER_NODE_DIGEST_COUNT_V2: usize = 12;
const FRONTIER_NODE_PREFIX_BYTES_V2: usize =
    8 + 2 + 1 + 1 + 8 * 3 + 4 + 8 * 2 + 4 + 2 + FRONTIER_NODE_DIGEST_COUNT_V2 * 32 + 4;
const FRONTIER_JOURNAL_BYTES_V2: usize = 8 + 2 + 1 + 8 * 3 + 4 + 32 * 4;
const FRONTIER_MAX_JOURNAL_ENTRIES_V2: usize = 32_768;
const FRONTIER_MAX_LEAF_FILES_V2: usize = 4_096;
const FRONTIER_MAX_NODE_FILES_V2: usize = 8_192;
const FRONTIER_MAX_ROOT_ENTRIES_V2: usize = 8;
const FRONTIER_NODE_MAX_BYTES_V2: usize =
    PLONKY3_PUBLISH_BYTES_V2 + FRONTIER_NODE_PREFIX_BYTES_V2 + 32;

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
/// absent and are checked later against compact leaf records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochFrontierAuthorityV2 {
    cadence_class: EpochCadenceClassV2,
    epoch_index: u64,
    start_height: u64,
    end_height: u64,
    cadence_blocks: u64,
    leaf_count: u32,
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
        let leaf_count = u32::try_from(cadence_blocks).map_err(|_| CheckpointError::Limit)?;
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
            leaf_count,
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
    pub const fn leaf_count(&self) -> u32 {
        self.leaf_count
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
pub(super) struct EpochCanonicalLeafV2 {
    pub(super) height: u64,
    pub(super) checkpoint_id: [u8; 32],
    pub(super) predecessor: Option<[u8; 32]>,
    pub(super) checkpoint_statement_digest: [u8; 32],
    pub(super) checkpoint_statement_core_digest: [u8; 32],
    pub(super) checkpoint_link_digest: [u8; 32],
    pub(super) delta_root: [u8; 32],
    pub(super) witness_root: [u8; 32],
    pub(super) challenge_content_digest: [u8; 32],
    pub(super) da_payload_commitment: [u8; 32],
    pub(super) checkpoint_artifact_digest: [u8; 32],
    pub(super) pre_settlement_root: [u8; 32],
    pub(super) post_settlement_root: [u8; 32],
}

impl EpochCanonicalLeafV2 {
    fn from_verified_range(range: Plonky3BaseRangeBindingV2) -> Self {
        Self {
            height: range.height,
            checkpoint_id: range.checkpoint_id,
            predecessor: range.predecessor,
            checkpoint_statement_digest: range.checkpoint_statement_digest,
            checkpoint_statement_core_digest: range.checkpoint_statement_core_digest,
            checkpoint_link_digest: range.checkpoint_link_digest,
            delta_root: range.delta_root,
            witness_root: range.witness_root,
            challenge_content_digest: range.challenge_content_digest,
            da_payload_commitment: range.da_payload_commitment,
            checkpoint_artifact_digest: range.checkpoint_artifact_digest,
            pre_settlement_root: range.pre_settlement_root,
            post_settlement_root: range.post_settlement_root,
        }
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
    pub verified_base_proof_root: [u8; 32],
}

/// Bounded, non-secret restart status for one exact epoch frontier.
///
/// This contains only counters and the next canonical height requiring a
/// verified base proof. It deliberately exposes neither proof nor witness
/// bytes and is derived from the journal plus fixed leaf-record names rather
/// than directory enumeration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochFrontierProgressV2 {
    admitted_leaf_count: u32,
    total_leaf_count: u32,
    active_range_count: u32,
    next_missing_height: Option<u64>,
}

impl EpochFrontierProgressV2 {
    #[must_use]
    pub const fn admitted_leaf_count(self) -> u32 {
        self.admitted_leaf_count
    }

    #[must_use]
    pub const fn total_leaf_count(self) -> u32 {
        self.total_leaf_count
    }

    #[must_use]
    pub const fn active_range_count(self) -> u32 {
        self.active_range_count
    }

    #[must_use]
    pub const fn next_missing_height(self) -> Option<u64> {
        self.next_missing_height
    }

    #[must_use]
    pub const fn all_leaves_admitted(self) -> bool {
        self.admitted_leaf_count == self.total_leaf_count && self.next_missing_height.is_none()
    }
}

impl EpochRangeRootsV2 {
    #[allow(clippy::too_many_arguments)]
    pub fn statement_inputs(
        self,
        authority: &EpochFrontierAuthorityV2,
        epoch_close_anchor_digest: [u8; 32],
        recursive_base_proof_commitment: [u8; 32],
        nova_chain_root: Option<[u8; 32]>,
    ) -> Result<EpochRangeInputsV2, CheckpointError> {
        if epoch_close_anchor_digest == [0; 32]
            || recursive_base_proof_commitment == [0; 32]
            || nova_chain_root == Some([0; 32])
        {
            return Err(CheckpointError::Canonical);
        }
        Ok(EpochRangeInputsV2 {
            cadence_class: authority.cadence_class,
            epoch_index: authority.epoch_index,
            start_height: authority.start_height,
            end_height: authority.end_height,
            cadence_blocks: authority.cadence_blocks,
            leaf_count: authority.leaf_count,
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
            epoch_close_anchor_digest,
            start_root: self.start_root,
            end_root: self.end_root,
            statement_digest_root: self.statement_digest_root,
            checkpoint_artifact_root: self.checkpoint_artifact_root,
            checkpoint_link_root: self.checkpoint_link_root,
            delta_root: self.delta_root,
            witness_root: self.witness_root,
            challenge_content_root: self.challenge_content_root,
            da_payload_commitment: self.da_payload_commitment,
            verified_base_proof_root: self.verified_base_proof_root,
            recursive_base_proof_commitment,
            nova_chain_root,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum FrontierNodeKindV2 {
    Base = 1,
    Parent = 2,
}

#[derive(Clone)]
struct FrontierNodeV2 {
    kind: FrontierNodeKindV2,
    tree_level: u8,
    start_height: u64,
    end_height: u64,
    leaf_count: u32,
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
            .field("leaf_count", &self.leaf_count)
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
    base_pair: Option<EpochBasePairProofInputsV2>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EpochBaseLeafProofInputsV2 {
    pub(super) ordinal: u32,
    pub(super) canonical: EpochCanonicalLeafV2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EpochBasePairProofInputsV2 {
    pub(super) left: EpochBaseLeafProofInputsV2,
    pub(super) right: EpochBaseLeafProofInputsV2,
    pub(super) total: u32,
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

    pub(super) fn is_base_pair(&self) -> bool {
        self.left.kind == FrontierNodeKindV2::Base && self.right.kind == FrontierNodeKindV2::Base
    }

    pub(super) const fn base_pair_inputs(&self) -> Option<EpochBasePairProofInputsV2> {
        self.base_pair
    }

    pub(super) fn left_range(&self) -> (u64, u64, u32, u8) {
        (
            self.left.start_height,
            self.left.end_height,
            self.left.leaf_count,
            self.left.tree_level,
        )
    }

    pub(super) fn right_range(&self) -> (u64, u64, u32, u8) {
        (
            self.right.start_height,
            self.right.end_height,
            self.right.leaf_count,
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

    pub(super) fn leaf_count(&self) -> Result<u32, CheckpointError> {
        self.left
            .leaf_count
            .checked_add(self.right.leaf_count)
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
    pub(super) leaf_count: u32,
    pub(super) tree_level: u8,
    pub(super) range_binding_digest: [u8; 32],
    pub(super) proof_digest: [u8; 32],
    pub(super) verification_receipt_digest: [u8; 32],
    pub(super) proof_bytes: Vec<u8>,
}

pub(super) struct EpochFinalizationNodeV2 {
    pub(super) start_height: u64,
    pub(super) end_height: u64,
    pub(super) leaf_count: u32,
    pub(super) tree_level: u8,
    pub(super) proof_digest: [u8; 32],
    pub(super) proof_bytes: Vec<u8>,
}

pub struct EpochProofFrontierV2 {
    authority: EpochFrontierAuthorityV2,
    nodes: SecureDir,
    leaves: SecureDir,
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
        let leaves = root
            .ensure_dir("leaves")
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
            leaves,
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
            scavenge_temporary_files(&frontier.leaves, FRONTIER_MAX_LEAF_FILES_V2)?;
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

    pub fn admit_verified_base(
        &self,
        proof: &Plonky3BaseProofV2,
        receipt: &Plonky3BaseVerificationReceiptV2,
    ) -> Result<(), CheckpointError> {
        self.process_lock
            .lock_exclusive()
            .map_err(|_| CheckpointError::Storage)?;
        let result = self.admit_verified_base_locked(proof, receipt);
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    fn admit_verified_base_locked(
        &self,
        proof: &Plonky3BaseProofV2,
        receipt: &Plonky3BaseVerificationReceiptV2,
    ) -> Result<(), CheckpointError> {
        let admission = receipt.bind_epoch_admission(proof)?;
        let canonical_leaf = EpochCanonicalLeafV2::from_verified_range(admission.range);
        self.validate_base_admission(&admission, canonical_leaf)?;
        let ordinal = u32::try_from(
            canonical_leaf
                .height
                .checked_sub(self.authority.start_height)
                .ok_or(CheckpointError::Overflow)?,
        )
        .map_err(|_| CheckpointError::Limit)?;
        let leaf_record = FrontierLeafRecordV2::new(
            ordinal,
            canonical_leaf,
            admission.range.base_statement_digest,
            admission.range.proof_digest,
            admission.receipt_digest,
        )?;
        let leaf_name = leaf_file_name(ordinal);
        if self
            .leaves
            .read_file_bounded(&leaf_name, FRONTIER_LEAF_BYTES_V2 as u64)
            .is_ok()
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StepRepeated,
            ));
        }
        let node = FrontierNodeV2::base(
            &self.authority,
            canonical_leaf.height,
            leaf_record.verified_base_binding_digest()?,
            admission,
            proof.canonical_bytes().to_vec(),
        )?;
        let state = self.journal_state()?;
        if state.overlaps(node.start_height, node.end_height) {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StepRepeated,
            ));
        }
        write_once(&self.leaves, &leaf_name, &leaf_record.encode())?;
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
                && left.leaf_count == right.leaf_count
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
                    base_pair: if left.kind == FrontierNodeKindV2::Base
                        && right.kind == FrontierNodeKindV2::Base
                    {
                        Some(self.load_base_pair_inputs(left, right)?)
                    } else {
                        None
                    },
                }));
            }
        }
        Ok(None)
    }

    fn load_base_pair_inputs(
        &self,
        left: &FrontierNodeV2,
        right: &FrontierNodeV2,
    ) -> Result<EpochBasePairProofInputsV2, CheckpointError> {
        let load = |node: &FrontierNodeV2| -> Result<EpochBaseLeafProofInputsV2, CheckpointError> {
            if node.kind != FrontierNodeKindV2::Base
                || node.start_height != node.end_height
                || node.leaf_count != 1
                || node.tree_level != 0
            {
                return Err(CheckpointError::Canonical);
            }
            let ordinal = u32::try_from(
                node.start_height
                    .checked_sub(self.authority.start_height)
                    .ok_or(CheckpointError::Overflow)?,
            )
            .map_err(|_| CheckpointError::Limit)?;
            let bytes = self
                .leaves
                .read_file_bounded(&leaf_file_name(ordinal), FRONTIER_LEAF_BYTES_V2 as u64)
                .map_err(|_| CheckpointError::Storage)?;
            let record = FrontierLeafRecordV2::decode(&bytes)?;
            if record.ordinal != ordinal
                || record.canonical.height != node.start_height
                || record.base_proof_digest != node.proof_digest
                || record.verified_base_binding_digest()? != node.range_binding_digest
            {
                return Err(CheckpointError::Canonical);
            }
            Ok(EpochBaseLeafProofInputsV2 {
                ordinal,
                canonical: record.canonical,
            })
        };
        Ok(EpochBasePairProofInputsV2 {
            left: load(left)?,
            right: load(right)?,
            total: self.authority.leaf_count,
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
            || left.leaf_count != right.leaf_count
            || left.end_height.checked_add(1) != Some(right.start_height)
            || verified.start_height != left.start_height
            || verified.end_height != right.end_height
            || verified.leaf_count
                != left
                    .leaf_count
                    .checked_add(right.leaf_count)
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

    /// Compact roots become available only after every exact leaf was admitted
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
            pair[0].tree_level == pair[1].tree_level && pair[0].leaf_count == pair[1].leaf_count
        }) {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
            ));
        }
        let max_segments = usize::try_from(self.authority.leaf_count.ilog2() + 1)
            .map_err(|_| CheckpointError::Limit)?;
        if active.len() > max_segments {
            return Err(CheckpointError::Limit);
        }
        let mut leaves: Vec<FrontierLeafRecordV2> =
            Vec::with_capacity(self.authority.leaf_count as usize);
        for ordinal in 0..self.authority.leaf_count {
            let bytes = self
                .leaves
                .read_file_bounded(&leaf_file_name(ordinal), FRONTIER_LEAF_BYTES_V2 as u64)
                .map_err(|_| {
                    CheckpointError::RecursiveRejected(
                        RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
                    )
                })?;
            let leaf = FrontierLeafRecordV2::decode(&bytes)?;
            if leaf.ordinal != ordinal
                || leaf.canonical.height
                    != self
                        .authority
                        .start_height
                        .checked_add(u64::from(ordinal))
                        .ok_or(CheckpointError::Overflow)?
            {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::StepReordered,
                ));
            }
            if let Some(previous) = leaves.last() {
                if previous.canonical.post_settlement_root != leaf.canonical.pre_settlement_root
                    || leaf.canonical.predecessor != Some(previous.canonical.checkpoint_id)
                {
                    return Err(CheckpointError::RecursiveRejected(
                        RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
                    ));
                }
            }
            leaves.push(leaf);
        }
        let roots = |domain: &str,
                     select: fn(&FrontierLeafRecordV2) -> [u8; 32]|
         -> Result<[u8; 32], CheckpointError> {
            let values = leaves.iter().map(select).collect::<Vec<_>>();
            epoch_ordered_digest_root_v2(domain, &values)
        };
        let verified_base_statements = leaves
            .iter()
            .map(FrontierLeafRecordV2::verified_base_binding_digest)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(EpochRangeRootsV2 {
            start_root: leaves
                .first()
                .ok_or(CheckpointError::Canonical)?
                .canonical
                .pre_settlement_root,
            end_root: leaves
                .last()
                .ok_or(CheckpointError::Canonical)?
                .canonical
                .post_settlement_root,
            statement_digest_root: roots(EPOCH_STATEMENT_ROOT_DOMAIN_V2, |leaf| {
                leaf.canonical.checkpoint_statement_digest
            })?,
            checkpoint_artifact_root: roots(EPOCH_ARTIFACT_ROOT_DOMAIN_V2, |leaf| {
                leaf.canonical.checkpoint_artifact_digest
            })?,
            checkpoint_link_root: roots(EPOCH_LINK_ROOT_DOMAIN_V2, |leaf| {
                leaf.canonical.checkpoint_link_digest
            })?,
            delta_root: roots(EPOCH_DELTA_ROOT_DOMAIN_V2, |leaf| leaf.canonical.delta_root)?,
            witness_root: roots(EPOCH_WITNESS_ROOT_DOMAIN_V2, |leaf| {
                leaf.canonical.witness_root
            })?,
            challenge_content_root: roots(EPOCH_CHALLENGE_ROOT_DOMAIN_V2, |leaf| {
                leaf.canonical.challenge_content_digest
            })?,
            da_payload_commitment: roots(EPOCH_DA_ROOT_DOMAIN_V2, |leaf| {
                leaf.canonical.da_payload_commitment
            })?,
            verified_base_proof_root: epoch_ordered_digest_root_v2(
                EPOCH_VERIFIED_BASE_ROOT_DOMAIN_V2,
                &verified_base_statements,
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
                pair[0].tree_level == pair[1].tree_level && pair[0].leaf_count == pair[1].leaf_count
            })
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
            ));
        }
        let max_segments = usize::try_from(self.authority.leaf_count.ilog2() + 1)
            .map_err(|_| CheckpointError::Limit)?;
        if active.len() > max_segments {
            return Err(CheckpointError::Limit);
        }
        Ok(active
            .into_iter()
            .map(|node| EpochFinalizationNodeV2 {
                start_height: node.start_height,
                end_height: node.end_height,
                leaf_count: node.leaf_count,
                tree_level: node.tree_level,
                proof_digest: node.proof_digest,
                proof_bytes: node.proof_bytes,
            })
            .collect())
    }

    pub fn verify_sealed_statement(
        &self,
        statement: &EpochRangeStatementV2,
    ) -> Result<(), CheckpointError> {
        let roots = self.range_roots()?;
        let inputs = statement.inputs();
        if statement.cadence_class() != self.authority.cadence_class
            || statement.epoch_index() != self.authority.epoch_index
            || statement.start_height() != self.authority.start_height
            || statement.end_height() != self.authority.end_height
            || statement.leaf_count() != self.authority.leaf_count
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
            || inputs.start_root != roots.start_root
            || inputs.end_root != roots.end_root
            || inputs.statement_digest_root != roots.statement_digest_root
            || inputs.checkpoint_artifact_root != roots.checkpoint_artifact_root
            || inputs.checkpoint_link_root != roots.checkpoint_link_root
            || inputs.delta_root != roots.delta_root
            || inputs.witness_root != roots.witness_root
            || inputs.challenge_content_root != roots.challenge_content_root
            || inputs.da_payload_commitment != roots.da_payload_commitment
            || inputs.verified_base_proof_root != roots.verified_base_proof_root
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
            ));
        }
        Ok(())
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
    /// Leaf authority is the exact ordinal sequence `0..leaf_count`; missing
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

    fn progress_locked(&self) -> Result<EpochFrontierProgressV2, CheckpointError> {
        let state = self.journal_state()?;
        let active_range_count =
            u32::try_from(state.active_nodes.len()).map_err(|_| CheckpointError::Limit)?;
        let mut admitted_leaf_count = 0_u32;
        let mut next_missing_height = None;
        for ordinal in 0..self.authority.leaf_count {
            let height = self
                .authority
                .start_height
                .checked_add(u64::from(ordinal))
                .ok_or(CheckpointError::Overflow)?;
            let name = leaf_file_name(ordinal);
            match self
                .leaves
                .read_file_bounded(&name, FRONTIER_LEAF_BYTES_V2 as u64)
            {
                Ok(bytes) => {
                    let leaf = FrontierLeafRecordV2::decode(&bytes)?;
                    if leaf.ordinal != ordinal
                        || leaf.canonical.height != height
                        || state.admitted_leaves.get(&height)
                            != Some(&leaf.verified_base_binding_digest()?)
                    {
                        return Err(CheckpointError::Canonical);
                    }
                    admitted_leaf_count = admitted_leaf_count
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
                Err(IoError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                    if next_missing_height.is_none() {
                        next_missing_height = Some(height);
                    }
                }
                Err(_) => return Err(CheckpointError::Storage),
            }
        }
        Ok(EpochFrontierProgressV2 {
            admitted_leaf_count,
            total_leaf_count: self.authority.leaf_count,
            active_range_count,
            next_missing_height,
        })
    }

    fn validate_base_admission(
        &self,
        admission: &VerifiedPlonky3BaseAdmissionV2,
        canonical: EpochCanonicalLeafV2,
    ) -> Result<(), CheckpointError> {
        let range = admission.range;
        if canonical.height < self.authority.start_height
            || canonical.height > self.authority.end_height
            || canonical.height != range.height
            || canonical.checkpoint_id != range.checkpoint_id
            || canonical.predecessor != range.predecessor
            || canonical.checkpoint_statement_digest != range.checkpoint_statement_digest
            || canonical.checkpoint_statement_core_digest != range.checkpoint_statement_core_digest
            || canonical.checkpoint_link_digest != range.checkpoint_link_digest
            || canonical.checkpoint_artifact_digest != range.checkpoint_artifact_digest
            || canonical.delta_root != range.delta_root
            || canonical.witness_root != range.witness_root
            || canonical.challenge_content_digest != range.challenge_content_digest
            || canonical.da_payload_commitment != range.da_payload_commitment
            || canonical.pre_settlement_root != range.pre_settlement_root
            || canonical.post_settlement_root != range.post_settlement_root
            || range.chain_context_digest != self.authority.chain_context_digest
            || range.predicate_digest != self.authority.predicate_digest
            || range.parameter_digest != self.authority.parameter_digest
            || range.security_budget_digest != self.authority.security_budget_digest
            || range.verifier_bundle_digest != self.authority.verifier_bundle_digest
            || admission.config_digest != self.authority.config_digest
            || admission.registry_digest != self.authority.registry_digest
            || admission.runtime_profile_manifest_digest
                != self.authority.runtime_profile_manifest_digest
            || admission.config_generation != self.authority.config_generation
            || admission.authority_generation != self.authority.authority_generation
            || admission.parameter_generation != self.authority.parameter_generation
            || admission.runtime_profile_generation != self.authority.runtime_profile_generation
            || [
                canonical.challenge_content_digest,
                canonical.da_payload_commitment,
                canonical.checkpoint_artifact_digest,
            ]
            .contains(&[0; 32])
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
        if (node.start_height, node.end_height, node.leaf_count) != *expected {
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
        let mut seen_leaves = BTreeSet::new();
        let mut leaves_changed = false;
        for name in self
            .leaves
            .read_dir_bounded(FRONTIER_MAX_LEAF_FILES_V2)
            .map_err(|_| CheckpointError::Storage)?
        {
            let name = name.into_string().map_err(|_| CheckpointError::Canonical)?;
            let bytes = self
                .leaves
                .read_file_bounded(&name, FRONTIER_LEAF_BYTES_V2 as u64)
                .map_err(|_| CheckpointError::Storage)?;
            let leaf = FrontierLeafRecordV2::decode(&bytes)?;
            if name != leaf_file_name(leaf.ordinal)
                || leaf.canonical.height
                    != self
                        .authority
                        .start_height
                        .checked_add(u64::from(leaf.ordinal))
                        .ok_or(CheckpointError::Overflow)?
            {
                return Err(CheckpointError::Canonical);
            }
            match state.admitted_leaves.get(&leaf.canonical.height) {
                Some(binding) if *binding == leaf.verified_base_binding_digest()? => {
                    if !seen_leaves.insert(leaf.canonical.height) {
                        return Err(CheckpointError::Canonical);
                    }
                }
                Some(_) => return Err(CheckpointError::Canonical),
                None => {
                    self.leaves
                        .remove_file(&name)
                        .map_err(|_| CheckpointError::Storage)?;
                    leaves_changed = true;
                }
            }
        }
        if seen_leaves.len() != state.admitted_leaves.len() {
            return Err(CheckpointError::Canonical);
        }
        if leaves_changed {
            self.leaves.sync().map_err(|_| CheckpointError::Storage)?;
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
struct FrontierLeafRecordV2 {
    ordinal: u32,
    canonical: EpochCanonicalLeafV2,
    base_statement_digest: [u8; 32],
    base_proof_digest: [u8; 32],
    receipt_digest: [u8; 32],
    record_digest: [u8; 32],
}

impl FrontierLeafRecordV2 {
    fn new(
        ordinal: u32,
        canonical: EpochCanonicalLeafV2,
        base_statement_digest: [u8; 32],
        base_proof_digest: [u8; 32],
        receipt_digest: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        if [
            canonical.checkpoint_id,
            canonical.checkpoint_statement_digest,
            canonical.checkpoint_statement_core_digest,
            canonical.checkpoint_link_digest,
            canonical.delta_root,
            canonical.witness_root,
            canonical.challenge_content_digest,
            canonical.da_payload_commitment,
            canonical.checkpoint_artifact_digest,
            canonical.pre_settlement_root,
            canonical.post_settlement_root,
            base_statement_digest,
            base_proof_digest,
            receipt_digest,
        ]
        .contains(&[0; 32])
            || canonical.predecessor == Some([0; 32])
        {
            return Err(CheckpointError::Canonical);
        }
        let mut record = Self {
            ordinal,
            canonical,
            base_statement_digest,
            base_proof_digest,
            receipt_digest,
            record_digest: [0; 32],
        };
        record.record_digest = sha256_256(
            "z00z.storage.checkpoint.epoch-frontier-leaf.v2",
            "record",
            &[&record.prefix()],
        );
        Ok(record)
    }

    fn prefix(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FRONTIER_LEAF_BYTES_V2 - 32);
        bytes.extend_from_slice(&FRONTIER_LEAF_MAGIC_V2);
        bytes.extend_from_slice(&FRONTIER_WIRE_VERSION_V2.to_le_bytes());
        bytes.extend_from_slice(&self.ordinal.to_le_bytes());
        bytes.extend_from_slice(&self.canonical.height.to_le_bytes());
        bytes.extend_from_slice(&self.canonical.checkpoint_id);
        bytes.push(u8::from(self.canonical.predecessor.is_some()));
        bytes.extend_from_slice(&self.canonical.predecessor.unwrap_or([0; 32]));
        for digest in [
            self.canonical.checkpoint_statement_digest,
            self.canonical.checkpoint_statement_core_digest,
            self.canonical.checkpoint_link_digest,
            self.canonical.delta_root,
            self.canonical.witness_root,
            self.canonical.challenge_content_digest,
            self.canonical.da_payload_commitment,
            self.canonical.checkpoint_artifact_digest,
            self.canonical.pre_settlement_root,
            self.canonical.post_settlement_root,
            self.base_statement_digest,
            self.base_proof_digest,
            self.receipt_digest,
        ] {
            bytes.extend_from_slice(&digest);
        }
        bytes
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = self.prefix();
        bytes.extend_from_slice(&self.record_digest);
        bytes
    }

    fn decode(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() != FRONTIER_LEAF_BYTES_V2 {
            return Err(CheckpointError::Canonical);
        }
        let mut reader = EpochCodecReaderV2::new(bytes);
        if reader.array::<8>()? != FRONTIER_LEAF_MAGIC_V2
            || reader.u16()? != FRONTIER_WIRE_VERSION_V2
        {
            return Err(CheckpointError::Canonical);
        }
        let ordinal = reader.u32()?;
        let height = reader.u64()?;
        let checkpoint_id = reader.array()?;
        let predecessor = decode_optional_digest(&mut reader)?;
        let canonical = EpochCanonicalLeafV2 {
            height,
            checkpoint_id,
            predecessor,
            checkpoint_statement_digest: reader.array()?,
            checkpoint_statement_core_digest: reader.array()?,
            checkpoint_link_digest: reader.array()?,
            delta_root: reader.array()?,
            witness_root: reader.array()?,
            challenge_content_digest: reader.array()?,
            da_payload_commitment: reader.array()?,
            checkpoint_artifact_digest: reader.array()?,
            pre_settlement_root: reader.array()?,
            post_settlement_root: reader.array()?,
        };
        let base_statement_digest = reader.array()?;
        let base_proof_digest = reader.array()?;
        let receipt_digest = reader.array()?;
        let record_digest = reader.array()?;
        if !reader.is_done() {
            return Err(CheckpointError::Canonical);
        }
        let record = Self::new(
            ordinal,
            canonical,
            base_statement_digest,
            base_proof_digest,
            receipt_digest,
        )?;
        if record.record_digest != record_digest || record.encode() != bytes {
            return Err(CheckpointError::Canonical);
        }
        Ok(record)
    }

    fn verified_base_binding_digest(&self) -> Result<[u8; 32], CheckpointError> {
        epoch_verified_base_statement_digest_v2(self.canonical.height, self.base_statement_digest)
    }
}

impl FrontierNodeV2 {
    fn base(
        authority: &EpochFrontierAuthorityV2,
        height: u64,
        range_binding_digest: [u8; 32],
        admission: VerifiedPlonky3BaseAdmissionV2,
        proof_bytes: Vec<u8>,
    ) -> Result<Self, CheckpointError> {
        let mut node = Self {
            kind: FrontierNodeKindV2::Base,
            tree_level: 0,
            start_height: height,
            end_height: height,
            leaf_count: 1,
            config_generation: admission.config_generation,
            authority_generation: admission.authority_generation,
            parameter_generation: admission.parameter_generation,
            runtime_profile_generation: admission.runtime_profile_generation,
            epoch_authority_digest: authority.digest,
            chain_context_digest: authority.chain_context_digest,
            config_digest: admission.config_digest,
            registry_digest: admission.registry_digest,
            runtime_profile_manifest_digest: admission.runtime_profile_manifest_digest,
            parameter_digest: admission.range.parameter_digest,
            security_budget_digest: admission.range.security_budget_digest,
            range_binding_digest,
            proof_digest: admission.range.proof_digest,
            verification_receipt_digest: admission.receipt_digest,
            left_dependency_digest: [0; 32],
            right_dependency_digest: [0; 32],
            proof_bytes,
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
            leaf_count: verified.leaf_count,
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
            || expected_count != Some(u64::from(self.leaf_count))
            || self.proof_bytes.is_empty()
            || self.proof_bytes.len() > PLONKY3_PUBLISH_BYTES_V2
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
            FrontierNodeKindV2::Base
                if self.tree_level != 0
                    || self.leaf_count != 1
                    || self.left_dependency_digest != [0; 32]
                    || self.right_dependency_digest != [0; 32] =>
            {
                Err(CheckpointError::Canonical)
            }
            FrontierNodeKindV2::Parent
                if self.tree_level == 0
                    || 1_u32.checked_shl(u32::from(self.tree_level)) != Some(self.leaf_count)
                    || self.left_dependency_digest == [0; 32]
                    || self.right_dependency_digest == [0; 32]
                    || self.left_dependency_digest == self.right_dependency_digest =>
            {
                Err(CheckpointError::Canonical)
            }
            FrontierNodeKindV2::Base | FrontierNodeKindV2::Parent => Ok(()),
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
    bytes.extend_from_slice(&u64::from(node.leaf_count).to_le_bytes());
    bytes.extend_from_slice(&node.leaf_count.to_le_bytes());
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
        1 => FrontierNodeKindV2::Base,
        2 => FrontierNodeKindV2::Parent,
        _ => return Err(CheckpointError::Canonical),
    };
    let tree_level = reader.u8()?;
    let start_height = reader.u64()?;
    let end_height = reader.u64()?;
    let redundant_count = reader.u64()?;
    let leaf_count = reader.u32()?;
    if redundant_count != u64::from(leaf_count) {
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
        leaf_count,
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
            if len == 0 || len > PLONKY3_PUBLISH_BYTES_V2 {
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
    leaf_count: u32,
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
            node.verification_receipt_digest,
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
            leaf_count: left
                .leaf_count
                .checked_add(right.leaf_count)
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
            leaf_count: node.leaf_count,
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
        bytes.extend_from_slice(&self.leaf_count.to_le_bytes());
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
            leaf_count: reader.u32()?,
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
    admitted_leaves: BTreeMap<u64, [u8; 32]>,
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
                    || record.start_height != record.end_height
                    || record.leaf_count != 1
                    || self
                        .admitted_leaves
                        .insert(record.start_height, record.left_digest)
                        .is_some()
                    || self
                        .active_nodes
                        .insert(
                            record.node_digest,
                            (record.start_height, record.end_height, record.leaf_count),
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
                    || (record.start_height, record.end_height, record.leaf_count)
                        != (left.0, right.1, expected_count)
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
                    || (record.start_height, record.end_height, record.leaf_count)
                        != (left.0, right.1, expected_count)
                    || self
                        .active_nodes
                        .insert(
                            record.node_digest,
                            (record.start_height, record.end_height, record.leaf_count),
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
                    || (record.start_height, record.end_height, record.leaf_count) != parent
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
            .checked_add(u64::from(node.leaf_count))
            .ok_or(CheckpointError::Overflow)?;
    }
    if expected
        != authority
            .end_height
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?
        || count != u64::from(authority.leaf_count)
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
    bytes.extend_from_slice(&authority.leaf_count.to_le_bytes());
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
        leaf_count: reader.u32()?,
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

fn decode_optional_digest(
    reader: &mut EpochCodecReaderV2<'_>,
) -> Result<Option<[u8; 32]>, CheckpointError> {
    let marker = reader.u8()?;
    let digest = reader.array()?;
    match (marker, digest) {
        (0, digest) if digest == [0; 32] => Ok(None),
        (1, digest) if digest != [0; 32] => Ok(Some(digest)),
        _ => Err(CheckpointError::Canonical),
    }
}

fn leaf_file_name(ordinal: u32) -> String {
    format!("{ordinal:010}.leaf")
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
    use crate::checkpoint::{
        plonky3::Plonky3BaseRangeBindingV2, receipt::VerifiedPlonky3BaseAdmissionV2,
    };

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
            cadence_blocks: 2,
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
            cadence_blocks: 2,
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

    #[test]
    fn test_verified_range_is_the_only_canonical_leaf_source() {
        let range = Plonky3BaseRangeBindingV2 {
            height: 17,
            checkpoint_id: digest(1),
            predecessor: Some(digest(2)),
            chain_context_digest: digest(3),
            predicate_digest: digest(4),
            base_statement_digest: digest(5),
            checkpoint_statement_digest: digest(6),
            checkpoint_statement_core_digest: digest(7),
            checkpoint_link_digest: digest(8),
            checkpoint_artifact_digest: digest(9),
            delta_root: digest(10),
            witness_root: digest(11),
            challenge_content_digest: digest(12),
            da_payload_commitment: digest(13),
            pre_settlement_root: digest(14),
            post_settlement_root: digest(15),
            event_vector_digest: digest(16),
            parameter_digest: digest(17),
            security_budget_digest: digest(18),
            verifier_bundle_digest: digest(19),
            air_binding_digest: digest(20),
            proof_digest: digest(21),
        };
        let leaf = EpochCanonicalLeafV2::from_verified_range(range);

        assert_eq!(leaf.height, range.height);
        assert_eq!(leaf.checkpoint_id, range.checkpoint_id);
        assert_eq!(leaf.predecessor, range.predecessor);
        assert_eq!(
            leaf.checkpoint_statement_digest,
            range.checkpoint_statement_digest
        );
        assert_eq!(
            leaf.checkpoint_statement_core_digest,
            range.checkpoint_statement_core_digest
        );
        assert_eq!(leaf.checkpoint_link_digest, range.checkpoint_link_digest);
        assert_eq!(
            leaf.checkpoint_artifact_digest,
            range.checkpoint_artifact_digest
        );
        assert_eq!(leaf.delta_root, range.delta_root);
        assert_eq!(leaf.witness_root, range.witness_root);
        assert_eq!(
            leaf.challenge_content_digest,
            range.challenge_content_digest
        );
        assert_eq!(leaf.da_payload_commitment, range.da_payload_commitment);
        assert_eq!(leaf.pre_settlement_root, range.pre_settlement_root);
        assert_eq!(leaf.post_settlement_root, range.post_settlement_root);
    }

    fn base_record(
        authority: EpochFrontierAuthorityV2,
        height: u64,
        mark: u8,
    ) -> (FrontierLeafRecordV2, FrontierNodeV2) {
        let canonical = EpochCanonicalLeafV2 {
            height,
            checkpoint_id: digest(mark),
            predecessor: (height > authority.start_height).then(|| digest(mark - 1)),
            checkpoint_statement_digest: digest(mark.wrapping_add(10)),
            checkpoint_statement_core_digest: digest(mark.wrapping_add(11)),
            checkpoint_link_digest: digest(mark.wrapping_add(12)),
            delta_root: digest(mark.wrapping_add(13)),
            witness_root: digest(mark.wrapping_add(14)),
            challenge_content_digest: digest(mark.wrapping_add(15)),
            da_payload_commitment: digest(mark.wrapping_add(16)),
            checkpoint_artifact_digest: digest(mark.wrapping_add(17)),
            pre_settlement_root: digest(mark.wrapping_add(18)),
            post_settlement_root: digest(mark.wrapping_add(19)),
        };
        let ordinal = u32::try_from(height - authority.start_height).expect("ordinal");
        let base_statement_digest = digest(mark.wrapping_add(20));
        let proof_digest = digest(mark.wrapping_add(21));
        let receipt_digest = digest(mark.wrapping_add(22));
        let record = FrontierLeafRecordV2::new(
            ordinal,
            canonical,
            base_statement_digest,
            proof_digest,
            receipt_digest,
        )
        .expect("leaf record");
        let admission = VerifiedPlonky3BaseAdmissionV2 {
            range: Plonky3BaseRangeBindingV2 {
                height,
                checkpoint_id: canonical.checkpoint_id,
                predecessor: canonical.predecessor,
                chain_context_digest: authority.chain_context_digest,
                predicate_digest: authority.predicate_digest,
                base_statement_digest,
                checkpoint_statement_digest: canonical.checkpoint_statement_digest,
                checkpoint_statement_core_digest: canonical.checkpoint_statement_core_digest,
                checkpoint_link_digest: canonical.checkpoint_link_digest,
                checkpoint_artifact_digest: canonical.checkpoint_artifact_digest,
                delta_root: canonical.delta_root,
                witness_root: canonical.witness_root,
                challenge_content_digest: canonical.challenge_content_digest,
                da_payload_commitment: canonical.da_payload_commitment,
                pre_settlement_root: canonical.pre_settlement_root,
                post_settlement_root: canonical.post_settlement_root,
                event_vector_digest: digest(mark.wrapping_add(23)),
                parameter_digest: authority.parameter_digest,
                security_budget_digest: authority.security_budget_digest,
                verifier_bundle_digest: authority.verifier_bundle_digest,
                air_binding_digest: digest(mark.wrapping_add(24)),
                proof_digest,
            },
            receipt_digest,
            registry_digest: authority.registry_digest,
            runtime_profile_manifest_digest: authority.runtime_profile_manifest_digest,
            config_digest: authority.config_digest,
            config_generation: authority.config_generation,
            authority_generation: authority.authority_generation,
            parameter_generation: authority.parameter_generation,
            runtime_profile_generation: authority.runtime_profile_generation,
        };
        let node = FrontierNodeV2::base(
            &authority,
            height,
            record
                .verified_base_binding_digest()
                .expect("verified base statement binding"),
            admission,
            vec![mark],
        )
        .expect("base node");
        (record, node)
    }

    fn write_base(
        frontier: &EpochProofFrontierV2,
        record: &FrontierLeafRecordV2,
        node: &FrontierNodeV2,
    ) {
        write_once(
            &frontier.leaves,
            &leaf_file_name(record.ordinal),
            &record.encode(),
        )
        .expect("leaf write");
        frontier.write_node(node).expect("node write");
    }

    fn commit_base(
        frontier: &EpochProofFrontierV2,
        record: &FrontierLeafRecordV2,
        node: &FrontierNodeV2,
        sequence: u64,
    ) {
        write_base(frontier, record, node);
        frontier
            .append_journal(
                FrontierJournalRecordV2::node_installed(sequence, node).expect("base journal"),
            )
            .expect("base commit");
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
                leaf_count: job.leaf_count().expect("parent count"),
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
        let (record, node) = base_record(authority, authority.start_height, 31);
        {
            let frontier = EpochProofFrontierV2::open(&root, authority).expect("open frontier");
            write_base(&frontier, &record, &node);
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
            .leaves
            .read_file_bounded(
                &leaf_file_name(record.ordinal),
                FRONTIER_LEAF_BYTES_V2 as u64
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
        assert_eq!(empty.admitted_leaf_count(), 0);
        assert_eq!(empty.total_leaf_count(), 2);
        assert_eq!(empty.active_range_count(), 0);
        assert_eq!(empty.next_missing_height(), Some(authority.start_height()));
        assert!(!empty.all_leaves_admitted());

        let (orphan_record, _) = base_record(authority, authority.start_height(), 31);
        write_once(
            &frontier.leaves,
            &leaf_file_name(orphan_record.ordinal),
            &orphan_record.encode(),
        )
        .expect("write unjournaled leaf");
        assert!(matches!(
            frontier.progress(),
            Err(CheckpointError::Canonical)
        ));
        frontier
            .leaves
            .remove_file(&leaf_file_name(orphan_record.ordinal))
            .expect("remove unjournaled leaf");

        let (right_record, right) = base_record(authority, authority.start_height() + 1, 33);
        commit_base(&frontier, &right_record, &right, 0);
        let out_of_order = frontier.progress().expect("out-of-order progress");
        assert_eq!(out_of_order.admitted_leaf_count(), 1);
        assert_eq!(
            out_of_order.next_missing_height(),
            Some(authority.start_height())
        );

        let (left_record, left) = base_record(authority, authority.start_height(), 32);
        commit_base(&frontier, &left_record, &left, 1);
        let complete = frontier.progress().expect("complete progress");
        assert_eq!(complete.admitted_leaf_count(), 2);
        assert_eq!(complete.total_leaf_count(), 2);
        assert_eq!(complete.active_range_count(), 2);
        assert_eq!(complete.next_missing_height(), None);
        assert!(complete.all_leaves_admitted());
    }

    #[test]
    fn test_recovery_rejects_substitution() {
        let temp = tempfile::tempdir().expect("frontier root");
        let root = temp.path().join("frontier");
        let authority = authority();
        let (record, node) = base_record(authority, authority.start_height, 41);
        {
            let frontier = EpochProofFrontierV2::open(&root, authority).expect("open frontier");
            write_base(&frontier, &record, &node);
            frontier
                .append_journal(
                    FrontierJournalRecordV2::node_installed(0, &node).expect("journal record"),
                )
                .expect("journal write");
            frontier
                .leaves
                .remove_file(&leaf_file_name(record.ordinal))
                .expect("remove leaf");
            let (replacement, _) = base_record(authority, authority.start_height, 42);
            write_once(
                &frontier.leaves,
                &leaf_file_name(replacement.ordinal),
                &replacement.encode(),
            )
            .expect("replacement leaf");
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
        let (left_record, left) = base_record(authority, authority.start_height, 51);
        let (right_record, right) = base_record(authority, authority.start_height + 1, 52);
        let parent_digest;
        {
            let frontier = EpochProofFrontierV2::open(&root, authority).expect("open frontier");
            commit_base(&frontier, &left_record, &left, 0);
            commit_base(&frontier, &right_record, &right, 1);
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
                    leaf_count: 2,
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
        let (left_record, left) = base_record(authority, authority.start_height, 81);
        let (right_record, right) = base_record(authority, authority.start_height + 1, 82);
        {
            let frontier = EpochProofFrontierV2::open(&root, authority).expect("open frontier");
            commit_base(&frontier, &left_record, &left, 0);
            commit_base(&frontier, &right_record, &right, 1);
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
        let (left_record, left) = base_record(authority, authority.start_height, 101);
        let (right_record, right) = base_record(authority, authority.start_height + 1, 102);
        commit_base(&frontier, &left_record, &left, 0);
        commit_base(&frontier, &right_record, &right, 1);
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
                leaf_count: job.leaf_count().expect("parent count"),
                tree_level: job.tree_level().expect("parent level"),
                range_binding_digest: digest(103),
                proof_digest: digest(104),
                verification_receipt_digest: digest(105),
                proof_bytes: vec![106],
            })
            .expect("install parent");

        let roots = frontier.range_roots().expect("range roots");
        let inputs = roots
            .statement_inputs(&authority, digest(107), digest(108), None)
            .expect("statement inputs");
        let statement = EpochRangeStatementV2::new(inputs).expect("statement");
        frontier
            .verify_sealed_statement(&statement)
            .expect("exact frontier statement");

        let mut substituted = inputs;
        substituted.checkpoint_artifact_root[0] ^= 1;
        let substituted =
            EpochRangeStatementV2::new(substituted).expect("well-formed substituted statement");
        assert!(matches!(
            frontier.verify_sealed_statement(&substituted),
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
        let (left_record, left) = base_record(authority, authority.start_height, 91);
        let (right_record, right) = base_record(authority, authority.start_height + 1, 92);
        let parent_digest;
        let left_path = root.join("nodes").join(node_file_name(left.node_digest));
        {
            let frontier = EpochProofFrontierV2::open(&root, authority).expect("open frontier");
            commit_base(&frontier, &left_record, &left, 0);
            commit_base(&frontier, &right_record, &right, 1);
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
