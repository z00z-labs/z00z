#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::sync::Arc;

use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::{rng::SystemRngProvider, time::TimeProvider};

use crate::db::wallet_io;
use crate::db::{
    create_wlt_with_deps, discover_wlt_with_deps, open_wlt_with_deps, read_wallet_profile,
    reveal_seed_phrase, verify_password_for_session, write_wallet_profile, WalletSession,
};
use crate::rpc::types::common::PersistWalletId;
use crate::rpc::types::wallet::PersistWalletDiscovery;
use crate::security::SecretBytes;
use crate::{WalletError, WalletResult};

/// Filesystem I/O boundary for `.wlt` persistence.
///
/// This centralizes wallet-owned path operations, atomic writes, and permission
/// changes behind one boundary while keeping native tmpfs and RedB file-handle
/// steps as internal implementation details of the create and open flows.
///
/// Tests can still inject a tracking or failing boundary for the wallet-owned
/// path operations without claiming that every native file handle is abstracted.
pub(crate) trait WalletIo: Send + Sync {
    fn create_dir_all(&self, path: &Path) -> WalletResult<()>;
    fn path_exists(&self, path: &Path) -> WalletResult<bool>;
    fn read_file(&self, path: &Path) -> WalletResult<Vec<u8>>;
    fn atomic_write_file_streaming(
        &self,
        path: &Path,
        write_fn: &mut dyn FnMut(&mut std::fs::File) -> Result<(), std::io::Error>,
    ) -> WalletResult<()>;
    fn remove_file_best_effort(&self, path: &Path);
    fn set_private_file_permissions(&self, path: &Path) -> WalletResult<()>;
}

/// Wallet identity (network + chain) persisted in `.wlt` meta and validated on open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletIdentity {
    /// Runtime-selected wallet network identifier (e.g., `p2p`).
    pub network: String,
    /// Runtime-selected chain identifier (e.g., `devnet`).
    pub chain: String,
}

/// Default `.wlt` I/O implementation backed by `z00z_utils::io` via `wallet_io`
/// for the wallet-owned path boundary.
#[derive(Debug, Default)]
pub(crate) struct Z00ZWalletIo;

impl WalletIo for Z00ZWalletIo {
    fn create_dir_all(&self, path: &Path) -> WalletResult<()> {
        wallet_io::create_dir_all(path)
    }

    fn path_exists(&self, path: &Path) -> WalletResult<bool> {
        wallet_io::path_exists(path)
    }

    fn read_file(&self, path: &Path) -> WalletResult<Vec<u8>> {
        wallet_io::read_file(path)
    }

    fn atomic_write_file_streaming(
        &self,
        path: &Path,
        write_fn: &mut dyn FnMut(&mut std::fs::File) -> Result<(), std::io::Error>,
    ) -> WalletResult<()> {
        z00z_utils::io::atomic_write_file_streaming(path, |file| {
            write_fn(file).map_err(|e| z00z_utils::io::IoError::Io(std::io::Error::other(e)))
        })
        .map_err(|e| WalletError::InvalidConfig(format!("atomic write failed: {e}")))
    }

    fn remove_file_best_effort(&self, path: &Path) {
        wallet_io::remove_file_best_effort(path)
    }

    fn set_private_file_permissions(&self, path: &Path) -> WalletResult<()> {
        wallet_io::set_private_file_permissions(path)
    }
}

/// Object-safe interface for `.wlt` persistence.
///
/// 📌 Purpose:
/// - Keep storage operations behind a stable boundary.
/// - Make it straightforward to swap backends or inject deterministic providers in tests.
/// - Prevent higher layers (Service/RPC) from depending on RedB internals.
pub(crate) trait WltStore: Send + Sync {
    fn create_wallet_store(
        &self,
        path: &Path,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
        seed_phrase: &str,
        identity: &WalletIdentity,
    ) -> WalletResult<()>;

    fn open_wallet_store(
        &self,
        path: &Path,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
        identity: &WalletIdentity,
    ) -> WalletResult<WalletSession>;

    fn discover_wallet_store(&self, path: &Path) -> WalletResult<PersistWalletDiscovery>;

    /// Reveal the main seed phrase.
    ///
    /// Policy note: this method does not enforce a persistent "show once" restriction.
    fn reveal_seed_phrase(&self, session: &WalletSession) -> WalletResult<String>;

    /// Verify the wallet password against the active session.
    fn verify_password(&self, session: &WalletSession, password: &SafePassword)
        -> WalletResult<()>;

    fn write_wallet_profile(
        &self,
        session: &WalletSession,
        profile_bytes: Vec<u8>,
    ) -> WalletResult<u64>;

    fn read_wallet_profile(&self, session: &WalletSession) -> WalletResult<SecretBytes>;
}

