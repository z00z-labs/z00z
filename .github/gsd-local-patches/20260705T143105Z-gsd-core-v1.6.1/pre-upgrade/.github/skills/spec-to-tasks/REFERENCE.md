# Spec To Tasks Reference

## Output Naming Rules

- If the source or requested output carries a stable numeric prefix, emit
  `NNN-TODO.md` and use `NNN-01`, `NNN-02`, and so on.
- If no stable prefix exists, emit `TODO.md` and use `TODO-01`, `TODO-02`, and
  so on.
- If the user gives a custom prefix, keep it consistent across the title,
  dependency chain, and task identifiers.

## Modern TODO Sections

The modern repository-style backlog should include these sections in order:

1. `# Title`
2. `Canonical design source`
3. `Execution rules`
4. `Decision Summary`
5. `Dependency Chain`
6. `File-First Implementation Order`
7. `Validation Matrix`
8. `Explicit Phase Boundary`
9. `Concrete Execution Tasks`
10. a dedicated validation-wave section when test waves are a real second-half
  lane, using local truthful wording such as `Concrete Test Execution Tasks`
  or `Mandatory Test Waves`
11. an explicit phase-level closure section such as `Completion Gate` only when
  the source phase actually uses one or would otherwise hide closure truth

Do not silently drop `Validation Matrix` or `Explicit Phase Boundary` in the
modern format. Keep closure truthful: if the source does not carry a named
closing section, rely on execution rules and task exit conditions instead of
inventing one.

## Source Normalization Heuristics

When the source is already a structured spec, preserve its language and just
normalize it into backlog sections.

When the source is a free-doc, normalize it into these buckets first:

- authority or source-of-truth statements
- scope boundaries
- architectural constraints
- explicit decisions
- repeated design preferences that imply decisions
- lifecycle or state-transition rules
- trust boundaries and fail-closed expectations
- stated or implied non-goals
- file or subsystem seams
- verification or test obligations

Never present an inferred bucket as explicit source truth unless the document
actually supports it.

## Decision Summary Extraction Rules

Use the Decision Summary to freeze execution intent before tasks start.

Good decision-summary items:

- choose one live authority surface over multiple overlapping seams
- prefer extending an existing boundary over creating a new facade
- define where producer or verifier logic must live
- explicitly postpone adjacent work that would destabilize the current phase

Weak decision-summary items:

- generic statements like `improve quality`
- work-step restatements that belong in the tasks, not in the baseline

Keep the section numbered and concise.

## Dependency Chain Rules

The dependency chain should express semantic order, not just file order.

Build it like this:

1. foundational seam or data-contract tasks
2. producer or construction tasks
3. verifier or consumer tasks
4. persistence or integration tasks
5. closure and regression tasks
6. test-wave tasks when present

Add `Hard dependencies` only when the relation is real and must not be left
implicit.

## File-First Implementation Order Rules

This section turns semantic tasks into an editor-friendly order.

- list real repository files first
- use `new <path>` only when a new file is genuinely justified
- cluster files by seam so another engineer can execute the backlog without
  reconstructing edit order
- keep test file groups near the end unless the phase is test-first by design

Do not use this section to smuggle in speculative modules.

## Validation Matrix Rules

The validation matrix proves that the TODO came from the source.

Recommended columns:

- source section or lines
- required theme
- TODO coverage
- status

Status language should stay concrete, for example:

- `Validated mapped`
- `Mapped as execution guardrail`
- `Mapped as preservation constraint`
- `Mapped with explicit deferrals`

The matrix is insufficient if it only repeats section names without explaining
the implementation meaning.

## Explicit Phase Boundary Rules

This section exists to stop overclaiming.

Add it whenever the source:

- rejects stale drafts or retired designs
- explicitly forbids future-only concepts
- leaves adjacent work intentionally out of scope
- would tempt an implementer to add a second layer or overbuilt solution

State the non-goal explicitly and explain what the current backlog does instead.

## Task Construction Rules

Each concrete execution task should include:

- task heading with stable identifier
- spec references
- `MANDATORY pre-read` block
- checklist of implementation steps
- `Files:` block
- `Tests:` block
- `Exit condition:` block

Each task should prove one seam or one tightly related cluster of seams.
Prefer focused task boundaries over vague umbrella tasks.

## Pre-Read Referencing Rules

Use file line ranges only when all of the following are true:

- the source file exists locally
- the line numbers are stable enough to be useful
- the task genuinely depends on those exact ranges

If any of those conditions fail, cite section headings instead.

Never fabricate numeric line ranges for pasted chat content.

## Test-Wave Rules

Add a dedicated validation-wave section when:

- the phase already has or clearly needs test-wave sequencing
- the source distinguishes implementation order from validation order
- multiple task groups must land before regression closure is meaningful

Use the heading style that matches the source or the closest truthful exemplar,
for example `Concrete Test Execution Tasks` or `Mandatory Test Waves`.

Keep tests embedded in each main task only when a second-half wave structure
would be artificial.

## Phase-Level Closure Rules

Use an explicit closing section such as `Completion Gate` only when the source
phase already uses one or when a named phase-level closeout is needed to avoid
hiding required closure criteria.

When such a section is present, it should usually cover:

- all numbered tasks completed or explicitly deferred by source update
- tests green for the required seams
- source and TODO still aligned
- no forbidden parallel layer introduced
- no stale or future-only concept smuggled in as implemented truth

If no explicit closing section is present, make the same closure truth visible
through execution rules plus task-local exit conditions.

## Ambiguity Stop Conditions

Stop and report ambiguity when the source cannot support truthful answers to any
of these questions:

- What is the canonical source of truth?
- What is in scope versus explicitly out of scope?
- Which seams are already live and must be preserved?
- What order do the tasks depend on?
- Which files or subsystems are actually involved?
- What tests or completion criteria would prove closure?

If two documents disagree on current truth, surface the contradiction before
generating the final backlog.

## Quality Checklist

Before finalizing a generated TODO artifact, verify:

- section order matches the modern repository format
- identifiers are stable and consistent
- decision summary is architectural, not generic
- dependency chain and file-first order do not contradict each other
- validation matrix maps real source themes to real TODO coverage
- explicit non-goals are named where needed
- every task has files, tests, and an exit condition
- any explicit closing section is concrete and not ceremonial
