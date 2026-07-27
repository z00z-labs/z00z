# DEMO-PLAN-2 Phase 1 Double-check

<!-- markdownlint-disable MD013 -->

Date: 2026-07-26

## Verdict

**PASS for the canonical registry and pure-state phase.** The new model is
loaded by the existing demo but is not yet used to render the shell. Phase 2 is
therefore still responsible for replacing global top tabs and mobile route
popups with the shared tree.

**NOT PASS for the responsive target.** The unchanged Wallet Assets 1024 px
overflow remains the single strict visual-gate failure carried from Phase 0.

## Task verdicts

| Task | Evidence | Verdict |
| --- | --- | --- |
| TASK-006 | `scripts/port/contracts.js` exposes 62 stable app-route IDs, Help/action IDs, namespace defaults, locale IDs, and independent axis enums. | PASS |
| TASK-007 | `scripts/port/navigation-model.js` is the sole owner of navigation nodes, parents, order, label keys, neutral icons, targets, capabilities, presentation mode, and Help-topic metadata. | PASS |
| TASK-008 | Twelve capability profiles declare independent `Maturity`, `Availability`, `EvidenceSource`, `Freshness`, and `PresentationMode`; the former mixed `capabilityStates` field is absent. | PASS |
| TASK-009 | Pure validation proves unique node and route IDs, one branch parent, depth ≤ 3, no cycles, namespace defaults, neutral/distinguishable icons, two canonical palette IDs, exact locale registry, and one planned Help topic per canonical route. | PASS |
| TASK-009A | The bundled inline sprite gained the missing `message` glyph. The registry/sprite order and 24×24 outline contract are checked by `check-port-readiness.mjs`. | PASS |
| TASK-010 | Presentation state now contains canonical `activeRoute`, `expandedBranchIds`, `drawerOpen`, `activeWalletId`, canonical palette, request generation/cancellation keys, and non-sensitive shell preferences. | PASS |
| TASK-011 | `test-port-contracts.mjs` covers open, close, multi-open, collapsed active ancestor, leaf selection, restore route, wallet switch, lock, logout, and request cancellation; reducers accept no gateway dependency. | PASS |

## Evidence and checks

- `node scripts/test-port-contracts.mjs` — PASS.
- `node scripts/check-port-readiness.mjs` — PASS.
- `run-smoke.sh` with system Chromium — PASS: locale parity, Help coverage,
  production-port checks, and 48/48 Playwright tests. That suite exercises both
  wide desktop behavior and the 390/320 px mobile drawer/Help flows.
- Full visual review at
  `crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-1/registry`
  captured 43 desktop-1280, 43 desktop-1024, 43 tablet-768, 44 mobile-390, and
  54 mobile-320 PNGs. The 180 geometry records contain one and only one failed
  pair: `desktop-1024 / wallet-assets` at 1139 px document width.
- `git diff --check` for the Phase 1 implementation scope — PASS.

## Boundary checks

- The canonical contract has exactly `z00z-default` and `z00z-corporate`.
  The legacy four-card Appearance UI remains intentionally untouched until
  TASK-018A; it is not a second canonical registry.
- Every target route has one planned Help-topic ID in the navigation model.
  Phase 7 creates and localizes the corresponding Help documents; current
  source Help coverage remains the separate 36-state, 38-topic baseline.
- Label keys are stable model metadata. TASK-047 adds their translated copy to
  every locale before the tree is rendered in Phase 2/7 UI flows.
- No reducer invokes a gateway; navigation expansion is presentation-only state.

## Carry-forward requirement

Phase 2 must consume this registry directly, preserve multi-open state on both
desktop and mobile, keep every icon neutral, and remove the 1024 px overflow
without weakening the paired screenshot gate.
