//! Asset RPC implementations backed by `WalletService`.

#[cfg(test)]
use super::asset_rpc_caches::AssetListCacheValue;
use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use tokio::sync::RwLock;
use z00z_core::assets::registry::AssetId;
use z00z_core::assets::wire::DefinitionWire;
#[cfg(test)]
use z00z_core::assets::AssetClass;
use z00z_core::assets::{decode_asset_pkg_json, payload_has_secret_field, AssetError, AssetWire};
use z00z_core::Asset;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    logger::{Logger, TracingLogger},
    rng::{RngCoreExt, SystemRngProvider},
    time::{SystemTimeProvider, TimeProvider},
};

use super::asset_rpc_rate_limits::ASSET_SEND_RATE_LIMIT_WINDOW;
use super::{
    asset_quarantine::{self, AssetQuarantineStore},
    asset_rpc::AssetRpcServer,
    asset_rpc_balance,
    asset_rpc_caches::{self, AssetListCache, AssetMetadataCache},
    asset_rpc_rate_limits::{self, AssetSendRateLimitMap},
    asset_rpc_registry,
    asset_rpc_stakes::{self, AssetStakeCounter},
    ownership_check::{
        check_stealth_ownership, check_transparent_ownership, OwnershipError, WalletOwnershipCtx,
    },
    tx_rpc_support, tx_runtime_state,
};
use crate::{
    chain::ReceiverCardRecord,
    claim::registry::{self as claim_registry, GlobalClaimRegistry},
    claim::{claim_scope_hash, sign_claim_receipt, verify_claim_receipt, ClaimReceipt},
    key::ReceiverKeys,
    persistence::tx::{TxStorage, TxStorageImpl},
    receiver::request::decode_request_compact,
    receiver::{
        PaymentRequest, ReceiveNext, ReceiveReject, ReceiveStatus, ReceiverCard, ScanResult,
        StealthOutputScanner, ValidatePaymentRequest, ValidationOutcome, VerifyResult,
    },
    rpc::error_mapping::map_wallet_error_to_rpc,
    rpc::types::{
        asset::{
            RuntimeAddAssetResponse, RuntimeAssetBalanceResponse, RuntimeAssetDetailsResponse,
            RuntimeAssetListFilter, RuntimeAssetMetadataResponse, RuntimeImportAssetResponse,
            RuntimeListAssetsResponse, RuntimeMergeAssetsResponse, RuntimeReceiveAssetResponse,
            RuntimeSendAssetResponse, RuntimeSplitAssetResponse, RuntimeStakeAssetsResponse,
            RuntimeSwapAssetsResponse, RuntimeUnstakeAssetsResponse,
        },
        common::{
            PersistTxId, PersistWalletId, RuntimeAssetAmount, RuntimeAssetRef,
            RuntimeOperationStatus,
        },
        security::RuntimeRateLimitError,
        security::SecurityErrorCode,
        wallet::SessionToken,
    },
    services::WalletService,
    stealth::{
        build_card_stealth_output_validated, build_tx_stealth_output_validated, BuildCheck,
        SenderWallet,
    },
    tx::{AssetSelector, AssetSelectorImpl, SelectionStrategy},
    wallet::WalletError,
    ChainType,
};
use z00z_crypto::KernelSignature;

const ASSET_LIST_DEFAULT_LIMIT: usize = 50;
const ASSET_LIST_MAX_LIMIT: usize = 50;
const ASSET_LIST_CACHE_TTL_MS: u64 = 50;
const ASSET_METADATA_CACHE_TTL_MS: u64 = 86_400_000;
const ASSET_LIST_CACHE_MAX_WALLETS: usize = 128;
const ASSET_METADATA_CACHE_MAX_ENTRIES: usize = 512;

#[path = "asset_rpc_support_assets.rs"]
mod asset_rpc_support_assets;

#[path = "asset_rpc_support_claims.rs"]
mod asset_rpc_support_claims;

