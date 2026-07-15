//! Frozen resource profile and circuit schema for recursive checkpoint V2.

use z00z_crypto::{sha256_256_role, CheckpointSha256BlockStreamV2, CheckpointShaRole};

use super::{
    recursive_reject::RecursiveV2Error, recursive_semantics::RECURSIVE_FLOW_PAYLOAD_MAX_BYTES_V2,
    recursive_trace::SOURCE_RECORD_HASH_LABEL_V2,
};

/// Sole version of the recursive checkpoint circuit profile.
pub const RECURSIVE_CIRCUIT_PROFILE_VERSION_V2: u8 = 2;
/// Sole version of the recursive checkpoint circuit schema.
pub const RECURSIVE_CIRCUIT_SPEC_VERSION_V2: u8 = 2;
/// V2 content bound required by the authority-pinned checkpoint contract.
pub const RECURSIVE_V2_MAX_CONTENT_BYTES: u64 = 64 * 1024 * 1024;

/// Fixed committed source records emitted around one canonical execution:
/// begin, uniqueness precommit/challenge/net merge, hierarchy promotion, and
/// finalization.  SHA controls are derived separately and never enter the
/// source-record commitment.
const SOURCE_FIXED_RECORDS_V2: u64 = 6;
/// Fixed uniqueness and finalization controls outside row-derived records.
const UNIQUENESS_FIXED_STEPS_V2: u64 = 6;
const FINALIZATION_STEPS_V2: u64 = 4;
/// Bounded JMT controls per update in addition to the sibling path.
const JMT_FIXED_STEPS_PER_UPDATE_V2: u64 = 7;
/// One canonical replay identifier is a fixed 32-byte terminal ID.
pub(crate) const RECURSIVE_IDENTIFIER_BYTES_V2: u64 = 32;
/// The two private sorters each retain a resident ID buffer.
const RECURSIVE_IDENTIFIER_SORTERS_V2: u64 = 2;

/// Fixed resource limits that determine the recursive circuit shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursiveCircuitProfileV2 {
    max_rows: u32,
    max_inputs_per_row: u32,
    max_outputs_per_row: u32,
    max_spent: u32,
    max_outputs: u32,
    max_net_ops: u32,
    max_touched_tree_groups: u32,
    max_jmt_updates: u32,
    max_siblings: u32,
    max_leaf_bytes: u32,
    max_content_bytes: u64,
    max_sha_blocks: u64,
    max_typed_events: u32,
    max_cumulative_steps: u64,
    resident_buffer_bytes: u32,
    total_spool_bytes: u64,
    max_spool_runs: u32,
    spool_merge_fan_in: u32,
}

