use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::Any;
use core::mem::{align_of, forget, size_of};

use hashbrown::HashMap;
use p3_circuit::ops::{NonPrimitivePreprocessedMap, NpoTypeId, PrimitiveOpType};
use p3_circuit::{Circuit, CircuitError, PreprocessedColumns};
use p3_field::{Algebra, ExtensionField, Field, PrimeCharacteristicRing, PrimeField64};
use p3_uni_stark::{StarkGenericConfig, SymbolicExpression, SymbolicExpressionExt, Val};
use p3_util::log2_ceil_usize;

use crate::air::{AluAir, AluExtMulKind, ConstAir, PublicAir};
use crate::config::StarkField;
use crate::field_params::ExtractBinomialW;
use crate::{ConstraintProfile, DynamicAirEntry, TablePacking};

/// Force a table's lane count to 1 when it holds only dummy data.
///
/// Multi-lane padding interacts incorrectly with lookup constraints during recursive
/// verification when a table has no real operations, so lanes are reduced to 1 (with a
/// warning) in that case.
pub(crate) fn reduce_lanes_if_dummy(
    table: &str,
    only_dummy: bool,
    configured_lanes: usize,
) -> usize {
    if only_dummy && configured_lanes > 1 {
        tracing::warn!(
            "{table} table holds only dummy operations but lanes={configured_lanes} > 1. \
             Reducing to lanes=1 to avoid recursive verification issues.",
        );
        1
    } else {
        configured_lanes
    }
}

/// Plugin trait for NPO-owned preprocessing over generic circuits.
///
/// Each implementation can update `PreprocessedColumns` (ext_reads, multiplicities, etc.)
/// and return base-field non-primitive preprocessed rows for its own `NpoTypeId`s.
pub trait NpoPreprocessor<F>: Send + Sync
where
    F: StarkField + PrimeField64,
{
    /// Whether preprocessing reads runtime-only metadata from the type-erased circuit.
    ///
    /// Implementations that return `false` must derive their output entirely from
    /// `PreprocessedColumns`. This lets common-data lowering release large builder and
    /// runner indices before materializing the preprocessed columns.
    fn requires_runtime_circuit_metadata(&self) -> bool {
        true
    }

    /// Capture the bounded part of runtime circuit metadata needed by this plugin.
    ///
    /// Common-data lowering uses this hook to preserve canonical plugin order while
    /// releasing the full verifier circuit before base-field NPO materialization.
    /// Runtime-dependent plugins that implement this hook must also implement
    /// [`Self::preprocess_with_runtime_metadata`].
    fn capture_runtime_metadata(
        &self,
        _circuit: &dyn Any,
        _preprocessed: &mut dyn Any,
    ) -> Result<Option<Box<dyn Any + Send + Sync>>, CircuitError> {
        Ok(None)
    }

    /// Run preprocessing from metadata returned by [`Self::capture_runtime_metadata`].
    fn preprocess_with_runtime_metadata(
        &self,
        runtime_metadata: &(dyn Any + Send + Sync),
        preprocessed: &mut dyn Any,
    ) -> Result<NonPrimitivePreprocessedMap<F>, CircuitError> {
        self.preprocess(runtime_metadata, preprocessed)
    }

    /// Run plugin-owned preprocessing over a generic circuit.
    ///
    /// `circuit` and `preprocessed` are type-erased; implementations downcast to the
    /// `PreprocessedColumns<ExtF>` shapes they support and return an empty map otherwise.
    fn preprocess(
        &self,
        circuit: &dyn Any,
        preprocessed: &mut dyn Any,
    ) -> Result<NonPrimitivePreprocessedMap<F>, CircuitError>;
}

