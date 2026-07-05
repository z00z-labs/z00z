<!-- markdownlint-disable-file MD022 MD031 MD032 MD036 MD040 MD060 -->

# 📌 Z00Z BIP-44 User Guide

📌 This document explains how Z00Z uses a strict BIP-44 derivation path for deterministic key derivation.

📌 It is a **user-facing guide**: it describes intent, meaning, and policy rules.

📌 It does **not** change the implementation.

---

## 🎯 BIP-44 Path Structure

📌 Z00Z uses the standard 5-component BIP-44 path shape:

```text
m / purpose' / coin_type' / account' / change / address_index
```

📌 In Z00Z code you may see `asset_type` used where BIP-44 terminology would say `coin_type`.

📌 In this guide, **coin type** means the SLIP-0044 coin type (network identifier), not the asset class.

---

## ⚙️ Component Meanings

📌 Each component has a standard BIP-44 meaning, and Z00Z adds a project-specific interpretation for `account`.

| BIP-44 name | Z00Z meaning | Hardened | Notes |
|---|---:|---:|---|
| `purpose` | 44 | Yes (`44'`) | Fixed to BIP-44 purpose. |
| `coin_type` | 1337 | Yes (`1337'`) | Fixed to Z00Z network id (SLIP-0044 style). |
| `account` | **Key Space ID** | Yes (`account'`) | Z00Z uses `account` as a **key space** namespace (convention). |
| `change` | external/internal branch | No | `0` = external, `1` = internal. |
| `address_index` | leaf index | No | Monotonic counter within a branch. |

📌 If you are reading the implementation, the mapping is:

- `purpose` → fixed `44'`
- `asset_type` (code) → `coin_type` (spec) → fixed `1337'`
- `account` (code/spec) → **key_space_id** (Z00Z convention)

---

## 🔑 “Key Space” Convention (Account Mapping)

📌 Z00Z treats `account'` as a **Key Space ID** to separate key material by asset policy/UX grouping.

📌 This is a **convention**, not a protocol requirement, and it is separate from
the spend/view helper namespace described below.

📌 Recommended mapping (reserved accounts):

| Key Space | `account` value | Purpose |
|---|---:|---|
| Coin | `0` | Base coin keys (default payments / main value transfers). |
| Token | `1` | Fungible token keys (ERC-20 style assets). |
| NFT | `2` | NFT keys (unique assets). |
| Gas/Fee (optional) | `3` | Fee keys, only if policy requires separation from Coin. |
| Reserved | `>= 4` | Reserved for future use (governance/system/other). |

📌 If gas is always paid in the base coin and policy does not require separation, treat gas as **Coin key space** and keep `account=0`.

📌 If legal/policy requirements demand separation, use a dedicated space (e.g., `account=3`) and document it as mandatory.

---

## 👁️ Spend/View Account Namespace

📌 Current helper APIs reserve an account offset for paired spend/view paths:

| Namespace | Account range | Meaning |
|---|---:|---|
| Spend helper accounts | `0..100000` | Ordinary spend/receive paths. |
| View helper accounts | `100000..200000` | Companion view-key paths derived by adding `100000`. |
| Out-of-band accounts | `>= 200000` | BIP-44-valid if the index fits, but outside the current paired helper namespace. |

📌 Example companion pair:

```text
spend: m/44'/1337'/0'/0/5
view:  m/44'/1337'/100000'/0/5
```

📌 `Bip44Path::new_z00z(...)` rejects direct construction inside
`100000..200000`, because that range is reserved for view-key companions.

📌 `to_view_key_path()` maps a spend helper account to the companion view
account. `to_spend_key_path()` maps it back.

---

## ✅ Valid Ranges and Allowed Values

📌 Z00Z enforces strict validation rules:

- `change` MUST be `0` or `1`.
- `purpose`, `coin_type`, and `account` MUST be hardened.
- `change` and `address_index` MUST be non-hardened.

📌 Practical ranges you can use:

- `account` (BIP-44 parser range): any integer `0 <= account < 2^31`.
- `account` (current spend helper namespace): prefer `0..100000`.
- `address_index`: any integer $0 \le address\_index < 2^{31}$.
- `change`: `0` or `1` only.

📌 Use small contiguous ranges (`0..N`) for UX simplicity and predictable
backups, and do not allocate product key spaces inside `100000..200000`.

---

## 🧭 Examples

📌 Coin keys (external):

```text
m/44'/1337'/0'/0/0
m/44'/1337'/0'/0/1
```

📌 Coin keys (internal/change):

```text
m/44'/1337'/0'/1/0
```

📌 Token keys (external):

```text
m/44'/1337'/1'/0/0
```

📌 NFT keys (external):

```text
m/44'/1337'/2'/0/0
```

📌 Optional dedicated gas keys (external):

```text
m/44'/1337'/3'/0/0
```

---

## 🧩 “No addresses” vs “Public keys”

📌 In many code paths, “derive address” actually means **derive a public key for a path**.

📌 The same derived public key can later be encoded or used differently (address encoding, scripts, ownership proofs), depending on higher layers.

📌 For UI and policy documents, it is often clearer to say:

- “derive a key” / “derive a public key”
- “derive destination key”

📌 Treat the derivation path as a **Key Derivation Path**, not necessarily as an “Address Path”.

---

## ⚠️ Terminology Notes (Avoid Confusion)

📌 Why `asset_type` is confusing: in the code it refers to the **SLIP-0044 coin type**, not to Z00Z `AssetClass`.

📌 Z00Z `AssetClass` (Coin/Token/Nft/Void) is part of asset definitions in `z00z_core`, and should not be conflated with BIP-44 coin type.

📌 Recommended human terms:

- Use **coin type** for `1337'`.
- Use **key space** for the semantic meaning of `account'`.

---

## 🔒 Policy and Compliance Guidance

📌 If your product has legal/policy constraints (e.g., segregating keys by asset class), you should treat the Key Space mapping as **mandatory**.

📌 Recommended compliance stance:

- Key Space mapping MUST be stable across versions.
- Non-reserved accounts MUST NOT be used implicitly.
- Any new key space MUST be explicitly documented and versioned.

📌 This prevents “silent key mixing” across asset categories, which can be a compliance and audit risk.

---

## ✅ Operational Recommendations

📌 Keep counters monotonic per (key_space/account, change) and persist them in wallet state.

📌 Do not reuse the same leaf index for different meanings within the same key space.

📌 If you need per-asset (per `AssetDefinition.id`) isolation, prefer a higher-level mapping layer rather than overloading BIP-44 fields.

---

## ❓FAQ

### ❓ Why is `coin_type` fixed to 1337?

📌 It is the Z00Z network identifier in the BIP-44/SLIP-0044 “coin type” slot.

### ❓ Why not rename `account` to `asset_class`?

📌 Because `account` is a standard BIP-44 term.

📌 Renaming it in API vocabulary tends to confuse integrators and auditors who already know BIP-44.

📌 Use “Key Space” as a semantic alias in docs/UX instead.

### ❓ Why not use “domain” terminology?

📌 Z00Z already uses “domain” heavily for cryptographic domain separation.

📌 Using “key domain” for BIP-44 `account` tends to collide conceptually with hash domains.

---

📌 Last updated: 2026-07-04