impl RecursiveCircuitProfileV2 {
    /// Create one fully bounded profile. Every zero or inconsistent cap rejects.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_rows: u32,
        max_inputs_per_row: u32,
        max_outputs_per_row: u32,
        max_spent: u32,
        max_outputs: u32,
        max_net_ops: u32,
        max_touched_tree_groups: u32,
        max_jmt_updates: u32,
        max_siblings: u32,
        max_leaf_bytes: u32,
        max_content_bytes: u64,
        max_sha_blocks: u64,
        max_typed_events: u32,
        max_cumulative_steps: u64,
        resident_buffer_bytes: u32,
        total_spool_bytes: u64,
        max_spool_runs: u32,
        spool_merge_fan_in: u32,
    ) -> Result<Self, RecursiveV2Error> {
        let nonzero_u32 = [
            max_rows,
            max_inputs_per_row,
            max_outputs_per_row,
            max_spent,
            max_outputs,
            max_net_ops,
            max_touched_tree_groups,
            max_jmt_updates,
            max_siblings,
            max_leaf_bytes,
            max_typed_events,
            resident_buffer_bytes,
            max_spool_runs,
            spool_merge_fan_in,
        ];
        if nonzero_u32.into_iter().any(|value| value == 0)
            || max_content_bytes == 0
            || max_sha_blocks == 0
            || max_cumulative_steps == 0
            || total_spool_bytes == 0
            || max_content_bytes > RECURSIVE_V2_MAX_CONTENT_BYTES
            || max_leaf_bytes < RECURSIVE_FLOW_PAYLOAD_MAX_BYTES_V2
            || u64::from(max_leaf_bytes) > max_content_bytes
            || u64::from(resident_buffer_bytes) > total_spool_bytes
            || max_spool_runs > spool_merge_fan_in
        {
            return Err(RecursiveV2Error::Limit);
        }
        Self::validate_spool_resources(
            max_spent,
            max_outputs,
            resident_buffer_bytes,
            max_content_bytes,
            total_spool_bytes,
            max_spool_runs,
            spool_merge_fan_in,
        )?;
        let max_source_records =
            Self::max_source_records(max_spent, max_outputs, max_leaf_bytes, max_content_bytes)?;
        let declared_sha_blocks =
            Self::sha_blocks_for_source_record_hashes(max_content_bytes, max_source_records)?;
        let e_max = Self::event_bound_from_parts(
            max_spent,
            max_outputs,
            max_net_ops,
            max_touched_tree_groups,
            max_jmt_updates,
            max_siblings,
            max_source_records,
            max_sha_blocks,
        )?;
        if max_sha_blocks < declared_sha_blocks
            || u64::from(max_typed_events) < e_max
            || max_cumulative_steps < e_max
        {
            return Err(RecursiveV2Error::Limit);
        }
        let max_rows_from_inputs = u64::from(max_rows)
            .checked_mul(u64::from(max_inputs_per_row))
            .ok_or(RecursiveV2Error::Overflow)?;
        let max_rows_from_outputs = u64::from(max_rows)
            .checked_mul(u64::from(max_outputs_per_row))
            .ok_or(RecursiveV2Error::Overflow)?;
        if u64::from(max_spent) > max_rows_from_inputs
            || u64::from(max_outputs) > max_rows_from_outputs
            || max_cumulative_steps < u64::from(max_typed_events)
        {
            return Err(RecursiveV2Error::Limit);
        }
        Ok(Self {
            max_rows,
            max_inputs_per_row,
            max_outputs_per_row,
            max_spent,
            max_outputs,
            max_net_ops,
            max_touched_tree_groups,
            max_jmt_updates,
            max_siblings,
            max_leaf_bytes,
            max_content_bytes,
            max_sha_blocks,
            max_typed_events,
            max_cumulative_steps,
            resident_buffer_bytes,
            total_spool_bytes,
            max_spool_runs,
            spool_merge_fan_in,
        })
    }

    /// A conservative profile suitable for the repository-local authority fixture.
    pub fn repository_fixture() -> Self {
        Self::new(
            1_000,
            16,
            16,
            16_000,
            16_000,
            16_000,
            1_000,
            1_000,
            256,
            64 * 1024,
            RECURSIVE_V2_MAX_CONTENT_BYTES,
            Self::sha_blocks_for_source_record_hashes(
                RECURSIVE_V2_MAX_CONTENT_BYTES,
                Self::max_source_records(16_000, 16_000, 64 * 1024, RECURSIVE_V2_MAX_CONTENT_BYTES)
                    .expect("fixed source-record cap is representable"),
            )
            .expect("fixed content cap has a valid SHA block count"),
            2_000_000,
            2_000_000,
            2 * 1024 * 1024,
            RECURSIVE_V2_MAX_CONTENT_BYTES * 2,
            16,
            16,
        )
        .expect("repository fixture profile is internally bounded")
    }

    /// Return `Q(L) = ceil((L + 9) / 64)` for a FIPS-authorized input size.
    pub fn sha_blocks_for_bytes(length: u64) -> Result<u64, RecursiveV2Error> {
        CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(length).map_err(Into::into)
    }

    /// Return SHA work for a role-framed stream with bounded source records.
    ///
    /// The count includes the frozen role DST, its outer length prefix, and a
    /// length prefix for every source record. This prevents raw-payload sizing
    /// from under-declaring the in-circuit SHA schedule.
    pub fn sha_blocks_for_role_parts(
        role: CheckpointShaRole,
        payload_bytes: u64,
        part_count: u64,
    ) -> Result<u64, RecursiveV2Error> {
        let framed_bytes =
            CheckpointSha256BlockStreamV2::framed_bytes_for_parts(role, payload_bytes, part_count)?;
        Self::sha_blocks_for_bytes(framed_bytes)
    }

    /// Conservative exact-framing bound for every individually expanded
    /// `source_record_hash` transcript.  Unlike the source trace digest, each
    /// source record starts a fresh role-framed FIPS stream and therefore owns
    /// its DST, label, length prefixes, and terminal padding block.
    pub fn sha_blocks_for_source_record_hashes(
        payload_bytes: u64,
        source_records: u64,
    ) -> Result<u64, RecursiveV2Error> {
        if source_records == 0 {
            return Err(RecursiveV2Error::Limit);
        }
        let label_bytes = u64::try_from(SOURCE_RECORD_HASH_LABEL_V2.len())
            .map_err(|_| RecursiveV2Error::Limit)?;
        let per_record_framing = CheckpointSha256BlockStreamV2::framed_bytes_for_parts(
            CheckpointShaRole::Trace,
            label_bytes,
            1,
        )?
        .checked_add(8)
        .ok_or(RecursiveV2Error::Overflow)?;
        // For each transcript Q(L)=ceil((L+9)/64).  Summing the integral
        // upper bound `(L+9+63)/64` gives a checked cap for any legal
        // distribution of the aggregate source bytes among the records.
        payload_bytes
            .checked_add(
                source_records
                    .checked_mul(
                        per_record_framing
                            .checked_add(72)
                            .ok_or(RecursiveV2Error::Overflow)?,
                    )
                    .ok_or(RecursiveV2Error::Overflow)?,
            )
            .map(|bytes| bytes / 64)
            .ok_or(RecursiveV2Error::Overflow)
    }

    /// Bound source records independently of the caller-selected event cap.
    pub fn max_source_records(
        max_spent: u32,
        max_outputs: u32,
        max_leaf_bytes: u32,
        max_content_bytes: u64,
    ) -> Result<u64, RecursiveV2Error> {
        if max_leaf_bytes < RECURSIVE_FLOW_PAYLOAD_MAX_BYTES_V2
            || u64::from(max_leaf_bytes) > max_content_bytes
        {
            return Err(RecursiveV2Error::Limit);
        }
        let jmt_records = max_content_bytes
            .checked_add(
                u64::from(max_leaf_bytes)
                    .checked_sub(1)
                    .ok_or(RecursiveV2Error::Limit)?,
            )
            .ok_or(RecursiveV2Error::Overflow)?
            / u64::from(max_leaf_bytes);
        SOURCE_FIXED_RECORDS_V2
            .checked_add(u64::from(max_spent))
            .and_then(|value| value.checked_add(u64::from(max_outputs)))
            .and_then(|value| value.checked_add(jmt_records))
            .and_then(|value| value.checked_add(u64::from(max_spent)))
            .and_then(|value| value.checked_add(u64::from(max_outputs)))
            .ok_or(RecursiveV2Error::Overflow)
    }

    /// Return the exact private disk bytes required for one sorted ID set.
    pub(crate) fn identifier_sort_bytes(ids: u32) -> Result<u64, RecursiveV2Error> {
        u64::from(ids)
            .checked_mul(RECURSIVE_IDENTIFIER_BYTES_V2)
            .ok_or(RecursiveV2Error::Overflow)
    }

    fn validate_spool_resources(
        max_spent: u32,
        max_outputs: u32,
        resident_buffer_bytes: u32,
        max_content_bytes: u64,
        total_spool_bytes: u64,
        max_spool_runs: u32,
        spool_merge_fan_in: u32,
    ) -> Result<(), RecursiveV2Error> {
        let spent_bytes = Self::identifier_sort_bytes(max_spent)?;
        let output_bytes = Self::identifier_sort_bytes(max_outputs)?;
        let required_spool_bytes = max_content_bytes
            .checked_add(spent_bytes)
            .and_then(|value| value.checked_add(output_bytes))
            .ok_or(RecursiveV2Error::Overflow)?;
        if total_spool_bytes < required_spool_bytes {
            return Err(RecursiveV2Error::Limit);
        }

        let resident_ids = u64::from(resident_buffer_bytes)
            / (RECURSIVE_IDENTIFIER_SORTERS_V2 * RECURSIVE_IDENTIFIER_BYTES_V2);
        if resident_ids == 0 {
            return Err(RecursiveV2Error::Limit);
        }
        let spent_runs = required_sort_runs(max_spent, resident_ids)?;
        let output_runs = required_sort_runs(max_outputs, resident_ids)?;
        let allowed_runs = u64::from(max_spool_runs);
        let merge_fan_in = u64::from(spool_merge_fan_in);
        if spent_runs > allowed_runs
            || output_runs > allowed_runs
            || spent_runs > merge_fan_in
            || output_runs > merge_fan_in
        {
            return Err(RecursiveV2Error::Limit);
        }
        Ok(())
    }

    /// Derive the finite worst-case control schedule from frozen bounds.
    #[allow(clippy::too_many_arguments)]
    pub fn event_bound_from_parts(
        max_spent: u32,
        max_outputs: u32,
        max_net_ops: u32,
        max_touched_tree_groups: u32,
        max_jmt_updates: u32,
        max_siblings: u32,
        max_source_records: u64,
        max_sha_blocks: u64,
    ) -> Result<u64, RecursiveV2Error> {
        let replay_ids = u64::from(max_spent)
            .checked_add(u64::from(max_outputs))
            .ok_or(RecursiveV2Error::Overflow)?;
        let hash_delimiters = max_source_records
            .checked_mul(2)
            .ok_or(RecursiveV2Error::Overflow)?;
        let uniqueness = replay_ids
            .checked_mul(2)
            .and_then(|value| value.checked_add(UNIQUENESS_FIXED_STEPS_V2))
            .ok_or(RecursiveV2Error::Overflow)?;
        let jmt_per_update = u64::from(max_siblings)
            .checked_add(JMT_FIXED_STEPS_PER_UPDATE_V2)
            .ok_or(RecursiveV2Error::Overflow)?;
        let jmt = u64::from(max_jmt_updates)
            .checked_mul(jmt_per_update)
            .ok_or(RecursiveV2Error::Overflow)?;
        max_source_records
            .checked_add(hash_delimiters)
            .and_then(|value| value.checked_add(max_sha_blocks))
            .and_then(|value| value.checked_add(uniqueness))
            .and_then(|value| value.checked_add(u64::from(max_net_ops)))
            .and_then(|value| value.checked_add(jmt))
            .and_then(|value| value.checked_add(u64::from(max_touched_tree_groups)))
            .and_then(|value| value.checked_add(FINALIZATION_STEPS_V2))
            .ok_or(RecursiveV2Error::Overflow)
    }

    /// Return the exact finite worst-case control schedule for this profile.
    pub fn e_max(&self) -> Result<u64, RecursiveV2Error> {
        Self::event_bound_from_parts(
            self.max_spent,
            self.max_outputs,
            self.max_net_ops,
            self.max_touched_tree_groups,
            self.max_jmt_updates,
            self.max_siblings,
            Self::max_source_records(
                self.max_spent,
                self.max_outputs,
                self.max_leaf_bytes,
                self.max_content_bytes,
            )?,
            self.max_sha_blocks,
        )
    }

    /// Encode all profile fields in their only frozen order.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 * 14 + 8 * 5);
        bytes.push(RECURSIVE_CIRCUIT_PROFILE_VERSION_V2);
        for value in [
            self.max_rows,
            self.max_inputs_per_row,
            self.max_outputs_per_row,
            self.max_spent,
            self.max_outputs,
            self.max_net_ops,
            self.max_touched_tree_groups,
            self.max_jmt_updates,
            self.max_siblings,
            self.max_leaf_bytes,
            self.max_typed_events,
            self.resident_buffer_bytes,
            self.max_spool_runs,
            self.spool_merge_fan_in,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in [
            self.max_content_bytes,
            self.max_sha_blocks,
            self.max_cumulative_steps,
            self.total_spool_bytes,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        sha256_256_role(
            CheckpointShaRole::Content,
            &[self.canonical_bytes().as_slice()],
        )
    }

    #[must_use]
    pub const fn max_typed_events(&self) -> u32 {
        self.max_typed_events
    }

    #[must_use]
    pub const fn max_spent(&self) -> u32 {
        self.max_spent
    }

    #[must_use]
    pub const fn max_outputs(&self) -> u32 {
        self.max_outputs
    }

    #[must_use]
    pub const fn max_leaf_bytes(&self) -> u32 {
        self.max_leaf_bytes
    }

    #[must_use]
    pub const fn max_content_bytes(&self) -> u64 {
        self.max_content_bytes
    }

    #[must_use]
    pub const fn max_sha_blocks(&self) -> u64 {
        self.max_sha_blocks
    }

    #[must_use]
    pub const fn max_cumulative_steps(&self) -> u64 {
        self.max_cumulative_steps
    }

    #[must_use]
    pub const fn resident_buffer_bytes(&self) -> u32 {
        self.resident_buffer_bytes
    }

    #[must_use]
    pub const fn total_spool_bytes(&self) -> u64 {
        self.total_spool_bytes
    }

    #[must_use]
    pub const fn max_spool_runs(&self) -> u32 {
        self.max_spool_runs
    }

    #[must_use]
    pub const fn spool_merge_fan_in(&self) -> u32 {
        self.spool_merge_fan_in
    }
}

