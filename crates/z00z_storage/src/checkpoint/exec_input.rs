use z00z_crypto::{expert::hash_domain, frame_bytes, hash_zk::hash_zk};
use z00z_utils::codec::{BincodeCodec, Codec};

use crate::{
    settlement::{
        CheckRoot, DefinitionId, SerialId, SettlementStateRoot, StoreItem, TerminalId, TerminalLeaf,
    },
    snapshot::PrepSnapshotId,
    CheckpointError,
};

hash_domain!(StorCheckpointExecDom, "z00z.storage.checkpoint.exec", 1);

const EXEC_TX_ROW_LABEL: &str = "checkpoint_exec_tx_row_v1";
const EXEC_TX_ROOT_LABEL: &str = "checkpoint_exec_tx_root_v1";
const EXEC_NOOP_ROOT_LABEL: &str = "checkpoint_exec_noop_v2";
const EXEC_TX_ROW_VER: u8 = 1;

/// Canonical execution-input schema version.
///
/// # Examples
///
/// ```
/// use z00z_storage::checkpoint::CheckpointExecVersion;
///
/// assert_eq!(CheckpointExecVersion::CURRENT.as_u8(), 1);
/// ```
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckpointExecVersion(u8);

impl CheckpointExecVersion {
    pub const CURRENT: Self = Self(1);
    /// Explicit, authority-gated recursive V2 empty-transition input.
    ///
    /// This is not a successor for ordinary checkpoint execution rows.  It
    /// can only be constructed by [`CheckpointExecInput::new_recursive_v2_noop`]
    /// and is accepted by the recursive V2 owner only when the pinned contract
    /// configuration names this exact version and mode.
    pub const RECURSIVE_V2_NOOP: Self = Self(2);

    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

/// Ordered input reference used by checkpoint replay.
///
/// # Examples
///
/// ```
/// use z00z_storage::settlement::TerminalId;
/// use z00z_storage::{
///     checkpoint::CheckpointInRef,
///     settlement::SerialId,
/// };
///
/// let item = CheckpointInRef::new(TerminalId::new([4u8; 32]), SerialId::new(5));
/// assert_eq!(item.serial_id().get(), 5);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointInRef {
    terminal_id: TerminalId,
    serial_id: SerialId,
}

impl CheckpointInRef {
    #[must_use]
    pub fn new(terminal_id: impl Into<TerminalId>, serial_id: SerialId) -> Self {
        Self {
            terminal_id: terminal_id.into(),
            serial_id,
        }
    }

    #[must_use]
    pub const fn terminal_id(&self) -> TerminalId {
        self.terminal_id
    }

    #[must_use]
    pub const fn serial_id(&self) -> SerialId {
        self.serial_id
    }
}

/// Ordered output record used by checkpoint replay.
///
/// # Examples
///
/// ```
/// use z00z_core::assets::AssetLeaf as CoreLeaf;
/// use z00z_storage::{
///     checkpoint::CheckpointExecOut,
///     settlement::{DefinitionId, TerminalLeaf},
/// };
///
/// let out = CheckpointExecOut::new(
///     DefinitionId::new([7u8; 32]),
///     TerminalLeaf::from(CoreLeaf::dummy_for_scan(9)),
/// )?;
/// assert_eq!(out.leaf().serial_id, 9);
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointExecOut {
    definition_id: DefinitionId,
    leaf: TerminalLeaf,
}

impl CheckpointExecOut {
    pub fn new(definition_id: DefinitionId, leaf: TerminalLeaf) -> Result<Self, CheckpointError> {
        let path = crate::settlement::SettlementPath::new(
            definition_id,
            SerialId::new(leaf.serial_id),
            TerminalId::new(leaf.asset_id),
        );
        StoreItem::new(path, leaf.clone()).map_err(|_| CheckpointError::ReplayMix)?;
        Ok(Self {
            definition_id,
            leaf,
        })
    }

    #[must_use]
    pub const fn definition_id(&self) -> DefinitionId {
        self.definition_id
    }

