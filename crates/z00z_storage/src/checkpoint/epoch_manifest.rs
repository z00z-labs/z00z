//! Canonical exported envelope for one verified Plonky3 epoch.
//!
//! The rolling-history proof is the only exported proof body. Its recursive
//! circuit verifies the exact epoch theorem whose complete canonical statement
//! is carried beside it. Standalone epoch and child proofs remain local inputs.

use z00z_crypto::sha256_256;

use super::{
    epoch_range::{EpochCodecReaderV2, EpochRangeStatementV2},
    plonky3::{
        Plonky3HistoryAdapterV2, Plonky3HistoryAuthorityResolverV2, Plonky3HistoryProofV2,
        Plonky3ProofSizeStatusV2, ResolvedPlonky3HistoryAuthorityV2,
    },
    recursive_reject::RecursiveCheckpointRejectReasonV2,
    version_registry::{
        RecursiveBoundedObjectV2, PLONKY3_PUBLISH_BYTES_V2, PLONKY3_TARGET_BYTES_V2,
        RECURSIVE_INGRESS_BYTES_V2, RECURSIVE_OBJECT_PREHEADER_BYTES_V2,
    },
};
use crate::CheckpointError;

const EPOCH_MANIFEST_MAGIC_V2: [u8; 8] = *b"Z00ZEMF2";
const EPOCH_MANIFEST_WIRE_VERSION_V2: u16 = 2;
const EPOCH_MANIFEST_RETENTION_GENERATION_V2: u16 = 1;
const EPOCH_MANIFEST_DIGEST_LABEL_V2: &str = "canonical_manifest";
const EPOCH_MANIFEST_DIGEST_COUNT_V2: usize = 5;
const EPOCH_MANIFEST_PREFIX_BYTES_V2: usize =
    8 + 2 + EPOCH_MANIFEST_DIGEST_COUNT_V2 * 32 + 4 * 3 + 1 + 2;
const EPOCH_MANIFEST_OVERHEAD_BYTES_V2: usize =
    RECURSIVE_OBJECT_PREHEADER_BYTES_V2 + EPOCH_MANIFEST_PREFIX_BYTES_V2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochManifestInputsV2 {
    pub checkpoint_artifact_root: [u8; 32],
    pub archive_availability_manifest_root: [u8; 32],
}

/// The only publishable per-epoch Plonky3 object.
///
/// The exact epoch statement is small and self-describing. The single history
/// proof recursively verifies its epoch child, so publishing that child proof
/// again would duplicate evidence without strengthening the theorem.
#[derive(Clone, PartialEq, Eq)]
pub struct EpochManifestV2 {
    inputs: EpochManifestInputsV2,
    epoch_statement: EpochRangeStatementV2,
    history_proof: Plonky3HistoryProofV2,
    size_status: Plonky3ProofSizeStatusV2,
    digest: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl core::fmt::Debug for EpochManifestV2 {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("EpochManifestV2")
            .field("epoch_statement_digest", &self.epoch_statement.digest())
            .field("history_proof_digest", &self.history_proof.proof_digest())
            .field("size_status", &self.size_status)
            .field("digest", &self.digest)
            .field("canonical_bytes_len", &self.canonical_bytes.len())
            .finish()
    }
}

impl EpochManifestV2 {
    /// Construct the one complete publishable envelope.
    ///
    /// Actual verification of the history proof proves the exact epoch
    /// statement. No native success flag or child-proof digest substitutes for
    /// that recursive verifier call.
    pub fn new(
        inputs: EpochManifestInputsV2,
        epoch_statement: EpochRangeStatementV2,
        history_proof: Plonky3HistoryProofV2,
    ) -> Result<Self, CheckpointError> {
        let authority = Plonky3HistoryAuthorityResolverV2::resolve_active()?;
        Self::new_with_authority(inputs, epoch_statement, history_proof, &authority)
    }

