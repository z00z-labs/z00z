//! Independent streaming predicate evaluator for recursive checkpoint V2.

use z00z_crypto::{CheckpointSha256BlockV2, CheckpointSha256V2, CheckpointShaRole, SHA256_IV_V2};

use crate::settlement::{
    derive_settlement_root_v2, RootGeneration, ScopeLeafKind, ScopeOpKind, SettlementStateRoot,
    SettlementStore, SettlementUpdateTraceEnvelopeV2,
};

use super::{
    recursive_context::{RecursiveAuthorityContextV2, RecursiveCheckpointBindingV2},
    recursive_reject::RecursiveV2Error,
    recursive_semantics::{
        decode_flow_header, decode_flow_item, decode_hierarchy_promotion, decode_net_merge,
        decode_uniqueness_challenge, decode_uniqueness_precommit, uniqueness_precommit_from_ids,
        CanonicalFlowHeaderV2, CanonicalFlowItemV2, UniquenessPrecommitV2,
    },
    recursive_statement::RecursiveTransitionStatementV2,
    recursive_trace::{
        decode_hash_control, decode_trace_chunk_control, hash_control_ordinal, structural_event_id,
        HashControlBindingV2, HashControlSchemaV2, HashControlStageV2, RecursiveTraceEventV2,
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
    ) -> Result<EvaluatedCheckpointTransitionV2, RecursiveV2Error> {
        let mut machine = TraceSemanticMachineV2::new(
            context,
            checkpoint,
            source.snapshot(),
            store,
            source.max_content_bytes(),
            source.profile().max_leaf_bytes(),
            source.profile().max_spent(),
            source.profile().max_outputs(),
        )?;
        let pass = source.event_pass(|event| machine.accept(event))?;
        let update_trace_digest = machine.update_trace_digest()?;
        let update_trace_count = machine.update_trace_count()?;
        machine.finish()?;

        let definition_root = store.recursive_v2_definition_root();
        let root = derive_settlement_root_v2(
            RootGeneration::SettlementV2,
            context.layout(),
            context.policy_digest(),
            definition_root,
        )
        .map_err(|_| RecursiveV2Error::Root)?;
        let trace = pass.precommit();
        let statement = RecursiveTransitionStatementV2::build(
            context,
            source.snapshot(),
            checkpoint,
            source.profile(),
            root,
            definition_root,
            update_trace_digest,
            update_trace_count,
            trace,
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
    UniquenessPrecommit,
    UniquenessChallenge,
    NetMerge,
    JmtUpdate,
    HierarchyPromotion,
    Commit,
    Finalized,
}

/// A bounded semantic state machine. It stores only two streaming transcript
/// digests and the single bounded JMT envelope; it never keeps an event tape or
/// delegates semantic validity to the update executor.
struct TraceSemanticMachineV2<'a> {
    context: RecursiveAuthorityContextV2,
    snapshot: super::recursive_context::RecursiveSnapshotHandleV2,
    store: &'a SettlementStore,
    max_content_bytes: u64,
    max_source_chunk_count: u32,
    max_spent: u32,
    max_outputs: u32,
    authority_noop: bool,
    phase: TracePhaseV2,
    header: Option<CanonicalFlowHeaderV2>,
    inputs: u64,
    outputs: u64,
    spent_ids: Vec<[u8; 32]>,
    output_ids: Vec<[u8; 32]>,
    uniqueness_precommit: Option<UniquenessPrecommitV2>,
    uniqueness_challenge: Option<[u8; 32]>,
    commits: u64,
    jmt_chunks: u64,
    jmt_payload: Vec<u8>,
    envelope_verified: bool,
    pending_chunks: Option<PendingCanonicalSourceChunksV2>,
    pending_hash: Option<PendingHashV2>,
    replay_transcript: CheckpointSha256V2,
    commit_transcript: CheckpointSha256V2,
}

#[derive(Clone, Copy)]
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
}

