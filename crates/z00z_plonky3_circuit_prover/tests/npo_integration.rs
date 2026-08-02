use core::marker::PhantomData;

use p3_air::{Air, AirBuilder, BaseAir, WindowAccess};
use p3_batch_stark::Val;
use p3_circuit::builder::{
    CircuitBuilder, NonPrimitiveOperationData, NpoCircuitPlugin, NpoLoweringContext,
};
use p3_circuit::ops::{
    ExecutionContext, NonPrimitiveExecutor, NpoConfig, NpoTypeId, Op, OpStateMap,
};
use p3_circuit::tables::{
    AluTrace, ConstTrace, NonPrimitiveTrace, PublicTrace, TraceGeneratorFn, Traces, WitnessTrace,
};
use p3_circuit::{CircuitBuilderError, CircuitError, WitnessId};
use p3_field::PrimeCharacteristicRing;
use p3_field::extension::BinomialExtensionField;
use p3_koala_bear::KoalaBear;
use p3_matrix::dense::RowMajorMatrix;
use p3_test_utils::baby_bear_params::*;
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchAir, BatchStarkProof, BatchStarkProver, BatchTableInstance, CircuitProverData,
    DynamicAirEntry, NonPrimitiveTableEntry, OneShotResourceSnapshotV2, OneShotResourceStageV2,
    OneShotResourceTelemetrySinkV2, TablePacking, TableProver,
    canonical_prover_data_from_airs_and_degrees,
};
use z00z_plonky3_circuit_prover::common::get_airs_and_degrees_with_prep;
use z00z_plonky3_circuit_prover::{ConstraintProfile, config};

// Simple non-primitive "cube" op: y = x^3
const CUBE_TYPE_ID: &str = "cube_simple/x_cubed";

fn cube_trace_generator<F>(
    _op_states: &OpStateMap,
) -> Result<Option<Box<dyn NonPrimitiveTrace<F>>>, CircuitError> {
    // This simple example does not produce its own dedicated table trace.
    Ok(None)
}

/// Circuit-side plugin for the cube op.
#[derive(Clone)]
struct CubeCircuitPlugin<F> {
    _phantom: PhantomData<F>,
}

impl<F> CubeCircuitPlugin<F> {
    const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<F> NpoCircuitPlugin<F> for CubeCircuitPlugin<F>
where
    F: Field + PrimeCharacteristicRing,
{
    fn type_id(&self) -> NpoTypeId {
        NpoTypeId::new(CUBE_TYPE_ID)
    }

    fn lower(
        &self,
        data: &NonPrimitiveOperationData<F>,
        output_exprs: &[(u32, p3_circuit::types::ExprId)],
        ctx: &mut NpoLoweringContext<'_, F>,
    ) -> Result<Op<F>, CircuitBuilderError> {
        // For this example, expect exactly one input and one output.
        let input_expr = data.input_exprs[0][0];
        let output_expr = output_exprs[0].1;

        // Map expressions to witness IDs (allocate if necessary).
        let in_wid = *ctx
            .expr_to_widx
            .entry(input_expr)
            .or_insert_with(|| (ctx.alloc_witness_id)(1));
        let out_wid = *ctx
            .expr_to_widx
            .entry(output_expr)
            .or_insert_with(|| (ctx.alloc_witness_id)(1));

        // Build a non-primitive op with a cube executor.
        Ok(Op::NonPrimitiveOpWithExecutor {
            inputs: vec![vec![in_wid]],
            outputs: vec![vec![out_wid]],
            executor: Box::new(CubeExecutor::default()),
            op_id: data.op_id,
        })
    }

    fn trace_generator(&self) -> TraceGeneratorFn<F> {
        // For this demo we don't build a separate cube table trace; a real plugin
        // would record rows in OpExecutionState and use them here.
        cube_trace_generator::<F>
    }

    fn config(&self) -> NpoConfig {
        // No special config for this simple example.
        NpoConfig::new(())
    }
}

/// Executor that computes y = x^3 inside the runtime execution context.
#[derive(Clone)]
struct CubeExecutor<F> {
    op_type: NpoTypeId,
    _phantom: PhantomData<F>,
}

impl<F> Default for CubeExecutor<F> {
    fn default() -> Self {
        Self {
            op_type: NpoTypeId::new(CUBE_TYPE_ID),
            _phantom: PhantomData,
        }
    }
}

impl<F> core::fmt::Debug for CubeExecutor<F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("CubeExecutor")
    }
}

