<!-- markdownlint-disable-file MD022 MD031 MD032 MD036 MD040 MD060 -->

# Z00Z Wallet Guide

Date: 2026-07-04
Scope: `crates/z00z_wallets`

This guide documents the canonical wallet model that is live in code: `.wlt`
reopen, save, export, restore, tx-history sidecar handling, typed object
inventory, and the guarded profile/catalog anchors that tests keep aligned with
repo configs. Historical snapshot terminology is compatibility-only and must
not be treated as normal wallet authority.

## 📦 Typed Object Inventory

The live wallet inventory has three families:

| Family | Live payload | Store path | Spendability |
| --- | --- | --- | --- |
| Asset | `OwnedAssetPayload` | `wallet_asset_store()` / `OwnedAsset` object rows | Ordinary cash value when status and policy checks allow it. |
| Voucher | `OwnedVoucherPayload` | `object_inventory_store()` / `OwnedVoucher` object rows | Conditional claim; never ordinary cash until a valid redeem path finalizes. |
| Right | `OwnedRightPayload` | `object_inventory_store()` / `OwnedRight` object rows | Zero-value authority inventory; never ordinary cash. |

`WalletInventoryPayload` can project assets, vouchers, and rights together for
typed inventory views. `OwnedNonAssetPayload` is intentionally voucher/right
only, so non-asset writes stay off the ordinary cash asset path.

Unknown-policy objects remain in durable quarantine and are excluded from
spendable balance.

## 🧾 Phase 060 Catalog Anchors

The wallet does not expose a generic runtime profile registry. The code-backed
anchors are:

- `crates/z00z_core/configs/devnet_genesis_config.yaml` contains
  `wallet_profiles` rows for the catalog ids below.
- `crates/z00z_core/configs/devnet_rights_config.yaml` contains live right
  anchors such as `service_entitlement`, `data_access`, `validator_mandate`,
  `machine_compute_capability`, and `one_time_agent_action`.
- `crates/z00z_wallets/src/tx/spend_rules.rs` contains the wallet-local
  `validator_mandate_lock_v1` tag, payload commitment rule, and lock checks
  that remove locked rows from ordinary spend selection.

The phrase "Proposed Phase 060 catalog id" is retained below because tests use
it as a documentation/config alignment marker. It does not mean every row is a
standalone runtime wallet profile registry entry.

| Profile id | Phase 060 status | Code/config anchor | Live wallet effect |
| --- | --- | --- | --- |
| `fee_credit_v1` | Proposed Phase 060 catalog id | `devnet_genesis_config.yaml` wallet profile row; voucher inventory model | Voucher-family catalog row; must not appear as ordinary asset balance. |
| `service_entitlement_v1` | Proposed Phase 060 catalog id | `service_entitlement` right in `devnet_rights_config.yaml` | Right-family inventory row gated by policy availability. |
| `data_access_v1` | Proposed Phase 060 catalog id | `data_access` right in `devnet_rights_config.yaml` | Right-family inventory row for access-like authority; no cash visibility. |
| `agent_budget_v1` | Proposed Phase 060 catalog id | `machine_compute_capability` and `one_time_agent_action` rights in `devnet_rights_config.yaml` | Right-family bounded-action inventory; no ordinary balance effect. |
| `validator_mandate_lock_v1` | Proposed Phase 060 catalog id | `validator_mandate` right plus wallet-local spend rule in `spend_rules.rs` | Active matching rights exclude bound assets from ordinary spend selection. |
| `transferable_claim_v1` | Proposed Phase 060 catalog id | `devnet_genesis_config.yaml` wallet profile row; voucher inventory model | Voucher-family transfer/redeem catalog row; no cash visibility before redeem. |

## 🗂️ Projection Grammar

The catalog anchors above are projected through one wallet authority model:

- `wallet.object.*` is the typed inventory and package-authority namespace for
  voucher/right rows and wallet-visible object package flows.
- `wallet.asset.*` remains cash-only. It must not present vouchers or rights as
  spendable value and must not invent a second typed-object persistence story.
