//! Authority-pinned Nova cadence and role-delivery policy.
//!
//! Folding, local recovery, proof compression, and proof publication are four
//! separate decisions. This module records the measured policy; it does not
//! schedule jobs or send network traffic.

use z00z_crypto::sha256_256;

use super::{
    authority_artifacts::ACTIVE_VERIFIER_BUNDLE_DIGEST_V2,
    nova::{lockfile_digest, manifest_digest, source_revision_digest},
    version_registry::{
        CheckpointVersionRegistryV2, RecursiveBoundedObjectV2, NOVA_CADENCE_DOMAIN_V2,
        RECURSIVE_OBJECT_PREHEADER_BYTES_V2, RECURSIVE_PARAMETER_GENERATION_V2,
        RECURSIVE_PROFILE_MANIFEST_DIGEST_V2, RECURSIVE_RUNTIME_PROFILE_GENERATION_V2,
        RECURSIVE_RUNTIME_PROFILE_V2,
    },
};
use crate::CheckpointError;

const CADENCE_MANIFEST_MAGIC_V2: [u8; 4] = *b"ZNCM";
const CADENCE_WIRE_VERSION_V2: u16 = 2;
const CADENCE_PAYLOAD_BYTES_V2: usize = 168;
const CADENCE_DIGEST_LABEL_V2: &str = "canonical_manifest";
const MEASUREMENT_DIGEST_LABEL_V2: &str = "measurement_packet";
const MEASUREMENT_BACKEND_LABEL_V2: &str = "measurement_backend";
const MEASUREMENT_WORKER_LABEL_V2: &str = "measurement_worker";
const MEASUREMENT_FIXTURE_LABEL_V2: &str = "measurement_fixture";
const NOVA_MEASUREMENT_BACKEND_ID_V2: &[u8] =
    b"nova-snark=0.73.0;features=io;curve-cycle=pasta;transcript=keccak256";
const NOVA_MEASUREMENT_FIXTURE_ID_V2: &[u8] =
    b"phase069-plan06-canonical-mixed-nova-artifacts-and-continuous-chain-v2";
const NOVA_MILESTONE_HARNESS_V2: &[u8] = include_bytes!(
    "../../../../.github/skills/smart-tests-bootstrap/scripts/nova_milestone_tests.sh"
);
const NOVA_VERIFIER_RSS_HARNESS_V2: &[u8] = include_bytes!(
    "../../../../.github/skills/smart-tests-bootstrap/scripts/nova_verifier_rss_measurement.sh"
);

pub(crate) const NOVA_IMAGE_MAX_BYTES_V2: usize = 512 * 1024 * 1024;
pub(crate) const NOVA_SNAPSHOT_MAX_BYTES_V2: usize = 540 * 1024 * 1024;
pub(crate) const NOVA_JOURNAL_MAX_BYTES_V2: usize = 64 * 1024;
pub(crate) const NOVA_JOURNAL_MAX_ENTRIES_V2: usize = 256;
pub(crate) const NOVA_HOT_CAP_BYTES_V2: usize =
    NOVA_SNAPSHOT_MAX_BYTES_V2 * 3 + NOVA_JOURNAL_MAX_BYTES_V2;

/// Source-bound measurements used to select the generation-2 cadence policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NovaCadenceMeasurementPacketV2 {
    source_revision_digest: [u8; 32],
    lockfile_digest: [u8; 32],
    workspace_manifest_digest: [u8; 32],
    backend_identity_digest: [u8; 32],
    worker_identity_digest: [u8; 32],
    fixture_identity_digest: [u8; 32],
    verifier_bundle_digest: [u8; 32],
    runtime_profile_manifest_digest: [u8; 32],
    parameter_generation: u32,
    runtime_profile_generation: u16,
    block_interval_millis: u64,
    representative_fold_millis: u64,
    compression_setup_millis: u64,
    compression_prove_millis: u64,
    clean_verify_millis: u64,
    compressed_proof_bytes: u64,
    framed_envelope_bytes: u64,
    observed_peak_rss_bytes: u64,
    accumulator_image_cap_bytes: u64,
}

