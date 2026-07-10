use std::fmt;

use z00z_crypto::{expert::hash_domain, frame_bytes, frame_str, hash_zk::hash_zk};
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::CheckpointError;

hash_domain!(
    StorCheckpointPqAnchorDom,
    "z00z.storage.checkpoint.pq_anchor",
    1
);

const PQ_ANCHOR_BIND_VER: u8 = 1;
const PQ_ANCHOR_STATEMENT_LABEL: &str = "post_quantum_checkpoint_anchor_statement_v1";
const PQ_ANCHOR_BIND_LABEL: &str = "post_quantum_checkpoint_anchor_v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct PostQuantumCheckpointAnchorVersion(u8);

impl PostQuantumCheckpointAnchorVersion {
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
pub enum PostQuantumCheckpointAnchorModeV1 {
    Plonky3EpochProof,
}

impl PostQuantumCheckpointAnchorModeV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Plonky3EpochProof => "plonky3_epoch_proof",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostQuantumCheckpointEnforcementStageV1 {
    PqAnchorWriter,
}

impl PostQuantumCheckpointEnforcementStageV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PqAnchorWriter => "pq_anchor_writer",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostQuantumCheckpointAnchorRejectReasonV1 {
    UnsupportedVersion,
    HeightZero,
    CadenceZero,
    NonCadenceHeight,
    StatementDigestMissing,
    DeltaRootMissing,
    WitnessRootMissing,
    ArchiveManifestRootMissing,
    Plonky3EpochStatementDigestMissing,
    Plonky3EpochProofDigestMissing,
    Plonky3PublicInputsDigestMissing,
    NovaChainRootMissing,
    PqSignatureMissing,
    PqStatementDigestMismatch,
    PqAnchorRootMismatch,
}

impl PostQuantumCheckpointAnchorRejectReasonV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedVersion => "unsupported_version",
            Self::HeightZero => "height_zero",
            Self::CadenceZero => "cadence_zero",
            Self::NonCadenceHeight => "non_cadence_height",
            Self::StatementDigestMissing => "statement_digest_missing",
            Self::DeltaRootMissing => "delta_root_missing",
            Self::WitnessRootMissing => "witness_root_missing",
            Self::ArchiveManifestRootMissing => "archive_manifest_root_missing",
            Self::Plonky3EpochStatementDigestMissing => "plonky3_epoch_statement_digest_missing",
            Self::Plonky3EpochProofDigestMissing => "plonky3_epoch_proof_digest_missing",
            Self::Plonky3PublicInputsDigestMissing => "plonky3_public_inputs_digest_missing",
            Self::NovaChainRootMissing => "nova_chain_root_missing",
            Self::PqSignatureMissing => "pq_signature_missing",
            Self::PqStatementDigestMismatch => "pq_statement_digest_mismatch",
            Self::PqAnchorRootMismatch => "pq_anchor_root_mismatch",
        }
    }
}

