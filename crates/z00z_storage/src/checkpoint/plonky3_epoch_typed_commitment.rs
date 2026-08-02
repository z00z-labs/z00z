//! Artifact and witness construction for the typed checkpoint commitment AIR.

use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;
use z00z_crypto::sha256_256;
use z00z_plonky3_circuit_prover::batch_stark_prover::TablePacking;

use super::plonky3_epoch_event_stream::transition_event_stream;
use super::plonky3_epoch_typed_commitment_air as air;
use super::{
    decode_canonical_batch_proof_v2, decode_typed_checkpoint_commitment,
    encode_canonical_batch_proof_v2, EpochAirTableV2, EpochPreparedTransitionV2, EpochTraceChunkV2,
    EpochTransitionBindingV2, RecursiveCheckpointRejectReasonV2, TypedCheckpointCommitmentKindV2,
    TYPED_CHECKPOINT_COMMITMENT_VERSION_V2,
};
use crate::CheckpointError;

const PROOF_DOMAIN_V2: &str = "z00z.storage.checkpoint.plonky3.epoch-typed-commitment.v2";
const PROOF_LABEL_V2: &str = "actual_verified_table_proof";

#[derive(Clone, Copy, Debug)]
struct TypedCommitmentWitnessV2 {
    event_ordinal: u64,
    kind: TypedCheckpointCommitmentKindV2,
    digest: [u8; 32],
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

pub(super) fn public_values(
    statement: &EpochTraceChunkV2,
    commitments: &[[[u8; 32]; air::COMMITMENTS_PER_TRANSITION_V2]],
) -> Result<Vec<KoalaBear>, CheckpointError> {
    validate_metadata(statement, commitments)?;
    let mut values = Vec::with_capacity(air::PUBLIC_FIELDS_V2);
    values.extend(
        statement
            .canonical_bytes()
            .chunks_exact(2)
            .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
    );
    for transition in 0..air::MAX_TRANSITIONS_V2 {
        for kind in 0..air::COMMITMENTS_PER_TRANSITION_V2 {
            extend_digest_limbs(
                &mut values,
                commitments
                    .get(transition)
                    .map(|digests| digests[kind])
                    .unwrap_or([0; 32]),
            );
        }
    }
    values.push(KoalaBear::from_usize(commitments.len()));
    if values.len() != air::PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

fn validate_metadata(
    statement: &EpochTraceChunkV2,
    commitments: &[[[u8; 32]; air::COMMITMENTS_PER_TRANSITION_V2]],
) -> Result<(), CheckpointError> {
    let inputs = statement.inputs();
    let binding_count = u32::try_from(commitments.len()).map_err(|_| CheckpointError::Limit)?;
    let expected_row_start = u64::from(inputs.first_transition)
        .checked_mul(air::COMMITMENTS_PER_TRANSITION_V2 as u64)
        .ok_or(CheckpointError::Overflow)?;
    let expected_row_count = u64::from(binding_count)
        .checked_mul(air::COMMITMENTS_PER_TRANSITION_V2 as u64)
        .ok_or(CheckpointError::Overflow)?;
    let covered_transitions = inputs
        .last_transition
        .checked_sub(inputs.first_transition)
        .and_then(|span| span.checked_add(1))
        .ok_or(CheckpointError::Overflow)?;
    if statement.canonical_bytes().len() != air::STATEMENT_LIMBS_V2 * 2
        || inputs.table != EpochAirTableV2::TypedCommitment
        || inputs.replica != 0
        || binding_count == 0
        || usize::try_from(binding_count).map_err(|_| CheckpointError::Limit)?
            > air::MAX_TRANSITIONS_V2
        || covered_transitions != binding_count
        || inputs.row_start != expected_row_start
        || inputs.row_count != expected_row_count
        || commitments
            .iter()
            .flat_map(|digests| digests.iter())
            .any(|digest| *digest == [0; 32])
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

fn transition_witness(
    prepared: &EpochPreparedTransitionV2,
    binding: EpochTransitionBindingV2,
) -> Result<[TypedCommitmentWitnessV2; air::COMMITMENTS_PER_TRANSITION_V2], CheckpointError> {
    if prepared.binding() != binding {
        return Err(CheckpointError::Invariant);
    }
    let stream = transition_event_stream(&prepared.material)?;
    let records = stream.typed_commitment_records().collect::<Vec<_>>();
    let records: [_; air::COMMITMENTS_PER_TRANSITION_V2] = records
        .try_into()
        .map_err(|_| CheckpointError::EventOrder)?;
    let expected_digests = binding.typed_commitment_digests();
    let mut witness = Vec::with_capacity(air::COMMITMENTS_PER_TRANSITION_V2);
    for index in 0..air::COMMITMENTS_PER_TRANSITION_V2 {
        let record = records[index];
        let (kind, digest) = decode_typed_checkpoint_commitment(record.payload())?;
        if kind != TypedCheckpointCommitmentKindV2::ALL[index]
            || digest != expected_digests[index]
            || (index != 0
                && record.ordinal()
                    != records[index - 1]
                        .ordinal()
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?)
        {
            return Err(CheckpointError::Canonical);
        }
        witness.push(TypedCommitmentWitnessV2 {
            event_ordinal: record.ordinal(),
            kind,
            digest,
        });
    }
    witness.try_into().map_err(|_| CheckpointError::Invariant)
}

fn increment_carries(value: u64) -> [bool; 4] {
    let limbs = value
        .to_le_bytes()
        .chunks_exact(2)
        .map(|limb| u16::from_le_bytes([limb[0], limb[1]]))
        .collect::<Vec<_>>();
    let mut carry = true;
    core::array::from_fn(|index| {
        carry &= limbs[index] == u16::MAX;
        carry
    })
}

pub(super) fn rows(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
) -> Result<Vec<air::TypedCommitmentRowV2>, CheckpointError> {
    if bindings.len() != prepared.len() {
        return Err(CheckpointError::Invariant);
    }
    let commitments = bindings
        .iter()
        .map(EpochTransitionBindingV2::typed_commitment_digests)
        .collect::<Vec<_>>();
    let public = public_values(statement, &commitments)?;
    let mut witness = Vec::with_capacity(
        bindings
            .len()
            .checked_mul(air::COMMITMENTS_PER_TRANSITION_V2)
            .ok_or(CheckpointError::Overflow)?,
    );
    for (prepared, binding) in prepared.iter().zip(bindings.iter().copied()) {
        witness.extend(transition_witness(prepared, binding)?);
    }
    let mut result = Vec::with_capacity(air::ROWS_V2);
    let mut running_count = 0_usize;
    for row_index in 0..air::ROWS_V2 {
        let mut values = if row_index == 0 {
            public.clone()
        } else {
            vec![KoalaBear::ZERO; public.len()]
        };
        values.push(KoalaBear::from_bool(row_index == 0));
        let record = witness.get(row_index).copied();
        values.push(KoalaBear::from_bool(record.is_some()));
        if let Some(record) = record {
            running_count = running_count
                .checked_add(1)
                .ok_or(CheckpointError::Overflow)?;
            let transition_index = row_index / air::COMMITMENTS_PER_TRANSITION_V2;
            let kind_index = row_index % air::COMMITMENTS_PER_TRANSITION_V2;
            for index in 0..air::MAX_TRANSITIONS_V2 {
                values.push(KoalaBear::from_bool(index == transition_index));
            }
            for index in 0..air::COMMITMENTS_PER_TRANSITION_V2 {
                values.push(KoalaBear::from_bool(index == kind_index));
            }
            extend_u64_limbs(&mut values, record.event_ordinal);
            values.extend(record.event_ordinal.to_le_bytes().map(KoalaBear::from_u8));
            values.push(KoalaBear::from_u16(u16::from(
                TYPED_CHECKPOINT_COMMITMENT_VERSION_V2,
            )));
            values.push(KoalaBear::from_u16(u16::from(record.kind as u8)));
            extend_digest_limbs(&mut values, record.digest);
            values.extend(record.digest.map(KoalaBear::from_u8));
            values.push(KoalaBear::from_usize(running_count));
            for carry in if kind_index + 1 < air::COMMITMENTS_PER_TRANSITION_V2 {
                increment_carries(record.event_ordinal)
            } else {
                [false; 4]
            } {
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
            values[row_start + air::RUNNING_ROW_COUNT_OFFSET_V2] =
                KoalaBear::from_usize(running_count);
        }
        if values.len() != air::CALL_FIELDS_V2 {
            return Err(CheckpointError::Invariant);
        }
        result.push(air::TypedCommitmentRowV2 { values });
    }
    Ok(result)
}

/// Internal actual-verified proof of the four typed checkpoint commitments.
///
/// This object is not publishable and cannot enter the epoch frontier until a
/// single Batch-STARK also proves its lookup against the canonical event table.
#[derive(Clone, Debug)]
pub struct Plonky3EpochTypedCommitmentV2 {
    statement: EpochTraceChunkV2,
    commitments: Vec<[[u8; 32]; air::COMMITMENTS_PER_TRANSITION_V2]>,
    proof_digest: [u8; 32],
    proof_bytes: Vec<u8>,
}

impl Plonky3EpochTypedCommitmentV2 {
    #[must_use]
    pub const fn statement(&self) -> &EpochTraceChunkV2 {
        &self.statement
    }

    #[must_use]
    pub fn binding_count(&self) -> usize {
        self.commitments.len()
    }

    #[must_use]
    pub fn local_proof_bytes(&self) -> &[u8] {
        &self.proof_bytes
    }

    pub fn verify(&self) -> Result<(), CheckpointError> {
        let expected_digest = proof_digest(&self.statement, &self.commitments, &self.proof_bytes)?;
        if self.proof_digest == [0; 32] || self.proof_digest != expected_digest {
            return Err(CheckpointError::Canonical);
        }
        let proof = decode_canonical_batch_proof_v2(&self.proof_bytes)?;
        air::verify_batch(
            &proof,
            &public_values(&self.statement, &self.commitments)?,
            TablePacking::new(1, 1).with_min_trace_height(air::ROWS_V2),
        )
    }
}

fn proof_digest(
    statement: &EpochTraceChunkV2,
    commitments: &[[[u8; 32]; air::COMMITMENTS_PER_TRANSITION_V2]],
    proof_bytes: &[u8],
) -> Result<[u8; 32], CheckpointError> {
    let mut commitment_bytes = Vec::with_capacity(
        commitments
            .len()
            .checked_mul(air::COMMITMENTS_PER_TRANSITION_V2 * 32)
            .ok_or(CheckpointError::Overflow)?,
    );
    for digest in commitments.iter().flat_map(|digests| digests.iter()) {
        commitment_bytes.extend_from_slice(digest);
    }
    Ok(sha256_256(
        PROOF_DOMAIN_V2,
        PROOF_LABEL_V2,
        &[&statement.digest(), &commitment_bytes, proof_bytes],
    ))
}

pub(super) fn prove_epoch_typed_commitments(
    statement: EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
    prepared: &[EpochPreparedTransitionV2],
) -> Result<Plonky3EpochTypedCommitmentV2, CheckpointError> {
    let commitments = bindings
        .iter()
        .map(EpochTransitionBindingV2::typed_commitment_digests)
        .collect::<Vec<_>>();
    let expected_public = public_values(&statement, &commitments)?;
    let proof = air::prove_rows(rows(&statement, bindings, prepared)?, &expected_public)?;
    let proof_bytes = encode_canonical_batch_proof_v2(&proof)?;
    let proof_digest = proof_digest(&statement, &commitments, &proof_bytes)?;
    let artifact = Plonky3EpochTypedCommitmentV2 {
        statement,
        commitments,
        proof_digest,
        proof_bytes,
    };
    artifact.verify()?;
    Ok(artifact)
}
