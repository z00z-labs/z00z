# Spec To Tasks Forms

## Modern TODO Artifact Template

````markdown
# PREFIX-TODO

Canonical design source:

- [PRIMARY-SOURCE](./PRIMARY-SOURCE.md)
- optional supporting sources

Execution rules:

- execute this file in order unless a dependency note says otherwise;
- treat the source as normative for requirement meaning and this file as
  normative for execution order;
- do not pull requirements from retired drafts or stale planning notes during
  implementation;
- do not add a parallel layer, facade, verifier, pipeline, or persistence path
  when an existing truthful seam can be extended;
- if execution discovers a new design constraint, update the source first,
  then this backlog, then the affected tests;
- before starting any numbered task, complete its `MANDATORY pre-read` block.

## 🎯 Decision Summary

The execution baseline for this backlog is:

1. decision one
2. decision two
3. decision three

## 🔗 Dependency Chain

Execution dependency chain:

1. `PREFIX-01` foundational seam
2. `PREFIX-02` producer or builder migration
3. `PREFIX-03` verifier or consumer path
4. `PREFIX-04` integration or persistence path
5. `PREFIX-05` closure and regression path

Hard dependencies:

- `PREFIX-02` depends on `PREFIX-01`
- `PREFIX-03` depends on `PREFIX-01` and `PREFIX-02`

## 🗂️ File-First Implementation Order

Edit order by file cluster:

1. `path/to/file_one.rs`
2. `path/to/file_two.rs`
3. new `path/to/new_file.rs`
4. test files

## ✅ Validation Matrix

| Source section | Required theme | TODO coverage | Status |
| --- | --- | --- | --- |
| `Authority And Scope` | keep one source of truth and reject stale drafts | execution rules; task exit conditions; optional closing section | Mapped as execution guardrail |
| `Current Contract` | preserve the live seam and avoid duplicate layers | `PREFIX-01` through `PREFIX-04` | Validated mapped |

## 🚫 Explicit Phase Boundary

The following topics are intentionally out of scope for this backlog:

- non-goal one;
- future-only concept two;
- stale draft path three.

## ⚙️ Concrete Execution Tasks

### PREFIX-01 Task Name

Spec references:

- `Section One`
- `Section Two`

MANDATORY pre-read in `PRIMARY-SOURCE.md`:

- lines `10-30`
- lines `40-52`

Alternative pre-read form when line anchors are not stable:

- section `Authority And Scope`
- section `Current Contract`

- [ ] implementation step one
- [ ] implementation step two
- [ ] implementation step three

Files:

- `path/to/file_one.rs`
- `path/to/file_two.rs`

Tests:

- [ ] extend `path/to/test_file.rs`
  - positive scenario
  - negative scenario

Exit condition:

- one sentence that says what is now true and what duplication or drift no
  longer exists.

### PREFIX-02 Task Name

Spec references:

- `Section Three`

MANDATORY pre-read in `PRIMARY-SOURCE.md`:

- section `Section Three`

- [ ] implementation step one
- [ ] implementation step two

Files:

- `path/to/file_three.rs`

Tests:

- [ ] extend `path/to/another_test.rs`
  - assertion one

Exit condition:

- task-local closure sentence.

## 🧪 Dedicated Validation Wave Section

Use the local truthful heading from the source or closest exemplar, for
example `Concrete Test Execution Tasks` or `Mandatory Test Waves`.

### PREFIX-10 Harness And Reuse Lock-In

Spec references:

- `Authority And Scope`
- `Security Invariants`

MANDATORY pre-read in `PRIMARY-SOURCE.md`:

- section `Authority And Scope`

- [ ] assign one truthful test home per scenario
- [ ] lock fixture notes for every major seam

Files:

- `path/to/test_one.rs`
- `path/to/test_two.rs`

Tests:

- [ ] ensure every scenario has exactly one selected home

Exit condition:

- no scenario still has an ambiguous test home.

## ✅ Optional Phase-Level Closure Section

Use the local truthful heading when needed, for example `Completion Gate`.
Omit this section entirely when execution rules plus task exit conditions
already carry the closure truth.

This backlog is complete only when all of the following hold:

- every numbered task is implemented or explicitly deferred by a source update;
- the required tests are green;
- the source and this TODO still agree on design and order;
- no forbidden parallel layer or stale concept has been introduced.
````

## Task Identifier Rules

```text
Phase-backed output:
- 045-01
- 045-02

Unprefixed output:
- TODO-01
- TODO-02

Custom prefix output:
- WALLET-01
- WALLET-02
```

## Ambiguity Report Template

````markdown
## Ambiguities That Block Final TODO Generation

| ID | Source Conflict Or Gap | Why It Blocks Truthful Tasking | Needed Resolution |
| --- | --- | --- | --- |
| A1 | section one says X, section two says Y | dependency order changes depending on which statement is true | choose one canonical source |
| A2 | no file seam is named for persistence | file-first order would be guessed instead of derived | provide or confirm the target subsystem |
````

## Decision-Summary Extraction Prompt

Use this prompt internally before drafting the final backlog:

- Which design choices must be frozen so implementation does not re-open the
  architecture?
- Which tempting adjacent solutions must be rejected explicitly?
- Which producer, verifier, persistence, or reporting seam should remain the
  live authority surface?

## Dependency Prompt Fragments

Use these prompts while deriving the task chain:

- Which task creates the seam that later tasks rely on?
- Which consumer task would be fake if the producer task did not land first?
- Which persistence or reload path depends on earlier contract work?
- Which tests only become meaningful after multiple earlier tasks stabilize?
