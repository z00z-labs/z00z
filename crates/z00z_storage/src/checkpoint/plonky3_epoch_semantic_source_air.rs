//! Canonical byte parser and semantic-source projection for epoch chunks.
//!
//! Every active row consumes exactly one byte from the SHA-bound packed event
//! source. The AIR proves vector/event framing and emits proof-bound payload
//! bytes for typed commitments, JMT records, and uniqueness transcript rows.

use core::any::Any;

use p3_air::{Air, AirBuilder, BaseAir, WindowAccess};
use p3_circuit::ops::NpoTypeId;
use p3_circuit::tables::{NonPrimitiveTrace, Traces};
use p3_field::extension::BinomialExtensionField;
use p3_field::{Field, PrimeCharacteristicRing};
use p3_koala_bear::KoalaBear;
use p3_lookup::{Count, InteractionBuilder};
use p3_matrix::dense::RowMajorMatrix;
use z00z_crypto::{CheckpointSha256BlockStreamV2, CheckpointShaRole};
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchAir, BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking,
    TableProver,
};

use super::plonky3_epoch_event_source_columns::EVENT_SOURCE_BYTE_BUS_V2;
use super::plonky3_epoch_sha256_columns::{
    SemanticShaJobKindV2, SEMANTIC_SHA_DIGEST_PAIR_BUS_V2, SEMANTIC_SHA_RAW_BYTE_BUS_V2,
};
use super::plonky3_epoch_transition_air::TRANSITION_FLOW_ROOT_LIMB_BUS_V2;
use super::plonky3_epoch_uniqueness_air::RANGE_BUS_V2;
use super::{
    Plonky3StarkConfigV2, RecursiveTraceOpcodeV2, EPOCH_CHUNK_BYTES_V2,
    EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2, NET_MERGE_BYTES_V2, PLONKY3_EVENT_VECTOR_MAGIC_V2,
    UNIQUENESS_CHALLENGE_BYTES_V2, UNIQUENESS_PRECOMMIT_BYTES_V2, UNIQUENESS_PRECOMMIT_VERSION_V2,
};
use crate::checkpoint::recursive_semantics::{NetEffectKindV2, UNIQUENESS_PRECOMMIT_LABEL_V2};
use crate::checkpoint::recursive_trace::STRUCTURAL_EVENT_HASH_LABEL_V2;

const TRANSITION_NPO_ID_V2: &str = "z00z/plonky3/epoch-semantic-source-transition/v2";
const UNIQUENESS_NPO_ID_V2: &str = "z00z/plonky3/epoch-semantic-source-uniqueness/v2";

pub(super) const SOURCE_TYPED_PAYLOAD_BYTE_BUS_V2: &str =
    "z00z/plonky3/epoch-source-typed-payload-byte/v2";
pub(super) const SOURCE_JMT_PAYLOAD_BYTE_BUS_V2: &str =
    "z00z/plonky3/epoch-source-jmt-payload-byte/v2";
pub(super) const SOURCE_UNIQUENESS_PAYLOAD_BYTE_BUS_V2: &str =
    "z00z/plonky3/epoch-source-uniqueness-payload-byte/v2";
pub(super) const SOURCE_REPLAY_SEMANTIC_BYTE_BUS_V2: &str =
    "z00z/plonky3/epoch-source-replay-semantic-byte/v2";
pub(super) const SOURCE_NET_EFFECT_BYTE_BUS_V2: &str =
    "z00z/plonky3/epoch-source-net-effect-byte/v2";
pub(super) const SOURCE_NET_MUTATION_BYTE_BUS_V2: &str =
    "z00z/plonky3/epoch-source-net-mutation-byte/v2";
const SOURCE_BEGIN_FINALIZE_BYTE_BUS_V2: &str = "z00z/plonky3/epoch-source-begin-finalize-byte/v2";
const SOURCE_REPLAY_OBJECT_BYTE_BUS_V2: &str = "z00z/plonky3/epoch-source-replay-object-byte/v2";
const SOURCE_PRECOMMIT_CHALLENGE_BYTE_BUS_V2: &str =
    "z00z/plonky3/epoch-source-precommit-challenge-byte/v2";
const SOURCE_CHALLENGE_CLOSE_BYTE_BUS_V2: &str =
    "z00z/plonky3/epoch-source-challenge-close-byte/v2";
const SOURCE_BINDING_DIGEST_BYTE_BUS_V2: &str = "z00z/plonky3/epoch-source-binding-digest-byte/v2";

pub(super) const TRANSITION_SLOTS_V2: usize = EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 as usize;
pub(super) const STATEMENT_LIMBS_V2: usize = EPOCH_CHUNK_BYTES_V2 / core::mem::size_of::<u16>();
pub(super) const PUBLIC_ACTIVE_OFFSET_V2: usize = STATEMENT_LIMBS_V2;
pub(super) const PUBLIC_SOURCE_LEN_OFFSET_V2: usize = PUBLIC_ACTIVE_OFFSET_V2 + TRANSITION_SLOTS_V2;
pub(super) const PUBLIC_EVENT_COUNT_OFFSET_V2: usize =
    PUBLIC_SOURCE_LEN_OFFSET_V2 + TRANSITION_SLOTS_V2;
pub(super) const PUBLIC_EVENT_COUNT_BYTE_OFFSET_V2: usize =
    PUBLIC_EVENT_COUNT_OFFSET_V2 + TRANSITION_SLOTS_V2;
pub(super) const PUBLIC_PRE_UNIQUENESS_BYTE_OFFSET_V2: usize =
    PUBLIC_EVENT_COUNT_BYTE_OFFSET_V2 + TRANSITION_SLOTS_V2 * 8;
pub(super) const PUBLIC_SPENT_PRECOMMIT_BYTE_OFFSET_V2: usize =
    PUBLIC_PRE_UNIQUENESS_BYTE_OFFSET_V2 + TRANSITION_SLOTS_V2 * 32;
pub(super) const PUBLIC_OUTPUT_PRECOMMIT_BYTE_OFFSET_V2: usize =
    PUBLIC_SPENT_PRECOMMIT_BYTE_OFFSET_V2 + TRANSITION_SLOTS_V2 * 32;
pub(super) const PUBLIC_EVENT_VECTOR_DIGEST_BYTE_OFFSET_V2: usize =
    PUBLIC_OUTPUT_PRECOMMIT_BYTE_OFFSET_V2 + TRANSITION_SLOTS_V2 * 32;
pub(super) const PUBLIC_SLICE_START_OFFSET_V2: usize =
    PUBLIC_EVENT_VECTOR_DIGEST_BYTE_OFFSET_V2 + TRANSITION_SLOTS_V2 * 32;
pub(super) const PUBLIC_SLICE_LEN_OFFSET_V2: usize = PUBLIC_SLICE_START_OFFSET_V2 + 1;
pub(super) const PUBLIC_FIELDS_V2: usize = PUBLIC_SLICE_LEN_OFFSET_V2 + 1;

pub(super) const ACTIVE_OFFSET_V2: usize = 0;
pub(super) const TRANSITION_SELECTOR_OFFSET_V2: usize = ACTIVE_OFFSET_V2 + 1;
pub(super) const PREFIX_SELECTOR_OFFSET_V2: usize =
    TRANSITION_SELECTOR_OFFSET_V2 + TRANSITION_SLOTS_V2;
pub(super) const PREFIX_BYTES_V2: usize = 16;
pub(super) const LENGTH_SELECTOR_OFFSET_V2: usize = PREFIX_SELECTOR_OFFSET_V2 + PREFIX_BYTES_V2;
pub(super) const LENGTH_BYTES_V2: usize = 4;
pub(super) const HEADER_SELECTOR_OFFSET_V2: usize = LENGTH_SELECTOR_OFFSET_V2 + LENGTH_BYTES_V2;
pub(super) const HEADER_BYTES_V2: usize = 45;
pub(super) const PAYLOAD_ACTIVE_OFFSET_V2: usize = HEADER_SELECTOR_OFFSET_V2 + HEADER_BYTES_V2;
pub(super) const BYTE_INDEX_OFFSET_V2: usize = PAYLOAD_ACTIVE_OFFSET_V2 + 1;
pub(super) const BYTE_OFFSET_V2: usize = BYTE_INDEX_OFFSET_V2 + 1;
pub(super) const BYTE_BITS_OFFSET_V2: usize = BYTE_OFFSET_V2 + 1;
pub(super) const EVENT_INDEX_OFFSET_V2: usize = BYTE_BITS_OFFSET_V2 + 8;
pub(super) const EVENT_LEN_BYTE_OFFSET_V2: usize = EVENT_INDEX_OFFSET_V2 + 1;
pub(super) const PAYLOAD_LEN_BYTE_OFFSET_V2: usize = EVENT_LEN_BYTE_OFFSET_V2 + LENGTH_BYTES_V2;
pub(super) const ORDINAL_BYTE_OFFSET_V2: usize = PAYLOAD_LEN_BYTE_OFFSET_V2 + LENGTH_BYTES_V2;
pub(super) const ORDINAL_BYTES_V2: usize = 8;
pub(super) const OPCODE_SELECTOR_OFFSET_V2: usize = ORDINAL_BYTE_OFFSET_V2 + ORDINAL_BYTES_V2;
pub(super) const OPCODE_COUNT_V2: usize = 17;
pub(super) const PAYLOAD_INDEX_OFFSET_V2: usize = OPCODE_SELECTOR_OFFSET_V2 + OPCODE_COUNT_V2;
pub(super) const PAYLOAD_NONZERO_OFFSET_V2: usize = PAYLOAD_INDEX_OFFSET_V2 + 1;
pub(super) const PAYLOAD_INVERSE_OFFSET_V2: usize = PAYLOAD_NONZERO_OFFSET_V2 + 1;
pub(super) const PAYLOAD_FINAL_OFFSET_V2: usize = PAYLOAD_INVERSE_OFFSET_V2 + 1;
pub(super) const SLOT_END_OFFSET_V2: usize = PAYLOAD_FINAL_OFFSET_V2 + 1;
pub(super) const PAYLOAD_PREFIX_SELECTOR_OFFSET_V2: usize = SLOT_END_OFFSET_V2 + 1;
pub(super) const PAYLOAD_PREFIX_BYTES_V2: usize = 9;
pub(super) const JMT_STAGE_SELECTOR_OFFSET_V2: usize =
    PAYLOAD_PREFIX_SELECTOR_OFFSET_V2 + PAYLOAD_PREFIX_BYTES_V2;
pub(super) const JMT_STAGE_COUNT_V2: usize = 3;
pub(super) const JMT_COUNT_OFFSET_V2: usize = JMT_STAGE_SELECTOR_OFFSET_V2 + JMT_STAGE_COUNT_V2;
pub(super) const UNIQUENESS_COUNTER_OFFSET_V2: usize = JMT_COUNT_OFFSET_V2 + 1;
pub(super) const UNIQUENESS_COUNTER_COUNT_V2: usize = 7;
pub(super) const UNIQUENESS_CLASS_SELECTOR_OFFSET_V2: usize =
    UNIQUENESS_COUNTER_OFFSET_V2 + UNIQUENESS_COUNTER_COUNT_V2;
pub(super) const UNIQUENESS_CLASS_COUNT_V2: usize = 8;
pub(super) const NET_KIND_SELECTOR_OFFSET_V2: usize =
    UNIQUENESS_CLASS_SELECTOR_OFFSET_V2 + UNIQUENESS_CLASS_COUNT_V2;
pub(super) const NET_KIND_COUNT_V2: usize = 5;
pub(super) const NET_TERMINAL_PAYLOAD_START_V2: usize = 38;
pub(super) const FLOW_HEADER_BYTES_V2: usize = 284;
pub(super) const FLOW_HEADER_COUNT_PAYLOAD_START_V2: usize =
    FLOW_HEADER_BYTES_V2 - DECLARED_COUNT_BYTE_COUNT_V2;
pub(super) const DECLARED_ITEM_LIMIT_V2: usize = 16_000;
pub(super) const NET_EFFECT_COUNTER_OFFSET_V2: usize =
    NET_KIND_SELECTOR_OFFSET_V2 + NET_KIND_COUNT_V2;
pub(super) const NET_MUTATION_COUNTER_OFFSET_V2: usize = NET_EFFECT_COUNTER_OFFSET_V2 + 1;
pub(super) const NET_TERMINAL_LIMB_OFFSET_V2: usize = NET_MUTATION_COUNTER_OFFSET_V2 + 1;
pub(super) const NET_TERMINAL_LIMB_COUNT_V2: usize = 16;
pub(super) const NET_TERMINAL_BYTE_SELECTOR_OFFSET_V2: usize =
    NET_TERMINAL_LIMB_OFFSET_V2 + NET_TERMINAL_LIMB_COUNT_V2;
pub(super) const NET_TERMINAL_BYTE_SELECTOR_COUNT_V2: usize = 32;
pub(super) const NET_TERMINAL_COUNTDOWN_OFFSET_V2: usize =
    NET_TERMINAL_BYTE_SELECTOR_OFFSET_V2 + NET_TERMINAL_BYTE_SELECTOR_COUNT_V2;
pub(super) const NET_TERMINAL_COUNTDOWN_INVERSE_OFFSET_V2: usize =
    NET_TERMINAL_COUNTDOWN_OFFSET_V2 + 1;
pub(super) const NET_TERMINAL_COUNTDOWN_ACTIVE_OFFSET_V2: usize =
    NET_TERMINAL_COUNTDOWN_INVERSE_OFFSET_V2 + 1;
pub(super) const FIXED_EVENT_COUNTER_OFFSET_V2: usize = NET_TERMINAL_COUNTDOWN_ACTIVE_OFFSET_V2 + 1;
pub(super) const FIXED_EVENT_COUNTER_COUNT_V2: usize = 6;
pub(super) const DECLARED_COUNT_BYTE_OFFSET_V2: usize =
    FIXED_EVENT_COUNTER_OFFSET_V2 + FIXED_EVENT_COUNTER_COUNT_V2;
pub(super) const DECLARED_COUNT_BYTE_COUNT_V2: usize = 8;
pub(super) const GLOBAL_PRODUCT_PAIR_OFFSET_V2: usize =
    DECLARED_COUNT_BYTE_OFFSET_V2 + DECLARED_COUNT_BYTE_COUNT_V2;
pub(super) const FLOW_COUNT_BYTE_SELECTOR_OFFSET_V2: usize = GLOBAL_PRODUCT_PAIR_OFFSET_V2 + 1;
pub(super) const FLOW_COUNT_BYTE_SELECTOR_COUNT_V2: usize = DECLARED_COUNT_BYTE_COUNT_V2;
pub(super) const FLOW_COUNT_COUNTDOWN_OFFSET_V2: usize =
    FLOW_COUNT_BYTE_SELECTOR_OFFSET_V2 + FLOW_COUNT_BYTE_SELECTOR_COUNT_V2;
pub(super) const FLOW_COUNT_COUNTDOWN_INVERSE_OFFSET_V2: usize = FLOW_COUNT_COUNTDOWN_OFFSET_V2 + 1;
pub(super) const FLOW_COUNT_COUNTDOWN_ACTIVE_OFFSET_V2: usize =
    FLOW_COUNT_COUNTDOWN_INVERSE_OFFSET_V2 + 1;
pub(super) const NET_MUTATION_NONZERO_OFFSET_V2: usize = FLOW_COUNT_COUNTDOWN_ACTIVE_OFFSET_V2 + 1;
pub(super) const NET_MUTATION_INVERSE_OFFSET_V2: usize = NET_MUTATION_NONZERO_OFFSET_V2 + 1;
pub(super) const DECLARED_SPENT_INVERSE_OFFSET_V2: usize = NET_MUTATION_INVERSE_OFFSET_V2 + 1;
pub(super) const DECLARED_OUTPUT_INVERSE_OFFSET_V2: usize = DECLARED_SPENT_INVERSE_OFFSET_V2 + 1;
pub(super) const DECLARED_COUNT_SLACK_BYTE_OFFSET_V2: usize = DECLARED_OUTPUT_INVERSE_OFFSET_V2 + 1;
pub(super) const DECLARED_COUNT_SLACK_BYTE_COUNT_V2: usize = 4;
pub(super) const REPLAY_PHASE_SELECTOR_OFFSET_V2: usize =
    DECLARED_COUNT_SLACK_BYTE_OFFSET_V2 + DECLARED_COUNT_SLACK_BYTE_COUNT_V2;
pub(super) const REPLAY_PHASE_COUNT_V2: usize = 14;
pub(super) const REPLAY_OP_KIND_PHASE_V2: usize = 0;
pub(super) const REPLAY_TX_LEN_LOW_PHASE_V2: usize = 1;
pub(super) const REPLAY_TX_LEN_HIGH_PHASE_V2: usize = 2;
pub(super) const REPLAY_TX_BYTES_PHASE_V2: usize = 3;
pub(super) const REPLAY_DEFINITION_LEN_LOW_PHASE_V2: usize = 4;
pub(super) const REPLAY_DEFINITION_LEN_HIGH_PHASE_V2: usize = 5;
pub(super) const REPLAY_DEFINITION_HEX_PHASE_V2: usize = 6;
pub(super) const REPLAY_SERIAL_PHASE_V2: usize = 7;
pub(super) const REPLAY_TERMINAL_LEN_LOW_PHASE_V2: usize = 8;
pub(super) const REPLAY_TERMINAL_LEN_HIGH_PHASE_V2: usize = 9;
pub(super) const REPLAY_TERMINAL_HEX_PHASE_V2: usize = 10;
pub(super) const REPLAY_LEAF_HASH_PHASE_V2: usize = 11;
pub(super) const REPLAY_LEAF_KIND_PHASE_V2: usize = 12;
pub(super) const REPLAY_FLAGS_PHASE_V2: usize = 13;
pub(super) const REPLAY_REMAINING_OFFSET_V2: usize =
    REPLAY_PHASE_SELECTOR_OFFSET_V2 + REPLAY_PHASE_COUNT_V2;
