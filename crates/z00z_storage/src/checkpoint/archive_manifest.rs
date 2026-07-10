use z00z_crypto::{expert::hash_domain, frame_bytes, frame_str, hash_zk::hash_zk};
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::{snapshot::PrepSnapshotId, CheckpointError};

use super::CheckpointExecInputId;

hash_domain!(
    StorCheckpointArchiveManifestDom,
    "z00z.storage.checkpoint.archive_manifest",
    1
);

const ARCHIVE_MANIFEST_BIND_VER: u8 = 1;
const ARCHIVE_MANIFEST_BIND_LABEL: &str = "checkpoint_archive_manifest_v1";
const ARCHIVE_ENTRY_BIND_LABEL: &str = "checkpoint_archive_entry_v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ArchiveManifestVersion(u8);

impl ArchiveManifestVersion {
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

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckpointArchiveEntryVersion(u8);

impl CheckpointArchiveEntryVersion {
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
pub enum CheckpointArchiveEntryKindV1 {
    RawTxPackage,
    ExactTxProofBytes,
    WitnessArchive,
    DeltaJournal,
    ArchiveProviderReceipt,
    RetrievalAudit,
    ContentAddressIndex,
}

impl CheckpointArchiveEntryKindV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawTxPackage => "raw_tx_package",
            Self::ExactTxProofBytes => "exact_tx_proof_bytes",
            Self::WitnessArchive => "witness_archive",
            Self::DeltaJournal => "delta_journal",
            Self::ArchiveProviderReceipt => "archive_provider_receipt",
            Self::RetrievalAudit => "retrieval_audit",
            Self::ContentAddressIndex => "content_address_index",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointArchiveRetentionClassV1 {
    ArchiveRequired,
    DisputeRequired,
    AuditRequired,
}

impl CheckpointArchiveRetentionClassV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ArchiveRequired => "archive_required",
            Self::DisputeRequired => "dispute_required",
            Self::AuditRequired => "audit_required",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointArchiveEncodingKindV1 {
    CanonicalBinV1,
    CanonicalJsonV1,
    ProviderPayloadV1,
}

impl CheckpointArchiveEncodingKindV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalBinV1 => "canonical_bin_v1",
            Self::CanonicalJsonV1 => "canonical_json_v1",
            Self::ProviderPayloadV1 => "provider_payload_v1",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointArchiveEntryV1 {
    version: CheckpointArchiveEntryVersion,
    entry_kind: CheckpointArchiveEntryKindV1,
    ordinal: u32,
    content_digest: [u8; 32],
    byte_length: u64,
    retention_class: CheckpointArchiveRetentionClassV1,
    encoding_kind: CheckpointArchiveEncodingKindV1,
}

impl CheckpointArchiveEntryV1 {
    pub fn new(
        version: CheckpointArchiveEntryVersion,
        entry_kind: CheckpointArchiveEntryKindV1,
        ordinal: u32,
        content_digest: [u8; 32],
        byte_length: u64,
        retention_class: CheckpointArchiveRetentionClassV1,
        encoding_kind: CheckpointArchiveEncodingKindV1,
    ) -> Result<Self, CheckpointError> {
        check_archive_entry_ver(version)?;
        if is_zero_root(&content_digest) || byte_length == 0 {
            return Err(CheckpointError::ArchiveMix);
        }
        Ok(Self {
            version,
            entry_kind,
            ordinal,
            content_digest,
            byte_length,
            retention_class,
            encoding_kind,
        })
    }

    #[must_use]
    pub const fn version(&self) -> CheckpointArchiveEntryVersion {
        self.version
    }

    #[must_use]
    pub const fn entry_kind(&self) -> CheckpointArchiveEntryKindV1 {
        self.entry_kind
    }

    #[must_use]
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }

    #[must_use]
    pub const fn content_digest(&self) -> [u8; 32] {
        self.content_digest
    }

    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }

    #[must_use]
    pub const fn retention_class(&self) -> CheckpointArchiveRetentionClassV1 {
        self.retention_class
    }

