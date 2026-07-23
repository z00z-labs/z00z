//! Deterministic chain verification over write-only real-Nova receipts.

use core::fmt;
use std::collections::BTreeSet;

use z00z_crypto::sha256_256;

use super::{
    receipt::{CryptographicVerificationReceiptV2, RecursiveVerificationResultV2},
    recursive_measurement::NovaCadenceManifestV2,
    recursive_reject::RecursiveCheckpointRejectReasonV2,
    recursive_statement::NOVA_BACKEND_LABEL_V2,
    sidecar::RecursiveCheckpointSidecarV2,
    version_registry::{
        RECURSIVE_PARAMETER_GENERATION_V2, RECURSIVE_RUNTIME_PROFILE_GENERATION_V2,
    },
};

const CHAIN_DOMAIN_V2: &str = "z00z.storage.checkpoint.nova-chain.v2";
const CHAIN_ROOT_LABEL_V2: &str = "ordered_verified_receipts";
const MEASUREMENT_ROOT_LABEL_V2: &str = "ordered_measurements";
const RETENTION_FACTS_LABEL_V2: &str = "retention_input_facts";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NovaVerifiedReceiptFieldsV2 {
    pub(super) authority_generation: u64,
    pub(super) config_digest: [u8; 32],
    pub(super) registry_digest: [u8; 32],
    pub(super) height: u64,
    pub(super) checkpoint_id: [u8; 32],
    pub(super) predecessor: Option<[u8; 32]>,
    pub(super) public_input_digest: [u8; 32],
    pub(super) statement_digest: [u8; 32],
    pub(super) prior_output_root: [u8; 32],
    pub(super) output_root: [u8; 32],
    pub(super) envelope_digest: [u8; 32],
    pub(super) verifier_bundle_digest: [u8; 32],
    pub(super) result: RecursiveVerificationResultV2,
    pub(super) receipt_digest: [u8; 32],
}

/// Independently persisted statement facts expected for one receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NovaChainStatementV2 {
    pub height: u64,
    pub checkpoint_id: [u8; 32],
    pub predecessor: Option<[u8; 32]>,
    pub statement_digest: [u8; 32],
    pub public_input_digest: [u8; 32],
    pub prior_output_root: [u8; 32],
    pub output_root: [u8; 32],
    pub proof_envelope_digest: [u8; 32],
}

impl NovaChainStatementV2 {
    /// Resolve independently persisted statement facts from the sidecar.
    #[must_use]
    pub fn from_sidecar(sidecar: &RecursiveCheckpointSidecarV2) -> Self {
        Self {
            height: sidecar.height(),
            checkpoint_id: sidecar.checkpoint_id(),
            predecessor: sidecar.predecessor(),
            statement_digest: sidecar.statement_digest(),
            public_input_digest: sidecar.public_input_digest(),
            prior_output_root: sidecar.prior_output_root(),
            output_root: sidecar.output_root(),
            proof_envelope_digest: sidecar.envelope_digest(),
        }
    }
}

/// Required local performance/provenance measurement for one chain step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NovaChainMeasurementV2 {
    pub chain_index: u8,
    pub height: u64,
    pub receipt_digest: [u8; 32],
    pub verification_micros: u64,
    pub peak_rss_bytes: u64,
    backend_label: [u8; 28],
}

impl NovaChainMeasurementV2 {
    #[must_use]
    pub fn for_verified_receipt(
        chain_index: u8,
        receipt: &CryptographicVerificationReceiptV2,
        verification_micros: u64,
        peak_rss_bytes: u64,
    ) -> Self {
        let fields = receipt.chain_fields();
        Self {
            chain_index,
            height: fields.height,
            receipt_digest: fields.receipt_digest,
            verification_micros,
            peak_rss_bytes,
            backend_label: NOVA_BACKEND_LABEL_V2,
        }
    }
}

/// One immutable verified receipt plus independently resolved statement facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NovaChainEvidenceStepV2 {
    receipt: NovaVerifiedReceiptFieldsV2,
    statement: NovaChainStatementV2,
    measurement: Option<NovaChainMeasurementV2>,
}

impl NovaChainEvidenceStepV2 {
    #[must_use]
    pub fn new(
        receipt: &CryptographicVerificationReceiptV2,
        statement: NovaChainStatementV2,
        measurement: Option<NovaChainMeasurementV2>,
    ) -> Self {
        Self {
            receipt: receipt.chain_fields(),
            statement,
            measurement,
        }
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.statement.height
    }
}

