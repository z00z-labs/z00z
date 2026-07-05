---
name: phase-exam-create
description: Auto-invoked when user wants to create an phase exam, exam sheet, implementation-phase question bank, repo-backed verification worksheet, or phase-audit questions that test whether implementation matches the phase intent, scenario coverage, and relevant security or cryptography expectations. Also triggers on EXAM-QUESTIONS-AND-ANSWERS, phase exam, audit questionnaire, implementation interview, verification questions, crypto review questions, security review questions, scenario coverage, and phase gap detection.
argument-hint: 'scope=<phase_dir[,phase_dir2,...]> report_path=<path.md> db_path=<path.jsonl> [max_variants=<N>] [question_count=<N>] [phase=<id>]'
---

# Phase Exam Create

This skill is pure LLM. There is no Python execution path.

Generate a repository-backed exam document for one implementation phase.

This skill creates the question sheet only. It does not answer the questions,
does not solve them in chat, and does not own answer-verification flow.
Answering belongs to `phase-exam-solve`.

The purpose of this skill is to create a phase-audit question bank that tests
whether the implemented phase actually closes the intended task, covers the
important functional scenarios, and honestly handles the relevant security or
cryptographic guarantees and residual gaps.

## Core Guarantees

- The exam generation uses internal SSoT "String Seed of Thought" variation to
  create multiple candidate question sets.
- Each candidate set is generated with an independent internal variant seed.
- Weak question sets are rejected by a skeptical quality gate.
- Only one strongest accepted question set is written to the final report.
- Internal seed strings and private drafting artifacts are never exposed in the
  final exam markdown.
- The database path is append-only and records only accepted variants.

## SSoT Random Seed And Model Contract (LLM-Only)

- Generate one seed per candidate set before drafting questions.
- Seed format: lowercase alphanumeric string, fixed length `16`.
- Alphabet: `a-z0-9`.
- Default mode: non-deterministic seeds per run.
- Reproducible mode (only when explicitly requested): derive seed from stable inputs (`phase`, normalized `scope`, `variant_index`, optional user salt).
- Never write raw seeds into exam markdown, run report, or JSONL rows.
- Do not reuse a seed inside one run.
- Apply a strict 3-role SSoT model flow per variant:
  - `Generator`: drafts candidate question set.
  - `Critic`: pressure-tests leadingness, coverage gaps, and hint leakage.
  - `Selector`: admits only sets that pass quality and verification gates.

## When to Use

- Use when the user wants an exam, interview sheet, or verification worksheet
  for a specific implementation phase.
- Use when the output must live under `.planning/phases/<phase>/` and be tied
  to the real repository state.
- Use when the user wants a reusable `EXAM-QUESTIONS-AND-ANSWERS.md` style file
  with numbered H3 questions plus blank answer slots.
- Use when the phase is crypto-sensitive, trust-boundary-sensitive, or
  security-sensitive and the question set must pressure-test only the relevant
  guarantees for that phase.
- Use when the user wants questions phrased at the level of phase intent,
  scenario coverage, guarantees, gaps, and adversarial interpretation rather
  than “read this file and report what line X does.”
- Use when the goal is to detect gaps between `TODO`, `CONTEXT`, validation
  artifacts, and live implementation.
- Do not use this skill as a code-review generator or as a direct answerer.

## Inputs

- Required: phase folder path such as `phase_dir`
  `phase_dir = .planning/phases/032-crypto-audit-scenario-1/`
- Required alternative: compact phase identifier such as `phase`
  `phase = 032-crypto-audit-scenario-1` or `phase = 032`
- Optional: custom output prefix such as `032`, `scenario1`, or `audit`
- Optional: extra focus note such as `focus on checkpoint and secrets`

If the phase folder follows the common repository shape `NNN-phase-name`,
derive the canonical default prefix from the leading numeric segment. Example:

- `032-crypto-audit-scenario-1` -> canonical prefix `032`

If a prefix is explicitly provided, write:
`.planning/phases/<phase>/<prefix>-EXAM-QUESTIONS-AND-ANSWERS.md`

If no prefix is explicitly provided but the phase folder starts with `NNN-`,
write:
`.planning/phases/<phase>/NNN-EXAM-QUESTIONS-AND-ANSWERS.md`

Only fall back to the unprefixed form
`.planning/phases/<phase>/EXAM-QUESTIONS-AND-ANSWERS.md`
when the phase folder does not expose a canonical numeric prefix and the user
did not provide one.

