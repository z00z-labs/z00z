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

## `asset.merge` — egui-style flow (asset commitments consolidation)

### 2) Merge commitments — selection + target preview

```text
┌──────────────────────────────────── Merge Commitments ───────────────────────┐
│ Asset: [ Z00Z ▾ ]        Wallet: [ Primary Wallet ▾ ]        [ ⟳ Refresh ]   │
│                                                                              │
│ Goal: Consolidate multiple small commitments into fewer larger ones.          │
│                                                                              │
│ Merge mode:  (•) Auto-select small        ( ) Manual select                   │
│ Target outputs: [ 1 ▾ ]   ( ) 1   ( ) 2   ( ) 3   ( ) Custom                  │
│                                                                              │
│ Fee policy:   [ Auto ▾ ]    Max fee: [ 0.0005 Z00Z ▾ ]                        │
│                                                                              │
│ ── Available commitments ──────────────────────────────────────────────────  │
│ Filter: [ min amount 0.01 ▾ ]  Sort: [ Smallest ▾ ]  [x] Hide locked/spent    │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ [x] #12   0.01200000 Z00Z   Age: 2d    Status: ● Unspent                 │  │
│  │ [x] #08   0.01550000 Z00Z   Age: 1d    Status: ● Unspent                 │  │
│  │ [ ] #03   0.05000000 Z00Z   Age: 5h    Status: ● Unspent                 │  │
│  │ [ ] #01   1.00000000 Z00Z   Age: 7m    Status: ● Unspent                 │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│ Selected: 2 commitments   Total in: 0.02750000 Z00Z                           │
│ Estimated fee: ~0.00012000 Z00Z                                               │
│ Output preview: 1 output ≈ 0.02738000 Z00Z                                    │
│                                                                              │
│                      [ Cancel ]            [ Preview merge ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Merge preview (outputs, fee, and warnings)

```text
┌──────────────────────────────────── Merge ▸ Preview ─────────────────────────┐
│ Asset: Z00Z                                                                  │
│                                                                              │
│ Inputs (commitments):                                                        │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ #12  0.01200000  │ #08  0.01550000                                      │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│ Outputs (new commitments):                                                   │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ Out#1  ≈ 0.02738000 Z00Z                                                 │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│ Fee:          0.00012000 Z00Z   (policy: Auto)                               │
│ Net change:   -0.00012000 Z00Z                                               │
│                                                                              │
│ Notes:                                                                        │
│  • Merging may reduce privacy if done repeatedly in predictable patterns.     │
│  • Recommended: merge during low-activity windows.                            │
│                                                                              │
│                     [ Back ]        [ Confirm merge ]                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Advanced options (collapsing headers)

```text
┌────────────────────────────── Merge Commitments ▸ Advanced ▾ ─────────────────┐
│  ▾ Advanced options                                                          │
│                                                                              │
│  Selection strategy: [ Smallest-first ▾ ]                                     │
│  Keep change separate: [x]                                                    │
│  Randomize ordering: [x]                                                      │
│  Max inputs per merge: [ 16 ▾ ]                                               │
│                                                                              │
│  Privacy:                                                                     │
│   Merge frequency hint: [ Rare ▾ ]   ( ) Rare  ( ) Normal  ( ) Aggressive     │
│                                                                              │
│  Diagnostics:                                                                 │
│   [ ] Show raw commitment ids                                                 │
│   [ ] Show asset-selection trace                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## States / errors (egui-style)

### 5) Loading commitments

```text
┌──────────────────────────────────── Merge Commitments ───────────────────────┐
│ Loading commitments…   ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                      │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6) Not enough inputs (cannot merge)

```text
┌──────────────────────────────────── Merge Commitments ───────────────────────┐
│ Nothing to merge.                                                            │
│                                                                              │
│ You need at least 2 unspent commitments matching your filters.               │
│                                                                              │
│ Suggestions:                                                                 │
│  [ Reset filters ]   [ View all commitments ]                                 │
│                                                                              │
│                              [ Close ]                                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Insufficient funds for fee

```text
┌──────────────────────────────────── Merge Commitments ───────────────────────┐
│ ⊗ Cannot build merge transaction                                              │
│                                                                              │
│ Reason: insufficient available balance for fee.                               │
│                                                                              │
│ Options:                                                                      │
│  (•) Reduce target outputs / inputs                                           │
│  ( ) Increase max fee (not recommended)                                       │
│                                                                              │
│ [ Back ]                                     [ Retry build ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 8) Merge progress + success

```text
┌──────────────────────────────────── Merging… ────────────────────────────────┐
│ Building merge transaction…                                                   │
│  Progress:  ────────────────▮▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯                                │
│                                                                              │
│ [..] Select inputs                                                           │
│ [..] Build tx                                                                │
│ [..] Update local store                                                      │
│                                                                              │
│                               [ Cancel ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────── Merge Done ──────────────────────────────┐
│ ● Merge completed                                                            │
│                                                                              │
│ New commitment created: 0.02738000 Z00Z                                       │
│                                                                              │
│ Next:  [ View asset ]  [ View tx details ]  [ Merge again ]                   │
│                                                                              │
│                                 [ Close ]                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```