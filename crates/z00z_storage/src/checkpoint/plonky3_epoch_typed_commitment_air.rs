//! Direct KoalaBear AIR for the four typed checkpoint commitments per transition.
//!
//! The table proves the fixed commitment kind order, exact public digest
//! binding, consecutive event ordinals, and bounded transition coverage. It is
//! an internal table until the epoch Batch-STARK joins these rows to the
//! canonical event-stream table.

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

use super::plonky3_epoch_semantic_source_air::SOURCE_TYPED_PAYLOAD_BYTE_BUS_V2;
use super::{
    hardened_koala_bear_config, Plonky3StarkConfigV2, RecursiveCheckpointRejectReasonV2,
    EPOCH_CHUNK_BYTES_V2, EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2,
    TYPED_CHECKPOINT_COMMITMENT_VERSION_V2,
};
use crate::CheckpointError;

const NPO_ID_V2: &str = "z00z/plonky3/epoch-typed-commitment/v2";
const LINKED_NPO_ID_V2: &str = "z00z/plonky3/epoch-typed-commitment-linked/v2";
pub(super) const EXPECTED_TYPED_COMMITMENT_BUS_V2: &str =
    "z00z/plonky3/epoch-typed-commitment-expected/v2";
pub(super) const COMMITMENTS_PER_TRANSITION_V2: usize = 4;
pub(super) const MAX_TRANSITIONS_V2: usize = EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 as usize;
pub(super) const ROWS_V2: usize = MAX_TRANSITIONS_V2 * COMMITMENTS_PER_TRANSITION_V2;
pub(super) const STATEMENT_LIMBS_V2: usize = EPOCH_CHUNK_BYTES_V2 / core::mem::size_of::<u16>();
const DIGEST_LIMBS_V2: usize = 16;
const EXPECTED_DIGEST_LIMBS_V2: usize =
    MAX_TRANSITIONS_V2 * COMMITMENTS_PER_TRANSITION_V2 * DIGEST_LIMBS_V2;
pub(super) const PUBLIC_BINDING_COUNT_OFFSET_V2: usize =
    STATEMENT_LIMBS_V2 + EXPECTED_DIGEST_LIMBS_V2;
pub(super) const PUBLIC_FIELDS_V2: usize = PUBLIC_BINDING_COUNT_OFFSET_V2 + 1;

const HEADER_ACTIVE_OFFSET_V2: usize = 0;
const ACTIVE_OFFSET_V2: usize = 1;
const TRANSITION_SELECTOR_OFFSET_V2: usize = 2;
const KIND_SELECTOR_OFFSET_V2: usize = TRANSITION_SELECTOR_OFFSET_V2 + MAX_TRANSITIONS_V2;
const EVENT_ORDINAL_OFFSET_V2: usize = KIND_SELECTOR_OFFSET_V2 + COMMITMENTS_PER_TRANSITION_V2;
const EVENT_ORDINAL_BYTE_OFFSET_V2: usize = EVENT_ORDINAL_OFFSET_V2 + 4;
const EVENT_ORDINAL_BYTES_V2: usize = 8;
const PAYLOAD_VERSION_OFFSET_V2: usize = EVENT_ORDINAL_BYTE_OFFSET_V2 + EVENT_ORDINAL_BYTES_V2;
const PAYLOAD_KIND_OFFSET_V2: usize = PAYLOAD_VERSION_OFFSET_V2 + 1;
const DIGEST_OFFSET_V2: usize = PAYLOAD_KIND_OFFSET_V2 + 1;
const DIGEST_BYTE_OFFSET_V2: usize = DIGEST_OFFSET_V2 + DIGEST_LIMBS_V2;
const DIGEST_BYTES_V2: usize = 32;
pub(super) const RUNNING_ROW_COUNT_OFFSET_V2: usize = DIGEST_BYTE_OFFSET_V2 + DIGEST_BYTES_V2;
const ORDINAL_CARRY_OFFSET_V2: usize = RUNNING_ROW_COUNT_OFFSET_V2 + 1;
pub(super) const ROW_FIELDS_V2: usize = ORDINAL_CARRY_OFFSET_V2 + 4;
pub(super) const CALL_FIELDS_V2: usize = PUBLIC_FIELDS_V2 + ROW_FIELDS_V2;

const PUBLIC_ROW_COUNT_OFFSET_V2: usize = 21;

#[derive(Clone, Debug)]
pub(super) struct TypedCommitmentRowV2 {
    pub(super) values: Vec<KoalaBear>,
}

#[derive(Clone, Debug)]
pub(super) struct TypedCommitmentTraceV2 {
    pub(super) role: TypedCommitmentAirRoleV2,
    pub(super) rows: Vec<TypedCommitmentRowV2>,
}

