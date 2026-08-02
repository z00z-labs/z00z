use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Instant;
use z00z_crypto::protocol::ecdh::compute_stealth_dh_sender;
use z00z_crypto::{
    decrypt_asset_package_transport, encrypt_asset_package_transport,
    verify_asset_output_proofs_batch, AssetOutputProof, AssetRangeProof, Commitment,
    Z00ZRistrettoPoint, Z00ZScalar,
};

fn test_batch_verification_performance() {
    let mut outputs = Vec::new();
    for _ in 0..10 {
        outputs.push(AssetOutputProof::new(100u64).expect("output"));
    }

    let start = Instant::now();
    let result = verify_asset_output_proofs_batch(&outputs).expect("batch verify");
    let elapsed = start.elapsed();

    assert!(result, "batch verification result must be true");
    assert!(
        elapsed.as_millis() < 100,
        "batch verification too slow: {:?}",
        elapsed
    );
}

// Baseline target: ECDH shared secret < 50μs.
fn bench_ecdh_shared(c: &mut Criterion) {
    let sk = Z00ZScalar::from_hash(&[11u8; 64]).unwrap();
    let view_sk = Z00ZScalar::from_hash(&[22u8; 64]).unwrap();
    let pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);

    c.bench_function("ecdh_shared_secret", |b| {
        b.iter(|| {
            let _ = compute_stealth_dh_sender(black_box(&sk), black_box(&pk));
        })
    });
}

// Baseline target: commitment generation < 10μs.
fn bench_commit_gen(c: &mut Criterion) {
    let value = 100u64;
    let blind = Z00ZScalar::from_hash(&[33u8; 64]).unwrap();

    c.bench_function("commitment_generation", |b| {
        b.iter(|| {
            let _ = Commitment::new_with_blinding(black_box(value), black_box(&blind));
        })
    });
}

// Baseline target: range proof generation < 50ms.
fn bench_range_gen(c: &mut Criterion) {
    let value = 100u64;
    let blind = Z00ZScalar::from_hash(&[44u8; 64]).unwrap();

    c.bench_function("range_proof_generation", |b| {
        b.iter(|| {
            let _ = AssetRangeProof::new(black_box(value), black_box(&blind));
        })
    });
}

// Baseline target: range proof verification < 20ms.
fn bench_range_verify(c: &mut Criterion) {
    let value = 100u64;
    let blind = Z00ZScalar::from_hash(&[55u8; 64]).unwrap();
    let proof = AssetRangeProof::new(value, &blind).expect("proof");
    let commitment = Commitment::new_with_blinding(value, &blind).expect("commitment");

    c.bench_function("range_proof_verification", |b| {
        b.iter(|| {
            let _ = proof.verify(black_box(&commitment));
        })
    });
}

// Baseline target: batch verification (10 proofs) < 100ms.
fn bench_batch_verify(c: &mut Criterion) {
    test_batch_verification_performance();

    let mut outputs = Vec::new();
    for _ in 0..10 {
        outputs.push(AssetOutputProof::new(100u64).expect("output"));
    }

    c.bench_function("range_proof_batch_verify_10", |b| {
        b.iter(|| {
            let _ = verify_asset_output_proofs_batch(black_box(&outputs));
        })
    });
}

// Baseline target: AEAD encrypt/decrypt (1KB) < 1ms.
fn bench_aead_encrypt(c: &mut Criterion) {
    let key = [0x42u8; 32];
    let aad = [0xAAu8; 64];
    let payload = vec![0x11u8; 1024];

    c.bench_function("aead_encrypt_1kb", |b| {
        b.iter(|| {
            let _ = encrypt_asset_package_transport(
                black_box(&key),
                black_box(aad.as_slice()),
                black_box(payload.as_slice()),
            );
        })
    });
}

fn bench_aead_decrypt(c: &mut Criterion) {
    let key = [0x42u8; 32];
    let aad = [0xAAu8; 64];
    let payload = vec![0x11u8; 1024];
    let envelope =
        encrypt_asset_package_transport(&key, aad.as_slice(), payload.as_slice()).expect("encrypt");

    c.bench_function("aead_decrypt_1kb", |b| {
        b.iter(|| {
            let _ = decrypt_asset_package_transport(
                black_box(&key),
                black_box(aad.as_slice()),
                black_box(envelope.as_slice()),
            );
        })
    });
}

criterion_group!(
    crypto_benches,
    bench_ecdh_shared,
    bench_commit_gen,
    bench_range_gen,
    bench_range_verify,
    bench_batch_verify,
    bench_aead_encrypt,
    bench_aead_decrypt,
);
criterion_main!(crypto_benches);
