//! Independent streaming predicate evaluator for recursive checkpoint V2.

use z00z_crypto::{CheckpointSha256BlockV2, CheckpointSha256V2, CheckpointShaRole, SHA256_IV_V2};

use crate::backend::types::terminal_value_hash;
use crate::settlement::{
    derive_settlement_root_v2, RootGeneration, ScopeLeafKind, ScopeOpKind, SettlementStateRoot,
    SettlementStore, SettlementUpdateTraceCircuitDecoderV2,
};
use crate::CheckpointError;

use super::{
    recursive_context::{RecursiveAuthorityContextV2, RecursiveCheckpointBindingV2},
    recursive_semantics::{
        decode_flow_header, decode_flow_item, decode_hierarchy_promotion, decode_net_effect,
        decode_net_merge, decode_typed_checkpoint_commitment, decode_uniqueness_challenge,
        decode_uniqueness_precommit, decode_uniqueness_sorted_row, CanonicalFlowHeaderV2,
        CanonicalFlowItemV2, NetEffectKindV2, NetEffectV2, TypedCheckpointCommitmentKindV2,
        UniquenessChallengesV2, UniquenessListKindV2, UniquenessPassV2, UniquenessPrecommitV2,
        UniquenessSemanticRowV2, UniquenessSetKindV2,
    },
    recursive_statement::RecursiveTransitionStatementV2,
    recursive_trace::{
        decode_hash_control, decode_source_memory_write_control, decode_trace_chunk_control,
        hash_control_ordinal, structural_event_id, HashControlBindingV2, HashControlSchemaV2,
        HashControlStageV2, RecursiveTraceEventCountsV2, RecursiveTraceEventV2,
        RecursiveTraceOpcodeV2, RecursiveTracePrecommitV2, RecursiveTransitionTraceSourceV2,
        TRACE_CANONICAL_CHUNK_BYTES_V2, TRACE_EVENT_HEADER_BYTES_V2,
    },
};

/// Independently evaluated transition result; no caller-supplied validity flag exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvaluatedCheckpointTransitionV2 {
    settlement_root: SettlementStateRoot,
    statement: RecursiveTransitionStatementV2,
    trace: RecursiveTracePrecommitV2,
}

impl EvaluatedCheckpointTransitionV2 {
    #[must_use]
    pub const fn settlement_root(&self) -> SettlementStateRoot {
        self.settlement_root
    }
    #[must_use]
    pub const fn statement_digest(&self) -> [u8; 32] {
        self.statement.digest()
    }
    #[must_use]
    pub const fn trace(&self) -> RecursiveTracePrecommitV2 {
        self.trace
    }

    #[must_use]
    pub const fn statement(&self) -> RecursiveTransitionStatementV2 {
        self.statement
    }
}

/// Backend-neutral semantic evaluator for the V2 typed event grammar.
pub struct CheckpointTransitionConsistencyV2;

impl CheckpointTransitionConsistencyV2 {
    /// Evaluate the sealed stream without calling the traced update executor.
    ///
    /// It decodes every typed replay payload, verifies the exact sealed JMT
    /// envelope through its native proof verifier, checks the post-state owner,
    /// and rejects any opcode, payload, order, EOF, or root mismatch before a
    /// statement can be returned.
    pub(crate) fn evaluate_stream(
        source: &mut RecursiveTransitionTraceSourceV2,
        context: RecursiveAuthorityContextV2,
        checkpoint: RecursiveCheckpointBindingV2,
        store: &SettlementStore,
    ) -> Result<EvaluatedCheckpointTransitionV2, CheckpointError> {
        Self::evaluate_stream_inner(source, context, checkpoint, store, |_| {})
    }

    #[cfg(test)]
    pub(crate) fn evaluate_stream_with_events(
        source: &mut RecursiveTransitionTraceSourceV2,
        context: RecursiveAuthorityContextV2,
        checkpoint: RecursiveCheckpointBindingV2,
        store: &SettlementStore,
    ) -> Result<(EvaluatedCheckpointTransitionV2, Vec<RecursiveTraceEventV2>), CheckpointError>
    {
        let mut events = Vec::new();
        let evaluated = Self::evaluate_stream_inner(source, context, checkpoint, store, |event| {
            events.push(event.clone())
        })?;
        Ok((evaluated, events))
    }

