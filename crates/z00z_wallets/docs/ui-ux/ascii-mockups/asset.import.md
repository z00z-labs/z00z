### 1) Entry point (Assets → Tools)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]                      │
├──────────────────────────────────────────────────────────────────────────────┤
│  Assets                                                     [ + Add asset ]   │
│                                                                              │
│  Tools:  [ Import asset ]  [ Merge ]  [ Split ]                              │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `asset.import` — egui-style import flow

### 2) Import Asset modal (choose source)

```text
┌──────────────────────────────────── Import Asset ────────────────────────────┐
│ Import source:                                                               │
│   (•) File / Backup bundle                                                   │
│   ( ) Paste (text / JSON / encoded)                                           │
│   ( ) Scan (QR / camera)                                                     │
│   ( ) From clipboard                                                         │
│                                                                              │
│ Destination wallet:   [ Primary Wallet ▾ ]                                    │
│ Destination asset:    [ Auto-detect  ▾ ]                                      │
│                                                                              │
│  [ Choose file… ]   Selected:  (none)                                         │
│                                                                              │
│                          [ Cancel ]                 [ Next ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Paste / QR import (raw payload input)

```text
┌────────────────────────────── Import Asset ▸ Paste ──────────────────────────┐
│ Destination wallet:   [ Primary Wallet ▾ ]                                    │
│                                                                              │
│ Paste payload (commitment / note / bundle / receipt):                         │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ { ... }                                                                    │ │
│ │                                                                            │ │
│ │                                                                            │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ Encoding:   [ Auto ▾ ]    ( ) Base64   ( ) Hex   ( ) JSON                     │
│                                                                              │
│ [ Validate ]        Status:  ○ Idle / ● Valid / ⊗ Invalid                     │
│                                                                              │
│                          [ Back ]          [ Import ]                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Scan (QR) view (egui panel)

```text
┌────────────────────────────── Import Asset ▸ Scan ───────────────────────────┐
│  Camera: [ Default ▾ ]                                                       │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                        │  │
│  │                       [  QR Preview Area  ]                            │  │
│  │                                                                        │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  Detected:  ○ None / ● Payload found                                          │
│                                                                              │
│  [ Toggle torch ]   [ Paste instead ]                                         │
│                                                                              │
│                          [ Back ]          [ Next ]                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Validation / preview before import

### 5) Preview (what will be imported)

```text
┌───────────────────────────── Import Asset ▸ Preview ─────────────────────────┐
│ Destination wallet:   Primary Wallet                                          │
│                                                                              │
│ Detected type:    Asset bundle / Commitment / Receipt                         │
│ Asset:            Z00Z  (Z00Z Asset)                                          │
│ Amount:           0.25000000 Z00Z                                             │
│ Network:          Mainnet                                                     │
│                                                                              │
│ Uniqueness:       ● New to wallet                                             │
│ Status:           ● Ready to import                                           │
│                                                                              │
│ Options:                                                                     │
│  [x] Show asset in Assets list                                                │
│  [x] Notify on incoming                                                       │
│  [ ] Tag as “offline received”                                                │
│                                                                              │
│                          [ Back ]        [ Import ]                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Conflicts / duplicates / risk checks

### 6) Duplicate / already exists

```text
┌───────────────────────────── Import Asset ─ Conflict ────────────────────────┐
│ This item already exists in your wallet.                                      │
│                                                                              │
│ Asset:   Z00Z                                                                │
│ Amount:  0.25000000 Z00Z                                                     │
│ Reason:  Duplicate commitment / already recorded                              │
│                                                                              │
│ What do you want to do?                                                      │
│  (•) Skip                                                                     │
│  ( ) Import anyway (may create duplicate entry)                               │
│                                                                              │
│                          [ Back ]       [ Continue ]                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Unverified / suspicious payload

```text
┌───────────────────────────── Import Asset ─ Warning ─────────────────────────┐
│ ⊗ Unverified payload / unknown asset                                          │
│                                                                              │
│ Asset:     ABC (unknown)                                                     │
│ Network:   Mainnet                                                           │
│                                                                              │
│ Risk: This could be malicious or fake.                                       │
│                                                                              │
│ [ ] I understand the risks and want to import anyway                          │
│                                                                              │
│                      [ Back ]                    [ Import anyway ]           │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Import progress + result

### 8) Importing (progress)

```text
┌────────────────────────────── Import Asset ─ Importing ──────────────────────┐
│ Importing asset into “Primary Wallet”…                                        │
│                                                                              │
│  Progress:  ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                                │
│                                                                              │
│  [..] Validate payload                                                       │
│  [..] Write to local store                                                   │
│  [..] Update balances                                                        │
│                                                                              │
│                              [ Cancel ]                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9) Success (toast + next actions)

```text
┌────────────────────────────── Import Asset ─ Done ───────────────────────────┐
│ ● Imported successfully                                                      │
│                                                                              │
│ Asset:    Z00Z                                                               │
│ Amount:   0.25000000 Z00Z                                                    │
│                                                                              │
│ Next:                                                                         │
│  [ View asset ]   [ View history ]   [ Import another ]                      │
│                                                                              │
│                                [ Close ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 10) Failure (error + troubleshooting)

```text
┌────────────────────────────── Import Asset ─ Error ──────────────────────────┐
│ ⊗ Import failed                                                              │
│                                                                              │
│ Reason: invalid format / corrupted file / unsupported network                 │
│                                                                              │
│ [ Retry ]                     [ Open Network settings ]                      │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```