use std::path::{Path, PathBuf};

use z00z_utils::io::{create_dir_all, path_exists, read_dir, read_file, write_file};

use crate::error::CheckpointError;

use super::{
    decode_link_bin, CheckpointArtifact, CheckpointDraftId, CheckpointExecInputId,
    CheckpointFsStore, CheckpointId, CheckpointLink, CheckpointStore,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CheckpointFinalLane {
    CanonicalSeal,
    NoncanonicalExport,
}

impl CheckpointFinalLane {
    const CANONICAL_SEAL_MARKER: &'static str = "canonical_seal\n";
    const NONCANONICAL_EXPORT_MARKER: &'static str = "noncanonical_export\n";

    fn from_bytes(bytes: &[u8]) -> Result<Self, CheckpointError> {
        let marker = std::str::from_utf8(bytes).map_err(|_| CheckpointError::ArtifactCompatMix)?;
        match marker.trim() {
            "canonical_seal" => Ok(Self::CanonicalSeal),
            "noncanonical_export" => Ok(Self::NoncanonicalExport),
            _ => Err(CheckpointError::ArtifactCompatMix),
        }
    }

    const fn marker_bytes(self) -> &'static [u8] {
        match self {
            Self::CanonicalSeal => Self::CANONICAL_SEAL_MARKER.as_bytes(),
            Self::NoncanonicalExport => Self::NONCANONICAL_EXPORT_MARKER.as_bytes(),
        }
    }
}

impl CheckpointFsStore {
    pub fn checkpoint_dir(&self) -> PathBuf {
        self.resolved_paths
            .checkpoint_artifacts
            .parent()
            .unwrap_or(&self.root)
            .to_path_buf()
    }

    pub fn draft_dir(&self) -> PathBuf {
        self.checkpoint_dir().join("draft")
    }

    pub fn artifact_dir(&self) -> PathBuf {
        self.resolved_paths.checkpoint_artifacts.clone()
    }

    pub fn link_dir(&self) -> PathBuf {
        self.resolved_paths.checkpoint_links.clone()
    }

    pub fn exec_dir(&self) -> PathBuf {
        self.resolved_paths.exec_inputs.clone()
    }

    pub fn audit_dir(&self) -> PathBuf {
        self.audit_dir.clone()
    }

    pub fn archive_manifest_dir(&self) -> PathBuf {
        self.resolved_paths.archive_manifests.clone()
    }

    pub fn staged_archive_manifest_dir(&self) -> PathBuf {
        self.checkpoint_dir().join("staged_archive_manifest")
    }

    pub fn da_reference_dir(&self) -> PathBuf {
        self.resolved_paths.da_references.clone()
    }

    pub fn staged_da_reference_dir(&self) -> PathBuf {
        self.checkpoint_dir().join("staged_da_reference")
    }

    pub fn publication_evidence_dir(&self) -> PathBuf {
        self.resolved_paths.publication_evidence.clone()
    }

    pub fn lifecycle_dir(&self) -> PathBuf {
        self.resolved_paths.checkpoint_lifecycles.clone()
    }

    pub fn final_lane_path(&self) -> PathBuf {
        self.final_lane_path.clone()
    }

    pub(super) fn draft_path(&self, draft_id: &CheckpointDraftId) -> PathBuf {
        self.draft_dir()
            .join(format!("{}.bin", id_hex(draft_id.as_bytes())))
    }

    pub(super) fn artifact_path(&self, checkpoint_id: &CheckpointId) -> PathBuf {
        self.artifact_dir()
            .join(format!("{}.bin", id_hex(checkpoint_id.as_bytes())))
    }

    pub(super) fn link_path(&self, checkpoint_id: &CheckpointId) -> PathBuf {
        self.link_dir()
            .join(format!("{}.bin", id_hex(checkpoint_id.as_bytes())))
    }

    pub(super) fn exec_path(&self, exec_id: &CheckpointExecInputId) -> PathBuf {
        self.exec_dir()
            .join(format!("{}.bin", id_hex(exec_id.as_bytes())))
    }

    pub(super) fn audit_path(&self, checkpoint_id: &CheckpointId) -> PathBuf {
        self.audit_dir()
            .join(format!("{}.bin", id_hex(checkpoint_id.as_bytes())))
    }

    pub(super) fn archive_manifest_path(&self, checkpoint_id: &CheckpointId) -> PathBuf {
        self.archive_manifest_dir()
            .join(format!("{}.bin", id_hex(checkpoint_id.as_bytes())))
    }

    pub(super) fn staged_archive_manifest_path(&self, exec_id: &CheckpointExecInputId) -> PathBuf {
        self.staged_archive_manifest_dir()
            .join(format!("{}.bin", id_hex(exec_id.as_bytes())))
    }

    pub(super) fn staged_statement_core_path(&self, exec_id: &CheckpointExecInputId) -> PathBuf {
        self.staged_archive_manifest_dir()
            .join(format!("{}.core.bin", id_hex(exec_id.as_bytes())))
    }

    pub(super) fn da_reference_path(&self, checkpoint_id: &CheckpointId) -> PathBuf {
        self.da_reference_dir()
            .join(format!("{}.bin", id_hex(checkpoint_id.as_bytes())))
    }

