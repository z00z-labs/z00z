//! Local path-machine constraints for the epoch JMT AIR.

use p3_air::AirBuilder;
use p3_field::{Field, PrimeCharacteristicRing};

use super::plonky3_epoch_jmt_air::*;
use crate::settlement::{
    JMT_INTERNAL_DOMAIN_V2, JMT_LEAF_DOMAIN_V2, JMT_SPARSE_PLACEHOLDER_HASH_V2,
};

fn constrain_raw_sha_preimage<AB, const D: usize>(
    builder: &mut AB,
    row: &[AB::Var],
    gate: AB::Expr,
    start: usize,
    domain: &[u8],
    message_bytes: usize,
) where
    AB: AirBuilder,
{
    for (index, expected) in domain.iter().copied().enumerate() {
        builder.assert_zero(
            gate.clone()
                * (record_byte::<AB, D>(row, start + index)
                    - AB::Expr::from_u64(u64::from(expected))),
        );
    }
    builder.assert_zero(
        gate.clone() * (record_byte::<AB, D>(row, start + message_bytes) - AB::Expr::from_u64(128)),
    );
    for index in message_bytes + 1..120 {
        builder.assert_zero(gate.clone() * record_byte::<AB, D>(row, start + index));
    }
    let bit_length = u64::try_from(message_bytes)
        .expect("fixed JMT preimage length")
        .checked_mul(8)
        .expect("fixed JMT preimage bit length");
    for (index, expected) in bit_length.to_be_bytes().into_iter().enumerate() {
        builder.assert_zero(
            gate.clone()
                * (record_byte::<AB, D>(row, start + 120 + index)
                    - AB::Expr::from_u64(u64::from(expected))),
        );
    }
}

