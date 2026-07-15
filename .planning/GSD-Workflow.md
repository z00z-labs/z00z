# GSH Workflow

[TOC]

---------------------

## ^H Find-Replace

069-Recursive-Proof

069

---

## ⬛️ 0. z00z-chat-init

```
/z00z-chat-init
```

## 🟥 1. gsd-add-phase

```
/gsd-add-phase `069-Recursive-Proof`
etot folder uzhe suwestvuet; novij ne sozdavat; rabotat s nim
регистрирую egoкак Phase 069 в ROADMAP и STATE, без создания нового каталога.
```

## 🟥 2. gsd-discuss-phase

```
/gsd-discuss-phase 069
Goal:
-----
Understand the problems and issues described in 069-TODO.md and provide comprehensive answers and solutions.
----
069-TODO.md is the canonical planning inventory for Phase 069.
The planner must cover all canonical Phase 069 tasks.
Planning must proceed sequentially, one canonical task after another.

069-TODO.md describes the specific tasks that must be scheduled for execution. You are forbidden from changing the task titles or their wording. Create a simple 069-CONTEXT.md describing this situation.

MANDATORY: Do not duplicate the existing codebase or its logic. Do not introduce a parallel layer. Prevent codebase concept drift.

During the planning phase, you will need to schedule the specific execution of these 069-TODO.md tasks, one after the another.

None of the tasks in the table can be excluded. Only in extreme cases where it is impossible to bypass a principle blocker will it be necessary to record this in the final report.
```

```
/gsd-ai-integration-phase 069
```

## 🟥 3. gsd-research-phase <mark>NOT IN USE</mark>

```
/gsd-research-phase 069
goal: ponjat problemi i voprosi opisannie v 069-TODO.md i dat' na nih polnocennij otvet i  reshenija
```

## 🟥 4. GSD-Review-Context <mark>x3 times</mark>

```
/GSD-Review-Context
current_context = 069-CONTEXT.md + 069-TODO.md
review_goal = 
-----
Verify that everything mentioned in 069-TODO.md has been transferred to the context for implementing 069-CONTEXT.md.
-----
Create a table in the 069-CONTEXT.md file confirming the transfer of each task from the 069-TODO.md that needs to be completed/checked/confirmed, etc.

Verify that all suggestions and issues of 069-TODO.md are included in context and in 069-CONTEXT.md. I need to verify everything 100% before implementation

MANDATORY: Do not duplicate the existing codebase or its logic. Do not introduce a parallel layer. Prevent codebase concept drift.

Run the second `doublecheck` against 069-TODO.md to confirm that all issues are in 069-CONTEXT.md.
```

---------------------

## 🟨 5. gsd-plan-phase

