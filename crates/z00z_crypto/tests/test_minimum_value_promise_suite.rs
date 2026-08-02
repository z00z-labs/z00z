//! Tests for minimum_value_promise parameter validation.
//!
//! Verifies that range proof verification correctly handles
//! minimum value promises and that mismatched values are rejected.

use z00z_crypto::{create_commitment, create_range_proof, verify_range_proof, Z00ZScalar};

/// Test that range proof verification accepts matching minimum value.
///
/// For now, Z00Z protocol uses standard minimum_value_promise=0.
/// This test verifies the happy path with standard value.
#[test]
fn test_range_proof_standard_min() {
    z00z_crypto::initialize();

    let amount = 1000u64;
    let blinding = Z00ZScalar::random(&mut z00z_utils::rng::SystemRngProvider.rng()).unwrap();
    let commitment = create_commitment(amount, &blinding).expect("commitment creation failed");

    let proof = create_range_proof(amount, &blinding, 64, 0).expect("Proof creation failed");

    // Verification with standard minimum (0) should succeed
    let result = verify_range_proof(&proof, &commitment, 64, 1, 0);
    assert!(
        result.is_ok(),
        "Verification should succeed with correct minimum value"
    );
}

/// Test that verification with zero minimum value works.
///
/// Zero is the standard minimum value promise in Z00Z.
#[test]
fn test_range_proof_zero_min() {
    z00z_crypto::initialize();

    // Test with various amounts (all should work with min=0)
    let test_cases = [0u64, 1, 100, 1000, u64::MAX];

    for &amount in &test_cases {
        let blinding = Z00ZScalar::random(&mut z00z_utils::rng::SystemRngProvider.rng()).unwrap();
        let commitment = create_commitment(amount, &blinding).expect("commitment creation failed");

        let proof = create_range_proof(amount, &blinding, 64, 0).expect("Proof creation failed");

        let result = verify_range_proof(&proof, &commitment, 64, 1, 0);
        assert!(
            result.is_ok(),
            "Verification should succeed for amount {} with minimum=0",
            amount
        );
    }
}

/// Test that minimum_value_promise parameter is correctly passed to Tari.
///
/// This is a functional test to ensure the parameter flows through
/// the entire verification pipeline.
#[test]
fn test_min_promise_flow() {
    z00z_crypto::initialize();

    let amount = 5000u64;
    let blinding = Z00ZScalar::random(&mut z00z_utils::rng::SystemRngProvider.rng()).unwrap();
    let commitment = create_commitment(amount, &blinding).expect("commitment creation failed");

    let min_value = 1000u64;
    let proof =
        create_range_proof(amount, &blinding, 64, min_value).expect("Proof creation failed");

    let result_ok = verify_range_proof(&proof, &commitment, 64, 1, min_value);
    assert!(
        result_ok.is_ok(),
        "Verification should succeed with matching minimum"
    );

    let result_bad = verify_range_proof(&proof, &commitment, 64, 1, 0);
    assert!(
        result_bad.is_err(),
        "Verification should fail with mismatched minimum"
    );
}

/// Test that batch verification accepts matching minimum values.
#[test]
fn test_batch_verify_min_values() {
    use z00z_crypto::batch_verify_range_proofs;

    z00z_crypto::initialize();

    let amounts = [100u64, 200, 300, 400, 500];
    let mut proofs = Vec::new();
    let mut commitments = Vec::new();

    for &amount in &amounts {
        let blinding = Z00ZScalar::random(&mut z00z_utils::rng::SystemRngProvider.rng()).unwrap();
        let commitment = create_commitment(amount, &blinding).expect("commitment creation failed");
        let proof = create_range_proof(amount, &blinding, 64, 0).expect("Proof creation failed");
        proofs.push(proof);
        commitments.push(commitment);
    }

    let proof_refs: Vec<_> = proofs.iter().collect();
    let commit_refs: Vec<_> = commitments.iter().collect();

    // All proofs created with minimum=0, verify with matching minimums
    let minimum_values = vec![0u64; amounts.len()];
    let result = batch_verify_range_proofs(&proof_refs, &commit_refs, 64, 1, &minimum_values);

    assert!(
        result.is_ok(),
        "Batch verification should succeed with correct minimum values"
    );
}

/// Test that batch verification rejects mismatched minimum_values array length.
#[test]
fn test_batch_verify_wrong_length() {
    use z00z_crypto::batch_verify_range_proofs;

    z00z_crypto::initialize();

    let amounts = [100u64, 200, 300];
    let mut proofs = Vec::new();
    let mut commitments = Vec::new();

    for &amount in &amounts {
        let blinding = Z00ZScalar::random(&mut z00z_utils::rng::SystemRngProvider.rng()).unwrap();
        let commitment = create_commitment(amount, &blinding).expect("commitment creation failed");
        let proof = create_range_proof(amount, &blinding, 64, 0).expect("Proof creation failed");
        proofs.push(proof);
        commitments.push(commitment);
    }

    let proof_refs: Vec<_> = proofs.iter().collect();
    let commit_refs: Vec<_> = commitments.iter().collect();

    // Wrong array length (2 instead of 3)
    let wrong_length_minimums = vec![0u64; 2];
    let result =
        batch_verify_range_proofs(&proof_refs, &commit_refs, 64, 1, &wrong_length_minimums);

    assert!(
        result.is_err(),
        "Batch verification should reject mismatched minimum_values length"
    );
}

#[test]
fn test_create_proof_min_gt() {
    z00z_crypto::initialize();

    let amount = 100u64;
    let min_value = 101u64;
    let blinding = Z00ZScalar::random(&mut z00z_utils::rng::SystemRngProvider.rng()).unwrap();

    let result = create_range_proof(amount, &blinding, 64, min_value);
    assert!(result.is_err(), "Creation should reject minimum > amount");
}