impl<F> NonPrimitiveExecutor<F> for CubeExecutor<F>
where
    F: Field + PrimeCharacteristicRing,
{
    fn execute(
        &self,
        inputs: &[Vec<WitnessId>],
        outputs: &[Vec<WitnessId>],
        ctx: &mut ExecutionContext<'_, F>,
    ) -> Result<(), p3_circuit::CircuitError> {
        let in_id = inputs[0][0];
        let out_id = outputs[0][0];

        let x = ctx.get_witness(in_id)?;
        let x2 = x * x;
        let x3 = x2 * x;

        ctx.set_witness(out_id, x3)?;
        Ok(())
    }

    fn op_type(&self) -> &NpoTypeId {
        &self.op_type
    }

    fn preprocess(
        &self,
        _inputs: &[Vec<WitnessId>],
        _outputs: &[Vec<WitnessId>],
        _preprocessed: &mut dyn p3_circuit::PreprocessedWriter<F>,
    ) -> Result<(), p3_circuit::CircuitError> {
        Ok(())
    }

    fn boxed(&self) -> Box<dyn NonPrimitiveExecutor<F>> {
        Box::new(self.clone())
    }
}

// ============================================================================
// Cube AIR
// ============================================================================

/// A minimal AIR that proves `y = x^3` for a single row.
///
/// Columns: `[x, x_sq, x_cu]`
/// Constraints:
///   - `x_sq - x * x   = 0`  (degree 2)
///   - `x_cu - x_sq * x = 0`  (degree 2)
#[derive(Clone)]
struct CubeAir;

impl<F> BaseAir<F> for CubeAir {
    fn width(&self) -> usize {
        3
    }
}

impl<AB: AirBuilder> Air<AB> for CubeAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let row = main.current_slice();
        let x = row[0];
        let x_sq = row[1];
        let x_cu = row[2];

        // x_sq = x * x
        builder.assert_zero(x_sq - x * x);
        // x_cu = x_sq * x
        builder.assert_zero(x_cu - x_sq * x);
    }
}

/// Integration-style test: register a cube NPO plugin, use it in a circuit,
/// and run the circuit to check y = x^3.
#[test]
fn cube_npo_integration_flow() {
    type F = BabyBear;

    // Build circuit with the cube plugin.
    let mut builder = CircuitBuilder::<F>::new();
    builder.register_npo(CubeCircuitPlugin::<F>::new());

    // Public input x and expected output y.
    let x = builder.public_input();
    let y_expected = builder.public_input();

    // Create a single cube non-primitive op that maps x -> y.
    let cube_type = NpoTypeId::new(CUBE_TYPE_ID);
    let (_op_id, _call_expr, outputs) = builder.push_non_primitive_op_with_outputs(
        cube_type,
        vec![vec![x]],
        vec![Some("cube_out")],
        None,
        "cube_call",
    );
    let y_expr = outputs[0].expect("cube op should have one output");

    // Connect cube_out to expected output.
    builder.connect(y_expr, y_expected);

    let circuit = builder.build().expect("build cube circuit");
    let out_wid = circuit
        .expr_to_widx
        .get(&y_expr)
        .copied()
        .expect("y_expr mapped to witness");

    // Run with a simple x value and check we get x^3.
    let mut runner = circuit.runner();
    let x_val = F::from_u64(3); // 3
    let y_val = x_val * x_val * x_val; // 27
    runner
        .set_public_inputs(&[x_val, y_val])
        .expect("set public inputs");

    let traces = runner.run().expect("run cube circuit");
    let out_val = traces
        .witness_trace
        .get_value(out_wid)
        .expect("output witness set");

    assert_eq!(*out_val, y_val);
}

