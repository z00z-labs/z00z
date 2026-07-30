//! Exact, registry-framed Plonky3 epoch range statement.
//!
//! The statement is non-authoritative shadow evidence. It commits the already
//! certified canonical range and never waits for, or embeds, provider
//! availability receipts.

use z00z_crypto::sha256_256;

use super::{
    plonky3::{Plonky3HistoryAuthorityResolverV2, ResolvedPlonky3HistoryAuthorityV2},
    recursive_reject::RecursiveCheckpointRejectReasonV2,
    version_registry::{RecursiveBoundedObjectV2, RECURSIVE_OBJECT_PREHEADER_BYTES_V2},
};
use crate::CheckpointError;

#[cfg(test)]
use super::{
    contract_config_v3::CheckpointConfigResolverV3,
    recursive_v2::RecursiveSecurityBudgetManifestV2, version_registry::CheckpointVersionRegistryV2,
};

const EPOCH_RANGE_MAGIC_V2: [u8; 8] = *b"Z00ZERG2";
const EPOCH_RANGE_WIRE_VERSION_V2: u16 = 2;
pub(super) const EPOCH_TREE_SHAPE_GENERATION_V2: u8 = 1;
const EPOCH_RANGE_DIGEST_LABEL_V2: &str = "canonical_statement";
pub(super) const ORDERED_ROOT_LABEL_V2: &str = "ordered_digest_root";
pub(super) const ORDERED_LEAF_LABEL_V2: &str = "ordered_digest_leaf";
pub(super) const ORDERED_PARENT_LABEL_V2: &str = "ordered_digest_parent";
pub(super) const EPOCH_STATEMENT_ROOT_DOMAIN_V2: &str =
    "z00z.storage.checkpoint.epoch.statement-digests.v2";
pub(super) const EPOCH_ARTIFACT_ROOT_DOMAIN_V2: &str =
    "z00z.storage.checkpoint.epoch.artifact-digests.v2";
pub(super) const EPOCH_LINK_ROOT_DOMAIN_V2: &str = "z00z.storage.checkpoint.epoch.link-digests.v2";
pub(super) const EPOCH_DELTA_ROOT_DOMAIN_V2: &str = "z00z.storage.checkpoint.epoch.delta-roots.v2";
pub(super) const EPOCH_WITNESS_ROOT_DOMAIN_V2: &str =
    "z00z.storage.checkpoint.epoch.witness-roots.v2";
pub(super) const EPOCH_CHALLENGE_ROOT_DOMAIN_V2: &str =
    "z00z.storage.checkpoint.epoch.challenge-content.v2";
pub(super) const EPOCH_DA_ROOT_DOMAIN_V2: &str = "z00z.storage.checkpoint.epoch.da-payloads.v2";
pub(super) const EPOCH_VERIFIED_BASE_ROOT_DOMAIN_V2: &str =
    "z00z.storage.checkpoint.epoch.verified-base-proofs.v2";
pub(super) const EPOCH_VERIFIED_BASE_STATEMENT_DOMAIN_V2: &str =
    "z00z.storage.checkpoint.epoch.verified-base-statement.v2";
pub(super) const EPOCH_VERIFIED_BASE_STATEMENT_LABEL_V2: &str = "actual_verified_base_statement";
const EPOCH_RANGE_DIGEST_COUNT_V2: usize = 21;
const EPOCH_RANGE_PAYLOAD_BYTES_V2: usize =
    8 + 2 + 1 + 1 + 8 * 4 + 4 * 2 + EPOCH_RANGE_DIGEST_COUNT_V2 * 32 + 1 + 32;

/// Whether an exact range is production cadence or a bounded real-proof
/// simulation. Both classes use the same statement and tree grammar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EpochCadenceClassV2 {
    Production = 1,
    BoundedSimulation = 2,
}

impl EpochCadenceClassV2 {
    fn decode(value: u8) -> Result<Self, CheckpointError> {
        match value {
            1 => Ok(Self::Production),
            2 => Ok(Self::BoundedSimulation),
            _ => Err(CheckpointError::Canonical),
        }
    }
}

