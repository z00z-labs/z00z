### 1) Entry point (App Settings → Logging)

```text
┌────────────────────────────── App Settings ▸ Logging ────────────────────────┐
│ Logging                                                                      │
│                                                                              │
│ Log level:      [ Info ▾ ]   ( ) Error  ( ) Warn  (•) Info  ( ) Debug  ( )Trace│
│                                                                              │
│ Destinations:                                                                │
│  [x] File logs                                                               │
│  [ ] Console (dev)                                                           │
│                                                                              │
│ Retention:     [ 7 days ▾ ]     Max size: [ 50 MB ▾ ]                        │
│                                                                              │
│ Actions:                                                                     │
│  [ View logs ]   [ Export logs… ]   [ Clear logs ]                           │
│                                                                              │
│                              [ Revert ]              [ Apply ]               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) View Logs (main screen: list + filter + tail)

```text
┌──────────────────────────────────── View Logs ───────────────────────────────┐
│ Source: [ app ▾ ]   Level: [ Info+ ▾ ]   Range: [ Today ▾ ]   [ ⟳ Refresh ]  │
│ Search: [ _____________________________________________ ]   [ Clear ]        │
│                                                                              │
│ Files: [ z00z_app_2025-12-20.log ▾ ]   [ Open folder ]                       │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ Time        │ Level │ Module            │ Message                         │ │
│ ├─────────────┼───────┼───────────────────┼─────────────────────────────────┤ │
│ │ 12:41:12.44 │ INFO  │ network::client   │ rpc ok: tx.pending=3            │ │
│ │ 12:41:10.03 │ INFO  │ tx::send          │ broadcast submitted tx=0x..B7   │ │
│ │ 12:41:09.88 │ WARN  │ storage::db       │ slow write: 120ms               │ │
│ │ 12:40:55.20 │ ERROR │ wallet::unlock    │ wrong password (attempt 2)      │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ Tail (selected row details)                                                  │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ 2025-12-20T12:41:10.03Z INFO tx::send                                     │ │
│ │ broadcast submitted tx=0x..B7 endpoint=auto                               │ │
│ │ request_id=... latency_ms=182                                             │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ [ Copy row ]  [ Copy details ]  [ Save selection… ]  [ Export logs… ]        │
│                                                                              │
│ Auto-follow: [x] Tail newest   Max lines: [ 5000 ▾ ]                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Log row “details” drawer (right panel alternative)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Log Details                                                      [ Close ]   │
├──────────────────────────────────────────────────────────────────────────────┤
│ Timestamp: 2025-12-20 12:41:10.03                                             │
│ Level:     INFO                                                               │
│ Module:    tx::send                                                           │
│ Thread:    main                                                               │
│                                                                              │
│ Message:                                                                     │
│  broadcast submitted tx=0x..B7 endpoint=auto                                  │
│                                                                              │
│ Context:                                                                     │
│  request_id: ...                                                              │
│  latency_ms: 182                                                              │
│                                                                              │
│ Actions:  [ Copy ]  [ Copy JSON ]  [ Pin ]                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Export Logs (share bundle)

```text
┌────────────────────────────── Export Logs ───────────────────────────────────┐
│ Export format:                                                               │
│  (•) Zip bundle (recommended)                                                │
│  ( ) Single file                                                             │
│                                                                              │
│ Include:                                                                     │
│  [x] App logs                                                                │
│  [x] Network logs                                                            │
│  [ ] Wallet logs (redacted)                                                  │
│  [ ] Diagnostics (system info)                                               │
│                                                                              │
│ Redaction:                                                                   │
│  [x] Redact addresses / keys / seeds                                         │
│  [x] Redact file paths (optional)                                            │
│                                                                              │
│ Destination: [ /home/user/Downloads/z00z_logs_2025-12-20.zip ] [ Browse… ]    │
│                                                                              │
│                      [ Cancel ]                     [ Export ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Clear Logs (danger confirm)

```text
┌────────────────────────────── Clear Logs ────────────────────────────────────┐
│ Delete local log files?                                                      │
│                                                                              │
│ Remove:  [x] App logs   [x] Network logs   [ ] Crash dumps                   │
│                                                                              │
│ [ ] I understand this cannot be undone                                       │
│                                                                              │
│                      [ Cancel ]                     [ Clear ]               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Errors / states

### 6a) Loading logs

```text
┌──────────────────────────────────── View Logs ───────────────────────────────┐
│ Loading…  ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                                 │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6b) Permission / file error

```text
┌────────────────────────────── View Logs ─ Error ─────────────────────────────┐
│ ⊗ Cannot read logs                                                          │
│ Reason: permission denied / file missing / path invalid                      │
│                                                                              │
│ [ Retry ]   [ Open logging settings ]                                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

Если хочешь, сделаю отдельный мокап **“Crash report”** (после падения приложения) — с кнопкой “Attach logs & diagnostics” и автосбором redacted bundle.
