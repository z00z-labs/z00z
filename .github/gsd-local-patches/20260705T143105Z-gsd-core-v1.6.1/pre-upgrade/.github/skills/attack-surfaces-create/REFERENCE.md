# Attack Surface Reference

## Scan Objective

Build a deterministic Rust inventory first, then produce ranked, repository-backed attack-surface findings from broad what-if analysis. The goal is complete coverage accounting plus strict admission, not a bag of weak suspicions.

Default output may contain multiple accepted findings. Use single-candidate output only when the user explicitly asks for the strongest candidate.

## Deterministic Inventory Reference

The inventory layer must map what exists before the audit layer decides what is vulnerable.

Required inventory dimensions when present:

- workspace members, crates, manifests, feature flags, dependency graph, `build.rs`, examples, tests, benches, fuzz targets, and CI/build files
- modules, files, public API, private functions, structs, enums, traits, impl blocks, constants, statics, macros, and unsafe blocks
- state objects, protocol objects, parsers, serializers, deserializers, validators, storage flows, network/API boundaries, operator/admin/debug surfaces
- crypto/proof/signature/commitment/nullifier/transcript/key/nonce/randomness/secret-handling flows
- async/concurrency primitives, shared mutable state, FFI/global/static boundaries, panic/unwrap/expect on security-critical paths
- cross-module caller/callee assumptions, cross-crate trait/API contracts, serialization/protocol compatibility, dependency and feature-flag trust edges

The report must state coverage gaps and manual-review areas explicitly. A missing tool is not a blocker if source inspection can cover the same dimension.

## Whole-Architecture Reference

Whole-architecture scope means repository root, workspace root, or multiple crates.

Map every workspace member to one or more architecture lanes before deep audit:

- core protocol, assets, rights, validation, and state transitions
- cryptography, proof, commitment, signature, KDF, AEAD, hash, domain separation, and crypto facade
- wallet secret handling, key management, backup/import/export, RPC, persistence, scan, claim, and nullifier state
- storage, checkpoint, serialization, settlement, snapshot, backend, and proof/state-root persistence
- rollup node, DA adapter, settlement theorem, inclusion verification, output proof, and node RPC
- runtime aggregators, validators, watchers, evidence export, publication, and failover
- network/RPC/onion routing, transport identity, session, admission, relay, exit, and telemetry
- utilities, I/O, codec, config, RNG, time, logging, metrics, compression, and OS hardening
- simulator, fixtures, tests, benches, fuzz, CI, build, feature flags, and supply-chain

For Z00Z repositories, seed the lane map from local files only:

