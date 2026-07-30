//! Canonical witness construction for one epoch SHA-256 compression table.

use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;

use super::plonky3_epoch_sha256_air::{
    ShaRowV2, CALL_FIELDS_V2, PUBLIC_FIELDS_V2, SHA_ROWS_V2, STATE_BIT_WORDS_V2,
};
use super::{EpochAirTableV2, EpochTraceChunkV2, EPOCH_CHUNK_BYTES_V2, SHA256_ROUND_CONSTANTS_V2};
use crate::CheckpointError;

fn push_word(values: &mut Vec<KoalaBear>, word: u32) {
    values.push(KoalaBear::from_u16((word & 0xffff) as u16));
    values.push(KoalaBear::from_u16((word >> 16) as u16));
}

fn push_bits(values: &mut Vec<KoalaBear>, value: u32, bits: usize) {
    for bit in 0..bits {
        values.push(KoalaBear::from_bool((value >> bit) & 1 == 1));
    }
}

fn block_words(block: &[u8; 64]) -> [u32; 16] {
    core::array::from_fn(|word| {
        let offset = word * 4;
        u32::from_be_bytes([
            block[offset],
            block[offset + 1],
            block[offset + 2],
            block[offset + 3],
        ])
    })
}

fn small_sigma_0(value: u32) -> u32 {
    value.rotate_right(7) ^ value.rotate_right(18) ^ (value >> 3)
}

fn small_sigma_1(value: u32) -> u32 {
    value.rotate_right(17) ^ value.rotate_right(19) ^ (value >> 10)
}

fn schedule(block: &[u8; 64], words: usize) -> Vec<u32> {
    let mut result = Vec::with_capacity(words.max(16));
    result.extend_from_slice(&block_words(block));
    while result.len() < words {
        let index = result.len();
        result.push(
            result[index - 16]
                .wrapping_add(small_sigma_0(result[index - 15]))
                .wrapping_add(result[index - 7])
                .wrapping_add(small_sigma_1(result[index - 2])),
        );
    }
    result
}

fn add_words(terms: &[u32]) -> (u32, u8, u8) {
    let low_sum = terms
        .iter()
        .map(|value| u64::from(value & 0xffff))
        .sum::<u64>();
    let low_carry = (low_sum >> 16) as u8;
    let high_sum = terms
        .iter()
        .map(|value| u64::from(value >> 16))
        .sum::<u64>()
        + u64::from(low_carry);
    let high_carry = (high_sum >> 16) as u8;
    let word = (low_sum as u32 & 0xffff) | ((high_sum as u32 & 0xffff) << 16);
    (word, low_carry, high_carry)
}

fn round_step(state: [u32; 8], word: u32, constant: u32) -> ([u32; 8], u32, u32) {
    let sigma_1 = state[4].rotate_right(6) ^ state[4].rotate_right(11) ^ state[4].rotate_right(25);
    let choose = (state[4] & state[5]) ^ (!state[4] & state[6]);
    let t1 = state[7]
        .wrapping_add(sigma_1)
        .wrapping_add(choose)
        .wrapping_add(constant)
        .wrapping_add(word);
    let sigma_0 = state[0].rotate_right(2) ^ state[0].rotate_right(13) ^ state[0].rotate_right(22);
    let majority = (state[0] & state[1]) ^ (state[0] & state[2]) ^ (state[1] & state[2]);
    let t2 = sigma_0.wrapping_add(majority);
    (
        [
            t1.wrapping_add(t2),
            state[0],
            state[1],
            state[2],
            state[3].wrapping_add(t1),
            state[4],
            state[5],
            state[6],
        ],
        t1,
        t2,
    )
}

pub(super) fn compress(input: [u32; 8], block: &[u8; 64]) -> [u32; 8] {
    let words = schedule(block, SHA_ROWS_V2);
    let mut state = input;
    for round in 0..SHA_ROWS_V2 {
        state = round_step(state, words[round], SHA256_ROUND_CONSTANTS_V2[round]).0;
    }
    core::array::from_fn(|word| input[word].wrapping_add(state[word]))
}

pub(super) fn words_bytes(words: [u32; 8]) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    for (index, word) in words.into_iter().enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    bytes
}

