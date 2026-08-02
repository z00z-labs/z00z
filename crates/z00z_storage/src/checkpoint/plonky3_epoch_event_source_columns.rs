//! Canonical columns for the packed epoch event-byte source table.

use core::any::Any;

use p3_air::{AirBuilder, BaseAir};
use p3_circuit::ops::NpoTypeId;
use p3_circuit::tables::NonPrimitiveTrace;
use p3_field::Field;
use p3_koala_bear::KoalaBear;
use p3_matrix::dense::RowMajorMatrix;

use super::{EPOCH_CHUNK_BYTES_V2, EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2};

const NPO_ID_V2: &str = "z00z/plonky3/epoch-packed-event-source/v2";
const SEMANTIC_TRANSITION_NPO_ID_V2: &str =
    "z00z/plonky3/epoch-packed-event-source-semantic-transition/v2";
const SEMANTIC_UNIQUENESS_NPO_ID_V2: &str =
    "z00z/plonky3/epoch-packed-event-source-semantic-uniqueness/v2";
pub(super) const EVENT_SOURCE_BYTE_BUS_V2: &str = "z00z/plonky3/epoch-event-source-byte/v2";

pub(super) const TRANSITION_SLOTS_V2: usize = EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 as usize;
pub(super) const BLOCK_PAIR_COUNT_V2: usize = 32;
pub(super) const STATIC_PREFIX_PAIRS_V2: usize = 50;
pub(super) const FRAMED_PREFIX_PAIRS_V2: usize = 54;
pub(super) const LENGTH_PAIRS_V2: usize = 4;
pub(super) const STATEMENT_LIMBS_V2: usize = EPOCH_CHUNK_BYTES_V2 / core::mem::size_of::<u16>();

pub(super) const PUBLIC_ACTIVE_OFFSET_V2: usize = STATEMENT_LIMBS_V2;
pub(super) const PUBLIC_BLOCK_COUNT_OFFSET_V2: usize =
    PUBLIC_ACTIVE_OFFSET_V2 + TRANSITION_SLOTS_V2;
pub(super) const PUBLIC_EVENT_BYTES_OFFSET_V2: usize =
    PUBLIC_BLOCK_COUNT_OFFSET_V2 + TRANSITION_SLOTS_V2;
pub(super) const PUBLIC_STATIC_PREFIX_OFFSET_V2: usize =
    PUBLIC_EVENT_BYTES_OFFSET_V2 + TRANSITION_SLOTS_V2;
pub(super) const PUBLIC_PART_LENGTH_OFFSET_V2: usize =
    PUBLIC_STATIC_PREFIX_OFFSET_V2 + STATIC_PREFIX_PAIRS_V2;
pub(super) const PUBLIC_BIT_LENGTH_OFFSET_V2: usize =
    PUBLIC_PART_LENGTH_OFFSET_V2 + TRANSITION_SLOTS_V2 * LENGTH_PAIRS_V2;
pub(super) const PUBLIC_SLICE_START_OFFSET_V2: usize =
    PUBLIC_BIT_LENGTH_OFFSET_V2 + TRANSITION_SLOTS_V2 * LENGTH_PAIRS_V2;
pub(super) const PUBLIC_SLICE_LEN_OFFSET_V2: usize = PUBLIC_SLICE_START_OFFSET_V2 + 1;
pub(super) const PUBLIC_FIELDS_V2: usize = PUBLIC_SLICE_LEN_OFFSET_V2 + 1;

pub(super) const ACTIVE_OFFSET_V2: usize = 0;
pub(super) const TRANSITION_SELECTOR_OFFSET_V2: usize = ACTIVE_OFFSET_V2 + 1;
pub(super) const PAIR_SELECTOR_OFFSET_V2: usize =
    TRANSITION_SELECTOR_OFFSET_V2 + TRANSITION_SLOTS_V2;
