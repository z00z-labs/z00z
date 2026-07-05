# Entrance Exam Solver Reference

Use this file when the exam is crypto-heavy, security-heavy, or large enough
that the solver needs stricter proof discipline than the main skill file alone
provides.

## Evidence Priority

When solving any question, weigh sources in this order:

1. Live code and tests
2. Verification or validation manifests and exact rerun logs
3. Honest closeout or current-status notes
4. Summary documents
5. Context and TODO planning artifacts

If a lower-priority artifact contradicts a higher-priority one, the answer must
surface that conflict instead of normalizing it away.

## Required Proof Lenses

Each answer should apply the strongest relevant lenses from this set:

- implementation truth versus planning intent
- positive path and negative path behavior
- fail-closed boundaries
- trust-boundary honesty
- replay or stale-artifact risk
- semantic freeze or schema drift
- secret handling and debug-only escape hatches
- proof, signature, commitment, or verifier continuity
- state mutation safety
- documentation overclaim versus live code

Do not allow an answer to collapse into a one-dimensional summary when multiple
proof lenses materially affect the question.

## Proof Status Classification

Use exactly one of these statuses per answer:

### Full Evidence

Use when the repository closes the claim with sufficient code, tests, artifacts,
and verification evidence.

Required properties:

- the core claim is closed
- the contrary reading is explicitly defeated
- the answer survives `doublecheck` without an unresolved contradiction

### Partial Evidence

Use when the repository closes only part of the claim.

Required properties:

- the answer names what is proven
- the answer names what is not proven
- the answer states exactly which evidence is missing
- the answer states how that gap could be closed

### Blocked

Use when the answer cannot be responsibly completed because of a principled
blocker.

Examples:

- the repository does not expose the needed artifact
- the proof depends on a missing implementation layer
- the phase artifacts contradict live code in a way that prevents closure
- the verification gate cannot be executed

Do not label a question `Blocked` just because it is difficult.

## Doublecheck Gate

Every answer must pass a verification gate before being written.

### Required Inputs To Doublecheck

Pass all of the following into the verification request:

1. the exact question text
2. the drafted answer text
3. the list of concrete evidence sources used
4. the intended proof status: `Full Evidence`, `Partial Evidence`, or `Blocked`
5. any explicit caveats, assumptions, or unresolved points

### Fail-Closed Rule

If `doublecheck` reports `DISPUTED`, `FABRICATION RISK`, or an equivalent hard
verification problem, do not write the old answer. Rewrite it so the disputed
or missing part is explicit, or mark the question blocked.

## SSoT Candidate Search

For each question, produce multiple seeded candidate answers from repository
evidence before selecting a final answer.

Required candidate pipeline:

1. extract evidence snippets from code, tests, manifests, and phase artifacts
2. generate `N` seeded candidate answers
3. run pro-cons analysis per candidate
4. discard weak or speculative candidates
5. run two validation passes for each surviving candidate

Selection rule:

- choose only from candidates that pass both validations
- prefer stronger contradiction handling and clearer evidence trace
- if no candidate survives, mark question `Blocked` with explicit reason

## SSoT Seed And Model Contract (Canonical)

Treat this section as normative for all phase-exam-solve runs.

- Generate one internal seed per answer-candidate variant before solving each question.
- Seed format: lowercase alphanumeric, fixed length `16`, alphabet `a-z0-9`.
- Default mode: non-deterministic seeds per run.
- Reproducible mode (only when explicitly requested): derive deterministic seeds from stable inputs (`phase` or `exam_file`, `question_index`, `variant_index`, optional user salt).
- Never expose raw seeds in answers, reports, or persisted rows.
- Never reuse a seed for the same question.
- Run every variant through strict 3-role SSoT model flow:
  - `Generator`: drafts answer candidate.
  - `Critic`: attacks claim-evidence mismatches and contradiction risks.
  - `Selector`: admits only candidates that pass validation A, validation B, and doublecheck.

## Two-Pass Validation Rule

Every selected candidate must pass both checks:

- Validation A (evidence consistency): every strong claim must map to concrete
  repository evidence in the answer trail.
- Validation B (skeptical contradiction gate): answer must not overstate closure
  when contradictory evidence exists.

If either validation fails, the candidate is rejected and cannot be written.

### Minimum Verification Record

Each written answer should record:

- that `doublecheck` was run
- the resulting verification classification
- any residual caveat that survived verification

## Sequential Write Discipline

The solver must behave like a transactional workflow.

For each question:

1. read question
2. gather evidence
3. draft answer
4. verify through `doublecheck`
5. write answer into the matching `Ans:` slot
6. refresh the summary table
7. move to the next question

Never draft a full 25-question answer set before writing. The artifact must be
updated incrementally.

## Conflict Resolution Rule

If the source task simultaneously says:

- only exact `Ans:` insertions are allowed, and
- a final summary table must exist,

then the solver must fail closed unless it records one explicit carve-out:

- all question text and other pre-existing sections remain immutable
- the summary table at end of file is the one and only permitted non-`Ans:`
  edit

Any broader rewrite remains forbidden.

## Allowed Richness Inside Ans Slots

Answers should be detailed, but the detail must earn its place.

Recommended additions when they improve proof quality:

- short examples
- mini-derivations
- threat-boundary tables
- contrast between optimistic reading and actual enforced reading
- Mermaid diagrams for state flow, authority binding, artifact flow, or replay
  boundaries

Avoid decorative diagrams that do not sharpen the proof.

## Junior And Expert Readability

Each answer should work on two levels:

- a plain-language explanation that a junior can follow
- a precise proof path that an expert can audit

The easiest way to achieve this is to start with a short conclusion, then show
the evidence trail and rigorous reasoning.

## Summary Table Rules

The exam file must end with a summary table that covers every question.

Columns should capture:

- question number
- short title
- proof status
- verification status
- missing evidence or blocker
- how to close the gap when applicable

If a question is fully proven, the gap column should say `None`.

## Common Failure Modes

Reject answers that do any of the following:

- summarize phase docs without checking live code
- hide unresolved contradictions inside confident prose
- claim cryptographic closure without showing the actual binding surface
- call a question complete even though `doublecheck` still disagrees
- modify question wording or other non-answer parts of the file
- skip unanswered earlier questions and solve later ones first

## Recommended Closing Audit

Before finishing the solving session, re-check that:

- every solved question has a non-empty `Ans:` body
- every `Ans:` body includes a verification line
- no question text changed
- the summary table matches the answers actually written
- partial and blocked answers explain the exact gap or blocker
