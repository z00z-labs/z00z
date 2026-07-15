### 1) Send screen (egui: form + preview + right-side summary)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]                      │
├──────────────────────────────────────────────────────────────────────────────┤
│  Send                                                                          │
│                                                                              │
│ ┌───────────────────────────────────────────┐  ┌───────────────────────────┐ │
│ │ From                                     │  │ Summary                   │ │
│ │  Wallet:   [ Primary Wallet ▾ ]          │  │ Asset:     Z00Z            │ │
│ │  Asset:    [ Z00Z ▾ ]                    │  │ Amount:    0.25000000      │ │
│ │  Available: 1.20000000 Z00Z              │  │ Fee:       ~0.00012        │ │
│ │                                           │  │ Total:     0.25012 Z00Z   │ │
│ │ To                                        │  │ ETA:       ~10–30s         │ │
│ │  Recipient (address/URI):                 │  │ Status:    ● Ready         │ │
│ │   [ z00z://recv?...____________________ ] │  └───────────────────────────┘ │
│ │  [ Paste ]  [ From clipboard ]  [ Scan QR ]                                │
│ │                                           │
│ │ Amount                                    │
│ │  [ 0.25000000 ]   ( ) Max   [x] Subtract fee from amount                  │
│ │                                           │
│ │ Memo (optional)                            │
│ │  [ _________________________________ ]    │
│ │  [ ] Require memo (if recipient requests) │
│ │                                           │
│ │ Fee policy                                │
│ │  Fee: [ Auto ▾ ]   Max fee: [ 0.0005 ▾ ]  │
│ │  [ Estimate ]                              │
│ └───────────────────────────────────────────┘
│                                                                              │
│  Actions:   [ Preview ]    [ Clear ]                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `asset.send` — Preview & confirm

### 2) Preview dialog (inputs/outputs + warnings)

```text
┌────────────────────────────── Send ▸ Preview ────────────────────────────────┐
│ Asset: Z00Z                       From: Primary Wallet                       │
│                                                                              │
│ To:     z00z://recv?...                                                      │
│ Amount: 0.25000000 Z00Z                                                      │
│ Fee:    0.00012000 Z00Z    (policy: Auto)                                    │
│ Total:  0.25012000 Z00Z                                                      │
│                                                                              │
│ Outputs:                                                                     │
│  Recipient:   0.25000000 Z00Z                                                │
│  Change:      0.94988000 Z00Z                                                │
│                                                                              │
│ Notes:                                                                       │
│  • Double-check recipient address.                                           │
│  • Network congestion may affect confirmation time.                          │
│                                                                              │
│                           [ Back ]     [ Confirm & Send ]                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Recipient parsing / request details (URI decoded)

### 3) Recipient request decoded (collapsing panel)

```text
┌──────────────────────────── Recipient Request ▾ ─────────────────────────────┐
│  ▾ Decoded request                                                           │
│   Network:        Mainnet                                                    │
│   Asset:          Z00Z                                                       │
│   Requested amt:  0.25000000                                                 │
│   Memo required:  No                                                         │
│   Expires:        29:14                                                      │
│                                                                              │
│  [x] Enforce requested amount                                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Advanced: asset/commitment selection (asset management UI)

### 4) Advanced asset selection (optional)

```text
┌────────────────────────────── Send ▸ Advanced ▸ Inputs ──────────────────────┐
│ Input selection:   (•) Auto    ( ) Manual                                    │
│                                                                              │
│ Available commitments:                                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ [x] #12   0.12000000 Z00Z   Status: ● Unspent   Age: 2d                 │  │
│  │ [x] #08   0.15000000 Z00Z   Status: ● Unspent   Age: 1d                 │  │
│  │ [ ] #03   1.00000000 Z00Z   Status: ● Unspent   Age: 7m                 │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│ Strategy: [ Smallest-first ▾ ]   Randomize: [x]   Max inputs: [ 16 ▾ ]        │
│                                                                              │
│                       [ Back ]                     [ Apply selection ]       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## States & validation (egui-friendly)

### 5) Wallet locked (blocked send)

```text
┌──────────────────────────────────── Send ────────────────────────────────────┐
│ ○ Wallet is locked                                                           │
│                                                                              │
│ You must unlock to send funds.                                               │
│                                                                              │
│ [ Unlock wallet ]                                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6) Invalid recipient / parse error

```text
┌──────────────────────────────────── Send ────────────────────────────────────┐
│ Recipient: [ z00z://... ]                                                    │
│ ⊗ Invalid address / unsupported URI                                          │
│                                                                              │
│ Tips:                                                                        │
│  • Paste full receive request (z00z://recv?...)                              │
│  • Check network mismatch                                                    │
│                                                                              │
│ [ Clear ]                                                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Insufficient funds

```text
┌──────────────────────────────────── Send ────────────────────────────────────┐
│ Amount: 0.25000000 Z00Z                                                      │
│ ⊗ Insufficient available balance                                             │
│ Available: 0.12000000 Z00Z    Needed (incl fee): 0.25012000 Z00Z             │
│                                                                              │
│ Options:  [ Use Max ]  [ Reduce amount ]  [ Merge commitments ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8) Estimating fee (loading)

```text
┌──────────────────────────────────── Send ────────────────────────────────────┐
│ Estimating fee…   ────────────────▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                         │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Sending & result

### 9) Sending progress (build + broadcast + pending)

```text
┌──────────────────────────────────── Sending… ────────────────────────────────┐
│ Sending 0.25000000 Z00Z                                                      │
│                                                                              │
│  [✓] Build transaction                                                       │
│  [..] Broadcast to network                                                   │
│  [..] Await initial acknowledgment                                           │
│                                                                              │
│  Progress: ────────────────▮▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯                                 │
│                                                                              │
│                           [ Hide ]   [ Cancel ]                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 10) Sent (success)

```text
┌──────────────────────────────────── Sent ────────────────────────────────────┐
│ ● Transaction submitted                                                      │
│                                                                              │
│ Amount: 0.25000000 Z00Z                                                      │
│ Status: Pending confirmation                                                 │
│ TxID:   0x..B7                                               [ Copy ]        │
│                                                                              │
│ [ View tx details ]   [ Back to assets ]   [ Send another ]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 11) Failed (error + recovery actions)

```text
┌──────────────────────────────────── Send ─ Error ────────────────────────────┐
│ ⊗ Failed to send                                                             │
│ Reason: network timeout / rejected / fee too low                             │
│                                                                              │
│ [ Retry ]   [ Increase fee ]   [ Save as draft ]   [ Network settings ]      │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```