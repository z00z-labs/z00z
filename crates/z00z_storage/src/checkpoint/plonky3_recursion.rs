//! Generation-bound PCS adapter and public-statement accumulator for binary W32 recursion.

use core::{any::Any, fmt};

use p3_air::{Air, AirBuilder, BaseAir, WindowAccess};
use p3_circuit::ops::{
    ExecutionContext, NonPrimitiveExecutor, NpoConfig, NpoTypeId, Op, OpStateMap,
    PreprocessedWriter,
};
use p3_circuit::tables::{NonPrimitiveTrace, TraceGeneratorFn, Traces};
use p3_circuit::{CircuitBuilder, CircuitBuilderError, NonPrimitiveOpId};
use p3_circuit::{
    CircuitError, ExprId, NonPrimitiveOperationData, NpoCircuitPlugin, NpoLoweringContext,
    PreprocessedColumns, WitnessId,
};
use p3_circuit_prover::batch_stark_prover::{
    BatchAir, BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking,
    TableProver,
};
use p3_commit::{
    BuildPeriodicLdeTableFast, Mmcs, OpenedValues, Pcs, PeriodicLdeTable, PolynomialSpace,
};
use p3_field::coset::TwoAdicMultiplicativeCoset;
use p3_field::{BasedVectorSpace, ExtensionField, Field, PrimeCharacteristicRing, TwoAdicField};
use p3_fri::{FriParameters, TwoAdicFriPcs};
use p3_koala_bear::KoalaBear;
use p3_lookup::{Count, InteractionBuilder};
use p3_matrix::dense::RowMajorMatrix;
use p3_recursion::challenger_perm::ChallengerPermConfig;
use p3_recursion::pcs::{
    verify_fri_circuit, FriProofTargets, InputProofTargets, MerkleCapTargets, MmcsProofTargets,
    RecExtensionValMmcs, RecValMmcs, Witness,
};
use p3_recursion::traits::ComsWithOpeningsTargets;
use p3_recursion::types::{OpenedValuesTargetsWithLookups, RecursiveLagrangeSelectors};
use p3_recursion::{
    CircuitChallenger, GenerationError, ObservableCommitment, PcsGeneration, Recursive,
    RecursiveChallenger, RecursiveMmcs, RecursivePcs, Target, VerificationError,
};
use p3_util::log2_ceil_usize;

use super::{
    plonky3_binary_mmcs::verify_binary_paths, Plonky3ChallengeMmcsV2, Plonky3ChallengeV2,
    Plonky3ChallengerV2, Plonky3CompressionV2, Plonky3HashV2, Plonky3InnerPcsV2,
    Plonky3StarkConfigV2, Plonky3ValueMmcsV2, PLONKY3_MMCS_DIGEST_ELEMS_V2,
};

type DomainV2 = TwoAdicMultiplicativeCoset<KoalaBear>;
type CommitmentV2 = MerkleCapTargets<KoalaBear, PLONKY3_MMCS_DIGEST_ELEMS_V2>;
type NativeProofV2 = <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::Proof;
type NativeCommitmentV2 =
    <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::Commitment;
type NativeDataV2 = <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::ProverData;
type NativeComsV2 = [(
    NativeCommitmentV2,
    Vec<(DomainV2, Vec<(Plonky3ChallengeV2, Vec<Plonky3ChallengeV2>)>)>,
)];
type UpstreamRecMmcsV2 =
    RecValMmcs<KoalaBear, PLONKY3_MMCS_DIGEST_ELEMS_V2, Plonky3HashV2, Plonky3CompressionV2>;
type UpstreamInputProofV2 = InputProofTargets<KoalaBear, Plonky3ChallengeV2, UpstreamRecMmcsV2>;
type UpstreamOpeningProofV2 = FriProofTargets<
    KoalaBear,
    Plonky3ChallengeV2,
    RecExtensionValMmcs<
        KoalaBear,
        Plonky3ChallengeV2,
        PLONKY3_MMCS_DIGEST_ELEMS_V2,
        UpstreamRecMmcsV2,
    >,
    UpstreamInputProofV2,
    Witness<KoalaBear>,
>;

