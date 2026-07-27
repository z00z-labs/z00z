# DEMO-PLAN-2 Phase 6 Double-check

<!-- markdownlint-disable MD013 -->

Date: 2026-07-26

## Verdict

**TASK-036–041 PASS.** Messenger now has deeply frozen, deterministic local
fixtures for advisory inbox items, payment requests, voucher proposals, claim
proposals, permission proposals, receiver-card invitations, delivery and
acknowledgement receipts, expiry, abuse, short-lived conversations, every
planned Outbox state, and an unavailable-relay scenario.

Fixture self-validation rejects duplicate IDs, missing request families,
remote URLs, and raw receiver/ACK/route/package/secret fields. The mock gateway
returns immutable advisory DTOs only and explicitly marks Wallet and settlement
mutation as false.

Contacts fixtures cover Known locally, Needs confirmation, Identity changed,
Expired, and Revoked. Every record keeps its contact identity key, Reticulum
destination reference, and Wallet recipient reference in three distinct
domains.

The renderer now provides complete Inbox, Requests, Conversations, Outbox, and
Receipts screens; read-only detail and request review; local acknowledgement,
delete, block, and abuse-report actions; explicit expiry; accepted/rejected
outcomes; and relay-unavailable/recovery states. A payment request reaches Send
only after the Wallet gateway validates the exact typed handoff. The recipient
remains blank.

Contacts now provides local search and status filtering, detail, source and
compatibility metadata, add/import concepts, explicit browser-unavailable
QR/native-share boundaries, edit label, identity-change review, expiry and
revocation handling, typed Pay/Request/Message/Export actions, and local-only
removal.

## Evidence

| Requirement | Workspace evidence | Independent check | Verdict |
| --- | --- | --- | --- |
| Required Messenger fixture families exist. | `scripts/port/messenger-catalog.js` defines the five request types plus advisory, abuse, expiry, conversations, Outbox, receipts, and relay states. | Contract tests assert exact request-type coverage, Inbox/Requests counts, expired acceptance rejection, and unavailable relay state. | VERIFIED |
| Fixtures contain no raw protocol or remote material. | Fixture construction uses safe local labels, summaries, typed scopes, and withheld recipient references only. | Self-validation and contract regexes reject URLs and receiver/ACK/route/package/secret fields; port-readiness passes. | VERIFIED |
| Advisory actions cannot mutate Wallet or settlement. | `MockMessengerGateway.advisoryAction()` returns local-presentation receipts with both mutation flags false. | Contract tests exercise opened, acknowledged, deleted, blocked, and reported. | VERIFIED |
| Payment handoff stays typed and bounded. | Accepted payment request produces an immutable handoff with blank recipient and Wallet revalidation requirement. | Wallet gateway accepts the exact handoff, rejects injected recipient data, and Wallet fixture snapshots remain identical. | VERIFIED |
| Contact identity domains remain separate. | Each fixture has distinct contact, Reticulum, and Wallet reference fields. | Self-validation plus contract tests assert three unique domains for every record. | VERIFIED |
| Contacts remain local. | Contacts gateway list/import/edit/review/remove/action DTOs expose `networkRequest: false` and no implicit trust. | Contract tests cover search, native-boundary rejection, add/edit/identity review/remove, safe Pay handoff, and unchanged Wallet fixtures. | VERIFIED |
| Messenger screens are complete and navigable. | `app.js` renders the three canonical Inbox/Sent/Conversations routes plus message detail, request review, outcome, and relay recovery states without `route-preview`. | Playwright exercises the full flow at 1280 px and 320 px; all 35 smoke tests pass. | VERIFIED |
| Advisory renderer actions preserve financial state. | All five actions pass through `MockMessengerGateway.advisoryAction()` and only update local presentation arrays. | Browser tests compare Asset rows and History counts before and after the advisory flow on desktop and mobile; contract tests compare serialized Wallet fixtures. | VERIFIED |
| Wallet owns accepted payment review. | Messenger creates a blank-recipient typed handoff; `MockWalletGateway.revalidateExternalReviewHandoff()` validates schema, authority, scope, asset, amount, and available balance before Send prefill. | Contract tests accept the exact handoff, reject recipient injection, and browser tests confirm `18.50 Z00Z`, `z00z`, and blank recipient after Wallet validation. | VERIFIED |
| Contact actions preserve domain separation. | Pay uses only the Wallet-recipient domain, Message uses the Reticulum domain, and local export/request use the Contact identity domain. | Wallet accepts the exact Contact Pay handoff and rejects a broadened reference domain; browser tests confirm blank recipient and no Asset/History changes. | VERIFIED |
| Desktop and mobile presentation is geometrically sound. | Phase 6 visual coverage captures Wallet compatibility/target flows, Telemetry, dApps, Messenger, Contacts, and compact Appearance cards at 1280 px and 320 px. | `phase-6-responsive-layout-audit.json` records 38 states and zero issues; representative desktop/mobile images were manually reviewed for absence of capability-summary cards, logo visibility, clipping, hierarchy, and mobile flow. | VERIFIED |

## Commands and results

| Check | Result |
| --- | --- |
| `node scripts/test-port-contracts.mjs` | PASS |
| `node scripts/check-port-readiness.mjs` | PASS |
| `node scripts/check-locales.mjs` | PASS — 10/10 packs; 211 static keys |
| `node scripts/test-palette-contrast.mjs` | PASS |
| `playwright test smoke.spec.js --workers=1` | PASS — 35/35 |
| `playwright test visual-review.spec.js --grep "capture Phase 6"` | PASS — 38 audited states; 0 geometry issues |

Visual evidence:

- `crates/z00z_storage/outputs/checkpoint/phase-110/ui-help-review/`
- `phase-6-responsive-layout-audit.json`

The repository-wide wrapper was not used to regenerate Help during this phase
because the English Help tree is concurrently being reorganized and
`help/en/app/app.md` is absent. Phase 6 validation used the already compiled
Help catalogue and direct contract/browser/visual suites, preserving those
unrelated Help edits.
