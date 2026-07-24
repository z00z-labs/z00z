# Production port map

The demo is an executable UX specification for the packaged Z00Z Wallet. Its
JavaScript, fabricated data, and mock gateway must never be imported by production.
Production translates the stable contracts below into Rust and Leptos types.

## Runtime decision

The application uses Leptos CSR/WASM inside the Tauri 2 WebView. It is not a browser product:
Trunk produces bundled static assets that Tauri packages with the application.
The renderer owns declarative views and ephemeral presentation state only.

The wallet backend stays in native Rust. Passwords, seeds, private keys, session
tokens, wallet files, signing, policy enforcement, configuration mutation,
operation journals, and settlement never enter renderer state. Windows/Linux use
an authenticated local IPC adapter to the wallet backend. iOS implements the same
typed `WalletGateway` contract through an in-process native adapter.

The GitHub Pages release checker and SHA cache-busting transform are preview
deployment tools only. They must not be ported into the packaged application.
The Tauri bundle is versioned and updated by the native application release
channel; the renderer must remain fully functional without internet access.

```text
Leptos views/store (Rust/WASM)
          |
          v
allowlisted Tauri commands and sanitized events
          |
          v
WalletGateway (native Rust)
    |                 |
Windows/Linux      iOS
authenticated IPC  typed in-process adapter
    |                 |
    +---- z00z_wallets+
```

## Mechanical module map

| Demo source | Production destination | Port rule |
|---|---|---|
| `scripts/port/contracts.js` | `z00z_wallet_ui_contract::{route, capability, command, error}` | Translate identifiers to exhaustive Rust enums and serialized DTOs. Reject unknown variants at the boundary. |
| `scripts/port/fixtures.js` | contract-test fixtures and `MockWalletGateway` fixtures | Keep fabricated data in test/demo features only. Do not compile it into release builds. |
| `scripts/port/presentation-state.js` | `z00z_wallet_ui::{model, store, route}` | Convert to typed reactive signals/resources. Keep only renderer-safe, ephemeral fields. |
| `scripts/port/mock-wallet-gateway.js` | `z00z_wallet_ui_contract::WalletGateway` test adapter | Preserve intent/result envelopes; replace mutation internals with native gateway calls. |
| `scripts/port/locale-registry.js` | locale enum, metadata registry, and catalogue builder | Generate every catalogue from one ordered Rust/build-time registry. |
| `scripts/port/icon-registry.js` | `IconName` enum and one inline SVG Leptos component | Preserve semantic object-type lookup; bundle paths locally and normalize to 24x24. |
| `help/topics.yaml` | build-time `HelpTopicId`/route matcher source | Generate an exhaustive Rust enum and fail the build when a routed view has no topic. |
| `help/<locale>/*.md` | bundled localized Help source | Compile at build time into a constrained plain-text AST; never parse or fetch Markdown at runtime. |
| `scripts/port/help-registry.js` | pure Help topic resolver | Port as an exhaustive state-to-topic match independent from Leptos view components. |
| `scripts/help-controller.js` | `HelpPanel`/`ContextHelpButton` components | Preserve focus restoration, Escape/backdrop close, target highlighting, desktop panel, and mobile bottom-sheet behavior. |
| `styles/colors.css` | `z00z_wallet_ui/styles/colors.css` | Preserve literal palette values as the single colour source. |
| `styles/foundation.css` | `z00z_wallet_ui/styles/foundation.css` | Bundle local fonts and global design tokens. |
| `styles/components.css` | Leptos component styles | Port by component; do not reproduce DOM selectors that no longer exist. |
| view/render functions in `app.js` | `z00z_wallet_ui::views` and `components` | Port behaviour and accessibility contracts, not HTML string construction. |
| event handlers in `app.js` | Leptos callbacks/actions | Convert UI intent to typed gateway methods. Never call transport primitives directly. |
| Tauri command handlers | `z00z_wallet_ui_tauri` bridge crate | Expose a small allowlist and return sanitized DTOs/errors only. |
| backend adapters | native gateway crate(s) | Own authentication, authorization, storage, config, signing, reconciliation, and telemetry acquisition. |