impl NovaCadenceMeasurementPacketV2 {
    fn authority_pinned() -> Self {
        let source_revision_digest = source_revision_digest();
        let lockfile_digest = lockfile_digest();
        let workspace_manifest_digest = manifest_digest();
        let backend_identity_digest = sha256_256(
            NOVA_CADENCE_DOMAIN_V2,
            MEASUREMENT_BACKEND_LABEL_V2,
            &[
                NOVA_MEASUREMENT_BACKEND_ID_V2,
                &lockfile_digest,
                &workspace_manifest_digest,
            ],
        );
        let worker_identity_digest = sha256_256(
            NOVA_CADENCE_DOMAIN_V2,
            MEASUREMENT_WORKER_LABEL_V2,
            &[
                NOVA_MILESTONE_HARNESS_V2,
                NOVA_VERIFIER_RSS_HARNESS_V2,
                &source_revision_digest,
            ],
        );
        let fixture_identity_digest = sha256_256(
            NOVA_CADENCE_DOMAIN_V2,
            MEASUREMENT_FIXTURE_LABEL_V2,
            &[
                NOVA_MEASUREMENT_FIXTURE_ID_V2,
                &source_revision_digest,
                &RECURSIVE_PROFILE_MANIFEST_DIGEST_V2,
                &RECURSIVE_PARAMETER_GENERATION_V2.to_le_bytes(),
                &RECURSIVE_RUNTIME_PROFILE_GENERATION_V2.to_le_bytes(),
            ],
        );
        Self {
            source_revision_digest,
            lockfile_digest,
            workspace_manifest_digest,
            backend_identity_digest,
            worker_identity_digest,
            fixture_identity_digest,
            verifier_bundle_digest: ACTIVE_VERIFIER_BUNDLE_DIGEST_V2,
            runtime_profile_manifest_digest: RECURSIVE_PROFILE_MANIFEST_DIGEST_V2,
            parameter_generation: RECURSIVE_PARAMETER_GENERATION_V2,
            runtime_profile_generation: RECURSIVE_RUNTIME_PROFILE_GENERATION_V2,
            block_interval_millis: 5_000,
            representative_fold_millis: 554,
            compression_setup_millis: 4_445,
            compression_prove_millis: 35_848,
            clean_verify_millis: 166_888,
            compressed_proof_bytes: 123_688,
            framed_envelope_bytes: 346_907,
            observed_peak_rss_bytes: 10_773_794_816,
            accumulator_image_cap_bytes: NOVA_IMAGE_MAX_BYTES_V2 as u64,
        }
    }

    fn has_complete_identities(self) -> bool {
        [
            self.source_revision_digest,
            self.lockfile_digest,
            self.workspace_manifest_digest,
            self.backend_identity_digest,
            self.worker_identity_digest,
            self.fixture_identity_digest,
            self.verifier_bundle_digest,
            self.runtime_profile_manifest_digest,
        ]
        .into_iter()
        .all(|digest| digest != [0; 32])
            && self.parameter_generation != 0
            && self.runtime_profile_generation != 0
    }

    fn digest(self) -> [u8; 32] {
        let values = [
            self.block_interval_millis,
            self.representative_fold_millis,
            self.compression_setup_millis,
            self.compression_prove_millis,
            self.clean_verify_millis,
            self.compressed_proof_bytes,
            self.framed_envelope_bytes,
            self.observed_peak_rss_bytes,
            self.accumulator_image_cap_bytes,
        ];
        let mut bytes = Vec::with_capacity(8 * 32 + 4 + 2 + values.len() * 8);
        for digest in [
            self.source_revision_digest,
            self.lockfile_digest,
            self.workspace_manifest_digest,
            self.backend_identity_digest,
            self.worker_identity_digest,
            self.fixture_identity_digest,
            self.verifier_bundle_digest,
            self.runtime_profile_manifest_digest,
        ] {
            bytes.extend_from_slice(&digest);
        }
        bytes.extend_from_slice(&self.parameter_generation.to_le_bytes());
        bytes.extend_from_slice(&self.runtime_profile_generation.to_le_bytes());
        for value in values {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        sha256_256(
            NOVA_CADENCE_DOMAIN_V2,
            MEASUREMENT_DIGEST_LABEL_V2,
            &[&bytes],
        )
    }
}

/// One versioned policy with four independently encoded cadence axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NovaCadenceManifestV2 {
    authority_generation: u32,
    parameter_generation: u32,
    runtime_profile_generation: u16,
    activation_height: u64,
    fold_cadence_blocks: u64,
    recovery_snapshot_cadence_blocks: u64,
    compression_cadence_blocks: u64,
    publication_cadence_blocks: u64,
    hot_recovery_snapshot_count: u16,
    max_hot_recovery_bytes: u64,
    runtime_profile_manifest_digest: [u8; 32],
    measurement_packet_digest: [u8; 32],
}

