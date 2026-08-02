use super::ecdh::{
    compute_stealth_dh_sender, derive_dh_key, generate_ephemeral_keypair,
    recover_stealth_dh_receiver, validate_stealth_point,
};
use crate::types::{Z00ZRistrettoPoint, Z00ZScalar};
use crate::CryptoError;
use z00z_utils::rng::MockRngProvider;

fn test_rng(seed: u64) -> rand::rngs::StdRng {
    MockRngProvider::with_u64_seed(seed).rng()
}

#[test]
#[allow(non_snake_case)]
fn test_ecdh_roundtrip_symmetry() {
    let mut rng = test_rng(101);
    let r = Z00ZScalar::random(&mut rng).unwrap();
    let view_sk = Z00ZScalar::random(&mut rng).unwrap();
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);

    let r_pub = generate_ephemeral_keypair(&r).unwrap();
    let dh_sender = compute_stealth_dh_sender(&r, &view_pk).unwrap();
    let k_dh_sender = derive_dh_key(&dh_sender);

    let dh_receiver = recover_stealth_dh_receiver(&view_sk, &r_pub).unwrap();
    let k_dh_receiver = derive_dh_key(&dh_receiver);

    assert_eq!(dh_sender.as_bytes(), dh_receiver.as_bytes());
    assert_eq!(k_dh_sender, k_dh_receiver);
}

#[test]
fn test_validate_rejects_identity() {
    let identity = Z00ZRistrettoPoint::identity();
    assert!(matches!(
        validate_stealth_point(&identity),
        Err(CryptoError::IdentityPoint)
    ));

    let mut rng = test_rng(102);
    let valid_point = Z00ZRistrettoPoint::from_secret_key(&Z00ZScalar::random(&mut rng).unwrap());
    assert!(validate_stealth_point(&valid_point).is_ok());
}

#[test]
fn test_different_r_different_dh() {
    let mut rng = test_rng(103);
    let view_sk = Z00ZScalar::random(&mut rng).unwrap();
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);
    let r1 = Z00ZScalar::random(&mut rng).unwrap();
    let r2 = Z00ZScalar::random(&mut rng).unwrap();

    let dh1 = compute_stealth_dh_sender(&r1, &view_pk).unwrap();
    let dh2 = compute_stealth_dh_sender(&r2, &view_pk).unwrap();

    assert_ne!(dh1.as_bytes(), dh2.as_bytes());

    let k_dh1 = derive_dh_key(&dh1);
    let k_dh2 = derive_dh_key(&dh2);
    assert_ne!(k_dh1, k_dh2);
}

#[test]
fn test_derive_dh_key_deterministic() {
    let mut rng = test_rng(104);
    let dh = Z00ZRistrettoPoint::from_secret_key(&Z00ZScalar::random(&mut rng).unwrap());

    let k1 = derive_dh_key(&dh);
    let k2 = derive_dh_key(&dh);

    assert_eq!(k1, k2, "derive_dh_key must be deterministic");
}

#[test]
fn test_ecdh_performance() {
    use std::time::Instant;

    let mut rng = test_rng(105);
    let r = Z00ZScalar::random(&mut rng).unwrap();
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&Z00ZScalar::random(&mut rng).unwrap());

    for _ in 0..10 {
        let _ = compute_stealth_dh_sender(&r, &view_pk).unwrap();
    }

    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = compute_stealth_dh_sender(&r, &view_pk).unwrap();
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() / iterations;
    let max_us = if cfg!(debug_assertions) { 15_000 } else { 250 };
    assert!(
        avg_us < max_us,
        "ECDH too slow: {}μs (expected <{}μs)",
        avg_us,
        max_us
    );
}
