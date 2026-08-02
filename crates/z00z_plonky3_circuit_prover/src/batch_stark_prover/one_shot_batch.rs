//! Memory-bounded one-shot Batch-STARK proving.
//!
//! The upstream borrowed-instance API keeps every caller-owned main trace alive
//! through the complete proof. A one-shot worker consumes them into the main
//! PCS commitment, then reconstructs at most one committed table at a time for
//! LogUp. The transcript, AIRs, quotient construction, PCS openings, and proof
//! format are identical to `p3_batch_stark::prove_batch`.

use alloc::{vec, vec::Vec};
use core::mem::size_of;

use p3_air::Air;
use p3_air::symbolic::{AirLayout, SymbolicExpressionExt};
use p3_batch_stark::proof::{BatchCommitments, BatchOpenedValues, OpenedValuesWithLookups};
use p3_batch_stark::symbolic::{get_log_num_quotient_chunks, get_symbolic_constraints};
use p3_batch_stark::{
    BatchProof, BatchTranscript, Challenge, Domain, ProverData, StarkGenericConfig, Val,
};
use p3_commit::{Pcs, PolynomialSpace};
use p3_field::{Algebra, BasedVectorSpace, PrimeField};
use p3_lookup::folder::ProverConstraintFolderWithLookups;
use p3_lookup::logup::LogUpGadget;
use p3_lookup::{
    InteractionSymbolicBuilder, Lookup, LookupProtocol, LookupTerminal,
    check_multiplicity_height_bound,
};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::util::reverse_matrix_index_bits;
use p3_uni_stark::OpenedValues;
use p3_util::log2_strict_usize;
use tracing::{info_span, instrument};

type InstanceQuotient<SC> = (Vec<Domain<SC>>, Vec<RowMajorMatrix<Val<SC>>>);

/// Canonical lifetime boundary emitted by the optional one-shot resource sink.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OneShotResourceStageV2 {
    Entry,
    PostMainCommit,
    PostLogUpPreTraceDrop,
    PostTraceDrop,
    PostPermutationCommit,
    PostQuotientAir,
    PostQuotientCommit,
    PreOpen,
    PostOpen,
}

impl OneShotResourceStageV2 {
    /// Stable JSON value used by the Phase 069 resource evidence parser.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Entry => "entry",
            Self::PostMainCommit => "post_main_commit",
            Self::PostLogUpPreTraceDrop => "post_logup_pre_trace_drop",
            Self::PostTraceDrop => "post_trace_drop",
            Self::PostPermutationCommit => "post_permutation_commit",
            Self::PostQuotientAir => "post_quotient_air",
            Self::PostQuotientCommit => "post_quotient_commit",
            Self::PreOpen => "pre_open",
            Self::PostOpen => "post_open",
        }
    }
}

/// Exact aggregate allocation size for visible matrix value buffers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OneShotBufferBytesV2 {
    pub len_bytes: usize,
    pub capacity_bytes: usize,
}

impl OneShotBufferBytesV2 {
    fn checked_add(self, other: Self) -> Self {
        Self {
            len_bytes: self
                .len_bytes
                .checked_add(other.len_bytes)
                .expect("one-shot telemetry length bytes fit usize"),
            capacity_bytes: self
                .capacity_bytes
                .checked_add(other.capacity_bytes)
                .expect("one-shot telemetry capacity bytes fit usize"),
        }
    }
}

/// Public-size-only buffer inventory at one one-shot lifetime boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OneShotVisibleBuffersV2 {
    pub main_trace: OneShotBufferBytesV2,
    pub permutation_trace: OneShotBufferBytesV2,
    pub quotient_lde: OneShotBufferBytesV2,
}

/// One schema-stable resource snapshot. Process RSS is supplied by the std host sink.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OneShotResourceSnapshotV2 {
    pub stage: OneShotResourceStageV2,
    pub air_index: Option<usize>,
    pub visible_buffers: OneShotVisibleBuffersV2,
}