/// Bounded, per-source canonical-byte feeder state.  The source itself is
/// still read from the one private spool; retaining at most one source record
/// worth of fixed 64-byte controls lets the independent evaluator reject a
/// reordered, omitted, duplicated, or substituted feeder before it reaches a
/// semantic source transition.
struct PendingCanonicalSourceChunksV2 {
    source_ordinal: u64,
    chunk_count: u32,
    chunks: Vec<super::recursive_trace::RecursiveTraceChunkControlV2>,
}

impl PendingCanonicalSourceChunksV2 {
    fn begin(
        chunk: super::recursive_trace::RecursiveTraceChunkControlV2,
        max_source_chunk_count: u32,
    ) -> Result<Self, RecursiveV2Error> {
        if chunk.chunk_ordinal != 0 || chunk.chunk_count > max_source_chunk_count {
            return Err(RecursiveV2Error::EventOrder);
        }
        let capacity = usize::try_from(chunk.chunk_count).map_err(|_| RecursiveV2Error::Limit)?;
        let mut chunks = Vec::new();
        chunks
            .try_reserve_exact(capacity)
            .map_err(|_| RecursiveV2Error::Limit)?;
        chunks.push(chunk);
        Ok(Self {
            source_ordinal: chunk.source_ordinal,
            chunk_count: chunk.chunk_count,
            chunks,
        })
    }

    fn push(
        &mut self,
        chunk: super::recursive_trace::RecursiveTraceChunkControlV2,
    ) -> Result<(), RecursiveV2Error> {
        let expected = u32::try_from(self.chunks.len()).map_err(|_| RecursiveV2Error::Limit)?;
        if chunk.source_ordinal != self.source_ordinal
            || chunk.chunk_count != self.chunk_count
            || chunk.chunk_ordinal != expected
            || expected >= self.chunk_count
        {
            return Err(RecursiveV2Error::EventOrder);
        }
        self.chunks.push(chunk);
        Ok(())
    }

    fn matches_source(&self, source: &RecursiveTraceEventV2) -> Result<bool, RecursiveV2Error> {
        if source.ordinal() != self.source_ordinal
            || usize::try_from(self.chunk_count).map_err(|_| RecursiveV2Error::Limit)?
                != self.chunks.len()
        {
            return Ok(false);
        }
        let canonical = source.canonical_chunks()?;
        if canonical.len() != self.chunks.len() {
            return Ok(false);
        }
        Ok(canonical
            .into_iter()
            .zip(&self.chunks)
            .all(|(expected, actual)| {
                expected.source_ordinal() == actual.source_ordinal
                    && expected.chunk_ordinal() == actual.chunk_ordinal
                    && expected.chunk_count() == actual.chunk_count
                    && expected.byte_count() == actual.byte_count
                    && expected.bytes() == actual.bytes
            }))
    }
}

