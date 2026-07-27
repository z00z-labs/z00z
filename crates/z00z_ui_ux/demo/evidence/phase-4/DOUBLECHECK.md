# DEMO-PLAN-2 Phase 4 Double-check

<!-- markdownlint-disable MD013 -->

Date: 2026-07-26

## Verdict

**TASK-024 PASS.** Telemetry is the only root accordion for network
observability. Reticulum, OnionNet, and Aggregators are first-level workspace
leaves. Their Overview and deeper routes remain canonical but are rendered only
through a workspace-local desktop context rail or mobile/narrow-tablet top tabs.
No component or deeper Telemetry route is rendered as a nested accordion.

**TASK-025 PASS.** Aggregators Overview, Ingress, Planning, Placement,
Publication, and Recovery are real workspace screens rather than generic route
placeholders. Each screen names the exact Rust runtime boundary and contract
fields, keeps live observations unavailable without a bridge, states
fail-closed guards, and links to its own localized standalone Help topic.

**TASK-026 PASS.** `MockTelemetryGateway` is a deterministic, read-only adapter
with seven exhaustive result states. Request lifecycle is kept independent from
the five capability axes, every response is deeply frozen and route-bound, and
invalid capability/route/scenario combinations fail closed. Fixture provenance
is explicit, `Unavailable` never invents a source, and returned data contains no
wallet-private labels or fields.

**TASK-027 PASS.** Watchers is an always-visible first-level Telemetry workspace
with six local destinations rendered through the shared desktop rail/mobile
tabs. The Roadmap preview completes a deterministic typed-alert → detail →
sanitized evidence → prepared-export workflow, supports source and evidence
filters, and exposes useful loading, success, degraded, unavailable, empty,
malformed, and error/recovery states without granting Watchers mutation or
protocol authority.

**TASK-028 PASS.** Explorer is an always-visible first-level Telemetry workspace
with Overview, Search, Checkpoints, Batches, and Public evidence rendered
through the shared desktop rail/mobile tabs. Its deterministic gateway
allowlists checkpoint, batch, publication, proof, and opaque DA-reference IDs;
private-looking, malformed, unsupported, unknown, and stale input fails closed
without echoing the rejected value. Summary/Technical detail uses explicit
public DTOs and never consumes wallet-local state.

**TASK-029 PASS.** Every typed Watcher alert exposes a separate safe Explorer
action carrying exactly one allowlisted public identifier. A frozen resolver
reduces that ID to `publicId`, `publicKind`, and canonical Explorer `routeId`;
invalid or private-looking input returns no identifier or route. Desktop/mobile
browser flows open the selected public checkpoint, batch, or DA evidence and
Back restores the original selected Watcher alert.

**TASK-030 PASS.** Redaction assertions cover every canonical Telemetry route
across all seven gateway result states and every Watcher export/deep-link DTO.
A browser matrix seeds private receiver/memo drafts, visits all Telemetry routes
at 1280 and 320 px without reloading state, and proves the main workspace never
contains wallet labels/addresses, counterparties, receiver/memo canaries, local
paths, inbox records, or secret field names. Returning to Send proves the
private drafts were isolated rather than discarded.

**Phase 4 PASS.** TASK-024 through TASK-030 are complete.

## Evidence

