#![cfg(not(target_arch = "wasm32"))]

const RECEIVER_FLOW: &str =
    include_str!("../../../wiki/04-wallet-and-rpc/receiver-request-flow.md");
const SESSION_LOCKS: &str = include_str!("../../../wiki/04-wallet-and-rpc/wallet-session-locks.md");
const STUB_SURFACE: &str = include_str!("../../../wiki/04-wallet-and-rpc/wallet-stub-surface.md");
const WLT_RESTORE: &str = include_str!("../../../wiki/04-wallet-and-rpc/wallet-wlt-restore.md");
const TODO_DOC: &str = include_str!("../../../.planning/phases/000/065-Attack-Surface/065-TODO.md");
const ROUTE_SRC: &str = include_str!("../src/rpc/wallet_dispatcher_routes.rs");
const CAP_SRC: &str = include_str!("../src/services/wallet_session_runtime_limits.rs");
const STORE_RESTORE: &str = include_str!("../src/services/wallet_store_restore.rs");
const APP_LIFE: &str = include_str!("../src/services/app_wallet_lifecycle.rs");

#[test]
fn cap_matrix_live() {
    assert!(
        TODO_DOC.contains("Publish and enforce a capability matrix for native and wasm targets.")
    );
    assert!(CAP_SRC.contains("enum SessionCapKind"));
    assert!(CAP_SRC.contains("enum SessionCapState"));
    assert!(CAP_SRC.contains("const SESSION_CAP_MATRIX"));
    assert!(CAP_SRC.contains("verified touch session is not supported on wasm32"));
    assert!(CAP_SRC.contains("verified no-touch session is not supported on wasm32"));
    assert!(ROUTE_SRC.contains("PRIV_ROUTE_SPECS"));
    assert!(ROUTE_SRC.contains("PrivRouteGuard::Touch"));
    assert!(ROUTE_SRC.contains("PrivRouteGuard::NoTouch"));
}

#[test]
fn wasm_wlt_bounds_live() {
    assert!(STORE_RESTORE.contains(".wlt persistence is not supported on wasm32"));
    assert!(STORE_RESTORE.contains(".wlt owned-asset loading is not supported on wasm32"));
    assert!(WLT_RESTORE.contains("Rejects wasm32 and routes native load through `spawn_blocking`."));
    assert!(SESSION_LOCKS.contains("browser builds do not get this live session model."));
    assert!(SESSION_LOCKS.contains("native and wasm surfaces are intentionally not equivalent"));
    assert!(RECEIVER_FLOW.contains("The durable TOFU and inbox receive flow is native-only today."));
    assert!(
        !RECEIVER_FLOW.contains("browser surfaces are equivalent to the native wallet trust model")
    );
}

#[test]
fn stub_surface_honest() {
    assert!(STUB_SURFACE.contains("| App wallet lifecycle | Real orchestration."));
    assert!(STUB_SURFACE.contains(
        "| `NetworkService`, `StorageService`, `KeyService`, `BackupService` | Placeholder-only."
    ));
    assert!(STUB_SURFACE.contains("opens a wallet source for discovery metadata"));
    assert!(APP_LIFE.contains("pub async fn open_wallet_source("));
    assert!(APP_LIFE.contains("self.wallets.open_wallet_source(source).await"));
}
