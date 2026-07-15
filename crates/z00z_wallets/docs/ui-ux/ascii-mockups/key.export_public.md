### 1) Entry point (Settings → Advanced → Developer Tools)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings ▸ Advanced                                                          │
├──────────────────────────────────────────────────────────────────────────────┤
│ Developer Tools                                                              │
│  Tools:  [ Key derive ]  [ Export public key ]                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## `key.export_public` — egui-style panel (copy/share)

### 2) Export Public Key (main)

```text
┌────────────────────────────── Export Public Key ─────────────────────────────┐
│ Wallet:   [ Primary Wallet ▾ ]         Network: [ Mainnet ▾ ]   [ ⟳ Refresh ] │
│                                                                              │
│ Key kind:   [ Account public key ▾ ]                                          │
│                                                                              │
│ Export format:                                                               │
│   (•) Bech32 address-like                                                     │
│   ( ) Hex                                                                     │
│   ( ) Base64                                                                  │
│   ( ) JSON (with metadata)                                                    │
│                                                                              │
│ Include:                                                                      │
│  [x] Network                                                                  │
│  [x] Fingerprint                                                              │
│  [ ] Derivation path                                                          │
│  [ ] Creation timestamp                                                       │
│                                                                              │
│ Output:                                                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │ z00zpub1q.............................................................  │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│ Fingerprint:  9A:3C:..:F1                                         [ Copy ]   │
│                                                                              │
│ Actions:  [ Copy ]  [ Share… ]  [ Show QR ]  [ Save to file… ]               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 3) “Key kind” dropdown (examples)

```text
┌────────────────────────────── Key kind ──────────────────────────────────────┐
│ Key kind:                                                                 ▾  │
│                                                                              │
│  • Account public key                                                        │
│  • Wallet identity key                                                       │
│  • Receive (external) pubkey @ path                                           │
│  • Change (internal) pubkey @ path                                            │
│  • Custom path…                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 4) Optional: choose derivation path (inline)

```text
┌──────────────────────────── Export Public Key ▸ Path ────────────────────────┐
│ Path source:  (•) Default account   ( ) Custom HD path                        │
│                                                                              │
│ HD path: [ m/44'/1234'/0'/0/0____________________________ ]   [ Validate ]   │
│                                                                              │
│ [ Derive & export ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## QR / Share overlays

### 5) Show QR (modal)

```text
┌────────────────────────────── Public Key (QR) ───────────────────────────────┐
│ Wallet: Primary Wallet        Format: Bech32                                 │
│                                                                              │
│     ┌─────────────────────────────────────────────────────────────────┐      │
│     │                                                                 │      │
│     │                       [   QR CODE LARGE   ]                     │      │
│     │                                                                 │      │
│     └─────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│ z00zpub1q.................................................................    │
│                                                                              │
│ [ Copy ]   [ Share… ]   [ Close ]                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6) Share menu (kebab / popup)

```text
┌────────────────────────────── Share… ────────────────────────────────────────┐
│ [ Copy to clipboard ]                                                        │
│ [ Save to file… ]                                                            │
│ [ Show QR ]                                                                  │
│ [ Copy fingerprint ]                                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## States / errors

### 7) Wallet locked (blocked if needed)

```text
┌────────────────────────────── Export Public Key ─────────────────────────────┐
│ ○ Wallet is locked                                                           │
│                                                                              │
│ Unlock may be required to access key material (implementation-dependent).     │
│                                                                              │
│ [ Unlock wallet ]                                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8) Loading / re-fetching

```text
┌────────────────────────────── Export Public Key ─────────────────────────────┐
│ Preparing export…   ────────────────▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯                       │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 9) Export error

```text
┌────────────────────────────── Export Public Key ─ Error ─────────────────────┐
│ ⊗ Failed to export public key                                                │
│ Reason: invalid path / unsupported format / internal error                    │
│                                                                              │
│ [ Retry ]   [ Reset ]                                                        │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

### 10) “Copied” toast (small overlay)

```text
┌───────────────────────────────┐
│ ● Copied to clipboard          │
└───────────────────────────────┘
```