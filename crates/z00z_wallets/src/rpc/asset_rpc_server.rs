use super::{
    asset_rpc_stakes, async_trait, build_card_stealth_output_validated,
    build_tx_stealth_output_validated, claim_registry, decode_asset_pkg_json, dto_reason,
    map_wallet_error_to_rpc, payload_has_secret_field, tx_rpc_support, wallet_chain_id, AssetError,
    AssetId, AssetRpcImpl, AssetRpcServer, AssetSelector, AssetSelectorImpl, AssetWire, BuildCheck,
    Codec, DefinitionWire, ErrorObjectOwned, ImportRejectReason, JsonCodec, Logger, Ordering,
    PersistWalletId, ReceiveNext, ReceiveReject, ReceiveStatus, RpcResult, RuntimeAddAssetResponse,
    RuntimeAssetAmount, RuntimeAssetBalanceResponse, RuntimeAssetDetailsResponse,
    RuntimeAssetListFilter, RuntimeAssetMetadataResponse, RuntimeAssetRef,
    RuntimeImportAssetResponse, RuntimeListAssetsResponse, RuntimeMergeAssetsResponse,
    RuntimeOperationStatus, RuntimeRateLimitError, RuntimeReceiveAssetResponse,
    RuntimeSendAssetResponse, RuntimeSplitAssetResponse, RuntimeStakeAssetsResponse,
    RuntimeSwapAssetsResponse, RuntimeUnstakeAssetsResponse, ScanResult, SecurityErrorCode,
    SelectionStrategy, SendTarget, SenderWallet, SessionToken, StealthOutputScanner,
    SystemRngProvider, TracingLogger, WalletService, ASSET_LIST_DEFAULT_LIMIT,
    ASSET_LIST_MAX_LIMIT, ASSET_SEND_RATE_LIMIT_WINDOW,
};

include!("asset_rpc_server_catalog.rs");
include!("asset_rpc_server_transfer.rs");
include!("asset_rpc_server_ops.rs");

#[async_trait]
impl AssetRpcServer for AssetRpcImpl {
    async fn list_assets(
        &self,
        wallet_id: PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        filter: Option<RuntimeAssetListFilter>,
    ) -> RpcResult<RuntimeListAssetsResponse> {
        self.list_assets_impl(wallet_id, limit, cursor, filter)
            .await
    }

    async fn add_asset(
        &self,
        session: SessionToken,
        asset_data: String,
    ) -> RpcResult<RuntimeAddAssetResponse> {
        self.add_asset_impl(session, asset_data).await
    }

    async fn get_asset_balance(
        &self,
        wallet_id: PersistWalletId,
        asset_id: AssetId,
    ) -> RpcResult<RuntimeAssetBalanceResponse> {
        self.get_asset_balance_impl(wallet_id, asset_id).await
    }

    async fn get_asset_details(
        &self,
        wallet_id: PersistWalletId,
        asset_id: AssetId,
    ) -> RpcResult<RuntimeAssetDetailsResponse> {
        self.get_asset_details_impl(wallet_id, asset_id).await
    }

    async fn import_asset(
        &self,
        session: SessionToken,
        asset_data: String,
    ) -> RpcResult<RuntimeImportAssetResponse> {
        self.import_asset_impl(session, asset_data).await
    }

    async fn merge_assets(
        &self,
        session: SessionToken,
        asset_ids: Vec<AssetId>,
    ) -> RpcResult<RuntimeMergeAssetsResponse> {
        self.merge_assets_impl(session, asset_ids).await
    }

    async fn get_asset_metadata(
        &self,
        asset_id: AssetId,
    ) -> RpcResult<RuntimeAssetMetadataResponse> {
        self.get_asset_metadata_impl(asset_id).await
    }

    async fn receive_asset(
        &self,
        session: SessionToken,
        asset_id: AssetId,
    ) -> RpcResult<RuntimeReceiveAssetResponse> {
        self.receive_asset_impl(session, asset_id).await
    }

    async fn send_asset(
        &self,
        session: SessionToken,
        asset_id: AssetId,
        recipient: String,
        amount: u64,
    ) -> RpcResult<RuntimeSendAssetResponse> {
        self.send_asset_impl(session, asset_id, recipient, amount)
            .await
    }

    async fn split_asset(
        &self,
        session: SessionToken,
        asset_id: AssetId,
        amounts: Vec<u64>,
    ) -> RpcResult<RuntimeSplitAssetResponse> {
        self.split_asset_impl(session, asset_id, amounts).await
    }

    async fn stake_assets(
        &self,
        session: SessionToken,
        asset_id: AssetId,
        amount: u64,
    ) -> RpcResult<RuntimeStakeAssetsResponse> {
        self.stake_assets_impl(session, asset_id, amount).await
    }

    async fn swap_assets(
        &self,
        session: SessionToken,
        from_asset_id: AssetId,
        to_asset_id: AssetId,
        amount: u64,
    ) -> RpcResult<RuntimeSwapAssetsResponse> {
        self.swap_assets_impl(session, from_asset_id, to_asset_id, amount)
            .await
    }

    async fn unstake_assets(
        &self,
        session: SessionToken,
        stake_id: String,
    ) -> RpcResult<RuntimeUnstakeAssetsResponse> {
        self.unstake_assets_impl(session, stake_id).await
    }
}
