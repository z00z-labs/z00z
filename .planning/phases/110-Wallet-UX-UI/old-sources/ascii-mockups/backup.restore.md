### 1) Restore Wizard (1/5) — choose source

```text
┌────────────────────────────── Restore Backup (1/5) ──────────────────────────┐
│ Restore source                                                                │
│                                                                              │
│  (•) From file (.z00zbak / .json)                                             │
│  ( ) Paste payload (text / JSON / encoded)                                    │
│  ( ) Scan QR (small backups only)                                             │
│                                                                              │
│ File:  [ /home/user/Downloads/Primary_2025-12-20.z00zbak ]  [ Browse… ]       │
│                                                                              │
│ Restore target:                                                               │
│  (•) This app instance                                                       │
│  ( ) New app profile (separate storage)                                       │
│                                                                              │
│                          [ Cancel ]                 [ Next ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 2) Restore Wizard (2/5) — unlock backup (password / security)

```text
┌────────────────────────────── Restore Backup (2/5) ──────────────────────────┐
│ Unlock backup                                                                 │
│                                                                              │
│ Password:   [ *********************** ]   [ show ]                            │
│                                                                              │
│ Options:                                                                      │
│  [x] Verify integrity before restore                                          │
│  [ ] Restore app settings                                                     │
│  [ ] Restore logs / diagnostics                                               │
│                                                                              │
│ Status:  ○ Waiting / ● Decrypted / ⊗ Wrong password                           │
│                                                                              │
│                         [ Back ]                  [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Restore Wizard (3/5) — preview contents + select what to restore

```text
┌────────────────────────────── Restore Backup (3/5) ──────────────────────────┐
│ Preview contents                                                              │
│                                                                              │
│ Backup info:                                                                  │
│  Format:    Encrypted (.z00zbak)                                              │
│  Created:   2025-12-20 12:34                                                  │
│  Scope:     Primary Wallet (full recovery)                                    │
│                                                                              │
│ Restore items:                                                                │
│  [x] Wallet data (commitments, history, labels)                               │
│  [x] Keys / seed material                                                     │
│  [ ] App settings (theme, timeouts)                                           │
│  [ ] Network settings                                                         │
│                                                                              │
│ Conflict policy:                                                              │
│  (•) Merge with existing (recommended)                                        │
│  ( ) Replace existing wallet data (danger)                                    │
│                                                                              │
│                          [ Back ]                 [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Restore Wizard (4/5) — safety confirmations (danger zone)

```text
┌────────────────────────────── Restore Backup (4/5) ──────────────────────────┐
│ Confirm restore                                                               │
│                                                                              │
│ This will restore wallet data into this app instance.                         │
│                                                                              │
│ Safety checks:                                                                │
│  [x] Backup decrypted successfully                                            │
│  [x] Integrity verified                                                       │
│  [ ] Network matches current config (optional)                                │
│                                                                              │
│ Confirmations:                                                                │
│  [ ] I understand restoring may overwrite local data                          │
│  [ ] I stored the backup password securely                                    │
│                                                                              │
│                        [ Back ]                [ Restore ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 5) Restore Wizard (5/5) — progress + completion

```text
┌────────────────────────────── Restoring… ────────────────────────────────────┐
│ Restoring wallet data…                                                       │
│                                                                              │
│ Progress:  ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                                │
│                                                                              │
│ [..] Read backup                                                              │
│ [..] Decrypt                                                                  │
│ [..] Write local store                                                        │
│ [..] Rebuild indexes                                                          │
│                                                                              │
│                               [ Cancel ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
┌────────────────────────────── Restore Complete ──────────────────────────────┐
│ ● Restore finished successfully                                               │
│                                                                              │
│ Restored: Primary Wallet                                                     │
│                                                                              │
│ Next:                                                                         │
│  [ Open wallet ]   [ Run rescan ]   [ Manage backups ]                        │
│                                                                              │
│                                [ Close ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Alternative source screens (paste / QR)

### 6) Restore from paste

```text
┌────────────────────────────── Restore ▸ Paste ───────────────────────────────┐
│ Paste payload (backup / JSON / encoded):                                      │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ { ... }                                                                    │ │
│ │                                                                            │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│ Encoding: [ Auto ▾ ]   ( ) Base64   ( ) Hex   ( ) JSON                        │
│                                                                              │
│ [ Validate ]   Status: ○ Idle / ● Valid / ⊗ Invalid                           │
│                                                                              │
│                          [ Back ]                 [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Restore from QR (small backups)

```text
┌────────────────────────────── Restore ▸ Scan QR ─────────────────────────────┐
│ Camera: [ Default ▾ ]                                                        │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                        [  QR Preview Area  ]                            │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│ Detected:  ○ None / ● Payload found                                           │
│                                                                              │
│                          [ Back ]                 [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Error states (wrong password / corrupted backup / conflicts)

### 8) Wrong password

```text
┌────────────────────────────── Restore Backup ─ Error ─────────────────────────┐
│ ⊗ Wrong password                                                             │
│                                                                              │
│ Password: [ **************** ]                                                │
│                                                                              │
│ [ Try again ]                          [ Cancel ]                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9) Corrupted / unsupported backup

```text
┌────────────────────────────── Restore Backup ─ Error ─────────────────────────┐
│ ⊗ Cannot restore backup                                                      │
│ Reason: corrupted file / unsupported version / invalid format                 │
│                                                                              │
│ [ Choose another file ]                 [ Cancel ]                            │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 10) Conflict warning (replace existing)

```text
┌────────────────────────────── Restore Conflict ──────────────────────────────┐
│ A wallet with the same identity already exists locally.                       │
│                                                                              │
│ Conflict policy:                                                              │
│  (•) Merge (recommended)                                                     │
│  ( ) Replace existing (will overwrite local data)                             │
│                                                                              │
│ [ ] I understand replacing is destructive                                     │
│                                                                              │
│                         [ Back ]                 [ Continue ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```