    #[must_use]
    pub const fn encoding_kind(&self) -> CheckpointArchiveEncodingKindV1 {
        self.encoding_kind
    }

    #[must_use]
    pub fn entry_bind(&self) -> [u8; 32] {
        hash_zk::<StorCheckpointArchiveManifestDom>(
            ARCHIVE_ENTRY_BIND_LABEL,
            &[self.bind_bytes().as_slice()],
        )
    }

    fn check_bind(&self) -> Result<(), CheckpointError> {
        check_archive_entry_ver(self.version)?;
        if is_zero_root(&self.content_digest) || self.byte_length == 0 {
            return Err(CheckpointError::ArchiveMix);
        }
        Ok(())
    }

    fn bind_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_framed_field(&mut bytes, "version", &[self.version.as_u8()]);
        push_framed_field(
            &mut bytes,
            "entry_kind",
            self.entry_kind.as_str().as_bytes(),
        );
        push_framed_field(&mut bytes, "ordinal", &self.ordinal.to_le_bytes());
        push_framed_field(&mut bytes, "content_digest", &self.content_digest);
        push_framed_field(&mut bytes, "byte_length", &self.byte_length.to_le_bytes());
        push_framed_field(
            &mut bytes,
            "retention_class",
            self.retention_class.as_str().as_bytes(),
        );
        push_framed_field(
            &mut bytes,
            "encoding_kind",
            self.encoding_kind.as_str().as_bytes(),
        );
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointArchiveManifestV1 {
    version: ArchiveManifestVersion,
    statement_core_digest: [u8; 32],
    checkpoint_exec_input_id: CheckpointExecInputId,
    prep_snapshot_id: PrepSnapshotId,
    tx_data_root: [u8; 32],
    delta_root: [u8; 32],
    witness_root: [u8; 32],
    journal_digest: [u8; 32],
    epoch_manifest_root: [u8; 32],
    raw_tx_package_root: [u8; 32],
    exact_tx_proof_bytes_root: [u8; 32],
    witness_archive_root: [u8; 32],
    delta_journal_root: [u8; 32],
    da_payload_commitment: [u8; 32],
    archive_provider_receipt_root: [u8; 32],
    retrieval_audit_root: [u8; 32],
    content_address_root: [u8; 32],
    entries: Vec<CheckpointArchiveEntryV1>,
    min_archive_replicas: u32,
    #[serde(default)]
    archive_manifest_bind_ver: u8,
    #[serde(default)]
    archive_manifest_bind: [u8; 32],
}

impl CheckpointArchiveManifestV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: ArchiveManifestVersion,
        statement_core_digest: [u8; 32],
        checkpoint_exec_input_id: CheckpointExecInputId,
        prep_snapshot_id: PrepSnapshotId,
        tx_data_root: [u8; 32],
        delta_root: [u8; 32],
        witness_root: [u8; 32],
        journal_digest: [u8; 32],
        epoch_manifest_root: [u8; 32],
        raw_tx_package_root: [u8; 32],
        exact_tx_proof_bytes_root: [u8; 32],
        witness_archive_root: [u8; 32],
        delta_journal_root: [u8; 32],
        da_payload_commitment: [u8; 32],
        archive_provider_receipt_root: [u8; 32],
        retrieval_audit_root: [u8; 32],
        content_address_root: [u8; 32],
        entries: Vec<CheckpointArchiveEntryV1>,
        min_archive_replicas: u32,
    ) -> Result<Self, CheckpointError> {
        check_archive_manifest_ver(version)?;
        if min_archive_replicas < 3
            || is_zero_id(checkpoint_exec_input_id.as_bytes())
            || is_zero_id(prep_snapshot_id.as_bytes())
            || entries.is_empty()
        {
            return Err(CheckpointError::ArchiveMix);
        }
        let roots = [
            statement_core_digest,
            tx_data_root,
            delta_root,
            witness_root,
            journal_digest,
            epoch_manifest_root,
            raw_tx_package_root,
            exact_tx_proof_bytes_root,
            witness_archive_root,
            delta_journal_root,
            da_payload_commitment,
            archive_provider_receipt_root,
            retrieval_audit_root,
            content_address_root,
        ];
        if roots.iter().any(is_zero_root) {
            return Err(CheckpointError::ArchiveMix);
        }
        for (index, entry) in entries.iter().enumerate() {
            entry.check_bind()?;
            if entry.ordinal() != index as u32 {
                return Err(CheckpointError::ArchiveMix);
            }
        }
        let archive_manifest_bind = archive_manifest_bind(
            statement_core_digest,
            checkpoint_exec_input_id,
            prep_snapshot_id,
            tx_data_root,
            delta_root,
            witness_root,
            journal_digest,
            epoch_manifest_root,
            raw_tx_package_root,
            exact_tx_proof_bytes_root,
            witness_archive_root,
            delta_journal_root,
            da_payload_commitment,
            archive_provider_receipt_root,
            retrieval_audit_root,
            content_address_root,
            &entries,
            min_archive_replicas,
        );
        Ok(Self {
            version,
            statement_core_digest,
            checkpoint_exec_input_id,
            prep_snapshot_id,
            tx_data_root,
            delta_root,
            witness_root,
            journal_digest,
            epoch_manifest_root,
            raw_tx_package_root,
            exact_tx_proof_bytes_root,
            witness_archive_root,
            delta_journal_root,
            da_payload_commitment,
            archive_provider_receipt_root,
            retrieval_audit_root,
            content_address_root,
            entries,
            min_archive_replicas,
            archive_manifest_bind_ver: ARCHIVE_MANIFEST_BIND_VER,
            archive_manifest_bind,
        })
    }

    #[must_use]
    pub const fn version(&self) -> ArchiveManifestVersion {
        self.version
    }

    #[must_use]
    pub const fn statement_core_digest(&self) -> [u8; 32] {
        self.statement_core_digest
    }

    #[must_use]
    pub const fn checkpoint_exec_input_id(&self) -> CheckpointExecInputId {
        self.checkpoint_exec_input_id
    }

    #[must_use]
    pub const fn prep_snapshot_id(&self) -> PrepSnapshotId {
        self.prep_snapshot_id
    }

    #[must_use]
    pub const fn tx_data_root(&self) -> [u8; 32] {
        self.tx_data_root
    }

    #[must_use]
    pub const fn delta_root(&self) -> [u8; 32] {
        self.delta_root
    }

    #[must_use]
    pub const fn witness_root(&self) -> [u8; 32] {
        self.witness_root
    }

    #[must_use]
    pub const fn journal_digest(&self) -> [u8; 32] {
        self.journal_digest
    }

    #[must_use]
    pub const fn epoch_manifest_root(&self) -> [u8; 32] {
        self.epoch_manifest_root
    }

    #[must_use]
    pub const fn raw_tx_package_root(&self) -> [u8; 32] {
        self.raw_tx_package_root
    }

    #[must_use]
    pub const fn exact_tx_proof_bytes_root(&self) -> [u8; 32] {
        self.exact_tx_proof_bytes_root
    }

    #[must_use]
    pub const fn witness_archive_root(&self) -> [u8; 32] {
        self.witness_archive_root
    }

    #[must_use]
    pub const fn delta_journal_root(&self) -> [u8; 32] {
        self.delta_journal_root
    }

    #[must_use]
    pub const fn da_payload_commitment(&self) -> [u8; 32] {
        self.da_payload_commitment
    }

    #[must_use]
    pub const fn archive_provider_receipt_root(&self) -> [u8; 32] {
        self.archive_provider_receipt_root
    }

    #[must_use]
    pub const fn retrieval_audit_root(&self) -> [u8; 32] {
        self.retrieval_audit_root
    }

    #[must_use]
    pub const fn content_address_root(&self) -> [u8; 32] {
        self.content_address_root
    }

    #[must_use]
    pub fn entries(&self) -> &[CheckpointArchiveEntryV1] {
        &self.entries
    }

    #[must_use]
    pub const fn min_archive_replicas(&self) -> u32 {
        self.min_archive_replicas
    }

    #[must_use]
    pub const fn archive_manifest_root(&self) -> [u8; 32] {
        self.archive_manifest_bind
    }

    fn check_bind(&self) -> Result<(), CheckpointError> {
        if self.archive_manifest_bind_ver != ARCHIVE_MANIFEST_BIND_VER
            || is_zero_id(self.checkpoint_exec_input_id.as_bytes())
            || is_zero_id(self.prep_snapshot_id.as_bytes())
            || self.entries.is_empty()
        {
            return Err(CheckpointError::ArchiveMix);
        }
        for (index, entry) in self.entries.iter().enumerate() {
            entry.check_bind()?;
            if entry.ordinal() != index as u32 {
                return Err(CheckpointError::ArchiveMix);
            }
        }
        if self.archive_manifest_bind
            != archive_manifest_bind(
                self.statement_core_digest,
                self.checkpoint_exec_input_id,
                self.prep_snapshot_id,
                self.tx_data_root,
                self.delta_root,
                self.witness_root,
                self.journal_digest,
                self.epoch_manifest_root,
                self.raw_tx_package_root,
                self.exact_tx_proof_bytes_root,
                self.witness_archive_root,
                self.delta_journal_root,
                self.da_payload_commitment,
                self.archive_provider_receipt_root,
                self.retrieval_audit_root,
                self.content_address_root,
                &self.entries,
                self.min_archive_replicas,
            )
        {
            return Err(CheckpointError::ArchiveMix);
        }
        Ok(())
    }
}

