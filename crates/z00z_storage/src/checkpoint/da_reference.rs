use z00z_crypto::{expert::hash_domain, frame_bytes, frame_str, hash_zk::hash_zk};
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::CheckpointError;

hash_domain!(
    StorCheckpointDaReferenceDom,
    "z00z.storage.checkpoint.da_reference",
    1
);

const DA_REFERENCE_BIND_VER: u8 = 1;
const DA_REFERENCE_BIND_LABEL: &str = "checkpoint_da_reference_v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckpointDaReferenceVersion(u8);

impl CheckpointDaReferenceVersion {
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
pub enum CheckpointDaProviderFamily {
    LocalArchive,
    ContentAddressedBlob,
    NamespaceBlob,
    ObjectStoreMirror,
}

impl CheckpointDaProviderFamily {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LocalArchive => "local_archive",
            Self::ContentAddressedBlob => "content_addressed_blob",
            Self::NamespaceBlob => "namespace_blob",
            Self::ObjectStoreMirror => "object_store_mirror",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointDaLocatorKind {
    OpaqueProviderRef,
    ContentCid,
    NamespaceLocator,
}

impl CheckpointDaLocatorKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpaqueProviderRef => "opaque_provider_ref",
            Self::ContentCid => "content_cid",
            Self::NamespaceLocator => "namespace_locator",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointDaReferenceV1 {
    version: CheckpointDaReferenceVersion,
    provider_family: CheckpointDaProviderFamily,
    locator_kind: CheckpointDaLocatorKind,
    locator_value: String,
    payload_commitment: [u8; 32],
    statement_core_digest: [u8; 32],
    archive_manifest_root: [u8; 32],
    published_height: u64,
    #[serde(default)]
    da_reference_bind_ver: u8,
    #[serde(default)]
    da_reference_bind: [u8; 32],
}

impl CheckpointDaReferenceV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: CheckpointDaReferenceVersion,
        provider_family: CheckpointDaProviderFamily,
        locator_kind: CheckpointDaLocatorKind,
        locator_value: impl Into<String>,
        payload_commitment: [u8; 32],
        statement_core_digest: [u8; 32],
        archive_manifest_root: [u8; 32],
        published_height: u64,
    ) -> Result<Self, CheckpointError> {
        check_da_reference_ver(version)?;
        let locator_value = locator_value.into();
        if locator_value.trim().is_empty()
            || !matches!(locator_kind, CheckpointDaLocatorKind::OpaqueProviderRef)
            || published_height == 0
            || is_zero_root(&payload_commitment)
            || is_zero_root(&statement_core_digest)
            || is_zero_root(&archive_manifest_root)
        {
            return Err(CheckpointError::ArchiveMix);
        }
        let da_reference_bind = da_reference_bind(
            provider_family,
            locator_kind,
            &locator_value,
            payload_commitment,
            statement_core_digest,
            archive_manifest_root,
            published_height,
        );
        Ok(Self {
            version,
            provider_family,
            locator_kind,
            locator_value,
            payload_commitment,
            statement_core_digest,
            archive_manifest_root,
            published_height,
            da_reference_bind_ver: DA_REFERENCE_BIND_VER,
            da_reference_bind,
        })
    }

    #[must_use]
    pub const fn version(&self) -> CheckpointDaReferenceVersion {
        self.version
    }

    #[must_use]
    pub const fn provider_family(&self) -> CheckpointDaProviderFamily {
        self.provider_family
    }

    #[must_use]
    pub const fn locator_kind(&self) -> CheckpointDaLocatorKind {
        self.locator_kind
    }

    #[must_use]
    pub fn locator_value(&self) -> &str {
        &self.locator_value
    }

    #[must_use]
    pub const fn payload_commitment(&self) -> [u8; 32] {
        self.payload_commitment
    }

    #[must_use]
    pub const fn statement_core_digest(&self) -> [u8; 32] {
        self.statement_core_digest
    }

    #[must_use]
    pub const fn archive_manifest_root(&self) -> [u8; 32] {
        self.archive_manifest_root
    }

    #[must_use]
    pub const fn published_height(&self) -> u64 {
        self.published_height
    }

    #[must_use]
    pub const fn da_ref(&self) -> [u8; 32] {
        self.da_reference_bind
    }

    fn check_bind(&self) -> Result<(), CheckpointError> {
        if self.da_reference_bind_ver != DA_REFERENCE_BIND_VER {
            return Err(CheckpointError::ArchiveMix);
        }
        if self.da_reference_bind
            != da_reference_bind(
                self.provider_family,
                self.locator_kind,
                &self.locator_value,
                self.payload_commitment,
                self.statement_core_digest,
                self.archive_manifest_root,
                self.published_height,
            )
        {
            return Err(CheckpointError::ArchiveMix);
        }
        Ok(())
    }
}

pub(crate) fn check_da_reference_ver(
    version: CheckpointDaReferenceVersion,
) -> Result<(), CheckpointError> {
    if version == CheckpointDaReferenceVersion::CURRENT {
        return Ok(());
    }
    Err(CheckpointError::VersionMix)
}

