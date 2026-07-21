//! Integration guard for the private fixed-shape Nova step owner.
//!
//! Cryptographic execution remains in the owner's unit tests because the Nova
//! dependency types are deliberately not public API. This guard prevents a
//! second module/function path from silently replacing that evidence.

const CHECKPOINT_MOD: &str = include_str!("../src/checkpoint/mod.rs");
const RECURSIVE_V2: &str = include_str!("../src/checkpoint/recursive_v2.rs");
const NOVA_OWNER: &str = include_str!("../src/checkpoint/nova.rs");

#[test]
fn test_nova_owner_is_unique() {
    assert_eq!(CHECKPOINT_MOD.matches("pub(crate) mod nova;").count(), 1);
    assert!(!RECURSIVE_V2.contains("mod nova"));
    assert!(!NOVA_OWNER.contains("recursive_v2::nova"));
    assert!(NOVA_OWNER.contains("z00z_storage::checkpoint::nova"));
}

#[test]
fn test_complete_relation_remains_wired() {
    for required in [
        "synthesize_uniqueness_products",
        "synthesize_net_merge_payload",
        "synthesize_jmt_hierarchy_payload",
        "test_checkpoint_commitments_bind_fields",
        "expected_public_state",
        "test_nova_checkpoint_proves_relation",
        "test_mixed_fixture_satisfies_testcs",
    ] {
        assert!(
            NOVA_OWNER.contains(required),
            "missing canonical Nova relation/evidence owner: {required}"
        );
    }
}