/// Complete constructor input. A struct keeps the sole constructor readable
/// without introducing a second partially bound statement path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochRangeInputsV2 {
    pub cadence_class: EpochCadenceClassV2,
    pub epoch_index: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub cadence_blocks: u64,
    pub leaf_count: u32,
    pub parameter_generation: u32,
    pub chain_context_digest: [u8; 32],
    pub predicate_digest: [u8; 32],
    pub parameter_digest: [u8; 32],
    pub verifier_bundle_digest: [u8; 32],
    pub security_budget_digest: [u8; 32],
    pub config_digest: [u8; 32],
    pub registry_digest: [u8; 32],
    pub runtime_profile_manifest_digest: [u8; 32],
    /// Generation-bound open-epoch authority persisted by the durable
    /// frontier. This prevents a valid range proof from being replayed across
    /// another epoch/configuration frontier.
    pub frontier_authority_digest: [u8; 32],
    pub epoch_close_anchor_digest: [u8; 32],
    pub start_root: [u8; 32],
    pub end_root: [u8; 32],
    pub statement_digest_root: [u8; 32],
    pub checkpoint_artifact_root: [u8; 32],
    pub checkpoint_link_root: [u8; 32],
    pub delta_root: [u8; 32],
    pub witness_root: [u8; 32],
    pub challenge_content_root: [u8; 32],
    pub da_payload_commitment: [u8; 32],
    /// Ordered root of the actual-verifier-admitted Plonky3 base-proof
    /// statements and proof digests. This is distinct from the canonical
    /// checkpoint-statement root.
    pub verified_base_proof_root: [u8; 32],
    /// Canonical eight-u32 encoding of the ordered Poseidon commitment exposed
    /// by the final actual-verifier-accepted epoch recursion proof.
    pub recursive_base_proof_commitment: [u8; 32],
    pub nova_chain_root: Option<[u8; 32]>,
}

