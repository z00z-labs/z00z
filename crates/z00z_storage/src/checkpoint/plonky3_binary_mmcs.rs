//! Binary W32 MMCS opening and reduction constraints.

use std::collections::BTreeMap;

use p3_circuit::CircuitBuilder;
use p3_field::{Field, PrimeCharacteristicRing, TwoAdicField};
use p3_koala_bear::KoalaBear;
use z00z_plonky3_circuit_prover::traits::ComsWithOpeningsTargets;
use z00z_plonky3_circuit_prover::{Target, VerificationError};

use super::plonky3_binary_fri_fold::verify_commit_paths;
use super::plonky3_binary_hash::{
    cap_height, compress_pair, compress_selected, connect_cap, hash_base_values, shape,
    CommitmentV2, DigestV2,
};
use super::{Plonky3ChallengeV2, Plonky3RecOpeningProofV2};

type DomainV2 = p3_field::coset::TwoAdicMultiplicativeCoset<KoalaBear>;
type OpeningsV2 = ComsWithOpeningsTargets<CommitmentV2, DomainV2>;

pub(super) fn verify_binary_paths(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    proof: &Plonky3RecOpeningProofV2,
    alpha: Target,
    betas: &[Target],
    query_bits: &[Vec<Target>],
    openings: &OpeningsV2,
    log_blowup: usize,
) -> Result<(), VerificationError> {
    validate_fri_shape(proof, betas, query_bits)?;
    let log_max_height = query_bits
        .first()
        .map(Vec::len)
        .ok_or_else(|| shape("binary MMCS requires at least one FRI query"))?;

    for (query_index, query) in proof.query_proofs.iter().enumerate() {
        let bits = query_bits
            .get(query_index)
            .ok_or_else(|| shape("missing binary MMCS query bits"))?;
        verify_input_paths(circuit, openings, &query.input_proof, bits, log_blowup)?;
        let reduced = reduce_openings(
            circuit,
            openings,
            &query.input_proof,
            bits,
            alpha,
            log_blowup,
        )?;
        verify_commit_paths(circuit, proof, query, betas, bits, log_max_height, reduced)?;
    }
    Ok(())
}

fn validate_fri_shape(
    proof: &Plonky3RecOpeningProofV2,
    betas: &[Target],
    query_bits: &[Vec<Target>],
) -> Result<(), VerificationError> {
    if betas.len() != proof.log_arities.len()
        || betas.len() != proof.commit_phase_commits.len()
        || query_bits.len() != proof.query_proofs.len()
        || betas.is_empty()
        || query_bits.is_empty()
    {
        return Err(shape("binary MMCS FRI shape mismatch"));
    }
    let bit_count = query_bits[0].len();
    if bit_count == 0 || query_bits.iter().any(|bits| bits.len() != bit_count) {
        return Err(shape("binary MMCS query-bit shape mismatch"));
    }
    if proof
        .log_arities
        .iter()
        .any(|log_arity| !(1..=3).contains(log_arity))
    {
        return Err(shape("binary MMCS supports FRI log arity 1 through 3"));
    }
    Ok(())
}

fn verify_input_paths(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    openings: &OpeningsV2,
    input_proofs: &[z00z_plonky3_circuit_prover::pcs::BatchOpeningTargets<
        KoalaBear,
        Plonky3ChallengeV2,
        super::plonky3_binary_pcs::BinaryRecMmcsV2,
    >],
    index_bits: &[Target],
    log_blowup: usize,
) -> Result<(), VerificationError> {
    if openings.len() != input_proofs.len() {
        return Err(shape("binary MMCS input batch count mismatch"));
    }
    for ((commitment, matrices), batch) in openings.iter().zip(input_proofs) {
        if matrices.len() != batch.opened_values.len() {
            return Err(shape("binary MMCS input matrix count mismatch"));
        }
        let heights = matrices
            .iter()
            .map(|(domain, _)| checked_height(domain.log_size(), log_blowup))
            .collect::<Result<Vec<_>, _>>()?;
        verify_base_batch(
            circuit,
            commitment,
            &heights,
            &batch.opened_values,
            batch.opening_proof.siblings(),
            index_bits,
        )?;
    }
    Ok(())
}

