//! One typed, acyclic V2 statement for the recursive checkpoint relation.
//!
//! The statement is constructed only after independent trace evaluation.  It
//! commits every currently materialized V2 relation field in a frozen order;
//! callers cannot substitute a generic configuration digest or a total count
//! for the individual authority, snapshot, root, grammar, trace, and JMT
//! bindings.

use z00z_crypto::{sha256_256, sha256_256_role, CheckpointShaRole};

use crate::{
    checkpoint::CheckpointId,
    settlement::{
        derive_settlement_root_v2, RootGeneration, SettlementStateRoot, SettlementStore,
        SettlementUpdateTraceEnvelopeV2,
    },
    CheckpointError,
};

use super::{
    recursive_circuit::{RecursiveCircuitProfileV2, RecursiveCircuitSpecV2},
    recursive_context::{
        RecursiveAuthorityContextV2, RecursiveCheckpointBindingV2, RecursiveCheckpointContextV2,
        RecursiveSnapshotHandleV2,
    },
    recursive_reject::RecursiveCheckpointRejectReasonV2,
    recursive_trace::{RecursiveTraceOpcodeV2, RecursiveTracePrecommitV2},
};

/// Frozen codec version for the acyclic count commitment and `P` transcript.
pub(crate) const PRE_UNIQUENESS_CONTEXT_VERSION_V2: u8 = 3;

/// Exact work declaration committed before any uniqueness challenge exists.
///
/// The per-opcode vector is the sole canonical schedule accounting. The
/// semantic aliases below are retained as independently framed fields so a
/// generic total cannot substitute for rows, inputs, outputs, net effects,
/// JMT updates, SHA blocks, or the complete expanded event count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RecursiveDeclaredWorkV2 {
    event_counts: super::recursive_trace::RecursiveTraceEventCountsV2,
    row_count: u64,
    input_count: u64,
    output_count: u64,
    net_effect_count: u64,
    net_mutation_count: u64,
    jmt_update_count: u64,
    hash_block_count: u64,
    event_count: u64,
    digest: [u8; 32],
}

impl RecursiveDeclaredWorkV2 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        event_counts: super::recursive_trace::RecursiveTraceEventCountsV2,
        row_count: u64,
        input_count: u64,
        output_count: u64,
        net_effect_count: u64,
        net_mutation_count: u64,
        jmt_update_count: u64,
        hash_block_count: u64,
        event_count: u64,
    ) -> Result<Self, CheckpointError> {
        if row_count
            != input_count
                .checked_add(output_count)
                .ok_or(CheckpointError::Overflow)?
            || net_effect_count > row_count
            || net_mutation_count > net_effect_count
            || (row_count != 0 && net_effect_count == 0)
            || hash_block_count != event_counts.count(RecursiveTraceOpcodeV2::ShaBlock)
            || event_count != event_counts.total_count()?
        {
            return Err(CheckpointError::Invariant);
        }
        let digest = declared_work_digest(
            event_counts,
            row_count,
            input_count,
            output_count,
            net_effect_count,
            net_mutation_count,
            jmt_update_count,
            hash_block_count,
            event_count,
        );
        Ok(Self {
            event_counts,
            row_count,
            input_count,
            output_count,
            net_effect_count,
            net_mutation_count,
            jmt_update_count,
            hash_block_count,
            event_count,
            digest,
        })
    }

    #[must_use]
    pub(crate) const fn event_counts(self) -> super::recursive_trace::RecursiveTraceEventCountsV2 {
        self.event_counts
    }

    #[must_use]
    pub(crate) const fn semantic_counts(self) -> [u64; 8] {
        [
            self.row_count,
            self.input_count,
            self.output_count,
            self.net_effect_count,
            self.net_mutation_count,
            self.jmt_update_count,
            self.hash_block_count,
            self.event_count,
        ]
    }

    #[must_use]
    pub(crate) const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub(crate) fn transcript_parts(self) -> Vec<Vec<u8>> {
        let [row, input, output, net, mutation, jmt, sha, events] = self.semantic_counts();
        vec![
            vec![PRE_UNIQUENESS_CONTEXT_VERSION_V2],
            row.to_le_bytes().to_vec(),
            input.to_le_bytes().to_vec(),
            output.to_le_bytes().to_vec(),
            net.to_le_bytes().to_vec(),
            mutation.to_le_bytes().to_vec(),
            jmt.to_le_bytes().to_vec(),
            sha.to_le_bytes().to_vec(),
            events.to_le_bytes().to_vec(),
            self.event_counts.canonical_bytes().to_vec(),
        ]
    }
}

/// Bundle-selected executable predicate authority. Raw digests never cross
/// into the canonical transition constructor independently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RecursiveVerifierAuthorityV2 {
    predicate_digest: [u8; 32],
    verifier_bundle_digest: [u8; 32],
}

impl RecursiveVerifierAuthorityV2 {
    pub(crate) fn new(
        predicate_digest: [u8; 32],
        verifier_bundle_digest: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        if predicate_digest == [0; 32] || verifier_bundle_digest == [0; 32] {
            return Err(CheckpointError::Authority);
        }
        Ok(Self {
            predicate_digest,
            verifier_bundle_digest,
        })
    }

    /// Transition-only binding used before a validated verifier bundle is
    /// available. Public evidence production replaces it with bundle authority
    /// before any proof envelope or receipt can be emitted.
    pub(crate) fn transition_only_binding(
        authority: RecursiveAuthorityContextV2,
        profile: &RecursiveCircuitProfileV2,
        spec: &RecursiveCircuitSpecV2,
    ) -> Result<Self, CheckpointError> {
        let predicate_digest = super::canonical_transition::executable_predicate_digest()?;
        let verifier_bundle_digest = sha256_256_role(
            CheckpointShaRole::Statement,
            &[
                b"z00z.recursive.v2.transition-only-verifier-binding",
                &authority.digest(),
                &profile.digest(),
                &spec.digest(),
                &predicate_digest,
            ],
        );
        Self::new(predicate_digest, verifier_bundle_digest)
    }

    #[must_use]
    pub(crate) const fn predicate_digest(self) -> [u8; 32] {
        self.predicate_digest
    }

    #[must_use]
    pub(crate) const fn verifier_bundle_digest(self) -> [u8; 32] {
        self.verifier_bundle_digest
    }
}

/// Exact acyclic pre-uniqueness transcript `P` and all of its typed inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RecursivePreUniquenessContextV2 {
    chain_context: [u8; 32],
    config_digest: [u8; 32],
    policy_digest: [u8; 32],
    authority_digest: [u8; 32],
    height: u64,
    predecessor_height: u64,
    layout: u32,
    authority_generation: u64,
    noop_execution_input_version: u8,
    epoch_cadence_blocks: u64,
    old_settlement_root: [u8; 32],
    old_definition_root: [u8; 32],
    tx_data_root: [u8; 32],
    update_trace_digest: [u8; 32],
    predicate_digest: [u8; 32],
    profile_digest: [u8; 32],
    spec_digest: [u8; 32],
    trace_grammar_digest: [u8; 32],
    verifier_bundle_digest: [u8; 32],
    declared_work: RecursiveDeclaredWorkV2,
    digest: [u8; 32],
}