/// Stable rejection plus the exact failing chain index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NovaChainErrorV2 {
    pub index: usize,
    pub reason: RecursiveCheckpointRejectReasonV2,
}

impl fmt::Display for NovaChainErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Nova chain index {}: {}",
            self.index, self.reason
        )
    }
}

impl std::error::Error for NovaChainErrorV2 {}

fn reject(index: usize, reason: RecursiveCheckpointRejectReasonV2) -> NovaChainErrorV2 {
    NovaChainErrorV2 { index, reason }
}

/// Cryptographically receipt-backed ordered chain after every semantic check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedNovaChainV2 {
    start_height: u64,
    end_height: u64,
    chain_root: [u8; 32],
    measurements_root: [u8; 32],
    final_envelope_digest: [u8; 32],
    authority_generation: u64,
    verifier_bundle_digest: [u8; 32],
}

impl VerifiedNovaChainV2 {
    pub fn verify(
        steps: &[NovaChainEvidenceStepV2],
        expected_chain_root: [u8; 32],
    ) -> Result<Self, NovaChainErrorV2> {
        if steps.len() < 3 {
            return Err(reject(
                steps.len(),
                RecursiveCheckpointRejectReasonV2::ChainTooShort,
            ));
        }
        if steps.len() > 5 {
            return Err(reject(5, RecursiveCheckpointRejectReasonV2::ChainTooLong));
        }

        let mut statements = BTreeSet::new();
        let mut checkpoint_ids = BTreeSet::new();
        for (index, step) in steps.iter().enumerate() {
            if step.receipt.result != RecursiveVerificationResultV2::VerifiedExactReload {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::BackendUnsupported,
                ));
            }
            let measurement = step.measurement.as_ref().ok_or_else(|| {
                reject(index, RecursiveCheckpointRejectReasonV2::MeasurementMissing)
            })?;
            if measurement.chain_index as usize != index
                || measurement.height != step.receipt.height
                || measurement.receipt_digest != step.receipt.receipt_digest
                || measurement.verification_micros == 0
                || measurement.peak_rss_bytes == 0
            {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::MeasurementMissing,
                ));
            }
            if measurement.backend_label != NOVA_BACKEND_LABEL_V2 {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::BackendUnsupported,
                ));
            }
            if step.statement.statement_digest != step.receipt.statement_digest {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::StatementDigestMismatch,
                ));
            }
            if step.statement.public_input_digest != step.receipt.public_input_digest {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::PublicInputDigestMismatch,
                ));
            }
            if step.statement.prior_output_root != step.receipt.prior_output_root {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
                ));
            }
            if step.statement.output_root != step.receipt.output_root {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::OutputRootMismatch,
                ));
            }
            if step.statement.proof_envelope_digest != step.receipt.envelope_digest {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::NovaChainRootMismatch,
                ));
            }
            if step.statement.height != step.receipt.height
                || step.statement.checkpoint_id != step.receipt.checkpoint_id
                || step.statement.predecessor != step.receipt.predecessor
            {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::StepReordered,
                ));
            }
            if !statements.insert(step.statement.statement_digest)
                || !checkpoint_ids.insert(step.statement.checkpoint_id)
            {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::StepRepeated,
                ));
            }
        }

        // Classify any globally non-monotonic sequence as a reorder before
        // classifying its earlier forward jump as a skipped step.
        for index in 1..steps.len() {
            if steps[index].statement.height == steps[index - 1].statement.height {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::StepRepeated,
                ));
            }
            if steps[index].statement.height < steps[index - 1].statement.height {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::StepReordered,
                ));
            }
        }
        for index in 1..steps.len() {
            let previous = &steps[index - 1];
            let step = &steps[index];
            if previous.statement.height.checked_add(1) != Some(step.statement.height) {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::StepSkipped,
                ));
            }
            if step.statement.predecessor != Some(previous.statement.checkpoint_id)
                || step.statement.prior_output_root != previous.statement.output_root
            {
                return Err(reject(
                    index,
                    RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
                ));
            }
            if step.receipt.authority_generation != previous.receipt.authority_generation
                || step.receipt.config_digest != previous.receipt.config_digest
                || step.receipt.registry_digest != previous.receipt.registry_digest
                || step.receipt.verifier_bundle_digest != previous.receipt.verifier_bundle_digest
            {
                return Err(reject(index, RecursiveCheckpointRejectReasonV2::MixedEra));
            }
        }

        let chain_root = Self::derive_chain_root(steps);
        if chain_root != expected_chain_root {
            return Err(reject(
                steps.len() - 1,
                RecursiveCheckpointRejectReasonV2::NovaChainRootMismatch,
            ));
        }
        let first = &steps[0].receipt;
        let last = &steps[steps.len() - 1].receipt;
        Ok(Self {
            start_height: first.height,
            end_height: last.height,
            chain_root,
            measurements_root: Self::derive_measurements_root(steps),
            final_envelope_digest: last.envelope_digest,
            authority_generation: first.authority_generation,
            verifier_bundle_digest: first.verifier_bundle_digest,
        })
    }

    #[must_use]
    pub fn derive_chain_root(steps: &[NovaChainEvidenceStepV2]) -> [u8; 32] {
        let mut bytes = Vec::with_capacity(8 + steps.len() * (8 + 32));
        bytes.extend_from_slice(&(steps.len() as u64).to_le_bytes());
        for (index, step) in steps.iter().enumerate() {
            bytes.extend_from_slice(&(index as u64).to_le_bytes());
            bytes.extend_from_slice(&step.receipt.envelope_digest);
        }
        sha256_256(CHAIN_DOMAIN_V2, CHAIN_ROOT_LABEL_V2, &[&bytes])
    }

    fn derive_measurements_root(steps: &[NovaChainEvidenceStepV2]) -> [u8; 32] {
        let mut bytes = Vec::with_capacity(8 + steps.len() * (1 + 8 * 3 + 32 + 28));
        bytes.extend_from_slice(&(steps.len() as u64).to_le_bytes());
        for (index, step) in steps.iter().enumerate() {
            bytes.extend_from_slice(&(index as u64).to_le_bytes());
            if let Some(measurement) = &step.measurement {
                bytes.push(measurement.chain_index);
                bytes.extend_from_slice(&measurement.height.to_le_bytes());
                bytes.extend_from_slice(&measurement.receipt_digest);
                bytes.extend_from_slice(&measurement.verification_micros.to_le_bytes());
                bytes.extend_from_slice(&measurement.peak_rss_bytes.to_le_bytes());
                bytes.extend_from_slice(&measurement.backend_label);
            }
        }
        sha256_256(CHAIN_DOMAIN_V2, MEASUREMENT_ROOT_LABEL_V2, &[&bytes])
    }

    #[must_use]
    pub const fn chain_root(&self) -> [u8; 32] {
        self.chain_root
    }

    #[must_use]
    pub const fn measurements_root(&self) -> [u8; 32] {
        self.measurements_root
    }

    pub fn retention_input_facts(
        &self,
        epoch_index: u64,
        reference_digest: [u8; 32],
    ) -> Result<NovaRetentionInputFactsV2, NovaChainErrorV2> {
        if reference_digest == [0; 32] {
            return Err(reject(
                0,
                RecursiveCheckpointRejectReasonV2::NovaChainRootMismatch,
            ));
        }
        let cadence = NovaCadenceManifestV2::authority_pinned();
        Ok(NovaRetentionInputFactsV2 {
            epoch_index,
            start_height: self.start_height,
            end_height: self.end_height,
            body_digest: self.final_envelope_digest,
            nova_chain_root: self.chain_root,
            authority_generation: self.authority_generation,
            parameter_generation: RECURSIVE_PARAMETER_GENERATION_V2,
            runtime_profile_generation: RECURSIVE_RUNTIME_PROFILE_GENERATION_V2,
            cadence_manifest_digest: cadence.digest(),
            verifier_bundle_digest: self.verifier_bundle_digest,
            reference_digest,
        })
    }
}

