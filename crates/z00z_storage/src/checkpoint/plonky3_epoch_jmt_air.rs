//! Direct JMT path/update AIR over the canonical settlement micro-op stream.
//!
//! Every active row contains one exact, zero-padded circuit record together
//! with the post-operation path-machine state. The AIR proves transcript
//! grammar, update/operation ordering, path directions, sibling counts, and
//! old/new parent-child chaining. Raw SHA requests remain explicit in these
//! rows for the proof-bound cross-table join; this table alone is never
//! frontier-admissible.

use core::any::Any;

use p3_air::{Air, AirBuilder, BaseAir, WindowAccess};
use p3_circuit::ops::NpoTypeId;
use p3_circuit::tables::NonPrimitiveTrace;
use p3_field::{Field, PrimeCharacteristicRing};
use p3_koala_bear::KoalaBear;
use p3_lookup::{Count, InteractionBuilder};
use p3_matrix::dense::RowMajorMatrix;
use z00z_plonky3_circuit_prover::batch_stark_prover::BatchAir;

use super::plonky3_epoch_jmt_constraints_local::eval_local_path;
use super::plonky3_epoch_jmt_constraints_row::eval_row_shape;
use super::plonky3_epoch_jmt_constraints_transition::eval_transition_state;
use super::plonky3_epoch_semantic_source_air::{
    SOURCE_JMT_PAYLOAD_BYTE_BUS_V2, SOURCE_NET_MUTATION_BYTE_BUS_V2,
};
use super::plonky3_epoch_sha256_columns::{
    JMT_SHA_BLOCK_PAIR_BUS_V2, JMT_SHA_DIGEST_BUS_V2, JMT_SHA_ROLE_NEW_LEAF_V2,
    JMT_SHA_ROLE_NEW_PARENT_V2, JMT_SHA_ROLE_NEW_VALUE_V2, JMT_SHA_ROLE_OLD_LEAF_V2,
    JMT_SHA_ROLE_OLD_PARENT_V2, JMT_SHA_ROLE_PRIOR_VALUE_V2, JMT_SHA_ROLE_SIBLING_V2,
};
use super::{
    Plonky3StarkConfigV2, RecursiveTraceOpcodeV2, EPOCH_CHUNK_BYTES_V2,
    EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2, UNIQUENESS_PRECOMMIT_VERSION_V2,
};
use crate::settlement::{
    noop_update_trace_digest, RootGeneration, JMT_CIRCUIT_HEADER_BYTES_V2, JMT_TRACE_NOOP_KIND_V2,
    JMT_UPDATE_TRACE_VERSION_V2, JMT_VALUE_MAX_BYTES_V2,
};

pub(super) const JMT_NPO_ID_V2: &str = "z00z/plonky3/epoch-jmt-update/v2";
pub(super) const JMT_CHUNK_NPO_ID_V2: &str = "z00z/plonky3/epoch-jmt-update-chunk/v2";
const _: () = assert!(JMT_VALUE_MAX_BYTES_V2 == 65_536);
pub(super) const JMT_MIN_ROWS_V2: usize = 8;
pub(super) const JMT_RECORD_BYTES_V2: usize = 403;
pub(super) const JMT_HEADER_FIELDS_V2: usize = JMT_CIRCUIT_HEADER_BYTES_V2;
pub(super) const STATEMENT_LIMBS_V2: usize = EPOCH_CHUNK_BYTES_V2 / core::mem::size_of::<u16>();
pub(super) const DIGEST_LIMBS_V2: usize = 32 / core::mem::size_of::<u16>();
pub(super) const PUBLIC_ROW_COUNT_OFFSET_V2: usize = 21;
pub(super) const PUBLIC_HEADER_OFFSET_V2: usize = STATEMENT_LIMBS_V2;
pub(super) const PUBLIC_INPUT_ROOT_OFFSET_V2: usize =
    PUBLIC_HEADER_OFFSET_V2 + JMT_HEADER_FIELDS_V2;
pub(super) const PUBLIC_OUTPUT_ROOT_OFFSET_V2: usize =
    PUBLIC_INPUT_ROOT_OFFSET_V2 + DIGEST_LIMBS_V2;
pub(super) const PUBLIC_FIELDS_V2: usize = PUBLIC_OUTPUT_ROOT_OFFSET_V2 + DIGEST_LIMBS_V2;

