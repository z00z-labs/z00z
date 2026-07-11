//! Chain RPC implementation.

use async_trait::async_trait;
use std::sync::Arc;

use super::chain_rpc::{ChainRpcServer, ChainScanRpc};
use crate::rpc::types::chain::{
    RuntimeBlockInfo, RuntimeScanStatus, RuntimeStartScanParams, RuntimeStartScanResponse,
};
use crate::rpc::types::common::{PersistWalletId, RuntimeOperationStatus};
use crate::rpc::types::network::RuntimeSwitchChainResponse;
use crate::services::AppService;

/// Chain RPC service implementation.
pub struct ChainRpcImpl {
    app_service: Arc<AppService>,
}

impl ChainRpcImpl {
    pub fn new(app_service: Arc<AppService>) -> Self {
        Self { app_service }
    }
}

#[async_trait]
impl ChainRpcServer for ChainRpcImpl {
    async fn switch_to_mainnet(&self) -> jsonrpsee::core::RpcResult<RuntimeSwitchChainResponse> {
        let chain = self
            .app_service
            .switch_chain_to_mainnet()
            .await
            .map_err(|e| jsonrpsee::types::ErrorObjectOwned::owned(1, e.to_string(), None::<()>))?;

        Ok(RuntimeSwitchChainResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: String::new(),
            },
            chain,
        })
    }

    async fn switch_to_testnet(&self) -> jsonrpsee::core::RpcResult<RuntimeSwitchChainResponse> {
        let chain = self
            .app_service
            .switch_chain_to_testnet()
            .await
            .map_err(|e| jsonrpsee::types::ErrorObjectOwned::owned(1, e.to_string(), None::<()>))?;

        Ok(RuntimeSwitchChainResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: String::new(),
            },
            chain,
        })
    }

    async fn switch_to_devnet(&self) -> jsonrpsee::core::RpcResult<RuntimeSwitchChainResponse> {
        let chain = self
            .app_service
            .switch_chain_to_devnet()
            .await
            .map_err(|e| jsonrpsee::types::ErrorObjectOwned::owned(1, e.to_string(), None::<()>))?;

        Ok(RuntimeSwitchChainResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: String::new(),
            },
            chain,
        })
    }
}

// ============================================================================
// Wallet-local scan RPC
// ============================================================================

/// Wallet-local ChainScanRpc implementation.
///
/// This keeps scan state per wallet inside the process and delegates the
/// authoritative behavior to `AppService`.
pub struct ChainScanRpcImpl {
    app_service: Arc<AppService>,
}

impl ChainScanRpcImpl {
    pub fn new(app_service: Arc<AppService>) -> Self {
        Self { app_service }
    }
}

#[async_trait]
impl ChainScanRpc for ChainScanRpcImpl {
    async fn start_local_scan(&self, params: RuntimeStartScanParams) -> RuntimeStartScanResponse {
        self.app_service.start_local_scan(params).await
    }

    async fn stop_local_scan(&self, wallet_id: PersistWalletId) {
        self.app_service.stop_local_scan(wallet_id).await;
    }

    async fn get_local_scan_status(&self, wallet_id: PersistWalletId) -> RuntimeScanStatus {
        self.app_service.get_local_scan_status(wallet_id).await
    }

