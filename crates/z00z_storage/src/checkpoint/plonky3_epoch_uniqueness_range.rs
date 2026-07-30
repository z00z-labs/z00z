//! Proof-bound packed range provider for uniqueness semantic bytes.

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

use super::plonky3_epoch_uniqueness_air::{MIN_ROWS_V2, RANGE_BUS_V2, STATEMENT_LIMBS_V2};
use super::{EpochTraceChunkV2, Plonky3StarkConfigV2};
use crate::CheckpointError;

const NPO_ID_V2: &str = "z00z/plonky3/epoch-uniqueness-packed-range/v2";
const QUERY_COUNT_LIMBS_V2: usize = 4;
pub(super) const PUBLIC_FIELDS_V2: usize = STATEMENT_LIMBS_V2 + QUERY_COUNT_LIMBS_V2;

const HEADER_ACTIVE_OFFSET_V2: usize = 0;
const ACTIVE_OFFSET_V2: usize = 1;
const SINGLE_BYTE_OFFSET_V2: usize = 2;
const BYTE_0_OFFSET_V2: usize = 3;
const BYTE_1_OFFSET_V2: usize = 4;
const BITS_OFFSET_V2: usize = 5;
const RUNNING_QUERY_COUNT_OFFSET_V2: usize = BITS_OFFSET_V2 + 16;
const ROW_FIELDS_V2: usize = RUNNING_QUERY_COUNT_OFFSET_V2 + 1;
const CALL_FIELDS_V2: usize = PUBLIC_FIELDS_V2 + ROW_FIELDS_V2;

#[derive(Clone, Copy, Debug)]
pub(super) struct UniquenessRangeQueryV2 {
    pub(super) byte_0: u8,
    pub(super) byte_1: u8,
    pub(super) single_byte: bool,
}

#[derive(Clone, Debug)]
pub(super) struct UniquenessRangeRowV2 {
    values: Vec<KoalaBear>,
}

#[derive(Clone, Debug)]
pub(super) struct UniquenessRangeTraceV2 {
    pub(super) rows: Vec<UniquenessRangeRowV2>,
}

impl NonPrimitiveTrace<KoalaBear> for UniquenessRangeTraceV2 {
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
struct UniquenessRangeAirV2;

impl UniquenessRangeAirV2 {
    fn trace_to_matrix(rows: &[UniquenessRangeRowV2]) -> RowMajorMatrix<KoalaBear> {
        RowMajorMatrix::new(
            rows.iter()
                .flat_map(|row| row.values[PUBLIC_FIELDS_V2..].iter().copied())
                .collect(),
            ROW_FIELDS_V2,
        )
    }
}

impl<F: Field> BaseAir<F> for UniquenessRangeAirV2 {
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

impl<AB> Air<AB> for UniquenessRangeAirV2
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

        let header_active = field::<AB>(local, HEADER_ACTIVE_OFFSET_V2);
        builder.assert_bool(header_active.clone());
        builder.when_first_row().assert_one(header_active);
        builder
            .when_transition()
            .assert_zero(field::<AB>(next, HEADER_ACTIVE_OFFSET_V2));

        let active = field::<AB>(local, ACTIVE_OFFSET_V2);
        let next_active = field::<AB>(next, ACTIVE_OFFSET_V2);
        builder.assert_bool(active.clone());
        builder
            .when_transition()
            .assert_zero(next_active.clone() * (one.clone() - active.clone()));

        let single = field::<AB>(local, SINGLE_BYTE_OFFSET_V2);
        builder.when(active.clone()).assert_bool(single.clone());
        let byte_0 = field::<AB>(local, BYTE_0_OFFSET_V2);
        let byte_1 = field::<AB>(local, BYTE_1_OFFSET_V2);
        let mut reconstructed_0 = AB::Expr::ZERO;
        let mut reconstructed_1 = AB::Expr::ZERO;
        for bit in 0..8 {
            let value = field::<AB>(local, BITS_OFFSET_V2 + bit);
            builder.when(active.clone()).assert_bool(value.clone());
            reconstructed_0 += value * AB::Expr::from_u64(1_u64 << bit);
        }
        for bit in 0..8 {
            let value = field::<AB>(local, BITS_OFFSET_V2 + 8 + bit);
            builder.when(active.clone()).assert_bool(value.clone());
            reconstructed_1 += value.clone() * AB::Expr::from_u64(1_u64 << bit);
            builder
                .when(active.clone())
                .assert_zero(single.clone() * value);
        }
        builder
            .when(active.clone())
            .assert_eq(byte_0.clone(), reconstructed_0);
        builder
            .when(active.clone())
            .assert_eq(byte_1.clone(), reconstructed_1);
        builder
            .when(active.clone())
            .assert_zero(single.clone() * byte_1.clone());

        builder.push_interaction(
            RANGE_BUS_V2,
            vec![byte_0, byte_1, single.clone()],
            Count::bounded(active.clone(), 1),
        );

