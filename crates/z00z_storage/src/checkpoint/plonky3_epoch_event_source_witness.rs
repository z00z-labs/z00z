//! Witness construction for the packed canonical event-byte source.

use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;
use z00z_crypto::{CheckpointSha256BlockStreamV2, CheckpointShaRole};

use super::plonky3_epoch_event_source_columns::{
    EventSourceAirRoleV2, EventSourceRowV2, EventSourceTraceV2, BLOCK_PAIR_COUNT_V2,
    CALL_FIELDS_V2, FRAMED_PREFIX_PAIRS_V2, JOB_KIND_SELECTOR_OFFSET_V2, LENGTH_PAIRS_V2,
    PADDING_ZERO_BITS_V2, PUBLIC_FIELDS_V2, ROW_FIELDS_V2, STATIC_PREFIX_PAIRS_V2,
    TRANSITION_SLOTS_V2,
};
use super::plonky3_epoch_event_stream::transition_event_stream;
use super::plonky3_epoch_sha256_columns::{SemanticShaJobKindV2, ShaAirRoleV2};
use super::plonky3_epoch_sha256_witness::{expected_block_count, semantic_jobs, SemanticShaJobV2};
use super::plonky3_epoch_uniqueness_slice::EpochUniquenessSliceV2;
use super::{
    EpochAirTableV2, EpochPreparedTransitionV2, EpochTraceChunkV2, EpochTransitionBindingV2,
    EPOCH_CHUNK_BYTES_V2,
};
use crate::CheckpointError;

fn pair_values(bytes: [u8; 8]) -> [u16; LENGTH_PAIRS_V2] {
    core::array::from_fn(|pair| u16::from_be_bytes([bytes[pair * 2], bytes[pair * 2 + 1]]))
}

fn append_bits(values: &mut Vec<KoalaBear>, byte: u8) {
    for bit in 0..8 {
        values.push(KoalaBear::from_bool((byte >> bit) & 1 == 1));
    }
}

fn semantic_trace_rows(real_rows: usize) -> Result<usize, CheckpointError> {
    real_rows
        .checked_add(1)
        .and_then(usize::checked_next_power_of_two)
        .ok_or(CheckpointError::Overflow)
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
        || bindings.is_empty()
        || bindings.len() > TRANSITION_SLOTS_V2
    {
        return Err(CheckpointError::Canonical);
    }
    let full_event_bytes = full_bindings.iter().try_fold(0_u64, |total, binding| {
        total
            .checked_add(binding.inputs().event_bytes)
            .ok_or(CheckpointError::Overflow)
    })?;
    let inputs = statement.inputs();
    if inputs.table != EpochAirTableV2::PackedRange
        || inputs.replica != 0
        || inputs.row_start != 0
        || inputs.row_count != full_event_bytes
    {
        return Err(CheckpointError::Canonical);
    }

    let prefix = CheckpointSha256BlockStreamV2::framed_role_prefix(CheckpointShaRole::EventVector);
    if prefix.len() != STATIC_PREFIX_PAIRS_V2 * 2 {
        return Err(CheckpointError::Invariant);
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
                .copied()
                .map(expected_block_count)
                .transpose()?
                .unwrap_or(0),
        ));
    }
    for slot in 0..TRANSITION_SLOTS_V2 {
        values.push(KoalaBear::from_u64(
            bindings
                .get(slot)
                .map(|binding| binding.inputs().event_bytes)
                .unwrap_or(0),
        ));
    }
    values.extend(
        prefix
            .chunks_exact(2)
            .map(|pair| KoalaBear::from_u16(u16::from_be_bytes([pair[0], pair[1]]))),
    );
    for slot in 0..TRANSITION_SLOTS_V2 {
        let pairs = bindings
            .get(slot)
            .map(|binding| pair_values(binding.inputs().event_bytes.to_le_bytes()))
            .unwrap_or([0; LENGTH_PAIRS_V2]);
        values.extend(pairs.map(KoalaBear::from_u16));
    }
    let framed_prefix_bytes = u64::try_from(prefix.len()).map_err(|_| CheckpointError::Limit)?;
    for slot in 0..TRANSITION_SLOTS_V2 {
        let pairs = bindings
            .get(slot)
            .map(|binding| {
                framed_prefix_bytes
                    .checked_add(8)
                    .and_then(|bytes| bytes.checked_add(binding.inputs().event_bytes))
                    .and_then(|bytes| bytes.checked_mul(8))
                    .map(u64::to_be_bytes)
                    .map(pair_values)
                    .ok_or(CheckpointError::Overflow)
            })
            .transpose()?
            .unwrap_or([0; LENGTH_PAIRS_V2]);
        values.extend(pairs.map(KoalaBear::from_u16));
    }
    values.push(KoalaBear::from_usize(slice.start()));
    values.push(KoalaBear::from_usize(slice.len()));
    if values.len() != PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

