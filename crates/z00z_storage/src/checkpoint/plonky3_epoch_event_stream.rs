//! Borrowed canonical event-stream view for one epoch transition.
//!
//! The source `TransitionMaterialV2::event_vector` remains the sole byte
//! authority. This module validates and indexes that allocation without
//! creating another witness tape. Direct AIR row builders consume these
//! borrowed records so trace framing, packed range, typed commitments, JMT,
//! and SHA cannot silently parse different byte grammars.

use z00z_crypto::{
    sha256_256_role, CheckpointSha256BlockStreamV2, CheckpointSha256BlockV2,
    CheckpointSha256BlockVisitError, CheckpointShaRole,
};

use super::{validate_event_vector, TransitionMaterialV2, PLONKY3_EVENT_VECTOR_MAGIC_V2};
use crate::{
    checkpoint::{
        recursive_semantics::decode_hierarchy_promotion_fields,
        recursive_trace::{
            structural_event_id, RecursiveTraceEventCountsV2, RecursiveTraceEventV2,
            RecursiveTraceOpcodeV2, TRACE_EVENT_HEADER_BYTES_V2,
        },
    },
    settlement::{SettlementUpdateTraceCircuitDecoderV2, JMT_CIRCUIT_HEADER_BYTES_V2},
    CheckpointError,
};

const EVENT_VECTOR_PREFIX_BYTES_V2: usize = 16;
const EVENT_LENGTH_PREFIX_BYTES_V2: usize = 4;
#[derive(Clone, Copy)]
struct ExpectedEventStreamV2 {
    event_count: u64,
    event_bytes: u64,
    opcode_counts: RecursiveTraceEventCountsV2,
    digest: [u8; 32],
}

/// One zero-copy record indexed from the canonical event vector.
#[derive(Clone, Copy, Debug)]
pub(super) struct EpochEventRecordV2<'a> {
    opcode: RecursiveTraceOpcodeV2,
    ordinal: u64,
    object_id: [u8; 32],
    canonical_bytes: &'a [u8],
    payload: &'a [u8],
}

impl<'a> EpochEventRecordV2<'a> {
    #[must_use]
    pub(super) const fn opcode(self) -> RecursiveTraceOpcodeV2 {
        self.opcode
    }

    #[must_use]
    pub(super) const fn ordinal(self) -> u64 {
        self.ordinal
    }

    #[must_use]
    pub(super) const fn object_id(self) -> [u8; 32] {
        self.object_id
    }

    #[must_use]
    pub(super) const fn canonical_bytes(self) -> &'a [u8] {
        self.canonical_bytes
    }

    #[must_use]
    pub(super) const fn payload(self) -> &'a [u8] {
        self.payload
    }
}

/// Indices of the one canonical JMT transcript inside a transition stream.
#[derive(Debug)]
pub(super) struct EpochJmtTranscriptV2 {
    header_event: usize,
    micro_events: Vec<usize>,
    promotion_event: usize,
    update_count: u32,
    terminal_operation_count: u64,
    trace_digest: [u8; 32],
    promoted_definition_root: [u8; 32],
}

impl EpochJmtTranscriptV2 {
    #[must_use]
    pub(super) const fn update_count(&self) -> u32 {
        self.update_count
    }

    #[must_use]
    pub(super) const fn terminal_operation_count(&self) -> u64 {
        self.terminal_operation_count
    }

    #[must_use]
    pub(super) const fn trace_digest(&self) -> [u8; 32] {
        self.trace_digest
    }

    #[must_use]
    pub(super) const fn promoted_definition_root(&self) -> [u8; 32] {
        self.promoted_definition_root
    }
}

/// Strict borrowed view of one complete transition event vector.
#[derive(Debug)]
pub(super) struct EpochTransitionEventStreamV2<'a> {
    source: &'a [u8],
    records: Vec<EpochEventRecordV2<'a>>,
    jmt: EpochJmtTranscriptV2,
    canonical_event_bytes: u64,
}

