//! One-row-per-limb packed range AIR for streamed epoch transition bytes.
//!
//! This is a private child of `checkpoint::plonky3`. It proves exact byte
//! range and byte-count geometry without the former sixteen-row expansion per
//! u16 value. Content identity is joined to SHA and typed-event tables by the
//! complete chunk prover; this table alone is never frontier-admissible.

use core::any::Any;

use p3_air::{Air, AirBuilder, BaseAir, WindowAccess};
use p3_circuit::ops::NpoTypeId;
use p3_circuit::tables::{
    AluTrace, ConstTrace, NonPrimitiveTrace, PublicTrace, Traces, WitnessTrace,
};
use p3_field::extension::BinomialExtensionField;
use p3_field::{Field, PrimeCharacteristicRing};
use p3_koala_bear::KoalaBear;
use p3_matrix::dense::RowMajorMatrix;
use z00z_crypto::sha256_256;
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchAir, BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking,
    TableProver,
};
use z00z_plonky3_circuit_prover::{BatchStarkProof, BatchStarkProver};

#[cfg(test)]
use super::EpochSmokeMetricsV2;
use super::{
    decode_canonical_batch_proof_v2, encode_canonical_batch_proof_v2, hardened_koala_bear_config,
    EpochAirTableV2, EpochTraceChunkV2, Plonky3StarkConfigV2, RecursiveCheckpointRejectReasonV2,
    EPOCH_CHUNK_BYTES_V2, EPOCH_PACKED_RANGE_BYTES_PER_PROOF_V2,
};
use crate::CheckpointError;

const PACKED_RANGE_NPO_ID_V2: &str = "z00z/plonky3/epoch-packed-range/v2";
// The pinned FRI profile has log_final_poly_len=0 and log_blowup=2, so every
// table needs at least 2^(0 + 2 + 1) rows.
const PACKED_RANGE_MIN_ROWS_V2: usize = 8;
const STATEMENT_LIMBS_V2: usize = EPOCH_CHUNK_BYTES_V2 / core::mem::size_of::<u16>();
const PUBLIC_FIELDS_V2: usize = STATEMENT_LIMBS_V2;
const PUBLIC_ROW_COUNT_OFFSET_V2: usize = 21;

const ACTIVE_OFFSET_V2: usize = 0;
const SINGLE_BYTE_OFFSET_V2: usize = 1;
const BYTE_0_OFFSET_V2: usize = 2;
const BYTE_1_OFFSET_V2: usize = 3;
const BITS_OFFSET_V2: usize = 4;
const RUNNING_BYTES_OFFSET_V2: usize = BITS_OFFSET_V2 + 16;
const ROW_FIELDS_V2: usize = RUNNING_BYTES_OFFSET_V2 + 1;
const CALL_FIELDS_V2: usize = PUBLIC_FIELDS_V2 + ROW_FIELDS_V2;

#[derive(Clone, Debug)]
struct PackedRangeRowV2 {
    values: Vec<KoalaBear>,
}

#[derive(Clone, Debug)]
struct PackedRangeTraceV2 {
    rows: Vec<PackedRangeRowV2>,
}

