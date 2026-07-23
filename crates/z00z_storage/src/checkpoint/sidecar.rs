//! Reference-only recursive checkpoint sidecar.

use serde::{de::DeserializeSeed, Deserialize, Deserializer, Serialize};
use z00z_crypto::sha256_256;
use z00z_utils::codec::{BincodeCodec, Codec};

use super::{
    nova::NovaVerificationBindingsV2,
    recursive_encoding::validate_object_ingress,
    recursive_reject::RecursiveCheckpointRejectReasonV2,
    recursive_statement::{NOVA_BACKEND_LABEL_V2, NOVA_PROOF_MODE_V2},
    version_registry::{
        CheckpointVersionRegistryV2, RecursiveBoundedObjectV2,
        RECURSIVE_CHECKPOINT_SIDECAR_DOMAIN_V2, RECURSIVE_SIDECAR_DIGEST_LABEL_V2,
    },
};
use crate::CheckpointError;

const RECURSIVE_SIDECAR_PAYLOAD_CAP_V2: usize = 64 * 1024;

/// Canonical absence marker until Plan 09 installs the storage-owned
/// `NovaRetentionStateV2` lifecycle. This is not a digest claim.
pub const NOVA_RETENTION_STATE_UNASSIGNED_V2: [u8; 32] = [0; 32];

/// Fixed, reference-only description of the sole keyless proof envelope.
/// No proof bytes, public-state vectors, nested payload, or decoder selector
/// can enter this type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveCheckpointProofV2 {
    version: u16,
    mode: [u8; 27],
    backend_label: [u8; 28],
    statement_digest: [u8; 32],
    public_input_digest: [u8; 32],
    prior_output_root: [u8; 32],
    output_root: [u8; 32],
    verifier_bundle_digest: [u8; 32],
    envelope_digest: [u8; 32],
    envelope_byte_length: u64,
    nova_retention_state_digest: [u8; 32],
}

/// Provider-neutral evidence reference. Proof bytes and public-state vectors
/// are intentionally absent; the content digest selects the keyless envelope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveCheckpointSidecarV2 {
    wire_version: u16,
    authority_generation: u64,
    storage_generation: u64,
    height: u64,
    steps: u64,
    checkpoint_id: [u8; 32],
    predecessor: Option<[u8; 32]>,
    statement_digest: [u8; 32],
    checkpoint_link_digest: [u8; 32],
    public_input_digest: [u8; 32],
    verifier_bundle_digest: [u8; 32],
    proof: RecursiveCheckpointProofV2,
}

impl RecursiveCheckpointSidecarV2 {
    pub(crate) fn new(
        storage_generation: u64,
        envelope_digest: [u8; 32],
        canonical_envelope_bytes: usize,
        bindings: NovaVerificationBindingsV2,
    ) -> Result<Self, CheckpointError> {
        // Generation zero is the canonical empty/genesis HJMT generation and
        // remains fully bound by the authority snapshot and sidecar bytes.
        if envelope_digest == [0; 32]
            || bindings.authority_generation == 0
            || bindings.steps == 0
            || canonical_envelope_bytes == 0
        {
            return Err(CheckpointError::Invariant);
        }
        Ok(Self {
            wire_version: 2,
            authority_generation: bindings.authority_generation,
            storage_generation,
            height: bindings.height,
            steps: bindings.steps,
            checkpoint_id: bindings.checkpoint_id,
            predecessor: bindings.predecessor,
            statement_digest: bindings.statement_digest,
            checkpoint_link_digest: bindings.checkpoint_link_digest,
            public_input_digest: bindings.public_input_digest,
            verifier_bundle_digest: bindings.bundle_digest,
            proof: RecursiveCheckpointProofV2 {
                version: 2,
                mode: NOVA_PROOF_MODE_V2,
                backend_label: NOVA_BACKEND_LABEL_V2,
                statement_digest: bindings.statement_digest,
                public_input_digest: bindings.public_input_digest,
                prior_output_root: bindings.prior_output_root,
                output_root: bindings.output_root,
                verifier_bundle_digest: bindings.bundle_digest,
                envelope_digest,
                envelope_byte_length: u64::try_from(canonical_envelope_bytes)
                    .map_err(|_| CheckpointError::Limit)?,
                nova_retention_state_digest: NOVA_RETENTION_STATE_UNASSIGNED_V2,
            },
        })
    }

