# DEMO-PLAN-2 Phase 5 Double-check

<!-- markdownlint-disable MD013 -->

Date: 2026-07-26

## Verdict

**TASK-031 PASS.** The demo loads exactly six deeply frozen local dApp
descriptors: Offline Pay, Private Voucher, External Asset Locker, Scoped
Expenses, Service Credits, and Agent Budget. Every descriptor has a unique
typed intent and neutral semantic icon, explicit object families, offline
behavior, disclosure classes, publisher provenance, maturity, availability,
value/fee path, review boundary, and contextual Help binding.

All six entries remain honest Roadmap-preview fixtures with unavailable
runtime capability. Publisher verification is false, Agent Budget remains
concept-only with value and fee kept separate, and the catalogue contains no
application URL, declared domain, iframe, remote bundle/code descriptor,
generic execution surface, or wallet bridge.

**TASK-032 PASS.** Discover, Installed, Connections, Permissions, and Activity
are real first-level routes inside the single dApps root accordion. Detail,
permission review, accepted/rejected decisions, revoke/expiry, and outcome
screens open inside the main window without adding a nested accordion or a
second permanent dApps navigation rail.

All screens consume deeply frozen local fixtures only. Installed means locally
approved descriptor—not loaded code. Connection decisions and permission
revocation update presentation state only; every outcome explicitly reports
that Wallet state remains unchanged. Canonical route, page title, breadcrumb,
active global leaf, and main content stay synchronized on desktop and mobile.

**TASK-033 PASS.** One reusable `dapp-permission-review-v1` projection now
drives every connection review. It exposes app identity/provenance, typed
intent, action, object family, exact scope, uses, expiry, delegation, value,
fee, disclosures, revoke behavior, and Wallet-owned re-auth behavior through a
deeply frozen DTO.

Acceptance fails closed until the exact scope is confirmed and, for a
value/fee-bearing intent, Wallet re-auth is explicitly acknowledged. The dApps
review never renders or accepts a credential field: actual authentication
remains owned by a later Wallet review. Reject requires no authority and both
decision paths return immutable, deterministic DTOs with `walletMutation:
null`.

**TASK-034 PASS.** An accepted, bounded decision can now produce one immutable
`dapp-wallet-review-handoff-v1` and enter an allowlisted existing Wallet route.
Offline Pay opens `wallet.send` with only the asset family, exact item, and
displayed amount prefilled. Recipient remains blank and the normal Wallet Send
entry/review screens remain the sole path toward submission.

The handoff cannot mutate a Wallet: its constraint record says
`walletMutation: false`, the decision carries no Wallet object, and the
gateway receives no Wallet state. Browser tests compare the complete visible
asset rows and History count before and after the handoff on desktop and
mobile, then prove that the user still has to supply a recipient and enter the
ordinary Wallet review. No Submit action is invoked by dApps.

The workspace-navigation clarification is also enforced globally. A workspace
may exist only as a first-level leaf below a root accordion, and only route
children may live inside it. One immutable local-destination projection feeds
the shared desktop rail/mobile top-tab renderer; Reticulum and every other
Telemetry component expose only their component row in the global tree.

**TASK-035 PASS.** Every Wallet-bound proposal now crosses one explicit
`dapp-intent-proposal-v1` validator paired with a typed
`dapp-held-authority-v1` snapshot. The validator rejects generic signing and
arbitrary payloads, URL/URI-bearing callbacks, remote resources/executable
material, unknown intent types, proposal/descriptor mismatch, any broader
object family/scope/uses/expiry/delegation, hidden or changed value, hidden or
changed fee/path, and malformed held-authority input.

The safe proposal must exactly reproduce the already displayed permission,
value, and fee review before `prepareWalletReview()` can construct a handoff.
The port-readiness gate independently scans every renderer/runtime module and
still finds no `fetch`, XHR, WebSocket, EventSource, beacon, service worker, or
browser persistence primitive.

## Evidence

