//! Public artifact boundary for the canonical epoch JMT update proof.

#[cfg(test)]
use p3_field::PrimeCharacteristicRing;
#[cfg(test)]
use p3_koala_bear::KoalaBear;
use z00z_crypto::sha256_256;
use z00z_plonky3_circuit_prover::batch_stark_prover::TablePacking;

#[cfg(test)]
use super::plonky3_epoch_jmt_witness::{check_constraints_for_rows, rows};
use super::plonky3_epoch_jmt_witness::{prove, public_values, verify_batch};

#[cfg(test)]
use super::plonky3_epoch_jmt_air::jmt_npo_type;
use super::plonky3_epoch_jmt_air::JMT_MIN_ROWS_V2;
#[cfg(test)]
use super::EpochSmokeMetricsV2;
use super::{
    decode_canonical_batch_proof_v2, encode_canonical_batch_proof_v2, EpochAirTableV2,
    EpochTraceChunkV2, RecursiveCheckpointRejectReasonV2,
};
use crate::settlement::JMT_CIRCUIT_HEADER_BYTES_V2;
use crate::CheckpointError;

#[cfg(test)]
pub(super) fn check_epoch_jmt_update_constraints(
    statement: &EpochTraceChunkV2,
    header: &[u8; JMT_CIRCUIT_HEADER_BYTES_V2],
    records: &[Vec<u8>],
) -> Result<(), CheckpointError> {
    let rows = rows(statement, header, records)?;
    let expected_public = public_values(statement, header)?;
    check_constraints_for_rows(&rows, &expected_public);
    Ok(())
}

/// Local actual proof for the direct JMT update/path table.
///
/// The canonical header is verifier-visible. Record bytes and every raw-SHA
/// request stay in the committed trace for the later cross-table closure.
#[derive(Clone, Debug)]
pub struct Plonky3EpochJmtUpdateV2 {
    statement: EpochTraceChunkV2,
    header: [u8; JMT_CIRCUIT_HEADER_BYTES_V2],
    proof_digest: [u8; 32],
    proof_bytes: Vec<u8>,
}

impl Plonky3EpochJmtUpdateV2 {
    #[must_use]
    pub const fn statement(&self) -> &EpochTraceChunkV2 {
        &self.statement
    }

    #[must_use]
    pub const fn header(&self) -> &[u8; JMT_CIRCUIT_HEADER_BYTES_V2] {
        &self.header
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
        let update_count = u32::from_le_bytes(
            self.header[35..39]
                .try_into()
                .map_err(|_| CheckpointError::Canonical)?,
        );
        if inputs.table != EpochAirTableV2::JmtUpdate
            || inputs.replica != 0
            || inputs.row_count == 0
            || update_count == 0
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        let expected_digest = sha256_256(
            "z00z.storage.checkpoint.plonky3.epoch-jmt-update.v2",
            "actual_verified_table_proof",
            &[&self.statement.digest(), &self.header, &self.proof_bytes],
        );
        if self.proof_digest == [0; 32] || self.proof_digest != expected_digest {
            return Err(CheckpointError::Canonical);
        }
        let proof = decode_canonical_batch_proof_v2(&self.proof_bytes)?;
        let rows = usize::try_from(inputs.row_count)
            .map_err(|_| CheckpointError::Limit)?
            .max(JMT_MIN_ROWS_V2)
            .next_power_of_two();
        verify_batch(
            &proof,
            &public_values(&self.statement, &self.header)?,
            TablePacking::new(1, 1).with_min_trace_height(rows),
        )
    }
}

pub(super) fn prove_epoch_jmt_update(
    statement: EpochTraceChunkV2,
    header: [u8; JMT_CIRCUIT_HEADER_BYTES_V2],
    records: &[Vec<u8>],
) -> Result<Plonky3EpochJmtUpdateV2, CheckpointError> {
    let proof = prove(&statement, &header, records)?;
    let proof_bytes = encode_canonical_batch_proof_v2(&proof)?;
    let proof_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.epoch-jmt-update.v2",
        "actual_verified_table_proof",
        &[&statement.digest(), &header, &proof_bytes],
    );
    let artifact = Plonky3EpochJmtUpdateV2 {
        statement,
        header,
        proof_digest,
        proof_bytes,
    };
    artifact.verify()?;
    Ok(artifact)
}

#[cfg(test)]
pub(super) fn prove_epoch_jmt_update_smoke(
    statement: EpochTraceChunkV2,
    header: [u8; JMT_CIRCUIT_HEADER_BYTES_V2],
    records: &[Vec<u8>],
) -> Result<EpochSmokeMetricsV2, CheckpointError> {
    let parameter_digest = statement.inputs().parameter_digest;
    let trace_rows = rows(&statement, &header, records)?.len();
    super::emit_resource_phase("proving");
    let artifact = prove_epoch_jmt_update(statement, header, records)?;
    super::emit_resource_phase("proof_ready");
    let proof_bytes = artifact.local_proof_bytes().len();
    let mut mutated = artifact.clone();
    let mut proof = decode_canonical_batch_proof_v2(&mutated.proof_bytes)?;
    let table_count = proof.non_primitives.len();
    let entry = proof
        .non_primitives
        .iter_mut()
        .find(|entry| entry.op_type == jmt_npo_type())
        .ok_or(CheckpointError::BackendVerificationFailed)?;
    let value = entry
        .public_values
        .first_mut()
        .ok_or(CheckpointError::BackendVerificationFailed)?;
    *value += KoalaBear::ONE;
    mutated.proof_bytes = encode_canonical_batch_proof_v2(&proof)?;
    mutated.proof_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.epoch-jmt-update.v2",
        "actual_verified_table_proof",
        &[
            &mutated.statement.digest(),
            &mutated.header,
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
        trace_rows,
        input_items: records.len(),
        table_count,
    })
}
