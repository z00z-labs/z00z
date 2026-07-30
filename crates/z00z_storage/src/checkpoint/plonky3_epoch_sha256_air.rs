//! Direct SHA-256 compression AIR for bounded epoch hash work.
//!
//! The trace proves one complete 64-round compression block. State and message
//! schedule limbs advance row by row; selective bit decompositions constrain
//! every Boolean SHA operation without materializing a full bit circuit per
//! epoch transition. This table alone is never frontier-admissible.

use core::any::Any;

use p3_air::{Air, AirBuilder, BaseAir, WindowAccess};
use p3_circuit::ops::NpoTypeId;
use p3_circuit::tables::NonPrimitiveTrace;
use p3_field::{Field, PrimeCharacteristicRing};
use p3_koala_bear::KoalaBear;
use p3_matrix::dense::RowMajorMatrix;
use z00z_plonky3_circuit_prover::batch_stark_prover::BatchAir;

use super::{Plonky3StarkConfigV2, EPOCH_CHUNK_BYTES_V2, SHA256_ROUND_CONSTANTS_V2};

const SHA_NPO_ID_V2: &str = "z00z/plonky3/epoch-sha256/v2";
pub(super) const SHA_ROWS_V2: usize = 64;
const STATEMENT_LIMBS_V2: usize = EPOCH_CHUNK_BYTES_V2 / core::mem::size_of::<u16>();
const INPUT_STATE_OFFSET_V2: usize = STATEMENT_LIMBS_V2;
const BLOCK_OFFSET_V2: usize = INPUT_STATE_OFFSET_V2 + 16;
const OUTPUT_STATE_OFFSET_V2: usize = BLOCK_OFFSET_V2 + 32;
pub(super) const PUBLIC_FIELDS_V2: usize = OUTPUT_STATE_OFFSET_V2 + 16;

const STATE_LIMBS_OFFSET_V2: usize = 0;
const STATE_LIMBS_V2: usize = 16;
const STATE_BITS_OFFSET_V2: usize = STATE_LIMBS_OFFSET_V2 + STATE_LIMBS_V2;
pub(super) const STATE_BIT_WORDS_V2: [usize; 6] = [0, 1, 2, 4, 5, 6];
const STATE_BITS_V2: usize = STATE_BIT_WORDS_V2.len() * 32;
const SCHEDULE_LIMBS_OFFSET_V2: usize = STATE_BITS_OFFSET_V2 + STATE_BITS_V2;
const SCHEDULE_LIMBS_V2: usize = 32;
const SCHEDULE_BITS_OFFSET_V2: usize = SCHEDULE_LIMBS_OFFSET_V2 + SCHEDULE_LIMBS_V2;
const SCHEDULE_BITS_V2: usize = 64;
const SELECTOR_OFFSET_V2: usize = SCHEDULE_BITS_OFFSET_V2 + SCHEDULE_BITS_V2;
const SELECTOR_FIELDS_V2: usize = SHA_ROWS_V2;
const T1_BITS_OFFSET_V2: usize = SELECTOR_OFFSET_V2 + SELECTOR_FIELDS_V2;
const T2_BITS_OFFSET_V2: usize = T1_BITS_OFFSET_V2 + 32;
const SCHEDULE_CARRY_OFFSET_V2: usize = T2_BITS_OFFSET_V2 + 32;
const T1_CARRY_OFFSET_V2: usize = SCHEDULE_CARRY_OFFSET_V2 + 4;
const T2_CARRY_OFFSET_V2: usize = T1_CARRY_OFFSET_V2 + 6;
const E_CARRY_OFFSET_V2: usize = T2_CARRY_OFFSET_V2 + 2;
const A_CARRY_OFFSET_V2: usize = E_CARRY_OFFSET_V2 + 2;
const OUTPUT_CARRY_OFFSET_V2: usize = A_CARRY_OFFSET_V2 + 2;
const ROW_FIELDS_V2: usize = OUTPUT_CARRY_OFFSET_V2 + 16;
pub(super) const CALL_FIELDS_V2: usize = PUBLIC_FIELDS_V2 + ROW_FIELDS_V2;
const PREPROCESSED_WIDTH_V2: usize = 1;
const RADIX_V2: u64 = 65_536;

#[derive(Clone, Debug)]
pub(super) struct ShaRowV2 {
    pub(super) values: Vec<KoalaBear>,
}

#[derive(Clone, Debug)]
pub(super) struct ShaTraceV2 {
    pub(super) rows: Vec<ShaRowV2>,
}

impl NonPrimitiveTrace<KoalaBear> for ShaTraceV2 {
    fn op_type(&self) -> NpoTypeId {
        sha_npo_type()
    }