    async fn get_local_scan_tip(&self) -> RuntimeBlockInfo {
        self.app_service.get_local_scan_tip().await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::types::chain::{
        RuntimeReceiveScanOutcome, RuntimeScanState, RuntimeStartScanParams as StartScanParams,
    };
    use crate::services::WalletService;

    #[tokio::test]
    async fn test_local_scan_job() {
        let app_service = Arc::new(AppService::new());
        let scan_rpc = ChainScanRpcImpl::new(app_service);
        let wallet_id = PersistWalletId("start-scan-wallet".to_string());
        let params = StartScanParams {
            wallet_id: wallet_id.clone(),
            from_height: Some(1000),
        };

        let response = scan_rpc.start_local_scan(params).await;

        let job_id = response.job.job_id.expect("job_id must be present");

        assert!(job_id.contains("start-scan-wallet"));
        assert_eq!(
            response.scan_range,
            Some(crate::rpc::types::chain::BlockRange {
                from_height: 1000,
                to_height: 11_000,
            })
        );

        let status = scan_rpc.get_local_scan_status(wallet_id).await;
        assert_eq!(status.state, RuntimeScanState::Scanning);
        assert_eq!(status.current_height, 1000);
        assert_eq!(status.job.job_id.as_deref(), Some(job_id.as_str()));
    }

    #[tokio::test]
    async fn test_start_scan_genesis() {
        let app_service = Arc::new(AppService::new());
        let scan_rpc = ChainScanRpcImpl::new(app_service);
        let wallet_id = PersistWalletId("wallet123".to_string());
        let params = StartScanParams {
            wallet_id: wallet_id.clone(),
            from_height: None,
        };

        let response = scan_rpc.start_local_scan(params).await;

        assert!(response.job.job_id.is_some());
        assert_eq!(response.scan_range, None);

        let status = scan_rpc.get_local_scan_status(wallet_id).await;

        assert_eq!(status.current_height, 0);
        assert_eq!(status.job.progress_or_zero(), 0.0);
    }

    #[tokio::test]
    async fn test_stop_local_scan_completes() {
        let app_service = Arc::new(AppService::new());
        let scan_rpc = ChainScanRpcImpl::new(app_service);
        let wallet_id = PersistWalletId("stop-scan-wallet".to_string());

        scan_rpc.stop_local_scan(wallet_id).await;
    }

    #[tokio::test]
    async fn test_get_scan_status() {
        let app_service = Arc::new(AppService::new());
        let scan_rpc = ChainScanRpcImpl::new(app_service);
        let wallet_id = PersistWalletId("idle-wallet".to_string());

        let status = scan_rpc.get_local_scan_status(wallet_id).await;

        assert_eq!(status.state, RuntimeScanState::Idle);
        assert!(status.is_scanned());
        assert_eq!(status.job.progress_or_zero(), 1.0);
    }

    #[tokio::test]
    async fn test_get_scan_tip() {
        let app_service = Arc::new(AppService::new());
        let scan_rpc = ChainScanRpcImpl::new(app_service);

        let tip = scan_rpc.get_local_scan_tip().await;

        assert!(tip.height > 0);
        assert!(!tip.hash.is_empty());
        assert!(tip.hash.starts_with("0x"));
        assert!(tip.tx_count > 0);
    }

    #[tokio::test]
    async fn test_scan_lifecycle() {
        let app_service = Arc::new(AppService::new());
        let scan_rpc = ChainScanRpcImpl::new(app_service);
        let wallet_id = PersistWalletId("lifecycle-wallet".to_string());

        let params = StartScanParams {
            wallet_id: wallet_id.clone(),
            from_height: Some(5000),
        };
        let _ = scan_rpc.start_local_scan(params).await;

        let status_started = scan_rpc.get_local_scan_status(wallet_id.clone()).await;
        assert_eq!(status_started.state, RuntimeScanState::Scanning);

        let status_scanning = scan_rpc.get_local_scan_status(wallet_id.clone()).await;
        assert_eq!(status_scanning.state, RuntimeScanState::Scanning);

        scan_rpc.stop_local_scan(wallet_id.clone()).await;

        let status = scan_rpc.get_local_scan_status(wallet_id).await;
        assert_eq!(status.state, RuntimeScanState::Paused);
    }

    #[tokio::test]
    async fn test_scan_progress_calculation() {
        let app_service = Arc::new(AppService::new());
        let scan_rpc = ChainScanRpcImpl::new(app_service);
        let params = StartScanParams {
            wallet_id: PersistWalletId("wallet123".to_string()),
            from_height: Some(2500),
        };

        let _ = scan_rpc.start_local_scan(params).await;

        let status = scan_rpc
            .get_local_scan_status(PersistWalletId("wallet123".to_string()))
            .await;

        assert_eq!(status.job.progress_or_zero(), 0.0);
        assert_eq!(status.progress_percent(), 0.0);
    }

    #[tokio::test]
    async fn test_local_scan_tip_timestamp() {
        let app_service = Arc::new(AppService::new());
        let scan_rpc = ChainScanRpcImpl::new(app_service);
        let tip = scan_rpc.get_local_scan_tip().await;

        assert!(tip.timestamp > 1577836800u64.saturating_mul(1000));
    }

    #[tokio::test]
    async fn test_multiple_wallets_scan() {
        let app_service = Arc::new(AppService::new());
        let scan_rpc = ChainScanRpcImpl::new(app_service);

        let wallet1 = PersistWalletId("wallet1".to_string());
        let wallet2 = PersistWalletId("wallet2".to_string());

        let params1 = StartScanParams {
            wallet_id: wallet1.clone(),
            from_height: None,
        };
        let params2 = StartScanParams {
            wallet_id: wallet2.clone(),
            from_height: Some(1000),
        };

        let response1 = scan_rpc.start_local_scan(params1).await;
        let response2 = scan_rpc.start_local_scan(params2).await;

        assert_ne!(response1.job.job_id, response2.job.job_id);

        let status1 = scan_rpc.get_local_scan_status(wallet1).await;
        let status2 = scan_rpc.get_local_scan_status(wallet2).await;
        assert_eq!(status1.state, RuntimeScanState::Scanning);
        assert_eq!(status2.state, RuntimeScanState::Scanning);
    }

    #[tokio::test]
    async fn test_eta_seconds_some_scanning() {
        let app_service = Arc::new(AppService::new());
        let scan_rpc = ChainScanRpcImpl::new(app_service);
        let params = StartScanParams {
            wallet_id: PersistWalletId("wallet123".to_string()),
            from_height: Some(0),
        };

        let response = scan_rpc.start_local_scan(params).await;

        assert!(response.job.eta_seconds.is_some());
        assert!(response.job.eta_seconds.unwrap() > 0);
    }

    #[tokio::test]
    async fn test_scan_status_includes_outcome() {
        let temp = tempfile::tempdir().unwrap();
        let wallets = Arc::new(WalletService::with_output_dir(temp.path().to_path_buf()));
        let wallet_id = PersistWalletId("receive-outcome-wallet".to_string());
        wallets
            .record_receive_scan_outcome(&wallet_id, RuntimeReceiveScanOutcome::ImportedHit)
            .await;

        let app_service = Arc::new(AppService::with_wallet_service(Arc::clone(&wallets)));
        let scan_rpc = ChainScanRpcImpl::new(app_service);

        let status = scan_rpc.get_local_scan_status(wallet_id).await;

        assert_eq!(
            status.last_receive_outcome,
            Some(RuntimeReceiveScanOutcome::ImportedHit)
        );
        assert!(status.is_scanned());
    }
}
