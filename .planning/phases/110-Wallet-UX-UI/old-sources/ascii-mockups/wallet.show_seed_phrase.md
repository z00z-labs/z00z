### 1) Entry point (Wallet Settings → Security)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet Settings ▸ Security                                       [ ← Back ]  │
├──────────────────────────────────────────────────────────────────────────────┤
│ Seed phrase                                                                   │
│  Your seed phrase controls all funds. Never share it.                         │
│                                                                              │
│  [ Show seed phrase… ]   (requires unlock)                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `wallet.session.show_seed_phrase(session, password, confirmation)` — gated flow

### 2) Gate 1 — unlock required (if locked)

```text
┌────────────────────────────── Unlock Required ───────────────────────────────┐
│ Wallet: Primary Wallet                                                       │
│                                                                              │
│ Unlock to view seed phrase.                                                  │
│                                                                              │
│ Password: [ *********************** ]   [ show ]                             │
│                                                                              │
│ [ Unlock ]   [ Cancel ]                                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Gate 2 — security warning + confirmation

```text
┌────────────────────────────── Security Warning ──────────────────────────────┐
│ ⚠ Your seed phrase gives FULL control of funds.                              │
│                                                                              │
│ Do NOT:                                                                      │
│  • share it                                                                  │
│  • screenshot it                                                             │
│  • copy/paste it into chat/apps                                              │
│                                                                              │
│ Safe storage: paper / hardware backup / offline vault.                       │
│                                                                              │
│ [ ] I understand the risks                                                    │
│                                                                              │
│                      [ Cancel ]                    [ Show seed ]            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Seed phrase display (masked-by-default)

### 4a) Seed phrase screen

```text
┌────────────────────────────── Seed Phrase ───────────────────────────────────┐
│ Wallet: Primary Wallet                                                       │
│                                                                              │
│ Visibility:  (•) Hidden   ( ) Reveal                                          │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │  1: ███████   2: ███████   3: ███████   4: ███████                        │ │
│ │  5: ███████   6: ███████   7: ███████   8: ███████                        │ │
│ │  9: ███████  10: ███████  11: ███████  12: ███████                        │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ Actions:  [ Reveal for 10s ]  [ Show QR ]  [ Close ]                          │
│                                                                              │
│ ⚠ Copy disabled by default (enable in settings).                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 4b) “Reveal for 10s” countdown overlay

```text
┌────────────────────────────── Seed Phrase ───────────────────────────────────┐
│ Revealed (auto-hide in 00:09)                                                │
│                                                                              │
│  1: word1    2: word2    3: word3    4: word4                                │
│  5: word5    6: word6    7: word7    8: word8                                │
│  9: word9   10: word10  11: word11  12: word12                               │
│                                                                              │
│ [ Hide now ]   [ Show QR ]                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Extra gate for copy / QR (optional “power user”)

### 5a) Enable copy (explicit)

```text
┌────────────────────────────── Copy Seed Phrase ──────────────────────────────┐
│ ⚠ Copying to clipboard is risky.                                             │
│ Clipboard may be read by other apps.                                         │
│                                                                              │
│ (•) Don’t copy                                                               │
│ ( ) Copy anyway                                                              │
│                                                                              │
│ [ Cancel ]  [ Copy ]                                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5b) QR display (large)

```text
┌────────────────────────────── Seed Phrase (QR) ──────────────────────────────┐
│ ⚠ Anyone who scans this QR controls your funds.                              │
│                                                                              │
│     ┌─────────────────────────────────────────────────────────────────┐      │
│     │                                                                 │      │
│     │                       [   QR CODE LARGE   ]                     │      │
│     │                                                                 │      │
│     └─────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│ [ Close ]                                                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Auto-hide on focus loss / lock event (banner)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ○ Seed phrase hidden (app lost focus / wallet locked).                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 7) Errors / edge cases

### 7a) Wallet has no seed (watch-only)

```text
┌────────────────────────────── Seed Phrase ───────────────────────────────────┐
│ ⊗ Seed phrase not available for this wallet                                  │
│ Reason: watch-only wallet / imported public key only                          │
│                                                                              │
│ [ Close ]                                                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7b) Unlock failed

```text
┌────────────────────────────── Unlock Required ─ Error ───────────────────────┐
│ ⊗ Wrong password                                                             │
│                                                                              │
│ Password: [ *********************** ]   [ show ]                             │
│                                                                              │
│ [ Try again ]   [ Cancel ]                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```