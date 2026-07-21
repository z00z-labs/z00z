//! Non-authoritative cryptographic verification receipt.

use serde::Serialize;
use z00z_crypto::sha256_256;
use z00z_utils::codec::{BincodeCodec, Codec};

use super::{
    adapter::PostwriteVerifiedV2,
    version_registry::{
        CheckpointVersionRegistryV2, RecursiveBoundedObjectV2,
        CRYPTOGRAPHIC_VERIFICATION_RECEIPT_DIGEST_LABEL_V2,
        CRYPTOGRAPHIC_VERIFICATION_RECEIPT_DOMAIN_V2,
    },
};
use crate::CheckpointError;

const RECURSIVE_RECEIPT_PAYLOAD_CAP_V2: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[repr(u8)]
pub enum RecursiveVerificationResultV2 {
    VerifiedExactReload = 1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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
    envelope_digest: [u8; 32],
    sidecar_digest: [u8; 32],
    gate_trace_digest: [u8; 32],
    backend_revision_result_digest: [u8; 32],
    result: RecursiveVerificationResultV2,
}

/// Private canonical receipt payload prepared before the final in-memory
/// issuance transition. Persisted bytes cannot construct this type or the
/// public receipt because no decoder exists.
pub(super) struct PreparedReceiptV2 {
    wire: ReceiptWireV2,
    canonical_bytes: Vec<u8>,
}

impl PreparedReceiptV2 {
    pub(super) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    #[cfg(test)]
    pub(super) fn corrupt_wire_for_test(&mut self) {
        self.wire.config_digest[0] ^= 1;
    }

    #[cfg(test)]
    pub(super) fn corrupt_bytes_for_test(&mut self) {
        self.canonical_bytes[0] ^= 1;
    }
}

/// Write-only evidence of the local unchanged-verifier result. There is no
/// decoder and therefore no path from receipt bytes to checkpoint authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CryptographicVerificationReceiptV2 {
    wire: ReceiptWireV2,
    canonical_bytes: Vec<u8>,
}

impl CryptographicVerificationReceiptV2 {
    pub(super) fn prepare_postwrite(
        postwrite: &PostwriteVerifiedV2,
    ) -> Result<PreparedReceiptV2, CheckpointError> {
        let preview = postwrite.receipt_preview();
        let registry = CheckpointVersionRegistryV2::authority_pinned()
            .map_err(|_| CheckpointError::Authority)?;
        let wire = receipt_wire_from_postwrite(&preview, &registry)?;
        let canonical_bytes = encode_receipt_wire(&wire, &registry)?;
        Ok(PreparedReceiptV2 {
            wire,
            canonical_bytes,
        })
    }

    /// Final checked type transition. All receipt bytes were prepared, durably
    /// reloaded, and authority-revalidated while `postwrite` still owned the
    /// 15-gate prefix. Release builds compare every prepared field and byte
    /// against the exact consumed postwrite capability before issuance.
    pub(super) fn issue_postwrite(
        postwrite: PostwriteVerifiedV2,
        prepared: PreparedReceiptV2,
    ) -> Result<Self, CheckpointError> {
        let issued = postwrite.issue_receipt()?;
        let registry = CheckpointVersionRegistryV2::authority_pinned()
            .map_err(|_| CheckpointError::Authority)?;
        let expected_wire = receipt_wire_from_postwrite(&issued, &registry)?;
        let expected_bytes = encode_receipt_wire(&expected_wire, &registry)?;
        if prepared.wire != expected_wire || prepared.canonical_bytes != expected_bytes {
            return Err(CheckpointError::Invariant);
        }
        Ok(Self {
            wire: prepared.wire,
            canonical_bytes: prepared.canonical_bytes,
        })
    }

    #[cfg(test)]
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
}

fn receipt_wire_from_postwrite(
    parts: &super::adapter::PostwriteReceiptPartsV2,
    registry: &CheckpointVersionRegistryV2,
) -> Result<ReceiptWireV2, CheckpointError> {
    // Generation zero is the canonical empty/genesis HJMT generation. It is
    // serialized and authority-revalidated like every later value.
    if parts.envelope_digest == [0; 32]
        || parts.sidecar_digest == [0; 32]
        || parts.bindings.steps == 0
    {
        return Err(CheckpointError::Invariant);
    }
    let bindings = parts.bindings;
    Ok(ReceiptWireV2 {
        wire_version: 2,
        authority_generation: bindings.authority_generation,
        storage_generation: parts.storage_generation,
        height: bindings.height,
        steps: bindings.steps,
        checkpoint_id: bindings.checkpoint_id,
        predecessor: bindings.predecessor,
        config_digest: bindings.config_digest,
        registry_digest: registry.digest(),
        verifier_bundle_digest: bindings.bundle_digest,
        public_input_digest: bindings.public_input_digest,
        initial_state_digest: bindings.initial_state_digest,
        final_state_digest: bindings.final_state_digest,
        final_state_limbs: bindings.final_state_limbs,
        successor_finalized_state_digest: bindings.successor_finalized_state_digest,
        statement_digest: bindings.statement_digest,
        checkpoint_link_digest: bindings.checkpoint_link_digest,
        envelope_digest: parts.envelope_digest,
        sidecar_digest: parts.sidecar_digest,
        gate_trace_digest: bindings.gate_trace_digest,
        backend_revision_result_digest: bindings.backend_revision_result_digest,
        result: RecursiveVerificationResultV2::VerifiedExactReload,
    })
}

fn encode_receipt_wire(
    wire: &ReceiptWireV2,
    registry: &CheckpointVersionRegistryV2,
) -> Result<Vec<u8>, CheckpointError> {
    if wire.wire_version != 2
        || wire.authority_generation == 0
        || wire.height == 0
        || wire.steps == 0
    {
        return Err(CheckpointError::Canonical);
    }
    let payload = BincodeCodec
        .serialize(wire)
        .map_err(|_| CheckpointError::Canonical)?;
    if payload.len() > RECURSIVE_RECEIPT_PAYLOAD_CAP_V2 {
        return Err(CheckpointError::Limit);
    }
    let header = registry
        .encode_preheader(
            RecursiveBoundedObjectV2::CryptographicVerificationReceipt,
            payload.len(),
        )
        .map_err(|_| CheckpointError::Canonical)?;
    let mut bytes = Vec::with_capacity(header.len() + payload.len());
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

pub(crate) fn recursive_receipt_digest(bytes: &[u8]) -> [u8; 32] {
    sha256_256(
        CRYPTOGRAPHIC_VERIFICATION_RECEIPT_DOMAIN_V2,
        CRYPTOGRAPHIC_VERIFICATION_RECEIPT_DIGEST_LABEL_V2,
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
                CRYPTOGRAPHIC_VERIFICATION_RECEIPT_DIGEST_LABEL_V2,
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
