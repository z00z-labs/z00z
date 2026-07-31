use super::*;
use crate::secret::SecretBytes;
use tari_crypto::{
    commitment::HomomorphicCommitmentFactory, keys::SecretKey,
    ristretto::pedersen::commitment_factory::PedersenCommitmentFactory, tari_utilities::ByteArray,
};
use z00z_utils::rng::MockRngProvider;
use zeroize::Zeroize;

fn test_rng(seed: u64) -> rand::rngs::StdRng {
    MockRngProvider::with_u64_seed(seed).rng()
}

#[test]
fn test_constants_are_correct() {
    assert_eq!(VERSION, 1);
    assert_eq!(VERSION_BYTES, 4);
    assert_eq!(LENGTH_BYTES, 4);
    assert_eq!(CHECKSUM_BYTES, 32);
    assert_eq!(RANGE_PROOF_BITS, 64);
    assert_eq!(RANGE_PROOF_BITS_EXTENDED, 128);
    assert_eq!(AGGREGATION_FACTOR, 1);
    assert_eq!(MAX_PROOF_SIZE, 10_240);
    assert_eq!(MAX_PROOF_SIZE_EXTENDED, 20_480);
}

#[test]
fn test_transfer_zero_reject() {
    assert!(validate_transfer_amount(0).is_err());
    assert!(validate_transfer_amount(1).is_ok());
}

#[test]
fn test_asset_zero_strict() {
    assert!(validate_asset_amount(0, false).is_err());
    assert!(validate_asset_amount(1, false).is_ok());
}

#[test]
fn test_asset_zero_allow() {
    assert!(validate_asset_amount(0, true).is_ok());
    assert!(validate_asset_amount(1, true).is_ok());
}

#[test]
fn test_amount_relaxed_allows_zero() {
    assert!(validate_amount_relaxed(0).is_ok());
    assert!(validate_amount_relaxed(1).is_ok());
}

#[test]
fn test_proof_size_standard_lane() {
    assert!(validate_proof_size(700, 1).is_ok());
    assert!(validate_proof_size(800, 1).is_ok());
    assert!(validate_proof_size(10_000, 1).is_ok());
    assert!(validate_proof_size(10_240, 1).is_ok());
    assert!(validate_proof_size(10_241, 1).is_err());
    assert!(validate_proof_size(20_000, 1).is_err());
}

#[test]
fn test_proof_size_extended_lane() {
    assert!(validate_proof_size(1400, 2).is_ok());
    assert!(validate_proof_size(1600, 2).is_ok());
    assert!(validate_proof_size(20_000, 2).is_ok());
    assert!(validate_proof_size(20_480, 2).is_ok());
    assert!(validate_proof_size(20_481, 2).is_err());
    assert!(validate_proof_size(30_000, 2).is_err());
}

#[test]
fn test_proof_size_invalid_version() {
    assert!(validate_proof_size(1000, 0).is_err());
    assert!(validate_proof_size(1000, 3).is_err());
    assert!(validate_proof_size(1000, 999).is_err());
}

#[test]
fn test_commitment_validation() {
    let factory = PedersenCommitmentFactory::default();
    let mut rng = test_rng(42);
    let blinding = RistrettoSecretKey::random(&mut rng);

    let commitment = factory.commit_value(&blinding, 1000);
    let wrapped_commitment = Z00ZCommitment::from_commitment(commitment);
    assert!(validate_commitment_non_zero(&wrapped_commitment).is_ok());

    let zero_blinding = RistrettoSecretKey::from_canonical_bytes(&[0u8; 32]).unwrap();
    let zero_commitment = factory.commit_value(&zero_blinding, 0);
    let wrapped_zero_commitment = Z00ZCommitment::from_commitment(zero_commitment);
    assert!(validate_commitment_non_zero(&wrapped_zero_commitment).is_err());
}

#[test]
fn test_z00z_scalar_from_bytes() {
    let valid_bytes = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x03,
    ];
    let wrapper = Z00ZScalar::try_from_bytes(valid_bytes).unwrap();
    assert_eq!(wrapper.as_bytes(), valid_bytes);
}

#[test]
fn test_z00z_scalar_allows_zero() {
    let scalar = Z00ZScalar::try_from_bytes([0u8; 32]).unwrap();
    assert!(scalar.is_zero());
}

