//! Core app kernel and entity.
//!
//! This layer exists to keep a consistent architecture:
//! RPC → Service → Core.
//!
//! Phase 1: `Z00ZApp` is a data container only, and intentionally contains no secrets.

use crate::{ChainType, WalletError, WalletResult};

const MAX_WALLET_NAME_LEN: usize = 64;

/// A core-level wallet creation request.
///
/// This type is intentionally non-secret. Sensitive inputs like passwords and seed phrases
/// must not be stored in the core app container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateWalletRequest {
    /// User-provided wallet name.
    pub name: String,

    /// Wallet network identifier (e.g., `p2p`).
    pub network: String,

    /// Wallet chain identifier (e.g., `devnet`).
    pub chain: String,
}

impl CreateWalletRequest {
    /// Create a validated wallet creation request.
    ///
    /// Meaning:
    /// - Represents a core-level, non-secret command produced by the core layer.
    /// - Carries canonicalized inputs (trimmed) that the service layer must use.
    /// - Has no side effects; persistence and secret handling are performed by the service layer.
    pub fn new(name: &str, network: &str, chain: &str) -> WalletResult<Self> {
        let name = name.trim();
        let network = network.trim();
        let chain = chain.trim();

        if name.is_empty() {
            return Err(WalletError::InvalidParams(
                "Wallet name cannot be empty".to_string(),
            ));
        }

        if network.is_empty() {
            return Err(WalletError::InvalidParams(
                "Wallet network cannot be empty".to_string(),
            ));
        }

        if chain.is_empty() {
            return Err(WalletError::InvalidParams(
                "Wallet chain cannot be empty".to_string(),
            ));
        }

        if name.len() > MAX_WALLET_NAME_LEN {
            return Err(WalletError::InvalidParams(format!(
                "Wallet name is too long (max {MAX_WALLET_NAME_LEN} characters)"
            )));
        }

        // Keep these bounds conservative; they are persisted and used in identity validation.
        const MAX_WALLET_NETWORK_LEN: usize = 32;
        const MAX_WALLET_CHAIN_LEN: usize = 32;

        if network.len() > MAX_WALLET_NETWORK_LEN {
            return Err(WalletError::InvalidParams(format!(
                "Wallet network is too long (max {MAX_WALLET_NETWORK_LEN} characters)"
            )));
        }

        if chain.len() > MAX_WALLET_CHAIN_LEN {
            return Err(WalletError::InvalidParams(format!(
                "Wallet chain is too long (max {MAX_WALLET_CHAIN_LEN} characters)"
            )));
        }

        Ok(Self {
            name: name.to_string(),
            network: network.to_string(),
            chain: chain.to_string(),
        })
    }
}

/// App-kernel state for app-owned control selection.
///
/// This type is intentionally app-scoped (not wallet-scoped).
#[derive(Debug, Default, Clone)]
pub struct AppKernel;

impl AppKernel {
    /// Create a new core app kernel.
    pub fn new() -> Self {
        Self
    }

    /// Select mainnet as the active app-level chain.
    pub fn switch_to_mainnet(&self) -> ChainType {
        ChainType::Mainnet
    }

    /// Select testnet as the active app-level chain.
    pub fn switch_to_testnet(&self) -> ChainType {
        ChainType::Testnet
    }

    /// Select devnet as the active app-level chain.
    pub fn switch_to_devnet(&self) -> ChainType {
        ChainType::Devnet
    }

    /// Resolve the current chain selection used when OnionNet is requested.
    ///
    /// OnionNet transport is not represented by `ChainType`. Until transport
    /// selection has its own runtime representation, this returns the current
    /// local fallback chain selection.
    pub fn configure_onionet(&self) -> ChainType {
        ChainType::Devnet
    }

    /// Apply the app-level Tor enablement preference.
    ///
    /// Transport execution lives in the dedicated network services; this
    /// control hook preserves the requested local preference.
    pub fn configure_tor(&self, enable: bool) -> bool {
        enable
    }

    /// App-layer control hook for wallet-local scan start.
    pub fn start_local_scan(&self) -> bool {
        true
    }

    /// App-layer control hook for wallet-local scan stop.
    pub fn stop_local_scan(&self) -> bool {
        true
    }

    /// App-layer control hook for wallet-local scan status reads.
    pub fn get_local_scan_status(&self) -> bool {
        true
    }

