### 1) Backups Manager (egui: list + actions + details pane)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Z00Z Wallet                                                     ● Connected  │
│  [Assets]  [Transactions]  [Send]  [Receive]  [Settings]                      │
├──────────────────────────────────────────────────────────────────────────────┤
│ Settings ▸ Backups                                          [ Create backup ]│
│                                                                              │
│ Search: [  filename / wallet name / tag...            ]   Sort: [ Date ▾ ]   │
│ Filters:  [x] Encrypted   [ ] Include watch-only   [ ] Show missing files     │
│                                                                              │
│ ┌───────────────────────────────────────────┐  ┌───────────────────────────┐ │
│ │ Backups list                              │  │ Backup details            │ │
│ │ ────────────────────────────────────────  │  │ File: Primary_2025-12-20… │ │
│ │ ▸ Primary_2025-12-20.z00zbak   ● OK       │  │ Path: /Downloads/...      │ │
│ │   Primary_2025-12-01.z00zbak   ● OK       │  │ Created: 2025-12-20 12:34 │ │
│ │   WatchOnly_2025-11-10.z00zbak  ○ Missing │  │ Size: 1.2 MB              │ │
│ │   Settings_2025-10-02.json      ⚠ Plain   │  │ Format: Encrypted (.z00z) │ │
│ │                                           │  │ Scope: Primary Wallet     │ │
│ │                                           │  │ Includes: keys, commits…  │ │
│ └───────────────────────────────────────────┘  │ Tags: [ weekly ] [ offline ]│ │
│                                                │ Notes: [ _____________ ]    │ │
│                                                │                             │ │
│                                                │ Actions:                    │ │
│                                                │  [ Restore ] [ Verify ]     │ │
│                                                │  [ Reveal in folder ]       │ │
│                                                │  [ Copy path ]              │ │
│                                                │  [ Rename ]  [ Retag ]      │ │
│                                                │  [ Delete entry ]           │ │
│                                                │                             │ │
│                                                │ [ Save notes ]              │ │
│                                                └───────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) Backup row context menu (right-click / kebab)

```text
┌────────────────────────────── Backup: Primary_2025-12-20 ────────────────────┐
│ [ Restore ]                                                                  │
│ [ Verify integrity ]                                                         │
│ [ Reveal in folder ]                                                         │
│ [ Copy path ]                                                                │
│ ───────────────────────────────────────────────────────────────────────────  │
│ [ Rename ]                                                                   │
│ [ Edit tags ]                                                                │
│ [ Delete entry ]                                                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Verify backup (quick dialog)

```text
┌──────────────────────────────── Verify Backup ───────────────────────────────┐
│ File:  Primary_2025-12-20.z00zbak                                            │
│                                                                              │
│ Password:  [ *********************** ]   [ show ]                            │
│                                                                              │
│ Checks:                                                                       │
│  [x] Can decrypt                                                              │
│  [x] Format valid                                                             │
│  [x] Wallet data present                                                      │
│  [ ] Keys included (depends on backup scope)                                  │
│                                                                              │
│                           [ Cancel ]    [ Verify ]                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Delete backup entry (danger confirm)

```text
┌──────────────────────────────── Delete Backup ───────────────────────────────┐
│ Delete backup entry?                                                         │
│                                                                              │
│ Primary_2025-12-20.z00zbak                                                   │
│                                                                              │
│ What to delete:                                                              │
│  (•) Remove from list only (keeps file on disk)                              │
│  ( ) Delete file from disk (cannot be undone)                                │
│                                                                              │
│ [ ] I understand this action may break recovery plans.                       │
│                                                                              │
│                      [ Cancel ]                     [ Delete ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Missing file state (re-link)

```text
┌────────────────────────────── Backup Missing ────────────────────────────────┐
│ ○ Backup file not found at saved path                                         │
│                                                                              │
│ File: WatchOnly_2025-11-10.z00zbak                                           │
│ Saved path: /mnt/usb/watchonly/...                                            │
│                                                                              │
│ Actions:                                                                      │
│  [ Locate file… ]   [ Remove entry ]                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Empty state (no backups)

```text
┌──────────────────────────────── Backups ─────────────────────────────────────┐
│ No backups found.                                                            │
│                                                                              │
│ [ Create backup ]                                                            │
│                                                                              │
│ Tip: Store encrypted backups offline (USB / cold storage).                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 7) Loading / error states

### 7a) Loading list

```text
┌──────────────────────────────── Backups ─────────────────────────────────────┐
│ Loading backups…  ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                         │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7b) Error reading backups index

```text
┌──────────────────────────────── Backups ─ Error ─────────────────────────────┐
│ ⊗ Failed to load backup list                                                 │
│ Reason: corrupted index / permission denied                                  │
│                                                                              │
│ [ Retry ]                         [ Rebuild index ]                          │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```