use z00z_crypto::expert::hash_domain;
use z00z_crypto::hash_zk::hash_zk;
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::CheckpointError;

hash_domain!(
    StorCheckpointStateSnapshotDom,
    "z00z.storage.checkpoint.state_snapshot",
    1
);

const STATE_SNAPSHOT_BIND_VER: u8 = 1;
const STATE_SNAPSHOT_BIND_LABEL: &str = "state_snapshot_v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct StateSnapshotVersion(u8);

impl StateSnapshotVersion {
    pub const CURRENT: Self = Self(1);

    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StateSnapshotV1 {
    version: StateSnapshotVersion,
    height: u64,
    cadence_epochs: u64,
    cadence_blocks: u64,
    state_root: [u8; 32],
    settlement_root: [u8; 32],
    last_plonky3_epoch_proof_digest: [u8; 32],
    last_epoch_manifest_root: [u8; 32],
    archive_manifest_root: [u8; 32],
    snapshot_chunk_root: [u8; 32],
    pq_anchor_root: [u8; 32],
    retrieval_audit_root: [u8; 32],
    #[serde(default)]
    state_snapshot_bind_ver: u8,
    #[serde(default)]
    state_snapshot_bind: [u8; 32],
}

impl StateSnapshotV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: StateSnapshotVersion,
        height: u64,
        cadence_epochs: u64,
        cadence_blocks: u64,
        state_root: [u8; 32],
        settlement_root: [u8; 32],
        last_plonky3_epoch_proof_digest: [u8; 32],
        last_epoch_manifest_root: [u8; 32],
        archive_manifest_root: [u8; 32],
        snapshot_chunk_root: [u8; 32],
        pq_anchor_root: [u8; 32],
        retrieval_audit_root: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        check_state_snapshot_ver(version)?;
        if height == 0
            || cadence_epochs == 0
            || cadence_blocks == 0
            || !height.is_multiple_of(cadence_blocks)
            || [
                state_root,
                settlement_root,
                last_plonky3_epoch_proof_digest,
                last_epoch_manifest_root,
                archive_manifest_root,
                snapshot_chunk_root,
                pq_anchor_root,
                retrieval_audit_root,
            ]
            .iter()
            .any(is_zero_root)
        {
            return Err(CheckpointError::SnapshotMix);
        }
        let state_snapshot_bind = state_snapshot_bind(
            height,
            cadence_epochs,
            cadence_blocks,
            state_root,
            settlement_root,
            last_plonky3_epoch_proof_digest,
            last_epoch_manifest_root,
            archive_manifest_root,
            snapshot_chunk_root,
            pq_anchor_root,
            retrieval_audit_root,
        );
        Ok(Self {
            version,
            height,
            cadence_epochs,
            cadence_blocks,
            state_root,
            settlement_root,
            last_plonky3_epoch_proof_digest,
            last_epoch_manifest_root,
            archive_manifest_root,
            snapshot_chunk_root,
            pq_anchor_root,
            retrieval_audit_root,
            state_snapshot_bind_ver: STATE_SNAPSHOT_BIND_VER,
            state_snapshot_bind,
        })
    }

    #[must_use]
    pub const fn version(&self) -> StateSnapshotVersion {
        self.version
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub const fn cadence_epochs(&self) -> u64 {
        self.cadence_epochs
    }

    #[must_use]
    pub const fn cadence_blocks(&self) -> u64 {
        self.cadence_blocks
    }

    #[must_use]
    pub const fn last_plonky3_epoch_proof_digest(&self) -> [u8; 32] {
        self.last_plonky3_epoch_proof_digest
    }

    #[must_use]
    pub const fn archive_manifest_root(&self) -> [u8; 32] {
        self.archive_manifest_root
    }

    fn check_bind(&self) -> Result<(), CheckpointError> {
        if self.state_snapshot_bind_ver != STATE_SNAPSHOT_BIND_VER {
            return Err(CheckpointError::SnapshotMix);
        }
        if self.state_snapshot_bind
            != state_snapshot_bind(
                self.height,
                self.cadence_epochs,
                self.cadence_blocks,
                self.state_root,
                self.settlement_root,
                self.last_plonky3_epoch_proof_digest,
                self.last_epoch_manifest_root,
                self.archive_manifest_root,
                self.snapshot_chunk_root,
                self.pq_anchor_root,
                self.retrieval_audit_root,
            )
        {
            return Err(CheckpointError::SnapshotMix);
        }
        Ok(())
    }
}

