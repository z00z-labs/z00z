use crate::CheckpointError;

use super::{
    derive_checkpoint_id, CheckpointArtifact, CheckpointId, CheckpointLink,
    CheckpointPublicationEvidenceV1, CheckpointPublicationState,
};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckpointLifecycleVersion(u8);

impl CheckpointLifecycleVersion {
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
pub enum CheckpointLifecycleStatus {
    Sealed,
    Linked,
    PublicationReady,
    ChallengeOpen,
    Finalized,
    Disputed,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointLifecycleV1 {
    version: CheckpointLifecycleVersion,
    checkpoint_id: CheckpointId,
    prev_checkpoint_id: Option<CheckpointId>,
    #[serde(default)]
    statement_core_digest: Option<[u8; 32]>,
    status: CheckpointLifecycleStatus,
    publication_evidence_root: Option<[u8; 32]>,
    challenge_window_start_height: Option<u64>,
}

impl CheckpointLifecycleV1 {
    pub fn sealed(
        version: CheckpointLifecycleVersion,
        checkpoint_id: CheckpointId,
    ) -> Result<Self, CheckpointError> {
        check_lifecycle_ver(version)?;
        Ok(Self {
            version,
            checkpoint_id,
            prev_checkpoint_id: None,
            statement_core_digest: None,
            status: CheckpointLifecycleStatus::Sealed,
            publication_evidence_root: None,
            challenge_window_start_height: None,
        })
    }

    pub fn from_artifact(artifact: &CheckpointArtifact) -> Result<Self, CheckpointError> {
        Self::sealed(
            CheckpointLifecycleVersion::CURRENT,
            derive_checkpoint_id(artifact)?,
        )
    }

