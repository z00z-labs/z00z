use std::path::PathBuf;
use std::sync::Arc;

use rand::rngs::OsRng;
use z00z_core::config_paths::{devnet_assets_path, devnet_genesis_path};
use z00z_core::genesis::{
    genesis_config::load_genesis_config, run_genesis, validator::GenesisError,
    GENESIS_POLICIES_FILE, GENESIS_RIGHTS_FILE, GENESIS_SETTLEMENT_MANIFEST_FILE,
    GENESIS_VOUCHERS_FILE,
};
use z00z_core::{
    Asset, AssetClass, AssetDefinition, AssetDefinitionRegistry, BlindingFactor, ChainType,
};
use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};

const README_DOC: &str = include_str!("../README.md");
const LIB_SRC: &str = include_str!("../src/lib.rs");
const ASSETS_DOC: &str = include_str!("../src/assets/mod.rs");
const ASSET_IMPL: &str = include_str!("../src/assets/assets.rs");
const ASSET_DEFINITION_IMPL: &str = include_str!("../src/assets/definition.rs");
const ASSET_LEAF_IMPL: &str = include_str!("../src/assets/leaf.rs");
const ASSET_NONCE_IMPL: &str = include_str!("../src/assets/nonce.rs");
const ASSET_REGISTRY_IMPL: &str = include_str!("../src/assets/registry.rs");
const ASSET_WIRE_IMPL: &str = include_str!("../src/assets/wire.rs");
const REGISTRY_CATALOG_DOC: &str = include_str!("../src/assets/registry_catalog.rs");
const GENESIS_DOC: &str = include_str!("../src/genesis/mod.rs");
const GENESIS_IMPL: &str = include_str!("../src/genesis/genesis.rs");
const GENESIS_VALIDATOR_IMPL: &str = include_str!("../src/genesis/validator.rs");
const CARGO_TOML: &str = include_str!("../Cargo.toml");
const GENESIS_CAVEATS_DOC: &str = include_str!("../../../wiki/03-core-protocol/genesis-caveats.md");
const STRUCTURE_DOC: &str = include_str!("../../../.planning/codebase/STRUCTURE.md");
const ARCHITECTURE_DOC: &str = include_str!("../../../.planning/codebase/ARCHITECTURE.md");
const PROFILING_DOC: &str = include_str!("../../../.planning/phases/profiling-comprehensive.md");
const ROOT_TYPES_DOC: &str =
    include_str!("../../../crates/z00z_storage/src/settlement/root_types.md");
const SCENARIO11_TODO: &str =
    include_str!("../../../.planning/phases/090-New-Scenarios/90-TODO.md");
const PHASE065_WORDING_AUDIT: &str =
    include_str!("../../../scripts/audit/audit_narrowed_wording.sh");
const UTILS_BOUNDARY_AUDIT: &str =
    include_str!("../../../scripts/audit/audit_z00z_utils_boundary.sh");
const BOUNDARY_WORKFLOW: &str = include_str!("../../../.github/workflows/boundary-guards.yml");

fn manifest_path() -> PathBuf {
    devnet_genesis_path()
}

fn registry_path() -> PathBuf {
    devnet_assets_path()
}

fn assert_absent(label: &str, source: &str, needle: &str) {
    assert!(
        !source.contains(needle),
        "{} must not contain {:?}",
        label,
        needle,
    );
}

fn assert_present(label: &str, source: &str, needle: &str) {
    assert!(source.contains(needle), "{} missing {:?}", label, needle);
}

#[test]
fn test_root_public_paths_compile() -> Result<(), Box<dyn std::error::Error>> {
    let definition = Arc::new(AssetDefinition::new(
        [1u8; 32],
        AssetClass::Coin,
        "Test Coin".into(),
        "TST".into(),
        8,
        10,
        1_000,
        "test.z00z".into(),
        1,
        1,
        0,
        None,
    )?);
    let mut rng = OsRng;
    let blinding = BlindingFactor::random(&mut rng);
    let asset = Asset::new(definition, 0, 1_000, &blinding, [42u8; 32], &mut rng)?;
    let _runner: fn(&str, Option<&str>) -> Result<(), GenesisError> = run_genesis;

    assert_eq!(asset.amount(), 1_000);
    assert_eq!(ChainType::Devnet.as_str(), "devnet");

    Ok(())
}

