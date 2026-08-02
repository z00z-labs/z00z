//! Canonical JMT witness construction and local Batch-STARK invocation.

use p3_circuit::tables::{
    AluTrace, ConstTrace, NonPrimitiveTrace, PublicTrace, Traces, WitnessTrace,
};
use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;
use z00z_crypto::SHA256_IV_V2;
use z00z_plonky3_circuit_prover::batch_stark_prover::TablePacking;
use z00z_plonky3_circuit_prover::{BatchStarkProof, BatchStarkProver};

#[cfg(test)]
use super::plonky3_epoch_jmt_air::JmtAirV2;
use super::plonky3_epoch_jmt_air::{
    jmt_npo_type, JmtChunkRowV2, JmtChunkTraceV2, JmtRowV2, JmtTraceV2, BIT_POSITION_COUNT_V2,
    BYTE_POSITION_COUNT_V2, CALL_FIELDS_V2, CASE_COUNT_V2, CHUNK_LANES_V2, CHUNK_LANE_FIELDS_V2,
    CHUNK_LANE_KIND_OFFSET_V2, CHUNK_LANE_OFFSET_V2, CHUNK_LANE_POST_ROOT_OFFSET_V2,
    CHUNK_LANE_PRE_ROOT_OFFSET_V2, CHUNK_LANE_RECORD_COUNT_OFFSET_V2,
    CHUNK_LANE_TRACE_DIGEST_OFFSET_V2, CHUNK_LANE_UPDATE_COUNT_OFFSET_V2, CHUNK_PUBLIC_FIELDS_V2,
    DIGEST_LIMBS_V2, JMT_MIN_ROWS_V2, JMT_RECORD_BYTES_V2, OPCODE_COUNT_V2, PUBLIC_FIELDS_V2,
    PUBLIC_ROW_COUNT_OFFSET_V2, RECORD_LENGTHS_V2, ROLE_COUNT_V2, ROW_FIELDS_V2, RUNNING_OFFSET_V2,
    SELECTED_BITS_COUNT_V2, SIBLING_TYPE_COUNT_V2, STATEMENT_LIMBS_V2,
};
use super::plonky3_epoch_jmt_table::JmtProverV2;
use super::plonky3_epoch_sha256_witness::{compress as sha_compress, words_bytes};
use super::{
    hardened_koala_bear_config, plonky3_epoch_event_stream, EpochAirTableV2,
    EpochPreparedTransitionV2, EpochTraceChunkV2, EpochTransitionBindingV2, Plonky3StarkConfigV2,
    RecursiveCheckpointRejectReasonV2, EPOCH_CHUNK_BYTES_V2,
};
use crate::settlement::{
    noop_update_trace_digest, RootGeneration, SettlementUpdateTraceCircuitDecoderV2,
    JMT_CIRCUIT_HEADER_BYTES_V2, JMT_SPARSE_PLACEHOLDER_HASH_V2, JMT_TRACE_MUTATING_KIND_V2,
    JMT_TRACE_NOOP_KIND_V2, JMT_UPDATE_TRACE_VERSION_V2,
};
use crate::CheckpointError;

struct JmtWitnessStateV2 {
    update_index: u32,
    operation_index: u32,
    value_present: bool,
    prior_present: bool,
    leaf_present: bool,
    expected_siblings: u16,
    consumed_siblings: u16,
    expected_split: u16,
    consumed_split: u16,
    expected_operations: u32,
    completed_operations: u32,
    coalesced: bool,
    new_parent_started: bool,
    operation_job: usize,
    expected_value_bytes: u32,
    expected_prior_value_bytes: u32,
    prior_value_block_count: u32,
    new_value_block_count: u32,
    value_padding_started: bool,
    value_kind: Option<u8>,
    role: u8,
    tree_definition: [u8; 32],
    tree_serial: [u8; 4],
    mutation_case: u8,
    key: [u8; 32],
    path_key: [u8; 32],
    update_current: [u8; 32],
    update_new_root: [u8; 32],
    old_current: [u8; 32],
    new_current: [u8; 32],
}