impl RecursivePreUniquenessContextV2 {
    #[cfg(test)]
    pub(crate) fn repository_trace_fixture(
        snapshot: RecursiveSnapshotHandleV2,
        profile: &RecursiveCircuitProfileV2,
    ) -> Result<Self, CheckpointError> {
        let work = RecursiveDeclaredWorkV2::new(
            super::recursive_trace::RecursiveTraceEventCountsV2::default(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        )?;
        Self::from_parts(
            [1; 32],
            [10; 32],
            [3; 32],
            [12; 32],
            1,
            0,
            7,
            1,
            1,
            1_000,
            *snapshot.root().as_bytes(),
            snapshot.pre_definition_root(),
            [2; 32],
            [3; 32],
            [9; 32],
            profile.digest(),
            [4; 32],
            RecursiveTraceOpcodeV2::grammar_digest(),
            [5; 32],
            work,
        )
    }

    pub(crate) fn build(
        authority: RecursiveAuthorityContextV2,
        snapshot: RecursiveSnapshotHandleV2,
        checkpoint: RecursiveCheckpointBindingV2,
        profile: &RecursiveCircuitProfileV2,
        verifier: RecursiveVerifierAuthorityV2,
        declared_work: RecursiveDeclaredWorkV2,
        update_trace_digest: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        let predecessor_height = checkpoint
            .height()
            .checked_sub(1)
            .ok_or(CheckpointError::Invariant)?;
        if checkpoint.predecessor().is_some() != (predecessor_height != 0)
            || checkpoint.pre_settlement_root() != snapshot.root()
        {
            return Err(CheckpointError::Authority);
        }
        let spec = RecursiveCircuitSpecV2::new(authority.layout(), profile)?;
        Self::from_parts(
            authority.network_context(),
            authority.config_digest(),
            authority.policy_digest(),
            authority.digest(),
            checkpoint.height(),
            predecessor_height,
            authority.layout(),
            authority.authority_generation(),
            authority.noop_execution_input_version(),
            authority.epoch_cadence_blocks(),
            *snapshot.root().as_bytes(),
            snapshot.pre_definition_root(),
            checkpoint.exec_tx_root(),
            update_trace_digest,
            verifier.predicate_digest(),
            profile.digest(),
            spec.digest(),
            RecursiveTraceOpcodeV2::grammar_digest(),
            verifier.verifier_bundle_digest(),
            declared_work,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_parts(
        chain_context: [u8; 32],
        config_digest: [u8; 32],
        policy_digest: [u8; 32],
        authority_digest: [u8; 32],
        height: u64,
        predecessor_height: u64,
        layout: u32,
        authority_generation: u64,
        noop_execution_input_version: u8,
        epoch_cadence_blocks: u64,
        old_settlement_root: [u8; 32],
        old_definition_root: [u8; 32],
        tx_data_root: [u8; 32],
        update_trace_digest: [u8; 32],
        predicate_digest: [u8; 32],
        profile_digest: [u8; 32],
        spec_digest: [u8; 32],
        trace_grammar_digest: [u8; 32],
        verifier_bundle_digest: [u8; 32],
        declared_work: RecursiveDeclaredWorkV2,
    ) -> Result<Self, CheckpointError> {
        if height == 0
            || predecessor_height.checked_add(1) != Some(height)
            || layout == 0
            || authority_generation == 0
            || noop_execution_input_version == 0
            || epoch_cadence_blocks == 0
            || [
                chain_context,
                config_digest,
                policy_digest,
                authority_digest,
                old_settlement_root,
                old_definition_root,
                tx_data_root,
                update_trace_digest,
                predicate_digest,
                profile_digest,
                spec_digest,
                trace_grammar_digest,
                verifier_bundle_digest,
                declared_work.digest(),
            ]
            .contains(&[0; 32])
        {
            return Err(CheckpointError::Invariant);
        }
        let version = [PRE_UNIQUENESS_CONTEXT_VERSION_V2];
        let digest = sha256_256_role(
            CheckpointShaRole::UniquenessContext,
            &[
                &version,
                &chain_context,
                &config_digest,
                &policy_digest,
                &authority_digest,
                &height.to_le_bytes(),
                &predecessor_height.to_le_bytes(),
                &layout.to_le_bytes(),
                &authority_generation.to_le_bytes(),
                &[noop_execution_input_version],
                &epoch_cadence_blocks.to_le_bytes(),
                &old_settlement_root,
                &old_definition_root,
                &tx_data_root,
                &update_trace_digest,
                &predicate_digest,
                &profile_digest,
                &spec_digest,
                &trace_grammar_digest,
                &verifier_bundle_digest,
                &declared_work.digest(),
            ],
        );
        Ok(Self {
            chain_context,
            config_digest,
            policy_digest,
            authority_digest,
            height,
            predecessor_height,
            layout,
            authority_generation,
            noop_execution_input_version,
            epoch_cadence_blocks,
            old_settlement_root,
            old_definition_root,
            tx_data_root,
            update_trace_digest,
            predicate_digest,
            profile_digest,
            spec_digest,
            trace_grammar_digest,
            verifier_bundle_digest,
            declared_work,
            digest,
        })
    }

    #[must_use]
    pub(crate) const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub(crate) fn validate_binding(
        self,
        authority: RecursiveAuthorityContextV2,
        snapshot: RecursiveSnapshotHandleV2,
        checkpoint: RecursiveCheckpointBindingV2,
        profile: &RecursiveCircuitProfileV2,
    ) -> Result<(), CheckpointError> {
        let spec = RecursiveCircuitSpecV2::new(authority.layout(), profile)?;
        if self.chain_context != authority.network_context()
            || self.config_digest != authority.config_digest()
            || self.policy_digest != authority.policy_digest()
            || self.authority_digest != authority.digest()
            || self.height != checkpoint.height()
            || self.predecessor_height.checked_add(1) != Some(checkpoint.height())
            || self.layout != authority.layout()
            || self.authority_generation != authority.authority_generation()
            || self.noop_execution_input_version != authority.noop_execution_input_version()
            || self.epoch_cadence_blocks != authority.epoch_cadence_blocks()
            || self.old_settlement_root != *snapshot.root().as_bytes()
            || self.old_definition_root != snapshot.pre_definition_root()
            || self.tx_data_root != checkpoint.exec_tx_root()
            || self.profile_digest != profile.digest()
            || self.spec_digest != spec.digest()
            || self.trace_grammar_digest != RecursiveTraceOpcodeV2::grammar_digest()
        {
            return Err(CheckpointError::Authority);
        }
        Ok(())
    }

    #[must_use]
    pub(crate) const fn declared_work(self) -> RecursiveDeclaredWorkV2 {
        self.declared_work
    }

    #[must_use]
    pub(crate) const fn update_trace_digest(self) -> [u8; 32] {
        self.update_trace_digest
    }

    #[must_use]
    pub(crate) const fn old_settlement_root(self) -> [u8; 32] {
        self.old_settlement_root
    }

    #[must_use]
    pub(crate) const fn old_definition_root(self) -> [u8; 32] {
        self.old_definition_root
    }

    pub(crate) fn settlement_root_transcript_parts(
        self,
        definition_root: [u8; 32],
    ) -> Vec<Vec<u8>> {
        vec![
            vec![RootGeneration::SettlementV2.version()],
            self.layout.to_le_bytes().to_vec(),
            self.policy_digest.to_vec(),
            definition_root.to_vec(),
        ]
    }

    #[must_use]
    pub(crate) const fn scalar_parts(self) -> [u64; 7] {
        [
            PRE_UNIQUENESS_CONTEXT_VERSION_V2 as u64,
            self.height,
            self.predecessor_height,
            self.layout as u64,
            self.authority_generation,
            self.noop_execution_input_version as u64,
            self.epoch_cadence_blocks,
        ]
    }

    #[must_use]
    pub(crate) const fn digest_parts(self) -> [[u8; 32]; 15] {
        [
            self.chain_context,
            self.config_digest,
            self.policy_digest,
            self.authority_digest,
            self.old_settlement_root,
            self.old_definition_root,
            self.tx_data_root,
            self.update_trace_digest,
            self.predicate_digest,
            self.profile_digest,
            self.spec_digest,
            self.trace_grammar_digest,
            self.verifier_bundle_digest,
            self.declared_work.digest(),
            self.digest,
        ]
    }

    pub(crate) fn transcript_parts(self) -> Vec<Vec<u8>> {
        vec![
            vec![PRE_UNIQUENESS_CONTEXT_VERSION_V2],
            self.chain_context.to_vec(),
            self.config_digest.to_vec(),
            self.policy_digest.to_vec(),
            self.authority_digest.to_vec(),
            self.height.to_le_bytes().to_vec(),
            self.predecessor_height.to_le_bytes().to_vec(),
            self.layout.to_le_bytes().to_vec(),
            self.authority_generation.to_le_bytes().to_vec(),
            vec![self.noop_execution_input_version],
            self.epoch_cadence_blocks.to_le_bytes().to_vec(),
            self.old_settlement_root.to_vec(),
            self.old_definition_root.to_vec(),
            self.tx_data_root.to_vec(),
            self.update_trace_digest.to_vec(),
            self.predicate_digest.to_vec(),
            self.profile_digest.to_vec(),
            self.spec_digest.to_vec(),
            self.trace_grammar_digest.to_vec(),
            self.verifier_bundle_digest.to_vec(),
            self.declared_work.digest().to_vec(),
        ]
    }
}

fn declared_work_digest(
    event_counts: super::recursive_trace::RecursiveTraceEventCountsV2,
    row_count: u64,
    input_count: u64,
    output_count: u64,
    net_effect_count: u64,
    net_mutation_count: u64,
    jmt_update_count: u64,
    hash_block_count: u64,
    event_count: u64,
) -> [u8; 32] {
    let version = [PRE_UNIQUENESS_CONTEXT_VERSION_V2];
    sha256_256_role(
        CheckpointShaRole::UniquenessCounts,
        &[
            &version,
            &row_count.to_le_bytes(),
            &input_count.to_le_bytes(),
            &output_count.to_le_bytes(),
            &net_effect_count.to_le_bytes(),
            &net_mutation_count.to_le_bytes(),
            &jmt_update_count.to_le_bytes(),
            &hash_block_count.to_le_bytes(),
            &event_count.to_le_bytes(),
            &event_counts.canonical_bytes(),
        ],
    )
}

/// Revision 6 adds the storage-derived typed core fields and both set-specific
/// prechallenge commitments.  A statement-core digest alone is no longer the
/// only route by which delta/witness/journal/prior-IVC data reaches T2.
/// Recursive V2 has
/// no compatibility decoder: this marker makes old, under-bound statement
/// bytes unambiguously distinct from the live canonical statement.
const RECURSIVE_TRANSITION_STATEMENT_VERSION_V2: u8 = 6;

/// Typed public commitment to one evaluated V2 transition.
///
/// It exposes no replay values, JMT operations, or spool bytes.  The exact
/// canonical bytes remain crate-private until the authority-pinned proof
/// boundary consumes them in T2/T3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursiveTransitionStatementV2 {
    digest: [u8; 32],
    checkpoint_id: CheckpointId,
    predecessor: Option<CheckpointId>,
    height: u64,
    checkpoint_exec_tx_root: [u8; 32],
    checkpoint_exec_tx_count: u32,
    checkpoint_statement_digest: [u8; 32],
    checkpoint_statement_core_digest: [u8; 32],
    delta_root: [u8; 32],
    witness_root: [u8; 32],
    journal_digest: [u8; 32],
    prior_recursive_output_root: Option<[u8; 32]>,
    checkpoint_link_digest: [u8; 32],
    pre_settlement_root: SettlementStateRoot,
    post_settlement_root: SettlementStateRoot,
    pre_definition_root: [u8; 32],
    post_definition_root: [u8; 32],
    trace_digest: [u8; 32],
    update_trace_digest: [u8; 32],
    declared_work_digest: [u8; 32],
    pre_uniqueness_context_digest: [u8; 32],
    spent_uniqueness_precommit: [u8; 32],
    output_uniqueness_precommit: [u8; 32],
    declared_event_count: u64,
    declared_byte_count: u64,
    declared_event_counts: super::recursive_trace::RecursiveTraceEventCountsV2,
    consumed_event_counts: super::recursive_trace::RecursiveTraceEventCountsV2,
}

impl RecursiveTransitionStatementV2 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        authority: RecursiveAuthorityContextV2,
        snapshot: RecursiveSnapshotHandleV2,
        checkpoint: RecursiveCheckpointBindingV2,
        profile: &RecursiveCircuitProfileV2,
        post_settlement_root: SettlementStateRoot,
        pre_definition_root: [u8; 32],
        post_definition_root: [u8; 32],
        update_trace_digest: [u8; 32],
        update_trace_count: u32,
        trace: RecursiveTracePrecommitV2,
        pre_uniqueness_context: RecursivePreUniquenessContextV2,
        declared_event_counts: super::recursive_trace::RecursiveTraceEventCountsV2,
        consumed_event_counts: super::recursive_trace::RecursiveTraceEventCountsV2,
    ) -> Result<Self, CheckpointError> {
        let authority_noop = checkpoint.is_recursive_v2_noop();
        pre_uniqueness_context.validate_binding(authority, snapshot, checkpoint, profile)?;
        let derived_pre_settlement_root = derive_settlement_root_v2(
            RootGeneration::SettlementV2,
            authority.layout(),
            authority.policy_digest(),
            pre_definition_root,
        )
        .map_err(|_| CheckpointError::Root)?;
        let derived_post_settlement_root = derive_settlement_root_v2(
            RootGeneration::SettlementV2,
            authority.layout(),
            authority.policy_digest(),
            post_definition_root,
        )
        .map_err(|_| CheckpointError::Root)?;
        if snapshot.root() != checkpoint.pre_settlement_root()
            || snapshot.pre_definition_root() != pre_definition_root
            || post_settlement_root != checkpoint.post_settlement_root()
            || derived_pre_settlement_root != snapshot.root()
            || derived_post_settlement_root != post_settlement_root
            || snapshot.root().generation() != post_settlement_root.generation()
            || snapshot.root().generation().version() != 2
        {
            return Err(CheckpointError::Root);
        }
        if snapshot.snapshot_id() != checkpoint.prep_snapshot_id() || checkpoint.height() == 0 {
            return Err(CheckpointError::Authority);
        }
        if update_trace_digest == [0; 32]
            || pre_uniqueness_context.update_trace_digest() != update_trace_digest
        {
            return Err(CheckpointError::Canonical);
        }
        if !authority_noop && update_trace_count == 0 {
            return Err(CheckpointError::TraceState);
        }
        if authority_noop
            && (update_trace_count != 0
                || post_settlement_root != checkpoint.pre_settlement_root()
                || !SettlementUpdateTraceEnvelopeV2::is_noop_digest(update_trace_digest))
        {
            return Err(CheckpointError::Invariant);
        }
        if trace.event_count() == 0 || trace.byte_count() == 0 {
            return Err(CheckpointError::TraceState);
        }
        if declared_event_counts != consumed_event_counts
            || pre_uniqueness_context.declared_work().event_counts() != declared_event_counts
            || declared_event_counts.source_record_count()? != trace.event_count()
        {
            return Err(CheckpointError::EventOrder);
        }
        let spec = RecursiveCircuitSpecV2::new(authority.layout(), profile)?;
        let grammar_digest = RecursiveTraceOpcodeV2::grammar_digest();
        let semantic_counts = pre_uniqueness_context.declared_work().semantic_counts();
        let spent_count = u32::try_from(semantic_counts[1]).map_err(|_| CheckpointError::Limit)?;
        let output_count = u32::try_from(semantic_counts[2]).map_err(|_| CheckpointError::Limit)?;
        let (spent_uniqueness_precommit, output_uniqueness_precommit) =
            super::recursive_semantics::uniqueness_set_precommits_from_parts(
                pre_uniqueness_context.digest(),
                spent_count,
                output_count,
                trace.spent_original_ids_digest(),
                trace.spent_sorted_ids_digest(),
                trace.output_original_ids_digest(),
                trace.output_sorted_ids_digest(),
            );
        let canonical = canonical_statement_bytes(
            authority,
            snapshot,
            checkpoint,
            profile.digest(),
            spec.digest(),
            grammar_digest,
            post_settlement_root,
            pre_definition_root,
            post_definition_root,
            update_trace_digest,
            update_trace_count,
            trace,
            pre_uniqueness_context,
            spent_uniqueness_precommit,
            output_uniqueness_precommit,
            declared_event_counts,
            consumed_event_counts,
        );
        let digest = sha256_256_role(CheckpointShaRole::Statement, &[&canonical]);
        Ok(Self {
            digest,
            checkpoint_id: checkpoint.checkpoint_id(),
            predecessor: checkpoint.predecessor(),
            height: checkpoint.height(),
            checkpoint_exec_tx_root: checkpoint.exec_tx_root(),
            checkpoint_exec_tx_count: checkpoint.exec_tx_count(),
            checkpoint_statement_digest: checkpoint.statement_digest(),
            checkpoint_statement_core_digest: checkpoint.statement_core_digest(),
            delta_root: checkpoint.delta_root(),
            witness_root: checkpoint.witness_root(),
            journal_digest: checkpoint.journal_digest(),
            prior_recursive_output_root: checkpoint.prior_recursive_output_root(),
            checkpoint_link_digest: checkpoint.checkpoint_link_digest(),
            pre_settlement_root: snapshot.root(),
            post_settlement_root,
            pre_definition_root,
            post_definition_root,
            trace_digest: trace.trace_digest(),
            update_trace_digest,
            declared_work_digest: pre_uniqueness_context.declared_work().digest(),
            pre_uniqueness_context_digest: pre_uniqueness_context.digest(),
            spent_uniqueness_precommit,
            output_uniqueness_precommit,
            declared_event_count: trace.event_count(),
            declared_byte_count: trace.byte_count(),
            declared_event_counts,
            consumed_event_counts,
        })
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub const fn checkpoint_id(&self) -> CheckpointId {
        self.checkpoint_id
    }

    #[must_use]
    pub const fn predecessor(&self) -> Option<CheckpointId> {
        self.predecessor
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub const fn checkpoint_exec_tx_root(&self) -> [u8; 32] {
        self.checkpoint_exec_tx_root
    }

    #[must_use]
    pub const fn checkpoint_exec_tx_count(&self) -> u32 {
        self.checkpoint_exec_tx_count
    }

    #[must_use]
    pub const fn checkpoint_statement_digest(&self) -> [u8; 32] {
        self.checkpoint_statement_digest
    }

    #[must_use]
    pub const fn checkpoint_statement_core_digest(&self) -> [u8; 32] {
        self.checkpoint_statement_core_digest
    }

    #[must_use]
    pub const fn delta_root(&self) -> [u8; 32] {
        self.delta_root
    }

    #[must_use]
    pub const fn witness_root(&self) -> [u8; 32] {
        self.witness_root
    }

    #[must_use]
    pub const fn journal_digest(&self) -> [u8; 32] {
        self.journal_digest
    }

    #[must_use]
    pub const fn prior_recursive_output_root(&self) -> Option<[u8; 32]> {
        self.prior_recursive_output_root
    }

    #[must_use]
    pub const fn checkpoint_link_digest(&self) -> [u8; 32] {
        self.checkpoint_link_digest
    }

    #[must_use]
    pub const fn pre_settlement_root(&self) -> SettlementStateRoot {
        self.pre_settlement_root
    }

    #[must_use]
    pub const fn post_settlement_root(&self) -> SettlementStateRoot {
        self.post_settlement_root
    }

    #[must_use]
    pub const fn pre_definition_root(&self) -> [u8; 32] {
        self.pre_definition_root
    }

    #[must_use]
    pub const fn post_definition_root(&self) -> [u8; 32] {
        self.post_definition_root
    }

    #[must_use]
    pub const fn trace_digest(&self) -> [u8; 32] {
        self.trace_digest
    }

    #[must_use]
    pub const fn update_trace_digest(&self) -> [u8; 32] {
        self.update_trace_digest
    }

    #[must_use]
    pub const fn declared_work_digest(&self) -> [u8; 32] {
        self.declared_work_digest
    }

    #[must_use]
    pub const fn pre_uniqueness_context_digest(&self) -> [u8; 32] {
        self.pre_uniqueness_context_digest
    }

    #[must_use]
    pub const fn spent_uniqueness_precommit(&self) -> [u8; 32] {
        self.spent_uniqueness_precommit
    }

    #[must_use]
    pub const fn output_uniqueness_precommit(&self) -> [u8; 32] {
        self.output_uniqueness_precommit
    }

    #[must_use]
    pub const fn declared_event_count(&self) -> u64 {
        self.declared_event_count
    }

    #[must_use]
    pub const fn declared_byte_count(&self) -> u64 {
        self.declared_byte_count
    }

    /// Return the fixed schedule declared by the source expansion.
    #[must_use]
    pub const fn declared_event_counts(
        &self,
    ) -> super::recursive_trace::RecursiveTraceEventCountsV2 {
        self.declared_event_counts
    }

    /// Return the independently accepted event counts from the evaluator.
    #[must_use]
    pub const fn consumed_event_counts(
        &self,
    ) -> super::recursive_trace::RecursiveTraceEventCountsV2 {
        self.consumed_event_counts
    }
}

const NOVA_STEP_INPUT_VERSION_V2: u8 = 2;
pub(crate) const FINAL_IVC_STATE_VERSION_V2: u8 = 1;
const NOVA_STEP_INPUT_MAGIC_V2: [u8; 8] = *b"Z00ZRCI2";
pub(crate) const FINAL_IVC_STATE_MAGIC_V2: [u8; 8] = *b"Z00ZRFS2";
pub const NOVA_BACKEND_LABEL_V2: [u8; 28] = *b"nova_streaming_compressed_v2";
pub const NOVA_PROOF_MODE_V2: [u8; 27] = *b"fast_classical_streaming_v2";
const CHECKPOINT_PUBLIC_INPUT_VERSION_V2: u16 = 2;
const CHECKPOINT_PUBLIC_INPUT_DOMAIN_V2: &str = "z00z.storage.checkpoint.recursive_public_input.v2";
const CHECKPOINT_PUBLIC_INPUT_LABEL_V2: &str = "public_input_digest";
pub(crate) const CHECKPOINT_PUBLIC_INPUT_BYTES_V2: usize =
    2 + 32 * 10 + 8 * 4 + 4 * 2 + NOVA_BACKEND_LABEL_V2.len() + NOVA_PROOF_MODE_V2.len();

/// Authority-selected Nova identities that enter `X_h` only after the
/// verifier bundle has authenticated its PP/VK generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RecursiveVerifierInputBindingV2 {
    predicate_digest: [u8; 32],
    profile_digest: [u8; 32],
    spec_digest: [u8; 32],
    pp_digest: [u8; 32],
    compressed_vk_digest: [u8; 32],
    verifier_bundle_digest: [u8; 32],
}

impl RecursiveVerifierInputBindingV2 {
    pub(crate) fn new(
        predicate_digest: [u8; 32],
        profile_digest: [u8; 32],
        spec_digest: [u8; 32],
        pp_digest: [u8; 32],
        compressed_vk_digest: [u8; 32],
        verifier_bundle_digest: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        if [
            predicate_digest,
            profile_digest,
            spec_digest,
            pp_digest,
            compressed_vk_digest,
            verifier_bundle_digest,
        ]
        .contains(&[0_u8; 32])
        {
            return Err(CheckpointError::Authority);
        }
        Ok(Self {
            predicate_digest,
            profile_digest,
            spec_digest,
            pp_digest,
            compressed_vk_digest,
            verifier_bundle_digest,
        })
    }
}

/// Typed finalized endpoint carried between consecutive block relations.
///
/// This is not a proof digest.  It is constructed after `X_h`, commits the
/// exact transition endpoint and cumulative Nova step count, and is the only
/// value that a successor may name as its prior recursive output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct RecursiveFinalizedIvcStateV2 {
    height: u64,
    checkpoint_id: CheckpointId,
    public_input_digest: [u8; 32],
    transition_statement_digest: [u8; 32],
    checkpoint_link_digest: [u8; 32],
    settlement_root: [u8; 32],
    definition_root: [u8; 32],
    cumulative_steps: u64,
    digest: [u8; 32],
}

