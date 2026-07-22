# 069-051 release provenance manifest

Status: **HISTORICAL T2 RELEASE PROVENANCE; SUPERSEDED FOR WHOLE-PLAN STATUS.**

This file preserves the T2 release chronology. Whole-plan closeout uses the
current `v1.18.0` `main`/`origin/main` commit and the source-bound paths under
`crates/z00z_storage/outputs/checkpoint/069-051/final/`, as recorded by
`069-051-PROFILING-REPORT.md` and `069-051-SUMMARY.md`.

This manifest records what Git actually contains. It does not infer a scoped
commit from an earlier staging plan and does not treat local-only planning
evidence as release-tag content.

## Runtime release provenance

The final T2 runtime packet is present in external commit
`0256a85c2d6504ed94f5551be18987e25e226483`, tagged `v1.14.0`. That commit
was created outside the T2 executor while the shared worktree was changing. A
direct `git show --name-status 0256a85c...` proves that it is a broad repository
commit containing the T2 runtime packet together with unrelated planning,
wallet, extension and evidence changes. It must not be described as a scoped
41-path T2 commit.

The T2 runtime paths in that commit include the sole
`crates/z00z_storage/src/checkpoint/nova.rs` owner, its storage/HJMT/cutover
dependencies, the Nova adversarial and step tests, both milestone scripts,
the bootstrap repair, the wallet golden-owner repairs and the exact final
proof/RSS evidence. The old nested Nova owner is absent.

## Final clean-clone repair

Detached verification of `v1.14.0` exposed one clean-clone-only Scenario 11
test dependency on a gitignored planning glossary. The assertion was preserved
while its exact 61 term/claim-level pairs moved into the tracked fixture.

The repository version manager created patch release `v1.14.1` at
`37ece6c797d3807283eaea611252e657e10faad2`. Its commit scope is exactly:

```text
A  crates/z00z_simulator/tests/fixtures/scenario11_claim_registry.tsv
M  crates/z00z_simulator/tests/test_scenario_11.rs
M  versions.yaml
```

The tag points exactly to the commit and `versions.yaml` records
`1.14.1`/`v1.14.1`. The original workspace has a read-only `.git` index, so
the version manager ran in writable clone
`/tmp/z00z-069051-version.8oiEpW`; the three corresponding working-tree files
in the repository were verified byte-for-byte against that commit.

## Detached clean-clone result

Clean clone `/tmp/z00z-069051-authoritative-v1141-20260720T092702Z` was
detached at exact commit/tag
`37ece6c797d3807283eaea611252e657e10faad2`/`v1.14.1` and had a clean status.

| Gate | Result |
| --- | --- |
| Mandatory bootstrap first | PASS, `2:55.62`, peak RSS `3,062,652 KiB` |
| Curated release packet | PASS, `1:41.07`, peak RSS `3,054,092 KiB` |
| All-target release build | PASS, `2:21.51`, peak RSS `3,365,040 KiB` |
| Full workspace release tests | PASS, `44:37.30`, peak RSS `4,076,048 KiB`; 350 result blocks, zero failures |
| Scenario 11 claim registry | PASS from the tracked fixture with no planning-tree dependency |

## Local-only planning and proof evidence

At T2 close, `.planning/` was gitignored and GSD `commit_docs=false`; Phase 069
authority and ledgers were not force-added to those historical release tags.
The accepted proof/RSS packet originally occupied the paths below, but those
old raw files were subsequently removed by the required checkpoint-output
cleanup. Their hashes remain historical provenance; current raw evidence lives
only under `crates/z00z_storage/outputs/checkpoint/069-051/final/`.

```text
crates/z00z_storage/outputs/checkpoint/nova-verifier-rss/20260720T053334Z-4/measurement.env
crates/z00z_storage/outputs/checkpoint/nova-verifier-rss/20260720T053334Z-4/measurement.partial.env
crates/z00z_storage/outputs/checkpoint/nova-verifier-rss/20260720T053334Z-4/process-tree.log
crates/z00z_storage/outputs/checkpoint/nova-verifier-rss/20260720T053334Z-4/proof-transcript.log
crates/z00z_storage/outputs/checkpoint/nova-verifier-rss/20260720T053334Z-4/verifier-lineage.log
```

Its final source revision is
`e58e2f9a2f715a64b37dd464248b57601e7deda4254086c0b6598160cf30dbd6`,
the Nova source SHA-256 is
`1e39544c8c58f7d5a8117cdcdbf6ca0836e5e70e056d6c84f77e88fe1336c053`,
and the measurement-bundle SHA-256 is
`465fe1322894af2ecc49d084932b5a20a462b6a92b2e6c8150124fb032a82136`.

Superseded chronology documents were removed from the local planning packet on
2026-07-20 after their unique live decisions/evidence were verified as retained
by the active authority, gap, benchmark, mutation, theorem and migration
ledgers.

## Remote state

Both ordinary pushes of `main` plus `v1.14.1` exited `128` with
`Could not resolve host: github.com`. No force push was attempted. The writable
versioning clone's local `main` and `v1.14.1` remain at
`37ece6c797d3807283eaea611252e657e10faad2`; its `origin/main` remains at
`0256a85c2d6504ed94f5551be18987e25e226483`. Remote synchronization is an
external-transport follow-up and is not reported as complete.

The only authorized retry is an ordinary push of the existing branch and tag:

```bash
git -C /tmp/z00z-069051-version.8oiEpW push origin main refs/tags/v1.14.1
```
