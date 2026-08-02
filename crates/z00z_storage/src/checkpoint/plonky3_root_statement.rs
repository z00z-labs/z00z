//! Canonical public statement propagated through checkpoint recursion.

use p3_circuit::CircuitError;
use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;

use super::{PLONKY3_COMMON_CAP_ROOTS_V2, PLONKY3_MMCS_DIGEST_ELEMS_V2};

pub(super) const ROOT_STATEMENT_NPO_ID_V2: &str = "z00z/plonky3/root-statement/v3";
pub(super) const ROOT_STATEMENT_DIGEST_LIMBS_V2: usize = 16;
pub(super) const ROOT_STATEMENT_DIGEST_COUNT_V2: usize = 26;
pub(super) const ROOT_STATEMENT_COMMITMENT_FIELDS_V2: usize = 8;
pub(super) const ROOT_STATEMENT_COMMITMENT_INDEX_V2: usize =
    1 + ROOT_STATEMENT_DIGEST_LIMBS_V2 * ROOT_STATEMENT_DIGEST_COUNT_V2;
pub(super) const ROOT_COMMON_CAP_FIELDS_V2: usize =
    PLONKY3_COMMON_CAP_ROOTS_V2 * PLONKY3_MMCS_DIGEST_ELEMS_V2;
pub(super) const ROOT_COMMON_CAP_INDEX_V2: usize =
    ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2;
pub(super) const ROOT_STATEMENT_REPLICA_INDEX_V2: usize =
    ROOT_COMMON_CAP_INDEX_V2 + ROOT_COMMON_CAP_FIELDS_V2;
pub(super) const ROOT_STATEMENT_START_INDEX_V2: usize = ROOT_STATEMENT_REPLICA_INDEX_V2 + 1;
pub(super) const ROOT_STATEMENT_COUNT_INDEX_V2: usize = ROOT_STATEMENT_START_INDEX_V2 + 1;
pub(super) const ROOT_STATEMENT_TOTAL_INDEX_V2: usize = ROOT_STATEMENT_COUNT_INDEX_V2 + 1;
pub(super) const ROOT_STATEMENT_SEMANTIC_INDEX_V2: usize = ROOT_STATEMENT_TOTAL_INDEX_V2 + 1;
pub(super) const ROOT_STATEMENT_HEIGHT_LIMBS_V2: usize = 4;
pub(super) const ROOT_STATEMENT_RANGE_START_HEIGHT_INDEX_V2: usize =
    ROOT_STATEMENT_SEMANTIC_INDEX_V2;
pub(super) const ROOT_STATEMENT_RANGE_END_HEIGHT_INDEX_V2: usize =
    ROOT_STATEMENT_RANGE_START_HEIGHT_INDEX_V2 + ROOT_STATEMENT_HEIGHT_LIMBS_V2;
pub(super) const ROOT_STATEMENT_FIRST_EPOCH_INDEX_V2: usize =
    ROOT_STATEMENT_RANGE_END_HEIGHT_INDEX_V2 + ROOT_STATEMENT_HEIGHT_LIMBS_V2;
pub(super) const ROOT_STATEMENT_LAST_EPOCH_INDEX_V2: usize =
    ROOT_STATEMENT_FIRST_EPOCH_INDEX_V2 + ROOT_STATEMENT_HEIGHT_LIMBS_V2;
pub(super) const ROOT_STATEMENT_CADENCE_INDEX_V2: usize =
    ROOT_STATEMENT_LAST_EPOCH_INDEX_V2 + ROOT_STATEMENT_HEIGHT_LIMBS_V2;
pub(super) const ROOT_STATEMENT_CADENCE_LIMBS_V2: usize = ROOT_STATEMENT_HEIGHT_LIMBS_V2;
pub(super) const ROOT_STATEMENT_PARAMETER_GENERATION_INDEX_V2: usize =
    ROOT_STATEMENT_CADENCE_INDEX_V2 + ROOT_STATEMENT_CADENCE_LIMBS_V2;
pub(super) const ROOT_STATEMENT_PARAMETER_GENERATION_LIMBS_V2: usize = 2;
pub(super) const ROOT_STATEMENT_RUNTIME_PROFILE_GENERATION_INDEX_V2: usize =
    ROOT_STATEMENT_PARAMETER_GENERATION_INDEX_V2 + ROOT_STATEMENT_PARAMETER_GENERATION_LIMBS_V2;