/// Public inputs for one exact finalized-block epoch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpochRangeStatementV2 {
    inputs: EpochRangeInputsV2,
    digest: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl EpochRangeStatementV2 {
    pub fn new(inputs: EpochRangeInputsV2) -> Result<Self, CheckpointError> {
        let authority = Plonky3HistoryAuthorityResolverV2::resolve_active()?;
        Self::new_with_authority(inputs, &authority)
    }

    fn new_with_authority(
        inputs: EpochRangeInputsV2,
        authority: &ResolvedPlonky3HistoryAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        validate_inputs(&inputs, authority)?;
        let registry = authority.registry();
        let payload = encode_payload(&inputs);
        if payload.len() != EPOCH_RANGE_PAYLOAD_BYTES_V2 {
            return Err(CheckpointError::Invariant);
        }
        let preheader = registry
            .encode_preheader(RecursiveBoundedObjectV2::EpochRangeStatement, payload.len())?;
        let mut canonical_bytes =
            Vec::with_capacity(RECURSIVE_OBJECT_PREHEADER_BYTES_V2 + payload.len());
        canonical_bytes.extend_from_slice(&preheader);
        canonical_bytes.extend_from_slice(&payload);
        let domain = registry
            .row(RecursiveBoundedObjectV2::EpochRangeStatement)?
            .cryptographic_domain;
        let digest = sha256_256(domain, EPOCH_RANGE_DIGEST_LABEL_V2, &[&canonical_bytes]);
        Ok(Self {
            inputs,
            digest,
            canonical_bytes,
        })
    }

    /// Fixed-preheader dispatch runs before any payload allocation or nested
    /// decode. Failure never selects another epoch statement decoder.
    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, CheckpointError> {
        let authority = Plonky3HistoryAuthorityResolverV2::resolve_active()?;
        Self::decode_canonical_with(bytes, &authority, |_| Ok(()))
    }

    /// Decode against one exact trusted historical authority generation.
    ///
    /// The caller must resolve `authority` from an immutable ConfigV3
    /// generation; proof bytes never select it and no current-config fallback
    /// is permitted.
    pub fn decode_canonical_with_authority(
        bytes: &[u8],
        authority: &ResolvedPlonky3HistoryAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        Self::decode_canonical_with(bytes, authority, |_| Ok(()))
    }

    pub(crate) fn decode_manifest_canonical_with_authority(
        bytes: &[u8],
        authority: &ResolvedPlonky3HistoryAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        Self::decode_canonical_with(bytes, authority, require_manifest_roots)
    }

    fn decode_canonical_with(
        bytes: &[u8],
        authority: &ResolvedPlonky3HistoryAuthorityV2,
        preflight: impl FnOnce(&EpochRangeInputsV2) -> Result<(), CheckpointError>,
    ) -> Result<Self, CheckpointError> {
        let registry = authority.registry();
        let preheader =
            registry.validate_preheader(bytes, RecursiveBoundedObjectV2::EpochRangeStatement)?;
        if preheader.header_len != RECURSIVE_OBJECT_PREHEADER_BYTES_V2
            || preheader.declared_len != EPOCH_RANGE_PAYLOAD_BYTES_V2 as u64
        {
            return Err(CheckpointError::Canonical);
        }
        let payload = bytes
            .get(preheader.header_len..)
            .ok_or(CheckpointError::Canonical)?;
        let inputs = decode_payload(payload)?;
        preflight(&inputs)?;
        let statement = Self::new_with_authority(inputs, authority)?;
        if statement.canonical_bytes != bytes {
            return Err(CheckpointError::Canonical);
        }
        Ok(statement)
    }

    #[must_use]
    pub const fn inputs(&self) -> EpochRangeInputsV2 {
        self.inputs
    }

    #[must_use]
    pub const fn cadence_class(&self) -> EpochCadenceClassV2 {
        self.inputs.cadence_class
    }

    #[must_use]
    pub const fn epoch_index(&self) -> u64 {
        self.inputs.epoch_index
    }

    #[must_use]
    pub const fn start_height(&self) -> u64 {
        self.inputs.start_height
    }

    #[must_use]
    pub const fn end_height(&self) -> u64 {
        self.inputs.end_height
    }

    #[must_use]
    pub const fn cadence_blocks(&self) -> u64 {
        self.inputs.cadence_blocks
    }

    #[must_use]
    pub const fn leaf_count(&self) -> u32 {
        self.inputs.leaf_count
    }

    #[must_use]
    pub const fn start_root(&self) -> [u8; 32] {
        self.inputs.start_root
    }

    #[must_use]
    pub const fn end_root(&self) -> [u8; 32] {
        self.inputs.end_root
    }

    #[must_use]
    pub const fn statement_digest_root(&self) -> [u8; 32] {
        self.inputs.statement_digest_root
    }

    #[must_use]
    pub const fn checkpoint_link_root(&self) -> [u8; 32] {
        self.inputs.checkpoint_link_root
    }

    #[must_use]
    pub const fn verified_base_proof_root(&self) -> [u8; 32] {
        self.inputs.verified_base_proof_root
    }

    #[must_use]
    pub const fn frontier_authority_digest(&self) -> [u8; 32] {
        self.inputs.frontier_authority_digest
    }

    #[must_use]
    pub const fn recursive_base_proof_commitment(&self) -> [u8; 32] {
        self.inputs.recursive_base_proof_commitment
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    #[must_use]
    pub const fn is_production_cadence(&self) -> bool {
        matches!(self.inputs.cadence_class, EpochCadenceClassV2::Production)
    }
}

fn require_manifest_roots(inputs: &EpochRangeInputsV2) -> Result<(), CheckpointError> {
    if [
        inputs.chain_context_digest,
        inputs.predicate_digest,
        inputs.parameter_digest,
        inputs.verifier_bundle_digest,
        inputs.security_budget_digest,
        inputs.config_digest,
        inputs.registry_digest,
        inputs.runtime_profile_manifest_digest,
        inputs.frontier_authority_digest,
        inputs.epoch_close_anchor_digest,
        inputs.start_root,
        inputs.end_root,
        inputs.statement_digest_root,
        inputs.checkpoint_artifact_root,
        inputs.checkpoint_link_root,
        inputs.delta_root,
        inputs.witness_root,
        inputs.challenge_content_root,
        inputs.da_payload_commitment,
        inputs.verified_base_proof_root,
        inputs.recursive_base_proof_commitment,
    ]
    .contains(&[0; 32])
        || inputs.nova_chain_root == Some([0; 32])
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::EpochManifestIncomplete,
        ));
    }
    Ok(())
}

