# 069-051 T2–T4 benchmark and artifact ledger

Current T4 candidate: proof-source
`1da05771ae22d8da4b8e8693954540f468708be47f25f7dc654a0f7f9df4c4e3`,
Nova owner
`dc075b43760601b3330e4738aae59312fdcb4415740333d96e2559d7b9aa07ef`.
Bootstrap, semantic, TestCS, artifact, proof/Model-C, clean-verifier, and
continuous-chain gates pass on this exact current identity.

## Active-candidate base shape and preflight

| Metric | Exact release value |
| --- | ---: |
| Base constraints / inputs / auxiliaries | `809,802 / 1 / 675,408` |
| Base nonzeros | `3,332,400` |
| Pedersen generators | `1,048,577` |
| PP / VK static payload lower bounds | `186,285,856 / 186,285,856 B` |
| Bundle / Pedersen RSS lower bounds | `523 / 201,326,784 B` |

Evidence: `final/semantic-source-1da/run.log`. Static lower bounds are
fail-fast arithmetic, not serialized-size or process-RSS measurements.

## Active generation-2 identity and T3 closure (2026-07-22)

Status: **T3/T4 COMPLETE ON CURRENT SOURCE.** The continuous same-`z_0` runner, strict
authority-selected artifact path, public receipt, and storage reload path pass.

| Active item | Exact value |
| --- | --- |
| Git release | `v1.18.0` / `df36df3d1b395c28cff91fc9d33c2494541e37de` (the source revision below is the proof-owned digest, not a Git SHA) |
| Source revision | `1da05771ae22d8da4b8e8693954540f468708be47f25f7dc654a0f7f9df4c4e3` |
| Nova source SHA-256 | `dc075b43760601b3330e4738aae59312fdcb4415740333d96e2559d7b9aa07ef` |
| Milestone worker source | `5573f73e36922368b8179551b47b2b03a31bf88ff6b67b23552eccf099961cf5` |
| Shape digest | `c0206283f9de5e4d75b007d0b05ea8491d8272665512a7ddd2f273229d16036e` |
| Authority generation / activation | `2` / heights `1..=5` |
| Authority digest | `8ae07172f268f67bf4d5d2b4b11562f6625d9b18e269741ce6d018fb01a4661c` |
| PP digest | `ee7b2d3863e6e9d54002eb2290f31b6b8a7a570e11a20fbf845b2ab617749500` |
| Prover material | `958,329,882 B`; raw SHA-256 `c449daa46d2522acfa9456a02c37e341edd4fca53483da39b9dfafd831f298cb` |
| Verifier bundle | `15,372,615 B`; raw SHA-256 `86da72808877492cf73bb5ac3e0878abfd8c97ecbe9e91b2a0efb3d6d68fdf38` |
| Prover/verifier header project digests | `f3bfbfc6ff3129df1ebf3c0dbc5ef3e52f22adde7a3de66b9c8c67a201c4e954` / `8ab9d9c065af68a4f58c7892ab440c1369f6f712f78815fb99f847ccb6b6ea21` |
| Authority role-framed bundle pin | `d76c545b57d2cffec8e2e2564241858d5ab66d91ea537ed18306461eaab0e1ff` |
| Decoded / compact / compressed compact VK | `977,729,672 / 910,096,377 / 15,372,093 B` |
| Augmented primary/secondary constraints | `1,272,192 / 10,349` |
| Augmented primary/secondary variables | `1,141,255 / 10,331` |
| Active artifact corpus | `3/3 PASS`; verifier mutation worker `355.403 s`, `10,640,736,256 B` peak RSS; source binding `191.304 s`, `10,564,128,768 B` peak RSS |
| Continuous T3 milestones | `(block, steps, framed envelope bytes) = (1,316,none/FoldOnly), (3,948,346907), (5,1580,346907)`; each envelope payload is header `449` + two portable inputs `417` each + two public states `110944` each + proof `123688`, plus the `48 B` public-object preheader |
| T3 local evidence | first set `346,907 + 533 + 354 = 347,794 B`; second adds `346,907 + 533 = 347,440 B` because the claim is shared; total `695,234 B`; opaque receipt `572 B` in memory only |
| Continuous T3 chain | `1/1 PASS` in `1,634.91 s` test time / `27:15.67` command wall; `9,843,589,120 B` peak RSS; swap 0 |
| Generation-2 full proof + Model C | PASS with proof-binding bundle digest `e6bd650d3020977f24a2c2cbac36009a6e465cc4cac784f7bbce0bf5be2125ec`; `2,526.747 s` bounded worker / `2,527.058 s` harness; `8,111,263,744 B` peak RSS; Model C `1,190.058 s`, unchanged verifier accepted its own statement and target comparator rejected |
| Generation-2 clean verifier | PASS; `58.343 s`; sampled VmRSS `3,289,395,200 B`; kernel VmHWM `3,348,504,576 B`; `946,462,720 B` below 4 GiB; process cleanup clean |
| Full-fixture proof/envelope | proof `123,688 B`; envelope payload/framed object `346,859/346,907 B`; two `417 B` portable inputs, two `110,944 B` public states, and a `449 B` payload header |
| Fixed-codec envelope ceiling | payload `354,243 B` plus `48 B` public-object preheader at the `131,072 B` proof-body cap; below the separate `524,288 B` authority cap |
| Verifier evidence digests | measurement bundle `3436796c4f1ea68d6d5624887e96180d4dd515bed0d56d0ac10c8e134f30fee8`; `measurement.env` SHA-256 `0bfaa954bd3b230ebeb69300cd2cf8d9023c89b812f35c1aad16eb94c35e418d`; transcript SHA-256 `104cb0e64d2c1900320727f8078e7ef73048ff3e2995b806a545034d9f95fafe` |

