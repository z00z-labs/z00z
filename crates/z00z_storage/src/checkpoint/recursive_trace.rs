//! Rewindable, bounded, two-pass typed trace source for recursive checkpoint V2.

use std::path::{Path, PathBuf};

use z00z_crypto::{
    sha256_256_role, CheckpointSha256BlockStreamV2, CheckpointSha256BlockV2,
    CheckpointSha256BlockVisitError, CheckpointSha256V2, CheckpointShaRole,
};
use z00z_utils::io::PrivateSpoolFile;

use super::{
    recursive_circuit::RecursiveCircuitProfileV2, recursive_context::RecursiveSnapshotHandleV2,
    recursive_reject::RecursiveV2Error,
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
    /// Derived, non-committed canonical-byte feeder for the private SHA lane.
    TraceChunk = 14,
}

impl RecursiveTraceOpcodeV2 {
    /// Commit the complete frozen opcode alphabet in numeric order.
    #[must_use]
    pub(crate) fn grammar_digest() -> [u8; 32] {
        use z00z_crypto::{sha256_256_role, CheckpointShaRole};

        const OPCODES: [u8; 14] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        sha256_256_role(
            CheckpointShaRole::Statement,
            &[b"z00z.recursive.v2.trace-opcode-grammar", &OPCODES],
        )
    }

