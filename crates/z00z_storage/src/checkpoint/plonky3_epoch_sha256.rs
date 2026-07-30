//! Canonical prover and artifact boundary for the epoch SHA-256 AIR.

use p3_circuit::ops::NpoTypeId;
use p3_circuit::tables::{
    AluTrace, ConstTrace, NonPrimitiveTrace, PublicTrace, Traces, WitnessTrace,
};
use p3_field::extension::BinomialExtensionField;
use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;
use z00z_crypto::sha256_256;
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking, TableProver,
};
use z00z_plonky3_circuit_prover::{BatchStarkProof, BatchStarkProver};

use super::plonky3_epoch_sha256_air::{
    sha_npo_type, ShaAirV2, ShaTraceV2, CALL_FIELDS_V2, PUBLIC_FIELDS_V2, SHA_ROWS_V2,
};
use super::plonky3_epoch_sha256_witness::{compress, public_values, rows, words_bytes};
#[cfg(test)]
use super::EpochSmokeMetricsV2;
use super::{
    decode_canonical_batch_proof_v2, encode_canonical_batch_proof_v2, hardened_koala_bear_config,
    EpochAirTableV2, EpochTraceChunkV2, Plonky3StarkConfigV2, RecursiveCheckpointRejectReasonV2,
};
use crate::CheckpointError;

#[derive(Clone, Copy, Debug)]
pub(super) struct ShaProverV2;

impl ShaProverV2 {
    fn batch_instance(
        &self,
        packing: &TablePacking,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces.non_primitive_trace::<ShaTraceV2>(&sha_npo_type())?;
        if trace.rows.len() != SHA_ROWS_V2
            || trace
                .rows
                .iter()
                .any(|row| row.values.len() != CALL_FIELDS_V2)
        {
            return None;
        }
        let preprocessed = vec![KoalaBear::ONE; SHA_ROWS_V2];
        let min_height = packing.min_trace_height().max(SHA_ROWS_V2);
        let air = ShaAirV2::<KoalaBear, 1>::new(preprocessed, min_height);
        Some(BatchTableInstance {
            op_type: sha_npo_type(),
            air: DynamicAirEntry::new(Box::new(air)),
            trace: ShaAirV2::<KoalaBear, 1>::trace_to_matrix(&trace.rows, min_height),
            public_values: trace.rows[0].values[..PUBLIC_FIELDS_V2].to_vec(),
            rows: SHA_ROWS_V2,
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for ShaProverV2 {
    fn op_type(&self) -> NpoTypeId {
        sha_npo_type()
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
            || entry.rows != SHA_ROWS_V2
            || entry.lanes != 1
            || entry.public_values.len() != PUBLIC_FIELDS_V2
        {
            return Err("epoch SHA-256 table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(
            ShaAirV2::<KoalaBear, 1>::new(Vec::new(), entry.rows),
        )))
    }

    fn air_with_committed_preprocessed(
        &self,
        committed: Vec<KoalaBear>,
        min_height: usize,
        lanes: usize,
        circuit_extension_degree: u32,
    ) -> Option<DynamicAirEntry<Plonky3StarkConfigV2>> {
        (lanes == 1
            && circuit_extension_degree == 1
            && committed.len() == SHA_ROWS_V2
            && committed.iter().all(|&value| value == KoalaBear::ONE))
        .then(|| {
            DynamicAirEntry::new(Box::new(ShaAirV2::<KoalaBear, 1>::new(
                committed, min_height,
            )))
        })
    }
}

fn verify_batch(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    expected_public: &[KoalaBear],
    table_packing: TablePacking,
) -> Result<(), CheckpointError> {
    let mut verifier =
        BatchStarkProver::new(hardened_koala_bear_config()).with_table_packing(table_packing);
    verifier.register_table_prover(Box::new(ShaProverV2));
    verifier
        .verify_all_tables::<KoalaBear>(proof)
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 epoch SHA-256 actual verifier rejected proof: {error}"
            ))
        })?;
    let mut entries = proof
        .non_primitives
        .iter()
        .filter(|entry| entry.op_type == sha_npo_type());
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
    input: [u32; 8],
    block: &[u8; 64],
) -> Result<(BatchStarkProof<Plonky3StarkConfigV2>, [u32; 8]), CheckpointError> {
    let (rows, output) = rows(statement, input, block)?;
    let expected_public = public_values(statement, input, block, output)?;
    #[cfg(test)]
    {
        let air = ShaAirV2::<KoalaBear, 1>::new(vec![KoalaBear::ONE; SHA_ROWS_V2], SHA_ROWS_V2);
        let matrix = ShaAirV2::<KoalaBear, 1>::trace_to_matrix(&rows, SHA_ROWS_V2);
        p3_air::check_constraints(&air, &matrix, &expected_public);
    }
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
            sha_npo_type(),
            Box::new(ShaTraceV2 { rows }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        )]
        .into_iter()
        .collect(),
        tag_to_witness: Default::default(),
    };
    let table_packing = TablePacking::new(1, 1).with_min_trace_height(SHA_ROWS_V2);
    let mut prover = BatchStarkProver::new(hardened_koala_bear_config())
        .with_table_packing(table_packing.clone());
    prover.register_table_prover(Box::new(ShaProverV2));
    let proof = prover.prove_direct_tables(&traces).map_err(|error| {
        CheckpointError::Backend(format!("Plonky3 epoch SHA-256 prove failed: {error}"))
    })?;
    drop(traces);
    verify_batch(&proof, &expected_public, table_packing)?;
    Ok((proof, output))
}

