### 1) Unstake main (egui: position + unbonding info)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]                      │
├──────────────────────────────────────────────────────────────────────────────┤
│  Unstake (Z00Z)                                                 [ ⟳ Refresh ] │
│                                                                              │
│ ┌───────────────────────────────────────────┐  ┌───────────────────────────┐ │
│ │ Current position                          │  │ Unbonding rules           │ │
│ │  Validator:   AlphaNode                   │  │ Unbonding period: 7 days  │ │
│ │  Staked:      10.00000000 Z00Z            │  │ Claims available after:   │ │
│ │  Rewards:      0.12000000 Z00Z            │  │   2025-12-27              │ │
│ │  Status:      ● Active                    │  │ Early cancel: Not allowed │ │
│ └───────────────────────────────────────────┘  └───────────────────────────┘ │
│                                                                              │
│  Unstake amount:  [ 1.00000000 ]   ( ) Max                                   │
│                                                                              │
│  Destination after unbonding:                                                │
│   (•) Return to wallet balance                                               │
│   ( ) Auto-restake to same validator                                         │
│                                                                              │
│  Fee policy:  [ Auto ▾ ]    Est. fee: ~0.00010 Z00Z                          │
│                                                                              │
│  Actions:   [ Preview unstake ]   [ Cancel ]                                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Preview & confirm

### 2) Unstake preview dialog

```text
┌──────────────────────────── Unstake ▸ Preview ───────────────────────────────┐
│ Validator:  AlphaNode                                                        │
│ Unstake:    1.00000000 Z00Z                                                  │
│ Fee:        0.00010000 Z00Z                                                  │
│                                                                              │
│ Unbonding period: 7 days                                                     │
│ Funds available on: 2025-12-27                                               │
│                                                                              │
│ Notes:                                                                       │
│  • During unbonding, funds may not be transferable.                          │
│                                                                              │
│                       [ Back ]     [ Confirm unstake ]                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Unbonding queue (active unstakes)

### 3) Unbonding queue panel (list + claim when ready)

```text
┌────────────────────────────── Unbonding Queue ───────────────────────────────┐
│  ▾ Unbonding requests                                                        │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ Started     │ Amount         │ Available on  │ Status        │ Action   │  │
│  ├─────────────┼────────────────┼──────────────┼───────────────┼──────────┤  │
│  │ 2025-12-20  │ 1.00000000     │ 2025-12-27   │ ● Unbonding   │ [ — ]    │  │
│  │ 2025-12-12  │ 0.50000000     │ 2025-12-19   │ ● Ready       │ [ Claim ]│  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  Claim destination:  [ Wallet balance ▾ ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## States & validation

### 4) Wallet locked (blocked)

```text
┌──────────────────────────────────── Unstake ─────────────────────────────────┐
│ ○ Wallet is locked                                                           │
│                                                                              │
│ Unlock required to unstake or claim.                                          │
│                                                                              │
│ [ Unlock wallet ]                                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5) Invalid amount / exceeds staked

```text
┌──────────────────────────────────── Unstake ─────────────────────────────────┐
│ Unstake amount: [ 20.00000000 ]                                              │
│ ⊗ Amount exceeds staked balance                                               │
│ Staked: 10.00000000 Z00Z                                                     │
│                                                                              │
│ [ Use Max ]   [ Fix amount ]                                                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6) Loading staking state

```text
┌──────────────────────────────────── Unstake ─────────────────────────────────┐
│ Loading staking position…   ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Error (module unavailable / network)

```text
┌──────────────────────────────────── Unstake ─────────────────────────────────┐
│ ⊗ Failed to load unstake data                                                │
│ Reason: node unreachable / staking not supported on this network              │
│                                                                              │
│ [ Retry ]                          [ Network settings ]                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Unstake progress + result

### 8) Unstaking (progress)

```text
┌──────────────────────────────────── Unstaking… ──────────────────────────────┐
│ Creating unstake request…                                                    │
│                                                                              │
│ [..] Build transaction                                                       │
│ [..] Submit to network                                                       │
│ [..] Await acknowledgment                                                    │
│                                                                              │
│ Progress: ────────────────▮▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯                                 │
│                                                                              │
│                          [ Hide ]   [ Cancel ]                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9) Success (request created)

```text
┌──────────────────────────────────── Unstake Done ────────────────────────────┐
│ ● Unstake request submitted                                                  │
│                                                                              │
│ Amount:  1.00000000 Z00Z                                                     │
│ Status:  Unbonding (7 days)                                                  │
│ Available on: 2025-12-27                                                     │
│ Ref: 0x..D2                                                    [ Copy ]      │
│                                                                              │
│ [ View tx details ]   [ Open staking ]   [ Close ]                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Claim (when ready) mini-dialog

### 10) Claim unbonded funds

```text
┌────────────────────────────── Claim Unbonded ────────────────────────────────┐
│ Request: 2025-12-12   Amount: 0.50000000 Z00Z                                │
│                                                                              │
│ Destination: [ Wallet balance ▾ ]                                            │
│ Fee policy:  [ Auto ▾ ]   Est. fee: ~0.00008 Z00Z                            │
│                                                                              │
│                       [ Cancel ]     [ Claim ]                               │
└──────────────────────────────────────────────────────────────────────────────┘
```