    #[must_use]
    pub fn leaf(&self) -> &TerminalLeaf {
        &self.leaf
    }
}

/// Ordered tx execution row used by checkpoint replay.
///
/// The `tx_proof` payload is the exact upstream proof byte sequence consumed by
/// verification. Canonical replay artifacts must preserve those bytes without
/// reconstructing or synthesizing a replacement later in the storage path.
///
/// # Examples
///
/// ```
/// use z00z_core::assets::AssetLeaf as CoreLeaf;
/// use z00z_storage::settlement::TerminalId;
/// use z00z_storage::{
///     checkpoint::{CheckpointExecTx, CheckpointExecOut, CheckpointInRef},
///     settlement::{DefinitionId, SerialId, TerminalLeaf},
/// };
///
/// let tx = CheckpointExecTx::new(
///     vec![CheckpointInRef::new(TerminalId::new([4u8; 32]), SerialId::new(5))],
///     vec![CheckpointExecOut::new(
///         DefinitionId::new([7u8; 32]),
///         TerminalLeaf::from(CoreLeaf::dummy_for_scan(9)),
///     )?],
///     vec![8u8],
/// )?;
/// assert_eq!(tx.input_refs().len(), 1);
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointExecTx {
    input_refs: Vec<CheckpointInRef>,
    outputs: Vec<CheckpointExecOut>,
    tx_proof: Vec<u8>,
}

impl CheckpointExecTx {
    pub fn new(
        input_refs: Vec<CheckpointInRef>,
        outputs: Vec<CheckpointExecOut>,
        tx_proof: Vec<u8>,
    ) -> Result<Self, CheckpointError> {
        if input_refs.is_empty() || outputs.is_empty() || tx_proof.is_empty() {
            return Err(CheckpointError::ReplayMix);
        }

        Ok(Self {
            input_refs,
            outputs,
            tx_proof,
        })
    }

    #[must_use]
    pub fn input_refs(&self) -> &[CheckpointInRef] {
        &self.input_refs
    }

    #[must_use]
    pub fn outputs(&self) -> &[CheckpointExecOut] {
        &self.outputs
    }

    #[must_use]
    pub fn tx_proof(&self) -> &[u8] {
        &self.tx_proof
    }
}

#[derive(serde::Serialize)]
struct ExecTxRowV1<'a> {
    version: u8,
    ordinal: u32,
    input_refs: &'a [CheckpointInRef],
    outputs: &'a [CheckpointExecOut],
    tx_proof: &'a [u8],
}

/// Derive the canonical transaction-data commitment for ordered execution rows.
pub fn derive_exec_tx_root(txs: &[CheckpointExecTx]) -> Result<[u8; 32], CheckpointError> {
    if txs.is_empty() {
        return Err(CheckpointError::ReplayMix);
    }

    let codec = BincodeCodec;
    let tx_count = u32::try_from(txs.len()).map_err(|_| CheckpointError::ReplayMix)?;
    let mut root_bytes = frame_bytes(&tx_count.to_le_bytes());
    for (ordinal, tx) in txs.iter().enumerate() {
        let ordinal = u32::try_from(ordinal).map_err(|_| CheckpointError::ReplayMix)?;
        let row = ExecTxRowV1 {
            version: EXEC_TX_ROW_VER,
            ordinal,
            input_refs: tx.input_refs(),
            outputs: tx.outputs(),
            tx_proof: tx.tx_proof(),
        };
        let row_bytes = codec.serialize(&row)?;
        let row_hash = hash_zk::<StorCheckpointExecDom>(EXEC_TX_ROW_LABEL, &[row_bytes.as_slice()]);
        root_bytes.extend_from_slice(&frame_bytes(&row_hash));
    }

    Ok(hash_zk::<StorCheckpointExecDom>(
        EXEC_TX_ROOT_LABEL,
        &[root_bytes.as_slice()],
    ))
}

