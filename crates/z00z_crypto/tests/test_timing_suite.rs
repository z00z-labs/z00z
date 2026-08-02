//! Timing attack resistance tests for constant-time operations.
//!
//! These tests verify that cryptographic operations use constant-time
//! techniques to prevent timing side-channels.
//!
//! ⚠️ NOTE: Timing tests are not perfect and may be flaky in CI environments
//! due to CPU load, caching, and other factors. They catch obvious leaks
//! but are not cryptographically rigorous.

use z00z_crypto::{batch_verify_range_proofs, create_commitment, create_range_proof, Z00ZScalar};

/// Test batch verification with invalid proof at different positions.
///
/// Verifies that validation time is similar regardless of where the
/// invalid proof appears in the batch (constant-time property).
#[test]
fn test_batch_verify_constant_time() {
    z00z_crypto::initialize();

    // Create valid proofs
    let mut valid_proofs = Vec::new();
    let mut valid_commitments = Vec::new();

    for _ in 0..10 {
        let blinding = Z00ZScalar::random(&mut z00z_utils::rng::SystemRngProvider.rng()).unwrap();
        let amount = 1000u64;
        let commitment = create_commitment(amount, &blinding).expect("commitment creation failed");
        let proof = create_range_proof(amount, &blinding, 64, 0).expect("proof creation failed");

        valid_proofs.push(proof);
        valid_commitments.push(commitment);
    }

    // Create an invalid proof (empty)
    let invalid_proof = vec![];

    // Measure multiple rounds and compare median ratio to reduce flakiness
    let mut ratios = Vec::with_capacity(9);

    for _ in 0..9 {
        // Test 1: Invalid proof at position 0 (early)
        let mut batch_early = vec![&invalid_proof];
        batch_early.extend(valid_proofs.iter().skip(1).take(5).collect::<Vec<_>>());
        let commits_early: Vec<_> = valid_commitments.iter().take(6).collect();
        let min_values_early = vec![0u64; 6];

        let t1 = std::time::Instant::now();
        let result_early =
            batch_verify_range_proofs(&batch_early, &commits_early, 64, 1, &min_values_early);
        let d1 = t1.elapsed();

        assert!(
            result_early.is_err(),
            "Expected batch with invalid proof to fail"
        );

        // Test 2: Invalid proof at position 5 (late)
        let mut batch_late = valid_proofs.iter().take(5).collect::<Vec<_>>();
        batch_late.push(&invalid_proof);
        let commits_late: Vec<_> = valid_commitments.iter().take(6).collect();
        let min_values_late = vec![0u64; 6];

        let t2 = std::time::Instant::now();
        let result_late =
            batch_verify_range_proofs(&batch_late, &commits_late, 64, 1, &min_values_late);
        let d2 = t2.elapsed();

        assert!(
            result_late.is_err(),
            "Expected batch with invalid proof to fail"
        );

        let d1_nanos = d1.as_nanos().max(1);
        let d2_nanos = d2.as_nanos().max(1);
        let ratio = if d1_nanos > d2_nanos {
            d1_nanos as f64 / d2_nanos as f64
        } else {
            d2_nanos as f64 / d1_nanos as f64
        };
        ratios.push(ratio);
    }

    ratios.sort_by(|a, b| a.partial_cmp(b).expect("ratio must be finite"));
    let median = ratios[ratios.len() / 2];

    // Median tolerance is intentionally looser for CI/VM noise.
    assert!(median < 5.0, "Timing median ratio too high: {:.2}x", median);
}