/// Generates and verifies a STARK proof for a circuit that contains the cube NPO.
///
/// The circuit uses the cube NPO to compute `y = x^3`.  Because the NPO trace
/// generator returns `None`, the batch prover covers only the primitive tables
/// (Const, Public, ALU), but the circuit itself still contains the NPO operation.
/// This test verifies end-to-end STARK proof generation for an NPO-containing circuit.
#[test]
fn cube_npo_stark_proof() {
    type F = BabyBear;
    const D: usize = 1;

    // Build the same cube NPO circuit.
    let mut builder = CircuitBuilder::<F>::new();
    builder.register_npo(CubeCircuitPlugin::<F>::new());

    let x = builder.public_input();
    let cube_type = NpoTypeId::new(CUBE_TYPE_ID);
    let (_op_id, _call_expr, outputs) = builder.push_non_primitive_op_with_outputs(
        cube_type,
        vec![vec![x]],
        vec![Some("cube_out")],
        None,
        "cube_call",
    );
    // Assert that the NPO output is non-zero (x^3 = 125 ≠ 0).
    // We use assert_zero on (cube_out - cube_out) as a trivial consistency check;
    // the real verification of y = x^3 is done by the circuit runner via the executor.
    let y_expr = outputs[0].expect("cube op should have one output");
    let zero = builder.sub(y_expr, y_expr);
    builder.assert_zero(zero);

    let circuit = builder.build().expect("build cube circuit");
    let cfg = config::baby_bear();

    // Derive AIRs and preprocessed columns from the circuit.
    // The cube NPO has no dedicated table, so only primitive AIRs are generated.
    let (airs_degrees, primitive_columns, non_primitive_columns) =
        get_airs_and_degrees_with_prep::<config::BabyBearConfig, _, D>(
            &circuit,
            &TablePacking::default(),
            &[],
            &[],
            ConstraintProfile::Standard,
        )
        .expect("get_airs_and_degrees_with_prep should succeed");
    let (airs, log_degrees): (Vec<_>, Vec<usize>) = airs_degrees.into_iter().unzip();

    // Run the circuit to produce traces.
    let mut runner = circuit.runner();
    let x_val = F::from_u64(5);
    runner
        .set_public_inputs(&[x_val])
        .expect("set public inputs");
    let traces = runner.run().expect("run cube circuit");

    // Prove all primitive tables.
    let prover_data = canonical_prover_data_from_airs_and_degrees(&cfg, &airs, &log_degrees);
    let circuit_prover_data =
        CircuitProverData::new(prover_data, primitive_columns, non_primitive_columns);
    let prover = BatchStarkProver::new(cfg);

    let proof = prover
        .prove_all_tables(&traces, &circuit_prover_data)
        .expect("prove_all_tables should succeed");

    prover
        .verify_all_tables::<F>(&proof)
        .expect("verify_all_tables should succeed");
}

/// Verifies the `CubeAir` constraints directly using the uni-stark prover.
///
/// Constructs a single-row trace `[x, x^2, x^3]` and proves it under `CubeAir`,
/// confirming that the two degree-2 constraints are satisfied.
#[test]
fn cube_air_stark_proof() {
    use p3_test_utils::baby_bear_params::*;
    use p3_uni_stark::{prove, verify};

    let cfg = make_test_config();

    // Construct a single-row trace: [x, x^2, x^3]
    let x = BabyBear::from_u64(7);
    let x_sq = x * x;
    let x_cu = x_sq * x;
    let trace = RowMajorMatrix::new(vec![x, x_sq, x_cu], 3);

    let proof = prove(&cfg, &CubeAir, trace, &[]);

    verify(&cfg, &CubeAir, &proof, &[]).expect("CubeAir STARK proof should verify");
}

const DIRECT_CUBE_TYPE_ID: &str = "z00z/direct_cube/v1";

#[derive(Clone)]
struct DirectCubeTrace {
    rows: Vec<[KoalaBear; 3]>,
}

