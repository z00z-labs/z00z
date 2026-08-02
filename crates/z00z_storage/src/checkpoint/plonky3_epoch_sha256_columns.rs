//! Canonical column and public-input layout for epoch SHA-256 tables.
//!
//! Standalone smoke proofs and streamed chunk chains share these exact round
//! columns.  Roles may change only boundary/public-input constraints; the
//! SHA-256 round relation has one implementation.

use core::any::Any;

use p3_air::{AirBuilder, BaseAir};
use p3_circuit::ops::NpoTypeId;
use p3_circuit::tables::NonPrimitiveTrace;
use p3_field::{Field, PrimeCharacteristicRing};
use p3_koala_bear::KoalaBear;
use p3_matrix::dense::RowMajorMatrix;

use super::{EPOCH_CHUNK_BYTES_V2, EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2};

const STANDALONE_NPO_ID_V2: &str = "z00z/plonky3/epoch-sha256/v2";
const CHAIN_NPO_ID_V2: &str = "z00z/plonky3/epoch-sha256-chain/v2";
const SEMANTIC_TRANSITION_CHAIN_NPO_ID_V2: &str =
    "z00z/plonky3/epoch-sha256-semantic-transition-chain/v2";
const SEMANTIC_UNIQUENESS_CHAIN_NPO_ID_V2: &str =
    "z00z/plonky3/epoch-sha256-semantic-uniqueness-chain/v2";
const JMT_LINKED_NPO_ID_V2: &str = "z00z/plonky3/epoch-jmt-sha256-linked/v2";
const RECURSIVE_COMPRESSION_NPO_ID_V2: &str = "z00z/plonky3/recursive-sha256-compression/v2";
pub(super) const SHA_BLOCK_PAIR_BUS_V2: &str = "z00z/plonky3/epoch-sha256-block-pair/v2";
pub(super) const JMT_SHA_BLOCK_PAIR_BUS_V2: &str = "z00z/plonky3/epoch-jmt-sha256-block-pair/v2";
pub(super) const JMT_SHA_DIGEST_BUS_V2: &str = "z00z/plonky3/epoch-jmt-sha256-digest/v2";
pub(super) const SEMANTIC_SHA_BLOCK_PAIR_BUS_V2: &str =
    "z00z/plonky3/epoch-semantic-sha256-block-pair/v2";
pub(super) const SEMANTIC_SHA_DIGEST_PAIR_BUS_V2: &str =
    "z00z/plonky3/epoch-semantic-sha256-digest-pair/v2";
pub(super) const SEMANTIC_SHA_RAW_BYTE_BUS_V2: &str =
    "z00z/plonky3/epoch-semantic-sha256-raw-byte/v2";

pub(super) const SHA_ROWS_V2: usize = 64;
pub(super) const STATEMENT_LIMBS_V2: usize = EPOCH_CHUNK_BYTES_V2 / core::mem::size_of::<u16>();
pub(super) const DIGEST_LIMBS_V2: usize = 16;
pub(super) const CHAIN_TRANSITION_SLOTS_V2: usize = EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 as usize;
pub(super) const CHAIN_PADDING_SLOT_V2: usize = CHAIN_TRANSITION_SLOTS_V2;
pub(super) const CHAIN_SELECTOR_COUNT_V2: usize = CHAIN_TRANSITION_SLOTS_V2 + 1;

pub(super) const STANDALONE_INPUT_STATE_OFFSET_V2: usize = STATEMENT_LIMBS_V2;
pub(super) const STANDALONE_BLOCK_OFFSET_V2: usize = STANDALONE_INPUT_STATE_OFFSET_V2 + 16;
pub(super) const STANDALONE_OUTPUT_STATE_OFFSET_V2: usize = STANDALONE_BLOCK_OFFSET_V2 + 32;
pub(super) const STANDALONE_PUBLIC_FIELDS_V2: usize = STANDALONE_OUTPUT_STATE_OFFSET_V2 + 16;

pub(super) const CHAIN_ACTIVE_OFFSET_V2: usize = STATEMENT_LIMBS_V2;
pub(super) const CHAIN_BLOCK_COUNT_OFFSET_V2: usize =
    CHAIN_ACTIVE_OFFSET_V2 + CHAIN_TRANSITION_SLOTS_V2;
