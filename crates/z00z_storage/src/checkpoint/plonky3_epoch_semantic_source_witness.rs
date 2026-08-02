//! Witness construction for the canonical epoch semantic-source parser.

use p3_field::{Field, PrimeCharacteristicRing};
use p3_koala_bear::KoalaBear;

use super::plonky3_epoch_event_stream::transition_event_stream;
use super::plonky3_epoch_semantic_source_air::*;
use super::plonky3_epoch_uniqueness_range::UniquenessRangeQueryV2;
use super::plonky3_epoch_uniqueness_slice::EpochUniquenessSliceV2;
use super::{
    decode_flow_item, decode_net_effect, decode_uniqueness_precommit, decode_uniqueness_sorted_row,
    EpochAirTableV2, EpochPreparedTransitionV2, EpochTraceChunkV2, EpochTransitionBindingV2,
    RecursiveCheckpointRejectReasonV2, RecursiveTraceOpcodeV2, UniquenessListKindV2,
    UniquenessPassV2, UniquenessSetKindV2, EPOCH_CHUNK_BYTES_V2,
};
use crate::checkpoint::recursive_semantics::NetEffectKindV2;
use crate::CheckpointError;

fn semantic_trace_rows(real_rows: usize) -> Result<usize, CheckpointError> {
    real_rows
        .checked_add(1)
        .and_then(usize::checked_next_power_of_two)
        .ok_or(CheckpointError::Overflow)
}

#[derive(Clone, Copy)]
enum SourcePhaseV2 {
    Prefix(usize),
    Length(usize),
    Header(usize),
    Payload(usize),
}

#[derive(Clone, Copy)]
struct EventMetaV2 {
    event_index: usize,
    event_len_bytes: [u8; 4],
    payload_len_bytes: [u8; 4],
    ordinal_bytes: [u8; 8],
    opcode: RecursiveTraceOpcodeV2,
    uniqueness_class: Option<usize>,
    net_kind: Option<usize>,
    net_terminal_id: Option<[u8; 32]>,
    global_product_pair: bool,
    payload_len: usize,
}

#[derive(Clone, Copy)]
struct SourceStateV2 {
    jmt_stage: usize,
    jmt_count: usize,
    uniqueness_counters: [usize; UNIQUENESS_COUNTER_COUNT_V2],
    net_effect_counter: usize,
    net_mutation_counter: usize,
    fixed_event_counters: [usize; FIXED_EVENT_COUNTER_COUNT_V2],
    declared_count_bytes: [u8; DECLARED_COUNT_BYTE_COUNT_V2],
    replay_counters: [usize; REPLAY_COUNTER_COUNT_V2],
    flow_root_low_byte: u8,
}

#[derive(Clone, Copy)]
struct ReplayRowMetaV2 {
    phase: usize,
    remaining: usize,
    tx_len_low: u8,
    tx_len_high: u8,
    hex_nibble: Option<u8>,
    hex_low: bool,
    hex_high_nibble: u8,
    ascii_low: u8,
    ascii_high: u8,
    semantic_index: usize,
}

#[derive(Clone, Copy)]
struct FlowRowMetaV2 {
    phase: usize,
    hex_nibble: Option<u8>,
    hex_low: bool,
    hex_high_nibble: u8,
    hex_byte_index: usize,
    root_limb_index: usize,
    root_byte_parity: bool,
}

pub(super) struct SemanticSourceWitnessV2 {
    pub(super) trace: SemanticSourceTraceV2,
    pub(super) range_queries: Vec<UniquenessRangeQueryV2>,
}

fn uniqueness_class(
    pass: UniquenessPassV2,
    set: UniquenessSetKindV2,
    list: UniquenessListKindV2,
) -> usize {
    usize::from(pass == UniquenessPassV2::Product) * 4
        + usize::from(list == UniquenessListKindV2::Sorted) * 2
        + usize::from(set == UniquenessSetKindV2::Output)
}

fn append_bits(values: &mut Vec<KoalaBear>, byte: u8) {
    values.extend((0..8).map(|bit| KoalaBear::from_bool((byte >> bit) & 1 == 1)));
}

fn replay_hex_nibble(byte: u8) -> Result<u8, CheckpointError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(CheckpointError::Canonical),
    }
}

