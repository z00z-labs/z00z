//! Blinding factor generation helpers for confidential assets.

use z00z_crypto::{CryptoError, Hidden, Z00ZScalar};
use z00z_utils::rng::{RngCoreExt, SystemRngProvider};

pub fn generate_blinding(rng: &mut impl rand::RngCore) -> Result<Hidden<Z00ZScalar>, CryptoError> {
    let mut bytes = [0u8; 32];

    for _ in 0..64 {
        rng.fill_bytes_ext(&mut bytes);
        if let Ok(scalar) = Z00ZScalar::try_from_bytes(bytes) {
            return Ok(Hidden::hide(scalar));
        }
    }

    Ok(Hidden::hide(Z00ZScalar::random_secure(&SystemRngProvider)?))
}

/// Stateless generator for secure transaction blinding factors.
pub struct BlindingFactorGenerator;

impl BlindingFactorGenerator {
    /// Generate one blinding factor wrapped in [`Hidden`].
    pub fn generate(&self) -> Result<Hidden<Z00ZScalar>, CryptoError> {
        Ok(Hidden::hide(Z00ZScalar::random_secure(&SystemRngProvider)?))
    }

    /// Generate a batch of independent blinding factors.
    pub fn generate_batch(&self, count: usize) -> Result<Vec<Hidden<Z00ZScalar>>, CryptoError> {
        (0..count).map(|_| self.generate()).collect()
    }
}

impl Default for BlindingFactorGenerator {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_blinding, BlindingFactorGenerator};
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use std::collections::BTreeSet;
    use z00z_crypto::Hidden;
    use zeroize::Zeroize;

    #[test]
    fn test_blinding_type_is_zscalar() {
        let generator = BlindingFactorGenerator;
        let blinding = generator.generate().unwrap();
        let _: [u8; 32] = blinding.reveal().to_bytes();
    }

    #[test]
    fn test_blinding_uniqueness() {
        let generator = BlindingFactorGenerator;
        let first = generator.generate().unwrap();
        let second = generator.generate().unwrap();

        assert!(!first.reveal().ct_eq(second.reveal()));
    }

    #[test]
    fn test_hidden_wrapping() {
        let generator = BlindingFactorGenerator;
        let blinding = generator.generate().unwrap();
        let mut wrapped = Hidden::hide(blinding.reveal().dangerous_clone());
        assert!(!wrapped.reveal().is_zero());
        wrapped.zeroize();
        assert!(wrapped.reveal().is_zero());
    }

    #[test]
    fn test_batch_all_unique() {
        let generator = BlindingFactorGenerator;
        let batch = generator.generate_batch(100).unwrap();

        let set: BTreeSet<[u8; 32]> = batch.iter().map(|item| item.reveal().to_bytes()).collect();
        assert_eq!(set.len(), batch.len());
    }

    #[test]
    fn test_generate_blind_fn() {
        let mut rng = ChaCha20Rng::from_seed([7u8; 32]);
        let first = generate_blinding(&mut rng).unwrap();
        let second = generate_blinding(&mut rng).unwrap();

        assert_ne!(first.reveal().to_bytes(), second.reveal().to_bytes());
    }
}