pub(super) const ROOT_STATEMENT_HISTORY_COMPOSITION_RULE_INDEX_V2: usize =
    ROOT_STATEMENT_RUNTIME_PROFILE_GENERATION_INDEX_V2 + 1;
pub(super) const ROOT_STATEMENT_HISTORY_PER_PROOF_ERROR_INDEX_V2: usize =
    ROOT_STATEMENT_HISTORY_COMPOSITION_RULE_INDEX_V2 + 1;
pub(super) const ROOT_STATEMENT_HISTORY_INHERITED_ERROR_INDEX_V2: usize =
    ROOT_STATEMENT_HISTORY_PER_PROOF_ERROR_INDEX_V2 + 1;
pub(super) const ROOT_STATEMENT_HISTORY_CUMULATIVE_ERROR_INDEX_V2: usize =
    ROOT_STATEMENT_HISTORY_INHERITED_ERROR_INDEX_V2 + 1;
pub(super) const ROOT_STATEMENT_HISTORY_MINIMUM_RESIDUAL_INDEX_V2: usize =
    ROOT_STATEMENT_HISTORY_CUMULATIVE_ERROR_INDEX_V2 + 1;
pub(super) const ROOT_STATEMENT_HISTORY_SECURITY_FIELDS_V2: usize = 6;
pub(super) const ROOT_STATEMENT_SEMANTIC_FIELDS_V2: usize = ROOT_STATEMENT_HEIGHT_LIMBS_V2 * 4
    + ROOT_STATEMENT_CADENCE_LIMBS_V2
    + ROOT_STATEMENT_PARAMETER_GENERATION_LIMBS_V2
    + ROOT_STATEMENT_HISTORY_SECURITY_FIELDS_V2;
pub(super) const ROOT_STATEMENT_FIELDS_V2: usize =
    ROOT_STATEMENT_SEMANTIC_INDEX_V2 + ROOT_STATEMENT_SEMANTIC_FIELDS_V2;

/// Fixed public statement propagated by every recursive layer.
///
/// Digests are encoded as sixteen little-endian `u16` limbs so every value is
/// canonical in KoalaBear. Eight native KoalaBear fields bind the ordered
/// leaf/subtree commitment. The next 128 fields carry the proof's complete
/// ordered preprocessed-common cap without a lossy intermediary: every parent
/// recursion circuit constrains those fields directly to the actual common
/// targets consumed by its pinned child verifier, while the outer verifier
/// checks and authority-pins the final root common. The trailing scalar fields
/// bind a physical replica or fold ordinal, an exact contiguous range,
/// start/end heights, first/last epoch, cadence, parameter generation, runtime
/// profile generation, and the rolling-history security-composition scalars.
/// Aggregation can therefore neither substitute a circuit shape nor duplicate,
/// omit, reorder, or silently rotate a proved range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RootStatementV2 {
    values: [KoalaBear; ROOT_STATEMENT_FIELDS_V2],
}