pub(super) fn eval_local_path<AB, const D: usize>(
    builder: &mut AB,
    local: &[AB::Var],
    public: &[AB::Expr],
) where
    AB: AirBuilder,
    AB::F: Field,
{
    let one = AB::Expr::ONE;
    let op = |row: &[AB::Var], opcode: usize| field::<AB, D>(row, OPCODE_OFFSET_V2 + opcode - 1);
    let active = field::<AB, D>(local, ACTIVE_OFFSET_V2);
    let path_row = op(local, 7) + op(local, 9);
    let update_begin = op(local, 1);
    let operation_begin = op(local, 2);
    let value_row = op(local, 3);
    let proof_row = op(local, 4);
    let operation_end = op(local, 5);
    let update_end = op(local, 6);
    let sibling_row = op(local, 7);
    let proof_end = op(local, 8);
    let split_row = op(local, 9);
    let value_present = field::<AB, D>(local, VALUE_PRESENT_OFFSET_V2);
    let prior_present = field::<AB, D>(local, PRIOR_PRESENT_OFFSET_V2);
    let leaf_present = field::<AB, D>(local, LEAF_PRESENT_OFFSET_V2);
    let new_parent_active = field::<AB, D>(local, NEW_PARENT_ACTIVE_OFFSET_V2);
    let direction = field::<AB, D>(local, DIRECTION_OFFSET_V2);
    builder.assert_zero(
        operation_begin.clone() * (value_present.clone() - record_byte::<AB, D>(local, 42)),
    );
    builder.assert_zero(
        operation_begin.clone() * (prior_present.clone() - record_byte::<AB, D>(local, 47)),
    );
    let operation_job = field::<AB, D>(local, OPERATION_JOB_OFFSET_V2);
    let expected_value_bytes = field::<AB, D>(local, EXPECTED_VALUE_BYTES_OFFSET_V2);
    let expected_prior_value_bytes = field::<AB, D>(local, EXPECTED_PRIOR_VALUE_BYTES_OFFSET_V2);
    let prior_value_block_count = field::<AB, D>(local, PRIOR_VALUE_BLOCK_COUNT_OFFSET_V2);
    let new_value_block_count = field::<AB, D>(local, NEW_VALUE_BLOCK_COUNT_OFFSET_V2);
    let running = field::<AB, D>(local, RUNNING_OFFSET_V2);
    builder.assert_zero(operation_begin.clone() * (operation_job.clone() - running));
    let value_length = record_byte::<AB, D>(local, 43)
        + record_byte::<AB, D>(local, 44) * AB::Expr::from_u64(256)
        + record_byte::<AB, D>(local, 45) * AB::Expr::from_u64(65_536)
        + record_byte::<AB, D>(local, 46) * AB::Expr::from_u64(16_777_216);
    let prior_length = record_byte::<AB, D>(local, 48)
        + record_byte::<AB, D>(local, 49) * AB::Expr::from_u64(256)
        + record_byte::<AB, D>(local, 50) * AB::Expr::from_u64(65_536)
        + record_byte::<AB, D>(local, 51) * AB::Expr::from_u64(16_777_216);
    builder.assert_zero(operation_begin.clone() * (expected_value_bytes.clone() - value_length));
    builder
        .assert_zero(operation_begin.clone() * (expected_prior_value_bytes.clone() - prior_length));
    for (low, middle, high, top) in [(43, 44, 45, 46), (48, 49, 50, 51)] {
        let high_byte = record_byte::<AB, D>(local, high);
        builder.assert_zero(
            operation_begin.clone() * high_byte.clone() * (high_byte.clone() - one.clone()),
        );
        builder.assert_zero(operation_begin.clone() * record_byte::<AB, D>(local, top));
        builder.assert_zero(
            operation_begin.clone() * high_byte.clone() * record_byte::<AB, D>(local, low),
        );
        builder
            .assert_zero(operation_begin.clone() * high_byte * record_byte::<AB, D>(local, middle));
    }
    builder.assert_zero(
        operation_begin.clone()
            * (one.clone() - value_present.clone())
            * expected_value_bytes.clone(),
    );
    builder.assert_zero(
        operation_begin.clone()
            * (one.clone() - prior_present.clone())
            * expected_prior_value_bytes.clone(),
    );
    builder.assert_zero(operation_begin.clone() * prior_value_block_count.clone());
    builder.assert_zero(operation_begin.clone() * new_value_block_count.clone());

    let value_kind = record_byte::<AB, D>(local, 18);
    builder
        .assert_zero(value_row.clone() * value_kind.clone() * (value_kind.clone() - one.clone()));
    builder.assert_zero(value_row.clone() * record_byte::<AB, D>(local, 12));
    builder.assert_zero(value_row.clone() * record_byte::<AB, D>(local, 13));
    builder.assert_zero(value_row.clone() * record_byte::<AB, D>(local, 16));
    builder.assert_zero(value_row.clone() * record_byte::<AB, D>(local, 17));
    let value_block_index = record_limb::<AB, D>(local, 10);
    let value_block_count = record_limb::<AB, D>(local, 14);
    let selected_value_bytes = value_kind.clone() * expected_prior_value_bytes.clone()
        + (one.clone() - value_kind.clone()) * expected_value_bytes.clone();
    let selected_value_present = value_kind.clone() * prior_present.clone()
        + (one.clone() - value_kind.clone()) * value_present.clone();
    let selected_value_block_count = value_kind.clone() * prior_value_block_count.clone()
        + (one.clone() - value_kind.clone()) * new_value_block_count.clone();
    builder.assert_zero(value_row.clone() * (one.clone() - selected_value_present));
    builder.assert_zero(
        value_row.clone() * (selected_value_block_count.clone() - value_block_count.clone()),
    );

    let padding_started = field::<AB, D>(local, VALUE_PADDING_STARTED_OFFSET_V2);
    let padding_start = field::<AB, D>(local, VALUE_PADDING_START_OFFSET_V2);
    builder.assert_bool(padding_started.clone());
    builder.assert_bool(padding_start.clone());
    builder.assert_zero((one.clone() - value_row.clone()) * padding_started.clone());
    builder.assert_zero((one.clone() - value_row.clone()) * padding_start.clone());
    builder.assert_zero(padding_start.clone() * (one.clone() - padding_started.clone()));

    let mut remainder = AB::Expr::ZERO;
    let mut remainder_sum = AB::Expr::ZERO;
    let mut long_padding = AB::Expr::ZERO;
    let mut remainder_selectors = Vec::with_capacity(VALUE_REMAINDER_COUNT_V2);
    for candidate in 0..VALUE_REMAINDER_COUNT_V2 {
        let selector = field::<AB, D>(local, VALUE_REMAINDER_OFFSET_V2 + candidate);
        builder.assert_bool(selector.clone());
        remainder_sum += selector.clone();
        remainder += selector.clone() * AB::Expr::from_usize(candidate);
        if candidate >= 56 {
            long_padding += selector.clone();
        }
        remainder_selectors.push(selector);
    }
    builder.assert_eq(remainder_sum, value_row.clone());
    builder.assert_zero(
        value_row.clone()
            * (selected_value_block_count.clone() * AB::Expr::from_u64(64)
                - selected_value_bytes.clone()
                + remainder.clone()
                - AB::Expr::from_u64(64) * (one.clone() + long_padding.clone())),
    );
    builder.assert_zero(
        padding_start.clone()
            * (value_block_index.clone() * AB::Expr::from_u64(64) + remainder.clone()
                - selected_value_bytes.clone()),
    );

    let bit_length = selected_value_bytes.clone() * AB::Expr::from_u64(8);
    let encoded_bit_length = record_byte::<AB, D>(local, 19 + 61) * AB::Expr::from_u64(65_536)
        + record_byte::<AB, D>(local, 19 + 62) * AB::Expr::from_u64(256)
        + record_byte::<AB, D>(local, 19 + 63);
    for (candidate, selector) in remainder_selectors.iter().enumerate() {
        let gate = padding_start.clone() * selector.clone();
        builder.assert_zero(
            gate.clone() * (record_byte::<AB, D>(local, 19 + candidate) - AB::Expr::from_u64(128)),
        );
        if candidate <= 55 {
            for byte in candidate + 1..61 {
                builder.assert_zero(gate.clone() * record_byte::<AB, D>(local, 19 + byte));
            }
            builder.assert_zero(gate * (encoded_bit_length.clone() - bit_length.clone()));
        } else {
            for byte in candidate + 1..64 {
                builder.assert_zero(gate.clone() * record_byte::<AB, D>(local, 19 + byte));
            }
        }
    }
    let trailing_padding = (padding_started.clone() - padding_start.clone()) * long_padding.clone();
    for byte in 0..61 {
        builder.assert_zero(trailing_padding.clone() * record_byte::<AB, D>(local, 19 + byte));
    }
    builder.assert_zero(trailing_padding * (encoded_bit_length - bit_length));

    builder
        .assert_zero(proof_row.clone() * (leaf_present.clone() - record_byte::<AB, D>(local, 10)));
    builder.assert_zero(
        sibling_row.clone() * (new_parent_active.clone() - record_byte::<AB, D>(local, 14)),
    );
    builder.assert_zero(path_row.clone() * (direction.clone() - record_byte::<AB, D>(local, 13)));
    builder.assert_zero(
        (active.clone() - sibling_row.clone() - split_row.clone()) * direction.clone(),
    );
    builder.assert_zero((active.clone() - sibling_row.clone()) * new_parent_active.clone());

    let expected_siblings = field::<AB, D>(local, EXPECTED_SIBLINGS_OFFSET_V2);
    let consumed_siblings = field::<AB, D>(local, CONSUMED_SIBLINGS_OFFSET_V2);
    let expected_split = field::<AB, D>(local, EXPECTED_SPLIT_OFFSET_V2);
    let consumed_split = field::<AB, D>(local, CONSUMED_SPLIT_OFFSET_V2);
    builder.assert_zero(
        proof_row.clone() * (expected_siblings.clone() - record_limb::<AB, D>(local, 11)),
    );
    builder.assert_zero(
        proof_row.clone() * (expected_split.clone() - record_limb::<AB, D>(local, 14)),
    );
    builder.assert_zero(proof_row.clone() * consumed_siblings.clone());
    builder.assert_zero(proof_row.clone() * consumed_split.clone());
    for selector in [update_begin.clone(), operation_begin.clone()] {
        builder.assert_zero(selector.clone() * expected_siblings.clone());
        builder.assert_zero(selector.clone() * consumed_siblings.clone());
        builder.assert_zero(selector.clone() * expected_split.clone());
        builder.assert_zero(selector * consumed_split.clone());
    }
    builder
        .assert_zero(proof_end.clone() * (expected_siblings.clone() - consumed_siblings.clone()));
    builder.assert_zero(proof_end.clone() * (expected_split.clone() - consumed_split.clone()));

    let expected_ops_low = field::<AB, D>(local, EXPECTED_OPERATIONS_OFFSET_V2);
    let expected_ops_high = field::<AB, D>(local, EXPECTED_OPERATIONS_OFFSET_V2 + 1);
    let completed_ops_low = field::<AB, D>(local, COMPLETED_OPERATIONS_OFFSET_V2);
    let completed_ops_high = field::<AB, D>(local, COMPLETED_OPERATIONS_OFFSET_V2 + 1);
    builder.assert_zero(
        update_begin.clone() * (expected_ops_low.clone() - record_limb::<AB, D>(local, 155)),
    );
    builder.assert_zero(
        update_begin.clone() * (expected_ops_high.clone() - record_limb::<AB, D>(local, 157)),
    );
    builder.assert_zero(update_begin.clone() * completed_ops_low.clone());
    builder.assert_zero(update_begin.clone() * completed_ops_high.clone());
    builder
        .assert_zero(update_end.clone() * (expected_ops_low.clone() - completed_ops_low.clone()));
    builder
        .assert_zero(update_end.clone() * (expected_ops_high.clone() - completed_ops_high.clone()));

    let aux = field::<AB, D>(local, AUX_INDEX_OFFSET_V2);
    builder.assert_zero(update_begin.clone() * aux.clone());
    builder.assert_zero(value_row.clone() * (aux.clone() - record_limb::<AB, D>(local, 10)));
    builder.assert_zero(value_row.clone() * record_byte::<AB, D>(local, 12));
    builder.assert_zero(value_row.clone() * record_byte::<AB, D>(local, 13));
    builder.assert_zero(path_row.clone() * (aux.clone() - record_limb::<AB, D>(local, 10)));
    builder.assert_bool(operation_end.clone() * aux.clone());
    builder.assert_bool(update_end.clone() * aux.clone());

    for byte in 0..32 {
        builder.assert_zero(
            operation_begin.clone()
                * (field::<AB, D>(local, KEY_OFFSET_V2 + byte)
                    - record_byte::<AB, D>(local, 10 + byte)),
        );
    }
    builder.assert_zero(operation_begin.clone() * leaf_present.clone());
    builder.assert_zero(operation_begin.clone() * field::<AB, D>(local, COALESCED_OFFSET_V2));
    builder
        .assert_zero(operation_begin.clone() * field::<AB, D>(local, NEW_PARENT_STARTED_OFFSET_V2));

    let case_start = (0..3)
        .map(|index| field::<AB, D>(local, CASE_OFFSET_V2 + index))
        .fold(AB::Expr::ZERO, |sum, value| sum + value);
    builder.assert_zero(
        proof_row.clone() * (field::<AB, D>(local, NEW_PARENT_STARTED_OFFSET_V2) - case_start),
    );
    builder.assert_zero(proof_row.clone() * field::<AB, D>(local, COALESCED_OFFSET_V2));
    builder.assert_zero(
        proof_row.clone() * prior_present.clone() * (one.clone() - leaf_present.clone()),
    );

    for limb in 0..16 {
        let placeholder = digest_constant_limb::<AB>(&JMT_SPARSE_PLACEHOLDER_HASH_V2, limb);
        let old_leaf = digest_limb::<AB, D>(local, OLD_LEAF_DIGEST_OFFSET_V2, limb);
        let new_leaf = digest_limb::<AB, D>(local, NEW_LEAF_DIGEST_OFFSET_V2, limb);
        let old_current = digest_limb::<AB, D>(local, OLD_CURRENT_OFFSET_V2, limb);
        let new_current = digest_limb::<AB, D>(local, NEW_CURRENT_OFFSET_V2, limb);
        builder.assert_zero(
            proof_row.clone()
                * (old_current
                    - leaf_present.clone() * old_leaf
                    - (one.clone() - leaf_present.clone()) * placeholder.clone()),
        );
        builder.assert_zero(
            proof_row.clone()
                * (new_current
                    - value_present.clone() * new_leaf
                    - (one.clone() - value_present.clone()) * placeholder.clone()),
        );
        builder.assert_zero(
            split_row.clone()
                * (digest_limb::<AB, D>(local, NEW_CURRENT_OFFSET_V2, limb)
                    - digest_limb::<AB, D>(local, NEW_PARENT_DIGEST_OFFSET_V2, limb)),
        );
        builder.assert_zero(
            sibling_row.clone()
                * (digest_limb::<AB, D>(local, OLD_CURRENT_OFFSET_V2, limb)
                    - digest_limb::<AB, D>(local, OLD_PARENT_DIGEST_OFFSET_V2, limb)),
        );
        builder.assert_zero(
            proof_end.clone()
                * (digest_limb::<AB, D>(local, OLD_CURRENT_OFFSET_V2, limb)
                    - digest_limb::<AB, D>(local, UPDATE_CURRENT_OFFSET_V2, limb)),
        );
        builder.assert_zero(
            operation_end.clone()
                * (digest_limb::<AB, D>(local, UPDATE_CURRENT_OFFSET_V2, limb)
                    - digest_limb::<AB, D>(local, NEW_CURRENT_OFFSET_V2, limb)),
        );
        builder.assert_zero(
            update_end.clone()
                * (digest_limb::<AB, D>(local, UPDATE_CURRENT_OFFSET_V2, limb)
                    - digest_limb::<AB, D>(local, UPDATE_NEW_ROOT_OFFSET_V2, limb)),
        );
        builder.assert_zero(
            update_begin.clone()
                * (digest_limb::<AB, D>(local, UPDATE_CURRENT_OFFSET_V2, limb)
                    - record_digest_limb::<AB, D>(local, 91, limb)),
        );
        builder.assert_zero(
            update_begin.clone()
                * (digest_limb::<AB, D>(local, UPDATE_NEW_ROOT_OFFSET_V2, limb)
                    - record_digest_limb::<AB, D>(local, 123, limb)),
        );
        let definition = field::<AB, D>(local, ROLE_OFFSET_V2);
        builder.assert_zero(
            update_begin.clone()
                * definition.clone()
                * (digest_limb::<AB, D>(local, UPDATE_CURRENT_OFFSET_V2, limb)
                    - public[PUBLIC_INPUT_ROOT_OFFSET_V2 + limb].clone()),
        );
        builder.assert_zero(
            update_begin.clone()
                * definition
                * (digest_limb::<AB, D>(local, UPDATE_NEW_ROOT_OFFSET_V2, limb)
                    - public[PUBLIC_OUTPUT_ROOT_OFFSET_V2 + limb].clone()),
        );
    }

    for byte in 0..128 {
        builder.assert_zero(
            proof_row.clone()
                * (one.clone() - leaf_present.clone())
                * record_byte::<AB, D>(local, 19 + byte),
        );
        builder.assert_zero(
            proof_row.clone()
                * (one.clone() - value_present.clone())
                * record_byte::<AB, D>(local, 147 + byte),
        );
        builder.assert_zero(
            field::<AB, D>(local, SIBLING_TYPE_OFFSET_V2) * record_byte::<AB, D>(local, 19 + byte),
        );
        builder.assert_zero(
            sibling_row.clone()
                * (one.clone() - new_parent_active.clone())
                * record_byte::<AB, D>(local, 275 + byte),
        );
    }

    let internal_sibling = field::<AB, D>(local, SIBLING_TYPE_OFFSET_V2 + 1);
    let leaf_sibling = field::<AB, D>(local, SIBLING_TYPE_OFFSET_V2 + 2);
    constrain_raw_sha_preimage::<AB, D>(
        builder,
        local,
        proof_row.clone() * leaf_present.clone(),
        19,
        JMT_LEAF_DOMAIN_V2,
        77,
    );
    constrain_raw_sha_preimage::<AB, D>(
        builder,
        local,
        proof_row.clone() * value_present.clone(),
        147,
        JMT_LEAF_DOMAIN_V2,
        77,
    );
    constrain_raw_sha_preimage::<AB, D>(
        builder,
        local,
        (sibling_row.clone() + split_row.clone()) * leaf_sibling,
        19,
        JMT_LEAF_DOMAIN_V2,
        77,
    );
    constrain_raw_sha_preimage::<AB, D>(
        builder,
        local,
        (sibling_row.clone() + split_row.clone()) * internal_sibling,
        19,
        JMT_INTERNAL_DOMAIN_V2,
        80,
    );
    constrain_raw_sha_preimage::<AB, D>(
        builder,
        local,
        sibling_row.clone(),
        147,
        JMT_INTERNAL_DOMAIN_V2,
        80,
    );
    constrain_raw_sha_preimage::<AB, D>(
        builder,
        local,
        sibling_row.clone() * new_parent_active.clone(),
        275,
        JMT_INTERNAL_DOMAIN_V2,
        80,
    );
    constrain_raw_sha_preimage::<AB, D>(
        builder,
        local,
        split_row.clone(),
        147,
        JMT_INTERNAL_DOMAIN_V2,
        80,
    );

    for byte in 0..32 {
        let key = field::<AB, D>(local, KEY_OFFSET_V2 + byte);
        let old_leaf_key = record_byte::<AB, D>(local, 19 + 13 + byte);
        let new_leaf_key = record_byte::<AB, D>(local, 147 + 13 + byte);
        builder.assert_zero(
            proof_row.clone() * prior_present.clone() * (old_leaf_key.clone() - key.clone()),
        );
        builder
            .assert_zero(proof_row.clone() * value_present.clone() * (new_leaf_key - key.clone()));
        builder.assert_zero(
            proof_row.clone()
                * (field::<AB, D>(local, PATH_KEY_OFFSET_V2 + byte)
                    - leaf_present.clone() * old_leaf_key
                    - (one.clone() - leaf_present.clone()) * key),
        );
    }

    let mut byte_position = AB::Expr::ZERO;
    let mut bit_position = AB::Expr::ZERO;
    let mut selected_key_byte = AB::Expr::ZERO;
    let mut selected_path_byte = AB::Expr::ZERO;
    let mut byte_selector_sum = AB::Expr::ZERO;
    let mut bit_selector_sum = AB::Expr::ZERO;
    for index in 0..BYTE_POSITION_COUNT_V2 {
        let selector = field::<AB, D>(local, BYTE_POSITION_OFFSET_V2 + index);
        builder.assert_bool(selector.clone());
        byte_selector_sum += selector.clone();
        byte_position += selector.clone() * AB::Expr::from_u64(index as u64);
        selected_key_byte += selector.clone() * field::<AB, D>(local, KEY_OFFSET_V2 + index);
        selected_path_byte += selector * field::<AB, D>(local, PATH_KEY_OFFSET_V2 + index);
    }
    for index in 0..BIT_POSITION_COUNT_V2 {
        let selector = field::<AB, D>(local, BIT_POSITION_OFFSET_V2 + index);
        builder.assert_bool(selector.clone());
        bit_selector_sum += selector.clone();
        bit_position += selector * AB::Expr::from_u64(index as u64);
    }
    builder.assert_eq(byte_selector_sum, path_row.clone());
    builder.assert_eq(bit_selector_sum, path_row.clone());
    let mut reconstructed_byte = AB::Expr::ZERO;
    let mut selected_direction = AB::Expr::ZERO;
    for bit in 0..SELECTED_BITS_COUNT_V2 {
        let value = field::<AB, D>(local, SELECTED_BITS_OFFSET_V2 + bit);
        builder.assert_bool(value.clone());
        reconstructed_byte += value.clone() * AB::Expr::from_u64(1_u64 << bit);
        selected_direction += value
            * field::<AB, D>(
                local,
                BIT_POSITION_OFFSET_V2 + (SELECTED_BITS_COUNT_V2 - 1 - bit),
            );
    }
    builder.assert_zero(split_row.clone() * (reconstructed_byte.clone() - selected_key_byte));
    builder.assert_zero(sibling_row.clone() * (reconstructed_byte - selected_path_byte));
    builder.assert_zero(path_row.clone() * (direction.clone() - selected_direction));
    builder.assert_zero(
        split_row.clone()
            * (byte_position.clone() * AB::Expr::from_u64(8)
                + bit_position.clone()
                + consumed_split.clone()
                - expected_split.clone()
                - expected_siblings.clone()),
    );
    builder.assert_zero(
        sibling_row.clone()
            * (byte_position * AB::Expr::from_u64(8) + bit_position + consumed_siblings.clone()
                - expected_siblings.clone()),
    );

    for limb in 0..16 {
        let sibling = digest_limb::<AB, D>(local, SIBLING_DIGEST_OFFSET_V2, limb);
        let placeholder = digest_constant_limb::<AB>(&JMT_SPARSE_PLACEHOLDER_HASH_V2, limb);
        builder.assert_zero(
            field::<AB, D>(local, SIBLING_TYPE_OFFSET_V2) * (sibling.clone() - placeholder),
        );
    }
}
