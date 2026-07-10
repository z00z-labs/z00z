use z00z_crypto::expert::hash_domain;
use z00z_crypto::hash_zk::hash_zk;
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::snapshot::PrepSnapshotId;

use super::ids::{CheckpointExecInputId, CheckpointId};
use crate::CheckpointError;

hash_domain!(StorCheckpointLinkDom, "z00z.storage.checkpoint.link", 1);

const LINK_BIND_VER: u8 = 1;
const LINK_BIND_LABEL: &str = "checkpoint_link_bind_v1";

/// Canonical checkpoint-link schema version.
///
/// # Examples
///
/// ```
/// use z00z_storage::checkpoint::CheckpointLinkVersion;
///
/// assert_eq!(CheckpointLinkVersion::CURRENT.as_u8(), 1);
/// ```
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckpointLinkVersion(u8);

impl CheckpointLinkVersion {
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

/// Canonical linkage between final artifact, snapshot, and execution input.
///
/// # Examples
///
/// ```
/// use z00z_storage::{
///     checkpoint::{CheckpointExecInputId, CheckpointId, CheckpointLink, CheckpointLinkVersion},
///     snapshot::PrepSnapshotId,
/// };
///
/// let link = CheckpointLink::new(
///     CheckpointLinkVersion::CURRENT,
///     CheckpointId::new([1u8; 32]),
///     PrepSnapshotId::new([2u8; 32]),
///     CheckpointExecInputId::new([3u8; 32]),
/// )?;
/// assert_eq!(link.version(), CheckpointLinkVersion::CURRENT);
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointLink {
    version: CheckpointLinkVersion,
    checkpoint_id: CheckpointId,
    #[serde(default)]
    prev_checkpoint_id: Option<CheckpointId>,
    prep_snapshot_id: PrepSnapshotId,
    exec_input_id: CheckpointExecInputId,
    #[serde(default)]
    link_bind_ver: u8,
    #[serde(default)]
    link_bind: [u8; 32],
}

impl CheckpointLink {
    pub fn new(
        version: CheckpointLinkVersion,
        checkpoint_id: CheckpointId,
        prep_snapshot_id: PrepSnapshotId,
        exec_input_id: CheckpointExecInputId,
    ) -> Result<Self, CheckpointError> {
        Self::with_prev(
            version,
            checkpoint_id,
            None,
            prep_snapshot_id,
            exec_input_id,
        )
    }

    pub fn with_prev(
        version: CheckpointLinkVersion,
        checkpoint_id: CheckpointId,
        prev_checkpoint_id: Option<CheckpointId>,
        prep_snapshot_id: PrepSnapshotId,
        exec_input_id: CheckpointExecInputId,
    ) -> Result<Self, CheckpointError> {
        check_link_ver(version)?;
        Ok(Self {
            version,
            checkpoint_id,
            prev_checkpoint_id,
            prep_snapshot_id,
            exec_input_id,
            link_bind_ver: LINK_BIND_VER,
            link_bind: link_bind(
                checkpoint_id,
                prev_checkpoint_id,
                prep_snapshot_id,
                exec_input_id,
            ),
        })
    }

    #[must_use]
    pub const fn version(&self) -> CheckpointLinkVersion {
        self.version
    }

    #[must_use]
    pub const fn checkpoint_id(&self) -> CheckpointId {
        self.checkpoint_id
    }

    #[must_use]
    pub const fn prev_checkpoint_id(&self) -> Option<CheckpointId> {
        self.prev_checkpoint_id
    }

    #[must_use]
    pub const fn prep_snapshot_id(&self) -> PrepSnapshotId {
        self.prep_snapshot_id
    }

    #[must_use]
    pub const fn exec_input_id(&self) -> CheckpointExecInputId {
        self.exec_input_id
    }

    #[must_use]
    pub const fn link_bind_ver(&self) -> u8 {
        self.link_bind_ver
    }

    #[must_use]
    pub const fn link_bind(&self) -> [u8; 32] {
        self.link_bind
    }

    fn check_bind(&self) -> Result<(), CheckpointError> {
        if self.link_bind_ver != LINK_BIND_VER {
            return Err(CheckpointError::LinkMix);
        }
        if self.link_bind
            != link_bind(
                self.checkpoint_id,
                self.prev_checkpoint_id,
                self.prep_snapshot_id,
                self.exec_input_id,
            )
        {
            return Err(CheckpointError::LinkMix);
        }
        Ok(())
    }
}

pub(crate) fn check_link_ver(version: CheckpointLinkVersion) -> Result<(), CheckpointError> {
    if version == CheckpointLinkVersion::CURRENT {
        return Ok(());
    }

    Err(CheckpointError::VersionMix)
}

pub(crate) fn encode_link_bin_checked(link: &CheckpointLink) -> Result<Vec<u8>, CheckpointError> {
    check_link_ver(link.version())?;
    link.check_bind()?;
    Ok(BincodeCodec.serialize(link)?)
}

pub(crate) fn decode_link_bin_checked(bytes: &[u8]) -> Result<CheckpointLink, CheckpointError> {
    let codec = BincodeCodec;
    let link = codec.deserialize::<CheckpointLink>(bytes)?;
    check_link_ver(link.version())?;
    link.check_bind()?;
    Ok(link)
}

pub(crate) fn encode_link_json_checked(link: &CheckpointLink) -> Result<Vec<u8>, CheckpointError> {
    check_link_ver(link.version())?;
    link.check_bind()?;
    Ok(JsonCodec.serialize_pretty(link)?)
}

pub(crate) fn decode_link_json_checked(bytes: &[u8]) -> Result<CheckpointLink, CheckpointError> {
    let codec = JsonCodec;
    let link = codec.deserialize::<CheckpointLink>(bytes)?;
    check_link_ver(link.version())?;
    link.check_bind()?;
    Ok(link)
}

fn link_bind(
    checkpoint_id: CheckpointId,
    prev_checkpoint_id: Option<CheckpointId>,
    prep_snapshot_id: PrepSnapshotId,
    exec_input_id: CheckpointExecInputId,
) -> [u8; 32] {
    let mut prev_bytes = [0u8; 33];
    if let Some(prev_checkpoint_id) = prev_checkpoint_id {
        prev_bytes[0] = 1;
        prev_bytes[1..].copy_from_slice(prev_checkpoint_id.as_bytes());
    }
    hash_zk::<StorCheckpointLinkDom>(
        LINK_BIND_LABEL,
        &[
            checkpoint_id.as_bytes(),
            &prev_bytes,
            prep_snapshot_id.as_bytes(),
            exec_input_id.as_bytes(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::{check_link_ver, CheckpointLink, CheckpointLinkVersion};
    use crate::{
        checkpoint::{CheckpointExecInputId, CheckpointId},
        snapshot::PrepSnapshotId,
        CheckpointError,
    };

    #[test]
    fn test_good_link_builds() {
        let link = CheckpointLink::new(
            CheckpointLinkVersion::CURRENT,
            CheckpointId::new([1u8; 32]),
            PrepSnapshotId::new([2u8; 32]),
            CheckpointExecInputId::new([3u8; 32]),
        )
        .expect("link");

        assert_eq!(link.version(), CheckpointLinkVersion::CURRENT);
        assert_eq!(link.prev_checkpoint_id(), None);
    }

    #[test]
    fn test_bad_link_ver_rejects() {
        let err = check_link_ver(CheckpointLinkVersion::new(9)).expect_err("bad link version");

        assert!(matches!(err, CheckpointError::VersionMix));
    }
}
