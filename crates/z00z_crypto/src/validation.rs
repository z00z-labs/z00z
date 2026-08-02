//! ECC Point and Scalar Validation.
//!
//! # SPEC Reference
//!
//! - §2.1.3 - ECC Validity Rules
//!
//! # Security Policy
//!
//! All untrusted points (from network, user input) MUST be validated:
//! - Rule 1: Canonical encoding (reject malformed points)
//! - Rule 2: Identity rejection (prevent linkability attacks)
//! - Rule 3: Scalar non-zero check (for ephemeral secrets)
//!
//! # Usage
//!
//! ```rust
//! use z00z_crypto::validation::{safe_decompress_point, validate_scalar_nonzero};
//! use z00z_crypto::types::{Z00ZRistrettoPoint, Z00ZScalar};
//! use z00z_utils::rng::MockRngProvider;
//!
//! let provider = MockRngProvider::with_u64_seed(42);
//! let mut rng = provider.rng();
//!
//! // Validate untrusted point
//! let seed = Z00ZScalar::random(&mut rng).unwrap();
//! let point = Z00ZRistrettoPoint::from_secret_key(&seed);
//! let point_bytes = point.as_bytes();
//! let point = safe_decompress_point(&point_bytes)?;
//!
//! // Validate scalar before use
//! let scalar = Z00ZScalar::random(&mut rng).unwrap();
//! validate_scalar_nonzero(&scalar)?;
//! # Ok::<_, z00z_crypto::CryptoError>(())
//! ```

use crate::{
    error::CryptoError,
    types::{Z00ZRistrettoPoint, Z00ZScalar},
};
use subtle::ConstantTimeEq;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    #[error("point size must be 32 bytes, got {0}")]
    InvalidPointSize(usize),
    #[error("point decompression failed")]
    DecompressionFailed,
    #[error("identity point not allowed")]
    IdentityPointRejected,
    #[error("zero scalar not allowed")]
    ZeroScalarRejected,
    #[error("non-canonical scalar encoding")]
    NonCanonicalScalar,
}

/// Safe point decompression.
///
/// # SPEC Reference
///
/// §2.1.3 - ECC Validity Rules:
/// - Rule 1: Canonical encoding (32 bytes, valid decompression)
/// - Rule 2: Identity rejection (prevent linkability)
///
/// # Arguments
///
/// - `bytes`: Compressed point (must be exactly 32 bytes)
///
/// # Returns
///
/// - `Ok(Z00ZRistrettoPoint)`: Valid non-identity point
/// - `Err(CryptoError)`: Validation failed
///
/// # Errors
///
/// - `InvalidPointLength`: Input not 32 bytes
/// - `InvalidPoint`: Decompression failed (non-canonical or invalid)
/// - `IdentityPoint`: Point is identity (forbidden)
///
/// # Security
///
/// - Prevents identity point attacks (all receivers same k_dh)
/// - Prevents non-canonical encoding (malleability)
/// - No panics on malformed input
///
/// # Examples
///
/// ```
/// use z00z_crypto::validation::safe_decompress_point;
/// use z00z_crypto::types::{Z00ZRistrettoPoint, Z00ZScalar};
/// use z00z_utils::rng::MockRngProvider;
///
/// // Valid point
/// let provider = MockRngProvider::with_u64_seed(77);
/// let mut rng = provider.rng();
/// let sk = Z00ZScalar::random(&mut rng).unwrap();
/// let pk = Z00ZRistrettoPoint::from_secret_key(&sk);
/// let bytes = pk.as_bytes();
/// let point = safe_decompress_point(bytes).unwrap();
///
/// // Identity point (rejected)
/// let identity_bytes = [0u8; 32];
/// assert!(safe_decompress_point(&identity_bytes).is_err());
/// ```
pub fn safe_decompress_point(bytes: &[u8]) -> Result<Z00ZRistrettoPoint, CryptoError> {
    validate_ecc_point(bytes).map_err(to_crypto_error)
}

