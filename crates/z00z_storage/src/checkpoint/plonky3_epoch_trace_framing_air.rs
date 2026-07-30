//! Direct KoalaBear AIR for streamed epoch trace framing.
//!
//! The table proves transition order, state-root continuity, height continuity,
//! and event totals for one bounded epoch chunk. It consumes a native D1 trace
//! directly; no general circuit, witness bus, ALU table, or D4 lowering is
//! allocated.

use core::any::Any;

use p3_air::{Air, AirBuilder, BaseAir, WindowAccess};
use p3_circuit::ops::NpoTypeId;
use p3_circuit::tables::{
    AluTrace, ConstTrace, NonPrimitiveTrace, PublicTrace, Traces, WitnessTrace,
};
use p3_field::extension::BinomialExtensionField;
use p3_field::{Field, PrimeCharacteristicRing};
use p3_koala_bear::KoalaBear;
use p3_lookup::{Count, InteractionBuilder};
use p3_matrix::dense::RowMajorMatrix;
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchAir, BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking,
    TableProver,
};
use z00z_plonky3_circuit_prover::{BatchStarkProof, BatchStarkProver};

use super::plonky3_epoch_transition_air::TRANSITION_TRACE_FRAMING_BUS_V2;
use super::{
    hardened_koala_bear_config, Plonky3StarkConfigV2, RecursiveCheckpointRejectReasonV2,
    EPOCH_CHUNK_BYTES_V2,
};
use crate::CheckpointError;

const NPO_ID_V2: &str = "z00z/plonky3/epoch-trace-framing/v2";
const LINKED_NPO_ID_V2: &str = "z00z/plonky3/epoch-trace-framing-linked/v2";
pub(super) const ROWS_V2: usize = 8;
pub(super) const STATEMENT_LIMBS_V2: usize = EPOCH_CHUNK_BYTES_V2 / core::mem::size_of::<u16>();
const EXTRA_PUBLIC_LIMBS_V2: usize = 12;
pub(super) const PUBLIC_FIELDS_V2: usize = STATEMENT_LIMBS_V2 + EXTRA_PUBLIC_LIMBS_V2;

const HEADER_ACTIVE_OFFSET_V2: usize = 0;
const ACTIVE_OFFSET_V2: usize = 1;
const ORDINAL_OFFSET_V2: usize = 2;
const HEIGHT_OFFSET_V2: usize = ORDINAL_OFFSET_V2 + 2;
const PRE_ROOT_OFFSET_V2: usize = HEIGHT_OFFSET_V2 + 4;
const POST_ROOT_OFFSET_V2: usize = PRE_ROOT_OFFSET_V2 + 16;
const BINDING_DIGEST_OFFSET_V2: usize = POST_ROOT_OFFSET_V2 + 16;
const EVENT_COUNT_OFFSET_V2: usize = BINDING_DIGEST_OFFSET_V2 + 16;
const EVENT_BYTES_OFFSET_V2: usize = EVENT_COUNT_OFFSET_V2 + 4;
pub(super) const RUNNING_COUNT_OFFSET_V2: usize = EVENT_BYTES_OFFSET_V2 + 4;
pub(super) const RUNNING_EVENT_COUNT_OFFSET_V2: usize = RUNNING_COUNT_OFFSET_V2 + 1;
pub(super) const RUNNING_EVENT_BYTES_OFFSET_V2: usize = RUNNING_EVENT_COUNT_OFFSET_V2 + 4;
const HEIGHT_CARRY_OFFSET_V2: usize = RUNNING_EVENT_BYTES_OFFSET_V2 + 4;
const EVENT_COUNT_CARRY_OFFSET_V2: usize = HEIGHT_CARRY_OFFSET_V2 + 4;
const EVENT_BYTES_CARRY_OFFSET_V2: usize = EVENT_COUNT_CARRY_OFFSET_V2 + 4;
pub(super) const ROW_FIELDS_V2: usize = EVENT_BYTES_CARRY_OFFSET_V2 + 4;
pub(super) const CALL_FIELDS_V2: usize = PUBLIC_FIELDS_V2 + ROW_FIELDS_V2;