const ROOT_STATEMENT_NPO_ID_V2: &str = "z00z/plonky3/root-statement/v2";
const ROOT_STATEMENT_DIGEST_LIMBS_V2: usize = 16;
const ROOT_STATEMENT_DIGEST_COUNT_V2: usize = 5;
pub(super) const ROOT_STATEMENT_COMMITMENT_FIELDS_V2: usize = 8;
pub(super) const ROOT_STATEMENT_COMMITMENT_INDEX_V2: usize =
    1 + ROOT_STATEMENT_DIGEST_LIMBS_V2 * ROOT_STATEMENT_DIGEST_COUNT_V2;
pub(super) const ROOT_STATEMENT_FIELDS_V2: usize =
    ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2 + 4;
pub(super) const ROOT_STATEMENT_REPLICA_INDEX_V2: usize =
    ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2;
pub(super) const ROOT_STATEMENT_START_INDEX_V2: usize = ROOT_STATEMENT_REPLICA_INDEX_V2 + 1;
pub(super) const ROOT_STATEMENT_COUNT_INDEX_V2: usize = ROOT_STATEMENT_START_INDEX_V2 + 1;
pub(super) const ROOT_STATEMENT_TOTAL_INDEX_V2: usize = ROOT_STATEMENT_COUNT_INDEX_V2 + 1;

/// Fixed public statement propagated by every recursive layer.
///
/// Digests are encoded as sixteen little-endian `u16` limbs so every value is
/// canonical in KoalaBear. Eight native KoalaBear fields bind the ordered
/// leaf/subtree commitment. The final four fields bind a physical-replica or
/// replica-fold ordinal and an exact contiguous leaf range; aggregation can
/// therefore neither duplicate nor omit a bounded domain proof or one of the
/// three security replicas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RootStatementV2 {
    values: [KoalaBear; ROOT_STATEMENT_FIELDS_V2],
}

