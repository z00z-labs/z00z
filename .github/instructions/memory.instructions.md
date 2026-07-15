---
description: 'Workspace memory for recurring z00z-specific environment and workflow facts.'
applyTo: '**'
---

# Workspace Memory

Persistent repository-specific facts that should be treated as baseline context.

## Z00Z Local Python

- The `z00z` repository has a local virtual environment at `.venv/`.
- For Python checks and Python-based helper commands in this repository, check `.venv/bin/python` and `.venv/bin/python3` before concluding that Python is unavailable.
- Prefer `.venv/bin/python` as the canonical repo-local interpreter when a Python command is needed here.

## Autonomous GSD Execution

- Treat `/gsd-execute-phase <phase> continue` as one continuous YOLO execution
  session: load the active plan, complete its tasks and required evidence,
  write the honest summary, then advance to the next dependency-eligible plan
  without waiting for a repeated user prompt.
- A long-running validation command is a monitoring state, not a stop point.
  Preserve its process, capture its exit result, and resume the same plan
  immediately afterward. Do not restart bootstrap merely because the user
  repeats `continue` while the current execution wave is still active.
- Report intermediate status only as commentary while execution continues.
  Send a final response only after the requested execution wave is completed or
  a factual, evidence-backed blocker leaves no authorized local implementation
  path.
- Do not close a plan or write a completion summary from compile-only, native
  precheck, reduced-profile, placeholder, or partial-circuit evidence. Keep
  implementing the active task until its plan acceptance and verification gates
  are satisfied; then create the required honest SUMMARY and proceed in order.
