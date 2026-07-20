<!-- markdownlint-disable MD013 -->

# Z00Z Wallet UI/UX Verification and Design Review

| Field | Result |
| --- | --- |
| Reviewed baseline | `UI-UX-SPEC.md` v0.4.0 and responsive HTML demo |
| Date | 2026-07-19 |
| Verdict | Pass for concept/prototype validation |
| Backend evidence method | CodeGraph first, then exact current source/manifests when needed |
| Prototype boundary | Fabricated data; no wallet, signing, persistence, or live RPC connection |

## Executive result

The v0.4 concept is internally consistent with the current object model and honest about missing backend capability. The former statement “Claim is a voucher action” is **DISPUTED by live code and removed**. Claim is now a distinct asset-claim transaction flow; Voucher and Permission have independent inventories, lifecycles, actions, and review dialogs. `Right` is zero-value and no visible Permission surface shows a Z00Z budget.

The deployment decision is also explicit: Tauri + Leptos remains the default self-custody shell. A static browser bundle and a local container are viable future surfaces, but current code has only a WASM WebSocket client seam—not an authenticated browser gateway or wallet RPC server. Docker packaging therefore does not make a remote key-bearing wallet self-custodial or production-ready.

The interface now has two consistent selection patterns only:

1. `ContextNav` for route hierarchy: grouped rail/tree on wide screens and a labelled horizontal route strip on narrow screens.
2. `ChoiceChip` for filters and small peer view choices.

Home quick actions are two equal pairs aligned to the two equal lower panels. Attention, Activity, Voucher, and Permission rows share divider, hover, focus, spacing, and metadata behavior. Target-only Reticulum/OnionNet telemetry, policy profiles, and UI↔YAML synchronization never masquerade as live capability.

## Claim ratings

| Rating | Count |
| --- | ---: |
| VERIFIED | 12 |
| PLAUSIBLE / architecture decision | 4 |
| UNVERIFIED | 0 |
| Current DISPUTED | 0 |
| Fabrication risk left in visible demo | 0 |

### VERIFIED against CodeGraph/current source

| ID | Claim | Evidence | Review consequence |
| --- | --- | --- | --- |
| C1 | Protocol and wallet inventory separate Asset, Voucher, and Right | `ObjectFamily`, `WalletInventoryPayload`, `OwnedObjectFamily` | Wallet routes are Assets, Vouchers, Permissions |
| C2 | Asset classes are Coin, Token, Nft, Void and wallet statuses include pending/quarantined states | `AssetClass`; `OwnedAssetStatus` | Class chips and authoritative availability/trust treatment; only verified spendable cash enters Available |
| C3 | Claim is an asset-claim transaction, not a Voucher action | `ClaimTxPackage`, `ClaimTxVerifier`, `verify_nullifier`, `build_claim_stmt` | Dedicated review of source proof, authority, recipient binding, outputs, and replay nullifier |
| C4 | Claimed/imported outputs persist as Asset with distinct provenance | `put_claimed_asset` → `OwnedAssetSource::ManualClaim`; `import_claimed_assets` → `Import` | Successful Claim appears as asset acquisition and later under Assets |
| C5 | Voucher has its own lifecycle and action set | `VoucherLeaf`, `VoucherLifecycleV1`, `VoucherAction`, dedicated `ObjectRpc` methods | Offered/Accepted/Redeemable/Redeemed/etc.; Accept/Reject/Transfer/Redeem/Refund only when state permits |
| C6 | Right is zero-value; amount/budget/value semantics are forbidden | `RightLeaf`, `RightPolicyV1::validate`, `FORBIDDEN_RIGHT_KEYS` | Permission cards show class/action/scope/uses/expiry/delegation, never money |
| C7 | Current high-level capabilities are incomplete | Complete wallet RPC registration and `ObjectRpc`: no claim intake/build route, no dedicated `grant_right`, no config/profile service | Claim, issuer grant, rich profiles, and config synchronization remain capability-gated TARGET contracts |
| C8 | Current network methods and OnionNet crate do not provide target telemetry; no live Reticulum runtime/API exists | `AppService::switch_to_onionet`, `switch_to_tor`, `z00z_networks/onionnet/src/lib.rs`, CodeGraph negative search | Detailed Reticulum/OnionNet screens are explicitly Phase 080 target simulation |
| C9 | Current UI sketch is feature-gated `eframe 0.28`; Tauri/Leptos are not current dependencies | `z00z_wallets/src/lib.rs`, `z00z_wallets/Cargo.toml`, `egui_views` entry files, CodeGraph plus manifest search | Tauri + Leptos is recorded as TARGET architecture; egui remains a migration/diagnostic client |
| C14 | Browser-facing WebSocket client exists, but no authenticated browser RPC gateway/listener exists | `WasmRpcClient`, `RpcTransport`, `LocalRpcTransport`; CodeGraph plus exact listener/Origin/CORS search | Browser UI remains a TARGET deployment until gateway, pairing, Origin policy, and session mapping exist |
| C15 | Key-bearing wallet sessions are process-local and `.wlt` persistence is native-only/file-backed | `WalletSessionManager`, `SessionToken`, `db/mod.rs`, `FileKeyStore` | Raw RPC token cannot be given to browser JS; container persistence requires a dedicated threat model |
| C16 | Existing Dockerfile packages `zuz-node`, not a wallet | `crates/z00z_networks/docker_nodes/Dockerfile` | No current wallet Docker deployment is implied or reused |

