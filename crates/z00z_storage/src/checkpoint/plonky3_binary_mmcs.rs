//! Binary W32 MMCS constraints for the private Plonky3 recursion adapter.

use std::collections::BTreeMap;

use p3_circuit::CircuitBuilder;
use p3_field::{Field, PrimeCharacteristicRing, TwoAdicField};
use p3_koala_bear::KoalaBear;
use z00z_plonky3_circuit_prover::traits::ComsWithOpeningsTargets;
use z00z_plonky3_circuit_prover::{Target, VerificationError};

use super::plonky3_binary_hash::{
    cap_height, compress_pair, compress_selected, connect_cap, hash_base_values,
    hash_extension_values, shape, CommitmentV2, DigestV2,
};
use super::{Plonky3ChallengeV2, Plonky3RecOpeningProofV2, PLONKY3_MMCS_DIGEST_ELEMS_V2};

type DomainV2 = p3_field::coset::TwoAdicMultiplicativeCoset<KoalaBear>;
type OpeningsV2 = ComsWithOpeningsTargets<CommitmentV2, DomainV2>;
type QueryProofTargetsV2 = z00z_plonky3_circuit_prover::pcs::QueryProofTargets<
    KoalaBear,
    Plonky3ChallengeV2,
    z00z_plonky3_circuit_prover::pcs::InputProofTargets<
        KoalaBear,
        Plonky3ChallengeV2,
        super::plonky3_recursion::BinaryRecMmcsV2,
    >,
    z00z_plonky3_circuit_prover::pcs::RecExtensionValMmcs<
        KoalaBear,
        Plonky3ChallengeV2,
        PLONKY3_MMCS_DIGEST_ELEMS_V2,
        super::plonky3_recursion::BinaryRecMmcsV2,
    >,
>;

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
        super::plonky3_recursion::BinaryRecMmcsV2,
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
    let mut digest = hash_base_height(circuit, heights, rows, max_height)?;
    let cap_height = cap_height(commitment.cap_targets.len())?;
    let path_depth = index_bits
        .len()
        .checked_sub(cap_height)
        .ok_or_else(|| shape("binary MMCS cap exceeds tree height"))?;
    let mut current_height = max_height;

    for level in 0..path_depth {
        digest = compress_selected(circuit, digest, siblings[level], index_bits[level])?;
        current_height /= 2;
        if heights.contains(&current_height) {
            let injected = hash_base_height(circuit, heights, rows, current_height)?;
            digest = compress_pair(circuit, digest, injected)?;
        }
    }
    connect_cap(circuit, commitment, digest, &index_bits[path_depth..])
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
    if expected_log != index_bits.len() {
        return Err(shape("binary MMCS index height mismatch"));
    }
    let cap_log = cap_height(commitment.cap_targets.len())?;
    if siblings.len() != expected_log.saturating_sub(cap_log) {
        return Err(shape("binary MMCS authentication path length mismatch"));
    }
    Ok(())
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
        super::plonky3_recursion::BinaryRecMmcsV2,
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

fn verify_commit_paths(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    proof: &Plonky3RecOpeningProofV2,
    query: &QueryProofTargetsV2,
    betas: &[Target],
    index_bits: &[Target],
    log_max_height: usize,
    reduced: Vec<(usize, Target)>,
) -> Result<(), VerificationError> {
    let mut current = initial_reduced(&reduced, log_max_height)?;
    let roll_ins = align_roll_ins(circuit, proof, &reduced, log_max_height)?;
    let mut bits_consumed = 0usize;

    for phase_index in 0..proof.log_arities.len() {
        let log_arity = proof.log_arities[phase_index];
        let opening = query
            .commit_phase_openings
            .get(phase_index)
            .ok_or_else(|| shape("binary MMCS commit opening is missing"))?;
        let siblings = opening.sibling_values_packed(circuit);
        let evals = reconstruct_evals(
            circuit,
            current,
            &siblings,
            &index_bits[bits_consumed..bits_consumed + log_arity],
        )?;
        let folded_height = log_max_height
            .checked_sub(bits_consumed + log_arity)
            .ok_or_else(|| shape("binary MMCS FRI folding exceeds query height"))?;
        if folded_height != 0 {
            verify_extension_batch(
                circuit,
                &proof.commit_phase_commits[phase_index],
                &evals,
                opening.opening_proof.siblings(),
                &index_bits[bits_consumed + log_arity..],
                folded_height,
            )?;
        }
        let subgroup_start = subgroup_start(
            circuit,
            index_bits,
            bits_consumed + log_arity,
            folded_height,
            log_arity,
        )?;
        current = fold_evals(
            circuit,
            &evals,
            betas[phase_index],
            subgroup_start,
            log_arity,
            roll_ins[phase_index],
        )?;
        bits_consumed += log_arity;
    }
    connect_final(circuit, proof, index_bits, bits_consumed, current)
}

