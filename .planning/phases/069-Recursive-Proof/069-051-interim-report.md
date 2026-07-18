# Phase 069-051 Interim Implementation Audit

**Audit date:** 2026-07-16  
**Scope:** complete Phase 069 planning corpus, historical Plans 01–05, corrective Plan 051 T0–T4, current T2 implementation, and downstream Plans 06–13  
**Primary question:** whether Plan 051 is being implemented consistently and whether T2 can safely proceed or close  
**Overall verdict:** **RISKY BUT SALVAGEABLE — BLOCKED AT 069-051-T2**

## 🚨 Executive Decision

Plan 051 must not be declared implemented and Plan 06 must not start. The current
work is useful, substantial, and mostly moving toward the intended architecture,
but it does not yet prove the Plan 051 theorem.

There are three independent blockers:

1. **Intrinsic theorem blocker:** the Nova step currently constrains the control
   machine, canonical source/global SHA paths, replay payload parsing, and the
   uniqueness precommit/challenge prefix, but it does not constrain net-effect,
   complete uniqueness/permutation, JMT, hierarchy, typed commitments, statement,
   or final application fields. The reserved state ranges for these families are
   not semantic implementations.
2. **T1/T2 boundary blocker:** the native relation and statement still do not
   explicitly bind the first definition-tree `old_root` to the immutable
   pre-state definition root. A self-consistent JMT transition ending at the real
   post-state is not sufficient to prove that it started from the selected
   pre-state. The statement also remains materially smaller than the frozen
   Section 13.1/T2 binding contract.
3. **External and process blocker:** no authority-pinned operating-budget tuple
   exists, so T2 action 11 cannot select a candidate. At the same time, expensive
   real-Nova tests are being rerun during intermediate edits, and the current
   in-process mutex does not serialize separate bootstrap processes. Two full
   bootstraps and concurrent high-RSS Nova workers were observed in this audit.

The correct response is not another closure review or another unconditional full
bootstrap. Reopen the narrow T1 theorem boundary, complete the missing T2
relations under fast semantic tests, install a repository-wide single-flight
gate for heavy Nova evidence, obtain the authority budget, and only then measure
the final circuit and run closure verification.

No S0 finding was established. Five S1 findings block acceptance.

## 🧭 Audit Basis And Limitations

The complete Phase 069 document corpus was read before source-code inspection,
as required. This included:

- `069-BASELINE.md`, `069-CONTEXT.md`, `069-COVERAGE.md`,
  `069-COVERAGE-AUDIT.py`, `069-FEASIBILITY.md`, `069-PATTERNS.md`,
  `069-PREDICATE.md`, `069-STOP-SPLIT.md`, `069-TEST-SPEC.md`,
  `069-TESTS-TASKS.md`, and all 5,289 lines of `069-TODO.md`;
- Plans and summaries 01–05, including Plan 05 matrix/profile/stop-split;
- Plan 051, both crypto audits, the T0 authority record, the V1-eradication
  manifest, and the ZIP evidence packet;
- downstream Plans 06–13.

Plans 01–05 were treated as historical context and were not changed. The current
normative overlay is Plan 051 plus the current `069-TODO.md` authority. The phrase
“06..14” is interpreted as the remaining packet sequence: there is no
`069-14-PLAN.md` in the directory. The repository currently contains Plans
01–13 plus the corrective 051 overlay.

This is an interim audit of a dirty, concurrently changing worktree. The source
snapshot used for the decisive code findings was:

| Item | SHA-256 at audit snapshot |
| --- | --- |
| `069-051-PLAN.md` | `b6fae8fe6c0e858bce36444bbc6d22041cb9e73b563cde2526c82d23111370ff` |
| `069-051-T2-CRYPTO-AUDIT-2.md` | `7638cbf46e7410b8627e9d734682a11fd185ddb4bb68f9d8b225f38cfc18751f` |
| `recursive_v2/nova.rs` | `97e85108ccc6070e1557156b6a567a3998d39629f9e427f7b9b043b913f40c62` |
| `recursive_predicate.rs` | `417e9a003a43df66f8b79a7ca70840d960f78fc4c8d51c829a093659e7b6b893` |
| `recursive_statement.rs` | `e899355c6ecdba0d02f75c02f682cd7bdd8812153e10cab7aefc37b002f28304` |
| `settlement/proof_batch.rs` | `7f98491ae8a4c83ed444d33c93ebf711369addbe9e3d601dabcf63591cbad0dc` |

No source or existing plan was edited by this audit. Existing concurrent test
processes were not interrupted. Because heavy Nova workers were already running,
this audit did not start a third setup/proof process. Observed processes are
process evidence, not retained pass evidence; their terminal results were not
available to this audit.
## 📊 Historical Implementation Status At Audit Snapshot

This table records the source snapshot identified above. The live reconciliation
near the end of this report supersedes its code-presence dispositions; historical
findings and measurements remain evidence of the state that was audited.

| Task | Spec-to-code status | Audit disposition |
| --- | --- | --- |
| T0 authority/V1 eradication | `full_match` with evidence-maintenance caveats | The scoped recursive V1 symbol scan is clean, the authority branch is explicit, and live code has one V2 path. Do not reopen Plans 01–05. |
| T1 native V2 relation | `partial_match` | Resolver, durable cutover, bounded source, real HJMT trace, strict decoding, hierarchy checks, and typed statement exist. Pre-state backend-root anchoring, complete statement layering, and resource-consistent streaming remain incomplete. T1 must be reopened narrowly rather than erased. |
| T2 uniform Nova relation | `partial_match` / `missing_in_code` for required families | SHA/source/global/replay, exact count/P/U/challenge/list SHA, full semantic-row uniqueness products/order, semantic Delete/Insert/Replace/Unchanged Net, and final per-opcode accounting are real. JMT proof algebra, hierarchy, roots, commitments, `X_h`/prior-IVC/application finals, Models A/B/C, full mutation ledger, final artifacts, and candidate selection remain absent. |
| T3 continuous runner | `not_started` as required | Correctly locked. Private proof-envelope code in T2 is preparatory and is not a T3 runner. |
| T4 closure/migration/theorem matrix | `not_started` as required | Correctly locked. Current coverage pass is planning coverage, not T4 theorem-to-code closure. |
| Plans 06–13 | `blocked_by_dependency` | Must remain locked until all T0–T4 acceptance gates pass. Several downstream path/owner references need reconciliation before execution. |