pub(crate) fn check_archive_manifest_ver(
    version: ArchiveManifestVersion,
) -> Result<(), CheckpointError> {
    if version == ArchiveManifestVersion::CURRENT {
        return Ok(());
    }
    Err(CheckpointError::VersionMix)
}

pub(crate) fn check_archive_entry_ver(
    version: CheckpointArchiveEntryVersion,
) -> Result<(), CheckpointError> {
    if version == CheckpointArchiveEntryVersion::CURRENT {
        return Ok(());
    }
    Err(CheckpointError::VersionMix)
}

pub(crate) fn encode_archive_manifest_bin_checked(
    manifest: &CheckpointArchiveManifestV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_archive_manifest_ver(manifest.version())?;
    manifest.check_bind()?;
    Ok(BincodeCodec.serialize(manifest)?)
}

pub(crate) fn decode_archive_manifest_bin_checked(
    bytes: &[u8],
) -> Result<CheckpointArchiveManifestV1, CheckpointError> {
    let manifest: CheckpointArchiveManifestV1 = BincodeCodec.deserialize(bytes)?;
    check_archive_manifest_ver(manifest.version())?;
    manifest.check_bind()?;
    Ok(manifest)
}

pub(crate) fn encode_archive_manifest_json_checked(
    manifest: &CheckpointArchiveManifestV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_archive_manifest_ver(manifest.version())?;
    manifest.check_bind()?;
    Ok(JsonCodec.serialize_pretty(manifest)?)
}