#[test]
fn test_z00z_commitment() {
    let factory = PedersenCommitmentFactory::default();
    let mut rng = test_rng(42);
    let blinding = RistrettoSecretKey::random(&mut rng);
    let commitment = factory.commit_value(&blinding, 1000);

    let wrapper = Z00ZCommitment::from_commitment(commitment.clone());
    let inner = wrapper.reveal();
    assert_eq!(inner, &commitment);
    let bytes = wrapper.to_bytes();
    assert_eq!(bytes, commitment.as_bytes());
}

#[test]
fn test_blinding_dangerous_clone() {
    let mut rng = test_rng(42);
    let bf1 = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let bf2 = bf1.dangerous_clone();
    assert!(bf1.ct_eq(&bf2));
    assert_ne!(bf1.as_bytes().as_ptr(), bf2.as_bytes().as_ptr());
}

#[test]
fn test_blinding_ct_eq_same() {
    let mut rng = test_rng(42);
    let bf1 = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let bf2 = bf1.dangerous_clone();
    assert!(bf1.ct_eq(&bf2));
}

#[test]
fn test_blinding_ct_eq_different() {
    let mut rng1 = test_rng(42);
    let bf1 = Z00ZScalar::random_from_rng(&mut rng1).unwrap();
    let mut rng2 = test_rng(99);
    let bf2 = Z00ZScalar::random_from_rng(&mut rng2).unwrap();
    assert!(!bf1.ct_eq(&bf2));
}

#[test]
fn test_blinding_to_bytes() {
    let bf = Z00ZScalar::try_from_bytes([1u8; 32]).unwrap();
    assert_eq!(bf.to_bytes(), [1u8; 32]);
}

#[test]
fn test_z00z_blinding_factor_zeroization() {
    use zeroize::Zeroize;

    let mut rng = test_rng(42);
    let mut bf = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let bytes_before = bf.as_bytes().to_vec();
    assert_ne!(bytes_before, vec![0u8; 32]);
    bf.zeroize();
    let bytes_after = bf.as_bytes();
    assert_eq!(bytes_after, &[0u8; 32]);
}

#[test]
fn test_blinding_drop_clears() {
    let mut rng = test_rng(42);
    let bf = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let bytes_before = bf.as_bytes().to_vec();
    drop(bf);
    assert_ne!(bytes_before, vec![0u8; 32]);
}

#[test]
fn test_blinding_from_uniform() {
    let uniform_bytes = [42u8; 64];
    let bf =
        Z00ZScalar::from_uniform_bytes(&uniform_bytes).expect("Should create from uniform bytes");
    assert_ne!(bf.as_bytes(), &[0u8; 32]);
}

#[test]
fn test_blinding_try_bytes_valid() {
    let valid_bytes = [1u8; 32];
    let result = Z00ZScalar::try_from_bytes(valid_bytes);
    assert!(result.is_ok());
}

#[test]
fn test_blinding_try_bytes_invalid() {
    let invalid_bytes = [0xFFu8; 32];
    let result = Z00ZScalar::try_from_bytes(invalid_bytes);
    assert!(result.is_err());
}

#[test]
fn test_blinding_algebra_add() {
    let mut rng = test_rng(42);
    let bf1 = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let bf2 = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let sum = &bf1 + &bf2;
    assert_ne!(sum.as_bytes(), &[0u8; 32]);
}

#[test]
fn test_blinding_algebra_sub() {
    let mut rng = test_rng(42);
    let bf1 = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let bf2 = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let _diff = &bf1 - &bf2;
    let self_diff = &bf1 - &bf1;
    assert_eq!(self_diff.as_bytes(), &[0u8; 32]);
}

#[test]
fn test_blinding_algebra_identity() {
    let mut rng = test_rng(42);
    let bf = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let another = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let zero = &another - &another;
    let bf_plus_zero = &bf + &zero;
    let bf_minus_zero = &bf - &zero;
    assert!(bf.ct_eq(&bf_plus_zero));
    assert!(bf.ct_eq(&bf_minus_zero));
}

#[test]
fn test_protocol_version_documentation() {
    assert_eq!(VERSION, 1);
}

#[test]
fn test_do_s_protection_limits() {
    let max_1 = MAX_PROOF_SIZE;
    let max_2 = MAX_PROOF_SIZE_EXTENDED;
    assert!(max_1 > 1000);
    assert!(max_2 > max_1);
    let legacy_actual_max = 800;
    assert!(MAX_PROOF_SIZE > legacy_actual_max * 10);
}

