# DEMO-PLAN-2 Phase 2 Double-check

<!-- markdownlint-disable MD013 -->

Date: 2026-07-26
Interaction addendum: 2026-07-27

## Verdict

**PHASE 2 PASS.** TASK-012 through TASK-018A now satisfy the branded responsive
shell gate. The global hierarchy is root accordion → workspace/action leaf.
Every deeper destination is rendered inside its workspace as a desktop vertical
rail and mobile/narrow-tablet top tabs. The standalone Help application uses the
same global/local split.

**TASK-018A PASS.** Appearance now exposes only `z00z-default` and
`z00z-corporate`; a palette owns the colour scheme, legacy inputs migrate
deterministically, selecting a card applies it immediately, and the same
canonical ID reaches the separate Help application. Exactly one selected card
shows `ACTIVE`; no Apply, Cancel, Reset, or duplicate applied-status controls
remain.

**TASK-013 PASS.** The application has no global route-tab, Network-nav, or
former mobile route-popup implementation. The Playwright suite now asserts the
canonical desktop tree and mobile drawer instead of recreating compatibility
controls. This has been checked against a fresh 240-PNG desktop/mobile capture
set.

TASK-012 and TASK-014 through TASK-018 are independently covered below.

## Phase 2 task verdicts

| Task | Workspace evidence | Independent check | Verdict |
| --- | --- | --- | --- |
| TASK-012 | One full-width topbar owns the persistent Z00Z logo/wordmark; desktop sidebar and main content begin below it. | Five-viewport screenshots show the logo on app, lock, dialog, drawer, Default/Corporate, and Help surfaces. | VERIFIED |
| TASK-014 | `navigation-model.js` defines only root `branch` toggles and first-level `workspace`/route leaves; deeper routes are workspace children. | Contract validation rejects nested branches and duplicate routes; desktop smoke proves independent Wallet/Telemetry expansion and one active workspace leaf. | VERIFIED |
| TASK-015 | The mobile drawer reuses the global registry; `.context-rail` becomes a sticky horizontal tab row below the mobile topbar. | Mobile smoke covers Assets, Send, Reticulum, and Help; 320/390 screenshots show workspace tabs without drawer reopening. | VERIFIED |
| TASK-016 | Drawer open/close owns inert background, focus containment/restoration, Escape, backdrop, and reduced-motion paths. | Playwright keyboard/focus scenario passes on the compact shell. | VERIFIED |
| TASK-017 | Breadcrumbs include root/workspace/current destination; title, selected wallet, privacy, attention, status, and lock utilities remain singular. | Smoke verifies breadcrumb, context, status, privacy, attention, and lock/unlock behavior. | VERIFIED |
| TASK-018 | Assets & Rights and Wallet Settings subroutes no longer render in the global tree or obsolete mobile route popups. | DOM assertions prove only their workspace rows are global; internal route buttons change canonical route/history. | VERIFIED |

## Workspace-local navigation contract addendum

The later user clarification is now a model-level invariant rather than a
Telemetry-specific renderer. `workspaceLocalDestinations()` derives one
ordered immutable list for every current workspace. Model validation rejects a
workspace below anything except a root branch and rejects any non-route child
inside a workspace.

The current 22-test browser suite checks all seven workspaces at 1280 and 320
px. It proves that the global tree contains only the workspace leaf, deeper
routes are absent there, desktop uses a vertical local rail, and mobile uses
one horizontal top-tab row. The latest five-width visual checkpoint reports
455 geometry audits with zero issues and includes manually inspected
Reticulum desktop/mobile and lower mobile-drawer captures.

Latest checkpoint:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-5/task-034`.

## TASK-013 verdict

| Requirement | Workspace evidence | Independent check | Verdict |
| --- | --- | --- | --- |
| No global route tabs or former Network sidebar are rendered. | `index.html`, `app.js`, and `styles/components.css` remove `#wallet-tabs`, `#network-nav`, and their hierarchy rules. | `smoke.spec.js` reads source and DOM; the active-source search finds those strings only in negative assertions. | VERIFIED |
| The desktop application hierarchy is the canonical tree. | `navigationNodeMarkup()` and `renderNavigationTree()` render the registry into `#app-navigation-tree`. | Full smoke suite: desktop tree, independent accordion, selection, wallet route, palette, and standalone Help scenarios pass. | VERIFIED |
| The mobile application hierarchy is the same canonical tree. | `mobileNavigationDrawerMarkup()` uses the same renderer and no old popup groups. | Full smoke suite verifies the drawer, independent accordion, wallet switch, Help, focus restoration, and 320/390 geometry. | VERIFIED |
| No visual regression or overflow was introduced by the cleanup. | Fresh checkpoint `.../demo-plan-2-phase-2/workspace-first/`. | Visual review: 240 audits, 0 issues, 295 PNG captures; desktop and mobile workspace pairs manually inspected. | VERIFIED |