fn checked_height(log_size: usize, log_blowup: usize) -> Result<usize, VerificationError> {
    let log_height = log_size
        .checked_add(log_blowup)
        .ok_or_else(|| shape("binary MMCS height overflow"))?;
    1usize
        .checked_shl(
            u32::try_from(log_height).map_err(|_| shape("binary MMCS height is too large"))?,
        )
        .ok_or_else(|| shape("binary MMCS height is too large"))
}

fn verify_base_batch(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    commitment: &CommitmentV2,
    heights: &[usize],
    rows: &[Vec<Target>],
    siblings: &[DigestV2],
    index_bits: &[Target],
) -> Result<(), VerificationError> {
    validate_batch_shape(commitment, heights, rows, siblings, index_bits)?;
    let max_height = heights
        .iter()
        .copied()
        .max()
        .ok_or_else(|| shape("binary MMCS batch is empty"))?;
    let batch_log = max_height.trailing_zeros() as usize;
    let batch_index_range = reduced_index_bit_range(index_bits.len(), batch_log)?;
    let batch_index_bits = &index_bits[batch_index_range];
    let mut digest = hash_base_height(circuit, heights, rows, max_height)?;
    let cap_height = cap_height(commitment.cap_targets.len())?;
    let path_depth = batch_index_bits
        .len()
        .checked_sub(cap_height)
        .ok_or_else(|| shape("binary MMCS cap exceeds tree height"))?;
    let mut current_height = max_height;

    for level in 0..path_depth {
        digest = compress_selected(circuit, digest, siblings[level], batch_index_bits[level])?;
        current_height /= 2;
        if heights.contains(&current_height) {
            let injected = hash_base_height(circuit, heights, rows, current_height)?;
            digest = compress_pair(circuit, digest, injected)?;
        }
    }
    connect_cap(circuit, commitment, digest, &batch_index_bits[path_depth..])
}

fn validate_batch_shape(
    commitment: &CommitmentV2,
    heights: &[usize],
    rows: &[Vec<Target>],
    siblings: &[DigestV2],
    index_bits: &[Target],
) -> Result<(), VerificationError> {
    if heights.is_empty()
        || heights.len() != rows.len()
        || heights.iter().any(|height| !height.is_power_of_two())
        || commitment.cap_targets.is_empty()
    {
        return Err(shape("binary MMCS batch geometry mismatch"));
    }
    let max_height = heights.iter().copied().max().unwrap_or(0);
    let expected_log = max_height.trailing_zeros() as usize;
    reduced_index_bit_range(index_bits.len(), expected_log)?;
    let cap_log = cap_height(commitment.cap_targets.len())?;
    if siblings.len() != expected_log.saturating_sub(cap_log) {
        return Err(shape("binary MMCS authentication path length mismatch"));
    }
    Ok(())
}

fn reduced_index_bit_range(
    global_log_height: usize,
    batch_log_height: usize,
) -> Result<core::ops::Range<usize>, VerificationError> {
    let start = global_log_height
        .checked_sub(batch_log_height)
        .ok_or_else(|| shape("binary MMCS batch height exceeds query height"))?;
    Ok(start..global_log_height)
}

fn hash_base_height(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    heights: &[usize],
    rows: &[Vec<Target>],
    height: usize,
) -> Result<DigestV2, VerificationError> {
    let mut values = Vec::new();
    for (matrix_height, row) in heights.iter().zip(rows) {
        if *matrix_height == height {
            values.extend(row.iter().copied());
        }
    }
    if values.is_empty() {
        return Err(shape("binary MMCS height has no opened row"));
    }
    hash_base_values(circuit, &values)
}

