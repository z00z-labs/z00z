const WHITEPAPER_DOC: &str = include_str!("../../../docs/whitepapers/Main-Whitepaper.md");
const ROADMAP_DOC: &str = include_str!("../../../docs/tech-papers/Z00Z-Roadmap-Blueprint.md");
const ONIONNET_DOC: &str = include_str!("../../z00z_networks/onionnet/README.md");
const WALLET_LIB_SRC: &str = include_str!("../src/lib.rs");
const APP_MOD_SRC: &str = include_str!("../src/app/mod.rs");
const APP_KERNEL_SRC: &str = include_str!("../src/app/app_kernel.rs");
const CHAIN_CLIENT_SRC: &str = include_str!("../src/chain/chain_client_impl.rs");
const REVIEW_EXECUTION_PROMPT: &str =
    include_str!("../../../.github/prompts/gsd-review-tasks-execution.prompt.md");
const FULL_VERIFY_SKILL: &str =
    include_str!("../../../.github/skills/z00z-full-verify-gate/SKILL.md");
const STORAGE_BENCHES_DOC: &str = include_str!("../../z00z_storage/benches/settlement_benches.md");
const SETTLEMENT_README_DOC: &str = include_str!("../../z00z_storage/src/settlement/README.md");
const HJMT_THREAT_DOC: &str =
    include_str!("../../../docs/tech-papers/done/Z00Z-HJMT-Threat-Model.md");

fn assert_present(label: &str, source: &str, needle: &str) {
    assert!(source.contains(needle), "{label} missing {needle:?}");
}

fn assert_absent(label: &str, source: &str, needle: &str) {
    assert!(
        !source.contains(needle),
        "{label} still contains {needle:?}"
    );
}

#[test]
fn onionnet_and_remote_chain_surfaces_stay_deferred() {
    assert_present(
        "whitepaper",
        WHITEPAPER_DOC,
        "privacy against network-level traffic analysis is not yet a shipped base-layer guarantee",
    );
    assert_present(
        "whitepaper",
        WHITEPAPER_DOC,
        "OnionNet currently exists as a reserved boundary crate",
    );
    assert_present(
        "whitepaper",
        WHITEPAPER_DOC,
        "wallet-side OnionNet switching still returns deterministic placeholder behavior",
    );
    assert_present(
        "roadmap",
        ROADMAP_DOC,
        "OnionNet | Reserved boundary with design specification",
    );
    assert_present(
        "roadmap",
        ROADMAP_DOC,
        "network/privacy claims remain bounded by executable evidence",
    );
    assert_present("onionnet readme", ONIONNET_DOC, "placeholder seam");
    assert_present(
        "onionnet readme",
        ONIONNET_DOC,
        "future OnionNet work lands in the same",
    );
    assert_present(
        "app kernel",
        APP_KERNEL_SRC,
        "OnionNet transport is not represented by `ChainType`",
    );
    assert_present(
        "app kernel",
        APP_KERNEL_SRC,
        "local fallback chain selection.",
    );
    assert_absent("app kernel", APP_KERNEL_SRC, "deterministic placeholder");
    assert_absent("app kernel", APP_KERNEL_SRC, "Core app stub:");
    assert_absent("app kernel", APP_KERNEL_SRC, "Phase 1 stub");
    assert_absent("wallet lib", WALLET_LIB_SRC, "stub-heavy");
    assert_absent("app mod", APP_MOD_SRC, "RPC stubs only");
    assert_present(
        "chain client",
        CHAIN_CLIENT_SRC,
        "Real remote-node transport remains an explicit adapter-only seam",
    );
    assert_present(
        "chain client",
        CHAIN_CLIENT_SRC,
        "remote node adapter is not configured",
    );
}

#[test]
fn da_slashing_and_fraud_claims_stay_honest() {
    assert_present(
        "whitepaper",
        WHITEPAPER_DOC,
        "there is no fully landed slashing or fraud-proof execution engine",
    );
    assert_present(
        "whitepaper",
        WHITEPAPER_DOC,
        "does not yet ship the full provider implementation",
    );
    assert_present(
        "whitepaper",
        WHITEPAPER_DOC,
        "the chain client is still a Phase 1 stub",
    );
    assert_present(
        "roadmap",
        ROADMAP_DOC,
        "still explicitly implementation-blocked",
    );
    assert_present(
        "roadmap",
        ROADMAP_DOC,
        "future anonymous ingress architecture and runtime sink",
    );
    assert_present(
        "roadmap",
        ROADMAP_DOC,
        "reserved or planned privacy boundaries rather than deployed",
    );
}

#[test]
fn release_authority_docs_do_not_normalize_debug_release_features() {
    assert!(
        !REVIEW_EXECUTION_PROMPT
            .contains("cargo test -p z00z_wallets --release --features test-params-fast"),
        "review prompt must not normalize release wallet builds with test-params-fast",
    );
    assert!(
        !REVIEW_EXECUTION_PROMPT.contains(
            "cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools",
        ),
        "review prompt must not normalize release simulator debug features",
    );
    assert_present(
        "review prompt",
        REVIEW_EXECUTION_PROMPT,
        "bash scripts/audit/audit_release_feature_guards.sh",
    );
    assert!(
        !FULL_VERIFY_SKILL.contains(
            "cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_tools"
        ),
        "full verify skill must not normalize release simulator debug runs",
    );
    assert!(
        !FULL_VERIFY_SKILL.contains(
            "cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools",
        ),
        "full verify skill must not normalize release simulator debug tests",
    );
    assert_present(
        "full verify skill",
        FULL_VERIFY_SKILL,
        "bash scripts/audit/audit_release_feature_guards.sh",
    );
    assert!(
        !STORAGE_BENCHES_DOC.contains(
            "cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools",
        ),
        "bench closeout doc must not normalize release simulator debug features",
    );
    assert!(
        !STORAGE_BENCHES_DOC.contains(
            "cargo run --release -p z00z_simulator --bin scenario_1 --features test-params-fast --features wallet_debug_tools",
        ),
        "bench closeout doc must not normalize release simulator debug runs",
    );
    assert!(
        !SETTLEMENT_README_DOC.contains(
            "cargo test -p z00z_wallets --release --features test-params-fast --features wallet_debug_tools",
        ),
        "settlement readme must not normalize release wallet debug features",
    );
    assert!(
        !SETTLEMENT_README_DOC.contains(
            "cargo test -p z00z_simulator --test test_stage4_digest --release --features test-params-fast --features wallet_debug_tools",
        ),
        "settlement readme must not normalize release simulator debug tests",
    );
    assert!(
        !HJMT_THREAT_DOC.contains(
            "cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario_settlement",
        ),
        "threat model doc must not normalize release simulator fast-test features",
    );
    assert_present(
        "settlement readme",
        SETTLEMENT_README_DOC,
        "bash scripts/audit/audit_release_feature_guards.sh",
    );
    assert_present(
        "threat model doc",
        HJMT_THREAT_DOC,
        "bash scripts/audit/audit_release_feature_guards.sh",
    );
}