fn replay_row_meta(
    payload: &[u8],
    payload_index: usize,
) -> Result<ReplayRowMetaV2, CheckpointError> {
    let tx_len = usize::from(u16::from_le_bytes([
        *payload.get(1).ok_or(CheckpointError::Canonical)?,
        *payload.get(2).ok_or(CheckpointError::Canonical)?,
    ]));
    let tx_start = 3_usize;
    let tx_end = tx_start
        .checked_add(tx_len)
        .ok_or(CheckpointError::Overflow)?;
    let definition_len_start = tx_end;
    let definition_hex_start = definition_len_start
        .checked_add(2)
        .ok_or(CheckpointError::Overflow)?;
    let definition_hex_end = definition_hex_start
        .checked_add(64)
        .ok_or(CheckpointError::Overflow)?;
    let serial_start = definition_hex_end;
    let serial_end = serial_start
        .checked_add(4)
        .ok_or(CheckpointError::Overflow)?;
    let terminal_len_start = serial_end;
    let terminal_hex_start = terminal_len_start
        .checked_add(2)
        .ok_or(CheckpointError::Overflow)?;
    let terminal_hex_end = terminal_hex_start
        .checked_add(64)
        .ok_or(CheckpointError::Overflow)?;
    let leaf_hash_start = terminal_hex_end;
    let leaf_hash_end = leaf_hash_start
        .checked_add(32)
        .ok_or(CheckpointError::Overflow)?;
    let leaf_kind_index = leaf_hash_end;
    let flags_start = leaf_kind_index
        .checked_add(1)
        .ok_or(CheckpointError::Overflow)?;
    let flags_end = flags_start
        .checked_add(3)
        .ok_or(CheckpointError::Overflow)?;
    if payload.len() != flags_end || payload_index >= flags_end {
        return Err(CheckpointError::Canonical);
    }

    let (phase, remaining, semantic_index) = match payload_index {
        0 => (REPLAY_OP_KIND_PHASE_V2, 1, 0),
        1 => (REPLAY_TX_LEN_LOW_PHASE_V2, 1, 0),
        2 => (REPLAY_TX_LEN_HIGH_PHASE_V2, 1, 0),
        index if (tx_start..tx_end).contains(&index) => {
            (REPLAY_TX_BYTES_PHASE_V2, tx_end - index, 0)
        }
        index if index == definition_len_start => (REPLAY_DEFINITION_LEN_LOW_PHASE_V2, 1, 0),
        index if index == definition_len_start + 1 => (REPLAY_DEFINITION_LEN_HIGH_PHASE_V2, 1, 0),
        index if (definition_hex_start..definition_hex_end).contains(&index) => {
            let relative = index - definition_hex_start;
            (
                REPLAY_DEFINITION_HEX_PHASE_V2,
                definition_hex_end - index,
                if relative % 2 == 1 { relative / 2 } else { 0 },
            )
        }
        index if (serial_start..serial_end).contains(&index) => (
            REPLAY_SERIAL_PHASE_V2,
            serial_end - index,
            32 + index - serial_start,
        ),
        index if index == terminal_len_start => (REPLAY_TERMINAL_LEN_LOW_PHASE_V2, 1, 0),
        index if index == terminal_len_start + 1 => (REPLAY_TERMINAL_LEN_HIGH_PHASE_V2, 1, 0),
        index if (terminal_hex_start..terminal_hex_end).contains(&index) => {
            let relative = index - terminal_hex_start;
            (
                REPLAY_TERMINAL_HEX_PHASE_V2,
                terminal_hex_end - index,
                if relative % 2 == 1 {
                    36 + relative / 2
                } else {
                    0
                },
            )
        }
        index if (leaf_hash_start..leaf_hash_end).contains(&index) => (
            REPLAY_LEAF_HASH_PHASE_V2,
            leaf_hash_end - index,
            68 + index - leaf_hash_start,
        ),
        index if index == leaf_kind_index => (REPLAY_LEAF_KIND_PHASE_V2, 1, 0),
        index if (flags_start..flags_end).contains(&index) => {
            (REPLAY_FLAGS_PHASE_V2, flags_end - index, 0)
        }
        _ => return Err(CheckpointError::Canonical),
    };
    let byte = payload[payload_index];
    let (hex_nibble, hex_low, hex_high_nibble) = if matches!(
        phase,
        REPLAY_DEFINITION_HEX_PHASE_V2 | REPLAY_TERMINAL_HEX_PHASE_V2
    ) {
        let segment_start = if phase == REPLAY_DEFINITION_HEX_PHASE_V2 {
            definition_hex_start
        } else {
            terminal_hex_start
        };
        let relative = payload_index - segment_start;
        let nibble = replay_hex_nibble(byte)?;
        let low = relative % 2 == 1;
        let high = if low {
            replay_hex_nibble(payload[payload_index - 1])?
        } else {
            nibble
        };
        (Some(nibble), low, high)
    } else {
        (None, false, 0)
    };
    let (ascii_low, ascii_high) = if phase == REPLAY_TX_BYTES_PHASE_V2 {
        (
            byte.checked_sub(33).ok_or(CheckpointError::Canonical)?,
            126_u8.checked_sub(byte).ok_or(CheckpointError::Canonical)?,
        )
    } else {
        (0, 0)
    };
    Ok(ReplayRowMetaV2 {
        phase,
        remaining,
        tx_len_low: payload[1],
        tx_len_high: payload[2],
        hex_nibble,
        hex_low,
        hex_high_nibble,
        ascii_low,
        ascii_high,
        semantic_index,
    })
}

fn flow_row_meta(payload: &[u8], payload_index: usize) -> Result<FlowRowMetaV2, CheckpointError> {
    if payload.len() != FLOW_HEADER_BYTES_V2 || payload_index >= payload.len() {
        return Err(CheckpointError::Canonical);
    }
    let canonical_hex_len = 64_u16.to_le_bytes();
    for prefix in [0_usize, 78, 144, 210] {
        if payload[prefix..prefix + 2] != canonical_hex_len {
            return Err(CheckpointError::Canonical);
        }
    }
    let (phase, hex_start) = match payload_index {
        0 => (0, None),
        1 => (1, None),
        2..=65 => (2, Some(2)),
        66..=77 => (3, None),
        78 => (4, None),
        79 => (5, None),
        80..=143 => (6, Some(80)),
        144 => (7, None),
        145 => (8, None),
        146..=209 => (9, Some(146)),
        210 => (10, None),
        211 => (11, None),
        212..=275 => (12, Some(212)),
        276..=283 => (13, None),
        _ => return Err(CheckpointError::Canonical),
    };
    let (hex_nibble, hex_low, hex_high_nibble, hex_byte_index) = if let Some(start) = hex_start {
        let relative = payload_index
            .checked_sub(start)
            .ok_or(CheckpointError::Invariant)?;
        let nibble = replay_hex_nibble(payload[payload_index])?;
        let low = relative % 2 == 1;
        let high = if low {
            replay_hex_nibble(payload[payload_index - 1])?
        } else {
            nibble
        };
        // The AIR exposes the decoded-byte index only on the low-nibble row,
        // where the two hex characters are emitted as one canonical byte.
        // Keeping it zero on high-nibble rows prevents unconstrained duplicate
        // positions from entering the flow-root and mutation projections.
        (Some(nibble), low, high, if low { relative / 2 } else { 0 })
    } else {
        (None, false, 0, 0)
    };
    let root = matches!(phase, 9 | 12) && hex_low;
    Ok(FlowRowMetaV2 {
        phase,
        hex_nibble,
        hex_low,
        hex_high_nibble,
        hex_byte_index,
        root_limb_index: if root { hex_byte_index / 2 } else { 0 },
        root_byte_parity: root && hex_byte_index % 2 == 1,
    })
}

fn declared_count_slack(count: u32) -> Result<[u8; 2], CheckpointError> {
    u16::try_from(
        u32::try_from(DECLARED_ITEM_LIMIT_V2)
            .map_err(|_| CheckpointError::Limit)?
            .checked_sub(count)
            .ok_or(CheckpointError::Limit)?,
    )
    .map(u16::to_le_bytes)
    .map_err(|_| CheckpointError::Limit)
}

