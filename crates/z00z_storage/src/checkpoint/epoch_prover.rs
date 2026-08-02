//! Canonical work and trace-chunk contracts for the production epoch prover.
//!
//! This module owns orchestration identities only. Pinned Plonky3 proving and
//! verification remain in `checkpoint::plonky3`; witness and event bytes are
//! never encoded into either durable contract below.

use z00z_crypto::sha256_256;

use super::{
    canonical_transition::CanonicalCheckpointTransitionV2,
    contract_config_v3::CheckpointConfigResolverV3,
    epoch_frontier::{EpochFrontierAuthorityInputsV2, EpochFrontierAuthorityV2},
    epoch_range::{epoch_ordered_digest_root_v2, EpochCadenceClassV2, EpochCodecReaderV2},
    plonky3::{
        base_native_predicate_digest_v2, transition_material, Plonky3HistoryAuthorityResolverV2,
        RecursiveSecurityBudgetManifestV2, ResolvedPlonky3HistoryAuthorityV2, TransitionMaterialV2,
    },
    recursive_circuit::RecursiveCircuitProfileV2,
    recursive_context::RecursiveAuthoritySnapshotV2,
    recursive_reject::RecursiveCheckpointRejectReasonV2,
};
use crate::{settlement::SettlementStore, CheckpointError};

const EPOCH_WORK_MAGIC_V2: [u8; 8] = *b"Z00ZEWM2";
const EPOCH_WORK_WIRE_V2: u16 = 2;
pub(super) const EPOCH_DIRECT_AIR_GENERATION_V2: u16 = 6;
pub(super) const EPOCH_CHUNK_GRAMMAR_GENERATION_V2: u16 = 5;
const EPOCH_WORK_MAX_TRANSITIONS_V2: u32 = 4_096;
const EPOCH_TRANSITION_DIGEST_COUNT_V2: usize = 25;
const EPOCH_TRANSITION_COUNT_FIELD_COUNT_V2: usize = 6;
const EPOCH_TRANSITION_OPTION_MARKER_COUNT_V2: usize = 2;
const EPOCH_TRANSITION_BYTES_V2: usize = 4
    + 8
    + EPOCH_TRANSITION_OPTION_MARKER_COUNT_V2
    + EPOCH_TRANSITION_DIGEST_COUNT_V2 * 32
    + 4
    + EPOCH_TRANSITION_COUNT_FIELD_COUNT_V2 * 8;
pub(super) const EPOCH_TRANSITION_BINDING_BYTES_V2: usize = EPOCH_TRANSITION_BYTES_V2 + 32;
const EPOCH_WORK_HEADER_DIGEST_COUNT_V2: usize = 13;
const EPOCH_WORK_HEADER_BYTES_V2: usize =
    8 + 2 + 1 + 2 + 2 + 8 * 3 + 4 * 2 + 2 + 8 * 2 + EPOCH_WORK_HEADER_DIGEST_COUNT_V2 * 32 + 1 + 32;
const EPOCH_WORK_TRAILER_BYTES_V2: usize = 32;
const EPOCH_TRANSITION_DOMAIN_V2: &str = "z00z.storage.checkpoint.epoch.transition-bindings.v2";
const EPOCH_TRANSITION_LABEL_V2: &str = "transition_binding";
const EPOCH_WORK_DOMAIN_V2: &str = "z00z.storage.checkpoint.epoch.work-manifest.v2";
const EPOCH_WORK_LABEL_V2: &str = "canonical_manifest";

const EPOCH_CHUNK_MAGIC_V2: [u8; 8] = *b"Z00ZETC2";
const EPOCH_CHUNK_WIRE_V2: u16 = 2;
const EPOCH_CHUNK_DIGEST_COUNT_V2: usize = 11;
const EPOCH_CHUNK_SCALAR_PREFIX_BYTES_V2: usize =
    8 + 2 + 2 + 1 + 1 + 5 * core::mem::size_of::<u32>() + 4 * core::mem::size_of::<u64>();
const EPOCH_CHUNK_INPUT_STATE_ROOT_BYTE_OFFSET_V2: usize =
    EPOCH_CHUNK_SCALAR_PREFIX_BYTES_V2 + 3 * 32;
const EPOCH_CHUNK_OUTPUT_STATE_ROOT_BYTE_OFFSET_V2: usize =
    EPOCH_CHUNK_INPUT_STATE_ROOT_BYTE_OFFSET_V2 + 32;
pub(super) const EPOCH_CHUNK_INPUT_STATE_ROOT_LIMB_OFFSET_V2: usize =
    EPOCH_CHUNK_INPUT_STATE_ROOT_BYTE_OFFSET_V2 / core::mem::size_of::<u16>();
pub(super) const EPOCH_CHUNK_OUTPUT_STATE_ROOT_LIMB_OFFSET_V2: usize =
    EPOCH_CHUNK_OUTPUT_STATE_ROOT_BYTE_OFFSET_V2 / core::mem::size_of::<u16>();
pub(super) const EPOCH_CHUNK_BYTES_V2: usize =
    8 + 2 + 2 + 1 + 1 + 4 * 5 + 8 * 4 + EPOCH_CHUNK_DIGEST_COUNT_V2 * 32 + 32;
const EPOCH_CHUNK_DOMAIN_V2: &str = "z00z.storage.checkpoint.epoch.trace-chunk.v2";
const EPOCH_CHUNK_LABEL_V2: &str = "canonical_statement";
pub(super) const EPOCH_TRANSITION_SLICE_DOMAIN_V2: &str =
    "z00z.storage.checkpoint.epoch.transition-slice.v2";
pub(super) const EPOCH_STREAM_ACCUMULATOR_DOMAIN_V2: &str =
    "z00z.storage.checkpoint.epoch.stream-accumulator.v2";
pub(super) const EPOCH_STREAM_INITIAL_LABEL_V2: &str = "initial";
pub(super) const EPOCH_STREAM_STEP_LABEL_V2: &str = "transition";

/// Fixed generation-bound transition span of one production proving work item.
///
/// A static span makes the complete chunk count knowable at epoch start, so an
/// early proof never depends on a close-only manifest digest or a future
/// data-dependent partition. Eight finalized transitions arrive every forty
/// seconds at the five-second production cadence and cap private material to
/// one bounded worker item.
pub const EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2: u32 = 8;

/// Public, non-secret binding for one canonical transition in an epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochTransitionInputsV2 {
    pub ordinal: u32,
    pub height: u64,
    pub checkpoint_id: [u8; 32],
    pub predecessor: Option<[u8; 32]>,
    pub recursive_transition_statement_digest: [u8; 32],
    pub checkpoint_exec_tx_root: [u8; 32],
    pub checkpoint_exec_tx_count: u32,
    pub checkpoint_statement_digest: [u8; 32],
    pub checkpoint_statement_core_digest: [u8; 32],
    pub checkpoint_link_digest: [u8; 32],
    pub checkpoint_artifact_digest: [u8; 32],
    pub delta_root: [u8; 32],
    pub witness_root: [u8; 32],
    pub journal_digest: [u8; 32],
    pub challenge_content_digest: [u8; 32],
    pub da_payload_commitment: [u8; 32],
    pub prior_recursive_output_root: Option<[u8; 32]>,
    pub pre_settlement_root: [u8; 32],
    pub post_settlement_root: [u8; 32],
    pub pre_definition_root: [u8; 32],
    pub post_definition_root: [u8; 32],
    pub trace_digest: [u8; 32],
    pub update_trace_digest: [u8; 32],
    pub declared_work_digest: [u8; 32],
    pub pre_uniqueness_context_digest: [u8; 32],
    pub spent_uniqueness_precommit: [u8; 32],
    pub output_uniqueness_precommit: [u8; 32],
    pub event_vector_digest: [u8; 32],
    pub event_count: u64,
    pub event_bytes: u64,
    pub uniqueness_row_count: u64,
    pub jmt_record_count: u64,
    pub jmt_envelope_count: u64,
    pub jmt_update_count: u64,
}

/// Content-addressed transition record retained by the immutable work manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochTransitionBindingV2 {
    inputs: EpochTransitionInputsV2,
    digest: [u8; 32],
}

impl EpochTransitionBindingV2 {
    pub fn new(inputs: EpochTransitionInputsV2) -> Result<Self, CheckpointError> {
        validate_transition(&inputs)?;
        let bytes = encode_transition(&inputs);
        let digest = sha256_256(
            EPOCH_TRANSITION_DOMAIN_V2,
            EPOCH_TRANSITION_LABEL_V2,
            &[&bytes],
        );
        Ok(Self { inputs, digest })
    }

    #[must_use]
    pub const fn inputs(&self) -> EpochTransitionInputsV2 {
        self.inputs
    }

    #[must_use]
    pub const fn ordinal(&self) -> u32 {
        self.inputs.ordinal
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.inputs.height
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub(super) const fn typed_commitment_digests(&self) -> [[u8; 32]; 4] {
        [
            self.inputs.delta_root,
            self.inputs.witness_root,
            self.inputs.journal_digest,
            self.inputs.checkpoint_link_digest,
        ]
    }

    pub(super) fn encode_canonical(&self) -> Vec<u8> {
        let mut bytes = encode_transition(&self.inputs);
        bytes.extend_from_slice(&self.digest);
        debug_assert_eq!(bytes.len(), EPOCH_TRANSITION_BINDING_BYTES_V2);
        bytes
    }

    pub(super) fn decode_canonical(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() != EPOCH_TRANSITION_BINDING_BYTES_V2 {
            return Err(CheckpointError::Canonical);
        }
        let mut reader = EpochCodecReaderV2::new(bytes);
        let binding = decode_transition(&mut reader)?;
        let digest: [u8; 32] = reader.array()?;
        if !reader.is_done() || digest != binding.digest || binding.encode_canonical() != bytes {
            return Err(CheckpointError::Canonical);
        }
        Ok(binding)
    }
}

/// Opaque, one-transition proving material produced from the native evaluator.
///
/// Public callers can inspect only the non-secret binding. Canonical event
/// bytes stay process-local, are never serialized, and are zeroized when this
/// value is dropped.
pub struct EpochPreparedTransitionV2 {
    binding: EpochTransitionBindingV2,
    pub(super) material: TransitionMaterialV2,
}

impl EpochPreparedTransitionV2 {
    #[must_use]
    pub const fn binding(&self) -> EpochTransitionBindingV2 {
        self.binding
    }

    #[must_use]
    pub const fn event_count(&self) -> u64 {
        self.binding.inputs.event_count
    }

