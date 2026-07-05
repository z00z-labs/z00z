---
name: smart-docs-fusion
description: 'Fuse multiple Markdown documents into one FUSION.md by parsing H1-H4 sections, merging by topic instead of file order, removing duplication, and proving that no source provision was lost.'
argument-hint: '[input1.md input2.md ...] -> [FUSION.md]'
---

# Smart MD Fusion

## When to Use

- The user wants multiple Markdown documents merged into one canonical document.
- The task requires topic-based fusion with proof that no source provision was lost.
- Duplicate sections across Markdown files should be consolidated instead of concatenated.
- The output must include an audit trail and completeness verification, not just a merged file.

## Mission

Act as a precision Markdown fusion editor.

Given multiple input Markdown files, produce one integrated output document,
defaulting to `FUSION.md`, that is organized by topic rather than source-file order.

The output must be complete enough that the input files could be safely archived or
deleted only after verification confirms that no source provision was lost.

Treat any unproven claim of completeness, section coverage, or deduplication as a failed
fusion, not as a soft warning.

## Primary Output

When this skill is invoked, the AI must:

1. Read every requested input Markdown file in full.
2. Build a semantic map of all H1, H2, H3, and H4 sections.
3. Extract every atomic provision from the body content of each section.
4. Merge overlapping material by topic, not by original file order.
5. Write the fused result to `FUSION.md` unless the user specifies another path.
6. Write a companion audit artifact, defaulting to `FUSION.audit.md` unless the user
  specifies another audit path.
7. Run a completeness, section-coverage, and duplication verification pass before
  finalizing.
8. Run `/doublecheck` in one-shot mode against the final verification summary before
  declaring the fusion complete.

Do not perform a raw concatenation.

## Non-Negotiable Invariants

- No source provision may disappear.
- No source H1, H2, H3, or H4 section may disappear without its content being explicitly
  mapped into the fused output.
- No materially duplicate provision may remain duplicated in the final body.
- If the same topic appears in multiple files, the final output must contain one fused
  section for that topic unless true conflict requires explicit variants.
- The fused section must preserve the strongest, most complete formulation across all
  inputs, including constraints, exceptions, caveats, thresholds, examples, and notes.
- Do not invent facts, requirements, or conclusions that are not supported by the inputs.
- Do not silently discard examples, tables, code blocks, warnings, or edge cases.
- Do not claim that inputs are safe to delete unless both section-level and provision-level
  coverage checks pass.
- If any source content cannot be mapped, fused, or justified, the invocation must end in
  explicit failure or escalation, not silent completion.

## Completion Gate

The skill is complete only if all of the following are true:

- `FUSION.md` exists and is the canonical fused output.
- `FUSION.audit.md` exists and proves source coverage and deduplication decisions.
- Every source H1-H4 section is accounted for in the audit.
- Every extracted provision is accounted for in the audit.
- Section coverage is exactly `100%`.
- Provision coverage is exactly `100%`.
- Unresolved duplicate propositions count is exactly `0`.
- No unresolved duplicate proposition remains in `FUSION.md`.
- Every unresolved semantic conflict is explicitly documented.
- `/doublecheck` has been run on the final verification summary.
- No `/doublecheck` result rated `FABRICATION RISK`, `DISPUTED`, or `UNVERIFIED`
  remains unresolved for claims about completeness, coverage, or deduplication.
- If `/doublecheck` is unavailable, unreachable, or incomplete, the skill must fail closed
  and report that completion could not be established.

## What Counts As A Provision

Treat each of the following as a provision candidate that must be preserved or fused:

- A normative statement, requirement, rule, or instruction
- A factual claim, assumption, decision, conclusion, or recommendation
- A warning, caveat, exception, limitation, or edge case
- A numbered or bulleted item
- A table row that carries distinct meaning
- A code block or command example that adds unique operational value
- A definition or terminology statement

If one paragraph contains multiple independent statements, split it into multiple atomic
provisions during analysis.

## Fusion Workflow

### Step 1: Ingest Sources

- Read every input file fully.
- Record the source path.
- Record the heading path for every content block using H1-H4 ancestry.
- Preserve enough source traceability to map every provision back to its origin.
- Build a section inventory for every source file covering all H1-H4 heading paths,
  including empty or thin sections.

### Step 2: Extract Atomic Provisions

