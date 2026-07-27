### 1) Send flow (form screen → “Send”)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Send                                                             [ ⟳ Refresh ]│
├──────────────────────────────────────────────────────────────────────────────┤
│ Wallet: [ Primary Wallet ▾ ]        Network: [ Mainnet ▾ ]                    │
│                                                                              │
│ Asset:  [ Z00Z ▾ ]                 Balance: 1.20000000 Z00Z                   │
│                                                                              │
│ To (address / request):                                                      │
│  [ z00z://recv?...____________________________________________ ]             │
│  [ Paste ]  [ From clipboard ]  [ Scan QR ]                                  │
│                                                                              │
│ Amount: [ 0.25000000 ]   ( ) Max   [x] Subtract fee from amount              │
│                                                                              │
│ Fee:    0.00012 Z00Z (Standard)   [ Change ] [ Re-estimate ]                 │
│                                                                              │
│ Memo (optional):                                                             │
│  [ ________________________________________________ ]                        │
│                                                                              │
│ Security:                                                                    │
│  [x] Require confirmation                                                    │
│  [ ] Biometric / quick confirm (if available)                                │
│                                                                              │
│ Actions:   [ Preview ]   [ Send ]   [ Save draft ]                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `tx.send` — Preview (final confirmation)

### 2) Preview modal (what will happen)

```text
┌────────────────────────────── Send ▸ Preview ────────────────────────────────┐
│ From:   Primary Wallet                                                       │
│ To:     z00z://recv?...                                        [ Copy ]      │
│                                                                              │
│ Asset:  Z00Z                                                                 │
│ Amount: 0.25000000 Z00Z                                                      │
│ Fee:    0.00012000 Z00Z  (Standard)                                          │
│ Total:  0.25012000 Z00Z                                                      │
│                                                                              │
│ Min privacy checks: [x] Randomize inputs   [x] Validate request               │
│                                                                              │
│ [ ] I confirm recipient and amount are correct                                │
│                                                                              │
│                       [ Back ]     [ Confirm & Send ]                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Signing / submitting progress

### 3) Sending… (stepper)

```text
┌───────────────────────────────── Sending… ───────────────────────────────────┐
│ Processing transaction                                                       │
│                                                                              │
│  [..] Build transaction                                                      │
│  [..] Sign                                                                    │
│  [..] Broadcast                                                               │
│  [..] Record as pending                                                      │
│                                                                              │
│ Progress: ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                                │
│                                                                              │
│                          [ Hide ]   [ Cancel ]                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Success / result

### 4) Sent (pending)

```text
┌────────────────────────────────── Sent ──────────────────────────────────────┐
│ ● Transaction submitted                                                      │
│                                                                              │
│ Status: Pending confirmation                                                 │
│ TxID:   0x..B7                                                [ Copy ]       │
│                                                                              │
│ Next:  [ View tx details ]   [ Open pending list ]   [ Close ]               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Failure states

### 5) Wallet locked (must unlock)

```text
┌────────────────────────────────── Send ──────────────────────────────────────┐
│ ○ Wallet is locked                                                           │
│                                                                              │
│ Unlock required to send.                                                     │
│                                                                              │
│ [ Unlock wallet ]                                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6) Invalid recipient / wrong network

```text
┌────────────────────────────────── Send ──────────────────────────────────────┐
│ ⊗ Cannot send: invalid recipient request / wrong network                      │
│                                                                              │
│ [ Edit recipient ]   [ Clear ]                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Insufficient funds

```text
┌────────────────────────────────── Send ──────────────────────────────────────┐
│ ⊗ Insufficient funds                                                         │
│ Available: 0.12000000 Z00Z    Needed (incl fee): 0.25012000 Z00Z             │
│                                                                              │
│ [ Use Max ]   [ Reduce amount ]   [ Merge commitments ]                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8) Broadcast failed (retry)

```text
┌────────────────────────────── Send ─ Error ──────────────────────────────────┐
│ ⊗ Broadcast failed                                                           │
│ Reason: timeout / endpoint unreachable / rejected                             │
│                                                                              │
│ [ Retry ]   [ Switch endpoint ]   [ Save draft ]   [ View details ]          │
└──────────────────────────────────────────────────────────────────────────────┘
```