impl NovaCadenceManifestV2 {
    /// Return the only active measured policy for runtime-profile generation 2.
    #[must_use]
    pub fn authority_pinned() -> Self {
        Self {
            authority_generation: 2,
            parameter_generation: RECURSIVE_PARAMETER_GENERATION_V2,
            runtime_profile_generation: RECURSIVE_RUNTIME_PROFILE_GENERATION_V2,
            activation_height: 1,
            fold_cadence_blocks: 1,
            recovery_snapshot_cadence_blocks: 100,
            compression_cadence_blocks: 1_000,
            publication_cadence_blocks: 1_000,
            hot_recovery_snapshot_count: 2,
            max_hot_recovery_bytes: NOVA_HOT_CAP_BYTES_V2 as u64,
            runtime_profile_manifest_digest: RECURSIVE_PROFILE_MANIFEST_DIGEST_V2,
            measurement_packet_digest: NovaCadenceMeasurementPacketV2::authority_pinned().digest(),
        }
    }

    pub fn validate(&self) -> Result<(), CheckpointError> {
        let measurement = NovaCadenceMeasurementPacketV2::authority_pinned();
        if self != &Self::authority_pinned()
            || !measurement.has_complete_identities()
            || self.fold_cadence_blocks != 1
            || self.recovery_snapshot_cadence_blocks == 0
            || self.compression_cadence_blocks == 0
            || self.publication_cadence_blocks == 0
            || self.hot_recovery_snapshot_count != 2
            || self.max_hot_recovery_bytes == 0
            || self.max_hot_recovery_bytes == 24 * 1024 * 1024 * 1024
        {
            return Err(CheckpointError::Authority);
        }
        Ok(())
    }

    #[must_use]
    pub const fn fold_cadence_blocks(&self) -> u64 {
        self.fold_cadence_blocks
    }

    #[must_use]
    pub const fn recovery_snapshot_cadence_blocks(&self) -> u64 {
        self.recovery_snapshot_cadence_blocks
    }

    #[must_use]
    pub const fn compression_cadence_blocks(&self) -> u64 {
        self.compression_cadence_blocks
    }

    #[must_use]
    pub const fn publication_cadence_blocks(&self) -> u64 {
        self.publication_cadence_blocks
    }

    #[must_use]
    pub const fn max_hot_recovery_bytes(&self) -> u64 {
        self.max_hot_recovery_bytes
    }

    #[must_use]
    pub const fn runtime_profile_manifest_digest(&self) -> [u8; 32] {
        self.runtime_profile_manifest_digest
    }

    #[must_use]
    pub const fn authority_generation(&self) -> u32 {
        self.authority_generation
    }

    #[must_use]
    pub const fn parameter_generation(&self) -> u32 {
        self.parameter_generation
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        sha256_256(
            NOVA_CADENCE_DOMAIN_V2,
            CADENCE_DIGEST_LABEL_V2,
            &[&self.canonical_payload()],
        )
    }

