//! Root-statement NPO and AIR for the checkpoint recursion circuit.

use core::{any::Any, fmt};

use p3_air::{Air, AirBuilder, BaseAir, WindowAccess};
use p3_circuit::ops::{
    ExecutionContext, NonPrimitiveExecutor, NpoConfig, NpoTypeId, Op, OpStateMap,
    PreprocessedWriter,
};
use p3_circuit::tables::{NonPrimitiveTrace, TraceGeneratorFn, Traces};
use p3_circuit::{
    CircuitBuilder, CircuitBuilderError, CircuitError, ExprId, NonPrimitiveOperationData,
    NpoCircuitPlugin, NpoLoweringContext, PreprocessedColumns, WitnessId,
};
use p3_field::{BasedVectorSpace, Field, PrimeCharacteristicRing};
use p3_koala_bear::KoalaBear;
use p3_lookup::{Count, InteractionBuilder};
use p3_matrix::dense::RowMajorMatrix;
use p3_util::log2_ceil_usize;
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchAir, BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking,
    TableProver,
};
use z00z_plonky3_circuit_prover::common::take_non_primitive_base_columns;

use super::plonky3_root_statement::{ROOT_STATEMENT_FIELDS_V2, ROOT_STATEMENT_NPO_ID_V2};
use super::{Plonky3ChallengeV2, Plonky3StarkConfigV2};

#[derive(Clone, Debug)]
struct RootStatementRowV2<F> {
    input_wids: Vec<WitnessId>,
    values: Vec<F>,
}

#[derive(Debug, Default)]
struct RootStatementStateV2<F> {
    rows: Vec<RootStatementRowV2<F>>,
}

#[derive(Clone)]
struct RootStatementExecutorV2 {
    op_type: NpoTypeId,
}

impl fmt::Debug for RootStatementExecutorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RootStatementExecutorV2")
            .field("op_type", &self.op_type)
            .finish()
    }
}

impl<F> NonPrimitiveExecutor<F> for RootStatementExecutorV2
where
    F: Field + Send + Sync + 'static,
{
    fn execute(
        &self,
        inputs: &[Vec<WitnessId>],
        outputs: &[Vec<WitnessId>],
        ctx: &mut ExecutionContext<'_, F>,
    ) -> Result<(), CircuitError> {
        if inputs.len() != 1 || inputs[0].len() != ROOT_STATEMENT_FIELDS_V2 || !outputs.is_empty() {
            return Err(CircuitError::NonPrimitiveOpLayoutMismatch {
                op: self.op_type.clone(),
                expected: format!(
                    "one {}-field input group and no outputs",
                    ROOT_STATEMENT_FIELDS_V2
                ),
                got: inputs.len(),
            });
        }
        let values = inputs[0]
            .iter()
            .map(|&wid| ctx.get_witness(wid))
            .collect::<Result<Vec<_>, _>>()?;
        ctx.get_op_state_mut::<RootStatementStateV2<F>>(&self.op_type)
            .rows
            .push(RootStatementRowV2 {
                input_wids: inputs[0].clone(),
                values,
            });
        Ok(())
    }

    fn op_type(&self) -> &NpoTypeId {
        &self.op_type
    }

    fn preprocess(
        &self,
        inputs: &[Vec<WitnessId>],
        _outputs: &[Vec<WitnessId>],
        preprocessed: &mut dyn PreprocessedWriter<F>,
    ) -> Result<(), CircuitError> {
        preprocessed.register_non_primitive_witness_reads(&self.op_type, &inputs[0])?;
        preprocessed.register_non_primitive_preprocessed_no_read(&self.op_type, &[F::ONE]);
        Ok(())
    }

    fn num_exposed_outputs(&self) -> Option<usize> {
        Some(0)
    }

    fn boxed(&self) -> Box<dyn NonPrimitiveExecutor<F>> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Copy, Debug)]
struct RootStatementConfigV2;