fn source_row(
    role: SemanticSourceAirRoleV2,
    slot: usize,
    byte_index: usize,
    byte: u8,
    phase: SourcePhaseV2,
    meta: Option<EventMetaV2>,
    slot_end: bool,
    state: SourceStateV2,
    replay: Option<ReplayRowMetaV2>,
    flow: Option<FlowRowMetaV2>,
) -> Result<SemanticSourceRowV2, CheckpointError> {
    let mut values = Vec::with_capacity(ROW_FIELDS_V2);
    values.push(KoalaBear::ONE);
    values
        .extend((0..TRANSITION_SLOTS_V2).map(|candidate| KoalaBear::from_bool(candidate == slot)));
    values.extend((0..PREFIX_BYTES_V2).map(|candidate| {
        KoalaBear::from_bool(matches!(phase, SourcePhaseV2::Prefix(index) if index == candidate))
    }));
    values.extend((0..LENGTH_BYTES_V2).map(|candidate| {
        KoalaBear::from_bool(matches!(phase, SourcePhaseV2::Length(index) if index == candidate))
    }));
    values.extend((0..HEADER_BYTES_V2).map(|candidate| {
        KoalaBear::from_bool(matches!(phase, SourcePhaseV2::Header(index) if index == candidate))
    }));
    values.push(KoalaBear::from_bool(matches!(
        phase,
        SourcePhaseV2::Payload(_)
    )));
    values.push(KoalaBear::from_usize(byte_index));
    values.push(KoalaBear::from_u8(byte));
    append_bits(&mut values, byte);

    if let Some(meta) = meta {
        values.push(KoalaBear::from_usize(meta.event_index));
        values.extend(meta.event_len_bytes.map(KoalaBear::from_u8));
        values.extend(meta.payload_len_bytes.map(KoalaBear::from_u8));
        values.extend(meta.ordinal_bytes.map(KoalaBear::from_u8));
        for opcode in 1..=OPCODE_COUNT_V2 {
            values.push(KoalaBear::from_bool(opcode == meta.opcode as usize));
        }
        let payload_index = match phase {
            SourcePhaseV2::Payload(index) => index,
            _ => 0,
        };
        values.push(KoalaBear::from_usize(payload_index));
        values.push(KoalaBear::from_bool(meta.payload_len != 0));
        values.push(if meta.payload_len == 0 {
            KoalaBear::ZERO
        } else {
            KoalaBear::from_usize(meta.payload_len).inverse()
        });
        values.push(KoalaBear::from_bool(
            matches!(phase, SourcePhaseV2::Payload(index) if index + 1 == meta.payload_len),
        ));
        values.push(KoalaBear::from_bool(slot_end));
        for candidate in 0..PAYLOAD_PREFIX_BYTES_V2 {
            values.push(KoalaBear::from_bool(
                matches!(phase, SourcePhaseV2::Payload(index) if index == candidate),
            ));
        }
    } else {
        values.extend(core::iter::repeat_n(
            KoalaBear::ZERO,
            JMT_STAGE_SELECTOR_OFFSET_V2 - EVENT_INDEX_OFFSET_V2,
        ));
    }

    for candidate in 0..JMT_STAGE_COUNT_V2 {
        values.push(KoalaBear::from_bool(candidate == state.jmt_stage));
    }
    values.push(KoalaBear::from_usize(state.jmt_count));
    values.extend(state.uniqueness_counters.map(KoalaBear::from_usize));
    let class = meta.and_then(|meta| meta.uniqueness_class);
    for candidate in 0..UNIQUENESS_CLASS_COUNT_V2 {
        values.push(KoalaBear::from_bool(class == Some(candidate)));
    }
    let net_kind = meta.and_then(|meta| meta.net_kind);
    for candidate in 0..NET_KIND_COUNT_V2 {
        values.push(KoalaBear::from_bool(net_kind == Some(candidate)));
    }
    values.push(KoalaBear::from_usize(state.net_effect_counter));
    values.push(KoalaBear::from_usize(state.net_mutation_counter));
    let net_terminal_id = meta.and_then(|meta| meta.net_terminal_id);
    for limb in 0..NET_TERMINAL_LIMB_COUNT_V2 {
        values.push(net_terminal_id.map_or(KoalaBear::ZERO, |terminal_id| {
            KoalaBear::from_u16(u16::from_le_bytes([
                terminal_id[limb * 2],
                terminal_id[limb * 2 + 1],
            ]))
        }));
    }
    let payload_index = match phase {
        SourcePhaseV2::Payload(index) => Some(index),
        _ => None,
    };
    for terminal_byte in 0..NET_TERMINAL_BYTE_SELECTOR_COUNT_V2 {
        values.push(KoalaBear::from_bool(
            net_terminal_id.is_some()
                && payload_index == Some(NET_TERMINAL_PAYLOAD_START_V2 + terminal_byte),
        ));
    }
    let net_terminal_countdown = payload_index
        .filter(|index| {
            net_terminal_id.is_some()
                && (PAYLOAD_PREFIX_BYTES_V2..NET_TERMINAL_PAYLOAD_START_V2).contains(index)
        })
        .map_or(0, |index| NET_TERMINAL_PAYLOAD_START_V2 - index);
    let net_terminal_countdown = KoalaBear::from_usize(net_terminal_countdown);
    values.push(net_terminal_countdown);
    values.push(if net_terminal_countdown.is_zero() {
        KoalaBear::ZERO
    } else {
        net_terminal_countdown.inverse()
    });
    values.push(KoalaBear::from_bool(!net_terminal_countdown.is_zero()));
    values.extend(state.fixed_event_counters.map(KoalaBear::from_usize));
    values.extend(state.declared_count_bytes.map(KoalaBear::from_u8));
    values.push(KoalaBear::from_bool(
        meta.is_some_and(|meta| meta.global_product_pair),
    ));
    let flow_header = meta.is_some_and(|meta| {
        matches!(
            meta.opcode,
            RecursiveTraceOpcodeV2::BeginBlock | RecursiveTraceOpcodeV2::FinalizeBlock
        )
    });
    for count_byte in 0..FLOW_COUNT_BYTE_SELECTOR_COUNT_V2 {
        values.push(KoalaBear::from_bool(
            flow_header && payload_index == Some(FLOW_HEADER_COUNT_PAYLOAD_START_V2 + count_byte),
        ));
    }
    let flow_count_countdown = payload_index
        .filter(|index| {
            flow_header
                && (PAYLOAD_PREFIX_BYTES_V2..FLOW_HEADER_COUNT_PAYLOAD_START_V2).contains(index)
        })
        .map_or(0, |index| FLOW_HEADER_COUNT_PAYLOAD_START_V2 - index);
    let flow_count_countdown = KoalaBear::from_usize(flow_count_countdown);
    values.push(flow_count_countdown);
    values.push(if flow_count_countdown.is_zero() {
        KoalaBear::ZERO
    } else {
        flow_count_countdown.inverse()
    });
    values.push(KoalaBear::from_bool(!flow_count_countdown.is_zero()));
    let net_mutation_count = KoalaBear::from_usize(state.net_mutation_counter);
    values.push(KoalaBear::from_bool(state.net_mutation_counter != 0));
    values.push(if net_mutation_count.is_zero() {
        KoalaBear::ZERO
    } else {
        net_mutation_count.inverse()
    });
    let declared_spent_count = u32::from_le_bytes(
        state.declared_count_bytes[..4]
            .try_into()
            .map_err(|_| CheckpointError::Invariant)?,
    );
    let declared_output_count = u32::from_le_bytes(
        state.declared_count_bytes[4..]
            .try_into()
            .map_err(|_| CheckpointError::Invariant)?,
    );
    for count in [declared_spent_count, declared_output_count] {
        let count = KoalaBear::from_u64(u64::from(count));
        values.push(if slot_end && !count.is_zero() {
            count.inverse()
        } else {
            KoalaBear::ZERO
        });
    }
    if role == SemanticSourceAirRoleV2::Uniqueness && slot_end {
        for count in [declared_spent_count, declared_output_count] {
            values.extend(declared_count_slack(count)?.map(KoalaBear::from_u8));
        }
    } else {
        values.extend([KoalaBear::ZERO; DECLARED_COUNT_SLACK_BYTE_COUNT_V2]);
    }
    if let Some(replay) = replay {
        for candidate in 0..REPLAY_PHASE_COUNT_V2 {
            values.push(KoalaBear::from_bool(candidate == replay.phase));
        }
        values.push(KoalaBear::from_usize(replay.remaining));
        values.push(if replay.remaining == 1 {
            KoalaBear::ZERO
        } else {
            (KoalaBear::from_usize(replay.remaining) - KoalaBear::ONE).inverse()
        });
        values.push(KoalaBear::from_bool(replay.remaining == 1));
        values.push(KoalaBear::from_u8(replay.tx_len_low));
        values.push(KoalaBear::from_u8(replay.tx_len_high));
        values.push(
            (KoalaBear::from_u8(replay.tx_len_low)
                + KoalaBear::from_u8(replay.tx_len_high) * KoalaBear::from_u64(256))
            .inverse(),
        );
        for candidate in 0..REPLAY_HEX_SELECTOR_COUNT_V2 {
            values.push(KoalaBear::from_bool(
                replay.hex_nibble
                    == Some(u8::try_from(candidate).map_err(|_| CheckpointError::Limit)?),
            ));
        }
        values.push(KoalaBear::from_bool(replay.hex_low));
        values.push(KoalaBear::from_u8(replay.hex_high_nibble));
        values.push(KoalaBear::from_u8(replay.ascii_low));
        values.push(KoalaBear::from_u8(replay.ascii_high));
        values.push(KoalaBear::from_usize(replay.semantic_index));
    } else {
        values.extend(core::iter::repeat_n(
            KoalaBear::ZERO,
            REPLAY_COUNTER_OFFSET_V2 - REPLAY_PHASE_SELECTOR_OFFSET_V2,
        ));
    }
    values.extend(state.replay_counters.map(KoalaBear::from_usize));
    if let Some(flow) = flow {
        for candidate in 0..FLOW_PHASE_COUNT_V2 {
            values.push(KoalaBear::from_bool(candidate == flow.phase));
        }
        for candidate in 0..FLOW_HEX_SELECTOR_COUNT_V2 {
            values.push(KoalaBear::from_bool(
                flow.hex_nibble
                    == Some(u8::try_from(candidate).map_err(|_| CheckpointError::Limit)?),
            ));
        }
        values.push(KoalaBear::from_bool(flow.hex_low));
        values.push(KoalaBear::from_u8(flow.hex_high_nibble));
        values.push(KoalaBear::from_usize(flow.hex_byte_index));
        values.push(KoalaBear::from_usize(flow.root_limb_index));
        values.push(KoalaBear::from_bool(flow.root_byte_parity));
    } else {
        values.extend(core::iter::repeat_n(
            KoalaBear::ZERO,
            FLOW_ROOT_LOW_BYTE_OFFSET_V2 - FLOW_PHASE_SELECTOR_OFFSET_V2,
        ));
    }
    values.push(KoalaBear::from_u8(state.flow_root_low_byte));
    let transcript_phase = if role == SemanticSourceAirRoleV2::Uniqueness {
        meta.and_then(|meta| {
            let payload_index = payload_index?;
            let phases = match meta.opcode {
                RecursiveTraceOpcodeV2::UniquenessPrecommit => 0..=PRECOMMIT_DIGEST_PHASE_V2,
                RecursiveTraceOpcodeV2::UniquenessChallenge => {
                    CHALLENGE_VERSION_PHASE_V2..=CHALLENGE_DIGEST_LAST_PHASE_V2
                }
                RecursiveTraceOpcodeV2::NetMerge
                    if meta.net_kind == Some(NetEffectKindV2::Close as usize) =>
                {
                    CLOSE_HEADER_PHASE_V2..=CLOSE_OUTPUT_PRECOMMIT_PHASE_V2
                }
                _ => return None,
            };
            phases.into_iter().find(|phase| {
                (TRANSCRIPT_PHASE_STARTS_V2[*phase]..=TRANSCRIPT_PHASE_ENDS_V2[*phase])
                    .contains(&payload_index)
            })
        })
    } else {
        None
    };
    for phase in 0..TRANSCRIPT_PHASE_COUNT_V2 {
        values.push(KoalaBear::from_bool(transcript_phase == Some(phase)));
    }
    let transcript_phase_final = transcript_phase
        .is_some_and(|phase| payload_index == Some(TRANSCRIPT_PHASE_ENDS_V2[phase]));
    values.push(KoalaBear::from_bool(transcript_phase_final));
    let transcript_phase_distance = transcript_phase.map_or(0, |phase| {
        TRANSCRIPT_PHASE_ENDS_V2[phase]
            .checked_sub(payload_index.expect("transcript phase owns a payload index"))
            .expect("transcript phase range is canonical")
    });
    values.push(if transcript_phase_distance == 0 {
        KoalaBear::ZERO
    } else {
        KoalaBear::from_usize(transcript_phase_distance).inverse()
    });
    let transcript_pair = transcript_phase
        .filter(|phase| {
            TRANSCRIPT_PHASE_ENDS_V2[*phase] - TRANSCRIPT_PHASE_STARTS_V2[*phase] + 1 == 32
        })
        .map(|phase| {
            payload_index.expect("transcript phase owns a payload index")
                - TRANSCRIPT_PHASE_STARTS_V2[phase]
        });
    values.push(KoalaBear::from_usize(
        transcript_pair.map_or(0, |index| index / 2),
    ));
    values.push(KoalaBear::from_bool(
        transcript_pair.is_some_and(|index| index % 2 == 1),
    ));
    if values.len() != ROW_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(SemanticSourceRowV2 { values })
}

