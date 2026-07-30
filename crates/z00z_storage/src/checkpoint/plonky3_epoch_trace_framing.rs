//! Local artifact and row construction for the epoch trace-framing AIR.

use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;
use z00z_crypto::sha256_256;
use z00z_plonky3_circuit_prover::batch_stark_prover::TablePacking;

use super::plonky3_epoch_trace_framing_air as air;
#[cfg(test)]
use super::EpochSmokeMetricsV2;
use super::{
    decode_canonical_batch_proof_v2, encode_canonical_batch_proof_v2, EpochAirTableV2,
    EpochTraceChunkV2, EpochTransitionBindingV2, RecursiveCheckpointRejectReasonV2,
    EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2,
};
use crate::CheckpointError;

pub(super) fn public_values(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    event_bytes: u64,
) -> Result<Vec<KoalaBear>, CheckpointError> {
    if bindings.is_empty() || bindings.len() > air::ROWS_V2 {
        return Err(CheckpointError::Canonical);
    }
    let first_height = bindings.first().ok_or(CheckpointError::Canonical)?.height();
    let last_height = bindings.last().ok_or(CheckpointError::Canonical)?.height();
    public_values_from_metadata(statement, first_height, last_height, event_bytes)
}

fn public_values_from_metadata(
    statement: &EpochTraceChunkV2,
    first_height: u64,
    last_height: u64,
    event_bytes: u64,
) -> Result<Vec<KoalaBear>, CheckpointError> {
    if statement.canonical_bytes().len() != air::STATEMENT_LIMBS_V2 * 2 {
        return Err(CheckpointError::Canonical);
    }
    let mut values = Vec::with_capacity(air::PUBLIC_FIELDS_V2);
    values.extend(
        statement
            .canonical_bytes()
            .chunks_exact(2)
            .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
    );
    for value in [first_height, last_height, event_bytes] {
        values.extend(
            value
                .to_le_bytes()
                .chunks_exact(2)
                .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
        );
    }
    if values.len() != air::PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

fn extend_digest_limbs(values: &mut Vec<KoalaBear>, digest: [u8; 32]) {
    values.extend(
        digest
            .chunks_exact(2)
            .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
    );
}

fn extend_u64_limbs(values: &mut Vec<KoalaBear>, value: u64) {
    values.extend(
        value
            .to_le_bytes()
            .chunks_exact(2)
            .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
    );
}

fn u64_addition_carries(left: u64, right: u64) -> [bool; 4] {
    let left = left.to_le_bytes();
    let right = right.to_le_bytes();
    let mut carry = 0_u32;
    core::array::from_fn(|limb| {
        let offset = limb * 2;
        let left = u32::from(u16::from_le_bytes([left[offset], left[offset + 1]]));
        let right = u32::from(u16::from_le_bytes([right[offset], right[offset + 1]]));
        let sum = left + right + carry;
        carry = sum >> 16;
        carry == 1
    })
}

pub(super) fn rows(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    event_bytes: u64,
) -> Result<Vec<air::TraceFramingRowV2>, CheckpointError> {
    let public = public_values(statement, bindings, event_bytes)?;
    let mut rows = Vec::with_capacity(air::ROWS_V2);
    let mut running_count = 0_u32;
    let mut running_event_count = 0_u64;
    let mut running_event_bytes = 0_u64;
    for row_index in 0..air::ROWS_V2 {
        let mut values = if row_index == 0 {
            public.clone()
        } else {
            vec![KoalaBear::ZERO; public.len()]
        };
        values.push(KoalaBear::from_bool(row_index == 0));
        let binding = bindings.get(row_index).copied();
        values.push(KoalaBear::from_bool(binding.is_some()));
        if let Some(binding) = binding {
            running_count = running_count
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
            let inputs = binding.inputs();
            running_event_count = running_event_count
                .checked_add(inputs.event_count)
                .ok_or(CheckpointError::Overflow)?;
            running_event_bytes = running_event_bytes
                .checked_add(inputs.event_bytes)
                .ok_or(CheckpointError::Overflow)?;
            values.push(KoalaBear::from_u16(
                u16::try_from(inputs.ordinal & 0xffff).map_err(|_| CheckpointError::Limit)?,
            ));
            values.push(KoalaBear::from_u16(
                u16::try_from(inputs.ordinal >> 16).map_err(|_| CheckpointError::Limit)?,
            ));
            extend_u64_limbs(&mut values, inputs.height);
            extend_digest_limbs(&mut values, inputs.pre_settlement_root);
            extend_digest_limbs(&mut values, inputs.post_settlement_root);
            extend_digest_limbs(&mut values, binding.digest());
            extend_u64_limbs(&mut values, inputs.event_count);
            extend_u64_limbs(&mut values, inputs.event_bytes);
            values.push(KoalaBear::from_u32(running_count));
            extend_u64_limbs(&mut values, running_event_count);
            extend_u64_limbs(&mut values, running_event_bytes);

            let height_bytes = inputs.height.to_le_bytes();
            let height_limbs = height_bytes
                .chunks_exact(2)
                .map(|limb| u16::from_le_bytes([limb[0], limb[1]]));
            let mut carry = row_index + 1 < bindings.len();
            for limb in height_limbs {
                carry &= limb == u16::MAX;
                values.push(KoalaBear::from_bool(carry));
            }
            let next_inputs = bindings.get(row_index + 1).map(|binding| binding.inputs());
            for carry in next_inputs
                .map(|next| u64_addition_carries(running_event_count, next.event_count))
                .unwrap_or([false; 4])
            {
                values.push(KoalaBear::from_bool(carry));
            }
            for carry in next_inputs
                .map(|next| u64_addition_carries(running_event_bytes, next.event_bytes))
                .unwrap_or([false; 4])
            {
                values.push(KoalaBear::from_bool(carry));
            }
        } else {
            values.extend(core::iter::repeat_n(
                KoalaBear::ZERO,
                air::ROW_FIELDS_V2 - 2,
            ));
            let row_start = values
                .len()
                .checked_sub(air::ROW_FIELDS_V2)
                .ok_or(CheckpointError::Invariant)?;
            values[row_start + air::RUNNING_COUNT_OFFSET_V2] = KoalaBear::from_u32(running_count);
            for (offset, value) in running_event_count
                .to_le_bytes()
                .chunks_exact(2)
                .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]])))
                .enumerate()
            {
                values[row_start + air::RUNNING_EVENT_COUNT_OFFSET_V2 + offset] = value;
            }
            for (offset, value) in running_event_bytes
                .to_le_bytes()
                .chunks_exact(2)
                .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]])))
                .enumerate()
            {
                values[row_start + air::RUNNING_EVENT_BYTES_OFFSET_V2 + offset] = value;
            }
        }
        if values.len() != air::CALL_FIELDS_V2 {
            return Err(CheckpointError::Invariant);
        }
        rows.push(air::TraceFramingRowV2 { values });
    }
    Ok(rows)
}

