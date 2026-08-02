use std::time::Instant;

use rand::RngCore;
use z00z_crypto::{
    batch_verify_range_proofs, commitment::commit_value, create_range_proof, verify_range_proof,
    Z00ZScalar,
};

fn verify_entries(
    entries: &[(z00z_crypto::Z00ZCommitment, Vec<u8>)],
) -> Result<Vec<bool>, z00z_crypto::CryptoError> {
    if entries.is_empty() {
        return Ok(Vec::new());
    }

    let proofs: Vec<&Vec<u8>> = entries.iter().map(|item| &item.1).collect();
    let commitments: Vec<&z00z_crypto::Z00ZCommitment> =
        entries.iter().map(|item| &item.0).collect();
    let mins = vec![0u64; entries.len()];

    match batch_verify_range_proofs(&proofs, &commitments, 64, 1, &mins) {
        Ok(()) => Ok(vec![true; entries.len()]),
        Err(_) => entries
            .iter()
            .map(|(commitment, proof)| {
                verify_range_proof(proof, commitment, 64, 1, 0)
                    .map(|_| true)
                    .or(Ok(false))
            })
            .collect(),
    }
}

#[test]
fn test_range_values() {
    let mut rng = z00z_utils::rng::SystemRngProvider.rng();
    let values = [0u64, 1, 100, 12345, u32::MAX as u64, u64::MAX - 1, u64::MAX];

    for value in values {
        let blinding = Z00ZScalar::random(&mut rng).unwrap();
        let commitment = commit_value(value, &blinding);
        let proof = create_range_proof(value, &blinding, 64, 0).expect("proof generation failed");
        let ok = verify_range_proof(&proof, &commitment, 64, 1, 0).is_ok();
        assert!(ok, "valid range proof rejected for value={}", value);
    }
}

#[test]
fn test_wrong_commitment() {
    let mut rng = z00z_utils::rng::SystemRngProvider.rng();

    let value = 1000u64;
    let blinding = Z00ZScalar::random(&mut rng).unwrap();
    let commitment_real = commit_value(value, &blinding);
    let proof = create_range_proof(value, &blinding, 64, 0).expect("proof");

    assert!(
        verify_range_proof(&proof, &commitment_real, 64, 1, 0).is_ok(),
        "real commitment must pass"
    );

    let wrong = commit_value(2000, &Z00ZScalar::random(&mut rng).unwrap());
    let ok_wrong = verify_range_proof(&proof, &wrong, 64, 1, 0).is_ok();
    assert!(!ok_wrong, "tampered commitment must fail");

    let rand_value = rng.next_u64();
    let rand_blind = Z00ZScalar::random(&mut rng).unwrap();
    let rand_pt = commit_value(rand_value, &rand_blind);
    let result = verify_range_proof(&proof, &rand_pt, 64, 1, 0);
    assert!(result.is_err());
}

#[test]
fn test_batch_verify_100() {
    let mut rng = z00z_utils::rng::SystemRngProvider.rng();
    let mut entries = Vec::new();

    for i in 0..100u64 {
        let v = (i + 1) * 100;
        let r = Z00ZScalar::random(&mut rng).unwrap();
        let c = commit_value(v, &r);
        let proof = create_range_proof(v, &r, 64, 0).expect("proof");
        entries.push((c, proof));
    }

    let results = verify_entries(&entries).expect("batch verify");
    assert!(results.iter().all(|&ok| ok), "all 100 proofs must pass");

    entries[42].1[0] ^= 0x01;
    let results_bad = verify_entries(&entries).expect("batch verify bad");
    assert!(!results_bad[42], "corrupted proof #42 must fail");
}

#[test]
#[ignore]
fn test_batch_perf_100ms() {
    let mut rng = z00z_utils::rng::SystemRngProvider.rng();
    let mut entries = Vec::new();

    for i in 0..100u64 {
        let v = (i + 1) * 100;
        let r = Z00ZScalar::random(&mut rng).unwrap();
        let c = commit_value(v, &r);
        let proof = create_range_proof(v, &r, 64, 0).expect("proof");
        entries.push((c, proof));
    }

    let start = Instant::now();
    let _ = verify_entries(&entries).expect("batch verify");
    let ms = start.elapsed().as_millis();

    assert!(ms < 100, "batch of 100 took {}ms, expected <100ms", ms);
}
