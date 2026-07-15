//! Authority-pinned V2 cutover contract and canonical transition boundary.

use std::path::Path;

use z00z_crypto::{sha256_256_role, CheckpointShaRole};

use crate::backend::redb::state::RecursiveV2CutoverManifestV2;
use crate::checkpoint::{CheckpointId, CheckpointStore};
use crate::settlement::{
    derive_settlement_root_v2, ScopeFlow, ScopeOpKind, SettlementExecHandoff, SettlementStateRoot,
    SettlementStore, SettlementUpdateTraceEnvelopeV2,
};
use crate::snapshot::PrepSnapshotStore;

use super::{
    recursive_circuit::RecursiveCircuitProfileV2,
    recursive_context::{
        RecursiveAuthoritySnapshotV2, RecursiveCheckpointBindingV2, RecursiveSnapshotHandleV2,
    },
    recursive_predicate::{CheckpointTransitionConsistencyV2, EvaluatedCheckpointTransitionV2},
    recursive_reject::RecursiveV2Error,
    recursive_semantics::{
        decode_canonical_hex32, decode_uniqueness_challenge, decode_uniqueness_precommit,
        encode_flow_header_with_v2_roots, encode_flow_item, encode_hierarchy_promotion,
        encode_net_merge, encode_uniqueness_challenge, encode_uniqueness_precommit,
    },
    recursive_trace::{
        structural_event_id, RecursiveTraceEventV2, RecursiveTraceOpcodeV2,
        RecursiveTracePrecommitV2, RecursiveTransitionTraceSourceV2,
    },
};

/// Isolated authority mode selected by the completed T0 evidence branch.
///
/// It deliberately cannot be represented as a live deployment authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettlementRootCutoverModeV2 {
    RepositoryLocalFixture,
}

/// Immutable record governing one V2 root-generation installation attempt.
pub struct SettlementRootGenerationCutoverV2 {
    mode: SettlementRootCutoverModeV2,
    authority: RecursiveAuthoritySnapshotV2,
    height: u64,
    opaque_last_root_record: [u8; 32],
    pinned_opaque_record_digest: [u8; 32],
    expected_definition_root: [u8; 32],
    expected_settlement_root: SettlementStateRoot,
    atomic_install_generation: u64,
    record_digest: [u8; 32],
}

impl SettlementRootGenerationCutoverV2 {
    /// Construct the only repository-local fixture cutover contract.
    ///
    /// This constructor validates fixture evidence only. It is intentionally
    /// not a production-authority constructor and cannot target a live root.
    #[allow(clippy::too_many_arguments)]
    pub fn repository_local_fixture(
        authority: RecursiveAuthoritySnapshotV2,
        store: &SettlementStore,
        height: u64,
        opaque_last_root_record: [u8; 32],
        pinned_opaque_record_digest: [u8; 32],
        expected_settlement_root: SettlementStateRoot,
        atomic_install_generation: u64,
    ) -> Result<Self, RecursiveV2Error> {
        if height == 0
            || atomic_install_generation == 0
            || opaque_record_digest(opaque_last_root_record) != pinned_opaque_record_digest
        {
            return Err(RecursiveV2Error::CutoverAuthority);
        }
        authority.revalidate_config()?;
        let context = authority.authority();
        let snapshot = authority.snapshot();
        let captured_snapshot =
            RecursiveSnapshotHandleV2::from_store(snapshot.snapshot_id(), store, context.layout())?;
        if captured_snapshot != snapshot {
            return Err(RecursiveV2Error::SnapshotChanged);
        }
        let expected_definition_root = store.recursive_v2_definition_root();
        let derived = derive_settlement_root_v2(
            expected_settlement_root.generation(),
            context.layout(),
            context.policy_digest(),
            expected_definition_root,
        )
        .map_err(|_| RecursiveV2Error::Root)?;
        if derived != expected_settlement_root {
            return Err(RecursiveV2Error::Root);
        }
        let height_bytes = height.to_le_bytes();
        let install_generation = atomic_install_generation.to_le_bytes();
        let record_digest = sha256_256_role(
            CheckpointShaRole::Link,
            &[
                b"z00z.recursive.v2.cutover.manifest",
                &context.digest(),
                &snapshot.digest(),
                &height_bytes,
                &opaque_last_root_record,
                &pinned_opaque_record_digest,
                &expected_definition_root,
                expected_settlement_root.as_bytes(),
                &install_generation,
            ],
        );
        Ok(Self {
            mode: SettlementRootCutoverModeV2::RepositoryLocalFixture,
            authority,
            height,
            opaque_last_root_record,
            pinned_opaque_record_digest,
            expected_definition_root,
            expected_settlement_root,
            atomic_install_generation,
            record_digest,
        })
    }

