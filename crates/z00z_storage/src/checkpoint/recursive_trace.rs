//! Rewindable, bounded, two-pass typed trace source for recursive checkpoint V2.

use std::path::{Path, PathBuf};

use crate::CheckpointError;
use z00z_crypto::{
    sha256_256_role, CheckpointSha256BlockStreamV2, CheckpointSha256BlockV2,
    CheckpointSha256BlockVisitError, CheckpointSha256V2, CheckpointShaRole,
};
use z00z_utils::io::PrivateSpoolFile;
use zeroize::{Zeroize, Zeroizing};

use super::{
    recursive_circuit::RecursiveCircuitProfileV2,
    recursive_context::RecursiveSnapshotHandleV2,
    recursive_semantics::{
        decode_flow_header, decode_flow_item, decode_hierarchy_promotion_fields,
        decode_uniqueness_challenge, decode_uniqueness_precommit, decode_uniqueness_sorted_row,
        UniquenessChallengesV2, UniquenessListKindV2, UniquenessPassV2, UniquenessPrecommitV2,
        UniquenessSemanticRowV2, UniquenessSetKindV2, UNIQUENESS_SEMANTIC_ROW_BYTES_V2,
    },
    recursive_statement::RecursivePreUniquenessContextV2,
};

pub(crate) const TRACE_EVENT_HEADER_BYTES_V2: usize = 1 + 8 + 32 + 4;
/// Fixed byte granularity for the live in-circuit source/global SHA feeder.
///
/// A chunk is only a view of the one canonical source-record encoding below;
/// it is never a second record serialization.
pub(crate) const TRACE_CANONICAL_CHUNK_BYTES_V2: usize = 64;
const TRACE_CHUNK_CONTROL_VERSION_V2: u8 = 1;
const TRACE_CHUNK_CONTROL_PAYLOAD_BYTES_V2: usize =
    1 + 8 + 4 + 4 + 1 + TRACE_CANONICAL_CHUNK_BYTES_V2;
const TRACE_CHUNK_ORDINAL_FLAG_V2: u64 = 1_u64 << 62;
const SOURCE_MEMORY_WRITE_ORDINAL_FLAG_V2: u64 = 1_u64 << 61;
/// Frozen first part of every per-source SHA control transcript.
pub(crate) const SOURCE_RECORD_HASH_LABEL_V2: &[u8] = b"z00z.recursive.v2.source-record-hash";

/// Frozen recursive V2 opcode grammar at the storage trace boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum RecursiveTraceOpcodeV2 {
    BeginBlock = 1,
    ReplayInput = 2,
    ReplayOutput = 3,
    BeginHash = 4,
    ShaBlock = 5,
    EndHash = 6,
    UniquenessPrecommit = 7,
    UniquenessChallenge = 8,
    NetMerge = 9,
    JmtUpdate = 10,
    PromoteChildRoot = 11,
    CommitTypedEvent = 12,
    FinalizeBlock = 13,
    /// Derived canonical source-window write for the private R1CS memory
    /// relation. It is emitted immediately before its matching `TraceChunk`.
    SourceMemoryWrite = 14,
    /// Derived, non-committed canonical-byte feeder for the private SHA lane.
    TraceChunk = 15,
    /// One source-authenticated strictly ordered identifier row. Original
    /// identifiers are extracted from replay rows; this separate row carries
    /// the sorted counterpart for the permutation/net relation.
    UniquenessSorted = 16,
    /// Circuit-consumable canonical JMT update/proof micro-operation. These
    /// records replace opaque proof bytes in the recursive source grammar and
    /// are hashed independently into the envelope trace digest.
    JmtMicroOp = 17,
}

const RECURSIVE_TRACE_OPCODE_COUNT_V2: usize = 17;

/// Exact declared or consumed count for every frozen trace opcode class.
///
/// The one fixed array is indexed only through [`RecursiveTraceOpcodeV2`], so
/// a source/control event cannot be silently charged to an adjacent semantic
/// class.  It carries no witness bytes and is committed by the transition
/// statement only after the evaluator proves declared equals consumed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecursiveTraceEventCountsV2 {
    counts: [u64; RECURSIVE_TRACE_OPCODE_COUNT_V2],
}

impl RecursiveTraceEventCountsV2 {
    pub(crate) fn add(
        &mut self,
        opcode: RecursiveTraceOpcodeV2,
        count: u64,
    ) -> Result<(), CheckpointError> {
        let index = opcode_index(opcode);
        self.counts[index] = self.counts[index]
            .checked_add(count)
            .ok_or(CheckpointError::Overflow)?;
        Ok(())
    }

    pub(crate) fn increment(
        &mut self,
        opcode: RecursiveTraceOpcodeV2,
    ) -> Result<(), CheckpointError> {
        let index = opcode_index(opcode);
        self.counts[index] = self.counts[index]
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        Ok(())
    }

    /// Return the exact count charged to one frozen opcode class.
    #[must_use]
    pub const fn count(&self, opcode: RecursiveTraceOpcodeV2) -> u64 {
        self.counts[opcode_index(opcode)]
    }

    /// Return the number of canonical source records, excluding derived hash
    /// and fixed-width chunk controls.
    pub fn source_record_count(&self) -> Result<u64, CheckpointError> {
        [
            RecursiveTraceOpcodeV2::BeginBlock,
            RecursiveTraceOpcodeV2::ReplayInput,
            RecursiveTraceOpcodeV2::ReplayOutput,
            RecursiveTraceOpcodeV2::UniquenessPrecommit,
            RecursiveTraceOpcodeV2::UniquenessSorted,
            RecursiveTraceOpcodeV2::UniquenessChallenge,
            RecursiveTraceOpcodeV2::NetMerge,
            RecursiveTraceOpcodeV2::JmtUpdate,
            RecursiveTraceOpcodeV2::JmtMicroOp,
            RecursiveTraceOpcodeV2::PromoteChildRoot,
            RecursiveTraceOpcodeV2::CommitTypedEvent,
            RecursiveTraceOpcodeV2::FinalizeBlock,
        ]
        .into_iter()
        .try_fold(0_u64, |total, opcode| {
            total
                .checked_add(self.count(opcode))
                .ok_or(CheckpointError::Overflow)
        })
    }

    /// Return the complete expanded schedule count, including derived control
    /// records. This is kept distinct from `source_record_count` on purpose.
    pub(crate) fn total_count(&self) -> Result<u64, CheckpointError> {
        self.counts.iter().try_fold(0_u64, |total, count| {
            total.checked_add(*count).ok_or(CheckpointError::Overflow)
        })
    }

    #[must_use]
    pub(crate) const fn counts(&self) -> [u64; RECURSIVE_TRACE_OPCODE_COUNT_V2] {
        self.counts
    }

    pub(crate) fn canonical_bytes(&self) -> [u8; RECURSIVE_TRACE_OPCODE_COUNT_V2 * 8] {
        let mut bytes = [0_u8; RECURSIVE_TRACE_OPCODE_COUNT_V2 * 8];
        for (index, count) in self.counts.iter().enumerate() {
            let offset = index * 8;
            bytes[offset..offset + 8].copy_from_slice(&count.to_le_bytes());
        }
        bytes
    }
}

const fn opcode_index(opcode: RecursiveTraceOpcodeV2) -> usize {
    (opcode as usize) - 1
}

impl RecursiveTraceOpcodeV2 {
    /// Commit the complete frozen opcode alphabet in numeric order.
    #[must_use]
    pub(crate) fn grammar_digest() -> [u8; 32] {
        const OPCODES: [u8; RECURSIVE_TRACE_OPCODE_COUNT_V2] =
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
        sha256_256_role(
            CheckpointShaRole::Statement,
            &[b"z00z.recursive.v2.trace-opcode-grammar", &OPCODES],
        )
    }

    fn decode(byte: u8) -> Result<Self, CheckpointError> {
        match byte {
            1 => Ok(Self::BeginBlock),
            2 => Ok(Self::ReplayInput),
            3 => Ok(Self::ReplayOutput),
            4 => Ok(Self::BeginHash),
            5 => Ok(Self::ShaBlock),
            6 => Ok(Self::EndHash),
            7 => Ok(Self::UniquenessPrecommit),
            8 => Ok(Self::UniquenessChallenge),
            9 => Ok(Self::NetMerge),
            10 => Ok(Self::JmtUpdate),
            11 => Ok(Self::PromoteChildRoot),
            12 => Ok(Self::CommitTypedEvent),
            13 => Ok(Self::FinalizeBlock),
            14 => Ok(Self::SourceMemoryWrite),
            15 => Ok(Self::TraceChunk),
            16 => Ok(Self::UniquenessSorted),
            17 => Ok(Self::JmtMicroOp),
            _ => Err(CheckpointError::Canonical),
        }
    }

    pub(crate) const fn is_source_record(self) -> bool {
        matches!(
            self,
            Self::BeginBlock
                | Self::ReplayInput
                | Self::ReplayOutput
                | Self::UniquenessPrecommit
                | Self::UniquenessSorted
                | Self::UniquenessChallenge
                | Self::NetMerge
                | Self::JmtUpdate
                | Self::JmtMicroOp
                | Self::PromoteChildRoot
                | Self::CommitTypedEvent
                | Self::FinalizeBlock
        )
    }
}

/// Decoded source-byte chunk emitted by the one canonical source owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RecursiveTraceChunkControlV2 {
    pub(crate) source_ordinal: u64,
    pub(crate) chunk_ordinal: u32,
    pub(crate) chunk_count: u32,
    pub(crate) byte_count: u8,
    pub(crate) bytes: [u8; TRACE_CANONICAL_CHUNK_BYTES_V2],
}

/// One structural event identifier shared by the source, evaluator, and
/// statement path.  The control expander never invents a second identifier
/// family.
pub(crate) fn structural_event_id(
    opcode: RecursiveTraceOpcodeV2,
    ordinal: u64,
    payload: &[u8],
) -> [u8; 32] {
    let opcode_byte = [opcode as u8];
    let ordinal_bytes = ordinal.to_le_bytes();
    sha256_256_role(
        CheckpointShaRole::Trace,
        &[
            b"z00z.recursive.v2.structural-event",
            &opcode_byte,
            &ordinal_bytes,
            payload,
        ],
    )
}

/// One bounded canonical typed event replayed in both passes.
#[derive(Clone, PartialEq, Eq)]
pub struct RecursiveTraceEventV2 {
    ordinal: u64,
    opcode: RecursiveTraceOpcodeV2,
    object_id: [u8; 32],
    payload: Vec<u8>,
}

impl Zeroize for RecursiveTraceEventV2 {
    fn zeroize(&mut self) {
        self.payload.zeroize();
    }
}

impl Drop for RecursiveTraceEventV2 {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// One zero-padded, ordered view of a canonical source record.
///
/// The explicit source ordinal, chunk ordinal, total count and byte count let
/// the Nova feeder constrain a bounded source parser without retaining the
/// whole trace.  `bytes[..byte_count]` is the only meaningful data; the tail
/// is required to be zero when it is materialized in R1CS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RecursiveTraceCanonicalChunkV2 {
    source_ordinal: u64,
    chunk_ordinal: u32,
    chunk_count: u32,
    byte_count: u8,
    bytes: [u8; TRACE_CANONICAL_CHUNK_BYTES_V2],
}

impl RecursiveTraceCanonicalChunkV2 {
    #[must_use]
    pub(crate) const fn source_ordinal(self) -> u64 {
        self.source_ordinal
    }

    #[must_use]
    pub(crate) const fn chunk_ordinal(self) -> u32 {
        self.chunk_ordinal
    }

    #[must_use]
    pub(crate) const fn chunk_count(self) -> u32 {
        self.chunk_count
    }

    #[must_use]
    pub(crate) const fn byte_count(self) -> u8 {
        self.byte_count
    }

    #[must_use]
    pub(crate) const fn bytes(self) -> [u8; TRACE_CANONICAL_CHUNK_BYTES_V2] {
        self.bytes
    }
}

fn trace_chunk_control_ordinal(
    source_ordinal: u64,
    chunk_ordinal: u32,
) -> Result<u64, CheckpointError> {
    TRACE_CHUNK_ORDINAL_FLAG_V2
        .checked_add(
            source_ordinal
                .checked_mul(HASH_CONTROL_ORDINAL_STRIDE)
                .and_then(|value| value.checked_add(u64::from(chunk_ordinal)))
                .ok_or(CheckpointError::Overflow)?,
        )
        .ok_or(CheckpointError::Overflow)
}

fn source_memory_write_control_ordinal(
    source_ordinal: u64,
    chunk_ordinal: u32,
) -> Result<u64, CheckpointError> {
    SOURCE_MEMORY_WRITE_ORDINAL_FLAG_V2
        .checked_add(
            source_ordinal
                .checked_mul(HASH_CONTROL_ORDINAL_STRIDE)
                .and_then(|value| value.checked_add(u64::from(chunk_ordinal)))
                .ok_or(CheckpointError::Overflow)?,
        )
        .ok_or(CheckpointError::Overflow)
}

/// Test helper that materializes fixed-width controls from the canonical source
/// encoder. Production uses the same indexed source view through
/// [`emit_derived_hash_controls`] and never retains a chunk vector.
#[cfg(test)]
pub(crate) fn emit_derived_trace_chunks(
    source: &RecursiveTraceEventV2,
    profile: &RecursiveCircuitProfileV2,
    mut emit: impl FnMut(RecursiveTraceEventV2) -> Result<(), CheckpointError>,
) -> Result<(), CheckpointError> {
    for chunk in source.canonical_chunks_for_test()? {
        emit(trace_chunk_control_event(chunk, profile)?)?;
    }
    Ok(())
}

fn trace_chunk_control_event(
    chunk: RecursiveTraceCanonicalChunkV2,
    profile: &RecursiveCircuitProfileV2,
) -> Result<RecursiveTraceEventV2, CheckpointError> {
    canonical_chunk_control_event(
        RecursiveTraceOpcodeV2::TraceChunk,
        trace_chunk_control_ordinal(chunk.source_ordinal(), chunk.chunk_ordinal())?,
        chunk,
        profile,
    )
}

fn source_memory_write_control_event(
    chunk: RecursiveTraceCanonicalChunkV2,
    profile: &RecursiveCircuitProfileV2,
) -> Result<RecursiveTraceEventV2, CheckpointError> {
    canonical_chunk_control_event(
        RecursiveTraceOpcodeV2::SourceMemoryWrite,
        source_memory_write_control_ordinal(chunk.source_ordinal(), chunk.chunk_ordinal())?,
        chunk,
        profile,
    )
}

fn canonical_chunk_control_event(
    opcode: RecursiveTraceOpcodeV2,
    ordinal: u64,
    chunk: RecursiveTraceCanonicalChunkV2,
    profile: &RecursiveCircuitProfileV2,
) -> Result<RecursiveTraceEventV2, CheckpointError> {
    let mut payload = Vec::with_capacity(TRACE_CHUNK_CONTROL_PAYLOAD_BYTES_V2);
    payload.push(TRACE_CHUNK_CONTROL_VERSION_V2);
    payload.extend_from_slice(&chunk.source_ordinal().to_le_bytes());
    payload.extend_from_slice(&chunk.chunk_ordinal().to_le_bytes());
    payload.extend_from_slice(&chunk.chunk_count().to_le_bytes());
    payload.push(chunk.byte_count());
    payload.extend_from_slice(&chunk.bytes());
    let object_id = structural_event_id(opcode, ordinal, &payload);
    RecursiveTraceEventV2::new(ordinal, opcode, object_id, payload, profile)
}

