use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use rayon::prelude::*;
use z00z_crypto::{create_commitment, Hidden, Z00ZScalar};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    rng::MockRngProvider,
};
use z00z_wallets::tx::{
    build_tx_package_digest, Prover, ProverImpl, TxAuthWire, TxContextWire, TxInputWire, TxOutRole,
    TxOutputWire, TxPackage, TxProofWire, TxVerifier, TxVerifierImpl, TxWire,
};

const CHAIN_ID: u32 = 3;
const CHAIN_TYPE: &str = "devnet";
const CHAIN_NAME: &str = "z00z-devnet-1";

fn make_scalar(seed: u64) -> Hidden<Z00ZScalar> {
    let mut rng = MockRngProvider::with_u64_seed(seed).rng();
    Hidden::hide(Z00ZScalar::random(&mut rng).unwrap())
}

fn make_pkg(out_count: usize) -> Vec<u8> {
    let mut outputs = Vec::with_capacity(out_count);
    for idx in 0..out_count {
        let serial_id = (idx as u32) + 1;
        let asset = z00z_core::genesis::asset_std::asset_from_dev_class(
            z00z_core::assets::AssetClass::Coin,
            serial_id,
            1_000_000,
        )
        .expect("asset");
        outputs.push(TxOutputWire {
            role: TxOutRole::Recipient,
            asset_wire: z00z_core::assets::AssetPkgWire::from_wire(
                &z00z_core::assets::AssetWire::from_asset(&asset),
            ),
        });
    }

    let tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: vec![TxInputWire {
            asset_id_hex: hex::encode([7u8; 32]),
            serial_id: 1,
        }],
        outputs,
        fee: 1,
        nonce: 0,
        context: TxContextWire::default(),
        proof: TxProofWire::default(),
        auth: TxAuthWire::default(),
    };
    let pkg = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: CHAIN_ID,
        chain_type: CHAIN_TYPE.to_string(),
        chain_name: CHAIN_NAME.to_string(),
        tx: tx.clone(),
        tx_digest_hex: build_tx_package_digest(
            "TxPackage",
            "regular_tx",
            1,
            CHAIN_ID,
            CHAIN_TYPE,
            CHAIN_NAME,
            &tx,
        )
        .expect("digest"),
        status: "prepared".to_string(),
    };

    JsonCodec.serialize(&pkg).expect("serialize")
}

fn bench_prove(c: &mut Criterion) {
    let prover = ProverImpl::new().expect("prover");
    let mut g = c.benchmark_group("tx_prove");
    g.sample_size(10);

    g.bench_function("prove_seq/single", |b| {
        b.iter_batched(
            || make_scalar(101),
            |blinding| {
                let out = prover.create_proof(black_box(1_000_000), black_box(&blinding));
                black_box(out.expect("proof"));
            },
            BatchSize::SmallInput,
        )
    });

    g.bench_function("prove_seq/batch64", |b| {
        b.iter_batched(
            || {
                (0..64usize)
                    .map(|i| (1_000_000 + i as u64, make_scalar(200 + i as u64)))
                    .collect::<Vec<_>>()
            },
            |items| {
                let out = prover.create_batch_proofs(black_box(&items));
                black_box(out.expect("proofs"));
            },
            BatchSize::SmallInput,
        )
    });

    g.finish();
}

fn bench_verify(c: &mut Criterion) {
    let prover = ProverImpl::new().expect("prover");
    let verifier = TxVerifierImpl::new();
    let mut g = c.benchmark_group("tx_verify");
    g.sample_size(10);

    g.bench_function("verify_seq/proof_single", |b| {
        b.iter_batched(
            || {
                let blind = make_scalar(303);
                let proof = prover.create_proof(10_000, &blind).expect("proof");
                let commit = create_commitment(10_000, blind.reveal()).expect("commit");
                (proof, commit.as_bytes().to_vec())
            },
            |(proof, commit)| {
                let ok = prover.verify_proof(black_box(&proof), black_box(&commit));
                black_box(ok.expect("verify"));
            },
            BatchSize::SmallInput,
        )
    });

    g.bench_function("verify_seq/pkg_struct_single", |b| {
        let pkg = make_pkg(1);
        b.iter(|| {
            let ok = verifier.verify_structure(black_box(&pkg));
            black_box(ok.expect("pkg verify"));
        })
    });

    g.bench_function("verify_seq/range_single", |b| {
        let pkg = make_pkg(1);
        b.iter(|| {
            let ok = verifier.verify_range_proofs(black_box(&pkg));
            black_box(ok.expect("range verify"));
        })
    });

    g.bench_function("verify_seq/pkg_struct_batch64", |b| {
        let payloads = (0..64usize).map(|_| make_pkg(1)).collect::<Vec<_>>();
        b.iter(|| {
            let mut all_ok = true;
            for item in &payloads {
                all_ok &= verifier
                    .verify_structure(black_box(item))
                    .expect("pkg verify");
            }
            black_box(all_ok);
        })
    });

    g.bench_function("verify_seq/range_batch64", |b| {
        let payloads = (0..64usize).map(|_| make_pkg(1)).collect::<Vec<_>>();
        b.iter(|| {
            let mut all_ok = true;
            for item in &payloads {
                all_ok &= verifier
                    .verify_range_proofs(black_box(item))
                    .expect("range verify");
            }
            black_box(all_ok);
        })
    });

    g.bench_function("verify_par/pkg_struct_batch64", |b| {
        let payloads = (0..64usize).map(|_| make_pkg(1)).collect::<Vec<_>>();
        b.iter(|| {
            let all_ok = payloads
                .par_iter()
                .map(|item| verifier.verify_structure(item).expect("pkg verify"))
                .all(|v| v);
            black_box(all_ok);
        })
    });

    g.bench_function("verify_par/range_batch64", |b| {
        let payloads = (0..64usize).map(|_| make_pkg(1)).collect::<Vec<_>>();
        b.iter(|| {
            let all_ok = payloads
                .par_iter()
                .map(|item| verifier.verify_range_proofs(item).expect("range verify"))
                .all(|v| v);
            black_box(all_ok);
        })
    });

    g.finish();
}

fn bench_ser(c: &mut Criterion) {
    let codec = JsonCodec;
    let mut g = c.benchmark_group("tx_serialize");
    g.sample_size(10);

    g.bench_function("ser_seq/pkg_single", |b| {
        let bytes = make_pkg(1);
        let pkg: TxPackage = codec.deserialize(&bytes).expect("decode");
        b.iter(|| {
            let out = codec.serialize(black_box(&pkg)).expect("serialize");
            black_box(out);
        })
    });

    g.bench_function("ser_seq/pkg_large", |b| {
        let bytes = make_pkg(16);
        let pkg: TxPackage = codec.deserialize(&bytes).expect("decode");
        b.iter(|| {
            let out = codec.serialize(black_box(&pkg)).expect("serialize");
            black_box(out);
        })
    });

    g.finish();
}

criterion_group!(benches, bench_prove, bench_verify, bench_ser);
criterion_main!(benches);
