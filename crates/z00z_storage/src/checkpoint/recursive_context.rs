//! Immutable authority and snapshot-handle bindings for recursive checkpoint V2.

use z00z_crypto::{sha256_256, sha256_256_role, CheckpointShaRole};

use crate::{
    checkpoint::{
        check_link_ids, derive_checkpoint_id, derive_exec_tx_root,
        ActiveCheckpointConfigIdentityV3, ActiveCheckpointConfigV3, CheckpointConfigResolverV3,
        CheckpointId, CheckpointStatement, CheckpointStore, CheckpointTransitionStatementFinalV1,
    },
    settlement::{RootGeneration, SettlementExecHandoff, SettlementStateRoot, SettlementStore},
    snapshot::{PrepSnapshotId, PrepSnapshotStore},
    CheckpointError,
};

const RECURSIVE_CHECKPOINT_CONTEXT_DOMAIN_V2: &str = "z00z.storage.checkpoint.recursive_context.v2";
const RECURSIVE_CHECKPOINT_CONTEXT_LABEL_V2: &str = "context_digest";
const RECURSIVE_CHECKPOINT_CONTEXT_VERSION_V2: u16 = 2;
const RECURSIVE_NOOP_EXECUTION_INPUT_VERSION_V2: u8 = 2;

/// Exact portable namespace bound into recursive-checkpoint public inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursiveCheckpointContextV2 {
    version: u16,
    chain_id: u32,
    network_id: [u8; 32],
    genesis_digest: [u8; 32],
    checkpoint_config_digest: [u8; 32],
    predicate_digest: [u8; 32],
}

impl RecursiveCheckpointContextV2 {
    fn from_installed_identity(
        checkpoint_config_digest: [u8; 32],
        predicate_digest: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        let identity = z00z_core::genesis::require_process_chain_identity()
            .map_err(|_| CheckpointError::Authority)?;
        if checkpoint_config_digest == [0; 32] || predicate_digest == [0; 32] {
            return Err(CheckpointError::Authority);
        }
        Ok(Self {
            version: RECURSIVE_CHECKPOINT_CONTEXT_VERSION_V2,
            chain_id: identity.chain_id(),
            network_id: identity.network_id(),
            genesis_digest: identity.genesis_digest(),
            checkpoint_config_digest,
            predicate_digest,
        })
    }

    #[must_use]
    pub const fn version(self) -> u16 {
        self.version
    }

    #[must_use]
    pub const fn chain_id(self) -> u32 {
        self.chain_id
    }

    #[must_use]
    pub const fn network_id(self) -> [u8; 32] {
        self.network_id
    }

    #[must_use]
    pub const fn genesis_digest(self) -> [u8; 32] {
        self.genesis_digest
    }

    #[must_use]
    pub const fn checkpoint_config_digest(self) -> [u8; 32] {
        self.checkpoint_config_digest
    }

    #[must_use]
    pub const fn predicate_digest(self) -> [u8; 32] {
        self.predicate_digest
    }

    #[must_use]
    pub fn canonical_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 + 4 + 32 * 4);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.chain_id.to_le_bytes());
        bytes.extend_from_slice(&self.network_id);
        bytes.extend_from_slice(&self.genesis_digest);
        bytes.extend_from_slice(&self.checkpoint_config_digest);
        bytes.extend_from_slice(&self.predicate_digest);
        bytes
    }

    #[must_use]
    pub fn digest(self) -> [u8; 32] {
        context_digest_fields(
            self.version,
            self.chain_id,
            self.network_id,
            self.genesis_digest,
            self.checkpoint_config_digest,
            self.predicate_digest,
        )
    }

    #[cfg(test)]
    pub(crate) fn test_fixture(
        checkpoint_config_digest: [u8; 32],
        predicate_digest: [u8; 32],
    ) -> Self {
        Self {
            version: RECURSIVE_CHECKPOINT_CONTEXT_VERSION_V2,
            chain_id: 1,
            network_id: [0x51; 32],
            genesis_digest: [0x61; 32],
            checkpoint_config_digest,
            predicate_digest,
        }
    }
}

fn context_digest_fields(
    version: u16,
    chain_id: u32,
    network_id: [u8; 32],
    genesis_digest: [u8; 32],
    checkpoint_config_digest: [u8; 32],
    predicate_digest: [u8; 32],
) -> [u8; 32] {
    sha256_256(
        RECURSIVE_CHECKPOINT_CONTEXT_DOMAIN_V2,
        RECURSIVE_CHECKPOINT_CONTEXT_LABEL_V2,
        &[
            &version.to_le_bytes(),
            &chain_id.to_le_bytes(),
            &network_id,
            &genesis_digest,
            &checkpoint_config_digest,
            &predicate_digest,
        ],
    )
}

