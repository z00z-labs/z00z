//! Immutable authority and snapshot-handle bindings for recursive checkpoint V2.

use z00z_crypto::{hash::sha256_256_simple, sha256_256_role, CheckpointShaRole};
use z00z_utils::io::read_file_bounded;

use crate::{
    checkpoint::{
        check_link_ids, derive_checkpoint_id, derive_exec_tx_root, repo_default_path,
        CheckpointContractConfigV1, CheckpointId, CheckpointStatement, CheckpointStore,
        CheckpointTransitionStatementFinalV1,
    },
    settlement::{RootGeneration, SettlementExecHandoff, SettlementStateRoot, SettlementStore},
    snapshot::{PrepSnapshotId, PrepSnapshotStore},
};

use super::recursive_reject::RecursiveV2Error;

/// Fixed layout of the isolated T0 repository-local fixture authority.
///
/// This is not a deployment selector.  `ConfiguredLiveV1` has no discovered
/// authority in the T0 resolution record and therefore has no constructor.
pub const RECURSIVE_V2_REPOSITORY_FIXTURE_LAYOUT: u32 = 7;
const RECURSIVE_V2_REPOSITORY_FIXTURE_GENERATION: u64 = 1;
const RECURSIVE_V2_CONFIG_MAX_BYTES: u64 = 256 * 1024;

/// Immutable authority context captured before recursive trace construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursiveAuthorityContextV2 {
    network_context: [u8; 32],
    config_digest: [u8; 32],
    policy_digest: [u8; 32],
    layout: u32,
    authority_generation: u64,
    noop_execution_input_version: u8,
}

impl RecursiveAuthorityContextV2 {
    /// Construct one nonzero-generation authority context.
    pub(crate) fn new(
        network_context: [u8; 32],
        config_digest: [u8; 32],
        policy_digest: [u8; 32],
        layout: u32,
        authority_generation: u64,
        noop_execution_input_version: u8,
    ) -> Result<Self, RecursiveV2Error> {
        if layout == 0 || authority_generation == 0 || noop_execution_input_version == 0 {
            return Err(RecursiveV2Error::Invariant);
        }
        Ok(Self {
            network_context,
            config_digest,
            policy_digest,
            layout,
            authority_generation,
            noop_execution_input_version,
        })
    }

    #[must_use]
    pub const fn network_context(&self) -> [u8; 32] {
        self.network_context
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

    /// Derive the authority identity consumed by every trace pass.
    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        let layout = self.layout.to_le_bytes();
        let generation = self.authority_generation.to_le_bytes();
        let noop_version = [self.noop_execution_input_version];
        sha256_256_role(
            CheckpointShaRole::UniquenessContext,
            &[
                &self.network_context,
                &self.config_digest,
                &self.policy_digest,
                &layout,
                &generation,
                &noop_version,
            ],
        )
    }
}

