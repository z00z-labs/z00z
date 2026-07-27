### 1) Fee Estimator (inline panel on Send/Swap/Build)

```text
┌────────────────────────────── Fee Estimation ────────────────────────────────┐
│ Fee policy:    [ Auto ▾ ]      Max fee: [ 0.0005 Z00Z ▾ ]                    │
│                                                                              │
│ Estimate:  ○ Not estimated yet                                               │
│                                                                              │
│ [ Estimate fee ]                                                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `tx.estimate_fee` — modal (details + presets)

### 2) Estimate fee dialog (speed presets + breakdown)

```text
┌────────────────────────────── Estimate Fee ──────────────────────────────────┐
│ Network: [ Mainnet ▾ ]         Endpoint: [ Auto (best) ▾ ]                    │
│                                                                              │
│ Tx type:  Send          Asset: Z00Z                                           │
│ Amount:   0.25000000                                                         │
│                                                                              │
│ Speed:   (•) Standard    ( ) Fast    ( ) Economy                             │
│                                                                              │
│ Slippage/deadline (if swap):  Slippage: [ 0.50% ▾ ]  Deadline: [ 10m ▾ ]      │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ Fee estimate                                                               │ │
│ │  Economy:   0.00008 Z00Z     ETA: ~30–90s                                  │ │
│ │  Standard:  0.00012 Z00Z     ETA: ~10–30s                                  │ │
│ │  Fast:      0.00020 Z00Z     ETA: ~3–10s                                   │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ Breakdown (Standard):                                                        │
│  Network fee:   0.00010 Z00Z                                                 │
│  Extra (size):  0.00002 Z00Z                                                 │
│                                                                              │
│ Apply as:  [ Standard ▾ ]                                                    │
│                                                                              │
│                         [ Cancel ]    [ Apply ]                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Custom fee override (advanced)

### 3) Custom fee input (only if user chooses “Custom”)

```text
┌────────────────────────────── Fee Policy ────────────────────────────────────┐
│ Fee policy:   [ Custom ▾ ]                                                   │
│                                                                              │
│ Fee amount:   [ 0.00012000 ]  Z00Z                                           │
│ Max fee:      [ 0.00050000 ]  Z00Z                                           │
│                                                                              │
│ Warning: too low fee may cause rejection or long pending time.               │
│                                                                              │
│ [ Re-estimate ]                         [ Apply ]                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Result states

### 4) Estimating (loading)

```text
┌────────────────────────────── Estimate Fee ──────────────────────────────────┐
│ Estimating fee…  ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯                         │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5) Quote stale / needs re-estimate (common on swap)

```text
┌────────────────────────────── Fee Warning ───────────────────────────────────┐
│ ○ Fee estimate is stale                                                      │
│ Reason: quote expired / network conditions changed                            │
│                                                                              │
│ [ Re-estimate fee ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6) Error / cannot estimate

```text
┌────────────────────────────── Estimate Fee ─ Error ──────────────────────────┐
│ ⊗ Failed to estimate fee                                                     │
│ Reason: network unavailable / tx incomplete / endpoint error                  │
│                                                                              │
│ [ Retry ]                          [ Network settings ]                      │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Tiny inline summary (used in Send/Build footer)

### 7) Applied fee pill / footer row

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Fee: 0.00012 Z00Z (Standard)    ETA: ~10–30s     [ Change ] [ Re-estimate ]  │
└──────────────────────────────────────────────────────────────────────────────┘
```