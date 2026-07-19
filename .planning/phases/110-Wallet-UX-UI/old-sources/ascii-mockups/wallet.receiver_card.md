# Wallet Receiver Card Mockup

## 1) Receive screen (where the receiver card lives)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Receive                                                          [ ← Back ] │
├──────────────────────────────────────────────────────────────────────────────┤
│ Wallet: [ Primary Wallet ▾ ]          Asset: [ Z00Z ▾ ]   Network: Mainnet    │
│                                                                              │
│ Current receiver card:                                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ z00z1q................................................................. │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│  [ Copy ]   [ Show QR ]   [ Share… ]                                         │
│                                                                              │
│ Request (optional):                                                          │
│  Amount: [ 0.25000000 ]   Memo: [ optional…________________ ]                │
│  [ Create payment request ]                                                  │
│                                                                              │
│ Actions:  [ New receiver card ]   [ Receiver history ]                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `wallet.receiver_card` — Receiver card confirmation + options

### 2) New receiver card (options modal)

```text
┌────────────────────────────── New Receiver Card ─────────────────────────────┐
│ Wallet: Primary Wallet        Asset: Z00Z                                     │
│                                                                              │
│ Card type:      [ Stealth ▾ ]                                                 │
│  [ ] One-time receiver card (recommended for privacy)                         │
│  [ ] Label this card: [ groceries / invoice #____ ]                           │
│                                                                              │
│ Generate:                                                                    │
│  (•) Now                                                                     │
│  ( ) After confirming (manual)                                                │
│                                                                              │
│                      [ Cancel ]                    [ Generate ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) New receiver card created (result panel)

```text
┌────────────────────────────── Receiver Card Ready ───────────────────────────┐
│ ● New receiver card                                                          │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ z00z1q.................................................................   │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ [ Copy ]   [ Show QR ]   [ Share… ]                                          │
│                                                                              │
│ Label: [ __________________________ ]   [ Save label ]                       │
│                                                                              │
│                           [ Done ]                                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Show QR (modal)

```text
┌──────────────────────────────── Receive (QR) ────────────────────────────────┐
│ Wallet: Primary Wallet        Asset: Z00Z                                     │
│                                                                              │
│     ┌─────────────────────────────────────────────────────────────────┐      │
│     │                                                                 │      │
│     │                       [   QR CODE LARGE   ]                     │      │
│     │                                                                 │      │
│     └─────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│ z00z1q.................................................................       │
│                                                                              │
│ [ Copy ]   [ Share… ]   [ Close ]                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Receiver history (list of generated receiver cards)

```text
┌────────────────────────────── Receiver History ──────────────────────────────┐
│ Wallet: Primary Wallet                                                       │
│                                                                              │
│ Search label: [ _____________ ]     Show: [ All ▾ ] (unused/used)             │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ Time              │ Receiver (short)       │ Label     │ Status │ Actions │ │
│ ├───────────────────┼────────────────────────┼───────────┼────────┼─────────┤ │
│ │ 2025-12-20 12:10  │ z00z1q..9f2a           │ invoice12 │ unused │ [ Copy ] │ │
│ │ 2025-12-19 18:03  │ z00z1q..c1b7           │ rent      │ used   │ [ Copy ] │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ [ Close ]                                                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Errors / states

### 6a) Wallet locked (can still receive; generating may require unlock depending on design)

```text
┌────────────────────────────────── Receive ───────────────────────────────────┐
│ ○ Wallet is locked                                                           │
│                                                                              │
│ You can still show the last card. Creating a new receiver may require unlock. │
│                                                                              │
│ [ Unlock wallet ]   [ Use current receiver ]                                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6b) Generating (brief spinner overlay)

```text
┌───────────────────────────────┐
│ Generating receiver card…      │
└───────────────────────────────┘
```

### 6c) Generation error

```text
┌────────────────────────────── Receive ─ Error ───────────────────────────────┐
│ ⊗ Failed to generate receiver card                                           │
│ Reason: keystore error / derivation index unavailable                          │
│                                                                              │
│ [ Retry ]   [ View logs ]                                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```
