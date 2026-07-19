### 1) Pending tab (list + actions)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Transactions                                                     [ ⟳ Refresh ]│
│  Tabs:  [ History ]  [ Pending ]                                             │
├──────────────────────────────────────────────────────────────────────────────┤
│ Pending (3)                                                                  │
│                                                                              │
│ Search: [ txid / type / asset...____________ ]   Sort: [ Newest ▾ ]           │
│ Filters:  Status [ Any ▾ ]   Type [ Any ▾ ]   Asset [ Any ▾ ]                 │
│                                                                              │
│ Actions:  [ Cancel selected ]  [ Retry failed ]  [ Clear finished ]          │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ Sel │ Time     │ Type │ Asset │ Amount      │ State            │ Actions  │ │
│ ├─────┼──────────┼──────┼───────┼─────────────┼──────────────────┼──────────┤ │
│ │ [ ] │ 12:41:10 │ Send │ Z00Z  │ 0.25000000  │ ● Broadcasting   │ [ Open ] │ │
│ │ [ ] │ 12:32:05 │ Swap │ Z00Z  │ 0.10000000  │ ● Pending ACK    │ [ Open ] │ │
│ │ [ ] │ 11:58:22 │ Send │ USTC  │ 5.00000000  │ ⚠ Retryable fail │ [ Open ] │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ Selected: 0                                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) Pending row “Open” (mini details drawer on right)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Pending ▸ Tx 0x..B7                                              [ Close ]  │
├──────────────────────────────────────────────────────────────────────────────┤
│ Status: ● Broadcasting                                                       │
│ Type:   Send                                                                 │
│ Asset:  Z00Z        Amount: 0.25000000                                       │
│ Fee:    0.00012000 (est)                                                     │
│                                                                              │
│ Endpoint: Auto (best)                                                        │
│ Attempts: 1 / 3                                                              │
│ Last error: (none)                                                           │
│                                                                              │
│ Actions:  [ View full details ]  [ Cancel ]  [ Save draft ]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Bulk cancel (selected rows)

```text
┌────────────────────────────── Cancel Selected ───────────────────────────────┐
│ Cancel 2 pending transactions?                                                │
│                                                                              │
│ What to do:                                                                  │
│  (•) Remove from list only (stop tracking)                                   │
│  ( ) Attempt network cancel (best-effort)                                     │
│                                                                              │
│ [ ] I understand cancel may fail if already accepted by the network.          │
│                                                                              │
│                      [ Back ]                    [ Cancel selected ]         │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Retry failed (one-click action + modal)

```text
┌────────────────────────────── Retry Pending Tx ──────────────────────────────┐
│ Tx: 0x..A9     Status: ⚠ Retryable fail                                      │
│ Reason: timeout / endpoint unreachable                                        │
│                                                                              │
│ Endpoint: [ Auto (best) ▾ ]     Retry policy: [ 3 ▾ ]   Timeout: [ 10s ▾ ]    │
│                                                                              │
│ After retry: [x] Notify on confirmation                                       │
│                                                                              │
│                         [ Cancel ]                [ Retry now ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) “Clear finished” (remove confirmed/abandoned from pending)

```text
┌────────────────────────────── Clear Finished ────────────────────────────────┐
│ Remove finished items from Pending list?                                     │
│                                                                              │
│ Remove:  [x] Confirmed  [x] Failed (non-retryable)  [ ] Abandoned            │
│                                                                              │
│ (•) Keep in History                                                          │
│ ( ) Remove from History too (danger)                                          │
│                                                                              │
│                      [ Cancel ]                     [ Clear ]               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Empty state

```text
┌──────────────────────────────── Transactions ▸ Pending ──────────────────────┐
│ No pending transactions.                                                     │
│                                                                              │
│ [ Go to History ]   [ Send ]   [ Receive ]                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 7) States / errors

### 7a) Loading pending list

```text
┌──────────────────────────────── Transactions ▸ Pending ──────────────────────┐
│ Loading pending transactions…  ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯            │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7b) Network down (show cached)

```text
┌──────────────────────────────── Transactions ▸ Pending ──────────────────────┐
│ ○ Network unavailable — showing cached pending list.                         │
│                                                                              │
│ [ Retry ]                           [ Network settings ]                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 8) Row context menu (right-click / kebab)

```text
┌────────────────────────────── Pending Row Actions ───────────────────────────┐
│ [ Open details ]                                                             │
│ [ View full details ]                                                        │
│ [ Copy TxID ]                                                                │
│ [ Cancel ]                                                                   │
│ [ Retry ] (if retryable)                                                     │
│ [ Remove from pending ]                                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```