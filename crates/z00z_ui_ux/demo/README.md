<!-- markdownlint-disable MD013 -->

# Z00Z Wallet Demo

This self-contained prototype is the executable companion to [`UI-UX-SPEC.md`](../../../.planning/phases/110-Wallet-UX-UI/UI-UX-SPEC.md). It uses fabricated data and never connects to a wallet or signs a transaction.

## ▶️ Run

From the repository root:

```bash
node crates/z00z_ui_ux/demo/scripts/serve-demo.mjs 4173
```

Open `http://127.0.0.1:4173`. This development server watches
`help/**/*.{md,yaml,yml}`, synchronizes English view evidence without touching
authored localized source, recompiles every local Help catalogue, and reloads the open page after a
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

Every locale uses the same folder metadata contract as `z00z-website`.
`_meta.yaml` accepts the Website metadata surface. Help navigation applies
`title`, `order`, and `icon`; page headings and link text remain owned by the
article front matter. A missing directory `title` falls back to the directory
name. Listed `order` entries come first; missing entries are ignored and
unlisted content is appended alphabetically, matching the Website loader.

The Demo navigation model remains authoritative for the primary Help navbar:
root branches and their first-level App destinations are projected unchanged,
so the navbar never nests more than once. A workspace directory such as
`wallet/assets/` represents one primary destination. When one topic needs more
than one article, create a directory for that topic and place its complete
Markdown pages there. Every direct page in that directory becomes Main View
navigation, in `_meta.yaml` order, while the directory itself remains one
primary-navbar item. Route, contextual, and dialog articles follow the same
folder rule and are never repeated in the primary navbar. Folder metadata may
override a displayed Help title without changing this App-owned structure.
`guides/` is the explicit standalone Help section.

Additional Markdown with `route: none` and `scope: article` or `scope: guide`
is discovered without editing a JavaScript registry. Put it directly inside an
App branch directory to add a first-level Help article, or inside a workspace
directory to add it to that workspace's Main View navigation. Dialog Help stays
contextual. `help/topics.yaml` contains the generated App route/topic contract;
content placement comes from the filesystem and `_meta.yaml`.

On mobile, Help reuses the App popup-drawer, backdrop, focus, and edge-swipe
contracts. Search lives in the topbar before Languages, opens a dedicated
results modal, and never replaces the navigation tree.

`scripts/help/markdown-renderer.mjs` imports an exact synchronized snapshot of
the sibling `z00z-website` renderer. Thus Help Markdown uses the same MarkdownIt
pipeline, sanitization, anchors, extensions, figure handling, and external-link
policy as the website without a runtime Website checkout. The resulting network-free catalogue is
`scripts/generated/help-catalog.js`.

Route articles contain front matter, an `App View` screenshot, overview,
workflow, terms and controls, and safety limits. General guides contain
front matter, overview, usage guidance, and safety limits. Missing topics must
be authored as complete canonical articles; the capture synchronizer never
creates parallel review Markdown. The global and contextual Help actions both open the named standalone
`help.html` tab at the exact topic and `#current-view` anchor without changing
the application page.

Run the non-destructive view sync after a UI change:

```bash
python3 scripts/help/sync_views.py
```

It captures all English views, extracts visible terms, sections, component
signatures, and settled presentation signatures, then compares them with
`help/en/_generated/`. A changed view updates only its screenshot and generated
capture state; it never creates or changes Markdown. Review the reported topic
and update its canonical article directly when its explanation must change. The
development server watches view sources and Help Markdown/YAML, rebuilds the
catalogue, and reloads open Help pages automatically. Use
`python3 scripts/help/sync_views.py --check` for
portable baseline integrity and `python3 scripts/help/sync_views.py --verify-current`
for a live Chromium drift gate, then run `node scripts/compile-help.mjs` and
`node scripts/check-help.mjs`.

Help inherits the App language through its `lang` URL parameter. The generated
catalogue resolves each registered locale from its existing stable-topic Markdown
source, wraps it in the same `App View`/Website-Markdown structure, and fails
closed if a source is missing or conflicts. English screenshots remain shared UI
evidence; localized Markdown is never machine-translated or overwritten.

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
4. Select Everyday, Savings, and Travel in the desktop wallet navigation. Confirm that Assets, History, Swap, Earn, Backup, Settings, and the bottom status bar reflect only the selected wallet. Hover the copy control beside the address to reveal the full selected-wallet ID.
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
- `scripts/port/icon-sprite.js` is the sole SVG geometry source for App and
  Help; HTML pages mount that shared local sprite instead of copying symbols.
- CSS tokens are intended to seed the Leptos production design system.
- Claim intake RPC, network detail, compliance-profile loading, and runtime YAML write/watch controls are simulated target capabilities, not claims about the live backend. Selected-wallet settings stay concept-local until a revisioned settings bridge exists; advanced settings can apply a safe YAML draft only to the in-browser concept state.
- The demo is a development-only visual reference. Production is the packaged standalone Tauri application with local-only IPC; it has no browser, container, or wallet HTTP/WebSocket profile and does not connect the demo to a wallet backend.

## 🧩 Refactoring seams

The port-facing modules under `scripts/port/` separate frozen identifiers,
fixtures, presentation state, the mock gateway, locales, and semantic icons from
DOM rendering. They map mechanically to Rust contracts, a Leptos store, and the
native `WalletGateway`; the JavaScript remains demo-only and is not a production
dependency. See [RUST-PORTING.md](RUST-PORTING.md) and
[Refactoring-PLAN.md](../../../.planning/phases/110-Wallet-UX-UI/Refactoring-PLAN.md).

The CSS entry imports `styles/colors.css`, `styles/foundation.css`, and
`styles/components.css` in that order. Literal application colours remain
centralized in `styles/colors.css`. Shared breakpoints, drawer gestures,
floating-panel geometry, language-picker structure, and icon rendering live in
`scripts/port/ui-primitives.js`; App and Help consume those LUTs.

Run the deterministic gates independently with:

```bash
node scripts/check-locales.mjs
node scripts/check-design-system.mjs
node scripts/check-menu-icons.mjs
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