    #[must_use]
    pub fn event_bytes(&self) -> u64 {
        u64::try_from(self.material.event_vector.len())
            .expect("canonical event material length always fits u64")
    }
}

/// Sole production orchestration entrypoint for one epoch-wide transition
/// stream. It retains only public bindings; private event bytes leave through
/// bounded chunk proving and never accumulate across the epoch.
pub struct EpochTransitionStreamV2 {
    authority: EpochFrontierAuthorityV2,
    transitions: Vec<EpochTransitionBindingV2>,
    pending: Vec<EpochPreparedTransitionV2>,
    total_chunk_count: u32,
    next_chunk_ordinal: u32,
    pending_event_start: u64,
    pending_input_accumulator: [u8; 32],
    running_accumulator: [u8; 32],
}

/// Linear private-material capture prepared before Nova closes the canonical
/// transition source and committed to exactly one unchanged epoch stream.
pub(crate) struct EpochPreparedTransitionCaptureV2 {
    authority_digest: [u8; 32],
    first_ordinal: u32,
    prepared: Vec<EpochPreparedTransitionV2>,
}

impl EpochPreparedTransitionCaptureV2 {
    pub(crate) fn transition_count(&self) -> usize {
        self.prepared.len()
    }
}

impl EpochTransitionStreamV2 {
    pub fn resolve_active(
        store: &SettlementStore,
        cadence_class: EpochCadenceClassV2,
        epoch_index: u64,
        cadence_blocks: u64,
    ) -> Result<Self, CheckpointError> {
        let transition_authority = RecursiveAuthoritySnapshotV2::resolve_active_authority(store)?;
        let context = transition_authority.authority();
        let profile = RecursiveCircuitProfileV2::authority_pinned();
        let predicate_digest = base_native_predicate_digest_v2(context, &profile)?;
        let history = Plonky3HistoryAuthorityResolverV2::resolve_active()?;
        let history_identity = history.identity();
        let authority = EpochFrontierAuthorityV2::new(EpochFrontierAuthorityInputsV2 {
            cadence_class,
            epoch_index,
            cadence_blocks,
            start_root: *transition_authority.snapshot().root().as_bytes(),
            chain_context_digest: context.digest(),
            predicate_digest,
            parameter_digest: history_identity.verifier_parameter_digest,
            verifier_bundle_digest: history_identity.verifier_bundle_digest,
        })?;
        let total_chunk_count = epoch_trace_chunk_count(authority.transition_count())?;
        let initial_accumulator = epoch_stream_initial_accumulator(authority);
        Ok(Self {
            authority,
            transitions: Vec::with_capacity(
                usize::try_from(authority.transition_count())
                    .map_err(|_| CheckpointError::Limit)?,
            ),
            pending: Vec::with_capacity(
                usize::try_from(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2)
                    .map_err(|_| CheckpointError::Limit)?,
            ),
            total_chunk_count,
            next_chunk_ordinal: 0,
            pending_event_start: 0,
            pending_input_accumulator: initial_accumulator,
            running_accumulator: initial_accumulator,
        })
    }

    #[must_use]
    pub const fn authority(&self) -> EpochFrontierAuthorityV2 {
        self.authority
    }

    #[must_use]
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }

    #[must_use]
    pub const fn total_chunk_count(&self) -> u32 {
        self.total_chunk_count
    }

    #[must_use]
    pub const fn emitted_chunk_count(&self) -> u32 {
        self.next_chunk_ordinal
    }

    /// Evaluate and bind exactly one next finalized transition. This function
    /// never invokes the Plan-07 base prover. Once the fixed generation-bound
    /// span is complete it moves all private material into exactly one bounded
    /// work item; the stream never retains prior-chunk witness/event bytes.
    pub fn append(
        &mut self,
        transition: &mut CanonicalCheckpointTransitionV2,
        store: &SettlementStore,
    ) -> Result<Option<EpochTraceChunkWorkV2>, CheckpointError> {
        let capture = self.prepare_capture(std::slice::from_mut(transition), store)?;
        let mut completed = self.append_capture(capture)?;
        if completed.len() > 1 {
            return Err(CheckpointError::Invariant);
        }
        Ok(completed.pop())
    }

    pub(crate) fn prepare_capture(
        &self,
        transitions: &mut [CanonicalCheckpointTransitionV2],
        store: &SettlementStore,
    ) -> Result<EpochPreparedTransitionCaptureV2, CheckpointError> {
        if transitions.is_empty() {
            return Err(range_missing());
        }
        let first_ordinal =
            u32::try_from(self.transitions.len()).map_err(|_| CheckpointError::Limit)?;
        let requested = u32::try_from(transitions.len()).map_err(|_| CheckpointError::Limit)?;
        if first_ordinal
            .checked_add(requested)
            .filter(|end| *end <= self.authority.transition_count())
            .is_none()
        {
            return Err(range_missing());
        }
        let mut previous = self.transitions.last().copied();
        let mut prepared = Vec::with_capacity(transitions.len());
        for (offset, transition) in transitions.iter_mut().enumerate() {
            let ordinal = first_ordinal
                .checked_add(u32::try_from(offset).map_err(|_| CheckpointError::Limit)?)
                .ok_or(CheckpointError::Overflow)?;
            let material = transition_material(transition, store)?;
            let next = prepare_epoch_transition(self.authority, ordinal, material)?;
            self.validate_prepared_link(ordinal, &next, previous)?;
            previous = Some(next.binding);
            prepared.push(next);
        }
        Ok(EpochPreparedTransitionCaptureV2 {
            authority_digest: self.authority.digest(),
            first_ordinal,
            prepared,
        })
    }

    pub(crate) fn append_capture(
        &mut self,
        capture: EpochPreparedTransitionCaptureV2,
    ) -> Result<Vec<EpochTraceChunkWorkV2>, CheckpointError> {
        if capture.authority_digest != self.authority.digest()
            || usize::try_from(capture.first_ordinal).map_err(|_| CheckpointError::Limit)?
                != self.transitions.len()
            || capture.prepared.is_empty()
        {
            return Err(CheckpointError::Authority);
        }
        let mut completed = Vec::new();
        for prepared in capture.prepared {
            if let Some(work) = self.append_prepared(prepared)? {
                completed.push(work);
            }
        }
        Ok(completed)
    }

    fn append_prepared(
        &mut self,
        prepared: EpochPreparedTransitionV2,
    ) -> Result<Option<EpochTraceChunkWorkV2>, CheckpointError> {
        let ordinal = u32::try_from(self.transitions.len()).map_err(|_| CheckpointError::Limit)?;
        self.validate_prepared_link(ordinal, &prepared, self.transitions.last().copied())?;
        let inputs = prepared.binding.inputs();
        if self.pending.is_empty() {
            self.pending_input_accumulator = self.running_accumulator;
        }
        self.running_accumulator = epoch_stream_step_accumulator(
            self.authority,
            self.running_accumulator,
            prepared.binding,
        );
        self.transitions.push(prepared.binding);
        self.pending.push(prepared);

        let (_, expected_last) = epoch_trace_chunk_transition_range(
            self.authority.transition_count(),
            self.next_chunk_ordinal,
        )?;
        if inputs.ordinal == expected_last {
            self.take_completed_chunk().map(Some)
        } else {
            Ok(None)
        }
    }

    fn validate_prepared_link(
        &self,
        ordinal: u32,
        prepared: &EpochPreparedTransitionV2,
        previous: Option<EpochTransitionBindingV2>,
    ) -> Result<(), CheckpointError> {
        if ordinal >= self.authority.transition_count() || prepared.binding.ordinal() != ordinal {
            return Err(range_missing());
        }
        let inputs = prepared.binding.inputs();
        let expected_height = self
            .authority
            .start_height()
            .checked_add(u64::from(ordinal))
            .ok_or(CheckpointError::Overflow)?;
        if inputs.height != expected_height {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StepReordered,
            ));
        }
        match previous {
            None => {
                if inputs.pre_settlement_root != self.authority.start_root()
                    || (expected_height == 1 && inputs.predecessor.is_some())
                    || (expected_height != 1 && inputs.predecessor.is_none())
                {
                    return Err(CheckpointError::RecursiveRejected(
                        RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
                    ));
                }
            }
            Some(previous) => {
                let previous = previous.inputs();
                if inputs.predecessor != Some(previous.checkpoint_id)
                    || inputs.pre_settlement_root != previous.post_settlement_root
                {
                    return Err(CheckpointError::RecursiveRejected(
                        RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
                    ));
                }
            }
        }
        Ok(())
    }

    /// Close the immutable work manifest exactly once after the configured
    /// transition count. Tentative actual-verified chunks still require
    /// `validate_closed_chunk` before epoch seal.
    pub fn close(
        self,
        epoch_close_anchor_digest: [u8; 32],
        nova_chain_root: Option<[u8; 32]>,
    ) -> Result<EpochProofWorkManifestV2, CheckpointError> {
        if self.transitions.len()
            != usize::try_from(self.authority.transition_count())
                .map_err(|_| CheckpointError::Limit)?
            || !self.pending.is_empty()
            || self.next_chunk_ordinal != self.total_chunk_count
        {
            return Err(range_missing());
        }
        let end_root = self
            .transitions
            .last()
            .ok_or_else(range_missing)?
            .inputs()
            .post_settlement_root;
        EpochProofWorkManifestV2::new(EpochProofWorkManifestInputsV2 {
            cadence_class: self.authority.cadence_class(),
            epoch_index: self.authority.epoch_index(),
            start_height: self.authority.start_height(),
            end_height: self.authority.end_height(),
            transition_count: self.authority.transition_count(),
            parameter_generation: self.authority.parameter_generation(),
            runtime_profile_generation: self.authority.runtime_profile_generation(),
            config_generation: self.authority.config_generation(),
            authority_generation: self.authority.authority_generation(),
            chain_context_digest: self.authority.chain_context_digest(),
            predicate_digest: self.authority.predicate_digest(),
            parameter_digest: self.authority.parameter_digest(),
            verifier_bundle_digest: self.authority.verifier_bundle_digest(),
            security_budget_digest: self.authority.security_budget_digest(),
            config_digest: self.authority.config_digest(),
            registry_digest: self.authority.registry_digest(),
            runtime_profile_manifest_digest: self.authority.runtime_profile_manifest_digest(),
            frontier_authority_digest: self.authority.digest(),
            epoch_close_anchor_digest,
            nova_chain_root,
            start_root: self.authority.start_root(),
            end_root,
            transitions: self.transitions,
        })
    }

    fn take_completed_chunk(&mut self) -> Result<EpochTraceChunkWorkV2, CheckpointError> {
        let chunk_ordinal = self.next_chunk_ordinal;
        let (first_transition, last_transition) =
            epoch_trace_chunk_transition_range(self.authority.transition_count(), chunk_ordinal)?;
        let expected_len = last_transition
            .checked_sub(first_transition)
            .and_then(|span| span.checked_add(1))
            .ok_or(CheckpointError::Overflow)?;
        if self.pending.len()
            != usize::try_from(expected_len).map_err(|_| CheckpointError::Limit)?
            || self
                .pending
                .first()
                .map(EpochPreparedTransitionV2::binding)
                .map(|binding| binding.ordinal())
                != Some(first_transition)
            || self
                .pending
                .last()
                .map(EpochPreparedTransitionV2::binding)
                .map(|binding| binding.ordinal())
                != Some(last_transition)
        {
            return Err(CheckpointError::Invariant);
        }
        let event_count = self.pending.iter().try_fold(0_u64, |total, prepared| {
            total
                .checked_add(prepared.event_count())
                .ok_or(CheckpointError::Overflow)
        })?;
        let event_bytes = self.pending.iter().try_fold(0_u64, |total, prepared| {
            total
                .checked_add(prepared.event_bytes())
                .ok_or(CheckpointError::Overflow)
        })?;
        if event_count == 0 || event_bytes == 0 {
            return Err(CheckpointError::Invariant);
        }
        let prepared = core::mem::replace(
            &mut self.pending,
            Vec::with_capacity(
                usize::try_from(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2)
                    .map_err(|_| CheckpointError::Limit)?,
            ),
        );
        let work = EpochTraceChunkWorkV2 {
            authority: self.authority,
            chunk_ordinal,
            chunk_count: self.total_chunk_count,
            first_transition,
            last_transition,
            event_start: self.pending_event_start,
            event_count,
            event_bytes,
            input_accumulator: self.pending_input_accumulator,
            output_accumulator: self.running_accumulator,
            prepared,
        };
        self.pending_event_start = self
            .pending_event_start
            .checked_add(event_count)
            .ok_or(CheckpointError::Overflow)?;
        self.pending_input_accumulator = self.running_accumulator;
        self.next_chunk_ordinal = self
            .next_chunk_ordinal
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        Ok(work)
    }
}

