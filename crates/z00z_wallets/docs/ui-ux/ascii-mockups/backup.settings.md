### 1) Backup Settings (egui: left menu + central panel)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]                      │
├──────────────────────────────────────────────────────────────────────────────┤
│ Settings ▸ Backups ▸ Backup Settings                                          │
│                                                                              │
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  BACKUPS              │  │  Backup Settings                               │ │
│ │ ───────────────────── │  │                                               │ │
│ │  ▸ Backup Settings    │  │  Default scope:       [ Selected wallet ▾ ]    │ │
│ │    Backup Manager     │  │  Default format:      [ Encrypted (.z00zbak)▾ ]│ │
│ │    Create Backup      │  │                                               │ │
│ │                       │  │  Default destination:                          │ │
│ │                       │  │   Path: [ /home/user/Backups/............. ]   │ │
│ │                       │  │         [ Browse… ]                             │ │
│ └───────────────────────┘  │                                               │ │
│                            │  Encryption                                    │ │
│                            │   [x] Encrypt metadata (names/labels)           │ │
│                            │   Password policy: [ Strong recommended ▾ ]     │ │
│                            │   [ ] Allow plaintext JSON exports              │ │
│                            │                                               │ │
│                            │  Auto-backup                                   │ │
│                            │   [ ] Enable scheduled backups                  │ │
│                            │   Frequency:   [ Weekly ▾ ]                     │ │
│                            │   Keep last:   [ 10 ▾ ] backups                 │ │
│                            │   When:        [ On app exit ▾ ]                │ │
│                            │                                               │ │
│                            │  Safety                                        │ │
│                            │   [x] Require wallet unlock for full backups    │ │
│                            │   [x] Verify integrity after backup             │ │
│                            │   [x] Warn if backup path is on removable media │ │
│                            │                                               │ │
│                            │  [ Apply ]  [ Revert ]  [ Restore defaults ]    │ │
│                            └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) Auto-backup configuration (expanded)

```text
┌────────────────────────────── Auto-backup ▾ ─────────────────────────────────┐
│  [x] Enable scheduled backups                                                │
│                                                                              │
│  Frequency:        [ Weekly ▾ ]       Day: [ Sun ▾ ]     Time: [ 02:00 ▾ ]    │
│  Keep last:        [ 10 ▾ ] backups                                         │
│                                                                              │
│  Trigger:                                                                    │
│   [x] On app exit                                                            │
│   [ ] On wallet lock                                                         │
│   [ ] On significant balance change                                          │
│                                                                              │
│  Destination:                                                                │
│   (•) Default path                                                           │
│   ( ) Ask every time                                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Security / plaintext warning (if enabled)

```text
┌────────────────────────────── Backup Warning ────────────────────────────────┐
│ ⚠ Allowing plaintext JSON exports is risky.                                  │
│ Anyone who gets the file can read wallet data.                               │
│                                                                              │
│ [ ] I understand and still want to enable plaintext exports                   │
│                                                                              │
│                      [ Cancel ]                    [ Enable ]               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Default destination chooser (path + validation)

```text
┌────────────────────────────── Default Backup Path ───────────────────────────┐
│ Path:  [ /home/user/Backups/ ]                                  [ Browse… ]  │
│                                                                              │
│ Status:  ● Writable   ○ Missing   ⊗ Permission denied                         │
│                                                                              │
│ Options:                                                                     │
│  [x] Create subfolder per wallet                                              │
│  Naming:  [ {wallet}_{date}.z00zbak ▾ ]                                       │
│                                                                              │
│                      [ Cancel ]                    [ Save ]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) States (loading / error)

```text
┌────────────────────────────── Backup Settings ───────────────────────────────┐
│ Loading backup settings…  ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                 │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
┌────────────────────────────── Backup Settings ─ Error ───────────────────────┐
│ ⊗ Failed to load/save backup settings                                         │
│ Reason: invalid path / permission denied / corrupted config                   │
│                                                                              │
│ [ Retry ]                           [ Restore defaults ]                      │
└──────────────────────────────────────────────────────────────────────────────┘
```