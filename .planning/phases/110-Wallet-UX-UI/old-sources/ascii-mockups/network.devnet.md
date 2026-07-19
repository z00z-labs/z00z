Ниже — ASCII мокапы для **App Devnet view / Explorer** (в стиле egui). Это именно “обзор сети + встроенный мини-explorer”, без кода.

------

## 1) Top bar network switcher + Devnet badge

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Z00Z Wallet   Network: [ Devnet ▾ ]  Endpoint: [ Auto ▾ ]   ● Connected      │
│ Wallet: Primary ▾     Pending: ●3     Sync: ● Synced        [ Settings ]     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) Devnet Dashboard (network overview)

```text
┌────────────────────────────── Network ▸ Devnet ──────────────────────────────┐
│ Network: Devnet              Endpoint: Auto (best)            [ ⟳ Refresh ]  │
├──────────────────────────────────────────────────────────────────────────────┤
│ Status: ● Connected   Latency: 182ms   Height: 12,345,678   Finality: ~2s    │
│ Peers: 18            RPC: OK          Indexer: OK (local)    Mempool: OK     │
│                                                                              │
│ Quick actions:  [ Open Explorer ]  [ Ping endpoint ]  [ Switch endpoint ]    │
│                                                                              │
│ ┌───────────────────────────────┐   ┌─────────────────────────────────────┐  │
│ │ Latest blocks (Devnet)        │   │ Latest transactions                  │  │
│ │  #123456  12:41:10  2 tx      │   │  0x..B7  Send   Pending              │  │
│ │  #123455  12:41:08  0 tx      │   │  0x..A9  Swap   Confirmed            │  │
│ │  #123454  12:41:06  1 tx      │   │  0x..91  Recv   Confirmed            │  │
│ │  [ View all ]                 │   │  [ View all ]                        │  │
│ └───────────────────────────────┘   └─────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Explorer (search-first UI)

```text
┌───────────────────────────────── Explorer (Devnet) ──────────────────────────┐
│ Search: [ txid / block / address / commitment / key image______________ ]     │
│ Type:   [ Auto ▾ ]   Scope: [ Devnet ▾ ]    [ Search ]   [ Paste ] [ QR ]    │
├──────────────────────────────────────────────────────────────────────────────┤
│ Quick links:  [ Latest blocks ]  [ Latest tx ]  [ Mempool ]  [ My activity ] │
│                                                                              │
│ Results:                                                                     │
│  ○ Enter a query, or pick a quick link.                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Explorer → Latest blocks (table)

```text
┌────────────────────────────── Explorer ▸ Latest Blocks ──────────────────────┐
│ Network: Devnet                                            [ ⟳ Refresh ]     │
│ Range: [ 100 ▾ ] blocks     Show: [ All ▾ ]                                  │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ Height    │ Time      │ Tx │ Size │ Producer │ Actions                    │ │
│ ├───────────┼───────────┼────┼──────┼──────────┼────────────────────────────┤ │
│ │ 123456    │ 12:41:10  │ 2  │ 12KB │ vld..9a  │ [ Open ] [ Copy hash ]     │ │
│ │ 123455    │ 12:41:08  │ 0  │  8KB │ vld..21  │ [ Open ] [ Copy hash ]     │ │
│ │ 123454    │ 12:41:06  │ 1  │ 10KB │ vld..d0  │ [ Open ] [ Copy hash ]     │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│ Page: [ ◀ ]  1 / 42  [ ▶ ]                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Block details page (drill-down)

```text
┌────────────────────────────── Explorer ▸ Block #123456 ──────────────────────┐
│ Network: Devnet                                                [ ← Back ]   │
├──────────────────────────────────────────────────────────────────────────────┤
│ Height: 123456     Hash: blk..7c21                               [ Copy ]    │
│ Time:   2025-12-20 12:41:10     Tx count: 2     Size: 12 KB                 │
│ Producer: vld..9a                                               [ Copy ]    │
│ Prev: blk..7c20  [ Open ]     Next: blk..7c22  [ Open ]                      │
│                                                                              │
│ ▾ Transactions                                                               │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ TxID      │ Type │ Status    │ Asset │ Amount      │ Actions              │ │
│ ├───────────┼──────┼───────────┼───────┼─────────────┼──────────────────────┤ │
│ │ 0x..B7    │ Send │ Pending   │ Z00Z  │ 0.25000000  │ [ Open tx ]          │ │
│ │ 0x..91    │ Recv │ Confirmed │ USTC  │ 5.00000000  │ [ Open tx ]          │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Tx details in Explorer (read-only; “import into wallet”)

```text
┌────────────────────────────── Explorer ▸ Tx 0x..B7 ──────────────────────────┐
│ Network: Devnet                                                [ ← Back ]   │
├──────────────────────────────────────────────────────────────────────────────┤
│ Status: ● Pending confirmation    Block: (not yet)                            │
│ Type:   Send      Asset: Z00Z      Amount: 0.25000000                         │
│ Fee:    0.00012000 (est)                                                     │
│                                                                              │
│ To: z00z://recv?...                                           [ Copy ]       │
│ TxID: 0x..B7                                                   [ Copy ]      │
│                                                                              │
│ ▾ Proofs / commitments (advanced)                                            │
│  Commitment: Cmt..7b10  [ Copy ]                                             │
│  Key image:  KI..F1A0    [ Copy ]                                            │
│                                                                              │
│ Actions:  [ Open in wallet (if exists) ]  [ Save raw JSON ]  [ Report ]      │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 7) Mempool view (Devnet)

```text
┌────────────────────────────── Explorer ▸ Mempool ────────────────────────────┐
│ Network: Devnet                       Sort: [ Newest ▾ ]     [ ⟳ Refresh ]   │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ Time     │ TxID   │ Type │ Asset │ Fee      │ State          │ Actions    │ │
│ ├──────────┼────────┼──────┼───────┼──────────┼────────────────┼────────────┤ │
│ │ 12:41:10 │ 0x..B7 │ Send │ Z00Z  │ 0.00012  │ ● Broadcasting │ [ Open ]   │ │
│ │ 12:40:58 │ 0x..A9 │ Swap │ Z00Z  │ 0.00020  │ ● In mempool   │ [ Open ]   │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│ Hint: Devnet mempool can be unstable; refresh often.                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 8) “Open external explorer” button (если есть web link)

```text
┌────────────────────────────── External Explorer ─────────────────────────────┐
│ Devnet explorer URL detected                                                 │
│                                                                              │
│ [ Open in browser ]   [ Copy link ]                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 9) Devnet warning banner (to avoid user confusion)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ⚠ You are on DEVNET. Funds are test-only and may reset.   [ Switch to Mainnet ]│
└──────────────────────────────────────────────────────────────────────────────┘
```

Если хочешь — сделаю отдельный мокап **“My activity (Devnet)”**: вкладка, которая показывает *только* транзакции/коммитменты связанные с активным кошельком, но в режиме explorer (чисто read-only).