    /// App-layer control hook for wallet-local chain-tip reads.
    pub fn get_local_scan_tip_height(&self) -> u64 {
        0
    }

    /// App-layer control hook for wallet listing.
    pub fn list_wallets(&self) -> bool {
        true
    }

    /// Create a wallet request.
    ///
    /// The core layer does not persist wallets and does not handle secrets.
    /// Persistence is orchestrated by the service layer.
    pub fn create_wallet(
        &self,
        name: &str,
        network: &str,
        chain: &str,
    ) -> WalletResult<CreateWalletRequest> {
        CreateWalletRequest::new(name, network, chain)
    }

    /// App-layer control hook for wallet deletion.
    pub fn delete_wallet(&self) -> bool {
        true
    }

    /// App-layer control hook for wallet export.
    pub fn export_wallet(&self) -> bool {
        true
    }

    /// App-layer control hook for wallet import.
    pub fn import_wallet(&self) -> bool {
        true
    }
}

/// Core app entity.
///
/// Phase 1: this is a data container that owns app-level infrastructure and shared utilities.
/// It MUST NOT store wallet secrets.
#[derive(Debug, Clone)]
pub struct Z00ZApp<App, Network, Chain, Time, Rng> {
    /// App-level kernel/state.
    pub app: App,
    /// App-owned network client/config.
    pub network_client: Network,
    /// App-owned chain client.
    pub chain_client: Chain,
    /// App-owned time provider.
    pub time_provider: Time,
    /// App-owned RNG provider.
    pub rng_provider: Rng,
}

impl<App, Network, Chain, Time, Rng> Z00ZApp<App, Network, Chain, Time, Rng> {
    /// Create a new core app container.
    pub fn new(
        app: App,
        network_client: Network,
        chain_client: Chain,
        time_provider: Time,
        rng_provider: Rng,
    ) -> Self {
        Self {
            app,
            network_client,
            chain_client,
            time_provider,
            rng_provider,
        }
    }
}

impl<Network, Chain, Time, Rng> Z00ZApp<AppKernel, Network, Chain, Time, Rng> {
    /// Select mainnet as the active app-level chain.
    pub fn switch_to_mainnet(&self) -> ChainType {
        self.app.switch_to_mainnet()
    }

    /// Select testnet as the active app-level chain.
    pub fn switch_to_testnet(&self) -> ChainType {
        self.app.switch_to_testnet()
    }

    /// Select devnet as the active app-level chain.
    pub fn switch_to_devnet(&self) -> ChainType {
        self.app.switch_to_devnet()
    }

    /// Resolve the current chain selection used when OnionNet is requested.
    pub fn configure_onionet(&self) -> ChainType {
        self.app.configure_onionet()
    }

    /// Apply the app-level Tor enablement preference.
    pub fn configure_tor(&self, enable: bool) -> bool {
        self.app.configure_tor(enable)
    }

    /// App-layer control hook for wallet-local scan start.
    pub fn start_local_scan(&self) -> bool {
        self.app.start_local_scan()
    }

    /// App-layer control hook for wallet-local scan stop.
    pub fn stop_local_scan(&self) -> bool {
        self.app.stop_local_scan()
    }

    /// App-layer control hook for wallet-local scan status reads.
    pub fn get_local_scan_status(&self) -> bool {
        self.app.get_local_scan_status()
    }

    /// App-layer control hook for wallet-local chain-tip reads.
    pub fn get_local_scan_tip_height(&self) -> u64 {
        self.app.get_local_scan_tip_height()
    }

    /// App-layer control hook for wallet listing.
    pub fn list_wallets(&self) -> bool {
        self.app.list_wallets()
    }

    /// Create a wallet request.
    pub fn create_wallet(
        &self,
        name: &str,
        network: &str,
        chain: &str,
    ) -> WalletResult<CreateWalletRequest> {
        self.app.create_wallet(name, network, chain)
    }

    /// App-layer control hook for wallet deletion.
    pub fn delete_wallet(&self) -> bool {
        self.app.delete_wallet()
    }

    /// App-layer control hook for wallet export.
    pub fn export_wallet(&self) -> bool {
        self.app.export_wallet()
    }

    /// App-layer control hook for wallet import.
    pub fn import_wallet(&self) -> bool {
        self.app.import_wallet()
    }
}