fn required_sort_runs(ids: u32, resident_ids: u64) -> Result<u64, RecursiveV2Error> {
    u64::from(ids)
        .checked_add(resident_ids.checked_sub(1).ok_or(RecursiveV2Error::Limit)?)
        .map(|value| value / resident_ids)
        .ok_or(RecursiveV2Error::Overflow)
}

/// Hash-bound schema selection for the only recursive circuit V2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursiveCircuitSpecV2 {
    layout: u32,
    profile_digest: [u8; 32],
    digest: [u8; 32],
}

impl RecursiveCircuitSpecV2 {
    /// Bind the profile into one authority-visible circuit specification.
    pub(crate) fn new(
        layout: u32,
        profile: &RecursiveCircuitProfileV2,
    ) -> Result<Self, RecursiveV2Error> {
        let profile_digest = profile.digest();
        let layout_bytes = layout.to_le_bytes();
        let version = [RECURSIVE_CIRCUIT_SPEC_VERSION_V2];
        let shape_digest = super::recursive_v2::nova::circuit_shape_digest()?;
        let digest = sha256_256_role(
            CheckpointShaRole::Statement,
            &[&version, &layout_bytes, &profile_digest, &shape_digest],
        );
        Ok(Self {
            layout,
            profile_digest,
            digest,
        })
    }

