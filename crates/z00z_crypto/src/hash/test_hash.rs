use std::convert::{TryFrom, TryInto};

use super::*;

#[test]
fn test_domain_separation_changes_output() {
    let first = blake2b_256("d1", "l", &[b"x"]);
    let second = blake2b_256("d2", "l", &[b"x"]);
    assert_ne!(first, second);

    let third = blake2b_256("d1", "l1", &[b"x"]);
    let fourth = blake2b_256("d1", "l2", &[b"x"]);
    assert_ne!(third, fourth);
}

#[test]
fn test_sha256_blake2b_are_deterministic() {
    let x1 = sha256_256("d", "l", &[b"x", b"y"]);
    let x2 = sha256_256("d", "l", &[b"x", b"y"]);
    assert_eq!(x1, x2);

    let y1 = blake2b_512("d", "l", &[b"x", b"y"]);
    let y2 = blake2b_512("d", "l", &[b"x", b"y"]);
    assert_eq!(y1, y2);
}

#[test]
fn checkpoint_sha_roles_are_distinct_and_fips_frozen() {
    let root = sha256_256_role(CheckpointShaRole::SettlementRoot, &[b"same"]);
    let trace = sha256_256_role(CheckpointShaRole::Trace, &[b"same"]);
    let spent = sha256_256_role(CheckpointShaRole::SpentOriginalIds, &[b"same"]);
    let output = sha256_256_role(CheckpointShaRole::OutputOriginalIds, &[b"same"]);
    assert_ne!(root, trace);
    assert_ne!(spent, output);
    assert_eq!(SHA256_BLOCK_BYTES_V2, 64);
    assert_eq!(SHA256_MAX_BYTES_V2, (1_u64 << 61) - 1);
    assert_eq!(SHA256_IV_V2[0], 0x6a09e667);
    assert_eq!(SHA256_IV_V2[7], 0x5be0cd19);
}

#[test]
fn checkpoint_sha_registry_has_no_duplicate_semantic_pair() {
    for (index, role) in crate::hash::domains::ALL_CHECKPOINT_SHA_ROLES_V2
        .iter()
        .enumerate()
    {
        for other in &crate::hash::domains::ALL_CHECKPOINT_SHA_ROLES_V2[index + 1..] {
            assert!(
                role.domain() != other.domain() || role.label() != other.label(),
                "checkpoint SHA roles must not share a semantic domain/label pair"
            );
        }
    }
}

#[test]
fn checkpoint_sha_stream_matches_the_role_framing() {
    let parts: &[&[u8]] = &[b"first", b"second", b""];
    let role = CheckpointShaRole::Trace;
    let expected = sha256_256(role.domain(), role.label(), parts);
    let mut stream = CheckpointSha256V2::new(role);
    for part in parts {
        stream.update_part(part).expect("bounded input");
    }
    assert_eq!(stream.finalize(), expected);
}

#[test]
fn checkpoint_sha_block_stream_matches_rustcrypto_at_fips_boundaries() {
    for length in [0_usize, 1, 55, 56, 63, 64, 65, 1_024] {
        let payload = vec![0xA5; length];
        let expected = sha256_256_role(
            CheckpointShaRole::Trace,
            &[b"z00z.recursive.v2.source-record-hash", &payload],
        );
        let mut blocks = Vec::new();
        let mut stream = CheckpointSha256BlockStreamV2::new(CheckpointShaRole::Trace);
        stream
            .update_part_with(b"z00z.recursive.v2.source-record-hash", &mut |block| {
                blocks.push(block);
                Ok::<(), ()>(())
            })
            .expect("fixed label fits");
        stream
            .update_part_with(&payload, &mut |block| {
                blocks.push(block);
                Ok::<(), ()>(())
            })
            .expect("bounded payload fits");
        let actual = stream
            .finalize_with(&mut |block| {
                blocks.push(block);
                Ok::<(), ()>(())
            })
            .expect("bounded transcript finalizes");
        assert_eq!(actual, expected, "payload length {length}");
        assert!(!blocks.is_empty());
        assert_eq!(blocks[0].chaining_before(), &SHA256_IV_V2);
        assert!(blocks.last().expect("one final block").final_block());
        assert!(blocks
            .iter()
            .all(CheckpointSha256BlockV2::verifies_transition));
        assert_eq!(
            blocks.len() as u64,
            crate::hash::sha256_hash::CheckpointSha256BlockStreamV2::framed_bytes_for_parts(
                CheckpointShaRole::Trace,
                u64::try_from(b"z00z.recursive.v2.source-record-hash".len() + length)
                    .expect("test length"),
                2,
            )
            .and_then(|bytes| {
                let with_padding = bytes
                    .checked_add(9)
                    .ok_or(CheckpointSha256Error::InputTooLong)?;
                Ok(with_padding.div_ceil(SHA256_BLOCK_BYTES_V2))
            })
            .expect("block count")
        );
        for pair in blocks.windows(2) {
            assert_eq!(pair[0].chaining_after(), pair[1].chaining_before());
        }
    }
}