/// Fixed layout selected by the active recursive-checkpoint authority.
pub const RECURSIVE_V2_ACTIVE_AUTHORITY_LAYOUT: u32 = 7;
/// Immutable authority context captured before recursive trace construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursiveAuthorityContextV2 {
    checkpoint_context: RecursiveCheckpointContextV2,
    network_context: [u8; 32],
    config_digest: [u8; 32],
    policy_digest: [u8; 32],
    layout: u32,
    authority_generation: u64,
    noop_execution_input_version: u8,
    epoch_cadence_blocks: u64,
}

impl RecursiveAuthorityContextV2 {
    /// Construct one nonzero-generation authority context.
    pub(crate) fn new(
        checkpoint_context: RecursiveCheckpointContextV2,
        config_digest: [u8; 32],
        policy_digest: [u8; 32],
        layout: u32,
        authority_generation: u64,
        epoch_cadence_blocks: u64,
    ) -> Result<Self, CheckpointError> {
        if layout == 0 || authority_generation == 0 || epoch_cadence_blocks == 0 {
            return Err(CheckpointError::Invariant);
        }
        let network_context = checkpoint_context.digest();
        if checkpoint_context.checkpoint_config_digest() != config_digest {
            return Err(CheckpointError::Authority);
        }
        Ok(Self {
            checkpoint_context,
            network_context,
            config_digest,
            policy_digest,
            layout,
            authority_generation,
            noop_execution_input_version: RECURSIVE_NOOP_EXECUTION_INPUT_VERSION_V2,
            epoch_cadence_blocks,
        })
    }

    #[must_use]
    pub const fn network_context(&self) -> [u8; 32] {
        self.network_context
    }

    #[must_use]
    pub const fn checkpoint_context(&self) -> RecursiveCheckpointContextV2 {
        self.checkpoint_context
    }

    #[must_use]
    pub const fn config_digest(&self) -> [u8; 32] {
        self.config_digest
    }

    #[must_use]
    pub const fn policy_digest(&self) -> [u8; 32] {
        self.policy_digest
    }

    #[must_use]
    pub const fn layout(&self) -> u32 {
        self.layout
    }

    #[must_use]
    pub const fn authority_generation(&self) -> u64 {
        self.authority_generation
    }

    #[must_use]
    pub const fn allows_noop_execution_input_version(&self, version: u8) -> bool {
        version == self.noop_execution_input_version
    }

    #[must_use]
    pub const fn noop_execution_input_version(&self) -> u8 {
        self.noop_execution_input_version
    }

    #[must_use]
    pub const fn epoch_cadence_blocks(&self) -> u64 {
        self.epoch_cadence_blocks
    }

    /// Derive the authority identity consumed by every trace pass.
    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        let layout = self.layout.to_le_bytes();
        let generation = self.authority_generation.to_le_bytes();
        let noop_version = [self.noop_execution_input_version];
        let epoch_cadence = self.epoch_cadence_blocks.to_le_bytes();
        sha256_256_role(
            CheckpointShaRole::UniquenessContext,
            &[
                &self.network_context,
                &self.config_digest,
                &self.policy_digest,
                &layout,
                &generation,
                &noop_version,
                &epoch_cadence,
            ],
        )
    }
}

/// The opaque immutable snapshot handle bound to both trace passes.
///
/// Equality includes the external snapshot ID, storage generation, canonical
/// V2 root, immutable pre-definition root, content digest, and exact counts.
/// Equal caller-supplied IDs alone are not sufficient to reopen a source under
/// a different handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursiveSnapshotHandleV2 {
    snapshot_id: PrepSnapshotId,
    storage_generation: u64,
    root: SettlementStateRoot,
    pre_definition_root: [u8; 32],
    record_count: u64,
    byte_count: u64,
    content_digest: [u8; 32],
}