/// Derive the only legal zero-row execution commitment.
///
/// The version, preparation snapshot, and V2 predecessor root are framed into
/// the commitment, so an empty vector cannot stand in for this typed marker.
fn derive_noop_exec_root_v2(
    prep_snapshot_id: PrepSnapshotId,
    prev_settlement_root: SettlementStateRoot,
) -> [u8; 32] {
    let version = CheckpointExecVersion::RECURSIVE_V2_NOOP.as_u8();
    let generation = prev_settlement_root.generation().version();
    let mut bytes = frame_bytes(&[version]);
    bytes.extend_from_slice(&frame_bytes(prep_snapshot_id.as_bytes()));
    bytes.extend_from_slice(&frame_bytes(&[generation]));
    bytes.extend_from_slice(&frame_bytes(prev_settlement_root.as_bytes()));
    hash_zk::<StorCheckpointExecDom>(EXEC_NOOP_ROOT_LABEL, &[bytes.as_slice()])
}

/// Canonical checkpoint execution-input artifact.
///
/// # Examples
///
/// ```
/// use z00z_core::assets::AssetLeaf as CoreLeaf;
/// use z00z_storage::settlement::TerminalId;
/// use z00z_storage::{
///     checkpoint::{CheckpointExecInput, CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion, CheckpointInRef},
///     settlement::{CheckRoot, DefinitionId, SerialId, TerminalLeaf},
///     snapshot::PrepSnapshotId,
/// };
///
/// let exec = CheckpointExecInput::new(
///     CheckpointExecVersion::CURRENT,
///     PrepSnapshotId::new([2u8; 32]),
///     CheckRoot::new([1u8; 32]),
///     vec![CheckpointExecTx::new(
///         vec![CheckpointInRef::new(TerminalId::new([3u8; 32]), SerialId::new(4))],
///         vec![CheckpointExecOut::new(
///             DefinitionId::new([5u8; 32]),
///             TerminalLeaf::from(CoreLeaf::dummy_for_scan(4)),
///         )?],
///         vec![6u8],
///     )?],
/// )?;
/// assert_eq!(exec.prev_root(), CheckRoot::new([1u8; 32]));
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointExecInput {
    version: CheckpointExecVersion,
    prep_snapshot_id: PrepSnapshotId,
    prev_root: CheckRoot,
    prev_settlement_root: SettlementStateRoot,
    tx_data_root: [u8; 32],
    txs: Vec<CheckpointExecTx>,
}

impl CheckpointExecInput {
    pub fn new(
        version: CheckpointExecVersion,
        prep_snapshot_id: PrepSnapshotId,
        prev_root: CheckRoot,
        txs: Vec<CheckpointExecTx>,
    ) -> Result<Self, CheckpointError> {
        Self::new_settlement(
            version,
            prep_snapshot_id,
            SettlementStateRoot::settlement_v1(prev_root.into_bytes()),
            txs,
        )
    }

    pub fn new_settlement(
        version: CheckpointExecVersion,
        prep_snapshot_id: PrepSnapshotId,
        prev_settlement_root: SettlementStateRoot,
        txs: Vec<CheckpointExecTx>,
    ) -> Result<Self, CheckpointError> {
        check_exec_ver(version)?;
        if version != CheckpointExecVersion::CURRENT {
            return Err(CheckpointError::VersionMix);
        }
        if txs.is_empty() {
            return Err(CheckpointError::ReplayMix);
        }
        let tx_data_root = derive_exec_tx_root(&txs)?;
        Ok(Self {
            version,
            prep_snapshot_id,
            prev_root: CheckRoot::from(prev_settlement_root),
            prev_settlement_root,
            tx_data_root,
            txs,
        })
    }

    /// Construct the explicit checkpoint input required for a canonical V2
    /// no-op transition. Ordinary execution constructors never accept empty
    /// transaction rows.
    pub fn new_recursive_v2_noop(
        prep_snapshot_id: PrepSnapshotId,
        prev_settlement_root: SettlementStateRoot,
    ) -> Result<Self, CheckpointError> {
        if prev_settlement_root.generation().version() != 2 {
            return Err(CheckpointError::ReplayMix);
        }
        let version = CheckpointExecVersion::RECURSIVE_V2_NOOP;
        let tx_data_root = derive_noop_exec_root_v2(prep_snapshot_id, prev_settlement_root);
        Ok(Self {
            version,
            prep_snapshot_id,
            prev_root: CheckRoot::from(prev_settlement_root),
            prev_settlement_root,
            tx_data_root,
            txs: Vec::new(),
        })
    }

