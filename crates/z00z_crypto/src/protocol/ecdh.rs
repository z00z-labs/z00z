//! ECDH Stealth Address Primitives.
//!
//! # SPEC Reference
//!
//! - §2.5 - ECDH Key Exchange for Stealth Addresses
//! - §2.5.1.3 - Sender/Receiver Symmetry
//!
//! # Security Policy
//!
//! **Critical validations:**
//! - MUST reject identity point (O) in all operations
//! - MUST validate untrusted `R_pub` from leaf data
//! - MUST use domain-separated hash for `derive_dh_key()`
//!
//! # Ownership Contract
//!
//! This module is the canonical owner for point-based stealth ECDH and the
//! base `derive_dh_key(&Z00ZRistrettoPoint)` formula chain. Wallet runtime
//! byte-oriented helpers stay outside this module until convergence removes the
//! remaining compatibility split explicitly.
//!
//! # Usage Example
//!
//! ```rust
//! use z00z_crypto::protocol::ecdh::{
//!     generate_ephemeral_keypair, compute_stealth_dh_sender,
//!     recover_stealth_dh_receiver, derive_dh_key
//! };
//! use z00z_crypto::types::{Z00ZRistrettoPoint, Z00ZScalar};
//! use z00z_utils::rng::MockRngProvider;
//!
//! let provider = MockRngProvider::with_u64_seed(42);
//! let mut rng = provider.rng();
//!
//! // Receiver generates view keypair
//! let view_sk = Z00ZScalar::random(&mut rng).unwrap();
//! let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);
//!
//! // Sender side
//! let r = Z00ZScalar::random(&mut rng).unwrap();
//! let r_pub = generate_ephemeral_keypair(&r)?;
//! let dh_sender = compute_stealth_dh_sender(&r, &view_pk)?;
//! let k_dh_sender = derive_dh_key(&dh_sender);
//!
//! // Receiver side
//! let dh_receiver = recover_stealth_dh_receiver(&view_sk, &r_pub)?;
//! let k_dh_receiver = derive_dh_key(&dh_receiver);
//!
//! // Verify symmetry
//! assert_eq!(k_dh_sender, k_dh_receiver);
//! # Ok::<_, z00z_crypto::CryptoError>(())
//! ```

use crate::{
    domains::DhKeyDomain,
    error::CryptoError,
    hash_zk::hash_zk,
    types::{Z00ZRistrettoPoint, Z00ZScalar},
};

/// Generate ephemeral keypair.
///
/// # SPEC Reference
///
/// §2.5.1.1 - Sender generates `R_pub = r * G`
///
/// # Arguments
///
/// - `r`: Ephemeral secret scalar
///
/// # Returns
///
/// Ephemeral public key `R_pub`
///
/// # Errors
///
/// - `CryptoError::ZeroScalar`: `r = 0` → `R_pub = identity` → security violation.
///
/// # Examples
///
/// ```rust
/// use z00z_crypto::protocol::ecdh::generate_ephemeral_keypair;
/// use z00z_crypto::types::Z00ZScalar;
/// use z00z_utils::rng::MockRngProvider;
///
/// let provider = MockRngProvider::with_u64_seed(10);
/// let mut rng = provider.rng();
/// let r = Z00ZScalar::random(&mut rng).unwrap();
/// let R_pub = generate_ephemeral_keypair(&r)?;
/// # Ok::<_, z00z_crypto::CryptoError>(())
/// ```
pub fn generate_ephemeral_keypair(r: &Z00ZScalar) -> Result<Z00ZRistrettoPoint, CryptoError> {
    // SPEC §2.1.3 Rule 3: r = 0 → R_pub = identity point
    if r.is_zero() {
        return Err(CryptoError::ZeroScalar);
    }
    Ok(Z00ZRistrettoPoint::from_secret_key(r))
}

/// Compute stealth DH (sender).
///
/// # SPEC Reference
///
/// §2.5.1.2 - Sender computes `dh = r * view_pk`
///
/// # Arguments
///
/// - `r`: Ephemeral secret scalar
/// - `view_pk`: Receiver view public key
///
/// # Returns
///
/// - `Ok(Z00ZRistrettoPoint)`: Shared secret point
/// - `Err(CryptoError)`: If `view_pk` is identity
///
/// # Errors
///
/// Returns `CryptoError::ZeroScalar` if `r` is zero.
/// Returns `CryptoError::IdentityPoint` if `view_pk` is identity point.
///
/// # Examples
///
/// ```rust
/// use z00z_crypto::protocol::ecdh::compute_stealth_dh_sender;
/// use z00z_crypto::types::{Z00ZRistrettoPoint, Z00ZScalar};
/// use z00z_utils::rng::MockRngProvider;
///
/// let provider = MockRngProvider::with_u64_seed(11);
/// let mut rng = provider.rng();
/// let view_sk = Z00ZScalar::random(&mut rng).unwrap();
/// let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);
/// let r = Z00ZScalar::random(&mut rng).unwrap();
/// let dh = compute_stealth_dh_sender(&r, &view_pk)?;
/// # Ok::<_, z00z_crypto::CryptoError>(())
/// ```
pub fn compute_stealth_dh_sender(
    r: &Z00ZScalar,
    view_pk: &Z00ZRistrettoPoint,
) -> Result<Z00ZRistrettoPoint, CryptoError> {
    // SPEC §2.1.3 Rule 3: r = 0 → R_pub = identity → same k_dh for all receivers
    if r.is_zero() {
        return Err(CryptoError::ZeroScalar);
    }
    validate_stealth_point(view_pk)?;

    // dh = r * view_pk
    Ok(view_pk * r)
}

