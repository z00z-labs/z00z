# Phase 7 doublecheck evidence

Date: 2026-07-26

## Verdict

PASS. Standalone Help and the application now share one generated route/topic
contract. All 71 global, route, detail, and review topics exist in every one of
the ten supported locales, use canonical topic directories, and carry verified
English-source plus native-review hashes.

## Phase 7 task verdicts

| Task | Result | Evidence |
| --- | --- | --- |
| TASK-042 | PASS | Every locale has canonical `app`, `wallets`, `telemetry`, `dapps`, `messenger`, `contacts`, and `settings` topics. The runtime catalogue uses Telemetry while all 19 original tracked paths per locale are required and additional English originals are protected when present by `preserved-sources.json`. |
| TASK-043 | PASS | `generate-help-topics.mjs` derives one context topic per 61 canonical routes plus explicit detail/review topics. `check-help.mjs` proves forward and reverse coverage. |
| TASK-044 | PASS | English is canonical; the local build-time translator synchronized 9 locales. `source-state.json` contains source and native-review SHA-256 hashes for 71 × 10 documents. |
| TASK-045 | PASS | Standalone Help exposes seven root-only, independently open accordions and first-level workspaces. Local topics render as a desktop rail and mobile/narrow-tablet top tabs. |
| TASK-046 | PASS | Smoke coverage proves global and contextual launchers reuse the named `z00z-help` surface, change its topic without opening a second page, and preserve app route/form state when either surface closes. |
| TASK-047 | PASS | `demo-plan-2.js` extends all ten catalogues with exact-parity action, state, palette, permission-field, and accessible-name keys; canonical navigation labels are reused by Help and the app. |

## Layer 1 — implementation inspection

- Topic definitions are generated from `PORT_CONTRACT.routes`; detail/review
  topics are a closed explicit list.
- Runtime Markdown comparison ignores topical `_drafts` and only the 19 exact
  legacy paths in `preserved-sources.json`; it requires every retained original
  and the exact canonical registered files everywhere else.
- Dialog/detail topics are not exposed as global tree rows and resolve only via
  contextual launch state.
- The Help tree contains no nested accordion. Multiple root groups use an
  independent `Set`, matching the application reducer behavior.
- The Help surface has its own always-visible branded topbar and inherits only
  `z00z-default` or `z00z-corporate`.
- The translation bridge runs locally at build time; runtime code contains no
  translation network path.

## Layer 2 — deterministic checks

```text
node scripts/check-locales.mjs
  PASS — 10/10 catalogues, 212 statically referenced UI keys

node scripts/test-help-sync.mjs
  PASS — stale English, missing translator, translation, structure, and review-hash behavior

node scripts/compile-help.mjs
  PASS — generated packaged Help catalogue

node scripts/check-help.mjs
  PASS — 61 routed states, 71 topics × 10 locales

node scripts/test-port-contracts.mjs
  PASS

node scripts/check-port-readiness.mjs
  PASS

node scripts/test-palette-contrast.mjs
  PASS

./run-smoke.sh
  PASS — 27/27 tests

git diff --check -- crates/z00z_ui_ux/demo
  PASS
```

Additional filesystem/hash assertions passed:

- no loose canonical locale-root Markdown;
- all 190 original tracked Markdown files remain present, including the legacy
  `network` sources; additional English originals are excluded from runtime
  matching without being deleted;
- all seven canonical topic directories exist in all ten locales;
- 710 locale-topic records contain both SHA-256 source and review hashes.

## Layer 3 — desktop/mobile visual review

Targeted Phase 7 capture:

`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-7/task-047/`

- 4 desktop screenshots at 1280×800;
- 14 mobile screenshots at 320×800, including all nine non-English locales;
- desktop context rail and multi-open tree;
- mobile top tabs and “close one, keep another open” drawer state;
- Watcher alert and Contact identity contextual topics;
- persistent Z00Z Help topbar;
- `phase-7-help-responsive-audit.json`: 0 issues.

The desktop and mobile contact sheets were manually inspected after waiting for
drawer transitions. No hidden logo, nested accordion, viewport overflow,
overlap, or stale half-open drawer was accepted.
