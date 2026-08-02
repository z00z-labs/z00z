//! Canonical slot partition for bounded uniqueness Batch-STARK proofs.
//!
//! The two slices are part of the direct proof statement. A caller cannot
//! select an arbitrary subrange: every bounded chunk has exactly the lower
//! slice and, only for chunks larger than four transitions, the upper slice.

use crate::CheckpointError;

pub(super) const UNIQUENESS_SLICE_WIDTH_V2: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct EpochUniquenessSliceV2 {
    start: usize,
    len: usize,
}

impl EpochUniquenessSliceV2 {
    /// Descriptor used by non-sliced groups whose public arrays cover the
    /// whole chunk. It is not accepted by the uniqueness proof codec for an
    /// eight-transition chunk.
    pub(super) fn full(binding_count: usize) -> Result<Self, CheckpointError> {
        if binding_count == 0 {
            return Err(CheckpointError::Canonical);
        }
        Ok(Self {
            start: 0,
            len: binding_count,
        })
    }

    pub(super) fn canonical(binding_count: usize) -> Result<Vec<Self>, CheckpointError> {
        if binding_count == 0 {
            return Err(CheckpointError::Canonical);
        }
        let lower_len = binding_count.min(UNIQUENESS_SLICE_WIDTH_V2);
        let mut slices = vec![Self {
            start: 0,
            len: lower_len,
        }];
        if binding_count > UNIQUENESS_SLICE_WIDTH_V2 {
            slices.push(Self {
                start: UNIQUENESS_SLICE_WIDTH_V2,
                len: binding_count - UNIQUENESS_SLICE_WIDTH_V2,
            });
        }
        Ok(slices)
    }

    pub(super) fn from_wire(
        binding_count: usize,
        start: usize,
        len: usize,
    ) -> Result<Self, CheckpointError> {
        Self::canonical(binding_count)?
            .into_iter()
            .find(|slice| slice.start == start && slice.len == len)
            .ok_or(CheckpointError::Canonical)
    }

    pub(super) fn validate_partition(
        binding_count: usize,
        slices: &[Self],
    ) -> Result<(), CheckpointError> {
        if slices != Self::canonical(binding_count)?.as_slice() {
            return Err(CheckpointError::Canonical);
        }
        Ok(())
    }

    #[must_use]
    pub(super) const fn start(self) -> usize {
        self.start
    }

    #[must_use]
    pub(super) const fn len(self) -> usize {
        self.len
    }

    pub(super) fn end(self) -> Result<usize, CheckpointError> {
        self.start
            .checked_add(self.len)
            .ok_or(CheckpointError::Overflow)
    }

    pub(super) fn local_slot(self, global_slot: usize) -> Result<usize, CheckpointError> {
        let end = self.end()?;
        if global_slot < self.start || global_slot >= end {
            return Err(CheckpointError::Canonical);
        }
        Ok(global_slot - self.start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_partition_is_exact_and_fail_closed() {
        assert_eq!(
            EpochUniquenessSliceV2::canonical(8).expect("canonical slices"),
            vec![
                EpochUniquenessSliceV2 { start: 0, len: 4 },
                EpochUniquenessSliceV2 { start: 4, len: 4 },
            ]
        );
        assert_eq!(
            EpochUniquenessSliceV2::canonical(4).expect("canonical slices"),
            vec![EpochUniquenessSliceV2 { start: 0, len: 4 }]
        );
        assert!(EpochUniquenessSliceV2::from_wire(8, 1, 4).is_err());
        assert!(EpochUniquenessSliceV2::from_wire(8, 0, 8).is_err());
        assert!(EpochUniquenessSliceV2::validate_partition(
            8,
            &[
                EpochUniquenessSliceV2 { start: 4, len: 4 },
                EpochUniquenessSliceV2 { start: 0, len: 4 },
            ],
        )
        .is_err());
    }
}
