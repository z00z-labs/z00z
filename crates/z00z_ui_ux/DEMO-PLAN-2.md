---
goal: Root-only accordion navigation with responsive workspace tabs and a protocol-honest Z00Z application demo prepared for a Tauri 2 and Leptos production port
version: 2.3
date_created: 2026-07-26
last_updated: 2026-07-26
owner: z00z UI/UX
status: 'Completed'
tags: [design, architecture, navigation, appearance, tauri, leptos, telemetry, dapps, messenger, accessibility, privacy]
---

<!-- markdownlint-disable MD013 -->

# Z00Z Application Demo Plan 2

![Status: Completed](https://img.shields.io/badge/status-Completed-brightgreen)

## Introduction

This plan replaces the current split navigation model—macro destinations in the
left sidebar and global route tabs in the topbar—with one canonical global
navigation tree. The same tree is rendered as a persistent left sidebar on
desktop and as a left drawer on mobile. Accordion toggles exist only on root
branches. The global tree exposes only first-level workspace/action leaves;
every deeper route is rendered inside its selected workspace.
Workspace-local navigation is intentionally responsive: desktop uses a visible
vertical context rail, while mobile and narrow tablet use a visible horizontal
tab row directly below the topbar.

The Z00Z logo is permanently visible in the topbar in every application state,
including onboarding, wallet creation/recovery, unlock, lock, empty-wallet,
authenticated, error, and Help states. Opening the mobile navigation does not
cover or replace that brand lockup. The standalone Help application owns the
same branded topbar in its separate surface.

This work is an information-architecture and interaction evolution, not a Z00Z
visual redesign. The current dark Z00Z Default palette, typography, spacing
rhythm, radii, surface hierarchy, object-family treatment, and restrained
gold/blue accents remain the visual baseline. Reference applications contribute
interaction patterns only. They do not replace the Z00Z shell or turn navigation
into a collection of coloured icon tiles.

Appearance exposes exactly two application palettes: **Z00Z Default**, the
current dark demo, and **Z00Z Corporate**, a light palette locally derived from
the live `z00z.io` Corporate theme. A palette owns its colour scheme; there is no
independent System/Dark/Light selector and therefore no hidden cross-product of
themes and palettes.

The executable browser demo remains a deterministic visual and interaction
specification backed by fabricated data. The production target remains a
packaged Tauri 2 application whose shared UI is rendered by Leptos CSR/WASM and
whose native Rust boundary owns secrets, storage, signing, lifecycle, local IPC,
configuration mutation, and all authoritative operations.

This document is ready to execute only after the review gates in Section 10
pass. It supersedes the navigation recommendation preserved in
`DEMO-PLAN-2.md.bak`; it does not supersede protocol, security, or wallet-domain
specifications.

## 1. Product Decisions, Requirements, and Constraints

### 1.1 Final design decision

Use a **unified left navigation tree** on desktop and mobile.

- Desktop renders the tree in a persistent 280 px sidebar below the topbar.
- Mobile and narrow tablet render the same tree in a modal left drawer below
  the topbar.
- Root branches are independently expandable. Several root branches may remain
  open.
- A first-level workspace leaf opens its default internal destination; it never
  expands as a nested accordion.
- Expanding or collapsing a branch never selects a route.
- A branch waits for an explicit leaf selection.
- Selecting a leaf changes the main view; on mobile it also closes the drawer.
- Closing one branch preserves every other expanded branch on desktop and
  mobile.
- Global route tabs are removed from the topbar.
- A selected workspace may expose local destinations through a desktop context
  rail and mobile top tabs. These controls are not global navigation and never
  require reopening the mobile drawer.

This model is preferred because the application is growing from a wallet demo
into a multi-workspace product. One route model is easier to learn, removes the
desktop/mobile navigation mismatch, keeps the current location visible, and
ports cleanly to typed Rust enums and one Leptos component tree.

### 1.2 Priority and scope

| Priority | Scope | Required result |
| --- | --- | --- |
| P0 | Shell, logo, route registry, desktop sidebar, mobile drawer | One canonical navigation model with no global route tabs |
| P1 | Existing Wallet and Telemetry routes | Existing demo behavior survives the shell migration |
| P2 | Watchers, Explorer, dApps, Messenger, Contacts | Mandatory, fully navigable roadmap-preview flows with deterministic states and honest boundaries |
| P3 | Help, all locales, screenshots, accessibility | Every route has local Help and responsive evidence |
| Future port | Tauri/Leptos architecture contract only | The pure-JS demo model is documented for a later typed native implementation |

P0 through P3 define and implement this pure-JS demo plan. The future-port
material is design guidance and acceptance criteria only: it does not authorize
creating Rust crates, adding Tauri/Leptos dependencies, or connecting the static
GitHub Pages demo to a wallet backend.

### 1.3 Brand and shell requirements

- **BRAND-001**: Show the canonical Z00Z gold logo and `Z00Z` wordmark in the
  topbar at every route and application state, including onboarding, wallet
  creation/recovery, unlock, lock, empty-wallet, authenticated, error, and Help
  states.
- **BRAND-002**: Keep the logo visible while the mobile drawer, sheet, dialog,
  notification panel, or Help launcher is open.
- **BRAND-003**: The standalone Help window MUST show its own Z00Z topbar; it
  must not rely on the wallet window behind it.
- **BRAND-004**: Use
  `demo/assets/logo/z00z-logo-gold-circle.png` as the canonical raster brand
  source and the existing local SVG/wordmark treatment for UI rendering.
- **SHELL-001**: The topbar contains brand, compact page context, and utilities;
  it contains no global route tabs.
- **SHELL-002**: Desktop utilities may contain privacy mode, notifications,
  wallet context, connection status, and lock/account actions.
- **SHELL-003**: Mobile utilities are reduced to Menu, the full compact Z00Z
  lockup, a truncating page title, and at most one priority action.
- **SHELL-004**: Never place a full receiver ID, seed, session token, private
  route, raw package, or detailed network diagnostics in the topbar.
- **SHELL-005**: Keep the existing calm dark/gold/blue direction. Do not convert
  the wallet into a trading terminal, cyberpunk dashboard, or promotional
  marketplace.
- **SHELL-006**: Treat the current demo as the component and visual baseline.
  Change geometry, typography, spacing, radii, elevation, or action hierarchy
  only where this plan explicitly requires navigation, responsive, accessibility,
  or Z00Z Corporate adaptation work.
- **SHELL-007**: Z00Z Corporate is a light translation of the same component
  system, not a separate branded skin. The canonical gold logo remains unchanged
  in both palettes.

### 1.4 Navigation requirements

- **NAV-001**: Define one immutable navigation registry used by desktop,
  mobile, breadcrumbs, Help coverage, route tests, and the future Rust port.
- **NAV-002**: The root order is Wallet, Telemetry, dApps, Messenger, Contacts,
  Data & Storage, Settings, Help, About, and Log out.
- **NAV-003**: Replace every user-facing `Network` root label with
  `Telemetry`; do not rename protocol components such as Reticulum or OnionNet.
- **NAV-004**: Use `dApps` and `Messenger` as canonical English labels.
- **NAV-005**: Limit the global navigation to two visible levels:
  root accordion → workspace/action leaf. Deeper routes belong to the selected
  workspace's local destination model. Only root nodes may be accordion toggles.
- **NAV-006**: Allow any number of root accordions to be expanded
  simultaneously.
- **NAV-007**: Collapsing a branch MUST NOT collapse sibling branches.
- **NAV-008**: A branch toggle MUST NOT navigate, change the selected route, or
  close the mobile drawer.
- **NAV-009**: A leaf selection MUST set exactly one `aria-current="page"`.
- **NAV-010**: Selecting a leaf opens all its ancestors. Browser/native Back
  restores the previous route without discarding unrelated expanded branches.
- **NAV-011**: If a user collapses the branch containing the active route, keep
  the active content and breadcrumb; reopening the branch restores the visible
  active leaf.
- **NAV-012**: Desktop and mobile use the same global-tree ordering, labels,
  icons, route IDs, capability state, and Help topic IDs. Workspace-local
  destinations use the same IDs and order but project as a vertical rail on
  desktop and horizontal tabs on mobile.
- **NAV-013**: On mobile, selecting a leaf closes the drawer and restores focus
  to the Menu trigger after a later drawer close.
- **NAV-014**: Escape, native Back, or backdrop click closes only the topmost
  overlay. It does not change the active route.
- **NAV-015**: Log out is an action, not a route. It is visually separated and
  never receives `aria-current`.
- **NAV-016**: Help is an external-window route with a visible
  `↗`/external-link icon and never masquerades as an in-app modal.
- **NAV-017**: Contacts is a leaf in this version. It becomes a branch only
  after at least two stable child routes exist.
- **NAV-018**: Pending work is represented by bounded badges and the History or
  Inbox screens, not by a second global navigation system.
- **NAV-019**: A workspace with local destinations MUST keep them visible:
  desktop renders a vertical context rail; mobile and narrow tablet render a
  horizontally scrollable top tab row below the branded topbar. Opening the
  global drawer is never required to switch among those local destinations.
- **NAV-020**: One shared workspace-local destination projection MUST implement
  this rule for every component. A workspace is always a first-level leaf below
  a root accordion; its default route and every deeper route are rendered only
  in the main-window context rail/top tabs and never recursively in the global
  tree.
- **NAV-021**: About is a direct in-app route immediately after Help. It uses a
  simple product identity surface rather than a card, shows the canonical Z00Z
  logo and application version, exposes Privacy Policy, Terms of Use, Z00Z
  website, and GitHub repository links, and keeps `Check for updates...` as the
  final control on the page.

### 1.5 Visual continuity, icon, and palette contract

- **VIS-001**: Freeze the current `z00z-default` dark semantic token values and
  representative screenshots before shell work. Navigation migration may move
  components, but it must not silently recolour or restyle existing content.
- **VIS-002**: Reuse the current type scale, spacing scale, radii, borders,
  surfaces, controls, cards, badges, tables, forms, dialogs, and object-family
  components. Any intentional deviation requires a named task, before/after
  evidence, and review approval.
- **VIS-003**: Borrow hierarchy, grouping, search, progressive disclosure, and
  review-flow ideas from the screenshot gallery; never copy its product chrome,
  bottom navigation, promotional surfaces, saturated category colours, or social
  graph assumptions.
- **VIS-004**: Each task screen has one dominant primary action and at most one
  peer alternative. Secondary, technical, advanced, and destructive actions use
  progressive disclosure or clearly separated groups.
- **VIS-005**: Every flow has an obvious entry, Back/Cancel path, review step
  where authority or disclosure changes, and actionable completion/error state;
  no Roadmap route ends in a dead-end mock card.
- **VIS-006**: Search and filters are local to the current collection or Help
  catalogue. They never create another global navigation hierarchy or send
  user-entered terms to a remote service.
- **ICON-001**: Every visible menu branch, subgroup, leaf, Help link, and Log out
  action has a stable `IconId` in the canonical navigation registry. Sibling
  destinations use distinguishable semantic icons; an icon may be reused
  elsewhere only for the same action or meaning.
- **ICON-002**: Reuse the bundled inline SVG registry and its current
  `currentColor`, 1.8 px outline contract on a normalized 20/24 px grid. Add a
  local icon only when the existing registry has no accurate semantic glyph.
- **ICON-003**: Navigation icons use neutral grey tiers in both palettes. Hover
  and selection are communicated by row surface, text weight, rail/border, and
  restrained Z00Z accent—not a different icon hue per destination and not
  coloured circular/square icon tiles.
- **ICON-004**: Existing content-level asset, voucher, Permission, severity, and
  status colours may remain where they encode domain meaning. They are not
  navigation colours, and no state may rely on colour alone.
- **PAL-001**: The application exposes exactly these two canonical palette IDs:

  | `PaletteId` | Scheme | Contract |
  | --- | --- | --- |
  | `z00z-default` | Dark | The current demo and initial fallback palette; its existing semantic token values are unchanged |
  | `z00z-corporate` | Light | A locally bundled Z00Z application mapping of the live `z00z.io` Corporate source palette |

- **PAL-002**: `PaletteId` determines `color-scheme`. Remove the independent
  System/Dark/Light control and do not create dark Corporate, light Default, or
  automatic-system variants in this plan. Replace generic application
  `data-theme` branching with the explicit palette mapping; code-highlight
  themes remain separately namespaced.
- **PAL-003**: The initial Corporate source snapshot, verified on 2026-07-26, is
  `base-100 #FFFFFF`, `base-200 #E8E8E8`, `base-300 #D1D1D1`,
  `base-content #181A2A`, `primary #0082C4`, `secondary #61738D`,
  `accent #009588`, `success #00A242`, `warning #F7C800`, and
  `error #FF6266`. Map these through local Z00Z semantic roles, preserve the gold
  logo, and contrast-test the resulting controls, text, focus, status, and
  privacy states.
- **PAL-004**: Vendor both palettes in the application bundle. Runtime rendering
  never downloads CSS, tokens, fonts, icons, or theme data from `z00z.io`.
- **PAL-005**: Migrate legacy appearance preferences deterministically in this
  precedence order: explicit `z00z-corporate` → Corporate; otherwise legacy
  `theme: light` → Corporate; every other combination—including legacy dark,
  removed palette IDs without a light theme, missing values, and invalid
  values—→ Z00Z Default. Persist only the canonical `PaletteId` after migration.
- **PAL-006**: Retire `black-gold-elegance`, `moonlit-stroll`, and
  `walking-at-night` from the application palette registry, UI, CSS mappings,
  fixtures, Help, configuration examples, and tests. Remove unsupported hidden
  light/dark palette combinations rather than merely hiding their controls.
- **PAL-007**: Palette choice uses two compact cards containing only the palette
  swatch, name, and selected `ACTIVE` marker—no descriptive paragraph or
  duplicate applied-status copy. Selecting a card applies it immediately across
  the app and Help. There are no Apply, Cancel, or Reset controls. Text scale,
  reduced motion, density, and code-syntax highlighting remain independent;
  code highlighting may colour code tokens only and never navigation or safety
  semantics.
- **PAL-008**: Both palettes must pass WCAG 2.2 AA contrast, keyboard focus,
  forced-colours, hidden-sensitive-value, 200% zoom, and semantic-state review.
- **PAL-009**: Keep `demo/styles/colors.css` as the only owner of literal
  application colours. Components consume semantic tokens such as
  `--bg-surface`, `--text-secondary`, `--nav-icon`, `--focus`, and status roles;
  no route or icon introduces an inline destination colour.

Initial structural icon assignments are fixed at the root level:

| Node | Initial `IconId` |
| --- | --- |
| Wallet | `wallet` |
| Telemetry | `network` |
| dApps | `spark` |
| Messenger | `message` |
| Contacts | `user` |
| Data & Storage | `storage` |
| Settings | `settings` |
| Help | `question` plus external-link affordance |
| About | `info` |
| Log out | `logout` |

The implementation may refine an initial icon assignment only when the glyph
meaning remains stable, the navigation/SVG registries and tests update
atomically, and the sibling-icon validation still passes.

### 1.6 Capability and claim-honesty requirements

Backend maturity and current UI availability are independent facts. The demo
must not compress them into one green/red badge.

| Axis | Values | Meaning |
| --- | --- | --- |
| `Maturity` | `live`, `target`, `concept` | Whether the underlying Z00Z contract is implemented, specified, or exploratory |
| `Availability` | `available`, `degraded`, `unavailable` | Whether this packaged app has an authoritative native bridge and usable data now |
| `EvidenceSource` | `native`, `fixture`, `none` | Whether values came from a native authority, deterministic demo data, or no source |
| `Freshness` | timestamp, stale, unknown, not applicable | When transport-derived or operational data was last known |
| `PresentationMode` | `product`, `roadmap_preview` | Whether the surface represents a currently productized flow or an explicitly forward-looking interactive design |

- **CAP-001**: Every capability-bearing Telemetry, dApp, Messenger, Contacts
  integration, or compatibility route declares all five fields above. Pure
  navigation, local Settings, and Help surfaces do not render meaningless
  capability badges; `Freshness::NotApplicable` is used only when time has no
  domain meaning.
- **CAP-002**: A deterministic fixture is never labelled live telemetry.
- **CAP-003**: Target, concept, and roadmap-preview screens are fully navigable,
  useful demonstrations whose functional availability, provenance where
  relevant, and empty/error states state the boundary in plain language.
- **CAP-004**: An unavailable authoritative value renders `Unavailable`; it is
  never replaced with `0`, `Healthy`, a fabricated percentage, or a synthetic
  success check.
- **CAP-005**: Do not show one universal privacy, network, or system-health
  score. Show layer-specific evidence and its provenance.
- **CAP-006**: Compatibility operations remain disabled or explicitly
  experimental until canonical mutation and settlement authority are proven.
- **CAP-007**: The visual demo MUST always expose Watchers, Explorer, dApps, and
  Messenger as selectable `Roadmap preview` branches. They are not disabled
  placeholders and do not collapse to a generic `Coming soon` page.
- **CAP-008**: Every mandatory roadmap branch includes deterministic loading,
  useful success, empty, degraded/unavailable, malformed, and error scenarios,
  plus at least one complete task or evidence flow defined in Section 3.
- **CAP-009**: Production builds may capability-gate an unimplemented module,
  but that release rule does not remove the mandatory interactive roadmap
  representation from this browser demo and design specification.
- **CAP-010**: The five typed axes remain canonical internal contracts and test
  inputs; do not repeat them in a generic capability-summary header/card at the
  top of each route. Render only task-relevant state at its point of use—for
  example `Unavailable`, a fixture timestamp, or a functional recovery control.
  Detailed authority/provenance belongs in contextual Help or an intentional
  technical disclosure, not a mandatory banner. Green never means roadmap
  maturity, and warning/danger remain reserved for actual status.

### 1.7 Protocol-derived product boundaries

- **WALLET-001**: Wallet remains the default and primary workspace.
- **WALLET-002**: Assets, Vouchers, and Permissions remain distinct object
  families. Permissions are zero-value Rights and never enter a balance.
- **WALLET-003**: Exported, submitted, admitted, and settled are distinct
  lifecycle states. A local or inbox acknowledgement is not settlement.
- **WALLET-004**: Swap and Staking remain compatibility/experimental;
  Exchange remains target-only until an authoritative provider contract exists.
- **WALLET-005**: Quarantine is a capability/count-gated Wallet leaf for
  unsupported, malformed, unknown-policy, or unsafe objects. It never enters
  Available and must not occupy permanent navigation when it has no actionable
  content.
- **TEL-001**: Telemetry is read-only observation. It does not configure
  Reticulum, OnionNet, aggregators, Watchers, storage, or settlement rules.
- **TEL-002**: Watchers expose observations, typed alerts, and evidence only.
  They never become planner, validator, storage, or settlement authority.
- **TEL-003**: Explorer exposes only intentionally public checkpoint, batch,
  publication, proof, and DA-reference material.
- **TEL-004**: Explorer and Watchers never expose wallet labels, balances,
  receivers, counterparties, memos, private route paths, local inbox records,
  or object openings.
- **TEL-005**: OnionNet remains a target transport/ingress architecture and
  must not be presented as shipped universal anonymity or universal messaging.
- **DAPP-001**: dApps mean curated applications over bounded Assets, Vouchers,
  Permissions, Claims, and service rights—not a universal private VM.
- **DAPP-002**: Every dApp connection uses an intent-level permission review
  with action, object family, scope, uses, expiry, delegation, value, and fee
  shown separately.
- **DAPP-003**: The demo loads no third-party remote code, iframe, or arbitrary
  application bundle.
- **DAPP-004**: A dApp cannot request generic raw-byte signing, direct wallet
  storage, a session token, a seed, or a private key.
- **MSG-001**: Messenger content is off-chain and conceptually short-lived.
  Public settlement may expose only minimal receipt relevance for a payment,
  claim, ticket, permission, or notice.
- **MSG-002**: Inbox records are advisory. Opening or accepting a message cannot
  mutate wallet ownership, advance a scan cursor, prove ownership, or settle an
  object.
- **MSG-003**: Avoid permanent public inbox identities, public presence, a
  public social graph, and default read receipts.
- **MSG-004**: The UI distinguishes the current wallet-local advisory inbox from
  target Encrypted Receipt Mailbox and future OnionNet transport.
- **CONTACT-001**: Contacts are encrypted, wallet-local labels over receiver
  cards, payment requests, or explicitly imported public material.
- **CONTACT-002**: Contacts are not a public address book and do not imply
  identity verification, presence, reachability, or settlement trust.
- **SET-001**: Application Settings contains only General and Appearance.
- **SET-002**: Wallet Security, Backup, Policies, and Advanced remain under the
  selected wallet, not application Settings.
- **SET-003**: Network/runtime configuration is not duplicated under Settings;
  future mutation belongs behind a separate typed native contract, not
  Telemetry.

### 1.8 Help, localization, accessibility, and offline constraints

- **HELP-001**: Help opens in a separate named browser tab for the demo and a
  separate Tauri window/webview in production.
- **HELP-002**: The global Help leaf opens the Help root; the fixed contextual
  question action opens the exact active route/topic.
- **HELP-003**: The app and Help windows remain independently open and can be
  focused or closed without closing the other.
- **HELP-004**: Help uses the same canonical navigation labels and independent
  multi-open behavior for its root accordion groups. Help has no nested
  accordions.
- **HELP-005**: Organize Help sources for every locale under
  `app`, `wallets`, `telemetry`, `dapps`, `messenger`, `contacts`, and
  `settings`. Copy each source into its canonical topic directory and migrate
  the runtime catalogue from `network` to `telemetry` without deleting any
  pre-existing Markdown. Retain every original source through
  `preserved-sources.json`; no canonical runtime topic may remain loose at a
  locale root.
- **HELP-006**: Desktop and mobile MUST let the user switch between the wallet
  and Help surfaces and close either one without closing or resetting the
  other. Pages uses independent browser tabs; the Tauri spike must prove the
  platform-native independent window/scene/webview model before implementation.
- **HELP-007**: Help initializes from the application's canonical `PaletteId`,
  supports only Z00Z Default and Z00Z Corporate, and remains fully usable if the
  wallet surface is closed. Palette synchronization carries no wallet data or
  secret state.
- **HELP-008**: Contextual Help invoked from a page, sheet, review, or dialog
  still opens/focuses the same separate Help surface at the exact topic. It never
  inserts a `HelpPanel` into the app overlay stack. The invoking app state remains
  intact for return, while required safety copy stays visible at the action.
- **HELP-009**: On phones, `parallel` means independently lifecycled native
  surfaces that the user can switch between and close independently; it does not
  promise side-by-side display. The Tauri spike must use Android Activity/iOS
  `UIScene` support, check runtime multi-window capability, and prove the exact
  behavior on real iPhone and iPad targets.
- **HELP-010**: Desktop Help keeps root groups and their first-level
  documentation workspaces visible in the left sidebar, then renders the
  selected workspace's topics in a vertical context rail. Mobile Help keeps the
  same global tree in its drawer and renders those topics as horizontally
  scrollable top tabs below the Help topbar.
- **I18N-001**: Preserve all ten canonical locales: `en`, `ru`, `fr`, `de`,
  `es`, `pt`, `ko`, `tr`, `ja`, and `zh-Hans`.
- **I18N-002**: Route labels, breadcrumbs, capability copy, errors, Help topics,
  and accessible names must have exact locale parity.
- **I18N-003**: User-authored names, protocol identifiers, fingerprints, and
  raw evidence are never translated.
- **I18N-004**: Every locale has exact canonical topic-directory,
  filename/topic-ID, and generated-catalogue parity with English; a build fails
  on a missing, extra, or misplaced runtime topic or on any missing preserved
  original listed in `preserved-sources.json`.
- **A11Y-001**: Meet WCAG 2.2 AA for core flows.
- **A11Y-002**: Use semantic `nav`, nested lists, buttons for toggles/actions,
  links for routes, `aria-expanded`, `aria-controls`, and one
  `aria-current="page"`.
- **A11Y-003**: Every pointer target is at least 44 × 44 CSS px.
- **A11Y-004**: Keyboard and screen-reader users can expand several branches,
  close one while preserving another, select a leaf, and identify the active
  path without relying on color.
- **A11Y-005**: At 200% text zoom and 320 CSS px, no ordinary route requires
  document-level horizontal scrolling.
- **A11Y-006**: Reduced-motion mode keeps state changes visible without
  accordion animation.
- **OFFLINE-001**: Bundle every renderer asset, icon, font, locale, fixture, and
  Help catalogue locally.
- **OFFLINE-002**: Renderer code uses no Internet, LAN, DNS, CDN, HTTP,
  WebSocket, TCP, or browser wallet-RPC dependency.
- **CON-001**: Preserve unrelated dirty worktree files.
- **CON-002**: The GitHub Pages demo remains fabricated, non-authoritative, and
  incapable of signing or connecting to a real wallet.
- **CON-003**: Tauri/Leptos dependency versions are selected only after the
  production spike verifies packaging, CSP, lifecycle, secure storage,
  updater/signing, and supported targets.

### 1.9 Screenshot reference decisions

The images in `crates/z00z_ui_ux/Z00Z-App-TODO/` are visual references, not
product requirements and not permission to copy another product's brand shell.
The live `z00z.io` mobile navigation was also checked at 390 × 844 on
2026-07-26: the Z00Z logo remained visible, two navigation groups could stay
open, and closing one preserved the other. This plan adopts that interaction
contract while retaining application-specific routes, capability states, and
security boundaries.

| Reference pattern | Adopt | Reject or adapt |
| --- | --- | --- |
| Gmail drawer | Clear groups, neutral icon/label rows, restrained hierarchy | Do not reproduce Google visual branding |
| Telegram settings | Dense grouped rows, predictable toggles, local search, compact mobile rhythm | No duplicate bottom navigation, coloured category-icon palette, or account-centric social graph |
| Telegram contacts | Search, alphabetic grouping, add/import entry, clear empty states | No public presence, phone-number dependency, or address-book upload |
| Telegram FAQ and Google Help | Searchable standalone Help, tree plus article, independent window | No remote Help dependency and no modal-only Help |
| MetaMask settings | Clear section groups and separated destructive Log out | No generic signing surface or browser-extension assumptions |
| NEAR wallet flows | Task-oriented quick actions, step-by-step review, compact pickers | No market ticker emphasis and no global top-tab hierarchy |
| Ledger and Slush | Balance/quick-action cards, account-scoped settings, advanced disclosure | No earn-first or portfolio-performance framing |
| Existing Z00Z demo | Dark/gold tokens, object-family separation, honest pending states, selected-wallet scope | Remove top route tabs and the old Network naming |

The resulting Z00Z synthesis is:

1. one quiet, scan-friendly hierarchy rather than simultaneous sidebar, top-tab,
   popup-menu, and bottom-navigation systems;
2. compact grouped rows and local search for long collections;
3. one dominant task action, followed by explicit review and outcome states;
4. progressive disclosure for technical, advanced, and destructive controls;
5. standalone searchable Help that can remain beside the application;
6. neutral structural icons, with colour reserved for Z00Z selection,
   object-family meaning, and semantic status.

These rules generalize the useful interaction concepts in the gallery while
preserving the existing Z00Z component language.

### 1.10 Recommended demo stories

The expanded demo should prove a small number of connected stories instead of
displaying a pile of unrelated mock screens.

1. **Offline private payment**: create/review a payment, export or queue it,
   broadcast later, then reconcile to a settled or failed state.
2. **Private voucher distribution**: receive a bounded voucher, review backing
   and rules, accept/redeem it, and keep its value separate from Available.
3. **Bounded dApp permission**: a curated service requests a Permission; the
   user reviews scope, uses, expiry, delegation, value, and fee before granting.
4. **Request-bound inbox**: receive a payment/claim/permission notice, inspect
   its advisory status, and explicitly enter the authoritative wallet flow.
5. **Watcher-to-Explorer evidence**: open a typed publication/DA alert, then
   inspect only its public checkpoint and batch evidence.
6. **Local contact handoff**: save a receiver card locally, use it to create a
   scoped payment request, and show that the contact is not a permanent public
   address.

The following remain labelled target or concept demonstrations: arbitrary dApp
execution, live OnionNet messaging, remote third-party app content, a universal
explorer/indexer, exchange liquidity, market data, autonomous agent spending,
and universal network-health claims.

Do not add separate Governance/DAO, Liability, Market, Developer tools,
Notifications, or Diagnostics roots in this plan. Governance and Linked
Liability remain later target layers; Exchange stays wallet-scoped; developer
tools stay in Wallet Settings → Advanced; attention stays in the topbar and
History; diagnostics remain contextual to the observed Telemetry layer.

## 2. Canonical Information Architecture

### 2.1 Navigation tree

```text
[Wallet switcher: selected wallet, abbreviated fingerprint, Add/Manage]

Wallet ▼
  Assets & Rights
  Send
  Receive
  History
  Swap                         [Compatibility]
  Exchange                     [Target]
  Staking                      [Compatibility]
  Backup
  Wallet Settings

Telemetry ▼
  Reticulum
  OnionNet
  Aggregators
  Watchers                     [Roadmap preview]
  Explorer                     [Roadmap preview]

dApps ▼                         [Roadmap preview]
  Discover
  Installed
  Connections
  Permissions
  Activity

Messenger ▼                     [Roadmap preview]
  Inbox
  Requests
  Conversations
  Outbox
  Receipts

Contacts

────────────────────────
Settings ▼
  General
  Appearance

Help ↗
About
────────────────────────
Log out
```

`Assets & Rights` opens one wallet workspace with local destinations `Assets`,
`Vouchers`, and `Permissions`. `Wallet Settings` opens one selected-wallet
workspace with local destinations `General`, `Security`, `Backup`, `Policies`,
and `Advanced`. On desktop these local destinations are an always-visible
vertical context rail inside the main workspace. On mobile and narrow tablet
they are an always-visible, horizontally scrollable top tab row. They never
appear as global-tree leaves. Quarantine is a conditional state/filter inside
the object-family workspace and is never a permanent global-tree row.

Each Telemetry component is likewise one first-level workspace leaf:

- Reticulum: Overview, Node, Interfaces, Radio, Entry points, Paths, Probes,
  Links.
- OnionNet: Overview, Epoch, Privacy, Transport, Queues & replay, Probation,
  Ingress.
- Aggregators: Overview, Ingress, Planning, Placement, Publication, Recovery.
- Watchers: Overview, Alerts, Publication checks, DA providers, Censorship
  signals, Evidence export.
- Explorer: Overview, Search, Checkpoints, Batches, Public evidence.

These component destinations use the same desktop context rail/mobile top-tab
projection and never appear as deeper rows in the global sidebar or drawer.

The wallet switcher is contextual state, not a route branch. Changing the
wallet refreshes wallet-owned routes and data while preserving the current
wallet-compatible route where safe. If the route is not valid for the selected
wallet, navigate to Wallet → Assets & Rights and explain why.

Watchers, Explorer, dApps, and Messenger are always present in the design demo.
Their selected destination content identifies the Roadmap/target presentation
state; the navigation tree never displays a `ROADMAP` badge. First run opens
Wallet only so the long tree remains calm. Roadmap root accordions start
collapsed, but selecting a deep link opens its root ancestor. A collapsed root
containing the active route shows a neutral active-descendant marker without
pretending that the root itself is the selected page.

### 2.2 Route namespaces

| Namespace | Stable route IDs | Default route |
| --- | --- | --- |
| Wallet | `wallet.assets`, `wallet.vouchers`, `wallet.permissions`, `wallet.quarantine`, `wallet.send`, `wallet.receive`, `wallet.history`, `wallet.swap`, `wallet.exchange`, `wallet.staking`, `wallet.backup`, `wallet.settings.*` | `wallet.assets` |
| Reticulum | `telemetry.reticulum.{overview,node,interfaces,radio,entrypoints,paths,probes,links}` | `telemetry.reticulum.overview` |
| OnionNet | `telemetry.onionnet.{overview,epoch,privacy,transport,queues,probation,ingress}` | `telemetry.onionnet.overview` |
| Aggregators | `telemetry.aggregators.{overview,ingress,planning,placement,publication,recovery}` | `telemetry.aggregators.overview` |
| Watchers | `telemetry.watchers.{overview,alerts,publication,providers,censorship,evidence}` | `telemetry.watchers.overview` |
| Explorer | `telemetry.explorer.{overview,search,checkpoints,batches,evidence}` | `telemetry.explorer.overview` |
| dApps | `dapps.{discover,installed,connections,permissions,activity}` | `dapps.discover` |
| Messenger | `messenger.{inbox,requests,conversations,outbox,receipts}` | `messenger.inbox` |
| Contacts | `contacts.list` | `contacts.list` |
| Settings | `settings.{general,appearance}` | `settings.general` |

Detail screens such as transaction, object, message, contact, dApp, alert,
checkpoint, or batch details are route variants owned by their parent
namespace. They do not become permanent sidebar leaves.

### 2.3 Accordion state contract

`expanded_branch_ids` is a set, not a single selected branch.
Only root accordions may enter this set. Workspace leaves such as Reticulum,
OnionNet, Aggregators, Watchers, and Explorer navigate to their default internal
destination and have no toggle state, `aria-expanded`, or chevron.

| Input | Desktop result | Mobile result |
| --- | --- | --- |
| Toggle a closed branch | Open it and keep all other branches unchanged | Same; drawer stays open and waits for selection |
| Toggle an open branch | Close it and keep all other branches unchanged | Same; drawer stays open |
| Select a leaf | Change route, preserve expansion state | Change route, preserve expansion state, close drawer |
| Navigate through Back/Forward | Restore route and ensure its ancestors are open | Same; drawer remains closed |
| Press Escape/Back with drawer open | No route change | Close drawer only |
| Log out | Clear route-sensitive presentation state | Same; return to lock surface |
| Unlock again | Restore non-sensitive navigation preference or default tree | Same |

Expansion state is low-sensitivity presentation state, but it can still reveal
which workspaces a person used. The browser demo keeps it in memory. Production
may persist only the branch set—not active routes, drafts, searches, message
identities, or wallet context—through an approved local app-preference gateway.
It is never synced or exported, and Log out resets it to the safe first-run
default. Lock closes the mobile drawer and obscures content without granting the
renderer new persistence authority.

### 2.4 Topbar contract

Desktop order:

1. Z00Z logo and wordmark.
2. Current page title and optional compact breadcrumb.
3. Layer-specific connection/scan status.
4. Selected wallet context.
5. Hide/show sensitive values.
6. Attention/notifications.
7. Lock/account action.

Mobile order:

1. Menu.
2. Z00Z logo and wordmark.
3. Truncating current page title.
4. One context action when essential; otherwise attention/notifications.

The mobile drawer begins below the sticky topbar and uses the remaining dynamic
viewport height. This keeps the Z00Z logo visible, respects safe-area insets,
and prevents duplicated branding inside the drawer. While the drawer is open,
Menu becomes a labelled Close control with `aria-expanded="true"` and
`aria-controls`; the logo stays visible, other app/topbar controls are inert,
and keyboard focus is contained between Close and the drawer.

The navigation tree owns the scrollable region. Settings, Help, About, and Log
out use a separate footer region only when it fits without covering a leaf; on
short heights the footer joins the scroll flow. Sticky/fixed controls may never
occlude the last route or the bottom safe area.

### 2.5 Local navigation controls

Use one page-local destination model when all choices share the same workspace,
ownership/security scope, and task family. Render it as a vertical context rail
on desktop and as a horizontally scrollable top tab row on mobile and narrow
tablet. Examples:

- Assets & Rights: Assets, Vouchers, Permissions.
- Send: Assets, Vouchers, Permissions.
- Wallet Settings: General, Security, Backup, Policies, Advanced.
- Standalone Help: current category/topic siblings.
- Activity filters: All, Assets, Vouchers, Permissions, System.
- Explorer evidence views: Summary, Technical details.
- Appearance palette previews: Z00Z Default and Z00Z Corporate.
- Data density or chart/list view.

Do not use local tabs for global Wallet, Telemetry, dApps, Messenger, Contacts,
application Settings roots, or the full Help tree. Local tabs use the existing
restrained selected-row treatment, not decorative pill navigation.

## 3. Workspace Specifications

### 3.1 Wallet

Preserve the existing Wallet specification and completed demo behavior while
moving every route into the canonical tree.

- Wallet has no Overview route. Assets & Rights is the default wallet
  workspace.
- Assets & Rights separates Assets, Vouchers, and Permissions through its
  desktop context rail/mobile top tabs. These local destinations do not appear
  as global-tree rows. Capability-gated Quarantine appears only inside this
  workspace when unsupported/unsafe objects require review or an explicit
  expert capability is active.
- History owns transactions, object events, pending/reconciliation alerts,
  security/backup events, and system changes.
- Swap and Staking remain experimental recipes until canonical authority is
  proven. Exchange renders an honest target/unavailable screen.
- Backup is the operational backup/restore workspace. Wallet Settings → Backup
  contains schedule and policy preferences; labels must distinguish the two.
- Wallet Settings remains selected-wallet scoped.
- The existing ASCII flow inventory under `Z00Z-App-TODO/views/` informs
  previews, progress, failure states, technical disclosures, and native
  file/share boundaries. It does not automatically promote every operation to
  a top-level production capability.

### 3.2 Telemetry

Telemetry screens do not begin with a repeated capability-summary card. They
show explicit unavailable/degraded reasoning, provenance/freshness only where
it qualifies a displayed value, and route-specific Help through the shared `?`
entry. No Telemetry surface renders a user route, receiver, counterparty, or
wallet-private identifier.

#### Reticulum

Use the existing Overview, Node, Interfaces, Radio, Entry points, Paths, Probes,
and Links routes. Report managed-node or local interface evidence only. Never
claim a complete global network view from local evidence.

#### OnionNet

Use Overview, Epoch, Privacy, Transport, Queues & replay, Probation, and Ingress.
The concept may show target public epoch state, aggregate transport classes,
queue health, replay posture, and admission gates. It must not show the user's
live route, permanent inbox identity, raw endpoint, session, or a universal
anonymity score.

#### Aggregators

Use Overview, Ingress, Planning, Placement, Publication, and Recovery.

- Ingress shows payload/digest admission outcomes.
- Planning shows work/batch planning without implying settlement authority.
- Placement shows operational shard placement as observational/runtime data.
- Publication shows publication state and binding provenance.
- Recovery shows lineage/generation/failover state.
- Storage remains the owner of settlement roots, proofs, and recovery truth.

#### Watchers

Use Overview, Alerts, Publication checks, DA providers, Censorship signals, and
Evidence export. Map the screens to the existing watcher modules:
`status`, `alerts`, `publication`, `da_health`/`provider`, `censorship`, and
`evidence_export`.

The demo implementation is a mandatory interactive Roadmap preview even where
the underlying watcher module is live. It must let a reviewer switch deterministic
observation sources, inspect a typed alert, filter evidence, export a sanitized
fixture envelope, and exercise empty/degraded/malformed/error recovery. The
Roadmap label describes the app surface and native bridge—not a claim that all
underlying watcher concepts are absent.

Watcher alerts must include typed kind, severity, subject, observation time,
provenance, affected public IDs, and a safe next action. Opening an alert may
deep-link to Explorer public evidence. It may not mutate or override runtime,
validator, storage, or settlement state.

#### Explorer

Explorer is a privacy-restricted public evidence viewer.

- The demo implementation is a mandatory interactive Roadmap preview with
  deterministic searchable public identifiers, not a disabled placeholder.
- Overview explains its narrow public scope.
- Search accepts only public checkpoint, batch, publication, proof, or opaque
  DA-reference identifiers.
- Checkpoints show lifecycle, roots, finality/publication evidence, and
  freshness where authoritative.
- Batches show public batch/publication relationships.
- Public evidence shows proof envelopes, publication bindings, route snapshots,
  and DA references only when intended for public display.
- Unknown, private, malformed, stale, or unsupported identifiers fail closed
  with sanitized errors.

### 3.3 dApps

The demo dApp area is a **bounded application catalogue and permission center**,
not a remote web browser.

It is a mandatory interactive Roadmap preview. Discover, dApp detail,
connection, permission review, accepted/rejected outcome, revoke/expiry, and
Activity must all be demonstrable with deterministic local fixtures. No route
may resolve to a generic `Coming soon` screen.

| Screen | Purpose |
| --- | --- |
| Discover | Curated local catalogue grouped by verified use-case family |
| Installed | Locally approved app descriptors and current capability state |
| Connections | Pending and active intent-level connections |
| Permissions | Rights/scopes granted to an app, with revoke and expiry |
| Activity | App-originated requests and wallet outcomes without private raw payloads |

Recommended deterministic catalogue:

1. Offline Pay — private cash handoff and later reconciliation.
2. Private Voucher — bounded aid/community distribution.
3. External Asset Locker — private right over external custody.
4. Scoped Expenses — organizational payment/permission review.
5. Service Credits — API, data, compute, or access Rights.
6. Agent Budget — concept-only composed Permission plus separate value/fee path.

Each card shows maturity, availability, publisher/trust provenance, requested
object families, offline behavior, data disclosure, and Help. No card claims
that a declared domain or icon is verified.

The permission review must show:

- human-readable intent;
- app identity and provenance;
- action;
- object family and exact scope;
- one-time or bounded uses;
- expiry;
- delegation and attenuation;
- amount/value, if any;
- fee path, separately;
- data disclosed to the app;
- revoke behavior;
- confirmation and re-auth requirement.

Production MVP executes no arbitrary third-party code in the wallet renderer.
If external app execution is designed later, it requires a separate threat
model and isolated Tauri webview/window with a per-app allowlist. That webview
must not receive the wallet command bridge; it exchanges typed intents through
an explicit broker.

### 3.4 Messenger

Messenger is a mandatory interactive Roadmap preview. Its navigation and
fixture-backed controls remain usable even though the durable mailbox and
OnionNet transport layers are target/future capabilities. The persistent
Roadmap label prevents a polished interaction from being mistaken for a shipped
protocol service.

| Screen | Purpose | Honest maturity |
| --- | --- | --- |
| Inbox | Local advisory items and delivery state | Current helper plus target durable profile |
| Requests | Payment, voucher, claim, permission, receiver-card, or service requests | Demo flow; mutation remains explicit |
| Conversations | Short-lived off-chain threads | Concept only |
| Outbox | Queued, relayed, acknowledged, expired, or failed delivery | Target |
| Receipts | Delivery/acknowledgement and settlement-relevant receipts | Target, non-authoritative |

Required behavior:

- Opening a request is read-only.
- Accepting a request first opens the appropriate Wallet review flow.
- The Wallet gateway revalidates the request and owns every mutation.
- Message and inbox status never means payment settlement.
- Retention, expiry, delete, block, and abuse-report controls are visible.
- Receiver material, request IDs, locators, route buckets, ACK secrets, compact
  requests, and raw packages are redacted from logs and ordinary UI.
- Conversation search is local and excludes secret/raw protocol fields.
- A provider or relay outage never becomes wallet ownership or finality truth.

### 3.5 Contacts

The Contacts screen adopts the compact searchable list pattern from the
reference gallery while preserving Z00Z privacy boundaries.

- Search local label, safe note, tag, abbreviated fingerprint, or supported
  public material.
- Add from receiver card, payment request, QR scan, native share/import, or
  manual reviewed public material.
- Show source, last local use, chain/profile compatibility, expiry when present,
  and verification/pinning state.
- Separate `Known locally`, `Needs confirmation`, `Identity changed`,
  `Expired`, and `Revoked`.
- Use explicit actions: Pay, Request, Message, Edit label, Export public
  material, Remove.
- Removal deletes only the local contact record; it does not revoke protocol
  objects or erase counterparty history.
- Never upload the contact list or infer a public social graph.

### 3.6 Settings

Application Settings contains:

- **General**: language, regional format, time zone, notifications, lock/session
  entry, sanitized diagnostics access, and app version/build provenance.
- **Appearance**: exactly two complete application choices—Z00Z Default (current
  dark) and Z00Z Corporate (light)—plus text scale, reduced motion, density, and
  code-syntax highlighting. It contains no independent System/Dark/Light toggle
  and no custom application-colour editor.

Selecting a palette applies its canonical `PaletteId` immediately. The selected
card alone displays `ACTIVE`; changing the palette updates both the app and Help
presentation without changing wallet/runtime data.

Wallet-specific Security, Backup, Policies, and Advanced remain under Wallet
Settings. Telemetry is observation and does not become a hidden configuration
editor.

### 3.7 Help

The Help application mirrors the canonical tree but contains documentation
groups instead of wallet data.

- The global Help action opens/focuses one named Help window at its root.
- The contextual `?` action opens/focuses the same window at the active topic
  and exact section.
- Closing the wallet leaves Help open; closing Help leaves the wallet open.
- Windows/Linux surfaces may be moved, focused, minimized, or closed
  independently. On iOS, the proven platform-native scene/window behavior must
  still allow switching and closing either surface without closing or resetting
  the other.
- Help search uses the bundled locale catalogue only.
- The Help tree supports independent multi-open root accordions and no nested
  accordions.
- The Help global tree exposes first-level documentation workspaces rather than
  every topic route.
- Desktop exposes a selected documentation workspace's topics in a vertical
  context rail; mobile keeps the global tree in its drawer and exposes the same
  topics as top tabs.
- Safety warnings, validation, unavailable state, destructive consequences,
  and errors remain in the application at the point of action.

## 4. Future Tauri and Rust Port Architecture

This section is a porting contract for a later production project. Nothing in
this section is part of the pure-JS demo implementation, and no Rust/Tauri crate
may be created while executing this demo plan.

### 4.1 Required production topology

```text
Leptos views
  └─ presentation stores
      └─ typed domain gateways
          ├─ Windows/Linux: authenticated local OS IPC to z00z-walletd
          └─ iOS: typed in-process native adapter

Tauri 2 host
  ├─ packaged local Leptos CSR/WASM assets
  ├─ lifecycle and secure platform bridge
  ├─ allowlisted intent commands
  ├─ sanitized native events
  ├─ native file/share/clipboard/notification adapters
  └─ separate Help window/scene/webview surface
```

The renderer owns declarative views and ephemeral presentation state only.
Native Rust owns passwords, seeds, keys, tokens, encrypted wallet files,
signing, authorization, policy enforcement, configuration mutation, operation
journals, settlement interaction, secure storage, and platform lifecycle.

### 4.2 Crate responsibilities

| Crate | Owns | Must not own |
| --- | --- | --- |
| `z00z_wallet_ui_contract` | Route-safe DTOs, enums, gateway traits, structured errors, sanitized events | Leptos, Tauri, wallet services, database, private keys |
| `z00z_wallet_ui` | Leptos components, route registry, presentation stores, accessibility, design tokens | Raw RPC strings, wallet DB, crypto, direct filesystem/network |
| `z00z_wallet_ui_tauri` | Tauri host, command allowlist, lifecycle, native window/file/share/clipboard adapters | Wallet business rules or duplicate DTO semantics |
| `z00z_walletd` | Desktop gateway adapter, authenticated OS IPC, durable operation journal | View state and arbitrary renderer code |
| `z00z_wallets` | Wallet-domain truth, validation, storage, signing, mutation | UI navigation and platform window state |

These are proposed future seams from `demo/PORTING.md`; they do not exist as
part of this demo. A later authorized production implementation must avoid
creating duplicate UI contracts or exposing current internal RPC method strings
as the product API.

### 4.3 Typed route and navigation model

The production contract should use small, exhaustive domain enums rather than
stringly typed routes:

```rust
pub enum AppRoute {
    Wallet(WalletRoute),
    Telemetry(TelemetryRoute),
    Dapps(DappRoute),
    Messenger(MessengerRoute),
    Contacts,
    Settings(SettingsRoute),
}

pub enum NavTarget {
    Branch,
    Route(AppRoute),
    Help(HelpTopicId),
    Action(AppAction),
}

pub enum PresentationMode {
    Product,
    RoadmapPreview,
}

pub enum PaletteId {
    Z00zDefault,
    Z00zCorporate,
}

pub struct NavNode {
    pub id: NavNodeId,
    pub parent: Option<NavNodeId>,
    pub label: LocaleKey,
    pub icon: IconId,
    pub target: NavTarget,
    pub capability: Option<CapabilityId>,
    pub presentation: PresentationMode,
}

pub struct ShellState {
    pub active_route: AppRoute,
    pub expanded: BTreeSet<NavNodeId>,
    pub drawer_open: bool,
    pub sensitive_values_hidden: bool,
    pub palette: PaletteId,
}
```

Exact enum variants are finalized during Phase 1. The rules are fixed now:

- one route registry;
- no duplicate string aliases as parallel authority;
- exhaustive matches for routes, Help, permissions, and error mapping;
- `NavNodeId` and `HelpTopicId` are stable identifiers;
- locale labels and icons are presentation metadata;
- `PaletteId` is exhaustive and has exactly two variants; light/dark is derived
  from it rather than stored as a second independent preference;
- `PresentationMode` is independent from protocol maturity, native availability,
  and fixture provenance;
- authorization never comes from route or presentation state.

### 4.4 Domain gateway boundaries

Use interface-segregated traits behind one application facade:

- `WalletGateway`
- `TelemetryGateway`
- `DappGateway`
- `MessengerGateway`
- `ContactsGateway`
- `SettingsGateway`
- `PlatformGateway`

Each trait exposes intent-level commands and presentation-safe queries. Do not
add generic `rpc_call`, `sign_bytes`, arbitrary filesystem path, arbitrary URL,
or raw transport methods.

The static design demo selects deterministic scenarios explicitly. A production
gateway must never fall back from missing, failed, or stale native data to a
fixture that looks successful. Test/preview fixture adapters are build-profile
seams with visible `EvidenceSource::Fixture` and `PresentationMode::RoadmapPreview`;
production returns typed unavailable/degraded results when no authority exists.
`SettingsGateway` persists only validated non-secret preferences such as the
canonical `PaletteId`; it does not make CSS or renderer storage authoritative.

All mutations:

1. accept a typed intent and idempotency key where supported;
2. revalidate authority and fresh state natively;
3. return a typed operation identity;
4. report timeout as an unknown outcome;
5. reconcile by operation/transaction identity;
6. emit sanitized status events.

### 4.5 dApp and Help window isolation

- Help receives only locale, topic ID, section ID, and bundled documentation.
- Help has no wallet mutation commands and no secret-bearing event stream.
- The main app and Help use explicit stable window labels.
- The Tauri host/`PlatformGateway` creates or focuses the fixed `help` surface;
  the renderer receives no generic create-window command. Capability files list
  the exact `main`/`help` labels and minimum permissions rather than a wildcard.
- Closing one window does not terminate the other unless the user explicitly
  quits the application.
- Main and Help windows consume the same typed route/palette contracts but own
  separate ephemeral navigation, search, scroll, and focus stores; closing or
  navigating one cannot reset the other.
- MVP dApps are local catalogue descriptors and typed intents; they do not
  create remote webviews.
- Any later external dApp window requires a separate capability broker, CSP,
  origin policy, navigation policy, permission store, lifecycle policy, and
  security review.

### 4.6 Demo-to-Rust mapping

| Demo seam | Production mapping |
| --- | --- |
| `scripts/port/contracts.js` | Exhaustive Rust enums/DTOs in `z00z_wallet_ui_contract` |
| Navigation registry | Static typed `RouteSpec`/`NavNode` registry |
| Presentation state | Leptos signals/stores with no secret persistence |
| Deterministic fixture gateways | Deterministic Rust gateway implementations for tests and Storybook-like preview |
| `app.js` route/render branching | Leptos router plus workspace components |
| CSS token files | Packaged semantic design tokens with exactly two `PaletteId` mappings shared by components |
| Browser named Help tab | Tauri Help window/webview |
| Browser file/share simulation | Native `PlatformGateway` adapters |

### 4.7 Runtime, performance, and failure-isolation invariants

- **NFR-001**: Accordion open/close is a synchronous presentation-state
  operation. It triggers no gateway query, native command, route mutation, Help
  load, or analytics event.
- **NFR-002**: Only the active route mounts its data-bound workspace. Expanded
  navigation branches render metadata only; hidden Roadmap screens do not retain
  timers, subscriptions, large fixture collections, or background work.
- **NFR-003**: Every asynchronous query is scoped by route, wallet/context ID,
  and request generation/cancellation. A late response cannot overwrite a newer
  route, wallet, filter, lock, or logout state.
- **NFR-004**: Each workspace has a sanitized error boundary. A Watchers,
  Explorer, dApps, Messenger, Help, or palette-switch failure cannot remove the
  logo, navigation, lock action, active route identity, or access to another
  workspace.
- **NFR-005**: Phase 0 records interaction, route-mount, bundle, and memory
  baselines on desktop and a constrained mobile profile. Phase 9 rejects
  material unexplained regressions; virtualization or lazy loading is added only
  for measured collection pressure, not pre-emptively to the small nav tree.
- **NFR-006**: Diagnostic events use typed operation/error classes and coarse
  performance timing only. They contain no search term, message/contact content,
  wallet label, receiver, object opening, private route, or secret material and
  are never remotely exported by the renderer.

## 5. Implementation Steps

### Implementation Phase 0 — Baseline and contract freeze

- **GOAL-000**: Preserve current behavior and approve the route, capability,
  visual-continuity, icon, and two-palette contracts before shell changes.

| Task | Description | Completed |
| --- | --- | --- |
| TASK-001 | Capture fresh 1280×800, 1024×768, 768×1024, 390×844, and 320×800 baselines for Wallet, every current Telemetry route, Settings, Help, drawers, popups, and dialogs; also archive the current `z00z-default` token map and a dated live `z00z.io` Corporate screenshot/token snapshot. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-0/README.md#baseline-result) |
| TASK-002 | Inventory every current `PORT_CONTRACT` route, Help topic/directory, locale key, navigation/content icon, palette/theme ID, badge, gateway query, command, error, fixture, and smoke assertion. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-0/README.md#current-inventory-summary) |
| TASK-003 | Record current live/compatibility/target evidence from `UI-UX-REVIEW.md`, wallet contracts, aggregator/watchers code, Inbox spec, and OnionNet whitepaper. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-0/README.md#authority-and-maturity-ledger) |
| TASK-004 | Approve the tree in Section 2.1, mandatory interactive Roadmap branches, separate capability/presentation axes, unchanged Z00Z visual baseline, neutral navigation-icon contract, and exact two-palette registry. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-0/README.md#approved-contract-freeze) |
| TASK-005 | Add explicit supersession notes to the affected navigation, Help, and Appearance sections of `UI-UX-SPEC.md`, `UI-UX-REVIEW.md`, and `demo/PORTING.md`; do not leave the old top-tab, in-app contextual `HelpPanel`, theme-toggle, or four-palette contracts as competing authority. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-0/README.md#task-verdicts) |

Phase 0 freezes one pre-existing strict responsive failure: Wallet Assets
overflows at 1024 px. The baseline capture task is complete, but Phase 2 cannot
close until paired desktop/mobile evidence proves the overflow is removed.

### Implementation Phase 1 — Canonical route and navigation registry

- **GOAL-001**: Make navigation data-driven before changing visuals.

| Task | Description | Completed |
| --- | --- | --- |
| TASK-006 | Extend `scripts/port/contracts.js` with stable Wallet, Telemetry, dApps, Messenger, Contacts, Settings, Help, and action route enums. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-1/DOUBLECHECK.md#task-verdicts) |
| TASK-007 | Add `scripts/port/navigation-model.js` as the only demo owner of node ID, parent, order, label key, `IconId`, target, capability, `PresentationMode`, and Help topic metadata. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-1/DOUBLECHECK.md#task-verdicts) |
| TASK-008 | Replace the current single mixed capability state with independent `Maturity`, `Availability`, `EvidenceSource`, `Freshness`, and `PresentationMode` values. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-1/DOUBLECHECK.md#task-verdicts) |
| TASK-009 | Add pure validation proving unique node/route IDs, one parent, maximum depth three, no cycles, one default route per namespace, a valid neutral icon for every menu node, distinguishable sibling icons, exact two-palette IDs, valid locales, and exact route-to-Help coverage. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-1/DOUBLECHECK.md#task-verdicts) |
| TASK-009A | Reuse the current semantic icons wherever accurate and extend the canonical inline SVG registry only for genuinely missing glyphs such as `message`; use the existing outline/currentColor contract and add sprite-order/registry parity tests. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-1/DOUBLECHECK.md#task-verdicts) |
| TASK-010 | Extend presentation state with `active_route`, `expanded_branch_ids`, `drawer_open`, active wallet, canonical palette, request-generation/cancellation keys, and approved non-sensitive shell preferences; prove accordion changes issue no gateway call. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-1/DOUBLECHECK.md#task-verdicts) |
| TASK-011 | Add pure reducer tests for open, close, multi-open, active-child collapse, leaf selection, Back/Forward, wallet switch, lock, and logout. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-1/DOUBLECHECK.md#task-verdicts) |

### Implementation Phase 2 — Branded responsive shell

- **GOAL-002**: Replace the top-route-tab shell with one branded topbar and one
  responsive tree.

| Task | Description | Completed |
| --- | --- | --- |
| TASK-012 | Rebuild `AppShell` so the topbar spans the viewport and always renders the Z00Z logo/wordmark before sidebar/main content while preserving the existing Z00Z component geometry, typography, surfaces, and action hierarchy outside documented shell changes. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-2/DOUBLECHECK.md#phase-2-task-verdicts) |
| TASK-013 | Remove global route tabs and all code/CSS/tests that treat them as the application hierarchy. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-2/DOUBLECHECK.md#task-013-verdict) |
| TASK-014 | Render the canonical tree as a persistent desktop sidebar with independent root-only accordions, first-level workspace/action leaves, scroll containment, active path/descendant state, separators, non-occluding Log out, and a stable neutral-grey SVG icon on every visible route. Deeper routes render only in the selected workspace. Roadmap honesty remains in the destination; navigation never displays a `ROADMAP` badge. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-2/DOUBLECHECK.md#phase-2-task-verdicts) |
| TASK-015 | Render the same global tree as a mobile drawer beginning below the topbar; keep the logo visible and the drawer internally scrollable. Render every active workspace's local destination model as top horizontal tabs so switching local content never requires reopening the drawer. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-2/DOUBLECHECK.md#phase-2-task-verdicts) |
| TASK-016 | Implement focus containment, focus restoration, Escape/native Back/backdrop closure, inert background, safe-area sizing, and reduced-motion behavior. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-2/DOUBLECHECK.md#phase-2-task-verdicts) |
| TASK-017 | Add compact breadcrumbs/page title, status, selected-wallet context, privacy, attention, and lock utilities without creating a second navigation row. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-2/DOUBLECHECK.md#phase-2-task-verdicts) |
| TASK-018 | Delete the Assets and Wallet Settings mobile route popups. Keep their destinations in the workspace-local model: desktop vertical context rail and mobile/narrow-tablet top tabs, with identical order and route IDs. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-2/DOUBLECHECK.md#phase-2-task-verdicts) |
| TASK-018A | Replace the separate theme toggle and four-palette registry with compact `z00z-default`/`z00z-corporate` cards containing only swatch/name/one `ACTIVE` marker, legacy preference migration, immediate selection, locally vendored Corporate tokens, and shared app/Help palette propagation. | ✅ 2026-07-26; interaction and copy simplified 2026-07-27 — [evidence](demo/evidence/phase-2/DOUBLECHECK.md#verdict) |

### Implementation Phase 3 — Wallet migration

- **GOAL-003**: Preserve the completed wallet experience under the new route
  hierarchy.

| Task | Description | Completed |
| --- | --- | --- |
| TASK-019 | Delete Wallet Overview and make Assets & Rights the wallet default. Move Assets, Vouchers, Permissions, capability-gated Quarantine, Send, Receive, History, Swap, Exchange, Staking, Backup, and Wallet Settings into the new route registry without changing wallet ownership boundaries. Assets/Vouchers/Permissions and Wallet Settings children stay out of the global tree and use the shared responsive local-destination model. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-3/DOUBLECHECK.md#verdict) |
| TASK-020 | Keep selected-wallet switching above the tree and verify every wallet-owned collection remains keyed by wallet ID. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-3/DOUBLECHECK.md#verdict) |
| TASK-021 | Preserve pending/reconciliation badges, deep links, form drafts, dialogs, and native Back order across the shell migration. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-3/DOUBLECHECK.md#verdict) |
| TASK-022 | Keep Swap/Staking compatibility and Exchange target states typed in the capability model while presenting unavailable state inside the functional flow, without a repeated capability-summary banner. | ✅ 2026-07-26; banner removed 2026-07-27 — [evidence](demo/evidence/phase-3/DOUBLECHECK.md#verdict) |
| TASK-023 | Integrate the useful preview/progress/error patterns from `Z00Z-App-TODO/views/` behind current route and native adapter boundaries. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-3/DOUBLECHECK.md#verdict) |

### Implementation Phase 4 — Telemetry expansion

- **GOAL-004**: Replace Network with a privacy-safe, evidence-driven Telemetry
  workspace.

| Task | Description | Completed |
| --- | --- | --- |
| TASK-024 | Migrate Reticulum, OnionNet, and Aggregators from topbar tabs/old Network routes into first-level workspaces inside the root Telemetry accordion. Render each component's deeper routes through its shared desktop context rail/mobile top tabs; no nested Telemetry accordion or deep global-tree row is permitted. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-4/DOUBLECHECK.md#verdict) |
| TASK-025 | Add Aggregators Ingress, Planning, Placement, Publication, and Recovery concept screens mapped to the runtime-owned boundaries. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-4/DOUBLECHECK.md#verdict) |
| TASK-026 | Add a deterministic `MockTelemetryGateway` that returns typed maturity, availability, source, freshness, presentation mode, loading, success, degraded, unavailable, empty, malformed, and error states. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-4/DOUBLECHECK.md#verdict) |
| TASK-027 | Add a fully interactive, always-visible Watchers Roadmap preview with Overview, Alerts, Publication checks, DA providers, Censorship signals, Evidence export, filters, detail/recovery states, and mappings to current watcher modules. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-4/DOUBLECHECK.md#verdict) |
| TASK-028 | Add a fully interactive, always-visible Explorer Roadmap preview with Overview, supported public-ID Search, Checkpoints, Batches, Public evidence, privacy-safe detail, and fail-closed recovery states. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-4/DOUBLECHECK.md#verdict) |
| TASK-029 | Add Watcher alert → Explorer evidence deep links that carry only public typed identifiers. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-4/DOUBLECHECK.md#verdict) |
| TASK-030 | Add redaction assertions proving Telemetry never renders fixture/private wallet labels, receivers, counterparties, memos, route paths, inbox records, or secret material. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-4/DOUBLECHECK.md#verdict) |

### Implementation Phase 5 — dApps and permission center

- **GOAL-005**: Demonstrate bounded Z00Z application use cases without
  pretending to ship a universal VM or remote app runtime.

| Task | Description | Completed |
| --- | --- | --- |
| TASK-031 | Add local curated dApp descriptors for Offline Pay, Private Voucher, External Asset Locker, Scoped Expenses, Service Credits, and Agent Budget. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-5/DOUBLECHECK.md#verdict) |
| TASK-032 | Implement the always-visible dApps Roadmap preview: Discover, Installed, Connections, Permissions, Activity, dApp detail, accepted/rejected review, revoke/expiry, and outcome routes using deterministic local fixtures only. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-5/DOUBLECHECK.md#verdict) |
| TASK-033 | Implement one reusable permission-review flow with intent, scope, uses, expiry, delegation, value, fee, disclosure, revoke, and re-auth fields. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-5/DOUBLECHECK.md#verdict) |
| TASK-034 | Route accepted demo intents into existing Wallet review flows; do not let dApp presentation state mutate wallet objects. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-5/DOUBLECHECK.md#verdict) |
| TASK-035 | Add security tests rejecting generic signing, arbitrary URLs, unknown intent types, broader-than-held Permissions, hidden value/fee, and remote resource loading. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-5/DOUBLECHECK.md#verdict) |

### Implementation Phase 6 — Messenger and Contacts

- **GOAL-006**: Demonstrate private request coordination while preserving
  wallet, Inbox, OnionNet, and settlement boundaries.

| Task | Description | Completed |
| --- | --- | --- |
| TASK-036 | Add typed local fixtures for advisory messages, payment requests, voucher/claim/permission proposals, receiver-card invitations, delivery receipts, expiry, abuse, and unavailable relay state. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-6/DOUBLECHECK.md#verdict) |
| TASK-037 | Implement the always-visible Messenger Roadmap preview: Inbox, Requests, Conversations, Outbox, Receipts, message/request detail, expiry/delete/block/report, accepted/rejected Wallet handoff, and unavailable-relay recovery controls. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-6/DOUBLECHECK.md#verdict) |
| TASK-038 | Prove that opening, deleting, acknowledging, blocking, or reporting a message never mutates Wallet state or changes settlement status. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-6/DOUBLECHECK.md#verdict) |
| TASK-039 | Route an accepted request through the correct Wallet review/gateway intent and revalidate it there. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-6/DOUBLECHECK.md#verdict) |
| TASK-040 | Implement Contacts search, add/import/QR/native-share concepts, detail, edit label, identity-change review, expiry/revocation, and local removal. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-6/DOUBLECHECK.md#verdict) |
| TASK-041 | Add privacy tests for redacted logs, no public presence/address graph, no implicit trust, and no contact upload/network call. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-6/DOUBLECHECK.md#verdict) |

### Implementation Phase 7 — Help and localization migration

- **GOAL-007**: Keep every new screen understandable and independently
  accessible in all locales.

| Task | Description | Completed |
| --- | --- | --- |
| TASK-042 | For all ten locales, copy Help into its canonical topic directory, migrate the runtime catalogue from `help/<locale>/network/` to `help/<locale>/telemetry/`, add `dapps`, `messenger`, and `contacts`, and preserve every pre-existing Markdown at its original path through `preserved-sources.json`. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-7/DOUBLECHECK.md#phase-7-task-verdicts) |
| TASK-043 | Extend `help/topics.yaml` so every route/detail/dialog has exactly one contextual topic and every topic resolves back to one supported context. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-7/DOUBLECHECK.md#phase-7-task-verdicts) |
| TASK-044 | Add English canonical topics, synchronize the nine translations through the existing local build-time translation boundary, and require native-language review hashes. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-7/DOUBLECHECK.md#phase-7-task-verdicts) |
| TASK-045 | Update standalone Help to consume canonical route labels and capability explanations, use multi-open root-only groups with first-level documentation workspaces, keep that global tree visible on desktop/in the mobile drawer, and expose the selected workspace's topics as a desktop context rail/mobile top tabs. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-7/DOUBLECHECK.md#phase-7-task-verdicts) |
| TASK-046 | Verify global Help and contextual `?` reuse/focus the same independent browser tab and preserve app state; retain equivalent independent-window acceptance criteria for the future native port. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-7/DOUBLECHECK.md#phase-7-task-verdicts) |
| TASK-047 | Add every new label, Roadmap presentation-state explanation, palette name/action, empty/loading/error state, permission field, and accessible name to the canonical locale registry with exact parity checks; do not add a navigation/UI `ROADMAP` badge. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-7/DOUBLECHECK.md#phase-7-task-verdicts) |

### Future Phase 8 — Tauri/Leptos production port (outside demo scope)

- **GOAL-008**: Preserve explicit acceptance criteria for a separately
  authorized future production port. This phase is not executed by the pure-JS
  demo plan.

| Task | Description | Completed |
| --- | --- | --- |
| TASK-048 | Create or update the proposed `z00z_wallet_ui_contract`, `z00z_wallet_ui`, and `z00z_wallet_ui_tauri` seams only after confirming their canonical workspace locations. | ⏸ DEFERRED — requires separate production-port authorization |
| TASK-049 | Port the route, navigation, five-axis capability/presentation, exact two-variant `PaletteId`, Help-topic, command, query, event, and error enums to Rust with exhaustive compile-time matches. | ⏸ DEFERRED — future Rust implementation |
| TASK-050 | Implement a minimal Leptos shell rendering the persistent logo, desktop sidebar, mobile drawer, multi-open reducer, and one deterministic route. | ⏸ DEFERRED — future Leptos implementation |
| TASK-051 | Open/focus a fixed, host-owned branded Help surface and verify independent focus/switch, close, restore, locale, CSP, exact window capability labels, Android Activity/iOS `UIScene` setup, runtime multi-window support, and packaged-asset behavior on Windows, Linux, real iPhone, and iPad; no generic renderer window creation or modal fallback satisfies this task. | ⏸ DEFERRED — future native platform evidence |
| TASK-052 | Verify Windows/Linux authenticated local IPC and the iOS typed in-process adapter can satisfy the same gateway contract without exposing raw RPC to views. | ⏸ DEFERRED — future native gateway implementation |
| TASK-053 | Verify CSP, no remote assets, offline cold start, secure lifecycle clearing, native file/share/clipboard boundaries, independent workspace failure isolation, stale-response cancellation, updater/signing feasibility, and bundle support before pinning versions. | ⏸ DEFERRED — future packaged-runtime evidence |

### Implementation Phase 9 — Verification and documentation closeout

- **GOAL-009**: Produce automated and visual evidence before marking the plan
  complete.

| Task | Description | Completed |
| --- | --- | --- |
| TASK-054 | Extend contract, locale, Help, port-readiness, Pages-release, smoke, route-mount, stale-response, workspace-error-boundary, and baseline-regression gates for every new route and capability state. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-9/DOUBLECHECK.md#task-verdicts) |
| TASK-055 | Capture and inspect the complete viewport/locale/state matrix defined in Section 8 for exactly Z00Z Default and Z00Z Corporate, including neutral menu icons and every mandatory Roadmap flow. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-9/DOUBLECHECK.md#task-verdicts) |
| TASK-056 | Run keyboard, screen-reader, touch-target, 200% text zoom, reduced-motion, safe-area, software-keyboard, and Back/Escape tests. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-9/DOUBLECHECK.md#task-verdicts) |
| TASK-057 | Run privacy/security assertions for renderer fields, logs, Help payloads, dApp intents, Messenger data, Contacts, Telemetry, Explorer, and native command allowlists. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-9/DOUBLECHECK.md#task-verdicts) |
| TASK-058 | Update `demo/README.md`, `demo/PORTING.md`, `UI-UX-SPEC.md`, `UI-UX-REVIEW.md`, and Help maintenance documentation so no obsolete top-tab/Network, in-app contextual `HelpPanel`, independent theme-mode, removed-palette, optional-roadmap, or coloured-menu-icon contract remains. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-9/DOUBLECHECK.md#task-verdicts) |
| TASK-059 | Run `git diff --check`, scoped tests, visual review, and a complete diff audit; preserve unrelated worktree changes. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-9/DOUBLECHECK.md#task-verdicts) |
| TASK-060 | Mark the pure-JS demo plan `Completed` only after every in-scope task has dated evidence and all browser-demo review gates pass; Future Phase 8 remains separately deferred. | ✅ 2026-07-26 — [evidence](demo/evidence/phase-9/DOUBLECHECK.md#task-verdicts) |

## 6. Alternatives

- **ALT-001 — Keep sidebar macro routes plus topbar route tabs. Rejected.**
  Persistent global route tabs create a second application hierarchy and become
  harder to scale across Telemetry, dApps, and Messenger. This does not reject
  workspace-local mobile tabs: those tabs project only the selected workspace's
  local destination model.
- **ALT-002 — Put every control in one permanently expanded tree. Rejected.**
  It creates an expert sitemap rather than a usable application. Use root-only
  accordions, first-level workspace leaves, and a visible local navigation
  surface for deeper routes.
- **ALT-003 — Use a bottom navigation bar on mobile. Rejected.** It duplicates
  the same routes, cannot represent the expanded hierarchy, and has already
  caused layout/content occlusion.
- **ALT-004 — Keep horizontal global tabs only on mobile. Rejected.** It breaks
  global information-architecture parity and does not scale to long locales or
  Telemetry. Horizontal mobile tabs are required only for the active workspace's
  local destinations.
- **ALT-005 — Make each desktop accordion exclusive. Rejected.** The requested
  behavior and expert comparison workflows require several open branches;
  exclusive accordions also diverge from mobile.
- **ALT-006 — Turn Help into a modal. Rejected.** Users must compare Help and
  application state, switch between them, and close either independently.
- **ALT-007 — Execute remote dApps in the main renderer. Rejected.** It would
  enlarge the trusted UI surface and expose the wallet bridge to untrusted code.
- **ALT-008 — Present Messenger as permanent on-chain chat. Rejected.** It
  conflicts with OnionNet's off-chain, short-lived relay boundary and would
  create a public communication graph.
- **ALT-009 — Give Explorer access to wallet-local data. Rejected.** Explorer is
  intentionally limited to public evidence and must not become a privacy leak.
- **ALT-010 — Collapse maturity and availability into one status. Rejected.**
  Live backend code can be unavailable to the app, while a complete demo
  fixture can represent only a target concept.

## 7. Dependencies and Files

### 7.1 Dependencies

- **DEP-001**: Existing `PORT_CONTRACT`, presentation store, deterministic
  fixtures, locale registry, Help compiler, and Help topic LUT.
- **DEP-002**: Existing dark/gold design tokens, bundled Geist fonts, SVG icon
  registry, and canonical Z00Z logo.
- **DEP-002A**: A dated source snapshot of the live `z00z.io` Corporate semantic
  palette. It is design provenance only; the packaged app has no runtime site
  dependency.
- **DEP-003**: Existing Playwright smoke/visual-review harness and GitHub Pages
  deterministic release builder.
- **DEP-004**: `UI-UX-SPEC.md`, `UI-UX-REVIEW.md`, and `demo/PORTING.md`.
- **DEP-005**: Runtime Aggregators and Watchers boundaries for accurate
  Telemetry naming.
- **DEP-006**: Request-Bound Inbox and OnionNet specifications for Messenger
  privacy and authority boundaries.
- **DEP-007**: Use Cases and Smart Cash whitepapers for curated dApp families
  and the no-universal-VM boundary.
- **DEP-008**: Tauri/Leptos production spike before dependency version pinning.

### 7.2 Expected demo files

- **FILE-001**: `crates/z00z_ui_ux/DEMO-PLAN-2.md`
- **FILE-002**: `crates/z00z_ui_ux/demo/scripts/port/contracts.js`
- **FILE-003**: `crates/z00z_ui_ux/demo/scripts/port/navigation-model.js`
- **FILE-004**: `crates/z00z_ui_ux/demo/scripts/port/presentation-state.js`
- **FILE-005**: `crates/z00z_ui_ux/demo/scripts/port/mock-wallet-gateway.js`
- **FILE-006**: `crates/z00z_ui_ux/demo/scripts/port/mock-telemetry-gateway.js`
- **FILE-007**: `crates/z00z_ui_ux/demo/scripts/port/mock-dapp-gateway.js`
- **FILE-008**: `crates/z00z_ui_ux/demo/scripts/port/mock-messenger-gateway.js`
- **FILE-009**: `crates/z00z_ui_ux/demo/scripts/port/mock-contacts-gateway.js`
- **FILE-010**: `crates/z00z_ui_ux/demo/app.js`
- **FILE-011**: `crates/z00z_ui_ux/demo/index.html`
- **FILE-012**: `crates/z00z_ui_ux/demo/styles/foundation.css`
- **FILE-013**: `crates/z00z_ui_ux/demo/styles/components.css`
- **FILE-013A**: `crates/z00z_ui_ux/demo/styles/colors.css`
- **FILE-013B**: `crates/z00z_ui_ux/demo/scripts/port/icon-registry.js`
- **FILE-014**: `crates/z00z_ui_ux/demo/help/topics.yaml`
- **FILE-015**: `crates/z00z_ui_ux/demo/help/<locale>/{app,wallets,telemetry,dapps,messenger,contacts,settings}/`
- **FILE-016**: `crates/z00z_ui_ux/demo/locales/*.js`
- **FILE-017**: `crates/z00z_ui_ux/demo/scripts/test-port-contracts.mjs`
- **FILE-018**: `crates/z00z_ui_ux/demo/scripts/check-port-readiness.mjs`
- **FILE-019**: `crates/z00z_ui_ux/demo/smoke.spec.js`
- **FILE-020**: `crates/z00z_ui_ux/demo/visual-review.spec.js`
- **FILE-021**: `crates/z00z_ui_ux/demo/README.md`
- **FILE-022**: `crates/z00z_ui_ux/demo/PORTING.md`
- **FILE-023**: `.planning/phases/110-Wallet-UX-UI/UI-UX-SPEC.md`
- **FILE-024**: `.planning/phases/110-Wallet-UX-UI/UI-UX-REVIEW.md`

Confirm every proposed new filename against the live tree before creation. Do
not create duplicate gateway, route, locale, Help, or design-token owners.

## 8. Testing and Acceptance Matrix

### 8.1 Navigation and brand

- **TEST-001**: Z00Z logo/wordmark is visible on onboarding, create/recover,
  unlock, lock, empty-wallet, authenticated, error, and Help states at 1440,
  1280, 1024, 768, 390, and 320 CSS px.
- **TEST-002**: Logo remains visible while the drawer, Help launcher,
  notification panel, sheet, and dialog are open.
- **TEST-003**: No global route tabs or duplicate bottom navigation are present.
- **TEST-003A**: Pixel/DOM review proves the existing Z00Z Default semantic
  tokens and unchanged content components retain their baseline appearance
  outside the explicitly approved topbar/sidebar/drawer migration.
- **TEST-004**: Desktop and mobile render identical canonical node order and
  localized labels; every visible menu node has a valid neutral-grey semantic
  icon, sibling destinations are distinguishable, and no destination gets a
  coloured icon tile.
- **TEST-005**: Open Wallet and Telemetry root accordions, close Wallet, and
  verify Telemetry remains open on desktop and mobile.
- **TEST-006**: Reticulum, OnionNet, Aggregators, Watchers, and Explorer render
  as first-level workspace leaves without chevrons, `aria-expanded`, or deep
  global-tree rows; every deeper route remains selectable through the workspace
  context rail/top tabs.
- **TEST-007**: Toggling a root accordion never navigates and mobile waits for a
  leaf selection.
- **TEST-008**: Selecting a mobile leaf closes the drawer, changes the route,
  and leaves the saved expansion set intact.
- **TEST-009**: Exactly one leaf has `aria-current="page"`; active ancestors are
  discoverable without pretending to be selected routes.
- **TEST-010**: Back/Forward and native Back restore route/overlay order without
  resetting unrelated open branches.
- **TEST-010A**: Expanding/collapsing any root accordion changes presentation
  state only:
  no route, gateway call, subscription, Help load, or active workspace mount is
  created or destroyed.
- **TEST-010B**: Assets & Rights, Send, Wallet Settings, and Help local
  destinations are visible as a vertical rail on desktop and a horizontal top
  tab row on mobile; switching them does not open the global drawer.
- **TEST-010C**: The immutable navigation model rejects a workspace below
  anything except a root branch and rejects non-route children inside a
  workspace. Every valid workspace derives one ordered local destination list
  used by both desktop and mobile renderers.

### 8.2 Capability and domain honesty

- **TEST-011**: Every capability-bearing screen exposes valid maturity,
  availability, evidence source, freshness, and presentation mode, while pure
  local Settings/Help/navigation screens render no meaningless capability badge.
- **TEST-011A**: Capability headers show at most two compact labels, Roadmap uses
  a neutral treatment, remaining axes are readable as metadata/disclosure, and
  maturity is never encoded as success/warning/danger colour.
- **TEST-012**: Fixtures never render as native/live evidence.
- **TEST-013**: Missing values render Unavailable, not zero/healthy/success.
- **TEST-013A**: Watchers, Explorer, dApps, and Messenger are always visible and
  selectable in the demo; every specified route and complete Roadmap scenario
  works against deterministic fixtures and no route renders a generic disabled
  or `Coming soon` placeholder.
- **TEST-013B**: Every recommended story has an obvious entry, one dominant
  primary action per step, Back/Cancel, required review, actionable
  success/error outcome, and no dead end.
- **TEST-014**: Watcher alerts cannot modify planner, validator, storage,
  wallet, or settlement state.
- **TEST-015**: Explorer accepts only supported public identifiers and contains
  no wallet-private fixture fields.
- **TEST-016**: dApps cannot call generic signing, raw RPC, filesystem, URL, or
  wallet-secret surfaces.
- **TEST-017**: Messenger actions are advisory until an explicit Wallet review
  intent is invoked and revalidated.
- **TEST-018**: Contacts remain local, do not imply verification, and produce no
  renderer network request.
- **TEST-018A**: Quarantined objects never enter Available; the Quarantine leaf
  appears only when its count/capability rule is satisfied and exposes only
  safe review, discard, retry, or diagnostics actions.

### 8.3 Responsive and accessibility

- **TEST-019**: No document-level horizontal overflow at 320 CSS px or 200%
  text zoom.
- **TEST-020**: Sidebar/drawer rows, toggles, leaves, close, Help, and Log out
  have 44 × 44 CSS px minimum targets.
- **TEST-021**: Focus enters the drawer, remains contained, and returns to Menu.
- **TEST-022**: Screen readers announce branch label/state, route label/current
  state, badge meaning, and external Help behavior.
- **TEST-023**: Long German, Russian, French, Portuguese, Turkish, Japanese,
  Korean, and Chinese labels wrap or truncate without hiding state or actions.
- **TEST-024**: Reduced motion removes nonessential transitions without
  concealing open/closed state.
- **TEST-025**: iOS safe areas and the software keyboard do not cover the
  topbar, active input, primary action, or final content.
- **TEST-025A**: Appearance exposes exactly `z00z-default` and
  `z00z-corporate`; Default is dark, Corporate is light, no independent
  System/Dark/Light selector exists, all removed IDs are absent, legacy
  preferences migrate deterministically, each card applies immediately, exactly
  one `ACTIVE` marker is visible, and app/Help use the same canonical selection.
- **TEST-025B**: Z00Z Corporate's local semantic mapping is traceable to the
  dated source snapshot and both palettes pass automated text/control/focus and
  success/warning/danger contrast checks without changing the gold logo.

### 8.4 Help, offline, and Tauri

- **TEST-026**: Every route/detail/dialog resolves exactly one contextual Help
  topic and every contextual topic maps back to a supported state.
- **TEST-027**: All locale directories have exact canonical topic parity and
  synchronized English source hashes; no runtime topic is loose or misplaced,
  and every original tracked Markdown listed in `preserved-sources.json`
  remains present at its original path.
- **TEST-028**: Global and contextual Help reuse one independent Help
  browser tab; on desktop and mobile either surface can close while the other
  remains open with its route, expansion state, filters, and drafts. The future
  native port must separately prove equivalent iPhone scene switching and iPad
  multi-window behavior; it does not require side-by-side phone display.
- **TEST-029**: Help has its own visible Z00Z topbar.
- **TEST-030**: The pure-JS demo cold-starts offline with all assets, fonts,
  icons, locales, fixtures, and Help present. Packaged-runtime verification is
  deferred to Future Phase 8.
- **TEST-031**: Renderer readiness checks reject remote URLs and browser network
  APIs.
- **TEST-032**: The JS port contract rejects unknown native-command intents,
  unknown fields, oversized payloads, raw secrets, arbitrary paths, and
  arbitrary URLs; the future Tauri host must implement the same allowlist.
- **TEST-033**: Demo lifecycle lock/background/suspend simulation clears
  sensitive presentation state in both application and Help surfaces; native
  lifecycle integration is deferred to Future Phase 8.
- **TEST-033A**: Late async responses are ignored after route/wallet/filter/lock
  changes, only the active data workspace remains mounted, and an injected
  failure in any Roadmap or Help workspace leaves the branded shell and other
  routes usable.

### 8.5 Visual review matrix

Every visual verification gate captures and inspects both desktop/tablet and
mobile evidence. A desktop-only or mobile-only screenshot set cannot close a
task whose behavior is shared by both layouts.

Capture at minimum:

- 1280×800 desktop for every root and every second-level branch;
- 1024×768 compact desktop for the deepest Telemetry paths;
- 768×1024 narrow tablet with drawer and long navigation;
- 390×844 and 320×800 for every canonical route;
- open/closed/multi-open drawer states;
- active leaf inside a collapsed ancestor;
- loading, empty, degraded, unavailable, target, concept, malformed, and error
  states;
- RU, DE, FR, PT, TR, JA, KO, and ZH long-label cases;
- global Help, contextual Help, dApp permission review, Messenger request,
  Contact identity change, Watcher alert, and Explorer detail;
- exactly Z00Z Default and Z00Z Corporate, including palette previews,
  application/Help propagation, neutral navigation icons, semantic states, and
  the legacy-preference migration result;
- 200% text zoom and reduced motion.

Every captured image is reviewed for logo visibility, hierarchy, active state,
overlap, clipping, scroll containment, density, contrast, focus, privacy
redaction, and capability honesty.

## 9. Risks and Assumptions

- **RISK-001 — Navigation becomes too long.** Mitigation: root-only
  accordions, first-level workspace leaves, workspace-local context navigation,
  scroll containment, active-ancestor indication, Wallet open by default,
  Roadmap roots collapsed by default, and no empty/disabled demo branches.
- **RISK-002 — Important Wallet routes become less discoverable.** Mitigation:
  Wallet opens by default, Assets & Rights is the default route, its local
  destinations stay visible as a desktop rail/mobile top tabs, and quick actions
  deep-link to stable destinations.
- **RISK-003 — Maturity and availability are confused.** Mitigation: separate
  typed axes, fixture provenance, tests, and canonical copy.
- **RISK-004 — Telemetry overclaims global truth.** Mitigation: show observed
  layer, source, freshness, scope, and explicit authority boundaries.
- **RISK-005 — Explorer leaks privacy.** Mitigation: public-evidence DTO
  allowlist, deny-by-default rendering, redaction tests, and no wallet gateway
  dependency.
- **RISK-006 — dApps enlarge the trusted renderer.** Mitigation: local
  descriptors only in MVP, typed intents, no arbitrary code, no generic signing,
  and separate future-webview security work.
- **RISK-007 — Messenger creates a stable social graph.** Mitigation: local
  contacts, rotating/scoped receive material, short retention, no public
  presence, and no permanent inbox claim.
- **RISK-008 — Inbox UI is mistaken for authority.** Mitigation: advisory labels,
  explicit Wallet review, gateway revalidation, and mutation-invariance tests.
- **RISK-009 — Help taxonomy and route taxonomy drift.** Mitigation: derive both
  from the route registry and fail builds on incomplete one-to-one coverage.
- **RISK-010 — Large scope delays the core navigation fix.** Mitigation: P0/P1
  phases land and verify before P2 concept modules.
- **RISK-011 — CSS works in a browser but fails in a WebView.** Mitigation:
  preserve dynamic viewport/safe-area tests and local assets in the JS demo,
  then require an explicitly authorized packaged Tauri spike and
  iOS/Windows/Linux review during the future production port.
- **RISK-012 — Renderer state becomes a second authority.** Mitigation:
  presentation-only state, typed gateways, native revalidation, and no browser
  persistence for secrets or operations.
- **RISK-013 — Reference screenshots dilute Z00Z identity.** Mitigation: borrow
  interaction patterns only; preserve Z00Z tokens, type, logo, copy, object
  model, and calm product character.
- **RISK-014 — Roadmap polish is mistaken for shipped protocol support.**
  Mitigation: persistent `Roadmap preview`, five independent capability/source
  fields, deterministic-fixture labels, no native claim, and module-specific
  Help.
- **RISK-015 — One icon per menu node becomes visual noise or a rainbow.**
  Mitigation: one outline family, normalized grid/stroke, neutral grey
  `currentColor`, sibling semantic validation, no coloured tiles, and colour
  reserved for selection/status/object meaning.
- **RISK-016 — The live Corporate site changes after implementation.**
  Mitigation: retain the dated source token/screenshot provenance, vendor the
  approved mapping locally, and change it only through a new reviewed palette
  revision—never at runtime.
- **RISK-017 — Legacy theme/palette preferences create unsupported combinations.**
  Mitigation: one canonical two-variant enum, deterministic one-time migration,
  invalid-value fallback, removal of old CSS selectors, and contract tests.
- **RISK-018 — Hidden workspaces consume resources or stale results overwrite
  current state.** Mitigation: active-route-only mounting, scoped request
  generations/cancellation, workspace error boundaries, lifecycle clearing, and
  constrained-mobile memory/interaction baselines.
- **ASSUMPTION-001**: English remains the canonical technical and Help source;
  localized user-facing content is generated and reviewed for all ten locales.
- **ASSUMPTION-002**: The current GitHub Pages demo remains a review artifact,
  while Tauri is the only production runtime target.
- **ASSUMPTION-003**: Watchers, Explorer, dApps, and Messenger are mandatory
  visible Roadmap previews in the demo and carry persistent, accurate
  presentation/maturity/availability/source labels.
- **ASSUMPTION-004**: No application route receives protocol or settlement
  authority merely because it is visible or enabled in the UI.

## 10. Plan Review Gates

This plan is approved for implementation only when all gates pass.

- **REVIEW-001**: Every user-requested root item appears in the exact required
  order, with Telemetry replacing Network.
- **REVIEW-002**: The Z00Z topbar logo requirement maps to implementation tasks
  and automated/visual tests at every viewport.
- **REVIEW-003**: Desktop and mobile use one global registry and the same
  independent root-accordion reducer; each workspace uses one local destination
  model projected as a desktop rail or mobile top tabs.
- **REVIEW-004**: Global top tabs and mobile popup navigation have an explicit
  removal/migration path.
- **REVIEW-005**: Every proposed Telemetry leaf maps to an actual repository
  implementation or named specification concept, states which one it is, and
  declares its authority/privacy boundary.
- **REVIEW-006**: dApps and Messenger claims remain within Smart Cash,
  Request-Bound Inbox, and OnionNet boundaries.
- **REVIEW-007**: Explorer cannot consume wallet-private DTOs.
- **REVIEW-008**: Help remains separate, parallel, localized, route-complete,
  offline, and independently closable.
- **REVIEW-009**: The route, navigation, capability, Help, locale, icon, and
  gateway models each have one canonical owner.
- **REVIEW-010**: Tauri commands are intent-level and no raw RPC/network or
  arbitrary signing/filesystem surface reaches views.
- **REVIEW-011**: Every phase has deterministic acceptance evidence and does not
  rely only on screenshots.
- **REVIEW-012**: Existing completed wallet behavior and unrelated worktree
  changes are preserved.
- **REVIEW-013**: The work is an evolution of the existing Z00Z design, not a
  rebrand; Z00Z Default is visually frozen, Corporate reuses the same component
  system, and every menu node has a neutral-grey semantic icon with no coloured
  destination tiles.
- **REVIEW-014**: Appearance has exactly the two canonical palette IDs,
  deterministic legacy migration, no independent System/Dark/Light selector,
  local-only assets, and automated semantic contrast evidence.
- **REVIEW-015**: Watchers, Explorer, dApps, and Messenger are mandatory,
  selectable, complete Roadmap preview flows rather than optional branches,
  disabled controls, or generic `Coming soon` pages.
- **REVIEW-016**: All ten Help locales have exact canonical topic-directory and
  catalogue parity, no loose runtime topic, and complete retention of the
  original tracked Markdown while the runtime catalogue uses Telemetry.
- **REVIEW-017**: Every main and Roadmap flow has clear entry/exit, one dominant
  primary action, progressive disclosure, actionable errors, and no dead end;
  search remains local to its current collection.
- **REVIEW-018**: Accordion state has no gateway side effects, only the active
  data workspace mounts, stale async results cannot cross route/wallet/lifecycle
  changes, and workspace failures remain isolated from the branded shell.

### Review result

- **REVIEW-RESULT-001**: Pre-implementation document review passed on
  2026-07-26 after resolving the optional-roadmap, old theme/palette, coloured
  menu-icon, visual-redesign, Help-layout, and presentation-state ambiguities.
- **REVIEW-RESULT-002**: Browser-demo implementation may begin at Phase 0. No
  task may skip the baseline freeze or specification-supersession work.
- **REVIEW-RESULT-003**: Native Tauri dependency selection and production Help
  implementation remain gated by the Windows/Linux/iOS Phase 8 spike. In
  particular, independent iOS scene/window behavior is unverified until that
  spike passes; no modal fallback satisfies the requirement.

## 11. Definition of Done

The plan is complete only when:

- one JavaScript navigation registry drives desktop, mobile, Help, routes, and
  tests, and supplies the documented future Rust port contract;
- the Z00Z logo/wordmark remains visible in the topbar in every application and
  Help state at every required viewport;
- existing Z00Z Default visuals remain the dark baseline, Z00Z Corporate is the
  only light application palette, exactly two palette IDs exist, and legacy
  theme/palette combinations migrate deterministically;
- every menu destination has a stable neutral-grey semantic icon while existing
  content/object/status colours retain their bounded meanings;
- the topbar contains no global route tabs;
- multi-open root-accordion behavior works identically on desktop and mobile,
  including closing one root while preserving another and waiting for an
  explicit leaf selection;
- workspace-local destinations remain visible as a vertical desktop rail and
  horizontal mobile top tabs without duplicating the global hierarchy;
- Wallet behavior and wallet-local object boundaries remain intact;
- capability-gated Quarantine safely exposes unsupported/unsafe objects without
  treating them as spendable value;
- Telemetry replaces Network and includes Reticulum, OnionNet, Aggregators,
  Watchers, and Explorer with honest evidence/provenance;
- Watchers, Explorer, dApps, and Messenger remain visible, selectable,
  fully interactive Roadmap previews with deterministic complete flows and no
  disabled/`Coming soon` substitutes;
- each flow has clear entry/exit, one dominant primary action per step,
  progressive disclosure, required review, and actionable success/error states;
- dApps, Messenger, and Contacts implement the bounded flows in this plan
  without remote code, false protocol claims, or privacy leaks;
- Settings contains General and Appearance, while wallet settings remain
  wallet-scoped;
- Help is separate, parallel, contextual, offline, and complete in all ten
  locales, with every Markdown topic stored under its canonical topic directory;
- accessibility, responsive, offline, privacy, contract, and visual gates pass;
- accordion toggles remain side-effect free, only the active data workspace is
  mounted, stale asynchronous results are rejected, and workspace failures stay
  isolated from the shell;
- the JS porting contract records the future Tauri/Leptos shell, typed boundary,
  independent desktop/mobile Help, CSP, lifecycle, and packaged-asset
  acceptance criteria without implementing Rust/Tauri in this demo;
- all affected specifications agree and no obsolete navigation contract remains;
- `git diff --check` and the full scoped verification suite pass;
- this document is updated with dates and evidence, then marked `Completed`.

## 12. Requirement Traceability

| Requirement family | Primary implementation tasks | Primary acceptance tests |
| --- | --- | --- |
| Persistent Z00Z topbar brand and shell | TASK-012 through TASK-018 | TEST-001 through TEST-003, TEST-029 |
| Unified desktop/mobile multi-open navigation | TASK-006 through TASK-018 | TEST-004 through TEST-010 |
| Preserved visual baseline and neutral menu icons | TASK-001, TASK-002, TASK-004, TASK-007, TASK-009, TASK-009A, TASK-012, TASK-014, TASK-055 | TEST-003A, TEST-004, Section 8.5 |
| Exactly two application palettes and legacy migration | TASK-001, TASK-002, TASK-004, TASK-005, TASK-018A, TASK-047, TASK-055, TASK-058 | TEST-025A, TEST-025B, Section 8.5 |
| Separate maturity, availability, source, freshness, and presentation | TASK-003, TASK-008, TASK-009, TASK-022, TASK-026 | TEST-011 through TEST-013A |
| Wallet object/lifecycle boundaries and Quarantine | TASK-019 through TASK-023 | TEST-017, TEST-018A |
| Mandatory Watchers and Explorer Roadmap previews | TASK-024 through TASK-030 | TEST-011 through TEST-015, TEST-013A |
| Mandatory bounded dApps Roadmap and permission review | TASK-031 through TASK-035 | TEST-013A, TEST-016, TEST-032 |
| Mandatory advisory Messenger Roadmap and local Contacts | TASK-036 through TASK-041 | TEST-013A, TEST-017, TEST-018 |
| Frictionless complete flows and actionable states | TASK-023, TASK-027 through TASK-040, TASK-055 | TEST-013B, Section 8.5 |
| Separate parallel Help and ten-locale parity | TASK-042 through TASK-047 | TEST-026 through TEST-030 |
| Future Tauri/Leptos typed native boundary, outside demo completion | TASK-048 through TASK-053 | Future TEST-030 through TEST-033 |
| Runtime performance, stale-state, and failure isolation | TASK-010, TASK-016, TASK-021, TASK-026, TASK-054 | TEST-010A, TEST-033A |
| Full accessibility, privacy, visual, and documentation closeout | TASK-054 through TASK-060 | TEST-019 through TEST-033 and Section 8.5 |

## 13. Related Specifications and Evidence

- `crates/z00z_ui_ux/demo/PORTING.md`
- `crates/z00z_ui_ux/demo/README.md`
- `.planning/phases/110-Wallet-UX-UI/UI-UX-SPEC.md`
- `.planning/phases/110-Wallet-UX-UI/UI-UX-REVIEW.md`
- `.planning/phases/071-Request-Bound-Inbox/071-Request-Bound-Inbox-Spec.md`
- `.planning/phases/085-OnionNet/Z00Z-OnionNet-Whitepaper.md`
- `.planning/phases/130-UseCases/Z00Z-UseCases-Whitepaper.md`
- `.planning/phases/130-UseCases/Z00Z-Smart-Cash-Whitepaper.md`
- `crates/z00z_runtime/aggregators/README.md`
- `crates/z00z_runtime/watchers/README.md`
- `.wiki/wiki/references/whitepaper-corpus.md`
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`
- `.github/instructions/rust.instructions.md`
- `crates/z00z_ui_ux/Z00Z-App-TODO/`
- `https://z00z.io/` Corporate theme, visually and token-source checked on
  2026-07-26; provenance only, never a packaged runtime dependency
- `https://v2.tauri.app/learn/mobile-multiwindow/` current Android Activity/iOS
  `UIScene` multi-window guidance; feasibility source only, with real-device
  behavior still gated by TASK-051