    fn evaluate_stream_inner(
        source: &mut RecursiveTransitionTraceSourceV2,
        context: RecursiveAuthorityContextV2,
        checkpoint: RecursiveCheckpointBindingV2,
        store: &SettlementStore,
        mut on_accepted: impl FnMut(&RecursiveTraceEventV2),
    ) -> Result<EvaluatedCheckpointTransitionV2, CheckpointError> {
        let expected_precommit = source.sealed_precommit()?;
        let pre_uniqueness_context = source
            .pre_uniqueness_context()
            .ok_or(CheckpointError::Authority)?;
        let mut machine = TraceSemanticMachineV2::new(
            context,
            checkpoint,
            source.snapshot(),
            store,
            expected_precommit,
            pre_uniqueness_context.digest(),
            source.profile(),
        )?;
        let pass = source.event_pass_with_source_context(|event, source_record| {
            machine.accept(event, source_record)?;
            on_accepted(event);
            Ok(())
        })?;
        let declared_event_counts = pass.event_counts();
        let update_trace_digest = machine.update_trace_digest()?;
        let update_trace_count = machine.update_trace_count()?;
        let pre_definition_root = machine.pre_definition_root()?;
        let consumed_event_counts = machine.consumed_event_counts();
        machine.finish(declared_event_counts)?;

        let definition_root = store.recursive_v2_definition_root();
        let root = derive_settlement_root_v2(
            RootGeneration::SettlementV2,
            context.layout(),
            context.policy_digest(),
            definition_root,
        )
        .map_err(|_| CheckpointError::Root)?;
        let trace = pass.precommit();
        let statement = RecursiveTransitionStatementV2::build(
            context,
            source.snapshot(),
            checkpoint,
            source.profile(),
            root,
            pre_definition_root,
            definition_root,
            update_trace_digest,
            update_trace_count,
            trace,
            pre_uniqueness_context,
            declared_event_counts,
            consumed_event_counts,
        )?;
        Ok(EvaluatedCheckpointTransitionV2 {
            settlement_root: root,
            statement,
            trace,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TracePhaseV2 {
    #[default]
    BeforeBegin,
    ReplayInputs,
    ReplayOutputs,
    AwaitSpentOriginal,
    AwaitOutputOriginal,
    UniquenessCommitRows,
    UniquenessChallenge,
    UniquenessRows,
    NetMerge,
    JmtUpdate,
    HierarchyPromotion,
    Commit,
    Finalized,
}

fn semantic_row(item: &CanonicalFlowItemV2) -> UniquenessSemanticRowV2 {
    UniquenessSemanticRowV2 {
        definition_id: item.definition_id,
        serial_id: item.serial_id,
        terminal_id: item.terminal_id,
        leaf_value_hash: item.leaf_value_hash,
    }
}

fn global_semantic_order_is_valid(
    previous: Option<(UniquenessSetKindV2, UniquenessSemanticRowV2)>,
    current_set: UniquenessSetKindV2,
    current: UniquenessSemanticRowV2,
) -> bool {
    let Some((previous_set, previous)) = previous else {
        return true;
    };
    match previous.terminal_id.cmp(&current.terminal_id) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Greater => false,
        std::cmp::Ordering::Equal => {
            previous_set == UniquenessSetKindV2::Spent
                && current_set == UniquenessSetKindV2::Output
                && previous.same_storage_path(current)
        }
    }
}

/// A bounded semantic state machine. It stores only two streaming transcript
/// digests and the single bounded JMT envelope; it never keeps an event tape or
/// delegates semantic validity to the update executor.
struct TraceSemanticMachineV2<'a> {
    context: RecursiveAuthorityContextV2,
    pre_uniqueness_digest: [u8; 32],
    snapshot: super::recursive_context::RecursiveSnapshotHandleV2,
    store: &'a SettlementStore,
    max_source_chunk_count: u32,
    max_spent: u32,
    max_outputs: u32,
    authority_noop: bool,
    phase: TracePhaseV2,
    header: Option<CanonicalFlowHeaderV2>,
    inputs: u64,
    outputs: u64,
    expected_precommit: RecursiveTracePrecommitV2,
    uniqueness_precommit: Option<UniquenessPrecommitV2>,
    uniqueness_challenge: Option<UniquenessChallengesV2>,
    spent_original_ids: Option<CheckpointSha256V2>,
    output_original_ids: Option<CheckpointSha256V2>,
    spent_sorted_ids: Option<CheckpointSha256V2>,
    output_sorted_ids: Option<CheckpointSha256V2>,
    spent_product_ids: Option<CheckpointSha256V2>,
    output_product_ids: Option<CheckpointSha256V2>,
    spent_product_count: u64,
    output_product_count: u64,
    spent_sorted_count: u64,
    output_sorted_count: u64,
    prior_spent_sorted_id: Option<[u8; 32]>,
    prior_output_sorted_id: Option<[u8; 32]>,
    prior_sorted_row: Option<(UniquenessSetKindV2, UniquenessSemanticRowV2)>,
    pending_replay_row: Option<(UniquenessSetKindV2, UniquenessSemanticRowV2)>,
    net_pending_spent: Option<UniquenessSemanticRowV2>,
    net_pending_output: Option<UniquenessSemanticRowV2>,
    net_effect_count: u64,
    net_mutation_count: u64,
    net_closed: bool,
    typed_commitment_progress: usize,
    expected_typed_commitments: [[u8; 32]; 4],
    jmt_header_seen: bool,
    jmt_decoder: Option<SettlementUpdateTraceCircuitDecoderV2>,
    jmt_micro_digest: Option<CheckpointSha256V2>,
    jmt_micro_count: u64,
    consumed_event_counts: RecursiveTraceEventCountsV2,
    envelope_verified: bool,
    update_trace_digest: Option<[u8; 32]>,
    update_trace_count: Option<u32>,
    definition_root_transition: Option<([u8; 32], [u8; 32])>,
    pending_hash: Option<PendingHashV2>,
}

struct PendingHashV2 {
    source_ordinal: u64,
    source_opcode: RecursiveTraceOpcodeV2,
    source_object_id: [u8; 32],
    source_hash: [u8; 32],
    message_bytes: u64,
    block_count: u64,
    blocks_seen: u64,
    chaining_state: Option<[u32; 8]>,
    next_stage: HashControlStageV2,
    source_memory_window_open: bool,
    chunks: PendingCanonicalSourceChunksV2,
}

/// O(1), per-source canonical-byte cursor. The source record remains borrowed
/// only while its immediately derived controls are emitted, so the evaluator
/// rejects malformed schedules without retaining a second chunk vector. Nova
/// independently proves the same byte relation with O(1) state.
struct PendingCanonicalSourceChunksV2 {
    source_ordinal: u64,
    chunk_count: u32,
    next_chunk: u32,
}

impl PendingCanonicalSourceChunksV2 {
    fn from_source(
        source: &RecursiveTraceEventV2,
        max_source_chunk_count: u32,
    ) -> Result<Self, CheckpointError> {
        let chunk_count = source.canonical_chunk_count()?;
        if chunk_count > max_source_chunk_count {
            return Err(CheckpointError::EventOrder);
        }
        Ok(Self {
            source_ordinal: source.ordinal(),
            chunk_count,
            next_chunk: 0,
        })
    }

    fn accept(
        &mut self,
        chunk: super::recursive_trace::RecursiveTraceChunkControlV2,
        source: &RecursiveTraceEventV2,
    ) -> Result<(), CheckpointError> {
        self.matches_next(chunk, source)?;
        self.next_chunk = self
            .next_chunk
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        Ok(())
    }

    /// Validate the current canonical chunk without consuming it. The one
    /// derived memory writer uses this before the following SHA reader commits
    /// the cursor advance, proving both controls address the same source view.
    fn matches_next(
        &self,
        chunk: super::recursive_trace::RecursiveTraceChunkControlV2,
        source: &RecursiveTraceEventV2,
    ) -> Result<(), CheckpointError> {
        let expected = source.canonical_chunk(self.next_chunk)?;
        if chunk.source_ordinal != self.source_ordinal
            || source.ordinal() != self.source_ordinal
            || chunk.source_ordinal != expected.source_ordinal()
            || chunk.chunk_ordinal != expected.chunk_ordinal()
            || chunk.chunk_count != expected.chunk_count()
            || chunk.byte_count != expected.byte_count()
            || chunk.bytes != expected.bytes()
        {
            return Err(CheckpointError::EventOrder);
        }
        Ok(())
    }

    fn complete(&self) -> bool {
        self.next_chunk == self.chunk_count
    }
}

impl<'a> TraceSemanticMachineV2<'a> {
    fn new(
        context: RecursiveAuthorityContextV2,
        checkpoint: RecursiveCheckpointBindingV2,
        snapshot: super::recursive_context::RecursiveSnapshotHandleV2,
        store: &'a SettlementStore,
        expected_precommit: RecursiveTracePrecommitV2,
        pre_uniqueness_digest: [u8; 32],
        profile: &super::recursive_circuit::RecursiveCircuitProfileV2,
    ) -> Result<Self, CheckpointError> {
        let max_leaf_bytes = profile.max_leaf_bytes();
        let max_spent = profile.max_spent();
        let max_outputs = profile.max_outputs();
        // The profile is the single authority for this native resident bound.
        // Evaluator construction fails closed if its checked accounting cannot
        // be represented before any spool byte is replayed.
        profile.native_evaluator_resident_bytes()?;
        let max_source_record_bytes = u64::try_from(TRACE_EVENT_HEADER_BYTES_V2)
            .map_err(|_| CheckpointError::Limit)?
            .checked_add(u64::from(max_leaf_bytes))
            .ok_or(CheckpointError::Overflow)?;
        let max_source_chunk_count = max_source_record_bytes
            .checked_add(
                u64::try_from(TRACE_CANONICAL_CHUNK_BYTES_V2 - 1)
                    .map_err(|_| CheckpointError::Limit)?,
            )
            .ok_or(CheckpointError::Overflow)?
            / u64::try_from(TRACE_CANONICAL_CHUNK_BYTES_V2).map_err(|_| CheckpointError::Limit)?;
        let max_source_chunk_count =
            u32::try_from(max_source_chunk_count).map_err(|_| CheckpointError::Limit)?;
        if context.policy_digest() != store.bucket_policy().bucket_policy_id() {
            return Err(CheckpointError::CutoverAuthority);
        }
        // The evaluator is an independent acceptance machine, not merely a
        // verifier reached through `CanonicalCheckpointTransitionV2`.  Keep
        // the authority-selected no-op version gate here as well so a
        // crate-internal caller cannot turn the typed empty marker into a
        // second admission path by bypassing the orchestrator.
        if checkpoint.is_recursive_v2_noop()
            && !context.allows_noop_execution_input_version(checkpoint.exec_version())
        {
            return Err(CheckpointError::Authority);
        }
        Ok(Self {
            context,
            pre_uniqueness_digest,
            snapshot,
            store,
            max_source_chunk_count,
            max_spent,
            max_outputs,
            authority_noop: checkpoint.is_recursive_v2_noop(),
            phase: TracePhaseV2::BeforeBegin,
            header: None,
            inputs: 0,
            outputs: 0,
            expected_precommit,
            uniqueness_precommit: None,
            uniqueness_challenge: None,
            spent_original_ids: Some(CheckpointSha256V2::new(CheckpointShaRole::SpentOriginalIds)),
            output_original_ids: Some(CheckpointSha256V2::new(
                CheckpointShaRole::OutputOriginalIds,
            )),
            spent_sorted_ids: Some(CheckpointSha256V2::new(CheckpointShaRole::SpentSortedIds)),
            output_sorted_ids: Some(CheckpointSha256V2::new(CheckpointShaRole::OutputSortedIds)),
            spent_product_ids: Some(CheckpointSha256V2::new(CheckpointShaRole::SpentOriginalIds)),
            output_product_ids: Some(CheckpointSha256V2::new(
                CheckpointShaRole::OutputOriginalIds,
            )),
            spent_product_count: 0,
            output_product_count: 0,
            spent_sorted_count: 0,
            output_sorted_count: 0,
            prior_spent_sorted_id: None,
            prior_output_sorted_id: None,
            prior_sorted_row: None,
            pending_replay_row: None,
            net_pending_spent: None,
            net_pending_output: None,
            net_effect_count: 0,
            net_mutation_count: 0,
            net_closed: false,
            typed_commitment_progress: 0,
            expected_typed_commitments: [
                checkpoint.delta_root(),
                checkpoint.witness_root(),
                checkpoint.journal_digest(),
                checkpoint.checkpoint_link_digest(),
            ],
            jmt_header_seen: false,
            jmt_decoder: None,
            jmt_micro_digest: Some(CheckpointSha256V2::new(CheckpointShaRole::Trace)),
            jmt_micro_count: 0,
            consumed_event_counts: RecursiveTraceEventCountsV2::default(),
            envelope_verified: false,
            update_trace_digest: None,
            update_trace_count: None,
            definition_root_transition: None,
            pending_hash: None,
        })
    }

    fn accept(
        &mut self,
        event: &RecursiveTraceEventV2,
        source_record: Option<&RecursiveTraceEventV2>,
    ) -> Result<(), CheckpointError> {
        let accepted = match event.opcode() {
            RecursiveTraceOpcodeV2::BeginHash
            | RecursiveTraceOpcodeV2::ShaBlock
            | RecursiveTraceOpcodeV2::EndHash => self.accept_hash_control(event),
            RecursiveTraceOpcodeV2::SourceMemoryWrite => {
                let chunk = decode_source_memory_write_control(event)?;
                let pending = self
                    .pending_hash
                    .as_mut()
                    .ok_or(CheckpointError::EventOrder)?;
                if pending.next_stage != HashControlStageV2::Block
                    || pending.source_memory_window_open
                {
                    return Err(CheckpointError::EventOrder);
                }
                pending
                    .chunks
                    .matches_next(chunk, source_record.ok_or(CheckpointError::EventOrder)?)?;
                pending.source_memory_window_open = true;
                Ok(())
            }
            RecursiveTraceOpcodeV2::TraceChunk => {
                let chunk = decode_trace_chunk_control(event)?;
                let pending = self
                    .pending_hash
                    .as_mut()
                    .ok_or(CheckpointError::EventOrder)?;
                if pending.next_stage != HashControlStageV2::Block
                    || !pending.source_memory_window_open
                {
                    return Err(CheckpointError::EventOrder);
                }
                pending
                    .chunks
                    .accept(chunk, source_record.ok_or(CheckpointError::EventOrder)?)?;
                pending.source_memory_window_open = false;
                Ok(())
            }
            _ => {
                if self.pending_hash.is_some() {
                    return Err(CheckpointError::EventOrder);
                }
                self.accept_source(event)?;
                self.expect_hash(event)
            }
        };
        #[cfg(test)]
        if let Err(error) = &accepted {
            eprintln!(
                "recursive evaluator rejected source ordinal={} opcode={:?} phase={:?}: {error:?}",
                event.ordinal(),
                event.opcode(),
                self.phase,
            );
        }
        accepted?;
        self.consumed_event_counts.increment(event.opcode())?;
        Ok(())
    }

    fn accept_source(&mut self, event: &RecursiveTraceEventV2) -> Result<(), CheckpointError> {
        match event.opcode() {
            RecursiveTraceOpcodeV2::BeginBlock if self.phase == TracePhaseV2::BeforeBegin => {
                self.require_structural_id(event)?;
                if event.ordinal() != 0 {
                    return Err(CheckpointError::EventOrder);
                }
                let header = decode_flow_header(event.payload())?;
                if header.prev_root != *self.snapshot.root().as_bytes()
                    || header.post_root
                        != *self
                            .store
                            .settlement_root_v2(self.context.layout())
                            .map_err(|_| CheckpointError::Root)?
                            .as_bytes()
                    || (self.authority_noop && header.prev_root != header.post_root)
                    || header.spent_count > self.max_spent
                    || header.output_count > self.max_outputs
                {
                    return Err(CheckpointError::Root);
                }
                self.spent_original_ids
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(&header.spent_count.to_le_bytes())?;
                self.output_original_ids
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(&header.output_count.to_le_bytes())?;
                self.header = Some(header);
                self.phase = TracePhaseV2::ReplayInputs;
            }
            RecursiveTraceOpcodeV2::ReplayInput if self.phase == TracePhaseV2::ReplayInputs => {
                if self.uniqueness_precommit.is_none() {
                    return Err(CheckpointError::EventOrder);
                }
                let item = self.decode_replay_item(event, ScopeOpKind::Delete)?;
                self.check_input(&item)?;
                if self.inputs >= u64::from(self.max_spent) {
                    return Err(CheckpointError::Limit);
                }
                self.spent_original_ids
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(&semantic_row(&item).canonical_bytes())?;
                self.pending_replay_row = Some((UniquenessSetKindV2::Spent, semantic_row(&item)));
                self.inputs = self
                    .inputs
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                self.phase = TracePhaseV2::AwaitSpentOriginal;
            }
            RecursiveTraceOpcodeV2::ReplayOutput
                if matches!(
                    self.phase,
                    TracePhaseV2::ReplayInputs | TracePhaseV2::ReplayOutputs
                ) =>
            {
                if self.uniqueness_precommit.is_none() {
                    return Err(CheckpointError::EventOrder);
                }
                let item = self.decode_replay_item(event, ScopeOpKind::Put)?;
                self.check_output(&item)?;
                if self.outputs >= u64::from(self.max_outputs) {
                    return Err(CheckpointError::Limit);
                }
                self.output_original_ids
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(&semantic_row(&item).canonical_bytes())?;
                self.pending_replay_row = Some((UniquenessSetKindV2::Output, semantic_row(&item)));
                self.outputs = self
                    .outputs
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                self.phase = TracePhaseV2::AwaitOutputOriginal;
            }
            RecursiveTraceOpcodeV2::UniquenessPrecommit
                if self.phase == TracePhaseV2::ReplayInputs
                    && self.inputs == 0
                    && self.outputs == 0
                    && self.uniqueness_precommit.is_none() =>
            {
                self.require_structural_id(event)?;
                let precommit = decode_uniqueness_precommit(event.payload())?;
                if precommit.spent_original_digest
                    != self.expected_precommit.spent_original_ids_digest()
                    || precommit.spent_sorted_digest
                        != self.expected_precommit.spent_sorted_ids_digest()
                    || precommit.output_original_digest
                        != self.expected_precommit.output_original_ids_digest()
                    || precommit.output_sorted_digest
                        != self.expected_precommit.output_sorted_ids_digest()
                {
                    return Err(CheckpointError::Invariant);
                }
                let header = self.header.ok_or(CheckpointError::Invariant)?;
                if header.spent_count != precommit.spent_count
                    || header.output_count != precommit.output_count
                {
                    return Err(CheckpointError::Invariant);
                }
                self.uniqueness_precommit = Some(precommit);
                // The replay-derived original commitments above are an
                // independent equality check. The canonical commit pass now
                // reconstructs all four list digests from its own typed rows
                // before the challenge record is accepted.
                self.spent_product_ids =
                    Some(CheckpointSha256V2::new(CheckpointShaRole::SpentOriginalIds));
                self.output_product_ids = Some(CheckpointSha256V2::new(
                    CheckpointShaRole::OutputOriginalIds,
                ));
                self.spent_product_ids
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(&precommit.spent_count.to_le_bytes())?;
                self.output_product_ids
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(&precommit.output_count.to_le_bytes())?;
                self.spent_sorted_ids
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(&precommit.spent_count.to_le_bytes())?;
                self.output_sorted_ids
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(&precommit.output_count.to_le_bytes())?;
                self.phase = TracePhaseV2::ReplayInputs;
            }
            RecursiveTraceOpcodeV2::UniquenessChallenge
                if self.phase == TracePhaseV2::UniquenessCommitRows
                    || (self.phase == TracePhaseV2::ReplayInputs
                        && self.inputs == 0
                        && self.outputs == 0
                        && self.uniqueness_precommit.is_some()) =>
            {
                self.require_structural_id(event)?;
                let precommit = self
                    .uniqueness_precommit
                    .ok_or(CheckpointError::Invariant)?;
                let replay_spent_original_digest = self
                    .spent_original_ids
                    .take()
                    .ok_or(CheckpointError::Invariant)?
                    .finalize();
                let replay_output_original_digest = self
                    .output_original_ids
                    .take()
                    .ok_or(CheckpointError::Invariant)?
                    .finalize();
                let spent_original_digest = self
                    .spent_product_ids
                    .take()
                    .ok_or(CheckpointError::Invariant)?
                    .finalize();
                let output_original_digest = self
                    .output_product_ids
                    .take()
                    .ok_or(CheckpointError::Invariant)?
                    .finalize();
                let spent_sorted_digest = self
                    .spent_sorted_ids
                    .take()
                    .ok_or(CheckpointError::Invariant)?
                    .finalize();
                let output_sorted_digest = self
                    .output_sorted_ids
                    .take()
                    .ok_or(CheckpointError::Invariant)?
                    .finalize();
                if self.spent_product_count != self.inputs
                    || self.output_product_count != self.outputs
                    || self.spent_sorted_count != self.inputs
                    || self.output_sorted_count != self.outputs
                    || precommit.spent_count
                        != u32::try_from(self.inputs).map_err(|_| CheckpointError::Limit)?
                    || precommit.output_count
                        != u32::try_from(self.outputs).map_err(|_| CheckpointError::Limit)?
                    || precommit.spent_original_digest != replay_spent_original_digest
                    || precommit.output_original_digest != replay_output_original_digest
                    || precommit.spent_original_digest != spent_original_digest
                    || precommit.output_original_digest != output_original_digest
                    || precommit.spent_sorted_digest != spent_sorted_digest
                    || precommit.output_sorted_digest != output_sorted_digest
                {
                    return Err(CheckpointError::Invariant);
                }
                self.uniqueness_challenge = Some(decode_uniqueness_challenge(
                    event.payload(),
                    self.pre_uniqueness_digest,
                    RecursiveTraceOpcodeV2::grammar_digest(),
                    precommit,
                )?);
                // The commit pass counters have reached the declared row
                // counts. Product pass owns an independent reconstruction, so
                // both its hashers and counters must restart from the domain
                // prefix before any challenge-dependent row is accepted.
                self.spent_product_count = 0;
                self.output_product_count = 0;
                self.spent_sorted_count = 0;
                self.output_sorted_count = 0;
                self.prior_spent_sorted_id = None;
                self.prior_output_sorted_id = None;
                self.prior_sorted_row = None;
                self.spent_product_ids =
                    Some(CheckpointSha256V2::new(CheckpointShaRole::SpentOriginalIds));
                self.output_product_ids = Some(CheckpointSha256V2::new(
                    CheckpointShaRole::OutputOriginalIds,
                ));
                self.spent_sorted_ids =
                    Some(CheckpointSha256V2::new(CheckpointShaRole::SpentSortedIds));
                self.output_sorted_ids =
                    Some(CheckpointSha256V2::new(CheckpointShaRole::OutputSortedIds));
                self.spent_product_ids
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(&precommit.spent_count.to_le_bytes())?;
                self.output_product_ids
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(&precommit.output_count.to_le_bytes())?;
                self.spent_sorted_ids
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(&precommit.spent_count.to_le_bytes())?;
                self.output_sorted_ids
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(&precommit.output_count.to_le_bytes())?;
                self.spent_product_count = 0;
                self.output_product_count = 0;
                self.spent_sorted_count = 0;
                self.output_sorted_count = 0;
                self.prior_spent_sorted_id = None;
                self.prior_output_sorted_id = None;
                self.prior_sorted_row = None;
                self.phase = TracePhaseV2::UniquenessChallenge;
            }
            RecursiveTraceOpcodeV2::UniquenessSorted
                if matches!(
                    self.phase,
                    TracePhaseV2::AwaitSpentOriginal
                        | TracePhaseV2::AwaitOutputOriginal
                        | TracePhaseV2::ReplayOutputs
                        | TracePhaseV2::UniquenessCommitRows
                        | TracePhaseV2::UniquenessChallenge
                        | TracePhaseV2::UniquenessRows
                        | TracePhaseV2::NetMerge
                ) =>
            {
                self.require_structural_id(event)?;
                let (pass, set, list, row) = decode_uniqueness_sorted_row(event.payload())?;
                let id = row.terminal_id;
                let commit_pass = pass == UniquenessPassV2::Commit;
                let legal_row = match self.phase {
                    TracePhaseV2::AwaitSpentOriginal => {
                        commit_pass
                            && set == UniquenessSetKindV2::Spent
                            && list == UniquenessListKindV2::Original
                    }
                    TracePhaseV2::AwaitOutputOriginal => {
                        commit_pass
                            && set == UniquenessSetKindV2::Output
                            && list == UniquenessListKindV2::Original
                    }
                    TracePhaseV2::ReplayOutputs | TracePhaseV2::UniquenessCommitRows => {
                        commit_pass && list == UniquenessListKindV2::Sorted
                    }
                    TracePhaseV2::UniquenessChallenge
                    | TracePhaseV2::UniquenessRows
                    | TracePhaseV2::NetMerge => !commit_pass,
                    _ => false,
                };
                if !legal_row {
                    return Err(CheckpointError::EventOrder);
                }
                if commit_pass
                    && list == UniquenessListKindV2::Original
                    && self.pending_replay_row.take() != Some((set, row))
                {
                    return Err(CheckpointError::Invariant);
                }
                match (list, set) {
                    (UniquenessListKindV2::Original, UniquenessSetKindV2::Spent) => {
                        if self.output_product_count != 0
                            || self.spent_sorted_count != 0
                            || self.spent_product_count >= self.inputs
                        {
                            return Err(CheckpointError::Invariant);
                        }
                        self.spent_product_ids
                            .as_mut()
                            .ok_or(CheckpointError::Invariant)?
                            .update_part(&row.canonical_bytes())?;
                        self.spent_product_count = self
                            .spent_product_count
                            .checked_add(1)
                            .ok_or(CheckpointError::Overflow)?;
                    }
                    (UniquenessListKindV2::Original, UniquenessSetKindV2::Output) => {
                        if self.spent_product_count != self.inputs
                            || self.spent_sorted_count != 0
                            || self.output_product_count >= self.outputs
                        {
                            return Err(CheckpointError::Invariant);
                        }
                        self.output_product_ids
                            .as_mut()
                            .ok_or(CheckpointError::Invariant)?
                            .update_part(&row.canonical_bytes())?;
                        self.output_product_count = self
                            .output_product_count
                            .checked_add(1)
                            .ok_or(CheckpointError::Overflow)?;
                    }
                    (UniquenessListKindV2::Sorted, UniquenessSetKindV2::Spent) => {
                        if self.spent_product_count != self.inputs
                            || self.output_product_count != self.outputs
                            || self.spent_sorted_count >= self.inputs
                            || self.prior_spent_sorted_id.is_some_and(|prior| prior >= id)
                            || (!commit_pass
                                && !global_semantic_order_is_valid(self.prior_sorted_row, set, row))
                        {
                            return Err(CheckpointError::DuplicateIdentifier);
                        }
                        self.spent_sorted_ids
                            .as_mut()
                            .ok_or(CheckpointError::Invariant)?
                            .update_part(&row.canonical_bytes())?;
                        self.prior_spent_sorted_id = Some(id);
                        if !commit_pass {
                            self.prior_sorted_row = Some((set, row));
                        }
                        self.spent_sorted_count = self
                            .spent_sorted_count
                            .checked_add(1)
                            .ok_or(CheckpointError::Overflow)?;
                    }
                    (UniquenessListKindV2::Sorted, UniquenessSetKindV2::Output) => {
                        if self.spent_product_count != self.inputs
                            || self.output_product_count != self.outputs
                            || self.output_sorted_count >= self.outputs
                            || self.prior_output_sorted_id.is_some_and(|prior| prior >= id)
                            || (!commit_pass
                                && !global_semantic_order_is_valid(self.prior_sorted_row, set, row))
                        {
                            return Err(CheckpointError::DuplicateIdentifier);
                        }
                        self.output_sorted_ids
                            .as_mut()
                            .ok_or(CheckpointError::Invariant)?
                            .update_part(&row.canonical_bytes())?;
                        self.prior_output_sorted_id = Some(id);
                        if !commit_pass {
                            self.prior_sorted_row = Some((set, row));
                        }
                        self.output_sorted_count = self
                            .output_sorted_count
                            .checked_add(1)
                            .ok_or(CheckpointError::Overflow)?;
                    }
                }
                if !commit_pass && list == UniquenessListKindV2::Sorted {
                    if self.net_closed {
                        return Err(CheckpointError::EventOrder);
                    }
                    match set {
                        UniquenessSetKindV2::Spent => {
                            if self.net_pending_spent.is_some() || self.net_pending_output.is_some()
                            {
                                return Err(CheckpointError::Invariant);
                            }
                            self.net_pending_spent = Some(row);
                        }
                        UniquenessSetKindV2::Output => {
                            if self.net_pending_output.is_some()
                                || self
                                    .net_pending_spent
                                    .is_some_and(|spent| spent.terminal_id != row.terminal_id)
                            {
                                return Err(CheckpointError::Invariant);
                            }
                            self.net_pending_output = Some(row);
                        }
                    }
                }
                self.phase = match self.phase {
                    TracePhaseV2::AwaitSpentOriginal => TracePhaseV2::ReplayInputs,
                    TracePhaseV2::AwaitOutputOriginal => TracePhaseV2::ReplayOutputs,
                    TracePhaseV2::ReplayOutputs | TracePhaseV2::UniquenessCommitRows => {
                        TracePhaseV2::UniquenessCommitRows
                    }
                    TracePhaseV2::UniquenessChallenge | TracePhaseV2::UniquenessRows => {
                        TracePhaseV2::UniquenessRows
                    }
                    TracePhaseV2::NetMerge => TracePhaseV2::NetMerge,
                    _ => return Err(CheckpointError::EventOrder),
                };
            }
            RecursiveTraceOpcodeV2::NetMerge
                if self.phase == TracePhaseV2::UniquenessRows
                    || self.phase == TracePhaseV2::NetMerge
                    || (self.phase == TracePhaseV2::UniquenessChallenge
                        && self.inputs == 0
                        && self.outputs == 0) =>
            {
                self.require_structural_id(event)?;
                let effect = decode_net_effect(event.payload())?;
                if effect.kind == NetEffectKindV2::Close {
                    if self.net_closed
                        || self.net_pending_spent.is_some()
                        || self.net_pending_output.is_some()
                    {
                        return Err(CheckpointError::Invariant);
                    }
                    let precommit = self
                        .uniqueness_precommit
                        .ok_or(CheckpointError::Invariant)?;
                    let spent_product_digest = self
                        .spent_product_ids
                        .take()
                        .ok_or(CheckpointError::Invariant)?
                        .finalize();
                    let output_product_digest = self
                        .output_product_ids
                        .take()
                        .ok_or(CheckpointError::Invariant)?
                        .finalize();
                    let spent_sorted_digest = self
                        .spent_sorted_ids
                        .take()
                        .ok_or(CheckpointError::Invariant)?
                        .finalize();
                    let output_sorted_digest = self
                        .output_sorted_ids
                        .take()
                        .ok_or(CheckpointError::Invariant)?
                        .finalize();
                    if self.spent_product_count != self.inputs
                        || self.output_product_count != self.outputs
                        || self.spent_sorted_count != self.inputs
                        || self.output_sorted_count != self.outputs
                        || precommit.spent_original_digest != spent_product_digest
                        || precommit.output_original_digest != output_product_digest
                        || precommit.spent_sorted_digest != spent_sorted_digest
                        || precommit.output_sorted_digest != output_sorted_digest
                    {
                        return Err(CheckpointError::Invariant);
                    }
                    decode_net_merge(
                        event.payload(),
                        precommit,
                        self.uniqueness_challenge
                            .ok_or(CheckpointError::Invariant)?,
                    )?;
                    self.net_closed = true;
                } else {
                    if self.net_closed {
                        return Err(CheckpointError::EventOrder);
                    }
                    let expected = NetEffectV2::from_rows(
                        self.net_pending_spent.take(),
                        self.net_pending_output.take(),
                    )?;
                    if effect != expected {
                        return Err(CheckpointError::Invariant);
                    }
                    self.net_effect_count = self
                        .net_effect_count
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                    if effect.kind != NetEffectKindV2::Unchanged {
                        self.net_mutation_count = self
                            .net_mutation_count
                            .checked_add(1)
                            .ok_or(CheckpointError::Overflow)?;
                    }
                }
                self.phase = TracePhaseV2::NetMerge;
            }
            RecursiveTraceOpcodeV2::JmtUpdate if self.phase == TracePhaseV2::NetMerge => {
                self.require_structural_id(event)?;
                if !self.net_closed
                    || (self.authority_noop && (self.inputs != 0 || self.outputs != 0))
                    || self.jmt_header_seen
                {
                    return Err(CheckpointError::Invariant);
                }
                self.jmt_decoder = Some(
                    SettlementUpdateTraceCircuitDecoderV2::new(event.payload())
                        .map_err(|_| CheckpointError::Canonical)?,
                );
                self.jmt_header_seen = true;
                self.phase = TracePhaseV2::JmtUpdate;
            }
            RecursiveTraceOpcodeV2::JmtMicroOp if self.phase == TracePhaseV2::JmtUpdate => {
                self.require_structural_id(event)?;
                if !self.jmt_header_seen || self.authority_noop {
                    return Err(CheckpointError::Invariant);
                }
                self.jmt_decoder
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .accept(event.payload())
                    .map_err(|_| CheckpointError::Canonical)?;
                self.jmt_micro_digest
                    .as_mut()
                    .ok_or(CheckpointError::Invariant)?
                    .update_part(event.payload())?;
                self.jmt_micro_count = self
                    .jmt_micro_count
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
            }
            RecursiveTraceOpcodeV2::PromoteChildRoot if self.phase == TracePhaseV2::JmtUpdate => {
                self.require_structural_id(event)?;
                self.verify_jmt_envelope()?;
                decode_hierarchy_promotion(
                    event.payload(),
                    self.store.recursive_v2_definition_root(),
                    self.update_trace_digest()?,
                )?;
                self.phase = TracePhaseV2::HierarchyPromotion;
            }
            RecursiveTraceOpcodeV2::CommitTypedEvent
                if matches!(
                    self.phase,
                    TracePhaseV2::HierarchyPromotion | TracePhaseV2::Commit
                ) =>
            {
                self.require_structural_id(event)?;
                let (kind, digest) = decode_typed_checkpoint_commitment(event.payload())?;
                let expected_kind = TypedCheckpointCommitmentKindV2::ALL
                    .get(self.typed_commitment_progress)
                    .copied()
                    .ok_or(CheckpointError::EventOrder)?;
                let expected_digest = self
                    .expected_typed_commitments
                    .get(self.typed_commitment_progress)
                    .copied()
                    .ok_or(CheckpointError::EventOrder)?;
                if kind != expected_kind || digest != expected_digest {
                    return Err(CheckpointError::Authority);
                }
                self.typed_commitment_progress = self
                    .typed_commitment_progress
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                self.phase = TracePhaseV2::Commit;
            }
            RecursiveTraceOpcodeV2::FinalizeBlock if self.phase == TracePhaseV2::Commit => {
                self.require_structural_id(event)?;
                let header = decode_flow_header(event.payload())?;
                if Some(header) != self.header
                    || header.prev_root != *self.snapshot.root().as_bytes()
                    || header.post_root
                        != *self
                            .store
                            .settlement_root_v2(self.context.layout())
                            .map_err(|_| CheckpointError::Root)?
                            .as_bytes()
                    || self.uniqueness_precommit.is_none()
                    || self.uniqueness_challenge.is_none()
                    || self.typed_commitment_progress != TypedCheckpointCommitmentKindV2::ALL.len()
                    || (!self.authority_noop && (self.inputs == 0 || self.outputs == 0))
                    || (self.authority_noop && (self.inputs != 0 || self.outputs != 0))
                    || !self.jmt_header_seen
                {
                    return Err(CheckpointError::Invariant);
                }
                self.phase = TracePhaseV2::Finalized;
            }
            _ => return Err(CheckpointError::EventOrder),
        }
        Ok(())
    }

    fn consumed_event_counts(&self) -> RecursiveTraceEventCountsV2 {
        self.consumed_event_counts
    }

    fn finish(
        self,
        declared_event_counts: RecursiveTraceEventCountsV2,
    ) -> Result<(), CheckpointError> {
        if self.phase != TracePhaseV2::Finalized
            || !self.envelope_verified
            || self.pending_hash.is_some()
            || !self.net_closed
            || self.net_pending_spent.is_some()
            || self.net_pending_output.is_some()
        {
            return Err(CheckpointError::TraceState);
        }
        if self.net_effect_count > self.inputs + self.outputs
            || ((self.inputs + self.outputs != 0) != (self.net_effect_count != 0))
        {
            return Err(CheckpointError::Invariant);
        }
        if self.consumed_event_counts != declared_event_counts {
            return Err(CheckpointError::EventOrder);
        }
        Ok(())
    }

    fn decode_replay_item(
        &self,
        event: &RecursiveTraceEventV2,
        expected_kind: ScopeOpKind,
    ) -> Result<CanonicalFlowItemV2, CheckpointError> {
        let item = decode_flow_item(event.payload())?;
        if item.op_kind != expected_kind || item.terminal_id != event.object_id() {
            return Err(CheckpointError::Canonical);
        }
        Ok(item)
    }

    fn check_input(&self, item: &CanonicalFlowItemV2) -> Result<(), CheckpointError> {
        if item.leaf_kind != ScopeLeafKind::Terminal
            || item.first_definition
            || item.first_serial
            || item.first_object
        {
            return Err(CheckpointError::Invariant);
        }
        Ok(())
    }

    fn check_output(&self, item: &CanonicalFlowItemV2) -> Result<(), CheckpointError> {
        if item.leaf_kind != ScopeLeafKind::Terminal {
            return Err(CheckpointError::Invariant);
        }
        let post_item = self
            .store
            .get_settlement_item(&item.path())
            .map_err(|_| CheckpointError::Storage)?
            .ok_or(CheckpointError::Invariant)?;
        if post_item.terminal_leaf().is_err()
            || terminal_value_hash(post_item.leaf())
                .map_err(|_| CheckpointError::Storage)?
                .0
                != item.leaf_value_hash
        {
            return Err(CheckpointError::Invariant);
        }
        Ok(())
    }

    fn verify_jmt_envelope(&mut self) -> Result<(), CheckpointError> {
        if self.envelope_verified || !self.jmt_header_seen {
            return Err(CheckpointError::Invariant);
        }
        let envelope = self
            .jmt_decoder
            .take()
            .ok_or(CheckpointError::Invariant)?
            .finish()
            .map_err(|_| CheckpointError::Canonical)?;
        if envelope.is_noop() != self.authority_noop
            || (!self.authority_noop && envelope.updates_empty())
            || (self.authority_noop && !envelope.updates_empty())
        {
            return Err(CheckpointError::Canonical);
        }
        if (!self.authority_noop && self.jmt_micro_count == 0)
            || (self.authority_noop && self.jmt_micro_count != 0)
        {
            return Err(CheckpointError::TraceState);
        }
        if !self.authority_noop
            && self
                .jmt_micro_digest
                .take()
                .ok_or(CheckpointError::Invariant)?
                .finalize()
                != envelope.trace_digest()
        {
            return Err(CheckpointError::Invariant);
        }
        let terminal_operation_count = envelope
            .terminal_operation_count()
            .map_err(|_| CheckpointError::Canonical)?;
        let update_trace_digest = envelope.trace_digest();
        let update_trace_count = envelope.update_count();
        let definition_root_transition = envelope
            .verify_hierarchy_semantics(self.store.recursive_v2_definition_root())
            .map_err(|_| CheckpointError::Canonical)?;
        let envelope_pre_settlement_root = derive_settlement_root_v2(
            RootGeneration::SettlementV2,
            self.context.layout(),
            self.context.policy_digest(),
            definition_root_transition.0,
        )
        .map_err(|_| CheckpointError::Root)?;
        if definition_root_transition.0 != self.snapshot.pre_definition_root()
            || envelope_pre_settlement_root != self.snapshot.root()
        {
            return Err(CheckpointError::Root);
        }
        if terminal_operation_count != self.net_mutation_count {
            return Err(CheckpointError::EventOrder);
        }
        self.definition_root_transition = Some(definition_root_transition);
        self.update_trace_digest = Some(update_trace_digest);
        self.update_trace_count = Some(update_trace_count);
        self.envelope_verified = true;
        Ok(())
    }

    fn pre_definition_root(&self) -> Result<[u8; 32], CheckpointError> {
        self.definition_root_transition
            .map(|(pre_definition_root, _)| pre_definition_root)
            .ok_or(CheckpointError::Invariant)
    }

    fn update_trace_digest(&self) -> Result<[u8; 32], CheckpointError> {
        if !self.envelope_verified {
            return Err(CheckpointError::Invariant);
        }
        self.update_trace_digest.ok_or(CheckpointError::Invariant)
    }

    fn update_trace_count(&self) -> Result<u32, CheckpointError> {
        if !self.envelope_verified {
            return Err(CheckpointError::Invariant);
        }
        self.update_trace_count.ok_or(CheckpointError::Invariant)
    }

    fn require_structural_id(&self, event: &RecursiveTraceEventV2) -> Result<(), CheckpointError> {
        if event.object_id()
            != structural_event_id(event.opcode(), event.ordinal(), event.payload())
        {
            return Err(CheckpointError::Canonical);
        }
        Ok(())
    }

    fn expect_hash(&mut self, event: &RecursiveTraceEventV2) -> Result<(), CheckpointError> {
        let (message_bytes, block_count) = event.hash_geometry()?;
        if block_count == 0 {
            return Err(CheckpointError::Invariant);
        }
        self.pending_hash = Some(PendingHashV2 {
            source_ordinal: event.ordinal(),
            source_opcode: event.opcode(),
            source_object_id: event.object_id(),
            source_hash: event.hash_binding()?,
            message_bytes,
            block_count,
            blocks_seen: 0,
            chaining_state: None,
            next_stage: HashControlStageV2::Begin,
            source_memory_window_open: false,
            chunks: PendingCanonicalSourceChunksV2::from_source(
                event,
                self.max_source_chunk_count,
            )?,
        });
        Ok(())
    }

    fn accept_hash_control(
        &mut self,
        event: &RecursiveTraceEventV2,
    ) -> Result<(), CheckpointError> {
        self.require_structural_id(event)?;
        let control = decode_hash_control(event)?;
        // The source owner regenerates the tagged whole-trace transcript from
        // its protected canonical spool and verifies it against the sealed
        // precommit before this evaluator receives the controls.  It is not a
        // second source-record hash schedule for this independent semantic
        // machine; Nova consumes the same decoded controls in its sole lane.
        if control.schema != HashControlSchemaV2::SourceRecord {
            return Ok(());
        }
        let completed = {
            let pending = self
                .pending_hash
                .as_mut()
                .ok_or(CheckpointError::EventOrder)?;
            if pending.source_memory_window_open {
                return Err(CheckpointError::EventOrder);
            }
            let expected_opcode = match pending.next_stage {
                HashControlStageV2::Begin => RecursiveTraceOpcodeV2::BeginHash,
                HashControlStageV2::Block => RecursiveTraceOpcodeV2::ShaBlock,
                HashControlStageV2::End => RecursiveTraceOpcodeV2::EndHash,
            };
            if event.opcode() != expected_opcode || !same_hash_binding(&control, pending) {
                return Err(CheckpointError::Canonical);
            }
            match control.stage {
                HashControlStageV2::Begin => {
                    if event.ordinal() != hash_control_ordinal(pending.source_ordinal, 0)?
                        || control.block.is_some()
                    {
                        return Err(CheckpointError::Canonical);
                    }
                    pending.next_stage = HashControlStageV2::Block;
                    false
                }
                HashControlStageV2::Block => {
                    let block = control.block.ok_or(CheckpointError::Canonical)?;
                    let expected_offset = pending
                        .blocks_seen
                        .checked_mul(64)
                        .ok_or(CheckpointError::Overflow)?;
                    let expected_final = pending
                        .blocks_seen
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?
                        == pending.block_count;
                    if pending.blocks_seen >= pending.block_count
                        || block.index != pending.blocks_seen
                        || block.byte_offset != expected_offset
                        || block.final_block != expected_final
                        || event.ordinal()
                            != hash_control_ordinal(
                                pending.source_ordinal,
                                block
                                    .index
                                    .checked_add(1)
                                    .ok_or(CheckpointError::Overflow)?,
                            )?
                        || block.chaining_before != pending.chaining_state.unwrap_or(SHA256_IV_V2)
                        || !block.verifies_transition()
                    {
                        return Err(CheckpointError::Canonical);
                    }
                    pending.chaining_state = Some(block.chaining_after);
                    pending.blocks_seen = pending
                        .blocks_seen
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                    if pending.blocks_seen == pending.block_count {
                        pending.next_stage = HashControlStageV2::End;
                    }
                    false
                }
                HashControlStageV2::End => {
                    if event.ordinal()
                        != hash_control_ordinal(
                            pending.source_ordinal,
                            pending
                                .block_count
                                .checked_add(1)
                                .ok_or(CheckpointError::Overflow)?,
                        )?
                        || control.block.is_some()
                        || pending.blocks_seen != pending.block_count
                        || pending.source_memory_window_open
                        || !pending.chunks.complete()
                        || CheckpointSha256BlockV2::digest_from_chaining(
                            &pending.chaining_state.ok_or(CheckpointError::Invariant)?,
                        ) != pending.source_hash
                    {
                        return Err(CheckpointError::Canonical);
                    }
                    true
                }
            }
        };
        if completed {
            self.pending_hash = None;
        }
        Ok(())
    }
}

fn same_hash_binding(control: &HashControlBindingV2, pending: &PendingHashV2) -> bool {
    control.schema == HashControlSchemaV2::SourceRecord
        && control.stage == pending.next_stage
        && control.source_ordinal == pending.source_ordinal
        && control.source_opcode == pending.source_opcode
        && control.source_object_id == pending.source_object_id
        && control.source_hash == pending.source_hash
        && control.message_bytes == pending.message_bytes
        && control.block_count == pending.block_count
}