pub(super) const REPLAY_REMAINING_INVERSE_OFFSET_V2: usize = REPLAY_REMAINING_OFFSET_V2 + 1;
pub(super) const REPLAY_PHASE_FINAL_OFFSET_V2: usize = REPLAY_REMAINING_INVERSE_OFFSET_V2 + 1;
pub(super) const REPLAY_TX_LEN_LOW_OFFSET_V2: usize = REPLAY_PHASE_FINAL_OFFSET_V2 + 1;
pub(super) const REPLAY_TX_LEN_HIGH_OFFSET_V2: usize = REPLAY_TX_LEN_LOW_OFFSET_V2 + 1;
pub(super) const REPLAY_TX_LEN_INVERSE_OFFSET_V2: usize = REPLAY_TX_LEN_HIGH_OFFSET_V2 + 1;
pub(super) const REPLAY_HEX_SELECTOR_OFFSET_V2: usize = REPLAY_TX_LEN_INVERSE_OFFSET_V2 + 1;
pub(super) const REPLAY_HEX_SELECTOR_COUNT_V2: usize = 16;
pub(super) const REPLAY_HEX_LOW_OFFSET_V2: usize =
    REPLAY_HEX_SELECTOR_OFFSET_V2 + REPLAY_HEX_SELECTOR_COUNT_V2;
pub(super) const REPLAY_HEX_HIGH_NIBBLE_OFFSET_V2: usize = REPLAY_HEX_LOW_OFFSET_V2 + 1;
pub(super) const REPLAY_ASCII_LOW_OFFSET_V2: usize = REPLAY_HEX_HIGH_NIBBLE_OFFSET_V2 + 1;
pub(super) const REPLAY_ASCII_HIGH_OFFSET_V2: usize = REPLAY_ASCII_LOW_OFFSET_V2 + 1;
pub(super) const REPLAY_SEMANTIC_INDEX_OFFSET_V2: usize = REPLAY_ASCII_HIGH_OFFSET_V2 + 1;
pub(super) const REPLAY_COUNTER_OFFSET_V2: usize = REPLAY_SEMANTIC_INDEX_OFFSET_V2 + 1;
pub(super) const REPLAY_COUNTER_COUNT_V2: usize = 2;
pub(super) const FLOW_PHASE_SELECTOR_OFFSET_V2: usize =
    REPLAY_COUNTER_OFFSET_V2 + REPLAY_COUNTER_COUNT_V2;
pub(super) const FLOW_PHASE_COUNT_V2: usize = 14;
pub(super) const FLOW_HEX_SELECTOR_OFFSET_V2: usize =
    FLOW_PHASE_SELECTOR_OFFSET_V2 + FLOW_PHASE_COUNT_V2;
pub(super) const FLOW_HEX_SELECTOR_COUNT_V2: usize = 16;
pub(super) const FLOW_HEX_LOW_OFFSET_V2: usize =
    FLOW_HEX_SELECTOR_OFFSET_V2 + FLOW_HEX_SELECTOR_COUNT_V2;
pub(super) const FLOW_HEX_HIGH_NIBBLE_OFFSET_V2: usize = FLOW_HEX_LOW_OFFSET_V2 + 1;
pub(super) const FLOW_HEX_BYTE_INDEX_OFFSET_V2: usize = FLOW_HEX_HIGH_NIBBLE_OFFSET_V2 + 1;
pub(super) const FLOW_ROOT_LIMB_INDEX_OFFSET_V2: usize = FLOW_HEX_BYTE_INDEX_OFFSET_V2 + 1;
pub(super) const FLOW_ROOT_BYTE_PARITY_OFFSET_V2: usize = FLOW_ROOT_LIMB_INDEX_OFFSET_V2 + 1;
pub(super) const FLOW_ROOT_LOW_BYTE_OFFSET_V2: usize = FLOW_ROOT_BYTE_PARITY_OFFSET_V2 + 1;
pub(super) const TRANSCRIPT_PHASE_SELECTOR_OFFSET_V2: usize = FLOW_ROOT_LOW_BYTE_OFFSET_V2 + 1;
pub(super) const TRANSCRIPT_PHASE_COUNT_V2: usize = 27;
pub(super) const TRANSCRIPT_PHASE_FINAL_OFFSET_V2: usize =
    TRANSCRIPT_PHASE_SELECTOR_OFFSET_V2 + TRANSCRIPT_PHASE_COUNT_V2;
pub(super) const TRANSCRIPT_PHASE_END_INVERSE_OFFSET_V2: usize =
    TRANSCRIPT_PHASE_FINAL_OFFSET_V2 + 1;
pub(super) const TRANSCRIPT_PAIR_INDEX_OFFSET_V2: usize =
    TRANSCRIPT_PHASE_END_INVERSE_OFFSET_V2 + 1;
pub(super) const TRANSCRIPT_PAIR_SECOND_OFFSET_V2: usize = TRANSCRIPT_PAIR_INDEX_OFFSET_V2 + 1;
pub(super) const ROW_FIELDS_V2: usize = TRANSCRIPT_PAIR_SECOND_OFFSET_V2 + 1;
pub(super) const MIN_ROWS_V2: usize = 32;