/// Local actual proof for one direct SHA-256 compression block.
///
/// Publication and frontier admission require the complete table receipt and
/// proof-bound cross-table closure owned by the epoch chunk prover.
#[derive(Clone, Debug)]
pub struct Plonky3EpochSha256V2 {
    statement: EpochTraceChunkV2,
    input_state: [u32; 8],
    block: [u8; 64],
    output_state: [u32; 8],
    proof_digest: [u8; 32],
    proof_bytes: Vec<u8>,
}

impl Plonky3EpochSha256V2 {
    #[must_use]
    pub const fn statement(&self) -> &EpochTraceChunkV2 {
        &self.statement
    }

    #[must_use]
    pub const fn output_state(&self) -> [u32; 8] {
        self.output_state
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
        if inputs.table != EpochAirTableV2::Sha256
            || inputs.replica != 0
            || inputs.row_count != 1
            || self.output_state != compress(self.input_state, &self.block)
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        let input_bytes = words_bytes(self.input_state);
        let output_bytes = words_bytes(self.output_state);
        let expected_digest = sha256_256(
            "z00z.storage.checkpoint.plonky3.epoch-sha256.v2",
            "actual_verified_table_proof",
            &[
                &self.statement.digest(),
                &input_bytes,
                &self.block,
                &output_bytes,
                &self.proof_bytes,
            ],
        );
        if self.proof_digest == [0; 32] || self.proof_digest != expected_digest {
            return Err(CheckpointError::Canonical);
        }
        let proof = decode_canonical_batch_proof_v2(&self.proof_bytes)?;
        verify_batch(
            &proof,
            &public_values(
                &self.statement,
                self.input_state,
                &self.block,
                self.output_state,
            )?,
            TablePacking::new(1, 1).with_min_trace_height(SHA_ROWS_V2),
        )
    }
}

pub(super) fn prove_epoch_sha256(
    statement: EpochTraceChunkV2,
    input_state: [u32; 8],
    block: [u8; 64],
) -> Result<Plonky3EpochSha256V2, CheckpointError> {
    let (proof, output_state) = prove(&statement, input_state, &block)?;
    let proof_bytes = encode_canonical_batch_proof_v2(&proof)?;
    let input_bytes = words_bytes(input_state);
    let output_bytes = words_bytes(output_state);
    let proof_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.epoch-sha256.v2",
        "actual_verified_table_proof",
        &[
            &statement.digest(),
            &input_bytes,
            &block,
            &output_bytes,
            &proof_bytes,
        ],
    );
    let artifact = Plonky3EpochSha256V2 {
        statement,
        input_state,
        block,
        output_state,
        proof_digest,
        proof_bytes,
    };
    artifact.verify()?;
    Ok(artifact)
}

#[cfg(test)]
pub(super) fn prove_epoch_sha256_smoke(
    statement: EpochTraceChunkV2,
    input_state: [u32; 8],
    block: [u8; 64],
) -> Result<EpochSmokeMetricsV2, CheckpointError> {
    let parameter_digest = statement.inputs().parameter_digest;
    super::emit_resource_phase("proving");
    let artifact = prove_epoch_sha256(statement, input_state, block)?;
    super::emit_resource_phase("proof_ready");
    let proof_bytes = artifact.local_proof_bytes().len();
    let mut mutated = artifact.clone();
    let mut proof = decode_canonical_batch_proof_v2(&mutated.proof_bytes)?;
    let table_count = proof.non_primitives.len();
    let entry = proof
        .non_primitives
        .iter_mut()
        .find(|entry| entry.op_type == sha_npo_type())
        .ok_or(CheckpointError::BackendVerificationFailed)?;
    let value = entry
        .public_values
        .first_mut()
        .ok_or(CheckpointError::BackendVerificationFailed)?;
    *value += KoalaBear::ONE;
    mutated.proof_bytes = encode_canonical_batch_proof_v2(&proof)?;
    let input_bytes = words_bytes(mutated.input_state);
    let output_bytes = words_bytes(mutated.output_state);
    mutated.proof_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.epoch-sha256.v2",
        "actual_verified_table_proof",
        &[
            &mutated.statement.digest(),
            &input_bytes,
            &mutated.block,
            &output_bytes,
            &mutated.proof_bytes,
        ],
    );
    super::emit_resource_phase("verifying");
    if mutated.verify().is_ok() {
        return Err(CheckpointError::BackendVerificationFailed);
    }
    super::emit_resource_phase("verify_complete");
    Ok(EpochSmokeMetricsV2 {
        parameter_digest,
        proof_bytes,
        trace_rows: SHA_ROWS_V2,
        input_items: 1,
        table_count,
    })
}
