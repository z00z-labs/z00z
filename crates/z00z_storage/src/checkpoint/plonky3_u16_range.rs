//! Packed `u16` decomposition table for the Plan-07 KoalaBear AIR.
//!
//! One circuit word is range-constrained by a 16-row micro-trace. The table
//! receives the word from the canonical WitnessChecks bus, produces the exact
//! sixteen little-endian bit wires consumed by the transition circuit, and
//! reconstructs the word with a row-local running accumulator. This replaces
//! sixteen general-purpose circuit boolean operations per selected word while
//! preserving the same bit-level theorem.

use core::{any::Any, fmt};
use std::collections::HashSet;

use p3_air::{Air, AirBuilder, BaseAir, WindowAccess};
use p3_circuit::ops::{
    ExecutionContext, NonPrimitiveExecutor, NpoConfig, NpoTypeId, Op, OpStateMap,
    PreprocessedWriter, PrimitiveOpType,
};
use p3_circuit::tables::{NonPrimitiveTrace, TraceGeneratorFn, Traces};
use p3_circuit::{
    Circuit, CircuitBuilder, CircuitBuilderError, CircuitError, ExprId, NonPrimitiveOperationData,
    NpoCircuitPlugin, NpoLoweringContext, PreprocessedColumns, WitnessId,
};
use p3_field::{BasedVectorSpace, ExtensionField, Field, PrimeCharacteristicRing, PrimeField64};
use p3_koala_bear::KoalaBear;
use p3_lookup::{Count, InteractionBuilder};
use p3_matrix::dense::RowMajorMatrix;
use p3_util::log2_ceil_usize;
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchAir, BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking,
    TableProver,
};
use z00z_plonky3_circuit_prover::common::take_non_primitive_base_columns;

use super::{Plonky3ChallengeV2, Plonky3StarkConfigV2};

const U16_RANGE_NPO_ID_V2: &str = "z00z/plonky3/u16-range/v2";
const U16_BITS_V2: usize = 16;
const U16_RAW_PREPROCESSED_WORDS_V2: usize = 1 + U16_BITS_V2 * 2;

const U16_WORD_OFFSET_V2: usize = 0;
const U16_BIT_OFFSET_V2: usize = 4;
const U16_ACCUMULATOR_OFFSET_V2: usize = 8;
const U16_MAIN_WIDTH_V2: usize = 9;

const U16_INPUT_INDEX_OFFSET_V2: usize = 0;
const U16_OUTPUT_INDEX_OFFSET_V2: usize = 1;
const U16_OUTPUT_MULTIPLICITY_OFFSET_V2: usize = 2;
const U16_ACTIVE_OFFSET_V2: usize = 3;
const U16_FIRST_OFFSET_V2: usize = 4;
const U16_LAST_OFFSET_V2: usize = 5;
const U16_WEIGHT_OFFSET_V2: usize = 6;
const U16_PREPROCESSED_WIDTH_V2: usize = 7;
const ALU_RAW_PREPROCESSED_WIDTH_V2: usize = 12;
const ALU_A_INDEX_OFFSET_V2: usize = 4;
const ALU_B_INDEX_OFFSET_V2: usize = 5;
const ALU_C_INDEX_OFFSET_V2: usize = 6;
const ALU_OUTPUT_INDEX_OFFSET_V2: usize = 7;
const ALU_A_STATE_OFFSET_V2: usize = 8;
const ALU_B_CREATOR_OFFSET_V2: usize = 9;
const ALU_C_STATE_OFFSET_V2: usize = 10;
const ALU_OUTPUT_CREATOR_OFFSET_V2: usize = 11;

#[derive(Clone, Debug)]
struct U16RangeWordV2 {
    input_wid: WitnessId,
    output_wids: [WitnessId; U16_BITS_V2],
    word: u16,
}

#[derive(Debug, Default)]
struct U16RangeStateV2 {
    words: Vec<U16RangeWordV2>,
}

#[derive(Clone)]
struct U16RangeExecutorV2 {
    op_type: NpoTypeId,
}

