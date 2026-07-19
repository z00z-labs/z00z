### 1) Staking main screen (egui: overview + actions)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]                      │
├──────────────────────────────────────────────────────────────────────────────┤
│  Staking (Z00Z)                                                [ ⟳ Refresh ] │
│                                                                              │
│ ┌───────────────────────────────────────────┐  ┌───────────────────────────┐ │
│ │ Overview                                  │  │ Position                  │ │
│ │  Staked:        10.00000000 Z00Z          │  │ Validator:  AlphaNode     │ │
│ │  Rewards:        0.12000000 Z00Z          │  │ Status:     ● Active      │ │
│ │  APR (est):      7.5%                     │  │ Since:      2025-12-01    │ │
│ │  Unbonding:      0.00000000 Z00Z          │  │                           │ │
│ └───────────────────────────────────────────┘  └───────────────────────────┘ │
│                                                                              │
│  Actions:  [ Stake ]   [ Claim rewards ]   [ Unstake ]   [ View validators ] │
│                                                                              │
│  ▾ Recent staking activity                                                   │
│   ┌───────────────────────────────────────────────────────────────────────┐  │
│   │ Time       │ Type        │ Amount       │ Status     │ Ref            │  │
│   ├────────────┼─────────────┼──────────────┼────────────┼────────────────┤  │
│   │ 12:10:02   │ Reward      │ +0.02000000  │ Confirmed  │ 0x..12         │  │
│   │ 08:44:11   │ Stake       │ 10.00000000  │ Confirmed  │ 0x..A9         │  │
│   └───────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Stake flow

### 2) Stake dialog (amount + validator + fee)

```text
┌──────────────────────────────────── Stake Z00Z ──────────────────────────────┐
│ From wallet:   [ Primary Wallet ▾ ]                                          │
│ Available:     1.20000000 Z00Z                                               │
│                                                                              │
│ Amount to stake:  [ 1.00000000 ]   ( ) Max                                   │
│                                                                              │
│ Validator:        [ AlphaNode ▾ ]   [ View list ]                             │
│                                                                              │
│ Fee policy:       [ Auto ▾ ]     Est. fee: ~0.00012 Z00Z                      │
│                                                                              │
│ Options:                                                                      │
│  [x] Auto-compound rewards (if supported)                                     │
│                                                                              │
│                           [ Cancel ]    [ Preview ]                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 3) Stake preview (confirm)

```text
┌────────────────────────────── Stake ▸ Preview ───────────────────────────────┐
│ Validator: AlphaNode                                                         │
│ Stake amount: 1.00000000 Z00Z                                                │
│ Fee:          0.00012000 Z00Z                                                │
│ Total spend:  1.00012000 Z00Z                                                │
│                                                                              │
│ Notes:                                                                       │
│  • Staked funds may be locked until unstake/unbonding completes.             │
│                                                                              │
│                       [ Back ]     [ Confirm stake ]                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Validators list

### 4) Validators (table + filters)

```text
┌────────────────────────────── Validators (Z00Z) ─────────────────────────────┐
│ Search: [ alpha... ]   Sort: [ APR ▾ ]   Filters: [x] Active  [ ] Hide risky │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ Name        │ APR (est) │ Fee │ Uptime │ Stake cap │ Status │ Select     │ │
│ ├─────────────┼───────────┼─────┼────────┼───────────┼────────┼────────────┤ │
│ │ AlphaNode   │ 7.5%      │ 5%  │ 99.9%  │  OK       │ ●      │ [ Select ] │ │
│ │ BetaStake   │ 7.1%      │ 3%  │ 98.7%  │  Near     │ ●      │ [ Select ] │ │
│ │ Gamma       │ 6.8%      │ 2%  │ 97.2%  │  OK       │ ●      │ [ Select ] │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ [ Close ]                                                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Claim rewards

### 5) Claim rewards dialog

```text
┌────────────────────────────── Claim Rewards ─────────────────────────────────┐
│ Wallet:     Primary Wallet                                                   │
│ Validator:  AlphaNode                                                        │
│ Available rewards:  0.12000000 Z00Z                                          │
│                                                                              │
│ Claim amount:  (•) All   ( ) Custom: [ 0.05000000 ]                          │
│                                                                              │
│ Destination:  (•) Wallet balance   ( ) Auto-restake                          │
│                                                                              │
│ Fee policy:   [ Auto ▾ ]    Est. fee: ~0.00008 Z00Z                          │
│                                                                              │
│                         [ Cancel ]   [ Claim ]                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## States (loading / error / locked)

### 6) Loading staking info

```text
┌──────────────────────────────────── Staking ─────────────────────────────────┐
│ Loading staking state…   ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                   │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Wallet locked (blocked)

```text
┌──────────────────────────────────── Staking ─────────────────────────────────┐
│ ○ Wallet is locked                                                           │
│                                                                              │
│ Unlock required to stake/claim/unstake.                                       │
│                                                                              │
│ [ Unlock wallet ]                                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8) Error (network / validator list unavailable)

```text
┌──────────────────────────────────── Staking ─────────────────────────────────┐
│ ⊗ Failed to load staking data                                                │
│ Reason: node unreachable / staking module not available                       │
│                                                                              │
│ [ Retry ]                          [ Network settings ]                      │
└──────────────────────────────────────────────────────────────────────────────┘
```