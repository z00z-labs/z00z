### 1) Wallet Settings → Export (entry point)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet: Primary Wallet                                      [ ← Back ]       │
├──────────────────────────────────────────────────────────────────────────────┤
│ Tabs:  [ Overview ]  [ Assets ]  [ Transactions ]  [ Settings ]              │
├──────────────────────────────────────────────────────────────────────────────┤
│ Settings ▸ Backup & Export                                                    │
│                                                                              │
│  [ Create backup… ]    [ Export wallet… ]                                     │
│                                                                              │
│ Note: Export creates an encrypted file you can restore later.                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `wallet.export` — Export wizard (encrypted export)

### 2) Export Wallet (1/4) — scope + destination

```text
┌────────────────────────────── Export Wallet (1/4) ───────────────────────────┐
│ Wallet:  [ Primary Wallet ▾ ]                                                │
│                                                                              │
│ Export scope:                                                                │
│  (•) Full export (keys + wallet data)                                        │
│  ( ) Watch-only export (public data only)                                    │
│  ( ) Wallet data only (no keys)                                              │
│                                                                              │
│ Destination:                                                                 │
│  (•) Save to file                                                            │
│  ( ) Copy to clipboard (small only)                                          │
│                                                                              │
│ Path:  [ /home/user/Downloads/PrimaryWallet_export_2025-12-20.z00zbak ]       │
│        [ Browse… ]                                                           │
│                                                                              │
│                          [ Cancel ]                 [ Next ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Export Wallet (2/4) — encryption password

```text
┌────────────────────────────── Export Wallet (2/4) ───────────────────────────┐
│ Encryption                                                                   │
│                                                                              │
│ Password:        [ *********************** ]   [ show ]                      │
│ Confirm:         [ *********************** ]                                  │
│ Password hint:   [ optional…____________________________ ]                    │
│                                                                              │
│ Options:                                                                      │
│  [x] Encrypt metadata (wallet name/labels)                                    │
│  [x] Verify integrity after export                                            │
│                                                                              │
│ Warning: If you forget the password, you cannot restore this export.         │
│                                                                              │
│                         [ Back ]                  [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Export Wallet (3/4) — what to include

```text
┌────────────────────────────── Export Wallet (3/4) ───────────────────────────┐
│ Include in export                                                            │
│                                                                              │
│  Wallet data                                                                 │
│   [x] Local commitments / asset store                                         │
│   [x] Labels / notes                                                          │
│   [x] Address book                                                            │
│   [ ] Transaction history (optional)                                          │
│                                                                              │
│  App settings (optional)                                                     │
│   [ ] Theme / UI settings                                                     │
│   [ ] Network settings                                                        │
│                                                                              │
│ Estimated size:  1.3 MB                                                      │
│                                                                              │
│                         [ Back ]                  [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 5) Export Wallet (4/4) — review + confirm

```text
┌────────────────────────────── Export Wallet (4/4) ───────────────────────────┐
│ Review                                                                       │
│                                                                              │
│ Wallet:       Primary Wallet                                                 │
│ Scope:        Full export (keys + data)                                       │
│ Format:       Encrypted (.z00zbak)                                            │
│ Destination:  /home/user/Downloads/PrimaryWallet_export_2025-12-20.z00zbak    │
│ Includes:     commitments, labels, address book                               │
│                                                                              │
│ [ ] I understand this file grants access to funds if password is known.      │
│                                                                              │
│                        [ Back ]                 [ Export wallet ]            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Progress + result

### 6) Exporting (progress)

```text
┌────────────────────────────── Exporting… ────────────────────────────────────┐
│ Exporting wallet…                                                            │
│                                                                              │
│ [..] Collect data                                                            │
│ [..] Encrypt                                                                 │
│ [..] Write file                                                              │
│ [..] Verify integrity                                                        │
│                                                                              │
│ Progress: ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                                │
│                                                                              │
│                               [ Cancel ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Success (next actions)

```text
┌────────────────────────────── Export Complete ───────────────────────────────┐
│ ● Encrypted export created                                                   │
│                                                                              │
│ File: PrimaryWallet_export_2025-12-20.z00zbak                    [ Open ]     │
│ Location: /home/user/Downloads/                                    [ Copy ]  │
│                                                                              │
│ Next:  [ Share… ]  [ Test restore… ]  [ Close ]                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## States / errors

### 8) Wallet locked (needs unlock for full export)

```text
┌────────────────────────────── Export Wallet ─────────────────────────────────┐
│ ○ Wallet is locked                                                           │
│                                                                              │
│ Unlock required to export keys (full export).                                 │
│                                                                              │
│ [ Unlock wallet ]   [ Export watch-only instead ]                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9) Weak password warning

```text
┌────────────────────────────── Export Warning ────────────────────────────────┐
│ ⚠ Password appears weak                                                      │
│                                                                              │
│ (•) Go back and choose stronger password                                     │
│ ( ) Continue anyway                                                          │
│                                                                              │
│                      [ Back ]                    [ Continue ]                │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 10) Write error

```text
┌────────────────────────────── Export Error ──────────────────────────────────┐
│ ⊗ Failed to export wallet                                                    │
│ Reason: permission denied / disk full / invalid path                          │
│                                                                              │
│ [ Retry ]                          [ Choose different path ]                  │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```