## Gateway boundary

Queries and commands come from `PORT_CONTRACT`. Production should expose one
typed method per allowlisted operation rather than a generic method-name
dispatcher. A successful transport response acknowledges delivery only; operation
state and final settlement come from authoritative reconciliation queries/events.

Wallet inventory DTOs are wallet-scoped: `assetKeys`, vouchers, permissions, and
activity never fall back to another profile. `create_voucher`,
`create_permission`, `transfer_voucher`, and `transfer_permission` are explicit
gateway intents in the demo contract. The deterministic adapter may complete
them locally; production must capability-check issuer/delegation authority,
persist an idempotent operation, and reconcile authoritative object state.

Renderer DTOs must not contain `password`, `seed_phrase`, `private_key`,
`session_token`, `raw_signed_package`, or `arbitrary_filesystem_path`. Password
fields are passed once to a native command and cleared immediately. Seed and key
inspection requires a short-lived native reveal surface; it is not application
state and is never serialized into YAML.

`WalletChainId` is a closed enum (`mainnet`, `testnet-1`, `testnet-2`,
`devnet-1`, `devnet-2`) supplied to new-wallet creation. The native gateway must
validate and persist it atomically with the profile. Renderer projections expose
the bound value as read-only; rename/settings/YAML commands cannot mutate it.

YAML in this demo is a presentation concept. Production parses, validates,
authorizes, revisions, and writes configuration in a typed native
`ConfigGateway`; the renderer receives a sanitized projection and diagnostics.

Help is also a build-time projection. Markdown and locale files are developer
inputs, not renderer data loaded from disk. The packaged renderer receives an
immutable embedded catalogue. Locale IDs are generated from the same canonical
registry as the UI. The Help build derives the full route matrix from
`PORT_CONTRACT` and fails unless each routed state has exactly one contextual
topic and every topic maps back to a route. Topic content cannot contain raw
HTML, script URLs, selectors, secrets, wallet labels, addresses, or other
user-authored values. English is an explicit fallback only for an unavailable
packaged catalogue; release builds fail locale-parity checks before packaging.

When Help is opened from a native sheet/dialog, the production `HelpPanel`
belongs to the same overlay stack, temporarily inerts the underlying sheet,
contains focus, handles Escape/backdrop closure, and restores the invoking
control. Do not render contextual Help behind a native top-layer dialog.

The shared compact-row contract ports as three semantic slots—label, value, and
action/status. Long values truncate without losing their accessible full value.
At 320 px, ordinary settings stay one logical row; only complex editors and
palette grids may use their declared stacked fallback. Do not port one-off
per-view width overrides.

## Vertical migration order

1. Create Rust enums/DTOs from `contracts.js`, with serialization and rejection
   tests for unknown identifiers.
2. Define the async `WalletGateway` trait and implement the deterministic mock
   with the fixture module.
3. Port the application shell, route store, local fonts, colour LUT, icon
   component, and locale registry into a minimal Leptos CSR/Trunk build.
4. Port one vertical wallet slice—wallet selection, Assets, Send, Receive, and
   History—against the mock gateway and retain the Playwright accessibility and
   responsive assertions.
5. Add the allowlisted Tauri bridge and desktop native gateway adapter; verify
   CSP, command scope, error sanitization, lifecycle, and operation recovery.
6. Implement the iOS adapter against the same gateway contract and validate
   safe-area, keyboard, touch-target, lock/background, and secure-storage flows.
7. Port wallet settings, backup/policy flows, and read-only telemetry one slice at
   a time. Delete the corresponding demo-only renderer implementation after each
   production acceptance gate passes.

## Deferred production spike

This refactor intentionally does not add or pin Tauri, Leptos, Trunk,
`wasm-bindgen`, platform plugins, or IPC protocol versions. The production spike
must prove Windows/Linux packaging and authenticated IPC, iOS lifecycle and native
adapter behaviour, CSP without remote assets, local font loading, accessibility,
secure storage, update/signing policy, and recovery after process interruption
before versions are selected.
