---
name: attack-surfaces-create
description: Auto-invoked when the user wants to find attack surfaces, map a Rust project, audit every component for possible attacks, threat surfaces, exploit entry points, or vulnerability candidates in one module, one crate, or an entire repository. Use for deterministic Rust inventory, security attack-surface mapping, what-if abuse-case brainstorming, threat-model-backed trust-boundary triage, crypto misuse review, exploitability prioritization, repeatable repo scans, and append-only attack inventory updates. Also triggers on attack surface, threat surface, vuln hunting, cryptographic gap, exploitability review, attack path analysis, residual risk, and trust-boundary audit.
argument-hint: 'scope=<path[,path2,...]> report_path=<path.md> db_path=<path.jsonl> [inventory_path=<path.md|jsonl>] [focus=<area[,area2,...]>] [threat_model=<path.md>] [max_variants=<N>] [admission_mode=ranked|single]'
---

# Attack Surfaces Create

This skill creates a skeptical, repeatable attack-surface discovery and admission workflow for Rust repositories and security-critical code.

The workflow has two mandatory layers:

1. A deterministic inventory layer that maps crates, modules, files, symbols, public APIs, call/dependency edges, feature flags, and high-risk boundary objects before security reasoning starts.
2. An LLM audit layer that runs deterministic question checks and broad adversarial brainstorming for every relevant component at file, module, cross-module, and cross-crate levels.

Final admission must include boundary-slice triage, threat-model pressure, security-control review, dependency/config scrutiny, and deep cryptography or protocol checks. Helper scripts may produce inventory and snapshots, but the assistant remains responsible for final security judgment.

## When to Use This Skill

Use this skill when the user wants to:

- find security attack surfaces in a module, crate, or whole repository
- hunt for cryptography attack surfaces or implementation gaps
- build or extend an append-only attack inventory database across repeated scans
- turn static code evidence into ranked, defensible attack-surface findings instead of a weak idea dump
- first build a project or crate map, then audit every relevant file, symbol, state object, and trust edge
- identify implementation nuances and required defensive controls for a discovered surface

Use it for both natural-language requests such as "find where this code can be attacked" and technical requests such as "run a trust-boundary scan", "map crypto exploit surfaces", or "enumerate vulnerability entry points".

## Core Guarantees

- The scan starts with deterministic inventory, not a single giant "check everything" prompt.
- The inventory covers crates, modules, files, public and private symbols, state objects, protocol objects, validators, parsers, serializers, crypto flows, storage flows, network/API boundaries, feature flags, and cross-crate dependencies when present.
- The audit combines fixed security questions with brainstorming-style "what if" attack exploration for each relevant element.
- The scan uses internal SSoT "String Seed of Thought" variation to generate multiple diverse attack attempts.
- Every attempt gets its own internal random seed string.
- Internal reasoning and seed strings are never exposed to the user-facing report.
- The assistant performs final admission and appends to the attack database only after final judgment.

## SSoT Random Seed Contract (LLM-Only)

- Generate one seed per variant before discovery starts.
- Seed format: lowercase alphanumeric string, fixed length `16`.
- Recommended alphabet: `a-z0-9`.
- Default mode: non-deterministic seeds per run.
- Reproducible mode (only when explicitly requested): derive each seed deterministically from stable inputs (`scope`, `db_path`, `variant_index`, optional user salt).
- Never print raw seeds in user output, reports, or persisted JSONL rows.
- Use each seed to rotate discovery axes (boundary slice priority, attacker capability framing, control-pressure order, and crypto/protocol stress focus).
- Do not reuse a seed inside one run.
- Apply a strict 3-role SSoT model flow per variant:
   - `Generator`: proposes candidate surfaces and evidence paths.
   - `Critic`: attacks reachability assumptions, attacker realism, and mitigation claims.
   - `Selector`: admits only candidates that pass skeptical gate, control review, and uniqueness checks.

## Safety Boundary

- This skill is defensive-only.
- It is allowed to discover and explain vulnerabilities, threat-model gaps, and hardening actions.
- It must not provide weaponized exploit instructions.
- It must not invent vulnerabilities or overstate impact.

## Attempt Policy