impl<'a> TraceSemanticMachineV2<'a> {
    fn new(
        context: RecursiveAuthorityContextV2,
        checkpoint: RecursiveCheckpointBindingV2,
        snapshot: super::recursive_context::RecursiveSnapshotHandleV2,
        store: &'a SettlementStore,
        max_content_bytes: u64,
        max_leaf_bytes: u32,
        max_spent: u32,
        max_outputs: u32,
    ) -> Result<Self, RecursiveV2Error> {
        let max_source_record_bytes = u64::try_from(TRACE_EVENT_HEADER_BYTES_V2)
            .map_err(|_| RecursiveV2Error::Limit)?
            .checked_add(u64::from(max_leaf_bytes))
            .ok_or(RecursiveV2Error::Overflow)?;
        let max_source_chunk_count = max_source_record_bytes
            .checked_add(
                u64::try_from(TRACE_CANONICAL_CHUNK_BYTES_V2 - 1)
                    .map_err(|_| RecursiveV2Error::Limit)?,
            )
            .ok_or(RecursiveV2Error::Overflow)?
            / u64::try_from(TRACE_CANONICAL_CHUNK_BYTES_V2).map_err(|_| RecursiveV2Error::Limit)?;
        let max_source_chunk_count =
            u32::try_from(max_source_chunk_count).map_err(|_| RecursiveV2Error::Limit)?;
        if context.policy_digest() != store.bucket_policy().bucket_policy_id() {
            return Err(RecursiveV2Error::CutoverAuthority);
        }
        // The evaluator is an independent acceptance machine, not merely a
        // verifier reached through `CanonicalCheckpointTransitionV2`.  Keep
        // the authority-selected no-op version gate here as well so a
        // crate-internal caller cannot turn the typed empty marker into a
        // second admission path by bypassing the orchestrator.
        if checkpoint.is_recursive_v2_noop()
            && !context.allows_noop_execution_input_version(checkpoint.exec_version())
        {
            return Err(RecursiveV2Error::Authority);
        }
        let mut replay_transcript = CheckpointSha256V2::new(CheckpointShaRole::Trace);
        let mut commit_transcript = CheckpointSha256V2::new(CheckpointShaRole::Trace);
        replay_transcript.update_part(b"z00z.recursive.v2.replay-transcript")?;
        commit_transcript.update_part(b"z00z.recursive.v2.replay-transcript")?;
        Ok(Self {
            context,
            snapshot,
            store,
            max_content_bytes,
            max_source_chunk_count,
            max_spent,
            max_outputs,
            authority_noop: checkpoint.is_recursive_v2_noop(),
            phase: TracePhaseV2::BeforeBegin,
            header: None,
            inputs: 0,
            outputs: 0,
            spent_ids: Vec::new(),
            output_ids: Vec::new(),
            uniqueness_precommit: None,
            uniqueness_challenge: None,
            commits: 0,
            jmt_chunks: 0,
            jmt_payload: Vec::new(),
            envelope_verified: false,
            pending_chunks: None,
            pending_hash: None,
            replay_transcript,
            commit_transcript,
        })
    }

    fn accept(&mut self, event: &RecursiveTraceEventV2) -> Result<(), RecursiveV2Error> {
        match event.opcode() {
            RecursiveTraceOpcodeV2::BeginHash
            | RecursiveTraceOpcodeV2::ShaBlock
            | RecursiveTraceOpcodeV2::EndHash => self.accept_hash_control(event),
            RecursiveTraceOpcodeV2::TraceChunk => {
                if self.pending_hash.is_some() {
                    return Err(RecursiveV2Error::EventOrder);
                }
                let chunk = decode_trace_chunk_control(event)?;
                match self.pending_chunks.as_mut() {
                    Some(pending) => pending.push(chunk),
                    None => {
                        self.pending_chunks = Some(PendingCanonicalSourceChunksV2::begin(
                            chunk,
                            self.max_source_chunk_count,
                        )?);
                        Ok(())
                    }
                }?;
                Ok(())
            }
            _ => {
                if self.pending_hash.is_some() || !self.consume_pending_chunks(event)? {
                    return Err(RecursiveV2Error::EventOrder);
                }
                self.accept_source(event)?;
                self.expect_hash(event)
            }
        }
    }