pub(super) const CHAIN_DIGEST_OFFSET_V2: usize =
    CHAIN_BLOCK_COUNT_OFFSET_V2 + CHAIN_TRANSITION_SLOTS_V2;
pub(super) const CHAIN_SLICE_START_OFFSET_V2: usize =
    CHAIN_DIGEST_OFFSET_V2 + CHAIN_TRANSITION_SLOTS_V2 * DIGEST_LIMBS_V2;
pub(super) const CHAIN_SLICE_LEN_OFFSET_V2: usize = CHAIN_SLICE_START_OFFSET_V2 + 1;
pub(super) const CHAIN_PUBLIC_FIELDS_V2: usize = CHAIN_SLICE_LEN_OFFSET_V2 + 1;
pub(super) const JMT_LINKED_PUBLIC_FIELDS_V2: usize = STATEMENT_LIMBS_V2;

pub(super) const TRANSITION_SELECTOR_OFFSET_V2: usize = 0;
pub(super) const BLOCK_INDEX_OFFSET_V2: usize =
    TRANSITION_SELECTOR_OFFSET_V2 + CHAIN_SELECTOR_COUNT_V2;
pub(super) const TRANSITION_FINAL_OFFSET_V2: usize = BLOCK_INDEX_OFFSET_V2 + 1;
pub(super) const BASE_STATE_OFFSET_V2: usize = TRANSITION_FINAL_OFFSET_V2 + 1;
pub(super) const STATE_LIMBS_OFFSET_V2: usize = BASE_STATE_OFFSET_V2 + 16;
const STATE_LIMBS_V2: usize = 16;
const STATE_BITS_OFFSET_V2: usize = STATE_LIMBS_OFFSET_V2 + STATE_LIMBS_V2;
pub(super) const STATE_BIT_WORDS_V2: [usize; 6] = [0, 1, 2, 4, 5, 6];
const STATE_BITS_V2: usize = STATE_BIT_WORDS_V2.len() * 32;
pub(super) const SCHEDULE_LIMBS_OFFSET_V2: usize = STATE_BITS_OFFSET_V2 + STATE_BITS_V2;
const SCHEDULE_LIMBS_V2: usize = 32;
const SCHEDULE_BITS_OFFSET_V2: usize = SCHEDULE_LIMBS_OFFSET_V2 + SCHEDULE_LIMBS_V2;
const SCHEDULE_BITS_V2: usize = 64;
pub(super) const SELECTOR_OFFSET_V2: usize = SCHEDULE_BITS_OFFSET_V2 + SCHEDULE_BITS_V2;
const SELECTOR_FIELDS_V2: usize = SHA_ROWS_V2;
pub(super) const T1_BITS_OFFSET_V2: usize = SELECTOR_OFFSET_V2 + SELECTOR_FIELDS_V2;
pub(super) const T2_BITS_OFFSET_V2: usize = T1_BITS_OFFSET_V2 + 32;
pub(super) const SCHEDULE_CARRY_OFFSET_V2: usize = T2_BITS_OFFSET_V2 + 32;
pub(super) const T1_CARRY_OFFSET_V2: usize = SCHEDULE_CARRY_OFFSET_V2 + 4;
pub(super) const T2_CARRY_OFFSET_V2: usize = T1_CARRY_OFFSET_V2 + 6;
pub(super) const E_CARRY_OFFSET_V2: usize = T2_CARRY_OFFSET_V2 + 2;
pub(super) const A_CARRY_OFFSET_V2: usize = E_CARRY_OFFSET_V2 + 2;
pub(super) const OUTPUT_CARRY_OFFSET_V2: usize = A_CARRY_OFFSET_V2 + 2;
pub(super) const SHA_COMMON_ROW_FIELDS_V2: usize = OUTPUT_CARRY_OFFSET_V2 + 16;
pub(super) const JMT_LANE_OFFSET_V2: usize = SHA_COMMON_ROW_FIELDS_V2;
pub(super) const JMT_RECORD_OFFSET_V2: usize = JMT_LANE_OFFSET_V2 + 1;
pub(super) const JMT_ROLE_OFFSET_V2: usize = JMT_RECORD_OFFSET_V2 + 1;
pub(super) const JMT_BLOCK_COUNT_OFFSET_V2: usize = JMT_ROLE_OFFSET_V2 + 1;
pub(super) const ROW_FIELDS_V2: usize = JMT_BLOCK_COUNT_OFFSET_V2 + 1;
pub(super) const PREPROCESSED_WIDTH_V2: usize = 1;
pub(super) const RECURSIVE_SHA_INPUT_LIMBS_V2: usize = 16 + 32;
pub(super) const RECURSIVE_SHA_OUTPUT_LIMBS_V2: usize = 16;
pub(super) const RECURSIVE_SHA_PREPROCESSED_WIDTH_V2: usize = 1 + RECURSIVE_SHA_INPUT_LIMBS_V2;
pub(super) const RADIX_V2: u64 = 65_536;