pub(super) const ACTIVE_OFFSET_V2: usize = 0;
pub(super) const OPCODE_OFFSET_V2: usize = 1;
pub(super) const OPCODE_COUNT_V2: usize = 9;
pub(super) const RUNNING_OFFSET_V2: usize = OPCODE_OFFSET_V2 + OPCODE_COUNT_V2;
pub(super) const RECORD_LEN_OFFSET_V2: usize = RUNNING_OFFSET_V2 + 1;
pub(super) const UPDATE_INDEX_OFFSET_V2: usize = RECORD_LEN_OFFSET_V2 + 1;
pub(super) const OPERATION_INDEX_OFFSET_V2: usize = UPDATE_INDEX_OFFSET_V2 + 2;
pub(super) const AUX_INDEX_OFFSET_V2: usize = OPERATION_INDEX_OFFSET_V2 + 2;
pub(super) const VALUE_PRESENT_OFFSET_V2: usize = AUX_INDEX_OFFSET_V2 + 1;
pub(super) const PRIOR_PRESENT_OFFSET_V2: usize = VALUE_PRESENT_OFFSET_V2 + 1;
pub(super) const LEAF_PRESENT_OFFSET_V2: usize = PRIOR_PRESENT_OFFSET_V2 + 1;
pub(super) const NEW_PARENT_ACTIVE_OFFSET_V2: usize = LEAF_PRESENT_OFFSET_V2 + 1;
pub(super) const DIRECTION_OFFSET_V2: usize = NEW_PARENT_ACTIVE_OFFSET_V2 + 1;
pub(super) const EXPECTED_SIBLINGS_OFFSET_V2: usize = DIRECTION_OFFSET_V2 + 1;
pub(super) const CONSUMED_SIBLINGS_OFFSET_V2: usize = EXPECTED_SIBLINGS_OFFSET_V2 + 1;
pub(super) const EXPECTED_SPLIT_OFFSET_V2: usize = CONSUMED_SIBLINGS_OFFSET_V2 + 1;
pub(super) const CONSUMED_SPLIT_OFFSET_V2: usize = EXPECTED_SPLIT_OFFSET_V2 + 1;
pub(super) const EXPECTED_OPERATIONS_OFFSET_V2: usize = CONSUMED_SPLIT_OFFSET_V2 + 1;
pub(super) const COMPLETED_OPERATIONS_OFFSET_V2: usize = EXPECTED_OPERATIONS_OFFSET_V2 + 2;
pub(super) const COALESCED_OFFSET_V2: usize = COMPLETED_OPERATIONS_OFFSET_V2 + 2;
pub(super) const NEW_PARENT_STARTED_OFFSET_V2: usize = COALESCED_OFFSET_V2 + 1;
pub(super) const ROLE_OFFSET_V2: usize = NEW_PARENT_STARTED_OFFSET_V2 + 1;
pub(super) const ROLE_COUNT_V2: usize = 5;
pub(super) const CASE_OFFSET_V2: usize = ROLE_OFFSET_V2 + ROLE_COUNT_V2;
pub(super) const CASE_COUNT_V2: usize = 6;
pub(super) const SIBLING_TYPE_OFFSET_V2: usize = CASE_OFFSET_V2 + CASE_COUNT_V2;
pub(super) const SIBLING_TYPE_COUNT_V2: usize = 3;
pub(super) const RECORD_OFFSET_V2: usize = SIBLING_TYPE_OFFSET_V2 + SIBLING_TYPE_COUNT_V2;
pub(super) const KEY_OFFSET_V2: usize = RECORD_OFFSET_V2 + JMT_RECORD_BYTES_V2;
pub(super) const PATH_KEY_OFFSET_V2: usize = KEY_OFFSET_V2 + 32;
pub(super) const UPDATE_CURRENT_OFFSET_V2: usize = PATH_KEY_OFFSET_V2 + 32;
pub(super) const UPDATE_NEW_ROOT_OFFSET_V2: usize = UPDATE_CURRENT_OFFSET_V2 + 16;
pub(super) const OLD_CURRENT_OFFSET_V2: usize = UPDATE_NEW_ROOT_OFFSET_V2 + 16;
pub(super) const NEW_CURRENT_OFFSET_V2: usize = OLD_CURRENT_OFFSET_V2 + 16;
pub(super) const SIBLING_DIGEST_OFFSET_V2: usize = NEW_CURRENT_OFFSET_V2 + 16;
pub(super) const OLD_PARENT_DIGEST_OFFSET_V2: usize = SIBLING_DIGEST_OFFSET_V2 + 16;
pub(super) const NEW_PARENT_DIGEST_OFFSET_V2: usize = OLD_PARENT_DIGEST_OFFSET_V2 + 16;
pub(super) const OLD_LEAF_DIGEST_OFFSET_V2: usize = NEW_PARENT_DIGEST_OFFSET_V2 + 16;
pub(super) const NEW_LEAF_DIGEST_OFFSET_V2: usize = OLD_LEAF_DIGEST_OFFSET_V2 + 16;
pub(super) const BYTE_POSITION_OFFSET_V2: usize = NEW_LEAF_DIGEST_OFFSET_V2 + 16;
pub(super) const BYTE_POSITION_COUNT_V2: usize = 32;
pub(super) const BIT_POSITION_OFFSET_V2: usize = BYTE_POSITION_OFFSET_V2 + BYTE_POSITION_COUNT_V2;
pub(super) const BIT_POSITION_COUNT_V2: usize = 8;
pub(super) const SELECTED_BITS_OFFSET_V2: usize = BIT_POSITION_OFFSET_V2 + BIT_POSITION_COUNT_V2;
pub(super) const SELECTED_BITS_COUNT_V2: usize = 8;
pub(super) const OPERATION_JOB_OFFSET_V2: usize = SELECTED_BITS_OFFSET_V2 + SELECTED_BITS_COUNT_V2;
pub(super) const EXPECTED_VALUE_BYTES_OFFSET_V2: usize = OPERATION_JOB_OFFSET_V2 + 1;
pub(super) const EXPECTED_PRIOR_VALUE_BYTES_OFFSET_V2: usize = EXPECTED_VALUE_BYTES_OFFSET_V2 + 1;
pub(super) const PRIOR_VALUE_BLOCK_COUNT_OFFSET_V2: usize =
    EXPECTED_PRIOR_VALUE_BYTES_OFFSET_V2 + 1;