- Default attempts: `12`
- Suggested scaling:
   - small module: `12-24`
   - crate: `16-32`
   - whole repository: `32-96`
- The scan should prefer broad hypothesis generation and strict rejection over early acceptance.
- Whole-architecture runs must be executed in waves. Do not try to reason about the whole repository from one context window.

## Deterministic Inventory Layer

Build the inventory before generating attack hypotheses.

Use repository-local evidence and available tools in this order, degrading gracefully when a tool is absent:

1. `rg --files` for file discovery.
2. `cargo metadata --format-version 1` for workspace/crate/dependency graph.
3. `cargo tree` for resolved dependency and feature visibility.
4. Rust source inspection and `scripts/rust_symbol_index.py` for files, modules, structs, enums, traits, impl blocks, functions, constants, statics, macros, unsafe blocks, and test-only symbols.
5. Optional local tools when already installed: rustdoc JSON, rust-analyzer, cargo geiger, cargo audit, cargo deny, cargo clippy, cargo nextest, cargo llvm-cov.

The inventory must identify, when present:

- Cargo manifests, workspace members, features, `build.rs`, examples, tests, benches, fuzz targets, CI/build files, and generated-code boundaries
- public API, private functions, constructors, destructors/drop behavior, trait implementations, validators, parsers, serializers, deserializers, and error paths
- stateful objects, protocol objects, transaction or consensus objects, storage reads/writes, network/API boundaries, operator/admin/debug surfaces
- crypto operations, proof verification, signatures, commitments, nullifiers, transcripts, randomness, nonce/key handling, and secret handling
- unsafe/FFI/global/static/shared mutable state, async/concurrency primitives, panic/unwrap/expect on security-critical paths
- cross-module and cross-crate dependency edges, caller/callee assumptions, trait/API contracts, serialization/protocol compatibility, and feature-flag drift

Do not proceed to final admission until the inventory states what was inspected, what was missing, and which high-risk areas need manual review.

## Whole-Architecture Mode

Use this mode when `scope` is the repository root, the workspace root, or more than one crate.

1. Parse the root `Cargo.toml` and `cargo metadata` package list first.
2. Classify every workspace member into architecture lanes before auditing details:
   - core protocol, assets, rights, validation, and state transition lane
   - cryptography, proof, commitment, signature, KDF, AEAD, hash, domain separation, and Tari facade lane
   - wallet secret handling, key management, backup/import/export, RPC, persistence, scan, claim, and nullifier lane
   - storage, checkpoint, serialization, settlement, snapshot, redb/backend, and JMT/proof lane
   - rollup node, DA adapter, settlement theorem, inclusion, output proof, and node RPC lane
   - runtime aggregators, validators, watchers, evidence export, publication, and failover lane
   - network/RPC/onion routing, transport identity, session, admission, relay, exit, and telemetry lane
   - utilities, I/O, codec, config, RNG, time, logging, metrics, compression, and OS hardening lane
   - simulator, fixtures, tests, benches, fuzz, CI, build, feature flags, and supply-chain lane
3. For Z00Z workspaces, load local canonical architecture evidence when present:
   - `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`
   - `.github/requirements/Tari-Crypto-Integration-Z00Z.md`
   - `.github/requirements/Tari-Crypto-Components-Cookbook.md`
   - `Cargo.toml` workspace members and crate-local `Cargo.toml` files
   Use these files to seed invariants and ownership boundaries, but verify concrete findings against source code.
4. Build at least one cross-crate trust path per architecture lane that exists in the workspace.
5. Audit high-impact lanes first, but keep every lane in coverage accounting.

Whole-architecture findings should prefer boundary-crossing failures over isolated local smells.

## Anti-Empty Run Gate

A no-candidate outcome is allowed only after all of these are true:

- workspace members or scoped files were inventoried and summarized
- every present mandatory review slice was either analyzed or explicitly marked `needs manual review`
- every present architecture lane was mapped to at least one file/module owner
- high-impact lanes had at least one concrete trust path checked from entry point to sink
- deterministic questions and brainstorming prompts produced a scenario matrix with accepted, rejected, deferred, or manual-review states
- at least one rejected or deferred candidate per high-impact lane is documented in the report summary
- existing controls were checked before rejecting candidates as mitigated
- duplicate checks against `db_path` were performed before deciding that no new accepted finding exists

