---
name: create-tests
description: Auto-invoked when user wants to create tests, add coverage, prove a completed phase works end to end, or write a phase-local test spec or test plan from planning files. Also triggers on create-spec, phase=, unit tests, E2E tests, integration coverage, TEST-SPEC, TEST-PLAN, TESTS-TASKS, RED-GREEN, SUMMARY.md, CONTEXT.md, TODO.md, VERIFICATION.md, and PLAN.md.
---

# Create Tests

Build a phase-local test contract first, then turn that contract into focused
unit, integration, and end-to-end tests without guessing scenario boundaries,
pass oracles, or failure paths.

## When to Use

- Use when a completed or near-complete GSD phase needs test coverage.
- Use when the user wants a phase-local `*-TEST-SPEC.md` document before test
  implementation.
- Use when the user wants a detailed test document that says what is tested,
  how it is tested, which inputs and expected outputs matter, which artifacts
  are created, and where each test file should live.
- Use when the user provides a phase folder plus optional instructions.
- Use when coverage must be derived from planning artifacts rather than from
  source code alone.
- Use when crypto, proof, storage, wallet, or security-sensitive flows need
  explicit invariant-driven tests.

## Inputs

- Required: phase folder path such as
  `.planning/phases/032-crypto-audit-scenario-1/`
- Required alternative: compact phase parameter such as
  `phase=032-crypto-audit-scenario-1` or `phase=29`
- Optional: extra focus instructions such as `focus on restart paths` or
  `prioritize negative proof cases`

If the user provides only a phase number, resolve it to the matching phase
folder before continuing.

A compact `phase=...` value is sufficient input for this skill. Do not require
the user to restate the full phase folder path when the phase identifier can be
resolved from the repository.

If the user invokes a compact spec-style prompt such as
`/create-spec phase=032-crypto-audit-scenario-1`, treat the phase parameter as
sufficient context to build the test contract from the phase artifacts.

The canonical skill name remains `create-tests`, but the public prompt shape
`/create-spec phase=...` is an accepted spec-first entry point for this skill.

## Working Rules

- Treat planning artifacts as specifications, not as decoration.
- Build the spec before writing tests.
- Build the execution plan before writing tests.
- Prefer existing test anchors over new files when the seam already exists.
- Apply RED-GREEN thinking even when the implementation already exists.
- For Rust-only phases, end-to-end proof usually means release-mode integration
  tests, not browser automation.
- When the phase touches cryptography, proofs, commitments, roots, signatures,
  wallet secrecy, or trust boundaries, explicitly load and apply
  `brainstorming` plus `crypto-architect` and/or `security-audit`.
- Include internal scenarios, edge cases, and fail-closed paths.
- Use Mermaid diagrams and short code snippets when they materially reduce
  ambiguity in stage flow, state transitions, trust boundaries, or fixture
  shape.
- Record the exact destination path for every planned test file.
- Unless the phase is obviously tiny or unusually large, default to a smart
  scenario set of roughly 10 to 20 tests or examples after brainstorming,
  then justify any narrower or broader scope.

## How It Works

1. Resolve the scope.
   Confirm the phase folder exists and capture any optional focus note.

2. Read the planning artifacts in priority order.
   Read, when present:
   - `*-VERIFICATION.md` or `*-VALIDATION.md`
  - `*-SUMMARY.md`
  - `*-CONTEXT.md` or `CONTEXT.md`
  - `*-TODO.md`, `TODO.md`, or `todo.md`
   - every `*-PLAN.md`

3. Choose the operating mode.
   - Use `verification-backed` mode when summary and verification artifacts
     exist.
   - Use `fallback-ready` mode when execution is incomplete but the planning
     artifacts are strong enough to define a truthful test contract.
   - Never pretend the phase is complete when completion artifacts are missing.

4. Inventory the executable seams.
   Classify files and behaviors into:
   - focused unit targets
   - integration targets
   - end-to-end targets
   - skip targets

5. Run scenario expansion deliberately.
   Use `brainstorming` to generate broad coverage candidates before narrowing.
   Rotate through these lenses:
   - happy path
   - negative path
   - restart and persistence
   - tamper and replay
   - configuration drift
   - cross-module contract mismatch

6. Deepen crypto and security scenarios when relevant.
   If the phase touches proofs, commitments, roots, signatures, nullifiers,
   stealth outputs, KDFs, secrets, or verifier boundaries:
   - apply `crypto-architect` to derive protocol invariants, proof paths,
     soundness risks, and rejection cases
   - apply `security-audit` to derive misuse paths, trust-boundary failures,
     secret-handling cases, malformed-input rejection, and fail-closed behavior

