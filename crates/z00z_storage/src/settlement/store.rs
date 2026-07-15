use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use crate::backend::{
    memory::MemTreeStore,
    redb::{state::RecursiveV2CutoverManifestV2, StoragePlane},
    roots::{HjmtRoots, TreeRoots},
};
use sha2::{Digest, Sha256};

#[cfg(test)]
use std::sync::{Mutex, OnceLock};

use jmt::{RootHash, Version};

#[allow(unused_imports)]
use super::tree_id::{PathIndexRec, TreeId, TreeRootRef};
#[allow(unused_imports)]
use super::{
    derive_settlement_root_v2, model::SettlementModel, tx_plan_types::ObjectDeltaSetV1,
    BucketPolicy, CheckRoot, ClaimSourceRoot, DefinitionId, DefinitionRootLeaf, FeeActorCtx,
    FeeEnvelope, FeeReplayKey, FeeReplayRec, PolicySetCommitmentV1, PolicySetMemberV1, ProofBlob,
    ProofItem, ProofScanOut, RightActionCtx, RightLeaf, RootGeneration, SerialId, SerialRootLeaf,
    SettlementListReq, SettlementLookup, SettlementPage, SettlementPageTok, SettlementPath,
    SettlementStateRoot, SnapItem, StoreItem, TerminalId,
};
use super::{
    hjmt_cache::ForestCache,
    hjmt_config::{bucket_policy_from_env, env_opt, SettlementBackendMode},
    hjmt_journal,
    hjmt_scheduler::ForestScheduler,
    hjmt_store::HjmtStore,
};
use crate::backend::error::StoreBackendError;
pub use crate::backend::types::{
    ClaimNullRec, ClaimNullStatus, ClaimNullTx, ClaimNullifier, SettlementStoreError, StoreOp,
};
use z00z_crypto::{expert::encoding::to_hex, CheckpointSha256V2, CheckpointShaRole};

type RecursiveV2SnapshotBinding = ([u8; 32], u64, u64, u64, [u8; 32]);

#[cfg(test)]
pub(crate) const TEST_HJMT_INJ_STAGE_ENV: &str = "Z00Z_STORAGE_HJMT_INJ_STAGE";

