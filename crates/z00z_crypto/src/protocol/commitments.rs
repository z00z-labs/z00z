//! Pedersen commitment helpers for Phase 5.

use tari_crypto::ristretto::pedersen::PedersenCommitment;
use tari_crypto::tari_utilities::ByteArray;
use thiserror::Error;

use crate::{
    create_commitment, kdf::hash_to_scalar_domain, CryptoError, Z00ZCommitment, Z00ZScalar,
};

#[cfg(not(target_arch = "wasm32"))]
use z00z_utils::rng::SystemRngProvider;

#[derive(Debug, Error)]
pub enum CommitmentErr {
    #[error("commitment verification failed")]
    VerifyFail,
    #[error("zero blinding factor rejected")]
    ZeroBlind,
    #[error("invalid commitment point")]
    InvalidPoint,
    #[error("crypto operation failed")]
    CryptoFail,
}

impl From<CryptoError> for CommitmentErr {
    fn from(_: CryptoError) -> Self {
        CommitmentErr::CryptoFail
    }
}

/// Generates cryptographically secure random blinding factor.
#[cfg(not(target_arch = "wasm32"))]
pub fn generate_blinding_factor() -> Result<Z00ZScalar, CommitmentErr> {
    const MAX_TRIES: usize = 32;
    let provider = SystemRngProvider;
    for _ in 0..MAX_TRIES {
        let scalar = Z00ZScalar::random_secure(&provider).map_err(|_| CommitmentErr::CryptoFail)?;
        if !scalar.is_zero() {
            return Ok(scalar);
        }
    }
    Err(CommitmentErr::ZeroBlind)
}

/// Generates blinding factor on wasm target.
#[cfg(target_arch = "wasm32")]
pub fn generate_blinding_factor() -> Result<Z00ZScalar, CommitmentErr> {
    const MAX_TRIES: usize = 32;
    for _ in 0..MAX_TRIES {
        let mut bytes = [0u8; 64];
        getrandom::getrandom(&mut bytes).map_err(|_| CommitmentErr::CryptoFail)?;
        let scalar =
            Z00ZScalar::from_uniform_bytes(&bytes).map_err(|_| CommitmentErr::CryptoFail)?;
        if !scalar.is_zero() {
            return Ok(scalar);
        }
    }
    Err(CommitmentErr::ZeroBlind)
}

/// Deterministic blinding factor generator.
pub struct BlindingFactorGenerator {
    seed: Option<[u8; 32]>,
}

impl BlindingFactorGenerator {
    pub fn new_random() -> Self {
        Self { seed: None }
    }

    pub fn new_deterministic(seed: [u8; 32]) -> Self {
        Self { seed: Some(seed) }
    }

    pub fn generate(&self, asset_idx: u64) -> Result<Z00ZScalar, CommitmentErr> {
        match self.seed {
            Some(seed) => {
                let idx_bytes = asset_idx.to_le_bytes();
                let scalar = hash_to_scalar_domain(b"blind", &[&seed, &idx_bytes]);
                if scalar.is_zero() {
                    return Err(CommitmentErr::ZeroBlind);
                }
                Ok(scalar)
            }
            None => generate_blinding_factor(),
        }
    }
}

/// Pedersen commitment wrapper.
#[derive(Debug, Clone)]
pub struct Commitment(pub Z00ZCommitment);

impl Commitment {
    pub fn new_with_blinding(value: u64, blinding: &Z00ZScalar) -> Result<Self, CommitmentErr> {
        if blinding.is_zero() {
            return Err(CommitmentErr::ZeroBlind);
        }
        let commitment =
            create_commitment(value, blinding).map_err(|_| CommitmentErr::CryptoFail)?;
        Ok(Self(commitment))
    }

    pub fn as_commitment(&self) -> &Z00ZCommitment {
        &self.0
    }

