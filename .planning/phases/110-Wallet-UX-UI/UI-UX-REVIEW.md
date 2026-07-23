<!-- markdownlint-disable MD013 -->

# Z00Z Wallet UI/UX Consistency and Capability Review

| Field | Result |
| --- | --- |
| Review date | 2026-07-20 |
| Scope | Responsive wallet demo, design specification, legacy mockups, current wallet code, and official Z00Z documentation |
| Review mode | Audit, implementation, and browser verification |
| Verdict | **Demo consistency backlog implemented; service-backed P2 capabilities remain explicitly unavailable** |
| Visual direction | Preserve the current dark, restrained, gold-accented Z00Z direction and its default palette |
| Primary implementation source | This review defines the backlog; `UI-UX-SPEC.md` remains the normative design contract |

## 🎯 Executive conclusion

### ✅ Implementation update — 2026-07-20

The executable demo now applies the P0/P1 findings in this review. It uses the normalized Geist/Geist Mono lookup table, with a reserved variable-weight Geist wordmark matching the public Z00Z docs header. It has no raw visible component font sizes below the documented token floor, and keeps prose in Geist while literal addresses, quantities, IDs, and YAML stay in Geist Mono—except the selected desktop-wallet address, which uses normal-weight Geist at the same desktop size and tracking as the Z00Z wordmark. All textual Cancel actions use the same neutral bordered secondary button; wallet selection uses the same horizontal rail grammar as the internal tabs; the wallet rail scrolls independently; route telemetry is a non-interactive item in the selected-wallet status bar rather than a top-bar button; and the mobile status surface is document-flow content rather than a viewport overlay.

Appearance now offers the preserved Z00Z Default palette plus Black & Gold, Moonlit Stroll, and Walking at Night. Each preset maps the full token set in light and dark modes. Moonlit Stroll uses moonlit teal and navy structure; Walking at Night uses cool blue-charcoal surfaces with a warm stone raised layer. Z00Z amber remains primary in both presets, while success, warning, failure, focus, and environment colours remain semantic. YAML syntax is configured application-wide with the canonical One Light, Xcode, One Dark, and Night Owl token sets from the Z00Z website; it never changes safety colours, wallet data, or runtime data.

Selected-wallet Advanced now exposes a safe, editable local concept YAML draft. Security, Backup, Policies, and Advanced are selected-wallet settings; General, Appearance, and Network & privacy remain application settings. Wallet controls and YAML update the same demo state, while secrets and local paths are rejected. `Apply locally` is deliberately restricted to the browser concept; the boundary explaining that runtime configuration write, watch, revision, conflict, and rollback RPCs do not exist remains visible. This is not a claim that a real wallet file can be changed.

Verification completed: `node --check`, `git diff --check`, and all 16 Playwright smoke tests pass. Desktop Wallets, Appearance, Advanced YAML, and telemetry screenshots were inspected after the change.