/// Host callback for optional process telemetry.
///
/// The prover crate stays `no_std`; the std host owns `/proc` reads and JSON output.
pub trait OneShotResourceTelemetrySinkV2 {
    fn record(&mut self, snapshot: OneShotResourceSnapshotV2);
}

/// Allocation-only lifecycle hooks for AIR-owned preprocessing sources.
///
/// The proof relation and verifier metadata are independent of these hooks.
/// They let materialized callers release deterministic rebuildable state.
/// Allocation-free streaming AIRs implement both hooks as no-ops.
pub(super) trait OneShotAirLifecycleV2 {
    fn release_one_shot_schedule(&mut self);
    fn release_one_shot_preprocessed_source(&mut self);
}

fn matrix_buffer_bytes<F>(matrix: &RowMajorMatrix<F>) -> OneShotBufferBytesV2 {
    OneShotBufferBytesV2 {
        len_bytes: matrix
            .values
            .len()
            .checked_mul(size_of::<F>())
            .expect("allocated matrix length bytes fit usize"),
        capacity_bytes: matrix
            .values
            .capacity()
            .checked_mul(size_of::<F>())
            .expect("allocated matrix capacity bytes fit usize"),
    }
}

fn matrix_buffers<'a, F: 'a>(
    matrices: impl IntoIterator<Item = &'a RowMajorMatrix<F>>,
) -> OneShotBufferBytesV2 {
    matrices
        .into_iter()
        .fold(OneShotBufferBytesV2::default(), |total, matrix| {
            total.checked_add(matrix_buffer_bytes(matrix))
        })
}

#[inline]
fn emit_resource_snapshot(
    telemetry: &mut Option<&mut dyn OneShotResourceTelemetrySinkV2>,
    stage: OneShotResourceStageV2,
    air_index: Option<usize>,
    visible_buffers: impl FnOnce() -> OneShotVisibleBuffersV2,
) {
    if let Some(sink) = telemetry.as_deref_mut() {
        sink.record(OneShotResourceSnapshotV2 {
            stage,
            air_index,
            visible_buffers: visible_buffers(),
        });
    }
}

