//! Proof-bound transition and typed-event tables for one bounded epoch chunk.
//!
//! This artifact is the first canonical multi-table Batch-STARK in the streamed
//! epoch path. The transition table supplies the expected typed commitments,
//! while independently materialized typed-event rows supply and consume the
//! matching LogUp multisets. Private event bytes never enter the artifact.

use p3_circuit::tables::{
    AluTrace, ConstTrace, NonPrimitiveTrace, PublicTrace, Traces, WitnessTrace,
};
use p3_koala_bear::KoalaBear;
use z00z_crypto::sha256_256;
use z00z_plonky3_circuit_prover::batch_stark_prover::TablePacking;
use z00z_plonky3_circuit_prover::{BatchStarkProof, BatchStarkProver};

use super::plonky3_epoch_trace_framing as trace_framing;
use super::plonky3_epoch_trace_framing_air::{
    TraceFramingAirRoleV2, TraceFramingProverV2, TraceFramingTraceV2,
    ROWS_V2 as TRACE_FRAMING_ROWS_V2,
};
use super::plonky3_epoch_transition_air::{
    npo_type as transition_npo_type, TransitionProverV2, TransitionTraceV2,
    ROWS_V2 as TRANSITION_ROWS_V2,
};
use super::plonky3_epoch_transition_witness as transition_witness;
use super::plonky3_epoch_typed_commitment as typed_commitment;
use super::plonky3_epoch_typed_commitment_air::{
    TypedCommitmentAirRoleV2, TypedCommitmentProverV2, TypedCommitmentTraceV2,
    COMMITMENTS_PER_TRANSITION_V2, ROWS_V2 as TYPED_ROWS_V2,
};
use super::plonky3_epoch_uniqueness_air::{UniquenessAirRoleV2, UniquenessProverV2, ROLE_COUNT_V2};
use super::plonky3_epoch_uniqueness_range::{
    self as uniqueness_range, UniquenessRangeProverV2, UniquenessRangeTraceV2,
};
use super::plonky3_epoch_uniqueness_witness::{
    self as uniqueness_witness, ParsedUniquenessWitnessV2, UniquenessAirWitnessV2,
};
use super::{
    decode_canonical_batch_proof_v2, encode_canonical_batch_proof_v2, hardened_koala_bear_config,
    EpochAirTableV2, EpochPreparedTransitionV2, EpochTraceChunkInputsV2, EpochTraceChunkV2,
    EpochTransitionBindingV2, Plonky3StarkConfigV2, RecursiveCheckpointRejectReasonV2,
};
use crate::CheckpointError;

const PROOF_DOMAIN_V2: &str = "z00z.storage.checkpoint.plonky3.epoch-transition-batch.v2";
const PROOF_LABEL_V2: &str = "actual_verified_linked_tables";
const TABLE_COUNT_V2: usize = 4 + ROLE_COUNT_V2 + 1;

/// One actual-verified Batch-STARK joining transition and typed-event tables.
#[derive(Clone, Debug)]
pub struct Plonky3EpochTransitionBatchV2 {
    transition_statement: EpochTraceChunkV2,
    trace_framing_statement: EpochTraceChunkV2,
    typed_statement: EpochTraceChunkV2,
    uniqueness_statement: EpochTraceChunkV2,
    uniqueness_range_query_count: u64,
    bindings: Vec<EpochTransitionBindingV2>,
    proof_digest: [u8; 32],
    proof_bytes: Vec<u8>,
}

impl Plonky3EpochTransitionBatchV2 {
    #[must_use]
    pub const fn transition_statement(&self) -> &EpochTraceChunkV2 {
        &self.transition_statement
    }

    #[must_use]
    pub const fn trace_framing_statement(&self) -> &EpochTraceChunkV2 {
        &self.trace_framing_statement
    }

    #[must_use]
    pub const fn typed_statement(&self) -> &EpochTraceChunkV2 {
        &self.typed_statement
    }