const PUBLIC_FIRST_TRANSITION_OFFSET_V2: usize = 11;
const PUBLIC_LAST_TRANSITION_OFFSET_V2: usize = 13;
const PUBLIC_ROW_COUNT_OFFSET_V2: usize = 21;
const PUBLIC_EVENT_COUNT_OFFSET_V2: usize = 29;
const PUBLIC_INPUT_STATE_ROOT_OFFSET_V2: usize = 49;
const PUBLIC_OUTPUT_STATE_ROOT_OFFSET_V2: usize = 65;
const PUBLIC_FIRST_HEIGHT_OFFSET_V2: usize = STATEMENT_LIMBS_V2;
const PUBLIC_LAST_HEIGHT_OFFSET_V2: usize = PUBLIC_FIRST_HEIGHT_OFFSET_V2 + 4;
const PUBLIC_EVENT_BYTES_OFFSET_V2: usize = PUBLIC_LAST_HEIGHT_OFFSET_V2 + 4;

#[derive(Clone, Debug)]
pub(super) struct TraceFramingRowV2 {
    pub(super) values: Vec<KoalaBear>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TraceFramingAirRoleV2 {
    Standalone,
    LinkedConsumer,
}

impl TraceFramingAirRoleV2 {
    pub(super) fn npo_type(self) -> NpoTypeId {
        NpoTypeId::new(match self {
            Self::Standalone => NPO_ID_V2,
            Self::LinkedConsumer => LINKED_NPO_ID_V2,
        })
    }
}

#[derive(Clone, Debug)]
pub(super) struct TraceFramingTraceV2 {
    pub(super) role: TraceFramingAirRoleV2,
    pub(super) rows: Vec<TraceFramingRowV2>,
}

impl NonPrimitiveTrace<KoalaBear> for TraceFramingTraceV2 {
    fn op_type(&self) -> NpoTypeId {
        self.role.npo_type()
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
struct TraceFramingAirV2 {
    role: TraceFramingAirRoleV2,
}

impl TraceFramingAirV2 {
    const fn new(role: TraceFramingAirRoleV2) -> Self {
        Self { role }
    }

    fn trace_to_matrix(rows: &[TraceFramingRowV2]) -> RowMajorMatrix<KoalaBear> {
        let values = rows
            .iter()
            .flat_map(|row| row.values[PUBLIC_FIELDS_V2..].iter().copied())
            .collect();
        RowMajorMatrix::new(values, ROW_FIELDS_V2)
    }
}

impl<F: Field> BaseAir<F> for TraceFramingAirV2 {
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

fn field<AB>(row: &[AB::Var], offset: usize) -> AB::Expr
where
    AB: AirBuilder,
{
    row[offset].into()
}

impl<AB> Air<AB> for TraceFramingAirV2
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
        let row = |slice: &[AB::Var], offset| field::<AB>(slice, offset);

        let header_active = row(local, HEADER_ACTIVE_OFFSET_V2);
        let next_header_active = row(next, HEADER_ACTIVE_OFFSET_V2);
        builder.assert_bool(header_active.clone());
        builder.when_first_row().assert_one(header_active);
        builder.when_transition().assert_zero(next_header_active);

        let active = row(local, ACTIVE_OFFSET_V2);
        let next_active = row(next, ACTIVE_OFFSET_V2);
        builder.assert_bool(active.clone());
        if self.role == TraceFramingAirRoleV2::LinkedConsumer {
            let mut fields = Vec::with_capacity(62);
            for offset in ORDINAL_OFFSET_V2..RUNNING_COUNT_OFFSET_V2 {
                fields.push(row(local, offset));
            }
            builder.push_interaction(
                TRANSITION_TRACE_FRAMING_BUS_V2,
                fields,
                -Count::bounded(active.clone(), 1),
            );
        }

        let ordinal_low = row(local, ORDINAL_OFFSET_V2);
        let ordinal_high = row(local, ORDINAL_OFFSET_V2 + 1);
        let next_ordinal_low = row(next, ORDINAL_OFFSET_V2);
        let next_ordinal_high = row(next, ORDINAL_OFFSET_V2 + 1);
        builder
            .when(active.clone())
            .assert_zero(ordinal_high.clone());

        let running = row(local, RUNNING_COUNT_OFFSET_V2);
        let next_running = row(next, RUNNING_COUNT_OFFSET_V2);
        {
            let mut first = builder.when_first_row();
            first.assert_one(active.clone());
            first.assert_one(running.clone());
            first.assert_eq(
                ordinal_low.clone(),
                public[PUBLIC_FIRST_TRANSITION_OFFSET_V2].clone(),
            );
            first.assert_zero(public[PUBLIC_FIRST_TRANSITION_OFFSET_V2 + 1].clone());
            for limb in 0..4 {
                first.assert_eq(
                    row(local, HEIGHT_OFFSET_V2 + limb),
                    public[PUBLIC_FIRST_HEIGHT_OFFSET_V2 + limb].clone(),
                );
            }
            for limb in 0..16 {
                first.assert_eq(
                    row(local, PRE_ROOT_OFFSET_V2 + limb),
                    public[PUBLIC_INPUT_STATE_ROOT_OFFSET_V2 + limb].clone(),
                );
            }
            for limb in 0..4 {
                first.assert_eq(
                    row(local, RUNNING_EVENT_COUNT_OFFSET_V2 + limb),
                    row(local, EVENT_COUNT_OFFSET_V2 + limb),
                );
                first.assert_eq(
                    row(local, RUNNING_EVENT_BYTES_OFFSET_V2 + limb),
                    row(local, EVENT_BYTES_OFFSET_V2 + limb),
                );
            }
        }

        {
            let mut transition = builder.when_transition();
            transition.assert_zero(next_active.clone() * (one.clone() - active.clone()));
            transition.assert_eq(next_running.clone(), running.clone() + next_active.clone());
            transition.assert_zero(
                next_active.clone()
                    * (next_ordinal_low.clone() - ordinal_low.clone() - one.clone()),
            );
            transition.assert_zero(next_active.clone() * next_ordinal_high);

            for limb in 0..16 {
                transition.assert_zero(
                    next_active.clone()
                        * (row(next, PRE_ROOT_OFFSET_V2 + limb)
                            - row(local, POST_ROOT_OFFSET_V2 + limb)),
                );
            }

            let radix = AB::Expr::from_u64(65_536);
            let mut increment = one.clone();
            for limb in 0..4 {
                let carry = row(local, HEIGHT_CARRY_OFFSET_V2 + limb);
                transition.assert_bool(carry.clone());
                transition.assert_zero(
                    next_active.clone()
                        * (row(next, HEIGHT_OFFSET_V2 + limb)
                            - row(local, HEIGHT_OFFSET_V2 + limb)
                            - increment
                            + carry.clone() * radix.clone()),
                );
                transition.assert_zero((one.clone() - next_active.clone()) * carry.clone());
                increment = carry;
            }
            transition.assert_zero(increment);

            let mut event_count_increment = AB::Expr::ZERO;
            let mut event_bytes_increment = AB::Expr::ZERO;
            for limb in 0..4 {
                let count_carry = row(local, EVENT_COUNT_CARRY_OFFSET_V2 + limb);
                let bytes_carry = row(local, EVENT_BYTES_CARRY_OFFSET_V2 + limb);
                transition.assert_bool(count_carry.clone());
                transition.assert_bool(bytes_carry.clone());
                transition.assert_zero(
                    row(next, RUNNING_EVENT_COUNT_OFFSET_V2 + limb)
                        - row(local, RUNNING_EVENT_COUNT_OFFSET_V2 + limb)
                        - next_active.clone() * row(next, EVENT_COUNT_OFFSET_V2 + limb)
                        - event_count_increment
                        + count_carry.clone() * radix.clone(),
                );
                transition.assert_zero(
                    row(next, RUNNING_EVENT_BYTES_OFFSET_V2 + limb)
                        - row(local, RUNNING_EVENT_BYTES_OFFSET_V2 + limb)
                        - next_active.clone() * row(next, EVENT_BYTES_OFFSET_V2 + limb)
                        - event_bytes_increment
                        + bytes_carry.clone() * radix.clone(),
                );
                transition.assert_zero((one.clone() - next_active.clone()) * count_carry.clone());
                transition.assert_zero((one.clone() - next_active.clone()) * bytes_carry.clone());
                event_count_increment = count_carry;
                event_bytes_increment = bytes_carry;
            }
            transition.assert_zero(event_count_increment);
            transition.assert_zero(event_bytes_increment);

            let end = active.clone() - next_active.clone();
            transition.assert_zero(
                end.clone()
                    * (ordinal_low.clone() - public[PUBLIC_LAST_TRANSITION_OFFSET_V2].clone()),
            );
            transition
                .assert_zero(end.clone() * public[PUBLIC_LAST_TRANSITION_OFFSET_V2 + 1].clone());
            for limb in 0..4 {
                transition.assert_zero(
                    end.clone()
                        * (row(local, HEIGHT_OFFSET_V2 + limb)
                            - public[PUBLIC_LAST_HEIGHT_OFFSET_V2 + limb].clone()),
                );
            }
            for limb in 0..16 {
                transition.assert_zero(
                    end.clone()
                        * (row(local, POST_ROOT_OFFSET_V2 + limb)
                            - public[PUBLIC_OUTPUT_STATE_ROOT_OFFSET_V2 + limb].clone()),
                );
            }
        }

        {
            let mut last = builder.when_last_row();
            last.assert_eq(running, public[PUBLIC_ROW_COUNT_OFFSET_V2].clone());
            for high_limb in 1..4 {
                last.assert_zero(public[PUBLIC_ROW_COUNT_OFFSET_V2 + high_limb].clone());
            }
            for limb in 0..4 {
                last.assert_eq(
                    row(local, RUNNING_EVENT_COUNT_OFFSET_V2 + limb),
                    public[PUBLIC_EVENT_COUNT_OFFSET_V2 + limb].clone(),
                );
                last.assert_eq(
                    row(local, RUNNING_EVENT_BYTES_OFFSET_V2 + limb),
                    public[PUBLIC_EVENT_BYTES_OFFSET_V2 + limb].clone(),
                );
            }
            last.assert_zero(
                active.clone() * (ordinal_low - public[PUBLIC_LAST_TRANSITION_OFFSET_V2].clone()),
            );
            for limb in 0..4 {
                last.assert_zero(row(local, HEIGHT_CARRY_OFFSET_V2 + limb));
                last.assert_zero(row(local, EVENT_COUNT_CARRY_OFFSET_V2 + limb));
                last.assert_zero(row(local, EVENT_BYTES_CARRY_OFFSET_V2 + limb));
                last.assert_zero(
                    active.clone()
                        * (row(local, HEIGHT_OFFSET_V2 + limb)
                            - public[PUBLIC_LAST_HEIGHT_OFFSET_V2 + limb].clone()),
                );
            }
            for limb in 0..16 {
                last.assert_zero(
                    active.clone()
                        * (row(local, POST_ROOT_OFFSET_V2 + limb)
                            - public[PUBLIC_OUTPUT_STATE_ROOT_OFFSET_V2 + limb].clone()),
                );
            }
        }

        let inactive = one - active;
        for offset in ORDINAL_OFFSET_V2..RUNNING_COUNT_OFFSET_V2 {
            builder.assert_zero(inactive.clone() * row(local, offset));
        }
    }
}

impl BatchAir<Plonky3StarkConfigV2> for TraceFramingAirV2 {}

#[derive(Clone, Copy, Debug)]
pub(super) struct TraceFramingProverV2 {
    role: TraceFramingAirRoleV2,
}

impl TraceFramingProverV2 {
    pub(super) const fn new(role: TraceFramingAirRoleV2) -> Self {
        Self { role }
    }

