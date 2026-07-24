//! Non-authoritative cryptographic verification receipt.

use z00z_crypto::sha256_256;

use super::{
    adapter::{ReceiptIssuedPartsV2, ReloadedEvidenceV2},
    contract_config_v3::CheckpointConfigResolverV3,
    version_registry::{
        CheckpointVersionRegistryV2, RecursiveBoundedObjectV2,
        CRYPTOGRAPHIC_VERIFICATION_RECEIPT_DOMAIN_V2, RECEIPT_DIGEST_LABEL_V2,
    },
};
use crate::CheckpointError;

pub(super) const RECURSIVE_RECEIPT_PAYLOAD_BYTES_V2: usize = 588;
const PLONKY3_BASE_RECEIPT_MAGIC_V2: [u8; 8] = *b"Z00ZP3R2";
const PLONKY3_BASE_RECEIPT_VERSION_V2: u16 = 2;
const PLONKY3_BASE_RECEIPT_PAYLOAD_BYTES_V2: usize = 8 + 2 + 8 + 32 * 10 + 8 + 8 + 4 + 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RecursiveVerificationResultV2 {
    VerifiedExactReload = 1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReceiptWireV2 {
    wire_version: u16,
    authority_generation: u64,
    storage_generation: u64,
    height: u64,
    steps: u64,
    checkpoint_id: [u8; 32],
    predecessor: Option<[u8; 32]>,
    config_digest: [u8; 32],
    registry_digest: [u8; 32],
    verifier_bundle_digest: [u8; 32],
    public_input_digest: [u8; 32],
    initial_state_digest: [u8; 32],
    final_state_digest: [u8; 32],
    final_state_limbs: u64,
    successor_finalized_state_digest: [u8; 32],
    statement_digest: [u8; 32],
    checkpoint_link_digest: [u8; 32],
    prior_output_root: [u8; 32],
    output_root: [u8; 32],
    envelope_digest: [u8; 32],
    sidecar_digest: [u8; 32],
    gate_trace_digest: [u8; 32],
    backend_revision_result_digest: [u8; 32],
    result: RecursiveVerificationResultV2,
}

/// Registry and fixed-size validation completed before the receipt-issued
/// gate. It contains no success result, gate digest, or receipt bytes.
pub(super) struct PreparedReceiptV2 {
    preheader: Vec<u8>,
    registry_digest: [u8; 32],
}

/// Write-only evidence of the local unchanged-verifier result. There is no
/// decoder and therefore no path from receipt bytes to checkpoint authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CryptographicVerificationReceiptV2 {
    wire: ReceiptWireV2,
    canonical_bytes: Vec<u8>,
}

