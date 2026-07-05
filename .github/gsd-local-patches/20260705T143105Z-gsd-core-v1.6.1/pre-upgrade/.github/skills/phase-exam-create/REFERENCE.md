# Final Exam Create Reference

Use this file when the phase is complex, crypto-heavy, or documentation-heavy
enough that the question bank needs stronger structure than the main skill file
alone provides.

The question bank should behave like a phase-audit instrument. It should test
whether the phase closes the intended problem and scenario set, not whether the
repository happens to contain code that can be discussed.

## Artifact Priority

When generating the exam, weigh sources in this order:

1. Live code and tests
2. Verification or validation manifests and exact rerun logs
3. Honest closeout or current-status notes
4. Summary documents
5. Context and TODO planning artifacts

If a lower-priority artifact contradicts a higher-priority one, generate
questions that force the conflict into the open.

## SSoT Variant Axes

The autonomous runner rotates internal variant seeds across these axes:

- audit lens (closure, scenarios, trust boundary, replay, drift)
- adversarial interpretation angle
- evidence pressure mode (code, tests, manifests, closeout)
- guarantee pressure mode (crypto, security, fail-closed, continuity)
- wording style (forensic, classification, contradiction, residual-risk)

Variant seeds are internal and must not appear in the generated exam markdown.

## SSoT Seed And Model Contract (Canonical)

Treat this section as normative for all phase-exam-create runs.

- Generate one internal seed per variant before drafting questions.
- Seed format: lowercase alphanumeric, fixed length `16`, alphabet `a-z0-9`.
- Default mode: non-deterministic seeds per run.
- Reproducible mode (only when explicitly requested): derive deterministic seeds from stable inputs (`phase`, normalized `scope`, `variant_index`, optional user salt).
- Never expose raw seeds in exam markdown, run report, or persisted JSONL rows.
- Never reuse a seed inside one run.
- Run every variant through strict 3-role SSoT model flow:
  - `Generator`: drafts candidate question set.
  - `Critic`: pressure-tests leadingness, coverage gaps, and hint leakage.
  - `Selector`: admits only sets that pass quality and verification gates.

## Mandatory Coverage Lenses

The final 25-question set should cover the strongest applicable lenses for the
phase:

- phase intent versus delivered closure
- implementation truth versus planning intent
- scenario coverage versus scenario promise
- positive path and negative path behavior
- fail-closed boundaries
- trust-boundary honesty
- replay or stale-artifact risk
- semantic freeze or schema drift
- secret handling and debug-only escape hatches
- proof, signature, commitment, or verifier continuity
- state mutation safety
- documentation overclaim versus live code

Do not let the set collapse into one repeated family of questions.

## Phase Intent Extraction

Before drafting questions, extract at minimum:

- what the phase said it would fix or harden
- which user, system, or adversarial scenarios made that phase necessary
- which success conditions were implied by `TODO`, `CONTEXT`, `TEST-SPEC`, or
  validation artifacts
- which risks were explicitly left open
- which summary or closeout statements sound stronger than the underlying proof

If those promises are unclear, generate questions that force the ambiguity into
the open rather than silently normalizing it.

## Scenario Coverage Matrix

The generated question set should pressure-test the strongest applicable
scenario families for the target phase:

- core happy path
- reject or negative path
- malformed or edge-case input path
- replay, stale-artifact, or persistence continuity path
- migration, compatibility, or fallback path
- trust-boundary and actor-boundary path
- security-sensitive and cryptography-sensitive path tied specifically to the
  phase intent

Do not ask all scenario families mechanically. Use the ones the phase actually
puts at risk.

## Question Quality Gates

Every question should satisfy all of these checks:

- Answerable only by studying the repository
- Anchored to a concrete seam, claim, or artifact
- Capable of disproving an optimistic but false interpretation
- Useful even if the answer turns out to be partial or negative
- Specific enough that two reviewers would search for similar evidence
- Written at the level of the real problem, guarantee, gap, or overclaim
- Not dependent on embedded source breadcrumbs to be solvable
- Allowed to be tightly scoped to a named problem area, actor, boundary, or
  classification target without being treated as a hint by default
- Useful for deciding whether the phase implementation covers the scenario it
  promised to handle

Reject weak questions such as:

- generic theory prompts
- generic code-review prompts
- purely stylistic questions
- yes or no questions with no evidence burden
- duplicate questions that only rename the same seam
- questions that leak the answer path by naming the exact file, helper, or test
  that the solver should inspect