pub(super) fn trace(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
) -> Result<EventSourceTraceV2, CheckpointError> {
    trace_with_role(EventSourceAirRoleV2::Hash, statement, bindings, prepared)
}

pub(super) fn semantic_trace(
    role: EventSourceAirRoleV2,
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
) -> Result<EventSourceTraceV2, CheckpointError> {
    semantic_trace_with_jobs(
        role,
        statement,
        bindings,
        bindings,
        prepared,
        EpochUniquenessSliceV2::full(bindings.len())?,
    )
}

pub(super) fn semantic_trace_for_slice(
    role: EventSourceAirRoleV2,
    statement: &EpochTraceChunkV2,
    full_bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
    slice: EpochUniquenessSliceV2,
) -> Result<EventSourceTraceV2, CheckpointError> {
    let end = slice.end()?;
    let bindings = full_bindings
        .get(slice.start()..end)
        .ok_or(CheckpointError::Canonical)?;
    let prepared = prepared
        .get(slice.start()..end)
        .ok_or(CheckpointError::Canonical)?;
    semantic_trace_with_jobs(role, statement, full_bindings, bindings, prepared, slice)
}

fn trace_with_role(
    role: EventSourceAirRoleV2,
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
) -> Result<EventSourceTraceV2, CheckpointError> {
    if role != EventSourceAirRoleV2::Hash || bindings.len() != prepared.len() || bindings.is_empty()
    {
        return Err(CheckpointError::Invariant);
    }
    let public_values = public_values(statement, bindings)?;
    let prefix = CheckpointSha256BlockStreamV2::framed_role_prefix(CheckpointShaRole::EventVector);
    let raw_start = u64::try_from(prefix.len())
        .map_err(|_| CheckpointError::Limit)?
        .checked_add(8)
        .ok_or(CheckpointError::Overflow)?;
    if raw_start != u64::try_from(FRAMED_PREFIX_PAIRS_V2 * 2).map_err(|_| CheckpointError::Limit)? {
        return Err(CheckpointError::Invariant);
    }

    let mut rows = Vec::new();
    for (slot, (binding, transition)) in bindings.iter().zip(prepared).enumerate() {
        if transition.binding() != *binding {
            return Err(CheckpointError::Invariant);
        }
        let stream = transition_event_stream(&transition.material)?;
        let raw_end = raw_start
            .checked_add(u64::try_from(stream.source().len()).map_err(|_| CheckpointError::Limit)?)
            .ok_or(CheckpointError::Overflow)?;
        let mut running = 0_u64;
        let mut blocks = 0_u64;
        let digest = stream.visit_digest_blocks(&mut |block| {
            if block.index() != blocks {
                return Err(CheckpointError::Canonical);
            }
            for pair in 0..BLOCK_PAIR_COUNT_V2 {
                let pair_byte_offset = block
                    .byte_offset()
                    .checked_add(u64::try_from(pair * 2).map_err(|_| CheckpointError::Limit)?)
                    .ok_or(CheckpointError::Overflow)?;
                let byte_0 = block.block()[pair * 2];
                let byte_1 = block.block()[pair * 2 + 1];
                let raw_0 = (raw_start..raw_end).contains(&pair_byte_offset);
                let raw_1 = (raw_start..raw_end).contains(
                    &pair_byte_offset
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?,
                );
                running = running
                    .checked_add(u64::from(raw_0) + u64::from(raw_1))
                    .ok_or(CheckpointError::Overflow)?;
                let raw_final =
                    raw_0 && pair_byte_offset + u64::from(raw_0) + u64::from(raw_1) == raw_end;
                let prefix_pair = (pair_byte_offset < raw_start)
                    .then(|| usize::try_from(pair_byte_offset / 2))
                    .transpose()
                    .map_err(|_| CheckpointError::Limit)?;

                let mut values = if rows.is_empty() {
                    public_values.clone()
                } else {
                    vec![KoalaBear::ZERO; PUBLIC_FIELDS_V2]
                };
                values.push(KoalaBear::ONE);
                for candidate in 0..TRANSITION_SLOTS_V2 {
                    values.push(KoalaBear::from_bool(candidate == slot));
                }
                for candidate in 0..BLOCK_PAIR_COUNT_V2 {
                    values.push(KoalaBear::from_bool(candidate == pair));
                }
                values.push(KoalaBear::from_u64(block.index()));
                values.push(KoalaBear::from_bool(block.final_block()));
                for candidate in 0..FRAMED_PREFIX_PAIRS_V2 {
                    values.push(KoalaBear::from_bool(prefix_pair == Some(candidate)));
                }
                values.push(KoalaBear::from_bool(raw_0));
                values.push(KoalaBear::from_bool(raw_1));
                values.push(KoalaBear::from_bool(raw_final));
                values.push(KoalaBear::from_bool(pair_byte_offset == raw_end));
                values.push(KoalaBear::from_bool(
                    pair_byte_offset
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?
                        == raw_end,
                ));
                values.push(KoalaBear::from_u8(byte_0));
                values.push(KoalaBear::from_u8(byte_1));
                append_bits(&mut values, byte_0);
                append_bits(&mut values, byte_1);
                values.push(KoalaBear::from_u64(running));
                values.extend(core::iter::repeat_n(
                    KoalaBear::ZERO,
                    super::plonky3_epoch_event_source_columns::ROW_FIELDS_V2
                        - super::plonky3_epoch_event_source_columns::JOB_KIND_SELECTOR_OFFSET_V2,
                ));
                if values.len() != CALL_FIELDS_V2 {
                    return Err(CheckpointError::Invariant);
                }
                rows.push(EventSourceRowV2 { values });
            }
            blocks = blocks.checked_add(1).ok_or(CheckpointError::Overflow)?;
            Ok(())
        })?;
        if blocks != expected_block_count(*binding)?
            || running != binding.inputs().event_bytes
            || digest != binding.inputs().event_vector_digest
        {
            return Err(CheckpointError::Canonical);
        }
    }

    let padded_rows = semantic_trace_rows(rows.len())?;
    rows.resize_with(padded_rows, || EventSourceRowV2 {
        values: vec![KoalaBear::ZERO; CALL_FIELDS_V2],
    });
    Ok(EventSourceTraceV2 {
        role,
        public_values,
        rows,
    })
}