    #[must_use]
    pub const fn version(&self) -> CheckpointLifecycleVersion {
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
    pub const fn statement_core_digest(&self) -> Option<[u8; 32]> {
        self.statement_core_digest
    }

    #[must_use]
    pub const fn status(&self) -> CheckpointLifecycleStatus {
        self.status
    }

    #[must_use]
    pub const fn publication_evidence_root(&self) -> Option<[u8; 32]> {
        self.publication_evidence_root
    }

    #[must_use]
    pub const fn challenge_window_start_height(&self) -> Option<u64> {
        self.challenge_window_start_height
    }

    pub fn link(
        &self,
        artifact: &CheckpointArtifact,
        link: &CheckpointLink,
        predecessor: Option<&CheckpointArtifact>,
        statement_core_digest: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        require_status(self.status, CheckpointLifecycleStatus::Sealed)?;
        if statement_core_digest == [0u8; 32] {
            return Err(CheckpointError::LifecycleMix);
        }
        let checkpoint_id = derive_checkpoint_id(artifact)?;
        if self.checkpoint_id != checkpoint_id || link.checkpoint_id() != checkpoint_id {
            return Err(CheckpointError::LinkMix);
        }
        match artifact.statement() {
            super::CheckpointStatement::V1(statement) => {
                if statement.prep_snapshot_id() != link.prep_snapshot_id()
                    || statement.exec_input_id() != link.exec_input_id()
                {
                    return Err(CheckpointError::LinkMix);
                }
            }
            super::CheckpointStatement::Detached => {
                return Err(CheckpointError::ArtifactCompatMix);
            }
        }
        match (predecessor, link.prev_checkpoint_id()) {
            (None, None) => {}
            (Some(predecessor), Some(prev_checkpoint_id)) => {
                if derive_checkpoint_id(predecessor)? != prev_checkpoint_id
                    || predecessor.new_root() != artifact.prev_root()
                {
                    return Err(CheckpointError::LinkMix);
                }
            }
            _ => return Err(CheckpointError::LinkMix),
        }
        Ok(Self {
            version: self.version,
            checkpoint_id: self.checkpoint_id,
            prev_checkpoint_id: link.prev_checkpoint_id(),
            statement_core_digest: Some(statement_core_digest),
            status: CheckpointLifecycleStatus::Linked,
            publication_evidence_root: None,
            challenge_window_start_height: None,
        })
    }

    pub fn publication_ready(
        &self,
        evidence: &CheckpointPublicationEvidenceV1,
    ) -> Result<Self, CheckpointError> {
        require_status(self.status, CheckpointLifecycleStatus::Linked)?;
        let statement_core_digest = self
            .statement_core_digest
            .ok_or(CheckpointError::LifecycleMix)?;
        if evidence.publication_state() != CheckpointPublicationState::DaPublicationReady {
            return Err(CheckpointError::ArchiveMix);
        }
        if evidence.statement_core_digest() != statement_core_digest {
            return Err(CheckpointError::LifecycleMix);
        }
        Ok(Self {
            version: self.version,
            checkpoint_id: self.checkpoint_id,
            prev_checkpoint_id: self.prev_checkpoint_id,
            statement_core_digest: Some(statement_core_digest),
            status: CheckpointLifecycleStatus::PublicationReady,
            publication_evidence_root: Some(evidence.publication_evidence_root()),
            challenge_window_start_height: Some(evidence.challenge_window_start_height()),
        })
    }

    pub fn challenge_open(&self, current_height: u64) -> Result<Self, CheckpointError> {
        require_status(self.status, CheckpointLifecycleStatus::PublicationReady)?;
        let challenge_window_start_height = self
            .challenge_window_start_height
            .ok_or(CheckpointError::ArchiveMix)?;
        if self.publication_evidence_root.is_none()
            || current_height < challenge_window_start_height
        {
            return Err(CheckpointError::ArchiveMix);
        }
        Ok(Self {
            version: self.version,
            checkpoint_id: self.checkpoint_id,
            prev_checkpoint_id: self.prev_checkpoint_id,
            statement_core_digest: self.statement_core_digest,
            status: CheckpointLifecycleStatus::ChallengeOpen,
            publication_evidence_root: self.publication_evidence_root,
            challenge_window_start_height: Some(challenge_window_start_height),
        })
    }

    pub fn finalize(&self) -> Result<Self, CheckpointError> {
        require_status(self.status, CheckpointLifecycleStatus::ChallengeOpen)?;
        Ok(Self {
            version: self.version,
            checkpoint_id: self.checkpoint_id,
            prev_checkpoint_id: self.prev_checkpoint_id,
            statement_core_digest: self.statement_core_digest,
            status: CheckpointLifecycleStatus::Finalized,
            publication_evidence_root: self.publication_evidence_root,
            challenge_window_start_height: self.challenge_window_start_height,
        })
    }

    pub fn dispute(&self) -> Result<Self, CheckpointError> {
        require_status(self.status, CheckpointLifecycleStatus::ChallengeOpen)?;
        Ok(Self {
            version: self.version,
            checkpoint_id: self.checkpoint_id,
            prev_checkpoint_id: self.prev_checkpoint_id,
            statement_core_digest: self.statement_core_digest,
            status: CheckpointLifecycleStatus::Disputed,
            publication_evidence_root: self.publication_evidence_root,
            challenge_window_start_height: self.challenge_window_start_height,
        })
    }

    pub fn reject(&self) -> Result<Self, CheckpointError> {
        if self.status == CheckpointLifecycleStatus::Finalized {
            return Err(CheckpointError::LifecycleMix);
        }
        Ok(Self {
            version: self.version,
            checkpoint_id: self.checkpoint_id,
            prev_checkpoint_id: self.prev_checkpoint_id,
            statement_core_digest: self.statement_core_digest,
            status: CheckpointLifecycleStatus::Rejected,
            publication_evidence_root: self.publication_evidence_root,
            challenge_window_start_height: self.challenge_window_start_height,
        })
    }
}

pub(crate) fn check_lifecycle_ver(
    version: CheckpointLifecycleVersion,
) -> Result<(), CheckpointError> {
    if version == CheckpointLifecycleVersion::CURRENT {
        return Ok(());
    }
    Err(CheckpointError::VersionMix)
}

fn require_status(
    status: CheckpointLifecycleStatus,
    want: CheckpointLifecycleStatus,
) -> Result<(), CheckpointError> {
    if status == want {
        return Ok(());
    }
    Err(CheckpointError::LifecycleMix)
}

#[cfg(test)]
mod tests {
    use super::{
        check_lifecycle_ver, CheckpointLifecycleStatus, CheckpointLifecycleV1,
        CheckpointLifecycleVersion,
    };
    use crate::{
        checkpoint::{
            CheckpointDaProviderFamily, CheckpointExecInputId, CheckpointLink,
            CheckpointLinkVersion, CheckpointPublicationEvidenceV1,
            CheckpointPublicationEvidenceVersion,
        },
        fixture_support::checkpoint_fixtures,
        CheckpointError,
    };

