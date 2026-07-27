//! Private Plonky3 owner for the recursive-checkpoint V2 base relation.
//!
//! The public surface is re-exported only by `checkpoint::recursive_v2`.  No
//! Plonky3 field, AIR, PCS, proof, or configuration type crosses that facade.
//! This base STARK is an internal leaf of the canonical hash/FRI recursion
//! chain. Nova is retained only as a differential oracle and is never a proof
//! wrapper or recursive ancestor. End-to-end post-quantum authority additionally
//! requires every outer layer to pass the pinned hash/FRI ancestry gate.

use core::{
    fmt,
    ops::{Deref, Range},
};

use p3_batch_stark::ProverData;
use p3_challenger::DuplexChallenger;
use p3_circuit::ops::poseidon2_perm::Poseidon2PermCall;
use p3_circuit::ops::{
    generate_poseidon2_trace, generate_recompose_trace, NpoTypeId, Poseidon2Config,
};
use p3_circuit::{Circuit, CircuitBuilder, CircuitRunner, ExprId, NonPrimitiveOpId};
use p3_circuit_prover::batch_stark_prover::{
    poseidon2_air_builders_for_configs, recompose_air_builders, recompose_preprocessor,
    Poseidon2Preprocessor,
};
use p3_circuit_prover::common::{get_airs_and_degrees_with_prep, NpoAirBuilder, NpoPreprocessor};
use p3_circuit_prover::{
    recompose_table_provers, BatchStarkProof, BatchStarkProver, CircuitProverData,
    ConstraintProfile, Poseidon2Prover, TablePacking, TableProver,
};
use p3_commit::{ExtensionMmcs, Pcs};
use p3_dft::Radix2DitParallel;
use p3_field::extension::BinomialExtensionField;
use p3_field::{BasedVectorSpace, Field, PrimeCharacteristicRing, PrimeField64};
use p3_fri::{FriParameters, TwoAdicFriPcs};
use p3_koala_bear::{
    default_koalabear_poseidon2_16, default_koalabear_poseidon2_32, KoalaBear, Poseidon2KoalaBear,
};
use p3_lookup::logup::LogUpGadget;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_poseidon2_circuit_air::{KoalaBearD4Width16, KoalaBearD4Width32};
use p3_recursion::backend::fri::FriVerifierResult;
use p3_recursion::pcs::{
    FriProofTargets, InputProofTargets, MerkleCapTargets, RecExtensionValMmcs, Witness,
};
use p3_recursion::verifier::verify_p3_batch_proof_circuit;
use p3_recursion::{
    prove_aggregation_layer, AggregationCircuitFingerprint, AggregationPrepCache, BatchOnly,
    FriRecursionConfig, FriVerifierParams, PcsRecursionBackend, ProveNextLayerParams,
    RecursionInput, RecursiveAir, RecursivePcs, VerificationError,
};
use p3_symmetric::{PaddingFreeSponge, Permutation, TruncatedPermutation};
use p3_uni_stark::{StarkConfig, StarkGenericConfig, Val};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use z00z_crypto::{
    sha256_256, sha256_256_role, CheckpointSha256BlockStreamV2, CheckpointShaRole, SHA256_IV_V2,
};
use z00z_utils::{
    config::{ConfigSource, EnvConfig},
    io::{atomic_write_file_private, path_exists_no_follow, read_file_bounded},
};
use zeroize::Zeroize;

use super::{
    authority_artifacts::{
        ACTIVE_PLONKY3_CIRCUIT_VERSION_V2, ACTIVE_PLONKY3_CRATES_IO_VERSION_V2,
        ACTIVE_PLONKY3_ROOT_COMMON_DIGEST_V2, ACTIVE_PLONKY3_SOURCE_REVISION_V2,
        ACTIVE_VERIFIER_BUNDLE_DIGEST_V2,
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
    version_registry::{
        CheckpointVersionRegistryV2, RecursiveBoundedObjectV2, PLONKY3_PUBLISH_BYTES_V2,
        PLONKY3_TARGET_BYTES_V2, RECURSIVE_INGRESS_BYTES_V2,
    },
};
use crate::{
    settlement::{SettlementStore, SettlementUpdateTraceCircuitDecoderV2},
    CheckpointError,
};

#[path = "plonky3_binary_hash.rs"]
mod plonky3_binary_hash;
#[path = "plonky3_binary_mmcs.rs"]
mod plonky3_binary_mmcs;
#[path = "plonky3_recursion.rs"]
mod plonky3_recursion;
#[path = "plonky3_u16_range.rs"]
mod plonky3_u16_range;

use plonky3_recursion::{
    bind_root_statement_targets, register_root_statement_npo, root_statement_npo_type,
    BinaryRecMmcsV2, Plonky3PcsV2, RootStatementAirBuilderV2, RootStatementPreprocessorV2,
    RootStatementProverV2, RootStatementV2, ROOT_STATEMENT_COMMITMENT_FIELDS_V2,
    ROOT_STATEMENT_COMMITMENT_INDEX_V2, ROOT_STATEMENT_COUNT_INDEX_V2, ROOT_STATEMENT_FIELDS_V2,
    ROOT_STATEMENT_REPLICA_INDEX_V2, ROOT_STATEMENT_START_INDEX_V2, ROOT_STATEMENT_TOTAL_INDEX_V2,
};
use plonky3_u16_range::{
    constrain_u16_bits, register_u16_range_npo, u16_range_npo_type, U16RangeAirBuilderV2,
    U16RangePreprocessorV2, U16RangeProverV2,
};

const PLONKY3_BASE_WIRE_VERSION_V2: u16 = 2;
const PLONKY3_BASE_MAGIC_V2: [u8; 8] = *b"Z00ZP3B2";
const PLONKY3_ROOT_MAGIC_V2: [u8; 8] = *b"Z00ZP3R2";
const PLONKY3_STATEMENT_MAGIC_V2: [u8; 8] = *b"Z00ZP3S2";
const PLONKY3_PARAMETER_MAGIC_V2: [u8; 8] = *b"Z00ZP3P2";
const PLONKY3_SECURITY_MAGIC_V2: [u8; 8] = *b"Z00ZP3Q2";
const PLONKY3_EVENT_VECTOR_MAGIC_V2: [u8; 8] = *b"Z00ZP3E2";
const PLONKY3_STATEMENT_EXEC_TX_COUNT_BYTES_V2: usize = core::mem::size_of::<u32>();
const PLONKY3_CHUNK_BYTES_V2: usize = RECURSIVE_INGRESS_BYTES_V2;
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
const PLONKY3_PREDICATE_VECTOR_LABEL_V2: &[u8] = b"z00z.plonky3.base.predicate-vector.v2";
const PLONKY3_FRI_LOG_BLOWUP_V2: u8 = 2;
const PLONKY3_FRI_LOG_FINAL_POLY_LEN_V2: u8 = 0;
const PLONKY3_FRI_MAX_LOG_ARITY_V2: u8 = 3;
const PLONKY3_FRI_NUM_QUERIES_V2: u16 = 62;
const PLONKY3_FRI_COMMIT_POW_BITS_V2: u8 = 0;
const PLONKY3_FRI_QUERY_POW_BITS_V2: u8 = 0;
const PLONKY3_BASE_FIELD_BITS_V2: u16 = 31;
const PLONKY3_CHALLENGE_EXTENSION_DEGREE_V2: u8 = 4;
const PLONKY3_FRI_REPLICA_COUNT_V2: u8 = 3;
const PLONKY3_FRI_PHYSICAL_CLASSICAL_BITS_V2: u16 = 124;
const PLONKY3_FRI_PHYSICAL_QUANTUM_SEARCH_BITS_V2: u16 = 62;
const PLONKY3_FRI_CLASSICAL_BITS_V2: u16 = 321;
const PLONKY3_FRI_QUANTUM_SEARCH_BITS_V2: u16 = 135;
const PLONKY3_HASH_OUTPUT_BITS_V2: u16 = 496;
const PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2: u16 = 165;
const PLONKY3_CHALLENGER_CAPACITY_BITS_V2: u16 = 248;
const PLONKY3_CHALLENGER_PHYSICAL_QUANTUM_PREIMAGE_BITS_V2: u16 = 124;
const PLONKY3_CHALLENGER_QUANTUM_PREIMAGE_BITS_V2: u16 = 321;
const PLONKY3_MMCS_WIDTH_V2: usize = 32;
const PLONKY3_MMCS_RATE_V2: usize = 24;
const PLONKY3_MMCS_DIGEST_ELEMS_V2: usize = 16;
const PLONKY3_TABLE_MIN_HEIGHT_V2: usize = 8;
const PLONKY3_TABLE_PUBLIC_LANES_V2: usize = 4;
const PLONKY3_TABLE_ALU_LANES_V2: usize = 4;
const PLONKY3_TRACE_EXTENSION_DEGREE_V2: u8 = 4;
const PLONKY3_SECURITY_GENERATION_V2: u16 = 6;
const PLONKY3_SECURITY_COMPOSITION_RULE_GENERATION_V2: u16 = 5;
const PLONKY3_BASE_RECURSION_DEPTH_V2: u16 = 31;
const PLONKY3_LOGICAL_LEAF_COUNT_V2: u32 = 65_535;
const PLONKY3_LOGICAL_NODE_COUNT_V2: u32 = PLONKY3_LOGICAL_LEAF_COUNT_V2 * 2 - 1;
const PLONKY3_MAX_ACCEPTED_EPOCH_PROOFS_V2: u64 = 1 << 20;
const PLONKY3_TRANSITION_JMT_FIRST_PART_V2: u16 = 2;
// Twelve workers avoid the repeated high-CPU Rayon tail observed with the
// sixteen-worker nested Batch-STARK/DFT schedule while retaining bounded
// parallel proving throughput. This is a prover-only scheduling choice; it
// does not alter the proof grammar, transcript, or verifier parameters.
const PLONKY3_PROVER_THREADS_V2: usize = 12;
// Two independent recursive pairs keep the 28-logical-CPU prover host busy
// without changing the canonical binary tree. Each pair retains the audited
// twelve-thread prover schedule; the separate 16 GiB acceptance gate controls
// their combined memory high-water mark.
const PLONKY3_AGGREGATION_WORKERS_V2: usize = 2;
const PLONKY3_PREDICATE_PACKING_GENERATION_V2: u8 = 5;
const PLONKY3_STRUCTURAL_ITEMS_PER_CHUNK_V2: u16 = 3;
// Six SHA blocks keep the packed ALU trace below the next power-of-two
// proving domain. Eight blocks would pad to the same domain as the failed
// twelve-block generation while four would add leaves without another
// domain reduction.
const PLONKY3_HASH_ITEMS_PER_CHUNK_V2: u16 = 6;
const PLONKY3_SOURCE_ITEMS_PER_CHUNK_V2: u16 = 4;
const PLONKY3_MINIMUM_RESIDUAL_BITS_V2: u16 = 100;
const PLONKY3_PER_PROOF_BOUND_BITS_V2: u16 = 133;
const PLONKY3_LIFETIME_BOUND_BITS_V2: u16 = 107;
const PLONKY3_CHUNK_CACHE_GENERATION_V2: u16 = 14;
const PLONKY3_PREVIOUS_AGGREGATION_TREE_GENERATION_V2: u8 = 10;
const PLONKY3_AGGREGATION_TREE_GENERATION_V2: u8 = 11;
const PLONKY3_FIRST_REPLICA_FOLD_ORDINAL_V2: u8 = PLONKY3_FRI_REPLICA_COUNT_V2;
const PLONKY3_FINAL_REPLICA_FOLD_ORDINAL_V2: u8 = PLONKY3_FRI_REPLICA_COUNT_V2 + 1;

type Plonky3TraceFieldV2 = BinomialExtensionField<KoalaBear, 4>;
type Plonky3PermutationV2 = Poseidon2KoalaBear<PLONKY3_MMCS_WIDTH_V2>;
type Plonky3HashV2 = PaddingFreeSponge<
    Plonky3PermutationV2,
    PLONKY3_MMCS_WIDTH_V2,
    PLONKY3_MMCS_RATE_V2,
    PLONKY3_MMCS_DIGEST_ELEMS_V2,
>;
type Plonky3CompressionV2 = TruncatedPermutation<
    Plonky3PermutationV2,
    2,
    PLONKY3_MMCS_DIGEST_ELEMS_V2,
    PLONKY3_MMCS_WIDTH_V2,
>;
type Plonky3ValueMmcsV2 = MerkleTreeMmcs<
    <KoalaBear as Field>::Packing,
    <KoalaBear as Field>::Packing,
    Plonky3HashV2,
    Plonky3CompressionV2,
    2,
    PLONKY3_MMCS_DIGEST_ELEMS_V2,
>;
type Plonky3ChallengeV2 = BinomialExtensionField<KoalaBear, 4>;
type Plonky3ChallengeMmcsV2 = ExtensionMmcs<KoalaBear, Plonky3ChallengeV2, Plonky3ValueMmcsV2>;
type Plonky3InnerPcsV2 = TwoAdicFriPcs<
    KoalaBear,
    Radix2DitParallel<KoalaBear>,
    Plonky3ValueMmcsV2,
    Plonky3ChallengeMmcsV2,
>;
type Plonky3ChallengerV2 =
    DuplexChallenger<KoalaBear, Plonky3PermutationV2, PLONKY3_MMCS_WIDTH_V2, PLONKY3_MMCS_RATE_V2>;
type Plonky3RawStarkConfigV2 = StarkConfig<Plonky3PcsV2, Plonky3ChallengeV2, Plonky3ChallengerV2>;
type Plonky3RecValueMmcsV2 = BinaryRecMmcsV2;
type Plonky3RecInputProofV2 =
    InputProofTargets<KoalaBear, Plonky3ChallengeV2, Plonky3RecValueMmcsV2>;
type Plonky3RecOpeningProofV2 = FriProofTargets<
    KoalaBear,
    Plonky3ChallengeV2,
    RecExtensionValMmcs<
        KoalaBear,
        Plonky3ChallengeV2,
        PLONKY3_MMCS_DIGEST_ELEMS_V2,
        Plonky3RecValueMmcsV2,
    >,
    Plonky3RecInputProofV2,
    Witness<KoalaBear>,
>;

#[derive(Clone)]
struct Plonky3StarkConfigV2 {
    config: Arc<Plonky3RawStarkConfigV2>,
    fri_verifier_params: FriVerifierParams,
}

impl Deref for Plonky3StarkConfigV2 {
    type Target = Plonky3RawStarkConfigV2;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl StarkGenericConfig for Plonky3StarkConfigV2 {
    type Challenge = Plonky3ChallengeV2;
    type Challenger = Plonky3ChallengerV2;
    type Pcs = Plonky3PcsV2;

    fn pcs(&self) -> &Self::Pcs {
        self.config.pcs()
    }

    fn initialise_challenger(&self) -> Self::Challenger {
        self.config.initialise_challenger()
    }
}

impl FriRecursionConfig for Plonky3StarkConfigV2
where
    Plonky3PcsV2: RecursivePcs<
        Plonky3StarkConfigV2,
        Plonky3RecInputProofV2,
        Plonky3RecOpeningProofV2,
        MerkleCapTargets<KoalaBear, PLONKY3_MMCS_DIGEST_ELEMS_V2>,
        <Plonky3PcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::Domain,
    >,
{
    type Commitment = MerkleCapTargets<KoalaBear, PLONKY3_MMCS_DIGEST_ELEMS_V2>;
    type InputProof = Plonky3RecInputProofV2;
    type OpeningProof = Plonky3RecOpeningProofV2;
    type RawOpeningProof = <Plonky3PcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::Proof;

    const DIGEST_ELEMS: usize = PLONKY3_MMCS_DIGEST_ELEMS_V2;

    fn with_fri_opening_proof<'a, A, R>(
        previous: &RecursionInput<'a, Self, A>,
        use_proof: impl FnOnce(&Self::RawOpeningProof) -> R,
    ) -> R
    where
        A: RecursiveAir<Val<Self>, Self::Challenge, LogUpGadget>,
    {
        match previous {
            RecursionInput::UniStark { proof, .. } => use_proof(&proof.opening_proof),
            RecursionInput::BatchStark { proof, .. } => use_proof(&proof.proof.opening_proof),
        }
    }

    fn prepare_circuit_for_verification(
        &self,
        circuit: &mut CircuitBuilder<Self::Challenge>,
    ) -> Result<(), VerificationError> {
        circuit.enable_poseidon2_perm::<KoalaBearD4Width16, _>(
            generate_poseidon2_trace::<Plonky3ChallengeV2, KoalaBearD4Width16>,
            default_koalabear_poseidon2_16(),
        );
        circuit.enable_poseidon2_perm_width_32::<KoalaBearD4Width32, _>(
            generate_poseidon2_trace::<Plonky3ChallengeV2, KoalaBearD4Width32>,
            default_koalabear_poseidon2_32(),
        );
        circuit.enable_recompose::<KoalaBear>(
            generate_recompose_trace::<KoalaBear, Plonky3ChallengeV2>,
        );
        // Keep the recursive proof table manifest invariant across every layer.
        // W16 appears in base proofs but is otherwise not needed by the outer
        // binary-W32 verifier; this fully constrained constant row prevents a
        // first-layer [W32, W16, recompose] proof from collapsing to
        // [W32, recompose] and becoming unverifiable by the next layer.
        let zero = circuit.define_const(Plonky3ChallengeV2::ZERO);
        let expected = default_koalabear_poseidon2_16().permute([KoalaBear::ZERO; 16]);
        let (_, outputs) = circuit.add_poseidon2_perm(&Poseidon2PermCall {
            config: Poseidon2Config::KOALA_BEAR_D4_W16,
            new_start: true,
            merkle_path: false,
            mmcs_bit: None,
            mmcs_bit2: None,
            inputs: vec![Some(zero); 4],
            out_ctl: vec![true; 2],
            return_all_outputs: false,
            mmcs_index_sum: None,
        })?;
        for (output, expected) in outputs.into_iter().take(2).zip(expected.chunks_exact(4)) {
            let output = output.ok_or(p3_circuit::CircuitBuilderError::MissingOutput)?;
            let expected = circuit.define_const(Plonky3ChallengeV2::new([
                expected[0],
                expected[1],
                expected[2],
                expected[3],
            ]));
            circuit.connect(output, expected);
        }
        Ok(())
    }

    fn pcs_verifier_params(
        &self,
    ) -> &<Plonky3PcsV2 as RecursivePcs<
        Plonky3StarkConfigV2,
        Plonky3RecInputProofV2,
        Plonky3RecOpeningProofV2,
        MerkleCapTargets<KoalaBear, PLONKY3_MMCS_DIGEST_ELEMS_V2>,
        <Plonky3PcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::Domain,
    >>::VerifierParams {
        &self.fri_verifier_params
    }

    fn set_fri_private_data(
        _runner: &mut CircuitRunner<'_, Self::Challenge>,
        op_ids: &[NonPrimitiveOpId],
        _opening_proof: &Self::RawOpeningProof,
    ) -> Result<(), &'static str> {
        if op_ids.is_empty() {
            Ok(())
        } else {
            Err("binary W32 recursion emitted unexpected private MMCS operations")
        }
    }
}
type CircuitByteBitsV2 = [ExprId; 8];
type Plonky3WordBitsV2 = [ExprId; 32];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum AirDomainV2 {
    Structural = 1,
    Hash = 2,
    Source = 3,
    Lists = 4,
    Uniqueness = 5,
    Trace = 6,
    Transition = 7,
    #[cfg(test)]
    Full = 255,
}