    #[must_use]
    pub const fn mode(&self) -> SettlementRootCutoverModeV2 {
        self.mode
    }
    #[must_use]
    pub const fn expected_settlement_root(&self) -> SettlementStateRoot {
        self.expected_settlement_root
    }
    #[must_use]
    pub const fn record_digest(&self) -> [u8; 32] {
        self.record_digest
    }
    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }
    #[must_use]
    pub const fn opaque_last_root_record(&self) -> [u8; 32] {
        self.opaque_last_root_record
    }

    /// Consume one exact fixture installation token exactly once.
    pub fn install_repository_fixture(
        &mut self,
        store: &mut SettlementStore,
        observed_install_generation: u64,
    ) -> Result<SettlementStateRoot, RecursiveV2Error> {
        if self.mode != SettlementRootCutoverModeV2::RepositoryLocalFixture
            || observed_install_generation != self.atomic_install_generation
        {
            return Err(RecursiveV2Error::CutoverInstall);
        }
        self.authority.revalidate_config()?;
        let context = self.authority.authority();
        let snapshot = self.authority.snapshot();
        let captured_snapshot =
            RecursiveSnapshotHandleV2::from_store(snapshot.snapshot_id(), store, context.layout())?;
        if snapshot != captured_snapshot {
            return Err(RecursiveV2Error::SnapshotChanged);
        }
        if store.recursive_v2_definition_root() != self.expected_definition_root {
            return Err(RecursiveV2Error::Root);
        }
        let derived = store
            .settlement_root_v2(context.layout())
            .map_err(|_| RecursiveV2Error::Root)?;
        if derived != self.expected_settlement_root {
            return Err(RecursiveV2Error::Root);
        }
        store
            .install_recursive_v2_cutover(self.durable_manifest())
            .map_err(|_| RecursiveV2Error::CutoverInstall)?;
        Ok(derived)
    }

    fn durable_manifest(&self) -> RecursiveV2CutoverManifestV2 {
        RecursiveV2CutoverManifestV2 {
            schema_version: RecursiveV2CutoverManifestV2::SCHEMA_VERSION,
            authority_digest: self.authority.authority().digest(),
            network_context: self.authority.authority().network_context(),
            config_digest: self.authority.authority().config_digest(),
            policy_digest: self.authority.authority().policy_digest(),
            layout: self.authority.authority().layout(),
            authority_generation: self.authority.authority().authority_generation(),
            noop_execution_input_version: self.authority.authority().noop_execution_input_version(),
            snapshot_id: *self.authority.snapshot().snapshot_id().as_bytes(),
            snapshot_digest: self.authority.snapshot().digest(),
            snapshot_storage_generation: self.authority.snapshot().storage_generation(),
            snapshot_root: *self.authority.snapshot().root().as_bytes(),
            snapshot_record_count: self.authority.snapshot().record_count(),
            snapshot_byte_count: self.authority.snapshot().byte_count(),
            snapshot_content_digest: self.authority.snapshot().content_digest(),
            height: self.height,
            opaque_last_root_record: self.opaque_last_root_record,
            pinned_opaque_record_digest: self.pinned_opaque_record_digest,
            expected_definition_root: self.expected_definition_root,
            expected_settlement_root: *self.expected_settlement_root.as_bytes(),
            storage_generation: self.authority.snapshot().storage_generation(),
            atomic_install_generation: self.atomic_install_generation,
            record_digest: self.record_digest,
        }
    }
}

fn opaque_record_digest(opaque_last_root_record: [u8; 32]) -> [u8; 32] {
    sha256_256_role(
        CheckpointShaRole::Link,
        &[
            b"z00z.recursive.v2.opaque-last-root-record",
            &opaque_last_root_record,
        ],
    )
}

