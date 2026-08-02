use z00z_crypto::{
    create_commitment, create_range_proof_rng, verify_range_proof, Z00ZScalar, AGGREGATION_FACTOR,
    MIN_VALUE_PROMISE, RANGE_PROOF_BITS,
};
use z00z_utils::rng::DeterministicRngProvider;

fn test_blind(seed: [u8; 32]) -> Z00ZScalar {
    let mut rng = DeterministicRngProvider::from_seed(seed).rng();
    Z00ZScalar::random(&mut rng).unwrap()
}

#[test]
fn test_range_proof_rng_stable() {
    let value = 777u64;
    let blind = test_blind([7u8; 32]);
    let commit = create_commitment(value, &blind).expect("commitment");
    let mut rng_a = DeterministicRngProvider::from_seed([9u8; 32]).rng();
    let mut rng_b = DeterministicRngProvider::from_seed([9u8; 32]).rng();

    let proof_a = create_range_proof_rng(
        value,
        &blind,
        RANGE_PROOF_BITS,
        MIN_VALUE_PROMISE,
        &mut rng_a,
    )
    .expect("proof a");
    let proof_b = create_range_proof_rng(
        value,
        &blind,
        RANGE_PROOF_BITS,
        MIN_VALUE_PROMISE,
        &mut rng_b,
    )
    .expect("proof b");

    assert_eq!(proof_a, proof_b);
    verify_range_proof(
        &proof_a,
        &commit,
        RANGE_PROOF_BITS,
        AGGREGATION_FACTOR,
        MIN_VALUE_PROMISE,
    )
    .expect("verify proof a");
}

#[test]
fn test_rng_proof_diff_seed() {
    let value = 777u64;
    let blind = test_blind([11u8; 32]);
    let mut rng_a = DeterministicRngProvider::from_seed([13u8; 32]).rng();
    let mut rng_b = DeterministicRngProvider::from_seed([17u8; 32]).rng();

    let proof_a = create_range_proof_rng(
        value,
        &blind,
        RANGE_PROOF_BITS,
        MIN_VALUE_PROMISE,
        &mut rng_a,
    )
    .expect("proof a");
    let proof_b = create_range_proof_rng(
        value,
        &blind,
        RANGE_PROOF_BITS,
        MIN_VALUE_PROMISE,
        &mut rng_b,
    )
    .expect("proof b");

    assert_ne!(proof_a, proof_b);
}