impl Default for JmtWitnessStateV2 {
    fn default() -> Self {
        Self {
            update_index: 0,
            operation_index: 0,
            value_present: false,
            prior_present: false,
            leaf_present: false,
            expected_siblings: 0,
            consumed_siblings: 0,
            expected_split: 0,
            consumed_split: 0,
            expected_operations: 0,
            completed_operations: 0,
            coalesced: false,
            new_parent_started: false,
            operation_job: 0,
            expected_value_bytes: 0,
            expected_prior_value_bytes: 0,
            prior_value_block_count: 0,
            new_value_block_count: 0,
            value_padding_started: false,
            value_kind: None,
            role: 0,
            tree_definition: [0; 32],
            tree_serial: [0; 4],
            mutation_case: 0,
            key: [0; 32],
            path_key: [0; 32],
            update_current: [0; 32],
            update_new_root: [0; 32],
            old_current: [0; 32],
            new_current: [0; 32],
        }
    }
}

fn take_array<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N], CheckpointError> {
    bytes
        .get(offset..offset + N)
        .ok_or(CheckpointError::Canonical)?
        .try_into()
        .map_err(|_| CheckpointError::Canonical)
}

fn take_u16(bytes: &[u8], offset: usize) -> Result<u16, CheckpointError> {
    Ok(u16::from_le_bytes(take_array(bytes, offset)?))
}

fn take_u32(bytes: &[u8], offset: usize) -> Result<u32, CheckpointError> {
    Ok(u32::from_le_bytes(take_array(bytes, offset)?))
}

fn digest_limbs(digest: [u8; 32]) -> [u16; 16] {
    core::array::from_fn(|limb| {
        let word = limb / 2;
        let within_word = if limb.is_multiple_of(2) { 2 } else { 0 };
        let byte = word * 4 + within_word;
        u16::from_be_bytes([digest[byte], digest[byte + 1]])
    })
}

fn raw_sha_digest(blocks: &[u8]) -> Result<[u8; 32], CheckpointError> {
    if blocks.is_empty() || blocks.len() % 64 != 0 {
        return Err(CheckpointError::Canonical);
    }
    let mut state = SHA256_IV_V2;
    for chunk in blocks.chunks_exact(64) {
        let block: [u8; 64] = chunk.try_into().map_err(|_| CheckpointError::Canonical)?;
        state = sha_compress(state, &block);
    }
    Ok(words_bytes(state))
}

fn append_digest(values: &mut Vec<KoalaBear>, digest: [u8; 32]) {
    values.extend(digest_limbs(digest).map(KoalaBear::from_u16));
}

fn append_u64_limbs(values: &mut Vec<KoalaBear>, value: u64) {
    values.extend(
        [
            value as u16,
            (value >> 16) as u16,
            (value >> 32) as u16,
            (value >> 48) as u16,
        ]
        .map(KoalaBear::from_u16),
    );
}

fn append_u32_bytes(values: &mut Vec<KoalaBear>, value: u32) {
    values.extend(value.to_le_bytes().map(KoalaBear::from_u8));
}

