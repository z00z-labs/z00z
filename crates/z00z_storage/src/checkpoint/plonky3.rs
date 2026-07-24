//! Private Plonky3 owner for the recursive-checkpoint V2 base relation.
//!
//! The public surface is re-exported only by `checkpoint::recursive_v2`.  No
//! Plonky3 field, AIR, PCS, proof, or configuration type crosses that facade.
//! This local outer STARK is PQ-oriented only under its pinned manifest. The
//! nested Nova/config authority remains classical, so this module makes no
//! end-to-end post-quantum, finality, admission, or production-authority claim.

use core::{fmt, marker::PhantomData};

use p3_batch_stark::ProverData;
use p3_challenger::DuplexChallenger;
use p3_circuit::ops::poseidon2_perm::Poseidon2PermCallBase;
use p3_circuit::ops::{generate_poseidon2_trace, KoalaBearD1Width16, Poseidon2Config};
use p3_circuit::{Circuit, CircuitBuilder, ExprId};
use p3_circuit_prover::batch_stark_prover::{
    poseidon2_air_builders_d5, poseidon2_table_provers_d5, Poseidon2Preprocessor,
};
use p3_circuit_prover::common::{get_airs_and_degrees_with_prep, NpoPreprocessor};
use p3_circuit_prover::config::KoalaBearConfig;
use p3_circuit_prover::{
    BatchStarkProof, BatchStarkProver, CircuitProverData, ConstraintProfile, TablePacking,
};
use p3_commit::ExtensionMmcs;
use p3_dft::Radix2DitParallel;
use p3_field::extension::{
    BinomialExtensionField, QuinticTrinomialExtendable, QuinticTrinomialExtensionField,
};
use p3_field::{BasedVectorSpace, PrimeCharacteristicRing};
use p3_fri::{FriParameters, TwoAdicFriPcs};
use p3_koala_bear::{default_koalabear_poseidon2_16, KoalaBear, Poseidon2KoalaBear};
use p3_merkle_tree::MerkleTreeMmcs;
use p3_symmetric::{PaddingFreeSponge, Permutation, TruncatedPermutation};
use p3_uni_stark::StarkConfig;
use z00z_crypto::{sha256_256, sha256_256_role, CheckpointShaRole};
use zeroize::Zeroize;

use super::{
    authority_artifacts::{
        ACTIVE_PLONKY3_CIRCUIT_VERSION_V2, ACTIVE_PLONKY3_CRATES_IO_VERSION_V2,
        ACTIVE_PLONKY3_SOURCE_REVISION_V2,
    },
    canonical_transition::CanonicalCheckpointTransitionV2,
    contract_config_v3::CheckpointConfigResolverV3,
    receipt::Plonky3BaseVerificationReceiptV2,
    recursive_circuit::{RecursiveCircuitProfileV2, RecursiveCircuitSpecV2},
    recursive_reject::RecursiveCheckpointRejectReasonV2,
    recursive_statement::RecursiveTransitionStatementV2,
    recursive_trace::{
        decode_hash_control, RecursiveTraceEventV2, RecursiveTraceOpcodeV2,
        HASH_CONTROL_BLOCK_BYTES_V2, RECURSIVE_TRACE_OPCODE_COUNT_V2, TRACE_EVENT_HEADER_BYTES_V2,
    },
    version_registry::{CheckpointVersionRegistryV2, RecursiveBoundedObjectV2},
};
use crate::{settlement::SettlementStore, CheckpointError};

const PLONKY3_BASE_WIRE_VERSION_V2: u16 = 2;
const PLONKY3_BASE_MAGIC_V2: [u8; 8] = *b"Z00ZP3B2";
const PLONKY3_STATEMENT_MAGIC_V2: [u8; 8] = *b"Z00ZP3S2";
const PLONKY3_PARAMETER_MAGIC_V2: [u8; 8] = *b"Z00ZP3P2";
const PLONKY3_SECURITY_MAGIC_V2: [u8; 8] = *b"Z00ZP3Q2";
const PLONKY3_EVENT_VECTOR_MAGIC_V2: [u8; 8] = *b"Z00ZP3E2";
const PLONKY3_BASE_MAX_PROOF_BYTES_V2: usize = 16 * 1024 * 1024;
const PLONKY3_BASE_MAX_VECTOR_BYTES_V2: usize = 16 * 1024 * 1024;
const PLONKY3_BASE_STATEMENT_BYTES_V2: usize = 8
    + 2
    + 32 * 11
    + 8
    + 32
    + 1
    + 32
    + 32
    + 8
    + 32 * 17
    + 8
    + 8
    + RECURSIVE_TRACE_OPCODE_COUNT_V2 * 8 * 2;
const PLONKY3_BASE_MAX_CANONICAL_BYTES_V2: usize =
    PLONKY3_BASE_MAX_PROOF_BYTES_V2 + PLONKY3_BASE_STATEMENT_BYTES_V2 + 256;
const PLONKY3_PREDICATE_VECTOR_LABEL_V2: &[u8] = b"z00z.plonky3.base.predicate-vector.v2";
const PLONKY3_FRI_LOG_BLOWUP_V2: u8 = 1;
const PLONKY3_FRI_LOG_FINAL_POLY_LEN_V2: u8 = 0;
const PLONKY3_FRI_MAX_LOG_ARITY_V2: u8 = 3;
const PLONKY3_FRI_NUM_QUERIES_V2: u16 = 112;
const PLONKY3_FRI_COMMIT_POW_BITS_V2: u8 = 0;
const PLONKY3_FRI_QUERY_POW_BITS_V2: u8 = 16;
const PLONKY3_BASE_FIELD_BITS_V2: u16 = 31;
const PLONKY3_CHALLENGE_EXTENSION_DEGREE_V2: u8 = 4;
const PLONKY3_FRI_CONJECTURED_BITS_V2: u16 = 124;
const PLONKY3_TABLE_MIN_HEIGHT_V2: usize = 8;
const PLONKY3_TABLE_PUBLIC_LANES_V2: usize = 4;
const PLONKY3_TABLE_ALU_LANES_V2: usize = 4;
const PLONKY3_TRACE_EXTENSION_DEGREE_V2: u8 = 5;
const PLONKY3_SECURITY_GENERATION_V2: u16 = 1;
const PLONKY3_SECURITY_COMPOSITION_RULE_GENERATION_V2: u16 = 1;
const PLONKY3_BASE_RECURSION_DEPTH_V2: u16 = 1;
const PLONKY3_MAX_ACCEPTED_EPOCH_PROOFS_V2: u64 = 1 << 20;
const PLONKY3_MINIMUM_RESIDUAL_BITS_V2: u16 = 100;
const PLONKY3_PER_PROOF_BOUND_BITS_V2: u16 = 123;
const PLONKY3_LIFETIME_BOUND_BITS_V2: u16 = 102;

type Plonky3TraceFieldV2 = QuinticTrinomialExtensionField<KoalaBear>;
type Plonky3WordBitsV2 = [ExprId; 32];

const SHA256_ROUND_CONSTANTS_V2: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// Conservative dyadic upper bound `2^-denominator_exponent`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DyadicErrorBoundV2 {
    denominator_exponent: u16,
}

impl DyadicErrorBoundV2 {
    fn new(denominator_exponent: u16) -> Result<Self, CheckpointError> {
        if denominator_exponent == 0 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3SecurityBudgetInvalid,
            ));
        }
        Ok(Self {
            denominator_exponent,
        })
    }

    #[must_use]
    pub const fn denominator_exponent(self) -> u16 {
        self.denominator_exponent
    }
}

/// Generation-pinned, integer-only composition record for the base STARK.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecursiveSecurityBudgetManifestV2 {
    generation: u16,
    parameter_generation: u32,
    base_field_bits: u16,
    challenge_extension_degree: u8,
    fri_log_blowup: u8,
    fri_num_queries: u16,
    fri_commit_pow_bits: u8,
    fri_query_pow_bits: u8,
    fri_conjectured_bits: u16,
    hash_collision_bits: u16,
    component_count: u16,
    recursion_depth: u16,
    composition_rule_generation: u16,
    per_proof_bound: DyadicErrorBoundV2,
    max_accepted_epoch_proofs: u64,
    inherited_bound: Option<DyadicErrorBoundV2>,
    lifetime_bound: DyadicErrorBoundV2,
    minimum_residual_bits: u16,
    canonical_bytes: Vec<u8>,
}

impl RecursiveSecurityBudgetManifestV2 {
    /// The one live Plan-07 budget. The KoalaBear challenge field caps the
    /// nominal 128-bit FRI calculation at 124 bits; composing that with the
    /// 128-bit collision family rounds upward to `2^-123`. At most `2^20`
    /// proofs plus inherited rotation loss round upward to `2^-102`.
    pub fn authority_pinned() -> Result<Self, CheckpointError> {
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        let parameter_generation = registry
            .row(RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest)?
            .parameter_generation
            .ok_or(CheckpointError::Authority)?;
        let per_proof_bound = derive_per_proof_bound(PLONKY3_FRI_CONJECTURED_BITS_V2, 128, 2)?;
        let inherited_bound = DyadicErrorBoundV2::new(128)?;
        let lifetime_bound = derive_lifetime_bound(
            per_proof_bound,
            PLONKY3_MAX_ACCEPTED_EPOCH_PROOFS_V2,
            inherited_bound,
        )?;
        let manifest = Self {
            generation: PLONKY3_SECURITY_GENERATION_V2,
            parameter_generation,
            base_field_bits: PLONKY3_BASE_FIELD_BITS_V2,
            challenge_extension_degree: PLONKY3_CHALLENGE_EXTENSION_DEGREE_V2,
            fri_log_blowup: PLONKY3_FRI_LOG_BLOWUP_V2,
            fri_num_queries: PLONKY3_FRI_NUM_QUERIES_V2,
            fri_commit_pow_bits: PLONKY3_FRI_COMMIT_POW_BITS_V2,
            fri_query_pow_bits: PLONKY3_FRI_QUERY_POW_BITS_V2,
            fri_conjectured_bits: PLONKY3_FRI_CONJECTURED_BITS_V2,
            hash_collision_bits: 128,
            component_count: 2,
            recursion_depth: PLONKY3_BASE_RECURSION_DEPTH_V2,
            composition_rule_generation: PLONKY3_SECURITY_COMPOSITION_RULE_GENERATION_V2,
            per_proof_bound,
            max_accepted_epoch_proofs: PLONKY3_MAX_ACCEPTED_EPOCH_PROOFS_V2,
            inherited_bound: Some(inherited_bound),
            lifetime_bound,
            minimum_residual_bits: PLONKY3_MINIMUM_RESIDUAL_BITS_V2,
            canonical_bytes: Vec::new(),
        };
        manifest.validate()?;
        let payload = manifest.payload_bytes();
        let preheader = registry.encode_preheader(
            RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest,
            payload.len(),
        )?;
        let mut manifest = manifest;
        manifest
            .canonical_bytes
            .reserve_exact(preheader.len() + payload.len());
        manifest.canonical_bytes.extend_from_slice(&preheader);
        manifest.canonical_bytes.extend_from_slice(&payload);
        Ok(manifest)
    }

