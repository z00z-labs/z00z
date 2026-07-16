//! Private Nova foundation for the sole recursive-V2 storage owner.
//!
//! This module intentionally has no public re-export.  It fixes the running
//! state layout, constrains the common control skeleton, and proves the exact
//! source/global SHA byte relation. Full replay, uniqueness, and JMT
//! relations remain connected only as T2 work is completed.

use ff::PrimeField;
#[cfg(test)]
use nova_snark::traits::snark::RelaxedR1CSSNARKTrait;
use nova_snark::{
    frontend::{
        gadgets::{
            boolean::{AllocatedBit, Boolean},
            num::AllocatedNum,
            sha256::sha256_compression_function,
            uint32::UInt32,
        },
        r1cs::NovaShape,
        shape_cs::ShapeCS,
        ConstraintSystem, SynthesisError,
    },
    nova::{CompressedSNARK, ProverKey, PublicParams, VerifierKey},
    provider::{ipa_pc::EvaluationEngine, pasta::pallas, PallasEngine, VestaEngine},
    spartan::ppsnark::RelaxedR1CSSNARK,
    traits::circuit::StepCircuit,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::checkpoint::{
    recursive_circuit::{
        RecursiveCircuitProfileV2, RecursiveCircuitSpecV2, RECURSIVE_CIRCUIT_PROFILE_VERSION_V2,
        RECURSIVE_CIRCUIT_SPEC_VERSION_V2,
    },
    recursive_context::RecursiveAuthoritySnapshotV2,
    recursive_reject::RecursiveV2Error,
    recursive_semantics::{
        NET_MERGE_BYTES_V2, UNIQUENESS_CHALLENGE_BYTES_V2, UNIQUENESS_PRECOMMIT_BYTES_V2,
        UNIQUENESS_PRECOMMIT_VERSION_V2,
    },
};

use crate::checkpoint::recursive_trace::{
    decode_hash_control, decode_trace_chunk_control, HashControlBindingV2, HashControlSchemaV2,
    HashControlStageV2, RecursiveTraceChunkControlV2, RecursiveTraceEventV2,
    RecursiveTraceOpcodeV2, SOURCE_RECORD_HASH_LABEL_V2, TRACE_CANONICAL_CHUNK_BYTES_V2,
    TRACE_EVENT_HEADER_BYTES_V2,
};
use z00z_crypto::SHA256_IV_V2;
use z00z_crypto::{CheckpointSha256BlockStreamV2, CheckpointShaRole};

type Scalar = pallas::Scalar;
type NovaSnark<E> = RelaxedR1CSSNARK<E, EvaluationEngine<E>>;
type NovaProof = CompressedSNARK<
    PallasEngine,
    VestaEngine,
    CheckpointNovaCircuitV2,
    NovaSnark<PallasEngine>,
    NovaSnark<VestaEngine>,
>;
type NovaPublicParameters = PublicParams<PallasEngine, VestaEngine, CheckpointNovaCircuitV2>;
type NovaProverKey = ProverKey<
    PallasEngine,
    VestaEngine,
    CheckpointNovaCircuitV2,
    NovaSnark<PallasEngine>,
    NovaSnark<VestaEngine>,
>;
type NovaVerifierKey = VerifierKey<
    PallasEngine,
    VestaEngine,
    CheckpointNovaCircuitV2,
    NovaSnark<PallasEngine>,
    NovaSnark<VestaEngine>,
>;

const DIGEST_LIMBS: usize = 16;
const ANCHOR_DIGESTS: usize = 9;
const ANCHOR_CELLS: usize = DIGEST_LIMBS * ANCHOR_DIGESTS;
const CHAIN_START: usize = ANCHOR_CELLS;
const CHAIN_END: usize = 212;
const SOURCE_TRACE_ORDINAL_CELL: usize = CHAIN_START;
const SOURCE_TRACE_BYTE_COUNT_CELL: usize = SOURCE_TRACE_ORDINAL_CELL + 1;
const SOURCE_EVENT_DIGEST_START: usize = CHAIN_START + 3;
const SOURCE_EVENT_DIGEST_END: usize = SOURCE_EVENT_DIGEST_START + DIGEST_LIMBS;
const SOURCE_TRACE_ROOT_START: usize = SOURCE_EVENT_DIGEST_END;
const SOURCE_TRACE_ROOT_END: usize = SOURCE_TRACE_ROOT_START + DIGEST_LIMBS;
const PHASE_CELL: usize = CHAIN_END;
const PRIOR_OPCODE_CELL: usize = PHASE_CELL + 1;
const ORDINAL_CELL: usize = PRIOR_OPCODE_CELL + 1;
const DONE_CELL: usize = ORDINAL_CELL + 1;
const COUNTERS_START: usize = DONE_CELL + 1;
const COUNTERS_END: usize = 240;
// The reserved counter family carries the one canonical, in-circuit hash
// control schedule.  These cells do not create a second trace or SHA owner:
// their witness is decoded from `recursive_trace`'s derived controls.
const HASH_SCHEDULE_ACTIVE_CELL: usize = COUNTERS_START;
const HASH_SCHEDULE_ORDINAL_CELL: usize = HASH_SCHEDULE_ACTIVE_CELL + 1;
const HASH_SCHEDULE_SOURCE_ORDINAL_CELL: usize = HASH_SCHEDULE_ORDINAL_CELL + 1;
const HASH_SCHEDULE_MESSAGE_BYTES_CELL: usize = HASH_SCHEDULE_SOURCE_ORDINAL_CELL + 1;
const HASH_SCHEDULE_BLOCK_COUNT_CELL: usize = HASH_SCHEDULE_MESSAGE_BYTES_CELL + 1;
const HASH_SCHEDULE_NEXT_BLOCK_INDEX_CELL: usize = HASH_SCHEDULE_BLOCK_COUNT_CELL + 1;
const HASH_SCHEDULE_FINAL_BLOCK_CELL: usize = HASH_SCHEDULE_NEXT_BLOCK_INDEX_CELL + 1;
const HASH_SCHEDULE_SOURCE_HASH_START: usize = HASH_SCHEDULE_FINAL_BLOCK_CELL + 1;
const HASH_SCHEDULE_SOURCE_HASH_END: usize = HASH_SCHEDULE_SOURCE_HASH_START + DIGEST_LIMBS;
const HASH_SCHEDULE_ROLE_CELL: usize = HASH_SCHEDULE_SOURCE_HASH_END;
const TRACE_HASH_ROLE_TAG_V2: u64 = 1;
const HASH_CONTROL_ORDINAL_FLAG_V2: u64 = 1_u64 << 63;
const HASH_CONTROL_ORDINAL_STRIDE_V2: u64 = 1_u64 << 24;
const SHA_START: usize = COUNTERS_END;
const SHA_ACTIVE_CELL: usize = SHA_START;
const SHA_BLOCK_ORDINAL_CELL: usize = SHA_ACTIVE_CELL + 1;
const SHA_CHAINING_START: usize = SHA_BLOCK_ORDINAL_CELL + 1;
const SHA_CHAINING_END: usize = SHA_CHAINING_START + 8;
const SHA_BLOCK_START: usize = SHA_CHAINING_END;
const SHA_BLOCK_END: usize = SHA_BLOCK_START + 64;
const SHA_END: usize = SHA_BLOCK_END;
const UNIQUENESS_START: usize = SHA_END;
// The first three cells of the reserved uniqueness family are the replay
// grammar state.  They are not a second replay stream: every update is
// selected from the exact source-record opcode already tied to the canonical
// source header and live byte contexts.
const REPLAY_MODE_CELL: usize = UNIQUENESS_START;
const REPLAY_INPUT_COUNT_CELL: usize = REPLAY_MODE_CELL + 1;
const REPLAY_OUTPUT_COUNT_CELL: usize = REPLAY_INPUT_COUNT_CELL + 1;
// The replay decoder is an O(1) streaming state machine over the same
// canonical `TraceChunk` bytes that feed the source/global SHA contexts. It
// retains no payload, record, or alternate byte stream: its four cells hold
// only parser control while a single replay record is open.
const REPLAY_PARSE_ACTIVE_CELL: usize = REPLAY_OUTPUT_COUNT_CELL + 1;
const REPLAY_PARSE_HEADER_CELL: usize = REPLAY_PARSE_ACTIVE_CELL + 1;
const REPLAY_PARSE_STAGE_CELL: usize = REPLAY_PARSE_HEADER_CELL + 1;
const REPLAY_PARSE_REMAINING_CELL: usize = REPLAY_PARSE_STAGE_CELL + 1;
// `UniquenessPrecommit` is decoded from the same source-record `TraceChunk`
// feeder as replay payloads. These cells are only bounded parser state plus
// exact little-endian field limbs; they do not introduce a byte tape, a
// second codec, or a second SHA/source owner.
const PRECOMMIT_PARSE_ACTIVE_CELL: usize = REPLAY_PARSE_REMAINING_CELL + 1;
const PRECOMMIT_PARSE_HEADER_CELL: usize = PRECOMMIT_PARSE_ACTIVE_CELL + 1;
const PRECOMMIT_PARSE_OFFSET_CELL: usize = PRECOMMIT_PARSE_HEADER_CELL + 1;
const PRECOMMIT_PARSE_LOW_BYTE_CELL: usize = PRECOMMIT_PARSE_OFFSET_CELL + 1;
const PRECOMMIT_SPENT_COUNT_LIMB_START: usize = PRECOMMIT_PARSE_LOW_BYTE_CELL + 1;
const PRECOMMIT_SPENT_COUNT_LIMB_END: usize = PRECOMMIT_SPENT_COUNT_LIMB_START + 2;
const PRECOMMIT_OUTPUT_COUNT_LIMB_START: usize = PRECOMMIT_SPENT_COUNT_LIMB_END;
const PRECOMMIT_OUTPUT_COUNT_LIMB_END: usize = PRECOMMIT_OUTPUT_COUNT_LIMB_START + 2;
const PRECOMMIT_DIGEST_BYTES_START: usize = 1 + 4 + 4;
const PRECOMMIT_DIGEST_LIMB_COUNT: usize =
    (UNIQUENESS_PRECOMMIT_BYTES_V2 - PRECOMMIT_DIGEST_BYTES_START) / 2;
const PRECOMMIT_DIGEST_LIMB_START: usize = PRECOMMIT_OUTPUT_COUNT_LIMB_END;
const PRECOMMIT_DIGEST_LIMB_END: usize = PRECOMMIT_DIGEST_LIMB_START + PRECOMMIT_DIGEST_LIMB_COUNT;
const PRECOMMIT_COMMITMENT_DIGEST_LIMB_START: usize =
    PRECOMMIT_DIGEST_LIMB_START + DIGEST_LIMBS * 4;
// `UniquenessChallenge` consumes the precommit digest directly from the
// authenticated state above and materializes only its own challenge digest.
// Its four bounded parser cells retain no byte tape or parallel source owner.
const CHALLENGE_PARSE_ACTIVE_CELL: usize = PRECOMMIT_DIGEST_LIMB_END;
const CHALLENGE_PARSE_HEADER_CELL: usize = CHALLENGE_PARSE_ACTIVE_CELL + 1;
const CHALLENGE_PARSE_OFFSET_CELL: usize = CHALLENGE_PARSE_HEADER_CELL + 1;
const CHALLENGE_PARSE_LOW_BYTE_CELL: usize = CHALLENGE_PARSE_OFFSET_CELL + 1;
const CHALLENGE_COMMITTED_PRECOMMIT_BYTES: usize = 1 + 32;
const CHALLENGE_DIGEST_BYTES_START: usize = CHALLENGE_COMMITTED_PRECOMMIT_BYTES;
const CHALLENGE_DIGEST_LIMB_COUNT: usize =
    (UNIQUENESS_CHALLENGE_BYTES_V2 - CHALLENGE_DIGEST_BYTES_START) / 2;
const CHALLENGE_DIGEST_LIMB_START: usize = CHALLENGE_PARSE_LOW_BYTE_CELL + 1;
const CHALLENGE_DIGEST_LIMB_END: usize = CHALLENGE_DIGEST_LIMB_START + CHALLENGE_DIGEST_LIMB_COUNT;
// This boundary is derived from the exact canonical codec allocations above.
// Keeping it structural prevents a future payload-field addition from silently
// aliasing the independently reserved net/JMT state families.
const UNIQUENESS_END: usize = CHALLENGE_DIGEST_LIMB_END;
// `NetMerge` materializes its canonical digest from the same source feeder.
// The four parser cells are bounded state, not an alternative record/byte
// owner. The semantic SHA relation remains a later constraint over these
// authenticated limbs; this parser does not treat a native digest assertion
// as that relation.
const NET_START: usize = UNIQUENESS_END;
const NET_PARSE_ACTIVE_CELL: usize = NET_START;
const NET_PARSE_HEADER_CELL: usize = NET_PARSE_ACTIVE_CELL + 1;
const NET_PARSE_OFFSET_CELL: usize = NET_PARSE_HEADER_CELL + 1;
const NET_PARSE_LOW_BYTE_CELL: usize = NET_PARSE_OFFSET_CELL + 1;
const NET_DIGEST_BYTES_START: usize = 1;
const NET_DIGEST_LIMB_COUNT: usize = (NET_MERGE_BYTES_V2 - NET_DIGEST_BYTES_START) / 2;
const NET_DIGEST_LIMB_START: usize = NET_PARSE_LOW_BYTE_CELL + 1;
const NET_DIGEST_LIMB_END: usize = NET_DIGEST_LIMB_START + NET_DIGEST_LIMB_COUNT;
const NET_END: usize = NET_DIGEST_LIMB_END;
const JMT_START: usize = NET_END;
const JMT_END: usize = JMT_START + 160;
const HIERARCHY_START: usize = JMT_END;
const HIERARCHY_END: usize = HIERARCHY_START + 68;
const COMMITMENTS_START: usize = HIERARCHY_END;
const COMMITMENTS_END: usize = COMMITMENTS_START + 96;
const EXPECTED_FINALS_START: usize = COMMITMENTS_END;
const EXPECTED_TRACE_ROOT_START: usize = EXPECTED_FINALS_START;
const EXPECTED_TRACE_ROOT_END: usize = EXPECTED_TRACE_ROOT_START + DIGEST_LIMBS;
const EXPECTED_TRACE_DIGEST_START: usize = EXPECTED_TRACE_ROOT_END;
const EXPECTED_TRACE_DIGEST_END: usize = EXPECTED_TRACE_DIGEST_START + DIGEST_LIMBS;
// S1 owns exactly two fixed-width byte contexts.  The source context feeds
// one source-record transcript; the global context consumes those same bytes
// under the whole-trace framing.  Neither context contains an event tape,
// source-record arena, or a second spool.
const BYTE_CONTEXT_START: usize = EXPECTED_FINALS_START + 80;
const BYTE_CONTEXT_COUNTERS: usize = 14;
const BYTE_CONTEXT_ACTIVE_OFFSET: usize = 0;
const BYTE_CONTEXT_SOURCE_ORDINAL_OFFSET: usize = 1;
// Canonical source-record bytes and role-framed SHA message bytes are distinct
// quantities. Keeping both makes the last FIPS block and its big-endian bit
// length a consequence of the context, never a BEGIN_HASH payload assertion.
const BYTE_CONTEXT_CANONICAL_BYTES_OFFSET: usize = 2;
const BYTE_CONTEXT_MESSAGE_BYTES_OFFSET: usize = 3;
const BYTE_CONTEXT_CONSUMED_BYTES_OFFSET: usize = 4;
const BYTE_CONTEXT_NEXT_CHUNK_OFFSET: usize = 5;
const BYTE_CONTEXT_CHUNK_COUNT_OFFSET: usize = 6;
const BYTE_CONTEXT_BUFFER_LEN_OFFSET: usize = 7;
const BYTE_CONTEXT_STARTED_OFFSET: usize = 8;
const BYTE_CONTEXT_PENDING_OFFSET: usize = 9;
const BYTE_CONTEXT_EOF_OFFSET: usize = 10;
const BYTE_CONTEXT_PADDING_BLOCKS_OFFSET: usize = 11;
// Every context owns its own FIPS block cursor. Source-local and whole-trace
// SHA schedules interleave, so a single shared `SHA_BLOCK_ORDINAL_CELL` would
// let one transcript overwrite the other.
const BYTE_CONTEXT_BLOCK_COUNT_OFFSET: usize = 12;
const BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET: usize = 13;
const BYTE_CONTEXT_CHAINING_START_OFFSET: usize = BYTE_CONTEXT_COUNTERS;
const BYTE_CONTEXT_CHAINING_END_OFFSET: usize = BYTE_CONTEXT_CHAINING_START_OFFSET + 8;
const BYTE_CONTEXT_BUFFER_START_OFFSET: usize = BYTE_CONTEXT_CHAINING_END_OFFSET;
// Static role/DST framing is derived directly for its first full compression
// blocks; it is never copied into the running state.  Afterwards a context
// retains at most a <64-byte tail plus one 64-byte canonical feeder chunk.
// 128 bytes is therefore a proof-theoretic fixed bound, not a payload arena.
const BYTE_CONTEXT_BUFFER_BYTES: usize = TRACE_CANONICAL_CHUNK_BYTES_V2 * 2;
const BYTE_CONTEXT_BUFFER_END_OFFSET: usize =
    BYTE_CONTEXT_BUFFER_START_OFFSET + BYTE_CONTEXT_BUFFER_BYTES;
// A source context retains exactly one fixed canonical header across the
// source-record → BEGIN_HASH → SHA_BLOCK* → END_HASH interval. It is not a
// payload arena: the variable body remains available only through TraceChunk.
const BYTE_CONTEXT_HEADER_START_OFFSET: usize = BYTE_CONTEXT_BUFFER_END_OFFSET;
const BYTE_CONTEXT_HEADER_END_OFFSET: usize =
    BYTE_CONTEXT_HEADER_START_OFFSET + TRACE_EVENT_HEADER_BYTES_V2;
const BYTE_CONTEXT_WIDTH: usize = BYTE_CONTEXT_HEADER_END_OFFSET;
const SOURCE_BYTE_CONTEXT_START: usize = BYTE_CONTEXT_START;
const GLOBAL_BYTE_CONTEXT_START: usize = SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_WIDTH;
const RUNNING_STATE_ARITY_V2: usize = GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_WIDTH;

/// The only circuit control phases. Numeric values are part of the private
/// circuit specification and are selected algebraically, never by a host
/// validity flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub(crate) enum ControlPhaseV2 {
    Idle = 0,
    Replay = 1,
    Precommit = 2,
    Challenge = 3,
    Net = 4,
    Jmt = 5,
    Promote = 6,
    Commit = 7,
    TraceClosure = 8,
}

/// The canonical replay grammar within the `Replay` control phase.  The
/// broader uniqueness relation is separately constrained in this same state
/// family; this enum prevents the circuit from accepting an input after the
/// first output while that work is connected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
enum ReplayModeV2 {
    Inputs = 0,
    Outputs = 1,
}

/// Private states of the canonical replay payload parser. The layout is the
/// single `CanonicalFlowItemV2` codec order; it never accepts a digest or
/// host-decoded substitute for the bytes consumed from `TraceChunk`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
enum ReplayParserStageV2 {
    Op = 0,
    TxLenLow = 1,
    TxLenHigh = 2,
    TxData = 3,
    DefinitionLenLow = 4,
    DefinitionLenHigh = 5,
    DefinitionData = 6,
    Serial = 7,
    TerminalLenLow = 8,
    TerminalLenHigh = 9,
    TerminalData = 10,
    Leaf = 11,
    FirstDefinition = 12,
    FirstSerial = 13,
    FirstObject = 14,
}

impl ReplayParserStageV2 {
    const ALL: [Self; 15] = [
        Self::Op,
        Self::TxLenLow,
        Self::TxLenHigh,
        Self::TxData,
        Self::DefinitionLenLow,
        Self::DefinitionLenHigh,
        Self::DefinitionData,
        Self::Serial,
        Self::TerminalLenLow,
        Self::TerminalLenHigh,
        Self::TerminalData,
        Self::Leaf,
        Self::FirstDefinition,
        Self::FirstSerial,
        Self::FirstObject,
    ];
}

impl ControlPhaseV2 {
    const ALL: [Self; 9] = [
        Self::Idle,
        Self::Replay,
        Self::Precommit,
        Self::Challenge,
        Self::Net,
        Self::Jmt,
        Self::Promote,
        Self::Commit,
        Self::TraceClosure,
    ];
}

/// One accepted edge of the frozen private control machine.  The absence of a
/// row is itself a typed rejection; there is no implicit state-preserving
/// fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ControlTransitionV2 {
    input_phase: ControlPhaseV2,
    input_done: bool,
    opcode: RecursiveTraceOpcodeV2,
    next_phase: ControlPhaseV2,
    next_done: bool,
}

impl ControlTransitionV2 {
    const fn new(
        input_phase: ControlPhaseV2,
        input_done: bool,
        opcode: RecursiveTraceOpcodeV2,
        next_phase: ControlPhaseV2,
        next_done: bool,
    ) -> Self {
        Self {
            input_phase,
            input_done,
            opcode,
            next_phase,
            next_done,
        }
    }
}

/// Typed private control rejection.  It is intentionally distinct from a
/// legal self-loop: a finalized state has no post-final event, while an
/// otherwise invalid tuple has no generic no-op successor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlTransitionRejectionV2 {
    Finalized,
    IllegalEdge,
}

impl ControlTransitionRejectionV2 {
    const fn message(self) -> &'static str {
        match self {
            Self::Finalized => "post-final control event",
            Self::IllegalEdge => "illegal control transition",
        }
    }
}

/// The one production opcode/phase/done table.  Hash controls are explicit
/// self-loop edges for every live phase; they never arise from a generic
/// no-op rule.  There are deliberately no `input_done = true` rows.
const CONTROL_TRANSITION_TABLE_V2: [ControlTransitionV2; 47] = [
    ControlTransitionV2::new(
        ControlPhaseV2::Idle,
        false,
        RecursiveTraceOpcodeV2::BeginBlock,
        ControlPhaseV2::Replay,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Replay,
        false,
        RecursiveTraceOpcodeV2::ReplayInput,
        ControlPhaseV2::Replay,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Replay,
        false,
        RecursiveTraceOpcodeV2::ReplayOutput,
        ControlPhaseV2::Replay,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Replay,
        false,
        RecursiveTraceOpcodeV2::UniquenessPrecommit,
        ControlPhaseV2::Precommit,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Precommit,
        false,
        RecursiveTraceOpcodeV2::UniquenessChallenge,
        ControlPhaseV2::Challenge,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Challenge,
        false,
        RecursiveTraceOpcodeV2::NetMerge,
        ControlPhaseV2::Net,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Net,
        false,
        RecursiveTraceOpcodeV2::JmtUpdate,
        ControlPhaseV2::Jmt,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Jmt,
        false,
        RecursiveTraceOpcodeV2::JmtUpdate,
        ControlPhaseV2::Jmt,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Jmt,
        false,
        RecursiveTraceOpcodeV2::PromoteChildRoot,
        ControlPhaseV2::Promote,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Promote,
        false,
        RecursiveTraceOpcodeV2::CommitTypedEvent,
        ControlPhaseV2::Commit,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Commit,
        false,
        RecursiveTraceOpcodeV2::FinalizeBlock,
        ControlPhaseV2::TraceClosure,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Idle,
        false,
        RecursiveTraceOpcodeV2::BeginHash,
        ControlPhaseV2::Idle,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Idle,
        false,
        RecursiveTraceOpcodeV2::ShaBlock,
        ControlPhaseV2::Idle,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Idle,
        false,
        RecursiveTraceOpcodeV2::EndHash,
        ControlPhaseV2::Idle,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Replay,
        false,
        RecursiveTraceOpcodeV2::BeginHash,
        ControlPhaseV2::Replay,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Replay,
        false,
        RecursiveTraceOpcodeV2::ShaBlock,
        ControlPhaseV2::Replay,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Replay,
        false,
        RecursiveTraceOpcodeV2::EndHash,
        ControlPhaseV2::Replay,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Precommit,
        false,
        RecursiveTraceOpcodeV2::BeginHash,
        ControlPhaseV2::Precommit,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Precommit,
        false,
        RecursiveTraceOpcodeV2::ShaBlock,
        ControlPhaseV2::Precommit,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Precommit,
        false,
        RecursiveTraceOpcodeV2::EndHash,
        ControlPhaseV2::Precommit,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Challenge,
        false,
        RecursiveTraceOpcodeV2::BeginHash,
        ControlPhaseV2::Challenge,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Challenge,
        false,
        RecursiveTraceOpcodeV2::ShaBlock,
        ControlPhaseV2::Challenge,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Challenge,
        false,
        RecursiveTraceOpcodeV2::EndHash,
        ControlPhaseV2::Challenge,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Net,
        false,
        RecursiveTraceOpcodeV2::BeginHash,
        ControlPhaseV2::Net,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Net,
        false,
        RecursiveTraceOpcodeV2::ShaBlock,
        ControlPhaseV2::Net,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Net,
        false,
        RecursiveTraceOpcodeV2::EndHash,
        ControlPhaseV2::Net,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Jmt,
        false,
        RecursiveTraceOpcodeV2::BeginHash,
        ControlPhaseV2::Jmt,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Jmt,
        false,
        RecursiveTraceOpcodeV2::ShaBlock,
        ControlPhaseV2::Jmt,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Jmt,
        false,
        RecursiveTraceOpcodeV2::EndHash,
        ControlPhaseV2::Jmt,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Promote,
        false,
        RecursiveTraceOpcodeV2::BeginHash,
        ControlPhaseV2::Promote,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Promote,
        false,
        RecursiveTraceOpcodeV2::ShaBlock,
        ControlPhaseV2::Promote,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Promote,
        false,
        RecursiveTraceOpcodeV2::EndHash,
        ControlPhaseV2::Promote,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Commit,
        false,
        RecursiveTraceOpcodeV2::BeginHash,
        ControlPhaseV2::Commit,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Commit,
        false,
        RecursiveTraceOpcodeV2::ShaBlock,
        ControlPhaseV2::Commit,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Commit,
        false,
        RecursiveTraceOpcodeV2::EndHash,
        ControlPhaseV2::Commit,
        false,
    ),
    // `TraceChunk` is an explicit bounded-byte feeder edge, not a generic
    // no-op.  Its source/global context and compression-input relation is
    // constrained separately in `synthesize_hash_control_schedule`.
    ControlTransitionV2::new(
        ControlPhaseV2::Idle,
        false,
        RecursiveTraceOpcodeV2::TraceChunk,
        ControlPhaseV2::Idle,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Replay,
        false,
        RecursiveTraceOpcodeV2::TraceChunk,
        ControlPhaseV2::Replay,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Precommit,
        false,
        RecursiveTraceOpcodeV2::TraceChunk,
        ControlPhaseV2::Precommit,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Challenge,
        false,
        RecursiveTraceOpcodeV2::TraceChunk,
        ControlPhaseV2::Challenge,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Net,
        false,
        RecursiveTraceOpcodeV2::TraceChunk,
        ControlPhaseV2::Net,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Jmt,
        false,
        RecursiveTraceOpcodeV2::TraceChunk,
        ControlPhaseV2::Jmt,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Promote,
        false,
        RecursiveTraceOpcodeV2::TraceChunk,
        ControlPhaseV2::Promote,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::Commit,
        false,
        RecursiveTraceOpcodeV2::TraceChunk,
        ControlPhaseV2::Commit,
        false,
    ),
    // The final canonical source record is included in the one global trace
    // transcript.  Its local hash controls and chunks therefore run after
    // FINALIZE_BLOCK, followed by exactly one schema-bound global END_HASH.
    // This explicit closure phase is not a generic post-final no-op path.
    ControlTransitionV2::new(
        ControlPhaseV2::TraceClosure,
        false,
        RecursiveTraceOpcodeV2::BeginHash,
        ControlPhaseV2::TraceClosure,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::TraceClosure,
        false,
        RecursiveTraceOpcodeV2::ShaBlock,
        ControlPhaseV2::TraceClosure,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::TraceClosure,
        false,
        RecursiveTraceOpcodeV2::EndHash,
        ControlPhaseV2::TraceClosure,
        false,
    ),
    ControlTransitionV2::new(
        ControlPhaseV2::TraceClosure,
        false,
        RecursiveTraceOpcodeV2::TraceChunk,
        ControlPhaseV2::TraceClosure,
        false,
    ),
];

fn control_transition(
    phase: ControlPhaseV2,
    done: bool,
    opcode: RecursiveTraceOpcodeV2,
) -> Result<ControlTransitionV2, ControlTransitionRejectionV2> {
    CONTROL_TRANSITION_TABLE_V2
        .iter()
        .copied()
        .find(|edge| edge.input_phase == phase && edge.input_done == done && edge.opcode == opcode)
        .ok_or(if done {
            ControlTransitionRejectionV2::Finalized
        } else {
            ControlTransitionRejectionV2::IllegalEdge
        })
}

/// One fixed running state. The ranges deliberately reserve all required V2
/// families while the active T2 implementation connects their semantic
/// constraints; unconnected families remain a T2 blocker, never a fallback.
#[derive(Clone)]
pub(crate) struct CheckpointRunningStateV2 {
    cells: [u64; RUNNING_STATE_ARITY_V2],
}

impl CheckpointRunningStateV2 {
    fn initial(anchors: &CheckpointNovaAnchorsV2) -> Self {
        let mut cells = [0_u64; RUNNING_STATE_ARITY_V2];
        for (index, limb) in anchors.limbs.iter().enumerate() {
            cells[index] = u64::from(*limb);
        }
        Self { cells }
    }

    fn with_control(
        anchors: &CheckpointNovaAnchorsV2,
        phase: ControlPhaseV2,
        ordinal: u64,
        done: bool,
    ) -> Self {
        let mut state = Self::initial(anchors);
        state.cells[PHASE_CELL] = phase as u64;
        state.cells[ORDINAL_CELL] = ordinal;
        state.cells[SOURCE_TRACE_ORDINAL_CELL] = ordinal;
        state.cells[DONE_CELL] = u64::from(done);
        state
    }

    fn with_source_event(mut self, event: &NovaTypedSourceEventV2) -> Self {
        for (state_index, limb) in
            (SOURCE_EVENT_DIGEST_START..SOURCE_EVENT_DIGEST_END).zip(event.payload_digest_limbs)
        {
            self.cells[state_index] = u64::from(limb);
        }
        self
    }

    fn with_trace_authority(mut self, authority: &NovaTraceRootAuthorityV2) -> Self {
        for (state_index, root_limb) in
            (EXPECTED_TRACE_ROOT_START..EXPECTED_TRACE_ROOT_END).zip(authority.expected_final_root)
        {
            self.cells[state_index] = root_limb;
        }
        for (state_index, digest_limb) in (EXPECTED_TRACE_DIGEST_START..EXPECTED_TRACE_DIGEST_END)
            .zip(authority.expected_trace_digest)
        {
            self.cells[state_index] = u64::from(digest_limb);
        }
        self
    }

    #[cfg(test)]
    fn with_trace_root_limb(mut self, index: usize, value: u64) -> Self {
        self.cells[SOURCE_TRACE_ROOT_START + index] = value;
        self
    }

    #[cfg(test)]
    fn with_trace_root(mut self, root: [u64; DIGEST_LIMBS]) -> Self {
        for (state_index, root_limb) in (SOURCE_TRACE_ROOT_START..SOURCE_TRACE_ROOT_END).zip(root) {
            self.cells[state_index] = root_limb;
        }
        self
    }

    fn scalars(&self) -> Vec<Scalar> {
        self.cells.iter().copied().map(Scalar::from).collect()
    }
}

/// Immutable authority-supplied terminal commitment for the private source
/// trace accumulator. It is state, never a host-side acceptance result.
#[derive(Clone, Copy)]
pub(crate) struct NovaTraceRootAuthorityV2 {
    expected_final_root: [u64; DIGEST_LIMBS],
    expected_trace_digest: [u16; DIGEST_LIMBS],
}

impl NovaTraceRootAuthorityV2 {
    fn new(expected_final_root: [u64; DIGEST_LIMBS]) -> Self {
        Self {
            expected_final_root,
            expected_trace_digest: [0_u16; DIGEST_LIMBS],
        }
    }
}

/// The invariant digest limbs that every Nova state binds on every step.
#[derive(Clone)]
pub(crate) struct CheckpointNovaAnchorsV2 {
    limbs: [u16; ANCHOR_CELLS],
}

impl CheckpointNovaAnchorsV2 {
    fn zero() -> Self {
        Self {
            limbs: [0_u16; ANCHOR_CELLS],
        }
    }

    #[cfg(test)]
    fn for_test() -> Self {
        let mut limbs = [0_u16; ANCHOR_CELLS];
        for (index, limb) in limbs.iter_mut().enumerate() {
            *limb = u16::try_from(index + 1).expect("fixed anchor limb fits in u16");
        }
        Self { limbs }
    }
}

/// The fixed canonical-record header retained from a source event while its
/// variable payload is consumed only through bounded `TraceChunk` rows.  This
/// is identity metadata, not a second record serialization or a payload arena.
#[derive(Clone)]
struct SourceRecordIdentityWitnessV2 {
    opcode: u8,
    ordinal: u64,
    object_id: [u8; 32],
    payload_len: u32,
}

impl SourceRecordIdentityWitnessV2 {
    fn inactive() -> Self {
        Self {
            opcode: 0,
            ordinal: 0,
            object_id: [0_u8; 32],
            payload_len: 0,
        }
    }

    fn from_source_record(event: &RecursiveTraceEventV2) -> Result<Self, RecursiveV2Error> {
        let payload_len =
            u32::try_from(event.payload().len()).map_err(|_| RecursiveV2Error::Limit)?;
        Ok(Self {
            opcode: event.opcode() as u8,
            ordinal: event.ordinal(),
            object_id: event.object_id(),
            payload_len,
        })
    }
}

/// One typed source event in the only recursive V2 owner. Its digest commits
/// the canonical source record, including the opcode, ordinal, object ID, and
/// bounded payload, without placing raw payload bytes in the circuit witness.
#[derive(Clone)]
pub(crate) struct NovaTypedSourceEventV2 {
    phase: ControlPhaseV2,
    opcode: RecursiveTraceOpcodeV2,
    ordinal: u64,
    payload_digest_limbs: [u16; DIGEST_LIMBS],
    next_payload_digest_limbs: [u16; DIGEST_LIMBS],
    hash_control: HashControlWitnessV2,
    sha_compression: ShaCompressionLaneWitnessV2,
    trace_chunk: TraceChunkWitnessV2,
    source_identity: SourceRecordIdentityWitnessV2,
}

impl NovaTypedSourceEventV2 {
    fn from_source(
        phase: ControlPhaseV2,
        event: &RecursiveTraceEventV2,
    ) -> Result<Self, RecursiveV2Error> {
        let (hash_control, sha_compression, trace_chunk) = match event.opcode() {
            opcode if opcode.is_source_record() => (
                HashControlWitnessV2::from_source_record(event)?,
                ShaCompressionLaneWitnessV2::inactive(),
                TraceChunkWitnessV2::inactive(),
            ),
            RecursiveTraceOpcodeV2::BeginHash
            | RecursiveTraceOpcodeV2::ShaBlock
            | RecursiveTraceOpcodeV2::EndHash => {
                let control = decode_hash_control(event)?;
                let expected_stage = match event.opcode() {
                    RecursiveTraceOpcodeV2::BeginHash => HashControlStageV2::Begin,
                    RecursiveTraceOpcodeV2::ShaBlock => HashControlStageV2::Block,
                    RecursiveTraceOpcodeV2::EndHash => HashControlStageV2::End,
                    _ => unreachable!("hash-control opcode was matched above"),
                };
                if control.stage != expected_stage {
                    return Err(RecursiveV2Error::Canonical);
                }
                let sha_compression = ShaCompressionLaneWitnessV2::from_control(&control)?;
                (
                    HashControlWitnessV2::from_control(&control),
                    sha_compression,
                    TraceChunkWitnessV2::inactive(),
                )
            }
            RecursiveTraceOpcodeV2::TraceChunk => (
                HashControlWitnessV2::inactive(),
                ShaCompressionLaneWitnessV2::inactive(),
                TraceChunkWitnessV2::from_control(decode_trace_chunk_control(event)?),
            ),
            _ => (
                HashControlWitnessV2::inactive(),
                ShaCompressionLaneWitnessV2::inactive(),
                TraceChunkWitnessV2::inactive(),
            ),
        };
        let source_identity = if event.opcode().is_source_record() {
            SourceRecordIdentityWitnessV2::from_source_record(event)?
        } else {
            SourceRecordIdentityWitnessV2::inactive()
        };
        Ok(Self {
            phase,
            opcode: event.opcode(),
            ordinal: event.ordinal(),
            payload_digest_limbs: digest_limbs(event.hash_binding()?),
            next_payload_digest_limbs: [0_u16; DIGEST_LIMBS],
            hash_control,
            sha_compression,
            trace_chunk,
            source_identity,
        })
    }

    fn with_successor(mut self, next: &Self) -> Self {
        self.next_payload_digest_limbs = next.payload_digest_limbs;
        self
    }

    #[cfg(test)]
    fn control(phase: ControlPhaseV2, opcode: RecursiveTraceOpcodeV2, ordinal: u64) -> Self {
        Self {
            phase,
            opcode,
            ordinal,
            payload_digest_limbs: [0_u16; DIGEST_LIMBS],
            next_payload_digest_limbs: [0_u16; DIGEST_LIMBS],
            hash_control: HashControlWitnessV2::inactive(),
            sha_compression: ShaCompressionLaneWitnessV2::inactive(),
            trace_chunk: TraceChunkWitnessV2::inactive(),
            source_identity: SourceRecordIdentityWitnessV2::inactive(),
        }
    }

    #[cfg(test)]
    fn tampered_payload(&self) -> Self {
        let mut event = self.clone();
        event.payload_digest_limbs[0] ^= 1;
        event
    }
}

/// Private decoded values from the sole derived hash-control expander.  The
/// values are all allocated on every step; the stage selectors decide which
/// canonical transition may consume them.
#[derive(Clone)]
struct HashControlWitnessV2 {
    schema: u8,
    stage: u8,
    role: u8,
    source_ordinal: u64,
    source_hash: [u8; 32],
    source_record_bytes: u64,
    trace_event_count: u64,
    trace_byte_count: u64,
    trace_padding_bytes: u64,
    trace_bit_length: u64,
    trace_eof: bool,
    message_bytes: u64,
    block_count: u64,
    block_index: u64,
    byte_offset: u64,
    final_block: bool,
}

impl HashControlWitnessV2 {
    fn inactive() -> Self {
        Self {
            schema: HashControlSchemaV2::SourceRecord as u8,
            stage: 0,
            role: 0,
            source_ordinal: 0,
            source_hash: [0_u8; 32],
            source_record_bytes: 0,
            trace_event_count: 0,
            trace_byte_count: 0,
            trace_padding_bytes: 0,
            trace_bit_length: 0,
            trace_eof: false,
            message_bytes: 0,
            block_count: 0,
            block_index: 0,
            byte_offset: 0,
            final_block: false,
        }
    }

    fn from_source_record(event: &RecursiveTraceEventV2) -> Result<Self, RecursiveV2Error> {
        Ok(Self {
            stage: 1,
            role: TRACE_HASH_ROLE_TAG_V2 as u8,
            source_ordinal: event.ordinal(),
            source_hash: event.hash_binding()?,
            source_record_bytes: event.canonical_len()?,
            ..Self::inactive()
        })
    }

    fn from_control(control: &HashControlBindingV2) -> Self {
        let source = control.source;
        let (block_index, byte_offset, final_block) = control
            .block
            .map(|block| (block.index, block.byte_offset, block.final_block))
            .unwrap_or((0, 0, false));
        Self {
            schema: control.schema as u8,
            // Stage zero is reserved for a non-hash event.  The decoded trace
            // grammar remains frozen at BEGIN=1/BLOCK=2/END=3, so the private
            // circuit selector space shifts only hash controls by one.
            stage: control.stage as u8 + 1,
            role: control.role,
            source_ordinal: source
                .map(|source| source.ordinal)
                .unwrap_or(control.source_ordinal),
            source_hash: control.binding,
            source_record_bytes: 0,
            trace_event_count: control.trace.map(|trace| trace.event_count).unwrap_or(0),
            trace_byte_count: control.trace.map(|trace| trace.byte_count).unwrap_or(0),
            trace_padding_bytes: control.trace.map(|trace| trace.padding_bytes).unwrap_or(0),
            trace_bit_length: control.trace.map(|trace| trace.bit_length).unwrap_or(0),
            trace_eof: control.trace.map(|trace| trace.eof).unwrap_or(false),
            message_bytes: control.message_bytes,
            block_count: control.block_count,
            block_index,
            byte_offset,
            final_block,
        }
    }
}

/// Exact witness bytes for one FIPS compression invocation. The native trace
/// expander owns framing and produces these values; this circuit only proves
/// the one already-framed 64-byte compression transition.
#[derive(Clone)]
struct ShaCompressionLaneWitnessV2 {
    ordinal: u64,
    block: [u8; 64],
    chaining_before: [u32; 8],
    chaining_after: [u32; 8],
}

impl ShaCompressionLaneWitnessV2 {
    fn inactive() -> Self {
        Self {
            ordinal: 0,
            block: [0_u8; 64],
            chaining_before: [0_u32; 8],
            chaining_after: [0_u32; 8],
        }
    }

    fn from_control(control: &HashControlBindingV2) -> Result<Self, RecursiveV2Error> {
        match control.block {
            Some(block) if control.stage == HashControlStageV2::Block => Ok(Self {
                ordinal: block.index,
                block: block.block,
                chaining_before: block.chaining_before,
                chaining_after: block.chaining_after,
            }),
            None if control.stage != HashControlStageV2::Block => Ok(Self::inactive()),
            _ => Err(RecursiveV2Error::Canonical),
        }
    }
}

/// Fixed-width view of bytes from the sole canonical source-record encoder.
/// It is decoded once from `recursive_trace`; the circuit subsequently binds
/// every field and byte to its bounded byte contexts before a SHA block can
/// consume it.  This is deliberately not a second record serialization.
#[derive(Clone)]
struct TraceChunkWitnessV2 {
    source_ordinal: u64,
    chunk_ordinal: u32,
    chunk_count: u32,
    byte_count: u8,
    bytes: [u8; TRACE_CANONICAL_CHUNK_BYTES_V2],
}

impl TraceChunkWitnessV2 {
    fn inactive() -> Self {
        Self {
            source_ordinal: 0,
            chunk_ordinal: 0,
            chunk_count: 0,
            byte_count: 0,
            bytes: [0_u8; TRACE_CANONICAL_CHUNK_BYTES_V2],
        }
    }

    fn from_control(control: RecursiveTraceChunkControlV2) -> Self {
        Self {
            source_ordinal: control.source_ordinal,
            chunk_ordinal: control.chunk_ordinal,
            chunk_count: control.chunk_count,
            byte_count: control.byte_count,
            bytes: control.bytes,
        }
    }
}

struct AllocatedTraceChunkV2 {
    source_ordinal: AllocatedNum<Scalar>,
    chunk_ordinal: AllocatedNum<Scalar>,
    chunk_count: AllocatedNum<Scalar>,
    byte_count: AllocatedNum<Scalar>,
    byte_count_selectors: Vec<AllocatedBit>,
    bytes: Vec<AllocatedNum<Scalar>>,
}

fn allocate_trace_chunk<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    witness: &TraceChunkWitnessV2,
) -> Result<AllocatedTraceChunkV2, SynthesisError> {
    let source_ordinal = allocate_constant(
        cs.namespace(|| "trace_chunk_source_ordinal"),
        witness.source_ordinal,
    )?;
    let chunk_ordinal = allocate_constant(
        cs.namespace(|| "trace_chunk_ordinal"),
        u64::from(witness.chunk_ordinal),
    )?;
    let chunk_count = allocate_constant(
        cs.namespace(|| "trace_chunk_count"),
        u64::from(witness.chunk_count),
    )?;
    let byte_count = allocate_constant(
        cs.namespace(|| "trace_chunk_byte_count"),
        u64::from(witness.byte_count),
    )?;
    range_bits(
        cs.namespace(|| "trace_chunk_source_ordinal_range"),
        &source_ordinal,
        64,
    )?;
    range_bits(
        cs.namespace(|| "trace_chunk_ordinal_range"),
        &chunk_ordinal,
        32,
    )?;
    range_bits(cs.namespace(|| "trace_chunk_count_range"), &chunk_count, 32)?;
    range_bits(
        cs.namespace(|| "trace_chunk_byte_count_range"),
        &byte_count,
        7,
    )?;
    let byte_count_value = byte_count.get_value().map(scalar_u64);
    // Zero is the unique inactive representation.  A legal TraceChunk is
    // separately selector-gated to 1..=64; keeping zero in this fixed shape
    // lets every non-chunk Nova step allocate the same cells without a hidden
    // witness-dependent branch.
    let mut byte_count_selectors = Vec::with_capacity(TRACE_CANONICAL_CHUNK_BYTES_V2 + 1);
    for candidate in 0..=TRACE_CANONICAL_CHUNK_BYTES_V2 {
        byte_count_selectors.push(AllocatedBit::alloc(
            cs.namespace(|| format!("trace_chunk_byte_count_selector_{candidate}")),
            byte_count_value.map(|value| value == candidate as u64),
        )?);
    }
    enforce_selector_value(
        &mut cs,
        &byte_count,
        &byte_count_selectors,
        |index| index as u64,
        "trace_chunk_byte_count",
    );

    let mut bytes = Vec::with_capacity(TRACE_CANONICAL_CHUNK_BYTES_V2);
    for (index, value) in witness.bytes.into_iter().enumerate() {
        let byte = allocate_constant(
            cs.namespace(|| format!("trace_chunk_byte_{index}")),
            u64::from(value),
        )?;
        range_bits(
            cs.namespace(|| format!("trace_chunk_byte_range_{index}")),
            &byte,
            8,
        )?;
        // Every candidate count at or before this offset makes this byte tail
        // data and therefore forces zero.  This stays in R1CS even though the
        // decoder also rejects non-zero tails.
        for (count_index, selector) in byte_count_selectors.iter().enumerate() {
            if count_index <= index {
                enforce_gated_constant(
                    cs.namespace(|| format!("trace_chunk_zero_tail_{index}_{count_index}")),
                    selector,
                    &byte,
                    0,
                );
            }
        }
        bytes.push(byte);
    }
    Ok(AllocatedTraceChunkV2 {
        source_ordinal,
        chunk_ordinal,
        chunk_count,
        byte_count,
        byte_count_selectors,
        bytes,
    })
}

/// Enforce the unique inactive representation without accepting an active
/// chunk as a generic control no-op.  Active chunks are consumed only by the
/// source/global byte-context relation below.
fn enforce_trace_chunk_representation<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    trace_chunk_selector: &AllocatedBit,
    chunk: &AllocatedTraceChunkV2,
) {
    for (index, value) in [
        &chunk.source_ordinal,
        &chunk.chunk_ordinal,
        &chunk.chunk_count,
        &chunk.byte_count,
    ]
    .into_iter()
    .enumerate()
    {
        enforce_inactive_zero(
            cs.namespace(|| format!("inactive_trace_chunk_field_{index}")),
            trace_chunk_selector,
            value,
        );
    }
    for (index, byte) in chunk.bytes.iter().enumerate() {
        enforce_inactive_zero(
            cs.namespace(|| format!("inactive_trace_chunk_byte_{index}")),
            trace_chunk_selector,
            byte,
        );
    }
    cs.enforce(
        || "inactive_trace_chunk_uses_zero_count_selector",
        |lc| lc + CS::one() - trace_chunk_selector.get_variable(),
        |lc| lc + chunk.byte_count_selectors[0].get_variable() - CS::one(),
        |lc| lc,
    );
    cs.enforce(
        || "active_trace_chunk_has_nonzero_count",
        |lc| lc + trace_chunk_selector.get_variable(),
        |lc| lc + chunk.byte_count_selectors[0].get_variable(),
        |lc| lc,
    );
}

/// Allocated fixed source-record identity.  The header fields are retained
/// until the first bounded chunk derives the canonical byte prefix; payload
/// bytes never enter this object or the running state wholesale.
struct AllocatedSourceRecordIdentityV2 {
    opcode: AllocatedNum<Scalar>,
    ordinal: AllocatedNum<Scalar>,
    object_id: Vec<AllocatedNum<Scalar>>,
    payload_len: AllocatedNum<Scalar>,
}

fn allocate_source_record_identity<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    witness: &SourceRecordIdentityWitnessV2,
) -> Result<AllocatedSourceRecordIdentityV2, SynthesisError> {
    let opcode = allocate_constant(
        cs.namespace(|| "source_record_identity_opcode"),
        u64::from(witness.opcode),
    )?;
    let ordinal = allocate_constant(
        cs.namespace(|| "source_record_identity_ordinal"),
        witness.ordinal,
    )?;
    let payload_len = allocate_constant(
        cs.namespace(|| "source_record_identity_payload_len"),
        u64::from(witness.payload_len),
    )?;
    range_bits(
        cs.namespace(|| "source_record_identity_opcode_range"),
        &opcode,
        8,
    )?;
    range_bits(
        cs.namespace(|| "source_record_identity_ordinal_range"),
        &ordinal,
        64,
    )?;
    range_bits(
        cs.namespace(|| "source_record_identity_payload_len_range"),
        &payload_len,
        32,
    )?;
    let mut object_id = Vec::with_capacity(witness.object_id.len());
    for (index, byte) in witness.object_id.into_iter().enumerate() {
        let byte = allocate_constant(
            cs.namespace(|| format!("source_record_identity_object_id_{index}")),
            u64::from(byte),
        )?;
        range_bits(
            cs.namespace(|| format!("source_record_identity_object_id_range_{index}")),
            &byte,
            8,
        )?;
        object_id.push(byte);
    }
    Ok(AllocatedSourceRecordIdentityV2 {
        opcode,
        ordinal,
        object_id,
        payload_len,
    })
}

fn enforce_inactive_source_record_identity<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    source_selector: &AllocatedBit,
    identity: &AllocatedSourceRecordIdentityV2,
) {
    for (index, value) in [&identity.opcode, &identity.ordinal, &identity.payload_len]
        .into_iter()
        .enumerate()
    {
        enforce_inactive_zero(
            cs.namespace(|| format!("inactive_source_record_identity_field_{index}")),
            source_selector,
            value,
        );
    }
    for (index, byte) in identity.object_id.iter().enumerate() {
        enforce_inactive_zero(
            cs.namespace(|| format!("inactive_source_record_identity_object_id_{index}")),
            source_selector,
            byte,
        );
    }
}

/// Derive little-endian bytes from one range-constrained scalar without
/// accepting a parallel byte witness.  The caller fixes `byte_len`, so this
/// is suitable only for the frozen canonical record header fields.
fn decompose_le_bytes<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    value: &AllocatedNum<Scalar>,
    byte_len: usize,
    label: &str,
) -> Result<Vec<AllocatedNum<Scalar>>, SynthesisError> {
    let byte_len_u32 = u32::try_from(byte_len).map_err(|_| {
        SynthesisError::Unsatisfiable("byte decomposition length overflow".to_owned())
    })?;
    let value_bits = byte_len_u32.checked_mul(8).ok_or_else(|| {
        SynthesisError::Unsatisfiable("byte decomposition bit overflow".to_owned())
    })?;
    let value_bits = usize::try_from(value_bits).map_err(|_| {
        SynthesisError::Unsatisfiable("byte decomposition width overflow".to_owned())
    })?;
    range_bits(
        cs.namespace(|| format!("{label}_value_range")),
        value,
        value_bits,
    )?;
    let value_u64 = value.get_value().map(scalar_u64);
    let mut bytes = Vec::with_capacity(byte_len);
    for index in 0..byte_len {
        let shift = index
            .checked_mul(8)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| {
                SynthesisError::Unsatisfiable("byte decomposition shift overflow".to_owned())
            })?;
        let byte = AllocatedNum::alloc(cs.namespace(|| format!("{label}_byte_{index}")), || {
            value_u64
                .map(|current| Scalar::from((current >> shift) & u64::from(u8::MAX)))
                .ok_or(SynthesisError::AssignmentMissing)
        })?;
        range_bits(
            cs.namespace(|| format!("{label}_byte_range_{index}")),
            &byte,
            8,
        )?;
        bytes.push(byte);
    }
    let coefficients = (0..byte_len)
        .map(|index| {
            let exponent = u32::try_from(index).map_err(|_| {
                SynthesisError::Unsatisfiable("byte decomposition exponent overflow".to_owned())
            })?;
            Ok(Scalar::from(256_u64.pow(exponent)))
        })
        .collect::<Result<Vec<_>, SynthesisError>>()?;
    cs.enforce(
        || format!("{label}_little_endian_reconstruction"),
        |lc| {
            let mut reconstructed = lc - value.get_variable();
            for (coefficient, byte) in coefficients.iter().zip(&bytes) {
                reconstructed = reconstructed + (*coefficient, byte.get_variable());
            }
            reconstructed
        },
        |lc| lc + CS::one(),
        |lc| lc,
    );
    Ok(bytes)
}

/// Materialize the only canonical source-record header from the fixed source
/// identity.  The variable payload is intentionally absent: it will be
/// supplied solely by ordered bounded `TraceChunk` bytes.
fn derive_canonical_header_bytes<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    identity: &AllocatedSourceRecordIdentityV2,
) -> Result<Vec<AllocatedNum<Scalar>>, SynthesisError> {
    let mut header = Vec::with_capacity(TRACE_EVENT_HEADER_BYTES_V2);
    header.push(identity.opcode.clone());
    header.extend(decompose_le_bytes(
        cs.namespace(|| "canonical_header_ordinal"),
        &identity.ordinal,
        8,
        "canonical_header_ordinal",
    )?);
    header.extend(identity.object_id.iter().cloned());
    header.extend(decompose_le_bytes(
        cs.namespace(|| "canonical_header_payload_len"),
        &identity.payload_len,
        4,
        "canonical_header_payload_len",
    )?);
    if header.len() != TRACE_EVENT_HEADER_BYTES_V2 {
        return Err(SynthesisError::Unsatisfiable(
            "canonical source-record header width mismatch".to_owned(),
        ));
    }
    Ok(header)
}

struct AllocatedHashControlV2 {
    schema_selectors: Vec<AllocatedBit>,
    stage_selectors: Vec<AllocatedBit>,
    role: AllocatedNum<Scalar>,
    source_ordinal: AllocatedNum<Scalar>,
    source_hash_bytes: Vec<AllocatedNum<Scalar>>,
    source_hash_limbs: Vec<AllocatedNum<Scalar>>,
    source_record_bytes: AllocatedNum<Scalar>,
    trace_event_count: AllocatedNum<Scalar>,
    trace_byte_count: AllocatedNum<Scalar>,
    trace_padding_bytes: AllocatedNum<Scalar>,
    trace_bit_length: AllocatedNum<Scalar>,
    trace_eof: AllocatedNum<Scalar>,
    message_bytes: AllocatedNum<Scalar>,
    block_count: AllocatedNum<Scalar>,
    block_index: AllocatedNum<Scalar>,
    byte_offset: AllocatedNum<Scalar>,
    final_block: AllocatedNum<Scalar>,
}

impl AllocatedHashControlV2 {
    fn inactive_selector(&self) -> &AllocatedBit {
        &self.stage_selectors[0]
    }

    fn source_schema_selector(&self) -> &AllocatedBit {
        &self.schema_selectors[0]
    }

    fn trace_schema_selector(&self) -> &AllocatedBit {
        &self.schema_selectors[1]
    }

    fn source_selector(&self) -> &AllocatedBit {
        &self.stage_selectors[1]
    }

    fn begin_selector(&self) -> &AllocatedBit {
        &self.stage_selectors[HashControlStageV2::Begin as usize + 1]
    }

    fn block_selector(&self) -> &AllocatedBit {
        &self.stage_selectors[HashControlStageV2::Block as usize + 1]
    }

    fn end_selector(&self) -> &AllocatedBit {
        &self.stage_selectors[HashControlStageV2::End as usize + 1]
    }
}

fn allocate_hash_control<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    witness: &HashControlWitnessV2,
) -> Result<AllocatedHashControlV2, SynthesisError> {
    let schema = allocate_constant(cs.namespace(|| "schema"), u64::from(witness.schema))?;
    let schema_value = schema.get_value().map(scalar_u64);
    let mut schema_selectors = Vec::with_capacity(2);
    for schema_candidate in [
        HashControlSchemaV2::SourceRecord as u64,
        HashControlSchemaV2::TracePrecommit as u64,
    ] {
        schema_selectors.push(AllocatedBit::alloc(
            cs.namespace(|| format!("schema_selector_{schema_candidate}")),
            schema_value.map(|current| current == schema_candidate),
        )?);
    }
    enforce_selector_value(
        &mut cs,
        &schema,
        &schema_selectors,
        |index| match index {
            0 => HashControlSchemaV2::SourceRecord as u64,
            1 => HashControlSchemaV2::TracePrecommit as u64,
            _ => unreachable!("fixed hash-control schema selector"),
        },
        "hash_control_schema",
    );
    range_bits(cs.namespace(|| "schema_range"), &schema, 2)?;

    let stage = allocate_constant(cs.namespace(|| "stage"), u64::from(witness.stage))?;
    let stage_value = stage.get_value().map(scalar_u64);
    let mut stage_selectors = Vec::with_capacity(5);
    for stage_value_candidate in 0_u64..=4 {
        stage_selectors.push(AllocatedBit::alloc(
            cs.namespace(|| format!("stage_selector_{stage_value_candidate}")),
            stage_value.map(|current| current == stage_value_candidate),
        )?);
    }
    enforce_selector_value(
        &mut cs,
        &stage,
        &stage_selectors,
        |index| index as u64,
        "hash_control_stage",
    );
    range_bits(cs.namespace(|| "stage_range"), &stage, 3)?;

    let role = allocate_constant(cs.namespace(|| "role"), u64::from(witness.role))?;
    let source_ordinal =
        allocate_constant(cs.namespace(|| "source_ordinal"), witness.source_ordinal)?;
    let source_record_bytes = allocate_constant(
        cs.namespace(|| "source_record_bytes"),
        witness.source_record_bytes,
    )?;
    let trace_event_count = allocate_constant(
        cs.namespace(|| "trace_event_count"),
        witness.trace_event_count,
    )?;
    let trace_byte_count = allocate_constant(
        cs.namespace(|| "trace_byte_count"),
        witness.trace_byte_count,
    )?;
    let trace_padding_bytes = allocate_constant(
        cs.namespace(|| "trace_padding_bytes"),
        witness.trace_padding_bytes,
    )?;
    let trace_bit_length = allocate_constant(
        cs.namespace(|| "trace_bit_length"),
        witness.trace_bit_length,
    )?;
    let trace_eof = allocate_constant(cs.namespace(|| "trace_eof"), u64::from(witness.trace_eof))?;
    let message_bytes = allocate_constant(cs.namespace(|| "message_bytes"), witness.message_bytes)?;
    let block_count = allocate_constant(cs.namespace(|| "block_count"), witness.block_count)?;
    let block_index = allocate_constant(cs.namespace(|| "block_index"), witness.block_index)?;
    let byte_offset = allocate_constant(cs.namespace(|| "byte_offset"), witness.byte_offset)?;
    let final_block = allocate_constant(
        cs.namespace(|| "final_block"),
        u64::from(witness.final_block),
    )?;
    range_bits(cs.namespace(|| "role_range"), &role, 8)?;
    for (index, selector) in stage_selectors.iter().enumerate().skip(1) {
        enforce_gated_constant(
            cs.namespace(|| format!("control_trace_role_{index}")),
            &selector,
            &role,
            TRACE_HASH_ROLE_TAG_V2,
        );
    }
    // A control ordinal encodes a 39-bit source ordinal and a 24-bit local
    // sequence.  These bounds make the integer encoding exact in the field.
    range_bits(cs.namespace(|| "source_ordinal_range"), &source_ordinal, 39)?;
    range_bits(
        cs.namespace(|| "source_record_bytes_range"),
        &source_record_bytes,
        64,
    )?;
    range_bits(
        cs.namespace(|| "trace_event_count_range"),
        &trace_event_count,
        39,
    )?;
    range_bits(
        cs.namespace(|| "trace_byte_count_range"),
        &trace_byte_count,
        64,
    )?;
    range_bits(
        cs.namespace(|| "trace_padding_bytes_range"),
        &trace_padding_bytes,
        6,
    )?;
    range_bits(
        cs.namespace(|| "trace_bit_length_range"),
        &trace_bit_length,
        64,
    )?;
    range_bits(cs.namespace(|| "trace_eof_range"), &trace_eof, 1)?;
    range_bits(cs.namespace(|| "message_bytes_range"), &message_bytes, 64)?;
    range_bits(cs.namespace(|| "block_count_range"), &block_count, 21)?;
    range_bits(cs.namespace(|| "block_index_range"), &block_index, 21)?;
    range_bits(cs.namespace(|| "byte_offset_range"), &byte_offset, 64)?;
    range_bits(cs.namespace(|| "final_block_range"), &final_block, 1)?;

    let mut source_hash_bytes = Vec::with_capacity(32);
    for (index, byte) in witness.source_hash.into_iter().enumerate() {
        let byte = allocate_constant(
            cs.namespace(|| format!("source_hash_byte_{index}")),
            u64::from(byte),
        )?;
        range_bits(
            cs.namespace(|| format!("source_hash_byte_range_{index}")),
            &byte,
            8,
        )?;
        source_hash_bytes.push(byte);
    }
    let mut source_hash_limbs = Vec::with_capacity(DIGEST_LIMBS);
    for index in 0..DIGEST_LIMBS {
        let bytes = [
            witness.source_hash[index * 2],
            witness.source_hash[index * 2 + 1],
        ];
        let limb = allocate_constant(
            cs.namespace(|| format!("source_hash_limb_{index}")),
            u64::from(u16::from_le_bytes(bytes)),
        )?;
        range_bits(
            cs.namespace(|| format!("source_hash_limb_range_{index}")),
            &limb,
            16,
        )?;
        cs.enforce(
            || format!("source_hash_limb_bytes_{index}"),
            |lc| {
                lc + limb.get_variable()
                    - source_hash_bytes[index * 2].get_variable()
                    - (
                        Scalar::from(256_u64),
                        source_hash_bytes[index * 2 + 1].get_variable(),
                    )
            },
            |lc| lc + CS::one(),
            |lc| lc,
        );
        source_hash_limbs.push(limb);
    }

    let allocated = AllocatedHashControlV2 {
        schema_selectors,
        stage_selectors,
        role,
        source_ordinal,
        source_hash_bytes,
        source_hash_limbs,
        source_record_bytes,
        trace_event_count,
        trace_byte_count,
        trace_padding_bytes,
        trace_bit_length,
        trace_eof,
        message_bytes,
        block_count,
        block_index,
        byte_offset,
        final_block,
    };
    // A non-hash event has no implicit source-record schedule.  Force every
    // decoded control value to its unique zero representation so an ordinary
    // grammar edge (including the live TraceChunk opcode) cannot smuggle a
    // second schedule.
    for (index, value) in [
        &allocated.role,
        &allocated.source_ordinal,
        &allocated.source_record_bytes,
        &allocated.trace_event_count,
        &allocated.trace_byte_count,
        &allocated.trace_padding_bytes,
        &allocated.trace_bit_length,
        &allocated.trace_eof,
        &allocated.message_bytes,
        &allocated.block_count,
        &allocated.block_index,
        &allocated.byte_offset,
        &allocated.final_block,
    ]
    .into_iter()
    .enumerate()
    {
        enforce_gated_constant(
            cs.namespace(|| format!("inactive_control_field_zero_{index}")),
            allocated.inactive_selector(),
            value,
            0,
        );
    }
    for (index, byte) in allocated.source_hash_bytes.iter().enumerate() {
        enforce_gated_constant(
            cs.namespace(|| format!("inactive_control_hash_byte_zero_{index}")),
            allocated.inactive_selector(),
            byte,
            0,
        );
    }
    for (index, value) in [
        &allocated.trace_event_count,
        &allocated.trace_byte_count,
        &allocated.trace_padding_bytes,
        &allocated.trace_bit_length,
        &allocated.trace_eof,
        &allocated.message_bytes,
        &allocated.block_count,
        &allocated.block_index,
        &allocated.byte_offset,
        &allocated.final_block,
    ]
    .into_iter()
    .enumerate()
    {
        enforce_gated_constant(
            cs.namespace(|| format!("source_record_control_field_zero_{index}")),
            allocated.source_selector(),
            value,
            0,
        );
    }
    for (selector_index, selector) in [allocated.begin_selector(), allocated.end_selector()]
        .into_iter()
        .enumerate()
    {
        for (value_index, value) in [
            &allocated.block_index,
            &allocated.byte_offset,
            &allocated.final_block,
        ]
        .into_iter()
        .enumerate()
        {
            enforce_gated_constant(
                cs.namespace(|| {
                    format!("non_block_control_field_zero_{selector_index}_{value_index}")
                }),
                selector,
                value,
                0,
            );
        }
    }
    enforce_gated_constant(
        cs.namespace(|| "source_record_schema"),
        allocated.source_selector(),
        &schema,
        HashControlSchemaV2::SourceRecord as u64,
    );
    for (selector_index, selector) in [
        allocated.begin_selector(),
        allocated.block_selector(),
        allocated.end_selector(),
    ]
    .into_iter()
    .enumerate()
    {
        enforce_gated_constant(
            cs.namespace(|| format!("control_source_record_bytes_zero_{selector_index}")),
            &selector,
            &allocated.source_record_bytes,
            0,
        );
    }
    Ok(allocated)
}

/// A private witness for one derived control event. It contains no native
/// validity result: acceptance follows only from the R1CS constraints.
#[derive(Clone)]
pub(crate) struct NovaStepWitnessV2 {
    event: NovaTypedSourceEventV2,
    next_phase: ControlPhaseV2,
    next_done: bool,
}

impl NovaStepWitnessV2 {
    fn new(done: bool, event: NovaTypedSourceEventV2) -> Result<Self, SynthesisError> {
        let edge = control_transition(event.phase, done, event.opcode)
            .map_err(|rejection| SynthesisError::Unsatisfiable(rejection.message().to_owned()))?;
        let trace_closure_end = event.phase == ControlPhaseV2::TraceClosure
            && event.opcode == RecursiveTraceOpcodeV2::EndHash
            && event.hash_control.schema == HashControlSchemaV2::TracePrecommit as u8
            && event.hash_control.stage == HashControlStageV2::End as u8 + 1;
        Ok(Self {
            event,
            next_phase: if trace_closure_end {
                ControlPhaseV2::Idle
            } else {
                edge.next_phase
            },
            next_done: trace_closure_end || edge.next_done,
        })
    }
}

/// The one private Nova step circuit. Its constraint geometry is independent
/// of opcode and phase: every selector pair and every candidate successor is
/// allocated on every synthesis.
#[derive(Clone)]
pub(crate) struct CheckpointNovaCircuitV2 {
    anchors: CheckpointNovaAnchorsV2,
    witness: NovaStepWitnessV2,
}

impl CheckpointNovaCircuitV2 {
    fn new(anchors: CheckpointNovaAnchorsV2, witness: NovaStepWitnessV2) -> Self {
        Self { anchors, witness }
    }
}

impl StepCircuit<Scalar> for CheckpointNovaCircuitV2 {
    fn arity(&self) -> usize {
        RUNNING_STATE_ARITY_V2
    }

    fn synthesize<CS: ConstraintSystem<Scalar>>(
        &self,
        cs: &mut CS,
        z: &[AllocatedNum<Scalar>],
    ) -> Result<Vec<AllocatedNum<Scalar>>, SynthesisError> {
        if z.len() != RUNNING_STATE_ARITY_V2 {
            return Err(SynthesisError::Unsatisfiable(
                "recursive V2 running-state arity mismatch".to_owned(),
            ));
        }

        for (index, anchor) in self.anchors.limbs.iter().enumerate() {
            enforce_constant(
                cs.namespace(|| format!("anchor_{index}")),
                &z[index],
                *anchor,
            );
            range_bits(
                cs.namespace(|| format!("anchor_range_{index}")),
                &z[index],
                16,
            )?;
        }
        constrain_state_ranges(cs, z)?;

        let event_phase = allocate_constant(
            cs.namespace(|| "source_event_phase"),
            self.witness.event.phase as u64,
        )?;
        let event_opcode = allocate_constant(
            cs.namespace(|| "source_event_opcode"),
            self.witness.event.opcode as u8 as u64,
        )?;
        let event_ordinal = allocate_constant(
            cs.namespace(|| "source_event_ordinal"),
            self.witness.event.ordinal,
        )?;
        range_bits(cs.namespace(|| "source_event_phase_range"), &event_phase, 8)?;
        range_bits(
            cs.namespace(|| "source_event_opcode_range"),
            &event_opcode,
            8,
        )?;
        range_bits(
            cs.namespace(|| "source_event_ordinal_range"),
            &event_ordinal,
            64,
        )?;
        enforce_equal(
            cs.namespace(|| "source_event_phase_matches_state"),
            &z[PHASE_CELL],
            &event_phase,
        );
        let mut source_digest_limbs = Vec::with_capacity(DIGEST_LIMBS);
        let mut next_source_digest_limbs = Vec::with_capacity(DIGEST_LIMBS);
        for (index, ((state_index, limb), next_limb)) in (SOURCE_EVENT_DIGEST_START
            ..SOURCE_EVENT_DIGEST_END)
            .zip(self.witness.event.payload_digest_limbs)
            .zip(self.witness.event.next_payload_digest_limbs)
            .enumerate()
        {
            let event_limb = allocate_constant(
                cs.namespace(|| format!("source_event_digest_{index}")),
                u64::from(limb),
            )?;
            range_bits(
                cs.namespace(|| format!("source_event_digest_range_{index}")),
                &event_limb,
                16,
            )?;
            enforce_equal(
                cs.namespace(|| format!("source_event_digest_matches_state_{index}")),
                &z[state_index],
                &event_limb,
            );
            source_digest_limbs.push(event_limb);
            let next_event_limb = allocate_constant(
                cs.namespace(|| format!("next_source_event_digest_{index}")),
                u64::from(next_limb),
            )?;
            range_bits(
                cs.namespace(|| format!("next_source_event_digest_range_{index}")),
                &next_event_limb,
                16,
            )?;
            next_source_digest_limbs.push((state_index, next_event_limb));
        }

        let phase_selectors = phase_selectors(cs, &event_phase)?;
        let opcode_selectors = opcode_selectors(cs, &event_opcode)?;
        let trace_chunk_opcode = allocate_constant(
            cs.namespace(|| "trace_chunk_opcode"),
            RecursiveTraceOpcodeV2::TraceChunk as u8 as u64,
        )?;
        let trace_chunk_selector = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| "trace_chunk_opcode_selector"),
            &event_opcode,
            &trace_chunk_opcode,
        )?;
        let trace_chunk = allocate_trace_chunk(
            cs.namespace(|| "trace_chunk"),
            &self.witness.event.trace_chunk,
        )?;
        enforce_trace_chunk_representation(
            cs.namespace(|| "trace_chunk_representation"),
            &trace_chunk_selector,
            &trace_chunk,
        );
        let source_opcode = AllocatedBit::alloc(
            cs.namespace(|| "source_opcode_selector"),
            Some(self.witness.event.opcode.is_source_record()),
        )?;
        let mut source_opcode_lc = nova_snark::frontend::LinearCombination::zero();
        for (index, opcode) in opcode_list().iter().copied().enumerate() {
            if opcode.is_source_record() {
                source_opcode_lc = source_opcode_lc + opcode_selectors[index].get_variable();
            }
        }
        cs.enforce(
            || "source_opcode_selector_matches_opcode_alphabet",
            |lc| lc + source_opcode.get_variable() - &source_opcode_lc,
            |lc| lc + CS::one(),
            |lc| lc,
        );
        let hash_control = allocate_hash_control(
            cs.namespace(|| "derived_hash_control"),
            &self.witness.event.hash_control,
        )?;
        cs.enforce(
            || "source_stage_matches_source_opcode",
            |lc| lc + hash_control.source_selector().get_variable() - source_opcode.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc,
        );
        let source_identity = allocate_source_record_identity(
            cs.namespace(|| "source_record_identity"),
            &self.witness.event.source_identity,
        )?;
        enforce_inactive_source_record_identity(
            cs.namespace(|| "source_record_identity_inactive"),
            &source_opcode,
            &source_identity,
        );
        let source_header = derive_canonical_header_bytes(
            cs.namespace(|| "canonical_source_record_header"),
            &source_identity,
        )?;
        for (index, byte) in source_header.iter().enumerate() {
            enforce_inactive_zero(
                cs.namespace(|| format!("inactive_canonical_source_header_byte_{index}")),
                &source_opcode,
                byte,
            );
        }
        enforce_gated_equal(
            cs.namespace(|| "source_record_identity_opcode_matches_event"),
            &source_opcode,
            &source_identity.opcode,
            &event_opcode,
        );
        enforce_gated_equal(
            cs.namespace(|| "source_record_identity_ordinal_matches_event"),
            &source_opcode,
            &source_identity.ordinal,
            &event_ordinal,
        );
        cs.enforce(
            || "source_record_identity_length_matches_hash_control",
            |lc| lc + source_opcode.get_variable(),
            |lc| {
                lc + hash_control.source_record_bytes.get_variable()
                    - source_identity.payload_len.get_variable()
                    - (Scalar::from(TRACE_EVENT_HEADER_BYTES_V2 as u64), CS::one())
            },
            |lc| lc,
        );
        let mut hash_stage_lc = nova_snark::frontend::LinearCombination::zero();
        let mut hash_opcode_lc = nova_snark::frontend::LinearCombination::zero();
        for (index, (selector, opcode)) in [
            (
                hash_control.begin_selector(),
                RecursiveTraceOpcodeV2::BeginHash,
            ),
            (
                hash_control.block_selector(),
                RecursiveTraceOpcodeV2::ShaBlock,
            ),
            (hash_control.end_selector(), RecursiveTraceOpcodeV2::EndHash),
        ]
        .into_iter()
        .enumerate()
        {
            hash_stage_lc = hash_stage_lc + selector.get_variable();
            let opcode_selector = opcode_selectors
                .iter()
                .zip(opcode_list())
                .find_map(|(selector, candidate)| (candidate == opcode).then_some(selector))
                .ok_or_else(|| {
                    SynthesisError::Unsatisfiable("hash-control opcode selector missing".to_owned())
                })?;
            hash_opcode_lc = hash_opcode_lc + opcode_selector.get_variable();
            enforce_gated_constant(
                cs.namespace(|| format!("hash_control_opcode_{index}")),
                selector,
                &event_opcode,
                opcode as u8 as u64,
            );
        }
        cs.enforce(
            || "hash_control_stage_matches_opcode_alphabet",
            |lc| lc + &hash_stage_lc - &hash_opcode_lc,
            |lc| lc + CS::one(),
            |lc| lc,
        );
        for (index, opcode_index) in [
            RecursiveTraceOpcodeV2::BeginHash as usize - 1,
            RecursiveTraceOpcodeV2::ShaBlock as usize - 1,
            RecursiveTraceOpcodeV2::EndHash as usize - 1,
        ]
        .into_iter()
        .enumerate()
        {
            let hash_opcode = opcode_selectors.get(opcode_index).ok_or_else(|| {
                SynthesisError::Unsatisfiable("hash-control selector missing".to_owned())
            })?;
            cs.enforce(
                || format!("source record is not a hash-control opcode_{index}"),
                |lc| lc + hash_control.source_selector().get_variable(),
                |lc| lc + hash_opcode.get_variable(),
                |lc| lc,
            );
        }
        let sha_block_selector = hash_control.block_selector();
        for (index, (event_limb, hash_limb)) in source_digest_limbs
            .iter()
            .zip(&hash_control.source_hash_limbs)
            .enumerate()
        {
            enforce_gated_equal(
                cs.namespace(|| format!("source_hash_matches_source_digest_{index}")),
                hash_control.source_selector(),
                event_limb,
                hash_limb,
            );
        }
        let done = allocate_state_bit(cs.namespace(|| "done"), &z[DONE_CELL])?;
        let not_done = AllocatedBit::alloc(
            cs.namespace(|| "not_done"),
            done.get_value().map(|value| !value),
        )?;
        cs.enforce(
            || "done complement",
            |lc| lc + done.get_variable() + not_done.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + CS::one(),
        );

        let mut valid_lc = nova_snark::frontend::LinearCombination::zero();
        let mut next_phase_lc = None;
        // Most rows leave `done` at zero. The only one-valued contribution is
        // the schema-bound global END_HASH in TraceClosure, so start from an
        // explicit zero relation instead of relying on a host default.
        let mut next_done_lc = Some(nova_snark::frontend::LinearCombination::zero());
        let mut prior_opcode_lc = None;
        let mut final_gate = None;

        for (phase_index, phase) in ControlPhaseV2::ALL.iter().copied().enumerate() {
            for (opcode_index, opcode) in opcode_list().iter().copied().enumerate() {
                let phase_opcode = AllocatedBit::and(
                    cs.namespace(|| format!("phase_opcode_{phase_index}_{opcode_index}")),
                    &phase_selectors[phase_index],
                    &opcode_selectors[opcode_index],
                )?;
                let gate = AllocatedBit::and(
                    cs.namespace(|| format!("active_{phase_index}_{opcode_index}")),
                    &phase_opcode,
                    &not_done,
                )?;
                if let Ok(edge) = control_transition(phase, false, opcode) {
                    valid_lc = valid_lc + gate.get_variable();
                    if phase == ControlPhaseV2::TraceClosure
                        && opcode == RecursiveTraceOpcodeV2::EndHash
                    {
                        let source_end = AllocatedBit::and(
                            cs.namespace(|| "trace_closure_source_end"),
                            &gate,
                            hash_control.source_schema_selector(),
                        )?;
                        let trace_end = AllocatedBit::and(
                            cs.namespace(|| "trace_closure_global_end"),
                            &gate,
                            hash_control.trace_schema_selector(),
                        )?;
                        add_weighted_bit(
                            &mut next_phase_lc,
                            &source_end,
                            Scalar::from(ControlPhaseV2::TraceClosure as u64),
                        );
                        add_weighted_bit(
                            &mut next_phase_lc,
                            &trace_end,
                            Scalar::from(ControlPhaseV2::Idle as u64),
                        );
                        add_weighted_bit(&mut next_done_lc, &trace_end, Scalar::from(1_u64));
                        final_gate = Some(trace_end);
                    } else {
                        add_weighted_bit(
                            &mut next_phase_lc,
                            &gate,
                            Scalar::from(edge.next_phase as u64),
                        );
                        if edge.next_done {
                            add_weighted_bit(&mut next_done_lc, &gate, Scalar::from(1_u64));
                        }
                    }
                    add_weighted_bit(
                        &mut prior_opcode_lc,
                        &gate,
                        Scalar::from(opcode as u8 as u64),
                    );
                    if edge.next_done && opcode == RecursiveTraceOpcodeV2::FinalizeBlock {
                        final_gate = Some(gate);
                    }
                }
            }
        }
        cs.enforce(
            || "one legal transition",
            |lc| lc + &valid_lc,
            |lc| lc + CS::one(),
            |lc| lc + CS::one(),
        );

        range_bits(cs.namespace(|| "state_ordinal_range"), &z[ORDINAL_CELL], 64)?;
        range_bits(
            cs.namespace(|| "source_trace_ordinal_range"),
            &z[SOURCE_TRACE_ORDINAL_CELL],
            64,
        )?;
        range_bits(
            cs.namespace(|| "source_trace_byte_count_range"),
            &z[SOURCE_TRACE_BYTE_COUNT_CELL],
            64,
        )?;

        let out_phase =
            allocate_constant(cs.namespace(|| "out_phase"), self.witness.next_phase as u64)?;
        let out_opcode = allocate_constant(
            cs.namespace(|| "out_prior_opcode"),
            self.witness.event.opcode as u8 as u64,
        )?;
        // `ORDINAL_CELL` is a monotonic Nova-step counter.  It deliberately
        // does not reuse the high-bit encoded control-event ordinal.
        let out_ordinal = allocate_incremented(
            cs.namespace(|| "out_ordinal"),
            &z[ORDINAL_CELL],
            "out_ordinal_value",
        )?;
        let out_done = allocate_constant(
            cs.namespace(|| "out_done"),
            u64::from(self.witness.next_done),
        )?;
        range_bits(cs.namespace(|| "out_phase_range"), &out_phase, 8)?;
        range_bits(cs.namespace(|| "out_opcode_range"), &out_opcode, 8)?;
        range_bits(cs.namespace(|| "out_ordinal_range"), &out_ordinal, 64)?;
        range_bits(cs.namespace(|| "out_done_range"), &out_done, 1)?;

        enforce_lc_equal(
            cs.namespace(|| "next_phase_relation"),
            &out_phase,
            next_phase_lc,
        )?;
        enforce_lc_equal(
            cs.namespace(|| "next_done_relation"),
            &out_done,
            next_done_lc,
        )?;
        enforce_lc_equal(
            cs.namespace(|| "prior_opcode_relation"),
            &out_opcode,
            prior_opcode_lc,
        )?;
        let replay_outputs =
            synthesize_replay_grammar(cs.namespace(|| "replay_grammar"), z, &opcode_selectors)?;
        let replay_payload = synthesize_replay_payload(
            cs.namespace(|| "replay_payload"),
            z,
            &opcode_selectors,
            &trace_chunk_selector,
            &trace_chunk,
            &hash_control,
        )?;
        let precommit_payload = synthesize_uniqueness_precommit_payload(
            cs.namespace(|| "uniqueness_precommit_payload"),
            z,
            &opcode_selectors,
            &trace_chunk_selector,
            &trace_chunk,
            &hash_control,
        )?;
        let challenge_payload = synthesize_uniqueness_challenge_payload(
            cs.namespace(|| "uniqueness_challenge_payload"),
            z,
            &opcode_selectors,
            &trace_chunk_selector,
            &trace_chunk,
            &hash_control,
        )?;
        let net_payload = synthesize_net_merge_payload(
            cs.namespace(|| "net_merge_payload"),
            z,
            &opcode_selectors,
            &trace_chunk_selector,
            &trace_chunk,
            &hash_control,
        )?;
        let sha_outputs = synthesize_sha_compression_lane(
            cs.namespace(|| "sha_compression_lane"),
            sha_block_selector,
            z,
            &hash_control,
            &self.witness.event.sha_compression,
        )?;
        let schedule_outputs = synthesize_hash_control_schedule(
            cs.namespace(|| "hash_control_schedule"),
            z,
            &event_ordinal,
            &hash_control,
            &trace_chunk_selector,
            &trace_chunk,
            &source_header,
            &sha_outputs,
        )?;

        let final_gate = final_gate.ok_or_else(|| {
            SynthesisError::Unsatisfiable("finalize transition missing from table".to_owned())
        })?;
        let zero = allocate_constant(cs.namespace(|| "finalize_source_bytes_zero"), 0)?;
        let replayed_source_bytes_empty = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| "finalize_replayed_source_bytes_empty"),
            &z[SOURCE_TRACE_BYTE_COUNT_CELL],
            &zero,
        )?;
        cs.enforce(
            || "finalize_requires_replayed_source_bytes",
            |lc| lc + final_gate.get_variable(),
            |lc| lc + replayed_source_bytes_empty.get_variable(),
            |lc| lc,
        );
        let mut output = z.to_vec();
        output[PHASE_CELL] = out_phase;
        output[PRIOR_OPCODE_CELL] = out_opcode;
        output[ORDINAL_CELL] = out_ordinal;
        output[SOURCE_TRACE_ORDINAL_CELL] = schedule_outputs.source_trace_ordinal;
        output[SOURCE_TRACE_BYTE_COUNT_CELL] = schedule_outputs.source_trace_byte_count;
        output[DONE_CELL] = out_done;
        // The schema-bound global TRACE END is the only final event.  It clears
        // the replay grammar together with the byte contexts so finalized Idle
        // cannot retain a replay prefix for a later proof step.
        output[REPLAY_MODE_CELL] = select_bit_num(
            cs.namespace(|| "final_replay_mode"),
            &final_gate,
            &zero,
            &replay_outputs.mode,
            "value",
        )?;
        output[REPLAY_INPUT_COUNT_CELL] = select_bit_num(
            cs.namespace(|| "final_replay_input_count"),
            &final_gate,
            &zero,
            &replay_outputs.input_count,
            "value",
        )?;
        output[REPLAY_OUTPUT_COUNT_CELL] = select_bit_num(
            cs.namespace(|| "final_replay_output_count"),
            &final_gate,
            &zero,
            &replay_outputs.output_count,
            "value",
        )?;
        for (state_index, value) in replay_payload.cells {
            output[state_index] = select_bit_num(
                cs.namespace(|| format!("final_replay_payload_{state_index}")),
                &final_gate,
                &zero,
                &value,
                "value",
            )?;
        }
        for (state_index, value) in precommit_payload.cells {
            output[state_index] = select_bit_num(
                cs.namespace(|| format!("final_uniqueness_precommit_payload_{state_index}")),
                &final_gate,
                &zero,
                &value,
                "value",
            )?;
        }
        for (state_index, value) in challenge_payload.cells {
            output[state_index] = select_bit_num(
                cs.namespace(|| format!("final_uniqueness_challenge_payload_{state_index}")),
                &final_gate,
                &zero,
                &value,
                "value",
            )?;
        }
        for (state_index, value) in net_payload.cells {
            output[state_index] = select_bit_num(
                cs.namespace(|| format!("final_net_merge_payload_{state_index}")),
                &final_gate,
                &zero,
                &value,
                "value",
            )?;
        }
        for (state_index, value) in schedule_outputs.cells {
            output[state_index] = value;
        }
        for (state_index, next_event_limb) in next_source_digest_limbs {
            output[state_index] = next_event_limb;
        }
        for (index, ((root_index, expected_root_index), event_limb)) in (SOURCE_TRACE_ROOT_START
            ..SOURCE_TRACE_ROOT_END)
            .zip(EXPECTED_TRACE_ROOT_START..EXPECTED_TRACE_ROOT_END)
            .zip(source_digest_limbs)
            .enumerate()
        {
            let next_root = AllocatedNum::alloc(
                cs.namespace(|| format!("source_trace_root_{index}")),
                || match (z[root_index].get_value(), event_limb.get_value()) {
                    (Some(current), Some(digest_limb)) => Ok(current + digest_limb),
                    _ => Err(SynthesisError::AssignmentMissing),
                },
            )?;
            enforce_sum(
                cs.namespace(|| format!("source_trace_root_successor_{index}")),
                &next_root,
                &z[root_index],
                &event_limb,
            );
            cs.enforce(
                || format!("final_trace_root_matches_authority_{index}"),
                |lc| lc + final_gate.get_variable(),
                |lc| lc + next_root.get_variable() - z[expected_root_index].get_variable(),
                |lc| lc,
            );
            output[root_index] = next_root;
        }
        // TraceClosure's schema-bound global END_HASH is the final consuming
        // event. Check the successor, not the pre-END input, so the last FIPS
        // transition is required to clear every transient context before Idle.
        for index in transient_cells() {
            cs.enforce(
                || format!("final_idle_zero_{index}"),
                |lc| lc + final_gate.get_variable(),
                |lc| lc + output[index].get_variable(),
                |lc| lc,
            );
        }
        Ok(output)
    }
}

fn opcode_list() -> [RecursiveTraceOpcodeV2; 14] {
    [
        RecursiveTraceOpcodeV2::BeginBlock,
        RecursiveTraceOpcodeV2::ReplayInput,
        RecursiveTraceOpcodeV2::ReplayOutput,
        RecursiveTraceOpcodeV2::BeginHash,
        RecursiveTraceOpcodeV2::ShaBlock,
        RecursiveTraceOpcodeV2::EndHash,
        RecursiveTraceOpcodeV2::UniquenessPrecommit,
        RecursiveTraceOpcodeV2::UniquenessChallenge,
        RecursiveTraceOpcodeV2::NetMerge,
        RecursiveTraceOpcodeV2::JmtUpdate,
        RecursiveTraceOpcodeV2::PromoteChildRoot,
        RecursiveTraceOpcodeV2::CommitTypedEvent,
        RecursiveTraceOpcodeV2::FinalizeBlock,
        RecursiveTraceOpcodeV2::TraceChunk,
    ]
}

struct ReplayGrammarOutputsV2 {
    mode: AllocatedNum<Scalar>,
    input_count: AllocatedNum<Scalar>,
    output_count: AllocatedNum<Scalar>,
}

/// Constrain the replay prefix directly from the frozen source-record opcode
/// alphabet.  This is deliberately a grammar/counter relation, not a native
/// replay verdict or a digest proxy for the replay payload: later T2 rows bind
/// the decoded canonical payload semantics through the same source-byte path.
fn synthesize_replay_grammar<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    z: &[AllocatedNum<Scalar>],
    opcode_selectors: &[AllocatedBit],
) -> Result<ReplayGrammarOutputsV2, SynthesisError> {
    let selector = |opcode: RecursiveTraceOpcodeV2| {
        opcode_selectors
            .get(usize::from(opcode as u8).saturating_sub(1))
            .ok_or_else(|| {
                SynthesisError::Unsatisfiable("replay opcode selector missing".to_owned())
            })
    };
    let begin_block = selector(RecursiveTraceOpcodeV2::BeginBlock)?;
    let replay_input = selector(RecursiveTraceOpcodeV2::ReplayInput)?;
    let replay_output = selector(RecursiveTraceOpcodeV2::ReplayOutput)?;
    let uniqueness_precommit = selector(RecursiveTraceOpcodeV2::UniquenessPrecommit)?;

    let inputs_mode =
        allocate_constant(cs.namespace(|| "inputs_mode"), ReplayModeV2::Inputs as u64)?;
    let outputs_mode = allocate_constant(
        cs.namespace(|| "outputs_mode"),
        ReplayModeV2::Outputs as u64,
    )?;
    let zero = allocate_constant(cs.namespace(|| "zero"), 0)?;
    let mode_is_inputs = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "mode_is_inputs"),
        &z[REPLAY_MODE_CELL],
        &inputs_mode,
    )?;
    let mode_is_outputs = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "mode_is_outputs"),
        &z[REPLAY_MODE_CELL],
        &outputs_mode,
    )?;
    cs.enforce(
        || "mode_is_frozen_replay_alphabet",
        |lc| lc + mode_is_inputs.get_variable() + mode_is_outputs.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc + CS::one(),
    );

    // A new block starts its replay prefix with both cardinalities empty.  The
    // first output irreversibly changes the mode, so a later ReplayInput cannot
    // be hidden behind the coarser ControlPhaseV2::Replay self-loop.
    enforce_gated_constant(
        cs.namespace(|| "begin_requires_inputs_mode"),
        begin_block,
        &z[REPLAY_MODE_CELL],
        ReplayModeV2::Inputs as u64,
    );
    for (label, cell) in [
        ("begin_input_count_zero", REPLAY_INPUT_COUNT_CELL),
        ("begin_output_count_zero", REPLAY_OUTPUT_COUNT_CELL),
    ] {
        enforce_gated_constant(cs.namespace(|| label), begin_block, &z[cell], 0);
    }
    enforce_gated_constant(
        cs.namespace(|| "input_requires_inputs_mode"),
        replay_input,
        &z[REPLAY_MODE_CELL],
        ReplayModeV2::Inputs as u64,
    );
    let input_or_output = allocate_bit_or(
        cs.namespace(|| "output_accepts_replay_prefix"),
        &mode_is_inputs,
        &mode_is_outputs,
        "value",
    )?;
    let input_or_output_num = bit_as_num(
        cs.namespace(|| "output_accepts_replay_prefix_num"),
        &input_or_output,
        "value",
    )?;
    enforce_gated_constant(
        cs.namespace(|| "output_requires_replay_prefix"),
        replay_output,
        &input_or_output_num,
        1,
    );

    let input_is_empty = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "input_count_is_empty"),
        &z[REPLAY_INPUT_COUNT_CELL],
        &zero,
    )?;
    let output_is_empty = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "output_count_is_empty"),
        &z[REPLAY_OUTPUT_COUNT_CELL],
        &zero,
    )?;
    // The native grammar allows a no-op precommit only while still in the
    // input prefix.  Otherwise precommit follows at least one output.  In both
    // cases the two replay sets are jointly empty or jointly nonempty.
    let empty_input_prefix = AllocatedBit::and(
        cs.namespace(|| "empty_input_prefix"),
        &mode_is_inputs,
        &input_is_empty,
    )?;
    let precommit_mode = allocate_bit_or(
        cs.namespace(|| "precommit_legal_mode"),
        &mode_is_outputs,
        &empty_input_prefix,
        "value",
    )?;
    let precommit_mode_num = bit_as_num(
        cs.namespace(|| "precommit_legal_mode_num"),
        &precommit_mode,
        "value",
    )?;
    enforce_gated_constant(
        cs.namespace(|| "precommit_requires_closed_replay_prefix"),
        uniqueness_precommit,
        &precommit_mode_num,
        1,
    );
    cs.enforce(
        || "precommit_requires_jointly_empty_or_nonempty_sets",
        |lc| lc + uniqueness_precommit.get_variable(),
        |lc| lc + input_is_empty.get_variable() - output_is_empty.get_variable(),
        |lc| lc,
    );

    let next_input_count = AllocatedNum::alloc(cs.namespace(|| "next_input_count"), || {
        match (
            z[REPLAY_INPUT_COUNT_CELL].get_value(),
            replay_input.get_value(),
        ) {
            (Some(count), Some(replay)) => {
                let count = scalar_u64(count);
                count
                    .checked_add(u64::from(replay))
                    .map(Scalar::from)
                    .ok_or_else(|| {
                        SynthesisError::Unsatisfiable("replay input count overflow".to_owned())
                    })
            }
            _ => Err(SynthesisError::AssignmentMissing),
        }
    })?;
    cs.enforce(
        || "input_count_successor",
        |lc| {
            lc + next_input_count.get_variable()
                - z[REPLAY_INPUT_COUNT_CELL].get_variable()
                - replay_input.get_variable()
        },
        |lc| lc + CS::one(),
        |lc| lc,
    );
    let next_output_count = AllocatedNum::alloc(cs.namespace(|| "next_output_count"), || {
        match (
            z[REPLAY_OUTPUT_COUNT_CELL].get_value(),
            replay_output.get_value(),
        ) {
            (Some(count), Some(replay)) => {
                let count = scalar_u64(count);
                count
                    .checked_add(u64::from(replay))
                    .map(Scalar::from)
                    .ok_or_else(|| {
                        SynthesisError::Unsatisfiable("replay output count overflow".to_owned())
                    })
            }
            _ => Err(SynthesisError::AssignmentMissing),
        }
    })?;
    cs.enforce(
        || "output_count_successor",
        |lc| {
            lc + next_output_count.get_variable()
                - z[REPLAY_OUTPUT_COUNT_CELL].get_variable()
                - replay_output.get_variable()
        },
        |lc| lc + CS::one(),
        |lc| lc,
    );
    // These state cells are 16-bit limbs in the fixed running state.  Apply
    // the range to each successor here, not only on a later Nova step, so an
    // active increment is non-wrapping at the current R1CS boundary.
    range_bits(
        cs.namespace(|| "next_input_count_range"),
        &next_input_count,
        16,
    )?;
    range_bits(
        cs.namespace(|| "next_output_count_range"),
        &next_output_count,
        16,
    )?;

    let after_begin = select_bit_num(
        cs.namespace(|| "mode_after_begin"),
        begin_block,
        &inputs_mode,
        &z[REPLAY_MODE_CELL],
        "value",
    )?;
    let after_input = select_bit_num(
        cs.namespace(|| "mode_after_input"),
        replay_input,
        &inputs_mode,
        &after_begin,
        "value",
    )?;
    let next_mode = select_bit_num(
        cs.namespace(|| "mode_after_output"),
        replay_output,
        &outputs_mode,
        &after_input,
        "value",
    )?;
    range_bits(cs.namespace(|| "next_mode_range"), &next_mode, 1)?;

    Ok(ReplayGrammarOutputsV2 {
        mode: next_mode,
        input_count: next_input_count,
        output_count: next_output_count,
    })
}

struct ReplayPayloadOutputsV2 {
    cells: Vec<(usize, AllocatedNum<Scalar>)>,
}

fn replay_parser_stage_selectors<CS: ConstraintSystem<Scalar>>(
    cs: &mut CS,
    stage: &AllocatedNum<Scalar>,
) -> Result<Vec<AllocatedBit>, SynthesisError> {
    let stage_value = stage.get_value().map(scalar_u64);
    let mut selectors = Vec::with_capacity(ReplayParserStageV2::ALL.len());
    for candidate in ReplayParserStageV2::ALL {
        selectors.push(AllocatedBit::alloc(
            cs.namespace(|| format!("replay_parser_stage_selector_{}", candidate as u64)),
            stage_value.map(|value| value == candidate as u64),
        )?);
    }
    enforce_selector_value(
        cs,
        stage,
        &selectors,
        |index| ReplayParserStageV2::ALL[index] as u64,
        "replay_parser_stage",
    );
    Ok(selectors)
}

/// Return whether `TraceChunk::bytes[index]` is one of its constrained visible
/// canonical bytes. The exact one-hot byte-count representation is allocated
/// by the sole chunk decoder, so this is not a second length witness.
fn trace_chunk_byte_visible<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    chunk: &AllocatedTraceChunkV2,
    index: usize,
) -> Result<AllocatedBit, SynthesisError> {
    if index >= TRACE_CANONICAL_CHUNK_BYTES_V2 {
        return Err(SynthesisError::Unsatisfiable(
            "trace chunk visibility index exceeds fixed width".to_owned(),
        ));
    }
    let visible = AllocatedBit::alloc(
        cs.namespace(|| format!("trace_chunk_byte_visible_{index}")),
        chunk
            .byte_count
            .get_value()
            .map(|count| scalar_u64(count) > index as u64),
    )?;
    cs.enforce(
        || format!("trace_chunk_byte_visible_relation_{index}"),
        |lc| {
            let mut relation = lc + visible.get_variable();
            for selector in chunk.byte_count_selectors.iter().skip(index + 1) {
                relation = relation - selector.get_variable();
            }
            relation
        },
        |lc| lc + CS::one(),
        |lc| lc,
    );
    Ok(visible)
}

/// Require a byte to belong to an exact finite alphabet only while `gate` is
/// selected. Inactive `TraceChunk` bytes stay unconstrained by this parser but
/// are independently fixed to zero by the canonical feeder representation.
fn enforce_gated_byte_set<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    gate: &AllocatedBit,
    byte: &AllocatedNum<Scalar>,
    allowed: &[u8],
    label: &str,
) -> Result<(), SynthesisError> {
    if allowed.is_empty() {
        return Err(SynthesisError::Unsatisfiable(
            "replay parser alphabet is empty".to_owned(),
        ));
    }
    let mut matches = Vec::with_capacity(allowed.len());
    for value in allowed {
        let candidate = allocate_constant(
            cs.namespace(|| format!("{label}_candidate_{value}")),
            u64::from(*value),
        )?;
        matches.push(nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| format!("{label}_selector_{value}")),
            byte,
            &candidate,
        )?);
    }
    let accepted = AllocatedBit::alloc(
        cs.namespace(|| format!("{label}_accepted")),
        byte.get_value().map(|byte| {
            allowed
                .iter()
                .any(|value| Scalar::from(u64::from(*value)) == byte)
        }),
    )?;
    cs.enforce(
        || format!("{label}_accepted_relation"),
        |lc| {
            let mut relation = lc + accepted.get_variable();
            for selector in &matches {
                relation = relation - selector.get_variable();
            }
            relation
        },
        |lc| lc + CS::one(),
        |lc| lc,
    );
    cs.enforce(
        || format!("{label}_active_member"),
        |lc| lc + gate.get_variable(),
        |lc| lc + accepted.get_variable() - CS::one(),
        |lc| lc,
    );
    Ok(())
}

/// Materialize lowercase hexadecimal ASCII from one constrained nibble. This
/// derives terminal-ID text from the source header object ID in R1CS rather
/// than accepting a second terminal-ID or decoded object witness.
fn lower_hex_ascii<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    nibble: &AllocatedNum<Scalar>,
    label: &str,
) -> Result<AllocatedNum<Scalar>, SynthesisError> {
    range_bits(cs.namespace(|| format!("{label}_range")), nibble, 4)?;
    let alphabet = b"0123456789abcdef";
    let value = AllocatedNum::alloc(cs.namespace(|| label), || {
        let index = usize::try_from(scalar_u64(
            nibble
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)?,
        ))
        .map_err(|_| SynthesisError::Unsatisfiable("hex nibble index overflow".to_owned()))?;
        alphabet
            .get(index)
            .copied()
            .map(|byte| Scalar::from(u64::from(byte)))
            .ok_or_else(|| SynthesisError::Unsatisfiable("hex nibble exceeds alphabet".to_owned()))
    })?;
    let mut selectors = Vec::with_capacity(alphabet.len());
    for (index, byte) in alphabet.iter().copied().enumerate() {
        let candidate = allocate_constant(
            cs.namespace(|| format!("{label}_nibble_{index}")),
            index as u64,
        )?;
        let selector = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| format!("{label}_nibble_selector_{index}")),
            nibble,
            &candidate,
        )?;
        enforce_gated_constant(
            cs.namespace(|| format!("{label}_ascii_{index}")),
            &selector,
            &value,
            u64::from(byte),
        );
        selectors.push(selector);
    }
    enforce_one_hot(&mut cs, &selectors, &format!("{label}_nibble_one_hot"));
    Ok(value)
}

fn source_object_hex_bytes<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    z: &[AllocatedNum<Scalar>],
) -> Result<Vec<AllocatedNum<Scalar>>, SynthesisError> {
    let mut output = Vec::with_capacity(DIGEST_LIMBS * 4);
    for index in 0..DIGEST_LIMBS * 2 {
        let source_byte =
            &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_HEADER_START_OFFSET + 1 + 8 + index];
        let high =
            AllocatedNum::alloc(cs.namespace(|| format!("object_hex_high_{index}")), || {
                let byte = scalar_u64(
                    source_byte
                        .get_value()
                        .ok_or(SynthesisError::AssignmentMissing)?,
                );
                Ok(Scalar::from(byte >> 4))
            })?;
        let low = AllocatedNum::alloc(cs.namespace(|| format!("object_hex_low_{index}")), || {
            let byte = scalar_u64(
                source_byte
                    .get_value()
                    .ok_or(SynthesisError::AssignmentMissing)?,
            );
            Ok(Scalar::from(byte & 0x0f))
        })?;
        range_bits(
            cs.namespace(|| format!("object_hex_high_range_{index}")),
            &high,
            4,
        )?;
        range_bits(
            cs.namespace(|| format!("object_hex_low_range_{index}")),
            &low,
            4,
        )?;
        cs.enforce(
            || format!("object_hex_nibble_relation_{index}"),
            |lc| {
                lc + source_byte.get_variable()
                    - (Scalar::from(16_u64), high.get_variable())
                    - low.get_variable()
            },
            |lc| lc + CS::one(),
            |lc| lc,
        );
        output.push(lower_hex_ascii(
            cs.namespace(|| format!("object_hex_high_ascii_{index}")),
            &high,
            "value",
        )?);
        output.push(lower_hex_ascii(
            cs.namespace(|| format!("object_hex_low_ascii_{index}")),
            &low,
            "value",
        )?);
    }
    Ok(output)
}

/// Decode `CanonicalFlowItemV2` directly from the meaningful canonical bytes
/// of the one source `TraceChunk` feeder. The state is deliberately only the
/// parser program counter and field remainder: replay semantics cannot be
/// accepted through a source digest, a host decoder, or a second payload tape.
fn synthesize_replay_payload<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    z: &[AllocatedNum<Scalar>],
    opcode_selectors: &[AllocatedBit],
    trace_chunk_selector: &AllocatedBit,
    trace_chunk: &AllocatedTraceChunkV2,
    control: &AllocatedHashControlV2,
) -> Result<ReplayPayloadOutputsV2, SynthesisError> {
    let selector = |opcode: RecursiveTraceOpcodeV2| {
        opcode_selectors
            .get(usize::from(opcode as u8).saturating_sub(1))
            .ok_or_else(|| {
                SynthesisError::Unsatisfiable("replay payload opcode selector missing".to_owned())
            })
    };
    let replay_input = selector(RecursiveTraceOpcodeV2::ReplayInput)?;
    let replay_output = selector(RecursiveTraceOpcodeV2::ReplayOutput)?;
    let replay_source = allocate_bit_or(
        cs.namespace(|| "replay_source_selector"),
        replay_input,
        replay_output,
        "value",
    )?;
    let zero = allocate_constant(cs.namespace(|| "zero"), 0)?;
    let one = allocate_constant(cs.namespace(|| "one"), 1)?;
    let header_width = allocate_constant(
        cs.namespace(|| "canonical_header_width"),
        TRACE_EVENT_HEADER_BYTES_V2 as u64,
    )?;

    for (label, index) in [
        (
            "replay_source_requires_inactive_parser",
            REPLAY_PARSE_ACTIVE_CELL,
        ),
        (
            "replay_source_requires_empty_header",
            REPLAY_PARSE_HEADER_CELL,
        ),
        (
            "replay_source_requires_initial_stage",
            REPLAY_PARSE_STAGE_CELL,
        ),
        (
            "replay_source_requires_empty_field",
            REPLAY_PARSE_REMAINING_CELL,
        ),
    ] {
        enforce_gated_constant(cs.namespace(|| label), &replay_source, &z[index], 0);
    }

    let source_header_opcode = &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_HEADER_START_OFFSET];
    let replay_input_code = allocate_constant(
        cs.namespace(|| "replay_input_opcode"),
        RecursiveTraceOpcodeV2::ReplayInput as u8 as u64,
    )?;
    let replay_output_code = allocate_constant(
        cs.namespace(|| "replay_output_opcode"),
        RecursiveTraceOpcodeV2::ReplayOutput as u8 as u64,
    )?;
    let source_is_input = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "source_is_replay_input"),
        source_header_opcode,
        &replay_input_code,
    )?;
    let source_is_output = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "source_is_replay_output"),
        source_header_opcode,
        &replay_output_code,
    )?;

    let parser_active = allocate_state_bit(
        cs.namespace(|| "parser_active"),
        &z[REPLAY_PARSE_ACTIVE_CELL],
    )?;
    cs.enforce(
        || "active_parser_has_replay_source_kind",
        |lc| lc + parser_active.get_variable(),
        |lc| lc + source_is_input.get_variable() + source_is_output.get_variable() - CS::one(),
        |lc| lc,
    );
    let source_replay_context = allocate_bit_or(
        cs.namespace(|| "source_replay_context"),
        &source_is_input,
        &source_is_output,
        "value",
    )?;
    let source_end = AllocatedBit::and(
        cs.namespace(|| "source_end"),
        control.end_selector(),
        control.source_schema_selector(),
    )?;
    let replay_source_end = AllocatedBit::and(
        cs.namespace(|| "replay_source_end"),
        &source_end,
        &source_replay_context,
    )?;
    for (label, index) in [
        (
            "replay_end_requires_parser_closed",
            REPLAY_PARSE_ACTIVE_CELL,
        ),
        (
            "replay_end_requires_header_consumed",
            REPLAY_PARSE_HEADER_CELL,
        ),
        ("replay_end_requires_initial_stage", REPLAY_PARSE_STAGE_CELL),
        (
            "replay_end_requires_no_field_bytes",
            REPLAY_PARSE_REMAINING_CELL,
        ),
    ] {
        enforce_gated_constant(cs.namespace(|| label), &replay_source_end, &z[index], 0);
    }

    let object_hex = source_object_hex_bytes(cs.namespace(|| "source_object_hex"), z)?;
    if object_hex.len() != DIGEST_LIMBS * 4 {
        return Err(SynthesisError::Unsatisfiable(
            "source object hexadecimal width mismatch".to_owned(),
        ));
    }

    let mut active = z[REPLAY_PARSE_ACTIVE_CELL].clone();
    let mut header_left = z[REPLAY_PARSE_HEADER_CELL].clone();
    let mut stage = z[REPLAY_PARSE_STAGE_CELL].clone();
    let mut remaining = z[REPLAY_PARSE_REMAINING_CELL].clone();
    let hex_alphabet = b"0123456789abcdef";
    let graphic_alphabet = (33_u8..=126_u8).collect::<Vec<_>>();
    for index in 0..TRACE_CANONICAL_CHUNK_BYTES_V2 {
        let visible = trace_chunk_byte_visible(
            cs.namespace(|| format!("byte_{index}_visible")),
            trace_chunk,
            index,
        )?;
        let selected = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_selected")),
            trace_chunk_selector,
            &visible,
        )?;
        let active_bit =
            allocate_state_bit(cs.namespace(|| format!("byte_{index}_active")), &active)?;
        let consuming = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_consuming")),
            &selected,
            &active_bit,
        )?;
        let header_is_zero = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| format!("byte_{index}_header_is_zero")),
            &header_left,
            &zero,
        )?;
        let header_not_zero = allocate_bit_not(
            cs.namespace(|| format!("byte_{index}_header_not_zero")),
            &header_is_zero,
            "value",
        )?;
        let header_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_header_gate")),
            &consuming,
            &header_not_zero,
        )?;
        let payload_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_payload_gate")),
            &consuming,
            &header_is_zero,
        )?;
        let header_after = AllocatedNum::alloc(
            cs.namespace(|| format!("byte_{index}_header_after")),
            || {
                let value = scalar_u64(
                    header_left
                        .get_value()
                        .ok_or(SynthesisError::AssignmentMissing)?,
                );
                Ok(Scalar::from(value.saturating_sub(1)))
            },
        )?;
        cs.enforce(
            || format!("byte_{index}_header_decrement"),
            |lc| lc + header_gate.get_variable(),
            |lc| lc + header_after.get_variable() + CS::one() - header_left.get_variable(),
            |lc| lc,
        );
        header_left = select_bit_num(
            cs.namespace(|| format!("byte_{index}_header_output")),
            &header_gate,
            &header_after,
            &header_left,
            "value",
        )?;

        let stage_selectors = replay_parser_stage_selectors(
            &mut cs.namespace(|| format!("byte_{index}_stage")),
            &stage,
        )?;
        let stage_value = stage.get_value().map(scalar_u64);
        let byte = &trace_chunk.bytes[index];
        let payload_stage =
            AllocatedNum::alloc(cs.namespace(|| format!("byte_{index}_next_stage")), || {
                let value = match stage_value.ok_or(SynthesisError::AssignmentMissing)? {
                    value if value == ReplayParserStageV2::Op as u64 => {
                        ReplayParserStageV2::TxLenLow as u64
                    }
                    value if value == ReplayParserStageV2::TxLenLow as u64 => {
                        ReplayParserStageV2::TxLenHigh as u64
                    }
                    value if value == ReplayParserStageV2::TxLenHigh as u64 => {
                        ReplayParserStageV2::TxData as u64
                    }
                    value if value == ReplayParserStageV2::TxData as u64 => {
                        if scalar_u64(
                            remaining
                                .get_value()
                                .ok_or(SynthesisError::AssignmentMissing)?,
                        ) == 1
                        {
                            ReplayParserStageV2::DefinitionLenLow as u64
                        } else {
                            ReplayParserStageV2::TxData as u64
                        }
                    }
                    value if value == ReplayParserStageV2::DefinitionLenLow as u64 => {
                        ReplayParserStageV2::DefinitionLenHigh as u64
                    }
                    value if value == ReplayParserStageV2::DefinitionLenHigh as u64 => {
                        ReplayParserStageV2::DefinitionData as u64
                    }
                    value if value == ReplayParserStageV2::DefinitionData as u64 => {
                        if scalar_u64(
                            remaining
                                .get_value()
                                .ok_or(SynthesisError::AssignmentMissing)?,
                        ) == 1
                        {
                            ReplayParserStageV2::Serial as u64
                        } else {
                            ReplayParserStageV2::DefinitionData as u64
                        }
                    }
                    value if value == ReplayParserStageV2::Serial as u64 => {
                        if scalar_u64(
                            remaining
                                .get_value()
                                .ok_or(SynthesisError::AssignmentMissing)?,
                        ) == 1
                        {
                            ReplayParserStageV2::TerminalLenLow as u64
                        } else {
                            ReplayParserStageV2::Serial as u64
                        }
                    }
                    value if value == ReplayParserStageV2::TerminalLenLow as u64 => {
                        ReplayParserStageV2::TerminalLenHigh as u64
                    }
                    value if value == ReplayParserStageV2::TerminalLenHigh as u64 => {
                        ReplayParserStageV2::TerminalData as u64
                    }
                    value if value == ReplayParserStageV2::TerminalData as u64 => {
                        if scalar_u64(
                            remaining
                                .get_value()
                                .ok_or(SynthesisError::AssignmentMissing)?,
                        ) == 1
                        {
                            ReplayParserStageV2::Leaf as u64
                        } else {
                            ReplayParserStageV2::TerminalData as u64
                        }
                    }
                    value if value == ReplayParserStageV2::Leaf as u64 => {
                        ReplayParserStageV2::FirstDefinition as u64
                    }
                    value if value == ReplayParserStageV2::FirstDefinition as u64 => {
                        ReplayParserStageV2::FirstSerial as u64
                    }
                    value if value == ReplayParserStageV2::FirstSerial as u64 => {
                        ReplayParserStageV2::FirstObject as u64
                    }
                    value if value == ReplayParserStageV2::FirstObject as u64 => 0,
                    _ => {
                        return Err(SynthesisError::Unsatisfiable(
                            "unknown replay parser stage".to_owned(),
                        ))
                    }
                };
                Ok(Scalar::from(value))
            })?;
        let payload_remaining = AllocatedNum::alloc(
            cs.namespace(|| format!("byte_{index}_next_remaining")),
            || {
                let current = scalar_u64(
                    remaining
                        .get_value()
                        .ok_or(SynthesisError::AssignmentMissing)?,
                );
                let byte = scalar_u64(byte.get_value().ok_or(SynthesisError::AssignmentMissing)?);
                let value = match stage_value.ok_or(SynthesisError::AssignmentMissing)? {
                    value if value == ReplayParserStageV2::TxLenLow as u64 => byte,
                    value if value == ReplayParserStageV2::TxLenHigh as u64 => current
                        .checked_add(byte.checked_mul(256).ok_or_else(|| {
                            SynthesisError::Unsatisfiable(
                                "replay transaction length overflow".to_owned(),
                            )
                        })?)
                        .ok_or_else(|| {
                            SynthesisError::Unsatisfiable(
                                "replay transaction length overflow".to_owned(),
                            )
                        })?,
                    value if value == ReplayParserStageV2::TxData as u64 => {
                        current.saturating_sub(1)
                    }
                    value if value == ReplayParserStageV2::DefinitionLenHigh as u64 => 64,
                    value if value == ReplayParserStageV2::DefinitionData as u64 => {
                        if current == 1 {
                            4
                        } else {
                            current.saturating_sub(1)
                        }
                    }
                    value if value == ReplayParserStageV2::Serial as u64 => {
                        current.saturating_sub(1)
                    }
                    value if value == ReplayParserStageV2::TerminalLenHigh as u64 => 64,
                    value if value == ReplayParserStageV2::TerminalData as u64 => {
                        current.saturating_sub(1)
                    }
                    _ => 0,
                };
                Ok(Scalar::from(value))
            },
        )?;
        let payload_active =
            AllocatedNum::alloc(cs.namespace(|| format!("byte_{index}_next_active")), || {
                let stage = stage_value.ok_or(SynthesisError::AssignmentMissing)?;
                Ok(Scalar::from(u64::from(
                    stage != ReplayParserStageV2::FirstObject as u64,
                )))
            })?;
        range_bits(
            cs.namespace(|| format!("byte_{index}_next_active_range")),
            &payload_active,
            1,
        )?;
        let remaining_is_zero = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| format!("byte_{index}_remaining_is_zero")),
            &remaining,
            &zero,
        )?;
        let remaining_is_one = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| format!("byte_{index}_remaining_is_one")),
            &remaining,
            &one,
        )?;
        for (stage_index, parser_stage) in ReplayParserStageV2::ALL.iter().copied().enumerate() {
            let gate = AllocatedBit::and(
                cs.namespace(|| format!("byte_{index}_gate_{stage_index}")),
                &payload_gate,
                &stage_selectors[stage_index],
            )?;
            match parser_stage {
                ReplayParserStageV2::TxData
                | ReplayParserStageV2::DefinitionData
                | ReplayParserStageV2::Serial
                | ReplayParserStageV2::TerminalData => {
                    let (last_stage, continuing_stage) = match parser_stage {
                        ReplayParserStageV2::TxData => (
                            ReplayParserStageV2::DefinitionLenLow,
                            ReplayParserStageV2::TxData,
                        ),
                        ReplayParserStageV2::DefinitionData => (
                            ReplayParserStageV2::Serial,
                            ReplayParserStageV2::DefinitionData,
                        ),
                        ReplayParserStageV2::Serial => (
                            ReplayParserStageV2::TerminalLenLow,
                            ReplayParserStageV2::Serial,
                        ),
                        ReplayParserStageV2::TerminalData => {
                            (ReplayParserStageV2::Leaf, ReplayParserStageV2::TerminalData)
                        }
                        _ => unreachable!("only dynamic replay parser stages are selected"),
                    };
                    let last_stage_value = allocate_constant(
                        cs.namespace(|| format!("byte_{index}_last_stage_{stage_index}")),
                        last_stage as u64,
                    )?;
                    let continuing_stage_value = allocate_constant(
                        cs.namespace(|| format!("byte_{index}_continuing_stage_{stage_index}")),
                        continuing_stage as u64,
                    )?;
                    let expected_stage = select_bit_num(
                        cs.namespace(|| format!("byte_{index}_stage_relation_{stage_index}")),
                        &remaining_is_one,
                        &last_stage_value,
                        &continuing_stage_value,
                        "value",
                    )?;
                    enforce_gated_equal(
                        cs.namespace(|| format!("byte_{index}_stage_selected_{stage_index}")),
                        &gate,
                        &payload_stage,
                        &expected_stage,
                    );
                }
                _ => {
                    let expected_stage = match parser_stage {
                        ReplayParserStageV2::Op => ReplayParserStageV2::TxLenLow,
                        ReplayParserStageV2::TxLenLow => ReplayParserStageV2::TxLenHigh,
                        ReplayParserStageV2::TxLenHigh => ReplayParserStageV2::TxData,
                        ReplayParserStageV2::DefinitionLenLow => {
                            ReplayParserStageV2::DefinitionLenHigh
                        }
                        ReplayParserStageV2::DefinitionLenHigh => {
                            ReplayParserStageV2::DefinitionData
                        }
                        ReplayParserStageV2::TerminalLenLow => ReplayParserStageV2::TerminalLenHigh,
                        ReplayParserStageV2::TerminalLenHigh => ReplayParserStageV2::TerminalData,
                        ReplayParserStageV2::Leaf => ReplayParserStageV2::FirstDefinition,
                        ReplayParserStageV2::FirstDefinition => ReplayParserStageV2::FirstSerial,
                        ReplayParserStageV2::FirstSerial => ReplayParserStageV2::FirstObject,
                        ReplayParserStageV2::FirstObject => ReplayParserStageV2::Op,
                        ReplayParserStageV2::TxData
                        | ReplayParserStageV2::DefinitionData
                        | ReplayParserStageV2::Serial
                        | ReplayParserStageV2::TerminalData => {
                            unreachable!("dynamic replay parser stage handled above")
                        }
                    };
                    enforce_gated_constant(
                        cs.namespace(|| format!("byte_{index}_stage_relation_{stage_index}")),
                        &gate,
                        &payload_stage,
                        expected_stage as u64,
                    );
                }
            }
            enforce_gated_constant(
                cs.namespace(|| format!("byte_{index}_active_relation_{stage_index}")),
                &gate,
                &payload_active,
                u64::from(parser_stage != ReplayParserStageV2::FirstObject),
            );
            match parser_stage {
                ReplayParserStageV2::Op => {
                    // The source opcode selects the replay set, while the
                    // canonical payload byte independently selects its
                    // storage operation.  In the only canonical codec these
                    // are `Put = 1` and `Delete = 2`; tying one to the other
                    // would reject a valid source transcript.  The bit
                    // equation is the compact exact set constraint
                    // `op_kind = 1 + is_delete`, without a variable-length
                    // alphabet gadget or a host semantic verdict.
                    let is_delete = AllocatedBit::alloc(
                        cs.namespace(|| format!("byte_{index}_op_kind_is_delete")),
                        byte.get_value().map(|value| value == Scalar::from(2_u64)),
                    )?;
                    cs.enforce(
                        || format!("byte_{index}_op_kind_canonical"),
                        |lc| lc + gate.get_variable(),
                        |lc| lc + byte.get_variable() - is_delete.get_variable() - CS::one(),
                        |lc| lc,
                    );
                    enforce_gated_constant(
                        cs.namespace(|| format!("byte_{index}_opcode_remaining")),
                        &gate,
                        &payload_remaining,
                        0,
                    );
                }
                ReplayParserStageV2::TxLenLow => {
                    enforce_gated_equal(
                        cs.namespace(|| format!("byte_{index}_tx_low")),
                        &gate,
                        &payload_remaining,
                        byte,
                    );
                }
                ReplayParserStageV2::TxLenHigh => {
                    enforce_gated_byte_set(
                        cs.namespace(|| format!("byte_{index}_tx_high")),
                        &gate,
                        byte,
                        &[0, 1],
                        "value",
                    )?;
                    cs.enforce(
                        || format!("byte_{index}_tx_length_relation"),
                        |lc| lc + gate.get_variable(),
                        |lc| {
                            lc + payload_remaining.get_variable()
                                - remaining.get_variable()
                                - (Scalar::from(256_u64), byte.get_variable())
                        },
                        |lc| lc,
                    );
                    let high_is_one = nova_snark::gadgets::utils::alloc_num_equals(
                        cs.namespace(|| format!("byte_{index}_tx_high_is_one")),
                        byte,
                        &one,
                    )?;
                    let high_gate = AllocatedBit::and(
                        cs.namespace(|| format!("byte_{index}_tx_high_gate")),
                        &gate,
                        &high_is_one,
                    )?;
                    enforce_gated_constant(
                        cs.namespace(|| format!("byte_{index}_tx_256_low_zero")),
                        &high_gate,
                        &remaining,
                        0,
                    );
                    let next_empty = nova_snark::gadgets::utils::alloc_num_equals(
                        cs.namespace(|| format!("byte_{index}_tx_length_empty")),
                        &payload_remaining,
                        &zero,
                    )?;
                    cs.enforce(
                        || format!("byte_{index}_tx_length_nonzero"),
                        |lc| lc + gate.get_variable(),
                        |lc| lc + next_empty.get_variable(),
                        |lc| lc,
                    );
                }
                ReplayParserStageV2::TxData => {
                    enforce_gated_byte_set(
                        cs.namespace(|| format!("byte_{index}_tx_ascii")),
                        &gate,
                        byte,
                        &graphic_alphabet,
                        "value",
                    )?;
                    cs.enforce(
                        || format!("byte_{index}_tx_nonempty"),
                        |lc| lc + gate.get_variable(),
                        |lc| lc + remaining_is_zero.get_variable(),
                        |lc| lc,
                    );
                    cs.enforce(
                        || format!("byte_{index}_tx_remaining"),
                        |lc| lc + gate.get_variable(),
                        |lc| {
                            lc + payload_remaining.get_variable() + CS::one()
                                - remaining.get_variable()
                        },
                        |lc| lc,
                    );
                }
                ReplayParserStageV2::DefinitionLenLow | ReplayParserStageV2::TerminalLenLow => {
                    enforce_gated_constant(
                        cs.namespace(|| format!("byte_{index}_hex_length_low_{stage_index}")),
                        &gate,
                        byte,
                        64,
                    );
                    enforce_gated_constant(
                        cs.namespace(|| {
                            format!("byte_{index}_hex_length_low_remaining_{stage_index}")
                        }),
                        &gate,
                        &payload_remaining,
                        0,
                    );
                }
                ReplayParserStageV2::DefinitionLenHigh | ReplayParserStageV2::TerminalLenHigh => {
                    enforce_gated_constant(
                        cs.namespace(|| format!("byte_{index}_hex_length_high_{stage_index}")),
                        &gate,
                        byte,
                        0,
                    );
                    enforce_gated_constant(
                        cs.namespace(|| {
                            format!("byte_{index}_hex_length_high_remaining_{stage_index}")
                        }),
                        &gate,
                        &payload_remaining,
                        64,
                    );
                }
                ReplayParserStageV2::DefinitionData | ReplayParserStageV2::TerminalData => {
                    enforce_gated_byte_set(
                        cs.namespace(|| format!("byte_{index}_hex_ascii_{stage_index}")),
                        &gate,
                        byte,
                        hex_alphabet,
                        "value",
                    )?;
                    cs.enforce(
                        || format!("byte_{index}_hex_nonempty_{stage_index}"),
                        |lc| lc + gate.get_variable(),
                        |lc| lc + remaining_is_zero.get_variable(),
                        |lc| lc,
                    );
                    if parser_stage == ReplayParserStageV2::DefinitionData {
                        let serial_width = allocate_constant(
                            cs.namespace(|| format!("byte_{index}_serial_width")),
                            4,
                        )?;
                        let decrement = AllocatedNum::alloc(
                            cs.namespace(|| format!("byte_{index}_definition_decrement")),
                            || {
                                let value = scalar_u64(
                                    remaining
                                        .get_value()
                                        .ok_or(SynthesisError::AssignmentMissing)?,
                                );
                                Ok(Scalar::from(value.saturating_sub(1)))
                            },
                        )?;
                        cs.enforce(
                            || format!("byte_{index}_definition_decrement_relation"),
                            |lc| lc + gate.get_variable(),
                            |lc| {
                                lc + decrement.get_variable() + CS::one() - remaining.get_variable()
                            },
                            |lc| lc,
                        );
                        let after_last = select_bit_num(
                            cs.namespace(|| format!("byte_{index}_definition_remaining_last")),
                            &remaining_is_one,
                            &serial_width,
                            &decrement,
                            "value",
                        )?;
                        enforce_gated_equal(
                            cs.namespace(|| format!("byte_{index}_definition_remaining")),
                            &gate,
                            &payload_remaining,
                            &after_last,
                        );
                    } else {
                        cs.enforce(
                            || format!("byte_{index}_terminal_remaining"),
                            |lc| lc + gate.get_variable(),
                            |lc| {
                                lc + payload_remaining.get_variable() + CS::one()
                                    - remaining.get_variable()
                            },
                            |lc| lc,
                        );
                        let mut positions = Vec::with_capacity(DIGEST_LIMBS * 4);
                        for candidate in 1..=DIGEST_LIMBS * 4 {
                            let value = allocate_constant(
                                cs.namespace(|| {
                                    format!("byte_{index}_terminal_position_{candidate}")
                                }),
                                candidate as u64,
                            )?;
                            positions.push(nova_snark::gadgets::utils::alloc_num_equals(
                                cs.namespace(|| {
                                    format!("byte_{index}_terminal_position_selector_{candidate}")
                                }),
                                &remaining,
                                &value,
                            )?);
                        }
                        cs.enforce(
                            || format!("byte_{index}_terminal_position_complete"),
                            |lc| lc + gate.get_variable(),
                            |lc| {
                                let mut relation = lc - CS::one();
                                for position in &positions {
                                    relation = relation + position.get_variable();
                                }
                                relation
                            },
                            |lc| lc,
                        );
                        for (position, expected) in positions.into_iter().enumerate() {
                            let position_gate = AllocatedBit::and(
                                cs.namespace(|| {
                                    format!("byte_{index}_terminal_position_gate_{position}")
                                }),
                                &gate,
                                &expected,
                            )?;
                            enforce_gated_equal(
                                cs.namespace(|| {
                                    format!("byte_{index}_terminal_matches_object_{position}")
                                }),
                                &position_gate,
                                byte,
                                &object_hex[DIGEST_LIMBS * 4 - position - 1],
                            );
                        }
                    }
                }
                ReplayParserStageV2::Serial => {
                    cs.enforce(
                        || format!("byte_{index}_serial_nonempty"),
                        |lc| lc + gate.get_variable(),
                        |lc| lc + remaining_is_zero.get_variable(),
                        |lc| lc,
                    );
                    cs.enforce(
                        || format!("byte_{index}_serial_remaining"),
                        |lc| lc + gate.get_variable(),
                        |lc| {
                            lc + payload_remaining.get_variable() + CS::one()
                                - remaining.get_variable()
                        },
                        |lc| lc,
                    );
                }
                ReplayParserStageV2::Leaf => {
                    enforce_gated_constant(
                        cs.namespace(|| format!("byte_{index}_terminal_leaf")),
                        &gate,
                        byte,
                        1,
                    );
                    enforce_gated_constant(
                        cs.namespace(|| format!("byte_{index}_leaf_remaining")),
                        &gate,
                        &payload_remaining,
                        0,
                    );
                }
                ReplayParserStageV2::FirstDefinition
                | ReplayParserStageV2::FirstSerial
                | ReplayParserStageV2::FirstObject => {
                    enforce_gated_byte_set(
                        cs.namespace(|| format!("byte_{index}_first_seen_{stage_index}")),
                        &gate,
                        byte,
                        &[0, 1],
                        "value",
                    )?;
                    enforce_gated_constant(
                        cs.namespace(|| format!("byte_{index}_first_seen_remaining_{stage_index}")),
                        &gate,
                        &payload_remaining,
                        0,
                    );
                    let input_gate = AllocatedBit::and(
                        cs.namespace(|| format!("byte_{index}_input_flag_{stage_index}")),
                        &gate,
                        &source_is_input,
                    )?;
                    enforce_gated_constant(
                        cs.namespace(|| format!("byte_{index}_input_flag_zero_{stage_index}")),
                        &input_gate,
                        byte,
                        0,
                    );
                    if parser_stage == ReplayParserStageV2::FirstObject {
                        enforce_gated_constant(
                            cs.namespace(|| format!("byte_{index}_payload_record_complete")),
                            &gate,
                            &trace_chunk.byte_count,
                            (index + 1) as u64,
                        );
                    }
                }
            }
        }
        stage = select_bit_num(
            cs.namespace(|| format!("byte_{index}_stage_output")),
            &payload_gate,
            &payload_stage,
            &stage,
            "value",
        )?;
        remaining = select_bit_num(
            cs.namespace(|| format!("byte_{index}_remaining_output")),
            &payload_gate,
            &payload_remaining,
            &remaining,
            "value",
        )?;
        active = select_bit_num(
            cs.namespace(|| format!("byte_{index}_active_output")),
            &payload_gate,
            &payload_active,
            &active,
            "value",
        )?;
    }

    let source_active = select_bit_num(
        cs.namespace(|| "source_active_output"),
        &replay_source,
        &one,
        &z[REPLAY_PARSE_ACTIVE_CELL],
        "value",
    )?;
    let source_header = select_bit_num(
        cs.namespace(|| "source_header_output"),
        &replay_source,
        &header_width,
        &z[REPLAY_PARSE_HEADER_CELL],
        "value",
    )?;
    let source_stage = select_bit_num(
        cs.namespace(|| "source_stage_output"),
        &replay_source,
        &zero,
        &z[REPLAY_PARSE_STAGE_CELL],
        "value",
    )?;
    let source_remaining = select_bit_num(
        cs.namespace(|| "source_remaining_output"),
        &replay_source,
        &zero,
        &z[REPLAY_PARSE_REMAINING_CELL],
        "value",
    )?;
    let output_active = select_bit_num(
        cs.namespace(|| "trace_active_output"),
        trace_chunk_selector,
        &active,
        &source_active,
        "value",
    )?;
    let output_header = select_bit_num(
        cs.namespace(|| "trace_header_output"),
        trace_chunk_selector,
        &header_left,
        &source_header,
        "value",
    )?;
    let output_stage = select_bit_num(
        cs.namespace(|| "trace_stage_output"),
        trace_chunk_selector,
        &stage,
        &source_stage,
        "value",
    )?;
    let output_remaining = select_bit_num(
        cs.namespace(|| "trace_remaining_output"),
        trace_chunk_selector,
        &remaining,
        &source_remaining,
        "value",
    )?;
    range_bits(cs.namespace(|| "output_active_range"), &output_active, 1)?;
    range_bits(cs.namespace(|| "output_header_range"), &output_header, 6)?;
    range_bits(cs.namespace(|| "output_stage_range"), &output_stage, 4)?;
    range_bits(
        cs.namespace(|| "output_remaining_range"),
        &output_remaining,
        9,
    )?;
    Ok(ReplayPayloadOutputsV2 {
        cells: vec![
            (REPLAY_PARSE_ACTIVE_CELL, output_active),
            (REPLAY_PARSE_HEADER_CELL, output_header),
            (REPLAY_PARSE_STAGE_CELL, output_stage),
            (REPLAY_PARSE_REMAINING_CELL, output_remaining),
        ],
    })
}

/// Streaming outputs for the exact canonical `UniquenessPrecommit` payload.
///
/// The count limbs and five digest values stay in the one running state only
/// so later uniqueness/challenge/net relations can consume these authenticated
/// bytes. They are never a host-decoded payload substitute.
struct UniquenessPrecommitPayloadOutputsV2 {
    cells: Vec<(usize, AllocatedNum<Scalar>)>,
}

/// Decode the frozen 169-byte `UniquenessPrecommit` grammar directly from the
/// meaningful bytes of the canonical source `TraceChunk` feeder.  The source
/// record header is consumed in-stream, exactly as the replay parser does;
/// consequently a count or digest limb has no witness path outside canonical
/// source-record bytes already bound to the FIPS schedule.
fn synthesize_uniqueness_precommit_payload<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    z: &[AllocatedNum<Scalar>],
    opcode_selectors: &[AllocatedBit],
    trace_chunk_selector: &AllocatedBit,
    trace_chunk: &AllocatedTraceChunkV2,
    control: &AllocatedHashControlV2,
) -> Result<UniquenessPrecommitPayloadOutputsV2, SynthesisError> {
    let selector = |opcode: RecursiveTraceOpcodeV2| {
        opcode_selectors
            .get(usize::from(opcode as u8).saturating_sub(1))
            .ok_or_else(|| {
                SynthesisError::Unsatisfiable(
                    "uniqueness precommit opcode selector missing".to_owned(),
                )
            })
    };
    let precommit_source = selector(RecursiveTraceOpcodeV2::UniquenessPrecommit)?;
    let zero = allocate_constant(cs.namespace(|| "zero"), 0)?;
    let one = allocate_constant(cs.namespace(|| "one"), 1)?;
    let header_width = allocate_constant(
        cs.namespace(|| "canonical_header_width"),
        TRACE_EVENT_HEADER_BYTES_V2 as u64,
    )?;
    let payload_width = allocate_constant(
        cs.namespace(|| "canonical_precommit_payload_width"),
        UNIQUENESS_PRECOMMIT_BYTES_V2 as u64,
    )?;

    let field_indices =
        (PRECOMMIT_SPENT_COUNT_LIMB_START..PRECOMMIT_DIGEST_LIMB_END).collect::<Vec<_>>();
    let mut field_high_positions = vec![2_usize, 4, 6, 8];
    field_high_positions
        .extend((PRECOMMIT_DIGEST_BYTES_START + 1..UNIQUENESS_PRECOMMIT_BYTES_V2).step_by(2));
    if field_indices.len() != field_high_positions.len()
        || PRECOMMIT_DIGEST_LIMB_COUNT != DIGEST_LIMBS * 5
    {
        return Err(SynthesisError::Unsatisfiable(
            "canonical uniqueness precommit field layout mismatch".to_owned(),
        ));
    }

    for (label, index) in [
        (
            "precommit_source_requires_inactive_parser",
            PRECOMMIT_PARSE_ACTIVE_CELL,
        ),
        (
            "precommit_source_requires_empty_header",
            PRECOMMIT_PARSE_HEADER_CELL,
        ),
        (
            "precommit_source_requires_zero_offset",
            PRECOMMIT_PARSE_OFFSET_CELL,
        ),
        (
            "precommit_source_requires_empty_low_byte",
            PRECOMMIT_PARSE_LOW_BYTE_CELL,
        ),
    ] {
        enforce_gated_constant(cs.namespace(|| label), &precommit_source, &z[index], 0);
    }
    for index in &field_indices {
        enforce_gated_constant(
            cs.namespace(|| format!("precommit_source_requires_zero_field_{index}")),
            &precommit_source,
            &z[*index],
            0,
        );
    }

    let source_header_opcode = &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_HEADER_START_OFFSET];
    let precommit_opcode = allocate_constant(
        cs.namespace(|| "precommit_opcode"),
        RecursiveTraceOpcodeV2::UniquenessPrecommit as u8 as u64,
    )?;
    let source_is_precommit = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "source_is_uniqueness_precommit"),
        source_header_opcode,
        &precommit_opcode,
    )?;
    let parser_active = allocate_state_bit(
        cs.namespace(|| "parser_active"),
        &z[PRECOMMIT_PARSE_ACTIVE_CELL],
    )?;
    cs.enforce(
        || "active_parser_has_precommit_source_kind",
        |lc| lc + parser_active.get_variable(),
        |lc| lc + source_is_precommit.get_variable() - CS::one(),
        |lc| lc,
    );

    let source_end = AllocatedBit::and(
        cs.namespace(|| "source_end"),
        control.end_selector(),
        control.source_schema_selector(),
    )?;
    let precommit_source_end = AllocatedBit::and(
        cs.namespace(|| "precommit_source_end"),
        &source_end,
        &source_is_precommit,
    )?;
    for (label, index, expected) in [
        (
            "precommit_end_requires_parser_closed",
            PRECOMMIT_PARSE_ACTIVE_CELL,
            0,
        ),
        (
            "precommit_end_requires_header_consumed",
            PRECOMMIT_PARSE_HEADER_CELL,
            0,
        ),
        (
            "precommit_end_requires_exact_payload_width",
            PRECOMMIT_PARSE_OFFSET_CELL,
            UNIQUENESS_PRECOMMIT_BYTES_V2 as u64,
        ),
        (
            "precommit_end_requires_empty_low_byte",
            PRECOMMIT_PARSE_LOW_BYTE_CELL,
            0,
        ),
    ] {
        enforce_gated_constant(
            cs.namespace(|| label),
            &precommit_source_end,
            &z[index],
            expected,
        );
    }
    for (label, index) in [
        (
            "precommit_spent_count_high_zero",
            PRECOMMIT_SPENT_COUNT_LIMB_START + 1,
        ),
        (
            "precommit_output_count_high_zero",
            PRECOMMIT_OUTPUT_COUNT_LIMB_START + 1,
        ),
    ] {
        enforce_gated_constant(cs.namespace(|| label), &precommit_source_end, &z[index], 0);
    }
    enforce_gated_equal(
        cs.namespace(|| "precommit_spent_count_matches_replay_input_count"),
        &precommit_source_end,
        &z[PRECOMMIT_SPENT_COUNT_LIMB_START],
        &z[REPLAY_INPUT_COUNT_CELL],
    );
    enforce_gated_equal(
        cs.namespace(|| "precommit_output_count_matches_replay_output_count"),
        &precommit_source_end,
        &z[PRECOMMIT_OUTPUT_COUNT_LIMB_START],
        &z[REPLAY_OUTPUT_COUNT_CELL],
    );

    let mut active = z[PRECOMMIT_PARSE_ACTIVE_CELL].clone();
    let mut header_left = z[PRECOMMIT_PARSE_HEADER_CELL].clone();
    let mut offset = z[PRECOMMIT_PARSE_OFFSET_CELL].clone();
    let mut low_byte = z[PRECOMMIT_PARSE_LOW_BYTE_CELL].clone();
    let mut fields = field_indices
        .iter()
        .map(|index| z[*index].clone())
        .collect::<Vec<_>>();
    let low_positions = std::iter::once(1_usize)
        .chain(std::iter::once(3))
        .chain(std::iter::once(5))
        .chain(std::iter::once(7))
        .chain((PRECOMMIT_DIGEST_BYTES_START..UNIQUENESS_PRECOMMIT_BYTES_V2).step_by(2))
        .collect::<Vec<_>>();
    let high_positions = field_high_positions.clone();
    if low_positions.len() != high_positions.len() {
        return Err(SynthesisError::Unsatisfiable(
            "canonical uniqueness precommit byte-pair layout mismatch".to_owned(),
        ));
    }

    for index in 0..TRACE_CANONICAL_CHUNK_BYTES_V2 {
        let visible = trace_chunk_byte_visible(
            cs.namespace(|| format!("byte_{index}_visible")),
            trace_chunk,
            index,
        )?;
        let selected = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_selected")),
            trace_chunk_selector,
            &visible,
        )?;
        let active_bit =
            allocate_state_bit(cs.namespace(|| format!("byte_{index}_active")), &active)?;
        let consuming = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_consuming")),
            &selected,
            &active_bit,
        )?;
        let header_is_zero = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| format!("byte_{index}_header_is_zero")),
            &header_left,
            &zero,
        )?;
        let header_not_zero = allocate_bit_not(
            cs.namespace(|| format!("byte_{index}_header_not_zero")),
            &header_is_zero,
            "value",
        )?;
        let header_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_header_gate")),
            &consuming,
            &header_not_zero,
        )?;
        let payload_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_payload_gate")),
            &consuming,
            &header_is_zero,
        )?;
        let header_after = AllocatedNum::alloc(
            cs.namespace(|| format!("byte_{index}_header_after")),
            || {
                let value = scalar_u64(
                    header_left
                        .get_value()
                        .ok_or(SynthesisError::AssignmentMissing)?,
                );
                Ok(Scalar::from(value.saturating_sub(1)))
            },
        )?;
        cs.enforce(
            || format!("byte_{index}_header_decrement"),
            |lc| lc + header_gate.get_variable(),
            |lc| lc + header_after.get_variable() + CS::one() - header_left.get_variable(),
            |lc| lc,
        );
        header_left = select_bit_num(
            cs.namespace(|| format!("byte_{index}_header_output")),
            &header_gate,
            &header_after,
            &header_left,
            "value",
        )?;

        let mut position_selectors = Vec::with_capacity(UNIQUENESS_PRECOMMIT_BYTES_V2);
        for position in 0..UNIQUENESS_PRECOMMIT_BYTES_V2 {
            let value = allocate_constant(
                cs.namespace(|| format!("byte_{index}_payload_position_{position}")),
                position as u64,
            )?;
            position_selectors.push(nova_snark::gadgets::utils::alloc_num_equals(
                cs.namespace(|| format!("byte_{index}_payload_position_selector_{position}")),
                &offset,
                &value,
            )?);
        }
        cs.enforce(
            || format!("byte_{index}_payload_position_complete"),
            |lc| lc + payload_gate.get_variable(),
            |lc| {
                let mut relation = lc - CS::one();
                for selector in &position_selectors {
                    relation = relation + selector.get_variable();
                }
                relation
            },
            |lc| lc,
        );

        let byte = &trace_chunk.bytes[index];
        let offset_value = offset.get_value().map(scalar_u64);
        let offset_after = AllocatedNum::alloc(
            cs.namespace(|| format!("byte_{index}_offset_after")),
            || {
                let value = scalar_u64(
                    offset
                        .get_value()
                        .ok_or(SynthesisError::AssignmentMissing)?,
                );
                let next = value.checked_add(1).ok_or_else(|| {
                    SynthesisError::Unsatisfiable(
                        "uniqueness precommit payload offset overflow".to_owned(),
                    )
                })?;
                Ok(Scalar::from(next))
            },
        )?;
        cs.enforce(
            || format!("byte_{index}_payload_offset_increment"),
            |lc| lc + payload_gate.get_variable(),
            |lc| lc + offset_after.get_variable() - offset.get_variable() - CS::one(),
            |lc| lc,
        );
        offset = select_bit_num(
            cs.namespace(|| format!("byte_{index}_offset_output")),
            &payload_gate,
            &offset_after,
            &offset,
            "value",
        )?;

        let low_position = AllocatedBit::alloc(
            cs.namespace(|| format!("byte_{index}_is_low_position")),
            offset_value.map(|value| low_positions.contains(&(value as usize))),
        )?;
        cs.enforce(
            || format!("byte_{index}_low_position_relation"),
            |lc| {
                let mut relation = lc + low_position.get_variable();
                for position in &low_positions {
                    relation = relation - position_selectors[*position].get_variable();
                }
                relation
            },
            |lc| lc + CS::one(),
            |lc| lc,
        );
        let high_position = AllocatedBit::alloc(
            cs.namespace(|| format!("byte_{index}_is_high_position")),
            offset_value.map(|value| high_positions.contains(&(value as usize))),
        )?;
        cs.enforce(
            || format!("byte_{index}_high_position_relation"),
            |lc| {
                let mut relation = lc + high_position.get_variable();
                for position in &high_positions {
                    relation = relation - position_selectors[*position].get_variable();
                }
                relation
            },
            |lc| lc + CS::one(),
            |lc| lc,
        );
        let low_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_low_gate")),
            &payload_gate,
            &low_position,
        )?;
        let high_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_high_gate")),
            &payload_gate,
            &high_position,
        )?;
        let low_after = select_bit_num(
            cs.namespace(|| format!("byte_{index}_low_after_low_position")),
            &low_gate,
            byte,
            &low_byte,
            "value",
        )?;
        low_byte = select_bit_num(
            cs.namespace(|| format!("byte_{index}_low_after_high_position")),
            &high_gate,
            &zero,
            &low_after,
            "value",
        )?;

        let parsed_limb =
            AllocatedNum::alloc(cs.namespace(|| format!("byte_{index}_parsed_limb")), || {
                let low = scalar_u64(
                    low_after
                        .get_value()
                        .ok_or(SynthesisError::AssignmentMissing)?,
                );
                let high = scalar_u64(byte.get_value().ok_or(SynthesisError::AssignmentMissing)?);
                let value = low
                    .checked_add(high.checked_mul(256).ok_or_else(|| {
                        SynthesisError::Unsatisfiable(
                            "uniqueness precommit limb overflow".to_owned(),
                        )
                    })?)
                    .ok_or_else(|| {
                        SynthesisError::Unsatisfiable(
                            "uniqueness precommit limb overflow".to_owned(),
                        )
                    })?;
                Ok(Scalar::from(value))
            })?;
        cs.enforce(
            || format!("byte_{index}_parsed_limb_relation"),
            |lc| lc + high_gate.get_variable(),
            |lc| {
                lc + parsed_limb.get_variable()
                    - low_after.get_variable()
                    - (Scalar::from(256_u64), byte.get_variable())
            },
            |lc| lc,
        );
        for ((field, position), state_index) in fields
            .iter_mut()
            .zip(&field_high_positions)
            .zip(&field_indices)
        {
            let field_gate = AllocatedBit::and(
                cs.namespace(|| format!("byte_{index}_field_gate_{state_index}")),
                &payload_gate,
                &position_selectors[*position],
            )?;
            let prior_field = field.clone();
            *field = select_bit_num(
                cs.namespace(|| format!("byte_{index}_field_output_{state_index}")),
                &field_gate,
                &parsed_limb,
                &prior_field,
                "value",
            )?;
        }

        let version_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_version_gate")),
            &payload_gate,
            &position_selectors[0],
        )?;
        enforce_gated_constant(
            cs.namespace(|| format!("byte_{index}_version")),
            &version_gate,
            byte,
            u64::from(UNIQUENESS_PRECOMMIT_VERSION_V2),
        );
        let final_payload_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_final_payload_gate")),
            &payload_gate,
            &position_selectors[UNIQUENESS_PRECOMMIT_BYTES_V2 - 1],
        )?;
        let final_chunk_width = allocate_constant(
            cs.namespace(|| format!("byte_{index}_final_chunk_width")),
            (index + 1) as u64,
        )?;
        enforce_gated_equal(
            cs.namespace(|| format!("byte_{index}_payload_has_no_trailing_bytes")),
            &final_payload_gate,
            &trace_chunk.byte_count,
            &final_chunk_width,
        );
        active = select_bit_num(
            cs.namespace(|| format!("byte_{index}_active_output")),
            &final_payload_gate,
            &zero,
            &active,
            "value",
        )?;
    }

    let source_active = select_bit_num(
        cs.namespace(|| "source_active_output"),
        &precommit_source,
        &one,
        &z[PRECOMMIT_PARSE_ACTIVE_CELL],
        "value",
    )?;
    let source_header = select_bit_num(
        cs.namespace(|| "source_header_output"),
        &precommit_source,
        &header_width,
        &z[PRECOMMIT_PARSE_HEADER_CELL],
        "value",
    )?;
    let source_offset = select_bit_num(
        cs.namespace(|| "source_offset_output"),
        &precommit_source,
        &zero,
        &z[PRECOMMIT_PARSE_OFFSET_CELL],
        "value",
    )?;
    let source_low_byte = select_bit_num(
        cs.namespace(|| "source_low_byte_output"),
        &precommit_source,
        &zero,
        &z[PRECOMMIT_PARSE_LOW_BYTE_CELL],
        "value",
    )?;
    let output_active = select_bit_num(
        cs.namespace(|| "trace_active_output"),
        trace_chunk_selector,
        &active,
        &source_active,
        "value",
    )?;
    let output_header = select_bit_num(
        cs.namespace(|| "trace_header_output"),
        trace_chunk_selector,
        &header_left,
        &source_header,
        "value",
    )?;
    let output_offset = select_bit_num(
        cs.namespace(|| "trace_offset_output"),
        trace_chunk_selector,
        &offset,
        &source_offset,
        "value",
    )?;
    let output_low_byte = select_bit_num(
        cs.namespace(|| "trace_low_byte_output"),
        trace_chunk_selector,
        &low_byte,
        &source_low_byte,
        "value",
    )?;
    let mut cells = vec![
        (PRECOMMIT_PARSE_ACTIVE_CELL, output_active),
        (PRECOMMIT_PARSE_HEADER_CELL, output_header),
        (PRECOMMIT_PARSE_OFFSET_CELL, output_offset),
        (PRECOMMIT_PARSE_LOW_BYTE_CELL, output_low_byte),
    ];
    for (state_index, field) in field_indices.iter().zip(fields) {
        let source_field = select_bit_num(
            cs.namespace(|| format!("source_field_output_{state_index}")),
            &precommit_source,
            &zero,
            &z[*state_index],
            "value",
        )?;
        let output_field = select_bit_num(
            cs.namespace(|| format!("trace_field_output_{state_index}")),
            trace_chunk_selector,
            &field,
            &source_field,
            "value",
        )?;
        range_bits(
            cs.namespace(|| format!("output_field_range_{state_index}")),
            &output_field,
            16,
        )?;
        cells.push((*state_index, output_field));
    }
    for (label, value, bits) in [
        ("output_active_range", &cells[0].1, 1),
        ("output_header_range", &cells[1].1, 6),
        ("output_offset_range", &cells[2].1, 8),
        ("output_low_byte_range", &cells[3].1, 8),
    ] {
        range_bits(cs.namespace(|| label), value, bits)?;
    }
    // Keep the canonical width as an allocated constant in the relation so a
    // future codec edit cannot silently alter only the end-of-record check.
    enforce_gated_equal(
        cs.namespace(|| "precommit_payload_width_is_canonical"),
        &precommit_source_end,
        &z[PRECOMMIT_PARSE_OFFSET_CELL],
        &payload_width,
    );
    Ok(UniquenessPrecommitPayloadOutputsV2 { cells })
}

/// The challenge record is decoded from the same canonical source-byte feeder
/// as its precommit.  In particular, the first 32 payload bytes must equal the
/// already materialized `precommit_digest` limbs; no host decoder, witness
/// digest, or parallel byte stream can substitute for that equality.  The
/// final 32 bytes are retained only as challenge material for the later
/// SHA-derived challenge-map relation.
struct UniquenessChallengePayloadOutputsV2 {
    cells: Vec<(usize, AllocatedNum<Scalar>)>,
}

fn synthesize_uniqueness_challenge_payload<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    z: &[AllocatedNum<Scalar>],
    opcode_selectors: &[AllocatedBit],
    trace_chunk_selector: &AllocatedBit,
    trace_chunk: &AllocatedTraceChunkV2,
    control: &AllocatedHashControlV2,
) -> Result<UniquenessChallengePayloadOutputsV2, SynthesisError> {
    let selector = |opcode: RecursiveTraceOpcodeV2| {
        opcode_selectors
            .get(usize::from(opcode as u8).saturating_sub(1))
            .ok_or_else(|| {
                SynthesisError::Unsatisfiable(
                    "uniqueness challenge opcode selector missing".to_owned(),
                )
            })
    };
    let challenge_source = selector(RecursiveTraceOpcodeV2::UniquenessChallenge)?;
    let zero = allocate_constant(cs.namespace(|| "zero"), 0)?;
    let one = allocate_constant(cs.namespace(|| "one"), 1)?;
    let header_width = allocate_constant(
        cs.namespace(|| "canonical_header_width"),
        TRACE_EVENT_HEADER_BYTES_V2 as u64,
    )?;
    let payload_width = allocate_constant(
        cs.namespace(|| "canonical_challenge_payload_width"),
        UNIQUENESS_CHALLENGE_BYTES_V2 as u64,
    )?;

    let field_indices =
        (CHALLENGE_DIGEST_LIMB_START..CHALLENGE_DIGEST_LIMB_END).collect::<Vec<_>>();
    let field_high_positions = (CHALLENGE_DIGEST_BYTES_START + 1..UNIQUENESS_CHALLENGE_BYTES_V2)
        .step_by(2)
        .collect::<Vec<_>>();
    let committed_precommit_high_positions = (2..CHALLENGE_DIGEST_BYTES_START)
        .step_by(2)
        .collect::<Vec<_>>();
    let committed_precommit_indices = (PRECOMMIT_COMMITMENT_DIGEST_LIMB_START
        ..PRECOMMIT_COMMITMENT_DIGEST_LIMB_START + DIGEST_LIMBS)
        .collect::<Vec<_>>();
    if field_indices.len() != field_high_positions.len()
        || field_indices.len() != DIGEST_LIMBS
        || committed_precommit_high_positions.len() != DIGEST_LIMBS
        || committed_precommit_indices.len() != DIGEST_LIMBS
        || CHALLENGE_DIGEST_LIMB_COUNT != DIGEST_LIMBS
    {
        return Err(SynthesisError::Unsatisfiable(
            "canonical uniqueness challenge field layout mismatch".to_owned(),
        ));
    }

    for (label, index) in [
        (
            "challenge_source_requires_inactive_parser",
            CHALLENGE_PARSE_ACTIVE_CELL,
        ),
        (
            "challenge_source_requires_empty_header",
            CHALLENGE_PARSE_HEADER_CELL,
        ),
        (
            "challenge_source_requires_zero_offset",
            CHALLENGE_PARSE_OFFSET_CELL,
        ),
        (
            "challenge_source_requires_empty_low_byte",
            CHALLENGE_PARSE_LOW_BYTE_CELL,
        ),
    ] {
        enforce_gated_constant(cs.namespace(|| label), &challenge_source, &z[index], 0);
    }
    for index in &field_indices {
        enforce_gated_constant(
            cs.namespace(|| format!("challenge_source_requires_zero_field_{index}")),
            &challenge_source,
            &z[*index],
            0,
        );
    }

    let source_header_opcode = &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_HEADER_START_OFFSET];
    let challenge_opcode = allocate_constant(
        cs.namespace(|| "challenge_opcode"),
        RecursiveTraceOpcodeV2::UniquenessChallenge as u8 as u64,
    )?;
    let source_is_challenge = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "source_is_uniqueness_challenge"),
        source_header_opcode,
        &challenge_opcode,
    )?;
    let parser_active = allocate_state_bit(
        cs.namespace(|| "parser_active"),
        &z[CHALLENGE_PARSE_ACTIVE_CELL],
    )?;
    cs.enforce(
        || "active_parser_has_challenge_source_kind",
        |lc| lc + parser_active.get_variable(),
        |lc| lc + source_is_challenge.get_variable() - CS::one(),
        |lc| lc,
    );

    let source_end = AllocatedBit::and(
        cs.namespace(|| "source_end"),
        control.end_selector(),
        control.source_schema_selector(),
    )?;
    let challenge_source_end = AllocatedBit::and(
        cs.namespace(|| "challenge_source_end"),
        &source_end,
        &source_is_challenge,
    )?;
    for (label, index, expected) in [
        (
            "challenge_end_requires_parser_closed",
            CHALLENGE_PARSE_ACTIVE_CELL,
            0,
        ),
        (
            "challenge_end_requires_header_consumed",
            CHALLENGE_PARSE_HEADER_CELL,
            0,
        ),
        (
            "challenge_end_requires_exact_payload_width",
            CHALLENGE_PARSE_OFFSET_CELL,
            UNIQUENESS_CHALLENGE_BYTES_V2 as u64,
        ),
        (
            "challenge_end_requires_empty_low_byte",
            CHALLENGE_PARSE_LOW_BYTE_CELL,
            0,
        ),
    ] {
        enforce_gated_constant(
            cs.namespace(|| label),
            &challenge_source_end,
            &z[index],
            expected,
        );
    }

    let mut active = z[CHALLENGE_PARSE_ACTIVE_CELL].clone();
    let mut header_left = z[CHALLENGE_PARSE_HEADER_CELL].clone();
    let mut offset = z[CHALLENGE_PARSE_OFFSET_CELL].clone();
    let mut low_byte = z[CHALLENGE_PARSE_LOW_BYTE_CELL].clone();
    let mut fields = field_indices
        .iter()
        .map(|index| z[*index].clone())
        .collect::<Vec<_>>();
    let low_positions = (1..UNIQUENESS_CHALLENGE_BYTES_V2)
        .step_by(2)
        .collect::<Vec<_>>();
    let high_positions = (2..UNIQUENESS_CHALLENGE_BYTES_V2)
        .step_by(2)
        .collect::<Vec<_>>();
    if low_positions.len() != high_positions.len() {
        return Err(SynthesisError::Unsatisfiable(
            "canonical uniqueness challenge byte-pair layout mismatch".to_owned(),
        ));
    }

    for index in 0..TRACE_CANONICAL_CHUNK_BYTES_V2 {
        let visible = trace_chunk_byte_visible(
            cs.namespace(|| format!("byte_{index}_visible")),
            trace_chunk,
            index,
        )?;
        let selected = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_selected")),
            trace_chunk_selector,
            &visible,
        )?;
        let active_bit =
            allocate_state_bit(cs.namespace(|| format!("byte_{index}_active")), &active)?;
        let consuming = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_consuming")),
            &selected,
            &active_bit,
        )?;
        let header_is_zero = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| format!("byte_{index}_header_is_zero")),
            &header_left,
            &zero,
        )?;
        let header_not_zero = allocate_bit_not(
            cs.namespace(|| format!("byte_{index}_header_not_zero")),
            &header_is_zero,
            "value",
        )?;
        let header_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_header_gate")),
            &consuming,
            &header_not_zero,
        )?;
        let payload_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_payload_gate")),
            &consuming,
            &header_is_zero,
        )?;
        let header_after = AllocatedNum::alloc(
            cs.namespace(|| format!("byte_{index}_header_after")),
            || {
                let value = scalar_u64(
                    header_left
                        .get_value()
                        .ok_or(SynthesisError::AssignmentMissing)?,
                );
                Ok(Scalar::from(value.saturating_sub(1)))
            },
        )?;
        cs.enforce(
            || format!("byte_{index}_header_decrement"),
            |lc| lc + header_gate.get_variable(),
            |lc| lc + header_after.get_variable() + CS::one() - header_left.get_variable(),
            |lc| lc,
        );
        header_left = select_bit_num(
            cs.namespace(|| format!("byte_{index}_header_output")),
            &header_gate,
            &header_after,
            &header_left,
            "value",
        )?;

        let mut position_selectors = Vec::with_capacity(UNIQUENESS_CHALLENGE_BYTES_V2);
        for position in 0..UNIQUENESS_CHALLENGE_BYTES_V2 {
            let value = allocate_constant(
                cs.namespace(|| format!("byte_{index}_payload_position_{position}")),
                position as u64,
            )?;
            position_selectors.push(nova_snark::gadgets::utils::alloc_num_equals(
                cs.namespace(|| format!("byte_{index}_payload_position_selector_{position}")),
                &offset,
                &value,
            )?);
        }
        cs.enforce(
            || format!("byte_{index}_payload_position_complete"),
            |lc| lc + payload_gate.get_variable(),
            |lc| {
                let mut relation = lc - CS::one();
                for selector in &position_selectors {
                    relation = relation + selector.get_variable();
                }
                relation
            },
            |lc| lc,
        );

        let byte = &trace_chunk.bytes[index];
        let offset_value = offset.get_value().map(scalar_u64);
        let offset_after = AllocatedNum::alloc(
            cs.namespace(|| format!("byte_{index}_offset_after")),
            || {
                let value = scalar_u64(
                    offset
                        .get_value()
                        .ok_or(SynthesisError::AssignmentMissing)?,
                );
                let next = value.checked_add(1).ok_or_else(|| {
                    SynthesisError::Unsatisfiable(
                        "uniqueness challenge payload offset overflow".to_owned(),
                    )
                })?;
                Ok(Scalar::from(next))
            },
        )?;
        cs.enforce(
            || format!("byte_{index}_payload_offset_increment"),
            |lc| lc + payload_gate.get_variable(),
            |lc| lc + offset_after.get_variable() - offset.get_variable() - CS::one(),
            |lc| lc,
        );
        offset = select_bit_num(
            cs.namespace(|| format!("byte_{index}_offset_output")),
            &payload_gate,
            &offset_after,
            &offset,
            "value",
        )?;

        let low_position = AllocatedBit::alloc(
            cs.namespace(|| format!("byte_{index}_is_low_position")),
            offset_value.map(|value| low_positions.contains(&(value as usize))),
        )?;
        cs.enforce(
            || format!("byte_{index}_low_position_relation"),
            |lc| {
                let mut relation = lc + low_position.get_variable();
                for position in &low_positions {
                    relation = relation - position_selectors[*position].get_variable();
                }
                relation
            },
            |lc| lc + CS::one(),
            |lc| lc,
        );
        let high_position = AllocatedBit::alloc(
            cs.namespace(|| format!("byte_{index}_is_high_position")),
            offset_value.map(|value| high_positions.contains(&(value as usize))),
        )?;
        cs.enforce(
            || format!("byte_{index}_high_position_relation"),
            |lc| {
                let mut relation = lc + high_position.get_variable();
                for position in &high_positions {
                    relation = relation - position_selectors[*position].get_variable();
                }
                relation
            },
            |lc| lc + CS::one(),
            |lc| lc,
        );
        let low_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_low_gate")),
            &payload_gate,
            &low_position,
        )?;
        let high_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_high_gate")),
            &payload_gate,
            &high_position,
        )?;
        let low_after = select_bit_num(
            cs.namespace(|| format!("byte_{index}_low_after_low_position")),
            &low_gate,
            byte,
            &low_byte,
            "value",
        )?;
        low_byte = select_bit_num(
            cs.namespace(|| format!("byte_{index}_low_after_high_position")),
            &high_gate,
            &zero,
            &low_after,
            "value",
        )?;

        let parsed_limb =
            AllocatedNum::alloc(cs.namespace(|| format!("byte_{index}_parsed_limb")), || {
                let low = scalar_u64(
                    low_after
                        .get_value()
                        .ok_or(SynthesisError::AssignmentMissing)?,
                );
                let high = scalar_u64(byte.get_value().ok_or(SynthesisError::AssignmentMissing)?);
                let value = low
                    .checked_add(high.checked_mul(256).ok_or_else(|| {
                        SynthesisError::Unsatisfiable(
                            "uniqueness challenge limb overflow".to_owned(),
                        )
                    })?)
                    .ok_or_else(|| {
                        SynthesisError::Unsatisfiable(
                            "uniqueness challenge limb overflow".to_owned(),
                        )
                    })?;
                Ok(Scalar::from(value))
            })?;
        cs.enforce(
            || format!("byte_{index}_parsed_limb_relation"),
            |lc| lc + high_gate.get_variable(),
            |lc| {
                lc + parsed_limb.get_variable()
                    - low_after.get_variable()
                    - (Scalar::from(256_u64), byte.get_variable())
            },
            |lc| lc,
        );
        for (limb, (position, state_index)) in committed_precommit_high_positions
            .iter()
            .zip(&committed_precommit_indices)
            .enumerate()
        {
            let committed_precommit_gate = AllocatedBit::and(
                cs.namespace(|| format!("byte_{index}_committed_precommit_gate_{limb}")),
                &payload_gate,
                &position_selectors[*position],
            )?;
            cs.enforce(
                || format!("byte_{index}_committed_precommit_limb_{limb}_matches"),
                |lc| lc + committed_precommit_gate.get_variable(),
                |lc| lc + parsed_limb.get_variable() - z[*state_index].get_variable(),
                |lc| lc,
            );
        }
        for ((field, position), state_index) in fields
            .iter_mut()
            .zip(&field_high_positions)
            .zip(&field_indices)
        {
            let field_gate = AllocatedBit::and(
                cs.namespace(|| format!("byte_{index}_field_gate_{state_index}")),
                &payload_gate,
                &position_selectors[*position],
            )?;
            let prior_field = field.clone();
            *field = select_bit_num(
                cs.namespace(|| format!("byte_{index}_field_output_{state_index}")),
                &field_gate,
                &parsed_limb,
                &prior_field,
                "value",
            )?;
        }

        let version_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_version_gate")),
            &payload_gate,
            &position_selectors[0],
        )?;
        enforce_gated_constant(
            cs.namespace(|| format!("byte_{index}_version")),
            &version_gate,
            byte,
            u64::from(UNIQUENESS_PRECOMMIT_VERSION_V2),
        );
        let final_payload_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_final_payload_gate")),
            &payload_gate,
            &position_selectors[UNIQUENESS_CHALLENGE_BYTES_V2 - 1],
        )?;
        let final_chunk_width = allocate_constant(
            cs.namespace(|| format!("byte_{index}_final_chunk_width")),
            (index + 1) as u64,
        )?;
        enforce_gated_equal(
            cs.namespace(|| format!("byte_{index}_payload_has_no_trailing_bytes")),
            &final_payload_gate,
            &trace_chunk.byte_count,
            &final_chunk_width,
        );
        active = select_bit_num(
            cs.namespace(|| format!("byte_{index}_active_output")),
            &final_payload_gate,
            &zero,
            &active,
            "value",
        )?;
    }

    let source_active = select_bit_num(
        cs.namespace(|| "source_active_output"),
        &challenge_source,
        &one,
        &z[CHALLENGE_PARSE_ACTIVE_CELL],
        "value",
    )?;
    let source_header = select_bit_num(
        cs.namespace(|| "source_header_output"),
        &challenge_source,
        &header_width,
        &z[CHALLENGE_PARSE_HEADER_CELL],
        "value",
    )?;
    let source_offset = select_bit_num(
        cs.namespace(|| "source_offset_output"),
        &challenge_source,
        &zero,
        &z[CHALLENGE_PARSE_OFFSET_CELL],
        "value",
    )?;
    let source_low_byte = select_bit_num(
        cs.namespace(|| "source_low_byte_output"),
        &challenge_source,
        &zero,
        &z[CHALLENGE_PARSE_LOW_BYTE_CELL],
        "value",
    )?;
    let output_active = select_bit_num(
        cs.namespace(|| "trace_active_output"),
        trace_chunk_selector,
        &active,
        &source_active,
        "value",
    )?;
    let output_header = select_bit_num(
        cs.namespace(|| "trace_header_output"),
        trace_chunk_selector,
        &header_left,
        &source_header,
        "value",
    )?;
    let output_offset = select_bit_num(
        cs.namespace(|| "trace_offset_output"),
        trace_chunk_selector,
        &offset,
        &source_offset,
        "value",
    )?;
    let output_low_byte = select_bit_num(
        cs.namespace(|| "trace_low_byte_output"),
        trace_chunk_selector,
        &low_byte,
        &source_low_byte,
        "value",
    )?;
    let mut cells = vec![
        (CHALLENGE_PARSE_ACTIVE_CELL, output_active),
        (CHALLENGE_PARSE_HEADER_CELL, output_header),
        (CHALLENGE_PARSE_OFFSET_CELL, output_offset),
        (CHALLENGE_PARSE_LOW_BYTE_CELL, output_low_byte),
    ];
    for (state_index, field) in field_indices.iter().zip(fields) {
        let source_field = select_bit_num(
            cs.namespace(|| format!("source_field_output_{state_index}")),
            &challenge_source,
            &zero,
            &z[*state_index],
            "value",
        )?;
        let output_field = select_bit_num(
            cs.namespace(|| format!("trace_field_output_{state_index}")),
            trace_chunk_selector,
            &field,
            &source_field,
            "value",
        )?;
        range_bits(
            cs.namespace(|| format!("output_field_range_{state_index}")),
            &output_field,
            16,
        )?;
        cells.push((*state_index, output_field));
    }
    for (label, value, bits) in [
        ("output_active_range", &cells[0].1, 1),
        ("output_header_range", &cells[1].1, 6),
        ("output_offset_range", &cells[2].1, 8),
        ("output_low_byte_range", &cells[3].1, 8),
    ] {
        range_bits(cs.namespace(|| label), value, bits)?;
    }
    enforce_gated_equal(
        cs.namespace(|| "challenge_payload_width_is_canonical"),
        &challenge_source_end,
        &z[CHALLENGE_PARSE_OFFSET_CELL],
        &payload_width,
    );
    Ok(UniquenessChallengePayloadOutputsV2 { cells })
}

/// Streaming outputs for the exact canonical `NetMerge` payload.
///
/// This materializes the record digest from the same source-byte feeder used
/// by the source/global FIPS contexts.  It deliberately does not claim to
/// prove the later semantic SHA preimage relation.
struct NetMergePayloadOutputsV2 {
    cells: Vec<(usize, AllocatedNum<Scalar>)>,
}

/// Decode the frozen 33-byte `NetMerge` grammar from meaningful canonical
/// `TraceChunk` bytes.  The authoritative semantic encoder remains the only
/// native codec owner; this relation only constrains the matching source bytes
/// and their fixed little-endian digest representation in the running state.
fn synthesize_net_merge_payload<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    z: &[AllocatedNum<Scalar>],
    opcode_selectors: &[AllocatedBit],
    trace_chunk_selector: &AllocatedBit,
    trace_chunk: &AllocatedTraceChunkV2,
    control: &AllocatedHashControlV2,
) -> Result<NetMergePayloadOutputsV2, SynthesisError> {
    let selector = |opcode: RecursiveTraceOpcodeV2| {
        opcode_selectors
            .get(usize::from(opcode as u8).saturating_sub(1))
            .ok_or_else(|| {
                SynthesisError::Unsatisfiable("net merge opcode selector missing".to_owned())
            })
    };
    let net_source = selector(RecursiveTraceOpcodeV2::NetMerge)?;
    let zero = allocate_constant(cs.namespace(|| "zero"), 0)?;
    let one = allocate_constant(cs.namespace(|| "one"), 1)?;
    let header_width = allocate_constant(
        cs.namespace(|| "canonical_header_width"),
        TRACE_EVENT_HEADER_BYTES_V2 as u64,
    )?;
    let payload_width = allocate_constant(
        cs.namespace(|| "canonical_net_merge_payload_width"),
        NET_MERGE_BYTES_V2 as u64,
    )?;
    let field_indices = (NET_DIGEST_LIMB_START..NET_DIGEST_LIMB_END).collect::<Vec<_>>();
    let field_high_positions = (NET_DIGEST_BYTES_START + 1..NET_MERGE_BYTES_V2)
        .step_by(2)
        .collect::<Vec<_>>();
    let low_positions = (NET_DIGEST_BYTES_START..NET_MERGE_BYTES_V2)
        .step_by(2)
        .collect::<Vec<_>>();
    if field_indices.len() != field_high_positions.len()
        || field_indices.len() != low_positions.len()
        || NET_DIGEST_LIMB_COUNT != DIGEST_LIMBS
    {
        return Err(SynthesisError::Unsatisfiable(
            "canonical net merge field layout mismatch".to_owned(),
        ));
    }

    for (label, index) in [
        ("net_source_requires_inactive_parser", NET_PARSE_ACTIVE_CELL),
        ("net_source_requires_empty_header", NET_PARSE_HEADER_CELL),
        ("net_source_requires_zero_offset", NET_PARSE_OFFSET_CELL),
        (
            "net_source_requires_empty_low_byte",
            NET_PARSE_LOW_BYTE_CELL,
        ),
    ] {
        enforce_gated_constant(cs.namespace(|| label), &net_source, &z[index], 0);
    }
    for index in &field_indices {
        enforce_gated_constant(
            cs.namespace(|| format!("net_source_requires_zero_field_{index}")),
            &net_source,
            &z[*index],
            0,
        );
    }

    let source_header_opcode = &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_HEADER_START_OFFSET];
    let net_opcode = allocate_constant(
        cs.namespace(|| "net_opcode"),
        RecursiveTraceOpcodeV2::NetMerge as u8 as u64,
    )?;
    let source_is_net = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "source_is_net_merge"),
        source_header_opcode,
        &net_opcode,
    )?;
    let parser_active =
        allocate_state_bit(cs.namespace(|| "parser_active"), &z[NET_PARSE_ACTIVE_CELL])?;
    cs.enforce(
        || "active_parser_has_net_source_kind",
        |lc| lc + parser_active.get_variable(),
        |lc| lc + source_is_net.get_variable() - CS::one(),
        |lc| lc,
    );

    let source_end = AllocatedBit::and(
        cs.namespace(|| "source_end"),
        control.end_selector(),
        control.source_schema_selector(),
    )?;
    let net_source_end = AllocatedBit::and(
        cs.namespace(|| "net_source_end"),
        &source_end,
        &source_is_net,
    )?;
    for (label, index, expected) in [
        ("net_end_requires_parser_closed", NET_PARSE_ACTIVE_CELL, 0),
        ("net_end_requires_header_consumed", NET_PARSE_HEADER_CELL, 0),
        (
            "net_end_requires_exact_payload_width",
            NET_PARSE_OFFSET_CELL,
            NET_MERGE_BYTES_V2 as u64,
        ),
        (
            "net_end_requires_empty_low_byte",
            NET_PARSE_LOW_BYTE_CELL,
            0,
        ),
    ] {
        enforce_gated_constant(cs.namespace(|| label), &net_source_end, &z[index], expected);
    }

    let mut active = z[NET_PARSE_ACTIVE_CELL].clone();
    let mut header_left = z[NET_PARSE_HEADER_CELL].clone();
    let mut offset = z[NET_PARSE_OFFSET_CELL].clone();
    let mut low_byte = z[NET_PARSE_LOW_BYTE_CELL].clone();
    let mut fields = field_indices
        .iter()
        .map(|index| z[*index].clone())
        .collect::<Vec<_>>();

    for index in 0..TRACE_CANONICAL_CHUNK_BYTES_V2 {
        let visible = trace_chunk_byte_visible(
            cs.namespace(|| format!("byte_{index}_visible")),
            trace_chunk,
            index,
        )?;
        let selected = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_selected")),
            trace_chunk_selector,
            &visible,
        )?;
        let active_bit =
            allocate_state_bit(cs.namespace(|| format!("byte_{index}_active")), &active)?;
        let consuming = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_consuming")),
            &selected,
            &active_bit,
        )?;
        let header_is_zero = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| format!("byte_{index}_header_is_zero")),
            &header_left,
            &zero,
        )?;
        let header_not_zero = allocate_bit_not(
            cs.namespace(|| format!("byte_{index}_header_not_zero")),
            &header_is_zero,
            "value",
        )?;
        let header_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_header_gate")),
            &consuming,
            &header_not_zero,
        )?;
        let payload_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_payload_gate")),
            &consuming,
            &header_is_zero,
        )?;
        let header_after = AllocatedNum::alloc(
            cs.namespace(|| format!("byte_{index}_header_after")),
            || {
                let value = scalar_u64(
                    header_left
                        .get_value()
                        .ok_or(SynthesisError::AssignmentMissing)?,
                );
                Ok(Scalar::from(value.saturating_sub(1)))
            },
        )?;
        cs.enforce(
            || format!("byte_{index}_header_decrement"),
            |lc| lc + header_gate.get_variable(),
            |lc| lc + header_after.get_variable() + CS::one() - header_left.get_variable(),
            |lc| lc,
        );
        header_left = select_bit_num(
            cs.namespace(|| format!("byte_{index}_header_output")),
            &header_gate,
            &header_after,
            &header_left,
            "value",
        )?;

        let mut position_selectors = Vec::with_capacity(NET_MERGE_BYTES_V2);
        for position in 0..NET_MERGE_BYTES_V2 {
            let value = allocate_constant(
                cs.namespace(|| format!("byte_{index}_payload_position_{position}")),
                position as u64,
            )?;
            position_selectors.push(nova_snark::gadgets::utils::alloc_num_equals(
                cs.namespace(|| format!("byte_{index}_payload_position_selector_{position}")),
                &offset,
                &value,
            )?);
        }
        cs.enforce(
            || format!("byte_{index}_payload_position_complete"),
            |lc| lc + payload_gate.get_variable(),
            |lc| {
                let mut relation = lc - CS::one();
                for selector in &position_selectors {
                    relation = relation + selector.get_variable();
                }
                relation
            },
            |lc| lc,
        );

        let byte = &trace_chunk.bytes[index];
        let offset_value = offset.get_value().map(scalar_u64);
        let offset_after = AllocatedNum::alloc(
            cs.namespace(|| format!("byte_{index}_offset_after")),
            || {
                let value = scalar_u64(
                    offset
                        .get_value()
                        .ok_or(SynthesisError::AssignmentMissing)?,
                );
                let next = value.checked_add(1).ok_or_else(|| {
                    SynthesisError::Unsatisfiable("net merge payload offset overflow".to_owned())
                })?;
                Ok(Scalar::from(next))
            },
        )?;
        cs.enforce(
            || format!("byte_{index}_payload_offset_increment"),
            |lc| lc + payload_gate.get_variable(),
            |lc| lc + offset_after.get_variable() - offset.get_variable() - CS::one(),
            |lc| lc,
        );
        offset = select_bit_num(
            cs.namespace(|| format!("byte_{index}_offset_output")),
            &payload_gate,
            &offset_after,
            &offset,
            "value",
        )?;

        let low_position = AllocatedBit::alloc(
            cs.namespace(|| format!("byte_{index}_is_low_position")),
            offset_value.map(|value| low_positions.contains(&(value as usize))),
        )?;
        cs.enforce(
            || format!("byte_{index}_low_position_relation"),
            |lc| {
                let mut relation = lc + low_position.get_variable();
                for position in &low_positions {
                    relation = relation - position_selectors[*position].get_variable();
                }
                relation
            },
            |lc| lc + CS::one(),
            |lc| lc,
        );
        let high_position = AllocatedBit::alloc(
            cs.namespace(|| format!("byte_{index}_is_high_position")),
            offset_value.map(|value| field_high_positions.contains(&(value as usize))),
        )?;
        cs.enforce(
            || format!("byte_{index}_high_position_relation"),
            |lc| {
                let mut relation = lc + high_position.get_variable();
                for position in &field_high_positions {
                    relation = relation - position_selectors[*position].get_variable();
                }
                relation
            },
            |lc| lc + CS::one(),
            |lc| lc,
        );
        let low_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_low_gate")),
            &payload_gate,
            &low_position,
        )?;
        let high_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_high_gate")),
            &payload_gate,
            &high_position,
        )?;
        let low_after = select_bit_num(
            cs.namespace(|| format!("byte_{index}_low_after_low_position")),
            &low_gate,
            byte,
            &low_byte,
            "value",
        )?;
        low_byte = select_bit_num(
            cs.namespace(|| format!("byte_{index}_low_after_high_position")),
            &high_gate,
            &zero,
            &low_after,
            "value",
        )?;
        let parsed_limb =
            AllocatedNum::alloc(cs.namespace(|| format!("byte_{index}_parsed_limb")), || {
                let low = scalar_u64(
                    low_after
                        .get_value()
                        .ok_or(SynthesisError::AssignmentMissing)?,
                );
                let high = scalar_u64(byte.get_value().ok_or(SynthesisError::AssignmentMissing)?);
                let value = low
                    .checked_add(high.checked_mul(256).ok_or_else(|| {
                        SynthesisError::Unsatisfiable("net merge limb overflow".to_owned())
                    })?)
                    .ok_or_else(|| {
                        SynthesisError::Unsatisfiable("net merge limb overflow".to_owned())
                    })?;
                Ok(Scalar::from(value))
            })?;
        cs.enforce(
            || format!("byte_{index}_parsed_limb_relation"),
            |lc| lc + high_gate.get_variable(),
            |lc| {
                lc + parsed_limb.get_variable()
                    - low_after.get_variable()
                    - (Scalar::from(256_u64), byte.get_variable())
            },
            |lc| lc,
        );
        for ((field, position), state_index) in fields
            .iter_mut()
            .zip(&field_high_positions)
            .zip(&field_indices)
        {
            let field_gate = AllocatedBit::and(
                cs.namespace(|| format!("byte_{index}_field_gate_{state_index}")),
                &payload_gate,
                &position_selectors[*position],
            )?;
            let prior_field = field.clone();
            *field = select_bit_num(
                cs.namespace(|| format!("byte_{index}_field_output_{state_index}")),
                &field_gate,
                &parsed_limb,
                &prior_field,
                "value",
            )?;
        }

        let version_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_version_gate")),
            &payload_gate,
            &position_selectors[0],
        )?;
        enforce_gated_constant(
            cs.namespace(|| format!("byte_{index}_version")),
            &version_gate,
            byte,
            u64::from(UNIQUENESS_PRECOMMIT_VERSION_V2),
        );
        let final_payload_gate = AllocatedBit::and(
            cs.namespace(|| format!("byte_{index}_final_payload_gate")),
            &payload_gate,
            &position_selectors[NET_MERGE_BYTES_V2 - 1],
        )?;
        let final_chunk_width = allocate_constant(
            cs.namespace(|| format!("byte_{index}_final_chunk_width")),
            (index + 1) as u64,
        )?;
        enforce_gated_equal(
            cs.namespace(|| format!("byte_{index}_payload_has_no_trailing_bytes")),
            &final_payload_gate,
            &trace_chunk.byte_count,
            &final_chunk_width,
        );
        active = select_bit_num(
            cs.namespace(|| format!("byte_{index}_active_output")),
            &final_payload_gate,
            &zero,
            &active,
            "value",
        )?;
    }

    let source_active = select_bit_num(
        cs.namespace(|| "source_active_output"),
        &net_source,
        &one,
        &z[NET_PARSE_ACTIVE_CELL],
        "value",
    )?;
    let source_header = select_bit_num(
        cs.namespace(|| "source_header_output"),
        &net_source,
        &header_width,
        &z[NET_PARSE_HEADER_CELL],
        "value",
    )?;
    let source_offset = select_bit_num(
        cs.namespace(|| "source_offset_output"),
        &net_source,
        &zero,
        &z[NET_PARSE_OFFSET_CELL],
        "value",
    )?;
    let source_low_byte = select_bit_num(
        cs.namespace(|| "source_low_byte_output"),
        &net_source,
        &zero,
        &z[NET_PARSE_LOW_BYTE_CELL],
        "value",
    )?;
    let output_active = select_bit_num(
        cs.namespace(|| "trace_active_output"),
        trace_chunk_selector,
        &active,
        &source_active,
        "value",
    )?;
    let output_header = select_bit_num(
        cs.namespace(|| "trace_header_output"),
        trace_chunk_selector,
        &header_left,
        &source_header,
        "value",
    )?;
    let output_offset = select_bit_num(
        cs.namespace(|| "trace_offset_output"),
        trace_chunk_selector,
        &offset,
        &source_offset,
        "value",
    )?;
    let output_low_byte = select_bit_num(
        cs.namespace(|| "trace_low_byte_output"),
        trace_chunk_selector,
        &low_byte,
        &source_low_byte,
        "value",
    )?;
    let mut cells = vec![
        (NET_PARSE_ACTIVE_CELL, output_active),
        (NET_PARSE_HEADER_CELL, output_header),
        (NET_PARSE_OFFSET_CELL, output_offset),
        (NET_PARSE_LOW_BYTE_CELL, output_low_byte),
    ];
    for (state_index, field) in field_indices.iter().zip(fields) {
        let source_field = select_bit_num(
            cs.namespace(|| format!("source_field_output_{state_index}")),
            &net_source,
            &zero,
            &z[*state_index],
            "value",
        )?;
        let output_field = select_bit_num(
            cs.namespace(|| format!("trace_field_output_{state_index}")),
            trace_chunk_selector,
            &field,
            &source_field,
            "value",
        )?;
        range_bits(
            cs.namespace(|| format!("output_field_range_{state_index}")),
            &output_field,
            16,
        )?;
        cells.push((*state_index, output_field));
    }
    for (label, value, bits) in [
        ("output_active_range", &cells[0].1, 1),
        ("output_header_range", &cells[1].1, 6),
        ("output_offset_range", &cells[2].1, 6),
        ("output_low_byte_range", &cells[3].1, 8),
    ] {
        range_bits(cs.namespace(|| label), value, bits)?;
    }
    enforce_gated_equal(
        cs.namespace(|| "net_payload_width_is_canonical"),
        &net_source_end,
        &z[NET_PARSE_OFFSET_CELL],
        &payload_width,
    );
    Ok(NetMergePayloadOutputsV2 { cells })
}

struct ShaCompressionOutputsV2 {
    block_next_chaining: Vec<AllocatedNum<Scalar>>,
    block_bytes: Vec<AllocatedNum<Scalar>>,
}

/// Allocate exactly one FIPS SHA-256 compression gadget for every Nova step.
/// The opcode selector only chooses how its result updates the persistent SHA
/// cells; it never changes the allocated gadget geometry.
fn synthesize_sha_compression_lane<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    sha_block_selector: &AllocatedBit,
    z: &[AllocatedNum<Scalar>],
    control: &AllocatedHashControlV2,
    witness: &ShaCompressionLaneWitnessV2,
) -> Result<ShaCompressionOutputsV2, SynthesisError> {
    let lane_ordinal = allocate_constant(cs.namespace(|| "ordinal"), witness.ordinal)?;
    enforce_inactive_zero(
        cs.namespace(|| "inactive_ordinal_zero"),
        sha_block_selector,
        &lane_ordinal,
    );
    let source_block_index = &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET];
    let trace_block_index = &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET];
    let selected_block_index = select_hash_schema_value(
        cs.namespace(|| "selected_context_block_index"),
        &control.schema_selectors,
        [source_block_index, trace_block_index],
    )?;
    enforce_gated_equal(
        cs.namespace(|| "active_ordinal_matches_selected_context"),
        sha_block_selector,
        &selected_block_index,
        &lane_ordinal,
    );
    let mut block_bits = Vec::with_capacity(64 * 8);
    let mut block_bytes = Vec::with_capacity(64);
    for (index, (state_index, byte)) in (SHA_BLOCK_START..SHA_BLOCK_END)
        .zip(witness.block)
        .enumerate()
    {
        let byte = allocate_constant(
            cs.namespace(|| format!("block_byte_{index}")),
            u64::from(byte),
        )?;
        enforce_inactive_zero(
            cs.namespace(|| format!("inactive_block_byte_{index}")),
            sha_block_selector,
            &byte,
        );
        let _ = state_index;
        block_bits.extend(allocated_num_bits_be(
            cs.namespace(|| format!("block_byte_bits_{index}")),
            &byte,
            8,
        )?);
        block_bytes.push(byte);
    }

    let mut chaining_before = Vec::with_capacity(8);
    let mut expected_after = Vec::with_capacity(8);
    for (index, ((state_index, before), after)) in (SHA_CHAINING_START..SHA_CHAINING_END)
        .zip(witness.chaining_before)
        .zip(witness.chaining_after)
        .enumerate()
    {
        let before = allocate_constant(
            cs.namespace(|| format!("chaining_before_{index}")),
            u64::from(before),
        )?;
        enforce_inactive_zero(
            cs.namespace(|| format!("inactive_chaining_before_{index}")),
            sha_block_selector,
            &before,
        );
        let source_chain =
            &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CHAINING_START_OFFSET + index];
        let trace_chain =
            &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_CHAINING_START_OFFSET + index];
        let selected_chain = select_hash_schema_value(
            cs.namespace(|| format!("selected_context_chaining_{index}")),
            &control.schema_selectors,
            [source_chain, trace_chain],
        )?;
        enforce_gated_equal(
            cs.namespace(|| format!("active_chaining_before_matches_context_{index}")),
            sha_block_selector,
            &selected_chain,
            &before,
        );
        let _ = state_index;
        chaining_before.push(allocated_num_to_uint32(
            cs.namespace(|| format!("chaining_before_bits_{index}")),
            &before,
        )?);

        let after = allocate_constant(
            cs.namespace(|| format!("chaining_after_{index}")),
            u64::from(after),
        )?;
        enforce_inactive_zero(
            cs.namespace(|| format!("inactive_chaining_after_{index}")),
            sha_block_selector,
            &after,
        );
        expected_after.push(after);
    }

    let compressed = sha256_compression_function(
        cs.namespace(|| "fips_compression"),
        &block_bits,
        &chaining_before,
    )?;
    if compressed.len() != expected_after.len() {
        return Err(SynthesisError::Unsatisfiable(
            "SHA-256 compression output arity mismatch".to_owned(),
        ));
    }

    for (index, (computed, expected)) in compressed.iter().zip(&expected_after).enumerate() {
        enforce_gated_lc_equal(
            cs.namespace(|| format!("active_chaining_after_matches_fips_{index}")),
            sha_block_selector,
            &uint32_lc::<CS>(computed),
            expected,
        );
    }

    Ok(ShaCompressionOutputsV2 {
        block_next_chaining: expected_after,
        block_bytes,
    })
}

struct HashScheduleOutputsV2 {
    source_trace_ordinal: AllocatedNum<Scalar>,
    source_trace_byte_count: AllocatedNum<Scalar>,
    cells: Vec<(usize, AllocatedNum<Scalar>)>,
}

/// Materialize the prefix the sole `CheckpointSha256BlockStreamV2` owner
/// would have absorbed before the first canonical source-record byte.  Only
/// the record length is dynamic, and it is decomposed directly from the
/// constrained source context rather than supplied as an alternate byte view.
fn source_static_frame_bytes<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    canonical_record_bytes: &AllocatedNum<Scalar>,
) -> Result<Vec<AllocatedNum<Scalar>>, SynthesisError> {
    let role_prefix = CheckpointSha256BlockStreamV2::framed_role_prefix(CheckpointShaRole::Trace);
    let label_len = u64::try_from(SOURCE_RECORD_HASH_LABEL_V2.len()).map_err(|_| {
        SynthesisError::Unsatisfiable("source SHA label length overflow".to_owned())
    })?;
    let mut frame = Vec::with_capacity(
        role_prefix
            .len()
            .checked_add(8)
            .and_then(|value| value.checked_add(SOURCE_RECORD_HASH_LABEL_V2.len()))
            .and_then(|value| value.checked_add(8))
            .ok_or_else(|| {
                SynthesisError::Unsatisfiable("source static SHA frame overflow".to_owned())
            })?,
    );
    for (index, byte) in role_prefix.into_iter().enumerate() {
        frame.push(allocate_constant(
            cs.namespace(|| format!("role_prefix_byte_{index}")),
            u64::from(byte),
        )?);
    }
    for (index, byte) in label_len.to_le_bytes().into_iter().enumerate() {
        frame.push(allocate_constant(
            cs.namespace(|| format!("label_length_byte_{index}")),
            u64::from(byte),
        )?);
    }
    for (index, byte) in SOURCE_RECORD_HASH_LABEL_V2.iter().copied().enumerate() {
        frame.push(allocate_constant(
            cs.namespace(|| format!("source_label_byte_{index}")),
            u64::from(byte),
        )?);
    }
    frame.extend(decompose_le_bytes(
        cs.namespace(|| "declared_source_record_length"),
        canonical_record_bytes,
        8,
        "declared_source_record_length",
    )?);
    Ok(frame)
}

/// Materialize only the role/DST bytes already owned by the canonical trace
/// stream. Per-record length prefixes and record bytes are deliberately added
/// later by the same source/chunk relation; this helper never acts as a second
/// whole-trace encoder.
fn trace_role_prefix_bytes<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
) -> Result<Vec<AllocatedNum<Scalar>>, SynthesisError> {
    CheckpointSha256BlockStreamV2::framed_role_prefix(CheckpointShaRole::Trace)
        .into_iter()
        .enumerate()
        .map(|(index, byte)| {
            allocate_constant(
                cs.namespace(|| format!("trace_role_prefix_byte_{index}")),
                u64::from(byte),
            )
        })
        .collect()
}

/// Allocate the Boolean complement without turning a selector into a native
/// branch.  Every byte-context update uses this relation for its explicit
/// inactive and terminal cases.
fn allocate_bit_not<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    bit: &AllocatedBit,
    label: &str,
) -> Result<AllocatedBit, SynthesisError> {
    let inverse = AllocatedBit::alloc(cs.namespace(|| label), bit.get_value().map(|value| !value))?;
    cs.enforce(
        || format!("{label}_complement"),
        |lc| lc + bit.get_variable() + inverse.get_variable() - CS::one(),
        |lc| lc + CS::one(),
        |lc| lc,
    );
    Ok(inverse)
}

/// Allocate Boolean OR with an explicit product relation.  Nova's Boolean
/// gadget intentionally exposes no `or`; spelling it out keeps the state
/// transition auditable and fixed-shape.
fn allocate_bit_or<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    left: &AllocatedBit,
    right: &AllocatedBit,
    label: &str,
) -> Result<AllocatedBit, SynthesisError> {
    let both = AllocatedBit::and(cs.namespace(|| format!("{label}_and")), left, right)?;
    let output = AllocatedBit::alloc(
        cs.namespace(|| label),
        match (left.get_value(), right.get_value()) {
            (Some(left), Some(right)) => Some(left || right),
            _ => None,
        },
    )?;
    cs.enforce(
        || format!("{label}_relation"),
        |lc| {
            lc + output.get_variable() - left.get_variable() - right.get_variable()
                + both.get_variable()
        },
        |lc| lc + CS::one(),
        |lc| lc,
    );
    Ok(output)
}

/// Lift a constrained Boolean into one field element for a state-cell
/// successor.  The equality is R1CS-enforced rather than host-derived.
fn bit_as_num<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    bit: &AllocatedBit,
    label: &str,
) -> Result<AllocatedNum<Scalar>, SynthesisError> {
    let value = AllocatedNum::alloc(cs.namespace(|| label), || {
        bit.get_value()
            .map(|value| Scalar::from(u64::from(value)))
            .ok_or(SynthesisError::AssignmentMissing)
    })?;
    cs.enforce(
        || format!("{label}_matches_bit"),
        |lc| lc + value.get_variable() - bit.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc,
    );
    Ok(value)
}

/// Select a state successor from a Boolean gate.  Both branches remain in the
/// fixed circuit shape and are constrained even when this row is inactive.
fn select_bit_num<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    selector: &AllocatedBit,
    selected: &AllocatedNum<Scalar>,
    otherwise: &AllocatedNum<Scalar>,
    label: &str,
) -> Result<AllocatedNum<Scalar>, SynthesisError> {
    let inverse = allocate_bit_not(cs.namespace(|| format!("{label}_not")), selector, "value")?;
    let value = AllocatedNum::alloc(cs.namespace(|| label), || {
        if selector.get_value() == Some(true) {
            selected
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)
        } else {
            otherwise
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)
        }
    })?;
    enforce_gated_equal(
        cs.namespace(|| format!("{label}_selected")),
        selector,
        &value,
        selected,
    );
    enforce_gated_equal(
        cs.namespace(|| format!("{label}_otherwise")),
        &inverse,
        &value,
        otherwise,
    );
    Ok(value)
}

/// Return a queue successor after appending exactly `visible_bytes` from the
/// fixed-width feeder.  Bytes beyond that count are already constrained to
/// zero by `TraceChunkWitnessV2`, so this remains one canonical byte view.
fn append_context_bytes<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    z: &[AllocatedNum<Scalar>],
    start: usize,
    active: &AllocatedBit,
    bytes: &[AllocatedNum<Scalar>],
    visible_bytes: &AllocatedNum<Scalar>,
    label: &str,
) -> Result<Vec<(usize, AllocatedNum<Scalar>)>, SynthesisError> {
    if bytes.len() > BYTE_CONTEXT_BUFFER_BYTES {
        return Err(SynthesisError::Unsatisfiable(
            "byte-context append width exceeds fixed queue".to_owned(),
        ));
    }
    constrain_bounded_u8(
        cs.namespace(|| format!("{label}_visible_bound")),
        visible_bytes,
        bytes.len(),
    )?;
    let buffer_len = &z[start + BYTE_CONTEXT_BUFFER_LEN_OFFSET];
    // The native expander emits every immediately available compression block
    // before another feeder record.  An active append therefore starts from a
    // strict <64-byte tail; allocating a full queue-width mux for every idle
    // row would make the fixed circuit quadratic in a payload buffer that is
    // never semantically live there.
    let mut selectors = Vec::with_capacity(TRACE_CANONICAL_CHUNK_BYTES_V2);
    let mut active_tail_relation = nova_snark::frontend::LinearCombination::zero() - CS::one();
    for length in 0..TRACE_CANONICAL_CHUNK_BYTES_V2 {
        let length_value = allocate_constant(
            cs.namespace(|| format!("{label}_length_value_{length}")),
            u64::try_from(length).map_err(|_| {
                SynthesisError::Unsatisfiable("byte-context tail length overflow".to_owned())
            })?,
        )?;
        let is_length = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| format!("{label}_length_selector_{length}")),
            buffer_len,
            &length_value,
        )?;
        active_tail_relation = active_tail_relation + is_length.get_variable();
        selectors.push(is_length);
    }
    cs.enforce(
        || format!("{label}_active_tail_is_short"),
        |lc| lc + active.get_variable(),
        |lc| lc + &active_tail_relation,
        |lc| lc,
    );
    let next_len = AllocatedNum::alloc(cs.namespace(|| format!("{label}_next_length")), || {
        let current = scalar_u64(
            buffer_len
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)?,
        );
        let appended = scalar_u64(
            visible_bytes
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)?,
        );
        if active.get_value() == Some(true) {
            current
                .checked_add(appended)
                .filter(|value| *value <= BYTE_CONTEXT_BUFFER_BYTES as u64)
                .map(Scalar::from)
                .ok_or_else(|| {
                    SynthesisError::Unsatisfiable(
                        "byte-context append exceeds fixed queue".to_owned(),
                    )
                })
        } else {
            Ok(Scalar::from(0_u64))
        }
    })?;
    cs.enforce(
        || format!("{label}_active_length_successor"),
        |lc| lc + active.get_variable(),
        |lc| {
            lc + next_len.get_variable() - buffer_len.get_variable() - visible_bytes.get_variable()
        },
        |lc| lc,
    );
    constrain_bounded_u8(
        cs.namespace(|| format!("{label}_next_length_bound")),
        &next_len,
        BYTE_CONTEXT_BUFFER_BYTES,
    )?;

    let zero = allocate_constant(cs.namespace(|| format!("{label}_zero")), 0)?;
    let mut outputs = Vec::with_capacity(BYTE_CONTEXT_BUFFER_BYTES + 1);
    outputs.push((start + BYTE_CONTEXT_BUFFER_LEN_OFFSET, next_len));
    for index in 0..BYTE_CONTEXT_BUFFER_BYTES {
        let output = AllocatedNum::alloc(cs.namespace(|| format!("{label}_byte_{index}")), || {
            let current = scalar_u64(
                buffer_len
                    .get_value()
                    .ok_or(SynthesisError::AssignmentMissing)?,
            ) as usize;
            if current >= TRACE_CANONICAL_CHUNK_BYTES_V2 {
                Ok(Scalar::from(0_u64))
            } else if index < current {
                z[start + BYTE_CONTEXT_BUFFER_START_OFFSET + index]
                    .get_value()
                    .ok_or(SynthesisError::AssignmentMissing)
            } else {
                let chunk_index = index - current;
                Ok(bytes
                    .get(chunk_index)
                    .map(|value| value.get_value().ok_or(SynthesisError::AssignmentMissing))
                    .transpose()?
                    .unwrap_or(zero.get_value().ok_or(SynthesisError::AssignmentMissing)?))
            }
        })?;
        for (length, selector) in selectors.iter().enumerate() {
            let expected = if index < length {
                &z[start + BYTE_CONTEXT_BUFFER_START_OFFSET + index]
            } else if index < length + bytes.len() {
                &bytes[index - length]
            } else {
                &zero
            };
            enforce_gated_equal(
                cs.namespace(|| format!("{label}_byte_{index}_length_{length}")),
                selector,
                &output,
                expected,
            );
        }
        outputs.push((start + BYTE_CONTEXT_BUFFER_START_OFFSET + index, output));
    }
    Ok(outputs)
}

/// Constrain one selected SHA control to consume the first 64 bytes of exactly
/// one byte context.  In EOF mode the same relation derives FIPS `0x80`, zero
/// fill, and the big-endian message bit length; the host block witness is only
/// accepted when it equals those constrained bytes.
fn consume_context_block<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    z: &[AllocatedNum<Scalar>],
    start: usize,
    selector: &AllocatedBit,
    clear_static_prefix_pending: bool,
    control: &AllocatedHashControlV2,
    sha: &ShaCompressionOutputsV2,
    label: &str,
) -> Result<Vec<(usize, AllocatedNum<Scalar>)>, SynthesisError> {
    let zero = allocate_constant(cs.namespace(|| format!("{label}_zero")), 0)?;
    let padding_marker =
        allocate_constant(cs.namespace(|| format!("{label}_padding_marker")), 0x80)?;
    enforce_gated_equal(
        cs.namespace(|| format!("{label}_block_index_matches_context")),
        selector,
        &control.block_index,
        &z[start + BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET],
    );
    enforce_gated_equal(
        cs.namespace(|| format!("{label}_block_count_matches_context")),
        selector,
        &control.block_count,
        &z[start + BYTE_CONTEXT_BLOCK_COUNT_OFFSET],
    );
    let buffer_len = &z[start + BYTE_CONTEXT_BUFFER_LEN_OFFSET];
    // A full block needs no exact-length mux: the shift relation below is
    // sufficient.  Exact selectors exist only for the <64-byte FIPS padding
    // case, keeping every inactive dynamic-block lane bounded by the one
    // canonical tail width rather than the queue capacity.
    let mut selectors = Vec::with_capacity(TRACE_CANONICAL_CHUNK_BYTES_V2);
    let mut short_lc = nova_snark::frontend::LinearCombination::zero();
    let mut short_high_lc = nova_snark::frontend::LinearCombination::zero();
    for length in 0..TRACE_CANONICAL_CHUNK_BYTES_V2 {
        let length_value = allocate_constant(
            cs.namespace(|| format!("{label}_length_value_{length}")),
            u64::try_from(length).map_err(|_| {
                SynthesisError::Unsatisfiable("SHA tail length overflow".to_owned())
            })?,
        )?;
        let length_selector = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| format!("{label}_length_selector_{length}")),
            buffer_len,
            &length_value,
        )?;
        short_lc = short_lc + length_selector.get_variable();
        if (56..64).contains(&length) {
            short_high_lc = short_high_lc + length_selector.get_variable();
        }
        selectors.push(length_selector);
    }
    let full = AllocatedBit::alloc(
        cs.namespace(|| format!("{label}_has_full_block")),
        buffer_len.get_value().map(|value| scalar_u64(value) >= 64),
    )?;
    cs.enforce(
        || format!("{label}_has_full_block_relation"),
        |lc| lc + full.get_variable() + &short_lc - CS::one(),
        |lc| lc + CS::one(),
        |lc| lc,
    );
    let not_full = allocate_bit_not(cs.namespace(|| format!("{label}_not_full")), &full, "value")?;
    let eof = allocate_state_bit(
        cs.namespace(|| format!("{label}_eof")),
        &z[start + BYTE_CONTEXT_EOF_OFFSET],
    )?;
    let short_high = AllocatedBit::alloc(
        cs.namespace(|| format!("{label}_short_high")),
        buffer_len
            .get_value()
            .map(|value| (56..64).contains(&(scalar_u64(value) as usize))),
    )?;
    cs.enforce(
        || format!("{label}_short_high_relation"),
        |lc| lc + short_high.get_variable() - &short_high_lc,
        |lc| lc + CS::one(),
        |lc| lc,
    );
    let eof_short = AllocatedBit::and(
        cs.namespace(|| format!("{label}_eof_short")),
        &eof,
        &not_full,
    )?;
    let two_padding = AllocatedBit::and(
        cs.namespace(|| format!("{label}_two_padding")),
        &eof_short,
        &short_high,
    )?;
    let not_two_padding = allocate_bit_not(
        cs.namespace(|| format!("{label}_not_two_padding")),
        &two_padding,
        "value",
    )?;
    let expected_final = AllocatedBit::and(
        cs.namespace(|| format!("{label}_expected_final")),
        &eof_short,
        &not_two_padding,
    )?;
    let final_bit = AllocatedBit::alloc(
        cs.namespace(|| format!("{label}_final_flag")),
        control
            .final_block
            .get_value()
            .map(|value| value == Scalar::from(1_u64)),
    )?;
    cs.enforce(
        || format!("{label}_final_flag_matches_control"),
        |lc| lc + final_bit.get_variable() - control.final_block.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc,
    );
    let final_bit_num = bit_as_num(
        cs.namespace(|| format!("{label}_final_flag_num")),
        &final_bit,
        "value",
    )?;
    let expected_final_num = bit_as_num(
        cs.namespace(|| format!("{label}_expected_final_num")),
        &expected_final,
        "value",
    )?;
    enforce_gated_equal(
        cs.namespace(|| format!("{label}_final_flag_relation")),
        selector,
        &final_bit_num,
        &expected_final_num,
    );
    let not_eof = allocate_bit_not(cs.namespace(|| format!("{label}_not_eof")), &eof, "value")?;
    let invalid_empty = AllocatedBit::and(
        cs.namespace(|| format!("{label}_invalid_empty")),
        &not_full,
        &not_eof,
    )?;
    cs.enforce(
        || format!("{label}_block_requires_data_or_eof"),
        |lc| lc + selector.get_variable(),
        |lc| lc + invalid_empty.get_variable(),
        |lc| lc,
    );

    let bit_length = AllocatedNum::alloc(cs.namespace(|| format!("{label}_bit_length")), || {
        scalar_u64(
            z[start + BYTE_CONTEXT_MESSAGE_BYTES_OFFSET]
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)?,
        )
        .checked_mul(8)
        .map(Scalar::from)
        .ok_or_else(|| SynthesisError::Unsatisfiable("SHA bit length overflow".to_owned()))
    })?;
    cs.enforce(
        || format!("{label}_bit_length_relation"),
        |lc| {
            lc + bit_length.get_variable()
                - (
                    Scalar::from(8_u64),
                    z[start + BYTE_CONTEXT_MESSAGE_BYTES_OFFSET].get_variable(),
                )
        },
        |lc| lc + CS::one(),
        |lc| lc,
    );
    let mut bit_length_bytes = decompose_le_bytes(
        cs.namespace(|| format!("{label}_bit_length_bytes")),
        &bit_length,
        8,
        &format!("{label}_bit_length"),
    )?;
    bit_length_bytes.reverse();
    let padding_blocks = allocate_state_bit(
        cs.namespace(|| format!("{label}_padding_blocks")),
        &z[start + BYTE_CONTEXT_PADDING_BLOCKS_OFFSET],
    )?;

    for index in 0_usize..64 {
        // Keep these candidates allocated for the entire `length` selector
        // loop.  Taking a reference to a per-branch temporary would make the
        // padding relation host-shape-dependent rather than one fixed R1CS
        // relation.
        let zero_pad_marker = select_bit_num(
            cs.namespace(|| format!("{label}_zero_pad_marker_{index}")),
            &padding_blocks,
            &zero,
            &padding_marker,
            "value",
        )?;
        let length_or_zero = select_bit_num(
            cs.namespace(|| format!("{label}_length_or_zero_{index}")),
            &two_padding,
            &zero,
            &bit_length_bytes[index.saturating_sub(56)],
            "value",
        )?;
        let full_selector = AllocatedBit::and(
            cs.namespace(|| format!("{label}_full_selector_{index}")),
            selector,
            &full,
        )?;
        enforce_gated_equal(
            cs.namespace(|| format!("{label}_full_block_byte_{index}")),
            &full_selector,
            &sha.block_bytes[index],
            &z[start + BYTE_CONTEXT_BUFFER_START_OFFSET + index],
        );
        let padding_selector = AllocatedBit::and(
            cs.namespace(|| format!("{label}_padding_selector_{index}")),
            selector,
            &eof_short,
        )?;
        for (length, length_selector) in selectors.iter().take(64).enumerate() {
            let expected = if index < length {
                &z[start + BYTE_CONTEXT_BUFFER_START_OFFSET + index]
            } else if index == length {
                if length == 0 {
                    &zero_pad_marker
                } else {
                    &padding_marker
                }
            } else if index >= 56 {
                &length_or_zero
            } else {
                &zero
            };
            let gate = AllocatedBit::and(
                cs.namespace(|| format!("{label}_padding_gate_{index}_{length}")),
                &padding_selector,
                length_selector,
            )?;
            enforce_gated_equal(
                cs.namespace(|| format!("{label}_padding_byte_{index}_{length}")),
                &gate,
                &sha.block_bytes[index],
                expected,
            );
        }
    }

    let next_len = AllocatedNum::alloc(cs.namespace(|| format!("{label}_next_length")), || {
        let current = scalar_u64(
            buffer_len
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)?,
        );
        if current >= 64 {
            Ok(Scalar::from(current - 64))
        } else {
            Ok(Scalar::from(0_u64))
        }
    })?;
    enforce_gated_constant(
        cs.namespace(|| format!("{label}_short_next_length_zero")),
        &not_full,
        &next_len,
        0,
    );
    cs.enforce(
        || format!("{label}_full_next_length"),
        |lc| lc + full.get_variable(),
        |lc| {
            lc + next_len.get_variable() + (Scalar::from(64_u64), CS::one())
                - buffer_len.get_variable()
        },
        |lc| lc,
    );
    let next_padding_bit = allocate_bit_or(
        cs.namespace(|| format!("{label}_next_padding_bit")),
        &padding_blocks,
        &two_padding,
        "value",
    )?;
    let next_padding = bit_as_num(
        cs.namespace(|| format!("{label}_next_padding")),
        &next_padding_bit,
        "value",
    )?;
    let next_block_index = allocate_incremented(
        cs.namespace(|| format!("{label}_next_block_index")),
        &z[start + BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET],
        &format!("{label}_next_block_index_value"),
    )?;
    let next_pending = if clear_static_prefix_pending {
        let pending = allocate_state_bit(
            cs.namespace(|| format!("{label}_pending")),
            &z[start + BYTE_CONTEXT_PENDING_OFFSET],
        )?;
        let first_static_block = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| format!("{label}_first_static_block")),
            &control.block_index,
            &zero,
        )?;
        let remains_pending = AllocatedBit::and(
            cs.namespace(|| format!("{label}_pending_after_static_block")),
            &pending,
            &first_static_block,
        )?;
        bit_as_num(
            cs.namespace(|| format!("{label}_pending_output")),
            &remains_pending,
            "value",
        )?
    } else {
        z[start + BYTE_CONTEXT_PENDING_OFFSET].clone()
    };
    let mut outputs = Vec::with_capacity(BYTE_CONTEXT_BUFFER_BYTES + 11);
    outputs.push((start + BYTE_CONTEXT_BUFFER_LEN_OFFSET, next_len));
    outputs.push((start + BYTE_CONTEXT_PADDING_BLOCKS_OFFSET, next_padding));
    outputs.push((start + BYTE_CONTEXT_PENDING_OFFSET, next_pending));
    outputs.push((
        start + BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET,
        next_block_index,
    ));
    for index in 0..BYTE_CONTEXT_BUFFER_BYTES {
        let shifted = if index + 64 < BYTE_CONTEXT_BUFFER_BYTES {
            &z[start + BYTE_CONTEXT_BUFFER_START_OFFSET + index + 64]
        } else {
            &zero
        };
        let value = select_bit_num(
            cs.namespace(|| format!("{label}_buffer_output_{index}")),
            &full,
            shifted,
            &zero,
            "value",
        )?;
        outputs.push((start + BYTE_CONTEXT_BUFFER_START_OFFSET + index, value));
    }
    for (index, after) in sha.block_next_chaining.iter().enumerate() {
        outputs.push((
            start + BYTE_CONTEXT_CHAINING_START_OFFSET + index,
            after.clone(),
        ));
    }
    Ok(outputs)
}

/// Consume one of the compile-time framing blocks that precede the live
/// canonical-byte feeder.  Those bytes belong to the sole SHA owner and are
/// derived by a fixed cursor; retaining them in the Nova state would turn a
/// three-byte tail into a 131-byte source-record arena.  Only chaining, the
/// block cursor, and the pending-static flag transition here.
fn consume_static_context_block<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    z: &[AllocatedNum<Scalar>],
    start: usize,
    selector: &AllocatedBit,
    control: &AllocatedHashControlV2,
    sha: &ShaCompressionOutputsV2,
    static_blocks: &[&[AllocatedNum<Scalar>]],
    label: &str,
) -> Result<Vec<(usize, AllocatedNum<Scalar>)>, SynthesisError> {
    if static_blocks.is_empty()
        || static_blocks
            .iter()
            .any(|block| block.len() != SHA_BLOCK_END - SHA_BLOCK_START)
    {
        return Err(SynthesisError::Unsatisfiable(
            "static SHA framing block width mismatch".to_owned(),
        ));
    }
    enforce_gated_constant(
        cs.namespace(|| format!("{label}_pending")),
        selector,
        &z[start + BYTE_CONTEXT_PENDING_OFFSET],
        1,
    );
    enforce_gated_equal(
        cs.namespace(|| format!("{label}_block_index_matches_context")),
        selector,
        &control.block_index,
        &z[start + BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET],
    );
    enforce_gated_equal(
        cs.namespace(|| format!("{label}_block_count_matches_context")),
        selector,
        &control.block_count,
        &z[start + BYTE_CONTEXT_BLOCK_COUNT_OFFSET],
    );
    enforce_gated_constant(
        cs.namespace(|| format!("{label}_not_final")),
        selector,
        &control.final_block,
        0,
    );

    let mut index_selectors = Vec::with_capacity(static_blocks.len());
    let mut selector_sum = nova_snark::frontend::LinearCombination::zero() - CS::one();
    for index in 0..static_blocks.len() {
        let index_value = allocate_constant(
            cs.namespace(|| format!("{label}_block_index_{index}")),
            u64::try_from(index).map_err(|_| {
                SynthesisError::Unsatisfiable("static SHA block index overflow".to_owned())
            })?,
        )?;
        let is_index = nova_snark::gadgets::utils::alloc_num_equals(
            cs.namespace(|| format!("{label}_is_block_{index}")),
            &control.block_index,
            &index_value,
        )?;
        selector_sum = selector_sum + is_index.get_variable();
        index_selectors.push(is_index);
    }
    cs.enforce(
        || format!("{label}_selects_one_static_block"),
        |lc| lc + selector.get_variable(),
        |lc| lc + &selector_sum,
        |lc| lc,
    );
    for (block_index, (block_selector, expected_block)) in
        index_selectors.iter().zip(static_blocks.iter()).enumerate()
    {
        let gate = AllocatedBit::and(
            cs.namespace(|| format!("{label}_block_gate_{block_index}")),
            selector,
            block_selector,
        )?;
        for (byte_index, (actual, expected)) in sha
            .block_bytes
            .iter()
            .zip(expected_block.iter())
            .enumerate()
        {
            enforce_gated_equal(
                cs.namespace(|| format!("{label}_byte_{block_index}_{byte_index}")),
                &gate,
                actual,
                expected,
            );
        }
    }

    let next_block_index = allocate_incremented(
        cs.namespace(|| format!("{label}_next_block_index")),
        &z[start + BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET],
        &format!("{label}_next_block_index_value"),
    )?;
    let next_pending =
        AllocatedNum::alloc(cs.namespace(|| format!("{label}_next_pending")), || {
            let index = usize::try_from(scalar_u64(
                control
                    .block_index
                    .get_value()
                    .ok_or(SynthesisError::AssignmentMissing)?,
            ))
            .map_err(|_| {
                SynthesisError::Unsatisfiable("static SHA block index exceeds usize".to_owned())
            })?;
            Ok(Scalar::from(u64::from(index + 1 < static_blocks.len())))
        })?;
    for (index, block_selector) in index_selectors.iter().enumerate() {
        let gate = AllocatedBit::and(
            cs.namespace(|| format!("{label}_pending_gate_{index}")),
            selector,
            block_selector,
        )?;
        enforce_gated_constant(
            cs.namespace(|| format!("{label}_pending_value_{index}")),
            &gate,
            &next_pending,
            u64::from(index + 1 < static_blocks.len()),
        );
    }

    let mut outputs = Vec::with_capacity(10);
    outputs.push((
        start + BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET,
        next_block_index,
    ));
    outputs.push((start + BYTE_CONTEXT_PENDING_OFFSET, next_pending));
    for (index, after) in sha.block_next_chaining.iter().enumerate() {
        outputs.push((
            start + BYTE_CONTEXT_CHAINING_START_OFFSET + index,
            after.clone(),
        ));
    }
    Ok(outputs)
}

/// Bind source records and their derived BEGIN_HASH / SHA_BLOCK* / END_HASH
/// controls as one fixed R1CS schedule.  The expander remains the sole owner
/// of the controls; this only consumes its decoded witness values.
fn synthesize_hash_control_schedule<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    z: &[AllocatedNum<Scalar>],
    event_ordinal: &AllocatedNum<Scalar>,
    control: &AllocatedHashControlV2,
    trace_chunk_selector: &AllocatedBit,
    trace_chunk: &AllocatedTraceChunkV2,
    source_header: &[AllocatedNum<Scalar>],
    sha: &ShaCompressionOutputsV2,
) -> Result<HashScheduleOutputsV2, SynthesisError> {
    if source_header.len() != TRACE_EVENT_HEADER_BYTES_V2 {
        return Err(SynthesisError::Unsatisfiable(
            "canonical source header width mismatch".to_owned(),
        ));
    }
    let selectors = &control.stage_selectors;
    let source = control.source_selector();
    let begin = control.begin_selector();
    let block = control.block_selector();
    let end = control.end_selector();
    let zero = allocate_constant(cs.namespace(|| "zero"), 0)?;
    let one = allocate_constant(cs.namespace(|| "one"), 1)?;
    // The sole SHA owner exposes the frozen role/DST prefix.  Nova materializes
    // that owner-provided prefix, the source-label part, and the constrained
    // declared record length here; it never reconstructs a second framing
    // grammar or accepts a host-side byte assertion.
    let source_static_frame = source_static_frame_bytes(
        cs.namespace(|| "source_static_frame"),
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CANONICAL_BYTES_OFFSET],
    )?;
    let static_block_bytes = SHA_BLOCK_END - SHA_BLOCK_START;
    let source_static_blocks = 2 * static_block_bytes;
    if source_static_frame.len() < source_static_blocks
        || source_static_frame.len() - source_static_blocks > BYTE_CONTEXT_BUFFER_BYTES
    {
        return Err(SynthesisError::Unsatisfiable(
            "source static SHA frame does not fit cursor plus tail".to_owned(),
        ));
    }
    let source_static_frame_len = u64::try_from(source_static_frame.len()).map_err(|_| {
        SynthesisError::Unsatisfiable("source static SHA frame length overflow".to_owned())
    })?;
    let source_static_tail_len_value = allocate_constant(
        cs.namespace(|| "source_static_frame_tail_len"),
        u64::try_from(source_static_frame.len() - source_static_blocks).map_err(|_| {
            SynthesisError::Unsatisfiable("source static SHA tail length overflow".to_owned())
        })?,
    )?;
    let trace_role_prefix = trace_role_prefix_bytes(cs.namespace(|| "trace_role_prefix"))?;
    if trace_role_prefix.len() < static_block_bytes
        || trace_role_prefix.len() - static_block_bytes > BYTE_CONTEXT_BUFFER_BYTES
    {
        return Err(SynthesisError::Unsatisfiable(
            "trace SHA role prefix does not fit cursor plus tail".to_owned(),
        ));
    }
    let trace_role_prefix_tail_len_value = allocate_constant(
        cs.namespace(|| "trace_role_prefix_tail_len"),
        u64::try_from(trace_role_prefix.len() - static_block_bytes).map_err(|_| {
            SynthesisError::Unsatisfiable("trace SHA role tail length overflow".to_owned())
        })?,
    )?;
    let iv_words = SHA256_IV_V2.map(u64::from);
    let mut iv = Vec::with_capacity(8);
    for (index, word) in iv_words.into_iter().enumerate() {
        iv.push(allocate_constant(
            cs.namespace(|| format!("sha_iv_{index}")),
            word,
        )?);
    }
    let source_trace_next = allocate_incremented(
        cs.namespace(|| "source_trace_next"),
        &control.source_ordinal,
        "source_trace_next_value",
    )?;
    let source_trace_byte_count_next =
        AllocatedNum::alloc(cs.namespace(|| "source_trace_byte_count_next"), || {
            match (
                z[SOURCE_TRACE_BYTE_COUNT_CELL].get_value(),
                control.source_record_bytes.get_value(),
            ) {
                (Some(current), Some(record_bytes)) => Ok(current + record_bytes),
                _ => Err(SynthesisError::AssignmentMissing),
            }
        })?;
    enforce_sum(
        cs.namespace(|| "source_trace_byte_count_successor"),
        &source_trace_byte_count_next,
        &z[SOURCE_TRACE_BYTE_COUNT_CELL],
        &control.source_record_bytes,
    );
    let control_ordinal_next = allocate_incremented(
        cs.namespace(|| "control_ordinal_next"),
        event_ordinal,
        "control_ordinal_next_value",
    )?;
    let block_index_next = allocate_incremented(
        cs.namespace(|| "block_index_next"),
        &control.block_index,
        "block_index_next_value",
    )?;

    // A source record is only legal after END_HASH has canonically cleared the
    // pending schedule and SHA cells.  It installs the source binding that its
    // immediately following BEGIN_HASH must consume.
    enforce_gated_equal(
        cs.namespace(|| "source_ordinal_matches_event"),
        source,
        event_ordinal,
        &control.source_ordinal,
    );
    let source_record_is_empty = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "source_record_bytes_nonzero"),
        &control.source_record_bytes,
        &zero,
    )?;
    cs.enforce(
        || "source_record_bytes_are_nonzero",
        |lc| lc + source.get_variable(),
        |lc| lc + source_record_is_empty.get_variable(),
        |lc| lc,
    );
    enforce_gated_equal(
        cs.namespace(|| "source_trace_matches_source_ordinal"),
        source,
        &z[SOURCE_TRACE_ORDINAL_CELL],
        &control.source_ordinal,
    );
    for index in COUNTERS_START..COUNTERS_END {
        enforce_gated_constant(
            cs.namespace(|| format!("source_schedule_input_zero_{index}")),
            source,
            &z[index],
            0,
        );
    }
    for index in SHA_ACTIVE_CELL..SHA_END {
        enforce_gated_constant(
            cs.namespace(|| format!("source_sha_input_zero_{index}")),
            source,
            &z[index],
            0,
        );
    }
    enforce_gated_constant(
        cs.namespace(|| "source_context_must_be_inactive"),
        source,
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_ACTIVE_OFFSET],
        0,
    );
    // A source record is only meaningful in the one global transcript that
    // was opened by its canonical TRACE BEGIN. Without this, a locally valid
    // source SHA schedule could be replayed before (or independently of) the
    // global byte context that must consume the same TraceChunk rows.
    enforce_gated_constant(
        cs.namespace(|| "source_requires_global_context_active"),
        source,
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_ACTIVE_OFFSET],
        1,
    );
    enforce_gated_constant(
        cs.namespace(|| "source_requires_global_context_started"),
        source,
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_STARTED_OFFSET],
        1,
    );

    let source_begin = AllocatedBit::and(
        cs.namespace(|| "source_begin"),
        begin,
        control.source_schema_selector(),
    )?;
    let source_block = AllocatedBit::and(
        cs.namespace(|| "source_block"),
        block,
        control.source_schema_selector(),
    )?;
    let source_end = AllocatedBit::and(
        cs.namespace(|| "source_end"),
        end,
        control.source_schema_selector(),
    )?;
    for (name, selector, expected_started) in [
        ("source_begin_context", &source_begin, 0_u64),
        ("source_block_context", &source_block, 1_u64),
        ("source_end_context", &source_end, 1_u64),
    ] {
        enforce_gated_constant(
            cs.namespace(|| format!("{name}_active")),
            selector,
            &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_ACTIVE_OFFSET],
            1,
        );
        enforce_gated_constant(
            cs.namespace(|| format!("{name}_started")),
            selector,
            &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_STARTED_OFFSET],
            expected_started,
        );
    }
    cs.enforce(
        || "source_begin_message_bytes_match_framing_and_record",
        |lc| lc + source_begin.get_variable(),
        |lc| {
            lc + control.message_bytes.get_variable()
                - z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CANONICAL_BYTES_OFFSET].get_variable()
                - (Scalar::from(source_static_frame_len), CS::one())
        },
        |lc| lc,
    );
    // Per-source controls refer to the just-consumed source record: its
    // source ordinal is one behind the next source-record ordinal in state.
    for (selector_index, selector) in [
        source_begin.clone(),
        source_block.clone(),
        source_end.clone(),
    ]
    .into_iter()
    .enumerate()
    {
        enforce_gated_increment(
            cs.namespace(|| format!("control_source_ordinal_matches_trace_{selector_index}")),
            &selector,
            &control.source_ordinal,
            &z[SOURCE_TRACE_ORDINAL_CELL],
        );
        enforce_gated_equal(
            cs.namespace(|| format!("control_source_ordinal_matches_schedule_{selector_index}")),
            &selector,
            &z[HASH_SCHEDULE_SOURCE_ORDINAL_CELL],
            &control.source_ordinal,
        );
        enforce_gated_equal(
            cs.namespace(|| format!("control_role_matches_schedule_{selector_index}")),
            &selector,
            &z[HASH_SCHEDULE_ROLE_CELL],
            &control.role,
        );
        for (index, hash_limb) in control.source_hash_limbs.iter().enumerate() {
            enforce_gated_equal(
                cs.namespace(|| {
                    format!("control_source_hash_matches_schedule_{selector_index}_{index}")
                }),
                &selector,
                &z[HASH_SCHEDULE_SOURCE_HASH_START + index],
                hash_limb,
            );
        }
    }

    let trace_begin = AllocatedBit::and(
        cs.namespace(|| "trace_begin"),
        begin,
        control.trace_schema_selector(),
    )?;
    let trace_block = AllocatedBit::and(
        cs.namespace(|| "trace_block"),
        block,
        control.trace_schema_selector(),
    )?;
    let trace_end = AllocatedBit::and(
        cs.namespace(|| "trace_end"),
        end,
        control.trace_schema_selector(),
    )?;
    enforce_gated_constant(
        cs.namespace(|| "trace_begin_global_context_inactive"),
        &trace_begin,
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_ACTIVE_OFFSET],
        0,
    );
    enforce_gated_constant(
        cs.namespace(|| "trace_begin_requires_no_replayed_sources"),
        &trace_begin,
        &z[SOURCE_TRACE_ORDINAL_CELL],
        0,
    );
    enforce_gated_constant(
        cs.namespace(|| "trace_begin_requires_no_replayed_source_bytes"),
        &trace_begin,
        &z[SOURCE_TRACE_BYTE_COUNT_CELL],
        0,
    );
    enforce_gated_equal(
        cs.namespace(|| "trace_begin_ordinal_matches_trace_count"),
        &trace_begin,
        &control.source_ordinal,
        &control.trace_event_count,
    );
    enforce_gated_trace_geometry(
        cs.namespace(|| "trace_begin_geometry"),
        &trace_begin,
        control,
    )?;
    enforce_gated_hash_control_ordinal(
        cs.namespace(|| "trace_begin_control_ordinal"),
        &trace_begin,
        event_ordinal,
        &control.source_ordinal,
        &zero,
        false,
    );
    enforce_gated_equal(
        cs.namespace(|| "trace_begin_role"),
        &trace_begin,
        &control.role,
        &one,
    );
    for (index, hash_limb) in control.source_hash_limbs.iter().enumerate() {
        enforce_gated_equal(
            cs.namespace(|| format!("trace_begin_hash_matches_precommit_{index}")),
            &trace_begin,
            hash_limb,
            &z[EXPECTED_TRACE_DIGEST_START + index],
        );
    }
    for (selector_index, selector) in [trace_block.clone(), trace_end.clone()]
        .into_iter()
        .enumerate()
    {
        enforce_gated_equal(
            cs.namespace(|| format!("trace_control_role_{selector_index}")),
            &selector,
            &control.role,
            &one,
        );
        for (index, hash_limb) in control.source_hash_limbs.iter().enumerate() {
            enforce_gated_equal(
                cs.namespace(|| {
                    format!("trace_control_hash_matches_precommit_{selector_index}_{index}")
                }),
                &selector,
                hash_limb,
                &z[EXPECTED_TRACE_DIGEST_START + index],
            );
        }
        enforce_gated_trace_geometry(
            cs.namespace(|| format!("trace_control_geometry_{selector_index}")),
            &selector,
            control,
        )?;
    }
    enforce_gated_equal(
        cs.namespace(|| "trace_end_count_matches_replayed_sources"),
        &trace_end,
        &control.trace_event_count,
        &z[SOURCE_TRACE_ORDINAL_CELL],
    );
    enforce_gated_equal(
        cs.namespace(|| "trace_end_bytes_match_replayed_sources"),
        &trace_end,
        &control.trace_byte_count,
        &z[SOURCE_TRACE_BYTE_COUNT_CELL],
    );
    for (selector_index, selector) in [trace_block.clone(), trace_end.clone()]
        .into_iter()
        .enumerate()
    {
        enforce_gated_equal(
            cs.namespace(|| format!("trace_control_ordinal_matches_context_{selector_index}")),
            &selector,
            &control.source_ordinal,
            &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_SOURCE_ORDINAL_OFFSET],
        );
        for (name, offset, expected) in [
            ("active", BYTE_CONTEXT_ACTIVE_OFFSET, 1_u64),
            ("started", BYTE_CONTEXT_STARTED_OFFSET, 1_u64),
        ] {
            enforce_gated_constant(
                cs.namespace(|| format!("trace_control_{name}_{selector_index}")),
                &selector,
                &z[GLOBAL_BYTE_CONTEXT_START + offset],
                expected,
            );
        }
        enforce_gated_equal(
            cs.namespace(|| format!("trace_control_message_bytes_{selector_index}")),
            &selector,
            &control.message_bytes,
            &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_MESSAGE_BYTES_OFFSET],
        );
        enforce_gated_equal(
            cs.namespace(|| format!("trace_control_block_count_{selector_index}")),
            &selector,
            &control.block_count,
            &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_BLOCK_COUNT_OFFSET],
        );
    }
    enforce_gated_equal(
        cs.namespace(|| "trace_block_index_matches_context"),
        &trace_block,
        &control.block_index,
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET],
    );
    enforce_gated_hash_control_ordinal(
        cs.namespace(|| "trace_block_control_ordinal"),
        &trace_block,
        event_ordinal,
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_SOURCE_ORDINAL_OFFSET],
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET],
        true,
    );
    enforce_gated_equal(
        cs.namespace(|| "trace_end_seen_all_blocks"),
        &trace_end,
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET],
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_BLOCK_COUNT_OFFSET],
    );
    enforce_gated_hash_control_ordinal(
        cs.namespace(|| "trace_end_control_ordinal"),
        &trace_end,
        event_ordinal,
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_SOURCE_ORDINAL_OFFSET],
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_BLOCK_COUNT_OFFSET],
        true,
    );

    // BEGIN_HASH starts exactly at sequence zero and may only consume the
    // source binding that the preceding source-record step installed.
    enforce_gated_constant(
        cs.namespace(|| "begin_schedule_inactive"),
        &source_begin,
        &z[HASH_SCHEDULE_ACTIVE_CELL],
        0,
    );
    for index in [
        HASH_SCHEDULE_ORDINAL_CELL,
        HASH_SCHEDULE_MESSAGE_BYTES_CELL,
        HASH_SCHEDULE_BLOCK_COUNT_CELL,
        HASH_SCHEDULE_NEXT_BLOCK_INDEX_CELL,
        HASH_SCHEDULE_FINAL_BLOCK_CELL,
    ] {
        enforce_gated_constant(
            cs.namespace(|| format!("begin_schedule_zero_{index}")),
            &source_begin,
            &z[index],
            0,
        );
    }
    enforce_gated_hash_control_ordinal(
        cs.namespace(|| "begin_control_ordinal"),
        &source_begin,
        event_ordinal,
        &control.source_ordinal,
        &zero,
        false,
    );
    let empty_block_count = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "begin_block_count_nonzero"),
        &control.block_count,
        &zero,
    )?;
    cs.enforce(
        || "begin_block_count_is_nonzero",
        |lc| lc + source_begin.get_variable(),
        |lc| lc + empty_block_count.get_variable(),
        |lc| lc,
    );
    for index in SHA_ACTIVE_CELL..SHA_END {
        enforce_gated_constant(
            cs.namespace(|| format!("begin_sha_input_zero_{index}")),
            &source_begin,
            &z[index],
            0,
        );
    }

    // Every SHA_BLOCK advances the canonical encoded ordinal and exactly one
    // 64-byte block index.  The final marker is derived in-circuit, not
    // accepted from native transition verification.
    enforce_gated_constant(
        cs.namespace(|| "block_schedule_active"),
        &source_block,
        &z[HASH_SCHEDULE_ACTIVE_CELL],
        1,
    );
    for (index, (state_index, witness_value)) in [
        (HASH_SCHEDULE_MESSAGE_BYTES_CELL, &control.message_bytes),
        (HASH_SCHEDULE_BLOCK_COUNT_CELL, &control.block_count),
        (HASH_SCHEDULE_NEXT_BLOCK_INDEX_CELL, &control.block_index),
    ]
    .into_iter()
    .enumerate()
    {
        enforce_gated_equal(
            cs.namespace(|| format!("block_schedule_metadata_matches_{index}")),
            &source_block,
            &z[state_index],
            witness_value,
        );
    }
    enforce_gated_constant(
        cs.namespace(|| "block_prior_final_is_zero"),
        &source_block,
        &z[HASH_SCHEDULE_FINAL_BLOCK_CELL],
        0,
    );
    enforce_gated_equal(
        cs.namespace(|| "block_schedule_ordinal_matches_event"),
        &source_block,
        &z[HASH_SCHEDULE_ORDINAL_CELL],
        event_ordinal,
    );
    enforce_gated_hash_control_ordinal(
        cs.namespace(|| "block_control_ordinal"),
        &source_block,
        event_ordinal,
        &control.source_ordinal,
        &control.block_index,
        true,
    );
    cs.enforce(
        || "block_byte_offset",
        |lc| lc + block.get_variable(),
        |lc| {
            lc + control.byte_offset.get_variable()
                - (Scalar::from(64_u64), control.block_index.get_variable())
        },
        |lc| lc,
    );
    let is_final_block = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "final_block_index_relation"),
        &block_index_next,
        &control.block_count,
    )?;
    cs.enforce(
        || "final_block_flag_relation",
        |lc| lc + block.get_variable(),
        |lc| lc + control.final_block.get_variable() - is_final_block.get_variable(),
        |lc| lc,
    );
    let final_block_bit = AllocatedBit::alloc(
        cs.namespace(|| "final_block_bit"),
        control
            .final_block
            .get_value()
            .map(|value| value == Scalar::from(1_u64)),
    )?;
    cs.enforce(
        || "final_block_bit_matches_control",
        |lc| lc + final_block_bit.get_variable() - control.final_block.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc,
    );
    let trace_final_block = AllocatedBit::and(
        cs.namespace(|| "trace_final_block"),
        &trace_block,
        &final_block_bit,
    )?;
    cs.enforce(
        || "trace_final_block_bit_length_bytes",
        |lc| lc + trace_final_block.get_variable(),
        |lc| {
            let mut reconstructed = nova_snark::frontend::LinearCombination::zero();
            for (index, byte) in sha.block_bytes[56..64].iter().enumerate() {
                let exponent = u32::try_from(7_usize.saturating_sub(index))
                    .expect("fixed SHA bit-length exponent fits u32");
                reconstructed =
                    reconstructed + (Scalar::from(256_u64.pow(exponent)), byte.get_variable());
            }
            lc + control.trace_bit_length.get_variable() - &reconstructed
        },
        |lc| lc,
    );

    // END_HASH can only close after the final block, and it binds the current
    // FIPS chaining state to the original source hash in standard SHA order.
    enforce_gated_constant(
        cs.namespace(|| "end_schedule_active"),
        &source_end,
        &z[HASH_SCHEDULE_ACTIVE_CELL],
        1,
    );
    for (index, (state_index, witness_value)) in [
        (HASH_SCHEDULE_MESSAGE_BYTES_CELL, &control.message_bytes),
        (HASH_SCHEDULE_BLOCK_COUNT_CELL, &control.block_count),
    ]
    .into_iter()
    .enumerate()
    {
        enforce_gated_equal(
            cs.namespace(|| format!("end_schedule_metadata_matches_{index}")),
            &source_end,
            &z[state_index],
            witness_value,
        );
    }
    enforce_gated_equal(
        cs.namespace(|| "end_schedule_ordinal_matches_event"),
        &source_end,
        &z[HASH_SCHEDULE_ORDINAL_CELL],
        event_ordinal,
    );
    enforce_gated_equal(
        cs.namespace(|| "end_seen_all_blocks"),
        &source_end,
        &z[HASH_SCHEDULE_NEXT_BLOCK_INDEX_CELL],
        &control.block_count,
    );
    enforce_gated_constant(
        cs.namespace(|| "end_requires_final_block"),
        &source_end,
        &z[HASH_SCHEDULE_FINAL_BLOCK_CELL],
        1,
    );
    enforce_gated_hash_control_ordinal(
        cs.namespace(|| "end_control_ordinal"),
        &source_end,
        event_ordinal,
        &control.source_ordinal,
        &control.block_count,
        true,
    );
    for (name, offset, expected) in [
        ("active", BYTE_CONTEXT_ACTIVE_OFFSET, 1_u64),
        ("started", BYTE_CONTEXT_STARTED_OFFSET, 1_u64),
        ("eof", BYTE_CONTEXT_EOF_OFFSET, 1_u64),
        ("pending", BYTE_CONTEXT_PENDING_OFFSET, 0_u64),
    ] {
        enforce_gated_constant(
            cs.namespace(|| format!("source_end_closure_context_{name}")),
            &source_end,
            &z[SOURCE_BYTE_CONTEXT_START + offset],
            expected,
        );
    }
    enforce_gated_equal(
        cs.namespace(|| "source_end_consumed_all_canonical_bytes"),
        &source_end,
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CONSUMED_BYTES_OFFSET],
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CANONICAL_BYTES_OFFSET],
    );
    enforce_gated_equal(
        cs.namespace(|| "source_end_consumed_all_chunks"),
        &source_end,
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_NEXT_CHUNK_OFFSET],
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CHUNK_COUNT_OFFSET],
    );
    enforce_gated_equal(
        cs.namespace(|| "source_end_context_seen_all_blocks"),
        &source_end,
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET],
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_BLOCK_COUNT_OFFSET],
    );
    for (name, offset, expected) in [
        ("active", BYTE_CONTEXT_ACTIVE_OFFSET, 1_u64),
        ("started", BYTE_CONTEXT_STARTED_OFFSET, 1_u64),
        ("eof", BYTE_CONTEXT_EOF_OFFSET, 1_u64),
    ] {
        enforce_gated_constant(
            cs.namespace(|| format!("trace_end_context_{name}")),
            &trace_end,
            &z[GLOBAL_BYTE_CONTEXT_START + offset],
            expected,
        );
    }
    enforce_gated_equal(
        cs.namespace(|| "trace_end_consumed_all_canonical_bytes"),
        &trace_end,
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_CONSUMED_BYTES_OFFSET],
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_CANONICAL_BYTES_OFFSET],
    );
    for word_index in 0..8 {
        enforce_gated_lc_equal(
            cs.namespace(|| format!("end_digest_matches_chaining_{word_index}")),
            &source_end,
            &source_hash_word_lc::<CS>(&control.source_hash_bytes, word_index),
            &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CHAINING_START_OFFSET + word_index],
        );
    }
    for (index, trace_digest_limb) in control.source_hash_limbs.iter().enumerate() {
        enforce_gated_equal(
            cs.namespace(|| format!("trace_end_digest_matches_precommit_{index}")),
            &trace_end,
            trace_digest_limb,
            &z[EXPECTED_TRACE_DIGEST_START + index],
        );
    }
    for word_index in 0..8 {
        enforce_gated_lc_equal(
            cs.namespace(|| format!("trace_end_digest_matches_chaining_{word_index}")),
            &trace_end,
            &source_hash_word_lc::<CS>(&control.source_hash_bytes, word_index),
            &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_CHAINING_START_OFFSET + word_index],
        );
    }

    let mut cells = Vec::with_capacity(COUNTERS_END - COUNTERS_START + SHA_END - SHA_ACTIVE_CELL);
    let stage_value = |name: &str,
                       inactive_value: &AllocatedNum<Scalar>,
                       source_value: &AllocatedNum<Scalar>,
                       begin_value: &AllocatedNum<Scalar>,
                       block_value: &AllocatedNum<Scalar>,
                       end_value: &AllocatedNum<Scalar>,
                       cs: &mut CS|
     -> Result<AllocatedNum<Scalar>, SynthesisError> {
        select_hash_stage_value(
            cs.namespace(|| name),
            selectors,
            [
                inactive_value,
                source_value,
                begin_value,
                block_value,
                end_value,
            ],
        )
    };
    // The legacy schedule cells now track the source-local transcript only.
    // Whole-trace controls have an independent live byte context and can
    // interleave after any source chunk; letting a trace SHA row overwrite
    // this source schedule would reintroduce the shared-state bug.
    let source_schedule_value = |name: &str,
                                 state_index: usize,
                                 inactive_value: &AllocatedNum<Scalar>,
                                 source_value: &AllocatedNum<Scalar>,
                                 begin_value: &AllocatedNum<Scalar>,
                                 block_value: &AllocatedNum<Scalar>,
                                 end_value: &AllocatedNum<Scalar>,
                                 cs: &mut CS|
     -> Result<AllocatedNum<Scalar>, SynthesisError> {
        let source_value = stage_value(
            &format!("{name}_source"),
            inactive_value,
            source_value,
            begin_value,
            block_value,
            end_value,
            cs,
        )?;
        select_hash_schema_value(
            cs.namespace(|| format!("{name}_schema")),
            &control.schema_selectors,
            [&source_value, &z[state_index]],
        )
    };

    cells.push((
        HASH_SCHEDULE_ACTIVE_CELL,
        source_schedule_value(
            "schedule_active_output",
            HASH_SCHEDULE_ACTIVE_CELL,
            &z[HASH_SCHEDULE_ACTIVE_CELL],
            &zero,
            &one,
            &one,
            &zero,
            &mut cs,
        )?,
    ));
    cells.push((
        HASH_SCHEDULE_ORDINAL_CELL,
        source_schedule_value(
            "schedule_ordinal_output",
            HASH_SCHEDULE_ORDINAL_CELL,
            &z[HASH_SCHEDULE_ORDINAL_CELL],
            &zero,
            &control_ordinal_next,
            &control_ordinal_next,
            &zero,
            &mut cs,
        )?,
    ));
    let begin_schedule_source_ordinal = select_hash_schema_value(
        cs.namespace(|| "begin_schedule_source_ordinal"),
        &control.schema_selectors,
        [
            &z[HASH_SCHEDULE_SOURCE_ORDINAL_CELL],
            &control.source_ordinal,
        ],
    )?;
    cells.push((
        HASH_SCHEDULE_SOURCE_ORDINAL_CELL,
        source_schedule_value(
            "schedule_source_ordinal_output",
            HASH_SCHEDULE_SOURCE_ORDINAL_CELL,
            &z[HASH_SCHEDULE_SOURCE_ORDINAL_CELL],
            &control.source_ordinal,
            &begin_schedule_source_ordinal,
            &z[HASH_SCHEDULE_SOURCE_ORDINAL_CELL],
            &zero,
            &mut cs,
        )?,
    ));
    for (index, (state_index, source_value, begin_value, block_value)) in [
        (
            HASH_SCHEDULE_MESSAGE_BYTES_CELL,
            &zero,
            &control.message_bytes,
            &z[HASH_SCHEDULE_MESSAGE_BYTES_CELL],
        ),
        (
            HASH_SCHEDULE_BLOCK_COUNT_CELL,
            &zero,
            &control.block_count,
            &z[HASH_SCHEDULE_BLOCK_COUNT_CELL],
        ),
        (
            HASH_SCHEDULE_NEXT_BLOCK_INDEX_CELL,
            &zero,
            &zero,
            &block_index_next,
        ),
        (
            HASH_SCHEDULE_FINAL_BLOCK_CELL,
            &zero,
            &zero,
            &control.final_block,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        cells.push((
            state_index,
            source_schedule_value(
                &format!("schedule_scalar_output_{index}"),
                state_index,
                &z[state_index],
                source_value,
                begin_value,
                block_value,
                &zero,
                &mut cs,
            )?,
        ));
    }
    for (index, hash_limb) in control.source_hash_limbs.iter().enumerate() {
        let state_index = HASH_SCHEDULE_SOURCE_HASH_START + index;
        let begin_hash = select_hash_schema_value(
            cs.namespace(|| format!("begin_source_hash_{index}")),
            &control.schema_selectors,
            [&z[state_index], hash_limb],
        )?;
        cells.push((
            state_index,
            source_schedule_value(
                &format!("schedule_source_hash_output_{index}"),
                state_index,
                &z[state_index],
                hash_limb,
                &begin_hash,
                &z[state_index],
                &zero,
                &mut cs,
            )?,
        ));
    }
    let begin_schedule_role = select_hash_schema_value(
        cs.namespace(|| "begin_schedule_role"),
        &control.schema_selectors,
        [&z[HASH_SCHEDULE_ROLE_CELL], &control.role],
    )?;
    cells.push((
        HASH_SCHEDULE_ROLE_CELL,
        source_schedule_value(
            "schedule_role_output",
            HASH_SCHEDULE_ROLE_CELL,
            &z[HASH_SCHEDULE_ROLE_CELL],
            &control.role,
            &begin_schedule_role,
            &z[HASH_SCHEDULE_ROLE_CELL],
            &zero,
            &mut cs,
        )?,
    ));

    // Each byte context has exactly one successor per state cell.  The rows
    // below build the base successor and later layers (record-length append,
    // TraceChunk append, or compression) replace only their own cells while
    // retaining that base.  Pushing independently-selected duplicates into
    // `cells` would make the final inactive layer overwrite BEGIN_HASH's IV
    // and queue initialization.
    let mut source_context_values =
        z[SOURCE_BYTE_CONTEXT_START..SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_WIDTH].to_vec();
    let mut global_context_values =
        z[GLOBAL_BYTE_CONTEXT_START..GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_WIDTH].to_vec();

    // The source header is promoted into the fixed source context only on the
    // source-record row. A source BEGIN_HASH retains only the static-frame tail
    // after its two cursor-derived compression blocks, then marks the context
    // started. Trace-wide controls preserve the source context.
    for offset in 0..BYTE_CONTEXT_WIDTH {
        let state_index = SOURCE_BYTE_CONTEXT_START + offset;
        let source_value = if offset == BYTE_CONTEXT_ACTIVE_OFFSET {
            &one
        } else if offset == BYTE_CONTEXT_SOURCE_ORDINAL_OFFSET {
            &control.source_ordinal
        } else if offset == BYTE_CONTEXT_CANONICAL_BYTES_OFFSET {
            &control.source_record_bytes
        } else if (BYTE_CONTEXT_HEADER_START_OFFSET..BYTE_CONTEXT_HEADER_END_OFFSET)
            .contains(&offset)
        {
            &source_header[offset - BYTE_CONTEXT_HEADER_START_OFFSET]
        } else {
            &zero
        };
        let source_begin_value = if matches!(
            offset,
            BYTE_CONTEXT_STARTED_OFFSET | BYTE_CONTEXT_PENDING_OFFSET
        ) {
            &one
        } else if offset == BYTE_CONTEXT_MESSAGE_BYTES_OFFSET {
            &control.message_bytes
        } else if offset == BYTE_CONTEXT_BUFFER_LEN_OFFSET {
            &source_static_tail_len_value
        } else if offset == BYTE_CONTEXT_BLOCK_COUNT_OFFSET {
            &control.block_count
        } else if offset == BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET {
            &zero
        } else if (BYTE_CONTEXT_CHAINING_START_OFFSET..BYTE_CONTEXT_CHAINING_END_OFFSET)
            .contains(&offset)
        {
            &iv[offset - BYTE_CONTEXT_CHAINING_START_OFFSET]
        } else if (BYTE_CONTEXT_BUFFER_START_OFFSET..BYTE_CONTEXT_BUFFER_END_OFFSET)
            .contains(&offset)
        {
            source_static_frame
                .get(source_static_blocks + offset - BYTE_CONTEXT_BUFFER_START_OFFSET)
                .unwrap_or(&zero)
        } else {
            &z[state_index]
        };
        let begin_value = select_hash_schema_value(
            cs.namespace(|| format!("source_context_begin_{offset}")),
            &control.schema_selectors,
            [source_begin_value, &z[state_index]],
        )?;
        let end_value = select_hash_schema_value(
            cs.namespace(|| format!("source_context_end_{offset}")),
            &control.schema_selectors,
            [&zero, &z[state_index]],
        )?;
        source_context_values[offset] = stage_value(
            &format!("source_context_output_{offset}"),
            &z[state_index],
            source_value,
            &begin_value,
            &z[state_index],
            &end_value,
            &mut cs,
        )?;
    }

    // TracePrecommit BEGIN owns the second, global O(1) context. It carries
    // only the canonical role prefix plus precommit counters at this stage;
    // source length prefixes and shared chunk bytes are intentionally not
    // admitted until the queue-to-block relation is installed atomically.
    for offset in 0..BYTE_CONTEXT_WIDTH {
        let state_index = GLOBAL_BYTE_CONTEXT_START + offset;
        let trace_begin_value = if offset == BYTE_CONTEXT_ACTIVE_OFFSET
            || offset == BYTE_CONTEXT_STARTED_OFFSET
            || offset == BYTE_CONTEXT_PENDING_OFFSET
        {
            &one
        } else if offset == BYTE_CONTEXT_CANONICAL_BYTES_OFFSET {
            &control.trace_byte_count
        } else if offset == BYTE_CONTEXT_SOURCE_ORDINAL_OFFSET {
            &control.source_ordinal
        } else if offset == BYTE_CONTEXT_MESSAGE_BYTES_OFFSET {
            &control.message_bytes
        } else if offset == BYTE_CONTEXT_CHUNK_COUNT_OFFSET {
            &control.trace_event_count
        } else if offset == BYTE_CONTEXT_BLOCK_COUNT_OFFSET {
            &control.block_count
        } else if offset == BYTE_CONTEXT_NEXT_BLOCK_INDEX_OFFSET {
            &zero
        } else if offset == BYTE_CONTEXT_BUFFER_LEN_OFFSET {
            &trace_role_prefix_tail_len_value
        } else if (BYTE_CONTEXT_CHAINING_START_OFFSET..BYTE_CONTEXT_CHAINING_END_OFFSET)
            .contains(&offset)
        {
            &iv[offset - BYTE_CONTEXT_CHAINING_START_OFFSET]
        } else if (BYTE_CONTEXT_BUFFER_START_OFFSET..BYTE_CONTEXT_BUFFER_END_OFFSET)
            .contains(&offset)
        {
            trace_role_prefix
                .get(static_block_bytes + offset - BYTE_CONTEXT_BUFFER_START_OFFSET)
                .unwrap_or(&zero)
        } else {
            &zero
        };
        let begin_value = select_hash_schema_value(
            cs.namespace(|| format!("global_context_begin_{offset}")),
            &control.schema_selectors,
            [&z[state_index], trace_begin_value],
        )?;
        let end_value = select_hash_schema_value(
            cs.namespace(|| format!("global_context_end_{offset}")),
            &control.schema_selectors,
            [&z[state_index], &zero],
        )?;
        global_context_values[offset] = stage_value(
            &format!("global_context_output_{offset}"),
            &z[state_index],
            &z[state_index],
            &begin_value,
            &z[state_index],
            &end_value,
            &mut cs,
        )?;
    }

    // The old shared SHA lane cells remain frozen-zero transient padding in
    // the public state layout. Both live transcripts keep their own chaining
    // state and block cursor in the two byte contexts above; writing these
    // shared cells would let interleaved source/global blocks corrupt one
    // another's cursor.

    // A source record contributes its declared canonical length to the one
    // whole-trace transcript before any of that record's shared chunks arrive.
    // The source record itself cannot choose the prefix bytes: they are the
    // little-endian decomposition of the already constrained record length.
    let source_record_length_bytes = decompose_le_bytes(
        cs.namespace(|| "global_source_record_length"),
        &control.source_record_bytes,
        8,
        "global_source_record_length",
    )?;
    let source_length_width = allocate_constant(cs.namespace(|| "global_source_length_width"), 8)?;
    let global_length_append = append_context_bytes(
        cs.namespace(|| "global_source_length_append"),
        z,
        GLOBAL_BYTE_CONTEXT_START,
        source,
        &source_record_length_bytes,
        &source_length_width,
        "global_source_length",
    )?;
    for (state_index, update) in global_length_append {
        let offset = state_index - GLOBAL_BYTE_CONTEXT_START;
        let prior = global_context_values[offset].clone();
        let selected = select_bit_num(
            cs.namespace(|| format!("global_source_length_output_{state_index}")),
            source,
            &update,
            &prior,
            "value",
        )?;
        global_context_values[offset] = selected;
    }
    for (name, state_index) in [
        (
            "global_active",
            GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_ACTIVE_OFFSET,
        ),
        (
            "global_started",
            GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_STARTED_OFFSET,
        ),
    ] {
        enforce_gated_constant(
            cs.namespace(|| format!("source_{name}")),
            source,
            &z[state_index],
            1,
        );
    }
    enforce_gated_constant(
        cs.namespace(|| "source_global_not_eof"),
        source,
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_EOF_OFFSET],
        0,
    );

    // A live TraceChunk has one explicit relation: append its zero-tail-checked
    // bytes to both source and whole-trace contexts, advance the source's
    // ordinal/count cursor, and seal each context exactly at its own EOF.
    let source_active = &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_ACTIVE_OFFSET];
    let source_started = &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_STARTED_OFFSET];
    let source_eof = &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_EOF_OFFSET];
    let source_pending = &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_PENDING_OFFSET];
    let global_active = &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_ACTIVE_OFFSET];
    let global_started = &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_STARTED_OFFSET];
    let global_pending = &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_PENDING_OFFSET];
    let global_eof = &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_EOF_OFFSET];
    for (name, value, expected) in [
        ("source_active", source_active, 1_u64),
        ("source_started", source_started, 1_u64),
        ("source_eof", source_eof, 0_u64),
        ("source_pending", source_pending, 0_u64),
        ("global_active", global_active, 1_u64),
        ("global_started", global_started, 1_u64),
        ("global_pending", global_pending, 0_u64),
        ("global_eof", global_eof, 0_u64),
    ] {
        enforce_gated_constant(
            cs.namespace(|| format!("trace_chunk_{name}")),
            trace_chunk_selector,
            value,
            expected,
        );
    }
    enforce_gated_equal(
        cs.namespace(|| "trace_chunk_source_ordinal"),
        trace_chunk_selector,
        &trace_chunk.source_ordinal,
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_SOURCE_ORDINAL_OFFSET],
    );
    enforce_gated_equal(
        cs.namespace(|| "trace_chunk_ordinal"),
        trace_chunk_selector,
        &trace_chunk.chunk_ordinal,
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_NEXT_CHUNK_OFFSET],
    );
    let source_next_chunk = allocate_incremented(
        cs.namespace(|| "trace_chunk_next_ordinal"),
        &trace_chunk.chunk_ordinal,
        "trace_chunk_next_ordinal_value",
    )?;
    let source_first = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "trace_chunk_is_first"),
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_NEXT_CHUNK_OFFSET],
        &zero,
    )?;
    let source_not_first = allocate_bit_not(
        cs.namespace(|| "trace_chunk_not_first"),
        &source_first,
        "value",
    )?;
    let source_first_gate = AllocatedBit::and(
        cs.namespace(|| "trace_chunk_first_gate"),
        trace_chunk_selector,
        &source_first,
    )?;
    let source_not_first_gate = AllocatedBit::and(
        cs.namespace(|| "trace_chunk_not_first_gate"),
        trace_chunk_selector,
        &source_not_first,
    )?;
    enforce_gated_constant(
        cs.namespace(|| "trace_chunk_first_count_zero"),
        &source_first_gate,
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CHUNK_COUNT_OFFSET],
        0,
    );
    enforce_gated_equal(
        cs.namespace(|| "trace_chunk_repeated_count"),
        &source_not_first_gate,
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CHUNK_COUNT_OFFSET],
        &trace_chunk.chunk_count,
    );
    let source_final = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "trace_chunk_is_final"),
        &source_next_chunk,
        &trace_chunk.chunk_count,
    )?;
    let source_not_final = allocate_bit_not(
        cs.namespace(|| "trace_chunk_not_final"),
        &source_final,
        "value",
    )?;
    let source_final_gate = AllocatedBit::and(
        cs.namespace(|| "trace_chunk_final_gate"),
        trace_chunk_selector,
        &source_final,
    )?;
    let source_not_final_gate = AllocatedBit::and(
        cs.namespace(|| "trace_chunk_not_final_gate"),
        trace_chunk_selector,
        &source_not_final,
    )?;
    enforce_gated_constant(
        cs.namespace(|| "trace_chunk_nonfinal_width"),
        &source_not_final_gate,
        &trace_chunk.byte_count,
        TRACE_CANONICAL_CHUNK_BYTES_V2 as u64,
    );
    for index in 0..TRACE_EVENT_HEADER_BYTES_V2 {
        enforce_gated_equal(
            cs.namespace(|| format!("trace_chunk_header_byte_{index}")),
            &source_first_gate,
            &trace_chunk.bytes[index],
            &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_HEADER_START_OFFSET + index],
        );
    }
    let source_consumed =
        AllocatedNum::alloc(cs.namespace(|| "trace_chunk_source_consumed"), || {
            let prior = scalar_u64(
                z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CONSUMED_BYTES_OFFSET]
                    .get_value()
                    .ok_or(SynthesisError::AssignmentMissing)?,
            );
            let count = scalar_u64(
                trace_chunk
                    .byte_count
                    .get_value()
                    .ok_or(SynthesisError::AssignmentMissing)?,
            );
            prior.checked_add(count).map(Scalar::from).ok_or_else(|| {
                SynthesisError::Unsatisfiable("source byte count overflow".to_owned())
            })
        })?;
    enforce_sum(
        cs.namespace(|| "trace_chunk_source_consumed_relation"),
        &source_consumed,
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CONSUMED_BYTES_OFFSET],
        &trace_chunk.byte_count,
    );
    enforce_gated_equal(
        cs.namespace(|| "trace_chunk_source_final_length"),
        &source_final_gate,
        &source_consumed,
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CANONICAL_BYTES_OFFSET],
    );
    let global_consumed =
        AllocatedNum::alloc(cs.namespace(|| "trace_chunk_global_consumed"), || {
            let prior = scalar_u64(
                z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_CONSUMED_BYTES_OFFSET]
                    .get_value()
                    .ok_or(SynthesisError::AssignmentMissing)?,
            );
            let count = scalar_u64(
                trace_chunk
                    .byte_count
                    .get_value()
                    .ok_or(SynthesisError::AssignmentMissing)?,
            );
            prior.checked_add(count).map(Scalar::from).ok_or_else(|| {
                SynthesisError::Unsatisfiable("global byte count overflow".to_owned())
            })
        })?;
    enforce_sum(
        cs.namespace(|| "trace_chunk_global_consumed_relation"),
        &global_consumed,
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_CONSUMED_BYTES_OFFSET],
        &trace_chunk.byte_count,
    );
    let global_final = nova_snark::gadgets::utils::alloc_num_equals(
        cs.namespace(|| "trace_chunk_is_global_final"),
        &global_consumed,
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_CANONICAL_BYTES_OFFSET],
    )?;
    let source_chunk_count = select_bit_num(
        cs.namespace(|| "trace_chunk_source_count_output"),
        &source_first,
        &trace_chunk.chunk_count,
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CHUNK_COUNT_OFFSET],
        "value",
    )?;
    let source_eof_output = bit_as_num(
        cs.namespace(|| "trace_chunk_source_eof_output"),
        &source_final,
        "value",
    )?;
    let global_eof_output = bit_as_num(
        cs.namespace(|| "trace_chunk_global_eof_output"),
        &global_final,
        "value",
    )?;
    let source_append = append_context_bytes(
        cs.namespace(|| "trace_chunk_source_append"),
        z,
        SOURCE_BYTE_CONTEXT_START,
        trace_chunk_selector,
        &trace_chunk.bytes,
        &trace_chunk.byte_count,
        "trace_chunk_source",
    )?;
    let global_append = append_context_bytes(
        cs.namespace(|| "trace_chunk_global_append"),
        z,
        GLOBAL_BYTE_CONTEXT_START,
        trace_chunk_selector,
        &trace_chunk.bytes,
        &trace_chunk.byte_count,
        "trace_chunk_global",
    )?;
    for (state_index, update) in source_append.into_iter().chain(global_append) {
        let prior = if state_index < GLOBAL_BYTE_CONTEXT_START {
            source_context_values[state_index - SOURCE_BYTE_CONTEXT_START].clone()
        } else {
            global_context_values[state_index - GLOBAL_BYTE_CONTEXT_START].clone()
        };
        let selected = select_bit_num(
            cs.namespace(|| format!("trace_chunk_buffer_output_{state_index}")),
            trace_chunk_selector,
            &update,
            &prior,
            "value",
        )?;
        if state_index < GLOBAL_BYTE_CONTEXT_START {
            source_context_values[state_index - SOURCE_BYTE_CONTEXT_START] = selected;
        } else {
            global_context_values[state_index - GLOBAL_BYTE_CONTEXT_START] = selected;
        }
    }
    for (state_index, update) in [
        (
            SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CONSUMED_BYTES_OFFSET,
            source_consumed,
        ),
        (
            SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_NEXT_CHUNK_OFFSET,
            source_next_chunk,
        ),
        (
            SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_CHUNK_COUNT_OFFSET,
            source_chunk_count,
        ),
        (
            SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_EOF_OFFSET,
            source_eof_output,
        ),
        (
            GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_CONSUMED_BYTES_OFFSET,
            global_consumed,
        ),
        (
            GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_EOF_OFFSET,
            global_eof_output,
        ),
    ] {
        let prior = if state_index < GLOBAL_BYTE_CONTEXT_START {
            source_context_values[state_index - SOURCE_BYTE_CONTEXT_START].clone()
        } else {
            global_context_values[state_index - GLOBAL_BYTE_CONTEXT_START].clone()
        };
        let selected = select_bit_num(
            cs.namespace(|| format!("trace_chunk_state_output_{state_index}")),
            trace_chunk_selector,
            &update,
            &prior,
            "value",
        )?;
        if state_index < GLOBAL_BYTE_CONTEXT_START {
            source_context_values[state_index - SOURCE_BYTE_CONTEXT_START] = selected;
        } else {
            global_context_values[state_index - GLOBAL_BYTE_CONTEXT_START] = selected;
        }
    }

    // The selected SHA control consumes bytes only from its matching context.
    // This equality is the byte-to-compression bridge; no host-provided block
    // or native SHA result can bypass it.
    let source_pending_bit = allocate_state_bit(
        cs.namespace(|| "source_static_pending"),
        &z[SOURCE_BYTE_CONTEXT_START + BYTE_CONTEXT_PENDING_OFFSET],
    )?;
    let source_not_pending = allocate_bit_not(
        cs.namespace(|| "source_not_static_pending"),
        &source_pending_bit,
        "value",
    )?;
    let source_static_block = AllocatedBit::and(
        cs.namespace(|| "source_static_block"),
        &source_block,
        &source_pending_bit,
    )?;
    let source_dynamic_block = AllocatedBit::and(
        cs.namespace(|| "source_dynamic_block"),
        &source_block,
        &source_not_pending,
    )?;
    let global_pending_bit = allocate_state_bit(
        cs.namespace(|| "global_static_pending"),
        &z[GLOBAL_BYTE_CONTEXT_START + BYTE_CONTEXT_PENDING_OFFSET],
    )?;
    let global_not_pending = allocate_bit_not(
        cs.namespace(|| "global_not_static_pending"),
        &global_pending_bit,
        "value",
    )?;
    let trace_static_block = AllocatedBit::and(
        cs.namespace(|| "trace_static_block"),
        &trace_block,
        &global_pending_bit,
    )?;
    let trace_dynamic_block = AllocatedBit::and(
        cs.namespace(|| "trace_dynamic_block"),
        &trace_block,
        &global_not_pending,
    )?;
    let source_static_blocks = [
        &source_static_frame[..static_block_bytes],
        &source_static_frame[static_block_bytes..source_static_blocks],
    ];
    let trace_static_blocks = [&trace_role_prefix[..static_block_bytes]];
    let source_static_context = consume_static_context_block(
        cs.namespace(|| "source_static_block_context"),
        z,
        SOURCE_BYTE_CONTEXT_START,
        &source_static_block,
        control,
        sha,
        &source_static_blocks,
        "source_static_block",
    )?;
    let trace_static_context = consume_static_context_block(
        cs.namespace(|| "trace_static_block_context"),
        z,
        GLOBAL_BYTE_CONTEXT_START,
        &trace_static_block,
        control,
        sha,
        &trace_static_blocks,
        "trace_static_block",
    )?;
    let source_block_context = consume_context_block(
        cs.namespace(|| "source_block_context"),
        z,
        SOURCE_BYTE_CONTEXT_START,
        &source_dynamic_block,
        false,
        control,
        sha,
        "source_block",
    )?;
    let trace_block_context = consume_context_block(
        cs.namespace(|| "trace_block_context"),
        z,
        GLOBAL_BYTE_CONTEXT_START,
        &trace_dynamic_block,
        false,
        control,
        sha,
        "trace_block",
    )?;
    for (state_index, update) in source_static_context {
        let offset = state_index - SOURCE_BYTE_CONTEXT_START;
        let prior = source_context_values[offset].clone();
        let selected = select_bit_num(
            cs.namespace(|| format!("source_static_context_output_{state_index}")),
            &source_static_block,
            &update,
            &prior,
            "value",
        )?;
        source_context_values[offset] = selected;
    }
    for (state_index, update) in trace_static_context {
        let offset = state_index - GLOBAL_BYTE_CONTEXT_START;
        let prior = global_context_values[offset].clone();
        let selected = select_bit_num(
            cs.namespace(|| format!("trace_static_context_output_{state_index}")),
            &trace_static_block,
            &update,
            &prior,
            "value",
        )?;
        global_context_values[offset] = selected;
    }
    for (state_index, update) in source_block_context {
        let offset = state_index - SOURCE_BYTE_CONTEXT_START;
        let prior = source_context_values[offset].clone();
        let selected = select_bit_num(
            cs.namespace(|| format!("source_block_context_output_{state_index}")),
            &source_dynamic_block,
            &update,
            &prior,
            "value",
        )?;
        source_context_values[offset] = selected;
    }
    for (state_index, update) in trace_block_context {
        let offset = state_index - GLOBAL_BYTE_CONTEXT_START;
        let prior = global_context_values[offset].clone();
        let selected = select_bit_num(
            cs.namespace(|| format!("trace_block_context_output_{state_index}")),
            &trace_dynamic_block,
            &update,
            &prior,
            "value",
        )?;
        global_context_values[offset] = selected;
    }

    cells.extend(
        source_context_values
            .into_iter()
            .enumerate()
            .map(|(offset, value)| (SOURCE_BYTE_CONTEXT_START + offset, value)),
    );
    cells.extend(
        global_context_values
            .into_iter()
            .enumerate()
            .map(|(offset, value)| (GLOBAL_BYTE_CONTEXT_START + offset, value)),
    );

    Ok(HashScheduleOutputsV2 {
        source_trace_ordinal: select_hash_stage_value(
            cs.namespace(|| "source_trace_ordinal_output"),
            selectors,
            [
                &z[SOURCE_TRACE_ORDINAL_CELL],
                &source_trace_next,
                &z[SOURCE_TRACE_ORDINAL_CELL],
                &z[SOURCE_TRACE_ORDINAL_CELL],
                &z[SOURCE_TRACE_ORDINAL_CELL],
            ],
        )?,
        source_trace_byte_count: select_hash_stage_value(
            cs.namespace(|| "source_trace_byte_count_output"),
            selectors,
            [
                &z[SOURCE_TRACE_BYTE_COUNT_CELL],
                &source_trace_byte_count_next,
                &z[SOURCE_TRACE_BYTE_COUNT_CELL],
                &z[SOURCE_TRACE_BYTE_COUNT_CELL],
                &z[SOURCE_TRACE_BYTE_COUNT_CELL],
            ],
        )?,
        cells,
    })
}

fn enforce_inactive_zero<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    active: &AllocatedBit,
    value: &AllocatedNum<Scalar>,
) {
    cs.enforce(
        || "inactive lane value is zero",
        |lc| lc + CS::one() - active.get_variable(),
        |lc| lc + value.get_variable(),
        |lc| lc,
    );
}

fn enforce_gated_equal<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    selector: &AllocatedBit,
    left: &AllocatedNum<Scalar>,
    right: &AllocatedNum<Scalar>,
) {
    cs.enforce(
        || "selected values equal",
        |lc| lc + selector.get_variable(),
        |lc| lc + left.get_variable() - right.get_variable(),
        |lc| lc,
    );
}

fn enforce_gated_constant<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    selector: &AllocatedBit,
    value: &AllocatedNum<Scalar>,
    expected: u64,
) {
    cs.enforce(
        || "selected value equals constant",
        |lc| lc + selector.get_variable(),
        |lc| lc + value.get_variable() - (Scalar::from(expected), CS::one()),
        |lc| lc,
    );
}

fn enforce_gated_lc_equal<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    selector: &AllocatedBit,
    left: &nova_snark::frontend::LinearCombination<Scalar>,
    right: &AllocatedNum<Scalar>,
) {
    cs.enforce(
        || "selected linear combination equals value",
        |lc| lc + selector.get_variable(),
        |lc| lc + left - right.get_variable(),
        |lc| lc,
    );
}

/// Check the frozen whole-trace SHA grammar for every tagged Trace control.
/// The transcript is role-framed as one length-prefixed canonical source
/// record per source event; padding and the terminal bit length are algebraic
/// schedule values, not a native validity signal.
fn enforce_gated_trace_geometry<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    selector: &AllocatedBit,
    control: &AllocatedHashControlV2,
) -> Result<(), SynthesisError> {
    let trace_domain_bytes =
        CheckpointSha256BlockStreamV2::framed_bytes_for_parts(CheckpointShaRole::Trace, 0, 0)
            .expect("fixed Trace SHA framing fits");
    cs.enforce(
        || "trace_eof_is_sealed",
        |lc| lc + selector.get_variable(),
        |lc| lc + control.trace_eof.get_variable() - CS::one(),
        |lc| lc,
    );
    cs.enforce(
        || "trace_role_framed_message_bytes",
        |lc| lc + selector.get_variable(),
        |lc| {
            lc + control.message_bytes.get_variable()
                - control.trace_byte_count.get_variable()
                - (
                    Scalar::from(8_u64),
                    control.trace_event_count.get_variable(),
                )
                - (Scalar::from(trace_domain_bytes), CS::one())
        },
        |lc| lc,
    );
    cs.enforce(
        || "trace_bit_length",
        |lc| lc + selector.get_variable(),
        |lc| {
            lc + control.trace_bit_length.get_variable()
                - (Scalar::from(8_u64), control.message_bytes.get_variable())
        },
        |lc| lc,
    );
    cs.enforce(
        || "trace_fips_padding_geometry",
        |lc| lc + selector.get_variable(),
        |lc| {
            lc + (Scalar::from(64_u64), control.block_count.get_variable())
                - control.message_bytes.get_variable()
                - control.trace_padding_bytes.get_variable()
                - (Scalar::from(9_u64), CS::one())
        },
        |lc| lc,
    );
    Ok(())
}

/// Select one of the five fixed hash-schedule successors.  The selector set
/// is one-hot, so these gated equalities define one output without any
/// witness-side validity branch.
fn select_hash_stage_value<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    selectors: &[AllocatedBit],
    candidates: [&AllocatedNum<Scalar>; 5],
) -> Result<AllocatedNum<Scalar>, SynthesisError> {
    if selectors.len() != candidates.len() {
        return Err(SynthesisError::Unsatisfiable(
            "hash-stage selector arity mismatch".to_owned(),
        ));
    }
    let output = AllocatedNum::alloc(cs.namespace(|| "hash_stage_output"), || {
        for (selector, candidate) in selectors.iter().zip(candidates) {
            if selector.get_value() == Some(true) {
                return candidate
                    .get_value()
                    .ok_or(SynthesisError::AssignmentMissing);
            }
        }
        Err(SynthesisError::AssignmentMissing)
    })?;
    for (index, (selector, candidate)) in selectors.iter().zip(candidates).enumerate() {
        enforce_gated_equal(
            cs.namespace(|| format!("hash_stage_candidate_{index}")),
            selector,
            &output,
            candidate,
        );
    }
    Ok(output)
}

/// Select a value from the frozen two-schema hash-control grammar.
fn select_hash_schema_value<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    selectors: &[AllocatedBit],
    candidates: [&AllocatedNum<Scalar>; 2],
) -> Result<AllocatedNum<Scalar>, SynthesisError> {
    if selectors.len() != candidates.len() {
        return Err(SynthesisError::Unsatisfiable(
            "hash-schema selector arity mismatch".to_owned(),
        ));
    }
    let output = AllocatedNum::alloc(cs.namespace(|| "hash_schema_output"), || {
        for (selector, candidate) in selectors.iter().zip(candidates) {
            if selector.get_value() == Some(true) {
                return candidate
                    .get_value()
                    .ok_or(SynthesisError::AssignmentMissing);
            }
        }
        Err(SynthesisError::AssignmentMissing)
    })?;
    for (index, (selector, candidate)) in selectors.iter().zip(candidates).enumerate() {
        cs.enforce(
            || format!("hash_schema_output_{index}"),
            |lc| lc + selector.get_variable(),
            |lc| lc + output.get_variable() - candidate.get_variable(),
            |lc| lc,
        );
    }
    Ok(output)
}

fn allocate_incremented<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    value: &AllocatedNum<Scalar>,
    label: &str,
) -> Result<AllocatedNum<Scalar>, SynthesisError> {
    let next = AllocatedNum::alloc(cs.namespace(|| label), || {
        scalar_u64(value.get_value().ok_or(SynthesisError::AssignmentMissing)?)
            .checked_add(1)
            .map(Scalar::from)
            .ok_or_else(|| SynthesisError::Unsatisfiable("ordinal overflow".to_owned()))
    })?;
    enforce_increment(cs.namespace(|| "increment_relation"), value, &next);
    Ok(next)
}

fn enforce_gated_hash_control_ordinal<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    selector: &AllocatedBit,
    event_ordinal: &AllocatedNum<Scalar>,
    source_ordinal: &AllocatedNum<Scalar>,
    sequence: &AllocatedNum<Scalar>,
    sequence_plus_one: bool,
) {
    cs.enforce(
        || "canonical hash-control ordinal",
        |lc| lc + selector.get_variable(),
        |lc| {
            let mut relation = lc + event_ordinal.get_variable()
                - (Scalar::from(HASH_CONTROL_ORDINAL_FLAG_V2), CS::one())
                - (
                    Scalar::from(HASH_CONTROL_ORDINAL_STRIDE_V2),
                    source_ordinal.get_variable(),
                )
                - sequence.get_variable();
            if sequence_plus_one {
                relation = relation - CS::one();
            }
            relation
        },
        |lc| lc,
    );
}

fn source_hash_word_lc<CS: ConstraintSystem<Scalar>>(
    hash_bytes: &[AllocatedNum<Scalar>],
    word_index: usize,
) -> nova_snark::frontend::LinearCombination<Scalar> {
    let start = word_index * 4;
    nova_snark::frontend::LinearCombination::zero()
        + (Scalar::from(1_u64 << 24), hash_bytes[start].get_variable())
        + (
            Scalar::from(1_u64 << 16),
            hash_bytes[start + 1].get_variable(),
        )
        + (
            Scalar::from(1_u64 << 8),
            hash_bytes[start + 2].get_variable(),
        )
        + hash_bytes[start + 3].get_variable()
}

fn allocated_num_bits_be<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    value: &AllocatedNum<Scalar>,
    bits: usize,
) -> Result<Vec<Boolean>, SynthesisError> {
    let raw = value.get_value().map(scalar_u64);
    let mut allocated = Vec::with_capacity(bits);
    for index in 0..bits {
        allocated.push(AllocatedBit::alloc(
            cs.namespace(|| format!("bit_{index}")),
            raw.map(|current| ((current >> index) & 1) == 1),
        )?);
    }
    cs.enforce(
        || "bit reconstruction",
        |lc| {
            let mut result = lc - value.get_variable();
            let mut coefficient = Scalar::from(1_u64);
            for bit in &allocated {
                result = result + (coefficient, bit.get_variable());
                coefficient = coefficient.double();
            }
            result
        },
        |lc| lc + CS::one(),
        |lc| lc,
    );
    Ok(allocated.into_iter().rev().map(Boolean::from).collect())
}

fn allocated_num_to_uint32<CS: ConstraintSystem<Scalar>>(
    cs: CS,
    value: &AllocatedNum<Scalar>,
) -> Result<UInt32, SynthesisError> {
    let bits = allocated_num_bits_be(cs, value, 32)?;
    Ok(UInt32::from_bits_be(&bits))
}

fn uint32_lc<CS: ConstraintSystem<Scalar>>(
    word: &UInt32,
) -> nova_snark::frontend::LinearCombination<Scalar> {
    let mut result = nova_snark::frontend::LinearCombination::zero();
    for (index, bit) in word.clone().into_bits_be().into_iter().enumerate() {
        result = result + &bit.lc(CS::one(), Scalar::from(1_u64 << (31 - index)));
    }
    result
}

fn transition_table_digest() -> [u8; 32] {
    use z00z_crypto::{sha256_256_role, CheckpointShaRole};

    let mut bytes = Vec::with_capacity(CONTROL_TRANSITION_TABLE_V2.len() * 5);
    for edge in CONTROL_TRANSITION_TABLE_V2 {
        bytes.extend_from_slice(&[
            edge.input_phase as u8,
            u8::from(edge.input_done),
            edge.opcode as u8,
            edge.next_phase as u8,
            u8::from(edge.next_done),
        ]);
    }
    sha256_256_role(
        CheckpointShaRole::Statement,
        &[b"z00z.recursive.v2.nova-control-table", &bytes],
    )
}

fn phase_selectors<CS: ConstraintSystem<Scalar>>(
    cs: &mut CS,
    phase: &AllocatedNum<Scalar>,
) -> Result<Vec<AllocatedBit>, SynthesisError> {
    let phase_value = phase.get_value().map(scalar_u64);
    let mut selectors = Vec::with_capacity(ControlPhaseV2::ALL.len());
    for value in ControlPhaseV2::ALL {
        selectors.push(AllocatedBit::alloc(
            cs.namespace(|| format!("phase_selector_{}", value as u64)),
            phase_value.map(|current| current == value as u64),
        )?);
    }
    enforce_selector_value(
        cs,
        phase,
        &selectors,
        |index| ControlPhaseV2::ALL[index] as u64,
        "phase",
    );
    Ok(selectors)
}

fn opcode_selectors<CS: ConstraintSystem<Scalar>>(
    cs: &mut CS,
    opcode: &AllocatedNum<Scalar>,
) -> Result<Vec<AllocatedBit>, SynthesisError> {
    let codes = opcode_list();
    let opcode_value = opcode.get_value().map(scalar_u64);
    let mut selectors = Vec::with_capacity(codes.len());
    for candidate in codes {
        selectors.push(AllocatedBit::alloc(
            cs.namespace(|| format!("opcode_selector_{}", candidate as u8)),
            opcode_value.map(|current| current == candidate as u8 as u64),
        )?);
    }
    enforce_selector_value(
        cs,
        opcode,
        &selectors,
        |index| opcode_list()[index] as u8 as u64,
        "opcode",
    );
    Ok(selectors)
}

fn enforce_selector_value<CS: ConstraintSystem<Scalar>>(
    cs: &mut CS,
    value: &AllocatedNum<Scalar>,
    selectors: &[AllocatedBit],
    code: impl Fn(usize) -> u64,
    selector_kind: &str,
) {
    enforce_one_hot(cs, selectors, &format!("{selector_kind}_one_hot"));
    cs.enforce(
        || format!("{selector_kind}_selector_value"),
        |lc| {
            let mut result = lc - value.get_variable();
            for (index, selector) in selectors.iter().enumerate() {
                result = result + (Scalar::from(code(index)), selector.get_variable());
            }
            result
        },
        |lc| lc + CS::one(),
        |lc| lc,
    );
}

fn enforce_one_hot<CS: ConstraintSystem<Scalar>>(
    cs: &mut CS,
    selectors: &[AllocatedBit],
    name: &str,
) {
    cs.enforce(
        || name,
        |lc| {
            let mut result = lc - CS::one();
            for selector in selectors {
                result = result + selector.get_variable();
            }
            result
        },
        |lc| lc + CS::one(),
        |lc| lc,
    );
}

fn add_weighted_bit(
    target: &mut Option<nova_snark::frontend::LinearCombination<Scalar>>,
    bit: &AllocatedBit,
    coefficient: Scalar,
) {
    let current = target
        .take()
        .unwrap_or_else(nova_snark::frontend::LinearCombination::zero);
    *target = Some(current + (coefficient, bit.get_variable()));
}

fn enforce_lc_equal<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    value: &AllocatedNum<Scalar>,
    relation: Option<nova_snark::frontend::LinearCombination<Scalar>>,
) -> Result<(), SynthesisError> {
    let relation = relation
        .ok_or_else(|| SynthesisError::Unsatisfiable("empty transition relation".to_owned()))?;
    cs.enforce(
        || "selected_transition_value",
        |lc| lc + value.get_variable() - &relation,
        |lc| lc + CS::one(),
        |lc| lc,
    );
    Ok(())
}

fn allocate_constant<CS: ConstraintSystem<Scalar>>(
    cs: CS,
    value: u64,
) -> Result<AllocatedNum<Scalar>, SynthesisError> {
    AllocatedNum::alloc(cs, || Ok(Scalar::from(value)))
}

fn enforce_constant<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    value: &AllocatedNum<Scalar>,
    expected: u16,
) {
    cs.enforce(
        || "constant value",
        |lc| lc + value.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc + (Scalar::from(u64::from(expected)), CS::one()),
    );
}

fn enforce_equal<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    left: &AllocatedNum<Scalar>,
    right: &AllocatedNum<Scalar>,
) {
    cs.enforce(
        || "numbers equal",
        |lc| lc + left.get_variable() - right.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc,
    );
}

fn enforce_increment<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    value: &AllocatedNum<Scalar>,
    next: &AllocatedNum<Scalar>,
) {
    cs.enforce(
        || "increment by one",
        |lc| lc + value.get_variable() + CS::one() - next.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc,
    );
}

fn enforce_gated_increment<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    selector: &AllocatedBit,
    value: &AllocatedNum<Scalar>,
    next: &AllocatedNum<Scalar>,
) {
    cs.enforce(
        || "selected increment by one",
        |lc| lc + selector.get_variable(),
        |lc| lc + value.get_variable() + CS::one() - next.get_variable(),
        |lc| lc,
    );
}

fn enforce_sum<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    sum: &AllocatedNum<Scalar>,
    left: &AllocatedNum<Scalar>,
    right: &AllocatedNum<Scalar>,
) {
    cs.enforce(
        || "numbers add",
        |lc| lc + sum.get_variable() - left.get_variable() - right.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc,
    );
}

fn allocate_state_bit<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    value: &AllocatedNum<Scalar>,
) -> Result<AllocatedBit, SynthesisError> {
    let bit = AllocatedBit::alloc(
        cs.namespace(|| "state_bit"),
        value.get_value().map(|current| scalar_u64(current) == 1),
    )?;
    cs.enforce(
        || "state_bit_matches_cell",
        |lc| lc + bit.get_variable() - value.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc,
    );
    Ok(bit)
}

fn range_bits<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    value: &AllocatedNum<Scalar>,
    bits: usize,
) -> Result<(), SynthesisError> {
    let raw = value.get_value().map(scalar_u64);
    let mut allocated = Vec::with_capacity(bits);
    for index in 0..bits {
        allocated.push(AllocatedBit::alloc(
            cs.namespace(|| format!("range_bit_{index}")),
            raw.map(|current| ((current >> index) & 1) == 1),
        )?);
    }
    cs.enforce(
        || "range reconstruction",
        |lc| {
            let mut result = lc - value.get_variable();
            let mut coefficient = Scalar::from(1_u64);
            for bit in &allocated {
                result = result + (coefficient, bit.get_variable());
                coefficient = coefficient.double();
            }
            result
        },
        |lc| lc + CS::one(),
        |lc| lc,
    );
    Ok(())
}

fn scalar_u64(value: Scalar) -> u64 {
    let repr = value.to_repr();
    let bytes = repr.as_ref();
    let mut low = [0_u8; 8];
    if let Some(prefix) = bytes.get(..low.len()) {
        low.copy_from_slice(prefix);
    }
    u64::from_le_bytes(low)
}

fn digest_limbs(digest: [u8; 32]) -> [u16; DIGEST_LIMBS] {
    let mut limbs = [0_u16; DIGEST_LIMBS];
    for (index, chunk) in digest.chunks_exact(2).enumerate() {
        limbs[index] = u16::from_le_bytes([chunk[0], chunk[1]]);
    }
    limbs
}

fn constrain_state_ranges<CS: ConstraintSystem<Scalar>>(
    cs: &mut CS,
    z: &[AllocatedNum<Scalar>],
) -> Result<(), SynthesisError> {
    for index in digest_cells() {
        range_bits(
            cs.namespace(|| format!("digest_limb_{index}")),
            &z[index],
            16,
        )?;
    }
    range_bits(
        cs.namespace(|| "hash_schedule_active"),
        &z[HASH_SCHEDULE_ACTIVE_CELL],
        1,
    )?;
    for index in [
        HASH_SCHEDULE_ORDINAL_CELL,
        HASH_SCHEDULE_SOURCE_ORDINAL_CELL,
        HASH_SCHEDULE_MESSAGE_BYTES_CELL,
        HASH_SCHEDULE_BLOCK_COUNT_CELL,
        HASH_SCHEDULE_NEXT_BLOCK_INDEX_CELL,
    ] {
        range_bits(
            cs.namespace(|| format!("hash_schedule_counter_{index}")),
            &z[index],
            64,
        )?;
    }
    range_bits(
        cs.namespace(|| "hash_schedule_final_block"),
        &z[HASH_SCHEDULE_FINAL_BLOCK_CELL],
        1,
    )?;
    for index in HASH_SCHEDULE_SOURCE_HASH_START..HASH_SCHEDULE_SOURCE_HASH_END {
        range_bits(
            cs.namespace(|| format!("hash_schedule_source_hash_{index}")),
            &z[index],
            16,
        )?;
    }
    for (label, index, bits) in [
        (
            "uniqueness_precommit_parser_active",
            PRECOMMIT_PARSE_ACTIVE_CELL,
            1,
        ),
        (
            "uniqueness_precommit_parser_header",
            PRECOMMIT_PARSE_HEADER_CELL,
            6,
        ),
        (
            "uniqueness_precommit_parser_offset",
            PRECOMMIT_PARSE_OFFSET_CELL,
            8,
        ),
        (
            "uniqueness_precommit_parser_low_byte",
            PRECOMMIT_PARSE_LOW_BYTE_CELL,
            8,
        ),
        (
            "uniqueness_challenge_parser_active",
            CHALLENGE_PARSE_ACTIVE_CELL,
            1,
        ),
        (
            "uniqueness_challenge_parser_header",
            CHALLENGE_PARSE_HEADER_CELL,
            6,
        ),
        (
            "uniqueness_challenge_parser_offset",
            CHALLENGE_PARSE_OFFSET_CELL,
            8,
        ),
        (
            "uniqueness_challenge_parser_low_byte",
            CHALLENGE_PARSE_LOW_BYTE_CELL,
            8,
        ),
        ("net_merge_parser_active", NET_PARSE_ACTIVE_CELL, 1),
        ("net_merge_parser_header", NET_PARSE_HEADER_CELL, 6),
        ("net_merge_parser_offset", NET_PARSE_OFFSET_CELL, 6),
        ("net_merge_parser_low_byte", NET_PARSE_LOW_BYTE_CELL, 8),
    ] {
        range_bits(cs.namespace(|| label), &z[index], bits)?;
    }
    range_bits(
        cs.namespace(|| "hash_schedule_role"),
        &z[HASH_SCHEDULE_ROLE_CELL],
        8,
    )?;
    for index in NET_DIGEST_LIMB_START..NET_DIGEST_LIMB_END {
        range_bits(
            cs.namespace(|| format!("net_digest_limb_{index}")),
            &z[index],
            16,
        )?;
    }
    range_bits(cs.namespace(|| "sha_active"), &z[SHA_ACTIVE_CELL], 1)?;
    range_bits(
        cs.namespace(|| "sha_block_ordinal"),
        &z[SHA_BLOCK_ORDINAL_CELL],
        64,
    )?;
    for index in SHA_CHAINING_START..SHA_CHAINING_END {
        range_bits(
            cs.namespace(|| format!("sha_chaining_{index}")),
            &z[index],
            32,
        )?;
    }
    for index in SHA_BLOCK_START..SHA_BLOCK_END {
        range_bits(
            cs.namespace(|| format!("sha_block_byte_{index}")),
            &z[index],
            8,
        )?;
    }
    constrain_byte_context_ranges(cs, z, SOURCE_BYTE_CONTEXT_START, "source")?;
    constrain_byte_context_ranges(cs, z, GLOBAL_BYTE_CONTEXT_START, "global")?;
    for index in EXPECTED_TRACE_ROOT_START..EXPECTED_TRACE_ROOT_END {
        range_bits(
            cs.namespace(|| format!("expected_trace_root_{index}")),
            &z[index],
            64,
        )?;
    }
    for index in EXPECTED_TRACE_DIGEST_START..EXPECTED_TRACE_DIGEST_END {
        range_bits(
            cs.namespace(|| format!("expected_trace_digest_{index}")),
            &z[index],
            16,
        )?;
    }
    Ok(())
}

fn constrain_byte_context_ranges<CS: ConstraintSystem<Scalar>>(
    cs: &mut CS,
    z: &[AllocatedNum<Scalar>],
    start: usize,
    label: &str,
) -> Result<(), SynthesisError> {
    for (name, offset, bits) in [
        ("active", BYTE_CONTEXT_ACTIVE_OFFSET, 1),
        ("source_ordinal", BYTE_CONTEXT_SOURCE_ORDINAL_OFFSET, 64),
        ("canonical_bytes", BYTE_CONTEXT_CANONICAL_BYTES_OFFSET, 64),
        ("message_bytes", BYTE_CONTEXT_MESSAGE_BYTES_OFFSET, 64),
        ("consumed_bytes", BYTE_CONTEXT_CONSUMED_BYTES_OFFSET, 64),
        ("next_chunk", BYTE_CONTEXT_NEXT_CHUNK_OFFSET, 32),
        ("chunk_count", BYTE_CONTEXT_CHUNK_COUNT_OFFSET, 32),
        ("buffer_len", BYTE_CONTEXT_BUFFER_LEN_OFFSET, 8),
        ("started", BYTE_CONTEXT_STARTED_OFFSET, 1),
        ("pending", BYTE_CONTEXT_PENDING_OFFSET, 1),
        ("eof", BYTE_CONTEXT_EOF_OFFSET, 1),
        ("padding_blocks", BYTE_CONTEXT_PADDING_BLOCKS_OFFSET, 2),
    ] {
        range_bits(
            cs.namespace(|| format!("{label}_byte_context_{name}")),
            &z[start + offset],
            bits,
        )?;
    }
    constrain_bounded_u8(
        cs.namespace(|| format!("{label}_byte_context_buffer_capacity")),
        &z[start + BYTE_CONTEXT_BUFFER_LEN_OFFSET],
        BYTE_CONTEXT_BUFFER_BYTES,
    )?;
    for index in BYTE_CONTEXT_CHAINING_START_OFFSET..BYTE_CONTEXT_CHAINING_END_OFFSET {
        range_bits(
            cs.namespace(|| format!("{label}_byte_context_chain_{index}")),
            &z[start + index],
            32,
        )?;
    }
    for index in BYTE_CONTEXT_BUFFER_START_OFFSET..BYTE_CONTEXT_BUFFER_END_OFFSET {
        range_bits(
            cs.namespace(|| format!("{label}_byte_context_buffer_{index}")),
            &z[start + index],
            8,
        )?;
    }
    for index in BYTE_CONTEXT_HEADER_START_OFFSET..BYTE_CONTEXT_HEADER_END_OFFSET {
        range_bits(
            cs.namespace(|| format!("{label}_byte_context_header_{index}")),
            &z[start + index],
            8,
        )?;
    }
    let active = &z[start + BYTE_CONTEXT_ACTIVE_OFFSET];
    let started = &z[start + BYTE_CONTEXT_STARTED_OFFSET];
    let pending = &z[start + BYTE_CONTEXT_PENDING_OFFSET];
    let eof = &z[start + BYTE_CONTEXT_EOF_OFFSET];
    let padding_blocks = &z[start + BYTE_CONTEXT_PADDING_BLOCKS_OFFSET];
    cs.enforce(
        || format!("{label}_byte_context_started_requires_active"),
        |lc| lc + started.get_variable(),
        |lc| lc + CS::one() - active.get_variable(),
        |lc| lc,
    );
    cs.enforce(
        || format!("{label}_byte_context_pending_requires_started"),
        |lc| lc + pending.get_variable(),
        |lc| lc + CS::one() - started.get_variable(),
        |lc| lc,
    );
    cs.enforce(
        || format!("{label}_byte_context_eof_requires_started"),
        |lc| lc + eof.get_variable(),
        |lc| lc + CS::one() - started.get_variable(),
        |lc| lc,
    );
    cs.enforce(
        || format!("{label}_byte_context_padding_requires_eof"),
        |lc| lc + padding_blocks.get_variable(),
        |lc| lc + CS::one() - eof.get_variable(),
        |lc| lc,
    );
    for offset in 1..BYTE_CONTEXT_WIDTH {
        cs.enforce(
            || format!("{label}_inactive_byte_context_zero_{offset}"),
            |lc| lc + CS::one() - active.get_variable(),
            |lc| lc + z[start + offset].get_variable(),
            |lc| lc,
        );
    }
    Ok(())
}

/// Constrain a byte-sized state value to an exact inclusive fixed bound.  A
/// plain eight-bit decomposition would permit values above the bounded queue
/// capacity, turning the context into an implicit variable-sized arena.
fn constrain_bounded_u8<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    value: &AllocatedNum<Scalar>,
    maximum: usize,
) -> Result<(), SynthesisError> {
    let maximum = u64::try_from(maximum)
        .map_err(|_| SynthesisError::Unsatisfiable("byte bound overflow".to_owned()))?;
    if maximum > u64::from(u8::MAX) {
        return Err(SynthesisError::Unsatisfiable(
            "byte bound exceeds u8".to_owned(),
        ));
    }
    let raw = value.get_value().map(scalar_u64);
    let maximum = maximum as usize;
    let mut selectors = Vec::with_capacity(maximum + 1);
    for candidate in 0..=maximum {
        selectors.push(AllocatedBit::alloc(
            cs.namespace(|| format!("bounded_u8_selector_{candidate}")),
            raw.map(|current| current == candidate as u64),
        )?);
    }
    enforce_selector_value(
        &mut cs,
        value,
        &selectors,
        |index| index as u64,
        "bounded_u8",
    );
    Ok(())
}

fn digest_cells() -> Vec<usize> {
    let mut cells = (0..ANCHOR_CELLS).collect::<Vec<_>>();
    for range in [
        SOURCE_EVENT_DIGEST_START..SOURCE_EVENT_DIGEST_END,
        SOURCE_TRACE_ROOT_END..CHAIN_END,
        UNIQUENESS_START..UNIQUENESS_END,
        JMT_START..JMT_END,
        HIERARCHY_START..HIERARCHY_END,
        COMMITMENTS_START..COMMITMENTS_END,
        EXPECTED_TRACE_DIGEST_END..BYTE_CONTEXT_START,
    ] {
        cells.extend(range);
    }
    cells
}

fn transient_cells() -> Vec<usize> {
    let mut cells = Vec::new();
    for range in [
        COUNTERS_START..EXPECTED_TRACE_ROOT_START,
        EXPECTED_TRACE_DIGEST_END..RUNNING_STATE_ARITY_V2,
        SOURCE_TRACE_ROOT_END..CHAIN_END,
    ] {
        cells.extend(range);
    }
    cells
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NovaShapeMetricsV2 {
    constraints: u64,
    inputs: u64,
    auxiliaries: u64,
    nonzeros: u64,
}

fn measure_shape(
    circuit: &CheckpointNovaCircuitV2,
    state: &CheckpointRunningStateV2,
) -> Result<NovaShapeMetricsV2, RecursiveV2Error> {
    let mut cs = ShapeCS::<PallasEngine>::new();
    let input = state
        .scalars()
        .into_iter()
        .map(|value| AllocatedNum::alloc(&mut cs, || Ok(value)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| RecursiveV2Error::Invariant)?;
    let output = circuit
        .synthesize(&mut cs, &input)
        .map_err(|_| RecursiveV2Error::Invariant)?;
    if output.len() != RUNNING_STATE_ARITY_V2 {
        return Err(RecursiveV2Error::Invariant);
    }
    // Match Nova's `ShapeCS::r1cs_shape` conversion exactly: it omits zero
    // coefficients before recording sparse matrix entries.
    let shape = cs.r1cs_shape().map_err(|_| RecursiveV2Error::Invariant)?;
    let nonzeros = u64::try_from(shape.A().len())
        .and_then(|a| u64::try_from(shape.B().len()).map(|b| (a, b)))
        .and_then(|(a, b)| u64::try_from(shape.C().len()).map(|c| (a, b, c)))
        .map_err(|_| RecursiveV2Error::Limit)?;
    let nonzeros = nonzeros
        .0
        .checked_add(nonzeros.1)
        .and_then(|value| value.checked_add(nonzeros.2))
        .ok_or(RecursiveV2Error::Overflow)?;
    Ok(NovaShapeMetricsV2 {
        constraints: u64::try_from(cs.num_constraints()).map_err(|_| RecursiveV2Error::Limit)?,
        inputs: u64::try_from(cs.num_inputs()).map_err(|_| RecursiveV2Error::Limit)?,
        auxiliaries: u64::try_from(cs.num_aux()).map_err(|_| RecursiveV2Error::Limit)?,
        nonzeros,
    })
}

/// Bind the private circuit geometry into the storage specification without
/// exposing a Nova type. The digest is independent of witness values and
/// fails closed if the frozen R1CS cannot be synthesized.
pub(crate) fn circuit_shape_digest(
) -> Result<[u8; 32], super::super::recursive_reject::RecursiveV2Error> {
    use z00z_crypto::{sha256_256_role, CheckpointShaRole};

    let anchors = CheckpointNovaAnchorsV2::zero();
    let profile = RecursiveCircuitProfileV2::repository_fixture();
    let source = RecursiveTraceEventV2::new(
        0,
        RecursiveTraceOpcodeV2::BeginBlock,
        [1_u8; 32],
        Vec::new(),
        &profile,
    )
    .map_err(|_| super::super::recursive_reject::RecursiveV2Error::Invariant)?;
    let event = NovaTypedSourceEventV2::from_source(ControlPhaseV2::Idle, &source)
        .map_err(|_| super::super::recursive_reject::RecursiveV2Error::Invariant)?;
    let event = event.clone().with_successor(&event);
    let trace_authority = NovaTraceRootAuthorityV2::new([0_u64; DIGEST_LIMBS]);
    let state = CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
        .with_source_event(&event)
        .with_trace_authority(&trace_authority);
    let witness = NovaStepWitnessV2::new(false, event)
        .map_err(|_| super::super::recursive_reject::RecursiveV2Error::Invariant)?;
    let circuit = CheckpointNovaCircuitV2::new(anchors, witness);
    let metrics = measure_shape(&circuit, &state)?;
    let arity = u64::try_from(RUNNING_STATE_ARITY_V2)
        .map_err(|_| super::super::recursive_reject::RecursiveV2Error::Invariant)?;
    let table_digest = transition_table_digest();
    Ok(sha256_256_role(
        CheckpointShaRole::Statement,
        &[
            b"z00z.recursive.v2.nova-control-shape",
            &arity.to_le_bytes(),
            &metrics.constraints.to_le_bytes(),
            &metrics.inputs.to_le_bytes(),
            &metrics.auxiliaries.to_le_bytes(),
            &table_digest,
        ],
    ))
}

// The verifier authority is intentionally private until the recursive runner
// owns both bundle construction and proof ingestion.  These constants still
// make its wire contract immutable now: a caller cannot choose a Nova
// suite, codec, feature set, or allocation bound at load time.
const VERIFIER_BUNDLE_MAGIC_V2: [u8; 8] = *b"Z00ZNBV2";
const VERIFIER_BUNDLE_FORMAT_V2: u8 = 1;
const NOVA_SUITE_ID_V2: &[u8] = b"nova-snark/0.73.0;pallas-vesta;ppsnark-ipa";
const NOVA_FEATURE_ID_V2: &[u8] = b"io";
#[cfg(test)]
const MAX_NOVA_PUBLIC_PARAMETERS_BYTES_V2: usize = 512 * 1024 * 1024;
// This prevents bundle amplification at the decoder boundary. It is not an
// authority-approved watcher distribution or operating-memory budget.
const NOVA_VK_SAFETY_CAP_V2: usize = 384 * 1024 * 1024;
const MAX_NOVA_COMPRESSED_PROOF_BYTES_V2: usize = 128 * 1024;
// A verifier bundle carries only an authority-pinned header and VK. PP and PK
// are prover-local recovery material and are never verifier-bundle payloads.
const NOVA_BUNDLE_SAFETY_CAP_V2: usize = NOVA_VK_SAFETY_CAP_V2 + 1024;
const NOVA_PROOF_ENVELOPE_MAGIC_V2: [u8; 8] = *b"Z00ZNPE2";
const NOVA_PROOF_ENVELOPE_FORMAT_V2: u8 = 1;
const NOVA_SCALAR_WIRE_BYTES_V2: usize = 32;
const NOVA_PUBLIC_STATE_BYTES_V2: usize = RUNNING_STATE_ARITY_V2 * NOVA_SCALAR_WIRE_BYTES_V2;
// The envelope is exactly one header, two public IVC endpoints, and one
// compressed Nova body. It has no extensible field or key-material slot.
const NOVA_PROOF_ENVELOPE_HEADER_BYTES_V2: usize = 8
    + 1
    + 1
    + NOVA_SUITE_ID_V2.len()
    + 1
    + NOVA_FEATURE_ID_V2.len()
    + 8
    + (3 * 8)
    + (4 * 32)
    + 4
    + 32;
const NOVA_PROOF_ENVELOPE_SAFETY_CAP_V2: usize = NOVA_PROOF_ENVELOPE_HEADER_BYTES_V2
    + (2 * NOVA_PUBLIC_STATE_BYTES_V2)
    + MAX_NOVA_COMPRESSED_PROOF_BYTES_V2;
#[cfg(test)]
const NOVA_PALLAS_AFFINE_WIRE_BYTES_V2: u64 = 32;
#[cfg(test)]
const NOVA_USIZE_WIRE_BYTES_V2: u64 = 8;
#[cfg(test)]
const NOVA_PEDERSEN_UNIFORM_BYTES_V2: u64 = 32;
#[cfg(test)]
// A diagnostic-worker emergency limit, never a steady-state resource budget.
const NOVA_WORKER_SAFETY_CAP_V2: u64 = 24 * 1024 * 1024 * 1024;
const VERIFIER_BUNDLE_HEADER_BYTES_V2: usize =
    8 + 4 + 1 + NOVA_SUITE_ID_V2.len() + 1 + NOVA_FEATURE_ID_V2.len() + 8 + (4 * 8) + (11 * 32) + 4;

/// Resource caps applied before the private Nova setup path can allocate its
/// commitment key. The RSS ceiling is an authority-pinned safety ceiling, not
/// an inference from currently free host memory or a steady-state requirement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
struct NovaResourceLimitsV2 {
    pp_payload_bytes: u64,
    vk_payload_bytes: u64,
    bundle_bytes: u64,
    setup_and_proof_rss_bytes: u64,
}

#[cfg(test)]
const NOVA_RESOURCE_LIMITS_V2: NovaResourceLimitsV2 = NovaResourceLimitsV2 {
    pp_payload_bytes: MAX_NOVA_PUBLIC_PARAMETERS_BYTES_V2 as u64,
    vk_payload_bytes: NOVA_VK_SAFETY_CAP_V2 as u64,
    bundle_bytes: NOVA_BUNDLE_SAFETY_CAP_V2 as u64,
    setup_and_proof_rss_bytes: NOVA_WORKER_SAFETY_CAP_V2,
};

/// A lower bound, not a replacement for the one permitted real measurement.
/// Every value comes from the exact current `ShapeCS` sparse matrices and the
/// pinned Nova 0.73.0 setup/IPA layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
struct NovaResourcePreflightV2 {
    shape: NovaShapeMetricsV2,
    ck_floor: u64,
    generator_count: u64,
    pp_payload_lower_bound: u64,
    vk_payload_lower_bound: u64,
    bundle_lower_bound: u64,
    pedersen_rss_lower_bound: u64,
}

#[cfg(test)]
fn checked_resource_add(left: u64, right: u64) -> Result<u64, RecursiveV2Error> {
    left.checked_add(right).ok_or(RecursiveV2Error::Overflow)
}

#[cfg(test)]
fn checked_resource_mul(left: u64, right: u64) -> Result<u64, RecursiveV2Error> {
    left.checked_mul(right).ok_or(RecursiveV2Error::Overflow)
}

#[cfg(test)]
fn nova_resource_preflight_from_shape(
    shape: NovaShapeMetricsV2,
    limits: NovaResourceLimitsV2,
) -> Result<NovaResourcePreflightV2, RecursiveV2Error> {
    // Pinned `RelaxedR1CSSNARK::ck_floor` is exactly |A| + |B| + |C|.
    let ck_floor = shape.nonzeros;
    let max_ck_size = shape.constraints.max(shape.auxiliaries).max(ck_floor);
    if max_ck_size == 0 {
        return Err(RecursiveV2Error::Resource);
    }

    // Pinned Pedersen setup asks `from_label` for N.next_power_of_two() + 1
    // affine points, then retains the remaining points as the commitment key.
    let generator_count = max_ck_size
        .checked_next_power_of_two()
        .and_then(|power| power.checked_add(1))
        .ok_or(RecursiveV2Error::Overflow)?;
    let primary_ck_wire_bytes =
        checked_resource_mul(generator_count, NOVA_PALLAS_AFFINE_WIRE_BYTES_V2)?;

    // A bincode-legacy `SparseMatrix` stores data (32-byte Pallas scalar),
    // column indices and row pointers (both u64), plus three Vec lengths and
    // `cols`. The primary augmented shape must retain every current step-C
    // matrix entry, so this is a lower bound on the serialized PP shape.
    let matrix_entries = checked_resource_mul(shape.nonzeros, 40)?;
    let row_pointers = checked_resource_mul(
        checked_resource_add(shape.constraints, 1)?,
        3 * NOVA_USIZE_WIRE_BYTES_V2,
    )?;
    let raw_r1cs_wire_bytes = checked_resource_add(
        checked_resource_add(matrix_entries, row_pointers)?,
        15 * NOVA_USIZE_WIRE_BYTES_V2,
    )?;
    let pp_payload_lower_bound = checked_resource_add(primary_ck_wire_bytes, raw_r1cs_wire_bytes)?;

    // Pinned IPA verifier setup clones the primary Pedersen key into ck_v;
    // the rest of the verifier key can only increase this payload.
    let vk_payload_lower_bound = primary_ck_wire_bytes;
    // The verifier artifact is strictly `header + VK`. PP and PK are private
    // prover recovery material, so including PP here would hide the actual
    // verifier-distribution cost behind a role-mixed estimate.
    let bundle_lower_bound = checked_resource_add(
        u64::try_from(VERIFIER_BUNDLE_HEADER_BYTES_V2).map_err(|_| RecursiveV2Error::Limit)?,
        vk_payload_lower_bound,
    )?;

    // During `DlogGroup::from_label`, pinned Nova simultaneously retains at
    // least uniform inputs, projective points, and affine points. Afterwards
    // the affine vector and the copied CK coexist. Use the larger definite
    // phase as a lower bound; allocator overhead and all other setup objects
    // are deliberately excluded, so passing this check is not a measurement.
    let affine_bytes = u64::try_from(std::mem::size_of::<pallas::Affine>())
        .map_err(|_| RecursiveV2Error::Limit)?;
    let point_bytes =
        u64::try_from(std::mem::size_of::<pallas::Point>()).map_err(|_| RecursiveV2Error::Limit)?;
    let from_label_phase = checked_resource_add(
        checked_resource_add(NOVA_PEDERSEN_UNIFORM_BYTES_V2, point_bytes)?,
        affine_bytes,
    )?;
    let ck_copy_phase = checked_resource_mul(affine_bytes, 2)?;
    let pedersen_rss_lower_bound =
        checked_resource_mul(generator_count, from_label_phase.max(ck_copy_phase))?;

    if pp_payload_lower_bound > limits.pp_payload_bytes
        || vk_payload_lower_bound > limits.vk_payload_bytes
        || bundle_lower_bound > limits.bundle_bytes
        || pedersen_rss_lower_bound > limits.setup_and_proof_rss_bytes
    {
        return Err(RecursiveV2Error::Resource);
    }

    Ok(NovaResourcePreflightV2 {
        shape,
        ck_floor,
        generator_count,
        pp_payload_lower_bound,
        vk_payload_lower_bound,
        bundle_lower_bound,
        pedersen_rss_lower_bound,
    })
}

#[cfg(test)]
fn nova_resource_preflight(
    circuit: &CheckpointNovaCircuitV2,
    state: &CheckpointRunningStateV2,
) -> Result<NovaResourcePreflightV2, RecursiveV2Error> {
    nova_resource_preflight_from_shape(measure_shape(circuit, state)?, NOVA_RESOURCE_LIMITS_V2)
}

// Static base-shape arithmetic is an early reject only. This test-only helper
// supplies reproducible real-proof fixtures; a production runner must instead
// first obtain the exact augmented primary/secondary shapes and run setup in a
// bounded worker that records actual RSS and time.
#[cfg(test)]
fn setup_public_parameters_after_preflight(
    circuit: &CheckpointNovaCircuitV2,
    state: &CheckpointRunningStateV2,
) -> Result<NovaPublicParameters, RecursiveV2Error> {
    // `?` is intentionally before every Nova setup input: a resource rejection
    // has no route to `PublicParams::setup`.
    let _preflight = nova_resource_preflight(circuit, state)?;
    let ck_primary = &*NovaSnark::<PallasEngine>::ck_floor();
    let ck_secondary = &*NovaSnark::<VestaEngine>::ck_floor();
    PublicParams::setup(circuit, ck_primary, ck_secondary).map_err(|_| RecursiveV2Error::Invariant)
}

/// Immutable binding values that must agree before the authority bundle can
/// deserialize any dependency-owned object.
#[derive(Clone, Copy)]
#[allow(dead_code)] // The private runner is the sole production bundle caller.
struct VerifierBundleBindingV2 {
    authority_digest: [u8; 32],
    profile_digest: [u8; 32],
    spec_digest: [u8; 32],
    pp_digest: [u8; 32],
}

#[allow(dead_code)] // See `VerifierBundleBindingV2`.
impl VerifierBundleBindingV2 {
    fn from_authority(
        authority: &RecursiveAuthoritySnapshotV2,
        profile: &RecursiveCircuitProfileV2,
        spec: &RecursiveCircuitSpecV2,
        pp: &NovaPublicParameters,
    ) -> Result<Self, RecursiveV2Error> {
        Ok(Self {
            authority_digest: authority.authority().digest(),
            profile_digest: profile.digest(),
            spec_digest: spec.digest(),
            pp_digest: scalar_digest(pp.digest())?,
        })
    }
}

/// Fixed-size, authenticated envelope for one Nova verifier authority.
///
/// The header is parsed and checked before the public-parameter, verifier-key,
/// or proof codec is entered. Dependency-specific objects never leave this
/// owner and all decoded inputs are re-encoded to reject alternate forms.
#[derive(Clone, Copy)]
#[allow(dead_code)] // The private runner integration follows the full T2 relation.
struct VerifierBundleHeaderV2 {
    pp_digest: [u8; 32],
    authority_digest: [u8; 32],
    profile_digest: [u8; 32],
    spec_digest: [u8; 32],
    grammar_digest: [u8; 32],
    shape_digest: [u8; 32],
    source_digest: [u8; 32],
    lockfile_digest: [u8; 32],
    manifest_digest: [u8; 32],
    vk_digest: [u8; 32],
    project_digest: [u8; 32],
    primary_constraints: u64,
    secondary_constraints: u64,
    primary_variables: u64,
    secondary_variables: u64,
    vk_payload_bytes: u32,
}

#[allow(dead_code)] // See `VerifierBundleHeaderV2`.
impl VerifierBundleHeaderV2 {
    fn new(
        pp: &NovaPublicParameters,
        binding: VerifierBundleBindingV2,
        vk_payload: &[u8],
    ) -> Result<Self, RecursiveV2Error> {
        let vk_payload_bytes =
            u32::try_from(vk_payload.len()).map_err(|_| RecursiveV2Error::Limit)?;
        let (primary_constraints, secondary_constraints) = pp.num_constraints();
        let (primary_variables, secondary_variables) = pp.num_variables();
        let pp_digest = scalar_digest(pp.digest())?;
        // Prevent an unusable or mixed-generation bundle from being emitted:
        // the expected authority binding and the supplied prover PP must agree
        // before any verifier artifact is framed.
        if pp_digest != binding.pp_digest {
            return Err(RecursiveV2Error::Authority);
        }
        let mut header = Self {
            pp_digest,
            authority_digest: binding.authority_digest,
            profile_digest: binding.profile_digest,
            spec_digest: binding.spec_digest,
            grammar_digest: transition_table_digest(),
            shape_digest: circuit_shape_digest()?,
            source_digest: source_revision_digest(),
            lockfile_digest: lockfile_digest(),
            manifest_digest: manifest_digest(),
            vk_digest: bundle_payload_digest(b"vk", vk_payload),
            project_digest: [0_u8; 32],
            primary_constraints: u64::try_from(primary_constraints)
                .map_err(|_| RecursiveV2Error::Limit)?,
            secondary_constraints: u64::try_from(secondary_constraints)
                .map_err(|_| RecursiveV2Error::Limit)?,
            primary_variables: u64::try_from(primary_variables)
                .map_err(|_| RecursiveV2Error::Limit)?,
            secondary_variables: u64::try_from(secondary_variables)
                .map_err(|_| RecursiveV2Error::Limit)?,
            vk_payload_bytes,
        };
        header.project_digest = bundle_project_digest(&header.canonical_prefix());
        Ok(header)
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = self.canonical_prefix();
        bytes.extend_from_slice(&self.project_digest);
        bytes
    }

    fn canonical_prefix(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(VERIFIER_BUNDLE_HEADER_BYTES_V2);
        bytes.extend_from_slice(&VERIFIER_BUNDLE_MAGIC_V2);
        bytes.push(VERIFIER_BUNDLE_FORMAT_V2);
        bytes.push(RECURSIVE_CIRCUIT_PROFILE_VERSION_V2);
        bytes.push(RECURSIVE_CIRCUIT_SPEC_VERSION_V2);
        bytes.push(2); // Recursive checkpoint V2.
        append_fixed_bytes(&mut bytes, NOVA_SUITE_ID_V2);
        append_fixed_bytes(&mut bytes, NOVA_FEATURE_ID_V2);
        bytes.extend_from_slice(&(RUNNING_STATE_ARITY_V2 as u64).to_le_bytes());
        for value in [
            self.primary_constraints,
            self.secondary_constraints,
            self.primary_variables,
            self.secondary_variables,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for digest in [
            self.pp_digest,
            self.authority_digest,
            self.profile_digest,
            self.spec_digest,
            self.grammar_digest,
            self.shape_digest,
            self.source_digest,
            self.lockfile_digest,
            self.manifest_digest,
            self.vk_digest,
        ] {
            bytes.extend_from_slice(&digest);
        }
        bytes.extend_from_slice(&self.vk_payload_bytes.to_le_bytes());
        bytes
    }

    fn decode(bytes: &[u8]) -> Result<Self, RecursiveV2Error> {
        if bytes.len() < VERIFIER_BUNDLE_HEADER_BYTES_V2 {
            return Err(RecursiveV2Error::Canonical);
        }
        let mut cursor = 0_usize;
        if take_fixed::<8>(bytes, &mut cursor)? != VERIFIER_BUNDLE_MAGIC_V2
            || take_u8_bundle(bytes, &mut cursor)? != VERIFIER_BUNDLE_FORMAT_V2
            || take_u8_bundle(bytes, &mut cursor)? != RECURSIVE_CIRCUIT_PROFILE_VERSION_V2
            || take_u8_bundle(bytes, &mut cursor)? != RECURSIVE_CIRCUIT_SPEC_VERSION_V2
            || take_u8_bundle(bytes, &mut cursor)? != 2
        {
            return Err(RecursiveV2Error::Version);
        }
        take_fixed_bytes(bytes, &mut cursor, NOVA_SUITE_ID_V2)?;
        take_fixed_bytes(bytes, &mut cursor, NOVA_FEATURE_ID_V2)?;
        if take_u64_bundle(bytes, &mut cursor)? != RUNNING_STATE_ARITY_V2 as u64 {
            return Err(RecursiveV2Error::Invariant);
        }
        let primary_constraints = take_u64_bundle(bytes, &mut cursor)?;
        let secondary_constraints = take_u64_bundle(bytes, &mut cursor)?;
        let primary_variables = take_u64_bundle(bytes, &mut cursor)?;
        let secondary_variables = take_u64_bundle(bytes, &mut cursor)?;
        let pp_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let authority_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let profile_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let spec_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let grammar_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let shape_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let source_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let lockfile_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let manifest_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let vk_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let vk_payload_bytes = take_u32_bundle(bytes, &mut cursor)?;
        let project_digest = take_fixed::<32>(bytes, &mut cursor)?;
        if cursor != VERIFIER_BUNDLE_HEADER_BYTES_V2
            || project_digest != bundle_project_digest(&bytes[..cursor - project_digest.len()])
        {
            return Err(RecursiveV2Error::Canonical);
        }
        Ok(Self {
            pp_digest,
            authority_digest,
            profile_digest,
            spec_digest,
            grammar_digest,
            shape_digest,
            source_digest,
            lockfile_digest,
            manifest_digest,
            vk_digest,
            project_digest,
            primary_constraints,
            secondary_constraints,
            primary_variables,
            secondary_variables,
            vk_payload_bytes,
        })
    }

    fn validate_binding(&self, binding: VerifierBundleBindingV2) -> Result<(), RecursiveV2Error> {
        if self.authority_digest != binding.authority_digest
            || self.profile_digest != binding.profile_digest
            || self.spec_digest != binding.spec_digest
            || self.pp_digest != binding.pp_digest
            || self.grammar_digest != transition_table_digest()
            || self.shape_digest != circuit_shape_digest()?
            || self.source_digest != source_revision_digest()
            || self.lockfile_digest != lockfile_digest()
            || self.manifest_digest != manifest_digest()
        {
            return Err(RecursiveV2Error::Authority);
        }
        Ok(())
    }
}

/// Prover-only material. Neither field is serialised into a verifier artifact:
/// PP and PK stay in the encrypted prover-recovery store owned by the runner.
#[allow(dead_code)] // The private runner is the sole production owner.
struct NovaProverMaterialV2 {
    pp: NovaPublicParameters,
    pk: NovaProverKey,
}

#[allow(dead_code)] // See `NovaProverMaterialV2`.
impl NovaProverMaterialV2 {
    fn new(pp: NovaPublicParameters, pk: NovaProverKey) -> Self {
        Self { pp, pk }
    }

    fn verifier_bundle(
        &self,
        vk: &NovaVerifierKey,
        binding: VerifierBundleBindingV2,
    ) -> Result<Vec<u8>, RecursiveV2Error> {
        NovaVerifierBundleV2::encode(&self.pp, vk, binding)
    }
}

/// The only owner of decoded Nova verifier objects. It is intentionally
/// private, immutable, and validated before a compressed proof is decoded.
#[allow(dead_code)] // Wired to the runner with the remaining semantic relations.
struct NovaVerifierBundleV2 {
    header: VerifierBundleHeaderV2,
    vk: NovaVerifierKey,
    bundle_digest: [u8; 32],
}

#[allow(dead_code)] // See `NovaVerifierBundleV2`.
impl NovaVerifierBundleV2 {
    fn encode(
        pp: &NovaPublicParameters,
        vk: &NovaVerifierKey,
        binding: VerifierBundleBindingV2,
    ) -> Result<Vec<u8>, RecursiveV2Error> {
        let vk_payload = encode_bincode(vk, NOVA_VK_SAFETY_CAP_V2)?;
        // `pp_digest` is authority-pinned in the header at generation time;
        // PP bytes never cross into the verifier artifact or its decoder.
        let header = VerifierBundleHeaderV2::new(pp, binding, &vk_payload)?;
        let mut encoded = header.encode();
        encoded.extend_from_slice(&vk_payload);
        if encoded.len() > NOVA_BUNDLE_SAFETY_CAP_V2 {
            return Err(RecursiveV2Error::Limit);
        }
        Ok(encoded)
    }

    fn load(bytes: &[u8], binding: VerifierBundleBindingV2) -> Result<Self, RecursiveV2Error> {
        if bytes.len() > NOVA_BUNDLE_SAFETY_CAP_V2 {
            return Err(RecursiveV2Error::Limit);
        }
        // This completed header validation deliberately precedes every
        // bincode allocation and every proof decode entry point.
        let header = VerifierBundleHeaderV2::decode(bytes)?;
        header.validate_binding(binding)?;
        let vk_len =
            usize::try_from(header.vk_payload_bytes).map_err(|_| RecursiveV2Error::Limit)?;
        if vk_len > NOVA_VK_SAFETY_CAP_V2 {
            return Err(RecursiveV2Error::Limit);
        }
        let vk_end = VERIFIER_BUNDLE_HEADER_BYTES_V2
            .checked_add(vk_len)
            .ok_or(RecursiveV2Error::Overflow)?;
        if bytes.len() != vk_end {
            return Err(RecursiveV2Error::Canonical);
        }
        let vk_payload = bytes
            .get(VERIFIER_BUNDLE_HEADER_BYTES_V2..vk_end)
            .ok_or(RecursiveV2Error::Canonical)?;
        if bundle_payload_digest(b"vk", vk_payload) != header.vk_digest {
            return Err(RecursiveV2Error::Canonical);
        }
        let vk = decode_bincode::<NovaVerifierKey, NOVA_VK_SAFETY_CAP_V2>(vk_payload)?;
        Ok(Self {
            header,
            vk,
            bundle_digest: bundle_payload_digest(b"verifier-bundle", bytes),
        })
    }

    fn decode_compressed_proof(&self, bytes: &[u8]) -> Result<NovaProof, RecursiveV2Error> {
        // `load` authenticated the authority-pinned `pp_digest` and `vk_digest`
        // before this decoder is reachable. Verifier operation intentionally
        // needs only VK; it has no PP/PK dependency or decode path.
        decode_bincode::<NovaProof, MAX_NOVA_COMPRESSED_PROOF_BYTES_V2>(bytes)
    }

    fn verify(
        &self,
        proof: &NovaProof,
        steps: usize,
        initial_state: &[Scalar],
    ) -> Result<Vec<Scalar>, RecursiveV2Error> {
        if initial_state.len() != RUNNING_STATE_ARITY_V2 {
            return Err(RecursiveV2Error::Invariant);
        }
        proof
            .verify(&self.vk, steps, initial_state)
            .map_err(|_| RecursiveV2Error::Invariant)
    }

    fn digest(&self) -> [u8; 32] {
        self.bundle_digest
    }
}

/// The only logical compressed Nova proof body. It is not a key container,
/// recovery image, bundle, or second proof wrapper.
#[allow(dead_code)] // Emitted by the private continuous runner in T3.
struct NovaCompressedSnapshotV2 {
    proof_bytes: Vec<u8>,
}

#[allow(dead_code)] // See `NovaCompressedSnapshotV2`.
impl NovaCompressedSnapshotV2 {
    fn new(proof_bytes: Vec<u8>) -> Result<Self, RecursiveV2Error> {
        if proof_bytes.is_empty() {
            return Err(RecursiveV2Error::Canonical);
        }
        if proof_bytes.len() > MAX_NOVA_COMPRESSED_PROOF_BYTES_V2 {
            return Err(RecursiveV2Error::Limit);
        }
        Ok(Self { proof_bytes })
    }
}

/// Fixed framing for the only portable Nova proof object.
///
/// The header commits to the already-selected verifier bundle, public initial
/// and final IVC states, range, step count, and exact compressed body before
/// the body reaches a Nova decoder. It deliberately has no PP, PK, VK, or
/// caller-selected parameter field.
#[derive(Clone, Copy)]
#[allow(dead_code)] // See `NovaProofEnvelopeV2`.
struct NovaProofEnvelopeHeaderV2 {
    bundle_digest: [u8; 32],
    initial_state_digest: [u8; 32],
    final_state_digest: [u8; 32],
    proof_digest: [u8; 32],
    start_height: u64,
    end_height: u64,
    steps: u64,
    proof_payload_bytes: u32,
    project_digest: [u8; 32],
}

#[allow(dead_code)] // See `NovaProofEnvelopeV2`.
impl NovaProofEnvelopeHeaderV2 {
    fn new(
        bundle_digest: [u8; 32],
        initial_state: &[u8],
        final_state: &[u8],
        start_height: u64,
        end_height: u64,
        steps: u64,
        proof_bytes: &[u8],
    ) -> Result<Self, RecursiveV2Error> {
        if end_height < start_height || steps == 0 {
            return Err(RecursiveV2Error::Invariant);
        }
        let proof_payload_bytes =
            u32::try_from(proof_bytes.len()).map_err(|_| RecursiveV2Error::Limit)?;
        let mut header = Self {
            bundle_digest,
            initial_state_digest: proof_envelope_component_digest(b"initial-state", initial_state),
            final_state_digest: proof_envelope_component_digest(b"final-state", final_state),
            proof_digest: proof_envelope_component_digest(b"compressed-proof", proof_bytes),
            start_height,
            end_height,
            steps,
            proof_payload_bytes,
            project_digest: [0_u8; 32],
        };
        header.project_digest = proof_envelope_project_digest(&header.canonical_prefix());
        Ok(header)
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = self.canonical_prefix();
        bytes.extend_from_slice(&self.project_digest);
        bytes
    }

    fn canonical_prefix(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(NOVA_PROOF_ENVELOPE_HEADER_BYTES_V2);
        bytes.extend_from_slice(&NOVA_PROOF_ENVELOPE_MAGIC_V2);
        bytes.push(NOVA_PROOF_ENVELOPE_FORMAT_V2);
        append_fixed_bytes(&mut bytes, NOVA_SUITE_ID_V2);
        append_fixed_bytes(&mut bytes, NOVA_FEATURE_ID_V2);
        bytes.extend_from_slice(&(RUNNING_STATE_ARITY_V2 as u64).to_le_bytes());
        for value in [self.start_height, self.end_height, self.steps] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for digest in [
            self.bundle_digest,
            self.initial_state_digest,
            self.final_state_digest,
            self.proof_digest,
        ] {
            bytes.extend_from_slice(&digest);
        }
        bytes.extend_from_slice(&self.proof_payload_bytes.to_le_bytes());
        bytes
    }

    fn decode(bytes: &[u8]) -> Result<Self, RecursiveV2Error> {
        if bytes.len() < NOVA_PROOF_ENVELOPE_HEADER_BYTES_V2 {
            return Err(RecursiveV2Error::Canonical);
        }
        let mut cursor = 0_usize;
        if take_fixed::<8>(bytes, &mut cursor)? != NOVA_PROOF_ENVELOPE_MAGIC_V2
            || take_u8_bundle(bytes, &mut cursor)? != NOVA_PROOF_ENVELOPE_FORMAT_V2
        {
            return Err(RecursiveV2Error::Version);
        }
        take_fixed_bytes(bytes, &mut cursor, NOVA_SUITE_ID_V2)?;
        take_fixed_bytes(bytes, &mut cursor, NOVA_FEATURE_ID_V2)?;
        if take_u64_bundle(bytes, &mut cursor)? != RUNNING_STATE_ARITY_V2 as u64 {
            return Err(RecursiveV2Error::Invariant);
        }
        let start_height = take_u64_bundle(bytes, &mut cursor)?;
        let end_height = take_u64_bundle(bytes, &mut cursor)?;
        let steps = take_u64_bundle(bytes, &mut cursor)?;
        let bundle_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let initial_state_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let final_state_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let proof_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let proof_payload_bytes = take_u32_bundle(bytes, &mut cursor)?;
        let project_digest = take_fixed::<32>(bytes, &mut cursor)?;
        if cursor != NOVA_PROOF_ENVELOPE_HEADER_BYTES_V2
            || end_height < start_height
            || steps == 0
            || project_digest
                != proof_envelope_project_digest(&bytes[..cursor - project_digest.len()])
        {
            return Err(RecursiveV2Error::Canonical);
        }
        Ok(Self {
            bundle_digest,
            initial_state_digest,
            final_state_digest,
            proof_digest,
            start_height,
            end_height,
            steps,
            proof_payload_bytes,
            project_digest,
        })
    }
}

/// The one portable framing for a compressed Nova snapshot. PP, PK, and VK
/// cannot enter this type: the selected verifier bundle is named only by its
/// content digest and must already be loaded before this envelope is decoded.
#[allow(dead_code)] // The private continuous runner owns construction in T3.
struct NovaProofEnvelopeV2 {
    header: NovaProofEnvelopeHeaderV2,
    initial_state: Vec<Scalar>,
    final_state: Vec<Scalar>,
    body: NovaCompressedSnapshotV2,
}

#[allow(dead_code)] // See `NovaProofEnvelopeV2`.
impl NovaProofEnvelopeV2 {
    fn new(
        bundle: &NovaVerifierBundleV2,
        start_height: u64,
        end_height: u64,
        steps: usize,
        initial_state: &[Scalar],
        final_state: &[Scalar],
        proof_bytes: Vec<u8>,
    ) -> Result<Self, RecursiveV2Error> {
        let steps = u64::try_from(steps).map_err(|_| RecursiveV2Error::Limit)?;
        let initial_bytes = encode_public_state(initial_state)?;
        let final_bytes = encode_public_state(final_state)?;
        let body = NovaCompressedSnapshotV2::new(proof_bytes)?;
        // A portable body is admitted only after the authority-selected bundle
        // has completed its binding and key checks. This decode has no PP/PK
        // path and prevents arbitrary payloads from being wrapped as proofs.
        let _ = bundle.decode_compressed_proof(&body.proof_bytes)?;
        let header = NovaProofEnvelopeHeaderV2::new(
            bundle.digest(),
            &initial_bytes,
            &final_bytes,
            start_height,
            end_height,
            steps,
            &body.proof_bytes,
        )?;
        Ok(Self {
            header,
            initial_state: initial_state.to_vec(),
            final_state: final_state.to_vec(),
            body,
        })
    }

    fn encode(&self) -> Result<Vec<u8>, RecursiveV2Error> {
        let initial_bytes = encode_public_state(&self.initial_state)?;
        let final_bytes = encode_public_state(&self.final_state)?;
        if proof_envelope_component_digest(b"initial-state", &initial_bytes)
            != self.header.initial_state_digest
            || proof_envelope_component_digest(b"final-state", &final_bytes)
                != self.header.final_state_digest
            || proof_envelope_component_digest(b"compressed-proof", &self.body.proof_bytes)
                != self.header.proof_digest
        {
            return Err(RecursiveV2Error::Canonical);
        }
        let mut bytes = self.header.encode();
        bytes.extend_from_slice(&initial_bytes);
        bytes.extend_from_slice(&final_bytes);
        bytes.extend_from_slice(&self.body.proof_bytes);
        if bytes.len() > NOVA_PROOF_ENVELOPE_SAFETY_CAP_V2 {
            return Err(RecursiveV2Error::Limit);
        }
        Ok(bytes)
    }

    fn load(bytes: &[u8], bundle: &NovaVerifierBundleV2) -> Result<Self, RecursiveV2Error> {
        if bytes.len() > NOVA_PROOF_ENVELOPE_SAFETY_CAP_V2 {
            return Err(RecursiveV2Error::Limit);
        }
        let header = NovaProofEnvelopeHeaderV2::decode(bytes)?;
        if header.bundle_digest != bundle.digest() {
            return Err(RecursiveV2Error::Authority);
        }
        let public_end = NOVA_PROOF_ENVELOPE_HEADER_BYTES_V2
            .checked_add(NOVA_PUBLIC_STATE_BYTES_V2)
            .and_then(|value| value.checked_add(NOVA_PUBLIC_STATE_BYTES_V2))
            .ok_or(RecursiveV2Error::Overflow)?;
        let proof_len =
            usize::try_from(header.proof_payload_bytes).map_err(|_| RecursiveV2Error::Limit)?;
        if proof_len == 0 || proof_len > MAX_NOVA_COMPRESSED_PROOF_BYTES_V2 {
            return Err(RecursiveV2Error::Limit);
        }
        let proof_end = public_end
            .checked_add(proof_len)
            .ok_or(RecursiveV2Error::Overflow)?;
        if bytes.len() != proof_end {
            return Err(RecursiveV2Error::Canonical);
        }
        let initial_bytes = bytes
            .get(
                NOVA_PROOF_ENVELOPE_HEADER_BYTES_V2
                    ..NOVA_PROOF_ENVELOPE_HEADER_BYTES_V2 + NOVA_PUBLIC_STATE_BYTES_V2,
            )
            .ok_or(RecursiveV2Error::Canonical)?;
        let final_bytes = bytes
            .get(NOVA_PROOF_ENVELOPE_HEADER_BYTES_V2 + NOVA_PUBLIC_STATE_BYTES_V2..public_end)
            .ok_or(RecursiveV2Error::Canonical)?;
        let proof_bytes = bytes
            .get(public_end..proof_end)
            .ok_or(RecursiveV2Error::Canonical)?;
        if proof_envelope_component_digest(b"initial-state", initial_bytes)
            != header.initial_state_digest
            || proof_envelope_component_digest(b"final-state", final_bytes)
                != header.final_state_digest
            || proof_envelope_component_digest(b"compressed-proof", proof_bytes)
                != header.proof_digest
        {
            return Err(RecursiveV2Error::Canonical);
        }
        let initial_state = decode_public_state(initial_bytes)?;
        let final_state = decode_public_state(final_bytes)?;
        let body = NovaCompressedSnapshotV2::new(proof_bytes.to_vec())?;
        // Strict bundle validation precedes this proof decode by the load API;
        // the decoder accepts a bounded canonical compressed proof only.
        let _ = bundle.decode_compressed_proof(&body.proof_bytes)?;
        Ok(Self {
            header,
            initial_state,
            final_state,
            body,
        })
    }

    fn verify(&self, bundle: &NovaVerifierBundleV2) -> Result<Vec<Scalar>, RecursiveV2Error> {
        if self.header.bundle_digest != bundle.digest() {
            return Err(RecursiveV2Error::Authority);
        }
        let steps = usize::try_from(self.header.steps).map_err(|_| RecursiveV2Error::Limit)?;
        let proof = bundle.decode_compressed_proof(&self.body.proof_bytes)?;
        let output = bundle.verify(&proof, steps, &self.initial_state)?;
        if output != self.final_state {
            return Err(RecursiveV2Error::Invariant);
        }
        Ok(output)
    }
}

fn encode_public_state(state: &[Scalar]) -> Result<Vec<u8>, RecursiveV2Error> {
    if state.len() != RUNNING_STATE_ARITY_V2 {
        return Err(RecursiveV2Error::Invariant);
    }
    let mut bytes = Vec::with_capacity(NOVA_PUBLIC_STATE_BYTES_V2);
    for scalar in state {
        let repr = scalar.to_repr();
        if repr.as_ref().len() != NOVA_SCALAR_WIRE_BYTES_V2 {
            return Err(RecursiveV2Error::Invariant);
        }
        bytes.extend_from_slice(repr.as_ref());
    }
    Ok(bytes)
}

fn decode_public_state(bytes: &[u8]) -> Result<Vec<Scalar>, RecursiveV2Error> {
    if bytes.len() != NOVA_PUBLIC_STATE_BYTES_V2 {
        return Err(RecursiveV2Error::Canonical);
    }
    bytes
        .chunks_exact(NOVA_SCALAR_WIRE_BYTES_V2)
        .map(|chunk| {
            let mut repr = <Scalar as PrimeField>::Repr::default();
            repr.as_mut().copy_from_slice(chunk);
            Option::<Scalar>::from(Scalar::from_repr(repr)).ok_or(RecursiveV2Error::Canonical)
        })
        .collect()
}

fn encode_bincode<T: Serialize>(value: &T, cap: usize) -> Result<Vec<u8>, RecursiveV2Error> {
    let bytes = bincode::serde::encode_to_vec(value, bincode::config::legacy())
        .map_err(|_| RecursiveV2Error::Canonical)?;
    if bytes.len() > cap {
        return Err(RecursiveV2Error::Limit);
    }
    Ok(bytes)
}

fn decode_bincode<T: DeserializeOwned + Serialize, const CAP: usize>(
    bytes: &[u8],
) -> Result<T, RecursiveV2Error> {
    if bytes.len() > CAP {
        return Err(RecursiveV2Error::Limit);
    }
    let (decoded, consumed) =
        bincode::serde::decode_from_slice(bytes, bincode::config::legacy().with_limit::<CAP>())
            .map_err(|_| RecursiveV2Error::Canonical)?;
    if consumed != bytes.len() || encode_bincode(&decoded, CAP)? != bytes {
        return Err(RecursiveV2Error::Canonical);
    }
    Ok(decoded)
}

fn scalar_digest(value: Scalar) -> Result<[u8; 32], RecursiveV2Error> {
    value
        .to_repr()
        .as_ref()
        .try_into()
        .map_err(|_| RecursiveV2Error::Invariant)
}

fn source_revision_digest() -> [u8; 32] {
    use z00z_crypto::{sha256_256_role, CheckpointShaRole};

    // `recursive_trace` owns the canonical source encoder, chunk grammar, and
    // derived SHA schedule consumed by this sole circuit owner. Binding only
    // this file would let a trace-owner source revision escape the immutable
    // verifier bundle identity.
    sha256_256_role(
        CheckpointShaRole::Statement,
        &[
            b"z00z.recursive.v2.nova-owner-source",
            include_bytes!("nova.rs"),
            include_bytes!("../recursive_trace.rs"),
        ],
    )
}

fn lockfile_digest() -> [u8; 32] {
    use z00z_crypto::{sha256_256_role, CheckpointShaRole};

    sha256_256_role(
        CheckpointShaRole::Statement,
        &[
            b"z00z.recursive.v2.cargo-lock",
            include_bytes!("../../../../../Cargo.lock"),
        ],
    )
}

fn manifest_digest() -> [u8; 32] {
    use z00z_crypto::{sha256_256_role, CheckpointShaRole};

    sha256_256_role(
        CheckpointShaRole::Statement,
        &[
            b"z00z.recursive.v2.storage-manifest",
            include_bytes!("../../../Cargo.toml"),
        ],
    )
}

fn bundle_payload_digest(kind: &[u8], payload: &[u8]) -> [u8; 32] {
    use z00z_crypto::{sha256_256_role, CheckpointShaRole};

    sha256_256_role(
        CheckpointShaRole::Statement,
        &[b"z00z.recursive.v2.nova-bundle-payload", kind, payload],
    )
}

fn bundle_project_digest(prefix: &[u8]) -> [u8; 32] {
    use z00z_crypto::{sha256_256_role, CheckpointShaRole};

    sha256_256_role(
        CheckpointShaRole::Statement,
        &[b"z00z.recursive.v2.nova-verifier-bundle", prefix],
    )
}

fn proof_envelope_component_digest(kind: &[u8], payload: &[u8]) -> [u8; 32] {
    use z00z_crypto::{sha256_256_role, CheckpointShaRole};

    sha256_256_role(
        CheckpointShaRole::Statement,
        &[
            b"z00z.recursive.v2.nova-proof-envelope-component",
            kind,
            payload,
        ],
    )
}

fn proof_envelope_project_digest(prefix: &[u8]) -> [u8; 32] {
    use z00z_crypto::{sha256_256_role, CheckpointShaRole};

    sha256_256_role(
        CheckpointShaRole::Statement,
        &[b"z00z.recursive.v2.nova-proof-envelope", prefix],
    )
}

fn append_fixed_bytes(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.push(value.len() as u8);
    bytes.extend_from_slice(value);
}

fn take_fixed<const N: usize>(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], RecursiveV2Error> {
    let end = cursor.checked_add(N).ok_or(RecursiveV2Error::Overflow)?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(RecursiveV2Error::Canonical)?
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    *cursor = end;
    Ok(value)
}

fn take_fixed_bytes(
    bytes: &[u8],
    cursor: &mut usize,
    expected: &[u8],
) -> Result<(), RecursiveV2Error> {
    let length = usize::from(take_u8_bundle(bytes, cursor)?);
    if length != expected.len() {
        return Err(RecursiveV2Error::Canonical);
    }
    let end = cursor
        .checked_add(length)
        .ok_or(RecursiveV2Error::Overflow)?;
    let actual = bytes.get(*cursor..end).ok_or(RecursiveV2Error::Canonical)?;
    *cursor = end;
    if actual == expected {
        Ok(())
    } else {
        Err(RecursiveV2Error::Version)
    }
}

fn take_u8_bundle(bytes: &[u8], cursor: &mut usize) -> Result<u8, RecursiveV2Error> {
    let value = *bytes.get(*cursor).ok_or(RecursiveV2Error::Canonical)?;
    *cursor = cursor.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;
    Ok(value)
}

fn take_u32_bundle(bytes: &[u8], cursor: &mut usize) -> Result<u32, RecursiveV2Error> {
    Ok(u32::from_le_bytes(take_fixed::<4>(bytes, cursor)?))
}

fn take_u64_bundle(bytes: &[u8], cursor: &mut usize) -> Result<u64, RecursiveV2Error> {
    Ok(u64::from_le_bytes(take_fixed::<8>(bytes, cursor)?))
}

#[cfg(test)]
mod tests {
    use std::{
        process::Command,
        sync::{Mutex, OnceLock},
        time::{Duration, Instant},
    };

    use super::{
        control_transition, measure_shape, nova_resource_preflight,
        nova_resource_preflight_from_shape, opcode_list, setup_public_parameters_after_preflight,
        CheckpointNovaAnchorsV2, CheckpointNovaCircuitV2, CheckpointRunningStateV2, ControlPhaseV2,
        ControlTransitionRejectionV2, NovaProofEnvelopeHeaderV2, NovaProofEnvelopeV2,
        NovaProverMaterialV2, NovaResourceLimitsV2, NovaShapeMetricsV2, NovaStepWitnessV2,
        NovaTraceRootAuthorityV2, NovaTypedSourceEventV2, NovaVerifierBundleV2,
        RecursiveAuthoritySnapshotV2, RecursiveCircuitProfileV2, RecursiveCircuitSpecV2,
        RecursiveV2Error, Scalar, VerifierBundleBindingV2, VerifierBundleHeaderV2,
        CONTROL_TRANSITION_TABLE_V2, MAX_NOVA_COMPRESSED_PROOF_BYTES_V2, NOVA_BUNDLE_SAFETY_CAP_V2,
        NOVA_PALLAS_AFFINE_WIRE_BYTES_V2, NOVA_PROOF_ENVELOPE_HEADER_BYTES_V2,
        NOVA_RESOURCE_LIMITS_V2, NOVA_VK_SAFETY_CAP_V2, NOVA_WORKER_SAFETY_CAP_V2,
        RUNNING_STATE_ARITY_V2, VERIFIER_BUNDLE_HEADER_BYTES_V2,
    };
    use nova_snark::{
        frontend::{
            gadgets::num::AllocatedNum, shape_cs::ShapeCS, test_cs::TestConstraintSystem,
            ConstraintSystem,
        },
        nova::{CompressedSNARK, RecursiveSNARK},
        provider::{ipa_pc::EvaluationEngine, PallasEngine, VestaEngine},
        spartan::ppsnark::RelaxedR1CSSNARK,
        traits::circuit::StepCircuit,
    };
    use sha2::{
        compress256,
        digest::generic_array::{typenum::U64, GenericArray},
    };
    use tempfile::TempDir;

    use crate::{
        checkpoint::{
            recursive_context::RecursiveSnapshotHandleV2,
            recursive_semantics::{
                decode_uniqueness_precommit, encode_flow_item, encode_net_merge,
                encode_uniqueness_challenge, encode_uniqueness_precommit,
            },
            recursive_trace::{
                decode_hash_control, structural_event_id, HashControlSchemaV2, HashControlStageV2,
                RecursiveTraceEventV2, RecursiveTraceOpcodeV2, RecursiveTransitionTraceSourceV2,
            },
        },
        settlement::{
            derive_settlement_root_v2, RootGeneration, ScopeFlow, ScopeFlowItem, ScopeLeafKind,
            ScopeOpKind, ScopeRootFlow, ScopeSeen,
        },
        snapshot::PrepSnapshotId,
    };

    type Snark<E> = RelaxedR1CSSNARK<E, EvaluationEngine<E>>;
    type Proof = CompressedSNARK<
        PallasEngine,
        VestaEngine,
        CheckpointNovaCircuitV2,
        Snark<PallasEngine>,
        Snark<VestaEngine>,
    >;

    fn recompute_sha256_chaining_after(block: &[u8; 64], chaining_before: [u32; 8]) -> [u32; 8] {
        let mut chaining_after = chaining_before;
        let block = GenericArray::<u8, U64>::clone_from_slice(block);
        compress256(&mut chaining_after, core::slice::from_ref(&block));
        chaining_after
    }

    const NOVA_WORKER_MARKER_V2: &str = "Z00Z_NOVA_WORKER_V2";
    const NOVA_WORKER_TIMEOUT_SECS_V2: u64 = 900;

    #[derive(Clone, Copy, Debug)]
    struct NovaWorkerReportV2 {
        peak_rss_bytes: u64,
        elapsed: Duration,
    }

    fn real_nova_proof_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn parse_peak_rss_bytes(report: &str) -> Option<u64> {
        let kibibytes = report.lines().find_map(|line| {
            line.trim_start()
                .strip_prefix("Maximum resident set size (kbytes):")?
                .trim()
                .parse::<u64>()
                .ok()
        })?;
        kibibytes.checked_mul(1024)
    }

    /// Runs one full real-Nova test in a separate address-space/time-bounded
    /// process. The child executes the unchanged test body; the parent only
    /// accepts its zero exit status plus an actual RSS measurement.
    fn run_bounded_nova_worker() -> Option<NovaWorkerReportV2> {
        if std::env::var_os(NOVA_WORKER_MARKER_V2).is_some() {
            return None;
        }

        let test_name = std::thread::current()
            .name()
            .expect("test harness names every real Nova test")
            .to_owned();
        // Cargo can unlink and replace the test binary while this parent
        // process is still live. Its PID-pinned `/proc/<pid>/exe` remains an
        // executable handle to this exact test harness, whereas `current_exe()`
        // can include the unusable ` (deleted)` suffix. It cannot be
        // `/proc/self/exe`: after `timeout` starts, "self" would be timeout.
        let test_binary = format!("/proc/{}/exe", std::process::id());
        let report = tempfile::NamedTempFile::new().expect("Nova worker time report");
        let started = Instant::now();
        let output = Command::new("/usr/bin/timeout")
            .arg("--kill-after=15s")
            .arg(format!("{NOVA_WORKER_TIMEOUT_SECS_V2}s"))
            .arg("/usr/bin/prlimit")
            .arg(format!("--as={NOVA_WORKER_SAFETY_CAP_V2}"))
            .arg("/usr/bin/time")
            .arg("-v")
            .arg("-o")
            .arg(report.path())
            .arg(test_binary)
            .arg("--exact")
            .arg(&test_name)
            .arg("--nocapture")
            .env(NOVA_WORKER_MARKER_V2, "1")
            .env("LC_ALL", "C")
            .output()
            .expect("start bounded Nova worker");
        let elapsed = started.elapsed();
        let time_report = std::fs::read_to_string(report.path()).unwrap_or_default();
        let peak_rss_bytes = parse_peak_rss_bytes(&time_report).unwrap_or_else(|| {
            panic!(
                "bounded Nova worker omitted peak RSS: status={:?}, stderr={}, time={}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
                time_report
            )
        });
        if !output.status.success() {
            panic!(
                "bounded Nova worker failed: status={:?}, stdout={}, stderr={}, time={}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
                time_report
            );
        }
        assert!(
            peak_rss_bytes <= NOVA_WORKER_SAFETY_CAP_V2,
            "bounded Nova worker exceeded its diagnostic safety cap: {peak_rss_bytes} > {NOVA_WORKER_SAFETY_CAP_V2}"
        );
        assert!(
            elapsed <= Duration::from_secs(NOVA_WORKER_TIMEOUT_SECS_V2),
            "bounded Nova worker exceeded its authority timeout: {elapsed:?}"
        );
        eprintln!(
            "bounded real Nova worker: test={test_name}, peak_rss_bytes={peak_rss_bytes}, elapsed_ms={}",
            elapsed.as_millis()
        );
        Some(NovaWorkerReportV2 {
            peak_rss_bytes,
            elapsed,
        })
    }

    #[test]
    fn verifier_source_identity_binds_nova_and_canonical_trace_owner() {
        let expected = z00z_crypto::sha256_256_role(
            z00z_crypto::CheckpointShaRole::Statement,
            &[
                b"z00z.recursive.v2.nova-owner-source",
                include_bytes!("nova.rs"),
                include_bytes!("../recursive_trace.rs"),
            ],
        );
        assert_eq!(super::source_revision_digest(), expected);
    }

    fn measured_bincode<T: serde::Serialize>(value: &T) -> Vec<u8> {
        bincode::serde::encode_to_vec(value, bincode::config::legacy())
            .expect("test payload serialization")
    }

    fn mutate_bundle_header(
        bundle: &[u8],
        mutate: impl FnOnce(&mut VerifierBundleHeaderV2),
    ) -> Vec<u8> {
        let mut header = VerifierBundleHeaderV2::decode(bundle).expect("canonical test header");
        mutate(&mut header);
        header.project_digest = super::bundle_project_digest(&header.canonical_prefix());
        let mut mutated = header.encode();
        mutated.extend_from_slice(&bundle[VERIFIER_BUNDLE_HEADER_BYTES_V2..]);
        mutated
    }

    fn assert_bundle_rejected_before_proof_decode(bundle: &[u8], binding: VerifierBundleBindingV2) {
        assert!(
            NovaVerifierBundleV2::load(bundle, binding).is_err(),
            "the bundle must reject before a proof decoder is reachable"
        );
    }

    fn mutate_proof_envelope_header(
        envelope: &[u8],
        mutate: impl FnOnce(&mut NovaProofEnvelopeHeaderV2),
    ) -> Vec<u8> {
        let mut header =
            NovaProofEnvelopeHeaderV2::decode(envelope).expect("canonical test envelope header");
        mutate(&mut header);
        header.project_digest = super::proof_envelope_project_digest(&header.canonical_prefix());
        let mut mutated = header.encode();
        mutated.extend_from_slice(&envelope[NOVA_PROOF_ENVELOPE_HEADER_BYTES_V2..]);
        mutated
    }

    /// Audited control result kept test-only and intentionally independent of
    /// the production table.  Each row is `(phase, done)` in `ControlPhaseV2`
    /// numeric order, and each column is frozen opcode order 1 through 13.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ExpectedControlEdgeV2 {
        Accept {
            next_phase: ControlPhaseV2,
            next_done: bool,
        },
        Reject(ControlTransitionRejectionV2),
    }

    const ILLEGAL: ExpectedControlEdgeV2 =
        ExpectedControlEdgeV2::Reject(ControlTransitionRejectionV2::IllegalEdge);
    const FINALIZED: ExpectedControlEdgeV2 =
        ExpectedControlEdgeV2::Reject(ControlTransitionRejectionV2::Finalized);
    const IDLE: ExpectedControlEdgeV2 = ExpectedControlEdgeV2::Accept {
        next_phase: ControlPhaseV2::Idle,
        next_done: false,
    };
    const REPLAY: ExpectedControlEdgeV2 = ExpectedControlEdgeV2::Accept {
        next_phase: ControlPhaseV2::Replay,
        next_done: false,
    };
    const PRECOMMIT: ExpectedControlEdgeV2 = ExpectedControlEdgeV2::Accept {
        next_phase: ControlPhaseV2::Precommit,
        next_done: false,
    };
    const CHALLENGE: ExpectedControlEdgeV2 = ExpectedControlEdgeV2::Accept {
        next_phase: ControlPhaseV2::Challenge,
        next_done: false,
    };
    const NET: ExpectedControlEdgeV2 = ExpectedControlEdgeV2::Accept {
        next_phase: ControlPhaseV2::Net,
        next_done: false,
    };
    const JMT: ExpectedControlEdgeV2 = ExpectedControlEdgeV2::Accept {
        next_phase: ControlPhaseV2::Jmt,
        next_done: false,
    };
    const PROMOTE: ExpectedControlEdgeV2 = ExpectedControlEdgeV2::Accept {
        next_phase: ControlPhaseV2::Promote,
        next_done: false,
    };
    const COMMIT: ExpectedControlEdgeV2 = ExpectedControlEdgeV2::Accept {
        next_phase: ControlPhaseV2::Commit,
        next_done: false,
    };
    const TRACE_CLOSURE: ExpectedControlEdgeV2 = ExpectedControlEdgeV2::Accept {
        next_phase: ControlPhaseV2::TraceClosure,
        next_done: false,
    };

    // This matrix is deliberately not generated from `CONTROL_TRANSITION_TABLE_V2`.
    // It freezes the action-15 grammar independently, including every rejected
    // phase/done/opcode tuple and all explicit hash-control self-loops.
    const EXPECTED_CONTROL_EDGE_MATRIX_V2: [[ExpectedControlEdgeV2; 13]; 18] = [
        [
            REPLAY, ILLEGAL, ILLEGAL, IDLE, IDLE, IDLE, ILLEGAL, ILLEGAL, ILLEGAL, ILLEGAL,
            ILLEGAL, ILLEGAL, ILLEGAL,
        ],
        [FINALIZED; 13],
        [
            ILLEGAL, REPLAY, REPLAY, REPLAY, REPLAY, REPLAY, PRECOMMIT, ILLEGAL, ILLEGAL, ILLEGAL,
            ILLEGAL, ILLEGAL, ILLEGAL,
        ],
        [FINALIZED; 13],
        [
            ILLEGAL, ILLEGAL, ILLEGAL, PRECOMMIT, PRECOMMIT, PRECOMMIT, ILLEGAL, CHALLENGE,
            ILLEGAL, ILLEGAL, ILLEGAL, ILLEGAL, ILLEGAL,
        ],
        [FINALIZED; 13],
        [
            ILLEGAL, ILLEGAL, ILLEGAL, CHALLENGE, CHALLENGE, CHALLENGE, ILLEGAL, ILLEGAL, NET,
            ILLEGAL, ILLEGAL, ILLEGAL, ILLEGAL,
        ],
        [FINALIZED; 13],
        [
            ILLEGAL, ILLEGAL, ILLEGAL, NET, NET, NET, ILLEGAL, ILLEGAL, ILLEGAL, JMT, ILLEGAL,
            ILLEGAL, ILLEGAL,
        ],
        [FINALIZED; 13],
        [
            ILLEGAL, ILLEGAL, ILLEGAL, JMT, JMT, JMT, ILLEGAL, ILLEGAL, ILLEGAL, JMT, PROMOTE,
            ILLEGAL, ILLEGAL,
        ],
        [FINALIZED; 13],
        [
            ILLEGAL, ILLEGAL, ILLEGAL, PROMOTE, PROMOTE, PROMOTE, ILLEGAL, ILLEGAL, ILLEGAL,
            ILLEGAL, ILLEGAL, COMMIT, ILLEGAL,
        ],
        [FINALIZED; 13],
        [
            ILLEGAL,
            ILLEGAL,
            ILLEGAL,
            COMMIT,
            COMMIT,
            COMMIT,
            ILLEGAL,
            ILLEGAL,
            ILLEGAL,
            ILLEGAL,
            ILLEGAL,
            ILLEGAL,
            TRACE_CLOSURE,
        ],
        [FINALIZED; 13],
        [
            ILLEGAL,
            ILLEGAL,
            ILLEGAL,
            TRACE_CLOSURE,
            TRACE_CLOSURE,
            TRACE_CLOSURE,
            ILLEGAL,
            ILLEGAL,
            ILLEGAL,
            ILLEGAL,
            ILLEGAL,
            ILLEGAL,
            ILLEGAL,
        ],
        [FINALIZED; 13],
    ];

    fn expected_control_edge(
        phase: ControlPhaseV2,
        done: bool,
        opcode: crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2,
    ) -> ExpectedControlEdgeV2 {
        if opcode == RecursiveTraceOpcodeV2::TraceChunk {
            return if done {
                ExpectedControlEdgeV2::Reject(ControlTransitionRejectionV2::Finalized)
            } else {
                ExpectedControlEdgeV2::Accept {
                    next_phase: phase,
                    next_done: false,
                }
            };
        }
        let state_index = (phase as usize) * 2 + usize::from(done);
        let opcode_index = usize::from(opcode as u8) - 1;
        EXPECTED_CONTROL_EDGE_MATRIX_V2[state_index][opcode_index]
    }

    fn circuit(
        anchors: CheckpointNovaAnchorsV2,
        phase: ControlPhaseV2,
        opcode: crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2,
        ordinal: u64,
    ) -> CheckpointNovaCircuitV2 {
        let witness = NovaStepWitnessV2::new(
            false,
            NovaTypedSourceEventV2::control(phase, opcode, ordinal),
        )
        .expect("test transition is legal");
        CheckpointNovaCircuitV2::new(anchors, witness)
    }

    fn source_event(
        phase: ControlPhaseV2,
        opcode: crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2,
        ordinal: u64,
        payload: &[u8],
    ) -> NovaTypedSourceEventV2 {
        let profile =
            crate::checkpoint::recursive_circuit::RecursiveCircuitProfileV2::repository_fixture();
        let object_id =
            crate::checkpoint::recursive_trace::structural_event_id(opcode, ordinal, payload);
        let source = crate::checkpoint::recursive_trace::RecursiveTraceEventV2::new(
            ordinal,
            opcode,
            object_id,
            payload.to_vec(),
            &profile,
        )
        .expect("bounded source event");
        NovaTypedSourceEventV2::from_source(phase, &source).expect("source event binding")
    }

    fn source_circuit(
        anchors: CheckpointNovaAnchorsV2,
        event: NovaTypedSourceEventV2,
    ) -> CheckpointNovaCircuitV2 {
        let witness = NovaStepWitnessV2::new(false, event).expect("legal source transition");
        CheckpointNovaCircuitV2::new(anchors, witness)
    }

    fn trace_root_after(
        prior_root: [u64; super::DIGEST_LIMBS],
        event: &NovaTypedSourceEventV2,
    ) -> [u64; super::DIGEST_LIMBS] {
        std::array::from_fn(|index| {
            prior_root[index]
                .checked_add(u64::from(event.payload_digest_limbs[index]))
                .expect("test trace root remains within u64")
        })
    }

    fn shape_of(
        circuit: &CheckpointNovaCircuitV2,
        state: &CheckpointRunningStateV2,
    ) -> (usize, usize, usize) {
        let mut cs = ShapeCS::<PallasEngine>::new();
        let inputs = state
            .scalars()
            .into_iter()
            .map(|value| AllocatedNum::alloc(&mut cs, || Ok(value)).expect("shape input alloc"))
            .collect::<Vec<_>>();
        let output = circuit
            .synthesize(&mut cs, &inputs)
            .expect("shape synthesis");
        assert_eq!(output.len(), RUNNING_STATE_ARITY_V2);
        (cs.num_constraints(), cs.num_inputs(), cs.num_aux())
    }

    fn synthesize_test(
        circuit: &CheckpointNovaCircuitV2,
        state: &CheckpointRunningStateV2,
    ) -> (TestConstraintSystem<Scalar>, Vec<AllocatedNum<Scalar>>) {
        let mut cs = TestConstraintSystem::<Scalar>::new();
        let input = state
            .scalars()
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                AllocatedNum::alloc(cs.namespace(|| format!("input_{index}")), || Ok(value))
                    .expect("test input allocation")
            })
            .collect::<Vec<_>>();
        let output = circuit.synthesize(&mut cs, &input).expect("test synthesis");
        (cs, output)
    }

    fn state_from_output(output: &[AllocatedNum<Scalar>]) -> CheckpointRunningStateV2 {
        assert_eq!(output.len(), RUNNING_STATE_ARITY_V2);
        CheckpointRunningStateV2 {
            cells: std::array::from_fn(|index| {
                super::scalar_u64(output[index].get_value().expect("concrete test output"))
            }),
        }
    }

    fn canonical_hash_control_events() -> (
        crate::checkpoint::recursive_trace::RecursiveTracePrecommitV2,
        Vec<NovaTypedSourceEventV2>,
    ) {
        canonical_hash_control_events_for_replay(
            RecursiveTraceOpcodeV2::ReplayInput,
            ScopeOpKind::Put,
        )
    }

    fn canonical_hash_control_events_for_replay(
        replay_opcode: RecursiveTraceOpcodeV2,
        replay_op_kind: ScopeOpKind,
    ) -> (
        crate::checkpoint::recursive_trace::RecursiveTracePrecommitV2,
        Vec<NovaTypedSourceEventV2>,
    ) {
        let (precommit, schedule) =
            global_trace_hash_control_fixture_for_replay(replay_opcode, replay_op_kind);
        let mut phase = ControlPhaseV2::Idle;
        let mut events = Vec::with_capacity(schedule.len());
        for event in schedule {
            let typed = NovaTypedSourceEventV2::from_source(phase, &event)
                .expect("canonical replay event has a typed Nova witness");
            phase = control_transition(phase, false, event.opcode())
                .expect("canonical replay event has a legal Nova edge")
                .next_phase;
            events.push(typed);
        }
        for index in 0..events.len().saturating_sub(1) {
            events[index] = events[index].clone().with_successor(&events[index + 1]);
        }
        let last = events.last().cloned().expect("full canonical replay");
        *events.last_mut().expect("last event") = last.clone().with_successor(&last);
        (precommit, events)
    }

    /// A real source-expander fixture for the canonical uniqueness-precommit
    /// codec.  It contains the paired replay rows whose native codec creates
    /// the precommit, so this test path exercises the same source-record bytes
    /// and not a hand-built TraceChunk witness.
    fn canonical_uniqueness_precommit_events() -> (
        crate::checkpoint::recursive_trace::RecursiveTracePrecommitV2,
        Vec<NovaTypedSourceEventV2>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
    ) {
        let profile = RecursiveCircuitProfileV2::repository_fixture();
        let snapshot_root =
            derive_settlement_root_v2(RootGeneration::SettlementV2, 7, [3; 32], [4; 32])
                .expect("test snapshot root");
        let snapshot = RecursiveSnapshotHandleV2::new(
            PrepSnapshotId::new([1; 32]),
            9,
            snapshot_root,
            5,
            1,
            [2; 32],
        )
        .expect("test snapshot");
        let flow = ScopeFlow {
            batch_id: "11".repeat(32),
            shard_id: 7,
            routing_generation: 9,
            route_table_digest: "22".repeat(32),
            items: vec![
                ScopeFlowItem {
                    tx_id: "delete-0000".to_owned(),
                    op_kind: ScopeOpKind::Delete,
                    definition_id: "33".repeat(32),
                    serial_id: 1,
                    terminal_id: "44".repeat(32),
                    leaf_family: ScopeLeafKind::Terminal,
                    first_seen: ScopeSeen {
                        definition: false,
                        serial: false,
                        object: false,
                    },
                },
                ScopeFlowItem {
                    tx_id: "put-0000".to_owned(),
                    op_kind: ScopeOpKind::Put,
                    definition_id: "55".repeat(32),
                    serial_id: 2,
                    terminal_id: "66".repeat(32),
                    leaf_family: ScopeLeafKind::Terminal,
                    first_seen: ScopeSeen {
                        definition: true,
                        serial: true,
                        object: true,
                    },
                },
            ],
            root_flow: ScopeRootFlow {
                prev_root: "77".repeat(32),
                post_root: "88".repeat(32),
            },
        };
        let precommit_payload =
            encode_uniqueness_precommit(&flow).expect("canonical uniqueness precommit payload");
        assert_eq!(
            precommit_payload.len(),
            super::UNIQUENESS_PRECOMMIT_BYTES_V2,
            "fixture must use the one canonical precommit codec width"
        );
        let precommit = decode_uniqueness_precommit(&precommit_payload)
            .expect("canonical precommit payload must decode for the canonical challenge codec");
        let challenge_payload = encode_uniqueness_challenge([0xC1; 32], precommit);
        assert_eq!(
            challenge_payload.len(),
            super::UNIQUENESS_CHALLENGE_BYTES_V2,
            "fixture must use the one canonical challenge codec width"
        );
        let challenge = challenge_payload[super::CHALLENGE_DIGEST_BYTES_START..]
            .try_into()
            .expect("canonical challenge payload contains exactly one digest");
        let net_merge_payload = encode_net_merge(precommit, challenge);
        assert_eq!(
            net_merge_payload.len(),
            super::NET_MERGE_BYTES_V2,
            "fixture must use the one canonical net merge codec width"
        );
        let begin_payload = vec![0xA5_u8; 95];
        let replay_input_payload = encode_flow_item(&flow.items[0]).expect("input payload");
        let replay_output_payload = encode_flow_item(&flow.items[1]).expect("output payload");
        let sources = [
            RecursiveTraceEventV2::new(
                0,
                RecursiveTraceOpcodeV2::BeginBlock,
                structural_event_id(RecursiveTraceOpcodeV2::BeginBlock, 0, &begin_payload),
                begin_payload,
                &profile,
            )
            .expect("begin source"),
            RecursiveTraceEventV2::new(
                1,
                RecursiveTraceOpcodeV2::ReplayInput,
                [0x44; 32],
                replay_input_payload,
                &profile,
            )
            .expect("replay input source"),
            RecursiveTraceEventV2::new(
                2,
                RecursiveTraceOpcodeV2::ReplayOutput,
                [0x66; 32],
                replay_output_payload,
                &profile,
            )
            .expect("replay output source"),
            RecursiveTraceEventV2::new(
                3,
                RecursiveTraceOpcodeV2::UniquenessPrecommit,
                structural_event_id(
                    RecursiveTraceOpcodeV2::UniquenessPrecommit,
                    3,
                    &precommit_payload,
                ),
                precommit_payload.clone(),
                &profile,
            )
            .expect("precommit source"),
            RecursiveTraceEventV2::new(
                4,
                RecursiveTraceOpcodeV2::UniquenessChallenge,
                structural_event_id(
                    RecursiveTraceOpcodeV2::UniquenessChallenge,
                    4,
                    &challenge_payload,
                ),
                challenge_payload.clone(),
                &profile,
            )
            .expect("challenge source"),
            RecursiveTraceEventV2::new(
                5,
                RecursiveTraceOpcodeV2::NetMerge,
                structural_event_id(RecursiveTraceOpcodeV2::NetMerge, 5, &net_merge_payload),
                net_merge_payload.clone(),
                &profile,
            )
            .expect("net merge source"),
        ];
        let temp = TempDir::new().expect("trace tempdir");
        let mut source =
            RecursiveTransitionTraceSourceV2::create_in(temp.path(), profile, snapshot)
                .expect("trace source");
        source
            .begin_canonical_precommit()
            .expect("open source precommit");
        for event in &sources {
            source
                .append_canonical_event(event.clone())
                .expect("append canonical source record");
        }
        let trace_precommit = source.seal_canonical_precommit().expect("seal precommit");
        let mut expanded = Vec::new();
        source
            .event_pass(|event| {
                expanded.push(event.clone());
                Ok(())
            })
            .expect("expand canonical precommit fixture");
        assert_eq!(
            source.finish(snapshot).expect("finish source"),
            trace_precommit
        );

        let mut phase = ControlPhaseV2::Idle;
        let mut events = Vec::with_capacity(expanded.len());
        for event in expanded {
            let typed = NovaTypedSourceEventV2::from_source(phase, &event)
                .expect("canonical precommit event has a typed Nova witness");
            phase = control_transition(phase, false, event.opcode())
                .expect("canonical precommit event has a legal Nova edge")
                .next_phase;
            events.push(typed);
        }
        for index in 0..events.len().saturating_sub(1) {
            events[index] = events[index].clone().with_successor(&events[index + 1]);
        }
        let last = events
            .last()
            .cloned()
            .expect("full canonical precommit trace");
        *events.last_mut().expect("last event") = last.clone().with_successor(&last);
        (
            trace_precommit,
            events,
            precommit_payload,
            challenge_payload,
            net_merge_payload,
        )
    }

    fn canonical_hash_control_fixture() -> (
        CheckpointNovaAnchorsV2,
        Vec<NovaTypedSourceEventV2>,
        Vec<CheckpointRunningStateV2>,
        CheckpointRunningStateV2,
    ) {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let (precommit, events) = canonical_hash_control_events();
        let mut trace_authority = NovaTraceRootAuthorityV2::new([0_u64; super::DIGEST_LIMBS]);
        trace_authority.expected_trace_digest = super::digest_limbs(precommit.trace_digest());
        let mut state =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
                .with_source_event(&events[0])
                .with_trace_authority(&trace_authority);
        let mut states = Vec::with_capacity(events.len());
        for (index, event) in events.iter().enumerate() {
            states.push(state.clone());
            let circuit = source_circuit(anchors.clone(), event.clone());
            let (cs, output) = synthesize_test(&circuit, &state);
            assert!(
                cs.is_satisfied(),
                "canonical control step {index} ({:?}@{}, schema {}, stage {}) violates {}; global context chain[0] {}, witness chain[0] {}, after TRACE BEGIN chain[0] {}",
                event.opcode,
                event.ordinal,
                event.hash_control.schema,
                event.hash_control.stage,
                cs.which_is_unsatisfied().unwrap_or("an unknown constraint"),
                state.cells[super::GLOBAL_BYTE_CONTEXT_START
                    + super::BYTE_CONTEXT_CHAINING_START_OFFSET],
                event.sha_compression.chaining_before[0],
                states
                    .get(1)
                    .map(|state| {
                        state.cells[super::GLOBAL_BYTE_CONTEXT_START
                            + super::BYTE_CONTEXT_CHAINING_START_OFFSET]
                    })
                    .unwrap_or_default(),
            );
            state = state_from_output(&output);
        }
        (anchors, events, states, state)
    }

    fn global_trace_hash_control_fixture() -> (
        crate::checkpoint::recursive_trace::RecursiveTracePrecommitV2,
        Vec<RecursiveTraceEventV2>,
    ) {
        global_trace_hash_control_fixture_for_replay(
            RecursiveTraceOpcodeV2::ReplayInput,
            ScopeOpKind::Put,
        )
    }

    fn global_trace_hash_control_fixture_for_replay(
        replay_opcode: RecursiveTraceOpcodeV2,
        replay_op_kind: ScopeOpKind,
    ) -> (
        crate::checkpoint::recursive_trace::RecursiveTracePrecommitV2,
        Vec<RecursiveTraceEventV2>,
    ) {
        let profile = RecursiveCircuitProfileV2::repository_fixture();
        let snapshot_root =
            derive_settlement_root_v2(RootGeneration::SettlementV2, 7, [3; 32], [4; 32])
                .expect("test snapshot root");
        let snapshot = RecursiveSnapshotHandleV2::new(
            PrepSnapshotId::new([1; 32]),
            9,
            snapshot_root,
            5,
            1,
            [2; 32],
        )
        .expect("test snapshot");
        let first_payload = vec![0xA5_u8; 95];
        let replay_terminal = [0x5A_u8; 32];
        let replay_tx_id = match replay_opcode {
            RecursiveTraceOpcodeV2::ReplayInput => "fixture-delete",
            RecursiveTraceOpcodeV2::ReplayOutput => "fixture-output",
            _ => unreachable!("the replay fixture requires a replay source opcode"),
        };
        let second_payload = encode_flow_item(&ScopeFlowItem {
            tx_id: replay_tx_id.to_owned(),
            op_kind: replay_op_kind,
            definition_id: "11".repeat(32),
            serial_id: 7,
            terminal_id: "5a".repeat(32),
            leaf_family: ScopeLeafKind::Terminal,
            first_seen: ScopeSeen {
                definition: false,
                serial: false,
                object: false,
            },
        })
        .expect("canonical replay payload");
        let sources = [
            RecursiveTraceEventV2::new(
                0,
                RecursiveTraceOpcodeV2::BeginBlock,
                structural_event_id(RecursiveTraceOpcodeV2::BeginBlock, 0, &first_payload),
                first_payload,
                &profile,
            )
            .expect("first source"),
            RecursiveTraceEventV2::new(1, replay_opcode, replay_terminal, second_payload, &profile)
                .expect("second source"),
        ];
        let temp = TempDir::new().expect("trace tempdir");
        let mut source =
            RecursiveTransitionTraceSourceV2::create_in(temp.path(), profile, snapshot)
                .expect("trace source");
        source
            .begin_canonical_precommit()
            .expect("open source precommit");
        for event in &sources {
            source
                .append_canonical_event(event.clone())
                .expect("append source record");
        }
        let precommit = source.seal_canonical_precommit().expect("seal precommit");
        let mut expanded = Vec::new();
        source
            .event_pass(|event| {
                expanded.push(event.clone());
                Ok(())
            })
            .expect("global control replay");
        assert_eq!(source.finish(snapshot).expect("finish source"), precommit);

        let chunk_controls = expanded
            .iter()
            .filter(|event| event.opcode() == RecursiveTraceOpcodeV2::TraceChunk)
            .count();
        assert!(
            chunk_controls >= 2,
            "every source record has live canonical chunks"
        );
        let first_chunk = expanded
            .iter()
            .find(|event| event.opcode() == RecursiveTraceOpcodeV2::TraceChunk)
            .expect("derived schedule has a canonical byte feeder");
        let parsed_chunk = NovaTypedSourceEventV2::from_source(ControlPhaseV2::Replay, first_chunk)
            .expect("the private chunk witness must have the fixed-width canonical shape");
        assert!(
            NovaStepWitnessV2::new(false, parsed_chunk).is_ok(),
            "TraceChunk must have its explicit constrained feeder edge"
        );

        (precommit, expanded)
    }

    fn assert_unsatisfied_hash_control(
        anchors: CheckpointNovaAnchorsV2,
        state: &CheckpointRunningStateV2,
        event: NovaTypedSourceEventV2,
        label: &str,
    ) {
        let circuit = source_circuit(anchors, event);
        let (cs, _) = synthesize_test(&circuit, state);
        assert!(
            !cs.is_satisfied(),
            "hash-control mutation survived: {label}"
        );
    }

    /// Execute one deliberately malformed row and the remaining canonical
    /// schedule.  A feeder row alone may legitimately update a byte-context;
    /// its soundness is established by the first later compression row that
    /// must consume the altered context.  This helper makes that downstream
    /// R1CS failure explicit instead of treating evaluator rejection or a
    /// payload digest as byte-to-block evidence.
    fn first_schedule_failure_after_mutation(
        anchors: CheckpointNovaAnchorsV2,
        events: &[NovaTypedSourceEventV2],
        states: &[CheckpointRunningStateV2],
        mutation_index: usize,
        mutation: NovaTypedSourceEventV2,
        label: &str,
    ) -> String {
        let circuit = source_circuit(anchors.clone(), mutation);
        let (cs, output) = synthesize_test(&circuit, &states[mutation_index]);
        if !cs.is_satisfied() {
            return cs
                .which_is_unsatisfied()
                .unwrap_or("mutated feeder row is unsatisfied")
                .to_owned();
        }

        let mut state = state_from_output(&output);
        for event in events.iter().skip(mutation_index + 1) {
            let circuit = source_circuit(anchors.clone(), event.clone());
            let (cs, output) = synthesize_test(&circuit, &state);
            if !cs.is_satisfied() {
                return cs
                    .which_is_unsatisfied()
                    .unwrap_or("mutated schedule is unsatisfied")
                    .to_owned();
            }
            state = state_from_output(&output);
        }

        panic!("hash-control mutation survived the remaining canonical schedule: {label}");
    }

    #[test]
    fn canonical_hash_controls_bind_the_fixed_fips_schedule() {
        let (anchors, events, states, final_state) = canonical_hash_control_fixture();
        let shapes = events
            .iter()
            .zip(&states)
            .map(|(event, state)| shape_of(&source_circuit(anchors.clone(), event.clone()), state))
            .collect::<Vec<_>>();
        assert!(shapes.windows(2).all(|pair| pair[0] == pair[1]));
        assert_eq!(final_state.cells[super::ORDINAL_CELL], events.len() as u64);
        assert_eq!(final_state.cells[super::SOURCE_TRACE_ORDINAL_CELL], 2);
        for index in super::COUNTERS_START..super::SHA_END {
            assert_eq!(final_state.cells[index], 0, "END_HASH clears cell {index}");
        }
        assert_eq!(
            final_state.cells[super::REPLAY_MODE_CELL],
            super::ReplayModeV2::Inputs as u64,
            "the fixture contains no ReplayOutput and must remain in the input prefix"
        );
        assert_eq!(final_state.cells[super::REPLAY_INPUT_COUNT_CELL], 1);
        assert_eq!(final_state.cells[super::REPLAY_OUTPUT_COUNT_CELL], 0);
    }

    #[test]
    fn replay_grammar_rejects_input_after_output_prefix() {
        let (anchors, events, states, _) = canonical_hash_control_fixture();
        let replay_input = events
            .iter()
            .position(|event| event.opcode == RecursiveTraceOpcodeV2::ReplayInput)
            .expect("canonical schedule contains a replay input");
        let mut state = states[replay_input].clone();
        state.cells[super::REPLAY_MODE_CELL] = super::ReplayModeV2::Outputs as u64;
        let circuit = source_circuit(anchors, events[replay_input].clone());
        let (cs, _) = synthesize_test(&circuit, &state);
        assert!(
            !cs.is_satisfied(),
            "a ReplayInput cannot appear after the replay output prefix"
        );
        assert!(
            cs.which_is_unsatisfied()
                .is_some_and(|constraint| constraint
                    .starts_with("replay_grammar/input_requires_inputs_mode/")),
            "the replay-order mutation must reach its direct R1CS gate, got {:?}",
            cs.which_is_unsatisfied()
        );
    }

    #[test]
    fn test_op_kind_replay_independent() {
        let (anchors, events, states, _) = canonical_hash_control_fixture();
        let replay_chunk = events
            .iter()
            .position(|event| {
                event.opcode == RecursiveTraceOpcodeV2::TraceChunk
                    && event.trace_chunk.source_ordinal == 1
                    && event.trace_chunk.chunk_ordinal == 0
            })
            .expect("canonical replay item has its first chunk");
        assert_eq!(
            events[replay_chunk].trace_chunk.bytes[super::TRACE_EVENT_HEADER_BYTES_V2],
            1,
            "ReplayInput may carry the canonical Put item"
        );

        let mut malformed = events[replay_chunk].clone();
        malformed.trace_chunk.bytes[super::TRACE_EVENT_HEADER_BYTES_V2] = 3;
        let failure = first_schedule_failure_after_mutation(
            anchors,
            &events,
            &states,
            replay_chunk,
            malformed,
            "noncanonical CanonicalFlowItemV2 op_kind",
        );
        assert!(
            failure.contains("replay_payload") && failure.contains("op_kind_canonical"),
            "the noncanonical payload op-kind mutation must reach its direct R1CS gate, got {failure}"
        );
    }

    #[test]
    fn test_output_delete_trace() {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let (precommit, events) = canonical_hash_control_events_for_replay(
            RecursiveTraceOpcodeV2::ReplayOutput,
            ScopeOpKind::Delete,
        );
        let mut trace_authority = NovaTraceRootAuthorityV2::new([0_u64; super::DIGEST_LIMBS]);
        trace_authority.expected_trace_digest = super::digest_limbs(precommit.trace_digest());
        let mut state =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
                .with_source_event(&events[0])
                .with_trace_authority(&trace_authority);
        let replay_chunk = events
            .iter()
            .position(|event| {
                event.opcode == RecursiveTraceOpcodeV2::TraceChunk
                    && event.trace_chunk.source_ordinal == 1
                    && event.trace_chunk.chunk_ordinal == 0
            })
            .expect("canonical replay output has its first chunk");
        assert_eq!(
            events[replay_chunk].trace_chunk.bytes[super::TRACE_EVENT_HEADER_BYTES_V2],
            2,
            "ReplayOutput may carry the canonical Delete item"
        );
        for (index, event) in events.iter().enumerate() {
            let circuit = source_circuit(anchors.clone(), event.clone());
            let (cs, output) = synthesize_test(&circuit, &state);
            assert!(
                cs.is_satisfied(),
                "canonical ReplayOutput/Delete schedule {index} violates {}",
                cs.which_is_unsatisfied().unwrap_or("an unknown constraint"),
            );
            state = state_from_output(&output);
        }
    }

    #[test]
    fn replay_payload_terminal_is_bound_to_the_source_object_id_in_r1cs() {
        let (anchors, events, states, _) = canonical_hash_control_fixture();
        let replay_chunk = events
            .iter()
            .position(|event| {
                event.opcode == RecursiveTraceOpcodeV2::TraceChunk
                    && event.trace_chunk.source_ordinal == 1
                    && event.trace_chunk.chunk_ordinal == 2
            })
            .expect("canonical replay item has its terminal-ID chunk");
        assert_eq!(
            events[replay_chunk].trace_chunk.bytes[6], b'5',
            "fixture must put the first terminal-ID hexadecimal byte at the fixed chunk offset"
        );

        let mut malformed = events[replay_chunk].clone();
        malformed.trace_chunk.bytes[6] = b'6';
        let failure = first_schedule_failure_after_mutation(
            anchors,
            &events,
            &states,
            replay_chunk,
            malformed,
            "terminal ID differs from the canonical source object ID",
        );
        assert!(
            failure.contains("replay_payload") && failure.contains("terminal_matches_object"),
            "the terminal-ID mutation must fail in the R1CS source-object relation, got {failure}"
        );
    }

    #[test]
    fn replay_output_switches_the_exact_canonical_replay_prefix() {
        let (anchors, events, states, _) = canonical_hash_control_fixture();
        let replay_input = events
            .iter()
            .position(|event| event.opcode == RecursiveTraceOpcodeV2::ReplayInput)
            .expect("canonical schedule contains the input-prefix state");
        let replay_output = source_event(
            ControlPhaseV2::Replay,
            RecursiveTraceOpcodeV2::ReplayOutput,
            events[replay_input].ordinal,
            b"replay-output-prefix-probe",
        );
        let state = states[replay_input]
            .clone()
            .with_source_event(&replay_output);
        let circuit = source_circuit(anchors, replay_output);
        let (cs, output) = synthesize_test(&circuit, &state);
        assert!(
            cs.is_satisfied(),
            "a first ReplayOutput must be a legal replay-prefix transition: {}",
            cs.which_is_unsatisfied().unwrap_or("unknown constraint")
        );
        let output = state_from_output(&output);
        assert_eq!(
            output.cells[super::REPLAY_MODE_CELL],
            super::ReplayModeV2::Outputs as u64
        );
        assert_eq!(output.cells[super::REPLAY_INPUT_COUNT_CELL], 0);
        assert_eq!(output.cells[super::REPLAY_OUTPUT_COUNT_CELL], 1);
    }

    #[test]
    fn precommit_rejects_an_unpaired_replay_set() {
        let (anchors, events, states, _) = canonical_hash_control_fixture();
        let replay_input = events
            .iter()
            .position(|event| event.opcode == RecursiveTraceOpcodeV2::ReplayInput)
            .expect("canonical schedule contains the input-prefix state");
        let precommit = source_event(
            ControlPhaseV2::Replay,
            RecursiveTraceOpcodeV2::UniquenessPrecommit,
            events[replay_input].ordinal,
            b"unpaired-replay-precommit",
        );
        let mut state = states[replay_input].clone().with_source_event(&precommit);
        state.cells[super::REPLAY_MODE_CELL] = super::ReplayModeV2::Outputs as u64;
        state.cells[super::REPLAY_INPUT_COUNT_CELL] = 1;
        let circuit = source_circuit(anchors, precommit);
        let (cs, _) = synthesize_test(&circuit, &state);
        assert!(
            !cs.is_satisfied(),
            "precommit cannot bind a spent set without an output set"
        );
        assert!(
            cs.which_is_unsatisfied()
                .is_some_and(|constraint| constraint.starts_with(
                    "replay_grammar/precommit_requires_jointly_empty_or_nonempty_sets"
                )),
            "the unpaired set must reach the replay-cardinality R1CS gate, got {:?}",
            cs.which_is_unsatisfied()
        );
    }

    #[test]
    fn uniqueness_precommit_payload_is_streamed_and_count_bound_in_r1cs() {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let (precommit, events, payload, _, _) = canonical_uniqueness_precommit_events();
        let mut trace_authority = NovaTraceRootAuthorityV2::new([0_u64; super::DIGEST_LIMBS]);
        trace_authority.expected_trace_digest = super::digest_limbs(precommit.trace_digest());
        let mut state =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
                .with_source_event(&events[0])
                .with_trace_authority(&trace_authority);
        let mut states = Vec::with_capacity(events.len());
        for (index, event) in events.iter().enumerate() {
            states.push(state.clone());
            let circuit = source_circuit(anchors.clone(), event.clone());
            let (cs, output) = synthesize_test(&circuit, &state);
            assert!(
                cs.is_satisfied(),
                "canonical uniqueness precommit step {index} violates {}",
                cs.which_is_unsatisfied().unwrap_or("an unknown constraint"),
            );
            state = state_from_output(&output);
        }
        let precommit_end = events
            .iter()
            .position(|event| {
                event.opcode == RecursiveTraceOpcodeV2::EndHash
                    && event.hash_control.schema == HashControlSchemaV2::SourceRecord as u8
                    && event.hash_control.source_ordinal == 3
            })
            .expect("canonical precommit source has an END_HASH");
        let precommit_chunk = events
            .iter()
            .position(|event| {
                event.opcode == RecursiveTraceOpcodeV2::TraceChunk
                    && event.trace_chunk.source_ordinal == 3
                    && event.trace_chunk.chunk_ordinal == 0
            })
            .expect("canonical precommit source has its first TraceChunk");
        assert_eq!(
            events[precommit_chunk].trace_chunk.bytes[super::TRACE_EVENT_HEADER_BYTES_V2],
            1,
            "fixture must place the codec version at the first payload byte",
        );
        assert_eq!(
            states[precommit_end].cells[super::PRECOMMIT_SPENT_COUNT_LIMB_START],
            1
        );
        assert_eq!(
            states[precommit_end].cells[super::PRECOMMIT_SPENT_COUNT_LIMB_START + 1],
            0
        );
        assert_eq!(
            states[precommit_end].cells[super::PRECOMMIT_OUTPUT_COUNT_LIMB_START],
            1
        );
        assert_eq!(
            states[precommit_end].cells[super::PRECOMMIT_OUTPUT_COUNT_LIMB_START + 1],
            0
        );
        for (index, bytes) in payload[super::PRECOMMIT_DIGEST_BYTES_START..]
            .chunks_exact(2)
            .enumerate()
        {
            assert_eq!(
                states[precommit_end].cells[super::PRECOMMIT_DIGEST_LIMB_START + index],
                u64::from(u16::from_le_bytes([bytes[0], bytes[1]])),
                "precommit digest limb {index} must come from its canonical payload bytes",
            );
        }

        let mut malformed_version = events[precommit_chunk].clone();
        malformed_version.trace_chunk.bytes[super::TRACE_EVENT_HEADER_BYTES_V2] ^= 1;
        let version_failure = first_schedule_failure_after_mutation(
            anchors.clone(),
            &events,
            &states,
            precommit_chunk,
            malformed_version,
            "uniqueness precommit codec version",
        );
        assert!(
            version_failure.contains("uniqueness_precommit_payload")
                && version_failure.contains("version"),
            "the precommit version mutation must fail in the R1CS byte parser, got {version_failure}",
        );

        let mut malformed_count_state = states[precommit_end].clone();
        malformed_count_state.cells[super::PRECOMMIT_SPENT_COUNT_LIMB_START] = 2;
        let circuit = source_circuit(anchors, events[precommit_end].clone());
        let (cs, _) = synthesize_test(&circuit, &malformed_count_state);
        assert!(
            !cs.is_satisfied(),
            "a precommit count that differs from replay cardinality must reject",
        );
        assert!(
            cs.which_is_unsatisfied()
                .is_some_and(|constraint| constraint.starts_with(
                    "uniqueness_precommit_payload/precommit_spent_count_matches_replay_input_count"
                )),
            "the count mutation must reach the direct precommit/replay R1CS gate, got {:?}",
            cs.which_is_unsatisfied(),
        );
    }

    #[test]
    fn uniqueness_challenge_payload_binds_precommit_bytes_in_r1cs() {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let (precommit, events, precommit_payload, challenge_payload, _) =
            canonical_uniqueness_precommit_events();
        let mut trace_authority = NovaTraceRootAuthorityV2::new([0_u64; super::DIGEST_LIMBS]);
        trace_authority.expected_trace_digest = super::digest_limbs(precommit.trace_digest());
        let mut state =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
                .with_source_event(&events[0])
                .with_trace_authority(&trace_authority);
        let mut states = Vec::with_capacity(events.len());
        for (index, event) in events.iter().enumerate() {
            states.push(state.clone());
            let circuit = source_circuit(anchors.clone(), event.clone());
            let (cs, output) = synthesize_test(&circuit, &state);
            assert!(
                cs.is_satisfied(),
                "canonical uniqueness challenge step {index} violates {}",
                cs.which_is_unsatisfied().unwrap_or("an unknown constraint"),
            );
            state = state_from_output(&output);
        }

        let challenge_end = events
            .iter()
            .position(|event| {
                event.opcode == RecursiveTraceOpcodeV2::EndHash
                    && event.hash_control.schema == HashControlSchemaV2::SourceRecord as u8
                    && event.hash_control.source_ordinal == 4
            })
            .expect("canonical challenge source has an END_HASH");
        let challenge_chunk = events
            .iter()
            .position(|event| {
                event.opcode == RecursiveTraceOpcodeV2::TraceChunk
                    && event.trace_chunk.source_ordinal == 4
                    && event.trace_chunk.chunk_ordinal == 0
            })
            .expect("canonical challenge source has its first TraceChunk");
        assert_eq!(
            events[challenge_chunk].trace_chunk.bytes[super::TRACE_EVENT_HEADER_BYTES_V2],
            super::UNIQUENESS_PRECOMMIT_VERSION_V2,
            "fixture must place the canonical codec version at the challenge payload start",
        );
        assert_eq!(
            &challenge_payload[1..super::CHALLENGE_DIGEST_BYTES_START],
            &precommit_payload[super::PRECOMMIT_DIGEST_BYTES_START + 32 * 4..],
            "the fixture must carry the canonical precommit digest, not a second value",
        );
        for (index, bytes) in challenge_payload[super::CHALLENGE_DIGEST_BYTES_START..]
            .chunks_exact(2)
            .enumerate()
        {
            assert_eq!(
                states[challenge_end].cells[super::CHALLENGE_DIGEST_LIMB_START + index],
                u64::from(u16::from_le_bytes([bytes[0], bytes[1]])),
                "challenge digest limb {index} must come from its canonical payload bytes",
            );
        }

        let mut malformed_version = events[challenge_chunk].clone();
        malformed_version.trace_chunk.bytes[super::TRACE_EVENT_HEADER_BYTES_V2] ^= 1;
        let version_failure = first_schedule_failure_after_mutation(
            anchors.clone(),
            &events,
            &states,
            challenge_chunk,
            malformed_version,
            "uniqueness challenge codec version",
        );
        assert!(
            version_failure.contains("uniqueness_challenge_payload")
                && version_failure.contains("version"),
            "the challenge version mutation must fail in the R1CS byte parser, got {version_failure}",
        );

        let mut malformed_precommit = events[challenge_chunk].clone();
        malformed_precommit.trace_chunk.bytes[super::TRACE_EVENT_HEADER_BYTES_V2 + 1] ^= 1;
        let circuit = source_circuit(anchors, malformed_precommit);
        let (cs, _) = synthesize_test(&circuit, &states[challenge_chunk]);
        assert!(
            !cs.is_satisfied(),
            "a challenge precommit byte that differs from the authenticated precommit must reject",
        );
        assert!(
            cs.which_is_unsatisfied()
                .is_some_and(|constraint| constraint
                    .contains("uniqueness_challenge_payload/byte_")
                    && constraint.contains("committed_precommit_limb_0_matches")),
            "the precommit mutation must reach the direct challenge/precommit R1CS gate, got {:?}",
            cs.which_is_unsatisfied(),
        );
    }

    #[test]
    fn net_merge_payload_is_streamed_from_canonical_source_bytes_in_r1cs() {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let (precommit, events, _, _, net_merge_payload) = canonical_uniqueness_precommit_events();
        let mut trace_authority = NovaTraceRootAuthorityV2::new([0_u64; super::DIGEST_LIMBS]);
        trace_authority.expected_trace_digest = super::digest_limbs(precommit.trace_digest());
        let mut state =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
                .with_source_event(&events[0])
                .with_trace_authority(&trace_authority);
        let mut states = Vec::with_capacity(events.len());
        for (index, event) in events.iter().enumerate() {
            states.push(state.clone());
            let circuit = source_circuit(anchors.clone(), event.clone());
            let (cs, output) = synthesize_test(&circuit, &state);
            assert!(
                cs.is_satisfied(),
                "canonical net merge step {index} violates {}",
                cs.which_is_unsatisfied().unwrap_or("an unknown constraint"),
            );
            state = state_from_output(&output);
        }

        let net_end = events
            .iter()
            .position(|event| {
                event.opcode == RecursiveTraceOpcodeV2::EndHash
                    && event.hash_control.schema == HashControlSchemaV2::SourceRecord as u8
                    && event.hash_control.source_ordinal == 5
            })
            .expect("canonical net merge source has an END_HASH");
        let net_chunk = events
            .iter()
            .position(|event| {
                event.opcode == RecursiveTraceOpcodeV2::TraceChunk
                    && event.trace_chunk.source_ordinal == 5
                    && event.trace_chunk.chunk_ordinal == 0
            })
            .expect("canonical net merge source has its first TraceChunk");
        assert_eq!(
            events[net_chunk].trace_chunk.bytes[super::TRACE_EVENT_HEADER_BYTES_V2],
            super::UNIQUENESS_PRECOMMIT_VERSION_V2,
            "fixture must place the canonical codec version at the net merge payload start",
        );
        for (index, bytes) in net_merge_payload[super::NET_DIGEST_BYTES_START..]
            .chunks_exact(2)
            .enumerate()
        {
            assert_eq!(
                states[net_end].cells[super::NET_DIGEST_LIMB_START + index],
                u64::from(u16::from_le_bytes([bytes[0], bytes[1]])),
                "net merge digest limb {index} must come from its canonical payload bytes",
            );
        }

        let mut malformed_version = events[net_chunk].clone();
        malformed_version.trace_chunk.bytes[super::TRACE_EVENT_HEADER_BYTES_V2] ^= 1;
        let version_failure = first_schedule_failure_after_mutation(
            anchors,
            &events,
            &states,
            net_chunk,
            malformed_version,
            "net merge codec version",
        );
        assert!(
            version_failure.contains("net_merge_payload") && version_failure.contains("version"),
            "the net merge version mutation must fail in the R1CS byte parser, got {version_failure}",
        );
    }

    #[test]
    fn canonical_hash_controls_reject_binding_and_order_mutations() {
        use crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2;

        let (anchors, events, states, _) = canonical_hash_control_fixture();
        let begin = events
            .iter()
            .position(|event| event.opcode == RecursiveTraceOpcodeV2::BeginHash)
            .expect("begin event");
        let block = events
            .iter()
            .position(|event| event.opcode == RecursiveTraceOpcodeV2::ShaBlock)
            .expect("block event");
        let end = events
            .iter()
            .position(|event| event.opcode == RecursiveTraceOpcodeV2::EndHash)
            .expect("end event");

        let mut source_hash = events[begin].clone();
        source_hash.hash_control.source_hash[0] ^= 1;
        assert_unsatisfied_hash_control(
            anchors.clone(),
            &states[begin],
            source_hash,
            "source hash",
        );

        let mut role = events[block].clone();
        role.hash_control.role ^= 1;
        assert_unsatisfied_hash_control(anchors.clone(), &states[block], role, "role");

        let mut message_length = events[block].clone();
        message_length.hash_control.message_bytes ^= 1;
        assert_unsatisfied_hash_control(
            anchors.clone(),
            &states[block],
            message_length,
            "role-framed message length",
        );

        let mut count = events[block].clone();
        count.hash_control.block_count ^= 1;
        assert_unsatisfied_hash_control(anchors.clone(), &states[block], count, "block count");

        let mut index = events[block].clone();
        index.hash_control.block_index += 1;
        assert_unsatisfied_hash_control(anchors.clone(), &states[block], index, "block index");

        let mut offset = events[block].clone();
        offset.hash_control.byte_offset += 1;
        assert_unsatisfied_hash_control(anchors.clone(), &states[block], offset, "block offset");

        let mut final_flag = events[block].clone();
        final_flag.hash_control.final_block = !final_flag.hash_control.final_block;
        assert_unsatisfied_hash_control(anchors.clone(), &states[block], final_flag, "final flag");

        let mut block_bytes = events[block].clone();
        block_bytes.sha_compression.block[0] ^= 1;
        assert_unsatisfied_hash_control(
            anchors.clone(),
            &states[block],
            block_bytes,
            "block bytes",
        );

        let mut chaining = events[block].clone();
        chaining.sha_compression.chaining_before[0] ^= 1;
        assert_unsatisfied_hash_control(
            anchors.clone(),
            &states[block],
            chaining,
            "chaining state",
        );

        assert_unsatisfied_hash_control(
            anchors,
            &states[begin + 1],
            events[end].clone(),
            "END_HASH before the required block schedule",
        );
    }

    #[test]
    fn trace_chunk_payload_reaches_the_constrained_source_and_global_sha_contexts() {
        use crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2;

        let (anchors, events, states, _) = canonical_hash_control_fixture();
        let first_chunk = events
            .iter()
            .position(|event| event.opcode == RecursiveTraceOpcodeV2::TraceChunk)
            .expect("canonical schedule has a source byte feeder");
        let first_payload_byte = super::TRACE_EVENT_HEADER_BYTES_V2;
        assert!(
            events[first_chunk].trace_chunk.byte_count as usize > first_payload_byte,
            "fixture must put a canonical payload byte in its first TraceChunk"
        );

        // This leaves all outer event/hash-control metadata untouched.  The
        // altered byte is admitted by the feeder row, then must make the next
        // selected source/global compression input inconsistent with its
        // already-derived FIPS witness.
        let mut payload = events[first_chunk].clone();
        payload.trace_chunk.bytes[first_payload_byte] ^= 1;
        let failure = first_schedule_failure_after_mutation(
            anchors.clone(),
            &events,
            &states,
            first_chunk,
            payload,
            "canonical TraceChunk payload byte",
        );
        assert!(
            failure.starts_with("hash_control_schedule/source_block_context/")
                || failure.starts_with("hash_control_schedule/trace_block_context/")
                || failure.starts_with("sha_compression_lane/"),
            "chunk payload must reach a selected R1CS compression relation, got {failure}"
        );

        let first_source_ordinal = events[first_chunk].trace_chunk.source_ordinal;
        let final_chunk = events
            .iter()
            .enumerate()
            .filter(|(_, event)| {
                event.opcode == RecursiveTraceOpcodeV2::TraceChunk
                    && event.trace_chunk.source_ordinal == first_source_ordinal
            })
            .max_by_key(|(_, event)| event.trace_chunk.chunk_ordinal)
            .map(|(index, _)| index)
            .expect("canonical source has a final TraceChunk");
        let final_count = events[final_chunk].trace_chunk.byte_count as usize;
        assert!(
            final_count < super::TRACE_CANONICAL_CHUNK_BYTES_V2,
            "fixture must exercise a short final canonical chunk"
        );

        let mut short_count = events[final_chunk].clone();
        short_count.trace_chunk.byte_count += 1;
        let failure = first_schedule_failure_after_mutation(
            anchors.clone(),
            &events,
            &states,
            final_chunk,
            short_count,
            "short final TraceChunk byte count",
        );
        assert!(
            failure.starts_with("hash_control_schedule/trace_chunk_source_final_length")
                || failure.starts_with("hash_control_schedule/source_block_context/")
                || failure.starts_with("sha_compression_lane/"),
            "short final count must fail in the source byte/FIPS relation, got {failure}"
        );

        let mut zero_tail = events[final_chunk].clone();
        zero_tail.trace_chunk.bytes[final_count] = 1;
        let failure = first_schedule_failure_after_mutation(
            anchors.clone(),
            &events,
            &states,
            final_chunk,
            zero_tail,
            "TraceChunk zero tail",
        );
        assert!(
            failure.starts_with("trace_chunk/trace_chunk_zero_tail_"),
            "zero-tail mutation must fail in its R1CS tail gate, got {failure}"
        );

        let mut source_ordinal = events[first_chunk].clone();
        source_ordinal.trace_chunk.source_ordinal += 1;
        let failure = first_schedule_failure_after_mutation(
            anchors.clone(),
            &events,
            &states,
            first_chunk,
            source_ordinal,
            "TraceChunk source ordinal",
        );
        assert!(
            failure.starts_with("hash_control_schedule/trace_chunk_source_ordinal"),
            "source-ordinal mutation must fail in its R1CS chunk gate, got {failure}"
        );

        // The feeder is not merely source-local: mutate the first dynamic
        // compression witness of each context after a live chunk. Both must
        // fail in the constrained context-byte/FIPS lane, never at a native
        // digest or a later terminal comparison. A satisfied R1CS may report
        // either linked equation first, so do not make the test depend on the
        // solver's diagnostic ordering.
        for (name, schema, namespace) in [
            (
                "source",
                HashControlSchemaV2::SourceRecord as u8,
                "source_block_context/source_block_full_block_byte_",
            ),
            (
                "global",
                HashControlSchemaV2::TracePrecommit as u8,
                "trace_block_context/trace_block_full_block_byte_",
            ),
        ] {
            let block_index = events
                .iter()
                .enumerate()
                .skip(first_chunk + 1)
                .find_map(|(index, event)| {
                    (event.opcode == RecursiveTraceOpcodeV2::ShaBlock
                        && event.hash_control.schema == schema
                        && event.hash_control.block_index > 0)
                        .then_some(index)
                })
                .expect("canonical chunk must make a dynamic SHA block available");
            let mut block = events[block_index].clone();
            block.sha_compression.block[0] ^= 1;
            block.sha_compression.chaining_after = recompute_sha256_chaining_after(
                &block.sha_compression.block,
                block.sha_compression.chaining_before,
            );
            let circuit = source_circuit(anchors.clone(), block);
            let (cs, _) = synthesize_test(&circuit, &states[block_index]);
            assert!(
                !cs.is_satisfied(),
                "{name} dynamic SHA witness must not bypass its byte context"
            );
            let failure = cs.which_is_unsatisfied().unwrap_or("unknown R1CS failure");
            assert!(
                failure.starts_with(&format!("hash_control_schedule/{namespace}"))
                    || failure.starts_with("sha_compression_lane/"),
                "{name} dynamic SHA witness must fail in its constrained byte/FIPS lane, got {failure}"
            );
        }
    }

    #[test]
    fn global_trace_hash_controls_share_live_chunks() {
        let (precommit, events) = global_trace_hash_control_fixture();
        let decoded = events
            .iter()
            .enumerate()
            .filter_map(|(index, event)| {
                let control = decode_hash_control(event).ok()?;
                (control.schema == HashControlSchemaV2::TracePrecommit).then_some((index, control))
            })
            .collect::<Vec<_>>();
        let (begin_index, begin) = decoded.first().expect("global BEGIN_HASH");
        let (end_index, end) = decoded.last().expect("global END_HASH");
        assert_eq!(
            *begin_index, 0,
            "global context begins before source replay"
        );
        assert_eq!(begin.stage, HashControlStageV2::Begin);
        assert_eq!(end.stage, HashControlStageV2::End);
        assert_eq!(end.binding, precommit.trace_digest());
        assert!(
            decoded.iter().any(|(index, control)| {
                control.stage == HashControlStageV2::Block
                    && *index < *end_index
                    && events[..*index]
                        .iter()
                        .any(|event| event.opcode() == RecursiveTraceOpcodeV2::TraceChunk)
            }),
            "global compression must be emitted from live canonical chunks"
        );
        let last_local_end = events
            .iter()
            .enumerate()
            .filter_map(|(index, event)| {
                let control = decode_hash_control(event).ok()?;
                (control.schema == HashControlSchemaV2::SourceRecord
                    && control.stage == HashControlStageV2::End)
                    .then_some(index)
            })
            .max()
            .expect("source END_HASH controls");
        assert!(
            decoded.iter().any(
                |(index, control)| control.stage == HashControlStageV2::Block
                    && *index < last_local_end
            ),
            "global blocks cannot be terminal-only after the final source schedule"
        );
        for (_, control) in decoded {
            let trace = control.trace.expect("tagged trace binding");
            assert_eq!(control.binding, precommit.trace_digest());
            assert_eq!(trace.event_count, precommit.event_count());
            assert_eq!(trace.byte_count, precommit.byte_count());
            assert_eq!(trace.bit_length, control.message_bytes * 8);
            assert!(trace.eof);
            if let Some(block) = control.block {
                assert!(block.verifies_transition());
            }
        }
    }

    #[test]
    fn hash_control_shape_metrics_cover_the_canonical_schedule() {
        let (anchors, events, states, _) = canonical_hash_control_fixture();
        let circuit = source_circuit(anchors, events[0].clone());
        let state = &states[0];
        let metrics = measure_shape(&circuit, &state).expect("shape metrics");
        eprintln!(
            "fixed SHA lane ShapeCS metrics: constraints={}, inputs={}, auxiliary={}, nonzeros={}",
            metrics.constraints, metrics.inputs, metrics.auxiliaries, metrics.nonzeros
        );
        assert_eq!(metrics.constraints, 275_561);
        assert_eq!(metrics.inputs, 1);
        assert_eq!(metrics.auxiliaries, 201_832);
        assert_eq!(metrics.nonzeros, 1_047_141);
    }

    #[test]
    fn sha_lane_resource_preflight_uses_pinned_wire_and_pedersen_sizes() {
        let (anchors, events, states, _) = canonical_hash_control_fixture();
        let circuit = source_circuit(anchors, events[0].clone());
        let plan = nova_resource_preflight(&circuit, &states[0])
            .expect("current canonical control shape fits preflight");

        assert_eq!(
            measured_bincode(&super::pallas::Affine::default()).len(),
            NOVA_PALLAS_AFFINE_WIRE_BYTES_V2 as usize,
            "legacy bincode must retain the pinned Pallas affine wire width"
        );
        assert_eq!(std::mem::size_of::<super::pallas::Affine>(), 64);
        assert_eq!(std::mem::size_of::<super::pallas::Point>(), 96);
        assert_eq!(plan.shape.constraints, 275_561);
        assert_eq!(plan.shape.auxiliaries, 201_832);
        assert_eq!(plan.shape.nonzeros, 1_047_141);
        assert_eq!(plan.ck_floor, 1_047_141);
        assert_eq!(plan.generator_count, 1_048_577);
        assert_eq!(plan.pp_payload_lower_bound, 82_053_712);
        assert_eq!(plan.vk_payload_lower_bound, 33_554_464);
        assert_eq!(plan.bundle_lower_bound, 33_554_918);
        assert_eq!(plan.pedersen_rss_lower_bound, 201_326_784);
        assert!(plan.pp_payload_lower_bound <= NOVA_RESOURCE_LIMITS_V2.pp_payload_bytes);
        assert!(plan.vk_payload_lower_bound <= NOVA_RESOURCE_LIMITS_V2.vk_payload_bytes);
        assert!(plan.bundle_lower_bound <= NOVA_RESOURCE_LIMITS_V2.bundle_bytes);
        assert!(plan.pedersen_rss_lower_bound <= NOVA_RESOURCE_LIMITS_V2.setup_and_proof_rss_bytes);
        eprintln!(
            "Nova preflight: C={}, V={}, NZ={}, N={}, G={}, pp_lb={}, vk_lb={}, bundle_lb={}, \
             pedersen_rss_lb={}, caps={}/{}/{}/{}",
            plan.shape.constraints,
            plan.shape.auxiliaries,
            plan.shape.nonzeros,
            plan.ck_floor,
            plan.generator_count,
            plan.pp_payload_lower_bound,
            plan.vk_payload_lower_bound,
            plan.bundle_lower_bound,
            plan.pedersen_rss_lower_bound,
            NOVA_RESOURCE_LIMITS_V2.pp_payload_bytes,
            NOVA_RESOURCE_LIMITS_V2.vk_payload_bytes,
            NOVA_RESOURCE_LIMITS_V2.bundle_bytes,
            NOVA_WORKER_SAFETY_CAP_V2,
        );
    }

    #[test]
    fn nova_resource_preflight_rejects_every_cap_plus_one_before_setup() {
        let metrics = NovaShapeMetricsV2 {
            constraints: 56_650,
            inputs: 1,
            auxiliaries: 51_785,
            nonzeros: 255_918,
        };
        let plan = nova_resource_preflight_from_shape(metrics, NOVA_RESOURCE_LIMITS_V2)
            .expect("baseline preflight");
        let limits = [
            NovaResourceLimitsV2 {
                pp_payload_bytes: plan.pp_payload_lower_bound - 1,
                ..NOVA_RESOURCE_LIMITS_V2
            },
            NovaResourceLimitsV2 {
                vk_payload_bytes: plan.vk_payload_lower_bound - 1,
                ..NOVA_RESOURCE_LIMITS_V2
            },
            NovaResourceLimitsV2 {
                bundle_bytes: plan.bundle_lower_bound - 1,
                ..NOVA_RESOURCE_LIMITS_V2
            },
            NovaResourceLimitsV2 {
                setup_and_proof_rss_bytes: plan.pedersen_rss_lower_bound - 1,
                ..NOVA_RESOURCE_LIMITS_V2
            },
        ];

        for limit in limits {
            let mut setup_called = false;
            let result = nova_resource_preflight_from_shape(metrics, limit).and_then(|_| {
                setup_called = true;
                Ok(())
            });
            assert!(matches!(result, Err(RecursiveV2Error::Resource)));
            assert!(!setup_called, "a rejected preflight must not start setup");
        }
    }

    #[test]
    fn nova_resource_preflight_checked_arithmetic_rejects_overflow() {
        let overflow = NovaShapeMetricsV2 {
            constraints: 1,
            inputs: 1,
            auxiliaries: 1,
            nonzeros: u64::MAX,
        };
        assert!(matches!(
            nova_resource_preflight_from_shape(overflow, NOVA_RESOURCE_LIMITS_V2),
            Err(RecursiveV2Error::Overflow)
        ));
    }

    #[test]
    fn finite_control_machine_matches_independent_frozen_edge_matrix() {
        let mut accepted_edges = 0_usize;

        for phase in ControlPhaseV2::ALL {
            for done in [false, true] {
                for opcode in opcode_list() {
                    let expected = expected_control_edge(phase, done, opcode);
                    let matching_rows = CONTROL_TRANSITION_TABLE_V2
                        .iter()
                        .filter(|edge| {
                            edge.input_phase == phase
                                && edge.input_done == done
                                && edge.opcode == opcode
                        })
                        .count();
                    let actual = control_transition(phase, done, opcode);

                    match expected {
                        ExpectedControlEdgeV2::Accept {
                            next_phase,
                            next_done,
                        } => {
                            assert_eq!(
                                matching_rows, 1,
                                "accepted tuple must have exactly one production edge: phase={phase:?}, done={done}, opcode={opcode:?}"
                            );
                            let edge = actual.expect("frozen accepted tuple must not reject");
                            assert_eq!(edge.next_phase, next_phase);
                            assert_eq!(edge.next_done, next_done);
                            assert_eq!(edge.input_phase, phase);
                            assert_eq!(edge.input_done, done);
                            assert_eq!(edge.opcode, opcode);
                            accepted_edges += 1;

                            if edge.next_phase == phase && !edge.next_done {
                                assert!(
                                    matches!(
                                        opcode,
                                        crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::ReplayInput
                                            | crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::ReplayOutput
                                            | crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::BeginHash
                                            | crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::ShaBlock
                                            | crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::EndHash
                                            | crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::TraceChunk
                                            | crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::JmtUpdate
                                    ),
                                    "only explicit action rows may preserve the control phase"
                                );
                            }
                            assert!(
                                !edge.next_done,
                                "done is selected only by the schema-bound TraceClosure END_HASH"
                            );
                        }
                        ExpectedControlEdgeV2::Reject(rejection) => {
                            assert_eq!(
                                matching_rows, 0,
                                "rejected tuple must not have a production successor: phase={phase:?}, done={done}, opcode={opcode:?}"
                            );
                            assert_eq!(actual, Err(rejection));
                        }
                    }

                    if done {
                        assert_eq!(
                            actual,
                            Err(ControlTransitionRejectionV2::Finalized),
                            "a finalized state must reject every post-final event"
                        );
                    }
                    if opcode
                        == crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::FinalizeBlock
                        && (phase != ControlPhaseV2::Commit || done)
                    {
                        assert!(
                            actual.is_err(),
                            "FINALIZE_BLOCK must reject before Commit and after finalization"
                        );
                    }
                }
            }
        }

        assert_eq!(accepted_edges, CONTROL_TRANSITION_TABLE_V2.len());
    }

    #[test]
    fn non_boolean_done_cell_is_unsatisfied() {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let event = source_event(
            ControlPhaseV2::Idle,
            crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::BeginBlock,
            0,
            b"non-boolean-done-cell",
        );
        let mut state =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
                .with_source_event(&event);
        state.cells[super::DONE_CELL] = 2;
        let circuit = source_circuit(anchors, event);
        let mut cs = TestConstraintSystem::<Scalar>::new();
        let input = state
            .scalars()
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                AllocatedNum::alloc(cs.namespace(|| format!("input_{index}")), || Ok(value))
                    .expect("input alloc")
            })
            .collect::<Vec<_>>();

        circuit
            .synthesize(&mut cs, &input)
            .expect("the malformed assignment still synthesizes");
        assert!(
            !cs.is_satisfied(),
            "the R1CS must bind the done bit to its state cell"
        );
    }

    #[test]
    fn real_nova_verifier_bundle_loads_and_verifies_compressed_proof() {
        let _guard = real_nova_proof_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(report) = run_bounded_nova_worker() {
            eprintln!(
                "worker-confirmed real Nova bundle: peak_rss_bytes={}, elapsed_ms={}",
                report.peak_rss_bytes,
                report.elapsed.as_millis()
            );
            return;
        }
        let profile = RecursiveCircuitProfileV2::repository_fixture();
        let store =
            crate::settlement::SettlementStore::try_new().expect("fixture settlement store");
        let authority = RecursiveAuthoritySnapshotV2::resolve_repository_local_fixture(&store)
            .expect("fixture authority snapshot");
        let spec = RecursiveCircuitSpecV2::new(authority.authority().layout(), &profile)
            .expect("fixture circuit specification");

        let (anchors, events, states, expected_final) = canonical_hash_control_fixture();
        let state = states[0].clone();
        let circuit = source_circuit(anchors.clone(), events[0].clone());

        // Keep the fixed R1CS shape evidence on the same concrete circuit
        // before deriving Nova parameters for the bundle.
        let shape = shape_of(&circuit, &state);
        assert!(
            shape.0 > 0 && shape.1 > 0 && shape.2 > 0,
            "nonempty fixed shape"
        );

        let base_shape = nova_resource_preflight(&circuit, &state)
            .expect("base-shape lower bound before bounded setup")
            .shape;
        let base_constraints =
            usize::try_from(base_shape.constraints).expect("base step constraint count fits usize");
        let base_auxiliaries =
            usize::try_from(base_shape.auxiliaries).expect("base step auxiliary count fits usize");
        let pp = setup_public_parameters_after_preflight(&circuit, &state)
            .expect("resource preflight and public parameters");
        let augmented_constraints = pp.num_constraints();
        let augmented_variables = pp.num_variables();
        eprintln!(
            "real Nova augmented shape: primary_constraints={}, secondary_constraints={}, primary_variables={}, secondary_variables={}",
            augmented_constraints.0,
            augmented_constraints.1,
            augmented_variables.0,
            augmented_variables.1,
        );
        assert!(
            augmented_constraints.0 > base_constraints && augmented_variables.0 > base_auxiliaries,
            "Nova augmented shape must be measured separately from the base step"
        );
        assert!(
            augmented_constraints.1 > 0 && augmented_variables.1 > 0,
            "Nova secondary augmented shape must be nonempty"
        );
        let binding = VerifierBundleBindingV2::from_authority(&authority, &profile, &spec, &pp)
            .expect("authority-pinned PP digest");
        let mut recursive =
            RecursiveSNARK::new(&pp, &circuit, &state.scalars()).expect("initial recursive proof");
        recursive.prove_step(&pp, &circuit).expect("real Nova step");
        for event in events.iter().skip(1) {
            let control_circuit = source_circuit(anchors.clone(), event.clone());
            recursive
                .prove_step(&pp, &control_circuit)
                .expect("canonical derived hash-control Nova step");
        }
        let (pk, vk) = Proof::setup(&pp).expect("compressed proof keys");
        let prover = NovaProverMaterialV2::new(pp, pk);
        let proof =
            Proof::prove(&prover.pp, &prover.pk, &recursive).expect("real compressed proof");

        // Measure exact encoded payloads before applying any existing cap. A
        // cap breach is evidence, not a reason to widen a private wire bound.
        let vk_payload = measured_bincode(&vk);
        let proof_payload = measured_bincode(&proof);
        eprintln!(
            "real Nova bundle payload bytes: vk={}, proof={}, header={}",
            vk_payload.len(),
            proof_payload.len(),
            VERIFIER_BUNDLE_HEADER_BYTES_V2,
        );
        assert!(
            vk_payload.len() <= NOVA_VK_SAFETY_CAP_V2,
            "VK payload safety-cap breach: {} > {}",
            vk_payload.len(),
            NOVA_VK_SAFETY_CAP_V2
        );
        assert!(
            proof_payload.len() <= MAX_NOVA_COMPRESSED_PROOF_BYTES_V2,
            "compressed-proof cap breach: {} > {}",
            proof_payload.len(),
            MAX_NOVA_COMPRESSED_PROOF_BYTES_V2
        );

        let mut wrong_pp_binding = binding;
        wrong_pp_binding.pp_digest[0] ^= 1;
        assert!(
            prover.verifier_bundle(&vk, wrong_pp_binding).is_err(),
            "bundle generation must reject a PP digest that is not the prover PP"
        );

        let bundle = prover
            .verifier_bundle(&vk, binding)
            .expect("bundle under existing caps");
        eprintln!(
            "real Nova verifier bundle bytes: {} (diagnostic safety cap {})",
            bundle.len(),
            NOVA_BUNDLE_SAFETY_CAP_V2,
        );
        assert_eq!(
            bundle.len(),
            VERIFIER_BUNDLE_HEADER_BYTES_V2 + vk_payload.len(),
            "verifier bundle must carry only its header and canonical VK payload"
        );
        assert!(bundle.len() <= NOVA_BUNDLE_SAFETY_CAP_V2);

        let loaded =
            NovaVerifierBundleV2::load(&bundle, binding).expect("strict canonical bundle load");
        assert!(
            loaded
                .decode_compressed_proof(&vec![0_u8; MAX_NOVA_COMPRESSED_PROOF_BYTES_V2 + 1])
                .is_err(),
            "the 128 KiB proof cap must reject before compressed-proof decoding"
        );
        let loaded_proof = loaded
            .decode_compressed_proof(&proof_payload)
            .expect("decode real compressed proof after successful bundle load");
        let output = loaded
            .verify(
                &loaded_proof,
                usize::try_from(events.len()).expect("test step count fits usize"),
                &state.scalars(),
            )
            .expect("loaded verifier accepts real compressed proof");
        assert_eq!(output, expected_final.scalars());

        let envelope = NovaProofEnvelopeV2::new(
            &loaded,
            7,
            7,
            events.len(),
            &state.scalars(),
            &expected_final.scalars(),
            proof_payload.clone(),
        )
        .expect("one portable proof envelope names the loaded bundle by digest");
        let encoded_envelope = envelope
            .encode()
            .expect("canonical proof envelope encoding");
        let loaded_envelope = NovaProofEnvelopeV2::load(&encoded_envelope, &loaded)
            .expect("bundle-bound proof envelope loads before verification");
        assert_eq!(
            loaded_envelope
                .verify(&loaded)
                .expect("bundle-bound envelope verifies the exact endpoints"),
            expected_final.scalars(),
        );
        assert_eq!(
            encoded_envelope.len(),
            NOVA_PROOF_ENVELOPE_HEADER_BYTES_V2
                + (2 * super::NOVA_PUBLIC_STATE_BYTES_V2)
                + proof_payload.len(),
            "the portable envelope carries only fixed public endpoints and one proof body",
        );
        let mut wrong_final_state = expected_final.scalars();
        wrong_final_state[0] += Scalar::from(1_u64);
        let wrong_endpoint_envelope = NovaProofEnvelopeV2::new(
            &loaded,
            7,
            7,
            events.len(),
            &state.scalars(),
            &wrong_final_state,
            proof_payload.clone(),
        )
        .expect("a syntactically framed envelope may carry an unverified endpoint claim");
        let loaded_wrong_endpoint = NovaProofEnvelopeV2::load(
            &wrong_endpoint_envelope
                .encode()
                .expect("wrong endpoint envelope framing"),
            &loaded,
        )
        .expect("correctly framed wrong endpoint reaches real verification");
        assert!(
            loaded_wrong_endpoint.verify(&loaded).is_err(),
            "a digest-consistent final-endpoint substitution must fail real proof verification"
        );
        let mutated_envelope = mutate_proof_envelope_header(&encoded_envelope, |header| {
            header.bundle_digest[0] ^= 1;
        });
        assert!(
            NovaProofEnvelopeV2::load(&mutated_envelope, &loaded).is_err(),
            "a proof envelope cannot substitute its verifier bundle"
        );
        let mut mutated_body = encoded_envelope.clone();
        let last = mutated_body
            .last_mut()
            .expect("canonical envelope has a nonempty compressed proof body");
        *last ^= 1;
        assert!(
            NovaProofEnvelopeV2::load(&mutated_body, &loaded).is_err(),
            "a proof-body mutation must reject before verification"
        );

        for mutate in [
            |header: &mut VerifierBundleHeaderV2| header.pp_digest[0] ^= 1,
            |header: &mut VerifierBundleHeaderV2| header.authority_digest[0] ^= 1,
            |header: &mut VerifierBundleHeaderV2| header.profile_digest[0] ^= 1,
            |header: &mut VerifierBundleHeaderV2| header.spec_digest[0] ^= 1,
            |header: &mut VerifierBundleHeaderV2| header.source_digest[0] ^= 1,
            |header: &mut VerifierBundleHeaderV2| header.lockfile_digest[0] ^= 1,
        ] {
            let mutated = mutate_bundle_header(&bundle, mutate);
            assert_bundle_rejected_before_proof_decode(&mutated, binding);
        }
        let mut vk_payload_mutation = bundle.clone();
        vk_payload_mutation[VERIFIER_BUNDLE_HEADER_BYTES_V2] ^= 1;
        assert_bundle_rejected_before_proof_decode(&vk_payload_mutation, binding);
    }

    #[test]
    fn every_opcode_uses_one_fixed_shape() {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let mut expected = None;
        for opcode in opcode_list() {
            let (phase, ordinal) = match opcode {
                crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::BeginBlock => {
                    (ControlPhaseV2::Idle, 0)
                }
                crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::ReplayInput
                | crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::ReplayOutput => {
                    (ControlPhaseV2::Replay, 1)
                }
                crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::UniquenessPrecommit => {
                    (ControlPhaseV2::Replay, 1)
                }
                crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::UniquenessChallenge => {
                    (ControlPhaseV2::Precommit, 2)
                }
                crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::NetMerge => {
                    (ControlPhaseV2::Challenge, 3)
                }
                crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::JmtUpdate => {
                    (ControlPhaseV2::Net, 4)
                }
                crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::PromoteChildRoot => {
                    (ControlPhaseV2::Jmt, 5)
                }
                crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::CommitTypedEvent => {
                    (ControlPhaseV2::Promote, 6)
                }
                crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::FinalizeBlock => {
                    (ControlPhaseV2::Commit, 7)
                }
                crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::BeginHash
                | crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::ShaBlock
                | crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::EndHash => {
                    (ControlPhaseV2::Replay, 1)
                }
                crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::TraceChunk => {
                    (ControlPhaseV2::Replay, 1)
                }
            };
            let state = CheckpointRunningStateV2::with_control(&anchors, phase, ordinal, false);
            let current = shape_of(&circuit(anchors.clone(), phase, opcode, ordinal), &state);
            match expected {
                Some(shape) => assert_eq!(current, shape, "opcode must not change shape"),
                None => expected = Some(current),
            }
        }
    }

    #[test]
    fn real_nova_proof_binds_one_source_event_after_trace_begin() {
        let _guard = real_nova_proof_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(report) = run_bounded_nova_worker() {
            eprintln!(
                "worker-confirmed real Nova source binding: peak_rss_bytes={}, elapsed_ms={}",
                report.peak_rss_bytes,
                report.elapsed.as_millis()
            );
            return;
        }
        let (anchors, events, states, _) = canonical_hash_control_fixture();
        let trace_begin = &events[0];
        let first_source = events
            .iter()
            .position(|event| {
                event.opcode
                    == crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::BeginBlock
            })
            .expect("canonical schedule has its first source record");
        assert_eq!(first_source, 1, "TRACE BEGIN precedes the first source");
        let first_source_event = &events[first_source];
        let state = states[0].clone();
        let trace_circuit = source_circuit(anchors.clone(), trace_begin.clone());
        let source_step_circuit = source_circuit(anchors.clone(), first_source_event.clone());
        let (first_cs, _) = synthesize_test(&trace_circuit, &state);
        assert!(
            first_cs.is_satisfied(),
            "TRACE BEGIN violates {}",
            first_cs
                .which_is_unsatisfied()
                .unwrap_or("an unknown constraint")
        );
        let (source_cs, _) = synthesize_test(&source_step_circuit, &states[first_source]);
        assert!(
            source_cs.is_satisfied(),
            "source record after TRACE BEGIN violates {}",
            source_cs
                .which_is_unsatisfied()
                .unwrap_or("an unknown constraint")
        );
        let base_shape = nova_resource_preflight(&trace_circuit, &state)
            .expect("base-shape lower bound before bounded setup")
            .shape;
        let base_constraints =
            usize::try_from(base_shape.constraints).expect("base step constraint count fits usize");
        let base_auxiliaries =
            usize::try_from(base_shape.auxiliaries).expect("base step auxiliary count fits usize");
        let pp = setup_public_parameters_after_preflight(&trace_circuit, &state)
            .expect("resource preflight and public parameters");
        let augmented_constraints = pp.num_constraints();
        let augmented_variables = pp.num_variables();
        eprintln!(
            "real Nova augmented shape: primary_constraints={}, secondary_constraints={}, primary_variables={}, secondary_variables={}",
            augmented_constraints.0,
            augmented_constraints.1,
            augmented_variables.0,
            augmented_variables.1,
        );
        assert!(
            augmented_constraints.0 > base_constraints && augmented_variables.0 > base_auxiliaries,
            "Nova augmented shape must be measured separately from the base step"
        );
        assert!(
            augmented_constraints.1 > 0 && augmented_variables.1 > 0,
            "Nova secondary augmented shape must be nonempty"
        );
        let mut recursive =
            RecursiveSNARK::new(&pp, &trace_circuit, &state.scalars()).expect("TRACE BEGIN");
        recursive
            .prove_step(&pp, &trace_circuit)
            .expect("TRACE BEGIN Nova step");
        recursive
            .prove_step(&pp, &source_step_circuit)
            .expect("canonical source Nova step");
        assert_eq!(recursive.num_steps(), 2);
        let uncompressed_output = recursive
            .verify(&pp, 2, &state.scalars())
            .expect("uncompressed trace-plus-source verification");
        let (pk, vk) = Proof::setup(&pp).expect("compressed keys");
        let proof = Proof::prove(&pp, &pk, &recursive).expect("compressed proof");
        let output = proof
            .verify(&vk, 2, &state.scalars())
            .expect("compressed verification");
        assert_eq!(output, uncompressed_output);
        assert_eq!(output, states[first_source + 1].scalars());

        let tampered_circuit = source_circuit(
            anchors.clone(),
            first_source_event.clone().tampered_payload(),
        );
        let mut tampered = RecursiveSNARK::new(&pp, &trace_circuit, &state.scalars())
            .expect("tampered TRACE BEGIN");
        tampered
            .prove_step(&pp, &trace_circuit)
            .expect("tampered TRACE BEGIN step");
        tampered
            .prove_step(&pp, &tampered_circuit)
            .expect("tampered source event records its invalid witness");
        assert!(
            tampered.verify(&pp, 2, &state.scalars()).is_err(),
            "the actual R1CS must reject a source-event payload digest mismatch"
        );

        let second_source = events
            .iter()
            .position(|event| {
                event.opcode
                    == crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::ReplayInput
            })
            .expect("canonical schedule has a second source record");
        let skipped_circuit = source_circuit(anchors.clone(), events[second_source].clone());
        let mut skipped = RecursiveSNARK::new(&pp, &trace_circuit, &state.scalars())
            .expect("skipped TRACE BEGIN");
        skipped
            .prove_step(&pp, &trace_circuit)
            .expect("skipped TRACE BEGIN step");
        skipped
            .prove_step(&pp, &skipped_circuit)
            .expect("skipped source event records its invalid witness");
        assert!(
            skipped.verify(&pp, 2, &state.scalars()).is_err(),
            "the actual R1CS must reject a skipped source ordinal"
        );

        let reordered_event = source_event(
            ControlPhaseV2::Idle,
            crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::BeginBlock,
            0,
            b"typed-source-event-reordered",
        );
        let reordered_circuit = source_circuit(anchors.clone(), reordered_event);
        let mut reordered = RecursiveSNARK::new(&pp, &trace_circuit, &state.scalars())
            .expect("reordered TRACE BEGIN");
        reordered
            .prove_step(&pp, &trace_circuit)
            .expect("reordered TRACE BEGIN step");
        reordered
            .prove_step(&pp, &reordered_circuit)
            .expect("reordered source event records its invalid witness");
        assert!(
            reordered.verify(&pp, 2, &state.scalars()).is_err(),
            "the actual R1CS must reject a reordered source event"
        );

        let accumulator_state = state.clone().with_trace_root_limb(0, 1);
        let mut accumulator_tampered =
            RecursiveSNARK::new(&pp, &trace_circuit, &accumulator_state.scalars())
                .expect("accumulator-tampered TRACE BEGIN");
        accumulator_tampered
            .prove_step(&pp, &trace_circuit)
            .expect("accumulator-tampered TRACE BEGIN step");
        accumulator_tampered
            .prove_step(&pp, &source_step_circuit)
            .expect("accumulator-tampered source step");
        assert!(
            accumulator_tampered
                .verify(&pp, 2, &state.scalars())
                .is_err(),
            "the actual verifier must bind the source trace accumulator state"
        );
    }

    #[test]
    fn source_record_rejects_a_second_record_before_hash_completion() {
        use crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2;

        let (anchors, events, states, _) = canonical_hash_control_fixture();
        let first_source = events
            .iter()
            .position(|event| event.opcode == RecursiveTraceOpcodeV2::BeginBlock)
            .expect("canonical replay starts with BeginBlock after TRACE BEGIN");
        let second_source = events
            .iter()
            .position(|event| event.opcode == RecursiveTraceOpcodeV2::ReplayInput)
            .expect("canonical replay carries a second source record");
        // Preserve the prior row's committed next-event digest so this
        // mutation reaches the source-context exclusion gate rather than the
        // generic digest-chain guard first.
        let mut premature = events[second_source].clone();
        premature.payload_digest_limbs = events[first_source + 1].payload_digest_limbs;
        premature.hash_control.source_hash = std::array::from_fn(|index| {
            events[first_source + 1].payload_digest_limbs[index / 2].to_le_bytes()[index % 2]
        });
        let second_circuit = source_circuit(anchors, premature);
        let (second_cs, _) = synthesize_test(&second_circuit, &states[first_source + 1]);
        assert!(
            !second_cs.is_satisfied(),
            "a second source record must not bypass BEGIN_HASH/SHA_BLOCK*/END_HASH"
        );
        assert!(
            second_cs
                .which_is_unsatisfied()
                .is_some_and(|constraint| constraint.starts_with("hash_control_schedule/")),
            "the rejection must come from the canonical hash schedule"
        );
    }

    #[test]
    fn source_stage_cannot_masquerade_as_a_hash_control() {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let mut forged = source_event(
            ControlPhaseV2::Idle,
            crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::BeginBlock,
            0,
            b"source-stage-opcode-confusion",
        );
        // `BEGIN_HASH` is a legal idle self-loop, so rejecting this witness
        // must come from the R1CS source-stage/opcode relation, not host
        // transition-table dispatch.
        forged.opcode = crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::BeginHash;
        let initial =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
                .with_source_event(&forged);
        let circuit = source_circuit(anchors, forged);
        let (cs, _) = synthesize_test(&circuit, &initial);
        assert!(
            !cs.is_satisfied(),
            "source-stage witness accepted under a hash-control opcode"
        );
        assert_eq!(
            cs.which_is_unsatisfied(),
            Some("source_stage_matches_source_opcode"),
            "the forged witness must reach the source-stage/opcode R1CS gate"
        );
    }

    #[test]
    fn source_record_requires_a_live_global_trace_context() {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let event = source_event(
            ControlPhaseV2::Idle,
            crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::BeginBlock,
            0,
            b"source-before-global-trace",
        );
        let initial =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
                .with_source_event(&event);

        let circuit = source_circuit(anchors.clone(), event.clone());
        let (cs, _) = synthesize_test(&circuit, &initial);
        assert!(
            !cs.is_satisfied(),
            "source cannot precede global TRACE BEGIN"
        );
        assert!(
            cs.which_is_unsatisfied()
                .is_some_and(|constraint| constraint
                    .starts_with("hash_control_schedule/source_requires_global_context_active/")),
            "the missing global TRACE BEGIN must reach its active-context gate, got {:?}",
            cs.which_is_unsatisfied()
        );

        let mut partial_global = initial;
        partial_global.cells
            [super::GLOBAL_BYTE_CONTEXT_START + super::BYTE_CONTEXT_ACTIVE_OFFSET] = 1;
        let circuit = source_circuit(anchors, event);
        let (cs, _) = synthesize_test(&circuit, &partial_global);
        assert!(
            !cs.is_satisfied(),
            "a globally active but unstarted context cannot accept a source"
        );
        assert!(
            cs.which_is_unsatisfied()
                .is_some_and(|constraint| constraint
                    .starts_with("hash_control_schedule/source_requires_global_context_started/")),
            "a partial global context must reach its started-context gate, got {:?}",
            cs.which_is_unsatisfied()
        );
    }

    #[test]
    fn final_source_record_requires_global_hash_closure() {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let final_event = source_event(
            ControlPhaseV2::Commit,
            crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::FinalizeBlock,
            0,
            b"typed-source-final",
        );
        let prior_root = [7_u64; super::DIGEST_LIMBS];
        let expected_root = trace_root_after(prior_root, &final_event);
        let authority = NovaTraceRootAuthorityV2::new(expected_root);
        let final_state =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Commit, 0, false)
                .with_source_event(&final_event)
                .with_trace_root(prior_root)
                .with_trace_authority(&authority);
        let final_circuit = source_circuit(anchors.clone(), final_event.clone());
        let (cs, _) = synthesize_test(&final_circuit, &final_state);
        assert!(
            !cs.is_satisfied(),
            "a FINALIZE_BLOCK source record cannot bypass its global TRACE BEGIN"
        );
        assert!(
            cs.which_is_unsatisfied()
                .is_some_and(|constraint| constraint
                    .starts_with("hash_control_schedule/source_requires_global_context_active/")),
            "the final source record must reach the global-context gate, got {:?}",
            cs.which_is_unsatisfied()
        );
    }

    #[test]
    fn schema_bound_trace_end_is_the_only_trace_closure_terminal_edge() {
        let (anchors, events, states, _) = canonical_hash_control_fixture();
        let global_end = events
            .iter()
            .enumerate()
            .rev()
            .find(|(_, event)| {
                event.opcode == RecursiveTraceOpcodeV2::EndHash
                    && event.hash_control.schema == HashControlSchemaV2::TracePrecommit as u8
            })
            .expect("canonical schedule must end its global trace hash");
        let local_end = events
            .iter()
            .enumerate()
            .take(global_end.0)
            .rev()
            .find(|(_, event)| {
                event.opcode == RecursiveTraceOpcodeV2::EndHash
                    && event.hash_control.schema == HashControlSchemaV2::SourceRecord as u8
            })
            .expect("final source record must close its local hash first");

        let mut local_event = local_end.1.clone();
        local_event.phase = ControlPhaseV2::TraceClosure;
        let mut local_state = states[local_end.0].clone();
        local_state.cells[super::PHASE_CELL] = ControlPhaseV2::TraceClosure as u64;
        let local_circuit = source_circuit(anchors.clone(), local_event);
        let (local_cs, local_output) = synthesize_test(&local_circuit, &local_state);
        assert!(
            local_cs.is_satisfied(),
            "the final source-local END_HASH remains a live TraceClosure step: {}",
            local_cs
                .which_is_unsatisfied()
                .unwrap_or("unknown constraint")
        );
        let local_output = state_from_output(&local_output);
        assert_eq!(
            local_output.cells[super::PHASE_CELL],
            ControlPhaseV2::TraceClosure as u64
        );
        assert_eq!(local_output.cells[super::DONE_CELL], 0);

        let mut global_event = global_end.1.clone();
        global_event.phase = ControlPhaseV2::TraceClosure;
        let mut global_state = states[global_end.0].clone();
        global_state.cells[super::PHASE_CELL] = ControlPhaseV2::TraceClosure as u64;
        let prior_root =
            std::array::from_fn(|index| global_state.cells[super::SOURCE_TRACE_ROOT_START + index]);
        let expected_root = trace_root_after(prior_root, &global_event);
        for (index, limb) in expected_root.into_iter().enumerate() {
            global_state.cells[super::EXPECTED_TRACE_ROOT_START + index] = limb;
        }
        let global_circuit = source_circuit(anchors, global_event);
        let (global_cs, global_output) = synthesize_test(&global_circuit, &global_state);
        assert!(
            global_cs.is_satisfied(),
            "the schema-bound global END_HASH must close TraceClosure: {}",
            global_cs
                .which_is_unsatisfied()
                .unwrap_or("unknown constraint")
        );
        let global_output = state_from_output(&global_output);
        assert_eq!(
            global_output.cells[super::PHASE_CELL],
            ControlPhaseV2::Idle as u64
        );
        assert_eq!(global_output.cells[super::DONE_CELL], 1);
        for index in super::transient_cells() {
            assert_eq!(
                global_output.cells[index], 0,
                "global END_HASH must leave no transient cell at {index}"
            );
        }
    }
}
