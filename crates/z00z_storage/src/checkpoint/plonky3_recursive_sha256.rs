//! Storage-owned SHA-256 compression NPO for recursive checkpoint circuits.
//!
//! Recursive normalization used to expand every SHA-256 bit operation into the
//! general D=4 ALU. This table keeps the identical compression relation in the
//! already audited SHA AIR and links its limbs to circuit witnesses through the
//! canonical `WitnessChecks` bus.

use core::{any::Any, fmt};

use p3_circuit::ops::{
    ExecutionContext, NonPrimitiveExecutor, NpoConfig, NpoTypeId, Op, OpStateMap,
    PreprocessedWriter,
};
use p3_circuit::tables::{NonPrimitiveTrace, TraceGeneratorFn, Traces};
use p3_circuit::{
    CircuitBuilder, CircuitBuilderError, CircuitError, ExprId, NonPrimitiveOperationData,
    NpoCircuitPlugin, NpoLoweringContext, PreprocessedColumns, WitnessId,
};
use p3_field::{BasedVectorSpace, PrimeCharacteristicRing, PrimeField64};
use p3_koala_bear::KoalaBear;
use p3_util::log2_ceil_usize;
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking, TableProver,
};
use z00z_plonky3_circuit_prover::common::{
    take_non_primitive_base_columns, NpoAirBuilder, NpoPreprocessor,
};

use super::plonky3_epoch_sha256_columns::{
    ShaAirRoleV2, ShaAirV2, ShaRowV2, RECURSIVE_SHA_INPUT_LIMBS_V2, RECURSIVE_SHA_OUTPUT_LIMBS_V2,
    RECURSIVE_SHA_PREPROCESSED_WIDTH_V2, ROW_FIELDS_V2, SHA_ROWS_V2,
};
use super::plonky3_epoch_sha256_witness::compression_rows;
use super::{Plonky3ChallengeV2, Plonky3StarkConfigV2};

const RECURSIVE_SHA_RAW_PREPROCESSED_FIELDS_V2: usize =
    RECURSIVE_SHA_INPUT_LIMBS_V2 + RECURSIVE_SHA_OUTPUT_LIMBS_V2 * 2;

#[derive(Clone, Debug)]
struct RecursiveShaOperationV2 {
    input_wids: Vec<WitnessId>,
    output_wids: Vec<WitnessId>,
    rows: Vec<ShaRowV2>,
}

#[derive(Debug, Default)]
struct RecursiveShaStateV2 {
    operations: Vec<RecursiveShaOperationV2>,
}

#[derive(Clone)]
struct RecursiveShaExecutorV2 {
    op_type: NpoTypeId,
}

impl fmt::Debug for RecursiveShaExecutorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RecursiveShaExecutorV2")
            .field("op_type", &self.op_type)
            .finish()
    }
}

fn embedded_u16(value: &Plonky3ChallengeV2) -> Result<u16, CircuitError> {
    let coefficients =
        <Plonky3ChallengeV2 as BasedVectorSpace<KoalaBear>>::as_basis_coefficients_slice(value);
    if coefficients.len() != 4
        || coefficients[1..]
            .iter()
            .any(|&coefficient| coefficient != KoalaBear::ZERO)
    {
        return Err(CircuitError::InvalidPreprocessedValues);
    }
    u16::try_from(coefficients[0].as_canonical_u64())
        .map_err(|_| CircuitError::InvalidPreprocessedValues)
}

fn embedded_base(value: u16) -> Plonky3ChallengeV2 {
    Plonky3ChallengeV2::new([
        KoalaBear::from_u16(value),
        KoalaBear::ZERO,
        KoalaBear::ZERO,
        KoalaBear::ZERO,
    ])
}