## 🔎 Findings

Severity follows the crypto-audit convention: S0 critical, S1 high/blocking,
S2 significant, S3 hardening/maintenance, S4 informational.

### 🔴 S1-01 — The current real Nova proof proves only a prefix of the required theorem

**Classification:** `missing_in_code`  
**Confidence:** high  
**Plan obligations:** T2 actions 3–8, 14–17; `I_unique`, `I_jmt`,
`I_hierarchy`, `I_root`, `I_count`, and `I_done`

`CheckpointRunningStateV2` reserves net, JMT, hierarchy, commitments, and
expected-final ranges. The source itself correctly says that unconnected
families are T2 blockers. In `StepCircuit::synthesize`, the implemented state
updates cover control, source/global byte/SHA contexts, replay payload,
precommit/challenge payload, and trace closure. Searches for `NET_START`,
`JMT_START`, `HIERARCHY_START`, and `COMMITMENTS_START` find no semantic
synthesis functions. The relevant cells are range-constrained or classified as
digest/transient cells and otherwise carried from `z` until final zero checks.

Consequences:

- `NetMerge`, `JmtUpdate`, `PromoteChildRoot`, and `CommitTypedEvent` can follow
  legal opcode/trace/SHA structure without the circuit deriving their semantic
  effects.
- The current compressed proof path demonstrates a functioning pinned Nova
  backend and the implemented prefix, not a complete checkpoint transition.
- The measured shape, PP/VK sizes, RSS, and latency necessarily change when the
  missing relations are added.

**Required correction:** implement each family as an explicit successor
relation over the one canonical chunk stream. For every family, add a test that
changes its canonical payload, recomputes all outer source/hash commitments, and
still requires the R1CS and unchanged compressed verifier to reject. A checksum
or stale-envelope rejection is not sufficient.

### 🔴 S1-02 — JMT verification is bound to the post-state but not to the selected pre-state backend root

**Classification:** `partial_match` / theorem under-binding  
**Confidence:** high from dataflow; an executable exploit fixture is still required  
**Plan obligations:** T1 actions 11 and 13, T2 action 7, audit `I_jmt`,
Section 13.1 previous/next definition roots

The native evaluator receives a snapshot containing the pre-settlement root,
but not the explicit pre-definition root. It accumulates and decodes the JMT
envelope, verifies each internal update proof, and calls
`verify_hierarchy_semantics` with only
`store.recursive_v2_definition_root()`—the already committed post-state root.
The hierarchy verifier checks that the final definition update's `new_root`
equals that value. It does not accept or compare an expected top-level
`old_root`.

`RecursiveTransitionStatementV2` binds the pre-settlement root and the
post-definition root, but not the pre-definition root. No current Nova anchor
repairs this omission; the application root/JMT families are not implemented
there yet.

This permits two different logical statements to remain insufficiently
distinguished: “a valid JMT transition from some old tree to this post tree” and
“a valid JMT transition from the authority-selected pre tree to this post tree.”
Post-state observation cannot establish causation.

**Required correction:** capture the canonical pre-definition root in the
immutable pre-state capability before mutation. Require:

1. `derive_settlement_root_v2(generation, layout, policy, pre_definition_root)`
   equals the snapshot/checkpoint pre-settlement root;
2. every typed tree's first update starts from the current authority-selected
   tree root (canonical null only for the explicitly permitted first-materialize
   case);
3. same-tree updates chain `prior.new_root == next.old_root`;
4. parent promotions consume the exact child `new_root`;
5. the final definition `new_root` equals the reloaded post-definition root;
6. both previous and next backend roots are committed into the appropriate
   transition/public-input layer and constrained in Nova.

**Decisive falsification fixture:** construct two distinct pre-states that
converge to the same post-state by deleting/replacing the differing leaf. Swap
the valid JMT envelope from pre-state B into a trace/header/authority for
pre-state A, recompute all source hashes, and require rejection at the first
old-root gate in the evaluator, TestConstraintSystem, and real Nova verifier.

### 🔴 S1-03 — T1 is recorded complete even though T2's declared input and theorem boundary are not complete

**Classification:** `mismatch` between status, audit, and code  
**Confidence:** high

Plan 051 T2 declares “Completed T1 code/evidence” as an input. ROADMAP/STATE
record T1 complete. AUDIT-2, however, records DC2-F11–F24 as the material
correction set, and the live code still has at least the pre-state/root and
statement gaps above. There is no current resolution ledger mapping each
DC2-F11–F24 item to a code symbol, negative test, command, and retained
evidence.

The statement responsibility is also blurred:

- the audit's frozen dependency order distinguishes base transition result,
  checkpoint ID/link, recursive public input `X_h`, expected `z_h`, proof, and
  receipt;
- the live `RecursiveTransitionStatementV2` includes an already resolved
  checkpoint ID/link and profile/spec/grammar data;
- it omits previous backend root, verifier-bundle/parameter identities,
  per-class declared/consumed counts, prior finalized IVC state, and expected
  final fields required by the full recursive public input.

This object may be a valid intermediate commitment, but its exact layer and
security claim are not currently unambiguous.

**Required correction:** change the status to “T1 reopened for bounded theorem
closure” without discarding completed T1 work. Freeze and test an explicit DAG:

```text
authority + immutable pre-state + execution
  -> independently evaluated storage transition
  -> transition commitment
  -> checkpoint/link binding
  -> recursive public input X_h + parameter/bundle identity + prior IVC state
  -> expected z_h
  -> proof
  -> receipt
```

Give every digest-bearing Rust type exactly one row in that DAG. Reject any
constructor input that belongs to a later row. Close T1 only after a
DC2-F11–F24 resolution ledger has no untested S0/S1 row.

### 🔴 S1-04 — No authority operating budget exists, so T2 action 11 is externally non-closeable

**Classification:** `ambiguous_authority` / external blocker  
**Confidence:** certain; Plan 051 states this explicitly

The 24 GiB/900 s values are emergency worker safety limits, not accepted
operating budgets. The measured incomplete candidate has approximately:

- setup/proof peak RSS: `17,264,693,248 B`;
- verifier rerun peak RSS: `17,075,798,016 B`;
- verifier check time: `612.074 s`;
- VK: `273,174,184 B`;
- verifier bundle: `273,174,638 B`.

The amended S1-01 source-byte relation has newer non-acceptance evidence: its
fresh 40-step full bounded release worker completed at `16,829,259,776 B` peak
RSS in `343.966 s`. Setup was `39.179 s`; accumulator construction was
`85 ms`; the special first `prove_step` was below `1 ms`; the following 39
folds took `6.999 s` (`179.465 ms/fold`); compression-key setup took `6.281 s`;
compression took `148.280 s`; verifier bundle load/proof decode/check took
`126.381 s`, and envelope load/check took `2.609 s`. The compressed proof is
`53,368 B`; the portable envelope is `134,451 B` (`251 B` header + two
`40,416 B` public states + proof). Its augmented primary/secondary shapes are
`C=303,019/10,349` and `V=256,347/10,331`. This
clears the former emergency-cap failure, but the circuit is
still semantically incomplete and the worker ceiling is still not an operating
budget.

No authority tuple exists for setup, fold, compression, verifier load/check,
RSS, cancellation, VK distribution/fanout, or cadence. Therefore no candidate
can be selected even if all code is correct.

**Required authority input:** one versioned tuple, bound to authority/profile/
spec generation, containing at minimum:

- maximum setup, fold, compression, verifier-load, and verifier-check wall time;
- maximum peak RSS for each stage and maximum concurrent workers;
- PP, PK, VK, verifier-bundle, proof-envelope, recovery-snapshot, and retained
  artifact byte limits;
- cancellation deadline and hard-kill policy;
- validator/watcher distribution, fanout, cold-load, and restart budgets;
- the finite permitted compile-time SHA-width candidate set;
- an explicit decision path if no complete candidate passes.

Until supplied, semantic implementation can continue, but T2 acceptance,
candidate selection, authoritative PP/PK/VK generation, and global clean-review
claims cannot.

### 🔴 S1-05 — Heavy Nova tests are not single-flight across processes

**Classification:** process defect / resource-safety evidence defect  
**Confidence:** high and directly observed

`real_nova_proof_lock()` is a `OnceLock<Mutex<()>>`. It serializes heavy tests
only inside one Rust test process. It cannot serialize two Cargo invocations or
two bootstrap agents. `bootstrap_tests.sh` unconditionally runs the entire
`z00z_storage --lib` suite, which includes the heavy real-Nova tests.

During this audit:

- two independent `bootstrap_tests.sh` processes were live;
- two bounded real-Nova workers were observed concurrently at `15,360,760 KiB`
  (about 14.65 GiB) and `6,311,804 KiB` (about 6.02 GiB) RSS at the observation
  point;
- both workers shared the host and workspace while producing timing/RSS
  evidence.

This invalidates the assumption that the test-only mutex prevents a multi-setup
RAM race. It can cause host pressure, timeouts, misleading peak/latency numbers,
and repeated “stalls.” It also explains why running the full bootstrap after
every small circuit edit can consume most of a day without advancing the
missing semantic relation.

**Required correction:** install a repository-wide, cross-process single-flight
gate before any PP/setup/prove/compress/verify worker starts. The gate must:

- use an OS-visible exclusive lock with owner PID, start time, source digest,
  command, and stale-owner recovery;
- be acquired by the parent only; the PID-pinned child must inherit/bypass the
  same ownership marker without deadlocking;
- reject or queue a second bootstrap/heavy-test invocation;
- record host contention separately from worker RSS/time;
- ensure one source revision cannot reuse evidence from another.

Add an integration test that launches two heavy-worker parents and proves only
one enters setup. A Rust `Mutex` test is not evidence for this property.

### 🟠 S2-01 — Resource measurements are being repeated before the final relation exists

**Classification:** process/order pitfall  
**Confidence:** high

Plan 051 correctly labels the current numbers diagnostic. Nevertheless, the
ROADMAP contains a long sequence of re-pinned shape and bootstrap updates for
intermediate source/SHA/replay slices. Every missing semantic family will alter
the augmented shape and can alter the generator count, PP/VK size, RSS, and
time discontinuously at the next power-of-two boundary.

**Correction:** use cheap shape synthesis and direct R1CS mutation tests during
the inner loop. Run one retained real-Nova smoke only at a named semantic
milestone. Run the finite candidate matrix and acceptance measurement only
after every theorem family and mutation row is present. Never optimize or
select `k` against the incomplete relation.

### 🟠 S2-02 — Required T2 artifacts and mutation evidence are not yet materialized

**Classification:** `missing_in_code` / `missing_evidence`

The plan names dedicated Nova step/adversarial integration tests, a theorem-code
matrix, benchmark report, mutation ledger, bounded-worker reports, and retained
proof fixtures. Current Nova coverage is extensive but concentrated in the
private `nova.rs` unit-test module. The named integration files and complete
mutation ledger are absent. Plan prose and ROADMAP updates are not substitutes
for source-bound command artifacts.