/// The opaque immutable snapshot handle bound to both trace passes.
///
/// Equality includes the external snapshot ID, storage generation, canonical
/// V2 root, content digest, and exact counts. Equal caller-supplied IDs alone
/// are not sufficient to reopen a source under a different handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursiveSnapshotHandleV2 {
    snapshot_id: PrepSnapshotId,
    storage_generation: u64,
    root: SettlementStateRoot,
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
        record_count: u64,
        byte_count: u64,
        content_digest: [u8; 32],
    ) -> Result<Self, RecursiveV2Error> {
        if root.generation() != RootGeneration::SettlementV2 {
            return Err(RecursiveV2Error::SnapshotChanged);
        }
        Ok(Self {
            snapshot_id,
            storage_generation,
            root,
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
    ) -> Result<Self, RecursiveV2Error> {
        let (root, storage_generation, record_count, byte_count, content_digest) = store
            .recursive_v2_snapshot_binding(layout)
            .map_err(|_| RecursiveV2Error::SnapshotChanged)?;
        Self::new(
            snapshot_id,
            storage_generation,
            SettlementStateRoot::settlement_v2(root),
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
    checkpoint_link_digest: [u8; 32],
    pre_settlement_root: SettlementStateRoot,
    post_settlement_root: SettlementStateRoot,
}

impl RecursiveCheckpointBindingV2 {
    /// Reload and cross-check one canonical checkpoint relation.  The backing
    /// store remains the sole source of its statement, link, replay input, and
    /// predecessor; no raw component is accepted by the recursive surface.
    pub(crate) fn resolve(
        checkpoint_store: &impl CheckpointStore,
        prep_snapshot_store: &impl PrepSnapshotStore,
        checkpoint_id: CheckpointId,
    ) -> Result<Self, RecursiveV2Error> {
        let artifact = checkpoint_store
            .load_artifact(&checkpoint_id)
            .map_err(|_| RecursiveV2Error::Authority)?;
        if derive_checkpoint_id(&artifact).map_err(|_| RecursiveV2Error::Authority)?
            != checkpoint_id
        {
            return Err(RecursiveV2Error::Authority);
        }
        let link = checkpoint_store
            .load_link(&checkpoint_id)
            .map_err(|_| RecursiveV2Error::Authority)?;
        if link.checkpoint_id() != checkpoint_id {
            return Err(RecursiveV2Error::Authority);
        }
        let exec = checkpoint_store
            .load_exec_input(&link.exec_input_id())
            .map_err(|_| RecursiveV2Error::Authority)?;
        check_link_ids(link.prep_snapshot_id(), &link, &exec)
            .map_err(|_| RecursiveV2Error::Authority)?;
        let prep_snapshot = prep_snapshot_store
            .load_snapshot(&link.prep_snapshot_id())
            .map_err(|_| RecursiveV2Error::Authority)?;

        let statement = match artifact.statement() {
            CheckpointStatement::V1(statement) => statement,
            CheckpointStatement::Detached => return Err(RecursiveV2Error::Authority),
        };
        let exec_tx_count = u32::try_from(exec.txs().len()).map_err(|_| RecursiveV2Error::Limit)?;
        let exec_tx_root = exec
            .expected_tx_data_root()
            .map_err(|_| RecursiveV2Error::Authority)?;
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
            return Err(RecursiveV2Error::Authority);
        }

        let core = artifact
            .statement_core()
            .ok_or(RecursiveV2Error::Authority)?;
        let da_ref = artifact.da_ref().ok_or(RecursiveV2Error::Authority)?;
        let final_bind = CheckpointTransitionStatementFinalV1::new(da_ref);
        let statement_core_digest = statement.statement_core_digest_v1(&core);
        let statement_digest = statement.final_statement_digest_v1(&core, &final_bind);
        if artifact.statement_digest_v1() != Some(statement_digest)
            || statement_digest == [0; 32]
            || statement_core_digest == [0; 32]
            || link.link_bind() == [0; 32]
        {
            return Err(RecursiveV2Error::Authority);
        }

        match link.prev_checkpoint_id() {
            None if statement.height() == 1 => {}
            Some(predecessor_id) if statement.height() > 1 => {
                let predecessor = checkpoint_store
                    .load_artifact(&predecessor_id)
                    .map_err(|_| RecursiveV2Error::Authority)?;
                if derive_checkpoint_id(&predecessor).map_err(|_| RecursiveV2Error::Authority)?
                    != predecessor_id
                {
                    return Err(RecursiveV2Error::Authority);
                }
                let predecessor_statement = match predecessor.statement() {
                    CheckpointStatement::V1(statement) => statement,
                    CheckpointStatement::Detached => return Err(RecursiveV2Error::Authority),
                };
                if predecessor_statement
                    .height()
                    .checked_add(1)
                    .ok_or(RecursiveV2Error::Overflow)?
                    != statement.height()
                    || predecessor_statement.new_settlement_root()
                        != statement.prev_settlement_root()
                {
                    return Err(RecursiveV2Error::Authority);
                }
            }
            _ => return Err(RecursiveV2Error::Authority),
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
    ) -> Result<(), RecursiveV2Error> {
        if self.exec_is_recursive_v2_noop {
            return if handoff.is_recursive_v2_noop() {
                Ok(())
            } else {
                Err(RecursiveV2Error::Authority)
            };
        }
        if u32::try_from(handoff.txs().len()).map_err(|_| RecursiveV2Error::Limit)?
            != self.exec_tx_count
            || derive_exec_tx_root(handoff.txs()).map_err(|_| RecursiveV2Error::Authority)?
                != self.exec_tx_root
        {
            return Err(RecursiveV2Error::Authority);
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
}

impl RecursiveAuthoritySnapshotV2 {
    /// Resolve the only T0-selected repository-local fixture capability.
    ///
    /// It is implementation evidence only: the resulting capability cannot
    /// represent a configured production authority or authorize deployment.
    pub fn resolve_repository_local_fixture(
        store: &SettlementStore,
    ) -> Result<Self, RecursiveV2Error> {
        let authority = repository_fixture_authority(store)?;
        let root = store
            .settlement_root_v2(authority.layout())
            .map_err(|_| RecursiveV2Error::Authority)?;
        let generation = store.recursive_v2_storage_generation().to_le_bytes();
        let snapshot_id = PrepSnapshotId::new(sha256_256_role(
            CheckpointShaRole::Content,
            &[
                b"z00z.recursive.v2.repository-local-snapshot",
                &authority.digest(),
                root.as_bytes(),
                &generation,
            ],
        ));
        Self::resolve_repository_local_fixture_for_snapshot(store, snapshot_id)
    }

    /// Resolve the repository-local fixture authority around the prep-snapshot
    /// ID reloaded from the canonical checkpoint artifact/link owner.
    pub(crate) fn resolve_repository_local_fixture_for_snapshot(
        store: &SettlementStore,
        snapshot_id: PrepSnapshotId,
    ) -> Result<Self, RecursiveV2Error> {
        let authority = repository_fixture_authority(store)?;
        let snapshot =
            RecursiveSnapshotHandleV2::from_store(snapshot_id, store, authority.layout())?;
        Ok(Self {
            authority,
            snapshot,
        })
    }

    /// Detect configuration rotation before each transition boundary.
    pub(crate) fn revalidate_config(&self) -> Result<(), RecursiveV2Error> {
        if repository_config_digest()? != self.authority.config_digest() {
            return Err(RecursiveV2Error::Authority);
        }
        Ok(())
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

fn repository_fixture_authority(
    store: &SettlementStore,
) -> Result<RecursiveAuthorityContextV2, RecursiveV2Error> {
    let config = repository_contract_config()?;
    let config_digest = repository_config_digest()?;
    let policy_digest = store.bucket_policy().bucket_policy_id();
    let network_context = sha256_256_role(
        CheckpointShaRole::Content,
        &[
            b"z00z.recursive.v2.repository-local-fixture",
            &config_digest,
            &policy_digest,
        ],
    );
    RecursiveAuthorityContextV2::new(
        network_context,
        config_digest,
        policy_digest,
        RECURSIVE_V2_REPOSITORY_FIXTURE_LAYOUT,
        RECURSIVE_V2_REPOSITORY_FIXTURE_GENERATION,
        config.branches.recursive.no_op.execution_input_version,
    )
}

fn repository_contract_config() -> Result<CheckpointContractConfigV1, RecursiveV2Error> {
    CheckpointContractConfigV1::load(repo_default_path()).map_err(|_| RecursiveV2Error::Authority)
}

fn repository_config_digest() -> Result<[u8; 32], RecursiveV2Error> {
    let path = repo_default_path();
    let _ = repository_contract_config()?;
    let bytes = read_file_bounded(path, RECURSIVE_V2_CONFIG_MAX_BYTES)
        .map_err(|_| RecursiveV2Error::Authority)?;
    Ok(sha256_256_simple(&bytes))
}