/// Write-only local receipt produced only after the real Plonky3 verifier
/// accepts the exact transition-selected AIR and proof bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plonky3BaseVerificationReceiptV2 {
    height: u64,
    statement_digest: [u8; 32],
    event_vector_digest: [u8; 32],
    parameter_digest: [u8; 32],
    security_budget_digest: [u8; 32],
    air_binding_digest: [u8; 32],
    proof_digest: [u8; 32],
    registry_digest: [u8; 32],
    runtime_profile_manifest_digest: [u8; 32],
    config_digest: [u8; 32],
    config_generation: u64,
    authority_generation: u64,
    parameter_generation: u32,
    runtime_profile_generation: u16,
    receipt_digest: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl Plonky3BaseVerificationReceiptV2 {
    pub(super) fn issue(
        verified: super::plonky3::VerifiedPlonky3BaseV2,
    ) -> Result<Self, CheckpointError> {
        if verified.height == 0
            || [
                verified.statement_digest,
                verified.event_vector_digest,
                verified.parameter_digest,
                verified.security_budget_digest,
                verified.air_binding_digest,
                verified.proof_digest,
            ]
            .contains(&[0; 32])
        {
            return Err(CheckpointError::Invariant);
        }
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        let row = registry.row(RecursiveBoundedObjectV2::Plonky3BaseVerificationReceipt)?;
        let active = CheckpointConfigResolverV3::resolve_active()?;
        let identity = active.identity();
        let runtime_profile_manifest_digest = row
            .runtime_profile_manifest_digest
            .ok_or(CheckpointError::Authority)?;
        let registry_digest = registry.digest();
        if identity.registry_digest != registry_digest
            || identity.runtime_profile_manifest_digest != runtime_profile_manifest_digest
            || row.runtime_profile_generation != Some(identity.runtime_profile_generation)
            || u64::from(row.authority_generation) != identity.authority_generation
            || row.parameter_generation != Some(identity.parameter_generation)
        {
            return Err(CheckpointError::Authority);
        }
        let mut prefix = Vec::with_capacity(PLONKY3_BASE_RECEIPT_PAYLOAD_BYTES_V2 - 32);
        prefix.extend_from_slice(&PLONKY3_BASE_RECEIPT_MAGIC_V2);
        prefix.extend_from_slice(&PLONKY3_BASE_RECEIPT_VERSION_V2.to_le_bytes());
        prefix.extend_from_slice(&verified.height.to_le_bytes());
        for digest in [
            verified.statement_digest,
            verified.event_vector_digest,
            verified.parameter_digest,
            verified.security_budget_digest,
            verified.air_binding_digest,
            verified.proof_digest,
        ] {
            prefix.extend_from_slice(&digest);
        }
        prefix.extend_from_slice(&registry_digest);
        prefix.extend_from_slice(&runtime_profile_manifest_digest);
        prefix.extend_from_slice(&identity.config_digest);
        prefix.extend_from_slice(&identity.config_generation.to_le_bytes());
        prefix.extend_from_slice(&identity.authority_generation.to_le_bytes());
        prefix.extend_from_slice(&identity.parameter_generation.to_le_bytes());
        prefix.extend_from_slice(&identity.runtime_profile_generation.to_le_bytes());
        let receipt_digest = sha256_256(
            "z00z.storage.checkpoint.plonky3.base-verification-receipt.v2",
            "receipt",
            &[&prefix],
        );
        prefix.extend_from_slice(&receipt_digest);
        if prefix.len() != PLONKY3_BASE_RECEIPT_PAYLOAD_BYTES_V2 {
            return Err(CheckpointError::Invariant);
        }
        let preheader = registry.encode_preheader(
            RecursiveBoundedObjectV2::Plonky3BaseVerificationReceipt,
            prefix.len(),
        )?;
        let mut canonical_bytes = Vec::with_capacity(preheader.len() + prefix.len());
        canonical_bytes.extend_from_slice(&preheader);
        canonical_bytes.extend_from_slice(&prefix);
        Ok(Self {
            height: verified.height,
            statement_digest: verified.statement_digest,
            event_vector_digest: verified.event_vector_digest,
            parameter_digest: verified.parameter_digest,
            security_budget_digest: verified.security_budget_digest,
            air_binding_digest: verified.air_binding_digest,
            proof_digest: verified.proof_digest,
            registry_digest,
            runtime_profile_manifest_digest,
            config_digest: identity.config_digest,
            config_generation: identity.config_generation,
            authority_generation: identity.authority_generation,
            parameter_generation: identity.parameter_generation,
            runtime_profile_generation: identity.runtime_profile_generation,
            receipt_digest,
            canonical_bytes,
        })
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
    pub const fn proof_digest(&self) -> [u8; 32] {
        self.proof_digest
    }

    #[must_use]
    pub const fn receipt_digest(&self) -> [u8; 32] {
        self.receipt_digest
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

impl CryptographicVerificationReceiptV2 {
    pub(super) fn prepare(
        _storage_generation: u64,
        envelope_digest: [u8; 32],
        sidecar_digest: [u8; 32],
        bindings: super::nova::NovaVerificationBindingsV2,
    ) -> Result<PreparedReceiptV2, CheckpointError> {
        if envelope_digest == [0; 32]
            || sidecar_digest == [0; 32]
            || bindings.authority_generation == 0
            || bindings.height == 0
            || bindings.steps == 0
            || bindings.gate_trace_digest != [0; 32]
        {
            return Err(CheckpointError::Invariant);
        }
        let registry = CheckpointVersionRegistryV2::authority_pinned()
            .map_err(|_| CheckpointError::Authority)?;
        let preheader = registry
            .encode_preheader(
                RecursiveBoundedObjectV2::CryptographicVerificationReceipt,
                RECURSIVE_RECEIPT_PAYLOAD_BYTES_V2,
            )
            .map_err(|_| CheckpointError::Canonical)?;
        Ok(PreparedReceiptV2 {
            preheader: preheader.to_vec(),
            registry_digest: registry.digest(),
        })
    }

    pub(super) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    #[must_use]
    pub fn height(&self) -> u64 {
        self.wire.height
    }

    #[must_use]
    pub fn result(&self) -> RecursiveVerificationResultV2 {
        self.wire.result
    }

    pub(super) fn chain_fields(&self) -> super::recursive_chain::NovaVerifiedReceiptFieldsV2 {
        super::recursive_chain::NovaVerifiedReceiptFieldsV2 {
            authority_generation: self.wire.authority_generation,
            config_digest: self.wire.config_digest,
            registry_digest: self.wire.registry_digest,
            height: self.wire.height,
            checkpoint_id: self.wire.checkpoint_id,
            predecessor: self.wire.predecessor,
            public_input_digest: self.wire.public_input_digest,
            statement_digest: self.wire.statement_digest,
            prior_output_root: self.wire.prior_output_root,
            output_root: self.wire.output_root,
            envelope_digest: self.wire.envelope_digest,
            verifier_bundle_digest: self.wire.verifier_bundle_digest,
            result: self.wire.result,
            receipt_digest: recursive_receipt_digest(&self.canonical_bytes),
        }
    }
}

impl PreparedReceiptV2 {
    /// Gate 16 supplies the only success capability. All subsequent operations
    /// are fixed-layout memory writes and cannot return a fallible status.
    pub(super) fn issue(
        self,
        issued: ReceiptIssuedPartsV2,
        _reloaded: ReloadedEvidenceV2,
    ) -> CryptographicVerificationReceiptV2 {
        let (storage_generation, envelope_digest, sidecar_digest, bindings) = issued.into_parts();
        let wire = ReceiptWireV2 {
            wire_version: 2,
            authority_generation: bindings.authority_generation,
            storage_generation,
            height: bindings.height,
            steps: bindings.steps,
            checkpoint_id: bindings.checkpoint_id,
            predecessor: bindings.predecessor,
            config_digest: bindings.config_digest,
            registry_digest: self.registry_digest,
            verifier_bundle_digest: bindings.bundle_digest,
            public_input_digest: bindings.public_input_digest,
            initial_state_digest: bindings.initial_state_digest,
            final_state_digest: bindings.final_state_digest,
            final_state_limbs: bindings.final_state_limbs,
            successor_finalized_state_digest: bindings.successor_finalized_state_digest,
            statement_digest: bindings.statement_digest,
            checkpoint_link_digest: bindings.checkpoint_link_digest,
            prior_output_root: bindings.prior_output_root,
            output_root: bindings.output_root,
            envelope_digest,
            sidecar_digest,
            gate_trace_digest: bindings.gate_trace_digest,
            backend_revision_result_digest: bindings.backend_revision_result_digest,
            result: RecursiveVerificationResultV2::VerifiedExactReload,
        };
        let canonical_bytes = encode_receipt_wire(&wire, self.preheader);
        CryptographicVerificationReceiptV2 {
            wire,
            canonical_bytes,
        }
    }
}

fn encode_receipt_wire(wire: &ReceiptWireV2, mut bytes: Vec<u8>) -> Vec<u8> {
    bytes.reserve(RECURSIVE_RECEIPT_PAYLOAD_BYTES_V2);
    bytes.extend_from_slice(&wire.wire_version.to_le_bytes());
    bytes.extend_from_slice(&wire.authority_generation.to_le_bytes());
    bytes.extend_from_slice(&wire.storage_generation.to_le_bytes());
    bytes.extend_from_slice(&wire.height.to_le_bytes());
    bytes.extend_from_slice(&wire.steps.to_le_bytes());
    bytes.extend_from_slice(&wire.checkpoint_id);
    bytes.push(u8::from(wire.predecessor.is_some()));
    bytes.extend_from_slice(&wire.predecessor.unwrap_or([0; 32]));
    for digest in [
        wire.config_digest,
        wire.registry_digest,
        wire.verifier_bundle_digest,
        wire.public_input_digest,
        wire.initial_state_digest,
        wire.final_state_digest,
    ] {
        bytes.extend_from_slice(&digest);
    }
    bytes.extend_from_slice(&wire.final_state_limbs.to_le_bytes());
    for digest in [
        wire.successor_finalized_state_digest,
        wire.statement_digest,
        wire.checkpoint_link_digest,
        wire.prior_output_root,
        wire.output_root,
        wire.envelope_digest,
        wire.sidecar_digest,
        wire.gate_trace_digest,
        wire.backend_revision_result_digest,
    ] {
        bytes.extend_from_slice(&digest);
    }
    bytes.push(wire.result as u8);
    bytes
}

pub(crate) fn recursive_receipt_digest(bytes: &[u8]) -> [u8; 32] {
    sha256_256(
        CRYPTOGRAPHIC_VERIFICATION_RECEIPT_DOMAIN_V2,
        RECEIPT_DIGEST_LABEL_V2,
        &[bytes],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_digest_is_framed() {
        assert_eq!(
            recursive_receipt_digest(b"receipt"),
            z00z_crypto::sha256_256(
                CRYPTOGRAPHIC_VERIFICATION_RECEIPT_DOMAIN_V2,
                RECEIPT_DIGEST_LABEL_V2,
                &[b"receipt"],
            )
        );
        assert_ne!(
            recursive_receipt_digest(b"receipt-a"),
            recursive_receipt_digest(b"receipt-b")
        );
        assert_ne!(
            recursive_receipt_digest(b"receipt"),
            recursive_receipt_digest(b"receipt\0")
        );
    }
}