#[test]
fn registry_doc_examples_stay_live() -> Result<(), Box<dyn std::error::Error>> {
    let registry = AssetDefinitionRegistry::load_catalog_from_yaml(
        &registry_path(),
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    )?;
    assert!(registry.len()? > 0);
    assert!(registry.get_version()? > 0);

    let snapshot = registry.create_snapshot()?;
    assert!(!snapshot.definitions.is_empty());

    let restored = AssetDefinitionRegistry::new(
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );
    restored.update_from_snapshot(snapshot.clone())?;
    assert_eq!(restored.len()?, snapshot.definitions.len());
    assert!(restored.get(&snapshot.definitions[0].id)?.is_some());

    Ok(())
}

#[test]
fn genesis_manifest_docs_stay_live() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_genesis_config(manifest_path().to_str().ok_or("utf8 manifest path")?)?;

    assert!(!config.assets.is_empty());
    assert!(!config.rights.is_empty());
    assert!(!config.policies.is_empty());
    assert!(!config.vouchers.is_empty());
    assert_eq!(GENESIS_RIGHTS_FILE, "genesis_rights.json");
    assert_eq!(GENESIS_POLICIES_FILE, "genesis_policies.json");
    assert_eq!(GENESIS_VOUCHERS_FILE, "genesis_vouchers.json");
    assert_eq!(
        GENESIS_SETTLEMENT_MANIFEST_FILE,
        "genesis_settlement_manifest.json"
    );

    Ok(())
}

#[test]
fn test_doc_pins_live_strings() {
    assert_present("readme", README_DOC, "single canonical bootstrap authority");
    assert_present("readme", README_DOC, "genesis_<SYMBOL>.json");
    assert_present("crate docs", LIB_SRC, "z00z_core::genesis");
    assert_present(
        "assets docs",
        ASSETS_DOC,
        "AssetDefinitionRegistry::load_catalog_from_yaml",
    );
    assert_present("asset impl", ASSET_IMPL, "#[path = \"test_asset.rs\"]");
    assert_present(
        "asset definition impl",
        ASSET_DEFINITION_IMPL,
        "#[path = \"test_definition.rs\"]",
    );
    assert_present(
        "asset leaf impl",
        ASSET_LEAF_IMPL,
        "#[path = \"test_leaf.rs\"]",
    );
    assert_present(
        "asset nonce impl",
        ASSET_NONCE_IMPL,
        "#[path = \"test_nonce.rs\"]",
    );
    assert_present(
        "asset registry impl",
        ASSET_REGISTRY_IMPL,
        "#[path = \"test_registry.rs\"]",
    );
    assert_present(
        "asset wire impl",
        ASSET_WIRE_IMPL,
        "#[path = \"test_wire_compat.rs\"]",
    );
    assert_present("assets docs", ASSETS_DOC, "create_snapshot()");
    assert_present(
        "registry catalog docs",
        REGISTRY_CATALOG_DOC,
        "`decimals`, `serials`, `nominal`, and `domain_name`",
    );
    assert_present("genesis docs", GENESIS_DOC, "manifest_refs");
    assert_present("genesis impl", GENESIS_IMPL, "mod test_genesis;");
    assert_present(
        "genesis validator impl",
        GENESIS_VALIDATOR_IMPL,
        "mod test_validator;",
    );
    assert_present(
        "genesis docs",
        GENESIS_DOC,
        "genesis_settlement_manifest.json",
    );
    assert_present("cargo", CARGO_TOML, "doctest = false");

    assert_absent(
        "crate docs",
        LIB_SRC,
        concat!("state, tx, and ", "validation"),
    );
    assert_absent(
        "crate docs",
        LIB_SRC,
        concat!("z00z_core::genesis::assets_", "generator"),
    );
    assert_absent(
        "assets docs",
        ASSETS_DOC,
        concat!("tests/", "assets/fixtures.rs"),
    );
    assert_absent(
        "asset impl",
        ASSET_IMPL,
        concat!("test_", "asset", "_suite", ".", "rs"),
    );
    assert_absent(
        "asset definition impl",
        ASSET_DEFINITION_IMPL,
        concat!("test_", "definition", "_suite", ".", "rs"),
    );
    assert_absent(
        "asset leaf impl",
        ASSET_LEAF_IMPL,
        concat!("test_", "leaf", "_suite", ".", "rs"),
    );
    assert_absent(
        "asset nonce impl",
        ASSET_NONCE_IMPL,
        concat!("test_", "nonce", "_suite", ".", "rs"),
    );
    assert_absent(
        "asset registry impl",
        ASSET_REGISTRY_IMPL,
        concat!("test_", "registry", "_suite", ".", "rs"),
    );
    assert_absent(
        "asset wire impl",
        ASSET_WIRE_IMPL,
        concat!("test_", "wire_compat", "_suite"),
    );
    assert_absent("assets docs", ASSETS_DOC, concat!("utils_", "traits"));
    assert_absent("assets docs", ASSETS_DOC, "insert_wire");
    assert_absent("assets docs", ASSETS_DOC, "to_snapshot");
    assert_absent("assets docs", ASSETS_DOC, "list_asset_ids");
    assert_absent(
        "registry catalog docs",
        REGISTRY_CATALOG_DOC,
        concat!("max_", "supply"),
    );
    assert_absent(
        "genesis docs",
        GENESIS_DOC,
        concat!("genesis_", "Z00Z.json"),
    );
    assert_absent("genesis docs", GENESIS_DOC, concat!("genesis_", "Z00Z.bin"));
    assert_absent(
        "genesis impl",
        GENESIS_IMPL,
        concat!("test_", "genesis", "_suite"),
    );
    assert_absent(
        "genesis validator impl",
        GENESIS_VALIDATOR_IMPL,
        concat!("test_", "validator", "_suite"),
    );
}