```markdown
/gsd-plan-phase 069
--skip-research
--prd 069-TODO.md
--text """
Create complete executable GSD plans for Phase 069.

MUST read first:
1. 069-TODO.md
2. 069-CONTEXT.md if it exists
3. Every Markdown source linked from 069-TODO.md task rows.
4. Relevant current code anchors listed in 069-TODO.md.

MUST treat 069-TODO.md as normative, not advisory.
MUST NOT drop, merge away, rename, renumber, or silently reinterpret any TASK-NNN.
MUST NOT create future/deferred/best-effort work for local correctness.
MUST close every local blocker by code/tests or by local deterministic simulation using real project primitives.

Before writing plans, perform a coverage audit:
- Count unique TASK-NNN from 069-TODO.md: 
- Count Required GSD Plan Groups: 
- Build a task-to-plan coverage table.
- Every TASK-NNN MUST map to exactly one grouped 069-NN-PLAN, unless explicitly split into 069-NN-PLAN fallback.
- Every plan MUST include every source ref from each included task row.
- Any missing/duplicate task MUST fail planning.

For each generated plan, MUST include:
- plan_id
- task_ids
- copied task_rows
- source_refs
- inputs
- outputs
- dependencies
- acceptance_tests
- simulation_gate
- negative_tests
- plan_artifacts
- plan_tests
- plan_results
- task_artifacts
- task_tests
- task_results
- anti_placeholder_gate
- current_code_refs
- blockers
- evidence_gate
- not_recommendation_gate

For each included TASK-NNN, MUST include its own:
- artifacts: exact files/APIs/configs/docs/simulator outputs to create or modify
- tests: exact commands, test modules, scenarios, positive and negative cases
- results: expected proof artifacts, pass conditions, and anti-placeholder evidence
- implementation_depth: one of full, simulated-full, live-claim-removed

MUST NOT close any task with placeholder/scaffold/TODO-only/panic-only/string-only/no-op implementation.
MUST NOT accept compile-only proof for runtime behavior.
MUST NOT accept docs-only proof for code behavior.
MUST prove every implementation through real project primitives.

For local simulation:
- Fake only external transport, remote process boundary, external DA transport, wall-clock/fault scheduler, or unavailable third-party network.
- MUST use real cryptography, package verification, planner output, route tables, HJMT journal entries, storage commit/recovery paths, wallet history, fee policy, publication bindings, validator/watcher checks, and per-component state.
- Distributed HJMT MUST include local simulation for replication, quorum, conflict resolution, standby catch-up, route rollout, dispatch, membership, restart, partition/heal, stale lineage, divergent roots, and failure telemetry.

In each <verify> section of every <task type=\"auto\">:
1. MUST run:
   ./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
   If it fails, stop, fix issues, rerun it, and only then continue.

2. SHOULD run when Rust/test-affecting changes are relevant:
   cargo test --release

3. MUST run /.github/prompts/gsd-review-tasks-execution.prompt.md (/GSD-Review-Tasks-Execution) at least 3 times in YOLO mode.
   MUST fix all issues and warnings.
   MUST continue review/fix cycles until at least 2 consecutive runs show no significant code issues.

4. If committing changes, MUST use /z00z-git-versioning.

5. MUST use nested skills/prompts/instructions from .github/ as needed, especially verification, doublecheck, spec-to-code compliance, smart tests, and Z00Z gates.

Final generated PLAN output MUST contain a Coverage Appendix:
- TASK-NNN
- PLAN id
- source refs
- inputs
- artifacts
- tests
- expected results
- simulation requirement
- anti-placeholder proof
- status

Planning MUST fail if any TASK-NNN lacks inputs, outputs, artifacts, tests, results, acceptance, or evidence gates.
"""
```



## 🟨 6. GSD-Review-Plan  ==x3 times== PLAN

```
/GSD-Review-Plan
current_plan = {069-*-PLAN.md}, where `*-PLAN.md` - is a naming pattern
review_goal = I need a 100% guarantee that every bullet from 069-TODO.md i references docs corpus is reflected in 069-CONTEXT.md and in 069-{01..N}-PLAN.md
If there are issues that need to be corrected or supplemented, do it in YOLO mode

Verify that all suggestions and issues of 069-TODO.md are included in context and in the plan. I need to verify everything 100% before implementation

MANDATORY: Do not duplicate the existing codebase or its logic. Do not introduce a parallel layer. Prevent codebase concept drift

Run the second `doublecheck` against 069-TODO.md to confirm that all issues are in plans.
```

---

## 🟦 7.1 gsd-add-tests ==x3 times==

```
/gsd-add-tests  069
goal = based on files 069-Recursive-Proof
define smart, complete, end-to-end integration tests and realistic examples that prove what the workflow does, how it behaves, which invariants it must preserve, and which failures it must reject.
подготовить phase-local E2E/unit test specification document для 069 на основе 069-CONTEXT.md, 069-TODO.md и всех 069-*-PLAN.md как planning artifact.
The result must be directly usable by another engineer or agent to implement E2E coverage without guessing scenario boundaries, success criteria, or test anchors. Derive the critical user journeys, state transitions, proof paths, and failure paths that must be verified end to end.

The specification must explicitly address all of the following when relevant:
  - which end-to-end behaviors must be proven;
  - which integration paths are critical;
  - which examples are needed to demonstrate successful execution;
  - which negative scenarios must prove rejection or failure handling;
  - which cryptographic invariants, soundness, proofs, commitments, roots, or signatures must be observed;
  - which assertions prove correctness;
  - what each test or example is meant to demonstrate.
   - what measurable success or failure conditions make the scenario pass."""
```