    fn rows(&self) -> usize {
        self.rows.len()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed_clone(&self) -> Box<dyn NonPrimitiveTrace<KoalaBear>> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug)]
pub(super) struct ShaAirV2<F, const D: usize> {
    preprocessed: Vec<F>,
    min_height: usize,
}

impl<F: Field, const D: usize> ShaAirV2<F, D> {
    const fn width_v2() -> usize {
        ROW_FIELDS_V2 * D
    }

    pub(super) fn new(preprocessed: Vec<F>, min_height: usize) -> Self {
        Self {
            preprocessed,
            min_height: min_height.max(SHA_ROWS_V2),
        }
    }
}

impl<const D: usize> ShaAirV2<KoalaBear, D> {
    pub(super) fn trace_to_matrix(
        rows: &[ShaRowV2],
        min_height: usize,
    ) -> RowMajorMatrix<KoalaBear> {
        let mut values = KoalaBear::zero_vec(rows.len() * Self::width_v2());
        for (row_index, row) in rows.iter().enumerate() {
            for (field_index, value) in row.values[PUBLIC_FIELDS_V2..].iter().copied().enumerate() {
                values[row_index * Self::width_v2() + field_index * D] = value;
            }
        }
        let mut matrix = RowMajorMatrix::new(values, Self::width_v2());
        matrix.pad_to_min_power_of_two_height(min_height.max(SHA_ROWS_V2), KoalaBear::ZERO);
        matrix
    }
}

impl<F: Field, const D: usize> BaseAir<F> for ShaAirV2<F, D> {
    fn width(&self) -> usize {
        Self::width_v2()
    }

    fn num_public_values(&self) -> usize {
        PUBLIC_FIELDS_V2
    }

    fn preprocessed_width(&self) -> usize {
        PREPROCESSED_WIDTH_V2
    }

    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
        let mut matrix = RowMajorMatrix::from_flat_padded(
            self.preprocessed.clone(),
            PREPROCESSED_WIDTH_V2,
            F::ZERO,
        );
        matrix.pad_to_min_power_of_two_height(self.min_height, F::ZERO);
        Some(matrix)
    }

    fn main_next_row_columns(&self) -> Vec<usize> {
        (0..Self::width_v2()).collect()
    }

    fn preprocessed_next_row_columns(&self) -> Vec<usize> {
        Vec::new()
    }
}

fn field<AB, const D: usize>(row: &[AB::Var], offset: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    row[offset * D].into()
}

fn state_limb<AB, const D: usize>(row: &[AB::Var], word: usize, limb: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    field::<AB, D>(row, STATE_LIMBS_OFFSET_V2 + word * 2 + limb)
}

fn state_bit<AB, const D: usize>(row: &[AB::Var], slot: usize, bit: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    field::<AB, D>(row, STATE_BITS_OFFSET_V2 + slot * 32 + bit)
}

fn schedule_limb<AB, const D: usize>(row: &[AB::Var], word: usize, limb: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    field::<AB, D>(row, SCHEDULE_LIMBS_OFFSET_V2 + word * 2 + limb)
}

fn schedule_bit<AB, const D: usize>(row: &[AB::Var], slot: usize, bit: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    field::<AB, D>(row, SCHEDULE_BITS_OFFSET_V2 + slot * 32 + bit)
}

fn bits_limb<AB, const D: usize>(row: &[AB::Var], offset: usize, limb: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    let mut result = AB::Expr::ZERO;
    for bit in 0..16 {
        result += field::<AB, D>(row, offset + limb * 16 + bit) * AB::Expr::from_u64(1_u64 << bit);
    }
    result
}

fn carry<AB, const D: usize>(row: &[AB::Var], offset: usize, bits: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    let mut result = AB::Expr::ZERO;
    for bit in 0..bits {
        result += field::<AB, D>(row, offset + bit) * AB::Expr::from_u64(1_u64 << bit);
    }
    result
}

