//! Separate rolling Plonky3 history statement and explicit generation bridge.
//!
//! These objects bind theorem statements and security accounting.
//! Cryptographic child verification remains exclusively in
//! `checkpoint::plonky3`; serialized proof-byte hashes are not theorem inputs.

use z00z_crypto::sha256_256;

use super::{
    epoch_range::EpochCodecReaderV2,
    plonky3::{Plonky3HistoryAuthorityResolverV2, ResolvedPlonky3HistoryAuthorityV2},
    recursive_reject::RecursiveCheckpointRejectReasonV2,
    version_registry::{RecursiveBoundedObjectV2, RECURSIVE_OBJECT_PREHEADER_BYTES_V2},
};
use crate::CheckpointError;

const HISTORY_STATEMENT_MAGIC_V2: [u8; 8] = *b"Z00ZHST2";
const HISTORY_ROTATION_MAGIC_V2: [u8; 8] = *b"Z00ZHBR2";
const HISTORY_WIRE_VERSION_V2: u16 = 2;
const HISTORY_STATEMENT_DIGEST_LABEL_V2: &str = "canonical_statement";
const HISTORY_ROTATION_DIGEST_LABEL_V2: &str = "canonical_bridge";
const HISTORY_STATEMENT_DIGEST_COUNT_V2: usize = 18;
const HISTORY_STATEMENT_PAYLOAD_BYTES_V2: usize =
    8 + 2 + 1 + 8 * 11 + 4 + 2 * 6 + HISTORY_STATEMENT_DIGEST_COUNT_V2 * 32 + 1 + 32;
const HISTORY_ROTATION_DIGEST_COUNT_V2: usize = 21;
const HISTORY_ROTATION_PAYLOAD_BYTES_V2: usize =
    8 + 2 + 8 * 3 + 4 * 2 + 2 * 3 + HISTORY_ROTATION_DIGEST_COUNT_V2 * 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum HistoryBranchV2 {
    Base = 1,
    Successor = 2,
}