    pub fn as_point(&self) -> &Z00ZCommitment {
        &self.0
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out.copy_from_slice(self.0.reveal().as_bytes());
        out
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, CommitmentErr> {
        let commitment = PedersenCommitment::from_canonical_bytes(bytes)
            .map_err(|_| CommitmentErr::InvalidPoint)?;
        Ok(Self(Z00ZCommitment::from_commitment(commitment)))
    }
}

/// Commitment opening.
pub struct CommitmentOpening {
    pub value: u64,
    pub blinding: Z00ZScalar,
}

impl CommitmentOpening {
    pub fn new(value: u64, blinding: Z00ZScalar) -> Self {
        Self { value, blinding }
    }

    pub fn to_commitment(&self) -> Result<Commitment, CommitmentErr> {
        Commitment::new_with_blinding(self.value, &self.blinding)
    }

    pub fn verify(&self, commitment: &Commitment) -> bool {
        use subtle::ConstantTimeEq;
        match self.to_commitment() {
            Ok(expected) => expected.to_bytes().ct_eq(&commitment.to_bytes()).into(),
            Err(_) => false,
        }
    }
}

pub fn verify_opening(com: &Commitment, value: u64, blinding: Z00ZScalar) -> bool {
    let opening = CommitmentOpening::new(value, blinding);
    opening.verify(com)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_blind_random_unique() {
        let mut seen = HashSet::new();
        for _ in 0..64 {
            let blind = generate_blinding_factor().expect("random blind");
            assert!(seen.insert(blind.to_bytes()));
        }
    }

    #[test]
    fn test_blind_not_zero() {
        for _ in 0..64 {
            let blind = generate_blinding_factor().expect("random blind");
            assert!(!blind.is_zero());
        }
    }

    #[test]
    fn test_blind_det_same() {
        let seed = [0x42u8; 32];
        let gen = BlindingFactorGenerator::new_deterministic(seed);
        let a = gen.generate(0).expect("blind a");
        let b = gen.generate(0).expect("blind b");
        assert_eq!(a.to_bytes(), b.to_bytes());
    }

    #[test]
    fn test_blind_det_diff_idx() {
        let seed = [0x42u8; 32];
        let gen = BlindingFactorGenerator::new_deterministic(seed);
        let a = gen.generate(0).expect("blind a");
        let b = gen.generate(1).expect("blind b");
        assert_ne!(a.to_bytes(), b.to_bytes());
    }

    #[test]
    fn test_opening_ok() {
        let value = 1000u64;
        let blind = Z00ZScalar::one();
        let com = Commitment::new_with_blinding(value, &blind).expect("commitment");
        assert!(verify_opening(&com, value, blind));
    }

    #[test]
    fn test_opening_bad_value() {
        let blind = Z00ZScalar::one();
        let com = Commitment::new_with_blinding(1000, &blind).expect("commitment");
        assert!(!verify_opening(&com, 999, blind));
    }

    #[test]
    fn test_opening_bad_blind() {
        let value = 1000u64;
        let com = Commitment::new_with_blinding(value, &Z00ZScalar::one()).expect("commitment");
        let wrong = Z00ZScalar::from_hash(&[1u8; 64]).unwrap();
        assert!(!verify_opening(&com, value, wrong));
    }

    #[test]
    fn test_commitment_homomorphic() {
        let first_value = 100u64;
        let second_value = 200u64;
        let r1 = Z00ZScalar::from_hash(&[11u8; 64]).unwrap();
        let r2 = Z00ZScalar::from_hash(&[22u8; 64]).unwrap();

        let c1 = Commitment::new_with_blinding(first_value, &r1).expect("c1");
        let c2 = Commitment::new_with_blinding(second_value, &r2).expect("c2");
        let c_sum =
            Commitment::new_with_blinding(first_value + second_value, &(&r1 + &r2)).expect("sum");

        let c_add = Commitment(&c1.0 + &c2.0);
        assert_eq!(c_add.to_bytes(), c_sum.to_bytes());
    }
}
