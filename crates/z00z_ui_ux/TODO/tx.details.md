### 1) Tx Details (drill-down screen)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Transaction Details                                           [ ← Back ]    │
├──────────────────────────────────────────────────────────────────────────────┤
│  Status:   ● Pending confirmation        Confirmations: 0 / 1   [ ⟳ Refresh ]│
│  Type:     Send                                                                │
│  Time:     2025-12-20 12:41:10                                                │
│                                                                              │
│  Asset:    Z00Z                                                               │
│  Amount:   -0.25000000 Z00Z                                                   │
│  Fee:      -0.00012000 Z00Z                                                   │
│  Total:    -0.25012000 Z00Z                                                   │
│                                                                              │
│  To:       z00z://recv?...                                     [ Copy ]      │
│  Memo:     (none)                                                             │
│                                                                              │
│  TxID:     0x..B7                                               [ Copy ]      │
│  Local ID: draft-2025-12-20-001                                  [ Copy ]     │
│                                                                              │
│  Actions:  [ View raw ]  [ Export ]  [ Cancel pending ]  [ Report issue ]    │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) Status timeline (optional “stepper” block)

```text
┌────────────────────────────── Status Timeline ───────────────────────────────┐
│  [✓] Built locally                                                           │
│  [✓] Signed                                                                  │
│  [✓] Broadcast submitted                                                     │
│  [..] In mempool                                                             │
│  [ ] Confirmed                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Inputs / Outputs (commitment view)

```text
┌──────────────────────────── Inputs / Outputs ▾ ──────────────────────────────┐
│  ▾ Inputs                                                                    │
│   ┌───────────────────────────────────────────────────────────────────────┐  │
│   │ #12  0.12000000 Z00Z   Commitment: Cmt..9a3f      (spent)              │  │
│   │ #08  0.15000000 Z00Z   Commitment: Cmt..2c11      (spent)              │  │
│   └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ▾ Outputs                                                                   │
│   ┌───────────────────────────────────────────────────────────────────────┐  │
│   │ Recipient  0.25000000 Z00Z   Commitment: Cmt..7b10   (new)             │  │
│   │ Change     0.94988000 Z00Z   Commitment: Cmt..aa05   (new)             │  │
│   └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  [ Copy all commitments ]   [ Show advanced ]                                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Raw / technical tab (for dev)

```text
┌────────────────────────────── Technical ▾ ───────────────────────────────────┐
│  ▾ Technical                                                                 │
│   Network:     Mainnet                                                       │
│   Endpoint:    https://rpc1.example.org                                      │
│   Size:        2.4 KB                                                        │
│   Nonce:       1042                                                          │
│                                                                              │
│   Proofs:      Bulletproof+  (present)                                       │
│   Key image:   KI..F1A0                                         [ Copy ]     │
│                                                                              │
│   Errors:      (none)                                                        │
│                                                                              │
│   [ View raw JSON ]   [ Copy raw ]   [ Save raw… ]                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Explorer link block (if exists)

```text
┌────────────────────────────── Explorer ──────────────────────────────────────┐
│ External explorer: (not available on this network)                            │
│ [ Copy TxID ]                                                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Export dialog (from Tx Details)

```text
┌────────────────────────────── Export Transaction ────────────────────────────┐
│ Export:                                                                      │
│  (•) Tx summary (CSV)                                                        │
│  ( ) Full JSON (raw)                                                         │
│  ( ) Proof bundle (dev)                                                      │
│                                                                              │
│ Destination: [ /home/user/Downloads/tx_0xB7.json ] [ Browse… ]                │
│                                                                              │
│                          [ Cancel ]               [ Export ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 7) Pending-only actions (shown only when pending)

### 7a) “Cancel pending” inline banner

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ○ Pending: you can try to cancel while still in mempool.  [ Cancel pending ] │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 8) States / errors

### 8a) Loading details

```text
┌──────────────────────────── Transaction Details ─────────────────────────────┐
│ Loading transaction…  ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                     │
│                                                                              │
│ [ Back ]                                                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8b) Not found (local record missing)

```text
┌──────────────────────────── Transaction Details ─────────────────────────────┐
│ ⊗ Transaction not found                                                      │
│                                                                              │
│ It may have been removed from local history or belongs to a different wallet.│
│                                                                              │
│ [ Back ]   [ Search history ]                                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8c) Network error (cannot refresh status)

```text
┌──────────────────────────── Transaction Details ─ Error ─────────────────────┐
│ ○ Showing cached details (network unavailable).                              │
│                                                                              │
│ [ Retry refresh ]                              [ Network settings ]          │
└──────────────────────────────────────────────────────────────────────────────┘
```