## 🟦 7.2 create-tests ==x3 times==

```
/create-tests 069
```

---

## 🟦 8. GSD-Review-Plan ==x3 times== TESTS

```
/GSD-Review-Plan
current_plan = {069-*-PLAN.md}, where `*-PLAN.md` - is a naming pattern + 069-TEST-SPEC.md + 069-TESTS-TASKS.md
review_goal = I need a 100% guarantee that every bullet from 069-TODO.md is reflected in 069-CONTEXT.md and in 069-{01..N}-PLAN.md
If there are issues that need to be corrected or supplemented, do it in YOLO mode

Verify that all suggestions and issues of 069-TODO.md are included in context and in the plan. I need to verify everything 100% before implementation

Run the second `doublecheck` against 069-TODO.md to confirm that all issues are in plans.
```

---------------------

## 🟩 9. gsd-executor PLAN

```markdown
/gsd-execute-phase 069 continue

Update STATE #sym:Status and ROADMAP

все такие future-only design terms становятся live scope и referenced docs как phase authority. I continue to use design/whitepapers as a source of requirements, not as a "to-be-in-the-future" status. 069-TODO.md explicitly states target/future design statement is now a mandatory scope of the live code.

Make sure there are no missing or missing strings, so there will be one canonical path
for all module structures and functions

MUST pay special attention to following instructions:
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` first as a mandatory fail-fast gate.
- If it fails, stop, fix the issues, and rerun it before any broader validation.
- Treat `bootstrap_tests.sh` as the early regression detector, not as a replacement for the broader cargo test command.

2. Then, when relevant, run: `cargo test --release` `cargo --release`; vse zapuski tolko v release mode

3. In each <verify> section, include a requirement to run /.github/prompts/gsd-review-tasks-execution.prompt.md (`/GSD-Review-Tasks-Execution`) at least 3 times in YOLO mode and fix all issues and warnings. Stop running gsd-review-tasks-execution only after at least 2 consecutive runs show no significant issues in the code.

4. After completing plan N: (N)-PLAN.md, moves to plan (N+1)-PLAN.md in YOLO mode

5. When you need to commit changes in GIT, use skill `/z00z-git-versioning`

6. All necessary nested skills, prompts, and instructions are in `.github/`, use them as needed.
```

## 🟩 10. gsd-executor  TESTS

```markdown
/gsd-execute-phase 069
implement 069-TEST-SPEC.md and 069-TESTS-TASKS.md

MUST pay special attention to following instructions:
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` first as a mandatory fail-fast gate.
- If it fails, stop, fix the issues, and rerun it before any broader validation.
- Treat `bootstrap_tests.sh` as the early regression detector, not as a replacement for the broader cargo test command.

2. When relevant, run: `cargo test --release`

3. In each <verify> section, include a requirement to run /.github/prompts/gsd-review-tasks-execution.prompt.md (`/GSD-Review-Tasks-Execution`) at least 3 times in YOLO mode and fix all issues and warnings. Stop running gsd-review-tasks-execution only after at least 2 consecutive runs show no significant issues in the code.

4. When you need to commit changes in GIT, use skill `/z00z-git-versioning`

5. All necessary nested skills, prompts, and instructions are in `.github/`, use them as needed.
```

---

## 🟪 11. GSD-Finalization

```
/gsd-secure-phase 069
```

```
/gsd-validate-phase 069
```

```
/gsd-verify-work 069
```

```
/gsd-eval-review 069
```

## 🟪 13. GSD-Review-Tasks-Execution ==x3 times== PLAN

```
/GSD-Review-Tasks-Execution 
current_spec = {069-*-PLAN.md}
current_task = `*-PLAN.md` - is a naming pattern
```

## 🟪 14. GSD-Review-Tasks-Execution ==x3 times== TESTS

```m
/GSD-Review-Tasks-Execution 
current_spec = 069-TEST-SPEC.md + 069-TESTS-TASKS.md
current_task = `*-PLAN.md` - is a naming pattern + 069-TEST-SPEC.md + 069-TESTS-TASKS.md
```