impl fmt::Debug for U16RangeExecutorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("U16RangeExecutorV2")
            .field("op_type", &self.op_type)
            .finish()
    }
}

impl NonPrimitiveExecutor<Plonky3ChallengeV2> for U16RangeExecutorV2 {
    fn execute(
        &self,
        inputs: &[Vec<WitnessId>],
        outputs: &[Vec<WitnessId>],
        ctx: &mut ExecutionContext<'_, Plonky3ChallengeV2>,
    ) -> Result<(), CircuitError> {
        if inputs.len() != 1
            || inputs[0].len() != 1
            || outputs.len() != U16_BITS_V2
            || outputs.iter().any(|group| group.len() != 1)
        {
            return Err(CircuitError::NonPrimitiveOpLayoutMismatch {
                op: self.op_type.clone(),
                expected: "one embedded-base input and sixteen bit outputs".into(),
                got: inputs.len(),
            });
        }
        let value = ctx.get_witness(inputs[0][0])?;
        let coefficients =
            <Plonky3ChallengeV2 as BasedVectorSpace<KoalaBear>>::as_basis_coefficients_slice(
                &value,
            );
        if coefficients.len() != 4
            || coefficients[1..]
                .iter()
                .any(|&coefficient| coefficient != KoalaBear::ZERO)
        {
            return Err(CircuitError::InvalidPreprocessedValues);
        }
        let canonical = coefficients[0].as_canonical_u64();
        let word = u16::try_from(canonical).map_err(|_| CircuitError::InvalidPreprocessedValues)?;
        let output_wids: [WitnessId; U16_BITS_V2] = outputs
            .iter()
            .map(|group| group[0])
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| CircuitError::InvalidPreprocessedValues)?;
        for (bit_index, &output_wid) in output_wids.iter().enumerate() {
            let bit = u8::from(((word >> bit_index) & 1) != 0);
            ctx.set_witness(
                output_wid,
                Plonky3ChallengeV2::new([
                    KoalaBear::from_u8(bit),
                    KoalaBear::ZERO,
                    KoalaBear::ZERO,
                    KoalaBear::ZERO,
                ]),
            )?;
        }
        ctx.get_op_state_mut::<U16RangeStateV2>(&self.op_type)
            .words
            .push(U16RangeWordV2 {
                input_wid: inputs[0][0],
                output_wids,
                word,
            });
        Ok(())
    }

    fn op_type(&self) -> &NpoTypeId {
        &self.op_type
    }

    fn preprocess(
        &self,
        inputs: &[Vec<WitnessId>],
        outputs: &[Vec<WitnessId>],
        preprocessed: &mut dyn PreprocessedWriter<Plonky3ChallengeV2>,
    ) -> Result<(), CircuitError> {
        preprocessed.register_non_primitive_witness_reads(&self.op_type, &inputs[0])?;
        for output in outputs {
            preprocessed.register_non_primitive_output_index(&self.op_type, output);
            preprocessed.register_non_primitive_preprocessed_no_read(
                &self.op_type,
                &[Plonky3ChallengeV2::ONE],
            );
        }
        Ok(())
    }

    fn num_exposed_outputs(&self) -> Option<usize> {
        Some(U16_BITS_V2)
    }

    fn boxed(&self) -> Box<dyn NonPrimitiveExecutor<Plonky3ChallengeV2>> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Copy, Debug)]
struct U16RangeConfigV2;

#[derive(Clone, Copy, Debug)]
struct U16RangePluginV2;

impl NpoCircuitPlugin<Plonky3ChallengeV2> for U16RangePluginV2 {
    fn type_id(&self) -> NpoTypeId {
        u16_range_npo_type()
    }