impl HistoryBranchV2 {
    fn decode(value: u8) -> Result<Self, CheckpointError> {
        match value {
            1 => Ok(Self::Base),
            2 => Ok(Self::Successor),
            _ => Err(CheckpointError::Canonical),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryAuthorityIdentityV2 {
    pub config_generation: u64,
    pub authority_generation: u64,
    pub parameter_generation: u32,
    pub activation_height: u64,
    pub rollback_floor: u64,
    pub runtime_profile_generation: u16,
    pub config_digest: [u8; 32],
    pub registry_digest: [u8; 32],
    pub runtime_profile_manifest_digest: [u8; 32],
    pub authority_bundle_digest: [u8; 32],
    pub verifier_bundle_digest: [u8; 32],
    pub verifier_parameter_digest: [u8; 32],
    pub security_budget_digest: [u8; 32],
}

impl HistoryAuthorityIdentityV2 {
    #[must_use]
    pub fn digest(self) -> [u8; 32] {
        sha256_256(
            "z00z.storage.checkpoint.history-authority-identity.v2",
            "identity",
            &[
                &self.config_generation.to_le_bytes(),
                &self.authority_generation.to_le_bytes(),
                &self.parameter_generation.to_le_bytes(),
                &self.activation_height.to_le_bytes(),
                &self.rollback_floor.to_le_bytes(),
                &self.runtime_profile_generation.to_le_bytes(),
                &self.config_digest,
                &self.registry_digest,
                &self.runtime_profile_manifest_digest,
                &self.authority_bundle_digest,
                &self.verifier_bundle_digest,
                &self.verifier_parameter_digest,
                &self.security_budget_digest,
            ],
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryAccumulatorInputsV2 {
    pub branch: HistoryBranchV2,
    pub first_epoch: u64,
    pub last_epoch: u64,
    pub first_height: u64,
    pub last_height: u64,
    pub cadence_blocks: u64,
    pub history_length: u64,
    pub accepted_epoch_count: u64,
    pub config_generation: u64,
    pub authority_generation: u64,
    pub activation_height: u64,
    pub rollback_floor: u64,
    pub parameter_generation: u32,
    pub runtime_profile_generation: u16,
    pub composition_rule_generation: u16,
    pub per_proof_error_exponent: u16,
    pub inherited_error_exponent: u16,
    pub cumulative_error_exponent: u16,
    pub minimum_residual_bits: u16,
    pub chain_context_digest: [u8; 32],
    pub genesis_trust_anchor_digest: [u8; 32],
    pub genesis_state_root: [u8; 32],
    pub previous_terminal_state_root: [u8; 32],
    pub current_terminal_state_root: [u8; 32],
    pub previous_epoch_anchor_root: [u8; 32],
    pub current_epoch_anchor_root: [u8; 32],
    pub exact_epoch_statement_digest: [u8; 32],
    pub predicate_digest: [u8; 32],
    pub verifier_parameter_digest: [u8; 32],
    pub security_budget_digest: [u8; 32],
    pub config_digest: [u8; 32],
    pub registry_digest: [u8; 32],
    pub runtime_profile_manifest_digest: [u8; 32],
    pub authority_bundle_digest: [u8; 32],
    pub verifier_bundle_digest: [u8; 32],
    pub epoch_anchor_mmr_root: [u8; 32],
    pub predecessor_statement_digest: Option<[u8; 32]>,
}

impl HistoryAccumulatorInputsV2 {
    #[must_use]
    pub const fn authority_identity(self) -> HistoryAuthorityIdentityV2 {
        HistoryAuthorityIdentityV2 {
            config_generation: self.config_generation,
            authority_generation: self.authority_generation,
            parameter_generation: self.parameter_generation,
            activation_height: self.activation_height,
            rollback_floor: self.rollback_floor,
            runtime_profile_generation: self.runtime_profile_generation,
            config_digest: self.config_digest,
            registry_digest: self.registry_digest,
            runtime_profile_manifest_digest: self.runtime_profile_manifest_digest,
            authority_bundle_digest: self.authority_bundle_digest,
            verifier_bundle_digest: self.verifier_bundle_digest,
            verifier_parameter_digest: self.verifier_parameter_digest,
            security_budget_digest: self.security_budget_digest,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryAccumulatorStatementV2 {
    inputs: HistoryAccumulatorInputsV2,
    digest: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl HistoryAccumulatorStatementV2 {
    pub fn new(inputs: HistoryAccumulatorInputsV2) -> Result<Self, CheckpointError> {
        let authority = Plonky3HistoryAuthorityResolverV2::resolve_active()?;
        Self::new_with_authority(inputs, &authority)
    }

    fn new_with_authority(
        inputs: HistoryAccumulatorInputsV2,
        authority: &ResolvedPlonky3HistoryAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        validate_history_inputs(&inputs, authority)?;
        let registry = authority.registry();
        let payload = encode_history_payload(&inputs);
        if payload.len() != HISTORY_STATEMENT_PAYLOAD_BYTES_V2 {
            return Err(CheckpointError::Invariant);
        }
        let preheader = registry.encode_preheader(
            RecursiveBoundedObjectV2::HistoryAccumulatorStatement,
            payload.len(),
        )?;
        let mut canonical_bytes =
            Vec::with_capacity(RECURSIVE_OBJECT_PREHEADER_BYTES_V2 + payload.len());
        canonical_bytes.extend_from_slice(&preheader);
        canonical_bytes.extend_from_slice(&payload);
        let domain = registry
            .row(RecursiveBoundedObjectV2::HistoryAccumulatorStatement)?
            .cryptographic_domain;
        let digest = sha256_256(
            domain,
            HISTORY_STATEMENT_DIGEST_LABEL_V2,
            &[&canonical_bytes],
        );
        Ok(Self {
            inputs,
            digest,
            canonical_bytes,
        })
    }

    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, CheckpointError> {
        let authority = Plonky3HistoryAuthorityResolverV2::resolve_active()?;
        Self::decode_canonical_with_authority(bytes, &authority)
    }

    pub(crate) fn decode_canonical_with_authority(
        bytes: &[u8],
        authority: &ResolvedPlonky3HistoryAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        let registry = authority.registry();
        let preheader = registry
            .validate_preheader(bytes, RecursiveBoundedObjectV2::HistoryAccumulatorStatement)?;
        if preheader.header_len != RECURSIVE_OBJECT_PREHEADER_BYTES_V2
            || preheader.declared_len != HISTORY_STATEMENT_PAYLOAD_BYTES_V2 as u64
        {
            return Err(CheckpointError::Canonical);
        }
        let inputs = decode_history_payload(
            bytes
                .get(preheader.header_len..)
                .ok_or(CheckpointError::Canonical)?,
        )?;
        let statement = Self::new_with_authority(inputs, authority)?;
        if statement.canonical_bytes != bytes {
            return Err(CheckpointError::Canonical);
        }
        Ok(statement)
    }

    #[must_use]
    pub const fn inputs(&self) -> HistoryAccumulatorInputsV2 {
        self.inputs
    }

    #[must_use]
    pub const fn branch(&self) -> HistoryBranchV2 {
        self.inputs.branch
    }

    #[must_use]
    pub const fn last_epoch(&self) -> u64 {
        self.inputs.last_epoch
    }

    #[must_use]
    pub const fn last_height(&self) -> u64 {
        self.inputs.last_height
    }

    #[must_use]
    pub const fn history_length(&self) -> u64 {
        self.inputs.history_length
    }

    #[must_use]
    pub const fn accepted_epoch_count(&self) -> u64 {
        self.inputs.accepted_epoch_count
    }

    #[must_use]
    pub const fn current_terminal_state_root(&self) -> [u8; 32] {
        self.inputs.current_terminal_state_root
    }

    #[must_use]
    pub const fn current_epoch_anchor_root(&self) -> [u8; 32] {
        self.inputs.current_epoch_anchor_root
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
    pub const fn authority_identity(&self) -> HistoryAuthorityIdentityV2 {
        self.inputs.authority_identity()
    }
}

fn validate_history_inputs(
    inputs: &HistoryAccumulatorInputsV2,
    authority: &ResolvedPlonky3HistoryAuthorityV2,
) -> Result<(), CheckpointError> {
    let identity = authority.identity();
    let registry = authority.registry();
    let security = authority.security();
    let expected_length = inputs
        .last_epoch
        .checked_sub(inputs.first_epoch)
        .and_then(|span| span.checked_add(1))
        .ok_or(CheckpointError::Overflow)?;
    let expected_first_height = inputs
        .first_epoch
        .checked_mul(inputs.cadence_blocks)
        .and_then(|height| height.checked_add(1));
    let expected_last_height = inputs
        .last_epoch
        .checked_add(1)
        .and_then(|epoch| epoch.checked_mul(inputs.cadence_blocks));
    let expected_cumulative = composed_history_error_exponent_v2(
        inputs.per_proof_error_exponent,
        inputs.accepted_epoch_count,
        inputs.inherited_error_exponent,
    )?;
    if inputs.cadence_blocks == 0
        || inputs.history_length == 0
        || inputs.accepted_epoch_count == 0
        || inputs.history_length != expected_length
        || inputs.accepted_epoch_count != inputs.history_length
        || expected_first_height != Some(inputs.first_height)
        || expected_last_height != Some(inputs.last_height)
        || inputs.parameter_generation != security.parameter_generation()
        || inputs.composition_rule_generation != security.composition_rule_generation()
        || inputs.per_proof_error_exponent != security.per_proof_error_exponent()
        || inputs.inherited_error_exponent
            != security.inherited_error_exponent().unwrap_or_default()
        || inputs.cumulative_error_exponent != expected_cumulative
        || inputs.minimum_residual_bits != security.minimum_residual_bits()
        || inputs.cumulative_error_exponent < inputs.minimum_residual_bits
        || inputs.accepted_epoch_count > security.max_accepted_epoch_proofs()
        || inputs.security_budget_digest != security.digest()
        || inputs.authority_identity() != identity
        || inputs.registry_digest != registry.digest()
    {
        return Err(security_or_history_error());
    }
    match inputs.branch {
        HistoryBranchV2::Base => {
            if inputs.history_length != 1
                || inputs.first_epoch != inputs.last_epoch
                || inputs.predecessor_statement_digest.is_some()
                || inputs.previous_terminal_state_root != inputs.genesis_state_root
            {
                return Err(security_or_history_error());
            }
        }
        HistoryBranchV2::Successor => {
            if inputs.history_length < 2
                || inputs.predecessor_statement_digest.is_none()
                || inputs.predecessor_statement_digest == Some([0; 32])
            {
                return Err(security_or_history_error());
            }
        }
    }
    if [
        inputs.chain_context_digest,
        inputs.genesis_trust_anchor_digest,
        inputs.genesis_state_root,
        inputs.previous_terminal_state_root,
        inputs.current_terminal_state_root,
        inputs.previous_epoch_anchor_root,
        inputs.current_epoch_anchor_root,
        inputs.exact_epoch_statement_digest,
        inputs.predicate_digest,
        inputs.verifier_parameter_digest,
        inputs.security_budget_digest,
        inputs.config_digest,
        inputs.registry_digest,
        inputs.runtime_profile_manifest_digest,
        inputs.authority_bundle_digest,
        inputs.verifier_bundle_digest,
        inputs.epoch_anchor_mmr_root,
    ]
    .contains(&[0; 32])
    {
        return Err(security_or_history_error());
    }
    Ok(())
}

/// Conservative integer-only upper bound for
/// `inherited + accepted_epoch_count * per_proof`.
pub fn composed_history_error_exponent_v2(
    per_proof_exponent: u16,
    accepted_epoch_count: u64,
    inherited_exponent: u16,
) -> Result<u16, CheckpointError> {
    if per_proof_exponent == 0 || accepted_epoch_count == 0 || inherited_exponent == 0 {
        return Err(security_or_history_error());
    }
    let count_loss = u16::try_from(u64::BITS - (accepted_epoch_count - 1).leading_zeros())
        .map_err(|_| CheckpointError::Overflow)?;
    let epoch_terms = per_proof_exponent
        .checked_sub(count_loss)
        .ok_or_else(security_or_history_error)?;
    epoch_terms
        .min(inherited_exponent)
        .checked_sub(1)
        .ok_or_else(security_or_history_error)
}

fn security_or_history_error() -> CheckpointError {
    CheckpointError::RecursiveRejected(
        RecursiveCheckpointRejectReasonV2::Plonky3SecurityBudgetInvalid,
    )
}

fn encode_history_payload(inputs: &HistoryAccumulatorInputsV2) -> Vec<u8> {
    let mut payload = Vec::with_capacity(HISTORY_STATEMENT_PAYLOAD_BYTES_V2);
    payload.extend_from_slice(&HISTORY_STATEMENT_MAGIC_V2);
    payload.extend_from_slice(&HISTORY_WIRE_VERSION_V2.to_le_bytes());
    payload.push(inputs.branch as u8);
    for value in [
        inputs.first_epoch,
        inputs.last_epoch,
        inputs.first_height,
        inputs.last_height,
        inputs.cadence_blocks,
        inputs.history_length,
        inputs.accepted_epoch_count,
        inputs.config_generation,
        inputs.authority_generation,
        inputs.activation_height,
        inputs.rollback_floor,
    ] {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload.extend_from_slice(&inputs.parameter_generation.to_le_bytes());
    payload.extend_from_slice(&inputs.runtime_profile_generation.to_le_bytes());
    for value in [
        inputs.composition_rule_generation,
        inputs.per_proof_error_exponent,
        inputs.inherited_error_exponent,
        inputs.cumulative_error_exponent,
        inputs.minimum_residual_bits,
    ] {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    for digest in [
        inputs.chain_context_digest,
        inputs.genesis_trust_anchor_digest,
        inputs.genesis_state_root,
        inputs.previous_terminal_state_root,
        inputs.current_terminal_state_root,
        inputs.previous_epoch_anchor_root,
        inputs.current_epoch_anchor_root,
        inputs.exact_epoch_statement_digest,
        inputs.predicate_digest,
        inputs.verifier_parameter_digest,
        inputs.security_budget_digest,
        inputs.config_digest,
        inputs.registry_digest,
        inputs.runtime_profile_manifest_digest,
        inputs.authority_bundle_digest,
        inputs.verifier_bundle_digest,
        inputs.epoch_anchor_mmr_root,
        history_branch_digest(inputs.branch),
    ] {
        payload.extend_from_slice(&digest);
    }
    encode_optional_digest(&mut payload, inputs.predecessor_statement_digest);
    payload
}

fn decode_history_payload(payload: &[u8]) -> Result<HistoryAccumulatorInputsV2, CheckpointError> {
    if payload.len() != HISTORY_STATEMENT_PAYLOAD_BYTES_V2 {
        return Err(CheckpointError::Canonical);
    }
    let mut reader = EpochCodecReaderV2::new(payload);
    if reader.array::<8>()? != HISTORY_STATEMENT_MAGIC_V2
        || reader.u16()? != HISTORY_WIRE_VERSION_V2
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::UnsupportedVersion,
        ));
    }
    let branch = HistoryBranchV2::decode(reader.u8()?)?;
    let inputs = HistoryAccumulatorInputsV2 {
        branch,
        first_epoch: reader.u64()?,
        last_epoch: reader.u64()?,
        first_height: reader.u64()?,
        last_height: reader.u64()?,
        cadence_blocks: reader.u64()?,
        history_length: reader.u64()?,
        accepted_epoch_count: reader.u64()?,
        config_generation: reader.u64()?,
        authority_generation: reader.u64()?,
        activation_height: reader.u64()?,
        rollback_floor: reader.u64()?,
        parameter_generation: reader.u32()?,
        runtime_profile_generation: reader.u16()?,
        composition_rule_generation: reader.u16()?,
        per_proof_error_exponent: reader.u16()?,
        inherited_error_exponent: reader.u16()?,
        cumulative_error_exponent: reader.u16()?,
        minimum_residual_bits: reader.u16()?,
        chain_context_digest: reader.array()?,
        genesis_trust_anchor_digest: reader.array()?,
        genesis_state_root: reader.array()?,
        previous_terminal_state_root: reader.array()?,
        current_terminal_state_root: reader.array()?,
        previous_epoch_anchor_root: reader.array()?,
        current_epoch_anchor_root: reader.array()?,
        exact_epoch_statement_digest: reader.array()?,
        predicate_digest: reader.array()?,
        verifier_parameter_digest: reader.array()?,
        security_budget_digest: reader.array()?,
        config_digest: reader.array()?,
        registry_digest: reader.array()?,
        runtime_profile_manifest_digest: reader.array()?,
        authority_bundle_digest: reader.array()?,
        verifier_bundle_digest: reader.array()?,
        epoch_anchor_mmr_root: reader.array()?,
        predecessor_statement_digest: {
            let branch_digest: [u8; 32] = reader.array()?;
            if branch_digest != history_branch_digest(branch) {
                return Err(CheckpointError::Canonical);
            }
            decode_optional_digest(&mut reader)?
        },
    };
    if !reader.is_done() {
        return Err(CheckpointError::Canonical);
    }
    Ok(inputs)
}

fn history_branch_digest(branch: HistoryBranchV2) -> [u8; 32] {
    sha256_256(
        "z00z.storage.checkpoint.history-branch.v2",
        "tag",
        &[&[branch as u8]],
    )
}

fn encode_optional_digest(bytes: &mut Vec<u8>, digest: Option<[u8; 32]>) {
    bytes.push(u8::from(digest.is_some()));
    bytes.extend_from_slice(&digest.unwrap_or([0; 32]));
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryRotationInputsV2 {
    pub inherited_accepted_epoch_count: u64,
    pub activation_height: u64,
    pub first_new_epoch: u64,
    pub old_parameter_generation: u32,
    pub new_parameter_generation: u32,
    pub inherited_error_exponent: u16,
    pub new_per_proof_error_exponent: u16,
    pub minimum_residual_bits: u16,
    pub chain_context_digest: [u8; 32],
    pub predicate_digest: [u8; 32],
    pub old_registry_digest: [u8; 32],
    pub new_registry_digest: [u8; 32],
    pub old_runtime_profile_manifest_digest: [u8; 32],
    pub new_runtime_profile_manifest_digest: [u8; 32],
    pub old_verifier_manifest_digest: [u8; 32],
    pub new_verifier_manifest_digest: [u8; 32],
    pub old_security_budget_digest: [u8; 32],
    pub new_security_budget_digest: [u8; 32],
    pub old_history_statement_digest: [u8; 32],
    pub first_new_epoch_statement_digest: [u8; 32],
    pub old_terminal_state_root: [u8; 32],
    pub first_new_epoch_start_root: [u8; 32],
    pub previous_epoch_anchor_root: [u8; 32],
    pub new_epoch_anchor_root: [u8; 32],
    pub authority_rotation_commitment: [u8; 32],
    pub new_config_digest: [u8; 32],
    pub output_history_statement_digest: [u8; 32],
    pub old_authority_identity_digest: [u8; 32],
    pub new_authority_identity_digest: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryRotationBridgeV2 {
    inputs: HistoryRotationInputsV2,
    digest: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl HistoryRotationBridgeV2 {
    pub fn new(inputs: HistoryRotationInputsV2) -> Result<Self, CheckpointError> {
        let authority = Plonky3HistoryAuthorityResolverV2::resolve_active()?;
        Self::new_with_authority(inputs, &authority)
    }

    fn new_with_authority(
        inputs: HistoryRotationInputsV2,
        authority: &ResolvedPlonky3HistoryAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        validate_rotation_inputs(&inputs, authority)?;
        let registry = authority.registry();
        let payload = encode_rotation_payload(&inputs);
        if payload.len() != HISTORY_ROTATION_PAYLOAD_BYTES_V2 {
            return Err(CheckpointError::Invariant);
        }
        let preheader = registry.encode_preheader(
            RecursiveBoundedObjectV2::HistoryRotationBridge,
            payload.len(),
        )?;
        let mut canonical_bytes =
            Vec::with_capacity(RECURSIVE_OBJECT_PREHEADER_BYTES_V2 + payload.len());
        canonical_bytes.extend_from_slice(&preheader);
        canonical_bytes.extend_from_slice(&payload);
        let domain = registry
            .row(RecursiveBoundedObjectV2::HistoryRotationBridge)?
            .cryptographic_domain;
        let digest = sha256_256(
            domain,
            HISTORY_ROTATION_DIGEST_LABEL_V2,
            &[&canonical_bytes],
        );
        Ok(Self {
            inputs,
            digest,
            canonical_bytes,
        })
    }

    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, CheckpointError> {
        let authority = Plonky3HistoryAuthorityResolverV2::resolve_active()?;
        Self::decode_canonical_with_authority(bytes, &authority)
    }

    pub(crate) fn decode_canonical_with_authority(
        bytes: &[u8],
        authority: &ResolvedPlonky3HistoryAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        let registry = authority.registry();
        let preheader =
            registry.validate_preheader(bytes, RecursiveBoundedObjectV2::HistoryRotationBridge)?;
        if preheader.header_len != RECURSIVE_OBJECT_PREHEADER_BYTES_V2
            || preheader.declared_len != HISTORY_ROTATION_PAYLOAD_BYTES_V2 as u64
        {
            return Err(CheckpointError::Canonical);
        }
        let inputs = decode_rotation_payload(
            bytes
                .get(preheader.header_len..)
                .ok_or(CheckpointError::Canonical)?,
        )?;
        let bridge = Self::new_with_authority(inputs, authority)?;
        if bridge.canonical_bytes != bytes {
            return Err(CheckpointError::Canonical);
        }
        Ok(bridge)
    }

    #[must_use]
    pub const fn inputs(&self) -> HistoryRotationInputsV2 {
        self.inputs
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

fn validate_rotation_inputs(
    inputs: &HistoryRotationInputsV2,
    authority: &ResolvedPlonky3HistoryAuthorityV2,
) -> Result<(), CheckpointError> {
    let identity = authority.identity();
    let registry = authority.registry();
    let security = authority.security();
    let cadence = authority.cadence_blocks();
    let expected_activation = inputs
        .first_new_epoch
        .checked_mul(cadence)
        .and_then(|height| height.checked_add(1));
    if inputs.inherited_accepted_epoch_count == 0
        || inputs.activation_height == 0
        || inputs.first_new_epoch == 0
        || expected_activation != Some(inputs.activation_height)
        || inputs.old_parameter_generation == 0
        || inputs.new_parameter_generation != identity.parameter_generation
        || inputs.new_parameter_generation != security.parameter_generation()
        || inputs.old_parameter_generation == inputs.new_parameter_generation
        || inputs.inherited_error_exponent < inputs.minimum_residual_bits
        || inputs.new_per_proof_error_exponent != security.per_proof_error_exponent()
        || inputs.minimum_residual_bits != security.minimum_residual_bits()
        || inputs.new_registry_digest != registry.digest()
        || inputs.new_registry_digest != identity.registry_digest
        || inputs.new_runtime_profile_manifest_digest != identity.runtime_profile_manifest_digest
        || inputs.new_verifier_manifest_digest != identity.verifier_parameter_digest
        || inputs.new_security_budget_digest != security.digest()
        || inputs.new_config_digest != identity.config_digest
        || inputs.new_authority_identity_digest != identity.digest()
        || inputs.old_authority_identity_digest == inputs.new_authority_identity_digest
        || inputs.authority_rotation_commitment != authority.rotation_commitment()
        || inputs.old_registry_digest == inputs.new_registry_digest
            && inputs.old_runtime_profile_manifest_digest
                == inputs.new_runtime_profile_manifest_digest
            && inputs.old_verifier_manifest_digest == inputs.new_verifier_manifest_digest
        || inputs.old_terminal_state_root != inputs.first_new_epoch_start_root
    {
        return Err(security_or_history_error());
    }
    let digests = [
        inputs.chain_context_digest,
        inputs.predicate_digest,
        inputs.old_registry_digest,
        inputs.new_registry_digest,
        inputs.old_runtime_profile_manifest_digest,
        inputs.new_runtime_profile_manifest_digest,
        inputs.old_verifier_manifest_digest,
        inputs.new_verifier_manifest_digest,
        inputs.old_security_budget_digest,
        inputs.new_security_budget_digest,
        inputs.old_history_statement_digest,
        inputs.first_new_epoch_statement_digest,
        inputs.old_terminal_state_root,
        inputs.first_new_epoch_start_root,
        inputs.previous_epoch_anchor_root,
        inputs.new_epoch_anchor_root,
        inputs.authority_rotation_commitment,
        inputs.new_config_digest,
        inputs.output_history_statement_digest,
        inputs.old_authority_identity_digest,
        inputs.new_authority_identity_digest,
    ];
    if digests.contains(&[0; 32]) {
        return Err(security_or_history_error());
    }
    Ok(())
}

fn encode_rotation_payload(inputs: &HistoryRotationInputsV2) -> Vec<u8> {
    let mut payload = Vec::with_capacity(HISTORY_ROTATION_PAYLOAD_BYTES_V2);
    payload.extend_from_slice(&HISTORY_ROTATION_MAGIC_V2);
    payload.extend_from_slice(&HISTORY_WIRE_VERSION_V2.to_le_bytes());
    payload.extend_from_slice(&inputs.inherited_accepted_epoch_count.to_le_bytes());
    payload.extend_from_slice(&inputs.activation_height.to_le_bytes());
    payload.extend_from_slice(&inputs.first_new_epoch.to_le_bytes());
    payload.extend_from_slice(&inputs.old_parameter_generation.to_le_bytes());
    payload.extend_from_slice(&inputs.new_parameter_generation.to_le_bytes());
    payload.extend_from_slice(&inputs.inherited_error_exponent.to_le_bytes());
    payload.extend_from_slice(&inputs.new_per_proof_error_exponent.to_le_bytes());
    payload.extend_from_slice(&inputs.minimum_residual_bits.to_le_bytes());
    for digest in [
        inputs.chain_context_digest,
        inputs.predicate_digest,
        inputs.old_registry_digest,
        inputs.new_registry_digest,
        inputs.old_runtime_profile_manifest_digest,
        inputs.new_runtime_profile_manifest_digest,
        inputs.old_verifier_manifest_digest,
        inputs.new_verifier_manifest_digest,
        inputs.old_security_budget_digest,
        inputs.new_security_budget_digest,
        inputs.old_history_statement_digest,
        inputs.first_new_epoch_statement_digest,
        inputs.old_terminal_state_root,
        inputs.first_new_epoch_start_root,
        inputs.previous_epoch_anchor_root,
        inputs.new_epoch_anchor_root,
        inputs.authority_rotation_commitment,
        inputs.new_config_digest,
        inputs.output_history_statement_digest,
        inputs.old_authority_identity_digest,
        inputs.new_authority_identity_digest,
    ] {
        payload.extend_from_slice(&digest);
    }
    payload
}

fn decode_rotation_payload(payload: &[u8]) -> Result<HistoryRotationInputsV2, CheckpointError> {
    if payload.len() != HISTORY_ROTATION_PAYLOAD_BYTES_V2 {
        return Err(CheckpointError::Canonical);
    }
    let mut reader = EpochCodecReaderV2::new(payload);
    if reader.array::<8>()? != HISTORY_ROTATION_MAGIC_V2 || reader.u16()? != HISTORY_WIRE_VERSION_V2
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::UnsupportedVersion,
        ));
    }
    let inputs = HistoryRotationInputsV2 {
        inherited_accepted_epoch_count: reader.u64()?,
        activation_height: reader.u64()?,
        first_new_epoch: reader.u64()?,
        old_parameter_generation: reader.u32()?,
        new_parameter_generation: reader.u32()?,
        inherited_error_exponent: reader.u16()?,
        new_per_proof_error_exponent: reader.u16()?,
        minimum_residual_bits: reader.u16()?,
        chain_context_digest: reader.array()?,
        predicate_digest: reader.array()?,
        old_registry_digest: reader.array()?,
        new_registry_digest: reader.array()?,
        old_runtime_profile_manifest_digest: reader.array()?,
        new_runtime_profile_manifest_digest: reader.array()?,
        old_verifier_manifest_digest: reader.array()?,
        new_verifier_manifest_digest: reader.array()?,
        old_security_budget_digest: reader.array()?,
        new_security_budget_digest: reader.array()?,
        old_history_statement_digest: reader.array()?,
        first_new_epoch_statement_digest: reader.array()?,
        old_terminal_state_root: reader.array()?,
        first_new_epoch_start_root: reader.array()?,
        previous_epoch_anchor_root: reader.array()?,
        new_epoch_anchor_root: reader.array()?,
        authority_rotation_commitment: reader.array()?,
        new_config_digest: reader.array()?,
        output_history_statement_digest: reader.array()?,
        old_authority_identity_digest: reader.array()?,
        new_authority_identity_digest: reader.array()?,
    };
    if !reader.is_done() {
        return Err(CheckpointError::Canonical);
    }
    Ok(inputs)
}