### PLAUSIBLE / architecture decisions

| ID | Decision | Why accepted | Required proof before production |
| --- | --- | --- | --- |
| C10 | Tauri 2 + Leptos static CSR | One Rust-oriented view stack, static web assets, strong responsive prototype transfer, four target OS families | Spike Android/iOS lifecycle, native accessibility trees, secure storage, CSP, IPC/sidecar, size and performance; then pin exact versions |
| C11 | Grouped context rail/tree | Preserves hierarchy, readable labels, badges, nested Network routes, and mobile transformation without tab explosion | Task test and screen-reader test with long/localized labels |
| C12 | Protocol → managed → local → per-action restrictions only | Prevents a lower layer from expanding authority and supports plain “Why blocked?” explanations | Dedicated policy/profile authority, signature, conflict, migration, rollback, and audit specification |
| C13 | Local-first Tauri profile with a separately gated browser/container profile | Keeps keys on the user's device by default while retaining a browser deployment option | Threat-model loopback pairing, gateway-owned session mapping, volume hardening, lifecycle, CSP, WebSocket Origin checks, and recovery semantics |

## Resolved conceptual drifts

| Previous drift | CodeGraph verdict | Resolution in v0.4 |
| --- | --- | --- |
| “Claim is an action over Voucher” | False | Claim is a capability-gated asset-claim transaction; Voucher has separate review/action flow |
| Generic Right displayed as monetary Budget | False | Visible label is Permission; internal icon/classes renamed; zero-value is explicit |
| OnionNet, Reticulum, and Tor treated as equivalent choices | False for Phase 080 target model; current live network is stubbed | OnionNet = privacy overlay, Reticulum = primary carrier, chain = separate; every detailed value labelled target |
| Rich profile shown as Applied | Unsupported by current RPC | Demo status is Preview and “would block,” never current enforcement |
| UI/YAML shown as already Synced | Unsupported by current RPC | Demo status is Target preview; preserve/watch/last-known-good are explicitly target requirements |
| Declared asset domain treated as trust | Unsupported | Domain is metadata; review/quarantine uses authoritative policy availability |
| Docker/container described as wallet security | False as a standalone claim | Container is a constrained packaging/operations option; key custody, RPC gateway, browser session, and data-volume security are separately specified |

## UI consistency audit

### Navigation

- Four global workspaces remain fixed: Home, Wallet, Activity, Settings.
- Wallet context routes are Assets, Vouchers, Permissions; no Claims pseudo-family.
- Settings uses grouped hierarchy, with Network as the one real on/off accordion branch: activation opens Network Overview and its children; repeat activation collapses it and restores Overview; child selection reopens it. Exactly one route is current—the parent for Overview, otherwise the child. Leaf routes show no trailing chevron.
- The Settings context rail starts directly with its first group and does not repeat the already visible Settings workspace/page title.
- Wide screens use a left context rail; 768–900 px portrait tablets and mobile use the same route model as a horizontally scrollable strip.
- Active entries use `aria-current="page"`; filter chips use pressed/selected choice semantics rather than tab semantics.

### Layout and component behavior

