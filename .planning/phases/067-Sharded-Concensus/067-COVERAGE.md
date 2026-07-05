# Phase 067 Coverage Appendix

**Source:** `.planning/phases/067-Sharded-Concensus/067-TODO.md` plus
`.planning/phases/067-Sharded-Concensus/067-verdict.md`
**Generated:** 2026-07-05
**Status:** Expanded planning coverage ledger

## Coverage Audit

- Unique `TASK-NNN` identifiers in `067-TODO.md` plus `067-verdict.md`: `0`
- Base required GSD Plan Groups from `067-TODO.md`: `9`
- Required verdict Local-Conformance-Simulation groups: `10`
- Total Required GSD Plan Groups: `19`
- Base group source: `067-TODO.md` `14.1` through `14.9`
- Verdict group source: `067-verdict.md` Strong Acceptance Gate, Concrete Add
  List, and MUST-solve list
- Coverage rule: each required implementation group maps to exactly one
  `067-NN-PLAN.md`
- Exact traceability source:
  `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md` locks all `19`
  H2 sections, all `55` H3 sections, `447` dash-list bullets, and `80`
  numbered-list items from `067-TODO.md`.
- Duplicate task status: none
- Missing task status: none
- Planning fail condition: if `TASK-NNN` rows appear later, or if any required
  group is renamed, split, merged, or dropped, this appendix and all plan
  mappings must be regenerated before execution.

## Plan ID Lock

Plan ids are file numbers. Wave numbers are execution order only and do not
replace plan ids. The required verdict expansion files are:
`067-10-PLAN.md`, `067-11-PLAN.md`, `067-12-PLAN.md`, `067-13-PLAN.md`,
`067-14-PLAN.md`, `067-15-PLAN.md`, `067-16-PLAN.md`, `067-17-PLAN.md`,
`067-18-PLAN.md`, and `067-19-PLAN.md`.

`067-12-PLAN.md` through `067-15-PLAN.md` are mandatory standalone plans:
planner authority, multi-process devnet, network fault matrix, and
HotStuff-like local backend contract.

## Task-To-Plan Coverage Table

