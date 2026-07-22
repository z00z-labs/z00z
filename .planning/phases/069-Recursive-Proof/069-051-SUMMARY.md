---
phase: 069
plan: 069-051
status: complete
completed: 2026-07-22
next_plan: 069-06
---

# 069-051 Summary

Plan 069-051 is complete. T0–T4 deliver one storage-owned recursive V2 path,
one current authority generation, one fixed-shape relation, one real Nova
proof/verification path, one post-write receipt path, and a complete
source-bound resource profile. No V1 decoder, alias, fallback, alternative
Nova owner, or proof-selected authority path is live.

## Final identity

| Identity | Value |
| --- | --- |
| Git release / commit | `v1.18.0` / `df36df3d1b395c28cff91fc9d33c2494541e37de` |
| Proof-source digest | `1da05771ae22d8da4b8e8693954540f468708be47f25f7dc654a0f7f9df4c4e3` |
| Nova owner SHA-256 | `dc075b43760601b3330e4738aae59312fdcb4415740333d96e2559d7b9aa07ef` |
| Milestone worker digest | `5573f73e36922368b8179551b47b2b03a31bf88ff6b67b23552eccf099961cf5` |
| Cargo.lock SHA-256 | `23a86f3341579b25ad5be96080a642405633df5f8c6e99dd4c3329d7d51f2a11` |
| Authority generation / activation | `2` / heights `1..=5` |
| Role-framed verifier-bundle pin | `d76c545b57d2cffec8e2e2564241858d5ab66d91ea537ed18306461eaab0e1ff` |

## T0–T4 result

| Task | Result |
| --- | --- |
| T0 | Recursive V1 implementation/runtime reachability eradicated; authority resolution, opaque historical evidence, compile-fail and zero-hit gates retained. |
| T1 | Canonical V2 transition trace, bounded segmented HJMT relation, typed state, ConfigV3/version-registry ownership, and exact storage endpoints implemented. |
| T2 | Complete fixed-shape Nova relation, split prover/verifier artifacts, conditional A-17 assumption ledger, real proof plus unchanged-verifier Model C, and numeric authority envelope accepted. |
| T3 | One continuous same-`z_0` runner folds every block; requested snapshots at blocks 3 and 5 pass public ingress, storage reload, unchanged verifier, exact endpoint comparison, and terminal receipt issuance. |
| T4 | Theorem/code/mutation coverage, release profilers, artifact/RSS/timing measurements, dependency/security disposition, full release validation, review convergence, and this summary are complete. |

The user-identified overlong test is now the canonical five-word
`test_uniqueness_row_version_bound`; the old identifier has zero repository
hits. `CheckpointProofSystem::VERIFIED` remains disabled because later Phase
069 promotion, publication, retention, scheduler, and production-SLO gates are
live requirements rather than work that Plan 051 may claim.

## Measured lifecycle

The complete resource and cost answer is
`069-051-PROFILING-REPORT.md`. Key current-source values are:

- base relation: `809,802` constraints, `675,408` auxiliaries and `3,332,400`
  nonzeros;
- prover material `958,329,882 B`; active verifier bundle `15,372,615 B`;
- proof body `123,688 B`; envelope payload/framed bytes
  `346,859 / 346,907 B`;
- full 1,727-step proof plus Model C: `2,526.747 s` bounded worker,
  `8,111,263,744 B` peak RSS;
- clean verifier: `58.343 s`, `3,348,504,576 B` kernel peak HWM;
- continuous blocks 1/3/5: `1,634.91 s` test time and
  `9,843,589,120 B` peak RSS;
- five-second finality cannot wait for the current sequential Nova lane: the
  representative fold work is about `42.29×` slower, while the full T3
  lifecycle averages `326.982 s/block` (`65.40×`).

The declared live cadence is one fold per block, a candidate local recovery
snapshot every 100 blocks, and candidate compression/publication every 1000
blocks. Only folding is an active automatic cadence here; compressed snapshots
exist only behind explicit requests. At five seconds per block the
90-day window is `1,555,200` blocks. A current framed envelope would occupy
`539,509,766,400 B` if incorrectly published every block,
`5,395,097,664 B` every 100 blocks, or `539,440,385 B` for the 1,555 complete
1000-block publications. These are exact envelope comparisons, not claims that
the downstream publisher exists.

Exact DA payload, accumulator-snapshot, challenge-pack, RS shard, IPFS/Kubo,
provider-receipt, retrieval-audit, production scheduler, and production p95/p99
values remain mandatory live Phase-069 scope. The profiling report gives
source-backed formulas and explicit blockers; it does not fabricate missing
bytes or hardware capacity.

## Final validation

| Gate | Terminal result |
| --- | --- |
| Mandatory bootstrap after final review fix | PASS, `4:57.48`, `9,657,732 KiB` peak RSS, swap `0` |
| Semantic / TestCS / artifacts | `36/36` / `1,727/1,727` / `3/3` PASS |
| Full proof + Model C / clean verifier | PASS / PASS |
| Continuous 1/3/5-block chain | PASS |
| `cargo test --release` workspace | PASS, exit `0` |
| Release all-target build / clippy `-D warnings` | PASS / PASS |
| Strict project rustdoc | PASS with vendored Tari packages excluded; vendor tree unchanged |
| `cargo audit` / `cargo deny check` | exit `0` / exit `0`; retained dependency-generation rotation dispositions are explicit |
| Release feature guard / coverage audit | PASS / `1,335` atoms, zero missing/duplicate/drift |
| Full verify `--max-safe-run` | PASS, exit `0`, `1:06:46`; 385 planned, 17 safety-skipped, 0 failed |

The raw evidence is retained only below
`crates/z00z_storage/outputs/checkpoint/069-051/final`. Superseded source
directories were moved through `gio trash`; the current tree is about `934 MiB`.
Repository-root `test-results` remains absent. All Phase-069 planning files are
owner-writable and have no immutable flag.

## Review record

Five inline `/GSD-Review-Tasks-Execution` YOLO passes were executed. Pass 1
fixed ledger identity/accounting defects; pass 2 removed superseded evidence;
pass 3 restored the Design Foundation vendor boundary; passes 4 and 5 found no
significant issue and are consecutive clean passes. Two complete `/doublecheck`
reports independently verified source/runtime claims and adversarial final-state
consistency.

## Handoff to 069-06

Plan 069-06 is active. It must implement and measure the positive finite
accumulator-snapshot cap, fork-bound snapshot codec, independent
fold/recovery/compression/publication cadence manifest, real restart/reorg
behavior, and immutable retention facts. It must not reinterpret future/target
language as optional, publish a proof every block, expose PP/PK to validators,
or take ownership of Plan-09 proof-body retention and Plan-10 scheduling/GC.
