### 1) Transactions → Export entry point (toolbar)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Transactions                                                     [ ⟳ Refresh ]│
│  Tabs:  [ History ]  [ Pending ]                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│  History                                                                      │
│                                                                              │
│  Search: [ txid / address / memo...____________ ]   Range: [ 30d ▾ ]          │
│  Filter: [ All ▾ ]   Asset: [ Any ▾ ]    Status: [ Any ▾ ]                   │
│                                                                              │
│  Actions:   [ Export… ]   [ Import (dev) ]                                    │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ Time       │ Type │ Asset │ Amount      │ Status     │                   │  │
│  │ ...                                                                      │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `tx.export` — export wizard (egui-style)

### 2) Export Transactions (1/4) — scope & filters

```text
┌────────────────────────────── Export Transactions (1/4) ─────────────────────┐
│ Scope                                                                         │
│                                                                              │
│ Wallet:   [ Primary Wallet ▾ ]                                                │
│ Include:  [x] History   [ ] Pending                                           │
│                                                                              │
│ Date range:  [ Last 30 days ▾ ]    Custom: [ from ____ ] [ to ____ ]          │
│                                                                              │
│ Filters:                                                                      │
│  Asset:   [ Any ▾ ]        Type:   [ Any ▾ ]        Status: [ Any ▾ ]         │
│  Search:  [ (optional) txid/receiver/memo____________________ ]              │
│                                                                              │
│                          [ Cancel ]                 [ Next ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Export Transactions (2/4) — format & fields

```text
┌────────────────────────────── Export Transactions (2/4) ─────────────────────┐
│ Format                                                                        │
│                                                                              │
│  (•) CSV (spreadsheet-friendly)                                               │
│  ( ) JSON (structured)                                                        │
│  ( ) JSONL (one tx per line)                                                  │
│                                                                              │
│ CSV options:                                                                  │
│  Delimiter: [ Comma ▾ ]    [x] Include header row                             │
│  Timezone:  [ Local ▾ ]                                                       │
│                                                                              │
│ Fields to include:                                                            │
│  [x] Time     [x] Type     [x] Asset     [x] Amount                           │
│  [x] Fee      [x] Status   [x] TxID      [ ] Counterparty                     │
│  [ ] Memo     [ ] Notes    [ ] Raw data (dev)                                 │
│                                                                              │
│                         [ Back ]                  [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Export Transactions (3/4) — privacy / redaction

```text
┌────────────────────────────── Export Transactions (3/4) ─────────────────────┐
│ Privacy                                                                        │
│                                                                              │
│ Redaction:                                                                    │
│  [x] Redact addresses (show short form)                                       │
│  [x] Redact amounts (optional)                                                │
│  [ ] Exclude memos                                                            │
│                                                                              │
│ Include internal IDs:                                                         │
│  [ ] Include local draft ids (dev)                                            │
│                                                                              │
│ Output preview (sample row):                                                  │
│  2025-12-20 12:41:10, Send, Z00Z, -0.25000000, -0.00012, Pending, 0x..B7     │
│                                                                              │
│                         [ Back ]                  [ Next ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 5) Export Transactions (4/4) — destination & confirm

```text
┌────────────────────────────── Export Transactions (4/4) ─────────────────────┐
│ Destination                                                                   │
│                                                                              │
│  (•) Save to file                                                             │
│  ( ) Copy to clipboard (small exports)                                        │
│  ( ) Share… (system share)                                                    │
│                                                                              │
│ Path:  [ /home/user/Downloads/tx_export_2025-12-20.csv ]  [ Browse… ]         │
│                                                                              │
│ Estimated rows:  128                                                         │
│ Estimated size:  54 KB                                                       │
│                                                                              │
│                         [ Back ]               [ Export ]                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Progress + result

### 6) Exporting (progress)

```text
┌────────────────────────────── Exporting… ────────────────────────────────────┐
│ Exporting transactions…                                                      │
│                                                                              │
│ Progress:  ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                                │
│                                                                              │
│ [..] Collect rows                                                             │
│ [..] Apply filters                                                            │
│ [..] Write file                                                               │
│                                                                              │
│                               [ Cancel ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Success (share/open)

```text
┌────────────────────────────── Export Complete ───────────────────────────────┐
│ ● Export created                                                             │
│                                                                              │
│ File: tx_export_2025-12-20.csv                                  [ Open ]     │
│ Location: /home/user/Downloads/                                    [ Copy ]  │
│                                                                              │
│ Next:  [ Share… ]  [ Export again ]  [ Close ]                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Error / warnings

### 8) Large export warning

```text
┌────────────────────────────── Export Warning ────────────────────────────────┐
│ ⚠ Large export                                                               │
│ This export may include 120,000 rows and take time / produce a large file.   │
│                                                                              │
│ (•) Continue                                                                  │
│ ( ) Reduce date range                                                        │
│                                                                              │
│                      [ Back ]                    [ Continue ]                │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9) File write error

```text
┌────────────────────────────── Export Error ──────────────────────────────────┐
│ ⊗ Failed to export                                                           │
│ Reason: permission denied / disk full / invalid path                          │
│                                                                              │
│ [ Retry ]                          [ Choose different path ]                  │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```