**Correction:** create artifacts only as their relation lands. Every ledger row
must name the requirement, state cells, constraint/gate, positive test,
adversarial mutation, evaluator differential, real-verifier result where
applicable, source digest, command, exit status, and residual assumption.

### 🟠 S2-03 — The native “streaming” evaluator exceeds its declared resident model

**Classification:** `code_weaker_than_spec`  
**Confidence:** high

The trace source owns a private spool and external-sort resource profile, but
`TraceSemanticMachineV2` retains:

- `spent_ids: Vec<[u8; 32]>`;
- `output_ids: Vec<[u8; 32]>`;
- `jmt_payload: Vec<u8>` up to `max_content_bytes`;
- a per-record `Vec<RecursiveTraceCanonicalChunkV2>`.

These are bounded, so this is not an unbounded-allocation claim. It is still a
second resident representation that can approach the 64 MiB content cap and is
not accounted as the profile's small resident sorter buffer. It weakens the
bounded-memory theorem and complicates differential parity with the O(1) Nova
machine.

**Correction:** either stream strict JMT-envelope decoding and uniqueness
comparison from the existing spool/external-sort owners, or explicitly add and
measure these native evaluator buffers in the authority profile. Do not keep a
hidden “bounded therefore free” memory class.

### 🟠 S2-04 — Planning coverage passes while cardinality and authority mirrors disagree

**Classification:** documentation/control-plane drift  
**Confidence:** high

The coverage command currently passes:

```text
ATOMIC COVERAGE AUDIT PASSED: atoms=1319 owners=13 shoulds=8 missing=0
duplicate=0 drift=0 semantic_pointers=1319/1319 ...
```

That is useful planning-map evidence only. It does not inspect implementation
symbols or constraints. In addition:

- the script's legacy primary-ID loop still enumerates `RCP-INV-001..022` and
  `RCP-AC-001..041`;
- the current coverage mirror documents 25 invariants and 47 acceptance
  criteria elsewhere;
- the footer says 23 invariant owners and 42 acceptance owners;
- the done-definition heading says 26 bullets while the displayed table ends at
  `DOD-25`;
- the handoff map has entries 1..14, while other text reports 13 items.

These discrepancies do not prove a missing implementation by themselves, but
they make a green audit easy to overinterpret and make future high-numbered
requirements fragile.

**Correction:** derive all cardinalities from parsed source, fail on any
hard-coded disagreement, and add an implementation-evidence mode that requires
the theorem-code/mutation matrix. Keep the existing atomic inventory as a
planning gate, not an implementation acceptance gate.

### 🟠 S2-05 — Canonical owner text is correct, but stale mirrors can recreate the deleted crate

**Classification:** documentation drift with future implementation risk

The controlling Plan 051 ownership table and current TODO module table place
Nova and Plonky3 under `z00z_storage::checkpoint::recursive_v2`; live code
matches this and keeps `nova-snark` private to the storage adapter boundary.

However:

- Plan 051 frontmatter still lists many `crates/z00z_recursive_proofs/...`
  targets even though T0 deleted that crate;
- Plan 07 still proposes creating `z00z_recursive_proofs` and runs package
  commands against it;
- the TODO C4 diagram still shows a `z00z_recursive_proofs` container;
- a Coverage SHOULD disposition says that crate owns both adapters.

Following those mirrors literally in Plan 07 would recreate the forbidden
second owner.

**Correction:** do not edit historical Plans 01–05. Before Plan 06 begins,
issue one current-authority reconciliation for Plan 051 and Plans 06–13:
storage-owned backend modules, no `z00z_recursive_proofs` crate, no duplicate
statement/root authority, and exact replacement commands/paths.

### 🟡 S3-01 — The ZIP evidence packet contains a stale Plan 051 copy

Four archived evidence documents are byte-identical to their live copies, but
the archived Plan 051 digest is
`21f2d28b77076688ef3d9f0e0db6cf522cc69cd7e0342ba760808c4083073f0e`,
while the live plan digest is
`b6fae8fe6c0e858bce36444bbc6d22041cb9e73b563cde2526c82d23111370ff`.

**Correction:** mark the ZIP as a dated non-authoritative snapshot and add a
manifest naming each archived digest and the controlling live-plan digest. Do
not silently replace or delete it while it may be evidence.

### 🟡 S3-02 — T0 manifest counts are stale even though the current scoped V1 scan is clean

The V1-eradication manifest retains an older 1,084-atom coverage statement,
while current coverage reports 1,319 atoms. The manifest's security claim is
recursive-V1 eradication, not current whole-phase coverage, so this does not
reopen T0 by itself.

**Correction:** add a non-destructive note that the count is historical and
link the current coverage digest. Do not rewrite Plans 01–05 or retroactively
change captured T0 evidence.

## ✅ Confirmed Strengths To Preserve

The correction must preserve the following good work:

- exact `nova-snark = 0.73.0` pin with a narrow `io` feature set;
- concrete Nova types remain private to `recursive_v2::nova`;
- no scoped recursive-proof V1 symbol was found in live code by the current
  seed scan;
- authority and snapshot constructors are no longer public caller-selected
  values; the repository-local fixture limitation is explicit;
- canonical transition preflights and seals the source against an isolated
  clone before the live commit and returns `CommittedWithoutSource` after an
  admitted commit instead of implying rollback;
- real pinned JMT update proofs are canonically decoded, independently executed,
  and hierarchy promotions are checked against the live post-definition root;
- the source/global byte contexts now consume the same canonical chunk stream,
  constrain FIPS framing/padding/chaining, and close against the precommitted
  trace digest;
- replay payloads are parsed from the canonical chunk feeder rather than a
  second witness tape;
- the uniqueness precommit/challenge payload relationship is directly
  constrained;
- the control-transition table has exhaustive finite-state tests;
- PP/PK are separated from the verifier bundle, proof envelopes contain no key
  material, and strict bounded/canonical decoding precedes proof verification;
