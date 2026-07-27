<!-- markdownlint-disable MD013 -->

# Z00Z Wallet Demo

This self-contained prototype is the executable companion to [`UI-UX-SPEC.md`](../../../.planning/phases/110-Wallet-UX-UI/UI-UX-SPEC.md). It uses fabricated data and never connects to a wallet or signs a transaction.

## ▶️ Run

From the repository root:

```bash
node crates/z00z_ui_ux/demo/scripts/serve-demo.mjs 4173
```

Open `http://127.0.0.1:4173`. This development server watches
`help/en/**/*.{md,yaml,yml}`, runs the English-to-all-locales Help synchronizer,
recompiles the local Help catalogue, and reloads the open page after a
successful update.

The files can also be opened directly. The local HTTP server and its live-reload
channel exist only for development; neither is part of the product runtime.

## 📱 GitHub Pages

GitHub renders repository files as source code; it does not execute `index.html`
from the file view. The `publish-wallet-demo` workflow packages this static demo
for GitHub Pages at:

```text
https://z00z-labs.github.io/z00z/wallet-demo/
```

One-time repository setup: **Settings → Pages → Build and deployment → Source →
GitHub Actions**. After it is enabled, the authenticated repository version
manager dispatches the Pages workflow immediately after it pushes a release to
`main`. A normal push to `main` without a skip instruction also publishes it; a
manual release remains available at **Actions → publish-wallet-demo → Run
workflow**. Open the URL on a phone, or use the browser's responsive-device mode.
Before publishing, the workflow verifies the complete local bundle: styles,
fonts, icons, locale catalogues, and asset imagery. A partial unstyled upload
fails in CI instead of reaching Pages.

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

The separate `run-visual-review.sh` produces the complete review matrix for all
61 canonical routes in both Z00Z Default and Z00Z Corporate at 1280, 1024, 768,
390, and 320 CSS px, plus long-locale, reduced-motion, 200% text-zoom,
localized-Help, detail, dialog, error, and Roadmap states. It finishes with the
frozen Phase 0 token/debt and current screenshot/audit regression gate.

```bash
crates/z00z_ui_ux/demo/run-visual-review.sh
```

## 🌐 Languages

The concept includes English, Russian, French, German, Spanish, Portuguese,
Korean, Turkish, Japanese, and Simplified Chinese UI catalogues. One canonical
locale registry owns their metadata and load order. Language, regional format,
and display time zone are independent preferences. See
[I18N-ARCHITECTURE.md](../../../.planning/phases/110-Wallet-UX-UI/I18N-ARCHITECTURE.md) for the catalogue contract, local
machine-translation bridge, and required checks.

## ❔ Local contextual Help

Application Help is authored as Markdown under
`help/<locale>/{app,wallets,telemetry,dapps,messenger,contacts,data-storage,settings}/`. The canonical
`help/topics.yaml` LUT owns each topic's group and maps presentation state to one
stable topic ID. Locale IDs come from `scripts/port/locale-registry.js`; compile,
check, and scaffold tools do not maintain a second language list.
`scripts/compile-help.mjs` converts the constrained Markdown subset into
`scripts/generated/help-catalog.js`. Runtime rendering reads that bundled
catalogue only: it does not fetch Markdown, load a CDN, or require an Internet
connection.

Pre-existing Help Markdown is never deleted by the canonicalization workflow.
`help/preserved-sources.json` requires the original tracked paths in every
locale and allows explicitly listed locale-only user sources to coexist without
entering the runtime catalogue.

`node scripts/check-help.mjs` derives every routed state from `PORT_CONTRACT`.
It fails unless each routed state resolves exactly one contextual topic, every
context topic maps back to a route or supported detail state, one global topic
exists, all 76 topics exist in all ten locale folders, all translated documents have the English structure,
the English SHA-256 source hashes are synchronized, and the generated catalogue
is current.

English is the canonical Help source. `help/source-state.json` records the
English source and per-locale review hashes for every topic. When an English
topic changes, `scripts/sync-help.mjs` updates all nine localized Markdown
documents, synchronizes folder metadata and landing-page structure, updates the
hashes, and leaves the runtime network-free. The configured translation bridge
takes precedence:

```bash
Z00Z_TRANSLATE_COMMAND=/absolute/path/to/local-translate \
  node scripts/sync-help.mjs
```

Without that override, the bundled synchronizer preserves unchanged reviewed
translations, uses the exact English message for a newly added or changed key,
and never leaves an older structure or source hash behind. Translation output
is a draft and still requires native-language review.

`compile-help.mjs`, the development server, smoke suite, Pages release, and the
repository pre-commit hook all run this synchronization contract. A commit that
stages `help/en/` automatically stages the affected localized files,
`help/source-state.json`, and the compiled Help catalogue. New clones enable the
tracked hooks once with:

```bash
git config core.hooksPath .github/hooks
```

To add a view:

1. Add its state match and stable ID to `help/topics.yaml`.
2. Run `node scripts/scaffold-help.mjs <topic-id>`.
3. Replace every generated translation placeholder.
4. Run `node scripts/sync-help.mjs --record-reviewed` once after that explicit
   review, then `node scripts/compile-help.mjs` and
   `node scripts/check-help.mjs`.