impl RecursiveSnapshotHandleV2 {
    /// Build one V2-only immutable handle after the storage snapshot is frozen.
    pub(crate) fn new(
        snapshot_id: PrepSnapshotId,
        storage_generation: u64,
        root: SettlementStateRoot,
        pre_definition_root: [u8; 32],
        record_count: u64,
        byte_count: u64,
        content_digest: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        if root.generation() != RootGeneration::SettlementV2 {
            return Err(CheckpointError::SnapshotChanged);
        }
        Ok(Self {
            snapshot_id,
            storage_generation,
            root,
            pre_definition_root,
            record_count,
            byte_count,
            content_digest,
        })
    }

    /// Capture every snapshot field from the sole live settlement-store owner.
    pub(crate) fn from_store(
        snapshot_id: PrepSnapshotId,
        store: &SettlementStore,
        layout: u32,
    ) -> Result<Self, CheckpointError> {
        let (
            root,
            pre_definition_root,
            storage_generation,
            record_count,
            byte_count,
            content_digest,
        ) = store
            .recursive_v2_snapshot_binding(layout)
            .map_err(|_| CheckpointError::SnapshotChanged)?;
        Self::new(
            snapshot_id,
            storage_generation,
            SettlementStateRoot::settlement_v2(root),
            pre_definition_root,
            record_count,
            byte_count,
            content_digest,
        )
    }

    #[must_use]
    pub const fn snapshot_id(&self) -> PrepSnapshotId {
        self.snapshot_id
    }

    #[must_use]
    pub const fn storage_generation(&self) -> u64 {
        self.storage_generation
    }

    #[must_use]
    pub const fn root(&self) -> SettlementStateRoot {
        self.root
    }

    /// Return the immutable definition-tree root from which `root` was derived.
    #[must_use]
    pub const fn pre_definition_root(&self) -> [u8; 32] {
        self.pre_definition_root
    }

    #[must_use]
    pub const fn record_count(&self) -> u64 {
        self.record_count
    }

    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    #[must_use]
    pub const fn content_digest(&self) -> [u8; 32] {
        self.content_digest
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        let generation = self.storage_generation.to_le_bytes();
        let records = self.record_count.to_le_bytes();
        let bytes = self.byte_count.to_le_bytes();
        sha256_256_role(
            CheckpointShaRole::Content,
            &[
                self.snapshot_id.as_bytes(),
                &generation,
                self.root.as_bytes(),
                &self.pre_definition_root,
                &records,
                &bytes,
                &self.content_digest,
            ],
        )
    }
}

/// Canonical checkpoint lineage resolved from the existing storage owners.
///
/// This is deliberately crate-private: a caller cannot supply a height,
/// predecessor, statement digest, or checkpoint link beside a V2 transition.
/// The resolver reloads the artifact, link, and execution input under their
/// canonical IDs, then carries only their immutable typed bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RecursiveCheckpointBindingV2 {
    checkpoint_id: CheckpointId,
    predecessor: Option<CheckpointId>,
    height: u64,
    prep_snapshot_id: PrepSnapshotId,
    exec_tx_root: [u8; 32],
    exec_tx_count: u32,
    exec_is_recursive_v2_noop: bool,
    exec_version: u8,
    statement_digest: [u8; 32],
    statement_core_digest: [u8; 32],
    delta_root: [u8; 32],
    witness_root: [u8; 32],
    journal_digest: [u8; 32],
    prior_recursive_output_root: Option<[u8; 32]>,
    checkpoint_link_digest: [u8; 32],
    pre_settlement_root: SettlementStateRoot,
    post_settlement_root: SettlementStateRoot,
}