/// One bounded, move-only direct-AIR worker item emitted while an epoch is
/// still open. It contains private event material for only its fixed transition
/// range and has no serialization path. Dropping it transitively zeroizes every
/// retained event vector.
pub struct EpochTraceChunkWorkV2 {
    authority: EpochFrontierAuthorityV2,
    chunk_ordinal: u32,
    chunk_count: u32,
    first_transition: u32,
    last_transition: u32,
    event_start: u64,
    event_count: u64,
    event_bytes: u64,
    input_accumulator: [u8; 32],
    output_accumulator: [u8; 32],
    prepared: Vec<EpochPreparedTransitionV2>,
}

impl EpochTraceChunkWorkV2 {
    #[must_use]
    pub const fn authority(&self) -> EpochFrontierAuthorityV2 {
        self.authority
    }

    #[must_use]
    pub const fn chunk_ordinal(&self) -> u32 {
        self.chunk_ordinal
    }

    #[must_use]
    pub const fn chunk_count(&self) -> u32 {
        self.chunk_count
    }

    #[must_use]
    pub const fn transition_range(&self) -> (u32, u32) {
        (self.first_transition, self.last_transition)
    }

    #[must_use]
    pub const fn event_range(&self) -> (u64, u64) {
        (self.event_start, self.event_count)
    }

    #[must_use]
    pub const fn event_bytes(&self) -> u64 {
        self.event_bytes
    }

    #[must_use]
    pub const fn accumulator_boundary(&self) -> ([u8; 32], [u8; 32]) {
        (self.input_accumulator, self.output_accumulator)
    }

    #[must_use]
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = EpochTransitionBindingV2> + '_ {
        self.prepared.iter().map(EpochPreparedTransitionV2::binding)
    }

    #[must_use]
    pub(super) fn prepared(&self) -> &[EpochPreparedTransitionV2] {
        &self.prepared
    }

    pub(super) fn statement(
        &self,
        table: EpochAirTableV2,
        replica: u8,
        row_start: u64,
        row_count: u64,
    ) -> Result<EpochTraceChunkV2, CheckpointError> {
        let transitions = self.bindings().collect::<Vec<_>>();
        let first = transitions.first().ok_or_else(range_missing)?.inputs();
        let last = transitions.last().ok_or_else(range_missing)?.inputs();
        let transition_digests = transitions
            .iter()
            .map(EpochTransitionBindingV2::digest)
            .collect::<Vec<_>>();
        let input_slice_commitment =
            epoch_ordered_digest_root_v2(EPOCH_TRANSITION_SLICE_DOMAIN_V2, &transition_digests)?;
        EpochTraceChunkV2::new(
            &self.authority,
            &transitions,
            EpochTraceChunkInputsV2 {
                table,
                replica,
                chunk_ordinal: self.chunk_ordinal,
                chunk_count: self.chunk_count,
                first_transition: self.first_transition,
                last_transition: self.last_transition,
                transition_count: self.authority.transition_count(),
                row_start,
                row_count,
                event_start: self.event_start,
                event_count: self.event_count,
                frontier_authority_digest: self.authority.digest(),
                chain_context_digest: self.authority.chain_context_digest(),
                predicate_digest: self.authority.predicate_digest(),
                input_state_root: first.pre_settlement_root,
                output_state_root: last.post_settlement_root,
                input_accumulator: self.input_accumulator,
                output_accumulator: self.output_accumulator,
                input_slice_commitment,
                parameter_digest: self.authority.parameter_digest(),
                verifier_bundle_digest: self.authority.verifier_bundle_digest(),
                security_budget_digest: self.authority.security_budget_digest(),
            },
        )
    }

    pub(super) fn into_prepared(self) -> Vec<EpochPreparedTransitionV2> {
        self.prepared
    }
}

fn epoch_trace_chunk_count(transition_count: u32) -> Result<u32, CheckpointError> {
    if transition_count == 0 {
        return Err(range_missing());
    }
    transition_count
        .checked_add(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 - 1)
        .and_then(|count| count.checked_div(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2))
        .filter(|count| *count != 0)
        .ok_or(CheckpointError::Overflow)
}

fn epoch_trace_chunk_transition_range(
    transition_count: u32,
    chunk_ordinal: u32,
) -> Result<(u32, u32), CheckpointError> {
    let chunk_count = epoch_trace_chunk_count(transition_count)?;
    if chunk_ordinal >= chunk_count {
        return Err(range_missing());
    }
    let first = chunk_ordinal
        .checked_mul(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2)
        .ok_or(CheckpointError::Overflow)?;
    let last = first
        .checked_add(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 - 1)
        .map(|last| last.min(transition_count - 1))
        .ok_or(CheckpointError::Overflow)?;
    Ok((first, last))
}

pub(super) fn epoch_stream_initial_accumulator(authority: EpochFrontierAuthorityV2) -> [u8; 32] {
    sha256_256(
        EPOCH_STREAM_ACCUMULATOR_DOMAIN_V2,
        EPOCH_STREAM_INITIAL_LABEL_V2,
        &[&authority.digest(), &authority.start_root()],
    )
}

pub(super) fn epoch_stream_step_accumulator(
    authority: EpochFrontierAuthorityV2,
    previous: [u8; 32],
    transition: EpochTransitionBindingV2,
) -> [u8; 32] {
    sha256_256(
        EPOCH_STREAM_ACCUMULATOR_DOMAIN_V2,
        EPOCH_STREAM_STEP_LABEL_V2,
        &[
            &authority.digest(),
            &previous,
            &transition.ordinal().to_le_bytes(),
            &transition.digest(),
        ],
    )
}

fn prepare_epoch_transition(
    authority: EpochFrontierAuthorityV2,
    ordinal: u32,
    material: TransitionMaterialV2,
) -> Result<EpochPreparedTransitionV2, CheckpointError> {
    let statement = material.transition_statement;
    let declared_counts = statement.declared_event_counts();
    let uniqueness_row_count = declared_counts
        .count(super::recursive_trace::RecursiveTraceOpcodeV2::ReplayInput)
        .checked_add(
            declared_counts.count(super::recursive_trace::RecursiveTraceOpcodeV2::ReplayOutput),
        )
        .ok_or(CheckpointError::Overflow)?;
    let jmt_record_count =
        declared_counts.count(super::recursive_trace::RecursiveTraceOpcodeV2::JmtMicroOp);
    let jmt_envelope_count =
        declared_counts.count(super::recursive_trace::RecursiveTraceOpcodeV2::JmtUpdate);
    let jmt_update_count = u64::from(super::plonky3::epoch_jmt_update_count(&material)?);
    let expected_event_bytes = 16_u64
        .checked_add(
            statement
                .declared_event_count()
                .checked_mul(4)
                .ok_or(CheckpointError::Overflow)?,
        )
        .and_then(|bytes| bytes.checked_add(statement.declared_byte_count()))
        .ok_or(CheckpointError::Overflow)?;
    let event_bytes =
        u64::try_from(material.event_vector.len()).map_err(|_| CheckpointError::Limit)?;
    let mismatch = [
        ("height", material.statement.height() != statement.height()),
        (
            "chain_context",
            material.statement.chain_context_digest() != authority.chain_context_digest(),
        ),
        (
            "predicate",
            material.statement.predicate_digest() != authority.predicate_digest(),
        ),
        (
            "parameter",
            material.parameter_digest != authority.parameter_digest(),
        ),
        (
            "security",
            material.security_budget_digest != authority.security_budget_digest(),
        ),
        (
            "verifier",
            material.verifier_bundle_digest != authority.verifier_bundle_digest(),
        ),
        ("event_bytes", event_bytes != expected_event_bytes),
        (
            "checkpoint_artifact",
            material.checkpoint_artifact_digest != statement.checkpoint_id().into_bytes(),
        ),
    ];
    if mismatch.iter().any(|(_, mismatched)| *mismatched) {
        if std::env::var_os("Z00Z_PLONKY3_RESOURCE_TELEMETRY").is_some() {
            let labels = mismatch
                .iter()
                .filter_map(|(label, mismatched)| mismatched.then_some(*label))
                .collect::<Vec<_>>()
                .join(",");
            eprintln!("Z00Z_PLONKY3_BINDING_MISMATCH_V1 fields={labels}");
        }
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let binding = EpochTransitionBindingV2::new(EpochTransitionInputsV2 {
        ordinal,
        height: statement.height(),
        checkpoint_id: statement.checkpoint_id().into_bytes(),
        predecessor: statement.predecessor().map(|id| id.into_bytes()),
        recursive_transition_statement_digest: statement.digest(),
        checkpoint_exec_tx_root: statement.checkpoint_exec_tx_root(),
        checkpoint_exec_tx_count: statement.checkpoint_exec_tx_count(),
        checkpoint_statement_digest: statement.checkpoint_statement_digest(),
        checkpoint_statement_core_digest: statement.checkpoint_statement_core_digest(),
        checkpoint_link_digest: statement.checkpoint_link_digest(),
        checkpoint_artifact_digest: material.checkpoint_artifact_digest,
        delta_root: statement.delta_root(),
        witness_root: statement.witness_root(),
        journal_digest: statement.journal_digest(),
        challenge_content_digest: material.challenge_content_digest,
        da_payload_commitment: material.da_payload_commitment,
        prior_recursive_output_root: statement.prior_recursive_output_root(),
        pre_settlement_root: *statement.pre_settlement_root().as_bytes(),
        post_settlement_root: *statement.post_settlement_root().as_bytes(),
        pre_definition_root: statement.pre_definition_root(),
        post_definition_root: statement.post_definition_root(),
        trace_digest: statement.trace_digest(),
        update_trace_digest: statement.update_trace_digest(),
        declared_work_digest: statement.declared_work_digest(),
        pre_uniqueness_context_digest: statement.pre_uniqueness_context_digest(),
        spent_uniqueness_precommit: statement.spent_uniqueness_precommit(),
        output_uniqueness_precommit: statement.output_uniqueness_precommit(),
        event_vector_digest: material.statement.event_vector_digest(),
        event_count: statement.declared_event_count(),
        event_bytes,
        uniqueness_row_count,
        jmt_record_count,
        jmt_envelope_count,
        jmt_update_count,
    })?;
    Ok(EpochPreparedTransitionV2 { binding, material })
}

/// Complete constructor input for one immutable epoch work manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpochProofWorkManifestInputsV2 {
    pub cadence_class: EpochCadenceClassV2,
    pub epoch_index: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub transition_count: u32,
    pub parameter_generation: u32,
    pub runtime_profile_generation: u16,
    pub config_generation: u64,
    pub authority_generation: u64,
    pub chain_context_digest: [u8; 32],
    pub predicate_digest: [u8; 32],
    pub parameter_digest: [u8; 32],
    pub verifier_bundle_digest: [u8; 32],
    pub security_budget_digest: [u8; 32],
    pub config_digest: [u8; 32],
    pub registry_digest: [u8; 32],
    pub runtime_profile_manifest_digest: [u8; 32],
    pub frontier_authority_digest: [u8; 32],
    pub epoch_close_anchor_digest: [u8; 32],
    pub nova_chain_root: Option<[u8; 32]>,
    pub start_root: [u8; 32],
    pub end_root: [u8; 32],
    pub transitions: Vec<EpochTransitionBindingV2>,
}