    fn decode(byte: u8) -> Result<Self, RecursiveV2Error> {
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
            14 => Ok(Self::TraceChunk),
            _ => Err(RecursiveV2Error::Canonical),
        }
    }

    pub(crate) const fn is_source_record(self) -> bool {
        matches!(
            self,
            Self::BeginBlock
                | Self::ReplayInput
                | Self::ReplayOutput
                | Self::UniquenessPrecommit
                | Self::UniquenessChallenge
                | Self::NetMerge
                | Self::JmtUpdate
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
) -> Result<u64, RecursiveV2Error> {
    TRACE_CHUNK_ORDINAL_FLAG_V2
        .checked_add(
            source_ordinal
                .checked_mul(HASH_CONTROL_ORDINAL_STRIDE)
                .and_then(|value| value.checked_add(u64::from(chunk_ordinal)))
                .ok_or(RecursiveV2Error::Overflow)?,
        )
        .ok_or(RecursiveV2Error::Overflow)
}

/// Test helper that materializes fixed-width controls from the canonical source
/// encoder. Production uses [`emit_derived_hash_controls`] so chunks and SHA
/// controls retain their one streaming order.
#[cfg(test)]
pub(crate) fn emit_derived_trace_chunks(
    source: &RecursiveTraceEventV2,
    profile: &RecursiveCircuitProfileV2,
    mut emit: impl FnMut(RecursiveTraceEventV2) -> Result<(), RecursiveV2Error>,
) -> Result<(), RecursiveV2Error> {
    for chunk in source.canonical_chunks()? {
        emit(trace_chunk_control_event(chunk, profile)?)?;
    }
    Ok(())
}

fn trace_chunk_control_event(
    chunk: RecursiveTraceCanonicalChunkV2,
    profile: &RecursiveCircuitProfileV2,
) -> Result<RecursiveTraceEventV2, RecursiveV2Error> {
    let mut payload = Vec::with_capacity(TRACE_CHUNK_CONTROL_PAYLOAD_BYTES_V2);
    payload.push(TRACE_CHUNK_CONTROL_VERSION_V2);
    payload.extend_from_slice(&chunk.source_ordinal().to_le_bytes());
    payload.extend_from_slice(&chunk.chunk_ordinal().to_le_bytes());
    payload.extend_from_slice(&chunk.chunk_count().to_le_bytes());
    payload.push(chunk.byte_count());
    payload.extend_from_slice(&chunk.bytes());
    let ordinal = trace_chunk_control_ordinal(chunk.source_ordinal(), chunk.chunk_ordinal())?;
    let object_id = structural_event_id(RecursiveTraceOpcodeV2::TraceChunk, ordinal, &payload);
    RecursiveTraceEventV2::new(
        ordinal,
        RecursiveTraceOpcodeV2::TraceChunk,
        object_id,
        payload,
        profile,
    )
}

/// Decode and validate one fixed-width canonical-byte feeder control.
pub(crate) fn decode_trace_chunk_control(
    event: &RecursiveTraceEventV2,
) -> Result<RecursiveTraceChunkControlV2, RecursiveV2Error> {
    if event.opcode() != RecursiveTraceOpcodeV2::TraceChunk
        || event.payload().len() != TRACE_CHUNK_CONTROL_PAYLOAD_BYTES_V2
        || event.object_id()
            != structural_event_id(event.opcode(), event.ordinal(), event.payload())
        || event.payload()[0] != TRACE_CHUNK_CONTROL_VERSION_V2
    {
        return Err(RecursiveV2Error::Canonical);
    }
    let payload = event.payload();
    let source_ordinal = u64::from_le_bytes(
        payload[1..9]
            .try_into()
            .map_err(|_| RecursiveV2Error::Canonical)?,
    );
    let chunk_ordinal = u32::from_le_bytes(
        payload[9..13]
            .try_into()
            .map_err(|_| RecursiveV2Error::Canonical)?,
    );
    let chunk_count = u32::from_le_bytes(
        payload[13..17]
            .try_into()
            .map_err(|_| RecursiveV2Error::Canonical)?,
    );
    let byte_count = payload[17];
    let bytes: [u8; TRACE_CANONICAL_CHUNK_BYTES_V2] = payload[18..]
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    if chunk_count == 0
        || chunk_ordinal >= chunk_count
        || byte_count == 0
        || usize::from(byte_count) > TRACE_CANONICAL_CHUNK_BYTES_V2
        || bytes[usize::from(byte_count)..]
            .iter()
            .any(|byte| *byte != 0)
        || event.ordinal() != trace_chunk_control_ordinal(source_ordinal, chunk_ordinal)?
    {
        return Err(RecursiveV2Error::Canonical);
    }
    Ok(RecursiveTraceChunkControlV2 {
        source_ordinal,
        chunk_ordinal,
        chunk_count,
        byte_count,
        bytes,
    })
}

impl RecursiveTraceEventV2 {
    /// Construct one profile-bounded event.
    pub(crate) fn new(
        ordinal: u64,
        opcode: RecursiveTraceOpcodeV2,
        object_id: [u8; 32],
        payload: Vec<u8>,
        profile: &RecursiveCircuitProfileV2,
    ) -> Result<Self, RecursiveV2Error> {
        if object_id == [0; 32]
            || payload.len()
                > usize::try_from(profile.max_leaf_bytes()).map_err(|_| RecursiveV2Error::Limit)?
        {
            return Err(RecursiveV2Error::Limit);
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

    pub(crate) fn canonical_len(&self) -> Result<u64, RecursiveV2Error> {
        u64::try_from(
            TRACE_EVENT_HEADER_BYTES_V2
                .checked_add(self.payload.len())
                .ok_or(RecursiveV2Error::Overflow)?,
        )
        .map_err(|_| RecursiveV2Error::Limit)
    }

    pub(crate) fn canonical_bytes(&self) -> Result<Vec<u8>, RecursiveV2Error> {
        encode_canonical_source_record(self.opcode, self.ordinal, self.object_id, &self.payload)
    }

    /// Return the sole fixed-width view used to feed both source and global
    /// SHA contexts.  The caller must retain only one chunk at a time.
    pub(crate) fn canonical_chunks(
        &self,
    ) -> Result<Vec<RecursiveTraceCanonicalChunkV2>, RecursiveV2Error> {
        let bytes = self.canonical_bytes()?;
        let chunk_count = bytes
            .len()
            .checked_add(TRACE_CANONICAL_CHUNK_BYTES_V2 - 1)
            .map(|value| value / TRACE_CANONICAL_CHUNK_BYTES_V2)
            .ok_or(RecursiveV2Error::Overflow)?;
        let chunk_count = u32::try_from(chunk_count).map_err(|_| RecursiveV2Error::Limit)?;
        if chunk_count == 0 {
            return Err(RecursiveV2Error::Canonical);
        }
        bytes
            .chunks(TRACE_CANONICAL_CHUNK_BYTES_V2)
            .enumerate()
            .map(|(chunk_ordinal, chunk)| {
                let byte_count = u8::try_from(chunk.len()).map_err(|_| RecursiveV2Error::Limit)?;
                let mut padded = [0_u8; TRACE_CANONICAL_CHUNK_BYTES_V2];
                padded[..chunk.len()].copy_from_slice(chunk);
                Ok(RecursiveTraceCanonicalChunkV2 {
                    source_ordinal: self.ordinal,
                    chunk_ordinal: u32::try_from(chunk_ordinal)
                        .map_err(|_| RecursiveV2Error::Limit)?,
                    chunk_count,
                    byte_count,
                    bytes: padded,
                })
            })
            .collect()
    }

    pub(crate) fn hash_binding(&self) -> Result<[u8; 32], RecursiveV2Error> {
        let bytes = self.canonical_bytes()?;
        Ok(sha256_256_role(
            CheckpointShaRole::Trace,
            &[SOURCE_RECORD_HASH_LABEL_V2, &bytes],
        ))
    }

    /// Exact FIPS message length and compression count for this source record.
    pub(crate) fn hash_geometry(&self) -> Result<(u64, u64), RecursiveV2Error> {
        let bytes = self.canonical_bytes()?;
        let part_bytes = u64::try_from(SOURCE_RECORD_HASH_LABEL_V2.len())
            .map_err(|_| RecursiveV2Error::Limit)?
            .checked_add(u64::try_from(bytes.len()).map_err(|_| RecursiveV2Error::Limit)?)
            .ok_or(RecursiveV2Error::Overflow)?;
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
    ) -> Result<Self, RecursiveV2Error> {
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
) -> Result<Vec<u8>, RecursiveV2Error> {
    let payload_len = u32::try_from(payload.len()).map_err(|_| RecursiveV2Error::Limit)?;
    let mut bytes = Vec::with_capacity(
        TRACE_EVENT_HEADER_BYTES_V2
            .checked_add(payload.len())
            .ok_or(RecursiveV2Error::Overflow)?,
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
) -> Result<(RecursiveTraceOpcodeV2, u64, [u8; 32], Vec<u8>), RecursiveV2Error> {
    if bytes.len() < TRACE_EVENT_HEADER_BYTES_V2 {
        return Err(RecursiveV2Error::Canonical);
    }
    let opcode = RecursiveTraceOpcodeV2::decode(bytes[0])?;
    let ordinal = u64::from_le_bytes(
        bytes[1..9]
            .try_into()
            .map_err(|_| RecursiveV2Error::Canonical)?,
    );
    let object_id = bytes[9..41]
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    let payload_len = usize::try_from(u32::from_le_bytes(
        bytes[41..45]
            .try_into()
            .map_err(|_| RecursiveV2Error::Canonical)?,
    ))
    .map_err(|_| RecursiveV2Error::Limit)?;
    if payload_len
        > usize::try_from(profile.max_leaf_bytes()).map_err(|_| RecursiveV2Error::Limit)?
        || bytes.len()
            != TRACE_EVENT_HEADER_BYTES_V2
                .checked_add(payload_len)
                .ok_or(RecursiveV2Error::Overflow)?
    {
        return Err(RecursiveV2Error::Canonical);
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
const HASH_CONTROL_SOURCE_COMMON_BYTES: usize =
    HASH_CONTROL_SCHEMA_BYTES + HASH_CONTROL_SOURCE_BINDING_BYTES;
const HASH_CONTROL_TRACE_COMMON_BYTES: usize =
    HASH_CONTROL_SCHEMA_BYTES + HASH_CONTROL_TRACE_BINDING_BYTES;
const HASH_CONTROL_BLOCK_BYTES: usize = 8 + 8 + 64 + 32 + 32 + 1;
const HASH_CONTROL_SOURCE_BLOCK_PAYLOAD_BYTES: usize =
    HASH_CONTROL_SOURCE_COMMON_BYTES + HASH_CONTROL_BLOCK_BYTES;
const HASH_CONTROL_TRACE_BLOCK_PAYLOAD_BYTES: usize =
    HASH_CONTROL_TRACE_COMMON_BYTES + HASH_CONTROL_BLOCK_BYTES;
const HASH_CONTROL_ORDINAL_FLAG: u64 = 1_u64 << 63;
const HASH_CONTROL_ORDINAL_STRIDE: u64 = 1_u64 << 24;
const TRACE_HASH_ROLE_TAG_V2: u8 = 1;

/// Frozen control grammar discriminator shared by per-source bindings and
/// the one whole-trace precommit stream.  The byte is explicit so the shared
/// decoder and Nova SHA lane never reinterpret one transcript as the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum HashControlSchemaV2 {
    SourceRecord = 1,
    TracePrecommit = 2,
}

impl HashControlSchemaV2 {
    fn decode(value: u8) -> Result<Self, RecursiveV2Error> {
        match value {
            1 => Ok(Self::SourceRecord),
            2 => Ok(Self::TracePrecommit),
            _ => Err(RecursiveV2Error::Canonical),
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
    mut emit: impl FnMut(RecursiveTraceEventV2) -> Result<(), RecursiveV2Error>,
) -> Result<(), RecursiveV2Error> {
    let mut forward = |event: &RecursiveTraceEventV2| emit(event.clone());
    let mut ignore_chunk =
        |_: RecursiveTraceCanonicalChunkV2, _: &mut _| Ok::<(), RecursiveV2Error>(());
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
) -> Result<(), RecursiveV2Error>
where
    F: FnMut(&RecursiveTraceEventV2) -> Result<(), RecursiveV2Error>,
    G: FnMut(RecursiveTraceCanonicalChunkV2, &mut F) -> Result<(), RecursiveV2Error>,
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

    let source_bytes = source.canonical_bytes()?;
    let canonical_chunks = source.canonical_chunks()?;
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
        .begin_part_with(
            u64::try_from(source_bytes.len()).map_err(|_| RecursiveV2Error::Limit)?,
            &mut |block| {
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
            },
        )
        .map_err(map_block_visit_error)?;
    for chunk in canonical_chunks {
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
    stream.finish_part().map_err(RecursiveV2Error::from)?;
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
        return Err(RecursiveV2Error::Invariant);
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
    emit: &mut impl FnMut(&RecursiveTraceEventV2) -> Result<(), RecursiveV2Error>,
    emitted_blocks: &mut u64,
    block: CheckpointSha256BlockV2,
) -> Result<(), RecursiveV2Error> {
    if block.index() != *emitted_blocks {
        return Err(RecursiveV2Error::Invariant);
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
        .ok_or(RecursiveV2Error::Overflow)?;
    Ok(())
}

fn map_block_visit_error(
    error: CheckpointSha256BlockVisitError<RecursiveV2Error>,
) -> RecursiveV2Error {
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
    fn decode(value: u8) -> Result<Self, RecursiveV2Error> {
        match value {
            1 => Ok(Self::Begin),
            2 => Ok(Self::Block),
            3 => Ok(Self::End),
            _ => Err(RecursiveV2Error::Canonical),
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
) -> Result<RecursiveTraceEventV2, RecursiveV2Error> {
    if matches!(stage, HashControlStageV2::Block) != block.is_some() {
        return Err(RecursiveV2Error::Invariant);
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
        .map_err(|_| RecursiveV2Error::Limit)?;
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
            .ok_or(RecursiveV2Error::Overflow)?,
        None if matches!(stage, HashControlStageV2::Begin) => 0,
        None => block_count
            .checked_add(1)
            .ok_or(RecursiveV2Error::Overflow)?,
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
) -> Result<(u64, u64, u64, u64), RecursiveV2Error> {
    let message_bytes = CheckpointSha256BlockStreamV2::framed_bytes_for_parts(
        CheckpointShaRole::Trace,
        precommit.byte_count,
        precommit.event_count,
    )
    .map_err(|_| RecursiveV2Error::Limit)?;
    let block_count = CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(message_bytes)
        .map_err(|_| RecursiveV2Error::Limit)?;
    let padding_bytes = sha256_padding_bytes(message_bytes, block_count)?;
    let bit_length = message_bytes
        .checked_mul(8)
        .ok_or(RecursiveV2Error::Overflow)?;
    if precommit.event_count == 0 || block_count == 0 {
        return Err(RecursiveV2Error::Invariant);
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
    emit: &mut impl FnMut(&RecursiveTraceEventV2) -> Result<(), RecursiveV2Error>,
    emitted_blocks: &mut u64,
) -> Result<(), RecursiveV2Error> {
    if block.index() != *emitted_blocks {
        return Err(RecursiveV2Error::Invariant);
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
        .ok_or(RecursiveV2Error::Overflow)?;
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
) -> Result<RecursiveTraceEventV2, RecursiveV2Error> {
    if matches!(stage, HashControlStageV2::Block) != block.is_some() {
        return Err(RecursiveV2Error::Invariant);
    }
    let payload_bytes = if block.is_some() {
        HASH_CONTROL_TRACE_BLOCK_PAYLOAD_BYTES
    } else {
        HASH_CONTROL_TRACE_COMMON_BYTES
    };
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(payload_bytes)
        .map_err(|_| RecursiveV2Error::Limit)?;
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
            .ok_or(RecursiveV2Error::Overflow)?,
        None if matches!(stage, HashControlStageV2::Begin) => 0,
        None => block_count
            .checked_add(1)
            .ok_or(RecursiveV2Error::Overflow)?,
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

fn sha256_padding_bytes(message_bytes: u64, block_count: u64) -> Result<u64, RecursiveV2Error> {
    block_count
        .checked_mul(64)
        .and_then(|bytes| bytes.checked_sub(message_bytes))
        .and_then(|bytes| bytes.checked_sub(9))
        .filter(|bytes| *bytes < 64)
        .ok_or(RecursiveV2Error::Invariant)
}

pub(crate) fn hash_control_ordinal(
    source_ordinal: u64,
    sequence: u64,
) -> Result<u64, RecursiveV2Error> {
    if sequence >= HASH_CONTROL_ORDINAL_STRIDE {
        return Err(RecursiveV2Error::Limit);
    }
    HASH_CONTROL_ORDINAL_FLAG
        .checked_add(
            source_ordinal
                .checked_mul(HASH_CONTROL_ORDINAL_STRIDE)
                .and_then(|value| value.checked_add(sequence))
                .ok_or(RecursiveV2Error::Overflow)?,
        )
        .ok_or(RecursiveV2Error::Overflow)
}

fn append_sha_state(payload: &mut Vec<u8>, state: &[u32; 8]) {
    for word in state {
        payload.extend_from_slice(&word.to_be_bytes());
    }
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
) -> Result<HashControlBindingV2, RecursiveV2Error> {
    let payload = event.payload();
    if payload.len() < HASH_CONTROL_SCHEMA_BYTES {
        return Err(RecursiveV2Error::Canonical);
    }
    let schema = HashControlSchemaV2::decode(payload[0])?;
    let role = payload[1];
    if role != TRACE_HASH_ROLE_TAG_V2 {
        return Err(RecursiveV2Error::Canonical);
    }
    let stage = HashControlStageV2::decode(payload[2])?;
    let binding = payload[3..35]
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    let message_bytes = u64::from_le_bytes(
        payload[35..43]
            .try_into()
            .map_err(|_| RecursiveV2Error::Canonical)?,
    );
    let block_count = u64::from_le_bytes(
        payload[43..51]
            .try_into()
            .map_err(|_| RecursiveV2Error::Canonical)?,
    );
    let (common_bytes, source, trace) = match schema {
        HashControlSchemaV2::SourceRecord => {
            if payload.len() < HASH_CONTROL_SOURCE_COMMON_BYTES {
                return Err(RecursiveV2Error::Canonical);
            }
            let ordinal = u64::from_le_bytes(
                payload[51..59]
                    .try_into()
                    .map_err(|_| RecursiveV2Error::Canonical)?,
            );
            let opcode = RecursiveTraceOpcodeV2::decode(payload[59])?;
            if !opcode.is_source_record() {
                return Err(RecursiveV2Error::Canonical);
            }
            let object_id = payload[60..92]
                .try_into()
                .map_err(|_| RecursiveV2Error::Canonical)?;
            (
                HASH_CONTROL_SOURCE_COMMON_BYTES,
                Some(HashControlSourceBindingV2 {
                    ordinal,
                    opcode,
                    object_id,
                }),
                None,
            )
        }
        HashControlSchemaV2::TracePrecommit => {
            if payload.len() < HASH_CONTROL_TRACE_COMMON_BYTES {
                return Err(RecursiveV2Error::Canonical);
            }
            let event_count = u64::from_le_bytes(
                payload[51..59]
                    .try_into()
                    .map_err(|_| RecursiveV2Error::Canonical)?,
            );
            let byte_count = u64::from_le_bytes(
                payload[59..67]
                    .try_into()
                    .map_err(|_| RecursiveV2Error::Canonical)?,
            );
            let padding_bytes = u64::from_le_bytes(
                payload[67..75]
                    .try_into()
                    .map_err(|_| RecursiveV2Error::Canonical)?,
            );
            let bit_length = u64::from_le_bytes(
                payload[75..83]
                    .try_into()
                    .map_err(|_| RecursiveV2Error::Canonical)?,
            );
            let eof = match payload[83] {
                0 => false,
                1 => true,
                _ => return Err(RecursiveV2Error::Canonical),
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
            )
        }
    };
    let block = match stage {
        HashControlStageV2::Block => {
            if payload.len()
                != common_bytes
                    .checked_add(HASH_CONTROL_BLOCK_BYTES)
                    .ok_or(RecursiveV2Error::Overflow)?
            {
                return Err(RecursiveV2Error::Canonical);
            }
            let index = u64::from_le_bytes(
                payload[common_bytes..common_bytes + 8]
                    .try_into()
                    .map_err(|_| RecursiveV2Error::Canonical)?,
            );
            let byte_offset = u64::from_le_bytes(
                payload[common_bytes + 8..common_bytes + 16]
                    .try_into()
                    .map_err(|_| RecursiveV2Error::Canonical)?,
            );
            let block = payload[common_bytes + 16..common_bytes + 80]
                .try_into()
                .map_err(|_| RecursiveV2Error::Canonical)?;
            let chaining_before =
                decode_sha_state(&payload[common_bytes + 80..common_bytes + 112])?;
            let chaining_after =
                decode_sha_state(&payload[common_bytes + 112..common_bytes + 144])?;
            let final_block = match payload[common_bytes + 144] {
                0 => false,
                1 => true,
                _ => return Err(RecursiveV2Error::Canonical),
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
                return Err(RecursiveV2Error::Canonical);
            }
            None
        }
    };
    Ok(HashControlBindingV2 {
        schema,
        role,
        stage,
        binding,
        source_ordinal: source.map(|binding| binding.ordinal).unwrap_or_else(|| {
            trace
                .expect("trace binding is present for the trace control schema")
                .event_count
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
        block,
    })
}

fn decode_sha_state(bytes: &[u8]) -> Result<[u32; 8], RecursiveV2Error> {
    if bytes.len() != 32 {
        return Err(RecursiveV2Error::Canonical);
    }
    let mut state = [0_u32; 8];
    for (slot, word) in state.iter_mut().zip(bytes.chunks_exact(4)) {
        *slot = u32::from_be_bytes(word.try_into().map_err(|_| RecursiveV2Error::Canonical)?);
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
}

impl RecursiveTracePassV2 {
    #[must_use]
    pub const fn precommit(&self) -> RecursiveTracePrecommitV2 {
        self.precommit
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
    ) -> Result<Self, RecursiveV2Error> {
        let spool_dir = dir.as_ref().to_path_buf();
        Ok(Self {
            spool: PrivateSpoolFile::create_in(&spool_dir, profile.max_content_bytes())?,
            profile,
            snapshot,
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
    pub(crate) const fn max_content_bytes(&self) -> u64 {
        self.profile.max_content_bytes()
    }

    #[must_use]
    pub(crate) const fn profile(&self) -> &RecursiveCircuitProfileV2 {
        &self.profile
    }

    /// Start streaming canonical events into the first pass.
    pub(crate) fn begin_canonical_precommit(&mut self) -> Result<(), RecursiveV2Error> {
        if self.state != TraceSourceStateV2::Open || self.precommit_builder.is_some() {
            return Err(RecursiveV2Error::TraceState);
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
    ) -> Result<(), RecursiveV2Error> {
        if self.state != TraceSourceStateV2::Open {
            return Err(RecursiveV2Error::TraceState);
        }
        let builder = self
            .precommit_builder
            .as_mut()
            .ok_or(RecursiveV2Error::TraceState)?;
        builder.append(event, &self.profile, &mut self.spool)
    }

    /// Seal the streamed first pass before challenge-dependent replay begins.
    pub(crate) fn seal_canonical_precommit(
        &mut self,
    ) -> Result<RecursiveTracePrecommitV2, RecursiveV2Error> {
        if self.state != TraceSourceStateV2::Open {
            return Err(RecursiveV2Error::TraceState);
        }
        let builder = self
            .precommit_builder
            .take()
            .ok_or(RecursiveV2Error::TraceState)?;
        let precommit = builder.finish()?;
        self.spool.rewind()?;
        self.precommit = Some(precommit);
        self.state = TraceSourceStateV2::Precommitted;
        Ok(precommit)
    }

    #[cfg(test)]
    fn precommit_from_canonical_events(
        &mut self,
        events: &[RecursiveTraceEventV2],
    ) -> Result<RecursiveTracePrecommitV2, RecursiveV2Error> {
        self.begin_canonical_precommit()?;
        for event in events {
            self.append_canonical_event(event.clone())?;
        }
        self.seal_canonical_precommit()
    }

    /// Replay exactly the precommitted spool, invoking `visit` once per event.
    pub(crate) fn event_pass(
        &mut self,
        mut visit: impl FnMut(&RecursiveTraceEventV2) -> Result<(), RecursiveV2Error>,
    ) -> Result<RecursiveTracePassV2, RecursiveV2Error> {
        if self.state != TraceSourceStateV2::Precommitted {
            return Err(RecursiveV2Error::TraceState);
        }
        let precommit = self.precommit.ok_or(RecursiveV2Error::TraceState)?;
        // The descriptor identity and exact bounded length were captured at
        // exclusive creation and checked again after fsync/rewind.  Recheck at
        // the pass boundary so a truncate, append, replacement, hard-link, or
        // permission change cannot be replayed as the precommitted source.
        self.spool.verify_integrity()?;
        let (trace_message_bytes, trace_block_count, trace_padding_bytes, trace_bit_length) =
            trace_hash_geometry(precommit)?;
        visit(&trace_hash_control_event(
            HashControlStageV2::Begin,
            RecursiveTraceOpcodeV2::BeginHash,
            precommit,
            trace_message_bytes,
            trace_block_count,
            trace_padding_bytes,
            trace_bit_length,
            None,
            &self.profile,
        )?)?;
        // This stream is started once and receives the very same chunks that
        // feed the source-local transcript below. It is never rebuilt from a
        // rewind, a block tape, or a second source encoder.
        let mut trace_stream = CheckpointSha256BlockStreamV2::new(CheckpointShaRole::Trace);
        let mut trace_blocks = 0_u64;
        let mut trace = CheckpointSha256V2::new(CheckpointShaRole::Trace);
        let mut identifiers = IdentifierPrecommitV2::new(&self.spool_dir, &self.profile)?;
        let mut event_count = 0_u64;
        let mut byte_count = 0_u64;
        loop {
            let Some(bytes) = read_spooled_trace_record(&mut self.spool, &self.profile)? else {
                break;
            };
            let total = bytes.len();
            let event = RecursiveTraceEventV2::decode_canonical(&bytes, &self.profile)?;
            if !event.opcode().is_source_record() {
                return Err(RecursiveV2Error::Canonical);
            }
            if event.ordinal() != event_count {
                return Err(RecursiveV2Error::EventOrder);
            }
            event_count = event_count
                .checked_add(1)
                .ok_or(RecursiveV2Error::Overflow)?;
            byte_count = byte_count
                .checked_add(u64::try_from(total).map_err(|_| RecursiveV2Error::Limit)?)
                .ok_or(RecursiveV2Error::Overflow)?;
            if event.opcode().is_source_record() {
                trace.update_part(&bytes)?;
            }
            identifiers.absorb(&event)?;
            visit(&event)?;
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
                        &mut visit,
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
            emit_derived_hash_controls_with_chunk(
                &event,
                &self.profile,
                &mut visit,
                &mut feed_trace_chunk,
            )?;
            trace_stream.finish_part().map_err(RecursiveV2Error::from)?;
        }
        let replayed = RecursiveTracePrecommitV2 {
            event_count,
            byte_count,
            trace_digest: trace.finalize(),
            ..identifiers.finish()?
        };
        self.spool.verify_integrity()?;
        if replayed != precommit || byte_count != self.spool.len() {
            return Err(RecursiveV2Error::Canonical);
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
                    &mut visit,
                    &mut trace_blocks,
                )
            })
            .map_err(map_block_visit_error)?;
        if trace_digest != precommit.trace_digest || trace_blocks != trace_block_count {
            return Err(RecursiveV2Error::Invariant);
        }
        visit(&trace_hash_control_event(
            HashControlStageV2::End,
            RecursiveTraceOpcodeV2::EndHash,
            precommit,
            trace_message_bytes,
            trace_block_count,
            trace_padding_bytes,
            trace_bit_length,
            None,
            &self.profile,
        )?)?;
        self.state = TraceSourceStateV2::Replayed;
        Ok(RecursiveTracePassV2 { precommit })
    }

    /// Close the source only after the caller proves the same handle remains live.
    pub(crate) fn finish(
        &mut self,
        observed_snapshot: RecursiveSnapshotHandleV2,
    ) -> Result<RecursiveTracePrecommitV2, RecursiveV2Error> {
        if self.state != TraceSourceStateV2::Replayed || observed_snapshot != self.snapshot {
            return Err(RecursiveV2Error::SnapshotChanged);
        }
        self.spool.verify_integrity()?;
        self.state = TraceSourceStateV2::Finished;
        self.precommit.ok_or(RecursiveV2Error::TraceState)
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
    fn new(dir: &Path, profile: &RecursiveCircuitProfileV2) -> Result<Self, RecursiveV2Error> {
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
    ) -> Result<(), RecursiveV2Error> {
        if !event.opcode().is_source_record()
            || event.ordinal() != self.expected_ordinal
            || self.event_count >= u64::from(profile.max_typed_events())
        {
            return Err(RecursiveV2Error::EventOrder);
        }
        let encoded = event.canonical_bytes()?;
        let encoded_len = u64::try_from(encoded.len()).map_err(|_| RecursiveV2Error::Limit)?;
        let next_bytes = self
            .byte_count
            .checked_add(encoded_len)
            .ok_or(RecursiveV2Error::Overflow)?;
        if next_bytes > profile.max_content_bytes() {
            return Err(RecursiveV2Error::Limit);
        }
        if event.opcode().is_source_record() {
            self.trace.update_part(&encoded)?;
        }
        self.identifiers.absorb(&event)?;
        spool.write_bounded(&encoded)?;
        self.expected_ordinal = self
            .expected_ordinal
            .checked_add(1)
            .ok_or(RecursiveV2Error::Overflow)?;
        self.event_count = self
            .event_count
            .checked_add(1)
            .ok_or(RecursiveV2Error::Overflow)?;
        self.byte_count = next_bytes;
        Ok(())
    }

    fn finish(self) -> Result<RecursiveTracePrecommitV2, RecursiveV2Error> {
        if self.event_count == 0 {
            return Err(RecursiveV2Error::TraceState);
        }
        Ok(RecursiveTracePrecommitV2 {
            event_count: self.event_count,
            byte_count: self.byte_count,
            trace_digest: self.trace.finalize(),
            ..self.identifiers.finish()?
        })
    }
}

/// Separate pre-challenge ID commitments for spent and output rows.
///
/// Only replay identifiers participate in uniqueness. Reused object IDs in
/// structural/hash/JMT opcodes are valid references and never enter this set.
struct IdentifierPrecommitV2 {
    spent_original: CheckpointSha256V2,
    output_original: CheckpointSha256V2,
    spent_sorted: ExternalIdSortV2,
    output_sorted: ExternalIdSortV2,
}

impl IdentifierPrecommitV2 {
    fn new(dir: &Path, profile: &RecursiveCircuitProfileV2) -> Result<Self, RecursiveV2Error> {
        let spent_sort_bytes =
            RecursiveCircuitProfileV2::identifier_sort_bytes(profile.max_spent())?;
        let output_sort_bytes =
            RecursiveCircuitProfileV2::identifier_sort_bytes(profile.max_outputs())?;
        let resident_ids = usize::try_from(profile.resident_buffer_bytes())
            .map_err(|_| RecursiveV2Error::Limit)?
            / (2 * std::mem::size_of::<[u8; 32]>());
        let max_runs =
            usize::try_from(profile.max_spool_runs()).map_err(|_| RecursiveV2Error::Limit)?;
        let merge_fan_in =
            usize::try_from(profile.spool_merge_fan_in()).map_err(|_| RecursiveV2Error::Limit)?;
        Ok(Self {
            spent_original: CheckpointSha256V2::new(CheckpointShaRole::SpentOriginalIds),
            output_original: CheckpointSha256V2::new(CheckpointShaRole::OutputOriginalIds),
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

    fn absorb(&mut self, event: &RecursiveTraceEventV2) -> Result<(), RecursiveV2Error> {
        match event.opcode() {
            RecursiveTraceOpcodeV2::ReplayInput => {
                self.spent_original.update_part(&event.object_id())?;
                self.spent_sorted.push(event.object_id())?;
            }
            RecursiveTraceOpcodeV2::ReplayOutput => {
                self.output_original.update_part(&event.object_id())?;
                self.output_sorted.push(event.object_id())?;
            }
            _ => {}
        }
        Ok(())
    }

    fn finish(mut self) -> Result<RecursiveTracePrecommitV2, RecursiveV2Error> {
        let spent_sorted_ids_digest = self.spent_sorted.digest_sorted_unique()?;
        let output_sorted_ids_digest = self.output_sorted.digest_sorted_unique()?;
        if !self.spent_sorted.is_disjoint(&mut self.output_sorted)? {
            return Err(RecursiveV2Error::DuplicateIdentifier);
        }

        Ok(RecursiveTracePrecommitV2 {
            event_count: 0,
            byte_count: 0,
            trace_digest: [0; 32],
            spent_original_ids_digest: self.spent_original.finalize(),
            spent_sorted_ids_digest,
            output_original_ids_digest: self.output_original.finalize(),
            output_sorted_ids_digest,
        })
    }
}

/// Disk-backed, bounded external sorter for one replay identifier list.
///
/// Run files remain private `PrivateSpoolFile`s; the source owns their lifetime
/// and never exposes a filesystem path or handle to checkpoint callers.
struct ExternalIdSortV2 {
    dir: PathBuf,
    byte_budget: u64,
    resident_ids: usize,
    max_runs: usize,
    merge_fan_in: usize,
    digest_role: CheckpointShaRole,
    written: u64,
    pending: Vec<[u8; 32]>,
    runs: Vec<PrivateSpoolFile>,
}

impl ExternalIdSortV2 {
    fn new(
        dir: &Path,
        byte_budget: u64,
        resident_ids: usize,
        max_runs: usize,
        merge_fan_in: usize,
        digest_role: CheckpointShaRole,
    ) -> Result<Self, RecursiveV2Error> {
        if byte_budget < 32 || resident_ids == 0 || max_runs == 0 || merge_fan_in == 0 {
            return Err(RecursiveV2Error::Limit);
        }
        Ok(Self {
            dir: dir.to_path_buf(),
            byte_budget,
            resident_ids,
            max_runs,
            merge_fan_in,
            digest_role,
            written: 0,
            pending: Vec::with_capacity(resident_ids),
            runs: Vec::new(),
        })
    }

    fn push(&mut self, id: [u8; 32]) -> Result<(), RecursiveV2Error> {
        self.pending.push(id);
        if self.pending.len() == self.resident_ids {
            self.flush_run()?;
        }
        Ok(())
    }

    fn flush_run(&mut self) -> Result<(), RecursiveV2Error> {
        if self.pending.is_empty() {
            return Ok(());
        }
        if self.runs.len() >= self.max_runs {
            return Err(RecursiveV2Error::Limit);
        }
        self.pending.sort_unstable();
        let run_bytes = u64::try_from(self.pending.len())
            .map_err(|_| RecursiveV2Error::Limit)?
            .checked_mul(32)
            .ok_or(RecursiveV2Error::Overflow)?;
        let next = self
            .written
            .checked_add(run_bytes)
            .ok_or(RecursiveV2Error::Overflow)?;
        if next > self.byte_budget {
            return Err(RecursiveV2Error::Limit);
        }
        let mut run = PrivateSpoolFile::create_in(&self.dir, run_bytes)?;
        for id in self.pending.drain(..) {
            run.write_bounded(&id)?;
        }
        run.rewind()?;
        self.written = next;
        self.runs.push(run);
        Ok(())
    }

    fn open_sorted(&mut self) -> Result<SortedIdStreamV2<'_>, RecursiveV2Error> {
        self.flush_run()?;
        if self.runs.len() > self.merge_fan_in {
            return Err(RecursiveV2Error::Limit);
        }
        SortedIdStreamV2::new(&mut self.runs)
    }

    fn digest_sorted_unique(&mut self) -> Result<[u8; 32], RecursiveV2Error> {
        let digest_role = self.digest_role;
        let mut stream = self.open_sorted()?;
        let mut previous = None;
        let mut digest = CheckpointSha256V2::new(digest_role);
        while let Some(id) = stream.next_id()? {
            if previous == Some(id) {
                return Err(RecursiveV2Error::DuplicateIdentifier);
            }
            digest.update_part(&id)?;
            previous = Some(id);
        }
        Ok(digest.finalize())
    }

    fn is_disjoint(&mut self, other: &mut Self) -> Result<bool, RecursiveV2Error> {
        let mut left = self.open_sorted()?;
        let mut right = other.open_sorted()?;
        let mut left_id = left.next_id()?;
        let mut right_id = right.next_id()?;
        while let (Some(current_left), Some(current_right)) = (left_id, right_id) {
            match current_left.cmp(&current_right) {
                std::cmp::Ordering::Less => left_id = left.next_id()?,
                std::cmp::Ordering::Greater => right_id = right.next_id()?,
                std::cmp::Ordering::Equal => return Ok(false),
            }
        }
        Ok(true)
    }
}

/// Deterministic bounded k-way merge over private sorted runs.
struct SortedIdStreamV2<'a> {
    runs: &'a mut [PrivateSpoolFile],
    heads: Vec<Option<[u8; 32]>>,
}

impl<'a> SortedIdStreamV2<'a> {
    fn new(runs: &'a mut [PrivateSpoolFile]) -> Result<Self, RecursiveV2Error> {
        let mut heads = Vec::with_capacity(runs.len());
        for run in runs.iter_mut() {
            run.rewind()?;
            heads.push(read_spooled_id(run)?);
        }
        Ok(Self { runs, heads })
    }

    fn next_id(&mut self) -> Result<Option<[u8; 32]>, RecursiveV2Error> {
        let index = self
            .heads
            .iter()
            .enumerate()
            .filter_map(|(index, id)| id.map(|id| (index, id)))
            .min_by_key(|(_, id)| *id)
            .map(|(index, _)| index);
        let Some(index) = index else {
            return Ok(None);
        };
        let id = self.heads[index]
            .take()
            .ok_or(RecursiveV2Error::Canonical)?;
        self.heads[index] = read_spooled_id(&mut self.runs[index])?;
        Ok(Some(id))
    }
}

fn read_spooled_id(spool: &mut PrivateSpoolFile) -> Result<Option<[u8; 32]>, RecursiveV2Error> {
    let mut id = [0_u8; 32];
    let read = spool.read_chunk(&mut id)?;
    if read == 0 {
        return Ok(None);
    }
    spool_read_exact(spool, &mut id[read..])?;
    Ok(Some(id))
}

fn spool_read_exact(
    spool: &mut PrivateSpoolFile,
    mut bytes: &mut [u8],
) -> Result<(), RecursiveV2Error> {
    while !bytes.is_empty() {
        let read = spool.read_chunk(bytes)?;
        if read == 0 {
            return Err(RecursiveV2Error::Canonical);
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
) -> Result<Option<Vec<u8>>, RecursiveV2Error> {
    let mut tag = [0_u8; 1];
    if spool.read_chunk(&mut tag)? == 0 {
        return Ok(None);
    }
    let mut rest = [0_u8; TRACE_EVENT_HEADER_BYTES_V2 - 1];
    spool_read_exact(spool, &mut rest)?;
    let payload_len = u32::from_le_bytes(
        rest[40..44]
            .try_into()
            .map_err(|_| RecursiveV2Error::Canonical)?,
    );
    let payload_len = usize::try_from(payload_len).map_err(|_| RecursiveV2Error::Limit)?;
    if payload_len
        > usize::try_from(profile.max_leaf_bytes()).map_err(|_| RecursiveV2Error::Limit)?
    {
        return Err(RecursiveV2Error::Limit);
    }
    let total = TRACE_EVENT_HEADER_BYTES_V2
        .checked_add(payload_len)
        .ok_or(RecursiveV2Error::Overflow)?;
    let mut bytes = vec![0_u8; total];
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
        RecursiveTraceOpcodeV2, RecursiveTransitionTraceSourceV2, TRACE_CANONICAL_CHUNK_BYTES_V2,
    };
    use crate::{
        checkpoint::{
            recursive_circuit::RecursiveCircuitProfileV2,
            recursive_context::RecursiveSnapshotHandleV2,
        },
        settlement::{derive_settlement_root_v2, RootGeneration},
        snapshot::PrepSnapshotId,
    };
    use z00z_crypto::{CheckpointSha256BlockStreamV2, CheckpointShaRole};

    fn profile() -> RecursiveCircuitProfileV2 {
        RecursiveCircuitProfileV2::repository_fixture()
    }

    fn multi_run_profile() -> RecursiveCircuitProfileV2 {
        let source_records = RecursiveCircuitProfileV2::max_source_records(4, 4, 512, 4 * 1024)
            .expect("small source record cap");
        let sha_blocks = RecursiveCircuitProfileV2::sha_blocks_for_source_record_hashes(
            4 * 1024,
            source_records,
        )
        .expect("small SHA cap");
        let e_max = RecursiveCircuitProfileV2::event_bound_from_parts(
            4,
            4,
            4,
            4,
            4,
            8,
            source_records,
            sha_blocks,
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
            128,
            8 * 1024,
            4,
            4,
        )
        .expect("multi-run profile")
    }

    fn snapshot() -> RecursiveSnapshotHandleV2 {
        let root = derive_settlement_root_v2(RootGeneration::SettlementV2, 7, [3; 32], [4; 32])
            .expect("V2 root");
        RecursiveSnapshotHandleV2::new(PrepSnapshotId::new([1; 32]), 9, root, 5, 1, [2; 32])
            .expect("immutable snapshot")
    }

    fn event(ordinal: u64, opcode: RecursiveTraceOpcodeV2, id: u8) -> RecursiveTraceEventV2 {
        RecursiveTraceEventV2::new(ordinal, opcode, [id; 32], vec![id], &profile())
            .expect("bounded event")
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
            let chunks = source.canonical_chunks().expect("canonical chunk view");
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
        let source_chunks = source.canonical_chunks().expect("canonical chunk source");
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
            event(0, RecursiveTraceOpcodeV2::BeginBlock, 1),
            event(1, RecursiveTraceOpcodeV2::ReplayInput, 2),
            event(2, RecursiveTraceOpcodeV2::ReplayOutput, 3),
            event(3, RecursiveTraceOpcodeV2::CommitTypedEvent, 2),
            event(4, RecursiveTraceOpcodeV2::FinalizeBlock, 4),
        ];
        let expected_hash_controls = events
            .iter()
            .map(|event| event.hash_geometry().expect("bounded event geometry").1 + 2)
            .sum::<u64>();
        let expected_chunk_controls = events
            .iter()
            .map(|event| {
                u64::try_from(event.canonical_chunks().expect("canonical chunks").len())
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
                .and_then(|count| count.checked_add(expected_hash_controls))
                .and_then(|count| count.checked_add(global_hash_controls))
                .expect("one source record plus its FIPS controls"),
            "the spool commits source records while the sole canonical byte feeder and exact FIPS SHA schedule are regenerated"
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
            let chunks = source_event.canonical_chunks().expect("source chunks");
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
            source.canonical_chunks().expect("canonical chunks").len()
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
                event(0, RecursiveTraceOpcodeV2::BeginBlock, 1),
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
            event(0, RecursiveTraceOpcodeV2::BeginBlock, 10),
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
}