impl RecursiveCheckpointBindingV2 {
    /// Reload and cross-check one canonical checkpoint relation.  The backing
    /// store remains the sole source of its statement, link, replay input, and
    /// predecessor; no raw component is accepted by the recursive surface.
    pub(crate) fn resolve(
        checkpoint_store: &(impl CheckpointStore + ?Sized),
        prep_snapshot_store: &(impl PrepSnapshotStore + ?Sized),
        checkpoint_id: CheckpointId,
    ) -> Result<Self, CheckpointError> {
        let artifact = checkpoint_store
            .load_artifact(&checkpoint_id)
            .map_err(|_| CheckpointError::Authority)?;
        if derive_checkpoint_id(&artifact).map_err(|_| CheckpointError::Authority)? != checkpoint_id
        {
            return Err(CheckpointError::Authority);
        }
        let link = checkpoint_store
            .load_link(&checkpoint_id)
            .map_err(|_| CheckpointError::Authority)?;
        if link.checkpoint_id() != checkpoint_id {
            return Err(CheckpointError::Authority);
        }
        let exec = checkpoint_store
            .load_exec_input(&link.exec_input_id())
            .map_err(|_| CheckpointError::Authority)?;
        check_link_ids(link.prep_snapshot_id(), &link, &exec)
            .map_err(|_| CheckpointError::Authority)?;
        let prep_snapshot = prep_snapshot_store
            .load_snapshot(&link.prep_snapshot_id())
            .map_err(|_| CheckpointError::Authority)?;

        let statement = match artifact.statement() {
            CheckpointStatement::V1(statement) => statement,
            CheckpointStatement::Detached => return Err(CheckpointError::Authority),
        };
        let exec_tx_count = u32::try_from(exec.txs().len()).map_err(|_| CheckpointError::Limit)?;
        let exec_tx_root = exec
            .expected_tx_data_root()
            .map_err(|_| CheckpointError::Authority)?;
        let exec_is_recursive_v2_noop = exec.is_recursive_v2_noop();
        if statement.prep_snapshot_id() != link.prep_snapshot_id()
            || statement.exec_input_id() != link.exec_input_id()
            || statement.height() == 0
            || exec.prev_settlement_root() != statement.prev_settlement_root()
            || exec.tx_data_root() != exec_tx_root
            || statement.prev_settlement_root().generation() != RootGeneration::SettlementV2
            || statement.new_settlement_root().generation() != RootGeneration::SettlementV2
            || prep_snapshot.settlement_root_v2() != Some(statement.prev_settlement_root())
            || (exec_is_recursive_v2_noop
                && (exec_tx_count != 0
                    || statement.prev_settlement_root() != statement.new_settlement_root()))
            || (!exec_is_recursive_v2_noop && exec_tx_count == 0)
        {
            return Err(CheckpointError::Authority);
        }

        let core = artifact
            .statement_core()
            .ok_or(CheckpointError::Authority)?;
        let da_ref = artifact.da_ref().ok_or(CheckpointError::Authority)?;
        let final_bind = CheckpointTransitionStatementFinalV1::new(da_ref);
        let statement_core_digest = statement.statement_core_digest_v1(&core);
        let statement_digest = statement.final_statement_digest_v1(&core, &final_bind);
        if artifact.statement_digest_v1() != Some(statement_digest)
            || statement_digest == [0; 32]
            || statement_core_digest == [0; 32]
            || link.link_bind() == [0; 32]
        {
            return Err(CheckpointError::Authority);
        }

        match link.prev_checkpoint_id() {
            None if statement.height() == 1 => {}
            Some(predecessor_id) if statement.height() > 1 => {
                let predecessor = checkpoint_store
                    .load_artifact(&predecessor_id)
                    .map_err(|_| CheckpointError::Authority)?;
                if derive_checkpoint_id(&predecessor).map_err(|_| CheckpointError::Authority)?
                    != predecessor_id
                {
                    return Err(CheckpointError::Authority);
                }
                let predecessor_statement = match predecessor.statement() {
                    CheckpointStatement::V1(statement) => statement,
                    CheckpointStatement::Detached => return Err(CheckpointError::Authority),
                };
                if predecessor_statement
                    .height()
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?
                    != statement.height()
                    || predecessor_statement.new_settlement_root()
                        != statement.prev_settlement_root()
                {
                    return Err(CheckpointError::Authority);
                }
            }
            _ => return Err(CheckpointError::Authority),
        }

        Ok(Self {
            checkpoint_id,
            predecessor: link.prev_checkpoint_id(),
            height: statement.height(),
            prep_snapshot_id: link.prep_snapshot_id(),
            exec_tx_root,
            exec_tx_count,
            exec_is_recursive_v2_noop,
            exec_version: exec.version().as_u8(),
            statement_digest,
            statement_core_digest,
            delta_root: core.delta_root(),
            witness_root: core.witness_root(),
            journal_digest: core.journal_digest(),
            prior_recursive_output_root: core.prior_recursive_output_root(),
            checkpoint_link_digest: link.link_bind(),
            pre_settlement_root: statement.prev_settlement_root(),
            post_settlement_root: statement.new_settlement_root(),
        })
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
    pub(crate) const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub(crate) const fn prep_snapshot_id(&self) -> PrepSnapshotId {
        self.prep_snapshot_id
    }

    #[must_use]
    pub(crate) const fn exec_tx_root(&self) -> [u8; 32] {
        self.exec_tx_root
    }

    #[must_use]
    pub(crate) const fn exec_tx_count(&self) -> u32 {
        self.exec_tx_count
    }

    #[must_use]
    pub(crate) const fn is_recursive_v2_noop(&self) -> bool {
        self.exec_is_recursive_v2_noop
    }

    #[must_use]
    pub(crate) const fn exec_version(&self) -> u8 {
        self.exec_version
    }

    /// Bind the storage handoff to the canonical execution rows loaded from
    /// the checkpoint store before either preflight or live mutation begins.
    pub(crate) fn verify_handoff(
        &self,
        handoff: &SettlementExecHandoff,
    ) -> Result<(), CheckpointError> {
        if self.exec_is_recursive_v2_noop {
            return if handoff.is_recursive_v2_noop() {
                Ok(())
            } else {
                Err(CheckpointError::Authority)
            };
        }
        if u32::try_from(handoff.txs().len()).map_err(|_| CheckpointError::Limit)?
            != self.exec_tx_count
            || derive_exec_tx_root(handoff.txs()).map_err(|_| CheckpointError::Authority)?
                != self.exec_tx_root
        {
            return Err(CheckpointError::Authority);
        }
        Ok(())
    }

    #[must_use]
    pub(crate) const fn statement_digest(&self) -> [u8; 32] {
        self.statement_digest
    }

    #[must_use]
    pub(crate) const fn statement_core_digest(&self) -> [u8; 32] {
        self.statement_core_digest
    }

    #[must_use]
    pub(crate) const fn delta_root(&self) -> [u8; 32] {
        self.delta_root
    }

    #[must_use]
    pub(crate) const fn witness_root(&self) -> [u8; 32] {
        self.witness_root
    }

    #[must_use]
    pub(crate) const fn journal_digest(&self) -> [u8; 32] {
        self.journal_digest
    }

    #[must_use]
    pub(crate) const fn prior_recursive_output_root(&self) -> Option<[u8; 32]> {
        self.prior_recursive_output_root
    }

    #[must_use]
    pub(crate) const fn checkpoint_link_digest(&self) -> [u8; 32] {
        self.checkpoint_link_digest
    }

    #[must_use]
    pub(crate) const fn pre_settlement_root(&self) -> SettlementStateRoot {
        self.pre_settlement_root
    }

    #[must_use]
    pub(crate) const fn post_settlement_root(&self) -> SettlementStateRoot {
        self.post_settlement_root
    }
}

/// The only capability accepted by the V2 transition and cutover paths.
///
/// It derives both authority and snapshot identity from the repository-owned
/// checkpoint configuration and the current settlement-store owner.  No
/// public constructor accepts caller-selected network/config/layout/generation
/// values or a caller-invented `PrepSnapshotId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursiveAuthoritySnapshotV2 {
    authority: RecursiveAuthorityContextV2,
    snapshot: RecursiveSnapshotHandleV2,
    config_identity: ActiveCheckpointConfigIdentityV3,
}