- Break each section body into atomic provisions.
- Assign each provision an internal source identifier such as `SRC-001`.
- Keep the original heading path with each provision.
- Preserve exact values, numbers, qualifiers, and exceptions.
- Record the parent source section identifier for every provision.

### Step 3: Normalize Topics

- Group provisions by semantic topic, not by literal heading text.
- Merge synonymous headings into one topic when they discuss the same subject.
- Keep closely related but distinct topics separate.
- If a source section spans multiple topics, split its provisions across the correct
  target topics.

### Step 4: Fuse By Topic

For each topic cluster:

- Collapse exact duplicates into one statement.
- Merge partial overlaps into one richer statement containing all unique meaning.
- Retain the clearest wording while preserving every source constraint.
- Prefer integrated prose over repeated bullet duplication.
- Preserve examples, tables, or code blocks when they add unique value.
- If multiple sources contribute complementary detail, produce one fused section that
  reads as if it had been authored once.
- Prefer the most information-dense and least lossy fused formulation, not the shortest
  formulation.
- If two source sections cover the same theme at different granularity, absorb both into
  one canonical structure instead of keeping parallel near-duplicate sections.

### Step 5: Handle Conflicts Explicitly

If two sources disagree or are not safely reconcilable:

- Do not choose one silently.
- Create a single topic section with an explicit conflict or variants subsection.
- Preserve both positions and label their provenance.
- State that the topic requires human resolution if no deterministic merge is possible.

### Step 6: Compose The Final Document

- Write one coherent `FUSION.md`.
- Organize the output by merged topics, not by input file sequence.
- Use a clean heading hierarchy.
- Avoid empty headings and orphan subsections.
- Remove repeated statements that no longer add meaning.
- Keep the final document readable as a standalone canonical source.

### Step 7: Write The Audit Artifact

Write `FUSION.audit.md` as a mandatory verification artifact.

It must include at least these sections:

- `## Source Files`
- `## Source Section Inventory`
- `## Provision Coverage Matrix`
- `## Deduplication Decisions`
- `## Conflict Register`
- `## Deletion-Safety Verdict`
- `## Doublecheck Review`

The audit must make it possible to prove that the original documents can be reconstructed
in meaning from the fused output, even if the original file order is lost.

## Mandatory Audit Schema

`FUSION.audit.md` must contain these mandatory tables or equivalent machine-readable lists.

### Source Section Inventory

Each row must contain:

- `Section ID`
- `Source File`
- `Heading Level`
- `Heading Path`
- `Provision Count`
- `Destination Section IDs`
- `Status` using only `FUSED`, `SPLIT`, `CONFLICT`, or `BLOCKED`

### Provision Coverage Matrix

Each row must contain:

- `Provision ID`
- `Source File`
- `Source Section ID`
- `Source Heading Path`
- `Provision Summary`
- `Destination Section ID`
- `Coverage Status` using only `FUSED`, `CONFLICT`, or `BLOCKED`
- `Notes`

### Deduplication Decisions

Each row must contain:

- `Decision ID`
- `Duplicate Source Provision IDs`
- `Kept In Destination`
- `Removal Rationale`
- `Why No Meaning Was Lost`

### Conflict Register

Each row must contain:

- `Conflict ID`
- `Topic`
- `Source Provision IDs`
- `Conflict Description`
- `Why Automatic Fusion Was Unsafe`
- `Required Human Resolution`

### Doublecheck Review

Each row must contain:

- `Claim ID`
- `Verification Claim`
- `Doublecheck Rating`
- `Disposition`
- `Follow-Up Action`

## Required Verification Pass

Before finalizing `FUSION.md`, the AI must verify all of the following and record the
results in `FUSION.audit.md`:

### Section Coverage Check

- Every source H1-H4 section appears in the section inventory.
- Every source H1-H4 section maps to one or more destination sections in `FUSION.md`.
- If a source section is split across multiple fused sections, the audit must list all
  destination sections explicitly.
- No source section may remain unmapped.
- A source section may be treated as fully fused only if all of its provisions are either
  fused into destination sections or explicitly placed in the conflict register.
- A source section may not be marked complete merely because a similar heading exists in
  `FUSION.md`.

### Coverage Check

- Every extracted source provision maps to exactly one destination section, or to one
  explicit conflict block if it cannot be fully fused.
- No source provision remains unmapped.
- Every mapped provision must reference both its source section and destination section.
- A provision may not be omitted on the basis that it is "minor", "implicit", "stylistic",
  or "already obvious".

### Duplication Check