- the code and plan explicitly avoid claiming PQ authority or canonical
  admission from Nova sidecars;
- the current code honestly labels unconnected families and resource numbers as
  non-acceptance.

These are valuable foundations. The problem is incomplete composition, not a
need to discard the implementation.

## 🛠️ Ordered Correction Plan For Coder-Agent

### 🧱 Gate 0 — Stop false closure work and establish single ownership

1. Keep T3, T4, and Plans 06–13 locked.
2. Mark T1 `reopened-for-bounded-closure` and T2 `active-but-non-closeable`.
3. Stop global clean-review loops until the intrinsic theorem is complete and
   the external authority budget exists. Use scoped code reviews per relation.
4. Install the cross-process heavy-Nova single-flight gate before another full
   bootstrap.
5. Obtain or explicitly escalate the authority budget tuple from S1-04.

**Exit evidence:** status text names both intrinsic and external blockers;
second heavy invocation queues/rejects; no process-local mutex claim remains.

### 🧱 Gate 1 — Close the narrow T1 theorem boundary

1. Add the immutable pre-definition-root binding and root transitivity checks.
2. Add per-tree old/new root chaining, including the explicit null-root
   first-materialization exception.
3. Freeze the statement/public-input dependency DAG and assign every current
   field to exactly one layer.
4. Add previous/next backend roots and exact per-class declared/consumed counts
   to the correct layer; defer PP/VK/prior-IVC identities to `X_h`, not an
   earlier transition object.
5. Remove or profile the second resident `Vec` representations.
6. Produce the DC2-F11–F24 resolution ledger with source and tests.

**Exit evidence:** converging-pre-state cross-substitution rejects; one-field
statement substitutions reject; no unresolved S0/S1 audit row; memory profile
accounts for every retained byte class.

### 🧱 Gate 2 — Complete T2 semantics in theorem order

Implement and close one family at a time:

1. full uniqueness original/sorted products, strict lexicographic order,
   counts, acyclic challenges, and symbolic/toy-field evidence;
2. net-effect and same-path replacement relation;
3. JMT old-path authentication, all mutation cases, versions, roots, and
   update chaining;
4. terminal→bucket→serial→definition hierarchy induction;
5. replay/commit equivalence, unchanged-leaf commitment, delta/journal/link;
6. statement, parameter/bundle, prior state, expected final state, and every
   declared/consumed counter;
7. finalization with all transient machines empty and no trailing event.

For each family, require Model A logical attack, Model B evaluator/circuit
differential, and Model C recomputed real-proof mutation where applicable.

**Exit evidence:** no reserved family is referenced only by range/zero helpers;
the mutation ledger has one killed mutant for every security-critical gate.

### 🧱 Gate 3 — Measure only the complete circuit

1. Separate fast semantic verification from heavy acceptance verification.
2. Fast loop: format/check, shape equality, TestConstraintSystem,
   evaluator/circuit differential, targeted mutation tests, coverage/diff check.
3. Heavy milestone: exactly one source-bound real-Nova proof under the global
   lock.
4. After Gate 2, run the finite compile-time width matrix under the authority
   budget; record augmented primary and secondary shapes, PP/PK/VK/bundle bytes,
   stage RSS/time, cancellation, and verifier distribution/load.
5. Freeze exactly one passing width. If none passes, stop and request the
   theorem-preserving amendment already allowed by Plan 051; do not reduce the
   theorem or fixture.

**Exit evidence:** one complete selected candidate, no runtime selector, no
unselected production symbol, and all budget components pass.

### 🧱 Gate 4 — Materialize auditable evidence

Create the named integration tests, theorem-code matrix, benchmark record,
mutation ledger, proof fixtures, and bounded-worker reports. Each retained
report must include:

- source/lock/profile/spec/grammar/bundle digests;
- exact command and environment limits;
- start/end time, PID, exit/signal/timeout, peak RSS, and wall/CPU time;
- base and augmented shapes;
- PP/PK/VK/bundle/proof sizes and digests;
- all public initial/final state limbs;
- evaluator parity and mutation result;
- authority-budget version and pass/fail reason.

### 🧱 Gate 5 — Reconcile planning mirrors and run closure

1. Update current Plan 051/ROADMAP/STATE truth, not historical Plans 01–05.
2. Fix Plan 06–13 paths so none recreates `z00z_recursive_proofs`.
3. Regenerate coverage cardinalities and distinguish planning coverage from
   implementation coverage.
4. Mark the ZIP snapshot as non-authoritative.
5. Run dependency/source/feature/public-type checks, full release suites, two
   consecutive clean reviews, and the two required doublechecks.
6. Unlock Plan 06 only if T0–T4 gates and retained evidence all pass.

## 🧪 Minimum New Test And Evidence Matrix

| Scenario | Native evaluator | R1CS/TestCS | Real verifier | Required result |
| --- | --- | --- | --- | --- |
| Two different pre-states converge to one post-state; cross-substitute JMT envelope | yes | yes | yes | reject first top-level old-root mismatch |
| Mutate `NetMerge` payload and recompute canonical source/global hashes | yes | yes | yes | reject semantic net mismatch |
| Mutate JMT key/value/sibling/case/version/old root/new root and recompute envelopes | yes | yes | selected cases | reject exact JMT gate |
| Mutate hierarchy child root while recomputing parent/source hashes | yes | yes | yes | reject parent-child induction |
| Replay and commit streams differ but have equal counts | yes | yes | yes | reject transcript/item mismatch |
| Duplicate/unsorted spent or output IDs with recomputed precommit | yes | yes | yes | reject uniqueness/permutation gate |
| Change any profile/spec/grammar/bundle/PP/VK generation field | N/A | yes | yes | reject before proof decode or at bound state gate |
| Change any final-state limb and recompute envelope digest | N/A | N/A | yes | unchanged verifier or endpoint comparison rejects |
| Skip/trail/reorder one source/hash/JMT event | yes | yes | yes | reject count/order/finalization gate |
| Two bootstrap processes request heavy Nova simultaneously | N/A | N/A | process test | exactly one enters setup |
| Maximum authorized trace/evaluator memory | yes | yes | milestone only | measured within declared resident/total budgets |
| Coverage high-ID deletion/drift (`INV-025`, `AC-047`, handoff 14) | script | N/A | N/A | coverage audit fails |