impl RecursiveAuthoritySnapshotV2 {
    /// Resolve the active authority from repository-owned configuration and state.
    pub fn resolve_active_authority(store: &SettlementStore) -> Result<Self, CheckpointError> {
        let active =
            CheckpointConfigResolverV3::resolve_active().map_err(|_| CheckpointError::Authority)?;
        let authority = active_authority(store, &active)?;
        let root = store
            .settlement_root_v2(authority.layout())
            .map_err(|_| CheckpointError::Authority)?;
        let generation = store.recursive_v2_storage_generation().to_le_bytes();
        let snapshot_id = PrepSnapshotId::new(sha256_256_role(
            CheckpointShaRole::Content,
            &[
                b"z00z.recursive.v2.active-authority-snapshot",
                &authority.digest(),
                root.as_bytes(),
                &generation,
            ],
        ));
        Self::resolve_active_authority_for_snapshot_with_config(store, snapshot_id, &active)
    }

    /// Resolve the active authority around the prep-snapshot ID reloaded from
    /// the canonical checkpoint artifact/link owner.
    pub(crate) fn resolve_active_authority_for_snapshot(
        store: &SettlementStore,
        snapshot_id: PrepSnapshotId,
    ) -> Result<Self, CheckpointError> {
        let active =
            CheckpointConfigResolverV3::resolve_active().map_err(|_| CheckpointError::Authority)?;
        Self::resolve_active_authority_for_snapshot_with_config(store, snapshot_id, &active)
    }

