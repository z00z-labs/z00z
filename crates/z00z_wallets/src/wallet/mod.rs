//! Wallet components and core entity.
//!
//! This module contains the Z00ZWallet core entity, system components
//! (backup/restore, policy), and supporting types (rollback, test-only stub defaults).

// Core entity
#[path = "core.rs"]
pub mod core;

// Wallet state machine and policy
pub mod auto_lock;
pub mod wallet_state;

// Core wallet types
pub mod errors;

// Persistence
pub mod persistence;

// Session management
pub mod session;

// System components
pub mod policy;

// Supporting types
pub mod rollback;
#[cfg(test)]
#[path = "stub_defaults.rs"]
pub mod stub_defaults;

// Re-export core entity
pub use core::{
    ChainId, WalletId, WalletKernel, WalletRecord, WalletSystemMetadata, WalletUserFields,
    Z00ZWallet,
};

// Re-export wallet state/policy
pub use auto_lock::{AutoLockPolicy, LockTrigger};
pub use wallet_state::WalletState;

// Re-export wallet core types
pub use errors::{IsTransient, StateTransitionError, WalletError, WalletPublicError, WalletResult};

// Internal diagnostic codes (not part of the public API).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use errors::WalletErrorStage;

// Re-export session management.
pub use session::{SecretSession, SessionHandle};

// Re-export system components
pub use policy::{
    Policy, PolicyError, PolicyImpl, PolicyResult, PolicyRules, PolicySpendContext,
    TimeRestrictions,
};

#[cfg(all(test, not(target_arch = "wasm32")))]
mod api_contract_tests {
    use crate::security::encryption::WalletEncryption;
    use crate::security::SecretBytes;
    use crate::wallet::errors::WalletResult;

    #[test]
    fn test_decrypt_entrypoints_secret_bytes() {
        fn test_assert_decrypt_sig(
            _: fn(
                &z00z_crypto::expert::encoding::SafePassword,
                &[u8],
                &crate::security::encryption::EncryptedWalletContainer,
            ) -> WalletResult<SecretBytes>,
        ) {
        }

        test_assert_decrypt_sig(WalletEncryption::decrypt_wallet);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_wlt_profile_reads_secret() {
        use crate::db::wallet_store::{RedbWalletStore, WltStore};
        use crate::db::WalletSession;

        fn assert_read_sig(
            _: for<'a, 'b> fn(&'a RedbWalletStore, &'b WalletSession) -> WalletResult<SecretBytes>,
        ) {
        }

        assert_read_sig(<RedbWalletStore as WltStore>::read_wallet_profile);
    }
}
