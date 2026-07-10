use z00z_crypto::{expert::hash_domain, frame_bytes, frame_str, hash_zk::hash_zk};
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::CheckpointError;

use super::da_reference::CheckpointDaProviderFamily;

hash_domain!(
    StorCheckpointPublicationEvidenceDom,
    "z00z.storage.checkpoint.publication_evidence",
    1
);

const PUBLICATION_EVIDENCE_BIND_VER: u8 = 1;
const PUBLICATION_EVIDENCE_BIND_LABEL: &str = "checkpoint_publication_evidence_v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckpointPublicationEvidenceVersion(u8);

impl CheckpointPublicationEvidenceVersion {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointPublicationState {
    DaPublicationReady,
}

impl CheckpointPublicationState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DaPublicationReady => "da_publication_ready",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointPublicationEvidenceV1 {
    version: CheckpointPublicationEvidenceVersion,
    statement_core_digest: [u8; 32],
    da_ref: [u8; 32],
    archive_manifest_root: [u8; 32],
    payload_commitment: [u8; 32],
    publication_state: CheckpointPublicationState,
    provider_family: CheckpointDaProviderFamily,
    readiness_height: u64,
    challenge_window_start_height: u64,
    observations_root: [u8; 32],
    #[serde(default)]
    publication_evidence_bind_ver: u8,
    #[serde(default)]
    publication_evidence_bind: [u8; 32],
}

impl CheckpointPublicationEvidenceV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: CheckpointPublicationEvidenceVersion,
        statement_core_digest: [u8; 32],
        da_ref: [u8; 32],
        archive_manifest_root: [u8; 32],
        payload_commitment: [u8; 32],
        publication_state: CheckpointPublicationState,
        provider_family: CheckpointDaProviderFamily,
        readiness_height: u64,
        challenge_window_start_height: u64,
        observations_root: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        check_publication_evidence_ver(version)?;
        if readiness_height == 0
            || challenge_window_start_height == 0
            || challenge_window_start_height < readiness_height
            || is_zero_root(&statement_core_digest)
            || is_zero_root(&da_ref)
            || is_zero_root(&archive_manifest_root)
            || is_zero_root(&payload_commitment)
            || is_zero_root(&observations_root)
        {
            return Err(CheckpointError::ArchiveMix);
        }
        let publication_evidence_bind = publication_evidence_bind(
            statement_core_digest,
            da_ref,
            archive_manifest_root,
            payload_commitment,
            publication_state,
            provider_family,
            readiness_height,
            challenge_window_start_height,
            observations_root,
        );
        Ok(Self {
            version,
            statement_core_digest,
            da_ref,
            archive_manifest_root,
            payload_commitment,
            publication_state,
            provider_family,
            readiness_height,
            challenge_window_start_height,
            observations_root,
            publication_evidence_bind_ver: PUBLICATION_EVIDENCE_BIND_VER,
            publication_evidence_bind,
        })
    }

    #[must_use]
    pub const fn version(&self) -> CheckpointPublicationEvidenceVersion {
        self.version
    }

    #[must_use]
    pub const fn statement_core_digest(&self) -> [u8; 32] {
        self.statement_core_digest
    }

    #[must_use]
    pub const fn da_ref(&self) -> [u8; 32] {
        self.da_ref
    }

    #[must_use]
    pub const fn archive_manifest_root(&self) -> [u8; 32] {
        self.archive_manifest_root
    }

    #[must_use]
    pub const fn payload_commitment(&self) -> [u8; 32] {
        self.payload_commitment
    }

    #[must_use]
    pub const fn publication_state(&self) -> CheckpointPublicationState {
        self.publication_state
    }

    #[must_use]
    pub const fn provider_family(&self) -> CheckpointDaProviderFamily {
        self.provider_family
    }

    #[must_use]
    pub const fn readiness_height(&self) -> u64 {
        self.readiness_height
    }

    #[must_use]
    pub const fn challenge_window_start_height(&self) -> u64 {
        self.challenge_window_start_height
    }

    #[must_use]
    pub const fn observations_root(&self) -> [u8; 32] {
        self.observations_root
    }

    #[must_use]
    pub const fn publication_evidence_root(&self) -> [u8; 32] {
        self.publication_evidence_bind
    }

    fn check_bind(&self) -> Result<(), CheckpointError> {
        if self.publication_evidence_bind_ver != PUBLICATION_EVIDENCE_BIND_VER {
            return Err(CheckpointError::ArchiveMix);
        }
        if self.publication_evidence_bind
            != publication_evidence_bind(
                self.statement_core_digest,
                self.da_ref,
                self.archive_manifest_root,
                self.payload_commitment,
                self.publication_state,
                self.provider_family,
                self.readiness_height,
                self.challenge_window_start_height,
                self.observations_root,
            )
        {
            return Err(CheckpointError::ArchiveMix);
        }
        Ok(())
    }
}

pub(crate) fn check_publication_evidence_ver(
    version: CheckpointPublicationEvidenceVersion,
) -> Result<(), CheckpointError> {
    if version == CheckpointPublicationEvidenceVersion::CURRENT {
        return Ok(());
    }
    Err(CheckpointError::VersionMix)
}

