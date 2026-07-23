<!-- markdownlint-disable MD013 -->

# Z00Z Wallet Interactive Concept

This self-contained prototype is the executable companion to [`UI-UX-SPEC.md`](../../../.planning/phases/110-Wallet-UX-UI/UI-UX-SPEC.md). It uses fabricated data and never connects to a wallet or signs a transaction.

## ▶️ Run

From the repository root:

```bash
python3 -m http.server 4173 --directory crates/z00z_ui_ux/demo
```

Open `http://127.0.0.1:4173`.

The files can also be opened directly. The local HTTP server exists only for development screenshots and smoke tests; it is not part of the product runtime.

## 📱 GitHub Pages

GitHub renders repository files as source code; it does not execute `index.html`
from the file view. The `publish-wallet-demo` workflow packages this static demo
for GitHub Pages at:

```text
https://z00z-labs.github.io/z00z/wallet-demo/
```

One-time repository setup: **Settings → Pages → Build and deployment → Source →
GitHub Actions**. After it is enabled, each version tag created by the repository
version manager publishes the current demo automatically. A normal push to
`main` without a skip instruction also publishes it; a manual release remains
available at **Actions → publish-wallet-demo → Run workflow**. Open the URL on a
phone, or use the browser's responsive-device mode. Before publishing, the
workflow verifies the complete local bundle: styles, fonts, icons, locale
catalogues, and asset imagery. A partial unstyled upload fails in CI instead of
reaching Pages.

The page exposes a local Web App Manifest plus 192 px, 512 px, maskable, and
Apple touch icons generated from `assets/logo/z00z-logo-gold-circle.png`.
Android launchers cache existing shortcuts independently of the browser cache:
after an icon update, remove the old home-screen shortcut, deploy the new
bundle, reload the page, and add the shortcut again.

Every Pages artifact versions its CSS, JavaScript, manifest, fonts, and images
with the deployed commit SHA. The Pages-only release checker reads
`deployment.json` on startup, when the page becomes visible or online, and once
per minute. When `main` publishes a new SHA, the checker reloads the stable demo
URL with that SHA, bypassing stale mobile-browser resources. A failed update
check never blocks the locally loaded preview. This checker is deployment
tooling; it is not part of the offline Tauri runtime.

Optional full visual smoke test (it starts and stops its own local HTTP server):

```bash
crates/z00z_ui_ux/demo/run-smoke.sh
```

## 🌐 Languages

The concept includes English, Russian, French, German, Spanish, Portuguese,
Korean, Turkish, Japanese, and Simplified Chinese UI catalogues. One canonical
locale registry owns their metadata and load order. Language, regional format,
and display time zone are independent preferences. See
[I18N-ARCHITECTURE.md](../../../.planning/phases/110-Wallet-UX-UI/I18N-ARCHITECTURE.md) for the catalogue contract, local
machine-translation bridge, and required checks.

## 🧪 Suggested walkthrough