/// Decode and validate a fixed-width control derived from one canonical source
/// window. The expected opcode makes the source-memory write and SHA feeder
/// distinct schedule edges without creating a second byte encoding.
fn decode_canonical_chunk_control(
    event: &RecursiveTraceEventV2,
    expected_opcode: RecursiveTraceOpcodeV2,
    expected_ordinal: impl FnOnce(u64, u32) -> Result<u64, CheckpointError>,
) -> Result<RecursiveTraceChunkControlV2, CheckpointError> {
    if event.opcode() != expected_opcode
        || event.payload().len() != TRACE_CHUNK_CONTROL_PAYLOAD_BYTES_V2
        || event.object_id()
            != structural_event_id(event.opcode(), event.ordinal(), event.payload())
        || event.payload()[0] != TRACE_CHUNK_CONTROL_VERSION_V2
    {
        return Err(CheckpointError::Canonical);
    }
    let payload = event.payload();
    let source_ordinal = u64::from_le_bytes(
        payload[1..9]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    let chunk_ordinal = u32::from_le_bytes(
        payload[9..13]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    let chunk_count = u32::from_le_bytes(
        payload[13..17]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    let byte_count = payload[17];
    let bytes: [u8; TRACE_CANONICAL_CHUNK_BYTES_V2] = payload[18..]
        .try_into()
        .map_err(|_| CheckpointError::Canonical)?;
    if chunk_count == 0
        || chunk_ordinal >= chunk_count
        || byte_count == 0
        || usize::from(byte_count) > TRACE_CANONICAL_CHUNK_BYTES_V2
        || bytes[usize::from(byte_count)..]
            .iter()
            .any(|byte| *byte != 0)
        || event.ordinal() != expected_ordinal(source_ordinal, chunk_ordinal)?
    {
        return Err(CheckpointError::Canonical);
    }
    Ok(RecursiveTraceChunkControlV2 {
        source_ordinal,
        chunk_ordinal,
        chunk_count,
        byte_count,
        bytes,
    })
}

/// Decode and validate one fixed-width canonical-byte feeder control.
pub(crate) fn decode_trace_chunk_control(
    event: &RecursiveTraceEventV2,
) -> Result<RecursiveTraceChunkControlV2, CheckpointError> {
    decode_canonical_chunk_control(
        event,
        RecursiveTraceOpcodeV2::TraceChunk,
        trace_chunk_control_ordinal,
    )
}

/// Decode the immediately preceding source-memory write from the same sole
/// canonical chunk grammar as [`decode_trace_chunk_control`].
pub(crate) fn decode_source_memory_write_control(
    event: &RecursiveTraceEventV2,
) -> Result<RecursiveTraceChunkControlV2, CheckpointError> {
    decode_canonical_chunk_control(
        event,
        RecursiveTraceOpcodeV2::SourceMemoryWrite,
        source_memory_write_control_ordinal,
    )
}

impl RecursiveTraceEventV2 {
    /// Construct one profile-bounded event.
    pub(crate) fn new(
        ordinal: u64,
        opcode: RecursiveTraceOpcodeV2,
        object_id: [u8; 32],
        payload: Vec<u8>,
        profile: &RecursiveCircuitProfileV2,
    ) -> Result<Self, CheckpointError> {
        if object_id == [0; 32]
            || payload.len()
                > usize::try_from(profile.max_leaf_bytes()).map_err(|_| CheckpointError::Limit)?
        {
            return Err(CheckpointError::Limit);
        }
        Ok(Self {
            ordinal,
            opcode,
            object_id,
            payload,
        })
    }

    #[must_use]
    pub const fn ordinal(&self) -> u64 {
        self.ordinal
    }

    #[must_use]
    pub const fn opcode(&self) -> RecursiveTraceOpcodeV2 {
        self.opcode
    }

    #[must_use]
    pub const fn object_id(&self) -> [u8; 32] {
        self.object_id
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub(crate) fn canonical_len(&self) -> Result<u64, CheckpointError> {
        u64::try_from(
            TRACE_EVENT_HEADER_BYTES_V2
                .checked_add(self.payload.len())
                .ok_or(CheckpointError::Overflow)?,
        )
        .map_err(|_| CheckpointError::Limit)
    }

    pub(crate) fn canonical_bytes(&self) -> Result<Vec<u8>, CheckpointError> {
        encode_canonical_source_record(self.opcode, self.ordinal, self.object_id, &self.payload)
    }

    /// Return the fixed number of canonical chunks without materializing them.
    pub(crate) fn canonical_chunk_count(&self) -> Result<u32, CheckpointError> {
        let bytes = usize::try_from(self.canonical_len()?).map_err(|_| CheckpointError::Limit)?;
        let chunk_count = bytes
            .checked_add(TRACE_CANONICAL_CHUNK_BYTES_V2 - 1)
            .map(|value| value / TRACE_CANONICAL_CHUNK_BYTES_V2)
            .ok_or(CheckpointError::Overflow)?;
        let chunk_count = u32::try_from(chunk_count).map_err(|_| CheckpointError::Limit)?;
        if chunk_count == 0 {
            return Err(CheckpointError::Canonical);
        }
        Ok(chunk_count)
    }

    /// Reconstruct exactly one fixed-width canonical chunk from the one source
    /// record.  This is the production feeder view: it uses a stack header and
    /// borrows the existing payload, so no per-record chunk tape is allocated.
    pub(crate) fn canonical_chunk(
        &self,
        chunk_ordinal: u32,
    ) -> Result<RecursiveTraceCanonicalChunkV2, CheckpointError> {
        let chunk_count = self.canonical_chunk_count()?;
        if chunk_ordinal >= chunk_count {
            return Err(CheckpointError::Canonical);
        }
        let payload_len = u32::try_from(self.payload.len()).map_err(|_| CheckpointError::Limit)?;
        let mut header = [0_u8; TRACE_EVENT_HEADER_BYTES_V2];
        header[0] = self.opcode as u8;
        header[1..9].copy_from_slice(&self.ordinal.to_le_bytes());
        header[9..41].copy_from_slice(&self.object_id);
        header[41..45].copy_from_slice(&payload_len.to_le_bytes());

        let start = usize::try_from(chunk_ordinal)
            .map_err(|_| CheckpointError::Limit)?
            .checked_mul(TRACE_CANONICAL_CHUNK_BYTES_V2)
            .ok_or(CheckpointError::Overflow)?;
        let total = TRACE_EVENT_HEADER_BYTES_V2
            .checked_add(self.payload.len())
            .ok_or(CheckpointError::Overflow)?;
        let end = start
            .checked_add(TRACE_CANONICAL_CHUNK_BYTES_V2)
            .map(|value| value.min(total))
            .ok_or(CheckpointError::Overflow)?;
        let byte_count = u8::try_from(end.checked_sub(start).ok_or(CheckpointError::Overflow)?)
            .map_err(|_| CheckpointError::Limit)?;
        let mut bytes = [0_u8; TRACE_CANONICAL_CHUNK_BYTES_V2];
        for (offset, output) in bytes[..usize::from(byte_count)].iter_mut().enumerate() {
            let source_offset = start.checked_add(offset).ok_or(CheckpointError::Overflow)?;
            *output = if source_offset < TRACE_EVENT_HEADER_BYTES_V2 {
                header[source_offset]
            } else {
                self.payload[source_offset - TRACE_EVENT_HEADER_BYTES_V2]
            };
        }
        Ok(RecursiveTraceCanonicalChunkV2 {
            source_ordinal: self.ordinal,
            chunk_ordinal,
            chunk_count,
            byte_count,
            bytes,
        })
    }

    /// Test-only materialization for vector assertions. Production callers use
    /// [`Self::canonical_chunk_count`] plus [`Self::canonical_chunk`].
    #[cfg(test)]
    pub(crate) fn canonical_chunks_for_test(
        &self,
    ) -> Result<Vec<RecursiveTraceCanonicalChunkV2>, CheckpointError> {
        let count = self.canonical_chunk_count()?;
        (0..count)
            .map(|ordinal| self.canonical_chunk(ordinal))
            .collect()
    }

    pub(crate) fn hash_binding(&self) -> Result<[u8; 32], CheckpointError> {
        let bytes = self.canonical_bytes()?;
        Ok(sha256_256_role(
            CheckpointShaRole::Trace,
            &[SOURCE_RECORD_HASH_LABEL_V2, &bytes],
        ))
    }

    /// Exact FIPS message length and compression count for this source record.
    pub(crate) fn hash_geometry(&self) -> Result<(u64, u64), CheckpointError> {
        let bytes = self.canonical_bytes()?;
        let part_bytes = u64::try_from(SOURCE_RECORD_HASH_LABEL_V2.len())
            .map_err(|_| CheckpointError::Limit)?
            .checked_add(u64::try_from(bytes.len()).map_err(|_| CheckpointError::Limit)?)
            .ok_or(CheckpointError::Overflow)?;
        let message_bytes = CheckpointSha256BlockStreamV2::framed_bytes_for_parts(
            CheckpointShaRole::Trace,
            part_bytes,
            2,
        )?;
        Ok((
            message_bytes,
            CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(message_bytes)?,
        ))
    }

    fn decode_canonical(
        bytes: &[u8],
        profile: &RecursiveCircuitProfileV2,
    ) -> Result<Self, CheckpointError> {
        let (opcode, ordinal, object_id, payload) = decode_canonical_source_record(bytes, profile)?;
        Self::new(ordinal, opcode, object_id, payload, profile)
    }
}

/// The one canonical source-record encoder used by spool writes, source SHA,
/// global SHA chunks and test construction.
fn encode_canonical_source_record(
    opcode: RecursiveTraceOpcodeV2,
    ordinal: u64,
    object_id: [u8; 32],
    payload: &[u8],
) -> Result<Vec<u8>, CheckpointError> {
    let payload_len = u32::try_from(payload.len()).map_err(|_| CheckpointError::Limit)?;
    let mut bytes = Vec::with_capacity(
        TRACE_EVENT_HEADER_BYTES_V2
            .checked_add(payload.len())
            .ok_or(CheckpointError::Overflow)?,
    );
    bytes.push(opcode as u8);
    bytes.extend_from_slice(&ordinal.to_le_bytes());
    bytes.extend_from_slice(&object_id);
    bytes.extend_from_slice(&payload_len.to_le_bytes());
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

/// The matching canonical source-record parser.  It is deliberately shared
/// with spool replay so a chunk feeder cannot accept a different byte grammar.
fn decode_canonical_source_record(
    bytes: &[u8],
    profile: &RecursiveCircuitProfileV2,
) -> Result<(RecursiveTraceOpcodeV2, u64, [u8; 32], Vec<u8>), CheckpointError> {
    if bytes.len() < TRACE_EVENT_HEADER_BYTES_V2 {
        return Err(CheckpointError::Canonical);
    }
    let opcode = RecursiveTraceOpcodeV2::decode(bytes[0])?;
    let ordinal = u64::from_le_bytes(
        bytes[1..9]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    let object_id = bytes[9..41]
        .try_into()
        .map_err(|_| CheckpointError::Canonical)?;
    let payload_len = usize::try_from(u32::from_le_bytes(
        bytes[41..45]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    ))
    .map_err(|_| CheckpointError::Limit)?;
    if payload_len
        > usize::try_from(profile.max_leaf_bytes()).map_err(|_| CheckpointError::Limit)?
        || bytes.len()
            != TRACE_EVENT_HEADER_BYTES_V2
                .checked_add(payload_len)
                .ok_or(CheckpointError::Overflow)?
    {
        return Err(CheckpointError::Canonical);
    }
    Ok((
        opcode,
        ordinal,
        object_id,
        bytes[TRACE_EVENT_HEADER_BYTES_V2..].to_vec(),
    ))
}

const HASH_CONTROL_SCHEMA_BYTES: usize = 1 + 1 + 1 + 32 + 8 + 8;
const HASH_CONTROL_SOURCE_BINDING_BYTES: usize = 8 + 1 + 32;
const HASH_CONTROL_TRACE_BINDING_BYTES: usize = 8 + 8 + 8 + 8 + 1;
const HASH_CONTROL_UNIQUENESS_LIST_BINDING_BYTES: usize = 1 + 4 + 8;
const HASH_CONTROL_UNIQUENESS_TRANSCRIPT_BINDING_BYTES: usize = 1 + 8;
const HASH_CONTROL_SOURCE_COMMON_BYTES: usize =
    HASH_CONTROL_SCHEMA_BYTES + HASH_CONTROL_SOURCE_BINDING_BYTES;
const HASH_CONTROL_TRACE_COMMON_BYTES: usize =
    HASH_CONTROL_SCHEMA_BYTES + HASH_CONTROL_TRACE_BINDING_BYTES;
const HASH_CONTROL_UNIQUENESS_LIST_COMMON_BYTES: usize =
    HASH_CONTROL_SCHEMA_BYTES + HASH_CONTROL_UNIQUENESS_LIST_BINDING_BYTES;
const HASH_CONTROL_UNIQUENESS_TRANSCRIPT_COMMON_BYTES: usize =
    HASH_CONTROL_SCHEMA_BYTES + HASH_CONTROL_UNIQUENESS_TRANSCRIPT_BINDING_BYTES;
const HASH_CONTROL_BLOCK_BYTES: usize = 8 + 8 + 64 + 32 + 32 + 1;
const HASH_CONTROL_SOURCE_BLOCK_PAYLOAD_BYTES: usize =
    HASH_CONTROL_SOURCE_COMMON_BYTES + HASH_CONTROL_BLOCK_BYTES;
const HASH_CONTROL_TRACE_BLOCK_PAYLOAD_BYTES: usize =
    HASH_CONTROL_TRACE_COMMON_BYTES + HASH_CONTROL_BLOCK_BYTES;
const HASH_CONTROL_UNIQUENESS_LIST_BLOCK_PAYLOAD_BYTES: usize =
    HASH_CONTROL_UNIQUENESS_LIST_COMMON_BYTES + HASH_CONTROL_BLOCK_BYTES;
const HASH_CONTROL_UNIQUENESS_TRANSCRIPT_BLOCK_PAYLOAD_BYTES: usize =
    HASH_CONTROL_UNIQUENESS_TRANSCRIPT_COMMON_BYTES + HASH_CONTROL_BLOCK_BYTES;
const HASH_CONTROL_ORDINAL_FLAG: u64 = 1_u64 << 63;
const HASH_CONTROL_ORDINAL_STRIDE: u64 = 1_u64 << 24;
const TRACE_HASH_ROLE_TAG_V2: u8 = 1;
const SPENT_ORIGINAL_HASH_ROLE_TAG_V2: u8 = 2;
const OUTPUT_ORIGINAL_HASH_ROLE_TAG_V2: u8 = 3;
const SPENT_SORTED_HASH_ROLE_TAG_V2: u8 = 4;
const OUTPUT_SORTED_HASH_ROLE_TAG_V2: u8 = 5;

/// Frozen control grammar discriminator shared by per-source bindings and
/// the one whole-trace precommit stream.  The byte is explicit so the shared
/// decoder and Nova SHA lane never reinterpret one transcript as the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum HashControlSchemaV2 {
    SourceRecord = 1,
    TracePrecommit = 2,
    UniquenessList = 3,
    UniquenessTranscript = 4,
}

impl HashControlSchemaV2 {
    fn decode(value: u8) -> Result<Self, CheckpointError> {
        match value {
            1 => Ok(Self::SourceRecord),
            2 => Ok(Self::TracePrecommit),
            3 => Ok(Self::UniquenessList),
            4 => Ok(Self::UniquenessTranscript),
            _ => Err(CheckpointError::Canonical),
        }
    }
}

/// Fixed acyclic transcript jobs emitted after the challenge source has been
/// byte-authenticated and before any challenge-dependent product row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum UniquenessTranscriptHashJobV2 {
    DeclaredCounts = 0,
    PreUniquenessContext = 1,
    SpentPrecommit = 2,
    OutputPrecommit = 3,
    SpentPair0Alpha = 4,
    SpentPair0Beta = 5,
    SpentPair1Alpha = 6,
    SpentPair1Beta = 7,
    OutputPair0Alpha = 8,
    OutputPair0Beta = 9,
    OutputPair1Alpha = 10,
    OutputPair1Beta = 11,
    SettlementPreRoot = 12,
    SettlementPostRoot = 13,
}

impl UniquenessTranscriptHashJobV2 {
    pub(crate) const UNIQUENESS_JOB_COUNT: usize = 12;
    pub(crate) const ALL: [Self; 14] = [
        Self::DeclaredCounts,
        Self::PreUniquenessContext,
        Self::SpentPrecommit,
        Self::OutputPrecommit,
        Self::SpentPair0Alpha,
        Self::SpentPair0Beta,
        Self::SpentPair1Alpha,
        Self::SpentPair1Beta,
        Self::OutputPair0Alpha,
        Self::OutputPair0Beta,
        Self::OutputPair1Alpha,
        Self::OutputPair1Beta,
        Self::SettlementPreRoot,
        Self::SettlementPostRoot,
    ];

    fn decode(value: u8) -> Result<Self, CheckpointError> {
        Self::ALL
            .get(usize::from(value))
            .copied()
            .ok_or(CheckpointError::Canonical)
    }

    pub(crate) const fn role(self) -> CheckpointShaRole {
        match self {
            Self::DeclaredCounts => CheckpointShaRole::UniquenessCounts,
            Self::PreUniquenessContext => CheckpointShaRole::UniquenessContext,
            Self::SpentPrecommit | Self::OutputPrecommit => CheckpointShaRole::IdPrecommit,
            Self::SettlementPreRoot | Self::SettlementPostRoot => CheckpointShaRole::SettlementRoot,
            _ => CheckpointShaRole::IdChallenge,
        }
    }

    const fn role_tag(self) -> u8 {
        6 + self as u8
    }

    pub(crate) const fn set(self) -> Option<UniquenessSetKindV2> {
        match self {
            Self::DeclaredCounts
            | Self::PreUniquenessContext
            | Self::SettlementPreRoot
            | Self::SettlementPostRoot => None,
            Self::SpentPrecommit
            | Self::SpentPair0Alpha
            | Self::SpentPair0Beta
            | Self::SpentPair1Alpha
            | Self::SpentPair1Beta => Some(UniquenessSetKindV2::Spent),
            _ => Some(UniquenessSetKindV2::Output),
        }
    }

    pub(crate) const fn challenge_coordinate(self) -> Option<(u8, u8)> {
        match self {
            Self::DeclaredCounts
            | Self::PreUniquenessContext
            | Self::SpentPrecommit
            | Self::OutputPrecommit
            | Self::SettlementPreRoot
            | Self::SettlementPostRoot => None,
            Self::SpentPair0Alpha | Self::OutputPair0Alpha => Some((0, 0)),
            Self::SpentPair0Beta | Self::OutputPair0Beta => Some((0, 1)),
            Self::SpentPair1Alpha | Self::OutputPair1Alpha => Some((1, 0)),
            Self::SpentPair1Beta | Self::OutputPair1Beta => Some((1, 1)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum UniquenessListHashJobV2 {
    SpentOriginal = 0,
    OutputOriginal = 1,
    SpentSorted = 2,
    OutputSorted = 3,
}

impl UniquenessListHashJobV2 {
    pub(crate) const ALL: [Self; 4] = [
        Self::SpentOriginal,
        Self::OutputOriginal,
        Self::SpentSorted,
        Self::OutputSorted,
    ];

    fn decode(value: u8) -> Result<Self, CheckpointError> {
        Self::ALL
            .get(usize::from(value))
            .copied()
            .ok_or(CheckpointError::Canonical)
    }

    pub(crate) const fn role(self) -> CheckpointShaRole {
        match self {
            Self::SpentOriginal => CheckpointShaRole::SpentOriginalIds,
            Self::OutputOriginal => CheckpointShaRole::OutputOriginalIds,
            Self::SpentSorted => CheckpointShaRole::SpentSortedIds,
            Self::OutputSorted => CheckpointShaRole::OutputSortedIds,
        }
    }

    const fn role_tag(self) -> u8 {
        match self {
            Self::SpentOriginal => SPENT_ORIGINAL_HASH_ROLE_TAG_V2,
            Self::OutputOriginal => OUTPUT_ORIGINAL_HASH_ROLE_TAG_V2,
            Self::SpentSorted => SPENT_SORTED_HASH_ROLE_TAG_V2,
            Self::OutputSorted => OUTPUT_SORTED_HASH_ROLE_TAG_V2,
        }
    }

    const fn row(self) -> (UniquenessSetKindV2, UniquenessListKindV2) {
        match self {
            Self::SpentOriginal => (UniquenessSetKindV2::Spent, UniquenessListKindV2::Original),
            Self::OutputOriginal => (UniquenessSetKindV2::Output, UniquenessListKindV2::Original),
            Self::SpentSorted => (UniquenessSetKindV2::Spent, UniquenessListKindV2::Sorted),
            Self::OutputSorted => (UniquenessSetKindV2::Output, UniquenessListKindV2::Sorted),
        }
    }
}

/// Deterministically derive the non-committed byte and SHA schedule for one
/// committed source record. The source digest never includes these events;
/// they are reconstructed from the one committed source record on both passes.
///
/// `BEGIN_HASH` starts the only FIPS transcript before the first source byte.
/// Each fixed-width canonical chunk is then emitted immediately before the
/// compression events it makes available. This ordering is necessary for the
/// Nova relation to derive every compression input with only bounded partial
/// state; it never retains a source-record block tape or treats a chunk as a
/// generic self-loop.
#[cfg(test)]
pub(crate) fn emit_derived_hash_controls(
    source: &RecursiveTraceEventV2,
    profile: &RecursiveCircuitProfileV2,
    mut emit: impl FnMut(RecursiveTraceEventV2) -> Result<(), CheckpointError>,
) -> Result<(), CheckpointError> {
    let mut forward = |event: &RecursiveTraceEventV2| emit(event.clone());
    let mut ignore_chunk =
        |_: RecursiveTraceCanonicalChunkV2, _: &mut _| Ok::<(), CheckpointError>(());
    emit_derived_hash_controls_with_chunk(source, profile, &mut forward, &mut ignore_chunk)
}

/// Emit one source SHA schedule while handing each canonical chunk to the
/// single concurrent whole-trace feeder before the local compression controls
/// that the chunk makes available. The callback receives the same event sink;
/// it cannot create a second source encoding or an uncommitted byte path.
fn emit_derived_hash_controls_with_chunk<F, G>(
    source: &RecursiveTraceEventV2,
    profile: &RecursiveCircuitProfileV2,
    emit: &mut F,
    feed_chunk: &mut G,
) -> Result<(), CheckpointError>
where
    F: FnMut(&RecursiveTraceEventV2) -> Result<(), CheckpointError>,
    G: FnMut(RecursiveTraceCanonicalChunkV2, &mut F) -> Result<(), CheckpointError>,
{
    let binding = source.hash_binding()?;
    let (message_bytes, block_count) = source.hash_geometry()?;
    let begin = source_hash_control_event(
        HashControlStageV2::Begin,
        RecursiveTraceOpcodeV2::BeginHash,
        source,
        binding,
        message_bytes,
        block_count,
        None,
        profile,
    )?;
    emit(&begin)?;

    let source_bytes = source.canonical_len()?;
    let canonical_chunk_count = source.canonical_chunk_count()?;
    let mut stream = CheckpointSha256BlockStreamV2::new(CheckpointShaRole::Trace);
    let mut emitted_blocks = 0_u64;
    stream
        .update_part_with(SOURCE_RECORD_HASH_LABEL_V2, &mut |block| {
            emit_source_hash_block(
                source,
                binding,
                message_bytes,
                block_count,
                profile,
                &mut *emit,
                &mut emitted_blocks,
                block,
            )
        })
        .map_err(map_block_visit_error)?;
    stream
        .begin_part_with(source_bytes, &mut |block| {
            emit_source_hash_block(
                source,
                binding,
                message_bytes,
                block_count,
                profile,
                &mut *emit,
                &mut emitted_blocks,
                block,
            )
        })
        .map_err(map_block_visit_error)?;
    for chunk_ordinal in 0..canonical_chunk_count {
        let chunk = source.canonical_chunk(chunk_ordinal)?;
        let source_memory_write = source_memory_write_control_event(chunk, profile)?;
        emit(&source_memory_write)?;
        let trace_chunk = trace_chunk_control_event(chunk, profile)?;
        emit(&trace_chunk)?;
        feed_chunk(chunk, &mut *emit)?;
        let byte_count = usize::from(chunk.byte_count());
        stream
            .update_part_bytes_with(&chunk.bytes()[..byte_count], &mut |block| {
                emit_source_hash_block(
                    source,
                    binding,
                    message_bytes,
                    block_count,
                    profile,
                    &mut *emit,
                    &mut emitted_blocks,
                    block,
                )
            })
            .map_err(map_block_visit_error)?;
    }
    stream.finish_part().map_err(CheckpointError::from)?;
    let digest = stream
        .finalize_with(&mut |block| {
            emit_source_hash_block(
                source,
                binding,
                message_bytes,
                block_count,
                profile,
                &mut *emit,
                &mut emitted_blocks,
                block,
            )
        })
        .map_err(map_block_visit_error)?;
    if digest != binding || emitted_blocks != block_count {
        return Err(CheckpointError::Invariant);
    }
    let end = source_hash_control_event(
        HashControlStageV2::End,
        RecursiveTraceOpcodeV2::EndHash,
        source,
        binding,
        message_bytes,
        block_count,
        None,
        profile,
    )?;
    emit(&end)
}

#[allow(clippy::too_many_arguments)]
fn emit_source_hash_block(
    source: &RecursiveTraceEventV2,
    binding: [u8; 32],
    message_bytes: u64,
    block_count: u64,
    profile: &RecursiveCircuitProfileV2,
    emit: &mut impl FnMut(&RecursiveTraceEventV2) -> Result<(), CheckpointError>,
    emitted_blocks: &mut u64,
    block: CheckpointSha256BlockV2,
) -> Result<(), CheckpointError> {
    if block.index() != *emitted_blocks {
        return Err(CheckpointError::Invariant);
    }
    let control = source_hash_control_event(
        HashControlStageV2::Block,
        RecursiveTraceOpcodeV2::ShaBlock,
        source,
        binding,
        message_bytes,
        block_count,
        Some(block),
        profile,
    )?;
    emit(&control)?;
    *emitted_blocks = emitted_blocks
        .checked_add(1)
        .ok_or(CheckpointError::Overflow)?;
    Ok(())
}

fn map_block_visit_error(
    error: CheckpointSha256BlockVisitError<CheckpointError>,
) -> CheckpointError {
    match error {
        CheckpointSha256BlockVisitError::Hash(error) => error.into(),
        CheckpointSha256BlockVisitError::Visitor(error) => error,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum HashControlStageV2 {
    Begin = 1,
    Block = 2,
    End = 3,
}

impl HashControlStageV2 {
    fn decode(value: u8) -> Result<Self, CheckpointError> {
        match value {
            1 => Ok(Self::Begin),
            2 => Ok(Self::Block),
            3 => Ok(Self::End),
            _ => Err(CheckpointError::Canonical),
        }
    }
}

fn source_hash_control_event(
    stage: HashControlStageV2,
    opcode: RecursiveTraceOpcodeV2,
    source: &RecursiveTraceEventV2,
    binding: [u8; 32],
    message_bytes: u64,
    block_count: u64,
    block: Option<CheckpointSha256BlockV2>,
    profile: &RecursiveCircuitProfileV2,
) -> Result<RecursiveTraceEventV2, CheckpointError> {
    if matches!(stage, HashControlStageV2::Block) != block.is_some() {
        return Err(CheckpointError::Invariant);
    }
    let source_ordinal = source.ordinal().to_le_bytes();
    let source_opcode = [source.opcode() as u8];
    let payload_bytes = if block.is_some() {
        HASH_CONTROL_SOURCE_BLOCK_PAYLOAD_BYTES
    } else {
        HASH_CONTROL_SOURCE_COMMON_BYTES
    };
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(payload_bytes)
        .map_err(|_| CheckpointError::Limit)?;
    payload.push(HashControlSchemaV2::SourceRecord as u8);
    payload.push(TRACE_HASH_ROLE_TAG_V2);
    payload.push(stage as u8);
    payload.extend_from_slice(&binding);
    payload.extend_from_slice(&message_bytes.to_le_bytes());
    payload.extend_from_slice(&block_count.to_le_bytes());
    payload.extend_from_slice(&source_ordinal);
    payload.extend_from_slice(&source_opcode);
    payload.extend_from_slice(&source.object_id());
    let sequence = match block {
        Some(block) => block
            .index()
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?,
        None if matches!(stage, HashControlStageV2::Begin) => 0,
        None => block_count
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?,
    };
    if let Some(block) = block {
        payload.extend_from_slice(&block.index().to_le_bytes());
        payload.extend_from_slice(&block.byte_offset().to_le_bytes());
        payload.extend_from_slice(block.block());
        append_sha_state(&mut payload, block.chaining_before());
        append_sha_state(&mut payload, block.chaining_after());
        payload.push(u8::from(block.final_block()));
    }
    let ordinal = hash_control_ordinal(source.ordinal(), sequence)?;
    let object_id = structural_event_id(opcode, ordinal, &payload);
    RecursiveTraceEventV2::new(ordinal, opcode, object_id, payload, profile)
}

/// Derive the fixed whole-trace FIPS geometry from the precommitted canonical
/// source records. This is shared by the live expander and never accepts a
/// caller-supplied padding or length claim.
fn trace_hash_geometry(
    precommit: RecursiveTracePrecommitV2,
) -> Result<(u64, u64, u64, u64), CheckpointError> {
    let message_bytes = CheckpointSha256BlockStreamV2::framed_bytes_for_parts(
        CheckpointShaRole::Trace,
        precommit.byte_count,
        precommit.event_count,
    )
    .map_err(|_| CheckpointError::Limit)?;
    let block_count = CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(message_bytes)
        .map_err(|_| CheckpointError::Limit)?;
    let padding_bytes = sha256_padding_bytes(message_bytes, block_count)?;
    let bit_length = message_bytes
        .checked_mul(8)
        .ok_or(CheckpointError::Overflow)?;
    if precommit.event_count == 0 || block_count == 0 {
        return Err(CheckpointError::Invariant);
    }
    Ok((message_bytes, block_count, padding_bytes, bit_length))
}

fn emit_trace_hash_block(
    block: CheckpointSha256BlockV2,
    precommit: RecursiveTracePrecommitV2,
    message_bytes: u64,
    block_count: u64,
    padding_bytes: u64,
    bit_length: u64,
    profile: &RecursiveCircuitProfileV2,
    emit: &mut impl FnMut(&RecursiveTraceEventV2) -> Result<(), CheckpointError>,
    emitted_blocks: &mut u64,
) -> Result<(), CheckpointError> {
    if block.index() != *emitted_blocks {
        return Err(CheckpointError::Invariant);
    }
    let control = trace_hash_control_event(
        HashControlStageV2::Block,
        RecursiveTraceOpcodeV2::ShaBlock,
        precommit,
        message_bytes,
        block_count,
        padding_bytes,
        bit_length,
        Some(block),
        profile,
    )?;
    emit(&control)?;
    *emitted_blocks = emitted_blocks
        .checked_add(1)
        .ok_or(CheckpointError::Overflow)?;
    Ok(())
}

fn trace_hash_control_event(
    stage: HashControlStageV2,
    opcode: RecursiveTraceOpcodeV2,
    precommit: RecursiveTracePrecommitV2,
    message_bytes: u64,
    block_count: u64,
    padding_bytes: u64,
    bit_length: u64,
    block: Option<CheckpointSha256BlockV2>,
    profile: &RecursiveCircuitProfileV2,
) -> Result<RecursiveTraceEventV2, CheckpointError> {
    if matches!(stage, HashControlStageV2::Block) != block.is_some() {
        return Err(CheckpointError::Invariant);
    }
    let payload_bytes = if block.is_some() {
        HASH_CONTROL_TRACE_BLOCK_PAYLOAD_BYTES
    } else {
        HASH_CONTROL_TRACE_COMMON_BYTES
    };
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(payload_bytes)
        .map_err(|_| CheckpointError::Limit)?;
    payload.push(HashControlSchemaV2::TracePrecommit as u8);
    payload.push(TRACE_HASH_ROLE_TAG_V2);
    payload.push(stage as u8);
    payload.extend_from_slice(&precommit.trace_digest);
    payload.extend_from_slice(&message_bytes.to_le_bytes());
    payload.extend_from_slice(&block_count.to_le_bytes());
    payload.extend_from_slice(&precommit.event_count.to_le_bytes());
    payload.extend_from_slice(&precommit.byte_count.to_le_bytes());
    payload.extend_from_slice(&padding_bytes.to_le_bytes());
    payload.extend_from_slice(&bit_length.to_le_bytes());
    payload.push(1);
    let sequence = match block {
        Some(block) => block
            .index()
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?,
        None if matches!(stage, HashControlStageV2::Begin) => 0,
        None => block_count
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?,
    };
    if let Some(block) = block {
        payload.extend_from_slice(&block.index().to_le_bytes());
        payload.extend_from_slice(&block.byte_offset().to_le_bytes());
        payload.extend_from_slice(block.block());
        append_sha_state(&mut payload, block.chaining_before());
        append_sha_state(&mut payload, block.chaining_after());
        payload.push(u8::from(block.final_block()));
    }
    let ordinal = hash_control_ordinal(precommit.event_count, sequence)?;
    let object_id = structural_event_id(opcode, ordinal, &payload);
    RecursiveTraceEventV2::new(ordinal, opcode, object_id, payload, profile)
}

fn sha256_padding_bytes(message_bytes: u64, block_count: u64) -> Result<u64, CheckpointError> {
    block_count
        .checked_mul(64)
        .and_then(|bytes| bytes.checked_sub(message_bytes))
        .and_then(|bytes| bytes.checked_sub(9))
        .filter(|bytes| *bytes < 64)
        .ok_or(CheckpointError::Invariant)
}

pub(crate) fn hash_control_ordinal(
    source_ordinal: u64,
    sequence: u64,
) -> Result<u64, CheckpointError> {
    if sequence >= HASH_CONTROL_ORDINAL_STRIDE {
        return Err(CheckpointError::Limit);
    }
    HASH_CONTROL_ORDINAL_FLAG
        .checked_add(
            source_ordinal
                .checked_mul(HASH_CONTROL_ORDINAL_STRIDE)
                .and_then(|value| value.checked_add(sequence))
                .ok_or(CheckpointError::Overflow)?,
        )
        .ok_or(CheckpointError::Overflow)
}

fn append_sha_state(payload: &mut Vec<u8>, state: &[u32; 8]) {
    for word in state {
        payload.extend_from_slice(&word.to_be_bytes());
    }
}

fn uniqueness_list_hash_control_event(
    stage: HashControlStageV2,
    job: UniquenessListHashJobV2,
    count: u32,
    expected_digest: [u8; 32],
    trace_event_count: u64,
    message_bytes: u64,
    block_count: u64,
    block: Option<CheckpointSha256BlockV2>,
    profile: &RecursiveCircuitProfileV2,
) -> Result<RecursiveTraceEventV2, CheckpointError> {
    if matches!(stage, HashControlStageV2::Block) != block.is_some() {
        return Err(CheckpointError::Invariant);
    }
    let payload_bytes = if block.is_some() {
        HASH_CONTROL_UNIQUENESS_LIST_BLOCK_PAYLOAD_BYTES
    } else {
        HASH_CONTROL_UNIQUENESS_LIST_COMMON_BYTES
    };
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(payload_bytes)
        .map_err(|_| CheckpointError::Limit)?;
    payload.push(HashControlSchemaV2::UniquenessList as u8);
    payload.push(job.role_tag());
    payload.push(stage as u8);
    payload.extend_from_slice(&expected_digest);
    payload.extend_from_slice(&message_bytes.to_le_bytes());
    payload.extend_from_slice(&block_count.to_le_bytes());
    payload.push(job as u8);
    payload.extend_from_slice(&count.to_le_bytes());
    payload.extend_from_slice(&trace_event_count.to_le_bytes());
    let sequence = match block {
        Some(block) => block
            .index()
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?,
        None if stage == HashControlStageV2::Begin => 0,
        None => block_count
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?,
    };
    if let Some(block) = block {
        payload.extend_from_slice(&block.index().to_le_bytes());
        payload.extend_from_slice(&block.byte_offset().to_le_bytes());
        payload.extend_from_slice(block.block());
        append_sha_state(&mut payload, block.chaining_before());
        append_sha_state(&mut payload, block.chaining_after());
        payload.push(u8::from(block.final_block()));
    }
    let synthetic_source_ordinal = trace_event_count
        .checked_add(1)
        .and_then(|value| value.checked_add(u64::from(job as u8)))
        .ok_or(CheckpointError::Overflow)?;
    let ordinal = hash_control_ordinal(synthetic_source_ordinal, sequence)?;
    let opcode = match stage {
        HashControlStageV2::Begin => RecursiveTraceOpcodeV2::BeginHash,
        HashControlStageV2::Block => RecursiveTraceOpcodeV2::ShaBlock,
        HashControlStageV2::End => RecursiveTraceOpcodeV2::EndHash,
    };
    let object_id = structural_event_id(opcode, ordinal, &payload);
    RecursiveTraceEventV2::new(ordinal, opcode, object_id, payload, profile)
}

struct ActiveUniquenessListHashV2 {
    job: UniquenessListHashJobV2,
    expected_digest: [u8; 32],
    expected_count: u32,
    seen: u32,
    message_bytes: u64,
    block_count: u64,
    emitted_blocks: u64,
    stream: Option<CheckpointSha256BlockStreamV2>,
}

#[derive(Default)]
struct UniquenessListHashScheduleV2 {
    precommit: Option<UniquenessPrecommitV2>,
    next_job: usize,
    active: Option<ActiveUniquenessListHashV2>,
}

impl UniquenessListHashScheduleV2 {
    fn before_source(
        &mut self,
        event: &RecursiveTraceEventV2,
        trace_event_count: u64,
        profile: &RecursiveCircuitProfileV2,
        emit: &mut impl FnMut(&RecursiveTraceEventV2) -> Result<(), CheckpointError>,
    ) -> Result<(), CheckpointError> {
        match event.opcode() {
            RecursiveTraceOpcodeV2::UniquenessSorted => {
                let (pass, set, list, _) = decode_uniqueness_sorted_row(event.payload())?;
                if pass != UniquenessPassV2::Commit {
                    return Ok(());
                }
                let target = UniquenessListHashJobV2::ALL
                    .iter()
                    .position(|job| job.row() == (set, list))
                    .ok_or(CheckpointError::Canonical)?;
                self.finish_empty_jobs_before(target, trace_event_count, profile, emit)?;
                if self.active.is_none() {
                    self.start_job(trace_event_count, profile, emit)?;
                }
                if self.active.as_ref().map(|active| active.job as usize) != Some(target) {
                    return Err(CheckpointError::EventOrder);
                }
            }
            RecursiveTraceOpcodeV2::UniquenessChallenge => {
                self.finish_empty_jobs_before(
                    UniquenessListHashJobV2::ALL.len(),
                    trace_event_count,
                    profile,
                    emit,
                )?;
                if self.active.is_some() || self.next_job != UniquenessListHashJobV2::ALL.len() {
                    return Err(CheckpointError::EventOrder);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn after_source(
        &mut self,
        event: &RecursiveTraceEventV2,
        trace_event_count: u64,
        profile: &RecursiveCircuitProfileV2,
        emit: &mut impl FnMut(&RecursiveTraceEventV2) -> Result<(), CheckpointError>,
    ) -> Result<(), CheckpointError> {
        match event.opcode() {
            RecursiveTraceOpcodeV2::UniquenessPrecommit => {
                if self.precommit.is_some() || self.active.is_some() || self.next_job != 0 {
                    return Err(CheckpointError::EventOrder);
                }
                self.precommit = Some(decode_uniqueness_precommit(event.payload())?);
            }
            RecursiveTraceOpcodeV2::UniquenessSorted => {
                let (pass, set, list, row) = decode_uniqueness_sorted_row(event.payload())?;
                if pass != UniquenessPassV2::Commit {
                    return Ok(());
                }
                let active = self.active.as_mut().ok_or(CheckpointError::EventOrder)?;
                if active.job.row() != (set, list) || active.seen >= active.expected_count {
                    return Err(CheckpointError::EventOrder);
                }
                let stream = active.stream.as_mut().ok_or(CheckpointError::Invariant)?;
                stream
                    .update_part_with(&row.canonical_bytes(), &mut |block| {
                        let control = uniqueness_list_hash_control_event(
                            HashControlStageV2::Block,
                            active.job,
                            active.expected_count,
                            active.expected_digest,
                            trace_event_count,
                            active.message_bytes,
                            active.block_count,
                            Some(block),
                            profile,
                        )?;
                        emit(&control)?;
                        active.emitted_blocks = active
                            .emitted_blocks
                            .checked_add(1)
                            .ok_or(CheckpointError::Overflow)?;
                        Ok(())
                    })
                    .map_err(map_block_visit_error)?;
                active.seen = active
                    .seen
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                if active.seen == active.expected_count {
                    self.finish_active(trace_event_count, profile, emit)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn finish_empty_jobs_before(
        &mut self,
        target: usize,
        trace_event_count: u64,
        profile: &RecursiveCircuitProfileV2,
        emit: &mut impl FnMut(&RecursiveTraceEventV2) -> Result<(), CheckpointError>,
    ) -> Result<(), CheckpointError> {
        while self.next_job < target && self.active.is_none() {
            let (_, count) = self.job_binding(self.next_job)?;
            if count != 0 {
                return Err(CheckpointError::EventOrder);
            }
            self.start_job(trace_event_count, profile, emit)?;
            self.finish_active(trace_event_count, profile, emit)?;
        }
        Ok(())
    }

    fn start_job(
        &mut self,
        trace_event_count: u64,
        profile: &RecursiveCircuitProfileV2,
        emit: &mut impl FnMut(&RecursiveTraceEventV2) -> Result<(), CheckpointError>,
    ) -> Result<(), CheckpointError> {
        if self.active.is_some() {
            return Err(CheckpointError::EventOrder);
        }
        let job = *UniquenessListHashJobV2::ALL
            .get(self.next_job)
            .ok_or(CheckpointError::EventOrder)?;
        let (expected_digest, expected_count) = self.job_binding(self.next_job)?;
        let part_bytes = u64::from(expected_count)
            .checked_mul(UNIQUENESS_SEMANTIC_ROW_BYTES_V2 as u64)
            .and_then(|value| value.checked_add(4))
            .ok_or(CheckpointError::Overflow)?;
        let part_count = u64::from(expected_count)
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        let message_bytes = CheckpointSha256BlockStreamV2::framed_bytes_for_parts(
            job.role(),
            part_bytes,
            part_count,
        )?;
        let block_count =
            CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(message_bytes)?;
        let begin = uniqueness_list_hash_control_event(
            HashControlStageV2::Begin,
            job,
            expected_count,
            expected_digest,
            trace_event_count,
            message_bytes,
            block_count,
            None,
            profile,
        )?;
        emit(&begin)?;
        let mut active = ActiveUniquenessListHashV2 {
            job,
            expected_digest,
            expected_count,
            seen: 0,
            message_bytes,
            block_count,
            emitted_blocks: 0,
            stream: Some(CheckpointSha256BlockStreamV2::new(job.role())),
        };
        active
            .stream
            .as_mut()
            .ok_or(CheckpointError::Invariant)?
            .update_part_with(&expected_count.to_le_bytes(), &mut |block| {
                let control = uniqueness_list_hash_control_event(
                    HashControlStageV2::Block,
                    active.job,
                    active.expected_count,
                    active.expected_digest,
                    trace_event_count,
                    active.message_bytes,
                    active.block_count,
                    Some(block),
                    profile,
                )?;
                emit(&control)?;
                active.emitted_blocks = active
                    .emitted_blocks
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                Ok(())
            })
            .map_err(map_block_visit_error)?;
        self.active = Some(active);
        Ok(())
    }

    fn finish_active(
        &mut self,
        trace_event_count: u64,
        profile: &RecursiveCircuitProfileV2,
        emit: &mut impl FnMut(&RecursiveTraceEventV2) -> Result<(), CheckpointError>,
    ) -> Result<(), CheckpointError> {
        let mut active = self.active.take().ok_or(CheckpointError::EventOrder)?;
        if active.seen != active.expected_count {
            return Err(CheckpointError::EventOrder);
        }
        let digest = active
            .stream
            .take()
            .ok_or(CheckpointError::Invariant)?
            .finalize_with(&mut |block| {
                let control = uniqueness_list_hash_control_event(
                    HashControlStageV2::Block,
                    active.job,
                    active.expected_count,
                    active.expected_digest,
                    trace_event_count,
                    active.message_bytes,
                    active.block_count,
                    Some(block),
                    profile,
                )?;
                emit(&control)?;
                active.emitted_blocks = active
                    .emitted_blocks
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                Ok(())
            })
            .map_err(map_block_visit_error)?;
        if digest != active.expected_digest || active.emitted_blocks != active.block_count {
            return Err(CheckpointError::Invariant);
        }
        let end = uniqueness_list_hash_control_event(
            HashControlStageV2::End,
            active.job,
            active.expected_count,
            active.expected_digest,
            trace_event_count,
            active.message_bytes,
            active.block_count,
            None,
            profile,
        )?;
        emit(&end)?;
        self.next_job = self
            .next_job
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        Ok(())
    }

    fn job_binding(&self, index: usize) -> Result<([u8; 32], u32), CheckpointError> {
        let precommit = self.precommit.ok_or(CheckpointError::EventOrder)?;
        match UniquenessListHashJobV2::ALL
            .get(index)
            .copied()
            .ok_or(CheckpointError::EventOrder)?
        {
            UniquenessListHashJobV2::SpentOriginal => {
                Ok((precommit.spent_original_digest, precommit.spent_count))
            }
            UniquenessListHashJobV2::OutputOriginal => {
                Ok((precommit.output_original_digest, precommit.output_count))
            }
            UniquenessListHashJobV2::SpentSorted => {
                Ok((precommit.spent_sorted_digest, precommit.spent_count))
            }
            UniquenessListHashJobV2::OutputSorted => {
                Ok((precommit.output_sorted_digest, precommit.output_count))
            }
        }
    }
}

fn uniqueness_transcript_hash_control_event(
    stage: HashControlStageV2,
    job: UniquenessTranscriptHashJobV2,
    expected_digest: [u8; 32],
    trace_event_count: u64,
    message_bytes: u64,
    block_count: u64,
    block: Option<CheckpointSha256BlockV2>,
    profile: &RecursiveCircuitProfileV2,
) -> Result<RecursiveTraceEventV2, CheckpointError> {
    if matches!(stage, HashControlStageV2::Block) != block.is_some() {
        return Err(CheckpointError::Invariant);
    }
    let payload_bytes = if block.is_some() {
        HASH_CONTROL_UNIQUENESS_TRANSCRIPT_BLOCK_PAYLOAD_BYTES
    } else {
        HASH_CONTROL_UNIQUENESS_TRANSCRIPT_COMMON_BYTES
    };
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(payload_bytes)
        .map_err(|_| CheckpointError::Limit)?;
    payload.push(HashControlSchemaV2::UniquenessTranscript as u8);
    payload.push(job.role_tag());
    payload.push(stage as u8);
    payload.extend_from_slice(&expected_digest);
    payload.extend_from_slice(&message_bytes.to_le_bytes());
    payload.extend_from_slice(&block_count.to_le_bytes());
    payload.push(job as u8);
    payload.extend_from_slice(&trace_event_count.to_le_bytes());
    let sequence = match block {
        Some(block) => block
            .index()
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?,
        None if stage == HashControlStageV2::Begin => 0,
        None => block_count
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?,
    };
    if let Some(block) = block {
        payload.extend_from_slice(&block.index().to_le_bytes());
        payload.extend_from_slice(&block.byte_offset().to_le_bytes());
        payload.extend_from_slice(block.block());
        append_sha_state(&mut payload, block.chaining_before());
        append_sha_state(&mut payload, block.chaining_after());
        payload.push(u8::from(block.final_block()));
    }
    let synthetic_source_ordinal = trace_event_count
        .checked_add(1 + UniquenessListHashJobV2::ALL.len() as u64)
        .and_then(|value| value.checked_add(u64::from(job as u8)))
        .ok_or(CheckpointError::Overflow)?;
    let ordinal = hash_control_ordinal(synthetic_source_ordinal, sequence)?;
    let opcode = match stage {
        HashControlStageV2::Begin => RecursiveTraceOpcodeV2::BeginHash,
        HashControlStageV2::Block => RecursiveTraceOpcodeV2::ShaBlock,
        HashControlStageV2::End => RecursiveTraceOpcodeV2::EndHash,
    };
    RecursiveTraceEventV2::new(
        ordinal,
        opcode,
        structural_event_id(opcode, ordinal, &payload),
        payload,
        profile,
    )
}

struct UniquenessTranscriptHashScheduleV2 {
    context: Option<RecursivePreUniquenessContextV2>,
    precommit: Option<UniquenessPrecommitV2>,
    post_definition_root: Option<[u8; 32]>,
    next_job: usize,
}

impl UniquenessTranscriptHashScheduleV2 {
    const fn new(context: Option<RecursivePreUniquenessContextV2>) -> Self {
        Self {
            context,
            precommit: None,
            post_definition_root: None,
            next_job: 0,
        }
    }

    fn after_source(
        &mut self,
        event: &RecursiveTraceEventV2,
        trace_event_count: u64,
        profile: &RecursiveCircuitProfileV2,
        emit: &mut impl FnMut(&RecursiveTraceEventV2) -> Result<(), CheckpointError>,
    ) -> Result<(), CheckpointError> {
        match event.opcode() {
            RecursiveTraceOpcodeV2::UniquenessPrecommit => {
                if self.precommit.is_some() || self.next_job != 0 {
                    return Err(CheckpointError::EventOrder);
                }
                self.precommit = Some(decode_uniqueness_precommit(event.payload())?);
                let context = self.context.ok_or(CheckpointError::Authority)?;
                for job in [
                    UniquenessTranscriptHashJobV2::DeclaredCounts,
                    UniquenessTranscriptHashJobV2::PreUniquenessContext,
                ] {
                    self.emit_job(
                        job,
                        context,
                        self.precommit.ok_or(CheckpointError::EventOrder)?,
                        None,
                        None,
                        None,
                        RecursiveTraceOpcodeV2::grammar_digest(),
                        trace_event_count,
                        profile,
                        emit,
                    )?;
                    self.next_job = self
                        .next_job
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
            }
            RecursiveTraceOpcodeV2::UniquenessChallenge => {
                if self.next_job != 2 {
                    return Err(CheckpointError::EventOrder);
                }
                let precommit = self.precommit.ok_or(CheckpointError::EventOrder)?;
                let grammar = RecursiveTraceOpcodeV2::grammar_digest();
                let challenges = decode_uniqueness_challenge(
                    event.payload(),
                    self.context.ok_or(CheckpointError::Authority)?.digest(),
                    grammar,
                    precommit,
                )?;
                for job in UniquenessTranscriptHashJobV2::ALL
                    .into_iter()
                    .take(UniquenessTranscriptHashJobV2::UNIQUENESS_JOB_COUNT)
                    .skip(2)
                {
                    self.emit_job(
                        job,
                        self.context.ok_or(CheckpointError::Authority)?,
                        precommit,
                        Some(challenges),
                        None,
                        None,
                        grammar,
                        trace_event_count,
                        profile,
                        emit,
                    )?;
                    self.next_job = self
                        .next_job
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
            }
            RecursiveTraceOpcodeV2::NetMerge
                if self.next_job != UniquenessTranscriptHashJobV2::UNIQUENESS_JOB_COUNT =>
            {
                return Err(CheckpointError::EventOrder);
            }
            RecursiveTraceOpcodeV2::PromoteChildRoot => {
                if self.post_definition_root.is_some()
                    || self.next_job != UniquenessTranscriptHashJobV2::UNIQUENESS_JOB_COUNT
                {
                    return Err(CheckpointError::EventOrder);
                }
                let (definition_root, update_trace_digest) =
                    decode_hierarchy_promotion_fields(event.payload())?;
                if update_trace_digest
                    != self
                        .context
                        .ok_or(CheckpointError::Authority)?
                        .update_trace_digest()
                {
                    return Err(CheckpointError::Canonical);
                }
                self.post_definition_root = Some(definition_root);
            }
            RecursiveTraceOpcodeV2::FinalizeBlock => {
                if self.next_job != UniquenessTranscriptHashJobV2::UNIQUENESS_JOB_COUNT {
                    return Err(CheckpointError::EventOrder);
                }
                let context = self.context.ok_or(CheckpointError::Authority)?;
                let precommit = self.precommit.ok_or(CheckpointError::EventOrder)?;
                let post_definition_root = self
                    .post_definition_root
                    .ok_or(CheckpointError::EventOrder)?;
                let header = decode_flow_header(event.payload())?;
                for job in [
                    UniquenessTranscriptHashJobV2::SettlementPreRoot,
                    UniquenessTranscriptHashJobV2::SettlementPostRoot,
                ] {
                    self.emit_job(
                        job,
                        context,
                        precommit,
                        None,
                        Some(post_definition_root),
                        Some(header.post_root),
                        RecursiveTraceOpcodeV2::grammar_digest(),
                        trace_event_count,
                        profile,
                        emit,
                    )?;
                    self.next_job = self
                        .next_job
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_job(
        &self,
        job: UniquenessTranscriptHashJobV2,
        context: RecursivePreUniquenessContextV2,
        precommit: UniquenessPrecommitV2,
        challenges: Option<UniquenessChallengesV2>,
        post_definition_root: Option<[u8; 32]>,
        post_settlement_root: Option<[u8; 32]>,
        grammar: [u8; 32],
        trace_event_count: u64,
        profile: &RecursiveCircuitProfileV2,
        emit: &mut impl FnMut(&RecursiveTraceEventV2) -> Result<(), CheckpointError>,
    ) -> Result<(), CheckpointError> {
        let (expected, parts) = transcript_job_parts(
            job,
            context,
            precommit,
            challenges,
            post_definition_root,
            post_settlement_root,
            grammar,
        )?;
        let part_bytes = parts.iter().try_fold(0_u64, |total, part| {
            total
                .checked_add(u64::try_from(part.len()).map_err(|_| CheckpointError::Limit)?)
                .ok_or(CheckpointError::Overflow)
        })?;
        let message_bytes = CheckpointSha256BlockStreamV2::framed_bytes_for_parts(
            job.role(),
            part_bytes,
            u64::try_from(parts.len()).map_err(|_| CheckpointError::Limit)?,
        )?;
        let block_count =
            CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(message_bytes)?;
        emit(&uniqueness_transcript_hash_control_event(
            HashControlStageV2::Begin,
            job,
            expected,
            trace_event_count,
            message_bytes,
            block_count,
            None,
            profile,
        )?)?;
        let mut stream = CheckpointSha256BlockStreamV2::new(job.role());
        let mut emitted_blocks = 0_u64;
        for part in &parts {
            stream
                .update_part_with(part, &mut |block| {
                    emit(&uniqueness_transcript_hash_control_event(
                        HashControlStageV2::Block,
                        job,
                        expected,
                        trace_event_count,
                        message_bytes,
                        block_count,
                        Some(block),
                        profile,
                    )?)?;
                    emitted_blocks = emitted_blocks
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                    Ok(())
                })
                .map_err(map_block_visit_error)?;
        }
        let digest = stream
            .finalize_with(&mut |block| {
                emit(&uniqueness_transcript_hash_control_event(
                    HashControlStageV2::Block,
                    job,
                    expected,
                    trace_event_count,
                    message_bytes,
                    block_count,
                    Some(block),
                    profile,
                )?)?;
                emitted_blocks = emitted_blocks
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                Ok(())
            })
            .map_err(map_block_visit_error)?;
        if digest != expected || emitted_blocks != block_count {
            return Err(CheckpointError::Invariant);
        }
        emit(&uniqueness_transcript_hash_control_event(
            HashControlStageV2::End,
            job,
            expected,
            trace_event_count,
            message_bytes,
            block_count,
            None,
            profile,
        )?)
    }
}

/// Emit one exact settlement-root transcript job for sibling R1CS tests.
/// The production schedule and its private `emit_job` owner remain the only
/// encoder; this test hook merely exposes that canonical output without
/// reimplementing its framing or SHA block stream.
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_settlement_transcript_hash_controls_for_test(
    job: UniquenessTranscriptHashJobV2,
    context: RecursivePreUniquenessContextV2,
    precommit: UniquenessPrecommitV2,
    post_definition_root: [u8; 32],
    post_settlement_root: [u8; 32],
    trace_event_count: u64,
    profile: &RecursiveCircuitProfileV2,
    emit: &mut impl FnMut(&RecursiveTraceEventV2) -> Result<(), CheckpointError>,
) -> Result<(), CheckpointError> {
    if !matches!(
        job,
        UniquenessTranscriptHashJobV2::SettlementPreRoot
            | UniquenessTranscriptHashJobV2::SettlementPostRoot
    ) {
        return Err(CheckpointError::Invariant);
    }
    UniquenessTranscriptHashScheduleV2::new(Some(context)).emit_job(
        job,
        context,
        precommit,
        None,
        Some(post_definition_root),
        Some(post_settlement_root),
        RecursiveTraceOpcodeV2::grammar_digest(),
        trace_event_count,
        profile,
        emit,
    )
}

fn transcript_job_parts(
    job: UniquenessTranscriptHashJobV2,
    context: RecursivePreUniquenessContextV2,
    precommit: UniquenessPrecommitV2,
    challenges: Option<UniquenessChallengesV2>,
    post_definition_root: Option<[u8; 32]>,
    post_settlement_root: Option<[u8; 32]>,
    grammar: [u8; 32],
) -> Result<([u8; 32], Vec<Vec<u8>>), CheckpointError> {
    if job == UniquenessTranscriptHashJobV2::DeclaredCounts {
        return Ok((
            context.declared_work().digest(),
            context.declared_work().transcript_parts(),
        ));
    }
    if job == UniquenessTranscriptHashJobV2::PreUniquenessContext {
        return Ok((context.digest(), context.transcript_parts()));
    }
    if job == UniquenessTranscriptHashJobV2::SettlementPreRoot {
        return Ok((
            context.old_settlement_root(),
            context.settlement_root_transcript_parts(context.old_definition_root()),
        ));
    }
    if job == UniquenessTranscriptHashJobV2::SettlementPostRoot {
        let definition_root = post_definition_root.ok_or(CheckpointError::EventOrder)?;
        return Ok((
            post_settlement_root.ok_or(CheckpointError::EventOrder)?,
            context.settlement_root_transcript_parts(definition_root),
        ));
    }
    let challenges = challenges.ok_or(CheckpointError::EventOrder)?;
    let set_kind = job.set().ok_or(CheckpointError::EventOrder)?;
    let set = [set_kind as u8];
    if job.challenge_coordinate().is_none() {
        let (expected, count, original, sorted) = match set_kind {
            UniquenessSetKindV2::Spent => (
                challenges.spent_precommit,
                precommit.spent_count,
                precommit.spent_original_digest,
                precommit.spent_sorted_digest,
            ),
            UniquenessSetKindV2::Output => (
                challenges.output_precommit,
                precommit.output_count,
                precommit.output_original_digest,
                precommit.output_sorted_digest,
            ),
        };
        return Ok((
            expected,
            vec![
                challenges.context.to_vec(),
                set.to_vec(),
                count.to_le_bytes().to_vec(),
                original.to_vec(),
                sorted.to_vec(),
            ],
        ));
    }
    let (pair, coordinate) = job
        .challenge_coordinate()
        .expect("challenge job has a fixed coordinate");
    let (set_precommit, expected) = match set_kind {
        UniquenessSetKindV2::Spent => (
            challenges.spent_precommit,
            challenges.spent[usize::from(pair) * 2 + usize::from(coordinate)],
        ),
        UniquenessSetKindV2::Output => (
            challenges.output_precommit,
            challenges.output[usize::from(pair) * 2 + usize::from(coordinate)],
        ),
    };
    Ok((
        expected,
        vec![
            set_precommit.to_vec(),
            grammar.to_vec(),
            set.to_vec(),
            vec![pair],
            vec![coordinate],
        ],
    ))
}

/// Decoded expected SHA-control binding used by the independent evaluator.
pub(crate) struct HashControlBindingV2 {
    pub(crate) schema: HashControlSchemaV2,
    pub(crate) role: u8,
    pub(crate) stage: HashControlStageV2,
    pub(crate) binding: [u8; 32],
    // Retained decoded mirrors for the existing per-source evaluator.  They
    // are meaningful only for `SourceRecord`; TracePrecommit uses the tagged
    // `trace` binding below.
    pub(crate) source_ordinal: u64,
    pub(crate) source_opcode: RecursiveTraceOpcodeV2,
    pub(crate) source_object_id: [u8; 32],
    pub(crate) source_hash: [u8; 32],
    pub(crate) message_bytes: u64,
    pub(crate) block_count: u64,
    pub(crate) source: Option<HashControlSourceBindingV2>,
    pub(crate) trace: Option<HashControlTraceBindingV2>,
    pub(crate) uniqueness_list: Option<HashControlUniquenessListBindingV2>,
    pub(crate) uniqueness_transcript: Option<HashControlUniquenessTranscriptBindingV2>,
    pub(crate) block: Option<HashControlBlockV2>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HashControlSourceBindingV2 {
    pub(crate) ordinal: u64,
    pub(crate) opcode: RecursiveTraceOpcodeV2,
    pub(crate) object_id: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HashControlTraceBindingV2 {
    pub(crate) event_count: u64,
    pub(crate) byte_count: u64,
    pub(crate) padding_bytes: u64,
    pub(crate) bit_length: u64,
    pub(crate) eof: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HashControlUniquenessListBindingV2 {
    pub(crate) job: UniquenessListHashJobV2,
    pub(crate) count: u32,
    pub(crate) trace_event_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HashControlUniquenessTranscriptBindingV2 {
    pub(crate) job: UniquenessTranscriptHashJobV2,
    pub(crate) trace_event_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HashControlBlockV2 {
    pub(crate) index: u64,
    pub(crate) byte_offset: u64,
    pub(crate) block: [u8; 64],
    pub(crate) chaining_before: [u32; 8],
    pub(crate) chaining_after: [u32; 8],
    pub(crate) final_block: bool,
}

impl HashControlBlockV2 {
    pub(crate) fn verifies_transition(&self) -> bool {
        CheckpointSha256BlockV2::verify_transition_parts(
            &self.block,
            &self.chaining_before,
            &self.chaining_after,
        )
    }
}

pub(crate) fn decode_hash_control(
    event: &RecursiveTraceEventV2,
) -> Result<HashControlBindingV2, CheckpointError> {
    let payload = event.payload();
    if payload.len() < HASH_CONTROL_SCHEMA_BYTES {
        return Err(CheckpointError::Canonical);
    }
    let schema = HashControlSchemaV2::decode(payload[0])?;
    let role = payload[1];
    let stage = HashControlStageV2::decode(payload[2])?;
    let binding = payload[3..35]
        .try_into()
        .map_err(|_| CheckpointError::Canonical)?;
    let message_bytes = u64::from_le_bytes(
        payload[35..43]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    let block_count = u64::from_le_bytes(
        payload[43..51]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    let (common_bytes, source, trace, uniqueness_list, uniqueness_transcript) = match schema {
        HashControlSchemaV2::SourceRecord => {
            if role != TRACE_HASH_ROLE_TAG_V2 {
                return Err(CheckpointError::Canonical);
            }
            if payload.len() < HASH_CONTROL_SOURCE_COMMON_BYTES {
                return Err(CheckpointError::Canonical);
            }
            let ordinal = u64::from_le_bytes(
                payload[51..59]
                    .try_into()
                    .map_err(|_| CheckpointError::Canonical)?,
            );
            let opcode = RecursiveTraceOpcodeV2::decode(payload[59])?;
            if !opcode.is_source_record() {
                return Err(CheckpointError::Canonical);
            }
            let object_id = payload[60..92]
                .try_into()
                .map_err(|_| CheckpointError::Canonical)?;
            (
                HASH_CONTROL_SOURCE_COMMON_BYTES,
                Some(HashControlSourceBindingV2 {
                    ordinal,
                    opcode,
                    object_id,
                }),
                None,
                None,
                None,
            )
        }
        HashControlSchemaV2::TracePrecommit => {
            if role != TRACE_HASH_ROLE_TAG_V2 {
                return Err(CheckpointError::Canonical);
            }
            if payload.len() < HASH_CONTROL_TRACE_COMMON_BYTES {
                return Err(CheckpointError::Canonical);
            }
            let event_count = u64::from_le_bytes(
                payload[51..59]
                    .try_into()
                    .map_err(|_| CheckpointError::Canonical)?,
            );
            let byte_count = u64::from_le_bytes(
                payload[59..67]
                    .try_into()
                    .map_err(|_| CheckpointError::Canonical)?,
            );
            let padding_bytes = u64::from_le_bytes(
                payload[67..75]
                    .try_into()
                    .map_err(|_| CheckpointError::Canonical)?,
            );
            let bit_length = u64::from_le_bytes(
                payload[75..83]
                    .try_into()
                    .map_err(|_| CheckpointError::Canonical)?,
            );
            let eof = match payload[83] {
                0 => false,
                1 => true,
                _ => return Err(CheckpointError::Canonical),
            };
            (
                HASH_CONTROL_TRACE_COMMON_BYTES,
                None,
                Some(HashControlTraceBindingV2 {
                    event_count,
                    byte_count,
                    padding_bytes,
                    bit_length,
                    eof,
                }),
                None,
                None,
            )
        }
        HashControlSchemaV2::UniquenessList => {
            if payload.len() < HASH_CONTROL_UNIQUENESS_LIST_COMMON_BYTES {
                return Err(CheckpointError::Canonical);
            }
            let job = UniquenessListHashJobV2::decode(payload[51])?;
            if role != job.role_tag() {
                return Err(CheckpointError::Canonical);
            }
            let count = u32::from_le_bytes(
                payload[52..56]
                    .try_into()
                    .map_err(|_| CheckpointError::Canonical)?,
            );
            let trace_event_count = u64::from_le_bytes(
                payload[56..64]
                    .try_into()
                    .map_err(|_| CheckpointError::Canonical)?,
            );
            (
                HASH_CONTROL_UNIQUENESS_LIST_COMMON_BYTES,
                None,
                None,
                Some(HashControlUniquenessListBindingV2 {
                    job,
                    count,
                    trace_event_count,
                }),
                None,
            )
        }
        HashControlSchemaV2::UniquenessTranscript => {
            if payload.len() < HASH_CONTROL_UNIQUENESS_TRANSCRIPT_COMMON_BYTES {
                return Err(CheckpointError::Canonical);
            }
            let job = UniquenessTranscriptHashJobV2::decode(payload[51])?;
            if role != job.role_tag() {
                return Err(CheckpointError::Canonical);
            }
            let trace_event_count = u64::from_le_bytes(
                payload[52..60]
                    .try_into()
                    .map_err(|_| CheckpointError::Canonical)?,
            );
            (
                HASH_CONTROL_UNIQUENESS_TRANSCRIPT_COMMON_BYTES,
                None,
                None,
                None,
                Some(HashControlUniquenessTranscriptBindingV2 {
                    job,
                    trace_event_count,
                }),
            )
        }
    };
    let block = match stage {
        HashControlStageV2::Block => {
            if payload.len()
                != common_bytes
                    .checked_add(HASH_CONTROL_BLOCK_BYTES)
                    .ok_or(CheckpointError::Overflow)?
            {
                return Err(CheckpointError::Canonical);
            }
            let index = u64::from_le_bytes(
                payload[common_bytes..common_bytes + 8]
                    .try_into()
                    .map_err(|_| CheckpointError::Canonical)?,
            );
            let byte_offset = u64::from_le_bytes(
                payload[common_bytes + 8..common_bytes + 16]
                    .try_into()
                    .map_err(|_| CheckpointError::Canonical)?,
            );
            let block = payload[common_bytes + 16..common_bytes + 80]
                .try_into()
                .map_err(|_| CheckpointError::Canonical)?;
            let chaining_before =
                decode_sha_state(&payload[common_bytes + 80..common_bytes + 112])?;
            let chaining_after =
                decode_sha_state(&payload[common_bytes + 112..common_bytes + 144])?;
            let final_block = match payload[common_bytes + 144] {
                0 => false,
                1 => true,
                _ => return Err(CheckpointError::Canonical),
            };
            Some(HashControlBlockV2 {
                index,
                byte_offset,
                block,
                chaining_before,
                chaining_after,
                final_block,
            })
        }
        HashControlStageV2::Begin | HashControlStageV2::End => {
            if payload.len() != common_bytes {
                return Err(CheckpointError::Canonical);
            }
            None
        }
    };
    Ok(HashControlBindingV2 {
        schema,
        role,
        stage,
        binding,
        source_ordinal: source
            .map(|binding| binding.ordinal)
            .or_else(|| trace.map(|binding| binding.event_count))
            .or_else(|| {
                uniqueness_list
                    .map(|binding| binding.trace_event_count + 1 + u64::from(binding.job as u8))
            })
            .unwrap_or_else(|| {
                let binding = uniqueness_transcript
                    .expect("uniqueness-transcript binding is present for its schema");
                binding.trace_event_count
                    + 1
                    + UniquenessListHashJobV2::ALL.len() as u64
                    + u64::from(binding.job as u8)
            }),
        source_opcode: source
            .map(|binding| binding.opcode)
            .unwrap_or(RecursiveTraceOpcodeV2::BeginBlock),
        source_object_id: source
            .map(|binding| binding.object_id)
            .unwrap_or([0_u8; 32]),
        source_hash: binding,
        message_bytes,
        block_count,
        source,
        trace,
        uniqueness_list,
        uniqueness_transcript,
        block,
    })
}

fn decode_sha_state(bytes: &[u8]) -> Result<[u32; 8], CheckpointError> {
    if bytes.len() != 32 {
        return Err(CheckpointError::Canonical);
    }
    let mut state = [0_u32; 8];
    for (slot, word) in state.iter_mut().zip(bytes.chunks_exact(4)) {
        *slot = u32::from_be_bytes(word.try_into().map_err(|_| CheckpointError::Canonical)?);
    }
    Ok(state)
}

/// Values fixed by pass one and independently regenerated by pass two.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursiveTracePrecommitV2 {
    event_count: u64,
    byte_count: u64,
    trace_digest: [u8; 32],
    spent_original_ids_digest: [u8; 32],
    spent_sorted_ids_digest: [u8; 32],
    output_original_ids_digest: [u8; 32],
    output_sorted_ids_digest: [u8; 32],
}

impl RecursiveTracePrecommitV2 {
    #[must_use]
    pub const fn event_count(&self) -> u64 {
        self.event_count
    }
    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }
    #[must_use]
    pub const fn trace_digest(&self) -> [u8; 32] {
        self.trace_digest
    }
    #[must_use]
    pub const fn spent_original_ids_digest(&self) -> [u8; 32] {
        self.spent_original_ids_digest
    }
    #[must_use]
    pub const fn spent_sorted_ids_digest(&self) -> [u8; 32] {
        self.spent_sorted_ids_digest
    }
    #[must_use]
    pub const fn output_original_ids_digest(&self) -> [u8; 32] {
        self.output_original_ids_digest
    }
    #[must_use]
    pub const fn output_sorted_ids_digest(&self) -> [u8; 32] {
        self.output_sorted_ids_digest
    }
}

/// Completion values from the second pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursiveTracePassV2 {
    precommit: RecursiveTracePrecommitV2,
    event_counts: RecursiveTraceEventCountsV2,
}

impl RecursiveTracePassV2 {
    #[must_use]
    pub const fn precommit(&self) -> RecursiveTracePrecommitV2 {
        self.precommit
    }

    /// Return the independently counted fixed schedule emitted by this pass.
    #[must_use]
    pub const fn event_counts(&self) -> RecursiveTraceEventCountsV2 {
        self.event_counts
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TraceSourceStateV2 {
    Open,
    Precommitted,
    Replayed,
    Finished,
}

/// Sole private-spool owner for a recursive V2 transition trace.
pub struct RecursiveTransitionTraceSourceV2 {
    profile: RecursiveCircuitProfileV2,
    snapshot: RecursiveSnapshotHandleV2,
    pre_uniqueness_context: Option<RecursivePreUniquenessContextV2>,
    spool_dir: PathBuf,
    spool: PrivateSpoolFile,
    state: TraceSourceStateV2,
    precommit_builder: Option<TracePrecommitBuilderV2>,
    precommit: Option<RecursiveTracePrecommitV2>,
}

impl RecursiveTransitionTraceSourceV2 {
    /// Open one V2 source under an immutable snapshot handle.
    pub(crate) fn create_in(
        dir: impl AsRef<Path>,
        profile: RecursiveCircuitProfileV2,
        snapshot: RecursiveSnapshotHandleV2,
    ) -> Result<Self, CheckpointError> {
        let spool_dir = dir.as_ref().to_path_buf();
        Ok(Self {
            spool: PrivateSpoolFile::create_in(&spool_dir, profile.max_content_bytes())?,
            profile,
            snapshot,
            pre_uniqueness_context: None,
            spool_dir,
            state: TraceSourceStateV2::Open,
            precommit_builder: None,
            precommit: None,
        })
    }

    #[must_use]
    pub(crate) const fn snapshot(&self) -> RecursiveSnapshotHandleV2 {
        self.snapshot
    }

    #[must_use]
    pub(crate) const fn profile(&self) -> &RecursiveCircuitProfileV2 {
        &self.profile
    }

    pub(crate) fn bind_pre_uniqueness_context(
        &mut self,
        context: RecursivePreUniquenessContextV2,
    ) -> Result<(), CheckpointError> {
        if self.state != TraceSourceStateV2::Open || self.precommit_builder.is_some() {
            return Err(CheckpointError::TraceState);
        }
        self.pre_uniqueness_context = Some(context);
        Ok(())
    }

    #[must_use]
    pub(crate) const fn pre_uniqueness_context(&self) -> Option<RecursivePreUniquenessContextV2> {
        self.pre_uniqueness_context
    }

    /// Start streaming canonical events into the first pass.
    pub(crate) fn begin_canonical_precommit(&mut self) -> Result<(), CheckpointError> {
        if self.state != TraceSourceStateV2::Open || self.precommit_builder.is_some() {
            return Err(CheckpointError::TraceState);
        }
        self.spool.verify_integrity()?;
        self.precommit_builder = Some(TracePrecommitBuilderV2::new(
            &self.spool_dir,
            &self.profile,
        )?);
        Ok(())
    }

    /// Append exactly one source event; this is reachable only from the
    /// canonical storage orchestrator and retains no event tape in memory.
    pub(crate) fn append_canonical_event(
        &mut self,
        event: RecursiveTraceEventV2,
    ) -> Result<(), CheckpointError> {
        if self.state != TraceSourceStateV2::Open {
            return Err(CheckpointError::TraceState);
        }
        let builder = self
            .precommit_builder
            .as_mut()
            .ok_or(CheckpointError::TraceState)?;
        builder.append(event, &self.profile, &mut self.spool)
    }

    /// Seal the streamed first pass before challenge-dependent replay begins.
    pub(crate) fn seal_canonical_precommit(
        &mut self,
    ) -> Result<RecursiveTracePrecommitV2, CheckpointError> {
        if self.state != TraceSourceStateV2::Open {
            return Err(CheckpointError::TraceState);
        }
        let builder = self
            .precommit_builder
            .take()
            .ok_or(CheckpointError::TraceState)?;
        let precommit = builder.finish()?;
        self.spool.rewind()?;
        self.precommit = Some(precommit);
        self.state = TraceSourceStateV2::Precommitted;
        Ok(precommit)
    }

    /// Return the sealed first-pass commitment before its one replay pass.
    ///
    /// The native evaluator uses these external-sort commitments instead of
    /// retaining a second in-memory replay-ID representation.
    pub(crate) fn sealed_precommit(&self) -> Result<RecursiveTracePrecommitV2, CheckpointError> {
        if self.state != TraceSourceStateV2::Precommitted {
            return Err(CheckpointError::TraceState);
        }
        self.precommit.ok_or(CheckpointError::TraceState)
    }

    #[cfg(test)]
    fn precommit_from_canonical_events(
        &mut self,
        events: &[RecursiveTraceEventV2],
    ) -> Result<RecursiveTracePrecommitV2, CheckpointError> {
        self.begin_canonical_precommit()?;
        for event in events {
            self.append_canonical_event(event.clone())?;
        }
        self.seal_canonical_precommit()
    }

    /// Replay exactly the precommitted spool, invoking `visit` once per event.
    #[cfg(test)]
    pub(crate) fn event_pass(
        &mut self,
        mut visit: impl FnMut(&RecursiveTraceEventV2) -> Result<(), CheckpointError>,
    ) -> Result<RecursiveTracePassV2, CheckpointError> {
        self.event_pass_with_source_context(|event, _source| visit(event))
    }

    /// Replay the sealed source while retaining a borrow of the active source
    /// record for its immediately derived controls.  The extra context is
    /// available only synchronously during this pass, preventing an evaluator
    /// from copying a per-record chunk tape while preserving the one canonical
    /// schedule consumed by Nova.
    pub(crate) fn event_pass_with_source_context(
        &mut self,
        mut visit: impl FnMut(
            &RecursiveTraceEventV2,
            Option<&RecursiveTraceEventV2>,
        ) -> Result<(), CheckpointError>,
    ) -> Result<RecursiveTracePassV2, CheckpointError> {
        if !matches!(
            self.state,
            TraceSourceStateV2::Precommitted | TraceSourceStateV2::Replayed
        ) {
            return Err(CheckpointError::TraceState);
        }
        if self.state == TraceSourceStateV2::Replayed {
            self.spool.rewind()?;
        }
        let precommit = self.precommit.ok_or(CheckpointError::TraceState)?;
        let mut event_counts = RecursiveTraceEventCountsV2::default();
        let mut visit_counted =
            |event: &RecursiveTraceEventV2, source_record: Option<&RecursiveTraceEventV2>| {
                event_counts.increment(event.opcode())?;
                visit(event, source_record)
            };
        // The descriptor identity and exact bounded length were captured at
        // exclusive creation and checked again after fsync/rewind.  Recheck at
        // the pass boundary so a truncate, append, replacement, hard-link, or
        // permission change cannot be replayed as the precommitted source.
        self.spool.verify_integrity()?;
        let (trace_message_bytes, trace_block_count, trace_padding_bytes, trace_bit_length) =
            trace_hash_geometry(precommit)?;
        visit_counted(
            &trace_hash_control_event(
                HashControlStageV2::Begin,
                RecursiveTraceOpcodeV2::BeginHash,
                precommit,
                trace_message_bytes,
                trace_block_count,
                trace_padding_bytes,
                trace_bit_length,
                None,
                &self.profile,
            )?,
            None,
        )?;
        // This stream is started once and receives the very same chunks that
        // feed the source-local transcript below. It is never rebuilt from a
        // rewind, a block tape, or a second source encoder.
        let mut trace_stream = CheckpointSha256BlockStreamV2::new(CheckpointShaRole::Trace);
        let mut trace_blocks = 0_u64;
        let mut trace = CheckpointSha256V2::new(CheckpointShaRole::Trace);
        let mut identifiers = IdentifierPrecommitV2::new(&self.spool_dir, &self.profile)?;
        let mut uniqueness_hashes = UniquenessListHashScheduleV2::default();
        let context = self.pre_uniqueness_context;
        let mut uniqueness_transcript = UniquenessTranscriptHashScheduleV2::new(context);
        let mut event_count = 0_u64;
        let mut byte_count = 0_u64;
        while let Some(bytes) = read_spooled_trace_record(&mut self.spool, &self.profile)? {
            let total = bytes.len();
            let event = RecursiveTraceEventV2::decode_canonical(&bytes, &self.profile)?;
            if !event.opcode().is_source_record() {
                return Err(CheckpointError::Canonical);
            }
            if event.ordinal() != event_count {
                return Err(CheckpointError::EventOrder);
            }
            event_count = event_count
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
            byte_count = byte_count
                .checked_add(u64::try_from(total).map_err(|_| CheckpointError::Limit)?)
                .ok_or(CheckpointError::Overflow)?;
            if event.opcode().is_source_record() {
                trace.update_part(&bytes)?;
            }
            identifiers.absorb(&event)?;
            let uniqueness_before = uniqueness_hashes.before_source(
                &event,
                precommit.event_count,
                &self.profile,
                &mut |control| visit_counted(control, None),
            );
            #[cfg(test)]
            if let Err(error) = &uniqueness_before {
                eprintln!(
                    "recursive source replay rejected uniqueness-before ordinal={} opcode={:?}: {error:?}",
                    event.ordinal(),
                    event.opcode(),
                );
            }
            uniqueness_before?;
            visit_counted(&event, None)?;
            let record_bytes = event.canonical_len()?;
            trace_stream
                .begin_part_with(record_bytes, &mut |block| {
                    emit_trace_hash_block(
                        block,
                        precommit,
                        trace_message_bytes,
                        trace_block_count,
                        trace_padding_bytes,
                        trace_bit_length,
                        &self.profile,
                        &mut |control| visit_counted(control, Some(&event)),
                        &mut trace_blocks,
                    )
                })
                .map_err(map_block_visit_error)?;
            let mut feed_trace_chunk = |chunk: RecursiveTraceCanonicalChunkV2, emit: &mut _| {
                let byte_count = usize::from(chunk.byte_count());
                trace_stream
                    .update_part_bytes_with(&chunk.bytes()[..byte_count], &mut |block| {
                        emit_trace_hash_block(
                            block,
                            precommit,
                            trace_message_bytes,
                            trace_block_count,
                            trace_padding_bytes,
                            trace_bit_length,
                            &self.profile,
                            emit,
                            &mut trace_blocks,
                        )
                    })
                    .map_err(map_block_visit_error)
            };
            let mut emit_derived =
                |control: &RecursiveTraceEventV2| visit_counted(control, Some(&event));
            emit_derived_hash_controls_with_chunk(
                &event,
                &self.profile,
                &mut emit_derived,
                &mut feed_trace_chunk,
            )?;
            let uniqueness_after = uniqueness_hashes.after_source(
                &event,
                precommit.event_count,
                &self.profile,
                &mut |control| visit_counted(control, None),
            );
            #[cfg(test)]
            if let Err(error) = &uniqueness_after {
                eprintln!(
                    "recursive source replay rejected uniqueness-after ordinal={} opcode={:?}: {error:?}",
                    event.ordinal(),
                    event.opcode(),
                );
            }
            uniqueness_after?;
            let transcript_after = uniqueness_transcript.after_source(
                &event,
                precommit.event_count,
                &self.profile,
                &mut |control| visit_counted(control, None),
            );
            #[cfg(test)]
            if let Err(error) = &transcript_after {
                eprintln!(
                    "recursive source replay rejected transcript-after ordinal={} opcode={:?}: {error:?}",
                    event.ordinal(),
                    event.opcode(),
                );
            }
            transcript_after?;
            trace_stream.finish_part().map_err(CheckpointError::from)?;
        }
        let replayed = RecursiveTracePrecommitV2 {
            event_count,
            byte_count,
            trace_digest: trace.finalize(),
            ..identifiers.finish()?
        };
        self.spool.verify_integrity()?;
        if replayed != precommit || byte_count != self.spool.len() {
            return Err(CheckpointError::Canonical);
        }
        let trace_digest = trace_stream
            .finalize_with(&mut |block| {
                emit_trace_hash_block(
                    block,
                    precommit,
                    trace_message_bytes,
                    trace_block_count,
                    trace_padding_bytes,
                    trace_bit_length,
                    &self.profile,
                    &mut |control| visit_counted(control, None),
                    &mut trace_blocks,
                )
            })
            .map_err(map_block_visit_error)?;
        if trace_digest != precommit.trace_digest || trace_blocks != trace_block_count {
            #[cfg(test)]
            eprintln!(
                "recursive source replay rejected trace-finalize: digest_match={} blocks={trace_blocks}/{trace_block_count}",
                trace_digest == precommit.trace_digest,
            );
            return Err(CheckpointError::Invariant);
        }
        visit_counted(
            &trace_hash_control_event(
                HashControlStageV2::End,
                RecursiveTraceOpcodeV2::EndHash,
                precommit,
                trace_message_bytes,
                trace_block_count,
                trace_padding_bytes,
                trace_bit_length,
                None,
                &self.profile,
            )?,
            None,
        )?;
        self.state = TraceSourceStateV2::Replayed;
        Ok(RecursiveTracePassV2 {
            precommit,
            event_counts,
        })
    }

    /// Close the source only after the caller proves the same handle remains live.
    pub(crate) fn finish(
        &mut self,
        observed_snapshot: RecursiveSnapshotHandleV2,
    ) -> Result<RecursiveTracePrecommitV2, CheckpointError> {
        if self.state != TraceSourceStateV2::Replayed || observed_snapshot != self.snapshot {
            return Err(CheckpointError::SnapshotChanged);
        }
        self.spool.verify_integrity()?;
        self.state = TraceSourceStateV2::Finished;
        self.precommit.ok_or(CheckpointError::TraceState)
    }
}

/// Mutable state of the strictly one-way first source pass.
struct TracePrecommitBuilderV2 {
    expected_ordinal: u64,
    event_count: u64,
    byte_count: u64,
    trace: CheckpointSha256V2,
    identifiers: IdentifierPrecommitV2,
}

impl TracePrecommitBuilderV2 {
    fn new(dir: &Path, profile: &RecursiveCircuitProfileV2) -> Result<Self, CheckpointError> {
        Ok(Self {
            expected_ordinal: 0,
            event_count: 0,
            byte_count: 0,
            trace: CheckpointSha256V2::new(CheckpointShaRole::Trace),
            identifiers: IdentifierPrecommitV2::new(dir, profile)?,
        })
    }

    fn append(
        &mut self,
        event: RecursiveTraceEventV2,
        profile: &RecursiveCircuitProfileV2,
        spool: &mut PrivateSpoolFile,
    ) -> Result<(), CheckpointError> {
        if !event.opcode().is_source_record()
            || event.ordinal() != self.expected_ordinal
            || self.event_count >= u64::from(profile.max_typed_events())
        {
            return Err(CheckpointError::EventOrder);
        }
        let encoded = event.canonical_bytes()?;
        let encoded_len = u64::try_from(encoded.len()).map_err(|_| CheckpointError::Limit)?;
        let next_bytes = self
            .byte_count
            .checked_add(encoded_len)
            .ok_or(CheckpointError::Overflow)?;
        if next_bytes > profile.max_content_bytes() {
            return Err(CheckpointError::Limit);
        }
        if event.opcode().is_source_record() {
            self.trace.update_part(&encoded)?;
        }
        self.identifiers.absorb(&event)?;
        spool.write_bounded(&encoded)?;
        self.expected_ordinal = self
            .expected_ordinal
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        self.event_count = self
            .event_count
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        self.byte_count = next_bytes;
        Ok(())
    }

    fn finish(self) -> Result<RecursiveTracePrecommitV2, CheckpointError> {
        if self.event_count == 0 {
            return Err(CheckpointError::TraceState);
        }
        Ok(RecursiveTracePrecommitV2 {
            event_count: self.event_count,
            byte_count: self.byte_count,
            trace_digest: self.trace.finalize(),
            ..self.identifiers.finish()?
        })
    }
}

/// Separate pre-challenge semantic-row commitments for spent and output.
///
/// Only replay rows participate. Reused object IDs in structural/hash/JMT
/// opcodes are valid references and never enter this set.
struct IdentifierPrecommitV2 {
    spent_original: Option<CheckpointSha256V2>,
    output_original: Option<CheckpointSha256V2>,
    spent_count: Option<u32>,
    output_count: Option<u32>,
    spent_sorted: ExternalIdSortV2,
    output_sorted: ExternalIdSortV2,
}

impl IdentifierPrecommitV2 {
    fn new(dir: &Path, profile: &RecursiveCircuitProfileV2) -> Result<Self, CheckpointError> {
        let spent_sort_bytes =
            RecursiveCircuitProfileV2::identifier_sort_bytes(profile.max_spent())?;
        let output_sort_bytes =
            RecursiveCircuitProfileV2::identifier_sort_bytes(profile.max_outputs())?;
        let resident_ids = usize::try_from(profile.resident_buffer_bytes())
            .map_err(|_| CheckpointError::Limit)?
            / (2 * UNIQUENESS_SEMANTIC_ROW_BYTES_V2);
        let max_runs =
            usize::try_from(profile.max_spool_runs()).map_err(|_| CheckpointError::Limit)?;
        let merge_fan_in =
            usize::try_from(profile.spool_merge_fan_in()).map_err(|_| CheckpointError::Limit)?;
        Ok(Self {
            spent_original: None,
            output_original: None,
            spent_count: None,
            output_count: None,
            spent_sorted: ExternalIdSortV2::new(
                dir,
                spent_sort_bytes,
                resident_ids,
                max_runs,
                merge_fan_in,
                CheckpointShaRole::SpentSortedIds,
            )?,
            output_sorted: ExternalIdSortV2::new(
                dir,
                output_sort_bytes,
                resident_ids,
                max_runs,
                merge_fan_in,
                CheckpointShaRole::OutputSortedIds,
            )?,
        })
    }

    fn absorb(&mut self, event: &RecursiveTraceEventV2) -> Result<(), CheckpointError> {
        match event.opcode() {
            RecursiveTraceOpcodeV2::BeginBlock => {
                if self.spent_original.is_some() || self.output_original.is_some() {
                    return Err(CheckpointError::EventOrder);
                }
                let header = decode_flow_header(event.payload())?;
                let mut spent = CheckpointSha256V2::new(CheckpointShaRole::SpentOriginalIds);
                spent.update_part(&header.spent_count.to_le_bytes())?;
                let mut output = CheckpointSha256V2::new(CheckpointShaRole::OutputOriginalIds);
                output.update_part(&header.output_count.to_le_bytes())?;
                self.spent_original = Some(spent);
                self.output_original = Some(output);
                self.spent_count = Some(header.spent_count);
                self.output_count = Some(header.output_count);
            }
            RecursiveTraceOpcodeV2::ReplayInput => {
                let item = decode_flow_item(event.payload())?;
                let row = UniquenessSemanticRowV2 {
                    definition_id: item.definition_id,
                    serial_id: item.serial_id,
                    terminal_id: item.terminal_id,
                    leaf_value_hash: item.leaf_value_hash,
                };
                self.spent_original
                    .as_mut()
                    .ok_or(CheckpointError::EventOrder)?
                    .update_part(&row.canonical_bytes())?;
                self.spent_sorted.push(row)?;
            }
            RecursiveTraceOpcodeV2::ReplayOutput => {
                let item = decode_flow_item(event.payload())?;
                let row = UniquenessSemanticRowV2 {
                    definition_id: item.definition_id,
                    serial_id: item.serial_id,
                    terminal_id: item.terminal_id,
                    leaf_value_hash: item.leaf_value_hash,
                };
                self.output_original
                    .as_mut()
                    .ok_or(CheckpointError::EventOrder)?
                    .update_part(&row.canonical_bytes())?;
                self.output_sorted.push(row)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn finish(mut self) -> Result<RecursiveTracePrecommitV2, CheckpointError> {
        let spent_count = self.spent_count.ok_or(CheckpointError::EventOrder)?;
        let output_count = self.output_count.ok_or(CheckpointError::EventOrder)?;
        let spent_sorted_ids_digest = self.spent_sorted.digest_sorted_unique(spent_count)?;
        let output_sorted_ids_digest = self.output_sorted.digest_sorted_unique(output_count)?;
        if !self
            .spent_sorted
            .cross_set_replacements_are_same_path(&mut self.output_sorted)?
        {
            return Err(CheckpointError::DuplicateIdentifier);
        }

        Ok(RecursiveTracePrecommitV2 {
            event_count: 0,
            byte_count: 0,
            trace_digest: [0; 32],
            spent_original_ids_digest: self
                .spent_original
                .take()
                .ok_or(CheckpointError::EventOrder)?
                .finalize(),
            spent_sorted_ids_digest,
            output_original_ids_digest: self
                .output_original
                .take()
                .ok_or(CheckpointError::EventOrder)?
                .finalize(),
            output_sorted_ids_digest,
        })
    }
}

/// Disk-backed, bounded external sorter for one replay semantic-row list.
///
/// Run files remain private `PrivateSpoolFile`s; the source owns their lifetime
/// and never exposes a filesystem path or handle to checkpoint callers.
struct ExternalIdSortV2 {
    dir: PathBuf,
    byte_budget: u64,
    resident_rows: usize,
    max_runs: usize,
    merge_fan_in: usize,
    digest_role: CheckpointShaRole,
    written: u64,
    pending: Vec<UniquenessSemanticRowV2>,
    runs: Vec<PrivateSpoolFile>,
}

impl ExternalIdSortV2 {
    fn new(
        dir: &Path,
        byte_budget: u64,
        resident_rows: usize,
        max_runs: usize,
        merge_fan_in: usize,
        digest_role: CheckpointShaRole,
    ) -> Result<Self, CheckpointError> {
        if byte_budget < UNIQUENESS_SEMANTIC_ROW_BYTES_V2 as u64
            || resident_rows == 0
            || max_runs == 0
            || merge_fan_in == 0
        {
            return Err(CheckpointError::Limit);
        }
        Ok(Self {
            dir: dir.to_path_buf(),
            byte_budget,
            resident_rows,
            max_runs,
            merge_fan_in,
            digest_role,
            written: 0,
            pending: Vec::with_capacity(resident_rows),
            runs: Vec::new(),
        })
    }

    fn push(&mut self, row: UniquenessSemanticRowV2) -> Result<(), CheckpointError> {
        self.pending.push(row);
        if self.pending.len() == self.resident_rows {
            self.flush_run()?;
        }
        Ok(())
    }

    fn flush_run(&mut self) -> Result<(), CheckpointError> {
        if self.pending.is_empty() {
            return Ok(());
        }
        if self.runs.len() >= self.max_runs {
            return Err(CheckpointError::Limit);
        }
        self.pending.sort_unstable_by_key(|row| row.terminal_id);
        let run_bytes = u64::try_from(self.pending.len())
            .map_err(|_| CheckpointError::Limit)?
            .checked_mul(UNIQUENESS_SEMANTIC_ROW_BYTES_V2 as u64)
            .ok_or(CheckpointError::Overflow)?;
        let next = self
            .written
            .checked_add(run_bytes)
            .ok_or(CheckpointError::Overflow)?;
        if next > self.byte_budget {
            return Err(CheckpointError::Limit);
        }
        let mut run = PrivateSpoolFile::create_in(&self.dir, run_bytes)?;
        for row in self.pending.drain(..) {
            run.write_bounded(&row.canonical_bytes())?;
        }
        run.rewind()?;
        self.written = next;
        self.runs.push(run);
        Ok(())
    }

    fn open_sorted(&mut self) -> Result<SortedIdStreamV2<'_>, CheckpointError> {
        self.flush_run()?;
        if self.runs.len() > self.merge_fan_in {
            return Err(CheckpointError::Limit);
        }
        SortedIdStreamV2::new(&mut self.runs)
    }

    fn digest_sorted_unique(&mut self, expected_count: u32) -> Result<[u8; 32], CheckpointError> {
        let digest_role = self.digest_role;
        let mut stream = self.open_sorted()?;
        let mut previous = None;
        let mut digest = CheckpointSha256V2::new(digest_role);
        digest.update_part(&expected_count.to_le_bytes())?;
        let mut observed_count = 0_u32;
        while let Some(row) = stream.next_row()? {
            if previous == Some(row.terminal_id) {
                return Err(CheckpointError::DuplicateIdentifier);
            }
            digest.update_part(&row.canonical_bytes())?;
            previous = Some(row.terminal_id);
            observed_count = observed_count
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
        }
        if observed_count != expected_count {
            return Err(CheckpointError::Invariant);
        }
        Ok(digest.finalize())
    }

    fn cross_set_replacements_are_same_path(
        &mut self,
        other: &mut Self,
    ) -> Result<bool, CheckpointError> {
        let mut left = self.open_sorted()?;
        let mut right = other.open_sorted()?;
        let mut left_row = left.next_row()?;
        let mut right_row = right.next_row()?;
        while let (Some(current_left), Some(current_right)) = (left_row, right_row) {
            match current_left.terminal_id.cmp(&current_right.terminal_id) {
                std::cmp::Ordering::Less => left_row = left.next_row()?,
                std::cmp::Ordering::Greater => right_row = right.next_row()?,
                std::cmp::Ordering::Equal => {
                    if !current_left.same_storage_path(current_right) {
                        return Ok(false);
                    }
                    left_row = left.next_row()?;
                    right_row = right.next_row()?;
                }
            }
        }
        Ok(true)
    }
}

/// Deterministic bounded k-way merge over private sorted runs.
struct SortedIdStreamV2<'a> {
    runs: &'a mut [PrivateSpoolFile],
    heads: Vec<Option<UniquenessSemanticRowV2>>,
}

impl<'a> SortedIdStreamV2<'a> {
    fn new(runs: &'a mut [PrivateSpoolFile]) -> Result<Self, CheckpointError> {
        let mut heads = Vec::with_capacity(runs.len());
        for run in runs.iter_mut() {
            run.rewind()?;
            heads.push(read_spooled_semantic_row(run)?);
        }
        Ok(Self { runs, heads })
    }

    fn next_row(&mut self) -> Result<Option<UniquenessSemanticRowV2>, CheckpointError> {
        let index = self
            .heads
            .iter()
            .enumerate()
            .filter_map(|(index, row)| row.map(|row| (index, row)))
            .min_by_key(|(_, row)| row.terminal_id)
            .map(|(index, _)| index);
        let Some(index) = index else {
            return Ok(None);
        };
        let row = self.heads[index].take().ok_or(CheckpointError::Canonical)?;
        self.heads[index] = read_spooled_semantic_row(&mut self.runs[index])?;
        Ok(Some(row))
    }
}

fn read_spooled_semantic_row(
    spool: &mut PrivateSpoolFile,
) -> Result<Option<UniquenessSemanticRowV2>, CheckpointError> {
    let mut bytes = [0_u8; UNIQUENESS_SEMANTIC_ROW_BYTES_V2];
    let read = spool.read_chunk(&mut bytes)?;
    if read == 0 {
        return Ok(None);
    }
    spool_read_exact(spool, &mut bytes[read..])?;
    UniquenessSemanticRowV2::from_canonical_bytes(&bytes).map(Some)
}

fn spool_read_exact(
    spool: &mut PrivateSpoolFile,
    mut bytes: &mut [u8],
) -> Result<(), CheckpointError> {
    while !bytes.is_empty() {
        let read = spool.read_chunk(bytes)?;
        if read == 0 {
            return Err(CheckpointError::Canonical);
        }
        bytes = &mut bytes[read..];
    }
    Ok(())
}

/// Read exactly one canonical source record from the sole private spool.
/// Returning `None` is legal only at a record boundary (EOF).
fn read_spooled_trace_record(
    spool: &mut PrivateSpoolFile,
    profile: &RecursiveCircuitProfileV2,
) -> Result<Option<Zeroizing<Vec<u8>>>, CheckpointError> {
    let mut tag = [0_u8; 1];
    if spool.read_chunk(&mut tag)? == 0 {
        return Ok(None);
    }
    let mut rest = [0_u8; TRACE_EVENT_HEADER_BYTES_V2 - 1];
    spool_read_exact(spool, &mut rest)?;
    let payload_len = u32::from_le_bytes(
        rest[40..44]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    );
    let payload_len = usize::try_from(payload_len).map_err(|_| CheckpointError::Limit)?;
    if payload_len
        > usize::try_from(profile.max_leaf_bytes()).map_err(|_| CheckpointError::Limit)?
    {
        return Err(CheckpointError::Limit);
    }
    let total = TRACE_EVENT_HEADER_BYTES_V2
        .checked_add(payload_len)
        .ok_or(CheckpointError::Overflow)?;
    let mut bytes = Zeroizing::new(vec![0_u8; total]);
    bytes[0] = tag[0];
    bytes[1..TRACE_EVENT_HEADER_BYTES_V2].copy_from_slice(&rest);
    spool_read_exact(spool, &mut bytes[TRACE_EVENT_HEADER_BYTES_V2..])?;
    Ok(Some(bytes))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::{
        decode_hash_control, decode_trace_chunk_control, emit_derived_hash_controls,
        emit_derived_trace_chunks, HashControlSchemaV2, HashControlStageV2, RecursiveTraceEventV2,
        RecursiveTraceOpcodeV2, RecursiveTransitionTraceSourceV2, UniquenessListHashJobV2,
        UniquenessTranscriptHashJobV2, TRACE_CANONICAL_CHUNK_BYTES_V2, TRACE_HASH_ROLE_TAG_V2,
    };
    use crate::{
        checkpoint::{
            recursive_circuit::RecursiveCircuitProfileV2,
            recursive_context::RecursiveSnapshotHandleV2, recursive_semantics::encode_flow_item,
        },
        settlement::{
            derive_settlement_root_v2, RootGeneration, ScopeFlowItem, ScopeLeafKind, ScopeOpKind,
            ScopeSeen,
        },
        snapshot::PrepSnapshotId,
    };
    use z00z_crypto::{CheckpointSha256BlockStreamV2, CheckpointShaRole};

    fn profile() -> RecursiveCircuitProfileV2 {
        RecursiveCircuitProfileV2::authority_pinned()
    }

    fn multi_run_profile() -> RecursiveCircuitProfileV2 {
        let source_records = RecursiveCircuitProfileV2::max_source_records(4, 4, 512, 4 * 1024)
            .expect("small source record cap");
        let sha_blocks = RecursiveCircuitProfileV2::sha_blocks_for_complete_trace(
            4 * 1024,
            source_records,
            4,
            4,
        )
        .expect("small complete SHA cap");
        let e_max = RecursiveCircuitProfileV2::event_bound_from_parts(
            4,
            4,
            4,
            8,
            source_records,
            sha_blocks,
            4 * 1024,
        )
        .expect("small E_max");
        RecursiveCircuitProfileV2::new(
            2,
            2,
            2,
            4,
            4,
            4,
            4,
            4,
            8,
            512,
            4 * 1024,
            sha_blocks,
            u32::try_from(e_max).expect("small event cap"),
            e_max,
            400,
            8 * 1024,
            4,
            4,
        )
        .expect("multi-run profile")
    }

    fn snapshot() -> RecursiveSnapshotHandleV2 {
        let root = derive_settlement_root_v2(RootGeneration::SettlementV2, 7, [3; 32], [4; 32])
            .expect("V2 root");
        RecursiveSnapshotHandleV2::new(
            PrepSnapshotId::new([1; 32]),
            9,
            root,
            [4; 32],
            5,
            1,
            [2; 32],
        )
        .expect("immutable snapshot")
    }

    fn event(ordinal: u64, opcode: RecursiveTraceOpcodeV2, id: u8) -> RecursiveTraceEventV2 {
        let payload = match opcode {
            RecursiveTraceOpcodeV2::ReplayInput | RecursiveTraceOpcodeV2::ReplayOutput => {
                encode_flow_item(&ScopeFlowItem {
                    tx_id: format!("trace-{ordinal}"),
                    op_kind: if opcode == RecursiveTraceOpcodeV2::ReplayInput {
                        ScopeOpKind::Delete
                    } else {
                        ScopeOpKind::Put
                    },
                    definition_id: format!("{:02x}", id.wrapping_add(1)).repeat(32),
                    serial_id: u32::from(id),
                    terminal_id: format!("{id:02x}").repeat(32),
                    leaf_value_hash: [id.wrapping_add(2); 32],
                    leaf_family: ScopeLeafKind::Terminal,
                    first_seen: ScopeSeen {
                        definition: false,
                        serial: false,
                        object: false,
                    },
                })
                .expect("canonical replay payload")
            }
            _ => vec![id],
        };
        RecursiveTraceEventV2::new(ordinal, opcode, [id; 32], payload, &profile())
            .expect("bounded event")
    }

    fn begin_block_event(
        ordinal: u64,
        id: u8,
        spent_count: u32,
        output_count: u32,
    ) -> RecursiveTraceEventV2 {
        fn append_hex32(payload: &mut Vec<u8>, byte: u8) {
            let encoded = format!("{byte:02x}").repeat(32);
            payload.extend_from_slice(&(encoded.len() as u16).to_le_bytes());
            payload.extend_from_slice(encoded.as_bytes());
        }

        let mut payload = Vec::new();
        append_hex32(&mut payload, id);
        payload.extend_from_slice(&1_u32.to_le_bytes());
        payload.extend_from_slice(&1_u64.to_le_bytes());
        append_hex32(&mut payload, id.wrapping_add(1));
        append_hex32(&mut payload, id.wrapping_add(2));
        append_hex32(&mut payload, id.wrapping_add(3));
        payload.extend_from_slice(&spent_count.to_le_bytes());
        payload.extend_from_slice(&output_count.to_le_bytes());
        RecursiveTraceEventV2::new(
            ordinal,
            RecursiveTraceOpcodeV2::BeginBlock,
            [id; 32],
            payload,
            &profile(),
        )
        .expect("bounded canonical BeginBlock")
    }

    #[test]
    fn canonical_source_chunks_are_the_single_encoder_view_at_boundaries() {
        for payload_len in [0_usize, 19, 20, 21, 83] {
            let source = RecursiveTraceEventV2::new(
                7,
                RecursiveTraceOpcodeV2::ReplayInput,
                [9; 32],
                vec![0xA5; payload_len],
                &profile(),
            )
            .expect("bounded source event");
            let canonical = source.canonical_bytes().expect("canonical source bytes");
            let chunks = source
                .canonical_chunks_for_test()
                .expect("canonical chunk view");
            assert!(!chunks.is_empty());
            assert_eq!(chunks[0].source_ordinal(), source.ordinal());
            assert!(chunks.windows(2).all(|pair| {
                pair[1].chunk_ordinal() == pair[0].chunk_ordinal() + 1
                    && pair[1].chunk_count() == pair[0].chunk_count()
            }));
            let mut reconstructed = Vec::new();
            for chunk in chunks {
                let bytes = chunk.bytes();
                let byte_count = usize::from(chunk.byte_count());
                assert!((1..=TRACE_CANONICAL_CHUNK_BYTES_V2).contains(&byte_count));
                assert!(bytes[byte_count..].iter().all(|byte| *byte == 0));
                reconstructed.extend_from_slice(&bytes[..byte_count]);
            }
            assert_eq!(reconstructed, canonical);
        }
    }

    #[test]
    fn derived_chunk_controls_have_a_disjoint_ordinal_and_exact_zero_padding() {
        let source = RecursiveTraceEventV2::new(
            3,
            RecursiveTraceOpcodeV2::ReplayInput,
            [7; 32],
            vec![0xA5; 83],
            &profile(),
        )
        .expect("bounded source event");
        let mut controls = Vec::new();
        emit_derived_trace_chunks(&source, &profile(), |control| {
            controls.push(control);
            Ok(())
        })
        .expect("derived chunk controls");
        let source_chunks = source
            .canonical_chunks_for_test()
            .expect("canonical chunk source");
        assert_eq!(controls.len(), source_chunks.len());
        for (control, expected) in controls.iter().zip(source_chunks) {
            let decoded = decode_trace_chunk_control(control).expect("canonical chunk control");
            assert_eq!(decoded.source_ordinal, source.ordinal());
            assert_eq!(decoded.chunk_ordinal, expected.chunk_ordinal());
            assert_eq!(decoded.chunk_count, expected.chunk_count());
            assert_eq!(decoded.byte_count, expected.byte_count());
            assert_eq!(decoded.bytes, expected.bytes());
            assert_ne!(control.ordinal() >> 62, 0, "chunk namespace is reserved");
            assert_eq!(
                control.ordinal() >> 63,
                0,
                "chunk namespace differs from SHA controls"
            );
        }
    }

    #[test]
    fn storage_internal_trace_replays_exactly_once() {
        let temp = TempDir::new().expect("temp dir");
        let handle = snapshot();
        let mut source =
            RecursiveTransitionTraceSourceV2::create_in(temp.path(), profile(), handle)
                .expect("trace source");
        let events = [
            begin_block_event(0, 1, 1, 1),
            event(1, RecursiveTraceOpcodeV2::ReplayInput, 2),
            event(2, RecursiveTraceOpcodeV2::ReplayOutput, 3),
        ];
        let expected_hash_controls = events
            .iter()
            .map(|event| event.hash_geometry().expect("bounded event geometry").1 + 2)
            .sum::<u64>();
        let expected_chunk_controls = events
            .iter()
            .map(|event| {
                u64::try_from(
                    event
                        .canonical_chunks_for_test()
                        .expect("canonical chunks")
                        .len(),
                )
                .expect("chunk count fits u64")
            })
            .sum::<u64>();
        let precommit = source
            .precommit_from_canonical_events(&events)
            .expect("internal precommit");
        let global_hash_controls = CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(
            CheckpointSha256BlockStreamV2::framed_bytes_for_parts(
                CheckpointShaRole::Trace,
                precommit.byte_count(),
                precommit.event_count(),
            )
            .expect("global trace geometry"),
        )
        .expect("global FIPS block count")
        .checked_add(2)
        .expect("global begin/end controls");
        let mut visited = 0_u64;
        let mut global_controls = Vec::new();
        let mut schedule = Vec::new();
        let replay = source
            .event_pass(|event| {
                visited += 1;
                schedule.push(event.clone());
                if matches!(
                    decode_hash_control(event),
                    Ok(control) if control.schema == HashControlSchemaV2::TracePrecommit
                ) {
                    global_controls.push(event.clone());
                }
                Ok(())
            })
            .expect("exact replay");
        assert_eq!(
            visited,
            precommit
                .event_count()
                .checked_add(expected_chunk_controls)
                .and_then(|count| count.checked_add(expected_chunk_controls))
                .and_then(|count| count.checked_add(expected_hash_controls))
                .and_then(|count| count.checked_add(global_hash_controls))
                .expect("one source record plus its FIPS controls"),
            "the spool commits source records while the sole memory writer, canonical byte reader, and exact FIPS SHA schedule are regenerated"
        );
        for source_event in &events {
            let source_position = schedule
                .iter()
                .position(|event| {
                    event.opcode() == source_event.opcode()
                        && event.ordinal() == source_event.ordinal()
                        && event.object_id() == source_event.object_id()
                })
                .expect("source record in derived schedule");
            let chunks = source_event
                .canonical_chunks_for_test()
                .expect("source chunks");
            let (begin_position, begin) = schedule
                .iter()
                .enumerate()
                .skip(source_position + 1)
                .find_map(|(index, candidate)| {
                    let control = decode_hash_control(candidate).ok()?;
                    (control.schema == HashControlSchemaV2::SourceRecord
                        && control.stage == HashControlStageV2::Begin
                        && control.source_ordinal == source_event.ordinal())
                    .then_some((index, control))
                })
                .expect("source has a following source BEGIN_HASH");
            assert_eq!(begin.schema, HashControlSchemaV2::SourceRecord);
            assert_eq!(begin.stage, HashControlStageV2::Begin);
            assert_eq!(begin.source_ordinal, source_event.ordinal());
            let end_position = schedule
                .iter()
                .enumerate()
                .skip(begin_position + 1)
                .find_map(|(index, event)| {
                    let control = decode_hash_control(event).ok()?;
                    (control.schema == HashControlSchemaV2::SourceRecord
                        && control.stage == HashControlStageV2::End
                        && control.source_ordinal == source_event.ordinal())
                    .then_some(index)
                })
                .expect("source END_HASH in derived schedule");
            let feeder = schedule[begin_position + 1..end_position]
                .iter()
                .filter(|event| event.opcode() == RecursiveTraceOpcodeV2::TraceChunk)
                .collect::<Vec<_>>();
            assert_eq!(feeder.len(), chunks.len());
            for (control, expected) in feeder.into_iter().zip(chunks) {
                let decoded = decode_trace_chunk_control(control).expect("canonical feeder");
                assert_eq!(decoded.source_ordinal, expected.source_ordinal());
                assert_eq!(decoded.chunk_ordinal, expected.chunk_ordinal());
                assert_eq!(decoded.chunk_count, expected.chunk_count());
                assert_eq!(decoded.byte_count, expected.byte_count());
                assert_eq!(decoded.bytes, expected.bytes());
            }
        }
        assert_eq!(replay.precommit(), precommit);
        let decoded_global = global_controls
            .iter()
            .map(|event| decode_hash_control(event).expect("canonical global trace control"))
            .collect::<Vec<_>>();
        assert_eq!(
            decoded_global.first().expect("global begin").stage,
            HashControlStageV2::Begin
        );
        assert_eq!(
            decoded_global.last().expect("global end").stage,
            HashControlStageV2::End
        );
        for control in &decoded_global {
            let trace = control.trace.expect("tagged trace binding");
            assert_eq!(control.binding, precommit.trace_digest());
            assert_eq!(trace.event_count, precommit.event_count());
            assert_eq!(trace.byte_count, precommit.byte_count());
            assert_eq!(trace.bit_length, control.message_bytes * 8);
            assert!(trace.eof);
        }
        let blocks = decoded_global
            .iter()
            .filter_map(|control| control.block)
            .collect::<Vec<_>>();
        assert_eq!(
            u64::try_from(blocks.len()).expect("global control block count"),
            decoded_global[0].block_count
        );
        assert!(blocks.iter().all(|block| block.verifies_transition()));
        assert!(blocks.last().expect("global final block").final_block);
        assert_eq!(source.finish(handle).expect("same snapshot"), precommit);
    }

    #[test]
    fn source_rejects_control_input() {
        let temp = TempDir::new().expect("temp dir");
        let mut source =
            RecursiveTransitionTraceSourceV2::create_in(temp.path(), profile(), snapshot())
                .expect("trace source");
        source
            .begin_canonical_precommit()
            .expect("open canonical source");
        assert!(source
            .append_canonical_event(event(0, RecursiveTraceOpcodeV2::BeginHash, 1))
            .is_err());
    }

    #[test]
    fn sha_control_expansion_carries_every_fips_block_and_rejects_state_mutation() {
        let source = RecursiveTraceEventV2::new(
            0,
            RecursiveTraceOpcodeV2::BeginBlock,
            [7; 32],
            vec![0xA5; 65],
            &profile(),
        )
        .expect("bounded source event");
        let (_, expected_blocks) = source.hash_geometry().expect("source geometry");
        let mut controls = Vec::new();
        emit_derived_hash_controls(&source, &profile(), |control| {
            controls.push(control);
            Ok(())
        })
        .expect("FIPS control expansion");

        let chunks = controls
            .iter()
            .filter(|control| control.opcode() == RecursiveTraceOpcodeV2::TraceChunk)
            .count();
        assert_eq!(
            chunks,
            source
                .canonical_chunks_for_test()
                .expect("canonical chunks")
                .len()
        );
        let decoded = controls
            .iter()
            .filter_map(|control| decode_hash_control(control).ok())
            .collect::<Vec<_>>();
        assert_eq!(
            decoded.first().expect("begin").stage,
            HashControlStageV2::Begin
        );
        assert_eq!(decoded.last().expect("end").stage, HashControlStageV2::End);
        let blocks = decoded
            .iter()
            .filter_map(|control| control.block)
            .collect::<Vec<_>>();
        assert_eq!(
            u64::try_from(blocks.len()).expect("test block count"),
            expected_blocks
        );
        assert!(blocks.iter().all(|block| block.verifies_transition()));
        assert!(blocks.last().expect("final SHA block").final_block);

        let mut mutated = blocks[0];
        mutated.chaining_after[0] ^= 1;
        assert!(
            !mutated.verifies_transition(),
            "the evaluator cannot accept a mutated FIPS chaining state"
        );
    }

    #[test]
    fn storage_internal_trace_rejects_duplicate_spent_ids() {
        let temp = TempDir::new().expect("temp dir");
        let mut source =
            RecursiveTransitionTraceSourceV2::create_in(temp.path(), profile(), snapshot())
                .expect("trace source");
        assert!(source
            .precommit_from_canonical_events(&[
                begin_block_event(0, 1, 2, 0),
                event(1, RecursiveTraceOpcodeV2::ReplayInput, 2),
                event(2, RecursiveTraceOpcodeV2::ReplayInput, 2),
            ])
            .is_err());
    }

    #[test]
    fn external_sort_merges_private_runs_without_retaining_identifier_lists() {
        let temp = TempDir::new().expect("temp dir");
        let mut source = RecursiveTransitionTraceSourceV2::create_in(
            temp.path(),
            multi_run_profile(),
            snapshot(),
        )
        .expect("trace source");
        let events = [
            begin_block_event(0, 10, 4, 4),
            event(1, RecursiveTraceOpcodeV2::ReplayInput, 4),
            event(2, RecursiveTraceOpcodeV2::ReplayInput, 1),
            event(3, RecursiveTraceOpcodeV2::ReplayInput, 3),
            event(4, RecursiveTraceOpcodeV2::ReplayInput, 2),
            event(5, RecursiveTraceOpcodeV2::ReplayOutput, 8),
            event(6, RecursiveTraceOpcodeV2::ReplayOutput, 5),
            event(7, RecursiveTraceOpcodeV2::ReplayOutput, 7),
            event(8, RecursiveTraceOpcodeV2::ReplayOutput, 6),
            event(9, RecursiveTraceOpcodeV2::CommitTypedEvent, 11),
            event(10, RecursiveTraceOpcodeV2::FinalizeBlock, 12),
        ];
        let precommit = source
            .precommit_from_canonical_events(&events)
            .expect("two-run external sort");
        assert_ne!(
            precommit.spent_original_ids_digest(),
            precommit.spent_sorted_ids_digest()
        );
        assert_ne!(
            precommit.output_original_ids_digest(),
            precommit.output_sorted_ids_digest()
        );
    }

    #[test]
    fn secret_canary_stays_redacted() {
        const CANARY: &str = "z00z-recursive-secret-canary-7f64b7c8";
        let error = match RecursiveTraceEventV2::new(
            0,
            RecursiveTraceOpcodeV2::ReplayInput,
            [0; 32],
            CANARY.as_bytes().to_vec(),
            &profile(),
        ) {
            Ok(_) => panic!("zero object identity must reject"),
            Err(error) => error,
        };
        let diagnostic = format!("{error:?} {error}");
        assert!(
            !diagnostic.contains(CANARY),
            "recursive trace errors must never retain source payload bytes"
        );
    }

    #[test]
    fn trace_event_payload_zeroizes_in_place() {
        use zeroize::Zeroize;

        let mut event = RecursiveTraceEventV2::new(
            1,
            RecursiveTraceOpcodeV2::ReplayInput,
            [1; 32],
            vec![0xa5; 97],
            &profile(),
        )
        .expect("bounded secret-boundary fixture");
        event.zeroize();
        assert!(event.payload.is_empty());
    }

    #[test]
    fn hash_job_registry_is_injective() {
        let mut rows = std::collections::BTreeSet::new();
        assert!(rows.insert((
            HashControlSchemaV2::SourceRecord as u8,
            TRACE_HASH_ROLE_TAG_V2,
        )));
        assert!(rows.insert((
            HashControlSchemaV2::TracePrecommit as u8,
            TRACE_HASH_ROLE_TAG_V2,
        )));
        for job in UniquenessListHashJobV2::ALL {
            assert!(rows.insert((HashControlSchemaV2::UniquenessList as u8, job.role_tag(),)));
        }
        for job in UniquenessTranscriptHashJobV2::ALL {
            assert!(rows.insert((
                HashControlSchemaV2::UniquenessTranscript as u8,
                job.role_tag(),
            )));
        }
        assert_eq!(rows.len(), 20, "every theorem hash job has one schema row");
    }
}