## 🔁 Doublecheck Record

### 🔍 Pass 1 — Normative-to-code consistency

Every material claim was checked against both the current normative text and
live source rather than ROADMAP prose alone. The decisive source checks were:

- state-family reference search and direct `StepCircuit::synthesize` inspection;
- native evaluator → envelope → JMT proof → hierarchy dataflow;
- statement canonical byte field inventory;
- authority/snapshot constructor reachability;
- Nova dependency/privacy and proof/bundle decode order;
- current test inventory and artifact-path inventory;
- coverage script/cardinality comparison;
- process tree and bootstrap script inspection.

### 🔍 Pass 2 — Adversarial alternative explanations

| Alternative claim | Doublecheck result |
| --- | --- |
| “T2 is only slow; waiting longer will close it.” | Refuted. Source explicitly lacks required semantic families and authority budget. |
| “The post-definition root proves the JMT trace started at the selected pre-state.” | Refuted. The verifier API receives only the expected post-definition root; the pre-definition root is not a statement field. Distinct pre-states can converge after deletion/replacement. |
| “The mutex prevents concurrent heavy Nova setup.” | Refuted. It is process-local; two independent bootstraps/workers were observed. |
| “A passing 1,319-atom coverage audit proves implementation completeness.” | Refuted. The script proves planning mappings and has stale cardinality mirrors; it does not inspect constraints or real-verifier reachability. |
| “The large VK/RSS measurement selects the candidate.” | Refuted by Plan 051 itself: the relation is incomplete and the authority budget is absent. |
| “The live code created the wrong backend crate.” | Refuted. The controlling owner is storage and live code matches it; the drift is in stale frontmatter/diagram/downstream plan mirrors. |

### ⚠️ Remaining limitations

- No executable pre-state convergence exploit fixture exists yet; S1-02 is a
  high-confidence theorem/dataflow finding, not a demonstrated public live-path
  exploit.
- Concurrent coder-agent changes after the recorded source hashes require a
  targeted recheck of affected findings.
- Existing heavy test terminal outputs were not captured by this audit and are
  not counted as pass evidence.
- The audit did not attempt to prove Nova's external cryptographic assumptions;
  A-01–A-17 remain assumptions/applicability obligations as defined by the
  crypto audit.

## 🏁 Final Disposition

**Cryptographic verdict:** **RISKY BUT SALVAGEABLE**.  
**Implementation status:** T0 accepted with maintenance notes; T1 narrowly
reopened; T2 partially implemented and blocked; T3/T4 and Plans 06–13 locked.  
**Immediate next action:** install cross-process single-flight, freeze the
pre-definition-root/statement DAG correction, obtain the authority budget, and
continue fast semantic T2 work without another unconditional heavy bootstrap.  
**Unlock condition:** complete theorem, selected budget-compliant candidate,
retained Model A/B/C evidence, and converged reviews—never shape/proof smoke or
planning coverage alone.

## 📌 Live Remediation Record (2026-07-17)

This record is the current disposition of the findings above. It supersedes no
captured audit evidence and does not convert a partial repair into T1 or T2
acceptance.