impl AirDomainV2 {
    const fn tag(self) -> u8 {
        self as u8
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Structural => "chunk_structural",
            Self::Hash => "chunk_hash",
            Self::Source => "chunk_source",
            Self::Lists => "chunk_lists",
            Self::Uniqueness => "chunk_uniqueness",
            Self::Trace => "chunk_trace",
            Self::Transition => "chunk_transition",
            #[cfg(test)]
            Self::Full => "chunk_full",
        }
    }

    const fn includes(self, domain: Self) -> bool {
        #[cfg(test)]
        if matches!(self, Self::Full) {
            return true;
        }
        self.tag() == domain.tag()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AirChunkV2 {
    domain: AirDomainV2,
    index: u16,
    count: u16,
    replica: u8,
}

struct AggregationNodeV2 {
    proof: BatchStarkProof<Plonky3StarkConfigV2>,
    replica: u8,
    leaf_start: u16,
    leaf_count: u16,
    depth: u16,
}

struct AggregationWaveJobV2 {
    ordinal: usize,
    segment: usize,
    left: AggregationNodeV2,
    right: AggregationNodeV2,
}

struct AggregationWaveResultV2 {
    ordinal: usize,
    segment: usize,
    node: AggregationNodeV2,
}

enum AggregationWaveMessageV2 {
    Job(AggregationWaveJobV2),
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AggregationRelationV2 {
    LeafRange,
    FirstReplicaFold,
    FinalReplicaFold,
}

impl AggregationRelationV2 {
    const fn cache_tag(self) -> u8 {
        match self {
            Self::LeafRange => 0,
            Self::FirstReplicaFold => 1,
            Self::FinalReplicaFold => 2,
        }
    }

    const fn commitment_domain(self) -> RootCommitmentDomainV2 {
        match self {
            Self::LeafRange => RootCommitmentDomainV2::LeafRange,
            Self::FirstReplicaFold => RootCommitmentDomainV2::FirstReplicaFold,
            Self::FinalReplicaFold => RootCommitmentDomainV2::FinalReplicaFold,
        }
    }

    const fn replica_ordinals(self) -> Option<(u8, u8, u8)> {
        match self {
            Self::LeafRange => None,
            Self::FirstReplicaFold => Some((0, 1, PLONKY3_FIRST_REPLICA_FOLD_ORDINAL_V2)),
            Self::FinalReplicaFold => Some((
                PLONKY3_FIRST_REPLICA_FOLD_ORDINAL_V2,
                2,
                PLONKY3_FINAL_REPLICA_FOLD_ORDINAL_V2,
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootCommitmentDomainV2 {
    LeafRange,
    FirstReplicaFold,
    FinalReplicaFold,
}

impl RootCommitmentDomainV2 {
    const fn tag(self) -> u8 {
        match self {
            Self::LeafRange => 2,
            Self::FirstReplicaFold => 3,
            Self::FinalReplicaFold => 4,
        }
    }
}

struct RecursiveRootProofV2 {
    replica: u8,
    leaf_count: u16,
    depth: u16,
    proof: BatchStarkProof<Plonky3StarkConfigV2>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RecursiveRootBindingV2 {
    fold_ordinal: u8,
    leaf_count: u16,
    depth: u16,
    common_digest: [u8; 32],
    proof_digest: [u8; 32],
}

struct RecursiveRootEnvelopeV2 {
    leaf_manifest_digest: [u8; 32],
    root: RecursiveRootProofV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LocalRecursiveProofBindingV2 {
    root: RecursiveRootBindingV2,
}

impl AirChunkV2 {
    #[cfg(test)]
    const fn singleton(domain: AirDomainV2) -> Self {
        Self {
            domain,
            index: 0,
            count: 1,
            replica: 0,
        }
    }

    const fn replicated(domain: AirDomainV2, index: u16, count: u16, replica: u8) -> Self {
        Self {
            domain,
            index,
            count,
            replica,
        }
    }

    fn validate(self) -> Result<(), CheckpointError> {
        if self.count == 0
            || self.index >= self.count
            || self.replica >= PLONKY3_FRI_REPLICA_COUNT_V2
        {
            return Err(CheckpointError::Canonical);
        }
        if !matches!(
            self.domain,
            AirDomainV2::Structural
                | AirDomainV2::Hash
                | AirDomainV2::Source
                | AirDomainV2::Lists
                | AirDomainV2::Uniqueness
                | AirDomainV2::Transition
        ) && (self.index != 0 || self.count != 1)
        {
            return Err(CheckpointError::Canonical);
        }
        Ok(())
    }
}

fn bounded_chunk_count(item_count: usize, items_per_chunk: u16) -> Result<u16, CheckpointError> {
    if item_count == 0 || items_per_chunk == 0 {
        return Err(CheckpointError::Canonical);
    }
    let items_per_chunk = usize::from(items_per_chunk);
    let count = item_count
        .checked_add(items_per_chunk - 1)
        .ok_or(CheckpointError::Overflow)?
        / items_per_chunk;
    u16::try_from(count).map_err(|_| CheckpointError::Limit)
}

fn bounded_chunk_range(
    item_count: usize,
    chunk: AirChunkV2,
    items_per_chunk: u16,
) -> Result<Range<u16>, CheckpointError> {
    chunk.validate()?;
    if chunk.count != bounded_chunk_count(item_count, items_per_chunk)? {
        return Err(CheckpointError::Canonical);
    }
    let item_count = u16::try_from(item_count).map_err(|_| CheckpointError::Limit)?;
    let start = chunk
        .index
        .checked_mul(items_per_chunk)
        .ok_or(CheckpointError::Overflow)?;
    let end = start
        .checked_add(items_per_chunk)
        .unwrap_or(u16::MAX)
        .min(item_count);
    if start >= end {
        return Err(CheckpointError::Canonical);
    }
    Ok(start..end)
}

fn ordinal_in_bounded_chunk(ordinal: u64, chunk: AirChunkV2, items_per_chunk: u16) -> bool {
    ordinal / u64::from(items_per_chunk) == u64::from(chunk.index)
}

fn emit_resource_phase(phase: &str) {
    if matches!(
        EnvConfig.get("Z00Z_PLONKY3_RESOURCE_TELEMETRY"),
        Ok(Some(_))
    ) {
        eprintln!("Z00Z_PLONKY3_PHASE_V1 {phase}");
    }
}

fn emit_resource_error(label: &str, error: &impl core::fmt::Debug) {
    if matches!(
        EnvConfig.get("Z00Z_PLONKY3_RESOURCE_TELEMETRY"),
        Ok(Some(_))
    ) {
        eprintln!("Z00Z_PLONKY3_ERROR_V1 {label} {error:?}");
    }
}

static PLONKY3_CHUNK_TIMER_V2: OnceLock<Instant> = OnceLock::new();

fn emit_chunk_progress(stage: &str, chunk: AirChunkV2) {
    if matches!(
        EnvConfig.get("Z00Z_PLONKY3_RESOURCE_TELEMETRY"),
        Ok(Some(_))
    ) {
        let elapsed_ms = PLONKY3_CHUNK_TIMER_V2
            .get_or_init(Instant::now)
            .elapsed()
            .as_millis();
        eprintln!(
            "Z00Z_PLONKY3_CHUNK_V1 {{\"stage\":\"{stage}\",\"domain\":\"{}\",\"replica\":{},\"index\":{},\"count\":{},\"elapsed_ms\":{elapsed_ms}}}",
            chunk.domain.name(),
            chunk.replica,
            chunk.index,
            chunk.count
        );
    }
}

fn emit_chunk_trace_dimensions(chunk: AirChunkV2, dimensions: &Plonky3TraceDimensionsV2) {
    if matches!(
        EnvConfig.get("Z00Z_PLONKY3_RESOURCE_TELEMETRY"),
        Ok(Some(_))
    ) {
        eprintln!(
            concat!(
                "Z00Z_PLONKY3_TRACE_DIMENSIONS_V1 ",
                "{{\"domain\":\"{}\",\"replica\":{},\"index\":{},\"count\":{},",
                "\"dimensions\":{{\"chunk_count\":{},\"predicate_words\":{},",
                "\"event_vector_bytes\":{},\"circuit_witnesses\":{},",
                "\"circuit_operations\":{},\"private_inputs\":{},\"witness_rows\":{},",
                "\"constant_rows\":{},\"public_rows\":{},\"alu_rows\":{},",
                "\"non_primitive_tables\":{},\"non_primitive_rows\":{},",
                "\"max_chunk_witnesses\":{},\"max_chunk_operations\":{},",
                "\"max_chunk_alu_rows\":{},\"max_chunk_npo_rows\":{}}}}}"
            ),
            chunk.domain.name(),
            chunk.replica,
            chunk.index,
            chunk.count,
            dimensions.chunk_count,
            dimensions.predicate_words,
            dimensions.event_vector_bytes,
            dimensions.circuit_witnesses,
            dimensions.circuit_operations,
            dimensions.private_inputs,
            dimensions.witness_rows,
            dimensions.constant_rows,
            dimensions.public_rows,
            dimensions.alu_rows,
            dimensions.non_primitive_tables,
            dimensions.non_primitive_rows,
            dimensions.max_chunk_witnesses,
            dimensions.max_chunk_operations,
            dimensions.max_chunk_alu_rows,
            dimensions.max_chunk_npo_rows,
        );
    }
}

/// Run exactly one heavyweight backend operation in fresh bounded Rayon
/// workers. Dropping the pool at the operation boundary releases worker-local
/// allocator arenas and prevents one chunk's prover state from accumulating in
/// the next chunk, cache verification, or recursive aggregation layer.
fn build_bounded_prover_pool(
    operation: &'static str,
) -> Result<rayon::ThreadPool, CheckpointError> {
    build_bounded_prover_pool_with_threads(operation, PLONKY3_PROVER_THREADS_V2)
}

fn build_bounded_prover_pool_with_threads(
    operation: &'static str,
    threads: usize,
) -> Result<rayon::ThreadPool, CheckpointError> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .thread_name(move |index| format!("z00z-p3-{operation}-{index}"))
        .build()
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "bounded Plonky3 {operation} pool construction failed: {error}"
            ))
        })
}

fn run_in_fresh_prover_pool<T>(
    operation: &'static str,
    task: impl FnOnce() -> Result<T, CheckpointError> + Send,
) -> Result<T, CheckpointError>
where
    T: Send,
{
    let pool = build_bounded_prover_pool(operation)?;
    let result = pool.install(task);
    drop(pool);
    trim_prover_heap();
    result
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
    fri_replica_count: u8,
    fri_physical_classical_bits: u16,
    fri_physical_quantum_search_bits: u16,
    fri_classical_bits: u16,
    fri_quantum_search_bits: u16,
    hash_output_bits: u16,
    hash_collision_bits: u16,
    challenger_capacity_bits: u16,
    challenger_physical_quantum_preimage_bits: u16,
    challenger_quantum_preimage_bits: u16,
    component_count: u16,
    recursion_depth: u16,
    logical_leaf_count: u32,
    logical_node_count: u32,
    composition_rule_generation: u16,
    per_proof_bound: DyadicErrorBoundV2,
    max_accepted_epoch_proofs: u64,
    inherited_bound: Option<DyadicErrorBoundV2>,
    lifetime_bound: DyadicErrorBoundV2,
    minimum_residual_bits: u16,
    canonical_bytes: Vec<u8>,
}

impl RecursiveSecurityBudgetManifestV2 {
    /// The one live Plan-07 quantum-aware budget. Each degree-4 KoalaBear
    /// proof contributes 124 classical / 62 conservative QROM-search bits.
    /// The `131,069`-node union is charged inside each transcript-separated
    /// replica before the three independent replica bounds are multiplied:
    /// FRI therefore contributes `(62 - 17) * 3 = 135` quantum bits, not the
    /// unsound `62 * 3 - 17`. The shared 496-bit Poseidon2 digest family
    /// contributes `165 - 17 = 148` generic quantum-collision bits and is not
    /// amplified across replicas. Composing FRI, hash, and challenger families
    /// gives `2^-133`; the finite `2^20` horizon plus inherited rotation loss
    /// remains at `2^-107`.
    pub fn authority_pinned() -> Result<Self, CheckpointError> {
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        let parameter_generation = registry
            .row(RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest)?
            .parameter_generation
            .ok_or(CheckpointError::Authority)?;
        let per_proof_bound = derive_replica_tree_bound(
            PLONKY3_FRI_PHYSICAL_QUANTUM_SEARCH_BITS_V2,
            PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2,
            PLONKY3_CHALLENGER_PHYSICAL_QUANTUM_PREIMAGE_BITS_V2,
            PLONKY3_FRI_REPLICA_COUNT_V2,
            PLONKY3_LOGICAL_NODE_COUNT_V2,
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
            fri_replica_count: PLONKY3_FRI_REPLICA_COUNT_V2,
            fri_physical_classical_bits: PLONKY3_FRI_PHYSICAL_CLASSICAL_BITS_V2,
            fri_physical_quantum_search_bits: PLONKY3_FRI_PHYSICAL_QUANTUM_SEARCH_BITS_V2,
            fri_classical_bits: PLONKY3_FRI_CLASSICAL_BITS_V2,
            fri_quantum_search_bits: PLONKY3_FRI_QUANTUM_SEARCH_BITS_V2,
            hash_output_bits: PLONKY3_HASH_OUTPUT_BITS_V2,
            hash_collision_bits: PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2,
            challenger_capacity_bits: PLONKY3_CHALLENGER_CAPACITY_BITS_V2,
            challenger_physical_quantum_preimage_bits:
                PLONKY3_CHALLENGER_PHYSICAL_QUANTUM_PREIMAGE_BITS_V2,
            challenger_quantum_preimage_bits: PLONKY3_CHALLENGER_QUANTUM_PREIMAGE_BITS_V2,
            component_count: 3,
            recursion_depth: PLONKY3_BASE_RECURSION_DEPTH_V2,
            logical_leaf_count: PLONKY3_LOGICAL_LEAF_COUNT_V2,
            logical_node_count: PLONKY3_LOGICAL_NODE_COUNT_V2,
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
        let expected_physical_fri = raw_fri.min(field_security);
        let expected_physical_quantum = expected_physical_fri
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?
            / 2;
        let node_union_loss = ceil_log2_terms(u64::from(self.logical_node_count))?;
        let expected_fri = expected_physical_fri
            .checked_sub(node_union_loss)
            .ok_or_else(security_budget_error)?
            .checked_mul(u16::from(self.fri_replica_count))
            .ok_or(CheckpointError::Overflow)?;
        let expected_fri_quantum = expected_physical_quantum
            .checked_sub(node_union_loss)
            .ok_or_else(security_budget_error)?
            .checked_mul(u16::from(self.fri_replica_count))
            .ok_or(CheckpointError::Overflow)?;
        let expected_challenger_physical = self.challenger_capacity_bits / 2;
        let expected_challenger = expected_challenger_physical
            .checked_sub(node_union_loss)
            .ok_or_else(security_budget_error)?
            .checked_mul(u16::from(self.fri_replica_count))
            .ok_or(CheckpointError::Overflow)?;
        let expected_per_proof = derive_replica_tree_bound(
            self.fri_physical_quantum_search_bits,
            self.hash_collision_bits,
            self.challenger_physical_quantum_preimage_bits,
            self.fri_replica_count,
            self.logical_node_count,
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
            || self.fri_replica_count != PLONKY3_FRI_REPLICA_COUNT_V2
            || self.fri_physical_classical_bits != expected_physical_fri
            || self.fri_physical_classical_bits != PLONKY3_FRI_PHYSICAL_CLASSICAL_BITS_V2
            || self.fri_physical_quantum_search_bits != expected_physical_quantum
            || self.fri_physical_quantum_search_bits != PLONKY3_FRI_PHYSICAL_QUANTUM_SEARCH_BITS_V2
            || self.fri_classical_bits != expected_fri
            || self.fri_quantum_search_bits != expected_fri_quantum
            || self.fri_quantum_search_bits != PLONKY3_FRI_QUANTUM_SEARCH_BITS_V2
            || self.hash_output_bits != PLONKY3_HASH_OUTPUT_BITS_V2
            || self.hash_collision_bits != self.hash_output_bits / 3
            || self.hash_collision_bits != PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2
            || self.challenger_capacity_bits != PLONKY3_CHALLENGER_CAPACITY_BITS_V2
            || self.challenger_physical_quantum_preimage_bits != expected_challenger_physical
            || self.challenger_physical_quantum_preimage_bits
                != PLONKY3_CHALLENGER_PHYSICAL_QUANTUM_PREIMAGE_BITS_V2
            || self.challenger_quantum_preimage_bits != expected_challenger
            || self.challenger_quantum_preimage_bits != PLONKY3_CHALLENGER_QUANTUM_PREIMAGE_BITS_V2
            || self.component_count != 3
            || self.recursion_depth != PLONKY3_BASE_RECURSION_DEPTH_V2
            || self.logical_leaf_count != PLONKY3_LOGICAL_LEAF_COUNT_V2
            || self.logical_node_count != PLONKY3_LOGICAL_NODE_COUNT_V2
            || self.logical_node_count
                != self
                    .logical_leaf_count
                    .checked_mul(2)
                    .and_then(|value| value.checked_sub(1))
                    .ok_or(CheckpointError::Overflow)?
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
        let mut bytes = Vec::with_capacity(71);
        bytes.extend_from_slice(&PLONKY3_SECURITY_MAGIC_V2);
        bytes.extend_from_slice(&self.generation.to_le_bytes());
        bytes.extend_from_slice(&self.parameter_generation.to_le_bytes());
        bytes.extend_from_slice(&self.base_field_bits.to_le_bytes());
        bytes.push(self.challenge_extension_degree);
        bytes.push(self.fri_log_blowup);
        bytes.extend_from_slice(&self.fri_num_queries.to_le_bytes());
        bytes.push(self.fri_commit_pow_bits);
        bytes.push(self.fri_query_pow_bits);
        bytes.push(self.fri_replica_count);
        bytes.extend_from_slice(&self.fri_physical_classical_bits.to_le_bytes());
        bytes.extend_from_slice(&self.fri_physical_quantum_search_bits.to_le_bytes());
        bytes.extend_from_slice(&self.fri_classical_bits.to_le_bytes());
        bytes.extend_from_slice(&self.fri_quantum_search_bits.to_le_bytes());
        bytes.extend_from_slice(&self.hash_output_bits.to_le_bytes());
        bytes.extend_from_slice(&self.hash_collision_bits.to_le_bytes());
        bytes.extend_from_slice(&self.challenger_capacity_bits.to_le_bytes());
        bytes.extend_from_slice(&self.challenger_physical_quantum_preimage_bits.to_le_bytes());
        bytes.extend_from_slice(&self.challenger_quantum_preimage_bits.to_le_bytes());
        bytes.extend_from_slice(&self.component_count.to_le_bytes());
        bytes.extend_from_slice(&self.recursion_depth.to_le_bytes());
        bytes.extend_from_slice(&self.logical_leaf_count.to_le_bytes());
        bytes.extend_from_slice(&self.logical_node_count.to_le_bytes());
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
        if payload.len() != 71 || payload[..8] != PLONKY3_SECURITY_MAGIC_V2 {
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
        let fri_replica_count = take_array::<1>(payload, &mut cursor)?[0];
        let fri_physical_classical_bits = take_u16(payload, &mut cursor)?;
        let fri_physical_quantum_search_bits = take_u16(payload, &mut cursor)?;
        let fri_classical_bits = take_u16(payload, &mut cursor)?;
        let fri_quantum_search_bits = take_u16(payload, &mut cursor)?;
        let hash_output_bits = take_u16(payload, &mut cursor)?;
        let hash_collision_bits = take_u16(payload, &mut cursor)?;
        let challenger_capacity_bits = take_u16(payload, &mut cursor)?;
        let challenger_physical_quantum_preimage_bits = take_u16(payload, &mut cursor)?;
        let challenger_quantum_preimage_bits = take_u16(payload, &mut cursor)?;
        let component_count = take_u16(payload, &mut cursor)?;
        let recursion_depth = take_u16(payload, &mut cursor)?;
        let logical_leaf_count = take_u32(payload, &mut cursor)?;
        let logical_node_count = take_u32(payload, &mut cursor)?;
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
            fri_replica_count,
            fri_physical_classical_bits,
            fri_physical_quantum_search_bits,
            fri_classical_bits,
            fri_quantum_search_bits,
            hash_output_bits,
            hash_collision_bits,
            challenger_capacity_bits,
            challenger_physical_quantum_preimage_bits,
            challenger_quantum_preimage_bits,
            component_count,
            recursion_depth,
            logical_leaf_count,
            logical_node_count,
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

fn derive_replica_tree_bound(
    fri_physical_bits: u16,
    hash_physical_bits: u16,
    challenger_physical_bits: u16,
    replica_count: u8,
    logical_node_count: u32,
    component_count: u16,
) -> Result<DyadicErrorBoundV2, CheckpointError> {
    if replica_count == 0 {
        return Err(security_budget_error());
    }
    let node_union_loss = ceil_log2_terms(u64::from(logical_node_count))?;
    let fri_amplified = fri_physical_bits
        .checked_sub(node_union_loss)
        .ok_or_else(security_budget_error)?;
    let fri_amplified = fri_amplified
        .checked_mul(u16::from(replica_count))
        .ok_or(CheckpointError::Overflow)?;
    let challenger_amplified = challenger_physical_bits
        .checked_sub(node_union_loss)
        .ok_or_else(security_budget_error)?
        .checked_mul(u16::from(replica_count))
        .ok_or(CheckpointError::Overflow)?;
    let shared_hash = hash_physical_bits
        .checked_sub(node_union_loss)
        .ok_or_else(security_budget_error)?;
    derive_per_proof_bound(
        fri_amplified,
        shared_hash,
        challenger_amplified,
        component_count,
    )
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
        Self::authority_pinned_for_aggregation_generation(
            security,
            PLONKY3_AGGREGATION_TREE_GENERATION_V2,
        )
    }

    fn authority_pinned_for_aggregation_generation(
        security: &RecursiveSecurityBudgetManifestV2,
        aggregation_generation: u8,
    ) -> Result<Self, CheckpointError> {
        if aggregation_generation != PLONKY3_AGGREGATION_TREE_GENERATION_V2
            && aggregation_generation != PLONKY3_PREVIOUS_AGGREGATION_TREE_GENERATION_V2
        {
            return Err(CheckpointError::Authority);
        }
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
        put_short_str(&mut bytes, "poseidon2_koala_bear_width32_rate24_digest16")?;
        put_short_str(&mut bytes, "poseidon2_koala_bear_d4_width16_semantic_air")?;
        put_short_str(
            &mut bytes,
            "p3_recursion_ordered_three_replica_fold_batch_stark",
        )?;
        bytes.push(PLONKY3_FRI_LOG_BLOWUP_V2);
        bytes.push(PLONKY3_FRI_LOG_FINAL_POLY_LEN_V2);
        bytes.push(PLONKY3_FRI_MAX_LOG_ARITY_V2);
        bytes.extend_from_slice(&PLONKY3_FRI_NUM_QUERIES_V2.to_le_bytes());
        bytes.push(PLONKY3_FRI_COMMIT_POW_BITS_V2);
        bytes.push(PLONKY3_FRI_QUERY_POW_BITS_V2);
        bytes.push(PLONKY3_CHALLENGE_EXTENSION_DEGREE_V2);
        bytes.push(PLONKY3_TRACE_EXTENSION_DEGREE_V2);
        bytes.push(PLONKY3_FRI_REPLICA_COUNT_V2);
        bytes.push(aggregation_generation);
        bytes.push(PLONKY3_PREDICATE_PACKING_GENERATION_V2);
        bytes.extend_from_slice(
            &u16::try_from(ROOT_STATEMENT_FIELDS_V2)
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&ACTIVE_VERIFIER_BUNDLE_DIGEST_V2);
        bytes.extend_from_slice(&PLONKY3_STRUCTURAL_ITEMS_PER_CHUNK_V2.to_le_bytes());
        bytes.extend_from_slice(&PLONKY3_HASH_ITEMS_PER_CHUNK_V2.to_le_bytes());
        bytes.extend_from_slice(&PLONKY3_SOURCE_ITEMS_PER_CHUNK_V2.to_le_bytes());
        bytes.extend_from_slice(&PLONKY3_BASE_RECURSION_DEPTH_V2.to_le_bytes());
        bytes.extend_from_slice(&PLONKY3_LOGICAL_LEAF_COUNT_V2.to_le_bytes());
        bytes.extend_from_slice(&PLONKY3_LOGICAL_NODE_COUNT_V2.to_le_bytes());
        bytes.extend_from_slice(
            &u16::try_from(PLONKY3_MMCS_DIGEST_ELEMS_V2)
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes(),
        );
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
        bytes.extend_from_slice(
            &u32::try_from(PLONKY3_TARGET_BYTES_V2)
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(
            &u32::try_from(PLONKY3_PUBLISH_BYTES_V2)
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(
            &u32::try_from(RECURSIVE_INGRESS_BYTES_V2)
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
    recursive_proof_binding: LocalRecursiveProofBindingV2,
}

impl Drop for LocalVerificationMaterialV2 {
    fn drop(&mut self) {
        self.event_vector.zeroize();
    }
}

/// Non-secret dimensions emitted by the isolated Plan-07 resource worker.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Plonky3TraceDimensionsV2 {
    chunk_count: usize,
    predicate_words: usize,
    event_vector_bytes: usize,
    circuit_witnesses: u32,
    circuit_operations: usize,
    private_inputs: usize,
    witness_rows: usize,
    constant_rows: usize,
    public_rows: usize,
    alu_rows: usize,
    non_primitive_tables: usize,
    non_primitive_rows: usize,
    max_chunk_witnesses: u32,
    max_chunk_operations: usize,
    max_chunk_alu_rows: usize,
    max_chunk_npo_rows: usize,
}

impl Plonky3TraceDimensionsV2 {
    fn empty(predicate_words: usize, event_vector_bytes: usize) -> Self {
        Self {
            chunk_count: 0,
            predicate_words,
            event_vector_bytes,
            circuit_witnesses: 0,
            circuit_operations: 0,
            private_inputs: 0,
            witness_rows: 0,
            constant_rows: 0,
            public_rows: 0,
            alu_rows: 0,
            non_primitive_tables: 0,
            non_primitive_rows: 0,
            max_chunk_witnesses: 0,
            max_chunk_operations: 0,
            max_chunk_alu_rows: 0,
            max_chunk_npo_rows: 0,
        }
    }

    fn add_chunk(&mut self, chunk: Self) -> Result<(), CheckpointError> {
        if self.predicate_words != chunk.predicate_words
            || self.event_vector_bytes != chunk.event_vector_bytes
            || chunk.chunk_count != 1
        {
            return Err(CheckpointError::Invariant);
        }
        self.chunk_count = self
            .chunk_count
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        self.circuit_witnesses = self
            .circuit_witnesses
            .checked_add(chunk.circuit_witnesses)
            .ok_or(CheckpointError::Overflow)?;
        self.circuit_operations = self
            .circuit_operations
            .checked_add(chunk.circuit_operations)
            .ok_or(CheckpointError::Overflow)?;
        self.private_inputs = self
            .private_inputs
            .checked_add(chunk.private_inputs)
            .ok_or(CheckpointError::Overflow)?;
        self.witness_rows = self
            .witness_rows
            .checked_add(chunk.witness_rows)
            .ok_or(CheckpointError::Overflow)?;
        self.constant_rows = self
            .constant_rows
            .checked_add(chunk.constant_rows)
            .ok_or(CheckpointError::Overflow)?;
        self.public_rows = self
            .public_rows
            .checked_add(chunk.public_rows)
            .ok_or(CheckpointError::Overflow)?;
        self.alu_rows = self
            .alu_rows
            .checked_add(chunk.alu_rows)
            .ok_or(CheckpointError::Overflow)?;
        self.non_primitive_tables = self
            .non_primitive_tables
            .checked_add(chunk.non_primitive_tables)
            .ok_or(CheckpointError::Overflow)?;
        self.non_primitive_rows = self
            .non_primitive_rows
            .checked_add(chunk.non_primitive_rows)
            .ok_or(CheckpointError::Overflow)?;
        self.max_chunk_witnesses = self.max_chunk_witnesses.max(chunk.circuit_witnesses);
        self.max_chunk_operations = self.max_chunk_operations.max(chunk.circuit_operations);
        self.max_chunk_alu_rows = self.max_chunk_alu_rows.max(chunk.alu_rows);
        self.max_chunk_npo_rows = self.max_chunk_npo_rows.max(chunk.non_primitive_rows);
        Ok(())
    }

    #[must_use]
    pub const fn chunk_count(&self) -> usize {
        self.chunk_count
    }

    #[must_use]
    pub const fn predicate_words(&self) -> usize {
        self.predicate_words
    }

    #[must_use]
    pub const fn event_vector_bytes(&self) -> usize {
        self.event_vector_bytes
    }

    #[must_use]
    pub const fn circuit_witnesses(&self) -> u32 {
        self.circuit_witnesses
    }

    #[must_use]
    pub const fn circuit_operations(&self) -> usize {
        self.circuit_operations
    }

    #[must_use]
    pub const fn private_inputs(&self) -> usize {
        self.private_inputs
    }

    #[must_use]
    pub const fn witness_rows(&self) -> usize {
        self.witness_rows
    }

    #[must_use]
    pub const fn constant_rows(&self) -> usize {
        self.constant_rows
    }

    #[must_use]
    pub const fn public_rows(&self) -> usize {
        self.public_rows
    }

    #[must_use]
    pub const fn alu_rows(&self) -> usize {
        self.alu_rows
    }

    #[must_use]
    pub const fn non_primitive_tables(&self) -> usize {
        self.non_primitive_tables
    }

    #[must_use]
    pub const fn non_primitive_rows(&self) -> usize {
        self.non_primitive_rows
    }

    #[must_use]
    pub const fn max_chunk_witnesses(&self) -> u32 {
        self.max_chunk_witnesses
    }

    #[must_use]
    pub const fn max_chunk_operations(&self) -> usize {
        self.max_chunk_operations
    }

    #[must_use]
    pub const fn max_chunk_alu_rows(&self) -> usize {
        self.max_chunk_alu_rows
    }

    #[must_use]
    pub const fn max_chunk_npo_rows(&self) -> usize {
        self.max_chunk_npo_rows
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
    trace_dimensions: Option<Plonky3TraceDimensionsV2>,
}

/// Deterministic publication-size classification for a verified proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plonky3ProofSizeStatusV2 {
    /// The complete canonical envelope meets the preferred production target.
    WithinTarget,
    /// The envelope is publishable but requires degraded-size evidence.
    TargetMissed,
}

impl Plonky3ProofSizeStatusV2 {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::WithinTarget => "within_target",
            Self::TargetMissed => "target_missed",
        }
    }
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
            .field("trace_dimensions", &self.trace_dimensions)
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
    pub const fn parameter_digest(&self) -> [u8; 32] {
        self.parameter_digest
    }

    #[must_use]
    pub const fn air_binding_digest(&self) -> [u8; 32] {
        self.air_binding_digest
    }

    #[must_use]
    pub const fn trace_dimensions(&self) -> Option<Plonky3TraceDimensionsV2> {
        self.trace_dimensions
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    #[must_use]
    pub fn size_status(&self) -> Plonky3ProofSizeStatusV2 {
        if self.canonical_bytes.len() <= PLONKY3_TARGET_BYTES_V2 {
            Plonky3ProofSizeStatusV2::WithinTarget
        } else {
            Plonky3ProofSizeStatusV2::TargetMissed
        }
    }

    /// Strict local decoder used only by the base verifier and mutation tests.
    pub fn decode_local(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() > RECURSIVE_INGRESS_BYTES_V2 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::ProofBytesTooLarge,
            ));
        }
        if bytes.len() > PLONKY3_PUBLISH_BYTES_V2 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::ProofSizeBudgetExceeded,
            ));
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
        if proof_len == 0 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3ProofMalformed,
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
        let root_envelope = decode_recursive_roots(&proof_bytes)?;
        if root_binding_digest(&root_envelope)? != air_binding_digest {
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
            trace_dimensions: None,
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
        decoded.trace_dimensions = source.trace_dimensions;
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
        Self::prove_material(material)
    }

    fn prove_material(
        material: TransitionMaterialV2,
    ) -> Result<Plonky3BaseProofV2, CheckpointError> {
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
        let predicate_word_count = words.len();
        let mut trace_dimensions =
            Plonky3TraceDimensionsV2::empty(predicate_word_count, material.event_vector.len());
        let leaf_manifest_digest = leaf_manifest_digest(&material.event_vector)?;
        let replica_chunks = (0..PLONKY3_FRI_REPLICA_COUNT_V2)
            .map(|replica| air_chunks(&material.event_vector, replica))
            .collect::<Result<Vec<_>, CheckpointError>>()?;
        let root_statement = RootStatementAuthorityV2::new(
            material.statement.digest(),
            leaf_manifest_digest,
            parameters.digest,
            security.digest(),
            &words,
            &material.event_vector,
            &replica_chunks,
        )?;

        // Base AIR proving and recursive aggregation have independent memory
        // high-water marks. Create every missing bounded leaf first so an
        // aggregation-preparation cache never overlaps a base prover
        // trace/circuit lifetime. Existing files are not trusted here: the
        // recursive-tree pass canonical-decodes and actual-verifies every leaf
        // exactly once before consuming it. Invalid cache bytes therefore fail
        // closed without paying for the same valid proof twice.
        let leaf_pool = build_bounded_prover_pool("base-materialize")?;
        let materialization_result = replica_chunks.iter().try_for_each(|chunks| {
            ensure_replica_chunk_cache(
                &leaf_pool,
                &words,
                &material.event_vector,
                chunks,
                &root_statement,
            )
        });
        drop(leaf_pool);
        trim_prover_heap();
        materialization_result?;

        let mut replica_chunks = replica_chunks.into_iter();
        let first_chunks = replica_chunks.next().ok_or(CheckpointError::Canonical)?;
        let mut root = prove_replica_tree(
            &words,
            &material.event_vector,
            first_chunks,
            &root_statement,
            &mut trace_dimensions,
        )?;
        for (relation, chunks) in [
            AggregationRelationV2::FirstReplicaFold,
            AggregationRelationV2::FinalReplicaFold,
        ]
        .into_iter()
        .zip(replica_chunks)
        {
            let next = prove_replica_tree(
                &words,
                &material.event_vector,
                chunks,
                &root_statement,
                &mut trace_dimensions,
            )?;
            root = fold_replica_roots(root, next, relation)?;
        }
        if root.replica != PLONKY3_FINAL_REPLICA_FOLD_ORDINAL_V2 {
            return Err(CheckpointError::Canonical);
        }
        drop(words);
        let root_envelope = RecursiveRootEnvelopeV2 {
            leaf_manifest_digest,
            root,
        };
        let air_binding_digest = root_binding_digest(&root_envelope)?;
        let recursive_proof_binding = LocalRecursiveProofBindingV2 {
            root: root_proof_binding(&root_envelope)?,
        };
        let proof_bytes = encode_recursive_roots(&root_envelope)?;
        drop(root_envelope);
        if proof_bytes.is_empty() || proof_bytes.len() > PLONKY3_PUBLISH_BYTES_V2 {
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
                recursive_proof_binding,
            }),
            trace_dimensions: Some(trace_dimensions),
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
        let root_envelope = decode_recursive_roots(&proof.proof_bytes)?;
        let expected_leaf_manifest = leaf_manifest_digest(&expected_material.event_vector)?;
        let replica_chunks = (0..PLONKY3_FRI_REPLICA_COUNT_V2)
            .map(|replica| air_chunks(&expected_material.event_vector, replica))
            .collect::<Result<Vec<_>, CheckpointError>>()?;
        let leaf_total = replica_chunks
            .first()
            .map(Vec::len)
            .ok_or(CheckpointError::Canonical)
            .and_then(|count| u16::try_from(count).map_err(|_| CheckpointError::Limit))?;
        if root_envelope.root.leaf_count != leaf_total {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        let root_statement = RootStatementAuthorityV2::new(
            expected_material.statement.digest(),
            expected_leaf_manifest,
            parameters.digest,
            security.digest(),
            &words,
            &expected_material.event_vector,
            &replica_chunks,
        )?;
        if proof.air_binding_digest != root_binding_digest(&root_envelope)?
            || root_envelope.leaf_manifest_digest != expected_leaf_manifest
            || root_proof_binding(&root_envelope)? != material.recursive_proof_binding.root
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        verify_aggregation_proof(
            &root_envelope.root.proof,
            &root_statement.final_root()?,
            true,
        )?;
        drop(root_envelope);
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct RootStatementAuthorityV2 {
    statement_digest: [u8; 32],
    leaf_manifest_digest: [u8; 32],
    parameter_digest: [u8; 32],
    security_digest: [u8; 32],
    leaf_total: u16,
    leaf_commitments: Vec<Vec<[KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2]>>,
}

impl RootStatementAuthorityV2 {
    #[allow(clippy::too_many_arguments)]
    fn new(
        statement_digest: [u8; 32],
        leaf_manifest_digest: [u8; 32],
        parameter_digest: [u8; 32],
        security_digest: [u8; 32],
        words: &[u16],
        event_vector: &[u8],
        replica_chunks: &[Vec<AirChunkV2>],
    ) -> Result<Self, CheckpointError> {
        let leaf_total = replica_chunks
            .first()
            .map(Vec::len)
            .ok_or(CheckpointError::Canonical)
            .and_then(|count| u16::try_from(count).map_err(|_| CheckpointError::Limit))?;
        if leaf_total == 0
            || replica_chunks.len() != usize::from(PLONKY3_FRI_REPLICA_COUNT_V2)
            || replica_chunks
                .iter()
                .any(|chunks| chunks.len() != usize::from(leaf_total))
        {
            return Err(CheckpointError::Invariant);
        }
        let leaf_commitments = replica_chunks
            .iter()
            .enumerate()
            .map(|(replica, chunks)| {
                chunks
                    .iter()
                    .copied()
                    .map(|chunk| {
                        if usize::from(chunk.replica) != replica {
                            return Err(CheckpointError::Canonical);
                        }
                        chunk_commitment(words, Some(event_vector), chunk)
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            statement_digest,
            leaf_manifest_digest,
            parameter_digest,
            security_digest,
            leaf_total,
            leaf_commitments,
        })
    }

    fn leaf(&self, replica: u8, start: u16) -> Result<RootStatementV2, CheckpointError> {
        let commitment = self
            .leaf_commitments
            .get(usize::from(replica))
            .and_then(|commitments| commitments.get(usize::from(start)))
            .copied()
            .ok_or(CheckpointError::Canonical)?;
        RootStatementV2::leaf(
            self.statement_digest,
            self.leaf_manifest_digest,
            self.parameter_digest,
            self.security_digest,
            ACTIVE_VERIFIER_BUNDLE_DIGEST_V2,
            commitment,
            replica,
            start,
            self.leaf_total,
        )
        .map_err(|_| CheckpointError::Canonical)
    }

    fn root(&self, replica: u8) -> Result<RootStatementV2, CheckpointError> {
        let commitments = self
            .leaf_commitments
            .get(usize::from(replica))
            .ok_or(CheckpointError::Canonical)?;
        let commitment = aggregate_commitments(commitments)?;
        Ok(self.leaf(replica, 0)?.root(commitment))
    }

    fn final_root(&self) -> Result<RootStatementV2, CheckpointError> {
        let replica_zero = self.root(0)?;
        let replica_one = self.root(1)?;
        let replica_two = self.root(2)?;
        let first_commitment = poseidon_pair_hash_for_domain(
            replica_zero.commitment(),
            replica_one.commitment(),
            RootCommitmentDomainV2::FirstReplicaFold,
        );
        let final_commitment = poseidon_pair_hash_for_domain(
            first_commitment,
            replica_two.commitment(),
            RootCommitmentDomainV2::FinalReplicaFold,
        );
        replica_zero
            .replica_fold_root(final_commitment, PLONKY3_FINAL_REPLICA_FOLD_ORDINAL_V2)
            .map_err(|_| CheckpointError::Canonical)
    }
}

#[cfg(test)]
fn root_statement_fixture(
    words: &[u16],
    event_vector: Option<&[u8]>,
    chunk: AirChunkV2,
) -> Result<RootStatementV2, CheckpointError> {
    RootStatementV2::leaf(
        [1; 32],
        [2; 32],
        [3; 32],
        [4; 32],
        [5; 32],
        chunk_commitment(words, event_vector, chunk)?,
        chunk.replica,
        0,
        1,
    )
    .map_err(|_| CheckpointError::Canonical)
}

#[cfg(test)]
fn prove_small_batch(
    words: &[u16],
    root_statement: &RootStatementV2,
) -> Result<BatchStarkProof<Plonky3StarkConfigV2>, CheckpointError> {
    let prepared = prepare_circuit(
        words,
        None,
        AirChunkV2::singleton(AirDomainV2::Full),
        root_statement,
    )?;
    let mut runner = prepared.circuit.runner();
    runner
        .set_private_inputs(&prepared.private_inputs)
        .map_err(|_| CheckpointError::BackendVerificationFailed)?;
    let traces = runner
        .run()
        .map_err(|_| CheckpointError::BackendVerificationFailed)?;
    let mut prover =
        BatchStarkProver::new(prepared.config).with_table_packing(prepared.table_packing);
    register_canonical_recursive_tables(&mut prover);
    let proof = prover
        .prove_all_tables(&traces, &prepared.data)
        .map_err(|_| CheckpointError::BackendVerificationFailed)?;
    prover
        .verify_all_tables::<Plonky3TraceFieldV2>(&proof)
        .map_err(|_| CheckpointError::BackendVerificationFailed)?;
    Ok(proof)
}

#[cfg(test)]
fn aggregation_common_for_test(
    circuit: &Circuit<Plonky3ChallengeV2>,
) -> Result<[u8; 32], CheckpointError> {
    let config = hardened_koala_bear_config();
    let preprocessors = canonical_recursive_preprocessors();
    let air_builders = canonical_recursive_air_builders();
    let packing = aggregation_table_packing();
    let (airs_degrees, primitive_columns, non_primitive_columns) =
        get_airs_and_degrees_with_prep::<Plonky3StarkConfigV2, _, 4>(
            circuit,
            &packing,
            &preprocessors,
            &air_builders,
            ConstraintProfile::Standard,
        )
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 aggregation common derivation failed: {error}"
            ))
        })?;
    let (airs, degrees): (Vec<_>, Vec<_>) = airs_degrees.into_iter().unzip();
    let ext_degrees = degrees
        .iter()
        .map(|&degree| degree + config.is_zk())
        .collect::<Vec<_>>();
    let prover_data = ProverData::from_airs_and_degrees(&config, &airs, &ext_degrees);
    let data = CircuitProverData::new(prover_data, primitive_columns, non_primitive_columns);
    common_binding_digest(data.common_data())
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

fn air_chunks(event_vector: &[u8], replica: u8) -> Result<Vec<AirChunkV2>, CheckpointError> {
    if replica >= PLONKY3_FRI_REPLICA_COUNT_V2 {
        return Err(CheckpointError::Canonical);
    }
    if event_vector.len() < 16 || event_vector[..8] != PLONKY3_EVENT_VECTOR_MAGIC_V2 {
        return Err(CheckpointError::Canonical);
    }
    let profile = RecursiveCircuitProfileV2::authority_pinned();
    let mut cursor = 16_usize;
    let mut source_count = 0_usize;
    let mut hash_block_count = 0_usize;
    let mut jmt_opcodes = Vec::new();
    while cursor < event_vector.len() {
        let event_len = usize::try_from(take_u32(event_vector, &mut cursor)?)
            .map_err(|_| CheckpointError::Limit)?;
        let event = RecursiveTraceEventV2::decode_canonical(
            take_slice(event_vector, &mut cursor, event_len)?,
            &profile,
        )?;
        if event.opcode().is_source_record() {
            source_count = source_count
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
        }
        if event.opcode() == RecursiveTraceOpcodeV2::ShaBlock {
            hash_block_count = hash_block_count
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
        }
        if event.opcode() == RecursiveTraceOpcodeV2::JmtMicroOp {
            jmt_opcodes.push(
                event
                    .payload()
                    .get(1)
                    .copied()
                    .ok_or(CheckpointError::Invariant)?,
            );
        }
    }
    if cursor != event_vector.len() || source_count == 0 || hash_block_count == 0 {
        return Err(CheckpointError::Canonical);
    }
    let structural_count =
        bounded_chunk_count(source_count, PLONKY3_STRUCTURAL_ITEMS_PER_CHUNK_V2)?;
    let hash_count = bounded_chunk_count(hash_block_count, PLONKY3_HASH_ITEMS_PER_CHUNK_V2)?;
    let source_chunk_count = bounded_chunk_count(source_count, PLONKY3_SOURCE_ITEMS_PER_CHUNK_V2)?;
    let transition_count = transition_chunk_count(jmt_update_group_ranges(&jmt_opcodes)?.len())?;
    let mut chunks = Vec::with_capacity(
        usize::from(structural_count)
            .checked_add(usize::from(hash_count))
            .and_then(|count| count.checked_add(usize::from(source_chunk_count)))
            .and_then(|count| {
                count.checked_add(
                    UniquenessListHashJobV2::ALL.len()
                        + UniquenessTranscriptHashJobV2::ALL.len()
                        + usize::from(transition_count)
                        + 1,
                )
            })
            .ok_or(CheckpointError::Overflow)?,
    );
    for index in 0..structural_count {
        chunks.push(AirChunkV2::replicated(
            AirDomainV2::Structural,
            index,
            structural_count,
            replica,
        ));
    }
    for index in 0..hash_count {
        chunks.push(AirChunkV2::replicated(
            AirDomainV2::Hash,
            index,
            hash_count,
            replica,
        ));
    }
    for index in 0..source_chunk_count {
        chunks.push(AirChunkV2::replicated(
            AirDomainV2::Source,
            index,
            source_chunk_count,
            replica,
        ));
    }
    let list_count =
        u16::try_from(UniquenessListHashJobV2::ALL.len()).map_err(|_| CheckpointError::Limit)?;
    for index in 0..list_count {
        chunks.push(AirChunkV2::replicated(
            AirDomainV2::Lists,
            index,
            list_count,
            replica,
        ));
    }
    let uniqueness_count = u16::try_from(UniquenessTranscriptHashJobV2::ALL.len())
        .map_err(|_| CheckpointError::Limit)?;
    for index in 0..uniqueness_count {
        chunks.push(AirChunkV2::replicated(
            AirDomainV2::Uniqueness,
            index,
            uniqueness_count,
            replica,
        ));
    }
    chunks.push(AirChunkV2::replicated(AirDomainV2::Trace, 0, 1, replica));
    for index in 0..transition_count {
        chunks.push(AirChunkV2::replicated(
            AirDomainV2::Transition,
            index,
            transition_count,
            replica,
        ));
    }
    if chunks.len()
        > usize::try_from(PLONKY3_LOGICAL_LEAF_COUNT_V2).map_err(|_| CheckpointError::Limit)?
    {
        return Err(CheckpointError::Limit);
    }
    Ok(chunks)
}

struct PreparedCircuitV2 {
    circuit: Circuit<Plonky3TraceFieldV2>,
    private_inputs: Vec<Plonky3TraceFieldV2>,
    config: Plonky3StarkConfigV2,
    data: CircuitProverData<Plonky3StarkConfigV2>,
    table_packing: TablePacking,
}

struct PreparedRunnerV2 {
    circuit: Circuit<Plonky3TraceFieldV2>,
    private_inputs: Vec<Plonky3TraceFieldV2>,
}

struct PreparedBuilderV2 {
    builder: CircuitBuilder<Plonky3TraceFieldV2>,
    private_inputs: Vec<Plonky3TraceFieldV2>,
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

fn domain_uses_statement_bits(domain: AirDomainV2) -> bool {
    domain.includes(AirDomainV2::Uniqueness)
        || domain.includes(AirDomainV2::Trace)
        || domain.includes(AirDomainV2::Transition)
}

fn domain_uses_event_bits(
    chunk: AirChunkV2,
    event: &RecursiveTraceEventV2,
) -> Result<bool, CheckpointError> {
    let domain = chunk.domain;
    #[cfg(test)]
    if matches!(domain, AirDomainV2::Full) {
        return Ok(true);
    }
    if domain.includes(AirDomainV2::Structural) {
        return Ok(event.opcode().is_source_record());
    }
    if domain.includes(AirDomainV2::Hash) {
        return Ok(event.opcode() == RecursiveTraceOpcodeV2::ShaBlock);
    }
    if domain.includes(AirDomainV2::Transition) {
        return Ok(event.opcode().is_source_record());
    }
    let hash_schema = if matches!(
        event.opcode(),
        RecursiveTraceOpcodeV2::BeginHash
            | RecursiveTraceOpcodeV2::ShaBlock
            | RecursiveTraceOpcodeV2::EndHash
    ) {
        Some(decode_hash_control(event)?.schema)
    } else {
        None
    };
    if domain.includes(AirDomainV2::Source) {
        if event.opcode().is_source_record() {
            return Ok(ordinal_in_bounded_chunk(
                event.ordinal(),
                chunk,
                PLONKY3_SOURCE_ITEMS_PER_CHUNK_V2,
            ));
        }
        if event.opcode() == RecursiveTraceOpcodeV2::SourceMemoryWrite {
            return Ok(ordinal_in_bounded_chunk(
                decode_source_memory_write_control(event)?.source_ordinal,
                chunk,
                PLONKY3_SOURCE_ITEMS_PER_CHUNK_V2,
            ));
        }
        if event.opcode() == RecursiveTraceOpcodeV2::TraceChunk {
            return Ok(ordinal_in_bounded_chunk(
                decode_trace_chunk_control(event)?.source_ordinal,
                chunk,
                PLONKY3_SOURCE_ITEMS_PER_CHUNK_V2,
            ));
        }
        if hash_schema == Some(HashControlSchemaV2::SourceRecord) {
            return Ok(decode_hash_control(event)?
                .source
                .map(|binding| {
                    ordinal_in_bounded_chunk(
                        binding.ordinal,
                        chunk,
                        PLONKY3_SOURCE_ITEMS_PER_CHUNK_V2,
                    )
                })
                .unwrap_or(false));
        }
        return Ok(false);
    }
    if domain.includes(AirDomainV2::Lists) {
        let job = *UniquenessListHashJobV2::ALL
            .get(usize::from(chunk.index))
            .ok_or(CheckpointError::Canonical)?;
        if event.opcode() == RecursiveTraceOpcodeV2::UniquenessPrecommit {
            return Ok(true);
        }
        if event.opcode() == RecursiveTraceOpcodeV2::UniquenessSorted {
            let (pass, set, list, _) = decode_uniqueness_sorted_row(event.payload())?;
            return Ok(pass == UniquenessPassV2::Commit && (set, list) == job.row());
        }
        if hash_schema == Some(HashControlSchemaV2::UniquenessList) {
            return Ok(decode_hash_control(event)?
                .uniqueness_list
                .map(|binding| binding.job)
                == Some(job));
        }
        return Ok(false);
    }
    if domain.includes(AirDomainV2::Uniqueness) {
        let job = *UniquenessTranscriptHashJobV2::ALL
            .get(usize::from(chunk.index))
            .ok_or(CheckpointError::Canonical)?;
        if matches!(
            event.opcode(),
            RecursiveTraceOpcodeV2::UniquenessPrecommit
                | RecursiveTraceOpcodeV2::UniquenessChallenge
        ) {
            return Ok(true);
        }
        if hash_schema == Some(HashControlSchemaV2::UniquenessTranscript) {
            return Ok(decode_hash_control(event)?
                .uniqueness_transcript
                .map(|binding| binding.job)
                == Some(job));
        }
        return Ok(false);
    }
    if domain.includes(AirDomainV2::Trace) {
        return Ok(event.opcode().is_source_record()
            || hash_schema == Some(HashControlSchemaV2::TracePrecommit));
    }
    Ok(false)
}

/// Selects only the canonical bytes consumed by one bounded AIR domain.
///
/// A selected byte promotes its whole eight-u16 Poseidon2 rate block so every
/// coefficient is range-constrained before that block is committed.
fn predicate_bit_mask(
    words: &[u16],
    event_vector: Option<&[u8]>,
    chunk: AirChunkV2,
) -> Result<Vec<bool>, CheckpointError> {
    chunk.validate()?;
    let domain = chunk.domain;
    let byte_len = words
        .len()
        .checked_mul(2)
        .ok_or(CheckpointError::Overflow)?;
    let mut mask = vec![false; byte_len];
    #[cfg(test)]
    if matches!(domain, AirDomainV2::Full) {
        mask.fill(true);
        return Ok(mask);
    }
    let Some(event_vector) = event_vector else {
        return Ok(mask);
    };
    let statement_start = PLONKY3_PREDICATE_VECTOR_LABEL_V2
        .len()
        .checked_add(8)
        .ok_or(CheckpointError::Overflow)?;
    if domain_uses_statement_bits(domain) {
        mask.get_mut(statement_start..statement_start + PLONKY3_BASE_STATEMENT_BYTES_V2)
            .ok_or(CheckpointError::Invariant)?
            .fill(true);
    }
    let event_vector_offset = statement_start
        .checked_add(PLONKY3_BASE_STATEMENT_BYTES_V2)
        .and_then(|offset| offset.checked_add(8))
        .ok_or(CheckpointError::Overflow)?;
    if event_vector.len() < 16 || event_vector[..8] != PLONKY3_EVENT_VECTOR_MAGIC_V2 {
        return Err(CheckpointError::Canonical);
    }
    let profile = RecursiveCircuitProfileV2::authority_pinned();
    let mut cursor = 16_usize;
    let chunk_items = match domain {
        AirDomainV2::Structural => Some(PLONKY3_STRUCTURAL_ITEMS_PER_CHUNK_V2),
        AirDomainV2::Hash => Some(PLONKY3_HASH_ITEMS_PER_CHUNK_V2),
        _ => None,
    };
    let mut candidate_index = 0_u16;
    while cursor < event_vector.len() {
        let event_len = usize::try_from(take_u32(event_vector, &mut cursor)?)
            .map_err(|_| CheckpointError::Limit)?;
        let event_start = cursor;
        let event_bytes = take_slice(event_vector, &mut cursor, event_len)?;
        let event = RecursiveTraceEventV2::decode_canonical(event_bytes, &profile)?;
        let domain_uses_event = domain_uses_event_bits(chunk, &event)?;
        let selected = if domain_uses_event {
            if let Some(items_per_chunk) = chunk_items {
                let selected = candidate_index / items_per_chunk == chunk.index;
                candidate_index = candidate_index
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                selected
            } else {
                true
            }
        } else {
            false
        };
        if selected {
            let start = event_vector_offset
                .checked_add(event_start)
                .ok_or(CheckpointError::Overflow)?;
            mask.get_mut(start..start + event_len)
                .ok_or(CheckpointError::Invariant)?
                .fill(true);
        }
    }
    if cursor != event_vector.len() {
        return Err(CheckpointError::Canonical);
    }
    if let Some(items_per_chunk) = chunk_items {
        if chunk.count != bounded_chunk_count(usize::from(candidate_index), items_per_chunk)? {
            return Err(CheckpointError::Canonical);
        }
    }
    for rate_block in mask.chunks_mut(16) {
        if rate_block.iter().any(|selected| *selected) {
            rate_block.fill(true);
        }
    }
    Ok(mask)
}

fn domain_commitment_header(
    predicate_word_count: usize,
    bit_mask: &[bool],
    chunk: AirChunkV2,
) -> Result<Vec<u16>, CheckpointError> {
    let selected_word_count = bit_mask
        .chunks_exact(2)
        .filter(|word| word.iter().all(|selected| *selected))
        .count();
    let mask_bytes: Vec<u8> = bit_mask
        .chunks(8)
        .map(|bits| {
            bits.iter().enumerate().fold(0_u8, |byte, (bit, selected)| {
                byte | (u8::from(*selected) << bit)
            })
        })
        .collect();
    let mask_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.domain-mask.v2",
        "selected_predicate_bytes",
        &[&mask_bytes],
    );
    let mut header = vec![
        0x5a30,
        2,
        u16::from(chunk.domain.tag()),
        u16::from(chunk.replica),
        chunk.index,
        chunk.count,
        0,
    ];
    header.extend(
        u64::try_from(predicate_word_count)
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes()
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]])),
    );
    header.extend(
        u64::try_from(selected_word_count)
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes()
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]])),
    );
    header.extend(
        mask_digest
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]])),
    );
    while !header.len().is_multiple_of(8) {
        header.push(0);
    }
    Ok(header)
}