pub(super) const PRECOMMIT_VERSION_PHASE_V2: usize = 0;
pub(super) const PRECOMMIT_SPENT_ORIGINAL_PHASE_V2: usize = 3;
pub(super) const PRECOMMIT_SPENT_SORTED_PHASE_V2: usize = 4;
pub(super) const PRECOMMIT_OUTPUT_ORIGINAL_PHASE_V2: usize = 5;
pub(super) const PRECOMMIT_OUTPUT_SORTED_PHASE_V2: usize = 6;
pub(super) const PRECOMMIT_DIGEST_PHASE_V2: usize = 7;
pub(super) const CHALLENGE_VERSION_PHASE_V2: usize = 8;
pub(super) const CHALLENGE_PRECOMMIT_PHASE_V2: usize = 9;
pub(super) const CHALLENGE_CONTEXT_PHASE_V2: usize = 10;
pub(super) const CHALLENGE_SPENT_PRECOMMIT_PHASE_V2: usize = 11;
pub(super) const CHALLENGE_OUTPUT_PRECOMMIT_PHASE_V2: usize = 12;
pub(super) const CHALLENGE_DIGEST_FIRST_PHASE_V2: usize = 13;
pub(super) const CHALLENGE_DIGEST_LAST_PHASE_V2: usize = 20;
pub(super) const CLOSE_HEADER_PHASE_V2: usize = 21;
pub(super) const CLOSE_PRECOMMIT_PHASE_V2: usize = 22;
pub(super) const CLOSE_CONTEXT_PHASE_V2: usize = 24;
pub(super) const CLOSE_SPENT_PRECOMMIT_PHASE_V2: usize = 25;
pub(super) const CLOSE_OUTPUT_PRECOMMIT_PHASE_V2: usize = 26;
pub(super) const TRANSCRIPT_PHASE_STARTS_V2: [usize; TRANSCRIPT_PHASE_COUNT_V2] = [
    0, 1, 5, 9, 41, 73, 105, 137, 0, 1, 33, 65, 97, 129, 161, 193, 225, 257, 289, 321, 353, 0, 2,
    34, 38, 70, 102,
];
pub(super) const TRANSCRIPT_PHASE_ENDS_V2: [usize; TRANSCRIPT_PHASE_COUNT_V2] = [
    0, 4, 8, 40, 72, 104, 136, 168, 0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 1,
    33, 37, 69, 101, 133,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SemanticSourceAirRoleV2 {
    Transition,
    Uniqueness,
}

impl SemanticSourceAirRoleV2 {
    pub(super) fn npo_type(self) -> NpoTypeId {
        NpoTypeId::new(match self {
            Self::Transition => TRANSITION_NPO_ID_V2,
            Self::Uniqueness => UNIQUENESS_NPO_ID_V2,
        })
    }
}

#[derive(Clone, Debug)]
pub(super) struct SemanticSourceRowV2 {
    pub(super) values: Vec<KoalaBear>,
}

#[derive(Clone, Debug)]
pub(super) struct SemanticSourceTraceV2 {
    pub(super) role: SemanticSourceAirRoleV2,
    pub(super) public_values: Vec<KoalaBear>,
    pub(super) rows: Vec<SemanticSourceRowV2>,
}

impl NonPrimitiveTrace<KoalaBear> for SemanticSourceTraceV2 {
    fn op_type(&self) -> NpoTypeId {
        self.role.npo_type()
    }

    fn rows(&self) -> usize {
        self.rows.len()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed_clone(&self) -> Box<dyn NonPrimitiveTrace<KoalaBear>> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Copy, Debug)]
struct SemanticSourceAirV2 {
    role: SemanticSourceAirRoleV2,
}

impl SemanticSourceAirV2 {
    const fn new(role: SemanticSourceAirRoleV2) -> Self {
        Self { role }
    }

    fn trace_to_matrix(rows: &[SemanticSourceRowV2]) -> RowMajorMatrix<KoalaBear> {
        RowMajorMatrix::new(
            rows.iter()
                .flat_map(|row| row.values.iter().copied())
                .collect(),
            ROW_FIELDS_V2,
        )
    }
}

impl<F: Field> BaseAir<F> for SemanticSourceAirV2 {
    fn width(&self) -> usize {
        ROW_FIELDS_V2
    }

    fn num_public_values(&self) -> usize {
        PUBLIC_FIELDS_V2
    }

    fn main_next_row_columns(&self) -> Vec<usize> {
        (0..ROW_FIELDS_V2).collect()
    }
}

fn field<AB: AirBuilder>(row: &[AB::Var], offset: usize) -> AB::Expr {
    row[offset].into()
}

fn fields<AB: AirBuilder>(row: &[AB::Var], offset: usize, count: usize) -> Vec<AB::Expr> {
    (0..count)
        .map(|index| field::<AB>(row, offset + index))
        .collect()
}

fn replay_terminal_leaf_kind_polynomial<R: PrimeCharacteristicRing>(byte: R) -> R {
    // The accepted transition predicate narrows the codec enum to Terminal=1
    // for both replay directions.
    byte - R::ONE
}

fn global_schedule_polynomials<R: PrimeCharacteristicRing>(
    current_spent: R,
    current_output: R,
    current_pair: R,
    current_net: R,
    next_spent: R,
    next_output: R,
    next_close: R,
    next_delete: R,
    next_insert: R,
    next_replace: R,
    next_unchanged: R,
) -> [R; 4] {
    let one = R::ONE;
    [
        current_spent.clone() * (next_output.clone() + next_delete - one.clone()),
        current_output.clone() * (next_insert - (one.clone() - current_pair.clone())),
        current_output * (next_replace + next_unchanged - current_pair),
        current_net * (next_spent + next_output + next_close - one),
    ]
}

fn global_pair_transition_polynomial<R: PrimeCharacteristicRing>(
    event_continues: R,
    event_final: R,
    current_pair: R,
    current_spent: R,
    next_output: R,
    next_pair: R,
) -> R {
    next_pair - event_continues * current_pair - event_final * current_spent * next_output
}

fn global_schedule_entry_polynomial<R: PrimeCharacteristicRing>(
    current_product_or_net: R,
    next_net_effect: R,
) -> R {
    (R::ONE - current_product_or_net) * next_net_effect
}

fn nonzero_indicator_polynomials<R: PrimeCharacteristicRing>(
    indicator: R,
    value: R,
    inverse: R,
) -> [R; 3] {
    let one_minus_indicator = R::ONE - indicator.clone();
    [
        value.clone() * inverse.clone() - indicator,
        value * one_minus_indicator.clone(),
        inverse * one_minus_indicator,
    ]
}

fn bounded_count_polynomial<R: PrimeCharacteristicRing>(count: R, slack: R) -> R {
    count + slack - R::from_usize(DECLARED_ITEM_LIMIT_V2)
}

fn push_semantic_raw_byte<AB>(
    builder: &mut AB,
    transition: AB::Expr,
    kind: SemanticShaJobKindV2,
    id: AB::Expr,
    index: AB::Expr,
    byte: AB::Expr,
    gate: AB::Expr,
) where
    AB: AirBuilder + InteractionBuilder,
{
    builder.push_interaction(
        SEMANTIC_SHA_RAW_BYTE_BUS_V2,
        vec![
            transition,
            AB::Expr::from_u64(u64::from(kind as u8)),
            id,
            index,
            byte,
        ],
        Count::bounded(gate, 1),
    );
}

fn push_semantic_digest_pair<AB>(
    builder: &mut AB,
    transition: AB::Expr,
    kind: SemanticShaJobKindV2,
    id: AB::Expr,
    pair: AB::Expr,
    value: AB::Expr,
    gate: AB::Expr,
) where
    AB: AirBuilder + InteractionBuilder,
{
    builder.push_interaction(
        SEMANTIC_SHA_DIGEST_PAIR_BUS_V2,
        vec![
            transition,
            AB::Expr::from_u64(u64::from(kind as u8)),
            id,
            pair,
            value,
        ],
        -Count::bounded(gate, 1),
    );
}

fn semantic_part_body_offsets(role: CheckpointShaRole, part_lengths: &[usize]) -> Vec<usize> {
    let mut cursor = CheckpointSha256BlockStreamV2::framed_role_prefix(role).len();
    part_lengths
        .iter()
        .map(|part_len| {
            cursor = cursor.checked_add(8).expect("fixed SHA framing fits usize");
            let body = cursor;
            cursor = cursor
                .checked_add(*part_len)
                .expect("fixed SHA framing fits usize");
            body
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn push_semantic_constant_bytes<AB>(
    builder: &mut AB,
    transition: AB::Expr,
    kind: SemanticShaJobKindV2,
    id: AB::Expr,
    start: usize,
    bytes: &[u8],
    gate: AB::Expr,
) where
    AB: AirBuilder + InteractionBuilder,
{
    for (offset, byte) in bytes.iter().copied().enumerate() {
        push_semantic_raw_byte(
            builder,
            transition.clone(),
            kind,
            id.clone(),
            AB::Expr::from_usize(start + offset),
            AB::Expr::from_u64(u64::from(byte)),
            gate.clone(),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn push_semantic_fixed_framing<AB>(
    builder: &mut AB,
    transition: AB::Expr,
    kind: SemanticShaJobKindV2,
    id: AB::Expr,
    role: CheckpointShaRole,
    part_lengths: &[usize],
    gate: AB::Expr,
) -> Vec<usize>
where
    AB: AirBuilder + InteractionBuilder,
{
    let prefix = CheckpointSha256BlockStreamV2::framed_role_prefix(role);
    let bodies = semantic_part_body_offsets(role, part_lengths);
    push_semantic_constant_bytes(
        builder,
        transition.clone(),
        kind,
        id.clone(),
        0,
        &prefix,
        gate.clone(),
    );
    for (body, part_len) in bodies.iter().copied().zip(part_lengths.iter().copied()) {
        push_semantic_constant_bytes(
            builder,
            transition.clone(),
            kind,
            id.clone(),
            body - 8,
            &u64::try_from(part_len)
                .expect("fixed semantic SHA part length fits u64")
                .to_le_bytes(),
            gate.clone(),
        );
    }
    bodies
}

impl<AB> Air<AB> for SemanticSourceAirV2
where
    AB: AirBuilder + InteractionBuilder,
    AB::F: Field,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.current_slice();
        let next = main.next_slice();
        let public = builder
            .public_values()
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<AB::Expr>>();
        let one = AB::Expr::ONE;

        let active = field::<AB>(local, ACTIVE_OFFSET_V2);
        let next_active = field::<AB>(next, ACTIVE_OFFSET_V2);
        builder.assert_bool(active.clone());
        builder.when_first_row().assert_one(active.clone());
        builder.when_last_row().assert_zero(active.clone());

        let public_active = (0..TRANSITION_SLOTS_V2)
            .map(|slot| public[PUBLIC_ACTIVE_OFFSET_V2 + slot].clone())
            .collect::<Vec<_>>();
        for flag in &public_active {
            builder.assert_bool(flag.clone());
        }
        builder.assert_one(public_active[0].clone());
        for slot in 1..TRANSITION_SLOTS_V2 {
            builder.assert_zero(
                public_active[slot].clone() * (one.clone() - public_active[slot - 1].clone()),
            );
        }
        builder.assert_eq(
            public[PUBLIC_SLICE_LEN_OFFSET_V2].clone(),
            public_active
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, flag| sum + flag),
        );
        for slot in 0..TRANSITION_SLOTS_V2 {
            let bytes = &public[PUBLIC_EVENT_COUNT_BYTE_OFFSET_V2 + slot * 8
                ..PUBLIC_EVENT_COUNT_BYTE_OFFSET_V2 + (slot + 1) * 8];
            let count = bytes[..4]
                .iter()
                .enumerate()
                .fold(AB::Expr::ZERO, |sum, (index, byte)| {
                    sum + byte.clone() * AB::Expr::from_u64(1_u64 << (index * 8))
                });
            builder.assert_eq(public[PUBLIC_EVENT_COUNT_OFFSET_V2 + slot].clone(), count);
            for byte in &bytes[4..] {
                builder.assert_zero(byte.clone());
            }
        }

        let transition_selectors =
            fields::<AB>(local, TRANSITION_SELECTOR_OFFSET_V2, TRANSITION_SLOTS_V2);
        let next_transition_selectors =
            fields::<AB>(next, TRANSITION_SELECTOR_OFFSET_V2, TRANSITION_SLOTS_V2);
        for (slot, selector) in transition_selectors.iter().enumerate() {
            builder.assert_bool(selector.clone());
            builder.assert_zero(selector.clone() * (one.clone() - public_active[slot].clone()));
        }
        builder.assert_eq(
            transition_selectors
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value),
            active.clone(),
        );
        let transition_index = transition_selectors
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (slot, selector)| {
                sum + selector.clone() * AB::Expr::from_usize(slot)
            });
        let transition_index = transition_index + public[PUBLIC_SLICE_START_OFFSET_V2].clone();
        let selected_public = |offset: usize| {
            transition_selectors
                .iter()
                .enumerate()
                .fold(AB::Expr::ZERO, |sum, (slot, selector)| {
                    sum + selector.clone() * public[offset + slot].clone()
                })
        };
        let selected_digest_byte = |offset: usize, byte_index: usize| {
            transition_selectors
                .iter()
                .enumerate()
                .fold(AB::Expr::ZERO, |sum, (slot, selector)| {
                    sum + selector.clone() * public[offset + slot * 32 + byte_index].clone()
                })
        };

        let prefix = fields::<AB>(local, PREFIX_SELECTOR_OFFSET_V2, PREFIX_BYTES_V2);
        let next_prefix = fields::<AB>(next, PREFIX_SELECTOR_OFFSET_V2, PREFIX_BYTES_V2);
        let length = fields::<AB>(local, LENGTH_SELECTOR_OFFSET_V2, LENGTH_BYTES_V2);
        let next_length = fields::<AB>(next, LENGTH_SELECTOR_OFFSET_V2, LENGTH_BYTES_V2);
        let header = fields::<AB>(local, HEADER_SELECTOR_OFFSET_V2, HEADER_BYTES_V2);
        let next_header = fields::<AB>(next, HEADER_SELECTOR_OFFSET_V2, HEADER_BYTES_V2);
        let payload = field::<AB>(local, PAYLOAD_ACTIVE_OFFSET_V2);
        let next_payload = field::<AB>(next, PAYLOAD_ACTIVE_OFFSET_V2);
        for selector in prefix
            .iter()
            .chain(length.iter())
            .chain(header.iter())
            .chain(core::iter::once(&payload))
        {
            builder.assert_bool(selector.clone());
        }
        let prefix_active = prefix
            .iter()
            .cloned()
            .fold(AB::Expr::ZERO, |sum, value| sum + value);
        let length_active = length
            .iter()
            .cloned()
            .fold(AB::Expr::ZERO, |sum, value| sum + value);
        let header_active = header
            .iter()
            .cloned()
            .fold(AB::Expr::ZERO, |sum, value| sum + value);
        let event_active = length_active.clone() + header_active.clone() + payload.clone();
        builder.assert_eq(prefix_active.clone() + event_active.clone(), active.clone());

        let byte_index = field::<AB>(local, BYTE_INDEX_OFFSET_V2);
        let next_byte_index = field::<AB>(next, BYTE_INDEX_OFFSET_V2);
        let byte = field::<AB>(local, BYTE_OFFSET_V2);
        let next_byte = field::<AB>(next, BYTE_OFFSET_V2);
        let mut reconstructed_byte = AB::Expr::ZERO;
        for bit in 0..8 {
            let value = field::<AB>(local, BYTE_BITS_OFFSET_V2 + bit);
            builder.assert_bool(value.clone());
            reconstructed_byte += value * AB::Expr::from_u64(1_u64 << bit);
        }
        builder.assert_eq(byte.clone(), reconstructed_byte);
        builder.push_interaction(
            EVENT_SOURCE_BYTE_BUS_V2,
            vec![transition_index.clone(), byte_index.clone(), byte.clone()],
            Count::bounded(active.clone(), 1),
        );

        let event_index = field::<AB>(local, EVENT_INDEX_OFFSET_V2);
        let next_event_index = field::<AB>(next, EVENT_INDEX_OFFSET_V2);
        let event_len_bytes = fields::<AB>(local, EVENT_LEN_BYTE_OFFSET_V2, LENGTH_BYTES_V2);
        let next_event_len_bytes = fields::<AB>(next, EVENT_LEN_BYTE_OFFSET_V2, LENGTH_BYTES_V2);
        let payload_len_bytes = fields::<AB>(local, PAYLOAD_LEN_BYTE_OFFSET_V2, LENGTH_BYTES_V2);
        let next_payload_len_bytes =
            fields::<AB>(next, PAYLOAD_LEN_BYTE_OFFSET_V2, LENGTH_BYTES_V2);
        let ordinal_bytes = fields::<AB>(local, ORDINAL_BYTE_OFFSET_V2, ORDINAL_BYTES_V2);
        let next_ordinal_bytes = fields::<AB>(next, ORDINAL_BYTE_OFFSET_V2, ORDINAL_BYTES_V2);
        let opcode_selectors = fields::<AB>(local, OPCODE_SELECTOR_OFFSET_V2, OPCODE_COUNT_V2);
        let next_opcode_selectors = fields::<AB>(next, OPCODE_SELECTOR_OFFSET_V2, OPCODE_COUNT_V2);
        for selector in &opcode_selectors {
            builder.assert_bool(selector.clone());
        }
        builder.assert_eq(
            opcode_selectors
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value),
            event_active.clone(),
        );
        let opcode = opcode_selectors
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (index, selector)| {
                sum + selector.clone() * AB::Expr::from_usize(index + 1)
            });

        for (index, selector) in length.iter().enumerate() {
            builder.assert_zero(selector.clone() * (byte.clone() - event_len_bytes[index].clone()));
        }
        builder.assert_zero(header[0].clone() * (byte.clone() - opcode.clone()));
        for index in 0..ORDINAL_BYTES_V2 {
            builder.assert_zero(
                header[index + 1].clone() * (byte.clone() - ordinal_bytes[index].clone()),
            );
        }
        for index in 0..LENGTH_BYTES_V2 {
            builder.assert_zero(
                header[41 + index].clone() * (byte.clone() - payload_len_bytes[index].clone()),
            );
        }

        let event_len = event_len_bytes[..3]
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (index, value)| {
                sum + value.clone() * AB::Expr::from_u64(1_u64 << (index * 8))
            });
        let payload_len = payload_len_bytes[..3]
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (index, value)| {
                sum + value.clone() * AB::Expr::from_u64(1_u64 << (index * 8))
            });
        builder.assert_zero(event_active.clone() * event_len_bytes[3].clone());
        builder.assert_zero(event_active.clone() * payload_len_bytes[3].clone());
        builder.assert_bool(event_active.clone() * payload_len_bytes[2].clone());
        builder.assert_zero(
            event_active.clone()
                * payload_len_bytes[2].clone()
                * (payload_len_bytes[0].clone()
                    + payload_len_bytes[1].clone() * AB::Expr::from_u64(256)),
        );
        builder.assert_zero(
            event_active.clone()
                * (event_len.clone() - payload_len.clone() - AB::Expr::from_u64(45)),
        );

        let ordinal = ordinal_bytes[..4]
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (index, value)| {
                sum + value.clone() * AB::Expr::from_u64(1_u64 << (index * 8))
            });
        for value in &ordinal_bytes[4..] {
            builder.assert_zero(event_active.clone() * value.clone());
        }
        builder.assert_zero(event_active.clone() * (event_index.clone() - ordinal));

        for index in 0..8 {
            let expected = if index < PLONKY3_EVENT_VECTOR_MAGIC_V2.len() {
                AB::Expr::from_u64(u64::from(PLONKY3_EVENT_VECTOR_MAGIC_V2[index]))
            } else {
                selected_public(PUBLIC_EVENT_COUNT_BYTE_OFFSET_V2 + index - 8)
            };
            builder.assert_zero(prefix[index].clone() * (byte.clone() - expected));
        }
        for index in 8..PREFIX_BYTES_V2 {
            let expected = transition_selectors.iter().enumerate().fold(
                AB::Expr::ZERO,
                |sum, (slot, selector)| {
                    sum + selector.clone()
                        * public[PUBLIC_EVENT_COUNT_BYTE_OFFSET_V2 + slot * 8 + index - 8].clone()
                },
            );
            builder.assert_zero(prefix[index].clone() * (byte.clone() - expected));
        }

        let payload_index = field::<AB>(local, PAYLOAD_INDEX_OFFSET_V2);
        let next_payload_index = field::<AB>(next, PAYLOAD_INDEX_OFFSET_V2);
        let payload_nonzero = field::<AB>(local, PAYLOAD_NONZERO_OFFSET_V2);
        let next_payload_nonzero = field::<AB>(next, PAYLOAD_NONZERO_OFFSET_V2);
        let payload_inverse = field::<AB>(local, PAYLOAD_INVERSE_OFFSET_V2);
        let next_payload_inverse = field::<AB>(next, PAYLOAD_INVERSE_OFFSET_V2);
        let payload_final = field::<AB>(local, PAYLOAD_FINAL_OFFSET_V2);
        let slot_end = field::<AB>(local, SLOT_END_OFFSET_V2);
        builder.assert_bool(payload_nonzero.clone());
        builder.assert_bool(payload_final.clone());
        builder.assert_bool(slot_end.clone());
        builder.assert_zero(prefix_active.clone() * payload_nonzero.clone());
        builder.assert_zero((active.clone() - payload.clone()) * payload_final.clone());
        builder.assert_zero(
            event_active.clone()
                * (payload_len.clone() * payload_inverse.clone() - payload_nonzero.clone()),
        );
        builder.assert_zero(
            event_active.clone() * (one.clone() - payload_nonzero.clone()) * payload_len.clone(),
        );
        builder.assert_zero(
            payload_final.clone() * (payload_index.clone() + one.clone() - payload_len.clone()),
        );
        builder.assert_zero((active.clone() - payload.clone()) * payload_index.clone());

        let event_final = header[HEADER_BYTES_V2 - 1].clone()
            * (one.clone() - payload_nonzero.clone())
            + payload.clone() * payload_final.clone();
        builder.assert_zero(slot_end.clone() * (one.clone() - event_final.clone()));
        builder.assert_zero(
            slot_end.clone()
                * (byte_index.clone() + one.clone() - selected_public(PUBLIC_SOURCE_LEN_OFFSET_V2)),
        );
        builder.assert_zero(
            slot_end.clone()
                * (event_index.clone() + one.clone()
                    - selected_public(PUBLIC_EVENT_COUNT_OFFSET_V2)),
        );

        let payload_prefix = fields::<AB>(
            local,
            PAYLOAD_PREFIX_SELECTOR_OFFSET_V2,
            PAYLOAD_PREFIX_BYTES_V2,
        );
        let next_payload_prefix = fields::<AB>(
            next,
            PAYLOAD_PREFIX_SELECTOR_OFFSET_V2,
            PAYLOAD_PREFIX_BYTES_V2,
        );
        for selector in &payload_prefix {
            builder.assert_bool(selector.clone());
        }
        let payload_prefix_sum = payload_prefix
            .iter()
            .cloned()
            .fold(AB::Expr::ZERO, |sum, value| sum + value);
        builder.assert_bool(payload_prefix_sum.clone());
        builder.assert_zero(payload_prefix_sum * (one.clone() - payload.clone()));

        let uniqueness_classes = fields::<AB>(
            local,
            UNIQUENESS_CLASS_SELECTOR_OFFSET_V2,
            UNIQUENESS_CLASS_COUNT_V2,
        );
        let next_uniqueness_classes = fields::<AB>(
            next,
            UNIQUENESS_CLASS_SELECTOR_OFFSET_V2,
            UNIQUENESS_CLASS_COUNT_V2,
        );
        for selector in &uniqueness_classes {
            builder.assert_bool(selector.clone());
        }
        let uniqueness_opcode =
            opcode_selectors[RecursiveTraceOpcodeV2::UniquenessSorted as usize - 1].clone();
        builder.assert_eq(
            uniqueness_classes
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value),
            uniqueness_opcode.clone(),
        );
        for (class, selector) in uniqueness_classes.iter().enumerate() {
            let pass = usize::from(class >= 4);
            let set = class % 2;
            let list = usize::from((2..4).contains(&class) || class >= 6);
            for (payload_byte, expected) in [
                (0, usize::from(UNIQUENESS_PRECOMMIT_VERSION_V2)),
                (1, pass),
                (2, set),
                (3, list),
            ] {
                builder.assert_zero(
                    selector.clone()
                        * payload_prefix[payload_byte].clone()
                        * (byte.clone() - AB::Expr::from_usize(expected)),
                );
            }
        }

        let net_kind_selectors =
            fields::<AB>(local, NET_KIND_SELECTOR_OFFSET_V2, NET_KIND_COUNT_V2);
        let next_net_kind_selectors =
            fields::<AB>(next, NET_KIND_SELECTOR_OFFSET_V2, NET_KIND_COUNT_V2);
        for selector in &net_kind_selectors {
            builder.assert_bool(selector.clone());
        }
        let net_opcode = opcode_selectors[RecursiveTraceOpcodeV2::NetMerge as usize - 1].clone();
        builder.assert_eq(
            net_kind_selectors
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value),
            net_opcode.clone(),
        );
        builder.assert_zero(
            net_opcode.clone() * (payload_len.clone() - AB::Expr::from_usize(NET_MERGE_BYTES_V2)),
        );
        for (kind, selector) in net_kind_selectors.iter().enumerate() {
            for (payload_byte, expected) in
                [(0, usize::from(UNIQUENESS_PRECOMMIT_VERSION_V2)), (1, kind)]
            {
                builder.assert_zero(
                    selector.clone()
                        * payload_prefix[payload_byte].clone()
                        * (byte.clone() - AB::Expr::from_usize(expected)),
                );
            }
        }
        let net_non_close = net_kind_selectors[1..]
            .iter()
            .cloned()
            .fold(AB::Expr::ZERO, |sum, value| sum + value);
        let net_mutation = net_kind_selectors[1..4]
            .iter()
            .cloned()
            .fold(AB::Expr::ZERO, |sum, value| sum + value);
        let net_effect_counter = field::<AB>(local, NET_EFFECT_COUNTER_OFFSET_V2);
        let next_net_effect_counter = field::<AB>(next, NET_EFFECT_COUNTER_OFFSET_V2);
        let net_mutation_counter = field::<AB>(local, NET_MUTATION_COUNTER_OFFSET_V2);
        let next_net_mutation_counter = field::<AB>(next, NET_MUTATION_COUNTER_OFFSET_V2);
        let net_terminal_limbs = fields::<AB>(
            local,
            NET_TERMINAL_LIMB_OFFSET_V2,
            NET_TERMINAL_LIMB_COUNT_V2,
        );
        let next_net_terminal_limbs = fields::<AB>(
            next,
            NET_TERMINAL_LIMB_OFFSET_V2,
            NET_TERMINAL_LIMB_COUNT_V2,
        );
        let net_terminal_byte_selectors = fields::<AB>(
            local,
            NET_TERMINAL_BYTE_SELECTOR_OFFSET_V2,
            NET_TERMINAL_BYTE_SELECTOR_COUNT_V2,
        );
        let next_net_terminal_byte_selectors = fields::<AB>(
            next,
            NET_TERMINAL_BYTE_SELECTOR_OFFSET_V2,
            NET_TERMINAL_BYTE_SELECTOR_COUNT_V2,
        );
        for selector in &net_terminal_byte_selectors {
            builder.assert_bool(selector.clone());
        }
        let net_terminal_byte_active = net_terminal_byte_selectors
            .iter()
            .cloned()
            .fold(AB::Expr::ZERO, |sum, value| sum + value);
        builder.assert_bool(net_terminal_byte_active.clone());
        builder.assert_zero(net_terminal_byte_active.clone() * (one.clone() - payload.clone()));
        builder.assert_zero(net_terminal_byte_active.clone() * (one.clone() - net_opcode.clone()));
        for (terminal_byte, selector) in net_terminal_byte_selectors.iter().enumerate() {
            builder.assert_zero(
                selector.clone()
                    * (payload_index.clone()
                        - AB::Expr::from_usize(NET_TERMINAL_PAYLOAD_START_V2 + terminal_byte)),
            );
            if terminal_byte.is_multiple_of(2) {
                builder.assert_zero(
                    selector.clone()
                        * (net_terminal_limbs[terminal_byte / 2].clone()
                            - byte.clone()
                            - next_byte.clone() * AB::Expr::from_u64(256)),
                );
            }
        }
        for limb in &net_terminal_limbs {
            builder.assert_zero((one.clone() - net_opcode.clone()) * limb.clone());
        }
        let net_terminal_countdown = field::<AB>(local, NET_TERMINAL_COUNTDOWN_OFFSET_V2);
        let next_net_terminal_countdown = field::<AB>(next, NET_TERMINAL_COUNTDOWN_OFFSET_V2);
        let net_terminal_countdown_inverse =
            field::<AB>(local, NET_TERMINAL_COUNTDOWN_INVERSE_OFFSET_V2);
        let net_terminal_countdown_active =
            field::<AB>(local, NET_TERMINAL_COUNTDOWN_ACTIVE_OFFSET_V2);
        let next_net_terminal_countdown_active =
            field::<AB>(next, NET_TERMINAL_COUNTDOWN_ACTIVE_OFFSET_V2);
        builder.assert_bool(net_terminal_countdown_active.clone());
        builder.assert_eq(
            net_terminal_countdown.clone() * net_terminal_countdown_inverse.clone(),
            net_terminal_countdown_active.clone(),
        );
        builder.assert_zero(
            (one.clone() - net_terminal_countdown_active.clone()) * net_terminal_countdown.clone(),
        );
        builder.assert_zero(
            (one.clone() - net_terminal_countdown_active.clone()) * net_terminal_countdown_inverse,
        );
        builder
            .assert_zero(net_terminal_countdown_active.clone() * (one.clone() - payload.clone()));
        builder.assert_zero(
            net_terminal_countdown_active.clone() * (one.clone() - net_opcode.clone()),
        );
        let fixed_event_counters = fields::<AB>(
            local,
            FIXED_EVENT_COUNTER_OFFSET_V2,
            FIXED_EVENT_COUNTER_COUNT_V2,
        );
        let next_fixed_event_counters = fields::<AB>(
            next,
            FIXED_EVENT_COUNTER_OFFSET_V2,
            FIXED_EVENT_COUNTER_COUNT_V2,
        );
        let declared_count_bytes = fields::<AB>(
            local,
            DECLARED_COUNT_BYTE_OFFSET_V2,
            DECLARED_COUNT_BYTE_COUNT_V2,
        );
        let next_declared_count_bytes = fields::<AB>(
            next,
            DECLARED_COUNT_BYTE_OFFSET_V2,
            DECLARED_COUNT_BYTE_COUNT_V2,
        );
        let global_product_pair = field::<AB>(local, GLOBAL_PRODUCT_PAIR_OFFSET_V2);
        let next_global_product_pair = field::<AB>(next, GLOBAL_PRODUCT_PAIR_OFFSET_V2);
        let flow_count_byte_selectors = fields::<AB>(
            local,
            FLOW_COUNT_BYTE_SELECTOR_OFFSET_V2,
            FLOW_COUNT_BYTE_SELECTOR_COUNT_V2,
        );
        let next_flow_count_byte_selectors = fields::<AB>(
            next,
            FLOW_COUNT_BYTE_SELECTOR_OFFSET_V2,
            FLOW_COUNT_BYTE_SELECTOR_COUNT_V2,
        );
        for selector in &flow_count_byte_selectors {
            builder.assert_bool(selector.clone());
        }
        let flow_count_byte_active = flow_count_byte_selectors
            .iter()
            .cloned()
            .fold(AB::Expr::ZERO, |sum, value| sum + value);
        builder.assert_bool(flow_count_byte_active.clone());
        let flow_count_countdown = field::<AB>(local, FLOW_COUNT_COUNTDOWN_OFFSET_V2);
        let next_flow_count_countdown = field::<AB>(next, FLOW_COUNT_COUNTDOWN_OFFSET_V2);
        let flow_count_countdown_inverse =
            field::<AB>(local, FLOW_COUNT_COUNTDOWN_INVERSE_OFFSET_V2);
        let flow_count_countdown_active = field::<AB>(local, FLOW_COUNT_COUNTDOWN_ACTIVE_OFFSET_V2);
        let next_flow_count_countdown_active =
            field::<AB>(next, FLOW_COUNT_COUNTDOWN_ACTIVE_OFFSET_V2);
        builder.assert_bool(flow_count_countdown_active.clone());
        builder.assert_eq(
            flow_count_countdown.clone() * flow_count_countdown_inverse.clone(),
            flow_count_countdown_active.clone(),
        );
        builder.assert_zero(
            (one.clone() - flow_count_countdown_active.clone()) * flow_count_countdown.clone(),
        );
        builder.assert_zero(
            (one.clone() - flow_count_countdown_active.clone()) * flow_count_countdown_inverse,
        );
        let net_mutation_nonzero = field::<AB>(local, NET_MUTATION_NONZERO_OFFSET_V2);
        let net_mutation_inverse = field::<AB>(local, NET_MUTATION_INVERSE_OFFSET_V2);
        builder.assert_bool(net_mutation_nonzero.clone());
        for polynomial in nonzero_indicator_polynomials(
            net_mutation_nonzero.clone(),
            net_mutation_counter.clone(),
            net_mutation_inverse,
        ) {
            builder.assert_zero(polynomial);
        }
        let declared_spent_inverse = field::<AB>(local, DECLARED_SPENT_INVERSE_OFFSET_V2);
        let declared_output_inverse = field::<AB>(local, DECLARED_OUTPUT_INVERSE_OFFSET_V2);
        let declared_count_slack_bytes = fields::<AB>(
            local,
            DECLARED_COUNT_SLACK_BYTE_OFFSET_V2,
            DECLARED_COUNT_SLACK_BYTE_COUNT_V2,
        );
        for slack_byte in &declared_count_slack_bytes {
            builder.assert_zero((one.clone() - slot_end.clone()) * slack_byte.clone());
            if self.role == SemanticSourceAirRoleV2::Transition {
                builder.assert_zero(slot_end.clone() * slack_byte.clone());
            }
        }

        let jmt_stage = fields::<AB>(local, JMT_STAGE_SELECTOR_OFFSET_V2, JMT_STAGE_COUNT_V2);
        let next_jmt_stage = fields::<AB>(next, JMT_STAGE_SELECTOR_OFFSET_V2, JMT_STAGE_COUNT_V2);
        for selector in &jmt_stage {
            builder.assert_bool(selector.clone());
        }
        builder.assert_eq(
            jmt_stage
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value),
            active.clone(),
        );
        let jmt_header = opcode_selectors[RecursiveTraceOpcodeV2::JmtUpdate as usize - 1].clone();
        let jmt_promotion =
            opcode_selectors[RecursiveTraceOpcodeV2::PromoteChildRoot as usize - 1].clone();
        let jmt_micro = opcode_selectors[RecursiveTraceOpcodeV2::JmtMicroOp as usize - 1].clone();
        builder.assert_zero(jmt_header.clone() * (one.clone() - jmt_stage[0].clone()));
        builder.assert_zero(jmt_micro.clone() * (one.clone() - jmt_stage[1].clone()));
        builder.assert_zero(jmt_promotion.clone() * (one.clone() - jmt_stage[1].clone()));
        builder.assert_zero(
            slot_end.clone()
                * (one.clone()
                    - jmt_stage[2].clone()
                    - event_final.clone() * jmt_promotion.clone()),
        );

        let jmt_count = field::<AB>(local, JMT_COUNT_OFFSET_V2);
        let next_jmt_count = field::<AB>(next, JMT_COUNT_OFFSET_V2);
        let uniqueness_counters = fields::<AB>(
            local,
            UNIQUENESS_COUNTER_OFFSET_V2,
            UNIQUENESS_COUNTER_COUNT_V2,
        );
        let next_uniqueness_counters = fields::<AB>(
            next,
            UNIQUENESS_COUNTER_OFFSET_V2,
            UNIQUENESS_COUNTER_COUNT_V2,
        );
        let replay_phases = fields::<AB>(
            local,
            REPLAY_PHASE_SELECTOR_OFFSET_V2,
            REPLAY_PHASE_COUNT_V2,
        );
        let next_replay_phases =
            fields::<AB>(next, REPLAY_PHASE_SELECTOR_OFFSET_V2, REPLAY_PHASE_COUNT_V2);
        let replay_remaining = field::<AB>(local, REPLAY_REMAINING_OFFSET_V2);
        let next_replay_remaining = field::<AB>(next, REPLAY_REMAINING_OFFSET_V2);
        let replay_remaining_inverse = field::<AB>(local, REPLAY_REMAINING_INVERSE_OFFSET_V2);
        let replay_phase_final = field::<AB>(local, REPLAY_PHASE_FINAL_OFFSET_V2);
        let replay_tx_len_low = field::<AB>(local, REPLAY_TX_LEN_LOW_OFFSET_V2);
        let next_replay_tx_len_low = field::<AB>(next, REPLAY_TX_LEN_LOW_OFFSET_V2);
        let replay_tx_len_high = field::<AB>(local, REPLAY_TX_LEN_HIGH_OFFSET_V2);
        let next_replay_tx_len_high = field::<AB>(next, REPLAY_TX_LEN_HIGH_OFFSET_V2);
        let replay_tx_len_inverse = field::<AB>(local, REPLAY_TX_LEN_INVERSE_OFFSET_V2);
        let replay_hex_selectors = fields::<AB>(
            local,
            REPLAY_HEX_SELECTOR_OFFSET_V2,
            REPLAY_HEX_SELECTOR_COUNT_V2,
        );
        let replay_hex_low = field::<AB>(local, REPLAY_HEX_LOW_OFFSET_V2);
        let next_replay_hex_low = field::<AB>(next, REPLAY_HEX_LOW_OFFSET_V2);
        let replay_hex_high_nibble = field::<AB>(local, REPLAY_HEX_HIGH_NIBBLE_OFFSET_V2);
        let next_replay_hex_high_nibble = field::<AB>(next, REPLAY_HEX_HIGH_NIBBLE_OFFSET_V2);
        let replay_ascii_low = field::<AB>(local, REPLAY_ASCII_LOW_OFFSET_V2);
        let replay_ascii_high = field::<AB>(local, REPLAY_ASCII_HIGH_OFFSET_V2);
        let replay_semantic_index = field::<AB>(local, REPLAY_SEMANTIC_INDEX_OFFSET_V2);
        let replay_counters =
            fields::<AB>(local, REPLAY_COUNTER_OFFSET_V2, REPLAY_COUNTER_COUNT_V2);
        let next_replay_counters =
            fields::<AB>(next, REPLAY_COUNTER_OFFSET_V2, REPLAY_COUNTER_COUNT_V2);
        let flow_phases = fields::<AB>(local, FLOW_PHASE_SELECTOR_OFFSET_V2, FLOW_PHASE_COUNT_V2);
        let next_flow_phases =
            fields::<AB>(next, FLOW_PHASE_SELECTOR_OFFSET_V2, FLOW_PHASE_COUNT_V2);
        let flow_hex_selectors = fields::<AB>(
            local,
            FLOW_HEX_SELECTOR_OFFSET_V2,
            FLOW_HEX_SELECTOR_COUNT_V2,
        );
        let flow_hex_low = field::<AB>(local, FLOW_HEX_LOW_OFFSET_V2);
        let next_flow_hex_low = field::<AB>(next, FLOW_HEX_LOW_OFFSET_V2);
        let flow_hex_high_nibble = field::<AB>(local, FLOW_HEX_HIGH_NIBBLE_OFFSET_V2);
        let next_flow_hex_high_nibble = field::<AB>(next, FLOW_HEX_HIGH_NIBBLE_OFFSET_V2);
        let flow_hex_byte_index = field::<AB>(local, FLOW_HEX_BYTE_INDEX_OFFSET_V2);
        let flow_root_limb_index = field::<AB>(local, FLOW_ROOT_LIMB_INDEX_OFFSET_V2);
        let flow_root_byte_parity = field::<AB>(local, FLOW_ROOT_BYTE_PARITY_OFFSET_V2);
        let flow_root_low_byte = field::<AB>(local, FLOW_ROOT_LOW_BYTE_OFFSET_V2);
        let next_flow_root_low_byte = field::<AB>(next, FLOW_ROOT_LOW_BYTE_OFFSET_V2);
        let transcript_phases = fields::<AB>(
            local,
            TRANSCRIPT_PHASE_SELECTOR_OFFSET_V2,
            TRANSCRIPT_PHASE_COUNT_V2,
        );
        let next_transcript_phases = fields::<AB>(
            next,
            TRANSCRIPT_PHASE_SELECTOR_OFFSET_V2,
            TRANSCRIPT_PHASE_COUNT_V2,
        );
        let transcript_phase_final = field::<AB>(local, TRANSCRIPT_PHASE_FINAL_OFFSET_V2);
        let transcript_phase_end_inverse =
            field::<AB>(local, TRANSCRIPT_PHASE_END_INVERSE_OFFSET_V2);
        let transcript_pair_index = field::<AB>(local, TRANSCRIPT_PAIR_INDEX_OFFSET_V2);
        let transcript_pair_second = field::<AB>(local, TRANSCRIPT_PAIR_SECOND_OFFSET_V2);

        let begin_block = opcode_selectors[RecursiveTraceOpcodeV2::BeginBlock as usize - 1].clone();
        let uniqueness_precommit =
            opcode_selectors[RecursiveTraceOpcodeV2::UniquenessPrecommit as usize - 1].clone();
        let replay_input =
            opcode_selectors[RecursiveTraceOpcodeV2::ReplayInput as usize - 1].clone();
        let replay_output =
            opcode_selectors[RecursiveTraceOpcodeV2::ReplayOutput as usize - 1].clone();
        let uniqueness_challenge =
            opcode_selectors[RecursiveTraceOpcodeV2::UniquenessChallenge as usize - 1].clone();
        let typed = opcode_selectors[RecursiveTraceOpcodeV2::CommitTypedEvent as usize - 1].clone();
        let finalize = opcode_selectors[RecursiveTraceOpcodeV2::FinalizeBlock as usize - 1].clone();
        let flow_header = begin_block.clone() + finalize.clone();
        for (gate, expected_len) in [
            (flow_header.clone(), FLOW_HEADER_BYTES_V2),
            (uniqueness_precommit.clone(), UNIQUENESS_PRECOMMIT_BYTES_V2),
            (uniqueness_challenge.clone(), UNIQUENESS_CHALLENGE_BYTES_V2),
        ] {
            builder.assert_zero(gate * (payload_len.clone() - AB::Expr::from_usize(expected_len)));
        }
        for gate in [
            flow_count_countdown_active.clone(),
            flow_count_byte_active.clone(),
        ] {
            builder.assert_zero(gate.clone() * (one.clone() - payload.clone()));
            builder.assert_zero(gate * (one.clone() - flow_header.clone()));
        }
        for (count_byte, selector) in flow_count_byte_selectors.iter().enumerate() {
            builder.assert_zero(
                finalize.clone()
                    * selector.clone()
                    * (byte.clone() - declared_count_bytes[count_byte].clone()),
            );
        }
        builder.assert_zero(
            (uniqueness_precommit.clone() + uniqueness_challenge.clone())
                * payload_prefix[0].clone()
                * (byte.clone() - AB::Expr::from_u64(u64::from(UNIQUENESS_PRECOMMIT_VERSION_V2))),
        );
        for (count_byte, declared) in declared_count_bytes.iter().enumerate() {
            builder.assert_zero(
                uniqueness_precommit.clone()
                    * payload_prefix[count_byte + 1].clone()
                    * (declared.clone() - byte.clone()),
            );
        }
        let event_phases = vec![
            begin_block.clone(),
            uniqueness_precommit.clone(),
            replay_input.clone() + uniqueness_classes[0].clone(),
            replay_output.clone() + uniqueness_classes[1].clone(),
            uniqueness_classes[2].clone(),
            uniqueness_classes[3].clone(),
            uniqueness_challenge.clone(),
            uniqueness_classes[4].clone(),
            uniqueness_classes[5].clone(),
            uniqueness_classes[6].clone() + uniqueness_classes[7].clone() + net_non_close.clone(),
            net_kind_selectors[0].clone(),
            jmt_header.clone(),
            jmt_micro.clone(),
            jmt_promotion.clone(),
            typed.clone(),
            finalize.clone(),
        ];
        let next_net_non_close = next_net_kind_selectors[1..]
            .iter()
            .cloned()
            .fold(AB::Expr::ZERO, |sum, value| sum + value);
        let product_spent = uniqueness_classes[6].clone();
        let product_output = uniqueness_classes[7].clone();
        let next_product_spent = next_uniqueness_classes[6].clone();
        let next_product_output = next_uniqueness_classes[7].clone();
        builder.assert_bool(global_product_pair.clone());
        builder.assert_zero(global_product_pair.clone() * (one.clone() - product_output.clone()));
        let current_product_or_net =
            product_spent.clone() + product_output.clone() + net_non_close.clone();
        builder.when_transition().assert_zero(
            event_final.clone()
                * global_schedule_entry_polynomial(
                    current_product_or_net,
                    next_net_non_close.clone(),
                ),
        );
        for polynomial in global_schedule_polynomials(
            product_spent.clone(),
            product_output.clone(),
            global_product_pair.clone(),
            net_non_close.clone(),
            next_product_spent.clone(),
            next_product_output.clone(),
            next_net_kind_selectors[0].clone(),
            next_net_kind_selectors[1].clone(),
            next_net_kind_selectors[2].clone(),
            next_net_kind_selectors[3].clone(),
            next_net_kind_selectors[4].clone(),
        ) {
            builder
                .when_transition()
                .assert_zero(event_final.clone() * polynomial);
        }
        let next_event_phases = vec![
            next_opcode_selectors[RecursiveTraceOpcodeV2::BeginBlock as usize - 1].clone(),
            next_opcode_selectors[RecursiveTraceOpcodeV2::UniquenessPrecommit as usize - 1].clone(),
            next_opcode_selectors[RecursiveTraceOpcodeV2::ReplayInput as usize - 1].clone()
                + next_uniqueness_classes[0].clone(),
            next_opcode_selectors[RecursiveTraceOpcodeV2::ReplayOutput as usize - 1].clone()
                + next_uniqueness_classes[1].clone(),
            next_uniqueness_classes[2].clone(),
            next_uniqueness_classes[3].clone(),
            next_opcode_selectors[RecursiveTraceOpcodeV2::UniquenessChallenge as usize - 1].clone(),
            next_uniqueness_classes[4].clone(),
            next_uniqueness_classes[5].clone(),
            next_uniqueness_classes[6].clone()
                + next_uniqueness_classes[7].clone()
                + next_net_non_close,
            next_net_kind_selectors[0].clone(),
            next_opcode_selectors[RecursiveTraceOpcodeV2::JmtUpdate as usize - 1].clone(),
            next_opcode_selectors[RecursiveTraceOpcodeV2::JmtMicroOp as usize - 1].clone(),
            next_opcode_selectors[RecursiveTraceOpcodeV2::PromoteChildRoot as usize - 1].clone(),
            next_opcode_selectors[RecursiveTraceOpcodeV2::CommitTypedEvent as usize - 1].clone(),
            next_opcode_selectors[RecursiveTraceOpcodeV2::FinalizeBlock as usize - 1].clone(),
        ];
        builder.assert_eq(
            event_phases
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value),
            event_active.clone(),
        );
        for (current_phase, current_selector) in event_phases.iter().enumerate() {
            for earlier_selector in next_event_phases.iter().take(current_phase) {
                builder.when_transition().assert_zero(
                    event_final.clone() * current_selector.clone() * earlier_selector.clone(),
                );
            }
        }
        builder.when_transition().assert_zero(
            prefix[PREFIX_BYTES_V2 - 1].clone() * (next_event_phases[0].clone() - one.clone()),
        );
        builder.assert_zero(slot_end.clone() * (one.clone() - event_phases[15].clone()));
        builder.when_transition().assert_zero(
            event_final.clone()
                * replay_input.clone()
                * (one.clone() - next_uniqueness_classes[0].clone()),
        );
        builder.when_transition().assert_zero(
            event_final.clone()
                * replay_output.clone()
                * (one.clone() - next_uniqueness_classes[1].clone()),
        );

        for (gate, count) in [(begin_block.clone(), 1_i8), (finalize.clone(), -1_i8)] {
            builder.push_interaction(
                SOURCE_BEGIN_FINALIZE_BYTE_BUS_V2,
                vec![
                    transition_index.clone(),
                    payload_index.clone(),
                    byte.clone(),
                ],
                if count > 0 {
                    Count::bounded(payload.clone() * gate, 1)
                } else {
                    -Count::bounded(payload.clone() * gate, 1)
                },
            );
        }
        let fixed_event_gates = [
            begin_block,
            uniqueness_precommit.clone(),
            uniqueness_challenge.clone(),
            net_kind_selectors[0].clone(),
            typed.clone(),
            finalize,
        ];
        let fixed_event_expected = [1_u64, 1, 1, 1, 4, 1];
        for (counter, (gate, expected)) in fixed_event_counters
            .iter()
            .zip(fixed_event_gates.iter().zip(fixed_event_expected))
        {
            builder.assert_zero(
                slot_end.clone()
                    * (counter.clone() + event_final.clone() * gate.clone()
                        - AB::Expr::from_u64(expected)),
            );
        }

        for pair in 0..16 {
            push_semantic_digest_pair(
                builder,
                transition_index.clone(),
                SemanticShaJobKindV2::EventVector,
                AB::Expr::ZERO,
                AB::Expr::from_usize(pair),
                selected_digest_byte(PUBLIC_EVENT_VECTOR_DIGEST_BYTE_OFFSET_V2, pair * 2)
                    * AB::Expr::from_u64(256)
                    + selected_digest_byte(PUBLIC_EVENT_VECTOR_DIGEST_BYTE_OFFSET_V2, pair * 2 + 1),
                prefix[0].clone(),
            );
        }

        match self.role {
            SemanticSourceAirRoleV2::Transition => {
                for offset in TRANSCRIPT_PHASE_SELECTOR_OFFSET_V2..ROW_FIELDS_V2 {
                    builder.assert_zero(field::<AB>(local, offset));
                }
                for offset in REPLAY_PHASE_SELECTOR_OFFSET_V2..FLOW_PHASE_SELECTOR_OFFSET_V2 {
                    builder.assert_zero(field::<AB>(local, offset));
                }
                let structural =
                    jmt_header.clone() + jmt_micro.clone() + jmt_promotion.clone() + typed.clone();
                let structural_header_gate = header[0].clone() * structural.clone();
                let structural_prefix =
                    CheckpointSha256BlockStreamV2::framed_role_prefix(CheckpointShaRole::Trace);
                let structural_bodies = semantic_part_body_offsets(
                    CheckpointShaRole::Trace,
                    &[STRUCTURAL_EVENT_HASH_LABEL_V2.len(), 1, 8, 0],
                );
                push_semantic_constant_bytes(
                    builder,
                    transition_index.clone(),
                    SemanticShaJobKindV2::StructuralEventId,
                    event_index.clone(),
                    0,
                    &structural_prefix,
                    structural_header_gate.clone(),
                );
                for (part, length) in [
                    u64::try_from(STRUCTURAL_EVENT_HASH_LABEL_V2.len())
                        .expect("structural label length fits u64"),
                    1,
                    8,
                ]
                .into_iter()
                .enumerate()
                {
                    push_semantic_constant_bytes(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::StructuralEventId,
                        event_index.clone(),
                        structural_bodies[part] - 8,
                        &length.to_le_bytes(),
                        structural_header_gate.clone(),
                    );
                }
                push_semantic_constant_bytes(
                    builder,
                    transition_index.clone(),
                    SemanticShaJobKindV2::StructuralEventId,
                    event_index.clone(),
                    structural_bodies[0],
                    STRUCTURAL_EVENT_HASH_LABEL_V2,
                    structural_header_gate.clone(),
                );
                push_semantic_raw_byte(
                    builder,
                    transition_index.clone(),
                    SemanticShaJobKindV2::StructuralEventId,
                    event_index.clone(),
                    AB::Expr::from_usize(structural_bodies[1]),
                    opcode.clone(),
                    structural_header_gate.clone(),
                );
                for (index, ordinal_byte) in ordinal_bytes.iter().cloned().enumerate() {
                    push_semantic_raw_byte(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::StructuralEventId,
                        event_index.clone(),
                        AB::Expr::from_usize(structural_bodies[2] + index),
                        ordinal_byte,
                        structural_header_gate.clone(),
                    );
                }
                for index in 0..8 {
                    push_semantic_raw_byte(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::StructuralEventId,
                        event_index.clone(),
                        AB::Expr::from_usize(structural_bodies[3] - 8 + index),
                        payload_len_bytes
                            .get(index)
                            .cloned()
                            .unwrap_or(AB::Expr::ZERO),
                        structural_header_gate.clone(),
                    );
                }
                push_semantic_raw_byte(
                    builder,
                    transition_index.clone(),
                    SemanticShaJobKindV2::StructuralEventId,
                    event_index.clone(),
                    AB::Expr::from_usize(structural_bodies[3]) + payload_index.clone(),
                    byte.clone(),
                    payload.clone() * structural.clone(),
                );
                for pair in 0..16 {
                    push_semantic_digest_pair(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::StructuralEventId,
                        event_index.clone(),
                        AB::Expr::from_usize(pair),
                        byte.clone() * AB::Expr::from_u64(256) + next_byte.clone(),
                        header[9 + pair * 2].clone() * structural.clone(),
                    );
                }
                for selector in &flow_phases {
                    builder.assert_bool(selector.clone());
                }
                let flow_phase_active = flow_phases
                    .iter()
                    .cloned()
                    .fold(AB::Expr::ZERO, |sum, value| sum + value);
                builder.assert_eq(flow_phase_active, payload.clone() * flow_header.clone());
                for (phase, expected) in [
                    (0_usize, 64_u64),
                    (1, 0),
                    (4, 64),
                    (5, 0),
                    (7, 64),
                    (8, 0),
                    (10, 64),
                    (11, 0),
                ] {
                    builder.assert_zero(
                        flow_phases[phase].clone() * (byte.clone() - AB::Expr::from_u64(expected)),
                    );
                }

                for selector in &flow_hex_selectors {
                    builder.assert_bool(selector.clone());
                }
                let flow_hex_active = [2_usize, 6, 9, 12]
                    .into_iter()
                    .fold(AB::Expr::ZERO, |sum, phase| {
                        sum + flow_phases[phase].clone()
                    });
                builder.assert_eq(
                    flow_hex_selectors
                        .iter()
                        .cloned()
                        .fold(AB::Expr::ZERO, |sum, value| sum + value),
                    flow_hex_active.clone(),
                );
                let flow_hex_nibble = flow_hex_selectors
                    .iter()
                    .enumerate()
                    .fold(AB::Expr::ZERO, |sum, (index, selector)| {
                        sum + selector.clone() * AB::Expr::from_usize(index)
                    });
                let flow_hex_ascii = flow_hex_selectors.iter().enumerate().fold(
                    AB::Expr::ZERO,
                    |sum, (index, selector)| {
                        let ascii = if index < 10 {
                            b'0' + u8::try_from(index).expect("hex selector index fits u8")
                        } else {
                            b'a' + u8::try_from(index - 10).expect("hex selector index fits u8")
                        };
                        sum + selector.clone() * AB::Expr::from_u64(u64::from(ascii))
                    },
                );
                builder.assert_eq(flow_hex_active.clone() * byte.clone(), flow_hex_ascii);
                builder.assert_bool(flow_hex_low.clone());
                builder.assert_zero(flow_hex_low.clone() * (one.clone() - flow_hex_active.clone()));
                builder.assert_zero(
                    (one.clone() - flow_hex_active.clone()) * flow_hex_high_nibble.clone(),
                );
                let flow_hex_high_row =
                    flow_hex_active.clone() * (one.clone() - flow_hex_low.clone());
                builder.assert_zero(
                    flow_hex_high_row.clone()
                        * (flow_hex_high_nibble.clone() - flow_hex_nibble.clone()),
                );
                let flow_hex_emit = flow_hex_active.clone() * flow_hex_low.clone();
                builder.assert_zero(
                    (one.clone() - flow_hex_emit.clone()) * flow_hex_byte_index.clone(),
                );
                for (phase, low_start) in [(2_usize, 3_u64), (6, 81), (9, 147), (12, 213)] {
                    builder.assert_zero(
                        flow_phases[phase].clone()
                            * flow_hex_low.clone()
                            * (flow_hex_byte_index.clone() * AB::Expr::from_u64(2)
                                - payload_index.clone()
                                + AB::Expr::from_u64(low_start)),
                    );
                }

                let flow_root_emit =
                    (flow_phases[9].clone() + flow_phases[12].clone()) * flow_hex_low.clone();
                builder.assert_bool(flow_root_byte_parity.clone());
                builder.assert_zero(
                    flow_root_byte_parity.clone() * (one.clone() - flow_root_emit.clone()),
                );
                builder.assert_zero(
                    (one.clone() - flow_root_emit.clone()) * flow_root_limb_index.clone(),
                );
                builder.assert_zero(
                    flow_root_emit.clone()
                        * (flow_hex_byte_index.clone()
                            - flow_root_limb_index.clone() * AB::Expr::from_u64(2)
                            - flow_root_byte_parity.clone()),
                );
                let flow_decoded_byte =
                    flow_hex_high_nibble.clone() * AB::Expr::from_u64(16) + flow_hex_nibble;
                let flow_root_even =
                    flow_root_emit.clone() * (one.clone() - flow_root_byte_parity.clone());
                let flow_root_odd = flow_root_emit.clone() * flow_root_byte_parity.clone();
                builder.push_interaction(
                    TRANSITION_FLOW_ROOT_LIMB_BUS_V2,
                    vec![
                        transition_index.clone(),
                        flow_phases[12].clone(),
                        flow_root_limb_index.clone(),
                        flow_root_low_byte.clone()
                            + flow_decoded_byte.clone() * AB::Expr::from_u64(256),
                    ],
                    -Count::bounded(flow_root_odd.clone() * fixed_event_gates[0].clone(), 1),
                );
                builder
                    .when_first_row()
                    .assert_zero(flow_root_low_byte.clone());
                builder
                    .when_last_row()
                    .assert_zero(flow_root_low_byte.clone());
                builder.when_transition().assert_eq(
                    next_flow_root_low_byte,
                    (one.clone() - slot_end.clone())
                        * ((one.clone() - flow_root_emit.clone()) * flow_root_low_byte
                            + flow_root_even * flow_decoded_byte),
                );

                let flow_phase_end = [
                    0_u64, 1, 65, 77, 78, 79, 143, 144, 145, 209, 210, 211, 275, 283,
                ];
                builder.when_transition().assert_zero(
                    header[HEADER_BYTES_V2 - 1].clone()
                        * flow_header.clone()
                        * (next_flow_phases[0].clone() - one.clone()),
                );
                for phase in 0..FLOW_PHASE_COUNT_V2 - 1 {
                    builder.assert_zero(event_final.clone() * flow_phases[phase].clone());
                    builder.when_transition().assert_zero(
                        flow_phases[phase].clone()
                            * (next_flow_phases[phase].clone()
                                + next_flow_phases[phase + 1].clone()
                                - one.clone()),
                    );
                    builder.when_transition().assert_zero(
                        flow_phases[phase].clone()
                            * next_flow_phases[phase + 1].clone()
                            * (payload_index.clone() - AB::Expr::from_u64(flow_phase_end[phase])),
                    );
                }
                builder.assert_zero(
                    event_final.clone()
                        * flow_header.clone()
                        * (one.clone() - flow_phases[FLOW_PHASE_COUNT_V2 - 1].clone()),
                );
                builder.assert_zero(
                    event_final.clone()
                        * flow_phases[FLOW_PHASE_COUNT_V2 - 1].clone()
                        * (payload_index.clone()
                            - AB::Expr::from_u64(flow_phase_end[FLOW_PHASE_COUNT_V2 - 1])),
                );
                builder.when_transition().assert_zero(
                    flow_phases[FLOW_PHASE_COUNT_V2 - 1].clone()
                        * (one.clone() - event_final.clone())
                        * (next_flow_phases[FLOW_PHASE_COUNT_V2 - 1].clone() - one.clone()),
                );
                for (hex_phase, prior_phase) in [(2_usize, 1_usize), (6, 5), (9, 8), (12, 11)] {
                    builder.when_transition().assert_zero(
                        flow_phases[prior_phase].clone()
                            * next_flow_phases[hex_phase].clone()
                            * next_flow_hex_low.clone(),
                    );
                    builder.when_transition().assert_zero(
                        flow_phases[hex_phase].clone()
                            * next_flow_phases[hex_phase].clone()
                            * (next_flow_hex_low.clone() + flow_hex_low.clone() - one.clone()),
                    );
                    builder.when_transition().assert_zero(
                        flow_phases[hex_phase].clone()
                            * next_flow_phases[hex_phase].clone()
                            * (one.clone() - flow_hex_low.clone())
                            * (next_flow_hex_high_nibble.clone() - flow_hex_high_nibble.clone()),
                    );
                    builder.when_transition().assert_zero(
                        flow_phases[hex_phase].clone()
                            * next_flow_phases[hex_phase + 1].clone()
                            * (one.clone() - flow_hex_low.clone()),
                    );
                }
                builder.push_interaction(
                    SOURCE_NET_MUTATION_BYTE_BUS_V2,
                    [transition_index.clone()]
                        .into_iter()
                        .chain(net_terminal_limbs.iter().cloned())
                        .chain([payload_index.clone(), byte.clone()])
                        .collect::<Vec<_>>(),
                    Count::bounded(payload.clone() * net_mutation.clone(), 1),
                );
                builder.push_interaction(
                    SOURCE_TYPED_PAYLOAD_BYTE_BUS_V2,
                    [transition_index.clone()]
                        .into_iter()
                        .chain(ordinal_bytes.iter().cloned())
                        .chain([payload_index.clone(), byte.clone()])
                        .collect::<Vec<AB::Expr>>(),
                    Count::bounded(payload.clone() * typed, 1),
                );
                for (event_opcode, gate, position) in [
                    (
                        RecursiveTraceOpcodeV2::JmtUpdate as u8,
                        jmt_header.clone(),
                        AB::Expr::ZERO,
                    ),
                    (
                        RecursiveTraceOpcodeV2::PromoteChildRoot as u8,
                        jmt_promotion.clone(),
                        AB::Expr::ZERO,
                    ),
                    (
                        RecursiveTraceOpcodeV2::JmtMicroOp as u8,
                        jmt_micro.clone(),
                        jmt_count.clone() + one.clone(),
                    ),
                ] {
                    builder.push_interaction(
                        SOURCE_JMT_PAYLOAD_BYTE_BUS_V2,
                        vec![
                            transition_index.clone(),
                            AB::Expr::from_u64(u64::from(event_opcode)),
                            position,
                            payload_index.clone(),
                            byte.clone(),
                        ],
                        Count::bounded(payload.clone() * gate, 1),
                    );
                }
            }
            SemanticSourceAirRoleV2::Uniqueness => {
                for offset in FLOW_PHASE_SELECTOR_OFFSET_V2..TRANSCRIPT_PHASE_SELECTOR_OFFSET_V2 {
                    builder.assert_zero(field::<AB>(local, offset));
                }
                for selector in &transcript_phases {
                    builder.assert_bool(selector.clone());
                }
                builder.assert_bool(transcript_phase_final.clone());
                let transcript_phase_active = transcript_phases
                    .iter()
                    .cloned()
                    .fold(AB::Expr::ZERO, |sum, selector| sum + selector);
                let close = net_kind_selectors[NetEffectKindV2::Close as usize].clone();
                let transcript_event =
                    uniqueness_precommit.clone() + uniqueness_challenge.clone() + close.clone();
                builder.assert_eq(
                    transcript_phase_active.clone(),
                    payload.clone() * transcript_event.clone(),
                );
                builder.assert_zero(
                    transcript_phase_final.clone()
                        * (one.clone() - transcript_phase_active.clone()),
                );
                builder.assert_zero(
                    (one.clone() - transcript_phase_active.clone())
                        * transcript_phase_end_inverse.clone(),
                );
                let selected_phase_end = transcript_phases.iter().enumerate().fold(
                    AB::Expr::ZERO,
                    |sum, (phase, selector)| {
                        sum + selector.clone()
                            * AB::Expr::from_usize(TRANSCRIPT_PHASE_ENDS_V2[phase])
                    },
                );
                let selected_phase_start = transcript_phases.iter().enumerate().fold(
                    AB::Expr::ZERO,
                    |sum, (phase, selector)| {
                        sum + selector.clone()
                            * AB::Expr::from_usize(TRANSCRIPT_PHASE_STARTS_V2[phase])
                    },
                );
                let transcript_pair_active = transcript_phases
                    .iter()
                    .enumerate()
                    .filter(|(phase, _)| {
                        TRANSCRIPT_PHASE_ENDS_V2[*phase] - TRANSCRIPT_PHASE_STARTS_V2[*phase] + 1
                            == 32
                    })
                    .fold(AB::Expr::ZERO, |sum, (_, selector)| sum + selector.clone());
                builder.assert_bool(transcript_pair_second.clone());
                builder.assert_zero(
                    transcript_pair_second.clone() * (one.clone() - transcript_pair_active.clone()),
                );
                builder.assert_zero(
                    (one.clone() - transcript_pair_active.clone()) * transcript_pair_index.clone(),
                );
                builder.assert_zero(
                    transcript_pair_active
                        * (payload_index.clone()
                            - selected_phase_start
                            - transcript_pair_index.clone() * AB::Expr::from_u64(2)
                            - transcript_pair_second.clone()),
                );
                let phase_distance = selected_phase_end - payload_index.clone();
                let phase_not_final =
                    transcript_phase_active.clone() - transcript_phase_final.clone();
                builder.assert_zero(
                    transcript_phase_active.clone()
                        * (phase_distance.clone() * transcript_phase_end_inverse.clone()
                            - phase_not_final),
                );
                builder.assert_zero(
                    transcript_phase_active.clone()
                        * phase_distance.clone()
                        * transcript_phase_final.clone(),
                );
                builder.assert_zero(
                    transcript_phase_active.clone()
                        * transcript_phase_end_inverse.clone()
                        * transcript_phase_final.clone(),
                );

                let precommit_header =
                    header[HEADER_BYTES_V2 - 1].clone() * uniqueness_precommit.clone();
                let challenge_header =
                    header[HEADER_BYTES_V2 - 1].clone() * uniqueness_challenge.clone();
                let close_header = header[HEADER_BYTES_V2 - 1].clone() * close.clone();
                {
                    let mut transition = builder.when_transition();
                    transition.assert_zero(
                        precommit_header.clone()
                            * (next_transcript_phases[PRECOMMIT_VERSION_PHASE_V2].clone()
                                - one.clone()),
                    );
                    transition.assert_zero(
                        challenge_header.clone()
                            * (next_transcript_phases[CHALLENGE_VERSION_PHASE_V2].clone()
                                - one.clone()),
                    );
                    transition.assert_zero(
                        close_header.clone()
                            * (next_transcript_phases[CLOSE_HEADER_PHASE_V2].clone() - one.clone()),
                    );
                    for phase in 0..TRANSCRIPT_PHASE_COUNT_V2 {
                        let last = matches!(
                            phase,
                            PRECOMMIT_DIGEST_PHASE_V2
                                | CHALLENGE_DIGEST_LAST_PHASE_V2
                                | CLOSE_OUTPUT_PRECOMMIT_PHASE_V2
                        );
                        if last {
                            transition.assert_zero(
                                transcript_phases[phase].clone()
                                    * (next_transcript_phases[phase].clone()
                                        - (one.clone() - event_final.clone())),
                            );
                        } else {
                            transition.assert_zero(
                                transcript_phases[phase].clone()
                                    * (next_transcript_phases[phase].clone()
                                        - (one.clone() - transcript_phase_final.clone())),
                            );
                            transition.assert_zero(
                                transcript_phases[phase].clone()
                                    * (next_transcript_phases[phase + 1].clone()
                                        - transcript_phase_final.clone()),
                            );
                        }
                    }
                }
                for phase in 0..TRANSCRIPT_PHASE_COUNT_V2 {
                    let last = matches!(
                        phase,
                        PRECOMMIT_DIGEST_PHASE_V2
                            | CHALLENGE_DIGEST_LAST_PHASE_V2
                            | CLOSE_OUTPUT_PRECOMMIT_PHASE_V2
                    );
                    if !last {
                        builder.assert_zero(event_final.clone() * transcript_phases[phase].clone());
                    } else {
                        builder.assert_zero(
                            transcript_phases[phase].clone()
                                * (transcript_phase_final.clone() - event_final.clone()),
                        );
                    }
                }
                builder.assert_zero(
                    event_final.clone()
                        * uniqueness_precommit.clone()
                        * (one.clone() - transcript_phases[PRECOMMIT_DIGEST_PHASE_V2].clone()),
                );
                builder.assert_zero(
                    event_final.clone()
                        * uniqueness_challenge.clone()
                        * (one.clone() - transcript_phases[CHALLENGE_DIGEST_LAST_PHASE_V2].clone()),
                );
                builder.assert_zero(
                    event_final.clone()
                        * close.clone()
                        * (one.clone()
                            - transcript_phases[CLOSE_OUTPUT_PRECOMMIT_PHASE_V2].clone()),
                );

                let field_byte = |phase: usize| {
                    payload_index.clone() - AB::Expr::from_usize(TRANSCRIPT_PHASE_STARTS_V2[phase])
                };
                let digest_pair_gate = |phase: usize| {
                    transcript_phases[phase].clone()
                        * (one.clone() - transcript_pair_second.clone())
                };

                let list_roles = [
                    CheckpointShaRole::SpentOriginalIds,
                    CheckpointShaRole::SpentSortedIds,
                    CheckpointShaRole::OutputOriginalIds,
                    CheckpointShaRole::OutputSortedIds,
                ];
                for (id, role) in list_roles.into_iter().enumerate() {
                    let prefix = CheckpointSha256BlockStreamV2::framed_role_prefix(role);
                    let count_body = semantic_part_body_offsets(role, &[4])[0];
                    let job_id = AB::Expr::from_usize(id);
                    push_semantic_constant_bytes(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::UniquenessList,
                        job_id.clone(),
                        0,
                        &prefix,
                        precommit_header.clone(),
                    );
                    push_semantic_constant_bytes(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::UniquenessList,
                        job_id.clone(),
                        count_body - 8,
                        &4_u64.to_le_bytes(),
                        precommit_header.clone(),
                    );
                    let count_start = if id < 2 { 0 } else { 4 };
                    for count_byte in 0..4 {
                        push_semantic_raw_byte(
                            builder,
                            transition_index.clone(),
                            SemanticShaJobKindV2::UniquenessList,
                            job_id.clone(),
                            AB::Expr::from_usize(count_body + count_byte),
                            declared_count_bytes[count_start + count_byte].clone(),
                            precommit_header.clone(),
                        );
                    }
                }
                for (job_id, phase) in [
                    PRECOMMIT_SPENT_ORIGINAL_PHASE_V2,
                    PRECOMMIT_SPENT_SORTED_PHASE_V2,
                    PRECOMMIT_OUTPUT_ORIGINAL_PHASE_V2,
                    PRECOMMIT_OUTPUT_SORTED_PHASE_V2,
                ]
                .into_iter()
                .enumerate()
                {
                    push_semantic_digest_pair(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::UniquenessList,
                        AB::Expr::from_usize(job_id),
                        transcript_pair_index.clone(),
                        byte.clone() * AB::Expr::from_u64(256) + next_byte.clone(),
                        digest_pair_gate(phase),
                    );
                }

                let precommit_bodies = push_semantic_fixed_framing(
                    builder,
                    transition_index.clone(),
                    SemanticShaJobKindV2::UniquenessPrecommit,
                    AB::Expr::ZERO,
                    CheckpointShaRole::IdPrecommit,
                    &[UNIQUENESS_PRECOMMIT_LABEL_V2.len(), 4, 4, 32, 32, 32, 32],
                    precommit_header.clone(),
                );
                push_semantic_constant_bytes(
                    builder,
                    transition_index.clone(),
                    SemanticShaJobKindV2::UniquenessPrecommit,
                    AB::Expr::ZERO,
                    precommit_bodies[0],
                    UNIQUENESS_PRECOMMIT_LABEL_V2,
                    precommit_header.clone(),
                );
                for (part, count_start) in [(1_usize, 0_usize), (2, 4)] {
                    for count_byte in 0..4 {
                        push_semantic_raw_byte(
                            builder,
                            transition_index.clone(),
                            SemanticShaJobKindV2::UniquenessPrecommit,
                            AB::Expr::ZERO,
                            AB::Expr::from_usize(precommit_bodies[part] + count_byte),
                            declared_count_bytes[count_start + count_byte].clone(),
                            precommit_header.clone(),
                        );
                    }
                }
                for (part, phase) in [
                    PRECOMMIT_SPENT_ORIGINAL_PHASE_V2,
                    PRECOMMIT_SPENT_SORTED_PHASE_V2,
                    PRECOMMIT_OUTPUT_ORIGINAL_PHASE_V2,
                    PRECOMMIT_OUTPUT_SORTED_PHASE_V2,
                ]
                .into_iter()
                .enumerate()
                {
                    push_semantic_raw_byte(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::UniquenessPrecommit,
                        AB::Expr::ZERO,
                        AB::Expr::from_usize(precommit_bodies[part + 3]) + field_byte(phase),
                        byte.clone(),
                        transcript_phases[phase].clone(),
                    );
                }
                push_semantic_digest_pair(
                    builder,
                    transition_index.clone(),
                    SemanticShaJobKindV2::UniquenessPrecommit,
                    AB::Expr::ZERO,
                    transcript_pair_index.clone(),
                    byte.clone() * AB::Expr::from_u64(256) + next_byte.clone(),
                    digest_pair_gate(PRECOMMIT_DIGEST_PHASE_V2),
                );

                for set in 0..2 {
                    let job_id = AB::Expr::from_usize(set);
                    let bodies = push_semantic_fixed_framing(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::UniquenessSetPrecommit,
                        job_id.clone(),
                        CheckpointShaRole::IdPrecommit,
                        &[32, 1, 4, 32, 32],
                        precommit_header.clone(),
                    );
                    push_semantic_constant_bytes(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::UniquenessSetPrecommit,
                        job_id.clone(),
                        bodies[1],
                        &[u8::try_from(set).expect("set index fits u8")],
                        precommit_header.clone(),
                    );
                    for count_byte in 0..4 {
                        push_semantic_raw_byte(
                            builder,
                            transition_index.clone(),
                            SemanticShaJobKindV2::UniquenessSetPrecommit,
                            job_id.clone(),
                            AB::Expr::from_usize(bodies[2] + count_byte),
                            declared_count_bytes[set * 4 + count_byte].clone(),
                            precommit_header.clone(),
                        );
                    }
                    push_semantic_raw_byte(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::UniquenessSetPrecommit,
                        job_id.clone(),
                        AB::Expr::from_usize(bodies[0]) + field_byte(CHALLENGE_CONTEXT_PHASE_V2),
                        byte.clone(),
                        transcript_phases[CHALLENGE_CONTEXT_PHASE_V2].clone(),
                    );
                    for (part, phase) in if set == 0 {
                        [
                            (3_usize, PRECOMMIT_SPENT_ORIGINAL_PHASE_V2),
                            (4, PRECOMMIT_SPENT_SORTED_PHASE_V2),
                        ]
                    } else {
                        [
                            (3_usize, PRECOMMIT_OUTPUT_ORIGINAL_PHASE_V2),
                            (4, PRECOMMIT_OUTPUT_SORTED_PHASE_V2),
                        ]
                    } {
                        push_semantic_raw_byte(
                            builder,
                            transition_index.clone(),
                            SemanticShaJobKindV2::UniquenessSetPrecommit,
                            job_id.clone(),
                            AB::Expr::from_usize(bodies[part]) + field_byte(phase),
                            byte.clone(),
                            transcript_phases[phase].clone(),
                        );
                    }
                    let digest_phase = if set == 0 {
                        CHALLENGE_SPENT_PRECOMMIT_PHASE_V2
                    } else {
                        CHALLENGE_OUTPUT_PRECOMMIT_PHASE_V2
                    };
                    push_semantic_digest_pair(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::UniquenessSetPrecommit,
                        job_id,
                        transcript_pair_index.clone(),
                        byte.clone() * AB::Expr::from_u64(256) + next_byte.clone(),
                        digest_pair_gate(digest_phase),
                    );
                }

                let challenge_bodies =
                    semantic_part_body_offsets(CheckpointShaRole::IdChallenge, &[32, 32, 1, 1, 1]);
                let grammar_digest = RecursiveTraceOpcodeV2::grammar_digest();
                for job_id in 0..8 {
                    let id = AB::Expr::from_usize(job_id);
                    let bodies = push_semantic_fixed_framing(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::UniquenessChallenge,
                        id.clone(),
                        CheckpointShaRole::IdChallenge,
                        &[32, 32, 1, 1, 1],
                        challenge_header.clone(),
                    );
                    debug_assert_eq!(bodies, challenge_bodies);
                    push_semantic_constant_bytes(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::UniquenessChallenge,
                        id.clone(),
                        bodies[1],
                        &grammar_digest,
                        challenge_header.clone(),
                    );
                    let set = job_id / 4;
                    let coordinate_index = job_id % 4;
                    for (part, value) in [
                        (2_usize, set),
                        (3, coordinate_index / 2),
                        (4, coordinate_index % 2),
                    ] {
                        push_semantic_constant_bytes(
                            builder,
                            transition_index.clone(),
                            SemanticShaJobKindV2::UniquenessChallenge,
                            id.clone(),
                            bodies[part],
                            &[u8::try_from(value).expect("challenge coordinate fits u8")],
                            challenge_header.clone(),
                        );
                    }
                    let set_phase = if set == 0 {
                        CHALLENGE_SPENT_PRECOMMIT_PHASE_V2
                    } else {
                        CHALLENGE_OUTPUT_PRECOMMIT_PHASE_V2
                    };
                    push_semantic_raw_byte(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::UniquenessChallenge,
                        id.clone(),
                        AB::Expr::from_usize(bodies[0]) + field_byte(set_phase),
                        byte.clone(),
                        transcript_phases[set_phase].clone(),
                    );
                    let digest_phase = CHALLENGE_DIGEST_FIRST_PHASE_V2 + job_id;
                    push_semantic_digest_pair(
                        builder,
                        transition_index.clone(),
                        SemanticShaJobKindV2::UniquenessChallenge,
                        id,
                        transcript_pair_index.clone(),
                        byte.clone() * AB::Expr::from_u64(256) + next_byte.clone(),
                        digest_pair_gate(digest_phase),
                    );
                }
                builder.push_interaction(
                    SOURCE_PRECOMMIT_CHALLENGE_BYTE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        field_byte(PRECOMMIT_DIGEST_PHASE_V2),
                        byte.clone(),
                    ],
                    Count::bounded(transcript_phases[PRECOMMIT_DIGEST_PHASE_V2].clone(), 1),
                );
                builder.push_interaction(
                    SOURCE_PRECOMMIT_CHALLENGE_BYTE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        field_byte(CHALLENGE_PRECOMMIT_PHASE_V2),
                        byte.clone(),
                    ],
                    -Count::bounded(transcript_phases[CHALLENGE_PRECOMMIT_PHASE_V2].clone(), 1),
                );
                for (tag, challenge_phase, close_phase) in [
                    (
                        0_usize,
                        CHALLENGE_PRECOMMIT_PHASE_V2,
                        CLOSE_PRECOMMIT_PHASE_V2,
                    ),
                    (1, CHALLENGE_CONTEXT_PHASE_V2, CLOSE_CONTEXT_PHASE_V2),
                    (
                        2,
                        CHALLENGE_SPENT_PRECOMMIT_PHASE_V2,
                        CLOSE_SPENT_PRECOMMIT_PHASE_V2,
                    ),
                    (
                        3,
                        CHALLENGE_OUTPUT_PRECOMMIT_PHASE_V2,
                        CLOSE_OUTPUT_PRECOMMIT_PHASE_V2,
                    ),
                ] {
                    builder.push_interaction(
                        SOURCE_CHALLENGE_CLOSE_BYTE_BUS_V2,
                        vec![
                            transition_index.clone(),
                            AB::Expr::from_usize(tag),
                            field_byte(challenge_phase),
                            byte.clone(),
                        ],
                        Count::bounded(transcript_phases[challenge_phase].clone(), 1),
                    );
                    builder.push_interaction(
                        SOURCE_CHALLENGE_CLOSE_BYTE_BUS_V2,
                        vec![
                            transition_index.clone(),
                            AB::Expr::from_usize(tag),
                            field_byte(close_phase),
                            byte.clone(),
                        ],
                        -Count::bounded(transcript_phases[close_phase].clone(), 1),
                    );
                }
                for (tag, phase, public_offset) in [
                    (
                        0_usize,
                        CHALLENGE_CONTEXT_PHASE_V2,
                        PUBLIC_PRE_UNIQUENESS_BYTE_OFFSET_V2,
                    ),
                    (
                        1,
                        CHALLENGE_SPENT_PRECOMMIT_PHASE_V2,
                        PUBLIC_SPENT_PRECOMMIT_BYTE_OFFSET_V2,
                    ),
                    (
                        2,
                        CHALLENGE_OUTPUT_PRECOMMIT_PHASE_V2,
                        PUBLIC_OUTPUT_PRECOMMIT_BYTE_OFFSET_V2,
                    ),
                ] {
                    builder.push_interaction(
                        SOURCE_BINDING_DIGEST_BYTE_BUS_V2,
                        vec![
                            transition_index.clone(),
                            AB::Expr::from_usize(tag),
                            field_byte(phase),
                            byte.clone(),
                        ],
                        Count::bounded(transcript_phases[phase].clone(), 1),
                    );
                    for byte_index in 0..32 {
                        builder.push_interaction(
                            SOURCE_BINDING_DIGEST_BYTE_BUS_V2,
                            vec![
                                transition_index.clone(),
                                AB::Expr::from_usize(tag),
                                AB::Expr::from_usize(byte_index),
                                selected_digest_byte(public_offset, byte_index),
                            ],
                            -Count::bounded(prefix[0].clone(), 1),
                        );
                    }
                }
                builder.push_interaction(
                    SOURCE_NET_EFFECT_BYTE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        net_effect_counter.clone(),
                        payload_index.clone(),
                        byte.clone(),
                    ],
                    Count::bounded(payload.clone() * net_non_close.clone(), 1),
                );
                let replay_opcode = replay_input.clone() + replay_output.clone();
                let replay_payload = payload.clone() * replay_opcode.clone();
                for selector in &replay_phases {
                    builder.assert_bool(selector.clone());
                }
                let replay_phase_active = replay_phases
                    .iter()
                    .cloned()
                    .fold(AB::Expr::ZERO, |sum, value| sum + value);
                builder.assert_eq(replay_phase_active.clone(), replay_payload);
                builder.assert_bool(replay_phase_final.clone());
                builder.assert_zero(
                    replay_phase_final.clone() * (one.clone() - replay_phase_active.clone()),
                );
                let replay_remaining_minus_one =
                    replay_remaining.clone() - replay_phase_active.clone();
                builder
                    .assert_zero(replay_phase_final.clone() * replay_remaining_minus_one.clone());
                builder.assert_eq(
                    replay_remaining_minus_one * replay_remaining_inverse.clone(),
                    replay_phase_active.clone() - replay_phase_final.clone(),
                );
                builder.assert_zero(
                    (one.clone() - replay_phase_active.clone()) * replay_remaining_inverse.clone(),
                );
                builder.assert_zero(
                    (one.clone() - replay_phase_active.clone()) * replay_remaining.clone(),
                );
                builder
                    .assert_zero(replay_opcode.clone() * (one.clone() - payload_nonzero.clone()));
                builder.assert_eq(
                    payload_final.clone() * replay_opcode.clone(),
                    replay_phases[REPLAY_FLAGS_PHASE_V2].clone() * replay_phase_final.clone(),
                );

                let replay_tx_len = replay_tx_len_low.clone()
                    + replay_tx_len_high.clone() * AB::Expr::from_u64(256);
                builder.assert_bool(replay_tx_len_high.clone());
                builder.assert_zero(replay_tx_len_high.clone() * replay_tx_len_low.clone());
                builder.assert_eq(
                    replay_tx_len * replay_tx_len_inverse.clone(),
                    replay_phase_active.clone(),
                );
                for value in [
                    replay_tx_len_low.clone(),
                    replay_tx_len_high.clone(),
                    replay_tx_len_inverse.clone(),
                ] {
                    builder.assert_zero((one.clone() - replay_phase_active.clone()) * value);
                }
                builder.assert_zero(
                    replay_phases[REPLAY_TX_LEN_LOW_PHASE_V2].clone()
                        * (replay_tx_len_low.clone() - byte.clone()),
                );
                builder.assert_zero(
                    replay_phases[REPLAY_TX_LEN_HIGH_PHASE_V2].clone()
                        * (replay_tx_len_high.clone() - byte.clone()),
                );

                let replay_expected_op_kind =
                    replay_input.clone() * AB::Expr::from_u64(2) + replay_output.clone();
                builder.assert_zero(
                    replay_phases[REPLAY_OP_KIND_PHASE_V2].clone()
                        * (byte.clone() - replay_expected_op_kind),
                );
                for (phase, expected) in [
                    (REPLAY_DEFINITION_LEN_LOW_PHASE_V2, 64_u64),
                    (REPLAY_DEFINITION_LEN_HIGH_PHASE_V2, 0),
                    (REPLAY_TERMINAL_LEN_LOW_PHASE_V2, 64),
                    (REPLAY_TERMINAL_LEN_HIGH_PHASE_V2, 0),
                ] {
                    builder.assert_zero(
                        replay_phases[phase].clone()
                            * (byte.clone() - AB::Expr::from_u64(expected)),
                    );
                }
                builder.assert_zero(
                    replay_phases[REPLAY_LEAF_KIND_PHASE_V2].clone()
                        * replay_terminal_leaf_kind_polynomial(byte.clone()),
                );
                builder
                    .when(replay_phases[REPLAY_FLAGS_PHASE_V2].clone())
                    .assert_bool(byte.clone());
                builder.assert_zero(
                    replay_input.clone()
                        * replay_phases[REPLAY_FLAGS_PHASE_V2].clone()
                        * byte.clone(),
                );

                let replay_tx_bytes = replay_phases[REPLAY_TX_BYTES_PHASE_V2].clone();
                builder.assert_zero(
                    replay_tx_bytes.clone()
                        * (replay_ascii_low.clone() - byte.clone() + AB::Expr::from_u64(33)),
                );
                builder.assert_zero(
                    replay_tx_bytes.clone()
                        * (replay_ascii_high.clone() + byte.clone() - AB::Expr::from_u64(126)),
                );
                builder.assert_zero(
                    (one.clone() - replay_tx_bytes.clone()) * replay_ascii_low.clone(),
                );
                builder.assert_zero(
                    (one.clone() - replay_tx_bytes.clone()) * replay_ascii_high.clone(),
                );
                builder.push_interaction(
                    RANGE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        replay_ascii_low,
                        replay_ascii_high,
                        AB::Expr::ZERO,
                    ],
                    Count::bounded(-replay_tx_bytes, 1),
                );

                for selector in &replay_hex_selectors {
                    builder.assert_bool(selector.clone());
                }
                let replay_hex_active = replay_phases[REPLAY_DEFINITION_HEX_PHASE_V2].clone()
                    + replay_phases[REPLAY_TERMINAL_HEX_PHASE_V2].clone();
                builder.assert_eq(
                    replay_hex_selectors
                        .iter()
                        .cloned()
                        .fold(AB::Expr::ZERO, |sum, value| sum + value),
                    replay_hex_active.clone(),
                );
                let replay_hex_nibble = replay_hex_selectors
                    .iter()
                    .enumerate()
                    .fold(AB::Expr::ZERO, |sum, (index, selector)| {
                        sum + selector.clone() * AB::Expr::from_usize(index)
                    });
                let replay_hex_ascii = replay_hex_selectors.iter().enumerate().fold(
                    AB::Expr::ZERO,
                    |sum, (index, selector)| {
                        let ascii = if index < 10 {
                            b'0' + u8::try_from(index).expect("hex selector index fits u8")
                        } else {
                            b'a' + u8::try_from(index - 10).expect("hex selector index fits u8")
                        };
                        sum + selector.clone() * AB::Expr::from_u64(u64::from(ascii))
                    },
                );
                builder.assert_eq(replay_hex_active.clone() * byte.clone(), replay_hex_ascii);
                builder.assert_bool(replay_hex_low.clone());
                builder.assert_zero(
                    replay_hex_low.clone() * (one.clone() - replay_hex_active.clone()),
                );
                builder.assert_zero(
                    replay_hex_active.clone()
                        * replay_phase_final.clone()
                        * (one.clone() - replay_hex_low.clone()),
                );
                builder.assert_zero(
                    (one.clone() - replay_hex_active.clone()) * replay_hex_high_nibble.clone(),
                );
                let replay_hex_high_row =
                    replay_hex_active.clone() * (one.clone() - replay_hex_low.clone());
                builder.assert_zero(
                    replay_hex_high_row.clone()
                        * (replay_hex_high_nibble.clone() - replay_hex_nibble.clone()),
                );

                let replay_definition_emit =
                    replay_phases[REPLAY_DEFINITION_HEX_PHASE_V2].clone() * replay_hex_low.clone();
                let replay_terminal_emit =
                    replay_phases[REPLAY_TERMINAL_HEX_PHASE_V2].clone() * replay_hex_low.clone();
                let replay_serial_emit = replay_phases[REPLAY_SERIAL_PHASE_V2].clone();
                let replay_leaf_emit = replay_phases[REPLAY_LEAF_HASH_PHASE_V2].clone();
                let replay_semantic_emit = replay_definition_emit.clone()
                    + replay_serial_emit.clone()
                    + replay_terminal_emit.clone()
                    + replay_leaf_emit.clone();
                builder.assert_zero(
                    (one.clone() - replay_semantic_emit.clone()) * replay_semantic_index.clone(),
                );
                builder.assert_zero(
                    replay_definition_emit.clone()
                        * (replay_semantic_index.clone() * AB::Expr::from_u64(2)
                            + replay_remaining.clone()
                            - AB::Expr::from_u64(63)),
                );
                builder.assert_zero(
                    replay_serial_emit.clone()
                        * (replay_semantic_index.clone() + replay_remaining.clone()
                            - AB::Expr::from_u64(36)),
                );
                builder.assert_zero(
                    replay_terminal_emit.clone()
                        * (replay_semantic_index.clone() * AB::Expr::from_u64(2)
                            + replay_remaining.clone()
                            - AB::Expr::from_u64(135)),
                );
                builder.assert_zero(
                    replay_leaf_emit.clone()
                        * (replay_semantic_index.clone() + replay_remaining.clone()
                            - AB::Expr::from_u64(100)),
                );

                let replay_set = replay_output.clone();
                let replay_position = replay_input.clone() * replay_counters[0].clone()
                    + replay_output.clone() * replay_counters[1].clone();
                let replay_hex_byte =
                    replay_hex_high_nibble.clone() * AB::Expr::from_u64(16) + replay_hex_nibble;
                let replay_object_header = header[9..41]
                    .iter()
                    .cloned()
                    .fold(AB::Expr::ZERO, |sum, selector| sum + selector);
                let replay_object_index = header[9..41]
                    .iter()
                    .enumerate()
                    .fold(AB::Expr::ZERO, |sum, (index, selector)| {
                        sum + selector.clone() * AB::Expr::from_usize(index)
                    });
                builder.push_interaction(
                    SOURCE_REPLAY_OBJECT_BYTE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        replay_set.clone(),
                        replay_position.clone(),
                        replay_object_index,
                        byte.clone(),
                    ],
                    Count::bounded(
                        replay_object_header * (replay_input.clone() + replay_output.clone()),
                        1,
                    ),
                );
                builder.push_interaction(
                    SOURCE_REPLAY_OBJECT_BYTE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        replay_set.clone(),
                        replay_position.clone(),
                        replay_semantic_index.clone() - AB::Expr::from_u64(36),
                        replay_hex_byte.clone(),
                    ],
                    Count::bounded(-replay_terminal_emit.clone(), 1),
                );
                for (gate, semantic_byte) in [
                    (replay_definition_emit, replay_hex_byte.clone()),
                    (replay_serial_emit, byte.clone()),
                    (replay_terminal_emit, replay_hex_byte),
                    (replay_leaf_emit, byte.clone()),
                ] {
                    builder.push_interaction(
                        SOURCE_REPLAY_SEMANTIC_BYTE_BUS_V2,
                        vec![
                            transition_index.clone(),
                            replay_set.clone(),
                            replay_position.clone(),
                            replay_semantic_index.clone(),
                            semantic_byte,
                        ],
                        Count::bounded(gate, 1),
                    );
                }

                for (class, selector) in uniqueness_classes.iter().enumerate() {
                    let counter = if class >= 6 { 6 } else { class };
                    builder.push_interaction(
                        SOURCE_UNIQUENESS_PAYLOAD_BYTE_BUS_V2,
                        vec![
                            transition_index.clone(),
                            AB::Expr::from_usize(class),
                            uniqueness_counters[counter].clone(),
                            payload_index.clone(),
                            byte.clone(),
                        ],
                        Count::bounded(payload.clone() * selector.clone(), 1),
                    );
                }
            }
        }

        let declared_spent_count = declared_count_bytes[..4]
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (index, byte)| {
                sum + byte.clone() * AB::Expr::from_u64(1_u64 << (index * 8))
            });
        let declared_output_count = declared_count_bytes[4..]
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (index, byte)| {
                sum + byte.clone() * AB::Expr::from_u64(1_u64 << (index * 8))
            });
        // The native predicate permits Insert/Delete effects inside a transition,
        // but a complete non-noop transition must contain at least one replay row
        // on each side. A no-op transition must contain neither.
        for (declared, inverse) in [
            (&declared_spent_count, &declared_spent_inverse),
            (&declared_output_count, &declared_output_inverse),
        ] {
            for polynomial in nonzero_indicator_polynomials(
                net_mutation_nonzero.clone(),
                declared.clone(),
                inverse.clone(),
            ) {
                builder.assert_zero(slot_end.clone() * polynomial);
            }
        }
        if self.role == SemanticSourceAirRoleV2::Uniqueness {
            for high_byte in [2_usize, 3, 6, 7] {
                builder.assert_zero(slot_end.clone() * declared_count_bytes[high_byte].clone());
            }
            let declared_spent_low = declared_count_bytes[0].clone()
                + declared_count_bytes[1].clone() * AB::Expr::from_u64(256);
            let declared_output_low = declared_count_bytes[4].clone()
                + declared_count_bytes[5].clone() * AB::Expr::from_u64(256);
            for (declared, slack) in [
                (
                    declared_spent_low,
                    declared_count_slack_bytes[0].clone()
                        + declared_count_slack_bytes[1].clone() * AB::Expr::from_u64(256),
                ),
                (
                    declared_output_low,
                    declared_count_slack_bytes[2].clone()
                        + declared_count_slack_bytes[3].clone() * AB::Expr::from_u64(256),
                ),
            ] {
                builder.assert_zero(slot_end.clone() * bounded_count_polynomial(declared, slack));
            }
            for pair in declared_count_slack_bytes.chunks_exact(2) {
                builder.push_interaction(
                    RANGE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        pair[0].clone(),
                        pair[1].clone(),
                        AB::Expr::ZERO,
                    ],
                    Count::bounded(-slot_end.clone(), 1),
                );
            }
        }
        for (declared, uniqueness_counter) in [
            (&declared_spent_count, &uniqueness_counters[0]),
            (&declared_output_count, &uniqueness_counters[1]),
            (&declared_spent_count, &uniqueness_counters[2]),
            (&declared_output_count, &uniqueness_counters[3]),
            (&declared_spent_count, &uniqueness_counters[4]),
            (&declared_output_count, &uniqueness_counters[5]),
        ] {
            builder.assert_zero(slot_end.clone() * (declared.clone() - uniqueness_counter.clone()));
        }
        builder.assert_zero(
            slot_end.clone()
                * (uniqueness_counters[6].clone()
                    - declared_spent_count.clone()
                    - declared_output_count.clone()),
        );
        if self.role == SemanticSourceAirRoleV2::Uniqueness {
            for (replay_counter, declared) in [
                (&replay_counters[0], &declared_spent_count),
                (&replay_counters[1], &declared_output_count),
            ] {
                builder.assert_zero(slot_end.clone() * (replay_counter.clone() - declared.clone()));
            }
        }

        {
            let mut first = builder.when_first_row();
            first.assert_one(transition_selectors[0].clone());
            first.assert_one(prefix[0].clone());
            first.assert_zero(byte_index.clone());
            first.assert_zero(event_index.clone());
            first.assert_one(jmt_stage[0].clone());
            first.assert_zero(jmt_count.clone());
            for counter in &uniqueness_counters {
                first.assert_zero(counter.clone());
            }
            first.assert_zero(net_effect_counter.clone());
            first.assert_zero(net_mutation_counter.clone());
            first.assert_zero(net_terminal_countdown.clone());
            first.assert_zero(net_terminal_countdown_active.clone());
            for selector in &net_terminal_byte_selectors {
                first.assert_zero(selector.clone());
            }
            for counter in &fixed_event_counters {
                first.assert_zero(counter.clone());
            }
            for count_byte in &declared_count_bytes {
                first.assert_zero(count_byte.clone());
            }
            for selector in &flow_count_byte_selectors {
                first.assert_zero(selector.clone());
            }
            first.assert_zero(flow_count_countdown.clone());
            first.assert_zero(flow_count_countdown_active.clone());
            first.assert_zero(net_mutation_nonzero.clone());
            first.assert_zero(declared_spent_inverse.clone());
            first.assert_zero(declared_output_inverse.clone());
            for counter in &replay_counters {
                first.assert_zero(counter.clone());
            }
        }

        let next_slot_active = transition_selectors.iter().enumerate().fold(
            AB::Expr::ZERO,
            |sum, (slot, selector)| {
                sum + selector.clone()
                    * public_active
                        .get(slot + 1)
                        .cloned()
                        .unwrap_or(AB::Expr::ZERO)
            },
        );
        let continues = active.clone() - slot_end.clone();
        let event_continues = event_active.clone() - event_final.clone();
        {
            let mut transition = builder.when_transition();
            transition.assert_eq(
                next_active.clone(),
                continues.clone() + slot_end.clone() * next_slot_active.clone(),
            );
            for slot in 0..TRANSITION_SLOTS_V2 {
                let incoming = if slot == 0 {
                    AB::Expr::ZERO
                } else {
                    slot_end.clone()
                        * transition_selectors[slot - 1].clone()
                        * public_active[slot].clone()
                };
                transition.assert_eq(
                    next_transition_selectors[slot].clone(),
                    continues.clone() * transition_selectors[slot].clone() + incoming,
                );
            }
            transition.assert_eq(
                next_byte_index,
                continues.clone() * (byte_index + one.clone()),
            );
            transition.assert_eq(
                next_event_index,
                continues.clone() * (event_index + event_final.clone()),
            );

            transition.assert_eq(
                next_prefix[0].clone(),
                slot_end.clone() * next_slot_active.clone(),
            );
            for index in 1..PREFIX_BYTES_V2 {
                transition.assert_eq(next_prefix[index].clone(), prefix[index - 1].clone());
            }
            transition.assert_eq(
                next_length[0].clone(),
                prefix[PREFIX_BYTES_V2 - 1].clone()
                    + event_final.clone() * (one.clone() - slot_end.clone()),
            );
            for index in 1..LENGTH_BYTES_V2 {
                transition.assert_eq(next_length[index].clone(), length[index - 1].clone());
            }
            transition.assert_eq(next_header[0].clone(), length[LENGTH_BYTES_V2 - 1].clone());
            for index in 1..HEADER_BYTES_V2 {
                transition.assert_eq(next_header[index].clone(), header[index - 1].clone());
            }
            transition.assert_eq(
                next_payload,
                header[HEADER_BYTES_V2 - 1].clone() * payload_nonzero.clone()
                    + payload.clone() * (one.clone() - payload_final.clone()),
            );
            transition.assert_eq(
                next_payload_index,
                payload.clone()
                    * (one.clone() - payload_final.clone())
                    * (payload_index.clone() + one.clone()),
            );
            transition.assert_eq(
                next_payload_prefix[0].clone(),
                header[HEADER_BYTES_V2 - 1].clone() * payload_nonzero.clone(),
            );
            for index in 1..PAYLOAD_PREFIX_BYTES_V2 {
                transition.assert_eq(
                    next_payload_prefix[index].clone(),
                    payload_prefix[index - 1].clone() * (one.clone() - payload_final.clone()),
                );
            }

            for (local_value, next_value) in event_len_bytes
                .iter()
                .zip(&next_event_len_bytes)
                .chain(payload_len_bytes.iter().zip(&next_payload_len_bytes))
                .chain(ordinal_bytes.iter().zip(&next_ordinal_bytes))
                .chain(opcode_selectors.iter().zip(&next_opcode_selectors))
                .chain(uniqueness_classes.iter().zip(&next_uniqueness_classes))
                .chain(net_kind_selectors.iter().zip(&next_net_kind_selectors))
                .chain(net_terminal_limbs.iter().zip(&next_net_terminal_limbs))
                .chain(declared_count_bytes.iter().zip(&next_declared_count_bytes))
            {
                transition.assert_zero(
                    event_continues.clone() * (next_value.clone() - local_value.clone()),
                );
            }
            transition.assert_zero(
                event_continues.clone() * (next_payload_nonzero.clone() - payload_nonzero.clone()),
            );
            transition.assert_zero(
                event_continues.clone() * (next_payload_inverse - payload_inverse.clone()),
            );
            transition.assert_zero(global_pair_transition_polynomial(
                event_continues,
                event_final.clone(),
                global_product_pair,
                product_spent,
                next_product_output,
                next_global_product_pair,
            ));

            let header_done = event_final.clone() * jmt_header;
            let promotion_done = event_final.clone() * jmt_promotion;
            transition.assert_eq(
                next_jmt_stage[0].clone(),
                slot_end.clone() * next_slot_active.clone()
                    + (one.clone() - slot_end.clone())
                        * (jmt_stage[0].clone() - header_done.clone()),
            );
            transition.assert_eq(
                next_jmt_stage[1].clone(),
                (one.clone() - slot_end.clone())
                    * (jmt_stage[1].clone() + header_done - promotion_done.clone()),
            );
            transition.assert_eq(
                next_jmt_stage[2].clone(),
                (one.clone() - slot_end.clone()) * (jmt_stage[2].clone() + promotion_done),
            );
            transition.assert_eq(
                next_jmt_count,
                (one.clone() - slot_end.clone()) * (jmt_count + event_final.clone() * jmt_micro),
            );
            for counter in 0..UNIQUENESS_COUNTER_COUNT_V2 {
                let increment = uniqueness_classes
                    .iter()
                    .enumerate()
                    .filter(|(class, _)| {
                        if *class >= 6 {
                            counter == 6
                        } else {
                            *class == counter
                        }
                    })
                    .fold(AB::Expr::ZERO, |sum, (_, selector)| sum + selector.clone());
                transition.assert_eq(
                    next_uniqueness_counters[counter].clone(),
                    (one.clone() - slot_end.clone())
                        * (uniqueness_counters[counter].clone() + event_final.clone() * increment),
                );
            }
            transition.assert_eq(
                next_net_effect_counter,
                (one.clone() - slot_end.clone())
                    * (net_effect_counter.clone() + event_final.clone() * net_non_close.clone()),
            );
            transition.assert_eq(
                next_net_mutation_counter,
                (one.clone() - slot_end.clone())
                    * (net_mutation_counter.clone() + event_final.clone() * net_mutation.clone()),
            );
            for counter in 0..FIXED_EVENT_COUNTER_COUNT_V2 {
                transition.assert_eq(
                    next_fixed_event_counters[counter].clone(),
                    (one.clone() - slot_end.clone())
                        * (fixed_event_counters[counter].clone()
                            + event_final.clone() * fixed_event_gates[counter].clone()),
                );
            }
            let next_precommit = next_opcode_selectors
                [RecursiveTraceOpcodeV2::UniquenessPrecommit as usize - 1]
                .clone();
            let declared_count_write = event_final.clone() * next_precommit;
            let declared_count_carry = continues.clone() - declared_count_write;
            for (declared, next_declared) in
                declared_count_bytes.iter().zip(&next_declared_count_bytes)
            {
                transition.assert_zero(
                    declared_count_carry.clone() * (next_declared.clone() - declared.clone()),
                );
                transition.assert_zero(
                    slot_end.clone() * next_slot_active.clone() * next_declared.clone(),
                );
            }
            let flow_count_countdown_start =
                payload_prefix[PAYLOAD_PREFIX_BYTES_V2 - 1].clone() * flow_header;
            transition.assert_eq(
                next_flow_count_countdown,
                flow_count_countdown_start
                    * AB::Expr::from_usize(
                        FLOW_HEADER_COUNT_PAYLOAD_START_V2 - PAYLOAD_PREFIX_BYTES_V2,
                    )
                    + flow_count_countdown_active.clone()
                        * (flow_count_countdown.clone() - one.clone()),
            );
            transition.assert_eq(
                next_flow_count_byte_selectors[0].clone(),
                flow_count_countdown_active.clone()
                    * (one.clone() - next_flow_count_countdown_active),
            );
            for count_byte in 1..FLOW_COUNT_BYTE_SELECTOR_COUNT_V2 {
                transition.assert_eq(
                    next_flow_count_byte_selectors[count_byte].clone(),
                    flow_count_byte_selectors[count_byte - 1].clone(),
                );
            }
            let net_terminal_countdown_start =
                payload_prefix[PAYLOAD_PREFIX_BYTES_V2 - 1].clone() * net_opcode.clone();
            transition.assert_eq(
                next_net_terminal_countdown,
                net_terminal_countdown_start
                    * AB::Expr::from_usize(NET_TERMINAL_PAYLOAD_START_V2 - PAYLOAD_PREFIX_BYTES_V2)
                    + net_terminal_countdown_active.clone()
                        * (net_terminal_countdown.clone() - one.clone()),
            );
            transition.assert_eq(
                next_net_terminal_byte_selectors[0].clone(),
                net_terminal_countdown_active.clone()
                    * (one.clone() - next_net_terminal_countdown_active),
            );
            for terminal_byte in 1..NET_TERMINAL_BYTE_SELECTOR_COUNT_V2 {
                transition.assert_eq(
                    next_net_terminal_byte_selectors[terminal_byte].clone(),
                    net_terminal_byte_selectors[terminal_byte - 1].clone(),
                );
            }
            if self.role == SemanticSourceAirRoleV2::Uniqueness {
                let replay_input =
                    opcode_selectors[RecursiveTraceOpcodeV2::ReplayInput as usize - 1].clone();
                let replay_output =
                    opcode_selectors[RecursiveTraceOpcodeV2::ReplayOutput as usize - 1].clone();
                let replay_opcode = replay_input.clone() + replay_output.clone();
                let replay_start = header[HEADER_BYTES_V2 - 1].clone() * replay_opcode;
                let replay_phase_active = replay_phases
                    .iter()
                    .cloned()
                    .fold(AB::Expr::ZERO, |sum, value| sum + value);
                transition.assert_eq(next_replay_phases[0].clone(), replay_start.clone());
                for phase in 1..REPLAY_PHASE_COUNT_V2 {
                    transition.assert_eq(
                        next_replay_phases[phase].clone(),
                        replay_phases[phase].clone() * (one.clone() - replay_phase_final.clone())
                            + replay_phases[phase - 1].clone() * replay_phase_final.clone(),
                    );
                }

                let replay_tx_len = replay_tx_len_low.clone()
                    + replay_tx_len_high.clone() * AB::Expr::from_u64(256);
                let next_phase_remaining = [
                    AB::Expr::ONE,
                    AB::Expr::ONE,
                    replay_tx_len,
                    AB::Expr::ONE,
                    AB::Expr::ONE,
                    AB::Expr::from_u64(64),
                    AB::Expr::from_u64(4),
                    AB::Expr::ONE,
                    AB::Expr::ONE,
                    AB::Expr::from_u64(64),
                    AB::Expr::from_u64(32),
                    AB::Expr::ONE,
                    AB::Expr::from_u64(3),
                ];
                let expected_next_remaining = replay_phases
                    .iter()
                    .take(REPLAY_PHASE_COUNT_V2 - 1)
                    .zip(next_phase_remaining)
                    .fold(
                        replay_start
                            + replay_phase_active.clone()
                                * (one.clone() - replay_phase_final.clone())
                                * (replay_remaining.clone() - one.clone()),
                        |sum, (phase, next_remaining)| {
                            sum + phase.clone() * replay_phase_final.clone() * next_remaining
                        },
                    );
                transition.assert_eq(next_replay_remaining, expected_next_remaining);

                let replay_payload_continues =
                    replay_phase_active.clone() * (one.clone() - payload_final.clone());
                transition.assert_zero(
                    replay_payload_continues.clone()
                        * (next_replay_tx_len_low - replay_tx_len_low.clone()),
                );
                transition.assert_zero(
                    replay_payload_continues
                        * (next_replay_tx_len_high - replay_tx_len_high.clone()),
                );

                let replay_hex_active = replay_phases[REPLAY_DEFINITION_HEX_PHASE_V2].clone()
                    + replay_phases[REPLAY_TERMINAL_HEX_PHASE_V2].clone();
                let replay_hex_continues =
                    replay_hex_active.clone() * (one.clone() - replay_phase_final.clone());
                transition.assert_zero(
                    replay_hex_continues
                        * (next_replay_hex_low.clone() + replay_hex_low.clone() - one.clone()),
                );
                let replay_hex_start = (replay_phases[REPLAY_DEFINITION_LEN_HIGH_PHASE_V2].clone()
                    + replay_phases[REPLAY_TERMINAL_LEN_HIGH_PHASE_V2].clone())
                    * replay_phase_final.clone();
                transition.assert_zero(replay_hex_start * next_replay_hex_low);
                let replay_hex_high_row =
                    replay_hex_active * (one.clone() - replay_hex_low.clone());
                transition.assert_zero(
                    replay_hex_high_row
                        * (next_replay_hex_high_nibble - replay_hex_high_nibble.clone()),
                );

                for counter in 0..REPLAY_COUNTER_COUNT_V2 {
                    let increment = if counter == 0 {
                        replay_input.clone()
                    } else {
                        replay_output.clone()
                    };
                    transition.assert_eq(
                        next_replay_counters[counter].clone(),
                        (one.clone() - slot_end.clone())
                            * (replay_counters[counter].clone() + event_final.clone() * increment),
                    );
                }
            }
        }

        let prefix_metadata = prefix_active;
        for value in event_len_bytes
            .iter()
            .chain(payload_len_bytes.iter())
            .chain(ordinal_bytes.iter())
            .chain(opcode_selectors.iter())
            .chain(uniqueness_classes.iter())
            .chain(net_kind_selectors.iter())
        {
            builder.assert_zero(prefix_metadata.clone() * value.clone());
        }
        for value in [
            payload_index,
            payload_nonzero,
            payload_inverse,
            payload_final,
            slot_end,
        ] {
            builder.assert_zero(prefix_metadata.clone() * value);
        }

        let inactive = one - active;
        for offset in TRANSITION_SELECTOR_OFFSET_V2..ROW_FIELDS_V2 {
            builder.assert_zero(inactive.clone() * field::<AB>(local, offset));
        }
    }
}

