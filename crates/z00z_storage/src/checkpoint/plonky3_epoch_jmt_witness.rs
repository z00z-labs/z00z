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
    jmt_npo_type, JmtRowV2, JmtTraceV2, BIT_POSITION_COUNT_V2, BYTE_POSITION_COUNT_V2,
    CALL_FIELDS_V2, CASE_COUNT_V2, JMT_MIN_ROWS_V2, JMT_RECORD_BYTES_V2, OPCODE_COUNT_V2,
    PUBLIC_FIELDS_V2, RECORD_LENGTHS_V2, ROLE_COUNT_V2, RUNNING_OFFSET_V2, SELECTED_BITS_COUNT_V2,
    SIBLING_TYPE_COUNT_V2,
};
use super::plonky3_epoch_jmt_table::JmtProverV2;
use super::plonky3_epoch_sha256_witness::{compress as sha_compress, words_bytes};
use super::{
    hardened_koala_bear_config, EpochAirTableV2, EpochTraceChunkV2, Plonky3StarkConfigV2,
    RecursiveCheckpointRejectReasonV2, EPOCH_CHUNK_BYTES_V2,
};
use crate::settlement::{
    SettlementUpdateTraceCircuitDecoderV2, JMT_CIRCUIT_HEADER_BYTES_V2,
    JMT_SPARSE_PLACEHOLDER_HASH_V2,
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
    role: u8,
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
            role: 0,
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
    core::array::from_fn(|index| u16::from_le_bytes([digest[index * 2], digest[index * 2 + 1]]))
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
    if values.len() != PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

pub(super) fn rows(
    statement: &EpochTraceChunkV2,
    header: &[u8],
    records: &[Vec<u8>],
) -> Result<Vec<JmtRowV2>, CheckpointError> {
    if records.is_empty()
        || header.len() != JMT_CIRCUIT_HEADER_BYTES_V2
        || statement.inputs().table != EpochAirTableV2::JmtUpdate
        || statement.inputs().replica != 0
        || statement.inputs().row_count
            != u64::try_from(records.len()).map_err(|_| CheckpointError::Limit)?
    {
        return Err(CheckpointError::Canonical);
    }
    let mut decoder = SettlementUpdateTraceCircuitDecoderV2::new(header)
        .map_err(|error| CheckpointError::Backend(format!("JMT header rejected: {error}")))?;
    for record in records {
        decoder
            .accept(record)
            .map_err(|error| CheckpointError::Backend(format!("JMT record rejected: {error}")))?;
    }
    decoder
        .finish()
        .map_err(|error| CheckpointError::Backend(format!("JMT transcript rejected: {error}")))?;

    let public = public_values(statement, header)?;
    let padded_rows = records.len().max(JMT_MIN_ROWS_V2).next_power_of_two();
    let mut result = Vec::with_capacity(padded_rows);
    let mut state = JmtWitnessStateV2::default();
    for (record_index, record) in records.iter().enumerate() {
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

        match opcode {
            1 => {
                state.update_index = take_u32(record, 2)?;
                state.role = record[6];
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
                state.mutation_case = 0;
                state.key = [0; 32];
                state.path_key = [0; 32];
                state.old_current = [0; 32];
                state.new_current = [0; 32];
            }
            2 => {
                state.update_index = take_u32(record, 2)?;
                state.operation_index = take_u32(record, 6)?;
                state.key = take_array(record, 10)?;
                state.value_present = record[42] == 1;
                state.prior_present = record[47] == 1;
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
                aux = u16::try_from(take_u32(record, 10)?).map_err(|_| CheckpointError::Limit)?;
            }
            4 => {
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
