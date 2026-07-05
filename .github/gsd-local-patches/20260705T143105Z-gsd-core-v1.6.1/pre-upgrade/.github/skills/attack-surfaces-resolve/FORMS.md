# Attack Surfaces Resolve Forms

## Resolve Spec Template

````markdown
# Attack Surfaces Resolve Spec

## Metadata
- Generated At: <ISO8601>
- Input DB: <path>
- Output Spec: <path>
- Closure Ledger: <path>
- Filters: <id/class/keyword summary>
- Run Status: <fully-resolved|partial|blocked|blocked:no-target|plan-only>

## Target Findings
| Finding ID | Class | Severity | Confidence | Exploitability | Scope | Boundary | Evidence Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| <id> | <attack_class> | <severity> | <confidence> | <exploitability> | <paths> | <trust_boundary> | <valid|missing|changed> |

## Finding Coverage Summary
| Finding ID | Reproduced/Proven | Patch | Anti-Test | Positive Test | Verification | Re-Scan | Final State |
| --- | --- | --- | --- | --- | --- | --- | --- |
| <id> | <pass|blocked> | <applied|plan-only|none> | <pass|fail|missing> | <pass|fail|missing> | <pass|fail|blocked> | <pass|fail|blocked> | <resolved|mitigated|blocked|plan-only> |

## Closure Gate Matrix
| Finding ID | Gate | Status | Evidence |
| --- | --- | --- | --- |
| <id> | finding-load | <pass|fail|blocked> | <source paths checked> |
| <id> | reproduction-or-proof | <pass|fail|blocked> | <test/proof obligation> |
| <id> | fix-contract | <pass|fail|blocked> | <invariant> |
| <id> | candidate-selection | <pass|fail|blocked> | <candidate id> |
| <id> | implementation | <pass|fail|blocked|plan-only> | <files changed> |
| <id> | proof-of-fix-tests | <pass|fail|blocked> | <test names> |
| <id> | verification | <pass|fail|blocked> | <commands> |
| <id> | rescan | <pass|fail|blocked> | <scan/audit result> |
| <id> | ledger | <pass|fail|blocked> | <ledger entry id> |

## Candidate Solutions
| Candidate ID | Finding ID | Score | Validation A | Validation B | Doublecheck | Selectable |
| --- | --- | ---: | --- | --- | --- | --- |
| <candidate> | <finding> | <score> | <pass|fail> | <pass|fail> | <pass|pass-with-risk|blocked|fail> | <yes|no> |

## Candidate Cards
### Candidate <id>
- Finding ID: <id>
- Problem: <source-backed vulnerability statement>
- Fix Contract: <new invariant and fail-closed behavior>
- Proposed Controls:
  - <code action>
- Positive Examples:
  - <valid behavior preserved>
- Negative Anti-Examples:
  - <original or adjacent attack rejected>
- Pros:
  - <pro>
- Cons:
  - <con>
- Validation A: <status and notes>
- Validation B: <status and notes>
- Doublecheck Summary: <status and notes>
- Residual Risk: <bounded risk or none known>

## Selected Fix Contract
- Finding ID(s): <ids>
- Candidate ID(s): <ids>
- Violated Assumption Closed: <assumption>
- Trust Boundary Enforced: <boundary>
- New Invariant: <invariant>
- Fail-Closed Behavior: <behavior>
- Why Alternatives Were Rejected: <brief rationale>

## Implementation Patch Summary
| File | Change | Finding IDs |
| --- | --- | --- |
| <path> | <summary> | <ids> |

## Test And Anti-Test Matrix
| Finding ID | Test Type | Test Name/Path | What It Proves | Result |
| --- | --- | --- | --- | --- |
| <id> | anti-test | <test> | <original attack blocked> | <pass|fail|not-run> |
| <id> | positive | <test> | <valid behavior preserved> | <pass|fail|not-run> |
| <id> | property/fuzz/integration | <test> | <boundary robustness> | <pass|fail|not-run> |

