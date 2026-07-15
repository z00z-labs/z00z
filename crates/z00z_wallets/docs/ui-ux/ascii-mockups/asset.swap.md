### 1) Swap screen (egui: form left + quote/summary right)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]                      │
├──────────────────────────────────────────────────────────────────────────────┤
│  Swap                                                           [ ⟳ Refresh ] │
│                                                                              │
│ ┌───────────────────────────────────────────┐  ┌───────────────────────────┐ │
│ │ From                                     │  │ Quote / Summary            │ │
│ │  Wallet:   [ Primary Wallet ▾ ]          │  │ Rate:     1 Z00Z ≈ 80 USTC  │ │
│ │  Asset:    [ Z00Z ▾ ]        [ ⇄ ]       │  │ Route:    DEX-A → DEX-B     │ │
│ │  Amount:   [ 0.25000000 ]   ( ) Max      │  │ Fee est:  ~0.00030 Z00Z     │ │
│ │  Balance:  1.20000000 Z00Z               │  │ Slippage: 0.50%             │ │
│ │                                           │  │ Min recv: 19.90000000 USTC │ │
│ │ To                                        │  │ Price impact: 0.12%         │ │
│ │  Asset:    [ USTC ▾ ]                     │  │ Status:   ● Quote ready     │ │
│ │  You receive (est):  20.00000000 USTC     │  └───────────────────────────┘ │
│ │                                           │
│ │ Slippage tolerance:  [ 0.50%  ▾ ]         │
│ │ Deadline:            [ 10 min ▾ ]         │
│ │ [x] Use best route                        │
│ │ [ ] Prefer verified pools                 │
│ └───────────────────────────────────────────┘
│                                                                              │
│  Actions:   [ Get quote ]   [ Preview swap ]   [ Clear ]                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Quote details (collapsing headers)

### 2) Quote breakdown (route + fees)

```text
┌────────────────────────────── Quote Details ▾ ───────────────────────────────┐
│  ▾ Quote breakdown                                                           │
│   Rate:              1 Z00Z ≈ 80.0000 USTC                                   │
│   Price impact:      0.12%                                                   │
│   Liquidity fee:     0.00020 Z00Z                                            │
│   Network fee:       0.00010 Z00Z                                            │
│   Total fee est:     0.00030 Z00Z                                            │
│                                                                              │
│  ▾ Route                                                                     │
│   Hop #1: Z00Z → USDC  via Pool A                                            │
│   Hop #2: USDC → USTC via Pool B                                            │
│                                                                              │
│  Min received (after slippage 0.50%): 19.90000000 USTC                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Swap preview & confirmation

### 3) Preview dialog (confirm swap)

```text
┌────────────────────────────── Swap ▸ Preview ────────────────────────────────┐
│ From:  0.25000000 Z00Z                                                       │
│ To:    ≈ 20.00000000 USTC                                                    │
│                                                                              │
│ Min received:  19.90000000 USTC     (slippage 0.50%)                         │
│ Deadline:      10 minutes                                                   │
│ Fees (est):    0.00030 Z00Z                                                  │
│ Price impact:  0.12%                                                         │
│ Route:         DEX-A → DEX-B                                                 │
│                                                                              │
│ [ ] I understand price may change before execution                            │
│                                                                              │
│                       [ Back ]     [ Confirm swap ]                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Token selector (asset picker inside swap)

### 4) Asset picker modal (for From/To)

```text
┌──────────────────────────────────── Select Asset ────────────────────────────┐
│ Search: [  type symbol/name...                               ]               │
│ Filters: [x] Verified only   [x] Hide spam   Network: [ Mainnet ▾ ]          │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ Z00Z     Balance: 1.2345      Verified ●                                │  │
│  │ USTC     Balance: 99.1000     Verified ●                                │  │
│  │ GOLD     Balance: 0.0500      Verified ●                                │  │
│  │ ABC      Balance: 10.0000     Unverified ○                              │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│                         [ Cancel ]        [ Select ]                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## States & validation

### 5) No quote yet (empty state)

```text
┌──────────────────────────────────── Swap ────────────────────────────────────┐
│ Enter amount and press [ Get quote ].                                        │
│                                                                              │
│ Status: ○ Waiting for input                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6) Quoting (loading)

```text
┌──────────────────────────────────── Swap ────────────────────────────────────┐
│ Getting quote…   ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                         │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Insufficient funds / amount too small

```text
┌──────────────────────────────────── Swap ────────────────────────────────────┐
│ ⊗ Cannot quote swap                                                          │
│                                                                              │
│ Reason: insufficient balance / amount too small / pool liquidity too low      │
│                                                                              │
│ [ Reduce amount ]   [ Change route settings ]   [ Retry ]                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8) High price impact warning

```text
┌──────────────────────────────────── Swap ─ Warning ──────────────────────────┐
│ ⚠ High price impact detected: 5.8%                                           │
│                                                                              │
│ Min received: 18.84000000 USTC                                               │
│                                                                              │
│ (•) Cancel                                                                   │
│ ( ) Continue anyway                                                          │
│                                                                              │
│                     [ Back ]                    [ Continue ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Executing swap (progress + result)

### 9) Executing (build + submit)

```text
┌──────────────────────────────────── Swapping… ───────────────────────────────┐
│ Executing swap                                                               │
│                                                                              │
│ [✓] Build transaction                                                        │
│ [..] Submit to network                                                       │
│ [..] Await confirmation                                                      │
│                                                                              │
│ Progress: ────────────────▮▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯                                 │
│                                                                              │
│                          [ Hide ]   [ Cancel ]                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 10) Success

```text
┌──────────────────────────────────── Swap Done ───────────────────────────────┐
│ ● Swap submitted                                                             │
│                                                                              │
│ Spent:   0.25000000 Z00Z                                                     │
│ Got:     ≈ 20.00000000 USTC                                                  │
│ Status:  Pending confirmation                                                │
│ TxID:    0x..C9                                               [ Copy ]       │
│                                                                              │
│ [ View tx details ]   [ Back to assets ]   [ Swap again ]                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 11) Failed

```text
┌──────────────────────────────────── Swap ─ Error ────────────────────────────┐
│ ⊗ Swap failed                                                                │
│ Reason: slippage exceeded / route unavailable / network timeout               │
│                                                                              │
│ [ Retry ]   [ Increase slippage ]   [ Get new quote ]   [ Network settings ] │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```