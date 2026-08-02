//! Phase 1 ECC Primitives Tests
//!
//! SPEC Reference: specs/007-z00z-ecc-spec-2/E2E-TEST-EXAMPLES.md §1.1–§1.6
//!
//! Covers:
//! - §1.1 Point encoding roundtrip
//! - §1.2 Canonical encoding
//! - §1.3 Identity point rejection
//! - §1.4 Zero scalar rejection
//! - §1.5 Scalar arithmetic (group laws + ECDH algebraic identity)
//! - §1.6 Safe decompression (graceful failure, DoS no-panic)

use z00z_crypto::{
    protocol::ecdh::{
        compute_stealth_dh_sender, generate_ephemeral_keypair, recover_stealth_dh_receiver,
        validate_stealth_point,
    },
    types::{Z00ZRistrettoPoint, Z00ZScalar},
    validation::{safe_decompress_point, validate_scalar_nonzero},
    CryptoError,
};
use z00z_utils::rng::MockRngProvider;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn make_rng(seed: u64) -> rand::rngs::StdRng {
    MockRngProvider::with_u64_seed(seed).rng()
}

fn rand_point(rng: &mut (impl rand::RngCore + rand::CryptoRng)) -> Z00ZRistrettoPoint {
    let scalar = Z00ZScalar::random(rng).unwrap();
    Z00ZRistrettoPoint::from_secret_key(&scalar)
}

// ─── §1.1 Point Encoding Roundtrip ───────────────────────────────────────────

/// Every random point survives compress → decompress unchanged.
#[test]
fn test_point_encoding_roundtrip() {
    let mut rng = make_rng(101);
    for _ in 0..1_000 {
        let point = rand_point(&mut rng);
        let bytes = point.to_bytes();
        assert_eq!(bytes.len(), 32, "compressed point must be 32 bytes");
        let back = safe_decompress_point(&bytes).expect("roundtrip decompress failed");
        assert!(back.ct_eq(&point), "decompressed point must equal original");
    }
}

// ─── §1.2 Canonical Encoding ─────────────────────────────────────────────────

/// Same point compressed twice must give identical bytes.
#[test]
fn test_canonical_encoding_deterministic() {
    let mut rng = make_rng(102);
    let point = rand_point(&mut rng);
    let enc1 = point.to_bytes();
    let enc2 = point.to_bytes();
    assert_eq!(enc1, enc2, "point encoding must be deterministic");
}

/// Byte arrays that are definitively non-canonical must be rejected.
///
/// Only uses values guaranteed to exceed field prime p ≈ 2^255-19:
/// - `[0xFF; 32]`: all bytes 0xFF → value ≫ p
/// - `[0x80; 32]`: byte[31]=0x80 → bit 255 set → value ≥ 2^255 > p
/// - `[0xFE; 32]`: byte[31]=0xFE → bit 255 set → value ≥ 2^255 > p
///
/// NOTE: `[0x01u8; 32]` is NOT used — its value ≊ 2^248 < p is within the
/// valid field range and may decode to a valid Ristretto point.
#[test]
fn test_non_canonical_bytes_reject() {
    let cases: &[&[u8]] = &[
        &[0xFF_u8; 32], // value >> p, definitely invalid
        &[0x80_u8; 32], // bit 255 set → value >= 2^255 > p
        &[0xFE_u8; 32], // bit 255 set → value >= 2^255 > p
    ];
    for &bytes in cases {
        assert!(
            safe_decompress_point(bytes).is_err(),
            "bytes with value > p must be rejected: {:02x?}",
            &bytes[28..]
        );
    }
}

// ─── §1.3 Identity Point Rejection ───────────────────────────────────────────

/// Identity bytes ([0u8;32]) must return IdentityPoint error.
#[test]
fn test_identity_point_validate_reject() {
    let r = safe_decompress_point(&[0u8; 32]);
    assert!(
        matches!(r, Err(CryptoError::IdentityPoint)),
        "identity bytes must return CryptoError::IdentityPoint"
    );

    // Also via the point struct path
    let id = Z00ZRistrettoPoint::identity();
    let r2 = validate_stealth_point(&id);
    assert!(
        matches!(r2, Err(CryptoError::IdentityPoint)),
        "identity point struct must be rejected by validate_stealth_point"
    );
}

/// Valid non-identity point must pass validation.
#[test]
fn test_valid_point_accepted() {
    let mut rng = make_rng(103);
    let scalar = Z00ZScalar::random(&mut rng).unwrap();
    let point = Z00ZRistrettoPoint::from_secret_key(&scalar);

    assert!(
        validate_stealth_point(&point).is_ok(),
        "valid point must pass"
    );
    assert!(
        safe_decompress_point(&point.to_bytes()).is_ok(),
        "valid point bytes must decompress"
    );
}

// ─── §1.4 Zero Scalar Rejection ──────────────────────────────────────────────

/// validate_scalar_nonzero must reject Z00ZScalar::zero().
#[test]
fn test_zero_scalar_validate_reject() {
    let zero = Z00ZScalar::zero();
    let r = validate_scalar_nonzero(&zero);
    assert!(
        matches!(r, Err(CryptoError::ZeroScalar)),
        "zero scalar must return CryptoError::ZeroScalar"
    );
}

/// compute_stealth_dh_sender must reject zero ephemeral scalar.
#[test]
fn test_zero_scalar_dh_reject() {
    let mut rng = make_rng(104);
    let view_sk = Z00ZScalar::random(&mut rng).unwrap();
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);

    let r = compute_stealth_dh_sender(&Z00ZScalar::zero(), &view_pk);
    assert!(
        matches!(r, Err(CryptoError::ZeroScalar)),
        "zero ephemeral r must be rejected"
    );
}

