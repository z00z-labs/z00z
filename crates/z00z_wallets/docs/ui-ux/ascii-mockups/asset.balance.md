### 1) Assets header (portfolio balance + refresh)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]                      │
├──────────────────────────────────────────────────────────────────────────────┤
│  Portfolio                                                                    │
│                                                                              │
│  Total balance:   $ 1,234.56                              [ ⟳ Refresh ]      │
│  Last update:     12:34:56                                  Sync: ●●○        │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  Z00Z      1.23450000        ≈ $123.45         24h:  +2.1%              │  │
│  │  USTC     99.10000000        ≈ $ 99.10         24h:  +0.0%              │  │
│  │  GOLD      0.05000000        ≈ $ 12.34         24h:  -0.3%              │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 2) Asset card (per-asset balance widget)

```text
┌──────────────────────────────────── Asset ───────────────────────────────────┐
│  Z00Z  (Z00Z Asset)                                            [ ⟳ Refresh ] │
│  Balance:      1.23450000 Z00Z                 Fiat:  ≈ $123.45              │
│  Available:    1.20000000 Z00Z                 Locked: 0.03450000           │
│                                                                              │
│  Status:  ● Synced      Last update: 12:34:56      Network: Mainnet          │
│                                                                              │
│  Quick actions:  [ Send ]  [ Receive ]  [ Swap ]  [ Details ]                │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Top “balance strip” widget (compact header)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Total: $ 1,234.56     Available: $ 1,210.11     Pending: $ 24.45   [ ⟳ ]     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Balance states (loading / stale / error)

#### 4a) Loading (after refresh)

```text
┌──────────────────────────────────── Portfolio ───────────────────────────────┐
│ Total balance:   …                                           [ ⟳ Refresh ]   │
│ Updating…  ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                                  │
│ Last update: 12:34:56  (in progress)                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

#### 4b) Stale data (offline / no sync)

```text
┌──────────────────────────────────── Portfolio ───────────────────────────────┐
│ Total balance:   $ 1,234.56                               [ ⟳ Refresh ]      │
│ Status:  ○ Offline  • Showing cached balances (last: 10:12:03)               │
│                                                                              │
│ [ Open Network settings ]        [ Retry ]                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

#### 4c) Error (RPC failure)

```text
┌──────────────────────────────────── Portfolio ───────────────────────────────┐
│ Total balance:   —                                         [ ⟳ Refresh ]     │
│ ⊗ Failed to load balances.                                                    │
│                                                                              │
│ Details:  timeout / node unreachable                                          │
│                                                                              │
│ [ Retry ]                          [ Network settings ]                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 5) Balance breakdown (expandable section like egui collapsing header)

```text
┌──────────────────────────────────── Balance ▸ Breakdown ─────────────────────┐
│  ▾ Breakdown                                                                  │
│    Available:        1.20000000 Z00Z     ≈ $120.00                            │
│    Locked:           0.03450000 Z00Z     ≈ $  3.45                            │
│    Pending incoming: 0.01000000 Z00Z     ≈ $  1.00                            │
│    Pending outgoing: 0.01000000 Z00Z     ≈ $  1.00                            │
│                                                                              │
│  [ ⟳ Refresh ]                                                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 6) Inline refresh affordances (egui-like small controls)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Balance: 1.23450000 Z00Z   ≈ $123.45     [ ⟳ ]  [ Auto-refresh ▾ ]           │
└──────────────────────────────────────────────────────────────────────────────┘
```