### 1) Wallet list screen (switcher / hub)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  Wallets                                                          [ ⟳ Refresh ]│
├──────────────────────────────────────────────────────────────────────────────┤
│ Active wallet: [ Primary Wallet ▾ ]        Search: [ _____________ ]         │
│                                                                              │
│ Actions:  [ + Create ]  [ + Add existing ]  [ Import… ]  [ Recover seed… ]   │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ Name               │ Type       │ Status     │ Assets │ Last used │ Open  │ │
│ ├────────────────────┼────────────┼────────────┼────────┼───────────┼───────┤ │
│ │ Primary Wallet      │ Standard   │ ● Unlocked │ 4      │ Today     │[Open] │ │
│ │ Trading Wallet      │ Standard   │ ○ Locked   │ 2      │ Yesterday │[Open] │ │
│ │ Watch-only Wallet   │ Watch-only │ ○ Locked   │ 1      │ 2025-12-01│[Open] │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ Quick actions (selected wallet):                                             │
│  [ Unlock / Lock ]   [ Settings ]   [ Export ]   [ Delete ]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) Wallet switch dropdown (compact selector)

```text
┌────────────────────────────── Active wallet ─────────────────────────────────┐
│ Active wallet:                                                           ▾   │
│                                                                              │
│  • Primary Wallet            ● Unlocked                                       │
│  • Trading Wallet            ○ Locked                                         │
│  • Watch-only Wallet         ○ Locked                                         │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────   │
│  + Create new wallet                                                         │
│  + Add existing                                                              │
│  + Import wallet…                                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Wallet card layout (alternative view)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallets (Cards)                                                  [ + Add ]   │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────────────┐   ┌───────────────────────────────┐       │
│ │ Primary Wallet                │   │ Trading Wallet                │       │
│ │ Type: Standard                │   │ Type: Standard                │       │
│ │ Status: ● Unlocked            │   │ Status: ○ Locked              │       │
│ │ Assets: 4                     │   │ Assets: 2                     │       │
│ │                               │   │                               │       │
│ │ [ Open ] [ Send ] [ Receive ] │   │ [ Open ] [ Unlock ] [ Export ]│       │
│ └───────────────────────────────┘   └───────────────────────────────┘       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Inline wallet switcher (top bar chip)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet: Primary Wallet ▾     Pending: ●3     Sync: ● Synced     [ Settings ] │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Empty state (no wallets)

```text
┌────────────────────────────────── Wallets ───────────────────────────────────┐
│ No wallets on this device.                                                   │
│                                                                              │
│ [ Create new wallet ]   [ Recover from seed ]   [ Import backup file ]       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Loading / error states

### 6a) Loading list

```text
┌────────────────────────────────── Wallets ───────────────────────────────────┐
│ Loading wallets…  ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                       │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6b) Error loading

```text
┌────────────────────────────────── Wallets ─ Error ───────────────────────────┐
│ ⊗ Failed to load wallet list                                                 │
│ Reason: corrupted local store / permission denied                            │
│                                                                              │
│ [ Retry ]                          [ Open logs ]                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 7) Context menu (right-click / kebab on a row)

```text
┌────────────────────────────── Wallet Actions ────────────────────────────────┐
│ [ Open ]                                                                     │
│ [ Set as active ]                                                            │
│ [ Unlock ] / [ Lock ]                                                        │
│ [ Settings ]                                                                 │
│ [ Export ]                                                                   │
│ [ Delete ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```