/// Recover stealth DH (receiver).
///
/// # SPEC Reference
///
/// §2.5.1.3 - Receiver computes `dh' = view_sk * R_pub`
///
/// # Arguments
///
/// - `view_sk`: Receiver view secret key
/// - `R_pub`: Ephemeral public key from leaf data (SPEC notation)
///
/// # Returns
///
/// - `Ok(Z00ZRistrettoPoint)`: Shared secret point
/// - `Err(CryptoError)`: If `R_pub` is identity
///
/// # Errors
///
/// Returns `CryptoError::InvalidPublicKey` if ephemeral public key
/// from untrusted leaf data is identity point.
///
/// # Security
///
/// **CRITICAL:** This function processes untrusted data from blockchain.
/// Always validates `R_pub` before ECDH operation.
///
/// # Examples
///
/// ```rust
/// use z00z_crypto::protocol::ecdh::recover_stealth_dh_receiver;
/// use z00z_crypto::types::{Z00ZRistrettoPoint, Z00ZScalar};
/// use z00z_utils::rng::MockRngProvider;
///
/// let provider = MockRngProvider::with_u64_seed(12);
/// let mut rng = provider.rng();
/// let r = Z00ZScalar::random(&mut rng).unwrap();
/// let r_pub = Z00ZRistrettoPoint::from_secret_key(&r);
/// let view_sk = Z00ZScalar::random(&mut rng).unwrap();
/// let dh = recover_stealth_dh_receiver(&view_sk, &r_pub)?;
/// # Ok::<_, z00z_crypto::CryptoError>(())
/// ```
#[allow(non_snake_case)]
pub fn recover_stealth_dh_receiver(
    view_sk: &Z00ZScalar,
    r_pub: &Z00ZRistrettoPoint,
) -> Result<Z00ZRistrettoPoint, CryptoError> {
    // SPEC §2.1.3 Rule 3: view_sk = 0 → dh = identity → k_dh identical for all receivers
    if view_sk.is_zero() {
        return Err(CryptoError::ZeroScalar);
    }
    validate_stealth_point(r_pub)?;

    // dh' = view_sk * R_pub
    Ok(r_pub * view_sk)
}

/// Derive k_dh from ECDH.
///
/// # SPEC Reference
///
/// §2.5.2 - KDF from shared secret: `k_dh = H_zk("dh_key", dh_bytes)`
///
/// # Arguments
///
/// - `dh`: Shared secret point from ECDH
///
/// # Returns
///
/// 32-byte derived key for stealth address generation
///
/// # Security
///
/// Uses consensus hash `H_zk` with `DhKeyDomain` per SPEC §2.2.2.1.
/// Output is deterministic for same `dh` input.
///
/// # Examples
///
/// ```rust
/// use z00z_crypto::protocol::ecdh::derive_dh_key;
/// use z00z_utils::rng::MockRngProvider;
/// use z00z_crypto::types::{Z00ZRistrettoPoint, Z00ZScalar};
///
/// let provider = MockRngProvider::with_u64_seed(13);
/// let mut rng = provider.rng();
/// let sk = Z00ZScalar::random(&mut rng).unwrap();
/// let dh = Z00ZRistrettoPoint::from_secret_key(&sk);
/// let k_dh = derive_dh_key(&dh);
/// assert_eq!(k_dh.len(), 32);
/// ```
pub fn derive_dh_key(dh: &Z00ZRistrettoPoint) -> [u8; 32] {
    // Compress point to canonical 32-byte encoding
    let dh_bytes = dh.as_bytes();

    // Domain-separated hash: H_zk<DhKeyDomain>("", dh_bytes)
    // Domain string: "z00z.consensus.dh_key.v1" per SPEC §2.2.2.1
    hash_zk::<DhKeyDomain>("", &[dh_bytes])
}

/// Validate stealth point.
///
/// # SPEC Reference
///
/// §2.5.1.2 - Identity point validation
///
/// # Arguments
///
/// - `point`: Ristretto point to validate
///
/// # Returns
///
/// - `Ok(())`: Point is valid (non-identity)
/// - `Err(CryptoError)`: Point is identity
///
/// # Security
///
/// **CRITICAL:** Rejecting identity point prevents:
/// - Shared secret becoming `O` (zero point)
/// - Derived keys becoming predictable
/// - Cross-protocol attacks
///
/// # Examples
///
/// ```rust
/// use z00z_crypto::protocol::ecdh::validate_stealth_point;
/// use z00z_utils::rng::MockRngProvider;
/// use z00z_crypto::types::{Z00ZRistrettoPoint, Z00ZScalar};
///
/// let provider = MockRngProvider::with_u64_seed(14);
/// let mut rng = provider.rng();
/// let sk = Z00ZScalar::random(&mut rng).unwrap();
/// let valid_point = Z00ZRistrettoPoint::from_secret_key(&sk);
/// assert!(validate_stealth_point(&valid_point).is_ok());
/// ```
pub fn validate_stealth_point(point: &Z00ZRistrettoPoint) -> Result<(), CryptoError> {
    if point.is_identity() {
        return Err(CryptoError::IdentityPoint);
    }

    Ok(())
}
