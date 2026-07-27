# DEMO-PLAN-2 Phase 3 Double-check

<!-- markdownlint-disable MD013 -->

Date: 2026-07-26

## Verdict

**TASK-019 PASS.** Wallet Overview and its Help topic are removed. `wallet.assets`
is the wallet default and is owned by the `Assets & Rights` workspace.
Assets/Vouchers/Permissions and Wallet Settings subroutes remain canonical
routes but render only through the workspace-local desktop rail/mobile top tabs.
The global tree contains only first-level wallet workspaces/actions.

**TASK-020 PASS.** The selected-wallet control remains above the global tree,
has a fixed three-row desktop holder, and drives wallet-owned assets, vouchers,
permissions, activities, preferences, send drafts, and exchange drafts by
wallet ID. Switching from Everyday to Savings while Vouchers is active changes
the eight-row collection to the correct empty Savings collection; switching
back restores the eight Everyday rows.

**TASK-021 PASS.** Canonical routes remain directly deep-linkable, wallet-owned
form drafts and pending/reconciliation presentation survive route history, and
dialogs now own an explicit browser-history overlay entry. Native Back closes
the dialog before changing the route; Forward restores the same dialog state;
Escape and dialog controls consume only that overlay entry.

**TASK-022 PASS.** Swap and Staking remain backed by live-but-noncanonical
compatibility RPC contracts and unavailable native product authority. Exchange
remains a target-only request builder with no provider evidence or authoritative
quote/execution route. All three consume the shared five-axis capability model
without rendering a repeated capability-summary card. Unavailable values stay
inside the functional flow; staking totals render `Unavailable`, never a
fabricated zero.

**TASK-023 PASS.** Send now exposes explicit review, submitting, unknown-outcome,
reconciliation, and pending-confirmation states behind the typed mock gateway.
One stable idempotency key owns one native operation; a timeout after handoff
does not imply failure or authorize a blind retry. The reviewed wallet object
uses a stable snapshot so voucher/permission identity cannot drift after
submission.

**Phase 3 PASS.** All Wallet migration tasks are complete.

## Evidence

| Requirement | Workspace evidence | Independent check | Verdict |
| --- | --- | --- | --- |
| Wallet Overview is absent from the product UI. | `PORT_CONTRACT` has no `home` view or `wallet.overview`; navigation default is `wallet.assets`; the legacy `app-home.md` source is retained for provenance but excluded from `topics.yaml` and the compiled catalogue. | Contract test and smoke assert no Overview node/route. | VERIFIED |
| First-level global Wallet rows stay concise. | `wallet.assets-rights` and `wallet.settings` are `workspace` nodes; their children are local routes. | Desktop/mobile DOM assertions prove no Vouchers, Permissions, Quarantine, or Wallet Settings child row is rendered globally. | VERIFIED |
| Deeper wallet destinations remain canonical. | Workspace buttons call `selectCanonicalRoute()` for Assets/Vouchers/Permissions and Wallet Settings routes. | Smoke changes routes through internal controls and verifies content/active state. | VERIFIED |
| Wallet ownership is keyed by selected wallet. | `activeWallet()` resolves collections from `state.wallets` by `selectedWalletId`; drafts/preferences use wallet-ID maps. | Browser regression switches Everyday → Savings → Everyday on Vouchers and observes 8 → 0 → 8 rows. | VERIFIED |
| Deep links and drafts survive browser history. | Canonical route selection pushes route IDs while `sendDrafts` remains keyed by wallet ID. | At 1280 and 320 px, smoke restores Send through Back after Receive and verifies recipient, amount, and memo values. | VERIFIED |
| Pending/reconciliation presentation survives navigation. | Wallet status uses the selected wallet summary; activity rows render canonical lifecycle status. | At 1280 and 320 px, smoke verifies Pending in `960.00`, Pending out `240.00`, and a `Settling` activity badge after route history. | VERIFIED |
| Dialog/native Back order is explicit. | Dialog opening pushes a serializable `flow-dialog` history entry; Back closes it before route handling and Forward restores it. | At 1280 and 320 px, smoke proves Back → hidden dialog/same Assets route, Forward → restored Asset details, Escape → only overlay closed, next Back → Send. | VERIFIED |
| Compatibility and Target are not product availability. | `wallet.swap`/`wallet.staking` profiles are live + fixture + unavailable; `wallet.exchange` is target + none + unavailable. | Contract tests freeze all five axes; smoke checks the rendered profile attributes and `Compatibility`/`Target` plus `Unavailable` labels at 1280 and 320 px. | VERIFIED |
| Unavailable authority is not fabricated. | Staking replaces unknown staked/reward totals with `Unavailable`; Exchange review leaves rate, output, minimum, fee, ETA, deposit, and status unavailable. | Smoke verifies no staking `0.00 Z00Z` total and seven unavailable Exchange target fields at both widths. | VERIFIED |
| Unknown submission outcomes reconcile before retry. | `submitPayment` journals an operation by idempotency key and returns a typed `timeout_unknown_outcome`; `reconcileOperation` resolves by operation ID. | Contract test repeats the same intent and proves one activity only; smoke completes timeout → reconcile → history at 1280 and 320 px. | VERIFIED |
| Reviewed object identity cannot drift. | Send stores a stable `reviewedItem` snapshot and preserves its original key after the live transferable collection changes. | Static contract and the full Send operation flow complete without choosing a replacement wallet object. | VERIFIED |
| Preview, progress, error, and result states remain actionable. | Send renders distinct submitting, unknown-outcome, reconciling, and pending-confirmation panels with Back/Reconcile/History exits. | Browser regression traverses the full flow at desktop/mobile widths and asserts no dead end or viewport overflow. | VERIFIED |
| Desktop and mobile presentation remains valid. | Task 023 checkpoint includes all reviewed routes and four Send operation states at 1280, 1024, 768, 390, and 320 px. | 260 responsive audits, 0 issues; desktop/mobile submitting, error, reconciling, and result PNGs manually inspected. | VERIFIED |

## Commands and results

| Check | Result |
| --- | --- |
| `node scripts/test-port-contracts.mjs` | PASS |
| `node scripts/check-port-readiness.mjs` | PASS |
| `./run-smoke.sh` | PASS — 16/16 |
| Full visual review | PASS — 260 audits, 0 issues, 315 PNGs |
| `git diff --check -- crates/z00z_ui_ux` | PASS |

Visual checkpoint:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-3/task-023`.