Public reference check: [`z00z.io/docs`](https://www.z00z.io/docs) renders its header wordmark with the Geist family, variable weight around 780, uppercase lettering, `0.045em` tracking, a 44 px desktop mark, and a 26 px desktop wordmark. The demo keeps the same font treatment but deliberately enlarges the desktop workspace lockup to a 52 px mark and 34 px wordmark for the requested navigation prominence; it does not copy the documentation site's web navigation or content density.

The following audit records the pre-implementation baseline and remains useful as the rationale and acceptance evidence for the changes above. The current demo has the correct product direction and most of the requested information architecture. The baseline issue was not different wallet font files: every sidebar wallet card was generated from the same template and inherited the same `Geist`/`Geist Mono` tokens. The perceived mismatch came from inconsistent semantic assignment, small raw sizes, and card/dialog overrides that did not match the specification LUT.

The font direction itself is now aligned with the public site. The public Z00Z docs header uses Geist Sans for the wordmark with a variable weight around 780, uppercase treatment, `0.045em` tracking, a 44 px logo mark, and a desktop wordmark around 26 px. The demo follows that type contract, scales the workspace lockup up to 52/34 px, and removes the redundant `Wallet` product suffix. `Geist Mono` remains limited to addresses, identifiers, YAML, timestamps, and aligned numeric data. Wallet names, descriptions, helper text, navigation, and ordinary status explanations use Geist.

The current `UI-UX-SPEC.md` contains a full typography LUT, but the demo does not consistently consume it and several LUT minimums are too small for the visual density requested. Before styling implementation, amend the LUT for ordinary rows, controls, metadata, labels, and status text, then remove every local size override that has no documented exception. All wallet cards, asset cards, activity rows, dialogs, settings rows, and navigation surfaces must map to the same semantic tokens.

Interaction styling also has a confirmed systemic inconsistency. The shared `.button` component is bordered, but `.button-quiet` explicitly removes the border. Add Wallet, Send, Receive, Claim, Permission, and Remove Wallet dialogs use that quiet variant for `Cancel`; Open Existing Wallet uses a bordered `Cancel`. Every textual `Cancel` must use the same neutral bordered secondary button. Icon-only close buttons remain icon buttons and are not part of this rule.

The application/capability model must remain honest. The current dispatcher registers 75 methods, but several asset/network routes are compatibility, preview, or stub paths rather than canonical completed functionality. There is no exchange provider API, app configuration CRUD/watch/revision API, notification preferences API, wallet rename API, or safe local-profile detach API distinct from permanent wallet deletion. The UI must not present these as live operations until the matching contract exists.

Appearance needs to evolve from theme plus accent into a real semantic-palette system. The current palette becomes the immutable `z00z-default` preset. The three selected palettes become optional presets with protected success, warning, danger, focus, privacy, and environment meanings. Selecting a palette must update the complete token map, not merely replace gold with another accent.

Advanced settings must eventually expose the real effective App and selected-Wallet configuration as editable YAML with bidirectional UI synchronization. The current runtime only reads and merges YAML; it does not expose write, watch, revision, conflict, rollback, or provenance RPCs. The existing demo editor is correctly described as a target preview, but its Apply affordance must remain disabled in production until those services exist. Secrets and machine-specific absolute paths must never appear in the editor.

## 👁️‍🗨️ Scope and evidence

| Evidence set | Method | Result |
| --- | --- | --- |
| Current demo | Source inspection plus 1440 px, 390 px, wallet removal, Assets, Add Wallet, Appearance, and Advanced Settings screenshots | Layout and visual defects recorded below |
| Typography | Computed token/source audit of `demo/styles.css`, `demo/index.html`, and wallet render templates | Three-family contract confirmed; token drift quantified |
| Interaction consistency | Shared component classes and every current `Cancel` call site | Border inconsistency confirmed and scoped |
| Legacy ASCII mockups | All 58 files and `001-functions-table.txt` | Useful task flows separated from unsupported legacy assumptions |
| Guarda references | Images `1.png` through `8.png` | Reusable interaction patterns identified; custodial/market patterns rejected |
| Zano references | Images `1.png` through `7.png`, `10.png`, `11.png`, and `12.png` | Multi-wallet, assets, history, receive, add-wallet, and copy-address patterns retained |
| Palette references | Four requested palette images | Exact source colors and contrast constraints recorded |
| Official Z00Z docs | Learn, protocol, receiver, wallet, RPC, configuration, network, OnionNet, and payment-request pages | Product states and privacy boundaries mapped into the UI |
| Current codebase | CodeGraph first, then live Rust types, dispatcher wiring, runtime configuration, and egui theme seam | Live, compatibility, target, and rejected capabilities separated |

## 🚨 Priority findings

| ID | Severity | Finding | Required outcome |
| --- | --- | --- | --- |
| P0-01 | Blocker | Typography tokens exist but are bypassed by 38 raw size declarations; visible text reaches 11.2 px | Amend the LUT, map every visible text node, and remove unsupported raw sizes |
| P0-02 | Blocker | `Geist Mono` is used for ordinary descriptions and wallet/card metadata, creating the apparent mixed-font result | Restrict mono to verbatim data and tabular values; all ordinary language uses Geist |
| P0-03 | Blocker | The Appearance `System` theme button has a malformed class attribute in the template and is not rendered as an independent control | Repair markup and add a DOM/screenshot assertion for all three theme choices |
| P0-04 | Blocker | Advanced YAML is a read-only fabricated preview while Apply looks actionable | Disable or capability-gate Apply until real get/validate/apply/watch/revision/rollback services exist |
| P0-05 | Blocker | UI label `Remove wallet` can be confused with registered permanent `delete_wallet`; no distinct safe detach RPC exists | Define detach semantics/API or use explicit permanent deletion language, consequences, password, and confirmation |
| P0-06 | Blocker | Mobile fixed status/navigation surfaces overlap page content and actionable rows | Reserve safe-area and fixed-bar space; verify no content is obscured at 390 × 844 and 200% zoom |
| P1-01 | Major | Textual `Cancel` uses both borderless and bordered variants | Use one neutral bordered secondary `Cancel` everywhere |
| P1-02 | Major | Wallet cards use local density/truncation rules; selected and unselected cards do not preserve equal text rhythm | One wallet-card component, fixed semantic rows, equal padding, stable metrics, and scrollable wallet viewport |
| P1-03 | Major | Remove-wallet card text still uses raw `0.86rem` and `0.7rem` overrides | Map title and metadata to wallet-card LUT rows |
| P1-04 | Major | Current Appearance offers accent swatches, not semantic palette presets | Add default plus four complete, validated palette presets and custom editing |
| P1-05 | Major | Asset `Value` and `Price` are fabricated demo values; current wallet RPC supplies balance/metadata but no fiat market feed | Label them indicative with source/time or hide them until a market-data capability exists |
| P1-06 | Major | Tabs and compact navigation are visually dense at the current control size | Raise control/row sizing and preserve 44 px minimum hit targets without increasing bar height arbitrarily |
| P1-07 | Major | Top bar and status surfaces use translucency/blur inconsistently; internal wallet tabs are already opaque | Keep wallet tabs opaque; document intentional overlay surfaces or make application chrome opaque for consistency |
| P2-01 | Minor | General UI and the Z00Z lockup need distinct weight boundaries | Keep interface text at 400–700; reserve the public-site-matched 780 weight for the wordmark only, with the selected desktop-wallet address at normal 400 |
| P2-02 | Minor | Tooltips, status badges, and helper labels use several independent sizes and tracking values | Map them to shared tooltip/status/label components |
| P2-03 | Minor | Several prototype-only controls lack a visible capability status | Add Live, Preview, Unavailable, or Experimental states without relying on color alone |

## 🔑 Typography audit

### 🔑 Current font lookup table

| Token / source | Current family | Loaded weights | Current role | Audit decision |
| --- | --- | --- | --- | --- |
| `--font-sans` | Geist | 400, 500, 600, 700 | General UI | Keep; package locally; use only 400–700 |
| `--font-mono` | Geist Mono | 400, 500, 600, 700 | Addresses, IDs, data, but also excessive card prose | Keep; narrow its role to verbatim/technical/tabular data |
| `--font-logo` | Geist | variable 400/780 | Z00Z wordmark at 780 and selected desktop-wallet address at 400 | Keep for the topbar lockup pair only; never use in controls or wallet-card content |
| Browser/system fallback | `sans-serif`, `ui-monospace`, local fallback faces | Platform dependent | Load failure only | Accept only as fallback, never an intentional component family |

There are no per-wallet font rules. `renderWalletShell()` maps every wallet through the same `.wallet-nav-item`, `.wallet-nav-copy strong`, and `.wallet-nav-copy small` selectors. Wallet names and explanatory copy use Geist; only the literal summary amount is wrapped in Geist Mono. Different wallet names and values only expose truncation, density, and contrast differences already present in the shared component.

### 📏 Current size implementation

| Measurement | Current result | Consequence |
| --- | ---: | --- |
| Total `font-size` declarations | 113 | Too many local type decisions for a tokenized system |
| Token-backed declarations | 75 | Majority is tokenized but not enough for consistency |
| Raw declarations | 38 | Component-level drift remains substantial |
| Smallest repeated raw size | `0.7rem` / 11.2 px | Below the 12 px specification floor and too small for requested density |
| Other repeated raw sizes | `0.75rem`, `0.76rem`, `0.78rem`, `0.8rem`, `0.82rem`, `0.84rem`, `0.86rem`, `0.88rem` | Near-identical roles render with visibly different metrics |
| Raw relative `.mono` size | `0.88em` | Technical text changes size according to parent rather than semantic role |

### 📏 Recommended typography LUT amendment

The specification LUT is structurally correct. Before implementation, revise the small rows below; unchanged display rows remain as specified. This is a controlled spec amendment, not an undocumented CSS override.

| ID / token | Family | Weight | Desktop | Mobile | Line height | Use |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `TYPE-01` `--type-balance` | Geist Mono | 700 | 35.2–50.4 px fluid | 35.2 px minimum | 1.00 | Primary available balance only |
| `TYPE-02` `--type-address` | Geist | 400 | 25 px | 20 px compact top bar | 1.04 | Selected desktop-wallet address; paired with a 13 px / 16 px wallet-name line to match Copy-control height |
| `TYPE-03` `--type-page-title` | Geist | 700 | 28 px | 22 px | 1.20 | Standalone page title |
| `TYPE-04` `--type-page-section` | Geist | 700 | 23.2 px | 20 px | 1.20 | Major page section |
| `TYPE-05` `--type-section` | Geist | 700 | 20 px | 18 px | 1.20 | Panel/dialog section |
| `TYPE-06` `--type-card-title` | Geist | 700 | 16 px | 16 px | 1.25 | Card title/action |
| `TYPE-07` `--type-row-title` | Geist | 700 | **16 px** | **16 px** | 1.30 | Wallet, asset, history, dialog-row title |
| `TYPE-08` `--type-control` | Geist | 600 | **15 px** | **15 px** | 1.20 | Buttons, inputs, compact controls |
| `TYPE-18` `--type-nav` | Geist | 600/700 | **16 px** | **16 px** | 1.20 | High-visibility wallet tabs, sidebar navigation, and `WALLETS` label |
| `TYPE-09` `--type-body` | Geist | 400 | 16 px | 16 px | 1.50 | Explanatory prose |
| `TYPE-10` `--type-support` | Geist | 500 | **15 px** | **15 px** | 1.45 | Helper and card descriptions |
| `TYPE-11` `--type-metric` | Geist Mono | 700 | 21.6 px | 21.6 px | 1.15 | Compact total/metric |
| `TYPE-12` `--type-data-key` | Geist Mono | 700 | **15 px** | **15 px** | 1.25 | Aligned amount, value, price |
| `TYPE-13` `--type-data-meta` | Geist Mono | 500 | **14 px** | **14 px** | 1.35 | Technical secondary data only |
| `TYPE-14` `--type-label` | Geist | 700 | **13 px** | **13 px** | 1.25 | Column/field/eyebrow label |
| `TYPE-15` `--type-status` | Geist | 600 | **13 px** | **13 px** | 1.25 | Human-readable badge/status label |
| `TYPE-16` `--type-id` | Geist Mono | 500 | **14 px** | **14 px** | 1.35 | IDs, hashes, YAML, full-address tooltip |
| `TYPE-17` `--type-brand` | Geist | 780 | 34 px max; fluid | 26 px | 1.00 | Z00Z wordmark only |

Implementation rules:

1. Human-readable status text uses Geist; only a verbatim status code uses Geist Mono.
2. Wallet card line one is `TYPE-07`; line two is either Geist `TYPE-10` for prose or Mono `TYPE-13` for an address/amount.
3. All numeric columns use `font-variant-numeric: tabular-nums`.
4. `Name`, `Balance`, `Value`, and `Price` headers align to their own data-column start/end edges; icon geometry is not the `Name` anchor.
5. No visible text is smaller than 13 px after the amendment, except an explicitly nonessential build/version label at 12 px.
6. Mobile retains control, row, metadata, and label sizes; it reflows instead of shrinking them.
7. The desktop sidebar brand row and sticky top bar share the same 80 px frame, keeping the logo, wordmark, selected address, and copy control on one horizontal centerline.
8. Active, selected, hover, and disabled states never change family, size, weight, or layout metrics.
9. Every exception requires an explicit LUT row or documented component mapping in the specification.

## ⚙️ Component and interaction consistency

### ⚙️ Cancel and secondary actions

| Surface | Current class | Current border | Required class/behavior |
| --- | --- | --- | --- |
| Add Wallet | `.button.button-quiet` | Transparent | Neutral bordered secondary button |
| Send dialog | `.button.button-quiet` | Transparent | Neutral bordered secondary button |
| Receive dialog | `.button.button-quiet` | Transparent | Neutral bordered secondary button |
| Asset Claim dialog | `.button.button-quiet` | Transparent | Neutral bordered secondary button |
| Permission dialog | `.button.button-quiet` | Transparent | Neutral bordered secondary button |
| Remove Wallet dialog | `.button.button-quiet` | Transparent | Neutral bordered secondary button beside danger action |
| Open Existing Wallet | `.button` | 1 px | Use as the reference behavior |
| Icon-only dialog close | Icon button | 1 px | Keep icon treatment; accessible name required |

All textual `Cancel` controls must share the same 44 px minimum height, 1 px neutral border, control radius, horizontal padding, Geist `TYPE-08`, hover border, pressed state, disabled state, and protected focus ring. Cancel never becomes gold or danger-colored. `Back` is navigation and may remain distinct, but it must not be used where the action actually cancels a flow.

### ⚙️ Wallet rail

- Keep the left rail organized as a single three-row wallet placeholder, a compact Network telemetry group (OnionNet, Reticulum, Aggregators), and fixed utility actions below it. Wallet cards, Add, and Remove are direct children of one ordered scroll list; they scroll together. Network shortcuts open read-only telemetry workspaces, never Settings.
- The left rail has exactly one active destination at a time: wallet, Network shortcut, or Settings. Selecting one clears the active state and `aria-current` from the other rail groups; inner wallet tabs and the Settings context rail remain separately scoped navigation.
- Wallet cards use one height/padding/icon/text grid. Long names truncate only after preserving the amount/status line.
- The active wallet uses the same bottom gold indicator grammar as the internal active tab, not an arbitrary full-height side stripe.
- `Add wallet` and `Remove wallet` remain inside the wallet placeholder after the scrollable wallet cards. They settle at its lower edge for a short or empty list and scroll with a longer wallet list. `Settings` and `Log out` remain application-level utility actions outside that placeholder. Remove stays visible but disabled when the wallet list is empty.
- Zero wallets is a valid state. Removing all local profiles is allowed in the concept; the UI must then show Add Wallet without retaining a stale selected-wallet route.
- Selection for removal uses true 20 × 20 px square checkboxes, a compact 3 px radius, and selected card border/background derived from danger semantics. The checkbox itself must not stretch with the row.
- The full row remains the checkbox label, keyboard accessible, and visibly focused.

### ⚙️ Navigation and chrome

- Internal wallet tabs begin at the main content edge, not at the application rail edge.
- Tabs remain sticky below the top bar and fully opaque. Current `.wallet-tabs` already uses `var(--bg-canvas)` without transparency; preserve this behavior.
- Increase tab label size through `TYPE-08`; do not add per-tab font overrides.
- Wallet tabs begin with `Assets`: `History`, `Swap`, `Exchange`, `Stacking`, `Backup`, and `Settings` follow. `Overview` is never a wallet tab; the app-level Home may present a selected-wallet snapshot without duplicating a second wallet route. `Send` and `Receive` are asset-agnostic actions available from all supported assets.
- `Exchange` must be disabled/target-labelled until an exchange contract exists. `Swap` and `Stacking` must disclose compatibility/noncanonical status until their confirmed lifecycle is canonical.
- The top address and its single Copy button remain together. Route/network telemetry is a non-interactive item in the selected-wallet status bar, not a top-bar action.
- Copy uses the ordinary top-bar icon hover state. Its tooltip uses the shared tooltip surface and shows the full address; the button does not become gold.
- Fixed desktop/mobile status bars must reserve layout space and safe-area insets. They may not cover lists, buttons, or settings rows.

### ⚙️ Buttons, cards, tooltips, and forms

- Primary gold is authorization/primary action, not a generic hover color.
- Neutral secondary buttons share one width-by-content pattern; only form-layout constraints may equalize a small button group.
- Destructive selection uses danger tint only after selection; the neutral card remains quiet before selection.
- Clickable cards use buttons/links and the shared hover/focus treatment. Static cards do not gain hover borders.
- Tooltip default is the same dark raised surface used by other top-bar controls, with `TYPE-16` for a full address. No separate blue strip or saturated yellow tooltip is allowed.
- Form labels and helper text must not use raw local font sizes. Inputs and selects use `TYPE-08` and 44 px minimum height.
- Every async action needs default, loading, success, empty, validation-error, service-error, and retry behavior where applicable.

## 🖼️ Legacy reference analysis

### 🖼️ Zano screenshots

| Reference | Relevant pattern | Decision for Z00Z |
| --- | --- | --- |
| `zano pics/1.png` | Multi-wallet rail; top wallet address plus copy; Assets columns Name/Balance/Value/Price | Retain structure; Value/Price require a named market source and timestamp |
| `zano pics/2.png` | Wallet-scoped History table | Retain wallet ownership and column discipline; use Z00Z lifecycle states |
| `zano pics/3.png` | Receive QR and address | Adapt to receiver card/payment request; never imply a permanent public address is the only receive model |
| `zano pics/4.png` | Create, open, restore choices | Retain compact centered Add Wallet choice flow and bordered Cancel |
| `zano pics/5.png` | General settings layout and visible Secret field | Retain row alignment; **reject visible secret field** |
| `zano pics/6.png` | Password/security settings | Retain app lock and authenticated secret actions |
| `zano pics/7.png` | Asset name, ticker, owner, asset ID, current/max supply | Retain fields when supplied by metadata/details RPC; show unknown explicitly |
| `zano pics/10.png` | Hide sensitive balance | Retain as an app/session privacy control |
| `zano pics/11.png` | External withdrawal confirmation | Use only as a review-density reference; do not copy provider semantics |
| `zano pics/12.png` | Full-address copy tooltip | Retain with shared neutral tooltip styling |

Do not copy the legacy giant empty panels, bright blue primary actions, tiny low-contrast labels, always-visible secrets, or wallet-card staking switches without capability evidence.

### 🖼️ Guarda screenshots

| Reference | Relevant pattern | Decision for Z00Z |
| --- | --- | --- |
| `guarda pics/1.png` | Asset selector, asset detail, Receive QR, primary actions | Retain selection/detail relationship and receive clarity |
| `guarda pics/2.png` | Staged Send flow | Retain entry → fee/authority review → confirmation → result |
| `guarda pics/3.png` | Portfolio allocation chart | Optional target only after trustworthy pricing and accessible table fallback |
| `guarda pics/4.png` | Buy/Sell provider | Reject until explicit provider, jurisdiction, privacy, custody, and failure contracts exist |
| `guarda pics/5.png` | Exchange provider flow | Target only; current dispatcher has no exchange method |
| `guarda pics/6.png` | Backup management | Retain history, destination, encryption, integrity, and create/restore actions |
| `guarda pics/7.png` | Restore backup | Retain validation before replacement and explicit recovery consequence |
| `guarda pics/8.png` | Change password | Retain only with re-authentication, password policy, and session invalidation behavior |

Do not import custodial account language, loans/earn promotion, crowded market lists, or fiat portfolio assumptions into the private wallet.

### 🖼️ ASCII mockups

| Source family | Reusable behavior | Capability decision |
| --- | --- | --- |
| `app.*` | Lock, settings, sanitized logs | Lock is live; settings/config and log viewer need explicit services; Log out means end local session unless a host process API exists |
| `wallet.*` | List/create/open/import/export/recover/lock/unlock/show seed | Mostly mapped to current routes; seed/export actions remain authenticated and privacy-gated |
| `wallet.remove` / `wallet.delete` | Multi-select and destructive confirmation | Split local detach from permanent deletion; never alias them silently |
| `asset.*` | List/detail/balance/metadata/send/receive plus staged review | List/detail/balance/metadata are relevant; send/receive use canonical tx/receiver flows; advanced operations remain gated |
| `asset.swap`, `staking`, `merge`, `split` | Preview, fee, confirmation, result | Compatibility/experimental until canonical mutation and lifecycle authority are proven |
| `tx.*` | Build, verify, fee, send, broadcast, import/export, history, pending, details | Strong fit; expose exact lifecycle rather than generic Pending/Success |
| `backup.*` | Create, list, configure, validate, restore | Strong fit; distinguish seed recovery from full local-state recovery |
| `key.*` | Receiver derivation/card and public export | Strong fit; never expose secret key material in ordinary settings |
| `network.*` | Chain selection, Tor/OnionNet, privacy status | Chain scan/switch is available; OnionNet remains target/stub-labelled |
| `logging.*` | Level, destination, redacted diagnostics | Target settings; sanitize paths, tokens, receiver material, and secrets |
| `notifications.*` | App notification preference | Target; no registered settings/notification RPC exists |

General ASCII patterns to retain are safe defaults, preview before commitment, explicit fee/authority review, bounded inputs, clear empty/loading/error states, backup verification, and exact irreversible-action copy. Hardware signers, biometrics, watch-only mode, provider trading, explorer links, and automatic cloud backup remain out of scope until code and threat-model evidence exists.

## 🎨 Appearance and palette system

The current palette is approved and remains the default. No implementation task may alter its visual character while adding configurability.

### 🎨 Preset source colors

| Preset ID | Source colors | Contrast note | Decision |
| --- | --- | --- | --- |
| `z00z-default` | Existing demo semantic tokens | Approved baseline | Default and Reset target |
| `black-gold-elegance` | `#000000`, `#14213D`, `#FCA311`, `#E5E5E5`, `#FFFFFF` | Gold on navy is approximately 7.90:1 | Eligible |
| `moonlit-stroll` | `#004955`, `#105E60`, `#6B7D7F`, `#14365C`, `#10284E` | The source set is intentionally teal and navy; Z00Z amber remains the accessible action accent | Eligible |
| `walking-at-night` | `#7B6D62`, `#423A37`, `#0E191F`, `#2B3C43`, `#597276` | Warm stone and blue-charcoal surfaces retain a separate amber action colour | Eligible |

Each preset must map these semantic tokens: canvas, sidebar, surface, raised, primary text, secondary text, tertiary text, border, strong border, brand, brand-strong, brand-ink, privacy rail, success, warning, danger, and focus. Source colors are inspiration/input, not permission to weaken semantic contrast.

Required behavior:

1. Theme mode (`system`, `dark`, `light`) and palette preset are independent settings.
2. Preset cards show five source swatches and a small live component preview.
3. Selection previews immediately but requires Apply or reverts on Cancel.
4. Reset restores `z00z-default` without touching nonappearance settings.
5. Custom palette editing validates every required text/surface, focus, control-boundary, and state pair before Apply.
6. Success, warning, danger, focus, privacy rail, testnet/devnet, and quarantine meanings remain stable across presets.
7. State is never communicated by color alone.
8. Text scale, reduced motion, and compact density stay separate from palette.
9. Production packages fonts and palettes locally; no remote dependency is required at runtime.

## 🧭 App and wallet information architecture

| Scope | Owns | Must not own |
| --- | --- | --- |
| Application | Wallet list, active wallet selection, language, notifications, appearance, global app lock/session controls, sanitized diagnostics, network/runtime status, effective configuration | Per-wallet balances, History, Backup inventory, asset/object records |
| Wallet | Assets, Vouchers, Permissions, History, Swap, Exchange target, Stacking compatibility, Backup, per-wallet Settings, receive identities/payment requests, wallet security and transaction defaults | Other wallets' activity or balances |
| Asset | Name, ticker, class, balance, available/pending amounts, supply, owner/issuer, asset ID, policy/trust metadata, send/receive entry | Global wallet history or unrelated asset operations |
| Receiver/payment request | Receiver card, scoped request, expiry, amount/metadata when present, local acceptance/publication/settlement state | Claim of final settlement before confirmed evidence |

Navigation contract:

- The app rail contains a three-card wallet placeholder where wallet cards and the Add/Remove footer scroll together; Settings remains a separate app action.
- A selected wallet owns its inner tab bar and all displayed balances/activity.
- Rename Activity to History everywhere, including headings, links, empty states, and accessibility names.
- Send and Receive apply to every supported asset; asset detail may preselect an asset but does not create separate asset-specific send semantics.
- Backup precedes per-wallet Settings; Exchange remains visibly unavailable until supported.
- Wallet Settings uses its own local context rail: General, Security, Backup, Policies, and Advanced. It is selected-wallet scoped and must never mutate app settings or another wallet profile.
- Vouchers and Permissions remain independently discoverable through Assets-local navigation at narrow widths.

## 🔍 Capability map from current code

Status definitions:

- **LIVE**: registered current route plus an implementation appropriate to connect to the UI.
- **COMPAT**: registered route exists, but comments/behavior mark it noncanonical, preview, stub, or compatibility-only.
- **TARGET**: desired by docs/mockups but absent from the current UI service contract.
- **REJECT**: unsafe or misleading legacy behavior that must not enter the product.

| Area | Status | Current evidence | UI consequence |
| --- | --- | --- | --- |
| List/create/open/import/recover wallets | LIVE | `app.wallet.*` dispatcher routes | Implement staged local wallet flows |
| Permanent delete wallet | LIVE | `app.wallet.delete_wallet` requires password | Use explicit permanent-delete semantics, not ambiguous Remove |
| Safe local profile detach | TARGET | No distinct route | Keep concept-only or define API/storage semantics |
| Remove all local profiles | UI-valid | Demo supports empty wallet list | Must not imply underlying wallet files were permanently deleted |
| Wallet lock/unlock/lifecycle | LIVE | Session and lifecycle routes | Selected-wallet Security can expose lock timeout and Lock now when configured |
| Wallet rename | TARGET runtime / concept-local demo | No registered route | Re-authenticated local demo may show the intended flow, but labels it local-only until a durable rename capability exists |
| Asset list/balance/details/metadata | LIVE | Registered asset routes and typed responses | Populate Assets and detail fields from RPC |
| Fiat Value/Price | TARGET | No market-data response/provider | Hide or label indicative with provider and timestamp |
| Canonical Send | LIVE through transaction flow | Build/fee/verify/send/broadcast/reconcile routes | Prefer tx lifecycle over convenience asset mutation |
| Receive/request | LIVE | Receiver card and payment-request routes | Use QR/card/request and exact local/final states |
| Swap/Stacking/Merge/Split | COMPAT | Registered but described as compatibility/noncanonical round trips | Disabled/experimental labels; never claim confirmed settlement authority |
| Exchange | TARGET | No exchange route/provider | Disabled or removed from release navigation |
| History/pending/details | LIVE | Transaction history, pending, details, reconcile routes | Map exact lifecycle states and receipts |
| Vouchers/Rights | LIVE for listed object actions | Object list/action/package routes | Separate value from authority; show manual review/quarantine |
| Asset Claim intake | TARGET gap | No dedicated claim intake/build route | Keep target-labelled despite lower-level claim support |
| Backup create/list/configure/restore | LIVE with caveats | Registered backup routes | Show encryption/integrity/destination and recovery scope |
| Seed display/key rotation/public export | LIVE, sensitive | Authenticated key/session routes | Require re-authentication and private-display warnings |
| Local chain scan/switch | LIVE | Scan and mainnet/testnet/devnet routes | Show wallet-local scan status, not generic global sync |
| Tor switch | Partial | Registered control | Show current capability and failure states exactly |
| OnionNet telemetry/routing | COMPAT/TARGET | Switch is unsuccessful/stub; crate is stub | No fabricated route telemetry; target label required |
| Language/notifications persistence | TARGET | No settings/notification CRUD route | Form may be prototype-only until service exists |
| App exit/log out process | TARGET | No app exit route | Define Log out as session end/lock unless host shell owns exit |
| UI theme/palette persistence | TARGET | egui `ui_themes` is `None` in stub config | Demo-only until real configuration seam exists |
| UI↔YAML get/apply/watch/revision | TARGET | Runtime reads/merges YAML only | Advanced editor stays nonactionable until service delivery |
| Visible secret in General settings | REJECT | Violates wallet-local secret handling | Never include |

The dispatcher currently registers 75 unique methods. That number proves route breadth, not product maturity. Capability labels must be based on implementation semantics and tests, not method count.

## 🔄 UI and YAML bidirectional contract

### 🔄 Current state

- `YamlConfig` reads a bounded YAML document with a 256 KiB maximum and exposes scalar dot-path lookup.
- Runtime wallet configuration merges an embedded default with an optional file override and serializes the merged value for reading.
- Environment values can override selected settings and therefore need visible provenance/read-only treatment in the UI.
- The wallet service does not expose config get, validate, apply, watch, revision, rollback, or conflict RPCs.
- Serialization through YAML values is not a round-trip editor contract: comments, presentation, and stable source ordering are not guaranteed.
- The current wallet YAML contains machine-specific absolute paths. Those must be normalized, redacted, or represented through safe path pickers before any UI exposure.
- The egui configuration seam has `ui_themes: None`; it is not a persistence implementation.

### 🔄 Target effective configuration model

The editor must show real effective configuration, not a decorative sample. App and wallet scopes must be explicit. The following is a target schema shape, not a claim that the current runtime accepts it:

```yaml
schema_version: 1

app:
  general:
    language: en
    notifications: true
  appearance:
    theme: dark
    palette: z00z-default
    text_scale: 1.0
    reduced_motion: system
    compact_desktop_lists: false
  security:
    lock_after_minutes: 15
  diagnostics:
    level: info

wallets:
  wallet-id:
    display:
      currency: Z00Z
    transactions:
      default_fee: "0.001"
    security:
      lock_after_minutes: 15
    backup:
      auto_backup: false
      interval_hours: 24
      encrypt: true
    network:
      chain: devnet
      tor: false
```

No password, seed phrase, private key, session token, receiver secret, raw recovery material, or unredacted diagnostic credential may be represented in this YAML.

### 🔄 Required synchronization behavior

1. Service returns `source_yaml`, `effective_yaml`, `schema_version`, `revision`, per-path provenance, editable paths, read-only paths, and validation diagnostics.
2. Form controls and YAML editor bind to the same typed draft model; neither edits the live file directly.
3. Every UI edit updates the YAML draft and Diff view immediately.
4. Every valid YAML edit updates corresponding form controls immediately; invalid YAML preserves user text and shows line/column diagnostics without mutating live state.
5. Apply uses compare-and-swap on `revision`, validates schema and semantic constraints, writes atomically with private permissions, reloads, and returns the new effective configuration.
6. External file change creates a revision conflict. The UI offers Reload, Compare, or Keep draft; it never overwrites silently.
7. Last-known-good content and rollback metadata are retained without storing secrets.
8. Unknown safe keys and comments must be preserved. If the chosen YAML library cannot guarantee this, the service must use a round-trip-capable document model or explicitly restrict editing to generated managed sections.
9. Environment- or policy-owned values show their provenance and are read-only in forms/YAML.
10. Apply reports which changes are immediate, require wallet reopen, require rescan, or require application restart.
11. App settings and selected-wallet settings can be filtered independently, while Effective shows their resolved combination.
12. Import/export validates size, schema, forbidden keys, paths, and secrets before preview; it never applies on file selection alone.

## ⚠️ Z00Z domain requirements from official documentation

- Assets are spendable/final value, Vouchers are conditional value, and Rights are authority. They require separate labels, balances, actions, and empty states.
- Unknown or unsupported policy objects enter quarantine/manual review rather than a generic asset balance.
- Receiver flow distinguishes Pending, Accepted locally, Quarantined, Published, Settled, and Rejected/refunded. A local acceptance is not final settlement.
- A PaymentRequest is a scoped receive intent, not a permanent public address and not proof of settlement.
- Keys, openings, scan state, imported packages, receiver history, local labels, disclosures, and backups are wallet-local possessions.
- Network state cannot reconstruct local secrets, memos, or history it never received. Backup copy must state whether recovery is seed-only or full local-state recovery.
- RPC/publication evidence is not settlement authority. The UI must not collapse exported, submitted, admitted, and confirmed into one generic success state.
- Public status/explorer surfaces must not expose wallet labels, receivers, private route paths, or other wallet-local metadata.
- OnionNet remains target architecture until its blockers and verification gates are complete. The wallet may show safe aggregate status, not invented path telemetry.
- Wallet configuration does not confer protocol or genesis authority. Protocol rules remain locked and cannot be overridden by UI/YAML preferences.

## 📡 Offline-first renderer audit — 2026-07-23

The present prototype is self-contained at the renderer boundary: `index.html` loads local runtime scripts and one local CSS entry point; CSS, Geist fonts, locale catalogues, icons, and object imagery are vendored under `demo/`. Its production-readiness check rejects remote resource URLs and direct browser network APIs in renderer runtime files. SVG XML namespaces such as `http://www.w3.org/2000/svg` are document metadata, not browser network requests.

This confirms that the interface can be packaged without an Internet dependency. It does **not** claim a live Reticulum radio stack: radio connectivity remains a target native-Rust transport and requires its own adapter, carrier-state contract, and air-gapped device verification.

| Area | Evidence in prototype | Production rule |
| --- | --- | --- |
| Bootstrap and styles | Local `index.html`, `styles.css`, and CSS imports only | Bundle all renderer resources in Tauri; deny remote origins |
| Typography and imagery | Vendored Geist `.woff2`, inline SVG sprite, and local asset files | No CDN fonts, icon fonts, remote images, or remote code themes |
| Language | Local locale registry and catalogues | Locale changes remain local; no automatic web translation |
| Renderer state | `WalletGateway` mock and static fixtures; no browser storage or generic RPC transport | Native typed gateway only; keys, sessions, and raw transport data stay outside the renderer |
| Optional connectivity | No renderer network API; Reticulum/OnionNet is capability-labelled | Native Rust owns radio/transport; unavailable radio shows an explicit local state |
| Prototype publishing | GitHub Pages can publish the visual demo | Pages is review-only, never a production wallet runtime |

### Required production safeguards

1. Package every renderer resource and verify the final Tauri bundle contains no remote URI dependency.
2. Launch each target in airplane mode before first unlock and after restart; local wallet state must remain usable.
3. Treat radio loss as a typed native capability state, not as a renderer exception or an Internet fallback trigger.
4. Show freshness and queue status for transport-derived data; preserve the difference between local intent, submitted, and settled.
5. Keep diagnostic telemetry opt-in, local, redacted, and bounded. It must not become an analytics/network dependency.

## ✅ Current strengths to preserve

- The current dark/gold/blue visual direction is distinctive, calm, and appropriate for a privacy wallet.
- `logo-31-bg.png` is already used consistently in favicon, desktop rail, lock surface, and mobile brand.
- Wallet-scoped tabs, wallet-owned data, History naming, Add/Remove placement, full-address tooltip, and asset column alignment are represented in the current demo.
- Internal wallet navigation is sticky and opaque.
- Add Wallet primary choices use the existing Z00Z palette rather than saturated legacy blue.
- Remove Wallet now supports selecting all profiles and reaching a valid zero-wallet state.
- Remove-wallet selection cards use danger color and checkboxes are explicitly constrained to 20 × 20 px.
- The top Copy button uses normal panel interaction styling rather than turning gold.
- The demo visibly labels several target-only configuration/network concepts instead of silently claiming backend completion.

## ♿ Accessibility and responsive requirements

1. Every control must have an accessible name; icon-only buttons need tooltips plus `aria-label`.
2. All pointer targets are at least 44 × 44 px; primary mobile actions may be 48–52 px.
3. Keyboard focus is a protected 3 px ring with sufficient contrast and is never hidden behind sticky/fixed bars.
4. Tabs use correct tab semantics or navigation-link current semantics consistently, including arrow-key behavior if implemented as ARIA tabs.
5. Remove-wallet selection uses native checkbox semantics, `fieldset`/`legend`, selected count, and non-color selection state.
6. Tooltips are supplemental. Copy and privacy actions remain understandable and usable without hover.
7. At 200% zoom, no page-level horizontal scrolling is required for ordinary tasks and no action is clipped.
8. At 390 × 844, fixed status bars, bottom navigation, safe areas, and software keyboard do not cover content.
9. Reduced-motion preference disables nonessential transitions; state changes remain visible without motion.
10. Palette presets pass WCAG AA for normal text and controls; focus, warning, danger, success, and selected states are tested independently.
11. Tables transform into labelled rows/cards on narrow screens without losing the relationship between header and value.
12. Sensitive amounts remain masked in accessible names when hidden; screen readers must not receive the secret value.

## 🛠️ Implementation backlog

### 🚨 P0: correctness and honesty

- [x] Replace the former three-position segmented control with one Dark/Light toggle and cover the default and transition with DOM and screenshot checks.
- [x] Amend the normative typography LUT using the reviewed sizes, then remove raw component sizes below the amended floor.
- [x] Audit every `font-family` and `.mono` use; keep ordinary language in Geist.
- [x] Resolve mobile fixed-bar/status overlap at 390 × 844 with long content; 200% zoom remains a production acceptance check.
- [x] Relabel the demo operation as removing local concept profiles and state that wallet files are not deleted; a production detach API remains P2.
- [x] Capability-gate YAML behavior: the demo applies only browser-local state and discloses the absent production configuration service.
- [x] Mark market `Value` and `Price` unavailable until a provider/source/time contract exists.
- [x] Lock the prototype renderer to local assets and no direct browser-network APIs; record the equivalent Tauri packaged-resource and airplane-mode gate.

### ⚙️ P1: component standardization

- [x] Replace every textual borderless `Cancel` with the shared neutral bordered secondary button.
- [x] Normalize wallet card, remove-wallet card, asset row, history row, settings row, dialog, tooltip, status, and tab typography to the LUT.
- [x] Preserve the square 20 × 20 px checkbox and danger-selected card treatment in the executable states.
- [x] Enforce one button geometry, one secondary style, one destructive style, one icon-button style, and one tooltip style.
- [x] Verify the wallet-list scroll container with generated long content while utility actions remain reachable; 0/1/3 live demo states are smoke-tested.
- [x] Verify `Name`, `Balance`, `Value`, and `Price` column/header alignment in the desktop demo.
- [x] Keep wallet tabs sticky, opaque, and aligned to the main content edge.
- [x] Add the five semantic palette presets and contrast-gated protected custom accent validation.
- [x] Separate app settings from selected-wallet settings in the local YAML/Form/Mapping concept state.
- [x] Replace generic lifecycle copy with exact receiver, transaction, object, scan, and backup states where the demo has an authoritative concept state.

### 🔔 P2: service-backed completion

- [ ] Add configuration get/schema/validate/apply/watch/revision/rollback/provenance RPCs.
- [ ] Add language and notification preference persistence or remove the controls from production builds.
- [ ] Add a safe local wallet-profile detach contract if Remove Wallet must not delete wallet files.
- [ ] Add wallet rename only with a registered persistence contract.
- [ ] Add a market-data interface before enabling fiat value/price and portfolio charts.
- [ ] Add exchange only after provider, privacy, custody, jurisdiction, quote-expiry, fee, and failure contracts exist.
- [ ] Promote Swap/Stacking only after canonical transaction/lifecycle authority and tests exist.
- [ ] Add sanitized diagnostics with redaction, bounded export, and no private route/receiver leakage.

## ✅ Acceptance criteria

| Area | Acceptance gate |
| --- | --- |
| Typography | Computed font family/size/weight for every visible text node maps to one LUT row; no wallet-specific divergence and no unsupported raw override |
| Readability | No ordinary visible text below 13 px; body remains 16 px; 200% zoom has no clipping |
| Wallet rail | 0–30 wallets work; list scrolls independently; Settings/Add/Remove remain reachable; selected indicator matches tab grammar; exactly one wallet/network/Settings rail destination is active |
| Cancel | Every textual Cancel is the same neutral bordered secondary control in default/hover/focus/pressed/disabled states |
| Checkboxes | Computed box is exactly 20 × 20 px, square at all viewports, keyboard focus visible, selected card uses danger semantics |
| Tabs | Start at main-content edge, remain sticky/opaque, do not move on content scroll, and retain readable labels |
| Copy | One Copy button beside address; ordinary panel hover; shared tooltip shows full address; no blue strip or gold button hover |
| Assets | Name/Balance/Value/Price headers align with their data; value/price disclose source/time or remain unavailable |
| Appearance | Default palette is unchanged; four optional presets update full semantic tokens and four application-wide code themes change YAML syntax only |
| YAML | UI and YAML reflect the same typed draft; syntax highlighting remains an accessible overlay above the editable source; validation, provenance, revision conflicts, atomic apply, rollback, and secret exclusion are enforced |
| Capability honesty | Every nonlive function is disabled or labelled Preview/Unavailable/Experimental; telemetry is read-only, local, freshness-labelled, and never fabricated |
| Offline-first | Airplane-mode launch and navigation use bundled resources and local wallet state only; radio loss reports a typed capability state without Internet fallback |
| Responsive | 390 × 844 and 1440 × 1000 screenshots show no overlap, clipping, transparent wallet tabs, or unreachable actions |

## 🔍 Verification plan

1. Run HTML, CSS, and JavaScript validation before screenshots.
2. Add a DOM typography audit that records computed family, size, weight, line height, and matching LUT token for every visible text node.
3. Add component-state screenshot cases for default, hover, focus-visible, selected/current, disabled, loading, error, empty, and long-content states.
4. Capture desktop and mobile views for Home, Assets, asset detail, History, Add Wallet, Remove Wallet with all selected, Appearance, Advanced YAML, Security, and Backup.
5. Test wallet rail with 0, 1, 3, 10, and 30 wallets and names/addresses at maximum supported length.
6. Test themes and all palettes with automated contrast checks, forced colors, 200% zoom, reduced motion, and hidden sensitive values.
7. Test keyboard-only and screen-reader flows for wallet switching, copy address, tabs, Add Wallet, Remove Wallet, dialogs, and config validation.
8. When backend integration begins, contract-test each visible function against the exact registered method and response type; registered-but-compatibility routes remain gated.
9. Run the packaged Tauri app in airplane mode before unlock, after unlock, and after restart. Assert no remote resource request, all local renderer assets load, radio loss is explicit, and queued/submitted/settled states remain distinct.

## 🔗 Evidence sources

### 📁 Local sources

- `UI-UX-SPEC.md`
- `demo/index.html`, `demo/app.js`, and `demo/styles.css`
- `old-sources/ascii-mockups/`
- `old-sources/guarda pics/`
- `old-sources/zano pics/`
- `colors/Black & Gold Elegance.png`
- [`Color Meanings: Moonlit Stroll and Walking at Night`](https://www.color-meanings.com/dark-color-palettes/)
- `crates/z00z_wallets/src/rpc/app_dispatcher_wiring.rs`
- `crates/z00z_wallets/src/rpc/wallet_dispatcher_routes.rs`
- `crates/z00z_wallets/src/services/wallet_runtime_config.rs`
- `crates/z00z_wallets/src/config/wallet_config.yaml`
- `crates/z00z_wallets/src/rpc/wallet_types.rs`
- `crates/z00z_utils/src/config/yaml.rs`
- `crates/z00z_utils/src/config/layered.rs`

### 🌐 Official Z00Z documentation

- [Z00Z documentation](https://www.z00z.io/docs)
- [Live versus target](https://www.z00z.io/docs/learn/live-vs-target)
- [Private objects](https://www.z00z.io/docs/learn/private-objects)
- [Assets, vouchers, and rights](https://www.z00z.io/docs/protocol/assets-vouchers-rights)
- [Object lifecycle](https://www.z00z.io/docs/protocol/object-lifecycle)
- [Receiver flow](https://www.z00z.io/docs/protocol/receiver-flow)
- [Wallet-local possession](https://www.z00z.io/docs/protocol/wallet-local-possession)
- [Wallet developer documentation](https://www.z00z.io/docs/developers/wallet)
- [RPC developer documentation](https://www.z00z.io/docs/developers/rpc)
- [Configuration and genesis](https://www.z00z.io/docs/developers/configuration-genesis)
- [Payment requests](https://www.z00z.io/docs/developers/payment-requests)
- [WASM wallet](https://www.z00z.io/docs/developers/wasm-wallet)
- [Network status and explorer](https://www.z00z.io/docs/network/status-explorer)
- [OnionNet](https://www.z00z.io/docs/network/onionnet)