impl RecursiveFinalizedIvcStateV2 {
    /// Build the unique finalized successor after `X_h` exists.
    pub(super) fn expected_successor(
        input: &RecursiveNovaStepInputV2,
        cumulative_steps: u64,
    ) -> Result<Self, CheckpointError> {
        if cumulative_steps <= input.prior.cumulative_steps {
            return Err(CheckpointError::Invariant);
        }
        let mut state = Self {
            height: input.height,
            checkpoint_id: input.checkpoint_id,
            public_input_digest: input.digest,
            transition_statement_digest: input.transition_statement_digest,
            checkpoint_link_digest: input.checkpoint_link_digest,
            settlement_root: input.post_settlement_root,
            definition_root: input.post_definition_root,
            cumulative_steps,
            digest: [0_u8; 32],
        };
        state.digest = sha256_256_role(CheckpointShaRole::Statement, &[&state.canonical_prefix()]);
        Ok(state)
    }

    /// Derive the authority-pinned cutover endpoint from the sole durable
    /// manifest validated against the unchanged settlement store.
    pub(crate) fn from_cutover_store(store: &SettlementStore) -> Result<Self, CheckpointError> {
        let manifest = store
            .load_recursive_v2_cutover()
            .map_err(|_| CheckpointError::CutoverAuthority)?;
        if [
            manifest.record_digest,
            manifest.expected_settlement_root,
            manifest.expected_definition_root,
        ]
        .contains(&[0_u8; 32])
        {
            return Err(CheckpointError::CutoverAuthority);
        }
        let height = manifest
            .height
            .checked_sub(1)
            .ok_or(CheckpointError::CutoverAuthority)?;
        let mut state = Self {
            height,
            checkpoint_id: CheckpointId::new(manifest.record_digest),
            public_input_digest: manifest.record_digest,
            transition_statement_digest: manifest.record_digest,
            checkpoint_link_digest: manifest.record_digest,
            settlement_root: manifest.expected_settlement_root,
            definition_root: manifest.expected_definition_root,
            cumulative_steps: 0,
            digest: [0_u8; 32],
        };
        state.digest = sha256_256_role(CheckpointShaRole::Statement, &[&state.canonical_prefix()]);
        Ok(state)
    }

