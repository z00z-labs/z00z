//! One typed, acyclic V2 statement for the recursive checkpoint relation.
//!
//! The statement is constructed only after independent trace evaluation.  It
//! commits every currently materialized V2 relation field in a frozen order;
//! callers cannot substitute a generic configuration digest or a total count
//! for the individual authority, snapshot, root, grammar, trace, and JMT
//! bindings.

use z00z_crypto::{sha256_256_role, CheckpointShaRole};

use crate::{
    checkpoint::CheckpointId,
    settlement::{SettlementStateRoot, SettlementUpdateTraceEnvelopeV2},
};

use super::{
    recursive_circuit::{RecursiveCircuitProfileV2, RecursiveCircuitSpecV2},
    recursive_context::{
        RecursiveAuthorityContextV2, RecursiveCheckpointBindingV2, RecursiveSnapshotHandleV2,
    },
    recursive_reject::RecursiveV2Error,
    recursive_trace::{RecursiveTraceOpcodeV2, RecursiveTracePrecommitV2},
};

const RECURSIVE_TRANSITION_STATEMENT_VERSION_V2: u8 = 2;

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
    checkpoint_link_digest: [u8; 32],
    pre_settlement_root: SettlementStateRoot,
    post_settlement_root: SettlementStateRoot,
    trace_digest: [u8; 32],
    update_trace_digest: [u8; 32],
    declared_event_count: u64,
    declared_byte_count: u64,
}

impl RecursiveTransitionStatementV2 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        authority: RecursiveAuthorityContextV2,
        snapshot: RecursiveSnapshotHandleV2,
        checkpoint: RecursiveCheckpointBindingV2,
        profile: &RecursiveCircuitProfileV2,
        post_settlement_root: SettlementStateRoot,
        post_definition_root: [u8; 32],
        update_trace_digest: [u8; 32],
        update_trace_count: u32,
        trace: RecursiveTracePrecommitV2,
    ) -> Result<Self, RecursiveV2Error> {
        let authority_noop = checkpoint.is_recursive_v2_noop();
        if snapshot.root() != checkpoint.pre_settlement_root()
            || post_settlement_root != checkpoint.post_settlement_root()
            || snapshot.snapshot_id() != checkpoint.prep_snapshot_id()
            || snapshot.root().generation() != post_settlement_root.generation()
            || snapshot.root().generation().version() != 2
            || checkpoint.height() == 0
            || update_trace_digest == [0; 32]
            || (!authority_noop && update_trace_count == 0)
            || (authority_noop
                && (update_trace_count != 0
                    || post_settlement_root != checkpoint.pre_settlement_root()
                    || !SettlementUpdateTraceEnvelopeV2::is_noop_digest(update_trace_digest)))
            || trace.event_count() == 0
            || trace.byte_count() == 0
        {
            return Err(RecursiveV2Error::Invariant);
        }
        let spec = RecursiveCircuitSpecV2::new(authority.layout(), profile)?;
        let grammar_digest = RecursiveTraceOpcodeV2::grammar_digest();
        let canonical = canonical_statement_bytes(
            authority,
            snapshot,
            checkpoint,
            profile.digest(),
            spec.digest(),
            grammar_digest,
            post_settlement_root,
            post_definition_root,
            update_trace_digest,
            update_trace_count,
            trace,
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
            checkpoint_link_digest: checkpoint.checkpoint_link_digest(),
            pre_settlement_root: snapshot.root(),
            post_settlement_root,
            trace_digest: trace.trace_digest(),
            update_trace_digest,
            declared_event_count: trace.event_count(),
            declared_byte_count: trace.byte_count(),
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
    pub const fn trace_digest(&self) -> [u8; 32] {
        self.trace_digest
    }

    #[must_use]
    pub const fn update_trace_digest(&self) -> [u8; 32] {
        self.update_trace_digest
    }

    #[must_use]
    pub const fn declared_event_count(&self) -> u64 {
        self.declared_event_count
    }

    #[must_use]
    pub const fn declared_byte_count(&self) -> u64 {
        self.declared_byte_count
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
    post_definition_root: [u8; 32],
    update_trace_digest: [u8; 32],
    update_trace_count: u32,
    trace: RecursiveTracePrecommitV2,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(1 + 32 * 20 + 8 * 6 + 4 * 2);
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
    bytes.extend_from_slice(post_settlement_root.as_bytes());
    bytes.extend_from_slice(&post_definition_root);
    bytes.extend_from_slice(&update_trace_digest);
    bytes.extend_from_slice(&update_trace_count.to_le_bytes());
    bytes.extend_from_slice(&trace.trace_digest());
    bytes.extend_from_slice(&trace.spent_original_ids_digest());
    bytes.extend_from_slice(&trace.spent_sorted_ids_digest());
    bytes.extend_from_slice(&trace.output_original_ids_digest());
    bytes.extend_from_slice(&trace.output_sorted_ids_digest());
    bytes.extend_from_slice(&trace.event_count().to_le_bytes());
    bytes.extend_from_slice(&trace.byte_count().to_le_bytes());
    bytes
}