    fn validate(&self) -> Result<(), CheckpointError> {
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        let expected_parameter_generation = registry
            .row(RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest)?
            .parameter_generation
            .ok_or(CheckpointError::Authority)?;
        let field_security = self
            .base_field_bits
            .checked_mul(u16::from(self.challenge_extension_degree))
            .ok_or(CheckpointError::Overflow)?;
        let raw_fri = u16::from(self.fri_log_blowup)
            .checked_mul(self.fri_num_queries)
            .and_then(|value| value.checked_add(u16::from(self.fri_query_pow_bits)))
            .ok_or(CheckpointError::Overflow)?;
        let expected_fri = raw_fri.min(field_security);
        let expected_per_proof = derive_per_proof_bound(
            self.fri_conjectured_bits,
            self.hash_collision_bits,
            self.component_count,
        )?;
        let inherited_bound = self.inherited_bound.ok_or_else(security_budget_error)?;
        let expected_lifetime = derive_lifetime_bound(
            self.per_proof_bound,
            self.max_accepted_epoch_proofs,
            inherited_bound,
        )?;
        if self.generation != PLONKY3_SECURITY_GENERATION_V2
            || self.parameter_generation != expected_parameter_generation
            || self.base_field_bits != PLONKY3_BASE_FIELD_BITS_V2
            || self.challenge_extension_degree != PLONKY3_CHALLENGE_EXTENSION_DEGREE_V2
            || self.fri_log_blowup != PLONKY3_FRI_LOG_BLOWUP_V2
            || self.fri_num_queries != PLONKY3_FRI_NUM_QUERIES_V2
            || self.fri_commit_pow_bits != PLONKY3_FRI_COMMIT_POW_BITS_V2
            || self.fri_query_pow_bits != PLONKY3_FRI_QUERY_POW_BITS_V2
            || self.fri_conjectured_bits != expected_fri
            || self.hash_collision_bits != 128
            || self.component_count != 2
            || self.recursion_depth != PLONKY3_BASE_RECURSION_DEPTH_V2
            || self.composition_rule_generation != PLONKY3_SECURITY_COMPOSITION_RULE_GENERATION_V2
            || self.per_proof_bound != expected_per_proof
            || self.per_proof_bound.denominator_exponent() != PLONKY3_PER_PROOF_BOUND_BITS_V2
            || self.max_accepted_epoch_proofs != PLONKY3_MAX_ACCEPTED_EPOCH_PROOFS_V2
            || inherited_bound.denominator_exponent() != 128
            || self.lifetime_bound != expected_lifetime
            || self.lifetime_bound.denominator_exponent() != PLONKY3_LIFETIME_BOUND_BITS_V2
            || self.minimum_residual_bits != PLONKY3_MINIMUM_RESIDUAL_BITS_V2
            || self.lifetime_bound.denominator_exponent() < self.minimum_residual_bits
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3SecurityBudgetInvalid,
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        self.canonical_bytes.clone()
    }

    fn payload_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(48);
        bytes.extend_from_slice(&PLONKY3_SECURITY_MAGIC_V2);
        bytes.extend_from_slice(&self.generation.to_le_bytes());
        bytes.extend_from_slice(&self.parameter_generation.to_le_bytes());
        bytes.extend_from_slice(&self.base_field_bits.to_le_bytes());
        bytes.push(self.challenge_extension_degree);
        bytes.push(self.fri_log_blowup);
        bytes.extend_from_slice(&self.fri_num_queries.to_le_bytes());
        bytes.push(self.fri_commit_pow_bits);
        bytes.push(self.fri_query_pow_bits);
        bytes.extend_from_slice(&self.fri_conjectured_bits.to_le_bytes());
        bytes.extend_from_slice(&self.hash_collision_bits.to_le_bytes());
        bytes.extend_from_slice(&self.component_count.to_le_bytes());
        bytes.extend_from_slice(&self.recursion_depth.to_le_bytes());
        bytes.extend_from_slice(&self.composition_rule_generation.to_le_bytes());
        bytes.extend_from_slice(&self.per_proof_bound.denominator_exponent().to_le_bytes());
        bytes.extend_from_slice(&self.max_accepted_epoch_proofs.to_le_bytes());
        bytes.extend_from_slice(
            &self
                .inherited_bound
                .map(DyadicErrorBoundV2::denominator_exponent)
                .unwrap_or_default()
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&self.lifetime_bound.denominator_exponent().to_le_bytes());
        bytes.extend_from_slice(&self.minimum_residual_bits.to_le_bytes());
        bytes
    }

    /// Decode one exact manifest generation.  There is no fallback generation,
    /// floating-point path, or default for a missing inherited-loss field.
    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, CheckpointError> {
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        let header = registry.validate_preheader(
            bytes,
            RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest,
        )?;
        let payload = &bytes[header.header_len..];
        if payload.len() != 48 || payload[..8] != PLONKY3_SECURITY_MAGIC_V2 {
            return Err(CheckpointError::Canonical);
        }
        let mut cursor = 8;
        let generation = take_u16(payload, &mut cursor)?;
        let parameter_generation = take_u32(payload, &mut cursor)?;
        let base_field_bits = take_u16(payload, &mut cursor)?;
        let challenge_extension_degree = take_array::<1>(payload, &mut cursor)?[0];
        let fri_log_blowup = take_array::<1>(payload, &mut cursor)?[0];
        let fri_num_queries = take_u16(payload, &mut cursor)?;
        let fri_commit_pow_bits = take_array::<1>(payload, &mut cursor)?[0];
        let fri_query_pow_bits = take_array::<1>(payload, &mut cursor)?[0];
        let fri_conjectured_bits = take_u16(payload, &mut cursor)?;
        let hash_collision_bits = take_u16(payload, &mut cursor)?;
        let component_count = take_u16(payload, &mut cursor)?;
        let recursion_depth = take_u16(payload, &mut cursor)?;
        let composition_rule_generation = take_u16(payload, &mut cursor)?;
        let per_proof_bound = DyadicErrorBoundV2::new(take_u16(payload, &mut cursor)?)?;
        let max_accepted_epoch_proofs = u64::from_le_bytes(take_array::<8>(payload, &mut cursor)?);
        let inherited_exponent = take_u16(payload, &mut cursor)?;
        let inherited_bound = Some(DyadicErrorBoundV2::new(inherited_exponent)?);
        let lifetime_bound = DyadicErrorBoundV2::new(take_u16(payload, &mut cursor)?)?;
        let minimum_residual_bits = take_u16(payload, &mut cursor)?;
        if cursor != payload.len() {
            return Err(CheckpointError::Canonical);
        }
        let manifest = Self {
            generation,
            parameter_generation,
            base_field_bits,
            challenge_extension_degree,
            fri_log_blowup,
            fri_num_queries,
            fri_commit_pow_bits,
            fri_query_pow_bits,
            fri_conjectured_bits,
            hash_collision_bits,
            component_count,
            recursion_depth,
            composition_rule_generation,
            per_proof_bound,
            max_accepted_epoch_proofs,
            inherited_bound,
            lifetime_bound,
            minimum_residual_bits,
            canonical_bytes: bytes.to_vec(),
        };
        manifest.validate()?;
        if manifest.payload_bytes() != payload {
            return Err(CheckpointError::Canonical);
        }
        Ok(manifest)
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        sha256_256(
            "z00z.storage.checkpoint.plonky3.security-budget.v2",
            "manifest",
            &[&self.canonical_bytes()],
        )
    }

    #[must_use]
    pub const fn lifetime_residual_bits(&self) -> u16 {
        self.lifetime_bound.denominator_exponent()
    }
}

fn security_budget_error() -> CheckpointError {
    CheckpointError::RecursiveRejected(
        RecursiveCheckpointRejectReasonV2::Plonky3SecurityBudgetInvalid,
    )
}

fn ceil_log2_terms(value: u64) -> Result<u16, CheckpointError> {
    if value == 0 {
        return Err(security_budget_error());
    }
    let exponent = u64::BITS - (value - 1).leading_zeros();
    u16::try_from(exponent).map_err(|_| CheckpointError::Overflow)
}

fn derive_per_proof_bound(
    fri_bits: u16,
    hash_bits: u16,
    component_count: u16,
) -> Result<DyadicErrorBoundV2, CheckpointError> {
    if component_count == 0 {
        return Err(security_budget_error());
    }
    let weakest_component = fri_bits.min(hash_bits);
    let composition_loss = ceil_log2_terms(u64::from(component_count))?;
    let exponent = weakest_component
        .checked_sub(composition_loss)
        .ok_or_else(security_budget_error)?;
    DyadicErrorBoundV2::new(exponent)
}