If these conditions are not met, emit an `incomplete audit` result instead of `no candidate`.

No accepted finding is not an empty run. The report must still include inventory coverage, scenario matrix, rejected/deferred candidates, manual-review gaps, and next verification targets.

## Architecture Invariant Catalog

Use these invariants as first-class attack prompts when the repository resembles Z00Z:

- business logic must not bypass `z00z_utils` abstractions for I/O, serialization, config, time, RNG, logging, and metrics
- application code must use the `z00z_crypto` facade rather than direct Tari imports, except inside explicit vendor/backend integration boundaries
- `crates/z00z_crypto/tari/` is read-only vendor code and must not become an application-owned modification surface
- cryptographic operations must preserve domain separation, nonce/RNG discipline, canonical serialization, transcript/statement binding, and fail-closed verification
- wallet code must not persist, log, clone, serialize, or expose secrets outside approved wrappers and redaction boundaries
- storage and settlement code must bind roots, proofs, checkpoints, versions, route/publication data, and serialized statements to the exact state transition they claim
- validators, aggregators, watchers, and rollup-node code must not confuse observation, publication, validation, settlement, or planner authority
- networking/RPC surfaces must not implicitly trust transport identity, operator-only routes, debug methods, unbounded request size, replayed requests, or stale session state
- feature flags, tests, fixtures, simulators, and benches must not compile weaker production behavior or hide missing controls
- errors, logs, telemetry, and metrics must not leak secrets, proof witnesses, private wallet metadata, or sensitive topology

## Deterministic And Brainstorming Question Layers

For every relevant file, symbol, object, and trust edge, apply both layers:

1. Deterministic checks:
   - What does this component accept, trust, transform, persist, verify, or emit?
   - Which invariants must hold before and after it runs?
   - Which caller assumptions are required for safety?
   - Which downstream modules trust its output?
   - Which controls exist, and are they missing, partial, or present?
   - Which tests, fuzz targets, assertions, or type-level constraints cover the invariant?

2. Brainstorming attack prompts:
   - What if this function is modified maliciously, returns wrong data, silently fails, or is bypassed?
   - What if this object is missing, disabled, stale, corrupted, duplicated, replayed, reordered, or partially initialized?
   - What if illegal operations are attempted against this object or state transition?
   - What if validation is skipped, caller assumptions are false, upstream input is adversarial, or downstream modules over-trust the output?
   - What if feature flags compile a weaker path?
   - What if panic, unwrap, expect, timeout, allocation, or recursion creates denial of service?
   - What if serialization format, canonicalization, versioning, or protocol compatibility changes?
   - What if concurrency creates a race, reentrancy, TOCTOU, deadlock, cancellation hole, or lost update?
   - What if cryptographic domain separation, nonce discipline, randomness, key handling, transcript binding, proof binding, or verifier fail-closed behavior is wrong?

Keep brainstorming broad in the internal inventory. Only promote candidates that survive evidence, reachability, control review, and exploitability gates.

## Four-Level Attack Analysis

Analyze each promoted and high-risk inventory item at four levels:

- File level: local invariants, local input/output abuse, unsafe assumptions, missing tests.
- Module level: module boundary assumptions, public vs private API misuse, state transition correctness, illegal operation handling.
- Cross-module level: trust chains, caller/callee assumptions, validation gaps, confused-deputy risk, replay/reordering risk, inconsistent error semantics.
- Cross-crate level: dependency trust, feature flag risk, security advisories, trait/API contracts, serialization/protocol compatibility, workspace-wide invariants.

## Advanced Analysis Lenses

Every run must apply these lenses before admitting any candidate:

- Threat model lens:
   - identify attacker classes, protected assets, trust boundaries, and realistic attacker capabilities
   - reject findings that require impossible assumptions about control, privilege, or reachability
- Security control lens:
   - validate whether existing controls already mitigate the candidate (auth checks, fail-closed checks, replay controls, input guards, rate or resource boundaries)
   - distinguish missing controls from present-but-incomplete controls