#[cfg(test)]
pub(crate) fn test_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub struct SettlementStore {
    pub(super) backend_mode: SettlementBackendMode,
    pub(super) bucket_policy: BucketPolicy,
    pub(crate) backend: StoragePlane,
    pub(crate) forest_cache: ForestCache,
    pub(super) scheduler: ForestScheduler,
    pub(super) hjmt_store: HjmtStore,
    pub(crate) hjmt_roots: HjmtRoots,
    pub(crate) flat_store: MemTreeStore,
    pub(crate) flat_version: Version,
    pub(crate) flat_root: Option<RootHash>,
    pub(crate) model: SettlementModel,
    pub(crate) tree_roots: TreeRoots,
    pub(crate) path_by_terminal_id: HashMap<TerminalId, SettlementPath>,
    pub(crate) nullifier: BTreeMap<ClaimNullifier, ClaimNullRec>,
    pub(crate) claim_null_seq: u64,
    pub(crate) fee_replays: BTreeMap<FeeReplayKey, FeeReplayRec>,
    pub(crate) fee_replay_seq: u64,
    pub(crate) settlement_root_by_ver: HashMap<Version, SettlementStateRoot>,
    pub(crate) model_by_ver: HashMap<Version, SettlementModel>,
    pub(crate) hjmt_roots_by_ver: HashMap<Version, HjmtRoots>,
    pub(crate) last_object_delta: Option<ObjectDeltaSetV1>,
    pub(crate) object_deltas_by_ver: HashMap<Version, ObjectDeltaSetV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettlementRouteCtx {
    batch_id: [u8; 32],
    shard_id: u32,
    routing_generation: u64,
    route_table_digest: [u8; 32],
}

impl SettlementRouteCtx {
    #[must_use]
    pub const fn new(
        batch_id: [u8; 32],
        shard_id: u32,
        routing_generation: u64,
        route_table_digest: [u8; 32],
    ) -> Self {
        Self {
            batch_id,
            shard_id,
            routing_generation,
            route_table_digest,
        }
    }

    #[must_use]
    pub const fn batch_id(self) -> [u8; 32] {
        self.batch_id
    }

    #[must_use]
    pub const fn shard_id(self) -> u32 {
        self.shard_id
    }

    #[must_use]
    pub const fn routing_generation(self) -> u64 {
        self.routing_generation
    }

    #[must_use]
    pub const fn route_table_digest(self) -> [u8; 32] {
        self.route_table_digest
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettlementExecHandoff {
    route: SettlementRouteCtx,
    ops: Vec<StoreOp>,
    txs: Vec<crate::checkpoint::CheckpointExecTx>,
}

impl SettlementExecHandoff {
    #[must_use]
    pub fn new(
        route: SettlementRouteCtx,
        ops: Vec<StoreOp>,
        txs: Vec<crate::checkpoint::CheckpointExecTx>,
    ) -> Self {
        Self { route, ops, txs }
    }

    /// Construct the sole canonical empty handoff accepted by the recursive V2
    /// authority-defined no-op relation.  Its sentinel route prevents an empty
    /// ad-hoc execution request from becoming a second no-op encoding.
    #[must_use]
    pub fn recursive_v2_noop() -> Self {
        Self {
            route: SettlementRouteCtx::new([0x4e; 32], u32::MAX, u64::MAX, [0xa5; 32]),
            ops: Vec::new(),
            txs: Vec::new(),
        }
    }

    #[must_use]
    pub(crate) fn is_recursive_v2_noop(&self) -> bool {
        self.ops.is_empty()
            && self.txs.is_empty()
            && self.route == SettlementRouteCtx::new([0x4e; 32], u32::MAX, u64::MAX, [0xa5; 32])
    }

    #[must_use]
    pub const fn route(&self) -> SettlementRouteCtx {
        self.route
    }

    #[must_use]
    pub fn ops(&self) -> &[StoreOp] {
        &self.ops
    }

    #[must_use]
    pub fn txs(&self) -> &[crate::checkpoint::CheckpointExecTx] {
        &self.txs
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        SettlementRouteCtx,
        Vec<StoreOp>,
        Vec<crate::checkpoint::CheckpointExecTx>,
    ) {
        (self.route, self.ops, self.txs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeOpKind {
    Put,
    Delete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeLeafKind {
    Terminal,
    Right,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeSeen {
    pub definition: bool,
    pub serial: bool,
    pub object: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeFlowItem {
    pub tx_id: String,
    pub op_kind: ScopeOpKind,
    pub definition_id: String,
    pub serial_id: u32,
    pub terminal_id: String,
    pub leaf_family: ScopeLeafKind,
    pub first_seen: ScopeSeen,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeRootFlow {
    pub prev_root: String,
    pub post_root: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeFlow {
    pub batch_id: String,
    pub shard_id: u32,
    pub routing_generation: u64,
    pub route_table_digest: String,
    pub items: Vec<ScopeFlowItem>,
    pub root_flow: ScopeRootFlow,
}

impl ScopeFlow {
    #[must_use]
    pub fn new(
        route: SettlementRouteCtx,
        items: Vec<ScopeFlowItem>,
        prev_root: SettlementStateRoot,
        post_root: SettlementStateRoot,
    ) -> Self {
        Self {
            batch_id: to_hex(&route.batch_id()),
            shard_id: route.shard_id(),
            routing_generation: route.routing_generation(),
            route_table_digest: to_hex(&route.route_table_digest()),
            items,
            root_flow: ScopeRootFlow {
                prev_root: to_hex(prev_root.as_bytes()),
                post_root: to_hex(post_root.as_bytes()),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettlementRecoveryState {
    pub version: u64,
    pub state_root: SettlementStateRoot,
    pub root_generation: u8,
    pub proof_version: u16,
    pub bucket_policy_generation: u32,
    pub bucket_policy_id: [u8; 32],
    pub journal_lineage: [u8; 32],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<SettlementRouteCtx>,
}

impl SettlementRecoveryState {
    #[must_use]
    pub fn new(
        version: u64,
        state_root: SettlementStateRoot,
        root_generation: u8,
        proof_version: u16,
        bucket_policy_generation: u32,
        bucket_policy_id: [u8; 32],
        journal_lineage: [u8; 32],
    ) -> Self {
        Self {
            version,
            state_root,
            root_generation,
            proof_version,
            bucket_policy_generation,
            bucket_policy_id,
            journal_lineage,
            route: None,
        }
    }

    #[must_use]
    pub fn with_route(mut self, route: SettlementRouteCtx) -> Self {
        self.route = Some(route);
        self
    }

    #[must_use]
    pub fn live_policy_member_v1(&self, activation_checkpoint: u64) -> PolicySetMemberV1 {
        PolicySetMemberV1::new(
            u64::from(self.bucket_policy_generation),
            self.bucket_policy_id,
            activation_checkpoint,
            None,
        )
    }

    #[must_use]
    pub fn live_policy_set_v1(&self, activation_checkpoint: u64) -> PolicySetCommitmentV1 {
        PolicySetCommitmentV1::new(vec![self.live_policy_member_v1(activation_checkpoint)])
    }
}

pub(crate) fn scope_tx_id(index: usize, proof: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"z00z.scope.tx-id.v1");
    hasher.update((index as u64).to_be_bytes());
    hasher.update((proof.len() as u64).to_be_bytes());
    hasher.update(proof);
    to_hex(&hasher.finalize())
}

/// Storage-owned semantic facade for generalized settlement operations.
///
/// This trait is the only public semantic boundary above the raw backend layer.
/// Physical backend roots and table layout stay private to storage internals.
pub trait SettlementTreeBackend {
    fn settlement_root(&self) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn settlement_root_v2(&self, layout: u32) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn settlement_root_for_version(
        &self,
        version: Version,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn get_settlement_item(
        &self,
        path: &SettlementPath,
    ) -> Result<Option<StoreItem>, SettlementStoreError>;

    fn lookup_settlement(
        &self,
        lookup: SettlementLookup,
    ) -> Result<Option<StoreItem>, SettlementStoreError>;

    fn list_settlement(
        &self,
        req: SettlementListReq,
    ) -> Result<SettlementPage, SettlementStoreError>;

    fn put_settlement_item(
        &mut self,
        item: StoreItem,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn del_settlement_item(
        &mut self,
        path: &SettlementPath,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn apply_settlement_ops(
        &mut self,
        ops: Vec<StoreOp>,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn apply_exec_handoff(
        &mut self,
        handoff: SettlementExecHandoff,
    ) -> Result<ScopeFlow, SettlementStoreError>;

    fn recovery_state(&self) -> Result<SettlementRecoveryState, SettlementStoreError>;

    fn settlement_proof_item(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofItem, SettlementStoreError>;

    fn settlement_proof_blob(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofBlob, SettlementStoreError>;

    fn settlement_proof_scan(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofScanOut, SettlementStoreError>;

    fn settlement_inclusion_batch_v1(
        &self,
        paths: &[SettlementPath],
    ) -> Result<crate::settlement::BatchProofBlobV1, SettlementStoreError>;

    fn settlement_nonexistence_batch_v1(
        &self,
        paths: &[SettlementPath],
        leaf_family: crate::settlement::SettlementLeafFamily,
    ) -> Result<crate::settlement::BatchProofBlobV1, SettlementStoreError>;

    fn settlement_deletion_batch_v1(
        &self,
        paths: &[SettlementPath],
    ) -> Result<crate::settlement::BatchProofBlobV1, SettlementStoreError>;

    fn create_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn create_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn transfer_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn transfer_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn consume_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn consume_right_with_fee(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn revoke_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn revoke_right_with_fee(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn expire_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn challenge_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn challenge_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;
}

impl SettlementStore {
    /// Create a managed local store for tests, simulations, and local benches.
    ///
    /// This constructor ignores env-driven durable root discovery, uses a
    /// managed local backend outside `#[cfg(test)]`, and never panics on
    /// startup drift. Use `try_new()` or `load()` for fallible operator-bound
    /// open or reload boundaries.
    pub fn new() -> Self {
        let mode = SettlementBackendMode::from_env_or_default().unwrap_or_default();
        let bucket_policy =
            bucket_policy_from_env().unwrap_or_else(|_| BucketPolicy::default_fixed());
        Self::build_with_policy(StoragePlane::managed_default(), mode, bucket_policy)
    }

    /// Create a settlement store using the canonical fallible startup path.
    pub fn try_new() -> Result<Self, SettlementStoreError> {
        let mode = SettlementBackendMode::from_env_or_default()?;
        let bucket_policy = bucket_policy_from_env()?;
        if let Some(root) = env_opt("Z00Z_STORAGE_REDB_ROOT") {
            return Self::open_with_policy(PathBuf::from(root), mode, bucket_policy);
        }

        Ok(Self::build_with_policy(
            StoragePlane::default(),
            mode,
            bucket_policy,
        ))
    }

    #[cfg(not(test))]
    pub(crate) fn transient_hjmt() -> Result<Self, SettlementStoreError> {
        Ok(Self::build_with_policy(
            StoragePlane::off(),
            SettlementBackendMode::Hjmt,
            bucket_policy_from_env()?,
        ))
    }

    #[cfg(test)]
    pub(crate) fn test_hjmt_store() -> Self {
        Self::build_with_policy(
            StoragePlane::off(),
            SettlementBackendMode::Hjmt,
            BucketPolicy::default_fixed(),
        )
    }

    pub fn load(root: impl Into<PathBuf>) -> Result<Self, SettlementStoreError> {
        let mode = SettlementBackendMode::from_env_or_default()?;
        let bucket_policy = bucket_policy_from_env()?;
        Self::open_with_policy(root, mode, bucket_policy)
    }

    #[cfg(all(test, feature = "test-params-fast"))]
    pub(crate) fn load_with_backend_mode(
        root: impl Into<PathBuf>,
        mode: SettlementBackendMode,
    ) -> Result<Self, SettlementStoreError> {
        Self::open_with_policy(root, mode, BucketPolicy::default_fixed())
    }

    fn open_with_policy(
        root: impl Into<PathBuf>,
        mode: SettlementBackendMode,
        bucket_policy: BucketPolicy,
    ) -> Result<Self, SettlementStoreError> {
        let backend = StoragePlane::new(root.into());
        let mut store = Self::build_with_policy(StoragePlane::off(), mode, bucket_policy);
        crate::backend::JournalBackend::recover_journal(&backend)?;
        if let Some(state) = backend.load_state()? {
            store.hjmt_rehydrate(state)?;
        }
        store.backend = backend;
        Ok(store)
    }

    pub(super) fn build_with_policy(
        backend: StoragePlane,
        backend_mode: SettlementBackendMode,
        bucket_policy: BucketPolicy,
    ) -> Self {
        Self {
            backend_mode,
            bucket_policy,
            backend,
            forest_cache: ForestCache::new(),
            scheduler: ForestScheduler::new(),
            hjmt_store: HjmtStore::new(),
            hjmt_roots: HjmtRoots::new(),
            flat_store: MemTreeStore::new(),
            flat_version: 0,
            flat_root: None,
            model: SettlementModel::new(),
            tree_roots: TreeRoots::default(),
            path_by_terminal_id: HashMap::new(),
            nullifier: BTreeMap::new(),
            claim_null_seq: 0,
            fee_replays: BTreeMap::new(),
            fee_replay_seq: 0,
            settlement_root_by_ver: HashMap::new(),
            model_by_ver: HashMap::new(),
            hjmt_roots_by_ver: HashMap::new(),
            last_object_delta: None,
            object_deltas_by_ver: HashMap::new(),
        }
    }

    #[must_use]
    pub fn backend_name(&self) -> &'static str {
        self.backend_mode.name()
    }

    #[must_use]
    pub const fn bucket_policy(&self) -> BucketPolicy {
        self.bucket_policy
    }

    pub(crate) fn require_hjmt_mode(&self) -> Result<(), SettlementStoreError> {
        let _ = self;
        Ok(())
    }

    pub fn settlement_root(&self) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        Ok(self.hjmt_roots.settlement_root())
    }

    /// Derive the only recursive V2 settlement root from the live HJMT
    /// definition-tree root. This does not reinterpret legacy semantic roots.
    pub fn settlement_root_v2(
        &self,
        layout: u32,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        if layout == 0 {
            return Err(SettlementStoreError::Backend(
                "recursive V2 settlement layout must be nonzero".to_string(),
            ));
        }
        let definition_root = self.recursive_v2_definition_root();
        derive_settlement_root_v2(
            RootGeneration::SettlementV2,
            layout,
            self.bucket_policy.bucket_policy_id(),
            definition_root,
        )
        .map_err(|err| {
            SettlementStoreError::Backend(format!("recursive V2 root derivation: {err}"))
        })
    }

    /// Return the storage-owned definition-tree root used by the sole recursive
    /// V2 root derivation. This is crate-visible so the independent evaluator
    /// cannot accept a caller-supplied post-state root.
    #[must_use]
    pub(crate) fn recursive_v2_definition_root(&self) -> [u8; 32] {
        self.hjmt_roots
            .def_root
            .map(TreeRootRef::into_bytes)
            .unwrap_or(*b"SPARSE_MERKLE_PLACEHOLDER_HASH__")
    }

    /// Return the live HJMT generation captured by a recursive V2 snapshot.
    #[must_use]
    pub const fn recursive_v2_storage_generation(&self) -> u64 {
        self.hjmt_roots.version
    }

    /// Enumerate canonical records once and return the V2 snapshot binding.
    ///
    /// The result is crate-visible only. The sole public V2 path obtains it
    /// through `RecursiveAuthoritySnapshotV2::resolve_repository_local_fixture`,
    /// which prevents caller-selected snapshot, count, byte-count, or
    /// content-digest claims.
    pub(crate) fn recursive_v2_snapshot_binding(
        &self,
        layout: u32,
    ) -> Result<RecursiveV2SnapshotBinding, SettlementStoreError> {
        let root = self.settlement_root_v2(layout)?;
        let mut record_count = 0_u64;
        let mut byte_count = 0_u64;
        let mut content = CheckpointSha256V2::new(CheckpointShaRole::Content);
        for path in self.sorted_paths() {
            let Some(item) = self.hjmt_get_settlement_item(&path)? else {
                continue;
            };
            let leaf = item.leaf().encode().map_err(|error| {
                SettlementStoreError::Backend(format!(
                    "recursive V2 snapshot leaf encoding: {error}"
                ))
            })?;
            let leaf_len = u32::try_from(leaf.len()).map_err(|_| {
                SettlementStoreError::Backend("recursive V2 snapshot leaf too large".to_string())
            })?;
            let mut record = Vec::with_capacity(32 + 4 + 32 + 1 + 4 + leaf.len());
            record.extend_from_slice(&path.definition_id.into_bytes());
            record.extend_from_slice(&path.serial_id.get().to_le_bytes());
            record.extend_from_slice(&path.terminal_id().into_bytes());
            record.push(item.leaf().family_tag());
            record.extend_from_slice(&leaf_len.to_le_bytes());
            record.extend_from_slice(&leaf);
            content.update_part(&record).map_err(|_| {
                SettlementStoreError::Backend(
                    "recursive V2 snapshot content exceeds SHA limit".to_string(),
                )
            })?;
            record_count = record_count.checked_add(1).ok_or_else(|| {
                SettlementStoreError::Backend(
                    "recursive V2 snapshot record-count overflow".to_string(),
                )
            })?;
            byte_count = byte_count
                .checked_add(u64::try_from(record.len()).map_err(|_| {
                    SettlementStoreError::Backend(
                        "recursive V2 snapshot byte-count overflow".to_string(),
                    )
                })?)
                .ok_or_else(|| {
                    SettlementStoreError::Backend(
                        "recursive V2 snapshot byte-count overflow".to_string(),
                    )
                })?;
        }
        Ok((
            *root.as_bytes(),
            self.recursive_v2_storage_generation(),
            record_count,
            byte_count,
            content.finalize(),
        ))
    }

    /// Persist the sole recursive-V2 cutover record with a durable CAS against
    /// the exact live HJMT generation. The store owner independently checks
    /// the V2 root before the backend writes the authority manifest.
    pub(crate) fn install_recursive_v2_cutover(
        &mut self,
        manifest: RecursiveV2CutoverManifestV2,
    ) -> Result<(), SettlementStoreError> {
        if !self.backend.is_on() {
            return Err(SettlementStoreError::Backend(
                "recursive V2 cutover requires a durable isolated store".to_string(),
            ));
        }
        if manifest.policy_digest != self.bucket_policy().bucket_policy_id()
            || manifest.storage_generation != self.recursive_v2_storage_generation()
            || manifest.expected_definition_root != self.recursive_v2_definition_root()
            || manifest.expected_settlement_root
                != *self.settlement_root_v2(manifest.layout)?.as_bytes()
        {
            return Err(SettlementStoreError::Backend(
                "recursive V2 cutover manifest does not match the live store".to_string(),
            ));
        }
        self.backend
            .install_recursive_v2_cutover(&manifest)
            .map_err(|err| SettlementStoreError::Backend(err.to_string()))?;
        if self
            .backend
            .load_recursive_v2_cutover()
            .map_err(|err| SettlementStoreError::Backend(err.to_string()))?
            .as_ref()
            != Some(&manifest)
        {
            return Err(SettlementStoreError::Backend(
                "recursive V2 cutover durable readback mismatch".to_string(),
            ));
        }
        Ok(())
    }

    pub fn settlement_root_for_version(
        &self,
        version: Version,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        if let Some(root) = self.settlement_root_by_ver.get(&version).copied() {
            return Ok(root);
        }

        let Some((_, roots)) = self.hjmt_history_at(version)? else {
            return Err(SettlementStoreError::Backend(
                "missing settlement root version".to_string(),
            ));
        };
        Ok(roots.settlement_root())
    }

    pub(crate) fn hjmt_history_at(
        &self,
        version: Version,
    ) -> Result<Option<(SettlementModel, HjmtRoots)>, SettlementStoreError> {
        if let (Some(model), Some(roots)) = (
            self.model_by_ver.get(&version),
            self.hjmt_roots_by_ver.get(&version),
        ) {
            return Ok(Some((model.clone(), roots.clone())));
        }
        let Some(store) = self.hjmt_store_at(version)? else {
            return Ok(None);
        };
        Ok(Some((store.model, store.hjmt_roots)))
    }

    pub(crate) fn hjmt_store_at(
        &self,
        version: Version,
    ) -> Result<Option<Self>, SettlementStoreError> {
        let Some(state) = self
            .backend
            .load_hjmt_state_at(version)
            .map_err(|err| SettlementStoreError::Backend(err.to_string()))?
        else {
            return Ok(None);
        };

        let mut store = Self::build_with_policy(
            StoragePlane::off(),
            SettlementBackendMode::Hjmt,
            self.bucket_policy(),
        );
        store.hjmt_rehydrate(state)?;
        Ok(Some(store))
    }

    /// Build an isolated, non-durable exact copy for recursive-V2 preflight.
    ///
    /// The canonical V2 transition uses this clone to construct and seal its
    /// sole witness source before it mutates the live store.  The clone keeps
    /// every semantic and HJMT versioned field, but its backend is explicitly
    /// off so a rejected preflight cannot create a durable side effect.
    pub(crate) fn recursive_v2_preflight_clone(&self) -> Self {
        let mut clone =
            Self::build_with_policy(StoragePlane::off(), self.backend_mode, self.bucket_policy());
        clone.forest_cache.restore(self.forest_cache.snapshot());
        clone.hjmt_store.restore(self.hjmt_store.snap());
        clone.hjmt_roots = self.hjmt_roots.clone();
        clone.flat_store.restore(self.flat_store.snap());
        clone.flat_version = self.flat_version;
        clone.flat_root = self.flat_root;
        clone.model = self.model.clone();
        clone.tree_roots = self.tree_roots.clone();
        clone.path_by_terminal_id = self.path_by_terminal_id.clone();
        clone.nullifier = self.nullifier.clone();
        clone.claim_null_seq = self.claim_null_seq;
        clone.fee_replays = self.fee_replays.clone();
        clone.fee_replay_seq = self.fee_replay_seq;
        clone.settlement_root_by_ver = self.settlement_root_by_ver.clone();
        clone.model_by_ver = self.model_by_ver.clone();
        clone.hjmt_roots_by_ver = self.hjmt_roots_by_ver.clone();
        clone.last_object_delta = self.last_object_delta.clone();
        clone.object_deltas_by_ver = self.object_deltas_by_ver.clone();
        clone
    }

    pub fn apply_settlement_ops(
        &mut self,
        ops: Vec<StoreOp>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_apply_settlement_ops(ops)
    }

    pub fn apply_exec_handoff(
        &mut self,
        handoff: SettlementExecHandoff,
    ) -> Result<ScopeFlow, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_apply_exec_handoff(handoff)
    }

    /// Execute the only recursive-V2 eligible handoff and expose the traced
    /// pinned-JMT result. The ordinary semantic API intentionally does not
    /// retain this proving artifact.
    pub(crate) fn apply_exec_handoff_v2(
        &mut self,
        handoff: SettlementExecHandoff,
    ) -> Result<
        (
            ScopeFlow,
            super::proof_batch::SettlementUpdateTraceEnvelopeV2,
        ),
        SettlementStoreError,
    > {
        self.require_hjmt_mode()?;
        self.hjmt_apply_exec_handoff_with_update_trace(handoff)
    }

    pub fn recovery_state(&self) -> Result<SettlementRecoveryState, SettlementStoreError> {
        self.require_hjmt_mode()?;

        let version = self.hjmt_roots.version;
        let state_root = self.settlement_root()?;
        if version == 0 {
            return Ok(SettlementRecoveryState::new(
                0,
                state_root,
                hjmt_journal::HJMT_JOURNAL_ROOT_GENERATION,
                hjmt_journal::HJMT_JOURNAL_PROOF_VERSION,
                self.bucket_policy().compatibility_generation(),
                self.bucket_policy().bucket_policy_id(),
                [0u8; 32],
            ));
        }

        let state = self.backend.load_hjmt_state_at(version)?.ok_or_else(|| {
            SettlementStoreError::Backend(
                "missing persisted hjmt recovery state for the active version".to_string(),
            )
        })?;
        let journal = state.hjmt_journal.ok_or_else(|| {
            SettlementStoreError::Backend(
                "missing persisted hjmt journal for the active version".to_string(),
            )
        })?;

        let mut recovery = SettlementRecoveryState::new(
            version,
            state_root,
            journal.root_generation,
            journal.proof_version,
            self.bucket_policy().compatibility_generation(),
            journal.bucket_policy_id,
            hjmt_journal::hjmt_journal_digest(&journal),
        );
        if let Some(route) = journal.route {
            recovery = recovery.with_route(route);
        }

        Ok(recovery)
    }
}

impl SettlementTreeBackend for SettlementStore {
    fn settlement_root(&self) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::settlement_root(self)
    }

    fn settlement_root_v2(&self, layout: u32) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::settlement_root_v2(self, layout)
    }

    fn settlement_root_for_version(
        &self,
        version: Version,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::settlement_root_for_version(self, version)
    }

    fn get_settlement_item(
        &self,
        path: &SettlementPath,
    ) -> Result<Option<StoreItem>, SettlementStoreError> {
        SettlementStore::get_settlement_item(self, path)
    }

    fn lookup_settlement(
        &self,
        lookup: SettlementLookup,
    ) -> Result<Option<StoreItem>, SettlementStoreError> {
        SettlementStore::lookup_settlement(self, lookup)
    }

    fn list_settlement(
        &self,
        req: SettlementListReq,
    ) -> Result<SettlementPage, SettlementStoreError> {
        SettlementStore::list_settlement(self, req)
    }

    fn put_settlement_item(
        &mut self,
        item: StoreItem,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::put_settlement_item(self, item)
    }

    fn del_settlement_item(
        &mut self,
        path: &SettlementPath,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::del_settlement_item(self, path)
    }

    fn apply_settlement_ops(
        &mut self,
        ops: Vec<StoreOp>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::apply_settlement_ops(self, ops)
    }

    fn apply_exec_handoff(
        &mut self,
        handoff: SettlementExecHandoff,
    ) -> Result<ScopeFlow, SettlementStoreError> {
        SettlementStore::apply_exec_handoff(self, handoff)
    }

    fn recovery_state(&self) -> Result<SettlementRecoveryState, SettlementStoreError> {
        SettlementStore::recovery_state(self)
    }

    fn settlement_proof_item(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofItem, SettlementStoreError> {
        SettlementStore::settlement_proof_item(self, path)
    }

    fn settlement_proof_blob(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofBlob, SettlementStoreError> {
        SettlementStore::settlement_proof_blob(self, path)
    }

    fn settlement_proof_scan(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofScanOut, SettlementStoreError> {
        SettlementStore::settlement_proof_scan(self, path)
    }

    fn settlement_inclusion_batch_v1(
        &self,
        paths: &[SettlementPath],
    ) -> Result<crate::settlement::BatchProofBlobV1, SettlementStoreError> {
        SettlementStore::settlement_inclusion_batch_v1(self, paths)
    }

    fn settlement_nonexistence_batch_v1(
        &self,
        paths: &[SettlementPath],
        leaf_family: crate::settlement::SettlementLeafFamily,
    ) -> Result<crate::settlement::BatchProofBlobV1, SettlementStoreError> {
        SettlementStore::settlement_nonexistence_batch_v1(self, paths, leaf_family)
    }

    fn settlement_deletion_batch_v1(
        &self,
        paths: &[SettlementPath],
    ) -> Result<crate::settlement::BatchProofBlobV1, SettlementStoreError> {
        SettlementStore::settlement_deletion_batch_v1(self, paths)
    }

    fn create_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::create_right(self, path, leaf, ctx)
    }

    fn create_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::create_right_with_fee(self, path, leaf, ctx, envelope, actor)
    }

    fn transfer_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::transfer_right(self, path, leaf, ctx)
    }

    fn transfer_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::transfer_right_with_fee(self, path, leaf, ctx, envelope, actor)
    }

    fn consume_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::consume_right(self, path, ctx)
    }

    fn consume_right_with_fee(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::consume_right_with_fee(self, path, ctx, envelope, actor)
    }

    fn revoke_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::revoke_right(self, path, ctx)
    }

    fn revoke_right_with_fee(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::revoke_right_with_fee(self, path, ctx, envelope, actor)
    }

    fn expire_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::expire_right(self, path, ctx)
    }

    fn challenge_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::challenge_right(self, path, leaf, ctx)
    }

    fn challenge_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::challenge_right_with_fee(self, path, leaf, ctx, envelope, actor)
    }
}

impl Default for SettlementStore {
    fn default() -> Self {
        Self::new()
    }
}

impl From<StoreBackendError> for SettlementStoreError {
    fn from(err: StoreBackendError) -> Self {
        match err {
            StoreBackendError::UnsupportedGeneration(message) => {
                Self::UnsupportedGeneration(message)
            }
            other => Self::Backend(other.to_string()),
        }
    }
}

pub(super) fn next_ver(version: Version) -> Version {
    version.saturating_add(1)
}