impl RootStatementV2 {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn leaf(
        statement_digest: [u8; 32],
        leaf_manifest_digest: [u8; 32],
        parameter_digest: [u8; 32],
        security_digest: [u8; 32],
        verifier_bundle_digest: [u8; 32],
        commitment: [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
        replica: u8,
        start: u16,
        total: u16,
    ) -> Result<Self, CircuitError> {
        if [
            statement_digest,
            leaf_manifest_digest,
            parameter_digest,
            security_digest,
            verifier_bundle_digest,
        ]
        .contains(&[0; 32])
            || total == 0
            || start >= total
        {
            return Err(CircuitError::InvalidPreprocessedValues);
        }
        let mut values = [KoalaBear::ZERO; ROOT_STATEMENT_FIELDS_V2];
        values[0] = KoalaBear::from_u8(2);
        let mut cursor = 1;
        for digest in [
            statement_digest,
            leaf_manifest_digest,
            parameter_digest,
            security_digest,
            verifier_bundle_digest,
        ] {
            for limb in digest.chunks_exact(2) {
                values[cursor] = KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]));
                cursor += 1;
            }
        }
        values[ROOT_STATEMENT_COMMITMENT_INDEX_V2
            ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
            .copy_from_slice(&commitment);
        values[ROOT_STATEMENT_REPLICA_INDEX_V2] = KoalaBear::from_u8(replica);
        values[ROOT_STATEMENT_START_INDEX_V2] = KoalaBear::from_u16(start);
        values[ROOT_STATEMENT_COUNT_INDEX_V2] = KoalaBear::ONE;
        values[ROOT_STATEMENT_TOTAL_INDEX_V2] = KoalaBear::from_u16(total);
        Ok(Self { values })
    }

    pub(super) fn root(
        &self,
        commitment: [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
    ) -> Self {
        let mut root = self.clone();
        root.values[ROOT_STATEMENT_COMMITMENT_INDEX_V2
            ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
            .copy_from_slice(&commitment);
        root.values[ROOT_STATEMENT_START_INDEX_V2] = KoalaBear::ZERO;
        root.values[ROOT_STATEMENT_COUNT_INDEX_V2] = root.values[ROOT_STATEMENT_TOTAL_INDEX_V2];
        root
    }

    /// Convert a complete physical-replica root into one ordered replica-fold
    /// root. Callers must first prove the exact input ordinal pair and derive
    /// `commitment` through the corresponding domain-separated fold hash.
    pub(super) fn replica_fold_root(
        &self,
        commitment: [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
        fold_ordinal: u8,
    ) -> Result<Self, CircuitError> {
        if self.values[ROOT_STATEMENT_START_INDEX_V2] != KoalaBear::ZERO
            || self.values[ROOT_STATEMENT_COUNT_INDEX_V2]
                != self.values[ROOT_STATEMENT_TOTAL_INDEX_V2]
        {
            return Err(CircuitError::InvalidPreprocessedValues);
        }
        let mut root = self.clone();
        root.values[ROOT_STATEMENT_COMMITMENT_INDEX_V2
            ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
            .copy_from_slice(&commitment);
        root.values[ROOT_STATEMENT_REPLICA_INDEX_V2] = KoalaBear::from_u8(fold_ordinal);
        Ok(root)
    }

    #[must_use]
    pub(super) fn commitment(&self) -> [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2] {
        self.values[ROOT_STATEMENT_COMMITMENT_INDEX_V2
            ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
            .try_into()
            .expect("fixed root-statement commitment width")
    }

    #[must_use]
    pub(super) const fn values(&self) -> &[KoalaBear; ROOT_STATEMENT_FIELDS_V2] {
        &self.values
    }
}

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

impl p3_circuit_prover::common::NpoPreprocessor<KoalaBear> for RootStatementPreprocessorV2 {
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
        let Some(values) = preprocessed.non_primitive.get(&root_statement_npo_type()) else {
            return Ok(result);
        };
        let values = values
            .iter()
            .map(|value| {
                value
                    .as_base()
                    .ok_or(CircuitError::InvalidPreprocessedValues)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if values.len() != ROOT_STATEMENT_FIELDS_V2 + 1 {
            return Err(CircuitError::InvalidPreprocessedValues);
        }
        result.insert(root_statement_npo_type(), values);
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RootStatementAirBuilderV2;

impl p3_circuit_prover::common::NpoAirBuilder<Plonky3StarkConfigV2, 4>
    for RootStatementAirBuilderV2
{
    fn try_build(
        &self,
        op_type: &NpoTypeId,
        preprocessed: &[KoalaBear],
        min_height: usize,
        lanes: usize,
        _constraint_profile: p3_circuit_prover::ConstraintProfile,
    ) -> Option<(
        p3_circuit_prover::common::CircuitTableAir<Plonky3StarkConfigV2, 4>,
        usize,
    )> {
        if op_type != &root_statement_npo_type()
            || lanes != 1
            || preprocessed.len() != ROOT_STATEMENT_FIELDS_V2 + 1
        {
            return None;
        }
        let padded_rows = min_height.max(1).next_power_of_two();
        Some((
            p3_circuit_prover::common::CircuitTableAir::Dynamic(DynamicAirEntry::new(Box::new(
                RootStatementAirV2::<KoalaBear, 4>::new(preprocessed.to_vec(), min_height),
            ))),
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

pub(super) struct BinaryProofTargetsV2 {
    siblings: Vec<[Target; 4]>,
}

impl BinaryProofTargetsV2 {
    pub(super) fn siblings(&self) -> &[[Target; 4]] {
        &self.siblings
    }
}

impl Recursive<Plonky3ChallengeV2> for BinaryProofTargetsV2 {
    type Input = <Plonky3ValueMmcsV2 as Mmcs<KoalaBear>>::Proof;

    fn new(circuit: &mut CircuitBuilder<Plonky3ChallengeV2>, input: &Self::Input) -> Self {
        let siblings = input
            .iter()
            .map(|_| circuit.alloc_private_input_array("binary W32 MMCS sibling"))
            .collect();
        Self { siblings }
    }

    fn get_values(_input: &Self::Input) -> Vec<Plonky3ChallengeV2> {
        Vec::new()
    }

    fn get_private_values(input: &Self::Input) -> Vec<Plonky3ChallengeV2> {
        input
            .iter()
            .flat_map(|digest| {
                digest
                    .chunks_exact(4)
                    .map(|chunk| Plonky3ChallengeV2::new([chunk[0], chunk[1], chunk[2], chunk[3]]))
            })
            .collect()
    }
}

impl MmcsProofTargets for BinaryProofTargetsV2 {
    fn salt_targets(&self) -> &[Vec<Target>] {
        &[]
    }
}

pub(super) struct BinaryRecMmcsV2;

impl RecursiveMmcs<KoalaBear, Plonky3ChallengeV2> for BinaryRecMmcsV2 {
    type Input = Plonky3ValueMmcsV2;
    type Commitment = CommitmentV2;
    type Proof = BinaryProofTargetsV2;
}

type BinaryExtMmcsV2 = RecExtensionValMmcs<
    KoalaBear,
    Plonky3ChallengeV2,
    PLONKY3_MMCS_DIGEST_ELEMS_V2,
    BinaryRecMmcsV2,
>;
type BinaryInputProofV2 = InputProofTargets<KoalaBear, Plonky3ChallengeV2, BinaryRecMmcsV2>;
type BinaryOpeningProofV2 = FriProofTargets<
    KoalaBear,
    Plonky3ChallengeV2,
    BinaryExtMmcsV2,
    BinaryInputProofV2,
    Witness<KoalaBear>,
>;

#[derive(Clone)]
pub(super) struct Plonky3PcsV2 {
    inner: Plonky3InnerPcsV2,
}

impl Plonky3PcsV2 {
    pub(super) fn new(
        dft: p3_dft::Radix2DitParallel<KoalaBear>,
        value_mmcs: Plonky3ValueMmcsV2,
        fri: FriParameters<Plonky3ChallengeMmcsV2>,
    ) -> Self {
        Self {
            inner: TwoAdicFriPcs::new(dft, value_mmcs, fri),
        }
    }
}

impl BuildPeriodicLdeTableFast for Plonky3PcsV2 {
    type PeriodicDomain = DomainV2;

    fn maybe_build_periodic_lde_table_fast(
        &self,
        periodic_cols: &[Vec<KoalaBear>],
        trace_domain: DomainV2,
        quotient_domain: DomainV2,
    ) -> Option<PeriodicLdeTable<KoalaBear>> {
        self.inner
            .maybe_build_periodic_lde_table_fast(periodic_cols, trace_domain, quotient_domain)
    }
}

impl Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2> for Plonky3PcsV2 {
    type Domain = DomainV2;
    type Commitment = NativeCommitmentV2;
    type ProverData = NativeDataV2;
    type EvaluationsOnDomain<'a> = <Plonky3InnerPcsV2 as Pcs<
        Plonky3ChallengeV2,
        Plonky3ChallengerV2,
    >>::EvaluationsOnDomain<'a>;
    type Proof = NativeProofV2;
    type Error = <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::Error;

    const ZK: bool = <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::ZK;

    fn natural_domain_for_degree(&self, degree: usize) -> Self::Domain {
        <Plonky3InnerPcsV2 as Pcs<
            Plonky3ChallengeV2,
            Plonky3ChallengerV2,
        >>::natural_domain_for_degree(&self.inner, degree)
    }

    fn log_max_lde_height(&self) -> usize {
        <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::log_max_lde_height(
            &self.inner,
        )
    }

    fn commit(
        &self,
        evaluations: impl IntoIterator<Item = (Self::Domain, RowMajorMatrix<KoalaBear>)>,
    ) -> (Self::Commitment, Self::ProverData) {
        <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::commit(
            &self.inner,
            evaluations,
        )
    }

    fn get_quotient_ldes(
        &self,
        evaluations: impl IntoIterator<Item = (Self::Domain, RowMajorMatrix<KoalaBear>)>,
        num_chunks: usize,
    ) -> Vec<RowMajorMatrix<KoalaBear>> {
        <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::get_quotient_ldes(
            &self.inner,
            evaluations,
            num_chunks,
        )
    }

    fn commit_ldes(
        &self,
        ldes: Vec<RowMajorMatrix<KoalaBear>>,
    ) -> (Self::Commitment, Self::ProverData) {
        <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::commit_ldes(
            &self.inner,
            ldes,
        )
    }

    fn get_evaluations_on_domain<'a>(
        &self,
        prover_data: &'a Self::ProverData,
        index: usize,
        domain: Self::Domain,
    ) -> Self::EvaluationsOnDomain<'a> {
        <Plonky3InnerPcsV2 as Pcs<
            Plonky3ChallengeV2,
            Plonky3ChallengerV2,
        >>::get_evaluations_on_domain(&self.inner, prover_data, index, domain)
    }

    fn open(
        &self,
        data_with_points: Vec<(&Self::ProverData, Vec<Vec<Plonky3ChallengeV2>>)>,
        challenger: &mut Plonky3ChallengerV2,
    ) -> (OpenedValues<Plonky3ChallengeV2>, Self::Proof) {
        <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::open(
            &self.inner,
            data_with_points,
            challenger,
        )
    }

    fn verify(
        &self,
        openings: Vec<(
            Self::Commitment,
            Vec<(
                Self::Domain,
                Vec<(Plonky3ChallengeV2, Vec<Plonky3ChallengeV2>)>,
            )>,
        )>,
        proof: &Self::Proof,
        challenger: &mut Plonky3ChallengerV2,
    ) -> Result<(), Self::Error> {
        <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::verify(
            &self.inner,
            openings,
            proof,
            challenger,
        )
    }
}

impl PcsGeneration<Plonky3StarkConfigV2, NativeProofV2> for Plonky3PcsV2 {
    fn generate_challenges(
        &self,
        config: &Plonky3StarkConfigV2,
        challenger: &mut Plonky3ChallengerV2,
        openings: &NativeComsV2,
        proof: &NativeProofV2,
        extra_params: Option<&[usize]>,
    ) -> Result<Vec<Plonky3ChallengeV2>, GenerationError> {
        <Plonky3InnerPcsV2 as PcsGeneration<Plonky3StarkConfigV2, NativeProofV2>>::generate_challenges(
            &self.inner,
            config,
            challenger,
            openings,
            proof,
            extra_params,
        )
    }

    fn num_challenges(
        proof: &NativeProofV2,
        extra_params: Option<&[usize]>,
    ) -> Result<usize, GenerationError> {
        <Plonky3InnerPcsV2 as PcsGeneration<Plonky3StarkConfigV2, NativeProofV2>>::num_challenges(
            proof,
            extra_params,
        )
    }
}

impl
    RecursivePcs<
        Plonky3StarkConfigV2,
        BinaryInputProofV2,
        BinaryOpeningProofV2,
        CommitmentV2,
        DomainV2,
    > for Plonky3PcsV2
{
    type VerifierParams = p3_recursion::FriVerifierParams;
    type RecursiveProof = BinaryOpeningProofV2;

    fn get_challenges_circuit<const WIDTH: usize, const RATE: usize, C: ChallengerPermConfig>(
        circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
        challenger: &mut CircuitChallenger<WIDTH, RATE, C>,
        proof: &BinaryOpeningProofV2,
        _opened_values: &OpenedValuesTargetsWithLookups<Plonky3StarkConfigV2>,
        params: &Self::VerifierParams,
    ) -> Result<Vec<Target>, CircuitBuilderError> {
        let alpha = <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
            KoalaBear,
            Plonky3ChallengeV2,
        >>::sample_ext(challenger, circuit);
        let mut betas = Vec::with_capacity(proof.commit_phase_commits.len());
        for (commitment, witness) in proof
            .commit_phase_commits
            .iter()
            .zip(&proof.commit_pow_witnesses)
        {
            <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
                KoalaBear,
                Plonky3ChallengeV2,
            >>::observe_slice(challenger, circuit, &commitment.to_observation_targets());
            <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
                KoalaBear,
                Plonky3ChallengeV2,
            >>::check_pow_witness(
                challenger, circuit, params.commit_pow_bits, witness.witness
            )?;
            betas.push(<CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
                KoalaBear,
                Plonky3ChallengeV2,
            >>::sample_ext(challenger, circuit));
        }
        <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
            KoalaBear,
            Plonky3ChallengeV2,
        >>::observe_ext_slice(challenger, circuit, &proof.final_poly);
        for log_arity in &proof.log_arities {
            let target =
                circuit.alloc_const(Plonky3ChallengeV2::from_usize(*log_arity), "FRI log arity");
            <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
                KoalaBear,
                Plonky3ChallengeV2,
            >>::observe(challenger, circuit, target);
        }
        <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
            KoalaBear,
            Plonky3ChallengeV2,
        >>::check_pow_witness(
            challenger,
            circuit,
            params.query_pow_bits,
            proof.pow_witness.witness,
        )?;
        let mut challenges = Vec::with_capacity(1 + betas.len());
        challenges.push(alpha);
        challenges.extend(betas);
        Ok(challenges)
    }

    fn verify_circuit<const WIDTH: usize, const RATE: usize, C: ChallengerPermConfig>(
        &self,
        circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
        challenges: &[Target],
        challenger: &mut CircuitChallenger<WIDTH, RATE, C>,
        openings: &ComsWithOpeningsTargets<CommitmentV2, DomainV2>,
        proof: &BinaryOpeningProofV2,
        params: &Self::VerifierParams,
    ) -> Result<Vec<NonPrimitiveOpId>, VerificationError> {
        let num_betas = proof.commit_phase_commits.len();
        if challenges.len() != num_betas + 1 || proof.query_proofs.len() != params.num_queries {
            return Err(shape("binary W32 recursive FRI challenge shape mismatch"));
        }
        let total_reduction = proof
            .log_arities
            .iter()
            .try_fold(0usize, |sum, value| sum.checked_add(*value))
            .ok_or_else(|| shape("binary W32 recursive FRI reduction overflow"))?;
        let log_max_height = total_reduction
            .checked_add(params.log_final_poly_len)
            .and_then(|value| value.checked_add(params.log_blowup))
            .ok_or_else(|| shape("binary W32 recursive FRI height overflow"))?;
        if log_max_height > KoalaBear::TWO_ADICITY {
            return Err(shape(
                "binary W32 recursive FRI height exceeds field two-adicity",
            ));
        }
        let query_bits = (0..proof.query_proofs.len())
            .map(|_| {
                <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
                    KoalaBear,
                    Plonky3ChallengeV2,
                >>::sample_bits(challenger, circuit, log_max_height)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| shape(format!("binary W32 query sampling failed: {error:?}")))?;
        let beta_targets = &challenges[1..];
        let op_ids = verify_fri_circuit(
            circuit,
            proof,
            challenges[0],
            beta_targets,
            &query_bits,
            openings,
            params.log_blowup,
            None,
        )?;
        if !op_ids.is_empty() {
            return Err(shape(
                "arithmetic-only recursive FRI emitted MMCS operations",
            ));
        }
        verify_binary_paths(
            circuit,
            proof,
            challenges[0],
            beta_targets,
            &query_bits,
            openings,
            params.log_blowup,
        )?;
        Ok(Vec::new())
    }

    fn selectors_at_point_circuit(
        &self,
        circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
        domain: &DomainV2,
        point: &Target,
    ) -> RecursiveLagrangeSelectors {
        <Plonky3InnerPcsV2 as RecursivePcs<
            Plonky3StarkConfigV2,
            UpstreamInputProofV2,
            UpstreamOpeningProofV2,
            CommitmentV2,
            DomainV2,
        >>::selectors_at_point_circuit(&self.inner, circuit, domain, point)
    }

    fn evaluate_periodic_columns_at_point_circuit(
        &self,
        circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
        domain: &DomainV2,
        columns: &[Vec<KoalaBear>],
        point: Target,
    ) -> Result<Vec<Target>, VerificationError> {
        <Plonky3InnerPcsV2 as RecursivePcs<
            Plonky3StarkConfigV2,
            UpstreamInputProofV2,
            UpstreamOpeningProofV2,
            CommitmentV2,
            DomainV2,
        >>::evaluate_periodic_columns_at_point_circuit(
            &self.inner, circuit, domain, columns, point
        )
    }

    fn create_disjoint_domain(&self, domain: DomainV2, degree: usize) -> DomainV2 {
        <Plonky3InnerPcsV2 as RecursivePcs<
            Plonky3StarkConfigV2,
            UpstreamInputProofV2,
            UpstreamOpeningProofV2,
            CommitmentV2,
            DomainV2,
        >>::create_disjoint_domain(&self.inner, domain, degree)
    }

    fn split_domains(&self, domain: &DomainV2, degree: usize) -> Vec<DomainV2> {
        <Plonky3InnerPcsV2 as RecursivePcs<
            Plonky3StarkConfigV2,
            UpstreamInputProofV2,
            UpstreamOpeningProofV2,
            CommitmentV2,
            DomainV2,
        >>::split_domains(&self.inner, domain, degree)
    }

    fn log_size(&self, domain: &DomainV2) -> usize {
        domain.log_size()
    }

    fn first_point(&self, domain: &DomainV2) -> Plonky3ChallengeV2 {
        domain.first_point().into()
    }

    fn get_fri_random_opened_values(_proof: &BinaryOpeningProofV2) -> &[Vec<Vec<Vec<Target>>>] {
        &[]
    }
}

fn shape(message: impl Into<String>) -> VerificationError {
    VerificationError::InvalidProofShape(message.into())
}
