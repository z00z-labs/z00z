use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use z00z_utils::io::{create_dir_all, read_file, write_file};

use crate::checkpoint::CheckpointContractConfigV1;
use crate::settlement::{
    chk_blob_settlement_inclusion, CheckRoot, HjmtProofFamily, ModelErr, ProofBlob, ProofChkErr,
    ProofItem, RootGeneration, SettlementStateRoot, SettlementStore, SettlementStoreError,
    SnapItem, StoreItem, StoreOp,
};

use super::{
    codec::{decode_snap, derive_id, encode_snap},
    PrepSnapshot, PrepSnapshotError, PrepSnapshotId, PrepSnapshotVersion,
};

/// Build one validated canonical snapshot and derive its external id.
///
/// # Examples
///
/// ```
/// use z00z_storage::{
///     snapshot::{build_snapshot, PrepSnapshotVersion},
///     settlement::{SettlementStore, CheckRoot, SnapItem},
/// };
///
/// let prev_root = CheckRoot::from(
///     SettlementStore::new()
///         .settlement_root()
///         .map_err(|err| z00z_storage::snapshot::PrepSnapshotError::Backend(err.to_string()))?,
/// );
/// let (snapshot, _snap_id) = build_snapshot(prev_root, Vec::<SnapItem>::new())?;
/// assert_eq!(snapshot.version, PrepSnapshotVersion::CURRENT);
/// # Ok::<(), z00z_storage::snapshot::PrepSnapshotError>(())
/// ```
pub fn build_snapshot(
    prev_root: CheckRoot,
    entries: Vec<SnapItem>,
) -> Result<(PrepSnapshot, PrepSnapshotId), PrepSnapshotError> {
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, prev_root, entries);
    check_snapshot(&snapshot)?;
    let snapshot_id = derive_id(&snapshot)?;
    Ok((snapshot, snapshot_id))
}

/// Build one canonical prep snapshot bound to the typed V2 settlement root.
///
/// Unlike the legacy builder, this records the root generation alongside the
/// raw checkpoint field. Every nonempty entry must carry a proof for that exact
/// typed root; the V2 transition resolver subsequently compares it to the live
/// settlement-store snapshot before any witness pass begins.
pub fn build_snapshot_v2(
    prev_settlement_root: SettlementStateRoot,
    entries: Vec<SnapItem>,
) -> Result<(PrepSnapshot, PrepSnapshotId), PrepSnapshotError> {
    if prev_settlement_root.generation() != RootGeneration::SettlementV2 {
        return Err(PrepSnapshotError::RootMix);
    }
    let snapshot = PrepSnapshot::new_settlement_v2(
        PrepSnapshotVersion::CURRENT,
        prev_settlement_root,
        entries,
    );
    check_snapshot(&snapshot)?;
    let snapshot_id = derive_id(&snapshot)?;
    Ok((snapshot, snapshot_id))
}

pub(crate) fn build_snapshot_bin(
    prev_root: CheckRoot,
    entries: Vec<SnapItem>,
) -> Result<(PrepSnapshot, PrepSnapshotId, Vec<u8>), PrepSnapshotError> {
    let (snapshot, snapshot_id) = build_snapshot(prev_root, entries)?;
    let bytes = encode_snap(&snapshot)?;
    Ok((snapshot, snapshot_id, bytes))
}

/// Storage-owned replay entry returned by the canonical snapshot accessor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrepReplayEntry {
    item: SnapItem,
    proof_item: ProofItem,
}

impl PrepReplayEntry {
    #[must_use]
    const fn new(item: SnapItem, proof_item: ProofItem) -> Self {
        Self { item, proof_item }
    }

    /// Decode one storage-owned witness blob and bind it back to the canonical entry.
    pub(crate) fn from_blob(item: SnapItem) -> Result<Self, PrepSnapshotError> {
        let blob = ProofBlob::decode(item.wit()).map_err(PrepSnapshotError::WitDecode)?;
        if blob.item().path() != item.path() {
            return Err(PrepSnapshotError::ReplayPathMix);
        }
        if blob.item().leaf() != item.leaf() {
            return Err(PrepSnapshotError::ReplayLeafMix);
        }
        Ok(Self::new(item, blob.item().clone()))
    }

    /// Borrow the canonical snapshot item.
    #[must_use]
    pub const fn item(&self) -> &SnapItem {
        &self.item
    }

    /// Borrow the decoded typed proof context.
    #[must_use]
    pub const fn proof_item(&self) -> &ProofItem {
        &self.proof_item
    }
}