/// RedB-backed `.wlt` store.
pub(crate) struct RedbWalletStore {
    time_provider: Arc<dyn TimeProvider>,
    io: Arc<dyn WalletIo>,
}

impl RedbWalletStore {
    pub(crate) fn new(time_provider: Arc<dyn TimeProvider>, io: Arc<dyn WalletIo>) -> Self {
        Self { time_provider, io }
    }
}

impl WltStore for RedbWalletStore {
    fn create_wallet_store(
        &self,
        path: &Path,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
        seed_phrase: &str,
        identity: &WalletIdentity,
    ) -> WalletResult<()> {
        create_wlt_with_deps(
            path,
            wallet_id,
            password,
            seed_phrase,
            identity,
            SystemRngProvider,
            self.time_provider.as_ref(),
            self.io.clone(),
        )
    }

    fn open_wallet_store(
        &self,
        path: &Path,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
        identity: &WalletIdentity,
    ) -> WalletResult<WalletSession> {
        open_wlt_with_deps(
            path,
            wallet_id,
            password,
            identity,
            self.time_provider.clone(),
            self.io.clone(),
        )
    }

    fn discover_wallet_store(&self, path: &Path) -> WalletResult<PersistWalletDiscovery> {
        discover_wlt_with_deps(path, self.time_provider.as_ref(), self.io.clone())
    }

    fn reveal_seed_phrase(&self, session: &WalletSession) -> WalletResult<String> {
        reveal_seed_phrase(session, SystemRngProvider, self.time_provider.as_ref())
    }

    fn verify_password(
        &self,
        session: &WalletSession,
        password: &SafePassword,
    ) -> WalletResult<()> {
        verify_password_for_session(session, password)
    }

    fn write_wallet_profile(
        &self,
        session: &WalletSession,
        profile_bytes: Vec<u8>,
    ) -> WalletResult<u64> {
        write_wallet_profile(session, profile_bytes, SystemRngProvider)
    }

    fn read_wallet_profile(&self, session: &WalletSession) -> WalletResult<SecretBytes> {
        read_wallet_profile(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WalletError;
    use tempfile::TempDir;
    use z00z_utils::time::MockTimeProvider;

    #[test]
    fn test_wlt_store_std_time() {
        let src = include_str!("wallet_store.rs");
        let needle = ["use std", "::time"].concat();
        assert!(
            !src.contains(&needle),
            "wallet persistence boundary should not import std::time directly"
        );
    }

    #[test]
    fn test_wlt_open_rejects_passwords() {
        let dir = TempDir::new().unwrap();
        let wlt_path = dir.path().join("wallet.wlt");

        let wallet_id = PersistWalletId("wlt_open_test".to_string());
        let identity = WalletIdentity {
            network: "p2p".to_string(),
            chain: "devnet".to_string(),
        };

        let time_provider: Arc<dyn TimeProvider> = Arc::new(MockTimeProvider::default());
        let io: Arc<dyn WalletIo> = Arc::new(Z00ZWalletIo);
        let store = RedbWalletStore::new(Arc::clone(&time_provider), io);

        let correct = SafePassword::from("StrongPassw0rd!");
        let wrong = SafePassword::from("WrongPassw0rd!");
        let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

        store
            .create_wallet_store(&wlt_path, &wallet_id, &correct, seed_phrase, &identity)
            .unwrap();

        // Correct password must open.
        {
            let _session = store
                .open_wallet_store(&wlt_path, &wallet_id, &correct, &identity)
                .unwrap();
        }

        // Wrong password must be a bounded auth failure.
        let err = store
            .open_wallet_store(&wlt_path, &wallet_id, &wrong, &identity)
            .unwrap_err();
        assert!(matches!(err, WalletError::InvalidPassword));
    }

    #[test]
    fn test_wlt_create_restores_missing_parent_before_lock() {
        let dir = TempDir::new().unwrap();
        let wlt_path = dir.path().join("nested/wallets/wallet_missing_parent.wlt");

        let wallet_id = PersistWalletId("wlt_missing_parent".to_string());
        let identity = WalletIdentity {
            network: "p2p".to_string(),
            chain: "devnet".to_string(),
        };

        let time_provider: Arc<dyn TimeProvider> = Arc::new(MockTimeProvider::default());
        let io: Arc<dyn WalletIo> = Arc::new(Z00ZWalletIo);
        let store = RedbWalletStore::new(Arc::clone(&time_provider), io);

        let password = SafePassword::from("StrongPassw0rd!");
        let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

        store
            .create_wallet_store(&wlt_path, &wallet_id, &password, seed_phrase, &identity)
            .unwrap();

        assert!(wlt_path.exists(), "wallet file should be created");
    }
}