pub(crate) fn decode_archive_manifest_json_checked(
    bytes: &[u8],
) -> Result<CheckpointArchiveManifestV1, CheckpointError> {
    let manifest: CheckpointArchiveManifestV1 = JsonCodec.deserialize(bytes)?;
    check_archive_manifest_ver(manifest.version())?;
    manifest.check_bind()?;
    Ok(manifest)
}

#[allow(clippy::too_many_arguments)]
fn archive_manifest_bind(
    statement_core_digest: [u8; 32],
    checkpoint_exec_input_id: CheckpointExecInputId,
    prep_snapshot_id: PrepSnapshotId,
    tx_data_root: [u8; 32],
    delta_root: [u8; 32],
    witness_root: [u8; 32],
    journal_digest: [u8; 32],
    epoch_manifest_root: [u8; 32],
    raw_tx_package_root: [u8; 32],
    exact_tx_proof_bytes_root: [u8; 32],
    witness_archive_root: [u8; 32],
    delta_journal_root: [u8; 32],
    da_payload_commitment: [u8; 32],
    archive_provider_receipt_root: [u8; 32],
    retrieval_audit_root: [u8; 32],
    content_address_root: [u8; 32],
    entries: &[CheckpointArchiveEntryV1],
    min_archive_replicas: u32,
) -> [u8; 32] {
    let mut bytes = Vec::new();
    push_framed_field(&mut bytes, "statement_core_digest", &statement_core_digest);
    push_framed_field(
        &mut bytes,
        "checkpoint_exec_input_id",
        checkpoint_exec_input_id.as_bytes(),
    );
    push_framed_field(&mut bytes, "prep_snapshot_id", prep_snapshot_id.as_bytes());
    push_framed_field(&mut bytes, "tx_data_root", &tx_data_root);
    push_framed_field(&mut bytes, "delta_root", &delta_root);
    push_framed_field(&mut bytes, "witness_root", &witness_root);
    push_framed_field(&mut bytes, "journal_digest", &journal_digest);
    push_framed_field(&mut bytes, "epoch_manifest_root", &epoch_manifest_root);
    push_framed_field(&mut bytes, "raw_tx_package_root", &raw_tx_package_root);
    push_framed_field(
        &mut bytes,
        "exact_tx_proof_bytes_root",
        &exact_tx_proof_bytes_root,
    );
    push_framed_field(&mut bytes, "witness_archive_root", &witness_archive_root);
    push_framed_field(&mut bytes, "delta_journal_root", &delta_journal_root);
    push_framed_field(&mut bytes, "da_payload_commitment", &da_payload_commitment);
    push_framed_field(
        &mut bytes,
        "archive_provider_receipt_root",
        &archive_provider_receipt_root,
    );
    push_framed_field(&mut bytes, "retrieval_audit_root", &retrieval_audit_root);
    push_framed_field(&mut bytes, "content_address_root", &content_address_root);
    push_framed_field(&mut bytes, "entries", &encode_entries(entries));
    push_framed_field(
        &mut bytes,
        "min_archive_replicas",
        &min_archive_replicas.to_le_bytes(),
    );
    hash_zk::<StorCheckpointArchiveManifestDom>(ARCHIVE_MANIFEST_BIND_LABEL, &[bytes.as_slice()])
}