    fn batch_instance(
        &self,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces.non_primitive_trace::<TraceFramingTraceV2>(&self.role.npo_type())?;
        if trace.role != self.role
            || trace.rows.len() != ROWS_V2
            || trace
                .rows
                .iter()
                .any(|row| row.values.len() != CALL_FIELDS_V2)
        {
            return None;
        }
        Some(BatchTableInstance {
            op_type: self.role.npo_type(),
            air: DynamicAirEntry::new(Box::new(TraceFramingAirV2::new(self.role))),
            trace: TraceFramingAirV2::trace_to_matrix(&trace.rows),
            public_values: trace.rows[0].values[..PUBLIC_FIELDS_V2].to_vec(),
            rows: ROWS_V2,
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for TraceFramingProverV2 {
    fn op_type(&self) -> NpoTypeId {
        self.role.npo_type()
    }

    fn batch_instance_d1(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        self.batch_instance(traces)
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
            || entry.op_type != self.role.npo_type()
            || entry.rows != ROWS_V2
            || entry.lanes != 1
            || entry.public_values.len() != PUBLIC_FIELDS_V2
        {
            return Err("epoch trace-framing table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(TraceFramingAirV2::new(
            self.role,
        ))))
    }
}

pub(super) fn npo_type() -> NpoTypeId {
    TraceFramingAirRoleV2::Standalone.npo_type()
}

fn direct_traces(rows: Vec<TraceFramingRowV2>) -> Traces<KoalaBear> {
    Traces {
        witness_trace: WitnessTrace::new(Vec::new()),
        const_trace: ConstTrace {
            index: Vec::new(),
            values: Vec::new(),
        },
        public_trace: PublicTrace {
            index: Vec::new(),
            values: Vec::new(),
        },
        alu_trace: AluTrace::from_records(Vec::new()),
        non_primitive_traces: [(
            npo_type(),
            Box::new(TraceFramingTraceV2 {
                role: TraceFramingAirRoleV2::Standalone,
                rows,
            }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        )]
        .into_iter()
        .collect(),
        tag_to_witness: Default::default(),
    }
}

pub(super) fn prove_rows(
    rows: Vec<TraceFramingRowV2>,
    expected_public: &[KoalaBear],
) -> Result<BatchStarkProof<Plonky3StarkConfigV2>, CheckpointError> {
    if rows.len() != ROWS_V2 || expected_public.len() != PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Canonical);
    }
    #[cfg(test)]
    p3_air::check_constraints(
        &TraceFramingAirV2::new(TraceFramingAirRoleV2::Standalone),
        &TraceFramingAirV2::trace_to_matrix(&rows),
        expected_public,
    );
    let traces = direct_traces(rows);
    let table_packing = TablePacking::new(1, 1).with_min_trace_height(ROWS_V2);
    let mut prover = BatchStarkProver::new(hardened_koala_bear_config())
        .with_table_packing(table_packing.clone());
    prover.register_table_prover(Box::new(TraceFramingProverV2::new(
        TraceFramingAirRoleV2::Standalone,
    )));
    let proof = prover.prove_direct_tables(&traces).map_err(|error| {
        CheckpointError::Backend(format!("Plonky3 epoch trace-framing prove failed: {error}"))
    })?;
    drop(traces);
    verify_batch(&proof, expected_public, table_packing)?;
    Ok(proof)
}

pub(super) fn verify_batch(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    expected_public: &[KoalaBear],
    table_packing: TablePacking,
) -> Result<(), CheckpointError> {
    let mut verifier =
        BatchStarkProver::new(hardened_koala_bear_config()).with_table_packing(table_packing);
    verifier.register_table_prover(Box::new(TraceFramingProverV2::new(
        TraceFramingAirRoleV2::Standalone,
    )));
    verifier
        .verify_all_tables::<KoalaBear>(proof)
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 epoch trace-framing actual verifier rejected proof: {error}"
            ))
        })?;
    let mut entries = proof
        .non_primitives
        .iter()
        .filter(|entry| entry.op_type == npo_type());
    let actual_public = entries
        .next()
        .map(|entry| entry.public_values.as_slice())
        .ok_or(CheckpointError::BackendVerificationFailed)?;
    if entries.next().is_some() || actual_public != expected_public {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}