/// Narrow storage-facing facade for canonical snapshot operations.
///
/// Implementations must keep serialization on `z00z_utils::codec::Codec` and
/// persistence on `z00z_utils::io` helpers instead of ad-hoc local codecs or
/// direct `std::fs` calls.
///
/// # Examples
///
/// ```
/// use z00z_storage::snapshot::PrepSnapshotStore;
///
/// fn accept_store<T: PrepSnapshotStore>(_store: &T) {}
/// ```
pub trait PrepSnapshotStore {
    /// Persist one canonical snapshot artifact and return its external id.
    fn save_snapshot(
        &mut self,
        snapshot: &PrepSnapshot,
    ) -> Result<PrepSnapshotId, PrepSnapshotError>;

    /// Load one canonical snapshot artifact by its external id.
    fn load_snapshot(
        &self,
        snapshot_id: &PrepSnapshotId,
    ) -> Result<PrepSnapshot, PrepSnapshotError>;

    /// Validate one canonical snapshot artifact.
    fn validate_snapshot(&self, snapshot: &PrepSnapshot) -> Result<(), PrepSnapshotError>;

    /// Derive one external snapshot id from canonical artifact bytes.
    fn derive_snapshot_id(
        &self,
        snapshot: &PrepSnapshot,
    ) -> Result<PrepSnapshotId, PrepSnapshotError>;

    /// Materialize replay-ready entries from one canonical snapshot artifact.
    fn replay_entries(
        &self,
        snapshot: &PrepSnapshot,
    ) -> Result<Vec<PrepReplayEntry>, PrepSnapshotError>;
}

/// File-backed canonical snapshot store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrepFsStore {
    snap_dir: PathBuf,
}

impl PrepFsStore {
    /// Build one file-backed snapshot store rooted at the given directory.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self::try_new(root).expect("repo checkpoint contract must validate snapshot paths")
    }

    /// Build one file-backed snapshot store rooted at the repository contract
    /// paths for the given lane root.
    pub fn try_new(root: impl Into<PathBuf>) -> Result<Self, PrepSnapshotError> {
        let cfg = CheckpointContractConfigV1::load_repo_default()
            .map_err(|err| PrepSnapshotError::Backend(err.to_string()))?;
        let resolved = cfg.resolve_paths(root.into());
        Ok(Self::with_snapshot_dir(resolved.prep_snapshots))
    }

    /// Build one file-backed snapshot store for an exact already-validated
    /// snapshot directory.
    #[must_use]
    pub fn with_snapshot_dir(snap_dir: impl Into<PathBuf>) -> Self {
        Self {
            snap_dir: snap_dir.into(),
        }
    }

    #[must_use]
    pub fn snapshot_dir(&self) -> &Path {
        &self.snap_dir
    }

    pub(super) fn snap_path(&self, snapshot_id: &PrepSnapshotId) -> PathBuf {
        self.snap_dir.join(format!("{}.bin", id_hex(snapshot_id)))
    }
}

impl PrepSnapshotStore for PrepFsStore {
    fn save_snapshot(
        &mut self,
        snapshot: &PrepSnapshot,
    ) -> Result<PrepSnapshotId, PrepSnapshotError> {
        self.validate_snapshot(snapshot)?;
        let snapshot_id = self.derive_snapshot_id(snapshot)?;
        let bytes = encode_snap(snapshot)?;
        create_dir_all(&self.snap_dir)?;
        write_file(self.snap_path(&snapshot_id), &bytes)?;

        let reloaded = self.load_snapshot(&snapshot_id)?;
        let reloaded_id = self.derive_snapshot_id(&reloaded)?;
        if reloaded_id != snapshot_id {
            return Err(PrepSnapshotError::IdMix);
        }

        Ok(snapshot_id)
    }

    fn load_snapshot(
        &self,
        snapshot_id: &PrepSnapshotId,
    ) -> Result<PrepSnapshot, PrepSnapshotError> {
        let bytes = read_file(self.snap_path(snapshot_id))?;
        let snapshot = decode_snap(&bytes)?;
        self.validate_snapshot(&snapshot)?;

        if derive_id(&snapshot)? != *snapshot_id {
            return Err(PrepSnapshotError::IdMix);
        }

        Ok(snapshot)
    }

    fn validate_snapshot(&self, snapshot: &PrepSnapshot) -> Result<(), PrepSnapshotError> {
        check_snapshot(snapshot)
    }

    fn derive_snapshot_id(
        &self,
        snapshot: &PrepSnapshot,
    ) -> Result<PrepSnapshotId, PrepSnapshotError> {
        self.validate_snapshot(snapshot)?;
        derive_id(snapshot)
    }

    fn replay_entries(
        &self,
        snapshot: &PrepSnapshot,
    ) -> Result<Vec<PrepReplayEntry>, PrepSnapshotError> {
        self.validate_snapshot(snapshot)?;
        snapshot
            .entries
            .iter()
            .cloned()
            .map(PrepReplayEntry::from_blob)
            .collect()
    }
}