fn derive_lifetime_bound(
    per_proof: DyadicErrorBoundV2,
    max_accepted_epoch_proofs: u64,
    inherited: DyadicErrorBoundV2,
) -> Result<DyadicErrorBoundV2, CheckpointError> {
    if max_accepted_epoch_proofs == 0 {
        return Err(security_budget_error());
    }
    let composed_terms = max_accepted_epoch_proofs
        .checked_add(1)
        .ok_or(CheckpointError::Overflow)?;
    let composition_loss = ceil_log2_terms(composed_terms)?;
    let exponent = per_proof
        .denominator_exponent()
        .min(inherited.denominator_exponent())
        .checked_sub(composition_loss)
        .ok_or_else(security_budget_error)?;
    DyadicErrorBoundV2::new(exponent)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Plonky3ParameterManifestV2 {
    canonical_bytes: Vec<u8>,
    digest: [u8; 32],
}

impl Plonky3ParameterManifestV2 {
    fn authority_pinned(
        security: &RecursiveSecurityBudgetManifestV2,
    ) -> Result<Self, CheckpointError> {
        security.validate()?;
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        let row = registry.row(RecursiveBoundedObjectV2::Plonky3BaseProof)?;
        let active = CheckpointConfigResolverV3::resolve_active()?;
        let identity = active.identity();
        if identity.registry_digest != registry.digest()
            || row.runtime_profile != Some(active.config().runtime_profile.identifier.as_str())
            || row.runtime_profile_generation != Some(identity.runtime_profile_generation)
            || row.runtime_profile_manifest_digest != Some(identity.runtime_profile_manifest_digest)
            || u64::from(row.authority_generation) != identity.authority_generation
            || row.parameter_generation != Some(identity.parameter_generation)
        {
            return Err(CheckpointError::Authority);
        }
        let mut bytes = Vec::with_capacity(256);
        bytes.extend_from_slice(&PLONKY3_PARAMETER_MAGIC_V2);
        put_short_str(&mut bytes, ACTIVE_PLONKY3_SOURCE_REVISION_V2)?;
        put_short_str(&mut bytes, ACTIVE_PLONKY3_CRATES_IO_VERSION_V2)?;
        put_short_str(&mut bytes, ACTIVE_PLONKY3_CIRCUIT_VERSION_V2)?;
        put_short_str(&mut bytes, "koala_bear")?;
        put_short_str(&mut bytes, "poseidon2_width16_rate8")?;
        put_short_str(&mut bytes, "batch_stark_circuit_air")?;
        bytes.push(PLONKY3_FRI_LOG_BLOWUP_V2);
        bytes.push(PLONKY3_FRI_LOG_FINAL_POLY_LEN_V2);
        bytes.push(PLONKY3_FRI_MAX_LOG_ARITY_V2);
        bytes.extend_from_slice(&PLONKY3_FRI_NUM_QUERIES_V2.to_le_bytes());
        bytes.push(PLONKY3_FRI_COMMIT_POW_BITS_V2);
        bytes.push(PLONKY3_FRI_QUERY_POW_BITS_V2);
        bytes.push(PLONKY3_CHALLENGE_EXTENSION_DEGREE_V2);
        bytes.push(PLONKY3_TRACE_EXTENSION_DEGREE_V2);
        bytes.extend_from_slice(
            &u32::try_from(PLONKY3_TABLE_MIN_HEIGHT_V2)
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(
            &u16::try_from(PLONKY3_TABLE_PUBLIC_LANES_V2)
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(
            &u16::try_from(PLONKY3_TABLE_ALU_LANES_V2)
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes(),
        );
        put_short_str(&mut bytes, &active.config().runtime_profile.identifier)?;
        bytes.extend_from_slice(&identity.runtime_profile_generation.to_le_bytes());
        bytes.extend_from_slice(&identity.runtime_profile_manifest_digest);
        bytes.extend_from_slice(&identity.registry_digest);
        bytes.extend_from_slice(&identity.config_digest);
        bytes.extend_from_slice(&identity.config_generation.to_le_bytes());
        bytes.extend_from_slice(&identity.authority_generation.to_le_bytes());
        bytes.extend_from_slice(&identity.parameter_generation.to_le_bytes());
        bytes.extend_from_slice(&identity.activation_height.to_le_bytes());
        bytes.extend_from_slice(&security.digest());
        let digest = sha256_256(
            "z00z.storage.checkpoint.plonky3.parameters.v2",
            "manifest",
            &[&bytes],
        );
        Ok(Self {
            canonical_bytes: bytes,
            digest,
        })
    }
}

/// Backend-neutral public statement bound into the exact base AIR.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plonky3BaseStatementV2 {
    canonical_bytes: Vec<u8>,
    digest: [u8; 32],
    height: u64,
    event_vector_digest: [u8; 32],
}

impl Plonky3BaseStatementV2 {
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub const fn event_vector_digest(&self) -> [u8; 32] {
        self.event_vector_digest
    }
}

/// Local-only real Plonky3 base proof.  It deliberately has no `Serialize`
/// implementation and is not a checkpoint, wallet, validator, or network
/// payload.
#[derive(Clone, PartialEq, Eq)]
struct LocalVerificationMaterialV2 {
    event_vector: Vec<u8>,
}

impl Drop for LocalVerificationMaterialV2 {
    fn drop(&mut self) {
        self.event_vector.zeroize();
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Plonky3BaseProofV2 {
    statement: Plonky3BaseStatementV2,
    parameter_digest: [u8; 32],
    security_budget_digest: [u8; 32],
    air_binding_digest: [u8; 32],
    proof_digest: [u8; 32],
    proof_bytes: Vec<u8>,
    canonical_bytes: Vec<u8>,
    local_verification_material: Option<LocalVerificationMaterialV2>,
}

impl fmt::Debug for Plonky3BaseProofV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Plonky3BaseProofV2")
            .field("statement", &self.statement)
            .field("parameter_digest", &self.parameter_digest)
            .field("security_budget_digest", &self.security_budget_digest)
            .field("air_binding_digest", &self.air_binding_digest)
            .field("proof_digest", &self.proof_digest)
            .field("proof_bytes_len", &self.proof_bytes.len())
            .field("canonical_bytes_len", &self.canonical_bytes.len())
            .field(
                "local_verification_material",
                &self
                    .local_verification_material
                    .as_ref()
                    .map(|_| "<redacted>"),
            )
            .finish()
    }
}

impl Plonky3BaseProofV2 {
    #[must_use]
    pub fn statement(&self) -> &Plonky3BaseStatementV2 {
        &self.statement
    }

    #[must_use]
    pub const fn proof_digest(&self) -> [u8; 32] {
        self.proof_digest
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Strict local decoder used only by the base verifier and mutation tests.
    pub fn decode_local(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() > PLONKY3_BASE_MAX_CANONICAL_BYTES_V2 {
            return Err(CheckpointError::Canonical);
        }
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        let preheader =
            registry.validate_preheader(bytes, RecursiveBoundedObjectV2::Plonky3BaseProof)?;
        let payload = bytes
            .get(preheader.header_len..)
            .ok_or(CheckpointError::Canonical)?;
        if payload.len() < 8 + 2 + 4 + 32 * 5 + 4 || payload[..8] != PLONKY3_BASE_MAGIC_V2 {
            return Err(CheckpointError::Canonical);
        }
        let mut cursor = 8;
        let version = take_u16(payload, &mut cursor)?;
        if version != PLONKY3_BASE_WIRE_VERSION_V2 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::UnsupportedVersion,
            ));
        }
        let statement_len =
            usize::try_from(take_u32(payload, &mut cursor)?).map_err(|_| CheckpointError::Limit)?;
        if statement_len != PLONKY3_BASE_STATEMENT_BYTES_V2 {
            return Err(CheckpointError::Canonical);
        }
        let statement_bytes = take_slice(payload, &mut cursor, statement_len)?.to_vec();
        let statement_digest = take_array::<32>(payload, &mut cursor)?;
        let parameter_digest = take_array::<32>(payload, &mut cursor)?;
        let security_budget_digest = take_array::<32>(payload, &mut cursor)?;
        let air_binding_digest = take_array::<32>(payload, &mut cursor)?;
        let proof_digest = take_array::<32>(payload, &mut cursor)?;
        let proof_len =
            usize::try_from(take_u32(payload, &mut cursor)?).map_err(|_| CheckpointError::Limit)?;
        if proof_len == 0 || proof_len > PLONKY3_BASE_MAX_PROOF_BYTES_V2 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::ProofBytesTooLarge,
            ));
        }
        let proof_bytes = take_slice(payload, &mut cursor, proof_len)?.to_vec();
        if cursor != payload.len() {
            return Err(CheckpointError::Canonical);
        }
        let statement = decode_base_statement(&statement_bytes)?;
        if statement.digest() != statement_digest
            || proof_digest != plonky3_proof_digest(&proof_bytes)
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3TranscriptMismatch,
            ));
        }
        let security = RecursiveSecurityBudgetManifestV2::authority_pinned()?;
        let parameters = Plonky3ParameterManifestV2::authority_pinned(&security)?;
        if parameter_digest != parameters.digest || security_budget_digest != security.digest() {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3TranscriptMismatch,
            ));
        }
        let (decoded_proof, remaining): (BatchStarkProof<KoalaBearConfig>, &[u8]) =
            postcard::take_from_bytes(&proof_bytes).map_err(|_| CheckpointError::Canonical)?;
        if !remaining.is_empty()
            || postcard::to_allocvec(&decoded_proof).map_err(|_| CheckpointError::Canonical)?
                != proof_bytes
            || common_binding_digest(&decoded_proof.stark_common)? != air_binding_digest
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        let decoded = Self {
            statement,
            parameter_digest,
            security_budget_digest,
            air_binding_digest,
            proof_digest,
            proof_bytes,
            canonical_bytes: bytes.to_vec(),
            local_verification_material: None,
        };
        if encode_base_proof(&decoded)? != bytes {
            return Err(CheckpointError::Canonical);
        }
        Ok(decoded)
    }

    /// Reattach the non-serializable verifier material from an unchanged
    /// locally produced proof.  This exists for exact reload/mutation
    /// verification without ever encoding the canonical transition witness.
    pub fn decode_local_with_source(bytes: &[u8], source: &Self) -> Result<Self, CheckpointError> {
        let mut decoded = Self::decode_local(bytes)?;
        if decoded.statement != source.statement
            || decoded.parameter_digest != source.parameter_digest
            || decoded.security_budget_digest != source.security_budget_digest
            || decoded.air_binding_digest != source.air_binding_digest
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3TranscriptMismatch,
            ));
        }
        decoded.local_verification_material = source.local_verification_material.clone();
        Ok(decoded)
    }
}

/// Sole public ingress for the private Plonky3 base backend.
pub struct Plonky3BaseAdapterV2;