    #[cfg(test)]
    pub(super) fn cutover_fixture(
        checkpoint_id: CheckpointId,
        settlement_root: [u8; 32],
        definition_root: [u8; 32],
        manifest_digest: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        if [settlement_root, definition_root, manifest_digest].contains(&[0_u8; 32]) {
            return Err(CheckpointError::Authority);
        }
        let mut state = Self {
            height: 0,
            checkpoint_id,
            public_input_digest: manifest_digest,
            transition_statement_digest: manifest_digest,
            checkpoint_link_digest: manifest_digest,
            settlement_root,
            definition_root,
            cumulative_steps: 0,
            digest: [0_u8; 32],
        };
        state.digest = sha256_256_role(CheckpointShaRole::Statement, &[&state.canonical_prefix()]);
        Ok(state)
    }

    fn canonical_prefix(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + 1 + 8 * 2 + 32 * 7);
        bytes.extend_from_slice(&FINAL_IVC_STATE_MAGIC_V2);
        bytes.push(FINAL_IVC_STATE_VERSION_V2);
        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(self.checkpoint_id.as_bytes());
        bytes.extend_from_slice(&self.public_input_digest);
        bytes.extend_from_slice(&self.transition_statement_digest);
        bytes.extend_from_slice(&self.checkpoint_link_digest);
        bytes.extend_from_slice(&self.settlement_root);
        bytes.extend_from_slice(&self.definition_root);
        bytes.extend_from_slice(&self.cumulative_steps.to_le_bytes());
        bytes
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub const fn cumulative_steps(&self) -> u64 {
        self.cumulative_steps
    }

    #[must_use]
    pub(crate) const fn checkpoint_id(&self) -> CheckpointId {
        self.checkpoint_id
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod cutover_state_tests {
    use std::path::Path;

    use redb::{Database, TableDefinition};
    use tempfile::TempDir;

    use super::*;
    use crate::{
        backend::redb::state::RecursiveV2CutoverManifestV2,
        fixture_support::settlement_corpus::{asset_item, load_fixture},
        settlement::hjmt_config::SettlementBackendMode,
    };

    const CUTOVER_TABLE: TableDefinition<&[u8], &[u8]> =
        TableDefinition::new("settlement_recursive_v2_cutover");
    const STORE_DB: &str = "settlement_state.redb";

    fn durable_store() -> (TempDir, SettlementStore) {
        let temp = tempfile::tempdir().expect("cutover tempdir");
        let mut store =
            SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)
                .expect("durable cutover store");
        let fixture = load_fixture();
        store
            .put_settlement_item(asset_item(&fixture.assets[0]))
            .expect("persisted cutover state");
        (temp, store)
    }

    fn cutover_manifest(store: &SettlementStore) -> RecursiveV2CutoverManifestV2 {
        let network_context = [1_u8; 32];
        let config_digest = [2_u8; 32];
        let policy_digest = store.bucket_policy().bucket_policy_id();
        let layout = 7_u32;
        let authority_generation = 1_u64;
        let noop_execution_input_version = 2_u8;
        let epoch_cadence_blocks = 1_000_u64;
        let authority_digest = sha256_256_role(
            CheckpointShaRole::UniquenessContext,
            &[
                &network_context,
                &config_digest,
                &policy_digest,
                &layout.to_le_bytes(),
                &authority_generation.to_le_bytes(),
                &[noop_execution_input_version],
                &epoch_cadence_blocks.to_le_bytes(),
            ],
        );
        let (
            snapshot_root,
            expected_definition_root,
            storage_generation,
            snapshot_record_count,
            snapshot_byte_count,
            snapshot_content_digest,
        ) = store
            .recursive_v2_snapshot_binding(layout)
            .expect("live cutover binding");
        let snapshot_id = [4_u8; 32];
        let snapshot_digest = sha256_256_role(
            CheckpointShaRole::Content,
            &[
                &snapshot_id,
                &storage_generation.to_le_bytes(),
                &snapshot_root,
                &expected_definition_root,
                &snapshot_record_count.to_le_bytes(),
                &snapshot_byte_count.to_le_bytes(),
                &snapshot_content_digest,
            ],
        );
        let opaque_last_root_record = [7_u8; 32];
        let pinned_opaque_record_digest = sha256_256_role(
            CheckpointShaRole::Link,
            &[
                b"z00z.recursive.v2.opaque-last-root-record",
                &opaque_last_root_record,
            ],
        );
        let height = 10_u64;
        let atomic_install_generation = 11_u64;
        let record_digest = sha256_256_role(
            CheckpointShaRole::Link,
            &[
                b"z00z.recursive.v2.cutover.manifest",
                &authority_digest,
                &snapshot_digest,
                &height.to_le_bytes(),
                &opaque_last_root_record,
                &pinned_opaque_record_digest,
                &expected_definition_root,
                &snapshot_root,
                &atomic_install_generation.to_le_bytes(),
            ],
        );
        RecursiveV2CutoverManifestV2 {
            schema_version: RecursiveV2CutoverManifestV2::SCHEMA_VERSION,
            authority_digest,
            network_context,
            config_digest,
            policy_digest,
            layout,
            authority_generation,
            noop_execution_input_version,
            epoch_cadence_blocks,
            snapshot_id,
            snapshot_digest,
            snapshot_storage_generation: storage_generation,
            snapshot_root,
            snapshot_record_count,
            snapshot_byte_count,
            snapshot_content_digest,
            height,
            opaque_last_root_record,
            pinned_opaque_record_digest,
            expected_definition_root,
            expected_settlement_root: snapshot_root,
            storage_generation,
            atomic_install_generation,
            record_digest,
        }
    }

    fn install_cutover(store: &mut SettlementStore) -> RecursiveV2CutoverManifestV2 {
        let manifest = cutover_manifest(store);
        store
            .install_recursive_v2_cutover(manifest.clone())
            .expect("durable cutover install");
        manifest
    }

    fn corrupt_cutover(root: &Path) {
        let database = Database::open(root.join(STORE_DB)).expect("open cutover database");
        let write = database.begin_write().expect("begin corrupt write");
        {
            let mut table = write.open_table(CUTOVER_TABLE).expect("open cutover table");
            table
                .insert(&b"installed"[..], &b"corrupt"[..])
                .expect("corrupt cutover record");
        }
        write.commit().expect("commit corrupt record");
    }

    #[test]
    fn test_missing_cutover_rejected() {
        let (_temp, store) = durable_store();
        assert!(store.load_recursive_v2_cutover().is_err());
    }

    #[test]
    fn test_corrupt_cutover_rejected() {
        let (temp, mut store) = durable_store();
        install_cutover(&mut store);
        drop(store);
        corrupt_cutover(temp.path());
        let store =
            SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)
                .expect("reload corrupt cutover store");
        assert!(store.load_recursive_v2_cutover().is_err());
    }