    fn root(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn statement_core_digest() -> [u8; 32] {
        root(1)
    }

    fn lifecycle_link() -> (crate::checkpoint::CheckpointArtifact, CheckpointLink) {
        let artifact = checkpoint_fixtures::artifact();
        let checkpoint_id =
            crate::checkpoint::derive_checkpoint_id(&artifact).expect("checkpoint id");
        let link = CheckpointLink::new(
            CheckpointLinkVersion::CURRENT,
            checkpoint_id,
            crate::snapshot::PrepSnapshotId::new([7u8; 32]),
            CheckpointExecInputId::new([8u8; 32]),
        )
        .expect("link");
        (artifact, link)
    }

    fn evidence() -> CheckpointPublicationEvidenceV1 {
        CheckpointPublicationEvidenceV1::new(
            CheckpointPublicationEvidenceVersion::CURRENT,
            root(1),
            root(2),
            root(3),
            root(4),
            crate::checkpoint::CheckpointPublicationState::DaPublicationReady,
            CheckpointDaProviderFamily::LocalArchive,
            55,
            57,
            root(5),
        )
        .expect("publication evidence")
    }

    #[test]
    fn test_linked_lifecycle_builds() {
        let (artifact, link) = lifecycle_link();
        let lifecycle = CheckpointLifecycleV1::from_artifact(&artifact)
            .expect("sealed lifecycle")
            .link(&artifact, &link, None, statement_core_digest())
            .expect("linked lifecycle");

        assert_eq!(lifecycle.status(), CheckpointLifecycleStatus::Linked);
        assert_eq!(lifecycle.prev_checkpoint_id(), None);
        assert_eq!(
            lifecycle.statement_core_digest(),
            Some(statement_core_digest())
        );
    }

    #[test]
    fn test_lifecycle_rejects_status_skip() {
        let (artifact, link) = lifecycle_link();
        let lifecycle = CheckpointLifecycleV1::from_artifact(&artifact)
            .expect("sealed lifecycle")
            .link(&artifact, &link, None, statement_core_digest())
            .expect("linked lifecycle");

        let err = lifecycle
            .finalize()
            .expect_err("linked lifecycle cannot finalize");

        assert!(matches!(err, CheckpointError::LifecycleMix));
    }

    #[test]
    fn test_lifecycle_rejects_link_stmt_drift() {
        let artifact = checkpoint_fixtures::artifact();
        let checkpoint_id =
            crate::checkpoint::derive_checkpoint_id(&artifact).expect("checkpoint id");
        let bad_link = CheckpointLink::new(
            CheckpointLinkVersion::CURRENT,
            checkpoint_id,
            crate::snapshot::PrepSnapshotId::new([7u8; 32]),
            CheckpointExecInputId::new([9u8; 32]),
        )
        .expect("bad link");

        let err = CheckpointLifecycleV1::from_artifact(&artifact)
            .expect("sealed lifecycle")
            .link(&artifact, &bad_link, None, statement_core_digest())
            .expect_err("link drift must reject");

        assert!(matches!(err, CheckpointError::LinkMix));
    }

    #[test]
    fn test_lifecycle_rejects_early_challenge_start() {
        let (artifact, link) = lifecycle_link();
        let lifecycle = CheckpointLifecycleV1::from_artifact(&artifact)
            .expect("sealed lifecycle")
            .link(&artifact, &link, None, statement_core_digest())
            .expect("linked lifecycle")
            .publication_ready(&evidence())
            .expect("publication-ready lifecycle");

        let err = lifecycle
            .challenge_open(56)
            .expect_err("challenge window must wait for readiness height");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_lifecycle_rejects_finalized_after_dispute() {
        let (artifact, link) = lifecycle_link();
        let lifecycle = CheckpointLifecycleV1::from_artifact(&artifact)
            .expect("sealed lifecycle")
            .link(&artifact, &link, None, statement_core_digest())
            .expect("linked lifecycle")
            .publication_ready(&evidence())
            .expect("publication-ready lifecycle")
            .challenge_open(57)
            .expect("challenge-open lifecycle")
            .dispute()
            .expect("disputed lifecycle");

        let err = lifecycle
            .finalize()
            .expect_err("disputed lifecycle cannot finalize");

        assert!(matches!(err, CheckpointError::LifecycleMix));
    }

    #[test]
    fn test_lifecycle_bad_version() {
        let err = check_lifecycle_ver(CheckpointLifecycleVersion::new(9))
            .expect_err("bad version must reject");

        assert!(matches!(err, CheckpointError::VersionMix));
    }

    #[test]
    fn test_lifecycle_rejects_publication_evidence_drift() {
        let (artifact, link) = lifecycle_link();
        let lifecycle = CheckpointLifecycleV1::from_artifact(&artifact)
            .expect("sealed lifecycle")
            .link(&artifact, &link, None, statement_core_digest())
            .expect("linked lifecycle");
        let drifted = CheckpointPublicationEvidenceV1::new(
            CheckpointPublicationEvidenceVersion::CURRENT,
            root(9),
            root(2),
            root(3),
            root(4),
            crate::checkpoint::CheckpointPublicationState::DaPublicationReady,
            CheckpointDaProviderFamily::LocalArchive,
            55,
            57,
            root(5),
        )
        .expect("drifted evidence");

        let err = lifecycle
            .publication_ready(&drifted)
            .expect_err("foreign publication evidence must reject");

        assert!(matches!(err, CheckpointError::LifecycleMix));
    }
}