- Cryptography lens:
   - check nonce or randomness discipline, domain separation, transcript or context binding, canonical serialization, and fail-closed verification behavior
   - escalate boundary-critical crypto misuse (proof-binding gaps, verification bypass patterns, weak randomness near secret operations)
- Exploitability lens:
   - prioritize attack paths that cross meaningful boundaries with realistic attacker influence
   - down-rank theoretical-only patterns with no practical trigger path
- Residual-risk lens:
   - every accepted finding must document what remains risky after the proposed fix
   - never imply full closure when only partial hardening is supported by evidence

## Mandatory Review Slices

Each run must map the scoped code into boundary slices before final admission. Use only the slices that actually exist in the scoped code, but do not skip a slice silently when evidence suggests it is present:

- external input and parser slice
- authn, authz, and capability boundary slice
- secret handling, storage, and logging slice
- cryptographic and proof-verification slice
- replay, nonce, uniqueness, and state-consumption slice
- configuration, feature-flag, and deployment-default slice
- dependency, build, CI, and supply-chain slice
- operator, admin, and debug-only surface slice

If the scope contains a high-assurance cryptographic, protocol, or proof system path, treat the cryptographic and proof-verification slice as mandatory even when the scanner reports no strong candidate on the first pass.

## Required Inputs

- `scope`: one or more paths inside the repository
- `report_path`: markdown report destination
- `db_path`: JSONL attack inventory destination

Optional:

- `max_variants`: how many SSoT attempts to generate, defaults to `12`
- `inventory_path`: optional standalone deterministic inventory artifact
- `focus`: optional focus areas such as cryptography, blockchain consensus, wallet logic, transaction validation, storage, networking, parser/deserializer, permissions, state transitions, async/concurrency, unsafe code, FFI, or feature flags
- `threat_model`: optional threat model file to load before audit
- `admission_mode`: `ranked` by default; use `single` only when the user asks for one strongest candidate

## How It Works

1. Resolve the scope and architecture context.
   - Accept module, crate, or repository-level paths.
   - Prefer the smallest meaningful scope first.
   - Load `threat_model` when provided.
   - Identify entry points, sensitive assets, trust boundaries, critical data flows, privileged actors, external dependencies, feature flags, operator surfaces, and mandatory review slices.
   - For whole-architecture scope, classify every workspace member into architecture lanes before symbol-level audit.

2. Build deterministic inventory.
   - Run `scripts/rust_inventory.sh <scope>` when the scope is Rust and shell execution is available.
   - Run `scripts/rust_symbol_index.py <scope>` for a line-backed symbol index when Python is available.
   - Run `scripts/rust_security_snapshot.sh <scope>` when a quick dependency/config/security tool snapshot is useful.
   - If helpers cannot run, reproduce the same inventory manually with `rg`, `cargo metadata`, `cargo tree`, and source inspection.
   - Persist `inventory_path` when requested and summarize inventory coverage in `report_path`.

3. Map inventory to attack-surface objects.
   - Group items by crate, module, file, symbol, state object, protocol object, parser/serializer, validator, crypto flow, storage flow, network/API edge, feature flag, dependency edge, and operator/debug edge.
   - Mark each group with present boundary slices and missing evidence.
   - Identify high-risk first-pass targets, but keep low-risk items in coverage accounting.
   - Create architecture-lane coverage rows before candidate selection starts.

4. Run LLM-owned seeded discovery over the inventory.
   - Generate internal variant seeds in-memory.
   - For each seed, vary boundary-slice priority, attacker model, control-pressure order, and crypto/protocol focus.
   - For each relevant inventory item, apply deterministic questions and brainstorming prompts at file, module, cross-module, and cross-crate levels.
   - Keep a broad candidate inventory with evidence, violated assumptions, illegal operations, abuse cases, and preliminary risk notes.
   - For whole-architecture scope, force at least one cross-crate trust-path scenario per present architecture lane.

5. Expand discovery across under-covered slices.
   - Add assistant-side discovery for dependency exposure, CI or deployment trust, config fail-open defaults, trusted setup assumptions, verifier or transcript binding, feature-flag drift, admin/debug surfaces, async/concurrency, unsafe/FFI, and storage/serialization boundaries.
   - Treat seeded discovery as incomplete until every present slice has been pressure-tested or explicitly marked as needing manual review.