pub(super) const NEW_VALUE_BLOCK_COUNT_OFFSET_V2: usize = PRIOR_VALUE_BLOCK_COUNT_OFFSET_V2 + 1;
pub(super) const VALUE_PADDING_STARTED_OFFSET_V2: usize = NEW_VALUE_BLOCK_COUNT_OFFSET_V2 + 1;
pub(super) const VALUE_PADDING_START_OFFSET_V2: usize = VALUE_PADDING_STARTED_OFFSET_V2 + 1;
pub(super) const VALUE_REMAINDER_OFFSET_V2: usize = VALUE_PADDING_START_OFFSET_V2 + 1;
pub(super) const VALUE_REMAINDER_COUNT_V2: usize = 64;
pub(super) const TREE_DEFINITION_OFFSET_V2: usize =
    VALUE_REMAINDER_OFFSET_V2 + VALUE_REMAINDER_COUNT_V2;
pub(super) const TREE_DEFINITION_BYTES_V2: usize = 32;
pub(super) const TREE_SERIAL_OFFSET_V2: usize =
    TREE_DEFINITION_OFFSET_V2 + TREE_DEFINITION_BYTES_V2;
pub(super) const TREE_SERIAL_BYTES_V2: usize = 4;
pub(super) const ROW_FIELDS_V2: usize = TREE_SERIAL_OFFSET_V2 + TREE_SERIAL_BYTES_V2;
pub(super) const CALL_FIELDS_V2: usize = PUBLIC_FIELDS_V2 + ROW_FIELDS_V2;
pub(super) const PREPROCESSED_WIDTH_V2: usize = 1;
pub(super) const CHUNK_LANES_V2: usize = EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 as usize;
pub(super) const CHUNK_ACTIVE_OFFSET_V2: usize = STATEMENT_LIMBS_V2;
pub(super) const CHUNK_LANE_OFFSET_V2: usize = CHUNK_ACTIVE_OFFSET_V2 + CHUNK_LANES_V2;
pub(super) const CHUNK_LANE_PRE_ROOT_OFFSET_V2: usize = 0;
pub(super) const CHUNK_LANE_POST_ROOT_OFFSET_V2: usize =
    CHUNK_LANE_PRE_ROOT_OFFSET_V2 + DIGEST_LIMBS_V2;
pub(super) const CHUNK_LANE_POST_ROOT_BYTE_OFFSET_V2: usize =
    CHUNK_LANE_POST_ROOT_OFFSET_V2 + DIGEST_LIMBS_V2;
pub(super) const CHUNK_LANE_TRACE_DIGEST_OFFSET_V2: usize =
    CHUNK_LANE_POST_ROOT_BYTE_OFFSET_V2 + 32;
pub(super) const CHUNK_LANE_RECORD_COUNT_OFFSET_V2: usize = CHUNK_LANE_TRACE_DIGEST_OFFSET_V2 + 32;
pub(super) const CHUNK_LANE_UPDATE_COUNT_OFFSET_V2: usize = CHUNK_LANE_RECORD_COUNT_OFFSET_V2 + 4;
pub(super) const CHUNK_LANE_KIND_OFFSET_V2: usize = CHUNK_LANE_UPDATE_COUNT_OFFSET_V2 + 4;
pub(super) const CHUNK_LANE_FIELDS_V2: usize = CHUNK_LANE_KIND_OFFSET_V2 + 1;
pub(super) const CHUNK_PUBLIC_FIELDS_V2: usize =
    CHUNK_LANE_OFFSET_V2 + CHUNK_LANES_V2 * CHUNK_LANE_FIELDS_V2;
pub(super) const CHUNK_ROW_FIELDS_V2: usize = CHUNK_LANES_V2 * ROW_FIELDS_V2;

pub(super) const RECORD_LENGTHS_V2: [usize; OPCODE_COUNT_V2] =
    [159, 52, 83, 275, 10, 6, 403, 10, 275];

pub(super) fn first_row_preprocessed<F: Field>(rows: usize) -> Vec<F> {
    let mut values = F::zero_vec(rows.max(1));
    values[0] = F::ONE;
    values
}

#[derive(Clone, Debug)]
pub(super) struct JmtRowV2 {
    pub(super) values: Vec<KoalaBear>,
}

#[derive(Clone, Debug)]
pub(super) struct JmtTraceV2 {
    pub(super) rows: Vec<JmtRowV2>,
}

