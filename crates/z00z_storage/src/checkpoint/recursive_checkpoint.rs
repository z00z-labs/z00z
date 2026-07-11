use std::{collections::BTreeSet, fmt, path::PathBuf};

use z00z_crypto::{
    expert::{encoding::to_hex, hash_domain},
    frame_bytes, frame_str,
    hash_zk::hash_zk,
};
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::CheckpointError;

use super::{
    artifact_stmt::{
        CheckpointTransitionStatementCoreV1, CheckpointTransitionStatementFinalV1,
        CheckpointTransitionStatementV1,
    },
    contract_config::{CheckpointContractConfigV1, CheckpointResolvedPaths},
    ids::CheckpointId,
    link::CheckpointLink,
};

hash_domain!(
    StorRecursiveCheckpointDom,
    "z00z.storage.checkpoint.recursive",
    1
);

const RECURSIVE_VER: u8 = 1;
const PUBLIC_INPUT_LABEL: &str = "recursive_checkpoint_public_input_v1";
const PROOF_LABEL: &str = "recursive_checkpoint_proof_v1";
const PROOF_BYTES_LABEL: &str = "recursive_checkpoint_proof_bytes_v1";
const MEASUREMENT_LABEL: &str = "recursive_checkpoint_measurement_v1";
const SIDECAR_LABEL: &str = "recursive_checkpoint_sidecar_v1";
const STEP_LABEL: &str = "recursive_checkpoint_chain_step_v1";
const CHAIN_LABEL: &str = "recursive_checkpoint_chain_evidence_v1";
const CHAIN_ROOT_LABEL: &str = "recursive_checkpoint_chain_root_v1";
const MEASUREMENTS_ROOT_LABEL: &str = "recursive_checkpoint_measurements_root_v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct RecursiveCheckpointVersion(u8);

impl RecursiveCheckpointVersion {
    pub const CURRENT: Self = Self(RECURSIVE_VER);

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
pub enum RecursiveCheckpointModeV1 {
    FastClassicalCompressed,
    PqEpochFinality,
}

impl RecursiveCheckpointModeV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FastClassicalCompressed => "fast_classical_compressed",
            Self::PqEpochFinality => "pq_epoch_finality",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecursiveCheckpointVerdictV1 {
    Accepted,
    Rejected,
}

impl RecursiveCheckpointVerdictV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecursiveCheckpointProofFamilyV1 {
    Nova,
    Stark,
}

impl RecursiveCheckpointProofFamilyV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nova => "nova",
            Self::Stark => "stark",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecursiveCheckpointRejectReasonV1 {
    UnsupportedVersion,
    UnknownField,
    StatementDigestMismatch,
    PublicInputDigestMismatch,
    PriorOutputMismatch,
    OutputRootMismatch,
    BackendUnsupported,
    BackendClaimUnsupported,
    ProofBytesEmpty,
    ProofBytesTooLarge,
    ProofSizeBudgetExceeded,
    NovaPqAuthorityUnsupported,
    SidecarAuthoritative,
    MeasurementMissing,
    ChainTooShort,
    ChainTooLong,
    StepSkipped,
    StepRepeated,
    StepReordered,
    CanonicalAdmissionAttempt,
    VerifiedCodecMissing,
    MixedEra,
    PqCadenceDisabled,
    PqCadenceInvalid,
}

impl RecursiveCheckpointRejectReasonV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedVersion => "unsupported_version",
            Self::UnknownField => "unknown_field",
            Self::StatementDigestMismatch => "statement_digest_mismatch",
            Self::PublicInputDigestMismatch => "public_input_digest_mismatch",
            Self::PriorOutputMismatch => "prior_output_mismatch",
            Self::OutputRootMismatch => "output_root_mismatch",
            Self::BackendUnsupported => "backend_unsupported",
            Self::BackendClaimUnsupported => "backend_claim_unsupported",
            Self::ProofBytesEmpty => "proof_bytes_empty",
            Self::ProofBytesTooLarge => "proof_bytes_too_large",
            Self::ProofSizeBudgetExceeded => "proof_size_budget_exceeded",
            Self::NovaPqAuthorityUnsupported => "nova_pq_authority_unsupported",
            Self::SidecarAuthoritative => "sidecar_authoritative",
            Self::MeasurementMissing => "measurement_missing",
            Self::ChainTooShort => "chain_too_short",
            Self::ChainTooLong => "chain_too_long",
            Self::StepSkipped => "step_skipped",
            Self::StepRepeated => "step_repeated",
            Self::StepReordered => "step_reordered",
            Self::CanonicalAdmissionAttempt => "canonical_admission_attempt",
            Self::VerifiedCodecMissing => "verified_codec_missing",
            Self::MixedEra => "mixed_era",
            Self::PqCadenceDisabled => "pq_cadence_disabled",
            Self::PqCadenceInvalid => "pq_cadence_invalid",
        }
    }
}

