### 1) Network Settings (egui: selector + endpoints + status)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settings ▸ Network                                             [ ⟳ Refresh ] │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────┐  ┌───────────────────────────────────────────────┐ │
│ │  SETTINGS             │  │  Network                                      │ │
│ │ ───────────────────── │  │                                               │ │
│ │    General            │  │  Active network:   [ Mainnet ▾ ]               │ │
│ │    Appearance         │  │  Connection:       ● Connected                 │ │
│ │    Security           │  │  Latency:          42 ms                       │ │
│ │  ▸ Network            │  │  Sync status:      ● Synced                    │ │
│ │    Notifications      │  │                                               │ │
│ │    Backups            │  │  Node / RPC endpoint:                          │ │
│ │    Advanced           │  │   Mode:   (•) Auto (recommended)               │ │
│ │                       │  │          ( ) Custom list                       │ │
│ └───────────────────────┘  │                                               │ │
│                            │  Endpoints                                     │ │
│                            │   ┌─────────────────────────────────────────┐ │ │
│                            │   │ ● https://rpc1.example.org   42 ms  [✓] │ │ │
│                            │   │ ○ https://rpc2.example.org   —     [ ]  │ │ │
│                            │   │ ○ https://rpc3.example.org   120ms [ ]  │ │ │
│                            │   └─────────────────────────────────────────┘ │ │
│                            │                                               │ │
│                            │  Actions:  [ Test ] [ Add ] [ Edit ] [ Remove ]│ │
│                            │                                               │ │
│                            │  Advanced                                      │ │
│                            │   [ ] Use proxy                                │ │
│                            │   Timeout: [ 10s ▾ ]   Retries: [ 3 ▾ ]         │ │
│                            │                                               │ │
│                            │  [ Apply ]  [ Revert ]                          │ │
│                            └───────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) Network selector dropdown (Mainnet/Testnet/Local)

```text
┌────────────────────────────── Active network ────────────────────────────────┐
│ Active network:                                                           ▾  │
│                                                                              │
│  • Mainnet       (recommended)                                               │
│  • Testnet                                                                │
│  • Devnet                                                                 │
│  • Local node                                                             │
│                                                                              │
│  • Custom profile…                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 3) Endpoint editor (Add/Edit)

```text
┌────────────────────────────── Add RPC Endpoint ──────────────────────────────┐
│ Network: [ Mainnet ▾ ]                                                       │
│                                                                              │
│ Name:        [ MyRPC-1________________ ]                                      │
│ URL:         [ https://rpc.myhost.tld____________________________ ]           │
│                                                                              │
│ Auth:        (•) None   ( ) API key   ( ) Basic                              │
│ API key:     [ ******************** ]                                        │
│                                                                              │
│ TLS:         [x] Verify certificates                                          │
│                                                                              │
│ Health check:  [ Test now ]   Result: ○ Pending / ● OK / ⊗ Fail               │
│                                                                              │
│                      [ Cancel ]                    [ Save ]                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Endpoint test results (table)

```text
┌────────────────────────────── Test Endpoints ────────────────────────────────┐
│ Testing network: Mainnet                                                     │
│                                                                              │
│ ┌──────────────────────────────────────────────────────────────────────────┐ │
│ │ Endpoint                     │ Latency │ Status  │ Notes                 │ │
│ ├──────────────────────────────┼─────────┼─────────┼───────────────────────┤ │
│ │ https://rpc1.example.org      │ 42 ms   │ ● OK    │ synced               │ │
│ │ https://rpc2.example.org      │ —       │ ⊗ FAIL  │ timeout              │ │
│ │ https://rpc3.example.org      │ 120 ms  │ ● OK    │ behind 2 blocks       │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│ [ Use best ]   [ Keep current ]   [ Close ]                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Proxy settings (advanced)

```text
┌────────────────────────────── Network ▸ Proxy ▾ ─────────────────────────────┐
│  ▾ Proxy                                                                     │
│  [ ] Use proxy                                                               │
│  Type:  [ SOCKS5 ▾ ]                                                         │
│  Host:  [ 127.0.0.1 ]     Port: [ 9050 ]                                     │
│  Auth:  [ ] Username/password                                                │
│        User: [ ____ ]  Pass: [ ******** ]                                     │
│                                                                              │
│  [ Test proxy ]   Result: ○ Pending / ● OK / ⊗ Fail                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) Common states / errors

### 6a) Offline / disconnected banner

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ ○ Offline — showing cached data.   [ Retry ]   [ Open Network settings ]     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6b) Network mismatch warning (wallet expects different network)

```text
┌────────────────────────────── Network Warning ───────────────────────────────┐
│ ⚠ Wallet data appears to be for Testnet, but active network is Mainnet.      │
│                                                                              │
│ (•) Switch to Testnet                                                        │
│ ( ) Stay on Mainnet (balances may be incorrect)                               │
│                                                                              │
│                      [ Cancel ]                    [ Continue ]              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6c) Apply settings (loading)

```text
┌──────────────────────────────── Network ─────────────────────────────────────┐
│ Applying network settings…  ────────────────▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯                  │
│                                                                              │
│ [ Cancel ]                                                                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 6d) Error saving settings

```text
┌────────────────────────────── Network ─ Error ───────────────────────────────┐
│ ⊗ Failed to save network settings                                            │
│ Reason: invalid URL / permission denied / config corrupted                    │
│                                                                              │
│ [ Retry ]                           [ Restore defaults ]                      │
└──────────────────────────────────────────────────────────────────────────────┘
```