pub(super) fn public_values(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
) -> Result<Vec<KoalaBear>, CheckpointError> {
    public_values_for_slice(
        statement,
        bindings,
        EpochUniquenessSliceV2::full(bindings.len())?,
    )
}

pub(super) fn public_values_for_slice(
    statement: &EpochTraceChunkV2,
    full_bindings: &[EpochTransitionBindingV2],
    slice: EpochUniquenessSliceV2,
) -> Result<Vec<KoalaBear>, CheckpointError> {
    let end = slice.end()?;
    let bindings = full_bindings
        .get(slice.start()..end)
        .ok_or(CheckpointError::Canonical)?;
    if statement.canonical_bytes().len() != EPOCH_CHUNK_BYTES_V2
        || statement.inputs().table != EpochAirTableV2::PackedRange
        || statement.inputs().replica != 0
        || bindings.is_empty()
        || bindings.len() > TRANSITION_SLOTS_V2
    {
        return Err(CheckpointError::Canonical);
    }
    let mut values = Vec::with_capacity(PUBLIC_FIELDS_V2);
    values.extend(
        statement
            .canonical_bytes()
            .chunks_exact(2)
            .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
    );
    for slot in 0..TRANSITION_SLOTS_V2 {
        values.push(KoalaBear::from_bool(slot < bindings.len()));
    }
    for slot in 0..TRANSITION_SLOTS_V2 {
        values.push(KoalaBear::from_u64(
            bindings
                .get(slot)
                .map(|binding| binding.inputs().event_bytes)
                .unwrap_or(0),
        ));
    }
    for slot in 0..TRANSITION_SLOTS_V2 {
        values.push(KoalaBear::from_u64(
            bindings
                .get(slot)
                .map(|binding| binding.inputs().event_count)
                .unwrap_or(0),
        ));
    }
    for slot in 0..TRANSITION_SLOTS_V2 {
        values.extend(
            bindings
                .get(slot)
                .map(|binding| binding.inputs().event_count.to_le_bytes())
                .unwrap_or([0; 8])
                .map(KoalaBear::from_u8),
        );
    }
    let binding_digests: [fn(EpochTransitionBindingV2) -> [u8; 32]; 4] = [
        |binding: EpochTransitionBindingV2| binding.inputs().pre_uniqueness_context_digest,
        |binding: EpochTransitionBindingV2| binding.inputs().spent_uniqueness_precommit,
        |binding: EpochTransitionBindingV2| binding.inputs().output_uniqueness_precommit,
        |binding: EpochTransitionBindingV2| binding.inputs().event_vector_digest,
    ];
    for digest in binding_digests {
        for slot in 0..TRANSITION_SLOTS_V2 {
            values.extend(
                bindings
                    .get(slot)
                    .copied()
                    .map(digest)
                    .unwrap_or([0; 32])
                    .map(KoalaBear::from_u8),
            );
        }
    }
    values.push(KoalaBear::from_usize(slice.start()));
    values.push(KoalaBear::from_usize(slice.len()));
    if values.len() != PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

pub(super) fn witness(
    role: SemanticSourceAirRoleV2,
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
) -> Result<SemanticSourceWitnessV2, CheckpointError> {
    witness_for_slice(
        role,
        statement,
        bindings,
        prepared,
        EpochUniquenessSliceV2::full(bindings.len())?,
    )
}

pub(super) fn witness_for_slice(
    role: SemanticSourceAirRoleV2,
    statement: &EpochTraceChunkV2,
    full_bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
    slice: EpochUniquenessSliceV2,
) -> Result<SemanticSourceWitnessV2, CheckpointError> {
    let end = slice.end()?;
    let bindings = full_bindings
        .get(slice.start()..end)
        .ok_or(CheckpointError::Canonical)?;
    let prepared = prepared
        .get(slice.start()..end)
        .ok_or(CheckpointError::Canonical)?;
    if bindings.len() != prepared.len() || bindings.is_empty() {
        return Err(CheckpointError::Invariant);
    }
    let public_values = public_values_for_slice(statement, full_bindings, slice)?;
    let mut rows = Vec::new();
    let mut range_queries = Vec::new();
    for (slot, (binding, transition)) in bindings.iter().zip(prepared).enumerate() {
        if transition.binding() != *binding {
            return Err(CheckpointError::Invariant);
        }
        let stream = transition_event_stream(&transition.material)?;
        let source = stream.source();
        if u64::try_from(source.len()).map_err(|_| CheckpointError::Limit)?
            != binding.inputs().event_bytes
            || u64::try_from(stream.records().len()).map_err(|_| CheckpointError::Limit)?
                != binding.inputs().event_count
            || source.len() < PREFIX_BYTES_V2
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        let mut byte_index = 0_usize;
        let mut state = SourceStateV2 {
            jmt_stage: 0,
            jmt_count: 0,
            uniqueness_counters: [0; UNIQUENESS_COUNTER_COUNT_V2],
            net_effect_counter: 0,
            net_mutation_counter: 0,
            fixed_event_counters: [0; FIXED_EVENT_COUNTER_COUNT_V2],
            declared_count_bytes: [0; DECLARED_COUNT_BYTE_COUNT_V2],
            replay_counters: [0; REPLAY_COUNTER_COUNT_V2],
            flow_root_low_byte: 0,
        };
        for (prefix_index, byte) in source[..PREFIX_BYTES_V2].iter().copied().enumerate() {
            rows.push(source_row(
                role,
                slot,
                byte_index,
                byte,
                SourcePhaseV2::Prefix(prefix_index),
                None,
                false,
                state,
                None,
                None,
            )?);
            byte_index = byte_index.checked_add(1).ok_or(CheckpointError::Overflow)?;
        }

        let mut previous_global_spent = false;
        for (event_index, record) in stream.records().iter().copied().enumerate() {
            if record.opcode() == RecursiveTraceOpcodeV2::UniquenessPrecommit {
                let precommit = decode_uniqueness_precommit(record.payload())?;
                state.declared_count_bytes[..4]
                    .copy_from_slice(&precommit.spent_count.to_le_bytes());
                state.declared_count_bytes[4..]
                    .copy_from_slice(&precommit.output_count.to_le_bytes());
            }
            let canonical = record.canonical_bytes();
            let event_len = u32::try_from(canonical.len()).map_err(|_| CheckpointError::Limit)?;
            let payload_len =
                u32::try_from(record.payload().len()).map_err(|_| CheckpointError::Limit)?;
            let uniqueness_class = if record.opcode() == RecursiveTraceOpcodeV2::UniquenessSorted {
                let (pass, set, list, _) = decode_uniqueness_sorted_row(record.payload())?;
                Some(uniqueness_class(pass, set, list))
            } else {
                None
            };
            let global_product_pair = uniqueness_class == Some(7) && previous_global_spent;
            let net_effect = if record.opcode() == RecursiveTraceOpcodeV2::NetMerge {
                Some(decode_net_effect(record.payload())?)
            } else {
                None
            };
            let net_kind = net_effect.map(|effect| effect.kind as usize);
            let meta = EventMetaV2 {
                event_index,
                event_len_bytes: event_len.to_le_bytes(),
                payload_len_bytes: payload_len.to_le_bytes(),
                ordinal_bytes: record.ordinal().to_le_bytes(),
                opcode: record.opcode(),
                uniqueness_class,
                net_kind,
                net_terminal_id: net_effect.map(|effect| effect.path_and_old.terminal_id),
                global_product_pair,
                payload_len: usize::try_from(payload_len).map_err(|_| CheckpointError::Limit)?,
            };
            let replay_rows = if role == SemanticSourceAirRoleV2::Uniqueness
                && matches!(
                    record.opcode(),
                    RecursiveTraceOpcodeV2::ReplayInput | RecursiveTraceOpcodeV2::ReplayOutput
                ) {
                let _ = decode_flow_item(record.payload())?;
                let replay_rows = (0..record.payload().len())
                    .map(|payload_index| replay_row_meta(record.payload(), payload_index))
                    .collect::<Result<Vec<_>, _>>()?;
                let global_slot = slice
                    .start()
                    .checked_add(slot)
                    .ok_or(CheckpointError::Overflow)?;
                range_queries.extend(
                    replay_rows
                        .iter()
                        .filter(|row| row.phase == REPLAY_TX_BYTES_PHASE_V2)
                        .map(|row| UniquenessRangeQueryV2 {
                            slot: global_slot,
                            byte_0: row.ascii_low,
                            byte_1: row.ascii_high,
                            single_byte: false,
                        }),
                );
                Some(replay_rows)
            } else {
                None
            };
            for (index, byte) in event_len.to_le_bytes().into_iter().enumerate() {
                rows.push(source_row(
                    role,
                    slot,
                    byte_index,
                    byte,
                    SourcePhaseV2::Length(index),
                    Some(meta),
                    false,
                    state,
                    None,
                    None,
                )?);
                byte_index = byte_index.checked_add(1).ok_or(CheckpointError::Overflow)?;
            }
            for (index, byte) in canonical.iter().copied().enumerate() {
                let phase = if index < HEADER_BYTES_V2 {
                    SourcePhaseV2::Header(index)
                } else {
                    SourcePhaseV2::Payload(index - HEADER_BYTES_V2)
                };
                let event_final = index + 1 == canonical.len();
                let slot_end = event_final && event_index + 1 == stream.records().len();
                let replay = match phase {
                    SourcePhaseV2::Payload(payload_index) => replay_rows
                        .as_ref()
                        .and_then(|rows| rows.get(payload_index))
                        .copied(),
                    _ => None,
                };
                let flow = match phase {
                    SourcePhaseV2::Payload(payload_index)
                        if role.is_transition()
                            && matches!(
                                record.opcode(),
                                RecursiveTraceOpcodeV2::BeginBlock
                                    | RecursiveTraceOpcodeV2::FinalizeBlock
                            ) =>
                    {
                        Some(flow_row_meta(record.payload(), payload_index)?)
                    }
                    _ => None,
                };
                rows.push(source_row(
                    role,
                    slot,
                    byte_index,
                    byte,
                    phase,
                    Some(meta),
                    slot_end,
                    state,
                    replay,
                    flow,
                )?);
                if let Some(flow) = flow {
                    if matches!(flow.phase, 9 | 12) && flow.hex_low {
                        let nibble = flow.hex_nibble.ok_or(CheckpointError::Invariant)?;
                        let decoded = flow
                            .hex_high_nibble
                            .checked_mul(16)
                            .and_then(|high| high.checked_add(nibble))
                            .ok_or(CheckpointError::Overflow)?;
                        if flow.root_byte_parity {
                            state.flow_root_low_byte = 0;
                        } else {
                            state.flow_root_low_byte = decoded;
                        }
                    }
                }
                byte_index = byte_index.checked_add(1).ok_or(CheckpointError::Overflow)?;
            }
            match record.opcode() {
                RecursiveTraceOpcodeV2::BeginBlock => {
                    state.fixed_event_counters[0] = state.fixed_event_counters[0]
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
                RecursiveTraceOpcodeV2::UniquenessPrecommit => {
                    state.fixed_event_counters[1] = state.fixed_event_counters[1]
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
                RecursiveTraceOpcodeV2::UniquenessChallenge => {
                    state.fixed_event_counters[2] = state.fixed_event_counters[2]
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
                RecursiveTraceOpcodeV2::JmtUpdate => state.jmt_stage = 1,
                RecursiveTraceOpcodeV2::PromoteChildRoot => state.jmt_stage = 2,
                RecursiveTraceOpcodeV2::JmtMicroOp => {
                    state.jmt_count = state
                        .jmt_count
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
                RecursiveTraceOpcodeV2::UniquenessSorted => {
                    let class = uniqueness_class.ok_or(CheckpointError::Invariant)?;
                    let counter = if class >= 6 { 6 } else { class };
                    state.uniqueness_counters[counter] = state.uniqueness_counters[counter]
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
                RecursiveTraceOpcodeV2::NetMerge => {
                    let kind = net_kind.ok_or(CheckpointError::Invariant)?;
                    if kind == NetEffectKindV2::Close as usize {
                        state.fixed_event_counters[3] = state.fixed_event_counters[3]
                            .checked_add(1)
                            .ok_or(CheckpointError::Overflow)?;
                    }
                    if kind != NetEffectKindV2::Close as usize {
                        state.net_effect_counter = state
                            .net_effect_counter
                            .checked_add(1)
                            .ok_or(CheckpointError::Overflow)?;
                    }
                    if (NetEffectKindV2::Delete as usize..=NetEffectKindV2::Replace as usize)
                        .contains(&kind)
                    {
                        state.net_mutation_counter = state
                            .net_mutation_counter
                            .checked_add(1)
                            .ok_or(CheckpointError::Overflow)?;
                    }
                }
                RecursiveTraceOpcodeV2::CommitTypedEvent => {
                    state.fixed_event_counters[4] = state.fixed_event_counters[4]
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
                RecursiveTraceOpcodeV2::FinalizeBlock => {
                    state.fixed_event_counters[5] = state.fixed_event_counters[5]
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
                RecursiveTraceOpcodeV2::ReplayInput
                    if role == SemanticSourceAirRoleV2::Uniqueness =>
                {
                    state.replay_counters[0] = state.replay_counters[0]
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
                RecursiveTraceOpcodeV2::ReplayOutput
                    if role == SemanticSourceAirRoleV2::Uniqueness =>
                {
                    state.replay_counters[1] = state.replay_counters[1]
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
                _ => {}
            }
            previous_global_spent = uniqueness_class == Some(6);
        }
        if byte_index != source.len() || state.jmt_stage != 2 || state.flow_root_low_byte != 0 {
            return Err(CheckpointError::Canonical);
        }
        if role == SemanticSourceAirRoleV2::Uniqueness {
            for count_bytes in [
                state.declared_count_bytes[..4]
                    .try_into()
                    .map_err(|_| CheckpointError::Invariant)?,
                state.declared_count_bytes[4..]
                    .try_into()
                    .map_err(|_| CheckpointError::Invariant)?,
            ] {
                let count = u32::from_le_bytes(count_bytes);
                let slack = declared_count_slack(count)?;
                range_queries.push(UniquenessRangeQueryV2 {
                    slot: slice
                        .start()
                        .checked_add(slot)
                        .ok_or(CheckpointError::Overflow)?,
                    byte_0: slack[0],
                    byte_1: slack[1],
                    single_byte: false,
                });
            }
        }
    }

    let padded_rows = semantic_trace_rows(rows.len())?.max(MIN_ROWS_V2);
    rows.resize_with(padded_rows, || SemanticSourceRowV2 {
        values: vec![KoalaBear::ZERO; ROW_FIELDS_V2],
    });
    Ok(SemanticSourceWitnessV2 {
        trace: SemanticSourceTraceV2 {
            role,
            public_values,
            rows,
        },
        range_queries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_slicing_halves_semantic_source_geometry() {
        assert_eq!(
            semantic_trace_rows(262_143).expect("full geometry"),
            262_144
        );
        assert_eq!(
            semantic_trace_rows(131_071).expect("lower geometry"),
            131_072
        );
    }

    fn flow_payload() -> Vec<u8> {
        let mut payload = Vec::with_capacity(FLOW_HEADER_BYTES_V2);
        let hex_digits = b"0123456789abcdef";
        for byte in [0x11_u8, 0x22, 0x33, 0x44] {
            payload.extend_from_slice(&64_u16.to_le_bytes());
            for _ in 0..32 {
                payload.push(hex_digits[usize::from(byte >> 4)]);
                payload.push(hex_digits[usize::from(byte & 0x0f)]);
            }
            if byte == 0x11 {
                payload.extend_from_slice(&0x0403_0201_u32.to_le_bytes());
                payload.extend_from_slice(&0x0c0b_0a09_0807_0605_u64.to_le_bytes());
            }
        }
        payload.extend_from_slice(&0x100f_0e0d_u32.to_le_bytes());
        payload.extend_from_slice(&0x1413_1211_u32.to_le_bytes());
        assert_eq!(payload.len(), FLOW_HEADER_BYTES_V2);
        payload
    }

    fn replay_payload(tx_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.push(2);
        payload.extend_from_slice(
            &u16::try_from(tx_id.len())
                .expect("bounded test tx id")
                .to_le_bytes(),
        );
        payload.extend_from_slice(tx_id);
        payload.extend_from_slice(&64_u16.to_le_bytes());
        payload.extend_from_slice("01".repeat(32).as_bytes());
        payload.extend_from_slice(&0x0403_0201_u32.to_le_bytes());
        payload.extend_from_slice(&64_u16.to_le_bytes());
        payload.extend_from_slice("ab".repeat(32).as_bytes());
        payload.extend_from_slice(&[0x5a; 32]);
        payload.push(1);
        payload.extend_from_slice(&[0, 0, 0]);
        payload
    }

    #[test]
    fn replay_parser_projects_exact_semantic_row() {
        let payload = replay_payload(b"tx-1");
        let rows = (0..payload.len())
            .map(|index| replay_row_meta(&payload, index).expect("canonical replay row"))
            .collect::<Vec<_>>();
        assert_eq!(payload.len(), 175 + 4);
        assert_eq!(rows[0].phase, REPLAY_OP_KIND_PHASE_V2);
        assert_eq!(
            rows.last().expect("final replay row").phase,
            REPLAY_FLAGS_PHASE_V2
        );
        assert_eq!(rows.last().expect("final replay row").remaining, 1);

        let definition_start = 3 + b"tx-1".len() + 2;
        let terminal_start = definition_start + 64 + 4 + 2;
        for (start, semantic_start) in [(definition_start, 0_usize), (terminal_start, 36)] {
            for byte_index in 0..32 {
                let high_nibble_row = &rows[start + byte_index * 2];
                let low_nibble_row = &rows[start + byte_index * 2 + 1];
                assert!(!high_nibble_row.hex_low);
                assert_eq!(high_nibble_row.semantic_index, 0);
                assert!(low_nibble_row.hex_low);
                assert_eq!(low_nibble_row.semantic_index, semantic_start + byte_index);
            }
        }

        let mut semantic = [0_u8; 100];
        let mut seen = [false; 100];
        for (index, row) in rows.iter().copied().enumerate() {
            let value = match row.phase {
                REPLAY_DEFINITION_HEX_PHASE_V2 | REPLAY_TERMINAL_HEX_PHASE_V2 if row.hex_low => {
                    row.hex_high_nibble * 16 + row.hex_nibble.expect("hex nibble")
                }
                REPLAY_SERIAL_PHASE_V2 | REPLAY_LEAF_HASH_PHASE_V2 => payload[index],
                _ => continue,
            };
            assert!(!seen[row.semantic_index]);
            semantic[row.semantic_index] = value;
            seen[row.semantic_index] = true;
        }
        assert!(seen.into_iter().all(core::convert::identity));
        assert_eq!(&semantic[..32], &[0x01; 32]);
        assert_eq!(&semantic[32..36], &0x0403_0201_u32.to_le_bytes());
        assert_eq!(&semantic[36..68], &[0xab; 32]);
        assert_eq!(&semantic[68..], &[0x5a; 32]);
    }

    #[test]
    fn replay_parser_rejects_noncanonical_text_and_shape() {
        let mut bad_ascii = replay_payload(b" ");
        assert!(matches!(
            replay_row_meta(&bad_ascii, 3),
            Err(CheckpointError::Canonical)
        ));

        let definition_start =
            3 + usize::from(u16::from_le_bytes([bad_ascii[1], bad_ascii[2]])) + 2;
        bad_ascii[definition_start] = b'A';
        assert!(matches!(
            replay_row_meta(&bad_ascii, definition_start),
            Err(CheckpointError::Canonical)
        ));

        let mut truncated = replay_payload(b"tx");
        truncated.pop();
        assert!(matches!(
            replay_row_meta(&truncated, 0),
            Err(CheckpointError::Canonical)
        ));
    }

    #[test]
    fn flow_parser_projects_exact_phases_and_root_limbs() {
        let payload = flow_payload();
        let rows = (0..payload.len())
            .map(|index| flow_row_meta(&payload, index).expect("canonical flow row"))
            .collect::<Vec<_>>();
        let phase_ends = [
            0_usize, 1, 65, 77, 78, 79, 143, 144, 145, 209, 210, 211, 275, 283,
        ];
        let mut phase_start = 0;
        for (phase, phase_end) in phase_ends.into_iter().enumerate() {
            assert!(rows[phase_start..=phase_end]
                .iter()
                .all(|row| row.phase == phase));
            phase_start = phase_end + 1;
        }

        for (root_start, expected_byte) in [(146_usize, 0x33_u8), (212, 0x44)] {
            for byte_index in 0..32 {
                let high_nibble_row = &rows[root_start + byte_index * 2];
                let low_nibble_row = &rows[root_start + byte_index * 2 + 1];
                assert!(!high_nibble_row.hex_low);
                assert_eq!(high_nibble_row.hex_byte_index, 0);
                assert!(low_nibble_row.hex_low);
                assert_eq!(low_nibble_row.hex_byte_index, byte_index);
                assert_eq!(low_nibble_row.root_limb_index, byte_index / 2);
                assert_eq!(low_nibble_row.root_byte_parity, byte_index % 2 == 1);
                assert_eq!(
                    low_nibble_row.hex_high_nibble * 16
                        + low_nibble_row.hex_nibble.expect("root hex nibble"),
                    expected_byte,
                );
            }
        }
    }

    #[test]
    fn flow_parser_rejects_noncanonical_text_and_shape() {
        let mut uppercase = flow_payload();
        uppercase[2] = b'A';
        assert!(matches!(
            flow_row_meta(&uppercase, 2),
            Err(CheckpointError::Canonical)
        ));

        let mut wrong_hex_length = flow_payload();
        wrong_hex_length[78] = 63;
        assert!(matches!(
            flow_row_meta(&wrong_hex_length, 0),
            Err(CheckpointError::Canonical)
        ));

        let mut truncated = flow_payload();
        truncated.pop();
        assert!(matches!(
            flow_row_meta(&truncated, 0),
            Err(CheckpointError::Canonical)
        ));
    }

    #[test]
    fn declared_count_slack_accepts_noop_zero_and_authority_bound() {
        assert_eq!(
            declared_count_slack(0).expect("noop zero count is canonical"),
            16_000_u16.to_le_bytes(),
        );
        assert_eq!(
            declared_count_slack(16_000).expect("authority cap is canonical"),
            [0, 0],
        );
        assert!(matches!(
            declared_count_slack(16_001),
            Err(CheckpointError::Limit)
        ));
    }
}
