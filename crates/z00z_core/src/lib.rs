//! # Z00Z Core Protocol
//!
//! Live protocol crate for assets, canonical genesis bootstrap, policies,
//! rights, and vouchers.
//!
//! The crate root intentionally stays small:
//! - common runtime types are re-exported from the root
//! - owner-specific helpers stay under `assets`, `genesis`, `actions`,
//!   `policies`, `rights`, and `vouchers`
//! - bootstrap authority stays under `z00z_core::genesis`
//!
//! ## Live Public Modules
//!
//! - `actions`
//! - `assets`
//! - `domains`
//! - `genesis`
//! - `hashing`
//! - `policies`
//! - `rights`
//! - `vouchers`
//!
//! ## Root Facade Example
//!
//! ```rust,ignore
//! use rand::rngs::OsRng;
//! use std::sync::Arc;
//! use z00z_core::{Asset, AssetClass, AssetDefinition, BlindingFactor};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let definition = Arc::new(AssetDefinition::new(
//!     [0u8; 32],
//!     AssetClass::Coin,
//!     "Test Coin".into(),
//!     "TST".into(),
//!     8,
//!     1_000,
//!     100_000_000,
//!     "test.z00z".into(),
//!     1,
//!     1,
//!     0,
//!     None,
//! )?);
//!
//! let mut rng = OsRng;
//! let blinding = BlindingFactor::random(&mut rng);
//! let asset = Asset::new(
//!     definition,
//!     0,
//!     100_000_000,
//!     &blinding,
//!     [42u8; 32],
//!     &mut rng,
//! )?;
//!
//! assert_eq!(asset.amount(), 100_000_000);
//! # Ok(())
//! # }
//! ```
//!
//! ## Bootstrap Example
//!
//! ```rust,ignore
//! use z00z_core::genesis::genesis_config::load_genesis_config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = load_genesis_config("configs/devnet_genesis_config.yaml")?;
//! assert!(!config.assets.is_empty());
//! # Ok(())
//! # }
//! ```
//!
//! The longer crate overview lives in `README.md`, which is embedded below as
//! part of the public crate docs.

#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
#![warn(rust_2018_idioms, unused_qualifications, unreachable_pub)]
#![recursion_limit = "2048"]

#[allow(unexpected_cfgs)]
#[cfg(kani)]
extern crate self as z00z_core;

#[allow(unexpected_cfgs)]
#[cfg(kani)]
#[path = "../tests/generated_kani_asset_pkg_json.rs"]
mod generated_kani_asset_pkg_json;

pub mod actions;
pub mod assets;
mod config_name;
pub mod config_paths;
pub mod domains;
pub mod genesis;
pub mod hashing;
pub mod policies;
pub mod rights;
pub mod vouchers;

// Curated stable root facade: common runtime contracts stay at the crate root,
// while tooling, config loading, and generation helpers remain under their
// owning modules.
pub use actions::{ActionDescriptorV1, ActionId, ActionPoolDescriptorV1, ActionPoolId};
pub use assets::{
    Asset, AssetClass, AssetDefinition, AssetDefinitionRegistry, AssetError, AssetLeaf,
    AssetMetadata, AssetPkgWire, AssetWire, BlindingFactor, Commitment, ObjectFamily, ObjectRoleV1,
};
pub use genesis::ChainType;
pub use policies::{
    native_cash_policy_descriptor, validate_native_cash_policy_descriptor, ConditionDescriptorV1,
    ConditionKindV1, ConditionTrustTierV1, PolicyConfigEntryV1, PolicyDescriptorV1, PolicyId,
    PolicyTemplateV1,
};
pub use rights::{
    RightActionV1, RightClassConfig, RightPolicyV1, RightRequirementV1, RightScopeV1,
    RightsConfigEntry,
};
pub use vouchers::{
    VoucherAcceptanceTermsV1, VoucherBackingReferenceV1, VoucherBootstrapEntryV1,
    VoucherConfigEntry, VoucherLifecycleV1, VoucherPolicyV1, VoucherValidityWindowV1,
};