    pub(super) fn staged_da_reference_path(&self, exec_id: &CheckpointExecInputId) -> PathBuf {
        self.staged_da_reference_dir()
            .join(format!("{}.bin", id_hex(exec_id.as_bytes())))
    }

    pub(super) fn publication_evidence_path(&self, checkpoint_id: &CheckpointId) -> PathBuf {
        self.publication_evidence_dir()
            .join(format!("{}.bin", id_hex(checkpoint_id.as_bytes())))
    }

    pub(super) fn lifecycle_path(&self, checkpoint_id: &CheckpointId) -> PathBuf {
        self.lifecycle_dir()
            .join(format!("{}.bin", id_hex(checkpoint_id.as_bytes())))
    }

    pub(super) fn save_bin(path: &Path, bytes: &[u8]) -> Result<(), CheckpointError> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent).map_err(|err| CheckpointError::Backend(err.to_string()))?;
        }
        write_file(path, bytes).map_err(|err| CheckpointError::Backend(err.to_string()))?;
        Ok(())
    }

    pub(super) fn load_final_lane(&self) -> Result<Option<CheckpointFinalLane>, CheckpointError> {
        let path = self.final_lane_path();
        let has_marker =
            path_exists(&path).map_err(|err| CheckpointError::Backend(err.to_string()))?;
        if !has_marker {
            return Ok(None);
        }
        let bytes = read_file(&path).map_err(|err| CheckpointError::Backend(err.to_string()))?;
        CheckpointFinalLane::from_bytes(&bytes).map(Some)
    }

    pub(super) fn persist_final_lane(
        &self,
        lane: CheckpointFinalLane,
    ) -> Result<(), CheckpointError> {
        if let Some(current) = self.load_final_lane()? {
            if current != lane {
                return Err(CheckpointError::ArtifactCompatMix);
            }
        }
        Self::save_bin(&self.final_lane_path(), lane.marker_bytes())
    }

    pub(super) fn reject_noncanonical_final_lane(&self) -> Result<(), CheckpointError> {
        if matches!(
            self.load_final_lane()?,
            Some(CheckpointFinalLane::NoncanonicalExport)
        ) {
            return Err(CheckpointError::ArtifactCompatMix);
        }
        Ok(())
    }

    pub(super) fn require_noncanonical_final_lane(&self) -> Result<(), CheckpointError> {
        if matches!(
            self.load_final_lane()?,
            Some(CheckpointFinalLane::NoncanonicalExport)
        ) {
            return Ok(());
        }
        Err(CheckpointError::ArtifactCompatMix)
    }

    pub(super) fn reject_canonical_final_lane(&self) -> Result<(), CheckpointError> {
        if matches!(
            self.load_final_lane()?,
            Some(CheckpointFinalLane::CanonicalSeal)
        ) {
            return Err(CheckpointError::ArtifactCompatMix);
        }
        Ok(())
    }

    pub(super) fn check_link_art(&self, link: &CheckpointLink) -> Result<(), CheckpointError> {
        let path = self.artifact_path(&link.checkpoint_id());
        let has_art =
            path_exists(&path).map_err(|err| CheckpointError::Backend(err.to_string()))?;
        if !has_art {
            return Err(CheckpointError::LinkMix);
        }

        let artifact = self.load_persisted_artifact(&link.checkpoint_id())?;
        Self::check_link_stmt(link, &artifact)
    }

    pub(super) fn check_link_stmt(
        link: &CheckpointLink,
        artifact: &CheckpointArtifact,
    ) -> Result<(), CheckpointError> {
        match artifact.statement() {
            crate::checkpoint::CheckpointStatement::V1(stmt) => {
                if stmt.prep_snapshot_id() != link.prep_snapshot_id()
                    || stmt.exec_input_id() != link.exec_input_id()
                {
                    return Err(CheckpointError::LinkMix);
                }

                Ok(())
            }
            crate::checkpoint::CheckpointStatement::Detached => {
                Err(CheckpointError::ArtifactCompatMix)
            }
        }
    }

    pub(super) fn check_link_uniq(&self, link: &CheckpointLink) -> Result<(), CheckpointError> {
        create_dir_all(self.link_dir()).map_err(|err| CheckpointError::Backend(err.to_string()))?;
        for path in
            read_dir(self.link_dir()).map_err(|err| CheckpointError::Backend(err.to_string()))?
        {
            let bytes =
                read_file(&path).map_err(|err| CheckpointError::Backend(err.to_string()))?;
            let seen = decode_link_bin(&bytes)?;
            if seen.checkpoint_id() == link.checkpoint_id() {
                if seen == *link {
                    continue;
                }
                return Err(CheckpointError::LinkMix);
            }

            if seen.prev_checkpoint_id() == link.prev_checkpoint_id() {
                return Err(CheckpointError::LinkMix);
            }

            if seen.exec_input_id() == link.exec_input_id() {
                let reusable_noop = self
                    .load_exec_input(&link.exec_input_id())
                    .map_err(|_| CheckpointError::ReplayMix)?
                    .is_recursive_v2_noop();
                if !reusable_noop {
                    return Err(CheckpointError::LinkMix);
                }
            }
        }

        Ok(())
    }
}

fn id_hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
