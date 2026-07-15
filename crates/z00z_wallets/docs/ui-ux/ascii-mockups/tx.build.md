### 1) Build Transaction (draft composer → “Build/Preview”)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Build Transaction                                              [ ⟳ Refresh ] │
├──────────────────────────────────────────────────────────────────────────────┤
│ Type:   [ Send ▾ ]        Wallet: [ Primary Wallet ▾ ]                       │
│ Asset:  [ Z00Z ▾ ]        Available: 1.20000000 Z00Z                         │
│                                                                              │
│ To (recipient / request):                                                    │
│  [ z00z://recv?...____________________________________________ ]             │
│  [ Paste ]  [ From clipboard ]  [ Scan QR ]                                  │
│                                                                              │
│ Amount: [ 0.25000000 ]   ( ) Max   [x] Subtract fee from amount              │
│ Memo:   [ optional…__________________________________________ ]              │
│                                                                              │
│ Fee policy:  [ Auto ▾ ]    Max fee: [ 0.0005 ▾ ]   [ Estimate fee ]          │
│                                                                              │
│ Build options:                                                               │
│  [x] Validate recipient request                                               │
│  [x] Randomize input ordering (privacy)                                       │
│  [ ] Manual input selection (advanced)                                        │
│                                                                              │
│ Actions:  [ Preview ]   [ Build tx ]   [ Save draft ]   [ Clear ]            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `tx.build` — Preview (before building/saving as “built”)

### 2) Preview (decoded request + planned outputs)

```text
┌────────────────────────────── Tx Preview ────────────────────────────────────┐
│ Type: Send         Asset: Z00Z           Wallet: Primary Wallet               │
│                                                                              │
│ Recipient:  z00z://recv?...                                                   │
│ Decoded:    Network=Mainnet  Requested=0.25000000  MemoRequired=No            │
│                                                                              │
│ Amount: 0.25000000 Z00Z                                                      │
│ Fee (est): 0.00012000 Z00Z                                                   │
│ Total: 0.25012000 Z00Z                                                       │
│                                                                              │
│ Planned outputs:                                                             │
│  • Recipient: 0.25000000 Z00Z                                                │
│  • Change:    0.94988000 Z00Z                                                │
│                                                                              │
│ Warnings:                                                                    │
│  ○ None                                                                      │
│                                                                              │
│                     [ Back ]     [ Build now ]                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Advanced: manual input selection (asset commitments)

### 3) Inputs (advanced selection panel)

```text
┌────────────────────────────── Tx Build ▸ Inputs ─────────────────────────────┐
│ Input selection:   (•) Auto    ( ) Manual                                    │
│ Strategy:          [ Smallest-first ▾ ]   Randomize: [x]                      │
│                                                                              │
│ Available commitments:                                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ [x] #12  0.12000000 Z00Z   Status: ● Unspent   Age: 2d                  │  │
│  │ [x] #08  0.15000000 Z00Z   Status: ● Unspent   Age: 1d                  │  │
│  │ [ ] #03  1.00000000 Z00Z   Status: ● Unspent   Age: 7m                  │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│ Selected inputs: 2     Total in: 0.27000000 Z00Z                              │
│                                                                              │
│                     [ Back ]                 [ Apply ]                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Advanced: build options (collapsing header)

### 4) Build options (privacy / constraints / debug)

```text
┌────────────────────────────── Build Options ▾ ───────────────────────────────┐
│  ▾ Build options                                                              │
│   [x] Validate recipient request                                              │
│   [x] Anti-double-spend checks                                                │
│   [x] Randomize ordering                                                      │
│   [ ] Add dummy outputs (decoys)                                              │
│                                                                              │
│   Max inputs: [ 16 ▾ ]      Max outputs: [ 8 ▾ ]                              │
│                                                                              │
│   Diagnostics:                                                                │
│    [ ] Show builder trace                                                     │
│    [ ] Show raw tx JSON                                                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Build step result

### 5) Built transaction (ready to sign/broadcast)

```text
┌────────────────────────────── Tx Built ──────────────────────────────────────┐
│ ● Transaction built successfully                                              │
│                                                                              │
│ Local ID:   draft-2025-12-20-001                                  [ Copy ]   │
│ Fee (final): 0.00011800 Z00Z                                                 │
│ Size:        2.4 KB                                                          │
│                                                                              │
│ Next actions:                                                                │
│  [ Sign ]   [ Submit to network ]   [ Save draft ]   [ View raw ]            │
│                                                                              │
│                                [ Close ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Failure states

### 6) Invalid recipient / decode error

```text
┌──────────────────────────────── Tx Build ────────────────────────────────────┐
│ Recipient: [ z00z://... ]                                                    │
│ ⊗ Cannot build: invalid recipient request / wrong network                     │
│                                                                              │
│ [ Edit recipient ]   [ Clear ]                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Insufficient funds / asset selection failed

```text
┌──────────────────────────────── Tx Build ────────────────────────────────────┐
│ ⊗ Cannot build: insufficient funds                                           │
│ Available: 0.12000000 Z00Z    Needed (incl fee): 0.25011800 Z00Z             │
│                                                                              │
│ Options:  [ Use Max ]  [ Reduce amount ]  [ Merge commitments ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8) Building (progress)

```text
┌────────────────────────────── Building… ─────────────────────────────────────┐
│ Building transaction…                                                        │
│                                                                              │
│  [..] Validate inputs                                                        │
│  [..] Select commitments                                                     │
│  [..] Construct outputs                                                      │
│  [..] Finalize fee                                                           │
│                                                                              │
│ Progress: ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                                │
│                                                                              │
│                               [ Cancel ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```