If the user says the exam file was already renamed to a prefixed form, treat
that exact prefixed file as the canonical target and read it before generating
or rewriting anything.

If the user asks for direct answers instead of a question bank, route that work
to `phase-exam-solve` rather than extending this skill into answer flow.

## Autonomous SSoT Runner (LLM-Only)

Use assistant-side autonomous execution as the default path.

The assistant performs:

1. phase resolution
2. artifact and repository seam extraction
3. seeded candidate generation
4. skeptical pro-con quality audit
5. verification gate
6. strongest-candidate selection
7. report and append-only JSONL write

## Core Standard

- The generated exam must resemble the strongest repository examples of an
  phase exam for adversarial framing, thematic grouping, and problem-level
  pressure. Do not inherit breadcrumb-heavy wording from a benchmark example.
- Questions must be phrased as independent audit prompts. The solver should be
  forced to discover the evidence trail by studying the repository, not by
  following source-file breadcrumbs embedded in the question.
- Questions must operate at the level of guarantees, scenario closure, trust
  boundaries, semantic drift, replay safety, overclaim, and proof obligations.
- Thematic focus is allowed and preferred. Naming the audited guarantee,
  boundary, actor, artifact family, or disputed behavior is not a hint by
  itself.
- Questions may target concrete implementation seams, but the question text
  itself should not spoon-feed file paths, function names, or exact helper
  names unless the user explicitly asked for a code-guided worksheet.
- Treat file paths, helper names, test names, requirement IDs, stage labels,
  and internal symbol names as banned hint material in question text unless the
  user explicitly asked for a code-guided worksheet.
- Do not treat a question as too leading merely because it is tightly scoped,
  forensic, or classification-oriented. It becomes a hint only when it embeds
  the evidence route or preloads the verdict.
- Stronger questions ask “what is actually true, what remains open, and what
  evidence closes the claim?” rather than “what does function X return?”

## Question Design Focus

- Derive the question set from the phase's intended closure, not from whatever
  code happens to exist in the directory.
- Extract the central promises, tensions, and open risks from the phase
  artifacts before drafting any question.
- Ask whether the phase closes the hard scenarios it set out to solve.
- Ask whether the implementation truth matches the planning and closeout story.
- Ask whether the relevant security or cryptographic guarantees for this phase
  are genuinely closed, only partially closed, or still overstated.
- Ask questions that help reveal missing functional coverage, untested negative
  paths, boundary confusion, replay risk, persistence gaps, migration gaps,
  artifact-discipline drift, or documentation overclaim when those concerns are
  part of the phase.

## Anti-Pattern: This Is Not Code Review

- Do not turn the question bank into a generic code-review checklist.
- Do not ask style, naming, formatting, refactor, or local code-smell questions
  unless the phase itself is explicitly about architecture cleanup or refactor
  integrity.
- Do not ask locator questions such as “what does helper X do” or “which file
  implements Y.”
- Do not ask general security theory questions disconnected from the target
  phase.
- Do not ask broad crypto questions unless the phase actually claims a crypto,
  verifier, signature, commitment, root, nullifier, secrecy, or trust-boundary
  property.
- The correct target is phase-understanding and implementation-coverage audit,
  not repository style review.

## Working Rules

- Questions are the filled part. `Ans:` blocks must stay blank in the generated
  file.
- Default to **25** questions unless the user explicitly asks for a different
  count.
- Every question must require repository evidence. Avoid generic textbook or
  opinion questions.
- Every question must be strong enough to validate implementation truth, not
  only design intent.
- Every question must be answerable only by deep repository study, but the
  question itself should not leak the answer path through built-in hints.
- A question may still be precise about the theme under audit. Precision about
  the problem is allowed; precision about the source trail is not.
- Every question heading must be H3 and use ordered numbering:
  `### 1. Title`
- Every question block must use this exact shape:

  ```text
  ### 1. Title
  🔴 **Quest:** ...
  🔵 **Ans:**
  ```

- Keep `Ans:` empty. Do not prefill hints, outlines, bullets, or placeholder
  answers inside the answer slot.
- Use bold labels exactly as shown: `🔴 **Quest:**` and `🔵 **Ans:**`.
- Preserve the template MUST section. Do not expand this skill into an answer
  procedure; the generated sheet may constrain later answers, but this skill is
  only responsible for writing the questions.
