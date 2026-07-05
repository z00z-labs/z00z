# Create Tests Reference

## Artifact Precedence

Read the phase artifacts in this order and resolve conflicts conservatively:

1. `*-VERIFICATION.md` or `*-VALIDATION.md`
2. `*-SUMMARY.md` or `SUMMARY.md`
3. `*-CONTEXT.md` or `CONTEXT.md`
4. `*-TODO.md`, `TODO.md`, or `todo.md`
5. every `*-PLAN.md`

If a later artifact is more ambitious than an executed artifact, do not promote
the unexecuted claim into the test baseline without saying so.

## Repo Artifact Conventions

In this repository, the planning artifacts already separate test intent from
test implementation order.

- `NNN-TEST-SPEC.md` is the detailed behavioral contract.
- `NNN-TESTS-TASKS.md` is the concrete implementation-order breakdown used to
  sequence test work.
- `NNN-TEST-PLAN.md` is not the historical primary format in this repo.
  If requested, treat it as a synchronized compatibility alias rather than as a
  separate source of truth.

Current workspace evidence:

- `.planning/phases/000/029-crypto-audit-wallets/029-TEST-SPEC.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-TESTS-TASKS.md`
- `.planning/ROADMAP.md` references `029-TESTS-TASKS.md`

When integrating with GSD-oriented planning, prefer emitting
`NNN-TESTS-TASKS.md` and optionally mirror it into `NNN-TEST-PLAN.md` if a user
or surrounding workflow explicitly asks for that name.

## Mode Selection

### Verification-Backed

Use when the phase has both summary evidence and verification evidence.

Expected outputs:

- full `*-TEST-SPEC.md`
- approval plan
- executable tests

### Fallback-Ready

Use when the phase does not yet have full completion artifacts, but the plans,
context, and TODOs are strong enough to define truthful coverage.

Expected outputs:

- truthful `*-TEST-SPEC.md`
- explicit blocker note if implementation-time test generation would be fake

Do not mark the phase complete in this mode.

## Coverage Heuristics

### Strong Unit Candidates

- pure value transforms
- validation helpers
- digest framing and parsing
- serialization or decode boundaries
- proof or commitment field builders
- config accessors with fail-closed defaults

### Strong Integration Candidates

- storage plus wallet plus simulator contracts
- persisted artifact reloads
- signer/verifier roundtrips
- request/scan/spend parity
- state transitions with no partial mutation allowed

### Strong End-To-End Candidates

- realistic user or protocol journeys across crate boundaries
- publish/apply/finalize pipelines
- create/save/load/reopen flows
- build/prove/verify/accept flows
- checkpoint draft-to-final promotion gates

## When To Invoke Crypto And Security Lenses

### Always Invoke `brainstorming`

Use it to widen scenario discovery before convergence. Require at least one pass
across happy path, negative path, restart path, tamper path, and config path.

### Invoke `crypto-architect` When The Phase Includes

- proofs or verifiers
- commitments or roots
- signatures or transcript binding
- nullifiers or replay prevention
- stealth outputs, request tags, or ownership semantics
- KDF or seed-salt migration

Expected output from this lens:

- invariants that must hold
- proof paths that must succeed
- rejection paths that must fail
- exact fields that must be authenticated or bound

### Invoke `security-audit` When The Phase Includes

- trust boundaries
- secret-bearing artifacts
- malformed input handling
- permission or caller-controlled fields
- storage tamper surfaces
- unsafe defaults or compatibility fallbacks

Expected output from this lens:

- misuse cases
- secret leakage risks
- fail-closed expectations
- observable rejection signals

## Pass Oracle Rules

Every scenario in the spec should define:

- the behavior being proven
- the primary path under test
- the observable success condition
- the observable failure condition
- the exact assertion anchor

Good oracle examples:

- the same persisted checkpoint bytes decode to the same proof-bearing identity
- tampering one tuple field changes reject class from success to precise proof rejection
- a foreign scan result stays foreign across leaf, runtime, and stage-level flows
- an unknown KDF version rejects before decryption begins

Weak oracle examples:

- test passes without asserting state or output meaning
- no panic happened
- command returned zero but no contract state was checked

## What The Detailed Spec Must Contain

The spec is insufficient if it only lists scenario names. It should state:

- what is under test
- why the scenario matters
- exact inputs, fixtures, and preconditions
- expected outputs and persisted artifacts
- rejection signals for negative paths
- assertion anchors and command anchors
- exact repository destination for each test file

Use Mermaid diagrams when multi-stage flows or trust boundaries are easier to
understand visually than text.

Use short code snippets when fixture shape, proof tuple shape, or assertion form
would otherwise be ambiguous.

## What The Implementation-Order Artifact Must Contain

`NNN-TESTS-TASKS.md` should translate the spec into one execution order.

Minimum fields:

- phase metadata
- source spec reference
- execution strategy
- wave-by-wave order
- exact files to extend
- exact files to create
- per-wave completion gates
- per-wave verification commands
- sequencing notes that explain why one wave depends on another

## Existing Anchor Preference

Before proposing a new test file, search for:

- an existing focused integration test at the same seam
- inline tests already proving part of the behavior
- a simulator or storage test already carrying the correct fixture shape

Reuse first. Create a new file only when reuse would blur ownership or make the
test file too broad.

## Commit Rule

Only commit when the user wants the commit performed.

Required message:

`test(phase-{N}): add unit and E2E tests`