    fn accept_source(&mut self, event: &RecursiveTraceEventV2) -> Result<(), RecursiveV2Error> {
        match event.opcode() {
            RecursiveTraceOpcodeV2::BeginBlock if self.phase == TracePhaseV2::BeforeBegin => {
                self.require_structural_id(event)?;
                if event.ordinal() != 0 {
                    return Err(RecursiveV2Error::EventOrder);
                }
                let header = decode_flow_header(event.payload())?;
                if header.prev_root != *self.snapshot.root().as_bytes()
                    || header.post_root
                        != *self
                            .store
                            .settlement_root_v2(self.context.layout())
                            .map_err(|_| RecursiveV2Error::Root)?
                            .as_bytes()
                    || (self.authority_noop && header.prev_root != header.post_root)
                {
                    return Err(RecursiveV2Error::Root);
                }
                self.header = Some(header);
                self.phase = TracePhaseV2::ReplayInputs;
            }
            RecursiveTraceOpcodeV2::ReplayInput if self.phase == TracePhaseV2::ReplayInputs => {
                let item = self.decode_replay_item(event, ScopeOpKind::Delete)?;
                self.check_input(&item)?;
                self.push_spent_id(event.object_id())?;
                self.replay_transcript.update_part(event.payload())?;
                self.inputs = self
                    .inputs
                    .checked_add(1)
                    .ok_or(RecursiveV2Error::Overflow)?;
            }
            RecursiveTraceOpcodeV2::ReplayOutput
                if matches!(
                    self.phase,
                    TracePhaseV2::ReplayInputs | TracePhaseV2::ReplayOutputs
                ) =>
            {
                let item = self.decode_replay_item(event, ScopeOpKind::Put)?;
                self.check_output(&item)?;
                self.push_output_id(event.object_id())?;
                self.replay_transcript.update_part(event.payload())?;
                self.outputs = self
                    .outputs
                    .checked_add(1)
                    .ok_or(RecursiveV2Error::Overflow)?;
                self.phase = TracePhaseV2::ReplayOutputs;
            }
            RecursiveTraceOpcodeV2::UniquenessPrecommit
                if self.phase == TracePhaseV2::ReplayOutputs
                    || (self.authority_noop
                        && self.phase == TracePhaseV2::ReplayInputs
                        && self.inputs == 0
                        && self.outputs == 0) =>
            {
                self.require_structural_id(event)?;
                let precommit = decode_uniqueness_precommit(event.payload())?;
                if precommit != uniqueness_precommit_from_ids(&self.spent_ids, &self.output_ids)? {
                    return Err(RecursiveV2Error::Invariant);
                }
                self.uniqueness_precommit = Some(precommit);
                self.phase = TracePhaseV2::UniquenessPrecommit;
            }
            RecursiveTraceOpcodeV2::UniquenessChallenge
                if self.phase == TracePhaseV2::UniquenessPrecommit =>
            {
                self.require_structural_id(event)?;
                let precommit = self
                    .uniqueness_precommit
                    .ok_or(RecursiveV2Error::Invariant)?;
                self.uniqueness_challenge = Some(decode_uniqueness_challenge(
                    event.payload(),
                    self.context.digest(),
                    precommit,
                )?);
                self.phase = TracePhaseV2::UniquenessChallenge;
            }
            RecursiveTraceOpcodeV2::NetMerge if self.phase == TracePhaseV2::UniquenessChallenge => {
                self.require_structural_id(event)?;
                decode_net_merge(
                    event.payload(),
                    self.uniqueness_precommit
                        .ok_or(RecursiveV2Error::Invariant)?,
                    self.uniqueness_challenge
                        .ok_or(RecursiveV2Error::Invariant)?,
                )?;
                self.phase = TracePhaseV2::NetMerge;
            }
            RecursiveTraceOpcodeV2::JmtUpdate
                if matches!(self.phase, TracePhaseV2::NetMerge | TracePhaseV2::JmtUpdate) =>
            {
                self.require_structural_id(event)?;
                if (!self.authority_noop && (self.inputs == 0 || self.outputs == 0))
                    || (self.authority_noop && (self.inputs != 0 || self.outputs != 0))
                {
                    return Err(RecursiveV2Error::Invariant);
                }
                let next_len = self
                    .jmt_payload
                    .len()
                    .checked_add(event.payload().len())
                    .ok_or(RecursiveV2Error::Overflow)?;
                // The source has already bounded the total spool. Preserve that
                // bound here rather than permitting a second unbounded buffer.
                if u64::try_from(next_len).map_err(|_| RecursiveV2Error::Limit)?
                    > self.max_content_bytes
                {
                    return Err(RecursiveV2Error::Limit);
                }
                self.jmt_payload.extend_from_slice(event.payload());
                self.jmt_chunks = self
                    .jmt_chunks
                    .checked_add(1)
                    .ok_or(RecursiveV2Error::Overflow)?;
                self.phase = TracePhaseV2::JmtUpdate;
            }
            RecursiveTraceOpcodeV2::PromoteChildRoot if self.phase == TracePhaseV2::JmtUpdate => {
                self.require_structural_id(event)?;
                self.verify_jmt_envelope()?;
                let envelope = SettlementUpdateTraceEnvelopeV2::from_canon(&self.jmt_payload)
                    .map_err(|_| RecursiveV2Error::Canonical)?;
                decode_hierarchy_promotion(
                    event.payload(),
                    self.store.recursive_v2_definition_root(),
                    envelope.trace_digest(),
                )?;
                self.phase = TracePhaseV2::HierarchyPromotion;
            }
            RecursiveTraceOpcodeV2::CommitTypedEvent
                if matches!(
                    self.phase,
                    TracePhaseV2::HierarchyPromotion | TracePhaseV2::Commit
                ) && !self.authority_noop =>
            {
                self.require_structural_id(event)?;
                let item = decode_flow_item(event.payload())?;
                self.check_commit_item(&item)?;
                self.commit_transcript.update_part(event.payload())?;
                self.commits = self
                    .commits
                    .checked_add(1)
                    .ok_or(RecursiveV2Error::Overflow)?;
                self.phase = TracePhaseV2::Commit;
            }
            RecursiveTraceOpcodeV2::FinalizeBlock
                if (!self.authority_noop && self.phase == TracePhaseV2::Commit)
                    || (self.authority_noop && self.phase == TracePhaseV2::HierarchyPromotion) =>
            {
                self.require_structural_id(event)?;
                let header = decode_flow_header(event.payload())?;
                if Some(header) != self.header
                    || header.prev_root != *self.snapshot.root().as_bytes()
                    || header.post_root
                        != *self
                            .store
                            .settlement_root_v2(self.context.layout())
                            .map_err(|_| RecursiveV2Error::Root)?
                            .as_bytes()
                    || self.uniqueness_precommit.is_none()
                    || self.uniqueness_challenge.is_none()
                    || (!self.authority_noop && (self.inputs == 0 || self.outputs == 0))
                    || (self.authority_noop
                        && (self.inputs != 0 || self.outputs != 0 || self.commits != 0))
                    || self.jmt_chunks == 0
                    || (!self.authority_noop
                        && self.commits
                            != self
                                .inputs
                                .checked_add(self.outputs)
                                .ok_or(RecursiveV2Error::Overflow)?)
                {
                    return Err(RecursiveV2Error::Invariant);
                }
                self.phase = TracePhaseV2::Finalized;
            }
            _ => return Err(RecursiveV2Error::EventOrder),
        }
        Ok(())
    }

