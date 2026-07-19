### 1) Wallet Settings → Danger Zone (entry point)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet: Primary Wallet                                      [ ← Back ]       │
├──────────────────────────────────────────────────────────────────────────────┤
│ Tabs:  [ Overview ]  [ Assets ]  [ Transactions ]  [ Settings ]              │
├──────────────────────────────────────────────────────────────────────────────┤
│ Settings ▸ Danger Zone                                                       │
│                                                                              │
│ ⚠ Deleting removes the wallet from this device.                              │
│ You can restore later only if you have the seed phrase / backup.             │
│                                                                              │
│ Actions:                                                                     │
│  [ Export encrypted backup… ]                                                │
│  [ Show seed phrase… ]  (requires unlock)                                    │
│                                                                              │
│ ───────────────────────────────────────────────────────────────────────────  │
│  [ Delete wallet… ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `wallet.delete` — Delete wizard (danger confirmations)

### 2) Step 1 — Preconditions / checklist

```text
┌────────────────────────────── Delete Wallet (1/3) ───────────────────────────┐
│ Wallet: Primary Wallet                                                        │
│                                                                              │
│ Before you delete:                                                           │
│  [ ] I have saved my seed phrase / backup                                     │
│  [ ] I understand funds are NOT stored in the app                             │
│  [ ] I understand deletion is device-local and cannot be undone               │
│                                                                              │
│ Optional safety:                                                              │
│  [ Create encrypted backup now ]                                              │
│                                                                              │
│                      [ Cancel ]                    [ Next ]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Step 2 — Choose deletion mode

```text
┌────────────────────────────── Delete Wallet (2/3) ───────────────────────────┐
│ What do you want to remove?                                                  │
│                                                                              │
│  (•) Remove wallet from app (recommended)                                    │
│      - removes local wallet store, history, labels                            │
│      - keeps app settings                                                     │
│                                                                              │
│  ( ) Wipe all wallet traces (advanced)                                        │
│      - attempts secure wipe of local files                                    │
│      - may take longer                                                       │
│                                                                              │
│ Confirm by typing wallet name:                                                │
│  [ Primary Wallet________________ ]                                          │
│                                                                              │
│                         [ Back ]                 [ Next ]                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Step 3 — Final confirm (requires unlock)

```text
┌────────────────────────────── Delete Wallet (3/3) ───────────────────────────┐
│ Final confirmation                                                           │
│                                                                              │
│ Wallet: Primary Wallet                                                       │
│                                                                              │
│ [ ] I understand I can only recover with seed/backup                          │
│                                                                              │
│ If wallet is locked:                                                         │
│  ○ Locked — unlock to confirm deletion                                        │
│  [ Unlock wallet ]                                                           │
│                                                                              │
│                          [ Back ]      [ Delete wallet ]                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Progress + result

### 5) Deleting… (progress)

```text
┌────────────────────────────── Deleting… ─────────────────────────────────────┐
│ Deleting wallet from this device…                                            │
│                                                                              │
│ [..] Close sessions                                                          │
│ [..] Remove local database                                                   │
│ [..] Remove caches / indexes                                                  │
│                                                                              │
│ Progress: ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                               │
│                                                                              │
│                               [ Cancel ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6) Deleted (completion)

```text
┌────────────────────────────── Wallet Deleted ────────────────────────────────┐
│ ● Wallet removed from this device                                             │
│                                                                              │
│ You can restore it later using: seed phrase / encrypted backup.               │
│                                                                              │
│ Next:  [ Add existing wallet ]   [ Back to wallets ]                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Error / edge cases

### 7) Cannot delete (last wallet / policy)

```text
┌────────────────────────────── Delete Wallet ─ Blocked ───────────────────────┐
│ ⊗ Cannot delete this wallet                                                   │
│ Reason: last wallet in app / active pending transactions                      │
│                                                                              │
│ Options:                                                                     │
│  [ Cancel pending ]   [ Add another wallet ]                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8) Secure wipe warning (if selected)

```text
┌────────────────────────────── Secure Wipe Warning ───────────────────────────┐
│ ⚠ Secure wipe is best-effort                                                 │
│ Some files may remain due to OS / filesystem behavior (journaling, SSD).      │
│                                                                              │
│ (•) Use normal delete                                                        │
│ ( ) Continue with wipe                                                       │
│                                                                              │
│                      [ Back ]                    [ Continue ]                │
└──────────────────────────────────────────────────────────────────────────────┘
```