impl NonPrimitiveTrace<KoalaBear> for PackedRangeTraceV2 {
    fn op_type(&self) -> NpoTypeId {
        packed_range_npo_type()
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
struct PackedRangeAirV2;

impl PackedRangeAirV2 {
    fn trace_to_matrix(rows: &[PackedRangeRowV2], min_height: usize) -> RowMajorMatrix<KoalaBear> {
        let values = rows
            .iter()
            .flat_map(|row| row.values[PUBLIC_FIELDS_V2..].iter().copied())
            .collect();
        let mut matrix = RowMajorMatrix::new(values, ROW_FIELDS_V2);
        matrix.pad_to_min_power_of_two_height(min_height.max(rows.len()), KoalaBear::ZERO);
        matrix
    }
}

impl<F: Field> BaseAir<F> for PackedRangeAirV2 {
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

impl<AB> Air<AB> for PackedRangeAirV2
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

        let active = field::<AB>(local, ACTIVE_OFFSET_V2);
        let next_active = field::<AB>(next, ACTIVE_OFFSET_V2);
        builder.assert_bool(active.clone());
        builder.when_first_row().assert_one(active.clone());
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
            .assert_eq(byte_0, reconstructed_0);
        builder
            .when(active.clone())
            .assert_eq(byte_1.clone(), reconstructed_1);
        builder
            .when(active.clone())
            .assert_zero(single.clone() * byte_1);

        let inactive = one.clone() - active.clone();
        builder.assert_zero(inactive.clone() * single.clone());
        builder.assert_zero(inactive.clone() * field::<AB>(local, BYTE_0_OFFSET_V2));
        builder.assert_zero(inactive.clone() * field::<AB>(local, BYTE_1_OFFSET_V2));
        for bit in 0..16 {
            builder.assert_zero(inactive.clone() * field::<AB>(local, BITS_OFFSET_V2 + bit));
        }

        let byte_count = AB::Expr::from_u64(2) - single.clone();
        let running = field::<AB>(local, RUNNING_BYTES_OFFSET_V2);
        let next_running = field::<AB>(next, RUNNING_BYTES_OFFSET_V2);
        builder
            .when_first_row()
            .assert_eq(running.clone(), byte_count);

        let next_single = field::<AB>(next, SINGLE_BYTE_OFFSET_V2);
        let next_byte_count = next_active.clone() * (AB::Expr::from_u64(2) - next_single);
        {
            let mut transition = builder.when_transition();
            transition.assert_zero(next_active.clone() * single);
            transition.assert_eq(next_running, running.clone() + next_byte_count);
        }

        let public_byte_count = public[PUBLIC_ROW_COUNT_OFFSET_V2].clone()
            + public[PUBLIC_ROW_COUNT_OFFSET_V2 + 1].clone() * AB::Expr::from_u64(65_536);
        builder.assert_zero(public[PUBLIC_ROW_COUNT_OFFSET_V2 + 2].clone());
        builder.assert_zero(public[PUBLIC_ROW_COUNT_OFFSET_V2 + 3].clone());
        builder
            .when_transition()
            .assert_zero((active - next_active) * (running.clone() - public_byte_count.clone()));
        builder
            .when_last_row()
            .assert_eq(running, public_byte_count);
    }
}

impl BatchAir<Plonky3StarkConfigV2> for PackedRangeAirV2 {}

#[derive(Clone, Copy, Debug)]
pub(super) struct PackedRangeProverV2;

impl PackedRangeProverV2 {
    fn batch_instance(
        &self,
        packing: &TablePacking,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces.non_primitive_trace::<PackedRangeTraceV2>(&packed_range_npo_type())?;
        if trace.rows.is_empty()
            || trace
                .rows
                .iter()
                .any(|row| row.values.len() != CALL_FIELDS_V2)
        {
            return None;
        }
        let min_height = packing.min_trace_height().max(trace.rows.len());
        Some(BatchTableInstance {
            op_type: packed_range_npo_type(),
            air: DynamicAirEntry::new(Box::new(PackedRangeAirV2)),
            trace: PackedRangeAirV2::trace_to_matrix(&trace.rows, min_height),
            public_values: trace.rows[0].values[..PUBLIC_FIELDS_V2].to_vec(),
            rows: trace.rows.len(),
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for PackedRangeProverV2 {
    fn op_type(&self) -> NpoTypeId {
        packed_range_npo_type()
    }

    fn batch_instance_d1(
        &self,
        _config: &Plonky3StarkConfigV2,
        packing: &TablePacking,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        self.batch_instance(packing, traces)
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
            || entry.rows == 0
            || entry.lanes != 1
            || entry.public_values.len() != PUBLIC_FIELDS_V2
        {
            return Err("epoch packed-range table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(PackedRangeAirV2)))
    }
}

fn packed_range_npo_type() -> NpoTypeId {
    NpoTypeId::new(PACKED_RANGE_NPO_ID_V2)
}

fn public_values(statement: &EpochTraceChunkV2) -> Result<Vec<KoalaBear>, CheckpointError> {
    if statement.canonical_bytes().len() != EPOCH_CHUNK_BYTES_V2 {
        return Err(CheckpointError::Canonical);
    }
    Ok(statement
        .canonical_bytes()
        .chunks_exact(2)
        .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]])))
        .collect())
}

fn rows(
    statement: &EpochTraceChunkV2,
    bytes: &[u8],
) -> Result<Vec<PackedRangeRowV2>, CheckpointError> {
    if bytes.is_empty() || bytes.len() > EPOCH_PACKED_RANGE_BYTES_PER_PROOF_V2 {
        return Err(CheckpointError::Limit);
    }
    let inputs = statement.inputs();
    if inputs.table != EpochAirTableV2::PackedRange
        || inputs.replica != 0
        || inputs.row_count != u64::try_from(bytes.len()).map_err(|_| CheckpointError::Limit)?
    {
        return Err(CheckpointError::Canonical);
    }
    let public = public_values(statement)?;
    let data_rows = bytes.len().div_ceil(2);
    let padded_rows = data_rows.max(PACKED_RANGE_MIN_ROWS_V2).next_power_of_two();
    let mut result = Vec::with_capacity(padded_rows);
    let mut running = 0_u64;
    for row_index in 0..padded_rows {
        let mut values = if row_index == 0 {
            public.clone()
        } else {
            vec![KoalaBear::ZERO; PUBLIC_FIELDS_V2]
        };
        let is_active = row_index < data_rows;
        values.push(KoalaBear::from_bool(is_active));
        if is_active {
            let byte_start = row_index.checked_mul(2).ok_or(CheckpointError::Overflow)?;
            let chunk_end = byte_start
                .checked_add(2)
                .map(|end| end.min(bytes.len()))
                .ok_or(CheckpointError::Overflow)?;
            let chunk = bytes
                .get(byte_start..chunk_end)
                .ok_or(CheckpointError::Invariant)?;
            let single = chunk.len() == 1;
            let byte_0 = chunk[0];
            let byte_1 = chunk.get(1).copied().unwrap_or(0);
            values.push(KoalaBear::from_bool(single));
            values.push(KoalaBear::from_u8(byte_0));
            values.push(KoalaBear::from_u8(byte_1));
            for bit in 0..8 {
                values.push(KoalaBear::from_bool((byte_0 >> bit) & 1 == 1));
            }
            for bit in 0..8 {
                values.push(KoalaBear::from_bool((byte_1 >> bit) & 1 == 1));
            }
            running = running
                .checked_add(u64::try_from(chunk.len()).map_err(|_| CheckpointError::Limit)?)
                .ok_or(CheckpointError::Overflow)?;
        } else {
            values.extend(core::iter::repeat_n(KoalaBear::ZERO, 19));
        }
        values.push(KoalaBear::from_u64(running));
        if values.len() != CALL_FIELDS_V2 {
            return Err(CheckpointError::Invariant);
        }
        result.push(PackedRangeRowV2 { values });
    }
    Ok(result)
}

fn verify_batch(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    expected_public: &[KoalaBear],
    table_packing: TablePacking,
) -> Result<(), CheckpointError> {
    let mut verifier =
        BatchStarkProver::new(hardened_koala_bear_config()).with_table_packing(table_packing);
    verifier.register_table_prover(Box::new(PackedRangeProverV2));
    verifier
        .verify_all_tables::<KoalaBear>(proof)
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 epoch packed-range actual verifier rejected proof: {error}"
            ))
        })?;
    let mut entries = proof
        .non_primitives
        .iter()
        .filter(|entry| entry.op_type == packed_range_npo_type());
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