fn check_snapshot(snapshot: &PrepSnapshot) -> Result<(), PrepSnapshotError> {
    if snapshot.version != PrepSnapshotVersion::CURRENT {
        return Err(PrepSnapshotError::VersionMix);
    }

    let mut paths = BTreeSet::new();
    let mut terminal_ids = BTreeSet::new();

    for entry in &snapshot.entries {
        check_entry(snapshot.prev_root, snapshot.prev_settlement_root, entry)?;

        if !paths.insert(entry.path()) {
            return Err(PrepSnapshotError::DupPath);
        }
        if !terminal_ids.insert(entry.path().terminal_id()) {
            return Err(PrepSnapshotError::DupTerminalId);
        }
    }

    check_entries_root(snapshot)?;

    Ok(())
}

fn check_entries_root(snapshot: &PrepSnapshot) -> Result<(), PrepSnapshotError> {
    if let Some(root) = snapshot.prev_settlement_root {
        if root.generation() != RootGeneration::SettlementV2
            || CheckRoot::from(root) != snapshot.prev_root
        {
            return Err(PrepSnapshotError::RootMix);
        }
        // Entry-level inclusion checks above bind every materialized leaf to
        // the exact V2 root. The V2 root itself is derived by the live HJMT
        // store and is rechecked by the recursive transition resolver; do not
        // reconstruct a second legacy semantic root here.
        return Ok(());
    }
    #[cfg(test)]
    let mut store = SettlementStore::test_hjmt_store();
    #[cfg(not(test))]
    let mut store = SettlementStore::transient_hjmt().map_err(map_store)?;
    let mut ops = Vec::with_capacity(snapshot.entries.len());
    for entry in &snapshot.entries {
        let item = StoreItem::new(entry.path(), entry.leaf().clone()).map_err(map_model)?;
        ops.push(StoreOp::Put(Box::new(item)));
    }

    let root = if ops.is_empty() {
        store.settlement_root().map_err(map_store)?
    } else {
        store.apply_settlement_ops(ops).map_err(map_store)?
    };
    if CheckRoot::from(root) != snapshot.prev_root {
        return Err(PrepSnapshotError::RootMix);
    }

    Ok(())
}

fn check_entry(
    prev_root: CheckRoot,
    prev_settlement_root: Option<SettlementStateRoot>,
    entry: &SnapItem,
) -> Result<(), PrepSnapshotError> {
    entry.check_path().map_err(map_model)?;

    let blob = ProofBlob::decode(entry.wit()).map_err(PrepSnapshotError::WitDecode)?;
    if blob
        .hjmt_proof_family()
        .is_some_and(|family| family != HjmtProofFamily::Inclusion)
    {
        return Err(PrepSnapshotError::WitMix);
    }
    let proof_item = blob.item();
    let proof_path = proof_item.path();

    if CheckRoot::from(proof_item.settlement_root()) != prev_root
        || prev_settlement_root.is_some_and(|root| proof_item.settlement_root() != root)
    {
        return Err(PrepSnapshotError::RootMix);
    }
    if proof_path.definition_id != entry.path().definition_id {
        return Err(PrepSnapshotError::PathMix);
    }
    if proof_path.serial_id != entry.path().serial_id {
        return Err(PrepSnapshotError::SerialMix);
    }
    if proof_path.terminal_id() != entry.path().terminal_id() {
        return Err(PrepSnapshotError::TerminalIdMix);
    }
    if proof_item.leaf().terminal_id() != entry.leaf().terminal_id() {
        return Err(PrepSnapshotError::TerminalIdMix);
    }
    if proof_item.leaf().serial_id() != entry.leaf().serial_id() {
        return Err(PrepSnapshotError::SerialMix);
    }
    if proof_item.leaf() != entry.leaf() {
        return Err(PrepSnapshotError::LeafMix);
    }

    chk_blob_settlement_inclusion(
        entry.wit(),
        proof_item.settlement_root(),
        &entry.path(),
        proof_item.def_leaf(),
        proof_item.ser_leaf(),
        entry.leaf(),
    )
    .map_err(map_wit)?;

    Ok(())
}

fn map_model(err: ModelErr) -> PrepSnapshotError {
    match err {
        ModelErr::Codec(codec) => PrepSnapshotError::Codec(codec),
        ModelErr::PathDefMix | ModelErr::NoDef | ModelErr::NoSerial | ModelErr::NoTerminal => {
            PrepSnapshotError::PathMix
        }
        ModelErr::PathSerMix | ModelErr::PathSerialMix => PrepSnapshotError::SerialMix,
        ModelErr::PathLeafMix => PrepSnapshotError::TerminalIdMix,
        ModelErr::Right(_) | ModelErr::WrongLeafFamily => PrepSnapshotError::LeafMix,
    }
}

