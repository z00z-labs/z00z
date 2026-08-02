#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcTransport};
use z00z_utils::{
    codec::{json, Codec, JsonCodec},
    rng::SystemRngProvider,
    time::SystemTimeProvider,
};
use z00z_wallets::{
    key::generate_identity_keypair,
    rpc::{
        logging::{LoggedRpcTransport, RpcLoggingConfig},
        methods::{
            AppRpcImpl, AppRpcServer, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl,
            KeyRpcImpl, NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl, WalletRpcServer,
        },
        register_all_wallet_rpc_methods,
    },
    services::{AppService, WalletService},
    tx::{ThinSnapshot, ThinWalletTxPackage},
};

#[path = "test_inc/test_rpc_logger.inc"]
mod test_common;
#[path = "test_thin_support.rs"]
mod thin_test_support;

use thin_test_support::{context_for_entry, fixture_entry};

#[tokio::test]
async fn test_logging_keeps_metadata() {
    let dir = tempfile::tempdir().expect("tempdir");
    let service = Arc::new(WalletService::with_output_dir(dir.path().join("wallets")));

    let wallet_rpc = Arc::new(WalletRpcImpl::new(Arc::clone(&service)));
    let app_service = Arc::new(AppService::with_wallet_service(Arc::clone(&service)));
    let app_rpc = Arc::new(AppRpcImpl::new(Arc::clone(&app_service)));
    let asset_rpc = Arc::new(AssetRpcImpl::new());
    let tx_rpc = Arc::new(TxRpcImpl::new(Arc::clone(&service)));
    let backup_rpc = Arc::new(BackupRpcImpl::new(Arc::clone(&service)));
    let key_rpc = Arc::new(KeyRpcImpl::new(Arc::clone(&service)));
    let chain_rpc = Arc::new(ChainRpcImpl::new(Arc::clone(&app_service)));
    let network_rpc = Arc::new(NetworkRpcImpl::with_app_service(Arc::clone(&app_service)));
    let scan_rpc = Arc::new(ChainScanRpcImpl::new(Arc::clone(&app_service)));
    let storage_rpc = Arc::new(StorageRpcImpl::new(Arc::clone(&service)));

    let dispatcher = Arc::new(RpcDispatcher::new());
    register_all_wallet_rpc_methods(
        &dispatcher,
        Arc::clone(&app_rpc),
        Arc::clone(&wallet_rpc),
        Arc::clone(&asset_rpc),
        Arc::clone(&tx_rpc),
        Arc::clone(&backup_rpc),
        Arc::clone(&key_rpc),
        Arc::clone(&chain_rpc),
        Arc::clone(&network_rpc),
        Arc::clone(&scan_rpc),
        Arc::clone(&storage_rpc),
    )
    .expect("wallet RPC registration");

    let base = LocalRpcTransport::new(dispatcher);
    let (logger, vec_logger) = test_common::rpc_test_tee_logger();
    let transport = LoggedRpcTransport::new(
        base,
        RpcLoggingConfig::from_default_wallet_yaml().expect("rpc logging config"),
        logger,
        Arc::new(SystemTimeProvider),
        SystemRngProvider,
    );

    let created = app_rpc
        .create_wallet(
            "thin-privacy-wallet".to_string(),
            "StrongPassw0rd!".to_string(),
            None,
        )
        .await
        .expect("create wallet");
    let session = wallet_rpc
        .unlock_wallet(created.wallet_id.clone(), "StrongPassw0rd!".to_string())
        .await
        .expect("unlock wallet");

    let entry = fixture_entry().await;
    let (identity_sk, _) = generate_identity_keypair().unwrap();
    let snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 70, 1_700_000_000_000, 4_700_000_000_000),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("signed snapshot");
    tx_rpc
        .publish_thin_snapshot(snapshot.clone())
        .await
        .expect("publish snapshot");
    let pin = tx_rpc
        .pin_thin_snapshot(&snapshot.snapshot_digest_hex)
        .await
        .expect("pin snapshot");
    let thin = ThinWalletTxPackage::new(&pin, &entry).expect("build thin wrapper");
    let thin_json = String::from_utf8(JsonCodec.serialize(&thin).expect("serialize thin wrapper"))
        .expect("utf8 thin wrapper");

    transport
        .call(
            "wallet.tx.verify_transaction_package",
            json!({
                "session": session.clone(),
                "tx_data": thin_json,
            }),
        )
        .await
        .expect("verify thin package through logged transport");

    let lines = vec_logger.lines();
    assert!(!lines.is_empty(), "logged transport must emit a line");

    let verify_line = lines
        .iter()
        .find(|line| line.contains("\"method\":\"wallet.tx.verify_transaction_package\""))
        .expect("verify_transaction_package log line");

    assert!(
        verify_line.contains("\"transport_mode\":\"thin\""),
        "thin verification logs must record the transport mode"
    );
    assert!(
        verify_line.contains("\"input_refs_count\":"),
        "thin verification logs must keep only bounded helper metadata"
    );

    for forbidden in [
        thin.snapshot_entry_id_hex.as_str(),
        thin.snapshot_digest_hex.as_str(),
        thin.metadata_hash_hex.as_str(),
        thin.input_refs[0].asset_id_hex.as_str(),
        session.token.as_str(),
    ] {
        assert!(
            !verify_line.contains(forbidden),
            "helper/private metadata must not appear in logs: {forbidden}"
        );
    }

    assert!(
        !verify_line.contains("\"snapshot_entry_id_hex\"")
            && !verify_line.contains("\"snapshot_digest_hex\"")
            && !verify_line.contains("\"metadata_hash_hex\"")
            && !verify_line.contains("\"input_refs\""),
        "thin log summary must not expose helper-only fields"
    );
}
