# YOLO Remediation Mode

Use this branch only when the user explicitly requests YOLO remediation.

## Eligibility

Apply a fix only when all conditions hold:

- the finding survived the mandatory `doublecheck` gate
- the finding is a confirmed issue with High confidence
- the safe behavior is explicit in the surrounding code or framework contract
- the change can be narrow and preserve public behavior and trust boundaries
- relevant validation can run after the edit

Typical eligible changes include:

- replace a clearly hardcoded secret with an existing configuration or secret-store path
- replace string-built SQL with the project's parameterized-query pattern
- add a missing local validation or authorization check whose intended boundary is explicit
- enable an established secure default without changing product policy

## Ineligible Changes

Do not auto-apply:

- live credential rotation or replacement secret selection
- authentication, authorization, retention, or deployment policy decisions
- broad cross-module redesigns
- fixes with unresolved compatibility or exploitability assumptions
- changes that require production access or third-party coordination

Report these items as unresolved instead of guessing.

## Execution

1. Apply the smallest change that fully closes the verified path.
2. Preserve project style, existing abstractions, and public contracts.
3. Add or update a regression test that proves the exploit path is closed when feasible.
4. Run the narrowest reliable validation for the edited scope.
5. Re-run the final `doublecheck` closeout over applied fixes and residual findings.
6. Do not claim success if validation or final verification fails.
