//! Security-focused edge case tests.
//!
//! Covers vulnerabilities, boundary conditions, and regression risks.

use z00z_crypto::*;

mod zero_value_tests {
    use super::*;

    #[test]
    fn test_zero_blinding_factor_rejected() {
        let zero_blinding = Z00ZScalar::try_from_bytes([0u8; 32]).expect("zero scalar parsing");
        let result = create_commitment(1, &zero_blinding);
        assert!(result.is_err(), "Zero blinding factor must be rejected");
        assert!(matches!(
            result,
            Err(CryptoError::InvalidBlindingFactorZero)
        ));
    }

    #[test]
    fn test_non_zero_blinding_accepted() {
        let mut bytes = [0u8; 32];
        bytes[0] = 1;
        let result = Z00ZScalar::try_from_bytes(bytes);
        assert!(
            result.is_ok(),
            "Non-zero blinding factor should be accepted"
        );
    }
}

mod overflow_tests {
    use super::*;

    #[test]
    fn test_argon2_multiplication_overflow() {
        let params = Argon2Params {
            memory: u32::MAX,
            iterations: u32::MAX,
            parallelism: u32::MAX,
        };

        let result = params.validate_untrusted();
        assert!(result.is_err(), "Malicious parameters must be rejected");
    }

    #[test]
    fn test_argon2_valid_large_params() {
        let params = Argon2Params::moderate();
        assert!(params.validate_untrusted().is_ok());
    }
}

mod batch_tests {
    use super::*;
    use z00z_utils::rng::MockRngProvider;

    fn make_proof_and_commitment() -> (RangeProof, Z00ZCommitment) {
        let provider = MockRngProvider::with_u64_seed(42);
        let mut rng = provider.rng();

        let amount = 123u64;
        let blinding = Z00ZScalar::random(&mut rng).unwrap();
        let commitment = create_commitment(amount, &blinding).expect("commitment creation failed");
        let proof = create_range_proof(amount, &blinding, 64, 0).expect("valid proof");

        (proof, commitment)
    }

    #[test]
    fn test_batch_verify_all_invalid() {
        let count = 16usize;

        let provider = MockRngProvider::with_u64_seed(7);
        let mut rng = provider.rng();

        let mut proofs: Vec<RangeProof> = Vec::with_capacity(count);
        let mut commitments: Vec<Z00ZCommitment> = Vec::with_capacity(count);
        let minimum_value_promises: Vec<u64> = vec![0u64; count];

        for _ in 0..count {
            let amount = 1u64;
            let blinding = Z00ZScalar::random(&mut rng).unwrap();
            commitments
                .push(create_commitment(amount, &blinding).expect("commitment creation failed"));

            // Start from a valid proof, then corrupt it while keeping the same length.
            // This ensures we exercise verification logic (not just empty/parse rejection).
            let mut proof = create_range_proof(amount, &blinding, 64, 0).expect("valid proof");
            if let Some(last) = proof.last_mut() {
                *last ^= 0x01;
            }
            proofs.push(proof);
        }

        let proof_refs: Vec<&RangeProof> = proofs.iter().collect();
        let commit_refs: Vec<&Z00ZCommitment> = commitments.iter().collect();

        let result =
            batch_verify_range_proofs(&proof_refs, &commit_refs, 64, 1, &minimum_value_promises);

        match result {
            Ok(()) => panic!("All-invalid batch must fail"),
            Err(
                CryptoError::BatchVerificationFailed
                | CryptoError::ProofVerificationFailed
                | CryptoError::ProofCommitmentMismatch,
            ) => {}
            Err(other) => panic!("Unexpected error variant: {:?}", other),
        }
    }

    #[test]
    fn test_batch_verify_length_mismatch() {
        let (proof, commitment) = make_proof_and_commitment();
        let proofs = vec![&proof; 2];
        let commitments = vec![&commitment; 1];
        let minimum_value_promises = vec![0u64; 2];

        let result =
            batch_verify_range_proofs(&proofs, &commitments, 64, 1, &minimum_value_promises);
        assert!(result.is_err(), "Length mismatch must fail");
    }
}

mod hkdf_tests {
    use super::*;

    #[test]
    fn test_hkdf_empty_short_ikm() {
        let ikm = [1u8; 16];
        let result = hkdf_expand_32(&ikm, b"", b"context");
        assert!(result.is_err(), "Short IKM with empty salt must fail");
    }

    #[test]
    fn test_hkdf_empty_long_ikm() {
        let ikm = [1u8; 32];
        let result = hkdf_expand_32(&ikm, b"", b"context");
        assert!(result.is_ok(), "Long IKM allows empty salt");
    }

    #[test]
    fn test_hkdf_empty_info_rejected() {
        let ikm = [1u8; 32];
        let result = hkdf_expand_32(&ikm, b"salt", b"");
        assert!(result.is_err(), "Empty info must be rejected");
    }
}

mod collision_tests {
    use super::*;

    #[test]
    fn test_domain_separation_no_collisions() {
        let hash1 = derive_hash(b"domain", &[b"ab", b"cd"]);
        let hash2 = derive_hash(b"domain", &[b"a", b"bcd"]);

        assert_ne!(hash1, hash2, "Length-prefixing must prevent collisions");
    }

    #[test]
    fn test_domain_separation_utf8_normalization() {
        let hash1 = derive_hash(b"caf\xc3\xa9", &[b"data"]);
        let hash2 = derive_hash(b"cafe\xcc\x81", &[b"data"]);

        assert_ne!(hash1, hash2, "No implicit normalization is expected");
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod aead_tests {
    use super::*;
    use z00z_crypto::aead::seal_with_rng;
    use z00z_utils::rng::MockRngProvider;

    #[test]
    fn test_aead_nonce_uniqueness() {
        let provider = MockRngProvider::with_u64_seed(99);
        let mut rng = provider.rng();

        let key = [1u8; 32];
        let plaintext = b"test";
        let aad = b"aad";

        let ct1 = seal_with_rng(&mut rng, &key, aad, plaintext).unwrap();
        let ct2 = seal_with_rng(&mut rng, &key, aad, plaintext).unwrap();

        let nonce1 = &ct1[1..(1 + XCHACHA_NONCE_SIZE)];
        let nonce2 = &ct2[1..(1 + XCHACHA_NONCE_SIZE)];

        assert_ne!(nonce1, nonce2, "Nonces must differ across encryptions");
    }
}

mod timing_tests {
    use super::*;

    #[test]
    fn test_constant_time_comparison() {
        let mut bytes1 = [0u8; 32];
        bytes1[0] = 1;
        let mut bytes2 = [0u8; 32];
        bytes2[0] = 1;
        let mut bytes3 = [0u8; 32];
        bytes3[0] = 2;

        let bf1 = Z00ZScalar::try_from_bytes(bytes1).unwrap();
        let bf2 = Z00ZScalar::try_from_bytes(bytes2).unwrap();
        let bf3 = Z00ZScalar::try_from_bytes(bytes3).unwrap();

        assert!(bf1.ct_eq(&bf2));
        assert!(!bf1.ct_eq(&bf3));
    }
}