#[test]
fn test_compressed_roundtrip() {
    let bf = Z00ZScalar::try_from_bytes([2u8; 32]).unwrap();
    let point = Z00ZRistrettoPoint::from_secret_key(&bf);
    let compressed = point.compress();
    assert_eq!(compressed.to_bytes().len(), 32);
    let recovered = compressed.decompress().expect("decompress should work");
    assert!(recovered.ct_eq(&point));
}

#[test]
fn test_scalar_bytes_roundtrip() {
    let provider = MockRngProvider::with_u64_seed(42);
    let original = Z00ZScalar::random_deterministic(&provider).unwrap();
    let bytes = original.to_bytes();
    let recovered = Z00ZScalar::from_canonical_bytes(&bytes).unwrap();
    assert!(original.ct_eq(&recovered));
}

#[test]
fn try_from_nonzero_secret_bytes_zeroizes_all_paths() {
    let cases = [
        (
            vec![
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            true,
        ),
        (vec![0; 32], false),
        (vec![0xff; 32], false),
        (vec![7; 31], false),
        (vec![7; 33], false),
    ];

    for (bytes, should_succeed) in cases {
        let mut owner = SecretBytes::new(bytes);
        let mut result = Z00ZScalar::try_from_nonzero_secret_bytes(&mut owner);
        assert_eq!(result.is_ok(), should_succeed);
        assert!(owner.as_slice().iter().all(|byte| *byte == 0));
        if let Ok(scalar) = result.as_mut() {
            assert!(
                !scalar.is_zero(),
                "valid secret must produce a non-zero scalar"
            );
            scalar.zeroize();
            assert_eq!(scalar.to_bytes(), Z00ZScalar::ZERO_BYTES);
        }
    }
}

#[test]
fn test_scalar_tari_roundtrip() {
    let provider = MockRngProvider::with_u64_seed(43);
    let original = Z00ZScalar::random_deterministic(&provider).unwrap();
    let tari_secret = RistrettoSecretKey::try_from(original.dangerous_clone()).unwrap();
    let recovered = Z00ZScalar::try_from(tari_secret).unwrap();
    assert!(original.ct_eq(&recovered));
}

#[test]
fn test_point_bytes_roundtrip() {
    let provider = MockRngProvider::with_u64_seed(44);
    let scalar = Z00ZScalar::random_deterministic(&provider).unwrap();
    let point = Z00ZRistrettoPoint::from_secret_key(&scalar);
    let compressed = point.compress();
    let bytes = compressed.to_bytes();
    let recovered = Z00ZCompressedRistretto::from_bytes(&bytes)
        .decompress()
        .unwrap();
    assert!(point.ct_eq(&recovered));
}

#[test]
fn test_tari_bidir_interop() {
    let mut rng = test_rng(45);
    let tari_secret = RistrettoSecretKey::random(&mut rng);
    let z_scalar = Z00ZScalar::try_from(tari_secret.clone()).unwrap();
    let restored_secret = RistrettoSecretKey::try_from(z_scalar).unwrap();
    assert_eq!(tari_secret.as_bytes(), restored_secret.as_bytes());

    use tari_crypto::keys::PublicKey as _;
    let tari_point = RistrettoPublicKey::from_secret_key(&tari_secret);
    let z_point = Z00ZRistrettoPoint::try_from(tari_point.clone()).unwrap();
    let restored_point = RistrettoPublicKey::try_from(z_point).unwrap();
    assert_eq!(tari_point, restored_point);
}

#[test]
fn test_ct_eq_contract() {
    let provider_a = MockRngProvider::with_u64_seed(46);
    let provider_b = MockRngProvider::with_u64_seed(47);
    let scalar_a = Z00ZScalar::random_deterministic(&provider_a).unwrap();
    let scalar_b = Z00ZScalar::random_deterministic(&provider_b).unwrap();
    assert!(scalar_a.ct_eq(&scalar_a));
    assert!(!scalar_a.ct_eq(&scalar_b));
}

#[test]
fn test_scalar_field_laws() {
    let mut rng = test_rng(48);
    let scalar_a = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let scalar_b = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let scalar_c = Z00ZScalar::random_from_rng(&mut rng).unwrap();

    let left_assoc = &(&scalar_a + &scalar_b) + &scalar_c;
    let right_assoc = &scalar_a + &(&scalar_b + &scalar_c);
    assert!(left_assoc.ct_eq(&right_assoc));

    let add_ab = &scalar_a + &scalar_b;
    let add_ba = &scalar_b + &scalar_a;
    assert!(add_ab.ct_eq(&add_ba));

    let add_id = &scalar_a + &Z00ZScalar::zero();
    assert!(add_id.ct_eq(&scalar_a));

    let add_inv = &scalar_a + &(-&scalar_a);
    assert!(add_inv.ct_eq(&Z00ZScalar::zero()));

    let mul_dist_left = &scalar_a * &(&scalar_b + &scalar_c);
    let mul_dist_right = &(&scalar_a * &scalar_b) + &(&scalar_a * &scalar_c);
    assert!(mul_dist_left.ct_eq(&mul_dist_right));
}

#[test]
fn test_point_group_laws() {
    let mut rng = test_rng(49);
    let scalar_a = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let scalar_b = Z00ZScalar::random_from_rng(&mut rng).unwrap();
    let generator = Z00ZRistrettoPoint::generator();

    let add_id = &generator + &Z00ZRistrettoPoint::identity();
    assert!(add_id.ct_eq(&generator));

    let add_inv = &generator + &(-&generator);
    assert!(add_inv.ct_eq(&Z00ZRistrettoPoint::identity()));

    let left_lin = &generator * &(&scalar_a + &scalar_b);
    let right_lin = &(&generator * &scalar_a) + &(&generator * &scalar_b);
    assert!(left_lin.ct_eq(&right_lin));
}

#[test]
fn test_identity_encode_zero() {
    let identity = Z00ZRistrettoPoint::identity();
    let compressed = identity.compress();
    assert_eq!(compressed.as_bytes(), &[0u8; 32]);
}

#[test]
fn test_non_identity_not_zero() {
    let provider = MockRngProvider::with_u64_seed(50);
    let scalar = Z00ZScalar::random_deterministic(&provider).unwrap();
    let point = Z00ZRistrettoPoint::from_secret_key(&scalar);
    assert!(!point.is_identity());
}

#[test]
fn test_mock_rng_repeat() {
    let provider_a = MockRngProvider::with_u64_seed(51);
    let provider_b = MockRngProvider::with_u64_seed(51);
    let scalar_a = Z00ZScalar::random_deterministic(&provider_a).unwrap();
    let scalar_b = Z00ZScalar::random_deterministic(&provider_b).unwrap();
    assert!(scalar_a.ct_eq(&scalar_b));
}

#[test]
fn test_basepoint_table_multiply() {
    let scalar = Z00ZScalar::try_from_bytes([3u8; 32]).unwrap();
    let table = Z00ZBasepointTable::GENERATOR_TABLE;
    let point = table.multiply(&scalar);
    assert!(!point.is_identity());
}

#[test]
fn test_proof_size_boundaries() {
    assert!(validate_proof_size(MAX_PROOF_SIZE, 1).is_ok());
    assert!(validate_proof_size(MAX_PROOF_SIZE_EXTENDED, 2).is_ok());
    assert!(validate_proof_size(MAX_PROOF_SIZE + 1, 1).is_err());
    assert!(validate_proof_size(MAX_PROOF_SIZE_EXTENDED + 1, 2).is_err());
}

#[test]
fn test_wrapper_equality() {
    let bytes = [1u8; 32];
    let w1 = Z00ZScalar::try_from_bytes(bytes).unwrap();
    let w2 = Z00ZScalar::try_from_bytes(bytes).unwrap();
    assert!(w1.ct_eq(&w2));
}

#[test]
fn test_blinding_factor_dangerous_clone() {
    let bytes = [1u8; 32];
    let bf1 = Z00ZScalar::try_from_bytes(bytes).unwrap();
    let bf2 = bf1.dangerous_clone();
    assert!(bf1.ct_eq(&bf2));
}

#[test]
fn test_blind_ct_eq_diff() {
    let bf1 = Z00ZScalar::try_from_bytes([1u8; 32]).unwrap();
    let bf2 = Z00ZScalar::try_from_bytes([2u8; 32]).unwrap();
    assert!(!bf1.ct_eq(&bf2));
}

#[test]
fn test_point_mul_matches_public() {
    let scalar = Z00ZScalar::try_from_bytes([4u8; 32]).unwrap();
    let point1 = &Z00ZRistrettoPoint::generator() * &scalar;
    let point2 = Z00ZRistrettoPoint::from_secret_key(&scalar);
    assert!(point1.ct_eq(&point2));
}

#[test]
fn test_error_messages() {
    let err = validate_transfer_amount(0).unwrap_err();
    assert!(err.to_string().contains("amount"));

    let err = validate_proof_size(50000, 1).unwrap_err();
    assert!(err.to_string().contains("parameter"));
}
