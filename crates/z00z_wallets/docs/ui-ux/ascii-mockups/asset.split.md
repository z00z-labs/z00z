### 1) Entry point (Asset Details → Advanced tools)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Asset Details: Z00Z                                           [ ← Back ]    │
├──────────────────────────────────────────────────────────────────────────────┤
│  Balance: 1.23450000 Z00Z       Available: 1.20000000        [ ⟳ Refresh ]   │
│                                                                              │
│  Quick actions: [ Send ] [ Receive ] [ Swap ]                                 │
│  Advanced tools: [ Split ] [ Merge ] [ Import ]                               │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `asset.split` — egui-style flow (asset commitment splitting)

### 2) Split commitment — choose source + target pieces

```text
┌──────────────────────────────────── Split Commitment ────────────────────────┐
│ Asset: [ Z00Z ▾ ]        Wallet: [ Primary Wallet ▾ ]        [ ⟳ Refresh ]   │
│                                                                              │
│ Choose source commitment:                                                    │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  (•) Auto-select (largest available)                                     │  │
│  │  ( ) Manual select                                                       │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│ If manual:                                                                   │
│  Source: [ #01  1.00000000 Z00Z ▾ ]   Status: ● Unspent                      │
│                                                                              │
│ Split into:   [ 4 ▾ ] pieces     Preset: [ Equal parts ▾ ]                   │
│                                                                              │
│ Amounts (editable):                                                         │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  #1  [ 0.25000000 ]   #2  [ 0.25000000 ]                                │  │
│  │  #3  [ 0.25000000 ]   #4  [ 0.25000000 ]                                │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│ Remainder handling: (•) Add remainder to last  ( ) Create separate “change”  │
│                                                                              │
│ Fee policy:   [ Auto ▾ ]    Max fee: [ 0.0005 Z00Z ▾ ]                        │
│ Estimated fee: ~0.00014000 Z00Z                                              │
│                                                                              │
│                      [ Cancel ]            [ Preview split ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Split presets (egui combo opens list)

```text
┌────────────────────────────── Split ▸ Preset Select ─────────────────────────┐
│ Preset:                                                                    ▾ │
│                                                                              │
│  • Equal parts                                                               │
│  • Powers of 2 (0.5, 0.25, 0.125…)                                           │
│  • Common amounts (0.1 / 0.05 / 0.01)                                        │
│  • Custom                                                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Preview (inputs, outputs, fee, and warnings)

```text
┌────────────────────────────── Split ▸ Preview ───────────────────────────────┐
│ Asset: Z00Z                       Wallet: Primary Wallet                     │
│                                                                              │
│ Input:                                                                       │
│  Source commitment: 1.00000000 Z00Z                                          │
│                                                                              │
│ Outputs (new commitments):                                                   │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ Out#1  0.25000000 Z00Z                                                   │  │
│  │ Out#2  0.25000000 Z00Z                                                   │  │
│  │ Out#3  0.25000000 Z00Z                                                   │  │
│  │ Out#4  0.24986000 Z00Z   (remainder adjusted)                            │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│ Fee: 0.00014000 Z00Z   (policy: Auto)                                        │
│                                                                              │
│ Notes:                                                                       │
│  • Splitting may increase linkability if repeated with identical patterns.   │
│  • Prefer randomization or varied sizes when privacy matters.                │
│                                                                              │
│                     [ Back ]        [ Confirm split ]                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 5) Advanced options (privacy / limits / randomization)

```text
┌────────────────────────────── Split Commitment ▸ Advanced ▾ ─────────────────┐
│  ▾ Advanced options                                                          │
│                                                                              │
│  Randomize output ordering:  [x]                                             │
│  Add small jitter to amounts: [ ]   (disabled if exact amounts required)     │
│  Max outputs:               [ 16 ▾ ]                                         │
│                                                                              │
│  Change handling:                                                            │
│   (•) Fold into last output                                                  │
│   ( ) Separate change output                                                 │
│                                                                              │
│  Diagnostics:                                                                │
│   [ ] Show raw commitment ids                                                │
│   [ ] Show builder trace                                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## States / validation errors

### 6) Not enough balance / source too small

```text
┌──────────────────────────────────── Split Commitment ────────────────────────┐
│ ⊗ Cannot split with requested outputs                                         │
│                                                                              │
│ Reason: source commitment too small for chosen amounts + fee.                 │
│                                                                              │
│ Options:                                                                      │
│  [ Auto-select different source ]   [ Reduce pieces ]   [ Reduce amounts ]    │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Invalid amounts (sum mismatch)

```text
┌──────────────────────────────────── Split Commitment ────────────────────────┐
│ Amounts:                                                                      │
│  #1 [0.30]  #2 [0.30]  #3 [0.30]  #4 [0.30]                                   │
│                                                                              │
│ ⊗ Sum of outputs exceeds source commitment (incl fee).                        │
│ Source: 1.00000000   Outputs sum: 1.20000000   Fee est: 0.00014              │
│                                                                              │
│ [ Fix automatically ]  [ Clear ]                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8) Wallet locked (blocked)

```text
┌──────────────────────────────────── Split Commitment ────────────────────────┐
│ ○ Wallet is locked                                                            │
│                                                                              │
│ Unlock required to build split transaction.                                   │
│                                                                              │
│ [ Unlock wallet ]                                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Progress + result

### 9) Splitting progress

```text
┌──────────────────────────────────── Splitting… ──────────────────────────────┐
│ Building split transaction…                                                   │
│  Progress:  ────────────────▮▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯                                │
│                                                                              │
│ [..] Select source input                                                     │
│ [..] Build tx                                                                │
│ [..] Update local store                                                      │
│                                                                              │
│                               [ Cancel ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 10) Split done

```text
┌──────────────────────────────────── Split Done ──────────────────────────────┐
│ ● Split completed                                                            │
│                                                                              │
│ Created 4 new commitments.                                                    │
│                                                                              │
│ Next:  [ View asset ]  [ View tx details ]  [ Split again ]                   │
│                                                                              │
│                                 [ Close ]                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```