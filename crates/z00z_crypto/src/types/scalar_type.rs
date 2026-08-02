use core::convert::{TryFrom, TryInto};

use tari_crypto::keys::SecretKey;
use tari_crypto::ristretto::RistrettoSecretKey;
use tari_crypto::tari_utilities::ByteArray;
use zeroize::Zeroize;

use crate::{secret::SecretBytes, CryptoError};

#[derive(zeroize::Zeroize)]
#[zeroize(drop)]
#[repr(transparent)]
pub struct Z00ZScalar(pub(crate) RistrettoSecretKey);

impl Z00ZScalar {
    pub const ZERO_BYTES: [u8; 32] = [0u8; 32];
    pub const ONE_BYTES: [u8; 32] = [
        1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
    ];

    pub fn zero() -> Self {
        Self(RistrettoSecretKey::default())
    }

    pub fn one() -> Self {
        Self(RistrettoSecretKey::from(1u64))
    }

    pub fn from_ristretto_secret_key(key: RistrettoSecretKey) -> Self {
        Self(key)
    }

    pub fn try_from_bytes(bytes: [u8; 32]) -> Result<Self, CryptoError> {
        let key = RistrettoSecretKey::from_canonical_bytes(&bytes)
            .map_err(|_| CryptoError::InvalidParameters { param: "scalar" })?;
        Ok(Self(key))
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| CryptoError::InvalidParameters { param: "scalar" })?;
        Self::try_from_bytes(bytes)
    }

    /// Parse one non-zero canonical scalar directly from a zeroizing owner.
    ///
    /// The caller-owned buffer is wiped before every return, including length,
    /// canonical-encoding, and zero-scalar failures.
    pub fn try_from_nonzero_secret_bytes(secret: &mut SecretBytes) -> Result<Self, CryptoError> {
        let parsed = if secret.len() != 32 {
            Err(CryptoError::InvalidParameters { param: "scalar" })
        } else {
            RistrettoSecretKey::from_canonical_bytes(secret.as_slice())
                .map(Self)
                .map_err(|_| CryptoError::InvalidParameters { param: "scalar" })
        };
        secret.wipe();

        let scalar = parsed?;
        if scalar.is_zero() {
            return Err(CryptoError::InvalidScalar);
        }
        Ok(scalar)
    }

    pub fn from_uniform_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        let key = RistrettoSecretKey::from_uniform_bytes(bytes).map_err(|_| {
            CryptoError::InvalidParameters {
                param: "uniform_bytes",
            }
        })?;
        Ok(Self(key))
    }

    pub fn from_hash(hash: &[u8; 64]) -> Result<Self, CryptoError> {
        Self::try_from_hash(hash)
    }

    pub fn try_from_hash(hash: &[u8; 64]) -> Result<Self, CryptoError> {
        let scalar = Self::from_uniform_bytes(hash).map_err(|_| CryptoError::InvalidScalar)?;
        if scalar.is_zero() {
            return Err(CryptoError::InvalidScalar);
        }
        Ok(scalar)
    }

    pub fn is_zero(&self) -> bool {
        use subtle::ConstantTimeEq;
        self.0.as_bytes().ct_eq(&Self::ZERO_BYTES).into()
    }

    pub(crate) fn inner(&self) -> &RistrettoSecretKey {
        &self.0
    }

    pub fn reveal(&self) -> &RistrettoSecretKey {
        &self.0
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out.copy_from_slice(self.0.as_bytes());
        out
    }

    #[must_use]
    pub fn dangerous_clone(&self) -> Self {
        Self(self.0.clone())
    }

    pub fn ct_eq(&self, other: &Self) -> bool {
        use subtle::ConstantTimeEq;
        self.0.as_bytes().ct_eq(other.0.as_bytes()).into()
    }

    #[doc(hidden)]
    pub(crate) fn random_from_rng<R: rand::CryptoRng + rand::RngCore>(
        rng: &mut R,
    ) -> Result<Self, CryptoError> {
        const MAX_TRIES: usize = 16;
        for _ in 0..MAX_TRIES {
            let mut uniform = [0u8; 64];
            rng.try_fill_bytes(&mut uniform)
                .map_err(|_| CryptoError::RngFailure)?;
            let scalar = Self::from_uniform_bytes(&uniform)?;
            uniform.zeroize();
            if !scalar.is_zero() {
                return Ok(scalar);
            }
        }

        Err(CryptoError::RngFailure)
    }

    pub fn random<R: rand::CryptoRng + rand::RngCore>(rng: &mut R) -> Result<Self, CryptoError> {
        Self::random_from_rng(rng)
    }

    pub fn random_secure(
        provider: &impl z00z_utils::rng::SecureRngProvider,
    ) -> Result<Self, CryptoError> {
        let mut rng = provider.rng();
        Self::random_from_rng(&mut rng)
    }

    #[cfg(any(test, feature = "test-utils", feature = "test-params-fast"))]
    pub fn random_deterministic<P>(provider: &P) -> Result<Self, CryptoError>
    where
        P: z00z_utils::rng::DeterministicRngSource,
        P::Rng: rand::CryptoRng,
    {
        let mut rng = provider.rng();
        Self::random_from_rng(&mut rng)
    }
}

impl core::ops::Add for &Z00ZScalar {
    type Output = Z00ZScalar;

    fn add(self, other: Self) -> Self::Output {
        Z00ZScalar(&self.0 + &other.0)
    }
}

impl core::ops::Sub for &Z00ZScalar {
    type Output = Z00ZScalar;

    fn sub(self, other: Self) -> Self::Output {
        Z00ZScalar(&self.0 - &other.0)
    }
}

impl core::ops::Mul for &Z00ZScalar {
    type Output = Z00ZScalar;

    fn mul(self, rhs: Self) -> Self::Output {
        Z00ZScalar(&self.0 * &rhs.0)
    }
}

impl core::ops::Neg for &Z00ZScalar {
    type Output = Z00ZScalar;

    fn neg(self) -> Self::Output {
        let zero = Z00ZScalar::zero();
        &zero - self
    }
}

impl TryFrom<RistrettoSecretKey> for Z00ZScalar {
    type Error = CryptoError;

    fn try_from(value: RistrettoSecretKey) -> Result<Self, Self::Error> {
        Self::from_canonical_bytes(value.as_bytes())
    }
}

impl TryFrom<Z00ZScalar> for RistrettoSecretKey {
    type Error = CryptoError;

    fn try_from(value: Z00ZScalar) -> Result<Self, Self::Error> {
        RistrettoSecretKey::from_canonical_bytes(value.as_bytes())
            .map_err(|_| CryptoError::InvalidParameters { param: "scalar" })
    }
}