/// Local, strict, content-addressed authority for all epoch proof work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpochProofWorkManifestV2 {
    inputs: EpochProofWorkManifestInputsV2,
    transition_root: [u8; 32],
    digest: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl EpochProofWorkManifestV2 {
    pub fn new(inputs: EpochProofWorkManifestInputsV2) -> Result<Self, CheckpointError> {
        let authority = Plonky3HistoryAuthorityResolverV2::resolve_active()?;
        Self::new_with_authority(inputs, &authority)
    }

    fn new_with_authority(
        inputs: EpochProofWorkManifestInputsV2,
        authority: &ResolvedPlonky3HistoryAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        validate_manifest(&inputs, authority)?;
        let transition_digests = inputs
            .transitions
            .iter()
            .map(EpochTransitionBindingV2::digest)
            .collect::<Vec<_>>();
        let transition_root =
            epoch_ordered_digest_root_v2(EPOCH_TRANSITION_DOMAIN_V2, &transition_digests)?;
        let mut canonical_bytes = encode_manifest_prefix(&inputs, transition_root)?;
        let digest = sha256_256(
            EPOCH_WORK_DOMAIN_V2,
            EPOCH_WORK_LABEL_V2,
            &[&canonical_bytes],
        );
        canonical_bytes.extend_from_slice(&digest);
        Ok(Self {
            inputs,
            transition_root,
            digest,
            canonical_bytes,
        })
    }

    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, CheckpointError> {
        let authority = Plonky3HistoryAuthorityResolverV2::resolve_active()?;
        Self::decode_canonical_with_authority(bytes, &authority)
    }

    pub fn decode_canonical_with_authority(
        bytes: &[u8],
        authority: &ResolvedPlonky3HistoryAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        if bytes.len() < EPOCH_WORK_HEADER_BYTES_V2 + EPOCH_WORK_TRAILER_BYTES_V2 {
            return Err(CheckpointError::Canonical);
        }
        let mut reader = EpochCodecReaderV2::new(bytes);
        if reader.array::<8>()? != EPOCH_WORK_MAGIC_V2 || reader.u16()? != EPOCH_WORK_WIRE_V2 {
            return Err(CheckpointError::Canonical);
        }
        let cadence_class = decode_cadence(reader.u8()?)?;
        if reader.u16()? != EPOCH_DIRECT_AIR_GENERATION_V2
            || reader.u16()? != EPOCH_CHUNK_GRAMMAR_GENERATION_V2
        {
            return Err(CheckpointError::Canonical);
        }
        let epoch_index = reader.u64()?;
        let start_height = reader.u64()?;
        let end_height = reader.u64()?;
        let transition_count = reader.u32()?;
        if transition_count == 0 || transition_count > EPOCH_WORK_MAX_TRANSITIONS_V2 {
            return Err(CheckpointError::Limit);
        }
        let expected_len = manifest_encoded_len(transition_count)?;
        if bytes.len() != expected_len {
            return Err(CheckpointError::Canonical);
        }
        let parameter_generation = reader.u32()?;
        let runtime_profile_generation = reader.u16()?;
        let config_generation = reader.u64()?;
        let authority_generation = reader.u64()?;
        let chain_context_digest = reader.array()?;
        let predicate_digest = reader.array()?;
        let parameter_digest = reader.array()?;
        let verifier_bundle_digest = reader.array()?;
        let security_budget_digest = reader.array()?;
        let config_digest = reader.array()?;
        let registry_digest = reader.array()?;
        let runtime_profile_manifest_digest = reader.array()?;
        let frontier_authority_digest = reader.array()?;
        let epoch_close_anchor_digest = reader.array()?;
        let nova_chain_root = decode_optional_digest(&mut reader)?;
        let start_root = reader.array()?;
        let end_root = reader.array()?;
        let encoded_transition_root: [u8; 32] = reader.array()?;
        let mut transitions = Vec::with_capacity(transition_count as usize);
        for _ in 0..transition_count {
            transitions.push(decode_transition(&mut reader)?);
        }
        let encoded_digest: [u8; 32] = reader.array()?;
        if !reader.is_done() {
            return Err(CheckpointError::Canonical);
        }
        let manifest = Self::new_with_authority(
            EpochProofWorkManifestInputsV2 {
                cadence_class,
                epoch_index,
                start_height,
                end_height,
                transition_count,
                parameter_generation,
                runtime_profile_generation,
                config_generation,
                authority_generation,
                chain_context_digest,
                predicate_digest,
                parameter_digest,
                verifier_bundle_digest,
                security_budget_digest,
                config_digest,
                registry_digest,
                runtime_profile_manifest_digest,
                frontier_authority_digest,
                epoch_close_anchor_digest,
                nova_chain_root,
                start_root,
                end_root,
                transitions,
            },
            authority,
        )?;
        if manifest.transition_root != encoded_transition_root
            || manifest.digest != encoded_digest
            || manifest.canonical_bytes != bytes
        {
            return Err(CheckpointError::Canonical);
        }
        Ok(manifest)
    }

    #[must_use]
    pub const fn cadence_class(&self) -> EpochCadenceClassV2 {
        self.inputs.cadence_class
    }

    #[must_use]
    pub const fn epoch_index(&self) -> u64 {
        self.inputs.epoch_index
    }

    #[must_use]
    pub const fn start_height(&self) -> u64 {
        self.inputs.start_height
    }

    #[must_use]
    pub const fn end_height(&self) -> u64 {
        self.inputs.end_height
    }

    #[must_use]
    pub const fn transition_count(&self) -> u32 {
        self.inputs.transition_count
    }

    #[must_use]
    pub const fn frontier_authority_digest(&self) -> [u8; 32] {
        self.inputs.frontier_authority_digest
    }

    #[must_use]
    pub const fn epoch_close_anchor_digest(&self) -> [u8; 32] {
        self.inputs.epoch_close_anchor_digest
    }

    #[must_use]
    pub const fn nova_chain_root(&self) -> Option<[u8; 32]> {
        self.inputs.nova_chain_root
    }

    #[must_use]
    pub const fn start_root(&self) -> [u8; 32] {
        self.inputs.start_root
    }

    #[must_use]
    pub const fn end_root(&self) -> [u8; 32] {
        self.inputs.end_root
    }

    pub fn frontier_authority(&self) -> Result<EpochFrontierAuthorityV2, CheckpointError> {
        manifest_frontier_authority(&self.inputs)
    }

    #[must_use]
    pub const fn transition_root(&self) -> [u8; 32] {
        self.transition_root
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    #[must_use]
    pub fn transition(&self, ordinal: u32) -> Option<EpochTransitionBindingV2> {
        self.inputs.transitions.get(ordinal as usize).copied()
    }

    /// Promote one previously actual-verified tentative chunk into the closed
    /// manifest only after its exact ordered transition slice is known.
    pub fn validate_closed_chunk(&self, chunk: &EpochTraceChunkV2) -> Result<(), CheckpointError> {
        let authority = manifest_frontier_authority(&self.inputs)?;
        let inputs = chunk.inputs();
        let start = usize::try_from(inputs.first_transition).map_err(|_| CheckpointError::Limit)?;
        let end = usize::try_from(
            inputs
                .last_transition
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?,
        )
        .map_err(|_| CheckpointError::Limit)?;
        let transitions = self
            .inputs
            .transitions
            .get(start..end)
            .ok_or_else(range_missing)?;
        let canonical = EpochTraceChunkV2::new(&authority, transitions, inputs)?;
        if canonical != *chunk {
            return Err(CheckpointError::Canonical);
        }
        Ok(())
    }
}

/// Direct AIR table selected by one trace-chunk statement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EpochAirTableV2 {
    TraceFraming = 1,
    PackedRange = 2,
    Sha256 = 3,
    JmtUpdate = 4,
    Uniqueness = 5,
    TypedCommitment = 6,
    Transition = 7,
}

impl EpochAirTableV2 {
    fn decode(value: u8) -> Result<Self, CheckpointError> {
        match value {
            1 => Ok(Self::TraceFraming),
            2 => Ok(Self::PackedRange),
            3 => Ok(Self::Sha256),
            4 => Ok(Self::JmtUpdate),
            5 => Ok(Self::Uniqueness),
            6 => Ok(Self::TypedCommitment),
            7 => Ok(Self::Transition),
            _ => Err(CheckpointError::Canonical),
        }
    }
}

/// Complete public inputs for one bounded, direct-AIR trace chunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochTraceChunkInputsV2 {
    pub table: EpochAirTableV2,
    pub replica: u8,
    pub chunk_ordinal: u32,
    pub chunk_count: u32,
    pub first_transition: u32,
    pub last_transition: u32,
    pub transition_count: u32,
    pub row_start: u64,
    pub row_count: u64,
    pub event_start: u64,
    pub event_count: u64,
    pub frontier_authority_digest: [u8; 32],
    pub chain_context_digest: [u8; 32],
    pub predicate_digest: [u8; 32],
    pub input_state_root: [u8; 32],
    pub output_state_root: [u8; 32],
    pub input_accumulator: [u8; 32],
    pub output_accumulator: [u8; 32],
    pub input_slice_commitment: [u8; 32],
    pub parameter_digest: [u8; 32],
    pub verifier_bundle_digest: [u8; 32],
    pub security_budget_digest: [u8; 32],
}