impl NonPrimitiveTrace<KoalaBear> for JmtTraceV2 {
    fn op_type(&self) -> NpoTypeId {
        jmt_npo_type()
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
pub(super) struct JmtChunkRowV2 {
    pub(super) values: Vec<KoalaBear>,
}

#[derive(Clone, Debug)]
pub(super) struct JmtChunkTraceV2 {
    pub(super) public_values: Vec<KoalaBear>,
    pub(super) rows: Vec<JmtChunkRowV2>,
}

impl NonPrimitiveTrace<KoalaBear> for JmtChunkTraceV2 {
    fn op_type(&self) -> NpoTypeId {
        jmt_chunk_npo_type()
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
pub(super) struct JmtAirV2<F, const D: usize> {
    preprocessed: Vec<F>,
    min_height: usize,
}

impl<F: Field, const D: usize> JmtAirV2<F, D> {
    const fn width_v2() -> usize {
        ROW_FIELDS_V2 * D
    }

    pub(super) fn new(preprocessed: Vec<F>, min_height: usize) -> Self {
        Self {
            preprocessed,
            min_height: min_height.max(JMT_MIN_ROWS_V2),
        }
    }
}

impl<const D: usize> JmtAirV2<KoalaBear, D> {
    pub(super) fn trace_to_matrix(
        rows: &[JmtRowV2],
        min_height: usize,
    ) -> RowMajorMatrix<KoalaBear> {
        let mut values = KoalaBear::zero_vec(rows.len() * Self::width_v2());
        for (row_index, row) in rows.iter().enumerate() {
            for (field_index, value) in row.values[PUBLIC_FIELDS_V2..].iter().copied().enumerate() {
                values[row_index * Self::width_v2() + field_index * D] = value;
            }
        }
        let mut matrix = RowMajorMatrix::new(values, Self::width_v2());
        matrix.pad_to_min_power_of_two_height(min_height.max(rows.len()), KoalaBear::ZERO);
        matrix
    }
}

impl<F: Field, const D: usize> BaseAir<F> for JmtAirV2<F, D> {
    fn width(&self) -> usize {
        Self::width_v2()
    }

    fn num_public_values(&self) -> usize {
        PUBLIC_FIELDS_V2
    }

    fn preprocessed_width(&self) -> usize {
        PREPROCESSED_WIDTH_V2
    }

    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
        let mut matrix = RowMajorMatrix::from_flat_padded(self.preprocessed.clone(), 1, F::ZERO);
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

pub(super) fn sum_fields<AB, const D: usize>(
    row: &[AB::Var],
    offset: usize,
    count: usize,
) -> AB::Expr
where
    AB: AirBuilder,
{
    (0..count)
        .map(|index| field::<AB, D>(row, offset + index))
        .fold(AB::Expr::ZERO, |sum, value| sum + value)
}

pub(super) fn record_byte<AB, const D: usize>(row: &[AB::Var], index: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    field::<AB, D>(row, RECORD_OFFSET_V2 + index)
}

pub(super) fn record_limb<AB, const D: usize>(row: &[AB::Var], index: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    record_byte::<AB, D>(row, index)
        + record_byte::<AB, D>(row, index + 1) * AB::Expr::from_u64(256)
}

/// Return one SHA-native word limb from a 32-byte digest embedded in a record.
///
/// SHA columns store each big-endian u32 as `(low_u16, high_u16)`. Record bytes
/// remain canonical byte order, so the two bytes of each selected limb are
/// combined big-endian here.
pub(super) fn record_digest_limb<AB, const D: usize>(
    row: &[AB::Var],
    start: usize,
    limb: usize,
) -> AB::Expr
where
    AB: AirBuilder,
{
    let word = limb / 2;
    let within_word = if limb.is_multiple_of(2) { 2 } else { 0 };
    let index = start + word * 4 + within_word;
    record_byte::<AB, D>(row, index) * AB::Expr::from_u64(256)
        + record_byte::<AB, D>(row, index + 1)
}

/// Return one SHA-native word limb from a canonical 32-byte digest constant.
pub(super) fn digest_constant_limb<AB>(digest: &[u8; 32], limb: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    let word = limb / 2;
    let within_word = if limb.is_multiple_of(2) { 2 } else { 0 };
    let byte = word * 4 + within_word;
    AB::Expr::from_u64(u64::from(u16::from_be_bytes([
        digest[byte],
        digest[byte + 1],
    ])))
}

pub(super) fn digest_limb<AB, const D: usize>(
    row: &[AB::Var],
    offset: usize,
    limb: usize,
) -> AB::Expr
where
    AB: AirBuilder,
{
    field::<AB, D>(row, offset + limb)
}

impl<AB, const D: usize> Air<AB> for JmtAirV2<AB::F, D>
where
    AB: AirBuilder,
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
        eval_row_shape::<AB, D>(builder, local, next, &public);
        eval_local_path::<AB, D>(builder, local, &public);
        eval_transition_state::<AB, D>(builder, local, next, &public);
    }
}
impl BatchAir<Plonky3StarkConfigV2> for JmtAirV2<KoalaBear, 1> {}

#[derive(Clone, Debug)]
pub(super) struct JmtChunkAirV2<F, const D: usize> {
    preprocessed: Vec<F>,
    min_height: usize,
}

impl<F: Field, const D: usize> JmtChunkAirV2<F, D> {
    pub(super) fn new(preprocessed: Vec<F>, min_height: usize) -> Self {
        Self {
            preprocessed,
            min_height: min_height.max(JMT_MIN_ROWS_V2),
        }
    }
}

impl<const D: usize> JmtChunkAirV2<KoalaBear, D> {
    pub(super) fn trace_to_matrix(
        rows: &[JmtChunkRowV2],
        min_height: usize,
    ) -> RowMajorMatrix<KoalaBear> {
        let mut values = KoalaBear::zero_vec(rows.len() * CHUNK_ROW_FIELDS_V2 * D);
        for (row_index, row) in rows.iter().enumerate() {
            for (field_index, value) in row.values.iter().copied().enumerate() {
                values[row_index * CHUNK_ROW_FIELDS_V2 * D + field_index * D] = value;
            }
        }
        let mut matrix = RowMajorMatrix::new(values, CHUNK_ROW_FIELDS_V2 * D);
        matrix.pad_to_min_power_of_two_height(min_height.max(rows.len()), KoalaBear::ZERO);
        matrix
    }
}

impl<F: Field, const D: usize> BaseAir<F> for JmtChunkAirV2<F, D> {
    fn width(&self) -> usize {
        CHUNK_ROW_FIELDS_V2 * D
    }

    fn num_public_values(&self) -> usize {
        CHUNK_PUBLIC_FIELDS_V2
    }

    fn preprocessed_width(&self) -> usize {
        PREPROCESSED_WIDTH_V2
    }

    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
        let mut matrix = RowMajorMatrix::from_flat_padded(
            self.preprocessed.clone(),
            PREPROCESSED_WIDTH_V2,
            F::ZERO,
        );
        matrix.pad_to_min_power_of_two_height(self.min_height, F::ZERO);
        Some(matrix)
    }

    fn main_next_row_columns(&self) -> Vec<usize> {
        (0..self.width()).collect()
    }

    fn preprocessed_next_row_columns(&self) -> Vec<usize> {
        vec![0]
    }
}

fn chunk_lane_public<AB>(public: &[AB::Expr], lane: usize) -> Vec<AB::Expr>
where
    AB: AirBuilder,
{
    let lane_offset = CHUNK_LANE_OFFSET_V2 + lane * CHUNK_LANE_FIELDS_V2;
    let mut values = public[..STATEMENT_LIMBS_V2].to_vec();
    values[PUBLIC_ROW_COUNT_OFFSET_V2..PUBLIC_ROW_COUNT_OFFSET_V2 + 4].clone_from_slice(
        &public[lane_offset + CHUNK_LANE_RECORD_COUNT_OFFSET_V2
            ..lane_offset + CHUNK_LANE_RECORD_COUNT_OFFSET_V2 + 4],
    );
    values.push(AB::Expr::from_u64(u64::from(JMT_UPDATE_TRACE_VERSION_V2)));
    values.push(AB::Expr::from_u64(u64::from(
        RootGeneration::SettlementV2.version(),
    )));
    values.push(public[lane_offset + CHUNK_LANE_KIND_OFFSET_V2].clone());
    values.extend_from_slice(
        &public[lane_offset + CHUNK_LANE_TRACE_DIGEST_OFFSET_V2
            ..lane_offset + CHUNK_LANE_TRACE_DIGEST_OFFSET_V2 + 32],
    );
    values.extend_from_slice(
        &public[lane_offset + CHUNK_LANE_UPDATE_COUNT_OFFSET_V2
            ..lane_offset + CHUNK_LANE_UPDATE_COUNT_OFFSET_V2 + 4],
    );
    values.extend_from_slice(
        &public[lane_offset + CHUNK_LANE_PRE_ROOT_OFFSET_V2
            ..lane_offset + CHUNK_LANE_PRE_ROOT_OFFSET_V2 + DIGEST_LIMBS_V2],
    );
    values.extend_from_slice(
        &public[lane_offset + CHUNK_LANE_POST_ROOT_OFFSET_V2
            ..lane_offset + CHUNK_LANE_POST_ROOT_OFFSET_V2 + DIGEST_LIMBS_V2],
    );
    values
}

fn eval_jmt_sha_interactions<AB, const D: usize>(builder: &mut AB, row: &[AB::Var], lane: usize)
where
    AB: AirBuilder + InteractionBuilder,
{
    let op = |opcode: usize| field::<AB, D>(row, OPCODE_OFFSET_V2 + opcode - 1);
    let value_row = op(3);
    let proof_row = op(4);
    let sibling_row = op(7);
    let split_row = op(9);
    let leaf_present = field::<AB, D>(row, LEAF_PRESENT_OFFSET_V2);
    let value_present = field::<AB, D>(row, VALUE_PRESENT_OFFSET_V2);
    let new_parent_active = field::<AB, D>(row, NEW_PARENT_ACTIVE_OFFSET_V2);
    let non_placeholder_sibling = field::<AB, D>(row, SIBLING_TYPE_OFFSET_V2 + 1)
        + field::<AB, D>(row, SIBLING_TYPE_OFFSET_V2 + 2);
    let record = field::<AB, D>(row, RUNNING_OFFSET_V2);
    let lane = AB::Expr::from_usize(lane);

    let value_kind = record_byte::<AB, D>(row, 18);
    let value_role = value_kind.clone()
        * AB::Expr::from_u64(u64::from(JMT_SHA_ROLE_PRIOR_VALUE_V2))
        + (AB::Expr::ONE - value_kind) * AB::Expr::from_u64(u64::from(JMT_SHA_ROLE_NEW_VALUE_V2));
    let value_job = field::<AB, D>(row, OPERATION_JOB_OFFSET_V2);
    let value_block_index = record_limb::<AB, D>(row, 10);
    let value_block_count = record_limb::<AB, D>(row, 14);
    for pair in 0..32 {
        let byte = 19 + pair * 2;
        let pair_value = record_byte::<AB, D>(row, byte) * AB::Expr::from_u64(256)
            + record_byte::<AB, D>(row, byte + 1);
        builder.push_interaction(
            JMT_SHA_BLOCK_PAIR_BUS_V2,
            vec![
                lane.clone(),
                value_job.clone(),
                value_role.clone(),
                value_block_count.clone(),
                value_block_index.clone(),
                AB::Expr::from_usize(pair),
                pair_value,
            ],
            -Count::bounded(value_row.clone(), 1),
        );
    }

    let jobs = [
        (
            proof_row.clone() * leaf_present,
            JMT_SHA_ROLE_OLD_LEAF_V2,
            19,
            OLD_LEAF_DIGEST_OFFSET_V2,
        ),
        (
            proof_row.clone() * value_present,
            JMT_SHA_ROLE_NEW_LEAF_V2,
            147,
            NEW_LEAF_DIGEST_OFFSET_V2,
        ),
        (
            (sibling_row.clone() + split_row.clone()) * non_placeholder_sibling,
            JMT_SHA_ROLE_SIBLING_V2,
            19,
            SIBLING_DIGEST_OFFSET_V2,
        ),
        (
            sibling_row.clone(),
            JMT_SHA_ROLE_OLD_PARENT_V2,
            147,
            OLD_PARENT_DIGEST_OFFSET_V2,
        ),
        (
            sibling_row * new_parent_active,
            JMT_SHA_ROLE_NEW_PARENT_V2,
            275,
            NEW_PARENT_DIGEST_OFFSET_V2,
        ),
        (
            split_row,
            JMT_SHA_ROLE_NEW_PARENT_V2,
            147,
            NEW_PARENT_DIGEST_OFFSET_V2,
        ),
    ];
    for (gate, role, preimage_start, digest_offset) in jobs {
        let role = AB::Expr::from_u64(u64::from(role));
        for block in 0..2 {
            for pair in 0..32 {
                let byte = preimage_start + block * 64 + pair * 2;
                let pair_value = record_byte::<AB, D>(row, byte) * AB::Expr::from_u64(256)
                    + record_byte::<AB, D>(row, byte + 1);
                builder.push_interaction(
                    JMT_SHA_BLOCK_PAIR_BUS_V2,
                    vec![
                        lane.clone(),
                        record.clone(),
                        role.clone(),
                        AB::Expr::from_u64(2),
                        AB::Expr::from_usize(block),
                        AB::Expr::from_usize(pair),
                        pair_value,
                    ],
                    -Count::bounded(gate.clone(), 1),
                );
            }
        }
        for limb in 0..DIGEST_LIMBS_V2 {
            builder.push_interaction(
                JMT_SHA_DIGEST_BUS_V2,
                vec![
                    lane.clone(),
                    record.clone(),
                    role.clone(),
                    AB::Expr::from_u64(2),
                    AB::Expr::from_usize(limb),
                    digest_limb::<AB, D>(row, digest_offset, limb),
                ],
                -Count::bounded(gate.clone(), 1),
            );
        }
    }

    let value_digest_jobs = [
        (
            proof_row.clone() * field::<AB, D>(row, PRIOR_PRESENT_OFFSET_V2),
            JMT_SHA_ROLE_PRIOR_VALUE_V2,
            field::<AB, D>(row, PRIOR_VALUE_BLOCK_COUNT_OFFSET_V2),
            19 + 45,
        ),
        (
            proof_row * field::<AB, D>(row, VALUE_PRESENT_OFFSET_V2),
            JMT_SHA_ROLE_NEW_VALUE_V2,
            field::<AB, D>(row, NEW_VALUE_BLOCK_COUNT_OFFSET_V2),
            147 + 45,
        ),
    ];
    for (gate, role, block_count, digest_start) in value_digest_jobs {
        for limb in 0..DIGEST_LIMBS_V2 {
            builder.push_interaction(
                JMT_SHA_DIGEST_BUS_V2,
                vec![
                    lane.clone(),
                    value_job.clone(),
                    AB::Expr::from_u64(u64::from(role)),
                    block_count.clone(),
                    AB::Expr::from_usize(limb),
                    record_digest_limb::<AB, D>(row, digest_start, limb),
                ],
                -Count::bounded(gate.clone(), 1),
            );
        }
    }
}

fn eval_jmt_source_interactions<AB, const D: usize>(
    builder: &mut AB,
    row: &[AB::Var],
    public: &[AB::Expr],
    first: AB::Expr,
    lane: usize,
) where
    AB: AirBuilder + InteractionBuilder,
{
    let lane_offset = CHUNK_LANE_OFFSET_V2 + lane * CHUNK_LANE_FIELDS_V2;
    let lane_expr = AB::Expr::from_usize(lane);
    let lane_active = public[CHUNK_ACTIVE_OFFSET_V2 + lane].clone();
    let update_count = first.clone() * lane_active.clone();
    let header_bytes = [
        AB::Expr::from_u64(u64::from(JMT_UPDATE_TRACE_VERSION_V2)),
        AB::Expr::from_u64(u64::from(RootGeneration::SettlementV2.version())),
        public[lane_offset + CHUNK_LANE_KIND_OFFSET_V2].clone(),
    ]
    .into_iter()
    .chain(
        public[lane_offset + CHUNK_LANE_TRACE_DIGEST_OFFSET_V2
            ..lane_offset + CHUNK_LANE_TRACE_DIGEST_OFFSET_V2 + 32]
            .iter()
            .cloned(),
    )
    .chain(
        public[lane_offset + CHUNK_LANE_UPDATE_COUNT_OFFSET_V2
            ..lane_offset + CHUNK_LANE_UPDATE_COUNT_OFFSET_V2 + 4]
            .iter()
            .cloned(),
    );
    for (payload_index, byte) in header_bytes.enumerate() {
        builder.push_interaction(
            SOURCE_JMT_PAYLOAD_BYTE_BUS_V2,
            vec![
                lane_expr.clone(),
                AB::Expr::from_u64(RecursiveTraceOpcodeV2::JmtUpdate as u64),
                AB::Expr::ZERO,
                AB::Expr::from_usize(payload_index),
                byte,
            ],
            -Count::bounded(update_count.clone(), 1),
        );
    }

    let promotion_bytes = [AB::Expr::from_u64(u64::from(
        UNIQUENESS_PRECOMMIT_VERSION_V2,
    ))]
    .into_iter()
    .chain(
        public[lane_offset + CHUNK_LANE_POST_ROOT_BYTE_OFFSET_V2
            ..lane_offset + CHUNK_LANE_POST_ROOT_BYTE_OFFSET_V2 + 32]
            .iter()
            .cloned(),
    )
    .chain(
        public[lane_offset + CHUNK_LANE_TRACE_DIGEST_OFFSET_V2
            ..lane_offset + CHUNK_LANE_TRACE_DIGEST_OFFSET_V2 + 32]
            .iter()
            .cloned(),
    );
    for (payload_index, byte) in promotion_bytes.enumerate() {
        builder.push_interaction(
            SOURCE_JMT_PAYLOAD_BYTE_BUS_V2,
            vec![
                lane_expr.clone(),
                AB::Expr::from_u64(RecursiveTraceOpcodeV2::PromoteChildRoot as u64),
                AB::Expr::ZERO,
                AB::Expr::from_usize(payload_index),
                byte,
            ],
            -Count::bounded(first.clone() * lane_active.clone(), 1),
        );
    }

    let record = field::<AB, D>(row, RUNNING_OFFSET_V2);
    for byte_index in 0..JMT_RECORD_BYTES_V2 {
        let byte_active = (1..=OPCODE_COUNT_V2)
            .filter(|opcode| RECORD_LENGTHS_V2[*opcode - 1] > byte_index)
            .map(|opcode| field::<AB, D>(row, OPCODE_OFFSET_V2 + opcode - 1))
            .fold(AB::Expr::ZERO, |sum, selector| sum + selector);
        builder.push_interaction(
            SOURCE_JMT_PAYLOAD_BYTE_BUS_V2,
            vec![
                lane_expr.clone(),
                AB::Expr::from_u64(RecursiveTraceOpcodeV2::JmtMicroOp as u64),
                record.clone(),
                AB::Expr::from_usize(byte_index),
                record_byte::<AB, D>(row, byte_index),
            ],
            -Count::bounded(byte_active, 1),
        );
    }

    let proof_row = field::<AB, D>(row, OPCODE_OFFSET_V2 + 3);
    let terminal_role = field::<AB, D>(row, ROLE_OFFSET_V2 + 3);
    let mutation_gate = proof_row * terminal_role;
    let prior_present = field::<AB, D>(row, PRIOR_PRESENT_OFFSET_V2);
    let value_present = field::<AB, D>(row, VALUE_PRESENT_OFFSET_V2);
    let terminal_limbs = (0..16)
        .map(|limb| {
            field::<AB, D>(row, KEY_OFFSET_V2 + limb * 2)
                + field::<AB, D>(row, KEY_OFFSET_V2 + limb * 2 + 1) * AB::Expr::from_u64(256)
        })
        .collect::<Vec<_>>();
    let mutation_message = |payload_index: usize, payload_byte: AB::Expr| {
        [lane_expr.clone()]
            .into_iter()
            .chain(terminal_limbs.iter().cloned())
            .chain([AB::Expr::from_usize(payload_index), payload_byte])
            .collect::<Vec<_>>()
    };
    for (payload_index, payload_byte) in [
        (
            0,
            AB::Expr::from_u64(u64::from(UNIQUENESS_PRECOMMIT_VERSION_V2)),
        ),
        (
            1,
            prior_present.clone() + value_present.clone() * AB::Expr::from_u64(2),
        ),
    ] {
        builder.push_interaction(
            SOURCE_NET_MUTATION_BYTE_BUS_V2,
            mutation_message(payload_index, payload_byte),
            -Count::bounded(mutation_gate.clone(), 1),
        );
    }
    for byte in 0..TREE_DEFINITION_BYTES_V2 {
        builder.push_interaction(
            SOURCE_NET_MUTATION_BYTE_BUS_V2,
            mutation_message(
                2 + byte,
                field::<AB, D>(row, TREE_DEFINITION_OFFSET_V2 + byte),
            ),
            -Count::bounded(mutation_gate.clone(), 1),
        );
    }
    for byte in 0..TREE_SERIAL_BYTES_V2 {
        builder.push_interaction(
            SOURCE_NET_MUTATION_BYTE_BUS_V2,
            mutation_message(
                2 + TREE_DEFINITION_BYTES_V2 + byte,
                field::<AB, D>(row, TREE_SERIAL_OFFSET_V2 + byte),
            ),
            -Count::bounded(mutation_gate.clone(), 1),
        );
    }
    for byte in 0..32 {
        builder.push_interaction(
            SOURCE_NET_MUTATION_BYTE_BUS_V2,
            mutation_message(
                2 + TREE_DEFINITION_BYTES_V2 + TREE_SERIAL_BYTES_V2 + byte,
                field::<AB, D>(row, KEY_OFFSET_V2 + byte),
            ),
            -Count::bounded(mutation_gate.clone(), 1),
        );
        builder.push_interaction(
            SOURCE_NET_MUTATION_BYTE_BUS_V2,
            mutation_message(
                2 + TREE_DEFINITION_BYTES_V2 + TREE_SERIAL_BYTES_V2 + 32 + byte,
                prior_present.clone() * record_byte::<AB, D>(row, 19 + 45 + byte),
            ),
            -Count::bounded(mutation_gate.clone(), 1),
        );
        builder.push_interaction(
            SOURCE_NET_MUTATION_BYTE_BUS_V2,
            mutation_message(
                2 + TREE_DEFINITION_BYTES_V2 + TREE_SERIAL_BYTES_V2 + 64 + byte,
                value_present.clone() * record_byte::<AB, D>(row, 147 + 45 + byte),
            ),
            -Count::bounded(mutation_gate.clone(), 1),
        );
    }
}

impl<AB, const D: usize> Air<AB> for JmtChunkAirV2<AB::F, D>
where
    AB: AirBuilder + InteractionBuilder,
    AB::F: Field,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.current_slice();
        let next = main.next_slice();
        let preprocessed = builder.preprocessed().clone();
        let prep_local = preprocessed.current_slice();
        let prep_next = preprocessed.next_slice();
        let first: AB::Expr = prep_local[0].into();
        let next_first: AB::Expr = prep_next[0].into();
        builder.assert_bool(first.clone());
        builder.when_first_row().assert_one(first.clone());
        builder.when_transition().assert_zero(next_first);
        let public = builder
            .public_values()
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<AB::Expr>>();
        let one = AB::Expr::ONE;
        let radix = AB::Expr::from_u64(65_536);
        let mut total_records = AB::Expr::ZERO;
        for lane in 0..CHUNK_LANES_V2 {
            let active = public[CHUNK_ACTIVE_OFFSET_V2 + lane].clone();
            builder.assert_bool(active.clone());
            if lane == 0 {
                builder.assert_one(active.clone());
            } else {
                builder.assert_zero(
                    active.clone()
                        * (one.clone() - public[CHUNK_ACTIVE_OFFSET_V2 + lane - 1].clone()),
                );
            }
            let lane_offset = CHUNK_LANE_OFFSET_V2 + lane * CHUNK_LANE_FIELDS_V2;
            total_records += public[lane_offset + CHUNK_LANE_RECORD_COUNT_OFFSET_V2].clone()
                + public[lane_offset + CHUNK_LANE_RECORD_COUNT_OFFSET_V2 + 1].clone()
                    * radix.clone();
            let inactive = one.clone() - active;
            for field_offset in CHUNK_LANE_PRE_ROOT_OFFSET_V2..CHUNK_LANE_TRACE_DIGEST_OFFSET_V2 {
                builder.assert_zero(inactive.clone() * public[lane_offset + field_offset].clone());
            }
            for word in 0..8 {
                let byte_offset = lane_offset + CHUNK_LANE_POST_ROOT_BYTE_OFFSET_V2 + word * 4;
                builder.assert_eq(
                    public[lane_offset + CHUNK_LANE_POST_ROOT_OFFSET_V2 + word * 2].clone(),
                    public[byte_offset + 2].clone() * AB::Expr::from_u64(256)
                        + public[byte_offset + 3].clone(),
                );
                builder.assert_eq(
                    public[lane_offset + CHUNK_LANE_POST_ROOT_OFFSET_V2 + word * 2 + 1].clone(),
                    public[byte_offset].clone() * AB::Expr::from_u64(256)
                        + public[byte_offset + 1].clone(),
                );
            }
            for (index, byte) in noop_update_trace_digest().into_iter().enumerate() {
                builder.assert_zero(
                    inactive.clone()
                        * (public[lane_offset + CHUNK_LANE_TRACE_DIGEST_OFFSET_V2 + index].clone()
                            - AB::Expr::from_u64(u64::from(byte))),
                );
            }
            for field_offset in
                CHUNK_LANE_RECORD_COUNT_OFFSET_V2..CHUNK_LANE_UPDATE_COUNT_OFFSET_V2 + 4
            {
                builder.assert_zero(inactive.clone() * public[lane_offset + field_offset].clone());
            }
            builder.assert_zero(
                inactive
                    * (public[lane_offset + CHUNK_LANE_KIND_OFFSET_V2].clone()
                        - AB::Expr::from_u64(u64::from(JMT_TRACE_NOOP_KIND_V2))),
            );

            let start = lane * ROW_FIELDS_V2 * D;
            let end = start + ROW_FIELDS_V2 * D;
            let lane_public = chunk_lane_public::<AB>(&public, lane);
            eval_row_shape::<AB, D>(builder, &local[start..end], &next[start..end], &lane_public);
            eval_local_path::<AB, D>(builder, &local[start..end], &lane_public);
            eval_transition_state::<AB, D>(
                builder,
                &local[start..end],
                &next[start..end],
                &lane_public,
            );
            eval_jmt_sha_interactions::<AB, D>(builder, &local[start..end], lane);
            eval_jmt_source_interactions::<AB, D>(
                builder,
                &local[start..end],
                &public,
                first.clone(),
                lane,
            );
        }
        builder.assert_eq(
            public[PUBLIC_ROW_COUNT_OFFSET_V2].clone()
                + public[PUBLIC_ROW_COUNT_OFFSET_V2 + 1].clone() * radix,
            total_records,
        );
        builder.assert_zero(public[PUBLIC_ROW_COUNT_OFFSET_V2 + 2].clone());
        builder.assert_zero(public[PUBLIC_ROW_COUNT_OFFSET_V2 + 3].clone());
    }
}

impl BatchAir<Plonky3StarkConfigV2> for JmtChunkAirV2<KoalaBear, 1> {}

pub(super) fn jmt_npo_type() -> NpoTypeId {
    NpoTypeId::new(JMT_NPO_ID_V2)
}

pub(super) fn jmt_chunk_npo_type() -> NpoTypeId {
    NpoTypeId::new(JMT_CHUNK_NPO_ID_V2)
}