    fn canonical_payload(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(CADENCE_PAYLOAD_BYTES_V2);
        bytes.extend_from_slice(&CADENCE_MANIFEST_MAGIC_V2);
        bytes.extend_from_slice(&CADENCE_WIRE_VERSION_V2.to_le_bytes());
        bytes.extend_from_slice(&self.authority_generation.to_le_bytes());
        bytes.extend_from_slice(&self.parameter_generation.to_le_bytes());
        bytes.extend_from_slice(&self.runtime_profile_generation.to_le_bytes());
        bytes.extend_from_slice(&self.activation_height.to_le_bytes());
        bytes.extend_from_slice(&self.fold_cadence_blocks.to_le_bytes());
        bytes.extend_from_slice(&self.recovery_snapshot_cadence_blocks.to_le_bytes());
        bytes.extend_from_slice(&self.compression_cadence_blocks.to_le_bytes());
        bytes.extend_from_slice(&self.publication_cadence_blocks.to_le_bytes());
        bytes.extend_from_slice(&self.hot_recovery_snapshot_count.to_le_bytes());
        bytes.extend_from_slice(&self.max_hot_recovery_bytes.to_le_bytes());
        bytes.extend_from_slice(&self.runtime_profile_manifest_digest);
        bytes.extend_from_slice(&self.measurement_packet_digest);
        bytes.extend_from_slice(&(RECURSIVE_RUNTIME_PROFILE_V2.len() as u16).to_le_bytes());
        bytes.extend_from_slice(RECURSIVE_RUNTIME_PROFILE_V2.as_bytes());
        bytes
    }

    pub fn encode(&self) -> Result<Vec<u8>, CheckpointError> {
        self.validate()?;
        let payload = self.canonical_payload();
        if payload.len() != CADENCE_PAYLOAD_BYTES_V2 {
            return Err(CheckpointError::Invariant);
        }
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        let header = registry
            .encode_preheader(RecursiveBoundedObjectV2::NovaCadenceManifest, payload.len())?;
        let mut bytes = Vec::with_capacity(header.len() + payload.len());
        bytes.extend_from_slice(&header);
        bytes.extend_from_slice(&payload);
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, CheckpointError> {
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        let header =
            registry.validate_preheader(bytes, RecursiveBoundedObjectV2::NovaCadenceManifest)?;
        if header.header_len != RECURSIVE_OBJECT_PREHEADER_BYTES_V2
            || header.declared_len as usize != CADENCE_PAYLOAD_BYTES_V2
        {
            return Err(CheckpointError::Canonical);
        }
        let payload = bytes
            .get(header.header_len..)
            .ok_or(CheckpointError::Canonical)?;
        let expected = Self::authority_pinned();
        if payload != expected.canonical_payload() {
            return Err(CheckpointError::Authority);
        }
        expected.validate()?;
        Ok(expected)
    }
}

/// Who is allowed to request a non-scheduled compression/publication decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NovaCompressionAuthorityV2 {
    Scheduled,
    LocalOperator,
    RecoveryWorkflow,
    Peer,
    Wallet,
}

/// The independently requested cadence action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NovaCadenceRequestV2 {
    Scheduled,
    RecoverySnapshot,
    Compress,
    Publish,
    CompressAndPublish,
}

/// Independent decisions for one height. No field implies another field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NovaCadenceActionV2 {
    pub is_fold_required: bool,
    pub is_recovery_snapshot_required: bool,
    pub is_compression_required: bool,
    pub is_publication_required: bool,
}

/// Checked decision facade over the authority-pinned cadence manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NovaCompressionPolicyV2 {
    manifest: NovaCadenceManifestV2,
}

impl NovaCompressionPolicyV2 {
    pub fn authority_pinned() -> Result<Self, CheckpointError> {
        let manifest = NovaCadenceManifestV2::authority_pinned();
        manifest.validate()?;
        Ok(Self { manifest })
    }

    #[must_use]
    pub const fn manifest(&self) -> &NovaCadenceManifestV2 {
        &self.manifest
    }