pub fn validate_ecc_point(bytes: &[u8]) -> Result<Z00ZRistrettoPoint, ValidationError> {
    if bytes.len() != 32 {
        return Err(ValidationError::InvalidPointSize(bytes.len()));
    }

    let mut arr = [0u8; 32];
    arr.copy_from_slice(bytes);
    validate_point(&arr)
}

pub fn validate_point(bytes: &[u8; 32]) -> Result<Z00ZRistrettoPoint, ValidationError> {
    if is_identity_compressed(bytes) {
        return Err(ValidationError::IdentityPointRejected);
    }

    let point = canonical_decode(bytes)?;
    validate_point_not_identity(&point)?;
    Ok(point)
}

pub fn canonical_decode(bytes: &[u8; 32]) -> Result<Z00ZRistrettoPoint, ValidationError> {
    Z00ZRistrettoPoint::try_from_bytes(*bytes).map_err(|_| ValidationError::DecompressionFailed)
}

pub fn validate_point_not_identity(point: &Z00ZRistrettoPoint) -> Result<(), ValidationError> {
    if is_identity_compressed(&point.to_bytes()) {
        return Err(ValidationError::IdentityPointRejected);
    }
    Ok(())
}

pub fn is_identity_compressed(bytes: &[u8; 32]) -> bool {
    bool::from(bytes.ct_eq(&[0u8; 32]))
}

pub fn validate_canonical_scalar(bytes: &[u8; 32]) -> Result<Z00ZScalar, ValidationError> {
    Z00ZScalar::try_from_bytes(*bytes).map_err(|_| ValidationError::NonCanonicalScalar)
}

fn validate_scalar_nonzero_core(scalar: &Z00ZScalar) -> Result<(), ValidationError> {
    if scalar.is_zero() {
        return Err(ValidationError::ZeroScalarRejected);
    }
    Ok(())
}

fn to_crypto_error(err: ValidationError) -> CryptoError {
    match err {
        ValidationError::InvalidPointSize(_) => CryptoError::InvalidPointLength,
        ValidationError::DecompressionFailed => CryptoError::InvalidPoint,
        ValidationError::IdentityPointRejected => CryptoError::IdentityPoint,
        ValidationError::ZeroScalarRejected => CryptoError::ZeroScalar,
        ValidationError::NonCanonicalScalar => CryptoError::InvalidScalar,
    }
}