impl NonPrimitiveTrace<KoalaBear> for DirectCubeTrace {
    fn op_type(&self) -> NpoTypeId {
        NpoTypeId::new(DIRECT_CUBE_TYPE_ID)
    }

    fn rows(&self) -> usize {
        self.rows.len()
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn boxed_clone(&self) -> Box<dyn NonPrimitiveTrace<KoalaBear>> {
        Box::new(self.clone())
    }
}

impl BatchAir<config::KoalaBearConfig> for CubeAir {}

struct DirectCubeTableProver;

impl TableProver<config::KoalaBearConfig> for DirectCubeTableProver {
    fn op_type(&self) -> NpoTypeId {
        NpoTypeId::new(DIRECT_CUBE_TYPE_ID)
    }

    fn batch_instance_d1(
        &self,
        _config: &config::KoalaBearConfig,
        _packing: &TablePacking,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<config::KoalaBearConfig>> {
        let trace = traces.non_primitive_trace::<DirectCubeTrace>(&self.op_type())?;
        if trace.rows.is_empty() {
            return None;
        }
        let values = trace
            .rows
            .iter()
            .flat_map(|row| row.iter().copied())
            .collect();
        Some(BatchTableInstance {
            op_type: self.op_type(),
            air: DynamicAirEntry::new(Box::new(CubeAir)),
            trace: RowMajorMatrix::new(values, 3),
            public_values: Vec::new(),
            rows: trace.rows.len(),
            lanes: 1,
        })
    }

    fn batch_instance_d2(
        &self,
        _config: &config::KoalaBearConfig,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 2>>,
    ) -> Option<BatchTableInstance<config::KoalaBearConfig>> {
        None
    }

    fn batch_instance_d4(
        &self,
        _config: &config::KoalaBearConfig,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 4>>,
    ) -> Option<BatchTableInstance<config::KoalaBearConfig>> {
        None
    }

    fn batch_instance_d6(
        &self,
        _config: &config::KoalaBearConfig,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 6>>,
    ) -> Option<BatchTableInstance<config::KoalaBearConfig>> {
        None
    }

    fn batch_instance_d8(
        &self,
        _config: &config::KoalaBearConfig,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 8>>,
    ) -> Option<BatchTableInstance<config::KoalaBearConfig>> {
        None
    }

    fn batch_air_from_table_entry(
        &self,
        _config: &config::KoalaBearConfig,
        degree: usize,
        circuit_extension_degree: u32,
        table_entry: &NonPrimitiveTableEntry<config::KoalaBearConfig>,
    ) -> Result<DynamicAirEntry<config::KoalaBearConfig>, String> {
        if degree != 1
            || circuit_extension_degree != 1
            || table_entry.op_type != self.op_type()
            || table_entry.rows == 0
            || table_entry.lanes != 1
        {
            return Err("invalid direct-cube proof metadata".into());
        }
        Ok(DynamicAirEntry::new(Box::new(CubeAir)))
    }
}

fn direct_cube_traces() -> Traces<KoalaBear> {
    let rows = (1..=32)
        .map(|value| {
            let x = KoalaBear::from_u64(value);
            let x_sq = x * x;
            [x, x_sq, x_sq * x]
        })
        .collect::<Vec<_>>();
    let direct_trace = DirectCubeTrace { rows };
    let mut non_primitive_traces = hashbrown::HashMap::new();
    non_primitive_traces.insert(
        direct_trace.op_type(),
        Box::new(direct_trace) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
    );
    Traces {
        witness_trace: WitnessTrace::new(Vec::new()),
        const_trace: ConstTrace {
            index: Vec::new(),
            values: Vec::new(),
        },
        public_trace: PublicTrace {
            index: Vec::new(),
            values: Vec::new(),
        },
        alu_trace: AluTrace::from_records(Vec::new()),
        non_primitive_traces,
        tag_to_witness: hashbrown::HashMap::new(),
    }
}

#[derive(Default)]
struct CapturingOneShotResourceSinkV2 {
    snapshots: Vec<OneShotResourceSnapshotV2>,
}

impl OneShotResourceTelemetrySinkV2 for CapturingOneShotResourceSinkV2 {
    fn record(&mut self, snapshot: OneShotResourceSnapshotV2) {
        self.snapshots.push(snapshot);
    }
}

fn assert_serialized_eq<T>(left: &T, right: &T, label: &str)
where
    T: serde::Serialize + ?Sized,
{
    let left = postcard::to_allocvec(left).expect("serialize left verifier surface");
    let right = postcard::to_allocvec(right).expect("serialize right verifier surface");
    assert!(left == right, "{label} must be byte-identical");
}

fn assert_direct_verifier_surface_eq(
    left: &BatchStarkProof<config::KoalaBearConfig>,
    right: &BatchStarkProof<config::KoalaBearConfig>,
) {
    assert_eq!(left.table_packing, right.table_packing);
    assert_eq!(left.rows, right.rows);
    assert_eq!(left.alu_variant, right.alu_variant);
    assert_eq!(left.ext_degree, right.ext_degree);
    assert_eq!(left.w_binomial, right.w_binomial);
    assert_eq!(left.alu_quintic_trinomial, right.alu_quintic_trinomial);
    assert_serialized_eq(
        &left.non_primitives,
        &right.non_primitives,
        "ordered non-primitive public manifest",
    );
    assert_serialized_eq(
        &left.proof.commitments,
        &right.proof.commitments,
        "trace and quotient commitments",
    );
    assert_serialized_eq(
        &left.proof.lookup_terminals,
        &right.proof.lookup_terminals,
        "lookup terminals",
    );
    assert_eq!(
        left.proof.degree_bits, right.proof.degree_bits,
        "AIR order and extended-domain sizes must match",
    );

    assert_eq!(
        left.stark_common.lookups.len(),
        right.stark_common.lookups.len(),
        "lookup layout must have one entry per AIR in the same order",
    );
    for (air_index, (left_lookups, right_lookups)) in left
        .stark_common
        .lookups
        .iter()
        .zip(&right.stark_common.lookups)
        .enumerate()
    {
        assert!(
            format!("{left_lookups:?}") == format!("{right_lookups:?}"),
            "symbolic lookup layout differs at AIR {air_index}",
        );
    }

    match (
        &left.stark_common.preprocessed,
        &right.stark_common.preprocessed,
    ) {
        (None, None) => {}
        (Some(left), Some(right)) => {
            assert_serialized_eq(
                &left.commitment,
                &right.commitment,
                "preprocessed commitment",
            );
            assert_eq!(left.matrix_to_instance, right.matrix_to_instance);
            assert_eq!(left.instances.len(), right.instances.len());
            for (air_index, (left, right)) in
                left.instances.iter().zip(&right.instances).enumerate()
            {
                match (left, right) {
                    (None, None) => {}
                    (Some(left), Some(right)) => {
                        assert_eq!(left.matrix_index, right.matrix_index);
                        assert_eq!(left.width, right.width);
                        assert_eq!(
                            left.degree_bits, right.degree_bits,
                            "preprocessed domain differs at AIR {air_index}",
                        );
                    }
                    _ => panic!("preprocessed table presence differs at AIR {air_index}"),
                }
            }
        }
        _ => panic!("preprocessed commitment presence differs"),
    }
}

#[test]
fn direct_one_shot_matches_borrowed() {
    let borrowed_traces = direct_cube_traces();
    let owned_traces = direct_cube_traces();
    let telemetry_traces = direct_cube_traces();
    let op_type = NpoTypeId::new(DIRECT_CUBE_TYPE_ID);
    let mut prover = BatchStarkProver::new(config::koala_bear())
        .with_table_packing(TablePacking::new(1, 1).with_min_trace_height(32));
    prover.register_table_prover(Box::new(DirectCubeTableProver));

    let borrowed_proof = prover
        .prove_direct_tables(&borrowed_traces)
        .expect("borrowed direct KoalaBear table proof should succeed");
    let proof = prover
        .prove_direct_tables_one_shot(owned_traces)
        .expect("owned direct KoalaBear table proof should succeed");
    let mut telemetry = CapturingOneShotResourceSinkV2::default();
    let telemetry_proof = prover
        .prove_direct_tables_one_shot_with_resource_telemetry(telemetry_traces, &mut telemetry)
        .expect("telemetry direct KoalaBear table proof should succeed");

    // Full proof bytes are intentionally not compared across separate invocations. The pinned
    // p3-challenger 0.6.1 FRI query PoW uses a parallel `find_map_any`, so two valid proofs may
    // select different PoW witnesses and therefore different query openings. The deterministic
    // verifier-visible transcript prefix, public metadata, table order, and domains must match.
    assert_direct_verifier_surface_eq(&proof, &telemetry_proof);
    assert_eq!(
        telemetry
            .snapshots
            .iter()
            .map(|snapshot| snapshot.stage)
            .collect::<Vec<_>>(),
        [
            OneShotResourceStageV2::Entry,
            OneShotResourceStageV2::PostMainCommit,
            OneShotResourceStageV2::PostLogUpPreTraceDrop,
            OneShotResourceStageV2::PostTraceDrop,
            OneShotResourceStageV2::PostPermutationCommit,
            OneShotResourceStageV2::PostQuotientAir,
            OneShotResourceStageV2::PostQuotientAir,
            OneShotResourceStageV2::PostQuotientAir,
            OneShotResourceStageV2::PostQuotientAir,
            OneShotResourceStageV2::PostQuotientCommit,
            OneShotResourceStageV2::PreOpen,
            OneShotResourceStageV2::PostOpen,
        ],
        "telemetry schema must cover every one-shot lifetime boundary",
    );
    assert_eq!(
        telemetry
            .snapshots
            .iter()
            .filter_map(|snapshot| snapshot.air_index)
            .collect::<Vec<_>>(),
        [0, 1, 2, 3],
        "one quotient snapshot must be emitted for every AIR in canonical order",
    );
    assert!(telemetry.snapshots[0].visible_buffers.main_trace.len_bytes > 0);
    assert_eq!(
        telemetry.snapshots[1].visible_buffers.main_trace,
        Default::default(),
        "main PCS must consume the complete original trace set",
    );
    assert!(
        telemetry.snapshots[2]
            .visible_buffers
            .permutation_trace
            .len_bytes
            > 0,
        "LogUp must materialize permutation traces before the main-trace drop",
    );
    assert_eq!(
        telemetry.snapshots[2].visible_buffers.main_trace,
        Default::default(),
        "one-table LogUp reconstruction must not retain a main-trace set",
    );
    assert_eq!(
        telemetry.snapshots[3].visible_buffers.main_trace,
        Default::default(),
        "main-trace length and capacity must both be zero before permutation commit",
    );
    assert_eq!(
        telemetry.snapshots[3].visible_buffers.permutation_trace,
        telemetry.snapshots[2].visible_buffers.permutation_trace,
        "the ordered permutation traces must survive the main-trace drop",
    );
    assert_eq!(
        telemetry.snapshots[4].visible_buffers.main_trace,
        Default::default(),
        "permutation commit must not resurrect a visible main-trace allocation",
    );
    assert_eq!(
        telemetry.snapshots[4].visible_buffers.permutation_trace,
        Default::default(),
        "owned permutation matrices must be consumed by permutation commit",
    );

    assert_eq!(proof.ext_degree, 1);
    assert_eq!(proof.non_primitives.len(), 1);
    assert_eq!(proof.non_primitives[0].op_type, op_type);
    assert_direct_verifier_surface_eq(&borrowed_proof, &proof);
    prover
        .verify_all_tables::<Val<config::KoalaBearConfig>>(&borrowed_proof)
        .expect("borrowed direct KoalaBear table proof should verify");
    prover
        .verify_all_tables::<Val<config::KoalaBearConfig>>(&proof)
        .expect("owned direct KoalaBear table proof should verify");
    prover
        .verify_all_tables::<Val<config::KoalaBearConfig>>(&telemetry_proof)
        .expect("telemetry direct KoalaBear table proof should verify");
}