impl NonPrimitiveTrace<KoalaBear> for TypedCommitmentTraceV2 {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TypedCommitmentAirRoleV2 {
    Standalone,
    LinkedConsumer,
}

impl TypedCommitmentAirRoleV2 {
    pub(super) fn npo_type(self) -> NpoTypeId {
        NpoTypeId::new(match self {
            Self::Standalone => NPO_ID_V2,
            Self::LinkedConsumer => LINKED_NPO_ID_V2,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct TypedCommitmentAirV2 {
    role: TypedCommitmentAirRoleV2,
}

impl TypedCommitmentAirV2 {
    const fn new(role: TypedCommitmentAirRoleV2) -> Self {
        Self { role }
    }

    fn trace_to_matrix(rows: &[TypedCommitmentRowV2]) -> RowMajorMatrix<KoalaBear> {
        let values = rows
            .iter()
            .flat_map(|row| row.values[PUBLIC_FIELDS_V2..].iter().copied())
            .collect();
        RowMajorMatrix::new(values, ROW_FIELDS_V2)
    }
}

impl<F: Field> BaseAir<F> for TypedCommitmentAirV2 {
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

fn expected_digest_offset(transition: usize, kind: usize, limb: usize) -> usize {
    STATEMENT_LIMBS_V2
        + ((transition * COMMITMENTS_PER_TRANSITION_V2 + kind) * DIGEST_LIMBS_V2)
        + limb
}

impl<AB> Air<AB> for TypedCommitmentAirV2
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
        builder.assert_bool(header_active.clone());
        builder.when_first_row().assert_one(header_active);
        builder
            .when_transition()
            .assert_zero(row(next, HEADER_ACTIVE_OFFSET_V2));

        let active = row(local, ACTIVE_OFFSET_V2);
        let next_active = row(next, ACTIVE_OFFSET_V2);
        builder.assert_bool(active.clone());

        let transition_selectors = (0..MAX_TRANSITIONS_V2)
            .map(|index| row(local, TRANSITION_SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        let next_transition_selectors = (0..MAX_TRANSITIONS_V2)
            .map(|index| row(next, TRANSITION_SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        let kind_selectors = (0..COMMITMENTS_PER_TRANSITION_V2)
            .map(|index| row(local, KIND_SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        let next_kind_selectors = (0..COMMITMENTS_PER_TRANSITION_V2)
            .map(|index| row(next, KIND_SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        for selector in transition_selectors.iter().chain(kind_selectors.iter()) {
            builder.assert_bool(selector.clone());
        }
        builder.assert_eq(
            transition_selectors
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value),
            active.clone(),
        );
        builder.assert_eq(
            kind_selectors
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value),
            active.clone(),
        );

        builder.assert_eq(
            row(local, PAYLOAD_VERSION_OFFSET_V2),
            active.clone() * AB::Expr::from_u64(u64::from(TYPED_CHECKPOINT_COMMITMENT_VERSION_V2)),
        );
        let encoded_kind = kind_selectors
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (kind, selector)| {
                sum + selector.clone() * AB::Expr::from_usize(kind + 1)
            });
        builder.assert_eq(row(local, PAYLOAD_KIND_OFFSET_V2), encoded_kind);

        let transition_index = transition_selectors
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (index, selector)| {
                sum + selector.clone() * AB::Expr::from_usize(index)
            });
        let typed_fields = || {
            let mut fields = Vec::with_capacity(2 + DIGEST_LIMBS_V2);
            fields.push(transition_index.clone());
            fields.push(row(local, PAYLOAD_KIND_OFFSET_V2));
            for limb in 0..DIGEST_LIMBS_V2 {
                fields.push(row(local, DIGEST_OFFSET_V2 + limb));
            }
            fields
        };
        let source_payload_fields = |payload_index: usize, payload_byte: AB::Expr| {
            let mut fields = Vec::with_capacity(2 + EVENT_ORDINAL_BYTES_V2 + 2);
            fields.push(transition_index.clone());
            for byte in 0..EVENT_ORDINAL_BYTES_V2 {
                fields.push(row(local, EVENT_ORDINAL_BYTE_OFFSET_V2 + byte));
            }
            fields.push(AB::Expr::from_usize(payload_index));
            fields.push(payload_byte);
            fields
        };
        match self.role {
            TypedCommitmentAirRoleV2::Standalone => {}
            TypedCommitmentAirRoleV2::LinkedConsumer => {
                builder.push_interaction(
                    EXPECTED_TYPED_COMMITMENT_BUS_V2,
                    typed_fields(),
                    -Count::bounded(active.clone(), 1),
                );
                let payload_bytes = [row(local, PAYLOAD_VERSION_OFFSET_V2)]
                    .into_iter()
                    .chain([row(local, PAYLOAD_KIND_OFFSET_V2)])
                    .chain(
                        (0..DIGEST_BYTES_V2).map(|byte| row(local, DIGEST_BYTE_OFFSET_V2 + byte)),
                    );
                for (payload_index, payload_byte) in payload_bytes.enumerate() {
                    builder.push_interaction(
                        SOURCE_TYPED_PAYLOAD_BYTE_BUS_V2,
                        source_payload_fields(payload_index, payload_byte),
                        -Count::bounded(active.clone(), 1),
                    );
                }
            }
        }

        for limb in 0..4 {
            builder.assert_zero(
                active.clone()
                    * (row(local, EVENT_ORDINAL_OFFSET_V2 + limb)
                        - row(local, EVENT_ORDINAL_BYTE_OFFSET_V2 + limb * 2)
                        - row(local, EVENT_ORDINAL_BYTE_OFFSET_V2 + limb * 2 + 1)
                            * AB::Expr::from_u64(256)),
            );
        }
        for limb in 0..DIGEST_LIMBS_V2 {
            builder.assert_zero(
                active.clone()
                    * (row(local, DIGEST_OFFSET_V2 + limb)
                        - row(local, DIGEST_BYTE_OFFSET_V2 + limb * 2)
                        - row(local, DIGEST_BYTE_OFFSET_V2 + limb * 2 + 1)
                            * AB::Expr::from_u64(256)),
            );
        }

        for (transition, transition_selector) in transition_selectors.iter().enumerate() {
            for (kind, kind_selector) in kind_selectors.iter().enumerate() {
                let selected = transition_selector.clone() * kind_selector.clone();
                for limb in 0..DIGEST_LIMBS_V2 {
                    builder.assert_zero(
                        selected.clone()
                            * (row(local, DIGEST_OFFSET_V2 + limb)
                                - public[expected_digest_offset(transition, kind, limb)].clone()),
                    );
                }
            }
        }

        let running = row(local, RUNNING_ROW_COUNT_OFFSET_V2);
        let next_running = row(next, RUNNING_ROW_COUNT_OFFSET_V2);
        {
            let mut first = builder.when_first_row();
            first.assert_one(active.clone());
            first.assert_one(transition_selectors[0].clone());
            first.assert_one(kind_selectors[0].clone());
            first.assert_one(running.clone());
            first.assert_eq(
                public[PUBLIC_ROW_COUNT_OFFSET_V2].clone(),
                public[PUBLIC_BINDING_COUNT_OFFSET_V2].clone()
                    * AB::Expr::from_u64(COMMITMENTS_PER_TRANSITION_V2 as u64),
            );
            for limb in 1..4 {
                first.assert_zero(public[PUBLIC_ROW_COUNT_OFFSET_V2 + limb].clone());
            }
        }

        {
            let mut transition = builder.when_transition();
            transition.assert_zero(next_active.clone() * (one.clone() - active.clone()));
            transition.assert_eq(next_running, running.clone() + next_active.clone());

            for kind in 0..(COMMITMENTS_PER_TRANSITION_V2 - 1) {
                transition.assert_zero(
                    kind_selectors[kind].clone() * (one.clone() - next_active.clone()),
                );
                transition.assert_zero(
                    kind_selectors[kind].clone()
                        * (next_kind_selectors[kind + 1].clone() - one.clone()),
                );
                for table in 0..MAX_TRANSITIONS_V2 {
                    transition.assert_zero(
                        transition_selectors[table].clone()
                            * kind_selectors[kind].clone()
                            * (next_transition_selectors[table].clone() - one.clone()),
                    );
                }
            }

            let last_kind = kind_selectors[COMMITMENTS_PER_TRANSITION_V2 - 1].clone();
            for table in 0..MAX_TRANSITIONS_V2 {
                let selected = transition_selectors[table].clone() * last_kind.clone();
                if table + 1 < MAX_TRANSITIONS_V2 {
                    transition.assert_zero(
                        selected.clone()
                            * (next_transition_selectors[table + 1].clone() - next_active.clone()),
                    );
                    transition.assert_zero(
                        selected * (next_kind_selectors[0].clone() - next_active.clone()),
                    );
                } else {
                    transition.assert_zero(selected * next_active.clone());
                }
            }

            let continues = kind_selectors[..COMMITMENTS_PER_TRANSITION_V2 - 1]
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value);
            let radix = AB::Expr::from_u64(65_536);
            let mut increment = one.clone();
            for limb in 0..4 {
                let carry = row(local, ORDINAL_CARRY_OFFSET_V2 + limb);
                transition.assert_bool(carry.clone());
                transition.assert_zero(
                    continues.clone()
                        * (row(next, EVENT_ORDINAL_OFFSET_V2 + limb)
                            - row(local, EVENT_ORDINAL_OFFSET_V2 + limb)
                            - increment
                            + carry.clone() * radix.clone()),
                );
                transition.assert_zero((one.clone() - continues.clone()) * carry.clone());
                increment = carry;
            }
            transition.assert_zero(continues.clone() * increment);
        }

        {
            let mut last = builder.when_last_row();
            last.assert_eq(
                running,
                public[PUBLIC_BINDING_COUNT_OFFSET_V2].clone()
                    * AB::Expr::from_u64(COMMITMENTS_PER_TRANSITION_V2 as u64),
            );
            last.assert_zero(
                active.clone()
                    * (kind_selectors[COMMITMENTS_PER_TRANSITION_V2 - 1].clone() - one.clone()),
            );
            for limb in 0..4 {
                last.assert_zero(row(local, ORDINAL_CARRY_OFFSET_V2 + limb));
            }
        }

        let inactive = one - active;
        for offset in TRANSITION_SELECTOR_OFFSET_V2..RUNNING_ROW_COUNT_OFFSET_V2 {
            builder.assert_zero(inactive.clone() * row(local, offset));
        }
    }
}

impl BatchAir<Plonky3StarkConfigV2> for TypedCommitmentAirV2 {}

#[derive(Clone, Copy, Debug)]
pub(super) struct TypedCommitmentProverV2 {
    role: TypedCommitmentAirRoleV2,
}

impl TypedCommitmentProverV2 {
    pub(super) const fn new(role: TypedCommitmentAirRoleV2) -> Self {
        Self { role }
    }

    fn batch_instance(
        &self,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces.non_primitive_trace::<TypedCommitmentTraceV2>(&self.role.npo_type())?;
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
            air: DynamicAirEntry::new(Box::new(TypedCommitmentAirV2::new(self.role))),
            trace: TypedCommitmentAirV2::trace_to_matrix(&trace.rows),
            public_values: trace.rows[0].values[..PUBLIC_FIELDS_V2].to_vec(),
            rows: ROWS_V2,
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for TypedCommitmentProverV2 {
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
            || entry.rows != ROWS_V2
            || entry.lanes != 1
            || entry.public_values.len() != PUBLIC_FIELDS_V2
            || entry.op_type != self.role.npo_type()
        {
            return Err("epoch typed-commitment table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(TypedCommitmentAirV2::new(
            self.role,
        ))))
    }
}

pub(super) fn npo_type() -> NpoTypeId {
    TypedCommitmentAirRoleV2::Standalone.npo_type()
}

fn direct_traces(
    rows: Vec<TypedCommitmentRowV2>,
    role: TypedCommitmentAirRoleV2,
) -> Traces<KoalaBear> {
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
            role.npo_type(),
            Box::new(TypedCommitmentTraceV2 { role, rows })
                as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        )]
        .into_iter()
        .collect(),
        tag_to_witness: Default::default(),
    }
}

pub(super) fn prove_rows(
    rows: Vec<TypedCommitmentRowV2>,
    expected_public: &[KoalaBear],
) -> Result<BatchStarkProof<Plonky3StarkConfigV2>, CheckpointError> {
    if rows.len() != ROWS_V2 || expected_public.len() != PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Canonical);
    }
    #[cfg(test)]
    p3_air::check_constraints(
        &TypedCommitmentAirV2::new(TypedCommitmentAirRoleV2::Standalone),
        &TypedCommitmentAirV2::trace_to_matrix(&rows),
        expected_public,
    );
    let traces = direct_traces(rows, TypedCommitmentAirRoleV2::Standalone);
    let table_packing = TablePacking::new(1, 1).with_min_trace_height(ROWS_V2);
    let mut prover = BatchStarkProver::new(hardened_koala_bear_config())
        .with_table_packing(table_packing.clone());
    prover.register_table_prover(Box::new(TypedCommitmentProverV2::new(
        TypedCommitmentAirRoleV2::Standalone,
    )));
    let proof = prover.prove_direct_tables(&traces).map_err(|error| {
        CheckpointError::Backend(format!(
            "Plonky3 epoch typed-commitment prove failed: {error}"
        ))
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
    verifier.register_table_prover(Box::new(TypedCommitmentProverV2::new(
        TypedCommitmentAirRoleV2::Standalone,
    )));
    verifier
        .verify_all_tables::<KoalaBear>(proof)
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 epoch typed-commitment actual verifier rejected proof: {error}"
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