    #[test]
    fn test_stale_cutover_rejected() {
        let (_temp, mut store) = durable_store();
        install_cutover(&mut store);
        let fixture = load_fixture();
        store
            .put_settlement_item(asset_item(&fixture.assets[1]))
            .expect("advance settlement generation");
        assert!(store.load_recursive_v2_cutover().is_err());
    }

    #[test]
    fn test_cutover_state_is_canonical() {
        let (_temp, mut store) = durable_store();
        let manifest = install_cutover(&mut store);
        let first = RecursiveFinalizedIvcStateV2::from_cutover_store(&store)
            .expect("canonical cutover state");
        let second = RecursiveFinalizedIvcStateV2::from_cutover_store(&store)
            .expect("repeat canonical cutover state");

        assert_eq!(first, second);
        assert_eq!(first.height, manifest.height - 1);
        assert_eq!(first.cumulative_steps, 0);
        assert_eq!(first.checkpoint_id.as_bytes(), &manifest.record_digest);
        assert_eq!(first.public_input_digest, manifest.record_digest);
        assert_eq!(first.transition_statement_digest, manifest.record_digest);
        assert_eq!(first.checkpoint_link_digest, manifest.record_digest);
        assert_eq!(first.settlement_root, manifest.expected_settlement_root);
        assert_eq!(first.definition_root, manifest.expected_definition_root);
        assert_ne!(first.digest, [0_u8; 32]);
    }
}

/// Acyclic proof-facing recursive block input `X_h`.
///
/// Its canonical bytes are built only from storage-resolved transition data,
/// the independently evaluated trace, a validated verifier bundle, and the
/// prior finalized IVC endpoint.  Proof bytes, `z_h`, receipts, and worker
/// measurements have no field or constructor path into this object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(super) struct RecursiveNovaStepInputV2 {
    checkpoint_context: RecursiveCheckpointContextV2,
    chain_context: [u8; 32],
    config_digest: [u8; 32],
    policy_digest: [u8; 32],
    authority_digest: [u8; 32],
    authority_generation: u64,
    layout: u32,
    noop_execution_input_version: u8,
    epoch_cadence_blocks: u64,
    height: u64,
    predecessor_height: u64,
    checkpoint_id: CheckpointId,
    predecessor: Option<CheckpointId>,
    predicate_digest: [u8; 32],
    profile_digest: [u8; 32],
    spec_digest: [u8; 32],
    trace_grammar_digest: [u8; 32],
    pp_digest: [u8; 32],
    compressed_vk_digest: [u8; 32],
    verifier_bundle_digest: [u8; 32],
    pre_settlement_root: [u8; 32],
    post_settlement_root: [u8; 32],
    pre_definition_root: [u8; 32],
    post_definition_root: [u8; 32],
    tx_data_root: [u8; 32],
    delta_root: [u8; 32],
    witness_root: [u8; 32],
    trace_digest: [u8; 32],
    journal_digest: [u8; 32],
    checkpoint_link_digest: [u8; 32],
    spent_uniqueness_precommit: [u8; 32],
    output_uniqueness_precommit: [u8; 32],
    transition_statement_digest: [u8; 32],
    checkpoint_statement_digest: [u8; 32],
    checkpoint_statement_core_digest: [u8; 32],
    update_trace_digest: [u8; 32],
    declared_work_digest: [u8; 32],
    pre_uniqueness_context_digest: [u8; 32],
    semantic_counts: [u64; 8],
    opcode_counts: [u64; 17],
    tx_count: u32,
    trace_source_records: u64,
    trace_bytes: u64,
    prior: RecursiveFinalizedIvcStateV2,
    digest: [u8; 32],
}

#[derive(Clone, Copy)]
pub(crate) struct RecursiveNovaStateBindingsV2 {
    pub(crate) anchor_digests: [[u8; 32]; 17],
    pub(crate) anchor_scalars: [u64; 8],
    pub(crate) semantic_counts: [u64; 8],
    pub(crate) opcode_counts: [u64; 17],
    pub(crate) expected_trace_digest: [u8; 32],
    pub(crate) public_input_digest: [u8; 32],
    pub(crate) prior_finalized_state_digest: [u8; 32],
    pub(crate) post_settlement_root: [u8; 32],
    pub(crate) post_definition_root: [u8; 32],
    pub(crate) typed_checkpoint_commitments: [[u8; 32]; 4],
    pub(crate) statement_identity_digests: [[u8; 32]; 3],
}

