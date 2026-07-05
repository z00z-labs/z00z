---
name: spec-to-tasks
description: Auto-invoked when user wants to turn a spec, design doc, free-form requirements note, whitepaper section, or phase document into a structured TODO backlog like this repository's 040-TODO.md or 050-TODO.md. Also triggers on spec-to-tasks, spec to TODO, convert spec to backlog, execution backlog, task breakdown, dependency chain, file-first order, validation matrix, explicit phase boundary, and phase TODO generation.
---

# Spec To Tasks

Convert a source-of-truth document into one execution-ordered `TODO.md` or
`NNN-TODO.md` artifact that is concrete enough for implementation, review, and
verification without inventing a second architecture.

## 🎯 When to Use

- Use when the user wants to convert a specification into an implementation
  backlog.
- Use when the user has a free-form design note, whitepaper section, roadmap
  slice, ADR, or requirements document and wants a repository-style `TODO.md`.
- Use when the output must look like the modern phase backlogs already used in
  this repository, especially `.planning/phases/040-spend-proof/040-TODO.md`
  and `.planning/phases/050-offline-tx/050-TODO.md`, while matching their
  shared denominator instead of inventing stricter universal sections.
- Use when execution order, hard dependencies, file clusters, tests, and
  completion criteria must be explicit before code changes begin.
- Use when the user wants one normalized backlog format instead of an ad hoc
  bullet list.

## 📥 Inputs

- Required: one primary source document, pasted free-doc, or spec path.
- Optional: supporting source files that refine the same behavior.
- Optional: desired output path.
- Optional: numeric or textual task prefix.
- Optional: focus note such as `prioritize storage seams` or
  `keep test waves in the same file`.

If the source already carries a stable phase or numeric prefix such as `045` or
`050`, preserve it and emit `NNN-TODO.md`.

If the source has no stable prefix, emit `TODO.md` and use `TODO-01`, `TODO-02`
style task identifiers unless the user provides a different explicit prefix.

## ⚙️ Working Rules

- Treat the source document as normative for requirement meaning.
- Treat the generated TODO artifact as normative for execution order.
- Preserve one live architecture path; do not invent a parallel layer, facade,
  verifier, pipeline, or persistence path unless the source explicitly demands
  it.
- Reuse existing repository seams and truthful file clusters before proposing
  new modules.
- Separate explicit source claims from inferred normalization. If a claim is
  ambiguous, label it as ambiguous instead of flattening it into a fake task.
- If the source corpus contradicts itself, stop and surface the drift before
  generating the final backlog.
- If execution would need a new design constraint, update the source document
  first, then regenerate or update the TODO artifact.
- Keep repository artifacts in English.
- Prefer the shared modern backlog shape visible across Phase 040, Phase 045,
  and Phase 050 over older mandatory-task-only formats.

## 🧱 Output Contract

The default output should follow the modern section order captured in
`FORMS.md` and grounded in the shared denominator of Phase 045 and Phase 050:

1. Title
2. Canonical design source
3. Execution rules
4. Decision Summary
5. Dependency Chain
6. File-First Implementation Order
7. Validation Matrix
8. Explicit Phase Boundary
9. Concrete Execution Tasks
10. Optional dedicated validation-wave section using truthful local wording
11. Optional explicit phase-level closure section such as `Completion Gate`

The generated backlog is not finished until it has:

- explicit task identifiers
- task-local exit conditions
- files and tests for each concrete task
- truthful phase-level closure criteria, whether they live in task exit
  conditions or in an explicit closing section
- traceability from source sections into TODO coverage

## 🧭 How It Works

1. Resolve the source corpus.
   - Collect the primary source and any supporting docs.
   - Decide which source is canonical and which sources are supporting.

2. Choose the operating mode.
   - Use `spec-backed` mode when the source already has structured sections,
     requirements, constraints, or explicit decisions.
   - Use `free-doc normalization` mode when the source is narrative or rough.
     In that mode, first normalize it into decisions, invariants, constraints,
     non-goals, seams, and verification expectations before drafting tasks.