/// Immutable facts consumed by Plan 09. This type has no deletion decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NovaRetentionInputFactsV2 {
    pub epoch_index: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub body_digest: [u8; 32],
    pub nova_chain_root: [u8; 32],
    pub authority_generation: u64,
    pub parameter_generation: u32,
    pub runtime_profile_generation: u16,
    pub cadence_manifest_digest: [u8; 32],
    pub verifier_bundle_digest: [u8; 32],
    pub reference_digest: [u8; 32],
}

impl NovaRetentionInputFactsV2 {
    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        let mut bytes = Vec::with_capacity(4 * 8 + 4 + 2 + 5 * 32);
        bytes.extend_from_slice(&self.epoch_index.to_le_bytes());
        bytes.extend_from_slice(&self.start_height.to_le_bytes());
        bytes.extend_from_slice(&self.end_height.to_le_bytes());
        bytes.extend_from_slice(&self.authority_generation.to_le_bytes());
        bytes.extend_from_slice(&self.parameter_generation.to_le_bytes());
        bytes.extend_from_slice(&self.runtime_profile_generation.to_le_bytes());
        for digest in [
            self.body_digest,
            self.nova_chain_root,
            self.cadence_manifest_digest,
            self.verifier_bundle_digest,
            self.reference_digest,
        ] {
            bytes.extend_from_slice(&digest);
        }
        sha256_256(CHAIN_DOMAIN_V2, RETENTION_FACTS_LABEL_V2, &[&bytes])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type ChainMutationCaseV2 = (
        RecursiveCheckpointRejectReasonV2,
        fn(&mut NovaChainEvidenceStepV2),
    );