- `wallet_asset_store()` remains the only ordinary cash-persistence authority
  for asset rows. Non-asset rows stay on `WalletInventoryPayload` and the
  explicit `OwnedNonAssetPayload` voucher/right lanes.
- Any object whose policy availability is not `Available`, or that still
  requires manual review, remains in durable quarantine and is excluded from
  ordinary spendable balance.
- `.wlt` and `WalletExportPack` remain the only wallet-local authority surfaces.
  Backup and restore must round-trip `owned_assets` and `owned_objects` through
  the canonical encrypted bundle plus the explicit JSONL tx-history sidecar.
- Debug, preview, and forensic tooling may inspect the same state, but it must
  not create a second wallet database, a second export bundle, or a second
  spendability truth path.

## 🔒 `validator_mandate_lock_v1` Runtime Rule

- Identification: a right row must be `RightClass::ValidatorMandate`, policy
  availability `Available`, and carry the wallet-local
  `validator_mandate_lock_v1` label.
- Payload binding:
  `validator_mandate_lock_payload_commitment(...)` binds the locked asset id,
  locked amount, validity window, challenge window, `use_nonce`, transition,
  revocation, disclosure, and retention policy ids.
- Spend rule:
  an active matching right removes the bound asset row from ordinary spend
  selection. Tests cover both build and send rejection.
- Unlock readiness:
  `validator_mandate_lock_unlock_ready(...)` checks the right class and
  `valid_until`; expiry alone is not documented here as an automatic cash spend.
- Non-goals:
  no second wallet database, no second export contract, and no UI-only soft
  lock path.

## 🎯 Canonical Model

- `.wlt` is the canonical encrypted wallet database.
- `WalletProfilePayload` stores wallet profile metadata, verifier state,
  settings, and wallet lock state.
- `OwnedAssetPayload` stores wallet-owned asset state.
- `OwnedVoucherPayload` stores wallet-owned voucher state.
- `OwnedRightPayload` stores wallet-owned right state.
- `ScanStatePayload` stores the scan cursor and related receive progress.
- `KeysPayload`, `StealthMetaPayload`, and `TofuPinsPayload` store subsystem
  state that belongs inside the wallet boundary.
- Transaction history remains an explicit JSONL sidecar:
  `wallet_<stem>_tx_history.jsonl`.

## ⚙️ Normal Lifecycle

1. Create a wallet and initialize the `.wlt` file plus secret material.
2. Unlock the wallet and open a session-scoped `.wlt` handle.
3. Read `WalletProfilePayload` from `.wlt` and restore in-memory profile state.
4. Read live owned assets from `.wlt` object storage and install them into the
   service cache.
5. Keep scan state and tx history on their explicit planes instead of folding
   them into a single legacy blob.
6. Save by writing the updated wallet profile and any changed object payloads.
7. Export by building one canonical `WalletExportPack` and encrypting it.
8. Restore by validating the canonical pack, staging `.wlt`, and replaying the
   JSONL tx-history sidecar when present.

## 🔐 Persistence Boundaries

- Secrets stay in the dedicated secrets table.
- Profile, assets, scan state, and backup manifest data are encrypted object
  payloads inside `.wlt`.
- `recv_range(...)` and wallet-owned scanning remain the ownership authority.
- Export and restore use one canonical pack shape and one explicit tx-history
  sidecar plane.
- Debug helpers may inspect persisted state, but they must not reintroduce a
  second live authority model.
- Typed object inventory is additive: asset-only reopen remains valid, while
  vouchers/rights reuse the same `.wlt` object boundary instead of a second
  wallet database.

## 📦 Export And Restore Shape

The canonical `WalletExportPack` carries these live fields:

- `manifest`
- `wallet_profile`
- `owned_assets`
- `owned_objects`
- `scan_state`
- `stealth_meta`
- `tofu_pins`
- `keys`
- `tx_history_plane`
- `seed_phrase`
- `wallet_identity`

Normal wallet-state transfer must treat this explicit shape as the only live
bundle contract.

`export_wallet_payload` and `import_wallet_payload` move the canonical wallet
state pack only.

`create_backup` and `restore_backup_with_mode` are the surfaces that carry and
replay the explicit JSONL tx-history sidecar.