impl RecursiveNovaStepInputV2 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        authority: RecursiveAuthorityContextV2,
        checkpoint: RecursiveCheckpointBindingV2,
        profile: &RecursiveCircuitProfileV2,
        statement: RecursiveTransitionStatementV2,
        trace: RecursiveTracePrecommitV2,
        pre_uniqueness_context: RecursivePreUniquenessContextV2,
        verifier: RecursiveVerifierInputBindingV2,
        prior: RecursiveFinalizedIvcStateV2,
    ) -> Result<Self, CheckpointError> {
        let spec = RecursiveCircuitSpecV2::new(authority.layout(), profile)?;
        let predecessor_height = checkpoint
            .height()
            .checked_sub(1)
            .ok_or(CheckpointError::Invariant)?;
        if prior.height != predecessor_height
            || prior.settlement_root != *statement.pre_settlement_root().as_bytes()
            || prior.definition_root != statement.pre_definition_root()
            || checkpoint.prior_recursive_output_root() != Some(prior.digest)
            || statement.height() != checkpoint.height()
            || statement.checkpoint_id() != checkpoint.checkpoint_id()
            || statement.predecessor() != checkpoint.predecessor()
            || statement.checkpoint_statement_digest() != checkpoint.statement_digest()
            || statement.checkpoint_statement_core_digest() != checkpoint.statement_core_digest()
            || statement.checkpoint_link_digest() != checkpoint.checkpoint_link_digest()
            || statement.delta_root() != checkpoint.delta_root()
            || statement.witness_root() != checkpoint.witness_root()
            || statement.journal_digest() != checkpoint.journal_digest()
            || statement.trace_digest() != trace.trace_digest()
            || statement.declared_event_count() != trace.event_count()
            || statement.declared_byte_count() != trace.byte_count()
            || statement.pre_uniqueness_context_digest() != pre_uniqueness_context.digest()
            || verifier.predicate_digest != pre_uniqueness_context.predicate_digest
            || verifier.profile_digest != profile.digest()
            || verifier.spec_digest != spec.digest()
            || verifier.verifier_bundle_digest != pre_uniqueness_context.verifier_bundle_digest
        {
            return Err(CheckpointError::Authority);
        }
        match (checkpoint.predecessor(), predecessor_height) {
            (None, 0) => {}
            (Some(predecessor), _) if predecessor == prior.checkpoint_id => {}
            _ => return Err(CheckpointError::Authority),
        }
        let declared_work = pre_uniqueness_context.declared_work();
        if declared_work.digest() != statement.declared_work_digest()
            || declared_work.event_counts() != statement.declared_event_counts()
            || statement.declared_event_counts() != statement.consumed_event_counts()
        {
            return Err(CheckpointError::Invariant);
        }
        let mut input = Self {
            checkpoint_context: authority.checkpoint_context(),
            chain_context: authority.network_context(),
            config_digest: authority.config_digest(),
            policy_digest: authority.policy_digest(),
            authority_digest: authority.digest(),
            authority_generation: authority.authority_generation(),
            layout: authority.layout(),
            noop_execution_input_version: authority.noop_execution_input_version(),
            epoch_cadence_blocks: authority.epoch_cadence_blocks(),
            height: checkpoint.height(),
            predecessor_height,
            checkpoint_id: checkpoint.checkpoint_id(),
            predecessor: checkpoint.predecessor(),
            predicate_digest: verifier.predicate_digest,
            profile_digest: verifier.profile_digest,
            spec_digest: verifier.spec_digest,
            trace_grammar_digest: RecursiveTraceOpcodeV2::grammar_digest(),
            pp_digest: verifier.pp_digest,
            compressed_vk_digest: verifier.compressed_vk_digest,
            verifier_bundle_digest: verifier.verifier_bundle_digest,
            pre_settlement_root: *statement.pre_settlement_root().as_bytes(),
            post_settlement_root: *statement.post_settlement_root().as_bytes(),
            pre_definition_root: statement.pre_definition_root(),
            post_definition_root: statement.post_definition_root(),
            tx_data_root: statement.checkpoint_exec_tx_root(),
            delta_root: statement.delta_root(),
            witness_root: statement.witness_root(),
            trace_digest: statement.trace_digest(),
            journal_digest: statement.journal_digest(),
            checkpoint_link_digest: statement.checkpoint_link_digest(),
            spent_uniqueness_precommit: statement.spent_uniqueness_precommit(),
            output_uniqueness_precommit: statement.output_uniqueness_precommit(),
            transition_statement_digest: statement.digest(),
            checkpoint_statement_digest: statement.checkpoint_statement_digest(),
            checkpoint_statement_core_digest: statement.checkpoint_statement_core_digest(),
            update_trace_digest: statement.update_trace_digest(),
            declared_work_digest: statement.declared_work_digest(),
            pre_uniqueness_context_digest: statement.pre_uniqueness_context_digest(),
            semantic_counts: declared_work.semantic_counts(),
            opcode_counts: declared_work.event_counts().counts(),
            tx_count: statement.checkpoint_exec_tx_count(),
            trace_source_records: trace.event_count(),
            trace_bytes: trace.byte_count(),
            prior,
            digest: [0_u8; 32],
        };
        input.digest = sha256_256_role(CheckpointShaRole::Statement, &[&input.canonical_prefix()]);
        Ok(input)
    }

    fn canonical_prefix(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&NOVA_STEP_INPUT_MAGIC_V2);
        bytes.push(NOVA_STEP_INPUT_VERSION_V2);
        for digest in [
            self.chain_context,
            self.config_digest,
            self.policy_digest,
            self.authority_digest,
        ] {
            bytes.extend_from_slice(&digest);
        }
        bytes.extend_from_slice(&self.authority_generation.to_le_bytes());
        bytes.extend_from_slice(&self.layout.to_le_bytes());
        bytes.push(self.noop_execution_input_version);
        bytes.extend_from_slice(&self.epoch_cadence_blocks.to_le_bytes());
        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(&self.predecessor_height.to_le_bytes());
        bytes.extend_from_slice(self.checkpoint_id.as_bytes());
        match self.predecessor {
            Some(predecessor) => {
                bytes.push(1);
                bytes.extend_from_slice(predecessor.as_bytes());
            }
            None => {
                bytes.push(0);
                bytes.extend_from_slice(&[0_u8; 32]);
            }
        }
        for digest in [
            self.predicate_digest,
            self.profile_digest,
            self.spec_digest,
            self.trace_grammar_digest,
            self.pp_digest,
            self.compressed_vk_digest,
            self.verifier_bundle_digest,
            self.pre_settlement_root,
            self.post_settlement_root,
            self.pre_definition_root,
            self.post_definition_root,
            self.tx_data_root,
            self.delta_root,
            self.witness_root,
            self.trace_digest,
            self.journal_digest,
            self.checkpoint_link_digest,
            self.spent_uniqueness_precommit,
            self.output_uniqueness_precommit,
            self.transition_statement_digest,
            self.checkpoint_statement_digest,
            self.checkpoint_statement_core_digest,
            self.update_trace_digest,
            self.declared_work_digest,
            self.pre_uniqueness_context_digest,
            self.prior.digest,
        ] {
            bytes.extend_from_slice(&digest);
        }
        bytes.extend_from_slice(&self.semantic_counts.map(u64::to_le_bytes).concat());
        bytes.extend_from_slice(&self.opcode_counts.map(u64::to_le_bytes).concat());
        bytes.extend_from_slice(&self.tx_count.to_le_bytes());
        bytes.extend_from_slice(&self.trace_source_records.to_le_bytes());
        bytes.extend_from_slice(&self.trace_bytes.to_le_bytes());
        bytes.extend_from_slice(&self.prior.height.to_le_bytes());
        bytes.extend_from_slice(&self.prior.cumulative_steps.to_le_bytes());
        bytes.extend_from_slice(&(NOVA_BACKEND_LABEL_V2.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&NOVA_BACKEND_LABEL_V2);
        bytes.extend_from_slice(&(NOVA_PROOF_MODE_V2.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&NOVA_PROOF_MODE_V2);
        bytes
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub const fn prior_finalized_state_digest(&self) -> [u8; 32] {
        self.prior.digest
    }

    #[must_use]
    pub(crate) const fn checkpoint_id(&self) -> CheckpointId {
        self.checkpoint_id
    }

    #[must_use]
    pub(crate) const fn predecessor(&self) -> Option<CheckpointId> {
        self.predecessor
    }

    #[must_use]
    pub(crate) const fn trace_digest(&self) -> [u8; 32] {
        self.trace_digest
    }

    #[must_use]
    pub(crate) const fn config_digest(&self) -> [u8; 32] {
        self.config_digest
    }

    #[must_use]
    pub(crate) const fn authority_generation(&self) -> u64 {
        self.authority_generation
    }

    pub(crate) const fn nova_state_bindings(&self) -> RecursiveNovaStateBindingsV2 {
        let (predecessor_id, predecessor_present) = match self.predecessor {
            Some(predecessor) => (*predecessor.as_bytes(), 1),
            None => ([0_u8; 32], 0),
        };
        RecursiveNovaStateBindingsV2 {
            anchor_digests: [
                self.chain_context,
                self.config_digest,
                self.policy_digest,
                self.authority_digest,
                self.pre_settlement_root,
                self.pre_definition_root,
                self.tx_data_root,
                self.update_trace_digest,
                self.predicate_digest,
                self.profile_digest,
                self.spec_digest,
                self.trace_grammar_digest,
                self.verifier_bundle_digest,
                self.declared_work_digest,
                self.pre_uniqueness_context_digest,
                *self.checkpoint_id.as_bytes(),
                predecessor_id,
            ],
            anchor_scalars: [
                PRE_UNIQUENESS_CONTEXT_VERSION_V2 as u64,
                self.height,
                self.predecessor_height,
                self.layout as u64,
                self.authority_generation,
                self.noop_execution_input_version as u64,
                self.epoch_cadence_blocks,
                predecessor_present,
            ],
            semantic_counts: self.semantic_counts,
            opcode_counts: self.opcode_counts,
            expected_trace_digest: self.trace_digest,
            public_input_digest: self.digest,
            prior_finalized_state_digest: self.prior.digest,
            post_settlement_root: self.post_settlement_root,
            post_definition_root: self.post_definition_root,
            typed_checkpoint_commitments: [
                self.delta_root,
                self.witness_root,
                self.journal_digest,
                self.checkpoint_link_digest,
            ],
            statement_identity_digests: [
                self.transition_statement_digest,
                self.checkpoint_statement_digest,
                self.checkpoint_statement_core_digest,
            ],
        }
    }

    pub(crate) fn validate_recovery_identity(&self) -> Result<(), CheckpointError> {
        if self.height == 0
            || self.predecessor_height.checked_add(1) != Some(self.height)
            || self.digest
                != sha256_256_role(CheckpointShaRole::Statement, &[&self.canonical_prefix()])
            || self.prior.digest
                != sha256_256_role(
                    CheckpointShaRole::Statement,
                    &[&self.prior.canonical_prefix()],
                )
            || self.prior.height != self.predecessor_height
        {
            return Err(CheckpointError::Canonical);
        }
        Ok(())
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn nova_envelope_fixture(
        context: RecursivePreUniquenessContextV2,
        trace: RecursiveTracePrecommitV2,
        verifier: RecursiveVerifierInputBindingV2,
    ) -> Self {
        assert_eq!(context.height, 1, "envelope fixture starts at cutover");
        let prior = RecursiveFinalizedIvcStateV2::cutover_fixture(
            CheckpointId::new([1_u8; 32]),
            context.old_settlement_root,
            context.old_definition_root,
            [4_u8; 32],
        )
        .expect("nonzero cutover fixture");
        let semantic_counts = context.declared_work.semantic_counts();
        let (spent_uniqueness_precommit, output_uniqueness_precommit) =
            super::recursive_semantics::uniqueness_set_precommits_from_parts(
                context.digest,
                u32::try_from(semantic_counts[1]).expect("fixture spent count"),
                u32::try_from(semantic_counts[2]).expect("fixture output count"),
                trace.spent_original_ids_digest(),
                trace.spent_sorted_ids_digest(),
                trace.output_original_ids_digest(),
                trace.output_sorted_ids_digest(),
            );
        let checkpoint_context = RecursiveCheckpointContextV2::test_fixture(
            context.config_digest,
            verifier.predicate_digest,
        );
        let mut input = Self {
            checkpoint_context,
            chain_context: checkpoint_context.digest(),
            config_digest: context.config_digest,
            policy_digest: context.policy_digest,
            authority_digest: context.authority_digest,
            authority_generation: context.authority_generation,
            layout: context.layout,
            noop_execution_input_version: context.noop_execution_input_version,
            epoch_cadence_blocks: 1_000,
            height: context.height,
            predecessor_height: context.predecessor_height,
            checkpoint_id: CheckpointId::new([9; 32]),
            predecessor: None,
            predicate_digest: verifier.predicate_digest,
            profile_digest: verifier.profile_digest,
            spec_digest: verifier.spec_digest,
            trace_grammar_digest: context.trace_grammar_digest,
            pp_digest: verifier.pp_digest,
            compressed_vk_digest: verifier.compressed_vk_digest,
            verifier_bundle_digest: verifier.verifier_bundle_digest,
            pre_settlement_root: context.old_settlement_root,
            post_settlement_root: context.old_settlement_root,
            pre_definition_root: context.old_definition_root,
            post_definition_root: context.old_definition_root,
            tx_data_root: context.tx_data_root,
            delta_root: [20; 32],
            witness_root: [21; 32],
            trace_digest: trace.trace_digest(),
            journal_digest: [23; 32],
            checkpoint_link_digest: [24; 32],
            spent_uniqueness_precommit,
            output_uniqueness_precommit,
            transition_statement_digest: [27; 32],
            checkpoint_statement_digest: [28; 32],
            checkpoint_statement_core_digest: [29; 32],
            update_trace_digest: context.update_trace_digest,
            declared_work_digest: context.declared_work.digest(),
            pre_uniqueness_context_digest: context.digest,
            semantic_counts,
            opcode_counts: context.declared_work.event_counts().counts(),
            tx_count: 1,
            trace_source_records: trace.event_count(),
            trace_bytes: trace.byte_count(),
            prior,
            digest: [0; 32],
        };
        input.digest = sha256_256_role(CheckpointShaRole::Statement, &[&input.canonical_prefix()]);
        input
    }
}

/// Exact portable recursive-checkpoint V2 public input.
///
/// This is distinct from the private Nova running-state binding. It carries
/// only the nineteen fields in the public contract and is derived from one
/// storage-owned statement plus the installed process context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursiveCheckpointPublicInputV2 {
    version: u16,
    context_digest: [u8; 32],
    statement_digest: [u8; 32],
    statement_core_digest: [u8; 32],
    height: u64,
    chain_index: u32,
    chain_length: u32,
    epoch_index: u64,
    epoch_start_height: u64,
    epoch_end_height: u64,
    prev_root: [u8; 32],
    output_root: [u8; 32],
    prior_output_root: [u8; 32],
    delta_root: [u8; 32],
    witness_root: [u8; 32],
    checkpoint_link_digest: [u8; 32],
    backend_label: [u8; 28],
    verifier_params_digest: [u8; 32],
    proof_mode: [u8; 27],
}

impl RecursiveCheckpointPublicInputV2 {
    pub(super) fn from_nova_step(
        step: &RecursiveNovaStepInputV2,
        chain_index: u32,
        chain_length: u32,
    ) -> Result<Self, CheckpointError> {
        if chain_length == 0
            || chain_index >= chain_length
            || step.height == 0
            || step.epoch_cadence_blocks == 0
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StepSkipped,
            ));
        }
        if step.checkpoint_context.digest() != step.chain_context
            || step.checkpoint_context.checkpoint_config_digest() != step.config_digest
            || step.checkpoint_context.predicate_digest() != step.predicate_digest
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::PublicInputDigestMismatch,
            ));
        }
        let zero_based_height = step
            .height
            .checked_sub(1)
            .ok_or(CheckpointError::Overflow)?;
        let epoch_index = zero_based_height / step.epoch_cadence_blocks;
        let epoch_start_height = epoch_index
            .checked_mul(step.epoch_cadence_blocks)
            .and_then(|value| value.checked_add(1))
            .ok_or(CheckpointError::Overflow)?;
        let epoch_end_height = epoch_index
            .checked_add(1)
            .and_then(|value| value.checked_mul(step.epoch_cadence_blocks))
            .ok_or(CheckpointError::Overflow)?;
        Ok(Self {
            version: CHECKPOINT_PUBLIC_INPUT_VERSION_V2,
            context_digest: step.checkpoint_context.digest(),
            statement_digest: step.checkpoint_statement_digest,
            statement_core_digest: step.checkpoint_statement_core_digest,
            height: step.height,
            chain_index,
            chain_length,
            epoch_index,
            epoch_start_height,
            epoch_end_height,
            prev_root: step.pre_settlement_root,
            output_root: step.post_settlement_root,
            prior_output_root: step.pre_settlement_root,
            delta_root: step.delta_root,
            witness_root: step.witness_root,
            checkpoint_link_digest: step.checkpoint_link_digest,
            backend_label: NOVA_BACKEND_LABEL_V2,
            verifier_params_digest: step.verifier_bundle_digest,
            proof_mode: NOVA_PROOF_MODE_V2,
        })
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(CHECKPOINT_PUBLIC_INPUT_BYTES_V2);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.context_digest);
        bytes.extend_from_slice(&self.statement_digest);
        bytes.extend_from_slice(&self.statement_core_digest);
        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(&self.chain_index.to_le_bytes());
        bytes.extend_from_slice(&self.chain_length.to_le_bytes());
        bytes.extend_from_slice(&self.epoch_index.to_le_bytes());
        bytes.extend_from_slice(&self.epoch_start_height.to_le_bytes());
        bytes.extend_from_slice(&self.epoch_end_height.to_le_bytes());
        bytes.extend_from_slice(&self.prev_root);
        bytes.extend_from_slice(&self.output_root);
        bytes.extend_from_slice(&self.prior_output_root);
        bytes.extend_from_slice(&self.delta_root);
        bytes.extend_from_slice(&self.witness_root);
        bytes.extend_from_slice(&self.checkpoint_link_digest);
        bytes.extend_from_slice(&self.backend_label);
        bytes.extend_from_slice(&self.verifier_params_digest);
        bytes.extend_from_slice(&self.proof_mode);
        bytes
    }

    pub(super) fn decode_canonical(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() != CHECKPOINT_PUBLIC_INPUT_BYTES_V2 {
            return Err(CheckpointError::Canonical);
        }
        let mut cursor = 0;
        let version = u16::from_le_bytes(take_public_input::<2>(bytes, &mut cursor)?);
        if version != CHECKPOINT_PUBLIC_INPUT_VERSION_V2 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::UnsupportedVersion,
            ));
        }
        let context_digest = take_public_input::<32>(bytes, &mut cursor)?;
        let statement_digest = take_public_input::<32>(bytes, &mut cursor)?;
        let statement_core_digest = take_public_input::<32>(bytes, &mut cursor)?;
        let height = u64::from_le_bytes(take_public_input::<8>(bytes, &mut cursor)?);
        let chain_index = u32::from_le_bytes(take_public_input::<4>(bytes, &mut cursor)?);
        let chain_length = u32::from_le_bytes(take_public_input::<4>(bytes, &mut cursor)?);
        let epoch_index = u64::from_le_bytes(take_public_input::<8>(bytes, &mut cursor)?);
        let epoch_start_height = u64::from_le_bytes(take_public_input::<8>(bytes, &mut cursor)?);
        let epoch_end_height = u64::from_le_bytes(take_public_input::<8>(bytes, &mut cursor)?);
        let prev_root = take_public_input::<32>(bytes, &mut cursor)?;
        let output_root = take_public_input::<32>(bytes, &mut cursor)?;
        let prior_output_root = take_public_input::<32>(bytes, &mut cursor)?;
        let delta_root = take_public_input::<32>(bytes, &mut cursor)?;
        let witness_root = take_public_input::<32>(bytes, &mut cursor)?;
        let checkpoint_link_digest = take_public_input::<32>(bytes, &mut cursor)?;
        let backend_label = take_public_input::<28>(bytes, &mut cursor)?;
        let verifier_params_digest = take_public_input::<32>(bytes, &mut cursor)?;
        let proof_mode = take_public_input::<27>(bytes, &mut cursor)?;
        if cursor != bytes.len() {
            return Err(CheckpointError::Canonical);
        }
        if backend_label != NOVA_BACKEND_LABEL_V2 || proof_mode != NOVA_PROOF_MODE_V2 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::BackendUnsupported,
            ));
        }
        if height == 0 || chain_length == 0 || chain_index >= chain_length {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StepSkipped,
            ));
        }
        if prev_root != prior_output_root {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
            ));
        }
        let cadence = epoch_end_height
            .checked_sub(epoch_start_height)
            .and_then(|value| value.checked_add(1))
            .ok_or(CheckpointError::Overflow)?;
        let expected_start = epoch_index
            .checked_mul(cadence)
            .and_then(|value| value.checked_add(1))
            .ok_or(CheckpointError::Overflow)?;
        let expected_end = epoch_index
            .checked_add(1)
            .and_then(|value| value.checked_mul(cadence))
            .ok_or(CheckpointError::Overflow)?;
        if cadence == 0
            || epoch_start_height != expected_start
            || epoch_end_height != expected_end
            || !(epoch_start_height..=epoch_end_height).contains(&height)
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::HybridCadenceMismatch,
            ));
        }
        let input = Self {
            version,
            context_digest,
            statement_digest,
            statement_core_digest,
            height,
            chain_index,
            chain_length,
            epoch_index,
            epoch_start_height,
            epoch_end_height,
            prev_root,
            output_root,
            prior_output_root,
            delta_root,
            witness_root,
            checkpoint_link_digest,
            backend_label,
            verifier_params_digest,
            proof_mode,
        };
        if input.canonical_bytes() != bytes {
            return Err(CheckpointError::Canonical);
        }
        Ok(input)
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        sha256_256(
            CHECKPOINT_PUBLIC_INPUT_DOMAIN_V2,
            CHECKPOINT_PUBLIC_INPUT_LABEL_V2,
            &[&self.canonical_bytes()],
        )
    }

    #[must_use]
    pub const fn statement_digest(&self) -> [u8; 32] {
        self.statement_digest
    }

    #[must_use]
    pub const fn statement_core_digest(&self) -> [u8; 32] {
        self.statement_core_digest
    }

    #[must_use]
    pub const fn prior_output_root(&self) -> [u8; 32] {
        self.prior_output_root
    }

    #[must_use]
    pub const fn output_root(&self) -> [u8; 32] {
        self.output_root
    }

    #[must_use]
    pub const fn verifier_params_digest(&self) -> [u8; 32] {
        self.verifier_params_digest
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub const fn checkpoint_link_digest(&self) -> [u8; 32] {
        self.checkpoint_link_digest
    }

    #[cfg(test)]
    pub(crate) fn validate_required_local_chain(&self) -> Result<(), CheckpointError> {
        if self.chain_length < 3 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::ChainTooShort,
            ));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn validate_diagnostic_single_step(&self) -> Result<(), CheckpointError> {
        if self.chain_length != 1 || self.chain_index != 0 {
            return Err(CheckpointError::Invariant);
        }
        Ok(())
    }
}

