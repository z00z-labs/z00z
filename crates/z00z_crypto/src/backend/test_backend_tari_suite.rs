use super::{backend_tari::TariCryptoBackend, CryptoBackend};
use crate::types::{Z00ZCommitment, Z00ZScalar};
use crate::{CryptoError, RangeProof};
use z00z_utils::rng::MockRngProvider;

type TestScalar = Z00ZScalar;
type Commitment = Z00ZCommitment;

fn assert_invalid_proof_size(
    result: Result<(), CryptoError>,
    index: usize,
    size: usize,
    max_size: usize,
) {
    match result {
        Err(CryptoError::InvalidProofSize {
            index: actual_index,
            size: actual_size,
            max_size: actual_max_size,
        }) => {
            assert_eq!(actual_index, index);
            assert_eq!(actual_size, size);
            assert_eq!(actual_max_size, max_size);
        }
        _ => panic!("Expected InvalidProofSize, got {:?}", result),
    }
}

#[test]
fn test_create_commitment_deterministic() {
    let backend = TariCryptoBackend;
    let amount = 1000u64;

    let mut rng = MockRngProvider::with_u64_seed(1).rng();
    let blinding1 = TestScalar::random(&mut rng).unwrap();
    let blinding1_copy = blinding1.dangerous_clone();

    let c1 = backend.create_commitment(amount, &blinding1);
    let c2 = backend.create_commitment(amount, &blinding1_copy);
    assert_eq!(c1, c2);
}

#[test]
fn test_create_commitment_different_blinding() {
    let backend = TariCryptoBackend;
    let amount = 1000u64;

    let mut rng = MockRngProvider::with_u64_seed(2).rng();
    let blinding1 = TestScalar::random(&mut rng).unwrap();
    let blinding2 = TestScalar::random(&mut rng).unwrap();

    let c1 = backend.create_commitment(amount, &blinding1);
    let c2 = backend.create_commitment(amount, &blinding2);
    assert_ne!(c1, c2);
}

#[test]
fn test_create_range_proof_valid() {
    let backend = TariCryptoBackend;
    let amount = 1000u64;
    let mut rng = MockRngProvider::with_u64_seed(3).rng();
    let blinding = TestScalar::random(&mut rng).unwrap();

    let result = backend.create_range_proof(amount, &blinding, 64, 0);
    assert!(result.is_ok());
    let proof = result.unwrap();
    assert!(!proof.is_empty());
}

#[test]
fn test_create_range_proof_bits() {
    let backend = TariCryptoBackend;
    let amount = 1000u64;
    let mut rng = MockRngProvider::with_u64_seed(4).rng();
    let blinding = TestScalar::random(&mut rng).unwrap();

    let result = backend.create_range_proof(amount, &blinding, 0, 0);
    assert!(result.is_err());

    let result = backend.create_range_proof(amount, &blinding, 65, 0);
    assert!(result.is_err());

    let result = backend.create_range_proof(amount, &blinding, 32, 0);
    assert!(result.is_err());
}

#[test]
fn test_create_range_proof_exceeds() {
    let backend = TariCryptoBackend;
    let amount = u64::MAX;
    let mut rng = MockRngProvider::with_u64_seed(5).rng();
    let blinding = TestScalar::random(&mut rng).unwrap();

    let result = backend.create_range_proof(amount, &blinding, 64, 0);
    assert!(result.is_ok());
}

#[test]
fn test_verify_range_proof_valid() {
    let backend = TariCryptoBackend;
    let amount = 1000u64;
    let mut rng = MockRngProvider::with_u64_seed(6).rng();
    let blinding = TestScalar::random(&mut rng).unwrap();

    let commitment = backend.create_commitment(amount, &blinding);
    let proof = backend
        .create_range_proof(amount, &blinding, 64, 0)
        .expect("Proof generation failed");

    let result = backend.verify_range_proof(&proof, &commitment, 64, 1, 0);
    assert!(result.is_ok());
}

#[test]
fn test_verify_range_proof_tampered() {
    let backend = TariCryptoBackend;
    let amount = 1000u64;
    let mut rng = MockRngProvider::with_u64_seed(7).rng();
    let blinding = TestScalar::random(&mut rng).unwrap();

    let commitment = backend.create_commitment(amount, &blinding);
    let mut proof = backend
        .create_range_proof(amount, &blinding, 64, 0)
        .expect("Proof generation failed");

    if !proof.is_empty() {
        proof[0] ^= 0xFF;
    }

    let result = backend.verify_range_proof(&proof, &commitment, 64, 1, 0);
    assert!(result.is_err());
}

