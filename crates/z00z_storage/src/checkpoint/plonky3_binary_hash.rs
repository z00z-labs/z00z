//! Binary W32 hashing constraints for the private Plonky3 recursion adapter.

use p3_circuit::ops::{PermCall, PermConfig, Poseidon2Config};
use p3_circuit::{CircuitBuilder, CircuitBuilderError};
use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;
use p3_recursion::pcs::MerkleCapTargets;
use p3_recursion::{Target, VerificationError};

use super::{Plonky3ChallengeV2, PLONKY3_MMCS_DIGEST_ELEMS_V2};

pub(super) type CommitmentV2 = MerkleCapTargets<KoalaBear, PLONKY3_MMCS_DIGEST_ELEMS_V2>;
pub(super) type DigestV2 = [Target; 4];

pub(super) fn hash_base_values(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    values: &[Target],
) -> Result<DigestV2, VerificationError> {
    let chunk_count = values.len().div_ceil(24);
    let mut previous: Option<Vec<Option<Target>>> = None;
    let mut final_outputs = Vec::new();
    let chunks = values.chunks(24);
    for (chunk_index, chunk) in chunks.enumerate() {
        let mut inputs = vec![None; 8];
        for extension_index in 0..6 {
            let start = extension_index * 4;
            let count = chunk.len().saturating_sub(start).min(4);
            if count == 0 {
                continue;
            }
            let mut coefficients = Vec::with_capacity(4);
            coefficients.extend_from_slice(&chunk[start..start + count]);
            fill_coefficients(
                circuit,
                &mut coefficients,
                previous.as_deref(),
                extension_index,
            )?;
            inputs[extension_index] = Some(pack_coefficients(circuit, &coefficients)?);
        }
        let is_last = chunk_index + 1 == chunk_count;
        let exposed = exposed_base_outputs(values.len(), chunk_index, is_last);
        final_outputs = permute_w32(circuit, inputs, chunk_index == 0, &exposed)?;
        previous = Some(final_outputs.clone());
    }
    digest_from_outputs(&final_outputs)
}

fn exposed_base_outputs(value_count: usize, chunk_index: usize, is_last: bool) -> Vec<usize> {
    if is_last {
        return (0..4).collect();
    }
    let next_start = (chunk_index + 1) * 24;
    let next_len = value_count.saturating_sub(next_start).min(24);
    if next_len % 4 == 0 {
        Vec::new()
    } else {
        (0..=next_len / 4).collect()
    }
}

fn fill_coefficients(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    coefficients: &mut Vec<Target>,
    previous: Option<&[Option<Target>]>,
    extension_index: usize,
) -> Result<(), VerificationError> {
    if coefficients.len() == 4 {
        return Ok(());
    }
    let prior = if let Some(outputs) = previous {
        let output = outputs
            .get(extension_index)
            .and_then(|output| *output)
            .ok_or_else(|| shape("binary MMCS sponge output is missing"))?;
        Some(
            circuit
                .decompose_ext_to_base_coeffs::<KoalaBear>(output)
                .map_err(circuit_shape)?,
        )
    } else {
        None
    };
    while coefficients.len() < 4 {
        let index = coefficients.len();
        coefficients.push(
            prior
                .as_ref()
                .and_then(|values| values.get(index))
                .copied()
                .unwrap_or_else(|| circuit.define_const(Plonky3ChallengeV2::ZERO)),
        );
    }
    Ok(())
}

fn pack_coefficients(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    coefficients: &[Target],
) -> Result<Target, VerificationError> {
    if coefficients.len() != 4 {
        return Err(shape(
            "binary MMCS packed limb must contain four coefficients",
        ));
    }
    circuit
        .recompose_base_coeffs_to_ext_via_alu::<KoalaBear>(coefficients)
        .map_err(circuit_shape)
}