- questions that leak the answer path by naming requirement IDs, stage labels,
  or internal symbol names when the same challenge can be phrased at the level
  of the underlying problem

Do not reject a question merely because it is sharply thematic or forensic.
Naming the audited boundary, actor, guarantee, or classification problem is
acceptable when the wording still leaves the solver responsible for finding the
evidence trail and reaching the conclusion.

## Verification Gate

Admit a candidate question set only if all checks pass:

1. exact expected question count (default 25)
2. H3 numbering integrity from `1..N`
3. diversity across themes and scenario families
4. no answer-path breadcrumb leakage in `Quest:` lines
5. every question remains repository-evidence dependent
6. no duplicated or near-duplicated questions

If no candidate passes, emit a no-candidate report and do not append an
accepted record to the JSONL database.

## Strong Example Pattern

When the repository already contains a strong example exam, extract its
strengths instead of rewriting from a weaker generic template.

The benchmark qualities are:

- themed grouping that makes long audits readable
- adversarial framing that tests whether claims are actually true
- wording at the level of guarantees and problem understanding
- cross-artifact pressure on code, tests, manifests, and docs at once
- minimal hint leakage inside the question itself

For Phase 032, `.planning/032-EXAM-QUESTIONS-AND-ANSWERS-1.md` is a benchmark
for adversarial framing and grouping only. Do not copy its breadcrumb-heavy
wording when the stricter no-hint rule applies.

## Crypto And Security Escalation Triggers

Explicitly apply `crypto-architect` and `security-audit` question-design lenses
when the phase includes any of the following:

- signatures, proofs, commitments, roots, nullifiers, or transcript binding
- wallet secrets, ownership semantics, or receiver and sender authority
- batch verification, checkpoint acceptance, replay fences, or fail-closed
  state transitions
- claims about trustlessness, censorship resistance, data availability, or
  stronger cryptographic guarantees than the code can currently prove

When these triggers fire, the question bank should contain questions that can
distinguish:

- mathematically proven behavior versus implementation convention
- current-stack guarantees versus aspirational future architecture
- local self-consistency versus persisted or network-wide continuity

## Anti-Pattern: Not A Code Review

Reject question sets that mostly ask about:

- naming
- formatting
- refactor cleanliness
- local helper structure
- generic bug hunting detached from the phase contract

Those can matter only if the phase itself is an architecture-cleanup or
refactor-integrity phase. Otherwise the question bank must stay focused on
phase intent, scenario closure, relevant guarantees, and real gaps.

## Output Rules

- Keep the generated file in English.
- Prefer the canonical numeric phase prefix in the output filename whenever the
  phase folder name exposes one, such as `032-EXAM-QUESTIONS-AND-ANSWERS.md`
  for `032-crypto-audit-scenario-1`.
- Use readable themed sections. Prefer H2 for thematic groups and H3 for the
  numbered questions.
- Use H3 numbered headings for every question.
- Use exactly one `**Quest:**` line and one blank `**Ans:**` line block per
  question.
- Do not insert suggested answers, outlines, or hints under `Ans:`.
- Keep the MUST section near the top of the generated file.

## Recommended Document Preamble

The generated exam file should always make these points explicit:

- the exam is tied to the live repository state
- the answers must be proved, not merely asserted
- `doublecheck` is mandatory before any answer is accepted as final
- unresolved proof should result in a precise missing-evidence statement
- discovered bugs must be named explicitly together with remediation direction
- the solver must independently locate the evidence trail without hidden help
  from the question phrasing

## Regeneration Rules

If an exam file already exists:

- read it first
- if a prefixed renamed exam file already exists, treat it as canonical when
  the user points to it explicitly
- if the phase folder itself exposes a canonical numeric prefix, prefer that
  prefixed filename over an older unprefixed exam filename
- inspect prior brainstorming artifacts in the same phase folder and reuse only
  the strongest non-duplicative seams
- keep stronger existing wording when it is still correct
- replace outdated, weaker, or duplicated questions
- normalize numbering and headings after edits
- keep `Ans:` slots blank even when improving questions
- if an older file has stronger question phrasing than the current exam,
  preserve that phrasing style and only swap in fresher seams

## Database Semantics

The exam generation database is append-only JSONL and stores accepted variants
only.

Each accepted entry should include:

- stable signature
- created_at timestamp
- phase identity and scope paths
- candidate score and gate verdict
- selected question set payload

Rejected variants are allowed in stdout summary telemetry but must not be
written as accepted database records.
