//! Frozen resource profile and circuit schema for recursive checkpoint V2.

use z00z_crypto::{sha256_256_role, CheckpointSha256BlockStreamV2, CheckpointShaRole};

use super::{
    recursive_reject::RecursiveV2Error,
    recursive_semantics::{
        RECURSIVE_FLOW_PAYLOAD_MAX_BYTES_V2, UNIQUENESS_CHALLENGE_BITS_V2,
        UNIQUENESS_ROW_FACTOR_DEGREE_V2, UNIQUENESS_RO_QUERY_LOG2_V2,
        UNIQUENESS_SEMANTIC_ROW_BYTES_V2, UNIQUENESS_SEMANTIC_ROW_LIMBS_V2,
    },
    recursive_trace::{SOURCE_RECORD_HASH_LABEL_V2, TRACE_EVENT_HEADER_BYTES_V2},
};

/// Sole version of the recursive checkpoint circuit profile.
pub const RECURSIVE_CIRCUIT_PROFILE_VERSION_V2: u8 = 3;
/// Sole version of the recursive checkpoint circuit schema.
pub const RECURSIVE_CIRCUIT_SPEC_VERSION_V2: u8 = 6;
/// V2 content bound required by the authority-pinned checkpoint contract.
pub const RECURSIVE_V2_MAX_CONTENT_BYTES: u64 = 64 * 1024 * 1024;