1. Resize between desktop, 390 px, and 320 px. The shell has one sticky top row at every width: its scrollable tabs change with the selected wallet, network workspace, or application Settings. On mobile the desktop identity/actions are replaced in that same row by Menu and the Z00Z mark; bottom navigation is absent. Menu opens a full-height drawer with Wallets, Network, Settings, and Log out; Wallets and Network open nested pickers. Assets and wallet Settings open compact third-level popup menus.
2. Use Send, asset Claim, and Give permission from Home; confirm Receive opens the selected wallet's single Receiver Card.
3. Confirm that submitted sends, claim outputs, voucher redemption, and permission delegation show honest non-final states.
4. Select Everyday, Savings, and Travel in the desktop wallet navigation. Confirm that Assets, History, Swap, Exchange, Staking, Backup, Settings, and the bottom status bar reflect only the selected wallet. Hover the copy control beside the address to reveal the full selected-wallet ID.
5. Use **Log out**, confirm the application shell is hidden, then unlock; the password field and visible sensitive presentation state must be cleared on lock.
6. Open Assets and compare Assets, Vouchers, and Permissions in the context rail; conditional and zero-value objects never appear in Available.
7. Verify that Claim and Voucher are separate flows: Claim reviews source proof/recipient/output/nullifier, while Voucher uses accept/redeem lifecycle actions.
8. Open OnionNet, Reticulum, and Aggregators from the left Network group. Each opens a separate read-only telemetry workspace, never a setup page. The single top row replaces the wallet tabs with that workspace's tabs. OnionNet uses Overview, Epoch, Privacy, Transport, Queues & Replay, Probation, and Ingress. These panels distinguish public deterministic state, local evidence, and aggregate synthetic health; they never reveal a user route, endpoint, session, or universal privacy score. Reticulum begins with Overview, then uses Node, Interfaces, Radio, Entry points, Paths, Probes, and Links for managed-node/local evidence rather than claims about the whole network. Aggregators begins with its own Overview and describes only a future wallet-to-node status bridge. Unavailable evidence stays unavailable.
9. Inspect a selected wallet’s Settings → Policies and the restriction-layer / “Why blocked?” model.
10. Inspect a selected wallet’s Settings → Advanced for the local concept YAML draft. It validates and updates only that wallet’s demo state; production configuration write/watch/revision remains unavailable and is explicitly labelled.
11. Inspect Settings on desktop and narrow widths: the page heading, context tree, and detail card now use one aligned layout; the Network branch opens only when selected.
12. In application Settings → Appearance, use the Dark/Light theme toggle (Dark is the default) and choose Z00Z Default, Black & Gold, Moonlit Stroll, or Walking at Night. Palette changes update semantic tokens while safety colours remain protected. Appearance also selects the application-wide YAML syntax theme: One Light, Xcode, One Dark, or Night Owl.
13. Filter the selected wallet's History and open technical details.
14. Use **Add wallet** to create, open, or restore a profile, or choose **Cancel** to return to the selected wallet. **Remove wallet** confirms before removing one or more selected concept profiles; removing all profiles returns to **Add wallet**. The Wallets placeholder shows exactly three rows and one scrollbar: wallet cards, Add, and Remove are one ordered scroll list. Remove becomes disabled when the list is empty. The recovery helper inserts 24 demonstration words that are never a real seed.
15. Open Assets and select an asset name to inspect its asset-details fields; desktop columns show Name, Balance, Value, and Price. At 390/320 px, each row keeps asset identity on the left and the three numeric fields in a non-overlapping right stack.
16. Open a selected wallet's Settings. Confirm that General, Security, Backup, Policies, and Advanced are scoped to that wallet. Sensitive actions require a fresh password and their typed confirmation; secret/private material is never rendered or placed in YAML.

## 🧱 Constraints

- No production cryptography or RPC calls.
- Official Geist and Geist Mono variable fonts and their OFL license are bundled
  under `assets/fonts/geist/`; the concept makes no remote font request.
- Inline SVG symbols provide icons.
- CSS tokens are intended to seed the Leptos production design system.
- Claim intake RPC, network detail, compliance-profile loading, and runtime YAML write/watch controls are simulated target capabilities, not claims about the live backend. Selected-wallet settings stay concept-local until a revisioned settings bridge exists; advanced settings can apply a safe YAML draft only to the in-browser concept state.
- The demo is a development-only visual reference. Production is the packaged standalone Tauri application with local-only IPC; it has no browser, container, or wallet HTTP/WebSocket profile and does not connect the demo to a wallet backend.

## 🧩 Refactoring seams

The port-facing modules under `scripts/port/` separate frozen identifiers,
fixtures, presentation state, the mock gateway, locales, and semantic icons from
DOM rendering. They map mechanically to Rust contracts, a Leptos store, and the
native `WalletGateway`; the JavaScript remains demo-only and is not a production
dependency. See [PORTING.md](PORTING.md) and
[Refactoring-PLAN.md](../../../.planning/phases/110-Wallet-UX-UI/Refactoring-PLAN.md).

The CSS entry imports `styles/colors.css`, `styles/foundation.css`, and
`styles/components.css` in that order. Literal application colours remain
centralized in `styles/colors.css`.

Run the deterministic gates independently with:

```bash
node scripts/check-locales.mjs
node scripts/test-port-contracts.mjs
node scripts/check-port-readiness.mjs
node scripts/test-pages-release.mjs
```

`run-smoke.sh` runs these gates before the full Playwright suite.
