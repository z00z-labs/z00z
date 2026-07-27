### 0) Entry point (Assets screen)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]                      │
├──────────────────────────────────────────────────────────────────────────────┤
│  Assets                                                     [ + Add asset ]   │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  Z00Z                         1.2345           ≈ $123.45                │  │
│  │  USTC                         99.1000          ≈ $99.10                 │  │
│  │  GOLD                         0.0500           ≈ $12.34                 │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 1) Add Asset modal (search + tabs)

```text
┌──────────────────────────────────── Add Asset ───────────────────────────────┐
│ Search:  [  type symbol / name / address / id...                       ] [🔎] │
│                                                                              │
│  Tabs:  [ Recommended ]  [ Verified ]  [ Custom ]  [ Watch-only ]            │
│                                                                              │
│  Filters:  [x] Hide spam   [x] Verified only   Network: [ Mainnet ▾ ]        │
│                                                                              │
│  Results:                                                                    │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  [ + ]  Z00Z     Z00Z Asset                Verified  ●                   │  │
│  │  [ + ]  USTC     Stable asset (bridged)    Verified  ●                   │  │
│  │  [ + ]  GOLD     Tokenized gold           Verified  ●                   │  │
│  │  [ + ]  ABC      ???                      Unverified ○                   │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│                     [ Cancel ]                        [ Add selected ]       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) Verified asset details (preview before add)

```text
┌────────────────────────────── Asset Details (Preview) ───────────────────────┐
│  Symbol:        Z00Z                                                         │
│  Name:          Z00Z Asset                                                   │
│  Network:       Mainnet                                                      │
│  Decimals:      9                                                            │
│  Asset ID:      z00z:asset:0x...A19F                                         │
│                                                                              │
│  Source:        Verified registry                                            │
│  Risk flags:    None                                                         │
│                                                                              │
│  Options:                                                                     │
│   [x] Show in Assets list                                                    │
│   [x] Enable notifications for incoming                                      │
│                                                                              │
│                       [ Back ]              [ Add asset ]                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Custom asset (manual add: id/address + metadata fetch)

```text
┌──────────────────────────────────── Add Asset ▸ Custom ──────────────────────┐
│ Network: [ Mainnet ▾ ]                                                       │
│                                                                              │
│ Asset identifier (address / id / contract):                                  │
│   [  0x................................................................. ]   │
│                                                                              │
│ [ Fetch metadata ]    Status:  ○ Idle / ● Found / ⊗ Not found                │
│                                                                              │
│ ── Metadata (editable if unknown) ─────────────────────────────────────────  │
│ Symbol:     [  ABC        ]      Name:     [  ABC Token            ]         │
│ Decimals:   [  9    ▾     ]      Icon:     [ Choose… ]                       │
│                                                                              │
│ Risk acknowledgement:                                                        │
│ [ ] I understand this asset may be malicious or fake                         │
│                                                                              │
│                      [ Cancel ]                    [ Add asset ]             │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Watch-only asset (track without enabling spend actions)

```text
┌────────────────────────────────── Add Asset ▸ Watch-only ────────────────────┐
│ Track an asset without enabling send/swap actions.                            │
│                                                                              │
│ Asset identifier: [  z00z:asset:0x... ]  [ Fetch ]                            │
│                                                                              │
│ Display name:     [  Custom Label (optional) ]                                │
│                                                                              │
│ Options:                                                                     │
│  [x] Show in Assets list                                                     │
│  [x] Notify on incoming                                                      │
│  [ ] Allow send/swap (disabled for watch-only)                               │
│                                                                              │
│                      [ Cancel ]                  [ Add watch-only ]          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Add flow states (empty / loading / error) — egui-style panels

### 5a) Empty search

```text
┌──────────────────────────────────── Add Asset ───────────────────────────────┐
│ Search: [  ]                                                                 │
│                                                                              │
│  No results yet. Try searching by symbol, name, or asset id.                  │
│                                                                              │
│  Suggestions:  [ Z00Z ]  [ USTC ]  [ GOLD ]                                  │
│                                                                              │
│                           [ Cancel ]                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5b) Loading registry

```text
┌──────────────────────────────────── Add Asset ───────────────────────────────┐
│ Search: [ z00z ]                                                             │
│                                                                              │
│  Loading…   ────────────────▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯▯                                │
│                                                                              │
│                           [ Cancel ]                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5c) Error / spam warning

```text
┌──────────────────────────────────── Add Asset ───────────────────────────────┐
│ Search: [ ABC ]                                                              │
│                                                                              │
│  ⊗ Could not verify asset.                                                   │
│                                                                              │
│  Warning: This asset is unverified and may be a scam.                         │
│  [ ] I understand the risks                                                  │
│                                                                              │
│                 [ Back ]                 [ Add anyway ]                       │
└──────────────────────────────────────────────────────────────────────────────┘
```