Canonical final evidence is retained under
`crates/z00z_storage/outputs/checkpoint/069-051/final/`, specifically
`artifacts/source-1da-current.log`, `artifacts/source-1da-active-material/`,
`t3-chain-source-1da/`, and `verifier-rss-source-1da/`. No 8–30 KiB proof-size claim is made; the
body is `123,688 B`, under the hard 128-KiB cap with `7,384 B` headroom.

Status: release-only resource ledger. Diagnostic ceilings and historical
incomplete relations are never candidate acceptance budgets.

## Exact release-profiler ledger

Cargo resolved the `test_recursive_circuit` executable from JSON compiler
artifacts; no target hash was guessed. The final executable is
`target/workspace/release/deps/test_recursive_circuit-a48ea54fcfb8117c`, SHA-256
`f5fcf9b9d70adea6ce2a40514027277c8b721d5fe79ce75264137a7a0184d49a`.
It exposes two bounded circuit-owner/terminal-shape tests. The complete Nova
lifecycle remains measured by the source-bound T2/T3 workers above; the small
resolved target is the exact profiler target required by T4 and must not be
misrepresented as a full-proof RSS sample.

| Release command/tool | Exact result |
| --- | --- |
| `/usr/bin/time -v <resolved-test> --nocapture --test-threads=1` | `2/2 PASS`; `<0.01 s` wall; `3,584 KiB` peak RSS; 0 major / 137 minor faults; exit 0 |
| Valgrind Massif 3.22.0 | `2/2 PASS`; `0.25 s` wall; `36,704 KiB` process peak RSS; `15.93 KiB` peak Massif-tracked heap; 67 snapshots; exit 0 |
| `strace -c -f` 6.8 | `2/2 PASS`; 135 calls, 6 expected probing errors, `0.001090 s` traced syscall time; exit 0 |
| Criterion release profile | exit 0; `3:28.77` wall, `371.69/1.17 s` user/system, 178% CPU, `336,704 KiB` peak RSS; authority resolve median `8.8196 us`; representative preheader encode about `12.6 ns`; validation about `14–15 ns`; fail-closed rejects about `58–62 ns` |
| `perf stat` / `perf record` 6.8.12 | unavailable on this host: kernel `perf_event_paranoid=4`; both exit 255 before the target ran |
| Samply 0.13.1 | unavailable for the same PMU policy; exit 1 before the target ran |

The non-root attempt to lower the PMU policy was denied and the original value
remained `4`; no synthetic counters or flame graph are reported. Raw commands,
tool versions, Cargo JSON, executable SHA, Massif snapshots, strace summary,
Criterion output, and denied-tool diagnostics are retained in
`final/profilers/`.

