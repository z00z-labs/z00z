Ниже — ASCII мокапы для **App Tor** (настройки + статус + тест + “circuit” view). Всё под egui, без кода.

------

## 1) App Settings → Network → Tor (основной экран)

```text
┌────────────────────────────── App Settings ▸ Network ────────────────────────┐
│ Tabs: [ General ] [ Network ] [ Security ] [ Logging ]                       │
├──────────────────────────────────────────────────────────────────────────────┤
│ Network ▸ Tor                                                                 │
│                                                                              │
│ Tor mode:        [ Off ▾ ]   ( ) Off   ( ) SOCKS5 proxy   ( ) Embedded Tor   │
│                                                                              │
│ Proxy (SOCKS5):                                                             │
│  Host: [ 127.0.0.1____________ ]   Port: [ 9050 ]                            │
│  Auth: (•) None   ( ) Username/Password  ( ) Cookie file                     │
│                                                                              │
│ DNS:                                                                          │
│  (•) Remote DNS via proxy (recommended)                                      │
│  ( ) System DNS (leaks possible)                                             │
│                                                                              │
│ Apply to:                                                                     │
│  [x] RPC / network calls                                                     │
│  [x] Explorer / external links                                               │
│  [ ] Updates (optional)                                                      │
│                                                                              │
│ Safety:                                                                       │
│  [x] Block clearnet fallback (recommended)                                   │
│                                                                              │
│ Actions:  [ Test connection ]  [ View status ]                               │
│                                                                              │
│                              [ Revert ]              [ Apply ]               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 2) Quick status in top bar (indicator)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Z00Z Wallet   Network: Devnet ▾   Tor: ● ON (SOCKS5)   RPC: ● Connected      │
└──────────────────────────────────────────────────────────────────────────────┘
```

Variants:

```text
Tor: ○ OFF
Tor: ⚠ DEGRADED (proxy unreachable)
Tor: ● ON (Embedded)
```

------

## 3) Test connection modal (basic + leak checks)

```text
┌────────────────────────────── Tor Test Connection ───────────────────────────┐
│ Mode: SOCKS5 proxy     Host: 127.0.0.1:9050                                  │
│                                                                              │
│ Tests:                                                                       │
│  [..] Proxy reachable                                                        │
│  [..] Remote DNS via proxy                                                   │
│  [..] Fetch lightweight endpoint (tor-check)                                 │
│  [..] RPC handshake through Tor                                              │
│                                                                              │
│ Result:  ○ Running…                                                          │
│                                                                              │
│ Details: [ Show ▸ ]                                                          │
│                                                                              │
│                               [ Close ]                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

Success:

```text
┌────────────────────────────── Tor Test Connection ───────────────────────────┐
│ ● Passed                                                                     │
│ Proxy: OK     DNS via proxy: OK     RPC via Tor: OK                          │
│                                                                              │
│ Exit IP: (redacted)      Country: (optional)                                 │
│                                                                              │
│ [ View status ]   [ Close ]                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

Failure:

```text
┌────────────────────────────── Tor Test Connection ───────────────────────────┐
│ ⊗ Failed                                                                     │
│ Reason: proxy unreachable / auth failed / DNS leak risk                       │
│                                                                              │
│ [ Fix settings ]   [ Retry ]   [ Disable Tor ]                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 4) Tor status screen (runtime health)

```text
┌───────────────────────────────── Tor Status ─────────────────────────────────┐
│ Tor mode: ● ON (SOCKS5)                                                     │
│ Proxy: 127.0.0.1:9050      Remote DNS: ● Enabled     Clearnet fallback: OFF │
│                                                                              │
│ Circuit: ● Established     Age: 00:12     Streams: 3                         │
│                                                                              │
│ Traffic (approx):  Up: 12 KB/s   Down: 48 KB/s                               │
│                                                                              │
│ Endpoints using Tor:                                                        │
│  • RPC: https://rpc.devnet.example                                           │
│  • Explorer: builtin                                                         │
│  • Updates: disabled                                                         │
│                                                                              │
│ Actions:  [ New identity ]  [ Rebuild circuit ]  [ Copy status ]             │
│                                                                              │
│ Logs: [ Show ▸ ]                                                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 5) Circuit view (advanced / dev)

```text
┌────────────────────────────── Tor Circuit (Advanced) ────────────────────────┐
│ Circuit status: ● Established     Build time: 1.8s                            │
│                                                                              │
│ Guard:   node..A1  (fingerprint ..)      Latency: 45ms                        │
│ Middle:  node..C7  (fingerprint ..)      Latency: 80ms                        │
│ Exit:    node..F2  (fingerprint ..)      Latency: 120ms                       │
│                                                                              │
│ Streams:                                                                     │
│  • rpc.devnet.example    state: active   bytes: 120KB                         │
│  • explorer (builtin)    state: idle     bytes: 12KB                          │
│                                                                              │
│ [ New identity ]   [ Close ]                                                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 6) “New identity” confirm (rotates circuits)

```text
┌────────────────────────────── New Identity ──────────────────────────────────┐
│ Request a new Tor identity / circuit?                                        │
│                                                                              │
│ Effect: may interrupt ongoing requests and pending broadcasts.               │
│                                                                              │
│ [ ] Also reset RPC endpoint (if Auto)                                        │
│                                                                              │
│                      [ Cancel ]                     [ New identity ]        │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 7) Per-request warning (when Tor is required but down)

```text
┌────────────────────────────── Network Blocked ───────────────────────────────┐
│ ⊗ Tor is enabled but unavailable                                             │
│                                                                              │
│ Clearnet fallback is blocked (safe mode).                                    │
│                                                                              │
│ [ Retry ]   [ Open Tor settings ]   [ Disable Tor ]                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

------

## 8) Minimal toggle in Network selector (fast switch)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Network: [ Devnet ▾ ]     Tor: [ ON ▾ ]   ( ) OFF ( ) ON (SOCKS5) ( ) Embedded│
└──────────────────────────────────────────────────────────────────────────────┘
```

Если хочешь, я сделаю ещё 2 мокапа:

1. **“Tor + I2P routing”** (двойной транспорт, приоритеты, fallback-матрица)
2. **“RPC endpoints manager”** с флагами “Tor-only / clearnet-only / both” и health-check таблицей.