impl<'a> EpochTransitionEventStreamV2<'a> {
    #[must_use]
    pub(super) const fn source(&self) -> &'a [u8] {
        self.source
    }

    #[must_use]
    pub(super) fn records(&self) -> &[EpochEventRecordV2<'a>] {
        &self.records
    }

    pub(super) fn typed_commitment_records(
        &self,
    ) -> impl Iterator<Item = EpochEventRecordV2<'a>> + '_ {
        self.records
            .iter()
            .copied()
            .filter(|record| record.opcode() == RecursiveTraceOpcodeV2::CommitTypedEvent)
    }

    #[must_use]
    pub(super) const fn jmt(&self) -> &EpochJmtTranscriptV2 {
        &self.jmt
    }

    #[must_use]
    pub(super) const fn canonical_event_bytes(&self) -> u64 {
        self.canonical_event_bytes
    }

    #[must_use]
    pub(super) fn jmt_header(&self) -> &'a [u8; JMT_CIRCUIT_HEADER_BYTES_V2] {
        self.records[self.jmt.header_event]
            .payload()
            .try_into()
            .expect("validated fixed-width JMT header")
    }

    pub(super) fn jmt_micro_records(&self) -> impl ExactSizeIterator<Item = &'a [u8]> + '_ {
        self.jmt
            .micro_events
            .iter()
            .map(|index| self.records[*index].payload())
    }

    #[must_use]
    pub(super) fn promotion_record(&self) -> EpochEventRecordV2<'a> {
        self.records[self.jmt.promotion_event]
    }

    /// Visit the exact role-framed FIPS compression blocks whose terminal
    /// chaining state is the proof-bound event-vector digest.
    ///
    /// The visitor sees only one block at a time. It may build bounded AIR rows
    /// without materializing a second event-vector or SHA block tape.
    pub(super) fn visit_digest_blocks<F>(&self, visit: &mut F) -> Result<[u8; 32], CheckpointError>
    where
        F: FnMut(CheckpointSha256BlockV2) -> Result<(), CheckpointError>,
    {
        let mut stream = CheckpointSha256BlockStreamV2::new(CheckpointShaRole::EventVector);
        stream
            .update_part_with(self.source, visit)
            .map_err(map_digest_visit_error)?;
        let digest = stream
            .finalize_with(visit)
            .map_err(map_digest_visit_error)?;
        if digest != sha256_256_role(CheckpointShaRole::EventVector, &[self.source]) {
            return Err(CheckpointError::Invariant);
        }
        Ok(digest)
    }
}

pub(super) fn transition_event_stream(
    material: &TransitionMaterialV2,
) -> Result<EpochTransitionEventStreamV2<'_>, CheckpointError> {
    validate_event_vector(&material.statement, &material.event_vector)?;
    parse_event_stream(
        &material.event_vector,
        ExpectedEventStreamV2 {
            event_count: material.transition_statement.declared_event_count(),
            event_bytes: material.transition_statement.declared_byte_count(),
            opcode_counts: material
                .transition_statement
                .declared_event_counts()
                .source_record_counts(),
            digest: material.statement.event_vector_digest(),
        },
    )
}

