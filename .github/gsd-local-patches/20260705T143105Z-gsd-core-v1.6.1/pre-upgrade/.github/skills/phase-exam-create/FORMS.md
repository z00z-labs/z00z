# Phase Exam Forms

## Standard Exam Generation Card

```markdown
## Phase Exam Run: <phase-id>

**Status:** <accepted | no-candidate>
**Question Count:** <N>
**Variants Attempted:** <N>
**Accepted Variant Index:** <index or none>
**Scope Paths:** <path1, path2>
**Report Path:** <path.md>

### Candidate Audit

**Pros**
- <strong point 1>
- <strong point 2>

**Cons**
- <risk or weakness 1>
- <risk or weakness 2>

**Decision:** <accepted | rejected>

### Verification Gate

**Gate:** <passed | failed>
**Reason:** <why>

### Residual Risk

<what remains unresolved even after accepted generation>
```

## Accepted Database Entry Shape

```json
{
  "id": "PEX-20260501-001",
  "signature": "sha256:<stable-signature>",
  "created_at": "2026-05-01T00:00:00Z",
  "phase_id": "032-crypto-audit-scenario-1",
  "scope_paths": [".planning/phases/032-crypto-audit-scenario-1"],
  "report_path": ".planning/phases/032-crypto-audit-scenario-1/032-EXAM-QUESTIONS-AND-ANSWERS.md",
  "variant_index": 7,
  "score": 87,
  "verification": {
    "passed": true,
    "reason": "all gates passed"
  },
  "pro_con_audit": {
    "pros": ["coverage diversity", "no breadcrumb leakage"],
    "cons": ["one question needed rewrite"],
    "decision": "accepted"
  },
  "questions": [
    {
      "index": 1,
      "theme": "Closure Integrity",
      "title": "Claim Boundary",
      "quest": "What evidence closes the claim that this phase enforces the intended boundary under adversarial inputs?"
    }
  ]
}
```

## No-Candidate Report Stub

```markdown
## Scan Result

No candidate question set passed the quality and verification gate.

### Rejection Summary

- insufficient scenario diversity
- breadcrumb leakage in question phrasing
- duplicate or weak repository-evidence prompts
```
