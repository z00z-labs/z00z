//! Authority-pinned V2 cutover contract and canonical transition boundary.

use std::{collections::BTreeSet, path::Path};

use z00z_crypto::{sha256_256_role, CheckpointSha256BlockStreamV2, CheckpointShaRole};

use crate::backend::redb::state::RecursiveV2CutoverManifestV2;
use crate::checkpoint::{CheckpointId, CheckpointStore};
use crate::settlement::{
    derive_settlement_root_v2, JmtTraceSegmentContextV2, ScopeFlow, ScopeOpKind,
    SettlementExecHandoff, SettlementStateRoot, SettlementStore, SettlementUpdateTraceEnvelopeV2,
};
use crate::snapshot::PrepSnapshotStore;

use super::{
    recursive_circuit::{RecursiveCircuitProfileV2, RecursiveCircuitSpecV2},
    recursive_context::{
        RecursiveAuthorityContextV2, RecursiveAuthoritySnapshotV2, RecursiveCheckpointBindingV2,
        RecursiveSnapshotHandleV2,
    },
    recursive_predicate::{CheckpointTransitionConsistencyV2, EvaluatedCheckpointTransitionV2},
    recursive_reject::RecursiveV2Error,
    recursive_semantics::{
        decode_uniqueness_challenge, decode_uniqueness_precommit, encode_flow_header_with_v2_roots,
        encode_flow_item, encode_hierarchy_promotion, encode_net_effect, encode_net_merge,
        encode_typed_checkpoint_commitment, encode_uniqueness_challenge,
        encode_uniqueness_precommit, encode_uniqueness_sorted_row, NetEffectV2,
        TypedCheckpointCommitmentKindV2, UniquenessListKindV2, UniquenessPassV2,
        UniquenessSetKindV2, UNIQUENESS_SEMANTIC_ROW_BYTES_V2,
    },
    recursive_statement::{
        RecursiveDeclaredWorkV2, RecursivePreUniquenessContextV2, RecursiveVerifierAuthorityV2,
    },
    recursive_trace::{
        structural_event_id, RecursiveTraceEventCountsV2, RecursiveTraceEventV2,
        RecursiveTraceOpcodeV2, RecursiveTracePrecommitV2, RecursiveTransitionTraceSourceV2,
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
            .map_err(|_error| {
                #[cfg(test)]
                eprintln!("recursive V2 durable cutover rejected: {_error}");
                RecursiveV2Error::CutoverInstall
            })?;
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
        Self::from_exec_with_verifier_inner(
            dir,
            profile,
            checkpoint_store,
            prep_snapshot_store,
            checkpoint_id,
            store,
            handoff,
            None,
        )
    }

    #[cfg(test)]
    pub(crate) fn from_exec_with_verifier(
        dir: impl AsRef<Path>,
        profile: RecursiveCircuitProfileV2,
        checkpoint_store: &impl CheckpointStore,
        prep_snapshot_store: &impl PrepSnapshotStore,
        checkpoint_id: CheckpointId,
        store: &mut SettlementStore,
        handoff: SettlementExecHandoff,
        verifier: RecursiveVerifierAuthorityV2,
    ) -> Result<Self, RecursiveV2Error> {
        Self::from_exec_with_verifier_inner(
            dir,
            profile,
            checkpoint_store,
            prep_snapshot_store,
            checkpoint_id,
            store,
            handoff,
            Some(verifier),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_exec_with_verifier_inner(
        dir: impl AsRef<Path>,
        profile: RecursiveCircuitProfileV2,
        checkpoint_store: &impl CheckpointStore,
        prep_snapshot_store: &impl PrepSnapshotStore,
        checkpoint_id: CheckpointId,
        store: &mut SettlementStore,
        handoff: SettlementExecHandoff,
        verifier: Option<RecursiveVerifierAuthorityV2>,
    ) -> Result<Self, RecursiveV2Error> {
        let segment_dir = dir.as_ref().to_path_buf();
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
        let segment_context = JmtTraceSegmentContextV2::new(
            checkpoint.checkpoint_id().into_bytes(),
            checkpoint.height(),
        );
        if context.policy_digest() != store.bucket_policy().bucket_policy_id() {
            return Err(RecursiveV2Error::CutoverAuthority);
        }
        let pre_root = store
            .settlement_root_v2(context.layout())
            .map_err(|_| RecursiveV2Error::Root)?;
        let captured_snapshot =
            RecursiveSnapshotHandleV2::from_store(snapshot.snapshot_id(), store, context.layout())?;
        if snapshot.root() != pre_root
            || snapshot.pre_definition_root() != store.recursive_v2_definition_root()
            || checkpoint.pre_settlement_root() != pre_root
            || snapshot != captured_snapshot
        {
            return Err(RecursiveV2Error::SnapshotChanged);
        }

        // All fallible witness construction is completed against an isolated
        // exact clone before the live HJMT can advance. Once the real commit
        // succeeds, this method only compares deterministic commitments and
        // returns the already-sealed source; it never needs to create, append,
        // or seal a spool after durable state has moved.
        let mut preflight_store = store.recursive_v2_preflight_clone();
        let (preflight_flow, preflight_update_trace) = preflight_store
            .apply_exec_handoff_v2(handoff.clone(), &segment_dir, segment_context)
            .map_err(|_| RecursiveV2Error::Storage)?;
        let preflight_post_settlement_root = preflight_store
            .settlement_root_v2(context.layout())
            .map_err(|_| RecursiveV2Error::Root)?;
        let preflight_definition_root = preflight_store.recursive_v2_definition_root();
        if preflight_post_settlement_root != checkpoint.post_settlement_root() {
            return Err(RecursiveV2Error::Root);
        }
        let declared_work = plan_declared_work(
            &profile,
            &preflight_flow,
            checkpoint_typed_commitments(checkpoint),
            preflight_definition_root,
            pre_root,
            preflight_post_settlement_root,
            &preflight_update_trace,
            checkpoint.is_recursive_v2_noop(),
        )?;
        let spec = RecursiveCircuitSpecV2::new(context.layout(), &profile)?;
        let verifier = match verifier {
            Some(verifier)
                if verifier.predicate_digest() == super::nova::executable_predicate_digest()? =>
            {
                verifier
            }
            Some(_) => return Err(RecursiveV2Error::Authority),
            None => RecursiveVerifierAuthorityV2::repository_fixture(context, &profile, &spec)?,
        };
        let pre_uniqueness_context = RecursivePreUniquenessContextV2::build(
            context,
            snapshot,
            checkpoint,
            &profile,
            verifier,
            declared_work,
            preflight_update_trace.trace_digest(),
        )?;
        let mut source =
            RecursiveTransitionTraceSourceV2::create_in(&segment_dir, profile, snapshot)?;
        source.bind_pre_uniqueness_context(pre_uniqueness_context)?;
        source.begin_canonical_precommit()?;
        canonical_events(
            &profile,
            &preflight_flow,
            checkpoint_typed_commitments(checkpoint),
            pre_uniqueness_context.digest(),
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
            .apply_exec_handoff_v2(handoff, &segment_dir, segment_context)
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
        self.update_trace.update_count()
    }

    pub(crate) const fn recursive_authority_context(&self) -> RecursiveAuthorityContextV2 {
        self.authority.authority()
    }

    pub(crate) const fn recursive_checkpoint_binding(&self) -> RecursiveCheckpointBindingV2 {
        self.checkpoint
    }

    pub(crate) fn recursive_pre_uniqueness_context(
        &self,
    ) -> Result<RecursivePreUniquenessContextV2, RecursiveV2Error> {
        self.source
            .pre_uniqueness_context()
            .ok_or(RecursiveV2Error::Authority)
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

    #[cfg(test)]
    pub(crate) fn evaluate_with_nova_events(
        &mut self,
        store: &SettlementStore,
    ) -> Result<(EvaluatedCheckpointTransitionV2, Vec<RecursiveTraceEventV2>), RecursiveV2Error>
    {
        self.authority.revalidate_config()?;
        if store
            .settlement_root_v2(self.authority.authority().layout())
            .map_err(|_| RecursiveV2Error::Root)?
            != self.post_settlement_root
            || store.recursive_v2_storage_generation() != self.post_storage_generation
        {
            return Err(RecursiveV2Error::SnapshotChanged);
        }
        let (evaluated, events) = CheckpointTransitionConsistencyV2::evaluate_stream_with_events(
            &mut self.source,
            self.authority.authority(),
            self.checkpoint,
            store,
        )?;
        if evaluated.settlement_root() != self.post_settlement_root {
            return Err(RecursiveV2Error::Root);
        }
        Ok((evaluated, events))
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

#[allow(clippy::too_many_arguments)]
fn plan_declared_work(
    profile: &RecursiveCircuitProfileV2,
    flow: &ScopeFlow,
    typed_commitments: [[u8; 32]; 4],
    post_definition_root: [u8; 32],
    pre_settlement_root: SettlementStateRoot,
    post_settlement_root: SettlementStateRoot,
    update_trace: &SettlementUpdateTraceEnvelopeV2,
    authority_noop: bool,
) -> Result<RecursiveDeclaredWorkV2, RecursiveV2Error> {
    let mut counts = RecursiveTraceEventCountsV2::default();
    let mut source_bytes = 0_u64;
    let mut source_records = 0_u64;
    canonical_events(
        profile,
        flow,
        typed_commitments,
        [1; 32],
        post_definition_root,
        pre_settlement_root,
        post_settlement_root,
        update_trace,
        authority_noop,
        |event| {
            counts.increment(event.opcode())?;
            counts.add(RecursiveTraceOpcodeV2::BeginHash, 1)?;
            counts.add(RecursiveTraceOpcodeV2::EndHash, 1)?;
            counts.add(RecursiveTraceOpcodeV2::ShaBlock, event.hash_geometry()?.1)?;
            let chunks = u64::from(event.canonical_chunk_count()?);
            counts.add(RecursiveTraceOpcodeV2::SourceMemoryWrite, chunks)?;
            counts.add(RecursiveTraceOpcodeV2::TraceChunk, chunks)?;
            source_records = source_records
                .checked_add(1)
                .ok_or(RecursiveV2Error::Overflow)?;
            source_bytes = source_bytes
                .checked_add(event.canonical_len()?)
                .ok_or(RecursiveV2Error::Overflow)?;
            Ok(())
        },
    )?;

    let hash_blocks = |role, part_bytes, part_count| {
        let message_bytes =
            CheckpointSha256BlockStreamV2::framed_bytes_for_parts(role, part_bytes, part_count)?;
        CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(message_bytes)
            .map_err(RecursiveV2Error::from)
    };
    counts.add(RecursiveTraceOpcodeV2::BeginHash, 1)?;
    counts.add(RecursiveTraceOpcodeV2::EndHash, 1)?;
    counts.add(
        RecursiveTraceOpcodeV2::ShaBlock,
        hash_blocks(CheckpointShaRole::Trace, source_bytes, source_records)?,
    )?;

    let input_count = u64::try_from(
        flow.items
            .iter()
            .filter(|item| item.op_kind == ScopeOpKind::Delete)
            .count(),
    )
    .map_err(|_| RecursiveV2Error::Limit)?;
    let output_count = u64::try_from(
        flow.items
            .iter()
            .filter(|item| item.op_kind == ScopeOpKind::Put)
            .count(),
    )
    .map_err(|_| RecursiveV2Error::Limit)?;
    for (role, count) in [
        (CheckpointShaRole::SpentOriginalIds, input_count),
        (CheckpointShaRole::OutputOriginalIds, output_count),
        (CheckpointShaRole::SpentSortedIds, input_count),
        (CheckpointShaRole::OutputSortedIds, output_count),
    ] {
        counts.add(RecursiveTraceOpcodeV2::BeginHash, 1)?;
        counts.add(RecursiveTraceOpcodeV2::EndHash, 1)?;
        counts.add(
            RecursiveTraceOpcodeV2::ShaBlock,
            hash_blocks(
                role,
                count
                    .checked_mul(
                        u64::try_from(UNIQUENESS_SEMANTIC_ROW_BYTES_V2)
                            .map_err(|_| RecursiveV2Error::Limit)?,
                    )
                    .and_then(|value| value.checked_add(4))
                    .ok_or(RecursiveV2Error::Overflow)?,
                count.checked_add(1).ok_or(RecursiveV2Error::Overflow)?,
            )?,
        )?;
    }

    for (role, part_bytes, part_count, jobs) in [
        (
            CheckpointShaRole::UniquenessCounts,
            1 + 8 * 8 + 17 * 8,
            10,
            1,
        ),
        (
            CheckpointShaRole::UniquenessContext,
            1 + 2 * 8 + 4 + 8 + 1 + 14 * 32,
            20,
            1,
        ),
        (CheckpointShaRole::IdPrecommit, 32 + 1 + 4 + 32 + 32, 5, 2),
        (CheckpointShaRole::IdChallenge, 32 + 32 + 1 + 1 + 1, 5, 8),
        (CheckpointShaRole::SettlementRoot, 1 + 4 + 32 + 32, 4, 2),
    ] {
        counts.add(RecursiveTraceOpcodeV2::BeginHash, jobs)?;
        counts.add(RecursiveTraceOpcodeV2::EndHash, jobs)?;
        counts.add(
            RecursiveTraceOpcodeV2::ShaBlock,
            hash_blocks(role, part_bytes, part_count)?
                .checked_mul(jobs)
                .ok_or(RecursiveV2Error::Overflow)?,
        )?;
    }

    let row_count = input_count
        .checked_add(output_count)
        .ok_or(RecursiveV2Error::Overflow)?;
    let net_effect_count = u64::try_from(
        flow.items
            .iter()
            .map(crate::checkpoint::recursive_semantics::UniquenessSemanticRowV2::from_flow_item)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|row| row.terminal_id)
            .collect::<BTreeSet<_>>()
            .len(),
    )
    .map_err(|_| RecursiveV2Error::Limit)?;
    let jmt_update_count = u64::from(update_trace.update_count());
    let net_mutation_count = update_trace
        .terminal_operation_count()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    RecursiveDeclaredWorkV2::new(
        counts,
        row_count,
        input_count,
        output_count,
        net_effect_count,
        net_mutation_count,
        jmt_update_count,
        counts.count(RecursiveTraceOpcodeV2::ShaBlock),
        counts.total_count()?,
    )
}

fn canonical_events(
    profile: &RecursiveCircuitProfileV2,
    flow: &ScopeFlow,
    typed_commitments: [[u8; 32]; 4],
    pre_uniqueness_digest: [u8; 32],
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

    let uniqueness_precommit = encode_uniqueness_precommit(flow)?;
    let uniqueness = decode_uniqueness_precommit(&uniqueness_precommit)?;
    emit(structural_event(
        ordinal,
        RecursiveTraceOpcodeV2::UniquenessPrecommit,
        uniqueness_precommit,
        profile,
    )?)?;
    ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;

    // The commitment is fixed before replay, so every replay row can be
    // followed immediately by its one Original commitment row.  That
    // adjacency is the O(1) circuit binding from the authenticated replay
    // object ID to the later uniqueness/net stream; no random-access row map
    // or endpoint-free Merkle memory is required.
    for item in ordered_flow_items(flow) {
        let (opcode, set) = match item.op_kind {
            ScopeOpKind::Delete => (
                RecursiveTraceOpcodeV2::ReplayInput,
                UniquenessSetKindV2::Spent,
            ),
            ScopeOpKind::Put => (
                RecursiveTraceOpcodeV2::ReplayOutput,
                UniquenessSetKindV2::Output,
            ),
        };
        let semantic_row =
            crate::checkpoint::recursive_semantics::UniquenessSemanticRowV2::from_flow_item(item)?;
        let object_id = semantic_row.terminal_id;
        emit(RecursiveTraceEventV2::new(
            ordinal,
            opcode,
            object_id,
            encode_flow_item(item)?,
            profile,
        )?)?;
        ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;
        emit_uniqueness_row(
            profile,
            UniquenessPassV2::Commit,
            set,
            UniquenessListKindV2::Original,
            semantic_row,
            &mut ordinal,
            &mut emit,
        )?;
    }

    // Finish the first pass with the two sorted streams. Together with the
    // adjacent Original rows above, all four commitments are reconstructed
    // before any challenge bytes enter the source grammar.
    emit_uniqueness_rows(
        profile,
        flow,
        UniquenessPassV2::Commit,
        &mut ordinal,
        &mut emit,
    )?;

    let grammar_digest = RecursiveTraceOpcodeV2::grammar_digest();
    let uniqueness_challenge =
        encode_uniqueness_challenge(pre_uniqueness_digest, grammar_digest, uniqueness);
    let challenge = decode_uniqueness_challenge(
        &uniqueness_challenge,
        pre_uniqueness_digest,
        grammar_digest,
        uniqueness,
    )?;
    emit(structural_event(
        ordinal,
        RecursiveTraceOpcodeV2::UniquenessChallenge,
        uniqueness_challenge,
        profile,
    )?)?;
    ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;

    // Second pass: evaluate products only after the precommit-derived challenge
    // transcript. The codec discriminator prevents a commitment row from being
    // replayed as a product row (or vice versa).
    emit_uniqueness_rows(
        profile,
        flow,
        UniquenessPassV2::Product,
        &mut ordinal,
        &mut emit,
    )?;

    emit(structural_event(
        ordinal,
        RecursiveTraceOpcodeV2::NetMerge,
        encode_net_merge(uniqueness, challenge),
        profile,
    )?)?;
    ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;

    let envelope_header = update_trace
        .circuit_header_bytes()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    emit(structural_event(
        ordinal,
        RecursiveTraceOpcodeV2::JmtUpdate,
        envelope_header.to_vec(),
        profile,
    )?)?;
    ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;

    // The recursive source never carries the opaque pinned-JMT bincode blob.
    // Its sole live representation is the bounded circuit micro-operation
    // transcript derived by the storage proof owner. Each record is emitted
    // and dropped immediately, preserving the one-spool/O(1)-record contract.
    let mut emit_error = None;
    update_trace
        .visit_circuit_micro_operations(|record| {
            if emit_error.is_some() {
                return Ok(());
            }
            match structural_event(
                ordinal,
                RecursiveTraceOpcodeV2::JmtMicroOp,
                record.to_vec(),
                profile,
            )
            .and_then(&mut emit)
            {
                Ok(()) => match ordinal.checked_add(1) {
                    Some(next) => ordinal = next,
                    None => emit_error = Some(RecursiveV2Error::Overflow),
                },
                Err(error) => emit_error = Some(error),
            }
            Ok(())
        })
        .map_err(|_| RecursiveV2Error::Canonical)?;
    if let Some(error) = emit_error {
        return Err(error);
    }
    emit(structural_event(
        ordinal,
        RecursiveTraceOpcodeV2::PromoteChildRoot,
        encode_hierarchy_promotion(post_definition_root, update_trace.trace_digest()),
        profile,
    )?)?;
    ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;
    // ReplayInput/ReplayOutput are the sole canonical flow-item records.  A
    // former second copy under CommitTypedEvent forced the theorem to prove
    // equality between two byte transcripts and admitted needless parser and
    // hash state.  CommitTypedEvent now has exactly one meaning: the four
    // fixed checkpoint-core commitments below.
    for (kind, digest) in TypedCheckpointCommitmentKindV2::ALL
        .into_iter()
        .zip(typed_commitments)
    {
        emit(structural_event(
            ordinal,
            RecursiveTraceOpcodeV2::CommitTypedEvent,
            encode_typed_checkpoint_commitment(kind, digest),
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

fn checkpoint_typed_commitments(checkpoint: RecursiveCheckpointBindingV2) -> [[u8; 32]; 4] {
    [
        checkpoint.delta_root(),
        checkpoint.witness_root(),
        checkpoint.journal_digest(),
        checkpoint.checkpoint_link_digest(),
    ]
}

/// Emit one exact original-then-sorted uniqueness pass. Both pass instances
/// call this owner, so commit/product cannot drift in set order, ID decoding,
/// sort order, or row framing.
fn emit_uniqueness_rows(
    profile: &RecursiveCircuitProfileV2,
    flow: &ScopeFlow,
    pass: UniquenessPassV2,
    ordinal: &mut u64,
    emit: &mut impl FnMut(RecursiveTraceEventV2) -> Result<(), RecursiveV2Error>,
) -> Result<(), RecursiveV2Error> {
    if pass == UniquenessPassV2::Product {
        for (kind, set) in [
            (ScopeOpKind::Delete, UniquenessSetKindV2::Spent),
            (ScopeOpKind::Put, UniquenessSetKindV2::Output),
        ] {
            for row in flow.items.iter().filter(|item| item.op_kind == kind).map(
                crate::checkpoint::recursive_semantics::UniquenessSemanticRowV2::from_flow_item,
            ) {
                emit_uniqueness_row(
                    profile,
                    pass,
                    set,
                    UniquenessListKindV2::Original,
                    row?,
                    ordinal,
                    emit,
                )?;
            }
        }
    }

    let set_order: &[UniquenessSetKindV2] = if pass == UniquenessPassV2::Commit {
        &[UniquenessSetKindV2::Spent, UniquenessSetKindV2::Output]
    } else {
        &[]
    };
    if set_order.is_empty() {
        let mut sorted_rows = flow
            .items
            .iter()
            .map(|item| {
                let set = match item.op_kind {
                    ScopeOpKind::Delete => UniquenessSetKindV2::Spent,
                    ScopeOpKind::Put => UniquenessSetKindV2::Output,
                };
                crate::checkpoint::recursive_semantics::UniquenessSemanticRowV2::from_flow_item(
                    item,
                )
                .map(|row| (row, set))
            })
            .collect::<Result<Vec<_>, _>>()?;
        sorted_rows.sort_unstable_by_key(|(row, set)| (row.terminal_id, *set as u8));
        let mut index = 0_usize;
        while index < sorted_rows.len() {
            let (row, set) = sorted_rows[index];
            emit_uniqueness_row(
                profile,
                pass,
                set,
                UniquenessListKindV2::Sorted,
                row,
                ordinal,
                emit,
            )?;
            let next_is_same_terminal = sorted_rows
                .get(index + 1)
                .is_some_and(|(next, _)| next.terminal_id == row.terminal_id);
            let effect = if next_is_same_terminal {
                let (next, next_set) = sorted_rows[index + 1];
                if set != UniquenessSetKindV2::Spent || next_set != UniquenessSetKindV2::Output {
                    return Err(RecursiveV2Error::Invariant);
                }
                emit_uniqueness_row(
                    profile,
                    pass,
                    next_set,
                    UniquenessListKindV2::Sorted,
                    next,
                    ordinal,
                    emit,
                )?;
                index += 2;
                NetEffectV2::from_rows(Some(row), Some(next))?
            } else {
                index += 1;
                match set {
                    UniquenessSetKindV2::Spent => NetEffectV2::from_rows(Some(row), None)?,
                    UniquenessSetKindV2::Output => NetEffectV2::from_rows(None, Some(row))?,
                }
            };
            emit(structural_event(
                *ordinal,
                RecursiveTraceOpcodeV2::NetMerge,
                encode_net_effect(effect),
                profile,
            )?)?;
            *ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;
        }
    } else {
        for set in set_order {
            let kind = match set {
                UniquenessSetKindV2::Spent => ScopeOpKind::Delete,
                UniquenessSetKindV2::Output => ScopeOpKind::Put,
            };
            let mut sorted_rows = flow
                .items
                .iter()
                .filter(|item| item.op_kind == kind)
                .map(
                    crate::checkpoint::recursive_semantics::UniquenessSemanticRowV2::from_flow_item,
                )
                .collect::<Result<Vec<_>, _>>()?;
            sorted_rows.sort_unstable_by_key(|row| row.terminal_id);
            for row in sorted_rows {
                emit_uniqueness_row(
                    profile,
                    pass,
                    *set,
                    UniquenessListKindV2::Sorted,
                    row,
                    ordinal,
                    emit,
                )?;
            }
        }
    }
    Ok(())
}

fn emit_uniqueness_row(
    profile: &RecursiveCircuitProfileV2,
    pass: UniquenessPassV2,
    set: UniquenessSetKindV2,
    list: UniquenessListKindV2,
    row: crate::checkpoint::recursive_semantics::UniquenessSemanticRowV2,
    ordinal: &mut u64,
    emit: &mut impl FnMut(RecursiveTraceEventV2) -> Result<(), RecursiveV2Error>,
) -> Result<(), RecursiveV2Error> {
    emit(structural_event(
        *ordinal,
        RecursiveTraceOpcodeV2::UniquenessSorted,
        encode_uniqueness_sorted_row(pass, set, list, row),
        profile,
    )?)?;
    *ordinal = ordinal.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;
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

#[cfg(test)]
mod tests {
    use z00z_core::assets::{AssetLeaf, AssetPackPlain};
    use z00z_crypto::ZkPackEncrypted;

    use super::{
        canonical_events, opaque_record_digest, CanonicalCheckpointTransitionV2,
        SettlementRootGenerationCutoverV2,
    };
    use crate::{
        checkpoint::{
            recursive_circuit::RecursiveCircuitProfileV2,
            recursive_predicate::CheckpointTransitionConsistencyV2,
            recursive_semantics::{
                decode_typed_checkpoint_commitment, TypedCheckpointCommitmentKindV2,
            },
            recursive_trace::{
                structural_event_id, RecursiveTraceEventV2, RecursiveTraceOpcodeV2,
                RecursiveTransitionTraceSourceV2,
            },
            CheckpointDraft, CheckpointExecInput, CheckpointExecOut, CheckpointExecTx,
            CheckpointExecVersion, CheckpointFsStore, CheckpointId, CheckpointInRef,
            CheckpointStore, CheckpointVersion, CreatedEnt, SpentEnt,
        },
        fixture_support::{
            checkpoint_fixtures,
            settlement_corpus::{asset_item, load_fixture},
        },
        settlement::{
            hjmt_config::SettlementBackendMode, DefinitionId, SerialId, SettlementExecHandoff,
            SettlementPath, SettlementRouteCtx, SettlementStateRoot, SettlementStore, StoreItem,
            StoreOp, TerminalId, TerminalLeaf,
        },
        snapshot::{build_snapshot_v2, PrepFsStore, PrepSnapshotStore},
    };

    fn profile() -> RecursiveCircuitProfileV2 {
        RecursiveCircuitProfileV2::repository_fixture()
    }

    #[test]
    fn durable_cutover_generation_cas_failure_is_atomic() {
        let temp = tempfile::TempDir::new().expect("durable CAS directory");
        let mut store =
            SettlementStore::load_with_backend_mode(temp.path(), SettlementBackendMode::Hjmt)
                .expect("durable CAS fixture");
        let fixture = load_fixture();
        store
            .put_settlement_item(asset_item(&fixture.assets[0]))
            .expect("persist CAS pre-state");
        let authority =
            super::RecursiveAuthoritySnapshotV2::resolve_repository_local_fixture(&store)
                .expect("CAS authority snapshot");
        let expected = store.settlement_root_v2(7).expect("CAS V2 root");
        let opaque = [8; 32];
        let stale = SettlementRootGenerationCutoverV2::repository_local_fixture(
            authority,
            &store,
            10,
            opaque,
            opaque_record_digest(opaque),
            expected,
            11,
        )
        .expect("stale CAS manifest");
        let stale_manifest = stale.durable_manifest();

        store
            .put_settlement_item(asset_item(&fixture.assets[1]))
            .expect("advance durable generation before backend CAS");
        let error = store
            .backend
            .install_recursive_v2_cutover(&stale_manifest)
            .expect_err("backend must reject the stale generation");
        assert!(
            error.to_string().contains("compare-and-swap mismatch"),
            "the rejection must reach the durable backend CAS: {error}",
        );
        assert!(
            store
                .backend
                .load_recursive_v2_cutover()
                .expect("read back failed CAS")
                .is_none(),
            "a failed generation CAS must leave no partial manifest",
        );

        let current_authority =
            super::RecursiveAuthoritySnapshotV2::resolve_repository_local_fixture(&store)
                .expect("current CAS authority snapshot");
        let current_expected = store.settlement_root_v2(7).expect("current CAS V2 root");
        let mut current = SettlementRootGenerationCutoverV2::repository_local_fixture(
            current_authority,
            &store,
            10,
            opaque,
            opaque_record_digest(opaque),
            current_expected,
            11,
        )
        .expect("current CAS manifest");
        current
            .install_repository_fixture(&mut store, 11)
            .expect("failed stale CAS leaves the current install available");
    }

    fn path(definition: u8, serial: u32, terminal: u8) -> SettlementPath {
        SettlementPath::new(
            DefinitionId::new([definition; 32]),
            SerialId::new(serial),
            TerminalId::new([terminal; 32]),
        )
    }

    fn item(path: SettlementPath, value: u64) -> StoreItem {
        let payload = AssetPackPlain {
            value,
            blinding: [3; 32],
            s_out: [4; 32],
        }
        .to_bytes();
        let leaf = AssetLeaf {
            asset_id: path.terminal_id().into_bytes(),
            serial_id: path.serial_id.get(),
            r_pub: [1; 32],
            owner_tag: [2; 32],
            c_amount: [5; 32],
            enc_pack: ZkPackEncrypted {
                version: 1,
                ciphertext: payload,
                tag: [0; 16],
            },
            range_proof: vec![9; 4],
            tag16: 11,
        };
        StoreItem::new(path, TerminalLeaf::from(leaf)).expect("terminal item")
    }

    fn handoff(input: SettlementPath, output: StoreItem) -> SettlementExecHandoff {
        let tx = CheckpointExecTx::new(
            vec![CheckpointInRef::new(input.terminal_id(), input.serial_id)],
            vec![CheckpointExecOut::new(
                output.path().definition_id,
                output.terminal_leaf().expect("terminal leaf").clone(),
            )
            .expect("canonical output")],
            vec![8],
        )
        .expect("canonical transaction row");
        SettlementExecHandoff::new(
            SettlementRouteCtx::new([9; 32], 1, 1, [10; 32]),
            vec![StoreOp::Delete(input), StoreOp::Put(Box::new(output))],
            vec![tx],
        )
    }

    fn canonical_checkpoint(
        root: &std::path::Path,
        pre_settlement_root: SettlementStateRoot,
        post_settlement_root: SettlementStateRoot,
        handoff: &SettlementExecHandoff,
    ) -> (CheckpointFsStore, PrepFsStore, CheckpointId) {
        let draft = CheckpointDraft::new_settlement(
            CheckpointVersion::CURRENT,
            1,
            pre_settlement_root,
            post_settlement_root,
            vec![SpentEnt::new([0x51; 32])],
            vec![CreatedEnt::new([0x52; 32], [0x53; 32])],
        );
        let (snapshot, snapshot_id) =
            build_snapshot_v2(pre_settlement_root, Vec::new()).expect("prep snapshot");
        let mut prep_store = PrepFsStore::new(root);
        assert_eq!(
            prep_store
                .save_snapshot(&snapshot)
                .expect("persist prep snapshot"),
            snapshot_id
        );
        let exec = CheckpointExecInput::new_settlement(
            CheckpointExecVersion::CURRENT,
            snapshot_id,
            pre_settlement_root,
            handoff.txs().to_vec(),
        )
        .expect("canonical execution input");
        let mut checkpoint_store = CheckpointFsStore::new(root);
        let exec_id = checkpoint_store
            .save_exec_input(&exec)
            .expect("persist execution input");
        let manifest = checkpoint_fixtures::archive_manifest(&draft, &exec, exec_id);
        let da_reference = checkpoint_fixtures::da_reference(&manifest);
        let statement_core = checkpoint_fixtures::statement_core(&exec);
        checkpoint_store
            .stage_publication_contract(exec_id, &statement_core, &manifest, &da_reference)
            .expect("stage canonical checkpoint evidence");
        let link = checkpoint_store
            .seal_artifact(
                &draft,
                draft
                    .attest_proof(snapshot_id, exec_id)
                    .expect("attested checkpoint proof"),
                snapshot_id,
                exec_id,
            )
            .expect("persist canonical checkpoint artifact and link");
        (checkpoint_store, prep_store, link.checkpoint_id())
    }

    #[test]
    fn evaluator_rejects_a_real_jmt_envelope_from_a_converging_pre_state() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let input = path(1, 1, 1);
        let output = item(path(2, 2, 2), 20);
        let handoff = handoff(input, output);

        // The old terminal leaf differs, but both executions delete it and
        // create the same output. Their post-states therefore converge while
        // their definition and settlement pre-roots remain distinct.
        let mut store_a = SettlementStore::new();
        store_a
            .put_settlement_item(item(input, 10))
            .expect("seed pre-state A");
        let mut store_b = SettlementStore::new();
        store_b
            .put_settlement_item(item(input, 11))
            .expect("seed pre-state B");
        let pre_root_a = store_a.settlement_root_v2(7).expect("V2 pre-root A");
        let pre_root_b = store_b.settlement_root_v2(7).expect("V2 pre-root B");
        assert_ne!(pre_root_a, pre_root_b);

        let mut post_a = store_a.recursive_v2_preflight_clone();
        let (_, _) = post_a
            .apply_exec_handoff_v2_for_test(handoff.clone(), temp.path())
            .expect("converging A transition");
        let post_root = post_a.settlement_root_v2(7).expect("shared V2 post-root");
        let mut post_b = store_b.recursive_v2_preflight_clone();
        let (flow_b, envelope_b) = post_b
            .apply_exec_handoff_v2_for_test(handoff.clone(), temp.path())
            .expect("converging B transition");
        assert_eq!(
            post_b.settlement_root_v2(7).expect("shared V2 post-root"),
            post_root
        );

        let (checkpoint_store, prep_snapshot_store, checkpoint_id) =
            canonical_checkpoint(temp.path(), pre_root_a, post_root, &handoff);
        let transition_a = CanonicalCheckpointTransitionV2::from_exec(
            temp.path(),
            profile(),
            &checkpoint_store,
            &prep_snapshot_store,
            checkpoint_id,
            &mut store_a,
            handoff,
        )
        .expect("valid A transition supplies only the immutable A capability");

        let source_profile = profile();
        let mut substituted_source = RecursiveTransitionTraceSourceV2::create_in(
            temp.path(),
            source_profile,
            transition_a.authority.snapshot(),
        )
        .expect("isolated substituted source");
        let pre_uniqueness_context = transition_a
            .source
            .pre_uniqueness_context()
            .expect("canonical transition binds its pre-uniqueness context");
        substituted_source
            .bind_pre_uniqueness_context(pre_uniqueness_context)
            .expect("bind the same immutable pre-uniqueness context");
        substituted_source
            .begin_canonical_precommit()
            .expect("begin source precommit");
        canonical_events(
            &source_profile,
            &flow_b,
            super::checkpoint_typed_commitments(transition_a.checkpoint),
            pre_uniqueness_context.digest(),
            post_b.recursive_v2_definition_root(),
            pre_root_a,
            post_root,
            &envelope_b,
            transition_a.checkpoint.is_recursive_v2_noop(),
            |event| substituted_source.append_canonical_event(event),
        )
        .expect("recompute all source/hash controls around the substituted envelope");
        substituted_source
            .seal_canonical_precommit()
            .expect("seal substituted source");

        let rejected = CheckpointTransitionConsistencyV2::evaluate_stream(
            &mut substituted_source,
            transition_a.authority.authority(),
            transition_a.checkpoint,
            &post_b,
        );
        assert!(
            matches!(
                rejected,
                Err(crate::checkpoint::recursive_reject::RecursiveV2Error::Root)
            ),
            "the first old-root gate must reject B's real envelope under A's immutable pre-state"
        );
    }

    #[test]
    fn evaluator_rejects_a_recommitted_sorted_identifier_substitution() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let input = path(1, 1, 1);
        let output = item(path(2, 2, 2), 20);
        let handoff = handoff(input, output);
        let mut store = SettlementStore::new();
        store
            .put_settlement_item(item(input, 10))
            .expect("seed pre-state");
        let pre_root = store.settlement_root_v2(7).expect("V2 pre-root");

        let mut post = store.recursive_v2_preflight_clone();
        let (flow, envelope) = post
            .apply_exec_handoff_v2_for_test(handoff.clone(), temp.path())
            .expect("preflight transition");
        let post_root = post.settlement_root_v2(7).expect("V2 post-root");
        let post_definition_root = post.recursive_v2_definition_root();
        let (checkpoint_store, prep_snapshot_store, checkpoint_id) =
            canonical_checkpoint(temp.path(), pre_root, post_root, &handoff);
        let transition = CanonicalCheckpointTransitionV2::from_exec(
            temp.path(),
            profile(),
            &checkpoint_store,
            &prep_snapshot_store,
            checkpoint_id,
            &mut store,
            handoff,
        )
        .expect("valid canonical transition");

        let source_profile = profile();
        let mut substituted_source = RecursiveTransitionTraceSourceV2::create_in(
            temp.path(),
            source_profile,
            transition.authority.snapshot(),
        )
        .expect("isolated substituted source");
        let pre_uniqueness_context = transition
            .source
            .pre_uniqueness_context()
            .expect("canonical transition binds its pre-uniqueness context");
        substituted_source
            .bind_pre_uniqueness_context(pre_uniqueness_context)
            .expect("bind the same immutable pre-uniqueness context");
        substituted_source
            .begin_canonical_precommit()
            .expect("begin source precommit");
        let mut substituted = false;
        let mut replay_count = 0_usize;
        let mut commit_kinds = Vec::new();
        canonical_events(
            &source_profile,
            &flow,
            super::checkpoint_typed_commitments(transition.checkpoint),
            pre_uniqueness_context.digest(),
            post_definition_root,
            pre_root,
            post_root,
            &envelope,
            transition.checkpoint.is_recursive_v2_noop(),
            |mut event| {
                if matches!(
                    event.opcode(),
                    RecursiveTraceOpcodeV2::ReplayInput | RecursiveTraceOpcodeV2::ReplayOutput
                ) {
                    replay_count += 1;
                }
                if event.opcode() == RecursiveTraceOpcodeV2::CommitTypedEvent {
                    let (kind, _) = decode_typed_checkpoint_commitment(event.payload())?;
                    commit_kinds.push(kind);
                }
                if !substituted && event.opcode() == RecursiveTraceOpcodeV2::UniquenessSorted {
                    let mut payload = event.payload().to_vec();
                    *payload.last_mut().expect("fixed-width sorted payload") ^= 1;
                    event = RecursiveTraceEventV2::new(
                        event.ordinal(),
                        event.opcode(),
                        structural_event_id(event.opcode(), event.ordinal(), &payload),
                        payload,
                        &source_profile,
                    )?;
                    substituted = true;
                }
                substituted_source.append_canonical_event(event)
            },
        )
        .expect("recommit every source/hash record around the substituted identifier");
        assert_eq!(replay_count, flow.items.len());
        assert_eq!(commit_kinds, TypedCheckpointCommitmentKindV2::ALL);
        assert!(substituted, "fixture must alter one sorted identifier row");
        substituted_source
            .seal_canonical_precommit()
            .expect("seal recomputed source precommit");

        let rejected = CheckpointTransitionConsistencyV2::evaluate_stream(
            &mut substituted_source,
            transition.authority.authority(),
            transition.checkpoint,
            &store,
        );
        assert!(
            matches!(
                rejected,
                Err(crate::checkpoint::recursive_reject::RecursiveV2Error::Invariant)
            ),
            "a source/hash-recommitted sorted-ID substitution must reach the uniqueness relation"
        );
    }
}