fn chunk_commitment(
    words: &[u16],
    event_vector: Option<&[u8]>,
    chunk: AirChunkV2,
) -> Result<[KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2], CheckpointError> {
    let bit_mask = predicate_bit_mask(words, event_vector, chunk)?;
    let commitment_header = domain_commitment_header(words.len(), &bit_mask, chunk)?;
    let selected_words = words
        .iter()
        .copied()
        .zip(bit_mask.chunks_exact(2))
        .filter_map(|(word, bits)| bits.iter().all(|selected| *selected).then_some(word))
        .collect::<Vec<_>>();
    chunk_commitment_from_parts(&commitment_header, &selected_words)
}

fn chunk_commitment_from_parts(
    commitment_header: &[u16],
    selected_words: &[u16],
) -> Result<[KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2], CheckpointError> {
    if selected_words.is_empty()
        || !commitment_header.len().is_multiple_of(8)
        || !selected_words.len().is_multiple_of(8)
    {
        return Err(CheckpointError::Invariant);
    }
    let mut committed_words =
        Vec::with_capacity(commitment_header.len().saturating_add(selected_words.len()));
    committed_words.extend_from_slice(commitment_header);
    committed_words.extend_from_slice(selected_words);
    Ok(poseidon_vector_hash(&committed_words))
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
    source_range: Option<Range<u16>>,
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<usize, CheckpointError> {
    let mut constrained = 0_usize;
    for view in views {
        if !view.event.opcode().is_source_record()
            || source_range
                .as_ref()
                .map(|range| {
                    view.event.ordinal() < u64::from(range.start)
                        || view.event.ordinal() >= u64::from(range.end)
                })
                .unwrap_or(false)
        {
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
    selected_job: Option<UniquenessListHashJobV2>,
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
        if selected_job
            .map(|selected| selected != job)
            .unwrap_or(false)
        {
            continue;
        }
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
    selected_job: Option<UniquenessTranscriptHashJobV2>,
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
        if selected_job
            .map(|selected| selected != job)
            .unwrap_or(false)
        {
            continue;
        }
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
    block_range: Option<Range<u16>>,
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<usize, CheckpointError> {
    let mut constrained = 0_usize;
    let mut candidate_index = 0_u16;
    for view in views {
        let event = &view.event;
        if event.opcode() != RecursiveTraceOpcodeV2::ShaBlock {
            continue;
        }
        let selected = block_range
            .as_ref()
            .map(|selected| selected.contains(&candidate_index))
            .unwrap_or(true);
        candidate_index = candidate_index
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        if !selected {
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
    selected_sources: Option<Range<u16>>,
    zero: ExprId,
    one: ExprId,
) -> Result<usize, CheckpointError> {
    let mut constrained_sources = 0_usize;
    for (source_index, source) in views.iter().enumerate() {
        if !source.event.opcode().is_source_record() {
            continue;
        }
        if selected_sources
            .as_ref()
            .map(|range| {
                source.event.ordinal() < u64::from(range.start)
                    || source.event.ordinal() >= u64::from(range.end)
            })
            .unwrap_or(false)
        {
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

fn jmt_update_group_ranges(opcodes: &[u8]) -> Result<Vec<Range<usize>>, CheckpointError> {
    let mut ranges = Vec::new();
    let mut group_start = None;
    for (index, opcode) in opcodes.iter().copied().enumerate() {
        match opcode {
            1 => {
                if group_start.replace(index).is_some() {
                    return Err(CheckpointError::Invariant);
                }
            }
            6 => {
                let start = group_start.take().ok_or(CheckpointError::Invariant)?;
                ranges.push(start..index + 1);
            }
            _ if group_start.is_none() => return Err(CheckpointError::Invariant),
            _ => {}
        }
    }
    if group_start.is_some() {
        return Err(CheckpointError::Invariant);
    }
    Ok(ranges)
}

fn transition_chunk_count(jmt_group_count: usize) -> Result<u16, CheckpointError> {
    let jmt_parts = u16::try_from(jmt_group_count.max(1)).map_err(|_| CheckpointError::Limit)?;
    PLONKY3_TRANSITION_JMT_FIRST_PART_V2
        .checked_add(jmt_parts)
        .and_then(|final_part| final_part.checked_add(1))
        .ok_or(CheckpointError::Overflow)
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
    selected_chunk: Option<AirChunkV2>,
    zero: ExprId,
    one: ExprId,
    two: ExprId,
) -> Result<usize, CheckpointError> {
    let jmt_opcodes = views
        .iter()
        .filter(|view| view.event.opcode() == RecursiveTraceOpcodeV2::JmtMicroOp)
        .map(|view| {
            view.event
                .payload()
                .get(1)
                .copied()
                .ok_or(CheckpointError::Invariant)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let expected_chunk_count =
        transition_chunk_count(jmt_update_group_ranges(&jmt_opcodes)?.len())?;
    let final_part = expected_chunk_count
        .checked_sub(1)
        .ok_or(CheckpointError::Invariant)?;
    if selected_chunk.is_some_and(|chunk| {
        chunk.domain != AirDomainV2::Transition
            || chunk.count != expected_chunk_count
            || chunk.index >= expected_chunk_count
    }) {
        return Err(CheckpointError::Canonical);
    }
    let selected_part = selected_chunk.map(|chunk| chunk.index);
    let active = |part: u16| {
        selected_part
            .map(|selected| selected == part)
            .unwrap_or(true)
    };
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
        if active(final_part) {
            constrain_full_trace_statement_counts(
                builder,
                views,
                &sources,
                statement,
                statement_bits,
                zero,
                one,
            )?;
        }

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
        if active(0) {
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
        }

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
                if active(0) {
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
                }
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
                if active(0) {
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
        if active(1) {
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
        }

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
                if active(1) {
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
            if active(1) {
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
            }
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
                    if active(1) {
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
                    }
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
            if active(1) {
                constrain_net_effect_from_rows(builder, net, spent_view, output_view, zero, one)?;
            }
        }

        stage = "net close";
        let close = take_source_view(&sources, &mut cursor, RecursiveTraceOpcodeV2::NetMerge)?;
        let close_payload = close.payload_bits()?;
        if close_payload.len() != NET_MERGE_BYTES_V2
            || decode_net_effect(close.event.payload())?.kind as u8 != 0
        {
            return Err(transition_semantics_error("net close"));
        }
        if active(1) {
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
        }

        stage = "JMT schedule decode";
        let jmt_header =
            take_source_view(&sources, &mut cursor, RecursiveTraceOpcodeV2::JmtUpdate)?;
        if active(PLONKY3_TRANSITION_JMT_FIRST_PART_V2) {
            connect_bytes_to_constants(
                builder,
                &jmt_header.payload_bits()?[..3],
                &jmt_header.event.payload()[..3],
                zero,
                one,
            )?;
        }
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
        let jmt_opcodes = jmt_micro_operations
            .iter()
            .map(|record| {
                record
                    .event
                    .payload()
                    .get(1)
                    .copied()
                    .ok_or(CheckpointError::Invariant)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let jmt_groups = jmt_update_group_ranges(&jmt_opcodes)?;
        // A transition can update several trees at one hierarchy level, so
        // update ordinal is not a tree-role tag. The single canonical decoder
        // above owns role ordering, coordinate uniqueness, and parent/child
        // promotion validation across the complete transcript. These ranges
        // only split whole updates into bounded proof chunks.
        match selected_part {
            None => constrain_jmt_micro_operations(
                builder,
                &jmt_micro_operations,
                pre_definition_root,
                post_definition_root,
                zero,
                one,
                two,
            )?,
            Some(part)
                if part >= PLONKY3_TRANSITION_JMT_FIRST_PART_V2
                    && usize::from(part - PLONKY3_TRANSITION_JMT_FIRST_PART_V2)
                        < jmt_groups.len() =>
            {
                let group = usize::from(part - PLONKY3_TRANSITION_JMT_FIRST_PART_V2);
                let range = jmt_groups.get(group).ok_or(CheckpointError::Invariant)?;
                constrain_jmt_micro_operations(
                    builder,
                    &jmt_micro_operations[range.clone()],
                    pre_definition_root,
                    post_definition_root,
                    zero,
                    one,
                    two,
                )?;
            }
            Some(_) => {}
        }
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
        if active(PLONKY3_TRANSITION_JMT_FIRST_PART_V2) {
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
        }

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
        if active(final_part) {
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
        }

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
            if active(final_part) {
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
        if active(final_part) {
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
        }
        Ok(sources.len())
    })();
    result.map_err(|error| transition_semantics_stage(stage, error))
}

fn add_recursive_npo_shape_constraints(
    builder: &mut CircuitBuilder<Plonky3TraceFieldV2>,
    private_inputs: &mut Vec<Plonky3TraceFieldV2>,
    zero: ExprId,
) -> Result<(), CheckpointError> {
    let expected_state = default_koalabear_poseidon2_32().permute([KoalaBear::ZERO; 32]);
    let (_, outputs) = builder
        .add_poseidon2_perm(&Poseidon2PermCall {
            config: Poseidon2Config::KOALA_BEAR_D4_W32,
            new_start: true,
            merkle_path: false,
            mmcs_bit: None,
            mmcs_bit2: None,
            inputs: vec![Some(zero); 8],
            out_ctl: vec![true; 6],
            return_all_outputs: false,
            mmcs_index_sum: None,
        })
        .map_err(|_| {
            CheckpointError::Backend(
                "Plonky3 recursive W32 shape constraint lowering failed".into(),
            )
        })?;
    for (output, expected) in outputs
        .into_iter()
        .take(6)
        .zip(expected_state.chunks_exact(4))
    {
        let output = output.ok_or_else(|| {
            CheckpointError::Backend("Plonky3 recursive W32 shape constraint output missing".into())
        })?;
        let expected = builder.alloc_const(
            Plonky3TraceFieldV2::new([expected[0], expected[1], expected[2], expected[3]]),
            "recursive_w32_shape_output",
        );
        builder.connect(output, expected);
    }

    let coefficients: Vec<_> = (0..4)
        .map(|_| builder.alloc_private_input("recursive_recompose_shape_coefficient"))
        .collect();
    private_inputs.extend([Plonky3TraceFieldV2::ZERO; 4]);
    for coefficient in &coefficients {
        builder.assert_bool(*coefficient);
    }
    let recomposed = builder
        .recompose_base_coeffs_to_ext::<KoalaBear>(&coefficients)
        .map_err(|_| {
            CheckpointError::Backend(
                "Plonky3 recursive recompose shape constraint lowering failed".into(),
            )
        })?;
    builder.connect(recomposed, zero);
    Ok(())
}

fn prepare_builder(
    words: &[u16],
    event_vector: Option<&[u8]>,
    retain_inputs: bool,
    chunk: AirChunkV2,
    root_statement: &RootStatementV2,
) -> Result<PreparedBuilderV2, CheckpointError> {
    if words.is_empty() || !words.len().is_multiple_of(8) {
        return Err(CheckpointError::Invariant);
    }
    chunk.validate()?;
    let domain = chunk.domain;
    let bit_mask = predicate_bit_mask(words, event_vector, chunk)?;
    let commitment_header = domain_commitment_header(words.len(), &bit_mask, chunk)?;
    let selected_words: Vec<u16> = words
        .iter()
        .copied()
        .zip(bit_mask.chunks_exact(2))
        .filter_map(|(word, bits)| bits.iter().all(|selected| *selected).then_some(word))
        .collect();
    if selected_words.is_empty() || !selected_words.len().is_multiple_of(8) {
        return Err(CheckpointError::Invariant);
    }
    let expected_hash = chunk_commitment_from_parts(&commitment_header, &selected_words)?;
    if root_statement.commitment() != expected_hash {
        return Err(CheckpointError::Canonical);
    }
    let mut builder = CircuitBuilder::<Plonky3TraceFieldV2>::new();
    builder.enable_poseidon2_perm::<KoalaBearD4Width16, _>(
        generate_poseidon2_trace::<Plonky3TraceFieldV2, KoalaBearD4Width16>,
        default_koalabear_poseidon2_16(),
    );
    builder.enable_poseidon2_perm_width_32::<KoalaBearD4Width32, _>(
        generate_poseidon2_trace::<Plonky3TraceFieldV2, KoalaBearD4Width32>,
        default_koalabear_poseidon2_32(),
    );
    builder
        .enable_recompose::<KoalaBear>(generate_recompose_trace::<KoalaBear, Plonky3TraceFieldV2>);
    register_root_statement_npo(&mut builder);
    register_u16_range_npo(&mut builder);
    let mut private_inputs = Vec::new();
    let zero = builder.alloc_const(lift_koala(KoalaBear::ZERO), "zero");
    let one = builder.alloc_const(lift_koala(KoalaBear::ONE), "one");
    let root_statement_targets = root_statement
        .values()
        .iter()
        .map(|&value| {
            private_inputs.push(lift_koala(value));
            let input = builder.alloc_private_input("root_statement");
            builder.mul_add(input, one, zero)
        })
        .collect::<Vec<_>>();
    bind_root_statement_targets(&mut builder, &root_statement_targets)
        .map_err(|_| CheckpointError::Canonical)?;
    let two = builder.alloc_const(lift_koala(KoalaBear::from_u64(2)), "two");
    add_recursive_npo_shape_constraints(&mut builder, &mut private_inputs, zero)?;
    let mut packed_word_exprs =
        Vec::with_capacity(commitment_header.len() / 4 + selected_words.len() / 4);
    for header_limb in commitment_header.chunks_exact(4) {
        packed_word_exprs.push(builder.alloc_const(
            Plonky3TraceFieldV2::new(core::array::from_fn(|coefficient| {
                KoalaBear::from_u64(u64::from(header_limb[coefficient]))
            })),
            "predicate_domain_header",
        ));
    }
    let mut predicate_byte_bits = Vec::with_capacity(words.len() * 2);
    for (limb_index, limb_words) in words.chunks_exact(4).enumerate() {
        let byte_start = limb_index.checked_mul(8).ok_or(CheckpointError::Overflow)?;
        let selected = bit_mask
            .get(byte_start..byte_start + 8)
            .ok_or(CheckpointError::Invariant)?
            .iter()
            .all(|selected| *selected);
        if selected {
            let mut word_coefficients = Vec::with_capacity(4);
            for word in limb_words.iter().copied() {
                let word_input = builder.alloc_private_input("selected_u16_word");
                if retain_inputs {
                    private_inputs.push(lift_koala(KoalaBear::from_u16(word)));
                }
                // Private inputs acquire their canonical WitnessChecks creator
                // role through the first ALU use. The u16 table then reads the
                // derived wire; one identity row per word replaces the former
                // sixteen boolean-expansion rows without introducing a second
                // creator for the same witness.
                let word_expr = builder.mul_add(word_input, one, zero);
                let bits = constrain_u16_bits(&mut builder, word_expr).map_err(|_| {
                    CheckpointError::Backend(
                        "Plonky3 packed u16 range table lowering failed".into(),
                    )
                })?;
                word_coefficients.push(word_expr);
                predicate_byte_bits.push(core::array::from_fn(|bit| bits[bit]));
                predicate_byte_bits.push(core::array::from_fn(|bit| bits[8 + bit]));
            }
            let packed = builder
                .recompose_base_coeffs_to_ext::<KoalaBear>(&word_coefficients)
                .map_err(|_| {
                    CheckpointError::Backend("Plonky3 packed u16 recompose lowering failed".into())
                })?;
            packed_word_exprs.push(packed);
        } else {
            predicate_byte_bits.extend(core::iter::repeat_n([zero; 8], 8));
        }
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
        let hash_block_count = event_views
            .iter()
            .filter(|view| view.event.opcode() == RecursiveTraceOpcodeV2::ShaBlock)
            .count();
        let has_complete_transition = event_views
            .iter()
            .any(|view| view.event.opcode() == RecursiveTraceOpcodeV2::FinalizeBlock);
        let constrained_ids = if domain.includes(AirDomainV2::Structural) {
            #[cfg(test)]
            let source_range = if matches!(domain, AirDomainV2::Full) {
                None
            } else {
                Some(bounded_chunk_range(
                    source_count,
                    chunk,
                    PLONKY3_STRUCTURAL_ITEMS_PER_CHUNK_V2,
                )?)
            };
            #[cfg(not(test))]
            let source_range = Some(bounded_chunk_range(
                source_count,
                chunk,
                PLONKY3_STRUCTURAL_ITEMS_PER_CHUNK_V2,
            )?);
            constrain_structural_source_event_ids(
                &mut builder,
                &event_views,
                source_range,
                zero,
                one,
                two,
            )
            .map_err(|error| air_construction_stage("structural event IDs", error))?
        } else {
            0
        };
        let constrained = if domain.includes(AirDomainV2::Hash) {
            #[cfg(test)]
            let block_range = if matches!(domain, AirDomainV2::Full) {
                None
            } else {
                Some(bounded_chunk_range(
                    hash_block_count,
                    chunk,
                    PLONKY3_HASH_ITEMS_PER_CHUNK_V2,
                )?)
            };
            #[cfg(not(test))]
            let block_range = Some(bounded_chunk_range(
                hash_block_count,
                chunk,
                PLONKY3_HASH_ITEMS_PER_CHUNK_V2,
            )?);
            constrain_sha_control_blocks(&mut builder, &event_views, block_range, zero, one, two)
                .map_err(|error| air_construction_stage("SHA control blocks", error))?
        } else {
            0
        };
        let constrained_sources = if domain.includes(AirDomainV2::Source) {
            #[cfg(test)]
            let source_range = if matches!(domain, AirDomainV2::Full) {
                None
            } else {
                Some(bounded_chunk_range(
                    source_count,
                    chunk,
                    PLONKY3_SOURCE_ITEMS_PER_CHUNK_V2,
                )?)
            };
            #[cfg(not(test))]
            let source_range = Some(bounded_chunk_range(
                source_count,
                chunk,
                PLONKY3_SOURCE_ITEMS_PER_CHUNK_V2,
            )?);
            constrain_source_record_bindings(&mut builder, &event_views, source_range, zero, one)
                .map_err(|error| air_construction_stage("source bindings", error))?
        } else {
            0
        };
        let has_uniqueness_lists = event_views.iter().any(|view| {
            decode_hash_control(&view.event)
                .map(|control| control.schema == HashControlSchemaV2::UniquenessList)
                .unwrap_or(false)
        });
        let constrained_uniqueness_lists = if has_uniqueness_lists
            && domain.includes(AirDomainV2::Lists)
        {
            #[cfg(test)]
            let selected_job = if matches!(domain, AirDomainV2::Full) {
                None
            } else {
                Some(
                    *UniquenessListHashJobV2::ALL
                        .get(usize::from(chunk.index))
                        .ok_or(CheckpointError::Canonical)?,
                )
            };
            #[cfg(not(test))]
            let selected_job = Some(
                *UniquenessListHashJobV2::ALL
                    .get(usize::from(chunk.index))
                    .ok_or(CheckpointError::Canonical)?,
            );
            constrain_uniqueness_list_bindings(&mut builder, &event_views, selected_job, zero, one)
                .map_err(|error| air_construction_stage("uniqueness list bindings", error))?
        } else {
            0
        };
        let has_uniqueness_transcript = event_views.iter().any(|view| {
            decode_hash_control(&view.event)
                .map(|control| control.schema == HashControlSchemaV2::UniquenessTranscript)
                .unwrap_or(false)
        });
        let constrained_uniqueness_transcript =
            if has_uniqueness_transcript && domain.includes(AirDomainV2::Uniqueness) {
                #[cfg(test)]
                let selected_job = if matches!(domain, AirDomainV2::Full) {
                    None
                } else {
                    Some(
                        *UniquenessTranscriptHashJobV2::ALL
                            .get(usize::from(chunk.index))
                            .ok_or(CheckpointError::Canonical)?,
                    )
                };
                #[cfg(not(test))]
                let selected_job = Some(
                    *UniquenessTranscriptHashJobV2::ALL
                        .get(usize::from(chunk.index))
                        .ok_or(CheckpointError::Canonical)?,
                );
                constrain_uniqueness_transcript_bindings(
                    &mut builder,
                    &event_views,
                    statement_bits,
                    selected_job,
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
        let constrained_trace = if has_trace_precommit && domain.includes(AirDomainV2::Trace) {
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
        let constrained_transition =
            if has_complete_transition && domain.includes(AirDomainV2::Transition) {
                #[cfg(test)]
                let selected_chunk = (!matches!(domain, AirDomainV2::Full)).then_some(chunk);
                #[cfg(not(test))]
                let selected_chunk = Some(chunk);
                constrain_frozen_transition_semantics(
                    &mut builder,
                    &event_views,
                    statement,
                    statement_bits,
                    selected_chunk,
                    zero,
                    one,
                    two,
                )
                .map_err(|error| air_construction_stage("frozen transition semantics", error))?
            } else {
                0
            };
        if domain.includes(AirDomainV2::Structural) {
            #[cfg(test)]
            let expected = if matches!(domain, AirDomainV2::Full) {
                source_count
            } else {
                bounded_chunk_range(source_count, chunk, PLONKY3_STRUCTURAL_ITEMS_PER_CHUNK_V2)?
                    .len()
            };
            #[cfg(not(test))]
            let expected =
                bounded_chunk_range(source_count, chunk, PLONKY3_STRUCTURAL_ITEMS_PER_CHUNK_V2)?
                    .len();
            if constrained_ids != expected {
                return Err(CheckpointError::Backend(format!(
                    "Plonky3 AIR structural chunk coverage mismatch: index {}, count {}, actual {constrained_ids}, expected {expected}, sources {source_count}",
                    chunk.index, chunk.count
                )));
            }
        }
        if domain.includes(AirDomainV2::Hash) {
            #[cfg(test)]
            let expected = if matches!(domain, AirDomainV2::Full) {
                hash_block_count
            } else {
                bounded_chunk_range(hash_block_count, chunk, PLONKY3_HASH_ITEMS_PER_CHUNK_V2)?.len()
            };
            #[cfg(not(test))]
            let expected =
                bounded_chunk_range(hash_block_count, chunk, PLONKY3_HASH_ITEMS_PER_CHUNK_V2)?
                    .len();
            if constrained != expected {
                return Err(CheckpointError::Backend(format!(
                    "Plonky3 AIR hash chunk coverage mismatch: index {}, count {}, actual {constrained}, expected {expected}, blocks {hash_block_count}",
                    chunk.index, chunk.count
                )));
            }
        }
        if domain.includes(AirDomainV2::Source) {
            #[cfg(test)]
            let expected = if matches!(domain, AirDomainV2::Full) {
                source_count
            } else {
                bounded_chunk_range(source_count, chunk, PLONKY3_SOURCE_ITEMS_PER_CHUNK_V2)?.len()
            };
            #[cfg(not(test))]
            let expected =
                bounded_chunk_range(source_count, chunk, PLONKY3_SOURCE_ITEMS_PER_CHUNK_V2)?.len();
            if constrained_sources != expected {
                return Err(CheckpointError::Backend(format!(
                    "Plonky3 AIR source chunk coverage mismatch: index {}, count {}, actual {constrained_sources}, expected {expected}, sources {source_count}",
                    chunk.index, chunk.count
                )));
            }
        }
        if has_trace_precommit && domain.includes(AirDomainV2::Trace) && constrained_trace == 0 {
            return Err(CheckpointError::Backend(
                "Plonky3 AIR trace-precommit coverage is empty".into(),
            ));
        }
        if has_uniqueness_lists && domain.includes(AirDomainV2::Lists) {
            #[cfg(test)]
            let expected = if matches!(domain, AirDomainV2::Full) {
                UniquenessListHashJobV2::ALL.len()
            } else {
                1
            };
            #[cfg(not(test))]
            let expected = 1;
            if (expected == 1 && usize::from(chunk.count) != UniquenessListHashJobV2::ALL.len())
                || constrained_uniqueness_lists != expected
            {
                return Err(CheckpointError::Backend(format!(
                    "Plonky3 AIR uniqueness-list chunk coverage mismatch: index {}, count {}, actual {constrained_uniqueness_lists}, expected {expected}",
                    chunk.index, chunk.count
                )));
            }
        }
        if has_uniqueness_transcript && domain.includes(AirDomainV2::Uniqueness) {
            #[cfg(test)]
            let expected = if matches!(domain, AirDomainV2::Full) {
                UniquenessTranscriptHashJobV2::ALL.len()
            } else {
                1
            };
            #[cfg(not(test))]
            let expected = 1;
            if (expected == 1
                && usize::from(chunk.count) != UniquenessTranscriptHashJobV2::ALL.len())
                || constrained_uniqueness_transcript != expected
            {
                return Err(CheckpointError::Backend(format!(
                    "Plonky3 AIR uniqueness-transcript chunk coverage mismatch: index {}, count {}, actual {constrained_uniqueness_transcript}, expected {expected}",
                    chunk.index, chunk.count
                )));
            }
        }
        if has_complete_transition && domain.includes(AirDomainV2::Transition) {
            if constrained_transition != source_count {
                return Err(CheckpointError::Backend(format!(
                    "Plonky3 AIR frozen-transition chunk coverage mismatch: index {}, count {}, actual {constrained_transition}, expected sources {source_count}",
                    chunk.index, chunk.count
                )));
            }
        }
    }
    let mut final_outputs = None;
    let chunk_count = packed_word_exprs.len() / 2;
    for (index, chunk) in packed_word_exprs.chunks_exact(2).enumerate() {
        let mut inputs = vec![None; 4];
        for (slot, packed) in inputs.iter_mut().take(2).zip(chunk.iter().copied()) {
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
    let commitment_targets = &root_statement_targets[ROOT_STATEMENT_COMMITMENT_INDEX_V2
        ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2];
    for (output, expected) in final_outputs
        .into_iter()
        .take(2)
        .zip(commitment_targets.chunks_exact(4))
    {
        let output = output.ok_or_else(|| {
            CheckpointError::Backend("Plonky3 AIR Poseidon output lane is missing".into())
        })?;
        let coefficients = builder
            .decompose_ext_to_base_coeffs::<KoalaBear>(output)
            .map_err(|_| {
                CheckpointError::Backend("Plonky3 AIR Poseidon output decomposition failed".into())
            })?;
        if coefficients.len() != expected.len() {
            return Err(CheckpointError::Invariant);
        }
        for (&coefficient, &expected) in coefficients.iter().zip(expected) {
            builder.connect(coefficient, expected);
        }
    }
    Ok(PreparedBuilderV2 {
        builder,
        private_inputs,
    })
}

fn build_runner(
    words: &[u16],
    event_vector: Option<&[u8]>,
    retain_inputs: bool,
    chunk: AirChunkV2,
    root_statement: &RootStatementV2,
) -> Result<PreparedRunnerV2, CheckpointError> {
    let PreparedBuilderV2 {
        builder,
        private_inputs,
    } = prepare_builder(words, event_vector, retain_inputs, chunk, root_statement)?;
    let circuit = builder
        .build()
        .map_err(|_| CheckpointError::Backend("Plonky3 circuit build failed".into()))?;
    Ok(PreparedRunnerV2 {
        circuit,
        private_inputs,
    })
}

fn prepare_runner(
    words: &[u16],
    event_vector: Option<&[u8]>,
    chunk: AirChunkV2,
    root_statement: &RootStatementV2,
) -> Result<PreparedRunnerV2, CheckpointError> {
    build_runner(words, event_vector, true, chunk, root_statement)
}

#[cfg(test)]
fn prepare_shape(words: &[u16], event_vector: Option<&[u8]>) -> Result<(), CheckpointError> {
    let chunk = AirChunkV2::singleton(AirDomainV2::Full);
    let root_statement = root_statement_fixture(words, event_vector, chunk)?;
    drop(prepare_builder(
        words,
        event_vector,
        false,
        chunk,
        &root_statement,
    )?);
    Ok(())
}

fn canonical_recursive_npo_types() -> [NpoTypeId; 5] {
    [
        NpoTypeId::poseidon2_perm(Poseidon2Config::KOALA_BEAR_D4_W32),
        NpoTypeId::poseidon2_perm(Poseidon2Config::KOALA_BEAR_D4_W16),
        NpoTypeId::recompose(),
        u16_range_npo_type(),
        root_statement_npo_type(),
    ]
}

fn canonical_recursive_preprocessors() -> Vec<Box<dyn NpoPreprocessor<KoalaBear>>> {
    vec![
        Box::new(Poseidon2Preprocessor),
        recompose_preprocessor::<KoalaBear>(true),
        Box::new(U16RangePreprocessorV2),
        Box::new(RootStatementPreprocessorV2),
    ]
}

fn canonical_recursive_air_builders() -> Vec<Box<dyn NpoAirBuilder<Plonky3StarkConfigV2, 4>>> {
    let mut builders = poseidon2_air_builders_for_configs::<Plonky3StarkConfigV2, 4>(vec![
        Poseidon2Config::KOALA_BEAR_D4_W32,
        Poseidon2Config::KOALA_BEAR_D4_W16,
    ]);
    builders.extend(recompose_air_builders::<Plonky3StarkConfigV2, 4>(1, true));
    builders.push(Box::new(U16RangeAirBuilderV2));
    builders.push(Box::new(RootStatementAirBuilderV2));
    builders
}

fn register_canonical_recursive_tables(prover: &mut BatchStarkProver<Plonky3StarkConfigV2>) {
    prover.register_poseidon2_table::<4>(Poseidon2Config::KOALA_BEAR_D4_W32);
    prover.register_poseidon2_table::<4>(Poseidon2Config::KOALA_BEAR_D4_W16);
    prover.register_recompose_table::<4>(true);
    prover.register_table_prover(Box::new(U16RangeProverV2));
    prover.register_table_prover(Box::new(RootStatementProverV2));
}

fn prepare_circuit(
    words: &[u16],
    event_vector: Option<&[u8]>,
    chunk: AirChunkV2,
    root_statement: &RootStatementV2,
) -> Result<PreparedCircuitV2, CheckpointError> {
    let PreparedRunnerV2 {
        circuit,
        private_inputs,
    } = prepare_runner(words, event_vector, chunk, root_statement)?;
    let table_packing =
        TablePacking::new(PLONKY3_TABLE_PUBLIC_LANES_V2, PLONKY3_TABLE_ALU_LANES_V2)
            .with_min_trace_height(PLONKY3_TABLE_MIN_HEIGHT_V2);
    let config = hardened_koala_bear_config();
    let preprocessors = canonical_recursive_preprocessors();
    let air_builders = canonical_recursive_air_builders();
    let (airs_degrees, primitive_columns, non_primitive_columns) =
        get_airs_and_degrees_with_prep::<Plonky3StarkConfigV2, _, 4>(
            &circuit,
            &table_packing,
            &preprocessors,
            &air_builders,
            ConstraintProfile::Standard,
        )
        .map_err(|error| {
            CheckpointError::Backend(format!("Plonky3 AIR lowering failed: {error}"))
        })?;
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

#[derive(Deserialize, Serialize)]
struct CachedChunkProofV2 {
    generation: u16,
    domain: u8,
    replica: u8,
    index: u16,
    count: u16,
    cache_key: [u8; 32],
    dimensions: Plonky3TraceDimensionsV2,
    proof: BatchStarkProof<Plonky3StarkConfigV2>,
}

#[derive(Serialize)]
struct CachedChunkProofRefV2<'a> {
    generation: u16,
    domain: u8,
    replica: u8,
    index: u16,
    count: u16,
    cache_key: [u8; 32],
    dimensions: Plonky3TraceDimensionsV2,
    proof: &'a BatchStarkProof<Plonky3StarkConfigV2>,
}

fn chunk_cache_root() -> Result<Option<PathBuf>, CheckpointError> {
    let Some(root) = EnvConfig
        .get("Z00Z_PLONKY3_CHUNK_CACHE_DIR")
        .map_err(|error| {
            CheckpointError::Backend(format!("invalid Plonky3 cache path: {error}"))
        })?
    else {
        return Ok(None);
    };
    let root = PathBuf::from(root);
    if !root.is_absolute() {
        return Err(CheckpointError::Backend(
            "Plonky3 chunk cache path must be absolute".into(),
        ));
    }
    Ok(Some(root))
}

#[derive(Clone, Copy)]
struct ChunkCacheKeysV2 {
    current: [u8; 32],
    // The immediately previous aggregation generation is accepted only after
    // actual verification against the current generation-bound leaf authority.
    previous_aggregation_generation: [u8; 32],
}

fn chunk_cache_keys(
    words: &[u16],
    event_vector: &[u8],
    chunk: AirChunkV2,
) -> Result<ChunkCacheKeysV2, CheckpointError> {
    let mut word_bytes = Vec::with_capacity(
        words
            .len()
            .checked_mul(2)
            .ok_or(CheckpointError::Overflow)?,
    );
    for word in words {
        word_bytes.extend_from_slice(&word.to_le_bytes());
    }
    let predicate_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.chunk-cache.v2",
        "predicate_words",
        &[&word_bytes],
    );
    word_bytes.zeroize();
    let event_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.chunk-cache.v2",
        "event_vector",
        &[event_vector],
    );
    let coordinates = [
        chunk.domain.tag(),
        chunk.replica,
        u8::try_from(chunk.index & 0xff).map_err(|_| CheckpointError::Limit)?,
        u8::try_from(chunk.index >> 8).map_err(|_| CheckpointError::Limit)?,
        u8::try_from(chunk.count & 0xff).map_err(|_| CheckpointError::Limit)?,
        u8::try_from(chunk.count >> 8).map_err(|_| CheckpointError::Limit)?,
    ];
    let security = RecursiveSecurityBudgetManifestV2::authority_pinned()?;
    let current_parameters = Plonky3ParameterManifestV2::authority_pinned(&security)?;
    let previous_parameters =
        Plonky3ParameterManifestV2::authority_pinned_for_aggregation_generation(
            &security,
            PLONKY3_PREVIOUS_AGGREGATION_TREE_GENERATION_V2,
        )?;
    let key_for = |parameter_digest: &[u8; 32]| {
        sha256_256(
            "z00z.storage.checkpoint.plonky3.chunk-cache.v2",
            "verified_chunk_identity",
            &[
                ACTIVE_PLONKY3_SOURCE_REVISION_V2.as_bytes(),
                ACTIVE_PLONKY3_CIRCUIT_VERSION_V2.as_bytes(),
                &PLONKY3_CHUNK_CACHE_GENERATION_V2.to_le_bytes(),
                &coordinates,
                parameter_digest,
                &security.digest(),
                &predicate_digest,
                &event_digest,
            ],
        )
    };
    Ok(ChunkCacheKeysV2 {
        current: key_for(&current_parameters.digest),
        previous_aggregation_generation: key_for(&previous_parameters.digest),
    })
}

fn chunk_cache_key(
    words: &[u16],
    event_vector: &[u8],
    chunk: AirChunkV2,
) -> Result<[u8; 32], CheckpointError> {
    Ok(chunk_cache_keys(words, event_vector, chunk)?.current)
}

fn chunk_cache_path(root: &Path, key: [u8; 32]) -> PathBuf {
    let mut name = String::with_capacity(64 + ".postcard".len());
    use core::fmt::Write as _;
    for byte in key {
        write!(&mut name, "{byte:02x}").expect("write cache-key hex");
    }
    name.push_str(".postcard");
    root.join(name)
}

fn cached_chunk_file_exists(
    words: &[u16],
    event_vector: &[u8],
    chunk: AirChunkV2,
) -> Result<bool, CheckpointError> {
    let Some(root) = chunk_cache_root()? else {
        return Ok(false);
    };
    let keys = chunk_cache_keys(words, event_vector, chunk)?;
    for key in [keys.current, keys.previous_aggregation_generation] {
        if path_exists_no_follow(chunk_cache_path(&root, key)).map_err(|error| {
            CheckpointError::Backend(format!("Plonky3 cache lookup failed: {error}"))
        })? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn load_cached_chunk(
    words: &[u16],
    event_vector: &[u8],
    chunk: AirChunkV2,
    root_statement: &RootStatementV2,
) -> Result<
    Option<(
        BatchStarkProof<Plonky3StarkConfigV2>,
        Plonky3TraceDimensionsV2,
    )>,
    CheckpointError,
> {
    let Some(root) = chunk_cache_root()? else {
        return Ok(None);
    };
    let keys = chunk_cache_keys(words, event_vector, chunk)?;
    let mut selected = None;
    for key in [keys.current, keys.previous_aggregation_generation] {
        let path = chunk_cache_path(&root, key);
        if path_exists_no_follow(&path).map_err(|error| {
            CheckpointError::Backend(format!("Plonky3 cache lookup failed: {error}"))
        })? {
            selected = Some((key, path));
            break;
        }
    }
    let Some((key, path)) = selected else {
        return Ok(None);
    };
    let bytes = match read_file_bounded(
        &path,
        u64::try_from(PLONKY3_CHUNK_BYTES_V2).map_err(|_| CheckpointError::Limit)?,
    ) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(None),
    };
    let (cached, remaining): (CachedChunkProofV2, &[u8]) = match postcard::take_from_bytes(&bytes) {
        Ok(decoded) => decoded,
        Err(_) => return Ok(None),
    };
    if !remaining.is_empty()
        || postcard::to_allocvec(&cached).map_err(|_| CheckpointError::Canonical)? != bytes
        || cached.generation != PLONKY3_CHUNK_CACHE_GENERATION_V2
        || cached.domain != chunk.domain.tag()
        || cached.replica != chunk.replica
        || cached.index != chunk.index
        || cached.count != chunk.count
        || cached.cache_key != key
    {
        return Ok(None);
    }
    if verify_domain(words, event_vector, chunk, root_statement, &cached.proof).is_err() {
        return Ok(None);
    }
    emit_chunk_progress("cache_verified", chunk);
    if matches!(
        EnvConfig.get("Z00Z_PLONKY3_RESOURCE_TELEMETRY"),
        Ok(Some(_))
    ) {
        eprintln!(
            "Z00Z_PLONKY3_CACHE_V1 hit {} {} {} {}",
            chunk.domain.name(),
            chunk.replica,
            chunk.index,
            chunk.count
        );
    }
    Ok(Some((cached.proof, cached.dimensions)))
}

fn save_cached_chunk(
    words: &[u16],
    event_vector: &[u8],
    chunk: AirChunkV2,
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    dimensions: Plonky3TraceDimensionsV2,
) -> Result<(), CheckpointError> {
    let Some(root) = chunk_cache_root()? else {
        return Ok(());
    };
    let key = chunk_cache_key(words, event_vector, chunk)?;
    let path = chunk_cache_path(&root, key);
    if path_exists_no_follow(&path).map_err(|error| {
        CheckpointError::Backend(format!("Plonky3 cache lookup failed: {error}"))
    })? {
        return Ok(());
    }
    let cached = CachedChunkProofRefV2 {
        generation: PLONKY3_CHUNK_CACHE_GENERATION_V2,
        domain: chunk.domain.tag(),
        replica: chunk.replica,
        index: chunk.index,
        count: chunk.count,
        cache_key: key,
        dimensions,
        proof,
    };
    let bytes = postcard::to_allocvec(&cached).map_err(|_| CheckpointError::Canonical)?;
    if bytes.len() > PLONKY3_CHUNK_BYTES_V2 {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::ProofBytesTooLarge,
        ));
    }
    atomic_write_file_private(&path, &bytes).map_err(|error| {
        CheckpointError::Backend(format!("Plonky3 chunk cache write failed: {error}"))
    })
}

fn batch_recursion_input(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
) -> Result<RecursionInput<'_, Plonky3StarkConfigV2, BatchOnly>, CheckpointError> {
    let instance_count = proof.proof.opened_values.instances.len();
    let primitive_count = instance_count
        .checked_sub(proof.non_primitives.len())
        .ok_or(CheckpointError::Canonical)?;
    if primitive_count != 3 {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3ProofMalformed,
        ));
    }
    let mut table_public_inputs = vec![Vec::new(); instance_count];
    for (index, entry) in proof.non_primitives.iter().enumerate() {
        table_public_inputs[primitive_count + index] = entry.public_values.clone();
    }
    Ok(RecursionInput::BatchStark {
        proof,
        common_data: &proof.stark_common,
        table_public_inputs,
    })
}

fn aggregation_table_packing() -> TablePacking {
    TablePacking::new(2, 2).with_min_trace_height(PLONKY3_TABLE_MIN_HEIGHT_V2)
}

fn merge_tree_root<T, E>(
    root: Option<T>,
    node: T,
    merge: impl FnOnce(T, T) -> Result<T, E>,
) -> Result<Option<T>, E> {
    match root {
        None => Ok(Some(node)),
        Some(left) => merge(left, node).map(Some),
    }
}

const PLONKY3_AGGREGATION_PREP_CACHE_SLOTS_V2: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AggregationPrepKeyV2 {
    fingerprint: AggregationCircuitFingerprint,
    left_common_digest: [u8; 32],
    right_common_digest: [u8; 32],
    relation_tag: u8,
}

struct AggregationPrepSlotV2 {
    key: AggregationPrepKeyV2,
    cache: Option<AggregationPrepCache<Plonky3StarkConfigV2>>,
    last_used: u64,
}

struct AggregationPrepPoolV2 {
    slots: Vec<AggregationPrepSlotV2>,
    clock: u64,
}

impl AggregationPrepPoolV2 {
    const fn new() -> Self {
        Self {
            slots: Vec::new(),
            clock: 0,
        }
    }

    fn cache_for(
        &mut self,
        circuit: &Circuit<Plonky3ChallengeV2>,
        left_common_digest: [u8; 32],
        right_common_digest: [u8; 32],
        relation: AggregationRelationV2,
    ) -> &mut Option<AggregationPrepCache<Plonky3StarkConfigV2>> {
        self.clock = self.clock.saturating_add(1);
        let key = AggregationPrepKeyV2 {
            fingerprint: AggregationCircuitFingerprint {
                witness_count: circuit.witness_count,
                public_flat_len: circuit.public_flat_len,
                private_flat_len: circuit.private_flat_len,
                ops_len: circuit.ops.len(),
            },
            left_common_digest,
            right_common_digest,
            relation_tag: relation.cache_tag(),
        };
        let slot_index = self
            .slots
            .iter()
            .position(|slot| slot.key == key)
            .unwrap_or_else(|| {
                if self.slots.len() < PLONKY3_AGGREGATION_PREP_CACHE_SLOTS_V2 {
                    self.slots.push(AggregationPrepSlotV2 {
                        key,
                        cache: None,
                        last_used: self.clock,
                    });
                    self.slots.len() - 1
                } else {
                    let replace = self
                        .slots
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, slot)| slot.last_used)
                        .map(|(index, _)| index)
                        .unwrap_or(0);
                    self.slots[replace] = AggregationPrepSlotV2 {
                        key,
                        cache: None,
                        last_used: self.clock,
                    };
                    replace
                }
            });
        self.slots[slot_index].last_used = self.clock;
        &mut self.slots[slot_index].cache
    }
}

#[derive(Clone, Copy, Debug)]
struct BoundRecursionBackendV2;

impl PcsRecursionBackend<Plonky3StarkConfigV2, BatchOnly, 4> for BoundRecursionBackendV2 {
    type VerifierResult = FriVerifierResult<Plonky3StarkConfigV2>;

    fn prepare_circuit(
        &self,
        config: &Plonky3StarkConfigV2,
        circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    ) -> Result<(), VerificationError> {
        config.prepare_circuit_for_verification(circuit)?;
        register_root_statement_npo(circuit);
        Ok(())
    }

    fn build_verifier_circuit(
        &self,
        previous: &RecursionInput<'_, Plonky3StarkConfigV2, BatchOnly>,
        config: &Plonky3StarkConfigV2,
        circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    ) -> Result<Self::VerifierResult, VerificationError> {
        let RecursionInput::BatchStark {
            proof, common_data, ..
        } = previous
        else {
            return Err(VerificationError::InvalidProofShape(
                "bound recursion accepts only batch-STARK inputs".into(),
            ));
        };
        let mut available = self.non_primitive_provers(proof.ext_degree);
        let mut provers = Vec::with_capacity(proof.non_primitives.len());
        for entry in &proof.non_primitives {
            let Some(index) = available
                .iter()
                .position(|prover| prover.op_type() == entry.op_type)
            else {
                return Err(VerificationError::InvalidProofShape(format!(
                    "unsupported recursive non-primitive table: {:?}",
                    entry.op_type
                )));
            };
            provers.push(available.remove(index));
        }
        let lookup = LogUpGadget::new();
        let (inputs, op_ids) = verify_p3_batch_proof_circuit::<
            Plonky3StarkConfigV2,
            MerkleCapTargets<KoalaBear, PLONKY3_MMCS_DIGEST_ELEMS_V2>,
            Plonky3RecInputProofV2,
            Plonky3RecOpeningProofV2,
            _,
            _,
            PLONKY3_MMCS_WIDTH_V2,
            PLONKY3_MMCS_RATE_V2,
            4,
        >(
            config,
            circuit,
            proof,
            config.pcs_verifier_params(),
            common_data,
            &lookup,
            Poseidon2Config::KOALA_BEAR_D4_W32,
            &provers,
        )?;
        Ok(FriVerifierResult::BatchStark(inputs, op_ids))
    }

    fn set_private_data(
        &self,
        _config: &Plonky3StarkConfigV2,
        runner: &mut CircuitRunner<'_, Plonky3ChallengeV2>,
        op_ids: &[NonPrimitiveOpId],
        previous: &RecursionInput<'_, Plonky3StarkConfigV2, BatchOnly>,
    ) -> Result<(), &'static str> {
        Plonky3StarkConfigV2::with_fri_opening_proof(previous, |opening_proof| {
            Plonky3StarkConfigV2::set_fri_private_data(runner, op_ids, opening_proof)
        })
    }

    fn non_primitive_preprocessors(&self) -> Vec<Box<dyn NpoPreprocessor<KoalaBear>>> {
        canonical_recursive_preprocessors()
    }

    fn non_primitive_provers(
        &self,
        ext_degree: usize,
    ) -> Vec<Box<dyn TableProver<Plonky3StarkConfigV2>>> {
        if ext_degree != 4 {
            return Vec::new();
        }
        let mut provers: Vec<Box<dyn TableProver<Plonky3StarkConfigV2>>> = vec![
            Box::new(Poseidon2Prover::new(
                Poseidon2Config::KOALA_BEAR_D4_W32,
                ConstraintProfile::Standard,
            )),
            Box::new(Poseidon2Prover::new(
                Poseidon2Config::KOALA_BEAR_D4_W16,
                ConstraintProfile::Standard,
            )),
        ];
        provers.extend(recompose_table_provers::<Plonky3StarkConfigV2, 4>(1, true));
        provers.push(Box::new(U16RangeProverV2));
        provers.push(Box::new(RootStatementProverV2));
        provers
    }

    fn non_primitive_air_builders(&self) -> Vec<Box<dyn NpoAirBuilder<Plonky3StarkConfigV2, 4>>> {
        canonical_recursive_air_builders()
    }
}

fn root_statement_targets(
    result: &FriVerifierResult<Plonky3StarkConfigV2>,
) -> Result<&[ExprId], CheckpointError> {
    let FriVerifierResult::BatchStark(inputs, _) = result else {
        return Err(CheckpointError::Canonical);
    };
    let targets = inputs
        .air_public_targets
        .last()
        .ok_or(CheckpointError::Canonical)?;
    if targets.len() != ROOT_STATEMENT_FIELDS_V2 {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(targets)
}

fn circuit_pair_hash(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    left: &[ExprId],
    right: &[ExprId],
    domain: RootCommitmentDomainV2,
) -> Result<[ExprId; ROOT_STATEMENT_COMMITMENT_FIELDS_V2], CheckpointError> {
    if left.len() != ROOT_STATEMENT_COMMITMENT_FIELDS_V2
        || right.len() != ROOT_STATEMENT_COMMITMENT_FIELDS_V2
    {
        return Err(CheckpointError::Canonical);
    }
    let pack = |circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
                fields: &[ExprId]|
     -> Result<[ExprId; 2], CheckpointError> {
        let packed = fields
            .chunks_exact(4)
            .map(|coefficients| {
                circuit
                    .recompose_base_coeffs_to_ext::<KoalaBear>(coefficients)
                    .map_err(|_| {
                        CheckpointError::Backend(
                            "Plonky3 root commitment recomposition failed".into(),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        packed.try_into().map_err(|_| CheckpointError::Invariant)
    };
    let left = pack(circuit, left)?;
    let right = pack(circuit, right)?;
    let domain = [
        circuit.define_const(Plonky3ChallengeV2::new([
            KoalaBear::from_u64(0x5a30),
            KoalaBear::from_u64(0x5254),
            KoalaBear::from_u8(domain.tag()),
            KoalaBear::ZERO,
        ])),
        circuit.define_const(Plonky3ChallengeV2::ZERO),
    ];
    let blocks = [domain, left, right];
    let mut final_outputs = None;
    for (index, block) in blocks.into_iter().enumerate() {
        let is_last = index + 1 == blocks.len();
        let mut inputs = vec![None; 4];
        inputs[0] = Some(block[0]);
        inputs[1] = Some(block[1]);
        let (_, outputs) = circuit
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
                CheckpointError::Backend("Plonky3 root commitment Poseidon lowering failed".into())
            })?;
        if is_last {
            final_outputs = Some(outputs);
        }
    }
    let mut commitment = Vec::with_capacity(ROOT_STATEMENT_COMMITMENT_FIELDS_V2);
    // RootStatement consumes the eight base-field commitment coefficients
    // directly, so the decomposition link must use the coefficient-producing
    // recompose table instead of leaving hint limbs off the WitnessChecks bus.
    circuit.set_recompose_coeff_ctl_for_decompose_links(true);
    for output in final_outputs
        .ok_or(CheckpointError::Invariant)?
        .into_iter()
        .take(2)
    {
        let output = output.ok_or_else(|| {
            CheckpointError::Backend("Plonky3 root commitment Poseidon output missing".into())
        })?;
        commitment.extend(
            circuit
                .decompose_ext_to_base_coeffs::<KoalaBear>(output)
                .map_err(|_| {
                    CheckpointError::Backend("Plonky3 root commitment decomposition failed".into())
                })?,
        );
    }
    circuit.set_recompose_coeff_ctl_for_decompose_links(false);
    commitment
        .try_into()
        .map_err(|_| CheckpointError::Invariant)
}

fn constrain_statement_equal(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    left: ExprId,
    right: ExprId,
) {
    if left == right {
        return;
    }
    let difference = circuit.sub(left, right);
    if difference == ExprId::ZERO {
        return;
    }
    // `connect(left, right)` aliases witness slots. That is invalid for two
    // independently allocated recursive public inputs because both Public AIR
    // rows would then claim the same WitnessChecks creator. A zero right-hand
    // side is simplified by CircuitBuilder::sub, so square that residual to
    // retain a distinct constrained ALU output before connecting it to zero.
    let residual = if difference == left || difference == right {
        circuit.mul(difference, difference)
    } else {
        difference
    };
    circuit.assert_zero(residual);
}

fn build_bound_aggregation_circuit(
    left: &RecursionInput<'_, Plonky3StarkConfigV2, BatchOnly>,
    right: &RecursionInput<'_, Plonky3StarkConfigV2, BatchOnly>,
    config: &Plonky3StarkConfigV2,
    backend: &BoundRecursionBackendV2,
    relation: AggregationRelationV2,
) -> Result<
    (
        Circuit<Plonky3ChallengeV2>,
        (
            FriVerifierResult<Plonky3StarkConfigV2>,
            FriVerifierResult<Plonky3StarkConfigV2>,
        ),
    ),
    CheckpointError,
> {
    let mut circuit = CircuitBuilder::new();
    backend
        .prepare_circuit(config, &mut circuit)
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 bound recursion preparation failed: {error:?}"
            ))
        })?;
    let left_result = backend
        .build_verifier_circuit(left, config, &mut circuit)
        .map_err(|error| {
            CheckpointError::Backend(format!("Plonky3 left recursion verifier failed: {error:?}"))
        })?;
    let right_result = backend
        .build_verifier_circuit(right, config, &mut circuit)
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 right recursion verifier failed: {error:?}"
            ))
        })?;
    let left_statement = root_statement_targets(&left_result)?.to_vec();
    let right_statement = root_statement_targets(&right_result)?.to_vec();
    for index in 0..ROOT_STATEMENT_FIELDS_V2 {
        let is_commitment = (ROOT_STATEMENT_COMMITMENT_INDEX_V2
            ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2)
            .contains(&index);
        if !is_commitment
            && index != ROOT_STATEMENT_START_INDEX_V2
            && index != ROOT_STATEMENT_COUNT_INDEX_V2
            && (index != ROOT_STATEMENT_REPLICA_INDEX_V2
                || relation == AggregationRelationV2::LeafRange)
        {
            constrain_statement_equal(&mut circuit, left_statement[index], right_statement[index]);
        }
    }
    let mut output_statement = left_statement.clone();
    match relation.replica_ordinals() {
        None => {
            let expected_right_start = circuit.add(
                left_statement[ROOT_STATEMENT_START_INDEX_V2],
                left_statement[ROOT_STATEMENT_COUNT_INDEX_V2],
            );
            constrain_statement_equal(
                &mut circuit,
                expected_right_start,
                right_statement[ROOT_STATEMENT_START_INDEX_V2],
            );
            output_statement[ROOT_STATEMENT_COUNT_INDEX_V2] = circuit.add(
                left_statement[ROOT_STATEMENT_COUNT_INDEX_V2],
                right_statement[ROOT_STATEMENT_COUNT_INDEX_V2],
            );
        }
        Some((left_replica, right_replica, output_replica)) => {
            let zero = circuit.define_const(Plonky3ChallengeV2::ZERO);
            let left_replica = circuit.define_const(lift_koala(KoalaBear::from_u8(left_replica)));
            let right_replica = circuit.define_const(lift_koala(KoalaBear::from_u8(right_replica)));
            let output_replica =
                circuit.define_const(lift_koala(KoalaBear::from_u8(output_replica)));
            constrain_statement_equal(
                &mut circuit,
                left_statement[ROOT_STATEMENT_REPLICA_INDEX_V2],
                left_replica,
            );
            constrain_statement_equal(
                &mut circuit,
                right_statement[ROOT_STATEMENT_REPLICA_INDEX_V2],
                right_replica,
            );
            constrain_statement_equal(
                &mut circuit,
                left_statement[ROOT_STATEMENT_START_INDEX_V2],
                zero,
            );
            constrain_statement_equal(
                &mut circuit,
                right_statement[ROOT_STATEMENT_START_INDEX_V2],
                zero,
            );
            constrain_statement_equal(
                &mut circuit,
                left_statement[ROOT_STATEMENT_COUNT_INDEX_V2],
                left_statement[ROOT_STATEMENT_TOTAL_INDEX_V2],
            );
            constrain_statement_equal(
                &mut circuit,
                right_statement[ROOT_STATEMENT_COUNT_INDEX_V2],
                right_statement[ROOT_STATEMENT_TOTAL_INDEX_V2],
            );
            output_statement[ROOT_STATEMENT_REPLICA_INDEX_V2] = output_replica;
            output_statement[ROOT_STATEMENT_START_INDEX_V2] = zero;
            output_statement[ROOT_STATEMENT_COUNT_INDEX_V2] =
                left_statement[ROOT_STATEMENT_TOTAL_INDEX_V2];
        }
    }
    let parent_commitment = circuit_pair_hash(
        &mut circuit,
        &output_statement[ROOT_STATEMENT_COMMITMENT_INDEX_V2
            ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
        &right_statement[ROOT_STATEMENT_COMMITMENT_INDEX_V2
            ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
        relation.commitment_domain(),
    )?;
    output_statement[ROOT_STATEMENT_COMMITMENT_INDEX_V2
        ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
        .copy_from_slice(&parent_commitment);
    bind_root_statement_targets(&mut circuit, &output_statement)
        .map_err(|_| CheckpointError::Canonical)?;
    let circuit = circuit.build().map_err(|error| {
        CheckpointError::Backend(format!(
            "Plonky3 bound aggregation circuit build failed: {error:?}"
        ))
    })?;
    Ok((circuit, (left_result, right_result)))
}

fn proof_root_statement_values(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
) -> Result<&[KoalaBear], CheckpointError> {
    let mut matches = proof
        .non_primitives
        .iter()
        .filter(|entry| entry.op_type == root_statement_npo_type());
    let values = matches
        .next()
        .map(|entry| entry.public_values.as_slice())
        .ok_or(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ))?;
    if matches.next().is_some() || values.len() != ROOT_STATEMENT_FIELDS_V2 {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(values)
}

#[cfg(test)]
fn combined_root_statement_values(
    left: &[KoalaBear],
    right: &[KoalaBear],
) -> Result<Vec<KoalaBear>, CheckpointError> {
    combined_root_statement_values_for_relation(left, right, AggregationRelationV2::LeafRange)
}

fn combined_root_statement_values_for_relation(
    left: &[KoalaBear],
    right: &[KoalaBear],
    relation: AggregationRelationV2,
) -> Result<Vec<KoalaBear>, CheckpointError> {
    if left.len() != ROOT_STATEMENT_FIELDS_V2 || right.len() != ROOT_STATEMENT_FIELDS_V2 {
        return Err(CheckpointError::Canonical);
    }
    for index in 0..ROOT_STATEMENT_FIELDS_V2 {
        let is_commitment = (ROOT_STATEMENT_COMMITMENT_INDEX_V2
            ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2)
            .contains(&index);
        if !is_commitment
            && index != ROOT_STATEMENT_START_INDEX_V2
            && index != ROOT_STATEMENT_COUNT_INDEX_V2
            && (index != ROOT_STATEMENT_REPLICA_INDEX_V2
                || relation == AggregationRelationV2::LeafRange)
            && left[index] != right[index]
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
    }
    let mut output = left.to_vec();
    match relation.replica_ordinals() {
        None => {
            if left[ROOT_STATEMENT_START_INDEX_V2] + left[ROOT_STATEMENT_COUNT_INDEX_V2]
                != right[ROOT_STATEMENT_START_INDEX_V2]
            {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
                ));
            }
            output[ROOT_STATEMENT_COUNT_INDEX_V2] =
                left[ROOT_STATEMENT_COUNT_INDEX_V2] + right[ROOT_STATEMENT_COUNT_INDEX_V2];
        }
        Some((left_replica, right_replica, output_replica)) => {
            if left[ROOT_STATEMENT_REPLICA_INDEX_V2].as_canonical_u64() != u64::from(left_replica)
                || right[ROOT_STATEMENT_REPLICA_INDEX_V2].as_canonical_u64()
                    != u64::from(right_replica)
                || left[ROOT_STATEMENT_START_INDEX_V2] != KoalaBear::ZERO
                || right[ROOT_STATEMENT_START_INDEX_V2] != KoalaBear::ZERO
                || left[ROOT_STATEMENT_COUNT_INDEX_V2] != left[ROOT_STATEMENT_TOTAL_INDEX_V2]
                || right[ROOT_STATEMENT_COUNT_INDEX_V2] != right[ROOT_STATEMENT_TOTAL_INDEX_V2]
            {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
                ));
            }
            output[ROOT_STATEMENT_REPLICA_INDEX_V2] = KoalaBear::from_u8(output_replica);
            output[ROOT_STATEMENT_START_INDEX_V2] = KoalaBear::ZERO;
            output[ROOT_STATEMENT_COUNT_INDEX_V2] = output[ROOT_STATEMENT_TOTAL_INDEX_V2];
        }
    }
    let left_commitment = left[ROOT_STATEMENT_COMMITMENT_INDEX_V2
        ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
        .try_into()
        .map_err(|_| CheckpointError::Canonical)?;
    let right_commitment = right[ROOT_STATEMENT_COMMITMENT_INDEX_V2
        ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
        .try_into()
        .map_err(|_| CheckpointError::Canonical)?;
    output[ROOT_STATEMENT_COMMITMENT_INDEX_V2
        ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
        .copy_from_slice(&poseidon_pair_hash_for_domain(
            left_commitment,
            right_commitment,
            relation.commitment_domain(),
        ));
    Ok(output)
}

fn emit_root_common_candidate(digest: [u8; 32]) {
    let mut encoded = String::with_capacity(64);
    use core::fmt::Write as _;
    for byte in digest {
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    eprintln!("Z00Z_PLONKY3_ROOT_COMMON_CANDIDATE_V2 {encoded}");
}

fn require_authorized_root_common(
    actual: [u8; 32],
    authority: [u8; 32],
) -> Result<(), CheckpointError> {
    if authority == [0; 32] {
        emit_root_common_candidate(actual);
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    if actual != authority {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

fn verify_aggregation_proof(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    expected_statement: &RootStatementV2,
    require_authority_pin: bool,
) -> Result<(), CheckpointError> {
    run_in_fresh_prover_pool("aggregation-verify", || {
        verify_aggregation_proof_in_pool(proof)?;
        if proof_root_statement_values(proof)? != expected_statement.values() {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        if require_authority_pin {
            let actual = common_binding_digest(&proof.stark_common)?;
            require_authorized_root_common(actual, ACTIVE_PLONKY3_ROOT_COMMON_DIGEST_V2)?;
        }
        Ok(())
    })
}

fn verify_aggregation_proof_in_pool(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
) -> Result<(), CheckpointError> {
    if proof.ext_degree != usize::from(PLONKY3_TRACE_EXTENSION_DEGREE_V2)
        || proof.w_binomial.is_none()
        || proof.alu_quintic_trinomial
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3ProofMalformed,
        ));
    }
    let mut verifier = BatchStarkProver::new(hardened_koala_bear_config())
        .with_table_packing(proof.table_packing.clone());
    register_canonical_recursive_tables(&mut verifier);
    verifier
        .verify_all_tables::<Plonky3ChallengeV2>(proof)
        .map_err(|error| {
            emit_resource_error("aggregation_verify", &error);
            CheckpointError::BackendVerificationFailed
        })
}

fn aggregate_pair(
    left: AggregationNodeV2,
    right: AggregationNodeV2,
    prep_pool: &mut AggregationPrepPoolV2,
) -> Result<AggregationNodeV2, CheckpointError> {
    let expected_right_start = left
        .leaf_start
        .checked_add(left.leaf_count)
        .ok_or(CheckpointError::Overflow)?;
    if left.replica != right.replica || right.leaf_start != expected_right_start {
        return Err(CheckpointError::Canonical);
    }
    let replica = left.replica;
    let leaf_start = left.leaf_start;
    let leaf_count = left
        .leaf_count
        .checked_add(right.leaf_count)
        .ok_or(CheckpointError::Overflow)?;
    let depth = left
        .depth
        .max(right.depth)
        .checked_add(1)
        .ok_or(CheckpointError::Overflow)?;
    if depth > PLONKY3_BASE_RECURSION_DEPTH_V2 {
        return Err(CheckpointError::Limit);
    }
    let proof = aggregate_proof_pair(
        left.proof,
        right.proof,
        AggregationRelationV2::LeafRange,
        prep_pool,
    )?;
    Ok(AggregationNodeV2 {
        proof,
        replica,
        leaf_start,
        leaf_count,
        depth,
    })
}

fn fold_replica_roots(
    left: RecursiveRootProofV2,
    right: RecursiveRootProofV2,
    relation: AggregationRelationV2,
) -> Result<RecursiveRootProofV2, CheckpointError> {
    let (left_replica, right_replica, output_replica) = relation
        .replica_ordinals()
        .ok_or(CheckpointError::Canonical)?;
    if left.replica != left_replica
        || right.replica != right_replica
        || left.leaf_count == 0
        || left.leaf_count != right.leaf_count
    {
        return Err(CheckpointError::Canonical);
    }
    let leaf_count = left.leaf_count;
    let depth = left
        .depth
        .max(right.depth)
        .checked_add(1)
        .ok_or(CheckpointError::Overflow)?;
    if depth > PLONKY3_BASE_RECURSION_DEPTH_V2 {
        return Err(CheckpointError::Limit);
    }
    let proof = run_in_fresh_prover_pool("replica-fold-prove", move || {
        let mut prep_pool = AggregationPrepPoolV2::new();
        aggregate_proof_pair(left.proof, right.proof, relation, &mut prep_pool)
    })?;
    Ok(RecursiveRootProofV2 {
        replica: output_replica,
        leaf_count,
        depth,
        proof,
    })
}

fn aggregate_proof_pair(
    left: BatchStarkProof<Plonky3StarkConfigV2>,
    right: BatchStarkProof<Plonky3StarkConfigV2>,
    relation: AggregationRelationV2,
    prep_pool: &mut AggregationPrepPoolV2,
) -> Result<BatchStarkProof<Plonky3StarkConfigV2>, CheckpointError> {
    let left_common_digest = common_binding_digest(&left.stark_common)?;
    let right_common_digest = common_binding_digest(&right.stark_common)?;
    let expected_statement = combined_root_statement_values_for_relation(
        proof_root_statement_values(&left)?,
        proof_root_statement_values(&right)?,
        relation,
    )?;
    emit_resource_phase("aggregation");
    let config = hardened_koala_bear_config();
    let backend = BoundRecursionBackendV2;
    let params = ProveNextLayerParams {
        table_packing: aggregation_table_packing(),
        constraint_profile: ConstraintProfile::Standard,
    };
    let left_input = batch_recursion_input(&left)?;
    let right_input = batch_recursion_input(&right)?;
    let (circuit, (left_result, right_result)) =
        build_bound_aggregation_circuit(&left_input, &right_input, &config, &backend, relation)?;
    let output = prove_aggregation_layer::<Plonky3StarkConfigV2, _, _, _, 4>(
        &left_input,
        &right_input,
        &left_result,
        &right_result,
        &circuit,
        &config,
        &backend,
        &params,
        Some(prep_pool.cache_for(&circuit, left_common_digest, right_common_digest, relation)),
    )
    .map_err(|error| {
        CheckpointError::Backend(format!("Plonky3 aggregation proving failed: {error:?}"))
    })?;
    let p3_recursion::RecursionOutput(proof, prover_data) = output;
    let prepared_common_digest = common_binding_digest(prover_data.common_data())?;
    if common_binding_digest(&proof.stark_common)? != prepared_common_digest {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    drop(circuit);
    drop(left_input);
    drop(right_input);
    drop(left);
    drop(right);
    drop(prover_data);
    verify_aggregation_proof_in_pool(&proof)?;
    if proof_root_statement_values(&proof)? != expected_statement {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(proof)
}

fn ensure_replica_chunk_cache(
    pool: &rayon::ThreadPool,
    words: &[u16],
    event_vector: &[u8],
    chunks: &[AirChunkV2],
    root_statement: &RootStatementAuthorityV2,
) -> Result<(), CheckpointError> {
    validate_chunk_sequence(chunks.iter().copied())?;
    for (leaf_index, &chunk) in chunks.iter().enumerate() {
        let statement = root_statement.leaf(
            chunk.replica,
            u16::try_from(leaf_index).map_err(|_| CheckpointError::Limit)?,
        )?;
        emit_resource_phase(chunk.domain.name());
        emit_chunk_progress("lookup", chunk);
        if cached_chunk_file_exists(words, event_vector, chunk)? {
            emit_chunk_progress("cache_verify_deferred", chunk);
            continue;
        }
        emit_chunk_progress("prove_start", chunk);
        let (proof, dimensions) = pool
            .install(|| prove_domain_in_pool(words, event_vector, chunk, &statement))
            .map_err(|error| {
                CheckpointError::Backend(format!("Plonky3 chunk {chunk:?} proving failed: {error}"))
            })?;
        emit_chunk_progress("prove_verified", chunk);
        save_cached_chunk(words, event_vector, chunk, &proof, dimensions)?;
        emit_chunk_progress("cache_saved", chunk);
        drop(proof);
        trim_prover_heap();
    }
    Ok(())
}

fn trim_prover_heap() {
    let _ = z00z_utils::os_hardening::trim_process_heap_best_effort();
}

fn prove_replica_tree(
    words: &[u16],
    event_vector: &[u8],
    chunks: Vec<AirChunkV2>,
    root_statement: &RootStatementAuthorityV2,
    trace_dimensions: &mut Plonky3TraceDimensionsV2,
) -> Result<RecursiveRootProofV2, CheckpointError> {
    validate_chunk_sequence(chunks.iter().copied())?;
    let replica = chunks
        .first()
        .map(|chunk| chunk.replica)
        .ok_or(CheckpointError::Canonical)?;
    let leaf_total = u16::try_from(chunks.len()).map_err(|_| CheckpointError::Limit)?;
    let leaves = run_in_fresh_prover_pool("base-verify", move || {
        let mut leaves = Vec::with_capacity(chunks.len());
        for (leaf_index, chunk) in chunks.into_iter().enumerate() {
            let statement = root_statement.leaf(
                chunk.replica,
                u16::try_from(leaf_index).map_err(|_| CheckpointError::Limit)?,
            )?;
            emit_resource_phase(chunk.domain.name());
            let (chunk_proof, chunk_dimensions) =
                load_cached_chunk(words, event_vector, chunk, &statement)?.ok_or_else(|| {
                    CheckpointError::Backend(format!(
                        "Plonky3 materialized chunk cache missing for {chunk:?}"
                    ))
                })?;
            trace_dimensions.add_chunk(chunk_dimensions)?;
            leaves.push(AggregationNodeV2 {
                proof: chunk_proof,
                replica,
                leaf_start: u16::try_from(leaf_index).map_err(|_| CheckpointError::Limit)?,
                leaf_count: 1,
                depth: 0,
            });
        }
        Ok(leaves)
    })?;
    let root = aggregate_canonical_nodes_bounded(leaves)?;
    if root.replica != replica || root.leaf_start != 0 || root.leaf_count != leaf_total {
        return Err(CheckpointError::Canonical);
    }
    if proof_root_statement_values(&root.proof)? != root_statement.root(replica)?.values() {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(RecursiveRootProofV2 {
        replica,
        leaf_count: leaf_total,
        depth: root.depth,
        proof: root.proof,
    })
}

fn canonical_aggregation_segments(
    nodes: Vec<AggregationNodeV2>,
) -> Result<Vec<Vec<AggregationNodeV2>>, CheckpointError> {
    if nodes.is_empty() {
        return Err(CheckpointError::Canonical);
    }
    let mut remaining = nodes.len();
    let mut nodes = nodes.into_iter();
    let mut segments = Vec::new();
    while remaining > 0 {
        let segment_len = 1_usize
            .checked_shl(remaining.ilog2())
            .ok_or(CheckpointError::Overflow)?;
        let segment: Vec<_> = nodes.by_ref().take(segment_len).collect();
        if segment.len() != segment_len {
            return Err(CheckpointError::Canonical);
        }
        segments.push(segment);
        remaining = remaining
            .checked_sub(segment_len)
            .ok_or(CheckpointError::Overflow)?;
    }
    if nodes.next().is_some() {
        return Err(CheckpointError::Canonical);
    }
    Ok(segments)
}

fn run_aggregation_worker(
    pool: rayon::ThreadPool,
    receiver: std::sync::mpsc::Receiver<AggregationWaveMessageV2>,
    sender: std::sync::mpsc::Sender<Result<AggregationWaveResultV2, CheckpointError>>,
) -> Result<(), CheckpointError> {
    pool.install(move || {
        let mut prep_pool = AggregationPrepPoolV2::new();
        let mut halted = false;
        while let Ok(message) = receiver.recv() {
            let AggregationWaveMessageV2::Job(job) = message else {
                break;
            };
            let result = if halted {
                Err(CheckpointError::Backend(
                    "Plonky3 aggregation worker halted after an earlier proof failure".into(),
                ))
            } else {
                aggregate_pair(job.left, job.right, &mut prep_pool).map(|node| {
                    AggregationWaveResultV2 {
                        ordinal: job.ordinal,
                        segment: job.segment,
                        node,
                    }
                })
            };
            halted |= result.is_err();
            sender.send(result).map_err(|_| {
                CheckpointError::Backend("Plonky3 aggregation result channel closed".into())
            })?;
        }
        drop(prep_pool);
        trim_prover_heap();
        Ok(())
    })
}

fn dispatch_aggregation_wave(
    jobs: Vec<AggregationWaveJobV2>,
    senders: &[std::sync::mpsc::Sender<AggregationWaveMessageV2>; PLONKY3_AGGREGATION_WORKERS_V2],
    receiver: &std::sync::mpsc::Receiver<Result<AggregationWaveResultV2, CheckpointError>>,
) -> Result<Vec<AggregationWaveResultV2>, CheckpointError> {
    let result_count = jobs.len();
    for job in jobs {
        let worker = job.ordinal % PLONKY3_AGGREGATION_WORKERS_V2;
        senders[worker]
            .send(AggregationWaveMessageV2::Job(job))
            .map_err(|_| {
                CheckpointError::Backend("Plonky3 aggregation job channel closed".into())
            })?;
    }
    let mut results = Vec::with_capacity(result_count);
    let mut first_error = None;
    for _ in 0..result_count {
        match receiver.recv().map_err(|_| {
            CheckpointError::Backend("Plonky3 aggregation result channel closed".into())
        })? {
            Ok(result) => results.push(result),
            Err(error) if first_error.is_none() => first_error = Some(error),
            Err(_) => {}
        }
    }
    if let Some(error) = first_error {
        return Err(error);
    }
    results.sort_unstable_by_key(|result| result.ordinal);
    Ok(results)
}

fn aggregate_canonical_segments_with_workers(
    mut segments: Vec<Vec<AggregationNodeV2>>,
    senders: &[std::sync::mpsc::Sender<AggregationWaveMessageV2>; PLONKY3_AGGREGATION_WORKERS_V2],
    receiver: &std::sync::mpsc::Receiver<Result<AggregationWaveResultV2, CheckpointError>>,
) -> Result<AggregationNodeV2, CheckpointError> {
    while segments.iter().any(|segment| segment.len() > 1) {
        let segment_count = segments.len();
        let mut jobs = Vec::new();
        let mut singletons = Vec::new();
        let mut ordinal = 0_usize;
        for (segment_index, segment) in segments.into_iter().enumerate() {
            if segment.len() == 1 {
                singletons.push((
                    segment_index,
                    segment
                        .into_iter()
                        .next()
                        .ok_or(CheckpointError::Canonical)?,
                ));
                continue;
            }
            if segment.len() % 2 != 0 {
                return Err(CheckpointError::Canonical);
            }
            let mut nodes = segment.into_iter();
            while let Some(left) = nodes.next() {
                let right = nodes.next().ok_or(CheckpointError::Canonical)?;
                let job = AggregationWaveJobV2 {
                    ordinal,
                    segment: segment_index,
                    left,
                    right,
                };
                jobs.push(job);
                ordinal = ordinal.checked_add(1).ok_or(CheckpointError::Overflow)?;
            }
        }
        let results = dispatch_aggregation_wave(jobs, senders, receiver)?;

        let mut next_segments: Vec<Vec<AggregationNodeV2>> =
            (0..segment_count).map(|_| Vec::new()).collect();
        for (segment, node) in singletons {
            next_segments[segment].push(node);
        }
        for result in results {
            next_segments[result.segment].push(result.node);
        }
        if next_segments.iter().any(Vec::is_empty) {
            return Err(CheckpointError::Canonical);
        }
        segments = next_segments;
        trim_prover_heap();
    }

    let mut roots = segments
        .into_iter()
        .map(|segment| segment.into_iter().next().ok_or(CheckpointError::Canonical));
    let mut root = roots.next().ok_or(CheckpointError::Canonical)??;
    for (ordinal, next) in roots.enumerate() {
        let next = next?;
        senders[0]
            .send(AggregationWaveMessageV2::Job(AggregationWaveJobV2 {
                ordinal,
                segment: 0,
                left: root,
                right: next,
            }))
            .map_err(|_| {
                CheckpointError::Backend("Plonky3 aggregation job channel closed".into())
            })?;
        root = receiver
            .recv()
            .map_err(|_| {
                CheckpointError::Backend("Plonky3 aggregation result channel closed".into())
            })??
            .node;
    }
    Ok(root)
}

fn aggregate_canonical_nodes_bounded(
    nodes: Vec<AggregationNodeV2>,
) -> Result<AggregationNodeV2, CheckpointError> {
    let segments = canonical_aggregation_segments(nodes)?;
    let pool_a =
        build_bounded_prover_pool_with_threads("aggregation-wave-a", PLONKY3_PROVER_THREADS_V2)?;
    let pool_b =
        build_bounded_prover_pool_with_threads("aggregation-wave-b", PLONKY3_PROVER_THREADS_V2)?;
    let (sender_a, receiver_a) = std::sync::mpsc::channel();
    let (sender_b, receiver_b) = std::sync::mpsc::channel();
    let senders = [sender_a, sender_b];
    let (result_sender_a, result_receiver) = std::sync::mpsc::channel();
    let result_sender_b = result_sender_a.clone();

    std::thread::scope(|scope| {
        let worker_a =
            scope.spawn(move || run_aggregation_worker(pool_a, receiver_a, result_sender_a));
        let worker_b =
            scope.spawn(move || run_aggregation_worker(pool_b, receiver_b, result_sender_b));
        let result =
            aggregate_canonical_segments_with_workers(segments, &senders, &result_receiver);
        for sender in &senders {
            let _ = sender.send(AggregationWaveMessageV2::Stop);
        }
        let worker_a = worker_a.join().map_err(|_| {
            CheckpointError::Backend("Plonky3 aggregation worker A panicked".into())
        })?;
        let worker_b = worker_b.join().map_err(|_| {
            CheckpointError::Backend("Plonky3 aggregation worker B panicked".into())
        })?;
        worker_a?;
        worker_b?;
        trim_prover_heap();
        result
    })
}

fn prove_domain_in_pool(
    words: &[u16],
    event_vector: &[u8],
    chunk: AirChunkV2,
    root_statement: &RootStatementV2,
) -> Result<
    (
        BatchStarkProof<Plonky3StarkConfigV2>,
        Plonky3TraceDimensionsV2,
    ),
    CheckpointError,
> {
    emit_chunk_progress("circuit_prepare", chunk);
    let prepared = prepare_circuit(words, Some(event_vector), chunk, root_statement)?;
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
    let private_input_count = private_inputs.len();
    drop(private_inputs);
    emit_chunk_progress("trace_build", chunk);
    let traces = runner
        .run()
        .map_err(|_| CheckpointError::BackendVerificationFailed)?;
    let dimensions = Plonky3TraceDimensionsV2 {
        chunk_count: 1,
        predicate_words: words.len(),
        event_vector_bytes: event_vector.len(),
        circuit_witnesses: circuit.witness_count,
        circuit_operations: circuit.ops.len(),
        private_inputs: private_input_count,
        witness_rows: traces.witness_trace.num_rows(),
        constant_rows: traces.const_trace.values.len(),
        public_rows: traces.public_trace.values.len(),
        alu_rows: traces.alu_trace.values.len(),
        non_primitive_tables: traces.non_primitive_traces.len(),
        non_primitive_rows: traces
            .non_primitive_traces
            .values()
            .map(|trace| trace.rows())
            .sum(),
        max_chunk_witnesses: circuit.witness_count,
        max_chunk_operations: circuit.ops.len(),
        max_chunk_alu_rows: traces.alu_trace.values.len(),
        max_chunk_npo_rows: traces
            .non_primitive_traces
            .values()
            .map(|trace| trace.rows())
            .sum(),
    };
    drop(circuit);
    emit_chunk_trace_dimensions(chunk, &dimensions);
    emit_chunk_progress("stark_prove", chunk);
    let mut prover = BatchStarkProver::new(config).with_table_packing(table_packing);
    register_canonical_recursive_tables(&mut prover);
    let proof = prover
        .prove_all_tables(&traces, &data)
        .map_err(|_| CheckpointError::BackendVerificationFailed)?;
    drop(traces);
    drop(data);
    emit_chunk_progress("stark_verify", chunk);
    prover
        .verify_all_tables::<Plonky3TraceFieldV2>(&proof)
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 {chunk:?} actual verifier rejected the generated proof: {error}"
            ))
        })?;
    Ok((proof, dimensions))
}

fn verify_domain(
    words: &[u16],
    event_vector: &[u8],
    chunk: AirChunkV2,
    root_statement: &RootStatementV2,
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
) -> Result<(), CheckpointError> {
    run_in_fresh_prover_pool("base-verify", || {
        verify_domain_in_pool(words, event_vector, chunk, root_statement, proof)
    })
}

fn verify_domain_in_pool(
    words: &[u16],
    event_vector: &[u8],
    chunk: AirChunkV2,
    root_statement: &RootStatementV2,
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
) -> Result<(), CheckpointError> {
    emit_chunk_progress("cache_verify_prepare", chunk);
    let prepared = prepare_circuit(words, Some(event_vector), chunk, root_statement)?;
    let expected_air_binding = common_binding_digest(prepared.data.common_data())?;
    if common_binding_digest(&proof.stark_common)? != expected_air_binding {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    if proof_root_statement_values(proof)? != root_statement.values() {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let expected_packing = TablePacking::new(1, PLONKY3_TABLE_ALU_LANES_V2)
        .with_min_trace_height(PLONKY3_TABLE_MIN_HEIGHT_V2);
    let expected_npo_types = canonical_recursive_npo_types();
    if proof.table_packing != expected_packing
        || proof.ext_degree != usize::from(PLONKY3_TRACE_EXTENSION_DEGREE_V2)
        || proof.w_binomial.is_none()
        || proof.alu_quintic_trinomial
        || proof.non_primitives.len() != expected_npo_types.len()
        || proof
            .non_primitives
            .iter()
            .zip(expected_npo_types.iter())
            .any(|(entry, expected)| &entry.op_type != expected)
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
    register_canonical_recursive_tables(&mut verifier);
    emit_chunk_progress("cache_verify_stark", chunk);
    verifier
        .verify_all_tables::<Plonky3TraceFieldV2>(proof)
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 cached {chunk:?} actual verifier rejected the proof: {error}"
            ))
        })
}

fn hardened_koala_bear_config() -> Plonky3StarkConfigV2 {
    let permutation = default_koalabear_poseidon2_32();
    let leaf_hash = Plonky3HashV2::new(permutation.clone());
    let compress = Plonky3CompressionV2::new(permutation.clone());
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
    let challenger = Plonky3ChallengerV2::new(permutation);
    Plonky3StarkConfigV2 {
        config: Arc::new(Plonky3RawStarkConfigV2::new(pcs, challenger)),
        fri_verifier_params: FriVerifierParams::with_mmcs(
            usize::from(PLONKY3_FRI_LOG_BLOWUP_V2),
            usize::from(PLONKY3_FRI_LOG_FINAL_POLY_LEN_V2),
            usize::from(PLONKY3_FRI_COMMIT_POW_BITS_V2),
            usize::from(PLONKY3_FRI_QUERY_POW_BITS_V2),
            usize::from(PLONKY3_FRI_NUM_QUERIES_V2),
            Poseidon2Config::KOALA_BEAR_D4_W32,
        ),
    }
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

fn poseidon_pair_hash(
    left: [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
    right: [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
) -> [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2] {
    poseidon_pair_hash_for_domain(left, right, RootCommitmentDomainV2::LeafRange)
}

fn poseidon_pair_hash_for_domain(
    left: [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
    right: [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
    commitment_domain: RootCommitmentDomainV2,
) -> [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2] {
    let permutation = default_koalabear_poseidon2_16();
    let mut domain = [KoalaBear::ZERO; ROOT_STATEMENT_COMMITMENT_FIELDS_V2];
    domain[0] = KoalaBear::from_u64(0x5a30);
    domain[1] = KoalaBear::from_u64(0x5254);
    domain[2] = KoalaBear::from_u8(commitment_domain.tag());
    let mut state = [KoalaBear::ZERO; 16];
    for block in [domain, left, right] {
        state[..ROOT_STATEMENT_COMMITMENT_FIELDS_V2].copy_from_slice(&block);
        state = permutation.permute(state);
    }
    state[..ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
        .try_into()
        .expect("fixed Poseidon2 rate")
}

fn aggregate_commitments(
    leaves: &[[KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2]],
) -> Result<[KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2], CheckpointError> {
    if leaves.is_empty() {
        return Err(CheckpointError::Canonical);
    }
    let mut slots: Vec<Option<[KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2]>> = Vec::new();
    for &leaf in leaves {
        let mut node = leaf;
        let mut level = 0_usize;
        loop {
            if slots.len() <= level {
                slots.resize(level + 1, None);
            }
            let Some(left) = slots[level].take() else {
                slots[level] = Some(node);
                break;
            };
            node = poseidon_pair_hash(left, node);
            level = level.checked_add(1).ok_or(CheckpointError::Overflow)?;
        }
    }
    let mut root = None;
    for node in slots.into_iter().rev().flatten() {
        root = merge_tree_root(root, node, |left, right| {
            Ok::<_, CheckpointError>(poseidon_pair_hash(left, right))
        })?;
    }
    root.ok_or(CheckpointError::Canonical)
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

fn leaf_manifest_digest(event_vector: &[u8]) -> Result<[u8; 32], CheckpointError> {
    let mut bytes = Vec::new();
    bytes.push(PLONKY3_FRI_REPLICA_COUNT_V2);
    for replica in 0..PLONKY3_FRI_REPLICA_COUNT_V2 {
        let chunks = air_chunks(event_vector, replica)?;
        validate_chunk_sequence(chunks.iter().copied())?;
        bytes.push(replica);
        bytes.extend_from_slice(
            &u16::try_from(chunks.len())
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes(),
        );
        for chunk in chunks {
            bytes.push(chunk.domain.tag());
            bytes.push(chunk.replica);
            bytes.extend_from_slice(&chunk.index.to_le_bytes());
            bytes.extend_from_slice(&chunk.count.to_le_bytes());
        }
    }
    Ok(sha256_256(
        "z00z.storage.checkpoint.plonky3.recursive-leaves.v2",
        "ordered_replica_chunk_manifest",
        &[&bytes],
    ))
}

fn expected_streaming_tree_depth(leaf_count: u16) -> Result<u16, CheckpointError> {
    if leaf_count == 0 {
        return Err(CheckpointError::Canonical);
    }
    let mut slots: Vec<Option<u16>> = Vec::new();
    for _ in 0..leaf_count {
        let mut depth = 0_u16;
        let mut level = 0_usize;
        loop {
            if slots.len() <= level {
                slots.resize(level + 1, None);
            }
            let Some(left_depth) = slots[level].take() else {
                slots[level] = Some(depth);
                break;
            };
            depth = left_depth
                .max(depth)
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
            level = level.checked_add(1).ok_or(CheckpointError::Overflow)?;
        }
    }
    let mut root = None;
    for depth in slots.into_iter().rev().flatten() {
        root = merge_tree_root(root, depth, |left_depth, right_depth| {
            left_depth
                .max(right_depth)
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)
        })?;
    }
    root.ok_or(CheckpointError::Canonical)
}

fn validate_root_envelope(envelope: &RecursiveRootEnvelopeV2) -> Result<(), CheckpointError> {
    if envelope.leaf_manifest_digest == [0; 32] {
        return Err(CheckpointError::Canonical);
    }
    let root = &envelope.root;
    let leaf_count = root.leaf_count;
    let replica_depth = expected_streaming_tree_depth(leaf_count)?;
    let expected_depth = replica_depth
        .checked_add(2)
        .ok_or(CheckpointError::Overflow)?;
    if expected_depth > PLONKY3_BASE_RECURSION_DEPTH_V2 {
        return Err(CheckpointError::Limit);
    }
    let statement = proof_root_statement_values(&root.proof)?;
    let common = common_binding_digest(&root.proof.stark_common)?;
    if root.replica != PLONKY3_FINAL_REPLICA_FOLD_ORDINAL_V2
        || root.depth != expected_depth
        || root.proof.validate().is_err()
        || common == [0; 32]
        || statement[ROOT_STATEMENT_REPLICA_INDEX_V2].as_canonical_u64()
            != u64::from(PLONKY3_FINAL_REPLICA_FOLD_ORDINAL_V2)
        || statement[ROOT_STATEMENT_START_INDEX_V2] != KoalaBear::ZERO
        || statement[ROOT_STATEMENT_COUNT_INDEX_V2].as_canonical_u64() != u64::from(leaf_count)
        || statement[ROOT_STATEMENT_TOTAL_INDEX_V2].as_canonical_u64() != u64::from(leaf_count)
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3ProofMalformed,
        ));
    }
    Ok(())
}

fn root_proof_binding(
    envelope: &RecursiveRootEnvelopeV2,
) -> Result<RecursiveRootBindingV2, CheckpointError> {
    validate_root_envelope(envelope)?;
    let root = &envelope.root;
    Ok(RecursiveRootBindingV2 {
        fold_ordinal: root.replica,
        leaf_count: root.leaf_count,
        depth: root.depth,
        common_digest: common_binding_digest(&root.proof.stark_common)?,
        proof_digest: recursive_root_proof_digest(&root.proof)?,
    })
}

fn recursive_root_proof_digest(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
) -> Result<[u8; 32], CheckpointError> {
    let bytes = postcard::to_allocvec(proof).map_err(|_| CheckpointError::Canonical)?;
    Ok(sha256_256(
        "z00z.storage.checkpoint.plonky3.recursive-root.v2",
        "root_proof",
        &[&bytes],
    ))
}

fn root_binding_digest(envelope: &RecursiveRootEnvelopeV2) -> Result<[u8; 32], CheckpointError> {
    let root = root_proof_binding(envelope)?;
    let mut bytes = Vec::with_capacity(103);
    bytes.extend_from_slice(&envelope.leaf_manifest_digest);
    bytes.push(PLONKY3_FRI_REPLICA_COUNT_V2);
    bytes.push(PLONKY3_AGGREGATION_TREE_GENERATION_V2);
    bytes.push(root.fold_ordinal);
    bytes.extend_from_slice(&root.leaf_count.to_le_bytes());
    bytes.extend_from_slice(&root.depth.to_le_bytes());
    bytes.extend_from_slice(&root.common_digest);
    bytes.extend_from_slice(&root.proof_digest);
    Ok(sha256_256(
        "z00z.storage.checkpoint.plonky3.recursive-air.v2",
        "ordered_three_replica_fold_root",
        &[&bytes],
    ))
}

fn encode_recursive_roots(envelope: &RecursiveRootEnvelopeV2) -> Result<Vec<u8>, CheckpointError> {
    validate_root_envelope(envelope)?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&PLONKY3_ROOT_MAGIC_V2);
    bytes.extend_from_slice(&PLONKY3_BASE_WIRE_VERSION_V2.to_le_bytes());
    bytes.push(PLONKY3_FRI_REPLICA_COUNT_V2);
    bytes.push(PLONKY3_AGGREGATION_TREE_GENERATION_V2);
    bytes.extend_from_slice(&envelope.leaf_manifest_digest);
    let root = &envelope.root;
    bytes.push(root.replica);
    bytes.extend_from_slice(&root.leaf_count.to_le_bytes());
    bytes.extend_from_slice(&root.depth.to_le_bytes());
    let proof = postcard::to_allocvec(&root.proof).map_err(|_| CheckpointError::Canonical)?;
    bytes.extend_from_slice(
        &u32::try_from(proof.len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&proof);
    if bytes.len() > PLONKY3_PUBLISH_BYTES_V2 {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::ProofSizeBudgetExceeded,
        ));
    }
    Ok(bytes)
}

fn decode_recursive_roots(bytes: &[u8]) -> Result<RecursiveRootEnvelopeV2, CheckpointError> {
    if bytes.len() < PLONKY3_ROOT_MAGIC_V2.len() + 36
        || bytes[..PLONKY3_ROOT_MAGIC_V2.len()] != PLONKY3_ROOT_MAGIC_V2
    {
        return Err(CheckpointError::Canonical);
    }
    let mut cursor = PLONKY3_ROOT_MAGIC_V2.len();
    if take_u16(bytes, &mut cursor)? != PLONKY3_BASE_WIRE_VERSION_V2
        || take_array::<1>(bytes, &mut cursor)?[0] != PLONKY3_FRI_REPLICA_COUNT_V2
        || take_array::<1>(bytes, &mut cursor)?[0] != PLONKY3_AGGREGATION_TREE_GENERATION_V2
    {
        return Err(CheckpointError::Canonical);
    }
    let leaf_manifest_digest = take_array::<32>(bytes, &mut cursor)?;
    let replica = take_array::<1>(bytes, &mut cursor)?[0];
    let leaf_count = take_u16(bytes, &mut cursor)?;
    let depth = take_u16(bytes, &mut cursor)?;
    let proof_len =
        usize::try_from(take_u32(bytes, &mut cursor)?).map_err(|_| CheckpointError::Limit)?;
    if proof_len == 0 || proof_len > PLONKY3_PUBLISH_BYTES_V2 {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::ProofBytesTooLarge,
        ));
    }
    let proof_bytes = take_slice(bytes, &mut cursor, proof_len)?;
    let (proof, remaining): (BatchStarkProof<Plonky3StarkConfigV2>, &[u8]) =
        postcard::take_from_bytes(proof_bytes).map_err(|_| CheckpointError::Canonical)?;
    if !remaining.is_empty()
        || postcard::to_allocvec(&proof).map_err(|_| CheckpointError::Canonical)? != proof_bytes
    {
        return Err(CheckpointError::Canonical);
    }
    if cursor != bytes.len() {
        return Err(CheckpointError::Canonical);
    }
    let envelope = RecursiveRootEnvelopeV2 {
        leaf_manifest_digest,
        root: RecursiveRootProofV2 {
            replica,
            leaf_count,
            depth,
            proof,
        },
    };
    validate_root_envelope(&envelope)?;
    Ok(envelope)
}

fn validate_chunk_sequence(
    chunks: impl IntoIterator<Item = AirChunkV2>,
) -> Result<(), CheckpointError> {
    let chunks: Vec<AirChunkV2> = chunks.into_iter().collect();
    let first = chunks.first().copied().ok_or(CheckpointError::Canonical)?;
    first.validate()?;
    let replica = first.replica;
    if chunks
        .iter()
        .any(|chunk| chunk.validate().is_err() || chunk.replica != replica)
    {
        return Err(CheckpointError::Canonical);
    }
    if first.domain != AirDomainV2::Structural || first.index != 0 {
        return Err(CheckpointError::Canonical);
    }
    let structural_count = usize::from(first.count);
    let hash_first = chunks
        .get(structural_count)
        .copied()
        .ok_or(CheckpointError::Canonical)?;
    if hash_first.domain != AirDomainV2::Hash || hash_first.index != 0 {
        return Err(CheckpointError::Canonical);
    }
    let hash_count = usize::from(hash_first.count);
    let source_start = structural_count
        .checked_add(hash_count)
        .ok_or(CheckpointError::Overflow)?;
    let source_first = chunks
        .get(source_start)
        .copied()
        .ok_or(CheckpointError::Canonical)?;
    if source_first.domain != AirDomainV2::Source || source_first.index != 0 {
        return Err(CheckpointError::Canonical);
    }
    let source_chunk_count = usize::from(source_first.count);
    let list_start = source_start
        .checked_add(source_chunk_count)
        .ok_or(CheckpointError::Overflow)?;
    let list_first = chunks
        .get(list_start)
        .copied()
        .ok_or(CheckpointError::Canonical)?;
    let list_count = UniquenessListHashJobV2::ALL.len();
    if list_first.domain != AirDomainV2::Lists
        || list_first.index != 0
        || usize::from(list_first.count) != list_count
    {
        return Err(CheckpointError::Canonical);
    }
    let uniqueness_start = list_start
        .checked_add(list_count)
        .ok_or(CheckpointError::Overflow)?;
    let uniqueness_first = chunks
        .get(uniqueness_start)
        .copied()
        .ok_or(CheckpointError::Canonical)?;
    let uniqueness_count = UniquenessTranscriptHashJobV2::ALL.len();
    if uniqueness_first.domain != AirDomainV2::Uniqueness
        || uniqueness_first.index != 0
        || usize::from(uniqueness_first.count) != uniqueness_count
    {
        return Err(CheckpointError::Canonical);
    }
    let trace_index = uniqueness_start
        .checked_add(uniqueness_count)
        .ok_or(CheckpointError::Overflow)?;
    let transition_start = trace_index
        .checked_add(1)
        .ok_or(CheckpointError::Overflow)?;
    let transition_first = chunks
        .get(transition_start)
        .copied()
        .ok_or(CheckpointError::Canonical)?;
    let transition_count = usize::from(transition_first.count);
    if chunks.get(trace_index).copied()
        != Some(AirChunkV2::replicated(AirDomainV2::Trace, 0, 1, replica))
        || transition_first.domain != AirDomainV2::Transition
        || transition_first.index != 0
        || chunks.len() != transition_start + transition_count
    {
        return Err(CheckpointError::Canonical);
    }
    for (index, chunk) in chunks.iter().take(structural_count).enumerate() {
        if *chunk
            != (AirChunkV2 {
                domain: AirDomainV2::Structural,
                index: u16::try_from(index).map_err(|_| CheckpointError::Limit)?,
                count: first.count,
                replica,
            })
        {
            return Err(CheckpointError::Canonical);
        }
    }
    for (index, chunk) in chunks
        .iter()
        .skip(structural_count)
        .take(hash_count)
        .enumerate()
    {
        if *chunk
            != (AirChunkV2 {
                domain: AirDomainV2::Hash,
                index: u16::try_from(index).map_err(|_| CheckpointError::Limit)?,
                count: hash_first.count,
                replica,
            })
        {
            return Err(CheckpointError::Canonical);
        }
    }
    for (index, chunk) in chunks
        .iter()
        .skip(source_start)
        .take(source_chunk_count)
        .enumerate()
    {
        if *chunk
            != (AirChunkV2 {
                domain: AirDomainV2::Source,
                index: u16::try_from(index).map_err(|_| CheckpointError::Limit)?,
                count: source_first.count,
                replica,
            })
        {
            return Err(CheckpointError::Canonical);
        }
    }
    for (index, chunk) in chunks.iter().skip(list_start).take(list_count).enumerate() {
        if *chunk
            != (AirChunkV2 {
                domain: AirDomainV2::Lists,
                index: u16::try_from(index).map_err(|_| CheckpointError::Limit)?,
                count: list_first.count,
                replica,
            })
        {
            return Err(CheckpointError::Canonical);
        }
    }
    for (index, chunk) in chunks
        .iter()
        .skip(uniqueness_start)
        .take(uniqueness_count)
        .enumerate()
    {
        if *chunk
            != (AirChunkV2 {
                domain: AirDomainV2::Uniqueness,
                index: u16::try_from(index).map_err(|_| CheckpointError::Limit)?,
                count: uniqueness_first.count,
                replica,
            })
        {
            return Err(CheckpointError::Canonical);
        }
    }
    for (index, chunk) in chunks[transition_start..].iter().enumerate() {
        if *chunk
            != (AirChunkV2 {
                domain: AirDomainV2::Transition,
                index: u16::try_from(index).map_err(|_| CheckpointError::Limit)?,
                count: transition_first.count,
                replica,
            })
        {
            return Err(CheckpointError::Canonical);
        }
    }
    Ok(())
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
    if bytes.len() > PLONKY3_PUBLISH_BYTES_V2 {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::ProofSizeBudgetExceeded,
        ));
    }
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
    use std::sync::Once;
    use tracing_subscriber::fmt::format::FmtSpan;

    fn init_resource_tracing() {
        static INIT: Once = Once::new();
        if std::env::var_os("Z00Z_PLONKY3_RESOURCE_TELEMETRY").is_none() {
            return;
        }
        INIT.call_once(|| {
            tracing_subscriber::fmt()
                .with_ansi(false)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_env_filter(tracing_subscriber::EnvFilter::new(
                    "p3_batch_stark=info,p3_circuit_prover=info,p3_fri=info,p3_dft=info",
                ))
                .try_init()
                .expect("install bounded Plonky3 resource tracing subscriber");
        });
    }

    fn copy_batch_proof(
        proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    ) -> BatchStarkProof<Plonky3StarkConfigV2> {
        let bytes = postcard::to_allocvec(proof).expect("encode test batch proof");
        let (copy, remaining) = postcard::take_from_bytes(&bytes).expect("decode test batch proof");
        assert!(remaining.is_empty());
        copy
    }

    #[test]
    fn test_aggregation_prep_key_binds_ordered_common_data() {
        let fingerprint = AggregationCircuitFingerprint {
            witness_count: 17,
            public_flat_len: 19,
            private_flat_len: 23,
            ops_len: 29,
        };
        let key = AggregationPrepKeyV2 {
            fingerprint,
            left_common_digest: [1; 32],
            right_common_digest: [2; 32],
            relation_tag: AggregationRelationV2::LeafRange.cache_tag(),
        };
        assert_ne!(
            key,
            AggregationPrepKeyV2 {
                fingerprint,
                left_common_digest: [3; 32],
                ..key
            }
        );
        assert_ne!(
            key,
            AggregationPrepKeyV2 {
                fingerprint,
                left_common_digest: key.right_common_digest,
                right_common_digest: key.left_common_digest,
                relation_tag: key.relation_tag,
            }
        );
    }

    #[test]
    fn test_streaming_tree_order() {
        fn merge_span(left: (u16, u16), right: (u16, u16)) -> Result<(u16, u16), CheckpointError> {
            if left.0.checked_add(left.1) != Some(right.0) {
                return Err(CheckpointError::Canonical);
            }
            Ok((
                left.0,
                left.1
                    .checked_add(right.1)
                    .ok_or(CheckpointError::Overflow)?,
            ))
        }

        let leaf_total = 182_u16;
        let mut slots: Vec<Option<(u16, u16)>> = Vec::new();
        for leaf_start in 0..leaf_total {
            let mut node = (leaf_start, 1_u16);
            let mut level = 0_usize;
            loop {
                if slots.len() <= level {
                    slots.resize(level + 1, None);
                }
                let Some(left) = slots[level].take() else {
                    slots[level] = Some(node);
                    break;
                };
                node = merge_span(left, node).expect("adjacent binary-counter spans");
                level += 1;
            }
        }
        let mut root = None;
        for node in slots.into_iter().rev().flatten() {
            root = merge_tree_root(root, node, merge_span).expect("ordered final fold");
        }
        assert_eq!(root, Some((0, leaf_total)));
    }

    #[test]
    fn test_chunk_cache_reuses_only_the_previous_outer_generation() {
        let chunk = AirChunkV2::singleton(AirDomainV2::Trace);
        let keys = chunk_cache_keys(&[1, 2, 3], &[4, 5, 6], chunk).unwrap();
        assert_eq!(
            keys.current,
            chunk_cache_key(&[1, 2, 3], &[4, 5, 6], chunk).unwrap()
        );
        assert_ne!(keys.current, keys.previous_aggregation_generation);
    }

    #[test]
    fn test_bounded_chunk_ranges_cover_once_in_order() {
        assert_eq!(bounded_chunk_count(17, 8).unwrap(), 3);
        let ranges = (0..3)
            .map(|index| {
                bounded_chunk_range(
                    17,
                    AirChunkV2::replicated(AirDomainV2::Structural, index, 3, 0),
                    8,
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(ranges, vec![0..8, 8..16, 16..17]);
        let covered = ranges.into_iter().flatten().collect::<Vec<_>>();
        assert_eq!(covered, (0..17).collect::<Vec<_>>());
        assert!(bounded_chunk_range(
            17,
            AirChunkV2::replicated(AirDomainV2::Structural, 0, 2, 0),
            8,
        )
        .is_err());
    }

    #[test]
    fn test_jmt_group_partition() {
        let opcodes = [
            1, 2, 4, 5, 6, //
            1, 2, 3, 4, 7, 5, 6, //
            1, 2, 4, 8, 5, 6, //
            1, 2, 4, 9, 7, 5, 6, //
            1, 2, 4, 5, 6,
        ];
        assert_eq!(
            jmt_update_group_ranges(&opcodes).unwrap(),
            vec![0..5, 5..12, 12..18, 18..25, 25..30]
        );
        let mut six_updates = opcodes.to_vec();
        six_updates.extend_from_slice(&[1, 2, 4, 5, 6]);
        assert_eq!(jmt_update_group_ranges(&six_updates).unwrap().len(), 6);
        assert_eq!(transition_chunk_count(0).unwrap(), 4);
        assert_eq!(transition_chunk_count(5).unwrap(), 8);
        assert_eq!(transition_chunk_count(6).unwrap(), 9);
        assert!(jmt_update_group_ranges(&[]).unwrap().is_empty());
        assert!(jmt_update_group_ranges(&opcodes[..29]).is_err());
        assert!(jmt_update_group_ranges(&[1, 2, 1, 6, 6]).is_err());
        assert!(jmt_update_group_ranges(&[2, 4, 5]).is_err());
    }

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

    fn bounded_hash_air_fixture() -> (Vec<u8>, u16) {
        let profile = RecursiveCircuitProfileV2::authority_pinned();
        let payload = (0..512).map(|index| index as u8).collect::<Vec<_>>();
        let source = RecursiveTraceEventV2::new(
            0,
            RecursiveTraceOpcodeV2::BeginBlock,
            structural_event_id(RecursiveTraceOpcodeV2::BeginBlock, 0, &payload),
            payload,
            &profile,
        )
        .unwrap();
        let mut events = vec![source.clone()];
        emit_derived_hash_controls(&source, &profile, |event| {
            events.push(event);
            Ok(())
        })
        .unwrap();
        let block_count = events
            .iter()
            .filter(|event| event.opcode() == RecursiveTraceOpcodeV2::ShaBlock)
            .count();
        let chunk_count =
            bounded_chunk_count(block_count, PLONKY3_HASH_ITEMS_PER_CHUNK_V2).unwrap();
        let chunk = AirChunkV2::replicated(AirDomainV2::Hash, 0, chunk_count, 0);
        assert_eq!(
            bounded_chunk_range(block_count, chunk, PLONKY3_HASH_ITEMS_PER_CHUNK_V2)
                .unwrap()
                .len(),
            usize::from(PLONKY3_HASH_ITEMS_PER_CHUNK_V2),
        );
        (encode_event_vector(&events), chunk_count)
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

    fn uniqueness_list_air_fixture(stale: bool) -> Vec<u8> {
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

        if stale {
            let block = events
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
        }
        encode_event_vector(&events)
    }

    fn uniqueness_transcript_air_fixture(missing_job: bool) -> (Vec<u8>, Vec<u8>) {
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
        if missing_job {
            let remove = events
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
            events.remove(remove);
        }
        (encode_event_vector(&events), statement)
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

    fn run_prepared(prepared: PreparedRunnerV2) -> Result<(), CheckpointError> {
        let PreparedRunnerV2 {
            circuit,
            private_inputs,
        } = prepared;
        let mut runner = circuit.runner();
        runner
            .set_private_inputs(&private_inputs)
            .map_err(|_| CheckpointError::Invariant)?;
        drop(private_inputs);
        runner
            .run()
            .map(|_| ())
            .map_err(|_| CheckpointError::BackendVerificationFailed)
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
            derive_replica_tree_bound(
                PLONKY3_FRI_PHYSICAL_QUANTUM_SEARCH_BITS_V2,
                PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2,
                PLONKY3_CHALLENGER_PHYSICAL_QUANTUM_PREIMAGE_BITS_V2,
                PLONKY3_FRI_REPLICA_COUNT_V2,
                PLONKY3_LOGICAL_NODE_COUNT_V2,
                3,
            )
            .unwrap()
            .denominator_exponent(),
            PLONKY3_PER_PROOF_BOUND_BITS_V2
        );
        assert_eq!(
            derive_per_proof_bound(
                PLONKY3_FRI_QUANTUM_SEARCH_BITS_V2,
                PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2 - 17,
                PLONKY3_CHALLENGER_QUANTUM_PREIMAGE_BITS_V2,
                3,
            )
            .unwrap()
            .denominator_exponent(),
            133
        );
        assert_eq!(
            derive_lifetime_bound(
                DyadicErrorBoundV2::new(PLONKY3_PER_PROOF_BOUND_BITS_V2).unwrap(),
                1 << 20,
                DyadicErrorBoundV2::new(128).unwrap(),
            )
            .unwrap()
            .denominator_exponent(),
            107
        );
        assert!(derive_replica_tree_bound(
            PLONKY3_FRI_PHYSICAL_QUANTUM_SEARCH_BITS_V2,
            PLONKY3_HASH_QUANTUM_COLLISION_BITS_V2,
            PLONKY3_CHALLENGER_PHYSICAL_QUANTUM_PREIMAGE_BITS_V2,
            0,
            PLONKY3_LOGICAL_NODE_COUNT_V2,
            0,
        )
        .is_err());
        assert!(derive_lifetime_bound(
            DyadicErrorBoundV2::new(PLONKY3_PER_PROOF_BOUND_BITS_V2).unwrap(),
            0,
            DyadicErrorBoundV2::new(128).unwrap(),
        )
        .is_err());
        assert!(matches!(
            derive_lifetime_bound(
                DyadicErrorBoundV2::new(PLONKY3_PER_PROOF_BOUND_BITS_V2).unwrap(),
                u64::MAX,
                DyadicErrorBoundV2::new(128).unwrap(),
            ),
            Err(CheckpointError::Overflow)
        ));

        let baseline = RecursiveSecurityBudgetManifestV2::authority_pinned().unwrap();
        let mutations: Vec<fn(&mut RecursiveSecurityBudgetManifestV2)> = vec![
            |value| value.generation += 1,
            |value| value.parameter_generation += 1,
            |value| value.base_field_bits -= 1,
            |value| value.challenge_extension_degree -= 1,
            |value| value.fri_log_blowup += 1,
            |value| value.fri_num_queries -= 1,
            |value| value.fri_commit_pow_bits += 1,
            |value| value.fri_query_pow_bits += 1,
            |value| value.fri_replica_count -= 1,
            |value| value.fri_physical_classical_bits -= 1,
            |value| value.fri_physical_quantum_search_bits -= 1,
            |value| value.fri_classical_bits -= 1,
            |value| value.fri_quantum_search_bits -= 1,
            |value| value.hash_output_bits -= 1,
            |value| value.hash_collision_bits -= 1,
            |value| value.challenger_capacity_bits -= 1,
            |value| value.challenger_physical_quantum_preimage_bits -= 1,
            |value| value.challenger_quantum_preimage_bits -= 1,
            |value| value.component_count = 0,
            |value| value.recursion_depth += 1,
            |value| value.logical_leaf_count -= 1,
            |value| value.logical_node_count -= 1,
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
    fn test_root_statement_values_share_common_binding() {
        let chunk = AirChunkV2::singleton(AirDomainV2::Full);
        let words_a = [1_u16, 2, 3, 4, 5, 6, 7, 8];
        let words_b = [8_u16, 7, 6, 5, 4, 3, 2, 1];
        let statement_a = root_statement_fixture(&words_a, None, chunk).unwrap();
        let statement_b = RootStatementV2::leaf(
            [9; 32],
            [8; 32],
            [7; 32],
            [6; 32],
            [5; 32],
            chunk_commitment(&words_b, None, chunk).unwrap(),
            0,
            0,
            1,
        )
        .unwrap();
        assert_ne!(statement_a.values(), statement_b.values());
        assert_ne!(statement_a.commitment(), statement_b.commitment());

        let prepared_a = prepare_circuit(&words_a, None, chunk, &statement_a).unwrap();
        let prepared_b = prepare_circuit(&words_b, None, chunk, &statement_b).unwrap();
        assert_eq!(
            common_binding_digest(prepared_a.data.common_data()).unwrap(),
            common_binding_digest(prepared_b.data.common_data()).unwrap(),
            "statement and leaf values must not change the verifier common data"
        );

        let wrong_statement = RootStatementV2::leaf(
            [9; 32],
            [8; 32],
            [7; 32],
            [6; 32],
            [5; 32],
            statement_a.commitment(),
            0,
            0,
            1,
        )
        .unwrap();
        assert!(prepare_circuit(&words_b, None, chunk, &wrong_statement).is_err());
    }

    #[test]
    fn test_recursive_common_ignores_statement_values() {
        let chunk = AirChunkV2::singleton(AirDomainV2::Full);
        let make_statement = |words: &[u16], mark: u8, start: u16| {
            RootStatementV2::leaf(
                [mark; 32],
                [mark.wrapping_add(1); 32],
                [mark.wrapping_add(2); 32],
                [mark.wrapping_add(3); 32],
                [mark.wrapping_add(4); 32],
                chunk_commitment(words, None, chunk).unwrap(),
                0,
                start,
                2,
            )
            .unwrap()
        };
        let words_a = [1_u16, 2, 3, 4, 5, 6, 7, 8];
        let words_b = [8_u16, 7, 6, 5, 4, 3, 2, 1];
        let a_left = prove_small_batch(&words_a, &make_statement(&words_a, 1, 0)).unwrap();
        let a_right = prove_small_batch(&words_a, &make_statement(&words_a, 1, 1)).unwrap();
        let b_left = prove_small_batch(&words_b, &make_statement(&words_b, 17, 0)).unwrap();
        let b_right = prove_small_batch(&words_b, &make_statement(&words_b, 17, 1)).unwrap();

        let config = hardened_koala_bear_config();
        let backend = BoundRecursionBackendV2;
        let a_left_input = batch_recursion_input(&a_left).unwrap();
        let a_right_input = batch_recursion_input(&a_right).unwrap();
        let b_left_input = batch_recursion_input(&b_left).unwrap();
        let b_right_input = batch_recursion_input(&b_right).unwrap();
        let (a_circuit, _) = build_bound_aggregation_circuit(
            &a_left_input,
            &a_right_input,
            &config,
            &backend,
            AggregationRelationV2::LeafRange,
        )
        .unwrap();
        let (b_circuit, _) = build_bound_aggregation_circuit(
            &b_left_input,
            &b_right_input,
            &config,
            &backend,
            AggregationRelationV2::LeafRange,
        )
        .unwrap();
        assert_eq!(
            aggregation_common_for_test(&a_circuit).unwrap(),
            aggregation_common_for_test(&b_circuit).unwrap(),
            "recursive verifier common must depend on shape, not statement values"
        );
    }

    #[test]
    fn test_pair_hash_binds_child_order() {
        let left = [KoalaBear::from_u8(1); ROOT_STATEMENT_COMMITMENT_FIELDS_V2];
        let right = [KoalaBear::from_u8(2); ROOT_STATEMENT_COMMITMENT_FIELDS_V2];
        assert_ne!(
            poseidon_pair_hash(left, right),
            poseidon_pair_hash(right, left)
        );
        assert_ne!(
            poseidon_pair_hash(left, right),
            poseidon_pair_hash_for_domain(left, right, RootCommitmentDomainV2::FirstReplicaFold,),
            "leaf-range and replica-fold commitments require separate domains"
        );
        assert_ne!(
            poseidon_pair_hash_for_domain(left, right, RootCommitmentDomainV2::FirstReplicaFold,),
            poseidon_pair_hash_for_domain(left, right, RootCommitmentDomainV2::FinalReplicaFold,),
            "the two ordered replica-fold stages require separate domains"
        );
    }

    #[test]
    fn test_replica_fold_binds_all_three_ordered_roots() {
        let digests = [[1_u8; 32], [2; 32], [3; 32], [4; 32], [5; 32]];
        let root = |replica: u8, mark: u16| {
            let commitment = [KoalaBear::from_u16(mark); ROOT_STATEMENT_COMMITMENT_FIELDS_V2];
            RootStatementV2::leaf(
                digests[0], digests[1], digests[2], digests[3], digests[4], commitment, replica, 0,
                7,
            )
            .unwrap()
            .root(commitment)
        };
        let replica_zero = root(0, 11);
        let replica_one = root(1, 12);
        let replica_two = root(2, 13);
        let first = combined_root_statement_values_for_relation(
            replica_zero.values(),
            replica_one.values(),
            AggregationRelationV2::FirstReplicaFold,
        )
        .unwrap();
        let final_values = combined_root_statement_values_for_relation(
            &first,
            replica_two.values(),
            AggregationRelationV2::FinalReplicaFold,
        )
        .unwrap();
        let first_commitment = poseidon_pair_hash_for_domain(
            replica_zero.commitment(),
            replica_one.commitment(),
            RootCommitmentDomainV2::FirstReplicaFold,
        );
        let final_commitment = poseidon_pair_hash_for_domain(
            first_commitment,
            replica_two.commitment(),
            RootCommitmentDomainV2::FinalReplicaFold,
        );
        assert_eq!(
            final_values.as_slice(),
            replica_zero
                .replica_fold_root(final_commitment, PLONKY3_FINAL_REPLICA_FOLD_ORDINAL_V2,)
                .unwrap()
                .values()
        );
        assert!(
            combined_root_statement_values_for_relation(
                replica_one.values(),
                replica_zero.values(),
                AggregationRelationV2::FirstReplicaFold,
            )
            .is_err(),
            "replica order must be fail-closed"
        );
        assert!(
            combined_root_statement_values_for_relation(
                replica_zero.values(),
                replica_two.values(),
                AggregationRelationV2::FirstReplicaFold,
            )
            .is_err(),
            "a missing physical replica must be fail-closed"
        );
    }

    #[test]
    fn test_root_common_authority_fails_closed() {
        let actual = [7_u8; 32];
        assert!(require_authorized_root_common(actual, [0; 32]).is_err());
        assert!(require_authorized_root_common(actual, [8; 32]).is_err());
        assert!(require_authorized_root_common(actual, actual).is_ok());
    }

    #[test]
    fn test_root_statement_composes_exact_ranges() {
        let digests = [[1_u8; 32], [2; 32], [3; 32], [4; 32], [5; 32]];
        let leaf = |start| {
            let commitment = [KoalaBear::from_u16(start + 1); ROOT_STATEMENT_COMMITMENT_FIELDS_V2];
            RootStatementV2::leaf(
                digests[0], digests[1], digests[2], digests[3], digests[4], commitment, 0, start, 3,
            )
            .unwrap()
        };
        let left = leaf(0);
        let middle = leaf(1);
        let right = leaf(2);

        let first_pair = combined_root_statement_values(left.values(), middle.values()).unwrap();
        assert_eq!(first_pair[ROOT_STATEMENT_START_INDEX_V2], KoalaBear::ZERO);
        assert_eq!(
            first_pair[ROOT_STATEMENT_COUNT_INDEX_V2],
            KoalaBear::from_u16(2)
        );
        let root = combined_root_statement_values(&first_pair, right.values()).unwrap();
        let commitment = poseidon_pair_hash(
            poseidon_pair_hash(left.commitment(), middle.commitment()),
            right.commitment(),
        );
        assert_eq!(root.as_slice(), left.root(commitment).values());
    }

    #[test]
    fn test_root_statement_rejects_range_and_authority_drift() {
        let digests = [[1_u8; 32], [2; 32], [3; 32], [4; 32], [5; 32]];
        let make = |values: [[u8; 32]; 5], replica, start, total| {
            let commitment = [KoalaBear::from_u16(start + 1); ROOT_STATEMENT_COMMITMENT_FIELDS_V2];
            RootStatementV2::leaf(
                values[0], values[1], values[2], values[3], values[4], commitment, replica, start,
                total,
            )
            .unwrap()
        };
        let left = make(digests, 0, 0, 3);

        for (start, label) in [(0, "duplicate"), (2, "missing")] {
            let right = make(digests, 0, start, 3);
            assert!(
                combined_root_statement_values(left.values(), right.values()).is_err(),
                "{label} leaf range must reject"
            );
        }
        for right in [make(digests, 1, 1, 3), make(digests, 0, 1, 4)] {
            assert!(combined_root_statement_values(left.values(), right.values()).is_err());
        }
        for digest_index in 0..digests.len() {
            for byte_index in 0..digests[digest_index].len() {
                let mut changed = digests;
                changed[digest_index][byte_index] ^= 1;
                let right = make(changed, 0, 1, 3);
                assert!(
                    combined_root_statement_values(left.values(), right.values()).is_err(),
                    "digest {digest_index} byte {byte_index} must reject"
                );
            }
        }
        for digest_index in 0..digests.len() {
            let mut changed = digests;
            changed[digest_index] = [0; 32];
            assert!(RootStatementV2::leaf(
                changed[0],
                changed[1],
                changed[2],
                changed[3],
                changed[4],
                [KoalaBear::ONE; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
                0,
                0,
                3,
            )
            .is_err());
        }
        assert!(RootStatementV2::leaf(
            digests[0],
            digests[1],
            digests[2],
            digests[3],
            digests[4],
            [KoalaBear::ONE; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
            0,
            0,
            0,
        )
        .is_err());
        assert!(RootStatementV2::leaf(
            digests[0],
            digests[1],
            digests[2],
            digests[3],
            digests[4],
            [KoalaBear::ONE; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
            0,
            3,
            3,
        )
        .is_err());
    }

    #[test]
    fn test_real_batch_stark_roundtrip_small() {
        let words = [0_u16, u16::MAX, 1, 2, 3, 4, 5, 6];
        let chunk = AirChunkV2::singleton(AirDomainV2::Full);
        let root_statement = root_statement_fixture(&words, None, chunk).unwrap();
        let prepared = prepare_circuit(&words, None, chunk, &root_statement).unwrap();
        let expected_binding = common_binding_digest(prepared.data.common_data()).unwrap();
        let mut runner = prepared.circuit.runner();
        runner.set_private_inputs(&prepared.private_inputs).unwrap();
        let traces = runner.run().unwrap();
        let mut prover =
            BatchStarkProver::new(prepared.config).with_table_packing(prepared.table_packing);
        register_canonical_recursive_tables(&mut prover);
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
        assert_eq!(
            proof
                .non_primitives
                .iter()
                .map(|entry| entry.op_type.clone())
                .collect::<Vec<_>>(),
            canonical_recursive_npo_types()
        );
        prover
            .verify_all_tables::<Plonky3TraceFieldV2>(&proof)
            .unwrap();

        for statement_index in [
            ROOT_STATEMENT_COMMITMENT_INDEX_V2,
            ROOT_STATEMENT_REPLICA_INDEX_V2,
            ROOT_STATEMENT_START_INDEX_V2,
        ] {
            let mut mutated = copy_batch_proof(&proof);
            let statement = mutated
                .non_primitives
                .iter_mut()
                .find(|entry| entry.op_type == root_statement_npo_type())
                .expect("root statement table");
            statement.public_values[statement_index] += KoalaBear::ONE;
            assert!(
                prover
                    .verify_all_tables::<Plonky3TraceFieldV2>(&mutated)
                    .is_err(),
                "actual verifier must reject root statement mutation at field {statement_index}"
            );
        }

        let mut mutated_common = copy_batch_proof(&proof);
        let preprocessed = mutated_common
            .stark_common
            .preprocessed
            .as_mut()
            .expect("preprocessed common commitment");
        let mut roots = preprocessed.commitment.clone().into_roots();
        roots[0][0] += KoalaBear::ONE;
        preprocessed.commitment = roots.into();
        assert!(
            prover
                .verify_all_tables::<Plonky3TraceFieldV2>(&mutated_common)
                .is_err(),
            "actual verifier must reject a preprocessed-common commitment mutation"
        );

        let input = batch_recursion_input(&proof).unwrap();
        let (verification_circuit, _) = build_bound_aggregation_circuit(
            &input,
            &input,
            &hardened_koala_bear_config(),
            &BoundRecursionBackendV2,
            AggregationRelationV2::LeafRange,
        )
        .unwrap();
        drop(verification_circuit);
    }

    #[test]
    fn test_source_sha_binding() {
        let (event_vector, stale_vector) = source_air_fixture();
        let words = predicate_words_for_test(&event_vector);
        let chunk = AirChunkV2::singleton(AirDomainV2::Full);
        let root_statement = root_statement_fixture(&words, Some(&event_vector), chunk).unwrap();
        let prepared = prepare_runner(&words, Some(&event_vector), chunk, &root_statement).unwrap();
        drop(words);
        drop(event_vector);
        run_prepared(prepared).unwrap();

        let stale_words = predicate_words_for_test(&stale_vector);
        assert!(
            prepare_runner(&stale_words, Some(&stale_vector), chunk, &root_statement,).is_err()
        );
    }

    #[test]
    #[ignore = "run in the isolated Phase 069 resource worker"]
    fn test_real_source_batch_stark_roundtrip() {
        init_resource_tracing();
        let (event_vector, _) = source_air_fixture();
        let words = predicate_words_for_test(&event_vector);
        let chunk = AirChunkV2::singleton(AirDomainV2::Source);
        let root_statement = root_statement_fixture(&words, Some(&event_vector), chunk).unwrap();
        let prepared =
            prepare_circuit(&words, Some(&event_vector), chunk, &root_statement).unwrap();
        let mut runner = prepared.circuit.runner();
        runner.set_private_inputs(&prepared.private_inputs).unwrap();
        let traces = runner.run().unwrap();
        let mut prover = BatchStarkProver::new(prepared.config)
            .with_table_packing(prepared.table_packing)
            .with_debug_lookups();
        register_canonical_recursive_tables(&mut prover);
        let proof = prover.prove_all_tables(&traces, &prepared.data).unwrap();
        prover
            .verify_all_tables::<Plonky3TraceFieldV2>(&proof)
            .unwrap();
    }

    #[test]
    #[ignore = "run in the isolated Phase 069 resource worker"]
    fn test_real_hash_chunk_batch_stark_roundtrip() {
        const DIAGNOSTIC_ROUNDS: u8 = 8;

        init_resource_tracing();
        let (event_vector, chunk_count) = bounded_hash_air_fixture();
        let words = predicate_words_for_test(&event_vector);
        let pool = build_bounded_prover_pool("hash-diagnostic").unwrap();
        for round in 0..DIAGNOSTIC_ROUNDS {
            let replica = round % PLONKY3_FRI_REPLICA_COUNT_V2;
            for index in 0..chunk_count {
                eprintln!(
                    "Z00Z_PLONKY3_DIAGNOSTIC_SEQUENCE_V1 \
                     {{\"round\":{round},\"rounds\":{DIAGNOSTIC_ROUNDS},\
                     \"index\":{index},\"count\":{chunk_count},\"replica\":{replica}}}"
                );
                let chunk = AirChunkV2::replicated(AirDomainV2::Hash, index, chunk_count, replica);
                let root_statement =
                    root_statement_fixture(&words, Some(&event_vector), chunk).unwrap();
                emit_resource_phase(AirDomainV2::Hash.name());
                let (proof, dimensions) = pool
                    .install(|| prove_domain_in_pool(&words, &event_vector, chunk, &root_statement))
                    .unwrap();
                assert!(
                    dimensions.circuit_witnesses <= 2_500_000,
                    "bounded hash leaf has {} witnesses",
                    dimensions.circuit_witnesses,
                );
                assert!(
                    dimensions.circuit_operations <= 1 << 21,
                    "bounded hash leaf has {} circuit operations",
                    dimensions.circuit_operations,
                );
                assert!(
                    dimensions.alu_rows <= 1 << 21,
                    "bounded hash leaf has {} ALU rows",
                    dimensions.alu_rows,
                );
                drop(proof);
                trim_prover_heap();
            }
        }
        drop(pool);
        trim_prover_heap();
    }

    #[test]
    #[ignore = "run in the isolated Phase 069 resource worker"]
    fn test_real_recursive_aggregation_wave_roundtrip() {
        init_resource_tracing();
        let words = [0_u16, u16::MAX, 1, 2, 3, 4, 5, 6];
        let chunk = AirChunkV2::singleton(AirDomainV2::Full);
        let commitment = chunk_commitment(&words, None, chunk).unwrap();
        let make_statement = |start| {
            RootStatementV2::leaf(
                [1; 32], [2; 32], [3; 32], [4; 32], [5; 32], commitment, 0, start, 4,
            )
            .unwrap()
        };
        let statements: Vec<_> = (0_u16..4).map(make_statement).collect();
        let mut leaves = Vec::with_capacity(statements.len());
        for (start, statement) in statements.iter().enumerate() {
            leaves.push(AggregationNodeV2 {
                proof: prove_small_batch(&words, statement).unwrap(),
                replica: 0,
                leaf_start: u16::try_from(start).unwrap(),
                leaf_count: 1,
                depth: 0,
            });
        }
        let expected_left =
            combined_root_statement_values(statements[0].values(), statements[1].values()).unwrap();
        let expected_right =
            combined_root_statement_values(statements[2].values(), statements[3].values()).unwrap();
        let expected = combined_root_statement_values(&expected_left, &expected_right).unwrap();
        let root = aggregate_canonical_nodes_bounded(leaves).unwrap();
        verify_aggregation_proof_in_pool(&root.proof).unwrap();
        assert_eq!(root.leaf_start, 0);
        assert_eq!(root.leaf_count, 4);
        assert_eq!(root.depth, 2);
        assert_eq!(proof_root_statement_values(&root.proof).unwrap(), expected);
    }

    #[test]
    fn test_trace_precommit_sha_binding() {
        let (event_vector, stale_vector) = trace_air_fixture();
        let words = predicate_words_for_test(&event_vector);
        let chunk = AirChunkV2::singleton(AirDomainV2::Full);
        let root_statement = root_statement_fixture(&words, Some(&event_vector), chunk).unwrap();
        let prepared = prepare_runner(&words, Some(&event_vector), chunk, &root_statement).unwrap();
        drop(words);
        drop(event_vector);
        run_prepared(prepared).unwrap();

        let stale_words = predicate_words_for_test(&stale_vector);
        let stale_root = root_statement_fixture(&stale_words, Some(&stale_vector), chunk).unwrap();
        let prepared =
            prepare_runner(&stale_words, Some(&stale_vector), chunk, &stale_root).unwrap();
        assert!(run_prepared(prepared).is_err());
    }

    #[test]
    fn test_uniqueness_list_sha_binding() {
        let event_vector = uniqueness_list_air_fixture(false);
        let words = predicate_words_for_test(&event_vector);
        let chunk = AirChunkV2::singleton(AirDomainV2::Full);
        let root_statement = root_statement_fixture(&words, Some(&event_vector), chunk).unwrap();
        let prepared = prepare_runner(&words, Some(&event_vector), chunk, &root_statement).unwrap();
        drop(words);
        drop(event_vector);
        run_prepared(prepared).unwrap();

        let stale_vector = uniqueness_list_air_fixture(true);
        let stale_words = predicate_words_for_test(&stale_vector);
        let prepared = prepare_runner(&stale_words, Some(&stale_vector), chunk, &root_statement);
        drop(stale_words);
        drop(stale_vector);
        let rejected = match prepared {
            Err(_) => true,
            Ok(prepared) => run_prepared(prepared).is_err(),
        };
        assert!(rejected);
    }

    #[test]
    fn test_uniqueness_transcript_binding() {
        let (event_vector, statement) = uniqueness_transcript_air_fixture(false);
        let words = predicate_words_for_test_with_statement(&event_vector, &statement);
        assert!(prepare_shape(&words, Some(&event_vector)).is_ok());
        drop(words);
        drop(event_vector);
        drop(statement);

        let (missing_job, statement) = uniqueness_transcript_air_fixture(true);
        let missing_words = predicate_words_for_test_with_statement(&missing_job, &statement);
        assert!(prepare_shape(&missing_words, Some(&missing_job)).is_err());
    }

    #[test]
    fn test_complete_transition_air_enables_backend_evidence() {
        assert!(require_complete_transition_air_v2().is_ok());
    }
}