/// Strict fixed-width public statement for one direct-AIR proof unit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpochTraceChunkV2 {
    inputs: EpochTraceChunkInputsV2,
    digest: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl EpochTraceChunkV2 {
    pub fn new(
        authority: &EpochFrontierAuthorityV2,
        transitions: &[EpochTransitionBindingV2],
        inputs: EpochTraceChunkInputsV2,
    ) -> Result<Self, CheckpointError> {
        validate_chunk(authority, transitions, &inputs)?;
        let mut canonical_bytes = encode_chunk_prefix(&inputs);
        if canonical_bytes.len() != EPOCH_CHUNK_BYTES_V2 - 32 {
            return Err(CheckpointError::Invariant);
        }
        let digest = sha256_256(
            EPOCH_CHUNK_DOMAIN_V2,
            EPOCH_CHUNK_LABEL_V2,
            &[&canonical_bytes],
        );
        canonical_bytes.extend_from_slice(&digest);
        Ok(Self {
            inputs,
            digest,
            canonical_bytes,
        })
    }

    pub fn decode_canonical(
        authority: &EpochFrontierAuthorityV2,
        transitions: &[EpochTransitionBindingV2],
        bytes: &[u8],
    ) -> Result<Self, CheckpointError> {
        if bytes.len() != EPOCH_CHUNK_BYTES_V2 {
            return Err(CheckpointError::Canonical);
        }
        let mut reader = EpochCodecReaderV2::new(bytes);
        if reader.array::<8>()? != EPOCH_CHUNK_MAGIC_V2
            || reader.u16()? != EPOCH_CHUNK_WIRE_V2
            || reader.u16()? != EPOCH_DIRECT_AIR_GENERATION_V2
        {
            return Err(CheckpointError::Canonical);
        }
        let table = EpochAirTableV2::decode(reader.u8()?)?;
        let replica = reader.u8()?;
        let chunk_ordinal = reader.u32()?;
        let chunk_count = reader.u32()?;
        let first_transition = reader.u32()?;
        let last_transition = reader.u32()?;
        let transition_count = reader.u32()?;
        let row_start = reader.u64()?;
        let row_count = reader.u64()?;
        let event_start = reader.u64()?;
        let event_count = reader.u64()?;
        let frontier_authority_digest = reader.array()?;
        let chain_context_digest = reader.array()?;
        let predicate_digest = reader.array()?;
        let input_state_root = reader.array()?;
        let output_state_root = reader.array()?;
        let input_accumulator = reader.array()?;
        let output_accumulator = reader.array()?;
        let input_slice_commitment = reader.array()?;
        let parameter_digest = reader.array()?;
        let verifier_bundle_digest = reader.array()?;
        let security_budget_digest = reader.array()?;
        let encoded_digest: [u8; 32] = reader.array()?;
        if !reader.is_done() {
            return Err(CheckpointError::Canonical);
        }
        let statement = Self::new(
            authority,
            transitions,
            EpochTraceChunkInputsV2 {
                table,
                replica,
                chunk_ordinal,
                chunk_count,
                first_transition,
                last_transition,
                transition_count,
                row_start,
                row_count,
                event_start,
                event_count,
                frontier_authority_digest,
                chain_context_digest,
                predicate_digest,
                input_state_root,
                output_state_root,
                input_accumulator,
                output_accumulator,
                input_slice_commitment,
                parameter_digest,
                verifier_bundle_digest,
                security_budget_digest,
            },
        )?;
        if statement.digest != encoded_digest || statement.canonical_bytes != bytes {
            return Err(CheckpointError::Canonical);
        }
        Ok(statement)
    }

    #[must_use]
    pub const fn inputs(&self) -> EpochTraceChunkInputsV2 {
        self.inputs
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

fn validate_transition(inputs: &EpochTransitionInputsV2) -> Result<(), CheckpointError> {
    let counted_semantic_events = inputs
        .uniqueness_row_count
        .checked_add(inputs.jmt_record_count)
        .and_then(|count| count.checked_add(inputs.jmt_envelope_count))
        .ok_or(CheckpointError::Overflow)?;
    if inputs.height == 0
        || inputs.event_count == 0
        || inputs.event_bytes == 0
        || inputs.jmt_envelope_count != 1
        || (inputs.jmt_record_count == 0) != (inputs.jmt_update_count == 0)
        || inputs.jmt_update_count > inputs.jmt_record_count
        || counted_semantic_events > inputs.event_count
        || [
            inputs.checkpoint_id,
            inputs.recursive_transition_statement_digest,
            inputs.checkpoint_exec_tx_root,
            inputs.checkpoint_statement_digest,
            inputs.checkpoint_statement_core_digest,
            inputs.checkpoint_link_digest,
            inputs.checkpoint_artifact_digest,
            inputs.delta_root,
            inputs.witness_root,
            inputs.journal_digest,
            inputs.challenge_content_digest,
            inputs.da_payload_commitment,
            inputs.pre_settlement_root,
            inputs.post_settlement_root,
            inputs.pre_definition_root,
            inputs.post_definition_root,
            inputs.trace_digest,
            inputs.update_trace_digest,
            inputs.declared_work_digest,
            inputs.pre_uniqueness_context_digest,
            inputs.spent_uniqueness_precommit,
            inputs.output_uniqueness_precommit,
            inputs.event_vector_digest,
        ]
        .contains(&[0; 32])
        || inputs.predecessor == Some([0; 32])
        || inputs.prior_recursive_output_root == Some([0; 32])
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::EpochManifestIncomplete,
        ));
    }
    Ok(())
}

fn validate_manifest(
    inputs: &EpochProofWorkManifestInputsV2,
    authority: &ResolvedPlonky3HistoryAuthorityV2,
) -> Result<(), CheckpointError> {
    let count = inputs.transition_count;
    if count == 0
        || count > EPOCH_WORK_MAX_TRANSITIONS_V2
        || inputs.transitions.len() != count as usize
        || inputs.start_height == 0
        || inputs.end_height
            != inputs
                .start_height
                .checked_add(u64::from(count))
                .and_then(|height| height.checked_sub(1))
                .ok_or(CheckpointError::Overflow)?
        || inputs.start_height
            != inputs
                .epoch_index
                .checked_mul(u64::from(count))
                .and_then(|height| height.checked_add(1))
                .ok_or(CheckpointError::Overflow)?
    {
        return Err(range_missing());
    }
    let resolved = CheckpointConfigResolverV3::resolve_active()?;
    let identity = resolved.identity();
    let security = RecursiveSecurityBudgetManifestV2::authority_pinned()?;
    let history_identity = authority.identity();
    let production_cadence = authority.cadence_blocks();
    match inputs.cadence_class {
        EpochCadenceClassV2::Production if u64::from(count) == production_cadence => {}
        EpochCadenceClassV2::BoundedSimulation
            if count > 0 && u64::from(count) < production_cadence => {}
        EpochCadenceClassV2::Production | EpochCadenceClassV2::BoundedSimulation => {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::HybridCadenceMismatch,
            ));
        }
    }
    if inputs.parameter_generation != history_identity.parameter_generation
        || inputs.runtime_profile_generation != identity.runtime_profile_generation
        || inputs.config_generation != identity.config_generation
        || inputs.authority_generation != identity.authority_generation
        || inputs.parameter_digest != history_identity.verifier_parameter_digest
        || inputs.verifier_bundle_digest != history_identity.verifier_bundle_digest
        || inputs.security_budget_digest != security.digest()
        || inputs.config_digest != identity.config_digest
        || inputs.registry_digest != identity.registry_digest
        || inputs.runtime_profile_manifest_digest != identity.runtime_profile_manifest_digest
        || [
            inputs.chain_context_digest,
            inputs.predicate_digest,
            inputs.frontier_authority_digest,
            inputs.epoch_close_anchor_digest,
            inputs.start_root,
            inputs.end_root,
        ]
        .contains(&[0; 32])
        || inputs.nova_chain_root == Some([0; 32])
    {
        return Err(range_missing());
    }
    let frontier = manifest_frontier_authority(inputs)?;
    if frontier.digest() != inputs.frontier_authority_digest {
        return Err(range_missing());
    }
    for (index, transition) in inputs.transitions.iter().enumerate() {
        let expected_ordinal = u32::try_from(index).map_err(|_| CheckpointError::Limit)?;
        let expected_height = inputs
            .start_height
            .checked_add(u64::from(expected_ordinal))
            .ok_or(CheckpointError::Overflow)?;
        let transition_inputs = transition.inputs();
        if transition.ordinal() != expected_ordinal || transition.height() != expected_height {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StepReordered,
            ));
        }
        if index == 0 {
            if transition_inputs.pre_settlement_root != inputs.start_root
                || (inputs.start_height == 1 && transition_inputs.predecessor.is_some())
                || (inputs.start_height != 1 && transition_inputs.predecessor.is_none())
            {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
                ));
            }
        } else {
            let previous = inputs.transitions[index - 1].inputs();
            if transition_inputs.predecessor != Some(previous.checkpoint_id)
                || transition_inputs.pre_settlement_root != previous.post_settlement_root
            {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
                ));
            }
        }
    }
    if inputs
        .transitions
        .last()
        .ok_or_else(range_missing)?
        .inputs()
        .post_settlement_root
        != inputs.end_root
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::OutputRootMismatch,
        ));
    }
    Ok(())
}

fn manifest_frontier_authority(
    inputs: &EpochProofWorkManifestInputsV2,
) -> Result<EpochFrontierAuthorityV2, CheckpointError> {
    EpochFrontierAuthorityV2::new(EpochFrontierAuthorityInputsV2 {
        cadence_class: inputs.cadence_class,
        epoch_index: inputs.epoch_index,
        cadence_blocks: u64::from(inputs.transition_count),
        start_root: inputs.start_root,
        chain_context_digest: inputs.chain_context_digest,
        predicate_digest: inputs.predicate_digest,
        parameter_digest: inputs.parameter_digest,
        verifier_bundle_digest: inputs.verifier_bundle_digest,
    })
}

fn validate_chunk(
    authority: &EpochFrontierAuthorityV2,
    transitions: &[EpochTransitionBindingV2],
    inputs: &EpochTraceChunkInputsV2,
) -> Result<(), CheckpointError> {
    let slice_len = inputs
        .last_transition
        .checked_sub(inputs.first_transition)
        .and_then(|count| count.checked_add(1))
        .ok_or(CheckpointError::Overflow)?;
    if inputs.replica >= 3
        || inputs.chunk_count == 0
        || inputs.chunk_ordinal >= inputs.chunk_count
        || inputs.transition_count != authority.transition_count()
        || inputs.first_transition > inputs.last_transition
        || inputs.last_transition >= inputs.transition_count
        || usize::try_from(slice_len).map_err(|_| CheckpointError::Limit)? != transitions.len()
        || (inputs.row_count == 0
            && !matches!(
                inputs.table,
                EpochAirTableV2::JmtUpdate | EpochAirTableV2::Uniqueness
            ))
        || inputs.event_count == 0
        || inputs.frontier_authority_digest != authority.digest()
        || inputs.chain_context_digest != authority.chain_context_digest()
        || inputs.predicate_digest != authority.predicate_digest()
        || [
            inputs.chain_context_digest,
            inputs.predicate_digest,
            inputs.input_state_root,
            inputs.output_state_root,
            inputs.input_accumulator,
            inputs.output_accumulator,
            inputs.input_slice_commitment,
            inputs.parameter_digest,
            inputs.verifier_bundle_digest,
            inputs.security_budget_digest,
        ]
        .contains(&[0; 32])
        || inputs.parameter_digest != authority.parameter_digest()
        || inputs.verifier_bundle_digest != authority.verifier_bundle_digest()
        || inputs.security_budget_digest != authority.security_budget_digest()
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let first = transitions.first().ok_or_else(range_missing)?.inputs();
    let last = transitions.last().ok_or_else(range_missing)?.inputs();
    if inputs.input_state_root != first.pre_settlement_root
        || inputs.output_state_root != last.post_settlement_root
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
        ));
    }
    let mut event_count = 0_u64;
    let mut transition_digests = Vec::with_capacity(transitions.len());
    for (offset, transition) in transitions.iter().enumerate() {
        let offset = u32::try_from(offset).map_err(|_| CheckpointError::Limit)?;
        let expected_ordinal = inputs
            .first_transition
            .checked_add(offset)
            .ok_or(CheckpointError::Overflow)?;
        let expected_height = authority
            .start_height()
            .checked_add(u64::from(expected_ordinal))
            .ok_or(CheckpointError::Overflow)?;
        if transition.ordinal() != expected_ordinal || transition.height() != expected_height {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StepReordered,
            ));
        }
        if let Some(previous) = offset
            .checked_sub(1)
            .and_then(|index| transitions.get(index as usize))
        {
            let previous = previous.inputs();
            let current = transition.inputs();
            if current.predecessor != Some(previous.checkpoint_id)
                || current.pre_settlement_root != previous.post_settlement_root
            {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
                ));
            }
        }
        event_count = event_count
            .checked_add(transition.inputs().event_count)
            .ok_or(CheckpointError::Overflow)?;
        transition_digests.push(transition.digest());
    }
    let slice_commitment =
        epoch_ordered_digest_root_v2(EPOCH_TRANSITION_SLICE_DOMAIN_V2, &transition_digests)?;
    let output_accumulator =
        transitions
            .iter()
            .copied()
            .fold(inputs.input_accumulator, |accumulator, transition| {
                epoch_stream_step_accumulator(*authority, accumulator, transition)
            });
    let invalid_initial_boundary = inputs.first_transition == 0
        && inputs.input_accumulator != epoch_stream_initial_accumulator(*authority);
    if inputs.event_count != event_count
        || inputs.input_slice_commitment != slice_commitment
        || inputs.output_accumulator != output_accumulator
        || invalid_initial_boundary
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

