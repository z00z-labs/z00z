//! Z00Z Wallets - Asset-based HD Wallet Implementation
//!
//! Privacy-focused hierarchical deterministic wallet for Z00Z blockchain.
//!
//! ## Architecture
//!
//! Following Z00Z Design Foundation principles:
//! - **ONE SOURCE OF TRUTH** - Uses z00z_utils abstractions for I/O, time, config
//! - **TRAIT-BASED DEPENDENCY INJECTION** - All components injectable via traits
//! - **NO ACCOUNTS** - Bitcoin-style Asset model with HD keys (m/44'/0'/chain/index)
//! - **VENDOR ISOLATION** - No modifications to tari-wallet (read-only reference)
//!
//! ## Module Structure
//!
//! ```text
//! z00z_wallets/
//! ├── wallet/                  # 🧩 Wallet entity + state + policy
//! ├── services/                # 🧩 Orchestration boundary with local wallet authority
//! ├── rpc/                     # 🧩 RPC boundary over local wallet and chain state
//! └── egui_views/              # 🧩 Desktop UI (feature-gated)
//! ```
//!
//! ## Implemented Boundaries
//!
//! | Area | Status | Notes |
//! |---|---|---|
//! | `wallet` | Live local authority | Wallet identity, state, encryption, persisted state, and auto-lock policy |
//! | `key` | Live local authority | Deterministic BIP-44 paths and wallet-owned key operations |
//! | `tx` | Live local authority | Proof generation plus local transaction package construction |
//! | `chain` | Partial | Deterministic local simulation is live; real remote-node transport remains adapter-only |
//! | `services` | Live local authority | Local orchestration, capability gating, and persistence-backed wallet flows |
//! | `rpc` | Live local authority | JSON-RPC routes expose current local wallet, storage, and chain truth |
//!
//! ## Roadmap
//!
//! - Phase 2 implementation work should start at service boundaries (`services/*`) and then fill in
//!   missing core behavior behind trait interfaces.
//! - Architectural constraints and layering rules are documented in `crates/Z00Z_DESIGN_FOUNDATION.md`.
//! - Callers should prefer the shallow wallet entrypoints: `db`, `services`, `receiver`,
//!   `key`, and `tx`.
//! - Transport DTOs and dispatcher registration stay under `rpc` and
//!   `rpc::types`; they are not part of the stable crate-root facade.
//! - The `services` facade is the canonical orchestration entrypoint. Encryption helpers stay
//!   under `security::encryption`.
//!
//! ## Examples
//!
//! ```no_run
//! # use z00z_wallets::key::{Bip44Path, KeyManager, KeyManagerImpl};
//! # use z00z_crypto::Hidden;
//! # use z00z_core::genesis::ChainType;
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut key_manager = KeyManagerImpl::new();
//!
//! // Initialize from raw seed
//! let seed = vec![42u8; 64];
//! key_manager.init_from_seed(&seed, ChainType::Devnet)?;
//!
//! // Derive payment key
//! let payment_key = key_manager.derive_payment_key(0)?;
//!
//! // Derive change key
//! let change_key = key_manager.derive_change_key(0)?;
//!
//! // Original: Derive using BIP-44 path
//! let receiving_key = key_manager.derive_key(&Bip44Path::payment(0)?);
//!
//! // Verify key
//! let is_valid = key_manager.verify_key(&payment_key);
//!
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod config;

#[cfg(all(
    feature = "ownership_policy_keyring",
    feature = "ownership_policy_challenge",
    not(feature = "ownership_policy_dual_ok")
))]
compile_error!(
    "Enable exactly one: `ownership_policy_keyring` OR `ownership_policy_challenge`. \
     To allow both simultaneously, also enable `ownership_policy_dual_ok`."
);

#[cfg(not(any(
    feature = "ownership_policy_keyring",
    feature = "ownership_policy_challenge"
)))]
compile_error!("Enable exactly one: ownership_policy_keyring OR ownership_policy_challenge");

#[cfg(all(not(test), not(debug_assertions), feature = "test-params-fast"))]
compile_error!("`test-params-fast` MUST NOT be compiled into release-capable z00z_wallets builds");

#[cfg(all(not(test), not(debug_assertions), feature = "wallet_debug_tools"))]
compile_error!(
    "`wallet_debug_tools` MUST NOT be compiled into release-capable z00z_wallets builds"
);

// ✅ Wallet runtime modules
pub mod app;
pub mod backup;
pub mod chain;
pub mod claim;
pub mod domains;
pub mod key;
pub mod network;
pub mod persistence;
pub mod receiver;
pub mod rpc;
pub mod security;
pub mod stealth;
pub mod tx;
pub mod wallet;

// ✅ Services layer - Business logic
#[cfg(not(target_arch = "wasm32"))]
pub mod services;

// Portable `.wlt` contract surface (wasm32-friendly)
pub mod wasm;

/// Stable sender-output surface for wallet and app code.
/// Public sender construction now lives under `stealth`, not `tx`.
pub use stealth::{
    bind_stealth_output_wire, build_card_stealth_leaf, build_card_stealth_output_validated,
    build_output_bundle, build_output_bundle_with_rng, build_stealth_leaf,
    build_stealth_leaf_with_blind, build_stealth_leaf_with_rng, build_tx_output_unchecked,
    build_tx_stealth_output_validated, validate_output_self, BuildCheck, OwnerTag,
    OwnerTagOperations, SenderValidationCtx, SenderWallet, StealthError, TagMode, TxStealthOutput,
};

// Native-only wallet persistence backends (RedB `.wlt`)
#[cfg(not(target_arch = "wasm32"))]
pub mod db;

/// Internal-only native debug helpers.
///
/// These helpers are intentionally absent from release-capable builds and are
/// not part of the stable public wallet facade.
#[cfg(all(
    feature = "wallet_debug_tools",
    debug_assertions,
    not(target_arch = "wasm32")
))]
pub mod internal_debug_tools {
    use std::path::Path;

    use z00z_crypto::expert::encoding::SafePassword;

    use crate::{db::WalletIdentity, rpc::types::common::PersistWalletId, WalletResult};

    /// Debug-only wallet export helper for local inspection flows.
    pub fn debug_export_wallet(
        wlt_path: &Path,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
        identity: &WalletIdentity,
        out_path: &Path,
    ) -> WalletResult<()> {
        crate::db::redb_store::debug_export_wallet(
            wlt_path, wallet_id, password, identity, out_path,
        )
    }
}

// Desktop EGUI views (feature-gated)
#[cfg(feature = "egui")]
pub mod egui_views;

// Re-export shallow caller-visible root types.
pub use wallet::{WalletError, WalletPublicError, WalletResult};
pub use z00z_core::ChainType;

#[cfg(not(target_arch = "wasm32"))]
pub use services::{AppService, WalletService};

// Re-export receiver types
pub use receiver::{
    PaymentRequest, PaymentRequestError, ReceiveNext, ReceiveReject, ReceiveReport, ReceiveStatus,
    ReceiverCard, ReceiverCardError, RequestParams, ScanResult, ValidityStatus, WalletReveal,
    WalletStealthOutput,
};

// Re-export the public transaction-wire and verification surface used by E2E callers.
pub use tx::{
    asset_wire_to_leaf, build_tx_package_digest, verify_tx_public_spend_contract,
    wire_decrypt_leaf, TxOutRole, TxOutputWire, TxPackage, TxVerifier, TxVerifierImpl,
};