fn is_zero_root(root: &[u8; 32]) -> bool {
    root.iter().all(|byte| *byte == 0)
}

fn is_zero_id(bytes: &[u8; 32]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}

fn push_framed_field(out: &mut Vec<u8>, name: &str, value: &[u8]) {
    out.extend_from_slice(&frame_str(name));
    out.extend_from_slice(&frame_bytes(value));
}

fn encode_entries(entries: &[CheckpointArchiveEntryV1]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(entries.len() as u64).to_le_bytes());
    for entry in entries {
        bytes.extend_from_slice(&frame_bytes(&entry.entry_bind()));
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::{
        check_archive_entry_ver, check_archive_manifest_ver, ArchiveManifestVersion,
        CheckpointArchiveEncodingKindV1, CheckpointArchiveEntryKindV1, CheckpointArchiveEntryV1,
        CheckpointArchiveEntryVersion, CheckpointArchiveManifestV1,
        CheckpointArchiveRetentionClassV1,
    };
    use crate::{checkpoint::CheckpointExecInputId, snapshot::PrepSnapshotId, CheckpointError};

    fn root(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn entries() -> Vec<CheckpointArchiveEntryV1> {
        vec![
            CheckpointArchiveEntryV1::new(
                CheckpointArchiveEntryVersion::CURRENT,
                CheckpointArchiveEntryKindV1::RawTxPackage,
                0,
                root(31),
                4096,
                CheckpointArchiveRetentionClassV1::ArchiveRequired,
                CheckpointArchiveEncodingKindV1::CanonicalBinV1,
            )
            .expect("entry 0"),
            CheckpointArchiveEntryV1::new(
                CheckpointArchiveEntryVersion::CURRENT,
                CheckpointArchiveEntryKindV1::ExactTxProofBytes,
                1,
                root(32),
                2048,
                CheckpointArchiveRetentionClassV1::DisputeRequired,
                CheckpointArchiveEncodingKindV1::CanonicalBinV1,
            )
            .expect("entry 1"),
        ]
    }

    fn manifest() -> CheckpointArchiveManifestV1 {
        CheckpointArchiveManifestV1::new(
            ArchiveManifestVersion::CURRENT,
            root(1),
            CheckpointExecInputId::new([11u8; 32]),
            PrepSnapshotId::new([12u8; 32]),
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            root(9),
            root(10),
            root(11),
            root(12),
            root(13),
            root(14),
            entries(),
            3,
        )
        .expect("archive manifest")
    }

    #[test]
    fn test_archive_manifest_builds() {
        let got = manifest();

        assert_eq!(got.min_archive_replicas(), 3);
        assert_eq!(got.exact_tx_proof_bytes_root(), root(8));
        assert_eq!(got.statement_core_digest(), root(1));
        assert_eq!(
            got.checkpoint_exec_input_id(),
            CheckpointExecInputId::new([11u8; 32])
        );
        assert_eq!(got.prep_snapshot_id(), PrepSnapshotId::new([12u8; 32]));
        assert_eq!(got.entries().len(), 2);
        assert_ne!(got.archive_manifest_root(), [0u8; 32]);
    }

    #[test]
    fn test_manifest_min_replicas() {
        let err = CheckpointArchiveManifestV1::new(
            ArchiveManifestVersion::CURRENT,
            root(1),
            CheckpointExecInputId::new([11u8; 32]),
            PrepSnapshotId::new([12u8; 32]),
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            root(9),
            root(10),
            root(11),
            root(12),
            root(13),
            root(14),
            entries(),
            2,
        )
        .expect_err("low replica count rejects");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_manifest_nonzero_roots() {
        let err = CheckpointArchiveManifestV1::new(
            ArchiveManifestVersion::CURRENT,
            [0u8; 32],
            CheckpointExecInputId::new([11u8; 32]),
            PrepSnapshotId::new([12u8; 32]),
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            root(9),
            root(10),
            root(11),
            root(12),
            root(13),
            root(14),
            entries(),
            3,
        )
        .expect_err("zero root rejects");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_manifest_bad_version() {
        let err =
            check_archive_manifest_ver(ArchiveManifestVersion::new(9)).expect_err("bad version");

        assert!(matches!(err, CheckpointError::VersionMix));
    }

    #[test]
    fn test_archive_entry_bad_version() {
        let err =
            check_archive_entry_ver(CheckpointArchiveEntryVersion::new(9)).expect_err("bad entry");

        assert!(matches!(err, CheckpointError::VersionMix));
    }

    #[test]
    fn test_manifest_rejects_non_contiguous_ordinals() {
        let err = CheckpointArchiveManifestV1::new(
            ArchiveManifestVersion::CURRENT,
            root(1),
            CheckpointExecInputId::new([11u8; 32]),
            PrepSnapshotId::new([12u8; 32]),
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            root(9),
            root(10),
            root(11),
            root(12),
            root(13),
            root(14),
            vec![CheckpointArchiveEntryV1::new(
                CheckpointArchiveEntryVersion::CURRENT,
                CheckpointArchiveEntryKindV1::RawTxPackage,
                1,
                root(31),
                4096,
                CheckpointArchiveRetentionClassV1::ArchiveRequired,
                CheckpointArchiveEncodingKindV1::CanonicalBinV1,
            )
            .expect("entry")],
            3,
        )
        .expect_err("non contiguous ordinal rejects");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }
}