/// Plan 07 cannot issue base proofs or verification receipts until the AIR
/// independently constrains every frozen transition family.
///
/// The current pinned P3 circuit proves canonical-vector equality and its
/// Poseidon2 binding, but it does not yet constrain replay, HJMT, uniqueness,
/// delta, journal, linkage, or the storage-owned SHA-256 relations. Treating
/// that circuit as the transition theorem would violate the Plan-07
/// anti-placeholder gate, so both adapter directions fail closed.
fn require_complete_transition_air_v2() -> Result<(), CheckpointError> {
    Err(CheckpointError::RecursiveRejected(
        RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
    ))
}

impl Plonky3BaseAdapterV2 {
    /// Run the independent native evaluator, construct the exact canonical
    /// witness vector, and generate a real pinned Batch-STARK proof.
    pub fn prove(
        transition: &mut CanonicalCheckpointTransitionV2,
        store: &SettlementStore,
    ) -> Result<Plonky3BaseProofV2, CheckpointError> {
        require_complete_transition_air_v2()?;
        let material = transition_material(transition, store)?;
        let security = RecursiveSecurityBudgetManifestV2::authority_pinned()?;
        let parameters = Plonky3ParameterManifestV2::authority_pinned(&security)?;
        let words = predicate_words(&material.statement, &material.event_vector, &parameters)?;
        let prepared = prepare_circuit(&words, Some(&material.event_vector))?;
        let air_binding_digest = common_binding_digest(prepared.data.common_data())?;
        let mut runner = prepared.circuit.runner();
        runner
            .set_private_inputs(&prepared.private_inputs)
            .map_err(|_| CheckpointError::Invariant)?;
        let traces = runner.run().map_err(|_| CheckpointError::Invariant)?;
        let mut prover = BatchStarkProver::new(prepared.config)
            .with_table_packing(prepared.table_packing.clone());
        for table in poseidon2_table_provers_d5(Poseidon2Config::KOALA_BEAR_D1_W16) {
            prover.register_table_prover(table);
        }
        let proof = prover
            .prove_all_tables(&traces, &prepared.data)
            .map_err(|_| CheckpointError::BackendVerificationFailed)?;
        prover
            .verify_all_tables::<Plonky3TraceFieldV2>(&proof)
            .map_err(|_| CheckpointError::BackendVerificationFailed)?;
        let proof_bytes = postcard::to_allocvec(&proof).map_err(|_| CheckpointError::Canonical)?;
        if proof_bytes.is_empty() || proof_bytes.len() > PLONKY3_BASE_MAX_PROOF_BYTES_V2 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::ProofSizeBudgetExceeded,
            ));
        }
        let mut result = Plonky3BaseProofV2 {
            statement: material.statement.clone(),
            parameter_digest: parameters.digest,
            security_budget_digest: security.digest(),
            air_binding_digest,
            proof_digest: plonky3_proof_digest(&proof_bytes),
            proof_bytes,
            canonical_bytes: Vec::new(),
            local_verification_material: Some(LocalVerificationMaterialV2 {
                event_vector: material.event_vector,
            }),
        };
        result.canonical_bytes = encode_base_proof(&result)?;
        Ok(result)
    }

    /// Re-evaluate the canonical transition, reconstruct the verifier-chosen
    /// AIR/common-data binding, run the actual Plonky3 verifier, and only then
    /// issue a typed local receipt.
    pub fn verify(
        transition: &mut CanonicalCheckpointTransitionV2,
        store: &SettlementStore,
        proof: &Plonky3BaseProofV2,
    ) -> Result<Plonky3BaseVerificationReceiptV2, CheckpointError> {
        require_complete_transition_air_v2()?;
        let canonical = Plonky3BaseProofV2::decode_local(proof.canonical_bytes())?;
        if canonical.statement != proof.statement
            || canonical.parameter_digest != proof.parameter_digest
            || canonical.security_budget_digest != proof.security_budget_digest
            || canonical.air_binding_digest != proof.air_binding_digest
            || canonical.proof_digest != proof.proof_digest
            || canonical.proof_bytes != proof.proof_bytes
        {
            return Err(CheckpointError::Canonical);
        }
        transition.revalidate_evidence_authority(store)?;
        if transition.checkpoint_height() != proof.statement.height() {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3TranscriptMismatch,
            ));
        }
        let material = proof.local_verification_material.as_ref().ok_or(
            CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::WitnessUnavailable,
            ),
        )?;
        let security = RecursiveSecurityBudgetManifestV2::authority_pinned()?;
        let parameters = Plonky3ParameterManifestV2::authority_pinned(&security)?;
        if proof.parameter_digest != parameters.digest
            || proof.security_budget_digest != security.digest()
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3TranscriptMismatch,
            ));
        }
        let words = predicate_words(&proof.statement, &material.event_vector, &parameters)?;
        let prepared = prepare_circuit(&words, Some(&material.event_vector))?;
        let expected_air_binding = common_binding_digest(prepared.data.common_data())?;
        if proof.air_binding_digest != expected_air_binding {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        let (decoded, remaining): (BatchStarkProof<KoalaBearConfig>, &[u8]) =
            postcard::take_from_bytes(&proof.proof_bytes)
                .map_err(|_| CheckpointError::Canonical)?;
        if !remaining.is_empty()
            || postcard::to_allocvec(&decoded).map_err(|_| CheckpointError::Canonical)?
                != proof.proof_bytes
            || common_binding_digest(&decoded.stark_common)? != expected_air_binding
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        let expected_effective_packing = TablePacking::new(1, PLONKY3_TABLE_ALU_LANES_V2)
            .with_min_trace_height(PLONKY3_TABLE_MIN_HEIGHT_V2);
        if decoded.table_packing != expected_effective_packing
            || decoded.ext_degree != usize::from(PLONKY3_TRACE_EXTENSION_DEGREE_V2)
            || decoded.w_binomial.is_some()
            || !decoded.alu_quintic_trinomial
            || decoded.non_primitives.len() != 1
            || decoded.non_primitives[0].op_type
                != p3_circuit::ops::NpoTypeId::poseidon2_perm(Poseidon2Config::KOALA_BEAR_D1_W16)
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3ProofMalformed,
            ));
        }
        let mut verifier =
            BatchStarkProver::new(prepared.config).with_table_packing(prepared.table_packing);
        for table in poseidon2_table_provers_d5(Poseidon2Config::KOALA_BEAR_D1_W16) {
            verifier.register_table_prover(table);
        }
        verifier
            .verify_all_tables::<Plonky3TraceFieldV2>(&decoded)
            .map_err(|_| CheckpointError::BackendVerificationFailed)?;
        Plonky3BaseVerificationReceiptV2::issue(VerifiedPlonky3BaseV2 {
            height: proof.statement.height(),
            statement_digest: proof.statement.digest(),
            event_vector_digest: proof.statement.event_vector_digest(),
            parameter_digest: proof.parameter_digest,
            security_budget_digest: proof.security_budget_digest,
            air_binding_digest: proof.air_binding_digest,
            proof_digest: proof.proof_digest,
        })
    }

    pub fn prove_and_verify(
        transition: &mut CanonicalCheckpointTransitionV2,
        store: &SettlementStore,
    ) -> Result<(Plonky3BaseProofV2, Plonky3BaseVerificationReceiptV2), CheckpointError> {
        let proof = Self::prove(transition, store)?;
        let receipt = Self::verify(transition, store, &proof)?;
        Ok((proof, receipt))
    }
}

pub(super) struct VerifiedPlonky3BaseV2 {
    pub(super) height: u64,
    pub(super) statement_digest: [u8; 32],
    pub(super) event_vector_digest: [u8; 32],
    pub(super) parameter_digest: [u8; 32],
    pub(super) security_budget_digest: [u8; 32],
    pub(super) air_binding_digest: [u8; 32],
    pub(super) proof_digest: [u8; 32],
}

struct TransitionMaterialV2 {
    statement: Plonky3BaseStatementV2,
    event_vector: Vec<u8>,
}

fn transition_material(
    transition: &mut CanonicalCheckpointTransitionV2,
    store: &SettlementStore,
) -> Result<TransitionMaterialV2, CheckpointError> {
    let evaluated = transition.evaluate(store)?;
    let mut event_vector = Vec::with_capacity(
        usize::try_from(evaluated.statement().declared_byte_count())
            .map_err(|_| CheckpointError::Limit)?,
    );
    event_vector.extend_from_slice(&PLONKY3_EVENT_VECTOR_MAGIC_V2);
    event_vector.extend_from_slice(&0_u64.to_le_bytes());
    let mut count = 0_u64;
    transition.replay_canonical_events(store, |event| {
        let bytes = event.canonical_bytes()?;
        let len = u32::try_from(bytes.len()).map_err(|_| CheckpointError::Limit)?;
        event_vector.extend_from_slice(&len.to_le_bytes());
        event_vector.extend_from_slice(&bytes);
        count = count.checked_add(1).ok_or(CheckpointError::Overflow)?;
        if event_vector.len() > PLONKY3_BASE_MAX_VECTOR_BYTES_V2 {
            return Err(CheckpointError::Limit);
        }
        Ok(())
    })?;
    event_vector[8..16].copy_from_slice(&count.to_le_bytes());
    let security = RecursiveSecurityBudgetManifestV2::authority_pinned()?;
    let parameters = Plonky3ParameterManifestV2::authority_pinned(&security)?;
    let statement = build_base_statement(
        transition,
        evaluated.statement(),
        &event_vector,
        &parameters,
        &security,
    )?;
    Ok(TransitionMaterialV2 {
        statement,
        event_vector,
    })
}