pub(super) fn hash_extension_values(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    values: &[Target],
) -> Result<DigestV2, VerificationError> {
    if values.is_empty() {
        return Err(shape("binary MMCS extension row is empty"));
    }
    let mut final_outputs = Vec::new();
    let chunk_count = values.len().div_ceil(6);
    for (chunk_index, chunk) in values.chunks(6).enumerate() {
        let mut inputs = vec![None; 8];
        for (input, &value) in inputs.iter_mut().zip(chunk) {
            *input = Some(value);
        }
        let is_last = chunk_index + 1 == chunk_count;
        let exposed = if is_last {
            (0..4).collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        final_outputs = permute_w32(circuit, inputs, chunk_index == 0, &exposed)?;
    }
    digest_from_outputs(&final_outputs)
}

fn permute_w32(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    inputs: Vec<Option<Target>>,
    new_start: bool,
    exposed: &[usize],
) -> Result<Vec<Option<Target>>, VerificationError> {
    let mut out_ctl = vec![false; 6];
    for &index in exposed {
        let output = out_ctl
            .get_mut(index)
            .ok_or_else(|| shape("binary MMCS output index is out of range"))?;
        *output = true;
    }
    let (_, outputs) = circuit
        .add_perm(
            PermConfig::poseidon2(Poseidon2Config::KOALA_BEAR_D4_W32),
            &PermCall {
                new_start,
                merkle_path: false,
                mmcs_bit: None,
                mmcs_bit2: None,
                inputs,
                out_ctl,
                return_all_outputs: false,
                mmcs_index_sum: None,
            },
        )
        .map_err(circuit_shape)?;
    Ok(outputs.into_iter().take(6).collect())
}

fn digest_from_outputs(outputs: &[Option<Target>]) -> Result<DigestV2, VerificationError> {
    outputs
        .get(..4)
        .ok_or_else(|| shape("binary MMCS digest output is missing"))?
        .iter()
        .map(|output| {
            output
                .as_ref()
                .copied()
                .ok_or_else(|| shape("binary MMCS digest output is missing"))
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| shape("binary MMCS digest has the wrong width"))
}

pub(super) fn compress_selected(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    current: DigestV2,
    sibling: DigestV2,
    direction: Target,
) -> Result<DigestV2, VerificationError> {
    circuit.assert_bool(direction);
    // Siblings are private extension-field inputs. Materialize each one through
    // an ALU row before it can become the minuend of `select`'s backwards
    // subtraction. This gives both the private value and the subtraction result
    // an explicit WitnessChecks creator without exposing or cloning witness data.
    let one = circuit.define_const(Plonky3ChallengeV2::ONE);
    let zero = circuit.define_const(Plonky3ChallengeV2::ZERO);
    let sibling = sibling.map(|value| circuit.mul_add(value, one, zero));
    let left =
        core::array::from_fn(|index| circuit.select(direction, sibling[index], current[index]));
    let right =
        core::array::from_fn(|index| circuit.select(direction, current[index], sibling[index]));
    compress_pair(circuit, left, right)
}

pub(super) fn compress_pair(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    left: DigestV2,
    right: DigestV2,
) -> Result<DigestV2, VerificationError> {
    let inputs = left.into_iter().chain(right).map(Some).collect::<Vec<_>>();
    let outputs = permute_w32(circuit, inputs, true, &[0, 1, 2, 3])?;
    digest_from_outputs(&outputs)
}

pub(super) fn cap_height(root_count: usize) -> Result<usize, VerificationError> {
    if root_count == 0 || !root_count.is_power_of_two() {
        return Err(shape("binary MMCS cap root count is not a power of two"));
    }
    Ok(root_count.trailing_zeros() as usize)
}

pub(super) fn connect_cap(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    commitment: &CommitmentV2,
    digest: DigestV2,
    cap_bits: &[Target],
) -> Result<(), VerificationError> {
    if commitment.cap_targets.len()
        != 1usize
            .checked_shl(
                u32::try_from(cap_bits.len())
                    .map_err(|_| shape("binary MMCS cap height is too large"))?,
            )
            .ok_or_else(|| shape("binary MMCS cap height is too large"))?
    {
        return Err(shape("binary MMCS cap selector shape mismatch"));
    }
    let mut roots = commitment
        .cap_targets
        .iter()
        .map(|root| pack_root(circuit, root))
        .collect::<Result<Vec<_>, _>>()?;
    for &bit in cap_bits {
        circuit.assert_bool(bit);
        roots = roots
            .chunks_exact(2)
            .map(|pair| {
                core::array::from_fn(|index| circuit.select(bit, pair[1][index], pair[0][index]))
            })
            .collect();
    }
    let selected = roots
        .first()
        .ok_or_else(|| shape("binary MMCS cap selector is empty"))?;
    for (actual, expected) in digest.into_iter().zip(selected.iter().copied()) {
        circuit.connect(actual, expected);
    }
    Ok(())
}

fn pack_root(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    root: &[Target; PLONKY3_MMCS_DIGEST_ELEMS_V2],
) -> Result<DigestV2, VerificationError> {
    let packed = root
        .chunks_exact(4)
        .map(|chunk| pack_coefficients(circuit, chunk))
        .collect::<Result<Vec<_>, _>>()?;
    packed
        .try_into()
        .map_err(|_| shape("binary MMCS commitment digest has the wrong width"))
}

fn circuit_shape(error: CircuitBuilderError) -> VerificationError {
    shape(format!("binary W32 circuit construction failed: {error:?}"))
}

pub(super) fn shape(message: impl Into<String>) -> VerificationError {
    VerificationError::InvalidProofShape(message.into())
}
