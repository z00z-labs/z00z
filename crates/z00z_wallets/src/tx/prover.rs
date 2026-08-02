//! Range proof generation/verification.

use crate::key::{sign_identity, sign_identity_with_rng, verify_identity};
use thiserror::Error;
use z00z_crypto::vendor::tari::ExtendedPedersenCommitmentFactory;
#[cfg(test)]
use z00z_crypto::Z00ZCommitment;
use z00z_crypto::{
    create_range_proof, BulletproofsPlusService, Commitment as ParsedCommitment, Hidden,
    RangeProof, RangeProofService, Z00ZRistrettoPoint, Z00ZScalar, Z00ZSchnorrSignature,
};
use z00z_crypto::{AGGREGATION_FACTOR, MIN_VALUE_PROMISE, RANGE_PROOF_BITS};

/// Domain-separation label for public spend authorization signatures.
pub const SPEND_AUTH_CTX: &[u8] = b"Z00Z/SPEND_AUTH_V1";

/// Prover errors.
#[derive(Debug, Error)]
pub enum ProverError {
    /// Proof generation failed.
    #[error("proof generation failed: {0}")]
    ProofFailed(String),

    /// Proof verification failed.
    #[error("proof verification failed: {0}")]
    VerificationFailed(String),

    /// Invalid range.
    #[error("invalid range: {0}")]
    InvalidRange(String),

    /// Cryptographic error.
    #[error("cryptographic error: {0}")]
    Crypto(String),
}

/// Prover result type.
pub type ProverResult<T> = std::result::Result<T, ProverError>;

/// Sign a canonical public spend-authorization statement with the receiver identity key.
pub fn sign_spend_authorization_with_rng<R>(
    identity_sk: &Z00ZScalar,
    statement: &[u8],
    rng: &mut R,
) -> ProverResult<Z00ZSchnorrSignature>
where
    R: rand::CryptoRng + rand::RngCore,
{
    sign_identity_with_rng(identity_sk, statement, SPEND_AUTH_CTX, rng)
        .map_err(|err| ProverError::Crypto(format!("spend authorization sign failed: {err}")))
}

/// Sign a canonical public spend-authorization statement with the receiver identity key.
pub fn sign_spend_authorization(
    identity_sk: &Z00ZScalar,
    statement: &[u8],
) -> ProverResult<Z00ZSchnorrSignature> {
    sign_identity(identity_sk, statement, SPEND_AUTH_CTX)
        .map_err(|err| ProverError::Crypto(format!("spend authorization sign failed: {err}")))
}