fn validate_inputs(
    inputs: &EpochRangeInputsV2,
    authority: &ResolvedPlonky3HistoryAuthorityV2,
) -> Result<(), CheckpointError> {
    let registry = authority.registry();
    let security = authority.security();
    let identity = authority.identity();
    let production_cadence = authority.cadence_blocks();
    if !authority.epoch_enabled()
        || !authority.has_transition_range_proof()
        || !authority.has_independent_transition_proof()
        || inputs.cadence_blocks == 0
        || inputs.leaf_count == 0
        || u64::from(inputs.leaf_count) != inputs.cadence_blocks
        || inputs.start_height == 0
        || inputs.end_height < inputs.start_height
        || inputs
            .end_height
            .checked_sub(inputs.start_height)
            .and_then(|span| span.checked_add(1))
            != Some(inputs.cadence_blocks)
        || inputs
            .epoch_index
            .checked_mul(inputs.cadence_blocks)
            .and_then(|height| height.checked_add(1))
            != Some(inputs.start_height)
        || inputs
            .start_height
            .checked_add(inputs.cadence_blocks)
            .and_then(|height| height.checked_sub(1))
            != Some(inputs.end_height)
        || inputs.config_digest != identity.config_digest
        || inputs.registry_digest != registry.digest()
        || inputs.registry_digest != identity.registry_digest
        || inputs.runtime_profile_manifest_digest != identity.runtime_profile_manifest_digest
        || inputs.security_budget_digest != security.digest()
        || inputs.security_budget_digest != identity.security_budget_digest
        || inputs.parameter_digest != identity.verifier_parameter_digest
        || inputs.parameter_generation != identity.parameter_generation
        || inputs.verifier_bundle_digest != identity.verifier_bundle_digest
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
        ));
    }
    match inputs.cadence_class {
        EpochCadenceClassV2::Production if inputs.cadence_blocks != production_cadence => {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::HybridCadenceMismatch,
            ));
        }
        EpochCadenceClassV2::BoundedSimulation if inputs.cadence_blocks >= production_cadence => {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::HybridCadenceMismatch,
            ));
        }
        EpochCadenceClassV2::Production | EpochCadenceClassV2::BoundedSimulation => {}
    }
    let required = [
        inputs.chain_context_digest,
        inputs.predicate_digest,
        inputs.parameter_digest,
        inputs.verifier_bundle_digest,
        inputs.security_budget_digest,
        inputs.config_digest,
        inputs.registry_digest,
        inputs.runtime_profile_manifest_digest,
        inputs.frontier_authority_digest,
        inputs.epoch_close_anchor_digest,
        inputs.start_root,
        inputs.end_root,
        inputs.statement_digest_root,
        inputs.checkpoint_artifact_root,
        inputs.checkpoint_link_root,
        inputs.delta_root,
        inputs.witness_root,
        inputs.challenge_content_root,
        inputs.da_payload_commitment,
        inputs.verified_base_proof_root,
        inputs.recursive_base_proof_commitment,
    ];
    if required.contains(&[0; 32]) || inputs.nova_chain_root == Some([0; 32]) {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
        ));
    }
    Ok(())
}

fn encode_payload(inputs: &EpochRangeInputsV2) -> Vec<u8> {
    let mut payload = Vec::with_capacity(EPOCH_RANGE_PAYLOAD_BYTES_V2);
    payload.extend_from_slice(&EPOCH_RANGE_MAGIC_V2);
    payload.extend_from_slice(&EPOCH_RANGE_WIRE_VERSION_V2.to_le_bytes());
    payload.push(inputs.cadence_class as u8);
    payload.push(EPOCH_TREE_SHAPE_GENERATION_V2);
    payload.extend_from_slice(&inputs.epoch_index.to_le_bytes());
    payload.extend_from_slice(&inputs.start_height.to_le_bytes());
    payload.extend_from_slice(&inputs.end_height.to_le_bytes());
    payload.extend_from_slice(&inputs.cadence_blocks.to_le_bytes());
    payload.extend_from_slice(&inputs.leaf_count.to_le_bytes());
    payload.extend_from_slice(&inputs.parameter_generation.to_le_bytes());
    for digest in [
        inputs.chain_context_digest,
        inputs.predicate_digest,
        inputs.parameter_digest,
        inputs.verifier_bundle_digest,
        inputs.security_budget_digest,
        inputs.config_digest,
        inputs.registry_digest,
        inputs.runtime_profile_manifest_digest,
        inputs.frontier_authority_digest,
        inputs.epoch_close_anchor_digest,
        inputs.start_root,
        inputs.end_root,
        inputs.statement_digest_root,
        inputs.checkpoint_artifact_root,
        inputs.checkpoint_link_root,
        inputs.delta_root,
        inputs.witness_root,
        inputs.challenge_content_root,
        inputs.da_payload_commitment,
        inputs.verified_base_proof_root,
        inputs.recursive_base_proof_commitment,
    ] {
        payload.extend_from_slice(&digest);
    }
    payload.push(u8::from(inputs.nova_chain_root.is_some()));
    payload.extend_from_slice(&inputs.nova_chain_root.unwrap_or([0; 32]));
    payload
}

