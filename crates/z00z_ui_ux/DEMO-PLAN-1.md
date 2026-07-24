---
goal: Modular multilingual contextual Help and compact app-first mobile UI
version: 1.0
date_created: 2026-07-23
last_updated: 2026-07-23
owner: z00z UI/UX
status: 'Completed'
tags: [design, architecture, accessibility, i18n, mobile, offline, help]
---

# Introduction

![Status: Completed](https://img.shields.io/badge/status-Completed-brightgreen)

This plan converts the wallet demo from a permanently explanatory website-style
surface into a compact application UI. Explanatory copy moves into local,
multilingual Markdown Help topics. Operational values, validation, warnings,
errors, and irreversible-action disclosures remain visible at the point of use.
The implementation remains static, offline-capable, and structured for a direct
port to Leptos CSR inside Tauri 2.

## 1. Requirements & Constraints

- **REQ-001**: Store Help source documents under
  `crates/z00z_ui_ux/demo/help/<locale>/<topic>.md`.
- **REQ-002**: Support all application locales: `en`, `ru`, `fr`, `de`, `es`,
  `pt`, `ko`, `tr`, `ja`, and `zh-Hans`.
- **REQ-003**: Give every routed view or routed subsection exactly one stable
  Help topic ID.
- **REQ-004**: Replace the current account/person topbar action with a global
  Help action.
- **REQ-004A**: Expose the same global Help action from the mobile application
  menu because desktop topbar actions are intentionally hidden below 768 px.
- **REQ-005**: Show one `akar-icons:question` contextual Help action in every
  rendered view.
- **REQ-006**: Open contextual Help without navigating away from the active
  wallet, settings section, telemetry section, filters, or draft input.
- **REQ-007**: Highlight the active Help target when a topic section declares a
  target ID.
- **REQ-008**: Render contextual Help as a compact anchored panel on desktop and
  a bottom sheet on mobile.
- **REQ-009**: Keep the global Help entry focused on application-wide concepts;
  keep contextual Help focused on the current view.
- **REQ-010**: Remove permanently visible explanatory prose when it does not
  affect immediate action or safety.
- **REQ-011**: Keep errors, validation, destructive-action confirmation,
  security warnings, recovery warnings, unavailable/read-only state, and
  authoritative capability boundaries visible.
- **REQ-012**: Keep logically related labels, values, statuses, and compact
  actions on one mobile row whenever they fit at a 320 px viewport without
  reducing the standard font scale.
- **REQ-013**: Truncate long user or protocol identifiers with ellipsis; expose
  the complete value through an accessible title, copy action, or detail view.
- **REQ-014**: Keep the mobile topbar on one line and scroll the selected tab
  fully into view.
- **REQ-015**: Preserve the current visual structure, colour LUT, typography,
  selected states, hover states, and wallet/network information architecture.
- **REQ-016**: Update each Help locale and generated catalogue atomically when a
  topic is added or removed.
- **REQ-017**: Keep Help content available without network access.
- **REQ-018**: Add or remove a Help topic without editing unrelated view
  renderer functions.
- **REQ-019**: Keep Help strings separate from user-authored wallet names,
  addresses, asset names, identifiers, and other non-translatable data.
- **REQ-020**: Verify desktop and mobile visual output after each UI phase.
- **REQ-021**: Give content-bearing modal flows, including Asset details,
  their own Help topic when their concepts are not fully covered by the parent
  view.
- **A11Y-001**: Use buttons with explicit accessible names for global and
  contextual Help actions.
- **A11Y-002**: Move focus into Help when opened and restore it to the invoking
  action when closed.
- **A11Y-003**: Support Escape, backdrop close, keyboard section navigation,
  and visible focus.
- **A11Y-004**: Expose Help as a labelled dialog and announce the current
  section without relying on colour or highlight alone.
- **A11Y-005**: Keep touch targets at least 44 by 44 CSS pixels on mobile.
- **SEC-001**: Do not allow Markdown HTML, script URLs, inline event handlers,
  or arbitrary selectors into the generated Help catalogue.
- **SEC-002**: Compile Markdown into a constrained data model containing plain
  text headings, paragraphs, lists, and declared target IDs.
- **CON-001**: Do not add runtime CDN, web-font, icon-font, translation,
  Markdown-parser, or network dependencies.
- **CON-002**: Use local inline SVG for the question and Help icons.
- **CON-003**: Do not change unrelated dirty worktree files.
- **CON-004**: Keep the demo usable from GitHub Pages and the future Tauri
  packaged asset directory.
- **CON-005**: Do not hide warnings merely to reduce vertical space.
- **PAT-001**: Use one canonical Help topic LUT to map presentation state to a
  topic ID.
- **PAT-002**: Use YAML front matter in Markdown for stable topic metadata.
- **PAT-003**: Generate and commit a browser-loadable Help catalogue so runtime
  rendering never fetches Markdown.
- **PAT-004**: Use `data-help-anchor` IDs for optional visual highlighting.
- **PAT-005**: Use shared compact row and key/value primitives instead of
  per-view mobile overrides.
- **PAT-006**: Use repository-local Playwright smoke tests and screenshots for
  responsive validation.

## 2. Implementation Steps

### Implementation Phase 1

- GOAL-001: Capture the current information architecture and define the Help
  and compact-layout contracts before changing runtime UI.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-001 | Record the route and subsection matrix from the canonical `PORT_CONTRACT`: Home, wallet assets/vouchers/permissions, send, receive, history, swap, exchange, staking, backup, wallet settings general/security/backup/policies/advanced, application settings general/appearance/Reticulum/OnionNet, every Reticulum/OnionNet/Aggregators telemetry tab, and content-bearing modal flows such as Asset details. | ✅ | 2026-07-23 |
| TASK-002 | Capture 1280×800, 390×844, and 320×800 baselines for wallet settings General/Security/Backup/Policies, application General/Appearance, Assets, and asset details. | ✅ | 2026-07-23 |
| TASK-003 | Classify current helper copy as `operational`, `safety`, `validation`, `status`, or `explanation`; only `explanation` moves exclusively into Help. | ✅ | 2026-07-23 |
| TASK-004 | Define a shared compact-row contract for label, value, status, control, and action slots with truncation behavior. | ✅ | 2026-07-23 |

### Implementation Phase 2

- GOAL-002: Create an offline, multilingual Markdown Help source and catalogue
  pipeline with deterministic coverage checks.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-005 | Add `demo/help/topics.yaml` as the canonical topic LUT containing topic ID, Markdown basename, global/context scope, and presentation-state match fields. | ✅ | 2026-07-23 |
| TASK-006 | Add one Markdown source file per LUT topic under each locale directory. Use YAML front matter fields `id`, `title`, `summary`, and `scope`; use `## Heading {#target-id}` for optional highlight targets. | ✅ | 2026-07-23 |
| TASK-007 | Add `demo/scripts/compile-help.mjs` with a dependency-free parser for the constrained YAML front matter and Markdown subset. Reject duplicate IDs, unknown metadata, raw HTML, unsupported headings, unsafe links, undeclared target syntax, and malformed lists. | ✅ | 2026-07-23 |
| TASK-008 | Generate `demo/scripts/generated/help-catalog.js` containing immutable locale catalogues and the route LUT. The generated file must not contain executable content from Markdown. | ✅ | 2026-07-23 |
| TASK-009 | Add `demo/scripts/check-help.mjs` to derive every routed state from `PORT_CONTRACT` and prove a one-to-one context-topic match; also verify the one global topic, canonical locale directories, exact topic parity, matching front matter IDs, non-empty titles/summaries/sections, required/forbidden target IDs, view-specific guidance rather than generic boilerplate, valid syntax, and generated-catalogue freshness. | ✅ | 2026-07-23 |
| TASK-010 | Run Help compilation and coverage checks from `demo/run-smoke.sh`, `demo/scripts/build-pages-release.mjs`, and the production-port readiness checks. | ✅ | 2026-07-23 |
| TASK-010A | Add `demo/scripts/scaffold-help.mjs` so one command creates a new topic file in all locale directories without overwriting existing translations; generated placeholders must fail the completeness check until translated. | ✅ | 2026-07-23 |

### Implementation Phase 3

- GOAL-003: Implement reusable global and contextual Help components without
  coupling Help content to individual render functions.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-011 | Add a local inline SVG symbol derived from `akar-icons:question` and register it in the icon LUT. | ✅ | 2026-07-23 |
| TASK-012 | Add `demo/scripts/port/help-registry.js` with pure functions to resolve a topic from presentation state, resolve locale fallback, and expose immutable topic data. | ✅ | 2026-07-23 |
| TASK-013 | Add `demo/scripts/help-controller.js` to own open/close state, focus restoration, focus containment, Escape/backdrop behavior, section navigation, target highlighting, and desktop/mobile presentation. When invoked from a native dialog, mount Help in the same top layer and inert only the underlying dialog content. | ✅ | 2026-07-23 |
| TASK-014 | Add one Help host element to `index.html`; load the generated catalogue, registry, and controller before `app.js`. | ✅ | 2026-07-23 |
| TASK-015 | Replace the topbar account/person button with the global Help button while preserving the current action sizing and visual treatment. | ✅ | 2026-07-23 |
| TASK-015A | Add the global Help action to the mobile application drawer with the same topic and translated accessible name. | ✅ | 2026-07-23 |
| TASK-016 | Inject one contextual question button through the shared `render()` pipeline. Resolve its topic from state rather than adding bespoke buttons to every renderer. | ✅ | 2026-07-23 |
| TASK-016A | Let the shared dialog frame accept an optional resolved Help topic; use it for Asset details and other content-bearing modal views without duplicating Help controller logic. | ✅ | 2026-07-23 |
| TASK-017 | Add translated UI chrome strings for Help title, close, section navigation/count, and unavailable fallback to all locale catalogues. | ✅ | 2026-07-23 |
| TASK-018 | Add shared Help component styles using existing colour, border, radius, spacing, shadow, typography, and focus tokens. | ✅ | 2026-07-23 |

### Implementation Phase 4

- GOAL-004: Replace website-style explanatory layouts with compact,
  application-style information rows.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-019 | Refactor wallet General to compact rows: `Wallet name | value | Rename`, `Wallet ID | value | Read-only`, and `Chain | badge`. Move the runtime-boundary explanation into Help while keeping any authoritative unavailable status visible. | ✅ | 2026-07-23 |
| TASK-020 | Refactor wallet Security to compact rows for lock interval, Lock now, password, recovery phrase, public keys, and master-key rotation. Keep password/recovery warnings in sensitive dialogs. | ✅ | 2026-07-23 |
| TASK-021 | Refactor wallet Backup to compact key/value status rows plus automatic backup, interval, create, and restore action rows. Keep integrity and destructive restore warnings visible. | ✅ | 2026-07-23 |
| TASK-022 | Refactor wallet Policies to compact profile/rule rows. Move conceptual descriptions into Help; keep Locked/Local/Target status and any fail-closed warning visible. | ✅ | 2026-07-23 |
| TASK-023 | Refactor wallet Advanced toolbars and local capability messaging so repeated explanatory text lives in Help while validation results remain visible. | ✅ | 2026-07-23 |
| TASK-024 | Refactor application General to compact Language, Regional format, Time zone, and Notifications rows. | ✅ | 2026-07-23 |
| TASK-025 | Refactor Appearance to compact Theme and palette cards. Remove palette prose from the persistent card; keep palette name and swatches. Keep protected semantic-colour behavior in Help. | ✅ | 2026-07-23 |
| TASK-026 | Refactor asset details to one key/value row per property at mobile widths. Truncate long owner/asset IDs and preserve full accessible values. | ✅ | 2026-07-23 |
| TASK-027 | Remove or shorten non-operational second-line copy in Assets, Vouchers, Permissions, Send, Swap, Staking, Backup, History, and telemetry only where the Help topic now contains the explanation. | ✅ | 2026-07-23 |

### Implementation Phase 5

- GOAL-005: Standardize responsive behavior and eliminate accidental mobile
  wrapping without shrinking typography.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-028 | Replace the mobile single-column `.setting-line` override with the shared compact-row grid. Define controlled fallback behavior for complex palette/YAML rows only. | ✅ | 2026-07-23 |
| TASK-029 | Add reusable `.compact-row`, `.compact-row-label`, `.compact-value`, and `.compact-action` styles with `min-width: 0`, ellipsis, and stable control sizing. | ✅ | 2026-07-23 |
| TASK-030 | Keep current topbar tabs on one line and make the active tab fully visible after render, font load, language change, viewport resize, and Help close. | ✅ | 2026-07-23 |
| TASK-031 | Verify that 320 px layouts have no document-level horizontal overflow and no clipped active tab label. | ✅ | 2026-07-23 |
| TASK-032 | Verify that long German, Russian, Japanese, Korean, and Chinese Help/UI labels do not overlap values or actions. | ✅ | 2026-07-23 |

### Implementation Phase 6

- GOAL-006: Verify behavior, accessibility, offline operation, and production
  portability; then synchronize specifications and documentation.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-033 | Add Playwright tests for global Help, contextual Help topic resolution, section highlighting, focus restoration, Escape, locale switching, and fallback. | ✅ | 2026-07-23 |
| TASK-034 | Add tests proving all 36 `PORT_CONTRACT` routed states resolve exactly one context topic, every context topic maps to a routed state, one global topic exists, all 38 topics have all ten Markdown locale files, and the generated catalogue is current. | ✅ | 2026-07-23 |
| TASK-035 | Add mobile geometry tests for compact rows, asset-detail rows, active-tab visibility, Help bottom sheet, 44 px touch targets, and zero horizontal document overflow at 390 px and 320 px. | ✅ | 2026-07-23 |
| TASK-036 | Run the locale audit, Help audit, production-port contract tests, production-port readiness, and Playwright smoke suite. | ✅ | 2026-07-23 |
| TASK-037 | Capture desktop baselines plus every one of the 36 routed states at both 390 px and 320 px, including long-locale, global Help, contextual Help, and dialog Help overlays; visually inspect every screenshot for wrapping, overlap, clipping, spacing, colour, borders, focus, and scroll behavior. | ✅ | 2026-07-23 |
| TASK-038 | Update `demo/README.md`, `demo/PORTING.md`, `UI-UX-REVIEW.md`, and `UI-UX-SPEC.md` with the Help source contract, offline guarantee, topic-extension procedure, compact-row contract, and mobile verification matrix where those files exist. | ✅ | 2026-07-23 |
| TASK-039 | Run `git diff --check`, inspect the complete scoped diff, confirm unrelated dirty files are untouched, and update this plan to `Completed` only after every task is verified. | ✅ | 2026-07-23 |

## 3. Alternatives

- **ALT-001**: Fetch Markdown at runtime. Rejected because `file://`, packaged
  Tauri assets, cache state, and offline radio use would make runtime behavior
  environment-dependent.
- **ALT-002**: Store Help only in locale JavaScript objects. Rejected because
  Help must remain independently editable Markdown documentation.
- **ALT-003**: Add an external Markdown parser. Rejected because it creates a
  runtime/build dependency for a deliberately constrained content format.
- **ALT-004**: Add a separate contextual button manually to every render
  function. Rejected because topic addition/removal would couple unrelated
  renderers.
- **ALT-005**: Hide all helper and warning copy. Rejected because validation,
  destructive actions, recovery, security, and unavailable state require
  point-of-action disclosure.
- **ALT-006**: Reduce font size until mobile rows fit. Rejected because it
  damages readability and contradicts the established typography contract.
- **ALT-007**: Keep the current single-column mobile settings layout. Rejected
  because it separates one logical control into multiple visual rows and
  produces excessive vertical scrolling.

## 4. Dependencies

- **DEP-001**: Existing `window.Z00ZI18n` locale registry and ten language
  catalogues.
- **DEP-002**: Existing presentation state in
  `demo/scripts/port/presentation-state.js`.
- **DEP-003**: Existing local SVG sprite and icon registry.
- **DEP-004**: Existing CSS design tokens in `styles/foundation.css` and
  `styles/colors.css`.
- **DEP-005**: Existing Playwright smoke harness in `demo/run-smoke.sh`.
- **DEP-006**: Existing Pages release builder must copy the `help/` source and
  generated Help catalogue for inspectability, although runtime uses only the
  generated catalogue.

## 5. Files

- **FILE-001**: `crates/z00z_ui_ux/DEMO-PLAN-1.md` — executable plan and
  completion record.
- **FILE-002**: `crates/z00z_ui_ux/demo/help/topics.yaml` — canonical Help LUT.
- **FILE-003**: `crates/z00z_ui_ux/demo/help/<locale>/*.md` — multilingual Help
  source.
- **FILE-004**: `crates/z00z_ui_ux/demo/scripts/compile-help.mjs` — compiler.
- **FILE-005**: `crates/z00z_ui_ux/demo/scripts/check-help.mjs` — coverage and
  freshness gate.
- **FILE-006**: `crates/z00z_ui_ux/demo/scripts/generated/help-catalog.js` —
  committed browser catalogue.
- **FILE-007**: `crates/z00z_ui_ux/demo/scripts/port/help-registry.js` — pure
  topic resolution.
- **FILE-008**: `crates/z00z_ui_ux/demo/scripts/help-controller.js` — reusable
  Help interaction component.
- **FILE-009**: `crates/z00z_ui_ux/demo/index.html` — icon, Help host, and script
  loading.
- **FILE-010**: `crates/z00z_ui_ux/demo/app.js` — state-to-topic context,
  compact markup, and global/contextual Help wiring.
- **FILE-011**: `crates/z00z_ui_ux/demo/styles/components.css` — compact-row and
  Help component styles.
- **FILE-012**: `crates/z00z_ui_ux/demo/locales/*.js` — translated Help chrome.
- **FILE-013**: `crates/z00z_ui_ux/demo/smoke.spec.js` — behavior and responsive
  regression tests.
- **FILE-014**: `crates/z00z_ui_ux/demo/run-smoke.sh` — Help gates.
- **FILE-015**: `crates/z00z_ui_ux/demo/scripts/build-pages-release.mjs` —
  deterministic Pages output.
- **FILE-016**: `crates/z00z_ui_ux/demo/README.md` and `PORTING.md` —
  maintenance and porting contracts.
- **FILE-017**: `crates/z00z_ui_ux/demo/visual-review.spec.js` — reproducible
  desktop/mobile/long-locale screenshot matrix.
- **FILE-018**: `crates/z00z_ui_ux/demo/run-visual-review.sh` — isolated local
  visual-review runner and screenshot output setup.

## 6. Testing

- **TEST-001**: Every topic in `topics.yaml` exists once in every locale.
- **TEST-002**: Every locale has exactly the English topic set.
- **TEST-003**: Every Markdown file compiles without raw HTML or unsupported
  syntax.
- **TEST-004**: Generated Help catalogue matches source content.
- **TEST-005**: Global Help replaces the account button and opens application
  Help.
- **TEST-005A**: Global Help is reachable from the mobile menu while desktop
  topbar actions are hidden.
- **TEST-006**: Contextual Help resolves the exact active section.
- **TEST-006A**: Contextual Help resolves the exact active telemetry tab and
  content-bearing dialog topic rather than only the parent route.
- **TEST-007**: Contextual section navigation highlights only declared anchors.
- **TEST-008**: Help closes with Escape/backdrop and restores focus.
- **TEST-009**: Language changes update Help without changing user-authored
  wallet data.
- **TEST-010**: Help works with network access disabled.
- **TEST-011**: Wallet/application setting rows remain one logical row at
  390 px and use declared controlled fallbacks at 320 px.
- **TEST-012**: Asset-detail label and value remain in one row on mobile.
- **TEST-013**: Active topbar tabs are fully visible and never wrap.
- **TEST-014**: No document-level horizontal overflow at 1280, 1024, 390, or
  320 px.
- **TEST-015**: Visible validation, warnings, errors, status, and capability
  boundaries remain after explanatory-copy migration.
- **TEST-016**: Desktop and mobile screenshot comparison shows no overlap,
  clipping, inconsistent borders, accidental colour changes, or undersized
  touch targets.

## 7. Risks & Assumptions

- **RISK-001**: Moving copy into Help could hide safety-critical information.
  Mitigation: classify copy before removal and test retained warning classes.
- **RISK-002**: Ten-locale topic drift could create silent English fallback.
  Mitigation: exact locale parity is a failing build/smoke gate.
- **RISK-003**: A loose Markdown renderer could create injection risk.
  Mitigation: compile a constrained plain-text AST and reject HTML/unsafe links.
- **RISK-004**: One-row mobile layouts can overflow with translated labels.
  Mitigation: fixed action sizing, `min-width: 0`, ellipsis, title/copy access,
  language matrix screenshots, and controlled complex-row fallback.
- **RISK-005**: Highlight overlays can obscure controls or trap pointer input.
  Mitigation: use a non-intercepting outline layer and clear it on section
  change/close/render.
- **RISK-006**: Help state could become stale after a route change.
  Mitigation: resolve from current presentation state on every render.
- **RISK-007**: Generated content could be forgotten in commits.
  Mitigation: freshness check in smoke and Pages build.
- **ASSUMPTION-001**: English remains the explicit fallback locale.
- **ASSUMPTION-002**: The existing ten-locale registry is the canonical
  language set for Help.
- **ASSUMPTION-003**: Help content describes the demo behavior and does not make
  unsupported runtime or protocol claims.
- **ASSUMPTION-004**: User-entered labels and protocol identifiers are never
  translated.

## 8. Related Specifications / Further Reading

- `crates/z00z_ui_ux/demo/PORTING.md`
- `crates/z00z_ui_ux/demo/README.md`
- `.planning/phases/110-Wallet-UX-UI/UI-UX-REVIEW.md`
- `.planning/phases/110-Wallet-UX-UI/UI-UX-SPEC.md`
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`

## 9. Plan Review

The plan must pass this review before Phase 2 implementation starts:

- **REVIEW-001**: Every user requirement maps to at least one REQ and one TASK.
- **REVIEW-002**: Topic routing is data-driven and independent from renderer
  branching.
- **REVIEW-003**: Markdown remains the editable source while runtime remains
  fetch-free and offline.
- **REVIEW-004**: Locale completeness fails closed rather than silently
  producing partial Help.
- **REVIEW-005**: Safety and capability disclosures are explicitly excluded
  from blanket copy removal.
- **REVIEW-006**: Mobile compactness does not depend on smaller typography.
- **REVIEW-007**: Accessibility behavior is testable and includes focus
  restoration and keyboard closure.
- **REVIEW-008**: Pages and future Tauri packaging consume the same generated
  catalogue.
- **REVIEW-009**: All new modules have a single responsibility and explicit
  dependencies.
- **REVIEW-010**: Completion requires both automated checks and human visual
  inspection of desktop and mobile screenshots.

### Review Result

- **REVIEW-RESULT-001**: Corrected mobile reachability by adding the global Help
  action to the application drawer; replacing the hidden desktop account button
  alone was insufficient.
- **REVIEW-RESULT-002**: Added Exchange to the route matrix even though the
  current tab is disabled, so enabling it later does not require a Help
  architecture change.
- **REVIEW-RESULT-003**: Expanded telemetry mapping from one topic per source to
  one topic per active telemetry tab.
- **REVIEW-RESULT-004**: Added Help contexts for content-bearing modal flows,
  beginning with Asset details.
- **REVIEW-RESULT-005**: Added a locale-safe scaffolding command; adding or
  removing a topic now has an explicit all-locale workflow.
- **REVIEW-RESULT-006**: The corrected plan satisfies REVIEW-001 through
  REVIEW-010 and is approved for implementation.
- **REVIEW-RESULT-007**: Replaced redundant previous/next Help controls with one
  keyboard-operable section navigator. It exposes the same ordered content on
  desktop and mobile with fewer controls and no duplicated navigation state.
- **REVIEW-RESULT-008**: The completion audit found that the `Home` route was
  absent from the initial LUT. Added `app.home` and changed the Help gate to
  derive all 36 routed states directly from `PORT_CONTRACT`, preventing future
  route/topic drift.
- **REVIEW-RESULT-009**: Native `<dialog>` top-layer behavior could otherwise
  cover contextual Help. The controller now mounts Help in the invoking dialog,
  contains keyboard focus, inerts only underlying content, restores focus, and
  is verified on desktop, 390 px, and 320 px.
- **REVIEW-RESULT-010**: Removed duplicated locale arrays from Help tooling.
  Compile, check, and scaffold commands now read the canonical locale registry.
- **REVIEW-RESULT-011**: Replaced residual generic “inspect or change” Help
  copy in every locale with view-specific actions and added a fail-closed
  content check to prevent the placeholder language from returning.
- **REVIEW-RESULT-012**: Expanded the visual matrix to capture global,
  contextual, and native-dialog Help independently at every viewport.
- **REVIEW-RESULT-013**: Long-locale Help screenshots exposed a transient toast
  above the mobile bottom sheet. The shared overlay contract now suppresses
  toast presentation while Help is open, with a regression assertion.

## 10. Completion Evidence

- Locale catalogue: 10/10 catalogues, 105 static UI keys.
- Help catalogue: 38 topics × 10 locale folders = 380 Markdown sources.
- Route proof: all 36 `PORT_CONTRACT` routed states resolve exactly one
  contextual topic; every contextual topic resolves back to a routed state.
- Runtime: bundled generated catalogue; offline opening verified after the
  application is loaded.
- Automated validation: JavaScript syntax, locale audit, Help coverage and
  freshness, port contracts, port readiness, Pages release cache behavior, and
  36 Playwright smoke tests all pass.
- Visual validation: 101 fresh screenshots inspected at 1280×800, 390×844, and
  320×800. Both mobile widths cover all 36 routed states; the matrix also
  includes RU/DE/JA/KO/ZH long-label and localized Help cases, Asset details,
  and open desktop/mobile global, contextual, and native Asset details dialog
  Help.
- Repository hygiene: scoped diff reviewed, `git diff --check` passes, and
  unrelated dirty files remain untouched.

## 11. Help source-hash synchronization addendum — 2026-07-24

- [x] Make English the canonical Help source and record SHA-256 topic revisions
  per locale in `demo/help/source-state.json`.
- [x] Translate only changed English topics through the existing local
  `Z00Z_TRANSLATE_COMMAND` boundary; never expose Markdown syntax or runtime
  wallet data to the translator.
- [x] Fail compilation, smoke, and Pages publication when a locale is on an
  older English hash or its section/block structure differs.
- [x] Add a local development server that watches English Help, rebuilds the
  bundled catalogue, and reloads the page after successful synchronization.
- [x] Move the contextual question action to one fixed shell host aligned below
  the topbar Help action; clear the sticky status bar and mobile safe areas.
- [x] Verify hash-change rejection, automatic translated reconstruction,
  scroll-stable geometry, focus restoration, offline runtime behavior, and
  desktop/mobile screenshots.