- Prefer diversity over repetition: mix architecture, invariants, security,
  honest-closeout language, edge cases, replay, trust boundaries, negative
  paths, documentation drift, and test-evidence questions.
- Group the question bank into readable thematic sections. The default target
  shape is five themed sections with five questions each.
- Themed sections should improve navigation, not weaken difficulty. Do not turn
  sections into answer hints.
- Avoid formulaic prompts like “Using file X, prove Y.” Prefer “What closes the
  claim that Y is true, and where does the repository still refuse to close
  it?”
- Avoid verdict-preloading phrases like “reopened gap”, “still open”, “missing
  semantics”, “no longer passes”, or any wording that tells the solver in
  advance what conclusion they are supposed to reach.
- Do not over-correct into vague questions. If the phase is about claim trust,
  spend boundaries, checkpoint acceptance, artifact hygiene, or semantic freeze,
  the question should be allowed to say so explicitly as long as it does not
  point to the exact file, helper, test, requirement ID, or predetermined
  answer.

## How It Works

1. Resolve the target phase.
   - Confirm the phase folder exists.
   - Resolve compact phase identifiers to the real directory.
   - Resolve the output filename using the explicit prefix when provided.
   - Otherwise derive the canonical numeric prefix from the phase folder name
     when available.

2. Read the phase artifacts and live repository evidence before drafting
   anything.
   Minimum set when present:
   - `*-CONTEXT.md`
   - `*-TODO.md`
   - `*-TEST-SPEC.md`
   - `*-VERIFICATION.md` or `*-VALIDATION.md`
   - `*-SUMMARY.md`
   - `*-HONEST-CLOSEOUT.md`
   - requirement or state files outside the phase folder when the phase points
     to them
   - prior question-bank artifacts such as
     `BRAINSTORMING-VERIFICATION-QUESTIONS.md` or existing exam files when they
     already live in the same phase folder

2.5. Prefer autonomous LLM execution.
  - Run seeded SSoT generation in-memory.
  - Keep the same gates and acceptance thresholds.

3. Extract the phase audit target.
   At minimum identify:
   - what the phase intended to close
   - which functional scenarios and boundary cases the phase promised to cover
   - which guarantees are in scope for this phase and which are not
   - which residual gaps are explicitly acknowledged
   - which documents risk overstating closure

4. Build a scenario coverage map.
   The map should capture the strongest applicable families for the phase, such
   as:
   - happy-path execution
   - negative and reject paths
   - edge conditions and malformed inputs
   - replay, stale-artifact, or persistence continuity scenarios
   - migration, compatibility, or fallback scenarios
   - trust-boundary and actor-boundary scenarios
   - security-sensitive and cryptography-sensitive scenarios tied to this phase

5. Inspect live repository evidence.
   Read the code, tests, fixtures, manifests, and logs that implement or verify
   the phase claims. Do not build the output from planning files alone.

6. Study any strong pre-existing exam artifact.
   - If the repository already contains a stronger question bank, mine its
     structure and strengths before generating a replacement.
   - Extract what makes it strong: adversarial framing, problem-level wording,
     low hint leakage, thematic grouping, and pressure on overclaim.
   - Preserve those strengths while updating stale or weaker questions.
   - Do not preserve benchmark wording that leaks the answer route through
     direct artifact names, helpers, tests, requirement IDs, or stage labels.

7. Escalate question quality when the scope is sensitive.
   When the phase touches cryptography, proofs, signatures, commitments,
   nullifiers, roots, secret material, ownership semantics, wallet security,
   checkpoint semantics, or trust-boundary honesty, explicitly apply:
   - `crypto-architect` for protocol, cryptographic, and verifier-boundary
     reasoning
   - `security-audit` for misuse, fail-closed, secret-handling, replay, and
     exploit-surface reasoning

8. Produce the question set.
   - default to 25 questions
   - make the set intentionally diverse
   - ensure the combined set pressure-tests the phase's intended closure,
     disputed seams, and scenario coverage
   - include questions that can expose missing functional coverage,
     implementation bugs, or documentation drift
   - phrase questions so the solver must do the repository archaeology alone
     and do not embed the evidence route in the `Quest:` line

9. Create the exam document from the bundled template.
   Use `templates/Q-A-TEMPLATE.md` as the starting form.
   - Fill the phase metadata.
   - Keep the MUST instructions intact.
   - Replace placeholder titles and `Quest:` lines.
   - Keep every `Ans:` blank.
   - Keep the readability scaffolding intact: challenge, constraints, answer
     standard, and thematic grouping.

