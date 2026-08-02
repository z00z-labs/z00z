//! Reference-model stealth tests inspired by stealth_address_kit.
//!
//! These tests intentionally target only the algebraic core shared with the
//! reference model. They do not claim that the full Z00Z stealth protocol is
//! equivalent to stealth_address_kit.

use std::convert::TryInto;

use z00z_crypto::{
    domains::HashToScalarDomain,
    hash::hash_to_scalar_zk,
    protocol::ecdh::{
        compute_stealth_dh_sender, generate_ephemeral_keypair, recover_stealth_dh_receiver,
    },
    Z00ZRistrettoPoint, Z00ZScalar,
};
use z00z_utils::rng::MockRngProvider;

fn make_rng(seed: u64) -> rand::rngs::StdRng {
    MockRngProvider::with_u64_seed(seed).rng()
}

fn ref_hash_scalar(point: &Z00ZRistrettoPoint) -> Z00ZScalar {
    hash_to_scalar_zk::<HashToScalarDomain>("REF/STEALTH_ADDRESS_KIT/H2S", &[point.as_bytes()])
        .expect("reference scalar")
}

fn ref_view_tag(scalar: &Z00ZScalar) -> u64 {
    let bytes = scalar.to_bytes();
    u64::from_le_bytes(bytes[..8].try_into().expect("tag bytes"))
}

fn ref_addr(
    view_pk: &Z00ZRistrettoPoint,
    spend_pk: &Z00ZRistrettoPoint,
    eph_sk: &Z00ZScalar,
) -> (Z00ZRistrettoPoint, u64) {
    let shared = compute_stealth_dh_sender(eph_sk, view_pk).expect("sender shared point");
    let tweak = ref_hash_scalar(&shared);
    let tweak_pk = Z00ZRistrettoPoint::from_secret_key(&tweak);
    (&tweak_pk + spend_pk, ref_view_tag(&tweak))
}

fn ref_priv(
    eph_pk: &Z00ZRistrettoPoint,
    view_sk: &Z00ZScalar,
    spend_sk: &Z00ZScalar,
    expect_tag: u64,
) -> Option<Z00ZScalar> {
    let shared = recover_stealth_dh_receiver(view_sk, eph_pk).expect("receiver shared point");
    let tweak = ref_hash_scalar(&shared);
    if ref_view_tag(&tweak) != expect_tag {
        return None;
    }

    Some(spend_sk + &tweak)
}

#[test]
fn test_ref_pub_from_scalar() {
    let mut rng = make_rng(9001);
    let sk = Z00ZScalar::random(&mut rng).unwrap();
    let pk = Z00ZRistrettoPoint::from_secret_key(&sk);

    assert_eq!(
        Z00ZRistrettoPoint::from_secret_key(&sk).to_bytes(),
        pk.to_bytes()
    );
}

#[test]
fn test_ref_hash_scalar_diff() {
    let mut rng = make_rng(9002);
    let point_a = Z00ZRistrettoPoint::from_secret_key(&Z00ZScalar::random(&mut rng).unwrap());
    let point_b = Z00ZRistrettoPoint::from_secret_key(&Z00ZScalar::random(&mut rng).unwrap());

    let hash_a = ref_hash_scalar(&point_a);
    let hash_b = ref_hash_scalar(&point_b);

    assert_ne!(hash_a.to_bytes(), hash_b.to_bytes());
}

#[test]
fn test_ref_shared_point_eq() {
    let mut rng = make_rng(9003);
    let key_a = Z00ZScalar::random(&mut rng).unwrap();
    let key_b = Z00ZScalar::random(&mut rng).unwrap();
    let pub_a = Z00ZRistrettoPoint::from_secret_key(&key_a);
    let pub_b = Z00ZRistrettoPoint::from_secret_key(&key_b);

    let left = compute_stealth_dh_sender(&key_a, &pub_b).expect("left");
    let right = compute_stealth_dh_sender(&key_b, &pub_a).expect("right");

    assert_eq!(left.to_bytes(), right.to_bytes());
}

#[test]
fn test_ref_private_public_match() {
    let mut rng = make_rng(9004);
    let spend_sk = Z00ZScalar::random(&mut rng).unwrap();
    let spend_pk = Z00ZRistrettoPoint::from_secret_key(&spend_sk);
    let view_sk = Z00ZScalar::random(&mut rng).unwrap();
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);
    let eph_sk = Z00ZScalar::random(&mut rng).unwrap();
    let eph_pk = generate_ephemeral_keypair(&eph_sk).expect("ephemeral public key");

    let (stealth_pk, view_tag) = ref_addr(&view_pk, &spend_pk, &eph_sk);
    let stealth_sk = ref_priv(&eph_pk, &view_sk, &spend_sk, view_tag).expect("stealth key");
    let derived_pk = Z00ZRistrettoPoint::from_secret_key(&stealth_sk);

    assert_eq!(derived_pk.to_bytes(), stealth_pk.to_bytes());
}

#[test]
fn test_ref_view_tag_gate() {
    let mut rng = make_rng(9005);
    let spend_sk = Z00ZScalar::random(&mut rng).unwrap();
    let view_sk = Z00ZScalar::random(&mut rng).unwrap();
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);
    let eph_sk = Z00ZScalar::random(&mut rng).unwrap();
    let eph_pk = generate_ephemeral_keypair(&eph_sk).expect("ephemeral public key");

    let (_, good_tag) = ref_addr(
        &view_pk,
        &Z00ZRistrettoPoint::from_secret_key(&spend_sk),
        &eph_sk,
    );

    assert!(ref_priv(&eph_pk, &view_sk, &spend_sk, good_tag ^ 1).is_none());
}
