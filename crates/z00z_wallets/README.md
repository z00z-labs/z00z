# Z00Z Wallets

Privacy-focused hierarchical deterministic wallet for Z00Z blockchain.

## 🎯 Current Status

**Branch:** `z00z-wallet`  
**Version:** 0.1.0  
**Architecture:** Clean Architecture / Hexagonal Architecture with RPC layer

## 📦 Phase 059 Object Model

The live wallet surface now carries one typed object inventory instead of an
asset-only worldview.

- Assets remain the only spendable cash projection.
- Vouchers are conditional claims with explicit lifecycle and validator-checked
  redemption paths.
- Rights are authority inventory and contribute zero to spendable balance.
- Unknown-policy objects remain in durable quarantine until the relevant policy
  descriptor is available and accepted by the existing rules.

The canonical public object RPC namespace is `wallet.object.*`. Existing
`wallet.asset.*` methods stay cash-only and must not be overloaded for
voucher/right semantics.

## 🔑 Live Object Surfaces

- `ObjectInventoryStore` and the wallet owned-object payloads provide one
  inventory facade for assets, vouchers, and rights.
- `wallet_asset_store()` remains the asset-only spendable-value authority.
- `wallet.object.list`, `wallet.object.preview`, `wallet.object.build`, and the
  voucher/right lifecycle wrappers expose typed object actions without turning
  those objects into cash inputs.
- Backup/export/import flows roundtrip typed voucher/right payloads through the
  canonical encrypted backup packet and reject checksum or wallet-id drift.

## 🚫 Cash Boundary

- Voucher ids must not be accepted by asset send/receive/build surfaces.
- Right ids must not be accepted as value inputs.
- Unknown-policy vouchers/rights must not appear as spendable balance after
  restart, restore, scan, or RPC projection.

---

## 🏗️ Architecture

### Core Principles:
- **ONE SOURCE OF TRUTH** - Uses `z00z_utils` abstractions for I/O, time, config
- **TRAIT-BASED DEPENDENCY INJECTION** - All components injectable via traits
- **NO ACCOUNTS** - Bitcoin-style Asset model with HD keys
- **VENDOR ISOLATION** - No modifications to tari-wallet (read-only reference)
- **RPC-BASED COMMUNICATION** - Clean boundary between UI and business logic

### Directory Structure:

```text
crates/z00z_wallets/
├── src/
│   ├── core/                    # ✅ Domain facades (`receiver`, `key`, `tx`, `wallet`)
│   ├── adapters/                # ✅ External communication
│   │   └── rpc/                 # JSON-RPC 2.0 protocol layer
│   │       ├── methods/         # RPC method handlers
│   │       ├── types/           # Request/Response DTOs
│   │       └── error_mapping.rs # Error conversion
│   │
│   ├── db/                      # ✅ Native `.wlt` facade and RedB backends
│   ├── services/                # ✅ Wallet orchestration facades
│   │
│   ├── wallet_worker.rs         # ✅ WASM Web Worker entry point
│   │
├── www/                         # 🌐 Web UI
│   ├── index.html               # WASM integration
│   └── pkg/                     # Built WASM output
│
└── scripts/
    ├── build_wasm.sh            # WASM build script
    └── serve_wasm.sh            # HTTP test server
```

---

## 🚀 Quick Start

### Prerequisites:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install wasm-opt (optional, for optimization)
# On Ubuntu/Debian:
sudo apt install binaryen

# On macOS:
brew install binaryen
```

---

## 🔧 Development

### Check Compilation:

```bash
# Native build
cd crates/z00z_wallets
cargo check

# WASM build
cargo check --target wasm32-unknown-unknown --features wasm

# Or use script:
../../scripts/test_wallet_wasm.sh check
```

### Build WASM:

```bash
# Development build (faster, with debug symbols)
./scripts/build_wasm.sh dev

# Production build (optimized, smaller size)
./scripts/build_wasm.sh prod
```

### Test in Browser:

```bash
# Build + serve (dev mode)
../../scripts/test_wallet_wasm.sh

# Or production mode:
../../scripts/test_wallet_wasm.sh prod