    pub fn action(
        &self,
        height: u64,
        authority: NovaCompressionAuthorityV2,
        request: NovaCadenceRequestV2,
    ) -> Result<NovaCadenceActionV2, CheckpointError> {
        self.manifest.validate()?;
        if height < self.manifest.activation_height {
            return Err(CheckpointError::Authority);
        }
        let scheduled_recovery =
            height.is_multiple_of(self.manifest.recovery_snapshot_cadence_blocks);
        let (recovery_snapshot, compress, publish) = match (authority, request) {
            (NovaCompressionAuthorityV2::Scheduled, NovaCadenceRequestV2::Scheduled) => (
                scheduled_recovery,
                height.is_multiple_of(self.manifest.compression_cadence_blocks),
                height.is_multiple_of(self.manifest.publication_cadence_blocks),
            ),
            (NovaCompressionAuthorityV2::Scheduled, NovaCadenceRequestV2::RecoverySnapshot)
                if scheduled_recovery =>
            {
                (true, false, false)
            }
            (
                NovaCompressionAuthorityV2::LocalOperator
                | NovaCompressionAuthorityV2::RecoveryWorkflow,
                NovaCadenceRequestV2::RecoverySnapshot,
            ) => (true, false, false),
            (
                NovaCompressionAuthorityV2::LocalOperator
                | NovaCompressionAuthorityV2::RecoveryWorkflow,
                NovaCadenceRequestV2::Compress,
            ) => (scheduled_recovery, true, false),
            (
                NovaCompressionAuthorityV2::LocalOperator
                | NovaCompressionAuthorityV2::RecoveryWorkflow,
                NovaCadenceRequestV2::Publish,
            ) => (scheduled_recovery, false, true),
            (
                NovaCompressionAuthorityV2::LocalOperator
                | NovaCompressionAuthorityV2::RecoveryWorkflow,
                NovaCadenceRequestV2::CompressAndPublish,
            ) => (scheduled_recovery, true, true),
            _ => return Err(CheckpointError::Authority),
        };
        Ok(NovaCadenceActionV2 {
            is_fold_required: true,
            is_recovery_snapshot_required: recovery_snapshot,
            is_compression_required: compress,
            is_publication_required: publish,
        })
    }
}

/// Roles relevant to recursive evidence distribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NovaEvidenceRoleV2 {
    CanonicalValidator,
    Watcher,
    Wallet,
    RecursiveVerifier,
}

/// A byte-count-free routing decision; transport ownership remains outside Plan 06.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NovaRoleDeliveryV2 {
    pub public_parameters_bytes: u64,
    pub prover_key_bytes: u64,
    pub verifier_key_fetches: u8,
    pub proof_envelope_fetches: u8,
}