| Task row | PLAN id | Source refs | Inputs | Artifacts | Tests | Expected results | Simulation requirement | Anti-placeholder proof | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `PHASE-0` Terminology and boundary cleanup | `067-01` | `067-TODO.md` `3.6`, `3.7`, `14.1`, `16`, `17`, `19`; `067-verdict.md` CFT/BFT vocabulary hardening; `090-New-Scenarios` `15.3` | live `standby` names, CFT/BFT drift, `sim_5a7s` config and tests | renamed runtime/config/docs/tests, no-alias guard, updated topology fixtures | targeted aggregator and rollup topology tests, active-code grep audit | one live protocol term: `secondary aggregator`; local seam described as CFT until stronger proof exists | local config/home loading and quorum tests only; no external network | active code, config, and tests change together; no alias or doc-only rename | complete |
| `PHASE-1` Commit subject and certificate types | `067-02` | `067-TODO.md` `14.2`, `9.4`, `9.5`, `9.6`, `19`; `067-verdict.md` artifact-binding hardening; `090-New-Scenarios` `15.5`, `15.9` | route digest, placement digest, plan digest, state roots, lineage, proof metadata | `commit_subject.rs`, `shard_vote.rs`, `shard_quorum_certificate.rs`, root exports, digest tests | new unit tests for canonical encode, drift sensitivity, vote and certificate rejection | quorum artifacts become first-class and deterministic | repeat-encode and mutation fixtures with real route/placement/recovery inputs | real binary encoding, domain bytes, and negative tests; no constant digest | complete |
| `PHASE-2` Secondary replay verifier | `067-03` | `067-TODO.md` `14.3`, `9.3`, `13.1`, `19`; `067-verdict.md` replay-origin hardening; `090-New-Scenarios` `15.5`, `15.9`, `15.10` | ingress-normalized package, route lookup, placement view, recovery record, publication binding, theorem digest | `secondary_replay.rs`, verifier result types, replay tests, fixture helpers | new unit/integration tests for exact replay acceptance and drift rejection | secondary votes mean independent deterministic replay | local replay uses real planner, recovery, DA, and theorem inputs; no copied primary bytes | vote creation must depend on recomputed subject, not fixture constants | complete |
| `PHASE-3` Local quorum certificate integration | `067-04` | `067-TODO.md` `14.4`, `4.3`, `10.3`, `19`; `067-verdict.md` local-QC artifact hardening; `090-New-Scenarios` `15.10` | current `ConsensusAdapter`, live membership rules, new vote/certificate artifacts | updated `consensus_adapter.rs`, extended commit path, consensus tests, publication handoff checks | targeted consensus tests for honest parity, split-brain freeze, removed/unready voter rejection | local majority path and certificate path agree on honest inputs | same-term and mixed-membership local conflicts simulated with real records | certificate path must drive real commit decisions; no shadow DTO path | complete |
| `PHASE-4` End-to-end `sim_5a7s` harness | `067-05` | `067-TODO.md` `14.5`, `13`, `19`; `067-verdict.md` Gate 2 and `SIM-5A7S` default local-conformance hardening; `090-New-Scenarios` `15.1`-`15.15` | wallet-style package fixture, route table, placement table, recovery, local DA, validator | independent `scenario_11` home, route-bound publication evidence, quorum JSON artifacts, package-to-validator harness | unit, integration, and E2E `scenario_11` tests including happy path, dual-primary path, all-shard sweep, offline-owner defer, and crash resume | one local package-to-validator flow binds one subject digest through replay, certificate, DA, and validator while offline-owner paths defer instead of rerouting | real `sim_5a7s` routing, planning, publication, and validator boundaries with only external transport faked | evidence files must be produced from live runs; no scenario_1 piggyback or report-only closure | complete |
| `PHASE-5` Join, removal, and rotation simulation | `067-06` | `067-TODO.md` `14.6`, `10.4`-`10.7`, `19`; `067-verdict.md` RAID-like redistribution and Gate 13 hardening; `090-New-Scenarios` `15.11`, `15.12` | ready-state transitions, recovery lineage, route generation, takeover rules | extended join/failover/route-rollout tests, scenario fault matrix, transition evidence | targeted runtime tests plus scenario_11 crash, stale, and mixed-generation cases | topology changes become certificate-safe and fail closed when state drifts | simulated join, removal, planned rotation, emergency takeover, partition/heal, restart | no transition may succeed through docs or config only; tests must observe real vote eligibility changes | complete |
| `PHASE-6` Validator and theorem binding | `067-07` | `067-TODO.md` `14.7`, `4.5`, `11`, `12`, `19`; `067-verdict.md` Gate 3 and concrete add list for `067-07`; `090-New-Scenarios` `15.5`, `15.10`, `15.14`, `15.15` | local DA publication, theorem bundle, checkpoint flow, resolved batch, certificate digest | DA binding fields, validator gate, theorem/link checks, new rollup and validator tests | validator rejects missing, detached, stale, or mismatched certificate binding | local DA resolve and validator acceptance use real publication/theorem/certificate state | resolved batches and theorem bundles must be validated against live certificate digests | no constant certificate digest or ignored gate path may survive tests | complete |
| `PHASE-7` Network and signature adapter | `067-08` | `067-TODO.md` `14.8`, `7.1`, `8.6`, `12`, `19`; `067-verdict.md` Gates 6, 9, and 14 plus concrete add list for `067-08`; `090-New-Scenarios` `15.7`, `15.13`, `15.14` | current vote path, local replay verifier, `z00z_crypto` domain and signature primitives | signature trait, deterministic local signer seam, in-memory vote transport, replay-verified vote service, equivocation and payload-withholding evidence, transport tests | local tests for real signatures, transport mediation, and equivocation evidence | external transport is faked locally, but replay, signatures, and evidence are real | in-memory or loopback transport only; no live libp2p required | transport cannot bypass replay verifier; equivocation evidence must be emitted from live conflicting votes | complete |
| `PHASE-8` BFT and Celestia local backend | `067-09` | `067-TODO.md` `14.9`, `8.1`-`8.12`, `11.2`, `12`; `067-verdict.md` Gates 4, 5, 7, and 8 plus concrete add list for `067-09`; `090-New-Scenarios` `15.7`, `15.13`, `15.15` | proven local subject interface, simulated larger committees, local external-DA adapter, validator gate | local BFT backend adapter, local Celestia-style blob adapter, 3f+1 committee fixtures, scenario or rollup tests | simulated BFT quorum tests, local Celestia resolution tests, validator independence tests | all external network and DA behavior stays local and deterministic while using real subject, vote, publication, and validator logic | simulated 7/10/13-node committees and local blob retrieval only | no future-only backend claims; tests must prove 3f+1 or 2f+1 semantics and artifact equality | complete |

## Verdict Expansion Coverage Table

