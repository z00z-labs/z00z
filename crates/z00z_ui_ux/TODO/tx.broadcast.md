### 1) Entry point (Send/Swap/Build → “Submit to network”)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Transaction Ready                                                            │
├──────────────────────────────────────────────────────────────────────────────┤
│  Type:        Send                                                           │
│  Asset:       Z00Z                                                           │
│  Amount:      0.25000000                                                     │
│  Fee (est):   0.00012000                                                     │
│  Recipient:   z00z://recv?...                                                │
│                                                                              │
│  Signed:      ● Yes                                                          │
│  Local status: ● Built (not broadcast)                                       │
│                                                                              │
│  Actions:   [ Submit to network ]   [ Save draft ]   [ Back ]                │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `tx.broadcast` — broadcast modal

### 2) Broadcast dialog (network + endpoint + options)

```text
┌────────────────────────────── Submit to Network ─────────────────────────────┐
│ Network:     [ Mainnet ▾ ]                                                   │
│ Endpoint:    [ Auto (best) ▾ ]     Status: ● Connected                        │
│                                                                              │
│ Broadcast options:                                                           │
│  (•) Normal broadcast                                                        │
│  ( ) Private relay (if available)                                            │
│                                                                              │
│ Retry policy: [ 3 retries ▾ ]     Timeout: [ 10s ▾ ]                          │
│                                                                              │
│ After submit:                                                                │
│  [x] Show as Pending                                                         │
│  [x] Notify on confirmation                                                  │
│                                                                              │
│                              [ Cancel ]    [ Broadcast ]                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Progress + detailed steps

### 3) Broadcasting… (stepper + progress)

```text
┌────────────────────────────── Broadcasting… ─────────────────────────────────┐
│ Submitting transaction to network…                                           │
│                                                                              │
│ Endpoint: https://rpc1.example.org     Latency: 42 ms                         │
│                                                                              │
│  [✓] Validate tx format                                                      │
│  [✓] Select endpoint                                                         │
│  [..] Submit to mempool                                                      │
│  [..] Await ACK                                                              │
│                                                                              │
│ Progress: ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                                │
│                                                                              │
│                         [ Hide ]    [ Cancel ]                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Success (ACK received)

### 4) Broadcast success (pending + next actions)

```text
┌────────────────────────────── Submitted ─────────────────────────────────────┐
│ ● Transaction submitted                                                      │
│                                                                              │
│ TxID:   0x..B7                                               [ Copy ]        │
│ Status: Pending confirmation                                                 │
│                                                                              │
│ Next:                                                                        │
│  [ View tx details ]   [ Open pending list ]   [ Back to assets ]            │
│                                                                              │
│                                [ Close ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Failure modes

### 5) Network error / timeout (retry)

```text
┌──────────────────────────── Submit to Network ─ Error ───────────────────────┐
│ ⊗ Broadcast failed                                                           │
│                                                                              │
│ Reason: timeout / endpoint unreachable                                       │
│                                                                              │
│ Attempts: 2 / 3                                                              │
│ Endpoint: https://rpc2.example.org                                           │
│                                                                              │
│ Actions:                                                                     │
│  [ Retry now ]   [ Switch endpoint… ]   [ Save as draft ]                    │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6) Rejected by network (fee too low / invalid / already spent)

```text
┌──────────────────────────── Submit to Network ─ Rejected ────────────────────┐
│ ⊗ Transaction rejected                                                       │
│                                                                              │
│ Reason: fee too low / invalid signature / double-spend / expired             │
│                                                                              │
│ Suggestions:                                                                 │
│  [ Re-estimate fee ]   [ Rebuild tx ]   [ View details ]                     │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Advanced: endpoint picker (manual)

### 7) Choose endpoint (modal)

```text
┌────────────────────────────── Choose Endpoint ───────────────────────────────┐
│ Network: Mainnet                                                             │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ Endpoint                     │ Latency │ Status  │ Select                │ │
│ ├──────────────────────────────┼─────────┼─────────┼───────────────────────┤ │
│ │ https://rpc1.example.org      │ 42 ms   │ ● OK    │ (•)                   │ │
│ │ https://rpc3.example.org      │ 120 ms  │ ● OK    │ ( )                   │ │
│ │ https://rpc2.example.org      │ —       │ ⊗ FAIL  │ ( )                   │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ [ Close ]                                                        [ Use ]     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## “Hidden” mode banner (if user clicks Hide)

### 8) Background/pending banner (in-app)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● Broadcasting…  (2/4) Submit to mempool   [ View ] [ Cancel ]               │
└──────────────────────────────────────────────────────────────────────────────┘
```