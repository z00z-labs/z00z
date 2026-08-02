//! Canonical witness construction for one epoch SHA-256 compression table.

use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;
use z00z_crypto::{
    sha256_256_role, CheckpointSha256BlockStreamV2, CheckpointSha256BlockV2,
    CheckpointSha256BlockVisitError, CheckpointShaRole, SHA256_IV_V2,
};

use super::plonky3_epoch_event_stream::transition_event_stream;
use super::plonky3_epoch_sha256_columns::{
    SemanticShaJobKindV2, ShaAirRoleV2, ShaRowV2, ShaTraceV2, CHAIN_PADDING_SLOT_V2,
    CHAIN_PUBLIC_FIELDS_V2, CHAIN_SELECTOR_COUNT_V2, CHAIN_TRANSITION_SLOTS_V2,
    JMT_LINKED_PUBLIC_FIELDS_V2, JMT_SHA_ROLE_NEW_LEAF_V2, JMT_SHA_ROLE_NEW_PARENT_V2,
    JMT_SHA_ROLE_NEW_VALUE_V2, JMT_SHA_ROLE_OLD_LEAF_V2, JMT_SHA_ROLE_OLD_PARENT_V2,
    JMT_SHA_ROLE_PRIOR_VALUE_V2, JMT_SHA_ROLE_SIBLING_V2, ROW_FIELDS_V2, SHA_ROWS_V2,
    STANDALONE_PUBLIC_FIELDS_V2, STATE_BIT_WORDS_V2,
};
use super::plonky3_epoch_uniqueness_slice::EpochUniquenessSliceV2;
use super::{
    decode_uniqueness_precommit, decode_uniqueness_sorted_row, EpochAirTableV2,
    EpochPreparedTransitionV2, EpochTraceChunkV2, EpochTransitionBindingV2,
    RecursiveCheckpointRejectReasonV2, RecursiveTraceOpcodeV2, UniquenessListKindV2,
    UniquenessPassV2, UniquenessSetKindV2, EPOCH_CHUNK_BYTES_V2, SHA256_ROUND_CONSTANTS_V2,
};
use crate::checkpoint::recursive_semantics::UNIQUENESS_PRECOMMIT_LABEL_V2;
use crate::checkpoint::recursive_trace::STRUCTURAL_EVENT_HASH_LABEL_V2;
use crate::CheckpointError;

#[derive(Clone, Debug)]
pub(super) struct SemanticShaJobV2 {
    pub(super) kind: SemanticShaJobKindV2,
    pub(super) id: u64,
    pub(super) role: CheckpointShaRole,
    pub(super) parts: Vec<Vec<u8>>,
    pub(super) expected_digest: [u8; 32],
}