/// Verify a canonical public spend-authorization statement against the receiver identity key.
pub fn verify_spend_authorization(
    identity_pk: &Z00ZRistrettoPoint,
    statement: &[u8],
    signature: &Z00ZSchnorrSignature,
) -> ProverResult<bool> {
    match verify_identity(identity_pk, statement, SPEND_AUTH_CTX, signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Range prover trait.
///
/// Implementation contract:
///
/// - Must use z00z_crypto::BulletproofsPlusService for range proofs
/// - Must use z00z_crypto::PedersenCommitmentFactory-compatible factories for commitments
/// - Must use z00z_utils::rng::SecureRngProvider for cryptographic randomness
/// - Must support batch proof generation for efficiency
pub trait Prover {
    /// Create a range proof for a value.
    ///
    /// Contract:
    /// - Use Bulletproofs+ service from z00z_crypto
    /// - Use RNG for proof generation (via z00z_utils::rng::SecureRngProvider)
    fn create_proof(&self, amount: u64, blinding: &Hidden<Z00ZScalar>) -> ProverResult<RangeProof>;

    /// Create proofs for multiple values.
    ///
    /// Contract:
    /// - Use batch API for efficiency
    fn create_batch_proofs(
        &self,
        outputs: &[(u64, Hidden<Z00ZScalar>)],
    ) -> ProverResult<Vec<RangeProof>>;

    /// Verify a range proof against an opaque commitment bytes.
    ///
    /// Contract:
    /// - Parse commitment from bytes
    /// - Verify proof with z00z_crypto
    fn verify_proof(&self, proof: &RangeProof, commitment: &[u8]) -> ProverResult<bool>;

    /// Verify multiple proofs.
    ///
    /// Contract:
    /// - Verify all proofs and return false if any fail
    fn verify_batch_proofs(
        &self,
        proofs: &[RangeProof],
        commitments: &[&[u8]],
    ) -> ProverResult<bool>;
}

/// Default Prover implementation.
pub struct ProverImpl {
    service: BulletproofsPlusService,
}

impl ProverImpl {
    /// Create a new prover.
    pub fn new() -> ProverResult<Self> {
        let service = BulletproofsPlusService::init(
            RANGE_PROOF_BITS,
            AGGREGATION_FACTOR,
            ExtendedPedersenCommitmentFactory::default(),
        )
        .map_err(|e| ProverError::Crypto(format!("BulletproofsPlusService init failed: {e}")))?;

        Ok(Self { service })
    }
}

impl Prover for ProverImpl {
    fn create_proof(&self, amount: u64, blinding: &Hidden<Z00ZScalar>) -> ProverResult<RangeProof> {
        create_range_proof(
            amount,
            blinding.reveal(),
            RANGE_PROOF_BITS,
            MIN_VALUE_PROMISE,
        )
        .map_err(|e| ProverError::ProofFailed(format!("construct_proof failed: {e:?}")))
    }

    fn create_batch_proofs(
        &self,
        outputs: &[(u64, Hidden<Z00ZScalar>)],
    ) -> ProverResult<Vec<RangeProof>> {
        let mut proofs = Vec::with_capacity(outputs.len());
        for (amount, blinding) in outputs {
            proofs.push(self.create_proof(*amount, blinding)?);
        }
        Ok(proofs)
    }

    fn verify_proof(&self, proof: &RangeProof, commitment: &[u8]) -> ProverResult<bool> {
        let commitment = ParsedCommitment::from_bytes(
            commitment
                .try_into()
                .map_err(|_| ProverError::Crypto("invalid commitment bytes length".to_string()))?,
        )
        .map_err(|e| ProverError::Crypto(format!("invalid commitment bytes: {e}")))?;

        Ok(self
            .service
            .verify(proof, commitment.as_commitment().reveal()))
    }

    fn verify_batch_proofs(
        &self,
        proofs: &[RangeProof],
        commitments: &[&[u8]],
    ) -> ProverResult<bool> {
        if proofs.len() != commitments.len() {
            return Ok(false);
        }

        for (proof, commitment_bytes) in proofs.iter().zip(commitments.iter()) {
            if !self.verify_proof(proof, commitment_bytes)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_crypto::{create_commitment, Z00ZScalar};
    use z00z_utils::rng::MockRngProvider;

    #[test]
    fn test_create_proof_verify_roundtrip() {
        let prover = ProverImpl::new().unwrap();
        let mut rng = MockRngProvider::with_u64_seed(7).rng();
        let blinding = Hidden::hide(Z00ZScalar::random(&mut rng).unwrap());

        let amount = 123u64;
        let proof = prover.create_proof(amount, &blinding).unwrap();
        let commitment = create_commitment(amount, blinding.reveal()).expect("valid blinding");

        assert!(prover.verify_proof(&proof, commitment.as_bytes()).unwrap());
    }

    #[test]
    fn test_create_batch_proofs_verify() {
        let prover = ProverImpl::new().unwrap();

        let mut rng = MockRngProvider::with_u64_seed(7).rng();
        let b1 = Hidden::hide(Z00ZScalar::random(&mut rng).unwrap());
        let b2 = Hidden::hide(Z00ZScalar::random(&mut rng).unwrap());

        let outputs = vec![(1u64, b1), (2u64, b2)];
        let proofs = prover.create_batch_proofs(&outputs).unwrap();

        let commitments: Vec<Z00ZCommitment> = outputs
            .iter()
            .map(|(amount, blinding)| {
                create_commitment(*amount, blinding.reveal()).expect("valid blinding")
            })
            .collect();

        let commitment_bytes: Vec<&[u8]> = commitments.iter().map(|c| c.as_bytes()).collect();
        assert!(prover
            .verify_batch_proofs(&proofs, &commitment_bytes)
            .unwrap());
    }

    #[test]
    fn test_reject_tampered_proof() {
        // Test Flow:
        // 1) Generate valid commitment + range proof.
        // 2) Tamper proof bytes.
        // 3) Verify tampered proof is rejected.
        let prover = ProverImpl::new().unwrap();
        let mut rng = MockRngProvider::with_u64_seed(11).rng();
        let blinding = Hidden::hide(Z00ZScalar::random(&mut rng).unwrap());

        let amount = 7_777u64;
        let proof = prover.create_proof(amount, &blinding).unwrap();
        let commitment = create_commitment(amount, blinding.reveal()).expect("valid blinding");

        assert!(prover.verify_proof(&proof, commitment.as_bytes()).unwrap());

        let mut tampered = proof.clone();
        assert!(!tampered.is_empty());
        let flip_idx = tampered.len() / 2;
        tampered[flip_idx] ^= 0x01;

        let verify_result = prover.verify_proof(&tampered, commitment.as_bytes());
        assert!(verify_result.is_err() || matches!(verify_result, Ok(false)));
    }
}