/// Validate scalar is non-zero.
///
/// # SPEC Reference
///
/// §2.1.3 - Rule 3: Scalar non-zero check for ephemeral `r`
///
/// # Arguments
///
/// - `scalar`: Scalar to validate
///
/// # Returns
///
/// - `Ok(())`: Scalar is non-zero
/// - `Err(CryptoError)`: Scalar is zero (forbidden)
///
/// # Security
///
/// Zero scalar → identity point → linkability attack.
/// `r = 0` → `R_pub = 0*G = identity` → all receivers compute same k_dh.
///
/// # Examples
///
/// ```
/// use z00z_crypto::validation::validate_scalar_nonzero;
/// use z00z_crypto::types::Z00ZScalar;
/// use z00z_utils::rng::MockRngProvider;
///
/// // Valid scalar
/// let provider = MockRngProvider::with_u64_seed(78);
/// let mut rng = provider.rng();
/// let scalar = Z00ZScalar::random(&mut rng).unwrap();
/// validate_scalar_nonzero(&scalar).unwrap();
/// ```
pub fn validate_scalar_nonzero(scalar: &Z00ZScalar) -> Result<(), CryptoError> {
    validate_scalar_nonzero_core(scalar).map_err(to_crypto_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::rng::MockRngProvider;

    fn rng_from_seed(seed: u64) -> rand::rngs::StdRng {
        MockRngProvider::with_u64_seed(seed).rng()
    }

    // === safe_decompress_point Tests ===

    #[test]
    fn test_reject_oversized_point() {
        let oversized = [0u8; 33]; // 33 bytes (too long)
        assert!(matches!(
            safe_decompress_point(&oversized),
            Err(CryptoError::InvalidPointLength)
        ));
        assert!(matches!(
            validate_ecc_point(&oversized),
            Err(ValidationError::InvalidPointSize(33))
        ));
    }

    #[test]
    fn test_reject_undersized_point() {
        let undersized = [0u8; 31]; // 31 bytes (too short)
        assert!(matches!(
            safe_decompress_point(&undersized),
            Err(CryptoError::InvalidPointLength)
        ));
        assert!(matches!(
            validate_ecc_point(&undersized),
            Err(ValidationError::InvalidPointSize(31))
        ));
    }

    #[test]
    fn test_reject_bad_point() {
        // Non-canonical point (all 0xFF)
        let invalid = [0xFFu8; 32];
        assert!(matches!(
            safe_decompress_point(&invalid),
            Err(CryptoError::InvalidPoint)
        ));
        assert!(matches!(
            validate_point(&invalid),
            Err(ValidationError::DecompressionFailed)
        ));
    }

    #[test]
    fn test_reject_identity_point() {
        let identity = [0u8; 32]; // Identity point
        assert!(matches!(
            safe_decompress_point(&identity),
            Err(CryptoError::IdentityPoint)
        ));
        assert!(matches!(
            validate_point(&identity),
            Err(ValidationError::IdentityPointRejected)
        ));
    }

    #[test]
    fn test_accept_valid_point() {
        // Generate valid point from scalar
        let mut rng = rng_from_seed(201);
        let scalar = Z00ZScalar::random(&mut rng).unwrap();
        let point = Z00ZRistrettoPoint::from_secret_key(&scalar);
        let bytes = point.as_bytes();

        let result = safe_decompress_point(bytes);
        assert!(result.is_ok());
        assert!(validate_ecc_point(bytes).is_ok());
    }

    #[test]
    fn test_identity_bytes_check() {
        let identity = [0u8; 32];
        let not_identity = [1u8; 32];
        assert!(is_identity_compressed(&identity));
        assert!(!is_identity_compressed(&not_identity));
    }

    #[test]
    fn test_valid_random_point() {
        // Generate random point
        let mut rng = rng_from_seed(202);
        let scalar = Z00ZScalar::random(&mut rng).unwrap();
        let point = Z00ZRistrettoPoint::from_secret_key(&scalar);
        let bytes = point.as_bytes();

        let result = safe_decompress_point(bytes);
        assert!(result.is_ok());
    }

    // === validate_scalar_nonzero Tests ===

    #[test]
    fn test_reject_zero_scalar() {
        let zero = Z00ZScalar::zero(); // Zero scalar
        assert!(matches!(
            validate_scalar_nonzero(&zero),
            Err(CryptoError::ZeroScalar)
        ));
        assert!(matches!(
            validate_scalar_nonzero_core(&zero),
            Err(ValidationError::ZeroScalarRejected)
        ));
    }

    #[test]
    fn test_accept_nonzero_scalar() {
        let mut rng = rng_from_seed(203);
        let scalar = Z00ZScalar::random(&mut rng).unwrap();
        let result = validate_scalar_nonzero(&scalar);
        assert!(result.is_ok());
        assert!(validate_scalar_nonzero_core(&scalar).is_ok());
    }

    #[test]
    fn test_multiple_random_scalars() {
        // Test multiple random scalars (all should be non-zero)
        let mut rng = rng_from_seed(204);
        for _ in 0..100 {
            let scalar = Z00ZScalar::random(&mut rng).unwrap();
            validate_scalar_nonzero(&scalar).expect("random scalar must be non-zero");
        }
    }

    #[test]
    fn test_reject_noncanon_scalar() {
        let noncanon = [0xFFu8; 32];
        assert!(matches!(
            validate_canonical_scalar(&noncanon),
            Err(ValidationError::NonCanonicalScalar)
        ));
    }

    #[test]
    fn test_accept_canon_scalar() {
        let mut rng = rng_from_seed(205);
        let scalar = Z00ZScalar::random(&mut rng).unwrap();
        let bytes = scalar.to_bytes();
        assert!(validate_canonical_scalar(&bytes).is_ok());
    }
}