#[derive(Clone, Copy, Debug)]
struct RootStatementPluginV2;

impl NpoCircuitPlugin<Plonky3ChallengeV2> for RootStatementPluginV2 {
    fn type_id(&self) -> NpoTypeId {
        root_statement_npo_type()
    }

    fn lower(
        &self,
        data: &NonPrimitiveOperationData<Plonky3ChallengeV2>,
        output_exprs: &[(u32, ExprId)],
        ctx: &mut NpoLoweringContext<'_, Plonky3ChallengeV2>,
    ) -> Result<Op<Plonky3ChallengeV2>, CircuitBuilderError> {
        if data.input_exprs.len() != 1
            || data.input_exprs[0].len() != ROOT_STATEMENT_FIELDS_V2
            || !data.output_exprs.is_empty()
            || !output_exprs.is_empty()
            || data.params.is_some()
        {
            return Err(CircuitBuilderError::NonPrimitiveOpArity {
                op: "RootStatementV2",
                expected: format!(
                    "one {}-field input group and no outputs",
                    ROOT_STATEMENT_FIELDS_V2
                ),
                got: data.input_exprs.len(),
            });
        }
        let inputs = data.input_exprs[0]
            .iter()
            .enumerate()
            .map(|(index, &expr)| {
                ctx.resolve_witness_id(expr, || format!("RootStatementV2 input field {index}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Op::NonPrimitiveOpWithExecutor {
            inputs: vec![inputs],
            outputs: Vec::new(),
            executor: Box::new(RootStatementExecutorV2 {
                op_type: root_statement_npo_type(),
            }),
            op_id: data.op_id,
        })
    }

    fn trace_generator(&self) -> TraceGeneratorFn<Plonky3ChallengeV2> {
        generate_root_statement_trace_v2
    }

    fn config(&self) -> NpoConfig {
        NpoConfig::new(RootStatementConfigV2)
    }
}

#[derive(Clone, Debug)]
struct RootStatementTraceV2<F> {
    rows: Vec<RootStatementRowV2<F>>,
}

impl<TraceF, CircuitF> NonPrimitiveTrace<CircuitF> for RootStatementTraceV2<TraceF>
where
    TraceF: Clone + Send + Sync + 'static,
{
    fn op_type(&self) -> NpoTypeId {
        root_statement_npo_type()
    }

    fn rows(&self) -> usize {
        self.rows.len()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed_clone(&self) -> Box<dyn NonPrimitiveTrace<CircuitF>> {
        Box::new(self.clone())
    }
}

fn generate_root_statement_trace_v2(
    states: &OpStateMap,
) -> Result<Option<Box<dyn NonPrimitiveTrace<Plonky3ChallengeV2>>>, CircuitError> {
    let Some(state) = states
        .get(&root_statement_npo_type())
        .and_then(|state| state.downcast_ref::<RootStatementStateV2<Plonky3ChallengeV2>>())
    else {
        return Ok(None);
    };
    if state.rows.len() != 1 {
        return Err(CircuitError::InvalidPreprocessedValues);
    }
    let mut rows = Vec::with_capacity(1);
    for row in &state.rows {
        if row.values.len() != ROOT_STATEMENT_FIELDS_V2 {
            return Err(CircuitError::InvalidPreprocessedValues);
        }
        let mut values: Vec<KoalaBear> = Vec::with_capacity(ROOT_STATEMENT_FIELDS_V2);
        for value in &row.values {
            let coefficients =
                <Plonky3ChallengeV2 as BasedVectorSpace<KoalaBear>>::as_basis_coefficients_slice(
                    value,
                );
            if coefficients.len() != 4
                || coefficients[1..]
                    .iter()
                    .any(|&value| value != KoalaBear::ZERO)
            {
                return Err(CircuitError::InvalidPreprocessedValues);
            }
            values.push(coefficients[0]);
        }
        rows.push(RootStatementRowV2 {
            input_wids: row.input_wids.clone(),
            values,
        });
    }
    Ok(Some(Box::new(RootStatementTraceV2 { rows })))
}

#[derive(Clone, Debug)]
struct RootStatementAirV2<F, const D: usize> {
    preprocessed: Vec<F>,
    min_height: usize,
}

impl<F: Field, const D: usize> RootStatementAirV2<F, D> {
    const fn width_v2() -> usize {
        ROOT_STATEMENT_FIELDS_V2 * D
    }

    const fn preprocessed_width_v2() -> usize {
        ROOT_STATEMENT_FIELDS_V2 + 1
    }

    fn new(preprocessed: Vec<F>, min_height: usize) -> Self {
        Self {
            preprocessed,
            min_height: min_height.max(1),
        }
    }

    fn trace_to_matrix(row: &RootStatementRowV2<F>, min_height: usize) -> RowMajorMatrix<F> {
        let mut values = F::zero_vec(Self::width_v2());
        for (index, value) in row.values.iter().copied().enumerate() {
            values[index * D] = value;
        }
        let mut matrix = RowMajorMatrix::new(values, Self::width_v2());
        matrix.pad_to_min_power_of_two_height(min_height.max(1), F::ZERO);
        matrix
    }
}

impl<F: Field, const D: usize> BaseAir<F> for RootStatementAirV2<F, D> {
    fn width(&self) -> usize {
        Self::width_v2()
    }

    fn num_public_values(&self) -> usize {
        ROOT_STATEMENT_FIELDS_V2
    }

    fn preprocessed_width(&self) -> usize {
        Self::preprocessed_width_v2()
    }

    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
        let mut matrix = RowMajorMatrix::from_flat_padded(
            self.preprocessed.clone(),
            Self::preprocessed_width_v2(),
            F::ZERO,
        );
        matrix.pad_to_min_power_of_two_height(self.min_height, F::ZERO);
        Some(matrix)
    }

    fn main_next_row_columns(&self) -> Vec<usize> {
        Vec::new()
    }

    fn preprocessed_next_row_columns(&self) -> Vec<usize> {
        Vec::new()
    }
}

impl<AB, const D: usize> Air<AB> for RootStatementAirV2<AB::F, D>
where
    AB: AirBuilder + InteractionBuilder,
    AB::F: Field,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let main = main.current_slice();
        let preprocessed = builder.preprocessed().clone();
        let preprocessed = preprocessed.current_slice();
        let public = builder
            .public_values()
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<AB::Expr>>();
        let active: AB::Expr = preprocessed[ROOT_STATEMENT_FIELDS_V2].into();

        for index in 0..ROOT_STATEMENT_FIELDS_V2 {
            let offset = index * D;
            {
                let mut when_active = builder.when(active.clone());
                when_active.assert_eq(main[offset], public[index].clone());
                for coefficient in 1..D {
                    when_active.assert_zero(main[offset + coefficient]);
                }
            }
            let mut fields = Vec::with_capacity(D + 1);
            fields.push(preprocessed[index].into());
            for coefficient in 0..D {
                fields.push(main[offset + coefficient].into());
            }
            builder.push_interaction("WitnessChecks", fields, Count::bounded(-active.clone(), 1));
        }
    }
}

