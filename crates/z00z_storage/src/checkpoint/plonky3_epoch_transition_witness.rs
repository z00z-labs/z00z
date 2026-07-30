//! Canonical public and private row construction for the epoch transition AIR.

use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;

use super::plonky3_epoch_transition_air::{
    TransitionRowV2, BINDING_FIELDS_V2, BINDING_SLOTS_V2, CALL_FIELDS_V2, PUBLIC_FIELDS_V2, ROWS_V2,
};
use super::{EpochTraceChunkV2, EpochTransitionBindingV2};
use crate::CheckpointError;

const U64_LIMBS_V2: usize = 4;

fn extend_u64(values: &mut Vec<KoalaBear>, value: u64) {
    values.extend(
        value
            .to_le_bytes()
            .chunks_exact(2)
            .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
    );
}

fn extend_digest(values: &mut Vec<KoalaBear>, digest: [u8; 32]) {
    values.extend(
        digest
            .chunks_exact(2)
            .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
    );
}

fn binding_fields(binding: EpochTransitionBindingV2) -> Vec<KoalaBear> {
    let inputs = binding.inputs();
    let mut fields = Vec::with_capacity(BINDING_FIELDS_V2);
    fields.push(KoalaBear::from_u16((inputs.ordinal & 0xffff) as u16));
    fields.push(KoalaBear::from_u16((inputs.ordinal >> 16) as u16));
    extend_u64(&mut fields, inputs.height);
    extend_digest(&mut fields, inputs.pre_settlement_root);
    extend_digest(&mut fields, inputs.post_settlement_root);
    extend_digest(&mut fields, binding.digest());
    extend_u64(&mut fields, inputs.event_count);
    extend_u64(&mut fields, inputs.event_bytes);
    for digest in binding.typed_commitment_digests() {
        extend_digest(&mut fields, digest);
    }
    fields
}

pub(super) fn public_values(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
) -> Result<Vec<KoalaBear>, CheckpointError> {
    if bindings.is_empty() || bindings.len() > BINDING_SLOTS_V2 {
        return Err(CheckpointError::Canonical);
    }
    let mut values = Vec::with_capacity(PUBLIC_FIELDS_V2);
    values.extend(
        statement
            .canonical_bytes()
            .chunks_exact(2)
            .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]]))),
    );
    for index in 0..BINDING_SLOTS_V2 {
        values.extend(
            bindings
                .get(index)
                .copied()
                .map(binding_fields)
                .unwrap_or_else(|| vec![KoalaBear::ZERO; BINDING_FIELDS_V2]),
        );
    }
    values.push(KoalaBear::from_usize(bindings.len()));
    let event_bytes = bindings.iter().try_fold(0_u64, |total, binding| {
        total
            .checked_add(binding.inputs().event_bytes)
            .ok_or(CheckpointError::Overflow)
    })?;
    extend_u64(&mut values, event_bytes);
    if values.len() != PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

fn addition_carries(left: u64, right: u64) -> [bool; U64_LIMBS_V2] {
    let left = left.to_le_bytes();
    let right = right.to_le_bytes();
    let mut carry = 0_u32;
    core::array::from_fn(|limb| {
        let offset = limb * 2;
        let sum = u32::from(u16::from_le_bytes([left[offset], left[offset + 1]]))
            + u32::from(u16::from_le_bytes([right[offset], right[offset + 1]]))
            + carry;
        carry = sum >> 16;
        carry == 1
    })
}

fn increment_carries(bytes: &[u8]) -> Vec<bool> {
    let mut carry = true;
    bytes
        .chunks_exact(2)
        .map(|limb| {
            carry &= u16::from_le_bytes([limb[0], limb[1]]) == u16::MAX;
            carry
        })
        .collect()
}

pub(super) fn rows(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
) -> Result<Vec<TransitionRowV2>, CheckpointError> {
    let public = public_values(statement, bindings)?;
    let mut rows = Vec::with_capacity(ROWS_V2);
    let mut running_count = 0_usize;
    let mut running_event_count = 0_u64;
    let mut running_event_bytes = 0_u64;
    for index in 0..ROWS_V2 {
        let mut values = if index == 0 {
            public.clone()
        } else {
            vec![KoalaBear::ZERO; PUBLIC_FIELDS_V2]
        };
        values.push(KoalaBear::from_bool(index == 0));
        let binding = bindings.get(index).copied();
        values.push(KoalaBear::from_bool(binding.is_some()));
        for selector in 0..BINDING_SLOTS_V2 {
            values.push(KoalaBear::from_bool(binding.is_some() && selector == index));
        }
        if let Some(binding) = binding {
            running_count += 1;
            let inputs = binding.inputs();
            values.extend(binding_fields(binding));
            running_event_count = running_event_count
                .checked_add(inputs.event_count)
                .ok_or(CheckpointError::Overflow)?;
            running_event_bytes = running_event_bytes
                .checked_add(inputs.event_bytes)
                .ok_or(CheckpointError::Overflow)?;
        } else {
            values.extend(core::iter::repeat_n(KoalaBear::ZERO, BINDING_FIELDS_V2));
        }
        values.push(KoalaBear::from_usize(running_count));
        extend_u64(&mut values, running_event_count);
        extend_u64(&mut values, running_event_bytes);
        if let Some(binding) = binding.filter(|_| index + 1 < bindings.len()) {
            values.extend(
                increment_carries(&binding.inputs().ordinal.to_le_bytes())
                    .into_iter()
                    .map(KoalaBear::from_bool),
            );
            values.extend(
                increment_carries(&binding.inputs().height.to_le_bytes())
                    .into_iter()
                    .map(KoalaBear::from_bool),
            );
            let next = bindings[index + 1].inputs();
            values.extend(
                addition_carries(running_event_count, next.event_count)
                    .into_iter()
                    .map(KoalaBear::from_bool),
            );
            values.extend(
                addition_carries(running_event_bytes, next.event_bytes)
                    .into_iter()
                    .map(KoalaBear::from_bool),
            );
        } else {
            values.extend(core::iter::repeat_n(KoalaBear::ZERO, 2 + U64_LIMBS_V2 * 3));
        }
        if values.len() != CALL_FIELDS_V2 {
            return Err(CheckpointError::Invariant);
        }
        rows.push(TransitionRowV2 { values });
    }
    Ok(rows)
}