pub(crate) fn check_state_snapshot_ver(
    version: StateSnapshotVersion,
) -> Result<(), CheckpointError> {
    if version == StateSnapshotVersion::CURRENT {
        return Ok(());
    }
    Err(CheckpointError::VersionMix)
}

pub(crate) fn encode_state_snapshot_bin_checked(
    snapshot: &StateSnapshotV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_state_snapshot_ver(snapshot.version())?;
    snapshot.check_bind()?;
    Ok(BincodeCodec.serialize(snapshot)?)
}

pub(crate) fn decode_state_snapshot_bin_checked(
    bytes: &[u8],
) -> Result<StateSnapshotV1, CheckpointError> {
    let snapshot: StateSnapshotV1 = BincodeCodec.deserialize(bytes)?;
    check_state_snapshot_ver(snapshot.version())?;
    snapshot.check_bind()?;
    Ok(snapshot)
}

pub(crate) fn encode_state_snapshot_json_checked(
    snapshot: &StateSnapshotV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_state_snapshot_ver(snapshot.version())?;
    snapshot.check_bind()?;
    Ok(JsonCodec.serialize_pretty(snapshot)?)
}

pub(crate) fn decode_state_snapshot_json_checked(
    bytes: &[u8],
) -> Result<StateSnapshotV1, CheckpointError> {
    let snapshot: StateSnapshotV1 = JsonCodec.deserialize(bytes)?;
    check_state_snapshot_ver(snapshot.version())?;
    snapshot.check_bind()?;
    Ok(snapshot)
}

#[allow(clippy::too_many_arguments)]
fn state_snapshot_bind(
    height: u64,
    cadence_epochs: u64,
    cadence_blocks: u64,
    state_root: [u8; 32],
    settlement_root: [u8; 32],
    last_plonky3_epoch_proof_digest: [u8; 32],
    last_epoch_manifest_root: [u8; 32],
    archive_manifest_root: [u8; 32],
    snapshot_chunk_root: [u8; 32],
    pq_anchor_root: [u8; 32],
    retrieval_audit_root: [u8; 32],
) -> [u8; 32] {
    let height = height.to_le_bytes();
    let cadence_epochs = cadence_epochs.to_le_bytes();
    let cadence_blocks = cadence_blocks.to_le_bytes();
    hash_zk::<StorCheckpointStateSnapshotDom>(
        STATE_SNAPSHOT_BIND_LABEL,
        &[
            &height,
            &cadence_epochs,
            &cadence_blocks,
            &state_root,
            &settlement_root,
            &last_plonky3_epoch_proof_digest,
            &last_epoch_manifest_root,
            &archive_manifest_root,
            &snapshot_chunk_root,
            &pq_anchor_root,
            &retrieval_audit_root,
        ],
    )
}

fn is_zero_root(root: &[u8; 32]) -> bool {
    root.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::{check_state_snapshot_ver, StateSnapshotV1, StateSnapshotVersion};
    use crate::CheckpointError;

    fn root(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn snapshot() -> StateSnapshotV1 {
        StateSnapshotV1::new(
            StateSnapshotVersion::CURRENT,
            10_000,
            5,
            10_000,
            root(1),
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
        )
        .expect("snapshot")
    }

    #[test]
    fn test_state_snapshot_builds() {
        let snapshot = snapshot();

        assert_eq!(snapshot.height(), 10_000);
        assert_eq!(snapshot.cadence_epochs(), 5);
    }

    #[test]
    fn test_snapshot_plonky3_missing() {
        let err = StateSnapshotV1::new(
            StateSnapshotVersion::CURRENT,
            10_000,
            10,
            10_000,
            root(1),
            root(2),
            [0u8; 32],
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
        )
        .expect_err("missing plonky3 digest rejects");

        assert!(matches!(err, CheckpointError::SnapshotMix));
    }

    #[test]
    fn test_snapshot_requires_cadence_height() {
        let err = StateSnapshotV1::new(
            StateSnapshotVersion::CURRENT,
            9_999,
            5,
            10_000,
            root(1),
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
        )
        .expect_err("non cadence height rejects");

        assert!(matches!(err, CheckpointError::SnapshotMix));
    }

    #[test]
    fn test_snapshot_bad_version() {
        let err = check_state_snapshot_ver(StateSnapshotVersion::new(9)).expect_err("bad version");

        assert!(matches!(err, CheckpointError::VersionMix));
    }
}
