---
name: spec-to-code-compliance
description: Auto-invoked when user wants to check whether code matches the spec, compare implementation to docs, find what is missing from the implementation, or identify behavior that exists in code but not in requirements. Also triggers on spec compliance, design-to-code alignment, whitepaper audit, protocol conformance, requirements traceability, undocumented behavior, implementation gap, and spec drift.
---

# Spec-to-Code Compliance

Verify whether an implementation matches the documented intent, requirements,
flows, invariants, formulas, and guarantees described by its specification or
design corpus.

## When to Use

- User asks whether code matches a specification, design document, whitepaper,
  README, ADR, workflow document, or requirements file.
- The task is to find missing implementation, undocumented behavior, or drift
  between intended behavior and actual code.
- A review needs traceable evidence that a claim in documentation is enforced,
  partially enforced, contradicted, or absent in code.
- The project has at least one meaningful spec source and at least one code
  implementation to compare against it.
- The code may have been written by a human, AI assistant, or mixed workflow.
  This skill is coder-agnostic: audit the artifact, not the author.

## When Not to Use

- There is no usable specification, requirements source, or design corpus.
- The task is a general code review, bug hunt, or security audit unrelated to
  documented intent.
- The user wants help writing or improving documentation instead of verifying
  compliance.
- The codebase is too incomplete to compare meaningfully against any stated
  behavior.

## How It Works

1. **Build the spec corpus**
   - Collect every source that describes intended behavior: specs, whitepapers,
     ADRs, design notes, README sections, comments with normative claims,
     issue text, or acceptance criteria.
   - Treat diagrams, tables, and formulas as spec material once converted into
     explicit text.
   - Keep provenance for every claim so later mapping stays traceable.

2. **Normalize the documented intent**
   - Extract clear behavioral units from the spec corpus: invariants,
     preconditions, postconditions, state transitions, formulas, ordering
     rules, actor permissions, error behavior, and security guarantees.
   - Separate explicit claims from inferred context. If a point is ambiguous,
     label it as ambiguous instead of guessing.
   - Convert each extracted claim into a reviewable item with source section,
     quote or paraphrase, semantic type, and confidence.

3. **Extract the code behavior**
   - Read the implementation at the level needed to trace real behavior:
     state reads and writes, control flow, error paths, boundary checks,
     external calls, formulas, permission checks, and side effects.
   - Capture what the code actually guarantees, not what naming or comments
     imply.
   - Track hidden behavior too: initialization order, defaults, fallback logic,
     implicit assumptions, and undocumented branches.

4. **Align spec to code claim-by-claim**
   - For each spec item, find the matching code path or conclude that none is
     present.
   - For each code behavior, ask whether it is documented, stronger than the
     spec, weaker than the spec, or undocumented.
   - Require evidence on both sides: a spec quote and a code reference or a
     documented absence of one.

5. **Classify the alignment result**
   - `full_match`: code implements the claim as documented.
   - `partial_match`: some but not all documented behavior is enforced.
   - `missing_in_code`: the spec claim is not implemented.
   - `mismatch`: code behavior contradicts the documented claim.
   - `code_stronger_than_spec`: code enforces a stricter rule than documented.
   - `code_weaker_than_spec`: code under-enforces the documented claim.
   - `undocumented_code_behavior`: code does something material that the spec
     does not mention.
   - `ambiguous_spec`: the documentation is too unclear to justify a stronger
     conclusion.

6. **Score confidence and ambiguity**
   - Assign confidence to each mapping based on evidence quality, traceability,
     and semantic clarity.
   - Low confidence is not a substitute for analysis. Push until the result is
     either well-supported or explicitly marked ambiguous.
   - Never infer what the spec "must have meant" without textual support.

7. **Classify divergence severity**
   - `critical`: documented guarantees are contradicted or a missing invariant
     can cause serious failure or exploitation.
   - `high`: major behavior is partially implemented, misordered, or materially
     undocumented.
   - `medium`: edge-case handling, error behavior, or constraints do not align
     and may affect safety or correctness.
   - `low`: wording drift, minor semantic mismatch, or documentation lag with
     low operational impact.

8. **Report with traceability**
   - For every finding, include the spec claim, code evidence, alignment class,
     severity, confidence, and reasoning.
   - Distinguish clearly between missing implementation, contradictory
     implementation, undocumented implementation, and ambiguous documentation.
   - End with the next verification step needed to raise certainty, such as
     reading one more module, resolving an ambiguous phrase, or adding a
     requirements note.

## Evidence Rules

- Never claim compliance without both a documented requirement and supporting
  code evidence.
- Never claim non-compliance solely from absence of a quick search hit; verify
  surrounding control flow and related modules first.
- Treat comments, names, and README summaries as lower-grade evidence than real
  enforcement logic.
- If multiple documents conflict, report the contradiction instead of choosing
  one silently.
- If code is stronger than docs, record that as a separate class instead of
  forcing it into mismatch.

## Review Output

When using this skill, structure the output around these fields:

- Scope
- Spec corpus
- Extracted claims
- Alignment findings ordered by severity
- Evidence and confidence for each finding
- Ambiguities and contradictions
- Recommended remediation

## Examples

### Example 1: Protocol Whitepaper Against Code

```text
User: Check whether this implementation matches the protocol whitepaper.
Assistant: First extract the whitepaper claims into explicit behavioral items,
then map each claim to code paths and classify matches, gaps, contradictions,
and undocumented behavior with evidence on both sides.
```

### Example 2: Requirements Against Service Code

```text
User: Compare these requirements to the current service implementation and show
what is missing.
Assistant: Build a normalized claim set from the requirements, extract the real
service behavior, then report missing implementation, weaker enforcement, and
undocumented branches with confidence and severity.
```

### Example 3: Mixed Human And AI Code

```text
User: Audit whether this feature matches the design doc. Some files were
written by AI, some by people.
Assistant: Keep the review coder-agnostic, compare only the design claims and
the implemented behavior, and report divergence using traceable evidence rather
than assumptions about who wrote the code.
```

## Notes

- This is a compliance and traceability review, not a generic bug hunt.
- Ambiguity is a finding category, not a reason to guess.
- Strong spec-to-code review is claim-by-claim, not vibe-based.
- If the user asks for a review, findings come first and summary stays short.