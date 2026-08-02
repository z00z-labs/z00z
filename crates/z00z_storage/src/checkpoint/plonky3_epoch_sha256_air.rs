//! Direct SHA-256 compression AIR for bounded epoch hash work.
//!
//! The canonical round relation is shared by one-block smoke proofs and a
//! bounded multi-block chunk chain. State and message schedule limbs advance
//! row by row; selective bit decompositions constrain every Boolean SHA
//! operation without materializing a full bit circuit per epoch transition.

use p3_air::{Air, AirBuilder, WindowAccess};
use p3_field::{Field, PrimeCharacteristicRing};
use p3_koala_bear::KoalaBear;
use p3_lookup::{Count, InteractionBuilder};
use z00z_plonky3_circuit_prover::batch_stark_prover::BatchAir;

use super::plonky3_epoch_sha256_columns::{
    base_state_limb, bits_limb, carry, field, schedule_bit, schedule_limb, state_bit, state_limb,
    SemanticShaJobKindV2, ShaAirRoleV2, ShaAirV2, A_CARRY_OFFSET_V2, BASE_STATE_OFFSET_V2,
    BLOCK_INDEX_OFFSET_V2, CHAIN_ACTIVE_OFFSET_V2, CHAIN_BLOCK_COUNT_OFFSET_V2,
    CHAIN_DIGEST_OFFSET_V2, CHAIN_PADDING_SLOT_V2, CHAIN_SELECTOR_COUNT_V2,
    CHAIN_SLICE_START_OFFSET_V2, CHAIN_TRANSITION_SLOTS_V2, E_CARRY_OFFSET_V2,
    JMT_BLOCK_COUNT_OFFSET_V2, JMT_LANE_OFFSET_V2, JMT_RECORD_OFFSET_V2, JMT_ROLE_OFFSET_V2,
    JMT_SHA_BLOCK_PAIR_BUS_V2, JMT_SHA_DIGEST_BUS_V2, OUTPUT_CARRY_OFFSET_V2, RADIX_V2,
    RECURSIVE_SHA_INPUT_LIMBS_V2, ROW_FIELDS_V2, SCHEDULE_CARRY_OFFSET_V2,
    SCHEDULE_LIMBS_OFFSET_V2, SELECTOR_OFFSET_V2, SEMANTIC_SHA_BLOCK_PAIR_BUS_V2,
    SEMANTIC_SHA_DIGEST_PAIR_BUS_V2, SHA_BLOCK_PAIR_BUS_V2, SHA_COMMON_ROW_FIELDS_V2, SHA_ROWS_V2,
    STANDALONE_BLOCK_OFFSET_V2, STANDALONE_INPUT_STATE_OFFSET_V2,
    STANDALONE_OUTPUT_STATE_OFFSET_V2, STATE_BIT_WORDS_V2, T1_BITS_OFFSET_V2, T1_CARRY_OFFSET_V2,
    T2_BITS_OFFSET_V2, T2_CARRY_OFFSET_V2, TRANSITION_FINAL_OFFSET_V2,
    TRANSITION_SELECTOR_OFFSET_V2,
};
use super::{Plonky3StarkConfigV2, SHA256_ROUND_CONSTANTS_V2};

