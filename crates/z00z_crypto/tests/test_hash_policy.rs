//! Phase 2 Hash Policy Tests
//!
//! SPEC Reference: specs/007-z00z-ecc-spec-2/E2E-TEST-EXAMPLES.md §2.1–§2.5

use std::collections::HashSet;

use rand::RngCore;
use z00z_crypto::hash::{
    blake2b_hash, hash_fn_for_domain, poseidon2_hash, ALL_DOMAINS, CONS_DOMAINS, WALLET_DOMAINS,
};
use z00z_crypto::hash_policy::HashFunction;

#[test]
fn test_hzk_and_hwallet_differ() {
    let data = [0x42u8; 32];
    let domains: &[&[u8]] = &[
        b"z00z.consensus.receiver_id.v1",
        b"z00z.consensus.dh_key.v1",
        b"WALLET/SEED",
    ];

    for &domain in domains {
        let zk = poseidon2_hash(domain, &[&data]);
        let wallet = blake2b_hash(domain, &[&data]);
        assert_ne!(
            zk, wallet,
            "H_zk and H_wallet must differ for domain {:?}",
            domain
        );
    }
}

#[test]
fn test_domain_registry_uniqueness() {
    let data = [0x42u8; 32];
    let mut seen: HashSet<[u8; 32]> = HashSet::new();

    for &domain in ALL_DOMAINS {
        let h = poseidon2_hash(domain, &[&data]);
        assert!(seen.insert(h), "domain collision: {:?}", domain);
    }

    assert_eq!(seen.len(), 20, "expected 20 unique domain hashes");
}

#[test]
fn test_poseidon2_deterministic() {
    let mut rng = z00z_utils::rng::SystemRngProvider.rng();

    for _ in 0..1000 {
        let mut data = [0u8; 32];
        rng.fill_bytes(&mut data);

        let h1 = poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&data]);
        let h2 = poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&data]);
        assert_eq!(h1, h2, "non-deterministic poseidon2 output");
    }
}

#[test]
fn test_poseidon2_len_always32() {
    let lens: &[usize] = &[0, 1, 31, 32, 33, 64, 128, 1000];

    for &len in lens {
        let data = vec![0x42u8; len];
        let h = poseidon2_hash(b"z00z.consensus.test.v1", &[data.as_slice()]);
        assert_eq!(h.len(), 32, "hash length must be 32 for input len={}", len);
    }
}

#[test]
fn test_hash_policy_enforce() {
    for &domain in CONS_DOMAINS {
        let f = hash_fn_for_domain(domain);
        assert!(
            matches!(f, HashFunction::Poseidon2),
            "domain {:?} must use poseidon2",
            domain
        );
    }

    for &domain in WALLET_DOMAINS {
        let f = hash_fn_for_domain(domain);
        assert!(
            matches!(f, HashFunction::Blake2b),
            "domain {:?} must use blake2b",
            domain
        );
    }

    let lowercase_wallet_domains: &[&[u8]] = &[
        b"z00z/wallet/seed",
        b"z00z/wallet/db_id",
        b"z00z/wallet/cache",
    ];
    for &domain in lowercase_wallet_domains {
        let f = hash_fn_for_domain(domain);
        assert!(
            matches!(f, HashFunction::Blake2b),
            "lowercase wallet domain {:?} must use blake2b",
            domain
        );
    }
}

#[test]
fn test_hash_policy_unknown_fallback() {
    let f = hash_fn_for_domain(b"CUSTOM/EXPERIMENT");
    assert!(
        matches!(f, HashFunction::Blake2b),
        "unknown domains must use blake2b fallback"
    );
}

#[test]
fn test_req_domain_policy() {
    let f = hash_fn_for_domain(b"z00z.payment.request.v1");
    assert!(matches!(f, HashFunction::Poseidon2));
}

#[test]
fn test_card_domain_policy() {
    let f = hash_fn_for_domain(b"z00z.receiver.card.v1");
    assert!(matches!(f, HashFunction::Poseidon2));
}
