## App_Logout_Events-TBD

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]          [ Exit ⎋ ]  │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  (click [ Exit ⎋ ])                                                          │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 1) Exit confirm (basic)

```text
┌────────────────────────────────────── Exit ──────────────────────────────────┐
│ Exit Z00Z Wallet?                                                             │
│                                                                              │
│  • Wallet UI will close.                                                      │
│  • No funds are moved.                                                        │
│                                                                              │
│  [ ] Lock wallet on exit                                                      │
│                                                                              │
│                           [ Cancel ]    [ Exit now ]                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 2) Exit with pending ops (choice)

```text
┌────────────────────────────────────── Exit ──────────────────────────────────┐
│ You have pending operations:                                                  │
│   • Pending transactions:        2                                            │
│   • Sync in progress:            Yes                                          │
│                                                                              │
│ Choose an action:                                                             │
│   (•) Exit anyway                                                             │
│   ( ) Exit and cancel pending (recommended if offline)                        │
│   ( ) Go to Pending list                                                      │
│                                                                              │
│                           [ Back ]      [ Continue ]                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 3) Graceful shutdown progress (after confirm)

```text
┌───────────────────────────────────── Exiting ────────────────────────────────┐
│ Graceful shutdown…                                                            │
│                                                                              │
│  [✓] Save UI state                                                            │
│  [✓] Flush logs                                                               │
│  [..] Close RPC connections                                                   │
│  [..] Lock wallet session                                                     │
│                                                                              │
│  Progress:  62%    ────────────────▮▮▮▮▮▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯                        │
│                                                                              │
│                           [ Force quit ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 4) “Danger zone” inside Settings (entry point to Exit)

```text
┌────────────────────────────────── Settings ▸ General ─────────────────────────┐
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  ▸ General            │  │  ...                                          │ │
│ │    Appearance         │  │                                               │ │
│ │    Security           │  │  ───────────────────────────────────────────  │ │
│ │    Notifications      │  │  Danger zone                                  │ │
│ │    Network            │  │  [ Exit application ]                          │ │
│ │    Advanced           │  │                                               │ │
│ │    About              │  │                                               │ │
│ └───────────────────────┘  └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```
