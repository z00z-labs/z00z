# DEMO-PLAN-2 Phase 0 Double-check

<!-- markdownlint-disable MD013 -->

Date: 2026-07-26

## Verdict

**PASS for Phase 0 baseline and contract freeze.**

**NOT PASS for the target responsive implementation.** The frozen current UI
still overflows on Wallet Assets at 1024 px. No failure is suppressed, and
Phase 2 remains blocked from completion until paired desktop/mobile evidence
proves the defect is removed.

## Claim verification

| Claim | Workspace evidence | Independent check | Rating |
| --- | --- | --- | --- |
| TASK-001 captured all required baseline sizes and both Corporate reference sizes. | `baseline-manifest.json`, `z00z-default-tokens.json`, `z00z-corporate-source.json` | Manifest recount: 43 desktop-1280, 43 desktop-1024, 43 tablet-768, 44 mobile-390, 54 mobile-320, and 2 Corporate PNGs; 229 total. | VERIFIED |
| TASK-002 inventories the current demo contract. | `current-inventory.json` plus source SHA-256 values | Generator rerun succeeded; counts match the live contract, Help, locale, icon, gateway, fixture, and smoke sources. | VERIFIED |
| TASK-003 records protocol and implementation authority without promoting Roadmap UI to protocol truth. | `README.md`, Authority and maturity ledger | Direct source reads confirm native/render ownership, runtime/storage separation, watcher observation-only output, advisory Inbox status, and off-chain short-lived Messenger boundaries. | VERIFIED |
| TASK-004 freezes one coherent target contract. | `DEMO-PLAN-2.md` Sections 1.5, 1.6, 2.1, and 2.3; `README.md`, Approved contract freeze | Tree, set-based accordion state, mandatory Roadmap branches, five axes, neutral icons, and exactly two target palette IDs agree. | VERIFIED |
| TASK-005 removes competing implementation authority. | Dated supersession notices in `UI-UX-SPEC.md`, `UI-UX-REVIEW.md`, and `demo/PORTING.md` | Text search confirms notices for navigation, Help, and Appearance; the old contracts are explicitly historical baseline. | VERIFIED |
| Phase 0 changes preserve current executable behavior. | Only planning, evidence, port documentation, and visual-review harness files changed in this scope. | Final full Playwright run passed 48/48; locale, Help, port, syntax, Markdown, and whitespace gates passed. | VERIFIED |
| The current UI satisfies the new responsive target. | `baseline-manifest.json`, `responsive-layout-audit.json` | 179/180 route-viewport pairs are clean; desktop-1024 Wallet Assets is 1139 px wide and fails two overflow assertions. | CONTRADICTED |

## Desktop and mobile visual evidence

Representative pairs were inspected rather than inferred from file existence:

- current Wallet Assets:
  `desktop-1280-wallet-assets.png` and `mobile-390-wallet-assets.png`;
- known compact-desktop debt:
  `desktop-1024-wallet-assets.png`, cross-checked against its geometry record;
- Corporate provenance:
  `z00z-corporate-desktop.png` and `z00z-corporate-mobile.png`.

The automated matrix covers all current routes at both desktop sizes, tablet,
and both mobile sizes. A future shared-layout task cannot close with only one
side of that evidence.

## Gate results

| Gate | Result |
| --- | --- |
| `capture-phase-0.mjs` regeneration | PASS |
| JavaScript syntax and shell syntax | PASS |
| Locale parity | PASS — 10/10 packs; 179 static UI keys by the locale checker's metric |
| Help synchronization and compile coverage | PASS — 36 routed states; 38 topics × 10 locales |
| Production-port contract and readiness | PASS |
| Markdown lint on all modified Markdown | PASS — 0 issues |
| `git diff --check` on the Phase 0 scope | PASS |
| Isolated repeat of three transient smoke failures | PASS — 3/3 |
| Final full Playwright smoke | PASS — 48/48 |
| Strict responsive visual geometry | FAIL — one unsuppressed desktop-1024 Wallet Assets overflow |

The first full smoke rerun transiently reported three failures. All three passed
unchanged in isolation, and the next complete run passed 48/48. This is retained
as a test-stability observation; it is not represented as a product fix.

## Open items carried forward

1. Phase 2 must remove the 1024 px Wallet Assets overflow and produce a clean
   desktop/tablet/mobile geometry report.
2. Later phases must replace the current four-palette and theme registries; Phase
   0 only freezes and records their current state.
3. Full removal of historical contract prose remains TASK-058. Until then, the
   dated supersession notices are the implementation authority boundary.