7. Write the phase-local test specification.
    Create or update `NNN-TEST-SPEC.md` in the phase folder using the template in
    `FORMS.md`.

8. Make the specification implementation-ready.
    The spec must explicitly define, when relevant:
  - which end-to-end behaviors must be proven
  - which integration paths are critical
  - which examples demonstrate successful execution
  - which negative scenarios prove rejection or failure handling
  - which cryptographic invariants, soundness properties, proofs,
    commitments, roots, or signatures must be observed
  - which assertions prove correctness
  - what each test is meant to demonstrate
  - what measurable pass or fail conditions close the scenario
  - which inputs, fixtures, and preconditions are required
  - which outputs and persisted artifacts must be observed
  - where every test file should live in the repository
  - which commands verify each scenario or wave
  - Mermaid diagrams for complex flows when they clarify boundaries better than prose
  - short code snippets when fixture shape or assertion shape would otherwise be guessed

9. Write the execution-planning artifact for GSD.
    In this repository, the canonical implementation-order artifact is
    `NNN-TESTS-TASKS.md`. Generate or update it from `NNN-TEST-SPEC.md` using the
    template in `FORMS.md`.

10. Support `NNN-TEST-PLAN.md` when explicitly requested.
      If the user or phase workflow asks for `NNN-TEST-PLAN.md`, create it as a
      synchronized compatibility artifact that mirrors or points to
      `NNN-TESTS-TASKS.md` instead of inventing a divergent third planning format.

11. Present a test plan for approval.
      Summarize:
  - generated planning artifacts
   - proposed new test files
   - existing anchors to extend
   - exact scenarios to implement
   - execution commands
   - known gaps or blockers

12. Generate tests only after approval.
    Follow project conventions for test location and naming. Prefer one focused
    integration file per seam instead of large omnibus files.

13. Verify with RED-GREEN discipline.
    - RED means the test would catch the intended regression or rejection case.
    - GREEN means the implementation satisfies the intended contract.
    - If the test finds a product bug, report it precisely instead of weakening
      the test.

14. Commit only when the user wants the test changes committed.
    Use this exact message:
    `test(phase-{N}): add unit and E2E tests`

## Outputs

- A phase-local `NNN-TEST-SPEC.md` planning artifact
- A phase-local `NNN-TESTS-TASKS.md` implementation-order planning artifact
- An optional `NNN-TEST-PLAN.md` compatibility handoff when explicitly requested
- A user-reviewable test plan
- Test files under the repository's existing test structure
- A concise gap report for anything not yet provable
- Optional commit using the required commit message

## Supporting Files

- `REFERENCE.md` — artifact precedence, coverage heuristics, crypto/security
  escalation rules, and acceptance criteria
- `FORMS.md` — templates for the phase-local test spec, tests-tasks artifact,
  optional compatibility test-plan artifact, and approval plan

## Examples

### Example 1: Minimal Phase Invocation

```text
/create-spec phase=032-crypto-audit-scenario-1

Assistant: resolves the phase folder, reads verification, summary, context, todo, and all plan artifacts, runs `brainstorming` first and then `crypto-architect` and or `security-audit` when the phase is crypto or security sensitive, and produces a test contract dense enough to drive roughly 10 to 20 realistic scenarios without guessing boundaries.
```

### Example 2: Minimal Numbered Phase Invocation

```text
/create-spec phase=029

Assistant: resolves Phase 029, derives the wallet test contract from the planning chain, expands it with `brainstorming` plus `crypto-architect` and or `security-audit` where needed, and writes the phase-local spec and tests-tasks artifacts so another engineer can implement end-to-end coverage directly.
```

### Example 3: Explicit Criteria Inline

```text
/create-spec phase=032-crypto-audit-scenario-1

define smart, complete, end-to-end integration tests and realistic examples that prove what the workflow does, how it behaves, which invariants it must preserve, and which failures it must reject.

Assistant: writes `032-TEST-SPEC.md` and `032-TESTS-TASKS.md` with explicit end-to-end behaviors, critical integration paths, successful examples, negative scenarios, cryptographic invariants, correctness assertions, measurable pass or fail conditions, exact test-file placement, Mermaid flow, and short clarifying snippets where the fixture or assertion shape would otherwise be guessed.
```

## Notes

- In this repository, browser-style E2E is often not the correct model.
  For Rust phases, end-to-end usually means realistic multi-crate integration
  and release-mode verification.
- If the planning artifacts contradict each other, stop and surface the drift
  before generating tests.
- Do not downgrade open requirements to closed simply because a local test
  passes.