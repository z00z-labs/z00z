use std::path::PathBuf;

use z00z_utils::codec::{BincodeCodec, Codec};
use z00z_utils::io::{path_exists, read_dir, read_file, remove_file};

use crate::error::CheckpointError;
use crate::snapshot::{PrepFsStore, PrepSnapshot, PrepSnapshotId, PrepSnapshotStore};

use super::{
    audit::CheckpointAudit,
    codec::{
        decode_archive_manifest_bin, decode_art_bin, decode_audit_bin, decode_da_reference_bin,
        decode_draft_bin, decode_exec_bin, decode_link_bin, decode_publication_evidence_bin,
        decode_recursive_sidecar_bin, encode_archive_manifest_bin, encode_art_bin,
        encode_audit_bin, encode_da_reference_bin, encode_draft_bin, encode_exec_bin,
        encode_link_bin, encode_publication_evidence_bin, encode_recursive_sidecar_bin,
    },
    contract_config::{CheckpointContractConfigV1, CheckpointResolvedPaths},
    exec_input::CheckpointExecInput,
    ids::{
        derive_checkpoint_id, derive_draft_id, derive_exec_id, CheckpointDraftId,
        CheckpointExecInputId, CheckpointId,
    },
    lifecycle::{check_lifecycle_ver, CheckpointLifecycleV1},
    link::CheckpointLink,
    store_fs::CheckpointFinalLane,
    CheckpointArchiveManifestV1, CheckpointArtifact, CheckpointDaReferenceV1, CheckpointDraft,
    CheckpointProof, CheckpointPublicationEvidenceV1, CheckpointTransitionStatementFinalV1,
    RecursiveCheckpointSidecarV1, RecursiveCheckpointVerifierV1,
};

/// Load one canonical checkpoint draft from canonical storage bytes.
///
/// # Examples
///
/// ```
/// use z00z_storage::{
///     checkpoint::{load_draft, CheckpointDraft, CheckpointVersion, CreatedEnt, SpentEnt},
///     settlement::CheckRoot,
/// };
/// use z00z_utils::codec::{BincodeCodec, Codec};
///
/// let draft = CheckpointDraft::new(
///     CheckpointVersion::CURRENT,
///     9,
///     CheckRoot::new([1u8; 32]),
///     CheckRoot::new([2u8; 32]),
///     vec![SpentEnt::new([3u8; 32])],
///     vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
/// );
/// let bytes = BincodeCodec.serialize(&draft)?;
/// let loaded = load_draft(&bytes)?;
/// assert_eq!(loaded, draft);
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
pub fn load_draft(bytes: &[u8]) -> Result<CheckpointDraft, CheckpointError> {
    match decode_draft_bin(bytes) {
        Ok(draft) => Ok(draft),
        Err(err) => {
            if decode_art_bin(bytes).is_ok() {
                return Err(CheckpointError::WrongClass);
            }
            Err(err)
        }
    }
}

/// Load one canonical final checkpoint artifact from canonical storage bytes.
///
/// # Examples
///
/// ```
/// use z00z_storage::{
///     checkpoint::{load_artifact, CheckpointArtifact, CheckpointDraft, CheckpointExecInputId, CheckpointVersion, CreatedEnt, SpentEnt},
///     settlement::CheckRoot,
///     snapshot::PrepSnapshotId,
/// };
/// use z00z_utils::codec::{BincodeCodec, Codec};
///
/// let draft = CheckpointDraft::new(
///     CheckpointVersion::CURRENT,
///     9,
///     CheckRoot::new([1u8; 32]),
///     CheckRoot::new([2u8; 32]),
///     vec![SpentEnt::new([3u8; 32])],
///     vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
/// );
/// let proof = draft.attest_proof(
///     PrepSnapshotId::new([6u8; 32]),
///     CheckpointExecInputId::new([7u8; 32]),
/// )?;
/// let art = draft.finalize(proof)?;
/// let bytes = BincodeCodec.serialize(&art)?;
/// let loaded = load_artifact(&bytes)?;
/// assert_eq!(loaded, art);
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
pub fn load_artifact(bytes: &[u8]) -> Result<CheckpointArtifact, CheckpointError> {
    match decode_art_bin(bytes) {
        Ok(artifact) => Ok(artifact),
        Err(err) => {
            if decode_draft_bin(bytes).is_ok() {
                return Err(CheckpointError::WrongClass);
            }
            Err(err)
        }
    }
}

/// Check one backend draft key against its expected external id.
pub fn check_draft_key(
    want: CheckpointDraftId,
    got: CheckpointDraftId,
) -> Result<(), CheckpointError> {
    if want == got {
        return Ok(());
    }

    Err(CheckpointError::KeyMix)
}

