# Z00Z Wallet Persistence Overview

Date: 2026-07-04
Scope: `crates/z00z_wallets`

This document summarizes the current wallet persistence model.
It complements `WALLET-GUIDE.md` and `WLT-BREAKDOWN.md` and focuses on the
current code-backed storage boundaries rather than historical narrative.

## 🔑 Bottom line

The live wallet has two canonical local persistence planes:

- `wallet_<wallet_stem>.wlt`: the encrypted RedB-backed wallet container
- `wallet_<wallet_stem>_tx_history.jsonl`: the canonical append-only tx-history
  sidecar

`wallet_stem` is derived by hashing the wallet id with
`compute_wallet_file_id(...)` and taking the first 8 bytes as hex. It is not
the raw wallet id string.

The legacy JSON metadata helper may still exist for narrow compatibility flows,
but it is not the primary live authority for wallet state.

## 📦 Disk artifacts

### `.wlt`

The `.wlt` file is the authoritative wallet-state container.
It is opened through `WltStore` / `RedbWalletStore`, validated, unpacked into a
tmpfs-backed work file, and then written back atomically.

The live `.wlt` model stores:

- meta keys such as wallet id, schema version, KDF params, save sequence, and
  object pointers
- encrypted secrets such as the master key and main seed secret
- encrypted object payloads such as wallet profile, owned assets, owned objects,
  scan state, keys, and backup manifest data
- index tables and `index_manifest` for canonical lookup/update support

`owned_assets` remains the cash-only asset plane. Voucher and right rows are
stored as typed object inventory and exported through `owned_objects`.

### `wallet_<stem>_tx_history.jsonl`

The tx-history sidecar is the canonical live transaction history plane.
It is append-only JSONL and stores replayable rows with sequence numbers and
hash chaining.

The existence of tx indexes inside `.wlt` does not make `.wlt` the canonical
history journal.
The code still treats JSONL as the live tx-history artifact.

### `{wallet_id}.json`

`WalletStorageImpl` may still write a metadata helper JSON file.
That helper is real, but it is not the main create/open/restore boundary used
by the current wallet service flow.

### Encrypted backup container

`WalletExportPack` is the canonical wallet-transfer bundle shape.
The encrypted backup path may additionally carry forensic tx-history data,
including exact JSONL bytes.

## 🧱 Create, open, and restore boundaries

The current service model is:

1. create or discover the `.wlt` container
2. validate zstd header, schema markers, meta keys, and secret records
3. restore the live wallet profile and owned rows from `.wlt`
4. keep tx-history on the explicit JSONL plane
5. export and restore through the canonical `WalletExportPack`

Important consequence:

- `.wlt` is the canonical restoreable wallet-state boundary
- JSONL is the canonical tx-history boundary
- backup/export is the canonical portable bundle boundary

## 🔐 Security model

The storage stack uses a layered design:

```text
password + salt
    ↓
Argon2id
    ↓
PW_KEY
    ↓
encrypted MASTER_KEY inside .wlt
    ↓
HKDF-separated DATA / INDEX / INTEGRITY keys
    ↓
XChaCha20-Poly1305 for secrets and object payloads
```

Current operational properties:

- `.wlt` work files live on `/dev/shm` instead of persistent disk
- final `.wlt` writes are atomic and private-file scoped
- secrets and objects are sealed with bound metadata
- open and restore paths fail closed on corrupted or version-mismatched data
- tx-history JSONL is canonical but not automatically protected by the `.wlt`
  encryption layer

## 🧭 Export and restore shape

The live export pack includes these planes:

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

The restore path must treat that shape as canonical.
It must not invent a second bundle contract or a second wallet-state authority.
Assets stay on `owned_assets`; Voucher and Right inventory rows stay on
`owned_objects`.

## ⚙️ Receive-to-persist path

The preferred receive path is request-aware and explicit:

1. load scan state
2. include non-expired payment requests when present
3. scan candidate chunks
4. claim hits only through `ReceiveNext::PersistClaim`
5. persist claimed rows back into the wallet state

Important distinctions:

- report-only receive surfaces do not become persisted balance
- detected output DTOs are not canonical stored balance rows
- `wallet.asset.*` remains the cash-asset surface
- voucher and right inventory stay on the typed object plane

## 🚫 What not to claim

The current code does **not** justify these shortcuts:

- ".wlt stores all live tx history"
- "the JSON helper is the primary wallet authority"
- "receive correctness depends only on tag16 cache hits"
- "typed object inventory is a second persistence database"

## 📚 Source map

- [wallet_service.rs](../src/services/wallet_service.rs)
- [wallet_store_create_unlock.rs](../src/services/wallet_store_create_unlock.rs)
- [wallet_store_restore.rs](../src/services/wallet_store_restore.rs)
- [wallet_store_export_pack.rs](../src/services/wallet_store_export_pack.rs)
- [wallet_store_persistence_pack.rs](../src/services/wallet_store_persistence_pack.rs)
- [wallet_actions_backup.rs](../src/services/wallet_actions_backup.rs)
- [schema_keys.rs](../src/db/schema_keys.rs)
- [open_discovery.rs](../src/redb_store/open_discovery.rs)
- [open_wallet.rs](../src/redb_store/open_wallet.rs)
- [tables.rs](../src/redb_store/tables.rs)
- [record_codecs.rs](../src/redb_store/record_codecs.rs)
- [tx_storage_impl.rs](../src/persistence/tx_storage_impl.rs)
- [backup_exporter_impl.rs](../src/backup/backup_exporter_impl.rs)
- [backup_importer_impl.rs](../src/backup/backup_importer_impl.rs)
- [WALLET-GUIDE.md](./WALLET-GUIDE.md)
- [WLT-BREAKDOWN.md](./WLT-BREAKDOWN.md)
