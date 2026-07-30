//! Local path-machine constraints for the epoch JMT AIR.

use p3_air::AirBuilder;
use p3_field::{Field, PrimeCharacteristicRing};

use super::plonky3_epoch_jmt_air::*;
use crate::settlement::JMT_SPARSE_PLACEHOLDER_HASH_V2;

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
        let placeholder = AB::Expr::from_u64(u64::from(u16::from_le_bytes([
            JMT_SPARSE_PLACEHOLDER_HASH_V2[limb * 2],
            JMT_SPARSE_PLACEHOLDER_HASH_V2[limb * 2 + 1],
        ])));
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
                    - record_limb::<AB, D>(local, 91 + limb * 2)),
        );
        builder.assert_zero(
            update_begin.clone()
                * (digest_limb::<AB, D>(local, UPDATE_NEW_ROOT_OFFSET_V2, limb)
                    - record_limb::<AB, D>(local, 123 + limb * 2)),
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
        let placeholder = AB::Expr::from_u64(u64::from(u16::from_le_bytes([
            JMT_SPARSE_PLACEHOLDER_HASH_V2[limb * 2],
            JMT_SPARSE_PLACEHOLDER_HASH_V2[limb * 2 + 1],
        ])));
        builder.assert_zero(
            field::<AB, D>(local, SIBLING_TYPE_OFFSET_V2) * (sibling.clone() - placeholder),
        );
    }
}
