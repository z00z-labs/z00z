### 1) Top bar “Lock” action (global)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet: Primary Wallet ▾     Status: ● Unlocked     [ Lock ]   [ Settings ]  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `wallet.lock` — Lock confirm (optional)

### 2) Quick lock (confirmation modal)

```text
┌────────────────────────────── Lock Wallet ───────────────────────────────────┐
│ Wallet: Primary Wallet                                                       │
│                                                                              │
│ Lock this wallet now?                                                        │
│                                                                              │
│ Effect: keys are removed from memory, sending/signing disabled until unlock. │
│                                                                              │
│ [ ] Also lock all wallets                                                     │
│                                                                              │
│                      [ Cancel ]                     [ Lock ]                │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Locked state UI (after lock)

### 3a) Wallet header (status changes)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet: Primary Wallet ▾     Status: ○ Locked       [ Unlock ] [ Settings ]  │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 3b) Disabled actions (Send button disabled)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Assets                                                                       │
├──────────────────────────────────────────────────────────────────────────────┤
│  [ Send ] (disabled)    [ Receive ]    [ Swap ] (disabled)                   │
│                                                                              │
│  ○ Wallet locked — unlock to send/swap                                        │
│  [ Unlock wallet ]                                                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Wallet list quick action (lock from list)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallets                                                                       │
├──────────────────────────────────────────────────────────────────────────────┤
│ Primary Wallet     ● Unlocked                          [ Open ] [ Lock ]     │
│ Trading Wallet     ○ Locked                            [ Open ] [ Unlock ]   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Auto-lock indicator (status bar)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● Connected  |  Auto-lock: 5 min  |  Wallet: ○ Locked                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Lock in progress (brief overlay)

```text
┌───────────────────────────────┐
│ Locking wallet…                │
└───────────────────────────────┘
```

------

## 7) Edge case: pending tx warning (non-blocking)

```text
┌────────────────────────────── Lock Wallet ───────────────────────────────────┐
│ Wallet: Primary Wallet                                                       │
│                                                                              │
│ You have 3 pending transactions. Locking won’t cancel them.                  │
│                                                                              │
│ [ View pending ]                                                             │
│                                                                              │
│                      [ Cancel ]                     [ Lock ]                │
└──────────────────────────────────────────────────────────────────────────────┘
```