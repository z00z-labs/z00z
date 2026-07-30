//! Row framing and transcript-grammar constraints for the epoch JMT AIR.

use p3_air::AirBuilder;
use p3_field::{Field, PrimeCharacteristicRing};

use super::plonky3_epoch_jmt_air::*;

pub(super) fn eval_row_shape<AB, const D: usize>(
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
    for offset in 0..ROW_FIELDS_V2 {
        for coefficient in 1..D {
            builder.assert_zero(local[offset * D + coefficient]);
        }
    }

    let active = field::<AB, D>(local, ACTIVE_OFFSET_V2);
    let next_active = field::<AB, D>(next, ACTIVE_OFFSET_V2);
    builder.assert_bool(active.clone());
    builder.when_first_row().assert_one(active.clone());
    builder.when_first_row().assert_one(op(local, 1));
    builder
        .when_transition()
        .assert_zero(next_active.clone() * (one.clone() - active.clone()));
    builder
        .when_last_row()
        .assert_zero(active.clone() * (one.clone() - op(local, 6)));
    builder
        .when_transition()
        .assert_zero((active.clone() - next_active.clone()) * (one.clone() - op(local, 6)));

    let mut opcode_sum = AB::Expr::ZERO;
    let mut encoded_opcode = AB::Expr::ZERO;
    let mut encoded_length = AB::Expr::ZERO;
    for opcode in 1..=OPCODE_COUNT_V2 {
        let selector = op(local, opcode);
        builder.assert_bool(selector.clone());
        opcode_sum += selector.clone();
        encoded_opcode += selector.clone() * AB::Expr::from_u64(opcode as u64);
        encoded_length += selector * AB::Expr::from_u64(RECORD_LENGTHS_V2[opcode - 1] as u64);
    }
    builder.assert_eq(opcode_sum, active.clone());
    builder.assert_zero(active.clone() * (record_byte::<AB, D>(local, 0) - AB::Expr::from_u64(3)));
    builder.assert_eq(record_byte::<AB, D>(local, 1), encoded_opcode);
    builder.assert_eq(field::<AB, D>(local, RECORD_LEN_OFFSET_V2), encoded_length);
    for byte in 0..JMT_RECORD_BYTES_V2 {
        let zero_gate = (1..=OPCODE_COUNT_V2)
            .filter(|opcode| RECORD_LENGTHS_V2[*opcode - 1] <= byte)
            .map(|opcode| op(local, opcode))
            .fold(AB::Expr::ZERO, |sum, selector| sum + selector);
        builder.assert_zero(zero_gate * record_byte::<AB, D>(local, byte));
    }

    for offset in [
        VALUE_PRESENT_OFFSET_V2,
        PRIOR_PRESENT_OFFSET_V2,
        LEAF_PRESENT_OFFSET_V2,
        NEW_PARENT_ACTIVE_OFFSET_V2,
        DIRECTION_OFFSET_V2,
        COALESCED_OFFSET_V2,
        NEW_PARENT_STARTED_OFFSET_V2,
    ] {
        builder.assert_bool(field::<AB, D>(local, offset));
    }

    let role_sum = sum_fields::<AB, D>(local, ROLE_OFFSET_V2, ROLE_COUNT_V2);
    let case_sum = sum_fields::<AB, D>(local, CASE_OFFSET_V2, CASE_COUNT_V2);
    let sibling_type_sum =
        sum_fields::<AB, D>(local, SIBLING_TYPE_OFFSET_V2, SIBLING_TYPE_COUNT_V2);
    builder.assert_eq(role_sum.clone(), active.clone());
    builder.assert_bool(case_sum.clone());
    for offset in ROLE_OFFSET_V2..ROLE_OFFSET_V2 + ROLE_COUNT_V2 {
        builder.assert_bool(field::<AB, D>(local, offset));
    }
    for offset in CASE_OFFSET_V2..CASE_OFFSET_V2 + CASE_COUNT_V2 {
        builder.assert_bool(field::<AB, D>(local, offset));
    }
    for offset in SIBLING_TYPE_OFFSET_V2..SIBLING_TYPE_OFFSET_V2 + SIBLING_TYPE_COUNT_V2 {
        builder.assert_bool(field::<AB, D>(local, offset));
    }
    let path_row = op(local, 7) + op(local, 9);
    builder.assert_eq(sibling_type_sum.clone(), path_row.clone());

    let encoded_role = (0..ROLE_COUNT_V2)
        .map(|index| {
            field::<AB, D>(local, ROLE_OFFSET_V2 + index) * AB::Expr::from_u64((index + 1) as u64)
        })
        .fold(AB::Expr::ZERO, |sum, value| sum + value);
    builder.assert_zero(op(local, 1) * (record_byte::<AB, D>(local, 6) - encoded_role));
    let encoded_case = (0..CASE_COUNT_V2)
        .map(|index| {
            field::<AB, D>(local, CASE_OFFSET_V2 + index) * AB::Expr::from_u64((index + 1) as u64)
        })
        .fold(AB::Expr::ZERO, |sum, value| sum + value);
    builder.assert_zero(op(local, 4) * (record_byte::<AB, D>(local, 13) - encoded_case));
    let encoded_sibling_type = (0..SIBLING_TYPE_COUNT_V2)
        .map(|index| {
            field::<AB, D>(local, SIBLING_TYPE_OFFSET_V2 + index) * AB::Expr::from_u64(index as u64)
        })
        .fold(AB::Expr::ZERO, |sum, value| sum + value);
    builder
        .assert_zero(path_row.clone() * (record_byte::<AB, D>(local, 12) - encoded_sibling_type));

    let update_low = field::<AB, D>(local, UPDATE_INDEX_OFFSET_V2);
    let update_high = field::<AB, D>(local, UPDATE_INDEX_OFFSET_V2 + 1);
    builder.assert_zero(active.clone() * (update_low.clone() - record_limb::<AB, D>(local, 2)));
    builder.assert_zero(active.clone() * (update_high.clone() - record_limb::<AB, D>(local, 4)));
    let operation_row = active.clone() - op(local, 1) - op(local, 6);
    let operation_low = field::<AB, D>(local, OPERATION_INDEX_OFFSET_V2);
    let operation_high = field::<AB, D>(local, OPERATION_INDEX_OFFSET_V2 + 1);
    builder.assert_zero(
        operation_row.clone() * (operation_low.clone() - record_limb::<AB, D>(local, 6)),
    );
    builder.assert_zero(operation_row * (operation_high.clone() - record_limb::<AB, D>(local, 8)));

    let running = field::<AB, D>(local, RUNNING_OFFSET_V2);
    let next_running = field::<AB, D>(next, RUNNING_OFFSET_V2);
    builder.when_first_row().assert_one(running.clone());
    builder
        .when_transition()
        .assert_eq(next_running, running.clone() + next_active.clone());
    let public_rows = public[PUBLIC_ROW_COUNT_OFFSET_V2].clone()
        + public[PUBLIC_ROW_COUNT_OFFSET_V2 + 1].clone() * radix.clone();
    builder.assert_zero(public[PUBLIC_ROW_COUNT_OFFSET_V2 + 2].clone());
    builder.assert_zero(public[PUBLIC_ROW_COUNT_OFFSET_V2 + 3].clone());
    builder
        .when_last_row()
        .assert_eq(running.clone(), public_rows.clone());
    builder
        .when_transition()
        .assert_zero((active.clone() - next_active.clone()) * (running - public_rows));

    let allowed = [
        (1, vec![6]),
        (2, vec![1, 5]),
        (3, vec![2, 3]),
        (4, vec![2, 3]),
        (5, vec![8]),
        (6, vec![5]),
        (7, vec![4, 7, 9]),
        (8, vec![4, 7, 9]),
        (9, vec![4, 9]),
    ];
    for (next_opcode, predecessors) in allowed {
        let predecessor_sum = predecessors
            .into_iter()
            .map(|opcode| op(local, opcode))
            .fold(AB::Expr::ZERO, |sum, selector| sum + selector);
        builder
            .when_transition()
            .assert_zero(op(next, next_opcode) * (one.clone() - predecessor_sum));
    }

    let update_begin = op(local, 1);
    let operation_begin = op(local, 2);
    let value_row = op(local, 3);
    let proof_row = op(local, 4);
    let update_end = op(local, 6);
    let sibling_row = op(local, 7);
    let split_row = op(local, 9);
    builder.assert_zero(
        (update_begin.clone() + operation_begin.clone() + value_row.clone() + update_end.clone())
            * case_sum.clone(),
    );
    builder.assert_zero(proof_row.clone() * (case_sum.clone() - one.clone()));

    for byte in 16..19 {
        builder.assert_zero(proof_row.clone() * record_byte::<AB, D>(local, byte));
    }
    for byte in 15..19 {
        builder.assert_zero(sibling_row.clone() * record_byte::<AB, D>(local, byte));
    }
    for byte in 14..19 {
        builder.assert_zero(split_row.clone() * record_byte::<AB, D>(local, byte));
    }
}