    fn lower(
        &self,
        data: &NonPrimitiveOperationData<Plonky3ChallengeV2>,
        output_exprs: &[(u32, ExprId)],
        ctx: &mut NpoLoweringContext<'_, Plonky3ChallengeV2>,
    ) -> Result<Op<Plonky3ChallengeV2>, CircuitBuilderError> {
        if data.input_exprs.len() != 1
            || data.input_exprs[0].len() != 1
            || data.output_exprs.len() != U16_BITS_V2
            || data.output_exprs.iter().any(|group| group.len() != 1)
            || output_exprs.len() != U16_BITS_V2
            || data.params.is_some()
        {
            return Err(CircuitBuilderError::NonPrimitiveOpArity {
                op: "U16RangeV2",
                expected: "one input and sixteen outputs".into(),
                got: output_exprs.len(),
            });
        }
        let input =
            ctx.resolve_witness_id(data.input_exprs[0][0], || "U16RangeV2 input word".into())?;
        let mut ordered_outputs = output_exprs.to_vec();
        ordered_outputs.sort_by_key(|(index, _)| *index);
        if ordered_outputs
            .iter()
            .enumerate()
            .any(|(index, (actual, _))| usize::try_from(*actual).ok() != Some(index))
        {
            return Err(CircuitBuilderError::NonPrimitiveOpArity {
                op: "U16RangeV2",
                expected: "ordered output indexes 0..16".into(),
                got: ordered_outputs.len(),
            });
        }
        let outputs = ordered_outputs
            .into_iter()
            .map(|(_, expression)| {
                ctx.resolve_witness_id(expression, || "U16RangeV2 output bit".into())
                    .map(|wid| vec![wid])
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Op::NonPrimitiveOpWithExecutor {
            inputs: vec![vec![input]],
            outputs,
            executor: Box::new(U16RangeExecutorV2 {
                op_type: u16_range_npo_type(),
            }),
            op_id: data.op_id,
        })
    }

    fn trace_generator(&self) -> TraceGeneratorFn<Plonky3ChallengeV2> {
        generate_u16_range_trace_v2
    }

    fn config(&self) -> NpoConfig {
        NpoConfig::new(U16RangeConfigV2)
    }
}

#[derive(Clone, Debug)]
struct U16RangeTraceV2 {
    words: Vec<U16RangeWordV2>,
}

impl NonPrimitiveTrace<Plonky3ChallengeV2> for U16RangeTraceV2 {
    fn op_type(&self) -> NpoTypeId {
        u16_range_npo_type()
    }

    fn rows(&self) -> usize {
        self.words.len().saturating_mul(U16_BITS_V2)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed_clone(&self) -> Box<dyn NonPrimitiveTrace<Plonky3ChallengeV2>> {
        Box::new(self.clone())
    }
}

fn generate_u16_range_trace_v2(
    states: &OpStateMap,
) -> Result<Option<Box<dyn NonPrimitiveTrace<Plonky3ChallengeV2>>>, CircuitError> {
    let Some(state) = states
        .get(&u16_range_npo_type())
        .and_then(|state| state.downcast_ref::<U16RangeStateV2>())
    else {
        return Ok(None);
    };
    if state.words.is_empty() {
        return Err(CircuitError::InvalidPreprocessedValues);
    }
    Ok(Some(Box::new(U16RangeTraceV2 {
        words: state.words.clone(),
    })))
}

#[derive(Clone, Debug)]
struct U16RangeAirV2<F, const D: usize> {
    preprocessed: Vec<F>,
    min_height: usize,
}

impl<F: Field, const D: usize> U16RangeAirV2<F, D> {
    fn new(preprocessed: Vec<F>, min_height: usize) -> Self {
        Self {
            preprocessed,
            min_height: min_height.max(U16_BITS_V2),
        }
    }