#[cfg(test)]
mod geometry_tests {
    use super::*;

    #[test]
    fn slot_slicing_halves_event_source_semantic_geometry() {
        assert_eq!(
            semantic_trace_rows(131_071).expect("full geometry"),
            131_072
        );
        assert_eq!(semantic_trace_rows(65_535).expect("lower geometry"), 65_536);
    }
}

fn semantic_trace_with_jobs(
    role: EventSourceAirRoleV2,
    statement: &EpochTraceChunkV2,
    full_bindings: &[EpochTransitionBindingV2],
    bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
    slice: EpochUniquenessSliceV2,
) -> Result<EventSourceTraceV2, CheckpointError> {
    let sha_role = match role {
        EventSourceAirRoleV2::SemanticTransition => ShaAirRoleV2::SemanticTransitionChain,
        EventSourceAirRoleV2::SemanticUniqueness => ShaAirRoleV2::SemanticUniquenessChain,
        EventSourceAirRoleV2::Hash => return Err(CheckpointError::Invariant),
    };
    if bindings.len() != prepared.len() || bindings.is_empty() {
        return Err(CheckpointError::Invariant);
    }
    let public_values = public_values_for_slice(statement, full_bindings, slice)?;
    let mut rows = Vec::new();
    for (lane, (binding, transition)) in bindings.iter().zip(prepared).enumerate() {
        for job in semantic_jobs(sha_role, *binding, transition)? {
            append_semantic_job_rows(&mut rows, &public_values, lane, &job)?;
        }
    }
    let padded_rows = semantic_trace_rows(rows.len())?;
    rows.resize_with(padded_rows, || EventSourceRowV2 {
        values: vec![KoalaBear::ZERO; CALL_FIELDS_V2],
    });
    Ok(EventSourceTraceV2 {
        role,
        public_values,
        rows,
    })
}