---------------------

## 🟪 15. doublecheck ==x3 times==

```
/doublecheck
Verify that ALL task plans in 069-TODO.md are fully implemented. Check the code independently of the summaries and give your conclusion.
1. Quality of implementation
2. Correctness of implementation
3. Check for logical errors and incomplete implementation features
4. Check for missing scafolds or placeholders instead of the planned full implementation
5. Check thoroughly

Run all --release mode if needed
Fix all in YOLO mode
```

## 🟪 16. GSD-Audit-4  ==x3 times==

```markdown
/GSD-Audit-4 
phase_dir = 069-Recursive-Proof
```

---

## 📉 17. Alert-Concept-Drift

```markdown
/alert-concept-drift current codebase vs git branch `main` feat(v2.175.0)
create 069-CONCEPT-DRIFT-REPORT.md
```

---

## ❓ 18. phase-exam-create

```markdown
/phase-exam-create
phase_dir = 069-Recursive-Proof
```

## ✔️ 19. phase-exam-solve

```markdown
/phase-exam-solve
phase_dir = 069-Recursive-Proof
```

---

##  ✅ 20. z00z-verification-orchestrator

```
/z00z-verification-orchestrator find-and-fix project
```

```markdown
/z00z-design-foundation-compliance   fix all issues in yolo mode
```
---

## 🔔 21. attack-surfaces-create

```markdown
/attack-surfaces-create
scope = 069-TODO.md + 069-verdict.md
report_path = 069-Recursive-Proof
db_path = 069-Recursive-Proof-db.jsonl
max_variants = 20
```

## 👍 22. attack-surfaces-resolve

```markdown
/attack-surfaces-resolve
db_path = 069-Recursive-Proof-db.jsonl
surface_id = [id1,id2,...]
out_spec = 069-Recursive-Proof-report.md
```


---

## ✅ 23. z00z-full-verify-gate

```
/z00z-full-verify-gate  max-safe;  fix all YOLO mode
```

## 🟧 24. z00z-git-versioning

```
/z00z-git-versioning  minor-commit stage-all
```

---

## 🔰 25. z00z-git-merge to main

```
/z00z-git-merge merge `main` to the current release commit on `z00z-dev`:  z00z-dev --> main; The local main, origin/main, and release tag must all point to the same commit.
If they don't match, correct, push, verify hash, and verify to z00z-dev.
Don't just compare, achieve this state.
```

---

## 🪣 25. CLEAN-UP

```
./scripts/z00z_cleanup.sh --yes
```

-------------------------

# 🌀 Crypto-Architect FUSION

## 🔷 crypto-architect

```
/crypto-architect sozdaj glubokij otchet `.planning/phases/069-Strix/storage-audit-sonet46.md` konkretno po crate `z00z_wallets/` izuchi tolko *.rs implementaciju i ne trogaj drugie dokumenti (ne vkluchaj tari/ vendor)
```

-------------------------

## 🔷 smart-docs-fusion

```
/smart-docs-fusion [ ] --> .planning/phases/069-Strix/FUSION.md
```



---

# 👀 Comprehensive Crypto Audit

## 🧱 crypto-architect

```
/crypto-architect sozdaj glubokij otchet reception-spec.md

Read 6-Output-Reception.md; the document reflects possibly correct ideas, but with obsolete symbols and verified architectural patterns and unverified cryptography relative to the current codebase. You need to conduct a thorough, comprehensive audit and create a new `reception-spec.md`, which will be cryptographically correct and linked to the current codebase.
Check your position using the `doublecheck` skill; thoroughly check the compatibility of all new cryptography with the current codebase; v kriticheskih dlja ponimanija mestah davaj govie code-snippets;

ispolzuj alerts:
> [!NOTE]  
> Highlights information that users should take into account, even when skimming.
> [!TIP]
> Optional information to help a user be more successful.
> [!IMPORTANT]  
> Crucial information necessary for users to succeed.
> [!WARNING]  
> Critical content demanding immediate user attention due to potential risks.
> [!CAUTION]
> Negative potential consequences of an action.

Derive the critical user journeys, state transitions, proof paths, and failure paths that must be verified end to end.

Categorise by Severity
🔴 CRITICAL
🟠 HIGH
🟡 MEDIUM
🔵 LOW
⚪ INFO

The specification must explicitly address all of the following when relevant:
 - which end-to-end behaviors must be proven;
 - which integration paths are critical;
 - which examples are needed to demonstrate successful execution;
 - which negative scenarios must prove rejection or failure handling;
 - which cryptographic invariants, soundness, proofs, commitments, roots, or signatures must be observed;
 - which assertions prove correctness;
 - what measurable success or failure conditions make the scenario pass.
```