    fn preprocessed_for_words(words: &[U16RangeWordV2]) -> Vec<F>
    where
        F: PrimeCharacteristicRing,
    {
        let mut preprocessed =
            Vec::with_capacity(words.len() * U16_BITS_V2 * U16_PREPROCESSED_WIDTH_V2);
        for word in words {
            for (bit_index, &output_wid) in word.output_wids.iter().enumerate() {
                preprocessed.push(if bit_index == 0 {
                    word.input_wid.base_field_index::<F, D>()
                } else {
                    F::ZERO
                });
                preprocessed.push(output_wid.base_field_index::<F, D>());
                preprocessed.push(F::ZERO);
                preprocessed.push(F::ONE);
                preprocessed.push(F::from_bool(bit_index == 0));
                preprocessed.push(F::from_bool(bit_index + 1 == U16_BITS_V2));
                preprocessed.push(F::from_u64(1_u64 << bit_index));
            }
        }
        preprocessed
    }

    fn trace_to_matrix(words: &[U16RangeWordV2], min_height: usize) -> RowMajorMatrix<F>
    where
        F: PrimeCharacteristicRing,
    {
        let mut values = Vec::with_capacity(words.len() * U16_BITS_V2 * U16_MAIN_WIDTH_V2);
        for word in words {
            let mut accumulator = 0_u64;
            for bit_index in 0..U16_BITS_V2 {
                let bit = u64::from((word.word >> bit_index) & 1);
                values.push(F::from_u64(u64::from(word.word)));
                values.extend(core::iter::repeat_n(F::ZERO, D.saturating_sub(1)));
                values.push(F::from_u64(bit));
                values.extend(core::iter::repeat_n(F::ZERO, D.saturating_sub(1)));
                values.push(F::from_u64(accumulator));
                accumulator += bit << bit_index;
            }
        }
        let mut matrix = RowMajorMatrix::new(values, D * 2 + 1);
        matrix.pad_to_min_power_of_two_height(min_height.max(U16_BITS_V2), F::ZERO);
        matrix
    }
}

impl<F: Field, const D: usize> BaseAir<F> for U16RangeAirV2<F, D> {
    fn width(&self) -> usize {
        D * 2 + 1
    }

    fn preprocessed_width(&self) -> usize {
        U16_PREPROCESSED_WIDTH_V2
    }

    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
        let mut matrix = RowMajorMatrix::from_flat_padded(
            self.preprocessed.clone(),
            U16_PREPROCESSED_WIDTH_V2,
            F::ZERO,
        );
        matrix.pad_to_min_power_of_two_height(self.min_height, F::ZERO);
        Some(matrix)
    }

    fn preprocessed_next_row_columns(&self) -> Vec<usize> {
        Vec::new()
    }
}

impl<AB, const D: usize> Air<AB> for U16RangeAirV2<AB::F, D>
where
    AB: AirBuilder + InteractionBuilder,
    AB::F: Field,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.current_slice();
        let next = main.next_slice();
        let preprocessed = builder.preprocessed().clone();
        let prep = preprocessed.current_slice();

        let word = (0..D)
            .map(|index| local[U16_WORD_OFFSET_V2 + index].into())
            .collect::<Vec<AB::Expr>>();
        let next_word = (0..D)
            .map(|index| next[U16_WORD_OFFSET_V2 + index].into())
            .collect::<Vec<AB::Expr>>();
        let bit = (0..D)
            .map(|index| local[U16_BIT_OFFSET_V2 + index].into())
            .collect::<Vec<AB::Expr>>();
        let accumulator: AB::Expr = local[U16_ACCUMULATOR_OFFSET_V2].into();
        let next_accumulator: AB::Expr = next[U16_ACCUMULATOR_OFFSET_V2].into();
        let active: AB::Expr = prep[U16_ACTIVE_OFFSET_V2].into();
        let first: AB::Expr = prep[U16_FIRST_OFFSET_V2].into();
        let last: AB::Expr = prep[U16_LAST_OFFSET_V2].into();
        let weight: AB::Expr = prep[U16_WEIGHT_OFFSET_V2].into();

