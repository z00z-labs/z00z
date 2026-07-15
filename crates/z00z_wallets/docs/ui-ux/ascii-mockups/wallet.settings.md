### 1) Wallet Settings (tabs + save/apply)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet: Primary Wallet ▸ Settings                              [ ← Back ]    │
├──────────────────────────────────────────────────────────────────────────────┤
│ Tabs:  [ General ]  [ Security ]  [ Privacy ]  [ Advanced ]  [ Danger Zone ] │
├──────────────────────────────────────────────────────────────────────────────┤
│ General                                                                      │
│                                                                              │
│ Wallet name:     [ Primary Wallet________________ ]                          │
│ Default asset:   [ Z00Z ▾ ]                                                  │
│ Default view:    [ Portfolio ▾ ]                                             │
│                                                                              │
│ Display:                                                                      │
│  Fiat currency:  [ USD ▾ ]                                                   │
│  Number format:  [ 8 decimals ▾ ]                                            │
│                                                                              │
│ Actions:   [ Revert ]                                           [ Apply ]    │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) Security tab (lock / session / confirmations)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet Settings ▸ Security                                                    │
├──────────────────────────────────────────────────────────────────────────────┤
│ Unlock protection:                                                           │
│  (•) Password                                                                │
│  ( ) OS keychain (if available)                                              │
│                                                                              │
│ Auto-lock:                                                                   │
│  Lock after: [ 5 min ▾ ]   ( ) Never (not recommended)                       │
│                                                                              │
│ Require confirm:                                                             │
│  [x] For send                                                                │
│  [x] For export / backup                                                     │
│  [ ] For viewing seed phrase                                                 │
│                                                                              │
│ Quick unlock (optional):                                                     │
│  [ ] Biometric / OS unlock (if available)                                    │
│                                                                              │
│                           [ Revert ]                          [ Apply ]      │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Privacy tab (receiver behavior / history)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet Settings ▸ Privacy                                                     │
├──────────────────────────────────────────────────────────────────────────────┤
│ Receiving:                                                                   │
│  [x] Use one-time receiver cards (recommended)                                │
│  Receiver labeling: [ Enabled ▾ ]                                            │
│                                                                              │
│ Sending:                                                                     │
│  [x] Randomize input ordering                                                │
│  [ ] Add dummy outputs (decoys)                                              │
│                                                                              │
│ Local data:                                                                  │
│  [x] Cache tx history                                                        │
│  [ ] Hide amounts in UI (privacy mode)                                       │
│                                                                              │
│                           [ Revert ]                          [ Apply ]      │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Advanced tab (derivation / rescan / debug)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet Settings ▸ Advanced                                                    │
├──────────────────────────────────────────────────────────────────────────────┤
│ Derivation preset:  [ Default ▾ ]                                            │
│ HD base path:       [ m/44'/1234'/0'________________ ]   [ Validate ]        │
│ Discovery depth:    [ 100 ▾ ]                                                │
│                                                                              │
│ Maintenance:                                                                 │
│  [ Rescan blockchain… ]   [ Rebuild indexes ]   [ Clear cache ]              │
│                                                                              │
│ Developer:                                                                   │
│  [ ] Enable verbose logs for this wallet                                     │
│  [ Export diagnostics bundle… ]                                              │
│                                                                              │
│                           [ Revert ]                          [ Apply ]      │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Danger Zone (delete/remove/export shortcuts)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Wallet Settings ▸ Danger Zone                                                 │
├──────────────────────────────────────────────────────────────────────────────┤
│ Backup:                                                                      │
│  [ Create encrypted backup… ]   [ Export wallet… ]                            │
│                                                                              │
│ Removal:                                                                     │
│  [ Remove from app… ]   (detach)                                             │
│  [ Delete wallet… ]     (danger)                                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) “Get / Update settings” mechanics (egui patterns)

### 6a) Unsaved changes bar

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ○ Unsaved changes                                         [ Revert ] [ Apply ]│
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6b) Applying settings (progress)

```text
┌────────────────────────────── Applying… ─────────────────────────────────────┐
│ Saving wallet settings…  ────────────────▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯▯                 │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6c) Save error

```text
┌──────────────────────────── Wallet Settings ─ Error ─────────────────────────┐
│ ⊗ Failed to apply settings                                                   │
│ Reason: invalid derivation path / storage locked                              │
│                                                                              │
│ [ Retry ]                          [ Revert changes ]                        │
└──────────────────────────────────────────────────────────────────────────────┘
```