    fn steps(count: usize) -> Vec<NovaChainEvidenceStepV2> {
        (0..count)
            .map(|index| {
                let height = index as u64 + 1;
                let checkpoint_id = [index as u8 + 10; 32];
                let predecessor = index.checked_sub(1).map(|prior| [prior as u8 + 10; 32]);
                let prior_output_root = [index as u8 + 20; 32];
                let output_root = [index as u8 + 21; 32];
                let receipt = NovaVerifiedReceiptFieldsV2 {
                    authority_generation: 2,
                    config_digest: [61; 32],
                    registry_digest: [62; 32],
                    height,
                    checkpoint_id,
                    predecessor,
                    public_input_digest: [index as u8 + 30; 32],
                    statement_digest: [index as u8 + 40; 32],
                    prior_output_root,
                    output_root,
                    envelope_digest: [index as u8 + 50; 32],
                    verifier_bundle_digest: [60; 32],
                    result: RecursiveVerificationResultV2::VerifiedExactReload,
                    receipt_digest: [index as u8 + 70; 32],
                };
                NovaChainEvidenceStepV2 {
                    statement: NovaChainStatementV2 {
                        height,
                        checkpoint_id,
                        predecessor,
                        statement_digest: receipt.statement_digest,
                        public_input_digest: receipt.public_input_digest,
                        prior_output_root,
                        output_root,
                        proof_envelope_digest: receipt.envelope_digest,
                    },
                    measurement: Some(NovaChainMeasurementV2 {
                        chain_index: index as u8,
                        height,
                        receipt_digest: receipt.receipt_digest,
                        verification_micros: 1,
                        peak_rss_bytes: 1,
                        backend_label: NOVA_BACKEND_LABEL_V2,
                    }),
                    receipt,
                }
            })
            .collect()
    }

    fn verify(steps: &[NovaChainEvidenceStepV2]) -> Result<VerifiedNovaChainV2, NovaChainErrorV2> {
        VerifiedNovaChainV2::verify(steps, VerifiedNovaChainV2::derive_chain_root(steps))
    }

    #[test]
    fn test_valid_chains_pass() {
        for count in [3, 5] {
            let chain = steps(count);
            let verified = verify(&chain).unwrap();
            assert_eq!(
                verified.chain_root(),
                VerifiedNovaChainV2::derive_chain_root(&chain)
            );
            assert_eq!(
                verified.measurements_root(),
                VerifiedNovaChainV2::derive_measurements_root(&chain)
            );
            assert_ne!(
                verified
                    .retention_input_facts(0, [91; 32])
                    .unwrap()
                    .digest(),
                [0; 32]
            );
        }

        let chain = steps(3);
        let mut changed = chain.clone();
        changed[1].measurement.as_mut().unwrap().verification_micros += 1;
        assert_eq!(
            VerifiedNovaChainV2::derive_chain_root(&chain),
            VerifiedNovaChainV2::derive_chain_root(&changed)
        );
        assert_ne!(
            VerifiedNovaChainV2::derive_measurements_root(&chain),
            VerifiedNovaChainV2::derive_measurements_root(&changed)
        );
    }

