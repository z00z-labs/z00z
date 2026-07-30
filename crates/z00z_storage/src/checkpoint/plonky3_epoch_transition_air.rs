//! Direct KoalaBear AIR for one bounded epoch transition slice.
//!
//! The table binds every row to the exact transition records retained by the
//! closed work manifest, enforces ordinal/height/state continuity, accounts
//! for the complete source-event geometry, and supplies the expected typed
//! commitments to the cross-table LogUp bus.

use core::any::Any;

use p3_air::{Air, AirBuilder, BaseAir, WindowAccess};
use p3_circuit::ops::NpoTypeId;
use p3_circuit::tables::{NonPrimitiveTrace, Traces};
use p3_field::extension::BinomialExtensionField;
use p3_field::{Field, PrimeCharacteristicRing};
use p3_koala_bear::KoalaBear;
use p3_lookup::{Count, InteractionBuilder};
use p3_matrix::dense::RowMajorMatrix;
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchAir, BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking,
    TableProver,
};

use super::plonky3_epoch_typed_commitment_air::{
    COMMITMENTS_PER_TRANSITION_V2, EXPECTED_TYPED_COMMITMENT_BUS_V2,
};
use super::{Plonky3StarkConfigV2, EPOCH_CHUNK_BYTES_V2, EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2};

const NPO_ID_V2: &str = "z00z/plonky3/epoch-transition-linked/v2";
pub(super) const TRANSITION_TRACE_FRAMING_BUS_V2: &str =
    "z00z/plonky3/epoch-transition-trace-framing/v2";
pub(super) const BINDING_SLOTS_V2: usize = EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 as usize;
pub(super) const ROWS_V2: usize = 32;
const STATEMENT_LIMBS_V2: usize = EPOCH_CHUNK_BYTES_V2 / core::mem::size_of::<u16>();
const DIGEST_LIMBS_V2: usize = 16;
const U64_LIMBS_V2: usize = 4;

const BINDING_ORDINAL_OFFSET_V2: usize = 0;
const BINDING_HEIGHT_OFFSET_V2: usize = BINDING_ORDINAL_OFFSET_V2 + 2;
const BINDING_PRE_ROOT_OFFSET_V2: usize = BINDING_HEIGHT_OFFSET_V2 + U64_LIMBS_V2;
const BINDING_POST_ROOT_OFFSET_V2: usize = BINDING_PRE_ROOT_OFFSET_V2 + DIGEST_LIMBS_V2;
const BINDING_DIGEST_OFFSET_V2: usize = BINDING_POST_ROOT_OFFSET_V2 + DIGEST_LIMBS_V2;
const BINDING_EVENT_COUNT_OFFSET_V2: usize = BINDING_DIGEST_OFFSET_V2 + DIGEST_LIMBS_V2;
const BINDING_EVENT_BYTES_OFFSET_V2: usize = BINDING_EVENT_COUNT_OFFSET_V2 + U64_LIMBS_V2;
const BINDING_TYPED_OFFSET_V2: usize = BINDING_EVENT_BYTES_OFFSET_V2 + U64_LIMBS_V2;
pub(super) const BINDING_FIELDS_V2: usize =
    BINDING_TYPED_OFFSET_V2 + COMMITMENTS_PER_TRANSITION_V2 * DIGEST_LIMBS_V2;

const PUBLIC_BINDINGS_OFFSET_V2: usize = STATEMENT_LIMBS_V2;
const PUBLIC_BINDING_COUNT_OFFSET_V2: usize =
    PUBLIC_BINDINGS_OFFSET_V2 + BINDING_SLOTS_V2 * BINDING_FIELDS_V2;
const PUBLIC_EVENT_BYTES_OFFSET_V2: usize = PUBLIC_BINDING_COUNT_OFFSET_V2 + 1;
pub(super) const PUBLIC_FIELDS_V2: usize = PUBLIC_EVENT_BYTES_OFFSET_V2 + U64_LIMBS_V2;

