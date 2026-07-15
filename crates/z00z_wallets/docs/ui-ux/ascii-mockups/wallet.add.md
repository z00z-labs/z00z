### 1) Wallets hub (where “Add wallet” lives)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  Wallets                                                          [ + Add ]  │
├──────────────────────────────────────────────────────────────────────────────┤
│  ▸ Primary Wallet        ● Unlocked        Assets: 4        [ Open ]          │
│    Trading Wallet        ○ Locked          Assets: 2        [ Open ]          │
│    Watch-only Wallet     ○ Locked          Assets: 1        [ Open ]          │
│                                                                              │
│  Actions:  [ Create new ]   [ + Add existing ]                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `wallet.import` — Add existing wallet wizard

### 2) Step 1 — Choose import method

```text
┌────────────────────────────── Add Wallet (1/5) ──────────────────────────────┐
│ Add existing wallet                                                          │
│                                                                              │
│ Choose method:                                                               │
│  (•) Seed phrase (recovery)                                                  │
│  ( ) Encrypted backup file (.z00zbak)                                        │
│  ( ) Watch-only (public key only)                                            │
│  ( ) Hardware signer (if supported)                                          │
│                                                                              │
│ Wallet name:  [ Imported Wallet______________________ ]                      │
│                                                                              │
│                          [ Cancel ]                 [ Next ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Step 2A — Seed phrase input (recovery)

```text
┌────────────────────────────── Add Wallet ▸ Seed (2/5) ───────────────────────┐
│ Seed phrase                                                                  │
│                                                                              │
│ Words (12/24):   [ 12 words ▾ ]                                              │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ word1  word2  word3  ...                                                  │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ Passphrase (optional):  [ __________________________ ]  [ show ]             │
│                                                                              │
│ Derivation preset:  [ Default ▾ ]     Network: [ Mainnet ▾ ]                 │
│                                                                              │
│ [x] Validate checksum                                                        │
│                                                                              │
│                          [ Back ]                 [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 3) Step 2B — Backup file selection (.z00zbak)

```text
┌────────────────────────────── Add Wallet ▸ Backup (2/5) ─────────────────────┐
│ Backup file                                                                  │
│                                                                              │
│ File:  [ /home/user/Downloads/Primary_2025-12-20.z00zbak ]  [ Browse… ]       │
│ Password: [ *********************** ]   [ show ]                             │
│                                                                              │
│ [x] Verify integrity before import                                           │
│                                                                              │
│                          [ Back ]                 [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 3) Step 2C — Watch-only (public key)

```text
┌────────────────────────────── Add Wallet ▸ Watch-only (2/5) ─────────────────┐
│ Watch-only wallet                                                            │
│                                                                              │
│ Public key / wallet id:                                                      │
│  [ z00zpub1q....................................................... ]        │
│  [ Paste ]  [ From clipboard ]  [ Scan QR ]                                  │
│                                                                              │
│ Format: [ Auto ▾ ]   ( ) Bech32   ( ) Hex   ( ) Base64                        │
│                                                                              │
│ [x] Verify key format                                                        │
│                                                                              │
│                          [ Back ]                 [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Step 3 — Import options (merge vs new, settings)

```text
┌────────────────────────────── Add Wallet (3/5) ──────────────────────────────┐
│ Import options                                                               │
│                                                                              │
│ Storage profile:  (•) Add as new wallet                                      │
│                  ( ) Merge into existing wallet: [ Primary Wallet ▾ ]        │
│                                                                              │
│ Network settings:                                                            │
│  (•) Use current app network (Mainnet)                                       │
│  ( ) Use network from backup / key (if available)                            │
│                                                                              │
│ After import:                                                                │
│  [x] Start rescan / sync                                                     │
│  [x] Create encrypted backup reminder                                        │
│                                                                              │
│                         [ Back ]                  [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 5) Step 4 — Review + confirm

```text
┌────────────────────────────── Add Wallet (4/5) ──────────────────────────────┐
│ Review                                                                       │
│                                                                              │
│ Name:        Imported Wallet                                                 │
│ Method:      Seed phrase / Backup / Watch-only                               │
│ Network:     Mainnet                                                         │
│ Mode:        Full wallet (keys) / Watch-only                                 │
│                                                                              │
│ [ ] I understand adding a seed grants full control of funds on this device.  │
│                                                                              │
│                        [ Back ]                [ Import wallet ]             │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 6) Step 5 — Progress + completion (rescan)

```text
┌────────────────────────────── Importing… ────────────────────────────────────┐
│ Importing wallet…                                                            │
│                                                                              │
│ [..] Validate input                                                          │
│ [..] Create local store                                                      │
│ [..] Initialize keys                                                         │
│ [..] Start rescan / sync                                                     │
│                                                                              │
│ Progress: ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                                │
│                                                                              │
│                               [ Hide ]                                      │
└──────────────────────────────────────────────────────────────────────────────┘
┌────────────────────────────── Wallet Added ──────────────────────────────────┐
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

## Common errors / validations

### 7) Seed invalid

```text
┌────────────────────────────── Add Wallet ─ Error ────────────────────────────┐
│ ⊗ Invalid seed phrase                                                        │
│ Reason: checksum mismatch / wrong word count                                 │
│                                                                              │
│ [ Fix seed ]   [ Back ]                                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8) Wrong backup password / corrupted file

```text
┌────────────────────────────── Add Wallet ─ Error ────────────────────────────┐
│ ⊗ Cannot import backup                                                       │
│ Reason: wrong password / corrupted backup / unsupported version              │
│                                                                              │
│ [ Choose another file ]   [ Try again ]                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9) Watch-only key invalid

```text
┌────────────────────────────── Add Wallet ─ Error ────────────────────────────┐
│ ⊗ Invalid public key format                                                  │
│                                                                              │
│ [ Edit key ]   [ Clear ]                                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```
