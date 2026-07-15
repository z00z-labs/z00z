### 1) Entry point (Settings → Backups)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]                      │
├──────────────────────────────────────────────────────────────────────────────┤
│ Settings ▸ Backups                                                            │
│                                                                              │
│  Backups:   [ Create backup ]   [ Manage backups ]                            │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `backup.create` — egui-style wizard

### 2) Step 1 — Choose backup type + destination

```text
┌────────────────────────────── Create Backup (1/4) ───────────────────────────┐
│ What do you want to back up?                                                 │
│                                                                              │
│  Backup scope:                                                               │
│   (•) Selected wallet         [ Primary Wallet ▾ ]                            │
│   ( ) All wallets in app                                                   │
│   ( ) App settings only                                                     │
│                                                                              │
│ Backup format:                                                               │
│   (•) Encrypted backup file  (.z00zbak)                                      │
│   ( ) Plain JSON (not recommended)                                           │
│                                                                              │
│ Destination:                                                                 │
│   (•) Save to file                                                           │
│   ( ) Copy to clipboard                                                      │
│   ( ) Show as QR (small backups only)                                        │
│                                                                              │
│ Path:  [ /home/user/Downloads/PrimaryWallet_2025-12-20.z00zbak ] [ Browse… ] │
│                                                                              │
│                          [ Cancel ]                 [ Next ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Step 2 — Encryption settings (password + hints)

```text
┌────────────────────────────── Create Backup (2/4) ───────────────────────────┐
│ Encryption                                                                   │
│                                                                              │
│ Password:        [ *********************** ]   [ show ]                      │
│ Confirm:         [ *********************** ]                                  │
│                                                                              │
│ Password hint (optional):                                                    │
│  [ ________________________________________________ ]                        │
│                                                                              │
│ Options:                                                                     │
│  [x] Encrypt metadata (names, labels)                                         │
│  [x] Include app settings                                                     │
│  [ ] Include logs / diagnostics                                               │
│                                                                              │
│ Warning: If you forget the password, backup cannot be recovered.             │
│                                                                              │
│                         [ Back ]                  [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Step 3 — Content selection (what’s included)

```text
┌────────────────────────────── Create Backup (3/4) ───────────────────────────┐
│ Include in backup                                                            │
│                                                                              │
│  Wallet data                                                                 │
│   [x] Wallet configuration                                                    │
│   [x] Local asset store / commitments                                         │
│   [x] Address book / labels                                                   │
│                                                                              │
│  Keys                                                                         │
│   (•) Include seed / key material (full recovery)                             │
│   ( ) Watch-only (no private keys)                                            │
│                                                                              │
│  Network                                                                      │
│   [x] Network settings                                                        │
│   [ ] Custom nodes list                                                       │
│                                                                              │
│ Estimated size:  1.2 MB                                                      │
│                                                                              │
│                         [ Back ]                  [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 5) Step 4 — Final review (summary + confirm)

```text
┌────────────────────────────── Create Backup (4/4) ───────────────────────────┐
│ Review                                                                        │
│                                                                              │
│ Scope:        Selected wallet (Primary Wallet)                                │
│ Format:       Encrypted (.z00zbak)                                            │
│ Destination:  /home/user/Downloads/PrimaryWallet_2025-12-20.z00zbak           │
│ Includes:     wallet config, commitments, labels, seed, app settings          │
│                                                                              │
│ [ ] I understand backups must be stored securely (offline recommended).       │
│                                                                              │
│                         [ Back ]          [ Create backup ]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Progress + result

### 6) Creating backup (progress)

```text
┌────────────────────────────── Creating Backup… ──────────────────────────────┐
│ Writing encrypted backup file…                                                │
│                                                                              │
│ Progress:  ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                                │
│                                                                              │
│ [..] Collect wallet data                                                      │
│ [..] Encrypt                                                                  │
│ [..] Write file                                                               │
│                                                                              │
│                               [ Cancel ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Success (next actions)

```text
┌────────────────────────────── Backup Created ────────────────────────────────┐
│ ● Backup created successfully                                                 │
│                                                                              │
│ File:  PrimaryWallet_2025-12-20.z00zbak                           [ Show ]    │
│ Size:  1.2 MB                                                                │
│                                                                              │
│ Next:                                                                        │
│  [ Create another ]   [ Manage backups ]   [ Close ]                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Error / safety warnings

### 8) Wallet locked / needs unlock (if full recovery includes keys)

```text
┌────────────────────────────── Create Backup ─────────────────────────────────┐
│ ○ Wallet is locked                                                            │
│                                                                              │
│ Unlock required to export seed / private keys.                                │
│                                                                              │
│ [ Unlock wallet ]   [ Back ]                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9) Error writing file

```text
┌────────────────────────────── Create Backup ─ Error ─────────────────────────┐
│ ⊗ Failed to create backup                                                     │
│                                                                              │
│ Reason: permission denied / disk full / path invalid                           │
│                                                                              │
│ [ Retry ]                          [ Choose different path ]                  │
│                                                                              │
│ Details: [ Show ▸ ]                                                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 10) “Insecure format” warning (if Plain JSON selected)

```text
┌────────────────────────────── Backup Warning ────────────────────────────────┐
│ ⚠ Plain JSON backups are not encrypted.                                      │
│ Anyone with the file can access your wallet data.                             │
│                                                                              │
│ (•) Go back and use encrypted backup                                          │
│ ( ) Continue anyway                                                           │
│                                                                              │
│                      [ Back ]                    [ Continue ]                │
└──────────────────────────────────────────────────────────────────────────────┘
```