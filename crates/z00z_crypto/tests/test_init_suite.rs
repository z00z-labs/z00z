//! Initialization tests for cryptographic services.
//!
//! Verifies that all crypto services initialize correctly at startup
//! and are accessible after initialization.

use z00z_crypto::{create_commitment, create_range_proof, verify_range_proof, Z00ZScalar};

/// Test that crypto services initialize without panic.
/// Also verifies idempotency - safe to call multiple times.
#[test]
fn test_crypto_init() {
    // Should not panic with valid system
    z00z_crypto::initialize();

    // Idempotent - safe to call again
    z00z_crypto::initialize();
}

/// Test that services are accessible after initialization.
#[test]
fn test_services_work_after_init() {
    // Force initialization
    z00z_crypto::initialize();

    // Create commitment - uses COMMITMENT_FACTORY
    let blinding = Z00ZScalar::random(&mut z00z_utils::rng::SystemRngProvider.rng()).unwrap();
    let amount = 1000u64;
    let commitment = create_commitment(amount, &blinding).expect("commitment creation failed");

    // Create range proof - uses BULLETPROOF_SERVICE
    let proof =
        create_range_proof(amount, &blinding, 64, 0).expect("Range proof generation failed");

    // Verify range proof - uses BULLETPROOF_SERVICE
    verify_range_proof(&proof, &commitment, 64, 1, 0).expect("Range proof verification failed");
}
