# Z00Z Verification Final Report Format

This file defines the canonical Markdown contract for `z00z-verification-report.md`.

The report is a run-root artifact summary. It must describe only evidence that exists under the active run root plus stable repository metadata explicitly cited in the report. It must not invent results, upgrade tool conclusions, or turn missing models into proof claims.

## 🎯 Format Goals

- Standard structure across all orchestrator runs.
- Evidence-first wording with explicit artifact paths.
- Separation between:
  - project-owned fixable findings
  - protected vendor findings
  - missing evidence or missing models
- Clear validity ceiling for every conclusion.
- Performance and resource profiling grounded in run-root artifacts.
- Professional Markdown suitable for audit review.

## 📦 Mandatory Inputs

The report must cite, when present:

- `logs/`
- `profiling/events.tsv`
- `profiling/summary.json`
- `profiling/tool-availability.json`
- `profiling/resources/`
- `profiling/resources-summary.json`
- `profiling/run-footprint.json`
- `profiling/hjmt-summary.json`
- `coverage/manifest.tsv`
- `coverage/summary.json`
- `bootstrap-summary.json`
- `runtime-bootstrap-summary.json`
- `security/adversarial-summary.json`
- `security/adversarial-review.md`
- any gate-local artifacts referenced from the gate matrix

All artifact paths must be rendered relative to the repository root.

## 🧭 Required Section Order

The final report must use this section order.

1. `# Z00Z Verification Orchestrator Report`
2. `## 🎯 Executive Verdict`
3. `## 📦 Evidence Provenance`
4. `## 🚦 Gate Matrix`
5. `## 🧪 Conclusion Ledger`
6. `## 🔍 Validity And Doublecheck`
7. `## 🏗️ Bootstrap Artifact Provenance`
8. `## 📊 Performance And Resource Profiling`
9. `## 🌲 HJMT Runtime Evidence`
10. `## 🗺️ Coverage Inventory` for `project` scope
11. `## 🚨 Risk Register`
12. `## 🔗 Supply-Chain Highlights`
13. `## 🛡️ Adversarial Security Review`
14. `## 🧰 Project-Owned Fixable Findings`
15. `## 📚 Protected Vendor Findings`
16. `## 🧩 Missing Evidence Or Missing Models`
17. `## ✅ Recommended Actions`
18. `## 📝 Execution Notes`

## ✅ Section Requirements

### `## 🎯 Executive Verdict`

Must include:

- overall status
- scope
- mode
- selected levels
- run-root path
- total gate count
- total profiling event count
- tracked-file coverage count when available
- blocking counts split into `FAIL`, `UNKNOWN`, `NEEDS_HUMAN_CRYPTO_REVIEW`, `SKIPPED`
- crate-unmapped count for `project` scope

### `## 📦 Evidence Provenance`

Must include:

- generated UTC timestamp
- run timestamp token
- report format path
- run-root-local cache root
- run-root-local cargo home and cargo install root
- tmp/specs/verification/fuzz/runtime roots
- release profile args
- python bytecode write policy
- prior stale run cleanup count
- external interferer cleanup count
- links to the core JSON/TSV evidence files

### `## 🚦 Gate Matrix`

Must be a table with:

- gate id
- checker module
- status
- elapsed seconds
- log path
- primary artifacts

### `## 🧪 Conclusion Ledger`

Must be a table with:

- gate id
- checker module
- machine conclusion
- validity ceiling
- anchoring artifact

### `## 🔍 Validity And Doublecheck`

Must state:

- no claim stronger than underlying tool evidence is allowed
- `UNKNOWN` means missing closure, not soft success
- `NEEDS_HUMAN_CRYPTO_REVIEW` means expert review is still open
- artifact confinement result
- whether any `repo/.cache` production-cache manifests were observed under `interference/`
- tool-specific validity ceilings surfaced from logs when present, such as Kani unsupported-construct / concurrency reductions or Miri optimization caveats
- exact doublecheck inputs used to produce the report

### `## 🏗️ Bootstrap Artifact Provenance`

Must distinguish:

- report-only mode with no repo-owned edits
- generated vs refreshed vs skipped artifacts
- runtime bootstrap outputs versus pre-existing manual artifacts

### `## 📊 Performance And Resource Profiling`

Must include:

- tool availability inventory with path and version when available
- profiling events path and summary path
- resource profile directory and summary path
- run-footprint summary path
- total event counts
- slowest top `5%` of events
- top CPU / memory / filesystem-I/O gate consumers when resource profiles exist
- run-root disk footprint summary
- concise acceleration recommendations derived from measured artifacts

If Mermaid is emitted, it may only visualize measured data from this run.

### `## 🌲 HJMT Runtime Evidence`

Must include only run-root HJMT artifacts.

When HJMT artifacts exist, include:

- primary metrics artifact path
- primary proof-size artifact path
- cache hit/miss totals and hit ratio
- root reuse ratio
- proof-segment reuse ratio
- scheduler queue depth, max active workers, backpressure count, reject/cancel counts
- proof example count
- proof-size min/median/max
- verify-time min/median/max
- slowest examples and largest examples

When no throughput artifact exists in the run root, the report must say that TPS was not measured in this run. Do not infer TPS from proof-size or one-shot verify-time samples.

### `## 🗺️ Coverage Inventory`

Required for `project` scope only.

Must include:

- tracked-file count
- manifest path
- summary path
- status counts
- crate-unmapped count

### `## 🚨 Risk Register`

Must summarize the highest-signal blocking or security items in a table with:

- class
- source module or artifact
- severity
- rationale
- anchor path

Severity must be labeled as orchestrator triage severity, not exploit proof severity.

### `## 🔗 Supply-Chain Highlights`

Must use run-local supply-chain artifacts and distinguish:

- project-owned advisories
- protected vendor advisories
- mixed advisories

### `## 🛡️ Adversarial Security Review`

Must include:

- scanned code-file count
- scanned `.github` prompt corpus count
- findings split by severity and class
- ownership split
- direct artifact paths to the adversarial JSON and Markdown outputs
- top hypotheses with exact evidence file anchors

### `## 🧰 Project-Owned Fixable Findings`

Must list only project-owned items that can be acted on locally.

### `## 📚 Protected Vendor Findings`

Must include:

- vendor evidence count
- vendor report path
- explicit policy statement that protected vendor code is report-only

### `## 🧩 Missing Evidence Or Missing Models`

Must list every `UNKNOWN` and `SKIPPED` gate with anchor logs.

### `## ✅ Recommended Actions`

Must be ordered from blocking assurance issues to optimization follow-ups.

Recommendations must be tied to concrete artifacts or measured bottlenecks.

### `## 📝 Execution Notes`

Must state whether the run edited code or was report-only.

## 📈 Allowed Visuals

The report may embed:

- Mermaid pie chart for gate status distribution
- Mermaid xychart for slowest measured events
- Mermaid xychart for top memory or disk consumers

Do not fabricate diagrams when the underlying data is absent.

## 🚫 Prohibited Claims

The report must not:

- claim proof from a plain `PASS`
- claim TPS without a run-root throughput artifact
- turn one-shot note timings into benchmark authority
- mix vendor findings into project-owned remediation counts
- cite repository-root runtime leaks as acceptable output
- cite artifacts outside the active run root as current-run evidence
