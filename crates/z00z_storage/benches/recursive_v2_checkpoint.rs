//! Release benchmark for the strict recursive-V2 registry/preheader ingress.
//!
//! This target measures bounded framing and admission for the three public
//! recursive object families. It does not claim to measure Nova proving,
//! cryptographic verification, persistence, or the complete lifecycle.

use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use z00z_storage::checkpoint::recursive_v2::{
    CheckpointVersionRegistryV2, RecursiveBoundedObjectV2, RECURSIVE_OBJECT_PREHEADER_BYTES_V2,
};

const KIB: usize = 1024;
const PUBLIC_OBJECTS: [(RecursiveBoundedObjectV2, &str, usize); 3] = [
    (
        RecursiveBoundedObjectV2::NovaBlockProof,
        "nova_proof",
        16 * KIB,
    ),
    (
        RecursiveBoundedObjectV2::RecursiveCheckpointSidecar,
        "sidecar",
        8 * KIB,
    ),
    (
        RecursiveBoundedObjectV2::CryptographicVerificationReceipt,
        "receipt",
        2 * KIB,
    ),
];

struct FramedFixture {
    class: &'static str,
    bytes: Vec<u8>,
}

fn framed_payload(
    registry: &CheckpointVersionRegistryV2,
    object: RecursiveBoundedObjectV2,
    payload_len: usize,
) -> Vec<u8> {
    let header = registry
        .encode_preheader(object, payload_len)
        .expect("authority-pinned public-object preheader");
    let mut framed = Vec::with_capacity(RECURSIVE_OBJECT_PREHEADER_BYTES_V2 + payload_len);
    framed.extend_from_slice(&header);
    framed.resize(RECURSIVE_OBJECT_PREHEADER_BYTES_V2 + payload_len, 0xA5);
    framed
}

fn object_cap(registry: &CheckpointVersionRegistryV2, object: RecursiveBoundedObjectV2) -> usize {
    usize::try_from(
        registry
            .row(object)
            .expect("registered public recursive object")
            .max_encoded_len,
    )
    .expect("public recursive object cap fits usize")
}

fn fixtures(
    registry: &CheckpointVersionRegistryV2,
    object: RecursiveBoundedObjectV2,
    representative_len: usize,
) -> Vec<FramedFixture> {
    let cap = object_cap(registry, object);
    assert!(representative_len > 0 && representative_len < cap);
    [
        ("minimum", 1),
        ("representative", representative_len),
        ("maximum", cap),
    ]
    .into_iter()
    .map(|(class, payload_len)| {
        let bytes = framed_payload(registry, object, payload_len);
        let validated = registry
            .validate_preheader(&bytes, object)
            .expect("canonical benchmark fixture passes strict ingress");
        assert_eq!(
            validated.declared_len,
            u64::try_from(payload_len).expect("fixture length fits u64")
        );
        FramedFixture { class, bytes }
    })
    .collect()
}

fn assert_fail_closed_boundaries(registry: &CheckpointVersionRegistryV2) {
    for (object, _, representative_len) in PUBLIC_OBJECTS {
        let cap = object_cap(registry, object);
        registry
            .validate_preheader(
                &framed_payload(registry, object, representative_len),
                object,
            )
            .expect("representative public object passes strict ingress");
        assert!(registry.encode_preheader(object, cap + 1).is_err());
    }

    let nova = framed_payload(registry, RecursiveBoundedObjectV2::NovaBlockProof, 1);
    assert!(registry
        .validate_preheader(&nova, RecursiveBoundedObjectV2::RecursiveCheckpointSidecar,)
        .is_err());

    let mut wrong_wire = nova.clone();
    wrong_wire[8] ^= 1;
    assert!(registry
        .validate_preheader(&wrong_wire, RecursiveBoundedObjectV2::NovaBlockProof)
        .is_err());
    assert!(registry
        .validate_preheader(
            &nova[..RECURSIVE_OBJECT_PREHEADER_BYTES_V2 - 1],
            RecursiveBoundedObjectV2::NovaBlockProof,
        )
        .is_err());
}

