#![doc = include_str!("../README.md")]
//! Z00Z Networks RPC - transport-focused request dispatch
//!
//! This crate provides a transport-focused RPC layer for Z00Z components.
//! It owns request dispatch, transport adaptation, and local testing helpers only.
//! It does not define the whole network stack.
//!
//! The following higher-level concerns stay outside this crate on purpose:
//! - peer identity
//! - authentication
//! - retry policy
//! - connection lifecycle
//!
//! Wallet adapters, node overlays, and future network crates may compose those
//! concerns around `RpcTransport`, but they must not smuggle them into the
//! dispatcher or transport abstractions here.
//!
//! This crate is reusable across different Z00Z components (wallets, rollup nodes, DA layers)
//! precisely because it stays limited to transport and dispatch semantics.
//!
//! # Architecture
//!
//! ```text
//! Application Layer (z00z_wallets, z00z_rollup_node, etc.)
//!           ↓
//!    RpcTransport trait (generic)
//!           ↓
//!    ┌──────────┬──────────┬──────────┐
//!    │          │          │          │
//! LocalRpc  WasmRpc   HttpRpc   (custom)
//! (native)  (browser) (remote)
//! ```
//!
//! # Features
//!
//! - **Transport Abstraction**: `RpcTransport` trait for pluggable backends
//! - **Request Dispatch**: `RpcDispatcher` for method routing without business ownership
//! - **Local Testing Helpers**: `LocalRpcTransport` for in-process integration and transport-local tests
//! - **WASM Support**: `WasmRpcClient` for browser environments
//! - **Error Handling**: Generic `RpcError` type
//! - **Logger Integration**: Optional z00z_utils logger support
//!
//! # Example
//!
//! ```no_run
//! use z00z_networks_rpc::RpcTransport;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! #[cfg(target_arch = "wasm32")]
//! {
//!     use z00z_networks_rpc::WasmRpcClient;
//!     let client = WasmRpcClient::new("ws://localhost:9944").await?;
//!     let response = client
//!         .call("method.name", z00z_utils::codec::Value::Object(Default::default()))
//!         .await?;
//! }
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "dynamic-rpc")]
/// Dynamic method/value RPC error types.
pub mod error;
/// Transport abstraction for RPC communication.
pub mod transport;

#[cfg(all(feature = "dynamic-rpc", target_arch = "wasm32"))]
/// WASM RPC client implementation.
pub mod wasm_client;

// Dispatcher is native-only (uses tracing)
#[cfg(all(feature = "dynamic-rpc", not(target_arch = "wasm32")))]
/// Native RPC dispatcher.
pub mod dispatcher;

#[cfg(all(feature = "dynamic-rpc", not(target_arch = "wasm32")))]
/// Native in-process transport.
pub mod local_transport;

// Re-exports
#[cfg(feature = "dynamic-rpc")]
/// RPC error type facade.
pub use error::RpcError;
#[cfg(feature = "dynamic-rpc")]
/// Transport trait facade.
pub use transport::RpcTransport;
/// Closed typed transport trait facade.
pub use transport::TypedRpcTransport;

#[cfg(all(feature = "dynamic-rpc", target_arch = "wasm32"))]
/// WASM client facade.
pub use wasm_client::WasmRpcClient;

#[cfg(all(feature = "dynamic-rpc", not(target_arch = "wasm32")))]
/// Native dispatcher facade.
pub use dispatcher::RpcDispatcher;

#[cfg(all(feature = "dynamic-rpc", not(target_arch = "wasm32")))]
/// Native transport facade.
pub use local_transport::LocalRpcTransport;

#[cfg(test)]
mod tests {
    const LIB_SRC: &str = include_str!("lib.rs");
    const TRANSPORT_SRC: &str = include_str!("transport.rs");

    fn public_lib_src() -> &'static str {
        LIB_SRC
            .split("#[cfg(test)]")
            .next()
            .expect("lib source before tests")
    }

    #[test]
    fn test_crate_docs_and_dispatch() {
        for needle in [
            "transport-focused",
            "request dispatch",
            "local testing helpers",
        ] {
            assert!(
                public_lib_src().contains(needle),
                "expected crate docs to mention `{needle}`",
            );
        }
    }

    #[test]
    fn test_crate_docs_network_concerns() {
        for needle in [
            "peer identity",
            "authentication",
            "retry policy",
            "connection lifecycle",
        ] {
            assert!(
                public_lib_src().contains(needle) || TRANSPORT_SRC.contains(needle),
                "expected RPC docs to reserve `{needle}` as an external concern",
            );
        }
    }
}