/// Prove a single-use batch while consuming its main trace matrices.
///
/// This path is intentionally restricted to the pinned non-ZK PCS used by the
/// recursive worker. It changes storage lifetime only; all verifier-visible
/// inputs and transcript operations match the upstream batch prover.
#[instrument(skip_all)]
pub(super) fn prove_batch_one_shot_owned<SC, A>(
    config: &SC,
    airs: &mut [A],
    traces: Vec<RowMajorMatrix<Val<SC>>>,
    public_values: &[Vec<Val<SC>>],
    prover_data: &ProverData<SC>,
    mut telemetry: Option<&mut dyn OneShotResourceTelemetrySinkV2>,
    pre_commit_reclaimer: Option<fn()>,
) -> BatchProof<SC>
where
    SC: StarkGenericConfig,
    A: for<'a> Air<InteractionSymbolicBuilder<Val<SC>, SC::Challenge>>
        + for<'a> Air<ProverConstraintFolderWithLookups<'a, SC>>
        + Clone
        + OneShotAirLifecycleV2,
    Val<SC>: PrimeField,
    SC::Challenge: BasedVectorSpace<Val<SC>>,
    SymbolicExpressionExt<Val<SC>, SC::Challenge>: Algebra<SC::Challenge>,
    Domain<SC>: Send + Sync,
    SC::Pcs: Sync,
    <SC::Pcs as Pcs<SC::Challenge, SC::Challenger>>::ProverData: Sync,
    <SC::Pcs as Pcs<SC::Challenge, SC::Challenger>>::Commitment: Sync,
{
    assert!(
        !SC::Pcs::ZK,
        "one-shot owned proving requires the pinned non-ZK PCS"
    );
    assert_eq!(airs.len(), traces.len());
    assert_eq!(airs.len(), public_values.len());

    // Release deterministic AIR-owned rebuild sources before PCS commits the
    // complete main trace set. LogUp reconstructs committed preprocessing from
    // PCS prover data below, so these sources are not proof inputs here.
    for air in airs.iter_mut() {
        air.release_one_shot_schedule();
        air.release_one_shot_preprocessed_source();
    }
    if let Some(reclaim) = pre_commit_reclaimer {
        reclaim();
    }

    emit_resource_snapshot(&mut telemetry, OneShotResourceStageV2::Entry, None, || {
        OneShotVisibleBuffersV2 {
            main_trace: matrix_buffers(&traces),
            ..Default::default()
        }
    });

    let common = &prover_data.common;
    let lookup_gadget = LogUpGadget::new();
    let pcs = config.pcs();
    let mut transcript = BatchTranscript::<SC>::new(config.initialise_challenger());

    let degrees: Vec<usize> = traces.iter().map(Matrix::height).collect();
    let log_degrees: Vec<usize> = degrees.iter().copied().map(log2_strict_usize).collect();
    let log_ext_degrees: Vec<usize> = log_degrees
        .iter()
        .map(|&degree| degree + config.is_zk())
        .collect();
    check_multiplicity_height_bound(&common.lookups, &degrees)
        .expect("LogUp multiplicity height-bound violated");

    let all_lookups: Vec<&[Lookup<Val<SC>>]> =
        common.lookups.iter().map(|lookups| &**lookups).collect();
    let mut lookup_terminals: Vec<Option<LookupTerminal<SC::Challenge>>> =
        all_lookups.iter().map(|_| None).collect();
    let (trace_domains, ext_trace_domains): (Vec<Domain<SC>>, Vec<Domain<SC>>) = degrees
        .iter()
        .map(|&degree| {
            (
                pcs.natural_domain_for_degree(degree),
                pcs.natural_domain_for_degree(degree * (config.is_zk() + 1)),
            )
        })
        .unzip();
    let pub_vals: Vec<&[Val<SC>]> = public_values.iter().map(Vec::as_slice).collect();

    let mut preprocessed_widths = Vec::with_capacity(airs.len());
    let (log_num_quotient_chunks, num_quotient_chunks): (Vec<usize>, Vec<usize>) = airs
        .iter()
        .enumerate()
        .map(|(index, air)| {
            let preprocessed_width = common
                .preprocessed
                .as_ref()
                .and_then(|global| global.instances[index].as_ref().map(|meta| meta.width))
                .unwrap_or(0);
            preprocessed_widths.push(preprocessed_width);
            let layout = AirLayout {
                preprocessed_width,
                main_width: air.width(),
                num_public_values: air.num_public_values(),
                num_periodic_columns: air.num_periodic_columns(),
                ..Default::default()
            };
            let log_chunks = info_span!("infer log of constraint degree", air_idx = index)
                .in_scope(|| {
                    get_log_num_quotient_chunks::<Val<SC>, SC::Challenge, A, LogUpGadget>(
                        air,
                        layout,
                        all_lookups[index],
                        config.is_zk(),
                        &lookup_gadget,
                    )
                });
            (log_chunks, 1 << (log_chunks + config.is_zk()))
        })
        .unzip();

    transcript.observe_instance_count(airs.len());
    for index in 0..airs.len() {
        transcript.observe_instance_binding(
            log_ext_degrees[index],
            log_degrees[index],
            airs[index].width(),
            num_quotient_chunks[index],
        );
    }

    // Consume the original main matrices into PCS. The committed LDE prover
    // data is sufficient to reconstruct one natural-domain table at a time
    // for LogUp below, so retaining a second complete main-trace set would be
    // a pure allocation overlap with no theorem or transcript purpose.
    let main_commit_inputs = traces
        .into_iter()
        .zip(ext_trace_domains.iter().copied())
        .map(|(trace, domain)| (domain, trace));
    let (main_commit, main_data) = pcs.commit(main_commit_inputs);
    emit_resource_snapshot(
        &mut telemetry,
        OneShotResourceStageV2::PostMainCommit,
        None,
        OneShotVisibleBuffersV2::default,
    );
    transcript.observe_main(&main_commit, &pub_vals);
    transcript.observe_preprocessed(&preprocessed_widths, common.preprocessed.as_ref());

    let challenges_per_instance = transcript.sample_perm_challenges(&all_lookups, &lookup_gadget);
    // Reconstruct at most one committed main table at a time, preserving
    // canonical AIR order and terminal assignment. `get_evaluations_on_domain`
    // derives the exact natural-domain evaluations from the already-bound PCS
    // prover data; no witness or verifier-facing input is regenerated.
    let mut permutation_traces = Vec::with_capacity(
        all_lookups
            .iter()
            .filter(|lookups| !lookups.is_empty())
            .count(),
    );
    for index in 0..airs.len() {
        if all_lookups[index].is_empty() {
            continue;
        }
        let preprocessed = common.preprocessed.as_ref().and_then(|global| {
            global.instances[index].as_ref().map(|meta| {
                let preprocessed_data = prover_data
                    .prover_only
                    .preprocessed_prover_data
                    .as_ref()
                    .expect("preprocessed PCS data exists for committed metadata");
                assert_eq!(
                    meta.degree_bits, log_ext_degrees[index],
                    "preprocessed and main trace degrees must match"
                );
                let mut matrix = pcs
                    .get_evaluations_on_domain(
                        preprocessed_data,
                        meta.matrix_index,
                        trace_domains[index],
                    )
                    .to_row_major_matrix();
                // The PCS exposes its FRI-native bit-reversed row view. LogUp
                // consumes the original natural preprocessing order.
                reverse_matrix_index_bits(&mut matrix);
                matrix
            })
        });
        let mut main_trace = pcs
            .get_evaluations_on_domain(&main_data, index, trace_domains[index])
            .to_row_major_matrix();
        // `Pcs::get_evaluations_on_domain` exposes the FRI-native bit-reversed
        // row view used by quotient evaluation. LogUp consumes the original
        // natural trace order, so undo that row permutation in place.
        reverse_matrix_index_bits(&mut main_trace);
        let (generated, terminal) = lookup_gadget.generate_permutation::<SC>(
            &main_trace,
            &preprocessed,
            public_values[index].as_slice(),
            all_lookups[index],
            &challenges_per_instance[index],
        );
        drop(main_trace);
        lookup_terminals[index] = terminal;
        permutation_traces.push((ext_trace_domains[index], generated));
    }

    emit_resource_snapshot(
        &mut telemetry,
        OneShotResourceStageV2::PostLogUpPreTraceDrop,
        None,
        || OneShotVisibleBuffersV2 {
            permutation_trace: matrix_buffers(permutation_traces.iter().map(|(_, trace)| trace)),
            ..Default::default()
        },
    );

    // No original or reconstructed main matrix remains before permutation PCS
    // allocates its LDE/MMCS working set.
    emit_resource_snapshot(
        &mut telemetry,
        OneShotResourceStageV2::PostTraceDrop,
        None,
        || OneShotVisibleBuffersV2 {
            permutation_trace: matrix_buffers(permutation_traces.iter().map(|(_, trace)| trace)),
            ..Default::default()
        },
    );

    let permutation_commit_and_data = if permutation_traces.is_empty() {
        None
    } else {
        let permutation_commit_inputs =
            permutation_traces.into_iter().map(|(domain, generated)| {
                // `RowMajorMatrix::flatten_to_base` clones the owned extension-field
                // vector first. Flatten the owned values directly so the conversion
                // reuses their allocation and preserves the same row-major layout.
                let width = generated.width() * SC::Challenge::DIMENSION;
                let values = SC::Challenge::flatten_to_base(generated.values);
                (domain, RowMajorMatrix::new(values, width))
            });
        Some(pcs.commit(permutation_commit_inputs))
    };
    emit_resource_snapshot(
        &mut telemetry,
        OneShotResourceStageV2::PostPermutationCommit,
        None,
        OneShotVisibleBuffersV2::default,
    );
    let alpha = transcript.observe_perm_and_sample_alpha(
        permutation_commit_and_data
            .as_ref()
            .map(|(commitment, _)| commitment),
        &lookup_terminals,
    );
    let permutation_data = permutation_commit_and_data.as_ref().map(|(_, data)| data);
    let perm_indices: Vec<usize> = all_lookups
        .iter()
        .scan(0_usize, |next, lookups| {
            let index = *next;
            if !lookups.is_empty() {
                *next += 1;
            }
            Some(index)
        })
        .collect();

    // Quotient/LDE construction is the peak-allocation stage.  The final PCS
    // commitment still consumes the same ordered matrices below, but producing
    // every table's temporary quotient state in parallel transiently holds a
    // full working set per table.  Materialize one instance at a time so the
    // bounded direct-AIR worker has a single temporary quotient working set.
    // This preserves AIR order, transcript observations, quotient domains, and
    // the committed matrix sequence.
    let mut per_instance: Vec<InstanceQuotient<SC>> = Vec::with_capacity(airs.len());
    let mut quotient_lde_buffers = OneShotBufferBytesV2::default();
    for index in 0..airs.len() {
        let _span = info_span!("compute quotient", air_idx = index).entered();
        let log_chunks = log_num_quotient_chunks[index];
        let chunk_count = num_quotient_chunks[index];
        let quotient_domain = ext_trace_domains[index]
            .create_disjoint_domain(1 << (log_ext_degrees[index] + log_chunks));
        let layout = AirLayout {
            preprocessed_width: preprocessed_widths[index],
            main_width: airs[index].width(),
            num_public_values: airs[index].num_public_values(),
            num_periodic_columns: airs[index].num_periodic_columns(),
            ..Default::default()
        };
        debug_assert!(
            airs[index].num_constraints().is_none_or(|expected| {
                expected
                    == get_symbolic_constraints(
                        &airs[index],
                        layout,
                        all_lookups[index],
                        &lookup_gadget,
                    )
                    .0
                    .len()
            }),
            "static constraint count does not match symbolic analysis"
        );

        let trace_on_quotient = pcs.get_evaluations_on_domain(&main_data, index, quotient_domain);
        let permutation_on_quotient = permutation_data
            .filter(|_| !all_lookups[index].is_empty())
            .map(|data| pcs.get_evaluations_on_domain(data, perm_indices[index], quotient_domain));
        let preprocessed_on_quotient = common
            .preprocessed
            .as_ref()
            .and_then(|global| global.instances[index].as_ref())
            .map(|meta| {
                let data = prover_data
                    .prover_only
                    .preprocessed_prover_data
                    .as_ref()
                    .expect("preprocessed commitment requires prover data");
                pcs.get_evaluations_on_domain_no_random(data, meta.matrix_index, quotient_domain)
            });
        let permutation_values: Vec<_> = lookup_terminals[index]
            .iter()
            .map(|terminal| terminal.0)
            .collect();
        let quotient_values = p3_batch_stark::prover::quotient_values(
            pcs,
            &airs[index],
            pub_vals[index],
            layout,
            trace_domains[index],
            quotient_domain,
            &trace_on_quotient,
            permutation_on_quotient.as_ref(),
            all_lookups[index],
            &permutation_values,
            &lookup_gadget,
            &challenges_per_instance[index],
            preprocessed_on_quotient.as_ref(),
            alpha,
        );
        // `RowMajorMatrix::flatten_to_base` clones its backing vector before
        // flattening. The quotient is already owned here, so flatten it directly
        // and retain the same row-major DIMENSION-column layout without a second
        // full extension-field buffer.
        let quotient_flat = RowMajorMatrix::new(
            SC::Challenge::flatten_to_base(quotient_values),
            SC::Challenge::DIMENSION,
        );
        let chunk_matrices = quotient_domain.split_evals(chunk_count, quotient_flat);
        let chunk_domains = quotient_domain.split_domains(chunk_count);
        let evaluations = chunk_domains.iter().copied().zip(chunk_matrices);
        let ldes = pcs.get_quotient_ldes(evaluations, chunk_count);
        if telemetry.is_some() {
            quotient_lde_buffers = quotient_lde_buffers.checked_add(matrix_buffers(&ldes));
        }
        per_instance.push((chunk_domains, ldes));
        emit_resource_snapshot(
            &mut telemetry,
            OneShotResourceStageV2::PostQuotientAir,
            Some(index),
            || OneShotVisibleBuffersV2 {
                quotient_lde: quotient_lde_buffers,
                ..Default::default()
            },
        );
    }

    let mut quotient_chunk_domains = Vec::new();
    let mut quotient_chunk_matrices = Vec::new();
    let mut quotient_chunk_ranges = Vec::with_capacity(airs.len());
    for (domains, matrices) in per_instance {
        let start = quotient_chunk_domains.len();
        quotient_chunk_domains.extend(domains);
        quotient_chunk_matrices.extend(matrices);
        quotient_chunk_ranges.push((start, quotient_chunk_domains.len()));
    }
    let (quotient_commit, quotient_data) = pcs.commit_ldes(quotient_chunk_matrices);
    emit_resource_snapshot(
        &mut telemetry,
        OneShotResourceStageV2::PostQuotientCommit,
        None,
        OneShotVisibleBuffersV2::default,
    );
    transcript.observe_quotient_commitment(&quotient_commit);

    let (random_commit, random_data) = if SC::Pcs::ZK {
        let (commitment, data) = pcs
            .get_opt_randomization_poly_commitment(ext_trace_domains.iter().copied())
            .expect("ZK PCS requires randomization commitments");
        (Some(commitment), Some(data))
    } else {
        (None, None)
    };
    if let Some(commitment) = &random_commit {
        transcript.observe_random_commitment(commitment);
    }
    let zeta: Challenge<SC> = transcript.sample_zeta();

    let (opened_values, opening_proof) = {
        let mut rounds = Vec::new();
        if let Some(data) = random_data.as_ref() {
            rounds.push((data, trace_domains.iter().map(|_| vec![zeta]).collect()));
        }
        let main_points = trace_domains
            .iter()
            .enumerate()
            .map(|(index, domain)| {
                if airs[index].main_next_row_columns().is_empty() {
                    vec![zeta]
                } else {
                    vec![
                        zeta,
                        domain
                            .next_point(zeta)
                            .expect("trace domain supports next point"),
                    ]
                }
            })
            .collect();
        rounds.push((&main_data, main_points));
        let quotient_points = quotient_chunk_ranges
            .iter()
            .copied()
            .flat_map(|(start, end)| (start..end).map(|_| vec![zeta]))
            .collect();
        rounds.push((&quotient_data, quotient_points));

        if let Some(global) = &common.preprocessed {
            let data = prover_data
                .prover_only
                .preprocessed_prover_data
                .as_ref()
                .expect("preprocessed commitment requires prover data");
            let points = global
                .matrix_to_instance
                .iter()
                .map(|&index| {
                    if airs[index].preprocessed_next_row_columns().is_empty() {
                        vec![zeta]
                    } else {
                        vec![
                            zeta,
                            trace_domains[index]
                                .next_point(zeta)
                                .expect("trace domain supports next point"),
                        ]
                    }
                })
                .collect();
            rounds.push((data, points));
        }
        if let Some((_, data)) = &permutation_commit_and_data {
            let points = trace_domains
                .iter()
                .zip(&all_lookups)
                .filter(|(_, lookups)| !lookups.is_empty())
                .map(|(domain, _)| {
                    vec![
                        zeta,
                        domain
                            .next_point(zeta)
                            .expect("trace domain supports next point"),
                    ]
                })
                .collect();
            rounds.push((data, points));
        }
        emit_resource_snapshot(
            &mut telemetry,
            OneShotResourceStageV2::PreOpen,
            None,
            OneShotVisibleBuffersV2::default,
        );
        let opened = pcs.open_with_preprocessing(
            rounds,
            &mut transcript.challenger,
            common.preprocessed.is_some(),
        );
        emit_resource_snapshot(
            &mut telemetry,
            OneShotResourceStageV2::PostOpen,
            None,
            OneShotVisibleBuffersV2::default,
        );
        opened
    };

    let permutation_round_index = if common.preprocessed.is_some() {
        SC::Pcs::PREPROCESSED_TRACE_IDX + 1
    } else {
        SC::Pcs::PREPROCESSED_TRACE_IDX
    };
    let main_openings = &opened_values[SC::Pcs::TRACE_IDX];
    let preprocessed_openings = common
        .preprocessed
        .as_ref()
        .map(|_| &opened_values[SC::Pcs::PREPROCESSED_TRACE_IDX]);
    let empty_permutation_openings = Vec::new();
    let permutation_openings = if permutation_commit_and_data.is_some() {
        &opened_values[permutation_round_index]
    } else {
        &empty_permutation_openings
    };
    let mut permutation_openings = permutation_openings.iter();
    let mut quotient_openings = opened_values[SC::Pcs::QUOTIENT_IDX].iter();
    let mut instance_openings = Vec::with_capacity(airs.len());

    for (index, (start, end)) in quotient_chunk_ranges.iter().copied().enumerate() {
        let main = &main_openings[index];
        let trace_local = main[0].clone();
        let trace_next = if airs[index].main_next_row_columns().is_empty() {
            None
        } else {
            Some(main[1].clone())
        };
        let quotient_chunks = (start..end)
            .map(|_| {
                quotient_openings
                    .next()
                    .expect("quotient chunk opening exists")[0]
                    .clone()
            })
            .collect();
        let (preprocessed_local, preprocessed_next) =
            if let (Some(global), Some(round)) = (&common.preprocessed, preprocessed_openings) {
                global.instances[index]
                    .as_ref()
                    .map_or((None, None), |meta| {
                        let values = &round[meta.matrix_index];
                        if airs[index].preprocessed_next_row_columns().is_empty() {
                            (Some(values[0].clone()), None)
                        } else {
                            (Some(values[0].clone()), Some(values[1].clone()))
                        }
                    })
            } else {
                (None, None)
            };
        let (permutation_local, permutation_next) = if all_lookups[index].is_empty() {
            (Vec::new(), Vec::new())
        } else {
            let values = permutation_openings
                .next()
                .expect("lookup permutation opening exists");
            (values[0].clone(), values[1].clone())
        };
        instance_openings.push(OpenedValuesWithLookups {
            base_opened_values: OpenedValues {
                trace_local,
                trace_next,
                preprocessed_local,
                preprocessed_next,
                quotient_chunks,
                random: random_data
                    .as_ref()
                    .map(|_| opened_values[0][index][0].clone()),
            },
            permutation_local,
            permutation_next,
        });
    }

    BatchProof {
        commitments: BatchCommitments {
            main: main_commit,
            quotient_chunks: quotient_commit,
            random: random_commit,
            permutation: permutation_commit_and_data
                .as_ref()
                .map(|(commitment, _)| commitment.clone()),
        },
        opened_values: BatchOpenedValues {
            instances: instance_openings,
        },
        opening_proof,
        lookup_terminals,
        degree_bits: log_ext_degrees,
    }
}
