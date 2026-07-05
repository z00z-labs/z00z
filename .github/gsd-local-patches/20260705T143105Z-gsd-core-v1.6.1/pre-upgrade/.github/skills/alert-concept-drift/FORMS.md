# Alert Concept Drift Forms

Use these templates when producing a concept-drift audit.

## Detailed Report Template

```markdown
# Concept Drift Report

## Scope

- Baseline reference:
- Current reference:
- Scope:
- Focus dimensions:
- Report date:

## Executive Verdict

- Overall conclusion:
- Confirmed suspicious findings:
- Critical regressions:
- Cleared healthy evolution items:
- Ambiguous items:

## Baseline Concept Inventory

| ID | Dimension | Historical concept | Historical evidence |
| --- | --- | --- | --- |
| B01 | security | ... | ... |

## Candidate Classification Table

| ID | Dimension | Candidate summary | Initial class | Doublecheck result | Final class | Severity | Confidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| C01 | crypto | ... | suspicious_drift | confirmed | suspicious_drift | high | high |

## Findings First

### Confirmed Drift Findings

Repeat the finding card below for each `suspicious_drift` or
`critical_regression` item.

### Ambiguous Or Blocked Items

Repeat the finding card below for each `ambiguous` item and name the missing
evidence.

## Cleared Healthy Evolution

List items that looked risky at first but were cleared as
`expected_evolution` or `justified_change`.

## Doublecheck Ledger

| ID | What was challenged | Alternative explanation tested | Outcome | Notes |
| --- | --- | --- | --- | --- |
| C01 | fail-open candidate | stale docs vs real code drift | confirmed | ... |

## Recommendations

- Immediate actions:
- Follow-up validation:
- Documentation updates:
- Optional cleanup:
```

## Finding Card Template

```markdown
### [ID] [Dimension] [Final class] [Severity]

- Baseline concept:
- Current behavior:
- Why this is not just a diff:
- Historical evidence:
- Current evidence:
- Doublecheck outcome:
- Confidence:
- Recommended action:
```

## Cleared Evolution Card Template

```markdown
### [ID] [Dimension] cleared as [expected_evolution|justified_change]

- Initial concern:
- Clearing evidence:
- Why the concept is still intact:
- Optional note:
```

## Minimal Summary Table For Chat-Only Output

```markdown
| ID | Dimension | Verdict | Why it matters |
| --- | --- | --- | --- |
| C01 | security | suspicious_drift | validation weakened from fail-closed to soft warning |
```
