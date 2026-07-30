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
use p3_matrix::dense::RowMajorMatrix;
use z00z_plonky3_circuit_prover::batch_stark_prover::BatchAir;

use super::plonky3_epoch_jmt_constraints_local::eval_local_path;
use super::plonky3_epoch_jmt_constraints_row::eval_row_shape;
use super::plonky3_epoch_jmt_constraints_transition::eval_transition_state;
use super::{Plonky3StarkConfigV2, EPOCH_CHUNK_BYTES_V2};
use crate::settlement::JMT_CIRCUIT_HEADER_BYTES_V2;

pub(super) const JMT_NPO_ID_V2: &str = "z00z/plonky3/epoch-jmt-update/v2";
pub(super) const JMT_MIN_ROWS_V2: usize = 8;
pub(super) const JMT_RECORD_BYTES_V2: usize = 403;
pub(super) const JMT_HEADER_FIELDS_V2: usize = JMT_CIRCUIT_HEADER_BYTES_V2;
pub(super) const STATEMENT_LIMBS_V2: usize = EPOCH_CHUNK_BYTES_V2 / core::mem::size_of::<u16>();
pub(super) const PUBLIC_FIELDS_V2: usize = STATEMENT_LIMBS_V2 + JMT_HEADER_FIELDS_V2;
pub(super) const PUBLIC_ROW_COUNT_OFFSET_V2: usize = 21;
pub(super) const PUBLIC_INPUT_ROOT_OFFSET_V2: usize = 49;
pub(super) const PUBLIC_OUTPUT_ROOT_OFFSET_V2: usize = 65;
pub(super) const PUBLIC_HEADER_OFFSET_V2: usize = STATEMENT_LIMBS_V2;

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
pub(super) const ROW_FIELDS_V2: usize = SELECTED_BITS_OFFSET_V2 + SELECTED_BITS_COUNT_V2;
pub(super) const CALL_FIELDS_V2: usize = PUBLIC_FIELDS_V2 + ROW_FIELDS_V2;
pub(super) const PREPROCESSED_WIDTH_V2: usize = 1;

pub(super) const RECORD_LENGTHS_V2: [usize; OPCODE_COUNT_V2] =
    [159, 52, 83, 275, 10, 6, 403, 10, 275];

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

pub(super) fn jmt_npo_type() -> NpoTypeId {
    NpoTypeId::new(JMT_NPO_ID_V2)
}