fn prove(
    statement: &EpochTraceChunkV2,
    bytes: &[u8],
) -> Result<BatchStarkProof<Plonky3StarkConfigV2>, CheckpointError> {
    let rows = rows(statement, bytes)?;
    let expected_public = public_values(statement)?;
    #[cfg(test)]
    {
        let matrix = PackedRangeAirV2::trace_to_matrix(&rows, rows.len());
        p3_air::check_constraints(&PackedRangeAirV2, &matrix, &expected_public);
    }
    let table_packing = TablePacking::new(1, 1).with_min_trace_height(rows.len());
    let config = hardened_koala_bear_config();
    let traces: Traces<KoalaBear> = Traces {
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
            packed_range_npo_type(),
            Box::new(PackedRangeTraceV2 { rows }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        )]
        .into_iter()
        .collect(),
        tag_to_witness: Default::default(),
    };
    let mut prover =
        BatchStarkProver::new(config.clone()).with_table_packing(table_packing.clone());
    prover.register_table_prover(Box::new(PackedRangeProverV2));
    let proof = prover.prove_direct_tables(&traces).map_err(|error| {
        CheckpointError::Backend(format!("Plonky3 epoch packed-range prove failed: {error}"))
    })?;
    drop(traces);
    verify_batch(&proof, &expected_public, table_packing)?;
    Ok(proof)
}

