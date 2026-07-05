# Current Derivation Trace

This note tracks the current live path behind `wallet.key.derive_receiver`.
It is intentionally service-first and omits legacy path aliases.

## 📍 Canonical RPC flow

```text
RPC request: wallet.key.derive_receiver
    ↓
rpc/key_rpc_server.rs::KeyRpcServer::derive_receiver()
    ↓
rpc/key_rpc_impl.rs::KeyRpcImpl::verify_no_touch_cap()
    ↓
WalletService::verify_session_no_touch()
    ↓
rpc/key_rpc_server_derive.rs::KeyRpcImpl::derive_receiver_checked()
    ↓
WalletService::key_derive_rate_limit_precheck()
    limit: 20
    ↓
String::parse::<Bip44Path>()
    ↓
WalletService::derive_public_key_for_path()
      implemented in services/wallet_session_derivation.rs
    ↓
WalletService::get_create_wallet_receiver_deriver()
    ↓
WalletService::create_receiver_deriver_state()
    ↓
key::KeyManagerImpl::init_from_seed()
    ↓
key::Bip44KeyManager::new()
    ↓
receiver::ReceiverManagerImpl::derive_spend_key()
    ↓
key::KeyManagerImpl::derive_key()
    ↓
key::Bip44KeyManager::derive_address_key_for_path()
    ↓
key::RistrettoBridge::to_ristretto_key()
    ↓
RuntimeDeriveReceiverResponse { public_key, path }
```

## 🧭 What this flow guarantees

- The path string is parsed and validated before derivation.
- No-touch session validation and wallet-state checks happen before key
  exposure.
- The RPC method applies the current key-derive precheck limit before it asks
  the receiver manager for a key.
- The service returns public bytes plus a canonical path string, not a secret.
- The receiver-manager layer is the current cache-aware boundary above the key
  manager.

## 📚 Source pointers

- `crates/z00z_wallets/src/rpc/key_rpc_server_derive.rs`
- `crates/z00z_wallets/src/services/wallet_session_derivation.rs`
- `crates/z00z_wallets/src/receiver/receiver_manager_impl_trait_impl.rs`
- `crates/z00z_wallets/src/key/manager_core.rs`
- `crates/z00z_wallets/src/key/bip32.rs`
- `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`
