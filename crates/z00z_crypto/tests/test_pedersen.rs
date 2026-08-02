use rand::RngCore;
use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::vendor::tari::PedersenCommitmentFactory;
use z00z_crypto::{
    commitment::{commit_value, h_base, verify_opening},
    HomomorphicCommitmentFactory, Z00ZScalar,
};

#[test]
fn test_pedersen_roundtrip() {
    let mut rng = z00z_utils::rng::SystemRngProvider.rng();
    let values = [0u64, 1, 100, 12345, u64::MAX];

    for value in values {
        let blinding = Z00ZScalar::random(&mut rng).unwrap();
        let c = commit_value(value, &blinding);

        assert!(
            verify_opening(&c, value, &blinding),
            "correct opening rejected for value={}",
            value
        );
        assert!(
            !verify_opening(&c, value.wrapping_add(1), &blinding),
            "wrong value accepted"
        );

        let mut wrong_r = Z00ZScalar::random(&mut rng).unwrap();
        while wrong_r.to_bytes() == blinding.to_bytes() {
            wrong_r = Z00ZScalar::random(&mut rng).unwrap();
        }
        assert!(
            !verify_opening(&c, value, &wrong_r),
            "wrong blinding accepted"
        );
    }
}

#[test]
fn test_pedersen_homomorphism() {
    let mut rng = z00z_utils::rng::SystemRngProvider.rng();
    let first_value = 100u64;
    let second_value = 200u64;
    let r1 = Z00ZScalar::random(&mut rng).unwrap();
    let r2 = Z00ZScalar::random(&mut rng).unwrap();

    let c1 = commit_value(first_value, &r1);
    let c2 = commit_value(second_value, &r2);
    let c_sum = &c1 + &c2;
    let r_sum = &r1 + &r2;

    assert!(
        verify_opening(&c_sum, first_value + second_value, &r_sum),
        "homomorphism violated"
    );
}

#[test]
fn test_pedersen_balance_simulation() {
    let mut rng = z00z_utils::rng::SystemRngProvider.rng();
    let r_in = Z00ZScalar::random(&mut rng).unwrap();
    let r_out1 = Z00ZScalar::random(&mut rng).unwrap();
    let r_out2 = Z00ZScalar::random(&mut rng).unwrap();

    let c_in = commit_value(300, &r_in);
    let c_out1 = commit_value(100, &r_out1);
    let c_out2 = commit_value(200, &r_out2);

    let r_excess = &(&r_in - &r_out1) - &r_out2;
    let c_excess = &(&c_in - &c_out1) - &c_out2;

    assert!(
        verify_opening(&c_excess, 0, &r_excess),
        "balance proof failed"
    );
}

#[test]
fn test_pedersen_diff_blindings() {
    let mut rng = z00z_utils::rng::SystemRngProvider.rng();

    for _ in 0..10_000 {
        let v = rng.next_u64();
        let r1 = Z00ZScalar::random(&mut rng).unwrap();
        let mut r2 = Z00ZScalar::random(&mut rng).unwrap();
        while r2.to_bytes() == r1.to_bytes() {
            r2 = Z00ZScalar::random(&mut rng).unwrap();
        }
        let c1 = commit_value(v, &r1);
        let c2 = commit_value(v, &r2);

        assert_ne!(
            c1.as_bytes(),
            c2.as_bytes(),
            "same value with different blindings must yield different commitments"
        );
    }
}

#[test]
fn test_generators_independence() {
    let factory = PedersenCommitmentFactory::default();
    let g = factory.commit_value(&tari_crypto::ristretto::RistrettoSecretKey::from(0u64), 1);
    let h = h_base();
    let h2 = h_base();
    let id = factory.commit_value(&tari_crypto::ristretto::RistrettoSecretKey::from(0u64), 0);

    assert_ne!(h.as_bytes(), g.as_bytes(), "H must differ from G");
    assert_ne!(h.as_bytes(), id.as_bytes(), "H must not be identity");
    assert_eq!(h.as_bytes(), h2.as_bytes(), "H must be deterministic");
}
