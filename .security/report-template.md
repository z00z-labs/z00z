# Phase 066 Pentest Report Template

Use this template as the canonical human-facing structure for Phase 066 report
exports under `reports/z00z-pentests_report-<timestamp>/`.

## Required Sections

1. Run identity
   - run id
   - generated at
   - mode
   - profile
   - artifact root
   - host report root
2. Scope and authorization
   - scope status
   - allowed hosts
   - allowed URLs
   - allowed paths
3. Tool versions and exact commands
   - per-tool status
   - command string when executed
   - stdout, stderr, exit, and raw artifact paths
4. Findings summary ordered by severity
   - critical
   - high
   - medium
   - low
   - info
5. Per-finding evidence
   - source file or source evidence
   - scanner artifact
   - local reproduction or proof
   - confidence
   - false-positive status or reason when applicable
   - fix recommendation
   - required regression test
6. Open questions and skipped scans
   - missing tools
   - skipped scans with explicit reason
   - unconfirmed findings that still require evidence
7. Redaction notes
   - no secret values copied into host-facing Markdown
8. Doublecheck
   - review-loop evidence status

## Evidence Rules

- A scanner signal is never a confirmed finding by itself.
- A finding may be `confirmed`, `false-positive`, `unconfirmed`, or `skipped`.
- `confirmed` requires `source_evidence`, `scanner_artifact`, `proof`,
  `confidence`, `fix_recommendation`, and `regression_test`.
- `false-positive` requires `scanner_artifact` and `false_positive_reason`.
- `unconfirmed` requires `scanner_artifact` and `confidence`.
- `skipped` requires `skip_reason`.
