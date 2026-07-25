//! Private Plonky3 owner for the recursive-checkpoint V2 base relation.
//!
//! The public surface is re-exported only by `checkpoint::recursive_v2`.  No
//! Plonky3 field, AIR, PCS, proof, or configuration type crosses that facade.
//! This base STARK is an internal leaf of the canonical hash/FRI recursion
//! chain. Nova is retained only as a differential oracle and is never a proof
//! wrapper or recursive ancestor. End-to-end post-quantum authority additionally
//! requires every outer layer to pass the pinned hash/FRI ancestry gate.

use core::fmt;

use p3_batch_stark::ProverData;
use p3_challenger::{HashChallenger, SerializingChallenger32};
use p3_circuit::ops::poseidon2_perm::Poseidon2PermCall;
use p3_circuit::ops::{generate_poseidon2_trace, Poseidon2Config};
use p3_circuit::{Circuit, CircuitBuilder, ExprId};
use p3_circuit_prover::batch_stark_prover::{
    poseidon2_air_builders_d4, poseidon2_table_provers_d4, Poseidon2Preprocessor,
};
use p3_circuit_prover::common::{get_airs_and_degrees_with_prep, NpoPreprocessor};
use p3_circuit_prover::{
    BatchStarkProof, BatchStarkProver, CircuitProverData, ConstraintProfile, TablePacking,
};
use p3_commit::ExtensionMmcs;
use p3_dft::Radix2DitParallel;
use p3_field::extension::BinomialExtensionField;
use p3_field::{BasedVectorSpace, PrimeCharacteristicRing};
use p3_fri::{FriParameters, TwoAdicFriPcs};
use p3_koala_bear::{default_koalabear_poseidon2_16, KoalaBear};
use p3_merkle_tree::MerkleTreeMmcs;
use p3_poseidon2_circuit_air::KoalaBearD4Width16;
use p3_symmetric::{
    CompressionFunctionFromHasher, CryptographicHasher, Permutation, SerializingHasher,
};
use p3_uni_stark::StarkConfig;
use sha2::{Digest, Sha512};
use z00z_crypto::{
    sha256_256, sha256_256_role, CheckpointSha256BlockStreamV2, CheckpointShaRole, SHA256_IV_V2,
};
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
    recursive_semantics::{
        decode_flow_header, decode_flow_item, decode_hierarchy_promotion_fields, decode_net_effect,
        decode_typed_checkpoint_commitment, decode_uniqueness_precommit,
        decode_uniqueness_sorted_row, encode_net_effect, NetEffectV2,
        TypedCheckpointCommitmentKindV2, UniquenessListKindV2, UniquenessPassV2,
        UniquenessSemanticRowV2, UniquenessSetKindV2, NET_MERGE_BYTES_V2,
        TYPED_CHECKPOINT_COMMITMENT_BYTES_V2, UNIQUENESS_CHALLENGE_BYTES_V2,
        UNIQUENESS_PRECOMMIT_BYTES_V2, UNIQUENESS_PRECOMMIT_LABEL_V2,
        UNIQUENESS_SEMANTIC_ROW_BYTES_V2, UNIQUENESS_SORTED_ROW_BYTES_V2,
    },
    recursive_statement::RecursiveTransitionStatementV2,
    recursive_trace::{
        decode_hash_control, decode_source_memory_write_control, decode_trace_chunk_control,
        HashControlSchemaV2, HashControlStageV2, RecursiveTraceEventV2, RecursiveTraceOpcodeV2,
        UniquenessListHashJobV2, UniquenessTranscriptHashJobV2, HASH_CONTROL_BLOCK_BYTES_V2,
        HASH_CONTROL_SOURCE_COMMON_BYTES_V2, HASH_CONTROL_TRACE_COMMON_BYTES_V2,
        RECURSIVE_TRACE_OPCODE_COUNT_V2, SOURCE_RECORD_HASH_LABEL_V2,
        STRUCTURAL_EVENT_HASH_LABEL_V2, TRACE_CANONICAL_CHUNK_BYTES_V2,
        TRACE_CHUNK_CONTROL_HEADER_BYTES_V2, TRACE_CHUNK_CONTROL_VERSION_V2,
        TRACE_CONTROL_PAYLOAD_BYTES_V2, TRACE_EVENT_HEADER_BYTES_V2,
        UNIQUENESS_LIST_COMMON_BYTES_V2, UNIQUENESS_TRANSCRIPT_COMMON_BYTES_V2,
    },
    version_registry::{CheckpointVersionRegistryV2, RecursiveBoundedObjectV2},
};
use crate::{
    settlement::{SettlementStore, SettlementUpdateTraceCircuitDecoderV2},
    CheckpointError,
};

const PLONKY3_BASE_WIRE_VERSION_V2: u16 = 2;
const PLONKY3_BASE_MAGIC_V2: [u8; 8] = *b"Z00ZP3B2";
const PLONKY3_STATEMENT_MAGIC_V2: [u8; 8] = *b"Z00ZP3S2";
const PLONKY3_PARAMETER_MAGIC_V2: [u8; 8] = *b"Z00ZP3P2";
const PLONKY3_SECURITY_MAGIC_V2: [u8; 8] = *b"Z00ZP3Q2";
const PLONKY3_EVENT_VECTOR_MAGIC_V2: [u8; 8] = *b"Z00ZP3E2";
const PLONKY3_STATEMENT_EXEC_TX_COUNT_BYTES_V2: usize = core::mem::size_of::<u32>();
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
    + PLONKY3_STATEMENT_EXEC_TX_COUNT_BYTES_V2
    + 32 * 17
    + 8
    + 8
    + RECURSIVE_TRACE_OPCODE_COUNT_V2 * 8 * 2;
const PLONKY3_STATEMENT_DIGESTS_OFFSET_V2: usize = 8 + 2;
const PLONKY3_STATEMENT_GRAMMAR_DIGEST_INDEX_V2: usize = 5;
const PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2: usize = PLONKY3_STATEMENT_DIGESTS_OFFSET_V2
    + 32 * 11
    + 8
    + 32
    + 1
    + 32
    + 32
    + PLONKY3_STATEMENT_EXEC_TX_COUNT_BYTES_V2;
const PLONKY3_STATEMENT_PRE_SETTLEMENT_INDEX_V2: usize = 7;
const PLONKY3_STATEMENT_POST_SETTLEMENT_INDEX_V2: usize = 8;
const PLONKY3_STATEMENT_DECLARED_WORK_INDEX_V2: usize = 13;
const PLONKY3_STATEMENT_PRE_UNIQUENESS_INDEX_V2: usize = 14;
const PLONKY3_STATEMENT_SPENT_PRECOMMIT_INDEX_V2: usize = 15;
const PLONKY3_STATEMENT_OUTPUT_PRECOMMIT_INDEX_V2: usize = 16;
const PLONKY3_STATEMENT_TRACE_DIGEST_INDEX_V2: usize = 11;
const PLONKY3_STATEMENT_UPDATE_TRACE_DIGEST_INDEX_V2: usize = 12;
const PLONKY3_STATEMENT_PRE_DEFINITION_INDEX_V2: usize = 9;
const PLONKY3_STATEMENT_POST_DEFINITION_INDEX_V2: usize = 10;
const PLONKY3_STATEMENT_DELTA_ROOT_INDEX_V2: usize = 2;
const PLONKY3_STATEMENT_WITNESS_ROOT_INDEX_V2: usize = 3;
const PLONKY3_STATEMENT_JOURNAL_DIGEST_INDEX_V2: usize = 4;
const PLONKY3_STATEMENT_LINK_DIGEST_INDEX_V2: usize = 6;
const PLONKY3_STATEMENT_HEIGHT_OFFSET_V2: usize = 8 + 2 + 32 * 11;
const PLONKY3_STATEMENT_PREDECESSOR_MARKER_OFFSET_V2: usize =
    PLONKY3_STATEMENT_HEIGHT_OFFSET_V2 + 8 + 32;
const PLONKY3_STATEMENT_DECLARED_EVENT_COUNT_OFFSET_V2: usize =
    PLONKY3_STATEMENT_PREDECESSOR_MARKER_OFFSET_V2
        + 1
        + 32
        + 32
        + PLONKY3_STATEMENT_EXEC_TX_COUNT_BYTES_V2
        + 32 * 17;
const PLONKY3_STATEMENT_DECLARED_COUNTS_OFFSET_V2: usize =
    PLONKY3_STATEMENT_DECLARED_EVENT_COUNT_OFFSET_V2 + 16;
const PLONKY3_BASE_MAX_CANONICAL_BYTES_V2: usize =
    PLONKY3_BASE_MAX_PROOF_BYTES_V2 + PLONKY3_BASE_STATEMENT_BYTES_V2 + 256;
const PLONKY3_PREDICATE_VECTOR_LABEL_V2: &[u8] = b"z00z.plonky3.base.predicate-vector.v2";
const PLONKY3_FRI_LOG_BLOWUP_V2: u8 = 1;
const PLONKY3_FRI_LOG_FINAL_POLY_LEN_V2: u8 = 0;
const PLONKY3_FRI_MAX_LOG_ARITY_V2: u8 = 3;
const PLONKY3_FRI_NUM_QUERIES_V2: u16 = 232;
const PLONKY3_FRI_COMMIT_POW_BITS_V2: u8 = 0;
const PLONKY3_FRI_QUERY_POW_BITS_V2: u8 = 16;
const PLONKY3_BASE_FIELD_BITS_V2: u16 = 31;
const PLONKY3_CHALLENGE_EXTENSION_DEGREE_V2: u8 = 8;
const PLONKY3_FRI_CLASSICAL_BITS_V2: u16 = 248;
const PLONKY3_FRI_QUANTUM_SEARCH_BITS_V2: u16 = 124;
const PLONKY3_HASH_OUTPUT_BITS_V2: u16 = 512;
const PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2: u16 = 170;
const PLONKY3_CHALLENGER_CAPACITY_BITS_V2: u16 = 512;
const PLONKY3_CHALLENGER_QUANTUM_PREIMAGE_BITS_V2: u16 = 256;
const PLONKY3_TABLE_MIN_HEIGHT_V2: usize = 8;
const PLONKY3_TABLE_PUBLIC_LANES_V2: usize = 4;
const PLONKY3_TABLE_ALU_LANES_V2: usize = 4;
const PLONKY3_TRACE_EXTENSION_DEGREE_V2: u8 = 4;
const PLONKY3_SECURITY_GENERATION_V2: u16 = 2;
const PLONKY3_SECURITY_COMPOSITION_RULE_GENERATION_V2: u16 = 2;
const PLONKY3_BASE_RECURSION_DEPTH_V2: u16 = 1;
const PLONKY3_MAX_ACCEPTED_EPOCH_PROOFS_V2: u64 = 1 << 20;
const PLONKY3_MINIMUM_RESIDUAL_BITS_V2: u16 = 100;
const PLONKY3_PER_PROOF_BOUND_BITS_V2: u16 = 122;
const PLONKY3_LIFETIME_BOUND_BITS_V2: u16 = 101;

type Plonky3TraceFieldV2 = BinomialExtensionField<KoalaBear, 4>;
type Plonky3HashV2 = Sha512HasherV2<1>;
type Plonky3CompressionV2 = CompressionFunctionFromHasher<Sha512HasherV2<2>, 2, 8>;
type Plonky3ValueMmcsV2 =
    MerkleTreeMmcs<KoalaBear, u64, SerializingHasher<Plonky3HashV2>, Plonky3CompressionV2, 2, 8>;
type Plonky3ChallengeV2 = BinomialExtensionField<KoalaBear, 8>;
type Plonky3ChallengeMmcsV2 = ExtensionMmcs<KoalaBear, Plonky3ChallengeV2, Plonky3ValueMmcsV2>;
type Plonky3PcsV2 = TwoAdicFriPcs<
    KoalaBear,
    Radix2DitParallel<KoalaBear>,
    Plonky3ValueMmcsV2,
    Plonky3ChallengeMmcsV2,
>;
type Plonky3ChallengerV2 =
    SerializingChallenger32<KoalaBear, HashChallenger<u8, Sha512HasherV2<3>, 64>>;
type Plonky3StarkConfigV2 = StarkConfig<Plonky3PcsV2, Plonky3ChallengeV2, Plonky3ChallengerV2>;
type CircuitByteBitsV2 = [ExprId; 8];
type Plonky3WordBitsV2 = [ExprId; 32];

/// SHA-512 adapter with explicit protocol-domain and byte-length framing.
///
/// Domains are fixed at the type level: `1` is a Merkle leaf, `2` is an
/// internal Merkle node, and `3` is the Fiat–Shamir transcript.  The framing
/// prevents cross-role and variable-length ambiguity while retaining the full
/// 512-bit digest for both the FRI PCS and challenger.
#[derive(Clone, Copy, Debug, Default)]
struct Sha512HasherV2<const DOMAIN: u8>;

impl<const DOMAIN: u8> CryptographicHasher<u8, [u8; 64]> for Sha512HasherV2<DOMAIN> {
    fn hash_iter<I>(&self, input: I) -> [u8; 64]
    where
        I: IntoIterator<Item = u8>,
    {
        let bytes: Vec<u8> = input.into_iter().collect();
        let mut hasher = Sha512::new();
        hasher.update(b"z00z.plonky3.sha512.v2");
        hasher.update([DOMAIN]);
        hasher.update(
            u64::try_from(bytes.len())
                .expect("Plonky3 SHA-512 input length fits u64")
                .to_le_bytes(),
        );
        hasher.update(&bytes);
        hasher.finalize().into()
    }
}

impl<const DOMAIN: u8> CryptographicHasher<u64, [u64; 8]> for Sha512HasherV2<DOMAIN> {
    fn hash_iter<I>(&self, input: I) -> [u64; 8]
    where
        I: IntoIterator<Item = u64>,
    {
        let words: Vec<u64> = input.into_iter().collect();
        let mut hasher = Sha512::new();
        hasher.update(b"z00z.plonky3.sha512.v2");
        hasher.update([DOMAIN]);
        hasher.update(
            u64::try_from(words.len())
                .expect("Plonky3 SHA-512 input length fits u64")
                .to_le_bytes(),
        );
        for word in words {
            hasher.update(word.to_le_bytes());
        }
        let digest: [u8; 64] = hasher.finalize().into();
        core::array::from_fn(|index| {
            u64::from_le_bytes(
                digest[index * 8..(index + 1) * 8]
                    .try_into()
                    .expect("fixed SHA-512 word"),
            )
        })
    }
}

#[derive(Clone)]
struct CircuitEventViewV2 {
    event: RecursiveTraceEventV2,
    canonical_bits: Vec<CircuitByteBitsV2>,
}

impl CircuitEventViewV2 {
    fn payload_bits(&self) -> Result<&[CircuitByteBitsV2], CheckpointError> {
        self.canonical_bits
            .get(TRACE_EVENT_HEADER_BYTES_V2..)
            .ok_or(CheckpointError::Invariant)
    }
}

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
    fri_classical_bits: u16,
    fri_quantum_search_bits: u16,
    hash_output_bits: u16,
    hash_collision_bits: u16,
    challenger_capacity_bits: u16,
    challenger_quantum_preimage_bits: u16,
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
    /// The one live Plan-07 quantum-aware budget.  Degree-8 KoalaBear caps the
    /// concrete FRI calculation at 248 classical bits, conservatively halved
    /// to 124 bits for quantum search.  Domain-separated SHA-512 retains all
    /// 512 digest bits in both the MMCS and challenger, conservatively giving
    /// 170 bits under the generic quantum collision bound and 256 bits against
    /// Grover preimage search.
    /// Composing the three families rounds upward to `2^-122`; at most `2^20`
    /// proofs plus inherited rotation loss round upward to `2^-101`.
    pub fn authority_pinned() -> Result<Self, CheckpointError> {
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        let parameter_generation = registry
            .row(RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest)?
            .parameter_generation
            .ok_or(CheckpointError::Authority)?;
        let per_proof_bound = derive_per_proof_bound(
            PLONKY3_FRI_QUANTUM_SEARCH_BITS_V2,
            PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2,
            PLONKY3_CHALLENGER_QUANTUM_PREIMAGE_BITS_V2,
            3,
        )?;
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
            fri_classical_bits: PLONKY3_FRI_CLASSICAL_BITS_V2,
            fri_quantum_search_bits: PLONKY3_FRI_QUANTUM_SEARCH_BITS_V2,
            hash_output_bits: PLONKY3_HASH_OUTPUT_BITS_V2,
            hash_collision_bits: PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2,
            challenger_capacity_bits: PLONKY3_CHALLENGER_CAPACITY_BITS_V2,
            challenger_quantum_preimage_bits: PLONKY3_CHALLENGER_QUANTUM_PREIMAGE_BITS_V2,
            component_count: 3,
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
            self.fri_quantum_search_bits,
            self.hash_collision_bits,
            self.challenger_quantum_preimage_bits,
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
            || self.fri_classical_bits != expected_fri
            || self.fri_quantum_search_bits
                != self
                    .fri_classical_bits
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?
                    / 2
            || self.fri_quantum_search_bits != PLONKY3_FRI_QUANTUM_SEARCH_BITS_V2
            || self.hash_output_bits != PLONKY3_HASH_OUTPUT_BITS_V2
            || self.hash_collision_bits != self.hash_output_bits / 3
            || self.hash_collision_bits != PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2
            || self.challenger_capacity_bits != PLONKY3_CHALLENGER_CAPACITY_BITS_V2
            || self.challenger_quantum_preimage_bits != self.challenger_capacity_bits / 2
            || self.challenger_quantum_preimage_bits != PLONKY3_CHALLENGER_QUANTUM_PREIMAGE_BITS_V2
            || self.component_count != 3
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
        let mut bytes = Vec::with_capacity(56);
        bytes.extend_from_slice(&PLONKY3_SECURITY_MAGIC_V2);
        bytes.extend_from_slice(&self.generation.to_le_bytes());
        bytes.extend_from_slice(&self.parameter_generation.to_le_bytes());
        bytes.extend_from_slice(&self.base_field_bits.to_le_bytes());
        bytes.push(self.challenge_extension_degree);
        bytes.push(self.fri_log_blowup);
        bytes.extend_from_slice(&self.fri_num_queries.to_le_bytes());
        bytes.push(self.fri_commit_pow_bits);
        bytes.push(self.fri_query_pow_bits);
        bytes.extend_from_slice(&self.fri_classical_bits.to_le_bytes());
        bytes.extend_from_slice(&self.fri_quantum_search_bits.to_le_bytes());
        bytes.extend_from_slice(&self.hash_output_bits.to_le_bytes());
        bytes.extend_from_slice(&self.hash_collision_bits.to_le_bytes());
        bytes.extend_from_slice(&self.challenger_capacity_bits.to_le_bytes());
        bytes.extend_from_slice(&self.challenger_quantum_preimage_bits.to_le_bytes());
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
        if payload.len() != 56 || payload[..8] != PLONKY3_SECURITY_MAGIC_V2 {
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
        let fri_classical_bits = take_u16(payload, &mut cursor)?;
        let fri_quantum_search_bits = take_u16(payload, &mut cursor)?;
        let hash_output_bits = take_u16(payload, &mut cursor)?;
        let hash_collision_bits = take_u16(payload, &mut cursor)?;
        let challenger_capacity_bits = take_u16(payload, &mut cursor)?;
        let challenger_quantum_preimage_bits = take_u16(payload, &mut cursor)?;
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
            fri_classical_bits,
            fri_quantum_search_bits,
            hash_output_bits,
            hash_collision_bits,
            challenger_capacity_bits,
            challenger_quantum_preimage_bits,
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
    challenger_bits: u16,
    component_count: u16,
) -> Result<DyadicErrorBoundV2, CheckpointError> {
    if component_count == 0 {
        return Err(security_budget_error());
    }
    let weakest_component = fri_bits.min(hash_bits).min(challenger_bits);
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
        put_short_str(
            &mut bytes,
            "sha2_512_domain_separated_len_framed_merkle_and_fiat_shamir",
        )?;
        put_short_str(&mut bytes, "poseidon2_koala_bear_d4_width16")?;
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
        let (decoded_proof, remaining): (BatchStarkProof<Plonky3StarkConfigV2>, &[u8]) =
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

/// Fail-closed construction checkpoint for the complete Plan-07 AIR.
///
/// Keeping this explicit gate in both adapter directions prevents a later
/// partial refactor from bypassing the single canonical transition relation.
fn require_complete_transition_air_v2() -> Result<(), CheckpointError> {
    Ok(())
}

impl Plonky3BaseAdapterV2 {
    /// Run the independent native evaluator, construct the exact canonical
    /// witness vector, and generate a real pinned Batch-STARK proof.
    pub fn prove(
        transition: &mut CanonicalCheckpointTransitionV2,
        store: &SettlementStore,
    ) -> Result<Plonky3BaseProofV2, CheckpointError> {
        require_complete_transition_air_v2()?;
        let material = transition_material(transition, store).map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 transition material construction failed: {error}"
            ))
        })?;
        let security = RecursiveSecurityBudgetManifestV2::authority_pinned().map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 security manifest resolution failed: {error}"
            ))
        })?;
        let parameters =
            Plonky3ParameterManifestV2::authority_pinned(&security).map_err(|error| {
                CheckpointError::Backend(format!(
                    "Plonky3 parameter manifest resolution failed: {error}"
                ))
            })?;
        let words = predicate_words(&material.statement, &material.event_vector, &parameters)
            .map_err(|error| {
                CheckpointError::Backend(format!("Plonky3 predicate encoding failed: {error}"))
            })?;
        let prepared = prepare_circuit(&words, Some(&material.event_vector)).map_err(|error| {
            CheckpointError::Backend(format!("Plonky3 AIR construction failed: {error}"))
        })?;
        let air_binding_digest = common_binding_digest(prepared.data.common_data())?;
        let PreparedCircuitV2 {
            circuit,
            private_inputs,
            config,
            data,
            table_packing,
        } = prepared;
        let mut runner = circuit.runner();
        runner
            .set_private_inputs(&private_inputs)
            .map_err(|_| CheckpointError::Backend("Plonky3 witness loading failed".into()))?;
        let traces = runner
            .run()
            .map_err(|_| CheckpointError::BackendVerificationFailed)?;
        drop(circuit);
        drop(private_inputs);
        let mut prover = BatchStarkProver::new(config).with_table_packing(table_packing.clone());
        for table in poseidon2_table_provers_d4(Poseidon2Config::KOALA_BEAR_D4_W16) {
            prover.register_table_prover(table);
        }
        let proof = prover
            .prove_all_tables(&traces, &data)
            .map_err(|_| CheckpointError::BackendVerificationFailed)?;
        drop(traces);
        drop(data);
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
        let material = proof.local_verification_material.as_ref().ok_or(
            CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::WitnessUnavailable,
            ),
        )?;
        let expected_material = transition_material(transition, store)?;
        if expected_material.statement != proof.statement
            || expected_material.event_vector != material.event_vector
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3TranscriptMismatch,
            ));
        }
        let security = RecursiveSecurityBudgetManifestV2::authority_pinned()?;
        let parameters = Plonky3ParameterManifestV2::authority_pinned(&security)?;
        if proof.parameter_digest != parameters.digest
            || proof.security_budget_digest != security.digest()
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3TranscriptMismatch,
            ));
        }
        let words = predicate_words(
            &expected_material.statement,
            &expected_material.event_vector,
            &parameters,
        )?;
        let prepared = prepare_circuit(&words, Some(&material.event_vector)).map_err(|error| {
            CheckpointError::Backend(format!("Plonky3 AIR reconstruction failed: {error}"))
        })?;
        let expected_air_binding = common_binding_digest(prepared.data.common_data())?;
        if proof.air_binding_digest != expected_air_binding {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        let (decoded, remaining): (BatchStarkProof<Plonky3StarkConfigV2>, &[u8]) =
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
            || decoded.w_binomial.is_none()
            || decoded.alu_quintic_trinomial
            || decoded.non_primitives.len() != 1
            || decoded.non_primitives[0].op_type
                != p3_circuit::ops::NpoTypeId::poseidon2_perm(Poseidon2Config::KOALA_BEAR_D4_W16)
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3ProofMalformed,
            ));
        }
        let PreparedCircuitV2 {
            circuit,
            private_inputs,
            config,
            data,
            table_packing,
        } = prepared;
        drop(circuit);
        drop(private_inputs);
        drop(data);
        let mut verifier = BatchStarkProver::new(config).with_table_packing(table_packing);
        for table in poseidon2_table_provers_d4(Poseidon2Config::KOALA_BEAR_D4_W16) {
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
    let evaluated = transition.evaluate(store).map_err(|error| {
        CheckpointError::Backend(format!(
            "Plonky3 native transition evaluation failed: {error}"
        ))
    })?;
    let mut event_vector = Vec::with_capacity(
        usize::try_from(evaluated.statement().declared_byte_count())
            .map_err(|_| CheckpointError::Limit)?,
    );
    event_vector.extend_from_slice(&PLONKY3_EVENT_VECTOR_MAGIC_V2);
    event_vector.extend_from_slice(&0_u64.to_le_bytes());
    let mut count = 0_u64;
    transition
        .replay_canonical_events(store, |event| {
            let bytes = event.canonical_bytes()?;
            let len = u32::try_from(bytes.len()).map_err(|_| CheckpointError::Limit)?;
            event_vector.extend_from_slice(&len.to_le_bytes());
            event_vector.extend_from_slice(&bytes);
            count = count.checked_add(1).ok_or(CheckpointError::Overflow)?;
            if event_vector.len() > PLONKY3_BASE_MAX_VECTOR_BYTES_V2 {
                return Err(CheckpointError::Limit);
            }
            Ok(())
        })
        .map_err(|error| {
            CheckpointError::Backend(format!("Plonky3 canonical event replay failed: {error}"))
        })?;
    event_vector[8..16].copy_from_slice(&count.to_le_bytes());
    let security = RecursiveSecurityBudgetManifestV2::authority_pinned().map_err(|error| {
        CheckpointError::Backend(format!(
            "Plonky3 material security manifest resolution failed: {error}"
        ))
    })?;
    let parameters = Plonky3ParameterManifestV2::authority_pinned(&security).map_err(|error| {
        CheckpointError::Backend(format!(
            "Plonky3 material parameter manifest resolution failed: {error}"
        ))
    })?;
    let statement = build_base_statement(
        transition,
        evaluated.statement(),
        &event_vector,
        &parameters,
        &security,
    )
    .map_err(|error| {
        CheckpointError::Backend(format!(
            "Plonky3 base statement construction failed: {error}"
        ))
    })?;
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
    let spec = RecursiveCircuitSpecV2::new(authority.layout(), profile).map_err(|error| {
        CheckpointError::Backend(format!(
            "Plonky3 base-statement circuit spec resolution failed: {error}"
        ))
    })?;
    let registry = CheckpointVersionRegistryV2::authority_pinned().map_err(|error| {
        CheckpointError::Backend(format!(
            "Plonky3 base-statement registry resolution failed: {error}"
        ))
    })?;
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
        return Err(CheckpointError::Backend(format!(
            "Plonky3 base-statement width mismatch: actual {}, expected {}",
            bytes.len(),
            PLONKY3_BASE_STATEMENT_BYTES_V2
        )));
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
    const DECLARED_EVENT_COUNT_OFFSET: usize = PREDECESSOR_MARKER_OFFSET
        + 1
        + 32
        + 32
        + PLONKY3_STATEMENT_EXEC_TX_COUNT_BYTES_V2
        + 32 * TRANSITION_DIGEST_COUNT;
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
    for opcode in [
        RecursiveTraceOpcodeV2::BeginBlock,
        RecursiveTraceOpcodeV2::ReplayInput,
        RecursiveTraceOpcodeV2::ReplayOutput,
        RecursiveTraceOpcodeV2::UniquenessPrecommit,
        RecursiveTraceOpcodeV2::UniquenessSorted,
        RecursiveTraceOpcodeV2::UniquenessChallenge,
        RecursiveTraceOpcodeV2::NetMerge,
        RecursiveTraceOpcodeV2::JmtUpdate,
        RecursiveTraceOpcodeV2::JmtMicroOp,
        RecursiveTraceOpcodeV2::PromoteChildRoot,
        RecursiveTraceOpcodeV2::CommitTypedEvent,
        RecursiveTraceOpcodeV2::FinalizeBlock,
    ] {
        let start = (opcode as usize - 1) * 8;
        count_sum = count_sum
            .checked_add(u64::from_le_bytes(
                declared_counts[start..start + 8]
                    .try_into()
                    .map_err(|_| CheckpointError::Canonical)?,
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
    let decoded_statement =
        decode_base_statement(statement.canonical_bytes()).map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 base-statement canonical decode failed: {error}"
            ))
        })?;
    if decoded_statement != *statement {
        return Err(CheckpointError::Backend(
            "Plonky3 base-statement canonical roundtrip mismatch".into(),
        ));
    }
    validate_event_vector(statement, event_vector).map_err(|error| {
        CheckpointError::Backend(format!(
            "Plonky3 event-vector canonical validation failed: {error}"
        ))
    })?;
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
    while !words.len().is_multiple_of(8) {
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
    config: Plonky3StarkConfigV2,
    data: CircuitProverData<Plonky3StarkConfigV2>,
    table_packing: TablePacking,
}