fn append_semantic_job_rows(
    rows: &mut Vec<EventSourceRowV2>,
    public_values: &[KoalaBear],
    lane: usize,
    job: &SemanticShaJobV2,
) -> Result<(), CheckpointError> {
    let framed = job.framed_message()?;
    let event_vector = job.kind == SemanticShaJobKindV2::EventVector;
    let prefix_bytes = if event_vector {
        let prefix = CheckpointSha256BlockStreamV2::framed_role_prefix(job.role);
        prefix
            .len()
            .checked_add(8)
            .ok_or(CheckpointError::Overflow)?
    } else {
        0
    };
    if event_vector && prefix_bytes != FRAMED_PREFIX_PAIRS_V2 * 2 {
        return Err(CheckpointError::Invariant);
    }
    let raw_len = framed
        .len()
        .checked_sub(prefix_bytes)
        .ok_or(CheckpointError::Invariant)?;
    let raw_start = u64::try_from(prefix_bytes).map_err(|_| CheckpointError::Limit)?;
    let raw_end = u64::try_from(framed.len()).map_err(|_| CheckpointError::Limit)?;
    let message_len = raw_end;
    let block_count = CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(message_len)
        .map_err(|_| CheckpointError::Limit)?;
    let padding_zeros = block_count
        .checked_mul(64)
        .and_then(|bytes| bytes.checked_sub(message_len))
        .and_then(|bytes| bytes.checked_sub(9))
        .ok_or(CheckpointError::Invariant)?;
    if padding_zeros >= 64 {
        return Err(CheckpointError::Invariant);
    }

    let mut running = 0_u64;
    let mut observed_blocks = 0_u64;
    let digest = job.visit_blocks(&mut |block| {
        if block.index() != observed_blocks {
            return Err(CheckpointError::Canonical);
        }
        for pair in 0..BLOCK_PAIR_COUNT_V2 {
            let pair_offset = block
                .byte_offset()
                .checked_add(u64::try_from(pair * 2).map_err(|_| CheckpointError::Limit)?)
                .ok_or(CheckpointError::Overflow)?;
            let next_offset = pair_offset
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
            let raw_0 = (raw_start..raw_end).contains(&pair_offset);
            let raw_1 = (raw_start..raw_end).contains(&next_offset);
            running = running
                .checked_add(u64::from(raw_0) + u64::from(raw_1))
                .ok_or(CheckpointError::Overflow)?;
            let raw_final = raw_0
                && pair_offset
                    .checked_add(u64::from(raw_0) + u64::from(raw_1))
                    .ok_or(CheckpointError::Overflow)?
                    == raw_end;
            let prefix_pair = event_vector
                .then_some(pair_offset)
                .filter(|offset| *offset < raw_start)
                .map(|offset| usize::try_from(offset / 2))
                .transpose()
                .map_err(|_| CheckpointError::Limit)?;
            let byte_0 = block.block()[pair * 2];
            let byte_1 = block.block()[pair * 2 + 1];
            let mut values = if rows.is_empty() {
                public_values.to_vec()
            } else {
                vec![KoalaBear::ZERO; PUBLIC_FIELDS_V2]
            };
            values.push(KoalaBear::ONE);
            values.extend(
                (0..TRANSITION_SLOTS_V2).map(|candidate| KoalaBear::from_bool(candidate == lane)),
            );
            values.extend(
                (0..BLOCK_PAIR_COUNT_V2).map(|candidate| KoalaBear::from_bool(candidate == pair)),
            );
            values.push(KoalaBear::from_u64(block.index()));
            values.push(KoalaBear::from_bool(block.final_block()));
            values.extend(
                (0..FRAMED_PREFIX_PAIRS_V2)
                    .map(|candidate| KoalaBear::from_bool(prefix_pair == Some(candidate))),
            );
            values.push(KoalaBear::from_bool(raw_0));
            values.push(KoalaBear::from_bool(raw_1));
            values.push(KoalaBear::from_bool(raw_final));
            values.push(KoalaBear::from_bool(pair_offset == raw_end));
            values.push(KoalaBear::from_bool(next_offset == raw_end));
            values.push(KoalaBear::from_u8(byte_0));
            values.push(KoalaBear::from_u8(byte_1));
            append_bits(&mut values, byte_0);
            append_bits(&mut values, byte_1);
            values.push(KoalaBear::from_u64(running));
            debug_assert_eq!(values.len(), PUBLIC_FIELDS_V2 + JOB_KIND_SELECTOR_OFFSET_V2);
            values.extend(
                SemanticShaJobKindV2::ALL
                    .into_iter()
                    .map(|kind| KoalaBear::from_bool(kind == job.kind)),
            );
            values.push(KoalaBear::from_u64(job.id));
            values.push(KoalaBear::from_usize(raw_len));
            values.push(KoalaBear::from_u64(block_count));
            values.extend(
                pair_values(
                    message_len
                        .checked_mul(8)
                        .ok_or(CheckpointError::Overflow)?
                        .to_be_bytes(),
                )
                .map(KoalaBear::from_u16),
            );
            values.push(KoalaBear::from_bool(block.index() == 0 && pair == 0));
            values.extend(
                (0..PADDING_ZERO_BITS_V2)
                    .map(|bit| KoalaBear::from_bool((padding_zeros >> bit) & 1 == 1)),
            );
            if values.len() != CALL_FIELDS_V2 || values.len() != PUBLIC_FIELDS_V2 + ROW_FIELDS_V2 {
                return Err(CheckpointError::Invariant);
            }
            rows.push(EventSourceRowV2 { values });
        }
        observed_blocks = observed_blocks
            .checked_add(1)
            .ok_or(CheckpointError::Overflow)?;
        Ok(())
    })?;
    if digest != job.expected_digest
        || observed_blocks != block_count
        || running != u64::try_from(raw_len).map_err(|_| CheckpointError::Limit)?
    {
        return Err(CheckpointError::RecursiveRejected(
            super::RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}