/// generate_ephemeral_keypair must reject zero scalar.
///
/// r=0 → R_pub = identity → SPEC §2.1.3 Rule 3 violation.
#[test]
fn test_ephemeral_zero_r_rejected() {
    let r = generate_ephemeral_keypair(&Z00ZScalar::zero());
    assert!(
        matches!(r, Err(CryptoError::ZeroScalar)),
        "generate_ephemeral_keypair must reject zero scalar"
    );
}

/// recover_stealth_dh_receiver must reject zero view_sk.
///
/// view_sk=0 → dh = 0*R_pub = identity → k_dh identical for all receivers.
#[test]
fn test_recover_zero_sk_reject() {
    let mut rng = make_rng(110);
    let r = Z00ZScalar::random(&mut rng).unwrap();
    let r_pub = Z00ZRistrettoPoint::from_secret_key(&r);

    let result = recover_stealth_dh_receiver(&Z00ZScalar::zero(), &r_pub);
    assert!(
        matches!(result, Err(CryptoError::ZeroScalar)),
        "zero view_sk must return CryptoError::ZeroScalar"
    );
}

// ─── §1.5 Scalar Arithmetic: Group Laws ──────────────────────────────────────

/// (a + b) * P == a*P + b*P  (distributivity).
#[test]
fn test_scalar_mul_distributivity() {
    let mut rng = make_rng(105);
    let a = Z00ZScalar::random(&mut rng).unwrap();
    let b = Z00ZScalar::random(&mut rng).unwrap();
    let p = rand_point(&mut rng);

    let ab = &a + &b;
    let lhs = &p * &ab;
    let pa = &p * &a;
    let pb = &p * &b;
    let rhs = &pa + &pb;

    assert!(lhs.ct_eq(&rhs), "(a+b)*P must equal a*P + b*P");
}

/// a*(b*P) == b*(a*P)  (commutativity in exponent).
#[test]
fn test_scalar_exponent_commutativity() {
    let mut rng = make_rng(106);
    let a = Z00ZScalar::random(&mut rng).unwrap();
    let b = Z00ZScalar::random(&mut rng).unwrap();
    let p = rand_point(&mut rng);

    let pa = &p * &a;
    let pb = &p * &b;
    let lhs = &pa * &b; // (p*a)*b
    let rhs = &pb * &a; // (p*b)*a

    assert!(lhs.ct_eq(&rhs), "a*(b*P) must equal b*(a*P)");
}

/// ECDH algebraic identity: r*(view_sk*G) == view_sk*(r*G).
///
/// This is the foundational correctness property for stealth addresses.
#[test]
#[allow(non_snake_case)]
fn test_ecdh_algebraic_identity() {
    let mut rng = make_rng(107);
    for _ in 0..100 {
        let r = Z00ZScalar::random(&mut rng).unwrap();
        let view_sk = Z00ZScalar::random(&mut rng).unwrap();

        let R_pub = Z00ZRistrettoPoint::from_secret_key(&r);
        let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);

        let dh_s =
            compute_stealth_dh_sender(&r, &view_pk).expect("sender DH must succeed for valid keys");
        let dh_r = recover_stealth_dh_receiver(&view_sk, &R_pub)
            .expect("receiver DH must succeed for valid keys");

        assert!(dh_s.ct_eq(&dh_r), "ECDH algebraic identity violated");
    }
}

// ─── §1.6 Safe Decompression: Graceful Failure ───────────────────────────────

/// Wrong-length inputs must return InvalidPointLength (not panic).
#[test]
fn test_safe_decompress_wrong_length() {
    assert!(
        matches!(
            safe_decompress_point(&[0u8; 16]),
            Err(CryptoError::InvalidPointLength)
        ),
        "16-byte input must give InvalidPointLength"
    );
    assert!(
        matches!(
            safe_decompress_point(&[0u8; 64]),
            Err(CryptoError::InvalidPointLength)
        ),
        "64-byte input must give InvalidPointLength"
    );
    assert!(
        matches!(
            safe_decompress_point(&[]),
            Err(CryptoError::InvalidPointLength)
        ),
        "empty slice must give InvalidPointLength"
    );
}

/// Non-canonical 32-byte input must return InvalidPoint (not panic).
#[test]
fn test_safe_decompress_noncanonical() {
    let mut nc = [0u8; 32];
    nc[31] = 0xFF;
    assert!(
        matches!(safe_decompress_point(&nc), Err(CryptoError::InvalidPoint)),
        "high-bit-set input must return InvalidPoint"
    );
}

/// Identity 32-zero-bytes must return IdentityPoint (not panic).
#[test]
fn test_safe_decompress_identity_rejected() {
    assert!(
        matches!(
            safe_decompress_point(&[0u8; 32]),
            Err(CryptoError::IdentityPoint)
        ),
        "identity bytes must return IdentityPoint"
    );
}

/// Valid compressed point must decompress successfully.
#[test]
fn test_safe_decompress_valid_point() {
    let mut rng = make_rng(108);
    let point = rand_point(&mut rng);
    assert!(
        safe_decompress_point(&point.to_bytes()).is_ok(),
        "valid point bytes must decompress"
    );
}

/// 100 000 random byte arrays must never cause a panic.
///
/// DoS guard: adversary cannot crash wallet with malformed R_pub inputs.
#[test]
fn test_decompress_dos_no_panic() {
    use rand::RngCore;

    let mut rng = make_rng(109);
    let mut bytes = [0u8; 32];
    for _ in 0..100_000 {
        rng.fill_bytes(&mut bytes);
        // Only Ok or Err are acceptable – any panic fails the test
        let _ = safe_decompress_point(&bytes);
    }
}