/// Test constant-time validation with oversized proofs (not just empty).
///
/// Verifies that oversized proof detection is also constant-time.
#[test]
fn test_oversized_proof_timing() {
    z00z_crypto::initialize();

    // Create valid proofs
    let mut valid_proofs = Vec::new();
    let mut valid_commitments = Vec::new();

    for _ in 0..10 {
        let blinding = Z00ZScalar::random(&mut z00z_utils::rng::SystemRngProvider.rng()).unwrap();
        let amount = 1000u64;
        let commitment = create_commitment(amount, &blinding).expect("commitment creation failed");
        let proof = create_range_proof(amount, &blinding, 64, 0).expect("proof creation failed");

        valid_proofs.push(proof);
        valid_commitments.push(commitment);
    }

    // Create an oversized proof (> 10 KB)
    let oversized_proof = vec![0u8; 11_000];

    // Measure multiple rounds and compare median ratio to reduce scheduler noise.
    let mut ratios = Vec::with_capacity(9);

    for _ in 0..9 {
        // Test 1: Oversized proof at position 0
        let mut batch_early = vec![&oversized_proof];
        batch_early.extend(valid_proofs.iter().skip(1).take(5).collect::<Vec<_>>());
        let commits_early: Vec<_> = valid_commitments.iter().take(6).collect();
        let min_values_early = vec![0u64; 6];

        let t1 = std::time::Instant::now();
        let result_early =
            batch_verify_range_proofs(&batch_early, &commits_early, 64, 1, &min_values_early);
        let d1 = t1.elapsed();

        assert!(
            result_early.is_err(),
            "Expected batch with oversized proof to fail"
        );

        // Test 2: Oversized proof at position 5
        let mut batch_late = valid_proofs.iter().take(5).collect::<Vec<_>>();
        batch_late.push(&oversized_proof);
        let commits_late: Vec<_> = valid_commitments.iter().take(6).collect();
        let min_values_late = vec![0u64; 6];

        let t2 = std::time::Instant::now();
        let result_late =
            batch_verify_range_proofs(&batch_late, &commits_late, 64, 1, &min_values_late);
        let d2 = t2.elapsed();

        assert!(
            result_late.is_err(),
            "Expected batch with oversized proof to fail"
        );

        let d1_nanos = d1.as_nanos().max(1);
        let d2_nanos = d2.as_nanos().max(1);
        let ratio = if d1_nanos > d2_nanos {
            d1_nanos as f64 / d2_nanos as f64
        } else {
            d2_nanos as f64 / d1_nanos as f64
        };
        ratios.push(ratio);
    }

    ratios.sort_by(|a, b| a.partial_cmp(b).expect("ratio must be finite"));
    let median = ratios[ratios.len() / 2];
    let max_ratio = if cfg!(debug_assertions) { 14.0 } else { 10.0 };

    assert!(
        median < max_ratio,
        "Oversized proof timing median ratio too high: {:.2}x",
        median
    );
}

/// Test that error reporting is correct for different invalid proof positions.
///
/// Verifies that the function correctly identifies which proof in the batch
/// is invalid (functional correctness, not timing).
#[test]
fn test_batch_error_reporting() {
    z00z_crypto::initialize();

    // Create valid proofs
    let mut valid_proofs = Vec::new();
    let mut valid_commitments = Vec::new();

    for _ in 0..5 {
        let blinding = Z00ZScalar::random(&mut z00z_utils::rng::SystemRngProvider.rng()).unwrap();
        let amount = 1000u64;
        let commitment = create_commitment(amount, &blinding).expect("commitment creation failed");
        let proof = create_range_proof(amount, &blinding, 64, 0).expect("proof creation failed");

        valid_proofs.push(proof);
        valid_commitments.push(commitment);
    }

    // Test empty proof at position 0
    let empty_proof = vec![];
    let mut batch = vec![&empty_proof];
    batch.extend(valid_proofs.iter().take(4).collect::<Vec<_>>());
    let commits: Vec<_> = valid_commitments.iter().take(5).collect();
    let min_values = vec![0u64; 5];

    let result = batch_verify_range_proofs(&batch, &commits, 64, 1, &min_values);
    assert!(result.is_err(), "Expected error for empty proof");

    // Test empty proof at position 2
    let mut batch = valid_proofs.iter().take(2).collect::<Vec<_>>();
    batch.push(&empty_proof);
    batch.extend(valid_proofs.iter().skip(2).take(2).collect::<Vec<_>>());
    let commits: Vec<_> = valid_commitments.iter().take(5).collect();
    let min_values = vec![0u64; 5];

    let result = batch_verify_range_proofs(&batch, &commits, 64, 1, &min_values);
    assert!(
        result.is_err(),
        "Expected error for empty proof at position 2"
    );

    // Test oversized proof at position 1
    let oversized_proof = vec![0u8; 11_000];
    let mut batch = vec![valid_proofs.first().unwrap()];
    batch.push(&oversized_proof);
    batch.extend(valid_proofs.iter().skip(1).take(3).collect::<Vec<_>>());
    let commits: Vec<_> = valid_commitments.iter().take(5).collect();
    let min_values = vec![0u64; 5];

    let result = batch_verify_range_proofs(&batch, &commits, 64, 1, &min_values);
    assert!(
        result.is_err(),
        "Expected error for oversized proof at position 1"
    );
}