## Dependency-scanner disposition

`cargo audit` and `cargo deny check` both completed with exit 0 on the pinned
`Cargo.lock`. Their informational warnings are retained, not silently treated
as vulnerabilities or as clean output:

| Finding | Current reachability and disposition |
| --- | --- |
| `RUSTSEC-2026-0186`, `memmap2 0.9.10` | The affected range-advice/flush APIs have zero project-source callers. The dependency is reachable only through the optional wallet GUI Wayland stack (`eframe -> winit/smithay`), not the recursive prover, verifier, storage admission, or aggregator runtime path. `0.9.11` is patched; changing the generation-2 lockfile here would invalidate the pinned Nova dependency identity and every authoritative PP/VK/proof artifact, so the upgrade is a mandatory coordinated dependency-generation rotation rather than an in-place Plan-051 edit. |
| `derivative`, `instant`, `ttf-parser` unmaintained | Transitive wallet GUI/accessibility/font dependencies; no recursive path ownership. Replace through a coordinated GUI dependency upgrade. |
| `paste` unmaintained | Transitive macro dependency in both `halo2curves`/Nova and Plonky3. No runtime API surface; replacement belongs to an audited proof-stack rotation. |
| `spin 0.10.0` yanked | Selected transitively by Plonky3 `p3-dft`/`p3-monty-31`; no RustSec vulnerability was reported. Keep the source-bound generation-2 lock and rotate only with the proof dependency identity and regenerated authority artifacts. |
| `nova-snark 0.73.0` lacks an SPDX manifest expression | The registry package declares `license-file = "LICENSE"`; the shipped license text is the MIT license. This is metadata debt, not missing license material. |

Two obsolete quick-xml advisory ignores that no longer matched the resolved
graph were removed from `deny.toml`; the live bincode migration exception
remains explicit. Raw scanner and reverse-dependency logs are retained under
`final/gates/`.

## Authority-selected streaming profile

The finite candidate space was segment caps `{1, 4, 8 MiB}`, HJMT thread caps
`{1, 2, 4}`, prover concurrency `{1}`, and SHA width `{1}`. Authority selected
exactly one production tuple: 1 MiB segments, one HJMT worker, 2 MiB in-flight
results (`2 × threads × segment`), a separate 64 MiB input/snapshot
reservation, one Nova prover, `k=1`, a 1 GiB identity-bound PP/PK cache, and
deterministic replay recovery. No runtime candidate selector remains.

| HJMT CPU candidate | Same real transition wall time | Peak RSS | Result |
| ---: | ---: | ---: | --- |
| 1 | 1.17 s | 588,700 KiB | selected; canonical bytes/root/digest accepted |
| 2 | 1.16 s | 573,184 KiB | measured alternative; not production-selectable |
| 4 | 1.16 s | 572,460 KiB | measured alternative; not production-selectable |

The selected native evaluator resident equation is exactly `5,374,042 B`:
one 1 MiB segment, 2 MiB bounded results, two 65,581-byte source records, and
a 2 MiB sorter buffer. The separate 64 MiB input/snapshot reservation is not
double-counted as in-flight result memory. Segment caps 4/8 MiB were retained
only for finite cap arithmetic; final proof and artifacts bind the selected
1 MiB compile-time profile.

## Superseded generation-1 complete-relation measurements