impl fmt::Display for RecursiveCheckpointRejectReasonV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<RecursiveCheckpointRejectReasonV1> for CheckpointError {
    fn from(reason: RecursiveCheckpointRejectReasonV1) -> Self {
        match reason {
            RecursiveCheckpointRejectReasonV1::UnsupportedVersion => Self::VersionMix,
            other => Self::Backend(format!("recursive checkpoint reject: {other}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveCheckpointPublicInputV1 {
    version: RecursiveCheckpointVersion,
    mode: RecursiveCheckpointModeV1,
    backend_label: String,
    statement_digest: [u8; 32],
    statement_core_digest: [u8; 32],
    height: u64,
    chain_index: u32,
    chain_length: u32,
    epoch_index: u64,
    epoch_start_height: u64,
    epoch_end_height: u64,
    prev_root: [u8; 32],
    output_root: [u8; 32],
    prior_output_root: [u8; 32],
    delta_root: [u8; 32],
    witness_root: [u8; 32],
    checkpoint_link_digest: [u8; 32],
    verifier_params_digest: [u8; 32],
}

impl RecursiveCheckpointPublicInputV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mode: RecursiveCheckpointModeV1,
        backend_label: impl Into<String>,
        statement_digest: [u8; 32],
        statement_core_digest: [u8; 32],
        height: u64,
        chain_index: u32,
        chain_length: u32,
        epoch_index: u64,
        epoch_start_height: u64,
        epoch_end_height: u64,
        prev_root: [u8; 32],
        output_root: [u8; 32],
        prior_output_root: [u8; 32],
        delta_root: [u8; 32],
        witness_root: [u8; 32],
        checkpoint_link_digest: [u8; 32],
        verifier_params_digest: [u8; 32],
    ) -> Result<Self, RecursiveCheckpointRejectReasonV1> {
        let backend_label = backend_label.into();
        if backend_label.trim().is_empty()
            || height == 0
            || chain_length == 0
            || chain_index >= chain_length
            || epoch_end_height < epoch_start_height
            || !epoch_contains(epoch_start_height, epoch_end_height, height)
            || is_zero_root(&statement_digest)
            || is_zero_root(&statement_core_digest)
            || is_zero_root(&prev_root)
            || is_zero_root(&output_root)
            || is_zero_root(&prior_output_root)
            || is_zero_root(&delta_root)
            || is_zero_root(&witness_root)
            || is_zero_root(&checkpoint_link_digest)
            || is_zero_root(&verifier_params_digest)
        {
            return Err(RecursiveCheckpointRejectReasonV1::StatementDigestMismatch);
        }

        Ok(Self {
            version: RecursiveCheckpointVersion::CURRENT,
            mode,
            backend_label,
            statement_digest,
            statement_core_digest,
            height,
            chain_index,
            chain_length,
            epoch_index,
            epoch_start_height,
            epoch_end_height,
            prev_root,
            output_root,
            prior_output_root,
            delta_root,
            witness_root,
            checkpoint_link_digest,
            verifier_params_digest,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_statement(
        statement: &CheckpointTransitionStatementV1,
        core: &CheckpointTransitionStatementCoreV1,
        final_bind: &CheckpointTransitionStatementFinalV1,
        link: &CheckpointLink,
        mode: RecursiveCheckpointModeV1,
        backend_label: impl Into<String>,
        chain_index: u32,
        chain_length: u32,
        epoch_len: u64,
        verifier_params_digest: [u8; 32],
    ) -> Result<Self, RecursiveCheckpointRejectReasonV1> {
        if epoch_len == 0 {
            return Err(RecursiveCheckpointRejectReasonV1::PqCadenceInvalid);
        }
        let height = statement.height();
        let epoch_index = (height - 1) / epoch_len;
        let epoch_start_height = epoch_index
            .checked_mul(epoch_len)
            .and_then(|value| value.checked_add(1))
            .ok_or(RecursiveCheckpointRejectReasonV1::PqCadenceInvalid)?;
        let epoch_end_height = epoch_start_height
            .checked_add(epoch_len - 1)
            .ok_or(RecursiveCheckpointRejectReasonV1::PqCadenceInvalid)?;
        Self::new(
            mode,
            backend_label,
            statement.final_statement_digest_v1(core, final_bind),
            statement.statement_core_digest_v1(core),
            height,
            chain_index,
            chain_length,
            epoch_index,
            epoch_start_height,
            epoch_end_height,
            statement.prev_root().into_bytes(),
            statement.new_root().into_bytes(),
            statement.prev_root().into_bytes(),
            core.delta_root(),
            core.witness_root(),
            link.link_bind(),
            verifier_params_digest,
        )
    }

    #[must_use]
    pub const fn version(&self) -> RecursiveCheckpointVersion {
        self.version
    }

    #[must_use]
    pub const fn mode(&self) -> RecursiveCheckpointModeV1 {
        self.mode
    }

    #[must_use]
    pub fn backend_label(&self) -> &str {
        &self.backend_label
    }

    #[must_use]
    pub const fn statement_digest(&self) -> [u8; 32] {
        self.statement_digest
    }

    #[must_use]
    pub const fn statement_core_digest(&self) -> [u8; 32] {
        self.statement_core_digest
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub const fn chain_index(&self) -> u32 {
        self.chain_index
    }

    #[must_use]
    pub const fn chain_length(&self) -> u32 {
        self.chain_length
    }

    #[must_use]
    pub const fn epoch_index(&self) -> u64 {
        self.epoch_index
    }

    #[must_use]
    pub const fn epoch_start_height(&self) -> u64 {
        self.epoch_start_height
    }

    #[must_use]
    pub const fn epoch_end_height(&self) -> u64 {
        self.epoch_end_height
    }

    #[must_use]
    pub const fn prev_root(&self) -> [u8; 32] {
        self.prev_root
    }

    #[must_use]
    pub const fn output_root(&self) -> [u8; 32] {
        self.output_root
    }

    #[must_use]
    pub const fn prior_output_root(&self) -> [u8; 32] {
        self.prior_output_root
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
    pub const fn checkpoint_link_digest(&self) -> [u8; 32] {
        self.checkpoint_link_digest
    }

    #[must_use]
    pub const fn verifier_params_digest(&self) -> [u8; 32] {
        self.verifier_params_digest
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_framed_field(&mut bytes, "version", &[self.version.as_u8()]);
        push_framed_field(&mut bytes, "mode", self.mode.as_str().as_bytes());
        push_framed_field(&mut bytes, "backend_label", self.backend_label.as_bytes());
        push_framed_field(&mut bytes, "statement_digest", &self.statement_digest);
        push_framed_field(
            &mut bytes,
            "statement_core_digest",
            &self.statement_core_digest,
        );
        push_framed_field(&mut bytes, "height", &self.height.to_le_bytes());
        push_framed_field(&mut bytes, "chain_index", &self.chain_index.to_le_bytes());
        push_framed_field(&mut bytes, "chain_length", &self.chain_length.to_le_bytes());
        push_framed_field(&mut bytes, "epoch_index", &self.epoch_index.to_le_bytes());
        push_framed_field(
            &mut bytes,
            "epoch_start_height",
            &self.epoch_start_height.to_le_bytes(),
        );
        push_framed_field(
            &mut bytes,
            "epoch_end_height",
            &self.epoch_end_height.to_le_bytes(),
        );
        push_framed_field(&mut bytes, "prev_root", &self.prev_root);
        push_framed_field(&mut bytes, "output_root", &self.output_root);
        push_framed_field(&mut bytes, "prior_output_root", &self.prior_output_root);
        push_framed_field(&mut bytes, "delta_root", &self.delta_root);
        push_framed_field(&mut bytes, "witness_root", &self.witness_root);
        push_framed_field(
            &mut bytes,
            "checkpoint_link_digest",
            &self.checkpoint_link_digest,
        );
        push_framed_field(
            &mut bytes,
            "verifier_params_digest",
            &self.verifier_params_digest,
        );
        bytes
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        let bytes = self.canonical_bytes();
        hash_zk::<StorRecursiveCheckpointDom>(PUBLIC_INPUT_LABEL, &[bytes.as_slice()])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveCheckpointProofV1 {
    version: RecursiveCheckpointVersion,
    mode: RecursiveCheckpointModeV1,
    backend_label: String,
    statement_digest: [u8; 32],
    public_input_digest: [u8; 32],
    prior_output_root: [u8; 32],
    output_root: [u8; 32],
    verifier_params_digest: [u8; 32],
    proof_bytes_digest: [u8; 32],
    proof_bytes: Vec<u8>,
}

impl RecursiveCheckpointProofV1 {
    pub fn new(
        mode: RecursiveCheckpointModeV1,
        backend_label: impl Into<String>,
        public_input: &RecursiveCheckpointPublicInputV1,
        proof_bytes: Vec<u8>,
    ) -> Result<Self, RecursiveCheckpointRejectReasonV1> {
        let backend_label = backend_label.into();
        if backend_label.trim().is_empty() {
            return Err(RecursiveCheckpointRejectReasonV1::BackendUnsupported);
        }
        if proof_bytes.is_empty() {
            return Err(RecursiveCheckpointRejectReasonV1::ProofBytesEmpty);
        }
        let proof_bytes_digest = proof_bytes_digest(&proof_bytes);
        Ok(Self {
            version: RecursiveCheckpointVersion::CURRENT,
            mode,
            backend_label,
            statement_digest: public_input.statement_digest(),
            public_input_digest: public_input.digest(),
            prior_output_root: public_input.prior_output_root(),
            output_root: public_input.output_root(),
            verifier_params_digest: public_input.verifier_params_digest(),
            proof_bytes_digest,
            proof_bytes,
        })
    }

    #[must_use]
    pub const fn version(&self) -> RecursiveCheckpointVersion {
        self.version
    }

    #[must_use]
    pub const fn mode(&self) -> RecursiveCheckpointModeV1 {
        self.mode
    }

    #[must_use]
    pub fn backend_label(&self) -> &str {
        &self.backend_label
    }

    #[must_use]
    pub const fn statement_digest(&self) -> [u8; 32] {
        self.statement_digest
    }

    #[must_use]
    pub const fn public_input_digest(&self) -> [u8; 32] {
        self.public_input_digest
    }

    #[must_use]
    pub const fn prior_output_root(&self) -> [u8; 32] {
        self.prior_output_root
    }

    #[must_use]
    pub const fn output_root(&self) -> [u8; 32] {
        self.output_root
    }

    #[must_use]
    pub const fn verifier_params_digest(&self) -> [u8; 32] {
        self.verifier_params_digest
    }

    #[must_use]
    pub const fn proof_bytes_digest(&self) -> [u8; 32] {
        self.proof_bytes_digest
    }

    #[must_use]
    pub fn proof_bytes(&self) -> &[u8] {
        &self.proof_bytes
    }

    #[must_use]
    pub fn with_statement_digest(mut self, statement_digest: [u8; 32]) -> Self {
        self.statement_digest = statement_digest;
        self
    }

    #[must_use]
    pub fn with_public_input_digest(mut self, public_input_digest: [u8; 32]) -> Self {
        self.public_input_digest = public_input_digest;
        self
    }

    #[must_use]
    pub fn with_prior_output_root(mut self, prior_output_root: [u8; 32]) -> Self {
        self.prior_output_root = prior_output_root;
        self
    }

    #[must_use]
    pub fn with_output_root(mut self, output_root: [u8; 32]) -> Self {
        self.output_root = output_root;
        self
    }

    #[must_use]
    pub fn with_backend_label(mut self, backend_label: impl Into<String>) -> Self {
        self.backend_label = backend_label.into();
        self
    }

    #[must_use]
    pub fn with_mode(mut self, mode: RecursiveCheckpointModeV1) -> Self {
        self.mode = mode;
        self
    }

    #[must_use]
    pub fn with_proof_bytes(mut self, proof_bytes: Vec<u8>) -> Self {
        self.proof_bytes_digest = proof_bytes_digest(&proof_bytes);
        self.proof_bytes = proof_bytes;
        self
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_framed_field(&mut bytes, "version", &[self.version.as_u8()]);
        push_framed_field(&mut bytes, "mode", self.mode.as_str().as_bytes());
        push_framed_field(&mut bytes, "backend_label", self.backend_label.as_bytes());
        push_framed_field(&mut bytes, "statement_digest", &self.statement_digest);
        push_framed_field(&mut bytes, "public_input_digest", &self.public_input_digest);
        push_framed_field(&mut bytes, "prior_output_root", &self.prior_output_root);
        push_framed_field(&mut bytes, "output_root", &self.output_root);
        push_framed_field(
            &mut bytes,
            "verifier_params_digest",
            &self.verifier_params_digest,
        );
        push_framed_field(&mut bytes, "proof_bytes_digest", &self.proof_bytes_digest);
        push_framed_field(&mut bytes, "proof_bytes", &self.proof_bytes);
        bytes
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        let bytes = self.canonical_bytes();
        hash_zk::<StorRecursiveCheckpointDom>(PROOF_LABEL, &[bytes.as_slice()])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveCheckpointMeasurementV1 {
    version: RecursiveCheckpointVersion,
    backend_label: String,
    mode: RecursiveCheckpointModeV1,
    chain_length: u32,
    epoch_length: u64,
    aggregation_nodes: u32,
    proof_family: RecursiveCheckpointProofFamilyV1,
    security_bits: u16,
    proof_bytes: u64,
    witness_bytes: u64,
    prover_ms: u64,
    verifier_ms: u64,
    peak_memory_bytes: u64,
    statement_bytes: u64,
    public_input_bytes: u64,
}

impl RecursiveCheckpointMeasurementV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        backend_label: impl Into<String>,
        mode: RecursiveCheckpointModeV1,
        chain_length: u32,
        epoch_length: u64,
        aggregation_nodes: u32,
        proof_family: RecursiveCheckpointProofFamilyV1,
        security_bits: u16,
        proof_bytes: u64,
        witness_bytes: u64,
        prover_ms: u64,
        verifier_ms: u64,
        peak_memory_bytes: u64,
        statement_bytes: u64,
        public_input_bytes: u64,
    ) -> Result<Self, RecursiveCheckpointRejectReasonV1> {
        let backend_label = backend_label.into();
        if backend_label.trim().is_empty()
            || chain_length == 0
            || epoch_length == 0
            || aggregation_nodes == 0
            || proof_bytes == 0
            || witness_bytes == 0
            || statement_bytes == 0
            || public_input_bytes == 0
        {
            return Err(RecursiveCheckpointRejectReasonV1::MeasurementMissing);
        }
        Ok(Self {
            version: RecursiveCheckpointVersion::CURRENT,
            backend_label,
            mode,
            chain_length,
            epoch_length,
            aggregation_nodes,
            proof_family,
            security_bits,
            proof_bytes,
            witness_bytes,
            prover_ms,
            verifier_ms,
            peak_memory_bytes,
            statement_bytes,
            public_input_bytes,
        })
    }

    #[must_use]
    pub const fn version(&self) -> RecursiveCheckpointVersion {
        self.version
    }

    #[must_use]
    pub fn backend_label(&self) -> &str {
        &self.backend_label
    }

    #[must_use]
    pub const fn mode(&self) -> RecursiveCheckpointModeV1 {
        self.mode
    }

    #[must_use]
    pub const fn chain_length(&self) -> u32 {
        self.chain_length
    }

    #[must_use]
    pub const fn epoch_length(&self) -> u64 {
        self.epoch_length
    }

    #[must_use]
    pub const fn aggregation_nodes(&self) -> u32 {
        self.aggregation_nodes
    }

    #[must_use]
    pub const fn proof_family(&self) -> RecursiveCheckpointProofFamilyV1 {
        self.proof_family
    }

    #[must_use]
    pub const fn security_bits(&self) -> u16 {
        self.security_bits
    }

    #[must_use]
    pub const fn proof_bytes(&self) -> u64 {
        self.proof_bytes
    }

    #[must_use]
    pub const fn witness_bytes(&self) -> u64 {
        self.witness_bytes
    }

    #[must_use]
    pub const fn prover_ms(&self) -> u64 {
        self.prover_ms
    }

    #[must_use]
    pub const fn verifier_ms(&self) -> u64 {
        self.verifier_ms
    }

    #[must_use]
    pub const fn peak_memory_bytes(&self) -> u64 {
        self.peak_memory_bytes
    }

    #[must_use]
    pub const fn statement_bytes(&self) -> u64 {
        self.statement_bytes
    }

    #[must_use]
    pub const fn public_input_bytes(&self) -> u64 {
        self.public_input_bytes
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_framed_field(&mut bytes, "version", &[self.version.as_u8()]);
        push_framed_field(&mut bytes, "backend_label", self.backend_label.as_bytes());
        push_framed_field(&mut bytes, "mode", self.mode.as_str().as_bytes());
        push_framed_field(&mut bytes, "chain_length", &self.chain_length.to_le_bytes());
        push_framed_field(&mut bytes, "epoch_length", &self.epoch_length.to_le_bytes());
        push_framed_field(
            &mut bytes,
            "aggregation_nodes",
            &self.aggregation_nodes.to_le_bytes(),
        );
        push_framed_field(
            &mut bytes,
            "proof_family",
            self.proof_family.as_str().as_bytes(),
        );
        push_framed_field(
            &mut bytes,
            "security_bits",
            &self.security_bits.to_le_bytes(),
        );
        push_framed_field(&mut bytes, "proof_bytes", &self.proof_bytes.to_le_bytes());
        push_framed_field(
            &mut bytes,
            "witness_bytes",
            &self.witness_bytes.to_le_bytes(),
        );
        push_framed_field(&mut bytes, "prover_ms", &self.prover_ms.to_le_bytes());
        push_framed_field(&mut bytes, "verifier_ms", &self.verifier_ms.to_le_bytes());
        push_framed_field(
            &mut bytes,
            "peak_memory_bytes",
            &self.peak_memory_bytes.to_le_bytes(),
        );
        push_framed_field(
            &mut bytes,
            "statement_bytes",
            &self.statement_bytes.to_le_bytes(),
        );
        push_framed_field(
            &mut bytes,
            "public_input_bytes",
            &self.public_input_bytes.to_le_bytes(),
        );
        bytes
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        let bytes = self.canonical_bytes();
        hash_zk::<StorRecursiveCheckpointDom>(MEASUREMENT_LABEL, &[bytes.as_slice()])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveCheckpointSidecarV1 {
    version: RecursiveCheckpointVersion,
    mode: RecursiveCheckpointModeV1,
    statement_digest: [u8; 32],
    public_input_digest: [u8; 32],
    public_input: RecursiveCheckpointPublicInputV1,
    checkpoint_id_hint: Option<CheckpointId>,
    proof: RecursiveCheckpointProofV1,
    verifier_verdict: RecursiveCheckpointVerdictV1,
    reject_reason: Option<RecursiveCheckpointRejectReasonV1>,
    chain_index: u32,
    chain_length: u32,
    measurements: Option<RecursiveCheckpointMeasurementV1>,
}

impl RecursiveCheckpointSidecarV1 {
    pub fn accepted(
        public_input: RecursiveCheckpointPublicInputV1,
        checkpoint_id_hint: Option<CheckpointId>,
        proof: RecursiveCheckpointProofV1,
        measurements: RecursiveCheckpointMeasurementV1,
    ) -> Result<Self, RecursiveCheckpointRejectReasonV1> {
        Ok(Self {
            version: RecursiveCheckpointVersion::CURRENT,
            mode: public_input.mode(),
            statement_digest: public_input.statement_digest(),
            public_input_digest: public_input.digest(),
            chain_index: public_input.chain_index(),
            chain_length: public_input.chain_length(),
            public_input,
            checkpoint_id_hint,
            proof,
            verifier_verdict: RecursiveCheckpointVerdictV1::Accepted,
            reject_reason: None,
            measurements: Some(measurements),
        })
    }

    #[must_use]
    pub const fn version(&self) -> RecursiveCheckpointVersion {
        self.version
    }

    #[must_use]
    pub const fn mode(&self) -> RecursiveCheckpointModeV1 {
        self.mode
    }

    #[must_use]
    pub const fn statement_digest(&self) -> [u8; 32] {
        self.statement_digest
    }

    #[must_use]
    pub const fn public_input_digest(&self) -> [u8; 32] {
        self.public_input_digest
    }

    #[must_use]
    pub fn public_input(&self) -> &RecursiveCheckpointPublicInputV1 {
        &self.public_input
    }

    #[must_use]
    pub const fn checkpoint_id_hint(&self) -> Option<CheckpointId> {
        self.checkpoint_id_hint
    }

    #[must_use]
    pub fn proof(&self) -> &RecursiveCheckpointProofV1 {
        &self.proof
    }

    #[must_use]
    pub const fn verifier_verdict(&self) -> RecursiveCheckpointVerdictV1 {
        self.verifier_verdict
    }

    #[must_use]
    pub const fn reject_reason(&self) -> Option<RecursiveCheckpointRejectReasonV1> {
        self.reject_reason
    }

    #[must_use]
    pub const fn chain_index(&self) -> u32 {
        self.chain_index
    }

    #[must_use]
    pub const fn chain_length(&self) -> u32 {
        self.chain_length
    }

    #[must_use]
    pub fn measurements(&self) -> Option<&RecursiveCheckpointMeasurementV1> {
        self.measurements.as_ref()
    }

    #[must_use]
    pub fn with_statement_digest(mut self, statement_digest: [u8; 32]) -> Self {
        self.statement_digest = statement_digest;
        self
    }

    #[must_use]
    pub fn with_public_input_digest(mut self, public_input_digest: [u8; 32]) -> Self {
        self.public_input_digest = public_input_digest;
        self
    }

    #[must_use]
    pub fn with_measurements(
        mut self,
        measurements: Option<RecursiveCheckpointMeasurementV1>,
    ) -> Self {
        self.measurements = measurements;
        self
    }

    #[must_use]
    pub fn with_proof(mut self, proof: RecursiveCheckpointProofV1) -> Self {
        self.proof = proof;
        self
    }

    #[must_use]
    pub fn with_mode(mut self, mode: RecursiveCheckpointModeV1) -> Self {
        self.mode = mode;
        self
    }

    #[must_use]
    pub fn with_reject_reason(
        mut self,
        reject_reason: Option<RecursiveCheckpointRejectReasonV1>,
    ) -> Self {
        self.reject_reason = reject_reason;
        self.verifier_verdict = if reject_reason.is_some() {
            RecursiveCheckpointVerdictV1::Rejected
        } else {
            RecursiveCheckpointVerdictV1::Accepted
        };
        self
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_framed_field(&mut bytes, "version", &[self.version.as_u8()]);
        push_framed_field(&mut bytes, "mode", self.mode.as_str().as_bytes());
        push_framed_field(&mut bytes, "statement_digest", &self.statement_digest);
        push_framed_field(&mut bytes, "public_input_digest", &self.public_input_digest);
        push_framed_field(
            &mut bytes,
            "public_input_digest_self",
            &self.public_input.digest(),
        );
        push_framed_field(
            &mut bytes,
            "checkpoint_id_hint",
            &encode_opt_checkpoint_id(self.checkpoint_id_hint),
        );
        push_framed_field(&mut bytes, "proof_digest", &self.proof.digest());
        push_framed_field(
            &mut bytes,
            "verifier_verdict",
            self.verifier_verdict.as_str().as_bytes(),
        );
        push_framed_field(
            &mut bytes,
            "reject_reason",
            &encode_opt_reason(self.reject_reason),
        );
        push_framed_field(&mut bytes, "chain_index", &self.chain_index.to_le_bytes());
        push_framed_field(&mut bytes, "chain_length", &self.chain_length.to_le_bytes());
        push_framed_field(
            &mut bytes,
            "measurement_digest",
            &encode_opt_digest(
                self.measurements
                    .as_ref()
                    .map(RecursiveCheckpointMeasurementV1::digest),
            ),
        );
        bytes
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        let bytes = self.canonical_bytes();
        hash_zk::<StorRecursiveCheckpointDom>(SIDECAR_LABEL, &[bytes.as_slice()])
    }

    #[must_use]
    pub fn storage_path(&self, paths: &CheckpointResolvedPaths) -> PathBuf {
        let stem = match self.checkpoint_id_hint {
            Some(checkpoint_id) => to_hex(checkpoint_id.as_bytes()),
            None => to_hex(&self.digest()),
        };
        paths.recursive_sidecars.join(format!("{stem}.sidecar.bin"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveCheckpointChainStepV1 {
    height: u64,
    statement_digest: [u8; 32],
    prev_root: [u8; 32],
    prior_output_root: [u8; 32],
    output_root: [u8; 32],
    chain_index: u32,
    sidecar_digest: [u8; 32],
}

impl RecursiveCheckpointChainStepV1 {
    #[must_use]
    pub fn from_sidecar(sidecar: &RecursiveCheckpointSidecarV1) -> Self {
        let public_input = sidecar.public_input();
        Self {
            height: public_input.height(),
            statement_digest: public_input.statement_digest(),
            prev_root: public_input.prev_root(),
            prior_output_root: public_input.prior_output_root(),
            output_root: public_input.output_root(),
            chain_index: public_input.chain_index(),
            sidecar_digest: sidecar.digest(),
        }
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub const fn statement_digest(&self) -> [u8; 32] {
        self.statement_digest
    }

    #[must_use]
    pub const fn prev_root(&self) -> [u8; 32] {
        self.prev_root
    }

    #[must_use]
    pub const fn prior_output_root(&self) -> [u8; 32] {
        self.prior_output_root
    }

    #[must_use]
    pub const fn output_root(&self) -> [u8; 32] {
        self.output_root
    }

    #[must_use]
    pub const fn chain_index(&self) -> u32 {
        self.chain_index
    }

    #[must_use]
    pub const fn sidecar_digest(&self) -> [u8; 32] {
        self.sidecar_digest
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_framed_field(&mut bytes, "height", &self.height.to_le_bytes());
        push_framed_field(&mut bytes, "statement_digest", &self.statement_digest);
        push_framed_field(&mut bytes, "prev_root", &self.prev_root);
        push_framed_field(&mut bytes, "prior_output_root", &self.prior_output_root);
        push_framed_field(&mut bytes, "output_root", &self.output_root);
        push_framed_field(&mut bytes, "chain_index", &self.chain_index.to_le_bytes());
        push_framed_field(&mut bytes, "sidecar_digest", &self.sidecar_digest);
        bytes
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        let bytes = self.canonical_bytes();
        hash_zk::<StorRecursiveCheckpointDom>(STEP_LABEL, &[bytes.as_slice()])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveCheckpointChainEvidenceV1 {
    version: RecursiveCheckpointVersion,
    mode: RecursiveCheckpointModeV1,
    backend_label: String,
    chain_length: u32,
    first_statement_digest: [u8; 32],
    last_statement_digest: [u8; 32],
    first_prev_root: [u8; 32],
    last_output_root: [u8; 32],
    nova_chain_root: [u8; 32],
    step_digests: Vec<[u8; 32]>,
    measurements_root: [u8; 32],
    steps: Vec<RecursiveCheckpointChainStepV1>,
}

impl RecursiveCheckpointChainEvidenceV1 {
    pub fn from_sidecars(
        sidecars: &[RecursiveCheckpointSidecarV1],
    ) -> Result<Self, RecursiveCheckpointRejectReasonV1> {
        if sidecars.len() < 3 {
            return Err(RecursiveCheckpointRejectReasonV1::ChainTooShort);
        }
        if sidecars.len() > 5 {
            return Err(RecursiveCheckpointRejectReasonV1::ChainTooLong);
        }
        let first = sidecars
            .first()
            .ok_or(RecursiveCheckpointRejectReasonV1::ChainTooShort)?;
        let mode = first.mode();
        let backend_label = first.public_input().backend_label().to_string();
        let steps: Vec<_> = sidecars
            .iter()
            .map(RecursiveCheckpointChainStepV1::from_sidecar)
            .collect();
        let step_digests: Vec<_> = steps
            .iter()
            .map(RecursiveCheckpointChainStepV1::digest)
            .collect();
        let measurement_digests = sidecars
            .iter()
            .map(|sidecar| {
                sidecar
                    .measurements()
                    .map(RecursiveCheckpointMeasurementV1::digest)
                    .ok_or(RecursiveCheckpointRejectReasonV1::MeasurementMissing)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let measurements_root = digest_list(MEASUREMENTS_ROOT_LABEL, &measurement_digests);
        let nova_chain_root = digest_list(CHAIN_ROOT_LABEL, &step_digests);
        let last = steps
            .last()
            .ok_or(RecursiveCheckpointRejectReasonV1::ChainTooShort)?;
        Ok(Self {
            version: RecursiveCheckpointVersion::CURRENT,
            mode,
            backend_label,
            chain_length: u32::try_from(steps.len())
                .map_err(|_| RecursiveCheckpointRejectReasonV1::ChainTooLong)?,
            first_statement_digest: steps[0].statement_digest(),
            last_statement_digest: last.statement_digest(),
            first_prev_root: steps[0].prev_root(),
            last_output_root: last.output_root(),
            nova_chain_root,
            step_digests,
            measurements_root,
            steps,
        })
    }

    #[must_use]
    pub const fn version(&self) -> RecursiveCheckpointVersion {
        self.version
    }

    #[must_use]
    pub const fn mode(&self) -> RecursiveCheckpointModeV1 {
        self.mode
    }

    #[must_use]
    pub fn backend_label(&self) -> &str {
        &self.backend_label
    }

    #[must_use]
    pub const fn chain_length(&self) -> u32 {
        self.chain_length
    }

    #[must_use]
    pub const fn first_statement_digest(&self) -> [u8; 32] {
        self.first_statement_digest
    }

    #[must_use]
    pub const fn last_statement_digest(&self) -> [u8; 32] {
        self.last_statement_digest
    }

    #[must_use]
    pub const fn first_prev_root(&self) -> [u8; 32] {
        self.first_prev_root
    }

    #[must_use]
    pub const fn last_output_root(&self) -> [u8; 32] {
        self.last_output_root
    }

    #[must_use]
    pub const fn nova_chain_root(&self) -> [u8; 32] {
        self.nova_chain_root
    }

    #[must_use]
    pub fn step_digests(&self) -> &[[u8; 32]] {
        &self.step_digests
    }

    #[must_use]
    pub const fn measurements_root(&self) -> [u8; 32] {
        self.measurements_root
    }

    #[must_use]
    pub fn steps(&self) -> &[RecursiveCheckpointChainStepV1] {
        &self.steps
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_framed_field(&mut bytes, "version", &[self.version.as_u8()]);
        push_framed_field(&mut bytes, "mode", self.mode.as_str().as_bytes());
        push_framed_field(&mut bytes, "backend_label", self.backend_label.as_bytes());
        push_framed_field(&mut bytes, "chain_length", &self.chain_length.to_le_bytes());
        push_framed_field(
            &mut bytes,
            "first_statement_digest",
            &self.first_statement_digest,
        );
        push_framed_field(
            &mut bytes,
            "last_statement_digest",
            &self.last_statement_digest,
        );
        push_framed_field(&mut bytes, "first_prev_root", &self.first_prev_root);
        push_framed_field(&mut bytes, "last_output_root", &self.last_output_root);
        push_framed_field(&mut bytes, "nova_chain_root", &self.nova_chain_root);
        push_framed_field(&mut bytes, "measurements_root", &self.measurements_root);
        push_framed_field(
            &mut bytes,
            "step_digests",
            &encode_digest_vec(&self.step_digests),
        );
        bytes
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        let bytes = self.canonical_bytes();
        hash_zk::<StorRecursiveCheckpointDom>(CHAIN_LABEL, &[bytes.as_slice()])
    }
}

#[derive(Clone, Debug)]
pub struct RecursiveCheckpointVerifierV1 {
    min_chain_steps: u32,
    max_chain_steps: u32,
    epoch_len: u64,
    max_recursive_proof_bytes: usize,
    max_recursive_sidecar_bytes: usize,
    max_nova_block_proof_bytes: usize,
    max_plonky3_epoch_proof_bytes: usize,
    recursive_non_authoritative: bool,
}

impl RecursiveCheckpointVerifierV1 {
    pub fn new(
        cfg: &CheckpointContractConfigV1,
    ) -> Result<Self, RecursiveCheckpointRejectReasonV1> {
        if !cfg.post_quantum.is_enabled {
            return Err(RecursiveCheckpointRejectReasonV1::PqCadenceDisabled);
        }
        if cfg.post_quantum.cadence_blocks == 0 {
            return Err(RecursiveCheckpointRejectReasonV1::PqCadenceInvalid);
        }
        if !cfg.gates.artifacts.has_recursive_sidecar_non_authoritative
            || cfg.authority_promotion.recursive_authority_allowed
            || cfg.branches.recursive.is_authoritative
        {
            return Err(RecursiveCheckpointRejectReasonV1::SidecarAuthoritative);
        }
        cfg.validate()
            .map_err(|_| RecursiveCheckpointRejectReasonV1::BackendClaimUnsupported)?;
        Ok(Self {
            min_chain_steps: cfg.branches.recursive.min_chain_steps,
            max_chain_steps: cfg.branches.recursive.target_chain_steps,
            epoch_len: cfg.post_quantum.cadence_blocks,
            max_recursive_proof_bytes: cfg.limits.max_recursive_proof_bytes,
            max_recursive_sidecar_bytes: cfg.limits.max_recursive_sidecar_bytes,
            max_nova_block_proof_bytes: cfg.limits.max_nova_block_proof_bytes,
            max_plonky3_epoch_proof_bytes: cfg.limits.max_plonky3_epoch_proof_bytes,
            recursive_non_authoritative: true,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build_public_input(
        &self,
        statement: &CheckpointTransitionStatementV1,
        core: &CheckpointTransitionStatementCoreV1,
        final_bind: &CheckpointTransitionStatementFinalV1,
        link: &CheckpointLink,
        mode: RecursiveCheckpointModeV1,
        backend_label: impl Into<String>,
        chain_index: u32,
        chain_length: u32,
        verifier_params_digest: [u8; 32],
    ) -> Result<RecursiveCheckpointPublicInputV1, RecursiveCheckpointRejectReasonV1> {
        let backend_label = backend_label.into();
        self.check_mode_backend(mode, &backend_label)?;
        self.check_chain_bounds(chain_index, chain_length)?;
        RecursiveCheckpointPublicInputV1::from_statement(
            statement,
            core,
            final_bind,
            link,
            mode,
            backend_label,
            chain_index,
            chain_length,
            self.epoch_len,
            verifier_params_digest,
        )
    }

    pub fn check_sidecar_shape(
        &self,
        sidecar: &RecursiveCheckpointSidecarV1,
    ) -> Result<(), RecursiveCheckpointRejectReasonV1> {
        if !self.recursive_non_authoritative {
            return Err(RecursiveCheckpointRejectReasonV1::SidecarAuthoritative);
        }
        check_recursive_ver(sidecar.version())?;
        check_recursive_ver(sidecar.public_input().version())?;
        check_recursive_ver(sidecar.proof().version())?;
        let measurements = sidecar
            .measurements()
            .ok_or(RecursiveCheckpointRejectReasonV1::MeasurementMissing)?;
        check_recursive_ver(measurements.version())?;
        self.check_mode_backend(sidecar.mode(), sidecar.public_input().backend_label())?;
        self.check_mode_backend(sidecar.proof().mode(), sidecar.proof().backend_label())?;
        self.check_chain_bounds(sidecar.chain_index(), sidecar.chain_length())?;
        if sidecar.mode() != sidecar.public_input().mode()
            || sidecar.mode() != sidecar.proof().mode()
            || sidecar.chain_index() != sidecar.public_input().chain_index()
            || sidecar.chain_length() != sidecar.public_input().chain_length()
        {
            return Err(RecursiveCheckpointRejectReasonV1::MixedEra);
        }
        if sidecar.statement_digest() != sidecar.public_input().statement_digest()
            || sidecar.statement_digest() != sidecar.proof().statement_digest()
        {
            return Err(RecursiveCheckpointRejectReasonV1::StatementDigestMismatch);
        }
        let public_input_digest = sidecar.public_input().digest();
        if sidecar.public_input_digest() != public_input_digest
            || sidecar.proof().public_input_digest() != public_input_digest
        {
            return Err(RecursiveCheckpointRejectReasonV1::PublicInputDigestMismatch);
        }
        if measurements.backend_label() != sidecar.public_input().backend_label()
            || measurements.mode() != sidecar.mode()
            || measurements.chain_length() != sidecar.chain_length()
        {
            return Err(RecursiveCheckpointRejectReasonV1::MixedEra);
        }
        self.check_proof_budget(sidecar.proof())?;
        let sidecar_size = BincodeCodec
            .serialize(sidecar)
            .map_err(|_| RecursiveCheckpointRejectReasonV1::ProofSizeBudgetExceeded)?
            .len();
        if sidecar_size > self.max_recursive_sidecar_bytes {
            return Err(RecursiveCheckpointRejectReasonV1::ProofSizeBudgetExceeded);
        }
        match sidecar.verifier_verdict() {
            RecursiveCheckpointVerdictV1::Accepted => {
                if sidecar.reject_reason().is_some() {
                    return Err(RecursiveCheckpointRejectReasonV1::MixedEra);
                }
            }
            RecursiveCheckpointVerdictV1::Rejected => {
                return Err(sidecar
                    .reject_reason()
                    .unwrap_or(RecursiveCheckpointRejectReasonV1::MixedEra));
            }
        }
        Ok(())
    }

    pub fn verify_step(
        &self,
        public_input: &RecursiveCheckpointPublicInputV1,
        proof: &RecursiveCheckpointProofV1,
    ) -> Result<(), RecursiveCheckpointRejectReasonV1> {
        check_recursive_ver(public_input.version())?;
        check_recursive_ver(proof.version())?;
        self.check_mode_backend(public_input.mode(), public_input.backend_label())?;
        self.check_mode_backend(proof.mode(), proof.backend_label())?;
        self.check_chain_bounds(public_input.chain_index(), public_input.chain_length())?;
        if public_input.mode() != proof.mode()
            || public_input.backend_label() != proof.backend_label()
        {
            return Err(RecursiveCheckpointRejectReasonV1::MixedEra);
        }
        if public_input.prior_output_root() != public_input.prev_root() {
            return Err(RecursiveCheckpointRejectReasonV1::PriorOutputMismatch);
        }
        if proof.statement_digest() != public_input.statement_digest() {
            return Err(RecursiveCheckpointRejectReasonV1::StatementDigestMismatch);
        }
        if proof.public_input_digest() != public_input.digest() {
            return Err(RecursiveCheckpointRejectReasonV1::PublicInputDigestMismatch);
        }
        if proof.prior_output_root() != public_input.prior_output_root() {
            return Err(RecursiveCheckpointRejectReasonV1::PriorOutputMismatch);
        }
        if proof.output_root() != public_input.output_root() {
            return Err(RecursiveCheckpointRejectReasonV1::OutputRootMismatch);
        }
        if proof.verifier_params_digest() != public_input.verifier_params_digest() {
            return Err(RecursiveCheckpointRejectReasonV1::PublicInputDigestMismatch);
        }
        self.check_proof_budget(proof)
    }

    pub fn verify_sidecar(
        &self,
        sidecar: &RecursiveCheckpointSidecarV1,
        statement: &CheckpointTransitionStatementV1,
        core: &CheckpointTransitionStatementCoreV1,
        final_bind: &CheckpointTransitionStatementFinalV1,
        link: &CheckpointLink,
    ) -> Result<(), RecursiveCheckpointRejectReasonV1> {
        self.check_sidecar_shape(sidecar)?;
        let public_input = sidecar.public_input();
        let expected = self.build_public_input(
            statement,
            core,
            final_bind,
            link,
            public_input.mode(),
            public_input.backend_label().to_string(),
            public_input.chain_index(),
            public_input.chain_length(),
            public_input.verifier_params_digest(),
        )?;
        if public_input.statement_core_digest() != expected.statement_core_digest()
            || public_input.statement_digest() != expected.statement_digest()
            || public_input.prev_root() != expected.prev_root()
            || public_input.output_root() != expected.output_root()
            || public_input.prior_output_root() != expected.prior_output_root()
            || public_input.delta_root() != expected.delta_root()
            || public_input.witness_root() != expected.witness_root()
            || public_input.checkpoint_link_digest() != expected.checkpoint_link_digest()
        {
            return Err(RecursiveCheckpointRejectReasonV1::StatementDigestMismatch);
        }
        self.verify_step(public_input, sidecar.proof())
    }

    pub fn build_chain(
        &self,
        sidecars: &[RecursiveCheckpointSidecarV1],
    ) -> Result<RecursiveCheckpointChainEvidenceV1, RecursiveCheckpointRejectReasonV1> {
        let first = sidecars
            .first()
            .ok_or(RecursiveCheckpointRejectReasonV1::ChainTooShort)?;
        for sidecar in sidecars {
            self.check_sidecar_shape(sidecar)?;
            if sidecar.mode() != first.mode()
                || sidecar.public_input().backend_label() != first.public_input().backend_label()
                || sidecar.chain_length() != first.chain_length()
            {
                return Err(RecursiveCheckpointRejectReasonV1::MixedEra);
            }
        }
        let chain = RecursiveCheckpointChainEvidenceV1::from_sidecars(sidecars)?;
        self.verify_chain(&chain)?;
        Ok(chain)
    }

    pub fn verify_chain(
        &self,
        chain: &RecursiveCheckpointChainEvidenceV1,
    ) -> Result<(), RecursiveCheckpointRejectReasonV1> {
        check_recursive_ver(chain.version())?;
        self.check_mode_backend(chain.mode(), chain.backend_label())?;
        self.check_chain_len(chain.chain_length())?;
        if usize::try_from(chain.chain_length())
            .map_err(|_| RecursiveCheckpointRejectReasonV1::ChainTooLong)?
            != chain.steps().len()
            || chain.step_digests().len() != chain.steps().len()
        {
            return Err(RecursiveCheckpointRejectReasonV1::StepSkipped);
        }
        let mut seen = BTreeSet::new();
        let mut prev_height = 0u64;
        let mut prev_output = [0u8; 32];
        for (idx, step) in chain.steps().iter().enumerate() {
            if !seen.insert(step.statement_digest()) {
                return Err(RecursiveCheckpointRejectReasonV1::StepRepeated);
            }
            let want_index =
                u32::try_from(idx).map_err(|_| RecursiveCheckpointRejectReasonV1::ChainTooLong)?;
            if step.chain_index() != want_index {
                return if step.chain_index() > want_index {
                    Err(RecursiveCheckpointRejectReasonV1::StepSkipped)
                } else {
                    Err(RecursiveCheckpointRejectReasonV1::StepReordered)
                };
            }
            if step.prev_root() != step.prior_output_root() {
                return Err(RecursiveCheckpointRejectReasonV1::PriorOutputMismatch);
            }
            let got_digest = step.digest();
            if chain.step_digests()[idx] != got_digest {
                return Err(RecursiveCheckpointRejectReasonV1::StatementDigestMismatch);
            }
            if idx > 0 {
                if step.height() <= prev_height {
                    return Err(RecursiveCheckpointRejectReasonV1::StepReordered);
                }
                if prev_output != step.prior_output_root() {
                    return Err(RecursiveCheckpointRejectReasonV1::PriorOutputMismatch);
                }
                if prev_output != step.prev_root() {
                    return Err(RecursiveCheckpointRejectReasonV1::OutputRootMismatch);
                }
            }
            prev_height = step.height();
            prev_output = step.output_root();
        }
        if chain.first_statement_digest() != chain.steps()[0].statement_digest()
            || chain.first_prev_root() != chain.steps()[0].prev_root()
        {
            return Err(RecursiveCheckpointRejectReasonV1::StatementDigestMismatch);
        }
        let last = chain
            .steps()
            .last()
            .ok_or(RecursiveCheckpointRejectReasonV1::ChainTooShort)?;
        if chain.last_statement_digest() != last.statement_digest() {
            return Err(RecursiveCheckpointRejectReasonV1::StatementDigestMismatch);
        }
        if chain.last_output_root() != last.output_root() {
            return Err(RecursiveCheckpointRejectReasonV1::OutputRootMismatch);
        }
        if chain.nova_chain_root() != digest_list(CHAIN_ROOT_LABEL, chain.step_digests()) {
            return Err(RecursiveCheckpointRejectReasonV1::StatementDigestMismatch);
        }
        if is_zero_root(&chain.measurements_root()) {
            return Err(RecursiveCheckpointRejectReasonV1::MeasurementMissing);
        }
        Ok(())
    }

    pub fn reject_canonical_admission(
        &self,
        _sidecar: &RecursiveCheckpointSidecarV1,
    ) -> Result<(), RecursiveCheckpointRejectReasonV1> {
        Err(RecursiveCheckpointRejectReasonV1::CanonicalAdmissionAttempt)
    }

    fn check_chain_bounds(
        &self,
        chain_index: u32,
        chain_length: u32,
    ) -> Result<(), RecursiveCheckpointRejectReasonV1> {
        self.check_chain_len(chain_length)?;
        if chain_index >= chain_length {
            return Err(RecursiveCheckpointRejectReasonV1::StepSkipped);
        }
        Ok(())
    }

    fn check_chain_len(&self, chain_length: u32) -> Result<(), RecursiveCheckpointRejectReasonV1> {
        if chain_length < self.min_chain_steps {
            return Err(RecursiveCheckpointRejectReasonV1::ChainTooShort);
        }
        if chain_length > self.max_chain_steps {
            return Err(RecursiveCheckpointRejectReasonV1::ChainTooLong);
        }
        Ok(())
    }

    fn check_proof_budget(
        &self,
        proof: &RecursiveCheckpointProofV1,
    ) -> Result<(), RecursiveCheckpointRejectReasonV1> {
        if proof.proof_bytes().is_empty() {
            return Err(RecursiveCheckpointRejectReasonV1::ProofBytesEmpty);
        }
        if proof_bytes_digest(proof.proof_bytes()) != proof.proof_bytes_digest() {
            return Err(RecursiveCheckpointRejectReasonV1::PublicInputDigestMismatch);
        }
        let cap = match proof.backend_label() {
            "nova_compressed_v1" => self
                .max_nova_block_proof_bytes
                .min(self.max_recursive_proof_bytes),
            "plonky3_stark_epoch_v1" => self
                .max_plonky3_epoch_proof_bytes
                .min(self.max_recursive_proof_bytes),
            _ => self.max_recursive_proof_bytes,
        };
        if proof.proof_bytes().len() > cap {
            return Err(RecursiveCheckpointRejectReasonV1::ProofBytesTooLarge);
        }
        Ok(())
    }

    fn check_mode_backend(
        &self,
        mode: RecursiveCheckpointModeV1,
        backend_label: &str,
    ) -> Result<(), RecursiveCheckpointRejectReasonV1> {
        if backend_label.trim().is_empty() {
            return Err(RecursiveCheckpointRejectReasonV1::BackendUnsupported);
        }
        if backend_label.contains("verified") {
            return Err(RecursiveCheckpointRejectReasonV1::VerifiedCodecMissing);
        }
        if backend_label.contains("audited")
            || backend_label.contains("production")
            || backend_label.contains("authoritative")
        {
            return Err(RecursiveCheckpointRejectReasonV1::BackendClaimUnsupported);
        }
        match (mode, backend_label) {
            (RecursiveCheckpointModeV1::FastClassicalCompressed, "nova_compressed_v1") => Ok(()),
            (RecursiveCheckpointModeV1::PqEpochFinality, "plonky3_stark_epoch_v1") => Ok(()),
            (RecursiveCheckpointModeV1::PqEpochFinality, "nova_compressed_v1") => {
                Err(RecursiveCheckpointRejectReasonV1::NovaPqAuthorityUnsupported)
            }
            (RecursiveCheckpointModeV1::FastClassicalCompressed, "plonky3_stark_epoch_v1") => {
                Err(RecursiveCheckpointRejectReasonV1::MixedEra)
            }
            _ => Err(RecursiveCheckpointRejectReasonV1::BackendUnsupported),
        }
    }
}

pub struct RecursiveCheckpointSidecarCodecV1;

impl RecursiveCheckpointSidecarCodecV1 {
    pub fn encode_bin(sidecar: &RecursiveCheckpointSidecarV1) -> Result<Vec<u8>, CheckpointError> {
        let verifier = repo_verifier()?;
        verifier.check_sidecar_shape(sidecar)?;
        Ok(BincodeCodec.serialize(sidecar)?)
    }

    pub fn decode_bin(bytes: &[u8]) -> Result<RecursiveCheckpointSidecarV1, CheckpointError> {
        let sidecar: RecursiveCheckpointSidecarV1 = BincodeCodec.deserialize(bytes)?;
        let verifier = repo_verifier()?;
        verifier.check_sidecar_shape(&sidecar)?;
        Ok(sidecar)
    }

    pub fn encode_json(sidecar: &RecursiveCheckpointSidecarV1) -> Result<Vec<u8>, CheckpointError> {
        let verifier = repo_verifier()?;
        verifier.check_sidecar_shape(sidecar)?;
        Ok(JsonCodec.serialize_pretty(sidecar)?)
    }

    pub fn decode_json(bytes: &[u8]) -> Result<RecursiveCheckpointSidecarV1, CheckpointError> {
        let sidecar: RecursiveCheckpointSidecarV1 = JsonCodec.deserialize(bytes)?;
        let verifier = repo_verifier()?;
        verifier.check_sidecar_shape(&sidecar)?;
        Ok(sidecar)
    }
}

pub(crate) fn encode_recursive_sidecar_bin_checked(
    sidecar: &RecursiveCheckpointSidecarV1,
) -> Result<Vec<u8>, CheckpointError> {
    RecursiveCheckpointSidecarCodecV1::encode_bin(sidecar)
}

pub(crate) fn decode_recursive_sidecar_bin_checked(
    bytes: &[u8],
) -> Result<RecursiveCheckpointSidecarV1, CheckpointError> {
    RecursiveCheckpointSidecarCodecV1::decode_bin(bytes)
}

pub(crate) fn encode_recursive_sidecar_json_checked(
    sidecar: &RecursiveCheckpointSidecarV1,
) -> Result<Vec<u8>, CheckpointError> {
    RecursiveCheckpointSidecarCodecV1::encode_json(sidecar)
}

pub(crate) fn decode_recursive_sidecar_json_checked(
    bytes: &[u8],
) -> Result<RecursiveCheckpointSidecarV1, CheckpointError> {
    RecursiveCheckpointSidecarCodecV1::decode_json(bytes)
}

fn repo_verifier() -> Result<RecursiveCheckpointVerifierV1, CheckpointError> {
    let cfg = CheckpointContractConfigV1::load_repo_default()?;
    Ok(RecursiveCheckpointVerifierV1::new(&cfg)?)
}

fn check_recursive_ver(
    version: RecursiveCheckpointVersion,
) -> Result<(), RecursiveCheckpointRejectReasonV1> {
    if version == RecursiveCheckpointVersion::CURRENT {
        return Ok(());
    }
    Err(RecursiveCheckpointRejectReasonV1::UnsupportedVersion)
}

fn digest_list(label: &'static str, items: &[[u8; 32]]) -> [u8; 32] {
    let mut bytes = frame_bytes(&(items.len() as u32).to_le_bytes());
    for item in items {
        bytes.extend_from_slice(&frame_bytes(item));
    }
    hash_zk::<StorRecursiveCheckpointDom>(label, &[bytes.as_slice()])
}

fn proof_bytes_digest(bytes: &[u8]) -> [u8; 32] {
    hash_zk::<StorRecursiveCheckpointDom>(PROOF_BYTES_LABEL, &[bytes])
}

fn push_framed_field(out: &mut Vec<u8>, name: &str, value: &[u8]) {
    out.extend_from_slice(&frame_str(name));
    out.extend_from_slice(&frame_bytes(value));
}

fn encode_opt_checkpoint_id(checkpoint_id: Option<CheckpointId>) -> Vec<u8> {
    let mut bytes = vec![0u8; 33];
    if let Some(checkpoint_id) = checkpoint_id {
        bytes[0] = 1;
        bytes[1..].copy_from_slice(checkpoint_id.as_bytes());
    }
    bytes
}

fn encode_opt_digest(digest: Option<[u8; 32]>) -> Vec<u8> {
    let mut bytes = vec![0u8; 33];
    if let Some(digest) = digest {
        bytes[0] = 1;
        bytes[1..].copy_from_slice(&digest);
    }
    bytes
}

fn encode_opt_reason(reason: Option<RecursiveCheckpointRejectReasonV1>) -> Vec<u8> {
    match reason {
        Some(reason) => {
            let raw = reason.as_str().as_bytes();
            let mut bytes = Vec::with_capacity(raw.len() + 1);
            bytes.push(1);
            bytes.extend_from_slice(raw);
            bytes
        }
        None => vec![0],
    }
}

fn encode_digest_vec(digests: &[[u8; 32]]) -> Vec<u8> {
    let mut bytes = frame_bytes(&(digests.len() as u32).to_le_bytes());
    for digest in digests {
        bytes.extend_from_slice(&frame_bytes(digest));
    }
    bytes
}

fn epoch_contains(start: u64, end: u64, height: u64) -> bool {
    start <= height && height <= end
}

fn is_zero_root(root: &[u8; 32]) -> bool {
    root.iter().all(|byte| *byte == 0)
}
