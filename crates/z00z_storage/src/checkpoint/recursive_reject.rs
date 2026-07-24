//! Stable rejection taxonomy for the sole recursive checkpoint V2 wire surface.

use core::fmt;

use crate::CheckpointError;

/// Stable semantic rejection reasons emitted by recursive checkpoint V2 admission.
///
/// Codes are consensus-facing `u16` values. They are append-only and encode as
/// exactly two little-endian bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u16)]
pub enum RecursiveCheckpointRejectReasonV2 {
    UnsupportedVersion = 1,
    UnknownField = 2,
    StatementDigestMismatch = 3,
    PublicInputDigestMismatch = 4,
    PriorOutputMismatch = 5,
    OutputRootMismatch = 6,
    BackendUnsupported = 7,
    BackendClaimUnsupported = 8,
    ProofBytesEmpty = 9,
    ProofBytesTooLarge = 10,
    NovaPqAuthorityUnsupported = 11,
    NovaChainRootMismatch = 12,
    Plonky3CanonicalRangeMissing = 13,
    Plonky3DependsOnlyOnNova = 14,
    Plonky3UnauditedPromotion = 15,
    HybridCadenceMismatch = 16,
    EpochManifestIncomplete = 17,
    ProofSizeBudgetExceeded = 18,
    CelestiaPermanentStorageUnsupported = 19,
    IpfsPinningMissing = 20,
    ArchiveReplicationInsufficient = 21,
    ArchiveProviderReceiptMissing = 22,
    RetrievalAuditMissing = 23,
    RetrievalAuditFailed = 24,
    SnapshotBindingIncomplete = 25,
    PruningBeforeArchiveFinality = 26,
    ArchiveNodePruningUnsupported = 27,
    SidecarAuthoritative = 28,
    MeasurementMissing = 29,
    ChainTooShort = 30,
    ChainTooLong = 31,
    StepSkipped = 32,
    StepRepeated = 33,
    StepReordered = 34,
    WitnessUnavailable = 35,
    CanonicalAdmissionAttempt = 36,
    VerifiedCodecMissing = 37,
    MixedEra = 38,
    DaReadinessMissing = 39,
    PqInlineAnchorUnsupported = 40,
    PqCadenceDisabled = 41,
    PqCadenceInvalid = 42,
    PqLiveCadenceStageMismatch = 43,
    PqAnchorMissing = 44,
    PqAnchorDigestMismatch = 45,
    PqAnchorIncomplete = 46,
    RecursiveDocumentationIncomplete = 47,
    Plonky3SecurityBudgetInvalid = 48,
    Plonky3TranscriptMismatch = 49,
    Plonky3AirBindingMismatch = 50,
    Plonky3ProofMalformed = 51,
}

impl RecursiveCheckpointRejectReasonV2 {
    pub const ALL: [Self; 51] = [
        Self::UnsupportedVersion,
        Self::UnknownField,
        Self::StatementDigestMismatch,
        Self::PublicInputDigestMismatch,
        Self::PriorOutputMismatch,
        Self::OutputRootMismatch,
        Self::BackendUnsupported,
        Self::BackendClaimUnsupported,
        Self::ProofBytesEmpty,
        Self::ProofBytesTooLarge,
        Self::NovaPqAuthorityUnsupported,
        Self::NovaChainRootMismatch,
        Self::Plonky3CanonicalRangeMissing,
        Self::Plonky3DependsOnlyOnNova,
        Self::Plonky3UnauditedPromotion,
        Self::HybridCadenceMismatch,
        Self::EpochManifestIncomplete,
        Self::ProofSizeBudgetExceeded,
        Self::CelestiaPermanentStorageUnsupported,
        Self::IpfsPinningMissing,
        Self::ArchiveReplicationInsufficient,
        Self::ArchiveProviderReceiptMissing,
        Self::RetrievalAuditMissing,
        Self::RetrievalAuditFailed,
        Self::SnapshotBindingIncomplete,
        Self::PruningBeforeArchiveFinality,
        Self::ArchiveNodePruningUnsupported,
        Self::SidecarAuthoritative,
        Self::MeasurementMissing,
        Self::ChainTooShort,
        Self::ChainTooLong,
        Self::StepSkipped,
        Self::StepRepeated,
        Self::StepReordered,
        Self::WitnessUnavailable,
        Self::CanonicalAdmissionAttempt,
        Self::VerifiedCodecMissing,
        Self::MixedEra,
        Self::DaReadinessMissing,
        Self::PqInlineAnchorUnsupported,
        Self::PqCadenceDisabled,
        Self::PqCadenceInvalid,
        Self::PqLiveCadenceStageMismatch,
        Self::PqAnchorMissing,
        Self::PqAnchorDigestMismatch,
        Self::PqAnchorIncomplete,
        Self::RecursiveDocumentationIncomplete,
        Self::Plonky3SecurityBudgetInvalid,
        Self::Plonky3TranscriptMismatch,
        Self::Plonky3AirBindingMismatch,
        Self::Plonky3ProofMalformed,
    ];

