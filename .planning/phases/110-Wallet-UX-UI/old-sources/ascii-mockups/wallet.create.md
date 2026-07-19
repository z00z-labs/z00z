### 1) Wallet Create wizard (egui: multi-step)

```text
┌────────────────────────────── Create Wallet (1/6) ───────────────────────────┐
│ Welcome                                                                       │
│                                                                              │
│ Wallet name:     [ My Wallet________________________ ]                       │
│ Wallet type:     (•) Standard   ( ) Watch-only   ( ) Hardware (if supported) │
│ Network:         [ Mainnet ▾ ]                                               │
│                                                                              │
│ Security level:  (•) Recommended   ( ) Advanced                               │
│                                                                              │
│                         [ Cancel ]                  [ Next ]                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Step 2 — Auth / unlock method

### 2) Unlock method + password

```text
┌────────────────────────────── Create Wallet (2/6) ───────────────────────────┐
│ Unlock / local protection                                                     │
│                                                                              │
│ Unlock method:                                                               │
│  (•) Password                                                                │
│  ( ) Password + OS keychain (if available)                                   │
│  ( ) No lock (dev only)                                                      │
│                                                                              │
│ Password:        [ *********************** ]   [ show ]                      │
│ Confirm:         [ *********************** ]                                  │
│ Password hint:   [ optional…____________________________ ]                    │
│                                                                              │
│ Auto-lock:                                                                    │
│  Lock after:  [ 5 min ▾ ]   ( ) Never (not recommended)                       │
│                                                                              │
│                         [ Back ]                   [ Next ]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Step 3 — Seed phrase generation

### 3) Seed options (12/24, language)

```text
┌────────────────────────────── Create Wallet (3/6) ───────────────────────────┐
│ Recovery seed                                                                 │
│                                                                              │
│ Seed length:   (•) 12 words   ( ) 24 words                                   │
│ Language:      [ English ▾ ]                                                 │
│                                                                              │
│ Seed phrase (write it down):                                                 │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ word1  word2  word3  word4  word5  word6                                  │ │
│ │ word7  word8  word9  word10 word11 word12                                 │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ [ Copy ]  [ Show as QR ]   (copy discouraged)                                │
│                                                                              │
│ ⚠ Store offline. Anyone with this seed can control your funds.               │
│                                                                              │
│                         [ Back ]                   [ Next ]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Step 4 — Seed confirmation quiz

### 4) Confirm seed (word checks)

```text
┌────────────────────────────── Create Wallet (4/6) ───────────────────────────┐
│ Confirm recovery seed                                                         │
│                                                                              │
│ Enter word #3:   [ __________ ]                                               │
│ Enter word #9:   [ __________ ]                                               │
│                                                                              │
│ ( ) I saved it in a password manager (not recommended)                        │
│                                                                              │
│ Status:  ○ Waiting / ● Correct / ⊗ Incorrect                                  │
│                                                                              │
│                         [ Back ]                   [ Next ]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Step 5 — Optional extras (backup reminder, labels)

### 5) Post-create options

```text
┌────────────────────────────── Create Wallet (5/6) ───────────────────────────┐
│ Optional setup                                                                │
│                                                                              │
│ [x] Enable backup reminder                                                    │
│ Remind me:  [ Weekly ▾ ]                                                     │
│                                                                              │
│ [ ] Create first encrypted backup now                                         │
│                                                                              │
│ Default asset view:  [ Portfolio ▾ ]                                          │
│                                                                              │
│                         [ Back ]                [ Create wallet ]            │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Step 6 — Progress + completion

### 6) Creating… (progress)

```text
┌────────────────────────────── Creating Wallet… ──────────────────────────────┐
│ Creating wallet…                                                             │
│                                                                              │
│ [..] Generate seed                                                           │
│ [..] Initialize local store                                                  │
│ [..] Create wallet identity                                                  │
│ [..] Apply settings                                                          │
│                                                                              │
│ Progress: ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯                                │
│                                                                              │
│                               [ Hide ]                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Done (open wallet / receive)

```text
┌────────────────────────────── Wallet Created ────────────────────────────────┐
│ ● Wallet created successfully                                                │
│                                                                              │
│ Name: My Wallet                                                              │
│ Status: ● Unlocked                                                           │
│                                                                              │
│ Next:  [ Open wallet ]   [ Receive ]   [ Create backup ]                     │
│                                                                              │
│                                [ Close ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Error / safety modals

### 8) Weak password warning

```text
┌────────────────────────────── Security Warning ──────────────────────────────┐
│ ⚠ Password appears weak                                                      │
│                                                                              │
│ Suggestions: longer passphrase, mix words, avoid reuse.                       │
│                                                                              │
│ (•) Go back and choose stronger password                                     │
│ ( ) Continue anyway                                                          │
│                                                                              │
│                      [ Back ]                    [ Continue ]                │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9) Seed copy warning (if user clicks Copy)

```text
┌────────────────────────────── Seed Copy Warning ─────────────────────────────┐
│ ⚠ Copying seed to clipboard is risky (other apps can read it).               │
│                                                                              │
│ (•) Don’t copy                                                               │
│ ( ) Copy anyway                                                              │
│                                                                              │
│                      [ Cancel ]                  [ Copy ]                    │
└──────────────────────────────────────────────────────────────────────────────┘
```