#[test]
fn checkpoint_sha_block_stream_accepts_one_part_in_canonical_chunks() {
    let label = b"z00z.recursive.v2.source-record-hash";
    let payload = vec![0xA5; 129];
    let expected = sha256_256_role(CheckpointShaRole::Trace, &[label, &payload]);
    let mut blocks = Vec::new();
    let mut stream = CheckpointSha256BlockStreamV2::new(CheckpointShaRole::Trace);
    stream
        .update_part_with(label, &mut |block| {
            blocks.push(block);
            Ok::<(), ()>(())
        })
        .expect("label is framed once");
    stream
        .begin_part_with(
            u64::try_from(payload.len()).expect("test length"),
            &mut |block| {
                blocks.push(block);
                Ok::<(), ()>(())
            },
        )
        .expect("payload part begins");
    for chunk in payload.chunks(64) {
        stream
            .update_part_bytes_with(chunk, &mut |block| {
                blocks.push(block);
                Ok::<(), ()>(())
            })
            .expect("canonical chunk fits declared part");
    }
    stream.finish_part().expect("declared part is complete");
    let actual = stream
        .finalize_with(&mut |block| {
            blocks.push(block);
            Ok::<(), ()>(())
        })
        .expect("complete transcript finalizes");
    assert_eq!(actual, expected);
    assert!(blocks
        .iter()
        .all(CheckpointSha256BlockV2::verifies_transition));
}

#[test]
fn checkpoint_sha_block_stream_rejects_incomplete_or_oversized_part() {
    let mut stream = CheckpointSha256BlockStreamV2::new(CheckpointShaRole::Trace);
    stream
        .begin_part_with(2, &mut |_| Ok::<(), ()>(()))
        .expect("part starts");
    stream
        .update_part_bytes_with(&[1], &mut |_| Ok::<(), ()>(()))
        .expect("first byte fits");
    assert_eq!(
        stream.finish_part(),
        Err(CheckpointSha256Error::IncompletePart)
    );
    assert!(matches!(
        stream.update_part_bytes_with(&[2, 3], &mut |_| Ok::<(), ()>(())),
        Err(CheckpointSha256BlockVisitError::Hash(
            CheckpointSha256Error::PartTooLong
        ))
    ));
}

#[test]
fn checkpoint_sha_block_count_rejects_fips_length_overflow() {
    assert_eq!(
        CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(SHA256_MAX_BYTES_V2)
            .expect("maximum FIPS message"),
        (SHA256_MAX_BYTES_V2 + 9).div_ceil(SHA256_BLOCK_BYTES_V2)
    );
    assert!(
        CheckpointSha256BlockStreamV2::block_count_for_framed_bytes(SHA256_MAX_BYTES_V2 + 1)
            .is_err()
    );
}

