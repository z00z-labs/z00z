//! Private Nova foundation for the sole recursive-V2 storage owner.
//!
//! This module intentionally has no public re-export.  It fixes the running
//! state layout and constrains the common control skeleton before the full
//! replay, SHA, uniqueness, and JMT relations are connected to it.

use ff::PrimeField;
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
    nova::{CompressedSNARK, PublicParams, VerifierKey},
    provider::{ipa_pc::EvaluationEngine, pasta::pallas, PallasEngine, VestaEngine},
    spartan::ppsnark::RelaxedR1CSSNARK,
    traits::{circuit::StepCircuit, snark::RelaxedR1CSSNARKTrait},
};
use serde::{de::DeserializeOwned, Serialize};

use crate::checkpoint::{
    recursive_circuit::{
        RecursiveCircuitProfileV2, RecursiveCircuitSpecV2, RECURSIVE_CIRCUIT_PROFILE_VERSION_V2,
        RECURSIVE_CIRCUIT_SPEC_VERSION_V2,
    },
    recursive_context::RecursiveAuthoritySnapshotV2,
    recursive_reject::RecursiveV2Error,
};

#[cfg(test)]
use crate::checkpoint::recursive_trace::RecursiveTracePrecommitV2;
use crate::checkpoint::recursive_trace::{
    decode_hash_control, HashControlBindingV2, HashControlSchemaV2, HashControlStageV2,
    RecursiveTraceEventV2, RecursiveTraceOpcodeV2,
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
const UNIQUENESS_END: usize = 360;
const NET_START: usize = UNIQUENESS_END;
const NET_END: usize = 376;
const JMT_START: usize = NET_END;
const JMT_END: usize = 536;
const HIERARCHY_START: usize = JMT_END;
const HIERARCHY_END: usize = 604;
const COMMITMENTS_START: usize = HIERARCHY_END;
const COMMITMENTS_END: usize = 700;
const EXPECTED_FINALS_START: usize = COMMITMENTS_END;
const EXPECTED_TRACE_ROOT_START: usize = EXPECTED_FINALS_START;
const EXPECTED_TRACE_ROOT_END: usize = EXPECTED_TRACE_ROOT_START + DIGEST_LIMBS;
const EXPECTED_TRACE_DIGEST_START: usize = EXPECTED_TRACE_ROOT_END;
const EXPECTED_TRACE_DIGEST_END: usize = EXPECTED_TRACE_DIGEST_START + DIGEST_LIMBS;
const RUNNING_STATE_ARITY_V2: usize = EXPECTED_FINALS_START + 80;

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
}