const HEADER_ACTIVE_OFFSET_V2: usize = 0;
const ACTIVE_OFFSET_V2: usize = 1;
const SELECTOR_OFFSET_V2: usize = 2;
const BINDING_OFFSET_V2: usize = SELECTOR_OFFSET_V2 + BINDING_SLOTS_V2;
const RUNNING_COUNT_OFFSET_V2: usize = BINDING_OFFSET_V2 + BINDING_FIELDS_V2;
const RUNNING_EVENT_COUNT_OFFSET_V2: usize = RUNNING_COUNT_OFFSET_V2 + 1;
const RUNNING_EVENT_BYTES_OFFSET_V2: usize = RUNNING_EVENT_COUNT_OFFSET_V2 + U64_LIMBS_V2;
const ORDINAL_CARRY_OFFSET_V2: usize = RUNNING_EVENT_BYTES_OFFSET_V2 + U64_LIMBS_V2;
const HEIGHT_CARRY_OFFSET_V2: usize = ORDINAL_CARRY_OFFSET_V2 + 2;
const EVENT_COUNT_CARRY_OFFSET_V2: usize = HEIGHT_CARRY_OFFSET_V2 + U64_LIMBS_V2;
const EVENT_BYTES_CARRY_OFFSET_V2: usize = EVENT_COUNT_CARRY_OFFSET_V2 + U64_LIMBS_V2;
const ROW_FIELDS_V2: usize = EVENT_BYTES_CARRY_OFFSET_V2 + U64_LIMBS_V2;
pub(super) const CALL_FIELDS_V2: usize = PUBLIC_FIELDS_V2 + ROW_FIELDS_V2;

const STATEMENT_FIRST_TRANSITION_OFFSET_V2: usize = 11;
const STATEMENT_LAST_TRANSITION_OFFSET_V2: usize = 13;
const STATEMENT_ROW_COUNT_OFFSET_V2: usize = 21;
const STATEMENT_EVENT_COUNT_OFFSET_V2: usize = 29;
const STATEMENT_INPUT_ROOT_OFFSET_V2: usize = 49;
const STATEMENT_OUTPUT_ROOT_OFFSET_V2: usize = 65;

#[derive(Clone, Debug)]
pub(super) struct TransitionRowV2 {
    pub(super) values: Vec<KoalaBear>,
}

#[derive(Clone, Debug)]
pub(super) struct TransitionTraceV2 {
    pub(super) rows: Vec<TransitionRowV2>,
}