        {
            let mut when_active = builder.when(active.clone());
            when_active.assert_bool(bit[0].clone());
            for coefficient in 1..D {
                when_active.assert_zero(bit[coefficient].clone());
                when_active.assert_zero(word[coefficient].clone());
            }
        }
        {
            let mut when_first = builder.when(first.clone());
            when_first.assert_zero(accumulator.clone());
        }
        let accumulated = accumulator.clone() + bit[0].clone() * weight;
        {
            let mut when_last = builder.when(last.clone());
            when_last.assert_eq(word[0].clone(), accumulated.clone());
        }
        {
            let mut when_continuing = builder.when(active - last);
            for coefficient in 0..D {
                when_continuing
                    .assert_eq(next_word[coefficient].clone(), word[coefficient].clone());
            }
            when_continuing.assert_eq(next_accumulator, accumulated);
        }

        let input_index: AB::Expr = prep[U16_INPUT_INDEX_OFFSET_V2].into();
        let mut input = Vec::with_capacity(D + 1);
        input.push(input_index);
        input.extend(word);
        builder.push_interaction("WitnessChecks", input, Count::bounded(-first, 1));

        let output_index: AB::Expr = prep[U16_OUTPUT_INDEX_OFFSET_V2].into();
        let output_multiplicity: AB::Expr = prep[U16_OUTPUT_MULTIPLICITY_OFFSET_V2].into();
        let mut output = Vec::with_capacity(D + 1);
        output.push(output_index);
        output.extend(bit);
        builder.push_interaction(
            "WitnessChecks",
            output,
            Count::bounded(output_multiplicity, 1),
        );
    }
}

impl BatchAir<Plonky3StarkConfigV2> for U16RangeAirV2<KoalaBear, 4> {}

#[derive(Clone, Copy, Debug)]
pub(super) struct U16RangeProverV2;

