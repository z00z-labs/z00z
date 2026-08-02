use std::sync::Arc;

use z00z_core::assets::{
    commit_amount, verify_commitment_opening, Asset, AssetClass, AssetDefinition, AssetPackPlain,
};
use z00z_crypto::{
    create_commitment, create_range_proof, verify_range_proof, Z00ZScalar, AGGREGATION_FACTOR,
    MIN_VALUE_PROMISE, RANGE_PROOF_BITS,
};
use z00z_utils::rng::SystemRngProvider;

fn scalar_u64(seed: u64) -> Z00ZScalar {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    Z00ZScalar::try_from_bytes(bytes).expect("scalar")
}

fn make_def() -> Arc<AssetDefinition> {
    Arc::new(
        AssetDefinition::new(
            [7u8; 32],
            AssetClass::Coin,
            "Test Coin".to_string(),
            "TST".to_string(),
            8,
            1000,
            100_000_000,
            "test.io".to_string(),
            1,
            1,
            0b0001_0111,
            None,
        )
        .expect("definition"),
    )
}

#[test]
fn test_amount_encoding() {
    assert_eq!(0u64.to_le_bytes(), [0x00; 8]);
    assert_eq!(
        1u64.to_le_bytes(),
        [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        1000u64.to_le_bytes(),
        [0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
}

#[test]
fn test_amount_arithmetic() {
    assert_eq!(100u64.checked_add(50), Some(150));
    assert_eq!(100u64.checked_sub(50), Some(50));
    assert_eq!(u64::MAX.checked_add(1), None);
    assert_eq!(0u64.checked_sub(1), None);
}

#[test]
fn test_blinding_unique() {
    let mut rng = SystemRngProvider.rng();
    let first = Z00ZScalar::random(&mut rng).unwrap();
    let second = Z00ZScalar::random(&mut rng).unwrap();
    assert_ne!(first.as_bytes(), second.as_bytes());
}

#[test]
fn test_commitment_creation() {
    let amount = 1000u64;
    let blind = scalar_u64(101);
    let one = create_commitment(amount, &blind).expect("commit one");
    let two = create_commitment(amount, &blind).expect("commit two");
    assert_eq!(one.as_bytes(), two.as_bytes());

    let other = scalar_u64(102);
    let three = create_commitment(amount, &other).expect("commit three");
    assert_ne!(one.as_bytes(), three.as_bytes());
}

#[test]
fn test_opening_ok() {
    let amount = 500u64;
    let blind = scalar_u64(201);
    let commitment = commit_amount(amount, &blind).expect("commit");
    assert!(verify_commitment_opening(&commitment, amount, &blind).expect("opening"));
}

#[test]
fn test_opening_bad_amount() {
    let amount = 500u64;
    let blind = scalar_u64(202);
    let commitment = commit_amount(amount, &blind).expect("commit");
    assert!(!verify_commitment_opening(&commitment, amount + 1, &blind).expect("opening"));
}

#[test]
fn test_opening_bad_blind() {
    let amount = 500u64;
    let blind = scalar_u64(203);
    let wrong = scalar_u64(204);
    let commitment = commit_amount(amount, &blind).expect("commit");
    assert!(!verify_commitment_opening(&commitment, amount, &wrong).expect("opening"));
}

#[test]
fn test_commitment_homomorphic() {
    let value_one = 300u64;
    let blind_one = scalar_u64(301);
    let commit_one = commit_amount(value_one, &blind_one).expect("commit one");

    let value_two = 700u64;
    let blind_two = scalar_u64(302);
    let commit_two = commit_amount(value_two, &blind_two).expect("commit two");

    let sum_commit = &commit_one + &commit_two;
    let sum_value = value_one + value_two;
    let sum_blind = &blind_one + &blind_two;
    let expected = commit_amount(sum_value, &sum_blind).expect("commit sum");
    assert_eq!(sum_commit, expected);
}

#[test]
fn test_range_proof_creation() {
    let amount = 1000u64;
    let blind = scalar_u64(401);
    let commitment = commit_amount(amount, &blind).expect("commit");

    let proof =
        create_range_proof(amount, &blind, RANGE_PROOF_BITS, MIN_VALUE_PROMISE).expect("proof");
    verify_range_proof(
        &proof,
        &commitment,
        RANGE_PROOF_BITS,
        AGGREGATION_FACTOR,
        MIN_VALUE_PROMISE,
    )
    .expect("verify");
}

#[test]
fn test_range_proof_bad_amount() {
    let blind = scalar_u64(402);
    let commitment = commit_amount(1000, &blind).expect("commit");
    let proof =
        create_range_proof(2000, &blind, RANGE_PROOF_BITS, MIN_VALUE_PROMISE).expect("proof");

    assert!(verify_range_proof(
        &proof,
        &commitment,
        RANGE_PROOF_BITS,
        AGGREGATION_FACTOR,
        MIN_VALUE_PROMISE,
    )
    .is_err());
}

#[test]
fn test_output_roundtrip() {
    let amount = 5000u64;
    let blind = scalar_u64(501);
    let s_out = [0x11u8; 32];
    let pack = AssetPackPlain {
        value: amount,
        blinding: blind.to_bytes(),
        s_out,
    };

    let bytes = pack.to_bytes();
    assert_eq!(bytes.len(), AssetPackPlain::SIZE);
    let recovered = AssetPackPlain::from_bytes(&bytes).expect("decode");
    assert_eq!(recovered, pack);

    let commitment = commit_amount(amount, &blind).expect("commit");
    let recovered_blind = Z00ZScalar::try_from_bytes(recovered.blinding).expect("scalar");
    assert!(
        verify_commitment_opening(&commitment, recovered.value, &recovered_blind).expect("opening")
    );
}

#[test]
fn test_tx_balance() {
    let value_in = 1000u64;
    let blind_in = scalar_u64(601);
    let commit_in = commit_amount(value_in, &blind_in).expect("commit in");

    let value_out_one = 600u64;
    let blind_out_one = scalar_u64(602);
    let commit_out_one = commit_amount(value_out_one, &blind_out_one).expect("commit out one");

    let value_out_two = 400u64;
    let blind_out_two = &blind_in - &blind_out_one;
    let commit_out_two = commit_amount(value_out_two, &blind_out_two).expect("commit out two");

    assert_eq!(commit_in, &commit_out_one + &commit_out_two);
}

#[test]
fn test_new_confidential_flow() {
    let definition = make_def();
    let nonce = [0x42; 32];
    let (asset, blinding) = Asset::new_confidential(definition, 1, 1_000, nonce).expect("asset");
    asset
        .verify_commitment_opening(blinding.reveal())
        .expect("open");
    asset.verify_range_proof().expect("proof");
}

#[test]
fn test_golden_vectors() {
    let vectors = [
        (
            1000u64,
            scalar_u64(42),
            "16fcf640b0b456e06d28c5ece02e64f144213de51aaa14da28a2f6d318506966",
        ),
        (
            0u64,
            scalar_u64(1),
            "e2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d76",
        ),
        (
            u64::MAX,
            scalar_u64(2),
            "6a8c4596dd0b48c69724e2a1918869daf5fd4049291b936ec21006bbe0370f03",
        ),
    ];

    for (amount, blind, expected_hex) in vectors {
        let commitment = commit_amount(amount, &blind).expect("commit");
        assert_eq!(hex::encode(commitment.as_bytes()), expected_hex);
    }
}