impl SemanticShaJobV2 {
    fn new(
        kind: SemanticShaJobKindV2,
        id: u64,
        role: CheckpointShaRole,
        parts: Vec<Vec<u8>>,
        expected_digest: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        let refs = parts.iter().map(Vec::as_slice).collect::<Vec<_>>();
        if sha256_256_role(role, &refs) != expected_digest {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        Ok(Self {
            kind,
            id,
            role,
            parts,
            expected_digest,
        })
    }

    pub(super) fn framed_message(&self) -> Result<Vec<u8>, CheckpointError> {
        let mut bytes = CheckpointSha256BlockStreamV2::framed_role_prefix(self.role);
        let additional = self.parts.iter().try_fold(0_usize, |total, part| {
            total
                .checked_add(8)
                .and_then(|value| value.checked_add(part.len()))
                .ok_or(CheckpointError::Overflow)
        })?;
        bytes
            .try_reserve_exact(additional)
            .map_err(|_| CheckpointError::Limit)?;
        for part in &self.parts {
            bytes.extend_from_slice(
                &u64::try_from(part.len())
                    .map_err(|_| CheckpointError::Limit)?
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(part);
        }
        Ok(bytes)
    }

    pub(super) fn visit_blocks<F>(&self, visit: &mut F) -> Result<[u8; 32], CheckpointError>
    where
        F: FnMut(CheckpointSha256BlockV2) -> Result<(), CheckpointError>,
    {
        let mut stream = CheckpointSha256BlockStreamV2::new(self.role);
        for part in &self.parts {
            stream
                .update_part_with(part, visit)
                .map_err(map_semantic_sha_visit_error)?;
        }
        stream
            .finalize_with(visit)
            .map_err(map_semantic_sha_visit_error)
    }
}

fn map_semantic_sha_visit_error(
    error: CheckpointSha256BlockVisitError<CheckpointError>,
) -> CheckpointError {
    match error {
        CheckpointSha256BlockVisitError::Hash(error) => {
            CheckpointError::Backend(format!("semantic SHA block stream failed: {error}"))
        }
        CheckpointSha256BlockVisitError::Visitor(error) => error,
    }
}

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
    let mut values = Vec::with_capacity(STANDALONE_PUBLIC_FIELDS_V2);
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
    if values.len() != STANDALONE_PUBLIC_FIELDS_V2 {
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
    compression_rows(input, block)
}

pub(super) fn compression_rows(
    input: [u32; 8],
    block: &[u8; 64],
) -> Result<(Vec<ShaRowV2>, [u32; 8]), CheckpointError> {
    let mut result = Vec::with_capacity(SHA_ROWS_V2);
    let output = append_block_rows(&mut result, 0, 0, true, input, block)?;
    Ok((result, output))
}

fn append_block_rows(
    result: &mut Vec<ShaRowV2>,
    transition_slot: usize,
    block_index: u64,
    transition_final: bool,
    input: [u32; 8],
    block: &[u8; 64],
) -> Result<[u32; 8], CheckpointError> {
    append_block_rows_with_jmt_meta(
        result,
        transition_slot,
        block_index,
        transition_final,
        input,
        block,
        None,
    )
}

#[derive(Clone, Copy)]
struct JmtShaJobMetaV2 {
    lane: usize,
    record: usize,
    role: u8,
    block_count: usize,
}

fn append_block_rows_with_jmt_meta(
    result: &mut Vec<ShaRowV2>,
    transition_slot: usize,
    block_index: u64,
    transition_final: bool,
    input: [u32; 8],
    block: &[u8; 64],
    jmt_meta: Option<JmtShaJobMetaV2>,
) -> Result<[u32; 8], CheckpointError> {
    if transition_slot >= CHAIN_SELECTOR_COUNT_V2 {
        return Err(CheckpointError::Limit);
    }
    let words = schedule(block, SHA_ROWS_V2 + 15);
    let output = compress(input, block);
    let mut state = input;
    for round in 0..SHA_ROWS_V2 {
        let mut values = Vec::with_capacity(ROW_FIELDS_V2);
        for slot in 0..CHAIN_SELECTOR_COUNT_V2 {
            values.push(KoalaBear::from_bool(slot == transition_slot));
        }
        values.push(KoalaBear::from_u64(block_index));
        values.push(KoalaBear::from_bool(
            transition_final && round + 1 == SHA_ROWS_V2,
        ));
        for word in input {
            push_word(&mut values, word);
        }
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
        values.push(jmt_meta.map_or(KoalaBear::ZERO, |meta| KoalaBear::from_usize(meta.lane)));
        values.push(jmt_meta.map_or(KoalaBear::ZERO, |meta| KoalaBear::from_usize(meta.record)));
        values.push(jmt_meta.map_or(KoalaBear::ZERO, |meta| KoalaBear::from_u8(meta.role)));
        values.push(jmt_meta.map_or(KoalaBear::ZERO, |meta| {
            KoalaBear::from_usize(meta.block_count)
        }));
        if values.len() != ROW_FIELDS_V2 {
            return Err(CheckpointError::Invariant);
        }
        result.push(ShaRowV2 { values });
        state = next;
    }
    Ok(output)
}

pub(super) fn expected_block_count(
    binding: EpochTransitionBindingV2,
) -> Result<u64, CheckpointError> {
    let vector_bytes = binding.inputs().event_bytes;
    let framed = CheckpointSha256BlockStreamV2::framed_bytes_for_parts(
        CheckpointShaRole::EventVector,
        vector_bytes,
        1,
    )
    .map_err(|_| CheckpointError::Limit)?;
    CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(framed)
        .map_err(|_| CheckpointError::Limit)
}

pub(super) fn semantic_jobs(
    role: ShaAirRoleV2,
    binding: EpochTransitionBindingV2,
    transition: &EpochPreparedTransitionV2,
) -> Result<Vec<SemanticShaJobV2>, CheckpointError> {
    if transition.binding() != binding
        || !matches!(
            role,
            ShaAirRoleV2::SemanticTransitionChain | ShaAirRoleV2::SemanticUniquenessChain
        )
    {
        return Err(CheckpointError::Invariant);
    }
    let stream = transition_event_stream(&transition.material)?;
    let mut jobs = vec![SemanticShaJobV2::new(
        SemanticShaJobKindV2::EventVector,
        0,
        CheckpointShaRole::EventVector,
        vec![stream.source().to_vec()],
        binding.inputs().event_vector_digest,
    )?];

    if role == ShaAirRoleV2::SemanticTransitionChain {
        for record in stream.records().iter().copied().filter(|record| {
            matches!(
                record.opcode(),
                RecursiveTraceOpcodeV2::JmtUpdate
                    | RecursiveTraceOpcodeV2::JmtMicroOp
                    | RecursiveTraceOpcodeV2::PromoteChildRoot
                    | RecursiveTraceOpcodeV2::CommitTypedEvent
            )
        }) {
            jobs.push(SemanticShaJobV2::new(
                SemanticShaJobKindV2::StructuralEventId,
                record.ordinal(),
                CheckpointShaRole::Trace,
                vec![
                    STRUCTURAL_EVENT_HASH_LABEL_V2.to_vec(),
                    vec![record.opcode() as u8],
                    record.ordinal().to_le_bytes().to_vec(),
                    record.payload().to_vec(),
                ],
                record.object_id(),
            )?);
        }
        return Ok(jobs);
    }

    let precommit_record = stream
        .records()
        .iter()
        .copied()
        .find(|record| record.opcode() == RecursiveTraceOpcodeV2::UniquenessPrecommit)
        .ok_or(CheckpointError::Canonical)?;
    let challenge_record = stream
        .records()
        .iter()
        .copied()
        .find(|record| record.opcode() == RecursiveTraceOpcodeV2::UniquenessChallenge)
        .ok_or(CheckpointError::Canonical)?;
    let precommit = decode_uniqueness_precommit(precommit_record.payload())?;
    let challenge = challenge_record.payload();
    if challenge.len() != super::UNIQUENESS_CHALLENGE_BYTES_V2
        || challenge.first().copied() != Some(super::UNIQUENESS_PRECOMMIT_VERSION_V2)
        || challenge[1..33] != precommit.precommit_digest
    {
        return Err(CheckpointError::Canonical);
    }

    let mut lists: [Vec<Vec<u8>>; 4] = core::array::from_fn(|_| Vec::new());
    for record in stream
        .records()
        .iter()
        .copied()
        .filter(|record| record.opcode() == RecursiveTraceOpcodeV2::UniquenessSorted)
    {
        let (pass, set, list, semantic) = decode_uniqueness_sorted_row(record.payload())?;
        if pass != UniquenessPassV2::Commit {
            continue;
        }
        let id = usize::from(set == UniquenessSetKindV2::Output) * 2
            + usize::from(list == UniquenessListKindV2::Sorted);
        lists[id].push(semantic.canonical_bytes().to_vec());
    }
    let list_roles = [
        CheckpointShaRole::SpentOriginalIds,
        CheckpointShaRole::SpentSortedIds,
        CheckpointShaRole::OutputOriginalIds,
        CheckpointShaRole::OutputSortedIds,
    ];
    let list_expected = [
        precommit.spent_original_digest,
        precommit.spent_sorted_digest,
        precommit.output_original_digest,
        precommit.output_sorted_digest,
    ];
    let list_counts = [
        precommit.spent_count,
        precommit.spent_count,
        precommit.output_count,
        precommit.output_count,
    ];
    for id in 0..lists.len() {
        if usize::try_from(list_counts[id]).map_err(|_| CheckpointError::Limit)? != lists[id].len()
        {
            return Err(CheckpointError::Canonical);
        }
        let mut parts = Vec::with_capacity(lists[id].len() + 1);
        parts.push(list_counts[id].to_le_bytes().to_vec());
        parts.append(&mut lists[id]);
        jobs.push(SemanticShaJobV2::new(
            SemanticShaJobKindV2::UniquenessList,
            u64::try_from(id).map_err(|_| CheckpointError::Limit)?,
            list_roles[id],
            parts,
            list_expected[id],
        )?);
    }

    jobs.push(SemanticShaJobV2::new(
        SemanticShaJobKindV2::UniquenessPrecommit,
        0,
        CheckpointShaRole::IdPrecommit,
        vec![
            UNIQUENESS_PRECOMMIT_LABEL_V2.to_vec(),
            precommit.spent_count.to_le_bytes().to_vec(),
            precommit.output_count.to_le_bytes().to_vec(),
            precommit.spent_original_digest.to_vec(),
            precommit.spent_sorted_digest.to_vec(),
            precommit.output_original_digest.to_vec(),
            precommit.output_sorted_digest.to_vec(),
        ],
        precommit.precommit_digest,
    )?);

    let context: [u8; 32] = challenge[33..65]
        .try_into()
        .map_err(|_| CheckpointError::Canonical)?;
    let set_precommits: [[u8; 32]; 2] = [
        challenge[65..97]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
        challenge[97..129]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?,
    ];
    for set in 0..2 {
        let count = if set == 0 {
            precommit.spent_count
        } else {
            precommit.output_count
        };
        let (original, sorted) = if set == 0 {
            (
                precommit.spent_original_digest,
                precommit.spent_sorted_digest,
            )
        } else {
            (
                precommit.output_original_digest,
                precommit.output_sorted_digest,
            )
        };
        jobs.push(SemanticShaJobV2::new(
            SemanticShaJobKindV2::UniquenessSetPrecommit,
            u64::try_from(set).map_err(|_| CheckpointError::Limit)?,
            CheckpointShaRole::IdPrecommit,
            vec![
                context.to_vec(),
                vec![u8::try_from(set).map_err(|_| CheckpointError::Limit)?],
                count.to_le_bytes().to_vec(),
                original.to_vec(),
                sorted.to_vec(),
            ],
            set_precommits[set],
        )?);
    }

    let grammar = RecursiveTraceOpcodeV2::grammar_digest();
    for id in 0_usize..8 {
        let set = id / 4;
        let coordinate_index = id % 4;
        let pair = coordinate_index / 2;
        let coordinate = coordinate_index % 2;
        let start = 129_usize
            .checked_add(id.checked_mul(32).ok_or(CheckpointError::Overflow)?)
            .ok_or(CheckpointError::Overflow)?;
        let expected: [u8; 32] = challenge[start..start + 32]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?;
        jobs.push(SemanticShaJobV2::new(
            SemanticShaJobKindV2::UniquenessChallenge,
            u64::try_from(id).map_err(|_| CheckpointError::Limit)?,
            CheckpointShaRole::IdChallenge,
            vec![
                set_precommits[set].to_vec(),
                grammar.to_vec(),
                vec![u8::try_from(set).map_err(|_| CheckpointError::Limit)?],
                vec![u8::try_from(pair).map_err(|_| CheckpointError::Limit)?],
                vec![u8::try_from(coordinate).map_err(|_| CheckpointError::Limit)?],
            ],
            expected,
        )?);
    }
    Ok(jobs)
}

fn push_digest_words(values: &mut Vec<KoalaBear>, digest: [u8; 32]) {
    for bytes in digest.chunks_exact(4) {
        push_word(
            values,
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        );
    }
}

fn semantic_chain_rows(real_blocks: usize) -> Result<usize, CheckpointError> {
    real_blocks
        .max(1)
        .checked_next_power_of_two()
        .and_then(|blocks| blocks.checked_mul(SHA_ROWS_V2))
        .ok_or(CheckpointError::Overflow)
}

pub(super) fn chain_public_values(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
) -> Result<Vec<KoalaBear>, CheckpointError> {
    chain_public_values_for_slice(
        statement,
        bindings,
        EpochUniquenessSliceV2::full(bindings.len())?,
    )
}

pub(super) fn chain_public_values_for_slice(
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
        || bindings.len() > CHAIN_TRANSITION_SLOTS_V2
    {
        return Err(CheckpointError::Canonical);
    }
    let mut values = Vec::with_capacity(CHAIN_PUBLIC_FIELDS_V2);
    values.extend(
        statement
            .canonical_bytes()
            .chunks_exact(2)
            .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
    );
    for slot in 0..CHAIN_TRANSITION_SLOTS_V2 {
        values.push(KoalaBear::from_bool(slot < bindings.len()));
    }
    for slot in 0..CHAIN_TRANSITION_SLOTS_V2 {
        values.push(KoalaBear::from_u64(
            bindings
                .get(slot)
                .copied()
                .map(expected_block_count)
                .transpose()?
                .unwrap_or(0),
        ));
    }
    for slot in 0..CHAIN_TRANSITION_SLOTS_V2 {
        if let Some(binding) = bindings.get(slot).copied() {
            push_digest_words(&mut values, binding.inputs().event_vector_digest);
        } else {
            values.extend(core::iter::repeat_n(KoalaBear::ZERO, 16));
        }
    }
    values.push(KoalaBear::from_usize(slice.start()));
    values.push(KoalaBear::from_usize(slice.len()));
    if values.len() != CHAIN_PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

pub(super) fn chain_trace(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
) -> Result<(ShaTraceV2, u64), CheckpointError> {
    if bindings.len() != prepared.len() || bindings.is_empty() {
        return Err(CheckpointError::Invariant);
    }
    let public_values = chain_public_values(statement, bindings)?;
    let mut rows = Vec::new();
    let mut total_real_blocks = 0_u64;
    for (slot, (binding, transition)) in bindings.iter().zip(prepared).enumerate() {
        if transition.binding() != *binding {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        let stream = transition_event_stream(&transition.material)?;
        let mut expected_index = 0_u64;
        let mut saw_final = false;
        let digest = stream.visit_digest_blocks(&mut |block: CheckpointSha256BlockV2| {
            if saw_final
                || block.index() != expected_index
                || !block.verifies_transition()
                || append_block_rows(
                    &mut rows,
                    slot,
                    block.index(),
                    block.final_block(),
                    *block.chaining_before(),
                    block.block(),
                )? != *block.chaining_after()
            {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
                ));
            }
            expected_index = expected_index
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
            saw_final = block.final_block();
            Ok(())
        })?;
        if !saw_final
            || expected_index != expected_block_count(*binding)?
            || digest != binding.inputs().event_vector_digest
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        total_real_blocks = total_real_blocks
            .checked_add(expected_index)
            .ok_or(CheckpointError::Overflow)?;
    }

    let padded_blocks = usize::try_from(total_real_blocks)
        .map_err(|_| CheckpointError::Limit)?
        .next_power_of_two();
    let padding_blocks = padded_blocks
        .checked_sub(usize::try_from(total_real_blocks).map_err(|_| CheckpointError::Limit)?)
        .ok_or(CheckpointError::Invariant)?;
    let mut padding_state = SHA256_IV_V2;
    let padding_block = [0_u8; 64];
    for index in 0..padding_blocks {
        padding_state = append_block_rows(
            &mut rows,
            CHAIN_PADDING_SLOT_V2,
            u64::try_from(index).map_err(|_| CheckpointError::Limit)?,
            false,
            padding_state,
            &padding_block,
        )?;
    }
    let expected_rows = padded_blocks
        .checked_mul(SHA_ROWS_V2)
        .ok_or(CheckpointError::Overflow)?;
    if rows.len() != expected_rows || !rows.len().is_power_of_two() {
        return Err(CheckpointError::Invariant);
    }
    Ok((
        ShaTraceV2 {
            role: ShaAirRoleV2::Chain,
            public_values,
            rows,
        },
        total_real_blocks,
    ))
}

pub(super) fn semantic_chain_trace(
    role: ShaAirRoleV2,
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
) -> Result<(ShaTraceV2, u64), CheckpointError> {
    semantic_chain_trace_for_slice(
        role,
        statement,
        bindings,
        prepared,
        EpochUniquenessSliceV2::full(bindings.len())?,
    )
}

pub(super) fn semantic_chain_trace_for_slice(
    role: ShaAirRoleV2,
    statement: &EpochTraceChunkV2,
    full_bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
    slice: EpochUniquenessSliceV2,
) -> Result<(ShaTraceV2, u64), CheckpointError> {
    let end = slice.end()?;
    let bindings = full_bindings
        .get(slice.start()..end)
        .ok_or(CheckpointError::Canonical)?;
    let prepared = prepared
        .get(slice.start()..end)
        .ok_or(CheckpointError::Canonical)?;
    if bindings.len() != prepared.len()
        || bindings.is_empty()
        || !matches!(
            role,
            ShaAirRoleV2::SemanticTransitionChain | ShaAirRoleV2::SemanticUniquenessChain
        )
    {
        return Err(CheckpointError::Invariant);
    }
    let public_values = chain_public_values_for_slice(statement, full_bindings, slice)?;
    let mut rows = Vec::new();
    let mut total_real_blocks = 0_u64;
    for (lane, (binding, transition)) in bindings.iter().zip(prepared).enumerate() {
        for job in semantic_jobs(role, *binding, transition)? {
            let raw_len =
                u64::try_from(job.framed_message()?.len()).map_err(|_| CheckpointError::Limit)?;
            let block_count = CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(raw_len)
                .map_err(|_| CheckpointError::Limit)?;
            let mut expected_index = 0_u64;
            let digest = job.visit_blocks(&mut |block| {
                if block.index() != expected_index
                    || !block.verifies_transition()
                    || append_block_rows_with_jmt_meta(
                        &mut rows,
                        job.kind.index(),
                        block.index(),
                        block.final_block(),
                        *block.chaining_before(),
                        block.block(),
                        Some(JmtShaJobMetaV2 {
                            lane,
                            record: usize::try_from(job.id).map_err(|_| CheckpointError::Limit)?,
                            role: job.kind as u8,
                            block_count: usize::try_from(block_count)
                                .map_err(|_| CheckpointError::Limit)?,
                        }),
                    )? != *block.chaining_after()
                {
                    return Err(CheckpointError::RecursiveRejected(
                        RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
                    ));
                }
                expected_index = expected_index
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                Ok(())
            })?;
            if digest != job.expected_digest || expected_index != block_count {
                return Err(CheckpointError::RecursiveRejected(
                    RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
                ));
            }
            total_real_blocks = total_real_blocks
                .checked_add(block_count)
                .ok_or(CheckpointError::Overflow)?;
        }
    }
    let real_blocks = usize::try_from(total_real_blocks).map_err(|_| CheckpointError::Limit)?;
    let padded_blocks = real_blocks.max(1).next_power_of_two();
    let padding_blocks = padded_blocks
        .checked_sub(real_blocks)
        .ok_or(CheckpointError::Invariant)?;
    let padding_block = [0_u8; 64];
    for _ in 0..padding_blocks {
        let _ = append_block_rows(
            &mut rows,
            CHAIN_PADDING_SLOT_V2,
            0,
            false,
            SHA256_IV_V2,
            &padding_block,
        )?;
    }
    let expected_rows = semantic_chain_rows(real_blocks)?;
    if rows.len() != expected_rows || !rows.len().is_power_of_two() {
        return Err(CheckpointError::Invariant);
    }
    Ok((
        ShaTraceV2 {
            role,
            public_values,
            rows,
        },
        total_real_blocks,
    ))
}

#[cfg(test)]
mod geometry_tests {
    use super::*;

    #[test]
    fn slot_slicing_halves_semantic_sha_geometry() {
        assert_eq!(semantic_chain_rows(4_096).expect("full geometry"), 262_144);
        assert_eq!(semantic_chain_rows(2_048).expect("lower geometry"), 131_072);
    }
}

fn append_jmt_sha_job(
    rows: &mut Vec<ShaRowV2>,
    lane: usize,
    record: usize,
    role: u8,
    preimage: &[u8],
) -> Result<(), CheckpointError> {
    let preimage: &[u8; 128] = preimage
        .try_into()
        .map_err(|_| CheckpointError::Canonical)?;
    let first: &[u8; 64] = preimage[..64]
        .try_into()
        .map_err(|_| CheckpointError::Canonical)?;
    let second: &[u8; 64] = preimage[64..]
        .try_into()
        .map_err(|_| CheckpointError::Canonical)?;
    let meta = Some(JmtShaJobMetaV2 {
        lane,
        record,
        role,
        block_count: 2,
    });
    let middle = append_block_rows_with_jmt_meta(rows, 0, 0, false, SHA256_IV_V2, first, meta)?;
    append_block_rows_with_jmt_meta(rows, 0, 1, true, middle, second, meta)?;
    Ok(())
}

pub(super) fn jmt_linked_public_values(
    statement: &EpochTraceChunkV2,
) -> Result<Vec<KoalaBear>, CheckpointError> {
    if statement.canonical_bytes().len() != EPOCH_CHUNK_BYTES_V2
        || statement.inputs().table != EpochAirTableV2::JmtUpdate
        || statement.inputs().replica != 0
    {
        return Err(CheckpointError::Canonical);
    }
    let values = statement
        .canonical_bytes()
        .chunks_exact(2)
        .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]])))
        .collect::<Vec<_>>();
    if values.len() != JMT_LINKED_PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