#[test]
fn genesis_caveats_stay_local_and_narrow() {
    assert_present(
        "genesis caveats",
        GENESIS_CAVEATS_DOC,
        "manifest owns genesis thread count",
    );
    assert_present(
        "genesis caveats",
        GENESIS_CAVEATS_DOC,
        "dedicated genesis Rayon pool through `ThreadPoolBuilder`",
    );
    assert_present(
        "genesis caveats",
        GENESIS_CAVEATS_DOC,
        "`crates/z00z_core/src/genesis/genesis_run.rs`",
    );
    assert_absent(
        "genesis caveats",
        GENESIS_CAVEATS_DOC,
        "github.com/vasja34/z00z/blob",
    );
    assert_absent("genesis caveats", GENESIS_CAVEATS_DOC, "OnionNet");
    assert_absent("genesis caveats", GENESIS_CAVEATS_DOC, "remote chain");
}

#[test]
fn phase_064_boundary_audit_keeps_rng_guard_live() {
    assert_present(
        "utils boundary audit",
        UTILS_BOUNDARY_AUDIT,
        "rand::thread_rng\\(",
    );
    assert_present(
        "utils boundary audit",
        UTILS_BOUNDARY_AUDIT,
        "rand::random\\(",
    );
    assert_present(
        "boundary workflow",
        BOUNDARY_WORKFLOW,
        "bash scripts/audit/audit_z00z_utils_boundary.sh",
    );
}

#[test]
fn phase065_wording_guard() {
    assert_present(
        "phase065 wording audit",
        PHASE065_WORDING_AUDIT,
        "crates/z00z_core/src/assets/assets_config.yaml",
    );
    assert_present(
        "phase065 wording audit",
        PHASE065_WORDING_AUDIT,
        "still carries no nullifier semantics",
    );
    assert_present(
        "phase065 wording audit",
        PHASE065_WORDING_AUDIT,
        "V2 memo unsupported",
    );
    assert_absent(
        "structure doc",
        STRUCTURE_DOC,
        "crates/z00z_core/src/assets/assets_config.yaml",
    );
    assert_present(
        "structure doc",
        STRUCTURE_DOC,
        "crates/z00z_core/configs/devnet_assets_config.yaml",
    );
    assert_present(
        "structure doc",
        STRUCTURE_DOC,
        "secondary asset-registry catalog",
    );
    assert_absent(
        "architecture doc",
        ARCHITECTURE_DOC,
        "crates/z00z_core/src/assets/assets_config.yaml",
    );
    assert_present(
        "architecture doc",
        ARCHITECTURE_DOC,
        "crates/z00z_core/configs/devnet_genesis_config.yaml",
    );
    assert_present(
        "architecture doc",
        ARCHITECTURE_DOC,
        "secondary registry data",
    );
    assert_absent(
        "profiling doc",
        PROFILING_DOC,
        "frozen in `assets_config.yaml`",
    );
    assert_present("profiling doc", PROFILING_DOC, "typed genesis manifest");
    assert_absent(
        "root types doc",
        ROOT_TYPES_DOC,
        "canonical regeneration inputs for dev stores",
    );
    assert_present(
        "root types doc",
        ROOT_TYPES_DOC,
        "`crates/z00z_core/configs/devnet_genesis_config.yaml`",
    );
    assert_present(
        "root types doc",
        ROOT_TYPES_DOC,
        "registry/example data only",
    );
    assert_absent(
        "phase 066 todo",
        SCENARIO11_TODO,
        "crates/z00z_core/src/assets/assets_config.yaml",
    );
    assert_present(
        "phase 066 todo",
        SCENARIO11_TODO,
        "crates/z00z_core/configs/devnet_rights_config.yaml",
    );
}