10. Respect existing files carefully.
    - If the target exam file already exists, read it first.
    - Prefer the numeric-prefixed exam file when the phase name exposes a
      canonical numeric prefix.
    - If the phase already contains brainstorming or prior exam artifacts, mine
      them for stronger seams and remove duplicates rather than ignoring them.
    - Preserve stronger repository-backed wording.
    - Regenerate only when the existing file is weaker, stale, inconsistent, or
      the user asked for a rewrite.

11. Perform a final audit.
    Verify that:
    - all 25 questions are answerable only from repository study
    - the set covers both intended closure and still-open phase boundaries
    - the set meaningfully probes functional scenarios, negative paths, and the
      relevant security or crypto seams for this phase
    - question headings are H3 and correctly numbered
    - all `Ans:` sections are fully blank, including the final question
    - the MUST section is present and explicit
    - `Quest:` and `Ans:` use bold labels exactly
    - the document is grouped into readable thematic sections
    - the question text does not leak answer breadcrumbs through gratuitous
      file, helper, module, test, requirement, stage, or internal symbol naming
    - the set reads like a phase-audit of implementation truth and coverage,
      not like a generic code review

## Outputs

- a phase-local exam document named either:
  - `<canonical-prefix>-EXAM-QUESTIONS-AND-ANSWERS.md`, or
  - `EXAM-QUESTIONS-AND-ANSWERS.md` only when no canonical numeric prefix exists
- a document that contains filled questions and blank answer slots
- a document-level MUST section preserved from the template for later solving
- a document whose questions are phrased as independent, adversarial,
  problem-level verification prompts focused on phase intent, coverage, and
  unresolved gaps
- append-only JSONL record for accepted variant metadata and selected questions

## Supporting Files

- `REFERENCE.md` — coverage lenses, artifact priority, and question quality
  gates
- `FORMS.md` — output card, accepted JSONL shape, and no-candidate stub
- `templates/Q-A-TEMPLATE.md` — reusable markdown form for the generated exam

## Reference Execution Sequence (LLM-Only)

1. Parse inputs: `scope`, `phase`, `report_path`, `db_path`, `max_variants`, `question_count`.
2. Resolve target phase and exam output filename policy.
3. Read phase artifacts and relevant repository evidence.
4. Generate internal SSoT variants and draft candidate question sets.
5. Apply skeptical quality gate and reject weak sets.
6. Select one strongest accepted question set.
7. Render exam document with template and strict slot rules.
8. Write report and append accepted variant metadata to JSONL.

## Examples

### Example 1: Basic Phase Exam

```text
/phase-exam-create for phase=032-crypto-audit-scenario-1

Assistant: reads the phase artifacts and live code, designs 25 repository-only
questions, and writes
.planning/phases/032-crypto-audit-scenario-1/032-EXAM-QUESTIONS-AND-ANSWERS.md
with themed sections, filled `Quest:` lines, and blank `Ans:` slots.
```

### Example 2: Prefixed Exam File

```text
/phase-exam-create for phase 032 with prefix=security

Assistant: writes
.planning/phases/032-crypto-audit-scenario-1/security-EXAM-QUESTIONS-AND-ANSWERS.md
using the same 25-question exam format.
```

### Example 3: Crypto-Sensitive Phase

```text
/phase-exam-create for the crypto audit scenario and make the questions
strong enough to detect overclaim and hidden trust assumptions

Assistant: reads the phase artifacts, inspects the code and tests, explicitly
applies `crypto-architect` and `security-audit`, and writes a 25-question exam
that targets proof boundaries, ownership semantics, replay closure, secret
hygiene, and honest closeout language while keeping every `Ans:` block blank.
```

### Example 4: Non-Leading Audit Questions

```text
Weak question:
🔴 **Quest:** Using claim/v2.rs, prove whether source_root is signed.

Strong question:
🔴 **Quest:** What exactly is the authority signature allowed to authenticate in the
current claim path, and what mutation would demonstrate that the repository is
still trusting something important outside the signed statement?
```

## Notes

- This skill generates the question sheet only. It does not fill answers.
- Direct repository-backed answers belong to `phase-exam-solve`.
- If the repository evidence is too weak to support a clean exam, say so and
  record the missing evidence in the generated document scope notes.
- If the phase artifacts and live code materially disagree, surface that drift
  through questions instead of silently normalizing it.
