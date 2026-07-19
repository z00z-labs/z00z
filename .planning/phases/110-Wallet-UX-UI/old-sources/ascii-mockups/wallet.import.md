### 1) Wallets hub (entry point for import)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  Wallets                                                          [ + Add ]  │
├──────────────────────────────────────────────────────────────────────────────┤
│  Actions:  [ Create new ]   [ Import wallet… ]   [ Add existing ]            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `wallet.import` — Import wizard (file/backup oriented)

### 2) Import Wallet (1/5) — choose source

```text
┌────────────────────────────── Import Wallet (1/5) ───────────────────────────┐
│ Import wallet                                                                │
│                                                                              │
│ Source:                                                                      │
│  (•) Encrypted wallet export / backup file (.z00zbak)                        │
│  ( ) Plain JSON export (dev)                                                 │
│  ( ) Paste payload                                                           │
│                                                                              │
│ File:  [ /home/user/Downloads/PrimaryWallet_export_2025-12-20.z00zbak ]       │
│        [ Browse… ]                                                           │
│                                                                              │
│ Wallet name (after import): [ Imported Wallet______________________ ]         │
│                                                                              │
│                          [ Cancel ]                 [ Next ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Import Wallet (2/5) — unlock file (password + verify)

```text
┌────────────────────────────── Import Wallet (2/5) ───────────────────────────┐
│ Unlock import file                                                           │
│                                                                              │
│ Password:  [ *********************** ]   [ show ]                            │
│                                                                              │
│ Options:                                                                      │
│  [x] Verify integrity                                                        │
│  [x] Validate version compatibility                                          │
│                                                                              │
│ Status:  ○ Waiting / ● Decrypted / ⊗ Wrong password                           │
│                                                                              │
│                         [ Back ]                  [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Import Wallet (3/5) — preview contents + conflict policy

```text
┌────────────────────────────── Import Wallet (3/5) ───────────────────────────┐
│ Preview                                                                       │
│                                                                              │
│ File: PrimaryWallet_export_2025-12-20.z00zbak                                 │
│ Created: 2025-12-20 12:34                                                     │
│ Scope: Full export (keys + wallet data)                                       │
│                                                                              │
│ Includes:                                                                     │
│  [x] Keys / seed material                                                     │
│  [x] Commitments / asset store                                                │
│  [x] Labels / notes                                                           │
│  [ ] Transaction history (optional)                                           │
│                                                                              │
│ Conflict policy:                                                              │
│  (•) Add as new wallet                                                        │
│  ( ) Merge into existing wallet: [ Primary Wallet ▾ ]                         │
│  ( ) Replace existing wallet data (danger)                                    │
│                                                                              │
│                         [ Back ]                  [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 5) Import Wallet (4/5) — restore options (rescan / network)

```text
┌────────────────────────────── Import Wallet (4/5) ───────────────────────────┐
│ Restore options                                                              │
│                                                                              │
│ Network:                                                                     │
│  (•) Use current app network (Mainnet)                                       │
│  ( ) Use network stored in export (if present)                               │
│                                                                              │
│ After import:                                                                │
│  [x] Start rescan / sync                                                     │
│  [ ] Import app settings (theme/timeouts)                                    │
│                                                                              │
│ Privacy:                                                                     │
│  [x] Rebuild indexes locally                                                  │
│                                                                              │
│                         [ Back ]                 [ Import ]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Progress + completion

### 6) Importing… (progress)

```text
┌────────────────────────────── Importing… ────────────────────────────────────┐
│ Importing wallet…                                                            │
│                                                                              │
│ [..] Read file                                                               │
│ [..] Decrypt                                                                 │
│ [..] Validate version                                                        │
│ [..] Write local store                                                        │
│ [..] Start rescan / sync                                                     │
│                                                                              │
│ Progress: ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                                │
│                                                                              │
│                               [ Hide ]                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Done (next actions)

```text
┌────────────────────────────── Wallet Imported ───────────────────────────────┐
│ ● Wallet imported successfully                                               │
│                                                                              │
│ Name: Imported Wallet                                                        │
│ Status: Syncing…                                                             │
│                                                                              │
│ Next:  [ Open wallet ]   [ View assets ]   [ Manage backups ]                │
│                                                                              │
│                                [ Close ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Error / edge cases

### 8) Wrong password / corrupted file

```text
┌────────────────────────────── Import Wallet ─ Error ─────────────────────────┐
│ ⊗ Cannot import wallet export                                                │
│ Reason: wrong password / corrupted file / invalid format                      │
│                                                                              │
│ [ Choose another file ]   [ Try again ]                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9) Unsupported version / migration needed

```text
┌────────────────────────────── Import Wallet ─ Version ───────────────────────┐
│ ⚠ Export was created by a newer wallet version                               │
│                                                                              │
│ Options:                                                                     │
│  (•) Import as read-only (limited)                                           │
│  ( ) Abort                                                                   │
│                                                                              │
│ [ Continue ]                                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 10) Replace warning (danger)

```text
┌────────────────────────────── Replace Warning ───────────────────────────────┐
│ ⚠ Replace will overwrite local wallet data                                   │
│                                                                              │
│ [ ] I understand this is destructive                                         │
│                                                                              │
│                      [ Back ]                    [ Replace ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```