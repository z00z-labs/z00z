//! Cross-row state-transition constraints for the epoch JMT AIR.

use p3_air::AirBuilder;
use p3_field::{Field, PrimeCharacteristicRing};

use super::plonky3_epoch_jmt_air::*;

pub(super) fn eval_transition_state<AB, const D: usize>(
    builder: &mut AB,
    local: &[AB::Var],
    next: &[AB::Var],
    public: &[AB::Expr],
) where
    AB: AirBuilder,
    AB::F: Field,
{
    let one = AB::Expr::ONE;
    let radix = AB::Expr::from_u64(65_536);
    let op = |row: &[AB::Var], opcode: usize| field::<AB, D>(row, OPCODE_OFFSET_V2 + opcode - 1);
    let active = field::<AB, D>(local, ACTIVE_OFFSET_V2);
    let next_active = field::<AB, D>(next, ACTIVE_OFFSET_V2);
    let update_begin = op(local, 1);
    let operation_begin = op(local, 2);
    let value_row = op(local, 3);
    let operation_end = op(local, 5);
    let update_low = field::<AB, D>(local, UPDATE_INDEX_OFFSET_V2);
    let update_high = field::<AB, D>(local, UPDATE_INDEX_OFFSET_V2 + 1);
    let operation_low = field::<AB, D>(local, OPERATION_INDEX_OFFSET_V2);
    let operation_high = field::<AB, D>(local, OPERATION_INDEX_OFFSET_V2 + 1);
    let aux = field::<AB, D>(local, AUX_INDEX_OFFSET_V2);
    let completed_ops_low = field::<AB, D>(local, COMPLETED_OPERATIONS_OFFSET_V2);
    let completed_ops_high = field::<AB, D>(local, COMPLETED_OPERATIONS_OFFSET_V2 + 1);
    let consumed_siblings = field::<AB, D>(local, CONSUMED_SIBLINGS_OFFSET_V2);
    let consumed_split = field::<AB, D>(local, CONSUMED_SPLIT_OFFSET_V2);
    let next_update_begin = op(next, 1);
    let next_operation_begin = op(next, 2);
    let next_value = op(next, 3);
    let next_proof = op(next, 4);
    let next_operation_end = op(next, 5);
    let next_sibling = op(next, 7);
    let next_proof_end = op(next, 8);
    let next_split = op(next, 9);

    let carry_update = next_active.clone() - next_update_begin.clone();
    let carry_update_current = carry_update.clone() - next_operation_end.clone();
    for limb in 0..16 {
        builder.when_transition().assert_zero(
            carry_update_current.clone()
                * (digest_limb::<AB, D>(next, UPDATE_CURRENT_OFFSET_V2, limb)
                    - digest_limb::<AB, D>(local, UPDATE_CURRENT_OFFSET_V2, limb)),
        );
    }
    for offset in [UPDATE_NEW_ROOT_OFFSET_V2] {
        for limb in 0..16 {
            builder.when_transition().assert_zero(
                carry_update.clone()
                    * (digest_limb::<AB, D>(next, offset, limb)
                        - digest_limb::<AB, D>(local, offset, limb)),
            );
        }
    }
    for index in 0..ROLE_COUNT_V2 {
        builder.when_transition().assert_zero(
            carry_update.clone()
                * (field::<AB, D>(next, ROLE_OFFSET_V2 + index)
                    - field::<AB, D>(local, ROLE_OFFSET_V2 + index)),
        );
    }
    for byte in 0..TREE_DEFINITION_BYTES_V2 {
        builder.when_transition().assert_zero(
            carry_update.clone()
                * (field::<AB, D>(next, TREE_DEFINITION_OFFSET_V2 + byte)
                    - field::<AB, D>(local, TREE_DEFINITION_OFFSET_V2 + byte)),
        );
    }
    for byte in 0..TREE_SERIAL_BYTES_V2 {
        builder.when_transition().assert_zero(
            carry_update.clone()
                * (field::<AB, D>(next, TREE_SERIAL_OFFSET_V2 + byte)
                    - field::<AB, D>(local, TREE_SERIAL_OFFSET_V2 + byte)),
        );
    }
    for offset in [
        EXPECTED_OPERATIONS_OFFSET_V2,
        EXPECTED_OPERATIONS_OFFSET_V2 + 1,
    ] {
        builder.when_transition().assert_zero(
            carry_update.clone() * (field::<AB, D>(next, offset) - field::<AB, D>(local, offset)),
        );
    }

    let carry_completed = carry_update.clone() - next_operation_end.clone();
    for limb in 0..2 {
        builder.when_transition().assert_zero(
            carry_completed.clone()
                * (field::<AB, D>(next, COMPLETED_OPERATIONS_OFFSET_V2 + limb)
                    - field::<AB, D>(local, COMPLETED_OPERATIONS_OFFSET_V2 + limb)),
        );
    }
    builder.when_transition().assert_zero(
        next_operation_end.clone()
            * (field::<AB, D>(next, COMPLETED_OPERATIONS_OFFSET_V2)
                - completed_ops_low.clone()
                - one.clone()
                + field::<AB, D>(next, AUX_INDEX_OFFSET_V2) * radix.clone()),
    );
    builder.when_transition().assert_zero(
        next_operation_end.clone()
            * (field::<AB, D>(next, COMPLETED_OPERATIONS_OFFSET_V2 + 1)
                - completed_ops_high.clone()
                - field::<AB, D>(next, AUX_INDEX_OFFSET_V2)),
    );

    let carry_operation = next_value.clone()
        + next_proof.clone()
        + next_split.clone()
        + next_sibling.clone()
        + next_proof_end.clone()
        + next_operation_end.clone();
    for offset in [UPDATE_INDEX_OFFSET_V2, UPDATE_INDEX_OFFSET_V2 + 1] {
        builder.when_transition().assert_zero(
            (next_active.clone() - next_update_begin.clone())
                * (field::<AB, D>(next, offset) - field::<AB, D>(local, offset)),
        );
    }
    for offset in [
        OPERATION_INDEX_OFFSET_V2,
        OPERATION_INDEX_OFFSET_V2 + 1,
        VALUE_PRESENT_OFFSET_V2,
        PRIOR_PRESENT_OFFSET_V2,
    ] {
        builder.when_transition().assert_zero(
            carry_operation.clone()
                * (field::<AB, D>(next, offset) - field::<AB, D>(local, offset)),
        );
    }
    for byte in 0..32 {
        builder.when_transition().assert_zero(
            carry_operation.clone()
                * (field::<AB, D>(next, KEY_OFFSET_V2 + byte)
                    - field::<AB, D>(local, KEY_OFFSET_V2 + byte)),
        );
    }
    for offset in [
        OPERATION_JOB_OFFSET_V2,
        EXPECTED_VALUE_BYTES_OFFSET_V2,
        EXPECTED_PRIOR_VALUE_BYTES_OFFSET_V2,
    ] {
        builder.when_transition().assert_zero(
            carry_operation.clone()
                * (field::<AB, D>(next, offset) - field::<AB, D>(local, offset)),
        );
    }
    let next_value_kind = record_byte::<AB, D>(next, 18);
    let next_prior_count_write = next_value.clone() * next_value_kind.clone();
    let next_new_count_write = next_value.clone() * (one.clone() - next_value_kind.clone());
    builder.when_transition().assert_zero(
        (carry_operation.clone() - next_prior_count_write)
            * (field::<AB, D>(next, PRIOR_VALUE_BLOCK_COUNT_OFFSET_V2)
                - field::<AB, D>(local, PRIOR_VALUE_BLOCK_COUNT_OFFSET_V2)),
    );
    builder.when_transition().assert_zero(
        (carry_operation.clone() - next_new_count_write)
            * (field::<AB, D>(next, NEW_VALUE_BLOCK_COUNT_OFFSET_V2)
                - field::<AB, D>(local, NEW_VALUE_BLOCK_COUNT_OFFSET_V2)),
    );

    let prior_present = field::<AB, D>(local, PRIOR_PRESENT_OFFSET_V2);
    let value_present = field::<AB, D>(local, VALUE_PRESENT_OFFSET_V2);
    let first_value_required =
        prior_present.clone() + (one.clone() - prior_present.clone()) * value_present.clone();
    builder
        .when_transition()
        .assert_zero(operation_begin.clone() * (next_value.clone() - first_value_required));
    builder.when_transition().assert_zero(
        operation_begin.clone()
            * (next_proof.clone()
                - (one.clone() - prior_present.clone()) * (one.clone() - value_present.clone())),
    );
    let next_value_index = record_limb::<AB, D>(next, 10);
    let next_value_count = record_limb::<AB, D>(next, 14);
    builder
        .when_transition()
        .assert_zero(operation_begin.clone() * next_value.clone() * next_value_index.clone());
    builder.when_transition().assert_zero(
        operation_begin.clone()
            * next_value.clone()
            * (next_value_kind.clone() - prior_present.clone()),
    );
    builder.when_transition().assert_zero(
        operation_begin.clone()
            * next_value.clone()
            * (field::<AB, D>(next, VALUE_PADDING_STARTED_OFFSET_V2)
                - field::<AB, D>(next, VALUE_PADDING_START_OFFSET_V2)),
    );

    let value_kind = record_byte::<AB, D>(local, 18);
    let value_index = record_limb::<AB, D>(local, 10);
    let value_count = record_limb::<AB, D>(local, 14);
    let padding_started = field::<AB, D>(local, VALUE_PADDING_STARTED_OFFSET_V2);
    let next_padding_started = field::<AB, D>(next, VALUE_PADDING_STARTED_OFFSET_V2);
    let next_padding_start = field::<AB, D>(next, VALUE_PADDING_START_OFFSET_V2);
    builder
        .when_transition()
        .assert_zero(value_row.clone() * (next_value.clone() + next_proof.clone() - one.clone()));
    builder.when_transition().assert_zero(
        value_row.clone()
            * next_value.clone()
            * (one.clone() - value_kind.clone())
            * next_value_kind.clone(),
    );
    let switch_kind = next_value.clone() * (value_kind.clone() - next_value_kind.clone());
    let same_kind = next_value.clone() - switch_kind.clone();
    builder.when_transition().assert_zero(
        value_row.clone()
            * same_kind.clone()
            * (next_value_index.clone() - value_index.clone() - one.clone()),
    );
    builder.when_transition().assert_zero(
        value_row.clone() * same_kind.clone() * (next_value_count.clone() - value_count.clone()),
    );
    builder.when_transition().assert_zero(
        value_row.clone()
            * same_kind
            * (next_padding_started.clone() - padding_started.clone() - next_padding_start.clone()),
    );
    builder.when_transition().assert_zero(
        value_row.clone()
            * switch_kind.clone()
            * (value_index.clone() + one.clone() - value_count.clone()),
    );
    builder
        .when_transition()
        .assert_zero(value_row.clone() * switch_kind.clone() * next_value_index);
    builder.when_transition().assert_zero(
        value_row.clone() * switch_kind.clone() * (one.clone() - value_present.clone()),
    );
    builder.when_transition().assert_zero(
        value_row.clone() * switch_kind.clone() * (one.clone() - padding_started.clone()),
    );
    builder
        .when_transition()
        .assert_zero(value_row.clone() * switch_kind * (next_padding_started - next_padding_start));
    builder.when_transition().assert_zero(
        value_row.clone() * next_proof.clone() * (value_index + one.clone() - value_count),
    );
    builder
        .when_transition()
        .assert_zero(value_row.clone() * next_proof.clone() * (one.clone() - padding_started));
    builder
        .when_transition()
        .assert_zero(value_row * next_proof * value_kind * value_present);

    builder.when_transition().assert_zero(
        next_operation_begin.clone()
            * update_begin.clone()
            * field::<AB, D>(next, OPERATION_INDEX_OFFSET_V2),
    );
    builder.when_transition().assert_zero(
        next_operation_begin.clone()
            * update_begin
            * field::<AB, D>(next, OPERATION_INDEX_OFFSET_V2 + 1),
    );
    builder.when_transition().assert_zero(
        next_operation_begin.clone()
            * operation_end.clone()
            * (field::<AB, D>(next, OPERATION_INDEX_OFFSET_V2)
                - operation_low.clone()
                - one.clone()
                + aux.clone() * radix.clone()),
    );
    builder.when_transition().assert_zero(
        next_operation_begin.clone()
            * operation_end
            * (field::<AB, D>(next, OPERATION_INDEX_OFFSET_V2 + 1)
                - operation_high.clone()
                - aux.clone()),
    );
    builder.when_transition().assert_zero(
        next_update_begin.clone()
            * (field::<AB, D>(next, UPDATE_INDEX_OFFSET_V2) - update_low.clone() - one.clone()
                + aux.clone() * radix.clone()),
    );
    builder.when_transition().assert_zero(
        next_update_begin
            * (field::<AB, D>(next, UPDATE_INDEX_OFFSET_V2 + 1)
                - update_high.clone()
                - aux.clone()),
    );

    let carry_proof_state = next_split.clone()
        + next_sibling.clone()
        + next_proof_end.clone()
        + next_operation_end.clone();
    for offset in [
        EXPECTED_SIBLINGS_OFFSET_V2,
        EXPECTED_SPLIT_OFFSET_V2,
        LEAF_PRESENT_OFFSET_V2,
    ] {
        builder.when_transition().assert_zero(
            carry_proof_state.clone()
                * (field::<AB, D>(next, offset) - field::<AB, D>(local, offset)),
        );
    }
    for offset in CASE_OFFSET_V2..CASE_OFFSET_V2 + CASE_COUNT_V2 {
        builder.when_transition().assert_zero(
            carry_proof_state.clone()
                * (field::<AB, D>(next, offset) - field::<AB, D>(local, offset)),
        );
    }
    for byte in 0..32 {
        builder.when_transition().assert_zero(
            carry_proof_state.clone()
                * (field::<AB, D>(next, PATH_KEY_OFFSET_V2 + byte)
                    - field::<AB, D>(local, PATH_KEY_OFFSET_V2 + byte)),
        );
    }
    builder.when_transition().assert_zero(
        next_split.clone()
            * (field::<AB, D>(next, CONSUMED_SPLIT_OFFSET_V2)
                - consumed_split.clone()
                - one.clone()),
    );
    builder.when_transition().assert_zero(
        next_split.clone()
            * (field::<AB, D>(next, CONSUMED_SIBLINGS_OFFSET_V2) - consumed_siblings.clone()),
    );
    builder.when_transition().assert_zero(
        next_sibling.clone()
            * (field::<AB, D>(next, CONSUMED_SIBLINGS_OFFSET_V2)
                - consumed_siblings.clone()
                - one.clone()),
    );
    builder.when_transition().assert_zero(
        next_sibling.clone()
            * (field::<AB, D>(next, CONSUMED_SPLIT_OFFSET_V2) - consumed_split.clone()),
    );
    for selector in [next_proof_end.clone(), next_operation_end.clone()] {
        builder.when_transition().assert_zero(
            selector.clone()
                * (field::<AB, D>(next, CONSUMED_SIBLINGS_OFFSET_V2) - consumed_siblings.clone()),
        );
        builder.when_transition().assert_zero(
            selector * (field::<AB, D>(next, CONSUMED_SPLIT_OFFSET_V2) - consumed_split.clone()),
        );
    }

    for limb in 0..16 {
        let next_direction = field::<AB, D>(next, DIRECTION_OFFSET_V2);
        let next_sibling_digest = digest_limb::<AB, D>(next, SIBLING_DIGEST_OFFSET_V2, limb);
        let split_left = record_digest_limb::<AB, D>(next, 147 + 16, limb);
        let split_right = record_digest_limb::<AB, D>(next, 147 + 48, limb);
        builder.when_transition().assert_zero(
            next_split.clone()
                * (split_left
                    - (one.clone() - next_direction.clone())
                        * digest_limb::<AB, D>(local, NEW_CURRENT_OFFSET_V2, limb)
                    - next_direction.clone() * next_sibling_digest.clone()),
        );
        builder.when_transition().assert_zero(
            next_split.clone()
                * (split_right
                    - (one.clone() - next_direction.clone()) * next_sibling_digest.clone()
                    - next_direction.clone()
                        * digest_limb::<AB, D>(local, NEW_CURRENT_OFFSET_V2, limb)),
        );
        let old_left = record_digest_limb::<AB, D>(next, 147 + 16, limb);
        let old_right = record_digest_limb::<AB, D>(next, 147 + 48, limb);
        builder.when_transition().assert_zero(
            next_sibling.clone()
                * (old_left
                    - (one.clone() - next_direction.clone())
                        * digest_limb::<AB, D>(local, OLD_CURRENT_OFFSET_V2, limb)
                    - next_direction.clone() * next_sibling_digest.clone()),
        );
        builder.when_transition().assert_zero(
            next_sibling.clone()
                * (old_right
                    - (one.clone() - next_direction.clone()) * next_sibling_digest.clone()
                    - next_direction.clone()
                        * digest_limb::<AB, D>(local, OLD_CURRENT_OFFSET_V2, limb)),
        );
        let next_active_parent = field::<AB, D>(next, NEW_PARENT_ACTIVE_OFFSET_V2);
        let new_left = record_digest_limb::<AB, D>(next, 275 + 16, limb);
        let new_right = record_digest_limb::<AB, D>(next, 275 + 48, limb);
        builder.when_transition().assert_zero(
            next_sibling.clone()
                * next_active_parent.clone()
                * (new_left
                    - (one.clone() - next_direction.clone())
                        * digest_limb::<AB, D>(local, NEW_CURRENT_OFFSET_V2, limb)
                    - next_direction.clone() * next_sibling_digest.clone()),
        );
        builder.when_transition().assert_zero(
            next_sibling.clone()
                * next_active_parent.clone()
                * (new_right
                    - (one.clone() - next_direction.clone()) * next_sibling_digest
                    - next_direction * digest_limb::<AB, D>(local, NEW_CURRENT_OFFSET_V2, limb)),
        );
        builder.when_transition().assert_zero(
            next_split.clone()
                * (digest_limb::<AB, D>(next, OLD_CURRENT_OFFSET_V2, limb)
                    - digest_limb::<AB, D>(local, OLD_CURRENT_OFFSET_V2, limb)),
        );
        builder.when_transition().assert_zero(
            next_sibling.clone()
                * (digest_limb::<AB, D>(next, OLD_CURRENT_OFFSET_V2, limb)
                    - digest_limb::<AB, D>(next, OLD_PARENT_DIGEST_OFFSET_V2, limb)),
        );
        for selector in [next_proof_end.clone(), next_operation_end.clone()] {
            builder.when_transition().assert_zero(
                selector
                    * (digest_limb::<AB, D>(next, OLD_CURRENT_OFFSET_V2, limb)
                        - digest_limb::<AB, D>(local, OLD_CURRENT_OFFSET_V2, limb)),
            );
        }
        builder.when_transition().assert_zero(
            next_split.clone()
                * (digest_limb::<AB, D>(next, NEW_CURRENT_OFFSET_V2, limb)
                    - digest_limb::<AB, D>(next, NEW_PARENT_DIGEST_OFFSET_V2, limb)),
        );
        builder.when_transition().assert_zero(
            next_sibling.clone()
                * next_active_parent.clone()
                * (digest_limb::<AB, D>(next, NEW_CURRENT_OFFSET_V2, limb)
                    - digest_limb::<AB, D>(next, NEW_PARENT_DIGEST_OFFSET_V2, limb)),
        );
        for selector in [next_proof_end.clone(), next_operation_end.clone()] {
            builder.when_transition().assert_zero(
                selector
                    * (digest_limb::<AB, D>(next, NEW_CURRENT_OFFSET_V2, limb)
                        - digest_limb::<AB, D>(local, NEW_CURRENT_OFFSET_V2, limb)),
            );
        }
    }

    let case_six = field::<AB, D>(local, CASE_OFFSET_V2 + 5);
    let next_sibling_type_leaf = field::<AB, D>(next, SIBLING_TYPE_OFFSET_V2 + 2);
    let local_coalesced = field::<AB, D>(local, COALESCED_OFFSET_V2);
    let next_coalesced = field::<AB, D>(next, COALESCED_OFFSET_V2);
    let coalesced_delta = next_coalesced.clone() - local_coalesced.clone();
    builder.when_transition().assert_zero(
        next_sibling.clone() * coalesced_delta.clone() * (coalesced_delta.clone() - one.clone()),
    );
    builder
        .when_transition()
        .assert_zero(next_sibling.clone() * coalesced_delta.clone() * (one.clone() - case_six));
    builder.when_transition().assert_zero(
        next_sibling.clone()
            * coalesced_delta.clone()
            * (one.clone() - next_sibling_type_leaf.clone()),
    );
    builder.when_transition().assert_zero(
        next_sibling.clone()
            * coalesced_delta.clone()
            * field::<AB, D>(local, NEW_PARENT_STARTED_OFFSET_V2),
    );
    builder.when_transition().assert_zero(
        next_sibling.clone()
            * coalesced_delta.clone()
            * field::<AB, D>(next, NEW_PARENT_ACTIVE_OFFSET_V2),
    );
    for limb in 0..16 {
        builder.when_transition().assert_zero(
            next_sibling.clone()
                * coalesced_delta.clone()
                * (digest_limb::<AB, D>(next, NEW_CURRENT_OFFSET_V2, limb)
                    - digest_limb::<AB, D>(next, SIBLING_DIGEST_OFFSET_V2, limb)),
        );
        builder.when_transition().assert_zero(
            next_sibling.clone()
                * (one.clone()
                    - field::<AB, D>(next, NEW_PARENT_ACTIVE_OFFSET_V2)
                    - coalesced_delta.clone())
                * (digest_limb::<AB, D>(next, NEW_CURRENT_OFFSET_V2, limb)
                    - digest_limb::<AB, D>(local, NEW_CURRENT_OFFSET_V2, limb)),
        );
    }
    let carry_path_flags = next_split.clone() + next_proof_end.clone() + next_operation_end.clone();
    for offset in [COALESCED_OFFSET_V2, NEW_PARENT_STARTED_OFFSET_V2] {
        builder.when_transition().assert_zero(
            carry_path_flags.clone()
                * (field::<AB, D>(next, offset) - field::<AB, D>(local, offset)),
        );
    }
    builder.when_transition().assert_zero(
        next_sibling.clone()
            * (field::<AB, D>(next, NEW_PARENT_STARTED_OFFSET_V2)
                - field::<AB, D>(local, NEW_PARENT_STARTED_OFFSET_V2)
                - field::<AB, D>(next, NEW_PARENT_ACTIVE_OFFSET_V2)
                    * (one.clone() - field::<AB, D>(local, NEW_PARENT_STARTED_OFFSET_V2))),
    );

    let header_update_low = public[PUBLIC_HEADER_OFFSET_V2 + 35].clone()
        + public[PUBLIC_HEADER_OFFSET_V2 + 36].clone() * AB::Expr::from_u64(256);
    let header_update_high = public[PUBLIC_HEADER_OFFSET_V2 + 37].clone()
        + public[PUBLIC_HEADER_OFFSET_V2 + 38].clone() * AB::Expr::from_u64(256);
    let final_update_low = update_low + one.clone() - header_update_low - aux.clone() * radix;
    let final_update_high = update_high + aux - header_update_high;
    builder
        .when_last_row()
        .assert_zero(active.clone() * final_update_low.clone());
    builder
        .when_last_row()
        .assert_zero(active.clone() * final_update_high.clone());
    builder
        .when_transition()
        .assert_zero((active.clone() - next_active.clone()) * final_update_low);
    builder
        .when_transition()
        .assert_zero((active.clone() - next_active) * final_update_high);

    let inactive = one - active;
    for offset in 0..ROW_FIELDS_V2 {
        if offset != RUNNING_OFFSET_V2 {
            builder.assert_zero(inactive.clone() * field::<AB, D>(local, offset));
        }
    }
}