| Finding | Current remediation | Remaining closure condition |
| --- | --- | --- |
| S1-01 | The one Nova byte context, replay prefix, uniqueness codecs, and NetMerge byte grammar remain in the sole storage-owned circuit. **Authority-reviewed theorem amendment (2026-07-17):** the endpoint-free Poseidon root/frontier and additive pseudo-root remain removed. One stack-local canonical chunk still derives an immediately adjacent `SourceMemoryWrite` and `TraceChunk`; cursor/order, all metadata, 64 bytes, zero tail, both SHA contexts, EOF/padding/chaining, and final authority digest remain direct R1CS relations. The live exact-P continuation type-binds declared semantic/per-opcode work plus version, chain/height/predecessor, old roots, tx root, executable predicate, profile/spec/grammar and verifier-bundle identity. R1CS recomputes count/P, both U values, all eight challenge digests and all four list hashes before evaluating the two full-field original/sorted product pairs per set. Canonical replay now immediately precedes its commit-pass Original row; 34 O(1) state cells constrain pending-active, set, and all 32 terminal-ID bytes, while another replay or the challenge requires the pending row cleared. The replay codec now also carries the exact 32-byte storage-owned old/new JMT leaf-value hash and the fixed-shape R1CS parser consumes that authenticated field. Globally strict order rejects cross-set equality, and the sole global TraceClosure END checks all 16 live opcode counters. Current static shape is `C=221,624/V=138,756/NZ=810,066/G=1,048,577`, PP lower bound `71,276,224 B`; the resumed-cycle bootstrap was green before this slice and focused post-change release gates pass. No fresh augmented worker was run. All prior worker/proof/envelope figures are historical diagnostics only. | The source-byte, constrained count/P/U/challenge/list-hash, product/order, replay→Original identifier, and per-opcode-accounting subgaps are closed; the exact leaf commitment is now available but not yet consumed by full-row permutation/Net. Close full path/value semantic Net/unchanged-leaf, freeze/prove the degree/query bound and Models A/B/C, and implement JMT, hierarchy, settlement-root/statement/final-state relations and every remaining row in `069-051-T2-GAPS.md`; obtain the authority operating budget and remeasure only the complete circuit. No bounded arena, native/digest substitute, second owner, or T3 start is permitted. |
| S1-02 | `RecursiveSnapshotHandleV2` captures `pre_definition_root`; the snapshot digest, canonical transition, evaluator, and typed statement bind it. The native converging-prestate cross-substitution fixture now rejects B's real envelope under A at the first old-root gate. | Constrain the complete old/new-root family in R1CS and retain recomputed real-verifier mutations. |
| S1-03 | STATE and ROADMAP reopen T1 for bounded theorem closure; the typed statement is only a transition commitment, not `X_h`. Its frozen DAG row now commits exact declared/consumed per-opcode counts. | Add backend-root identities to `X_h`, bundle/parameter identity, prior IVC state, expected final fields, and one-field Model C mutations. |
| S1-04 | No budget tuple was inferred or fabricated. The missing authority tuple is recorded as an external acceptance blocker. | Authority must supply the versioned numeric budget and candidate decision path. |
| S1-05 | `nova.rs` now uses an OS-visible parent lease with PID/start/source-digest/command metadata and a child bypass marker; a cross-process competing-parent probe rejects. | Retain the two-real-heavy-parent setup-entry test only at the final Gate 3 milestone; do not repeat incomplete-circuit heavy measurements. |
| S2-01 | Fast semantic/shape/mutation checks are the active loop; retained real-Nova work remains milestone-only. | Measure only after the complete relation and authority budget exist. |
| S2-02 | The evidence contract remains open; no prose-only ledger is treated as verifier evidence. | Materialize theorem-code matrix, mutation ledger, integration fixtures, bounded-worker report, and Models A/B/C rows as relations land. |
| S2-03 | The evaluator no longer retains spent/output ID vectors: it compares canonical uniqueness payloads with sealed external-sort commitments and reconstructs each `TraceChunk` from a synchronous borrow of the sole source record. Its profile now commits the bounded JMT envelope, two current-record representations, and sorter buffer capacity. | Authority-bound measurement of the accounted classes and eventual streaming of the JMT envelope remain open. |
| S2-04 | Coverage now derives invariant and acceptance cardinalities from `069-TODO.md`; `--implementation-evidence` fails closed until the theorem/mutation artifacts exist. | Make every implementation-evidence row source-bound and complete before using it for acceptance. |
| S2-05 | Current Plan 051 and Plans 06–13 are reconciled to `z00z_storage::checkpoint::recursive_v2`; no later plan may recreate a backend crate or second owner. | Execute only after T2–T4 close; verify paths against then-live modules. |
| S3-01 | The referenced ZIP evidence packet is absent from this worktree, so it was not replaced or relabelled. | Its custodian must provide the archive path; attach a dated non-authoritative manifest with the recorded live/archived Plan 051 digests. |
| S3-02 | The T0 manifest now labels `1084/1084` as historical and points to the independently regenerated current inventory. | Preserve the capture; never rewrite its historical evidence as current coverage. |

### 🔑 Live Full-Row/Net/JMT-Header Amendment (2026-07-18)

The S1-01 row above is historical exact-P evidence. The current circuit binds
the complete 100-byte replay/Original/Sorted semantic row, consumes all fifty
`u16` limbs in four list hashes and both product pairs, and implements semantic
Delete/Insert/Replace/Unchanged/Close Net with exact-path and leaf-hash rules.
The `Close` row directly binds precommit, `P`, spent `U`, and output `U`, then
checks effect and non-unchanged mutation counts. Circuit-spec version 6 binds
the corrected full-row security parameters `(50, 49, 248, 128)`; symbolic and
toy-field release tests cover the `49n` degree ceiling and independent-pair
accounting. The same authenticated chunks now bind the JMT envelope
version/generation/kind/digest/update count, canonical no-op digest and exact
width, plus the PromoteChildRoot trace digest and definition root. Direct
release mutations reject count, no-op digest/width, and promoted-digest
changes. The inactive wide/digest-only parser bodies were removed so only one
canonical implementation path remains. The superseded pre-micro base ShapeCS was
`C=272,812/V=163,847/NZ=989,221/G=1,048,577`; the static
PP/VK/bundle/Pedersen lower bounds are
`79,670,936/33,554,464/33,554,918/201,326,784 B`.

Focused release R1CS tests reject independent definition, serial, terminal,
leaf-hash, ordering, product, Net-kind, pending-row, Close-binding, and
cardinality mutations. This closes only the live full-row uniqueness and
semantic-Net and JMT-header/no-op/Promote subrelations. It does not alter S1-02
through S1-05 or authorize a complete-circuit worker: mutating JMT
operation/proof/path algebra, full hierarchy/roots/commitments/`X_h`/finals,
Models A/B/C, final artifacts, and the authority budget remain open.

### 🔑 Live Single JMT-Micro Path Amendment (2026-07-18)

Circuit-spec version 6 removes the resource-duplicating JMT source path. The
canonical trace no longer stores an opaque envelope body alongside its derived
records: it emits one fixed 39-byte authenticated header and one bounded
`JmtMicroOp` transcript. The storage-owned streaming inverse decoder enforces
update/operation/value/proof/end ordering, indices, canonical value chunking,
typed proof reconstruction, project raw-SHA semantics, pinned-JMT verification,
hierarchy semantics, and the exact transcript digest. It does not retain a
second envelope body, and digest derivation no longer performs a deep
`updates.to_vec()` clone. Source-record authority accounting is now derived
from the content-byte cap and therefore covers short micro records rather than
assuming max-leaf-sized envelope chunks.

The uniform circuit requires the exact unique header width, binds micro-op
version/kind/update/operation framing and completed counts, and keeps the
Promote digest relation. The exact base pin is
`C=273,537/V=164,495/NZ=991,969/G=1,048,577`; static
PP/VK/bundle/Pedersen lower bounds are
`79,798,256/33,554,464/33,554,918/201,326,784 B`. Focused release gates and the
mandatory bootstrap pass. This amendment closes the duplicate-envelope
canonical/resource path, not `I_jmt`: raw-SHA mutation cases and root chaining
are still not R1CS relations. Full hierarchy, roots, commitments, `X_h`, final
successor, Models A/B/C, artifacts, reopened T1 evidence, and the authority
budget remain blockers; T3 stays locked.