## Verification Command Log
| Command | Scope | Result | Notes |
| --- | --- | --- | --- |
| `<command>` | <finding/module/crate> | <pass|fail|blocked> | <summary> |

## Re-Scan Or Source-Audit Evidence
| Finding ID | Method | Scope | Result | Evidence |
| --- | --- | --- | --- | --- |
| <id> | <attack-surfaces-create|focused-source-audit> | <paths> | <pass|fail|blocked> | <summary> |

## Closure Decisions
| Finding ID | Final State | Doublecheck | Residual Risk | Reason |
| --- | --- | --- | --- | --- |
| <id> | <resolved|mitigated|blocked|plan-only> | <pass|pass-with-risk|blocked|fail> | <risk> | <why this state is justified> |

## Closure Ledger Entries
```jsonl
{"finding_id":"<id>","closed_at":"<YYYY-MM-DD HH:MM:SS>","state":"<resolved|mitigated|blocked|plan-only>","resolver":"attack-surfaces-resolve","source_db_path":"<path>","changed_files":["<path>"],"tests":["<test>"],"anti_tests":["<test>"],"verification_commands":[{"command":"<command>","result":"<pass|fail|blocked>"}],"rescan_result":"<pass|fail|blocked>","residual_risk":"<risk>","doublecheck_status":"<pass|pass-with-risk|blocked|fail>"}
```
````

## Candidate Object Schema

```json
{
  "candidate_id": "cand-001",
  "finding_id": "as-...",
  "title": "Short mitigation title",
  "fix_contract": {
    "violated_assumption": "...",
    "trust_boundary": "...",
    "new_invariant": "...",
    "fail_closed_behavior": "..."
  },
  "actions": ["..."],
  "positive_examples": ["..."],
  "negative_anti_examples": ["..."],
  "tests": {
    "positive": ["..."],
    "anti": ["..."],
    "property_fuzz_or_integration": ["..."]
  },
  "pros": ["..."],
  "cons": ["..."],
  "score": 0,
  "validation_a": {
    "status": "pass",
    "notes": []
  },
  "validation_b": {
    "status": "pass",
    "notes": []
  },
  "doublecheck": {
    "status": "pass",
    "notes": []
  },
  "selectable": true,
  "residual_risk": "..."
}
```

## Closure Ledger JSONL Schema

```json
{
  "finding_id": "as-...",
  "closed_at": "YYYY-MM-DD HH:MM:SS",
  "state": "resolved",
  "resolver": "attack-surfaces-resolve",
  "source_db_path": "reports/attack-surfaces/db.jsonl",
  "changed_files": ["src/file.rs"],
  "tests": ["test_valid_flow"],
  "anti_tests": ["test_original_attack_rejected"],
  "verification_commands": [
    {
      "command": "cargo test -p crate test_original_attack_rejected",
      "result": "pass"
    }
  ],
  "rescan_result": "pass",
  "residual_risk": "none known",
  "doublecheck_status": "pass"
}
```

## Doublecheck Prompt Form

Use this prompt per candidate:

```text
/doublecheck verify this remediation candidate:
- finding_id: <finding_id>
- candidate_id: <candidate_id>
- source finding fields: <main_vulnerability, violated_assumption, trust_boundary, evidence>
- proposed controls: <actions>
- positive examples: <valid behavior preserved>
- negative anti-examples: <attacks rejected>
- expected tests: <tests>
Check workspace evidence first and flag unsupported claims.
```

Use this prompt for the final closure claim:

```text
/doublecheck verify this closure claim:
- finding_id: <finding_id>
- changed files: <paths>
- anti-tests: <tests proving original attack is blocked>
- positive tests: <tests proving valid behavior>
- verification commands and results: <commands>
- re-scan/source-audit evidence: <evidence>
Reject the closure if any required proof is missing.
```