fn encode_transition(inputs: &EpochTransitionInputsV2) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(EPOCH_TRANSITION_BYTES_V2);
    bytes.extend_from_slice(&inputs.ordinal.to_le_bytes());
    bytes.extend_from_slice(&inputs.height.to_le_bytes());
    bytes.extend_from_slice(&inputs.checkpoint_id);
    bytes.push(u8::from(inputs.predecessor.is_some()));
    bytes.extend_from_slice(&inputs.predecessor.unwrap_or([0; 32]));
    bytes.extend_from_slice(&inputs.recursive_transition_statement_digest);
    bytes.extend_from_slice(&inputs.checkpoint_exec_tx_root);
    bytes.extend_from_slice(&inputs.checkpoint_exec_tx_count.to_le_bytes());
    for digest in [
        inputs.checkpoint_statement_digest,
        inputs.checkpoint_statement_core_digest,
        inputs.checkpoint_link_digest,
        inputs.checkpoint_artifact_digest,
        inputs.delta_root,
        inputs.witness_root,
        inputs.journal_digest,
        inputs.challenge_content_digest,
        inputs.da_payload_commitment,
    ] {
        bytes.extend_from_slice(&digest);
    }
    bytes.push(u8::from(inputs.prior_recursive_output_root.is_some()));
    bytes.extend_from_slice(&inputs.prior_recursive_output_root.unwrap_or([0; 32]));
    for digest in [
        inputs.pre_settlement_root,
        inputs.post_settlement_root,
        inputs.pre_definition_root,
        inputs.post_definition_root,
        inputs.trace_digest,
        inputs.update_trace_digest,
        inputs.declared_work_digest,
        inputs.pre_uniqueness_context_digest,
        inputs.spent_uniqueness_precommit,
        inputs.output_uniqueness_precommit,
        inputs.event_vector_digest,
    ] {
        bytes.extend_from_slice(&digest);
    }
    bytes.extend_from_slice(&inputs.event_count.to_le_bytes());
    bytes.extend_from_slice(&inputs.event_bytes.to_le_bytes());
    bytes.extend_from_slice(&inputs.uniqueness_row_count.to_le_bytes());
    bytes.extend_from_slice(&inputs.jmt_record_count.to_le_bytes());
    bytes.extend_from_slice(&inputs.jmt_envelope_count.to_le_bytes());
    bytes.extend_from_slice(&inputs.jmt_update_count.to_le_bytes());
    debug_assert_eq!(bytes.len(), EPOCH_TRANSITION_BYTES_V2);
    bytes
}

fn decode_transition(
    reader: &mut EpochCodecReaderV2<'_>,
) -> Result<EpochTransitionBindingV2, CheckpointError> {
    let ordinal = reader.u32()?;
    let height = reader.u64()?;
    let checkpoint_id = reader.array()?;
    let predecessor = match (reader.u8()?, reader.array::<32>()?) {
        (0, digest) if digest == [0; 32] => None,
        (1, digest) if digest != [0; 32] => Some(digest),
        _ => return Err(CheckpointError::Canonical),
    };
    EpochTransitionBindingV2::new(EpochTransitionInputsV2 {
        ordinal,
        height,
        checkpoint_id,
        predecessor,
        recursive_transition_statement_digest: reader.array()?,
        checkpoint_exec_tx_root: reader.array()?,
        checkpoint_exec_tx_count: reader.u32()?,
        checkpoint_statement_digest: reader.array()?,
        checkpoint_statement_core_digest: reader.array()?,
        checkpoint_link_digest: reader.array()?,
        checkpoint_artifact_digest: reader.array()?,
        delta_root: reader.array()?,
        witness_root: reader.array()?,
        journal_digest: reader.array()?,
        challenge_content_digest: reader.array()?,
        da_payload_commitment: reader.array()?,
        prior_recursive_output_root: match (reader.u8()?, reader.array::<32>()?) {
            (0, digest) if digest == [0; 32] => None,
            (1, digest) if digest != [0; 32] => Some(digest),
            _ => return Err(CheckpointError::Canonical),
        },
        pre_settlement_root: reader.array()?,
        post_settlement_root: reader.array()?,
        pre_definition_root: reader.array()?,
        post_definition_root: reader.array()?,
        trace_digest: reader.array()?,
        update_trace_digest: reader.array()?,
        declared_work_digest: reader.array()?,
        pre_uniqueness_context_digest: reader.array()?,
        spent_uniqueness_precommit: reader.array()?,
        output_uniqueness_precommit: reader.array()?,
        event_vector_digest: reader.array()?,
        event_count: reader.u64()?,
        event_bytes: reader.u64()?,
        uniqueness_row_count: reader.u64()?,
        jmt_record_count: reader.u64()?,
        jmt_envelope_count: reader.u64()?,
        jmt_update_count: reader.u64()?,
    })
}

fn encode_manifest_prefix(
    inputs: &EpochProofWorkManifestInputsV2,
    transition_root: [u8; 32],
) -> Result<Vec<u8>, CheckpointError> {
    let expected = manifest_encoded_len(inputs.transition_count)?
        .checked_sub(EPOCH_WORK_TRAILER_BYTES_V2)
        .ok_or(CheckpointError::Overflow)?;
    let mut bytes = Vec::with_capacity(expected);
    bytes.extend_from_slice(&EPOCH_WORK_MAGIC_V2);
    bytes.extend_from_slice(&EPOCH_WORK_WIRE_V2.to_le_bytes());
    bytes.push(inputs.cadence_class as u8);
    bytes.extend_from_slice(&EPOCH_DIRECT_AIR_GENERATION_V2.to_le_bytes());
    bytes.extend_from_slice(&EPOCH_CHUNK_GRAMMAR_GENERATION_V2.to_le_bytes());
    bytes.extend_from_slice(&inputs.epoch_index.to_le_bytes());
    bytes.extend_from_slice(&inputs.start_height.to_le_bytes());
    bytes.extend_from_slice(&inputs.end_height.to_le_bytes());
    bytes.extend_from_slice(&inputs.transition_count.to_le_bytes());
    bytes.extend_from_slice(&inputs.parameter_generation.to_le_bytes());
    bytes.extend_from_slice(&inputs.runtime_profile_generation.to_le_bytes());
    bytes.extend_from_slice(&inputs.config_generation.to_le_bytes());
    bytes.extend_from_slice(&inputs.authority_generation.to_le_bytes());
    for digest in [
        inputs.chain_context_digest,
        inputs.predicate_digest,
        inputs.parameter_digest,
        inputs.verifier_bundle_digest,
        inputs.security_budget_digest,
        inputs.config_digest,
        inputs.registry_digest,
        inputs.runtime_profile_manifest_digest,
        inputs.frontier_authority_digest,
        inputs.epoch_close_anchor_digest,
    ] {
        bytes.extend_from_slice(&digest);
    }
    bytes.push(u8::from(inputs.nova_chain_root.is_some()));
    bytes.extend_from_slice(&inputs.nova_chain_root.unwrap_or([0; 32]));
    for digest in [inputs.start_root, inputs.end_root, transition_root] {
        bytes.extend_from_slice(&digest);
    }
    for transition in &inputs.transitions {
        bytes.extend_from_slice(&encode_transition(&transition.inputs()));
    }
    if bytes.len() != expected {
        return Err(CheckpointError::Invariant);
    }
    Ok(bytes)
}

fn encode_chunk_prefix(inputs: &EpochTraceChunkInputsV2) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(EPOCH_CHUNK_BYTES_V2 - 32);
    bytes.extend_from_slice(&EPOCH_CHUNK_MAGIC_V2);
    bytes.extend_from_slice(&EPOCH_CHUNK_WIRE_V2.to_le_bytes());
    bytes.extend_from_slice(&EPOCH_DIRECT_AIR_GENERATION_V2.to_le_bytes());
    bytes.push(inputs.table as u8);
    bytes.push(inputs.replica);
    bytes.extend_from_slice(&inputs.chunk_ordinal.to_le_bytes());
    bytes.extend_from_slice(&inputs.chunk_count.to_le_bytes());
    bytes.extend_from_slice(&inputs.first_transition.to_le_bytes());
    bytes.extend_from_slice(&inputs.last_transition.to_le_bytes());
    bytes.extend_from_slice(&inputs.transition_count.to_le_bytes());
    bytes.extend_from_slice(&inputs.row_start.to_le_bytes());
    bytes.extend_from_slice(&inputs.row_count.to_le_bytes());
    bytes.extend_from_slice(&inputs.event_start.to_le_bytes());
    bytes.extend_from_slice(&inputs.event_count.to_le_bytes());
    for digest in [
        inputs.frontier_authority_digest,
        inputs.chain_context_digest,
        inputs.predicate_digest,
        inputs.input_state_root,
        inputs.output_state_root,
        inputs.input_accumulator,
        inputs.output_accumulator,
        inputs.input_slice_commitment,
        inputs.parameter_digest,
        inputs.verifier_bundle_digest,
        inputs.security_budget_digest,
    ] {
        bytes.extend_from_slice(&digest);
    }
    bytes
}

fn manifest_encoded_len(transition_count: u32) -> Result<usize, CheckpointError> {
    let transition_bytes = usize::try_from(transition_count)
        .map_err(|_| CheckpointError::Limit)?
        .checked_mul(EPOCH_TRANSITION_BYTES_V2)
        .ok_or(CheckpointError::Overflow)?;
    EPOCH_WORK_HEADER_BYTES_V2
        .checked_add(transition_bytes)
        .and_then(|bytes| bytes.checked_add(EPOCH_WORK_TRAILER_BYTES_V2))
        .ok_or(CheckpointError::Overflow)
}

fn decode_cadence(value: u8) -> Result<EpochCadenceClassV2, CheckpointError> {
    match value {
        1 => Ok(EpochCadenceClassV2::Production),
        2 => Ok(EpochCadenceClassV2::BoundedSimulation),
        _ => Err(CheckpointError::Canonical),
    }
}

fn decode_optional_digest(
    reader: &mut EpochCodecReaderV2<'_>,
) -> Result<Option<[u8; 32]>, CheckpointError> {
    match (reader.u8()?, reader.array::<32>()?) {
        (0, digest) if digest == [0; 32] => Ok(None),
        (1, digest) if digest != [0; 32] => Ok(Some(digest)),
        _ => Err(CheckpointError::Canonical),
    }
}