| Task row | PLAN id | Source refs | Inputs | Artifacts | Tests | Expected results | Simulation requirement | Anti-placeholder proof | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `VERDICT-LCS-01` Dependency and runnable aggregator readiness | `067-10` | `067-verdict.md` dependency section, Gate 10, Concrete Add List; `067-TODO.md` `13`, `14.8`, `14.9`; `090-New-Scenarios` `15` | workspace Cargo graph, `sim_5a7s` process commands, rollup-node config | Cargo ownership changes, runnable rollup-node binary, process command tests | cargo metadata, CLI help, rollup-node process/topology tests | manifest commands are executable and dependency claims are honest | external backend crates remain non-claims until exercised | no crate-only or string-only process proof | complete |
| `VERDICT-LCS-02` Durable consensus evidence store | `067-11` | `067-verdict.md` Gates 12 and Hard Blockers; `067-TODO.md` restart/resume matrix and Section `13`; `090-New-Scenarios` `15` | QC, votes, DA binding, recovery, storage primitives | consensus store API, restart recovery tests, scenario evidence | store, recovery, failover, scenario tests | votes/QCs/anchors survive restart or fail closed | storage commit/recovery real; process and wall-clock may be simulated | no in-memory-only or log-only persistence | complete |
| `VERDICT-LCS-03` Planner authority and failover claim boundary | `067-12` | `067-verdict.md` Gate 11 and planner answer; `067-TODO.md` replay/planner requirements; `090-New-Scenarios` `15` | planner config, route digest, placement generation | planner authority checks, drift tests, claim registry row | planner, dispatch, scenario tests | planner truth is deterministic replicated config or real HA is tested | every aggregator recomputes canonical plan locally | no docs-only HA removal or planner stub | complete |
| `VERDICT-LCS-04` Multi-process devnet harness | `067-13` | `067-verdict.md` Gate 10 and process model answer; `067-TODO.md` Sections `13`, `14.8`; `090-New-Scenarios` `15` | runnable binary, durable store, planner authority, manifest | devnet script, optional Compose, process tests, per-process evidence | process-devnet tests, process tests, smoke script, scenario tests | process/devnet behavior is local simulated-full | local OS/Docker process boundary; consensus primitives real | no manifest-only or Docker-file-only proof | complete |
| `VERDICT-LCS-05` Network fault matrix and transport conformance | `067-14` | `067-verdict.md` Gate 6 and hard blockers; `067-TODO.md` partition/heal and failure telemetry requirements | signature, transport, durable evidence, process harness | fault scheduler, transport/evidence tests, fault evidence JSON | transport fault, adapter, equivocation, scenario tests | network simulation cannot inject consensus truth | delivery timing/faults simulated; replay/signature real | no label-only transport faults | planned |
| `VERDICT-LCS-06` HotStuff-like local backend contract | `067-15` | `067-verdict.md` Gates 7 and 8; `067-TODO.md` `14.9`; `090-New-Scenarios` `15` | BFT committee, subject/replay, transport, durable store | HotStuff-local backend, view/change tests, evidence | HotStuff-local, BFT rules, transport, scenario tests | HotStuff-like claim is executable locally | backend local; commit subject/replay/validator real | no name-only HotStuff or CFT-as-BFT | planned |
| `VERDICT-LCS-07` Celestia-local artifact conformance | `067-16` | `067-verdict.md` Gates 4 and 5; Concrete Add List; `067-TODO.md` DA/Celestia notes | QC binding, local DA, validator, BFT local artifacts | Celestia-local adapter, artifact schema, binding tests | Celestia binding, DA, validator, scenario tests | Celestia term is local simulated-full | fake external provider only; local artifact contract real | no provider-name-only or constant blob adapter | planned |
| `VERDICT-LCS-08` Structured evidence registry | `067-17` | `067-verdict.md` Gate 14; `067-TODO.md` evidence/report requirements; `090-New-Scenarios` `15` | transport, durable store, Celestia-local, report writer | evidence registry, evidence tests, scenario report JSON | evidence registry, equivocation, Celestia, scenario tests | safety evidence is machine-auditable | local faults simulated; artifact refs real | no string-only or un-emitted evidence | planned |
| `VERDICT-LCS-09` Glossary claim registry and report honesty | `067-18` | `067-verdict.md` Gates 1 and 15; all plan files; `067-TEST-SPEC.md`; `067-TESTS-TASKS.md` | plan corpus, evidence artifacts, glossary terms | claim registry, claim audit script, report tests | claim audit and scenario report tests | every term has enforced claim state | simulated-full must cite executable local evidence | no prose-only glossary or unqualified overclaim | planned |
| `VERDICT-LCS-10` Final local conformance simulation gate | `067-19` | `067-verdict.md` Gate 2 and hard blockers; `067-TODO.md` Section `13`; all plan files | all prior plans and evidence | final conformance doc, final evidence bundle, claim registry | scenario_11, claim audit, devnet smoke, release tests | integrated local conformance proof closes all local blockers | only allowed external boundaries faked; local primitives real | no compile-only or disconnected-unit closure | planned |

## Exact Mapping Assertion

The two coverage tables above are authoritative. They assign all `19` required
groups to exactly one `067-NN-PLAN.md` each, with no missing or duplicate group
assignments. Summary tables in `067-verdict.md` are sequencing/file inventory
only and do not create additional mappings.