impl fmt::Display for PostQuantumCheckpointAnchorRejectReasonV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<PostQuantumCheckpointAnchorRejectReasonV1> for CheckpointError {
    fn from(reason: PostQuantumCheckpointAnchorRejectReasonV1) -> Self {
        match reason {
            PostQuantumCheckpointAnchorRejectReasonV1::UnsupportedVersion => Self::VersionMix,
            other => Self::Backend(format!("pq anchor reject: {other}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostQuantumCheckpointAnchorV1 {
    version: PostQuantumCheckpointAnchorVersion,
    height: u64,
    cadence_blocks: u64,
    statement_digest: [u8; 32],
    pq_statement_digest: [u8; 32],
    pq_delta_root: [u8; 32],
    pq_witness_root: [u8; 32],
    pq_archive_manifest_root: [u8; 32],
    plonky3_epoch_statement_digest: [u8; 32],
    plonky3_epoch_proof_digest: [u8; 32],
    plonky3_public_inputs_digest: [u8; 32],
    nova_chain_root: [u8; 32],
    pq_signature_or_commitment: [u8; 32],
    mode: PostQuantumCheckpointAnchorModeV1,
    enforcement_stage: PostQuantumCheckpointEnforcementStageV1,
    #[serde(default)]
    pq_anchor_bind_ver: u8,
    #[serde(default)]
    pq_anchor_root: [u8; 32],
}

impl PostQuantumCheckpointAnchorV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: PostQuantumCheckpointAnchorVersion,
        height: u64,
        cadence_blocks: u64,
        statement_digest: [u8; 32],
        pq_delta_root: [u8; 32],
        pq_witness_root: [u8; 32],
        pq_archive_manifest_root: [u8; 32],
        plonky3_epoch_statement_digest: [u8; 32],
        plonky3_epoch_proof_digest: [u8; 32],
        plonky3_public_inputs_digest: [u8; 32],
        nova_chain_root: [u8; 32],
        pq_signature_or_commitment: [u8; 32],
        mode: PostQuantumCheckpointAnchorModeV1,
        enforcement_stage: PostQuantumCheckpointEnforcementStageV1,
    ) -> Result<Self, CheckpointError> {
        check_pq_anchor_ver(version)?;
        let pq_statement_digest = pq_statement_digest_v1(
            height,
            cadence_blocks,
            statement_digest,
            pq_delta_root,
            pq_witness_root,
            pq_archive_manifest_root,
            plonky3_epoch_statement_digest,
            plonky3_epoch_proof_digest,
            plonky3_public_inputs_digest,
            nova_chain_root,
            mode,
            enforcement_stage,
        );
        let pq_anchor_root = pq_anchor_root_v1(
            height,
            cadence_blocks,
            statement_digest,
            pq_statement_digest,
            pq_delta_root,
            pq_witness_root,
            pq_archive_manifest_root,
            plonky3_epoch_statement_digest,
            plonky3_epoch_proof_digest,
            plonky3_public_inputs_digest,
            nova_chain_root,
            pq_signature_or_commitment,
            mode,
            enforcement_stage,
        );
        let anchor = Self {
            version,
            height,
            cadence_blocks,
            statement_digest,
            pq_statement_digest,
            pq_delta_root,
            pq_witness_root,
            pq_archive_manifest_root,
            plonky3_epoch_statement_digest,
            plonky3_epoch_proof_digest,
            plonky3_public_inputs_digest,
            nova_chain_root,
            pq_signature_or_commitment,
            mode,
            enforcement_stage,
            pq_anchor_bind_ver: PQ_ANCHOR_BIND_VER,
            pq_anchor_root,
        };
        anchor.check_bind()?;
        Ok(anchor)
    }

    #[must_use]
    pub const fn version(&self) -> PostQuantumCheckpointAnchorVersion {
        self.version
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub const fn cadence_blocks(&self) -> u64 {
        self.cadence_blocks
    }

    #[must_use]
    pub const fn statement_digest(&self) -> [u8; 32] {
        self.statement_digest
    }

    #[must_use]
    pub const fn pq_statement_digest(&self) -> [u8; 32] {
        self.pq_statement_digest
    }

    #[must_use]
    pub const fn pq_delta_root(&self) -> [u8; 32] {
        self.pq_delta_root
    }

    #[must_use]
    pub const fn pq_witness_root(&self) -> [u8; 32] {
        self.pq_witness_root
    }

    #[must_use]
    pub const fn pq_archive_manifest_root(&self) -> [u8; 32] {
        self.pq_archive_manifest_root
    }

    #[must_use]
    pub const fn plonky3_epoch_statement_digest(&self) -> [u8; 32] {
        self.plonky3_epoch_statement_digest
    }

    #[must_use]
    pub const fn plonky3_epoch_proof_digest(&self) -> [u8; 32] {
        self.plonky3_epoch_proof_digest
    }

    #[must_use]
    pub const fn plonky3_public_inputs_digest(&self) -> [u8; 32] {
        self.plonky3_public_inputs_digest
    }

    #[must_use]
    pub const fn nova_chain_root(&self) -> [u8; 32] {
        self.nova_chain_root
    }

    #[must_use]
    pub const fn pq_signature_or_commitment(&self) -> [u8; 32] {
        self.pq_signature_or_commitment
    }

    #[must_use]
    pub const fn mode(&self) -> PostQuantumCheckpointAnchorModeV1 {
        self.mode
    }

    #[must_use]
    pub const fn enforcement_stage(&self) -> PostQuantumCheckpointEnforcementStageV1 {
        self.enforcement_stage
    }

    #[must_use]
    pub const fn pq_anchor_root(&self) -> [u8; 32] {
        self.pq_anchor_root
    }

    fn check_surface(&self) -> Result<(), CheckpointError> {
        if self.height == 0 {
            return Err(PostQuantumCheckpointAnchorRejectReasonV1::HeightZero.into());
        }
        if self.cadence_blocks == 0 {
            return Err(PostQuantumCheckpointAnchorRejectReasonV1::CadenceZero.into());
        }
        if !self.height.is_multiple_of(self.cadence_blocks) {
            return Err(PostQuantumCheckpointAnchorRejectReasonV1::NonCadenceHeight.into());
        }
        if is_zero_root(&self.statement_digest) {
            return Err(PostQuantumCheckpointAnchorRejectReasonV1::StatementDigestMissing.into());
        }
        if is_zero_root(&self.pq_delta_root) {
            return Err(PostQuantumCheckpointAnchorRejectReasonV1::DeltaRootMissing.into());
        }
        if is_zero_root(&self.pq_witness_root) {
            return Err(PostQuantumCheckpointAnchorRejectReasonV1::WitnessRootMissing.into());
        }
        if is_zero_root(&self.pq_archive_manifest_root) {
            return Err(
                PostQuantumCheckpointAnchorRejectReasonV1::ArchiveManifestRootMissing.into(),
            );
        }
        if is_zero_root(&self.plonky3_epoch_statement_digest) {
            return Err(
                PostQuantumCheckpointAnchorRejectReasonV1::Plonky3EpochStatementDigestMissing
                    .into(),
            );
        }
        if is_zero_root(&self.plonky3_epoch_proof_digest) {
            return Err(
                PostQuantumCheckpointAnchorRejectReasonV1::Plonky3EpochProofDigestMissing.into(),
            );
        }
        if is_zero_root(&self.plonky3_public_inputs_digest) {
            return Err(
                PostQuantumCheckpointAnchorRejectReasonV1::Plonky3PublicInputsDigestMissing.into(),
            );
        }
        if is_zero_root(&self.nova_chain_root) {
            return Err(PostQuantumCheckpointAnchorRejectReasonV1::NovaChainRootMissing.into());
        }
        if is_zero_root(&self.pq_signature_or_commitment) {
            return Err(PostQuantumCheckpointAnchorRejectReasonV1::PqSignatureMissing.into());
        }
        Ok(())
    }

    fn check_bind(&self) -> Result<(), CheckpointError> {
        self.check_surface()?;
        if self.pq_anchor_bind_ver != PQ_ANCHOR_BIND_VER {
            return Err(PostQuantumCheckpointAnchorRejectReasonV1::PqAnchorRootMismatch.into());
        }
        if self.pq_statement_digest
            != pq_statement_digest_v1(
                self.height,
                self.cadence_blocks,
                self.statement_digest,
                self.pq_delta_root,
                self.pq_witness_root,
                self.pq_archive_manifest_root,
                self.plonky3_epoch_statement_digest,
                self.plonky3_epoch_proof_digest,
                self.plonky3_public_inputs_digest,
                self.nova_chain_root,
                self.mode,
                self.enforcement_stage,
            )
        {
            return Err(
                PostQuantumCheckpointAnchorRejectReasonV1::PqStatementDigestMismatch.into(),
            );
        }
        if self.pq_anchor_root
            != pq_anchor_root_v1(
                self.height,
                self.cadence_blocks,
                self.statement_digest,
                self.pq_statement_digest,
                self.pq_delta_root,
                self.pq_witness_root,
                self.pq_archive_manifest_root,
                self.plonky3_epoch_statement_digest,
                self.plonky3_epoch_proof_digest,
                self.plonky3_public_inputs_digest,
                self.nova_chain_root,
                self.pq_signature_or_commitment,
                self.mode,
                self.enforcement_stage,
            )
        {
            return Err(PostQuantumCheckpointAnchorRejectReasonV1::PqAnchorRootMismatch.into());
        }
        Ok(())
    }
}

pub(crate) fn check_pq_anchor_ver(
    version: PostQuantumCheckpointAnchorVersion,
) -> Result<(), CheckpointError> {
    if version == PostQuantumCheckpointAnchorVersion::CURRENT {
        return Ok(());
    }
    Err(PostQuantumCheckpointAnchorRejectReasonV1::UnsupportedVersion.into())
}

pub(crate) fn encode_pq_anchor_bin_checked(
    anchor: &PostQuantumCheckpointAnchorV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_pq_anchor_ver(anchor.version())?;
    anchor.check_bind()?;
    Ok(BincodeCodec.serialize(anchor)?)
}

pub(crate) fn decode_pq_anchor_bin_checked(
    bytes: &[u8],
) -> Result<PostQuantumCheckpointAnchorV1, CheckpointError> {
    let anchor: PostQuantumCheckpointAnchorV1 = BincodeCodec.deserialize(bytes)?;
    check_pq_anchor_ver(anchor.version())?;
    anchor.check_bind()?;
    Ok(anchor)
}

pub(crate) fn encode_pq_anchor_json_checked(
    anchor: &PostQuantumCheckpointAnchorV1,
) -> Result<Vec<u8>, CheckpointError> {
    check_pq_anchor_ver(anchor.version())?;
    anchor.check_bind()?;
    Ok(JsonCodec.serialize_pretty(anchor)?)
}

pub(crate) fn decode_pq_anchor_json_checked(
    bytes: &[u8],
) -> Result<PostQuantumCheckpointAnchorV1, CheckpointError> {
    let anchor: PostQuantumCheckpointAnchorV1 = JsonCodec.deserialize(bytes)?;
    check_pq_anchor_ver(anchor.version())?;
    anchor.check_bind()?;
    Ok(anchor)
}

#[allow(clippy::too_many_arguments)]
fn pq_statement_digest_v1(
    height: u64,
    cadence_blocks: u64,
    statement_digest: [u8; 32],
    pq_delta_root: [u8; 32],
    pq_witness_root: [u8; 32],
    pq_archive_manifest_root: [u8; 32],
    plonky3_epoch_statement_digest: [u8; 32],
    plonky3_epoch_proof_digest: [u8; 32],
    plonky3_public_inputs_digest: [u8; 32],
    nova_chain_root: [u8; 32],
    mode: PostQuantumCheckpointAnchorModeV1,
    enforcement_stage: PostQuantumCheckpointEnforcementStageV1,
) -> [u8; 32] {
    let mut bytes = Vec::new();
    push_framed_field(&mut bytes, "height", &height.to_le_bytes());
    push_framed_field(&mut bytes, "cadence_blocks", &cadence_blocks.to_le_bytes());
    push_framed_field(&mut bytes, "statement_digest", &statement_digest);
    push_framed_field(&mut bytes, "pq_delta_root", &pq_delta_root);
    push_framed_field(&mut bytes, "pq_witness_root", &pq_witness_root);
    push_framed_field(
        &mut bytes,
        "pq_archive_manifest_root",
        &pq_archive_manifest_root,
    );
    push_framed_field(
        &mut bytes,
        "plonky3_epoch_statement_digest",
        &plonky3_epoch_statement_digest,
    );
    push_framed_field(
        &mut bytes,
        "plonky3_epoch_proof_digest",
        &plonky3_epoch_proof_digest,
    );
    push_framed_field(
        &mut bytes,
        "plonky3_public_inputs_digest",
        &plonky3_public_inputs_digest,
    );
    push_framed_field(&mut bytes, "nova_chain_root", &nova_chain_root);
    push_framed_field(&mut bytes, "mode", mode.as_str().as_bytes());
    push_framed_field(
        &mut bytes,
        "enforcement_stage",
        enforcement_stage.as_str().as_bytes(),
    );
    hash_zk::<StorCheckpointPqAnchorDom>(PQ_ANCHOR_STATEMENT_LABEL, &[bytes.as_slice()])
}

#[allow(clippy::too_many_arguments)]
fn pq_anchor_root_v1(
    height: u64,
    cadence_blocks: u64,
    statement_digest: [u8; 32],
    pq_statement_digest: [u8; 32],
    pq_delta_root: [u8; 32],
    pq_witness_root: [u8; 32],
    pq_archive_manifest_root: [u8; 32],
    plonky3_epoch_statement_digest: [u8; 32],
    plonky3_epoch_proof_digest: [u8; 32],
    plonky3_public_inputs_digest: [u8; 32],
    nova_chain_root: [u8; 32],
    pq_signature_or_commitment: [u8; 32],
    mode: PostQuantumCheckpointAnchorModeV1,
    enforcement_stage: PostQuantumCheckpointEnforcementStageV1,
) -> [u8; 32] {
    let mut bytes = Vec::new();
    push_framed_field(&mut bytes, "height", &height.to_le_bytes());
    push_framed_field(&mut bytes, "cadence_blocks", &cadence_blocks.to_le_bytes());
    push_framed_field(&mut bytes, "statement_digest", &statement_digest);
    push_framed_field(&mut bytes, "pq_statement_digest", &pq_statement_digest);
    push_framed_field(&mut bytes, "pq_delta_root", &pq_delta_root);
    push_framed_field(&mut bytes, "pq_witness_root", &pq_witness_root);
    push_framed_field(
        &mut bytes,
        "pq_archive_manifest_root",
        &pq_archive_manifest_root,
    );
    push_framed_field(
        &mut bytes,
        "plonky3_epoch_statement_digest",
        &plonky3_epoch_statement_digest,
    );
    push_framed_field(
        &mut bytes,
        "plonky3_epoch_proof_digest",
        &plonky3_epoch_proof_digest,
    );
    push_framed_field(
        &mut bytes,
        "plonky3_public_inputs_digest",
        &plonky3_public_inputs_digest,
    );
    push_framed_field(&mut bytes, "nova_chain_root", &nova_chain_root);
    push_framed_field(
        &mut bytes,
        "pq_signature_or_commitment",
        &pq_signature_or_commitment,
    );
    push_framed_field(&mut bytes, "mode", mode.as_str().as_bytes());
    push_framed_field(
        &mut bytes,
        "enforcement_stage",
        enforcement_stage.as_str().as_bytes(),
    );
    hash_zk::<StorCheckpointPqAnchorDom>(PQ_ANCHOR_BIND_LABEL, &[bytes.as_slice()])
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
        decode_pq_anchor_json_checked, pq_anchor_root_v1, pq_statement_digest_v1,
        PostQuantumCheckpointAnchorModeV1, PostQuantumCheckpointAnchorV1,
        PostQuantumCheckpointAnchorVersion, PostQuantumCheckpointEnforcementStageV1,
    };
    use crate::CheckpointError;

    fn root(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    #[test]
    fn test_decode_rejects_missing_required_pq_artifact_even_with_matching_bindings() {
        let anchor = PostQuantumCheckpointAnchorV1::new(
            PostQuantumCheckpointAnchorVersion::CURRENT,
            1000,
            1000,
            root(1),
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            root(9),
            PostQuantumCheckpointAnchorModeV1::Plonky3EpochProof,
            PostQuantumCheckpointEnforcementStageV1::PqAnchorWriter,
        )
        .expect("valid pq anchor");
        let mut value = serde_json::to_value(anchor).expect("pq anchor json");
        let statement_digest = [0u8; 32];
        let pq_statement_digest = pq_statement_digest_v1(
            1000,
            1000,
            statement_digest,
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            PostQuantumCheckpointAnchorModeV1::Plonky3EpochProof,
            PostQuantumCheckpointEnforcementStageV1::PqAnchorWriter,
        );
        let pq_anchor_root = pq_anchor_root_v1(
            1000,
            1000,
            statement_digest,
            pq_statement_digest,
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            root(9),
            PostQuantumCheckpointAnchorModeV1::Plonky3EpochProof,
            PostQuantumCheckpointEnforcementStageV1::PqAnchorWriter,
        );
        value["statement_digest"] = serde_json::json!(statement_digest);
        value["pq_statement_digest"] = serde_json::json!(pq_statement_digest);
        value["pq_anchor_root"] = serde_json::json!(pq_anchor_root);

        let err =
            decode_pq_anchor_json_checked(&serde_json::to_vec(&value).expect("pq anchor bytes"))
                .expect_err("missing required pq artifact must reject");

        assert!(
            matches!(err, CheckpointError::Backend(msg) if msg.contains("statement_digest_missing"))
        );
    }
}