    fn encode(&self) -> Result<Vec<u8>, CheckpointError> {
        self.validate()?;
        let payload = BincodeCodec
            .serialize(self)
            .map_err(|_| CheckpointError::Canonical)?;
        if payload.len() > RECURSIVE_SIDECAR_PAYLOAD_CAP_V2 {
            return Err(CheckpointError::Limit);
        }
        let registry = CheckpointVersionRegistryV2::authority_pinned()
            .map_err(|_| CheckpointError::Authority)?;
        let header = registry
            .encode_preheader(
                RecursiveBoundedObjectV2::RecursiveCheckpointSidecar,
                payload.len(),
            )
            .map_err(|_| CheckpointError::Canonical)?;
        let mut bytes = Vec::with_capacity(header.len() + payload.len());
        bytes.extend_from_slice(&header);
        bytes.extend_from_slice(&payload);
        validate_object_ingress(
            RecursiveBoundedObjectV2::RecursiveCheckpointSidecar,
            bytes.len(),
        )?;
        Ok(bytes)
    }

    /// Sole decoder: fixed registry preheader, then seeded compile-time-bounded
    /// bincode with exact consumption and canonical re-encoding.
    fn decode(bytes: &[u8]) -> Result<Self, CheckpointError> {
        let registry = CheckpointVersionRegistryV2::authority_pinned()
            .map_err(|_| CheckpointError::Authority)?;
        let validated = registry
            .validate_preheader(bytes, RecursiveBoundedObjectV2::RecursiveCheckpointSidecar)
            .map_err(|_| CheckpointError::Canonical)?;
        validate_object_ingress(
            RecursiveBoundedObjectV2::RecursiveCheckpointSidecar,
            bytes.len(),
        )?;
        let payload = &bytes[validated.header_len..];
        let sidecar = BincodeCodec
            .deserialize_seeded_bounded::<SidecarSeedV2, RECURSIVE_SIDECAR_PAYLOAD_CAP_V2>(
                payload,
                SidecarSeedV2,
            )
            .map_err(|_| CheckpointError::Canonical)?;
        sidecar.validate()?;
        if sidecar.encode()? != bytes {
            return Err(CheckpointError::Canonical);
        }
        Ok(sidecar)
    }

    fn validate(&self) -> Result<(), CheckpointError> {
        if self.wire_version != 2 || self.proof.version != 2 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::UnsupportedVersion,
            ));
        }
        if self.proof.backend_label != NOVA_BACKEND_LABEL_V2
            || self.proof.mode != NOVA_PROOF_MODE_V2
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::BackendUnsupported,
            ));
        }
        if self.proof.statement_digest != self.statement_digest {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StatementDigestMismatch,
            ));
        }
        if self.proof.public_input_digest != self.public_input_digest {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::PublicInputDigestMismatch,
            ));
        }
        if self.proof.verifier_bundle_digest != self.verifier_bundle_digest {
            return Err(CheckpointError::Authority);
        }
        if self.authority_generation == 0
            || self.height == 0
            || self.steps == 0
            || self.checkpoint_id == [0; 32]
            || self.statement_digest == [0; 32]
            || self.checkpoint_link_digest == [0; 32]
            || self.public_input_digest == [0; 32]
            || self.verifier_bundle_digest == [0; 32]
            || self.proof.envelope_digest == [0; 32]
            || self.proof.envelope_byte_length == 0
            || self.proof.prior_output_root == [0; 32]
            || self.proof.output_root == [0; 32]
            || self.proof.nova_retention_state_digest != NOVA_RETENTION_STATE_UNASSIGNED_V2
        {
            return Err(CheckpointError::Canonical);
        }
        Ok(())
    }

    #[must_use]
    pub fn envelope_digest(&self) -> [u8; 32] {
        self.proof.envelope_digest
    }

    #[must_use]
    pub fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub fn checkpoint_id(&self) -> [u8; 32] {
        self.checkpoint_id
    }

    #[must_use]
    pub fn predecessor(&self) -> Option<[u8; 32]> {
        self.predecessor
    }

    #[must_use]
    pub fn statement_digest(&self) -> [u8; 32] {
        self.statement_digest
    }

    #[must_use]
    pub fn public_input_digest(&self) -> [u8; 32] {
        self.public_input_digest
    }

    #[must_use]
    pub fn prior_output_root(&self) -> [u8; 32] {
        self.proof.prior_output_root
    }

    #[must_use]
    pub fn output_root(&self) -> [u8; 32] {
        self.proof.output_root
    }
}