/// Sole V2 transition orchestrator from a canonical settlement execution.
///
/// It owns the one HJMT update-trace envelope and the private two-pass spool.
/// Public callers provide a storage-owned `SettlementExecHandoff`, never an
/// event tape, final root, or evaluator result.
pub struct CanonicalCheckpointTransitionV2 {
    authority: RecursiveAuthoritySnapshotV2,
    checkpoint: RecursiveCheckpointBindingV2,
    post_settlement_root: SettlementStateRoot,
    post_storage_generation: u64,
    update_trace: SettlementUpdateTraceEnvelopeV2,
    source: RecursiveTransitionTraceSourceV2,
    precommit: RecursiveTracePrecommitV2,
}

impl CanonicalCheckpointTransitionV2 {
    /// Reload one canonical checkpoint binding, then commit its one storage
    /// execution and materialize the sole V2 proving source.
    ///
    /// Height, predecessor, statement digests, checkpoint link, and prep
    /// snapshot are resolved from the existing checkpoint and prep-snapshot
    /// stores. The caller supplies only their canonical checkpoint ID and
    /// storage-owned handoff; there is no parallel constructor for raw
    /// relation fields.
    pub fn from_exec(
        dir: impl AsRef<Path>,
        profile: RecursiveCircuitProfileV2,
        checkpoint_store: &impl CheckpointStore,
        prep_snapshot_store: &impl PrepSnapshotStore,
        checkpoint_id: CheckpointId,
        store: &mut SettlementStore,
        handoff: SettlementExecHandoff,
    ) -> Result<Self, RecursiveV2Error> {
        let checkpoint = RecursiveCheckpointBindingV2::resolve(
            checkpoint_store,
            prep_snapshot_store,
            checkpoint_id,
        )?;
        checkpoint.verify_handoff(&handoff)?;
        let authority =
            RecursiveAuthoritySnapshotV2::resolve_repository_local_fixture_for_snapshot(
                store,
                checkpoint.prep_snapshot_id(),
            )?;
        authority.revalidate_config()?;
        let context = authority.authority();
        if checkpoint.is_recursive_v2_noop()
            && !context.allows_noop_execution_input_version(checkpoint.exec_version())
        {
            return Err(RecursiveV2Error::Authority);
        }
        let snapshot = authority.snapshot();
        if context.policy_digest() != store.bucket_policy().bucket_policy_id() {
            return Err(RecursiveV2Error::CutoverAuthority);
        }
        let pre_root = store
            .settlement_root_v2(context.layout())
            .map_err(|_| RecursiveV2Error::Root)?;
        let captured_snapshot =
            RecursiveSnapshotHandleV2::from_store(snapshot.snapshot_id(), store, context.layout())?;
        if snapshot.root() != pre_root
            || checkpoint.pre_settlement_root() != pre_root
            || snapshot != captured_snapshot
        {
            return Err(RecursiveV2Error::SnapshotChanged);
        }

        let mut source = RecursiveTransitionTraceSourceV2::create_in(dir, profile, snapshot)?;
        source.begin_canonical_precommit()?;

        // All fallible witness construction is completed against an isolated
        // exact clone before the live HJMT can advance. Once the real commit
        // succeeds, this method only compares deterministic commitments and
        // returns the already-sealed source; it never needs to create, append,
        // or seal a spool after durable state has moved.
        let mut preflight_store = store.recursive_v2_preflight_clone();
        let (preflight_flow, preflight_update_trace) = preflight_store
            .apply_exec_handoff_v2(handoff.clone())
            .map_err(|_| RecursiveV2Error::Storage)?;
        let preflight_post_settlement_root = preflight_store
            .settlement_root_v2(context.layout())
            .map_err(|_| RecursiveV2Error::Root)?;
        let preflight_definition_root = preflight_store.recursive_v2_definition_root();
        if preflight_post_settlement_root != checkpoint.post_settlement_root() {
            return Err(RecursiveV2Error::Root);
        }
        canonical_events(
            &profile,
            &preflight_flow,
            context.digest(),
            preflight_definition_root,
            pre_root,
            preflight_post_settlement_root,
            &preflight_update_trace,
            checkpoint.is_recursive_v2_noop(),
            |event| source.append_canonical_event(event),
        )?;
        let precommit = source.seal_canonical_precommit()?;

        // The same immutable handoff now crosses the one live mutation
        // boundary. Its complete semantic result was already derived and
        // sealed from an exact, isolated clone above, so no witness/spool
        // operation remains that could turn a successful live commit into an
        // ambiguous post-commit error.
        let (live_flow, live_update_trace) = store
            .apply_exec_handoff_v2(handoff)
            .map_err(|_| RecursiveV2Error::Storage)?;
        let post_storage_generation = store.recursive_v2_storage_generation();
        let live_root = store.settlement_root_v2(context.layout()).map_err(|_| {
            RecursiveV2Error::CommittedWithoutSource {
                root: preflight_post_settlement_root,
                generation: post_storage_generation,
            }
        })?;
        let live_snapshot =
            RecursiveSnapshotHandleV2::from_store(snapshot.snapshot_id(), store, context.layout())
                .map_err(|_| RecursiveV2Error::CommittedWithoutSource {
                    root: live_root,
                    generation: post_storage_generation,
                })?;
        if live_flow != preflight_flow
            || live_update_trace != preflight_update_trace
            || live_root != preflight_post_settlement_root
            || store.recursive_v2_definition_root() != preflight_definition_root
            || live_snapshot.root() != live_root
            || live_snapshot.storage_generation() != post_storage_generation
        {
            return Err(RecursiveV2Error::CommittedWithoutSource {
                root: live_root,
                generation: post_storage_generation,
            });
        }

        Ok(Self {
            authority,
            checkpoint,
            post_settlement_root: preflight_post_settlement_root,
            post_storage_generation,
            update_trace: preflight_update_trace,
            source,
            precommit,
        })
    }