#[path = "asset_rpc_support_state.rs"]
mod asset_rpc_support_state;

#[path = "asset_rpc_server.rs"]
mod asset_rpc_server;

#[cfg(all(test, not(target_arch = "wasm32")))]
// Canonical RPC receive tests stay module-local under src/rpc/test_asset_impl.rs.
#[path = "test_asset_impl.rs"]
mod test_asset_impl;

fn wallet_chain_id() -> Result<u32, ErrorObjectOwned> {
    let chain = crate::services::wallet_runtime_config::resolve_wallet_chain_type_checked()
        .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;
    Ok(AssetRpcImpl::chain_meta(chain).0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportRejectReason {
    MalformedJson,
    CryptoVerifyFailed,
    OwnerMismatch,
    StealthInconsistent,
    SecretFieldForbidden,
    AlreadyExists,
    ClaimConflict,
    ConservationViolation,
    SessionInvalid,
}

impl ImportRejectReason {
    fn code(self) -> &'static str {
        match self {
            Self::MalformedJson => "IMPORT_MALFORMED_JSON",
            Self::CryptoVerifyFailed => "IMPORT_CRYPTO_VERIFY_FAILED",
            Self::OwnerMismatch => "IMPORT_OWNER_MISMATCH",
            Self::StealthInconsistent => "IMPORT_STEALTH_INCONSISTENT",
            Self::SecretFieldForbidden => "IMPORT_SECRET_FIELD_FORBIDDEN",
            Self::AlreadyExists => "IMPORT_ALREADY_EXISTS",
            Self::ClaimConflict => "IMPORT_CLAIM_CONFLICT",
            Self::ConservationViolation => "IMPORT_CONSERVATION_VIOLATION",
            Self::SessionInvalid => "IMPORT_SESSION_INVALID",
        }
    }

    fn rpc_code(self) -> i32 {
        match self {
            Self::MalformedJson | Self::SessionInvalid => -32602,
            Self::ClaimConflict | Self::ConservationViolation => -32603,
            Self::CryptoVerifyFailed
            | Self::OwnerMismatch
            | Self::StealthInconsistent
            | Self::SecretFieldForbidden
            | Self::AlreadyExists => -32602,
        }
    }
}

fn is_crypto_dto(msg: &str) -> bool {
    [
        "commitment:",
        "range_proof:",
        "owner_signature:",
        "owner_pub:",
    ]
    .iter()
    .any(|field| msg.contains(field))
}

fn dto_reason(error: &AssetError) -> ImportRejectReason {
    match error {
        AssetError::InvalidStealth(_) => ImportRejectReason::StealthInconsistent,
        AssetError::InvalidAsset(msg) if is_crypto_dto(msg.as_ref()) => {
            ImportRejectReason::CryptoVerifyFailed
        }
        _ => ImportRejectReason::MalformedJson,
    }
}

#[derive(Debug, Clone)]
enum SendTarget {
    Card(ReceiverCard),
    Request(PaymentRequest),
}

/// Asset RPC service implementation.
///
/// Canonical send or receive authority flows through the live `wallet.tx.*`
/// plus owned-asset path. Compatibility asset-op helpers must stay
/// non-canonical unless they route through the same live authority plane.
pub struct AssetRpcImpl {
    time_provider: Arc<dyn TimeProvider>,
    service: Arc<WalletService>,
    asset_list_cache: AssetListCache,
    asset_metadata_cache: AssetMetadataCache,
    asset_send_rate_limits: AssetSendRateLimitMap,
    stake_id_counter: AssetStakeCounter,
    quarantined: AssetQuarantineStore,
    finalize_fail_on: Arc<AtomicBool>,
    verify_complete_calls: Arc<AtomicU64>,
}

impl AssetRpcImpl {
    pub(super) fn chain_meta(chain: ChainType) -> (u32, &'static str, &'static str) {
        match chain {
            ChainType::Mainnet => (1, "mainnet", "mainnet"),
            ChainType::Testnet => (2, "testnet", "testnet"),
            ChainType::Devnet => (3, "devnet", "devnet"),
        }
    }

    pub(super) fn chain_meta_from_id(chain_id: u32) -> (u32, &'static str, &'static str) {
        match chain_id {
            1 => Self::chain_meta(ChainType::Mainnet),
            2 => Self::chain_meta(ChainType::Testnet),
            _ => Self::chain_meta(ChainType::Devnet),
        }
    }

    pub(super) fn wallet_service(&self) -> &Arc<WalletService> {
        &self.service
    }

    pub(super) async fn reject_non_asset_alias(
        &self,
        wallet_id: &PersistWalletId,
        stable_key: AssetId,
    ) -> RpcResult<()> {
        let object = match self
            .wallet_service()
            .lookup_non_asset_owned_object(wallet_id, stable_key)
            .await
        {
            Ok(object) => object,
            Err(
                WalletError::SessionExpired
                | WalletError::SessionInvalid
                | WalletError::Locked
                | WalletError::NotFound(_),
            ) => None,
            Err(error) => return Err(map_wallet_error_to_rpc(error)),
        };

        let Some(object) = object else {
            return Ok(());
        };

        let family = match object.payload {
            crate::db::OwnedObjectPayload::Asset(_) => "asset",
            crate::db::OwnedObjectPayload::Voucher(_) => "voucher",
            crate::db::OwnedObjectPayload::Right(_) => "right",
        };

        Err(ErrorObjectOwned::owned(
            -32602,
            format!("Asset RPC accepts cash assets only; id belongs to {family} inventory"),
            None::<()>,
        ))
    }

    #[cfg(test)]
    fn build_wallet_service(time_provider: Arc<dyn TimeProvider>) -> Arc<WalletService> {
        struct WalletConfigEnvRestore {
            prev_path: Option<std::ffi::OsString>,
            prev_network: Option<std::ffi::OsString>,
            prev_chain: Option<std::ffi::OsString>,
        }

        impl WalletConfigEnvRestore {
            fn capture() -> Self {
                Self {
                    prev_path: std::env::var_os("Z00Z_WALLET_CONFIG_PATH"),
                    prev_network: std::env::var_os("Z00Z_WALLET_NETWORK"),
                    prev_chain: std::env::var_os("Z00Z_WALLET_CHAIN"),
                }
            }
        }

        impl Drop for WalletConfigEnvRestore {
            fn drop(&mut self) {
                match &self.prev_path {
                    Some(value) => std::env::set_var("Z00Z_WALLET_CONFIG_PATH", value),
                    None => std::env::remove_var("Z00Z_WALLET_CONFIG_PATH"),
                }
                match &self.prev_network {
                    Some(value) => std::env::set_var("Z00Z_WALLET_NETWORK", value),
                    None => std::env::remove_var("Z00Z_WALLET_NETWORK"),
                }
                match &self.prev_chain {
                    Some(value) => std::env::set_var("Z00Z_WALLET_CHAIN", value),
                    None => std::env::remove_var("Z00Z_WALLET_CHAIN"),
                }
            }
        }

        let _lock = crate::rpc::logging::RpcLoggingConfig::__lock_wallet_config_env();
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        Arc::new(WalletService::with_dependencies(time_provider))
    }

    #[cfg(not(test))]
    fn build_wallet_service(time_provider: Arc<dyn TimeProvider>) -> Arc<WalletService> {
        Arc::new(WalletService::with_dependencies(time_provider))
    }

    fn recv_err(reason: ReceiveReject) -> ErrorObjectOwned {
        let rpc_code = match reason {
            ReceiveReject::RuntimeFail => -32603,
            _ => -32602,
        };
        ErrorObjectOwned::owned(rpc_code, reason.rpc_code().to_string(), None::<()>)
    }

    pub fn new() -> Self {
        Self {
            time_provider: Arc::new(SystemTimeProvider),
            service: Self::build_wallet_service(Arc::new(SystemTimeProvider)),
            asset_list_cache: Arc::new(RwLock::new(BTreeMap::new())),
            asset_metadata_cache: Arc::new(RwLock::new(BTreeMap::new())),
            asset_send_rate_limits: Arc::new(RwLock::new(BTreeMap::new())),
            stake_id_counter: Arc::new(RwLock::new(0)),
            quarantined: Arc::new(RwLock::new(BTreeMap::new())),
            finalize_fail_on: Arc::new(AtomicBool::new(false)),
            verify_complete_calls: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn with_wallet_service(service: Arc<WalletService>) -> Self {
        Self {
            time_provider: Arc::new(SystemTimeProvider),
            service: Arc::clone(&service),
            asset_list_cache: Arc::new(RwLock::new(BTreeMap::new())),
            asset_metadata_cache: Arc::new(RwLock::new(BTreeMap::new())),
            asset_send_rate_limits: Arc::new(RwLock::new(BTreeMap::new())),
            stake_id_counter: Arc::new(RwLock::new(0)),
            quarantined: Arc::new(RwLock::new(BTreeMap::new())),
            finalize_fail_on: Arc::new(AtomicBool::new(false)),
            verify_complete_calls: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn with_dependencies(time_provider: Arc<dyn TimeProvider>) -> Self {
        Self {
            service: Self::build_wallet_service(Arc::clone(&time_provider)),
            time_provider,
            asset_list_cache: Arc::new(RwLock::new(BTreeMap::new())),
            asset_metadata_cache: Arc::new(RwLock::new(BTreeMap::new())),
            asset_send_rate_limits: Arc::new(RwLock::new(BTreeMap::new())),
            stake_id_counter: Arc::new(RwLock::new(0)),
            quarantined: Arc::new(RwLock::new(BTreeMap::new())),
            finalize_fail_on: Arc::new(AtomicBool::new(false)),
            verify_complete_calls: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn with_dependencies_and_wallet_service(
        time_provider: Arc<dyn TimeProvider>,
        wallet_service: Arc<WalletService>,
    ) -> Self {
        Self {
            time_provider,
            service: Arc::clone(&wallet_service),
            asset_list_cache: Arc::new(RwLock::new(BTreeMap::new())),
            asset_metadata_cache: Arc::new(RwLock::new(BTreeMap::new())),
            asset_send_rate_limits: Arc::new(RwLock::new(BTreeMap::new())),
            stake_id_counter: Arc::new(RwLock::new(0)),
            quarantined: Arc::new(RwLock::new(BTreeMap::new())),
            finalize_fail_on: Arc::new(AtomicBool::new(false)),
            verify_complete_calls: Arc::new(AtomicU64::new(0)),
        }
    }

    fn wallet_tx_store(
        &self,
        wallet_id: &PersistWalletId,
    ) -> TxStorageImpl<tx_rpc_support::TimeProviderRef> {
        let history_path = self.service.wallet_history_jsonl_path(wallet_id);
        TxStorageImpl::new(
            &history_path,
            tx_rpc_support::TimeProviderRef(Arc::clone(&self.time_provider)),
        )
    }

    async fn load_wallet_reserved_asset_ids(
        &self,
        wallet_id: &PersistWalletId,
    ) -> RpcResult<BTreeSet<AssetId>> {
        let store = self.wallet_tx_store(wallet_id);
        let records = store
            .list_by_status(crate::persistence::TxStatus::Pending)
            .map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("TxStorage error: {error}"), None::<()>)
            })?;

        Ok(tx_runtime_state::pending_input_asset_ids(records))
    }
}

impl Default for AssetRpcImpl {
    fn default() -> Self {
        Self::new()
    }
}