pub(super) fn public_values(
    statement: &EpochTraceChunkV2,
    input: [u32; 8],
    block: &[u8; 64],
    output: [u32; 8],
) -> Result<Vec<KoalaBear>, CheckpointError> {
    if statement.canonical_bytes().len() != EPOCH_CHUNK_BYTES_V2 {
        return Err(CheckpointError::Canonical);
    }
    let mut values = Vec::with_capacity(PUBLIC_FIELDS_V2);
    values.extend(
        statement
            .canonical_bytes()
            .chunks_exact(2)
            .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
    );
    for word in input {
        push_word(&mut values, word);
    }
    for word in block_words(block) {
        push_word(&mut values, word);
    }
    for word in output {
        push_word(&mut values, word);
    }
    if values.len() != PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

pub(super) fn rows(
    statement: &EpochTraceChunkV2,
    input: [u32; 8],
    block: &[u8; 64],
) -> Result<(Vec<ShaRowV2>, [u32; 8]), CheckpointError> {
    let inputs = statement.inputs();
    if inputs.table != EpochAirTableV2::Sha256 || inputs.replica != 0 || inputs.row_count != 1 {
        return Err(CheckpointError::Canonical);
    }
    let words = schedule(block, SHA_ROWS_V2 + 15);
    let output = compress(input, block);
    let public = public_values(statement, input, block, output)?;
    let mut state = input;
    let mut result = Vec::with_capacity(SHA_ROWS_V2);
    for round in 0..SHA_ROWS_V2 {
        let mut values = if round == 0 {
            public.clone()
        } else {
            vec![KoalaBear::ZERO; PUBLIC_FIELDS_V2]
        };
        for word in state {
            push_word(&mut values, word);
        }
        for word in STATE_BIT_WORDS_V2 {
            push_bits(&mut values, state[word], 32);
        }
        for word in &words[round..round + 16] {
            push_word(&mut values, *word);
        }
        push_bits(&mut values, words[round + 1], 32);
        push_bits(&mut values, words[round + 14], 32);
        for selector in 0..SHA_ROWS_V2 {
            values.push(KoalaBear::from_bool(selector == round));
        }

        let (next, t1, t2) = round_step(state, words[round], SHA256_ROUND_CONSTANTS_V2[round]);
        push_bits(&mut values, t1, 32);
        push_bits(&mut values, t2, 32);
        if round + 1 < SHA_ROWS_V2 {
            let (_, low, high) = add_words(&[
                words[round],
                small_sigma_0(words[round + 1]),
                words[round + 9],
                small_sigma_1(words[round + 14]),
            ]);
            push_bits(&mut values, u32::from(low), 2);
            push_bits(&mut values, u32::from(high), 2);
        } else {
            values.extend(core::iter::repeat_n(KoalaBear::ZERO, 4));
        }
        let sigma_1 =
            state[4].rotate_right(6) ^ state[4].rotate_right(11) ^ state[4].rotate_right(25);
        let choose = (state[4] & state[5]) ^ (!state[4] & state[6]);
        let (_, low, high) = add_words(&[
            state[7],
            sigma_1,
            choose,
            SHA256_ROUND_CONSTANTS_V2[round],
            words[round],
        ]);
        push_bits(&mut values, u32::from(low), 3);
        push_bits(&mut values, u32::from(high), 3);
        let sigma_0 =
            state[0].rotate_right(2) ^ state[0].rotate_right(13) ^ state[0].rotate_right(22);
        let majority = (state[0] & state[1]) ^ (state[0] & state[2]) ^ (state[1] & state[2]);
        let (_, low, high) = add_words(&[sigma_0, majority]);
        push_bits(&mut values, u32::from(low), 1);
        push_bits(&mut values, u32::from(high), 1);
        let (_, low, high) = add_words(&[state[3], t1]);
        push_bits(&mut values, u32::from(low), 1);
        push_bits(&mut values, u32::from(high), 1);
        let (_, low, high) = add_words(&[t1, t2]);
        push_bits(&mut values, u32::from(low), 1);
        push_bits(&mut values, u32::from(high), 1);
        if round + 1 == SHA_ROWS_V2 {
            for word in 0..8 {
                let (_, low, high) = add_words(&[input[word], next[word]]);
                push_bits(&mut values, u32::from(low), 1);
                push_bits(&mut values, u32::from(high), 1);
            }
        } else {
            values.extend(core::iter::repeat_n(KoalaBear::ZERO, 16));
        }
        if values.len() != CALL_FIELDS_V2 {
            return Err(CheckpointError::Invariant);
        }
        result.push(ShaRowV2 { values });
        state = next;
    }
    Ok((result, output))
}