fn decode_payload(payload: &[u8]) -> Result<EpochRangeInputsV2, CheckpointError> {
    if payload.len() != EPOCH_RANGE_PAYLOAD_BYTES_V2 {
        return Err(CheckpointError::Canonical);
    }
    let mut reader = EpochCodecReaderV2::new(payload);
    if reader.array::<8>()? != EPOCH_RANGE_MAGIC_V2 || reader.u16()? != EPOCH_RANGE_WIRE_VERSION_V2
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::UnsupportedVersion,
        ));
    }
    let cadence_class = EpochCadenceClassV2::decode(reader.u8()?)?;
    if reader.u8()? != EPOCH_TREE_SHAPE_GENERATION_V2 {
        return Err(CheckpointError::Canonical);
    }
    let epoch_index = reader.u64()?;
    let start_height = reader.u64()?;
    let end_height = reader.u64()?;
    let cadence_blocks = reader.u64()?;
    let leaf_count = reader.u32()?;
    let parameter_generation = reader.u32()?;
    let inputs = EpochRangeInputsV2 {
        cadence_class,
        epoch_index,
        start_height,
        end_height,
        cadence_blocks,
        leaf_count,
        parameter_generation,
        chain_context_digest: reader.array()?,
        predicate_digest: reader.array()?,
        parameter_digest: reader.array()?,
        verifier_bundle_digest: reader.array()?,
        security_budget_digest: reader.array()?,
        config_digest: reader.array()?,
        registry_digest: reader.array()?,
        runtime_profile_manifest_digest: reader.array()?,
        frontier_authority_digest: reader.array()?,
        epoch_close_anchor_digest: reader.array()?,
        start_root: reader.array()?,
        end_root: reader.array()?,
        statement_digest_root: reader.array()?,
        checkpoint_artifact_root: reader.array()?,
        checkpoint_link_root: reader.array()?,
        delta_root: reader.array()?,
        witness_root: reader.array()?,
        challenge_content_root: reader.array()?,
        da_payload_commitment: reader.array()?,
        verified_base_proof_root: reader.array()?,
        recursive_base_proof_commitment: reader.array()?,
        nova_chain_root: decode_optional_digest(&mut reader)?,
    };
    if !reader.is_done() {
        return Err(CheckpointError::Canonical);
    }
    Ok(inputs)
}

fn decode_optional_digest(
    reader: &mut EpochCodecReaderV2<'_>,
) -> Result<Option<[u8; 32]>, CheckpointError> {
    let present = reader.u8()?;
    let digest = reader.array()?;
    match (present, digest) {
        (0, digest) if digest == [0; 32] => Ok(None),
        (1, digest) if digest != [0; 32] => Ok(Some(digest)),
        _ => Err(CheckpointError::Canonical),
    }
}

/// Versioned, order- and cardinality-binding digest tree used by all epoch
/// manifest roots. It never pads or duplicates the final leaf.
pub fn epoch_ordered_digest_root_v2(
    root_domain: &str,
    digests: &[[u8; 32]],
) -> Result<[u8; 32], CheckpointError> {
    if root_domain.is_empty() || digests.is_empty() || digests.contains(&[0; 32]) {
        return Err(CheckpointError::Canonical);
    }
    let total = u64::try_from(digests.len()).map_err(|_| CheckpointError::Limit)?;
    let mut slots: Vec<Option<OrderedDigestNodeV2>> = Vec::new();
    for (ordinal, digest) in digests.iter().copied().enumerate() {
        let ordinal = u64::try_from(ordinal).map_err(|_| CheckpointError::Limit)?;
        let mut node = OrderedDigestNodeV2 {
            start: ordinal,
            count: 1,
            digest: epoch_ordered_digest_leaf_v2(root_domain, ordinal, total, digest)?,
        };
        let mut level = 0_usize;
        loop {
            if slots.len() <= level {
                slots.resize(level + 1, None);
            }
            let Some(left) = slots[level].take() else {
                slots[level] = Some(node);
                break;
            };
            node = merge_ordered_digest_nodes(root_domain, total, left, node)?;
            level = level.checked_add(1).ok_or(CheckpointError::Overflow)?;
        }
    }
    let mut root = None;
    for node in slots.into_iter().rev().flatten() {
        root = Some(match root {
            None => node,
            Some(left) => merge_ordered_digest_nodes(root_domain, total, left, node)?,
        });
    }
    let root = root.ok_or(CheckpointError::Canonical)?;
    if root.start != 0 || root.count != total {
        return Err(CheckpointError::Invariant);
    }
    epoch_ordered_digest_root_node_v2(root_domain, total, root.digest)
}