impl NonPrimitiveTrace<KoalaBear> for TransitionTraceV2 {
    fn op_type(&self) -> NpoTypeId {
        npo_type()
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

#[derive(Clone, Copy, Debug)]
struct TransitionAirV2;

impl TransitionAirV2 {
    fn trace_to_matrix(rows: &[TransitionRowV2]) -> RowMajorMatrix<KoalaBear> {
        RowMajorMatrix::new(
            rows.iter()
                .flat_map(|row| row.values[PUBLIC_FIELDS_V2..].iter().copied())
                .collect(),
            ROW_FIELDS_V2,
        )
    }
}

impl<F: Field> BaseAir<F> for TransitionAirV2 {
    fn width(&self) -> usize {
        ROW_FIELDS_V2
    }

    fn num_public_values(&self) -> usize {
        PUBLIC_FIELDS_V2
    }

    fn main_next_row_columns(&self) -> Vec<usize> {
        (0..ROW_FIELDS_V2).collect()
    }
}

fn field<AB: AirBuilder>(row: &[AB::Var], offset: usize) -> AB::Expr {
    row[offset].into()
}

fn public_binding_offset(row: usize, field: usize) -> usize {
    PUBLIC_BINDINGS_OFFSET_V2 + row * BINDING_FIELDS_V2 + field
}

impl<AB> Air<AB> for TransitionAirV2
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
        let row = |slice: &[AB::Var], offset| field::<AB>(slice, offset);
        let one = AB::Expr::ONE;
        let radix = AB::Expr::from_u64(65_536);

        let header = row(local, HEADER_ACTIVE_OFFSET_V2);
        builder.assert_bool(header.clone());
        builder.when_first_row().assert_one(header);
        builder
            .when_transition()
            .assert_zero(row(next, HEADER_ACTIVE_OFFSET_V2));

        let active = row(local, ACTIVE_OFFSET_V2);
        let next_active = row(next, ACTIVE_OFFSET_V2);
        builder.assert_bool(active.clone());
        let selectors = (0..BINDING_SLOTS_V2)
            .map(|index| row(local, SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        let next_selectors = (0..BINDING_SLOTS_V2)
            .map(|index| row(next, SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        for selector in &selectors {
            builder.assert_bool(selector.clone());
        }
        builder.assert_eq(
            selectors
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value),
            active.clone(),
        );
        for field_index in 0..BINDING_FIELDS_V2 {
            let expected =
                selectors
                    .iter()
                    .enumerate()
                    .fold(AB::Expr::ZERO, |sum, (index, selector)| {
                        sum + selector.clone()
                            * public[public_binding_offset(index, field_index)].clone()
                    });
            builder.assert_eq(row(local, BINDING_OFFSET_V2 + field_index), expected);
        }

        let transition_index = selectors
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (index, selector)| {
                sum + selector.clone() * AB::Expr::from_usize(index)
            });
        for kind in 0..COMMITMENTS_PER_TRANSITION_V2 {
            let mut fields = Vec::with_capacity(2 + DIGEST_LIMBS_V2);
            fields.push(transition_index.clone());
            fields.push(AB::Expr::from_usize(kind + 1));
            for limb in 0..DIGEST_LIMBS_V2 {
                fields.push(row(
                    local,
                    BINDING_OFFSET_V2 + BINDING_TYPED_OFFSET_V2 + kind * DIGEST_LIMBS_V2 + limb,
                ));
            }
            builder.push_interaction(
                EXPECTED_TYPED_COMMITMENT_BUS_V2,
                fields,
                Count::bounded(active.clone(), 1),
            );
        }
        let mut framing_fields = Vec::with_capacity(BINDING_EVENT_BYTES_OFFSET_V2 + U64_LIMBS_V2);
        for offset in BINDING_ORDINAL_OFFSET_V2..(BINDING_EVENT_BYTES_OFFSET_V2 + U64_LIMBS_V2) {
            framing_fields.push(row(local, BINDING_OFFSET_V2 + offset));
        }
        builder.push_interaction(
            TRANSITION_TRACE_FRAMING_BUS_V2,
            framing_fields,
            Count::bounded(active.clone(), 1),
        );

        let running = row(local, RUNNING_COUNT_OFFSET_V2);
        let next_running = row(next, RUNNING_COUNT_OFFSET_V2);
        {
            let mut first = builder.when_first_row();
            first.assert_one(active.clone());
            first.assert_one(selectors[0].clone());
            first.assert_one(running.clone());
            first.assert_eq(
                row(local, RUNNING_EVENT_COUNT_OFFSET_V2),
                row(local, BINDING_OFFSET_V2 + BINDING_EVENT_COUNT_OFFSET_V2),
            );
            first.assert_eq(
                row(local, RUNNING_EVENT_BYTES_OFFSET_V2),
                row(local, BINDING_OFFSET_V2 + BINDING_EVENT_BYTES_OFFSET_V2),
            );
            for limb in 1..U64_LIMBS_V2 {
                first.assert_eq(
                    row(local, RUNNING_EVENT_COUNT_OFFSET_V2 + limb),
                    row(
                        local,
                        BINDING_OFFSET_V2 + BINDING_EVENT_COUNT_OFFSET_V2 + limb,
                    ),
                );
                first.assert_eq(
                    row(local, RUNNING_EVENT_BYTES_OFFSET_V2 + limb),
                    row(
                        local,
                        BINDING_OFFSET_V2 + BINDING_EVENT_BYTES_OFFSET_V2 + limb,
                    ),
                );
            }
            first.assert_eq(
                row(local, BINDING_OFFSET_V2 + BINDING_ORDINAL_OFFSET_V2),
                public[STATEMENT_FIRST_TRANSITION_OFFSET_V2].clone(),
            );
            first.assert_zero(public[STATEMENT_FIRST_TRANSITION_OFFSET_V2 + 1].clone());
            for limb in 0..DIGEST_LIMBS_V2 {
                first.assert_eq(
                    row(local, BINDING_OFFSET_V2 + BINDING_PRE_ROOT_OFFSET_V2 + limb),
                    public[STATEMENT_INPUT_ROOT_OFFSET_V2 + limb].clone(),
                );
            }
        }

        {
            let mut transition = builder.when_transition();
            transition.assert_zero(next_active.clone() * (one.clone() - active.clone()));
            transition.assert_eq(next_running, running.clone() + next_active.clone());
            for index in 0..BINDING_SLOTS_V2 {
                if index + 1 < BINDING_SLOTS_V2 {
                    transition.assert_zero(
                        selectors[index].clone()
                            * (next_selectors[index + 1].clone() - next_active.clone()),
                    );
                } else {
                    transition.assert_zero(selectors[index].clone() * next_active.clone());
                }
            }
            let final_active = active.clone() * (one.clone() - next_active.clone());
            transition.assert_zero(
                final_active.clone()
                    * (row(local, BINDING_OFFSET_V2 + BINDING_ORDINAL_OFFSET_V2)
                        - public[STATEMENT_LAST_TRANSITION_OFFSET_V2].clone()),
            );
            for limb in 0..DIGEST_LIMBS_V2 {
                transition.assert_zero(
                    final_active.clone()
                        * (row(
                            local,
                            BINDING_OFFSET_V2 + BINDING_POST_ROOT_OFFSET_V2 + limb,
                        ) - public[STATEMENT_OUTPUT_ROOT_OFFSET_V2 + limb].clone()),
                );
            }
            for limb in 0..DIGEST_LIMBS_V2 {
                transition.assert_zero(
                    next_active.clone()
                        * (row(next, BINDING_OFFSET_V2 + BINDING_PRE_ROOT_OFFSET_V2 + limb)
                            - row(
                                local,
                                BINDING_OFFSET_V2 + BINDING_POST_ROOT_OFFSET_V2 + limb,
                            )),
                );
            }
            constrain_increment(
                &mut transition,
                local,
                next,
                BINDING_OFFSET_V2 + BINDING_ORDINAL_OFFSET_V2,
                2,
                ORDINAL_CARRY_OFFSET_V2,
                next_active.clone(),
                radix.clone(),
            );
            constrain_increment(
                &mut transition,
                local,
                next,
                BINDING_OFFSET_V2 + BINDING_HEIGHT_OFFSET_V2,
                U64_LIMBS_V2,
                HEIGHT_CARRY_OFFSET_V2,
                next_active.clone(),
                radix.clone(),
            );
            constrain_running_total(
                &mut transition,
                local,
                next,
                RUNNING_EVENT_COUNT_OFFSET_V2,
                BINDING_OFFSET_V2 + BINDING_EVENT_COUNT_OFFSET_V2,
                EVENT_COUNT_CARRY_OFFSET_V2,
                next_active.clone(),
                radix.clone(),
            );
            constrain_running_total(
                &mut transition,
                local,
                next,
                RUNNING_EVENT_BYTES_OFFSET_V2,
                BINDING_OFFSET_V2 + BINDING_EVENT_BYTES_OFFSET_V2,
                EVENT_BYTES_CARRY_OFFSET_V2,
                next_active,
                radix,
            );
        }

        {
            let mut last = builder.when_last_row();
            last.assert_zero(active.clone());
            last.assert_eq(running, public[PUBLIC_BINDING_COUNT_OFFSET_V2].clone());
            last.assert_eq(
                public[STATEMENT_ROW_COUNT_OFFSET_V2].clone(),
                public[PUBLIC_BINDING_COUNT_OFFSET_V2].clone(),
            );
            for limb in 1..U64_LIMBS_V2 {
                last.assert_zero(public[STATEMENT_ROW_COUNT_OFFSET_V2 + limb].clone());
            }
            for limb in 0..U64_LIMBS_V2 {
                last.assert_eq(
                    row(local, RUNNING_EVENT_COUNT_OFFSET_V2 + limb),
                    public[STATEMENT_EVENT_COUNT_OFFSET_V2 + limb].clone(),
                );
                last.assert_eq(
                    row(local, RUNNING_EVENT_BYTES_OFFSET_V2 + limb),
                    public[PUBLIC_EVENT_BYTES_OFFSET_V2 + limb].clone(),
                );
            }
            last.assert_zero(
                active.clone()
                    * (row(local, BINDING_OFFSET_V2 + BINDING_ORDINAL_OFFSET_V2)
                        - public[STATEMENT_LAST_TRANSITION_OFFSET_V2].clone()),
            );
            for limb in 0..DIGEST_LIMBS_V2 {
                last.assert_zero(
                    active.clone()
                        * (row(
                            local,
                            BINDING_OFFSET_V2 + BINDING_POST_ROOT_OFFSET_V2 + limb,
                        ) - public[STATEMENT_OUTPUT_ROOT_OFFSET_V2 + limb].clone()),
                );
            }
            for offset in ORDINAL_CARRY_OFFSET_V2..ROW_FIELDS_V2 {
                last.assert_zero(row(local, offset));
            }
        }
    }
}

fn constrain_increment<AB>(
    builder: &mut AB,
    local: &[AB::Var],
    next: &[AB::Var],
    value_offset: usize,
    limbs: usize,
    carry_offset: usize,
    enabled: AB::Expr,
    radix: AB::Expr,
) where
    AB: AirBuilder,
{
    let mut increment = AB::Expr::ONE;
    for limb in 0..limbs {
        let carry = field::<AB>(local, carry_offset + limb);
        builder.assert_bool(carry.clone());
        builder.assert_zero(
            enabled.clone()
                * (field::<AB>(next, value_offset + limb)
                    - field::<AB>(local, value_offset + limb)
                    - increment
                    + carry.clone() * radix.clone()),
        );
        builder.assert_zero((AB::Expr::ONE - enabled.clone()) * carry.clone());
        increment = carry;
    }
    builder.assert_zero(enabled * increment);
}

fn constrain_running_total<AB>(
    builder: &mut AB,
    local: &[AB::Var],
    next: &[AB::Var],
    running_offset: usize,
    addend_offset: usize,
    carry_offset: usize,
    enabled: AB::Expr,
    radix: AB::Expr,
) where
    AB: AirBuilder,
{
    let mut increment = AB::Expr::ZERO;
    for limb in 0..U64_LIMBS_V2 {
        let carry = field::<AB>(local, carry_offset + limb);
        builder.assert_bool(carry.clone());
        builder.assert_zero(
            field::<AB>(next, running_offset + limb)
                - field::<AB>(local, running_offset + limb)
                - enabled.clone() * field::<AB>(next, addend_offset + limb)
                - increment
                + carry.clone() * radix.clone(),
        );
        builder.assert_zero((AB::Expr::ONE - enabled.clone()) * carry.clone());
        increment = carry;
    }
    builder.assert_zero(enabled * increment);
}

impl BatchAir<Plonky3StarkConfigV2> for TransitionAirV2 {}

#[derive(Clone, Copy, Debug)]
pub(super) struct TransitionProverV2;

impl TransitionProverV2 {
    fn batch_instance(
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces.non_primitive_trace::<TransitionTraceV2>(&npo_type())?;
        if trace.rows.len() != ROWS_V2
            || trace
                .rows
                .iter()
                .any(|row| row.values.len() != CALL_FIELDS_V2)
        {
            return None;
        }
        Some(BatchTableInstance {
            op_type: npo_type(),
            air: DynamicAirEntry::new(Box::new(TransitionAirV2)),
            trace: TransitionAirV2::trace_to_matrix(&trace.rows),
            public_values: trace.rows[0].values[..PUBLIC_FIELDS_V2].to_vec(),
            rows: ROWS_V2,
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for TransitionProverV2 {
    fn op_type(&self) -> NpoTypeId {
        npo_type()
    }

    fn batch_instance_d1(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        Self::batch_instance(traces)
    }

    fn batch_instance_d2(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 2>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d4(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 4>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d6(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 6>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d8(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 8>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_air_from_table_entry(
        &self,
        _config: &Plonky3StarkConfigV2,
        degree: usize,
        circuit_extension_degree: u32,
        entry: &NonPrimitiveTableEntry<Plonky3StarkConfigV2>,
    ) -> Result<DynamicAirEntry<Plonky3StarkConfigV2>, String> {
        if degree != 1
            || circuit_extension_degree != 1
            || entry.op_type != npo_type()
            || entry.rows != ROWS_V2
            || entry.lanes != 1
            || entry.public_values.len() != PUBLIC_FIELDS_V2
        {
            return Err("epoch transition linked table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(TransitionAirV2)))
    }
}

pub(super) fn npo_type() -> NpoTypeId {
    NpoTypeId::new(NPO_ID_V2)
}

pub(super) fn check_constraints(rows: &[TransitionRowV2], expected_public: &[KoalaBear]) {
    p3_air::check_constraints(
        &TransitionAirV2,
        &TransitionAirV2::trace_to_matrix(rows),
        expected_public,
    );
}