    #[must_use]
    pub const fn uniqueness_statement(&self) -> &EpochTraceChunkV2 {
        &self.uniqueness_statement
    }

    #[must_use]
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    #[must_use]
    pub fn local_proof_bytes(&self) -> &[u8] {
        &self.proof_bytes
    }

    pub fn trace_row_count(&self) -> Result<usize, CheckpointError> {
        decode_canonical_batch_proof_v2(&self.proof_bytes)?
            .non_primitives
            .iter()
            .try_fold(0_usize, |total, entry| {
                total
                    .checked_add(entry.rows)
                    .ok_or(CheckpointError::Overflow)
            })
    }

    #[must_use]
    pub const fn table_count(&self) -> usize {
        TABLE_COUNT_V2
    }

    pub fn verify(&self) -> Result<(), CheckpointError> {
        validate_statements(
            &self.transition_statement,
            &self.trace_framing_statement,
            &self.typed_statement,
            &self.uniqueness_statement,
            &self.bindings,
        )?;
        let expected_digest = proof_digest(
            &self.transition_statement,
            &self.trace_framing_statement,
            &self.typed_statement,
            &self.uniqueness_statement,
            self.uniqueness_range_query_count,
            &self.bindings,
            &self.proof_bytes,
        )?;
        if self.proof_digest == [0; 32] || self.proof_digest != expected_digest {
            return Err(CheckpointError::Canonical);
        }
        let proof = decode_canonical_batch_proof_v2(&self.proof_bytes)?;
        let event_bytes = binding_event_bytes(&self.bindings)?;
        verify_proof(
            &proof,
            &transition_witness::public_values(&self.transition_statement, &self.bindings)?,
            &trace_framing::public_values(
                &self.trace_framing_statement,
                &self.bindings,
                event_bytes,
            )?,
            &typed_public_values(&self.typed_statement, &self.bindings)?,
            &uniqueness_witness::public_values(&self.uniqueness_statement)?,
            &uniqueness_range::public_values(
                &self.uniqueness_statement,
                usize::try_from(self.uniqueness_range_query_count)
                    .map_err(|_| CheckpointError::Limit)?,
            )?,
        )
    }
}

fn typed_public_values(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
) -> Result<Vec<KoalaBear>, CheckpointError> {
    let commitments = bindings
        .iter()
        .map(EpochTransitionBindingV2::typed_commitment_digests)
        .collect::<Vec<_>>();
    typed_commitment::public_values(statement, &commitments)
}