| Requirement | Workspace evidence | Independent check | Verdict |
| --- | --- | --- | --- |
| Telemetry owns one root accordion. | `navigation-model.js` defines `telemetry` as a root branch; Reticulum, OnionNet, and Aggregators are direct children with `workspace` targets. | Contract tests assert all three parents, target kinds, default routes, child counts, and route-only children. | VERIFIED |
| Components are first-level global leaves. | Global tree rendering emits each workspace itself and does not recursively render workspace-owned routes. | Desktop and mobile smoke assert all three workspace rows are visible and component branches/deep route rows are absent. | VERIFIED |
| Deeper routes remain canonical. | `telemetryWorkspaceRoutes()` derives Overview plus canonical child routes from the shared navigation model. | Browser regression selects Links, Queues, and Publication and verifies route title, local active state, and global workspace active state. | VERIFIED |
| Desktop uses an internal vertical rail. | `telemetryWorkspaceFrame()` places the shared context navigation before the workspace panel. | At 1280 px, all three rails have the expected route count and finish before the content panel begins. | VERIFIED |
| Mobile uses top tabs without reopening the drawer. | The shared `.context-rail` responsive contract becomes a horizontally scrollable sticky row. | At 320 px, all local destinations share one top coordinate, selection works in place, the drawer stays unnecessary, and no viewport overflow occurs. | VERIFIED |
| Old topbar/Network hierarchy is absent. | The shell has no global route tab container or legacy Network navigation element. | Smoke source/DOM checks assert `#wallet-tabs` and `#network-nav` are absent. | VERIFIED |
| Visual behavior is stable across target widths. | Task 024 checkpoint covers 1280, 1024, 768, 390, and 320 px. | 260 responsive audits report 0 issues; Reticulum, OnionNet, and Aggregators desktop/mobile PNGs were manually inspected. | VERIFIED |
| Aggregator screens map to live runtime owners. | Ingress maps to `AggregatorIngress`; Planning to `BatchPlanner`/`PlannerAuthority`; Placement to `ShardPlacementView`; Publication to request/published/record bindings; Recovery to `RecoveryBoundary` and shard tickets. | CodeGraph plus direct Rust type reads verified every displayed type and field against `crates/z00z_runtime/aggregators/src/`. | VERIFIED |
| Concept screens do not fabricate live telemetry. | Observation values remain `Unavailable`; the typed model retains evidence `None`, freshness `Unknown`, and the observed authority while the renderer omits the redundant capability-summary card. | Contract tests assert all six profiles; browser regression asserts four unavailable observation values and absence of the summary card at 1280 and 320 px. | VERIFIED |
| Storage authority remains explicit. | Publication and Recovery screens state that Storage owns checkpoint roots, proofs, lifecycle evidence, and durable recovery truth. | Displayed guards match `PublicationRecord::validate_readiness_bundle` and `RecoveryBoundary::resume` fail-closed checks. | VERIFIED |
| Aggregator Help is route-specific and localized. | Aggregator contextual topics are registered under the canonical localized Telemetry tree. | Help synchronization passes for all current topics/locales; the shared contextual `?` resolves the active Aggregator route without requiring a route-local banner link. | VERIFIED |
| Aggregator visual density remains responsive. | Task 025 checkpoint includes all six screens plus standalone Publication Help at five widths. | 285 responsive audits report 0 issues; desktop/mobile Publication, Recovery, and Help PNGs were manually inspected. | VERIFIED |
| Telemetry result state and capability axes are independent. | `PORT_CONTRACT.telemetryResultStates` exhaustively defines loading, success, degraded, unavailable, empty, malformed, and error while the gateway returns maturity, availability, evidence source, freshness, and presentation mode separately. | Contract tests exercise every Cartesian record shape needed by the seven states and validate all five axes against their canonical enums. | VERIFIED |
| Mock results are deterministic and provenance-safe. | `readObservation()` binds capability, canonical route, request key, and generation; fixture results carry a fixed source ID/timestamp and source-free results carry no invented source ID. | Repeated calls serialize identically; mismatch and unknown-scenario tests throw; Watchers and Explorer preserve their own maturity/presentation profiles. | VERIFIED |
| Telemetry gateway is renderer-safe and offline. | The module has no wallet-state input, mutation method, network primitive, transport fallback, or secret field. | Port-readiness scans the module; redaction assertions reject wallet labels, receiver/counterparty/memo terms, seed, and private-key terms. | VERIFIED |
| Runtime loading and responsive shell remain stable. | `index.html` loads the telemetry adapter after the canonical contracts/navigation model and before the app bootstrap. | Direct smoke passes 18/18; Task 026 visual checkpoint reports 285 audits, 0 issues, and 345 PNGs, with desktop/mobile Aggregators screens manually inspected. | VERIFIED |
| Watchers uses the shared root-only navigation contract. | `telemetry.watchers` is one first-level workspace leaf; Overview, Alerts, Publication checks, DA providers, Censorship signals, and Evidence export are canonical local routes only. | Contract and browser tests assert the leaf/route hierarchy, six internal destinations, desktop vertical rail, mobile horizontal tabs, and absence of deep global-tree rows. | VERIFIED |
| Watcher module mappings are claim-honest. | Overview maps to `WatcherBoundary`/`ObservationSnapshot`; Alerts to `WatcherAlert`; publication to `PublicationWatch`; providers to `ProviderSignal` while disclosing the `ProviderCompare` marker; censorship discloses the `CensorshipWatch` marker; export maps to `EvidenceKey`/`EvidenceRecord`. | CodeGraph plus direct current Rust reads verified the displayed types and current marker-only boundaries under `crates/z00z_runtime/watchers/src/`. | VERIFIED |
| Typed alert inspection is complete and safe. | Each alert exposes kind, severity, typed public subject, fixed observation time, module/fixture provenance, affected public IDs, summary, and safe next action. | Browser flow inspects `MissingBlob` on 1280 and 320 px and carries only its public alert identity into local Evidence export. | VERIFIED |
| Evidence export is sanitized and non-mutating. | The gateway prepares a frozen fixture envelope containing public alert/evidence bindings and explicit redaction classes; it performs no download, filesystem, wallet, validator, storage, or settlement mutation. | Contract and browser tests validate the envelope, fixed provenance, selected batch/checkpoint/DA references, and absence of wallet fixture labels/private values. | VERIFIED |
| Every mandatory Watchers state recovers. | Scenario controls expose loading, success, degraded, unavailable, empty, malformed, and error; source/severity/kind controls are deterministic. | Browser regression exercises all states and retry/clear recovery at 1280 and 320 px; request generations advance without accepting stale hidden state. | VERIFIED |
| Watchers remains responsive and visually consistent. | Neutral grey navigation icons, the Z00Z logo, functional state controls, and existing dark/gold tokens are retained without a repeated capability-summary card. | Task 027 visual matrix reports 325 audits, 0 issues, and 385 PNGs across 1280, 1024, 768, 390, and 320 px; desktop/mobile Alerts, evidence export, and recovery states were manually inspected. | VERIFIED |
| Explorer uses the shared root-only navigation contract. | `telemetry.explorer` is one first-level workspace leaf; Overview, Search, Checkpoints, Batches, and Public evidence are canonical local routes only. | Contract and browser tests assert five internal destinations, desktop vertical rail, mobile horizontal tabs, and no Explorer deep rows in the global tree. | VERIFIED |
| Explorer exposes only intentionally public DTOs. | Checkpoint lifecycle/root/publication evidence, published-batch relationships, public proof envelopes, publication route snapshots, and opaque DA references are modeled as frozen fixture records. | Direct current Rust reads verified the displayed boundaries against `CheckpointLifecycleV1`, `CheckpointPublicationEvidenceV1`, `CheckpointDaReferenceV1`, `CheckpointPublicationV1`, `CheckpointPublicationProofV1`, `PublicationRouteSnapshotV1`, `PublishedBatch`, and `PublicationRecord`. | VERIFIED |
| Public-ID search is strict and fail-closed. | Exact allowlisted shapes cover checkpoint, batch, publication, proof, and opaque DA-reference IDs. Rejected results contain only sanitized error codes/copy and never retain the submitted identifier. | Unit and browser tests exercise five successful families plus private, malformed, unsupported, unknown, stale, degraded, unavailable, loading, empty, malformed-payload, and gateway-error outcomes. | VERIFIED |
| Explorer details remain privacy-restricted. | Summary and Technical modes are generated by explicit record-type DTO projections; related navigation carries public identifiers only. No wallet object or gateway is an input. | Redaction assertions reject wallet labels, receivers, counterparties, memos, route paths, inbox records, and secret material from Explorer fixtures/results; 1280 and 320 px browser flows verify the same rendered boundary. | VERIFIED |
| Explorer remains responsive and visually consistent. | Existing dark/gold tokens, neutral-grey icons, fixed Z00Z topbar logo, functional result state, and shared Telemetry context navigation are retained without a capability-summary card. | Task 028 visual matrix reports 410 audits, 0 issues, and 445 PNGs across 1280, 1024, 768, 390, and 320 px; desktop/mobile Search, checkpoint detail, publication Technical view, private rejection, filters, and recovery states were manually inspected. | VERIFIED |
| Watcher links carry public identity only. | Each alert declares one `explorerAction.publicId`; `resolveExplorerDeepLink()` validates it through the same strict Explorer classifier and returns only `ok`, `publicId`, `publicKind`, and `routeId`. | Contract tests verify checkpoint, batch, and DA-reference links and prove private input returns null identity/route without echoing the rejected value. | VERIFIED |
| Watcher-to-Explorer navigation is reversible. | The safe action selects a canonical Explorer local route, stores only the public ID, and leaves Watcher selection/evidence state intact. | Desktop and 320 px browser flows open `da_ref_72be91`, render its explicit public DTO, avoid route-preview fallback, then Back restores `MissingBlob` as the selected alert. | VERIFIED |
| The linked story remains responsive. | The existing Watcher detail gets one neutral secondary action; Explorer retains the shared desktop rail/mobile tabs and Z00Z topbar logo. | Task 029 visual matrix reports 415 audits, 0 issues, and 450 PNGs; the linked DA-reference detail was manually inspected at 1280 and 320 px. | VERIFIED |
| Gateway redaction covers the full Telemetry state space. | The matrix evaluates every canonical Telemetry route through loading, success, degraded, unavailable, empty, malformed, and error plus all Watcher exports and Explorer deep links. | Contract tests scan 230 frozen payloads against wallet labels/addresses/counterparties, receiver/memo canaries, local paths, inbox records, and renderer-forbidden secret fields. | VERIFIED |
| Renderer redaction preserves private state while isolating it. | The browser test seeds a receiver and memo in Send, transitions through all Telemetry routes without page reload, and scans current `#main-content` text plus markup only. | At 1280 and 320 px no canary appears; returning to Send restores both seeded values, proving route isolation rather than state erasure. | VERIFIED |
| TASK-030 does not alter the visual product. | Only contract/browser assertions changed after the Task 029 application checkpoint. | Full smoke passes 21/21; Task 029 remains the current UI visual matrix at 415 audits, 0 issues, and 450 PNGs. | VERIFIED |

## Commands and results

| Check | Result |
| --- | --- |
| `node scripts/test-port-contracts.mjs` | PASS |
| `node scripts/check-port-readiness.mjs` | PASS |
| `node scripts/check-locales.mjs` | PASS — 10/10 locales, 173 static keys |
| Direct current smoke | PASS — 21/21 |
| Direct current visual review | PASS — 415 audits, 0 issues, 450 PNGs |
| Scoped `git diff --check` | PASS |

Task 025 visual checkpoint:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-4/task-025`.

Task 026 visual checkpoint:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-4/task-026`.

Task 027 visual checkpoint:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-4/task-027`.

Task 028 visual checkpoint:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-4/task-028`.

Task 029 visual checkpoint:
`crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-4/task-029`.

## External workspace note

English Help is being reorganized concurrently outside TASK-026. In the current
workspace `help/en/app/app.md` is absent while new root/topic drafts and metadata
files are untracked. The strict wrapper therefore stops in `check-help.mjs`
before browser launch. Those changes were preserved without modification.
TASK-030 used its independent contract/port gates and direct browser regression
suites against the already compiled local catalogue.