3. Build the normalized claim inventory.
   - Extract authority statements, scope, invariants, actor boundaries,
     lifecycle rules, error behavior, file seams, test obligations, and stated
     or implied non-goals.
   - Keep provenance for every important claim so the validation matrix stays
     evidence-backed.

4. Derive the execution baseline.
   - Collapse the source into a numbered Decision Summary.
   - Prefer architectural decisions that reduce duplicate layers and preserve
     the existing truthful seam.
   - If the source does not say enough to justify a concrete decision, surface
     the ambiguity instead of inventing certainty.

5. Infer the implementation slices.
   - Group work into a dependency-ordered task chain.
   - Then derive a file-first implementation order that reflects real edit
     clusters.
   - Prefer 5 to 12 main execution tasks unless the source clearly demands a
     narrower or broader split.

6. Build the validation matrix.
   - Map source sections or stable requirement groups to implementation meaning,
     TODO coverage, and status.
   - This section proves the backlog was derived from the source, not improvised.

7. Mark explicit non-goals.
   - Add an `Explicit Phase Boundary` section whenever the source forbids stale
     ideas, future-only concepts, or overclaiming.
   - If a tempting adjacent feature is not part of the current truth, name it
     explicitly as a non-goal.

8. Draft each concrete task.
   - For every task include spec references, a `MANDATORY pre-read` block,
     implementation checklist, file list, tests, and exit condition.
   - Use line ranges only when the source file is stable and locally readable.
   - If the source is a pasted free-doc or line numbers are unstable, use
     section headings instead of fabricated numeric ranges.

9. Add test waves when the phase shape justifies them.
   - If the source already implies a second-half validation sequence, keep test
     waves in the same TODO artifact as modern phases do, using the local
     heading style that the source or exemplar supports.
   - Otherwise keep test duties embedded under the main execution tasks and the
     task-local exit conditions.

10. Close with truthful phase-level closure criteria.
    - If the source phase already uses an explicit closing section such as
      `Completion Gate`, keep it and make it honest.
    - If the exemplar shape does not carry a named closing section, keep the
      closure truth in execution rules and task-local exit conditions instead of
      inventing one just for symmetry.

11. Present ambiguities before pretending the backlog is final.
    - If the source is too thin to derive dependencies, file clusters, or test
      oracles truthfully, stop and return the missing inputs instead of emitting
      a fake high-confidence TODO.

## 📚 Supporting Files

- `REFERENCE.md` defines normalization rules, ambiguity gates, dependency and
  file-order heuristics, and the required modern TODO sections.
- `FORMS.md` contains the canonical backlog template, the task-block template,
  and the ambiguity report shape.

## 💡 Examples

### Example 1: Modern Phase Spec To `NNN-TODO.md`

```text
User: Turn this phase spec into a repo-style execution backlog.

Assistant: resolves the canonical source, extracts the execution baseline,
maps the source sections into a validation matrix, produces `NNN-TODO.md`, and
keeps the modern section order with dependency chain, file-first order,
exemplar-true validation-wave sections, and task-local closure criteria
without inventing a universal extra section.
```

### Example 2: Free-Form Design Note To `TODO.md`

```text
User: Convert this free doc into an implementation TODO.

Assistant: first normalizes the narrative into decisions, constraints,
non-goals, and seams, then emits `TODO.md` with `TODO-01`, `TODO-02`, and so
on, while using section-name pre-read references instead of fake line numbers.
```

### Example 3: Backlog With Explicit Drift Stop

```text
User: Build tasks from this spec draft even if parts are inconsistent.

Assistant: identifies the contradictions, lists them in an ambiguity report,
and asks for resolution before generating a final backlog that would otherwise
hide spec drift.
```

## 📝 Notes

- Good backlog generation is source-first and execution-ordered, not vibe-based.
- The output should feel implementable by another engineer without reopening
  the design loop.
- Do not silently downgrade ambiguity into generic tasks.