fn map_wit(err: ProofChkErr) -> PrepSnapshotError {
    match err {
        ProofChkErr::Codec(codec) => PrepSnapshotError::WitDecode(codec),
        ProofChkErr::RootMix
        | ProofChkErr::RootGenerationMix
        | ProofChkErr::BindVerMix
        | ProofChkErr::RootBindMix => PrepSnapshotError::RootMix,
        ProofChkErr::PathMix => PrepSnapshotError::PathMix,
        ProofChkErr::DefMix => PrepSnapshotError::PathMix,
        ProofChkErr::SerMix => PrepSnapshotError::SerialMix,
        ProofChkErr::LeafMix => PrepSnapshotError::LeafMix,
        ProofChkErr::LeafHashMix => PrepSnapshotError::LeafHashMix,
        ProofChkErr::DefProofMix
        | ProofChkErr::SerProofMix
        | ProofChkErr::TerminalProofMix
        | ProofChkErr::ProofFamilyMix
        | ProofChkErr::DefaultCommitmentMix
        | ProofChkErr::JournalCheckpointMix
        | ProofChkErr::UnsupportedHjmtProofVersion
        | ProofChkErr::BucketPolicyMix
        | ProofChkErr::BucketMix
        | ProofChkErr::BucketProofMix
        | ProofChkErr::PriorRootMix
        | ProofChkErr::PriorDefMix
        | ProofChkErr::PriorSerMix
        | ProofChkErr::PriorBucketMix
        | ProofChkErr::PriorDefProofMix
        | ProofChkErr::PriorSerProofMix
        | ProofChkErr::PriorBucketProofMix
        | ProofChkErr::PriorTerminalProofMix
        | ProofChkErr::UnsupportedBatchProofVersion
        | ProofChkErr::BatchTrunc
        | ProofChkErr::BatchTrailingBytes
        | ProofChkErr::BatchLimitMix
        | ProofChkErr::BatchTagMix
        | ProofChkErr::BatchBoolMix
        | ProofChkErr::BatchTranscriptMix
        | ProofChkErr::BatchOpeningKindMix
        | ProofChkErr::BatchIndexMix
        | ProofChkErr::BatchOrderMix
        | ProofChkErr::BatchPolicyMix
        | ProofChkErr::BatchCheckpointMix
        | ProofChkErr::BatchBindVerMix
        | ProofChkErr::BatchRootBindMix
        | ProofChkErr::BatchWitnessStepMix
        | ProofChkErr::BatchSubtreeMix
        | ProofChkErr::BatchDupPath
        | ProofChkErr::BatchWitnessDomainMix
        | ProofChkErr::BatchHashCountMix
        | ProofChkErr::PublicationTrunc
        | ProofChkErr::PublicationTrailingBytes
        | ProofChkErr::PublicationModeMix
        | ProofChkErr::PublicationOrderMix
        | ProofChkErr::PublicationDupShard
        | ProofChkErr::PublicationFlagMix
        | ProofChkErr::PublicationCountMix
        | ProofChkErr::PublicationRouteMix
        | ProofChkErr::PublicationCheckpointMix
        | ProofChkErr::PublicationMonotonicityMix
        | ProofChkErr::PublicationPolicyMix
        | ProofChkErr::PublicationProofIndexMix
        | ProofChkErr::PublicationProofRouteMix
        | ProofChkErr::PublicationProofCheckpointMix
        | ProofChkErr::PublicationProofPolicyMix => PrepSnapshotError::WitMix,
        ProofChkErr::BatchRootGenerationMix | ProofChkErr::BatchRootMix => {
            PrepSnapshotError::RootMix
        }
        ProofChkErr::PublicationRootGenerationMix
        | ProofChkErr::PublicationPriorRootMix
        | ProofChkErr::PublicationProofGenerationMix => PrepSnapshotError::RootMix,
        ProofChkErr::BatchPathMix
        | ProofChkErr::BatchShardCtxMix
        | ProofChkErr::PublicationProofShardMix => PrepSnapshotError::PathMix,
        ProofChkErr::BatchLeafFamilyMix => PrepSnapshotError::LeafMix,
        ProofChkErr::BatchDefaultCommitmentMix
        | ProofChkErr::UnsupportedJmtUpdateVersion
        | ProofChkErr::JmtUpdateTraceLimit
        | ProofChkErr::JmtUpdateTraceCanonical
        | ProofChkErr::JmtUpdateProofMix => PrepSnapshotError::WitMix,
    }
}

fn map_store(err: SettlementStoreError) -> PrepSnapshotError {
    match err {
        SettlementStoreError::Codec(codec) => PrepSnapshotError::Codec(codec),
        other => PrepSnapshotError::Backend(other.to_string()),
    }
}

fn id_hex(snapshot_id: &PrepSnapshotId) -> String {
    let mut out = String::with_capacity(64);
    for byte in snapshot_id.as_bytes() {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}
