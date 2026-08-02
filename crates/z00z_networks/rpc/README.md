# z00z_networks_rpc

This crate is the transport-focused RPC layer for Z00Z components.

## Boundary

- It owns request dispatch, transport adaptation, and local testing helpers.
- It does not own peer identity, authentication, retry policy, or connection
  lifecycle.
- Wallets, rollup nodes, and future overlays may compose these RPC boundaries,
  but they must keep higher-level network policy outside this crate.

## Canonical surface

- Default feature `dynamic-rpc` preserves the existing method/value JSON-RPC
  surface for current wallet, simulator, native, and WASM consumers.
- Closed typed consumers use `default-features = false`; that graph exposes only
  `TypedRpcTransport` and does not pull the JSON/WebSocket/server stack.
- `RpcTransport` defines the transport abstraction.
- `TypedRpcTransport` lets closed typed protocols reuse that abstraction
  without exposing generic method strings or dynamic values.
- `RpcDispatcher` owns method routing on native targets.
- `LocalRpcTransport` provides in-process testing transport.
- `WasmRpcClient` provides browser-facing RPC transport.
