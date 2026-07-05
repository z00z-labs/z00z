---
name: phase-exam-solve
description: Auto-invoked when user wants to solve an existing phase exam, fill Ans slots in EXAM-QUESTIONS-AND-ANSWERS.md, answer repository-backed verification questions sequentially, or produce verified exam answers with evidence. Also triggers on exam solve, solve exam, fill Ans, answer worksheet, repository-backed Q and A, doublecheck verification, partial proof, blocked proof, and summary table.
argument-hint: 'scope=<path[,path2,...]> report_path=<path.md> db_path=<path.jsonl> [exam_file=<path.md>] [max_variants=<N>] [question_index=<N>] [phase=<id>]'
---

# Phase Exam Solve

This skill is pure LLM. There is no Python execution path.

Solve a repository-backed exam question by question, or answer a direct
repository-backed verification question in no-file mode.

This skill does not generate a new exam. When an exam file exists, it reads the
existing `EXAM-QUESTIONS-AND-ANSWERS.md` file, answers the questions
sequentially, and writes each answer immediately into the matching `**Ans:**`
slot only after the answer survives a `doublecheck` verification pass.

When no exam file exists and the user asks a direct repository-backed question,
this skill answers in chat using the same proof discipline and verification
gate.

The solve must work like an evidence-driven auditor, not like a summarizer.
Every answer must be tied to the live repository state, to phase artifacts, and
to explicit logical, mathematical, cryptographic, and implementation evidence.

## Core Guarantees

- The solve path uses internal SSoT variation to generate multiple answer
  candidates per question.
- Every candidate answer is seeded independently.
- Weak candidates are rejected by a pro-cons analysis gate.
- The final answer is selected only from admitted candidates.
- Every selected answer is validated twice before file insertion:
  - validation pass A: skeptical evidence-consistency gate
  - validation pass B: contradiction and hallucination-risk gate
- Internal seeds and private drafts are never written into the final exam file.

## SSoT Random Seed And Model Contract (LLM-Only)

- Generate one seed per answer-candidate variant before solving each question.
- Seed format: lowercase alphanumeric string, fixed length `16`.
- Alphabet: `a-z0-9`.
- Default mode: non-deterministic seeds per run.
- Reproducible mode (only when explicitly requested): derive seed from stable inputs (`phase` or `exam_file`, `question_index`, `variant_index`, optional user salt).
- Never write raw seeds into exam answers, reports, or JSONL rows.
- Do not reuse a seed for the same question.
- Apply a strict 3-role SSoT model flow per variant:
  - `Generator`: drafts candidate answer.
  - `Critic`: attacks claim-evidence mismatches and contradiction risks.
  - `Selector`: admits only candidates that pass validation A, validation B, and doublecheck.

## When to Use

- Use when the user wants an existing phase exam solved instead of generated.
- Use when the target file already contains numbered `###` questions with blank
  `**Ans:**` sections.
- Use when the user wants answers written directly into
  `EXAM-QUESTIONS-AND-ANSWERS.md`.
- Use this skill only when an exam file already exists or the user explicitly
  wants file-backed `Ans:` insertion behavior.
- Use this skill when the user has a direct repository-backed question and
  wants a verified answer even if no exam file exists. In that no-file case,
  answer in chat using the same proof discipline instead of forcing question
  generation.
- Use when the user requires strict repository-backed proofs instead of loose
  commentary.
- Use when every answer must pass `doublecheck` before being accepted and
  written.
- Use when the user wants a final summary table classifying questions into full
  proof, partial proof, and blocked proof.

## Inputs

- Required: direct path to the exam file: `exam_file`, or a phase folder
  `phase_dir` from which the exam file can be resolved.
- Required alternative for no-file mode: a direct repository-backed question or
  a short ordered list of such questions.
- Optional: scope note such as crypto-only, storage-only, trust-boundary-only,
  or documentation-honesty-only.
- Optional: stop count for partial sessions, for example “solve the first 5
  questions only”.

If the phase exposes multiple exam files, prefer the canonical prefixed file,
for example `032-EXAM-QUESTIONS-AND-ANSWERS.md`.

For autonomous SSoT runner mode, use:

- required: `scope`, `report_path`, `db_path`
- optional: `exam_file`, `phase`, `question_index`, `root`, `max_variants`
- optional: `exam_file`, `phase`, `question_index`, `max_variants`

`report_path` stores run-level solve report, while `db_path` stores append-only
accepted candidate records.

## Copied Answering Rules

The following answering rules are intentionally copied forward from
`phase-exam-create` and remain authoritative for the solve:

1. Every final answer in this document MUST be independently re-checked through
   the `doublecheck` skill before it is accepted as final.
2. Every answer MUST be a repository-backed proof system, using factual,
   mathematical, cryptographic, and logical proof where applicable.
3. If a proof cannot be closed, the answer MUST state exactly what evidence,
   artifact, mathematical argument, cryptographic assumption, or repository
   behavior is missing.
4. Every answer MUST stay tied to the live codebase, tests, logs, manifests,
   and phase artifacts for this repository.
5. Every answer in this document MUST function as a verification exam of the
   correct implementation of this phase, not as freeform commentary.
6. If answering a question reveals a real bug, gap, or overclaim, the answer
   MUST name it explicitly and state the remediation path.
7. A valid answer must discover its own evidence path through code, tests,
   manifests, and closeout artifacts.
8. A valid answer must show why an optimistic but false reading would fail.
9. If the repository cannot close the proof, the answer must name the missing
   evidence precisely.

## Hard Editing Rules

- Do not rewrite question titles.
- Do not rewrite `**Quest:**` text.
- Do not rewrite the exam preamble, MUST section, themes, or question
  numbering.
- Do not reorder questions.
- Do not skip ahead to later questions while an earlier unanswered question is
  still open.
- Do not keep a full unsaved answer batch in memory.
- Write each verified answer immediately under its own `**Ans:**` marker.
- Treat the source constraint "only exact answer-slot insertions are allowed"
  as binding by default.
- Resolve the user-request conflict explicitly: if the same task also requires a
  final summary table, then the only permitted non-`Ans:` edit is the summary
  table section at the end of the file.
- No other non-`Ans:` edits are allowed under that carve-out.
- The only allowed file edits are:
  - filling content directly below existing `**Ans:**` markers
  - appending or refreshing the final summary table at the end of the document
    only when the user explicitly requested it and the conflict was recognized
    up front

## Core Standard

- Solve questions strictly in order.
- Start from the first unanswered question.
- If the file already contains answered questions, treat them as frozen unless
  the user explicitly asks for rework.
- For each question, gather evidence first, then reason, then verify via
  `doublecheck`, then write the answer.
- If `doublecheck` cannot be executed, fail closed: do not present the answer
  as accepted.
- A partial answer is allowed only when it explicitly names what proof is still
  missing and how that missing proof could be closed.
- A blocked answer is allowed only when a principled blocker exists and is
  stated precisely.
- Prefer repository evidence over planning prose when they disagree.
- Use examples, mini-derivations, short formulas, and Mermaid diagrams whenever
  they materially improve proof quality or clarity.
- Keep the writing readable for both a junior reader and an expert reviewer.
- In no-file direct-question mode, there is no file write; the answer is
  emitted in chat using the structured response form from `FORMS.md`.

## How It Works

1. Resolve the operating mode.
   - If an exam file path or phase-backed exam target exists, use file-backed
     solving mode.
   - If the input is a direct repository-backed question with no exam file,
     use no-file direct-question mode.

2. In file-backed solving mode, resolve the target exam file.
   - Accept a direct path when provided.
   - Otherwise resolve from the phase folder using the canonical prefixed exam
     filename when available.

3. In file-backed solving mode, read the exam file before any edit.
   - Detect the first unanswered `**Ans:**` slot.
   - Preserve all existing solved answers unless the user explicitly asks to
     revisit them.

4. Read the supporting evidence.
   Minimum set when present:
   - the phase exam file itself when it exists
   <!-- - `*-CONTEXT.md`
   - `*-TODO.md`
   - `*-TEST-SPEC.md`
   - `*-VERIFICATION.md` or `*-VALIDATION.md`
   - `*-SUMMARY.md`
   - `*-HONEST-CLOSEOUT.md` -->
   - live code, tests, manifests, fixtures, logs, and scripts relevant to the
     current question

5. Solve one question only.
   - In file-backed mode, answer the current question before touching the next
     one.
   - In no-file mode, answer one direct user question at a time.
   - Build the answer using the structure in `FORMS.md`.
   - Keep the answer anchored to proof, not to intuition.

6. Run `doublecheck` in one-shot mode on that draft answer.
   - Include the original question.
   - Include the exact claims made by the draft answer.
   - Include the evidence basis used.
   - If direct skill invocation is unavailable, use the `Doublecheck` subagent.
   - If the result is unresolved, disputed, or blocked, revise the answer so the
     unresolved status is explicit rather than hidden.

6.5. Apply double validation before acceptance.
   - Validation A: verify claim-to-evidence consistency and status correctness.
   - Validation B: run contradiction/hallucination-risk re-check over the same
     candidate.
   - Only candidates that pass both validations are eligible for final
     selection.