impl<AB, const D: usize> Air<AB> for ShaAirV2<AB::F, D>
where
    AB: AirBuilder + InteractionBuilder,
    AB::F: Field,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.current_slice();
        let next = main.next_slice();
        let public = builder
            .public_values()
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<AB::Expr>>();
        let one = AB::Expr::ONE;
        let radix = AB::Expr::from_u64(RADIX_V2);

        for offset in 0..ROW_FIELDS_V2 {
            for coefficient in 1..D {
                builder.assert_zero(local[offset * D + coefficient]);
            }
        }

        for (slot, &word) in STATE_BIT_WORDS_V2.iter().enumerate() {
            for bit in 0..32 {
                builder.assert_bool(state_bit::<AB, D>(local, slot, bit));
            }
            for limb in 0..2 {
                let mut reconstructed = AB::Expr::ZERO;
                for bit in 0..16 {
                    reconstructed += state_bit::<AB, D>(local, slot, limb * 16 + bit)
                        * AB::Expr::from_u64(1_u64 << bit);
                }
                builder.assert_eq(state_limb::<AB, D>(local, word, limb), reconstructed);
            }
        }
        for slot in 0..2 {
            let word = if slot == 0 { 1 } else { 14 };
            for bit in 0..32 {
                builder.assert_bool(schedule_bit::<AB, D>(local, slot, bit));
            }
            for limb in 0..2 {
                let mut reconstructed = AB::Expr::ZERO;
                for bit in 0..16 {
                    reconstructed += schedule_bit::<AB, D>(local, slot, limb * 16 + bit)
                        * AB::Expr::from_u64(1_u64 << bit);
                }
                builder.assert_eq(schedule_limb::<AB, D>(local, word, limb), reconstructed);
            }
        }
        for offset in T1_BITS_OFFSET_V2..SHA_COMMON_ROW_FIELDS_V2 {
            builder.assert_bool(field::<AB, D>(local, offset));
        }

        let round_selectors = (0..SHA_ROWS_V2)
            .map(|round| field::<AB, D>(local, SELECTOR_OFFSET_V2 + round))
            .collect::<Vec<_>>();
        let next_round_selectors = (0..SHA_ROWS_V2)
            .map(|round| field::<AB, D>(next, SELECTOR_OFFSET_V2 + round))
            .collect::<Vec<_>>();
        let mut selector_sum = AB::Expr::ZERO;
        for selector in &round_selectors {
            builder.assert_bool(selector.clone());
            selector_sum += selector.clone();
        }
        builder.assert_one(selector_sum);
        let round_first = round_selectors[0].clone();
        let round_last = round_selectors[SHA_ROWS_V2 - 1].clone();
        let normal_round = one.clone() - round_last.clone();

        let transition_selectors = (0..CHAIN_SELECTOR_COUNT_V2)
            .map(|slot| field::<AB, D>(local, TRANSITION_SELECTOR_OFFSET_V2 + slot))
            .collect::<Vec<_>>();
        let next_transition_selectors = (0..CHAIN_SELECTOR_COUNT_V2)
            .map(|slot| field::<AB, D>(next, TRANSITION_SELECTOR_OFFSET_V2 + slot))
            .collect::<Vec<_>>();
        let mut transition_selector_sum = AB::Expr::ZERO;
        for selector in &transition_selectors {
            builder.assert_bool(selector.clone());
            transition_selector_sum += selector.clone();
        }
        builder.assert_one(transition_selector_sum);

        let block_index = field::<AB, D>(local, BLOCK_INDEX_OFFSET_V2);
        let next_block_index = field::<AB, D>(next, BLOCK_INDEX_OFFSET_V2);
        let transition_final = field::<AB, D>(local, TRANSITION_FINAL_OFFSET_V2);
        builder.assert_bool(transition_final.clone());
        builder.assert_zero(transition_final.clone() * normal_round.clone());

        builder.when_first_row().assert_one(round_first.clone());
        builder.when_last_row().assert_one(round_last.clone());
        {
            let mut transition = builder.when_transition();
            transition.assert_eq(next_round_selectors[0].clone(), round_last.clone());
            for round in 1..SHA_ROWS_V2 {
                transition.assert_eq(
                    next_round_selectors[round].clone(),
                    round_selectors[round - 1].clone(),
                );
            }
        }

        match self.role {
            ShaAirRoleV2::Standalone => {
                for offset in JMT_LANE_OFFSET_V2..ROW_FIELDS_V2 {
                    builder.assert_zero(field::<AB, D>(local, offset));
                }
                builder.assert_one(transition_selectors[0].clone());
                for selector in &transition_selectors[1..] {
                    builder.assert_zero(selector.clone());
                }
                builder.assert_zero(block_index.clone());
                builder.assert_eq(transition_final.clone(), round_last.clone());
                for word in 0..8 {
                    for limb in 0..2 {
                        builder.assert_eq(
                            base_state_limb::<AB, D>(local, word, limb),
                            public[STANDALONE_INPUT_STATE_OFFSET_V2 + word * 2 + limb].clone(),
                        );
                    }
                }
                let mut first = builder.when_first_row();
                for word in 0..8 {
                    for limb in 0..2 {
                        first.assert_eq(
                            state_limb::<AB, D>(local, word, limb),
                            public[STANDALONE_INPUT_STATE_OFFSET_V2 + word * 2 + limb].clone(),
                        );
                    }
                }
                for word in 0..16 {
                    for limb in 0..2 {
                        first.assert_eq(
                            schedule_limb::<AB, D>(local, word, limb),
                            public[STANDALONE_BLOCK_OFFSET_V2 + word * 2 + limb].clone(),
                        );
                    }
                }
            }
            ShaAirRoleV2::Chain => {
                for offset in JMT_LANE_OFFSET_V2..ROW_FIELDS_V2 {
                    builder.assert_zero(field::<AB, D>(local, offset));
                }
                let active = (0..CHAIN_TRANSITION_SLOTS_V2)
                    .map(|slot| public[CHAIN_ACTIVE_OFFSET_V2 + slot].clone())
                    .collect::<Vec<_>>();
                for flag in &active {
                    builder.assert_bool(flag.clone());
                }
                builder.assert_one(active[0].clone());
                for slot in 1..CHAIN_TRANSITION_SLOTS_V2 {
                    builder.assert_zero(
                        active[slot].clone() * (one.clone() - active[slot - 1].clone()),
                    );
                }
                for slot in 0..CHAIN_TRANSITION_SLOTS_V2 {
                    builder.assert_zero(
                        transition_selectors[slot].clone() * (one.clone() - active[slot].clone()),
                    );
                    let expected_blocks = public[CHAIN_BLOCK_COUNT_OFFSET_V2 + slot].clone();
                    builder.assert_zero(
                        transition_final.clone()
                            * transition_selectors[slot].clone()
                            * (block_index.clone() + one.clone() - expected_blocks),
                    );
                }
                builder.assert_zero(
                    transition_final.clone() * transition_selectors[CHAIN_PADDING_SLOT_V2].clone(),
                );

                let transition_index = transition_selectors[..CHAIN_TRANSITION_SLOTS_V2]
                    .iter()
                    .enumerate()
                    .fold(AB::Expr::ZERO, |sum, (slot, selector)| {
                        sum + selector.clone() * AB::Expr::from_usize(slot)
                    });
                let real_block = one.clone() - transition_selectors[CHAIN_PADDING_SLOT_V2].clone();
                for round in 0..16 {
                    let pair_gate = real_block.clone() * round_selectors[round].clone();
                    builder.push_interaction(
                        SHA_BLOCK_PAIR_BUS_V2,
                        vec![
                            transition_index.clone(),
                            block_index.clone(),
                            AB::Expr::from_usize(round * 2),
                            schedule_limb::<AB, D>(local, 0, 1),
                        ],
                        Count::bounded(pair_gate.clone(), 1),
                    );
                    builder.push_interaction(
                        SHA_BLOCK_PAIR_BUS_V2,
                        vec![
                            transition_index.clone(),
                            block_index.clone(),
                            AB::Expr::from_usize(round * 2 + 1),
                            schedule_limb::<AB, D>(local, 0, 0),
                        ],
                        Count::bounded(pair_gate, 1),
                    );
                }

                {
                    let mut first = builder.when_first_row();
                    first.assert_one(transition_selectors[0].clone());
                    first.assert_zero(block_index.clone());
                    for word in 0..8 {
                        let iv = z00z_crypto::SHA256_IV_V2[word];
                        first.assert_eq(
                            base_state_limb::<AB, D>(local, word, 0),
                            AB::Expr::from_u64(u64::from(iv & 0xffff)),
                        );
                        first.assert_eq(
                            base_state_limb::<AB, D>(local, word, 1),
                            AB::Expr::from_u64(u64::from(iv >> 16)),
                        );
                    }
                }
                for word in 0..8 {
                    for limb in 0..2 {
                        builder.when(round_first.clone()).assert_eq(
                            state_limb::<AB, D>(local, word, limb),
                            base_state_limb::<AB, D>(local, word, limb),
                        );
                    }
                }

                let mut transition = builder.when_transition();
                transition.assert_zero(
                    normal_round.clone() * (next_block_index.clone() - block_index.clone()),
                );
                transition.assert_zero(
                    round_last.clone()
                        * (next_block_index.clone()
                            - (one.clone() - transition_final.clone())
                                * (block_index.clone() + one.clone())),
                );
                for slot in 0..CHAIN_SELECTOR_COUNT_V2 {
                    transition.assert_zero(
                        normal_round.clone()
                            * (next_transition_selectors[slot].clone()
                                - transition_selectors[slot].clone()),
                    );
                }
                for slot in 0..CHAIN_TRANSITION_SLOTS_V2 {
                    let incoming = if slot == 0 {
                        AB::Expr::ZERO
                    } else {
                        transition_final.clone()
                            * transition_selectors[slot - 1].clone()
                            * active[slot].clone()
                    };
                    let stay = (one.clone() - transition_final.clone())
                        * transition_selectors[slot].clone();
                    transition.assert_zero(
                        round_last.clone()
                            * (next_transition_selectors[slot].clone() - stay - incoming),
                    );
                }
                let padding_incoming =
                    (0..CHAIN_TRANSITION_SLOTS_V2).fold(AB::Expr::ZERO, |sum, slot| {
                        let next_active = active.get(slot + 1).cloned().unwrap_or(AB::Expr::ZERO);
                        sum + transition_final.clone()
                            * transition_selectors[slot].clone()
                            * (one.clone() - next_active)
                    });
                transition.assert_zero(
                    round_last.clone()
                        * (next_transition_selectors[CHAIN_PADDING_SLOT_V2].clone()
                            - transition_selectors[CHAIN_PADDING_SLOT_V2].clone()
                            - padding_incoming),
                );
                for word in 0..8 {
                    for limb in 0..2 {
                        transition.assert_zero(
                            normal_round.clone()
                                * (base_state_limb::<AB, D>(next, word, limb)
                                    - base_state_limb::<AB, D>(local, word, limb)),
                        );
                    }
                }

                let real_selector =
                    one.clone() - transition_selectors[CHAIN_PADDING_SLOT_V2].clone();
                builder
                    .when_last_row()
                    .assert_zero(real_selector * (one.clone() - transition_final.clone()));
            }
            ShaAirRoleV2::SemanticTransitionChain | ShaAirRoleV2::SemanticUniquenessChain => {
                let semantic_kind_count = SemanticShaJobKindV2::COUNT;
                let semantic_selectors = &transition_selectors[..semantic_kind_count];
                let real = semantic_selectors
                    .iter()
                    .cloned()
                    .fold(AB::Expr::ZERO, |sum, selector| sum + selector);
                let padding = transition_selectors[CHAIN_PADDING_SLOT_V2].clone();
                for selector in &transition_selectors[semantic_kind_count..CHAIN_PADDING_SLOT_V2] {
                    builder.assert_zero(selector.clone());
                }
                let semantic_kind = semantic_selectors.iter().enumerate().fold(
                    AB::Expr::ZERO,
                    |sum, (index, selector)| {
                        sum + selector.clone() * AB::Expr::from_usize(index + 1)
                    },
                );
                let lane = field::<AB, D>(local, JMT_LANE_OFFSET_V2);
                let global_lane = lane.clone() + public[CHAIN_SLICE_START_OFFSET_V2].clone();
                let record = field::<AB, D>(local, JMT_RECORD_OFFSET_V2);
                let role = field::<AB, D>(local, JMT_ROLE_OFFSET_V2);
                let block_count = field::<AB, D>(local, JMT_BLOCK_COUNT_OFFSET_V2);
                let next_lane = field::<AB, D>(next, JMT_LANE_OFFSET_V2);
                let next_record = field::<AB, D>(next, JMT_RECORD_OFFSET_V2);
                let next_role = field::<AB, D>(next, JMT_ROLE_OFFSET_V2);
                let next_block_count = field::<AB, D>(next, JMT_BLOCK_COUNT_OFFSET_V2);

                builder.assert_zero(real.clone() * (role.clone() - semantic_kind));
                builder.assert_zero(padding.clone() * block_index.clone());
                builder.assert_zero(padding.clone() * lane.clone());
                builder.assert_zero(padding.clone() * record.clone());
                builder.assert_zero(padding.clone() * role.clone());
                builder.assert_zero(padding.clone() * block_count.clone());
                builder.assert_zero(transition_final.clone() * padding.clone());
                builder.assert_zero(
                    transition_final.clone()
                        * (block_index.clone() + one.clone() - block_count.clone()),
                );
                {
                    let mut first = builder.when_first_row();
                    first.assert_one(
                        transition_selectors[SemanticShaJobKindV2::EventVector.index()].clone(),
                    );
                    first.assert_zero(block_index.clone());
                    first.assert_zero(lane.clone());
                    first.assert_zero(record.clone());
                }
                builder
                    .when_last_row()
                    .assert_zero(real.clone() * (one.clone() - transition_final.clone()));

                for round in 0..16 {
                    let pair_gate = real.clone() * round_selectors[round].clone();
                    for (pair, value) in [
                        (round * 2, schedule_limb::<AB, D>(local, 0, 1)),
                        (round * 2 + 1, schedule_limb::<AB, D>(local, 0, 0)),
                    ] {
                        builder.push_interaction(
                            SEMANTIC_SHA_BLOCK_PAIR_BUS_V2,
                            vec![
                                global_lane.clone(),
                                role.clone(),
                                record.clone(),
                                block_count.clone(),
                                block_index.clone(),
                                AB::Expr::from_usize(pair),
                                value,
                            ],
                            Count::bounded(pair_gate.clone(), 1),
                        );
                    }
                }

                {
                    let mut first = builder.when_first_row();
                    for word in 0..8 {
                        let iv = z00z_crypto::SHA256_IV_V2[word];
                        first.assert_eq(
                            base_state_limb::<AB, D>(local, word, 0),
                            AB::Expr::from_u64(u64::from(iv & 0xffff)),
                        );
                        first.assert_eq(
                            base_state_limb::<AB, D>(local, word, 1),
                            AB::Expr::from_u64(u64::from(iv >> 16)),
                        );
                    }
                }
                for word in 0..8 {
                    for limb in 0..2 {
                        builder.when(round_first.clone()).assert_eq(
                            state_limb::<AB, D>(local, word, limb),
                            base_state_limb::<AB, D>(local, word, limb),
                        );
                    }
                }

                let mut transition = builder.when_transition();
                transition.assert_zero(
                    normal_round.clone() * (next_block_index.clone() - block_index.clone()),
                );
                transition
                    .assert_zero(round_last.clone() * padding.clone() * next_block_index.clone());
                transition.assert_zero(
                    round_last.clone()
                        * real.clone()
                        * (one.clone() - transition_final.clone())
                        * (next_block_index.clone() - block_index.clone() - one.clone()),
                );
                transition.assert_zero(
                    round_last.clone()
                        * real.clone()
                        * transition_final.clone()
                        * next_block_index.clone(),
                );
                for slot in 0..CHAIN_SELECTOR_COUNT_V2 {
                    transition.assert_zero(
                        normal_round.clone()
                            * (next_transition_selectors[slot].clone()
                                - transition_selectors[slot].clone()),
                    );
                }
                transition.assert_zero(
                    round_last.clone()
                        * padding.clone()
                        * (next_transition_selectors[CHAIN_PADDING_SLOT_V2].clone() - one.clone()),
                );
                for slot in 0..semantic_kind_count {
                    transition.assert_zero(
                        round_last.clone()
                            * real.clone()
                            * (one.clone() - transition_final.clone())
                            * (next_transition_selectors[slot].clone()
                                - transition_selectors[slot].clone()),
                    );
                }
                for (local_meta, next_meta) in [
                    (lane, next_lane),
                    (record, next_record),
                    (role, next_role),
                    (block_count, next_block_count),
                ] {
                    transition.assert_zero(
                        normal_round.clone() * (next_meta.clone() - local_meta.clone()),
                    );
                    transition.assert_zero(
                        round_last.clone()
                            * real.clone()
                            * (one.clone() - transition_final.clone())
                            * (next_meta - local_meta),
                    );
                }
                for word in 0..8 {
                    for limb in 0..2 {
                        transition.assert_zero(
                            normal_round.clone()
                                * (base_state_limb::<AB, D>(next, word, limb)
                                    - base_state_limb::<AB, D>(local, word, limb)),
                        );
                    }
                }
            }
            ShaAirRoleV2::JmtLinked => {
                let real = transition_selectors[0].clone();
                let padding = transition_selectors[CHAIN_PADDING_SLOT_V2].clone();
                for selector in &transition_selectors[1..CHAIN_PADDING_SLOT_V2] {
                    builder.assert_zero(selector.clone());
                }
                let lane = field::<AB, D>(local, JMT_LANE_OFFSET_V2);
                let record = field::<AB, D>(local, JMT_RECORD_OFFSET_V2);
                let role = field::<AB, D>(local, JMT_ROLE_OFFSET_V2);
                let block_count = field::<AB, D>(local, JMT_BLOCK_COUNT_OFFSET_V2);
                let next_lane = field::<AB, D>(next, JMT_LANE_OFFSET_V2);
                let next_record = field::<AB, D>(next, JMT_RECORD_OFFSET_V2);
                let next_role = field::<AB, D>(next, JMT_ROLE_OFFSET_V2);
                let next_block_count = field::<AB, D>(next, JMT_BLOCK_COUNT_OFFSET_V2);

                builder.assert_zero(padding.clone() * block_index.clone());
                builder.assert_zero(padding.clone() * lane.clone());
                builder.assert_zero(padding.clone() * record.clone());
                builder.assert_zero(padding.clone() * role.clone());
                builder.assert_zero(padding.clone() * block_count.clone());
                builder.assert_zero(transition_final.clone() * padding.clone());
                builder.assert_zero(
                    transition_final.clone()
                        * (block_index.clone() + one.clone() - block_count.clone()),
                );
                builder
                    .when_first_row()
                    .assert_zero(real.clone() * block_index.clone());
                builder
                    .when_last_row()
                    .assert_zero(real.clone() * (one.clone() - transition_final.clone()));

                for round in 0..16 {
                    let pair_gate = real.clone() * round_selectors[round].clone();
                    builder.push_interaction(
                        JMT_SHA_BLOCK_PAIR_BUS_V2,
                        vec![
                            lane.clone(),
                            record.clone(),
                            role.clone(),
                            block_count.clone(),
                            block_index.clone(),
                            AB::Expr::from_usize(round * 2),
                            schedule_limb::<AB, D>(local, 0, 1),
                        ],
                        Count::bounded(pair_gate.clone(), 1),
                    );
                    builder.push_interaction(
                        JMT_SHA_BLOCK_PAIR_BUS_V2,
                        vec![
                            lane.clone(),
                            record.clone(),
                            role.clone(),
                            block_count.clone(),
                            block_index.clone(),
                            AB::Expr::from_usize(round * 2 + 1),
                            schedule_limb::<AB, D>(local, 0, 0),
                        ],
                        Count::bounded(pair_gate, 1),
                    );
                }

                {
                    let mut first = builder.when_first_row();
                    for word in 0..8 {
                        let iv = z00z_crypto::SHA256_IV_V2[word];
                        first.assert_eq(
                            base_state_limb::<AB, D>(local, word, 0),
                            AB::Expr::from_u64(u64::from(iv & 0xffff)),
                        );
                        first.assert_eq(
                            base_state_limb::<AB, D>(local, word, 1),
                            AB::Expr::from_u64(u64::from(iv >> 16)),
                        );
                    }
                }
                for word in 0..8 {
                    for limb in 0..2 {
                        builder.when(round_first.clone()).assert_eq(
                            state_limb::<AB, D>(local, word, limb),
                            base_state_limb::<AB, D>(local, word, limb),
                        );
                    }
                }

                let mut transition = builder.when_transition();
                transition.assert_zero(
                    normal_round.clone() * (next_block_index.clone() - block_index.clone()),
                );
                transition
                    .assert_zero(round_last.clone() * padding.clone() * next_block_index.clone());
                transition.assert_zero(
                    round_last.clone()
                        * real.clone()
                        * (one.clone() - transition_final.clone())
                        * (next_block_index.clone() - block_index.clone() - one.clone()),
                );
                transition.assert_zero(
                    round_last.clone()
                        * real.clone()
                        * transition_final.clone()
                        * next_block_index.clone(),
                );
                for slot in 0..CHAIN_SELECTOR_COUNT_V2 {
                    transition.assert_zero(
                        normal_round.clone()
                            * (next_transition_selectors[slot].clone()
                                - transition_selectors[slot].clone()),
                    );
                }
                transition.assert_zero(
                    round_last.clone()
                        * padding.clone()
                        * (next_transition_selectors[CHAIN_PADDING_SLOT_V2].clone() - one.clone()),
                );
                transition.assert_zero(
                    round_last.clone()
                        * real.clone()
                        * (one.clone() - transition_final.clone())
                        * (next_transition_selectors[0].clone() - one.clone()),
                );
                for (local_meta, next_meta) in [
                    (lane, next_lane),
                    (record, next_record),
                    (role, next_role),
                    (block_count, next_block_count),
                ] {
                    transition.assert_zero(
                        normal_round.clone() * (next_meta.clone() - local_meta.clone()),
                    );
                    transition.assert_zero(
                        round_last.clone()
                            * real.clone()
                            * (one.clone() - transition_final.clone())
                            * (next_meta - local_meta),
                    );
                }
                for word in 0..8 {
                    for limb in 0..2 {
                        transition.assert_zero(
                            normal_round.clone()
                                * (base_state_limb::<AB, D>(next, word, limb)
                                    - base_state_limb::<AB, D>(local, word, limb)),
                        );
                    }
                }
            }
            ShaAirRoleV2::RecursiveCompression => {
                for offset in JMT_LANE_OFFSET_V2..ROW_FIELDS_V2 {
                    builder.assert_zero(field::<AB, D>(local, offset));
                }
                builder.assert_one(transition_selectors[0].clone());
                for selector in &transition_selectors[1..] {
                    builder.assert_zero(selector.clone());
                }
                builder.assert_zero(block_index.clone());
                builder.assert_eq(transition_final.clone(), round_last.clone());
                for word in 0..8 {
                    for limb in 0..2 {
                        builder.when(round_first.clone()).assert_eq(
                            state_limb::<AB, D>(local, word, limb),
                            base_state_limb::<AB, D>(local, word, limb),
                        );
                    }
                }
                let mut transition = builder.when_transition();
                for word in 0..8 {
                    for limb in 0..2 {
                        transition.assert_zero(
                            normal_round.clone()
                                * (base_state_limb::<AB, D>(next, word, limb)
                                    - base_state_limb::<AB, D>(local, word, limb)),
                        );
                    }
                }
                drop(transition);

                let preprocessed = builder.preprocessed().clone();
                let prep = preprocessed.current_slice();
                let active: AB::Expr = prep[0].into();
                builder.assert_bool(active.clone());
                let input_gate = active * round_first.clone();
                for word in 0..8 {
                    for limb in 0..2 {
                        let input = word * 2 + limb;
                        let mut fields = Vec::with_capacity(D + 1);
                        fields.push(prep[1 + input].into());
                        for coefficient in 0..D {
                            fields.push(
                                local[(BASE_STATE_OFFSET_V2 + input) * D + coefficient].into(),
                            );
                        }
                        builder.push_interaction(
                            "WitnessChecks",
                            fields,
                            Count::bounded(-input_gate.clone(), 1),
                        );
                    }
                }
                for word in 0..16 {
                    for limb in 0..2 {
                        let input = 16 + word * 2 + limb;
                        let mut fields = Vec::with_capacity(D + 1);
                        fields.push(prep[1 + input].into());
                        for coefficient in 0..D {
                            fields.push(
                                local[(SCHEDULE_LIMBS_OFFSET_V2 + word * 2 + limb) * D
                                    + coefficient]
                                    .into(),
                            );
                        }
                        builder.push_interaction(
                            "WitnessChecks",
                            fields,
                            Count::bounded(-input_gate.clone(), 1),
                        );
                    }
                }
                debug_assert_eq!(RECURSIVE_SHA_INPUT_LIMBS_V2, 48);
            }
        }

        let xor_three = |a: AB::Expr, b: AB::Expr, c: AB::Expr| {
            a.clone() + b.clone() + c.clone()
                - AB::Expr::from_u64(2)
                    * (a.clone() * b.clone() + a.clone() * c.clone() + b.clone() * c.clone())
                + AB::Expr::from_u64(4) * a * b * c
        };
        let mut sigma_0 = [AB::Expr::ZERO, AB::Expr::ZERO];
        let mut sigma_1 = [AB::Expr::ZERO, AB::Expr::ZERO];
        let mut small_0 = [AB::Expr::ZERO, AB::Expr::ZERO];
        let mut small_1 = [AB::Expr::ZERO, AB::Expr::ZERO];
        let mut choose = [AB::Expr::ZERO, AB::Expr::ZERO];
        let mut majority = [AB::Expr::ZERO, AB::Expr::ZERO];
        for limb in 0..2 {
            for bit in 0..16 {
                let output = limb * 16 + bit;
                let weight = AB::Expr::from_u64(1_u64 << bit);
                sigma_0[limb] += xor_three(
                    state_bit::<AB, D>(local, 0, (output + 2) % 32),
                    state_bit::<AB, D>(local, 0, (output + 13) % 32),
                    state_bit::<AB, D>(local, 0, (output + 22) % 32),
                ) * weight.clone();
                sigma_1[limb] += xor_three(
                    state_bit::<AB, D>(local, 3, (output + 6) % 32),
                    state_bit::<AB, D>(local, 3, (output + 11) % 32),
                    state_bit::<AB, D>(local, 3, (output + 25) % 32),
                ) * weight.clone();
                let shift_0 = if output + 3 < 32 {
                    schedule_bit::<AB, D>(local, 0, output + 3)
                } else {
                    AB::Expr::ZERO
                };
                small_0[limb] += xor_three(
                    schedule_bit::<AB, D>(local, 0, (output + 7) % 32),
                    schedule_bit::<AB, D>(local, 0, (output + 18) % 32),
                    shift_0,
                ) * weight.clone();
                let shift_1 = if output + 10 < 32 {
                    schedule_bit::<AB, D>(local, 1, output + 10)
                } else {
                    AB::Expr::ZERO
                };
                small_1[limb] += xor_three(
                    schedule_bit::<AB, D>(local, 1, (output + 17) % 32),
                    schedule_bit::<AB, D>(local, 1, (output + 19) % 32),
                    shift_1,
                ) * weight.clone();
                let e = state_bit::<AB, D>(local, 3, output);
                let f = state_bit::<AB, D>(local, 4, output);
                let g = state_bit::<AB, D>(local, 5, output);
                choose[limb] += (e.clone() * f + (one.clone() - e) * g) * weight.clone();
                let a = state_bit::<AB, D>(local, 0, output);
                let b = state_bit::<AB, D>(local, 1, output);
                let c = state_bit::<AB, D>(local, 2, output);
                majority[limb] +=
                    (a.clone() * b.clone() + a.clone() * c.clone() + b.clone() * c.clone()
                        - AB::Expr::from_u64(2) * a * b * c)
                        * weight;
            }
        }

        let mut round_constant = [AB::Expr::ZERO, AB::Expr::ZERO];
        for (round, constant) in SHA256_ROUND_CONSTANTS_V2.into_iter().enumerate() {
            let selector = field::<AB, D>(local, SELECTOR_OFFSET_V2 + round);
            round_constant[0] +=
                selector.clone() * AB::Expr::from_u64(u64::from(constant & 0xffff));
            round_constant[1] += selector * AB::Expr::from_u64(u64::from(constant >> 16));
        }

        let schedule_low_carry = carry::<AB, D>(local, SCHEDULE_CARRY_OFFSET_V2, 2);
        let schedule_high_carry = carry::<AB, D>(local, SCHEDULE_CARRY_OFFSET_V2 + 2, 2);
        {
            let mut transition = builder.when_transition();
            for word in 0..15 {
                for limb in 0..2 {
                    transition.assert_zero(
                        normal_round.clone()
                            * (schedule_limb::<AB, D>(next, word, limb)
                                - schedule_limb::<AB, D>(local, word + 1, limb)),
                    );
                }
            }
            transition.assert_zero(
                normal_round.clone()
                    * (schedule_limb::<AB, D>(next, 15, 0)
                        - schedule_limb::<AB, D>(local, 0, 0)
                        - small_0[0].clone()
                        - schedule_limb::<AB, D>(local, 9, 0)
                        - small_1[0].clone()
                        + schedule_low_carry.clone() * radix.clone()),
            );
            transition.assert_zero(
                normal_round.clone()
                    * (schedule_limb::<AB, D>(next, 15, 1)
                        - schedule_limb::<AB, D>(local, 0, 1)
                        - small_0[1].clone()
                        - schedule_limb::<AB, D>(local, 9, 1)
                        - small_1[1].clone()
                        - schedule_low_carry
                        + schedule_high_carry * radix.clone()),
            );
        }
        for bit in 0..4 {
            builder
                .when(round_last.clone())
                .assert_zero(field::<AB, D>(local, SCHEDULE_CARRY_OFFSET_V2 + bit));
        }

        let t1 = [
            bits_limb::<AB, D>(local, T1_BITS_OFFSET_V2, 0),
            bits_limb::<AB, D>(local, T1_BITS_OFFSET_V2, 1),
        ];
        let t2 = [
            bits_limb::<AB, D>(local, T2_BITS_OFFSET_V2, 0),
            bits_limb::<AB, D>(local, T2_BITS_OFFSET_V2, 1),
        ];
        let t1_low_carry = carry::<AB, D>(local, T1_CARRY_OFFSET_V2, 3);
        let t1_high_carry = carry::<AB, D>(local, T1_CARRY_OFFSET_V2 + 3, 3);
        builder.assert_eq(
            t1[0].clone(),
            state_limb::<AB, D>(local, 7, 0)
                + sigma_1[0].clone()
                + choose[0].clone()
                + round_constant[0].clone()
                + schedule_limb::<AB, D>(local, 0, 0)
                - t1_low_carry.clone() * radix.clone(),
        );
        builder.assert_eq(
            t1[1].clone(),
            state_limb::<AB, D>(local, 7, 1)
                + sigma_1[1].clone()
                + choose[1].clone()
                + round_constant[1].clone()
                + schedule_limb::<AB, D>(local, 0, 1)
                + t1_low_carry
                - t1_high_carry * radix.clone(),
        );
        let t2_low_carry = field::<AB, D>(local, T2_CARRY_OFFSET_V2);
        let t2_high_carry = field::<AB, D>(local, T2_CARRY_OFFSET_V2 + 1);
        builder.assert_eq(
            t2[0].clone(),
            sigma_0[0].clone() + majority[0].clone() - t2_low_carry.clone() * radix.clone(),
        );
        builder.assert_eq(
            t2[1].clone(),
            sigma_0[1].clone() + majority[1].clone() + t2_low_carry - t2_high_carry * radix.clone(),
        );

        let e_low_carry = field::<AB, D>(local, E_CARRY_OFFSET_V2);
        let e_high_carry = field::<AB, D>(local, E_CARRY_OFFSET_V2 + 1);
        let next_e = [
            state_limb::<AB, D>(local, 3, 0) + t1[0].clone() - e_low_carry.clone() * radix.clone(),
            state_limb::<AB, D>(local, 3, 1) + t1[1].clone() + e_low_carry
                - e_high_carry * radix.clone(),
        ];
        let a_low_carry = field::<AB, D>(local, A_CARRY_OFFSET_V2);
        let a_high_carry = field::<AB, D>(local, A_CARRY_OFFSET_V2 + 1);
        let next_a = [
            t1[0].clone() + t2[0].clone() - a_low_carry.clone() * radix.clone(),
            t1[1].clone() + t2[1].clone() + a_low_carry - a_high_carry * radix.clone(),
        ];
        {
            let mut transition = builder.when_transition();
            for limb in 0..2 {
                transition.assert_zero(
                    normal_round.clone()
                        * (state_limb::<AB, D>(next, 0, limb) - next_a[limb].clone()),
                );
                transition.assert_zero(
                    normal_round.clone()
                        * (state_limb::<AB, D>(next, 1, limb)
                            - state_limb::<AB, D>(local, 0, limb)),
                );
                transition.assert_zero(
                    normal_round.clone()
                        * (state_limb::<AB, D>(next, 2, limb)
                            - state_limb::<AB, D>(local, 1, limb)),
                );
                transition.assert_zero(
                    normal_round.clone()
                        * (state_limb::<AB, D>(next, 3, limb)
                            - state_limb::<AB, D>(local, 2, limb)),
                );
                transition.assert_zero(
                    normal_round.clone()
                        * (state_limb::<AB, D>(next, 4, limb) - next_e[limb].clone()),
                );
                transition.assert_zero(
                    normal_round.clone()
                        * (state_limb::<AB, D>(next, 5, limb)
                            - state_limb::<AB, D>(local, 4, limb)),
                );
                transition.assert_zero(
                    normal_round.clone()
                        * (state_limb::<AB, D>(next, 6, limb)
                            - state_limb::<AB, D>(local, 5, limb)),
                );
                transition.assert_zero(
                    normal_round.clone()
                        * (state_limb::<AB, D>(next, 7, limb)
                            - state_limb::<AB, D>(local, 6, limb)),
                );
            }
        }
        for bit in 0..16 {
            builder.assert_zero(
                normal_round.clone() * field::<AB, D>(local, OUTPUT_CARRY_OFFSET_V2 + bit),
            );
        }

        let final_state = [
            next_a,
            [
                state_limb::<AB, D>(local, 0, 0),
                state_limb::<AB, D>(local, 0, 1),
            ],
            [
                state_limb::<AB, D>(local, 1, 0),
                state_limb::<AB, D>(local, 1, 1),
            ],
            [
                state_limb::<AB, D>(local, 2, 0),
                state_limb::<AB, D>(local, 2, 1),
            ],
            next_e,
            [
                state_limb::<AB, D>(local, 4, 0),
                state_limb::<AB, D>(local, 4, 1),
            ],
            [
                state_limb::<AB, D>(local, 5, 0),
                state_limb::<AB, D>(local, 5, 1),
            ],
            [
                state_limb::<AB, D>(local, 6, 0),
                state_limb::<AB, D>(local, 6, 1),
            ],
        ];
        let output_state = (0..8)
            .map(|word| {
                let low_carry = field::<AB, D>(local, OUTPUT_CARRY_OFFSET_V2 + word * 2);
                let high_carry = field::<AB, D>(local, OUTPUT_CARRY_OFFSET_V2 + word * 2 + 1);
                [
                    base_state_limb::<AB, D>(local, word, 0) + final_state[word][0].clone()
                        - low_carry.clone() * radix.clone(),
                    base_state_limb::<AB, D>(local, word, 1)
                        + final_state[word][1].clone()
                        + low_carry
                        - high_carry * radix.clone(),
                ]
            })
            .collect::<Vec<_>>();

        match self.role {
            ShaAirRoleV2::Standalone => {
                let mut last = builder.when(round_last);
                for word in 0..8 {
                    for limb in 0..2 {
                        last.assert_eq(
                            public[STANDALONE_OUTPUT_STATE_OFFSET_V2 + word * 2 + limb].clone(),
                            output_state[word][limb].clone(),
                        );
                    }
                }
            }
            ShaAirRoleV2::Chain => {
                for word in 0..8 {
                    for limb in 0..2 {
                        let expected =
                            (0..CHAIN_TRANSITION_SLOTS_V2).fold(AB::Expr::ZERO, |sum, slot| {
                                sum + transition_selectors[slot].clone()
                                    * public[CHAIN_DIGEST_OFFSET_V2 + slot * 16 + word * 2 + limb]
                                        .clone()
                            });
                        builder.assert_zero(
                            transition_final.clone()
                                * (output_state[word][limb].clone() - expected),
                        );
                    }
                }
                let mut transition = builder.when_transition();
                for word in 0..8 {
                    let iv = z00z_crypto::SHA256_IV_V2[word];
                    for limb in 0..2 {
                        let iv_limb = if limb == 0 { iv & 0xffff } else { iv >> 16 };
                        let expected_next = (one.clone() - transition_final.clone())
                            * output_state[word][limb].clone()
                            + transition_final.clone() * AB::Expr::from_u64(u64::from(iv_limb));
                        transition.assert_zero(
                            round_last.clone()
                                * (base_state_limb::<AB, D>(next, word, limb) - expected_next),
                        );
                    }
                }
            }
            ShaAirRoleV2::SemanticTransitionChain | ShaAirRoleV2::SemanticUniquenessChain => {
                let semantic_selectors = &transition_selectors[..SemanticShaJobKindV2::COUNT];
                let real = semantic_selectors
                    .iter()
                    .cloned()
                    .fold(AB::Expr::ZERO, |sum, selector| sum + selector);
                let lane = field::<AB, D>(local, JMT_LANE_OFFSET_V2);
                let global_lane = lane.clone() + public[CHAIN_SLICE_START_OFFSET_V2].clone();
                let record = field::<AB, D>(local, JMT_RECORD_OFFSET_V2);
                let role = field::<AB, D>(local, JMT_ROLE_OFFSET_V2);
                for word in 0..8 {
                    for (pair, value) in [
                        (word * 2, output_state[word][1].clone()),
                        (word * 2 + 1, output_state[word][0].clone()),
                    ] {
                        builder.push_interaction(
                            SEMANTIC_SHA_DIGEST_PAIR_BUS_V2,
                            vec![
                                global_lane.clone(),
                                role.clone(),
                                record.clone(),
                                AB::Expr::from_usize(pair),
                                value,
                            ],
                            Count::bounded(transition_final.clone() * real.clone(), 1),
                        );
                    }
                }
                let padding = transition_selectors[CHAIN_PADDING_SLOT_V2].clone();
                let mut transition = builder.when_transition();
                for word in 0..8 {
                    let iv = z00z_crypto::SHA256_IV_V2[word];
                    for limb in 0..2 {
                        let iv_limb = if limb == 0 { iv & 0xffff } else { iv >> 16 };
                        let expected_next = real.clone()
                            * (one.clone() - transition_final.clone())
                            * output_state[word][limb].clone()
                            + (padding.clone() + real.clone() * transition_final.clone())
                                * AB::Expr::from_u64(u64::from(iv_limb));
                        transition.assert_zero(
                            round_last.clone()
                                * (base_state_limb::<AB, D>(next, word, limb) - expected_next),
                        );
                    }
                }
            }
            ShaAirRoleV2::JmtLinked => {
                let lane = field::<AB, D>(local, JMT_LANE_OFFSET_V2);
                let record = field::<AB, D>(local, JMT_RECORD_OFFSET_V2);
                let role = field::<AB, D>(local, JMT_ROLE_OFFSET_V2);
                let block_count = field::<AB, D>(local, JMT_BLOCK_COUNT_OFFSET_V2);
                for word in 0..8 {
                    for limb in 0..2 {
                        builder.push_interaction(
                            JMT_SHA_DIGEST_BUS_V2,
                            vec![
                                lane.clone(),
                                record.clone(),
                                role.clone(),
                                block_count.clone(),
                                AB::Expr::from_usize(word * 2 + limb),
                                output_state[word][limb].clone(),
                            ],
                            Count::bounded(transition_final.clone(), 1),
                        );
                    }
                }
                let real = transition_selectors[0].clone();
                let padding = transition_selectors[CHAIN_PADDING_SLOT_V2].clone();
                let mut transition = builder.when_transition();
                for word in 0..8 {
                    let iv = z00z_crypto::SHA256_IV_V2[word];
                    for limb in 0..2 {
                        let iv_limb = if limb == 0 { iv & 0xffff } else { iv >> 16 };
                        let expected_next = real.clone()
                            * (one.clone() - transition_final.clone())
                            * output_state[word][limb].clone()
                            + (padding.clone() + real.clone() * transition_final.clone())
                                * AB::Expr::from_u64(u64::from(iv_limb));
                        transition.assert_zero(
                            round_last.clone()
                                * (base_state_limb::<AB, D>(next, word, limb) - expected_next),
                        );
                    }
                }
            }
            ShaAirRoleV2::RecursiveCompression => {
                let preprocessed = builder.preprocessed().clone();
                let prep = preprocessed.current_slice();
                let active: AB::Expr = prep[0].into();
                for word in 0..8 {
                    for limb in 0..2 {
                        let output = word * 2 + limb;
                        let mut fields = Vec::with_capacity(D + 1);
                        fields.push(prep[1 + output].into());
                        fields.push(output_state[word][limb].clone());
                        fields.extend(core::iter::repeat_n(AB::Expr::ZERO, D.saturating_sub(1)));
                        let output_multiplicity: AB::Expr = prep[1 + 16 + output].into();
                        builder.push_interaction(
                            "WitnessChecks",
                            fields,
                            Count::bounded(
                                round_last.clone() * active.clone() * output_multiplicity,
                                1,
                            ),
                        );
                    }
                }
            }
        }
    }
}

impl BatchAir<Plonky3StarkConfigV2> for ShaAirV2<KoalaBear, 1> {}
impl BatchAir<Plonky3StarkConfigV2> for ShaAirV2<KoalaBear, 4> {}