    fn consume_pending_chunks(
        &mut self,
        source: &RecursiveTraceEventV2,
    ) -> Result<bool, RecursiveV2Error> {
        let pending = self
            .pending_chunks
            .take()
            .ok_or(RecursiveV2Error::EventOrder)?;
        pending.matches_source(source)
    }

    fn finish(self) -> Result<(), RecursiveV2Error> {
        if self.phase != TracePhaseV2::Finalized
            || !self.envelope_verified
            || self.pending_chunks.is_some()
            || self.pending_hash.is_some()
            || self.replay_transcript.finalize() != self.commit_transcript.finalize()
        {
            return Err(RecursiveV2Error::Invariant);
        }
        Ok(())
    }

    fn decode_replay_item(
        &self,
        event: &RecursiveTraceEventV2,
        expected_kind: ScopeOpKind,
    ) -> Result<CanonicalFlowItemV2, RecursiveV2Error> {
        let item = decode_flow_item(event.payload())?;
        if item.op_kind != expected_kind || item.terminal_id != event.object_id() {
            return Err(RecursiveV2Error::Canonical);
        }
        Ok(item)
    }

    fn check_input(&self, item: &CanonicalFlowItemV2) -> Result<(), RecursiveV2Error> {
        if item.leaf_kind != ScopeLeafKind::Terminal
            || item.first_definition
            || item.first_serial
            || item.first_object
        {
            return Err(RecursiveV2Error::Invariant);
        }
        Ok(())
    }