impl BatchAir<Plonky3StarkConfigV2> for RootStatementAirV2<KoalaBear, 4> {}

#[derive(Clone, Copy, Debug)]
pub(super) struct RootStatementProverV2;

impl RootStatementProverV2 {
    fn batch_instance(
        &self,
        packing: &TablePacking,
        traces: &Traces<Plonky3ChallengeV2>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces
            .non_primitive_trace::<RootStatementTraceV2<KoalaBear>>(&root_statement_npo_type())?;
        let row = trace.rows.first()?;
        if trace.rows.len() != 1
            || row.input_wids.len() != ROOT_STATEMENT_FIELDS_V2
            || row.values.len() != ROOT_STATEMENT_FIELDS_V2
        {
            return None;
        }
        let mut preprocessed = Vec::with_capacity(ROOT_STATEMENT_FIELDS_V2 + 1);
        preprocessed.extend(
            row.input_wids
                .iter()
                .map(|&wid| wid.base_field_index::<KoalaBear, 4>()),
        );
        preprocessed.push(KoalaBear::ONE);
        let min_height = packing.min_trace_height();
        let air = RootStatementAirV2::<KoalaBear, 4>::new(preprocessed, min_height);
        Some(BatchTableInstance {
            op_type: root_statement_npo_type(),
            air: DynamicAirEntry::new(Box::new(air)),
            trace: RootStatementAirV2::<KoalaBear, 4>::trace_to_matrix(row, min_height),
            public_values: row.values.clone(),
            rows: 1,
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for RootStatementProverV2 {
    fn op_type(&self) -> NpoTypeId {
        root_statement_npo_type()
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
            || entry.rows != 1
            || entry.lanes != 1
            || entry.public_values.len() != ROOT_STATEMENT_FIELDS_V2
        {
            return Err("root-statement table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(RootStatementAirV2::<
            KoalaBear,
            4,
        >::new(Vec::new(), 1))))
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
            && committed.len().is_multiple_of(ROOT_STATEMENT_FIELDS_V2 + 1))
        .then(|| {
            DynamicAirEntry::new(Box::new(RootStatementAirV2::<KoalaBear, 4>::new(
                committed, min_height,
            )))
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RootStatementPreprocessorV2;

impl z00z_plonky3_circuit_prover::common::NpoPreprocessor<KoalaBear>
    for RootStatementPreprocessorV2
{
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
        let Some(values) =
            take_non_primitive_base_columns(preprocessed, &root_statement_npo_type())?
        else {
            return Ok(result);
        };
        if values.len() != ROOT_STATEMENT_FIELDS_V2 + 1 {
            return Err(CircuitError::InvalidPreprocessedValues);
        }
        result.insert(root_statement_npo_type(), values);
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RootStatementAirBuilderV2;

impl z00z_plonky3_circuit_prover::common::NpoAirBuilder<Plonky3StarkConfigV2, 4>
    for RootStatementAirBuilderV2
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
        if op_type != &root_statement_npo_type()
            || lanes != 1
            || preprocessed.len() != ROOT_STATEMENT_FIELDS_V2 + 1
        {
            return None;
        }
        let committed = if retain_preprocessed_columns {
            preprocessed.clone()
        } else {
            core::mem::take(preprocessed)
        };
        let padded_rows = min_height.max(1).next_power_of_two();
        Some((
            z00z_plonky3_circuit_prover::common::CircuitTableAir::Dynamic(DynamicAirEntry::new(
                Box::new(RootStatementAirV2::<KoalaBear, 4>::new(
                    committed, min_height,
                )),
            )),
            log2_ceil_usize(padded_rows),
        ))
    }
}

pub(super) fn root_statement_npo_type() -> NpoTypeId {
    NpoTypeId::new(ROOT_STATEMENT_NPO_ID_V2)
}

pub(super) fn register_root_statement_npo(circuit: &mut CircuitBuilder<Plonky3ChallengeV2>) {
    circuit.register_npo(RootStatementPluginV2);
}

pub(super) fn bind_root_statement_targets(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    targets: &[ExprId],
) -> Result<(), CircuitBuilderError> {
    if targets.len() != ROOT_STATEMENT_FIELDS_V2 {
        return Err(CircuitBuilderError::NonPrimitiveOpArity {
            op: "RootStatementV2",
            expected: format!("{} fields", ROOT_STATEMENT_FIELDS_V2),
            got: targets.len(),
        });
    }
    circuit.push_non_primitive_op_with_outputs(
        root_statement_npo_type(),
        vec![targets.to_vec()],
        Vec::new(),
        None,
        "root_statement",
    );
    Ok(())
}
