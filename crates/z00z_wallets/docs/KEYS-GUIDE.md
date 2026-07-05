<!-- markdownlint-disable-file MD022 MD031 MD032 MD036 MD040 MD060 -->

# Z00Z Keys User Guide

Date: 2026-07-04
Scope: `crates/z00z_wallets/src/key`

This guide summarizes the live key subsystem used by `z00z_wallets`.
Every statement below is intended to match current code and current tests.

---

## 🎯 What the key subsystem does

The wallet key stack starts from BIP-39 seed material, derives child keys with
BIP-32/BIP-44 rules, and maps leaf key material into Z00Z Ristretto keys.

In practice this gives the wallet three important properties:

- deterministic recovery: the same seed and path produce the same key again;
- path separation: different BIP-44 paths produce different keys;
- chain separation: the same seed and path produce different keys on different
  `ChainType` values.

The public receive surface is not a legacy string-address API.
The live RPC path `wallet.key.derive_receiver` returns a canonical path plus a
receiver-facing public key.

---

## 🔑 Canonical derivation path

Z00Z uses this canonical BIP-44 shape:

```text
m/44'/1337'/account'/change/address_index
```

Current enforced meaning:

- `44'`: fixed BIP-44 purpose
- `1337'`: fixed Z00Z coin type slot
- `account'`: hardened key-space branch
- `change`: `0` for external, `1` for internal/change
- `address_index`: non-hardened leaf index

The crate rejects non-canonical paths such as:

- wrong coin type
- hardened `change`
- hardened `address_index`
- `change` values outside `{0, 1}`

For code-backed examples, see:

- [KEYS-Bip44-UserGuide.md](./KEYS-Bip44-UserGuide.md)
- [bip44_derivation.md](./bip44_derivation.md)

Current helper APIs also reserve account namespace `100000..200000` for paired
view-key companion paths. Normal spend helper accounts are `0..100000`.
Accounts `>= 200000` can still parse when the BIP-44 shape is valid, but they
are outside the current spend/view companion helpers.

---

## 🧩 Main modules

The live key implementation is split across these files:

- `bip32.rs`: facade for the BIP-32/BIP-44 implementation.
- `bip32_constants.rs`: Z00Z constants such as purpose `44`, coin type `1337`,
  and the view-key account offset.
- `bip32_path*.rs`: path value types, parsing, builders, validation, serde, and
  path errors.
- `bip32_key_deriver.rs`: BIP-32 child-key derivation support.
- `bip32_ristretto_bridge.rs`: chain- and path-separated mapping from BIP-32
  leaf material into Z00Z Ristretto key material.
- `bip44_manager.rs`: canonical leaf derivation for explicit `Bip44Path`
  values.
- `manager_core.rs` and `manager_impl*.rs`: `KeyManagerImpl`, public-key cache,
  gap checks, signing, state handling, and transient-secret behavior.
- `manager_redb*.rs`: password-derived RedB wallet storage keys. This is
  storage crypto, not receiver-address derivation.
- `seed*.rs`: BIP-39 seed handling, mnemonic normalization, entropy checks,
  seed backup format, and seed cipher containers.
- `receiver_keys*.rs`: receiver key material, identity helpers, bundle support,
  and receiver-card export.

The receiver-facing service flow continues outside `src/key/`:

- `services/wallet_session_derivation.rs`
- `receiver/receiver_manager_impl*.rs`
- `rpc/key_rpc_server_derive.rs`

---

## 🔒 Security invariants

The current code and tests enforce these rules:

- Seed material is the root secret. If an attacker gets the BIP-39 seed, they
  can reproduce the full derivation tree.
- Private keys are derived transiently. The normal public API does not keep
  spend secrets cached as long-lived wallet state.
- Public keys may be cached for performance. Secret keys must not become a
  parallel cached authority plane.
- The BIP-32 chain code is sensitive and must be treated as secret material.
- The Ristretto bridge is domain-separated by chain and derivation path.
- RedB storage keys are derived separately from wallet-address keys.

Operationally safe to expose:

- canonical path strings
- public keys
- receiver cards after validation/export rules are applied

Operationally unsafe to expose:

- mnemonic words
- BIP-39 seed bytes
- BIP-32 extended private key material
- chain codes
- password-derived RedB master material

---

## ⚙️ Runtime behavior

### Public-key derivation

The live service path for public derivation is:

1. validate the session through the no-touch capability path
2. apply the current key-derive rate-limit precheck
3. parse and validate `Bip44Path`
4. resolve or build the per-wallet receiver deriver
5. derive the spend-side public key through the receiver manager
6. return the public key bytes plus the canonical path string

See [KEYS-DERIVATION.md](./KEYS-DERIVATION.md) for the current call trace.

### Caching

`KeyManagerImpl` caches derived public keys.
That cache exists to accelerate repeated UI and RPC requests.
The canonical correctness rule is still recomputation from seed plus path; the
cache is only a performance layer.

### Gap limit

The wallet tracks external and internal derivation progress separately.
This prevents unbounded forward derivation that would make recovery scanning
more expensive and less predictable.

### Signing

The signing path derives a transient secret for the requested path and computes
the Schnorr challenge through `compute_schnorr_challenge(...)`.
The steady-state contract is:

- derive secret on demand
- sign
- zeroize transient material

---

## 🧪 Code-backed anchors

These tests are the quickest way to verify the live key model:

- `crates/z00z_wallets/tests/test_bip44.rs`
  Covers canonical path behavior, BIP-39 seed determinism, and restart
  stability.
- `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`
  Covers the live `wallet.key.derive_receiver` RPC path.
- `crates/z00z_wallets/src/key/test_bip44_manager_suite.rs`
  Covers manager invariants and canonical path behavior inside the key module.
- `crates/z00z_wallets/src/key/test_bip44_manager_entropy_suite.rs`
  Covers entropy validation and weak-seed rejection.

---

## ❓ FAQ

### Are these keys interoperable with generic BIP-44 wallets?

Only partially.
The outer path shape is BIP-44, but the coin type is project-specific and the
leaf-to-Ristretto mapping is also project-specific.

### Why does the wallet not store every private key permanently?

Because deterministic derivation from seed is the canonical source of truth.
Persisting every derived private key would increase secret sprawl without adding
recovery value.

### Is `manager_redb.rs` part of address derivation?

No.
It derives storage-encryption material for `.wlt` persistence and remains
separate from the address/signing key tree.

### Does `wallet.key.derive_receiver` return a final address string?

No.
The live path returns a receiver-facing public key and a canonical BIP-44 path.
Address encoding and receiver-card workflows live at higher layers.