/// Check one backend-final-artifact key against its expected external id.
///
/// # Examples
///
/// ```
/// use z00z_storage::checkpoint::{check_art_key, CheckpointId};
///
/// let art_id = CheckpointId::new([7u8; 32]);
/// check_art_key(art_id, art_id)?;
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
pub fn check_art_key(want: CheckpointId, got: CheckpointId) -> Result<(), CheckpointError> {
    if want == got {
        return Ok(());
    }

    Err(CheckpointError::KeyMix)
}

/// Check one backend execution-input key against its expected external id.
///
/// # Examples
///
/// ```
/// use z00z_storage::checkpoint::{check_exec_key, CheckpointExecInputId};
///
/// let exec_id = CheckpointExecInputId::new([7u8; 32]);
/// check_exec_key(exec_id, exec_id)?;
/// # Ok::<(), z00z_storage::CheckpointError>(())
/// ```
pub fn check_exec_key(
    want: CheckpointExecInputId,
    got: CheckpointExecInputId,
) -> Result<(), CheckpointError> {
    if want == got {
        return Ok(());
    }

    Err(CheckpointError::KeyMix)
}

/// Check one canonical replay link against snapshot and execution input ids.
pub fn check_link_ids(
    snap_id: PrepSnapshotId,
    link: &CheckpointLink,
    exec: &CheckpointExecInput,
) -> Result<CheckpointExecInputId, CheckpointError> {
    if link.prep_snapshot_id() != snap_id || exec.prep_snapshot_id() != snap_id {
        return Err(CheckpointError::LinkMix);
    }

    let exec_id = derive_exec_id(&encode_exec_bin(exec)?);
    if link.exec_input_id() != exec_id {
        return Err(CheckpointError::ReplayMix);
    }

    Ok(exec_id)
}

/// Check one execution input root against one validated snapshot root.
pub fn check_exec_root(
    snapshot: &PrepSnapshot,
    exec: &CheckpointExecInput,
) -> Result<(), CheckpointError> {
    if snapshot.prev_root != exec.prev_root() {
        return Err(CheckpointError::RootMix);
    }

    Ok(())
}

fn check_exec_replay(
    snap_id: PrepSnapshotId,
    exec_id: CheckpointExecInputId,
    exec: &CheckpointExecInput,
) -> Result<(), CheckpointError> {
    if exec.prep_snapshot_id() != snap_id {
        return Err(CheckpointError::LinkMix);
    }

    let got = derive_exec_id(&encode_exec_bin(exec)?);
    if got != exec_id {
        return Err(CheckpointError::ReplayMix);
    }

    Ok(())
}

/// Narrow storage-owned checkpoint facade.
pub trait CheckpointStore {
    fn save_draft(&mut self, draft: &CheckpointDraft)
        -> Result<CheckpointDraftId, CheckpointError>;

    fn load_draft(&self, draft_id: &CheckpointDraftId) -> Result<CheckpointDraft, CheckpointError>;

