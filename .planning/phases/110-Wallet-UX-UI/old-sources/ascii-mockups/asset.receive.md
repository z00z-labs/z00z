# Asset Receive Mockup

## 1) Receive screen (main egui layout: amount + receiver card + QR)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]                      │
├──────────────────────────────────────────────────────────────────────────────┤
│  Receive                                                                      │
│                                                                              │
│  Asset:    [ Z00Z ▾ ]        Wallet: [ Primary Wallet ▾ ]                    │
│  Amount:   [ 0.00000000 ]   ( ) Any amount   [x] Include amount in request   │
│  Memo:     [ optional…______________________________ ]   [ ] Require memo    │
│                                                                              │
│  Request format:  [ Receiver URI ▾ ]  [x] Include network                     │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                         [  QR CODE PREVIEW  ]                           │  │
│  │                                                                        │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  Receiver card / commitment:                                                 │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ z00z://recv?asset=Z00Z&to=...&amount=...&memo=...                        │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  Actions:  [ Copy ]  [ Share… ]  [ Save QR… ]  [ New card ]     [⟳ Refresh ] │
│                                                                              │
│  Status:  ● Ready    Expires: [ 30 min ▾ ]   One-time: [x]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `asset.receive` details (egui-style panels)

### 2) “New receiver card / commitment” modal (one-time vs reusable)

```text
┌────────────────────────────── New Receiver Card ─────────────────────────────┐
│ Asset:  Z00Z                         Wallet: [ Primary Wallet ▾ ]            │
│                                                                              │
│ Type:   (•) One-time (recommended)      ( ) Reusable                         │
│                                                                              │
│ Expiration:   [ 30 min ▾ ]   ( ) No expiry                                   │
│                                                                              │
│ Options:                                                                      │
│  [x] Include amount in URI                                                   │
│  [ ] Require memo                                                            │
│  [x] Show as QR code                                                         │
│                                                                              │
│                      [ Cancel ]                 [ Generate ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Compact “Receive widget” (for asset card / sidebar)

```text
┌──────────────────────────── Receive (Z00Z) ──────────────────────────────────┐
│ Amount: [ 0.00 ]   ( ) Any   [x] Include amount                              │
│ Memo:   [ optional…________________ ]                                        │
│                                                                              │
│ Receiver: z00z://recv?...                                        [ Copy ]    │
│                                                                              │
│ [ Show QR ]   [ New ]                                                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## QR-focused view (fullscreen panel)

### 4) QR fullscreen (presentation mode)

```text
┌──────────────────────────────────── Receive ─────────────────────────────────┐
│  Z00Z (Z00Z Asset)                Amount: 0.25 Z00Z        Expires: 29:14    │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│     ┌─────────────────────────────────────────────────────────────────┐      │
│     │                                                                 │      │
│     │                       [   QR CODE LARGE   ]                     │      │
│     │                                                                 │      │
│     └─────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│  z00z://recv?asset=Z00Z&to=...&amount=0.25                                   │
│                                                                              │
│  [ Copy ]  [ Share… ]  [ New card ]     [ Back ]                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## States (loading / error / expired)

### 5) Generating receive commitment (loading)

```text
┌──────────────────────────────────── Receive ─────────────────────────────────┐
│ Generating receiver card/commitment…                                          │
│  ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                                          │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6) Error (cannot generate)

```text
┌──────────────────────────────────── Receive ─────────────────────────────────┐
│ ⊗ Failed to generate receiver card                                            │
│ Reason: wallet locked / network unavailable / key derivation error            │
│                                                                              │
│ [ Unlock wallet ]   [ Retry ]                     [ Network settings ]        │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Expired request (one-time/TTL)

```text
┌──────────────────────────────────── Receive ─────────────────────────────────┐
│ ○ This receive request has expired.                                           │
│                                                                              │
│ Receiver card/commitment is no longer recommended for use.                    │
│                                                                              │
│ [ Generate new ]                          [ Back ]                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Post-receive hint (optional toast / status area)

### 8) “Incoming detected” notification strip

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● Incoming transfer detected: +0.25000000 Z00Z   Status: Pending              │
│ [ View transaction ]                                       [ Dismiss ]       │
└──────────────────────────────────────────────────────────────────────────────┘
```