impl ControlPhaseV2 {
    const ALL: [Self; 8] = [
        Self::Idle,
        Self::Replay,
        Self::Precommit,
        Self::Challenge,
        Self::Net,
        Self::Jmt,
        Self::Promote,
        Self::Commit,
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
const CONTROL_TRANSITION_TABLE_V2: [ControlTransitionV2; 35] = [
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
        ControlPhaseV2::Idle,
        true,
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
/// families even though later T2 work will connect their semantic constraints.
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

    #[cfg(test)]
    fn with_trace_precommit(mut self, precommit: RecursiveTracePrecommitV2) -> Self {
        self.expected_trace_digest = digest_limbs(precommit.trace_digest());
        self
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
}

impl NovaTypedSourceEventV2 {
    fn from_source(
        phase: ControlPhaseV2,
        event: &RecursiveTraceEventV2,
    ) -> Result<Self, RecursiveV2Error> {
        let (hash_control, sha_compression) = match event.opcode() {
            opcode if opcode.is_source_record() => (
                HashControlWitnessV2::from_source_record(event)?,
                ShaCompressionLaneWitnessV2::inactive(),
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
                )
            }
            _ => return Err(RecursiveV2Error::Canonical),
        };
        Ok(Self {
            phase,
            opcode: event.opcode(),
            ordinal: event.ordinal(),
            payload_digest_limbs: digest_limbs(event.hash_binding()?),
            next_payload_digest_limbs: [0_u16; DIGEST_LIMBS],
            hash_control,
            sha_compression,
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
            role: TRACE_HASH_ROLE_TAG_V2 as u8,
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
            stage: control.stage as u8,
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
    fn source_schema_selector(&self) -> &AllocatedBit {
        &self.schema_selectors[0]
    }

    fn trace_schema_selector(&self) -> &AllocatedBit {
        &self.schema_selectors[1]
    }

    fn source_selector(&self) -> &AllocatedBit {
        &self.stage_selectors[0]
    }

    fn begin_selector(&self) -> &AllocatedBit {
        &self.stage_selectors[HashControlStageV2::Begin as usize]
    }

    fn block_selector(&self) -> &AllocatedBit {
        &self.stage_selectors[HashControlStageV2::Block as usize]
    }

    fn end_selector(&self) -> &AllocatedBit {
        &self.stage_selectors[HashControlStageV2::End as usize]
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
    let mut stage_selectors = Vec::with_capacity(4);
    for stage_value_candidate in 0_u64..=3 {
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
    range_bits(cs.namespace(|| "stage_range"), &stage, 2)?;

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
    enforce_gated_constant(
        cs.namespace(|| "trace_role"),
        &stage_selectors[0],
        &role,
        TRACE_HASH_ROLE_TAG_V2,
    );
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
        Ok(Self {
            event,
            next_phase: edge.next_phase,
            next_done: edge.next_done,
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
        let hash_control = allocate_hash_control(
            cs.namespace(|| "derived_hash_control"),
            &self.witness.event.hash_control,
        )?;
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
            enforce_gated_constant(
                cs.namespace(|| format!("hash_control_opcode_{index}")),
                selector,
                &event_opcode,
                opcode as u8 as u64,
            );
        }
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
        let mut next_done_lc = None;
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
                    add_weighted_bit(
                        &mut next_phase_lc,
                        &gate,
                        Scalar::from(edge.next_phase as u64),
                    );
                    if edge.next_done {
                        add_weighted_bit(&mut next_done_lc, &gate, Scalar::from(1_u64));
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
        let sha_outputs = synthesize_sha_compression_lane(
            cs.namespace(|| "sha_compression_lane"),
            sha_block_selector,
            z,
            &self.witness.event.sha_compression,
        )?;
        let schedule_outputs = synthesize_hash_control_schedule(
            cs.namespace(|| "hash_control_schedule"),
            z,
            &event_ordinal,
            &hash_control,
            &sha_outputs,
        )?;

        let final_gate = final_gate.ok_or_else(|| {
            SynthesisError::Unsatisfiable("finalize transition missing from table".to_owned())
        })?;
        for index in transient_cells() {
            cs.enforce(
                || format!("final_idle_zero_{index}"),
                |lc| lc + final_gate.get_variable(),
                |lc| lc + z[index].get_variable(),
                |lc| lc,
            );
        }

        let mut output = z.to_vec();
        output[PHASE_CELL] = out_phase;
        output[PRIOR_OPCODE_CELL] = out_opcode;
        output[ORDINAL_CELL] = out_ordinal;
        output[SOURCE_TRACE_ORDINAL_CELL] = schedule_outputs.source_trace_ordinal;
        output[SOURCE_TRACE_BYTE_COUNT_CELL] = schedule_outputs.source_trace_byte_count;
        output[DONE_CELL] = out_done;
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
        Ok(output)
    }
}

fn opcode_list() -> [RecursiveTraceOpcodeV2; 13] {
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
    ]
}

struct ShaCompressionOutputsV2 {
    block_next_ordinal: AllocatedNum<Scalar>,
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
    witness: &ShaCompressionLaneWitnessV2,
) -> Result<ShaCompressionOutputsV2, SynthesisError> {
    let lane_ordinal = allocate_constant(cs.namespace(|| "ordinal"), witness.ordinal)?;
    let ordinal_next = witness
        .ordinal
        .checked_add(1)
        .ok_or_else(|| SynthesisError::Unsatisfiable("SHA block ordinal overflow".to_owned()))?;
    let next_ordinal_value =
        allocate_constant(cs.namespace(|| "next_ordinal_value"), ordinal_next)?;
    enforce_inactive_zero(
        cs.namespace(|| "inactive_ordinal_zero"),
        sha_block_selector,
        &lane_ordinal,
    );
    enforce_gated_equal(
        cs.namespace(|| "active_ordinal_matches_state"),
        sha_block_selector,
        &z[SHA_BLOCK_ORDINAL_CELL],
        &lane_ordinal,
    );
    enforce_gated_constant(
        cs.namespace(|| "active_lane_flag"),
        sha_block_selector,
        &z[SHA_ACTIVE_CELL],
        1,
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
        enforce_gated_equal(
            cs.namespace(|| format!("active_chaining_before_matches_state_{index}")),
            sha_block_selector,
            &z[state_index],
            &before,
        );
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
        block_next_ordinal: next_ordinal_value,
        block_next_chaining: expected_after,
        block_bytes,
    })
}

struct HashScheduleOutputsV2 {
    source_trace_ordinal: AllocatedNum<Scalar>,
    source_trace_byte_count: AllocatedNum<Scalar>,
    cells: Vec<(usize, AllocatedNum<Scalar>)>,
}

/// Bind source records and their derived BEGIN_HASH / SHA_BLOCK* / END_HASH
/// controls as one fixed R1CS schedule.  The expander remains the sole owner
/// of the controls; this only consumes its decoded witness values.
fn synthesize_hash_control_schedule<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    z: &[AllocatedNum<Scalar>],
    event_ordinal: &AllocatedNum<Scalar>,
    control: &AllocatedHashControlV2,
    sha: &ShaCompressionOutputsV2,
) -> Result<HashScheduleOutputsV2, SynthesisError> {
    let selectors = &control.stage_selectors;
    let source = control.source_selector();
    let begin = control.begin_selector();
    let block = control.block_selector();
    let end = control.end_selector();
    let zero = allocate_constant(cs.namespace(|| "zero"), 0)?;
    let one = allocate_constant(cs.namespace(|| "one"), 1)?;
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
    // Per-source controls refer to the just-consumed source record: its
    // source ordinal is one behind the next source-record ordinal in state.
    for (selector_index, selector) in [source_begin, source_block, source_end]
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
    enforce_gated_equal(
        cs.namespace(|| "trace_begin_event_count_matches_source_count"),
        &trace_begin,
        &control.trace_event_count,
        &z[SOURCE_TRACE_ORDINAL_CELL],
    );
    enforce_gated_equal(
        cs.namespace(|| "trace_begin_byte_count_matches_source_bytes"),
        &trace_begin,
        &control.trace_byte_count,
        &z[SOURCE_TRACE_BYTE_COUNT_CELL],
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
    for (selector_index, selector) in [trace_block.clone(), trace_end.clone()]
        .into_iter()
        .enumerate()
    {
        enforce_gated_equal(
            cs.namespace(|| format!("trace_control_ordinal_matches_schedule_{selector_index}")),
            &selector,
            &control.source_ordinal,
            &z[HASH_SCHEDULE_SOURCE_ORDINAL_CELL],
        );
        enforce_gated_equal(
            cs.namespace(|| format!("trace_control_count_matches_schedule_{selector_index}")),
            &selector,
            &control.trace_event_count,
            &z[SOURCE_TRACE_ORDINAL_CELL],
        );
        enforce_gated_equal(
            cs.namespace(|| format!("trace_control_bytes_matches_schedule_{selector_index}")),
            &selector,
            &control.trace_byte_count,
            &z[SOURCE_TRACE_BYTE_COUNT_CELL],
        );
        enforce_gated_equal(
            cs.namespace(|| format!("trace_control_role_matches_schedule_{selector_index}")),
            &selector,
            &control.role,
            &z[HASH_SCHEDULE_ROLE_CELL],
        );
        for (index, hash_limb) in control.source_hash_limbs.iter().enumerate() {
            enforce_gated_equal(
                cs.namespace(|| {
                    format!("trace_control_hash_matches_schedule_{selector_index}_{index}")
                }),
                &selector,
                hash_limb,
                &z[HASH_SCHEDULE_SOURCE_HASH_START + index],
            );
        }
        enforce_gated_trace_geometry(
            cs.namespace(|| format!("trace_control_geometry_{selector_index}")),
            &selector,
            control,
        )?;
    }

    // BEGIN_HASH starts exactly at sequence zero and may only consume the
    // source binding that the preceding source-record step installed.
    enforce_gated_constant(
        cs.namespace(|| "begin_schedule_inactive"),
        begin,
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
            begin,
            &z[index],
            0,
        );
    }
    enforce_gated_hash_control_ordinal(
        cs.namespace(|| "begin_control_ordinal"),
        begin,
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
        |lc| lc + begin.get_variable(),
        |lc| lc + empty_block_count.get_variable(),
        |lc| lc,
    );
    for index in SHA_ACTIVE_CELL..SHA_END {
        enforce_gated_constant(
            cs.namespace(|| format!("begin_sha_input_zero_{index}")),
            begin,
            &z[index],
            0,
        );
    }

    // Every SHA_BLOCK advances the canonical encoded ordinal and exactly one
    // 64-byte block index.  The final marker is derived in-circuit, not
    // accepted from native transition verification.
    enforce_gated_constant(
        cs.namespace(|| "block_schedule_active"),
        block,
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
            block,
            &z[state_index],
            witness_value,
        );
    }
    enforce_gated_constant(
        cs.namespace(|| "block_prior_final_is_zero"),
        block,
        &z[HASH_SCHEDULE_FINAL_BLOCK_CELL],
        0,
    );
    enforce_gated_equal(
        cs.namespace(|| "block_schedule_ordinal_matches_event"),
        block,
        &z[HASH_SCHEDULE_ORDINAL_CELL],
        event_ordinal,
    );
    enforce_gated_hash_control_ordinal(
        cs.namespace(|| "block_control_ordinal"),
        block,
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
        end,
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
            end,
            &z[state_index],
            witness_value,
        );
    }
    enforce_gated_equal(
        cs.namespace(|| "end_schedule_ordinal_matches_event"),
        end,
        &z[HASH_SCHEDULE_ORDINAL_CELL],
        event_ordinal,
    );
    enforce_gated_equal(
        cs.namespace(|| "end_seen_all_blocks"),
        end,
        &z[HASH_SCHEDULE_NEXT_BLOCK_INDEX_CELL],
        &control.block_count,
    );
    enforce_gated_constant(
        cs.namespace(|| "end_requires_final_block"),
        end,
        &z[HASH_SCHEDULE_FINAL_BLOCK_CELL],
        1,
    );
    enforce_gated_hash_control_ordinal(
        cs.namespace(|| "end_control_ordinal"),
        end,
        event_ordinal,
        &control.source_ordinal,
        &control.block_count,
        true,
    );
    enforce_gated_constant(
        cs.namespace(|| "end_sha_active"),
        end,
        &z[SHA_ACTIVE_CELL],
        1,
    );
    enforce_gated_equal(
        cs.namespace(|| "end_sha_block_count"),
        end,
        &z[SHA_BLOCK_ORDINAL_CELL],
        &control.block_count,
    );
    for word_index in 0..8 {
        enforce_gated_lc_equal(
            cs.namespace(|| format!("end_digest_matches_chaining_{word_index}")),
            end,
            &source_hash_word_lc::<CS>(&control.source_hash_bytes, word_index),
            &z[SHA_CHAINING_START + word_index],
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

    let mut cells = Vec::with_capacity(COUNTERS_END - COUNTERS_START + SHA_END - SHA_ACTIVE_CELL);
    let stage_value = |name: &str,
                       source_value: &AllocatedNum<Scalar>,
                       begin_value: &AllocatedNum<Scalar>,
                       block_value: &AllocatedNum<Scalar>,
                       end_value: &AllocatedNum<Scalar>,
                       cs: &mut CS|
     -> Result<AllocatedNum<Scalar>, SynthesisError> {
        select_hash_stage_value(
            cs.namespace(|| name),
            selectors,
            [source_value, begin_value, block_value, end_value],
        )
    };

    cells.push((
        HASH_SCHEDULE_ACTIVE_CELL,
        stage_value("schedule_active_output", &zero, &one, &one, &zero, &mut cs)?,
    ));
    cells.push((
        HASH_SCHEDULE_ORDINAL_CELL,
        stage_value(
            "schedule_ordinal_output",
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
        stage_value(
            "schedule_source_ordinal_output",
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
            stage_value(
                &format!("schedule_scalar_output_{index}"),
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
            stage_value(
                &format!("schedule_source_hash_output_{index}"),
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
        stage_value(
            "schedule_role_output",
            &control.role,
            &begin_schedule_role,
            &z[HASH_SCHEDULE_ROLE_CELL],
            &zero,
            &mut cs,
        )?,
    ));

    let iv_words = SHA256_IV_V2.map(u64::from);
    let mut iv = Vec::with_capacity(8);
    for (index, word) in iv_words.into_iter().enumerate() {
        iv.push(allocate_constant(
            cs.namespace(|| format!("sha_iv_{index}")),
            word,
        )?);
    }
    cells.push((
        SHA_ACTIVE_CELL,
        stage_value(
            "sha_active_output",
            &z[SHA_ACTIVE_CELL],
            &one,
            &z[SHA_ACTIVE_CELL],
            &zero,
            &mut cs,
        )?,
    ));
    cells.push((
        SHA_BLOCK_ORDINAL_CELL,
        stage_value(
            "sha_ordinal_output",
            &z[SHA_BLOCK_ORDINAL_CELL],
            &zero,
            &sha.block_next_ordinal,
            &zero,
            &mut cs,
        )?,
    ));
    for (index, (iv_word, block_next)) in iv.iter().zip(&sha.block_next_chaining).enumerate() {
        let state_index = SHA_CHAINING_START + index;
        cells.push((
            state_index,
            stage_value(
                &format!("sha_chaining_output_{index}"),
                &z[state_index],
                iv_word,
                block_next,
                &zero,
                &mut cs,
            )?,
        ));
    }
    for (index, block_byte) in sha.block_bytes.iter().enumerate() {
        let state_index = SHA_BLOCK_START + index;
        cells.push((
            state_index,
            stage_value(
                &format!("sha_block_output_{index}"),
                &z[state_index],
                &zero,
                block_byte,
                &zero,
                &mut cs,
            )?,
        ));
    }

    Ok(HashScheduleOutputsV2 {
        source_trace_ordinal: select_hash_stage_value(
            cs.namespace(|| "source_trace_ordinal_output"),
            selectors,
            [
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

/// Select one of the four fixed hash-schedule successors.  The selector set
/// is one-hot, so these gated equalities define one output without any
/// witness-side validity branch.
fn select_hash_stage_value<CS: ConstraintSystem<Scalar>>(
    mut cs: CS,
    selectors: &[AllocatedBit],
    candidates: [&AllocatedNum<Scalar>; 4],
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
    range_bits(
        cs.namespace(|| "hash_schedule_role"),
        &z[HASH_SCHEDULE_ROLE_CELL],
        8,
    )?;
    for index in NET_START..NET_END {
        range_bits(cs.namespace(|| format!("net_cell_{index}")), &z[index], 64)?;
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

fn digest_cells() -> Vec<usize> {
    let mut cells = (0..ANCHOR_CELLS).collect::<Vec<_>>();
    for range in [
        SOURCE_EVENT_DIGEST_START..SOURCE_EVENT_DIGEST_END,
        SOURCE_TRACE_ROOT_END..CHAIN_END,
        UNIQUENESS_START..UNIQUENESS_END,
        JMT_START..JMT_END,
        HIERARCHY_START..HIERARCHY_END,
        COMMITMENTS_START..COMMITMENTS_END,
        EXPECTED_TRACE_DIGEST_END..RUNNING_STATE_ARITY_V2,
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
// make its wire contract immutable now: a later caller cannot choose a Nova
// suite, codec, feature set, or allocation bound at load time.
const VERIFIER_BUNDLE_MAGIC_V2: [u8; 8] = *b"Z00ZNBV2";
const VERIFIER_BUNDLE_FORMAT_V2: u8 = 1;
const NOVA_SUITE_ID_V2: &[u8] = b"nova-snark/0.73.0;pallas-vesta;ppsnark-ipa";
const NOVA_FEATURE_ID_V2: &[u8] = b"io";
const MAX_NOVA_PUBLIC_PARAMETERS_BYTES_V2: usize = 512 * 1024 * 1024;
const MAX_NOVA_VERIFIER_KEY_BYTES_V2: usize = 384 * 1024 * 1024;
const MAX_NOVA_COMPRESSED_PROOF_BYTES_V2: usize = 16 * 1024 * 1024;
const MAX_NOVA_VERIFIER_BUNDLE_BYTES_V2: usize =
    MAX_NOVA_PUBLIC_PARAMETERS_BYTES_V2 + MAX_NOVA_VERIFIER_KEY_BYTES_V2 + 1024;
const NOVA_PALLAS_AFFINE_WIRE_BYTES_V2: u64 = 32;
const NOVA_USIZE_WIRE_BYTES_V2: u64 = 8;
const NOVA_PEDERSEN_UNIFORM_BYTES_V2: u64 = 32;
const MAX_NOVA_SETUP_AND_PROOF_RSS_BYTES_V2: u64 = 24 * 1024 * 1024 * 1024;
const VERIFIER_BUNDLE_HEADER_BYTES_V2: usize = 8
    + 4
    + 1
    + NOVA_SUITE_ID_V2.len()
    + 1
    + NOVA_FEATURE_ID_V2.len()
    + 8
    + (4 * 8)
    + (12 * 32)
    + (2 * 4);

/// Resource caps applied before the private Nova setup path can allocate its
/// commitment key. The RSS ceiling reserves 13 GiB of the observed 37 GiB
/// free host memory for the process, allocator, and concurrent workspace work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NovaResourceLimitsV2 {
    pp_payload_bytes: u64,
    vk_payload_bytes: u64,
    bundle_bytes: u64,
    setup_and_proof_rss_bytes: u64,
}

const NOVA_RESOURCE_LIMITS_V2: NovaResourceLimitsV2 = NovaResourceLimitsV2 {
    pp_payload_bytes: MAX_NOVA_PUBLIC_PARAMETERS_BYTES_V2 as u64,
    vk_payload_bytes: MAX_NOVA_VERIFIER_KEY_BYTES_V2 as u64,
    bundle_bytes: MAX_NOVA_VERIFIER_BUNDLE_BYTES_V2 as u64,
    setup_and_proof_rss_bytes: MAX_NOVA_SETUP_AND_PROOF_RSS_BYTES_V2,
};

/// A lower bound, not a replacement for the one permitted real measurement.
/// Every value comes from the exact current `ShapeCS` sparse matrices and the
/// pinned Nova 0.73.0 setup/IPA layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NovaResourcePreflightV2 {
    shape: NovaShapeMetricsV2,
    ck_floor: u64,
    generator_count: u64,
    pp_payload_lower_bound: u64,
    vk_payload_lower_bound: u64,
    bundle_lower_bound: u64,
    pedersen_rss_lower_bound: u64,
}

fn checked_resource_add(left: u64, right: u64) -> Result<u64, RecursiveV2Error> {
    left.checked_add(right).ok_or(RecursiveV2Error::Overflow)
}

fn checked_resource_mul(left: u64, right: u64) -> Result<u64, RecursiveV2Error> {
    left.checked_mul(right).ok_or(RecursiveV2Error::Overflow)
}

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
    let bundle_lower_bound = checked_resource_add(
        checked_resource_add(
            u64::try_from(VERIFIER_BUNDLE_HEADER_BYTES_V2).map_err(|_| RecursiveV2Error::Limit)?,
            pp_payload_lower_bound,
        )?,
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

fn nova_resource_preflight(
    circuit: &CheckpointNovaCircuitV2,
    state: &CheckpointRunningStateV2,
) -> Result<NovaResourcePreflightV2, RecursiveV2Error> {
    nova_resource_preflight_from_shape(measure_shape(circuit, state)?, NOVA_RESOURCE_LIMITS_V2)
}

#[allow(dead_code)] // The future private runner must use this sole setup gate.
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
#[allow(dead_code)] // The future runner is the sole production bundle caller.
struct VerifierBundleBindingV2 {
    authority_digest: [u8; 32],
    profile_digest: [u8; 32],
    spec_digest: [u8; 32],
}

#[allow(dead_code)] // See `VerifierBundleBindingV2`.
impl VerifierBundleBindingV2 {
    fn from_authority(
        authority: &RecursiveAuthoritySnapshotV2,
        profile: &RecursiveCircuitProfileV2,
        spec: &RecursiveCircuitSpecV2,
    ) -> Self {
        Self {
            authority_digest: authority.authority().digest(),
            profile_digest: profile.digest(),
            spec_digest: spec.digest(),
        }
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
    pp_payload_digest: [u8; 32],
    vk_payload_digest: [u8; 32],
    project_digest: [u8; 32],
    primary_constraints: u64,
    secondary_constraints: u64,
    primary_variables: u64,
    secondary_variables: u64,
    pp_payload_bytes: u32,
    vk_payload_bytes: u32,
}

#[allow(dead_code)] // See `VerifierBundleHeaderV2`.
impl VerifierBundleHeaderV2 {
    fn new(
        pp: &NovaPublicParameters,
        binding: VerifierBundleBindingV2,
        pp_payload: &[u8],
        vk_payload: &[u8],
    ) -> Result<Self, RecursiveV2Error> {
        let pp_payload_bytes =
            u32::try_from(pp_payload.len()).map_err(|_| RecursiveV2Error::Limit)?;
        let vk_payload_bytes =
            u32::try_from(vk_payload.len()).map_err(|_| RecursiveV2Error::Limit)?;
        let (primary_constraints, secondary_constraints) = pp.num_constraints();
        let (primary_variables, secondary_variables) = pp.num_variables();
        let mut header = Self {
            pp_digest: scalar_digest(pp.digest())?,
            authority_digest: binding.authority_digest,
            profile_digest: binding.profile_digest,
            spec_digest: binding.spec_digest,
            grammar_digest: transition_table_digest(),
            shape_digest: circuit_shape_digest()?,
            source_digest: source_revision_digest(),
            lockfile_digest: lockfile_digest(),
            manifest_digest: manifest_digest(),
            pp_payload_digest: bundle_payload_digest(b"pp", pp_payload),
            vk_payload_digest: bundle_payload_digest(b"vk", vk_payload),
            project_digest: [0_u8; 32],
            primary_constraints: u64::try_from(primary_constraints)
                .map_err(|_| RecursiveV2Error::Limit)?,
            secondary_constraints: u64::try_from(secondary_constraints)
                .map_err(|_| RecursiveV2Error::Limit)?,
            primary_variables: u64::try_from(primary_variables)
                .map_err(|_| RecursiveV2Error::Limit)?,
            secondary_variables: u64::try_from(secondary_variables)
                .map_err(|_| RecursiveV2Error::Limit)?,
            pp_payload_bytes,
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
            self.pp_payload_digest,
            self.vk_payload_digest,
        ] {
            bytes.extend_from_slice(&digest);
        }
        bytes.extend_from_slice(&self.pp_payload_bytes.to_le_bytes());
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
        let pp_payload_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let vk_payload_digest = take_fixed::<32>(bytes, &mut cursor)?;
        let pp_payload_bytes = take_u32_bundle(bytes, &mut cursor)?;
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
            pp_payload_digest,
            vk_payload_digest,
            project_digest,
            primary_constraints,
            secondary_constraints,
            primary_variables,
            secondary_variables,
            pp_payload_bytes,
            vk_payload_bytes,
        })
    }

    fn validate_binding(&self, binding: VerifierBundleBindingV2) -> Result<(), RecursiveV2Error> {
        if self.authority_digest != binding.authority_digest
            || self.profile_digest != binding.profile_digest
            || self.spec_digest != binding.spec_digest
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

/// The only owner of decoded Nova verifier objects. It is intentionally
/// private, immutable, and validated before a compressed proof is decoded.
#[allow(dead_code)] // Wired to the runner with the remaining semantic relations.
struct CheckpointVerifierBundleV2 {
    header: VerifierBundleHeaderV2,
    pp: NovaPublicParameters,
    vk: NovaVerifierKey,
}

#[allow(dead_code)] // See `CheckpointVerifierBundleV2`.
impl CheckpointVerifierBundleV2 {
    fn encode(
        pp: &NovaPublicParameters,
        vk: &NovaVerifierKey,
        binding: VerifierBundleBindingV2,
    ) -> Result<Vec<u8>, RecursiveV2Error> {
        let pp_payload = encode_bincode(pp, MAX_NOVA_PUBLIC_PARAMETERS_BYTES_V2)?;
        let vk_payload = encode_bincode(vk, MAX_NOVA_VERIFIER_KEY_BYTES_V2)?;
        let header = VerifierBundleHeaderV2::new(pp, binding, &pp_payload, &vk_payload)?;
        let mut encoded = header.encode();
        encoded.extend_from_slice(&pp_payload);
        encoded.extend_from_slice(&vk_payload);
        if encoded.len() > MAX_NOVA_VERIFIER_BUNDLE_BYTES_V2 {
            return Err(RecursiveV2Error::Limit);
        }
        Ok(encoded)
    }

    fn load(bytes: &[u8], binding: VerifierBundleBindingV2) -> Result<Self, RecursiveV2Error> {
        if bytes.len() > MAX_NOVA_VERIFIER_BUNDLE_BYTES_V2 {
            return Err(RecursiveV2Error::Limit);
        }
        // This completed header validation deliberately precedes every
        // bincode allocation and every proof decode entry point.
        let header = VerifierBundleHeaderV2::decode(bytes)?;
        header.validate_binding(binding)?;
        let pp_len =
            usize::try_from(header.pp_payload_bytes).map_err(|_| RecursiveV2Error::Limit)?;
        let vk_len =
            usize::try_from(header.vk_payload_bytes).map_err(|_| RecursiveV2Error::Limit)?;
        if pp_len > MAX_NOVA_PUBLIC_PARAMETERS_BYTES_V2 || vk_len > MAX_NOVA_VERIFIER_KEY_BYTES_V2 {
            return Err(RecursiveV2Error::Limit);
        }
        let pp_end = VERIFIER_BUNDLE_HEADER_BYTES_V2
            .checked_add(pp_len)
            .ok_or(RecursiveV2Error::Overflow)?;
        let vk_end = pp_end
            .checked_add(vk_len)
            .ok_or(RecursiveV2Error::Overflow)?;
        if bytes.len() != vk_end {
            return Err(RecursiveV2Error::Canonical);
        }
        let pp_payload = bytes
            .get(VERIFIER_BUNDLE_HEADER_BYTES_V2..pp_end)
            .ok_or(RecursiveV2Error::Canonical)?;
        let vk_payload = bytes
            .get(pp_end..vk_end)
            .ok_or(RecursiveV2Error::Canonical)?;
        if bundle_payload_digest(b"pp", pp_payload) != header.pp_payload_digest
            || bundle_payload_digest(b"vk", vk_payload) != header.vk_payload_digest
        {
            return Err(RecursiveV2Error::Canonical);
        }
        let pp = decode_bincode::<NovaPublicParameters, MAX_NOVA_PUBLIC_PARAMETERS_BYTES_V2>(
            pp_payload,
        )?;
        let vk = decode_bincode::<NovaVerifierKey, MAX_NOVA_VERIFIER_KEY_BYTES_V2>(vk_payload)?;
        if scalar_digest(pp.digest())? != header.pp_digest {
            return Err(RecursiveV2Error::Invariant);
        }
        let (primary_constraints, secondary_constraints) = pp.num_constraints();
        let (primary_variables, secondary_variables) = pp.num_variables();
        if u64::try_from(primary_constraints).map_err(|_| RecursiveV2Error::Limit)?
            != header.primary_constraints
            || u64::try_from(secondary_constraints).map_err(|_| RecursiveV2Error::Limit)?
                != header.secondary_constraints
            || u64::try_from(primary_variables).map_err(|_| RecursiveV2Error::Limit)?
                != header.primary_variables
            || u64::try_from(secondary_variables).map_err(|_| RecursiveV2Error::Limit)?
                != header.secondary_variables
        {
            return Err(RecursiveV2Error::Invariant);
        }
        Ok(Self { header, pp, vk })
    }

    fn decode_compressed_proof(&self, bytes: &[u8]) -> Result<NovaProof, RecursiveV2Error> {
        // `load` authenticated this exact immutable header before this method
        // can be reached; recheck the PP binding so future callers cannot
        // accidentally use a detached verifier object.
        if scalar_digest(self.pp.digest())? != self.header.pp_digest {
            return Err(RecursiveV2Error::Invariant);
        }
        decode_bincode::<NovaProof, MAX_NOVA_COMPRESSED_PROOF_BYTES_V2>(bytes)
    }

    fn verify(
        &self,
        proof: &NovaProof,
        steps: usize,
        initial_state: &[Scalar],
    ) -> Result<Vec<Scalar>, RecursiveV2Error> {
        if initial_state.len() != RUNNING_STATE_ARITY_V2
            || scalar_digest(self.pp.digest())? != self.header.pp_digest
        {
            return Err(RecursiveV2Error::Invariant);
        }
        proof
            .verify(&self.vk, steps, initial_state)
            .map_err(|_| RecursiveV2Error::Invariant)
    }
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

    sha256_256_role(
        CheckpointShaRole::Statement,
        &[
            b"z00z.recursive.v2.nova-owner-source",
            include_bytes!("nova.rs"),
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
    use super::{
        control_transition, measure_shape, nova_resource_preflight,
        nova_resource_preflight_from_shape, opcode_list, setup_public_parameters_after_preflight,
        CheckpointNovaAnchorsV2, CheckpointNovaCircuitV2, CheckpointRunningStateV2,
        CheckpointVerifierBundleV2, ControlPhaseV2, ControlTransitionRejectionV2,
        NovaResourceLimitsV2, NovaShapeMetricsV2, NovaStepWitnessV2, NovaTraceRootAuthorityV2,
        NovaTypedSourceEventV2, RecursiveAuthoritySnapshotV2, RecursiveCircuitProfileV2,
        RecursiveCircuitSpecV2, RecursiveV2Error, Scalar, VerifierBundleBindingV2,
        VerifierBundleHeaderV2, CONTROL_TRANSITION_TABLE_V2, MAX_NOVA_COMPRESSED_PROOF_BYTES_V2,
        MAX_NOVA_PUBLIC_PARAMETERS_BYTES_V2, MAX_NOVA_SETUP_AND_PROOF_RSS_BYTES_V2,
        MAX_NOVA_VERIFIER_BUNDLE_BYTES_V2, MAX_NOVA_VERIFIER_KEY_BYTES_V2,
        NOVA_PALLAS_AFFINE_WIRE_BYTES_V2, NOVA_RESOURCE_LIMITS_V2, RUNNING_STATE_ARITY_V2,
        VERIFIER_BUNDLE_HEADER_BYTES_V2,
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
    use tempfile::TempDir;

    use crate::{
        checkpoint::{
            recursive_context::RecursiveSnapshotHandleV2,
            recursive_trace::{
                decode_hash_control, structural_event_id, HashControlSchemaV2,
                RecursiveTraceEventV2, RecursiveTraceOpcodeV2, RecursiveTransitionTraceSourceV2,
            },
        },
        settlement::{derive_settlement_root_v2, RootGeneration},
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
            CheckpointVerifierBundleV2::load(bundle, binding).is_err(),
            "the bundle must reject before a proof decoder is reachable"
        );
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
    const FINALIZE: ExpectedControlEdgeV2 = ExpectedControlEdgeV2::Accept {
        next_phase: ControlPhaseV2::Idle,
        next_done: true,
    };

    // This matrix is deliberately not generated from `CONTROL_TRANSITION_TABLE_V2`.
    // It freezes the action-15 grammar independently, including every rejected
    // phase/done/opcode tuple and all explicit hash-control self-loops.
    const EXPECTED_CONTROL_EDGE_MATRIX_V2: [[ExpectedControlEdgeV2; 13]; 16] = [
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
            ILLEGAL, ILLEGAL, ILLEGAL, COMMIT, COMMIT, COMMIT, ILLEGAL, ILLEGAL, ILLEGAL, ILLEGAL,
            ILLEGAL, ILLEGAL, FINALIZE,
        ],
        [FINALIZED; 13],
    ];

    fn expected_control_edge(
        phase: ControlPhaseV2,
        done: bool,
        opcode: crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2,
    ) -> ExpectedControlEdgeV2 {
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

    fn canonical_hash_control_events() -> Vec<NovaTypedSourceEventV2> {
        use crate::checkpoint::recursive_trace::{
            decode_hash_control, emit_derived_hash_controls, structural_event_id,
            RecursiveTraceEventV2, RecursiveTraceOpcodeV2,
        };

        let profile = RecursiveCircuitProfileV2::repository_fixture();
        let payload = vec![0xA5_u8; 96];
        let source = RecursiveTraceEventV2::new(
            0,
            RecursiveTraceOpcodeV2::BeginBlock,
            structural_event_id(RecursiveTraceOpcodeV2::BeginBlock, 0, &payload),
            payload,
            &profile,
        )
        .expect("bounded source record");
        let mut controls = Vec::new();
        emit_derived_hash_controls(&source, &profile, |event| {
            controls.push(event);
            Ok(())
        })
        .expect("canonical derived control expansion");
        assert_eq!(
            controls.first().expect("begin control").opcode(),
            RecursiveTraceOpcodeV2::BeginHash
        );
        assert_eq!(
            controls.last().expect("end control").opcode(),
            RecursiveTraceOpcodeV2::EndHash
        );
        assert!(
            controls
                .iter()
                .filter(|event| event.opcode() == RecursiveTraceOpcodeV2::ShaBlock)
                .all(|event| decode_hash_control(event).is_ok()),
            "tests must consume the existing expander, not construct controls"
        );

        let mut events = Vec::with_capacity(controls.len() + 1);
        events.push(
            NovaTypedSourceEventV2::from_source(ControlPhaseV2::Idle, &source)
                .expect("source witness"),
        );
        for control in controls {
            events.push(
                NovaTypedSourceEventV2::from_source(ControlPhaseV2::Replay, &control)
                    .expect("decoded canonical control witness"),
            );
        }
        for index in 0..events.len().saturating_sub(1) {
            events[index] = events[index].clone().with_successor(&events[index + 1]);
        }
        let last = events.last().cloned().expect("source plus controls");
        *events.last_mut().expect("last event") = last.clone().with_successor(&last);
        events
    }

    fn canonical_hash_control_fixture() -> (
        CheckpointNovaAnchorsV2,
        Vec<NovaTypedSourceEventV2>,
        Vec<CheckpointRunningStateV2>,
        CheckpointRunningStateV2,
    ) {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let events = canonical_hash_control_events();
        let mut state =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
                .with_source_event(&events[0]);
        let mut states = Vec::with_capacity(events.len());
        for event in &events {
            states.push(state.clone());
            let circuit = source_circuit(anchors.clone(), event.clone());
            let (cs, output) = synthesize_test(&circuit, &state);
            assert!(
                cs.is_satisfied(),
                "canonical control step must satisfy R1CS"
            );
            state = state_from_output(&output);
        }
        (anchors, events, states, state)
    }

    fn global_trace_hash_control_fixture() -> (
        CheckpointNovaAnchorsV2,
        crate::checkpoint::recursive_trace::RecursiveTracePrecommitV2,
        Vec<NovaTypedSourceEventV2>,
        Vec<CheckpointRunningStateV2>,
        CheckpointRunningStateV2,
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
        let second_payload = vec![0x5A_u8; 80];
        let sources = [
            RecursiveTraceEventV2::new(
                0,
                RecursiveTraceOpcodeV2::BeginBlock,
                structural_event_id(RecursiveTraceOpcodeV2::BeginBlock, 0, &first_payload),
                first_payload,
                &profile,
            )
            .expect("first source"),
            RecursiveTraceEventV2::new(
                1,
                RecursiveTraceOpcodeV2::ReplayInput,
                structural_event_id(RecursiveTraceOpcodeV2::ReplayInput, 1, &second_payload),
                second_payload,
                &profile,
            )
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
        assert!(
            NovaTypedSourceEventV2::from_source(ControlPhaseV2::Idle, first_chunk).is_err(),
            "the pre-cutover Nova table must reject TraceChunk rather than treating it as a no-op"
        );

        // Byte chunks are now emitted by the production expander and consumed
        // by the independent native evaluator.  The existing global-SHA
        // fixture intentionally exercises only the already-constrained hash
        // schedule; it must not pretend that dropping the chunks is a Nova
        // execution.  The coupled TraceChunk→SHA test is added with the
        // successor circuit state machine, rather than creating a generic
        // no-op edge here.
        let mut phase = ControlPhaseV2::Idle;
        let mut events = Vec::with_capacity(expanded.len());
        for source_event in expanded
            .iter()
            .filter(|event| event.opcode() != RecursiveTraceOpcodeV2::TraceChunk)
        {
            events.push(
                NovaTypedSourceEventV2::from_source(phase, source_event)
                    .expect("canonical source/control witness"),
            );
            phase = control_transition(phase, false, source_event.opcode())
                .expect("canonical control transition")
                .next_phase;
        }
        let trace_controls = expanded
            .iter()
            .filter(|event| {
                matches!(
                    decode_hash_control(event),
                    Ok(control) if control.schema == HashControlSchemaV2::TracePrecommit
                )
            })
            .count();
        assert!(trace_controls >= 3, "global BEGIN/BLOCK*/END is present");
        for index in 0..events.len().saturating_sub(1) {
            events[index] = events[index].clone().with_successor(&events[index + 1]);
        }
        let last = events.last().cloned().expect("expanded source trace");
        *events.last_mut().expect("last event") = last.clone().with_successor(&last);

        let anchors = CheckpointNovaAnchorsV2::for_test();
        let authority = NovaTraceRootAuthorityV2::new([0_u64; super::DIGEST_LIMBS])
            .with_trace_precommit(precommit);
        let mut state =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
                .with_source_event(&events[0])
                .with_trace_authority(&authority);
        let mut states = Vec::with_capacity(events.len());
        for event in &events {
            states.push(state.clone());
            let circuit = source_circuit(anchors.clone(), event.clone());
            let (cs, output) = synthesize_test(&circuit, &state);
            assert!(
                cs.is_satisfied(),
                "canonical global trace control satisfies R1CS"
            );
            state = state_from_output(&output);
        }
        (anchors, precommit, events, states, state)
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
        assert_eq!(final_state.cells[super::SOURCE_TRACE_ORDINAL_CELL], 1);
        for index in super::COUNTERS_START..super::SHA_END {
            assert_eq!(final_state.cells[index], 0, "END_HASH clears cell {index}");
        }
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
    fn global_trace_hash_controls_bind_two_source_precommit() {
        let (_anchors, precommit, events, states, final_state) =
            global_trace_hash_control_fixture();
        let global_end = events
            .iter()
            .enumerate()
            .rev()
            .find(|(_, event)| {
                event.opcode == RecursiveTraceOpcodeV2::EndHash
                    && event.hash_control.schema == HashControlSchemaV2::TracePrecommit as u8
            })
            .expect("global END_HASH");
        assert_eq!(
            final_state.cells[super::SOURCE_TRACE_ORDINAL_CELL],
            precommit.event_count()
        );
        assert_eq!(
            final_state.cells[super::SOURCE_TRACE_BYTE_COUNT_CELL],
            precommit.byte_count()
        );
        assert_eq!(
            events[global_end.0].hash_control.source_hash,
            precommit.trace_digest()
        );
        for index in super::COUNTERS_START..super::SHA_END {
            assert_eq!(
                final_state.cells[index], 0,
                "global END_HASH clears cell {index}"
            );
        }
        assert_eq!(
            states[global_end.0].cells[super::SOURCE_TRACE_ORDINAL_CELL],
            2
        );
    }

    fn assert_global_mutation_reaches_rejection(
        anchors: CheckpointNovaAnchorsV2,
        events: &[NovaTypedSourceEventV2],
        initial: &CheckpointRunningStateV2,
        index: usize,
        mut mutate: impl FnMut(&mut NovaTypedSourceEventV2),
        label: &str,
    ) {
        let mut state = initial.clone();
        for (current, event) in events.iter().cloned().enumerate() {
            let mut event = event;
            if current == index {
                mutate(&mut event);
            }
            let circuit = source_circuit(anchors.clone(), event);
            let (cs, output) = synthesize_test(&circuit, &state);
            if !cs.is_satisfied() {
                return;
            }
            state = state_from_output(&output);
        }
        panic!("global trace mutation survived: {label}");
    }

    #[test]
    fn global_trace_hash_controls_reject_record_framing_and_statement_mutations() {
        let (anchors, _precommit, events, states, _) = global_trace_hash_control_fixture();
        let global_begin = events
            .iter()
            .position(|event| {
                event.opcode == RecursiveTraceOpcodeV2::BeginHash
                    && event.hash_control.schema == HashControlSchemaV2::TracePrecommit as u8
            })
            .expect("global BEGIN_HASH");
        let global_block = events
            .iter()
            .position(|event| {
                event.opcode == RecursiveTraceOpcodeV2::ShaBlock
                    && event.hash_control.schema == HashControlSchemaV2::TracePrecommit as u8
            })
            .expect("global SHA_BLOCK");
        let global_end = events
            .iter()
            .position(|event| {
                event.opcode == RecursiveTraceOpcodeV2::EndHash
                    && event.hash_control.schema == HashControlSchemaV2::TracePrecommit as u8
            })
            .expect("global END_HASH");

        assert_unsatisfied_hash_control(
            anchors.clone(),
            &states[0],
            events[0].tampered_payload(),
            "canonical source-record binding",
        );
        assert_global_mutation_reaches_rejection(
            anchors.clone(),
            &events,
            &states[0],
            0,
            |event| event.hash_control.source_record_bytes += 1,
            "source record length reaches the global byte-count binding",
        );

        let mut role = events[global_begin].clone();
        role.hash_control.role ^= 1;
        assert_unsatisfied_hash_control(anchors.clone(), &states[global_begin], role, "role");

        let mut schema = events[global_begin].clone();
        schema.hash_control.schema = HashControlSchemaV2::SourceRecord as u8;
        assert_unsatisfied_hash_control(
            anchors.clone(),
            &states[global_begin],
            schema,
            "schema discriminator",
        );

        let mut framed_bytes = events[global_begin].clone();
        framed_bytes.hash_control.message_bytes += 1;
        assert_unsatisfied_hash_control(
            anchors.clone(),
            &states[global_begin],
            framed_bytes,
            "role-framed message bytes",
        );

        let mut padding = events[global_begin].clone();
        padding.hash_control.trace_padding_bytes += 1;
        assert_unsatisfied_hash_control(
            anchors.clone(),
            &states[global_begin],
            padding,
            "FIPS padding bytes",
        );

        let mut bit_length = events[global_begin].clone();
        bit_length.hash_control.trace_bit_length += 8;
        assert_unsatisfied_hash_control(
            anchors.clone(),
            &states[global_begin],
            bit_length,
            "FIPS bit length",
        );

        let mut source_count = events[global_begin].clone();
        source_count.hash_control.trace_event_count += 1;
        assert_unsatisfied_hash_control(
            anchors.clone(),
            &states[global_begin],
            source_count,
            "source count",
        );

        let mut eof = events[global_begin].clone();
        eof.hash_control.trace_eof = false;
        assert_unsatisfied_hash_control(anchors.clone(), &states[global_begin], eof, "EOF");

        let mut block_bytes = events[global_block].clone();
        block_bytes.sha_compression.block[0] ^= 1;
        assert_unsatisfied_hash_control(
            anchors.clone(),
            &states[global_block],
            block_bytes,
            "global canonical block bytes",
        );

        assert_unsatisfied_hash_control(
            anchors.clone(),
            &states[global_begin],
            events[global_end].clone(),
            "BEGIN/BLOCK*/END ordering",
        );

        let mut final_digest = events[global_end].clone();
        final_digest.hash_control.source_hash[0] ^= 1;
        assert_unsatisfied_hash_control(
            anchors,
            &states[global_end],
            final_digest,
            "expected trace precommit digest",
        );
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
        assert_eq!(metrics.constraints, 47_645);
        assert_eq!(metrics.inputs, 1);
        assert_eq!(metrics.auxiliaries, 45_937);
        assert_eq!(metrics.nonzeros, 226_133);
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
        assert_eq!(plan.shape.constraints, 47_645);
        assert_eq!(plan.shape.auxiliaries, 45_937);
        assert_eq!(plan.shape.nonzeros, 226_133);
        assert_eq!(plan.ck_floor, 226_133);
        assert_eq!(plan.generator_count, 262_145);
        assert_eq!(plan.pp_payload_lower_bound, 18_577_584);
        assert_eq!(plan.vk_payload_lower_bound, 8_388_640);
        assert_eq!(plan.bundle_lower_bound, 26_966_714);
        assert_eq!(plan.pedersen_rss_lower_bound, 50_331_840);
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
            MAX_NOVA_SETUP_AND_PROOF_RSS_BYTES_V2,
        );
    }

    #[test]
    fn nova_resource_preflight_rejects_every_cap_plus_one_before_setup() {
        let metrics = NovaShapeMetricsV2 {
            constraints: 47_645,
            inputs: 1,
            auxiliaries: 45_937,
            nonzeros: 226_133,
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
        let mut finalizing_edges = 0_usize;

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
                                            | crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::JmtUpdate
                                    ),
                                    "only explicit action rows may preserve the control phase"
                                );
                            }
                            if edge.next_done {
                                finalizing_edges += 1;
                                assert_eq!(opcode, crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::FinalizeBlock);
                                assert_eq!(phase, ControlPhaseV2::Commit);
                                assert_eq!(edge.next_phase, ControlPhaseV2::Idle);
                            }
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
        assert_eq!(
            finalizing_edges, 1,
            "FINALIZE_BLOCK is the sole done=1 edge"
        );
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
    fn real_nova_global_trace_bundle_loads_and_verifies_compressed_proof() {
        let profile = RecursiveCircuitProfileV2::repository_fixture();
        let store =
            crate::settlement::SettlementStore::try_new().expect("fixture settlement store");
        let authority = RecursiveAuthoritySnapshotV2::resolve_repository_local_fixture(&store)
            .expect("fixture authority snapshot");
        let spec = RecursiveCircuitSpecV2::new(authority.authority().layout(), &profile)
            .expect("fixture circuit specification");
        let binding = VerifierBundleBindingV2::from_authority(&authority, &profile, &spec);

        let (anchors, precommit, events, states, _) = global_trace_hash_control_fixture();
        let state = states[0].clone();
        let circuit = source_circuit(anchors.clone(), events[0].clone());

        // Keep the fixed R1CS shape evidence on the same concrete circuit
        // before deriving Nova parameters for the bundle.
        let shape = shape_of(&circuit, &state);
        assert!(
            shape.0 > 0 && shape.1 > 0 && shape.2 > 0,
            "nonempty fixed shape"
        );

        let pp = setup_public_parameters_after_preflight(&circuit, &state)
            .expect("resource preflight and public parameters");
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
        let proof = Proof::prove(&pp, &pk, &recursive).expect("real compressed proof");

        // Measure exact encoded payloads before applying any existing cap. A
        // cap breach is evidence, not a reason to widen a private wire bound.
        let pp_payload = measured_bincode(&pp);
        let vk_payload = measured_bincode(&vk);
        let proof_payload = measured_bincode(&proof);
        eprintln!(
            "real Nova bundle payload bytes: pp={}, vk={}, proof={}, header={}",
            pp_payload.len(),
            vk_payload.len(),
            proof_payload.len(),
            VERIFIER_BUNDLE_HEADER_BYTES_V2,
        );
        assert!(
            pp_payload.len() <= MAX_NOVA_PUBLIC_PARAMETERS_BYTES_V2,
            "PP payload cap breach: {} > {}",
            pp_payload.len(),
            MAX_NOVA_PUBLIC_PARAMETERS_BYTES_V2
        );
        assert!(
            vk_payload.len() <= MAX_NOVA_VERIFIER_KEY_BYTES_V2,
            "VK payload cap breach: {} > {}",
            vk_payload.len(),
            MAX_NOVA_VERIFIER_KEY_BYTES_V2
        );
        assert!(
            proof_payload.len() <= MAX_NOVA_COMPRESSED_PROOF_BYTES_V2,
            "compressed-proof cap breach: {} > {}",
            proof_payload.len(),
            MAX_NOVA_COMPRESSED_PROOF_BYTES_V2
        );

        let bundle = CheckpointVerifierBundleV2::encode(&pp, &vk, binding)
            .expect("bundle under existing caps");
        eprintln!(
            "real Nova verifier bundle bytes: {} (cap {})",
            bundle.len(),
            MAX_NOVA_VERIFIER_BUNDLE_BYTES_V2,
        );
        assert_eq!(
            bundle.len(),
            VERIFIER_BUNDLE_HEADER_BYTES_V2 + pp_payload.len() + vk_payload.len(),
            "bundle must carry exactly the measured canonical PP and VK payloads"
        );
        assert!(bundle.len() <= MAX_NOVA_VERIFIER_BUNDLE_BYTES_V2);

        let loaded = CheckpointVerifierBundleV2::load(&bundle, binding)
            .expect("strict canonical bundle load");
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
        assert_eq!(
            output[super::PHASE_CELL],
            Scalar::from(ControlPhaseV2::Replay as u64)
        );
        assert_eq!(output[super::DONE_CELL], Scalar::from(0_u64));
        assert_eq!(
            output[super::ORDINAL_CELL],
            Scalar::from(u64::try_from(events.len()).expect("test step count fits u64"))
        );
        assert_eq!(
            output[super::SOURCE_TRACE_ORDINAL_CELL],
            Scalar::from(precommit.event_count()),
            "global controls preserve the canonical source-record count"
        );
        assert_eq!(
            output[super::SOURCE_TRACE_BYTE_COUNT_CELL],
            Scalar::from(precommit.byte_count()),
            "global controls bind the canonical source-record bytes"
        );
        for index in super::COUNTERS_START..super::SHA_END {
            assert_eq!(
                output[index],
                Scalar::from(0_u64),
                "END_HASH clears cell {index}"
            );
        }

        for mutate in [
            |header: &mut VerifierBundleHeaderV2| header.authority_digest[0] ^= 1,
            |header: &mut VerifierBundleHeaderV2| header.profile_digest[0] ^= 1,
            |header: &mut VerifierBundleHeaderV2| header.spec_digest[0] ^= 1,
            |header: &mut VerifierBundleHeaderV2| header.source_digest[0] ^= 1,
            |header: &mut VerifierBundleHeaderV2| header.lockfile_digest[0] ^= 1,
        ] {
            let mutated = mutate_bundle_header(&bundle, mutate);
            assert_bundle_rejected_before_proof_decode(&mutated, binding);
        }
        let mut pp_payload_mutation = bundle.clone();
        pp_payload_mutation[VERIFIER_BUNDLE_HEADER_BYTES_V2] ^= 1;
        assert_bundle_rejected_before_proof_decode(&pp_payload_mutation, binding);
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
                    panic!(
                        "TraceChunk has no legal Nova edge before its constrained schedule lands"
                    )
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
    fn real_nova_proof_binds_one_source_event() {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let first_event = source_event(
            ControlPhaseV2::Idle,
            crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::BeginBlock,
            0,
            b"typed-source-event-zero",
        );
        let next_event = source_event(
            ControlPhaseV2::Replay,
            crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::ReplayInput,
            1,
            b"typed-source-event-one",
        );
        let event = first_event;
        let state =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
                .with_source_event(&event);
        let circuit = source_circuit(anchors.clone(), event.clone());
        let (first_cs, _) = synthesize_test(&circuit, &state);
        assert!(
            first_cs.is_satisfied(),
            "first source event violates {}",
            first_cs
                .which_is_unsatisfied()
                .unwrap_or("an unknown constraint")
        );
        let pp = setup_public_parameters_after_preflight(&circuit, &state)
            .expect("resource preflight and public parameters");
        let mut recursive =
            RecursiveSNARK::new(&pp, &circuit, &state.scalars()).expect("event zero");
        recursive
            .prove_step(&pp, &circuit)
            .expect("first Nova step");
        assert_eq!(recursive.num_steps(), 1);
        let uncompressed_output = recursive
            .verify(&pp, 1, &state.scalars())
            .expect("uncompressed one-step verification");
        let (pk, vk) = Proof::setup(&pp).expect("compressed keys");
        let proof = Proof::prove(&pp, &pk, &recursive).expect("compressed proof");
        let output = proof
            .verify(&vk, 1, &state.scalars())
            .expect("compressed verification");
        assert_eq!(output, uncompressed_output);
        assert_eq!(output.len(), RUNNING_STATE_ARITY_V2);
        assert_eq!(
            output[super::PHASE_CELL],
            Scalar::from(ControlPhaseV2::Replay as u64)
        );
        assert_eq!(output[super::ORDINAL_CELL], Scalar::from(1_u64));
        assert_eq!(
            output[super::SOURCE_TRACE_ORDINAL_CELL],
            Scalar::from(1_u64)
        );
        assert_eq!(output[super::DONE_CELL], Scalar::from(0_u64));
        for (index, first) in event.payload_digest_limbs.into_iter().enumerate() {
            assert_eq!(
                output[super::SOURCE_TRACE_ROOT_START + index],
                Scalar::from(u64::from(first)),
                "source trace root limb {index}"
            );
        }

        let tampered_circuit = source_circuit(anchors.clone(), event.tampered_payload());
        let mut tampered = RecursiveSNARK::new(&pp, &tampered_circuit, &state.scalars())
            .expect("tampered event zero");
        tampered
            .prove_step(&pp, &tampered_circuit)
            .expect("tampered Nova step records event zero");
        assert!(
            tampered.verify(&pp, 1, &state.scalars()).is_err(),
            "the actual R1CS must reject a source-event payload digest mismatch"
        );

        let skipped_circuit = source_circuit(anchors.clone(), next_event.clone());
        let mut skipped = RecursiveSNARK::new(&pp, &skipped_circuit, &state.scalars())
            .expect("skipped event zero");
        skipped
            .prove_step(&pp, &skipped_circuit)
            .expect("skipped Nova step records event zero");
        assert!(
            skipped.verify(&pp, 1, &state.scalars()).is_err(),
            "the actual R1CS must reject a skipped source ordinal"
        );

        let reordered_event = source_event(
            ControlPhaseV2::Replay,
            crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::ReplayOutput,
            0,
            b"typed-source-event-reordered",
        );
        let reordered_circuit = source_circuit(anchors.clone(), reordered_event);
        let mut reordered = RecursiveSNARK::new(&pp, &reordered_circuit, &state.scalars())
            .expect("reordered event zero");
        reordered
            .prove_step(&pp, &reordered_circuit)
            .expect("reordered Nova step records event zero");
        assert!(
            reordered.verify(&pp, 1, &state.scalars()).is_err(),
            "the actual R1CS must reject a reordered source event"
        );

        let accumulator_state = state.clone().with_trace_root_limb(0, 1);
        let mut accumulator_tampered =
            RecursiveSNARK::new(&pp, &circuit, &accumulator_state.scalars())
                .expect("accumulator-tampered event zero");
        accumulator_tampered
            .prove_step(&pp, &circuit)
            .expect("accumulator-tampered Nova step records event zero");
        assert!(
            accumulator_tampered
                .verify(&pp, 1, &state.scalars())
                .is_err(),
            "the actual verifier must bind the source trace accumulator state"
        );
    }

    #[test]
    fn source_record_rejects_a_second_record_before_hash_completion() {
        let anchors = CheckpointNovaAnchorsV2::for_test();
        let first = source_event(
            ControlPhaseV2::Idle,
            crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::BeginBlock,
            0,
            b"typed-source-event-zero",
        );
        let second = source_event(
            ControlPhaseV2::Replay,
            crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2::ReplayInput,
            1,
            b"typed-source-event-one",
        );
        let first = first.with_successor(&second);
        let initial =
            CheckpointRunningStateV2::with_control(&anchors, ControlPhaseV2::Idle, 0, false)
                .with_source_event(&first);
        let first_circuit = source_circuit(anchors.clone(), first);
        let (first_cs, first_output) = synthesize_test(&first_circuit, &initial);
        assert!(first_cs.is_satisfied());
        let second_circuit = source_circuit(anchors, second);
        let (second_cs, _) = synthesize_test(&second_circuit, &state_from_output(&first_output));
        assert!(
            !second_cs.is_satisfied(),
            "a second source record must not bypass BEGIN_HASH/SHA_BLOCK*/END_HASH"
        );
        assert_eq!(
            second_cs.which_is_unsatisfied(),
            Some("hash_control_schedule/source_schedule_input_zero_223/selected value equals constant")
        );
    }

    #[test]
    fn real_nova_finalization_binds_authority_trace_root() {
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
        let pp = setup_public_parameters_after_preflight(&final_circuit, &final_state)
            .expect("resource preflight and public parameters");
        let mut recursive = RecursiveSNARK::new(&pp, &final_circuit, &final_state.scalars())
            .expect("final event zero");
        recursive
            .prove_step(&pp, &final_circuit)
            .expect("final Nova step");
        let (pk, vk) = Proof::setup(&pp).expect("compressed keys");
        let proof = Proof::prove(&pp, &pk, &recursive).expect("compressed proof");
        let output = proof
            .verify(&vk, 1, &final_state.scalars())
            .expect("authority-bound final verification");
        assert_eq!(
            output[super::PHASE_CELL],
            Scalar::from(ControlPhaseV2::Idle as u64)
        );
        assert_eq!(output[super::DONE_CELL], Scalar::from(1_u64));
        for (index, expected) in expected_root.into_iter().enumerate() {
            assert_eq!(
                output[super::SOURCE_TRACE_ROOT_START + index],
                Scalar::from(expected),
                "final source root limb {index}"
            );
        }

        let early_root = std::array::from_fn(|index| {
            expected_root[index]
                .checked_add(1)
                .expect("test early root remains within u64")
        });
        let early_authority = NovaTraceRootAuthorityV2::new(early_root);
        let early_state = final_state.clone().with_trace_authority(&early_authority);
        let mut early = RecursiveSNARK::new(&pp, &final_circuit, &early_state.scalars())
            .expect("early final event zero");
        early
            .prove_step(&pp, &final_circuit)
            .expect("early final Nova step records event zero");
        assert!(
            early.verify(&pp, 1, &early_state.scalars()).is_err(),
            "the actual R1CS must reject an early final root"
        );

        let mut tampered_root = expected_root;
        tampered_root[0] ^= 1;
        let tampered_authority = NovaTraceRootAuthorityV2::new(tampered_root);
        let tampered_state = final_state.with_trace_authority(&tampered_authority);
        let mut tampered = RecursiveSNARK::new(&pp, &final_circuit, &tampered_state.scalars())
            .expect("tampered final event zero");
        tampered
            .prove_step(&pp, &final_circuit)
            .expect("tampered final Nova step records event zero");
        assert!(
            tampered.verify(&pp, 1, &tampered_state.scalars()).is_err(),
            "the actual R1CS must reject a tampered authority trace root"
        );
    }
}