    #[must_use]
    pub const fn precommit(&self) -> RecursiveTracePrecommitV2 {
        self.precommit
    }

    #[must_use]
    pub const fn post_settlement_root(&self) -> SettlementStateRoot {
        self.post_settlement_root
    }

    /// Expose only the protocol-required digest of the private JMT witness.
    #[must_use]
    pub fn update_trace_digest(&self) -> [u8; 32] {
        self.update_trace.trace_digest()
    }

    /// Expose only the bounded count of private JMT updates.
    #[must_use]
    pub fn update_trace_count(&self) -> u32 {
        u32::try_from(self.update_trace.updates().len()).unwrap_or(u32::MAX)
    }

    /// Independently evaluate the sealed trace against the actual post-commit
    /// store, rejecting any concurrent root replacement before a result exists.
    pub fn evaluate(
        &mut self,
        store: &SettlementStore,
    ) -> Result<EvaluatedCheckpointTransitionV2, RecursiveV2Error> {
        self.authority.revalidate_config()?;
        if store
            .settlement_root_v2(self.authority.authority().layout())
            .map_err(|_| RecursiveV2Error::Root)?
            != self.post_settlement_root
            || store.recursive_v2_storage_generation() != self.post_storage_generation
        {
            return Err(RecursiveV2Error::SnapshotChanged);
        }
        let evaluated = CheckpointTransitionConsistencyV2::evaluate_stream(
            &mut self.source,
            self.authority.authority(),
            self.checkpoint,
            store,
        )?;
        if evaluated.settlement_root() != self.post_settlement_root {
            return Err(RecursiveV2Error::Root);
        }
        Ok(evaluated)
    }

    /// Finalize only when the immutable pre-state handle remains identical and
    /// the store still exposes the root evaluated from this trace.
    pub fn finish(
        &mut self,
        store: &SettlementStore,
    ) -> Result<RecursiveTracePrecommitV2, RecursiveV2Error> {
        self.authority.revalidate_config()?;
        if store
            .settlement_root_v2(self.authority.authority().layout())
            .map_err(|_| RecursiveV2Error::Root)?
            != self.post_settlement_root
            || store.recursive_v2_storage_generation() != self.post_storage_generation
        {
            return Err(RecursiveV2Error::SnapshotChanged);
        }
        self.source.finish(self.authority.snapshot())
    }
}