pub(super) fn epoch_ordered_digest_leaf_v2(
    domain: &str,
    ordinal: u64,
    total: u64,
    digest: [u8; 32],
) -> Result<[u8; 32], CheckpointError> {
    if domain.is_empty() || total == 0 || ordinal >= total || digest == [0; 32] {
        return Err(CheckpointError::Canonical);
    }
    Ok(sha256_256(
        domain,
        ORDERED_LEAF_LABEL_V2,
        &[
            &EPOCH_TREE_SHAPE_GENERATION_V2.to_le_bytes(),
            &ordinal.to_le_bytes(),
            &total.to_le_bytes(),
            &digest,
        ],
    ))
}

pub(super) fn epoch_ordered_digest_root_node_v2(
    domain: &str,
    total: u64,
    node_digest: [u8; 32],
) -> Result<[u8; 32], CheckpointError> {
    if domain.is_empty() || total == 0 || node_digest == [0; 32] {
        return Err(CheckpointError::Canonical);
    }
    Ok(sha256_256(
        domain,
        ORDERED_ROOT_LABEL_V2,
        &[
            &EPOCH_TREE_SHAPE_GENERATION_V2.to_le_bytes(),
            &total.to_le_bytes(),
            &node_digest,
        ],
    ))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn epoch_ordered_digest_parent_v2(
    domain: &str,
    total: u64,
    left_start: u64,
    left_count: u64,
    right_start: u64,
    right_count: u64,
    left_digest: [u8; 32],
    right_digest: [u8; 32],
) -> Result<[u8; 32], CheckpointError> {
    let left = OrderedDigestNodeV2 {
        start: left_start,
        count: left_count,
        digest: left_digest,
    };
    let right = OrderedDigestNodeV2 {
        start: right_start,
        count: right_count,
        digest: right_digest,
    };
    if domain.is_empty()
        || total == 0
        || left_count == 0
        || right_count == 0
        || left_digest == [0; 32]
        || right_digest == [0; 32]
    {
        return Err(CheckpointError::Canonical);
    }
    let parent = merge_ordered_digest_nodes(domain, total, left, right)?;
    if parent
        .start
        .checked_add(parent.count)
        .is_none_or(|end| end > total)
    {
        return Err(CheckpointError::Canonical);
    }
    Ok(parent.digest)
}

pub(super) fn epoch_verified_base_statement_digest_v2(
    height: u64,
    statement_digest: [u8; 32],
) -> Result<[u8; 32], CheckpointError> {
    if height == 0 || statement_digest == [0; 32] {
        return Err(CheckpointError::Canonical);
    }
    Ok(sha256_256(
        EPOCH_VERIFIED_BASE_STATEMENT_DOMAIN_V2,
        EPOCH_VERIFIED_BASE_STATEMENT_LABEL_V2,
        &[&height.to_le_bytes(), &statement_digest],
    ))
}

#[derive(Clone, Copy)]
struct OrderedDigestNodeV2 {
    start: u64,
    count: u64,
    digest: [u8; 32],
}

fn merge_ordered_digest_nodes(
    domain: &str,
    total: u64,
    left: OrderedDigestNodeV2,
    right: OrderedDigestNodeV2,
) -> Result<OrderedDigestNodeV2, CheckpointError> {
    if left.start.checked_add(left.count) != Some(right.start) {
        return Err(CheckpointError::Canonical);
    }
    let count = left
        .count
        .checked_add(right.count)
        .ok_or(CheckpointError::Overflow)?;
    Ok(OrderedDigestNodeV2 {
        start: left.start,
        count,
        digest: sha256_256(
            domain,
            ORDERED_PARENT_LABEL_V2,
            &[
                &EPOCH_TREE_SHAPE_GENERATION_V2.to_le_bytes(),
                &left.start.to_le_bytes(),
                &left.count.to_le_bytes(),
                &right.start.to_le_bytes(),
                &right.count.to_le_bytes(),
                &total.to_le_bytes(),
                &left.digest,
                &right.digest,
            ],
        ),
    })
}

pub(super) struct EpochCodecReaderV2<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> EpochCodecReaderV2<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    pub(super) fn take(&mut self, len: usize) -> Result<&'a [u8], CheckpointError> {
        let end = self
            .cursor
            .checked_add(len)
            .ok_or(CheckpointError::Overflow)?;
        let value = self
            .bytes
            .get(self.cursor..end)
            .ok_or(CheckpointError::Canonical)?;
        self.cursor = end;
        Ok(value)
    }

    pub(super) fn array<const N: usize>(&mut self) -> Result<[u8; N], CheckpointError> {
        self.take(N)?
            .try_into()
            .map_err(|_| CheckpointError::Canonical)
    }

    pub(super) fn u8(&mut self) -> Result<u8, CheckpointError> {
        self.take(1)?
            .first()
            .copied()
            .ok_or(CheckpointError::Canonical)
    }

    pub(super) fn u16(&mut self) -> Result<u16, CheckpointError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    pub(super) fn u32(&mut self) -> Result<u32, CheckpointError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    pub(super) fn u64(&mut self) -> Result<u64, CheckpointError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    pub(super) const fn is_done(&self) -> bool {
        self.cursor == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(mark: u8) -> [u8; 32] {
        [mark; 32]
    }

    fn manifest_inputs() -> EpochRangeInputsV2 {
        let active = CheckpointConfigResolverV3::resolve_active().expect("active config");
        let identity = active.identity();
        let cadence = active.config().branches.plonky3_epoch.cadence_blocks;
        let security = RecursiveSecurityBudgetManifestV2::authority_pinned().expect("security");
        let history_authority =
            Plonky3HistoryAuthorityResolverV2::resolve_active().expect("history authority");
        let history_identity = history_authority.identity();
        EpochRangeInputsV2 {
            cadence_class: EpochCadenceClassV2::Production,
            epoch_index: 0,
            start_height: 1,
            end_height: cadence,
            cadence_blocks: cadence,
            leaf_count: u32::try_from(cadence).expect("cadence fits"),
            parameter_generation: identity.parameter_generation,
            chain_context_digest: digest(1),
            predicate_digest: digest(2),
            parameter_digest: history_identity.verifier_parameter_digest,
            verifier_bundle_digest: history_identity.verifier_bundle_digest,
            security_budget_digest: security.digest(),
            config_digest: identity.config_digest,
            registry_digest: CheckpointVersionRegistryV2::authority_pinned()
                .expect("registry")
                .digest(),
            runtime_profile_manifest_digest: identity.runtime_profile_manifest_digest,
            frontier_authority_digest: digest(5),
            epoch_close_anchor_digest: digest(6),
            start_root: digest(7),
            end_root: digest(8),
            statement_digest_root: digest(9),
            checkpoint_artifact_root: digest(18),
            checkpoint_link_root: digest(10),
            delta_root: digest(11),
            witness_root: digest(12),
            challenge_content_root: digest(13),
            da_payload_commitment: digest(14),
            verified_base_proof_root: digest(15),
            recursive_base_proof_commitment: digest(16),
            nova_chain_root: Some(digest(17)),
        }
    }

    #[test]
    fn test_manifest_roots_reject() {
        const DIGEST_OFFSET: usize =
            RECURSIVE_OBJECT_PREHEADER_BYTES_V2 + 8 + 2 + 1 + 1 + 8 * 4 + 4 * 2;
        let authority =
            Plonky3HistoryAuthorityResolverV2::resolve_active().expect("history authority");
        for digest_index in [9_usize, 12, 13, 14, 17, 18] {
            let statement = EpochRangeStatementV2::new(manifest_inputs()).expect("epoch statement");
            let mut bytes = statement.canonical_bytes().to_vec();
            let start = DIGEST_OFFSET + digest_index * 32;
            bytes[start..start + 32].fill(0);
            assert!(matches!(
                EpochRangeStatementV2::decode_manifest_canonical_with_authority(&bytes, &authority),
                Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::EpochManifestIncomplete
                ))
            ));
        }
    }
}