| Requirement | Workspace evidence | Independent check | Verdict |
| --- | --- | --- | --- |
| The curated catalogue contains the six planned applications. | `scripts/port/dapp-catalog.js` defines one canonical ordered descriptor list and lookup. | Contract tests assert the exact six IDs, order, uniqueness, lookup behavior, and deep freezing. | VERIFIED |
| Capability claims remain honest. | Every descriptor uses `roadmap_preview`, `fixture`, and `unavailable`; Agent Budget alone is `concept`. | Catalogue self-validation checks every capability axis against `PORT_CONTRACT` and fails startup on a mismatch. | VERIFIED |
| Permissions are typed and bounded. | Each entry declares one allowlisted intent, exact requested object families, disclosures, offline behavior, value/fee paths, and a review boundary. | Contract tests assert unique intents, allowlisted object families, and separate Agent Budget value/fee review. | VERIFIED |
| No third-party application runtime is introduced. | Descriptors expose `typed_intent_only` and deny both remote code and wallet-bridge access. | Self-validation and tests reject URL/domain/iframe/bundle/code/executable descriptor material; port-readiness scans the new runtime module for network/storage primitives. | VERIFIED |
| Existing app behavior remains stable. | The catalogue loads after the canonical contracts and before fixtures/presentation state. | Production-port contracts and readiness pass; the full direct browser smoke suite passes 22/22. | VERIFIED |
| TASK-031 does not alter the visual product. | This task adds data and startup validation only; the existing dApps route presentation is unchanged until TASK-032. | The current TASK-029 visual checkpoint remains the last UI-changing baseline. | VERIFIED |
| dApps uses one root accordion and direct first-level routes. | The canonical model retains exactly five direct dApps leaves and the renderer adds no local dApps rail, nested branch, or deep global-tree row. | Desktop/mobile smoke checks the root branch, five direct children, zero nested branch controls, selected leaf, title, and breadcrumb. | VERIFIED |
| Discover and Installed are claim-honest. | Six complete catalogue cards expose maturity, availability, unverified publisher provenance, object families, offline behavior, disclosures, and Help; Installed filters three locally approved descriptors. | Browser tests assert six/three cards and the absence of remote application runtime claims at 1280 and 320 px. | VERIFIED |
| Connection review is complete and deterministic. | Pending, active, and expired fixtures lead to one main-window review with identity, action, object, scope, uses, expiry, delegation, value, fee, disclosure, revoke, and re-auth fields. | Desktop/mobile browser tests inspect all twelve review fields and accept a bounded intent without creating a Wallet operation. | VERIFIED |
| Rejection, revoke, and expiry remain distinct. | Rejected connection and revoked permission use explicit negative outcomes; an expired grant remains unusable and cannot expose a revoke action. | Browser tests exercise accept, revoke, expired, and outcome-return flows; visual captures cover rejected/revoked outcomes and expiry at all target widths. | VERIFIED |
| Activity is sanitized and correctly classified. | Activity exposes only app label, typed event class, fixed time, outcome, and safe summary; revoked, accepted, rejected, and expired are distinct. | Manual desktop/mobile review caught and corrected missing Activity localization and revoke status mapping; locale gate passes 10/10. | VERIFIED |
| Canonical route ownership is stable. | `routeFromCurrentLegacyState()` preserves canonical `dapps.*` ownership while dApp detail/review/outcome remain route-local presentation variants. | Smoke verifies `dApps / Discover`, the `Discover` title, active global leaf, and correct subsequent Activity/Permissions titles. | VERIFIED |
| The implementation remains responsive and visually consistent. | Existing dark/gold and Corporate tokens are reused; dApp icons are neutral grey, the fixed Z00Z topbar logo remains visible, and no Roadmap badges were added. | Full matrix reports 450 responsive audits, 0 issues, and 485 PNGs across 1280, 1024, 768, 390, and 320 px; key desktop/mobile states were manually inspected. | VERIFIED |
| Final outcome corrections are verified. | Reject/revoke now use the semantic danger tone and product copy contains no internal task identifier. | Four post-matrix desktop/mobile outcome captures pass overflow checks and were manually inspected. | VERIFIED |
| Permission review has one canonical projection. | `MockDappGateway.readPermissionReview()` maps every deterministic connection into the same versioned immutable DTO. | Contract tests verify all required fields, exact Offline Pay values, deterministic repeat reads, deep freezing, and deny-by-default boundary flags. | VERIFIED |
| Acceptance is explicit and fail-closed. | `decidePermissionReview()` accepts only `accepted` or `rejected`; acceptance requires exact-scope confirmation and conditional re-auth acknowledgement. | Unit tests exercise missing scope, missing re-auth acknowledgement, accepted, and rejected outcomes; browser tests repeat the sequence at 1280 and 320 px. | VERIFIED |
| Value and fee remain separate. | The DTO exposes separate `value` and `fee` records plus fee path; both feed only the `walletReviewRequired` decision flag. | Contract and browser tests assert 24.00 Z00Z value separately from 0.001 Z00Z fee and its separate Wallet-review path. | VERIFIED |
| Re-auth stays Wallet-owned. | The review shows re-auth requirement/behavior and an acknowledgement, while declaring `wallet_review_only`. | Browser regression proves the dApps review contains no password or secure-entry control; port-readiness scans the gateway for forbidden transports/storage. | VERIFIED |
| Decision DTOs cannot mutate a Wallet. | Decisions contain only deterministic IDs, typed intent reference, decision, time, Wallet-review requirement, and a null mutation marker. | Contract tests assert immutable accepted/rejected results and `walletMutation: null`; full smoke remains 22/22. | VERIFIED |
| The extended review remains responsive. | Shared checkbox/error components use existing semantic tokens and preserve the single main-window flow. | TASK-033 matrix reports 450 audits, 0 issues, and 485 PNGs; desktop/mobile review plus two error-state captures were manually inspected. | VERIFIED |
| Accepted intent handoff is allowlisted and immutable. | `prepareWalletReview()` validates the exact accepted decision and maps only known intent types into fixed Wallet targets. | Contract tests reject rejected/corrupt decisions and assert a deeply frozen exact Offline Pay handoff. | VERIFIED |
| dApps cannot bypass Wallet review. | The handoff prefills no recipient, credential, fee authorization, idempotency key, operation ID, or submission state. | Desktop/mobile smoke proves recipient is blank, amount/item are bounded, and the existing Send screen must advance separately to `Review send`. | VERIFIED |
| Handoff does not mutate visible Wallet state. | Gateway code has no Wallet input or mutation command; presentation state holds the handoff separately from wallet fixtures. | Contract tests snapshot immutable wallet fixtures; browser tests compare all asset rows and the History count before/after handoff at 1280 and 320 px. | VERIFIED |
| Every deeper route stays inside its workspace. | `workspaceLocalDestinations()` derives the default plus ordered children; model validation forbids nested workspaces and non-route workspace children. | Contract adversarial cases fail as required; 22/22 smoke covers all seven current workspaces on desktop/mobile and verifies deep routes are absent from the global tree. | VERIFIED |
| Final responsive behavior remains clean. | Shared semantic styles render workspace rails vertically on desktop and as sticky horizontal tabs on compact widths; dApp handoff reuses the Wallet card language. | TASK-034 matrix reports 455 audits, 0 issues, and 490 PNGs across five widths; Reticulum, mobile drawer, and Wallet handoff pairs were manually inspected. | VERIFIED |
| Generic signing and arbitrary payloads fail closed. | The proposal boundary requires both flags to be exactly false and rejects common raw signing/payload fields before schema acceptance. | Adversarial contract cases assert the dedicated `generic_signing_forbidden` rejection. | VERIFIED |
| URLs and remote resources fail closed. | Recursive validation rejects URL/URI/href keys, URL-like schemes, remote resource flags, iframe/bundle/executable/source-code material. | Separate arbitrary-URL and remote-resource cases assert dedicated rejection codes; port-readiness independently forbids network/browser loading APIs. | VERIFIED |
| Unknown intents cannot reach Wallet. | Proposal intent must be in the six-entry allowlist and match the bound descriptor plus reviewed action. | A mutated `sign_anything` intent is rejected with `unknown_intent_type`. | VERIFIED |
| Permissions cannot become broader than held authority. | Proposal permission must exactly match the typed held-authority fixture across object family, scope, uses, expiry, and delegation. | Adversarial scope, unlimited uses, far-future expiry, and unrestricted delegation cases all return `permission_exceeds_held_authority`; tampered held authority is rejected separately. | VERIFIED |
| Value and fee cannot be hidden or rewritten. | Presence/display are required for value; presence/display/path are required for fee and must match the reviewed DTO exactly. | Independent hidden-value and hidden-fee mutations return their dedicated failure codes before handoff. | VERIFIED |

## Commands and results

| Check | Result |
| --- | --- |
| `node scripts/test-port-contracts.mjs` | PASS |
| `node scripts/check-port-readiness.mjs` | PASS |
| `node scripts/check-locales.mjs` | PASS — 10/10 locales, 173 static keys |
| Direct current smoke | PASS — 22/22 |
| Targeted TASK-035 dApps smoke | PASS — desktop 1280 and mobile 320 |
| Direct current visual review | PASS — 455 audits, 0 issues, 490 PNGs |
| Scoped `git diff --check` | PASS |

Task 032 visual checkpoint:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-5/task-032`.

Task 033 visual checkpoint:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-5/task-033`.

Task 034 visual/smoke checkpoint:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-5/task-034`.

## External workspace note

English Help is being reorganized concurrently outside TASK-031. In the current
workspace `help/en/app/app.md` is absent while new root/topic drafts and metadata
files are untracked. Those changes were preserved without modification; the
strict Help wrapper was not used for this task.
