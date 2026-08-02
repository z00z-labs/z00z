//! Packed canonical event-byte source linked to the epoch SHA chain.

use p3_air::{Air, AirBuilder, WindowAccess};
use p3_field::{Field, PrimeCharacteristicRing};
use p3_lookup::{Count, InteractionBuilder};

use super::plonky3_epoch_event_source_columns::{
    field, EventSourceAirRoleV2, EventSourceAirV2, ACTIVE_OFFSET_V2, BITS_OFFSET_V2,
    BLOCK_INDEX_OFFSET_V2, BLOCK_PAIR_COUNT_V2, BYTE_0_OFFSET_V2, BYTE_1_OFFSET_V2,
    EVENT_SOURCE_BYTE_BUS_V2, FINAL_BLOCK_OFFSET_V2, FRAMED_PREFIX_PAIRS_V2,
    JOB_BIT_LENGTH_PAIR_COUNT_V2, JOB_BIT_LENGTH_PAIR_OFFSET_V2, JOB_BLOCK_COUNT_OFFSET_V2,
    JOB_ID_OFFSET_V2, JOB_KIND_SELECTOR_OFFSET_V2, JOB_RAW_LEN_OFFSET_V2, JOB_START_OFFSET_V2,
    LENGTH_PAIRS_V2, PADDING_ZERO_BITS_OFFSET_V2, PADDING_ZERO_BITS_V2, PAD_BYTE_0_OFFSET_V2,
    PAD_BYTE_1_OFFSET_V2, PAIR_SELECTOR_OFFSET_V2, PREFIX_SELECTOR_OFFSET_V2,
    PUBLIC_ACTIVE_OFFSET_V2, PUBLIC_BIT_LENGTH_OFFSET_V2, PUBLIC_BLOCK_COUNT_OFFSET_V2,
    PUBLIC_EVENT_BYTES_OFFSET_V2, PUBLIC_PART_LENGTH_OFFSET_V2, PUBLIC_SLICE_LEN_OFFSET_V2,
    PUBLIC_SLICE_START_OFFSET_V2, PUBLIC_STATIC_PREFIX_OFFSET_V2, RAW_BYTE_0_OFFSET_V2,
    RAW_BYTE_1_OFFSET_V2, RAW_FINAL_OFFSET_V2, ROW_FIELDS_V2, RUNNING_RAW_BYTES_OFFSET_V2,
    STATIC_PREFIX_PAIRS_V2, TRANSITION_SELECTOR_OFFSET_V2, TRANSITION_SLOTS_V2,
};
use super::plonky3_epoch_sha256_columns::{
    SemanticShaJobKindV2, SEMANTIC_SHA_BLOCK_PAIR_BUS_V2, SEMANTIC_SHA_RAW_BYTE_BUS_V2,
    SHA_BLOCK_PAIR_BUS_V2,
};
use super::Plonky3StarkConfigV2;
use z00z_plonky3_circuit_prover::batch_stark_prover::BatchAir;

fn selected_public<AB>(selectors: &[AB::Expr], public: &[AB::Expr], offset: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    selectors
        .iter()
        .enumerate()
        .fold(AB::Expr::ZERO, |sum, (slot, selector)| {
            sum + selector.clone() * public[offset + slot].clone()
        })
}

fn assert_disjoint<AB>(builder: &mut AB, values: &[AB::Expr])
where
    AB: AirBuilder,
{
    for left in 0..values.len() {
        for right in left + 1..values.len() {
            builder.assert_zero(values[left].clone() * values[right].clone());
        }
    }
}