    fn load_artifact(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointArtifact, CheckpointError>;

    /// Finalize and persist one canonical attested artifact.
    ///
    /// This path succeeds only when the attested statement matches persisted
    /// snapshot and execution-input rows that already exist as replay evidence.
    fn seal_artifact(
        &mut self,
        draft: &CheckpointDraft,
        proof: CheckpointProof,
        snap_id: PrepSnapshotId,
        exec_id: CheckpointExecInputId,
    ) -> Result<CheckpointLink, CheckpointError>;

    fn save_link(&mut self, link: &CheckpointLink) -> Result<(), CheckpointError>;

    fn load_link(&self, checkpoint_id: &CheckpointId) -> Result<CheckpointLink, CheckpointError>;

    fn stage_publication_contract(
        &mut self,
        exec_id: CheckpointExecInputId,
        manifest: &CheckpointArchiveManifestV1,
        da_reference: &CheckpointDaReferenceV1,
    ) -> Result<(), CheckpointError>;

    fn save_exec_input(
        &mut self,
        exec: &CheckpointExecInput,
    ) -> Result<CheckpointExecInputId, CheckpointError>;

    fn load_exec_input(
        &self,
        exec_id: &CheckpointExecInputId,
    ) -> Result<CheckpointExecInput, CheckpointError>;

    fn save_audit(&mut self, audit: &CheckpointAudit) -> Result<(), CheckpointError>;

    fn load_audit(&self, checkpoint_id: &CheckpointId) -> Result<CheckpointAudit, CheckpointError>;

    fn save_archive_manifest(
        &mut self,
        checkpoint_id: CheckpointId,
        manifest: &CheckpointArchiveManifestV1,
    ) -> Result<(), CheckpointError>;

    fn load_archive_manifest(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointArchiveManifestV1, CheckpointError>;

    fn save_da_reference(
        &mut self,
        checkpoint_id: CheckpointId,
        da_reference: &CheckpointDaReferenceV1,
    ) -> Result<(), CheckpointError>;

    fn load_da_reference(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointDaReferenceV1, CheckpointError>;

    fn save_publication_evidence(
        &mut self,
        checkpoint_id: CheckpointId,
        evidence: &CheckpointPublicationEvidenceV1,
    ) -> Result<(), CheckpointError>;

    fn load_publication_evidence(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointPublicationEvidenceV1, CheckpointError>;

    fn save_lifecycle(&mut self, lifecycle: &CheckpointLifecycleV1) -> Result<(), CheckpointError>;

    fn load_lifecycle(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointLifecycleV1, CheckpointError>;

    /// Persist non-authoritative recursive evidence bound to one canonical checkpoint.
    fn save_recursive_sidecar(
        &mut self,
        checkpoint_id: CheckpointId,
        sidecar: &RecursiveCheckpointSidecarV1,
    ) -> Result<(), CheckpointError>;

    /// Load recursive shadow evidence only after rechecking its canonical binding.
    fn load_recursive_sidecar(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<RecursiveCheckpointSidecarV1, CheckpointError>;
}

/// File-backed checkpoint store with separate persistence surfaces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointFsStore {
    pub(super) root: PathBuf,
    pub(super) resolved_paths: CheckpointResolvedPaths,
    pub(super) audit_dir: PathBuf,
    pub(super) final_lane_path: PathBuf,
}

#[derive(Clone, Debug)]
struct StagedPublicationContract {
    manifest: CheckpointArchiveManifestV1,
    da_reference: CheckpointDaReferenceV1,
}

impl CheckpointFsStore {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self::try_new(root).expect("repo checkpoint contract must validate checkpoint store paths")
    }

    pub fn try_new(root: impl Into<PathBuf>) -> Result<Self, CheckpointError> {
        let root = root.into();
        let cfg = CheckpointContractConfigV1::load_repo_default()?;
        let resolved_paths = cfg.resolve_paths(&root);
        let checkpoint_namespace_dir = resolved_paths
            .checkpoint_artifacts
            .parent()
            .unwrap_or(&root)
            .to_path_buf();
        Ok(Self {
            root,
            audit_dir: checkpoint_namespace_dir.join("audit"),
            final_lane_path: checkpoint_namespace_dir.join("final_lane.marker"),
            resolved_paths,
        })
    }

    fn persist_artifact_bin(
        &mut self,
        artifact: &CheckpointArtifact,
    ) -> Result<CheckpointId, CheckpointError> {
        let checkpoint_id = derive_checkpoint_id(artifact)?;
        let bytes = encode_art_bin(artifact)?;
        Self::save_bin(&self.artifact_path(&checkpoint_id), &bytes)?;
        Ok(checkpoint_id)
    }

    pub(super) fn load_persisted_artifact(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointArtifact, CheckpointError> {
        let bytes = read_file(self.artifact_path(checkpoint_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        let artifact = load_artifact(&bytes)?;
        let got = derive_checkpoint_id(&artifact)?;
        check_art_key(*checkpoint_id, got)?;
        Ok(artifact)
    }

    fn persist_audit_bin(&self, audit: &CheckpointAudit) -> Result<(), CheckpointError> {
        let bytes = encode_audit_bin(audit)?;
        self.save_unique_bin(
            &self.audit_path(&audit.checkpoint_id()),
            &bytes,
            CheckpointError::ArchiveMix,
        )
    }

    fn load_persisted_audit(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointAudit, CheckpointError> {
        let bytes = read_file(self.audit_path(checkpoint_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        let audit = decode_audit_bin(&bytes)?;
        check_art_key(*checkpoint_id, audit.checkpoint_id())?;
        Ok(audit)
    }

    fn save_unique_bin(
        &self,
        path: &std::path::Path,
        bytes: &[u8],
        drift_err: CheckpointError,
    ) -> Result<(), CheckpointError> {
        if path_exists(path).map_err(|err| CheckpointError::Backend(err.to_string()))? {
            let current =
                read_file(path).map_err(|err| CheckpointError::Backend(err.to_string()))?;
            if current != bytes {
                return Err(drift_err);
            }
            return Ok(());
        }
        Self::save_bin(path, bytes)
    }

    fn persist_archive_manifest_bin(
        &self,
        checkpoint_id: CheckpointId,
        manifest: &CheckpointArchiveManifestV1,
    ) -> Result<(), CheckpointError> {
        let bytes = encode_archive_manifest_bin(manifest)?;
        self.save_unique_bin(
            &self.archive_manifest_path(&checkpoint_id),
            &bytes,
            CheckpointError::ArchiveMix,
        )
    }

    fn load_persisted_archive_manifest(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointArchiveManifestV1, CheckpointError> {
        let bytes = read_file(self.archive_manifest_path(checkpoint_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        decode_archive_manifest_bin(&bytes)
    }

    fn persist_staged_archive_manifest_bin(
        &self,
        exec_id: CheckpointExecInputId,
        manifest: &CheckpointArchiveManifestV1,
    ) -> Result<(), CheckpointError> {
        let bytes = encode_archive_manifest_bin(manifest)?;
        self.save_unique_bin(
            &self.staged_archive_manifest_path(&exec_id),
            &bytes,
            CheckpointError::ArchiveMix,
        )
    }

    fn load_staged_archive_manifest(
        &self,
        exec_id: &CheckpointExecInputId,
    ) -> Result<CheckpointArchiveManifestV1, CheckpointError> {
        let bytes = read_file(self.staged_archive_manifest_path(exec_id))
            .map_err(|_| CheckpointError::ArchiveMix)?;
        decode_archive_manifest_bin(&bytes)
    }

    fn persist_da_reference_bin(
        &self,
        checkpoint_id: CheckpointId,
        da_reference: &CheckpointDaReferenceV1,
    ) -> Result<(), CheckpointError> {
        let bytes = encode_da_reference_bin(da_reference)?;
        self.save_unique_bin(
            &self.da_reference_path(&checkpoint_id),
            &bytes,
            CheckpointError::ArchiveMix,
        )
    }

    fn load_persisted_da_reference(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointDaReferenceV1, CheckpointError> {
        let bytes = read_file(self.da_reference_path(checkpoint_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        decode_da_reference_bin(&bytes)
    }

    fn persist_staged_da_reference_bin(
        &self,
        exec_id: CheckpointExecInputId,
        da_reference: &CheckpointDaReferenceV1,
    ) -> Result<(), CheckpointError> {
        let bytes = encode_da_reference_bin(da_reference)?;
        self.save_unique_bin(
            &self.staged_da_reference_path(&exec_id),
            &bytes,
            CheckpointError::ArchiveMix,
        )
    }

    fn load_staged_da_reference(
        &self,
        exec_id: &CheckpointExecInputId,
    ) -> Result<CheckpointDaReferenceV1, CheckpointError> {
        let bytes = read_file(self.staged_da_reference_path(exec_id))
            .map_err(|_| CheckpointError::ArchiveMix)?;
        decode_da_reference_bin(&bytes)
    }

    fn load_staged_publication_contract(
        &self,
        exec_id: &CheckpointExecInputId,
    ) -> Result<StagedPublicationContract, CheckpointError> {
        let manifest = self.load_staged_archive_manifest(exec_id)?;
        let da_reference = self.load_staged_da_reference(exec_id)?;
        if manifest.checkpoint_exec_input_id() != *exec_id
            || da_reference.statement_core_digest() != manifest.statement_core_digest()
            || da_reference.archive_manifest_root() != manifest.archive_manifest_root()
            || da_reference.payload_commitment() != manifest.da_payload_commitment()
        {
            return Err(CheckpointError::ArchiveMix);
        }
        Ok(StagedPublicationContract {
            manifest,
            da_reference,
        })
    }

    fn clear_staged_publication_contract(
        &self,
        exec_id: &CheckpointExecInputId,
    ) -> Result<(), CheckpointError> {
        for path in [
            self.staged_archive_manifest_path(exec_id),
            self.staged_da_reference_path(exec_id),
        ] {
            if path_exists(&path).map_err(|err| CheckpointError::Backend(err.to_string()))? {
                remove_file(&path).map_err(|err| CheckpointError::Backend(err.to_string()))?;
            }
        }
        Ok(())
    }

    fn persist_publication_evidence_bin(
        &self,
        checkpoint_id: CheckpointId,
        evidence: &CheckpointPublicationEvidenceV1,
    ) -> Result<(), CheckpointError> {
        let bytes = encode_publication_evidence_bin(evidence)?;
        self.save_unique_bin(
            &self.publication_evidence_path(&checkpoint_id),
            &bytes,
            CheckpointError::ArchiveMix,
        )
    }

    fn load_persisted_publication_evidence(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointPublicationEvidenceV1, CheckpointError> {
        let bytes = read_file(self.publication_evidence_path(checkpoint_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        decode_publication_evidence_bin(&bytes)
    }

    fn persist_lifecycle_bin(
        &self,
        lifecycle: &CheckpointLifecycleV1,
    ) -> Result<(), CheckpointError> {
        check_lifecycle_ver(lifecycle.version())?;
        let bytes = BincodeCodec.serialize(lifecycle)?;
        self.save_unique_bin(
            &self.lifecycle_path(&lifecycle.checkpoint_id()),
            &bytes,
            CheckpointError::LifecycleMix,
        )
    }

    fn load_persisted_lifecycle(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointLifecycleV1, CheckpointError> {
        let bytes = read_file(self.lifecycle_path(checkpoint_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        let lifecycle: CheckpointLifecycleV1 = BincodeCodec.deserialize(&bytes)?;
        check_lifecycle_ver(lifecycle.version())?;
        if lifecycle.checkpoint_id() != *checkpoint_id {
            return Err(CheckpointError::KeyMix);
        }
        Ok(lifecycle)
    }

    fn check_recursive_sidecar_binding(
        &self,
        checkpoint_id: CheckpointId,
        sidecar: &RecursiveCheckpointSidecarV1,
    ) -> Result<(), CheckpointError> {
        if sidecar.checkpoint_id_hint() != Some(checkpoint_id) {
            return Err(CheckpointError::ArchiveMix);
        }
        let artifact = self.load_persisted_artifact(&checkpoint_id)?;
        let statement = match artifact.statement() {
            super::CheckpointStatement::V1(statement) => statement,
            super::CheckpointStatement::Detached => return Err(CheckpointError::ArchiveMix),
        };
        let statement_core = artifact
            .statement_core()
            .ok_or(CheckpointError::ArchiveMix)?;
        let da_ref = artifact.da_ref().ok_or(CheckpointError::ArchiveMix)?;
        let link = self.load_link_validated(&checkpoint_id)?;
        let cfg = CheckpointContractConfigV1::load_repo_default()?;
        let verifier =
            RecursiveCheckpointVerifierV1::new(&cfg).map_err(|_| CheckpointError::ArchiveMix)?;
        verifier
            .verify_sidecar(
                sidecar,
                &statement,
                &statement_core,
                &CheckpointTransitionStatementFinalV1::new(da_ref),
                &link,
            )
            .map_err(|_| CheckpointError::ArchiveMix)
    }

    fn persist_recursive_sidecar_bin(
        &self,
        checkpoint_id: CheckpointId,
        sidecar: &RecursiveCheckpointSidecarV1,
    ) -> Result<(), CheckpointError> {
        let bytes = encode_recursive_sidecar_bin(sidecar)?;
        self.save_unique_bin(
            &self.recursive_sidecar_path(&checkpoint_id),
            &bytes,
            CheckpointError::ArchiveMix,
        )
    }

    fn load_persisted_recursive_sidecar(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<RecursiveCheckpointSidecarV1, CheckpointError> {
        let bytes = read_file(self.recursive_sidecar_path(checkpoint_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        decode_recursive_sidecar_bin(&bytes)
    }

    fn write_link_bin(&self, link: &CheckpointLink) -> Result<(), CheckpointError> {
        let bytes = encode_link_bin(link)?;
        Self::save_bin(&self.link_path(&link.checkpoint_id()), &bytes)
    }

    fn predecessor_candidates(
        &self,
        artifact: &CheckpointArtifact,
        skip: Option<CheckpointId>,
    ) -> Result<Vec<CheckpointId>, CheckpointError> {
        let link_dir = self.link_dir();
        if !path_exists(&link_dir).map_err(|err| CheckpointError::Backend(err.to_string()))? {
            return Ok(Vec::new());
        }

        let mut out = Vec::new();
        for path in read_dir(link_dir).map_err(|err| CheckpointError::Backend(err.to_string()))? {
            let bytes =
                read_file(&path).map_err(|err| CheckpointError::Backend(err.to_string()))?;
            let link = decode_link_bin(&bytes)?;
            if skip == Some(link.checkpoint_id()) {
                continue;
            }
            let prev = self.load_persisted_artifact(&link.checkpoint_id())?;
            if prev.new_root() == artifact.prev_root() {
                out.push(link.checkpoint_id());
            }
        }
        Ok(out)
    }

    fn infer_prev_checkpoint_id(
        &self,
        artifact: &CheckpointArtifact,
    ) -> Result<Option<CheckpointId>, CheckpointError> {
        match self.predecessor_candidates(artifact, None)?.as_slice() {
            [] => Ok(None),
            [checkpoint_id] => Ok(Some(*checkpoint_id)),
            _ => Err(CheckpointError::LinkMix),
        }
    }

    fn check_link_chain(&self, link: &CheckpointLink) -> Result<(), CheckpointError> {
        let artifact = self.load_persisted_artifact(&link.checkpoint_id())?;
        let candidates = self.predecessor_candidates(&artifact, Some(link.checkpoint_id()))?;
        match (link.prev_checkpoint_id(), candidates.as_slice()) {
            (None, []) => Ok(()),
            (Some(prev_checkpoint_id), [candidate]) if *candidate == prev_checkpoint_id => Ok(()),
            _ => Err(CheckpointError::LinkMix),
        }
    }

    fn check_link_evidence(&self, link: &CheckpointLink) -> Result<(), CheckpointError> {
        let snapshot = PrepFsStore::new(&self.root)
            .load_snapshot(&link.prep_snapshot_id())
            .map_err(|_| CheckpointError::LinkMix)?;
        let exec = self
            .load_exec_input(&link.exec_input_id())
            .map_err(|_| CheckpointError::ReplayMix)?;
        check_link_ids(link.prep_snapshot_id(), link, &exec)?;
        check_exec_root(&snapshot, &exec)?;
        Ok(())
    }

    fn check_link_ready(&self, link: &CheckpointLink) -> Result<(), CheckpointError> {
        self.check_link_art(link)?;
        self.check_link_chain(link)?;
        self.check_link_uniq(link)?;
        self.check_link_evidence(link)
    }

    fn load_link_validated(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointLink, CheckpointError> {
        let bytes = read_file(self.link_path(checkpoint_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        let link = decode_link_bin(&bytes)?;
        check_art_key(*checkpoint_id, link.checkpoint_id())?;
        let artifact = self.load_persisted_artifact(checkpoint_id)?;
        Self::check_link_stmt(&link, &artifact)?;
        self.check_link_chain(&link)?;
        self.check_link_evidence(&link)?;
        Ok(link)
    }

    pub fn export_noncanonical_final_bundle(
        &mut self,
        artifact: &CheckpointArtifact,
        link: &CheckpointLink,
        audit: &CheckpointAudit,
    ) -> Result<CheckpointId, CheckpointError> {
        self.reject_canonical_final_lane()?;
        let checkpoint_id = derive_checkpoint_id(artifact)?;
        check_art_key(link.checkpoint_id(), checkpoint_id)?;
        check_art_key(audit.checkpoint_id(), checkpoint_id)?;
        Self::check_link_stmt(link, artifact)?;
        self.check_link_uniq(link)?;
        self.check_link_evidence(link)?;
        self.persist_artifact_bin(artifact)?;
        self.write_link_bin(link)?;
        self.persist_audit_bin(audit)?;
        self.persist_final_lane(CheckpointFinalLane::NoncanonicalExport)?;
        Ok(checkpoint_id)
    }

    pub fn load_noncanonical_artifact(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointArtifact, CheckpointError> {
        self.require_noncanonical_final_lane()?;
        self.load_persisted_artifact(checkpoint_id)
    }

    pub fn load_noncanonical_link(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointLink, CheckpointError> {
        self.require_noncanonical_final_lane()?;
        self.load_link_validated(checkpoint_id)
    }

    pub fn load_noncanonical_audit(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointAudit, CheckpointError> {
        self.require_noncanonical_final_lane()?;
        self.load_persisted_audit(checkpoint_id)
    }
}

impl CheckpointStore for CheckpointFsStore {
    fn save_draft(
        &mut self,
        draft: &CheckpointDraft,
    ) -> Result<CheckpointDraftId, CheckpointError> {
        let draft_id = derive_draft_id(draft)?;
        let bytes = encode_draft_bin(draft)?;
        Self::save_bin(&self.draft_path(&draft_id), &bytes)?;
        Ok(draft_id)
    }

    fn load_draft(&self, draft_id: &CheckpointDraftId) -> Result<CheckpointDraft, CheckpointError> {
        let bytes = read_file(self.draft_path(draft_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        let draft = load_draft(&bytes)?;
        let got = derive_draft_id(&draft)?;
        check_draft_key(*draft_id, got)?;
        Ok(draft)
    }

    fn load_artifact(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointArtifact, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_persisted_artifact(checkpoint_id)
    }

    fn stage_publication_contract(
        &mut self,
        exec_id: CheckpointExecInputId,
        manifest: &CheckpointArchiveManifestV1,
        da_reference: &CheckpointDaReferenceV1,
    ) -> Result<(), CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        if manifest.checkpoint_exec_input_id() != exec_id
            || da_reference.statement_core_digest() != manifest.statement_core_digest()
            || da_reference.archive_manifest_root() != manifest.archive_manifest_root()
            || da_reference.payload_commitment() != manifest.da_payload_commitment()
        {
            return Err(CheckpointError::ArchiveMix);
        }
        self.persist_staged_archive_manifest_bin(exec_id, manifest)?;
        self.persist_staged_da_reference_bin(exec_id, da_reference)
    }

    fn seal_artifact(
        &mut self,
        draft: &CheckpointDraft,
        proof: CheckpointProof,
        snap_id: PrepSnapshotId,
        exec_id: CheckpointExecInputId,
    ) -> Result<CheckpointLink, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        let stmt = proof.statement();
        if stmt.prep_snapshot_id() != snap_id || stmt.exec_input_id() != exec_id {
            return Err(CheckpointError::LinkMix);
        }
        let snapshot = PrepFsStore::new(&self.root)
            .load_snapshot(&snap_id)
            .map_err(|_| CheckpointError::LinkMix)?;
        let exec = self
            .load_exec_input(&exec_id)
            .map_err(|_| CheckpointError::ReplayMix)?;
        check_exec_replay(snap_id, exec_id, &exec)?;
        check_exec_root(&snapshot, &exec)?;
        let staged = self.load_staged_publication_contract(&exec_id)?;
        if staged.manifest.prep_snapshot_id() != snap_id
            || staged.manifest.tx_data_root() != exec.tx_data_root()
        {
            return Err(CheckpointError::ArchiveMix);
        }
        let statement_core = super::CheckpointTransitionStatementCoreV1::new(
            staged.manifest.tx_data_root(),
            staged.manifest.delta_root(),
            staged.manifest.witness_root(),
            staged.manifest.journal_digest(),
        );
        let statement_core_digest = stmt.statement_core_digest_v1(&statement_core);
        if staged.manifest.statement_core_digest() != statement_core_digest
            || staged.da_reference.statement_core_digest() != statement_core_digest
        {
            return Err(CheckpointError::ArchiveMix);
        }
        let artifact = draft.finalize(proof)?.bind_canonical_v1(
            statement_core,
            super::CheckpointTransitionStatementFinalV1::new(staged.da_reference.da_ref()),
        )?;
        let checkpoint_id = derive_checkpoint_id(&artifact)?;
        let link = CheckpointLink::with_prev(
            super::link::CheckpointLinkVersion::CURRENT,
            checkpoint_id,
            self.infer_prev_checkpoint_id(&artifact)?,
            snap_id,
            exec_id,
        )?;
        Self::check_link_stmt(&link, &artifact)?;
        self.check_link_uniq(&link)?;
        self.check_link_evidence(&link)?;
        self.persist_artifact_bin(&artifact)?;
        self.persist_archive_manifest_bin(checkpoint_id, &staged.manifest)?;
        self.persist_da_reference_bin(checkpoint_id, &staged.da_reference)?;
        self.clear_staged_publication_contract(&exec_id)?;
        self.write_link_bin(&link)?;
        self.persist_final_lane(CheckpointFinalLane::CanonicalSeal)?;
        Ok(link)
    }

    fn save_link(&mut self, link: &CheckpointLink) -> Result<(), CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.check_link_ready(link)?;
        self.write_link_bin(link)
    }

    fn load_link(&self, checkpoint_id: &CheckpointId) -> Result<CheckpointLink, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_link_validated(checkpoint_id)
    }

    fn save_exec_input(
        &mut self,
        exec: &CheckpointExecInput,
    ) -> Result<CheckpointExecInputId, CheckpointError> {
        let bytes = encode_exec_bin(exec)?;
        let exec_id = derive_exec_id(&bytes);
        Self::save_bin(&self.exec_path(&exec_id), &bytes)?;
        Ok(exec_id)
    }

    fn load_exec_input(
        &self,
        exec_id: &CheckpointExecInputId,
    ) -> Result<CheckpointExecInput, CheckpointError> {
        let bytes = read_file(self.exec_path(exec_id))
            .map_err(|err| CheckpointError::Backend(err.to_string()))?;
        let exec = decode_exec_bin(&bytes)?;
        let canon = encode_exec_bin(&exec)?;
        let got = derive_exec_id(&canon);
        check_exec_key(*exec_id, got)?;
        Ok(exec)
    }

    fn save_audit(&mut self, audit: &CheckpointAudit) -> Result<(), CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_persisted_artifact(&audit.checkpoint_id())?;
        self.persist_audit_bin(audit)
    }

    fn load_audit(&self, checkpoint_id: &CheckpointId) -> Result<CheckpointAudit, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_persisted_audit(checkpoint_id)
    }

    fn save_archive_manifest(
        &mut self,
        checkpoint_id: CheckpointId,
        manifest: &CheckpointArchiveManifestV1,
    ) -> Result<(), CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        let artifact = self.load_persisted_artifact(&checkpoint_id)?;
        if let Some(statement_core) = artifact.statement_core() {
            match artifact.statement() {
                super::CheckpointStatement::V1(statement) => {
                    if statement_core.tx_data_root() != manifest.tx_data_root()
                        || statement_core.delta_root() != manifest.delta_root()
                        || statement_core.witness_root() != manifest.witness_root()
                        || statement_core.journal_digest() != manifest.journal_digest()
                        || statement.prep_snapshot_id() != manifest.prep_snapshot_id()
                        || statement.exec_input_id() != manifest.checkpoint_exec_input_id()
                        || statement.statement_core_digest_v1(&statement_core)
                            != manifest.statement_core_digest()
                    {
                        return Err(CheckpointError::ArchiveMix);
                    }
                }
                super::CheckpointStatement::Detached => {
                    return Err(CheckpointError::ArchiveMix);
                }
            }
        }
        self.persist_archive_manifest_bin(checkpoint_id, manifest)
    }

    fn load_archive_manifest(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointArchiveManifestV1, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_persisted_archive_manifest(checkpoint_id)
    }

    fn save_da_reference(
        &mut self,
        checkpoint_id: CheckpointId,
        da_reference: &CheckpointDaReferenceV1,
    ) -> Result<(), CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        let artifact = self.load_persisted_artifact(&checkpoint_id)?;
        let manifest = self.load_persisted_archive_manifest(&checkpoint_id)?;
        if da_reference.statement_core_digest() != manifest.statement_core_digest()
            || da_reference.archive_manifest_root() != manifest.archive_manifest_root()
            || da_reference.payload_commitment() != manifest.da_payload_commitment()
            || artifact.da_ref() != Some(da_reference.da_ref())
        {
            return Err(CheckpointError::ArchiveMix);
        }
        self.persist_da_reference_bin(checkpoint_id, da_reference)
    }

    fn load_da_reference(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointDaReferenceV1, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_persisted_da_reference(checkpoint_id)
    }

    fn save_publication_evidence(
        &mut self,
        checkpoint_id: CheckpointId,
        evidence: &CheckpointPublicationEvidenceV1,
    ) -> Result<(), CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_persisted_artifact(&checkpoint_id)?;
        let manifest = self.load_persisted_archive_manifest(&checkpoint_id)?;
        let da_reference = self.load_persisted_da_reference(&checkpoint_id)?;
        if evidence.statement_core_digest() != manifest.statement_core_digest()
            || evidence.archive_manifest_root() != manifest.archive_manifest_root()
            || evidence.payload_commitment() != manifest.da_payload_commitment()
            || evidence.da_ref() != da_reference.da_ref()
        {
            return Err(CheckpointError::ArchiveMix);
        }
        self.persist_publication_evidence_bin(checkpoint_id, evidence)
    }

    fn load_publication_evidence(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointPublicationEvidenceV1, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_persisted_publication_evidence(checkpoint_id)
    }

    fn save_lifecycle(&mut self, lifecycle: &CheckpointLifecycleV1) -> Result<(), CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        let checkpoint_id = lifecycle.checkpoint_id();
        self.load_persisted_artifact(&checkpoint_id)?;
        if let Some(statement_core_digest) = lifecycle.statement_core_digest() {
            let manifest = self.load_persisted_archive_manifest(&checkpoint_id)?;
            if statement_core_digest != manifest.statement_core_digest() {
                return Err(CheckpointError::LifecycleMix);
            }
        }
        if let Some(publication_evidence_root) = lifecycle.publication_evidence_root() {
            let evidence = self.load_persisted_publication_evidence(&checkpoint_id)?;
            if evidence.publication_evidence_root() != publication_evidence_root {
                return Err(CheckpointError::LifecycleMix);
            }
        }
        self.persist_lifecycle_bin(lifecycle)
    }

    fn load_lifecycle(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointLifecycleV1, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.load_persisted_lifecycle(checkpoint_id)
    }

    fn save_recursive_sidecar(
        &mut self,
        checkpoint_id: CheckpointId,
        sidecar: &RecursiveCheckpointSidecarV1,
    ) -> Result<(), CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        self.check_recursive_sidecar_binding(checkpoint_id, sidecar)?;
        self.persist_recursive_sidecar_bin(checkpoint_id, sidecar)
    }

    fn load_recursive_sidecar(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<RecursiveCheckpointSidecarV1, CheckpointError> {
        self.reject_noncanonical_final_lane()?;
        let sidecar = self.load_persisted_recursive_sidecar(checkpoint_id)?;
        self.check_recursive_sidecar_binding(*checkpoint_id, &sidecar)?;
        Ok(sidecar)
    }
}