#[test]
fn test_verify_range_proof_wrong() {
    let backend = TariCryptoBackend;
    let amount1 = 1000u64;
    let amount2 = 2000u64;
    let mut rng = MockRngProvider::with_u64_seed(8).rng();
    let blinding1 = TestScalar::random(&mut rng).unwrap();
    let blinding2 = TestScalar::random(&mut rng).unwrap();

    let _commitment1 = backend.create_commitment(amount1, &blinding1);
    let commitment2 = backend.create_commitment(amount2, &blinding2);
    let proof = backend
        .create_range_proof(amount1, &blinding1, 64, 0)
        .expect("Proof generation failed");

    let result = backend.verify_range_proof(&proof, &commitment2, 64, 1, 0);
    assert!(result.is_err());
}

#[test]
fn test_derive_hash_deterministic() {
    let backend = TariCryptoBackend;
    let domain = b"test_domain";
    let data1 = b"test_data";
    let data: &[&[u8]] = &[data1];

    let hash1 = backend.derive_hash(domain, data);
    let hash2 = backend.derive_hash(domain, data);

    assert_eq!(hash1, hash2);
}

#[test]
fn test_derive_hash_domain_separation() {
    let backend = TariCryptoBackend;
    let data1 = b"test_data";
    let data: &[&[u8]] = &[data1];

    let hash1 = backend.derive_hash(b"domain1", data);
    let hash2 = backend.derive_hash(b"domain2", data);

    assert_ne!(hash1, hash2);
}

#[test]
fn test_derive_hash_multiple_chunks() {
    let backend = TariCryptoBackend;
    let domain = b"test";
    let chunk1 = b"hello";
    let chunk2 = b"world";
    let data: &[&[u8]] = &[chunk1, chunk2];

    let hash1 = backend.derive_hash(domain, data);
    let hash2 = backend.derive_hash(domain, data);

    assert_eq!(hash1, hash2);
}

#[test]
fn test_batch_verify_valid_proofs() {
    let backend = TariCryptoBackend;
    let mut rng = MockRngProvider::with_u64_seed(9).rng();

    let amounts = [100u64, 200, 300, 400, 500, 600, 700, 800, 900, 1000];
    let mut proofs = Vec::new();
    let mut commitments = Vec::new();

    for &amount in &amounts {
        let blinding = TestScalar::random(&mut rng).unwrap();
        let commitment = backend.create_commitment(amount, &blinding);
        let proof = backend
            .create_range_proof(amount, &blinding, 64, 0)
            .expect("Proof generation failed");
        proofs.push(proof);
        commitments.push(commitment);
    }

    let proof_refs: Vec<_> = proofs.iter().collect();
    let commit_refs: Vec<_> = commitments.iter().collect();
    let minimum_value_promises = vec![0u64; 10];
    let result = backend.batch_verify_range_proofs(
        &proof_refs,
        &commit_refs,
        64,
        1,
        &minimum_value_promises,
    );

    assert!(
        result.is_ok(),
        "Batch verification should succeed for valid proofs"
    );
}

#[test]
fn test_batch_verify_single_invalid() {
    let backend = TariCryptoBackend;
    let mut rng = MockRngProvider::with_u64_seed(10).rng();

    let amounts = [100u64, 200, 300, 400, 500];
    let mut proofs = Vec::new();
    let mut commitments = Vec::new();

    for &amount in &amounts {
        let blinding = TestScalar::random(&mut rng).unwrap();
        let commitment = backend.create_commitment(amount, &blinding);
        let proof = backend
            .create_range_proof(amount, &blinding, 64, 0)
            .expect("Proof generation failed");
        proofs.push(proof);
        commitments.push(commitment);
    }

    if !proofs[2].is_empty() {
        proofs[2][0] ^= 0xFF;
    }

    let proof_refs: Vec<_> = proofs.iter().collect();
    let commit_refs: Vec<_> = commitments.iter().collect();
    let minimum_value_promises = vec![0u64; 5];
    let result = backend.batch_verify_range_proofs(
        &proof_refs,
        &commit_refs,
        64,
        1,
        &minimum_value_promises,
    );

    assert!(
        result.is_err(),
        "Batch verification should fail when one proof is invalid"
    );
}

#[test]
fn test_batch_verify_empty() {
    let backend = TariCryptoBackend;

    let proofs: Vec<&RangeProof> = Vec::new();
    let commitments: Vec<&Commitment> = Vec::new();
    let minimum_value_promises = vec![0u64; 0];
    let result =
        backend.batch_verify_range_proofs(&proofs, &commitments, 64, 1, &minimum_value_promises);

    assert!(
        result.is_ok(),
        "Batch verification of empty set should succeed"
    );
}