    #[must_use]
    pub const fn version(&self) -> CheckpointExecVersion {
        self.version
    }

    #[must_use]
    pub const fn prep_snapshot_id(&self) -> PrepSnapshotId {
        self.prep_snapshot_id
    }

    #[must_use]
    pub const fn prev_root(&self) -> CheckRoot {
        self.prev_root
    }

    #[must_use]
    pub const fn prev_settlement_root(&self) -> SettlementStateRoot {
        self.prev_settlement_root
    }

    #[must_use]
    pub const fn tx_data_root(&self) -> [u8; 32] {
        self.tx_data_root
    }

    #[must_use]
    pub fn txs(&self) -> &[CheckpointExecTx] {
        &self.txs
    }

    #[must_use]
    pub const fn is_recursive_v2_noop(&self) -> bool {
        self.version.as_u8() == CheckpointExecVersion::RECURSIVE_V2_NOOP.as_u8()
    }

    pub(crate) fn expected_tx_data_root(&self) -> Result<[u8; 32], CheckpointError> {
        match self.version {
            CheckpointExecVersion::CURRENT => derive_exec_tx_root(&self.txs),
            CheckpointExecVersion::RECURSIVE_V2_NOOP if self.txs.is_empty() => Ok(
                derive_noop_exec_root_v2(self.prep_snapshot_id, self.prev_settlement_root),
            ),
            _ => Err(CheckpointError::ReplayMix),
        }
    }
}

pub(crate) fn check_exec_ver(version: CheckpointExecVersion) -> Result<(), CheckpointError> {
    if matches!(
        version,
        CheckpointExecVersion::CURRENT | CheckpointExecVersion::RECURSIVE_V2_NOOP
    ) {
        return Ok(());
    }

    Err(CheckpointError::VersionMix)
}

pub(crate) fn check_tx_root(exec: &CheckpointExecInput) -> Result<(), CheckpointError> {
    if exec.tx_data_root != exec.expected_tx_data_root()? {
        return Err(CheckpointError::ReplayMix);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use z00z_core::assets::AssetLeaf;

    use super::{
        check_exec_ver, CheckpointExecInput, CheckpointExecOut, CheckpointExecTx,
        CheckpointExecVersion, CheckpointInRef,
    };
    use crate::{
        settlement::{CheckRoot, DefinitionId, SerialId, TerminalLeaf},
        snapshot::PrepSnapshotId,
        CheckpointError,
    };

    fn exec_tx() -> CheckpointExecTx {
        CheckpointExecTx::new(
            vec![CheckpointInRef::new([2u8; 32], SerialId::new(7))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([3u8; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(7)),
            )
            .expect("exec out")],
            vec![4u8],
        )
        .expect("exec tx")
    }

    #[test]
    fn test_good_exec_input_builds() {
        let exec = CheckpointExecInput::new(
            CheckpointExecVersion::CURRENT,
            PrepSnapshotId::new([9u8; 32]),
            CheckRoot::new([1u8; 32]),
            vec![exec_tx()],
        )
        .expect("exec");

        assert_eq!(exec.txs().len(), 1);
    }

    #[test]
    fn test_empty_exec_tx_rejects() {
        let err = CheckpointExecTx::new(Vec::new(), Vec::new(), Vec::new())
            .expect_err("empty exec tx must reject");

        assert!(matches!(err, CheckpointError::ReplayMix));
    }

    #[test]
    fn test_bad_exec_ver_rejects() {
        let err = check_exec_ver(CheckpointExecVersion::new(9)).expect_err("bad exec version");

        assert!(matches!(err, CheckpointError::VersionMix));
    }
}