    fn check_output(&self, item: &CanonicalFlowItemV2) -> Result<(), RecursiveV2Error> {
        if item.leaf_kind != ScopeLeafKind::Terminal {
            return Err(RecursiveV2Error::Invariant);
        }
        let post_item = self
            .store
            .get_settlement_item(&item.path())
            .map_err(|_| RecursiveV2Error::Storage)?
            .ok_or(RecursiveV2Error::Invariant)?;
        if post_item.terminal_leaf().is_err() {
            return Err(RecursiveV2Error::Invariant);
        }
        Ok(())
    }

    fn check_commit_item(&self, item: &CanonicalFlowItemV2) -> Result<(), RecursiveV2Error> {
        if item.leaf_kind != ScopeLeafKind::Terminal
            || matches!(item.op_kind, ScopeOpKind::Delete)
                && (item.first_definition || item.first_serial || item.first_object)
        {
            return Err(RecursiveV2Error::Invariant);
        }
        Ok(())
    }

    fn push_spent_id(&mut self, id: [u8; 32]) -> Result<(), RecursiveV2Error> {
        if self.spent_ids.len()
            >= usize::try_from(self.max_spent).map_err(|_| RecursiveV2Error::Limit)?
        {
            return Err(RecursiveV2Error::Limit);
        }
        self.spent_ids
            .try_reserve(1)
            .map_err(|_| RecursiveV2Error::Limit)?;
        self.spent_ids.push(id);
        Ok(())
    }

    fn push_output_id(&mut self, id: [u8; 32]) -> Result<(), RecursiveV2Error> {
        if self.output_ids.len()
            >= usize::try_from(self.max_outputs).map_err(|_| RecursiveV2Error::Limit)?
        {
            return Err(RecursiveV2Error::Limit);
        }
        self.output_ids
            .try_reserve(1)
            .map_err(|_| RecursiveV2Error::Limit)?;
        self.output_ids.push(id);
        Ok(())
    }

    fn verify_jmt_envelope(&mut self) -> Result<(), RecursiveV2Error> {
        if self.envelope_verified || self.jmt_payload.is_empty() {
            return Err(RecursiveV2Error::Invariant);
        }
        let envelope = SettlementUpdateTraceEnvelopeV2::from_canon(&self.jmt_payload)
            .map_err(|_| RecursiveV2Error::Canonical)?;
        if envelope.is_noop() != self.authority_noop
            || (!self.authority_noop && envelope.updates().is_empty())
            || (self.authority_noop && !envelope.updates().is_empty())
        {
            return Err(RecursiveV2Error::Invariant);
        }
        envelope
            .verify_hierarchy_semantics(self.store.recursive_v2_definition_root())
            .map_err(|_| RecursiveV2Error::Canonical)?;
        self.envelope_verified = true;
        Ok(())
    }

    fn update_trace_digest(&self) -> Result<[u8; 32], RecursiveV2Error> {
        if !self.envelope_verified {
            return Err(RecursiveV2Error::Invariant);
        }
        let envelope = SettlementUpdateTraceEnvelopeV2::from_canon(&self.jmt_payload)
            .map_err(|_| RecursiveV2Error::Canonical)?;
        Ok(envelope.trace_digest())
    }

    fn update_trace_count(&self) -> Result<u32, RecursiveV2Error> {
        if !self.envelope_verified {
            return Err(RecursiveV2Error::Invariant);
        }
        let envelope = SettlementUpdateTraceEnvelopeV2::from_canon(&self.jmt_payload)
            .map_err(|_| RecursiveV2Error::Canonical)?;
        u32::try_from(envelope.updates().len()).map_err(|_| RecursiveV2Error::Limit)
    }