    fn new_with_authority(
        inputs: EpochManifestInputsV2,
        epoch_statement: EpochRangeStatementV2,
        history_proof: Plonky3HistoryProofV2,
        authority: &ResolvedPlonky3HistoryAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        Self::preflight_availability(&epoch_statement, inputs)?;
        validate_history_binding(&epoch_statement, &history_proof)?;

        let total_len = EPOCH_MANIFEST_OVERHEAD_BYTES_V2
            .checked_add(epoch_statement.canonical_bytes().len())
            .and_then(|len| len.checked_add(history_proof.canonical_bytes().len()))
            .ok_or(CheckpointError::Overflow)?;
        let size_status = Self::classify_encoded_len(total_len)?;

        Plonky3HistoryAdapterV2::verify_statement_link_with_authority(
            &history_proof,
            &epoch_statement,
            authority,
        )?;

        let registry = authority.registry();
        let payload_len = total_len
            .checked_sub(RECURSIVE_OBJECT_PREHEADER_BYTES_V2)
            .ok_or(CheckpointError::Overflow)?;
        let preheader =
            registry.encode_preheader(RecursiveBoundedObjectV2::EpochManifest, payload_len)?;
        let complete_payload_bytes =
            u32::try_from(total_len).map_err(|_| CheckpointError::Limit)?;
        let mut canonical_bytes = Vec::with_capacity(total_len);
        canonical_bytes.extend_from_slice(&preheader);
        canonical_bytes.extend_from_slice(&encode_prefix(
            inputs,
            &epoch_statement,
            &history_proof,
            complete_payload_bytes,
            size_status,
        )?);
        canonical_bytes.extend_from_slice(epoch_statement.canonical_bytes());
        canonical_bytes.extend_from_slice(history_proof.canonical_bytes());
        if canonical_bytes.len() != total_len {
            return Err(CheckpointError::Invariant);
        }
        let domain = registry
            .row(RecursiveBoundedObjectV2::EpochManifest)?
            .cryptographic_domain;
        let digest = sha256_256(domain, EPOCH_MANIFEST_DIGEST_LABEL_V2, &[&canonical_bytes]);
        Ok(Self {
            inputs,
            epoch_statement,
            history_proof,
            size_status,
            digest,
            canonical_bytes,
        })
    }

    /// Decode and actual-verify the one canonical epoch/history envelope.
    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, CheckpointError> {
        let authority = Plonky3HistoryAuthorityResolverV2::resolve_active()?;
        Self::decode_canonical_with_authority(bytes, &authority)
    }