fn parse_event_stream(
    source: &[u8],
    expected: ExpectedEventStreamV2,
) -> Result<EpochTransitionEventStreamV2<'_>, CheckpointError> {
    if source.len() < EVENT_VECTOR_PREFIX_BYTES_V2
        || source[..8] != PLONKY3_EVENT_VECTOR_MAGIC_V2
        || expected.event_count == 0
        || expected.event_bytes == 0
        || expected.digest == [0; 32]
    {
        return Err(CheckpointError::Canonical);
    }
    let declared_count = u64::from_le_bytes(
        source[8..EVENT_VECTOR_PREFIX_BYTES_V2]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    let expected_encoded_len = u64::try_from(EVENT_VECTOR_PREFIX_BYTES_V2)
        .map_err(|_| CheckpointError::Limit)?
        .checked_add(
            expected
                .event_count
                .checked_mul(
                    u64::try_from(EVENT_LENGTH_PREFIX_BYTES_V2)
                        .map_err(|_| CheckpointError::Limit)?,
                )
                .ok_or(CheckpointError::Overflow)?,
        )
        .and_then(|bytes| bytes.checked_add(expected.event_bytes))
        .ok_or(CheckpointError::Overflow)?;
    if declared_count != expected.event_count
        || u64::try_from(source.len()).map_err(|_| CheckpointError::Limit)? != expected_encoded_len
        || sha256_256_role(CheckpointShaRole::EventVector, &[source]) != expected.digest
    {
        return Err(CheckpointError::Canonical);
    }

    let profile = super::RecursiveCircuitProfileV2::authority_pinned();
    let mut cursor = EVENT_VECTOR_PREFIX_BYTES_V2;
    let mut records = Vec::new();
    records
        .try_reserve_exact(
            usize::try_from(expected.event_count).map_err(|_| CheckpointError::Limit)?,
        )
        .map_err(|_| CheckpointError::Limit)?;
    let mut opcode_counts = RecursiveTraceEventCountsV2::default();
    let mut canonical_event_bytes = 0_u64;

    let mut jmt_decoder = None;
    let mut jmt_header_event = None;
    let mut jmt_micro_events = Vec::new();
    let mut jmt_transcript = None;

    while cursor < source.len() {
        let event_len =
            usize::try_from(take_u32(source, &mut cursor)?).map_err(|_| CheckpointError::Limit)?;
        let event_bytes = take_slice(source, &mut cursor, event_len)?;
        let event = RecursiveTraceEventV2::decode_canonical(event_bytes, &profile)?;
        if event.canonical_bytes()? != event_bytes {
            return Err(CheckpointError::Canonical);
        }
        let payload = event_bytes
            .get(TRACE_EVENT_HEADER_BYTES_V2..)
            .ok_or(CheckpointError::Canonical)?;
        let record = EpochEventRecordV2 {
            opcode: event.opcode(),
            ordinal: event.ordinal(),
            object_id: event.object_id(),
            canonical_bytes: event_bytes,
            payload,
        };
        if record.ordinal() != u64::try_from(records.len()).map_err(|_| CheckpointError::Limit)? {
            return Err(CheckpointError::EventOrder);
        }
        opcode_counts.increment(record.opcode())?;
        canonical_event_bytes = canonical_event_bytes
            .checked_add(u64::try_from(event_len).map_err(|_| CheckpointError::Limit)?)
            .ok_or(CheckpointError::Overflow)?;
        let record_index = records.len();

        if jmt_decoder.is_some()
            && record.opcode().is_source_record()
            && !matches!(
                record.opcode(),
                RecursiveTraceOpcodeV2::JmtMicroOp | RecursiveTraceOpcodeV2::PromoteChildRoot
            )
        {
            return Err(CheckpointError::EventOrder);
        }
        match record.opcode() {
            RecursiveTraceOpcodeV2::JmtUpdate => {
                require_structural_id(record)?;
                if jmt_decoder.is_some()
                    || jmt_header_event.is_some()
                    || jmt_transcript.is_some()
                    || payload.len() != JMT_CIRCUIT_HEADER_BYTES_V2
                {
                    return Err(CheckpointError::EventOrder);
                }
                jmt_decoder = Some(
                    SettlementUpdateTraceCircuitDecoderV2::new(payload)
                        .map_err(|_| CheckpointError::Canonical)?,
                );
                jmt_header_event = Some(record_index);
            }
            RecursiveTraceOpcodeV2::JmtMicroOp => {
                require_structural_id(record)?;
                jmt_decoder
                    .as_mut()
                    .ok_or(CheckpointError::EventOrder)?
                    .accept(payload)
                    .map_err(|_| CheckpointError::Canonical)?;
                jmt_micro_events.push(record_index);
            }
            RecursiveTraceOpcodeV2::PromoteChildRoot => {
                require_structural_id(record)?;
                let decoder = jmt_decoder.take().ok_or(CheckpointError::EventOrder)?;
                let summary = decoder.finish().map_err(|_| CheckpointError::Canonical)?;
                let (promoted_definition_root, promoted_trace_digest) =
                    decode_hierarchy_promotion_fields(payload)?;
                if promoted_trace_digest != summary.trace_digest()
                    || (summary.is_noop() && !jmt_micro_events.is_empty())
                    || (!summary.is_noop() && jmt_micro_events.is_empty())
                {
                    return Err(CheckpointError::Canonical);
                }
                jmt_transcript = Some(EpochJmtTranscriptV2 {
                    header_event: jmt_header_event.ok_or(CheckpointError::EventOrder)?,
                    micro_events: core::mem::take(&mut jmt_micro_events),
                    promotion_event: record_index,
                    update_count: summary.update_count(),
                    terminal_operation_count: summary
                        .terminal_operation_count()
                        .map_err(|_| CheckpointError::Canonical)?,
                    trace_digest: summary.trace_digest(),
                    promoted_definition_root,
                });
            }
            RecursiveTraceOpcodeV2::CommitTypedEvent => {
                require_structural_id(record)?;
            }
            _ => {}
        }
        records.push(record);
        if u64::try_from(records.len()).map_err(|_| CheckpointError::Limit)? > expected.event_count
        {
            return Err(CheckpointError::Canonical);
        }
    }

    if cursor != source.len()
        || u64::try_from(records.len()).map_err(|_| CheckpointError::Limit)? != expected.event_count
        || canonical_event_bytes != expected.event_bytes
        || opcode_counts != expected.opcode_counts
        || jmt_decoder.is_some()
        || !jmt_micro_events.is_empty()
    {
        return Err(CheckpointError::Canonical);
    }
    Ok(EpochTransitionEventStreamV2 {
        source,
        records,
        jmt: jmt_transcript.ok_or(CheckpointError::EventOrder)?,
        canonical_event_bytes,
    })
}