6. Triage candidates with adversarial pressure.
   - Group candidates by boundary slice, failure mode, symbol, and trust edge.
   - Promote only candidates with concrete evidence and realistic exploitation path.
   - Keep brainstorming notes separate from formal findings.
   - Mark non-promoted candidates as rejected, duplicate, low evidence, needs manual review, or deferred.

7. Run embedded audit packs on the shortlist.
   - Threat-model pack: identify protected assets, attacker classes, trust boundaries, realistic attacker capabilities, and STRIDE-style pressure where useful.
   - Security-control pack: inspect auth checks, schema validation, replay controls, rate/resource boundaries, secret redaction, config defaults, dependency risk, and CI/supply-chain trust surfaces.
   - Cryptography and protocol pack: inspect randomness, nonce discipline, domain separation, transcript and statement binding, canonical encoding, constant-time risk, verifier fail-closed behavior, trusted setup assumptions, stealth/ownership derivation, and proof-system boundary mismatches.
   - Formal-closeout pack: define defensive implementation contract, regression coverage, and residual risk before final admission.

8. Apply the skeptical gate in the assistant workflow.
   - Layer 1: self-audit for concrete claim extraction from code evidence.
   - Layer 2: verification gate for reachability and attacker capability realism.
   - Layer 3: adversarial review to reject false positives and overstated impact.
   - Layer 4: control-coverage review for existing mitigations and residual-risk honesty.

9. Admit findings.
   - In default `ranked` mode, admit every candidate that passes the gate and rank by severity, exploitability, confidence, and blast radius.
   - In `single` mode, select only the strongest admitted candidate.
   - If no candidate survives but the anti-empty run gate is not satisfied, emit `incomplete audit`.
   - If no candidate survives and the anti-empty run gate is satisfied, emit the no-candidate outcome with inventory coverage, scenario matrix, rejected/deferred candidates, and manual-review gaps.

10. Persist only verified findings.
   - The attack database is append-only.
   - Before append, run semantic doublecheck against existing JSONL findings.
   - Uniqueness is enforced by attack-surface meaning (context), not by `id`.
   - Do not append if the same surface already exists under a different `id`.
   - DB append is executed only after the assistant final decision.
   - Rejected, deferred, brainstorming-only, or unverifiable ideas are never written as accepted findings.

11. Use the standard output contract.
   - The inventory, matrix, finding, and defense shapes are defined in [FORMS.md](./FORMS.md).
   - Taxonomy, question banks, rejection rules, and skepticism rules live in [REFERENCE.md](./REFERENCE.md).

## Reference Execution Sequence

1. Parse inputs: `scope`, `report_path`, `db_path`, `inventory_path`, `focus`, `threat_model`, `max_variants`, `admission_mode`.
2. Run deterministic inventory and record coverage gaps.
3. Build crate/module/file/symbol/dependency map and boundary-slice map.
4. For whole-architecture scope, classify workspace members into architecture lanes and build lane coverage rows.
5. Generate `max_variants` internal seeds.
6. Produce candidate inventory per seed and per analysis level with evidence excerpts.
7. Merge and dedupe candidates by semantic context.
8. Run deterministic question checks on each high-risk item.
9. Run brainstorming what-if prompts on each high-risk item.
10. Build scenario matrix and illegal-operation/abuse-case list.
11. Run threat-model pack on shortlist.
12. Run security-control pack on shortlist.
13. Run cryptography/protocol pack on shortlist.
14. Run adversarial skeptical gate and reject weak candidates.
15. Apply the anti-empty run gate.
16. Admit ranked findings, one strongest finding only when `admission_mode=single`, no-candidate outcome, or incomplete-audit outcome.
17. Build final report using [FORMS.md](./FORMS.md).
18. Doublecheck semantic uniqueness against existing `db_path` entries.
19. Append accepted findings to `db_path` only after final assistant admission.

## Workflow Rules For the Assistant

1. Be skeptical.
   - Assume most static hypotheses are wrong until corroborated.
   - Ask: is it reachable, attacker-controlled, and security-relevant under realistic capability?

