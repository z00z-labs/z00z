#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;
use std::sync::Arc;

use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::{rng::SystemRngProvider, time::MockTimeProvider};
use z00z_wallets::{
    rpc::methods::{
        BackupRpcImpl, BackupRpcServer, KeyRpcImpl, KeyRpcServer, WalletRpcImpl, WalletRpcServer,
    },
    rpc::types::{common::PersistWalletId, security::SecurityErrorCode, wallet::SessionToken},
    services::{AppService, WalletService},
};

#[path = "test_inc/test_wallet_env.inc"]
mod test_common;

const TEST_PASSWORD: &str = "Aa1!bB2@cC3#dD4$eE5%";
const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

const WALLET_RPC_IMPL: &str = include_str!("../src/rpc/wallet_rpc_impl.rs");
const KEY_RPC_SERVER: &str = include_str!("../src/rpc/key_rpc_server.rs");
const KEY_RPC_DERIVE: &str = include_str!("../src/rpc/key_rpc_server_derive.rs");
const KEY_RPC_REQUESTS: &str = include_str!("../src/rpc/key_rpc_server_requests.rs");
const KEY_RPC_ADMIN: &str = include_str!("../src/rpc/key_rpc_server_admin.rs");
const BACKUP_RPC_IMPL: &str = include_str!("../src/rpc/backup_rpc_impl.rs");
const ROUTE_SRC: &str = include_str!("../src/rpc/wallet_dispatcher_routes.rs");
const TODO_DOC: &str = include_str!("../../../.planning/phases/000/065-Attack-Surface/065-TODO.md");

fn fn_body<'a>(src: &'a str, signature: &str) -> &'a str {
    let start = src
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature {signature}"));
    let tail = &src[start..];
    let end = tail.find("\n    async fn ").unwrap_or(tail.len());
    &tail[..end]
}

fn assert_fn_has(src: &str, signature: &str, needle: &str) {
    let body = fn_body(src, signature);
    assert!(body.contains(needle), "{signature} must contain {needle}");
}

fn route_block<'a>(src: &'a str, rpc_name: &str) -> &'a str {
    let anchor = format!("dispatcher.register_typed(\n        \"{rpc_name}\"");
    let inline_anchor = format!("dispatcher.register_typed(\"{rpc_name}\"");
    let start = src
        .find(&anchor)
        .or_else(|| src.find(&inline_anchor))
        .unwrap_or_else(|| panic!("missing route {rpc_name}"));
    let tail = &src[start..];
    let end = tail
        .find("dispatcher.register_typed(")
        .filter(|idx| *idx > 0)
        .unwrap_or(tail.len());
    &tail[..end]
}

fn assert_route_has(rpc_name: &str, guard_call: &str, checked_call: &str) {
    let body = route_block(ROUTE_SRC, rpc_name);
    assert!(
        body.contains("typed_handler_cap("),
        "{rpc_name} must use typed_handler_cap"
    );
    assert!(
        body.contains(guard_call),
        "{rpc_name} must verify through {guard_call}"
    );
    assert!(
        body.contains(checked_call),
        "{rpc_name} must route through {checked_call}"
    );
}

fn mk_wallet_rpc() -> (WalletRpcImpl, KeyRpcImpl, BackupRpcImpl, Arc<WalletService>) {
    let dir = tempfile::tempdir().expect("tempdir");
    let _keep = Box::leak(Box::new(dir));
    let service = Arc::new(WalletService::create_service_custom_output_directory(
        PathBuf::from(_keep.path()),
        Arc::new(MockTimeProvider::default()),
        SystemRngProvider,
    ));
    (
        WalletRpcImpl::new(Arc::clone(&service)),
        KeyRpcImpl::new(Arc::clone(&service)),
        BackupRpcImpl::new(Arc::clone(&service)),
        service,
    )
}

async fn create_unlocked_wallet(service: Arc<WalletService>) -> (PersistWalletId, SessionToken) {
    let app = AppService::with_wallet_service(Arc::clone(&service));
    let wallet_id = app
        .create_wallet(
            "sensitive-rpc-session".to_string(),
            TEST_PASSWORD.to_string(),
            Some(TEST_SEED_PHRASE_24.to_string()),
        )
        .await
        .expect("create wallet")
        .wallet_id;
    let session = service
        .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(TEST_PASSWORD))
        .await
        .expect("unlock wallet");
    (wallet_id, session)
}

fn bad_session(session: &SessionToken) -> SessionToken {
    let mut bad = session.clone();
    bad.token = format!("{}-bad", bad.token);
    bad
}

fn assert_session_invalid(err: jsonrpsee::types::ErrorObjectOwned) {
    assert_eq!(err.code(), SecurityErrorCode::SessionInvalid.code());
    assert_eq!(err.message(), SecurityErrorCode::SessionInvalid.message());
}