#[test]
#[cfg(all(feature = "logging", not(target_arch = "wasm32")))]
fn test_empty_batch_warns() {
    let backend = TariCryptoBackend;

    let proofs: Vec<&RangeProof> = Vec::new();
    let commitments: Vec<&Commitment> = Vec::new();
    let minimum_value_promises = vec![];
    let result =
        backend.batch_verify_range_proofs(&proofs, &commitments, 64, 1, &minimum_value_promises);

    assert!(result.is_ok(), "Empty batch should succeed");
}

#[test]
fn test_batch_verify_mismatched_count() {
    let backend = TariCryptoBackend;
    let mut rng = MockRngProvider::with_u64_seed(11).rng();

    let amounts = [100u64, 200, 300];
    let mut proofs = Vec::new();
    let mut commitments = Vec::new();

    for (idx, &amount) in amounts.iter().enumerate() {
        let blinding = TestScalar::random(&mut rng).unwrap();
        let commitment = backend.create_commitment(amount, &blinding);
        let proof = backend
            .create_range_proof(amount, &blinding, 64, 0)
            .expect("Proof generation failed");
        proofs.push(proof);
        if idx < 2 {
            commitments.push(commitment);
        }
    }

    let proof_refs: Vec<_> = proofs.iter().collect();
    let commit_refs: Vec<_> = commitments.iter().collect();
    let minimum_value_promises = vec![0u64; 3];
    let result = backend.batch_verify_range_proofs(
        &proof_refs,
        &commit_refs,
        64,
        1,
        &minimum_value_promises,
    );

    assert!(
        result.is_err(),
        "Batch verification should fail on mismatched counts"
    );
    if let Err(CryptoError::InvalidParameters { param }) = result {
        assert_eq!(param, "proof_commitment_mismatch");
    } else {
        panic!("Expected InvalidParameters error");
    }
}

#[test]
fn test_batch_verify_single_proof() {
    let backend = TariCryptoBackend;
    let mut rng = MockRngProvider::with_u64_seed(12).rng();

    let amount = 1000u64;
    let blinding = TestScalar::random(&mut rng).unwrap();
    let commitment = backend.create_commitment(amount, &blinding);
    let proof = backend
        .create_range_proof(amount, &blinding, 64, 0)
        .expect("Proof generation failed");

    let proof_refs = vec![&proof];
    let commit_refs = vec![&commitment];
    let minimum_value_promises = vec![0u64; 1];
    let result = backend.batch_verify_range_proofs(
        &proof_refs,
        &commit_refs,
        64,
        1,
        &minimum_value_promises,
    );

    assert!(
        result.is_ok(),
        "Batch verification should work for single proof"
    );
}

#[test]
fn test_backend_info() {
    let backend = TariCryptoBackend;
    let info = backend.backend_info();

    assert_eq!(info.name, "TariCryptoBackend");
    assert!(!info.version.is_empty());
    assert!(info.algorithms.contains(&"Pedersen"));
    assert!(info.algorithms.contains(&"Bulletproofs+"));
    assert!(info.algorithms.contains(&"BLAKE2b"));

    let metadata_map: std::collections::HashMap<_, _> = info.metadata.iter().cloned().collect();
    assert_eq!(metadata_map.get("curve"), Some(&"Ristretto255"));
    assert_eq!(metadata_map.get("range_proof"), Some(&"Bulletproofs+"));
    assert_eq!(metadata_map.get("hash"), Some(&"BLAKE2b-256"));
    assert_eq!(metadata_map.get("lazy_static"), Some(&"true"));
    assert_eq!(metadata_map.get("thread_safe"), Some(&"true"));
}

#[test]
fn test_backend_info_methods() {
    let backend = TariCryptoBackend;
    let info = backend.backend_info();

    let formatted = format!(
        "Backend: {} v{} | Algorithms: {:?}",
        info.name, info.version, info.algorithms
    );

    assert!(formatted.contains("TariCryptoBackend"));
    assert!(formatted.contains("Pedersen"));
}