fn air_construction_stage(stage: &'static str, error: CheckpointError) -> CheckpointError {
    CheckpointError::Backend(format!("Plonky3 AIR {stage} failed: {error}"))
}

fn transition_semantics_error(stage: &'static str) -> CheckpointError {
    CheckpointError::Backend(format!(
        "Plonky3 frozen transition {stage} invariant mismatch"
    ))
}

fn transition_semantics_stage(stage: &'static str, error: CheckpointError) -> CheckpointError {
    CheckpointError::Backend(format!("Plonky3 frozen transition {stage} failed: {error}"))
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
    core::array::from_fn(|bit| circuit_xor3_bit(builder, first[bit], second[bit], third[bit], two))
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

fn circuit_constant_word(value: u32, zero: ExprId, one: ExprId) -> Plonky3WordBitsV2 {
    core::array::from_fn(|bit| if (value >> bit) & 1 == 0 { zero } else { one })
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
            &[schedule[index - 16], sigma_0, schedule[index - 7], sigma_1],
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
        circuit_word_add2(builder, &chaining_before[index], &state[index], zero, two)
    }))
}

fn circuit_event_views(
    event_vector: &[u8],
    predicate_byte_bits: &[[ExprId; 8]],
) -> Result<Vec<CircuitEventViewV2>, CheckpointError> {
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
    let declared_count = u64::from_le_bytes(
        event_vector[8..16]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    let profile = RecursiveCircuitProfileV2::authority_pinned();
    let mut cursor = 16_usize;
    let mut views = Vec::new();
    views
        .try_reserve_exact(
            usize::try_from(declared_count)
                .map_err(|_| CheckpointError::Limit)?
                .min(event_vector.len() / TRACE_EVENT_HEADER_BYTES_V2),
        )
        .map_err(|_| CheckpointError::Limit)?;
    while cursor < event_vector.len() {
        let event_len = usize::try_from(take_u32(event_vector, &mut cursor)?)
            .map_err(|_| CheckpointError::Limit)?;
        let event_start = cursor;
        let event_bytes = take_slice(event_vector, &mut cursor, event_len)?;
        let event = RecursiveTraceEventV2::decode_canonical(event_bytes, &profile)?;
        if event.canonical_bytes()? != event_bytes {
            return Err(CheckpointError::Canonical);
        }
        let canonical_bits = event_bits
            .get(event_start..event_start + event_len)
            .ok_or(CheckpointError::Invariant)?
            .to_vec();
        views.push(CircuitEventViewV2 {
            event,
            canonical_bits,
        });
        if u64::try_from(views.len()).map_err(|_| CheckpointError::Limit)? > declared_count {
            return Err(CheckpointError::Canonical);
        }
    }
    if cursor != event_vector.len()
        || u64::try_from(views.len()).map_err(|_| CheckpointError::Limit)? != declared_count
    {
        return Err(CheckpointError::Canonical);
    }
    Ok(views)
}

fn circuit_statement_bits(
    predicate_byte_bits: &[[ExprId; 8]],
) -> Result<&[[ExprId; 8]], CheckpointError> {
    let start = PLONKY3_PREDICATE_VECTOR_LABEL_V2
        .len()
        .checked_add(8)
        .ok_or(CheckpointError::Overflow)?;
    predicate_byte_bits
        .get(start..start + PLONKY3_BASE_STATEMENT_BYTES_V2)
        .ok_or(CheckpointError::Invariant)
}

fn statement_digest_bits(
    statement: &[[ExprId; 8]],
    offset: usize,
    index: usize,
) -> Result<&[[ExprId; 8]], CheckpointError> {
    let start = offset
        .checked_add(index.checked_mul(32).ok_or(CheckpointError::Overflow)?)
        .ok_or(CheckpointError::Overflow)?;
    statement
        .get(start..start + 32)
        .ok_or(CheckpointError::Invariant)
}

fn connect_byte_bits(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    left: &[ExprId; 8],
    right: &[ExprId; 8],
) {
    for (left, right) in left.iter().zip(right.iter()) {
        builder.connect(*left, *right);
    }
}

fn constant_byte_bits(value: u8, zero: ExprId, one: ExprId) -> [ExprId; 8] {
    core::array::from_fn(|bit| if (value >> bit) & 1 == 0 { zero } else { one })
}

fn connect_bytes_to_constants(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    actual: &[[ExprId; 8]],
    expected: &[u8],
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    if actual.len() != expected.len() {
        return Err(CheckpointError::Invariant);
    }
    for (actual, expected) in actual.iter().zip(expected.iter().copied()) {
        connect_byte_bits(builder, actual, &constant_byte_bits(expected, zero, one));
    }
    Ok(())
}

fn source_framed_message_bits(
    source: &CircuitEventViewV2,
    zero: ExprId,
    one: ExprId,
) -> Result<Vec<[ExprId; 8]>, CheckpointError> {
    let (message_bytes, block_count) = source.event.hash_geometry()?;
    let mut message = Vec::new();
    let prefix = CheckpointSha256BlockStreamV2::framed_role_prefix(CheckpointShaRole::Trace);
    message
        .try_reserve_exact(
            usize::try_from(
                block_count
                    .checked_mul(64)
                    .ok_or(CheckpointError::Overflow)?,
            )
            .map_err(|_| CheckpointError::Limit)?,
        )
        .map_err(|_| CheckpointError::Limit)?;
    message.extend(
        prefix
            .into_iter()
            .map(|byte| constant_byte_bits(byte, zero, one)),
    );
    let label_len =
        u64::try_from(SOURCE_RECORD_HASH_LABEL_V2.len()).map_err(|_| CheckpointError::Limit)?;
    message.extend(
        label_len
            .to_le_bytes()
            .into_iter()
            .map(|byte| constant_byte_bits(byte, zero, one)),
    );
    message.extend(
        SOURCE_RECORD_HASH_LABEL_V2
            .iter()
            .copied()
            .map(|byte| constant_byte_bits(byte, zero, one)),
    );
    let source_len =
        u64::try_from(source.canonical_bits.len()).map_err(|_| CheckpointError::Limit)?;
    message.extend(
        source_len
            .to_le_bytes()
            .into_iter()
            .map(|byte| constant_byte_bits(byte, zero, one)),
    );
    message.extend(source.canonical_bits.iter().copied());
    if u64::try_from(message.len()).map_err(|_| CheckpointError::Limit)? != message_bytes {
        return Err(CheckpointError::Invariant);
    }
    let padded_bytes = usize::try_from(
        block_count
            .checked_mul(64)
            .ok_or(CheckpointError::Overflow)?,
    )
    .map_err(|_| CheckpointError::Limit)?;
    if message.len() > padded_bytes.saturating_sub(9) {
        return Err(CheckpointError::Invariant);
    }
    message.push(constant_byte_bits(0x80, zero, one));
    while message.len() < padded_bytes - 8 {
        message.push(constant_byte_bits(0, zero, one));
    }
    let bit_length = message_bytes
        .checked_mul(8)
        .ok_or(CheckpointError::Overflow)?;
    message.extend(
        bit_length
            .to_be_bytes()
            .into_iter()
            .map(|byte| constant_byte_bits(byte, zero, one)),
    );
    if message.len() != padded_bytes {
        return Err(CheckpointError::Invariant);
    }
    Ok(message)
}

fn trace_framed_message_bits(
    sources: &[&CircuitEventViewV2],
    message_bytes: u64,
    block_count: u64,
    zero: ExprId,
    one: ExprId,
) -> Result<Vec<[ExprId; 8]>, CheckpointError> {
    let padded_bytes = usize::try_from(
        block_count
            .checked_mul(64)
            .ok_or(CheckpointError::Overflow)?,
    )
    .map_err(|_| CheckpointError::Limit)?;
    let mut message = Vec::new();
    message
        .try_reserve_exact(padded_bytes)
        .map_err(|_| CheckpointError::Limit)?;
    message.extend(
        CheckpointSha256BlockStreamV2::framed_role_prefix(CheckpointShaRole::Trace)
            .into_iter()
            .map(|byte| constant_byte_bits(byte, zero, one)),
    );
    for source in sources {
        let source_len =
            u64::try_from(source.canonical_bits.len()).map_err(|_| CheckpointError::Limit)?;
        message.extend(
            source_len
                .to_le_bytes()
                .into_iter()
                .map(|byte| constant_byte_bits(byte, zero, one)),
        );
        message.extend(source.canonical_bits.iter().copied());
    }
    if u64::try_from(message.len()).map_err(|_| CheckpointError::Limit)? != message_bytes
        || message.len() > padded_bytes.saturating_sub(9)
    {
        return Err(CheckpointError::Invariant);
    }
    message.push(constant_byte_bits(0x80, zero, one));
    while message.len() < padded_bytes - 8 {
        message.push(constant_byte_bits(0, zero, one));
    }
    let bit_length = message_bytes
        .checked_mul(8)
        .ok_or(CheckpointError::Overflow)?;
    message.extend(
        bit_length
            .to_be_bytes()
            .into_iter()
            .map(|byte| constant_byte_bits(byte, zero, one)),
    );
    if message.len() != padded_bytes {
        return Err(CheckpointError::Invariant);
    }
    Ok(message)
}

fn framed_parts_message_bits(
    role: CheckpointShaRole,
    parts: &[Vec<[ExprId; 8]>],
    zero: ExprId,
    one: ExprId,
) -> Result<(Vec<[ExprId; 8]>, u64, u64), CheckpointError> {
    if parts.is_empty() {
        return Err(CheckpointError::Invariant);
    }
    let part_bytes = parts.iter().try_fold(0_u64, |total, part| {
        total
            .checked_add(u64::try_from(part.len()).map_err(|_| CheckpointError::Limit)?)
            .ok_or(CheckpointError::Overflow)
    })?;
    let part_count = u64::try_from(parts.len()).map_err(|_| CheckpointError::Limit)?;
    let message_bytes =
        CheckpointSha256BlockStreamV2::framed_bytes_for_parts(role, part_bytes, part_count)
            .map_err(|_| CheckpointError::Limit)?;
    let block_count = CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(message_bytes)
        .map_err(|_| CheckpointError::Limit)?;
    let padded_bytes = usize::try_from(
        block_count
            .checked_mul(64)
            .ok_or(CheckpointError::Overflow)?,
    )
    .map_err(|_| CheckpointError::Limit)?;
    let mut message = Vec::new();
    message
        .try_reserve_exact(padded_bytes)
        .map_err(|_| CheckpointError::Limit)?;
    message.extend(
        CheckpointSha256BlockStreamV2::framed_role_prefix(role)
            .into_iter()
            .map(|byte| constant_byte_bits(byte, zero, one)),
    );
    for part in parts {
        message.extend(
            u64::try_from(part.len())
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes()
                .into_iter()
                .map(|byte| constant_byte_bits(byte, zero, one)),
        );
        message.extend(part.iter().copied());
    }
    if u64::try_from(message.len()).map_err(|_| CheckpointError::Limit)? != message_bytes
        || message.len() > padded_bytes.saturating_sub(9)
    {
        return Err(CheckpointError::Invariant);
    }
    message.push(constant_byte_bits(0x80, zero, one));
    while message.len() < padded_bytes - 8 {
        message.push(constant_byte_bits(0, zero, one));
    }
    message.extend(
        message_bytes
            .checked_mul(8)
            .ok_or(CheckpointError::Overflow)?
            .to_be_bytes()
            .into_iter()
            .map(|byte| constant_byte_bits(byte, zero, one)),
    );
    if message.len() != padded_bytes {
        return Err(CheckpointError::Invariant);
    }
    Ok((message, message_bytes, block_count))
}

fn circuit_word_to_be_bytes(word: &Plonky3WordBitsV2) -> [[ExprId; 8]; 4] {
    core::array::from_fn(|byte| {
        core::array::from_fn(|bit| word[(3_usize.saturating_sub(byte)) * 8 + bit])
    })
}

fn circuit_sha256_padded_message_digest(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    message: &[[ExprId; 8]],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<Vec<[ExprId; 8]>, CheckpointError> {
    if message.is_empty() || !message.len().is_multiple_of(64) {
        return Err(CheckpointError::Invariant);
    }
    let mut state: [Plonky3WordBitsV2; 8] =
        core::array::from_fn(|word| circuit_constant_word(SHA256_IV_V2[word], zero, one));
    for block in message.chunks_exact(64) {
        state = circuit_sha256_compress(builder, block, &state, zero, one, two)?;
    }
    Ok(state.iter().flat_map(circuit_word_to_be_bytes).collect())
}

fn structural_event_message_bits(
    view: &CircuitEventViewV2,
    zero: ExprId,
    one: ExprId,
) -> Result<Vec<[ExprId; 8]>, CheckpointError> {
    let payload = view.payload_bits()?;
    let payload_len = u64::try_from(payload.len()).map_err(|_| CheckpointError::Limit)?;
    let raw_bytes = u64::try_from(STRUCTURAL_EVENT_HASH_LABEL_V2.len())
        .map_err(|_| CheckpointError::Limit)?
        .checked_add(1)
        .and_then(|value| value.checked_add(8))
        .and_then(|value| value.checked_add(payload_len))
        .ok_or(CheckpointError::Overflow)?;
    let message_bytes = CheckpointSha256BlockStreamV2::framed_bytes_for_parts(
        CheckpointShaRole::Trace,
        raw_bytes,
        4,
    )
    .map_err(|_| CheckpointError::Limit)?;
    let block_count = CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(message_bytes)
        .map_err(|_| CheckpointError::Limit)?;
    let padded_bytes = usize::try_from(
        block_count
            .checked_mul(64)
            .ok_or(CheckpointError::Overflow)?,
    )
    .map_err(|_| CheckpointError::Limit)?;
    let mut message = Vec::new();
    message
        .try_reserve_exact(padded_bytes)
        .map_err(|_| CheckpointError::Limit)?;
    message.extend(
        CheckpointSha256BlockStreamV2::framed_role_prefix(CheckpointShaRole::Trace)
            .into_iter()
            .map(|byte| constant_byte_bits(byte, zero, one)),
    );
    for (length, bytes) in [
        (
            u64::try_from(STRUCTURAL_EVENT_HASH_LABEL_V2.len())
                .map_err(|_| CheckpointError::Limit)?,
            None,
        ),
        (1, Some(&view.canonical_bits[0..1])),
        (8, Some(&view.canonical_bits[1..9])),
        (payload_len, Some(payload)),
    ] {
        message.extend(
            length
                .to_le_bytes()
                .into_iter()
                .map(|byte| constant_byte_bits(byte, zero, one)),
        );
        if let Some(bytes) = bytes {
            message.extend(bytes.iter().copied());
        } else {
            message.extend(
                STRUCTURAL_EVENT_HASH_LABEL_V2
                    .iter()
                    .copied()
                    .map(|byte| constant_byte_bits(byte, zero, one)),
            );
        }
    }
    if u64::try_from(message.len()).map_err(|_| CheckpointError::Limit)? != message_bytes
        || message.len() > padded_bytes.saturating_sub(9)
    {
        return Err(CheckpointError::Invariant);
    }
    message.push(constant_byte_bits(0x80, zero, one));
    while message.len() < padded_bytes - 8 {
        message.push(constant_byte_bits(0, zero, one));
    }
    message.extend(
        message_bytes
            .checked_mul(8)
            .ok_or(CheckpointError::Overflow)?
            .to_be_bytes()
            .into_iter()
            .map(|byte| constant_byte_bits(byte, zero, one)),
    );
    if message.len() != padded_bytes {
        return Err(CheckpointError::Invariant);
    }
    Ok(message)
}

fn circuit_sha256_padded_message(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    message: &[CircuitByteBitsV2],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<Vec<[ExprId; 8]>, CheckpointError> {
    if message.is_empty() || !message.len().is_multiple_of(64) {
        return Err(CheckpointError::Invariant);
    }
    let mut state: [Plonky3WordBitsV2; 8] = core::array::from_fn(|word| {
        core::array::from_fn(|bit| {
            if (SHA256_IV_V2[word] >> bit) & 1 == 0 {
                zero
            } else {
                one
            }
        })
    });
    for block in message.chunks_exact(64) {
        state = circuit_sha256_compress(builder, block, &state, zero, one, two)?;
    }
    let mut digest = Vec::with_capacity(32);
    for word in state {
        for byte in 0..4 {
            digest.push(core::array::from_fn(|bit| word[(3 - byte) * 8 + bit]));
        }
    }
    Ok(digest)
}

fn constrain_structural_source_event_ids(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    views: &[CircuitEventViewV2],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<usize, CheckpointError> {
    let mut constrained = 0_usize;
    for view in views {
        if !view.event.opcode().is_source_record() {
            continue;
        }
        if matches!(
            view.event.opcode(),
            RecursiveTraceOpcodeV2::ReplayInput | RecursiveTraceOpcodeV2::ReplayOutput
        ) {
            let item = decode_flow_item(view.event.payload())?;
            connect_bytes_to_constants(
                builder,
                &view.canonical_bits[9..41],
                &item.terminal_id,
                zero,
                one,
            )?;
            constrained = constrained
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
            continue;
        }
        let message = structural_event_message_bits(view, zero, one)?;
        let digest = circuit_sha256_padded_message(builder, &message, zero, one, two)?;
        for (claimed, computed) in view.canonical_bits[9..41].iter().zip(digest.iter()) {
            connect_byte_bits(builder, claimed, computed);
        }
        constrained = constrained
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
    }
    Ok(constrained)
}

#[allow(clippy::too_many_arguments)]
fn constrain_source_control_common(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    control_view: &CircuitEventViewV2,
    source: &CircuitEventViewV2,
    stage: HashControlStageV2,
    message_bytes: u64,
    block_count: u64,
    final_digest: &[[ExprId; 8]],
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    if final_digest.len() != 32 {
        return Err(CheckpointError::Invariant);
    }
    let payload = control_view.payload_bits()?;
    if payload.len() < HASH_CONTROL_SOURCE_COMMON_BYTES_V2 {
        return Err(CheckpointError::Invariant);
    }
    connect_byte_bits(
        builder,
        &payload[0],
        &constant_byte_bits(HashControlSchemaV2::SourceRecord as u8, zero, one),
    );
    connect_byte_bits(builder, &payload[1], &constant_byte_bits(1, zero, one));
    connect_byte_bits(
        builder,
        &payload[2],
        &constant_byte_bits(stage as u8, zero, one),
    );
    for (binding, digest) in payload[3..35].iter().zip(final_digest.iter()) {
        connect_byte_bits(builder, binding, digest);
    }
    connect_bytes_to_constants(
        builder,
        &payload[35..43],
        &message_bytes.to_le_bytes(),
        zero,
        one,
    )?;
    connect_bytes_to_constants(
        builder,
        &payload[43..51],
        &block_count.to_le_bytes(),
        zero,
        one,
    )?;
    for (ordinal, source_ordinal) in payload[51..59]
        .iter()
        .zip(source.canonical_bits[1..9].iter())
    {
        connect_byte_bits(builder, ordinal, source_ordinal);
    }
    connect_byte_bits(builder, &payload[59], &source.canonical_bits[0]);
    for (object_id, source_object_id) in payload[60..HASH_CONTROL_SOURCE_COMMON_BYTES_V2]
        .iter()
        .zip(source.canonical_bits[9..41].iter())
    {
        connect_byte_bits(builder, object_id, source_object_id);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn constrain_trace_control_common(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    control_view: &CircuitEventViewV2,
    stage: HashControlStageV2,
    final_digest: &[[ExprId; 8]],
    event_count: u64,
    byte_count: u64,
    message_bytes: u64,
    block_count: u64,
    padding_bytes: u64,
    bit_length: u64,
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    if final_digest.len() != 32 {
        return Err(CheckpointError::Invariant);
    }
    let payload = control_view.payload_bits()?;
    if payload.len() < HASH_CONTROL_TRACE_COMMON_BYTES_V2 {
        return Err(CheckpointError::Invariant);
    }
    connect_byte_bits(
        builder,
        &payload[0],
        &constant_byte_bits(HashControlSchemaV2::TracePrecommit as u8, zero, one),
    );
    connect_byte_bits(builder, &payload[1], &constant_byte_bits(1, zero, one));
    connect_byte_bits(
        builder,
        &payload[2],
        &constant_byte_bits(stage as u8, zero, one),
    );
    for (binding, digest) in payload[3..35].iter().zip(final_digest.iter()) {
        connect_byte_bits(builder, binding, digest);
    }
    for (actual, expected) in [
        (&payload[35..43], message_bytes.to_le_bytes()),
        (&payload[43..51], block_count.to_le_bytes()),
        (&payload[51..59], event_count.to_le_bytes()),
        (&payload[59..67], byte_count.to_le_bytes()),
        (&payload[67..75], padding_bytes.to_le_bytes()),
        (&payload[75..83], bit_length.to_le_bytes()),
    ] {
        connect_bytes_to_constants(builder, actual, &expected, zero, one)?;
    }
    connect_byte_bits(builder, &payload[83], &constant_byte_bits(1, zero, one));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn constrain_uniqueness_list_control_common(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    control_view: &CircuitEventViewV2,
    job: UniquenessListHashJobV2,
    stage: HashControlStageV2,
    final_digest: &[[ExprId; 8]],
    expected_count: &[[ExprId; 8]],
    trace_event_count: u64,
    message_bytes: u64,
    block_count: u64,
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    if final_digest.len() != 32 || expected_count.len() != 4 {
        return Err(CheckpointError::Invariant);
    }
    let payload = control_view.payload_bits()?;
    if payload.len() < UNIQUENESS_LIST_COMMON_BYTES_V2 {
        return Err(CheckpointError::Invariant);
    }
    for (actual, expected) in [
        (
            &payload[0],
            constant_byte_bits(HashControlSchemaV2::UniquenessList as u8, zero, one),
        ),
        (&payload[1], constant_byte_bits(job.role_tag(), zero, one)),
        (&payload[2], constant_byte_bits(stage as u8, zero, one)),
        (&payload[51], constant_byte_bits(job as u8, zero, one)),
    ] {
        connect_byte_bits(builder, actual, &expected);
    }
    for (binding, digest) in payload[3..35].iter().zip(final_digest.iter()) {
        connect_byte_bits(builder, binding, digest);
    }
    connect_bytes_to_constants(
        builder,
        &payload[35..43],
        &message_bytes.to_le_bytes(),
        zero,
        one,
    )?;
    connect_bytes_to_constants(
        builder,
        &payload[43..51],
        &block_count.to_le_bytes(),
        zero,
        one,
    )?;
    for (actual, expected) in payload[52..56].iter().zip(expected_count.iter()) {
        connect_byte_bits(builder, actual, expected);
    }
    connect_bytes_to_constants(
        builder,
        &payload[56..64],
        &trace_event_count.to_le_bytes(),
        zero,
        one,
    )?;
    Ok(())
}

fn uniqueness_precommit_binding_bits(
    views: &[CircuitEventViewV2],
    job: UniquenessListHashJobV2,
) -> Result<(&[CircuitByteBitsV2], &[CircuitByteBitsV2]), CheckpointError> {
    let mut precommits = views
        .iter()
        .filter(|view| view.event.opcode() == RecursiveTraceOpcodeV2::UniquenessPrecommit);
    let precommit = precommits.next().ok_or(CheckpointError::Invariant)?;
    if precommits.next().is_some()
        || precommit.payload_bits()?.len() != UNIQUENESS_PRECOMMIT_BYTES_V2
    {
        return Err(CheckpointError::Invariant);
    }
    let payload = precommit.payload_bits()?;
    let (count, digest) = match job {
        UniquenessListHashJobV2::SpentOriginal => (&payload[1..5], &payload[9..41]),
        UniquenessListHashJobV2::OutputOriginal => (&payload[5..9], &payload[73..105]),
        UniquenessListHashJobV2::SpentSorted => (&payload[1..5], &payload[41..73]),
        UniquenessListHashJobV2::OutputSorted => (&payload[5..9], &payload[105..137]),
    };
    Ok((count, digest))
}

fn constrain_uniqueness_list_bindings(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    views: &[CircuitEventViewV2],
    zero: ExprId,
    one: ExprId,
) -> Result<usize, CheckpointError> {
    let trace_event_count = u64::try_from(
        views
            .iter()
            .filter(|view| view.event.opcode().is_source_record())
            .count(),
    )
    .map_err(|_| CheckpointError::Limit)?;
    let mut constrained_jobs = 0_usize;
    for job in UniquenessListHashJobV2::ALL {
        let (expected_count, expected_digest) = uniqueness_precommit_binding_bits(views, job)?;
        connect_byte_bits(
            builder,
            &views
                .iter()
                .find(|view| view.event.opcode() == RecursiveTraceOpcodeV2::UniquenessPrecommit)
                .ok_or(CheckpointError::Invariant)?
                .payload_bits()?[0],
            &constant_byte_bits(1, zero, one),
        );
        let (set, list) = job.row();
        let mut rows = Vec::new();
        for view in views
            .iter()
            .filter(|view| view.event.opcode() == RecursiveTraceOpcodeV2::UniquenessSorted)
        {
            let (pass, row_set, row_list, _) = decode_uniqueness_sorted_row(view.event.payload())?;
            if pass == UniquenessPassV2::Commit && (row_set, row_list) == (set, list) {
                let payload = view.payload_bits()?;
                if payload.len() != 4 + UNIQUENESS_SEMANTIC_ROW_BYTES_V2 {
                    return Err(CheckpointError::Invariant);
                }
                rows.push(payload[4..].to_vec());
            }
        }
        let row_count = u32::try_from(rows.len()).map_err(|_| CheckpointError::Limit)?;
        connect_bytes_to_constants(builder, expected_count, &row_count.to_le_bytes(), zero, one)?;
        let mut parts = Vec::with_capacity(rows.len() + 1);
        parts.push(expected_count.to_vec());
        parts.extend(rows);
        let (message, message_bytes, block_count) =
            framed_parts_message_bits(job.role(), &parts, zero, one)?;
        let expected_block_count =
            usize::try_from(block_count).map_err(|_| CheckpointError::Limit)?;
        let mut begin = None;
        let mut end = None;
        let mut blocks: Vec<Option<&CircuitEventViewV2>> = vec![None; expected_block_count];
        for view in views.iter().filter(|view| {
            matches!(
                view.event.opcode(),
                RecursiveTraceOpcodeV2::BeginHash
                    | RecursiveTraceOpcodeV2::ShaBlock
                    | RecursiveTraceOpcodeV2::EndHash
            )
        }) {
            let control = decode_hash_control(&view.event)?;
            if control.schema != HashControlSchemaV2::UniquenessList
                || control.uniqueness_list.map(|binding| binding.job) != Some(job)
            {
                continue;
            }
            let binding = control.uniqueness_list.ok_or(CheckpointError::Invariant)?;
            if binding.count != row_count
                || binding.trace_event_count != trace_event_count
                || control.message_bytes != message_bytes
                || control.block_count != block_count
            {
                return Err(CheckpointError::Invariant);
            }
            match control.stage {
                HashControlStageV2::Begin => {
                    if begin.replace(view).is_some() {
                        return Err(CheckpointError::Invariant);
                    }
                }
                HashControlStageV2::End => {
                    if end.replace(view).is_some() {
                        return Err(CheckpointError::Invariant);
                    }
                }
                HashControlStageV2::Block => {
                    let index =
                        usize::try_from(control.block.ok_or(CheckpointError::Invariant)?.index)
                            .map_err(|_| CheckpointError::Limit)?;
                    let slot = blocks.get_mut(index).ok_or(CheckpointError::Invariant)?;
                    if slot.replace(view).is_some() {
                        return Err(CheckpointError::Invariant);
                    }
                }
            }
        }
        let begin = begin.ok_or(CheckpointError::Invariant)?;
        let end = end.ok_or(CheckpointError::Invariant)?;
        let mut previous_after: Option<Vec<[ExprId; 8]>> = None;
        for (index, block_view) in blocks.into_iter().enumerate() {
            let block_view = block_view.ok_or(CheckpointError::Invariant)?;
            let payload = block_view.payload_bits()?;
            if payload.len() != UNIQUENESS_LIST_COMMON_BYTES_V2 + HASH_CONTROL_BLOCK_BYTES_V2 {
                return Err(CheckpointError::Invariant);
            }
            constrain_uniqueness_list_control_common(
                builder,
                block_view,
                job,
                HashControlStageV2::Block,
                expected_digest,
                expected_count,
                trace_event_count,
                message_bytes,
                block_count,
                zero,
                one,
            )?;
            let block_index_start = UNIQUENESS_LIST_COMMON_BYTES_V2;
            let block_offset_start = block_index_start + 8;
            let block_start = block_offset_start + 8;
            let before_start = block_start + 64;
            let after_start = before_start + 32;
            connect_bytes_to_constants(
                builder,
                &payload[block_index_start..block_index_start + 8],
                &u64::try_from(index)
                    .map_err(|_| CheckpointError::Limit)?
                    .to_le_bytes(),
                zero,
                one,
            )?;
            connect_bytes_to_constants(
                builder,
                &payload[block_offset_start..block_offset_start + 8],
                &u64::try_from(index)
                    .map_err(|_| CheckpointError::Limit)?
                    .checked_mul(64)
                    .ok_or(CheckpointError::Overflow)?
                    .to_le_bytes(),
                zero,
                one,
            )?;
            for (actual, expected) in payload[block_start..block_start + 64]
                .iter()
                .zip(message[index * 64..index * 64 + 64].iter())
            {
                connect_byte_bits(builder, actual, expected);
            }
            if let Some(previous) = previous_after.as_ref() {
                for (actual, expected) in payload[before_start..before_start + 32]
                    .iter()
                    .zip(previous.iter())
                {
                    connect_byte_bits(builder, actual, expected);
                }
            } else {
                let iv_bytes: Vec<u8> = SHA256_IV_V2
                    .iter()
                    .flat_map(|word| word.to_be_bytes())
                    .collect();
                connect_bytes_to_constants(
                    builder,
                    &payload[before_start..before_start + 32],
                    &iv_bytes,
                    zero,
                    one,
                )?;
            }
            connect_byte_bits(
                builder,
                &payload[after_start + 32],
                &constant_byte_bits(u8::from(index + 1 == expected_block_count), zero, one),
            );
            previous_after = Some(payload[after_start..after_start + 32].to_vec());
        }
        let final_digest = previous_after.ok_or(CheckpointError::Invariant)?;
        for (claimed, computed) in expected_digest.iter().zip(final_digest.iter()) {
            connect_byte_bits(builder, claimed, computed);
        }
        constrain_uniqueness_list_control_common(
            builder,
            begin,
            job,
            HashControlStageV2::Begin,
            &final_digest,
            expected_count,
            trace_event_count,
            message_bytes,
            block_count,
            zero,
            one,
        )?;
        constrain_uniqueness_list_control_common(
            builder,
            end,
            job,
            HashControlStageV2::End,
            &final_digest,
            expected_count,
            trace_event_count,
            message_bytes,
            block_count,
            zero,
            one,
        )?;
        constrained_jobs = constrained_jobs
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
    }
    Ok(constrained_jobs)
}

#[allow(clippy::too_many_arguments)]
fn constrain_uniqueness_transcript_control_common(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    control_view: &CircuitEventViewV2,
    job: UniquenessTranscriptHashJobV2,
    stage: HashControlStageV2,
    final_digest: &[[ExprId; 8]],
    trace_event_count: u64,
    message_bytes: u64,
    block_count: u64,
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    if final_digest.len() != 32 {
        return Err(CheckpointError::Invariant);
    }
    let payload = control_view.payload_bits()?;
    if payload.len() < UNIQUENESS_TRANSCRIPT_COMMON_BYTES_V2 {
        return Err(CheckpointError::Invariant);
    }
    for (actual, expected) in [
        (
            &payload[0],
            constant_byte_bits(HashControlSchemaV2::UniquenessTranscript as u8, zero, one),
        ),
        (&payload[1], constant_byte_bits(job.role_tag(), zero, one)),
        (&payload[2], constant_byte_bits(stage as u8, zero, one)),
        (&payload[51], constant_byte_bits(job as u8, zero, one)),
    ] {
        connect_byte_bits(builder, actual, &expected);
    }
    for (binding, digest) in payload[3..35].iter().zip(final_digest.iter()) {
        connect_byte_bits(builder, binding, digest);
    }
    connect_bytes_to_constants(
        builder,
        &payload[35..43],
        &message_bytes.to_le_bytes(),
        zero,
        one,
    )?;
    connect_bytes_to_constants(
        builder,
        &payload[43..51],
        &block_count.to_le_bytes(),
        zero,
        one,
    )?;
    connect_bytes_to_constants(
        builder,
        &payload[52..60],
        &trace_event_count.to_le_bytes(),
        zero,
        one,
    )
}

#[allow(clippy::too_many_arguments)]
fn constrain_uniqueness_transcript_job(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    views: &[CircuitEventViewV2],
    job: UniquenessTranscriptHashJobV2,
    expected_digest: &[[ExprId; 8]],
    parts: Option<&[Vec<[ExprId; 8]>]>,
    trace_event_count: u64,
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    let mut begin = None;
    let mut end = None;
    let mut message_bytes = None;
    let mut block_count = None;
    let mut blocks: Vec<Option<&CircuitEventViewV2>> = Vec::new();
    for view in views.iter().filter(|view| {
        matches!(
            view.event.opcode(),
            RecursiveTraceOpcodeV2::BeginHash
                | RecursiveTraceOpcodeV2::ShaBlock
                | RecursiveTraceOpcodeV2::EndHash
        )
    }) {
        let control = decode_hash_control(&view.event)?;
        if control.schema != HashControlSchemaV2::UniquenessTranscript
            || control.uniqueness_transcript.map(|binding| binding.job) != Some(job)
        {
            continue;
        }
        let binding = control
            .uniqueness_transcript
            .ok_or(CheckpointError::Invariant)?;
        if binding.trace_event_count != trace_event_count {
            return Err(CheckpointError::Invariant);
        }
        match (message_bytes, block_count) {
            (None, None) => {
                message_bytes = Some(control.message_bytes);
                block_count = Some(control.block_count);
                blocks.resize(
                    usize::try_from(control.block_count).map_err(|_| CheckpointError::Limit)?,
                    None,
                );
            }
            (Some(message), Some(blocks)) => {
                if message != control.message_bytes || blocks != control.block_count {
                    return Err(CheckpointError::Invariant);
                }
            }
            _ => return Err(CheckpointError::Invariant),
        }
        match control.stage {
            HashControlStageV2::Begin => {
                if begin.replace(view).is_some() {
                    return Err(CheckpointError::Invariant);
                }
            }
            HashControlStageV2::End => {
                if end.replace(view).is_some() {
                    return Err(CheckpointError::Invariant);
                }
            }
            HashControlStageV2::Block => {
                let index = usize::try_from(control.block.ok_or(CheckpointError::Invariant)?.index)
                    .map_err(|_| CheckpointError::Limit)?;
                let slot = blocks.get_mut(index).ok_or(CheckpointError::Invariant)?;
                if slot.replace(view).is_some() {
                    return Err(CheckpointError::Invariant);
                }
            }
        }
    }
    let begin = begin.ok_or(CheckpointError::Invariant)?;
    let end = end.ok_or(CheckpointError::Invariant)?;
    let message_bytes = message_bytes.ok_or(CheckpointError::Invariant)?;
    let block_count = block_count.ok_or(CheckpointError::Invariant)?;
    if CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(message_bytes)
        .map_err(|_| CheckpointError::Limit)?
        != block_count
    {
        return Err(CheckpointError::Invariant);
    }
    let expected_message = match parts {
        Some(parts) => {
            let (message, expected_bytes, expected_blocks) =
                framed_parts_message_bits(job.role(), parts, zero, one)?;
            if expected_bytes != message_bytes || expected_blocks != block_count {
                return Err(CheckpointError::Invariant);
            }
            Some(message)
        }
        None => None,
    };
    let padded_bytes = usize::try_from(
        block_count
            .checked_mul(64)
            .ok_or(CheckpointError::Overflow)?,
    )
    .map_err(|_| CheckpointError::Limit)?;
    let mut actual_message = Vec::with_capacity(padded_bytes);
    let mut previous_after: Option<Vec<[ExprId; 8]>> = None;
    let expected_block_count = blocks.len();
    for (index, block_view) in blocks.into_iter().enumerate() {
        let block_view = block_view.ok_or(CheckpointError::Invariant)?;
        let payload = block_view.payload_bits()?;
        if payload.len() != UNIQUENESS_TRANSCRIPT_COMMON_BYTES_V2 + HASH_CONTROL_BLOCK_BYTES_V2 {
            return Err(CheckpointError::Invariant);
        }
        constrain_uniqueness_transcript_control_common(
            builder,
            block_view,
            job,
            HashControlStageV2::Block,
            expected_digest,
            trace_event_count,
            message_bytes,
            block_count,
            zero,
            one,
        )?;
        let block_index_start = UNIQUENESS_TRANSCRIPT_COMMON_BYTES_V2;
        let block_offset_start = block_index_start + 8;
        let block_start = block_offset_start + 8;
        let before_start = block_start + 64;
        let after_start = before_start + 32;
        connect_bytes_to_constants(
            builder,
            &payload[block_index_start..block_index_start + 8],
            &u64::try_from(index)
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes(),
            zero,
            one,
        )?;
        connect_bytes_to_constants(
            builder,
            &payload[block_offset_start..block_offset_start + 8],
            &u64::try_from(index)
                .map_err(|_| CheckpointError::Limit)?
                .checked_mul(64)
                .ok_or(CheckpointError::Overflow)?
                .to_le_bytes(),
            zero,
            one,
        )?;
        let block_bits = &payload[block_start..block_start + 64];
        if let Some(expected) = expected_message.as_ref() {
            for (actual, expected) in block_bits
                .iter()
                .zip(expected[index * 64..index * 64 + 64].iter())
            {
                connect_byte_bits(builder, actual, expected);
            }
        }
        actual_message.extend_from_slice(block_bits);
        if let Some(previous) = previous_after.as_ref() {
            for (actual, expected) in payload[before_start..before_start + 32]
                .iter()
                .zip(previous.iter())
            {
                connect_byte_bits(builder, actual, expected);
            }
        } else {
            let iv_bytes: Vec<u8> = SHA256_IV_V2
                .iter()
                .flat_map(|word| word.to_be_bytes())
                .collect();
            connect_bytes_to_constants(
                builder,
                &payload[before_start..before_start + 32],
                &iv_bytes,
                zero,
                one,
            )?;
        }
        connect_byte_bits(
            builder,
            &payload[after_start + 32],
            &constant_byte_bits(u8::from(index + 1 == expected_block_count), zero, one),
        );
        previous_after = Some(payload[after_start..after_start + 32].to_vec());
    }
    if expected_message.is_none() {
        let message_len = usize::try_from(message_bytes).map_err(|_| CheckpointError::Limit)?;
        if actual_message.len() != padded_bytes || message_len > padded_bytes.saturating_sub(9) {
            return Err(CheckpointError::Invariant);
        }
        let prefix = CheckpointSha256BlockStreamV2::framed_role_prefix(job.role());
        if prefix.len() > message_len {
            return Err(CheckpointError::Invariant);
        }
        connect_bytes_to_constants(builder, &actual_message[..prefix.len()], &prefix, zero, one)?;
        connect_byte_bits(
            builder,
            &actual_message[message_len],
            &constant_byte_bits(0x80, zero, one),
        );
        for byte in &actual_message[message_len + 1..padded_bytes - 8] {
            connect_byte_bits(builder, byte, &constant_byte_bits(0, zero, one));
        }
        connect_bytes_to_constants(
            builder,
            &actual_message[padded_bytes - 8..],
            &message_bytes
                .checked_mul(8)
                .ok_or(CheckpointError::Overflow)?
                .to_be_bytes(),
            zero,
            one,
        )?;
    }
    let final_digest = previous_after.ok_or(CheckpointError::Invariant)?;
    for (claimed, computed) in expected_digest.iter().zip(final_digest.iter()) {
        connect_byte_bits(builder, claimed, computed);
    }
    constrain_uniqueness_transcript_control_common(
        builder,
        begin,
        job,
        HashControlStageV2::Begin,
        &final_digest,
        trace_event_count,
        message_bytes,
        block_count,
        zero,
        one,
    )?;
    constrain_uniqueness_transcript_control_common(
        builder,
        end,
        job,
        HashControlStageV2::End,
        &final_digest,
        trace_event_count,
        message_bytes,
        block_count,
        zero,
        one,
    )
}

#[allow(clippy::too_many_arguments)]
fn constrain_uniqueness_transcript_bindings(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    views: &[CircuitEventViewV2],
    statement: &[[ExprId; 8]],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<usize, CheckpointError> {
    let mut precommits = views
        .iter()
        .filter(|view| view.event.opcode() == RecursiveTraceOpcodeV2::UniquenessPrecommit);
    let precommit = precommits.next().ok_or(CheckpointError::Invariant)?;
    let mut challenges = views
        .iter()
        .filter(|view| view.event.opcode() == RecursiveTraceOpcodeV2::UniquenessChallenge);
    let challenge = challenges.next().ok_or(CheckpointError::Invariant)?;
    if precommits.next().is_some()
        || challenges.next().is_some()
        || precommit.payload_bits()?.len() != UNIQUENESS_PRECOMMIT_BYTES_V2
        || challenge.payload_bits()?.len() != UNIQUENESS_CHALLENGE_BYTES_V2
    {
        return Err(CheckpointError::Invariant);
    }
    let precommit_payload = precommit.payload_bits()?;
    let challenge_payload = challenge.payload_bits()?;
    connect_byte_bits(
        builder,
        &precommit_payload[0],
        &constant_byte_bits(1, zero, one),
    );
    connect_byte_bits(
        builder,
        &challenge_payload[0],
        &constant_byte_bits(1, zero, one),
    );

    let precommit_parts = vec![
        UNIQUENESS_PRECOMMIT_LABEL_V2
            .iter()
            .copied()
            .map(|byte| constant_byte_bits(byte, zero, one))
            .collect(),
        precommit_payload[1..5].to_vec(),
        precommit_payload[5..9].to_vec(),
        precommit_payload[9..41].to_vec(),
        precommit_payload[41..73].to_vec(),
        precommit_payload[73..105].to_vec(),
        precommit_payload[105..137].to_vec(),
    ];
    let (precommit_message, _, _) =
        framed_parts_message_bits(CheckpointShaRole::IdPrecommit, &precommit_parts, zero, one)?;
    let computed_precommit =
        circuit_sha256_padded_message_digest(builder, &precommit_message, zero, one, two)?;
    for ((stored, committed), computed) in precommit_payload[137..169]
        .iter()
        .zip(challenge_payload[1..33].iter())
        .zip(computed_precommit.iter())
    {
        connect_byte_bits(builder, stored, computed);
        connect_byte_bits(builder, committed, computed);
    }

    let grammar = statement_digest_bits(
        statement,
        PLONKY3_STATEMENT_DIGESTS_OFFSET_V2,
        PLONKY3_STATEMENT_GRAMMAR_DIGEST_INDEX_V2,
    )?;
    connect_bytes_to_constants(
        builder,
        grammar,
        &RecursiveTraceOpcodeV2::grammar_digest(),
        zero,
        one,
    )?;
    let declared_work = statement_digest_bits(
        statement,
        PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
        PLONKY3_STATEMENT_DECLARED_WORK_INDEX_V2,
    )?;
    let pre_uniqueness = statement_digest_bits(
        statement,
        PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
        PLONKY3_STATEMENT_PRE_UNIQUENESS_INDEX_V2,
    )?;
    for (statement_bit, source_bit) in pre_uniqueness.iter().zip(challenge_payload[33..65].iter()) {
        connect_byte_bits(builder, statement_bit, source_bit);
    }
    let spent_precommit = statement_digest_bits(
        statement,
        PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
        PLONKY3_STATEMENT_SPENT_PRECOMMIT_INDEX_V2,
    )?;
    let output_precommit = statement_digest_bits(
        statement,
        PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
        PLONKY3_STATEMENT_OUTPUT_PRECOMMIT_INDEX_V2,
    )?;
    for (statement_bit, source_bit) in spent_precommit.iter().zip(challenge_payload[65..97].iter())
    {
        connect_byte_bits(builder, statement_bit, source_bit);
    }
    for (statement_bit, source_bit) in output_precommit
        .iter()
        .zip(challenge_payload[97..129].iter())
    {
        connect_byte_bits(builder, statement_bit, source_bit);
    }

    let trace_event_count = u64::try_from(
        views
            .iter()
            .filter(|view| view.event.opcode().is_source_record())
            .count(),
    )
    .map_err(|_| CheckpointError::Limit)?;
    let pre_settlement = statement_digest_bits(
        statement,
        PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
        PLONKY3_STATEMENT_PRE_SETTLEMENT_INDEX_V2,
    )?;
    let post_settlement = statement_digest_bits(
        statement,
        PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
        PLONKY3_STATEMENT_POST_SETTLEMENT_INDEX_V2,
    )?;
    let mut constrained = 0_usize;
    for job in UniquenessTranscriptHashJobV2::ALL {
        let (expected, parts): (&[[ExprId; 8]], Option<Vec<Vec<[ExprId; 8]>>>) = match job {
            UniquenessTranscriptHashJobV2::DeclaredCounts => (declared_work, None),
            UniquenessTranscriptHashJobV2::PreUniquenessContext => (pre_uniqueness, None),
            UniquenessTranscriptHashJobV2::SpentPrecommit => (
                &challenge_payload[65..97],
                Some(vec![
                    challenge_payload[33..65].to_vec(),
                    vec![constant_byte_bits(0, zero, one)],
                    precommit_payload[1..5].to_vec(),
                    precommit_payload[9..41].to_vec(),
                    precommit_payload[41..73].to_vec(),
                ]),
            ),
            UniquenessTranscriptHashJobV2::OutputPrecommit => (
                &challenge_payload[97..129],
                Some(vec![
                    challenge_payload[33..65].to_vec(),
                    vec![constant_byte_bits(1, zero, one)],
                    precommit_payload[5..9].to_vec(),
                    precommit_payload[73..105].to_vec(),
                    precommit_payload[105..137].to_vec(),
                ]),
            ),
            UniquenessTranscriptHashJobV2::SettlementPreRoot => (pre_settlement, None),
            UniquenessTranscriptHashJobV2::SettlementPostRoot => (post_settlement, None),
            _ => {
                let set = job.set().ok_or(CheckpointError::Invariant)? as u8;
                let (pair, coordinate) = job
                    .challenge_coordinate()
                    .ok_or(CheckpointError::Invariant)?;
                let set_precommit = if set == 0 {
                    &challenge_payload[65..97]
                } else {
                    &challenge_payload[97..129]
                };
                let challenge_index = usize::from(job as u8)
                    .checked_sub(4)
                    .ok_or(CheckpointError::Invariant)?;
                let start = 129_usize
                    .checked_add(
                        challenge_index
                            .checked_mul(32)
                            .ok_or(CheckpointError::Overflow)?,
                    )
                    .ok_or(CheckpointError::Overflow)?;
                (
                    &challenge_payload[start..start + 32],
                    Some(vec![
                        set_precommit.to_vec(),
                        grammar.to_vec(),
                        vec![constant_byte_bits(set, zero, one)],
                        vec![constant_byte_bits(pair, zero, one)],
                        vec![constant_byte_bits(coordinate, zero, one)],
                    ]),
                )
            }
        };
        constrain_uniqueness_transcript_job(
            builder,
            views,
            job,
            expected,
            parts.as_deref(),
            trace_event_count,
            zero,
            one,
        )?;
        constrained = constrained
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
    }
    Ok(constrained)
}

fn constrain_sha_control_blocks(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    views: &[CircuitEventViewV2],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<usize, CheckpointError> {
    let mut constrained = 0_usize;
    for view in views {
        let event = &view.event;
        if event.opcode() != RecursiveTraceOpcodeV2::ShaBlock {
            continue;
        }
        let control = decode_hash_control(event)?;
        let block = control.block.ok_or(CheckpointError::Invariant)?;
        let common_bytes = event
            .payload()
            .len()
            .checked_sub(HASH_CONTROL_BLOCK_BYTES_V2)
            .ok_or(CheckpointError::Invariant)?;
        let block_start = common_bytes
            .checked_add(16)
            .ok_or(CheckpointError::Overflow)?;
        let before_start = block_start
            .checked_add(64)
            .ok_or(CheckpointError::Overflow)?;
        let after_start = before_start
            .checked_add(32)
            .ok_or(CheckpointError::Overflow)?;
        let payload_bits = view.payload_bits()?;
        let block_bits = payload_bits
            .get(block_start..block_start + 64)
            .ok_or(CheckpointError::Invariant)?;
        let before_bits = payload_bits
            .get(before_start..before_start + 32)
            .ok_or(CheckpointError::Invariant)?;
        let after_bits = payload_bits
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
                .ne(event.payload()[common_bytes + 80..common_bytes + 112]
                    .iter()
                    .copied())
            || block
                .chaining_after
                .iter()
                .flat_map(|word| word.to_be_bytes())
                .ne(event.payload()[common_bytes + 112..common_bytes + 144]
                    .iter()
                    .copied())
        {
            return Err(CheckpointError::Invariant);
        }
        constrained = constrained
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
    }
    Ok(constrained)
}

fn constrain_trace_precommit_bindings(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    views: &[CircuitEventViewV2],
    statement_bits: &[CircuitByteBitsV2],
    bind_statement: bool,
    zero: ExprId,
    one: ExprId,
) -> Result<usize, CheckpointError> {
    let sources: Vec<&CircuitEventViewV2> = views
        .iter()
        .filter(|view| view.event.opcode().is_source_record())
        .collect();
    if sources.is_empty() {
        return Err(CheckpointError::Invariant);
    }
    for (expected_ordinal, source) in sources.iter().enumerate() {
        if source.event.ordinal()
            != u64::try_from(expected_ordinal).map_err(|_| CheckpointError::Limit)?
        {
            return Err(CheckpointError::Invariant);
        }
    }
    let event_count = u64::try_from(sources.len()).map_err(|_| CheckpointError::Limit)?;
    let byte_count = sources.iter().try_fold(0_u64, |total, source| {
        total
            .checked_add(
                u64::try_from(source.canonical_bits.len()).map_err(|_| CheckpointError::Limit)?,
            )
            .ok_or(CheckpointError::Overflow)
    })?;
    let message_bytes = CheckpointSha256BlockStreamV2::framed_bytes_for_parts(
        CheckpointShaRole::Trace,
        byte_count,
        event_count,
    )
    .map_err(|_| CheckpointError::Limit)?;
    let block_count = CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(message_bytes)
        .map_err(|_| CheckpointError::Limit)?;
    let padding_bytes = block_count
        .checked_mul(64)
        .and_then(|bytes| bytes.checked_sub(message_bytes))
        .and_then(|bytes| bytes.checked_sub(9))
        .filter(|bytes| *bytes < 64)
        .ok_or(CheckpointError::Invariant)?;
    let bit_length = message_bytes
        .checked_mul(8)
        .ok_or(CheckpointError::Overflow)?;
    let expected_block_count = usize::try_from(block_count).map_err(|_| CheckpointError::Limit)?;
    let mut begin = None;
    let mut end = None;
    let mut blocks: Vec<Option<&CircuitEventViewV2>> = vec![None; expected_block_count];
    for view in views {
        if !matches!(
            view.event.opcode(),
            RecursiveTraceOpcodeV2::BeginHash
                | RecursiveTraceOpcodeV2::ShaBlock
                | RecursiveTraceOpcodeV2::EndHash
        ) {
            continue;
        }
        let control = decode_hash_control(&view.event)?;
        if control.schema != HashControlSchemaV2::TracePrecommit {
            continue;
        }
        let trace = control.trace.ok_or(CheckpointError::Invariant)?;
        if trace.event_count != event_count
            || trace.byte_count != byte_count
            || trace.padding_bytes != padding_bytes
            || trace.bit_length != bit_length
            || !trace.eof
            || control.message_bytes != message_bytes
            || control.block_count != block_count
        {
            return Err(CheckpointError::Invariant);
        }
        match control.stage {
            HashControlStageV2::Begin => {
                if begin.replace(view).is_some() || control.block.is_some() {
                    return Err(CheckpointError::Invariant);
                }
            }
            HashControlStageV2::Block => {
                let block = control.block.ok_or(CheckpointError::Invariant)?;
                let index = usize::try_from(block.index).map_err(|_| CheckpointError::Limit)?;
                let slot = blocks.get_mut(index).ok_or(CheckpointError::Invariant)?;
                if slot.replace(view).is_some()
                    || block.byte_offset
                        != block
                            .index
                            .checked_mul(64)
                            .ok_or(CheckpointError::Overflow)?
                    || block.final_block != (index + 1 == expected_block_count)
                {
                    return Err(CheckpointError::Invariant);
                }
            }
            HashControlStageV2::End => {
                if end.replace(view).is_some() || control.block.is_some() {
                    return Err(CheckpointError::Invariant);
                }
            }
        }
    }
    let begin = begin.ok_or(CheckpointError::Invariant)?;
    let end = end.ok_or(CheckpointError::Invariant)?;
    if blocks.iter().any(Option::is_none) {
        return Err(CheckpointError::Invariant);
    }
    let message = trace_framed_message_bits(&sources, message_bytes, block_count, zero, one)?;
    let iv_bytes: Vec<u8> = SHA256_IV_V2
        .iter()
        .flat_map(|word| word.to_be_bytes())
        .collect();
    let mut previous_after: Option<Vec<[ExprId; 8]>> = None;
    for (block_index, block_view) in blocks.into_iter().enumerate() {
        let block_view = block_view.ok_or(CheckpointError::Invariant)?;
        let payload = block_view.payload_bits()?;
        let block_start = HASH_CONTROL_TRACE_COMMON_BYTES_V2 + 16;
        let before_start = block_start + 64;
        let after_start = before_start + 32;
        let message_start = block_index
            .checked_mul(64)
            .ok_or(CheckpointError::Overflow)?;
        for (actual, expected) in payload[block_start..block_start + 64]
            .iter()
            .zip(message[message_start..message_start + 64].iter())
        {
            connect_byte_bits(builder, actual, expected);
        }
        if let Some(previous_after) = previous_after.as_ref() {
            for (actual, previous) in payload[before_start..before_start + 32]
                .iter()
                .zip(previous_after.iter())
            {
                connect_byte_bits(builder, actual, previous);
            }
        } else {
            connect_bytes_to_constants(
                builder,
                &payload[before_start..before_start + 32],
                &iv_bytes,
                zero,
                one,
            )?;
        }
        previous_after = Some(payload[after_start..after_start + 32].to_vec());
    }
    let final_digest = previous_after.ok_or(CheckpointError::Invariant)?;
    if bind_statement {
        connect_bit_slices(
            builder,
            &final_digest,
            statement_digest_bits(
                statement_bits,
                PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
                PLONKY3_STATEMENT_TRACE_DIGEST_INDEX_V2,
            )?,
        )?;
    }
    constrain_trace_control_common(
        builder,
        begin,
        HashControlStageV2::Begin,
        &final_digest,
        event_count,
        byte_count,
        message_bytes,
        block_count,
        padding_bytes,
        bit_length,
        zero,
        one,
    )?;
    constrain_trace_control_common(
        builder,
        end,
        HashControlStageV2::End,
        &final_digest,
        event_count,
        byte_count,
        message_bytes,
        block_count,
        padding_bytes,
        bit_length,
        zero,
        one,
    )?;
    for block_view in views.iter().filter(|view| {
        decode_hash_control(&view.event)
            .map(|control| control.schema == HashControlSchemaV2::TracePrecommit)
            .unwrap_or(false)
            && view.event.opcode() == RecursiveTraceOpcodeV2::ShaBlock
    }) {
        constrain_trace_control_common(
            builder,
            block_view,
            HashControlStageV2::Block,
            &final_digest,
            event_count,
            byte_count,
            message_bytes,
            block_count,
            padding_bytes,
            bit_length,
            zero,
            one,
        )?;
    }
    Ok(expected_block_count)
}

fn constrain_source_record_bindings(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    views: &[CircuitEventViewV2],
    zero: ExprId,
    one: ExprId,
) -> Result<usize, CheckpointError> {
    let mut constrained_sources = 0_usize;
    for (source_index, source) in views.iter().enumerate() {
        if !source.event.opcode().is_source_record() {
            continue;
        }
        let segment_end = views[source_index + 1..]
            .iter()
            .position(|view| view.event.opcode().is_source_record())
            .map(|offset| source_index + 1 + offset)
            .unwrap_or(views.len());
        let segment = &views[source_index + 1..segment_end];
        let source_ordinal = source.event.ordinal();
        let expected_chunk_count = source.event.canonical_chunk_count()?;
        let (expected_message_bytes, expected_block_count) = source.event.hash_geometry()?;
        let expected_chunk_count_usize =
            usize::try_from(expected_chunk_count).map_err(|_| CheckpointError::Limit)?;
        let expected_block_count_usize =
            usize::try_from(expected_block_count).map_err(|_| CheckpointError::Limit)?;
        let mut memory_chunks: Vec<Option<&CircuitEventViewV2>> =
            vec![None; expected_chunk_count_usize];
        let mut trace_chunks: Vec<Option<&CircuitEventViewV2>> =
            vec![None; expected_chunk_count_usize];
        let mut hash_blocks: Vec<Option<&CircuitEventViewV2>> =
            vec![None; expected_block_count_usize];
        let mut begin_hash = None;
        let mut end_hash = None;

        for (segment_index, view) in segment.iter().enumerate() {
            match view.event.opcode() {
                RecursiveTraceOpcodeV2::SourceMemoryWrite => {
                    let control = decode_source_memory_write_control(&view.event)?;
                    if control.source_ordinal != source_ordinal {
                        continue;
                    }
                    let index = usize::try_from(control.chunk_ordinal)
                        .map_err(|_| CheckpointError::Limit)?;
                    let slot = memory_chunks
                        .get_mut(index)
                        .ok_or(CheckpointError::Invariant)?;
                    if slot.replace(view).is_some() {
                        return Err(CheckpointError::Invariant);
                    }
                    let paired = segment
                        .get(segment_index + 1)
                        .ok_or(CheckpointError::Invariant)?;
                    if paired.event.opcode() != RecursiveTraceOpcodeV2::TraceChunk
                        || decode_trace_chunk_control(&paired.event)? != control
                    {
                        return Err(CheckpointError::Invariant);
                    }
                }
                RecursiveTraceOpcodeV2::TraceChunk => {
                    let control = decode_trace_chunk_control(&view.event)?;
                    if control.source_ordinal != source_ordinal {
                        continue;
                    }
                    let index = usize::try_from(control.chunk_ordinal)
                        .map_err(|_| CheckpointError::Limit)?;
                    let slot = trace_chunks
                        .get_mut(index)
                        .ok_or(CheckpointError::Invariant)?;
                    if slot.replace(view).is_some() {
                        return Err(CheckpointError::Invariant);
                    }
                }
                RecursiveTraceOpcodeV2::BeginHash
                | RecursiveTraceOpcodeV2::ShaBlock
                | RecursiveTraceOpcodeV2::EndHash => {
                    let control = decode_hash_control(&view.event)?;
                    if control.schema != HashControlSchemaV2::SourceRecord {
                        continue;
                    }
                    let binding = control.source.ok_or(CheckpointError::Invariant)?;
                    if binding.ordinal != source_ordinal {
                        return Err(CheckpointError::Invariant);
                    }
                    if binding.opcode != source.event.opcode()
                        || binding.object_id != source.event.object_id()
                        || control.message_bytes != expected_message_bytes
                        || control.block_count != expected_block_count
                    {
                        return Err(CheckpointError::Invariant);
                    }
                    match control.stage {
                        HashControlStageV2::Begin => {
                            if begin_hash.replace(view).is_some() || control.block.is_some() {
                                return Err(CheckpointError::Invariant);
                            }
                        }
                        HashControlStageV2::Block => {
                            let block = control.block.ok_or(CheckpointError::Invariant)?;
                            let index =
                                usize::try_from(block.index).map_err(|_| CheckpointError::Limit)?;
                            let slot = hash_blocks
                                .get_mut(index)
                                .ok_or(CheckpointError::Invariant)?;
                            if slot.replace(view).is_some()
                                || block.byte_offset
                                    != block
                                        .index
                                        .checked_mul(64)
                                        .ok_or(CheckpointError::Overflow)?
                                || block.final_block != (index + 1 == expected_block_count_usize)
                            {
                                return Err(CheckpointError::Invariant);
                            }
                        }
                        HashControlStageV2::End => {
                            if end_hash.replace(view).is_some() || control.block.is_some() {
                                return Err(CheckpointError::Invariant);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        let begin_hash = begin_hash.ok_or(CheckpointError::Invariant)?;
        let end_hash = end_hash.ok_or(CheckpointError::Invariant)?;
        if memory_chunks.iter().any(Option::is_none)
            || trace_chunks.iter().any(Option::is_none)
            || hash_blocks.iter().any(Option::is_none)
        {
            return Err(CheckpointError::Invariant);
        }

        for chunk_index in 0..expected_chunk_count_usize {
            let memory = memory_chunks[chunk_index].ok_or(CheckpointError::Invariant)?;
            let trace = trace_chunks[chunk_index].ok_or(CheckpointError::Invariant)?;
            let memory_control = decode_source_memory_write_control(&memory.event)?;
            let trace_control = decode_trace_chunk_control(&trace.event)?;
            if memory_control != trace_control || memory_control.chunk_count != expected_chunk_count
            {
                return Err(CheckpointError::Invariant);
            }
            let source_start = chunk_index
                .checked_mul(TRACE_CANONICAL_CHUNK_BYTES_V2)
                .ok_or(CheckpointError::Overflow)?;
            let source_end = source_start
                .checked_add(TRACE_CANONICAL_CHUNK_BYTES_V2)
                .map(|end| end.min(source.canonical_bits.len()))
                .ok_or(CheckpointError::Overflow)?;
            let expected_byte_count =
                u8::try_from(source_end - source_start).map_err(|_| CheckpointError::Limit)?;
            if memory_control.byte_count != expected_byte_count {
                return Err(CheckpointError::Invariant);
            }
            let memory_payload = memory.payload_bits()?;
            let trace_payload = trace.payload_bits()?;
            if memory_payload.len() != TRACE_CONTROL_PAYLOAD_BYTES_V2
                || trace_payload.len() != memory_payload.len()
            {
                return Err(CheckpointError::Invariant);
            }
            for (memory_byte, trace_byte) in memory_payload.iter().zip(trace_payload.iter()) {
                connect_byte_bits(builder, memory_byte, trace_byte);
            }
            connect_byte_bits(
                builder,
                &memory_payload[0],
                &constant_byte_bits(TRACE_CHUNK_CONTROL_VERSION_V2, zero, one),
            );
            for (actual, source_byte) in memory_payload[1..9]
                .iter()
                .zip(source.canonical_bits[1..9].iter())
            {
                connect_byte_bits(builder, actual, source_byte);
            }
            connect_bytes_to_constants(
                builder,
                &memory_payload[9..13],
                &u32::try_from(chunk_index)
                    .map_err(|_| CheckpointError::Limit)?
                    .to_le_bytes(),
                zero,
                one,
            )?;
            connect_bytes_to_constants(
                builder,
                &memory_payload[13..17],
                &expected_chunk_count.to_le_bytes(),
                zero,
                one,
            )?;
            connect_byte_bits(
                builder,
                &memory_payload[17],
                &constant_byte_bits(expected_byte_count, zero, one),
            );
            for byte_index in 0..TRACE_CANONICAL_CHUNK_BYTES_V2 {
                let expected = source_start
                    .checked_add(byte_index)
                    .and_then(|index| source.canonical_bits.get(index).copied());
                let expected = expected.unwrap_or_else(|| constant_byte_bits(0, zero, one));
                connect_byte_bits(
                    builder,
                    &memory_payload[TRACE_CHUNK_CONTROL_HEADER_BYTES_V2 + byte_index],
                    &expected,
                );
            }
        }

        let message = source_framed_message_bits(source, zero, one)?;
        let iv_bytes: Vec<u8> = SHA256_IV_V2
            .iter()
            .flat_map(|word| word.to_be_bytes())
            .collect();
        let mut previous_after: Option<Vec<[ExprId; 8]>> = None;
        for (block_index, block_view) in hash_blocks.into_iter().enumerate() {
            let block_view = block_view.ok_or(CheckpointError::Invariant)?;
            let payload = block_view.payload_bits()?;
            let common_bytes = payload
                .len()
                .checked_sub(HASH_CONTROL_BLOCK_BYTES_V2)
                .ok_or(CheckpointError::Invariant)?;
            if common_bytes != HASH_CONTROL_SOURCE_COMMON_BYTES_V2 {
                return Err(CheckpointError::Invariant);
            }
            let block_start = common_bytes + 16;
            let before_start = block_start + 64;
            let after_start = before_start + 32;
            let message_start = block_index
                .checked_mul(64)
                .ok_or(CheckpointError::Overflow)?;
            for (actual, expected) in payload[block_start..block_start + 64]
                .iter()
                .zip(message[message_start..message_start + 64].iter())
            {
                connect_byte_bits(builder, actual, expected);
            }
            if let Some(previous_after) = previous_after.as_ref() {
                for (actual, previous) in payload[before_start..before_start + 32]
                    .iter()
                    .zip(previous_after.iter())
                {
                    connect_byte_bits(builder, actual, previous);
                }
            } else {
                connect_bytes_to_constants(
                    builder,
                    &payload[before_start..before_start + 32],
                    &iv_bytes,
                    zero,
                    one,
                )?;
            }
            previous_after = Some(payload[after_start..after_start + 32].to_vec());
        }
        let final_digest = previous_after.ok_or(CheckpointError::Invariant)?;
        constrain_source_control_common(
            builder,
            begin_hash,
            source,
            HashControlStageV2::Begin,
            expected_message_bytes,
            expected_block_count,
            &final_digest,
            zero,
            one,
        )?;
        constrain_source_control_common(
            builder,
            end_hash,
            source,
            HashControlStageV2::End,
            expected_message_bytes,
            expected_block_count,
            &final_digest,
            zero,
            one,
        )?;
        for block_view in views[source_index + 1..segment_end]
            .iter()
            .filter(|view| view.event.opcode() == RecursiveTraceOpcodeV2::ShaBlock)
        {
            let control = decode_hash_control(&block_view.event)?;
            if control.schema != HashControlSchemaV2::SourceRecord
                || control.source.map(|binding| binding.ordinal) != Some(source_ordinal)
            {
                continue;
            }
            constrain_source_control_common(
                builder,
                block_view,
                source,
                HashControlStageV2::Block,
                expected_message_bytes,
                expected_block_count,
                &final_digest,
                zero,
                one,
            )?;
        }
        constrained_sources = constrained_sources
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
    }
    Ok(constrained_sources)
}

fn take_source_view<'a>(
    sources: &[&'a CircuitEventViewV2],
    cursor: &mut usize,
    opcode: RecursiveTraceOpcodeV2,
) -> Result<&'a CircuitEventViewV2, CheckpointError> {
    let view = sources
        .get(*cursor)
        .copied()
        .ok_or(CheckpointError::Invariant)?;
    if view.event.opcode() != opcode {
        return Err(CheckpointError::Invariant);
    }
    *cursor = cursor.checked_add(1).ok_or(CheckpointError::Overflow)?;
    Ok(view)
}

fn uniqueness_row_bits(view: &CircuitEventViewV2) -> Result<&[CircuitByteBitsV2], CheckpointError> {
    let payload = view.payload_bits()?;
    if payload.len() != UNIQUENESS_SORTED_ROW_BYTES_V2 {
        return Err(CheckpointError::Invariant);
    }
    payload
        .get(4..4 + UNIQUENESS_SEMANTIC_ROW_BYTES_V2)
        .ok_or(CheckpointError::Invariant)
}

fn circuit_little_endian_bits_value(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    bits: impl IntoIterator<Item = ExprId>,
    zero: ExprId,
) -> Result<ExprId, CheckpointError> {
    let mut value = zero;
    for (bit_index, bit) in bits.into_iter().enumerate() {
        let weight = 1_u64
            .checked_shl(u32::try_from(bit_index).map_err(|_| CheckpointError::Limit)?)
            .ok_or(CheckpointError::Overflow)?;
        let weight = builder.alloc_const(
            Plonky3TraceFieldV2::from_u64(weight),
            "little_endian_bit_weight",
        );
        value = builder.mul_add(bit, weight, value);
    }
    Ok(value)
}

fn circuit_little_endian_u16(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    bytes: &[CircuitByteBitsV2],
    zero: ExprId,
) -> Result<ExprId, CheckpointError> {
    if bytes.len() != 2 {
        return Err(CheckpointError::Invariant);
    }
    circuit_little_endian_bits_value(
        builder,
        bytes.iter().flat_map(|byte| byte.iter().copied()),
        zero,
    )
}

fn circuit_byte_equals_constant(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    byte: &CircuitByteBitsV2,
    expected: u8,
    one: ExprId,
) -> ExprId {
    byte.iter()
        .enumerate()
        .fold(one, |equal, (bit_index, bit)| {
            let matching_bit = if (expected >> bit_index) & 1 == 0 {
                builder.sub(one, *bit)
            } else {
                *bit
            };
            builder.mul(equal, matching_bit)
        })
}

fn circuit_decode_lower_hex_nibble(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    encoded: &CircuitByteBitsV2,
    zero: ExprId,
    one: ExprId,
) -> [ExprId; 4] {
    let selectors = core::array::from_fn::<_, 16, _>(|nibble| {
        let encoded_value = if nibble < 10 {
            b'0' + u8::try_from(nibble).expect("hex nibble fits u8")
        } else {
            b'a' + u8::try_from(nibble - 10).expect("hex nibble fits u8")
        };
        circuit_byte_equals_constant(builder, encoded, encoded_value, one)
    });
    let valid = selectors
        .iter()
        .copied()
        .fold(zero, |sum, selector| builder.add(sum, selector));
    builder.connect(valid, one);
    core::array::from_fn(|bit| {
        selectors
            .iter()
            .copied()
            .enumerate()
            .filter(|(nibble, _)| (nibble >> bit) & 1 == 1)
            .map(|(_, selector)| selector)
            .fold(zero, |sum, selector| builder.add(sum, selector))
    })
}

fn circuit_decode_lower_hex32(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    encoded: &[CircuitByteBitsV2],
    zero: ExprId,
    one: ExprId,
) -> Result<Vec<CircuitByteBitsV2>, CheckpointError> {
    if encoded.len() != 64 {
        return Err(CheckpointError::Invariant);
    }
    encoded
        .chunks_exact(2)
        .map(|pair| {
            let high = circuit_decode_lower_hex_nibble(builder, &pair[0], zero, one);
            let low = circuit_decode_lower_hex_nibble(builder, &pair[1], zero, one);
            Ok(core::array::from_fn(|bit| {
                if bit < 4 {
                    low[bit]
                } else {
                    high[bit - 4]
                }
            }))
        })
        .collect()
}

fn constrain_uniqueness_row_header(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    view: &CircuitEventViewV2,
    pass: UniquenessPassV2,
    set: UniquenessSetKindV2,
    list: UniquenessListKindV2,
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    let payload = view.payload_bits()?;
    if payload.len() != UNIQUENESS_SORTED_ROW_BYTES_V2 {
        return Err(CheckpointError::Invariant);
    }
    connect_bytes_to_constants(
        builder,
        &payload[..4],
        &[1, pass as u8, set as u8, list as u8],
        zero,
        one,
    )
}

fn constrain_net_effect_from_rows(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    net: &CircuitEventViewV2,
    spent: Option<&CircuitEventViewV2>,
    output: Option<&CircuitEventViewV2>,
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    let payload = net.payload_bits()?;
    if payload.len() != NET_MERGE_BYTES_V2 {
        return Err(CheckpointError::Invariant);
    }
    let kind = match (spent, output) {
        (Some(_), Some(_)) => decode_net_effect(net.event.payload())?.kind as u8,
        (Some(_), None) => 1,
        (None, Some(_)) => 2,
        (None, None) => return Err(CheckpointError::Invariant),
    };
    connect_bytes_to_constants(builder, &payload[..2], &[1, kind], zero, one)?;
    match (spent, output) {
        (Some(spent), Some(output)) => {
            let spent = uniqueness_row_bits(spent)?;
            let output = uniqueness_row_bits(output)?;
            connect_bit_slices(builder, &spent[..68], &output[..68])?;
            connect_bit_slices(builder, &payload[2..102], spent)?;
            connect_bit_slices(builder, &payload[102..134], &output[68..100])?;
            if kind == 4 {
                connect_bit_slices(builder, &spent[68..100], &output[68..100])?;
            } else if kind == 3 {
                constrain_bit_slices_not_equal(
                    builder,
                    &spent[68..100],
                    &output[68..100],
                    zero,
                    one,
                )?;
            } else {
                return Err(CheckpointError::Invariant);
            }
        }
        (Some(spent), None) => {
            connect_bit_slices(builder, &payload[2..102], uniqueness_row_bits(spent)?)?;
            connect_bytes_to_constants(builder, &payload[102..134], &[0; 32], zero, one)?;
        }
        (None, Some(output)) => {
            let output = uniqueness_row_bits(output)?;
            connect_bit_slices(builder, &payload[2..70], &output[..68])?;
            connect_bytes_to_constants(builder, &payload[70..102], &[0; 32], zero, one)?;
            connect_bit_slices(builder, &payload[102..134], &output[68..100])?;
        }
        (None, None) => return Err(CheckpointError::Invariant),
    }
    Ok(())
}

fn constrain_bit_slices_not_equal(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    left: &[CircuitByteBitsV2],
    right: &[CircuitByteBitsV2],
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    if left.len() != right.len() || left.is_empty() {
        return Err(CheckpointError::Invariant);
    }
    let mut different = zero;
    for (left, right) in left.iter().zip(right) {
        for (left_bit, right_bit) in left.iter().copied().zip(right.iter().copied()) {
            let product = builder.mul(left_bit, right_bit);
            let doubled_product = builder.add(product, product);
            let sum = builder.add(left_bit, right_bit);
            let xor = builder.sub(sum, doubled_product);
            let overlap = builder.mul(different, xor);
            let union = builder.add(different, xor);
            different = builder.sub(union, overlap);
        }
    }
    builder.connect(different, one);
    Ok(())
}

fn constrain_replay_to_uniqueness_row(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    replay: &CircuitEventViewV2,
    committed: &CircuitEventViewV2,
    expected_op_kind: u8,
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    let payload = replay.payload_bits()?;
    let payload_bytes = replay.event.payload();
    if payload.len() != payload_bytes.len() || payload.len() < 1 + 2 {
        return Err(CheckpointError::Invariant);
    }
    connect_byte_bits(
        builder,
        &payload[0],
        &constant_byte_bits(expected_op_kind, zero, one),
    );
    let tx_len = usize::from(u16::from_le_bytes(
        payload_bytes[1..3]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    ));
    connect_bytes_to_constants(
        builder,
        &payload[1..3],
        &u16::try_from(tx_len)
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
        zero,
        one,
    )?;
    let mut cursor = 3_usize
        .checked_add(tx_len)
        .ok_or(CheckpointError::Overflow)?;
    let definition_len_end = cursor.checked_add(2).ok_or(CheckpointError::Overflow)?;
    let definition_len = usize::from(u16::from_le_bytes(
        payload_bytes
            .get(cursor..definition_len_end)
            .ok_or(CheckpointError::Canonical)?
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    ));
    if definition_len != 64 {
        return Err(CheckpointError::Canonical);
    }
    connect_bytes_to_constants(
        builder,
        &payload[cursor..definition_len_end],
        &64_u16.to_le_bytes(),
        zero,
        one,
    )?;
    cursor = definition_len_end;
    let definition_end = cursor
        .checked_add(definition_len)
        .ok_or(CheckpointError::Overflow)?;
    let definition =
        circuit_decode_lower_hex32(builder, &payload[cursor..definition_end], zero, one)?;
    cursor = definition_end;
    let serial_end = cursor.checked_add(4).ok_or(CheckpointError::Overflow)?;
    let serial = payload
        .get(cursor..serial_end)
        .ok_or(CheckpointError::Canonical)?;
    cursor = serial_end;
    let terminal_len_end = cursor.checked_add(2).ok_or(CheckpointError::Overflow)?;
    let terminal_len = usize::from(u16::from_le_bytes(
        payload_bytes
            .get(cursor..terminal_len_end)
            .ok_or(CheckpointError::Canonical)?
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    ));
    if terminal_len != 64 {
        return Err(CheckpointError::Canonical);
    }
    connect_bytes_to_constants(
        builder,
        &payload[cursor..terminal_len_end],
        &64_u16.to_le_bytes(),
        zero,
        one,
    )?;
    cursor = terminal_len_end;
    let terminal_end = cursor
        .checked_add(terminal_len)
        .ok_or(CheckpointError::Overflow)?;
    let terminal = circuit_decode_lower_hex32(builder, &payload[cursor..terminal_end], zero, one)?;
    cursor = terminal_end;
    let leaf_end = cursor.checked_add(32).ok_or(CheckpointError::Overflow)?;
    let leaf = payload
        .get(cursor..leaf_end)
        .ok_or(CheckpointError::Canonical)?;
    let row = uniqueness_row_bits(committed)?;
    connect_bit_slices(builder, &definition, &row[..32])?;
    connect_bit_slices(builder, serial, &row[32..36])?;
    connect_bit_slices(builder, &terminal, &row[36..68])?;
    connect_bit_slices(builder, leaf, &row[68..100])?;
    connect_bit_slices(builder, &terminal, &replay.canonical_bits[9..41])?;
    Ok(())
}

struct CircuitFlowHeaderV2 {
    prev_root: Vec<CircuitByteBitsV2>,
    post_root: Vec<CircuitByteBitsV2>,
    spent_count: Vec<CircuitByteBitsV2>,
    output_count: Vec<CircuitByteBitsV2>,
}

fn take_circuit_lower_hex32(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    payload: &[CircuitByteBitsV2],
    bytes: &[u8],
    cursor: &mut usize,
    zero: ExprId,
    one: ExprId,
) -> Result<Vec<CircuitByteBitsV2>, CheckpointError> {
    let len_end = cursor.checked_add(2).ok_or(CheckpointError::Overflow)?;
    let len = usize::from(u16::from_le_bytes(
        bytes
            .get(*cursor..len_end)
            .ok_or(CheckpointError::Canonical)?
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    ));
    if len != 64 {
        return Err(CheckpointError::Canonical);
    }
    connect_bytes_to_constants(
        builder,
        &payload[*cursor..len_end],
        &64_u16.to_le_bytes(),
        zero,
        one,
    )?;
    *cursor = len_end;
    let end = cursor.checked_add(len).ok_or(CheckpointError::Overflow)?;
    let decoded = circuit_decode_lower_hex32(builder, &payload[*cursor..end], zero, one)?;
    *cursor = end;
    Ok(decoded)
}

fn constrain_flow_header_codec(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    view: &CircuitEventViewV2,
    zero: ExprId,
    one: ExprId,
) -> Result<CircuitFlowHeaderV2, CheckpointError> {
    let payload = view.payload_bits()?;
    let bytes = view.event.payload();
    if payload.len() != bytes.len() {
        return Err(CheckpointError::Invariant);
    }
    let mut cursor = 0_usize;
    let _batch_id = take_circuit_lower_hex32(builder, payload, bytes, &mut cursor, zero, one)?;
    cursor = cursor.checked_add(4 + 8).ok_or(CheckpointError::Overflow)?;
    let _route_table_digest =
        take_circuit_lower_hex32(builder, payload, bytes, &mut cursor, zero, one)?;
    let prev_root = take_circuit_lower_hex32(builder, payload, bytes, &mut cursor, zero, one)?;
    let post_root = take_circuit_lower_hex32(builder, payload, bytes, &mut cursor, zero, one)?;
    let spent_end = cursor.checked_add(4).ok_or(CheckpointError::Overflow)?;
    let spent_count = payload
        .get(cursor..spent_end)
        .ok_or(CheckpointError::Canonical)?
        .to_vec();
    cursor = spent_end;
    let output_end = cursor.checked_add(4).ok_or(CheckpointError::Overflow)?;
    let output_count = payload
        .get(cursor..output_end)
        .ok_or(CheckpointError::Canonical)?
        .to_vec();
    if output_end != payload.len() {
        return Err(CheckpointError::Canonical);
    }
    Ok(CircuitFlowHeaderV2 {
        prev_root,
        post_root,
        spent_count,
        output_count,
    })
}

/// Inject the low 124 digest bits into the degree-five KoalaBear trace field.
///
/// The five basis coefficients carry 25/25/25/25/24 bits respectively.  Every
/// coefficient is far below the KoalaBear modulus, so this is an injective
/// bit-to-field map with no modular alias.  Adding two domain-separates the
/// result from the zero/one product sentinels without approaching the modulus.
fn circuit_uniqueness_challenge_124(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    digest: &[CircuitByteBitsV2],
    zero: ExprId,
) -> Result<ExprId, CheckpointError> {
    if digest.len() != 32 {
        return Err(CheckpointError::Invariant);
    }
    let digest_bits = digest
        .iter()
        .flat_map(|byte| byte.iter().copied())
        .take(124)
        .collect::<Vec<_>>();
    if digest_bits.len() != 124 {
        return Err(CheckpointError::Invariant);
    }
    let mut challenge = builder.alloc_const(
        Plonky3TraceFieldV2::from_u64(2),
        "uniqueness_challenge_domain",
    );
    let mut offset = 0_usize;
    for (coefficient, width) in [25_usize, 25, 25, 25, 24].into_iter().enumerate() {
        let end = offset.checked_add(width).ok_or(CheckpointError::Overflow)?;
        let coefficient_value = circuit_little_endian_bits_value(
            builder,
            digest_bits[offset..end].iter().copied(),
            zero,
        )?;
        let basis = Plonky3TraceFieldV2::from_basis_coefficients_fn(|index| {
            if index == coefficient {
                KoalaBear::ONE
            } else {
                KoalaBear::ZERO
            }
        });
        let basis = builder.alloc_const(basis, "uniqueness_challenge_basis");
        challenge = builder.mul_add(coefficient_value, basis, challenge);
        offset = end;
    }
    if offset != 124 {
        return Err(CheckpointError::Invariant);
    }
    Ok(challenge)
}

fn circuit_uniqueness_row_polynomial(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    beta: ExprId,
    row: &[CircuitByteBitsV2],
    zero: ExprId,
) -> Result<ExprId, CheckpointError> {
    if row.len() != UNIQUENESS_SEMANTIC_ROW_BYTES_V2 || !row.len().is_multiple_of(2) {
        return Err(CheckpointError::Invariant);
    }
    let limbs = row
        .chunks_exact(2)
        .map(|bytes| circuit_little_endian_u16(builder, bytes, zero))
        .collect::<Result<Vec<_>, _>>()?;
    let mut polynomial = *limbs.last().ok_or(CheckpointError::Invariant)?;
    for limb in limbs[..limbs.len() - 1].iter().rev().copied() {
        polynomial = builder.mul_add(polynomial, beta, limb);
    }
    Ok(polynomial)
}

fn circuit_uniqueness_grand_product(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    alpha: ExprId,
    beta: ExprId,
    rows: &[&CircuitEventViewV2],
    zero: ExprId,
    one: ExprId,
) -> Result<ExprId, CheckpointError> {
    let mut product = one;
    for row in rows {
        let encoded =
            circuit_uniqueness_row_polynomial(builder, beta, uniqueness_row_bits(row)?, zero)?;
        let factor = builder.sub(alpha, encoded);
        product = builder.mul(product, factor);
    }
    Ok(product)
}

fn constrain_uniqueness_grand_products(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    challenge: &CircuitEventViewV2,
    original_spent: &[&CircuitEventViewV2],
    sorted_spent: &[&CircuitEventViewV2],
    original_output: &[&CircuitEventViewV2],
    sorted_output: &[&CircuitEventViewV2],
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    let payload = challenge.payload_bits()?;
    let challenge_bytes = payload
        .get(129..129 + 32 * 8)
        .ok_or(CheckpointError::Invariant)?;
    for pair in 0..2 {
        for (set_offset, original, sorted) in [
            (0_usize, original_spent, sorted_spent),
            (4_usize, original_output, sorted_output),
        ] {
            let alpha_index = set_offset
                .checked_add(pair * 2)
                .ok_or(CheckpointError::Overflow)?;
            let beta_index = alpha_index
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
            let alpha = circuit_uniqueness_challenge_124(
                builder,
                &challenge_bytes[alpha_index * 32..(alpha_index + 1) * 32],
                zero,
            )?;
            let beta = circuit_uniqueness_challenge_124(
                builder,
                &challenge_bytes[beta_index * 32..(beta_index + 1) * 32],
                zero,
            )?;
            let original_product =
                circuit_uniqueness_grand_product(builder, alpha, beta, original, zero, one)?;
            let sorted_product =
                circuit_uniqueness_grand_product(builder, alpha, beta, sorted, zero, one)?;
            builder.connect(original_product, sorted_product);
        }
    }
    Ok(())
}

fn constrain_strict_lexicographic_less(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    previous: &[CircuitByteBitsV2],
    candidate: &[CircuitByteBitsV2],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<(), CheckpointError> {
    if previous.len() != candidate.len() || previous.is_empty() {
        return Err(CheckpointError::Invariant);
    }
    let mut equal_prefix = one;
    let mut less = zero;
    for (previous_bit, candidate_bit) in
        previous
            .iter()
            .zip(candidate)
            .flat_map(|(previous_byte, candidate_byte)| {
                previous_byte
                    .iter()
                    .rev()
                    .copied()
                    .zip(candidate_byte.iter().rev().copied())
            })
    {
        let not_previous = builder.sub(one, previous_bit);
        let less_here = builder.mul(not_previous, candidate_bit);
        let first_less_here = builder.mul(equal_prefix, less_here);
        less = builder.add(less, first_less_here);
        let different = circuit_xor_bit(builder, previous_bit, candidate_bit, two);
        let same = builder.sub(one, different);
        equal_prefix = builder.mul(equal_prefix, same);
    }
    builder.connect(less, one);
    Ok(())
}

fn circuit_constant_digest(value: [u8; 32], zero: ExprId, one: ExprId) -> Vec<CircuitByteBitsV2> {
    value
        .into_iter()
        .map(|byte| constant_byte_bits(byte, zero, one))
        .collect()
}

fn constrain_raw_sha_padding(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    blocks: &[CircuitByteBitsV2],
    message_bytes: usize,
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    if blocks.is_empty()
        || !blocks.len().is_multiple_of(64)
        || message_bytes > blocks.len().saturating_sub(9)
    {
        return Err(CheckpointError::Invariant);
    }
    let bit_length = u64::try_from(message_bytes)
        .map_err(|_| CheckpointError::Limit)?
        .checked_mul(8)
        .ok_or(CheckpointError::Overflow)?;
    for (index, byte) in blocks.iter().enumerate().skip(message_bytes) {
        let expected = if index == message_bytes {
            0x80
        } else if index >= blocks.len() - 8 {
            bit_length.to_be_bytes()[index - (blocks.len() - 8)]
        } else {
            0
        };
        connect_byte_bits(builder, byte, &constant_byte_bits(expected, zero, one));
    }
    Ok(())
}

struct CircuitJmtLeafV2 {
    key_hash: Vec<CircuitByteBitsV2>,
    value_hash: Vec<CircuitByteBitsV2>,
    digest: Vec<CircuitByteBitsV2>,
}

fn constrain_jmt_leaf_raw_blocks(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    blocks: &[CircuitByteBitsV2],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<CircuitJmtLeafV2, CheckpointError> {
    if blocks.len() != 128 {
        return Err(CheckpointError::Invariant);
    }
    connect_bytes_to_constants(builder, &blocks[..13], b"JMT::LeafNode", zero, one)?;
    constrain_raw_sha_padding(builder, blocks, 77, zero, one)?;
    let key_hash = blocks[13..45].to_vec();
    let value_hash = blocks[45..64]
        .iter()
        .chain(&blocks[64..77])
        .copied()
        .collect::<Vec<_>>();
    let digest = circuit_sha256_padded_message_digest(builder, blocks, zero, one, two)?;
    Ok(CircuitJmtLeafV2 {
        key_hash,
        value_hash,
        digest,
    })
}

struct CircuitJmtInternalV2 {
    left_child: Vec<CircuitByteBitsV2>,
    right_child: Vec<CircuitByteBitsV2>,
    digest: Vec<CircuitByteBitsV2>,
}

fn constrain_jmt_internal_raw_blocks(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    blocks: &[CircuitByteBitsV2],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<CircuitJmtInternalV2, CheckpointError> {
    if blocks.len() != 128 {
        return Err(CheckpointError::Invariant);
    }
    connect_bytes_to_constants(builder, &blocks[..16], b"JMT::IntrnalNode", zero, one)?;
    constrain_raw_sha_padding(builder, blocks, 80, zero, one)?;
    let left_child = blocks[16..48].to_vec();
    let right_child = blocks[48..64]
        .iter()
        .chain(&blocks[64..80])
        .copied()
        .collect::<Vec<_>>();
    let digest = circuit_sha256_padded_message_digest(builder, blocks, zero, one, two)?;
    Ok(CircuitJmtInternalV2 {
        left_child,
        right_child,
        digest,
    })
}

fn constrain_jmt_record_tag(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    record: &CircuitEventViewV2,
    opcode: u8,
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    let payload = record.payload_bits()?;
    if payload.len() < 2 {
        return Err(CheckpointError::Invariant);
    }
    connect_bytes_to_constants(builder, &payload[..2], &[3, opcode], zero, one)
}

fn constrain_jmt_direction(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    direction: &CircuitByteBitsV2,
    key: &[CircuitByteBitsV2],
    bit_index: usize,
    zero: ExprId,
) -> Result<(), CheckpointError> {
    let direction_value =
        (key.get(bit_index / 8).ok_or(CheckpointError::Invariant)?)[7 - bit_index % 8];
    builder.connect(direction[0], direction_value);
    for bit in direction.iter().skip(1) {
        builder.connect(*bit, zero);
    }
    Ok(())
}

fn constrain_jmt_parent_children(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    parent: &CircuitJmtInternalV2,
    current: &[CircuitByteBitsV2],
    sibling: &[CircuitByteBitsV2],
    direction_is_right: bool,
) -> Result<(), CheckpointError> {
    let (expected_left, expected_right) = if direction_is_right {
        (sibling, current)
    } else {
        (current, sibling)
    };
    connect_bit_slices(builder, &parent.left_child, expected_left)?;
    connect_bit_slices(builder, &parent.right_child, expected_right)
}

fn constrain_jmt_sibling_hash(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    sibling_type: u8,
    blocks: &[CircuitByteBitsV2],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<Vec<CircuitByteBitsV2>, CheckpointError> {
    match sibling_type {
        0 => {
            connect_bytes_to_constants(builder, blocks, &[0; 128], zero, one)?;
            Ok(circuit_constant_digest(
                *b"SPARSE_MERKLE_PLACEHOLDER_HASH__",
                zero,
                one,
            ))
        }
        1 => Ok(constrain_jmt_internal_raw_blocks(builder, blocks, zero, one, two)?.digest),
        2 => Ok(constrain_jmt_leaf_raw_blocks(builder, blocks, zero, one, two)?.digest),
        _ => Err(CheckpointError::Invariant),
    }
}

struct CircuitJmtOperationV2 {
    key: Vec<CircuitByteBitsV2>,
    value_present: bool,
    prior_value_present: bool,
    expected_value_bytes: usize,
    expected_prior_value_bytes: usize,
    value_blocks: Vec<CircuitByteBitsV2>,
    prior_value_blocks: Vec<CircuitByteBitsV2>,
    expected_siblings: usize,
    expected_split_siblings: usize,
    consumed_siblings: usize,
    consumed_split_siblings: usize,
    path_key: Vec<CircuitByteBitsV2>,
    old_current: Vec<CircuitByteBitsV2>,
    new_current: Vec<CircuitByteBitsV2>,
    mutation_case: u8,
    new_parent_started: bool,
    coalesced_leaf_seen: bool,
    proof_seen: bool,
}

struct CircuitJmtUpdateV2 {
    new_root: Vec<CircuitByteBitsV2>,
    current_root: Vec<CircuitByteBitsV2>,
    operation: Option<CircuitJmtOperationV2>,
}

fn constrain_jmt_micro_operations(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    records: &[&CircuitEventViewV2],
    pre_definition_root: &[CircuitByteBitsV2],
    post_definition_root: &[CircuitByteBitsV2],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<(), CheckpointError> {
    let placeholder = circuit_constant_digest(*b"SPARSE_MERKLE_PLACEHOLDER_HASH__", zero, one);
    let mut update: Option<CircuitJmtUpdateV2> = None;
    for record in records {
        let bytes = record.event.payload();
        let payload = record.payload_bits()?;
        if bytes.len() != payload.len() || bytes.len() < 2 {
            return Err(CheckpointError::Invariant);
        }
        let record_opcode = bytes[1];
        constrain_jmt_record_tag(builder, record, record_opcode, zero, one)?;
        match record_opcode {
            1 => {
                if update.is_some() || payload.len() != 159 {
                    return Err(CheckpointError::Invariant);
                }
                let role_tag = bytes[6];
                if !(1..=5).contains(&role_tag) {
                    return Err(CheckpointError::Invariant);
                }
                connect_byte_bits(
                    builder,
                    &payload[6],
                    &constant_byte_bits(role_tag, zero, one),
                );
                match role_tag {
                    1 | 5 => {
                        connect_bytes_to_constants(builder, &payload[7..75], &[0; 68], zero, one)?;
                    }
                    2 => {
                        connect_bytes_to_constants(builder, &payload[39..75], &[0; 36], zero, one)?;
                    }
                    3 => {
                        connect_bytes_to_constants(builder, &payload[43..75], &[0; 32], zero, one)?;
                    }
                    4 => {}
                    _ => return Err(CheckpointError::Invariant),
                }
                let old_root = payload[91..123].to_vec();
                let new_root = payload[123..155].to_vec();
                if role_tag == 1 {
                    connect_bit_slices(builder, &old_root, pre_definition_root)?;
                    connect_bit_slices(builder, &new_root, post_definition_root)?;
                }
                update = Some(CircuitJmtUpdateV2 {
                    new_root,
                    current_root: old_root,
                    operation: None,
                });
            }
            2 => {
                let update = update.as_mut().ok_or(CheckpointError::Invariant)?;
                if update.operation.is_some() || payload.len() != 52 {
                    return Err(CheckpointError::Invariant);
                }
                let value_present = bytes[42] == 1;
                let expected_value_bytes = usize::try_from(u32::from_le_bytes(
                    bytes[43..47]
                        .try_into()
                        .map_err(|_| CheckpointError::Canonical)?,
                ))
                .map_err(|_| CheckpointError::Limit)?;
                let prior_value_present = bytes[47] == 1;
                let expected_prior_value_bytes = usize::try_from(u32::from_le_bytes(
                    bytes[48..52]
                        .try_into()
                        .map_err(|_| CheckpointError::Canonical)?,
                ))
                .map_err(|_| CheckpointError::Limit)?;
                connect_bytes_to_constants(builder, &payload[42..52], &bytes[42..52], zero, one)?;
                update.operation = Some(CircuitJmtOperationV2 {
                    key: payload[10..42].to_vec(),
                    value_present,
                    prior_value_present,
                    expected_value_bytes,
                    expected_prior_value_bytes,
                    value_blocks: Vec::new(),
                    prior_value_blocks: Vec::new(),
                    expected_siblings: 0,
                    expected_split_siblings: 0,
                    consumed_siblings: 0,
                    consumed_split_siblings: 0,
                    path_key: Vec::new(),
                    old_current: Vec::new(),
                    new_current: Vec::new(),
                    mutation_case: 0,
                    new_parent_started: false,
                    coalesced_leaf_seen: false,
                    proof_seen: false,
                });
            }
            3 => {
                let operation = update
                    .as_mut()
                    .and_then(|update| update.operation.as_mut())
                    .ok_or(CheckpointError::Invariant)?;
                if payload.len() != 83 {
                    return Err(CheckpointError::Invariant);
                }
                connect_bytes_to_constants(builder, &payload[10..19], &bytes[10..19], zero, one)?;
                match bytes[18] {
                    0 => operation.value_blocks.extend_from_slice(&payload[19..83]),
                    1 => operation
                        .prior_value_blocks
                        .extend_from_slice(&payload[19..83]),
                    _ => return Err(CheckpointError::Invariant),
                }
            }
            4 => {
                let operation = update
                    .as_mut()
                    .and_then(|update| update.operation.as_mut())
                    .ok_or(CheckpointError::Invariant)?;
                if operation.proof_seen || payload.len() != 275 {
                    return Err(CheckpointError::Invariant);
                }
                let leaf_present = bytes[10] == 1;
                operation.expected_siblings = usize::from(u16::from_le_bytes(
                    bytes[11..13]
                        .try_into()
                        .map_err(|_| CheckpointError::Canonical)?,
                ));
                operation.mutation_case = bytes[13];
                operation.expected_split_siblings = usize::from(u16::from_le_bytes(
                    bytes[14..16]
                        .try_into()
                        .map_err(|_| CheckpointError::Canonical)?,
                ));
                connect_bytes_to_constants(builder, &payload[10..19], &bytes[10..19], zero, one)?;
                let old_leaf = if leaf_present {
                    Some(constrain_jmt_leaf_raw_blocks(
                        builder,
                        &payload[19..147],
                        zero,
                        one,
                        two,
                    )?)
                } else {
                    connect_bytes_to_constants(builder, &payload[19..147], &[0; 128], zero, one)?;
                    None
                };
                if operation.prior_value_present {
                    constrain_raw_sha_padding(
                        builder,
                        &operation.prior_value_blocks,
                        operation.expected_prior_value_bytes,
                        zero,
                        one,
                    )?;
                    let prior_digest = circuit_sha256_padded_message_digest(
                        builder,
                        &operation.prior_value_blocks,
                        zero,
                        one,
                        two,
                    )?;
                    let old_leaf = old_leaf.as_ref().ok_or(CheckpointError::Invariant)?;
                    connect_bit_slices(builder, &old_leaf.key_hash, &operation.key)?;
                    connect_bit_slices(builder, &old_leaf.value_hash, &prior_digest)?;
                }
                operation.path_key = old_leaf
                    .as_ref()
                    .map_or_else(|| operation.key.clone(), |leaf| leaf.key_hash.clone());
                operation.old_current =
                    old_leaf.map_or_else(|| placeholder.clone(), |leaf| leaf.digest);
                operation.new_current = if operation.value_present {
                    constrain_raw_sha_padding(
                        builder,
                        &operation.value_blocks,
                        operation.expected_value_bytes,
                        zero,
                        one,
                    )?;
                    let value_digest = circuit_sha256_padded_message_digest(
                        builder,
                        &operation.value_blocks,
                        zero,
                        one,
                        two,
                    )?;
                    let new_leaf =
                        constrain_jmt_leaf_raw_blocks(builder, &payload[147..275], zero, one, two)?;
                    connect_bit_slices(builder, &new_leaf.key_hash, &operation.key)?;
                    connect_bit_slices(builder, &new_leaf.value_hash, &value_digest)?;
                    new_leaf.digest
                } else {
                    connect_bytes_to_constants(builder, &payload[147..275], &[0; 128], zero, one)?;
                    placeholder.clone()
                };
                operation.new_parent_started = matches!(operation.mutation_case, 1..=3);
                operation.proof_seen = true;
            }
            9 => {
                let operation = update
                    .as_mut()
                    .and_then(|update| update.operation.as_mut())
                    .ok_or(CheckpointError::Invariant)?;
                if !operation.proof_seen || payload.len() != 275 {
                    return Err(CheckpointError::Invariant);
                }
                let sibling_type = bytes[12];
                let direction_is_right = bytes[13] == 1;
                connect_bytes_to_constants(builder, &payload[10..19], &bytes[10..19], zero, one)?;
                let total_siblings = operation
                    .expected_split_siblings
                    .checked_add(operation.expected_siblings)
                    .ok_or(CheckpointError::Overflow)?;
                let bit_index = total_siblings
                    .checked_sub(operation.consumed_split_siblings + 1)
                    .ok_or(CheckpointError::Invariant)?;
                constrain_jmt_direction(builder, &payload[13], &operation.key, bit_index, zero)?;
                let sibling = constrain_jmt_sibling_hash(
                    builder,
                    sibling_type,
                    &payload[19..147],
                    zero,
                    one,
                    two,
                )?;
                let parent =
                    constrain_jmt_internal_raw_blocks(builder, &payload[147..275], zero, one, two)?;
                constrain_jmt_parent_children(
                    builder,
                    &parent,
                    &operation.new_current,
                    &sibling,
                    direction_is_right,
                )?;
                operation.new_current = parent.digest;
                operation.consumed_split_siblings = operation
                    .consumed_split_siblings
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
            }
            7 => {
                let operation = update
                    .as_mut()
                    .and_then(|update| update.operation.as_mut())
                    .ok_or(CheckpointError::Invariant)?;
                if !operation.proof_seen || payload.len() != 403 {
                    return Err(CheckpointError::Invariant);
                }
                let sibling_type = bytes[12];
                let direction_is_right = bytes[13] == 1;
                let new_parent_active = bytes[14] == 1;
                connect_bytes_to_constants(builder, &payload[10..19], &bytes[10..19], zero, one)?;
                let bit_index = operation
                    .expected_siblings
                    .checked_sub(operation.consumed_siblings + 1)
                    .ok_or(CheckpointError::Invariant)?;
                constrain_jmt_direction(
                    builder,
                    &payload[13],
                    &operation.path_key,
                    bit_index,
                    zero,
                )?;
                let sibling = constrain_jmt_sibling_hash(
                    builder,
                    sibling_type,
                    &payload[19..147],
                    zero,
                    one,
                    two,
                )?;
                let old_parent =
                    constrain_jmt_internal_raw_blocks(builder, &payload[147..275], zero, one, two)?;
                constrain_jmt_parent_children(
                    builder,
                    &old_parent,
                    &operation.old_current,
                    &sibling,
                    direction_is_right,
                )?;
                operation.old_current = old_parent.digest;
                if new_parent_active {
                    let new_parent = constrain_jmt_internal_raw_blocks(
                        builder,
                        &payload[275..403],
                        zero,
                        one,
                        two,
                    )?;
                    constrain_jmt_parent_children(
                        builder,
                        &new_parent,
                        &operation.new_current,
                        &sibling,
                        direction_is_right,
                    )?;
                    operation.new_current = new_parent.digest;
                    operation.new_parent_started = true;
                } else {
                    connect_bytes_to_constants(builder, &payload[275..403], &[0; 128], zero, one)?;
                    if operation.mutation_case == 6
                        && !operation.new_parent_started
                        && !operation.coalesced_leaf_seen
                        && sibling_type == 2
                    {
                        operation.new_current = sibling;
                        operation.coalesced_leaf_seen = true;
                    }
                }
                operation.consumed_siblings = operation
                    .consumed_siblings
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
            }
            8 => {
                let update = update.as_mut().ok_or(CheckpointError::Invariant)?;
                let operation = update
                    .operation
                    .as_ref()
                    .ok_or(CheckpointError::Invariant)?;
                if operation.consumed_siblings != operation.expected_siblings
                    || operation.consumed_split_siblings != operation.expected_split_siblings
                {
                    return Err(CheckpointError::Invariant);
                }
                connect_bit_slices(builder, &operation.old_current, &update.current_root)?;
            }
            5 => {
                let update = update.as_mut().ok_or(CheckpointError::Invariant)?;
                let operation = update.operation.take().ok_or(CheckpointError::Invariant)?;
                connect_bit_slices(builder, &operation.old_current, &update.current_root)?;
                update.current_root = operation.new_current;
            }
            6 => {
                let completed = update.take().ok_or(CheckpointError::Invariant)?;
                connect_bit_slices(builder, &completed.current_root, &completed.new_root)?;
            }
            _ => return Err(CheckpointError::Invariant),
        }
    }
    if update.is_some() {
        return Err(CheckpointError::Invariant);
    }
    Ok(())
}

fn connect_bit_slices(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    left: &[CircuitByteBitsV2],
    right: &[CircuitByteBitsV2],
) -> Result<(), CheckpointError> {
    if left.len() != right.len() {
        return Err(CheckpointError::Invariant);
    }
    for (left, right) in left.iter().zip(right) {
        connect_byte_bits(builder, left, right);
    }
    Ok(())
}

fn constrain_statement_value(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    statement_bits: &[CircuitByteBitsV2],
    offset: usize,
    expected: &[u8],
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    let actual = statement_bits
        .get(offset..offset + expected.len())
        .ok_or(CheckpointError::Invariant)?;
    connect_bytes_to_constants(builder, actual, expected, zero, one)
}

fn constrain_statement_digest_value(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    statement_bits: &[CircuitByteBitsV2],
    index: usize,
    expected: [u8; 32],
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    let actual = statement_digest_bits(
        statement_bits,
        PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
        index,
    )?;
    connect_bytes_to_constants(builder, actual, &expected, zero, one)
}

fn constrain_full_trace_statement_counts(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    views: &[CircuitEventViewV2],
    sources: &[&CircuitEventViewV2],
    statement: &[u8],
    statement_bits: &[CircuitByteBitsV2],
    zero: ExprId,
    one: ExprId,
) -> Result<(), CheckpointError> {
    let event_count = u64::try_from(sources.len()).map_err(|_| CheckpointError::Limit)?;
    let byte_count = sources.iter().try_fold(0_u64, |total, source| {
        total
            .checked_add(
                u64::try_from(source.canonical_bits.len()).map_err(|_| CheckpointError::Limit)?,
            )
            .ok_or(CheckpointError::Overflow)
    })?;
    for (offset, expected) in [
        (
            PLONKY3_STATEMENT_DECLARED_EVENT_COUNT_OFFSET_V2,
            event_count,
        ),
        (
            PLONKY3_STATEMENT_DECLARED_EVENT_COUNT_OFFSET_V2 + 8,
            byte_count,
        ),
    ] {
        let encoded = expected.to_le_bytes();
        if statement.get(offset..offset + 8) != Some(encoded.as_slice()) {
            return Err(CheckpointError::Invariant);
        }
        constrain_statement_value(builder, statement_bits, offset, &encoded, zero, one)?;
    }
    let mut counts = [0_u64; RECURSIVE_TRACE_OPCODE_COUNT_V2];
    for view in views {
        let opcode = usize::from(view.event.opcode() as u8);
        let slot = counts
            .get_mut(opcode.checked_sub(1).ok_or(CheckpointError::Invariant)?)
            .ok_or(CheckpointError::Invariant)?;
        *slot = slot.checked_add(1).ok_or(CheckpointError::Overflow)?;
    }
    for (index, count) in counts.into_iter().enumerate() {
        let encoded = count.to_le_bytes();
        for base in [
            PLONKY3_STATEMENT_DECLARED_COUNTS_OFFSET_V2,
            PLONKY3_STATEMENT_DECLARED_COUNTS_OFFSET_V2 + RECURSIVE_TRACE_OPCODE_COUNT_V2 * 8,
        ] {
            let offset = base
                .checked_add(index.checked_mul(8).ok_or(CheckpointError::Overflow)?)
                .ok_or(CheckpointError::Overflow)?;
            if statement.get(offset..offset + 8) != Some(encoded.as_slice()) {
                return Err(CheckpointError::Invariant);
            }
            constrain_statement_value(builder, statement_bits, offset, &encoded, zero, one)?;
        }
    }
    Ok(())
}

fn constrain_frozen_transition_semantics(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    views: &[CircuitEventViewV2],
    statement: &[u8],
    statement_bits: &[CircuitByteBitsV2],
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<usize, CheckpointError> {
    let mut stage = "source schedule";
    let result = (|| -> Result<usize, CheckpointError> {
        let sources = views
            .iter()
            .filter(|view| view.event.opcode().is_source_record())
            .collect::<Vec<_>>();
        if sources.is_empty() {
            return Err(transition_semantics_error("source schedule"));
        }
        for (expected, source) in sources.iter().enumerate() {
            if source.event.ordinal()
                != u64::try_from(expected).map_err(|_| CheckpointError::Limit)?
            {
                return Err(transition_semantics_error("source ordinal"));
            }
        }
        stage = "statement counts";
        constrain_full_trace_statement_counts(
            builder,
            views,
            &sources,
            statement,
            statement_bits,
            zero,
            one,
        )?;

        stage = "begin header";
        let mut cursor = 0_usize;
        let begin = take_source_view(&sources, &mut cursor, RecursiveTraceOpcodeV2::BeginBlock)?;
        let begin_header = decode_flow_header(begin.event.payload())?;
        let begin_circuit = constrain_flow_header_codec(builder, begin, zero, one)?;
        stage = "uniqueness precommit";
        let precommit_view = take_source_view(
            &sources,
            &mut cursor,
            RecursiveTraceOpcodeV2::UniquenessPrecommit,
        )?;
        let precommit = decode_uniqueness_precommit(precommit_view.event.payload())?;
        if begin_header.spent_count != precommit.spent_count
            || begin_header.output_count != precommit.output_count
        {
            return Err(transition_semantics_error("precommit counts"));
        }
        let precommit_payload = precommit_view.payload_bits()?;
        connect_byte_bits(
            builder,
            &precommit_payload[0],
            &constant_byte_bits(1, zero, one),
        );
        connect_bit_slices(
            builder,
            &begin_circuit.spent_count,
            &precommit_payload[1..5],
        )?;
        connect_bit_slices(
            builder,
            &begin_circuit.output_count,
            &precommit_payload[5..9],
        )?;

        let spent_count =
            usize::try_from(precommit.spent_count).map_err(|_| CheckpointError::Limit)?;
        let output_count =
            usize::try_from(precommit.output_count).map_err(|_| CheckpointError::Limit)?;
        let mut replay_spent = Vec::with_capacity(spent_count);
        let mut replay_output = Vec::with_capacity(output_count);
        let mut commit_original_spent = Vec::with_capacity(spent_count);
        let mut commit_original_output = Vec::with_capacity(output_count);
        stage = "replay and original uniqueness commits";
        for (opcode, set, expected_count, replay_rows, committed_rows) in [
            (
                RecursiveTraceOpcodeV2::ReplayInput,
                UniquenessSetKindV2::Spent,
                spent_count,
                &mut replay_spent,
                &mut commit_original_spent,
            ),
            (
                RecursiveTraceOpcodeV2::ReplayOutput,
                UniquenessSetKindV2::Output,
                output_count,
                &mut replay_output,
                &mut commit_original_output,
            ),
        ] {
            for _ in 0..expected_count {
                let replay = take_source_view(&sources, &mut cursor, opcode)?;
                let item = decode_flow_item(replay.event.payload())?;
                if item.terminal_id != replay.event.object_id() {
                    return Err(transition_semantics_error("replay object identity"));
                }
                let row = UniquenessSemanticRowV2::from_canonical_flow_item(&item);
                let committed = take_source_view(
                    &sources,
                    &mut cursor,
                    RecursiveTraceOpcodeV2::UniquenessSorted,
                )?;
                let (pass, committed_set, list, committed_row) =
                    decode_uniqueness_sorted_row(committed.event.payload())?;
                if pass != UniquenessPassV2::Commit
                    || committed_set != set
                    || list != UniquenessListKindV2::Original
                    || committed_row != row
                {
                    return Err(transition_semantics_error("original uniqueness commit"));
                }
                constrain_uniqueness_row_header(
                    builder,
                    committed,
                    UniquenessPassV2::Commit,
                    set,
                    UniquenessListKindV2::Original,
                    zero,
                    one,
                )?;
                constrain_replay_to_uniqueness_row(
                    builder,
                    replay,
                    committed,
                    if opcode == RecursiveTraceOpcodeV2::ReplayInput {
                        2
                    } else {
                        1
                    },
                    zero,
                    one,
                )?;
                replay_rows.push((row, replay));
                committed_rows.push((row, committed));
            }
        }

        stage = "sorted uniqueness commits";
        let mut commit_sorted_spent = Vec::with_capacity(spent_count);
        let mut commit_sorted_output = Vec::with_capacity(output_count);
        for (set, expected_count, rows) in [
            (
                UniquenessSetKindV2::Spent,
                spent_count,
                &mut commit_sorted_spent,
            ),
            (
                UniquenessSetKindV2::Output,
                output_count,
                &mut commit_sorted_output,
            ),
        ] {
            let mut prior = None;
            for _ in 0..expected_count {
                let view = take_source_view(
                    &sources,
                    &mut cursor,
                    RecursiveTraceOpcodeV2::UniquenessSorted,
                )?;
                let (pass, row_set, list, row) =
                    decode_uniqueness_sorted_row(view.event.payload())?;
                if pass != UniquenessPassV2::Commit
                    || row_set != set
                    || list != UniquenessListKindV2::Sorted
                    || prior.is_some_and(|(prior, _)| prior >= row.terminal_id)
                {
                    return Err(CheckpointError::DuplicateIdentifier);
                }
                constrain_uniqueness_row_header(
                    builder,
                    view,
                    UniquenessPassV2::Commit,
                    set,
                    UniquenessListKindV2::Sorted,
                    zero,
                    one,
                )?;
                if let Some((_, prior_view)) = prior {
                    let prior_row = uniqueness_row_bits(prior_view)?;
                    let candidate_row = uniqueness_row_bits(view)?;
                    constrain_strict_lexicographic_less(
                        builder,
                        &prior_row[36..68],
                        &candidate_row[36..68],
                        zero,
                        one,
                        two,
                    )?;
                }
                prior = Some((row.terminal_id, view));
                rows.push((row, view));
            }
        }

        for (original, sorted) in [
            (&commit_original_spent, &commit_sorted_spent),
            (&commit_original_output, &commit_sorted_output),
        ] {
            let mut expected = original.iter().map(|(row, _)| *row).collect::<Vec<_>>();
            expected.sort_unstable_by_key(|row| row.terminal_id);
            if expected != sorted.iter().map(|(row, _)| *row).collect::<Vec<_>>() {
                return Err(transition_semantics_error("sorted uniqueness commit"));
            }
        }

        stage = "uniqueness challenge and grand products";
        let challenge = take_source_view(
            &sources,
            &mut cursor,
            RecursiveTraceOpcodeV2::UniquenessChallenge,
        )?;
        if challenge.event.payload().len() != UNIQUENESS_CHALLENGE_BYTES_V2 {
            return Err(transition_semantics_error("uniqueness challenge width"));
        }
        let original_spent_views = commit_original_spent
            .iter()
            .map(|(_, view)| *view)
            .collect::<Vec<_>>();
        let sorted_spent_views = commit_sorted_spent
            .iter()
            .map(|(_, view)| *view)
            .collect::<Vec<_>>();
        let original_output_views = commit_original_output
            .iter()
            .map(|(_, view)| *view)
            .collect::<Vec<_>>();
        let sorted_output_views = commit_sorted_output
            .iter()
            .map(|(_, view)| *view)
            .collect::<Vec<_>>();
        constrain_uniqueness_grand_products(
            builder,
            challenge,
            &original_spent_views,
            &sorted_spent_views,
            &original_output_views,
            &sorted_output_views,
            zero,
            one,
        )?;

        stage = "original uniqueness products";
        for (set, committed) in [
            (UniquenessSetKindV2::Spent, &commit_original_spent),
            (UniquenessSetKindV2::Output, &commit_original_output),
        ] {
            for (expected_row, committed_view) in committed {
                let product = take_source_view(
                    &sources,
                    &mut cursor,
                    RecursiveTraceOpcodeV2::UniquenessSorted,
                )?;
                let (pass, product_set, list, product_row) =
                    decode_uniqueness_sorted_row(product.event.payload())?;
                if pass != UniquenessPassV2::Product
                    || product_set != set
                    || list != UniquenessListKindV2::Original
                    || product_row != *expected_row
                {
                    return Err(transition_semantics_error("original uniqueness product"));
                }
                constrain_uniqueness_row_header(
                    builder,
                    product,
                    UniquenessPassV2::Product,
                    set,
                    UniquenessListKindV2::Original,
                    zero,
                    one,
                )?;
                connect_bit_slices(
                    builder,
                    uniqueness_row_bits(committed_view)?,
                    uniqueness_row_bits(product)?,
                )?;
            }
        }

        stage = "global uniqueness products and net effects";
        let mut expected_global = commit_sorted_spent
            .iter()
            .map(|(row, view)| (*row, UniquenessSetKindV2::Spent, *view))
            .chain(
                commit_sorted_output
                    .iter()
                    .map(|(row, view)| (*row, UniquenessSetKindV2::Output, *view)),
            )
            .collect::<Vec<_>>();
        expected_global.sort_unstable_by_key(|(row, set, _)| (row.terminal_id, *set as u8));
        let mut global_index = 0_usize;
        while global_index < expected_global.len() {
            let (first_row, first_set, first_commit_view) = expected_global[global_index];
            let first = take_source_view(
                &sources,
                &mut cursor,
                RecursiveTraceOpcodeV2::UniquenessSorted,
            )?;
            let (pass, set, list, row) = decode_uniqueness_sorted_row(first.event.payload())?;
            if pass != UniquenessPassV2::Product
                || set != first_set
                || list != UniquenessListKindV2::Sorted
                || row != first_row
            {
                return Err(transition_semantics_error("global uniqueness product"));
            }
            constrain_uniqueness_row_header(
                builder,
                first,
                UniquenessPassV2::Product,
                first_set,
                UniquenessListKindV2::Sorted,
                zero,
                one,
            )?;
            connect_bit_slices(
                builder,
                uniqueness_row_bits(first_commit_view)?,
                uniqueness_row_bits(first)?,
            )?;
            let mut spent = (first_set == UniquenessSetKindV2::Spent).then_some(first_row);
            let mut output = (first_set == UniquenessSetKindV2::Output).then_some(first_row);
            let spent_view = (first_set == UniquenessSetKindV2::Spent).then_some(first);
            let mut output_view = (first_set == UniquenessSetKindV2::Output).then_some(first);
            global_index = global_index
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
            if let Some((next_row, next_set, next_commit_view)) =
                expected_global.get(global_index).copied()
            {
                if next_row.terminal_id == first_row.terminal_id {
                    if first_set != UniquenessSetKindV2::Spent
                        || next_set != UniquenessSetKindV2::Output
                        || !first_row.same_storage_path(next_row)
                    {
                        return Err(CheckpointError::DuplicateIdentifier);
                    }
                    let next = take_source_view(
                        &sources,
                        &mut cursor,
                        RecursiveTraceOpcodeV2::UniquenessSorted,
                    )?;
                    let (pass, set, list, row) =
                        decode_uniqueness_sorted_row(next.event.payload())?;
                    if pass != UniquenessPassV2::Product
                        || set != next_set
                        || list != UniquenessListKindV2::Sorted
                        || row != next_row
                    {
                        return Err(transition_semantics_error(
                            "paired global uniqueness product",
                        ));
                    }
                    constrain_uniqueness_row_header(
                        builder,
                        next,
                        UniquenessPassV2::Product,
                        next_set,
                        UniquenessListKindV2::Sorted,
                        zero,
                        one,
                    )?;
                    connect_bit_slices(
                        builder,
                        uniqueness_row_bits(next_commit_view)?,
                        uniqueness_row_bits(next)?,
                    )?;
                    output = Some(next_row);
                    output_view = Some(next);
                    global_index = global_index
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
            }
            let expected_effect = NetEffectV2::from_rows(spent.take(), output.take())?;
            let net = take_source_view(&sources, &mut cursor, RecursiveTraceOpcodeV2::NetMerge)?;
            if decode_net_effect(net.event.payload())? != expected_effect
                || net.event.payload() != encode_net_effect(expected_effect)
            {
                return Err(transition_semantics_error("net effect"));
            }
            constrain_net_effect_from_rows(builder, net, spent_view, output_view, zero, one)?;
        }

        stage = "net close";
        let close = take_source_view(&sources, &mut cursor, RecursiveTraceOpcodeV2::NetMerge)?;
        let close_payload = close.payload_bits()?;
        if close_payload.len() != NET_MERGE_BYTES_V2
            || decode_net_effect(close.event.payload())?.kind as u8 != 0
        {
            return Err(transition_semantics_error("net close"));
        }
        connect_bytes_to_constants(builder, &close_payload[..2], &[1, 0], zero, one)?;
        connect_bit_slices(
            builder,
            &precommit_view.payload_bits()?[UNIQUENESS_PRECOMMIT_BYTES_V2 - 32..],
            &close_payload[2..34],
        )?;
        connect_bit_slices(
            builder,
            &challenge.payload_bits()?[33..65],
            &close_payload[38..70],
        )?;
        connect_bit_slices(
            builder,
            &challenge.payload_bits()?[65..97],
            &close_payload[70..102],
        )?;
        connect_bit_slices(
            builder,
            &challenge.payload_bits()?[97..129],
            &close_payload[102..134],
        )?;

        stage = "JMT schedule decode";
        let jmt_header =
            take_source_view(&sources, &mut cursor, RecursiveTraceOpcodeV2::JmtUpdate)?;
        connect_bytes_to_constants(
            builder,
            &jmt_header.payload_bits()?[..3],
            &jmt_header.event.payload()[..3],
            zero,
            one,
        )?;
        let mut jmt_decoder =
            SettlementUpdateTraceCircuitDecoderV2::new(jmt_header.event.payload())
                .map_err(|_| CheckpointError::Canonical)?;
        let mut jmt_micro_operations = Vec::new();
        while sources
            .get(cursor)
            .is_some_and(|view| view.event.opcode() == RecursiveTraceOpcodeV2::JmtMicroOp)
        {
            let micro =
                take_source_view(&sources, &mut cursor, RecursiveTraceOpcodeV2::JmtMicroOp)?;
            jmt_decoder
                .accept(micro.event.payload())
                .map_err(|_| CheckpointError::Canonical)?;
            jmt_micro_operations.push(micro);
        }
        stage = "JMT circuit constraints";
        let pre_definition_root = statement_digest_bits(
            statement_bits,
            PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
            PLONKY3_STATEMENT_PRE_DEFINITION_INDEX_V2,
        )?;
        let post_definition_root = statement_digest_bits(
            statement_bits,
            PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
            PLONKY3_STATEMENT_POST_DEFINITION_INDEX_V2,
        )?;
        if jmt_micro_operations.is_empty() {
            connect_bit_slices(builder, pre_definition_root, post_definition_root)?;
        }
        constrain_jmt_micro_operations(
            builder,
            &jmt_micro_operations,
            pre_definition_root,
            post_definition_root,
            zero,
            one,
            two,
        )?;
        stage = "JMT decoder finish";
        let summary = jmt_decoder
            .finish()
            .map_err(|_| CheckpointError::Canonical)?;
        let pre_definition: [u8; 32] = statement[PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2
            + PLONKY3_STATEMENT_PRE_DEFINITION_INDEX_V2 * 32
            ..PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2
                + (PLONKY3_STATEMENT_PRE_DEFINITION_INDEX_V2 + 1) * 32]
            .try_into()
            .map_err(|_| CheckpointError::Invariant)?;
        let post_definition: [u8; 32] = statement[PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2
            + PLONKY3_STATEMENT_POST_DEFINITION_INDEX_V2 * 32
            ..PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2
                + (PLONKY3_STATEMENT_POST_DEFINITION_INDEX_V2 + 1) * 32]
            .try_into()
            .map_err(|_| CheckpointError::Invariant)?;
        stage = "JMT hierarchy root derivation";
        let (derived_pre, derived_post) = summary
            .verify_hierarchy_semantics(post_definition)
            .map_err(|_| CheckpointError::Canonical)?;
        if derived_pre != pre_definition || derived_post != post_definition {
            return Err(transition_semantics_error("JMT hierarchy roots"));
        }
        let jmt_digest: [u8; 32] = jmt_header.event.payload()[3..35]
            .try_into()
            .map_err(|_| CheckpointError::Invariant)?;
        constrain_statement_digest_value(
            builder,
            statement_bits,
            PLONKY3_STATEMENT_UPDATE_TRACE_DIGEST_INDEX_V2,
            jmt_digest,
            zero,
            one,
        )?;
        connect_bit_slices(
            builder,
            &jmt_header.payload_bits()?[3..35],
            statement_digest_bits(
                statement_bits,
                PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
                PLONKY3_STATEMENT_UPDATE_TRACE_DIGEST_INDEX_V2,
            )?,
        )?;

        stage = "JMT hierarchy promotion";
        let promotion = take_source_view(
            &sources,
            &mut cursor,
            RecursiveTraceOpcodeV2::PromoteChildRoot,
        )?;
        let (promoted_definition, promoted_trace) =
            decode_hierarchy_promotion_fields(promotion.event.payload())?;
        if promoted_definition != post_definition || promoted_trace != jmt_digest {
            return Err(transition_semantics_error("JMT hierarchy promotion"));
        }
        connect_byte_bits(
            builder,
            &promotion.payload_bits()?[0],
            &constant_byte_bits(1, zero, one),
        );
        connect_bit_slices(
            builder,
            &promotion.payload_bits()?[1..33],
            statement_digest_bits(
                statement_bits,
                PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
                PLONKY3_STATEMENT_POST_DEFINITION_INDEX_V2,
            )?,
        )?;
        connect_bit_slices(
            builder,
            &promotion.payload_bits()?[33..65],
            &jmt_header.payload_bits()?[3..35],
        )?;

        stage = "typed checkpoint commitments";
        for (kind, statement_index) in [
            (
                TypedCheckpointCommitmentKindV2::DeltaRoot,
                PLONKY3_STATEMENT_DELTA_ROOT_INDEX_V2,
            ),
            (
                TypedCheckpointCommitmentKindV2::WitnessRoot,
                PLONKY3_STATEMENT_WITNESS_ROOT_INDEX_V2,
            ),
            (
                TypedCheckpointCommitmentKindV2::JournalDigest,
                PLONKY3_STATEMENT_JOURNAL_DIGEST_INDEX_V2,
            ),
            (
                TypedCheckpointCommitmentKindV2::CheckpointLinkDigest,
                PLONKY3_STATEMENT_LINK_DIGEST_INDEX_V2,
            ),
        ] {
            let commitment = take_source_view(
                &sources,
                &mut cursor,
                RecursiveTraceOpcodeV2::CommitTypedEvent,
            )?;
            let (actual_kind, digest) =
                decode_typed_checkpoint_commitment(commitment.event.payload())?;
            if actual_kind != kind
                || commitment.event.payload().len() != TYPED_CHECKPOINT_COMMITMENT_BYTES_V2
            {
                return Err(transition_semantics_error("typed commitment"));
            }
            connect_bytes_to_constants(
                builder,
                &commitment.payload_bits()?[..2],
                &[0xc2, kind as u8],
                zero,
                one,
            )?;
            constrain_statement_digest_value(
                builder,
                statement_bits,
                statement_index,
                digest,
                zero,
                one,
            )?;
            connect_bit_slices(
                builder,
                &commitment.payload_bits()?[2..34],
                statement_digest_bits(
                    statement_bits,
                    PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
                    statement_index,
                )?,
            )?;
        }

        stage = "final source schedule";
        let finalize =
            take_source_view(&sources, &mut cursor, RecursiveTraceOpcodeV2::FinalizeBlock)?;
        if cursor != sources.len()
            || decode_flow_header(finalize.event.payload())? != begin_header
            || finalize.event.payload() != begin.event.payload()
        {
            return Err(transition_semantics_error("final source schedule"));
        }
        connect_bit_slices(builder, begin.payload_bits()?, finalize.payload_bits()?)?;
        for (index, root) in [
            (
                PLONKY3_STATEMENT_PRE_SETTLEMENT_INDEX_V2,
                &begin_circuit.prev_root,
            ),
            (
                PLONKY3_STATEMENT_POST_SETTLEMENT_INDEX_V2,
                &begin_circuit.post_root,
            ),
        ] {
            connect_bit_slices(
                builder,
                root,
                statement_digest_bits(
                    statement_bits,
                    PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2,
                    index,
                )?,
            )?;
        }
        Ok(sources.len())
    })();
    result.map_err(|error| transition_semantics_stage(stage, error))
}

fn prepare_circuit(
    words: &[u16],
    event_vector: Option<&[u8]>,
) -> Result<PreparedCircuitV2, CheckpointError> {
    if words.is_empty() || !words.len().is_multiple_of(8) {
        return Err(CheckpointError::Invariant);
    }
    let expected_hash = poseidon_vector_hash(words);
    let mut builder = CircuitBuilder::<Plonky3TraceFieldV2>::new();
    builder.enable_poseidon2_perm::<KoalaBearD4Width16, _>(
        generate_poseidon2_trace::<Plonky3TraceFieldV2, KoalaBearD4Width16>,
        default_koalabear_poseidon2_16(),
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
        let mut recomposed = zero;
        let mut bits = [zero; 16];
        for (bit_index, bit_slot) in bits.iter_mut().enumerate() {
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
            *bit_slot = bit;
        }
        builder.connect(word_expr, recomposed);
        word_exprs.push(word_expr);
        predicate_byte_bits.push(core::array::from_fn(|bit| bits[bit]));
        predicate_byte_bits.push(core::array::from_fn(|bit| bits[8 + bit]));
    }
    if let Some(event_vector) = event_vector {
        let event_views = circuit_event_views(event_vector, &predicate_byte_bits)
            .map_err(|error| air_construction_stage("event decode", error))?;
        let statement_bits = circuit_statement_bits(&predicate_byte_bits)
            .map_err(|error| air_construction_stage("statement view", error))?;
        let predicate_bytes = words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect::<Vec<_>>();
        let statement_start = PLONKY3_PREDICATE_VECTOR_LABEL_V2
            .len()
            .checked_add(8)
            .ok_or(CheckpointError::Overflow)?;
        let statement = predicate_bytes
            .get(statement_start..statement_start + PLONKY3_BASE_STATEMENT_BYTES_V2)
            .ok_or(CheckpointError::Invariant)?;
        let source_count = event_views
            .iter()
            .filter(|view| view.event.opcode().is_source_record())
            .count();
        let has_complete_transition = event_views
            .iter()
            .any(|view| view.event.opcode() == RecursiveTraceOpcodeV2::FinalizeBlock);
        let constrained_ids =
            constrain_structural_source_event_ids(&mut builder, &event_views, zero, one, two)
                .map_err(|error| air_construction_stage("structural event IDs", error))?;
        let constrained = constrain_sha_control_blocks(&mut builder, &event_views, zero, one, two)
            .map_err(|error| air_construction_stage("SHA control blocks", error))?;
        let constrained_sources =
            constrain_source_record_bindings(&mut builder, &event_views, zero, one)
                .map_err(|error| air_construction_stage("source bindings", error))?;
        let has_uniqueness_lists = event_views.iter().any(|view| {
            decode_hash_control(&view.event)
                .map(|control| control.schema == HashControlSchemaV2::UniquenessList)
                .unwrap_or(false)
        });
        let constrained_uniqueness_lists = if has_uniqueness_lists {
            constrain_uniqueness_list_bindings(&mut builder, &event_views, zero, one)
                .map_err(|error| air_construction_stage("uniqueness list bindings", error))?
        } else {
            0
        };
        let has_uniqueness_transcript = event_views.iter().any(|view| {
            decode_hash_control(&view.event)
                .map(|control| control.schema == HashControlSchemaV2::UniquenessTranscript)
                .unwrap_or(false)
        });
        let constrained_uniqueness_transcript = if has_uniqueness_transcript {
            constrain_uniqueness_transcript_bindings(
                &mut builder,
                &event_views,
                statement_bits,
                zero,
                one,
                two,
            )
            .map_err(|error| air_construction_stage("uniqueness transcript bindings", error))?
        } else {
            0
        };
        let has_trace_precommit = event_views.iter().any(|view| {
            decode_hash_control(&view.event)
                .map(|control| control.schema == HashControlSchemaV2::TracePrecommit)
                .unwrap_or(false)
        });
        let constrained_trace = if has_trace_precommit {
            constrain_trace_precommit_bindings(
                &mut builder,
                &event_views,
                statement_bits,
                has_complete_transition,
                zero,
                one,
            )
            .map_err(|error| air_construction_stage("trace precommit bindings", error))?
        } else {
            0
        };
        let constrained_transition = if has_complete_transition {
            constrain_frozen_transition_semantics(
                &mut builder,
                &event_views,
                statement,
                statement_bits,
                zero,
                one,
                two,
            )
            .map_err(|error| air_construction_stage("frozen transition semantics", error))?
        } else {
            0
        };
        if constrained_ids != source_count || constrained == 0 || constrained_sources == 0 {
            return Err(CheckpointError::Backend(format!(
                "Plonky3 AIR coverage mismatch: source_count={source_count}, structural_ids={constrained_ids}, sha_controls={constrained}, source_bindings={constrained_sources}"
            )));
        }
        if has_trace_precommit && constrained_trace == 0 {
            return Err(CheckpointError::Backend(
                "Plonky3 AIR trace-precommit coverage is empty".into(),
            ));
        }
        if has_uniqueness_lists
            && constrained_uniqueness_lists != UniquenessListHashJobV2::ALL.len()
        {
            return Err(CheckpointError::Backend(format!(
                "Plonky3 AIR uniqueness-list coverage mismatch: actual {constrained_uniqueness_lists}, expected {}",
                UniquenessListHashJobV2::ALL.len()
            )));
        }
        if has_uniqueness_transcript
            && constrained_uniqueness_transcript != UniquenessTranscriptHashJobV2::ALL.len()
        {
            return Err(CheckpointError::Backend(format!(
                "Plonky3 AIR uniqueness-transcript coverage mismatch: actual {constrained_uniqueness_transcript}, expected {}",
                UniquenessTranscriptHashJobV2::ALL.len()
            )));
        }
        if has_complete_transition && constrained_transition != source_count {
            return Err(CheckpointError::Backend(format!(
                "Plonky3 AIR frozen-transition coverage mismatch: actual {constrained_transition}, expected {source_count}"
            )));
        }
    }
    let mut final_outputs = None;
    let chunk_count = word_exprs.len() / 8;
    for (index, chunk) in word_exprs.chunks_exact(8).enumerate() {
        let mut inputs = vec![None; 4];
        for (slot, limbs) in inputs.iter_mut().take(2).zip(chunk.chunks_exact(4)) {
            let mut packed = zero;
            for (basis_index, limb) in limbs.iter().copied().enumerate() {
                let mut basis = [KoalaBear::ZERO; 4];
                basis[basis_index] = KoalaBear::ONE;
                let basis = builder.alloc_const(Plonky3TraceFieldV2::new(basis), "poseidon2_basis");
                let term = builder.mul(limb, basis);
                packed = builder.add(packed, term);
            }
            *slot = Some(packed);
        }
        let is_last = index + 1 == chunk_count;
        let (_, outputs) = builder
            .add_poseidon2_perm(&Poseidon2PermCall {
                config: Poseidon2Config::KOALA_BEAR_D4_W16,
                new_start: index == 0,
                merkle_path: false,
                mmcs_bit: None,
                mmcs_bit2: None,
                inputs,
                out_ctl: vec![is_last; 2],
                return_all_outputs: false,
                mmcs_index_sum: None,
            })
            .map_err(|_| {
                CheckpointError::Backend("Plonky3 AIR Poseidon transcript lowering failed".into())
            })?;
        if is_last {
            final_outputs = Some(outputs);
        }
    }
    let final_outputs = final_outputs.ok_or_else(|| {
        CheckpointError::Backend("Plonky3 AIR Poseidon transcript has no final output".into())
    })?;
    for (output, expected) in final_outputs
        .into_iter()
        .take(2)
        .zip(expected_hash.chunks_exact(4))
    {
        let output = output.ok_or_else(|| {
            CheckpointError::Backend("Plonky3 AIR Poseidon output lane is missing".into())
        })?;
        let expected = builder.alloc_const(
            Plonky3TraceFieldV2::new(
                expected
                    .try_into()
                    .map_err(|_| CheckpointError::Invariant)?,
            ),
            "poseidon2_transcript_output",
        );
        builder.connect(output, expected);
    }
    let circuit = builder
        .build()
        .map_err(|_| CheckpointError::Backend("Plonky3 circuit build failed".into()))?;
    let table_packing =
        TablePacking::new(PLONKY3_TABLE_PUBLIC_LANES_V2, PLONKY3_TABLE_ALU_LANES_V2)
            .with_min_trace_height(PLONKY3_TABLE_MIN_HEIGHT_V2);
    let config = hardened_koala_bear_config();
    let preprocessors: Vec<Box<dyn NpoPreprocessor<KoalaBear>>> =
        vec![Box::new(Poseidon2Preprocessor)];
    let air_builders = poseidon2_air_builders_d4::<Plonky3StarkConfigV2>();
    let (airs_degrees, primitive_columns, non_primitive_columns) =
        get_airs_and_degrees_with_prep::<Plonky3StarkConfigV2, _, 4>(
            &circuit,
            &table_packing,
            &preprocessors,
            &air_builders,
            ConstraintProfile::Standard,
        )
        .map_err(|_| CheckpointError::Backend("Plonky3 AIR lowering failed".into()))?;
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

fn hardened_koala_bear_config() -> Plonky3StarkConfigV2 {
    let leaf_hash = SerializingHasher::new(Sha512HasherV2::<1>);
    let compress = Plonky3CompressionV2::new(Sha512HasherV2::<2>);
    let value_mmcs = Plonky3ValueMmcsV2::new(leaf_hash, compress, 3);
    let challenge_mmcs = Plonky3ChallengeMmcsV2::new(value_mmcs.clone());
    let fri = FriParameters {
        log_blowup: usize::from(PLONKY3_FRI_LOG_BLOWUP_V2),
        log_final_poly_len: usize::from(PLONKY3_FRI_LOG_FINAL_POLY_LEN_V2),
        max_log_arity: usize::from(PLONKY3_FRI_MAX_LOG_ARITY_V2),
        num_queries: usize::from(PLONKY3_FRI_NUM_QUERIES_V2),
        commit_proof_of_work_bits: usize::from(PLONKY3_FRI_COMMIT_POW_BITS_V2),
        query_proof_of_work_bits: usize::from(PLONKY3_FRI_QUERY_POW_BITS_V2),
        mmcs: challenge_mmcs,
    };
    let pcs = Plonky3PcsV2::new(Radix2DitParallel::default(), value_mmcs, fri);
    let challenger = Plonky3ChallengerV2::new(HashChallenger::new(
        b"z00z.plonky3.fiat-shamir.v2".to_vec(),
        Sha512HasherV2::<3>,
    ));
    StarkConfig::new(pcs, challenger)
}

fn lift_koala(value: KoalaBear) -> Plonky3TraceFieldV2 {
    Plonky3TraceFieldV2::new([value, KoalaBear::ZERO, KoalaBear::ZERO, KoalaBear::ZERO])
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
    common: &p3_batch_stark::CommonData<Plonky3StarkConfigV2>,
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
    use crate::checkpoint::recursive_semantics::{
        encode_uniqueness_challenge, encode_uniqueness_precommit_value,
        uniqueness_precommit_from_rows,
    };
    use crate::checkpoint::recursive_statement::{
        RecursiveDeclaredWorkV2, RecursivePreUniquenessContextV2,
    };
    use crate::checkpoint::recursive_trace::{
        emit_derived_hash_controls, emit_expanded_trace_hash_controls_for_test,
        emit_expanded_uniqueness_list_hash_controls_for_test,
        emit_expanded_uniqueness_transcript_hash_controls_for_test, structural_event_id,
        RecursiveTraceEventCountsV2,
    };

    fn encode_event_vector(events: &[RecursiveTraceEventV2]) -> Vec<u8> {
        let mut vector = Vec::new();
        vector.extend_from_slice(&PLONKY3_EVENT_VECTOR_MAGIC_V2);
        vector.extend_from_slice(
            &u64::try_from(events.len())
                .expect("fixture event count")
                .to_le_bytes(),
        );
        for event in events {
            let bytes = event.canonical_bytes().expect("fixture event bytes");
            vector.extend_from_slice(
                &u32::try_from(bytes.len())
                    .expect("fixture event length")
                    .to_le_bytes(),
            );
            vector.extend_from_slice(&bytes);
        }
        vector
    }

    fn source_record_fixture() -> RecursiveTraceEventV2 {
        let profile = RecursiveCircuitProfileV2::authority_pinned();
        let payload = (0_u8..79).collect::<Vec<_>>();
        RecursiveTraceEventV2::new(
            0,
            RecursiveTraceOpcodeV2::BeginBlock,
            structural_event_id(RecursiveTraceOpcodeV2::BeginBlock, 0, &payload),
            payload,
            &profile,
        )
        .unwrap()
    }

    fn source_air_fixture() -> (Vec<u8>, Vec<u8>) {
        let profile = RecursiveCircuitProfileV2::authority_pinned();
        let source = source_record_fixture();
        let mut events = vec![source.clone()];
        emit_derived_hash_controls(&source, &profile, |event| {
            events.push(event);
            Ok(())
        })
        .unwrap();

        let mut stale_events = events.clone();
        let mut stale_payload = source.payload().to_vec();
        stale_payload[0] ^= 1;
        stale_events[0] = RecursiveTraceEventV2::new(
            0,
            RecursiveTraceOpcodeV2::BeginBlock,
            structural_event_id(RecursiveTraceOpcodeV2::BeginBlock, 0, &stale_payload),
            stale_payload,
            &profile,
        )
        .unwrap();

        (
            encode_event_vector(&events),
            encode_event_vector(&stale_events),
        )
    }

    fn trace_air_fixture() -> (Vec<u8>, Vec<u8>) {
        let profile = RecursiveCircuitProfileV2::authority_pinned();
        let source = source_record_fixture();
        let mut events = Vec::new();
        emit_expanded_trace_hash_controls_for_test(&[source], &profile, |event| {
            events.push(event);
            Ok(())
        })
        .unwrap();
        let mut stale_events = events.clone();
        let begin = stale_events
            .iter_mut()
            .find(|event| {
                event.opcode() == RecursiveTraceOpcodeV2::BeginHash
                    && decode_hash_control(event)
                        .map(|control| control.schema == HashControlSchemaV2::TracePrecommit)
                        .unwrap_or(false)
            })
            .expect("trace precommit begin");
        let mut payload = begin.payload().to_vec();
        payload[3] ^= 1;
        *begin = RecursiveTraceEventV2::new(
            begin.ordinal(),
            begin.opcode(),
            structural_event_id(begin.opcode(), begin.ordinal(), &payload),
            payload,
            &profile,
        )
        .unwrap();
        (
            encode_event_vector(&events),
            encode_event_vector(&stale_events),
        )
    }

    fn uniqueness_list_air_fixture() -> (Vec<u8>, Vec<u8>) {
        let profile = RecursiveCircuitProfileV2::authority_pinned();
        let precommit = uniqueness_precommit_from_rows(&[], &[]).unwrap();
        let source_payloads = [
            (
                RecursiveTraceOpcodeV2::UniquenessPrecommit,
                encode_uniqueness_precommit_value(precommit).unwrap(),
            ),
            (
                RecursiveTraceOpcodeV2::UniquenessChallenge,
                encode_uniqueness_challenge([0x55; 32], [0x66; 32], precommit),
            ),
        ];
        let sources = source_payloads
            .into_iter()
            .enumerate()
            .map(|(ordinal, (opcode, payload))| {
                let ordinal = u64::try_from(ordinal).unwrap();
                RecursiveTraceEventV2::new(
                    ordinal,
                    opcode,
                    structural_event_id(opcode, ordinal, &payload),
                    payload,
                    &profile,
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let mut events = Vec::new();
        emit_expanded_uniqueness_list_hash_controls_for_test(&sources, &profile, |event| {
            events.push(event);
            Ok(())
        })
        .unwrap();

        let mut stale_events = events.clone();
        let block = stale_events
            .iter_mut()
            .find(|event| {
                decode_hash_control(event)
                    .map(|control| {
                        control.schema == HashControlSchemaV2::UniquenessList
                            && control.stage == HashControlStageV2::Block
                            && control
                                .uniqueness_list
                                .map(|binding| {
                                    binding.job == UniquenessListHashJobV2::SpentOriginal
                                })
                                .unwrap_or(false)
                    })
                    .unwrap_or(false)
            })
            .expect("spent-original uniqueness-list block");
        let mut payload = block.payload().to_vec();
        payload[UNIQUENESS_LIST_COMMON_BYTES_V2 + 16] ^= 1;
        *block = RecursiveTraceEventV2::new(
            block.ordinal(),
            block.opcode(),
            structural_event_id(block.opcode(), block.ordinal(), &payload),
            payload,
            &profile,
        )
        .unwrap();
        (
            encode_event_vector(&events),
            encode_event_vector(&stale_events),
        )
    }

    fn uniqueness_transcript_air_fixture() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let profile = RecursiveCircuitProfileV2::authority_pinned();
        let declared_work = RecursiveDeclaredWorkV2::new(
            RecursiveTraceEventCountsV2::default(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        )
        .unwrap();
        let layout = 7;
        let policy_digest = [0x12; 32];
        let old_definition_root = [0x15; 32];
        let post_definition_root = [0x1c; 32];
        let old_settlement_root = *crate::settlement::derive_settlement_root_v2(
            crate::settlement::RootGeneration::SettlementV2,
            layout,
            policy_digest,
            old_definition_root,
        )
        .unwrap()
        .as_bytes();
        let post_settlement_root = *crate::settlement::derive_settlement_root_v2(
            crate::settlement::RootGeneration::SettlementV2,
            layout,
            policy_digest,
            post_definition_root,
        )
        .unwrap()
        .as_bytes();
        let context = RecursivePreUniquenessContextV2::from_parts(
            [0x10; 32],
            [0x11; 32],
            policy_digest,
            [0x13; 32],
            1,
            0,
            layout,
            1,
            1,
            1_000,
            old_settlement_root,
            old_definition_root,
            [0x16; 32],
            [0x17; 32],
            [0x18; 32],
            [0x19; 32],
            [0x1a; 32],
            RecursiveTraceOpcodeV2::grammar_digest(),
            [0x1b; 32],
            declared_work,
        )
        .unwrap();
        let precommit = uniqueness_precommit_from_rows(&[], &[]).unwrap();
        let challenge = encode_uniqueness_challenge(
            context.digest(),
            RecursiveTraceOpcodeV2::grammar_digest(),
            precommit,
        );
        let source_payloads = [
            (
                RecursiveTraceOpcodeV2::UniquenessPrecommit,
                encode_uniqueness_precommit_value(precommit).unwrap(),
            ),
            (
                RecursiveTraceOpcodeV2::UniquenessChallenge,
                challenge.clone(),
            ),
        ];
        let sources = source_payloads
            .into_iter()
            .enumerate()
            .map(|(ordinal, (opcode, payload))| {
                let ordinal = u64::try_from(ordinal).unwrap();
                RecursiveTraceEventV2::new(
                    ordinal,
                    opcode,
                    structural_event_id(opcode, ordinal, &payload),
                    payload,
                    &profile,
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let mut events = Vec::new();
        for source in &sources {
            events.push(source.clone());
            emit_derived_hash_controls(source, &profile, |control| {
                events.push(control);
                Ok(())
            })
            .unwrap();
        }
        emit_expanded_uniqueness_transcript_hash_controls_for_test(
            context,
            precommit,
            post_definition_root,
            post_settlement_root,
            u64::try_from(sources.len()).unwrap(),
            &profile,
            |control| {
                events.push(control);
                Ok(())
            },
        )
        .unwrap();

        let mut statement = vec![0_u8; PLONKY3_BASE_STATEMENT_BYTES_V2];
        let grammar_start =
            PLONKY3_STATEMENT_DIGESTS_OFFSET_V2 + PLONKY3_STATEMENT_GRAMMAR_DIGEST_INDEX_V2 * 32;
        statement[grammar_start..grammar_start + 32]
            .copy_from_slice(&RecursiveTraceOpcodeV2::grammar_digest());
        for (index, digest) in [
            (
                PLONKY3_STATEMENT_PRE_SETTLEMENT_INDEX_V2,
                context.old_settlement_root(),
            ),
            (
                PLONKY3_STATEMENT_POST_SETTLEMENT_INDEX_V2,
                post_settlement_root,
            ),
            (
                PLONKY3_STATEMENT_DECLARED_WORK_INDEX_V2,
                declared_work.digest(),
            ),
            (PLONKY3_STATEMENT_PRE_UNIQUENESS_INDEX_V2, context.digest()),
            (
                PLONKY3_STATEMENT_SPENT_PRECOMMIT_INDEX_V2,
                challenge[65..97].try_into().unwrap(),
            ),
            (
                PLONKY3_STATEMENT_OUTPUT_PRECOMMIT_INDEX_V2,
                challenge[97..129].try_into().unwrap(),
            ),
        ] {
            let start = PLONKY3_STATEMENT_TRANSITION_DIGESTS_OFFSET_V2 + index * 32;
            statement[start..start + 32].copy_from_slice(&digest);
        }
        let mut missing_job = events.clone();
        let remove = missing_job
            .iter()
            .position(|event| {
                decode_hash_control(event)
                    .map(|control| {
                        control.schema == HashControlSchemaV2::UniquenessTranscript
                            && control.stage == HashControlStageV2::End
                            && control.uniqueness_transcript.map(|binding| binding.job)
                                == Some(UniquenessTranscriptHashJobV2::OutputPair1Beta)
                    })
                    .unwrap_or(false)
            })
            .unwrap();
        missing_job.remove(remove);
        (
            encode_event_vector(&events),
            encode_event_vector(&missing_job),
            statement,
        )
    }

    fn predicate_words_for_test(event_vector: &[u8]) -> Vec<u16> {
        predicate_words_for_test_with_statement(
            event_vector,
            &vec![0; PLONKY3_BASE_STATEMENT_BYTES_V2],
        )
    }

    fn predicate_words_for_test_with_statement(event_vector: &[u8], statement: &[u8]) -> Vec<u16> {
        assert_eq!(statement.len(), PLONKY3_BASE_STATEMENT_BYTES_V2);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(PLONKY3_PREDICATE_VECTOR_LABEL_V2);
        bytes.extend_from_slice(
            &u64::try_from(PLONKY3_BASE_STATEMENT_BYTES_V2)
                .expect("fixed statement size")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(statement);
        bytes.extend_from_slice(
            &u64::try_from(event_vector.len())
                .expect("fixture vector size")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(event_vector);
        while bytes.len() % 16 != 0 {
            bytes.push(0);
        }
        bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect()
    }

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
            derive_per_proof_bound(
                PLONKY3_FRI_QUANTUM_SEARCH_BITS_V2,
                PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2,
                PLONKY3_CHALLENGER_QUANTUM_PREIMAGE_BITS_V2,
                3,
            )
            .unwrap()
            .denominator_exponent(),
            122
        );
        assert_eq!(
            derive_lifetime_bound(
                DyadicErrorBoundV2::new(122).unwrap(),
                1 << 20,
                DyadicErrorBoundV2::new(128).unwrap(),
            )
            .unwrap()
            .denominator_exponent(),
            101
        );
        assert!(derive_per_proof_bound(
            PLONKY3_FRI_QUANTUM_SEARCH_BITS_V2,
            PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2,
            PLONKY3_CHALLENGER_QUANTUM_PREIMAGE_BITS_V2,
            0,
        )
        .is_err());
        assert!(derive_lifetime_bound(
            DyadicErrorBoundV2::new(122).unwrap(),
            0,
            DyadicErrorBoundV2::new(128).unwrap(),
        )
        .is_err());
        assert!(matches!(
            derive_lifetime_bound(
                DyadicErrorBoundV2::new(122).unwrap(),
                u64::MAX,
                DyadicErrorBoundV2::new(128).unwrap(),
            ),
            Err(CheckpointError::Overflow)
        ));

        let baseline = RecursiveSecurityBudgetManifestV2::authority_pinned().unwrap();
        let mutations: [fn(&mut RecursiveSecurityBudgetManifestV2); 23] = [
            |value| value.generation += 1,
            |value| value.parameter_generation += 1,
            |value| value.base_field_bits -= 1,
            |value| value.challenge_extension_degree -= 1,
            |value| value.fri_log_blowup += 1,
            |value| value.fri_num_queries -= 1,
            |value| value.fri_commit_pow_bits += 1,
            |value| value.fri_query_pow_bits -= 1,
            |value| value.fri_classical_bits -= 1,
            |value| value.fri_quantum_search_bits -= 1,
            |value| value.hash_output_bits -= 1,
            |value| value.hash_collision_bits -= 1,
            |value| value.challenger_capacity_bits -= 1,
            |value| value.challenger_quantum_preimage_bits -= 1,
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
        for table in poseidon2_table_provers_d4(Poseidon2Config::KOALA_BEAR_D4_W16) {
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
    fn test_source_sha_binding() {
        let (event_vector, stale_vector) = source_air_fixture();
        let words = predicate_words_for_test(&event_vector);
        let prepared = prepare_circuit(&words, Some(&event_vector)).unwrap();
        let mut runner = prepared.circuit.runner();
        runner.set_private_inputs(&prepared.private_inputs).unwrap();
        runner.run().unwrap();

        let stale_words = predicate_words_for_test(&stale_vector);
        assert!(prepare_circuit(&stale_words, Some(&stale_vector)).is_err());
    }

    #[test]
    fn test_trace_precommit_sha_binding() {
        let (event_vector, stale_vector) = trace_air_fixture();
        let words = predicate_words_for_test(&event_vector);
        let prepared = prepare_circuit(&words, Some(&event_vector)).unwrap();
        let mut runner = prepared.circuit.runner();
        runner.set_private_inputs(&prepared.private_inputs).unwrap();
        runner.run().unwrap();

        let stale_words = predicate_words_for_test(&stale_vector);
        let prepared = prepare_circuit(&stale_words, Some(&stale_vector)).unwrap();
        let mut runner = prepared.circuit.runner();
        if runner.set_private_inputs(&prepared.private_inputs).is_ok() {
            assert!(runner.run().is_err());
        }
    }

    #[test]
    fn test_uniqueness_list_sha_binding() {
        let (event_vector, stale_vector) = uniqueness_list_air_fixture();
        let words = predicate_words_for_test(&event_vector);
        let prepared = prepare_circuit(&words, Some(&event_vector)).unwrap();
        let mut runner = prepared.circuit.runner();
        runner.set_private_inputs(&prepared.private_inputs).unwrap();
        runner.run().unwrap();

        let stale_words = predicate_words_for_test(&stale_vector);
        let rejected = match prepare_circuit(&stale_words, Some(&stale_vector)) {
            Err(_) => true,
            Ok(prepared) => {
                let mut runner = prepared.circuit.runner();
                match runner.set_private_inputs(&prepared.private_inputs) {
                    Err(_) => true,
                    Ok(()) => runner.run().is_err(),
                }
            }
        };
        assert!(rejected);
    }

    #[test]
    fn test_uniqueness_transcript_schedule_is_complete_and_statement_bound() {
        let (event_vector, missing_job, statement) = uniqueness_transcript_air_fixture();
        let words = predicate_words_for_test_with_statement(&event_vector, &statement);
        assert!(prepare_circuit(&words, Some(&event_vector)).is_ok());

        let missing_words = predicate_words_for_test_with_statement(&missing_job, &statement);
        assert!(prepare_circuit(&missing_words, Some(&missing_job)).is_err());
    }

    #[test]
    fn test_complete_transition_air_enables_backend_evidence() {
        assert!(require_complete_transition_air_v2().is_ok());
    }
}