/// Fixed committed source records emitted around one canonical execution:
/// begin, uniqueness precommit/challenge/net merge, hierarchy promotion, and
/// finalization.  SHA controls are derived separately and never enter the
/// source-record commitment.
const SOURCE_FIXED_RECORDS_V2: u64 = 6;
/// Fixed semantic-finalization micro-operations that remain outside the
/// canonical source-record expansion.
const FINALIZATION_STEPS_V2: u64 = 4;
/// Bounded JMT controls per update in addition to the sibling path.
const JMT_FIXED_STEPS_PER_UPDATE_V2: u64 = 7;
/// One canonical replay uniqueness row commits path, terminal and leaf value.
pub(crate) const RECURSIVE_IDENTIFIER_BYTES_V2: u64 = UNIQUENESS_SEMANTIC_ROW_BYTES_V2 as u64;
/// The two private sorters each retain a resident semantic-row buffer.
const RECURSIVE_IDENTIFIER_SORTERS_V2: u64 = 2;
/// One source record is held both as its exact spool bytes and as its decoded
/// event while the evaluator invokes its immediately derived controls.
const NATIVE_EVALUATOR_SOURCE_RECORD_COPIES_V2: u64 = 2;
/// One begin/end pair for the global trace hash, four list commitments,
/// twelve uniqueness transcripts, and two SettlementV2 root transcripts.
/// Source-record hash delimiters are counted separately because their number
/// is profile-dependent.
// BEGIN/END controls for the whole-trace stream, four uniqueness-list jobs,
// twelve uniqueness transcript jobs, and the two SettlementV2 root jobs.
const FIXED_DERIVED_HASH_DELIMITERS_V2: u64 = 2 * (1 + 4 + 14);
/// Every canonical source chunk has one authenticated memory write and one
/// byte-equal TraceChunk control.
const DERIVED_BYTE_CONTROLS_PER_CHUNK_V2: u64 = 2;
/// Exact raw part geometry for the two U transcripts.
const UNIQUENESS_PRECOMMIT_PART_BYTES_V2: u64 = 32 + 1 + 4 + 32 + 32;
const UNIQUENESS_PRECOMMIT_PART_COUNT_V2: u64 = 5;
/// Exact raw part geometry for each of the eight challenge transcripts.
const UNIQUENESS_CHALLENGE_PART_BYTES_V2: u64 = 32 + 32 + 1 + 1 + 1;
const UNIQUENESS_CHALLENGE_PART_COUNT_V2: u64 = 5;
const UNIQUENESS_COUNTS_PART_BYTES_V2: u64 = 1 + 8 * 8 + 17 * 8;
const UNIQUENESS_COUNTS_PART_COUNT_V2: u64 = 10;
const UNIQUENESS_CONTEXT_PART_BYTES_V2: u64 = 1 + 2 * 8 + 4 + 8 + 1 + 14 * 32;
const UNIQUENESS_CONTEXT_PART_COUNT_V2: u64 = 20;
const SETTLEMENT_ROOT_PART_BYTES_V2: u64 = 1 + 4 + 32 + 32;
const SETTLEMENT_ROOT_PART_COUNT_V2: u64 = 4;

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
        // This derives the native evaluator's bounded resident allocation from
        // the same frozen caps that are committed below.  It is deliberately
        // separate from disk spool feasibility: the decoded source record and
        // verified JMT envelope are live simultaneously during evaluation.
        Self::native_evaluator_resident_bytes_from_parts(
            max_leaf_bytes,
            max_content_bytes,
            resident_buffer_bytes,
        )?;
        let max_source_records =
            Self::max_source_records(max_spent, max_outputs, max_leaf_bytes, max_content_bytes)?;
        let declared_sha_blocks = Self::sha_blocks_for_complete_trace(
            max_content_bytes,
            max_source_records,
            max_spent,
            max_outputs,
        )?;
        let e_max = Self::event_bound_from_parts(
            max_net_ops,
            max_touched_tree_groups,
            max_jmt_updates,
            max_siblings,
            max_source_records,
            max_sha_blocks,
            max_content_bytes,
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
        let max_spent = 16_000;
        let max_outputs = 16_000;
        let max_leaf_bytes = 64 * 1024;
        let max_source_records = Self::max_source_records(
            max_spent,
            max_outputs,
            max_leaf_bytes,
            RECURSIVE_V2_MAX_CONTENT_BYTES,
        )
        .expect("fixed source-record cap is representable");
        let max_sha_blocks = Self::sha_blocks_for_complete_trace(
            RECURSIVE_V2_MAX_CONTENT_BYTES,
            max_source_records,
            max_spent,
            max_outputs,
        )
        .expect("fixed content cap has a valid complete SHA schedule");
        let e_max = Self::event_bound_from_parts(
            16_000,
            1_000,
            1_000,
            256,
            max_source_records,
            max_sha_blocks,
            RECURSIVE_V2_MAX_CONTENT_BYTES,
        )
        .expect("repository fixture has a finite event bound");
        let max_typed_events =
            u32::try_from(e_max).expect("repository fixture event bound fits u32");
        Self::new(
            1_000,
            16,
            16,
            max_spent,
            max_outputs,
            16_000,
            1_000,
            1_000,
            256,
            max_leaf_bytes,
            RECURSIVE_V2_MAX_CONTENT_BYTES,
            max_sha_blocks,
            max_typed_events,
            e_max,
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

    /// Bound every SHA compression event in the canonical expanded trace.
    ///
    /// This is the sole profile owner for source-local hashes, the whole-trace
    /// commitment, four ordered-list commitments, two U transcripts, and the
    /// eight challenge transcripts, and both SettlementV2 roots. Keeping these
    /// terms together prevents a newly authenticated transcript from silently
    /// escaping `max_sha_blocks`.
    pub fn sha_blocks_for_complete_trace(
        max_content_bytes: u64,
        max_source_records: u64,
        max_spent: u32,
        max_outputs: u32,
    ) -> Result<u64, RecursiveV2Error> {
        let source_local =
            Self::sha_blocks_for_source_record_hashes(max_content_bytes, max_source_records)?;
        let global = Self::sha_blocks_for_role_parts(
            CheckpointShaRole::Trace,
            max_content_bytes,
            max_source_records,
        )?;

        let list_blocks = |role, count: u32| {
            let part_bytes = u64::from(count)
                .checked_mul(RECURSIVE_IDENTIFIER_BYTES_V2)
                .and_then(|value| value.checked_add(4))
                .ok_or(RecursiveV2Error::Overflow)?;
            let part_count = u64::from(count)
                .checked_add(1)
                .ok_or(RecursiveV2Error::Overflow)?;
            Self::sha_blocks_for_role_parts(role, part_bytes, part_count)
        };
        let ordered_lists = [
            (CheckpointShaRole::SpentOriginalIds, max_spent),
            (CheckpointShaRole::OutputOriginalIds, max_outputs),
            (CheckpointShaRole::SpentSortedIds, max_spent),
            (CheckpointShaRole::OutputSortedIds, max_outputs),
        ]
        .into_iter()
        .try_fold(0_u64, |total, (role, count)| {
            total
                .checked_add(list_blocks(role, count)?)
                .ok_or(RecursiveV2Error::Overflow)
        })?;
        let uniqueness_precommits = Self::sha_blocks_for_role_parts(
            CheckpointShaRole::IdPrecommit,
            UNIQUENESS_PRECOMMIT_PART_BYTES_V2,
            UNIQUENESS_PRECOMMIT_PART_COUNT_V2,
        )?
        .checked_mul(2)
        .ok_or(RecursiveV2Error::Overflow)?;
        let uniqueness_challenges = Self::sha_blocks_for_role_parts(
            CheckpointShaRole::IdChallenge,
            UNIQUENESS_CHALLENGE_PART_BYTES_V2,
            UNIQUENESS_CHALLENGE_PART_COUNT_V2,
        )?
        .checked_mul(8)
        .ok_or(RecursiveV2Error::Overflow)?;
        let uniqueness_counts = Self::sha_blocks_for_role_parts(
            CheckpointShaRole::UniquenessCounts,
            UNIQUENESS_COUNTS_PART_BYTES_V2,
            UNIQUENESS_COUNTS_PART_COUNT_V2,
        )?;
        let uniqueness_context = Self::sha_blocks_for_role_parts(
            CheckpointShaRole::UniquenessContext,
            UNIQUENESS_CONTEXT_PART_BYTES_V2,
            UNIQUENESS_CONTEXT_PART_COUNT_V2,
        )?;
        let settlement_roots = Self::sha_blocks_for_role_parts(
            CheckpointShaRole::SettlementRoot,
            SETTLEMENT_ROOT_PART_BYTES_V2,
            SETTLEMENT_ROOT_PART_COUNT_V2,
        )?
        .checked_mul(2)
        .ok_or(RecursiveV2Error::Overflow)?;

        source_local
            .checked_add(global)
            .and_then(|value| value.checked_add(ordered_lists))
            .and_then(|value| value.checked_add(uniqueness_precommits))
            .and_then(|value| value.checked_add(uniqueness_challenges))
            .and_then(|value| value.checked_add(uniqueness_counts))
            .and_then(|value| value.checked_add(uniqueness_context))
            .and_then(|value| value.checked_add(settlement_roots))
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
        let replay_ids = u64::from(max_spent)
            .checked_add(u64::from(max_outputs))
            .ok_or(RecursiveV2Error::Overflow)?;
        // Each identifier appears once in replay, twice in the commit
        // original/sorted rows, twice in the product original/sorted rows,
        // and once in CommitTypedEvent.
        let identifier_records = replay_ids
            .checked_mul(6)
            .ok_or(RecursiveV2Error::Overflow)?;
        let semantic_minimum = SOURCE_FIXED_RECORDS_V2
            .checked_add(identifier_records)
            .and_then(|value| value.checked_add(4))
            .and_then(|value| value.checked_add(1))
            .ok_or(RecursiveV2Error::Overflow)?;
        // JMT is now represented by many bounded micro-op records rather than
        // max-leaf-sized envelope fragments. Every canonical source record
        // consumes at least its fixed header in the same max-content spool, so
        // this byte-derived cap covers every possible micro-op segmentation
        // without assuming a minimum JMT payload size.
        let content_record_bound = max_content_bytes
            / u64::try_from(TRACE_EVENT_HEADER_BYTES_V2).map_err(|_| RecursiveV2Error::Limit)?;
        if content_record_bound < semantic_minimum {
            return Err(RecursiveV2Error::Limit);
        }
        Ok(content_record_bound)
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
        max_net_ops: u32,
        max_touched_tree_groups: u32,
        max_jmt_updates: u32,
        max_siblings: u32,
        max_source_records: u64,
        max_sha_blocks: u64,
        max_content_bytes: u64,
    ) -> Result<u64, RecursiveV2Error> {
        let hash_delimiters = max_source_records
            .checked_mul(2)
            .and_then(|value| value.checked_add(FIXED_DERIVED_HASH_DELIMITERS_V2))
            .ok_or(RecursiveV2Error::Overflow)?;
        let canonical_chunks = max_content_bytes
            .checked_add(
                max_source_records
                    .checked_mul(63)
                    .ok_or(RecursiveV2Error::Overflow)?,
            )
            .map(|bytes| bytes / 64)
            .ok_or(RecursiveV2Error::Overflow)?;
        let byte_controls = canonical_chunks
            .checked_mul(DERIVED_BYTE_CONTROLS_PER_CHUNK_V2)
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
            .and_then(|value| value.checked_add(byte_controls))
            .and_then(|value| value.checked_add(u64::from(max_net_ops)))
            .and_then(|value| value.checked_add(jmt))
            .and_then(|value| value.checked_add(u64::from(max_touched_tree_groups)))
            .and_then(|value| value.checked_add(FINALIZATION_STEPS_V2))
            .ok_or(RecursiveV2Error::Overflow)
    }

    /// Return the exact finite worst-case control schedule for this profile.
    pub fn e_max(&self) -> Result<u64, RecursiveV2Error> {
        Self::event_bound_from_parts(
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
            self.max_content_bytes,
        )
    }

    /// Encode all profile fields in their only frozen order.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 * 14 + 8 * 6);
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
            self.native_evaluator_resident_bytes()
                .expect("validated profile has a representable native evaluator bound"),
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

    /// Return the accounted peak for the native evaluator's bounded resident
    /// data structures.
    ///
    /// The bound includes the complete canonical JMT envelope accumulated for
    /// strict decode, the raw and decoded copies of the one current source
    /// record, and both external-sorter resident buffers. It is a profile
    /// commitment, not an authority operating-budget measurement.
    pub fn native_evaluator_resident_bytes(&self) -> Result<u64, RecursiveV2Error> {
        Self::native_evaluator_resident_bytes_from_parts(
            self.max_leaf_bytes,
            self.max_content_bytes,
            self.resident_buffer_bytes,
        )
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

    fn native_evaluator_resident_bytes_from_parts(
        max_leaf_bytes: u32,
        max_content_bytes: u64,
        resident_buffer_bytes: u32,
    ) -> Result<u64, RecursiveV2Error> {
        let source_record_bytes = u64::try_from(TRACE_EVENT_HEADER_BYTES_V2)
            .map_err(|_| RecursiveV2Error::Limit)?
            .checked_add(u64::from(max_leaf_bytes))
            .ok_or(RecursiveV2Error::Overflow)?;
        max_content_bytes
            .checked_add(
                source_record_bytes
                    .checked_mul(NATIVE_EVALUATOR_SOURCE_RECORD_COPIES_V2)
                    .ok_or(RecursiveV2Error::Overflow)?,
            )
            .and_then(|value| value.checked_add(u64::from(resident_buffer_bytes)))
            .ok_or(RecursiveV2Error::Overflow)
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
        let uniqueness_security = [
            u32::try_from(UNIQUENESS_SEMANTIC_ROW_LIMBS_V2).map_err(|_| RecursiveV2Error::Limit)?,
            u32::try_from(UNIQUENESS_ROW_FACTOR_DEGREE_V2).map_err(|_| RecursiveV2Error::Limit)?,
            UNIQUENESS_CHALLENGE_BITS_V2,
            UNIQUENESS_RO_QUERY_LOG2_V2,
        ];
        let mut uniqueness_security_bytes = [0_u8; 16];
        for (index, value) in uniqueness_security.into_iter().enumerate() {
            let start = index * 4;
            uniqueness_security_bytes[start..start + 4].copy_from_slice(&value.to_le_bytes());
        }
        let shape_digest = super::recursive_v2::nova::circuit_shape_digest()?;
        let digest = sha256_256_role(
            CheckpointShaRole::Statement,
            &[
                &version,
                &layout_bytes,
                &profile_digest,
                &uniqueness_security_bytes,
                &shape_digest,
            ],
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
    use super::{
        RecursiveCircuitProfileV2, RECURSIVE_V2_MAX_CONTENT_BYTES, TRACE_EVENT_HEADER_BYTES_V2,
    };

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
        let complete_sha = RecursiveCircuitProfileV2::sha_blocks_for_complete_trace(
            RECURSIVE_V2_MAX_CONTENT_BYTES,
            source_records,
            16_000,
            16_000,
        )
        .expect("complete fixture SHA cap");
        assert!(
            complete_sha
                > RecursiveCircuitProfileV2::sha_blocks_for_source_record_hashes(
                    RECURSIVE_V2_MAX_CONTENT_BYTES,
                    source_records,
                )
                .expect("source-local SHA cap"),
            "global/list/transcript SHA work must be included"
        );
        assert_eq!(
            RecursiveCircuitProfileV2::repository_fixture().max_sha_blocks(),
            complete_sha
        );
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
    fn source_record_bound_covers_jmt_micro_records_and_the_largest_flow_payload() {
        assert!(RecursiveCircuitProfileV2::max_source_records(1, 1, 398, 4 * 1024).is_err());
        assert_eq!(
            RecursiveCircuitProfileV2::max_source_records(1, 1, 512, 4 * 1024)
                .expect("byte-derived bound covers arbitrarily short JMT micro records"),
            4 * 1024 / TRACE_EVENT_HEADER_BYTES_V2 as u64
        );
    }

    #[test]
    fn profile_rejects_spool_that_omits_identifier_sorters() {
        let content_bytes = 4 * 1_024;
        let source_records =
            RecursiveCircuitProfileV2::max_source_records(1, 1, 512, content_bytes)
                .expect("small source record cap");
        let sha_blocks = RecursiveCircuitProfileV2::sha_blocks_for_complete_trace(
            content_bytes,
            source_records,
            1,
            1,
        )
        .expect("small complete SHA cap");
        let e_max = RecursiveCircuitProfileV2::event_bound_from_parts(
            1,
            1,
            1,
            1,
            source_records,
            sha_blocks,
            content_bytes,
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
                content_bytes,
                sha_blocks,
                u32::try_from(e_max).expect("small event bound"),
                e_max,
                u32::try_from(2 * super::RECURSIVE_IDENTIFIER_BYTES_V2)
                    .expect("two semantic rows fit the resident-buffer field"),
                total_spool_bytes,
                1,
                1,
            )
        };
        assert!(build(content_bytes).is_err());
        assert!(build(content_bytes + 2 * super::RECURSIVE_IDENTIFIER_BYTES_V2).is_ok());
    }

    #[test]
    fn profile_commits_native_evaluator_resident_buffer_accounting() {
        let profile = RecursiveCircuitProfileV2::repository_fixture();
        let source_record_bytes = u64::try_from(TRACE_EVENT_HEADER_BYTES_V2)
            .expect("header length fits u64")
            + u64::from(profile.max_leaf_bytes());
        let expected = RECURSIVE_V2_MAX_CONTENT_BYTES
            + 2 * source_record_bytes
            + u64::from(profile.resident_buffer_bytes());
        assert_eq!(
            profile
                .native_evaluator_resident_bytes()
                .expect("fixture bound is representable"),
            expected,
            "JMT envelope, concurrent raw/decoded source record, and sort buffers must be explicit"
        );
        assert!(
            profile.canonical_bytes().ends_with(&expected.to_le_bytes()),
            "the native evaluator bound must be committed with the profile"
        );
    }
}