    #[must_use]
    pub const fn layout(&self) -> u32 {
        self.layout
    }

    #[must_use]
    pub const fn profile_digest(&self) -> [u8; 32] {
        self.profile_digest
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[cfg(test)]
mod tests {
    use super::{RecursiveCircuitProfileV2, RECURSIVE_V2_MAX_CONTENT_BYTES};

    #[test]
    fn sha_block_count_has_fips_padding_boundaries() {
        assert_eq!(
            RecursiveCircuitProfileV2::sha_blocks_for_bytes(0).expect("zero-byte message"),
            1
        );
        assert_eq!(
            RecursiveCircuitProfileV2::sha_blocks_for_bytes(55).expect("one padded block"),
            1
        );
        assert_eq!(
            RecursiveCircuitProfileV2::sha_blocks_for_bytes(56).expect("two padded blocks"),
            2
        );
        assert_eq!(
            RecursiveCircuitProfileV2::sha_blocks_for_bytes(63).expect("two padded blocks"),
            2
        );
        assert_eq!(
            RecursiveCircuitProfileV2::sha_blocks_for_bytes(64).expect("two padded blocks"),
            2
        );
        assert_eq!(
            RecursiveCircuitProfileV2::sha_blocks_for_bytes(65).expect("two padded blocks"),
            2
        );
        assert!(RecursiveCircuitProfileV2::sha_blocks_for_bytes(1_u64 << 61).is_err());
        assert_eq!(
            RecursiveCircuitProfileV2::sha_blocks_for_role_parts(
                z00z_crypto::CheckpointShaRole::Trace,
                RECURSIVE_V2_MAX_CONTENT_BYTES,
                1,
            )
            .expect("framed 64 MiB trace"),
            1_048_578
        );
        let source_records = RecursiveCircuitProfileV2::max_source_records(
            16_000,
            16_000,
            64 * 1024,
            RECURSIVE_V2_MAX_CONTENT_BYTES,
        )
        .expect("fixture source record cap");
        assert!(
            RecursiveCircuitProfileV2::sha_blocks_for_source_record_hashes(
                RECURSIVE_V2_MAX_CONTENT_BYTES,
                source_records,
            )
            .expect("individual source hash cap")
                > 1_048_578,
            "separately padded source-record hashes must not reuse the whole-trace bound"
        );
        assert!(RecursiveCircuitProfileV2::sha_blocks_for_source_record_hashes(1, 0).is_err());
        assert!(
            RecursiveCircuitProfileV2::repository_fixture().max_content_bytes()
                == RECURSIVE_V2_MAX_CONTENT_BYTES
        );
    }

    #[test]
    fn profile_rejects_the_former_contradictory_event_and_step_caps() {
        let rejected = RecursiveCircuitProfileV2::new(
            1_000,
            16,
            16,
            16_000,
            16_000,
            16_000,
            1_000,
            1_000,
            256,
            64 * 1024,
            RECURSIVE_V2_MAX_CONTENT_BYTES,
            1_048_578,
            32_000,
            1_000_000,
            2 * 1024 * 1024,
            RECURSIVE_V2_MAX_CONTENT_BYTES,
            1_024,
            16,
        );
        assert!(rejected.is_err());

        let fixture = RecursiveCircuitProfileV2::repository_fixture();
        assert!(u64::from(fixture.max_typed_events()) >= fixture.e_max().expect("E_max"));
        assert!(fixture.max_cumulative_steps() >= fixture.e_max().expect("E_max"));

        assert!(RecursiveCircuitProfileV2::new(
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1_024, 100, 200, 200, 128, 2_048, 2, 1,
        )
        .is_err());
    }

    #[test]
    fn source_record_bound_covers_jmt_chunking_and_the_largest_flow_payload() {
        assert!(RecursiveCircuitProfileV2::max_source_records(1, 1, 398, 4 * 1024).is_err());
        assert_eq!(
            RecursiveCircuitProfileV2::max_source_records(1, 1, 512, 4 * 1024)
                .expect("eight JMT chunks plus fixed and replay records"),
            18
        );
    }

    #[test]
    fn profile_rejects_spool_that_omits_identifier_sorters() {
        let source_records = RecursiveCircuitProfileV2::max_source_records(1, 1, 512, 1_024)
            .expect("small source record cap");
        let sha_blocks =
            RecursiveCircuitProfileV2::sha_blocks_for_source_record_hashes(1_024, source_records)
                .expect("small SHA cap");
        let e_max = RecursiveCircuitProfileV2::event_bound_from_parts(
            1,
            1,
            1,
            1,
            1,
            1,
            source_records,
            sha_blocks,
        )
        .expect("small event bound");
        let build = |total_spool_bytes| {
            RecursiveCircuitProfileV2::new(
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                1,
                512,
                1_024,
                sha_blocks,
                u32::try_from(e_max).expect("small event bound"),
                e_max,
                64,
                total_spool_bytes,
                1,
                1,
            )
        };
        assert!(build(1_024).is_err());
        assert!(build(1_024 + 2 * super::RECURSIVE_IDENTIFIER_BYTES_V2).is_ok());
    }
}