fn take_public_input<const N: usize>(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], CheckpointError> {
    let end = cursor.checked_add(N).ok_or(CheckpointError::Overflow)?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(CheckpointError::Canonical)?
        .try_into()
        .map_err(|_| CheckpointError::Canonical)?;
    *cursor = end;
    Ok(value)
}

#[cfg(test)]
// Keep canonical statement serialization as the final production item while
// colocating the public-input regression corpus with its decoder above.
#[allow(clippy::items_after_test_module)]
mod public_input_tests {
    use super::*;

    fn sample() -> RecursiveCheckpointPublicInputV2 {
        RecursiveCheckpointPublicInputV2 {
            version: 2,
            context_digest: [1; 32],
            statement_digest: [2; 32],
            statement_core_digest: [3; 32],
            height: 5,
            chain_index: 2,
            chain_length: 3,
            epoch_index: 0,
            epoch_start_height: 1,
            epoch_end_height: 1_000,
            prev_root: [4; 32],
            output_root: [5; 32],
            prior_output_root: [4; 32],
            delta_root: [6; 32],
            witness_root: [7; 32],
            checkpoint_link_digest: [8; 32],
            backend_label: NOVA_BACKEND_LABEL_V2,
            verifier_params_digest: [9; 32],
            proof_mode: NOVA_PROOF_MODE_V2,
        }
    }