impl BatchAir<Plonky3StarkConfigV2> for SemanticSourceAirV2 {}

#[derive(Clone, Copy, Debug)]
pub(super) struct SemanticSourceProverV2 {
    role: SemanticSourceAirRoleV2,
}

impl SemanticSourceProverV2 {
    pub(super) const fn new(role: SemanticSourceAirRoleV2) -> Self {
        Self { role }
    }

    fn batch_instance(
        &self,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces.non_primitive_trace::<SemanticSourceTraceV2>(&self.role.npo_type())?;
        if trace.role != self.role
            || trace.public_values.len() != PUBLIC_FIELDS_V2
            || trace.rows.len() < MIN_ROWS_V2
            || !trace.rows.len().is_power_of_two()
            || trace
                .rows
                .iter()
                .any(|row| row.values.len() != ROW_FIELDS_V2)
        {
            return None;
        }
        Some(BatchTableInstance {
            op_type: self.role.npo_type(),
            air: DynamicAirEntry::new(Box::new(SemanticSourceAirV2::new(self.role))),
            trace: SemanticSourceAirV2::trace_to_matrix(&trace.rows),
            public_values: trace.public_values.clone(),
            rows: trace.rows.len(),
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for SemanticSourceProverV2 {
    fn op_type(&self) -> NpoTypeId {
        self.role.npo_type()
    }

    fn batch_instance_d1(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        self.batch_instance(traces)
    }

    fn batch_instance_d2(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 2>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d4(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 4>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d6(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 6>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d8(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 8>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_air_from_table_entry(
        &self,
        _config: &Plonky3StarkConfigV2,
        degree: usize,
        circuit_extension_degree: u32,
        entry: &NonPrimitiveTableEntry<Plonky3StarkConfigV2>,
    ) -> Result<DynamicAirEntry<Plonky3StarkConfigV2>, String> {
        if degree != 1
            || circuit_extension_degree != 1
            || entry.op_type != self.role.npo_type()
            || entry.public_values.len() != PUBLIC_FIELDS_V2
            || entry.rows < MIN_ROWS_V2
            || !entry.rows.is_power_of_two()
            || entry.lanes != 1
        {
            return Err("epoch semantic-source table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(SemanticSourceAirV2::new(
            self.role,
        ))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_leaf_kind_accepts_terminal_only() {
        assert_eq!(
            replay_terminal_leaf_kind_polynomial(KoalaBear::from_u8(1)),
            KoalaBear::ZERO,
        );
        for rejected in [0_u8, 2, 3, u8::MAX] {
            assert_ne!(
                replay_terminal_leaf_kind_polynomial(KoalaBear::from_u8(rejected)),
                KoalaBear::ZERO,
            );
        }
    }

    #[test]
    fn nonzero_indicator_requires_exact_zero_status() {
        let accepts = |indicator: KoalaBear, value: KoalaBear, inverse: KoalaBear| {
            nonzero_indicator_polynomials(indicator, value, inverse)
                .into_iter()
                .all(|polynomial| polynomial.is_zero())
        };

        assert!(accepts(KoalaBear::ZERO, KoalaBear::ZERO, KoalaBear::ZERO,));
        assert!(accepts(
            KoalaBear::ONE,
            KoalaBear::from_u8(7),
            KoalaBear::from_u8(7).inverse(),
        ));
        assert!(!accepts(KoalaBear::ONE, KoalaBear::ZERO, KoalaBear::ZERO,));
        assert!(!accepts(KoalaBear::ZERO, KoalaBear::ONE, KoalaBear::ZERO,));
        assert!(!accepts(
            KoalaBear::ONE,
            KoalaBear::from_u8(7),
            KoalaBear::ONE,
        ));
    }

    #[test]
    fn bounded_count_matches_native_limit() {
        assert_eq!(
            bounded_count_polynomial(
                KoalaBear::ZERO,
                KoalaBear::from_usize(DECLARED_ITEM_LIMIT_V2),
            ),
            KoalaBear::ZERO,
        );
        assert_eq!(
            bounded_count_polynomial(
                KoalaBear::from_usize(DECLARED_ITEM_LIMIT_V2),
                KoalaBear::ZERO,
            ),
            KoalaBear::ZERO,
        );
        assert_eq!(
            bounded_count_polynomial(
                KoalaBear::from_usize(DECLARED_ITEM_LIMIT_V2 - 1),
                KoalaBear::ONE,
            ),
            KoalaBear::ZERO,
        );
        assert_ne!(
            bounded_count_polynomial(
                KoalaBear::from_usize(DECLARED_ITEM_LIMIT_V2 + 1),
                KoalaBear::ZERO,
            ),
            KoalaBear::ZERO,
        );
    }

    #[test]
    fn global_product_net_schedule_rejects_reordering() {
        let schedule_accepts = |current: [u8; 4], next: [u8; 7]| {
            global_schedule_polynomials(
                KoalaBear::from_u8(current[0]),
                KoalaBear::from_u8(current[1]),
                KoalaBear::from_u8(current[2]),
                KoalaBear::from_u8(current[3]),
                KoalaBear::from_u8(next[0]),
                KoalaBear::from_u8(next[1]),
                KoalaBear::from_u8(next[2]),
                KoalaBear::from_u8(next[3]),
                KoalaBear::from_u8(next[4]),
                KoalaBear::from_u8(next[5]),
                KoalaBear::from_u8(next[6]),
            )
            .into_iter()
            .all(|value| value.is_zero())
        };

        assert!(schedule_accepts([1, 0, 0, 0], [0, 0, 0, 1, 0, 0, 0]));
        assert!(schedule_accepts([1, 0, 0, 0], [0, 1, 0, 0, 0, 0, 0]));
        assert!(schedule_accepts([0, 1, 0, 0], [0, 0, 0, 0, 1, 0, 0]));
        assert!(schedule_accepts([0, 1, 1, 0], [0, 0, 0, 0, 0, 1, 0]));
        assert!(schedule_accepts([0, 1, 1, 0], [0, 0, 0, 0, 0, 0, 1]));
        assert!(schedule_accepts([0, 0, 0, 1], [1, 0, 0, 0, 0, 0, 0]));
        assert!(schedule_accepts([0, 0, 0, 1], [0, 1, 0, 0, 0, 0, 0]));
        assert!(schedule_accepts([0, 0, 0, 1], [0, 0, 1, 0, 0, 0, 0]));

        assert!(!schedule_accepts([1, 0, 0, 0], [1, 0, 0, 0, 0, 0, 0]));
        assert!(!schedule_accepts([0, 1, 0, 0], [0, 0, 0, 1, 0, 0, 0]));
        assert!(!schedule_accepts([0, 1, 1, 0], [0, 0, 0, 0, 1, 0, 0]));
        assert!(!schedule_accepts([0, 0, 0, 1], [0, 0, 0, 1, 0, 0, 0]));
        assert_eq!(
            global_schedule_entry_polynomial(KoalaBear::ONE, KoalaBear::ONE),
            KoalaBear::ZERO,
        );
        assert_ne!(
            global_schedule_entry_polynomial(KoalaBear::ZERO, KoalaBear::ONE),
            KoalaBear::ZERO,
        );

        assert_eq!(
            global_pair_transition_polynomial(
                KoalaBear::ZERO,
                KoalaBear::ONE,
                KoalaBear::ZERO,
                KoalaBear::ONE,
                KoalaBear::ONE,
                KoalaBear::ONE,
            ),
            KoalaBear::ZERO,
        );
        assert_ne!(
            global_pair_transition_polynomial(
                KoalaBear::ZERO,
                KoalaBear::ONE,
                KoalaBear::ZERO,
                KoalaBear::ONE,
                KoalaBear::ONE,
                KoalaBear::ZERO,
            ),
            KoalaBear::ZERO,
        );
    }
}