# Then open: http://localhost:8000
```

---

## 📋 Features

### ✅ Implemented:
- **Core Traits:** KeyManager, AssetSelector, TransactionBuilder, SettlementStore
- **HD Key Derivation:** BIP32-style hierarchical keys (m/44'/0'/chain/index)
- **Asset Selection:** Privacy-preserving asset commitment selection
- **Transaction Building:** Confidential transactions with range proofs
- **Native Asset Persistence:** Asset tracking for native wallets
- **Typed Object Inventory:** additive owned-object persistence for Assets,
  Vouchers, and Rights behind one inventory facade
- **Object RPC Namespace:** `wallet.object.*` list/preview/build/lifecycle
  methods
- **Durable Quarantine:** unknown-policy typed objects stay non-spendable until
  explicitly accepted by existing rules
- **RPC Protocol:** JSON-RPC 2.0 layer
- **WASM Worker:** Browser integration ready
- **Receiver Material Derivation:** Deterministic receiver keys, receiver-card export, and payment-request support

### Preferred Facades

- Use `z00z_wallets::db::{WltSession, ScanStatePayload}` for wallet-store boundary types.
- Use `z00z_wallets::services::{RateLimitPrecheck, WalletService}` for wallet orchestration and rate-limit integration points.
- Use `z00z_wallets::receiver::{ReceiverManagerImpl, DEFAULT_CACHE_SIZE, MAX_CACHE_SIZE}` for receiver-cache integration points.
- Use `z00z_wallets::key::{Bip44Path, MnemonicLanguage, SeedPhrase24}` for wallet key and seed flows.
- Use `z00z_wallets::tx::{ClaimTxVerifier, TxVerifier}` for transaction verification entrypoints.

Internal split files such as `redb_store/mod.rs`, `wallet_service.rs`, `receiver_manager_impl.rs`, `bip32.rs`, and `tx_verifier.rs` remain implementation details behind these facades.

Wallet encryption helpers live under `z00z_wallets::security::encryption`; they are not part of the `services` facade.

**Receiver material** is derived deterministically from the same BIP-44 spend path. Use `ReceiverManager::derive_wallet_keys()` to derive the receiver keys, then export a `ReceiverCard` or create a `PaymentRequest` as needed. Tari-style recipient derivation is no longer part of the live wallet API.

### 🚧 In Progress

- **Phase 2:** Business logic implementation behind the wallet service facades
- **Browser UI:** Full web interface

### 📋 Planned

- **Phase 3:** Network integration (OnionNet, P2P)
- **Phase 4:** Advanced features (staking, swaps, multi-sig)

---

## 🛡️ Seed Phrase Security Notes

📌 **Mixed-script detection is test-only tooling.**

📌 **BIP-39 standard mnemonics are ASCII-only per language.** Production validation relies on `bip39::Mnemonic::parse_in(...)` and the upstream wordlists.

📌 **Homoglyph protection verifies wordlist quality, not user input.** Tests include mixed-script / confusable samples to ensure they do not accidentally match any supported wordlist.

---

## 🔐 Logging & Diagnostics (Privacy)

⚠️ **Wallet logs are sensitive.** Logging derivation paths, receiver handles, or public keys can enable wallet fingerprinting and correlation.

✅ **Eviction diagnostics are disabled by default.** The wallet receiver-cache eviction logging/persistence is compile-time gated and not available in `--release` builds.

🚫 **Never enable debug logging features in production.** The `verbose-logging` and `eviction-logs` features are for development/testing only.

📌 **Details:** see [SECURITY.md](SECURITY.md).

---

## 🔐 Argon2 Parameter Guidance

📌 The wallet uses Argon2id for password-based key derivation. When parameters come from untrusted persisted metadata (e.g., `.wlt` files), the wallet enforces hard caps and rejects configurations that are likely to cause resource exhaustion.

| Preset | Use Case | Time (rough) | Memory | Security |
|--------|----------|--------------|--------|----------|
| `interactive()` | Dev / low-friction wallets | ~1s | 64 MiB | Lower |
| `moderate()` | General use | ~2–3s | 128 MiB | High |
| `strong()` | Cold storage / high-value | ~5–10s | 256 MiB (desktop) / 64 MiB (WASM) | Maximum |

⚠️ **Warning:** Malicious `.wlt` files may claim excessive Argon2 parameters to cause a denial-of-service during open/unlock. The wallet validates these parameters before any expensive KDF work is performed.

📌 **Note:** Time values are a conservative heuristic and will vary by hardware.

## 🔐 Backup Header Privacy

📌 **Backup metadata is versioned.** Format `v4` keeps `wallet_id` in the public header for local discovery/filtering, but redacts `network` from the unauthenticated header.

📌 **The effective restore identity now lives inside the encrypted payload.** Format `v4` carries both `network` and `chain` under authenticated encryption so restore does not silently rebind a wallet to the current runtime identity. Importers keep backward compatibility when reading older `v1`/`v2` files, but restore of chain-less export-pack backups is rejected instead of guessing.

📌 **`WalletExportPack` is not a public serialization format.** It may contain a plaintext mnemonic in memory, but public backup/export flows must only persist it inside authenticated encryption.

---

## 🧪 Testing

### Run Tests:

```bash
# All tests
cargo test --all