/// Sole seeded binary codec owner for the sidecar/reference schema.
pub struct RecursiveCheckpointSidecarCodecV2;

impl RecursiveCheckpointSidecarCodecV2 {
    pub fn encode_bin(sidecar: &RecursiveCheckpointSidecarV2) -> Result<Vec<u8>, CheckpointError> {
        sidecar.encode()
    }

    pub fn decode_bin(bytes: &[u8]) -> Result<RecursiveCheckpointSidecarV2, CheckpointError> {
        RecursiveCheckpointSidecarV2::decode(bytes)
    }
}

/// Shape/binding-only check for untrusted persisted shadow evidence. This is
/// intentionally not a cryptographic verifier and can never issue a receipt.
pub(crate) fn check_shadow_sidecar_binding(
    sidecar: &RecursiveCheckpointSidecarV2,
    storage_generation: u64,
    envelope_digest: [u8; 32],
    canonical_envelope_bytes: usize,
    bindings: NovaVerificationBindingsV2,
) -> Result<(), CheckpointError> {
    sidecar.validate()?;
    if sidecar.statement_digest != bindings.statement_digest
        || sidecar.proof.statement_digest != bindings.statement_digest
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::StatementDigestMismatch,
        ));
    }
    if sidecar.public_input_digest != bindings.public_input_digest
        || sidecar.proof.public_input_digest != bindings.public_input_digest
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::PublicInputDigestMismatch,
        ));
    }
    if sidecar.proof.prior_output_root != bindings.prior_output_root {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
        ));
    }
    if sidecar.proof.output_root != bindings.output_root {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::OutputRootMismatch,
        ));
    }
    if sidecar.authority_generation != bindings.authority_generation
        || sidecar.verifier_bundle_digest != bindings.bundle_digest
        || sidecar.proof.verifier_bundle_digest != bindings.bundle_digest
    {
        return Err(CheckpointError::Authority);
    }
    if sidecar.storage_generation != storage_generation {
        return Err(CheckpointError::Storage);
    }
    if sidecar.height != bindings.height || sidecar.steps != bindings.steps {
        return Err(CheckpointError::EventOrder);
    }
    if sidecar.checkpoint_id != bindings.checkpoint_id {
        return Err(CheckpointError::KeyMix);
    }
    if sidecar.predecessor != bindings.predecessor
        || sidecar.checkpoint_link_digest != bindings.checkpoint_link_digest
    {
        return Err(CheckpointError::LinkMix);
    }
    if sidecar.proof.envelope_digest != envelope_digest
        || sidecar.proof.envelope_byte_length
            != u64::try_from(canonical_envelope_bytes).map_err(|_| CheckpointError::Limit)?
        || sidecar.proof.nova_retention_state_digest != NOVA_RETENTION_STATE_UNASSIGNED_V2
    {
        return Err(CheckpointError::Canonical);
    }
    Ok(())
}

struct SidecarSeedV2;

impl<'de> DeserializeSeed<'de> for SidecarSeedV2 {
    type Value = RecursiveCheckpointSidecarV2;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        RecursiveCheckpointSidecarV2::deserialize(deserializer)
    }
}

