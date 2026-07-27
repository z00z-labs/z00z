### 1) Global pending badge (top nav)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions (3)]  [Send]  [Receive]  [Settings]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 2) Tab-level badge (Transactions view)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Transactions                                                                │
│  Tabs:  [ History ]    [ Pending ●3 ]                                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Sidebar badge (if you use left rail)

```text
┌───────────────────────────────┐
│  ▸ Assets                      │
│    Transactions        ●3      │
│    Settings                     │
└───────────────────────────────┘
```

------

## 4) Status bar indicator (bottom)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● Connected  |  Pending: ●3  |  Sync: ● Synced  |  Last update: 12:41:12     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) In-app banner summary (click → opens Pending tab)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● You have 3 pending transactions.   [ Open pending ]   [ Dismiss ]          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Badge states (colors implied by icons)

```text
Pending badge variants:
  ●3   = normal pending
  ⚠1   = at least one retryable failure
  ⊗1   = at least one hard failure (needs attention)
  ○0   = none (badge hidden)
```

------

## 7) Mini popover on hover/click (quick list)

```text
┌────────────────────────────── Pending (3) ───────────────────────────────────┐
│ ● Broadcasting    Send  0.2500 Z00Z                         [ Open ]         │
│ ● Pending ACK     Swap  0.1000 Z00Z                         [ Open ]         │
│ ⚠ Retryable fail   Send  5.0000 USTC                         [ Open ]         │
│                                                                              │
│ [ Go to Pending tab ]                                                        │
└──────────────────────────────────────────────────────────────────────────────┘
```