    #[test]
    fn test_public_input_binds_fields() {
        let input = sample();
        let expected = [
            0xd1, 0xbe, 0x92, 0xaa, 0x35, 0x60, 0xce, 0xd5, 0x43, 0x45, 0x8e, 0x3c, 0xbe, 0x62,
            0x6b, 0x6c, 0x0d, 0x12, 0x83, 0xee, 0x5e, 0xae, 0x18, 0xd5, 0xe4, 0x31, 0x9c, 0xbd,
            0x78, 0xf7, 0xb8, 0xf2,
        ];
        let bytes = input.canonical_bytes();
        assert_eq!(bytes.len(), CHECKPOINT_PUBLIC_INPUT_BYTES_V2);
        assert_eq!(input.digest(), expected);
        assert_eq!(
            RecursiveCheckpointPublicInputV2::decode_canonical(&bytes).ok(),
            Some(input)
        );

        let mut backend_label = NOVA_BACKEND_LABEL_V2;
        backend_label[0] ^= 1;
        let mut proof_mode = NOVA_PROOF_MODE_V2;
        proof_mode[0] ^= 1;
        let mutations = [
            RecursiveCheckpointPublicInputV2 {
                version: 3,
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                context_digest: [10; 32],
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                statement_digest: [10; 32],
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                statement_core_digest: [10; 32],
                ..input
            },
            RecursiveCheckpointPublicInputV2 { height: 6, ..input },
            RecursiveCheckpointPublicInputV2 {
                chain_index: 1,
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                chain_length: 4,
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                epoch_index: 1,
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                epoch_start_height: 2,
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                epoch_end_height: 999,
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                prev_root: [10; 32],
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                output_root: [10; 32],
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                prior_output_root: [10; 32],
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                delta_root: [10; 32],
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                witness_root: [10; 32],
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                checkpoint_link_digest: [10; 32],
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                backend_label,
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                verifier_params_digest: [10; 32],
                ..input
            },
            RecursiveCheckpointPublicInputV2 {
                proof_mode,
                ..input
            },
        ];
        assert!(mutations
            .into_iter()
            .all(|mutation| mutation.digest() != expected));
    }

    #[test]
    fn test_required_chain_bounds() {
        assert!(sample().validate_required_local_chain().is_ok());
        assert!(matches!(
            RecursiveCheckpointPublicInputV2 {
                chain_index: 0,
                chain_length: 2,
                ..sample()
            }
            .validate_required_local_chain(),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::ChainTooShort
            ))
        ));
        assert!(RecursiveCheckpointPublicInputV2 {
            chain_index: 5,
            chain_length: 6,
            ..sample()
        }
        .validate_required_local_chain()
        .is_ok());
    }

    #[test]
    fn test_public_input_rejects_width() {
        let bytes = sample().canonical_bytes();
        assert!(matches!(
            RecursiveCheckpointPublicInputV2::decode_canonical(&bytes[..bytes.len() - 1]),
            Err(CheckpointError::Canonical)
        ));
        let mut trailing = bytes;
        trailing.push(0);
        assert!(matches!(
            RecursiveCheckpointPublicInputV2::decode_canonical(&trailing),
            Err(CheckpointError::Canonical)
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn canonical_statement_bytes(
    authority: RecursiveAuthorityContextV2,
    snapshot: RecursiveSnapshotHandleV2,
    checkpoint: RecursiveCheckpointBindingV2,
    profile_digest: [u8; 32],
    spec_digest: [u8; 32],
    grammar_digest: [u8; 32],
    post_settlement_root: SettlementStateRoot,
    pre_definition_root: [u8; 32],
    post_definition_root: [u8; 32],
    update_trace_digest: [u8; 32],
    update_trace_count: u32,
    trace: RecursiveTracePrecommitV2,
    pre_uniqueness_context: RecursivePreUniquenessContextV2,
    spent_uniqueness_precommit: [u8; 32],
    output_uniqueness_precommit: [u8; 32],
    declared_event_counts: super::recursive_trace::RecursiveTraceEventCountsV2,
    consumed_event_counts: super::recursive_trace::RecursiveTraceEventCountsV2,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(1 + 32 * 23 + 8 * 6 + 4 * 2 + 2 * 14 * 8);
    bytes.push(RECURSIVE_TRANSITION_STATEMENT_VERSION_V2);
    bytes.extend_from_slice(checkpoint.checkpoint_id().as_bytes());
    bytes.extend_from_slice(&checkpoint.height().to_le_bytes());
    match checkpoint.predecessor() {
        Some(predecessor) => {
            bytes.push(1);
            bytes.extend_from_slice(predecessor.as_bytes());
        }
        None => {
            bytes.push(0);
            bytes.extend_from_slice(&[0_u8; 32]);
        }
    }
    bytes.extend_from_slice(&checkpoint.exec_tx_root());
    bytes.extend_from_slice(&checkpoint.exec_tx_count().to_le_bytes());
    bytes.extend_from_slice(&checkpoint.statement_digest());
    bytes.extend_from_slice(&checkpoint.statement_core_digest());
    bytes.extend_from_slice(&checkpoint.delta_root());
    bytes.extend_from_slice(&checkpoint.witness_root());
    bytes.extend_from_slice(&checkpoint.journal_digest());
    match checkpoint.prior_recursive_output_root() {
        Some(digest) => {
            bytes.push(1);
            bytes.extend_from_slice(&digest);
        }
        None => {
            bytes.push(0);
            bytes.extend_from_slice(&[0_u8; 32]);
        }
    }
    bytes.extend_from_slice(&checkpoint.checkpoint_link_digest());
    bytes.extend_from_slice(&authority.digest());
    bytes.extend_from_slice(&authority.network_context());
    bytes.extend_from_slice(&authority.config_digest());
    bytes.extend_from_slice(&authority.policy_digest());
    bytes.extend_from_slice(&authority.layout().to_le_bytes());
    bytes.extend_from_slice(&authority.authority_generation().to_le_bytes());
    bytes.extend_from_slice(snapshot.snapshot_id().as_bytes());
    bytes.extend_from_slice(&snapshot.digest());
    bytes.extend_from_slice(&snapshot.storage_generation().to_le_bytes());
    bytes.extend_from_slice(snapshot.root().as_bytes());
    bytes.extend_from_slice(&snapshot.record_count().to_le_bytes());
    bytes.extend_from_slice(&snapshot.byte_count().to_le_bytes());
    bytes.extend_from_slice(&snapshot.content_digest());
    bytes.extend_from_slice(&profile_digest);
    bytes.extend_from_slice(&spec_digest);
    bytes.extend_from_slice(&grammar_digest);
    bytes.extend_from_slice(&pre_definition_root);
    bytes.extend_from_slice(post_settlement_root.as_bytes());
    bytes.extend_from_slice(&post_definition_root);
    bytes.extend_from_slice(&update_trace_digest);
    bytes.extend_from_slice(&update_trace_count.to_le_bytes());
    bytes.extend_from_slice(&trace.trace_digest());
    bytes.extend_from_slice(&trace.spent_original_ids_digest());
    bytes.extend_from_slice(&trace.spent_sorted_ids_digest());
    bytes.extend_from_slice(&trace.output_original_ids_digest());
    bytes.extend_from_slice(&trace.output_sorted_ids_digest());
    bytes.extend_from_slice(&pre_uniqueness_context.declared_work().digest());
    bytes.extend_from_slice(&pre_uniqueness_context.digest());
    bytes.extend_from_slice(&spent_uniqueness_precommit);
    bytes.extend_from_slice(&output_uniqueness_precommit);
    bytes.extend_from_slice(&trace.event_count().to_le_bytes());
    bytes.extend_from_slice(&trace.byte_count().to_le_bytes());
    bytes.extend_from_slice(&declared_event_counts.canonical_bytes());
    bytes.extend_from_slice(&consumed_event_counts.canonical_bytes());
    bytes
}