fn map_digest_visit_error(
    error: CheckpointSha256BlockVisitError<CheckpointError>,
) -> CheckpointError {
    match error {
        CheckpointSha256BlockVisitError::Hash(error) => {
            CheckpointError::Backend(format!("event-vector SHA block stream failed: {error}"))
        }
        CheckpointSha256BlockVisitError::Visitor(error) => error,
    }
}

fn require_structural_id(record: EpochEventRecordV2<'_>) -> Result<(), CheckpointError> {
    if record.object_id()
        != structural_event_id(record.opcode(), record.ordinal(), record.payload())
    {
        return Err(CheckpointError::Canonical);
    }
    Ok(())
}

fn take_slice<'a>(
    bytes: &'a [u8],
    cursor: &mut usize,
    len: usize,
) -> Result<&'a [u8], CheckpointError> {
    let end = cursor.checked_add(len).ok_or(CheckpointError::Overflow)?;
    let value = bytes.get(*cursor..end).ok_or(CheckpointError::Canonical)?;
    *cursor = end;
    Ok(value)
}

fn take_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, CheckpointError> {
    Ok(u32::from_le_bytes(
        take_slice(bytes, cursor, core::mem::size_of::<u32>())?
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkpoint::{
        recursive_circuit::RecursiveCircuitProfileV2,
        recursive_semantics::encode_hierarchy_promotion,
    };

    fn event(
        ordinal: u64,
        opcode: RecursiveTraceOpcodeV2,
        payload: Vec<u8>,
    ) -> RecursiveTraceEventV2 {
        let object_id = structural_event_id(opcode, ordinal, &payload);
        RecursiveTraceEventV2::new(
            ordinal,
            opcode,
            object_id,
            payload,
            &RecursiveCircuitProfileV2::authority_pinned(),
        )
        .expect("bounded canonical event")
    }

    fn fixture() -> (Vec<u8>, ExpectedEventStreamV2, Vec<RecursiveTraceEventV2>) {
        let (header, records) = crate::settlement::jmt_mutation_case_circuit_transcripts_for_test()
            .into_iter()
            .find_map(|(label, header, records)| {
                (label == "split_insert").then_some((header, records))
            })
            .expect("split-insert fixture");
        let trace_digest: [u8; 32] = header[3..35].try_into().expect("header digest");
        let mut events = Vec::with_capacity(records.len() + 2);
        events.push(event(0, RecursiveTraceOpcodeV2::JmtUpdate, header));
        for (index, record) in records.into_iter().enumerate() {
            events.push(event(
                u64::try_from(index + 1).expect("bounded ordinal"),
                RecursiveTraceOpcodeV2::JmtMicroOp,
                record,
            ));
        }
        events.push(event(
            u64::try_from(events.len()).expect("bounded ordinal"),
            RecursiveTraceOpcodeV2::PromoteChildRoot,
            encode_hierarchy_promotion([0x5a; 32], trace_digest),
        ));

        let mut source = Vec::new();
        source.extend_from_slice(&PLONKY3_EVENT_VECTOR_MAGIC_V2);
        source.extend_from_slice(
            &u64::try_from(events.len())
                .expect("bounded event count")
                .to_le_bytes(),
        );
        let mut event_bytes = 0_u64;
        let mut opcode_counts = RecursiveTraceEventCountsV2::default();
        for event in &events {
            let canonical = event.canonical_bytes().expect("canonical event");
            source.extend_from_slice(
                &u32::try_from(canonical.len())
                    .expect("bounded event length")
                    .to_le_bytes(),
            );
            source.extend_from_slice(&canonical);
            event_bytes += u64::try_from(canonical.len()).expect("bounded event bytes");
            opcode_counts
                .increment(event.opcode())
                .expect("bounded opcode count");
        }
        let expected = ExpectedEventStreamV2 {
            event_count: u64::try_from(events.len()).expect("bounded event count"),
            event_bytes,
            opcode_counts,
            digest: sha256_256_role(CheckpointShaRole::EventVector, &[&source]),
        };
        (source, expected, events)
    }

    #[test]
    fn borrowed_event_stream_has_one_canonical_jmt_path() {
        let (source, expected, events) = fixture();
        let stream = parse_event_stream(&source, expected).expect("canonical event stream");
        assert_eq!(stream.source(), source);
        assert_eq!(stream.records().len(), events.len());
        assert_eq!(stream.canonical_event_bytes(), expected.event_bytes);
        assert_eq!(
            stream.jmt_header().as_slice(),
            events[0].payload(),
            "JMT header borrows the sole source bytes",
        );
        assert_eq!(
            stream.jmt_micro_records().count(),
            events.len() - 2,
            "every micro record is indexed exactly once",
        );
        assert_eq!(stream.jmt().trace_digest(), stream.jmt_header()[3..35]);
        assert_eq!(stream.jmt().promoted_definition_root(), [0x5a; 32]);
        assert_eq!(
            stream.promotion_record().opcode(),
            RecursiveTraceOpcodeV2::PromoteChildRoot,
        );
        assert!(stream.jmt().update_count() > 0);
        assert!(stream.jmt().terminal_operation_count() > 0);
        assert_eq!(
            stream.records()[0].canonical_bytes(),
            events[0].canonical_bytes().expect("canonical header event"),
        );
    }

    #[test]
    fn source_opcode_projection_excludes_derived_controls() {
        let (source, expected, _) = fixture();
        let mut expanded = expected.opcode_counts;
        for opcode in [
            RecursiveTraceOpcodeV2::BeginHash,
            RecursiveTraceOpcodeV2::ShaBlock,
            RecursiveTraceOpcodeV2::EndHash,
            RecursiveTraceOpcodeV2::SourceMemoryWrite,
            RecursiveTraceOpcodeV2::TraceChunk,
        ] {
            expanded.increment(opcode).expect("bounded derived count");
        }
        assert_eq!(
            expanded.source_record_counts(),
            expected.opcode_counts,
            "direct AIR source parsing must ignore only derived controls",
        );
        assert!(
            expanded.total_count().expect("bounded expanded count")
                > expanded
                    .source_record_count()
                    .expect("bounded source count"),
        );
        parse_event_stream(
            &source,
            ExpectedEventStreamV2 {
                opcode_counts: expanded.source_record_counts(),
                ..expected
            },
        )
        .expect("source-only projection preserves the canonical stream");
    }

    #[test]
    fn event_stream_rejects_digest_count_and_jmt_mutation() {
        let (source, expected, _) = fixture();

        let mut wrong_digest = expected;
        wrong_digest.digest[0] ^= 1;
        assert!(parse_event_stream(&source, wrong_digest).is_err());

        let mut wrong_count = expected;
        wrong_count.event_count += 1;
        assert!(parse_event_stream(&source, wrong_count).is_err());

        let mut malformed = source.clone();
        let first_event_start = EVENT_VECTOR_PREFIX_BYTES_V2 + EVENT_LENGTH_PREFIX_BYTES_V2;
        let payload_start = first_event_start + TRACE_EVENT_HEADER_BYTES_V2;
        malformed[payload_start + 3] ^= 1;
        let mut malformed_expected = expected;
        malformed_expected.digest = sha256_256_role(CheckpointShaRole::EventVector, &[&malformed]);
        assert!(parse_event_stream(&malformed, malformed_expected).is_err());
    }

    #[test]
    fn event_stream_exposes_one_canonical_sha_block_chain() {
        let (source, expected, _) = fixture();
        let stream = parse_event_stream(&source, expected).expect("canonical event stream");
        let mut prior = None;
        let mut block_count = 0_u64;
        let digest = stream
            .visit_digest_blocks(&mut |block| {
                assert_eq!(block.index(), block_count);
                assert!(block.verifies_transition());
                if let Some(previous) = prior {
                    assert_eq!(*block.chaining_before(), previous);
                }
                prior = Some(*block.chaining_after());
                block_count += 1;
                Ok(())
            })
            .expect("canonical SHA block stream");
        assert!(block_count > 0);
        assert_eq!(digest, expected.digest);
        assert_eq!(
            prior.map(|state| CheckpointSha256BlockV2::digest_from_chaining(&state)),
            Some(expected.digest),
        );
    }
}
