### 1) Pending list (where “Cancel” lives)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Transactions                                                     [ ⟳ Refresh ]│
│  Tabs:  [ History ]  [ Pending ]                                             │
├──────────────────────────────────────────────────────────────────────────────┤
│ Pending (3)                                                                  │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ Time     │ Type │ Asset │ Amount      │ Status        │ Actions           │ │
│ ├──────────┼──────┼───────┼─────────────┼───────────────┼───────────────────┤ │
│ │ 12:41:10 │ Send │ Z00Z  │ 0.25000000  │ ● Broadcasting │ [ View ] [ Cancel]│ │
│ │ 12:32:05 │ Swap │ Z00Z  │ 0.10000000  │ ● Pending ACK  │ [ View ] [ Cancel]│ │
│ │ 11:58:22 │ Send │ USTC  │ 5.00000000  │ ● In mempool   │ [ View ] [ Cancel]│ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ Hint: Cancel is best-effort (may fail if already accepted by the network).   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `tx.cancel` — cancel confirmation

### 2) Cancel pending transaction (modal)

```text
┌────────────────────────────── Cancel Transaction ────────────────────────────┐
│ Tx: 0x..B7                                                     Status: Pending│
│ Type: Send      Asset: Z00Z      Amount: 0.25000000                          │
│                                                                              │
│ What do you want to cancel?                                                  │
│  (•) Local pending entry only (stop tracking)                                │
│  ( ) Attempt network cancel (replace-by-fee / cancel tx)                      │
│                                                                              │
│ Network cancel options (if supported):                                       │
│  Fee bump: [ Auto ▾ ]    Max fee: [ 0.0010 Z00Z ▾ ]                          │
│  Endpoint: [ Auto (best) ▾ ]                                                 │
│                                                                              │
│ [ ] I understand cancel may fail if tx is already accepted/confirmed.         │
│                                                                              │
│                      [ Back ]                    [ Cancel tx ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Cancel in progress

### 3) Cancelling… (stepper)

```text
┌────────────────────────────── Cancelling… ───────────────────────────────────┐
│ Cancelling pending transaction…                                              │
│                                                                              │
│  [✓] Mark local pending as cancelling                                        │
│  [..] Submit cancel/replace request (if selected)                             │
│  [..] Await response                                                         │
│                                                                              │
│ Progress: ────────────────▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯▯                               │
│                                                                              │
│                          [ Hide ]   [ Stop ]                                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Results

### 4) Cancel succeeded (local)

```text
┌────────────────────────────── Cancelled ─────────────────────────────────────┐
│ ● Pending transaction removed from list                                      │
│                                                                              │
│ Tx: 0x..B7                                                                    │
│                                                                              │
│ [ Back to pending ]   [ Close ]                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5) Network cancel succeeded (replaced)

```text
┌────────────────────────────── Cancel Submitted ──────────────────────────────┐
│ ● Cancel / replace request submitted                                         │
│                                                                              │
│ Original: 0x..B7                                                             │
│ Cancel tx: 0x..C1                                                [ Copy ]    │
│ Status: Pending confirmation                                                 │
│                                                                              │
│ [ View tx details ]   [ Back to pending ]                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6) Cancel failed (already confirmed / not supported)

```text
┌────────────────────────────── Cancel Failed ─────────────────────────────────┐
│ ⊗ Could not cancel transaction                                               │
│                                                                              │
│ Reason: already confirmed / cancel not supported / fee too low               │
│                                                                              │
│ Options:                                                                     │
│  [ View tx details ]   [ Remove local pending ]   [ Retry ]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Edge UI: tx becomes confirmed while cancel modal open

### 7) Status changed banner (live update)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● Status update: Transaction 0x..B7 is now Confirmed.   [ View ] [ Close ]   │
└──────────────────────────────────────────────────────────────────────────────┘
```