### 1) Logging Settings (egui: settings page + live status)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings ▸ Advanced ▸ Logging                                   [ ⟳ Refresh ] │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  SETTINGS             │  │  Logging                                       │ │
│ │ ───────────────────── │  │                                               │ │
│ │    General            │  │  Logging enabled:   [x]                         │ │
│ │    Appearance         │  │  Mode:              [ Normal ▾ ]                │ │
│ │    Security           │  │                                               │ │
│ │    Network            │  │  Level:             [ INFO ▾ ]                  │ │
│ │    Notifications      │  │  Targets:           [ Console ▾ ]               │ │
│ │  ▸ Advanced           │  │                                               │ │
│ │     ▸ Logging         │  │  Log file path:      /home/user/.z00z/logs/     │ │
│ │     ▸ Developer Tools │  │                     [ Change… ] [ Open folder ] │ │
│ └───────────────────────┘  │                                               │ │
│                            │  Rotation:                                     │ │
│                            │   Max file size:     [ 10 MB ▾ ]                │ │
│                            │   Keep last files:   [ 10 ▾ ]                   │ │
│                            │                                               │ │
│                            │  Privacy:                                      │ │
│                            │   [x] Redact addresses                          │ │
│                            │   [x] Redact amounts                            │ │
│                            │   [ ] Include stack traces                      │ │
│                            │                                               │ │
│                            │  Actions:                                      │ │
│                            │   [ View logs ]   [ Export logs… ]              │ │
│                            │   [ Clear logs ]  [ Test log ]                 │ │
│                            │                                               │ │
│                            │  [ Apply ]  [ Revert ]                          │ │
│                            └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) Mode / level presets (egui combo dropdown feel)

### 2a) Mode dropdown

```text
┌────────────────────────────── Logging Mode ──────────────────────────────────┐
│ Mode:                                                                     ▾  │
│                                                                              │
│  • Normal    (recommended)                                                   │
│  • Verbose   (more details)                                                  │
│  • Debug     (dev only)                                                      │
│  • Trace     (very noisy)                                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 2b) Level dropdown

```text
┌────────────────────────────── Log Level ─────────────────────────────────────┐
│ Level:                                                                    ▾  │
│                                                                              │
│  • ERROR                                                                     │
│  • WARN                                                                      │
│  • INFO                                                                      │
│  • DEBUG                                                                     │
│  • TRACE                                                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Targets selection (multi-target)

```text
┌────────────────────────────── Logging Targets ───────────────────────────────┐
│ Targets:                                                                     │
│  [x] File                                                                     │
│  [x] Console                                                                  │
│  [ ] RPC stream (dev)                                                        │
│                                                                              │
│ File format:   [ JSON ▾ ]    [ ] Pretty print                                │
│ Console format:[ Compact ▾ ]                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) View logs (inline viewer panel)

```text
┌──────────────────────────────────── View Logs ───────────────────────────────┐
│ Filter: [ INFO ▾ ]   Search: [ txid / error / module...________ ]  [ Clear ] │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ 12:34:01  INFO  wallet: unlocked                                          │ │
│ │ 12:34:05  INFO  tx: build ok  id=0x..B7                                   │ │
│ │ 12:34:06  WARN  net: retrying node=...                                    │ │
│ │ 12:34:10  ERROR rpc: timeout method=asset.balance                         │ │
│ │ ...                                                                        │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ [ Copy selected ]   [ Export selection… ]   [ Close ]                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Export logs wizard (quick)

```text
┌──────────────────────────────────── Export Logs ─────────────────────────────┐
│ Range:      (•) Last 24h   ( ) Last 7d   ( ) Custom: [from] [to]             │
│ Level:      [ INFO+ ▾ ]                                                      │
│ Redaction:  [x] Redact addresses   [x] Redact amounts                        │
│ Format:     (•) Zip bundle   ( ) Text file   ( ) JSON lines                  │
│                                                                              │
│ Destination: [ /home/user/Downloads/z00z_logs_2025-12-20.zip ] [ Browse… ]    │
│                                                                              │
│                         [ Cancel ]                 [ Export ]                │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Dangerous actions confirmation

### 6a) Clear logs confirm

```text
┌──────────────────────────────────── Clear Logs ──────────────────────────────┐
│ Delete local log files?                                                      │
│                                                                              │
│ This removes logs from this device only.                                     │
│                                                                              │
│ [ ] I understand this cannot be undone                                       │
│                                                                              │
│                      [ Cancel ]                     [ Clear logs ]          │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6b) Privacy warning (if redaction disabled)

```text
┌────────────────────────────── Logging Privacy Warning ───────────────────────┐
│ ⚠ Redaction is disabled                                                      │
│ Logs may contain addresses, amounts, and metadata.                            │
│                                                                              │
│ (•) Keep redaction enabled                                                    │
│ ( ) Disable anyway                                                           │
│                                                                              │
│ [ ] I understand the risks                                                    │
│                                                                              │
│                      [ Back ]                      [ Continue ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 7) States / errors

### 7a) Applying settings (loading)

```text
┌──────────────────────────────── Logging ─────────────────────────────────────┐
│ Applying logging settings…  ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯                  │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7b) Error writing logs / permissions

```text
┌────────────────────────────── Logging ─ Error ───────────────────────────────┐
│ ⊗ Logging failed                                                             │
│ Reason: cannot write to log path / permission denied                          │
│                                                                              │
│ [ Choose different path ]   [ Retry ]                                         │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```