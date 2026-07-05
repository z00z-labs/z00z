---
name: z00z-l0-spec-gate
description: Auto-invoked when user wants to check docs, specs, requirements, whitepapers, invariants, or traceability for Z00Z. Also triggers on L0 verification, specification consistency, markdown validation, YAML invariants, traceability matrix, mdBook, lychee, taplo, and spec-before-code review.
---

# Z00Z L0 Spec Gate

Verify document, specification, invariant, and traceability consistency before or alongside code changes.

## When to Use

Use this skill when:

- The user changes docs, tech papers, specifications, invariants, threat models, or planning material.
- A code change introduces a new security-critical concept that needs an invariant ID.
- The task asks whether docs and code still agree.

## Workflow

1. Read the changed docs/spec files and the affected code paths directly.
2. Run `scripts/check-docs.sh` from this skill.
3. Run `scripts/check-traceability.py`; use strict mode for release gates.
4. Run `scripts/extract-invariants.py` when a new spec or protocol doc was added.
5. Report missing invariants as UNKNOWN, not as verified.

## Gate Criteria

- Markdown/TOML/YAML syntax checks must pass when the relevant tools are installed.
- Security-critical Rust code should reference an existing `ZINV:` invariant when strict traceability is enabled.
- No new proof, transcript, checkpoint, settlement, wallet, or parser behavior should exist only in prose without a machine-checkable follow-up.

## Scripts

```bash
.github/skills/z00z-l0-spec-gate/scripts/check-docs.sh
python3 .github/skills/z00z-l0-spec-gate/scripts/check-traceability.py --strict
python3 .github/skills/z00z-l0-spec-gate/scripts/extract-invariants.py --out reports/z00z-verification-orchestrator-<YYYYMMDD-HHMMSS>/verification<timestamp>/reports/extracted-invariants.json
```

## Examples

```text
User: Check whether this voucher spec is ready for implementation.
Assistant: Runs L0 checks, extracts invariants, checks traceability, and marks gaps before code starts.
```

```text
User: I changed a checkpoint proof doc.
Assistant: Checks markdown, extracts invariants, and requires a matching L1/L3 follow-up if the invariant affects code.
```