fn bench_registry_authority(c: &mut Criterion) {
    let registry = CheckpointVersionRegistryV2::authority_pinned()
        .expect("authority-pinned recursive version registry");
    let mut group = c.benchmark_group("recursive_v2_registry_authority");
    group.throughput(Throughput::Elements(1));
    group.bench_function("resolve_validate_and_digest", |bencher| {
        bencher.iter(|| {
            let resolved = CheckpointVersionRegistryV2::authority_pinned()
                .expect("authority remains pinned during benchmark");
            black_box(resolved.digest());
        });
    });
    group.bench_function("lookup_nova_row", |bencher| {
        bencher.iter(|| {
            let row = registry
                .row(black_box(RecursiveBoundedObjectV2::NovaBlockProof))
                .expect("Nova row remains registered");
            black_box(row);
        });
    });
    group.finish();
}

fn bench_preheader_encode(c: &mut Criterion) {
    let registry = CheckpointVersionRegistryV2::authority_pinned()
        .expect("authority-pinned recursive version registry");
    let mut group = c.benchmark_group("recursive_v2_preheader_encode");
    group.throughput(Throughput::Elements(1));

    for (object, object_name, representative_len) in PUBLIC_OBJECTS {
        let cap = object_cap(&registry, object);
        for (class, payload_len) in [
            ("minimum", 1),
            ("representative", representative_len),
            ("maximum", cap),
        ] {
            group.bench_with_input(
                BenchmarkId::new(object_name, class),
                &payload_len,
                |bencher, payload_len| {
                    bencher.iter(|| {
                        let header = registry
                            .encode_preheader(black_box(object), black_box(*payload_len))
                            .expect("registered public object stays encodable");
                        black_box(header);
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_preheader_validate(c: &mut Criterion) {
    let registry = CheckpointVersionRegistryV2::authority_pinned()
        .expect("authority-pinned recursive version registry");
    let mut group = c.benchmark_group("recursive_v2_preheader_validate");
    group.throughput(Throughput::Elements(1));

    for (object, object_name, representative_len) in PUBLIC_OBJECTS {
        for fixture in fixtures(&registry, object, representative_len) {
            group.bench_with_input(
                BenchmarkId::new(object_name, fixture.class),
                &fixture,
                |bencher, fixture| {
                    bencher.iter(|| {
                        let validated = registry
                            .validate_preheader(black_box(fixture.bytes.as_slice()), object)
                            .expect("canonical fixture stays valid");
                        black_box(validated);
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_preheader_reject(c: &mut Criterion) {
    let registry = CheckpointVersionRegistryV2::authority_pinned()
        .expect("authority-pinned recursive version registry");
    let nova = framed_payload(&registry, RecursiveBoundedObjectV2::NovaBlockProof, 1);
    let mut wrong_wire = nova.clone();
    wrong_wire[8] ^= 1;
    let cap = object_cap(&registry, RecursiveBoundedObjectV2::NovaBlockProof);

    let mut group = c.benchmark_group("recursive_v2_preheader_reject");
    group.throughput(Throughput::Elements(1));
    group.bench_function("cross_type", |bencher| {
        bencher.iter(|| {
            let rejected = registry
                .validate_preheader(
                    black_box(nova.as_slice()),
                    RecursiveBoundedObjectV2::RecursiveCheckpointSidecar,
                )
                .expect_err("cross-type bytes stay rejected");
            black_box(rejected);
        });
    });
    group.bench_function("wrong_wire", |bencher| {
        bencher.iter(|| {
            let rejected = registry
                .validate_preheader(
                    black_box(wrong_wire.as_slice()),
                    RecursiveBoundedObjectV2::NovaBlockProof,
                )
                .expect_err("wire mutation stays rejected");
            black_box(rejected);
        });
    });
    group.bench_function("cap_plus_one_before_allocation", |bencher| {
        bencher.iter(|| {
            let rejected = registry
                .encode_preheader(RecursiveBoundedObjectV2::NovaBlockProof, black_box(cap + 1))
                .expect_err("cap + 1 stays rejected before payload allocation");
            black_box(rejected);
        });
    });
    group.finish();
}

fn main() {
    let registry = CheckpointVersionRegistryV2::authority_pinned()
        .expect("authority-pinned recursive version registry");
    assert_fail_closed_boundaries(&registry);

    let mut criterion = Criterion::default().configure_from_args();
    bench_registry_authority(&mut criterion);
    bench_preheader_encode(&mut criterion);
    bench_preheader_validate(&mut criterion);
    bench_preheader_reject(&mut criterion);
    criterion.final_summary();
}