impl U16RangeProverV2 {
    fn batch_instance(
        &self,
        packing: &TablePacking,
        traces: &Traces<Plonky3ChallengeV2>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces.non_primitive_trace::<U16RangeTraceV2>(&u16_range_npo_type())?;
        if trace.words.is_empty() {
            return None;
        }
        let rows = trace.words.len().checked_mul(U16_BITS_V2)?;
        let min_height = packing.min_trace_height();
        let preprocessed = U16RangeAirV2::<KoalaBear, 4>::preprocessed_for_words(&trace.words);
        let air = U16RangeAirV2::<KoalaBear, 4>::new(preprocessed, min_height);
        Some(BatchTableInstance {
            op_type: u16_range_npo_type(),
            air: DynamicAirEntry::new(Box::new(air)),
            trace: U16RangeAirV2::<KoalaBear, 4>::trace_to_matrix(&trace.words, min_height),
            public_values: Vec::new(),
            rows,
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for U16RangeProverV2 {
    fn op_type(&self) -> NpoTypeId {
        u16_range_npo_type()
    }

    fn batch_instance_d1(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d2(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<p3_field::extension::BinomialExtensionField<KoalaBear, 2>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d4(
        &self,
        _config: &Plonky3StarkConfigV2,
        packing: &TablePacking,
        traces: &Traces<Plonky3ChallengeV2>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        self.batch_instance(packing, traces)
    }

    fn batch_instance_d6(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<p3_field::extension::BinomialExtensionField<KoalaBear, 6>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d8(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<p3_field::extension::BinomialExtensionField<KoalaBear, 8>>,
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
        if degree != 4
            || circuit_extension_degree != 4
            || entry.rows == 0
            || !entry.rows.is_multiple_of(U16_BITS_V2)
            || entry.lanes != 1
            || !entry.public_values.is_empty()
        {
            return Err("u16-range table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(
            U16RangeAirV2::<KoalaBear, 4>::new(Vec::new(), U16_BITS_V2),
        )))
    }

    fn air_with_committed_preprocessed(
        &self,
        committed: Vec<KoalaBear>,
        min_height: usize,
        lanes: usize,
        circuit_extension_degree: u32,
    ) -> Option<DynamicAirEntry<Plonky3StarkConfigV2>> {
        (lanes == 1
            && circuit_extension_degree == 4
            && !committed.is_empty()
            && committed.len().is_multiple_of(U16_PREPROCESSED_WIDTH_V2))
        .then(|| {
            DynamicAirEntry::new(Box::new(U16RangeAirV2::<KoalaBear, 4>::new(
                committed, min_height,
            )))
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct U16RangePreprocessorV2;

struct U16RangeRuntimeMetadataV2 {
    creator_flags: Vec<[bool; U16_BITS_V2]>,
}

fn witness_id_from_index(index: Plonky3ChallengeV2) -> Result<u32, CircuitError> {
    let index = <Plonky3ChallengeV2 as ExtensionField<KoalaBear>>::as_base(&index)
        .ok_or(CircuitError::InvalidPreprocessedValues)?
        .as_canonical_u64();
    if index % 4 != 0 {
        return Err(CircuitError::InvalidPreprocessedValues);
    }
    let witness_id = index / 4;
    u32::try_from(witness_id).map_err(|_| CircuitError::InvalidPreprocessedValues)
}

fn u16_output_creator_flags(
    circuit: &Circuit<Plonky3ChallengeV2>,
    preprocessed: &PreprocessedColumns<Plonky3ChallengeV2, 4>,
) -> Result<Vec<[bool; U16_BITS_V2]>, CircuitError> {
    let alu_preprocessed = preprocessed
        .primitive
        .get(PrimitiveOpType::Alu as usize)
        .ok_or(CircuitError::InvalidPreprocessedValues)?;
    if !alu_preprocessed
        .len()
        .is_multiple_of(ALU_RAW_PREPROCESSED_WIDTH_V2)
    {
        return Err(CircuitError::InvalidPreprocessedValues);
    }
    let mut alu_rows = alu_preprocessed.chunks_exact(ALU_RAW_PREPROCESSED_WIDTH_V2);
    let mut defined = HashSet::new();
    let mut creator_flags = Vec::new();
    for op in &circuit.ops {
        match op {
            Op::Const { out, .. } | Op::Public { out, .. } => {
                defined.insert(out.0);
            }
            Op::Alu { .. } => {
                let row = alu_rows
                    .next()
                    .ok_or(CircuitError::InvalidPreprocessedValues)?;
                for (index_offset, control_offset, creator_value) in [
                    (
                        ALU_A_INDEX_OFFSET_V2,
                        ALU_A_STATE_OFFSET_V2,
                        Plonky3ChallengeV2::TWO,
                    ),
                    (
                        ALU_B_INDEX_OFFSET_V2,
                        ALU_B_CREATOR_OFFSET_V2,
                        Plonky3ChallengeV2::ONE,
                    ),
                    (
                        ALU_C_INDEX_OFFSET_V2,
                        ALU_C_STATE_OFFSET_V2,
                        Plonky3ChallengeV2::TWO,
                    ),
                    (
                        ALU_OUTPUT_INDEX_OFFSET_V2,
                        ALU_OUTPUT_CREATOR_OFFSET_V2,
                        Plonky3ChallengeV2::ONE,
                    ),
                ] {
                    if row[control_offset] == creator_value {
                        defined.insert(witness_id_from_index(row[index_offset])?);
                    }
                }
            }
            Op::NonPrimitiveOpWithExecutor {
                outputs, executor, ..
            } => {
                let exposed_outputs = executor.num_exposed_outputs().unwrap_or(outputs.len());
                let is_u16 = executor.op_type() == &u16_range_npo_type();
                if is_u16
                    && (exposed_outputs != U16_BITS_V2
                        || outputs.len() < U16_BITS_V2
                        || outputs
                            .iter()
                            .take(U16_BITS_V2)
                            .any(|output| output.len() != 1))
                {
                    return Err(CircuitError::InvalidPreprocessedValues);
                }
                let mut operation_flags = [false; U16_BITS_V2];
                for (output_index, output) in outputs.iter().take(exposed_outputs).enumerate() {
                    for witness in output {
                        let is_creator = defined.insert(witness.0);
                        if is_u16 {
                            operation_flags[output_index] = is_creator;
                        }
                    }
                }
                if is_u16 {
                    creator_flags.push(operation_flags);
                }
            }
            Op::Hint { .. } => {}
        }
    }
    if !alu_rows.remainder().is_empty() || alu_rows.next().is_some() {
        return Err(CircuitError::InvalidPreprocessedValues);
    }
    Ok(creator_flags)
}

impl z00z_plonky3_circuit_prover::common::NpoPreprocessor<KoalaBear> for U16RangePreprocessorV2 {
    fn capture_runtime_metadata(
        &self,
        circuit: &dyn Any,
        preprocessed: &mut dyn Any,
    ) -> Result<Option<Box<dyn Any + Send + Sync>>, CircuitError> {
        let Some(circuit) = circuit.downcast_ref::<Circuit<Plonky3ChallengeV2>>() else {
            return Ok(None);
        };
        let Some(preprocessed) =
            preprocessed.downcast_mut::<PreprocessedColumns<Plonky3ChallengeV2, 4>>()
        else {
            return Ok(None);
        };
        Ok(Some(Box::new(U16RangeRuntimeMetadataV2 {
            creator_flags: u16_output_creator_flags(circuit, preprocessed)?,
        })))
    }

    fn preprocess_with_runtime_metadata(
        &self,
        runtime_metadata: &(dyn Any + Send + Sync),
        preprocessed: &mut dyn Any,
    ) -> Result<p3_circuit::ops::NonPrimitivePreprocessedMap<KoalaBear>, CircuitError> {
        let metadata = runtime_metadata
            .downcast_ref::<U16RangeRuntimeMetadataV2>()
            .ok_or(CircuitError::InvalidPreprocessedValues)?;
        let preprocessed = preprocessed
            .downcast_mut::<PreprocessedColumns<Plonky3ChallengeV2, 4>>()
            .ok_or(CircuitError::InvalidPreprocessedValues)?;
        u16_preprocess_with_creator_flags(preprocessed, &metadata.creator_flags)
    }

    fn preprocess(
        &self,
        circuit: &dyn Any,
        preprocessed: &mut dyn Any,
    ) -> Result<p3_circuit::ops::NonPrimitivePreprocessedMap<KoalaBear>, CircuitError> {
        let result = p3_circuit::ops::NonPrimitivePreprocessedMap::new();
        let Some(circuit) = circuit.downcast_ref::<Circuit<Plonky3ChallengeV2>>() else {
            return Ok(result);
        };
        let Some(preprocessed) =
            preprocessed.downcast_mut::<PreprocessedColumns<Plonky3ChallengeV2, 4>>()
        else {
            return Ok(result);
        };
        let creator_flags = u16_output_creator_flags(circuit, preprocessed)?;
        return u16_preprocess_with_creator_flags(preprocessed, &creator_flags);
    }
}

fn u16_preprocess_with_creator_flags(
    preprocessed: &mut PreprocessedColumns<Plonky3ChallengeV2, 4>,
    creator_flags: &[[bool; U16_BITS_V2]],
) -> Result<p3_circuit::ops::NonPrimitivePreprocessedMap<KoalaBear>, CircuitError> {
    let mut result = p3_circuit::ops::NonPrimitivePreprocessedMap::new();
    let Some(raw) = take_non_primitive_base_columns::<Plonky3ChallengeV2, KoalaBear, 4>(
        preprocessed,
        &u16_range_npo_type(),
    )?
    else {
        return Ok(result);
    };
    if raw.is_empty() || !raw.len().is_multiple_of(U16_RAW_PREPROCESSED_WORDS_V2) {
        return Err(CircuitError::InvalidPreprocessedValues);
    }
    if creator_flags.len() != raw.len() / U16_RAW_PREPROCESSED_WORDS_V2 {
        return Err(CircuitError::InvalidPreprocessedValues);
    }
    let mut committed = Vec::with_capacity(
        raw.len() / U16_RAW_PREPROCESSED_WORDS_V2 * U16_BITS_V2 * U16_PREPROCESSED_WIDTH_V2,
    );
    let negative_one = KoalaBear::ZERO - KoalaBear::ONE;
    for (operation, operation_creator_flags) in raw
        .chunks_exact(U16_RAW_PREPROCESSED_WORDS_V2)
        .zip(creator_flags)
    {
        let input_index = operation[0];
        for bit_index in 0..U16_BITS_V2 {
            let output_index = operation[1 + bit_index * 2];
            let output_wid = usize::try_from(output_index.as_canonical_u64() / 4)
                .map_err(|_| CircuitError::InvalidPreprocessedValues)?;
            let multiplicity = if operation_creator_flags[bit_index] {
                KoalaBear::from_u32(preprocessed.ext_reads.get(output_wid).copied().unwrap_or(0))
            } else {
                negative_one
            };
            committed.push(if bit_index == 0 {
                input_index
            } else {
                KoalaBear::ZERO
            });
            committed.push(output_index);
            committed.push(multiplicity);
            committed.push(KoalaBear::ONE);
            committed.push(KoalaBear::from_bool(bit_index == 0));
            committed.push(KoalaBear::from_bool(bit_index + 1 == U16_BITS_V2));
            committed.push(KoalaBear::from_u64(1_u64 << bit_index));
        }
    }
    result.insert(u16_range_npo_type(), committed);
    Ok(result)
}

#[derive(Clone, Copy, Debug)]
pub(super) struct U16RangeAirBuilderV2;

impl z00z_plonky3_circuit_prover::common::NpoAirBuilder<Plonky3StarkConfigV2, 4>
    for U16RangeAirBuilderV2
{
    fn try_build(
        &self,
        op_type: &NpoTypeId,
        preprocessed: &mut Vec<KoalaBear>,
        min_height: usize,
        lanes: usize,
        _constraint_profile: z00z_plonky3_circuit_prover::ConstraintProfile,
        retain_preprocessed_columns: bool,
    ) -> Option<(
        z00z_plonky3_circuit_prover::common::CircuitTableAir<Plonky3StarkConfigV2, 4>,
        usize,
    )> {
        if op_type != &u16_range_npo_type()
            || lanes != 1
            || preprocessed.is_empty()
            || !preprocessed.len().is_multiple_of(U16_PREPROCESSED_WIDTH_V2)
        {
            return None;
        }
        let rows = preprocessed.len() / U16_PREPROCESSED_WIDTH_V2;
        if !rows.is_multiple_of(U16_BITS_V2) {
            return None;
        }
        let committed = if retain_preprocessed_columns {
            preprocessed.clone()
        } else {
            core::mem::take(preprocessed)
        };
        let padded_rows = min_height.max(rows).next_power_of_two();
        Some((
            z00z_plonky3_circuit_prover::common::CircuitTableAir::Dynamic(DynamicAirEntry::new(
                Box::new(U16RangeAirV2::<KoalaBear, 4>::new(committed, min_height)),
            )),
            log2_ceil_usize(padded_rows),
        ))
    }
}

pub(super) fn u16_range_npo_type() -> NpoTypeId {
    NpoTypeId::new(U16_RANGE_NPO_ID_V2)
}

pub(super) fn register_u16_range_npo(circuit: &mut CircuitBuilder<Plonky3ChallengeV2>) {
    circuit.register_npo(U16RangePluginV2);
}

pub(super) fn constrain_u16_bits(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    word: ExprId,
) -> Result<[ExprId; U16_BITS_V2], CircuitBuilderError> {
    let (_, _, outputs) = circuit.push_non_primitive_op_with_outputs(
        u16_range_npo_type(),
        vec![vec![word]],
        vec![Some("u16_range_bit"); U16_BITS_V2],
        None,
        "u16_range",
    );
    outputs
        .into_iter()
        .map(|output| output.ok_or(CircuitBuilderError::MissingOutput))
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|outputs: Vec<_>| CircuitBuilderError::NonPrimitiveOpArity {
            op: "U16RangeV2",
            expected: "sixteen outputs".into(),
            got: outputs.len(),
        })
}