impl RootStatementV2 {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn leaf_with_semantics(
        digests: [[u8; 32]; ROOT_STATEMENT_DIGEST_COUNT_V2],
        commitment: [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
        replica: u8,
        start: u16,
        total: u16,
        semantic_fields: [KoalaBear; ROOT_STATEMENT_SEMANTIC_FIELDS_V2],
    ) -> Result<Self, CircuitError> {
        if digests[..5].contains(&[0; 32]) || total == 0 || start >= total {
            return Err(CircuitError::InvalidPreprocessedValues);
        }
        let mut values = [KoalaBear::ZERO; ROOT_STATEMENT_FIELDS_V2];
        values[0] = KoalaBear::from_u8(2);
        let mut cursor = 1;
        for digest in digests {
            for limb in digest.chunks_exact(2) {
                values[cursor] = KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]));
                cursor += 1;
            }
        }
        values[ROOT_STATEMENT_COMMITMENT_INDEX_V2
            ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
            .copy_from_slice(&commitment);
        values[ROOT_STATEMENT_REPLICA_INDEX_V2] = KoalaBear::from_u8(replica);
        values[ROOT_STATEMENT_START_INDEX_V2] = KoalaBear::from_u16(start);
        values[ROOT_STATEMENT_COUNT_INDEX_V2] = KoalaBear::ONE;
        values[ROOT_STATEMENT_TOTAL_INDEX_V2] = KoalaBear::from_u16(total);
        values[ROOT_STATEMENT_SEMANTIC_INDEX_V2
            ..ROOT_STATEMENT_SEMANTIC_INDEX_V2 + ROOT_STATEMENT_SEMANTIC_FIELDS_V2]
            .copy_from_slice(&semantic_fields);
        Ok(Self { values })
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn leaf(
        statement_digest: [u8; 32],
        leaf_manifest_digest: [u8; 32],
        parameter_digest: [u8; 32],
        security_digest: [u8; 32],
        verifier_bundle_digest: [u8; 32],
        commitment: [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
        replica: u8,
        start: u16,
        total: u16,
    ) -> Result<Self, CircuitError> {
        let mut digests = [[0_u8; 32]; ROOT_STATEMENT_DIGEST_COUNT_V2];
        digests[..5].copy_from_slice(&[
            statement_digest,
            leaf_manifest_digest,
            parameter_digest,
            security_digest,
            verifier_bundle_digest,
        ]);
        Self::leaf_with_semantics(
            digests,
            commitment,
            replica,
            start,
            total,
            [KoalaBear::ZERO; ROOT_STATEMENT_SEMANTIC_FIELDS_V2],
        )
    }

    pub(super) fn root(
        &self,
        commitment: [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
    ) -> Self {
        let mut root = self.clone();
        root.values[ROOT_STATEMENT_COMMITMENT_INDEX_V2
            ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
            .copy_from_slice(&commitment);
        root.values[ROOT_STATEMENT_START_INDEX_V2] = KoalaBear::ZERO;
        root.values[ROOT_STATEMENT_COUNT_INDEX_V2] = root.values[ROOT_STATEMENT_TOTAL_INDEX_V2];
        root
    }

    /// Bind the proof's declared complete ordered preprocessed-common cap.
    ///
    /// The field is deliberately value-only: common-data construction commits
    /// circuit shape and preprocessing, not these public statement values.
    /// Base and aggregation provers can therefore derive the common first and
    /// then supply the exact declaration without a digest cycle.
    #[must_use]
    pub(super) fn with_common_cap(
        &self,
        common_cap: [KoalaBear; ROOT_COMMON_CAP_FIELDS_V2],
    ) -> Self {
        let mut bound = self.clone();
        bound.values
            [ROOT_COMMON_CAP_INDEX_V2..ROOT_COMMON_CAP_INDEX_V2 + ROOT_COMMON_CAP_FIELDS_V2]
            .copy_from_slice(&common_cap);
        bound
    }

    /// Convert a complete physical-replica root into one ordered replica-fold
    /// root. Callers must first prove the exact input ordinal pair and derive
    /// `commitment` through the corresponding domain-separated fold hash.
    pub(super) fn replica_fold_root(
        &self,
        commitment: [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2],
        fold_ordinal: u8,
    ) -> Result<Self, CircuitError> {
        if self.values[ROOT_STATEMENT_START_INDEX_V2] != KoalaBear::ZERO
            || self.values[ROOT_STATEMENT_COUNT_INDEX_V2]
                != self.values[ROOT_STATEMENT_TOTAL_INDEX_V2]
        {
            return Err(CircuitError::InvalidPreprocessedValues);
        }
        let mut root = self.clone();
        root.values[ROOT_STATEMENT_COMMITMENT_INDEX_V2
            ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
            .copy_from_slice(&commitment);
        root.values[ROOT_STATEMENT_REPLICA_INDEX_V2] = KoalaBear::from_u8(fold_ordinal);
        Ok(root)
    }

    #[must_use = "the recursive root commitment must be bound or verified"]
    pub(super) fn commitment(&self) -> [KoalaBear; ROOT_STATEMENT_COMMITMENT_FIELDS_V2] {
        self.values[ROOT_STATEMENT_COMMITMENT_INDEX_V2
            ..ROOT_STATEMENT_COMMITMENT_INDEX_V2 + ROOT_STATEMENT_COMMITMENT_FIELDS_V2]
            .try_into()
            .expect("fixed root-statement commitment width")
    }

    #[must_use = "root statement values must be consumed by the recursive verifier"]
    pub(super) const fn values(&self) -> &[KoalaBear; ROOT_STATEMENT_FIELDS_V2] {
        &self.values
    }
}
