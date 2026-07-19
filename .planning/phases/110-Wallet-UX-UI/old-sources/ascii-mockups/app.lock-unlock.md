Ниже — **единый ASCII-мокап “App Lock / Unlock”** (уровень приложения, не конкретного кошелька). Ориентация под **egui**: top-bar, боковые табы, модалки, баннеры, тосты.

------

## 1) App LOCK overlay (когда приложение заблокировано)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Z00Z Wallet                                                        ○ Locked  │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                            ┌──────────────────────────┐                      │
│                            │        APP LOCKED         │                      │
│                            │                          │                      │
│                            │   Unlock to continue      │                      │
│                            │                          │                      │
│                            │  Password: [ ********** ] │                      │
│                            │            [ show ]       │                      │
│                            │                          │                      │
│                            │  Session: [ 10 min ▾ ]    │                      │
│                            │  Auto-lock: 5 min         │                      │
│                            │                          │                      │
│                            │  [ Unlock ]  [ Exit ]     │                      │
│                            └──────────────────────────┘                      │
│                                                                              │
│ Tip: App lock protects UI and local secrets; wallet keys may still be locked.│
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) App UNLOCK success (toast/banner)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● App unlocked (auto-lock in 5 min)                               [ Dismiss ]│
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Top bar lock button + status (когда приложение unlocked)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Z00Z Wallet   Wallet: Primary ▾    Pending: ●3   Sync: ● Synced   [ 🔒 Lock ]│
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) App lock settings (в app.settings → Security)

```text
┌──────────────────────────────── App Settings ▸ Security ─────────────────────┐
│ App lock                                                                     │
│  [x] Enable app lock                                                         │
│  Lock after:      [ 5 min ▾ ]                                                │
│  Lock on suspend: [x]                                                        │
│  Lock on focus loss (optional): [ ]                                          │
│                                                                              │
│ Unlock method:                                                               │
│  (•) App password                                                            │
│  ( ) OS keychain / biometric (if available)                                  │
│                                                                              │
│ Require app unlock for:                                                      │
│  [x] Open app                                                                │
│  [x] View balances (privacy)                                                 │
│  [x] Send / sign                                                             │
│  [x] Export / backup                                                         │
│  [x] Show seed phrase                                                        │
│                                                                              │
│                              [ Revert ]              [ Apply ]               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Wrong password / cooldown (в overlay)

```text
┌────────────────────────────── App Locked ────────────────────────────────────┐
│ Password: [ ********** ]  [ show ]                                           │
│ ⊗ Wrong password                                                             │
│ Attempts: 3 / 5                                                              │
│                                                                              │
│ [ Unlock ]   [ Exit ]                                                        │
└──────────────────────────────────────────────────────────────────────────────┘
┌────────────────────────────── App Locked ────────────────────────────────────┐
│ ⊗ Too many attempts                                                          │
│ Try again in: 00:30                                                          │
│                                                                              │
│ [ Exit ]                                                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) “Lock now” confirm modal (optional)

```text
┌────────────────────────────── Lock App ──────────────────────────────────────┐
│ Lock the application now?                                                    │
│                                                                              │
│ Effect: hides UI, blocks actions until app unlock.                            │
│                                                                              │
│ [ ] Also lock all wallets                                                    │
│                                                                              │
│                      [ Cancel ]                     [ Lock ]                │
└──────────────────────────────────────────────────────────────────────────────┘
```

Если хочешь, я сделаю **точный мокап для состояния “App unlocked, но wallet locked”** (часто путаница у юзеров) — там нужен явный двойной статус: *App: Unlocked* + *Wallet: Locked*.