pub(super) const BLOCK_INDEX_OFFSET_V2: usize = PAIR_SELECTOR_OFFSET_V2 + BLOCK_PAIR_COUNT_V2;
pub(super) const FINAL_BLOCK_OFFSET_V2: usize = BLOCK_INDEX_OFFSET_V2 + 1;
pub(super) const PREFIX_SELECTOR_OFFSET_V2: usize = FINAL_BLOCK_OFFSET_V2 + 1;
pub(super) const RAW_BYTE_0_OFFSET_V2: usize = PREFIX_SELECTOR_OFFSET_V2 + FRAMED_PREFIX_PAIRS_V2;
pub(super) const RAW_BYTE_1_OFFSET_V2: usize = RAW_BYTE_0_OFFSET_V2 + 1;
pub(super) const RAW_FINAL_OFFSET_V2: usize = RAW_BYTE_1_OFFSET_V2 + 1;
pub(super) const PAD_BYTE_0_OFFSET_V2: usize = RAW_FINAL_OFFSET_V2 + 1;
pub(super) const PAD_BYTE_1_OFFSET_V2: usize = PAD_BYTE_0_OFFSET_V2 + 1;
pub(super) const BYTE_0_OFFSET_V2: usize = PAD_BYTE_1_OFFSET_V2 + 1;
pub(super) const BYTE_1_OFFSET_V2: usize = BYTE_0_OFFSET_V2 + 1;
pub(super) const BITS_OFFSET_V2: usize = BYTE_1_OFFSET_V2 + 1;
pub(super) const RUNNING_RAW_BYTES_OFFSET_V2: usize = BITS_OFFSET_V2 + 16;
pub(super) const JOB_KIND_SELECTOR_OFFSET_V2: usize = RUNNING_RAW_BYTES_OFFSET_V2 + 1;
pub(super) const JOB_ID_OFFSET_V2: usize =
    JOB_KIND_SELECTOR_OFFSET_V2 + super::plonky3_epoch_sha256_columns::SemanticShaJobKindV2::COUNT;
pub(super) const JOB_RAW_LEN_OFFSET_V2: usize = JOB_ID_OFFSET_V2 + 1;
pub(super) const JOB_BLOCK_COUNT_OFFSET_V2: usize = JOB_RAW_LEN_OFFSET_V2 + 1;
pub(super) const JOB_BIT_LENGTH_PAIR_OFFSET_V2: usize = JOB_BLOCK_COUNT_OFFSET_V2 + 1;
pub(super) const JOB_BIT_LENGTH_PAIR_COUNT_V2: usize = 4;
pub(super) const JOB_START_OFFSET_V2: usize =
    JOB_BIT_LENGTH_PAIR_OFFSET_V2 + JOB_BIT_LENGTH_PAIR_COUNT_V2;
pub(super) const PADDING_ZERO_BITS_OFFSET_V2: usize = JOB_START_OFFSET_V2 + 1;
pub(super) const PADDING_ZERO_BITS_V2: usize = 6;
pub(super) const ROW_FIELDS_V2: usize = PADDING_ZERO_BITS_OFFSET_V2 + PADDING_ZERO_BITS_V2;
pub(super) const CALL_FIELDS_V2: usize = PUBLIC_FIELDS_V2 + ROW_FIELDS_V2;

#[derive(Clone, Debug)]
pub(super) struct EventSourceRowV2 {
    pub(super) values: Vec<KoalaBear>,
}

#[derive(Clone, Debug)]
pub(super) struct EventSourceTraceV2 {
    pub(super) role: EventSourceAirRoleV2,
    pub(super) public_values: Vec<KoalaBear>,
    pub(super) rows: Vec<EventSourceRowV2>,
}

impl NonPrimitiveTrace<KoalaBear> for EventSourceTraceV2 {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EventSourceAirRoleV2 {
    Hash,
    SemanticTransition,
    SemanticUniqueness,
}

impl EventSourceAirRoleV2 {
    pub(super) fn npo_type(self) -> NpoTypeId {
        NpoTypeId::new(match self {
            Self::Hash => NPO_ID_V2,
            Self::SemanticTransition => SEMANTIC_TRANSITION_NPO_ID_V2,
            Self::SemanticUniqueness => SEMANTIC_UNIQUENESS_NPO_ID_V2,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct EventSourceAirV2 {
    pub(super) role: EventSourceAirRoleV2,
}

impl EventSourceAirV2 {
    pub(super) const fn new(role: EventSourceAirRoleV2) -> Self {
        Self { role }
    }

    pub(super) fn trace_to_matrix(rows: &[EventSourceRowV2]) -> RowMajorMatrix<KoalaBear> {
        RowMajorMatrix::new(
            rows.iter()
                .flat_map(|row| row.values[PUBLIC_FIELDS_V2..].iter().copied())
                .collect(),
            ROW_FIELDS_V2,
        )
    }
}

impl<F: Field> BaseAir<F> for EventSourceAirV2 {
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

pub(super) fn field<AB>(row: &[AB::Var], offset: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    row[offset].into()
}

pub(super) fn npo_type() -> NpoTypeId {
    EventSourceAirRoleV2::Hash.npo_type()
}