## Claim verification

| Claim | Workspace evidence | Independent check | Verdict |
| --- | --- | --- | --- |
| Exactly two application palettes are rendered. | `scripts/port/presentation-state.js`, `app.js`, `styles/colors.css` | `test-port-contracts.mjs` and targeted browser test assert the IDs and cards. | VERIFIED |
| No independent application theme selector or `data-theme` mapping remains. | `index.html`, `help.html`, `app.js`, `styles/colors.css`, Help scripts | `check-port-readiness.mjs` rejects `data-theme`; browser test asserts it is absent in app and Help. | VERIFIED |
| Legacy palette migration obeys the plan precedence. | `resolvePalettePreference()` and `resolveInitialPalettePreference()` | Reducer contract tests and browser cases cover explicit Corporate, legacy light, legacy dark, and invalid values. | VERIFIED |
| Palette cards switch immediately and expose one `ACTIVE` marker. | `app.js` canonical palette state and direct card action | Targeted Playwright checks desktop and mobile, both switch directions, the unique active marker, and absence of the retired controls/status. | VERIFIED |
| Corporate provenance and accessible semantic mappings remain local and traceable. | `styles/colors.css` and `scripts/test-palette-contrast.mjs` | Source snapshot values plus Default/Corporate text, controls, focus, and status contrast pairs pass. | VERIFIED |
| Desktop and mobile remain visually valid. | `responsive-layout-audit.json` and PNGs under the checkpoint root | 240 route/viewport audits have zero issues; 295 PNGs cover desktop/tablet/mobile, Watchers/Explorer, Corporate app/Help, drawer, dialog, and lock states. | VERIFIED |
| All Phase 2 smoke tests have migrated from global tabs. | `smoke.spec.js` contains root-accordion, workspace navigation, mobile drawer, and standalone Help scenarios. | `./run-smoke.sh` passes 13/13 and active-source search finds no obsolete hierarchy implementation. | VERIFIED |

## Commands and results

| Check | Result |
| --- | --- |
| `node scripts/test-port-contracts.mjs` | PASS |
| `node scripts/check-port-readiness.mjs` | PASS |
| `node scripts/test-palette-contrast.mjs` | PASS |
| `node scripts/check-locales.mjs` | PASS — 10/10 packs, 175 static keys |
| Targeted Playwright palette interaction | PASS — immediate selection on desktop/mobile |
| `./run-smoke.sh` | PASS — 13/13 root-accordion/workspace desktop/mobile scenarios |
| Full visual review | PASS — 240 geometry audits, 0 issues, 295 screenshots |
| `git diff --check -- crates/z00z_ui_ux/demo` | PASS |

## Visual pairs inspected

- Desktop shell and responsive asset table:
  `desktop-1024-wallet-assets.png`.
- Mobile independent accordion state:
  `mobile-320-wallet-telemetry-multi-open.png` and
  `mobile-320-wallet-telemetry-multi-open-lower-tree.png`.
- Corporate Appearance on both form factors:
  `desktop-1280-settings-appearance-corporate.png` and
  `mobile-390-settings-appearance-corporate.png`.
- Independent Help surface with the applied Corporate palette:
  `desktop-1280-corporate-global-help.png` and
  `mobile-390-corporate-global-help.png`.

Checkpoint root:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-2/shell`.

TASK-013 checkpoint root:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-2/canonical-tree`.

Final Phase 2 checkpoint root:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-2/workspace-first`.

Fresh visual pairs manually inspected:

- Desktop Reticulum workspace with only `Reticulum` in the global tree and
  Overview/Node/etc. in the internal rail:
  `desktop-1280-telemetry-reticulum-node.png`.
- Mobile Reticulum and Watchers internal top tabs:
  `mobile-320-telemetry-reticulum-node.png` and
  `mobile-320-telemetry-watchers-alerts.png`.
- Mobile lower drawer with only Telemetry workspace leaves:
  `mobile-320-wallet-telemetry-multi-open-lower-tree.png`.
- Desktop/mobile Help global/local split:
  `desktop-1280-context-help.png` and `mobile-320-context-help.png`.
