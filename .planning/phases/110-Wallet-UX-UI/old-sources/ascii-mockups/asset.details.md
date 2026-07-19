### 1) Asset Details — main card (egui-style)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Asset Details                                                         [ ← ]  │
├──────────────────────────────────────────────────────────────────────────────┤
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  Z00Z   (Z00Z Asset)                              Status: ● Synced      │  │
│  │  Balance:  1.23450000 Z00Z                 ≈ $123.45        [ ⟳ Refresh ]│  │
│  │  Available: 1.20000000    Locked: 0.03450000    Pending: 0.01000000     │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  Quick actions:  [ Send ]  [ Receive ]  [ Swap ]  [ Split ]  [ Merge ]       │
│                                                                              │
│  ┌───────────────────────────────┬────────────────────────────────────────┐  │
│  │ ▸ Overview                     │ ▸ Market / Pricing                     │  │
│  │ ▸ Activity                      │ ▸ Risk & Verification                  │  │
│  │ ▸ Management                    │ ▸ Technical                            │  │
│  └───────────────────────────────┴────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Overview (collapsing headers / right panel)

### 2) Overview section (info rows)

```text
┌──────────────────────────────────── Asset Details ▸ Overview ─────────────────┐
│  ▾ Overview                                                                  │
│                                                                              │
│  Name:            Z00Z Asset                                                 │
│  Symbol:          Z00Z                                                       │
│  Network:         Mainnet                                                    │
│  Category:        Native / Utility                                           │
│  Display:         9 decimals                                                 │
│                                                                              │
│  Labels:   [ Verified ]   [ Primary asset ]   [ Popular ]                    │
│                                                                              │
│  Show on dashboard:     [x]                                                  │
│  Enable notifications:  [x] Incoming funds                                   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Activity (recent tx list inside details)

### 3) Activity section (table-style list)

```text
┌──────────────────────────────────── Asset Details ▸ Activity ─────────────────┐
│  ▾ Activity                                                                  │
│                                                                              │
│  Filters:  Type [ All ▾ ]   Status [ All ▾ ]   Search [ ............. ]      │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ Time       │ Type     │ Amount           │ Status        │ TxID         │  │
│  ├────────────┼──────────┼──────────────────┼───────────────┼──────────────┤  │
│  │ 12:31:02   │ Receive  │ +0.10000000      │ Confirmed     │ 0x..A1        │  │
│  │ 11:10:44   │ Send     │ -0.05000000      │ Pending       │ 0x..B7        │  │
│  │ 09:22:19   │ Swap     │ -0.20000000      │ Failed        │ 0x..C9        │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  [ View full history ]                                                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Management (asset-level controls)

### 4) Management section (hide/remove/watch-only)

```text
┌────────────────────────────────── Asset Details ▸ Management ─────────────────┐
│  ▾ Management                                                                │
│                                                                              │
│  Display                                                                     │
│   [x] Show asset in Assets list                                              │
│   [ ] Pin to top                                                             │
│                                                                              │
│  Notifications                                                               │
│   [x] Notify on incoming                                                     │
│   [ ] Notify on price change                                                 │
│                                                                              │
│  Watch-only                                                                  │
│   Mode:  (•) Full use    ( ) Watch-only                                      │
│                                                                              │
│  Danger zone                                                                 │
│   [ Remove from list ]   (keeps data, can be re-added)                       │
│   [ Forget asset data ]   (clears local cache)                               │
│                                                                              │
│                                 [ Apply ]  [ Revert ]                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Risk & Verification (trust surface)

### 5) Risk panel (verified / unverified / warnings)

```text
┌──────────────────────────── Asset Details ▸ Risk & Verification ─────────────┐
│  ▾ Risk & Verification                                                       │
│                                                                              │
│  Verification:   ● Verified registry                                         │
│  Source:         Official list / curated registry                            │
│                                                                              │
│  Risk flags:     None                                                        │
│                                                                              │
│  If unverified:                                                             │
│   Warning: This asset may be fake or malicious.                              │
│   [ ] I understand the risks                                                 │
│                                                                              │
│  Actions:   [ View source ]   [ Report asset ]                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Technical (metadata + identifiers)

### 6) Technical section (copyable fields)

```text
┌────────────────────────────────── Asset Details ▸ Technical ──────────────────┐
│  ▾ Technical                                                                 │
│                                                                              │
│  Asset ID:     z00z:asset:0x...A19F                             [ Copy ]     │
│  Contract:     0x.................................................. [ Copy ] │
│  Decimals:     9                                                              │
│  Icon:         (preview)   [ Change… ]                                        │
│                                                                              │
│  Address formats                                                             │
│   Receive format:   z00z://...                                                │
│   Memo supported:   [x]                                                       │
│                                                                              │
│  Advanced                                                                     │
│   [ Export public metadata ]   [ Re-fetch metadata ]                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## States (loading / empty / error)

### 7) Loading state

```text
┌──────────────────────────────────── Asset Details ────────────────────────────┐
│  Z00Z (Z00Z Asset)                                                            │
│                                                                              │
│  Loading asset details…   ────────────────▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                     │
│                                                                              │
│  [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8) Error state (failed metadata / node)

```text
┌──────────────────────────────────── Asset Details ────────────────────────────┐
│  ⊗ Failed to load asset details                                               │
│                                                                              │
│  Reason: node unreachable / invalid asset id                                  │
│                                                                              │
│  [ Retry ]                          [ Network settings ]                      │
└──────────────────────────────────────────────────────────────────────────────┘
```