pub(crate) fn encode_publication_evidence_bin_checked(
    evidence: &CheckpointPublicationEvidenceV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_publication_evidence_ver(evidence.version())?;
    evidence.check_bind()?;
    Ok(BincodeCodec.serialize(evidence)?)
}

pub(crate) fn decode_publication_evidence_bin_checked(
    bytes: &[u8],
) -> Result<CheckpointPublicationEvidenceV1, CheckpointError> {
    let evidence: CheckpointPublicationEvidenceV1 = BincodeCodec.deserialize(bytes)?;
    check_publication_evidence_ver(evidence.version())?;
    evidence.check_bind()?;
    Ok(evidence)
}

pub(crate) fn encode_publication_evidence_json_checked(
    evidence: &CheckpointPublicationEvidenceV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_publication_evidence_ver(evidence.version())?;
    evidence.check_bind()?;
    Ok(JsonCodec.serialize_pretty(evidence)?)
}

pub(crate) fn decode_publication_evidence_json_checked(
    bytes: &[u8],
) -> Result<CheckpointPublicationEvidenceV1, CheckpointError> {
    let evidence: CheckpointPublicationEvidenceV1 = JsonCodec.deserialize(bytes)?;
    check_publication_evidence_ver(evidence.version())?;
    evidence.check_bind()?;
    Ok(evidence)
}

#[allow(clippy::too_many_arguments)]
fn publication_evidence_bind(
    statement_core_digest: [u8; 32],
    da_ref: [u8; 32],
    archive_manifest_root: [u8; 32],
    payload_commitment: [u8; 32],
    publication_state: CheckpointPublicationState,
    provider_family: CheckpointDaProviderFamily,
    readiness_height: u64,
    challenge_window_start_height: u64,
    observations_root: [u8; 32],
) -> [u8; 32] {
    let mut bytes = Vec::new();
    push_framed_field(&mut bytes, "statement_core_digest", &statement_core_digest);
    push_framed_field(&mut bytes, "da_ref", &da_ref);
    push_framed_field(&mut bytes, "archive_manifest_root", &archive_manifest_root);
    push_framed_field(&mut bytes, "payload_commitment", &payload_commitment);
    push_framed_field(
        &mut bytes,
        "publication_state",
        publication_state.as_str().as_bytes(),
    );
    push_framed_field(
        &mut bytes,
        "provider_family",
        provider_family.as_str().as_bytes(),
    );
    push_framed_field(
        &mut bytes,
        "readiness_height",
        &readiness_height.to_le_bytes(),
    );
    push_framed_field(
        &mut bytes,
        "challenge_window_start_height",
        &challenge_window_start_height.to_le_bytes(),
    );
    push_framed_field(&mut bytes, "observations_root", &observations_root);
    hash_zk::<StorCheckpointPublicationEvidenceDom>(
        PUBLICATION_EVIDENCE_BIND_LABEL,
        &[bytes.as_slice()],
    )
}

fn push_framed_field(out: &mut Vec<u8>, name: &str, value: &[u8]) {
    out.extend_from_slice(&frame_str(name));
    out.extend_from_slice(&frame_bytes(value));
}

fn is_zero_root(root: &[u8; 32]) -> bool {
    root.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::{
        check_publication_evidence_ver, CheckpointPublicationEvidenceV1,
        CheckpointPublicationEvidenceVersion, CheckpointPublicationState,
    };
    use crate::{checkpoint::da_reference::CheckpointDaProviderFamily, CheckpointError};

    fn root(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn evidence() -> CheckpointPublicationEvidenceV1 {
        CheckpointPublicationEvidenceV1::new(
            CheckpointPublicationEvidenceVersion::CURRENT,
            root(1),
            root(2),
            root(3),
            root(4),
            CheckpointPublicationState::DaPublicationReady,
            CheckpointDaProviderFamily::LocalArchive,
            55,
            55,
            root(5),
        )
        .expect("publication evidence")
    }

    #[test]
    fn test_publication_evidence_builds() {
        let got = evidence();

        assert_eq!(
            got.publication_state(),
            CheckpointPublicationState::DaPublicationReady
        );
        assert_eq!(got.readiness_height(), 55);
        assert_ne!(got.publication_evidence_root(), [0u8; 32]);
    }

    #[test]
    fn test_publication_evidence_rejects_early_window() {
        let err = CheckpointPublicationEvidenceV1::new(
            CheckpointPublicationEvidenceVersion::CURRENT,
            root(1),
            root(2),
            root(3),
            root(4),
            CheckpointPublicationState::DaPublicationReady,
            CheckpointDaProviderFamily::LocalArchive,
            55,
            54,
            root(5),
        )
        .expect_err("early challenge window must reject");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_publication_evidence_bad_version() {
        let err = check_publication_evidence_ver(CheckpointPublicationEvidenceVersion::new(9))
            .expect_err("bad version must reject");

        assert!(matches!(err, CheckpointError::VersionMix));
    }
}