fn canonical_events(
    profile: &RecursiveCircuitProfileV2,
    flow: &ScopeFlow,
    authority_digest: [u8; 32],
    post_definition_root: [u8; 32],
    pre_settlement_root: SettlementStateRoot,
    post_settlement_root: SettlementStateRoot,
    update_trace: &SettlementUpdateTraceEnvelopeV2,
    authority_noop: bool,
    mut emit: impl FnMut(RecursiveTraceEventV2) -> Result<(), RecursiveV2Error>,
) -> Result<(), RecursiveV2Error> {
    if update_trace.is_noop() != authority_noop
        || (authority_noop
            && (!flow.items.is_empty() || pre_settlement_root != post_settlement_root))
    {
        return Err(RecursiveV2Error::Invariant);
    }
    let mut ordinal = 0_u64;
    emit(structural_event(
        ordinal,
        RecursiveTraceOpcodeV2::BeginBlock,
        encode_flow_header_with_v2_roots(flow, pre_settlement_root, post_settlement_root)?,
        profile,
    )?)?;
    ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;

    for item in ordered_flow_items(flow) {
        let opcode = match item.op_kind {
            ScopeOpKind::Delete => RecursiveTraceOpcodeV2::ReplayInput,
            ScopeOpKind::Put => RecursiveTraceOpcodeV2::ReplayOutput,
        };
        let object_id = decode_canonical_hex32(&item.terminal_id)?;
        emit(RecursiveTraceEventV2::new(
            ordinal,
            opcode,
            object_id,
            encode_flow_item(item)?,
            profile,
        )?)?;
        ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;
    }

    let uniqueness_precommit = encode_uniqueness_precommit(flow)?;
    let uniqueness = decode_uniqueness_precommit(&uniqueness_precommit)?;
    emit(structural_event(
        ordinal,
        RecursiveTraceOpcodeV2::UniquenessPrecommit,
        uniqueness_precommit,
        profile,
    )?)?;
    ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;
    let uniqueness_challenge = encode_uniqueness_challenge(authority_digest, uniqueness);
    let challenge =
        decode_uniqueness_challenge(&uniqueness_challenge, authority_digest, uniqueness)?;
    emit(structural_event(
        ordinal,
        RecursiveTraceOpcodeV2::UniquenessChallenge,
        uniqueness_challenge,
        profile,
    )?)?;
    ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;
    emit(structural_event(
        ordinal,
        RecursiveTraceOpcodeV2::NetMerge,
        encode_net_merge(uniqueness, challenge),
        profile,
    )?)?;
    ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;

    let envelope_bytes = update_trace
        .canonical_bytes()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    let chunk_bytes =
        usize::try_from(profile.max_leaf_bytes()).map_err(|_| RecursiveV2Error::Limit)?;
    if chunk_bytes == 0 {
        return Err(RecursiveV2Error::Limit);
    }
    for chunk in envelope_bytes.chunks(chunk_bytes) {
        emit(structural_event(
            ordinal,
            RecursiveTraceOpcodeV2::JmtUpdate,
            chunk.to_vec(),
            profile,
        )?)?;
        ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;
    }
    emit(structural_event(
        ordinal,
        RecursiveTraceOpcodeV2::PromoteChildRoot,
        encode_hierarchy_promotion(post_definition_root, update_trace.trace_digest()),
        profile,
    )?)?;
    ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;
    for item in ordered_flow_items(flow) {
        emit(structural_event(
            ordinal,
            RecursiveTraceOpcodeV2::CommitTypedEvent,
            encode_flow_item(item)?,
            profile,
        )?)?;
        ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;
    }
    emit(structural_event(
        ordinal,
        RecursiveTraceOpcodeV2::FinalizeBlock,
        encode_flow_header_with_v2_roots(flow, pre_settlement_root, post_settlement_root)?,
        profile,
    )?)?;
    ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;

    if ordinal > u64::from(profile.max_typed_events()) {
        return Err(RecursiveV2Error::Limit);
    }
    Ok(())
}

/// The frozen grammar consumes all deletes before all puts in both replay and
/// commit phases.  It does not preserve a caller's incidental operation order.
fn ordered_flow_items(flow: &ScopeFlow) -> impl Iterator<Item = &crate::settlement::ScopeFlowItem> {
    flow.items
        .iter()
        .filter(|item| item.op_kind == ScopeOpKind::Delete)
        .chain(
            flow.items
                .iter()
                .filter(|item| item.op_kind == ScopeOpKind::Put),
        )
}

fn structural_event(
    ordinal: u64,
    opcode: RecursiveTraceOpcodeV2,
    payload: Vec<u8>,
    profile: &RecursiveCircuitProfileV2,
) -> Result<RecursiveTraceEventV2, RecursiveV2Error> {
    let object_id = structural_event_id(opcode, ordinal, &payload);
    RecursiveTraceEventV2::new(ordinal, opcode, object_id, payload, profile)
}