    #[must_use]
    pub const fn code(self) -> u16 {
        self as u16
    }

    #[must_use]
    pub const fn canonical_bytes(self) -> [u8; 2] {
        self.code().to_le_bytes()
    }

    #[must_use]
    pub const fn from_code(code: u16) -> Option<Self> {
        match code {
            1 => Some(Self::UnsupportedVersion),
            2 => Some(Self::UnknownField),
            3 => Some(Self::StatementDigestMismatch),
            4 => Some(Self::PublicInputDigestMismatch),
            5 => Some(Self::PriorOutputMismatch),
            6 => Some(Self::OutputRootMismatch),
            7 => Some(Self::BackendUnsupported),
            8 => Some(Self::BackendClaimUnsupported),
            9 => Some(Self::ProofBytesEmpty),
            10 => Some(Self::ProofBytesTooLarge),
            11 => Some(Self::NovaPqAuthorityUnsupported),
            12 => Some(Self::NovaChainRootMismatch),
            13 => Some(Self::Plonky3CanonicalRangeMissing),
            14 => Some(Self::Plonky3DependsOnlyOnNova),
            15 => Some(Self::Plonky3UnauditedPromotion),
            16 => Some(Self::HybridCadenceMismatch),
            17 => Some(Self::EpochManifestIncomplete),
            18 => Some(Self::ProofSizeBudgetExceeded),
            19 => Some(Self::CelestiaPermanentStorageUnsupported),
            20 => Some(Self::IpfsPinningMissing),
            21 => Some(Self::ArchiveReplicationInsufficient),
            22 => Some(Self::ArchiveProviderReceiptMissing),
            23 => Some(Self::RetrievalAuditMissing),
            24 => Some(Self::RetrievalAuditFailed),
            25 => Some(Self::SnapshotBindingIncomplete),
            26 => Some(Self::PruningBeforeArchiveFinality),
            27 => Some(Self::ArchiveNodePruningUnsupported),
            28 => Some(Self::SidecarAuthoritative),
            29 => Some(Self::MeasurementMissing),
            30 => Some(Self::ChainTooShort),
            31 => Some(Self::ChainTooLong),
            32 => Some(Self::StepSkipped),
            33 => Some(Self::StepRepeated),
            34 => Some(Self::StepReordered),
            35 => Some(Self::WitnessUnavailable),
            36 => Some(Self::CanonicalAdmissionAttempt),
            37 => Some(Self::VerifiedCodecMissing),
            38 => Some(Self::MixedEra),
            39 => Some(Self::DaReadinessMissing),
            40 => Some(Self::PqInlineAnchorUnsupported),
            41 => Some(Self::PqCadenceDisabled),
            42 => Some(Self::PqCadenceInvalid),
            43 => Some(Self::PqLiveCadenceStageMismatch),
            44 => Some(Self::PqAnchorMissing),
            45 => Some(Self::PqAnchorDigestMismatch),
            46 => Some(Self::PqAnchorIncomplete),
            47 => Some(Self::RecursiveDocumentationIncomplete),
            48 => Some(Self::Plonky3SecurityBudgetInvalid),
            49 => Some(Self::Plonky3TranscriptMismatch),
            50 => Some(Self::Plonky3AirBindingMismatch),
            51 => Some(Self::Plonky3ProofMalformed),
            _ => None,
        }
    }

    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, CheckpointError> {
        let bytes: [u8; 2] = bytes
            .try_into()
            .map_err(|_| CheckpointError::RecursiveRejectCodec)?;
        Self::from_code(u16::from_le_bytes(bytes)).ok_or(CheckpointError::RecursiveRejectCodec)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedVersion => "unsupported version",
            Self::UnknownField => "unknown field",
            Self::StatementDigestMismatch => "statement digest mismatch",
            Self::PublicInputDigestMismatch => "public input digest mismatch",
            Self::PriorOutputMismatch => "prior output mismatch",
            Self::OutputRootMismatch => "output root mismatch",
            Self::BackendUnsupported => "backend unsupported",
            Self::BackendClaimUnsupported => "backend claim unsupported",
            Self::ProofBytesEmpty => "proof bytes empty",
            Self::ProofBytesTooLarge => "proof bytes too large",
            Self::NovaPqAuthorityUnsupported => "Nova PQ authority unsupported",
            Self::NovaChainRootMismatch => "Nova chain root mismatch",
            Self::Plonky3CanonicalRangeMissing => "Plonky3 canonical range missing",
            Self::Plonky3DependsOnlyOnNova => "Plonky3 depends only on Nova",
            Self::Plonky3UnauditedPromotion => "Plonky3 unaudited promotion",
            Self::HybridCadenceMismatch => "hybrid cadence mismatch",
            Self::EpochManifestIncomplete => "epoch manifest incomplete",
            Self::ProofSizeBudgetExceeded => "proof size budget exceeded",
            Self::CelestiaPermanentStorageUnsupported => "Celestia permanent storage unsupported",
            Self::IpfsPinningMissing => "IPFS pinning missing",
            Self::ArchiveReplicationInsufficient => "archive replication insufficient",
            Self::ArchiveProviderReceiptMissing => "archive provider receipt missing",
            Self::RetrievalAuditMissing => "retrieval audit missing",
            Self::RetrievalAuditFailed => "retrieval audit failed",
            Self::SnapshotBindingIncomplete => "snapshot binding incomplete",
            Self::PruningBeforeArchiveFinality => "pruning before archive finality",
            Self::ArchiveNodePruningUnsupported => "archive node pruning unsupported",
            Self::SidecarAuthoritative => "sidecar authoritative",
            Self::MeasurementMissing => "measurement missing",
            Self::ChainTooShort => "chain too short",
            Self::ChainTooLong => "chain too long",
            Self::StepSkipped => "step skipped",
            Self::StepRepeated => "step repeated",
            Self::StepReordered => "step reordered",
            Self::WitnessUnavailable => "witness unavailable",
            Self::CanonicalAdmissionAttempt => "canonical admission attempt",
            Self::VerifiedCodecMissing => "verified codec missing",
            Self::MixedEra => "mixed era",
            Self::DaReadinessMissing => "DA readiness missing",
            Self::PqInlineAnchorUnsupported => "PQ inline anchor unsupported",
            Self::PqCadenceDisabled => "PQ cadence disabled",
            Self::PqCadenceInvalid => "PQ cadence invalid",
            Self::PqLiveCadenceStageMismatch => "PQ live cadence stage mismatch",
            Self::PqAnchorMissing => "PQ anchor missing",
            Self::PqAnchorDigestMismatch => "PQ anchor digest mismatch",
            Self::PqAnchorIncomplete => "PQ anchor incomplete",
            Self::RecursiveDocumentationIncomplete => "recursive documentation incomplete",
            Self::Plonky3SecurityBudgetInvalid => "Plonky3 security budget invalid",
            Self::Plonky3TranscriptMismatch => "Plonky3 transcript mismatch",
            Self::Plonky3AirBindingMismatch => "Plonky3 AIR binding mismatch",
            Self::Plonky3ProofMalformed => "Plonky3 proof malformed",
        }
    }
}