    fn resolve_active_authority_for_snapshot_with_config(
        store: &SettlementStore,
        snapshot_id: PrepSnapshotId,
        active: &ActiveCheckpointConfigV3,
    ) -> Result<Self, CheckpointError> {
        let authority = active_authority(store, active)?;
        let snapshot =
            RecursiveSnapshotHandleV2::from_store(snapshot_id, store, authority.layout())?;
        Ok(Self {
            authority,
            snapshot,
            config_identity: active.identity(),
        })
    }

    /// Detect configuration rotation before each transition boundary.
    pub(crate) fn revalidate_config(&self) -> Result<(), CheckpointError> {
        CheckpointConfigResolverV3::require_current(self.config_identity)
            .map_err(|_| CheckpointError::Authority)
    }

    #[must_use]
    pub(crate) const fn authority(&self) -> RecursiveAuthorityContextV2 {
        self.authority
    }

    #[must_use]
    pub(crate) const fn snapshot(&self) -> RecursiveSnapshotHandleV2 {
        self.snapshot
    }
}

fn active_authority(
    store: &SettlementStore,
    active: &ActiveCheckpointConfigV3,
) -> Result<RecursiveAuthorityContextV2, CheckpointError> {
    let config = active.config();
    let config_digest = active.head().config_digest;
    let policy_digest = store.bucket_policy().bucket_policy_id();
    let predicate_digest = super::nova::executable_predicate_digest()?;
    let checkpoint_context =
        RecursiveCheckpointContextV2::from_installed_identity(config_digest, predicate_digest)?;
    RecursiveAuthorityContextV2::new(
        checkpoint_context,
        config_digest,
        policy_digest,
        RECURSIVE_V2_ACTIVE_AUTHORITY_LAYOUT,
        active.head().authority_generation,
        config.branches.plonky3_epoch.cadence_blocks,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_context_has_frozen_digest_and_field_mutation_separation() {
        let context = RecursiveCheckpointContextV2::test_fixture([1; 32], [2; 32]);
        let expected = [
            0x02, 0x99, 0x58, 0x79, 0x7f, 0x83, 0xcf, 0x2d, 0x64, 0x0e, 0x68, 0x4c, 0x5f, 0x08,
            0x97, 0x03, 0xfe, 0x12, 0x92, 0x32, 0x97, 0x03, 0x5a, 0xe6, 0x1a, 0x1f, 0x4c, 0x7c,
            0xb2, 0xd5, 0x3a, 0x71,
        ];
        assert_eq!(context.canonical_bytes().len(), 2 + 4 + 32 * 4);
        assert_eq!(context.digest(), expected);

        let fields = [
            context_digest_fields(3, 1, [0x51; 32], [0x61; 32], [1; 32], [2; 32]),
            context_digest_fields(2, 0, [0x51; 32], [0x61; 32], [1; 32], [2; 32]),
            context_digest_fields(2, 1, [0x50; 32], [0x61; 32], [1; 32], [2; 32]),
            context_digest_fields(2, 1, [0x51; 32], [0x60; 32], [1; 32], [2; 32]),
            context_digest_fields(2, 1, [0x51; 32], [0x61; 32], [0; 32], [2; 32]),
            context_digest_fields(2, 1, [0x51; 32], [0x61; 32], [1; 32], [3; 32]),
        ];
        assert!(fields.into_iter().all(|digest| digest != expected));
    }

    #[test]
    fn no_op_execution_input_version_is_private_and_canonical() {
        let context = RecursiveCheckpointContextV2::test_fixture([1; 32], [2; 32]);
        let authority = RecursiveAuthorityContextV2::new(
            context,
            [1; 32],
            [3; 32],
            RECURSIVE_V2_ACTIVE_AUTHORITY_LAYOUT,
            1,
            1_000,
        )
        .expect("canonical recursive authority");

        assert_eq!(authority.noop_execution_input_version(), 2);
        assert!(authority.allows_noop_execution_input_version(2));
        assert!(!authority.allows_noop_execution_input_version(1));
    }

    #[test]
    fn snapshot_captures_the_definition_root_used_by_the_v2_root() {
        let store = SettlementStore::new();
        let snapshot =
            RecursiveSnapshotHandleV2::from_store(PrepSnapshotId::new([7; 32]), &store, 7)
                .expect("storage-owned snapshot");

        assert_eq!(
            snapshot.pre_definition_root(),
            store.recursive_v2_definition_root()
        );
        assert_eq!(
            snapshot.root(),
            store
                .settlement_root_v2(7)
                .expect("derived settlement root")
        );
    }
}