- No final section repeats the same proposition in multiple phrasings without adding
  new information.
- Duplicate bullets created by file-order merging must be removed.
- Near-duplicates must be fused, not merely reworded side by side.
- The audit must record why removed duplicates were true duplicates rather than distinct
  provisions.
- If two statements overlap but differ in scope, threshold, exception, or operational detail,
  they are not duplicates and must be fused or separated explicitly.

### Fidelity Check

- Numeric values, thresholds, commands, filenames, caveats, and exceptions still exist
  in the fused output.
- Examples or code blocks with unique operational meaning still exist in the fused output
  or are represented without loss.
- If a table or list was collapsed into prose, the audit must explain why no meaning was
  lost.

### Best-Of Fusion Check

- For every topic represented by multiple sources, the fused section must contain the best
  available combination of clarity, completeness, and constraints from those sources.
- If one source contributes clearer language and another contributes deeper details, the
  final section must contain both advantages.
- The final section must not be weaker, narrower, or less actionable than any of its
  inputs.

### Self-Verification Check

- The AI must produce a concise verification summary stating whether section coverage,
  provision coverage, duplication removal, fidelity, and deletion-safety passed.
- That summary must be reviewed with `/doublecheck` in one-shot mode.
- The summary must state exact gate values for section coverage, provision coverage,
  unresolved duplicates, and conflict count.
- If `/doublecheck` flags any claim about completeness, coverage, or deduplication as
  `FABRICATION RISK`, `DISPUTED`, or `UNVERIFIED`, the skill must not declare success.
- The AI must continue auditing, revise the merge, or escalate the unresolved issue.
- If `/doublecheck` cannot be executed, the skill must return `BLOCKED` rather than `PASS`.

### Deletion-Safety Check

The inputs may be considered safe to archive or delete only if:

- Section coverage is complete.
- Coverage is complete.
- No unresolved duplicate content remains.
- All unresolved conflicts are explicitly documented.
- The final document can serve as the canonical source for the merged scope.
- `/doublecheck` did not invalidate the verification summary.

## Output Contract

After writing the fused document, report:

1. The output path
2. The audit path
3. The list of input files used
4. The number of source H1-H4 sections inventoried
5. The number of extracted provisions
6. The number of merged topic sections
7. Whether section coverage verification passed
8. Whether provision coverage verification passed
9. Whether duplication verification passed
10. Whether `/doublecheck` passed the verification summary
11. Any unresolved conflicts or manual-review items

These status fields must be reported as explicit `PASS`, `FAIL`, or `BLOCKED`, not vague
natural-language summaries.

Do not dump the full document into chat unless the user asks.

## Decision Rules

- If headings differ but topic meaning is the same, merge by meaning.
- If wording differs but meaning is identical, keep one best formulation.
- If wording differs and each version adds detail, fuse them into one fuller statement.
- If two statements appear similar but imply different scope, keep them separate.
- If a section is mostly example material, attach it to the main topic instead of
  creating a duplicate top-level topic.
- If one document is more detailed and another is clearer, keep the clarity and absorb
  the missing detail.
- If a table duplicates prose, keep the representation that preserves the most usable
  structure and remove the redundant one.

## Preferred Internal Working Model

Use this internal sequence when reasoning:

1. Inventory sections
2. Extract provisions
3. Cluster by topic
4. Fuse duplicates and overlaps
5. Compose canonical structure
6. Write audit artifact
7. Verify section coverage
8. Verify source provision coverage
9. Verify no duplication
10. Run `/doublecheck` on the verification summary
11. Finalize output

## Failure Modes To Avoid

- Concatenating files in original order
- Merging by heading text only
- Losing caveats hidden in bullets or tables
- Keeping near-duplicates because the wording is not identical
- Dropping code samples or commands that carry unique meaning
- Hiding conflicts by choosing one source silently
- Claiming completeness without an explicit coverage check
- Claiming section fusion without a section inventory and mapping table
- Declaring success before `/doublecheck` reviews the final verification summary
- Reporting fuzzy success language instead of explicit `PASS`, `FAIL`, or `BLOCKED` gates
- Marking a section as fused without proving provision-level coverage for that section

## Example Invocations

- Merge these reports into one canonical doc: `docs/a.md docs/b.md docs/c.md -> docs/FUSION.md`
- Fuse these architecture notes by topic and verify that nothing is lost.
- Build `FUSION.md` from these markdown files, eliminate duplicates, and keep a full
  coverage check.
