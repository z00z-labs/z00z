Ок — ниже ASCII mockups **в стиле egui**: “центральная панель + левый сайдбар + верхняя панель”, много **checkbox / radio / combo / slider / buttons**, без кода.

------

## `app.settings` — egui layout (master)

### A) Settings Window (TopBar + Left Nav + Central Panel)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]                      │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  SETTINGS             │  │  General                                      │ │
│ │ ───────────────────── │  │                                               │ │
│ │  ▸ General            │  │  Language             [ English           ▾ ]  │ │
│ │    Appearance         │  │  Currency             [ USD               ▾ ]  │ │
│ │    Security           │  │  Time format          [ 24h               ▾ ]  │ │
│ │    Notifications      │  │                                               │ │
│ │    Network            │  │  [x] Confirm on external links                │ │
│ │    Advanced           │  │  [ ] Share telemetry                          │ │
│ │    About              │  │  [x] Crash reports                            │ │
│ │                       │  │                                               │ │
│ │                       │  │  Buttons:                                     │ │
│ │                       │  │   [ Apply ]  [ Revert ]  [ Restore defaults ] │ │
│ └───────────────────────┘  └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Appearance (Color theme)

### B) Appearance Panel (Theme + Accent + Density + Preview)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings ▸ Appearance                                                         │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  ▸ Appearance         │  │  Theme                                        │ │
│ │    General            │  │  Color theme:                                 │ │
│ │    Security           │  │   (•) Dark     ( ) Light     ( ) System       │ │
│ │    Notifications      │  │                                               │ │
│ │    Network            │  │  Color Scheme:        [ Emerald          ▾ ]   │ │
│ │    Advanced           │  │  Font size:           [ Medium           ▾ ]   │ │
│ │    About              │  │  
│ │                       │  │                                                │ │
│ │                       │  │                                               │ │
│ └───────────────────────┘  │                                               │ │
│                            │  
│                            │  
│                            │ 
│                            │   
│                            │  
│                            │   
│                            │                                               │ │
│                            │  [ Apply ]  [ Cancel ]                         │ │
│                            └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Security (Lock timeouts)

### C) Security Panel (Auto-lock + Clipboard + Session actions)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings ▸ Security                                                           │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  ▸ Security           │  │  Authentication                               │ │
│ │    General            │  │  [x] Require password on unlock               │ │
│ │    Appearance         │  │  [ ] Allow quick unlock (biometric/OS)        │ │
│ │    Notifications      │  │                                               │ │
│ │    Network            │  │  Auto-lock                                    │ │
│ │    Advanced           │  │  [x] Enable auto-lock                          │ │
│ │    About              │  │  Lock after inactivity:   [  5 min      ▾ ]   │ │
│ │                       │  │  [x] Lock on minimize                          │ │
│ │                       │  │  [x] Lock on sleep                             │ │
│ │                       │  │                                               │ │
│ └───────────────────────┘  │  Clipboard safety                              │ │
│                            │  [x] Warn when copying addresses               │ │
│                            │  Clear clipboard after:  [ 30 sec       ▾ ]    │ │
│                            │                                               │ │
│                            │  Session                                       │ │
│                            │   [ Lock now ]   [ Re-authenticate ]           │ │
│                            │                                               │ │
│                            │  [ Apply ]  [ Revert ]                         │ │
│                            └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Notifications

### D) Notifications Panel (Tx status + Receive + Quiet hours)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings ▸ Notifications                                                      │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  ▸ Notifications      │  │  Transaction updates                           │ │
│ │    General            │  │  [x] Broadcasted                                │ │
│ │    Appearance         │  │  [x] Confirmed                                  │ │
│ │    Security           │  │  [x] Failed                                     │ │
│ │    Network            │  │                                                 │ │
│ │    Advanced           │  │  Receive                                        │ │
│ │    About              │  │  [x] Incoming funds                             │ │
│ │                       │  │  [ ] New receiver card generated                │ │
│ └───────────────────────┘  │                                                 │ │
│                            │  Desktop notifications:   [x] Enabled           │ │
│                            │  Sounds:                 [ On              ▾ ]   │ │
│                            │  Quiet hours:            [ 23:00 ] - [ 07:00 ]  │ │
│                            │                                                 │ │
│                            │  [ Apply ]  [ Revert ]                          │ │
│                            └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Network

### E) Network Panel (Endpoint + Test + Proxy)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings ▸ Network                                                            │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  ▸ Network            │  │  Connection                                   │ │
│ │    General            │  │  Network preset:        [ Mainnet        ▾ ]   │ │
│ │    Appearance         │  │  RPC endpoint:          [ https://node...   ]  │ │
│ │    Security           │  │  Timeout:              [ 10 sec         ▾ ]    │ │
│ │    Notifications      │  │  Retries:              [ 3              ▾ ]    │ │
│ │    Advanced           │  │                                                 │ │
│ │    About              │  │  Proxy:                [ None           ▾ ]     │ │
│ └───────────────────────┘  │  [ Test connection ]    Status: ● OK            │ │
│                            │                                                 │ │
│                            │  [ Apply ]  [ Revert ]                          │ │
│                            └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Advanced

### F) Advanced Panel (Logs + Dev toggles + Diagnostics export)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings ▸ Advanced                                                           │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  ▸ Advanced           │  │  Developer                                    │ │
│ │    General            │  │  [ ] Developer mode                            │ │
│ │    Appearance         │  │  [ ] Show debug panels                         │ │
│ │    Security           │  │  Logging level:       [ Info            ▾ ]    │ │
│ │    Notifications      │  │                                                 │ │
│ │    Network            │  │  Diagnostics                                   │ │
│ │    About              │  │  [ Export diagnostics bundle ]                 │ │
│ └───────────────────────┘  │  [ Copy system info ]                           │ │
│                            │                                                 │ │
│                            │  Experimental                                  │ │
│                            │  [ ] New TX builder                             │ │
│                            │  [ ] Faster sync                                │ │
│                            │                                                 │ │
│                            │  [ Apply ]  [ Revert ]                          │ │
│                            └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## About

### G) About Panel (Version + Links + Licenses)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings ▸ About                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  ▸ About              │  │  Z00Z Wallet                                  │ │
│ │    General            │  │  Version:      v0.1.0                          │ │
│ │    Appearance         │  │  Build:        2025-12-20                      │ │
│ │    Security           │  │  Network:      Mainnet                         │ │
│ │    Notifications      │  │                                                 │ │
│ │    Network            │  │  [ View licenses ]  [ Open docs ]  [ Website ]  │ │
│ │    Advanced           │  │                                                 │ │
│ └───────────────────────┘  │  Copyright ©                                   │ │
│                            │                                                 │ │
│                            │  [ Copy version info ]                          │ │
│                            └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Bonus: Exit entry inside Settings (egui “Danger zone” block)

### H) General → Danger Zone (links to `app.exit`)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings ▸ General                                                            │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  ▸ General            │  │  General                                      │ │
│ │    Appearance         │  │  ...                                           │ │
│ │    Security           │  │                                               │ │
│ │    Notifications      │  │  ───────────────────────────────────────────  │ │
│ │    Network            │  │  Danger zone                                  │ │
│ │    Advanced           │  │  [ Exit application ]                          │ │
│ │    About              │  │                                               │ │
│ └───────────────────────┘  │                                               │ │
│                            │  [ Apply ]  [ Revert ]                         │ │
│                            └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

Если надо — сделаю next **ASCII mockups для `wallet.settings`** (пер-кошелёк настройки) в том же egui-стиле (обычно: security policy, derivation, watch-only, backup policy override).