fn initial_reduced(
    reduced: &[(usize, Target)],
    log_max_height: usize,
) -> Result<Target, VerificationError> {
    let (height, value) = reduced
        .first()
        .copied()
        .ok_or_else(|| shape("binary MMCS has no reduced opening"))?;
    if height != log_max_height {
        return Err(shape("binary MMCS first reduced opening is not maximal"));
    }
    Ok(value)
}

fn align_roll_ins(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    proof: &Plonky3RecOpeningProofV2,
    reduced: &[(usize, Target)],
    log_max_height: usize,
) -> Result<Vec<Option<Target>>, VerificationError> {
    let mut heights = Vec::with_capacity(proof.log_arities.len());
    let mut consumed = 0usize;
    for log_arity in &proof.log_arities {
        consumed = consumed
            .checked_add(*log_arity)
            .ok_or_else(|| shape("binary MMCS FRI height overflow"))?;
        heights.push(
            log_max_height
                .checked_sub(consumed)
                .ok_or_else(|| shape("binary MMCS FRI height underflow"))?,
        );
    }
    let mut roll_ins = vec![None; heights.len()];
    for &(height, value) in reduced.iter().skip(1) {
        if let Some(index) = heights.iter().position(|candidate| *candidate == height) {
            if roll_ins[index].replace(value).is_some() {
                return Err(shape("binary MMCS duplicate reduced opening height"));
            }
        } else {
            let zero = circuit.define_const(Plonky3ChallengeV2::ZERO);
            circuit.connect(value, zero);
        }
    }
    Ok(roll_ins)
}

fn reconstruct_evals(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    folded: Target,
    siblings: &[Target],
    group_bits: &[Target],
) -> Result<Vec<Target>, VerificationError> {
    let arity = 1usize
        .checked_shl(
            u32::try_from(group_bits.len())
                .map_err(|_| shape("binary MMCS FRI arity is too large"))?,
        )
        .ok_or_else(|| shape("binary MMCS FRI arity is too large"))?;
    if siblings.len() + 1 != arity {
        return Err(shape("binary MMCS FRI sibling count mismatch"));
    }
    let selectors = one_hot(circuit, group_bits)?;
    let zero = circuit.define_const(Plonky3ChallengeV2::ZERO);
    let mut evals = Vec::with_capacity(arity);
    for position in 0..arity {
        let mut value = zero;
        for selected in 0..arity {
            let candidate = if position == selected {
                folded
            } else {
                siblings[position - usize::from(position > selected)]
            };
            value = circuit.mul_add(selectors[selected], candidate, value);
        }
        evals.push(value);
    }
    Ok(evals)
}

fn one_hot(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    bits: &[Target],
) -> Result<Vec<Target>, VerificationError> {
    if bits.len() > 3 {
        return Err(shape("binary MMCS FRI selector exceeds bounded arity"));
    }
    let arity = 1usize << bits.len();
    let one = circuit.define_const(Plonky3ChallengeV2::ONE);
    let negated = bits
        .iter()
        .map(|bit| {
            circuit.assert_bool(*bit);
            circuit.sub(one, *bit)
        })
        .collect::<Vec<_>>();
    let mut selectors = Vec::with_capacity(arity);
    for selected in 0..arity {
        let mut product = one;
        for (index, bit) in bits.iter().enumerate() {
            let factor = if (selected >> index) & 1 == 1 {
                *bit
            } else {
                negated[index]
            };
            product = circuit.mul(product, factor);
        }
        selectors.push(product);
    }
    Ok(selectors)
}

fn verify_extension_batch(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    commitment: &CommitmentV2,
    evals: &[Target],
    siblings: &[DigestV2],
    parent_bits: &[Target],
    log_height: usize,
) -> Result<(), VerificationError> {
    if parent_bits.len() < log_height {
        return Err(shape("binary MMCS parent query bits are missing"));
    }
    let cap_log = cap_height(commitment.cap_targets.len())?;
    let path_depth = log_height
        .checked_sub(cap_log)
        .ok_or_else(|| shape("binary MMCS commit cap exceeds tree height"))?;
    if siblings.len() != path_depth {
        return Err(shape("binary MMCS commit path length mismatch"));
    }
    let mut digest = hash_extension_values(circuit, evals)?;
    for level in 0..path_depth {
        digest = compress_selected(circuit, digest, siblings[level], parent_bits[level])?;
    }
    connect_cap(
        circuit,
        commitment,
        digest,
        &parent_bits[path_depth..log_height],
    )
}