    #[test]
    fn test_middle_step_errors() {
        let cases: &[ChainMutationCaseV2] = &[
            (
                RecursiveCheckpointRejectReasonV2::StatementDigestMismatch,
                |step| {
                    step.statement.statement_digest[0] ^= 1;
                },
            ),
            (
                RecursiveCheckpointRejectReasonV2::PublicInputDigestMismatch,
                |step| {
                    step.statement.public_input_digest[0] ^= 1;
                },
            ),
            (
                RecursiveCheckpointRejectReasonV2::PriorOutputMismatch,
                |step| {
                    step.statement.prior_output_root[0] ^= 1;
                },
            ),
            (
                RecursiveCheckpointRejectReasonV2::OutputRootMismatch,
                |step| {
                    step.statement.output_root[0] ^= 1;
                },
            ),
            (
                RecursiveCheckpointRejectReasonV2::NovaChainRootMismatch,
                |step| {
                    step.statement.proof_envelope_digest[0] ^= 1;
                },
            ),
            (
                RecursiveCheckpointRejectReasonV2::MeasurementMissing,
                |step| {
                    step.measurement = None;
                },
            ),
            (
                RecursiveCheckpointRejectReasonV2::BackendUnsupported,
                |step| {
                    step.measurement.as_mut().unwrap().backend_label = [0; 28];
                },
            ),
        ];
        for (reason, mutate) in cases {
            let mut chain = steps(5);
            mutate(&mut chain[2]);
            assert_eq!(
                verify(&chain).unwrap_err(),
                NovaChainErrorV2 {
                    index: 2,
                    reason: *reason,
                }
            );
        }
    }

    #[test]
    fn test_chain_order_errors() {
        let short = steps(2);
        assert_eq!(
            verify(&short).unwrap_err(),
            NovaChainErrorV2 {
                index: 2,
                reason: RecursiveCheckpointRejectReasonV2::ChainTooShort,
            }
        );
        let long = steps(6);
        assert_eq!(
            verify(&long).unwrap_err(),
            NovaChainErrorV2 {
                index: 5,
                reason: RecursiveCheckpointRejectReasonV2::ChainTooLong,
            }
        );

        let mut skipped = steps(5);
        skipped.remove(2);
        for (index, step) in skipped.iter_mut().enumerate() {
            step.measurement.as_mut().unwrap().chain_index = index as u8;
        }
        assert_eq!(
            verify(&skipped).unwrap_err(),
            NovaChainErrorV2 {
                index: 2,
                reason: RecursiveCheckpointRejectReasonV2::StepSkipped,
            }
        );

        let mut repeated = steps(5);
        repeated[2] = repeated[1].clone();
        repeated[2].measurement.as_mut().unwrap().chain_index = 2;
        assert_eq!(
            verify(&repeated).unwrap_err(),
            NovaChainErrorV2 {
                index: 2,
                reason: RecursiveCheckpointRejectReasonV2::StepRepeated,
            }
        );

        let mut repeated_height = steps(5);
        repeated_height[2].statement.height = repeated_height[1].statement.height;
        repeated_height[2].receipt.height = repeated_height[1].receipt.height;
        repeated_height[2].measurement.as_mut().unwrap().height = repeated_height[1].receipt.height;
        assert_eq!(
            verify(&repeated_height).unwrap_err(),
            NovaChainErrorV2 {
                index: 2,
                reason: RecursiveCheckpointRejectReasonV2::StepRepeated,
            }
        );

        let mut reordered = steps(5);
        reordered.swap(1, 2);
        for (index, step) in reordered.iter_mut().enumerate() {
            step.measurement.as_mut().unwrap().chain_index = index as u8;
        }
        assert_eq!(
            verify(&reordered).unwrap_err(),
            NovaChainErrorV2 {
                index: 2,
                reason: RecursiveCheckpointRejectReasonV2::StepReordered,
            }
        );

        let chain = steps(5);
        assert_eq!(
            VerifiedNovaChainV2::verify(&chain, [0; 32]).unwrap_err(),
            NovaChainErrorV2 {
                index: 4,
                reason: RecursiveCheckpointRejectReasonV2::NovaChainRootMismatch,
            }
        );

        let mut mixed_config = steps(5);
        mixed_config[2].receipt.config_digest[0] ^= 1;
        assert_eq!(
            verify(&mixed_config).unwrap_err(),
            NovaChainErrorV2 {
                index: 2,
                reason: RecursiveCheckpointRejectReasonV2::MixedEra,
            }
        );

        let mut mixed_registry = steps(5);
        mixed_registry[2].receipt.registry_digest[0] ^= 1;
        assert_eq!(
            verify(&mixed_registry).unwrap_err(),
            NovaChainErrorV2 {
                index: 2,
                reason: RecursiveCheckpointRejectReasonV2::MixedEra,
            }
        );
    }
}