fn range_missing() -> CheckpointError {
    CheckpointError::RecursiveRejected(
        RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emit_smoke_metrics(table: &str, metrics: crate::checkpoint::plonky3::EpochSmokeMetricsV2) {
        println!(
            "Z00Z_PLONKY3_TELEMETRY_V1 {}",
            serde_json::json!({
                "parameter_digest":
                    crate::checkpoint::contract_config_v3::hex_digest(metrics.parameter_digest),
                "canonical_proof_bytes": metrics.proof_bytes,
                "size_status": "internal_only",
                "trace_dimensions": {
                    "table": table,
                    "trace_rows": metrics.trace_rows,
                    "input_items": metrics.input_items,
                    "table_count": metrics.table_count,
                    "actual_verifier": true,
                },
            })
        );
    }

    fn digest(label: &str, ordinal: u64) -> [u8; 32] {
        sha256_256(
            "z00z.storage.checkpoint.epoch-prover.tests.v2",
            label,
            &[&ordinal.to_le_bytes()],
        )
    }

    fn transition(
        ordinal: u32,
        start_height: u64,
        previous_checkpoint: Option<[u8; 32]>,
        pre_root: [u8; 32],
    ) -> EpochTransitionBindingV2 {
        let height = start_height + u64::from(ordinal);
        EpochTransitionBindingV2::new(EpochTransitionInputsV2 {
            ordinal,
            height,
            checkpoint_id: digest("checkpoint", height),
            predecessor: previous_checkpoint,
            recursive_transition_statement_digest: digest("recursive-statement", height),
            checkpoint_exec_tx_root: digest("exec-tx-root", height),
            checkpoint_exec_tx_count: 1,
            checkpoint_statement_digest: digest("statement", height),
            checkpoint_statement_core_digest: digest("statement-core", height),
            checkpoint_link_digest: digest("link", height),
            checkpoint_artifact_digest: digest("artifact", height),
            delta_root: digest("delta", height),
            witness_root: digest("witness", height),
            journal_digest: digest("journal", height),
            challenge_content_digest: digest("challenge", height),
            da_payload_commitment: digest("da", height),
            prior_recursive_output_root: previous_checkpoint.map(|_| digest("prior-ivc", height)),
            pre_settlement_root: pre_root,
            post_settlement_root: digest("state", height),
            pre_definition_root: digest("definition-pre", height),
            post_definition_root: digest("definition-post", height),
            trace_digest: digest("trace", height),
            update_trace_digest: digest("update-trace", height),
            declared_work_digest: digest("declared-work", height),
            pre_uniqueness_context_digest: digest("pre-uniqueness", height),
            spent_uniqueness_precommit: digest("spent-precommit", height),
            output_uniqueness_precommit: digest("output-precommit", height),
            event_vector_digest: digest("events", height),
            event_count: 17,
            event_bytes: 512,
            uniqueness_row_count: 1,
            jmt_record_count: 1,
            jmt_envelope_count: 1,
            jmt_update_count: 1,
        })
        .expect("transition")
    }

    fn manifest(count: u32) -> EpochProofWorkManifestV2 {
        let active = CheckpointConfigResolverV3::resolve_active().expect("config");
        let identity = active.identity();
        let history =
            Plonky3HistoryAuthorityResolverV2::resolve_active().expect("history authority");
        let history_identity = history.identity();
        let security = RecursiveSecurityBudgetManifestV2::authority_pinned().expect("security");
        let cadence_class = if u64::from(count) == history.cadence_blocks() {
            EpochCadenceClassV2::Production
        } else {
            EpochCadenceClassV2::BoundedSimulation
        };
        let start_height = 1;
        let mut transitions = Vec::with_capacity(count as usize);
        let mut previous_checkpoint = None;
        let start_root = digest("state", 0);
        let mut pre_root = start_root;
        for ordinal in 0..count {
            let next = transition(ordinal, start_height, previous_checkpoint, pre_root);
            previous_checkpoint = Some(next.inputs().checkpoint_id);
            pre_root = next.inputs().post_settlement_root;
            transitions.push(next);
        }
        let chain_context_digest = digest("context", 0);
        let predicate_digest = digest("predicate", 0);
        let frontier = EpochFrontierAuthorityV2::new(EpochFrontierAuthorityInputsV2 {
            cadence_class,
            epoch_index: 0,
            cadence_blocks: u64::from(count),
            start_root,
            chain_context_digest,
            predicate_digest,
            parameter_digest: history_identity.verifier_parameter_digest,
            verifier_bundle_digest: history_identity.verifier_bundle_digest,
        })
        .expect("frontier authority");
        EpochProofWorkManifestV2::new(EpochProofWorkManifestInputsV2 {
            cadence_class,
            epoch_index: 0,
            start_height,
            end_height: u64::from(count),
            transition_count: count,
            parameter_generation: history_identity.parameter_generation,
            runtime_profile_generation: identity.runtime_profile_generation,
            config_generation: identity.config_generation,
            authority_generation: identity.authority_generation,
            chain_context_digest,
            predicate_digest,
            parameter_digest: history_identity.verifier_parameter_digest,
            verifier_bundle_digest: history_identity.verifier_bundle_digest,
            security_budget_digest: security.digest(),
            config_digest: identity.config_digest,
            registry_digest: identity.registry_digest,
            runtime_profile_manifest_digest: identity.runtime_profile_manifest_digest,
            frontier_authority_digest: frontier.digest(),
            epoch_close_anchor_digest: digest("close", 0),
            nova_chain_root: Some(digest("nova", 0)),
            start_root,
            end_root: pre_root,
            transitions,
        })
        .expect("manifest")
    }

    fn chunk(manifest: &EpochProofWorkManifestV2) -> EpochTraceChunkV2 {
        let authority = manifest_frontier_authority(&manifest.inputs).expect("authority");
        let transitions = manifest.inputs.transitions.as_slice();
        let first = manifest.transition(0).expect("first").inputs();
        let last = manifest
            .transition(manifest.transition_count() - 1)
            .expect("last")
            .inputs();
        let transition_digests = transitions
            .iter()
            .map(EpochTransitionBindingV2::digest)
            .collect::<Vec<_>>();
        let slice_commitment =
            epoch_ordered_digest_root_v2(EPOCH_TRANSITION_SLICE_DOMAIN_V2, &transition_digests)
                .expect("slice commitment");
        let event_count = transitions.iter().fold(0_u64, |count, transition| {
            count + transition.inputs().event_count
        });
        let input_accumulator = epoch_stream_initial_accumulator(authority);
        let output_accumulator =
            transitions
                .iter()
                .copied()
                .fold(input_accumulator, |accumulator, transition| {
                    epoch_stream_step_accumulator(authority, accumulator, transition)
                });
        EpochTraceChunkV2::new(
            &authority,
            transitions,
            EpochTraceChunkInputsV2 {
                table: EpochAirTableV2::Transition,
                replica: 0,
                chunk_ordinal: 0,
                chunk_count: 1,
                first_transition: 0,
                last_transition: manifest.transition_count() - 1,
                transition_count: manifest.transition_count(),
                row_start: 0,
                row_count: u64::from(manifest.transition_count()),
                event_start: 0,
                event_count,
                frontier_authority_digest: authority.digest(),
                chain_context_digest: authority.chain_context_digest(),
                predicate_digest: authority.predicate_digest(),
                input_state_root: first.pre_settlement_root,
                output_state_root: last.post_settlement_root,
                input_accumulator,
                output_accumulator,
                input_slice_commitment: slice_commitment,
                parameter_digest: manifest.inputs.parameter_digest,
                verifier_bundle_digest: manifest.inputs.verifier_bundle_digest,
                security_budget_digest: manifest.inputs.security_budget_digest,
            },
        )
        .expect("chunk")
    }

    #[test]
    fn test_work_manifest_roundtrip_and_exact_count() {
        let manifest = manifest(2);
        assert_eq!(manifest.transition_count(), 2);
        let decoded =
            EpochProofWorkManifestV2::decode_canonical(manifest.canonical_bytes()).expect("decode");
        assert_eq!(decoded, manifest);
        let decoded_transition = decoded.transition(0).expect("decoded transition").inputs();
        assert_eq!(decoded_transition.uniqueness_row_count, 1);
        assert_eq!(decoded_transition.jmt_record_count, 1);
        assert_eq!(decoded_transition.jmt_envelope_count, 1);
        assert_eq!(decoded_transition.jmt_update_count, 1);
        assert_ne!(manifest.transition_root(), [0; 32]);
    }

    #[test]
    fn test_transition_accepts_noop_semantics_and_rejects_overclaimed_coverage() {
        let baseline = transition(0, 1, None, digest("state", 0)).inputs();

        let mut no_op = baseline;
        no_op.post_settlement_root = no_op.pre_settlement_root;
        no_op.post_definition_root = no_op.pre_definition_root;
        no_op.update_trace_digest = crate::settlement::noop_update_trace_digest();
        no_op.uniqueness_row_count = 0;
        no_op.jmt_record_count = 0;
        no_op.jmt_update_count = 0;
        EpochTransitionBindingV2::new(no_op)
            .expect("canonical no-op transition has no uniqueness or JMT micro rows");

        for mutate in [
            |inputs: &mut EpochTransitionInputsV2| inputs.jmt_envelope_count = 0,
            |inputs: &mut EpochTransitionInputsV2| inputs.jmt_envelope_count = 2,
            |inputs: &mut EpochTransitionInputsV2| inputs.jmt_update_count = 0,
            |inputs: &mut EpochTransitionInputsV2| {
                inputs.uniqueness_row_count = inputs.event_count;
                inputs.jmt_record_count = 1;
            },
        ] {
            let mut mutated = baseline;
            mutate(&mut mutated);
            assert!(
                EpochTransitionBindingV2::new(mutated).is_err(),
                "semantic coverage mutation must fail closed",
            );
        }
    }

    #[test]
    fn test_manifest_rejects_reordered_and_discontinuous_transitions() {
        let manifest = manifest(2);
        let mut inputs = manifest.inputs.clone();
        inputs.transitions.swap(0, 1);
        assert!(matches!(
            EpochProofWorkManifestV2::new(inputs),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StepReordered
            ))
        ));

        let mut inputs = manifest.inputs.clone();
        inputs.transitions[1].inputs.pre_settlement_root[0] ^= 1;
        inputs.transitions[1] = EpochTransitionBindingV2::new(inputs.transitions[1].inputs)
            .expect("mutated transition");
        assert!(matches!(
            EpochProofWorkManifestV2::new(inputs),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::PriorOutputMismatch
            ))
        ));
    }

    #[test]
    fn test_trace_chunk_roundtrip_and_mutations() {
        let manifest = manifest(2);
        let authority = manifest_frontier_authority(&manifest.inputs).expect("authority");
        let transitions = manifest.inputs.transitions.as_slice();
        let chunk = chunk(&manifest);
        assert_eq!(
            EpochTraceChunkV2::decode_canonical(&authority, transitions, chunk.canonical_bytes())
                .expect("decode"),
            chunk
        );
        let chunk_inputs = chunk.inputs();
        assert_eq!(
            &chunk.canonical_bytes()[EPOCH_CHUNK_INPUT_STATE_ROOT_BYTE_OFFSET_V2
                ..EPOCH_CHUNK_INPUT_STATE_ROOT_BYTE_OFFSET_V2 + 32],
            &chunk_inputs.input_state_root,
        );
        assert_eq!(
            &chunk.canonical_bytes()[EPOCH_CHUNK_OUTPUT_STATE_ROOT_BYTE_OFFSET_V2
                ..EPOCH_CHUNK_OUTPUT_STATE_ROOT_BYTE_OFFSET_V2 + 32],
            &chunk_inputs.output_state_root,
        );
        manifest
            .validate_closed_chunk(&chunk)
            .expect("closed manifest validates tentative chunk");

        let mut inputs = chunk.inputs();
        inputs.frontier_authority_digest[0] ^= 1;
        assert!(EpochTraceChunkV2::new(&authority, transitions, inputs).is_err());

        let mut inputs = chunk.inputs();
        inputs.first_transition = 1;
        assert!(EpochTraceChunkV2::new(&authority, transitions, inputs).is_err());

        let mut inputs = chunk.inputs();
        inputs.output_accumulator[0] ^= 1;
        assert!(matches!(
            EpochTraceChunkV2::new(&authority, transitions, inputs),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch
            ))
        ));
    }

    #[test]
    fn test_production_manifest_has_exact_2000_transitions() {
        let manifest = manifest(2_000);
        assert_eq!(manifest.cadence_class(), EpochCadenceClassV2::Production);
        assert_eq!(manifest.transition_count(), 2_000);
        assert_eq!(manifest.start_height(), 1);
        assert_eq!(manifest.end_height(), 2_000);
        assert_eq!(manifest.transition(1_999).expect("last").height(), 2_000);
    }

    #[test]
    fn test_stream_chunk_geometry_is_exact_and_bounded() {
        assert_eq!(
            epoch_trace_chunk_count(2_000).expect("production chunk count"),
            250
        );
        assert_eq!(
            epoch_trace_chunk_transition_range(2_000, 0).expect("first production chunk"),
            (0, 7)
        );
        assert_eq!(
            epoch_trace_chunk_transition_range(2_000, 249).expect("last production chunk"),
            (1_992, 1_999)
        );

        assert_eq!(epoch_trace_chunk_count(9).expect("tail chunk count"), 2);
        assert_eq!(
            epoch_trace_chunk_transition_range(9, 0).expect("full tail fixture chunk"),
            (0, 7)
        );
        assert_eq!(
            epoch_trace_chunk_transition_range(9, 1).expect("partial tail fixture chunk"),
            (8, 8)
        );
        assert!(epoch_trace_chunk_count(0).is_err());
        assert!(epoch_trace_chunk_transition_range(2_000, 250).is_err());
    }

    #[test]
    #[ignore = "real Plonky3 proving must run through plonky3_resource_worker.sh"]
    fn test_direct_trace_framing_actual_roundtrip() {
        let manifest = manifest(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2);
        let authority = manifest_frontier_authority(&manifest.inputs).expect("authority");
        let transitions = manifest.inputs.transitions.as_slice();
        let mut inputs = chunk(&manifest).inputs();
        inputs.table = EpochAirTableV2::TraceFraming;
        let statement =
            EpochTraceChunkV2::new(&authority, transitions, inputs).expect("framing statement");
        let event_bytes = transitions
            .iter()
            .try_fold(0_u64, |total, transition| {
                total.checked_add(transition.inputs().event_bytes)
            })
            .expect("bounded event-byte total");
        println!("Z00Z_PLONKY3_PHASE_V1 fixture_ready");
        let metrics = crate::checkpoint::plonky3::prove_epoch_trace_framing_smoke(
            &statement,
            transitions,
            event_bytes,
        )
        .expect("real direct trace-framing proof");
        assert!(metrics.table_count > 0);
        emit_smoke_metrics("trace_framing", metrics);
    }

    #[test]
    #[ignore = "real Plonky3 proving must run through plonky3_resource_worker.sh"]
    fn test_direct_packed_range_actual_roundtrip() {
        let manifest = manifest(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2);
        let authority = manifest_frontier_authority(&manifest.inputs).expect("authority");
        let transitions = manifest.inputs.transitions.as_slice();
        let bytes = [0_u8, 1, u8::MAX, 128, 3];
        let mut inputs = chunk(&manifest).inputs();
        inputs.table = EpochAirTableV2::PackedRange;
        inputs.row_start = 0;
        inputs.row_count = u64::try_from(bytes.len()).expect("bounded fixture bytes");
        let statement =
            EpochTraceChunkV2::new(&authority, transitions, inputs).expect("range statement");
        println!("Z00Z_PLONKY3_PHASE_V1 fixture_ready");
        let metrics =
            crate::checkpoint::plonky3::prove_epoch_packed_range_smoke(statement.clone(), &bytes)
                .expect("real direct packed-range proof");
        assert!(metrics.proof_bytes > 0);
        emit_smoke_metrics("packed_range", metrics);
        assert!(
            crate::checkpoint::plonky3::prove_epoch_packed_range_smoke(
                statement,
                &bytes[..bytes.len() - 1],
            )
            .is_err(),
            "statement-bound byte count must reject a shortened witness",
        );
    }

    #[test]
    #[ignore = "real Plonky3 proving must run through plonky3_resource_worker.sh"]
    fn test_direct_sha256_actual_roundtrip() {
        let manifest = manifest(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2);
        let authority = manifest_frontier_authority(&manifest.inputs).expect("authority");
        let transitions = manifest.inputs.transitions.as_slice();
        let mut inputs = chunk(&manifest).inputs();
        inputs.table = EpochAirTableV2::Sha256;
        inputs.row_start = 0;
        inputs.row_count = 1;
        let statement =
            EpochTraceChunkV2::new(&authority, transitions, inputs).expect("SHA statement");
        let mut block = [0_u8; 64];
        block[..3].copy_from_slice(b"abc");
        block[3] = 0x80;
        block[63] = 24;
        println!("Z00Z_PLONKY3_PHASE_V1 fixture_ready");
        let metrics = crate::checkpoint::plonky3::prove_epoch_sha256_smoke(
            statement,
            z00z_crypto::SHA256_IV_V2,
            block,
        )
        .expect("real direct SHA-256 proof");
        assert!(metrics.proof_bytes > 0);
        assert_eq!(metrics.trace_rows, 64);
        emit_smoke_metrics("sha256", metrics);
    }

    #[test]
    fn test_direct_jmt_split_insert_constraints() {
        let (header, records) = crate::settlement::jmt_mutation_case_circuit_transcripts_for_test()
            .into_iter()
            .find_map(|(label, header, records)| {
                (label == "split_insert").then_some((header, records))
            })
            .expect("split-insert JMT fixture");
        let header: [u8; crate::settlement::JMT_CIRCUIT_HEADER_BYTES_V2] =
            header.try_into().expect("fixed JMT header");
        let manifest = manifest(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2);
        let authority = manifest_frontier_authority(&manifest.inputs).expect("authority");
        let transitions = manifest.inputs.transitions.as_slice();
        let mut inputs = chunk(&manifest).inputs();
        inputs.table = EpochAirTableV2::JmtUpdate;
        inputs.row_start = 0;
        inputs.row_count = u64::try_from(records.len()).expect("bounded JMT records");
        let statement =
            EpochTraceChunkV2::new(&authority, transitions, inputs).expect("JMT statement");
        crate::checkpoint::plonky3::check_epoch_jmt_update_constraints(
            &statement, &header, &records,
        )
        .expect("split-insert JMT AIR constraints");
    }

    #[test]
    fn test_direct_jmt_noop_constraints_and_header_mutations() {
        let mut manifest_inputs = manifest(1).inputs;
        let mut transition_inputs = manifest_inputs.transitions[0].inputs();
        transition_inputs.post_settlement_root = transition_inputs.pre_settlement_root;
        transition_inputs.post_definition_root = transition_inputs.pre_definition_root;
        transition_inputs.update_trace_digest = crate::settlement::noop_update_trace_digest();
        transition_inputs.uniqueness_row_count = 0;
        transition_inputs.jmt_record_count = 0;
        transition_inputs.jmt_update_count = 0;
        manifest_inputs.transitions[0] =
            EpochTransitionBindingV2::new(transition_inputs).expect("no-op transition");
        manifest_inputs.end_root = transition_inputs.post_settlement_root;
        let manifest =
            EpochProofWorkManifestV2::new(manifest_inputs).expect("canonical no-op manifest");
        let authority = manifest_frontier_authority(&manifest.inputs).expect("authority");
        let transitions = manifest.inputs.transitions.as_slice();
        let mut inputs = chunk(&manifest).inputs();
        inputs.table = EpochAirTableV2::JmtUpdate;
        inputs.row_start = 0;
        inputs.row_count = 0;
        let statement =
            EpochTraceChunkV2::new(&authority, transitions, inputs).expect("no-op JMT statement");

        let mut header = [0_u8; crate::settlement::JMT_CIRCUIT_HEADER_BYTES_V2];
        header[0] = crate::settlement::JMT_UPDATE_TRACE_VERSION_V2;
        header[1] = crate::settlement::RootGeneration::SettlementV2.version();
        header[2] = crate::settlement::JMT_TRACE_NOOP_KIND_V2;
        header[3..35].copy_from_slice(&crate::settlement::noop_update_trace_digest());
        let records = Vec::<Vec<u8>>::new();
        crate::checkpoint::plonky3::check_epoch_jmt_update_constraints(
            &statement, &header, &records,
        )
        .expect("no-op JMT AIR constraints");

        let mut wrong_version = header;
        wrong_version[0] ^= 1;
        assert!(
            crate::checkpoint::plonky3::check_epoch_jmt_update_constraints(
                &statement,
                &wrong_version,
                &records,
            )
            .is_err(),
            "non-canonical JMT header version must fail closed",
        );

        let mut wrong_digest = header;
        wrong_digest[3] ^= 1;
        assert!(
            crate::checkpoint::plonky3::check_epoch_jmt_update_constraints(
                &statement,
                &wrong_digest,
                &records,
            )
            .is_err(),
            "non-canonical JMT no-op digest must fail closed",
        );

        let mut mutating_without_records = header;
        mutating_without_records[2] = crate::settlement::JMT_TRACE_MUTATING_KIND_V2;
        assert!(
            crate::checkpoint::plonky3::check_epoch_jmt_update_constraints(
                &statement,
                &mutating_without_records,
                &records,
            )
            .is_err(),
            "mutating JMT header without records must fail closed",
        );
    }

    #[test]
    #[ignore = "real Plonky3 proving must run through plonky3_resource_worker.sh"]
    fn test_direct_jmt_actual_roundtrip() {
        let (header, records) = crate::settlement::jmt_mutation_case_circuit_transcripts_for_test()
            .into_iter()
            .find_map(|(label, header, records)| {
                (label == "split_insert").then_some((header, records))
            })
            .expect("split-insert JMT fixture");
        let header: [u8; crate::settlement::JMT_CIRCUIT_HEADER_BYTES_V2] =
            header.try_into().expect("fixed JMT header");
        let manifest = manifest(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2);
        let authority = manifest_frontier_authority(&manifest.inputs).expect("authority");
        let transitions = manifest.inputs.transitions.as_slice();
        let mut inputs = chunk(&manifest).inputs();
        inputs.table = EpochAirTableV2::JmtUpdate;
        inputs.row_start = 0;
        inputs.row_count = u64::try_from(records.len()).expect("bounded JMT records");
        let statement =
            EpochTraceChunkV2::new(&authority, transitions, inputs).expect("JMT statement");
        println!("Z00Z_PLONKY3_PHASE_V1 fixture_ready");
        let metrics =
            crate::checkpoint::plonky3::prove_epoch_jmt_update_smoke(statement, header, &records)
                .expect("real direct JMT proof");
        assert!(metrics.proof_bytes > 0);
        assert_eq!(metrics.input_items, records.len());
        emit_smoke_metrics("jmt_update", metrics);
    }
}