# 👀 Security Audit

## ⚡️ security-audit

```
/security-audit  sozdaj glubokij otchet crates/z00z_wallets

ispolzuj alerts:
> [!NOTE]  
> Highlights information that users should take into account, even when skimming.
> [!TIP]
> Optional information to help a user be more successful.
> [!IMPORTANT]  
> Crucial information necessary for users to succeed.
> [!WARNING]  
> Critical content demanding immediate user attention due to potential risks.
> [!CAUTION]
> Negative potential consequences of an action.

Derive the critical user journeys, state transitions, proof paths, and failure paths that must be verified end to end.
Findings Summary

Categorise by Severity
🔴 CRITICAL
🟠 HIGH
🟡 MEDIUM
🔵 LOW
⚪ INFO

The specification must explicitly address all of the following when relevant:
 - which end-to-end behaviors must be proven;
 - which integration paths are critical;
 - which examples are needed to demonstrate successful execution;
 - which negative scenarios must prove rejection or failure handling;
 - which assertions prove correctness;
 - what measurable success or failure conditions make the scenario pass.
```

# GSD-Upgrade-Version

```
/GSD-Upgrade-Version target_ref=v1.38.3 source_repo=https://github.com/gsd-build/get-shit-done runtime=--copilot
```

---
## Docker  z00z-verification-orchestrator

```
1. zapusti  `./pack_z00z_project.sh

2. zapusti  `./unpack_z00z_project.sh --archive ./z00z-<pack-date>.tar.gz --docker-sandbox` do konca, ne obrivaj nichego ; ves output terminala sohrani v log;
fix all log errors and warnings; 
docker kontainer ne stiraj chtob v nem zapuriti sleduwij step

3. zapusti v dokere "/z00z-verification-orchestrator report project" i copy folder s etim reportom v /home/vadim/Projects/z00z/reports/z00z-verification-orchestrator-<timestamp>
```



-------------------------

# 🍁 Emoji

➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
https://copychar.cc/
https://getemoji.com/
https://emojidb.org/pointer-emojis 
https://emojidb.org

⚠️ 🔰 ⭕️ ❗️ ❓ ❌ ⛔️ ✅❇️❎ ✔️ ☑️ 🔘 🔴 🟠 🟡 🟢 🔵 🟣 ⚫ 🟥 🟧  🟨 🟩 🟦 🟪 ⬛️ ⬜️ 
➔  ➤  ⌘  ⊚ ★ ✦ ✴ ✻ ➡️ 0️⃣ 1️⃣ 2️⃣ 3️⃣ 4️⃣ 5️⃣ 6️⃣ 7️⃣ 8️⃣ 9️⃣ 🔟 👍 👎 🟰 ➖ 💲 ☢️ ⚡️ 📈 📉  📌 📍 🍁

╰┈➤ ↪️ ↩️ 👀 ✍🏼 ⌚ 🔔 ⏰ 📞 ⭐ 🌟 🪣🗑
🐛🐞🪲 🪰🦋┈┈➤ 🔷
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖

```
FOLDING: ^K^2  ^K^J
```



Use alerts:

> [!NOTE]  
> Highlights information that users should take into account, even when skimming.

> [!TIP]
> Optional information to help a user be more successful.

> [!IMPORTANT]  
> Crucial information necessary for users to succeed.

> [!WARNING]  
> Critical content demanding immediate user attention due to potential risks.

> [!CAUTION]
> Negative potential consequences of an action.