impl NovaRoleDeliveryV2 {
    pub fn for_action(
        role: NovaEvidenceRoleV2,
        action: NovaCadenceActionV2,
        is_verifier_key_cached: bool,
    ) -> Self {
        match role {
            NovaEvidenceRoleV2::RecursiveVerifier => Self {
                public_parameters_bytes: 0,
                prover_key_bytes: 0,
                verifier_key_fetches: u8::from(!is_verifier_key_cached),
                proof_envelope_fetches: u8::from(action.is_publication_required),
            },
            NovaEvidenceRoleV2::CanonicalValidator
            | NovaEvidenceRoleV2::Watcher
            | NovaEvidenceRoleV2::Wallet => Self {
                public_parameters_bytes: 0,
                prover_key_bytes: 0,
                verifier_key_fetches: 0,
                proof_envelope_fetches: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measurement_binds_identities() {
        let packet = NovaCadenceMeasurementPacketV2::authority_pinned();
        assert!(packet.has_complete_identities());
        assert_eq!(packet.source_revision_digest, source_revision_digest());
        assert_eq!(packet.lockfile_digest, lockfile_digest());
        assert_eq!(packet.workspace_manifest_digest, manifest_digest());
        assert_eq!(
            packet.verifier_bundle_digest,
            ACTIVE_VERIFIER_BUNDLE_DIGEST_V2
        );

        for mutate in [
            |packet: &mut NovaCadenceMeasurementPacketV2| packet.source_revision_digest[0] ^= 1,
            |packet: &mut NovaCadenceMeasurementPacketV2| packet.lockfile_digest[0] ^= 1,
            |packet: &mut NovaCadenceMeasurementPacketV2| packet.workspace_manifest_digest[0] ^= 1,
            |packet: &mut NovaCadenceMeasurementPacketV2| packet.backend_identity_digest[0] ^= 1,
            |packet: &mut NovaCadenceMeasurementPacketV2| packet.worker_identity_digest[0] ^= 1,
            |packet: &mut NovaCadenceMeasurementPacketV2| packet.fixture_identity_digest[0] ^= 1,
            |packet: &mut NovaCadenceMeasurementPacketV2| packet.verifier_bundle_digest[0] ^= 1,
            |packet: &mut NovaCadenceMeasurementPacketV2| {
                packet.runtime_profile_manifest_digest[0] ^= 1
            },
        ] {
            let mut mutated = packet;
            mutate(&mut mutated);
            assert_ne!(mutated.digest(), packet.digest());
        }
    }

    #[test]
    fn test_cadence_registry_roundtrip() {
        let manifest = NovaCadenceManifestV2::authority_pinned();
        let bytes = manifest.encode().unwrap();
        assert_eq!(NovaCadenceManifestV2::decode(&bytes).unwrap(), manifest);
        assert_eq!(manifest.fold_cadence_blocks(), 1);
        assert_eq!(manifest.recovery_snapshot_cadence_blocks(), 100);
        assert_eq!(manifest.compression_cadence_blocks(), 1_000);
        assert_eq!(manifest.publication_cadence_blocks(), 1_000);
        assert!(manifest.max_hot_recovery_bytes() > 0);
    }

    #[test]
    fn test_cadence_axes_independent() {
        let policy = NovaCompressionPolicyV2::authority_pinned().unwrap();
        assert_eq!(
            policy
                .action(
                    1,
                    NovaCompressionAuthorityV2::Scheduled,
                    NovaCadenceRequestV2::Scheduled,
                )
                .unwrap(),
            NovaCadenceActionV2 {
                is_fold_required: true,
                is_recovery_snapshot_required: false,
                is_compression_required: false,
                is_publication_required: false,
            }
        );
        assert!(
            policy
                .action(
                    100,
                    NovaCompressionAuthorityV2::Scheduled,
                    NovaCadenceRequestV2::Scheduled,
                )
                .unwrap()
                .is_recovery_snapshot_required
        );
        let epoch = policy
            .action(
                1_000,
                NovaCompressionAuthorityV2::Scheduled,
                NovaCadenceRequestV2::Scheduled,
            )
            .unwrap();
        assert!(
            epoch.is_fold_required
                && epoch.is_recovery_snapshot_required
                && epoch.is_compression_required
                && epoch.is_publication_required
        );
        assert!(policy
            .action(
                999,
                NovaCompressionAuthorityV2::Peer,
                NovaCadenceRequestV2::Publish,
            )
            .is_err());
        assert!(
            policy
                .action(
                    999,
                    NovaCompressionAuthorityV2::RecoveryWorkflow,
                    NovaCadenceRequestV2::RecoverySnapshot,
                )
                .unwrap()
                .is_recovery_snapshot_required
        );
        assert!(policy
            .action(
                999,
                NovaCompressionAuthorityV2::Scheduled,
                NovaCadenceRequestV2::RecoverySnapshot,
            )
            .is_err());
        assert!(policy
            .action(
                999,
                NovaCompressionAuthorityV2::Peer,
                NovaCadenceRequestV2::RecoverySnapshot,
            )
            .is_err());
    }

    #[test]
    fn test_ordinary_roles_empty() {
        let action = NovaCadenceActionV2 {
            is_fold_required: true,
            is_recovery_snapshot_required: true,
            is_compression_required: true,
            is_publication_required: true,
        };
        for role in [
            NovaEvidenceRoleV2::CanonicalValidator,
            NovaEvidenceRoleV2::Watcher,
            NovaEvidenceRoleV2::Wallet,
        ] {
            assert_eq!(
                NovaRoleDeliveryV2::for_action(role, action, false),
                NovaRoleDeliveryV2 {
                    public_parameters_bytes: 0,
                    prover_key_bytes: 0,
                    verifier_key_fetches: 0,
                    proof_envelope_fetches: 0,
                }
            );
        }
        let verifier =
            NovaRoleDeliveryV2::for_action(NovaEvidenceRoleV2::RecursiveVerifier, action, false);
        assert_eq!(verifier.verifier_key_fetches, 1);
        assert_eq!(verifier.proof_envelope_fetches, 1);
    }
}