pub(super) const JMT_SHA_ROLE_OLD_LEAF_V2: u8 = 1;
pub(super) const JMT_SHA_ROLE_NEW_LEAF_V2: u8 = 2;
pub(super) const JMT_SHA_ROLE_SIBLING_V2: u8 = 3;
pub(super) const JMT_SHA_ROLE_OLD_PARENT_V2: u8 = 4;
pub(super) const JMT_SHA_ROLE_NEW_PARENT_V2: u8 = 5;
pub(super) const JMT_SHA_ROLE_PRIOR_VALUE_V2: u8 = 6;
pub(super) const JMT_SHA_ROLE_NEW_VALUE_V2: u8 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum SemanticShaJobKindV2 {
    EventVector = 1,
    StructuralEventId = 2,
    UniquenessList = 3,
    UniquenessPrecommit = 4,
    UniquenessSetPrecommit = 5,
    UniquenessChallenge = 6,
}

impl SemanticShaJobKindV2 {
    pub(super) const COUNT: usize = 6;

    pub(super) const ALL: [Self; Self::COUNT] = [
        Self::EventVector,
        Self::StructuralEventId,
        Self::UniquenessList,
        Self::UniquenessPrecommit,
        Self::UniquenessSetPrecommit,
        Self::UniquenessChallenge,
    ];