# Specific module
cargo test --package z00z_wallets core::impls

# With output
cargo test -- --nocapture
```

### Integration Tests:

```bash
# Native integration tests
cargo test --test '*'

# WASM tests (requires wasm-pack)
wasm-pack test --headless --chrome
```

---

## 📚 Documentation

### Generate Docs:

```bash
cargo doc --no-deps --open
```

### Key Documentation:

- [Z00Z_WALLETS_FOUNDATION.md](Z00Z_WALLETS_FOUNDATION.md) - Implementation roadmap
- [BUILDING_WASM.md](BUILDING_WASM.md) - WASM build guide

### Architecture Docs:

- [Z00Z_DESIGN_FOUNDATION.md](../../Z00Z_DESIGN_FOUNDATION.md) - Design patterns and abstraction principles
- [Tari Crypto Integration](../../.github/requirements/Tari-Crypto-Integration-Z00Z.md) - Crypto components

---

## 🔑 Key Components

### Core Traits:

```rust,ignore
// HD key derivation
pub trait KeyManager {
    fn derive_key(&self, chain: u32, index: u32) -> Result<RistrettoPublicKey>;
}

// Asset selection for transactions
pub trait AssetSelector {
    fn select_assets(&self, available: &[Asset], target: Amount) 
        -> Result<Selection>;
}

// Transaction construction
pub trait TransactionBuilder {
    fn build_transaction(&mut self, selection: Selection) 
        -> Result<Transaction>;
}

// Asset persistence
pub trait SettlementStore {
    fn save_asset(&mut self, asset: &Asset) -> Result<()>;
    fn load_asset(&self, serial_id: u32) -> Result<Asset>;
}
```

### RPC Methods:

```json
// List wallets
{"jsonrpc":"2.0","method":"wallet.list","params":{},"id":1}

// Create wallet
{"jsonrpc":"2.0","method":"wallet.create","params":{"name":"MyWallet","password":"***"},"id":2}

// Unlock wallet
{"jsonrpc":"2.0","method":"wallet.unlock","params":{"wallet_id":"...","password":"***"},"id":3}
```

---

## 🐛 Troubleshooting

### WASM Build Issues:

**Problem:** `wasm-pack not found`
```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

**Problem:** `wasm32-unknown-unknown target not installed`
```bash
rustup target add wasm32-unknown-unknown
```

**Problem:** Large WASM file size
```bash
# Install wasm-opt
sudo apt install binaryen  # Ubuntu/Debian
brew install binaryen       # macOS

# Use production build
./scripts/build_wasm.sh prod
```

### Runtime Issues:

**Problem:** Module not found in browser
- Check browser console for CORS errors
- Ensure HTTP server is running (`./scripts/serve_wasm.sh`)
- Verify WASM file exists in `www/pkg/`

**Problem:** RPC method not found
- Check method name (case-sensitive: `wallet.list`)
- Verify feature flags: `--features wasm`

---

## 📊 Build Statistics

| Build Type | Size (WASM) | Build Time | Optimization |
|------------|-------------|------------|--------------|
| Dev | 195 KB | ~10s | None |
| Prod | 175 KB | ~45s | wasm-opt -Oz |

---

## 🤝 Contributing

### Code Style:
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Run `cargo fmt` before commits
- Zero `cargo clippy` warnings
- All tests pass

### Commit Messages:

```text
type(scope): Brief description

Longer explanation if needed.

- Bullet points for details
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`

---

## 📝 License

See [LICENSE](../../LICENSE) file in project root.

---

## 🔗 Links

- **Repository:** [z00z-labs/z00z](https://github.com/z00z-labs/z00z)
- **PR:** [#12](https://github.com/z00z-labs/z00z/pull/12)
- **Branch:** `z00z-wallet`

---

**Last Updated:** 2025-12-21  
**Maintainer:** Z00Z Development Team