fn reduce_openings(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    openings: &OpeningsV2,
    input_proofs: &[z00z_plonky3_circuit_prover::pcs::BatchOpeningTargets<
        KoalaBear,
        Plonky3ChallengeV2,
        super::plonky3_binary_pcs::BinaryRecMmcsV2,
    >],
    index_bits: &[Target],
    alpha: Target,
    log_blowup: usize,
) -> Result<Vec<(usize, Target)>, VerificationError> {
    let mut reduced = BTreeMap::<usize, (Target, Target)>::new();
    for ((_, matrices), batch) in openings.iter().zip(input_proofs) {
        for ((domain, points), opened_values) in matrices.iter().zip(&batch.opened_values) {
            let log_height = domain
                .log_size()
                .checked_add(log_blowup)
                .ok_or_else(|| shape("binary MMCS reduction height overflow"))?;
            let x = evaluation_point(circuit, index_bits, log_height)?;
            for (z, point_values) in points {
                add_reduced_opening(
                    circuit,
                    &mut reduced,
                    log_height,
                    opened_values,
                    point_values,
                    *z,
                    x,
                    alpha,
                )?;
            }
        }
    }
    if let Some((_, constant_opening)) = reduced.get(&log_blowup) {
        let zero = circuit.define_const(Plonky3ChallengeV2::ZERO);
        circuit.connect(*constant_opening, zero);
    }
    Ok(reduced
        .into_iter()
        .rev()
        .map(|(height, (_, value))| (height, value))
        .collect())
}

#[allow(clippy::too_many_arguments)]
fn add_reduced_opening(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    reduced: &mut BTreeMap<usize, (Target, Target)>,
    log_height: usize,
    opened_values: &[Target],
    point_values: &[Target],
    z: Target,
    x: Target,
    alpha: Target,
) -> Result<(), VerificationError> {
    if opened_values.len() != point_values.len() {
        return Err(shape("binary MMCS reduced-opening width mismatch"));
    }
    let one = circuit.define_const(Plonky3ChallengeV2::ONE);
    let zero = circuit.define_const(Plonky3ChallengeV2::ZERO);
    let (alpha_power, sum) = reduced.entry(log_height).or_insert((one, zero));
    let mut inner = zero;
    for index in (0..opened_values.len()).rev() {
        inner = circuit.horner_acc_step(inner, alpha, point_values[index], opened_values[index]);
    }
    if !opened_values.is_empty() {
        let denominator = circuit.sub(z, x);
        let inverse = circuit.div(one, denominator);
        let scaled = circuit.mul(*alpha_power, inverse);
        *sum = circuit.mul_add(scaled, inner, *sum);
        let advance = pow_constant(circuit, alpha, opened_values.len());
        *alpha_power = circuit.mul(*alpha_power, advance);
    }
    Ok(())
}

fn evaluation_point(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    index_bits: &[Target],
    log_height: usize,
) -> Result<Target, VerificationError> {
    if log_height > index_bits.len() {
        return Err(shape("binary MMCS evaluation height exceeds query height"));
    }
    let offset = index_bits.len() - log_height;
    let generator = KoalaBear::two_adic_generator(log_height);
    let one = circuit.define_const(Plonky3ChallengeV2::ONE);
    let mut power = one;
    for index in 0..log_height {
        let bit = index_bits[offset + log_height - 1 - index];
        circuit.assert_bool(bit);
        let factor =
            circuit.define_const(Plonky3ChallengeV2::from(generator.exp_u64(1u64 << index)));
        let selected = circuit.select(bit, factor, one);
        power = circuit.mul(power, selected);
    }
    let shift = circuit.define_const(Plonky3ChallengeV2::from(KoalaBear::GENERATOR));
    Ok(circuit.mul(shift, power))
}

fn pow_constant(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    base: Target,
    exponent: usize,
) -> Target {
    if exponent == 1 {
        return base;
    }
    let one = circuit.define_const(Plonky3ChallengeV2::ONE);
    let mut result = one;
    let mut power = base;
    let mut remaining = exponent;
    while remaining != 0 {
        if remaining & 1 == 1 {
            result = circuit.mul(result, power);
        }
        remaining >>= 1;
        if remaining != 0 {
            power = circuit.mul(power, power);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::reduced_index_bit_range;

    #[test]
    fn reduced_batch_index_uses_high_query_bits() {
        assert_eq!(reduced_index_bit_range(9, 6).unwrap(), 3..9);
        assert_eq!(reduced_index_bit_range(9, 9).unwrap(), 0..9);
        assert_eq!(reduced_index_bit_range(9, 0).unwrap(), 9..9);
        assert!(reduced_index_bit_range(8, 9).is_err());
    }
}