    pub(super) const fn index(self) -> usize {
        self as usize - 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShaAirRoleV2 {
    Standalone,
    Chain,
    SemanticTransitionChain,
    SemanticUniquenessChain,
    JmtLinked,
    RecursiveCompression,
}

impl ShaAirRoleV2 {
    pub(super) fn npo_type(self) -> NpoTypeId {
        NpoTypeId::new(match self {
            Self::Standalone => STANDALONE_NPO_ID_V2,
            Self::Chain => CHAIN_NPO_ID_V2,
            Self::SemanticTransitionChain => SEMANTIC_TRANSITION_CHAIN_NPO_ID_V2,
            Self::SemanticUniquenessChain => SEMANTIC_UNIQUENESS_CHAIN_NPO_ID_V2,
            Self::JmtLinked => JMT_LINKED_NPO_ID_V2,
            Self::RecursiveCompression => RECURSIVE_COMPRESSION_NPO_ID_V2,
        })
    }

    pub(super) const fn public_fields(self) -> usize {
        match self {
            Self::Standalone => STANDALONE_PUBLIC_FIELDS_V2,
            Self::Chain | Self::SemanticTransitionChain | Self::SemanticUniquenessChain => {
                CHAIN_PUBLIC_FIELDS_V2
            }
            Self::JmtLinked => JMT_LINKED_PUBLIC_FIELDS_V2,
            Self::RecursiveCompression => 0,
        }
    }

    pub(super) const fn preprocessed_fields(self) -> usize {
        match self {
            Self::RecursiveCompression => RECURSIVE_SHA_PREPROCESSED_WIDTH_V2,
            _ => PREPROCESSED_WIDTH_V2,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct ShaRowV2 {
    pub(super) values: Vec<KoalaBear>,
}

#[derive(Clone, Debug)]
pub(super) struct ShaTraceV2 {
    pub(super) role: ShaAirRoleV2,
    pub(super) public_values: Vec<KoalaBear>,
    pub(super) rows: Vec<ShaRowV2>,
}

impl NonPrimitiveTrace<KoalaBear> for ShaTraceV2 {
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

#[derive(Clone, Debug)]
pub(super) struct ShaAirV2<F, const D: usize> {
    pub(super) role: ShaAirRoleV2,
    preprocessed: Vec<F>,
    min_height: usize,
}

impl<F: Field, const D: usize> ShaAirV2<F, D> {
    const fn width_v2() -> usize {
        ROW_FIELDS_V2 * D
    }

    pub(super) fn new(role: ShaAirRoleV2, preprocessed: Vec<F>, min_height: usize) -> Self {
        Self {
            role,
            preprocessed,
            min_height: min_height.max(SHA_ROWS_V2),
        }
    }
}

impl<const D: usize> ShaAirV2<KoalaBear, D> {
    pub(super) fn trace_to_matrix(
        rows: &[ShaRowV2],
        min_height: usize,
    ) -> RowMajorMatrix<KoalaBear> {
        let mut values = KoalaBear::zero_vec(rows.len() * Self::width_v2());
        for (row_index, row) in rows.iter().enumerate() {
            for (field_index, value) in row.values.iter().copied().enumerate() {
                values[row_index * Self::width_v2() + field_index * D] = value;
            }
        }
        let mut matrix = RowMajorMatrix::new(values, Self::width_v2());
        matrix.pad_to_min_power_of_two_height(min_height.max(SHA_ROWS_V2), KoalaBear::ZERO);
        matrix
    }
}

impl<F: Field, const D: usize> BaseAir<F> for ShaAirV2<F, D> {
    fn width(&self) -> usize {
        Self::width_v2()
    }

    fn num_public_values(&self) -> usize {
        self.role.public_fields()
    }

    fn preprocessed_width(&self) -> usize {
        self.role.preprocessed_fields()
    }

    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
        let mut matrix = RowMajorMatrix::from_flat_padded(
            self.preprocessed.clone(),
            self.role.preprocessed_fields(),
            F::ZERO,
        );
        matrix.pad_to_min_power_of_two_height(self.min_height, F::ZERO);
        Some(matrix)
    }

    fn main_next_row_columns(&self) -> Vec<usize> {
        (0..Self::width_v2()).collect()
    }

    fn preprocessed_next_row_columns(&self) -> Vec<usize> {
        Vec::new()
    }
}

pub(super) fn field<AB, const D: usize>(row: &[AB::Var], offset: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    row[offset * D].into()
}

pub(super) fn base_state_limb<AB, const D: usize>(
    row: &[AB::Var],
    word: usize,
    limb: usize,
) -> AB::Expr
where
    AB: AirBuilder,
{
    field::<AB, D>(row, BASE_STATE_OFFSET_V2 + word * 2 + limb)
}

pub(super) fn state_limb<AB, const D: usize>(row: &[AB::Var], word: usize, limb: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    field::<AB, D>(row, STATE_LIMBS_OFFSET_V2 + word * 2 + limb)
}

pub(super) fn state_bit<AB, const D: usize>(row: &[AB::Var], slot: usize, bit: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    field::<AB, D>(row, STATE_BITS_OFFSET_V2 + slot * 32 + bit)
}

pub(super) fn schedule_limb<AB, const D: usize>(
    row: &[AB::Var],
    word: usize,
    limb: usize,
) -> AB::Expr
where
    AB: AirBuilder,
{
    field::<AB, D>(row, SCHEDULE_LIMBS_OFFSET_V2 + word * 2 + limb)
}

pub(super) fn schedule_bit<AB, const D: usize>(row: &[AB::Var], slot: usize, bit: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    field::<AB, D>(row, SCHEDULE_BITS_OFFSET_V2 + slot * 32 + bit)
}

pub(super) fn bits_limb<AB, const D: usize>(row: &[AB::Var], offset: usize, limb: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    let mut result = AB::Expr::ZERO;
    for bit in 0..16 {
        result += field::<AB, D>(row, offset + limb * 16 + bit) * AB::Expr::from_u64(1_u64 << bit);
    }
    result
}

pub(super) fn carry<AB, const D: usize>(row: &[AB::Var], offset: usize, bits: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    let mut result = AB::Expr::ZERO;
    for bit in 0..bits {
        result += field::<AB, D>(row, offset + bit) * AB::Expr::from_u64(1_u64 << bit);
    }
    result
}