fn validate_statements(
    transition: &EpochTraceChunkV2,
    trace_framing: &EpochTraceChunkV2,
    typed: &EpochTraceChunkV2,
    uniqueness: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
) -> Result<(), CheckpointError> {
    let binding_count = u32::try_from(bindings.len()).map_err(|_| CheckpointError::Limit)?;
    if binding_count == 0 || binding_count > super::EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let transition_inputs = transition.inputs();
    let first = bindings
        .first()
        .copied()
        .ok_or(CheckpointError::Canonical)?
        .ordinal();
    let last = bindings
        .last()
        .copied()
        .ok_or(CheckpointError::Canonical)?
        .ordinal();
    let transition_count = u64::from(binding_count);
    let typed_row_start = u64::from(first)
        .checked_mul(COMMITMENTS_PER_TRANSITION_V2 as u64)
        .ok_or(CheckpointError::Overflow)?;
    let typed_row_count = transition_count
        .checked_mul(COMMITMENTS_PER_TRANSITION_V2 as u64)
        .ok_or(CheckpointError::Overflow)?;
    if transition_inputs.table != EpochAirTableV2::Transition
        || transition_inputs.replica != 0
        || transition_inputs.first_transition != first
        || transition_inputs.last_transition != last
        || transition_inputs.row_start != u64::from(first)
        || transition_inputs.row_count != transition_count
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let expected_trace_framing_inputs = EpochTraceChunkInputsV2 {
        table: EpochAirTableV2::TraceFraming,
        row_start: u64::from(first),
        row_count: transition_count,
        ..transition_inputs
    };
    if trace_framing.inputs() != expected_trace_framing_inputs {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let expected_typed_inputs = EpochTraceChunkInputsV2 {
        table: EpochAirTableV2::TypedCommitment,
        row_start: typed_row_start,
        row_count: typed_row_count,
        ..transition_inputs
    };
    if typed.inputs() != expected_typed_inputs {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let expected_uniqueness_inputs = EpochTraceChunkInputsV2 {
        table: EpochAirTableV2::Uniqueness,
        row_start: transition_inputs.event_start,
        row_count: uniqueness.inputs().row_count,
        ..transition_inputs
    };
    if uniqueness.inputs() != expected_uniqueness_inputs
        || uniqueness.inputs().row_count > uniqueness.inputs().event_count
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

fn binding_event_bytes(bindings: &[EpochTransitionBindingV2]) -> Result<u64, CheckpointError> {
    bindings.iter().try_fold(0_u64, |total, binding| {
        total
            .checked_add(binding.inputs().event_bytes)
            .ok_or(CheckpointError::Overflow)
    })
}

fn direct_traces(
    transition_rows: Vec<super::plonky3_epoch_transition_air::TransitionRowV2>,
    trace_framing_rows: Vec<super::plonky3_epoch_trace_framing_air::TraceFramingRowV2>,
    typed_rows: Vec<super::plonky3_epoch_typed_commitment_air::TypedCommitmentRowV2>,
    uniqueness: UniquenessAirWitnessV2,
    uniqueness_range_rows: Vec<uniqueness_range::UniquenessRangeRowV2>,
) -> Traces<KoalaBear> {
    let typed_source_rows = typed_rows.clone();
    let mut non_primitive_traces = vec![
        (
            transition_npo_type(),
            Box::new(TransitionTraceV2 {
                rows: transition_rows,
            }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            TraceFramingAirRoleV2::LinkedConsumer.npo_type(),
            Box::new(TraceFramingTraceV2 {
                role: TraceFramingAirRoleV2::LinkedConsumer,
                rows: trace_framing_rows,
            }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            TypedCommitmentAirRoleV2::LinkedConsumer.npo_type(),
            Box::new(TypedCommitmentTraceV2 {
                role: TypedCommitmentAirRoleV2::LinkedConsumer,
                rows: typed_rows,
            }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            TypedCommitmentAirRoleV2::EventSource.npo_type(),
            Box::new(TypedCommitmentTraceV2 {
                role: TypedCommitmentAirRoleV2::EventSource,
                rows: typed_source_rows,
            }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
    ];
    non_primitive_traces.extend(uniqueness.traces.into_iter().map(|trace| {
        (
            trace.role.npo_type(),
            Box::new(trace) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        )
    }));
    non_primitive_traces.push((
        uniqueness_range::npo_type(),
        Box::new(UniquenessRangeTraceV2 {
            rows: uniqueness_range_rows,
        }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
    ));
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
        non_primitive_traces: non_primitive_traces.into_iter().collect(),
        tag_to_witness: Default::default(),
    }
}

fn configured_prover(table_packing: TablePacking) -> BatchStarkProver<Plonky3StarkConfigV2> {
    let mut prover = BatchStarkProver::new(hardened_koala_bear_config())
        .with_table_packing(table_packing)
        .with_debug_lookups();
    prover.register_table_prover(Box::new(TransitionProverV2));
    prover.register_table_prover(Box::new(TraceFramingProverV2::new(
        TraceFramingAirRoleV2::LinkedConsumer,
    )));
    prover.register_table_prover(Box::new(TypedCommitmentProverV2::new(
        TypedCommitmentAirRoleV2::LinkedConsumer,
    )));
    prover.register_table_prover(Box::new(TypedCommitmentProverV2::new(
        TypedCommitmentAirRoleV2::EventSource,
    )));
    for role in UniquenessAirRoleV2::ALL {
        prover.register_table_prover(Box::new(UniquenessProverV2::new(role)));
    }
    prover.register_table_prover(Box::new(UniquenessRangeProverV2));
    prover
}

fn exact_public_values<'a>(
    proof: &'a BatchStarkProof<Plonky3StarkConfigV2>,
    op_type: p3_circuit::ops::NpoTypeId,
) -> Result<&'a [KoalaBear], CheckpointError> {
    let mut entries = proof
        .non_primitives
        .iter()
        .filter(|entry| entry.op_type == op_type);
    let values = entries
        .next()
        .map(|entry| entry.public_values.as_slice())
        .ok_or(CheckpointError::BackendVerificationFailed)?;
    if entries.next().is_some() {
        return Err(CheckpointError::BackendVerificationFailed);
    }
    Ok(values)
}

fn verify_proof(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    expected_transition_public: &[KoalaBear],
    expected_trace_framing_public: &[KoalaBear],
    expected_typed_public: &[KoalaBear],
    expected_uniqueness_public: &[KoalaBear],
    expected_uniqueness_range_public: &[KoalaBear],
) -> Result<(), CheckpointError> {
    let table_packing = TablePacking::new(1, 1).with_min_trace_height(TRACE_FRAMING_ROWS_V2);
    let verifier = configured_prover(table_packing);
    verifier
        .verify_all_tables::<KoalaBear>(proof)
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 epoch transition Batch-STARK verifier rejected proof: {error}"
            ))
        })?;
    let uniqueness_public_matches = UniquenessAirRoleV2::ALL.iter().try_fold(
        true,
        |matches, role| -> Result<bool, CheckpointError> {
            Ok(matches
                && exact_public_values(proof, role.npo_type())? == expected_uniqueness_public)
        },
    )?;
    if proof.non_primitives.len() != TABLE_COUNT_V2
        || exact_public_values(proof, transition_npo_type())? != expected_transition_public
        || exact_public_values(proof, TraceFramingAirRoleV2::LinkedConsumer.npo_type())?
            != expected_trace_framing_public
        || exact_public_values(proof, TypedCommitmentAirRoleV2::LinkedConsumer.npo_type())?
            != expected_typed_public
        || exact_public_values(proof, TypedCommitmentAirRoleV2::EventSource.npo_type())?
            != expected_typed_public
        || !uniqueness_public_matches
        || exact_public_values(proof, uniqueness_range::npo_type())?
            != expected_uniqueness_range_public
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

fn proof_digest(
    transition_statement: &EpochTraceChunkV2,
    trace_framing_statement: &EpochTraceChunkV2,
    typed_statement: &EpochTraceChunkV2,
    uniqueness_statement: &EpochTraceChunkV2,
    uniqueness_range_query_count: u64,
    bindings: &[EpochTransitionBindingV2],
    proof_bytes: &[u8],
) -> Result<[u8; 32], CheckpointError> {
    let mut binding_digests = Vec::with_capacity(
        bindings
            .len()
            .checked_mul(32)
            .ok_or(CheckpointError::Overflow)?,
    );
    for binding in bindings {
        binding_digests.extend_from_slice(&binding.digest());
    }
    Ok(sha256_256(
        PROOF_DOMAIN_V2,
        PROOF_LABEL_V2,
        &[
            &transition_statement.digest(),
            &trace_framing_statement.digest(),
            &typed_statement.digest(),
            &uniqueness_statement.digest(),
            &uniqueness_range_query_count.to_le_bytes(),
            &binding_digests,
            proof_bytes,
        ],
    ))
}

pub(super) fn prove_transition_batch(
    transition_statement: EpochTraceChunkV2,
    trace_framing_statement: EpochTraceChunkV2,
    typed_statement: EpochTraceChunkV2,
    uniqueness_statement: EpochTraceChunkV2,
    bindings: Vec<EpochTransitionBindingV2>,
    prepared: &[EpochPreparedTransitionV2],
    parsed_uniqueness: ParsedUniquenessWitnessV2,
) -> Result<Plonky3EpochTransitionBatchV2, CheckpointError> {
    validate_statements(
        &transition_statement,
        &trace_framing_statement,
        &typed_statement,
        &uniqueness_statement,
        &bindings,
    )?;
    if bindings.len() != prepared.len() {
        return Err(CheckpointError::Invariant);
    }
    let transition_public = transition_witness::public_values(&transition_statement, &bindings)?;
    let event_bytes = binding_event_bytes(&bindings)?;
    let trace_framing_public =
        trace_framing::public_values(&trace_framing_statement, &bindings, event_bytes)?;
    let typed_public = typed_public_values(&typed_statement, &bindings)?;
    let uniqueness_public = uniqueness_witness::public_values(&uniqueness_statement)?;
    let uniqueness = uniqueness_witness::air_witness(&uniqueness_statement, &parsed_uniqueness)?;
    let uniqueness_range_query_count =
        u64::try_from(uniqueness.range_queries.len()).map_err(|_| CheckpointError::Limit)?;
    let uniqueness_range_public =
        uniqueness_range::public_values(&uniqueness_statement, uniqueness.range_queries.len())?;
    let uniqueness_range_rows =
        uniqueness_range::rows(&uniqueness_statement, &uniqueness.range_queries)?;
    let transition_rows = transition_witness::rows(&transition_statement, &bindings)?;
    let trace_framing_rows = trace_framing::rows(&trace_framing_statement, &bindings, event_bytes)?;
    if std::env::var_os("Z00Z_PLONKY3_RESOURCE_TELEMETRY").is_some() {
        super::plonky3_epoch_transition_air::check_constraints(
            &transition_rows,
            &transition_public,
        );
        for trace in &uniqueness.traces {
            super::plonky3_epoch_uniqueness_air::check_constraints(trace, &uniqueness_public);
        }
        uniqueness_range::check_constraints(&uniqueness_range_rows, &uniqueness_range_public);
    }
    let traces = direct_traces(
        transition_rows,
        trace_framing_rows,
        typed_commitment::rows(&typed_statement, &bindings, prepared)?,
        uniqueness,
        uniqueness_range_rows,
    );
    let table_packing = TablePacking::new(1, 1).with_min_trace_height(TRACE_FRAMING_ROWS_V2);
    let prover = configured_prover(table_packing);
    let proof = prover.prove_direct_tables(&traces).map_err(|error| {
        CheckpointError::Backend(format!(
            "Plonky3 epoch transition Batch-STARK prove failed: {error}"
        ))
    })?;
    drop(traces);
    verify_proof(
        &proof,
        &transition_public,
        &trace_framing_public,
        &typed_public,
        &uniqueness_public,
        &uniqueness_range_public,
    )?;
    let proof_bytes = encode_canonical_batch_proof_v2(&proof)?;
    let proof_digest = proof_digest(
        &transition_statement,
        &trace_framing_statement,
        &typed_statement,
        &uniqueness_statement,
        uniqueness_range_query_count,
        &bindings,
        &proof_bytes,
    )?;
    let artifact = Plonky3EpochTransitionBatchV2 {
        transition_statement,
        trace_framing_statement,
        typed_statement,
        uniqueness_statement,
        uniqueness_range_query_count,
        bindings,
        proof_digest,
        proof_bytes,
    };
    artifact.verify()?;
    Ok(artifact)
}

const _: () = assert!(TRANSITION_ROWS_V2 <= TYPED_ROWS_V2);
const _: () = assert!(TRACE_FRAMING_ROWS_V2 <= TRANSITION_ROWS_V2);