    /// Historical dual-read path selected by an exact trusted ConfigV3
    /// generation. The manifest bytes cannot choose or synthesize authority.
    pub fn decode_canonical_with_authority(
        bytes: &[u8],
        authority: &ResolvedPlonky3HistoryAuthorityV2,
    ) -> Result<Self, CheckpointError> {
        if bytes.len() > RECURSIVE_INGRESS_BYTES_V2 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::ProofBytesTooLarge,
            ));
        }
        let decoded_size_status = Self::classify_encoded_len(bytes.len())?;
        let registry = authority.registry();
        let preheader =
            registry.validate_preheader(bytes, RecursiveBoundedObjectV2::EpochManifest)?;
        if preheader.header_len != RECURSIVE_OBJECT_PREHEADER_BYTES_V2
            || preheader.declared_len
                != u64::try_from(
                    bytes
                        .len()
                        .checked_sub(preheader.header_len)
                        .ok_or(CheckpointError::Canonical)?,
                )
                .map_err(|_| CheckpointError::Limit)?
        {
            return Err(CheckpointError::Canonical);
        }
        let payload = bytes
            .get(preheader.header_len..)
            .ok_or(CheckpointError::Canonical)?;
        let (decoded, statement_bytes, history_bytes) = decode_payload(payload)?;
        if decoded.complete_payload_bytes
            != u32::try_from(bytes.len()).map_err(|_| CheckpointError::Limit)?
            || decoded.size_status != decoded_size_status
        {
            return Err(CheckpointError::Canonical);
        }

        let epoch_statement = EpochRangeStatementV2::decode_manifest_canonical_with_authority(
            statement_bytes,
            authority,
        )?;
        let history_proof =
            Plonky3HistoryProofV2::decode_local_with_authority(history_bytes, authority)?;
        validate_decoded(&decoded, &epoch_statement, &history_proof)?;
        let manifest = Self::new_with_authority(
            EpochManifestInputsV2 {
                checkpoint_artifact_root: decoded.checkpoint_artifact_root,
                archive_availability_manifest_root: decoded.archive_availability_manifest_root,
            },
            epoch_statement,
            history_proof,
            authority,
        )?;
        if manifest.canonical_bytes != bytes {
            return Err(CheckpointError::Canonical);
        }
        Ok(manifest)
    }

    /// Cheap root preflight shared by the asynchronous publication pipeline.
    /// It does not issue verification evidence.
    pub fn preflight_availability(
        statement: &EpochRangeStatementV2,
        inputs: EpochManifestInputsV2,
    ) -> Result<(), CheckpointError> {
        let statement_inputs = statement.inputs();
        if [
            statement.digest(),
            statement_inputs.chain_context_digest,
            statement_inputs.predicate_digest,
            statement_inputs.epoch_close_anchor_digest,
            statement_inputs.statement_digest_root,
            inputs.checkpoint_artifact_root,
            statement_inputs.checkpoint_link_root,
            statement_inputs.challenge_content_root,
            statement_inputs.da_payload_commitment,
            inputs.archive_availability_manifest_root,
            statement_inputs.witness_root,
            statement_inputs.delta_root,
            statement_inputs.config_digest,
            statement_inputs.registry_digest,
            statement_inputs.runtime_profile_manifest_digest,
            statement_inputs.parameter_digest,
            statement_inputs.security_budget_digest,
            statement_inputs.frontier_authority_digest,
            statement_inputs.verified_trace_chunk_root,
            statement_inputs.recursive_epoch_commitment,
        ]
        .contains(&[0; 32])
            || inputs.checkpoint_artifact_root != statement_inputs.checkpoint_artifact_root
            || statement_inputs.nova_chain_root == Some([0; 32])
        {
            return Err(incomplete_manifest());
        }
        Ok(())
    }

    /// Arithmetic-only size preflight. A successful result is not proof
    /// verification or publication authority.
    pub fn classify_encoded_len(
        encoded_len: usize,
    ) -> Result<Plonky3ProofSizeStatusV2, CheckpointError> {
        if encoded_len == 0 || encoded_len > RECURSIVE_INGRESS_BYTES_V2 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::ProofBytesTooLarge,
            ));
        }
        if encoded_len > PLONKY3_PUBLISH_BYTES_V2 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::ProofSizeBudgetExceeded,
            ));
        }
        Ok(if encoded_len <= PLONKY3_TARGET_BYTES_V2 {
            Plonky3ProofSizeStatusV2::WithinTarget
        } else {
            Plonky3ProofSizeStatusV2::TargetMissed
        })
    }

    #[must_use]
    pub const fn epoch_statement_digest(&self) -> [u8; 32] {
        self.epoch_statement.digest()
    }

    #[must_use]
    pub const fn plonky3_history_proof_digest(&self) -> [u8; 32] {
        self.history_proof.proof_digest()
    }

    #[must_use]
    pub const fn archive_availability_manifest_root(&self) -> [u8; 32] {
        self.inputs.archive_availability_manifest_root
    }

    #[must_use]
    pub const fn epoch_statement(&self) -> &EpochRangeStatementV2 {
        &self.epoch_statement
    }

    #[must_use]
    pub const fn history_proof(&self) -> &Plonky3HistoryProofV2 {
        &self.history_proof
    }

    #[must_use]
    pub fn complete_payload_bytes(&self) -> usize {
        self.canonical_bytes.len()
    }

    #[must_use]
    pub const fn size_status(&self) -> Plonky3ProofSizeStatusV2 {
        self.size_status
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

fn validate_history_binding(
    epoch: &EpochRangeStatementV2,
    history: &Plonky3HistoryProofV2,
) -> Result<(), CheckpointError> {
    let epoch_inputs = epoch.inputs();
    let history_inputs = history.statement().inputs();
    if history_inputs.last_epoch != epoch_inputs.epoch_index
        || history_inputs.last_height != epoch_inputs.end_height
        || history_inputs.cadence_blocks != epoch_inputs.cadence_blocks
        || history_inputs.chain_context_digest != epoch_inputs.chain_context_digest
        || history_inputs.predicate_digest != epoch_inputs.predicate_digest
        || history_inputs.previous_terminal_state_root != epoch_inputs.start_root
        || history_inputs.current_terminal_state_root != epoch_inputs.end_root
        || history_inputs.current_epoch_anchor_root != epoch_inputs.epoch_close_anchor_digest
        || history_inputs.exact_epoch_statement_digest != epoch.digest()
        || history_inputs.verifier_parameter_digest != epoch_inputs.parameter_digest
        || history_inputs.security_budget_digest != epoch_inputs.security_budget_digest
        || history_inputs.config_digest != epoch_inputs.config_digest
        || history_inputs.registry_digest != epoch_inputs.registry_digest
        || history_inputs.runtime_profile_manifest_digest
            != epoch_inputs.runtime_profile_manifest_digest
    {
        return Err(incomplete_manifest());
    }
    Ok(())
}

fn incomplete_manifest() -> CheckpointError {
    CheckpointError::RecursiveRejected(RecursiveCheckpointRejectReasonV2::EpochManifestIncomplete)
}

fn encode_prefix(
    inputs: EpochManifestInputsV2,
    statement: &EpochRangeStatementV2,
    history_proof: &Plonky3HistoryProofV2,
    complete_payload_bytes: u32,
    size_status: Plonky3ProofSizeStatusV2,
) -> Result<Vec<u8>, CheckpointError> {
    let mut payload = Vec::with_capacity(EPOCH_MANIFEST_PREFIX_BYTES_V2);
    payload.extend_from_slice(&EPOCH_MANIFEST_MAGIC_V2);
    payload.extend_from_slice(&EPOCH_MANIFEST_WIRE_VERSION_V2.to_le_bytes());
    for digest in [
        statement.digest(),
        inputs.checkpoint_artifact_root,
        inputs.archive_availability_manifest_root,
        history_proof.statement().digest(),
        history_proof.proof_digest(),
    ] {
        payload.extend_from_slice(&digest);
    }
    payload.extend_from_slice(
        &u32::try_from(statement.canonical_bytes().len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    payload.extend_from_slice(
        &u32::try_from(history_proof.canonical_bytes().len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    payload.extend_from_slice(&complete_payload_bytes.to_le_bytes());
    payload.push(match size_status {
        Plonky3ProofSizeStatusV2::WithinTarget => 1,
        Plonky3ProofSizeStatusV2::TargetMissed => 2,
    });
    payload.extend_from_slice(&EPOCH_MANIFEST_RETENTION_GENERATION_V2.to_le_bytes());
    if payload.len() != EPOCH_MANIFEST_PREFIX_BYTES_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(payload)
}

struct DecodedEpochManifestV2 {
    epoch_statement_digest: [u8; 32],
    checkpoint_artifact_root: [u8; 32],
    archive_availability_manifest_root: [u8; 32],
    history_statement_digest: [u8; 32],
    plonky3_history_proof_digest: [u8; 32],
    statement_bytes: u32,
    history_proof_bytes: u32,
    complete_payload_bytes: u32,
    size_status: Plonky3ProofSizeStatusV2,
    retention_generation: u16,
}

fn decode_payload(
    payload: &[u8],
) -> Result<(DecodedEpochManifestV2, &[u8], &[u8]), CheckpointError> {
    if payload.len() < EPOCH_MANIFEST_PREFIX_BYTES_V2 {
        return Err(CheckpointError::Canonical);
    }
    let mut reader = EpochCodecReaderV2::new(payload);
    if reader.array::<8>()? != EPOCH_MANIFEST_MAGIC_V2
        || reader.u16()? != EPOCH_MANIFEST_WIRE_VERSION_V2
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::UnsupportedVersion,
        ));
    }
    let decoded = DecodedEpochManifestV2 {
        epoch_statement_digest: reader.array()?,
        checkpoint_artifact_root: reader.array()?,
        archive_availability_manifest_root: reader.array()?,
        history_statement_digest: reader.array()?,
        plonky3_history_proof_digest: reader.array()?,
        statement_bytes: reader.u32()?,
        history_proof_bytes: reader.u32()?,
        complete_payload_bytes: reader.u32()?,
        size_status: match reader.u8()? {
            1 => Plonky3ProofSizeStatusV2::WithinTarget,
            2 => Plonky3ProofSizeStatusV2::TargetMissed,
            _ => return Err(CheckpointError::Canonical),
        },
        retention_generation: reader.u16()?,
    };
    validate_decoded_header(&decoded)?;
    let statement_len =
        usize::try_from(decoded.statement_bytes).map_err(|_| CheckpointError::Limit)?;
    let history_len =
        usize::try_from(decoded.history_proof_bytes).map_err(|_| CheckpointError::Limit)?;
    if statement_len == 0 || history_len == 0 {
        return Err(incomplete_manifest());
    }
    let statement = reader.take(statement_len)?;
    let history = reader.take(history_len)?;
    if !reader.is_done() {
        return Err(CheckpointError::Canonical);
    }
    Ok((decoded, statement, history))
}

fn validate_decoded_header(decoded: &DecodedEpochManifestV2) -> Result<(), CheckpointError> {
    if [
        decoded.epoch_statement_digest,
        decoded.checkpoint_artifact_root,
        decoded.archive_availability_manifest_root,
        decoded.history_statement_digest,
        decoded.plonky3_history_proof_digest,
    ]
    .contains(&[0; 32])
        || decoded.retention_generation != EPOCH_MANIFEST_RETENTION_GENERATION_V2
    {
        return Err(incomplete_manifest());
    }
    Ok(())
}

fn validate_decoded(
    decoded: &DecodedEpochManifestV2,
    epoch_statement: &EpochRangeStatementV2,
    history_proof: &Plonky3HistoryProofV2,
) -> Result<(), CheckpointError> {
    validate_decoded_header(decoded)?;
    if decoded.epoch_statement_digest != epoch_statement.digest()
        || decoded.checkpoint_artifact_root != epoch_statement.inputs().checkpoint_artifact_root
        || decoded.history_statement_digest != history_proof.statement().digest()
        || decoded.plonky3_history_proof_digest != history_proof.proof_digest()
    {
        return Err(incomplete_manifest());
    }
    validate_history_binding(epoch_statement, history_proof)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_header_rejects() {
        let mut payload = Vec::with_capacity(EPOCH_MANIFEST_PREFIX_BYTES_V2 + 2);
        payload.extend_from_slice(&EPOCH_MANIFEST_MAGIC_V2);
        payload.extend_from_slice(&EPOCH_MANIFEST_WIRE_VERSION_V2.to_le_bytes());
        payload.extend_from_slice(&[0_u8; EPOCH_MANIFEST_DIGEST_COUNT_V2 * 32]);
        payload.extend_from_slice(&1_u32.to_le_bytes());
        payload.extend_from_slice(&1_u32.to_le_bytes());
        payload.extend_from_slice(
            &u32::try_from(EPOCH_MANIFEST_OVERHEAD_BYTES_V2 + 2)
                .expect("manifest length")
                .to_le_bytes(),
        );
        payload.push(1);
        payload.extend_from_slice(&EPOCH_MANIFEST_RETENTION_GENERATION_V2.to_le_bytes());
        payload.extend_from_slice(&[1, 1]);
        assert!(matches!(
            decode_payload(&payload),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::EpochManifestIncomplete
            ))
        ));
    }
}
