#![allow(deprecated)]

use std::collections::HashSet;

use rand::RngCore;
use z00z_crypto::{hash::poseidon2_hash, hash_to_scalar_domain, Z00ZRistrettoPoint, Z00ZScalar};
use z00z_utils::rng::SystemRngProvider;
use z00z_wallets::{
    stealth::ecdh::{receiver_derive_dh, sender_derive_dh_with_r},
    stealth::kdf::derive_k_dh,
    WalletError,
};

#[test]
fn test_shared_dh_eq() {
    let mut rng = SystemRngProvider.rng();

    for _ in 0..1000 {
        let mut receiver_secret = [0u8; 32];
        rng.fill_bytes(&mut receiver_secret);

        let view_sk = hash_to_scalar_domain(b"z00z.consensus.view_key.v1", &[&receiver_secret]);
        let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);

        let r = Z00ZScalar::random(&mut rng).unwrap();
        let sender = sender_derive_dh_with_r(&view_pk, &r).expect("sender derive failed");
        let dh_recv = receiver_derive_dh(&view_sk, &sender.r_pub).expect("receiver derive failed");

        assert_eq!(sender.dh, dh_recv, "ECDH mismatch");
        assert!(!sender.dh.is_identity(), "DH must not be identity");
    }
}

#[test]
fn test_unique_ephemeral_dh() {
    let view_sk = hash_to_scalar_domain(b"z00z.consensus.view_key.v1", &[&[0x42u8; 32]]);
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);

    let mut rng = SystemRngProvider.rng();
    let mut seen_r: HashSet<[u8; 32]> = HashSet::new();
    let mut seen_r_pub: HashSet<[u8; 32]> = HashSet::new();
    let mut seen_dh: HashSet<[u8; 32]> = HashSet::new();

    for _ in 0..1000 {
        let r = Z00ZScalar::random(&mut rng).unwrap();
        let result = sender_derive_dh_with_r(&view_pk, &r).expect("sender derive failed");
        assert!(seen_r.insert(result.r.to_bytes()), "r repeated");
        assert!(seen_r_pub.insert(result.r_pub.to_bytes()), "R_pub repeated");
        assert!(seen_dh.insert(result.dh.to_bytes()), "dh repeated");
    }
}

#[test]
fn test_receiver_isolation() {
    let bob_secret = [0x22u8; 32];
    let carol_secret = [0x33u8; 32];

    let bob_view_sk = hash_to_scalar_domain(b"z00z.consensus.view_key.v1", &[&bob_secret]);
    let bob_handle = poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&bob_secret]);
    let bob_view_pk = Z00ZRistrettoPoint::from_secret_key(&bob_view_sk);

    let r = Z00ZScalar::random(&mut SystemRngProvider.rng()).unwrap();
    let sender_res = sender_derive_dh_with_r(&bob_view_pk, &r).expect("sender derive failed");
    let bob_k_dh = derive_k_dh(&sender_res.dh.to_bytes());
    let bob_tag = poseidon2_hash(b"z00z.consensus.owner_tag.v1", &[&bob_handle, &bob_k_dh]);

    let carol_view_sk = hash_to_scalar_domain(b"z00z.consensus.view_key.v1", &[&carol_secret]);
    let carol_handle = poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&carol_secret]);
    let carol_dh =
        receiver_derive_dh(&carol_view_sk, &sender_res.r_pub).expect("carol derive failed");
    let carol_k_dh = derive_k_dh(&carol_dh.to_bytes());
    let carol_tag = poseidon2_hash(
        b"z00z.consensus.owner_tag.v1",
        &[&carol_handle, &carol_k_dh],
    );

    assert_ne!(bob_tag, carol_tag, "privacy violation");
}

#[test]
fn test_reject_identity_r_pub() {
    let view_sk = hash_to_scalar_domain(b"z00z.consensus.view_key.v1", &[&[0x42u8; 32]]);
    let id_r = Z00ZRistrettoPoint::identity();

    let result = receiver_derive_dh(&view_sk, &id_r);
    assert!(result.is_err(), "identity R_pub must be rejected");
    assert!(matches!(result, Err(WalletError::IdentityPointNotAllowed)));
}

#[test]
fn test_reject_identity_view_pk() {
    let id_view_pk = Z00ZRistrettoPoint::identity();
    let r = Z00ZScalar::random(&mut SystemRngProvider.rng()).unwrap();
    let result = sender_derive_dh_with_r(&id_view_pk, &r);
    assert!(result.is_err(), "identity view_pk must be rejected");
    assert!(matches!(result, Err(WalletError::IdentityPointNotAllowed)));
}

#[test]
fn test_reject_zero_view_sk() {
    let zero_view_sk = z00z_crypto::Z00ZScalar::zero();
    let r = Z00ZScalar::random(&mut SystemRngProvider.rng()).unwrap();
    let valid_r = sender_derive_dh_with_r(&Z00ZRistrettoPoint::generator(), &r)
        .expect("sender derive failed");
    let result = receiver_derive_dh(&zero_view_sk, &valid_r.r_pub);
    assert!(result.is_err(), "zero view_sk must be rejected");
    assert!(matches!(result, Err(WalletError::CryptoError(_))));
}
