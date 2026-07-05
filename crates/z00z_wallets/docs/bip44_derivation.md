# BIP-44 Derivation Examples (Z00Z)

This document is the practical companion to
`crates/z00z_wallets/docs/KEYS-Bip44-UserGuide.md`.

The live wallet supports only the canonical Z00Z BIP-44 path:

```text
m/44'/1337'/account'/change/address_index
```

## Basic examples

External payment branch:

```text
m/44'/1337'/0'/0/0
m/44'/1337'/0'/0/1
```

Internal/change branch:

```text
m/44'/1337'/0'/1/0
```

Separate key-space branch:

```text
m/44'/1337'/2'/0/10
```

Spend/view companion paths:

```text
spend: m/44'/1337'/0'/0/5
view:  m/44'/1337'/100000'/0/5
```

## Canonical invariants

- `purpose` is fixed to `44'`
- `coin_type` is fixed to `1337'`
- `account` must be hardened
- `change` must be `0` or `1` and must stay non-hardened
- `address_index` must stay non-hardened

The parser and service path reject invalid hardening, the wrong coin type,
invalid `change`, and hardened leaf indices. Helper APIs add a paired
spend/view namespace on top:

- normal spend helper accounts are `0..100000`
- view-key companion accounts are `100000..200000`
- accounts `>= 200000` can parse when the BIP-44 shape is otherwise valid, but
  they are outside the current spend/view companion helpers

## Security notes

Treat the following as secret material:

- BIP-39 mnemonic words
- BIP-39 seed bytes
- BIP-32 extended private key material
- BIP-32 chain code
- transient spend-side secret keys

Safe to expose in ordinary logs and user-facing responses:

- canonical path strings
- public keys
- receiver-facing exported material

The Ristretto bridge is deterministic and chain-separated, but it is not a
generic Bitcoin or Ethereum address-compatibility layer.

## Code-backed verification anchors

The current repository verifies the derivation model with these tests:

- `crates/z00z_wallets/tests/test_bip44.rs`
  Verifies canonical path behavior, restart stability, and BIP-39 seed
  determinism for the standard `abandon ... about` test mnemonic.
- `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`
  Verifies repeatable results through `wallet.key.derive_receiver`.
- `crates/z00z_wallets/src/key/test_bip44_manager_suite.rs`
  Verifies manager invariants and canonical leaf derivation.
- `crates/z00z_wallets/src/key/test_bip44_manager_entropy_suite.rs`
  Verifies weak-seed rejection and entropy guardrails.

This document intentionally does not duplicate raw vector blobs beyond what the
code itself anchors. If a seed vector matters for correctness, the canonical
source is the test or implementation file that enforces it.

## Key-derivation flow

```text
BIP-39 mnemonic + passphrase
    ↓
BIP-39 seed (64 bytes)
    ↓
BIP-32 master key
    ↓
BIP-44 path: m/44'/1337'/account'/change/address_index
    ↓
leaf private key
    ↓
Ristretto bridge (chain- and path-separated)
    ↓
receiver-facing public key material
```

## References

- [Z00Z BIP-44 User Guide](./KEYS-Bip44-UserGuide.md)
- [KEYS-GUIDE.md](./KEYS-GUIDE.md)
- [KEYS-DERIVATION.md](./KEYS-DERIVATION.md)