impl NonPrimitiveExecutor<Plonky3ChallengeV2> for RecursiveShaExecutorV2 {
    fn execute(
        &self,
        inputs: &[Vec<WitnessId>],
        outputs: &[Vec<WitnessId>],
        ctx: &mut ExecutionContext<'_, Plonky3ChallengeV2>,
    ) -> Result<(), CircuitError> {
        if inputs.len() != 1
            || inputs[0].len() != RECURSIVE_SHA_INPUT_LIMBS_V2
            || outputs.len() != RECURSIVE_SHA_OUTPUT_LIMBS_V2
            || outputs.iter().any(|group| group.len() != 1)
        {
            return Err(CircuitError::NonPrimitiveOpLayoutMismatch {
                op: self.op_type.clone(),
                expected: "48 embedded-u16 inputs and 16 embedded-u16 outputs".into(),
                got: inputs.len(),
            });
        }
        let limbs = inputs[0]
            .iter()
            .map(|&wid| ctx.get_witness(wid).and_then(|value| embedded_u16(&value)))
            .collect::<Result<Vec<_>, _>>()?;
        let input_state: [u32; 8] = core::array::from_fn(|word| {
            u32::from(limbs[word * 2]) | (u32::from(limbs[word * 2 + 1]) << 16)
        });
        let mut block = [0_u8; 64];
        for word in 0..16 {
            let offset = 16 + word * 2;
            let value = u32::from(limbs[offset]) | (u32::from(limbs[offset + 1]) << 16);
            block[word * 4..word * 4 + 4].copy_from_slice(&value.to_be_bytes());
        }
        let (rows, output_state) = compression_rows(input_state, &block)
            .map_err(|_| CircuitError::InvalidPreprocessedValues)?;
        let output_wids = outputs.iter().map(|group| group[0]).collect::<Vec<_>>();
        for (word, value) in output_state.into_iter().enumerate() {
            ctx.set_witness(output_wids[word * 2], embedded_base(value as u16))?;
            ctx.set_witness(
                output_wids[word * 2 + 1],
                embedded_base((value >> 16) as u16),
            )?;
        }
        ctx.get_op_state_mut::<RecursiveShaStateV2>(&self.op_type)
            .operations
            .push(RecursiveShaOperationV2 {
                input_wids: inputs[0].clone(),
                output_wids,
                rows,
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
        Some(RECURSIVE_SHA_OUTPUT_LIMBS_V2)
    }

    fn boxed(&self) -> Box<dyn NonPrimitiveExecutor<Plonky3ChallengeV2>> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Copy, Debug)]
struct RecursiveShaConfigV2;

#[derive(Clone, Copy, Debug)]
struct RecursiveShaPluginV2;

impl NpoCircuitPlugin<Plonky3ChallengeV2> for RecursiveShaPluginV2 {
    fn type_id(&self) -> NpoTypeId {
        recursive_sha_npo_type()
    }