impl fmt::Display for RecursiveCheckpointRejectReasonV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::RecursiveCheckpointRejectReasonV2 as Reason;

    #[test]
    fn test_reason_code_roundtrip() {
        for (index, reason) in Reason::ALL.into_iter().enumerate() {
            let expected = u16::try_from(index + 1).expect("51 entries fit u16");
            assert_eq!(reason.code(), expected);
            assert_eq!(Reason::from_code(expected), Some(reason));
            assert_eq!(
                Reason::decode_canonical(&reason.canonical_bytes()).ok(),
                Some(reason)
            );
            assert!(!reason.as_str().is_empty());
            assert_eq!(reason.to_string(), reason.as_str());
        }
    }

    #[test]
    fn test_reason_golden_vectors() {
        assert_eq!(Reason::UnsupportedVersion.canonical_bytes(), [0x01, 0x00]);
        assert_eq!(
            Reason::SnapshotBindingIncomplete.canonical_bytes(),
            [0x19, 0x00]
        );
        assert_eq!(
            Reason::RecursiveDocumentationIncomplete.canonical_bytes(),
            [0x2f, 0x00]
        );
    }

    #[test]
    fn test_reason_decoder_rejects() {
        for bytes in [&[][..], &[1][..], &[1, 0, 0][..], &[0, 0][..], &[52, 0][..]] {
            assert!(Reason::decode_canonical(bytes).is_err());
        }
    }
}