- `Cargo.toml`
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`
- `.github/requirements/Tari-Crypto-Integration-Z00Z.md`
- `.github/requirements/Tari-Crypto-Components-Cookbook.md`
- crate-local manifests and `src` trees

Do not use planning graphs or architecture summaries as concrete finding evidence. Concrete findings require source-code evidence.

## Anti-Empty Gate Reference

A scan may report no accepted findings only after it proves it was not shallow:

1. Workspace members or scoped files were inventoried.
2. Every present mandatory review slice was analyzed or marked `needs manual review`.
3. Every present architecture lane has at least one owner file/module.
4. High-impact lanes have at least one checked trust path from entry point to sink.
5. The scenario matrix includes accepted, rejected, deferred, or manual-review states.
6. At least one rejected or deferred candidate is summarized for each high-impact lane.
7. Existing controls were checked before rejecting candidates as mitigated.
8. The database was checked for semantic duplicates before reporting no new accepted finding.

If any item is missing, the result is `incomplete audit`, not `no candidate`.

No accepted finding must still produce useful artifacts: coverage table, scenario matrix, rejected/deferred candidates, manual-review gaps, and next verification targets.

## Z00Z Architecture Invariants

When auditing Z00Z, pressure-test these invariants against source code:

- `z00z_utils` remains the single abstraction owner for I/O, serialization, config, time, RNG, logging, metrics, and compression boundaries.
- `z00z_crypto` remains the facade for application cryptography; direct Tari usage outside vendor/backend boundaries is suspicious.
- `crates/z00z_crypto/tari/` remains read-only vendor code and must not become an application-owned patch surface.
- Domain separation, nonce/RNG discipline, canonical serialization, transcript/statement binding, and fail-closed verification are preserved across every crypto/proof boundary.
- Wallet secrets, passwords, seed material, nullifiers, and key-derived material do not cross logging, telemetry, storage, RPC, or backup boundaries without explicit protection.
- Storage roots, checkpoints, settlements, serialized statements, route/publication data, and proof payloads bind to the exact state transition they claim.
- Validators, aggregators, watchers, and rollup node code do not confuse observation, publication, validation, settlement, or planner authority.
- Network/RPC/onion surfaces do not trust transport identity, operator/debug methods, request size, replayed requests, stale sessions, or placeholder traits as production controls.
- Feature flags, tests, fixtures, simulators, and benches do not compile weaker production behavior or hide missing controls.
- Errors, logs, telemetry, and metrics do not leak secrets, proof witnesses, private wallet metadata, or sensitive topology.

## SSoT Variation Axes

Each variant uses a separate internal random seed string generated by the LLM and varies:

- threat model
- cryptographic primitive
- failure scenario
- implementation constraint
- adversarial angle

The variation exists to diversify search order and explanatory frame, not to lower the evidence threshold.

## SSoT Seed And Model Contract (Canonical)

Treat this section as normative for all attack-surfaces-create runs.

- Generate one internal seed per variant before discovery starts.
- Seed format: lowercase alphanumeric, fixed length `16`, alphabet `a-z0-9`.
- Default mode: non-deterministic seeds per run.
- Reproducible mode (only when explicitly requested): derive deterministic seeds from stable inputs (`scope`, `db_path`, `variant_index`, optional user salt).
- Never expose raw seeds in user output, report markdown, or persisted JSONL rows.
- Never reuse a seed inside one run.
- Run every variant through strict 3-role SSoT model flow:
  - `Generator`: proposes candidate surfaces and evidence paths.
  - `Critic`: attacks reachability, attacker capability realism, and mitigation assumptions.
  - `Selector`: admits only candidates that pass skeptical gates, control review, and uniqueness checks.

## Execution Sequence Reference

Follow this exact order:

1. scope ingestion and threat model loading
2. deterministic inventory and coverage-gap recording
3. crate/module/file/symbol/dependency and boundary-slice mapping
4. seed generation (`max_variants`)
5. candidate generation per seed, component, and analysis level
6. deterministic question checks and brainstorming what-if prompts
7. semantic dedupe and shortlist
8. threat model pressure
9. security control coverage review
10. cryptography or protocol deep checks
11. adversarial skeptical gate
12. anti-empty gate
13. ranked admission, single-candidate admission only on request, no-candidate, or incomplete-audit result
14. report assembly from FORMS contract
15. semantic uniqueness check against JSONL
16. append-only DB write after final admission

## Attack Surface Classes

- `secret-exposure`
- `proof-verification-bypass`
- `fail-open-validation`
- `weak-randomness-for-crypto`
- `constant-time-risk`
- `panic-driven-security-boundary`
- `unsafe-serialization-of-sensitive-state`
- `dependency-supply-chain-exposure`
- `config-or-deployment-fail-open`
- `trusted-setup-or-proof-assumption-gap`
- `operator-or-debug-surface`
- `state-transition-invariant-break`
- `parser-or-deserializer-confusion`
- `cross-module-trust-chain-gap`
- `cross-crate-contract-drift`
- `feature-flag-weakened-path`
- `async-concurrency-state-race`
- `storage-consistency-or-corruption`

## Boundary Slice Inventory

Before final admission, map evidence to one primary boundary slice and any
secondary slices that materially amplify risk:

- external input and parser slice
- authn, authz, and capability boundary slice
- secret handling, storage, and logging slice
- cryptographic and proof-verification slice
- replay, nonce, uniqueness, and state-consumption slice
- configuration, feature-flag, and deployment-default slice
- dependency, build, CI, and supply-chain slice
- operator, admin, and debug-only surface slice

Reject or defer candidates that cannot identify a primary slice.

## Required Analysis Domains

Apply when relevant to scope and stack:

- General security: input validation, authn/authz boundaries, replay control, denial-of-service surfaces, secret management.
- Cryptography: RNG/nonce safety, domain separation, canonical encoding, signature or proof binding, verifier fail-closed behavior.
- Threat model: attacker capability realism, trust boundaries crossed, assets at risk, entry-point reachability.
- Rust-specific: `unsafe` boundaries, `unwrap` or `expect` in security-critical paths, secret type formatting or serialization exposure.

## Deterministic Question Bank

Ask these questions for every high-risk component before brainstorming:

- What inputs does this component accept, and which are attacker-controlled?
- What state, secret, proof, protocol value, or permission does it trust?
- What invariant must hold before and after this function or object is used?
- What output is trusted by downstream code?
- What validation, canonicalization, replay control, fail-closed check, or permission gate exists?
- What happens on malformed input, stale state, partial initialization, duplicate data, reordered data, timeout, cancellation, or storage failure?
- Which test, fuzz target, assertion, type constraint, or review artifact proves the invariant?

## Brainstorming What-If Bank

Use these prompts broadly, but do not treat brainstormed ideas as findings until verified:

- What if this function is modified maliciously, returns wrong data, silently fails, or is bypassed?
- What if this object is missing, disabled, stale, corrupted, duplicated, replayed, reordered, or partially initialized?
- What if an illegal operation is attempted against this object or state transition?
- What if validation is skipped, caller assumptions are false, upstream input is adversarial, or downstream modules over-trust this output?
- What if feature flags compile a weaker path or disable a required check?
- What if panic, unwrap, expect, recursion, allocation, timeout, or cancellation creates denial of service?
- What if serialization format, canonicalization, versioning, or protocol compatibility changes?
- What if concurrency creates race, reentrancy, TOCTOU, deadlock, cancellation hole, or lost update?
- What if cryptographic domain separation, nonce discipline, randomness, key handling, transcript binding, proof binding, or verifier fail-closed behavior is wrong?

## Four-Level Analysis Reference

- File level: local invariants, local input/output abuse, unsafe assumptions, missing tests.
- Module level: module boundary assumptions, public vs private API misuse, state transition correctness, illegal operation handling.
- Cross-module level: trust chains, caller/callee assumptions, validation gaps, confused-deputy risk, replay/reordering risk, inconsistent error semantics.
- Cross-crate level: dependency trust, feature flag risk, version/security advisories, trait/API contracts, serialization/protocol compatibility, workspace-wide invariants.

## Threat Modeling Expansion

In addition to the baseline trust-boundary statement, pressure-test each
promoted candidate with:

- protected asset and security property: confidentiality, integrity,
  authenticity, uniqueness, unlinkability, availability
- attacker class: external caller, authenticated user, compromised dependency,
  malicious operator, build-pipeline influence, privileged insider
- attacker entry point, control surface, and sink
- trust transition path between the attacker-controlled edge and the protected
  asset
- mini data-flow narrative: what enters, what is transformed, what is
  persisted, what is verified, what leaves

Use STRIDE-style prompts when helpful, but do not force every STRIDE category
onto every candidate.

## Threat Model Baseline (Mandatory)

For each candidate, state these elements explicitly before acceptance:

- protected asset and primary security property at risk
- attacker class and capability assumptions
- trust boundary crossed
- attacker-controlled entry point and sink
- attack path realism under expected runtime conditions

Reject candidates that cannot name at least one concrete boundary crossing with realistic attacker control.

## Security Audit Expansion

For each promoted candidate, inspect these surfaces when they are present in
scope:

- dependency trust and supply-chain admission
- secret storage, redaction, and debug formatting
- configuration defaults and feature-flag drift
- deployment, CI, release, and build-pipeline assumptions
- operator-only or admin-only surfaces that can become reachable in production
- availability and resource-exhaustion abuse paths

If a candidate depends on one of these surfaces but the scan does not inspect
it, keep the candidate in shortlist state and do not promote it yet.

## Security Control Coverage Review

For each promoted candidate, classify control state:

- `missing`: no relevant control exists on the path
- `partial`: control exists but does not cover the vulnerable path or edge case
- `present`: control exists and meaningfully reduces exploitability

Control families to check:

- authentication and authorization gates
- input and schema validation
- replay and nonce-consumption controls
- fail-closed verification behavior
- rate and resource-abuse controls
- logging and secret redaction behavior near failure paths

Do not escalate severity while ignoring existing compensating controls.

## Cryptography Deep Checks

When cryptographic logic is in scope, require explicit checks for:

- nonce or randomness generation safety near secret operations
- domain separation across hashes, KDF contexts, commitments, nullifiers, and transcript domains
- canonical serialization before signing, hashing, or proof verification
- full context binding (chain, version, role, statement context, proof relation)
- fail-closed verifier behavior under malformed or adversarial inputs
- duplicated or conflicting relation elements (duplicate identifiers, overlap conditions, mismatched roots)

Escalate to high-risk review when a candidate indicates:

- proof-verification bypass potential
- weak randomness in key, nonce, or proof-related generation paths
- statement or transcript binding omissions across trust boundaries

## Cryptography And Protocol Expansion

When cryptographic or proof-system logic is in scope, also check for:

- trusted setup assumptions and whether the implementation silently relies on
  them
- transcript personalization and cross-protocol separation
- proof statement completeness, including roots, nullifiers, identities,
  versioning, and chain or role binding
- stealth-address or ownership-derivation collisions and relation drift
- AEAD context binding and associated-data completeness
- constant-time or secret-dependent branch and comparison risk near sensitive
  values
- proof-system verifier or parser behavior under malformed or adversarial
  payloads

## Verification Gate

A candidate is admitted only if all of the following are true:

1. It has concrete code evidence.
2. It crosses a meaningful trust boundary.
3. It explains one main vulnerability, not a vague smell.
4. Its evidence is not confined to tests, docs, comments, or placeholders.
5. It is reachable through a realistic execution path.
6. The attacker capability is realistic for the claimed boundary.
7. Existing mitigations are explicitly checked and accounted for.
8. Its skeptical score exceeds the acceptance threshold.
9. Its attacker capability model is realistic and explicit.
10. Its residual-risk statement is concrete and non-empty.

## Formal Closeout Requirements

Before writing an accepted finding, the final card must include:

- primary boundary slice and any secondary amplifying slices
- protected asset and security property at risk
- explicit control-state classification (`missing`, `partial`, or `present`)
- exact file-backed evidence, not abstract summaries only
- one main vulnerability statement rather than a cluster of smells
- a defense contract that includes at least one regression test or focused
  verification artifact expectation when code changes are proposed
- an honest residual-risk note that does not imply total closure

## Severity And Confidence Taxonomy

- Severity:
   - `critical`: direct key compromise, signature or proof bypass, unauthorized spend.
   - `high`: realistic privilege or integrity break across a trust boundary.
   - `medium`: real but constrained security impact.
   - `low`: defense-in-depth with limited direct impact.
- Impact categories:
   - fund loss
   - privacy break
   - double spend
   - consensus split
   - state corruption
   - denial of service
   - privilege escalation
   - data leakage
   - cryptographic unsoundness
   - invariant violation
- Confidence:
   - `high`: exact path and broken check are evidenced.
   - `medium`: strong evidence with one bounded uncertainty.
   - `low`: insufficient for accepted findings.
- Exploitability:
  - `high`: normal attacker capability can trigger reliably.
  - `medium`: requires specific state, timing, or partial privilege.
  - `low`: requires unrealistic or privileged conditions.

## Rejection Rules

Reject a candidate when any of these hold:

- evidence comes only from tests or markdown
- evidence is only a comment with no implementation support
- the candidate cannot name a concrete sink or security boundary
- the candidate is semantically duplicated by an already accepted finding
- the candidate depends on guessing runtime behavior not present in code
- the candidate ignores an existing mitigation that invalidates the claimed exploit path
- the candidate cannot separate proven behavior from assumptions

### Mandatory Reject-Unless Patterns

- Weak randomness claim: reject unless concrete insecure RNG usage is shown near crypto boundary.
- Missing authorization claim: reject unless reachable path lacks caller or capability check.
- Replay claim: reject unless missing nonce/domain/consumed-state control is evidenced.
- Privacy leak claim: reject unless concrete linkability channel or metadata leak is evidenced.

## Pro-Con Audit Semantics

### Pros

- multiple corroborating matches
- multiple production files
- security-relevant tokens near the evidence
- clear trust boundary and realistic attacker path

### Cons

- isolated or ambiguous match
- remediation already appears nearby
- evidence depends on naming only
- finding is actually a quality issue with no security consequence
- attacker capability assumptions are unrealistic for the claimed boundary
- controls already present fully block the claimed path

Only candidates with stronger pros than cons can pass.

## Mandatory Skeptical Questions

- Is the path actually reachable?
- Is attacker-controlled input or state present?
- Is a security property actually violated?
- Is there an existing mitigation that weakens or invalidates the claim?
- Is the impact statement precise and non-exaggerated?
- What control currently blocks or weakens this path, and why is it still insufficient?
- Which assumption, if disproven, would invalidate this finding?

## Defense Contract Requirements

Every admitted finding must include concrete hardening work:

- prevent the vulnerable behavior
- preserve fail-closed semantics
- tighten the boundary where the flaw manifests
- add regression coverage for the discovered surface
- note any remaining residual risk honestly

## Candidate Lifecycle (Internal)

Before final report admission, each candidate must pass through:

1. `discovered`: plausible scoped suspicion with initial evidence stub
2. `triaged`: grouped by boundary slice and checked for duplicates
3. `shortlisted`: survived slice review and remains worth formal audit
4. `promoted`: selected for deep verification based on exploitability and evidence quality
5. `accepted` or `rejected`: final gate outcome

Only `accepted` candidates may enter the append-only database.

## Severity Calibration Hints

Use these tie-breakers when severity is ambiguous:

- raise severity when the path can break asset integrity across a live trust boundary
- raise severity when attacker prerequisites are low and trigger path is deterministic
- lower severity when exploitation requires privileged or unrealistic preconditions
- lower confidence when critical claims depend on unverified runtime assumptions

## Database Semantics

The attack inventory is append-only JSONL.

Each entry must record:

- scope level and scope paths
- admitted attack class
- vulnerability statement
- evidence
- audit outcome
- verification outcome
- defenses

Each entry must not record raw internal seeds, seed axes, or hidden reasoning. Do not persist `signature`, `scan_seed`, `variant_seed`, or `seed_axes`.

Never write rejected attempts into the accepted database.