#[test]
fn checkpoint_sha_role_prefix_is_the_single_framing_owner() {
    for role in crate::hash::domains::ALL_CHECKPOINT_SHA_ROLES_V2 {
        let prefix = CheckpointSha256BlockStreamV2::framed_role_prefix(*role);
        assert_eq!(
            u64::try_from(prefix.len()).expect("fixed prefix length"),
            CheckpointSha256BlockStreamV2::framed_bytes_for_parts(*role, 0, 0)
                .expect("fixed role framing"),
        );
        let declared_len = u64::from_le_bytes(prefix[..8].try_into().expect("prefix length"));
        assert_eq!(
            usize::try_from(declared_len).expect("fixed domain length"),
            prefix.len() - 8
        );
    }
}

#[test]
fn test_hmac_sha_256_works() {
    let key = b"secret";
    let msg = b"hello";
    let first = hmac_sha256(key, "auth", "v1", msg);
    let second = hmac_sha256(key, "auth", "v1", msg);
    assert_eq!(first, second);

    let third = hmac_sha256(key, "auth", "v1", b"hello2");
    assert_ne!(first, third);
}

#[test]
fn test_domain_raw_hmac_deterministic() {
    let key = b"secret";
    let msg = b"hello";

    let first = hmac_sha256_raw(key, msg);
    let second = hmac_sha256_raw(key, msg);
    assert_eq!(first, second);

    let third = hmac_sha256(key, "wallet.index", "v1", msg);
    assert_ne!(first, third);
}

#[test]
fn test_length_prefixing_prevents_confusion() {
    let hash1 = blake2b_256("domain", "label", &[b"hello", b"world"]);
    let hash2 = blake2b_256("domain", "label", &[b"helloworld"]);
    assert_ne!(hash1, hash2);
}

#[test]
fn test_hmac_key_conditioning() {
    let long_key = [0x42u8; 100];
    let msg = b"test";
    let first = hmac_sha256(&long_key, "test", "v1", msg);
    let second = hmac_sha256(&long_key, "test", "v1", msg);
    assert_eq!(first, second);
}

#[test]
fn test_empty_parts() {
    let hash = blake2b_256("domain", "label", &[]);
    assert_eq!(hash.len(), 32);
}

#[test]
fn test_large_data() {
    let large_data = vec![0x42u8; 10000];
    let hash = blake2b_512("domain", "label", &[&large_data]);
    assert_eq!(hash.len(), 64);
}

#[test]
fn test_benchmark_blake2b256_small() {
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = blake2b_256("test.domain", "label", &[b"small"]);
    }
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 1000, "Performance issue");
}

#[test]
fn test_benchmark_blake2b512_medium() {
    let data = vec![0x42u8; 1024];
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = blake2b_512("test.domain", "label", &[&data]);
    }
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 1000, "Performance issue");
}

#[test]
fn test_benchmark_hmac_sha256() {
    let key = b"secret-key-32-bytes-long";
    let msg = b"message";
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = hmac_sha256(key, "test.domain", "v1", msg);
    }
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 1000, "Performance issue");
}

#[test]
fn test_domain_separation_consistency() {
    let domain = "wallet.keys";
    let label = "derive";
    let parts: &[&[u8]] = &[b"master", &1u64.to_le_bytes()];

    let hash1 = blake2b_256(domain, label, parts);
    let hash2 = blake2b_256(domain, label, parts);
    assert_eq!(hash1, hash2);

    let hash3 = blake2b_512(domain, label, parts);
    let hash4 = blake2b_512(domain, label, parts);
    assert_eq!(hash3, hash4);

    let hash5 = sha256_256(domain, label, parts);
    let hash6 = sha256_256(domain, label, parts);
    assert_eq!(hash5, hash6);
}

#[test]
fn test_cross_function_uniqueness() {
    let domain = "test";
    let label = "label";
    let parts: &[&[u8]] = &[b"data"];

    let blake256 = blake2b_256(domain, label, parts);
    let blake512 = blake2b_512(domain, label, parts);
    let sha256 = sha256_256(domain, label, parts);

    assert_eq!(blake256.len(), 32);
    assert_eq!(blake512.len(), 64);
    assert_eq!(sha256.len(), 32);
    assert_ne!(blake256.to_vec(), sha256.to_vec());
}