fn build_base_statement(
    transition: &CanonicalCheckpointTransitionV2,
    statement: RecursiveTransitionStatementV2,
    event_vector: &[u8],
    parameters: &Plonky3ParameterManifestV2,
    security: &RecursiveSecurityBudgetManifestV2,
) -> Result<Plonky3BaseStatementV2, CheckpointError> {
    let authority = transition.recursive_authority_context();
    let profile = transition.recursive_profile();
    let spec = RecursiveCircuitSpecV2::new(authority.layout(), profile)?;
    let registry = CheckpointVersionRegistryV2::authority_pinned()?;
    let event_vector_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.event-vector.v2",
        "canonical_events",
        &[event_vector],
    );
    let native_predicate_digest = sha256_256_role(
        CheckpointShaRole::Statement,
        &[
            b"z00z.recursive.v2.checkpoint-transition-consistency",
            &RecursiveTraceOpcodeV2::grammar_digest(),
            &profile.digest(),
            &spec.digest(),
        ],
    );
    let mut bytes = Vec::with_capacity(PLONKY3_BASE_STATEMENT_BYTES_V2);
    bytes.extend_from_slice(&PLONKY3_STATEMENT_MAGIC_V2);
    bytes.extend_from_slice(&PLONKY3_BASE_WIRE_VERSION_V2.to_le_bytes());
    for digest in [
        authority.digest(),
        native_predicate_digest,
        super::canonical_transition::executable_predicate_digest()?,
        profile.digest(),
        spec.digest(),
        RecursiveTraceOpcodeV2::grammar_digest(),
        registry.digest(),
        parameters.digest,
        security.digest(),
        statement.digest(),
        event_vector_digest,
    ] {
        bytes.extend_from_slice(&digest);
    }
    bytes.extend_from_slice(&statement.height().to_le_bytes());
    bytes.extend_from_slice(statement.checkpoint_id().as_bytes());
    bytes.push(u8::from(statement.predecessor().is_some()));
    bytes.extend_from_slice(
        statement
            .predecessor()
            .map(|id| id.into_bytes())
            .unwrap_or([0; 32])
            .as_slice(),
    );
    bytes.extend_from_slice(&statement.checkpoint_exec_tx_root());
    bytes.extend_from_slice(&statement.checkpoint_exec_tx_count().to_le_bytes());
    for digest in [
        statement.checkpoint_statement_digest(),
        statement.checkpoint_statement_core_digest(),
        statement.delta_root(),
        statement.witness_root(),
        statement.journal_digest(),
        statement
            .prior_recursive_output_root()
            .unwrap_or([0_u8; 32]),
        statement.checkpoint_link_digest(),
        *statement.pre_settlement_root().as_bytes(),
        *statement.post_settlement_root().as_bytes(),
        statement.pre_definition_root(),
        statement.post_definition_root(),
        statement.trace_digest(),
        statement.update_trace_digest(),
        statement.declared_work_digest(),
        statement.pre_uniqueness_context_digest(),
        statement.spent_uniqueness_precommit(),
        statement.output_uniqueness_precommit(),
    ] {
        bytes.extend_from_slice(&digest);
    }
    bytes.extend_from_slice(&statement.declared_event_count().to_le_bytes());
    bytes.extend_from_slice(&statement.declared_byte_count().to_le_bytes());
    bytes.extend_from_slice(&statement.declared_event_counts().canonical_bytes());
    bytes.extend_from_slice(&statement.consumed_event_counts().canonical_bytes());
    if bytes.len() != PLONKY3_BASE_STATEMENT_BYTES_V2 {
        return Err(CheckpointError::Invariant);
    }
    let digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.base-statement.v2",
        "statement",
        &[&bytes],
    );
    Ok(Plonky3BaseStatementV2 {
        canonical_bytes: bytes,
        digest,
        height: statement.height(),
        event_vector_digest,
    })
}