/// Take one raw extension-field NPO table and convert it to canonical base rows.
///
/// Removing the source entry before conversion lets plugin-owned preprocessing
/// release each raw table as soon as its base representation is complete instead
/// of retaining every extension and base representation until all plugins finish.
pub fn take_non_primitive_base_columns<ExtF, F, const D: usize>(
    preprocessed: &mut PreprocessedColumns<ExtF, D>,
    op_type: &NpoTypeId,
) -> Result<Option<Vec<F>>, CircuitError>
where
    ExtF: Field + ExtensionField<F>,
    F: Field,
{
    let Some(values) = preprocessed.non_primitive.remove(op_type) else {
        return Ok(None);
    };
    values
        .into_iter()
        .map(|value| {
            value
                .as_base()
                .ok_or(CircuitError::InvalidPreprocessedValues)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn release_processed_raw_columns<ExtF, F, const D: usize>(
    preprocessed: &mut PreprocessedColumns<ExtF, D>,
    plugin_preprocessed: &NonPrimitivePreprocessedMap<F>,
) {
    for op_type in plugin_preprocessed.keys() {
        drop(preprocessed.non_primitive.remove(op_type));
        drop(preprocessed.dup_npo_outputs.remove(op_type));
    }
}

fn emit_common_lowering_dimensions<ExtF, F, const D: usize>(
    stage: &'static str,
    plugin_index: usize,
    preprocessed: &PreprocessedColumns<ExtF, D>,
    non_primitive_base: &NonPrimitivePreprocessedMap<F>,
) {
    let raw_npo_len_bytes = preprocessed
        .non_primitive
        .values()
        .map(|columns| columns.len().saturating_mul(size_of::<ExtF>()))
        .sum::<usize>();
    let raw_npo_capacity_bytes = preprocessed
        .non_primitive
        .values()
        .map(|columns| columns.capacity().saturating_mul(size_of::<ExtF>()))
        .sum::<usize>();
    let base_npo_len_bytes = non_primitive_base
        .values()
        .map(|columns| columns.len().saturating_mul(size_of::<F>()))
        .sum::<usize>();
    let base_npo_capacity_bytes = non_primitive_base
        .values()
        .map(|columns| columns.capacity().saturating_mul(size_of::<F>()))
        .sum::<usize>();
    tracing::info!(
        target: "z00z_plonky3_circuit_prover::common_lowering",
        stage,
        plugin_index,
        raw_npo_len_bytes,
        raw_npo_capacity_bytes,
        base_npo_len_bytes,
        base_npo_capacity_bytes,
        "bounded common-data NPO ownership"
    );
}

fn release_runtime_circuit_metadata<F>(circuit: &mut Circuit<F>) {
    circuit.ops.shrink_to_fit();
    circuit.private_input_rows.shrink_to_fit();
    drop(core::mem::take(&mut circuit.public_rows));
    drop(core::mem::take(&mut circuit.enabled_ops));
    drop(core::mem::take(&mut circuit.expr_to_widx));
    drop(core::mem::take(&mut circuit.non_primitive_trace_generators));
    drop(core::mem::take(
        &mut circuit.non_primitive_trace_generator_order,
    ));
    drop(core::mem::take(&mut circuit.tag_to_witness));
    drop(core::mem::take(&mut circuit.tag_to_op_id));
    drop(circuit.witness_rewrite.take());
}

/// Builds (AIR, degree) from preprocessed base data for a given NPO op_type.
/// Used by `get_airs_and_degrees_with_prep` so that AIR construction is plugin-driven
/// without requiring generic methods on the preprocessor trait (object safety).
pub trait NpoAirBuilder<SC, const D: usize>: Send + Sync
where
    SC: StarkGenericConfig,
    SymbolicExpressionExt<Val<SC>, SC::Challenge>: Algebra<SymbolicExpression<Val<SC>>>,
{
    /// Number of operations packed into a single AIR row for this NPO.
    ///
    /// Must match the `lanes` value returned by the corresponding [`TableProver`] implementation.
    /// Defaults to 1.
    fn lanes(&self) -> usize {
        1
    }

    /// Attempt to build an AIR and compute its degree from committed preprocessed data.
    ///
    /// The `lanes` argument is `self.lanes()` forwarded by the framework.
    /// When `retain_preprocessed_columns` is false, a matching builder must move the
    /// supplied vector into the AIR instead of cloning it. The AIR still owns the exact
    /// commitment input; only the duplicate prover-column copy is omitted.
    fn try_build(
        &self,
        op_type: &NpoTypeId,
        prep_base: &mut Vec<Val<SC>>,
        min_height: usize,
        lanes: usize,
        constraint_profile: ConstraintProfile,
        retain_preprocessed_columns: bool,
    ) -> Option<(CircuitTableAir<SC, D>, usize)>;
}

/// Enum wrapper to allow heterogeneous table AIRs in a single batch STARK aggregation.
///
/// This enables different AIR types to be collected into a single vector for
/// batch STARK proving/verification while maintaining type safety.
pub enum CircuitTableAir<SC, const D: usize>
where
    SC: StarkGenericConfig,
    SymbolicExpressionExt<Val<SC>, SC::Challenge>: Algebra<SymbolicExpression<Val<SC>>>,
{
    Const(ConstAir<Val<SC>, D>),
    Public(PublicAir<Val<SC>, D>),
    /// Unified ALU table for all arithmetic operations
    Alu(AluAir<Val<SC>, D>),
    Dynamic(DynamicAirEntry<SC>),
}

impl<SC, const D: usize> Clone for CircuitTableAir<SC, D>
where
    SC: StarkGenericConfig,
    SymbolicExpressionExt<Val<SC>, SC::Challenge>: Algebra<SymbolicExpression<Val<SC>>>,
{
    fn clone(&self) -> Self {
        match self {
            Self::Const(air) => Self::Const(air.clone()),
            Self::Public(air) => Self::Public(air.clone()),
            Self::Alu(air) => Self::Alu(air.clone()),
            Self::Dynamic(air) => Self::Dynamic(air.clone()),
        }
    }
}

/// Type alias for a vector of circuit table AIRs paired with their respective degrees (log of their trace height).
type CircuitAirsWithDegrees<SC, const D: usize> = Vec<(CircuitTableAir<SC, D>, usize)>;

/// Output of [`get_airs_and_degrees_with_prep`]: AIRs with degrees, primitive columns, and non-primitive columns.
type PrepOutput<SC, const D: usize> = (
    CircuitAirsWithDegrees<SC, D>,
    Vec<Vec<Val<SC>>>,
    NonPrimitivePreprocessedMap<Val<SC>>,
);

fn air_owned_columns<F: Clone>(columns: &mut Vec<F>, retain: bool) -> Vec<F> {
    if retain {
        columns.clone()
    } else {
        core::mem::take(columns)
    }
}

fn into_base_columns_reusing_allocation<ExtF, F>(
    mut values: Vec<ExtF>,
) -> Result<Vec<F>, CircuitError>
where
    ExtF: Field + ExtensionField<F>,
    F: Field,
{
    if values.iter().any(|value| value.as_base().is_none()) {
        return Err(CircuitError::InvalidPreprocessedValues);
    }
    if values.is_empty() {
        return Ok(Vec::new());
    }

    let ext_size = size_of::<ExtF>();
    let base_size = size_of::<F>();
    let Some(allocation_bytes) = values.capacity().checked_mul(ext_size) else {
        return Err(CircuitError::InvalidPreprocessedValues);
    };
    if ext_size < base_size
        || base_size == 0
        || !ext_size.is_multiple_of(base_size)
        || align_of::<ExtF>() != align_of::<F>()
        || !allocation_bytes.is_multiple_of(base_size)
    {
        return values
            .into_iter()
            .map(|value| {
                value
                    .as_base()
                    .ok_or(CircuitError::InvalidPreprocessedValues)
            })
            .collect();
    }

    let len = values.len();
    let base_capacity = allocation_bytes / base_size;
    let ext_ptr = values.as_mut_ptr();
    let base_ptr = ext_ptr.cast::<F>();
    for index in 0..len {
        // SAFETY:
        // - every source element was validated as base-only before mutation;
        // - source reads advance by `size_of::<ExtF>()`, while destination
        //   writes advance by the smaller `size_of::<F>()`, so a write can
        //   overlap only source elements already read;
        // - field elements are `Copy`, so no destructor is skipped;
        // - equal alignment and equal total allocation bytes make the
        //   resulting `Vec<F>` allocation/deallocation layout identical.
        let value = unsafe { ext_ptr.add(index).read() };
        let base = unsafe { value.as_base().unwrap_unchecked() };
        unsafe { base_ptr.add(index).write(base) };
    }
    forget(values);

    // SAFETY: the loop initialized exactly `len` base elements in the reused
    // allocation, and `base_capacity * size_of::<F>()` equals the original
    // allocation size with identical alignment.
    Ok(unsafe { Vec::from_raw_parts(base_ptr, len, base_capacity) })
}

pub fn get_airs_and_degrees_with_prep<
    SC: StarkGenericConfig + 'static + Send + Sync,
    ExtF: Field + ExtensionField<Val<SC>> + ExtractBinomialW<Val<SC>>,
    const D: usize,
>(
    circuit: &Circuit<ExtF>,
    packing: &TablePacking,
    non_primitive_preprocessors: &[Box<dyn NpoPreprocessor<Val<SC>>>],
    non_primitive_air_builders: &[Box<dyn NpoAirBuilder<SC, D>>],
    constraint_profile: ConstraintProfile,
) -> Result<PrepOutput<SC, D>, CircuitError>
where
    SymbolicExpressionExt<Val<SC>, SC::Challenge>: Algebra<SymbolicExpression<Val<SC>>>,
    Val<SC>: StarkField,
{
    let (preprocessed, non_primitive_base) =
        generate_preprocessed_with_plugins::<SC, ExtF, D>(circuit, non_primitive_preprocessors)?;
    get_airs_and_degrees_from_preprocessed(
        preprocessed,
        non_primitive_base,
        packing,
        non_primitive_air_builders,
        constraint_profile,
        true,
    )
}

/// Lower a circuit for common-data derivation without retaining the duplicate
/// prover-trace columns returned by [`get_airs_and_degrees_with_prep`].
pub fn get_airs_and_degrees_for_common_data<
    SC: StarkGenericConfig + 'static + Send + Sync,
    ExtF: Field + ExtensionField<Val<SC>> + ExtractBinomialW<Val<SC>>,
    const D: usize,
>(
    mut circuit: Circuit<ExtF>,
    packing: &TablePacking,
    non_primitive_preprocessors: &[Box<dyn NpoPreprocessor<Val<SC>>>],
    non_primitive_air_builders: &[Box<dyn NpoAirBuilder<SC, D>>],
    constraint_profile: ConstraintProfile,
) -> Result<Vec<(CircuitTableAir<SC, D>, usize)>, CircuitError>
where
    SymbolicExpressionExt<Val<SC>, SC::Challenge>: Algebra<SymbolicExpression<Val<SC>>>,
    Val<SC>: StarkField,
{
    let has_runtime_circuit_plugin = non_primitive_preprocessors
        .iter()
        .any(|plugin| plugin.requires_runtime_circuit_metadata());
    if !has_runtime_circuit_plugin {
        release_runtime_circuit_metadata(&mut circuit);
    }
    let mut preprocessed = circuit.generate_preprocessed_columns::<D>()?;
    let mut non_primitive_base = NonPrimitivePreprocessedMap::new();
    emit_common_lowering_dimensions("raw-ready", usize::MAX, &preprocessed, &non_primitive_base);

    // Capture only the bounded operation-order facts needed by runtime-aware
    // plugins. When every such plugin supports capture, the full verifier
    // circuit can be released before any plugin expands base-field NPO rows,
    // while the original plugin order (and therefore ext-read accounting) stays
    // unchanged. Unknown runtime plugins retain the conservative borrowed path.
    let captured_runtime = non_primitive_preprocessors
        .iter()
        .map(|plugin| {
            if plugin.requires_runtime_circuit_metadata() {
                plugin.capture_runtime_metadata(&circuit, &mut preprocessed)
            } else {
                Ok(None)
            }
        })
        .collect::<Result<Vec<_>, CircuitError>>()?;
    let all_runtime_captured = non_primitive_preprocessors
        .iter()
        .zip(captured_runtime.iter())
        .all(|(plugin, metadata)| {
            !plugin.requires_runtime_circuit_metadata() || metadata.is_some()
        });

    if all_runtime_captured {
        drop(circuit);
        let no_runtime_circuit: &dyn Any = &();
        for (plugin_index, (plugin, runtime_metadata)) in non_primitive_preprocessors
            .iter()
            .zip(captured_runtime.iter())
            .enumerate()
        {
            let plugin_preprocessed = if let Some(runtime_metadata) = runtime_metadata {
                plugin.preprocess_with_runtime_metadata(
                    runtime_metadata.as_ref(),
                    &mut preprocessed,
                )?
            } else {
                plugin.preprocess(no_runtime_circuit, &mut preprocessed)?
            };
            release_processed_raw_columns(&mut preprocessed, &plugin_preprocessed);
            non_primitive_base.extend(plugin_preprocessed);
            emit_common_lowering_dimensions(
                "plugin-complete",
                plugin_index,
                &preprocessed,
                &non_primitive_base,
            );
        }
    } else {
        let circuit_any: &dyn Any = &circuit;
        for (plugin_index, plugin) in non_primitive_preprocessors.iter().enumerate() {
            let plugin_preprocessed = plugin.preprocess(circuit_any, &mut preprocessed)?;
            release_processed_raw_columns(&mut preprocessed, &plugin_preprocessed);
            non_primitive_base.extend(plugin_preprocessed);
            emit_common_lowering_dimensions(
                "plugin-complete-borrowed",
                plugin_index,
                &preprocessed,
                &non_primitive_base,
            );
        }
        drop(circuit);
    }
    let (airs, primitive_columns, non_primitive_columns) = get_airs_and_degrees_from_preprocessed(
        preprocessed,
        non_primitive_base,
        packing,
        non_primitive_air_builders,
        constraint_profile,
        false,
    )?;
    debug_assert!(primitive_columns.iter().all(Vec::is_empty));
    debug_assert!(non_primitive_columns.is_empty());
    Ok(airs)
}

fn generate_preprocessed_with_plugins<
    SC: StarkGenericConfig + 'static + Send + Sync,
    ExtF: Field + ExtensionField<Val<SC>> + ExtractBinomialW<Val<SC>>,
    const D: usize,
>(
    circuit: &Circuit<ExtF>,
    non_primitive_preprocessors: &[Box<dyn NpoPreprocessor<Val<SC>>>],
) -> Result<
    (
        PreprocessedColumns<ExtF, D>,
        NonPrimitivePreprocessedMap<Val<SC>>,
    ),
    CircuitError,
>
where
    SymbolicExpressionExt<Val<SC>, SC::Challenge>: Algebra<SymbolicExpression<Val<SC>>>,
    Val<SC>: StarkField,
{
    let mut preprocessed = circuit.generate_preprocessed_columns::<D>()?;
    let circuit_any: &dyn Any = circuit;
    let non_primitive_base = apply_preprocessed_plugins::<SC, ExtF, D>(
        circuit_any,
        &mut preprocessed,
        non_primitive_preprocessors,
    )?;
    Ok((preprocessed, non_primitive_base))
}

fn apply_preprocessed_plugins<
    SC: StarkGenericConfig + 'static + Send + Sync,
    ExtF: Field + ExtensionField<Val<SC>> + ExtractBinomialW<Val<SC>>,
    const D: usize,
>(
    circuit_any: &dyn Any,
    preprocessed: &mut PreprocessedColumns<ExtF, D>,
    non_primitive_preprocessors: &[Box<dyn NpoPreprocessor<Val<SC>>>],
) -> Result<NonPrimitivePreprocessedMap<Val<SC>>, CircuitError>
where
    SymbolicExpressionExt<Val<SC>, SC::Challenge>: Algebra<SymbolicExpression<Val<SC>>>,
    Val<SC>: StarkField,
{
    let mut non_primitive_base: NonPrimitivePreprocessedMap<Val<SC>> = HashMap::new();
    for plugin in non_primitive_preprocessors {
        let plugin_prep = plugin.preprocess(circuit_any, preprocessed as &mut dyn Any)?;
        release_processed_raw_columns(preprocessed, &plugin_prep);
        non_primitive_base.extend(plugin_prep);
    }
    Ok(non_primitive_base)
}

fn get_airs_and_degrees_from_preprocessed<
    SC: StarkGenericConfig + 'static + Send + Sync,
    ExtF: Field + ExtensionField<Val<SC>> + ExtractBinomialW<Val<SC>>,
    const D: usize,
>(
    mut preprocessed: PreprocessedColumns<ExtF, D>,
    mut non_primitive_base: NonPrimitivePreprocessedMap<Val<SC>>,
    packing: &TablePacking,
    non_primitive_air_builders: &[Box<dyn NpoAirBuilder<SC, D>>],
    constraint_profile: ConstraintProfile,
    retain_preprocessed_columns: bool,
) -> Result<PrepOutput<SC, D>, CircuitError>
where
    SymbolicExpressionExt<Val<SC>, SC::Challenge>: Algebra<SymbolicExpression<Val<SC>>>,
    Val<SC>: StarkField,
{
    // Check if Public/Alu tables are empty and lanes > 1.
    // Using lanes > 1 with empty tables causes issues in recursive verification
    // due to a bug in how multi-lane padding interacts with lookup constraints.
    // We automatically reduce lanes to 1 in these cases with a warning.
    // IMPORTANT: This must be synchronized with prove_all_tables in batch_stark_prover.rs
    let public_idx = PrimitiveOpType::Public as usize;
    let alu_idx = PrimitiveOpType::Alu as usize;

    let public_rows = preprocessed.primitive[public_idx].len();
    let effective_public_lanes =
        reduce_lanes_if_dummy("Public", public_rows <= 1, packing.public_lanes());

    let alu_empty = preprocessed.primitive[alu_idx].is_empty();
    let effective_alu_lanes = reduce_lanes_if_dummy("ALU", alu_empty, packing.alu_lanes());

    let w_binomial = ExtF::extract_w();

    // Materialize one representation at a time. Recursive-verifier circuits
    // can contain millions of extension-field micro-ops; retaining their raw
    // extension columns while also building base columns and AIR-owned
    // columns multiplies the lowering high-water mark without adding any
    // theorem data. Plugin preprocessing above has already consumed the raw
    // NPO tables, so release those bodies and consume each primitive vector
    // as it is converted to the canonical base representation.
    let primitive = core::mem::take(&mut preprocessed.primitive);
    drop(core::mem::take(&mut preprocessed.non_primitive));
    drop(core::mem::take(&mut preprocessed.dup_npo_outputs));
    drop(core::mem::take(&mut preprocessed.hint_output_wids));
    let mut base_prep: Vec<Vec<Val<SC>>> = primitive
        .into_iter()
        .map(into_base_columns_reusing_allocation)
        .collect::<Result<Vec<_>, CircuitError>>()?;

    // Get min_height from packing configuration and pass it to AIRs
    let min_height = packing.min_trace_height();

    // Helper to compute degree that respects min_height
    let compute_degree = |num_rows: usize| -> usize {
        let natural_height = num_rows.next_power_of_two();
        let min_rows = min_height.next_power_of_two();
        log2_ceil_usize(natural_height.max(min_rows))
    };

    let mut table_preps: Vec<(CircuitTableAir<SC, D>, usize)> =
        Vec::with_capacity(base_prep.len() + non_primitive_base.len());

    #[allow(clippy::needless_range_loop)]
    for idx in 0..base_prep.len() {
        let table = PrimitiveOpType::from(idx);
        match table {
            PrimitiveOpType::Alu => {
                // ALU preprocessed per op from circuit.rs: 12 values
                // [sel_add_vs_mul, sel_bool, sel_muladd, sel_horner, a_idx, b_idx, c_idx, out_idx,
                //  mult_a_eff, b_is_creator, mult_c_eff, out_is_creator]
                //
                // mult_a_eff / mult_c_eff: -1 (reader or later unconstrained), or +N (first
                // unconstrained creator). We convert to 13 values for AluAir (same order, mult_c_eff last).
                let lane_12 = 12_usize;
                let lane_13 = 13_usize;
                let neg_one = <Val<SC>>::ZERO - <Val<SC>>::ONE;

                if !base_prep[idx].len().is_multiple_of(lane_12) {
                    return Err(CircuitError::InvalidPreprocessedValues);
                }
                let raw_ops = base_prep[idx].len() / lane_12;
                let target_rows = raw_ops + usize::from(alu_empty);
                base_prep[idx].resize(target_rows * lane_13, <Val<SC>>::ZERO);
                for operation_index in (0..raw_ops).rev() {
                    let source_start = operation_index * lane_12;
                    let target_start = operation_index * lane_13;
                    let chunk: [Val<SC>; 12] = base_prep[idx][source_start..source_start + lane_12]
                        .try_into()
                        .map_err(|_| CircuitError::InvalidPreprocessedValues)?;
                    let sel1 = chunk[0];
                    let sel2 = chunk[1];
                    let sel3 = chunk[2];
                    let sel4 = chunk[3];
                    let a_idx = chunk[4];
                    let b_idx = chunk[5];
                    let c_idx = chunk[6];
                    let out_idx = chunk[7];
                    let a_state = chunk[8].as_canonical_u64();
                    let b_is_creator = chunk[9].as_canonical_u64() != 0;
                    let c_state = chunk[10].as_canonical_u64();
                    let out_is_creator = chunk[11].as_canonical_u64() != 0;

                    // mult_a = -1 for all active rows; active = -mult_a = 1 always.
                    // Effective a-lookup mult = mult_a * a_reader_col (in get_alu_index_lookups).
                    // Effective c-lookup mult = mult_a * c_reader_col (in get_alu_index_lookups).
                    //
                    // a_state / c_state encoding:
                    //   0 → skip: col = 0, eff = 0
                    //   1 → reader: col = 1, eff = (-1)*1 = -1
                    //   2 → private creator: col = -(n_reads), eff = (-1)*(-(n_reads)) = +n_reads
                    let mult_a = neg_one;
                    let a_reader_col = match a_state {
                        0 => <Val<SC>>::ZERO,
                        1 => <Val<SC>>::ONE,
                        2 => {
                            let a_wid = a_idx.as_canonical_u64() as usize / D;
                            let n_reads = preprocessed.ext_reads.get(a_wid).copied().unwrap_or(0);
                            <Val<SC>>::ZERO - <Val<SC>>::from_u32(n_reads)
                        }
                        _ => <Val<SC>>::ZERO,
                    };
                    let c_reader_col = match c_state {
                        0 => <Val<SC>>::ZERO,
                        1 => <Val<SC>>::ONE,
                        2 => {
                            let c_wid = c_idx.as_canonical_u64() as usize / D;
                            let n_reads = preprocessed.ext_reads.get(c_wid).copied().unwrap_or(0);
                            <Val<SC>>::ZERO - <Val<SC>>::from_u32(n_reads)
                        }
                        _ => <Val<SC>>::ZERO,
                    };

                    // b: creator if b_is_creator, reader otherwise.
                    let mult_b = if b_is_creator {
                        let b_wid = b_idx.as_canonical_u64() as usize / D;
                        let n_reads = preprocessed.ext_reads.get(b_wid).copied().unwrap_or(0);
                        <Val<SC>>::from_u32(n_reads)
                    } else {
                        neg_one
                    };

                    // out: creator if out_is_creator, reader otherwise.
                    let mult_out = if out_is_creator {
                        let out_wid = out_idx.as_canonical_u64() as usize / D;
                        let n_reads = preprocessed.ext_reads.get(out_wid).copied().unwrap_or(0);
                        <Val<SC>>::from_u32(n_reads)
                    } else {
                        neg_one
                    };

                    base_prep[idx][target_start..target_start + lane_13].copy_from_slice(&[
                        mult_a,
                        sel1,
                        sel2,
                        sel3,
                        sel4,
                        a_idx,
                        b_idx,
                        c_idx,
                        out_idx,
                        mult_b,
                        mult_out,
                        a_reader_col,
                        c_reader_col,
                    ]);
                }
                base_prep[idx].shrink_to_fit();

                let num_ops = base_prep[idx].len() / lane_13;
                let horner_k = packing.horner_packed_steps();
                let reduction = AluExtMulKind::resolve(
                    D,
                    w_binomial,
                    D == 5 && ExtF::alu_is_quintic_trinomial(),
                )
                .expect(
                    "ALU preprocessed path needs binomial W when D>1 and the element field is \
                     not the quintic-trinomial ALU variant. Use D=1 for base-field circuits \
                     (ExtF = Val<SC>); for extension circuits use D = ExtF::DIMENSION and a \
                     binomial or supported quintic ExtF.",
                );
                let alu_air = AluAir::from_reduction_with_preprocessed(
                    num_ops,
                    effective_alu_lanes,
                    reduction,
                    air_owned_columns(&mut base_prep[idx], retain_preprocessed_columns),
                    horner_k,
                )
                .with_min_height(min_height);
                let num_entries = alu_air.scheduled_entry_count();
                let num_rows = num_entries.div_ceil(effective_alu_lanes);
                table_preps.push((CircuitTableAir::Alu(alu_air), compute_degree(num_rows)));
            }
            PrimitiveOpType::Public => {
                // Public preprocessed per op from circuit.rs: 1 value (D-scaled out_idx).
                // Convert to [ext_mult, out_idx] pairs using ext_reads.
                let num_ops = base_prep[idx].len();
                base_prep[idx].resize(num_ops * 2, <Val<SC>>::ZERO);
                for operation_index in (0..num_ops).rev() {
                    let out_idx = base_prep[idx][operation_index];
                    let out_wid =
                        (<Val<SC> as PrimeField64>::as_canonical_u64(&out_idx) as usize) / D;
                    let n_reads = preprocessed.ext_reads.get(out_wid).copied().unwrap_or(0);
                    let target_start = operation_index * 2;
                    base_prep[idx][target_start] = <Val<SC>>::from_u32(n_reads);
                    base_prep[idx][target_start + 1] = out_idx;
                }
                base_prep[idx].shrink_to_fit();

                let public_air = PublicAir::new_with_preprocessed(
                    num_ops,
                    effective_public_lanes,
                    air_owned_columns(&mut base_prep[idx], retain_preprocessed_columns),
                )
                .with_min_height(min_height);
                let num_rows = num_ops.div_ceil(effective_public_lanes);
                table_preps.push((
                    CircuitTableAir::Public(public_air),
                    compute_degree(num_rows),
                ));
            }
            PrimitiveOpType::Const => {
                // Const preprocessed per op from circuit.rs: 1 value (D-scaled out_idx).
                // Convert to [ext_mult, out_idx] pairs using ext_reads.
                let height = base_prep[idx].len();
                base_prep[idx].resize(height * 2, <Val<SC>>::ZERO);
                for operation_index in (0..height).rev() {
                    let out_idx = base_prep[idx][operation_index];
                    let out_wid = out_idx.as_canonical_u64() as usize / D;
                    let n_reads = preprocessed.ext_reads.get(out_wid).copied().unwrap_or(0);
                    let target_start = operation_index * 2;
                    base_prep[idx][target_start] = <Val<SC>>::from_u32(n_reads);
                    base_prep[idx][target_start + 1] = out_idx;
                }
                base_prep[idx].shrink_to_fit();

                let const_air = ConstAir::new_with_preprocessed(
                    height,
                    air_owned_columns(&mut base_prep[idx], retain_preprocessed_columns),
                )
                .with_min_height(min_height);
                table_preps.push((CircuitTableAir::Const(const_air), compute_degree(height)));
            }
        }
    }

    // Primitive AIR construction is the last consumer of `ext_reads`. Recursive
    // verifier circuits can have tens of millions of witnesses, so retaining
    // that census while materializing the NPO AIRs needlessly overlaps two
    // lowering stages. End the primitive stage here; all NPO commitment columns
    // have already been emitted into `non_primitive_base` by the plugins above.
    drop(preprocessed);

    // Iterate air builders first (fixed registration order) so that the
    // resulting AIR ordering matches the prover's non_primitive_provers order.
    for builder in non_primitive_air_builders {
        for (op_type, prep_base) in non_primitive_base.iter_mut() {
            // TablePacking overrides the builder's own default lane count.
            let lanes = packing
                .npo_lanes(op_type)
                .unwrap_or_else(|| builder.lanes());
            if let Some((air, degree)) = builder.try_build(
                op_type,
                prep_base,
                min_height,
                lanes,
                constraint_profile,
                retain_preprocessed_columns,
            ) {
                table_preps.push((air, degree));
                break;
            }
        }
    }

    if !retain_preprocessed_columns {
        non_primitive_base.clear();
    }
    Ok((table_preps, base_prep, non_primitive_base))
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use p3_field::extension::BinomialExtensionField;
    use p3_field::{BasedVectorSpace, PrimeCharacteristicRing, PrimeField64};
    use p3_koala_bear::KoalaBear;

    use super::into_base_columns_reusing_allocation;

    type KoalaBearD4 = BinomialExtensionField<KoalaBear, 4>;

    #[test]
    fn base_column_compaction_reuses_the_extension_allocation() {
        let extension = (0_u64..32).map(KoalaBearD4::from_u64).collect::<Vec<_>>();
        let original_pointer = extension.as_ptr().cast::<u8>();
        let original_bytes = extension.capacity() * core::mem::size_of::<KoalaBearD4>();

        let base = into_base_columns_reusing_allocation::<_, KoalaBear>(extension).unwrap();

        assert_eq!(base.as_ptr().cast::<u8>(), original_pointer);
        assert_eq!(
            base.capacity() * core::mem::size_of::<KoalaBear>(),
            original_bytes
        );
        assert_eq!(
            base.iter()
                .map(PrimeField64::as_canonical_u64)
                .collect::<Vec<_>>(),
            (0_u64..32).collect::<Vec<_>>()
        );
    }

    #[test]
    fn base_column_compaction_rejects_non_base_values() {
        let non_base = KoalaBearD4::from_basis_coefficients_slice(&[
            KoalaBear::ONE,
            KoalaBear::ONE,
            KoalaBear::ZERO,
            KoalaBear::ZERO,
        ])
        .unwrap();

        assert!(into_base_columns_reusing_allocation::<_, KoalaBear>(vec![non_base]).is_err());
    }
}