| Metric | Value | Evidence class |
| --- | --- | --- |
| Base ShapeCS constraints | 533,794 | exact current static preflight |
| Base ShapeCS auxiliaries | 401,550 | exact current static preflight |
| Base ShapeCS nonzeros | 2,036,733 | exact current static preflight |
| Pedersen generator count | 1,048,577 | exact current static preflight |
| PP lower bound | 127,834,984 bytes | static lower bound, not serialization measurement |
| VK lower bound | 127,834,984 bytes | static lower bound, not serialization measurement |
| Verifier-bundle lower bound | 523 bytes | format-4 header plus nonempty compressed compact VK |
| Pedersen setup RSS lower bound | 201,326,784 bytes | lower bound, not peak RSS |
| Augmented primary/secondary constraints | 991,488 / 10,349 | exact release `PublicParams` shape |
| Augmented primary/secondary variables | 862,667 / 10,331 | exact release `PublicParams` shape |
| Source revision digest | `e58e2f9a2f715a64b37dd464248b57601e7deda4254086c0b6598160cf30dbd6` | exact final whole-source identity |
| Worker source digest | `272379f7f47f735dc2536682c23e3e3d93e1434933f863f8e8841e89106d8ca0` | exact final milestone-worker identity |
| Complete mixed steps | 1,727 | every one of the 17 opcode classes in one canonical transition |
| PP setup | 11.880 s | fresh cache-bypassing bounded release worker |
| Recursive accumulator creation | 0.381 s | same worker |
| First `prove_step` | <1 ms | same worker; Nova initialization step |
| Remaining folds | 1,726 in 1,139.896 s; 660.426 ms/fold | same worker |
| Compression setup / prove | 3.905 s / 19.755 s | same worker |
| In-process VK load + proof decode/check | 5.010 s | same worker |
| Clean verifier-only process | 29.340 s | PP/PK/setup path forbidden; bundle cold-loaded |
| Verifier-only sampled peak VmRSS | 3,056,861,184 bytes (`2,985,216 KiB`) | all-thread recursive `/proc` sample over the marker-bearing subtree |
| Verifier-only kernel peak VmHWM | 3,056,861,184 bytes (`2,985,216 KiB`) | authoritative verifier-stage peak; 1,238,106,112 bytes below the active 4 GiB cap |
| Full proof plus independently recomputed Model C | 2,317.436 s | Model C recomputation 1,075.488 s; target comparator rejected |
| Full worker peak RSS | 6,564,720,640 bytes | bounded worker, `RLIMIT_AS=24 GiB`, no Nova runtime cache |
| Compressed proof | 122,288 bytes | exact canonical bincode |
| Initial/final public state | 109,856 / 109,856 bytes | 3,433 Pallas scalars each |
| DA proof envelope | 342,353 bytes | 353-byte header + both states + proof |
| Decoded/compressed VK | 859,756,576 / 47,007,663 bytes | strict source-only real-key corpus |
| Authority-distributed VK bundle | 47,008,185 bytes | 522-byte header + compressed VK; excluded from each DA envelope |
| Private canonical PP/PK/header/total | 858,784,984 / 208 / 522 / 858,785,714 bytes | strict recovery roundtrip; deterministic PK regeneration |
| Current-source artifact corpus | 3/3 exact ignored tests passed | corrected worker/source identity; production parameters |
| Strict verifier artifact worker | 191.637 s / 7,828,090,880 bytes peak RSS | 47,008,185-byte bundle plus invalid-key corpus |
| Exact source-binding artifact worker | 111.073 s / 8,073,723,904 bytes peak RSS | real proof binds one source event after trace begin |
| Profile-accounted native resident buffers | 5,374,042 bytes | exact selected equation: 1 MiB segment + 2 MiB results + two 65,581-byte source records + 2 MiB sorter capacity |
| One-real-HJMT-transition selected release test | 1.17 s / 588,700 KiB peak RSS | exact same canonical transition on the selected single-worker path |
| Private PP/PK cache | 1 GiB cap; 858,785,714-byte canonical material | full identity key, exclusive lock, `0700/0600`, atomic no-clobber write, strict load/re-encode, corrupt/dangling-entry deletion and regeneration |
| Recovery boundary | deterministic replay only | identity-bound sealed trace digest and next height; no accumulator image or cached verdict |
| Owned cutover crash corpus | 5/5 abrupt-exit seams passed | complete-old-or-complete-new reopen; redb internal fsync/directory-sync remains equivalent-only |
| Project-owned process-secret corpus | 6/6 outcomes passed | success/failure/panic/timeout/cancel/hard-kill; core-zero/non-dumpable, canary-free private artifacts/diagnostics |

Residual-corpus source identity is exact but does not change the immutable Nova
proof-source identity:

| Source | SHA-256 |
| --- | --- |
| `backend/redb/helpers.rs` | `ed9d7589d7f9b2814690e28b50a871c9422a44ebd007c3d2c62a8798cc28ac4a` |
| `backend/redb/mod.rs` | `1f8739d161a180345a4c21beb5e8568662f0c6868f819c5d08a3a55288a7bf13` |
| `settlement/store.rs` | `8502fd89255fac6268a5be752232fcfe96304dc8ac5e4f472874998b0b6fefa0` |
| `test_recursive_v2_nova_adversarial.rs` | `324bf25f4e5a63bbde7586fe0c006ac08c98b3aeaee6c083103fec4d07f1e021` |
| `test_os_hardening_integration.rs` | `98efe53dba6e8c2150541f69c27f45b90dec841e2b753dafdc3502e83f649356` |

The static PP/VK lower bounds above use the currently pinned non-preprocessing
Spartan wire accounting. They do not authorize allocations. Exact serialized
sizes and the emergency-capped release measurements are retained separately;
neither is an authority operating budget.

The earlier `2,255.147 s`/`6,561,574,912 B` report is stale for the final
source identity: its ledger timestamp predates the test-tier edit while
`source_revision_digest()` binds the entire `nova.rs`. The values above come
from the corrected worker, which passes `--ignored` to the exact child and
requires a child execution marker before accepting exit status or RSS.

The historical verifier-only rows came from the generation-1 accepted release
harness. Its old raw output path was removed by the required cleanup; its
measurement-bundle SHA-256 remains
`465fe1322894af2ecc49d084932b5a20a462b6a92b2e6c8150124fb032a82136`;
measurement-file SHA-256
`86009e855e9f8be5ea621e25375bfe9af05cca3b4fbccbd2026dd1bd82e352d4`.
The harness binds the exact source/worker/Cargo.lock identities, samples the
complete marker-bearing process subtree, requires zero terminal exits, cleans
the isolated process group, and leaves the single-flight worker lock free.
The final source review corrected source retention/cap/zeroization/cache issues
in pass 1 and one stale harness comment in pass 2, then ended with consecutive
significant-clean passes 3–4. Two independent theorem/stream/concurrency and
Model-A/B/C/cache/recovery doublechecks pass. The exact bootstrap-first release
gate passed as follows: bootstrap `3:09.07`/`3,073,912 KiB`; semantic 36/36
`1:09:45`/`6,565,988 KiB`; TestCS 1,727/1,727 `4:23.55`/`5,039,244 KiB`;
artifacts 3/3 `7:48.43`/`7,884,496 KiB`; all-target build
`2:16.60`/`3,502,916 KiB`; workspace tests `30:44.10`/`4,167,768 KiB`.
The final `v1.14.1` detached clean clone passed bootstrap first
`2:55.62`/`3,062,652 KiB`, curated `1:41.07`/`3,054,092 KiB`, all-target build
`2:21.51`/`3,365,040 KiB`, and 350 workspace result blocks with zero failures
in `44:37.30`/`4,076,048 KiB`.

## Historical diagnostic only

The previously reported approximately 344–360 second pipelines and 53–57 KiB
proofs predate the complete relation and the format-3 canonical owner identity.
They demonstrate that setup, compression, and verifier cold-load dominate a
small fold, but they cannot select the current candidate. The generation-1
provider-neutral envelope was 342,353 bytes; the active generation-2 envelope
sizes are recorded at the top of this ledger. Live Celestia publication remains
a later-plan integration claim.

## DA accounting contract

For each accepted checkpoint, the portable Nova DA object is exactly the
format-2 proof-envelope header, initial public state, final public state, and
compressed proof body. PP, PK, and VK are excluded. The generation-bound VK
bundle is distributed as authority material and referenced by digest; it is not
repeated in every Celestia envelope. Celestia integration remains a later plan,
so this ledger records the exact payload contract without claiming live DA
publication.

## Active authority disposition

The numeric tuple originally accepted during generation 1 is carried unchanged
into active authority generation 2 and recorded in
`069-051-AUTHORITY-OPERATING-BUDGET-DRAFT.md` under decision reference
`phase-069-t2-interactive-authority-2026-07-20`. Generation 2 replaces the
artifact/profile identities; it does not create a second selectable resource
profile. The tuple selects only `k=1`, one Nova prover, one HJMT worker and the
1 MiB segmented stream. The 24 GiB worker limit remains an emergency harness
ceiling and is not the production budget.