fn subgroup_start(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    index_bits: &[Target],
    parent_start: usize,
    folded_height: usize,
    log_arity: usize,
) -> Result<Target, VerificationError> {
    let current_height = folded_height
        .checked_add(log_arity)
        .ok_or_else(|| shape("binary MMCS subgroup height overflow"))?;
    if current_height > index_bits.len()
        || parent_start
            .checked_add(folded_height)
            .is_none_or(|end| end > index_bits.len())
    {
        return Err(shape("binary MMCS subgroup bits are missing"));
    }
    let generator = KoalaBear::two_adic_generator(current_height);
    let one = circuit.define_const(Plonky3ChallengeV2::ONE);
    let mut result = one;
    for index in 0..folded_height {
        let bit = index_bits[parent_start + folded_height - 1 - index];
        circuit.assert_bool(bit);
        let power =
            circuit.define_const(Plonky3ChallengeV2::from(generator.exp_u64(1u64 << index)));
        let selected = circuit.select(bit, power, one);
        result = circuit.mul(result, selected);
    }
    Ok(result)
}

fn fold_evals(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    evals: &[Target],
    beta: Target,
    subgroup_start: Target,
    log_arity: usize,
    roll_in: Option<Target>,
) -> Result<Target, VerificationError> {
    let omega = KoalaBear::two_adic_generator(log_arity);
    let mut points = Vec::with_capacity(evals.len());
    for index in 0..evals.len() {
        let reversed = reverse_bits(index, log_arity);
        let factor = circuit.define_const(Plonky3ChallengeV2::from(omega.exp_u64(reversed as u64)));
        points.push(circuit.mul(subgroup_start, factor));
    }
    let zero = circuit.define_const(Plonky3ChallengeV2::ZERO);
    let one = circuit.define_const(Plonky3ChallengeV2::ONE);
    let mut folded = zero;
    for index in 0..evals.len() {
        let mut numerator = one;
        let mut denominator = one;
        for other in 0..evals.len() {
            if index != other {
                let beta_delta = circuit.sub(beta, points[other]);
                numerator = circuit.mul(numerator, beta_delta);
                let point_delta = circuit.sub(points[index], points[other]);
                denominator = circuit.mul(denominator, point_delta);
            }
        }
        let basis = circuit.div(numerator, denominator);
        folded = circuit.mul_add(evals[index], basis, folded);
    }
    if let Some(reduced) = roll_in {
        let beta_power = circuit.exp_power_of_2(beta, log_arity);
        folded = circuit.mul_add(beta_power, reduced, folded);
    }
    Ok(folded)
}

fn reverse_bits(value: usize, bit_count: usize) -> usize {
    let mut reversed = 0usize;
    for index in 0..bit_count {
        reversed |= ((value >> index) & 1) << (bit_count - 1 - index);
    }
    reversed
}

fn connect_final(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    proof: &Plonky3RecOpeningProofV2,
    index_bits: &[Target],
    consumed: usize,
    folded: Target,
) -> Result<(), VerificationError> {
    if consumed > index_bits.len() || proof.final_poly.is_empty() {
        return Err(shape("binary MMCS final polynomial shape mismatch"));
    }
    let point = final_query_point(circuit, index_bits, consumed)?;
    let zero = circuit.define_const(Plonky3ChallengeV2::ZERO);
    let mut evaluation = zero;
    for coefficient in proof.final_poly.iter().rev() {
        evaluation = circuit.horner_acc_step(evaluation, point, *coefficient, zero);
    }
    circuit.connect(folded, evaluation);
    Ok(())
}

fn final_query_point(
    circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
    index_bits: &[Target],
    consumed: usize,
) -> Result<Target, VerificationError> {
    let log_height = index_bits.len();
    let generator = KoalaBear::two_adic_generator(log_height);
    let one = circuit.define_const(Plonky3ChallengeV2::ONE);
    let mut result = one;
    for power_index in consumed..log_height {
        let source = log_height - 1 - (power_index - consumed);
        let bit = index_bits[source];
        circuit.assert_bool(bit);
        let power = circuit.define_const(Plonky3ChallengeV2::from(
            generator.exp_u64(1u64 << power_index),
        ));
        let selected = circuit.select(bit, power, one);
        result = circuit.mul(result, selected);
    }
    Ok(result)
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
