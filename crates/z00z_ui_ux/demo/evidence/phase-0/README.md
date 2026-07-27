# DEMO-PLAN-2 Phase 0 Evidence

<!-- markdownlint-disable MD013 -->

Captured on 2026-07-26. This directory freezes the executable demo baseline
before navigation, Help, or Appearance migration.

The independent claim-to-evidence verdict is in `DOUBLECHECK.md`.

## Task verdicts

| Task | Verdict | Evidence |
| --- | --- | --- |
| TASK-001 | Complete | `baseline-manifest.json`, `z00z-default-tokens.json`, and `z00z-corporate-source.json`; hashed screenshots are stored under the manifest's `root`. |
| TASK-002 | Complete | `current-inventory.json` records the current routes, Help trees, locales, icons, appearance IDs, badges, gateway surface, fixtures, and smoke assertions. |
| TASK-003 | Complete | The authority and maturity ledger below records the reviewed live, compatibility, target, and concept boundaries. |
| TASK-004 | Complete | The contract-freeze record below approves the Section 2.1 tree, Roadmap behavior, five capability axes, visual/icon baseline, and two-palette target. |
| TASK-005 | Complete | Dated supersession notices are present in `UI-UX-SPEC.md`, `UI-UX-REVIEW.md`, and `demo/PORTING.md`. |

Completion here means the Phase 0 evidence was captured and the target contract
was frozen. It does not mean the pre-existing UI passes the new target's strict
responsive gate.

## Baseline result

- Five viewports were captured: 1280×800, 1024×768, 768×1024, 390×844, and
  320×800.
- All 36 current route states were audited at every viewport: 180
  route-viewport geometry records.
- The manifest contains 229 desktop, tablet, mobile, Help, drawer, popup,
  dialog, and Corporate-reference PNG files with SHA-256 hashes.
- The strict geometry gate remains failed with zero suppressed failures:
  `wallet-assets` is 1139 px wide in the 1024 px viewport and reports both
  `viewport-overflow` and `element-outside-viewport`.
- That failure is frozen baseline debt. Phase 2 cannot close until a new paired
  desktop/mobile run removes it without weakening the gate.

## Current inventory summary

| Surface | Frozen count or IDs |
| --- | --- |
| Resolved route states | 36 |
| Help topics | 38 |
| Locale packs | 10 |
| Recursive English catalogue keys | 257 |
| Navigation/content icon IDs | 50 |
| Application palettes | `z00z-default`, `black-gold-elegance`, `moonlit-stroll`, `walking-at-night` |
| Theme modes | `dark`, `light` |
| Gateway queries / commands / errors | 10 / 16 / 8 |
| Fixture IDs | 29 |
| Playwright smoke assertions | 48 |

The recursive English-key count is an inventory metric from
`capture-phase-0.mjs`; it is not interchangeable with the static-key metric
reported by `check-locales.mjs`.

## Authority and maturity ledger

| Evidence | Workspace source | Frozen decision |
| --- | --- | --- |
| Renderer/native boundary | `demo/PORTING.md`, Runtime decision | Leptos owns declarative views and ephemeral presentation state; native Rust owns secrets, signing, configuration mutation, journals, and settlement. |
| Browser contract surface | `demo/scripts/port/contracts.js:7-155` | Version 1.2.0 is the compatibility baseline. Its route and gateway identifiers are inventory, not proof that every listed command is live production authority. |
| Wallet capability review | `UI-UX-REVIEW.md`, Capability map from current code | Registered routes are separated into live, compatibility, and target lanes. Exchange and UI configuration mutation remain target-only; Swap and Staking remain compatibility/noncanonical. |
| Aggregator authority | `crates/z00z_runtime/aggregators/README.md:3-6,24-35` | Runtime owns planning, placement, and publication binding; storage owns settlement roots, proof, and recovery truth. Operational notes remain advisory. |
| Watcher authority | `crates/z00z_runtime/watchers/README.md:3-16,33-34` | Watchers expose observations, alerts, and evidence only; they do not own planner, validator, storage, or settlement truth. |
| Inbox maturity | `.planning/phases/071-Request-Bound-Inbox/071-Request-Bound-Inbox-Spec.md:137-156` | Only the wallet-local in-memory advisory inbox is current. Sanitized durable records and the encrypted mailbox are targets; OnionNet transport is future and transport-only. |
| Current inbox implementation | `crates/z00z_wallets/src/receiver/request_inbox.rs:27-38` | The current recipient binding is explicitly wallet-local and advisory; it is not a Messenger or settlement authority. |
| Messenger privacy boundary | `.planning/phases/085-OnionNet/Z00Z-OnionNet-Whitepaper.md:450-456` | Message contents stay off-chain and short-lived; public settlement receives at most a minimal relevant receipt; permanent public inbox identities are rejected. |

## Approved contract freeze

- The canonical navigation tree is exactly Section 2.1 of
  `DEMO-PLAN-2.md`; desktop renders it in the left sidebar and mobile renders
  the same model in a left drawer.
- Accordion expansion is a set. Opening or closing one branch preserves every
  other branch, and mobile waits for leaf selection before closing the drawer.
- Watchers, Explorer, dApps, and Messenger are selectable, deterministic
  `Roadmap preview` flows, not shipped-protocol claims and not dead
  `Coming soon` placeholders.
- Capability-bearing screens keep five independent fields: `Maturity`,
  `Availability`, `EvidenceSource`, `Freshness`, and `PresentationMode`.
- The current Z00Z visual language and dark token values remain the baseline.
  Menu icons remain distinct, semantic, bundled, and neutral grey.
- The target application registry contains exactly `z00z-default` (dark) and
  `z00z-corporate` (light). There is no independent application theme mode.
  Code-syntax themes remain independent and code-only.
- Every shared visual gate requires both desktop/tablet and mobile screenshot
  evidence. One-sided screenshot evidence cannot close an implementation task.

## Reproduction

```bash
Z00Z_PLAYWRIGHT_EXECUTABLE_PATH=/usr/bin/chromium \
Z00Z_VISUAL_REVIEW_DIR="$PWD/crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-0/current" \
crates/z00z_ui_ux/demo/run-visual-review.sh
```

The command is expected to remain non-zero for this frozen baseline because the
1024 px Wallet Assets overflow is intentionally not suppressed.

```bash
node crates/z00z_ui_ux/demo/scripts/capture-phase-0.mjs \
  "$PWD/crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-0/current"
```

## Verification result

- locale, Help synchronization/compile, production-port, JavaScript/shell
  syntax, Markdown, and whitespace gates passed;
- three transient smoke failures passed unchanged in isolation (3/3), followed
  by a clean full run (48/48);
- the responsive visual gate remains failed only for the explicitly recorded
  desktop-1024 Wallet Assets overflow.