/// Local-only actual Plonky3 proof for one direct epoch AIR table.
///
/// This artifact is an internal worker result. It is neither publishable nor
/// sufficient for frontier admission; a complete chunk receipt requires every
/// authority-mandated direct table and their cross-table accumulators.
#[derive(Clone, Debug)]
pub struct Plonky3EpochTraceFramingV2 {
    statement: EpochTraceChunkV2,
    first_height: u64,
    last_height: u64,
    event_bytes: u64,
    proof_digest: [u8; 32],
    proof_bytes: Vec<u8>,
}

impl Plonky3EpochTraceFramingV2 {
    #[must_use]
    pub const fn statement(&self) -> &EpochTraceChunkV2 {
        &self.statement
    }

    #[must_use]
    pub const fn first_height(&self) -> u64 {
        self.first_height
    }

    #[must_use]
    pub const fn last_height(&self) -> u64 {
        self.last_height
    }

    #[must_use]
    pub const fn event_bytes(&self) -> u64 {
        self.event_bytes
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
        if inputs.table != EpochAirTableV2::TraceFraming
            || inputs.replica != 0
            || inputs.row_count == 0
            || inputs.row_count > u64::from(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2)
            || self.first_height > self.last_height
            || self.event_bytes == 0
        {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        let expected_digest = sha256_256(
            "z00z.storage.checkpoint.plonky3.epoch-trace-framing.v2",
            "actual_verified_table_proof",
            &[
                &self.statement.digest(),
                &self.first_height.to_le_bytes(),
                &self.last_height.to_le_bytes(),
                &self.event_bytes.to_le_bytes(),
                &self.proof_bytes,
            ],
        );
        if self.proof_digest == [0; 32] || self.proof_digest != expected_digest {
            return Err(CheckpointError::Canonical);
        }
        let proof = decode_canonical_batch_proof_v2(&self.proof_bytes)?;
        air::verify_batch(
            &proof,
            &public_values_from_metadata(
                &self.statement,
                self.first_height,
                self.last_height,
                self.event_bytes,
            )?,
            TablePacking::new(1, 1).with_min_trace_height(air::ROWS_V2),
        )
    }
}

pub(super) fn prove_epoch_trace_framing(
    statement: EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    event_bytes: u64,
) -> Result<Plonky3EpochTraceFramingV2, CheckpointError> {
    let first_height = bindings.first().ok_or(CheckpointError::Canonical)?.height();
    let last_height = bindings.last().ok_or(CheckpointError::Canonical)?.height();
    let expected_public = public_values(&statement, bindings, event_bytes)?;
    let proof = air::prove_rows(rows(&statement, bindings, event_bytes)?, &expected_public)?;
    let proof_bytes = encode_canonical_batch_proof_v2(&proof)?;
    let proof_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.epoch-trace-framing.v2",
        "actual_verified_table_proof",
        &[
            &statement.digest(),
            &first_height.to_le_bytes(),
            &last_height.to_le_bytes(),
            &event_bytes.to_le_bytes(),
            &proof_bytes,
        ],
    );
    let artifact = Plonky3EpochTraceFramingV2 {
        statement,
        first_height,
        last_height,
        event_bytes,
        proof_digest,
        proof_bytes,
    };
    artifact.verify()?;
    Ok(artifact)
}