pub(super) fn jmt_linked_trace(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
) -> Result<ShaTraceV2, CheckpointError> {
    if bindings.len() != prepared.len()
        || bindings.is_empty()
        || bindings.len() > CHAIN_TRANSITION_SLOTS_V2
    {
        return Err(CheckpointError::Invariant);
    }
    let expected_records = bindings.iter().try_fold(0_u64, |total, binding| {
        total
            .checked_add(binding.inputs().jmt_record_count)
            .ok_or(CheckpointError::Overflow)
    })?;
    if statement.inputs().row_count != expected_records {
        return Err(CheckpointError::Canonical);
    }

    let mut rows = Vec::new();
    let mut observed_records = 0_u64;
    for (lane, (binding, transition)) in bindings.iter().zip(prepared).enumerate() {
        if transition.binding() != *binding {
            return Err(CheckpointError::Invariant);
        }
        let stream = transition_event_stream(&transition.material)?;
        let mut lane_records = 0_u64;
        let mut value_present = false;
        let mut operation_job = None;
        let mut value_sha_job: Option<(usize, u8, usize, usize, [u32; 8])> = None;
        for (record_index, record) in stream.jmt_micro_records().enumerate() {
            let opcode = *record.get(1).ok_or(CheckpointError::Canonical)?;
            let expected_len = [159, 52, 83, 275, 10, 6, 403, 10, 275]
                .get(usize::from(opcode).wrapping_sub(1))
                .copied()
                .ok_or(CheckpointError::Canonical)?;
            if record.len() != expected_len {
                return Err(CheckpointError::Canonical);
            }
            let record_number = record_index
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
            match opcode {
                2 => {
                    if value_sha_job.is_some() {
                        return Err(CheckpointError::Canonical);
                    }
                    value_present = record[42] == 1;
                    operation_job = Some(record_number);
                }
                3 => {
                    let job = operation_job.ok_or(CheckpointError::Canonical)?;
                    let block_index = usize::try_from(u32::from_le_bytes(
                        record[10..14]
                            .try_into()
                            .map_err(|_| CheckpointError::Canonical)?,
                    ))
                    .map_err(|_| CheckpointError::Limit)?;
                    let block_count = usize::try_from(u32::from_le_bytes(
                        record[14..18]
                            .try_into()
                            .map_err(|_| CheckpointError::Canonical)?,
                    ))
                    .map_err(|_| CheckpointError::Limit)?;
                    let role = match record[18] {
                        0 => JMT_SHA_ROLE_NEW_VALUE_V2,
                        1 => JMT_SHA_ROLE_PRIOR_VALUE_V2,
                        _ => return Err(CheckpointError::Canonical),
                    };
                    if block_count == 0 || block_index >= block_count {
                        return Err(CheckpointError::Canonical);
                    }
                    let input = if block_index == 0 {
                        if value_sha_job.is_some() {
                            return Err(CheckpointError::Canonical);
                        }
                        SHA256_IV_V2
                    } else {
                        let (active_job, active_role, active_count, next_index, state) =
                            value_sha_job.ok_or(CheckpointError::Canonical)?;
                        if active_job != job
                            || active_role != role
                            || active_count != block_count
                            || next_index != block_index
                        {
                            return Err(CheckpointError::Canonical);
                        }
                        state
                    };
                    let block: &[u8; 64] = record[19..83]
                        .try_into()
                        .map_err(|_| CheckpointError::Canonical)?;
                    let final_block = block_index + 1 == block_count;
                    let output = append_block_rows_with_jmt_meta(
                        &mut rows,
                        0,
                        u64::try_from(block_index).map_err(|_| CheckpointError::Limit)?,
                        final_block,
                        input,
                        block,
                        Some(JmtShaJobMetaV2 {
                            lane,
                            record: job,
                            role,
                            block_count,
                        }),
                    )?;
                    value_sha_job =
                        (!final_block).then_some((job, role, block_count, block_index + 1, output));
                }
                4 => {
                    if value_sha_job.is_some() {
                        return Err(CheckpointError::Canonical);
                    }
                    if record[10] == 1 {
                        append_jmt_sha_job(
                            &mut rows,
                            lane,
                            record_number,
                            JMT_SHA_ROLE_OLD_LEAF_V2,
                            &record[19..147],
                        )?;
                    }
                    if value_present {
                        append_jmt_sha_job(
                            &mut rows,
                            lane,
                            record_number,
                            JMT_SHA_ROLE_NEW_LEAF_V2,
                            &record[147..275],
                        )?;
                    }
                }
                7 => {
                    if matches!(record[12], 1 | 2) {
                        append_jmt_sha_job(
                            &mut rows,
                            lane,
                            record_number,
                            JMT_SHA_ROLE_SIBLING_V2,
                            &record[19..147],
                        )?;
                    }
                    append_jmt_sha_job(
                        &mut rows,
                        lane,
                        record_number,
                        JMT_SHA_ROLE_OLD_PARENT_V2,
                        &record[147..275],
                    )?;
                    if record[14] == 1 {
                        append_jmt_sha_job(
                            &mut rows,
                            lane,
                            record_number,
                            JMT_SHA_ROLE_NEW_PARENT_V2,
                            &record[275..403],
                        )?;
                    }
                }
                9 => {
                    if matches!(record[12], 1 | 2) {
                        append_jmt_sha_job(
                            &mut rows,
                            lane,
                            record_number,
                            JMT_SHA_ROLE_SIBLING_V2,
                            &record[19..147],
                        )?;
                    }
                    append_jmt_sha_job(
                        &mut rows,
                        lane,
                        record_number,
                        JMT_SHA_ROLE_NEW_PARENT_V2,
                        &record[147..275],
                    )?;
                }
                _ => {}
            }
            lane_records = lane_records
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
        }
        if lane_records != binding.inputs().jmt_record_count || value_sha_job.is_some() {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        observed_records = observed_records
            .checked_add(lane_records)
            .ok_or(CheckpointError::Overflow)?;
    }
    if observed_records != expected_records || !rows.len().is_multiple_of(SHA_ROWS_V2) {
        return Err(CheckpointError::Invariant);
    }

    let real_blocks = rows.len() / SHA_ROWS_V2;
    let padded_blocks = real_blocks.max(1).next_power_of_two();
    let padding_block = [0_u8; 64];
    for _ in real_blocks..padded_blocks {
        append_block_rows_with_jmt_meta(
            &mut rows,
            CHAIN_PADDING_SLOT_V2,
            0,
            false,
            SHA256_IV_V2,
            &padding_block,
            None,
        )?;
    }
    if rows.len() != padded_blocks * SHA_ROWS_V2 || !rows.len().is_power_of_two() {
        return Err(CheckpointError::Invariant);
    }
    Ok(ShaTraceV2 {
        role: ShaAirRoleV2::JmtLinked,
        public_values: jmt_linked_public_values(statement)?,
        rows,
    })
}
