### 1) Entry point (Settings → Advanced → Developer Tools)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings ▸ Advanced                                                          │
├──────────────────────────────────────────────────────────────────────────────┤
│  Developer Tools                                                             │
│   [x] Developer mode                                                         │
│                                                                              │
│  Tools:  [ Key derive ]  [ Export public key ]  [ Logging ]                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `key.derive` — egui-style “HD Derivation” tool

### 2) Key Derivation panel (path input + wallet + output)

```text
┌──────────────────────────────────── Key Derive ──────────────────────────────┐
│ Wallet:   [ Primary Wallet ▾ ]         Network: [ Mainnet ▾ ]   [ ⟳ Refresh ] │
│                                                                              │
│ Derivation path (HD):                                                        │
│   [ m/44'/1234'/0'/0/0____________________________ ]   [ Validate ]          │
│                                                                              │
│ Path presets: [ BIP44 ▾ ]   [ Account + ]  [ Index + ]                        │
│                                                                              │
│ Key type:   (•) Address / pubkey     ( ) Full keypair (danger)               │
│ Derive:     (•) External (receive)   ( ) Internal (change)                   │
│                                                                              │
│ Options:                                                                      │
│  [x] Show as QR                                                               │
│  [x] Auto-copy on derive                                                      │
│  [ ] Include chain-code info (dev)                                            │
│                                                                              │
│ Output:                                                                       │
│  Address:   z00z1q...............................................  [ Copy ]  │
│  PubKey:    ed25519:............................................  [ Copy ]  │
│  Fingerprint:  9A:3C:..:F1                                          [ Copy ] │
│                                                                              │
│ Actions:   [ Derive ]   [ Clear ]   [ Save to notes ]                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) Presets dropdown (egui combo content)

```text
┌────────────────────────────── Path Presets ──────────────────────────────────┐
│ Presets:                                                                  ▾  │
│                                                                              │
│  • m/44'/1234'/0'/0/0    (Default receive)                                   │
│  • m/44'/1234'/0'/0/1    (Next receive)                                      │
│  • m/44'/1234'/0'/1/0    (Default change)                                    │
│  • m/44'/1234'/1'/0/0    (Account 1 receive)                                 │
│  • Custom                                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Batch derive (optional advanced panel)

```text
┌────────────────────────────── Key Derive ▸ Batch ▾ ──────────────────────────┐
│  ▾ Batch derive                                                              │
│                                                                              │
│  Range:  Start [ 0 ]   Count [ 20 ]      ( ) Receive   ( ) Change            │
│                                                                              │
│  Output format:  [ Table ▾ ]   [ Export CSV ]   [ Export JSON ]              │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ Index │ Path                 │ Address                         │ Copy   │  │
│  ├───────┼──────────────────────┼─────────────────────────────────┼────────┤  │
│  │ 0     │ m/.../0/0            │ z00z1q...                        │ [Copy] │  │
│  │ 1     │ m/.../0/1            │ z00z1q...                        │ [Copy] │  │
│  │ ...                                                                       │
│  └────────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## Safety / warnings

### 5) “Full keypair” warning (if selected)

```text
┌────────────────────────────── Danger: Export Keys ───────────────────────────┐
│ ⚠ Deriving/exporting full keypairs can expose private keys.                  │
│                                                                              │
│ (•) Derive address/public key only (recommended)                              │
│ ( ) Continue with full keypair                                                │
│                                                                              │
│ [ ] I understand the risks                                                    │
│                                                                              │
│                      [ Cancel ]                    [ Continue ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## States / errors

### 6) Wallet locked (blocked)

```text
┌──────────────────────────────────── Key Derive ──────────────────────────────┐
│ ○ Wallet is locked                                                           │
│                                                                              │
│ Unlock required to derive keys (depending on implementation).                │
│                                                                              │
│ [ Unlock wallet ]                                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 7) Invalid path format

```text
┌──────────────────────────────────── Key Derive ──────────────────────────────┐
│ Path: [ m/44'//0'/0/0 ]                                                      │
│ ⊗ Invalid derivation path                                                     │
│                                                                              │
│ Examples: m/44'/1234'/0'/0/0                                                 │
│                                                                              │
│ [ Fix ]                                                                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8) Deriving (loading)

```text
┌──────────────────────────────────── Key Derive ──────────────────────────────┐
│ Deriving…  ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                               │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9) Derive error

```text
┌──────────────────────────────────── Key Derive ─ Error ──────────────────────┐
│ ⊗ Failed to derive key                                                       │
│ Reason: unsupported curve / wallet not initialized / internal error           │
│                                                                              │
│ [ Retry ]     [ Clear ]                                                      │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```