#![allow(clippy::module_inception)]
//! # Genesis Module
//!
//! Canonical bootstrap orchestration for the live Z00Z object model.
//!
//! `z00z_core::genesis` is the single authority boundary that rehydrates
//! `GenesisConfig`, validates the manifest, and derives deterministic bootstrap
//! artifacts for assets, rights, policies, and vouchers.
//!
//! ## Live Public Surface
//!
//! - [`genesis_config`] owns `GenesisConfig` loading
//! - [`run_genesis`] executes the full bootstrap flow
//! - [`run_genesis_with_plan`] executes explicit lane selections under the same
//!   `GenesisConfig` authority
//! - [`load_validate_install_chain_identity`] strictly loads, reproduces, and
//!   pins the process genesis identity
//! - [`validator`] enforces fail-closed config and proof validation
//! - manifest and digest helpers stay under the same owner path
//!
//! Secondary YAML surfaces may provide registry data or manifest subfiles, but
//! they must rehydrate into one `GenesisConfig` before validation or
//! generation.
//!
//! ## Manifest Loading
//!
//! ```rust,ignore
//! use z00z_core::genesis::genesis_config::load_genesis_config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = load_genesis_config("configs/devnet_genesis_config.yaml")?;
//! assert!(!config.assets.is_empty());
//! assert!(!config.rights.is_empty());
//! # Ok(())
//! # }
//! ```
//!
//! ## Full Bootstrap
//!
//! ```rust,ignore
//! use z00z_core::genesis::run_genesis;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! run_genesis("configs/devnet_genesis_config.yaml", None)?;
//! # Ok(())
//! # }
//! ```
//!
//! `run_genesis()` produces a timestamped export directory rooted at
//! `GenesisConfig.outputs.assets_export_path`, per-asset JSON/Bincode exports
//! such as `genesis_<SYMBOL>.json` and `genesis_<SYMBOL>.bin`, and the typed
//! bootstrap artifacts:
//!
//! - `genesis_rights.json`
//! - `genesis_policies.json`
//! - `genesis_vouchers.json`
//! - `genesis_settlement_manifest.json`
//!
//! Snapshot ZIP exports remain rooted at
//! `GenesisConfig.outputs.snapshot_export_path`.
//!
//! Partial lane plans keep the same config authority but emit
//! `genesis_generation_receipt.json` instead of the canonical full settlement
//! manifest.
//!
//! ## Determinism and Boundaries
//!
//! Deterministic derivation is separated by chain identity, object family,
//! object-local ids, serial indices, and root-generation labels. Split manifest
//! files are allowed via `manifest_refs`, but split authority is not.
//!
//! For the phase-level semantics matrix and operator notes, see
//! `src/genesis/README.md`.

pub mod asset_std;
mod chain_identity;
pub mod genesis_config;
mod manifest_ref_loader;
pub mod serde;
pub mod validator;

pub use chain_identity::{
    load_validate_install_chain_identity, require_process_chain_identity,
    GenesisChainIdentityError, GenesisChainIdentityV2,
};
pub use validator::validate_genesis_config_for;

#[path = "genesis.rs"]
mod generation;
pub(crate) use generation::load_genesis_settlement_manifest;

// Curated facade for the generation submodule. Higher-level config/file entry
// points stay under their owning modules instead of flowing through wildcard
// exports.
pub use generation::{
    compute_genesis_manifest_hash, compute_genesis_policies_digest, compute_genesis_rights_digest,
    compute_genesis_seed_hash, compute_genesis_vouchers_digest, create_asset_definition,
    derive_deterministic_rng_seed, derive_genesis_blinding, ensure_terminal_collision_free,
    export_genesis_settlement_artifacts, generate_all_genesis_assets, generate_genesis_lanes,
    generate_genesis_policies, generate_genesis_settlement_corpus, generate_genesis_vouchers,
    load_genesis_context, resolve_genesis_context, run_genesis, run_genesis_with_plan, ChainType,
    GenesisAssetAccumulator, GenesisExportKind, GenesisGenerationPlan, GenesisGenerationReceipt,
    GenesisLane, GenesisLaneOutputs, GenesisPolicyRecord, GenesisResolvedContext, GenesisRightLeaf,
    GenesisRightRecord, GenesisSeed, GenesisSelection, GenesisSettlementCorpus,
    GenesisSettlementManifest, GenesisVoucherRecord, TerminalCollisionReport,
    GENESIS_GENERATION_RECEIPT_FILE, GENESIS_POLICIES_FILE, GENESIS_POLICIES_REPLAY_DIGEST_LABEL,
    GENESIS_POLICIES_ROUNDTRIP_DIGEST_LABEL, GENESIS_RIGHTS_FILE,
    GENESIS_RIGHTS_REPLAY_DIGEST_LABEL, GENESIS_RIGHTS_ROUNDTRIP_DIGEST_LABEL,
    GENESIS_ROOT_GENERATION, GENESIS_SETTLEMENT_MANIFEST_FILE, GENESIS_VOUCHERS_FILE,
    GENESIS_VOUCHERS_REPLAY_DIGEST_LABEL, GENESIS_VOUCHERS_ROUNDTRIP_DIGEST_LABEL,
};