pub(super) fn public_values(
    statement: &EpochTraceChunkV2,
    header: &[u8],
) -> Result<Vec<KoalaBear>, CheckpointError> {
    if statement.canonical_bytes().len() != EPOCH_CHUNK_BYTES_V2
        || header.len() != JMT_CIRCUIT_HEADER_BYTES_V2
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
    values.extend(header.iter().copied().map(KoalaBear::from_u8));
    append_digest(&mut values, statement.inputs().input_state_root);
    append_digest(&mut values, statement.inputs().output_state_root);
    if values.len() != PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

pub(super) fn chunk_public_values(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
) -> Result<Vec<KoalaBear>, CheckpointError> {
    let inputs = statement.inputs();
    let record_count = bindings.iter().try_fold(0_u64, |total, binding| {
        total
            .checked_add(binding.inputs().jmt_record_count)
            .ok_or(CheckpointError::Overflow)
    })?;
    if statement.canonical_bytes().len() != EPOCH_CHUNK_BYTES_V2
        || inputs.table != EpochAirTableV2::JmtUpdate
        || inputs.replica != 0
        || bindings.is_empty()
        || bindings.len() > CHUNK_LANES_V2
        || inputs.row_count != record_count
    {
        return Err(CheckpointError::Canonical);
    }
    let mut values = Vec::with_capacity(CHUNK_PUBLIC_FIELDS_V2);
    values.extend(
        statement
            .canonical_bytes()
            .chunks_exact(2)
            .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
    );
    for lane in 0..CHUNK_LANES_V2 {
        values.push(KoalaBear::from_bool(lane < bindings.len()));
    }
    for lane in 0..CHUNK_LANES_V2 {
        if let Some(binding) = bindings.get(lane) {
            let binding = binding.inputs();
            let update_count =
                u32::try_from(binding.jmt_update_count).map_err(|_| CheckpointError::Limit)?;
            append_digest(&mut values, binding.pre_definition_root);
            append_digest(&mut values, binding.post_definition_root);
            values.extend(
                binding
                    .post_definition_root
                    .into_iter()
                    .map(KoalaBear::from_u8),
            );
            values.extend(
                binding
                    .update_trace_digest
                    .into_iter()
                    .map(KoalaBear::from_u8),
            );
            append_u64_limbs(&mut values, binding.jmt_record_count);
            append_u32_bytes(&mut values, update_count);
            values.push(KoalaBear::from_u8(if binding.jmt_record_count == 0 {
                JMT_TRACE_NOOP_KIND_V2
            } else {
                JMT_TRACE_MUTATING_KIND_V2
            }));
        } else {
            append_digest(&mut values, [0; 32]);
            append_digest(&mut values, [0; 32]);
            values.extend(core::iter::repeat_n(KoalaBear::ZERO, 32));
            values.extend(
                noop_update_trace_digest()
                    .into_iter()
                    .map(KoalaBear::from_u8),
            );
            append_u64_limbs(&mut values, 0);
            append_u32_bytes(&mut values, 0);
            values.push(KoalaBear::from_u8(JMT_TRACE_NOOP_KIND_V2));
        }
    }
    if values.len() != CHUNK_PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

fn chunk_lane_public_values(
    chunk_public: &[KoalaBear],
    lane: usize,
) -> Result<Vec<KoalaBear>, CheckpointError> {
    if chunk_public.len() != CHUNK_PUBLIC_FIELDS_V2 || lane >= CHUNK_LANES_V2 {
        return Err(CheckpointError::Invariant);
    }
    let lane_offset = CHUNK_LANE_OFFSET_V2 + lane * CHUNK_LANE_FIELDS_V2;
    let mut values = chunk_public[..STATEMENT_LIMBS_V2].to_vec();
    values[PUBLIC_ROW_COUNT_OFFSET_V2..PUBLIC_ROW_COUNT_OFFSET_V2 + 4].copy_from_slice(
        &chunk_public[lane_offset + CHUNK_LANE_RECORD_COUNT_OFFSET_V2
            ..lane_offset + CHUNK_LANE_RECORD_COUNT_OFFSET_V2 + 4],
    );
    values.push(KoalaBear::from_u8(JMT_UPDATE_TRACE_VERSION_V2));
    values.push(KoalaBear::from_u8(RootGeneration::SettlementV2.version()));
    values.push(chunk_public[lane_offset + CHUNK_LANE_KIND_OFFSET_V2]);
    values.extend_from_slice(
        &chunk_public[lane_offset + CHUNK_LANE_TRACE_DIGEST_OFFSET_V2
            ..lane_offset + CHUNK_LANE_TRACE_DIGEST_OFFSET_V2 + 32],
    );
    values.extend_from_slice(
        &chunk_public[lane_offset + CHUNK_LANE_UPDATE_COUNT_OFFSET_V2
            ..lane_offset + CHUNK_LANE_UPDATE_COUNT_OFFSET_V2 + 4],
    );
    values.extend_from_slice(
        &chunk_public[lane_offset + CHUNK_LANE_PRE_ROOT_OFFSET_V2
            ..lane_offset + CHUNK_LANE_PRE_ROOT_OFFSET_V2 + DIGEST_LIMBS_V2],
    );
    values.extend_from_slice(
        &chunk_public[lane_offset + CHUNK_LANE_POST_ROOT_OFFSET_V2
            ..lane_offset + CHUNK_LANE_POST_ROOT_OFFSET_V2 + DIGEST_LIMBS_V2],
    );
    if values.len() != PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

fn canonical_noop_header() -> [u8; JMT_CIRCUIT_HEADER_BYTES_V2] {
    let mut header = [0_u8; JMT_CIRCUIT_HEADER_BYTES_V2];
    header[0] = JMT_UPDATE_TRACE_VERSION_V2;
    header[1] = RootGeneration::SettlementV2.version();
    header[2] = JMT_TRACE_NOOP_KIND_V2;
    header[3..35].copy_from_slice(&noop_update_trace_digest());
    header
}

pub(super) fn rows(
    statement: &EpochTraceChunkV2,
    header: &[u8],
    records: &[Vec<u8>],
) -> Result<Vec<JmtRowV2>, CheckpointError> {
    if header.len() != JMT_CIRCUIT_HEADER_BYTES_V2
        || statement.inputs().table != EpochAirTableV2::JmtUpdate
        || statement.inputs().replica != 0
        || statement.inputs().row_count
            != u64::try_from(records.len()).map_err(|_| CheckpointError::Limit)?
        || (records.is_empty() && header[2] != JMT_TRACE_NOOP_KIND_V2)
        || (!records.is_empty() && header[2] != JMT_TRACE_MUTATING_KIND_V2)
    {
        return Err(CheckpointError::Canonical);
    }
    rows_with_public(public_values(statement, header)?, header, records)
}

pub(super) fn rows_with_public<R: AsRef<[u8]>>(
    public: Vec<KoalaBear>,
    header: &[u8],
    records: &[R],
) -> Result<Vec<JmtRowV2>, CheckpointError> {
    if public.len() != PUBLIC_FIELDS_V2
        || header.len() != JMT_CIRCUIT_HEADER_BYTES_V2
        || (records.is_empty() && header[2] != JMT_TRACE_NOOP_KIND_V2)
        || (!records.is_empty() && header[2] != JMT_TRACE_MUTATING_KIND_V2)
    {
        return Err(CheckpointError::Canonical);
    }
    let mut decoder = SettlementUpdateTraceCircuitDecoderV2::new(header)
        .map_err(|error| CheckpointError::Backend(format!("JMT header rejected: {error}")))?;
    for record in records {
        decoder
            .accept(record.as_ref())
            .map_err(|error| CheckpointError::Backend(format!("JMT record rejected: {error}")))?;
    }
    decoder
        .finish()
        .map_err(|error| CheckpointError::Backend(format!("JMT transcript rejected: {error}")))?;

    let padded_rows = records.len().max(JMT_MIN_ROWS_V2).next_power_of_two();
    let mut result = Vec::with_capacity(padded_rows);
    let mut state = JmtWitnessStateV2::default();
    for (record_index, record) in records.iter().enumerate() {
        let record = record.as_ref();
        if record.len() < 2 || record.len() > JMT_RECORD_BYTES_V2 {
            return Err(CheckpointError::Limit);
        }
        let opcode = usize::from(record[1]);
        if !(1..=OPCODE_COUNT_V2).contains(&opcode) || record.len() != RECORD_LENGTHS_V2[opcode - 1]
        {
            return Err(CheckpointError::Canonical);
        }
        let mut sibling_digest = [0_u8; 32];
        let mut old_parent_digest = [0_u8; 32];
        let mut new_parent_digest = [0_u8; 32];
        let mut old_leaf_digest = [0_u8; 32];
        let mut new_leaf_digest = [0_u8; 32];
        let mut aux = 0_u16;
        let mut bit_index = None;
        let mut value_padding_start = false;
        let mut value_remainder = None;

        match opcode {
            1 => {
                state.update_index = take_u32(record, 2)?;
                state.role = record[6];
                state.tree_definition = take_array(record, 7)?;
                state.tree_serial = take_array(record, 39)?;
                state.update_current = take_array(record, 91)?;
                state.update_new_root = take_array(record, 123)?;
                state.expected_operations = take_u32(record, 155)?;
                state.completed_operations = 0;
                state.operation_index = 0;
                state.value_present = false;
                state.prior_present = false;
                state.leaf_present = false;
                state.expected_siblings = 0;
                state.consumed_siblings = 0;
                state.expected_split = 0;
                state.consumed_split = 0;
                state.coalesced = false;
                state.new_parent_started = false;
                state.operation_job = 0;
                state.expected_value_bytes = 0;
                state.expected_prior_value_bytes = 0;
                state.prior_value_block_count = 0;
                state.new_value_block_count = 0;
                state.value_padding_started = false;
                state.value_kind = None;
                state.mutation_case = 0;
                state.key = [0; 32];
                state.path_key = [0; 32];
                state.old_current = [0; 32];
                state.new_current = [0; 32];
            }
            2 => {
                state.update_index = take_u32(record, 2)?;
                state.operation_index = take_u32(record, 6)?;
                state.operation_job = record_index
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                state.key = take_array(record, 10)?;
                state.value_present = record[42] == 1;
                state.expected_value_bytes = take_u32(record, 43)?;
                state.prior_present = record[47] == 1;
                state.expected_prior_value_bytes = take_u32(record, 48)?;
                state.prior_value_block_count = 0;
                state.new_value_block_count = 0;
                state.value_padding_started = false;
                state.value_kind = None;
                state.leaf_present = false;
                state.expected_siblings = 0;
                state.consumed_siblings = 0;
                state.expected_split = 0;
                state.consumed_split = 0;
                state.coalesced = false;
                state.new_parent_started = false;
                state.mutation_case = 0;
                state.path_key = [0; 32];
                state.old_current = [0; 32];
                state.new_current = [0; 32];
            }
            3 => {
                state.update_index = take_u32(record, 2)?;
                state.operation_index = take_u32(record, 6)?;
                let block_index = take_u32(record, 10)?;
                let block_count = take_u32(record, 14)?;
                let value_kind = record[18];
                aux = u16::try_from(block_index).map_err(|_| CheckpointError::Limit)?;
                if state.value_kind != Some(value_kind) {
                    state.value_padding_started = false;
                }
                let expected_bytes = match value_kind {
                    0 => {
                        state.new_value_block_count = block_count;
                        state.expected_value_bytes
                    }
                    1 => {
                        state.prior_value_block_count = block_count;
                        state.expected_prior_value_bytes
                    }
                    _ => return Err(CheckpointError::Canonical),
                };
                let remainder = expected_bytes % 64;
                value_padding_start = block_index
                    .checked_mul(64)
                    .is_some_and(|start| start == expected_bytes - remainder);
                state.value_padding_started |= value_padding_start;
                state.value_kind = Some(value_kind);
                value_remainder =
                    Some(usize::try_from(remainder).map_err(|_| CheckpointError::Limit)?);
            }
            4 => {
                state.value_padding_started = false;
                state.value_kind = None;
                state.update_index = take_u32(record, 2)?;
                state.operation_index = take_u32(record, 6)?;
                state.leaf_present = record[10] == 1;
                state.expected_siblings = take_u16(record, 11)?;
                state.mutation_case = record[13];
                state.expected_split = take_u16(record, 14)?;
                state.consumed_siblings = 0;
                state.consumed_split = 0;
                state.coalesced = false;
                state.new_parent_started = matches!(state.mutation_case, 1..=3);
                if state.leaf_present {
                    old_leaf_digest = raw_sha_digest(&record[19..147])?;
                    state.path_key = take_array(record, 19 + 13)?;
                    state.old_current = old_leaf_digest;
                } else {
                    old_leaf_digest = JMT_SPARSE_PLACEHOLDER_HASH_V2;
                    state.path_key = state.key;
                    state.old_current = JMT_SPARSE_PLACEHOLDER_HASH_V2;
                }
                if state.value_present {
                    new_leaf_digest = raw_sha_digest(&record[147..275])?;
                    state.new_current = new_leaf_digest;
                } else {
                    new_leaf_digest = JMT_SPARSE_PLACEHOLDER_HASH_V2;
                    state.new_current = JMT_SPARSE_PLACEHOLDER_HASH_V2;
                }
            }
            5 => {
                state.update_index = take_u32(record, 2)?;
                state.operation_index = take_u32(record, 6)?;
                state.update_current = state.new_current;
                let before = state.completed_operations;
                state.completed_operations =
                    before.checked_add(1).ok_or(CheckpointError::Overflow)?;
                aux = u16::from((before & 0xffff) == 0xffff);
            }
            6 => {
                state.update_index = take_u32(record, 2)?;
                aux = u16::from((state.update_index & 0xffff) == 0xffff);
                state.mutation_case = 0;
            }
            7 => {
                state.update_index = take_u32(record, 2)?;
                state.operation_index = take_u32(record, 6)?;
                aux = take_u16(record, 10)?;
                let sibling_type = record[12];
                sibling_digest = match sibling_type {
                    0 => JMT_SPARSE_PLACEHOLDER_HASH_V2,
                    1 | 2 => raw_sha_digest(&record[19..147])?,
                    _ => return Err(CheckpointError::Canonical),
                };
                old_parent_digest = raw_sha_digest(&record[147..275])?;
                let consumed_before = state.consumed_siblings;
                state.consumed_siblings = consumed_before
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                bit_index = Some(
                    state
                        .expected_siblings
                        .checked_sub(state.consumed_siblings)
                        .ok_or(CheckpointError::Canonical)?,
                );
                state.old_current = old_parent_digest;
                if record[14] == 1 {
                    new_parent_digest = raw_sha_digest(&record[275..403])?;
                    state.new_current = new_parent_digest;
                    state.new_parent_started = true;
                } else if state.mutation_case == 6
                    && !state.new_parent_started
                    && !state.coalesced
                    && sibling_type == 2
                {
                    state.new_current = sibling_digest;
                    state.coalesced = true;
                }
            }
            8 => {
                state.update_index = take_u32(record, 2)?;
                state.operation_index = take_u32(record, 6)?;
            }
            9 => {
                state.update_index = take_u32(record, 2)?;
                state.operation_index = take_u32(record, 6)?;
                aux = take_u16(record, 10)?;
                let sibling_type = record[12];
                sibling_digest = match sibling_type {
                    0 => JMT_SPARSE_PLACEHOLDER_HASH_V2,
                    1 | 2 => raw_sha_digest(&record[19..147])?,
                    _ => return Err(CheckpointError::Canonical),
                };
                new_parent_digest = raw_sha_digest(&record[147..275])?;
                state.consumed_split = state
                    .consumed_split
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                let total = state
                    .expected_split
                    .checked_add(state.expected_siblings)
                    .ok_or(CheckpointError::Overflow)?;
                bit_index = Some(
                    total
                        .checked_sub(state.consumed_split)
                        .ok_or(CheckpointError::Canonical)?,
                );
                state.new_current = new_parent_digest;
            }
            _ => return Err(CheckpointError::Canonical),
        }

        let mut values = if record_index == 0 {
            public.clone()
        } else {
            vec![KoalaBear::ZERO; PUBLIC_FIELDS_V2]
        };
        values.push(KoalaBear::ONE);
        for candidate in 1..=OPCODE_COUNT_V2 {
            values.push(KoalaBear::from_bool(opcode == candidate));
        }
        values.push(KoalaBear::from_usize(record_index + 1));
        values.push(KoalaBear::from_usize(record.len()));
        values.push(KoalaBear::from_u16(state.update_index as u16));
        values.push(KoalaBear::from_u16((state.update_index >> 16) as u16));
        values.push(KoalaBear::from_u16(state.operation_index as u16));
        values.push(KoalaBear::from_u16((state.operation_index >> 16) as u16));
        values.push(KoalaBear::from_u16(aux));
        values.push(KoalaBear::from_bool(state.value_present));
        values.push(KoalaBear::from_bool(state.prior_present));
        values.push(KoalaBear::from_bool(state.leaf_present));
        values.push(KoalaBear::from_bool(opcode == 7 && record[14] == 1));
        values.push(KoalaBear::from_bool(
            matches!(opcode, 7 | 9) && record[13] == 1,
        ));
        values.push(KoalaBear::from_u16(state.expected_siblings));
        values.push(KoalaBear::from_u16(state.consumed_siblings));
        values.push(KoalaBear::from_u16(state.expected_split));
        values.push(KoalaBear::from_u16(state.consumed_split));
        values.push(KoalaBear::from_u16(state.expected_operations as u16));
        values.push(KoalaBear::from_u16(
            (state.expected_operations >> 16) as u16,
        ));
        values.push(KoalaBear::from_u16(state.completed_operations as u16));
        values.push(KoalaBear::from_u16(
            (state.completed_operations >> 16) as u16,
        ));
        values.push(KoalaBear::from_bool(state.coalesced));
        values.push(KoalaBear::from_bool(state.new_parent_started));
        for role in 1..=ROLE_COUNT_V2 {
            values.push(KoalaBear::from_bool(usize::from(state.role) == role));
        }
        for mutation_case in 1..=CASE_COUNT_V2 {
            values.push(KoalaBear::from_bool(
                usize::from(state.mutation_case) == mutation_case,
            ));
        }
        for sibling_type in 0..SIBLING_TYPE_COUNT_V2 {
            values.push(KoalaBear::from_bool(
                matches!(opcode, 7 | 9) && usize::from(record[12]) == sibling_type,
            ));
        }
        values.extend(
            record
                .iter()
                .copied()
                .chain(core::iter::repeat_n(0, JMT_RECORD_BYTES_V2 - record.len()))
                .map(KoalaBear::from_u8),
        );
        values.extend(state.key.map(KoalaBear::from_u8));
        values.extend(state.path_key.map(KoalaBear::from_u8));
        append_digest(&mut values, state.update_current);
        append_digest(&mut values, state.update_new_root);
        append_digest(&mut values, state.old_current);
        append_digest(&mut values, state.new_current);
        append_digest(&mut values, sibling_digest);
        append_digest(&mut values, old_parent_digest);
        append_digest(&mut values, new_parent_digest);
        append_digest(&mut values, old_leaf_digest);
        append_digest(&mut values, new_leaf_digest);

        let (byte_position, bit_position, selected_byte) =
            bit_index.map_or((None, None, 0_u8), |index| {
                let byte_position = usize::from(index / 8);
                let bit_position = usize::from(index % 8);
                let selected_byte = if opcode == 9 {
                    state.key[byte_position]
                } else {
                    state.path_key[byte_position]
                };
                (Some(byte_position), Some(bit_position), selected_byte)
            });
        for candidate in 0..BYTE_POSITION_COUNT_V2 {
            values.push(KoalaBear::from_bool(byte_position == Some(candidate)));
        }
        for candidate in 0..BIT_POSITION_COUNT_V2 {
            values.push(KoalaBear::from_bool(bit_position == Some(candidate)));
        }
        for bit in 0..SELECTED_BITS_COUNT_V2 {
            values.push(KoalaBear::from_bool((selected_byte >> bit) & 1 == 1));
        }
        values.push(KoalaBear::from_usize(state.operation_job));
        values.push(KoalaBear::from_u64(u64::from(state.expected_value_bytes)));
        values.push(KoalaBear::from_u64(u64::from(
            state.expected_prior_value_bytes,
        )));
        values.push(KoalaBear::from_u64(u64::from(
            state.prior_value_block_count,
        )));
        values.push(KoalaBear::from_u64(u64::from(state.new_value_block_count)));
        values.push(KoalaBear::from_bool(
            opcode == 3 && state.value_padding_started,
        ));
        values.push(KoalaBear::from_bool(opcode == 3 && value_padding_start));
        for remainder in 0..64 {
            values.push(KoalaBear::from_bool(
                opcode == 3 && value_remainder == Some(remainder),
            ));
        }
        values.extend(state.tree_definition.map(KoalaBear::from_u8));
        values.extend(state.tree_serial.map(KoalaBear::from_u8));
        if values.len() != CALL_FIELDS_V2 {
            return Err(CheckpointError::Invariant);
        }
        result.push(JmtRowV2 { values });
    }

    for _ in records.len()..padded_rows {
        let mut values = vec![KoalaBear::ZERO; CALL_FIELDS_V2];
        values[PUBLIC_FIELDS_V2 + RUNNING_OFFSET_V2] = KoalaBear::from_usize(records.len());
        result.push(JmtRowV2 { values });
    }
    Ok(result)
}

pub(super) fn chunk_trace(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
) -> Result<JmtChunkTraceV2, CheckpointError> {
    if bindings.len() != prepared.len() || bindings.is_empty() || bindings.len() > CHUNK_LANES_V2 {
        return Err(CheckpointError::Invariant);
    }
    let public = chunk_public_values(statement, bindings)?;
    let inactive_header = canonical_noop_header();
    let mut lane_rows = Vec::with_capacity(CHUNK_LANES_V2);
    let mut lane_record_counts = Vec::with_capacity(CHUNK_LANES_V2);
    for lane in 0..CHUNK_LANES_V2 {
        let lane_public = chunk_lane_public_values(&public, lane)?;
        if let (Some(binding), Some(prepared)) = (bindings.get(lane), prepared.get(lane)) {
            if prepared.binding() != *binding {
                return Err(CheckpointError::Invariant);
            }
            let binding = binding.inputs();
            let stream = plonky3_epoch_event_stream::transition_event_stream(&prepared.material)?;
            let header = stream.jmt_header();
            let records = stream.jmt_micro_records().collect::<Vec<_>>();
            let record_count = u64::try_from(records.len()).map_err(|_| CheckpointError::Limit)?;
            let update_count = u64::from(stream.jmt().update_count());
            let expected_kind = if record_count == 0 {
                JMT_TRACE_NOOP_KIND_V2
            } else {
                JMT_TRACE_MUTATING_KIND_V2
            };
            if header[0] != JMT_UPDATE_TRACE_VERSION_V2
                || header[1] != RootGeneration::SettlementV2.version()
                || header[2] != expected_kind
                || header[3..35] != binding.update_trace_digest
                || u64::from(u32::from_le_bytes(
                    header[35..39]
                        .try_into()
                        .map_err(|_| CheckpointError::Canonical)?,
                )) != binding.jmt_update_count
                || record_count != binding.jmt_record_count
                || update_count != binding.jmt_update_count
                || stream.jmt().trace_digest() != binding.update_trace_digest
                || stream.jmt().promoted_definition_root() != binding.post_definition_root
            {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
                ));
            }
            lane_record_counts
                .push(usize::try_from(record_count).map_err(|_| CheckpointError::Limit)?);
            lane_rows.push(rows_with_public(lane_public, header, &records)?);
        } else {
            let records: [&[u8]; 0] = [];
            lane_record_counts.push(0);
            lane_rows.push(rows_with_public(lane_public, &inactive_header, &records)?);
        }
    }
    let height = lane_rows
        .iter()
        .map(Vec::len)
        .max()
        .ok_or(CheckpointError::Invariant)?;
    let mut rows = Vec::with_capacity(height);
    for row_index in 0..height {
        let mut values = Vec::with_capacity(CHUNK_LANES_V2 * ROW_FIELDS_V2);
        for lane in 0..CHUNK_LANES_V2 {
            if let Some(row) = lane_rows[lane].get(row_index) {
                values.extend_from_slice(&row.values[PUBLIC_FIELDS_V2..]);
            } else {
                let start = values.len();
                values.resize(start + ROW_FIELDS_V2, KoalaBear::ZERO);
                values[start + RUNNING_OFFSET_V2] = KoalaBear::from_usize(lane_record_counts[lane]);
            }
        }
        if values.len() != CHUNK_LANES_V2 * ROW_FIELDS_V2 {
            return Err(CheckpointError::Invariant);
        }
        rows.push(JmtChunkRowV2 { values });
    }
    Ok(JmtChunkTraceV2 {
        public_values: public,
        rows,
    })
}

pub(super) fn verify_batch(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    expected_public: &[KoalaBear],
    table_packing: TablePacking,
) -> Result<(), CheckpointError> {
    let mut verifier =
        BatchStarkProver::new(hardened_koala_bear_config()).with_table_packing(table_packing);
    verifier.register_table_prover(Box::new(JmtProverV2));
    verifier
        .verify_all_tables::<KoalaBear>(proof)
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 epoch JMT actual verifier rejected proof: {error}"
            ))
        })?;
    let mut entries = proof
        .non_primitives
        .iter()
        .filter(|entry| entry.op_type == jmt_npo_type());
    let actual_public = entries
        .next()
        .map(|entry| entry.public_values.as_slice())
        .ok_or(CheckpointError::BackendVerificationFailed)?;
    if entries.next().is_some() || actual_public != expected_public {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn check_constraints_for_rows(rows: &[JmtRowV2], expected_public: &[KoalaBear]) {
    let air = JmtAirV2::<KoalaBear, 1>::new(vec![KoalaBear::ONE; rows.len()], rows.len());
    let matrix = JmtAirV2::<KoalaBear, 1>::trace_to_matrix(rows, rows.len());
    p3_air::check_constraints(&air, &matrix, expected_public);
}

pub(super) fn prove(
    statement: &EpochTraceChunkV2,
    header: &[u8; JMT_CIRCUIT_HEADER_BYTES_V2],
    records: &[Vec<u8>],
) -> Result<BatchStarkProof<Plonky3StarkConfigV2>, CheckpointError> {
    let rows = rows(statement, header, records)?;
    let expected_public = public_values(statement, header)?;
    #[cfg(test)]
    check_constraints_for_rows(&rows, &expected_public);
    let trace_rows = rows.len();
    let traces: Traces<KoalaBear> = Traces {
        witness_trace: WitnessTrace::new(Vec::new()),
        const_trace: ConstTrace {
            index: Vec::new(),
            values: Vec::new(),
        },
        public_trace: PublicTrace {
            index: Vec::new(),
            values: Vec::new(),
        },
        alu_trace: AluTrace::from_records(Vec::new()),
        non_primitive_traces: [(
            jmt_npo_type(),
            Box::new(JmtTraceV2 { rows }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        )]
        .into_iter()
        .collect(),
        tag_to_witness: Default::default(),
    };
    let table_packing = TablePacking::new(1, 1).with_min_trace_height(trace_rows);
    let mut prover = BatchStarkProver::new(hardened_koala_bear_config())
        .with_table_packing(table_packing.clone());
    prover.register_table_prover(Box::new(JmtProverV2));
    let proof = prover.prove_direct_tables(&traces).map_err(|error| {
        CheckpointError::Backend(format!("Plonky3 epoch JMT prove failed: {error}"))
    })?;
    drop(traces);
    verify_batch(&proof, &expected_public, table_packing)?;
    Ok(proof)
}