    fn lower(
        &self,
        data: &NonPrimitiveOperationData<Plonky3ChallengeV2>,
        output_exprs: &[(u32, ExprId)],
        ctx: &mut NpoLoweringContext<'_, Plonky3ChallengeV2>,
    ) -> Result<Op<Plonky3ChallengeV2>, CircuitBuilderError> {
        if data.input_exprs.len() != 1
            || data.input_exprs[0].len() != RECURSIVE_SHA_INPUT_LIMBS_V2
            || data.output_exprs.len() != RECURSIVE_SHA_OUTPUT_LIMBS_V2
            || data.output_exprs.iter().any(|group| group.len() != 1)
            || output_exprs.len() != RECURSIVE_SHA_OUTPUT_LIMBS_V2
            || data.params.is_some()
        {
            return Err(CircuitBuilderError::NonPrimitiveOpArity {
                op: "RecursiveShaCompressionV2",
                expected: "48 inputs and 16 outputs".into(),
                got: output_exprs.len(),
            });
        }
        let inputs = data.input_exprs[0]
            .iter()
            .enumerate()
            .map(|(index, &expr)| {
                ctx.resolve_witness_id(expr, || {
                    format!("RecursiveShaCompressionV2 input limb {index}")
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut ordered_outputs = output_exprs.to_vec();
        ordered_outputs.sort_by_key(|(index, _)| *index);
        if ordered_outputs
            .iter()
            .enumerate()
            .any(|(index, (actual, _))| usize::try_from(*actual).ok() != Some(index))
        {
            return Err(CircuitBuilderError::NonPrimitiveOpArity {
                op: "RecursiveShaCompressionV2",
                expected: "ordered output indexes 0..16".into(),
                got: ordered_outputs.len(),
            });
        }
        let outputs = ordered_outputs
            .into_iter()
            .map(|(_, expression)| {
                ctx.resolve_witness_id(expression, || {
                    "RecursiveShaCompressionV2 output limb".into()
                })
                .map(|wid| vec![wid])
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Op::NonPrimitiveOpWithExecutor {
            inputs: vec![inputs],
            outputs,
            executor: Box::new(RecursiveShaExecutorV2 {
                op_type: recursive_sha_npo_type(),
            }),
            op_id: data.op_id,
        })
    }

    fn trace_generator(&self) -> TraceGeneratorFn<Plonky3ChallengeV2> {
        generate_recursive_sha_trace_v2
    }

    fn config(&self) -> NpoConfig {
        NpoConfig::new(RecursiveShaConfigV2)
    }
}

#[derive(Clone, Debug)]
struct RecursiveShaTraceV2 {
    operations: Vec<RecursiveShaOperationV2>,
}

impl NonPrimitiveTrace<Plonky3ChallengeV2> for RecursiveShaTraceV2 {
    fn op_type(&self) -> NpoTypeId {
        recursive_sha_npo_type()
    }

    fn rows(&self) -> usize {
        self.operations.len().saturating_mul(SHA_ROWS_V2)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed_clone(&self) -> Box<dyn NonPrimitiveTrace<Plonky3ChallengeV2>> {
        Box::new(self.clone())
    }
}

fn generate_recursive_sha_trace_v2(
    states: &OpStateMap,
) -> Result<Option<Box<dyn NonPrimitiveTrace<Plonky3ChallengeV2>>>, CircuitError> {
    let Some(state) = states
        .get(&recursive_sha_npo_type())
        .and_then(|state| state.downcast_ref::<RecursiveShaStateV2>())
    else {
        return Ok(None);
    };
    if state.operations.is_empty()
        || state.operations.iter().any(|operation| {
            operation.input_wids.len() != RECURSIVE_SHA_INPUT_LIMBS_V2
                || operation.output_wids.len() != RECURSIVE_SHA_OUTPUT_LIMBS_V2
                || operation.rows.len() != SHA_ROWS_V2
                || operation
                    .rows
                    .iter()
                    .any(|row| row.values.len() != ROW_FIELDS_V2)
        })
    {
        return Err(CircuitError::InvalidPreprocessedValues);
    }
    Ok(Some(Box::new(RecursiveShaTraceV2 {
        operations: state.operations.clone(),
    })))
}

fn placeholder_preprocessed(operations: &[RecursiveShaOperationV2]) -> Vec<KoalaBear> {
    let mut values =
        KoalaBear::zero_vec(operations.len() * SHA_ROWS_V2 * RECURSIVE_SHA_PREPROCESSED_WIDTH_V2);
    for (operation_index, operation) in operations.iter().enumerate() {
        let block_start = operation_index * SHA_ROWS_V2 * RECURSIVE_SHA_PREPROCESSED_WIDTH_V2;
        for round in 0..SHA_ROWS_V2 {
            values[block_start + round * RECURSIVE_SHA_PREPROCESSED_WIDTH_V2] = KoalaBear::ONE;
        }
        for (index, &wid) in operation.input_wids.iter().enumerate() {
            values[block_start + 1 + index] = wid.base_field_index::<KoalaBear, 4>();
        }
        let last = block_start + (SHA_ROWS_V2 - 1) * RECURSIVE_SHA_PREPROCESSED_WIDTH_V2;
        for (index, &wid) in operation.output_wids.iter().enumerate() {
            values[last + 1 + index] = wid.base_field_index::<KoalaBear, 4>();
        }
    }
    values
}

fn padded_trace_rows(trace: &RecursiveShaTraceV2, min_height: usize) -> Option<Vec<ShaRowV2>> {
    let real_rows = trace.rows();
    let padded_rows = real_rows.max(min_height).checked_next_power_of_two()?;
    if !padded_rows.is_multiple_of(SHA_ROWS_V2) {
        return None;
    }
    let mut rows = Vec::with_capacity(padded_rows);
    for operation in &trace.operations {
        rows.extend(operation.rows.iter().cloned());
    }
    if rows.len() < padded_rows {
        let (dummy, _) = compression_rows(z00z_crypto::SHA256_IV_V2, &[0_u8; 64]).ok()?;
        while rows.len() < padded_rows {
            rows.extend(dummy.iter().cloned());
        }
    }
    (rows.len() == padded_rows).then_some(rows)
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RecursiveShaProverV2;

impl RecursiveShaProverV2 {
    fn batch_instance(
        &self,
        packing: &TablePacking,
        traces: &Traces<Plonky3ChallengeV2>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces.non_primitive_trace::<RecursiveShaTraceV2>(&recursive_sha_npo_type())?;
        let rows = trace.rows();
        if rows == 0 || !rows.is_multiple_of(SHA_ROWS_V2) {
            return None;
        }
        let min_height = packing.min_trace_height();
        let padded = padded_trace_rows(trace, min_height)?;
        let preprocessed = placeholder_preprocessed(&trace.operations);
        let air = ShaAirV2::<KoalaBear, 4>::new(
            ShaAirRoleV2::RecursiveCompression,
            preprocessed,
            min_height,
        );
        Some(BatchTableInstance {
            op_type: recursive_sha_npo_type(),
            air: DynamicAirEntry::new(Box::new(air)),
            trace: ShaAirV2::<KoalaBear, 4>::trace_to_matrix(&padded, padded.len()),
            public_values: Vec::new(),
            rows,
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for RecursiveShaProverV2 {
    fn op_type(&self) -> NpoTypeId {
        recursive_sha_npo_type()
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
            || !entry.rows.is_multiple_of(SHA_ROWS_V2)
            || entry.lanes != 1
            || !entry.public_values.is_empty()
        {
            return Err("recursive SHA-256 table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(
            ShaAirV2::<KoalaBear, 4>::new(
                ShaAirRoleV2::RecursiveCompression,
                Vec::new(),
                SHA_ROWS_V2,
            ),
        )))
    }

    fn air_with_committed_preprocessed(
        &self,
        committed: Vec<KoalaBear>,
        min_height: usize,
        lanes: usize,
        circuit_extension_degree: u32,
    ) -> Option<DynamicAirEntry<Plonky3StarkConfigV2>> {
        let rows = committed.len() / RECURSIVE_SHA_PREPROCESSED_WIDTH_V2;
        (lanes == 1
            && circuit_extension_degree == 4
            && !committed.is_empty()
            && committed
                .len()
                .is_multiple_of(RECURSIVE_SHA_PREPROCESSED_WIDTH_V2)
            && rows.is_multiple_of(SHA_ROWS_V2))
        .then(|| {
            DynamicAirEntry::new(Box::new(ShaAirV2::<KoalaBear, 4>::new(
                ShaAirRoleV2::RecursiveCompression,
                committed,
                min_height,
            )))
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RecursiveShaPreprocessorV2;

impl NpoPreprocessor<KoalaBear> for RecursiveShaPreprocessorV2 {
    fn requires_runtime_circuit_metadata(&self) -> bool {
        false
    }

    fn preprocess(
        &self,
        _circuit: &dyn Any,
        preprocessed: &mut dyn Any,
    ) -> Result<p3_circuit::ops::NonPrimitivePreprocessedMap<KoalaBear>, CircuitError> {
        let mut result = p3_circuit::ops::NonPrimitivePreprocessedMap::new();
        let Some(preprocessed) =
            preprocessed.downcast_mut::<PreprocessedColumns<Plonky3ChallengeV2, 4>>()
        else {
            return Ok(result);
        };
        let op_type = recursive_sha_npo_type();
        let Some(raw) = take_non_primitive_base_columns(preprocessed, &op_type)? else {
            return Ok(result);
        };
        if raw.is_empty()
            || !raw
                .len()
                .is_multiple_of(RECURSIVE_SHA_RAW_PREPROCESSED_FIELDS_V2)
        {
            return Err(CircuitError::InvalidPreprocessedValues);
        }
        let operations = raw.len() / RECURSIVE_SHA_RAW_PREPROCESSED_FIELDS_V2;
        let mut committed =
            KoalaBear::zero_vec(operations * SHA_ROWS_V2 * RECURSIVE_SHA_PREPROCESSED_WIDTH_V2);
        let negative_one = KoalaBear::ZERO - KoalaBear::ONE;
        for (operation_index, operation) in raw
            .chunks_exact(RECURSIVE_SHA_RAW_PREPROCESSED_FIELDS_V2)
            .enumerate()
        {
            let block_start = operation_index * SHA_ROWS_V2 * RECURSIVE_SHA_PREPROCESSED_WIDTH_V2;
            for round in 0..SHA_ROWS_V2 {
                committed[block_start + round * RECURSIVE_SHA_PREPROCESSED_WIDTH_V2] =
                    KoalaBear::ONE;
            }
            committed[block_start + 1..block_start + 1 + RECURSIVE_SHA_INPUT_LIMBS_V2]
                .copy_from_slice(&operation[..RECURSIVE_SHA_INPUT_LIMBS_V2]);
            let last = block_start + (SHA_ROWS_V2 - 1) * RECURSIVE_SHA_PREPROCESSED_WIDTH_V2;
            for output in 0..RECURSIVE_SHA_OUTPUT_LIMBS_V2 {
                let raw_offset = RECURSIVE_SHA_INPUT_LIMBS_V2 + output * 2;
                let output_index = operation[raw_offset];
                if operation[raw_offset + 1] != KoalaBear::ONE {
                    return Err(CircuitError::InvalidPreprocessedValues);
                }
                let canonical_index = output_index.as_canonical_u64();
                if !canonical_index.is_multiple_of(4) {
                    return Err(CircuitError::InvalidPreprocessedValues);
                }
                let output_wid = usize::try_from(canonical_index / 4)
                    .map_err(|_| CircuitError::InvalidPreprocessedValues)?;
                let duplicate = preprocessed
                    .dup_npo_outputs
                    .get(&op_type)
                    .and_then(|outputs| outputs.get(output_wid).copied())
                    .unwrap_or(false);
                let multiplicity = if duplicate {
                    negative_one
                } else {
                    KoalaBear::from_u32(
                        preprocessed.ext_reads.get(output_wid).copied().unwrap_or(0),
                    )
                };
                committed[last + 1 + output] = output_index;
                committed[last + 1 + RECURSIVE_SHA_OUTPUT_LIMBS_V2 + output] = multiplicity;
            }
        }
        result.insert(op_type, committed);
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RecursiveShaAirBuilderV2;

impl NpoAirBuilder<Plonky3StarkConfigV2, 4> for RecursiveShaAirBuilderV2 {
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
        if op_type != &recursive_sha_npo_type()
            || lanes != 1
            || preprocessed.is_empty()
            || !preprocessed
                .len()
                .is_multiple_of(RECURSIVE_SHA_PREPROCESSED_WIDTH_V2)
        {
            return None;
        }
        let rows = preprocessed.len() / RECURSIVE_SHA_PREPROCESSED_WIDTH_V2;
        if !rows.is_multiple_of(SHA_ROWS_V2) {
            return None;
        }
        let committed = if retain_preprocessed_columns {
            preprocessed.clone()
        } else {
            core::mem::take(preprocessed)
        };
        let padded_rows = min_height.max(rows).checked_next_power_of_two()?;
        Some((
            z00z_plonky3_circuit_prover::common::CircuitTableAir::Dynamic(DynamicAirEntry::new(
                Box::new(ShaAirV2::<KoalaBear, 4>::new(
                    ShaAirRoleV2::RecursiveCompression,
                    committed,
                    min_height,
                )),
            )),
            log2_ceil_usize(padded_rows),
        ))
    }
}

pub(super) fn recursive_sha_npo_type() -> NpoTypeId {
    ShaAirRoleV2::RecursiveCompression.npo_type()
}

pub(super) fn register_recursive_sha_npo(circuit: &mut CircuitBuilder<Plonky3ChallengeV2>) {
    circuit.register_npo(RecursiveShaPluginV2);
}

pub(super) fn recursive_sha_compress(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    state: [ExprId; 16],
    block: [ExprId; 32],
) -> Result<[ExprId; 16], CircuitBuilderError> {
    let mut inputs = Vec::with_capacity(RECURSIVE_SHA_INPUT_LIMBS_V2);
    inputs.extend(state);
    inputs.extend(block);
    let (_, _, outputs) = circuit.push_non_primitive_op_with_outputs(
        recursive_sha_npo_type(),
        vec![inputs],
        vec![Some("recursive_sha_limb"); RECURSIVE_SHA_OUTPUT_LIMBS_V2],
        None,
        "recursive_sha_compression",
    );
    outputs
        .into_iter()
        .map(|output| output.ok_or(CircuitBuilderError::MissingOutput))
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|outputs: Vec<_>| CircuitBuilderError::NonPrimitiveOpArity {
            op: "RecursiveShaCompressionV2",
            expected: "sixteen outputs".into(),
            got: outputs.len(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recursive_sha_runner_binds_inputs_and_outputs() {
        let mut builder = CircuitBuilder::new();
        register_recursive_sha_npo(&mut builder);
        let inputs = builder.alloc_public_inputs(RECURSIVE_SHA_INPUT_LIMBS_V2, "sha inputs");
        let state: [ExprId; 16] = inputs[..16].try_into().unwrap();
        let block_limbs: [ExprId; 32] = inputs[16..].try_into().unwrap();
        let outputs = recursive_sha_compress(&mut builder, state, block_limbs).unwrap();
        let claimed = builder.alloc_public_inputs(RECURSIVE_SHA_OUTPUT_LIMBS_V2, "sha outputs");
        for (actual, expected) in outputs.into_iter().zip(claimed) {
            builder.connect(actual, expected);
        }
        let circuit = builder.build().unwrap();

        let input_state = z00z_crypto::SHA256_IV_V2;
        let block = core::array::from_fn::<_, 64, _>(|index| index as u8);
        let (_, output_state) = compression_rows(input_state, &block).unwrap();
        let mut public =
            Vec::with_capacity(RECURSIVE_SHA_INPUT_LIMBS_V2 + RECURSIVE_SHA_OUTPUT_LIMBS_V2);
        for word in input_state {
            public.push(embedded_base(word as u16));
            public.push(embedded_base((word >> 16) as u16));
        }
        for bytes in block.chunks_exact(4) {
            let word = u32::from_be_bytes(bytes.try_into().unwrap());
            public.push(embedded_base(word as u16));
            public.push(embedded_base((word >> 16) as u16));
        }
        for word in output_state {
            public.push(embedded_base(word as u16));
            public.push(embedded_base((word >> 16) as u16));
        }

        let mut runner = circuit.runner();
        runner.set_public_inputs(&public).unwrap();
        runner.run().unwrap();

        let last = public.len() - 1;
        public[last] += Plonky3ChallengeV2::ONE;
        let mut runner = circuit.runner();
        runner.set_public_inputs(&public).unwrap();
        assert!(runner.run().is_err());
    }
}