impl<AB> Air<AB> for EventSourceAirV2
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

        let active = field::<AB>(local, ACTIVE_OFFSET_V2);
        let next_active = field::<AB>(next, ACTIVE_OFFSET_V2);
        builder.assert_bool(active.clone());
        builder.when_first_row().assert_one(active.clone());
        builder.when_last_row().assert_zero(active.clone());
        builder
            .when_transition()
            .assert_zero(next_active.clone() * (one.clone() - active.clone()));

        let public_active = (0..TRANSITION_SLOTS_V2)
            .map(|slot| public[PUBLIC_ACTIVE_OFFSET_V2 + slot].clone())
            .collect::<Vec<_>>();
        for flag in &public_active {
            builder.assert_bool(flag.clone());
        }
        builder.assert_one(public_active[0].clone());
        for slot in 1..TRANSITION_SLOTS_V2 {
            builder.assert_zero(
                public_active[slot].clone() * (one.clone() - public_active[slot - 1].clone()),
            );
        }
        builder.assert_eq(
            public[PUBLIC_SLICE_LEN_OFFSET_V2].clone(),
            public_active
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, flag| sum + flag),
        );

        let transition_selectors = (0..TRANSITION_SLOTS_V2)
            .map(|slot| field::<AB>(local, TRANSITION_SELECTOR_OFFSET_V2 + slot))
            .collect::<Vec<_>>();
        let next_transition_selectors = (0..TRANSITION_SLOTS_V2)
            .map(|slot| field::<AB>(next, TRANSITION_SELECTOR_OFFSET_V2 + slot))
            .collect::<Vec<_>>();
        let transition_sum = transition_selectors
            .iter()
            .cloned()
            .fold(AB::Expr::ZERO, |sum, value| sum + value);
        builder.assert_eq(transition_sum, active.clone());
        for (slot, selector) in transition_selectors.iter().enumerate() {
            builder.assert_bool(selector.clone());
            builder.assert_zero(selector.clone() * (one.clone() - public_active[slot].clone()));
        }
        let transition_index = transition_selectors
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (slot, selector)| {
                sum + selector.clone() * AB::Expr::from_usize(slot)
            });
        let global_transition_index =
            transition_index.clone() + public[PUBLIC_SLICE_START_OFFSET_V2].clone();

        let job_kind_selectors = (0..SemanticShaJobKindV2::COUNT)
            .map(|index| field::<AB>(local, JOB_KIND_SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        let next_job_kind_selectors = (0..SemanticShaJobKindV2::COUNT)
            .map(|index| field::<AB>(next, JOB_KIND_SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        let job_kind = job_kind_selectors
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (index, selector)| {
                sum + selector.clone() * AB::Expr::from_usize(index + 1)
            });
        let event_vector_job = if self.role == EventSourceAirRoleV2::Hash {
            active.clone()
        } else {
            job_kind_selectors[SemanticShaJobKindV2::EventVector.index()].clone()
        };
        let semantic_raw_job = active.clone() - event_vector_job.clone();
        let job_id = field::<AB>(local, JOB_ID_OFFSET_V2);
        let next_job_id = field::<AB>(next, JOB_ID_OFFSET_V2);
        let job_raw_len = field::<AB>(local, JOB_RAW_LEN_OFFSET_V2);
        let next_job_raw_len = field::<AB>(next, JOB_RAW_LEN_OFFSET_V2);
        let job_block_count = field::<AB>(local, JOB_BLOCK_COUNT_OFFSET_V2);
        let next_job_block_count = field::<AB>(next, JOB_BLOCK_COUNT_OFFSET_V2);
        let job_bit_length_pairs = (0..JOB_BIT_LENGTH_PAIR_COUNT_V2)
            .map(|pair| field::<AB>(local, JOB_BIT_LENGTH_PAIR_OFFSET_V2 + pair))
            .collect::<Vec<_>>();
        let next_job_bit_length_pairs = (0..JOB_BIT_LENGTH_PAIR_COUNT_V2)
            .map(|pair| field::<AB>(next, JOB_BIT_LENGTH_PAIR_OFFSET_V2 + pair))
            .collect::<Vec<_>>();
        let job_start = field::<AB>(local, JOB_START_OFFSET_V2);
        let next_job_start = field::<AB>(next, JOB_START_OFFSET_V2);
        let padding_zero_bits = (0..PADDING_ZERO_BITS_V2)
            .map(|bit| field::<AB>(local, PADDING_ZERO_BITS_OFFSET_V2 + bit))
            .collect::<Vec<_>>();
        let padding_zeros = padding_zero_bits
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (bit, value)| {
                sum + value.clone() * AB::Expr::from_u64(1_u64 << bit)
            });
        if self.role == EventSourceAirRoleV2::Hash {
            for offset in JOB_KIND_SELECTOR_OFFSET_V2..ROW_FIELDS_V2 {
                builder.assert_zero(field::<AB>(local, offset));
            }
        } else {
            for selector in &job_kind_selectors {
                builder.assert_bool(selector.clone());
            }
            builder.assert_eq(
                job_kind_selectors
                    .iter()
                    .cloned()
                    .fold(AB::Expr::ZERO, |sum, value| sum + value),
                active.clone(),
            );
            builder.assert_bool(job_start.clone());
            builder.assert_zero(job_start.clone() * (one.clone() - active.clone()));
            for bit in &padding_zero_bits {
                builder.assert_bool(bit.clone());
            }
            builder.assert_zero((one.clone() - active.clone()) * job_id.clone());
            builder.assert_zero((one.clone() - active.clone()) * job_raw_len.clone());
            builder.assert_zero((one.clone() - active.clone()) * job_block_count.clone());
        }

        let pair_selectors = (0..BLOCK_PAIR_COUNT_V2)
            .map(|pair| field::<AB>(local, PAIR_SELECTOR_OFFSET_V2 + pair))
            .collect::<Vec<_>>();
        let next_pair_selectors = (0..BLOCK_PAIR_COUNT_V2)
            .map(|pair| field::<AB>(next, PAIR_SELECTOR_OFFSET_V2 + pair))
            .collect::<Vec<_>>();
        let pair_sum = pair_selectors
            .iter()
            .cloned()
            .fold(AB::Expr::ZERO, |sum, value| sum + value);
        builder.assert_eq(pair_sum, active.clone());
        for selector in &pair_selectors {
            builder.assert_bool(selector.clone());
        }
        let pair_last = pair_selectors[BLOCK_PAIR_COUNT_V2 - 1].clone();

        let prefix_selectors = (0..FRAMED_PREFIX_PAIRS_V2)
            .map(|pair| field::<AB>(local, PREFIX_SELECTOR_OFFSET_V2 + pair))
            .collect::<Vec<_>>();
        let next_prefix_selectors = (0..FRAMED_PREFIX_PAIRS_V2)
            .map(|pair| field::<AB>(next, PREFIX_SELECTOR_OFFSET_V2 + pair))
            .collect::<Vec<_>>();
        let prefix_active = prefix_selectors
            .iter()
            .cloned()
            .fold(AB::Expr::ZERO, |sum, value| sum + value);
        builder.assert_bool(prefix_active.clone());
        for selector in &prefix_selectors {
            builder.assert_bool(selector.clone());
        }
        if self.role != EventSourceAirRoleV2::Hash {
            builder.assert_zero(semantic_raw_job.clone() * prefix_active.clone());
        }

        let block_index = field::<AB>(local, BLOCK_INDEX_OFFSET_V2);
        let next_block_index = field::<AB>(next, BLOCK_INDEX_OFFSET_V2);
        let final_block = field::<AB>(local, FINAL_BLOCK_OFFSET_V2);
        let next_final_block = field::<AB>(next, FINAL_BLOCK_OFFSET_V2);
        builder.assert_bool(final_block.clone());
        builder.assert_zero(final_block.clone() * (one.clone() - active.clone()));

        let public_block_count: AB::Expr =
            selected_public::<AB>(&transition_selectors, &public, PUBLIC_BLOCK_COUNT_OFFSET_V2);
        let public_event_bytes: AB::Expr =
            selected_public::<AB>(&transition_selectors, &public, PUBLIC_EVENT_BYTES_OFFSET_V2);
        let expected_block_count = if self.role == EventSourceAirRoleV2::Hash {
            public_block_count.clone()
        } else {
            builder.assert_zero(
                event_vector_job.clone() * (job_block_count.clone() - public_block_count),
            );
            builder.assert_zero(
                event_vector_job.clone() * (job_raw_len.clone() - public_event_bytes.clone()),
            );
            let prefix_bytes =
                event_vector_job.clone() * AB::Expr::from_usize(FRAMED_PREFIX_PAIRS_V2 * 2);
            let message_len = prefix_bytes.clone() + job_raw_len.clone();
            builder.assert_zero(
                active.clone()
                    * (job_block_count.clone() * AB::Expr::from_u64(64)
                        - message_len.clone()
                        - AB::Expr::from_u64(9)
                        - padding_zeros.clone()),
            );
            builder.assert_zero(active.clone() * job_bit_length_pairs[0].clone());
            builder.assert_zero(active.clone() * job_bit_length_pairs[1].clone());
            builder.assert_zero(
                active.clone()
                    * (job_bit_length_pairs[2].clone() * AB::Expr::from_u64(65_536)
                        + job_bit_length_pairs[3].clone()
                        - message_len * AB::Expr::from_u64(8)),
            );
            job_block_count.clone()
        };
        builder.assert_zero(
            final_block.clone() * (block_index.clone() + one.clone() - expected_block_count),
        );

        let raw_0 = field::<AB>(local, RAW_BYTE_0_OFFSET_V2);
        let raw_1 = field::<AB>(local, RAW_BYTE_1_OFFSET_V2);
        let next_raw_0 = field::<AB>(next, RAW_BYTE_0_OFFSET_V2);
        let next_raw_1 = field::<AB>(next, RAW_BYTE_1_OFFSET_V2);
        let raw_final = field::<AB>(local, RAW_FINAL_OFFSET_V2);
        let pad_0 = field::<AB>(local, PAD_BYTE_0_OFFSET_V2);
        let pad_1 = field::<AB>(local, PAD_BYTE_1_OFFSET_V2);
        let next_pad_0 = field::<AB>(next, PAD_BYTE_0_OFFSET_V2);
        for flag in [
            raw_0.clone(),
            raw_1.clone(),
            raw_final.clone(),
            pad_0.clone(),
            pad_1.clone(),
        ] {
            builder.assert_bool(flag);
        }
        builder.assert_zero(raw_1.clone() * (one.clone() - raw_0.clone()));
        builder.assert_zero(raw_final.clone() * (one.clone() - raw_0.clone()));
        builder.assert_zero((raw_0.clone() - raw_final.clone()) * (one.clone() - raw_1.clone()));
        builder.assert_eq(
            pad_1.clone(),
            raw_final.clone() * (one.clone() - raw_1.clone()),
        );

        let byte_0 = field::<AB>(local, BYTE_0_OFFSET_V2);
        let byte_1 = field::<AB>(local, BYTE_1_OFFSET_V2);
        for bit in 0..16 {
            builder.assert_bool(field::<AB>(local, BITS_OFFSET_V2 + bit));
        }
        let reconstructed_0 = (0..8).fold(AB::Expr::ZERO, |sum, bit| {
            sum + field::<AB>(local, BITS_OFFSET_V2 + bit) * AB::Expr::from_u64(1_u64 << bit)
        });
        let reconstructed_1 = (0..8).fold(AB::Expr::ZERO, |sum, bit| {
            sum + field::<AB>(local, BITS_OFFSET_V2 + 8 + bit) * AB::Expr::from_u64(1_u64 << bit)
        });
        builder.assert_eq(byte_0.clone(), reconstructed_0);
        builder.assert_eq(byte_1.clone(), reconstructed_1);
        let pair_value = byte_0.clone() * AB::Expr::from_u64(256) + byte_1.clone();

        let running = field::<AB>(local, RUNNING_RAW_BYTES_OFFSET_V2);
        let next_running = field::<AB>(next, RUNNING_RAW_BYTES_OFFSET_V2);
        let expected_raw_len = if self.role == EventSourceAirRoleV2::Hash {
            public_event_bytes.clone()
        } else {
            job_raw_len.clone()
        };
        builder.assert_zero(raw_final.clone() * (running.clone() - expected_raw_len.clone()));

        let length_pair_active = final_block.clone()
            * pair_selectors[BLOCK_PAIR_COUNT_V2 - LENGTH_PAIRS_V2..]
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value);
        let roles_0 = [
            prefix_active.clone(),
            raw_0.clone(),
            pad_0.clone(),
            length_pair_active.clone(),
        ];
        let roles_1 = [
            prefix_active.clone(),
            raw_1.clone(),
            pad_1.clone(),
            length_pair_active.clone(),
        ];
        assert_disjoint(builder, &roles_0);
        assert_disjoint(builder, &roles_1);
        let zero_0 = active.clone()
            - roles_0
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value);
        let zero_1 = active.clone()
            - roles_1
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value);
        builder.assert_zero(zero_0 * byte_0.clone());
        builder.assert_zero(zero_1 * byte_1.clone());
        builder.assert_zero(pad_0.clone() * (byte_0.clone() - AB::Expr::from_u64(128)));
        builder.assert_zero(pad_1.clone() * (byte_1.clone() - AB::Expr::from_u64(128)));

        for (index, selector) in prefix_selectors.iter().enumerate() {
            let expected = if index < STATIC_PREFIX_PAIRS_V2 {
                public[PUBLIC_STATIC_PREFIX_OFFSET_V2 + index].clone()
            } else {
                let pair = index - STATIC_PREFIX_PAIRS_V2;
                transition_selectors.iter().enumerate().fold(
                    AB::Expr::ZERO,
                    |sum, (slot, transition_selector)| {
                        sum + transition_selector.clone()
                            * public[PUBLIC_PART_LENGTH_OFFSET_V2 + slot * LENGTH_PAIRS_V2 + pair]
                                .clone()
                    },
                )
            };
            builder.assert_zero(selector.clone() * (pair_value.clone() - expected));
        }
        for pair in 0..LENGTH_PAIRS_V2 {
            let gate = final_block.clone() * pair_selectors[BLOCK_PAIR_COUNT_V2 - 4 + pair].clone();
            let expected = if self.role == EventSourceAirRoleV2::Hash {
                transition_selectors.iter().enumerate().fold(
                    AB::Expr::ZERO,
                    |sum, (slot, transition_selector)| {
                        sum + transition_selector.clone()
                            * public[PUBLIC_BIT_LENGTH_OFFSET_V2 + slot * LENGTH_PAIRS_V2 + pair]
                                .clone()
                    },
                )
            } else {
                job_bit_length_pairs[pair].clone()
            };
            builder.assert_zero(gate.clone() * (pair_value.clone() - expected));
            if self.role != EventSourceAirRoleV2::Hash && pair == 2 {
                builder.assert_zero(gate * byte_0.clone());
            }
        }

        let slot_end = final_block.clone() * pair_last.clone();
        let continues = active.clone() - slot_end.clone();
        let next_slot_active = transition_selectors.iter().enumerate().fold(
            AB::Expr::ZERO,
            |sum, (slot, selector)| {
                let successor = public_active
                    .get(slot + 1)
                    .cloned()
                    .unwrap_or(AB::Expr::ZERO);
                sum + selector.clone() * successor
            },
        );
        builder.assert_zero(slot_end.clone() * (running.clone() - expected_raw_len));

        {
            let mut first = builder.when_first_row();
            first.assert_one(transition_selectors[0].clone());
            first.assert_one(pair_selectors[0].clone());
            first.assert_one(prefix_selectors[0].clone());
            first.assert_zero(block_index.clone());
            first.assert_zero(running.clone());
            first.assert_zero(raw_0.clone());
            first.assert_zero(pad_0.clone());
            if self.role != EventSourceAirRoleV2::Hash {
                first.assert_one(job_start.clone());
                first.assert_one(event_vector_job.clone());
                first.assert_zero(job_id.clone());
            }
        }
        if self.role != EventSourceAirRoleV2::Hash {
            builder.assert_zero(job_start.clone() * (one.clone() - pair_selectors[0].clone()));
            builder.assert_zero(job_start.clone() * block_index.clone());
            builder.assert_zero(event_vector_job.clone() * job_id.clone());
        }

        {
            let mut transition = builder.when_transition();
            if self.role == EventSourceAirRoleV2::Hash {
                transition.assert_eq(
                    next_active.clone(),
                    continues.clone() + slot_end.clone() * next_slot_active.clone(),
                );
                for slot in 0..TRANSITION_SLOTS_V2 {
                    let incoming = if slot == 0 {
                        AB::Expr::ZERO
                    } else {
                        slot_end.clone()
                            * transition_selectors[slot - 1].clone()
                            * public_active[slot].clone()
                    };
                    transition.assert_eq(
                        next_transition_selectors[slot].clone(),
                        continues.clone() * transition_selectors[slot].clone() + incoming,
                    );
                }
                transition.assert_eq(
                    next_pair_selectors[0].clone(),
                    pair_last.clone() * continues.clone()
                        + slot_end.clone() * next_slot_active.clone(),
                );
                for pair in 1..BLOCK_PAIR_COUNT_V2 {
                    transition.assert_eq(
                        next_pair_selectors[pair].clone(),
                        continues.clone() * pair_selectors[pair - 1].clone(),
                    );
                }
                transition.assert_eq(
                    next_block_index,
                    continues.clone() * (block_index.clone() + pair_last.clone()),
                );
                transition.assert_zero(
                    (active.clone() - pair_last.clone()) * (next_final_block - final_block.clone()),
                );
                transition.assert_eq(
                    next_prefix_selectors[0].clone(),
                    slot_end.clone() * next_slot_active,
                );
                for pair in 1..FRAMED_PREFIX_PAIRS_V2 {
                    transition.assert_eq(
                        next_prefix_selectors[pair].clone(),
                        prefix_selectors[pair - 1].clone(),
                    );
                }
                transition.assert_eq(
                    next_raw_0.clone(),
                    prefix_selectors[FRAMED_PREFIX_PAIRS_V2 - 1].clone()
                        + raw_0.clone() * (one.clone() - raw_final.clone()),
                );
                transition.assert_eq(next_pad_0, raw_final.clone() * raw_1.clone());
                transition.assert_eq(
                    next_running,
                    continues * (running.clone() + next_raw_0.clone() + next_raw_1.clone()),
                );
            } else {
                transition.assert_zero(continues.clone() * (next_active.clone() - one.clone()));
                for slot in 0..TRANSITION_SLOTS_V2 {
                    transition.assert_zero(
                        continues.clone()
                            * (next_transition_selectors[slot].clone()
                                - transition_selectors[slot].clone()),
                    );
                }
                transition.assert_eq(
                    next_pair_selectors[0].clone(),
                    pair_last.clone() * continues.clone() + slot_end.clone() * next_active.clone(),
                );
                for pair in 1..BLOCK_PAIR_COUNT_V2 {
                    transition.assert_eq(
                        next_pair_selectors[pair].clone(),
                        continues.clone() * pair_selectors[pair - 1].clone(),
                    );
                }
                transition.assert_eq(
                    next_block_index,
                    continues.clone() * (block_index.clone() + pair_last.clone()),
                );
                transition.assert_zero(
                    (active.clone() - pair_last.clone()) * (next_final_block - final_block.clone()),
                );
                transition.assert_eq(
                    next_prefix_selectors[0].clone(),
                    slot_end.clone()
                        * next_job_kind_selectors[SemanticShaJobKindV2::EventVector.index()]
                            .clone(),
                );
                for pair in 1..FRAMED_PREFIX_PAIRS_V2 {
                    transition.assert_eq(
                        next_prefix_selectors[pair].clone(),
                        continues.clone() * prefix_selectors[pair - 1].clone(),
                    );
                }
                transition.assert_eq(
                    next_raw_0.clone(),
                    prefix_selectors[FRAMED_PREFIX_PAIRS_V2 - 1].clone()
                        + raw_0.clone() * (one.clone() - raw_final.clone())
                        + slot_end.clone()
                            * (next_active.clone()
                                - next_job_kind_selectors
                                    [SemanticShaJobKindV2::EventVector.index()]
                                .clone()),
                );
                transition.assert_eq(next_pad_0, raw_final.clone() * raw_1.clone());
                transition.assert_eq(
                    next_running,
                    continues.clone() * running.clone() + next_raw_0.clone() + next_raw_1.clone(),
                );
                transition.assert_eq(next_job_start, slot_end.clone() * next_active.clone());
                for index in 0..SemanticShaJobKindV2::COUNT {
                    transition.assert_zero(
                        continues.clone()
                            * (next_job_kind_selectors[index].clone()
                                - job_kind_selectors[index].clone()),
                    );
                }
                for (local_value, next_value) in [
                    (job_id.clone(), next_job_id),
                    (job_raw_len.clone(), next_job_raw_len),
                    (job_block_count.clone(), next_job_block_count),
                ] {
                    transition.assert_zero(continues.clone() * (next_value - local_value));
                }
                for pair in 0..JOB_BIT_LENGTH_PAIR_COUNT_V2 {
                    transition.assert_zero(
                        continues.clone()
                            * (next_job_bit_length_pairs[pair].clone()
                                - job_bit_length_pairs[pair].clone()),
                    );
                }
                for bit in 0..PADDING_ZERO_BITS_V2 {
                    transition.assert_zero(
                        continues.clone()
                            * (field::<AB>(next, PADDING_ZERO_BITS_OFFSET_V2 + bit)
                                - padding_zero_bits[bit].clone()),
                    );
                }
            }
        }
        if self.role != EventSourceAirRoleV2::Hash {
            let raw_before = running.clone() - raw_0.clone() - raw_1.clone();
            builder.push_interaction(
                EVENT_SOURCE_BYTE_BUS_V2,
                vec![
                    global_transition_index.clone(),
                    raw_before.clone(),
                    byte_0.clone(),
                ],
                -Count::bounded(raw_0.clone() * event_vector_job.clone(), 1),
            );
            builder.push_interaction(
                EVENT_SOURCE_BYTE_BUS_V2,
                vec![
                    global_transition_index.clone(),
                    running.clone() - raw_1.clone(),
                    byte_1.clone(),
                ],
                -Count::bounded(raw_1.clone() * event_vector_job.clone(), 1),
            );
            builder.push_interaction(
                SEMANTIC_SHA_RAW_BYTE_BUS_V2,
                vec![
                    global_transition_index.clone(),
                    job_kind.clone(),
                    job_id.clone(),
                    raw_before,
                    byte_0.clone(),
                ],
                -Count::bounded(raw_0.clone() * semantic_raw_job.clone(), 1),
            );
            builder.push_interaction(
                SEMANTIC_SHA_RAW_BYTE_BUS_V2,
                vec![
                    global_transition_index.clone(),
                    job_kind.clone(),
                    job_id.clone(),
                    running.clone() - raw_1.clone(),
                    byte_1.clone(),
                ],
                -Count::bounded(raw_1.clone() * semantic_raw_job.clone(), 1),
            );
        }
        let pair_index = pair_selectors
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (pair, selector)| {
                sum + selector.clone() * AB::Expr::from_usize(pair)
            });
        if self.role == EventSourceAirRoleV2::Hash {
            builder.push_interaction(
                SHA_BLOCK_PAIR_BUS_V2,
                vec![transition_index, block_index, pair_index, pair_value],
                -Count::bounded(active.clone(), 1),
            );
        } else {
            builder.push_interaction(
                SEMANTIC_SHA_BLOCK_PAIR_BUS_V2,
                vec![
                    global_transition_index,
                    job_kind,
                    job_id,
                    job_block_count,
                    block_index,
                    pair_index,
                    pair_value,
                ],
                -Count::bounded(active.clone(), 1),
            );
        }

        let inactive = one - active;
        for offset in BLOCK_INDEX_OFFSET_V2..ROW_FIELDS_V2 {
            builder.assert_zero(inactive.clone() * field::<AB>(local, offset));
        }
    }
}

impl BatchAir<Plonky3StarkConfigV2> for EventSourceAirV2 {}