2. Reject weak ideas explicitly in assistant review.
   - If no candidate survives assistant skeptical review, apply the anti-empty run gate before choosing `no candidate`.
   - Do not promote a weak candidate just to fill the report.

3. Explain the main vulnerability.
   - State the trust boundary.
   - State the concrete failure mode.
   - State why the implementation detail matters.

4. Preserve breadth without dumping noise.
   - Cover every relevant component through inventory and matrix accounting.
   - Do not turn the final accepted findings into a brainstorming dump.
   - Use `needs manual review`, `deferred`, or `rejected` for ideas that are useful but not verified.

5. Keep the output practical.
   - Include exact file evidence.
   - Include implementation nuance.
   - Include the concrete defensive actions needed to harden the surface.
   - Include the relevant threat boundary and attacker capability statement.
   - Include the boundary slice and current control state.
   - Include residual risk after mitigation.

6. Preserve append-only history.
   - Reuse the same `db_path` across reruns when the user wants cumulative attack knowledge.
   - Append to `db_path` only after assistant final admission.

7. Never force a finding.
   - If no candidate passes verification and coverage is complete, emit a no-candidate report and stop.
   - If coverage is incomplete, emit incomplete audit and list the exact missing lanes, files, or trust paths.

## Admission And Ranking Rule

Default mode admits all candidates that pass the gate and ranks them by:

- highest realistic impact
- clearest code evidence
- strongest reachability and boundary crossing
- fewest uncertain assumptions
- most actionable fix and regression coverage

When only one candidate is allowed, the assistant chooses the top ranked finding using the same criteria.

Reject even interesting candidates if they are speculative, low-confidence, or rely on unrealistic attacker power.

## Escalation And Follow-Through

After a candidate is accepted by the assistant, use this skill itself to carry the follow-through expectations instead of depending on neighboring skills:

- deepen cryptographic or proof review when the finding crosses a proof, commitment, signature, transcript, stealth-address, nullifier, or trusted-setup boundary
- deepen security-control review when the finding depends on authz, rate controls, config defaults, secret handling, dependency trust, or CI behavior
- strengthen threat-model wording until attacker capability, boundary crossing, and protected asset are explicit and non-exaggerated
- require a regression test or equivalent verification artifact in the defense contract whenever code changes are proposed

## Output Contract

Each report must include:

- deterministic project inventory summary
- architecture-lane coverage table for whole-architecture scope
- object/function inventory table or compact per-crate/per-module equivalent
- attack surface per file for inspected high-risk files
- attack surface per module for inspected modules
- cross-module attack paths
- cross-crate attack paths
- what-if scenario matrix
- critical invariants
- abuse cases and illegal operations
- risk ranking
- recommended tests, fuzz targets, assertions, or verification artifacts
- suggested hardening changes
- incomplete-audit or no-candidate gate result when no finding is admitted

Each assistant-admitted finding must include:

- severity
- confidence
- exploitability
- category metadata (domain, and optional CWE)
- attack-surface class
- boundary slice
- protected asset
- trust boundary
- attacker capability model
- existing control state
- main vulnerability
- implementation nuance
- evidence list
- pro-con audit result
- verification result
- defense contract
- residual risk
- affected file path, symbol name, module/crate, attack scenario, violated assumption, blast radius, recommended fix, recommended unit/property/fuzz/integration test

Use the exact structure from [FORMS.md](./FORMS.md).

Persisted JSONL contract for accepted records:

- Key order starts with: `id`, `created_at`, `crate`, `severity`, `title`
- `created_at` format: `YYYY-MM-DD HH:MM:SS`
- `created_at` MUST be the real append-time timestamp for newly accepted findings; do not emit placeholder midnight values such as `00:00:00` for new rows
- Historical placeholder timestamps are allowed only for explicitly normalized legacy imports when the original time is unknown and the row is marked accordingly (for example `rule_id: legacy-import`)
- Persisted records MUST NOT include: `signature`, `scan_seed`, `variant_seed`, `seed_axes`
- Persisted rows remain append-only and store one accepted JSON object per line
- When appending to an existing JSONL database, preserve the surrounding whitespace style of that file; do not switch one row to compact JSON if neighboring accepted rows use spaced JSON

## Output Artifacts

The skill produces two primary artifacts:

- `report_path` (Markdown report)
   - Human-readable inventory, scenario matrix, risk ranking, and assistant decision result for one run.
   - Includes ranked assistant-admitted findings, an explicit no-candidate outcome, or an incomplete-audit outcome.
   - Uses the inventory, matrix, and card structures defined in [FORMS.md](./FORMS.md).

- `db_path` (append-only JSONL database)
   - Machine-readable cumulative inventory across reruns.
   - Stores only assistant-accepted verified findings as one JSON object per line.
   - Uses the persisted contract above (leading keys, timestamp format, and removed fields).
   - Enforces semantic de-duplication before write (same contextual surface is rejected even with a new `id`).
   - Rejected and unverifiable attempts are never written as accepted records.

Secondary artifacts (when requested):

- Structured assistant-side candidate inventory summary with duplicate hints for final judgment.
- `inventory_path` deterministic inventory output for large workspaces or resumed audits.

## Examples

### Example 1: Crypto-Focused Scan

```text
User: Find crypto attack surfaces in z00z_wallets and keep only serious ones.
Assistant: Build deterministic inventory for crates/z00z_wallets, run crypto/protocol what-if scenarios across file/module/cross-crate levels, reject weak hypotheses, and return ranked verified findings with standard attack-surface cards.
```

### Example 2: Incremental Attack Database Update

```text
User: Re-scan the wallet tx module and append only newly verified attack surfaces.
Assistant: Reuse the existing JSONL database path, rebuild inventory for the scoped tx module, skip semantic duplicates, and append only newly verified findings that pass audit and verification.
```

### Example 3: Single-Candidate Mode

```text
User: Audit this parser module and give me only the strongest attack surface.
Assistant: Build the deterministic inventory, run full what-if coverage internally, then emit only the top ranked admitted finding or a no-candidate outcome.
```

### Example 4: Argument Hint Usage

Use this argument shape when the user wants an explicit run configuration:

```text
scope=<path[,path2,...]> report_path=<path.md> db_path=<path.jsonl> [inventory_path=<path.md|jsonl>] [focus=<area[,area2,...]>] [threat_model=<path.md>] [max_variants=<N>] [admission_mode=ranked|single]
```

Concrete crate run:

```text
User: Run attack-surfaces-create scope=crates/z00z_wallets report_path=reports/attack-surfaces/z00z-wallets.md db_path=reports/attack-surfaces/attack-surfaces.jsonl inventory_path=reports/attack-surfaces/z00z-wallets-inventory.md focus=wallet,secret-handling,privacy,serialization max_variants=32 admission_mode=ranked
Assistant: Audit only crates/z00z_wallets, build a deterministic wallet inventory, focus on secret-handling and privacy boundaries, then write the report and append only verified findings to the JSONL database.
```

Whole `crates/` run:

```text
User: Run attack-surfaces-create scope=crates report_path=reports/attack-surfaces/z00z-crates.md db_path=reports/attack-surfaces/attack-surfaces.jsonl inventory_path=reports/attack-surfaces/z00z-crates-inventory.md focus=crypto,storage,wallet,networking,state-transitions,feature-flags max_variants=96 admission_mode=ranked
Assistant: Treat crates/ as whole-architecture scope, classify all workspace crates into architecture lanes, run wave-based inventory and attack brainstorming, and preserve per-lane coverage before admitting findings.
```

Specific planning phase run:

```text
User: Run attack-surfaces-create scope=.planning/phases/065-Attack-Surface report_path=reports/attack-surfaces/phase-065-attack-surface.md db_path=reports/attack-surfaces/attack-surfaces.jsonl inventory_path=reports/attack-surfaces/phase-065-attack-surface-inventory.md focus=planning-artifacts,threat-model,security-assumptions admission_mode=ranked
Assistant: Audit the phase artifacts as security design input, extract threat assumptions and unresolved attack-surface claims, then write verified findings only when they can be tied back to repository source evidence.
```

## Notes

- This skill is inventory-first and static-analysis-first with assistant-owned final judgment.
- Discovery is broad; admission is strict.
- If a stronger follow-up is needed after an assistant-admitted hit, run a deeper cryptographic correctness pass, a deeper security control audit, and a focused repository validation pass.