#[cfg(test)]
pub(super) fn prove_epoch_trace_framing_smoke(
    statement: EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    event_bytes: u64,
) -> Result<EpochSmokeMetricsV2, CheckpointError> {
    let parameter_digest = statement.inputs().parameter_digest;
    super::emit_resource_phase("proving");
    let artifact = prove_epoch_trace_framing(statement, bindings, event_bytes)?;
    super::emit_resource_phase("proof_ready");
    let proof_bytes = artifact.local_proof_bytes().len();
    let mut mutated = artifact.clone();
    let mut proof = decode_canonical_batch_proof_v2(&mutated.proof_bytes)?;
    let table_count = proof.non_primitives.len();
    let entry = proof
        .non_primitives
        .iter_mut()
        .find(|entry| entry.op_type == air::npo_type())
        .ok_or(CheckpointError::BackendVerificationFailed)?;
    let value = entry
        .public_values
        .first_mut()
        .ok_or(CheckpointError::BackendVerificationFailed)?;
    *value += KoalaBear::ONE;
    mutated.proof_bytes = encode_canonical_batch_proof_v2(&proof)?;
    mutated.proof_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.epoch-trace-framing.v2",
        "actual_verified_table_proof",
        &[
            &mutated.statement.digest(),
            &mutated.first_height.to_le_bytes(),
            &mutated.last_height.to_le_bytes(),
            &mutated.event_bytes.to_le_bytes(),
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
        trace_rows: air::ROWS_V2,
        input_items: bindings.len(),
        table_count,
    })
}
