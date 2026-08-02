//! Encoding utilities for stealth Ristretto public points.

use z00z_crypto::Z00ZRistrettoPoint;

use crate::stealth::StealthError;

/// Encode a public key into 32-byte canonical form.
pub fn encode_public_key(pk: &Z00ZRistrettoPoint) -> [u8; 32] {
    let mut out = [0u8; 32];
    out.copy_from_slice(pk.as_bytes());
    out
}

/// Decode and validate a compressed public key.
pub fn decode_public_key(bytes: &[u8; 32]) -> Result<Z00ZRistrettoPoint, StealthError> {
    Z00ZRistrettoPoint::try_from_bytes(*bytes).map_err(|error| match error {
        z00z_crypto::CryptoError::IdentityPoint => StealthError::IdentityPointRejected,
        _ => StealthError::InvalidRistrettoPoint,
    })
}

/// Encode ephemeral `R_pub` point.
pub fn encode_r_pub(r_pub: &Z00ZRistrettoPoint) -> [u8; 32] {
    encode_public_key(r_pub)
}

/// Decode ephemeral `R_pub` point with identity rejection.
pub fn decode_r_pub(bytes: &[u8; 32]) -> Result<Z00ZRistrettoPoint, StealthError> {
    decode_public_key(bytes)
}

#[cfg(test)]
mod tests {
    use super::{decode_public_key, decode_r_pub, encode_public_key, encode_r_pub};
    use z00z_crypto::{Z00ZRistrettoPoint, Z00ZScalar};
    use z00z_utils::rng::SystemRngProvider;

    fn sample_pk() -> Z00ZRistrettoPoint {
        let provider = SystemRngProvider;
        let mut rng = provider.rng();
        let sk = Z00ZScalar::random(&mut rng).unwrap();
        Z00ZRistrettoPoint::from_secret_key(&sk)
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let pk = sample_pk();
        let bytes = encode_public_key(&pk);
        let decoded = decode_public_key(&bytes).expect("decode");
        assert_eq!(decoded.as_bytes(), pk.as_bytes());
    }

    #[test]
    fn test_identity_point_rejection() {
        let identity = [0u8; 32];
        assert!(decode_public_key(&identity).is_err());
    }

    #[test]
    fn test_invalid_point_encoding() {
        let mut bytes = [0xffu8; 32];
        bytes[31] = 0x7f;
        assert!(decode_public_key(&bytes).is_err());
    }

    #[test]
    fn test_r_pub_encode_decode() {
        let r_pub = sample_pk();
        let bytes = encode_r_pub(&r_pub);
        let decoded = decode_r_pub(&bytes).expect("decode");
        assert_eq!(decoded.as_bytes(), r_pub.as_bytes());
    }
}