### 🔑 Live Authenticated JMT Transition Amendment (2026-07-18)

The sole bounded `JmtMicroOp` transcript is now a full per-update R1CS
transition relation. It constrains exact old leaf/null materialization,
leaf/value and parent raw-SHA preimages, sibling order/direction, split-prefix
count/direction and former-leaf/null prelude, all six insert/update/delete
cases, the declared new root, and old-root continuity across successive real
operations. Real pinned-JMT fixtures cover all six cases and a two-operation
chain. Direct mutations reject changed split count, direction, former leaf,
parent, active/case selection, and declared new root. The exact base pin is
`C=325,091/V=206,541/NZ=1,221,306/G=2,097,153`; static
PP/VK/bundle/Pedersen lower bounds are
`123,763,464/67,108,896/67,109,350/402,653,376 B`. Canonically ordered prior
value blocks are recomputed in R1CS and bound by presence, key, and value hash
to the authenticated old proof leaf. The mandatory bootstrap is
green with `z00z_storage` 250 passed, zero failed, and two ignored.

This closes the JMT per-update transition relation, not T2. Canonical hierarchy
induction, SettlementV2 roots/commitments, `X_h`/prior-IVC/exact final
successor, Models A/B/C, complete artifacts/measurement, and the external
authority budget remain blockers; T3 stays locked.

### 🔑 Live Hierarchy Transition Amendment (2026-07-18)

The sole authenticated JMT stream now feeds one in-circuit hierarchy machine.
R1CS proves the canonical terminal → bucket → serial → definition → optional
path-index stage schedule, strict role order, non-equal hierarchy update roots,
and authenticated prior/new parent-value coordinates and child roots. For each
of the three parent levels, two independently challenged products over the
fixed transition row plus exact counts prove that every child transition is
consumed exactly once and no parent operation invents or duplicates a child.
The former `PromoteChildRoot` zero-state requirement, which made a real
mutating hierarchy impossible to promote, is removed.

Exact-P codec version 2 commits `update_trace_digest` as digest anchor 12, and
the JMT header is equal to that anchor before any hierarchy product is
accepted. The release four-level hierarchy fixture reaches promotion and
passes. The exact base pin is
`C=337,927/V=216,219/NZ=1,266,770/G=2,097,153`; static
PP/VK/bundle/Pedersen lower bounds are
`125,890,088/67,108,896/67,109,350/402,653,376 B`.

This amendment closes schedule/order/two-sided-root/unused-child induction, not
the complete hierarchy theorem. The exact SettlementV2 Goldilocks Poseidon2
Serial/Definition operation-key relation is still absent. Settlement roots,
typed commitments, `X_h`, prior IVC, exact final successor, Models A/B/C,
complete artifacts/measurement, reopened T1 evidence, and the authority budget
also remain blockers; T3 stays locked.

### 🔑 Live Net→JMT, Canonical-Flow, And Final-State Amendment (2026-07-18)

Every mutating semantic Net row is now consumed exactly once by a terminal-tree
JMT operation. The circuit evaluates two independently challenged products over
the exact 132-byte row
`definition[32] || serial_le[4] || terminal[32] || old_hash[32] || new_hash[32]`
and compares exact cardinality at the sole promotion gate. The challenges are
derived before witness rows from the authenticated precommit. Insert and delete
select a constrained absent zero hash, replacement binds both hashes, and
Unchanged emits no JMT operation. A retained release mutation changes only the
terminal JMT value hash and reaches the direct product equality.

The second flow-item representation formerly emitted as `CommitTypedEvent` was
deleted from the canonical producer and native evaluator. `ReplayInput` and
`ReplayOutput` are the sole flow-item codec. `CommitTypedEvent` now accepts
exactly four ordered checkpoint-core commitments—delta root, witness/unchanged
root, journal digest and checkpoint-link digest—and compares their bytes with
the X_h-derived public state. The R1CS first-chunk gate rejects the former
Put/Delete payload tags directly, including a retained handcrafted-trace
mutant. This removal eliminates the former
replay/commit-equivalence obligation instead of replacing it with an unchecked
digest or another SHA context.

The live fixed shape also includes exact SettlementV2 Poseidon2 hierarchy
parent keys, settlement pre/post root derivation, `X_h`, prior finalized IVC
state, and the independently derived exact successor. Current base ShapeCS is
`C=533,905/V=401,549/NZ=2,036,951/G=2,097,153`; static
PP/VK/bundle/Pedersen lower bounds are
`161,400,800/67,108,896/67,109,350/402,653,376 B`.

This is not T2 acceptance. The complete mixed real compressed proof, final
Models A/B/C corpus, theorem/mutation/benchmark/A-17 artifacts, reopened T1
evidence, authority numeric budget/candidate selection and global review
convergence remain open. The optional developer PP+PK setup cache remains
deferred until relation stabilization. Durable folding recovery and Celestia
DA publication are T3/later integrations and do not substitute for a T2 gate.
T3 stays locked.

### 🔒 Ordered Live Work List

1. Treat the amended S1-01 source-byte/window relation as the sole main path;
   retain its direct mutation and bounded-worker regressions. Do not restore the
   endpoint-free Poseidon root/frontier or introduce an inner/digest substitute.
2. Complete the remaining T2 evidence families in theorem order: the mixed
   compressed proof, Models A/B/C, final theorem/mutation/benchmark/A-17
   artifacts, and evaluator/circuit/verifier differential evidence.
3. Obtain the S1-04 authority budget, then run the sole complete-circuit
   candidate/worker measurement and freeze one compile-time SHA width.
4. Materialize the retained Model A/B/C artifacts and only then execute T3/T4
   and the two-heavy-parent setup test.