pub(crate) fn encode_da_reference_bin_checked(
    da_reference: &CheckpointDaReferenceV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_da_reference_ver(da_reference.version())?;
    da_reference.check_bind()?;
    Ok(BincodeCodec.serialize(da_reference)?)
}

pub(crate) fn decode_da_reference_bin_checked(
    bytes: &[u8],
) -> Result<CheckpointDaReferenceV1, CheckpointError> {
    let da_reference: CheckpointDaReferenceV1 = BincodeCodec.deserialize(bytes)?;
    check_da_reference_ver(da_reference.version())?;
    da_reference.check_bind()?;
    Ok(da_reference)
}

pub(crate) fn encode_da_reference_json_checked(
    da_reference: &CheckpointDaReferenceV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_da_reference_ver(da_reference.version())?;
    da_reference.check_bind()?;
    Ok(JsonCodec.serialize_pretty(da_reference)?)
}

pub(crate) fn decode_da_reference_json_checked(
    bytes: &[u8],
) -> Result<CheckpointDaReferenceV1, CheckpointError> {
    let da_reference: CheckpointDaReferenceV1 = JsonCodec.deserialize(bytes)?;
    check_da_reference_ver(da_reference.version())?;
    da_reference.check_bind()?;
    Ok(da_reference)
}

fn da_reference_bind(
    provider_family: CheckpointDaProviderFamily,
    locator_kind: CheckpointDaLocatorKind,
    locator_value: &str,
    payload_commitment: [u8; 32],
    statement_core_digest: [u8; 32],
    archive_manifest_root: [u8; 32],
    published_height: u64,
) -> [u8; 32] {
    let mut bytes = Vec::new();
    push_framed_field(
        &mut bytes,
        "provider_family",
        provider_family.as_str().as_bytes(),
    );
    push_framed_field(&mut bytes, "locator_kind", locator_kind.as_str().as_bytes());
    push_framed_field(&mut bytes, "locator_value", locator_value.as_bytes());
    push_framed_field(&mut bytes, "payload_commitment", &payload_commitment);
    push_framed_field(&mut bytes, "statement_core_digest", &statement_core_digest);
    push_framed_field(&mut bytes, "archive_manifest_root", &archive_manifest_root);
    push_framed_field(
        &mut bytes,
        "published_height",
        &published_height.to_le_bytes(),
    );
    hash_zk::<StorCheckpointDaReferenceDom>(DA_REFERENCE_BIND_LABEL, &[bytes.as_slice()])
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
        check_da_reference_ver, CheckpointDaLocatorKind, CheckpointDaProviderFamily,
        CheckpointDaReferenceV1, CheckpointDaReferenceVersion,
    };
    use crate::CheckpointError;

    fn root(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn da_reference() -> CheckpointDaReferenceV1 {
        CheckpointDaReferenceV1::new(
            CheckpointDaReferenceVersion::CURRENT,
            CheckpointDaProviderFamily::LocalArchive,
            CheckpointDaLocatorKind::OpaqueProviderRef,
            "local-da://proof/001",
            root(1),
            root(2),
            root(3),
            44,
        )
        .expect("da reference")
    }

    #[test]
    fn test_da_reference_builds() {
        let got = da_reference();

        assert_eq!(
            got.provider_family(),
            CheckpointDaProviderFamily::LocalArchive
        );
        assert_eq!(got.published_height(), 44);
        assert_ne!(got.da_ref(), [0u8; 32]);
    }

    #[test]
    fn test_da_ref_rejects_empty() {
        let err = CheckpointDaReferenceV1::new(
            CheckpointDaReferenceVersion::CURRENT,
            CheckpointDaProviderFamily::LocalArchive,
            CheckpointDaLocatorKind::OpaqueProviderRef,
            "",
            root(1),
            root(2),
            root(3),
            44,
        )
        .expect_err("empty locator must reject");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_da_ref_rejects_zero() {
        let err = CheckpointDaReferenceV1::new(
            CheckpointDaReferenceVersion::CURRENT,
            CheckpointDaProviderFamily::LocalArchive,
            CheckpointDaLocatorKind::OpaqueProviderRef,
            "local-da://proof/001",
            root(1),
            root(2),
            root(3),
            0,
        )
        .expect_err("zero height must reject");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_da_rejects_bare_kind() {
        let err = CheckpointDaReferenceV1::new(
            CheckpointDaReferenceVersion::CURRENT,
            CheckpointDaProviderFamily::LocalArchive,
            CheckpointDaLocatorKind::ContentCid,
            "bafybarecid",
            root(1),
            root(2),
            root(3),
            44,
        )
        .expect_err("bare cid locator must reject");

        assert!(matches!(err, CheckpointError::ArchiveMix));
    }

    #[test]
    fn test_da_reference_bad_version() {
        let err = check_da_reference_ver(CheckpointDaReferenceVersion::new(9))
            .expect_err("bad version must reject");

        assert!(matches!(err, CheckpointError::VersionMix));
    }
}
