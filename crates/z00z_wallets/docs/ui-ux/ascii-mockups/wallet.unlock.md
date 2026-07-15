### 1) Unlock entry points (top bar + per-action prompt)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet: Primary Wallet ▾     Status: ○ Locked       [ Unlock ] [ Settings ]  │
└──────────────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────────────┐
│ ○ Wallet locked — unlock to continue                                         │
│ [ Unlock wallet ]                                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `wallet.unlock` — Unlock modal (password/session)

### 2) Unlock modal (password + session options)

```text
┌────────────────────────────── Unlock Wallet ─────────────────────────────────┐
│ Wallet: Primary Wallet                                                       │
│                                                                              │
│ Password:   [ *********************** ]   [ show ]                           │
│                                                                              │
│ Session:                                                                     │
│  Unlock for:  [ 5 min ▾ ]   ( ) Until app closes   ( ) Remember (keychain)   │
│                                                                              │
│ Auto-lock:  [ 5 min ▾ ]  (from settings)                                     │
│                                                                              │
│ Options:                                                                      │
│  [ ] Allow quick unlock (OS/biometric) (if available)                        │
│                                                                              │
│                      [ Cancel ]                     [ Unlock ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Unlock success (brief toast / banner)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● Wallet unlocked (auto-lock in 5 min)                                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Unlocked state in UI (actions enabled)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet: Primary Wallet ▾     Status: ● Unlocked     [ Lock ]   [ Settings ]  │
└──────────────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────────────┐
│  Actions:  [ Send ]  [ Receive ]  [ Swap ]  [ Export ]                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Wrong password / lockout

### 5a) Wrong password inline error

```text
┌────────────────────────────── Unlock Wallet ─────────────────────────────────┐
│ Wallet: Primary Wallet                                                       │
│                                                                              │
│ Password:   [ *********************** ]   [ show ]                           │
│ ⊗ Wrong password                                                             │
│                                                                              │
│ Attempts: 2 / 5                                                              │
│                                                                              │
│ [ Unlock ]   [ Cancel ]                                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5b) Too many attempts (cooldown)

```text
┌────────────────────────────── Unlock Wallet ─────────────────────────────────┐
│ ⊗ Too many failed attempts                                                   │
│                                                                              │
│ Try again in: 00:30                                                          │
│                                                                              │
│ [ Close ]                                                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) “Unlock required” interstitial (for sensitive actions)

### 6a) Before “Export / Show seed phrase / Send”

```text
┌────────────────────────────── Unlock Required ───────────────────────────────┐
│ This action requires an unlocked wallet:                                     │
│  • Send / Sign                                                               │
│  • Export wallet / Backup                                                    │
│  • Show seed phrase                                                          │
│                                                                              │
│ [ Unlock now ]   [ Cancel ]                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 7) Optional: quick unlock (OS/biometric placeholder)

```text
┌────────────────────────────── Unlock Wallet ─────────────────────────────────┐
│ Wallet: Primary Wallet                                                       │
│                                                                              │
│ Quick unlock:  [ Use OS / Biometric unlock ]                                 │
│                                                                              │
│ Or password: [ *********************** ]   [ show ]                          │
│                                                                              │
│                      [ Cancel ]                     [ Unlock ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```