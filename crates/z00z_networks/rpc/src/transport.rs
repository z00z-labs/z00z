//! RPC transport abstraction for different execution environments.
//!
//! This module deliberately owns only the request/response carriage contract.
//! Peer identity, authentication, retry policy, and connection lifecycle are
//! adjacent concerns that higher-level network crates must define around this
//! trait instead of extending it ad hoc from inside the RPC crate.

#[cfg(feature = "dynamic-rpc")]
use crate::error::RpcError;
#[cfg(feature = "dynamic-rpc")]
use async_trait::async_trait;
#[cfg(feature = "dynamic-rpc")]
use z00z_utils::codec::Value;

/// Transport layer for RPC communication.
///
/// Different implementations:
/// - `LocalRpc`: In-process server (Desktop/TUI)
/// - `WasmRpc`: WebSocket client (Browser)
/// - `HttpRpc`: HTTP client (Future: remote backend)
///
/// This trait is intentionally narrow: it carries one method call and returns
/// one typed response or transport error. It does not own peer identity,
/// authentication, retry policy, or connection lifecycle state.
///
/// # WASM Compatibility
///
/// Uses `#[async_trait(?Send)]` to support WASM targets where
/// futures are `!Send` (single-threaded environment).
#[cfg(feature = "dynamic-rpc")]
#[async_trait(?Send)] // ?Send for WASM compatibility
pub trait RpcTransport {
    /// Send RPC request and receive response
    async fn call(&self, method: &str, params: Value) -> Result<Value, RpcError>;
}

/// Transport contract for a closed typed RPC protocol.
///
/// This additive facade lets application protocols reuse the project-owned
/// transport boundary without converting closed request and response types into
/// generic method strings or dynamic values. Protocol schemas, authorization,
/// deadlines, and retry policy remain owned by the consuming application.
pub trait TypedRpcTransport<Request> {
    /// Typed response returned by the protocol adapter.
    type Response;
    /// Protocol- or transport-specific failure returned to the caller.
    type Error;

    /// Exchange one closed typed request for its paired typed response.
    fn exchange(&mut self, request: Request) -> Result<Self::Response, Self::Error>;
}

// Allow boxed transports to be used transparently as transports.
#[cfg(feature = "dynamic-rpc")]
#[async_trait(?Send)]
impl<T> RpcTransport for Box<T>
where
    T: RpcTransport + ?Sized,
{
    async fn call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        (**self).call(method, params).await
    }
}

impl<Request, T> TypedRpcTransport<Request> for Box<T>
where
    T: TypedRpcTransport<Request> + ?Sized,
{
    type Response = T::Response;
    type Error = T::Error;

    fn exchange(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
        (**self).exchange(request)
    }
}

#[cfg(test)]
mod tests {
    use super::TypedRpcTransport;

    struct Echo;

    impl TypedRpcTransport<u8> for Echo {
        type Response = u8;
        type Error = core::convert::Infallible;

        fn exchange(&mut self, request: u8) -> Result<Self::Response, Self::Error> {
            Ok(request)
        }
    }

    #[test]
    fn typed_transport_preserves_closed_request_and_response_types() {
        assert_eq!(Echo.exchange(7), Ok(7));
    }
}
