### 1) Wallet list (entry point for “Remove from app”)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallets                                                          [ + Add ]   │
├──────────────────────────────────────────────────────────────────────────────┤
│ Primary Wallet      ● Unlocked    Type: Standard        [ Open ] [ ⋮ ]       │
│ Trading Wallet      ○ Locked      Type: Standard        [ Open ] [ ⋮ ]       │
│ Watch-only Wallet   ○ Locked      Type: Watch-only      [ Open ] [ ⋮ ]       │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 2) Wallet row context menu (⋮)

```text
┌────────────────────────────── Wallet Actions ────────────────────────────────┐
│ [ Open ]                                                                     │
│ [ Set as active ]                                                            │
│ [ Lock ] / [ Unlock ]                                                        │
│ [ Export ]                                                                   │
│ [ Remove from app… ]                                                         │
│ [ Delete (danger)… ]                                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `wallet.delete` — “Detach” modal (safe removal)

### 3) Remove wallet from app (detach) — confirmation

```text
┌────────────────────────────── Remove Wallet ─────────────────────────────────┐
│ Wallet: Trading Wallet                                                       │
│                                                                              │
│ Remove this wallet from the application?                                     │
│                                                                              │
│ What happens:                                                                │
│  • Removes wallet from this app instance                                     │
│  • Keeps funds on-chain / recoverable                                         │
│  • You can re-add later using seed/backup/public key                          │
│                                                                              │
│ What will be removed from this device:                                       │
│  [x] Local wallet store (cache / indexes)                                    │
│  [x] Local transaction history                                               │
│  [x] Labels / notes / address book (wallet-specific)                          │
│                                                                              │
│ Optional:                                                                    │
│  [ ] Keep watch-only reference (no keys, minimal metadata)                    │
│                                                                              │
│ Confirm by typing “REMOVE”:                                                  │
│  [ REMOVE________________ ]                                                  │
│                                                                              │
│                      [ Cancel ]                   [ Remove ]                │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Pre-removal safety prompts (links)

### 4a) Suggest export/backup before removing (non-blocking)

```text
┌────────────────────────────── Safety Tip ────────────────────────────────────┐
│ ⚠ You are about to remove this wallet from the app.                          │
│                                                                              │
│ Recommended before removing:                                                 │
│  [ Export encrypted backup… ]                                                │
│  [ Show seed phrase… ] (requires unlock)                                     │
│                                                                              │
│                      [ Skip ]                    [ Go to export ]            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Removing… + completion

### 5a) Removing progress

```text
┌────────────────────────────── Removing… ─────────────────────────────────────┐
│ Removing wallet from this device…                                            │
│                                                                              │
│ [..] Close sessions                                                          │
│ [..] Remove local store                                                      │
│ [..] Clear caches                                                            │
│                                                                              │
│ Progress: ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                               │
│                                                                              │
│                               [ Cancel ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5b) Removed success

```text
┌────────────────────────────── Wallet Removed ────────────────────────────────┐
│ ● Wallet removed from this app                                               │
│                                                                              │
│ To add it again later:                                                       │
│  [ Add existing wallet ]   [ Import backup ]   [ Recover from seed ]         │
│                                                                              │
│                                [ Close ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Edge cases / errors

### 6a) Wallet is active (must switch first)

```text
┌────────────────────────────── Remove Wallet ─────────────────────────────────┐
│ ⊗ Cannot remove the active wallet                                            │
│                                                                              │
│ Switch to another wallet first.                                              │
│                                                                              │
│ [ Switch wallet… ]   [ Cancel ]                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6b) Pending transactions warning

```text
┌────────────────────────────── Remove Wallet ─ Warning ───────────────────────┐
│ ⚠ This wallet has 3 pending transactions                                     │
│ Removing won’t cancel them; you may lose local tracking info.                │
│                                                                              │
│ [ View pending ]                                                             │
│                                                                              │
│ (•) Continue remove                                                          │
│ ( ) Cancel                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6c) Remove failed (storage error)

```text
┌────────────────────────────── Remove Wallet ─ Error ─────────────────────────┐
│ ⊗ Failed to remove wallet                                                    │
│ Reason: storage locked / permission denied                                   │
│                                                                              │
│ [ Retry ]                          [ Open logs ]                             │
└──────────────────────────────────────────────────────────────────────────────┘
```
