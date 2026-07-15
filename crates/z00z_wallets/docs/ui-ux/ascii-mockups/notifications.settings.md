### 1) Notifications Settings (egui: toggles + channels + per-event)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings ▸ Notifications                                      [ ⟳ Refresh ]  │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  SETTINGS             │  │  Notifications                                │ │
│ │ ───────────────────── │  │                                               │ │
│ │    General            │  │  Enable notifications:   [x]                    │ │
│ │    Appearance         │  │                                               │ │
│ │    Security           │  │  Channels                                      │ │
│ │    Network            │  │   [x] In-app banner                             │ │
│ │  ▸ Notifications      │  │   [ ] Desktop notifications                     │ │
│ │    Backups            │  │   [ ] Sound                                     │ │
│ │    Advanced           │  │                                               │ │
│ └───────────────────────┘  │  Quiet hours                                   │ │
│                            │   [ ] Enable quiet hours                        │ │
│                            │   From: [ 22:00 ▾ ]  To: [ 08:00 ▾ ]            │ │
│                            │                                               │ │
│                            │  Events                                        │ │
│                            │   [x] Incoming funds                            │ │
│                            │   [x] Confirmations / status changes            │ │
│                            │   [ ] Failed transactions                       │ │
│                            │   [ ] Price alerts                              │ │
│                            │   [ ] Security alerts (lock/unlock)             │ │
│                            │                                               │ │
│                            │  Frequency / noise control                      │ │
│                            │   Aggregation: [ Group similar ▾ ]              │ │
│                            │   Max per minute: [ 3 ▾ ]                       │ │
│                            │                                               │ │
│                            │  [ Apply ]  [ Revert ]                          │ │
│                            └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) Event-level configuration (expandable rows)

### 2a) Incoming funds event (advanced)

```text
┌──────────────────────────── Incoming funds ▾ ────────────────────────────────┐
│  ▾ Incoming funds                                                            │
│   Enabled: [x]                                                               │
│                                                                              │
│   Trigger:  (•) Any amount   ( ) Above threshold: [ 0.10 ]  Asset: [ Any ▾ ] │
│   Show:     [x] Amount   [ ] Sender hint (if available)                       │
│   Sound:    [ ] Play sound                                                   │
│                                                                              │
│   In-app banner:     [x]                                                     │
│   Desktop notify:    [ ]                                                     │
│                                                                              │
│   [ Test notification ]                                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 2b) Confirmations / status changes (pending → confirmed)

```text
┌──────────────────────── Confirmations / status ▾ ────────────────────────────┐
│  ▾ Confirmations / status changes                                             │
│   Enabled: [x]                                                                │
│                                                                              │
│   Notify when:                                                                │
│    [x] Pending → Confirmed                                                    │
│    [ ] Pending → Failed                                                       │
│    [ ] Confirmed → Reorged (dev)                                              │
│                                                                              │
│   Minimum confirmations: [ 1 ▾ ]                                             │
│   Include txid:           [x]                                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Channels sub-panel (desktop permissions / test)

```text
┌────────────────────────────── Channels ▾ ────────────────────────────────────┐
│  ▾ Channels                                                                  │
│   In-app banners:        [x]                                                 │
│   Desktop notifications: [ ]   (requires OS permission)                       │
│   Sound:                 [ ]                                                 │
│                                                                              │
│   Desktop permission:  ○ Not granted   [ Request permission ]                 │
│                                                                              │
│   [ Send test notification ]                                                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Quiet hours (schedule UI)

```text
┌────────────────────────────── Quiet hours ▾ ─────────────────────────────────┐
│  ▾ Quiet hours                                                               │
│   [x] Enable quiet hours                                                     │
│                                                                              │
│   Days:  [x] Mon  [x] Tue  [x] Wed  [x] Thu  [x] Fri  [ ] Sat  [ ] Sun        │
│                                                                              │
│   From:  [ 22:00 ▾ ]        To:  [ 08:00 ▾ ]                                 │
│                                                                              │
│   During quiet hours:                                                        │
│    (•) Suppress all notifications                                             │
│    ( ) Show in-app only                                                       │
│    ( ) Only security alerts                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Noise control / rate limiting

```text
┌────────────────────────── Noise control ▾ ───────────────────────────────────┐
│  ▾ Noise control                                                             │
│   Aggregation:      [ Group similar ▾ ]   ( ) None  ( ) Group  ( ) Digest     │
│   Max per minute:   [ 3 ▾ ]                                                  │
│   Cooldown:         [ 30s ▾ ]                                                │
│                                                                              │
│   Price alerts:     [ ] Enable                                               │
│   Price threshold:  [ 5% ▾ ]  Window: [ 24h ▾ ]                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Notification preview banner (in-app)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● Incoming funds: +0.25000000 Z00Z   Status: Pending   [ View tx ] [ Dismiss ]│
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 7) States / errors

### 7a) Applying settings (loading)

```text
┌────────────────────────── Notifications ─────────────────────────────────────┐
│ Applying notification settings…  ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯            │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7b) Permission denied / OS blocked

```text
┌────────────────────── Notifications ─ Permission Error ──────────────────────┐
│ ⊗ Desktop notifications are blocked by the OS/browser                         │
│                                                                              │
│ Fix: enable notifications in system settings.                                 │
│                                                                              │
│ [ Open help ]   [ Retry permission ]                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7c) Save error

```text
┌──────────────────────── Notifications ─ Error ───────────────────────────────┐
│ ⊗ Failed to save notification settings                                       │
│ Reason: config write failed / invalid schedule                                │
│                                                                              │
│ [ Retry ]                           [ Restore defaults ]                      │
└──────────────────────────────────────────────────────────────────────────────┘
```