7. Write or emit the answer immediately.
   - In file-backed mode, insert the verified answer directly below the matching
     `**Ans:**` marker.
   - Do not hold later answers in memory waiting for a bulk write.
   - Save after each solved question.
   - In no-file mode, emit the verified answer in chat and then move to the next
     direct question only after the current one is closed.

8. In file-backed mode, update the final summary table.
   - Maintain one row per question.
   - Mark each row as `Full Evidence`, `Partial Evidence`, or `Blocked`.
   - For `Partial Evidence`, state what is missing and how to close the gap.
   - For `Blocked`, state the principled blocker.

9. Continue sequentially.
   - In file-backed mode, only move from question `N` to `N+1` after question
     `N` has both a written answer and a recorded verification state.
   - In no-file mode, only move to the next direct user question after the
     current answer is fully verified and emitted.

10. Perform a final audit.
    Verify that:
    - in file-backed mode, every answered question was solved in order
    - every answer includes an explicit verification result
    - in file-backed mode, no question text or preamble content was modified
    - in file-backed mode, the only allowed non-`Ans:` edit, when requested, is
      the final summary table section
    - in file-backed mode, the final summary table matches the per-question
      outcomes

## Proof Expectations

- Use factual proof for repository state, artifact content, and code behavior.
- Use logical proof for invariants, control-flow closure, fail-closed behavior,
  and contradiction handling.
- Use mathematical proof where a property depends on equations, commitments,
  nullifiers, counters, hashes, or bijective mappings.
- Use cryptographic proof where a property depends on binding, hiding,
  authenticity, replay safety, transcript integrity, or authority boundaries.
- Use implementation proof by connecting code, tests, manifests, and produced
  artifacts.

See `REFERENCE.md` for evidence priority, proof classification, and fail-closed
rules.

## Supporting Files

- `REFERENCE.md` — detailed evidence hierarchy, proof taxonomy, and
  doublecheck gate rules
- `FORMS.md` — answer template for insertion under `**Ans:**`, no-file direct
  answers, and final summary table template

## Autonomous SSoT Runner (LLM-Only)

Assistant-side flow:

1. resolve phase/exam target
2. gather repository evidence for active question
3. generate seeded answer candidates in-memory
4. apply pro-cons analysis and reject weak candidates
5. apply validation A and validation B
6. select strongest surviving candidate
7. write answer slot and update summary status
8. append accepted run record to JSONL

## Reference Execution Sequence (LLM-Only)

1. Parse inputs: `scope`, `exam_file`, `report_path`, `db_path`, `max_variants`, `question_index`.
2. Resolve operating mode (file-backed vs direct-question mode).
3. Load active question and repository evidence.
4. Generate SSoT answer variants in-memory.
5. Apply pro-cons selection and reject weak variants.
6. Run validation A and validation B.
7. Run one-shot doublecheck on selected candidate.
8. Insert verified answer or emit blocked/partial proof state.
9. Update summary table and append run metadata.

## Examples

### Example 1: Solve The Whole Exam

```text
/phase-exam-solve for .planning/phases/032-crypto-audit-scenario-1/032-EXAM-QUESTIONS-AND-ANSWERS.md

Assistant: reads the exam, starts from question 1, solves it with repository
evidence, runs doublecheck, writes the answer into that `Ans:` slot, updates
the summary table, and only then proceeds to question 2.
```

### Example 2: Resume From First Blank Answer

```text
/phase-exam-solve resume phase 032 exam

Assistant: reads the existing exam file, detects the first still-empty `Ans:`
slot, preserves previous answers, and continues from that exact question.
```

### Example 3: Stop After A Small Verified Slice

```text
/phase-exam-solve solve first 3 questions in the phase 032 exam

Assistant: answers questions 1 through 3 in order, verifies each one through
doublecheck before writing, updates the summary table, and stops without
touching question 4.
```

### Example 4: Partial Proof Handling

```text
/phase-exam-solve Question: the repository closes only the local invariant but not the network-wide guarantee?

Assistant: writes a `Partial Evidence` answer, names the exact missing
artifact or proof, explains how the gap could be closed, records the
doublecheck result, and then proceeds.
```

## Notes

- This skill solves the exam; it does not generate a new question bank.
- For no-file structured answers to direct repository-backed questions, use this
  skill directly rather than routing through `phase-exam-create`.
- The solve must preserve the exam as an audit artifact, not turn it into a
  generic report.
- If the repository evidence is too weak to close a proof, the correct action
  is to write a precise partial or blocked answer, not to improvise certainty.