    fn require_structural_id(&self, event: &RecursiveTraceEventV2) -> Result<(), RecursiveV2Error> {
        if event.object_id()
            != structural_event_id(event.opcode(), event.ordinal(), event.payload())
        {
            return Err(RecursiveV2Error::Canonical);
        }
        Ok(())
    }

    fn expect_hash(&mut self, event: &RecursiveTraceEventV2) -> Result<(), RecursiveV2Error> {
        let (message_bytes, block_count) = event.hash_geometry()?;
        if block_count == 0 {
            return Err(RecursiveV2Error::Invariant);
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
        });
        Ok(())
    }

    fn accept_hash_control(
        &mut self,
        event: &RecursiveTraceEventV2,
    ) -> Result<(), RecursiveV2Error> {
        self.require_structural_id(event)?;
        let control = decode_hash_control(event)?;
        // The source owner regenerates the tagged whole-trace transcript from
        // its protected canonical spool and verifies it against the sealed
        // precommit before this evaluator receives the controls.  It is not a
        // second source-record hash schedule for this independent semantic
        // machine; Nova consumes the same decoded controls in its sole lane.
        if control.schema == HashControlSchemaV2::TracePrecommit {
            return Ok(());
        }
        let completed = {
            let pending = self
                .pending_hash
                .as_mut()
                .ok_or(RecursiveV2Error::EventOrder)?;
            let expected_opcode = match pending.next_stage {
                HashControlStageV2::Begin => RecursiveTraceOpcodeV2::BeginHash,
                HashControlStageV2::Block => RecursiveTraceOpcodeV2::ShaBlock,
                HashControlStageV2::End => RecursiveTraceOpcodeV2::EndHash,
            };
            if event.opcode() != expected_opcode || !same_hash_binding(&control, *pending) {
                return Err(RecursiveV2Error::Canonical);
            }
            match control.stage {
                HashControlStageV2::Begin => {
                    if event.ordinal() != hash_control_ordinal(pending.source_ordinal, 0)?
                        || control.block.is_some()
                    {
                        return Err(RecursiveV2Error::Canonical);
                    }
                    pending.next_stage = HashControlStageV2::Block;
                    false
                }
                HashControlStageV2::Block => {
                    let block = control.block.ok_or(RecursiveV2Error::Canonical)?;
                    let expected_offset = pending
                        .blocks_seen
                        .checked_mul(64)
                        .ok_or(RecursiveV2Error::Overflow)?;
                    let expected_final = pending
                        .blocks_seen
                        .checked_add(1)
                        .ok_or(RecursiveV2Error::Overflow)?
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
                                    .ok_or(RecursiveV2Error::Overflow)?,
                            )?
                        || block.chaining_before != pending.chaining_state.unwrap_or(SHA256_IV_V2)
                        || !block.verifies_transition()
                    {
                        return Err(RecursiveV2Error::Canonical);
                    }
                    pending.chaining_state = Some(block.chaining_after);
                    pending.blocks_seen = pending
                        .blocks_seen
                        .checked_add(1)
                        .ok_or(RecursiveV2Error::Overflow)?;
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
                                .ok_or(RecursiveV2Error::Overflow)?,
                        )?
                        || control.block.is_some()
                        || pending.blocks_seen != pending.block_count
                        || CheckpointSha256BlockV2::digest_from_chaining(
                            &pending.chaining_state.ok_or(RecursiveV2Error::Invariant)?,
                        ) != pending.source_hash
                    {
                        return Err(RecursiveV2Error::Canonical);
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

fn same_hash_binding(control: &HashControlBindingV2, pending: PendingHashV2) -> bool {
    control.schema == HashControlSchemaV2::SourceRecord
        && control.stage == pending.next_stage
        && control.source_ordinal == pending.source_ordinal
        && control.source_opcode == pending.source_opcode
        && control.source_object_id == pending.source_object_id
        && control.source_hash == pending.source_hash
        && control.message_bytes == pending.message_bytes
        && control.block_count == pending.block_count
}