- At 1920 px, the four quick actions are grouped 2+2 and their pair widths match the two lower panel widths within 1 px.
- At 768 px, context navigation moves above content; all four Voucher filters fit and the former clipped layout is gone.
- Attention, Activity, Voucher, and Permission lists use the same 1 px divider rhythm; the final row omits the divider.
- Interactive rows gain a contained hover outline without layout shift. Static Permission rows do not acquire button hover treatment.
- Buttons, route entries, chips, icon buttons, switches, segmented controls, links, and actionable cards have visible hover/focus states. Global workspace selection is neutral; gold is limited to local context, authorization, and meaningful status.
- Settings heading spans above the 240 px context rail and detail card; neither selected routes nor long labels cross into the content column. Common settings rows share one control edge, including input/select/button/toggle variants.
- The Settings detail form collapses to one column without horizontal document overflow at 390 px; the Network route strip remains internally scrollable and its active child is visible.
- Mobile 390 px has no document-level horizontal overflow; the active nested Network route is scrolled into view.

### Object-specific visual grammar

- Assets show class and availability/trust treatment; non-native/non-cash items never silently enter Available.
- Vouchers use ticket imagery and show issuer, backing, face/remaining value, lifecycle, validity, and allowed next actions.
- Permissions use a key/authority icon and show class, action, scope, uses, expiry, delegation; monetary value is explicitly none.
- Claim uses a claim/source-proof icon but is recorded as asset activity after submission.

## Scenario review

| Scenario | Result |
| --- | --- |
| Pay / Receive | One dominant action, review before mutation, submitted is not final |
| Claim allocation | Correct asset-claim fields, one-use nullifier, dedicated capability disclaimer |
| Voucher review/redeem | Separate three-step lifecycle; acceptance does not inflate Available |
| Give/revoke Permission | Starts from held delegable authority, enforces attenuation, zero-value review |
| Unsafe object | Quarantine language and authoritative policy reason; no silent balance promotion |
| Network diagnosis | Overlay/carrier/chain separated; current live telemetry stated unavailable |
| Policy profile | Preview-first and restriction-only; no legal-compliance claim |
| UI↔YAML | One target effective configuration with provenance/revision/conflict/rollback contract; current read-overlay baseline disclosed |

## Verification results

| Check | Result |
| --- | --- |
| CodeGraph status and focused exploration | Pass; 2,096 indexed files; exact object/RPC/network/config/UI seams checked |
| `node --check demo/app.js` | Pass |
| `html-validate demo/index.html` | Pass |
| `demo/run-smoke.sh` | Pass: 4/4; semantic flows, lock/unlock sensitive-field clearing, target disclosure, real Network accordion, exactly one current route, leaf affordances, alignment, mobile overflow, neutral global selection, hover, focus |
| Chromium screenshots | Pass: 1920×1080 Home; 1280×800 Permissions, Vouchers, and Network; 768×1024 Vouchers and Reticulum; 390×844 Home, OnionNet, Network, and General Settings |
| Lighthouse Home | Accessibility 100; Best Practices 100 |
| Lighthouse Advanced configuration | Accessibility 100; Best Practices 100 |

## Production gates not proven by this prototype

- Native Windows/Linux/iOS/Android rendering, assistive-technology trees, safe areas, keyboard avoidance, background/suspend behavior, biometrics, and secure storage.
- Real `WalletGateway` transport integration, session handling, idempotency, large inventory performance, and RPC latency/error races.
- Browser gateway, local pairing/session binding, exact-Origin WebSocket policy, CSP, loopback/TLS posture, container hardening, volume backup/restore, and a separate hosted-custody threat model.
- Lossless comment/order-preserving YAML editor and authoritative config revision/watch/rollback service.
- Reticulum/OnionNet status capability and no-fallback enforcement from Phase 080.
- Compliance-profile authority, legal ownership, signatures, schema migration, conflict semantics, and audit trail.
- Localization, RTL, 200% text, forced-colors on native targets, and real screen-reader task sessions.

## Final review verdict

The specification and demo are suitable as the Phase 110 concept baseline. Implementation must use the evidence ledger as a capability boundary: a `LIVE` method may be connected; a `TARGET` screen must remain disabled/unavailable until its authoritative RPC exists; a `UX` decision may change only through an explicit design revision and synchronized update of SPEC, demo, tests, and review.