#[test]
fn sensitive_rpc_sources_use_caps() {
    assert!(TODO_DOC.contains("VerifiedSession"));
    assert!(TODO_DOC.contains("Centralize sensitive-method registration"));
    assert!(ROUTE_SRC.contains("PRIV_ROUTE_SPECS"));

    assert_fn_has(
        WALLET_RPC_IMPL,
        "async fn lock_wallet_checked(",
        "cap: VerifiedSessionNoTouch",
    );
    assert_fn_has(
        WALLET_RPC_IMPL,
        "async fn show_seed_phrase_checked(",
        "cap: VerifiedSessionNoTouch",
    );
    assert_fn_has(
        WALLET_RPC_IMPL,
        "async fn lock_wallet(",
        "verify_no_touch_cap(session)",
    );
    assert_fn_has(
        WALLET_RPC_IMPL,
        "async fn show_seed_phrase(",
        "verify_no_touch_cap(session)",
    );

    assert_fn_has(
        KEY_RPC_DERIVE,
        "async fn derive_receiver_checked(",
        "cap: VerifiedSessionNoTouch",
    );
    assert_fn_has(
        KEY_RPC_DERIVE,
        "async fn get_receiver_card_checked(",
        "cap: VerifiedSession",
    );
    assert_fn_has(
        KEY_RPC_REQUESTS,
        "async fn create_payment_request_checked(",
        "cap: VerifiedSession",
    );
    assert_fn_has(
        KEY_RPC_REQUESTS,
        "async fn validate_payment_request_checked(",
        "cap: VerifiedSession",
    );
    assert_fn_has(
        KEY_RPC_REQUESTS,
        "async fn export_public_material_checked(",
        "cap: VerifiedSession",
    );
    assert_fn_has(
        KEY_RPC_ADMIN,
        "async fn rotate_master_key_checked(",
        "cap: VerifiedSessionNoTouch",
    );
    assert_fn_has(
        KEY_RPC_ADMIN,
        "async fn list_receivers_checked(",
        "cap: VerifiedSession",
    );
    assert_fn_has(
        KEY_RPC_ADMIN,
        "async fn label_receiver_checked(",
        "cap: VerifiedSession",
    );
    assert_fn_has(
        KEY_RPC_SERVER,
        "async fn rotate_master_key(",
        "verify_rotate_cap(session)",
    );

    assert_fn_has(
        BACKUP_RPC_IMPL,
        "async fn create_backup_checked(",
        "cap: VerifiedSession",
    );
    assert_fn_has(
        BACKUP_RPC_IMPL,
        "async fn list_backups_checked(",
        "cap: VerifiedSession",
    );
    assert_fn_has(
        BACKUP_RPC_IMPL,
        "async fn configure_backup_checked(",
        "cap: VerifiedSession",
    );

    assert_route_has(
        "wallet.session.lock_wallet",
        "verify_no_touch_cap(session)",
        "lock_wallet_checked(cap)",
    );
    assert_route_has(
        "wallet.session.show_seed_phrase",
        "verify_no_touch_cap(session)",
        "show_seed_phrase_checked(cap, p.password, p.confirmation)",
    );
    assert_route_has(
        "wallet.backup.create_backup",
        "verify_touch_cap(session)",
        "create_backup_checked(cap, p.password, p.destination)",
    );
    assert_route_has(
        "wallet.key.derive_receiver",
        "verify_no_touch_cap(session)",
        "derive_receiver_checked(cap, p.path)",
    );
    assert_route_has(
        "wallet.key.rotate_master_key",
        "verify_rotate_cap(session)",
        "rotate_master_key_checked(cap, p.password, p.confirmation)",
    );
}

#[tokio::test]
async fn show_seed_phrase_bad_session() {
    let _env = test_common::WalletEnvGuard::new("p2p", "devnet");
    let (wallet_rpc, _key_rpc, _backup_rpc, service) = mk_wallet_rpc();
    let (_wallet_id, session) = create_unlocked_wallet(service).await;
    let err = wallet_rpc
        .show_seed_phrase(
            bad_session(&session),
            TEST_PASSWORD.to_string(),
            TEST_PASSWORD.to_string(),
        )
        .await
        .expect_err("bad session must fail");
    assert_session_invalid(err);
}

#[tokio::test]
async fn derive_receiver_rejects_invalid_session() {
    let _env = test_common::WalletEnvGuard::new("p2p", "devnet");
    let (_wallet_rpc, key_rpc, _backup_rpc, service) = mk_wallet_rpc();
    let (_wallet_id, session) = create_unlocked_wallet(service).await;
    let err = key_rpc
        .derive_receiver(bad_session(&session), "m/44'/1337'/0'/0/0".to_string())
        .await
        .expect_err("bad session must fail");
    assert_session_invalid(err);
}

#[tokio::test]
async fn export_material_bad_session() {
    let _env = test_common::WalletEnvGuard::new("p2p", "devnet");
    let (_wallet_rpc, key_rpc, _backup_rpc, service) = mk_wallet_rpc();
    let (_wallet_id, session) = create_unlocked_wallet(service).await;
    let err = key_rpc
        .export_public_material(bad_session(&session), 0, TEST_PASSWORD.to_string())
        .await
        .expect_err("bad session must fail");
    assert_session_invalid(err);
}

#[tokio::test]
async fn create_backup_rejects_invalid_session() {
    let _env = test_common::WalletEnvGuard::new("p2p", "devnet");
    let (_wallet_rpc, _key_rpc, backup_rpc, service) = mk_wallet_rpc();
    let (_wallet_id, session) = create_unlocked_wallet(service).await;
    let err = backup_rpc
        .create_backup(bad_session(&session), TEST_PASSWORD.to_string(), None)
        .await
        .expect_err("bad session must fail");
    assert_session_invalid(err);
}