The global Help action opens `help.html` in a named parallel browser tab. Its
left tree is a root-only multi-open accordion organized by the seven canonical
source groups. Workspace-local destinations render as a vertical internal rail
on desktop and sticky horizontal tabs on mobile. The
fixed question action at the lower-right edge opens the same standalone Help
application at the active view's exact topic and `{#current-view}` section. The
wallet page remains open with its navigation, filters, drafts, and forms intact;
repeated Help actions reuse and focus the same Help tab. Explanatory prose
belongs in Help; validation, safety warnings, destructive consequences,
read-only/unavailable states, and errors stay beside the affected action.

## 🧪 Suggested walkthrough

1. Resize between desktop, 390 px, and 320 px. The topbar always retains the
   Z00Z logo and contains no global route tabs. Desktop uses the left sidebar;
   mobile uses the matching full-height drawer. Wallet, Telemetry, dApps,
   Messenger, and Settings are independent root accordions, Contacts is a direct
   destination, and Help/About/Log out remain terminal actions. Closing one accordion
   preserves every other open root and does not navigate. A first-level leaf
   opens its workspace; every deeper destination appears only in that
   workspace's desktop rail or mobile sticky top tabs.
2. Use Send, asset Claim, and Give permission from Home; confirm Receive opens the selected wallet's single Receiver Card.
3. Confirm that submitted sends, claim outputs, voucher redemption, and permission delegation show honest non-final states.
4. Select Everyday, Savings, and Travel in the desktop wallet navigation. Confirm that Assets, History, Swap, Exchange, Staking, Backup, Settings, and the bottom status bar reflect only the selected wallet. Hover the copy control beside the address to reveal the full selected-wallet ID.
5. Use **Log out**, confirm the application shell is hidden, then unlock; the password field and visible sensitive presentation state must be cleared on lock.
6. Open Assets and compare Assets, Vouchers, and Permissions in the context rail; conditional and zero-value objects never appear in Available.
7. Verify that Claim and Voucher are separate flows: Claim reviews source proof/recipient/output/nullifier, while Voucher uses accept/redeem lifecycle actions.
8. Open Reticulum, OnionNet, Aggregators, Watchers, and Explorer from the
   Telemetry root. Each first-level item opens a separate read-only workspace,
   never a nested global accordion or setup page. Its Overview/Node/etc.
   destinations stay in the main-window rail on desktop and top tabs on mobile.
   The panels distinguish public deterministic state, local evidence, and
   synthetic fixture evidence; they never reveal a user route, endpoint,
   session, receiver, message/contact content, or universal privacy score.
   Watchers and Explorer remain visibly bounded Roadmap previews. Unavailable
   evidence stays unavailable.
9. Inspect a selected wallet’s Settings → Policies and the restriction-layer / “Why blocked?” model.
10. Inspect a selected wallet’s Settings → Advanced for the local concept YAML draft. It validates and updates only that wallet’s demo state; production configuration write/watch/revision remains unavailable and is explicitly labelled.
11. Inspect application Settings on desktop and narrow widths: General and
    Appearance are first-level workspace destinations in its desktop rail/mobile
    tabs; there is no nested Network accordion or independent theme-mode control.
12. In application Settings → Appearance, preview and apply either Z00Z Default (dark) or Z00Z Corporate (light); Reset restores Z00Z Default. Palette changes update the bundled semantic tokens while safety colours remain protected. Appearance also selects the application-wide YAML syntax theme: One Light, Xcode, One Dark, or Night Owl.
13. Filter the selected wallet's History and open technical details.
14. Use **Add wallet** to create, open, or restore a profile, or choose **Cancel** to return to the selected wallet. **Remove wallet** confirms before removing one or more selected concept profiles; removing all profiles returns to **Add wallet**. The Wallets placeholder shows exactly three rows and one scrollbar: wallet cards, Add, and Remove are one ordered scroll list. Remove becomes disabled when the list is empty. The recovery helper inserts 24 demonstration words that are never a real seed.
15. Open Assets and select an asset name to inspect its asset-details fields; desktop columns show Name, Balance, Value, and Price. At 390/320 px, each row keeps asset identity on the left and the three numeric fields in a non-overlapping right stack.
16. Open a selected wallet's Settings. Confirm that General, Security, Backup, Policies, and Advanced are scoped to that wallet. Sensitive actions require a fresh password and their typed confirmation; secret/private material is never rendered or placed in YAML.

## 🧱 Constraints

- No production cryptography or RPC calls.
- Official Geist and Geist Mono variable fonts and their OFL license are bundled
  under `assets/fonts/geist/`; the concept makes no remote font request.
- Left sidebar/mobile drawer labels match the approved reference typography:
  Geist 16 px, weight 700, line-height 1.25. Workspace-local desktop rails and
  mobile top tabs retain the original 16 px/700 tab typography.
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

The reproducible visual-review matrix is captured with
`visual-review.spec.js` at 1280×800, 1024×768, 768×1024, 390×844, and
320×800 for both canonical palettes. Its screenshots are
written under `crates/z00z_storage/outputs/checkpoint/phase-110/ui-help-review/`.
Run it with:

```bash
crates/z00z_ui_ux/demo/run-visual-review.sh
```