/// Local-only actual proof for the packed byte/limb range stage.
///
/// It cannot be published or admitted into the epoch frontier without every
/// other direct table and their proof-bound cross-table links.
#[derive(Clone, Debug)]
pub struct Plonky3EpochPackedRangeV2 {
    statement: EpochTraceChunkV2,
    proof_digest: [u8; 32],
    proof_bytes: Vec<u8>,
}

impl Plonky3EpochPackedRangeV2 {
    #[must_use]
    pub const fn statement(&self) -> &EpochTraceChunkV2 {
        &self.statement
    }

    #[must_use]
    pub const fn proof_digest(&self) -> [u8; 32] {
        self.proof_digest
    }

    #[must_use]
    pub fn local_proof_bytes(&self) -> &[u8] {
        &self.proof_bytes
    }

    pub fn verify(&self) -> Result<(), CheckpointError> {
        let inputs = self.statement.inputs();
        if inputs.table != EpochAirTableV2::PackedRange
            || inputs.replica != 0
            || inputs.row_count == 0
            || inputs.row_count
                > u64::try_from(EPOCH_PACKED_RANGE_BYTES_PER_PROOF_V2)
                    .map_err(|_| CheckpointError::Limit)?
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        let expected_digest = sha256_256(
            "z00z.storage.checkpoint.plonky3.epoch-packed-range.v2",
            "actual_verified_table_proof",
            &[&self.statement.digest(), &self.proof_bytes],
        );
        if self.proof_digest == [0; 32] || self.proof_digest != expected_digest {
            return Err(CheckpointError::Canonical);
        }
        let proof = decode_canonical_batch_proof_v2(&self.proof_bytes)?;
        let rows = usize::try_from(inputs.row_count)
            .map_err(|_| CheckpointError::Limit)?
            .div_ceil(2)
            .max(PACKED_RANGE_MIN_ROWS_V2)
            .next_power_of_two();
        verify_batch(
            &proof,
            &public_values(&self.statement)?,
            TablePacking::new(1, 1).with_min_trace_height(rows),
        )
    }
}

pub(super) fn prove_epoch_packed_range(
    statement: EpochTraceChunkV2,
    bytes: &[u8],
) -> Result<Plonky3EpochPackedRangeV2, CheckpointError> {
    let proof = prove(&statement, bytes)?;
    let proof_bytes = encode_canonical_batch_proof_v2(&proof)?;
    let proof_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.epoch-packed-range.v2",
        "actual_verified_table_proof",
        &[&statement.digest(), &proof_bytes],
    );
    let artifact = Plonky3EpochPackedRangeV2 {
        statement,
        proof_digest,
        proof_bytes,
    };
    artifact.verify()?;
    Ok(artifact)
}

#[cfg(test)]
pub(super) fn prove_epoch_packed_range_smoke(
    statement: EpochTraceChunkV2,
    bytes: &[u8],
) -> Result<EpochSmokeMetricsV2, CheckpointError> {
    let parameter_digest = statement.inputs().parameter_digest;
    let trace_rows = rows(&statement, bytes)?.len();
    super::emit_resource_phase("proving");
    let artifact = prove_epoch_packed_range(statement, bytes)?;
    super::emit_resource_phase("proof_ready");
    let proof_bytes = artifact.local_proof_bytes().len();
    let mut mutated = artifact.clone();
    let mut proof = decode_canonical_batch_proof_v2(&mutated.proof_bytes)?;
    let table_count = proof.non_primitives.len();
    let entry = proof
        .non_primitives
        .iter_mut()
        .find(|entry| entry.op_type == packed_range_npo_type())
        .ok_or(CheckpointError::BackendVerificationFailed)?;
    let value = entry
        .public_values
        .first_mut()
        .ok_or(CheckpointError::BackendVerificationFailed)?;
    *value += KoalaBear::ONE;
    mutated.proof_bytes = encode_canonical_batch_proof_v2(&proof)?;
    mutated.proof_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.epoch-packed-range.v2",
        "actual_verified_table_proof",
        &[&mutated.statement.digest(), &mutated.proof_bytes],
    );
    super::emit_resource_phase("verifying");
    if mutated.verify().is_ok() {
        return Err(CheckpointError::BackendVerificationFailed);
    }
    super::emit_resource_phase("verify_complete");
    Ok(EpochSmokeMetricsV2 {
        parameter_digest,
        proof_bytes,
        trace_rows,
        input_items: bytes.len(),
        table_count,
    })
}