#[test]
fn test_verify_rejects_oversized_proofs() {
    use crate::types::{MAX_PROOF_SIZE, MAX_PROOF_SIZE_EXTENDED};

    let backend = TariCryptoBackend;
    let mut rng = MockRngProvider::with_u64_seed(13).rng();
    let blinding = TestScalar::random(&mut rng).unwrap();
    let commitment = backend.create_commitment(1000u64, &blinding);

    let oversized_proof = vec![0u8; MAX_PROOF_SIZE + 1];
    let proof_refs = [&oversized_proof];
    let commit_refs = [&commitment];
    let minimum_value_promises = vec![0u64; 1];

    let result = backend.batch_verify_range_proofs(
        &proof_refs,
        &commit_refs,
        64,
        1,
        &minimum_value_promises,
    );
    assert_invalid_proof_size(result, 0, MAX_PROOF_SIZE + 1, MAX_PROOF_SIZE);

    let oversized_proof_128 = vec![0u8; MAX_PROOF_SIZE_EXTENDED + 1];
    let proof_refs_128 = [&oversized_proof_128];
    let commit_refs_128 = [&commitment];
    let minimum_value_promises = vec![0u64; 1];

    let result = backend.batch_verify_range_proofs(
        &proof_refs_128,
        &commit_refs_128,
        64,
        1,
        &minimum_value_promises,
    );
    assert_invalid_proof_size(result, 0, MAX_PROOF_SIZE_EXTENDED + 1, MAX_PROOF_SIZE);
}

#[test]
fn test_verify_rejects_many_proofs() {
    use crate::types::MAX_BATCH_PROOF_COUNT;

    let backend = TariCryptoBackend;
    let mut rng = MockRngProvider::with_u64_seed(14).rng();
    let blinding = TestScalar::random(&mut rng).unwrap();
    let commitment = backend.create_commitment(1000u64, &blinding);
    let valid_proof = vec![0u8; 100];

    let proofs: Vec<RangeProof> = (0..=MAX_BATCH_PROOF_COUNT)
        .map(|_| valid_proof.clone())
        .collect();
    let commitments: Vec<Commitment> = (0..=MAX_BATCH_PROOF_COUNT)
        .map(|_| commitment.clone())
        .collect();

    let proof_refs: Vec<_> = proofs.iter().collect();
    let commit_refs: Vec<_> = commitments.iter().collect();
    let minimum_value_promises = vec![0u64; MAX_BATCH_PROOF_COUNT + 1];

    let result = backend.batch_verify_range_proofs(
        &proof_refs,
        &commit_refs,
        64,
        1,
        &minimum_value_promises,
    );

    match result {
        Err(CryptoError::BatchTooLarge { count, max }) => {
            assert_eq!(count, MAX_BATCH_PROOF_COUNT + 1);
            assert_eq!(max, MAX_BATCH_PROOF_COUNT);
        }
        _ => panic!("Expected BatchTooLarge, got {:?}", result),
    }
}

#[test]
fn test_batch_verify_mixed_sizes() {
    use crate::types::MAX_PROOF_SIZE;

    let backend = TariCryptoBackend;
    let mut rng = MockRngProvider::with_u64_seed(15).rng();
    let blinding = TestScalar::random(&mut rng).unwrap();
    let commitment = backend.create_commitment(1000u64, &blinding);

    let valid_proof_1 = vec![0u8; 100];
    let valid_proof_2 = vec![0u8; 200];
    let oversized_proof = vec![0u8; MAX_PROOF_SIZE + 1];

    let proof_refs = [&valid_proof_1, &valid_proof_2, &oversized_proof];
    let commit_refs = [&commitment, &commitment, &commitment];
    let minimum_value_promises = vec![0u64; 3];

    let result = backend.batch_verify_range_proofs(
        &proof_refs,
        &commit_refs,
        64,
        1,
        &minimum_value_promises,
    );
    assert_invalid_proof_size(result, 2, MAX_PROOF_SIZE + 1, MAX_PROOF_SIZE);
}

#[test]
fn test_verify_rejects_excessive_memory() {
    use crate::types::{MAX_BATCH_MEMORY, MAX_PROOF_SIZE};

    let backend = TariCryptoBackend;
    let mut rng = MockRngProvider::with_u64_seed(16).rng();

    let proof_count = (MAX_BATCH_MEMORY / MAX_PROOF_SIZE) + 100;
    let blinding = TestScalar::random(&mut rng).unwrap();
    let commitment = backend.create_commitment(1000u64, &blinding);
    let valid_proof = vec![0u8; MAX_PROOF_SIZE - 1];

    let proofs: Vec<RangeProof> = (0..proof_count).map(|_| valid_proof.clone()).collect();
    let commitments: Vec<Commitment> = (0..proof_count).map(|_| commitment.clone()).collect();

    let proof_refs: Vec<_> = proofs.iter().collect();
    let commit_refs: Vec<_> = commitments.iter().collect();
    let minimum_value_promises = vec![0u64; proof_count];

    let result = backend.batch_verify_range_proofs(
        &proof_refs,
        &commit_refs,
        64,
        1,
        &minimum_value_promises,
    );

    match result {
        Err(CryptoError::ExcessiveMemoryUsage) => {}
        _ => panic!("Expected ExcessiveMemoryUsage, got {:?}", result),
    }
}