## 🕵️ Privacy And Disclosure Boundaries

- `tag16` prefilter and wallet-owned stealth scan detection are wallet-local
  receive primitives only. They prove receive classification, not transport
  anonymity.
- `WalletReveal::{Present, Redacted, Unavailable}` defines the public
  disclosure matrix for wallet-owned receive data. Public DTO, report, and log
  surfaces must keep memo plaintext, receiver secrets, blindings, output
  secrets, and private scan keys redacted.
- Backup metadata, package verify or import or export reports, and RPC logging
  summaries may expose only bounded public fields such as wallet id, package
  version, digests, counts, lifecycle, and status. They must not expose raw
  package bytes, session tokens, seed phrases, memo plaintext, receiver
  secrets, or encrypted payload internals.
- Stealth pack privacy is a wallet and crypto receive property. It is not an OnionNet claim and does not claim live transport anonymity. Phase 062 does not claim live transport anonymity.

## 🛰️ Typed RPC And Package Boundary

- `wallet.asset.*` remains cash-only.
- `wallet.object.*` is the typed object namespace for inventory, preview/build,
  and voucher/right lifecycle operations.
- Wallet package building binds policy descriptors, selected action, required
  rights, typed deltas, roots, and fee-support data through one shared runtime
  object package shape.
- Wallet builders must reject voucher-as-cash, right-as-value, out-of-scope
  rights, expired rights, revoked rights, consumed rights, and unknown-policy
  spend attempts.

## ✅ Bounded Object-Family Scope

- Included live scope: `RightLeaf`, `VoucherLeaf`, `RightClass`,
  `FeeEnvelope`, the object policy registry, wallet object inventory, validator
  fail-closed checks, deterministic local voucher/right scenarios, and
  cash/object separation proofs.
- Excluded scope: external chain trust tiers, linked liability, live
  cross-chain settlement, and any claim that vouchers or rights are ordinary
  wallet cash.
- Canonical evidence anchors:
  `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`,
  `crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs`,
  `crates/z00z_wallets/src/rpc/test_asset_impl.rs`, and
  `crates/z00z_wallets/tests/test_asset_import_security.rs`.
- Closure rule: `wallet.object.*` remains the typed object namespace, while
  `wallet.asset.*` remains the cash-only authority even when the shared
  inventory view projects asset rows beside vouchers or rights.

## 🧭 Key Runtime Surfaces

- `services/wallet_store_open_source.rs`
  unlocks and restores the profile-first live wallet state.
- `services/wallet_store_restore.rs`
  loads wallet profile bytes and owned assets from `.wlt`.
- `services/wallet_store_persistence_pack.rs`
  defines wallet file naming and the explicit JSONL history path.
- `services/wallet_store_transfer_import.rs`
  encrypts and decrypts the canonical export payload.
- `services/wallet_actions_backup.rs`
  stages and publishes canonical restore state.
- `redb_store/debug_export.rs`
  exports on-disk debug state from the actual `.wlt` tables and objects.

## 🧪 Test Anchors

The canonical path should stay anchored by tests that prove:

- profile-only reopen succeeds without any legacy snapshot dependency;
- save keeps owned assets on the explicit object path;
- export and restore operate on `WalletExportPack` plus JSONL history only;
- duplicate owned-asset payloads in a restore pack fail closed.

## 🚫 Compatibility Notes

- `WalletPersistenceState` is not the normal live save, reopen, export, or
  restore contract.
- Legacy snapshot wording is historical only.
- `claimed_assets` must not be described as a second normal authority plane for
  reopen or export.
- If compatibility decode surfaces remain anywhere, they must fail closed and
  stay outside the normal live flow.

## ⭐ Practical Guidance

- When adding new wallet state, prefer explicit `.wlt` payload objects and
  explicit indexes.
- When changing export or restore behavior, keep `WalletExportPack` and backup
  manifest validation aligned in the same patch.
- When changing tx history behavior, preserve the explicit JSONL sidecar until a
  separate planned migration lands.
- When updating docs or tests, describe only the canonical profile-first plus
  owned-asset plus scan-state plus JSONL history story.