pub(crate) fn recursive_sidecar_digest(bytes: &[u8]) -> [u8; 32] {
    sha256_256(
        RECURSIVE_CHECKPOINT_SIDECAR_DOMAIN_V2,
        RECURSIVE_SIDECAR_DIGEST_LABEL_V2,
        &[bytes],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bindings() -> NovaVerificationBindingsV2 {
        NovaVerificationBindingsV2 {
            authority_generation: 2,
            config_digest: [1; 32],
            bundle_digest: [2; 32],
            public_input_digest: [3; 32],
            initial_state_digest: [4; 32],
            final_state_digest: [12; 32],
            final_state_limbs: 4096,
            successor_finalized_state_digest: [5; 32],
            statement_digest: [6; 32],
            checkpoint_link_digest: [7; 32],
            prior_output_root: [13; 32],
            output_root: [14; 32],
            trace_digest: [8; 32],
            gate_trace_digest: [0; 32],
            checkpoint_id: [9; 32],
            predecessor: None,
            height: 1,
            steps: 3,
            backend_revision_result_digest: [10; 32],
        }
    }

    #[test]
    fn test_sidecar_rejects_cross_type() {
        let bindings = bindings();
        let sidecar = RecursiveCheckpointSidecarV2::new(1, [11; 32], 4096, bindings).unwrap();
        let bytes = RecursiveCheckpointSidecarCodecV2::encode_bin(&sidecar).unwrap();
        assert_eq!(
            RecursiveCheckpointSidecarCodecV2::decode_bin(&bytes).unwrap(),
            sidecar
        );
        check_shadow_sidecar_binding(&sidecar, 1, [11; 32], 4096, bindings).unwrap();
        assert_eq!(
            sidecar.proof.nova_retention_state_digest,
            NOVA_RETENTION_STATE_UNASSIGNED_V2
        );
        let mut forged_retention_reference = sidecar.clone();
        forged_retention_reference.proof.nova_retention_state_digest = [0xA5; 32];
        assert!(matches!(
            RecursiveCheckpointSidecarCodecV2::encode_bin(&forged_retention_reference),
            Err(CheckpointError::Canonical)
        ));
        let mut wrong_statement = bindings;
        wrong_statement.statement_digest[0] ^= 1;
        assert!(matches!(
            check_shadow_sidecar_binding(&sidecar, 1, [11; 32], 4096, wrong_statement),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StatementDigestMismatch
            ))
        ));
        let mut trailing = bytes;
        trailing.push(0);
        assert!(RecursiveCheckpointSidecarCodecV2::decode_bin(&trailing).is_err());
    }

    #[test]
    fn test_sidecar_domain_owner() {
        assert_eq!(
            recursive_sidecar_digest(b"sidecar"),
            z00z_crypto::sha256_256(
                RECURSIVE_CHECKPOINT_SIDECAR_DOMAIN_V2,
                RECURSIVE_SIDECAR_DIGEST_LABEL_V2,
                &[b"sidecar"],
            )
        );
    }

    #[test]
    fn test_sidecar_field_rejections() {
        let bindings = bindings();
        let sidecar = RecursiveCheckpointSidecarV2::new(1, [11; 32], 4096, bindings).unwrap();

        let mut invalid = sidecar.clone();
        invalid.wire_version = 3;
        assert!(matches!(
            invalid.validate(),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::UnsupportedVersion
            ))
        ));

        let mut invalid = sidecar.clone();
        invalid.proof.version = 3;
        assert!(matches!(
            invalid.validate(),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::UnsupportedVersion
            ))
        ));

        let mut invalid = sidecar.clone();
        invalid.proof.mode[0] ^= 1;
        assert!(matches!(
            invalid.validate(),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::BackendUnsupported
            ))
        ));

        let mut invalid = sidecar.clone();
        invalid.proof.backend_label[0] ^= 1;
        assert!(matches!(
            invalid.validate(),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::BackendUnsupported
            ))
        ));

        let mut invalid = sidecar.clone();
        invalid.proof.statement_digest[0] ^= 1;
        assert!(matches!(
            invalid.validate(),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StatementDigestMismatch
            ))
        ));

        let mut invalid = sidecar.clone();
        invalid.proof.public_input_digest[0] ^= 1;
        assert!(matches!(
            invalid.validate(),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::PublicInputDigestMismatch
            ))
        ));

        let mut invalid = sidecar.clone();
        invalid.proof.prior_output_root[0] ^= 1;
        assert!(matches!(
            check_shadow_sidecar_binding(&invalid, 1, [11; 32], 4096, bindings),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::PriorOutputMismatch
            ))
        ));

        let mut invalid = sidecar.clone();
        invalid.proof.output_root[0] ^= 1;
        assert!(matches!(
            check_shadow_sidecar_binding(&invalid, 1, [11; 32], 4096, bindings),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::OutputRootMismatch
            ))
        ));

        let mut invalid = sidecar.clone();
        invalid.proof.verifier_bundle_digest[0] ^= 1;
        assert!(matches!(
            invalid.validate(),
            Err(CheckpointError::Authority)
        ));

        let mut invalid = sidecar.clone();
        invalid.authority_generation += 1;
        assert!(matches!(
            check_shadow_sidecar_binding(&invalid, 1, [11; 32], 4096, bindings),
            Err(CheckpointError::Authority)
        ));

        let mut invalid = sidecar.clone();
        invalid.statement_digest[0] ^= 1;
        assert!(matches!(
            invalid.validate(),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::StatementDigestMismatch
            ))
        ));

        let mut invalid = sidecar.clone();
        invalid.public_input_digest[0] ^= 1;
        assert!(matches!(
            invalid.validate(),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::PublicInputDigestMismatch
            ))
        ));

        let mut invalid = sidecar.clone();
        invalid.verifier_bundle_digest[0] ^= 1;
        assert!(matches!(
            invalid.validate(),
            Err(CheckpointError::Authority)
        ));

        let mut invalid = sidecar.clone();
        invalid.proof.envelope_digest[0] ^= 1;
        assert!(matches!(
            check_shadow_sidecar_binding(&invalid, 1, [11; 32], 4096, bindings),
            Err(CheckpointError::Canonical)
        ));

        let mut invalid = sidecar.clone();
        invalid.proof.envelope_byte_length += 1;
        assert!(matches!(
            check_shadow_sidecar_binding(&invalid, 1, [11; 32], 4096, bindings),
            Err(CheckpointError::Canonical)
        ));

        let mut invalid = sidecar.clone();
        invalid.proof.nova_retention_state_digest = [0xA5; 32];
        assert!(matches!(
            invalid.validate(),
            Err(CheckpointError::Canonical)
        ));

        let mut invalid = sidecar.clone();
        invalid.checkpoint_link_digest[0] ^= 1;
        assert!(matches!(
            check_shadow_sidecar_binding(&invalid, 1, [11; 32], 4096, bindings),
            Err(CheckpointError::LinkMix)
        ));

        let mut invalid = sidecar.clone();
        invalid.checkpoint_id[0] ^= 1;
        assert!(matches!(
            check_shadow_sidecar_binding(&invalid, 1, [11; 32], 4096, bindings),
            Err(CheckpointError::KeyMix)
        ));

        let mut invalid = sidecar.clone();
        invalid.predecessor = Some([0xB5; 32]);
        assert!(matches!(
            check_shadow_sidecar_binding(&invalid, 1, [11; 32], 4096, bindings),
            Err(CheckpointError::LinkMix)
        ));

        let mut invalid = sidecar.clone();
        invalid.height += 1;
        assert!(matches!(
            check_shadow_sidecar_binding(&invalid, 1, [11; 32], 4096, bindings),
            Err(CheckpointError::EventOrder)
        ));

        let mut invalid = sidecar.clone();
        invalid.steps += 1;
        assert!(matches!(
            check_shadow_sidecar_binding(&invalid, 1, [11; 32], 4096, bindings),
            Err(CheckpointError::EventOrder)
        ));

        let mut invalid = sidecar.clone();
        invalid.storage_generation += 1;
        assert!(matches!(
            check_shadow_sidecar_binding(&invalid, 1, [11; 32], 4096, bindings),
            Err(CheckpointError::Storage)
        ));

        let mut trailing = RecursiveCheckpointSidecarCodecV2::encode_bin(&sidecar).unwrap();
        trailing.push(0);
        assert!(matches!(
            RecursiveCheckpointSidecarCodecV2::decode_bin(&trailing),
            Err(CheckpointError::Canonical)
        ));
    }

    #[test]
    fn test_sidecar_genesis_binding() {
        let bindings = bindings();
        let sidecar = RecursiveCheckpointSidecarV2::new(0, [11; 32], 4096, bindings).unwrap();
        let bytes = RecursiveCheckpointSidecarCodecV2::encode_bin(&sidecar).unwrap();
        let reloaded = RecursiveCheckpointSidecarCodecV2::decode_bin(&bytes).unwrap();

        check_shadow_sidecar_binding(&reloaded, 0, [11; 32], 4096, bindings).unwrap();
        assert!(matches!(
            check_shadow_sidecar_binding(&reloaded, 1, [11; 32], 4096, bindings),
            Err(CheckpointError::Storage)
        ));
    }
}