fn decode_base_statement(bytes: &[u8]) -> Result<Plonky3BaseStatementV2, CheckpointError> {
    const DIGEST_COUNT: usize = 11;
    const TRANSITION_DIGEST_COUNT: usize = 17;
    const EVENT_COUNTS_BYTES: usize = RECURSIVE_TRACE_OPCODE_COUNT_V2 * 8;
    const HEIGHT_OFFSET: usize = 8 + 2 + 32 * 11;
    const EVENT_DIGEST_OFFSET: usize = 8 + 2 + 32 * 10;
    const PREDECESSOR_MARKER_OFFSET: usize = HEIGHT_OFFSET + 8 + 32;
    const DECLARED_EVENT_COUNT_OFFSET: usize =
        PREDECESSOR_MARKER_OFFSET + 1 + 32 + 32 + 8 + 32 * TRANSITION_DIGEST_COUNT;
    const DECLARED_COUNTS_OFFSET: usize = DECLARED_EVENT_COUNT_OFFSET + 8 + 8;
    const EXACT_STATEMENT_BYTES: usize = PLONKY3_BASE_STATEMENT_BYTES_V2;
    const _: () = assert!(HEIGHT_OFFSET == 8 + 2 + 32 * DIGEST_COUNT);
    if bytes.len() != EXACT_STATEMENT_BYTES
        || bytes[..8] != PLONKY3_STATEMENT_MAGIC_V2
        || u16::from_le_bytes(
            bytes[8..10]
                .try_into()
                .map_err(|_| CheckpointError::Canonical)?,
        ) != PLONKY3_BASE_WIRE_VERSION_V2
    {
        return Err(CheckpointError::Canonical);
    }
    let predecessor_marker = bytes[PREDECESSOR_MARKER_OFFSET];
    if predecessor_marker > 1
        || (predecessor_marker == 0
            && bytes[PREDECESSOR_MARKER_OFFSET + 1..PREDECESSOR_MARKER_OFFSET + 33]
                .iter()
                .any(|byte| *byte != 0))
    {
        return Err(CheckpointError::Canonical);
    }
    let declared_event_count = u64::from_le_bytes(
        bytes[DECLARED_EVENT_COUNT_OFFSET..DECLARED_EVENT_COUNT_OFFSET + 8]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    let declared_byte_count = u64::from_le_bytes(
        bytes[DECLARED_EVENT_COUNT_OFFSET + 8..DECLARED_COUNTS_OFFSET]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    if declared_event_count == 0 || declared_byte_count == 0 {
        return Err(CheckpointError::Canonical);
    }
    let declared_counts =
        &bytes[DECLARED_COUNTS_OFFSET..DECLARED_COUNTS_OFFSET + EVENT_COUNTS_BYTES];
    let consumed_counts =
        &bytes[DECLARED_COUNTS_OFFSET + EVENT_COUNTS_BYTES..EXACT_STATEMENT_BYTES];
    if declared_counts != consumed_counts {
        return Err(CheckpointError::Canonical);
    }
    let mut count_sum = 0_u64;
    for count in declared_counts.chunks_exact(8) {
        count_sum = count_sum
            .checked_add(u64::from_le_bytes(
                count.try_into().map_err(|_| CheckpointError::Canonical)?,
            ))
            .ok_or(CheckpointError::Overflow)?;
    }
    if count_sum != declared_event_count {
        return Err(CheckpointError::Canonical);
    }
    let event_vector_digest = bytes[EVENT_DIGEST_OFFSET..EVENT_DIGEST_OFFSET + 32]
        .try_into()
        .map_err(|_| CheckpointError::Canonical)?;
    let height = u64::from_le_bytes(
        bytes[HEIGHT_OFFSET..HEIGHT_OFFSET + 8]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    if height == 0 {
        return Err(CheckpointError::Canonical);
    }
    let digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.base-statement.v2",
        "statement",
        &[bytes],
    );
    Ok(Plonky3BaseStatementV2 {
        canonical_bytes: bytes.to_vec(),
        digest,
        height,
        event_vector_digest,
    })
}

fn predicate_words(
    statement: &Plonky3BaseStatementV2,
    event_vector: &[u8],
    parameters: &Plonky3ParameterManifestV2,
) -> Result<Vec<u16>, CheckpointError> {
    if decode_base_statement(statement.canonical_bytes())? != *statement {
        return Err(CheckpointError::Canonical);
    }
    validate_event_vector(statement, event_vector)?;
    let mut bytes = Vec::with_capacity(
        statement
            .canonical_bytes()
            .len()
            .checked_add(event_vector.len())
            .and_then(|value| value.checked_add(parameters.canonical_bytes.len()))
            .and_then(|value| value.checked_add(64))
            .ok_or(CheckpointError::Overflow)?,
    );
    bytes.extend_from_slice(PLONKY3_PREDICATE_VECTOR_LABEL_V2);
    bytes.extend_from_slice(
        &u64::try_from(statement.canonical_bytes().len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(statement.canonical_bytes());
    bytes.extend_from_slice(
        &u64::try_from(event_vector.len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(event_vector);
    bytes.extend_from_slice(
        &u64::try_from(parameters.canonical_bytes.len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&parameters.canonical_bytes);
    bytes.push(1);
    if bytes.len() % 2 != 0 {
        bytes.push(0);
    }
    let mut words: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let word_count = u64::try_from(bytes.len() / 2).map_err(|_| CheckpointError::Limit)?;
    words.extend(
        word_count
            .to_le_bytes()
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]])),
    );
    while words.len() % 8 != 0 {
        words.push(0);
    }
    Ok(words)
}

fn validate_event_vector(
    statement: &Plonky3BaseStatementV2,
    event_vector: &[u8],
) -> Result<(), CheckpointError> {
    if event_vector.len() < 16
        || event_vector.len() > PLONKY3_BASE_MAX_VECTOR_BYTES_V2
        || event_vector[..8] != PLONKY3_EVENT_VECTOR_MAGIC_V2
    {
        return Err(CheckpointError::Canonical);
    }
    let declared_count = u64::from_le_bytes(
        event_vector[8..16]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    if declared_count == 0 {
        return Err(CheckpointError::Canonical);
    }
    let profile = RecursiveCircuitProfileV2::authority_pinned();
    let mut cursor = 16_usize;
    let mut consumed = 0_u64;
    while cursor < event_vector.len() {
        let event_len = usize::try_from(take_u32(event_vector, &mut cursor)?)
            .map_err(|_| CheckpointError::Limit)?;
        let event_bytes = take_slice(event_vector, &mut cursor, event_len)?;
        let event = RecursiveTraceEventV2::decode_canonical(event_bytes, &profile)?;
        if event.canonical_bytes()? != event_bytes {
            return Err(CheckpointError::Canonical);
        }
        consumed = consumed.checked_add(1).ok_or(CheckpointError::Overflow)?;
        if consumed > declared_count {
            return Err(CheckpointError::Canonical);
        }
    }
    if cursor != event_vector.len() || consumed != declared_count {
        return Err(CheckpointError::Canonical);
    }
    let event_vector_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.event-vector.v2",
        "canonical_events",
        &[event_vector],
    );
    if event_vector_digest != statement.event_vector_digest() {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3TranscriptMismatch,
        ));
    }
    Ok(())
}

struct PreparedCircuitV2 {
    circuit: Circuit<Plonky3TraceFieldV2>,
    private_inputs: Vec<Plonky3TraceFieldV2>,
    config: KoalaBearConfig,
    data: CircuitProverData<KoalaBearConfig>,
    table_packing: TablePacking,
}

fn circuit_xor_bit(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    left: ExprId,
    right: ExprId,
    two: ExprId,
) -> ExprId {
    let product = builder.mul(left, right);
    let doubled = builder.mul(two, product);
    let sum = builder.add(left, right);
    builder.sub(sum, doubled)
}

fn circuit_xor3_bit(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    first: ExprId,
    second: ExprId,
    third: ExprId,
    two: ExprId,
) -> ExprId {
    let pair = circuit_xor_bit(builder, first, second, two);
    circuit_xor_bit(builder, pair, third, two)
}

fn circuit_majority_bit(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    first: ExprId,
    second: ExprId,
    third: ExprId,
    two: ExprId,
) -> ExprId {
    let first_second = builder.mul(first, second);
    let first_third = builder.mul(first, third);
    let second_third = builder.mul(second, third);
    let triple = builder.mul(first_second, third);
    let first_sum = builder.add(first_second, first_third);
    let pair_sum = builder.add(first_sum, second_third);
    let doubled_triple = builder.mul(two, triple);
    builder.sub(pair_sum, doubled_triple)
}

fn circuit_choose_bit(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    selector: ExprId,
    when_true: ExprId,
    when_false: ExprId,
    one: ExprId,
) -> ExprId {
    let selected_true = builder.mul(selector, when_true);
    let not_selector = builder.sub(one, selector);
    let selected_false = builder.mul(not_selector, when_false);
    builder.add(selected_true, selected_false)
}

fn circuit_word_rotate_right(word: &Plonky3WordBitsV2, shift: usize) -> Plonky3WordBitsV2 {
    core::array::from_fn(|bit| word[(bit + shift) % 32])
}

fn circuit_word_shift_right(
    word: &Plonky3WordBitsV2,
    shift: usize,
    zero: ExprId,
) -> Plonky3WordBitsV2 {
    core::array::from_fn(|bit| {
        bit.checked_add(shift)
            .filter(|source| *source < 32)
            .map(|source| word[source])
            .unwrap_or(zero)
    })
}

fn circuit_word_xor3(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    first: &Plonky3WordBitsV2,
    second: &Plonky3WordBitsV2,
    third: &Plonky3WordBitsV2,
    two: ExprId,
) -> Plonky3WordBitsV2 {
    core::array::from_fn(|bit| {
        circuit_xor3_bit(builder, first[bit], second[bit], third[bit], two)
    })
}

fn circuit_word_add2(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    left: &Plonky3WordBitsV2,
    right: &Plonky3WordBitsV2,
    zero: ExprId,
    two: ExprId,
) -> Plonky3WordBitsV2 {
    let mut carry = zero;
    core::array::from_fn(|bit| {
        let pair = circuit_xor_bit(builder, left[bit], right[bit], two);
        let sum = circuit_xor_bit(builder, pair, carry, two);
        carry = circuit_majority_bit(builder, left[bit], right[bit], carry, two);
        sum
    })
}

fn circuit_word_add_many(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    words: &[Plonky3WordBitsV2],
    zero: ExprId,
    two: ExprId,
) -> Plonky3WordBitsV2 {
    words.iter().fold([zero; 32], |sum, word| {
        circuit_word_add2(builder, &sum, word, zero, two)
    })
}

fn circuit_constant_word(
    value: u32,
    zero: ExprId,
    one: ExprId,
) -> Plonky3WordBitsV2 {
    core::array::from_fn(|bit| {
        if (value >> bit) & 1 == 0 {
            zero
        } else {
            one
        }
    })
}

fn circuit_word_from_be_bytes(bytes: &[[ExprId; 8]]) -> Result<Plonky3WordBitsV2, CheckpointError> {
    if bytes.len() != 4 {
        return Err(CheckpointError::Invariant);
    }
    Ok(core::array::from_fn(|bit| {
        let byte_from_end = bit / 8;
        let bit_in_byte = bit % 8;
        bytes[3 - byte_from_end][bit_in_byte]
    }))
}

fn circuit_sha256_compress(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    block: &[[ExprId; 8]],
    chaining_before: &[Plonky3WordBitsV2; 8],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<[Plonky3WordBitsV2; 8], CheckpointError> {
    if block.len() != 64 {
        return Err(CheckpointError::Invariant);
    }
    let mut schedule = Vec::with_capacity(64);
    for bytes in block.chunks_exact(4) {
        schedule.push(circuit_word_from_be_bytes(bytes)?);
    }
    for index in 16..64 {
        let rotate_7 = circuit_word_rotate_right(&schedule[index - 15], 7);
        let rotate_18 = circuit_word_rotate_right(&schedule[index - 15], 18);
        let shift_3 = circuit_word_shift_right(&schedule[index - 15], 3, zero);
        let sigma_0 = circuit_word_xor3(builder, &rotate_7, &rotate_18, &shift_3, two);
        let rotate_17 = circuit_word_rotate_right(&schedule[index - 2], 17);
        let rotate_19 = circuit_word_rotate_right(&schedule[index - 2], 19);
        let shift_10 = circuit_word_shift_right(&schedule[index - 2], 10, zero);
        let sigma_1 = circuit_word_xor3(builder, &rotate_17, &rotate_19, &shift_10, two);
        schedule.push(circuit_word_add_many(
            builder,
            &[
                schedule[index - 16],
                sigma_0,
                schedule[index - 7],
                sigma_1,
            ],
            zero,
            two,
        ));
    }

    let mut state = *chaining_before;
    for (round, round_constant) in SHA256_ROUND_CONSTANTS_V2.into_iter().enumerate() {
        let sigma_1 = circuit_word_xor3(
            builder,
            &circuit_word_rotate_right(&state[4], 6),
            &circuit_word_rotate_right(&state[4], 11),
            &circuit_word_rotate_right(&state[4], 25),
            two,
        );
        let choose = core::array::from_fn(|bit| {
            circuit_choose_bit(builder, state[4][bit], state[5][bit], state[6][bit], one)
        });
        let round_constant = circuit_constant_word(round_constant, zero, one);
        let temp_1 = circuit_word_add_many(
            builder,
            &[state[7], sigma_1, choose, round_constant, schedule[round]],
            zero,
            two,
        );
        let sigma_0 = circuit_word_xor3(
            builder,
            &circuit_word_rotate_right(&state[0], 2),
            &circuit_word_rotate_right(&state[0], 13),
            &circuit_word_rotate_right(&state[0], 22),
            two,
        );
        let majority = core::array::from_fn(|bit| {
            circuit_majority_bit(builder, state[0][bit], state[1][bit], state[2][bit], two)
        });
        let temp_2 = circuit_word_add2(builder, &sigma_0, &majority, zero, two);
        state = [
            circuit_word_add2(builder, &temp_1, &temp_2, zero, two),
            state[0],
            state[1],
            state[2],
            circuit_word_add2(builder, &state[3], &temp_1, zero, two),
            state[4],
            state[5],
            state[6],
        ];
    }
    Ok(core::array::from_fn(|index| {
        circuit_word_add2(
            builder,
            &chaining_before[index],
            &state[index],
            zero,
            two,
        )
    }))
}

fn constrain_sha_control_blocks(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    event_vector: &[u8],
    predicate_byte_bits: &[[ExprId; 8]],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<usize, CheckpointError> {
    let event_vector_offset = PLONKY3_PREDICATE_VECTOR_LABEL_V2
        .len()
        .checked_add(8)
        .and_then(|offset| offset.checked_add(PLONKY3_BASE_STATEMENT_BYTES_V2))
        .and_then(|offset| offset.checked_add(8))
        .ok_or(CheckpointError::Overflow)?;
    let event_vector_end = event_vector_offset
        .checked_add(event_vector.len())
        .ok_or(CheckpointError::Overflow)?;
    if predicate_byte_bits.len() < event_vector_end
        || event_vector.len() < 16
        || event_vector[..8] != PLONKY3_EVENT_VECTOR_MAGIC_V2
    {
        return Err(CheckpointError::Invariant);
    }
    let event_bits = &predicate_byte_bits[event_vector_offset..event_vector_end];
    let profile = RecursiveCircuitProfileV2::authority_pinned();
    let mut cursor = 16_usize;
    let mut constrained = 0_usize;
    while cursor < event_vector.len() {
        let event_len = usize::try_from(take_u32(event_vector, &mut cursor)?)
            .map_err(|_| CheckpointError::Limit)?;
        let event_start = cursor;
        let event_bytes = take_slice(event_vector, &mut cursor, event_len)?;
        let event = RecursiveTraceEventV2::decode_canonical(event_bytes, &profile)?;
        if event.opcode() != RecursiveTraceOpcodeV2::ShaBlock {
            continue;
        }
        let control = decode_hash_control(&event)?;
        let block = control.block.ok_or(CheckpointError::Invariant)?;
        let common_bytes = event
            .payload()
            .len()
            .checked_sub(HASH_CONTROL_BLOCK_BYTES_V2)
            .ok_or(CheckpointError::Invariant)?;
        let payload_start = event_start
            .checked_add(TRACE_EVENT_HEADER_BYTES_V2)
            .ok_or(CheckpointError::Overflow)?;
        let block_start = payload_start
            .checked_add(common_bytes)
            .and_then(|offset| offset.checked_add(16))
            .ok_or(CheckpointError::Overflow)?;
        let before_start = block_start.checked_add(64).ok_or(CheckpointError::Overflow)?;
        let after_start = before_start.checked_add(32).ok_or(CheckpointError::Overflow)?;
        let block_bits = event_bits
            .get(block_start..block_start + 64)
            .ok_or(CheckpointError::Invariant)?;
        let before_bits = event_bits
            .get(before_start..before_start + 32)
            .ok_or(CheckpointError::Invariant)?;
        let after_bits = event_bits
            .get(after_start..after_start + 32)
            .ok_or(CheckpointError::Invariant)?;
        let chaining_before: [Plonky3WordBitsV2; 8] = core::array::from_fn(|word| {
            circuit_word_from_be_bytes(&before_bits[word * 4..word * 4 + 4])
                .expect("fixed SHA state word width")
        });
        let chaining_after: [Plonky3WordBitsV2; 8] = core::array::from_fn(|word| {
            circuit_word_from_be_bytes(&after_bits[word * 4..word * 4 + 4])
                .expect("fixed SHA state word width")
        });
        let computed =
            circuit_sha256_compress(builder, block_bits, &chaining_before, zero, one, two)?;
        for (computed_word, claimed_word) in computed.iter().zip(chaining_after.iter()) {
            for (computed_bit, claimed_bit) in computed_word.iter().zip(claimed_word.iter()) {
                builder.connect(*computed_bit, *claimed_bit);
            }
        }
        if block.block != event.payload()[common_bytes + 16..common_bytes + 80]
            || block
                .chaining_before
                .iter()
                .flat_map(|word| word.to_be_bytes())
                .ne(event.payload()[common_bytes + 80..common_bytes + 112].iter().copied())
            || block
                .chaining_after
                .iter()
                .flat_map(|word| word.to_be_bytes())
                .ne(event.payload()[common_bytes + 112..common_bytes + 144].iter().copied())
        {
            return Err(CheckpointError::Invariant);
        }
        constrained = constrained
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
    }
    if cursor != event_vector.len() {
        return Err(CheckpointError::Canonical);
    }
    Ok(constrained)
}

fn prepare_circuit(
    words: &[u16],
    event_vector: Option<&[u8]>,
) -> Result<PreparedCircuitV2, CheckpointError> {
    if words.is_empty() || words.len() % 8 != 0 {
        return Err(CheckpointError::Invariant);
    }
    let expected_hash = poseidon_vector_hash(words);
    let mut builder = CircuitBuilder::<Plonky3TraceFieldV2>::new();
    builder.enable_poseidon2_perm_base::<KoalaBearD1Width16, _>(
        generate_poseidon2_trace::<Plonky3TraceFieldV2, KoalaBearD1Width16>,
        LiftPoseidonToQuinticV2::new(default_koalabear_poseidon2_16()),
    );
    let zero = builder.alloc_const(lift_koala(KoalaBear::ZERO), "zero");
    let one = builder.alloc_const(lift_koala(KoalaBear::ONE), "one");
    let two = builder.alloc_const(lift_koala(KoalaBear::from_u64(2)), "two");
    let mut private_inputs = Vec::with_capacity(words.len() * 17);
    let mut word_exprs = Vec::with_capacity(words.len());
    let mut predicate_byte_bits = Vec::with_capacity(words.len() * 2);
    for word in words {
        let word_expr = builder.alloc_private_input("canonical_u16_limb");
        private_inputs.push(lift_koala(KoalaBear::from_u64(u64::from(*word))));
        let expected = builder.alloc_const(
            lift_koala(KoalaBear::from_u64(u64::from(*word))),
            "expected_u16",
        );
        builder.connect(word_expr, expected);
        let mut recomposed = zero;
        let mut bits = [zero; 16];
        for bit_index in 0..16 {
            let bit = builder.alloc_private_input("canonical_u16_bit");
            let value = u64::from((*word >> bit_index) & 1);
            private_inputs.push(lift_koala(KoalaBear::from_u64(value)));
            builder.assert_bool(bit);
            let weight = builder.alloc_const(
                lift_koala(KoalaBear::from_u64(1_u64 << bit_index)),
                "u16_bit_weight",
            );
            let term = builder.mul(bit, weight);
            recomposed = builder.add(recomposed, term);
            bits[bit_index] = bit;
        }
        builder.connect(word_expr, recomposed);
        word_exprs.push(word_expr);
        predicate_byte_bits.push(core::array::from_fn(|bit| bits[bit]));
        predicate_byte_bits.push(core::array::from_fn(|bit| bits[8 + bit]));
    }
    if let Some(event_vector) = event_vector {
        let constrained = constrain_sha_control_blocks(
            &mut builder,
            event_vector,
            &predicate_byte_bits,
            zero,
            one,
            two,
        )?;
        if constrained == 0 {
            return Err(CheckpointError::Invariant);
        }
    }
    let mut final_outputs = None;
    let chunk_count = word_exprs.len() / 8;
    for (index, chunk) in word_exprs.chunks_exact(8).enumerate() {
        let mut inputs = [None; 16];
        for (slot, expr) in inputs.iter_mut().take(8).zip(chunk.iter().copied()) {
            *slot = Some(expr);
        }
        let is_last = index + 1 == chunk_count;
        let (_, outputs) = builder
            .add_poseidon2_perm_base(&Poseidon2PermCallBase {
                config: Poseidon2Config::KOALA_BEAR_D1_W16,
                new_start: index == 0,
                inputs,
                out_ctl: [is_last; 8],
                return_all_outputs: false,
                absorb_len: 0,
            })
            .map_err(|_| CheckpointError::Invariant)?;
        if is_last {
            final_outputs = Some(outputs);
        }
    }
    let final_outputs = final_outputs.ok_or(CheckpointError::Invariant)?;
    for (output, expected) in final_outputs.into_iter().zip(expected_hash) {
        let output = output.ok_or(CheckpointError::Invariant)?;
        let expected = builder.alloc_const(lift_koala(expected), "poseidon2_transcript_output");
        builder.connect(output, expected);
    }
    let circuit = builder.build().map_err(|_| CheckpointError::Invariant)?;
    let table_packing =
        TablePacking::new(PLONKY3_TABLE_PUBLIC_LANES_V2, PLONKY3_TABLE_ALU_LANES_V2)
            .with_min_trace_height(PLONKY3_TABLE_MIN_HEIGHT_V2);
    let config = hardened_koala_bear_config();
    let preprocessors: Vec<Box<dyn NpoPreprocessor<KoalaBear>>> =
        vec![Box::new(Poseidon2Preprocessor)];
    let air_builders = poseidon2_air_builders_d5::<KoalaBearConfig>();
    let (airs_degrees, primitive_columns, non_primitive_columns) =
        get_airs_and_degrees_with_prep::<KoalaBearConfig, _, 5>(
            &circuit,
            &table_packing,
            &preprocessors,
            &air_builders,
            ConstraintProfile::Standard,
        )
        .map_err(|_| CheckpointError::Invariant)?;
    let (airs, degrees): (Vec<_>, Vec<_>) = airs_degrees.into_iter().unzip();
    let prover_data = ProverData::from_airs_and_degrees(&config, &airs, &degrees);
    let data = CircuitProverData::new(prover_data, primitive_columns, non_primitive_columns);
    Ok(PreparedCircuitV2 {
        circuit,
        private_inputs,
        config,
        data,
        table_packing,
    })
}

fn hardened_koala_bear_config() -> KoalaBearConfig {
    type HashV2 = PaddingFreeSponge<Poseidon2KoalaBear<16>, 16, 8, 8>;
    type CompressV2 = TruncatedPermutation<Poseidon2KoalaBear<16>, 2, 8, 16>;
    type ValMmcsV2 = MerkleTreeMmcs<KoalaBear, KoalaBear, HashV2, CompressV2, 2, 8>;
    type ChallengeV2 = BinomialExtensionField<KoalaBear, 4>;
    type ChallengeMmcsV2 = ExtensionMmcs<KoalaBear, ChallengeV2, ValMmcsV2>;
    type PcsV2 = TwoAdicFriPcs<KoalaBear, Radix2DitParallel<KoalaBear>, ValMmcsV2, ChallengeMmcsV2>;
    type ChallengerV2 = DuplexChallenger<KoalaBear, Poseidon2KoalaBear<16>, 16, 8>;
    type ExactConfigV2 = StarkConfig<PcsV2, ChallengeV2, ChallengerV2>;
    let permutation = default_koalabear_poseidon2_16();
    let hash = HashV2::new(permutation.clone());
    let compress = CompressV2::new(permutation.clone());
    let value_mmcs = ValMmcsV2::new(hash, compress, 3);
    let challenge_mmcs = ChallengeMmcsV2::new(value_mmcs.clone());
    let fri = FriParameters {
        log_blowup: usize::from(PLONKY3_FRI_LOG_BLOWUP_V2),
        log_final_poly_len: usize::from(PLONKY3_FRI_LOG_FINAL_POLY_LEN_V2),
        max_log_arity: usize::from(PLONKY3_FRI_MAX_LOG_ARITY_V2),
        num_queries: usize::from(PLONKY3_FRI_NUM_QUERIES_V2),
        commit_proof_of_work_bits: usize::from(PLONKY3_FRI_COMMIT_POW_BITS_V2),
        query_proof_of_work_bits: usize::from(PLONKY3_FRI_QUERY_POW_BITS_V2),
        mmcs: challenge_mmcs,
    };
    let pcs = PcsV2::new(Radix2DitParallel::default(), value_mmcs, fri);
    let challenger = ChallengerV2::new(permutation);
    let config: ExactConfigV2 = StarkConfig::new(pcs, challenger);
    config
}

#[derive(Clone)]
struct LiftPoseidonToQuinticV2<P> {
    permutation: P,
    marker: PhantomData<KoalaBear>,
}

impl<P> LiftPoseidonToQuinticV2<P> {
    const fn new(permutation: P) -> Self {
        Self {
            permutation,
            marker: PhantomData,
        }
    }
}

impl<P> Permutation<[Plonky3TraceFieldV2; 16]> for LiftPoseidonToQuinticV2<P>
where
    KoalaBear: QuinticTrinomialExtendable,
    P: Permutation<[KoalaBear; 16]>,
{
    fn permute(&self, input: [Plonky3TraceFieldV2; 16]) -> [Plonky3TraceFieldV2; 16] {
        let base = core::array::from_fn(|index| {
            <Plonky3TraceFieldV2 as BasedVectorSpace<KoalaBear>>::as_basis_coefficients_slice(
                &input[index],
            )[0]
        });
        let output = self.permutation.permute(base);
        core::array::from_fn(|index| lift_koala(output[index]))
    }
}

fn lift_koala(value: KoalaBear) -> Plonky3TraceFieldV2 {
    Plonky3TraceFieldV2::new([
        value,
        KoalaBear::ZERO,
        KoalaBear::ZERO,
        KoalaBear::ZERO,
        KoalaBear::ZERO,
    ])
}

fn poseidon_vector_hash(words: &[u16]) -> [KoalaBear; 8] {
    let permutation = default_koalabear_poseidon2_16();
    let mut state = [KoalaBear::ZERO; 16];
    for chunk in words.chunks_exact(8) {
        for (slot, word) in state.iter_mut().take(8).zip(chunk) {
            *slot = KoalaBear::from_u64(u64::from(*word));
        }
        state = permutation.permute(state);
    }
    state[..8].try_into().expect("fixed Poseidon2 rate")
}

fn common_binding_digest(
    common: &p3_batch_stark::CommonData<KoalaBearConfig>,
) -> Result<[u8; 32], CheckpointError> {
    let preprocessed = common
        .preprocessed
        .as_ref()
        .ok_or(CheckpointError::Invariant)?;
    let mut bytes = Vec::new();
    let commitment =
        postcard::to_allocvec(&preprocessed.commitment).map_err(|_| CheckpointError::Canonical)?;
    bytes.extend_from_slice(
        &u32::try_from(commitment.len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&commitment);
    bytes.extend_from_slice(
        &u32::try_from(preprocessed.instances.len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    for instance in &preprocessed.instances {
        match instance {
            Some(meta) => {
                bytes.push(1);
                for value in [meta.matrix_index, meta.width, meta.degree_bits] {
                    bytes.extend_from_slice(
                        &u64::try_from(value)
                            .map_err(|_| CheckpointError::Limit)?
                            .to_le_bytes(),
                    );
                }
            }
            None => bytes.push(0),
        }
    }
    bytes.extend_from_slice(
        &u32::try_from(preprocessed.matrix_to_instance.len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    for value in &preprocessed.matrix_to_instance {
        bytes.extend_from_slice(
            &u64::try_from(*value)
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes(),
        );
    }
    Ok(sha256_256(
        "z00z.storage.checkpoint.plonky3.air-binding.v2",
        "preprocessed_common",
        &[&bytes],
    ))
}

fn encode_base_proof(proof: &Plonky3BaseProofV2) -> Result<Vec<u8>, CheckpointError> {
    let mut payload = Vec::with_capacity(
        proof
            .statement
            .canonical_bytes()
            .len()
            .checked_add(proof.proof_bytes.len())
            .and_then(|value| value.checked_add(192))
            .ok_or(CheckpointError::Overflow)?,
    );
    payload.extend_from_slice(&PLONKY3_BASE_MAGIC_V2);
    payload.extend_from_slice(&PLONKY3_BASE_WIRE_VERSION_V2.to_le_bytes());
    payload.extend_from_slice(
        &u32::try_from(proof.statement.canonical_bytes().len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    payload.extend_from_slice(proof.statement.canonical_bytes());
    for digest in [
        proof.statement.digest(),
        proof.parameter_digest,
        proof.security_budget_digest,
        proof.air_binding_digest,
        proof.proof_digest,
    ] {
        payload.extend_from_slice(&digest);
    }
    payload.extend_from_slice(
        &u32::try_from(proof.proof_bytes.len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    payload.extend_from_slice(&proof.proof_bytes);
    let registry = CheckpointVersionRegistryV2::authority_pinned()?;
    let preheader =
        registry.encode_preheader(RecursiveBoundedObjectV2::Plonky3BaseProof, payload.len())?;
    let mut bytes = Vec::with_capacity(
        preheader
            .len()
            .checked_add(payload.len())
            .ok_or(CheckpointError::Overflow)?,
    );
    bytes.extend_from_slice(&preheader);
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

fn plonky3_proof_digest(bytes: &[u8]) -> [u8; 32] {
    sha256_256(
        "z00z.storage.checkpoint.plonky3.base-proof.v2",
        "proof",
        &[bytes],
    )
}

fn put_short_str(bytes: &mut Vec<u8>, value: &str) -> Result<(), CheckpointError> {
    let len = u16::try_from(value.len()).map_err(|_| CheckpointError::Limit)?;
    bytes.extend_from_slice(&len.to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn take_slice<'a>(
    bytes: &'a [u8],
    cursor: &mut usize,
    len: usize,
) -> Result<&'a [u8], CheckpointError> {
    let end = cursor.checked_add(len).ok_or(CheckpointError::Overflow)?;
    let value = bytes.get(*cursor..end).ok_or(CheckpointError::Canonical)?;
    *cursor = end;
    Ok(value)
}

fn take_array<const N: usize>(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], CheckpointError> {
    take_slice(bytes, cursor, N)?
        .try_into()
        .map_err(|_| CheckpointError::Canonical)
}

fn take_u16(bytes: &[u8], cursor: &mut usize) -> Result<u16, CheckpointError> {
    Ok(u16::from_le_bytes(take_array::<2>(bytes, cursor)?))
}

fn take_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, CheckpointError> {
    Ok(u32::from_le_bytes(take_array::<4>(bytes, cursor)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_budget_is_upward_rounded_and_finite() {
        let manifest = RecursiveSecurityBudgetManifestV2::authority_pinned().unwrap();
        assert_eq!(
            manifest.per_proof_bound.denominator_exponent(),
            PLONKY3_PER_PROOF_BOUND_BITS_V2
        );
        assert_eq!(
            manifest.max_accepted_epoch_proofs,
            PLONKY3_MAX_ACCEPTED_EPOCH_PROOFS_V2
        );
        assert!(manifest.lifetime_residual_bits() >= PLONKY3_MINIMUM_RESIDUAL_BITS_V2);
        assert_ne!(manifest.digest(), [0; 32]);
    }

    #[test]
    fn test_security_budget_derivation_rejects_every_input_drift() {
        assert_eq!(ceil_log2_terms(1).unwrap(), 0);
        assert_eq!(ceil_log2_terms(3).unwrap(), 2);
        assert_eq!(ceil_log2_terms(1 << 20).unwrap(), 20);
        assert_eq!(ceil_log2_terms((1 << 20) + 1).unwrap(), 21);
        assert_eq!(
            derive_per_proof_bound(124, 128, 2)
                .unwrap()
                .denominator_exponent(),
            123
        );
        assert_eq!(
            derive_lifetime_bound(
                DyadicErrorBoundV2::new(123).unwrap(),
                1 << 20,
                DyadicErrorBoundV2::new(128).unwrap(),
            )
            .unwrap()
            .denominator_exponent(),
            102
        );
        assert!(derive_per_proof_bound(124, 128, 0).is_err());
        assert!(derive_lifetime_bound(
            DyadicErrorBoundV2::new(123).unwrap(),
            0,
            DyadicErrorBoundV2::new(128).unwrap(),
        )
        .is_err());
        assert!(matches!(
            derive_lifetime_bound(
                DyadicErrorBoundV2::new(123).unwrap(),
                u64::MAX,
                DyadicErrorBoundV2::new(128).unwrap(),
            ),
            Err(CheckpointError::Overflow)
        ));

        let baseline = RecursiveSecurityBudgetManifestV2::authority_pinned().unwrap();
        let mutations: [fn(&mut RecursiveSecurityBudgetManifestV2); 19] = [
            |value| value.generation += 1,
            |value| value.parameter_generation += 1,
            |value| value.base_field_bits -= 1,
            |value| value.challenge_extension_degree -= 1,
            |value| value.fri_log_blowup += 1,
            |value| value.fri_num_queries -= 1,
            |value| value.fri_commit_pow_bits += 1,
            |value| value.fri_query_pow_bits -= 1,
            |value| value.fri_conjectured_bits -= 1,
            |value| value.hash_collision_bits -= 1,
            |value| value.component_count = 0,
            |value| value.recursion_depth += 1,
            |value| value.composition_rule_generation += 1,
            |value| value.per_proof_bound = DyadicErrorBoundV2::new(127).unwrap(),
            |value| value.max_accepted_epoch_proofs = 0,
            |value| value.max_accepted_epoch_proofs = u64::MAX,
            |value| value.inherited_bound = None,
            |value| value.lifetime_bound = DyadicErrorBoundV2::new(99).unwrap(),
            |value| value.minimum_residual_bits = 106,
        ];
        for mutate in mutations {
            let mut changed = baseline.clone();
            mutate(&mut changed);
            assert!(changed.validate().is_err());
        }
    }

    #[test]
    fn test_poseidon_vector_hash_binds_order_and_length() {
        let a = vec![1_u16, 2, 3, 4, 5, 6, 7, 8];
        let mut b = a.clone();
        b.swap(1, 2);
        assert_ne!(poseidon_vector_hash(&a), poseidon_vector_hash(&b));
        b.push(0);
        while b.len() % 8 != 0 {
            b.push(0);
        }
        assert_ne!(poseidon_vector_hash(&a), poseidon_vector_hash(&b));
    }

    #[test]
    fn test_real_batch_stark_roundtrip_small() {
        let words = [1_u16, 2, 3, 4, 5, 6, 7, 8];
        let prepared = prepare_circuit(&words, None).unwrap();
        let expected_binding = common_binding_digest(prepared.data.common_data()).unwrap();
        let mut runner = prepared.circuit.runner();
        runner.set_private_inputs(&prepared.private_inputs).unwrap();
        let traces = runner.run().unwrap();
        let mut prover =
            BatchStarkProver::new(prepared.config).with_table_packing(prepared.table_packing);
        for table in poseidon2_table_provers_d5(Poseidon2Config::KOALA_BEAR_D1_W16) {
            prover.register_table_prover(table);
        }
        let proof = prover.prove_all_tables(&traces, &prepared.data).unwrap();
        assert_eq!(
            common_binding_digest(&proof.stark_common).unwrap(),
            expected_binding
        );
        assert_eq!(
            proof.table_packing,
            TablePacking::new(1, PLONKY3_TABLE_ALU_LANES_V2)
                .with_min_trace_height(PLONKY3_TABLE_MIN_HEIGHT_V2)
        );
        prover
            .verify_all_tables::<Plonky3TraceFieldV2>(&proof)
            .unwrap();
    }

    #[test]
    fn test_incomplete_transition_air_cannot_issue_backend_evidence() {
        assert!(matches!(
            require_complete_transition_air_v2(),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing
            ))
        ));
    }
}