        let running = field::<AB>(local, RUNNING_QUERY_COUNT_OFFSET_V2);
        let next_running = field::<AB>(next, RUNNING_QUERY_COUNT_OFFSET_V2);
        builder
            .when_first_row()
            .assert_eq(running.clone(), active.clone());
        builder
            .when_transition()
            .assert_eq(next_running, running.clone() + next_active);
        let count_offset = STATEMENT_LIMBS_V2;
        let public_count = public[count_offset].clone()
            + public[count_offset + 1].clone() * AB::Expr::from_u64(65_536);
        builder.assert_zero(public[count_offset + 2].clone());
        builder.assert_zero(public[count_offset + 3].clone());
        builder.when_last_row().assert_eq(running, public_count);

        let inactive = one - active;
        for offset in SINGLE_BYTE_OFFSET_V2..RUNNING_QUERY_COUNT_OFFSET_V2 {
            builder.assert_zero(inactive.clone() * field::<AB>(local, offset));
        }
    }
}

impl BatchAir<Plonky3StarkConfigV2> for UniquenessRangeAirV2 {}

#[derive(Clone, Copy, Debug)]
pub(super) struct UniquenessRangeProverV2;

impl UniquenessRangeProverV2 {
    fn batch_instance(
        &self,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces.non_primitive_trace::<UniquenessRangeTraceV2>(&npo_type())?;
        if trace.rows.len() < MIN_ROWS_V2
            || !trace.rows.len().is_power_of_two()
            || trace
                .rows
                .iter()
                .any(|row| row.values.len() != CALL_FIELDS_V2)
        {
            return None;
        }
        Some(BatchTableInstance {
            op_type: npo_type(),
            air: DynamicAirEntry::new(Box::new(UniquenessRangeAirV2)),
            trace: UniquenessRangeAirV2::trace_to_matrix(&trace.rows),
            public_values: trace.rows[0].values[..PUBLIC_FIELDS_V2].to_vec(),
            rows: trace.rows.len(),
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for UniquenessRangeProverV2 {
    fn op_type(&self) -> NpoTypeId {
        npo_type()
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
            || entry.op_type != npo_type()
            || entry.rows < MIN_ROWS_V2
            || !entry.rows.is_power_of_two()
            || entry.lanes != 1
            || entry.public_values.len() != PUBLIC_FIELDS_V2
        {
            return Err("epoch uniqueness range table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(UniquenessRangeAirV2)))
    }
}

pub(super) fn npo_type() -> NpoTypeId {
    NpoTypeId::new(NPO_ID_V2)
}

pub(super) fn public_values(
    statement: &EpochTraceChunkV2,
    query_count: usize,
) -> Result<Vec<KoalaBear>, CheckpointError> {
    let mut values = statement
        .canonical_bytes()
        .chunks_exact(2)
        .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]])))
        .collect::<Vec<_>>();
    let query_count = u64::try_from(query_count).map_err(|_| CheckpointError::Limit)?;
    values.extend(
        query_count
            .to_le_bytes()
            .chunks_exact(2)
            .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
    );
    if values.len() != PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

pub(super) fn rows(
    statement: &EpochTraceChunkV2,
    queries: &[UniquenessRangeQueryV2],
) -> Result<Vec<UniquenessRangeRowV2>, CheckpointError> {
    let public = public_values(statement, queries.len())?;
    let trace_rows = queries.len().max(MIN_ROWS_V2).next_power_of_two();
    let mut rows = Vec::with_capacity(trace_rows);
    let mut running = 0_usize;
    for index in 0..trace_rows {
        let mut values = if index == 0 {
            public.clone()
        } else {
            vec![KoalaBear::ZERO; PUBLIC_FIELDS_V2]
        };
        values.push(KoalaBear::from_bool(index == 0));
        let query = queries.get(index).copied();
        values.push(KoalaBear::from_bool(query.is_some()));
        if let Some(query) = query {
            running = running.checked_add(1).ok_or(CheckpointError::Overflow)?;
            values.push(KoalaBear::from_bool(query.single_byte));
            values.push(KoalaBear::from_u8(query.byte_0));
            values.push(KoalaBear::from_u8(query.byte_1));
            values.extend((0..8).map(|bit| KoalaBear::from_bool((query.byte_0 >> bit) & 1 == 1)));
            values.extend((0..8).map(|bit| KoalaBear::from_bool((query.byte_1 >> bit) & 1 == 1)));
        } else {
            values.extend(core::iter::repeat_n(KoalaBear::ZERO, 19));
        }
        values.push(KoalaBear::from_usize(running));
        if values.len() != CALL_FIELDS_V2 {
            return Err(CheckpointError::Invariant);
        }
        rows.push(UniquenessRangeRowV2 { values });
    }
    Ok(rows)
}

pub(super) fn check_constraints(rows: &[UniquenessRangeRowV2], expected_public: &[KoalaBear]) {
    p3_air::check_constraints(
        &UniquenessRangeAirV2,
        &UniquenessRangeAirV2::trace_to_matrix(rows),
        expected_public,
    );
}