impl<AB, const D: usize> Air<AB> for ShaAirV2<AB::F, D>
where
    AB: AirBuilder,
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
        for offset in T1_BITS_OFFSET_V2..ROW_FIELDS_V2 {
            builder.assert_bool(field::<AB, D>(local, offset));
        }

        let mut selector_sum = AB::Expr::ZERO;
        for round in 0..SHA_ROWS_V2 {
            let selector = field::<AB, D>(local, SELECTOR_OFFSET_V2 + round);
            builder.assert_bool(selector.clone());
            selector_sum += selector;
        }
        builder.assert_one(selector_sum);
        {
            let mut first = builder.when_first_row();
            first.assert_one(field::<AB, D>(local, SELECTOR_OFFSET_V2));
            for round in 1..SHA_ROWS_V2 {
                first.assert_zero(field::<AB, D>(local, SELECTOR_OFFSET_V2 + round));
            }
            for word in 0..8 {
                for limb in 0..2 {
                    first.assert_eq(
                        state_limb::<AB, D>(local, word, limb),
                        public[INPUT_STATE_OFFSET_V2 + word * 2 + limb].clone(),
                    );
                }
            }
            for word in 0..16 {
                for limb in 0..2 {
                    first.assert_eq(
                        schedule_limb::<AB, D>(local, word, limb),
                        public[BLOCK_OFFSET_V2 + word * 2 + limb].clone(),
                    );
                }
            }
        }
        {
            let mut transition = builder.when_transition();
            transition.assert_zero(field::<AB, D>(next, SELECTOR_OFFSET_V2));
            for round in 1..SHA_ROWS_V2 {
                transition.assert_eq(
                    field::<AB, D>(next, SELECTOR_OFFSET_V2 + round),
                    field::<AB, D>(local, SELECTOR_OFFSET_V2 + round - 1),
                );
            }
        }
        builder
            .when_last_row()
            .assert_one(field::<AB, D>(local, SELECTOR_OFFSET_V2 + SHA_ROWS_V2 - 1));

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
                    transition.assert_eq(
                        schedule_limb::<AB, D>(next, word, limb),
                        schedule_limb::<AB, D>(local, word + 1, limb),
                    );
                }
            }
            transition.assert_eq(
                schedule_limb::<AB, D>(next, 15, 0),
                schedule_limb::<AB, D>(local, 0, 0)
                    + small_0[0].clone()
                    + schedule_limb::<AB, D>(local, 9, 0)
                    + small_1[0].clone()
                    - schedule_low_carry.clone() * radix.clone(),
            );
            transition.assert_eq(
                schedule_limb::<AB, D>(next, 15, 1),
                schedule_limb::<AB, D>(local, 0, 1)
                    + small_0[1].clone()
                    + schedule_limb::<AB, D>(local, 9, 1)
                    + small_1[1].clone()
                    + schedule_low_carry
                    - schedule_high_carry * radix.clone(),
            );
        }
        for bit in 0..4 {
            builder
                .when_last_row()
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
                transition.assert_eq(state_limb::<AB, D>(next, 0, limb), next_a[limb].clone());
                transition.assert_eq(
                    state_limb::<AB, D>(next, 1, limb),
                    state_limb::<AB, D>(local, 0, limb),
                );
                transition.assert_eq(
                    state_limb::<AB, D>(next, 2, limb),
                    state_limb::<AB, D>(local, 1, limb),
                );
                transition.assert_eq(
                    state_limb::<AB, D>(next, 3, limb),
                    state_limb::<AB, D>(local, 2, limb),
                );
                transition.assert_eq(state_limb::<AB, D>(next, 4, limb), next_e[limb].clone());
                transition.assert_eq(
                    state_limb::<AB, D>(next, 5, limb),
                    state_limb::<AB, D>(local, 4, limb),
                );
                transition.assert_eq(
                    state_limb::<AB, D>(next, 6, limb),
                    state_limb::<AB, D>(local, 5, limb),
                );
                transition.assert_eq(
                    state_limb::<AB, D>(next, 7, limb),
                    state_limb::<AB, D>(local, 6, limb),
                );
            }
            for bit in 0..16 {
                transition.assert_zero(field::<AB, D>(local, OUTPUT_CARRY_OFFSET_V2 + bit));
            }
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
        {
            let mut last = builder.when_last_row();
            for word in 0..8 {
                let low_carry = field::<AB, D>(local, OUTPUT_CARRY_OFFSET_V2 + word * 2);
                let high_carry = field::<AB, D>(local, OUTPUT_CARRY_OFFSET_V2 + word * 2 + 1);
                last.assert_eq(
                    public[OUTPUT_STATE_OFFSET_V2 + word * 2].clone(),
                    public[INPUT_STATE_OFFSET_V2 + word * 2].clone() + final_state[word][0].clone()
                        - low_carry.clone() * radix.clone(),
                );
                last.assert_eq(
                    public[OUTPUT_STATE_OFFSET_V2 + word * 2 + 1].clone(),
                    public[INPUT_STATE_OFFSET_V2 + word * 2 + 1].clone()
                        + final_state[word][1].clone()
                        + low_carry
                        - high_carry * radix.clone(),
                );
            }
        }
    }
}

impl BatchAir<Plonky3StarkConfigV2> for ShaAirV2<KoalaBear, 1> {}

pub(super) fn sha_npo_type() -> NpoTypeId {
    NpoTypeId::new(SHA_NPO_ID_V2)
}
