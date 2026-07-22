# 069-051 authority operating-budget decision

Status: **ACTIVE — AUTHORITY GENERATION 2**

The historical filename is retained because Plan 051 and the benchmark ledger
reference it. Its contents are no longer a draft: this file is the sole active
generation-2 operating-budget decision. Superseded proposal, approval-template
and placeholder carriers were removed after activation.

## Active authority decision

```yaml
decision_schema: z00z.recursive.v2.operating-budget.decision.v1
decision_revision: 3
record_status: ACTIVE
authority_generation: 2
candidate_space:
  segment_cap_bytes: [1048576, 4194304, 8388608]
  hjmt_threads: [1, 2, 4]
  nova_prover_concurrency: [1]
  sha_batch_width_k: [1]
selected:
  segment_cap_bytes: 1048576
  hjmt_threads: 1
  in_flight_result_bytes: 2097152
  input_snapshot_reservation_bytes: 67108864
  segment_spool_bytes: 67108864
  nova_prover_concurrency: 1
  sha_batch_width_k: 1
  prover_material_resident_bytes: 1073741824
  recovery: DETERMINISTIC_REPLAY_FROM_SEALED_CANONICAL_SEGMENTS
  challenge_window_blocks_at_5s: 1555200
  nova_proof_retention: DISTINCT_PLAN09_POLICY_NOT_SET_BY_CHALLENGE_WINDOW
identities:
  authority_digest: 8ae07172f268f67bf4d5d2b4b11562f6625d9b18e269741ce6d018fb01a4661c
  profile_digest: 4568feb10fdb1ea33df48bc44c66a5a57c54035fe9b5021f51e45bc66428e93e
  spec_digest: 9eccb9666171da8be090debc03845147e7ecb624f7c0be3f6e56d1f341b300b7
  grammar_digest: 3ed491b3e252bd95044dfd6e921d74f09926157d9bb4a782ed16a316eedb50f0
  shape_digest: c0206283f9de5e4d75b007d0b05ea8491d8272665512a7ddd2f273229d16036e
  source_revision_digest: 1da05771ae22d8da4b8e8693954540f468708be47f25f7dc654a0f7f9df4c4e3
  worker_source_digest: 5573f73e36922368b8179551b47b2b03a31bf88ff6b67b23552eccf099961cf5
  nova_source_sha256: dc075b43760601b3330e4738aae59312fdcb4415740333d96e2559d7b9aa07ef
  cargo_lock_sha256: 23a86f3341579b25ad5be96080a642405633df5f8c6e99dd4c3329d7d51f2a11
  lockfile_digest: 6d2ca537433149f8c6956c42defa36caa44fd092fb40a04fa10a023b35898958
  manifest_digest: cd627622d5bfdc2bf3d3b4687d6c94ccafedda1e2facd245d0c13e430d7c1761
  public_parameters_digest: ee7b2d3863e6e9d54002eb2290f31b6b8a7a570e11a20fbf845b2ab617749500
  verifier_bundle_digest: d76c545b57d2cffec8e2e2564241858d5ab66d91ea537ed18306461eaab0e1ff
  verifier_bundle_project_digest: 8ab9d9c065af68a4f58c7892ab440c1369f6f712f78815fb99f847ccb6b6ea21
  prover_material_project_digest: f3bfbfc6ff3129df1ebf3c0dbc5ef3e52f22adde7a3de66b9c8c67a201c4e954
  activation_start_height: 1
  activation_end_height: 5
operational_security:
  q_v_max_verifier_attempts: 1048576
  n_max_cumulative_nova_steps: 4294967296
limits:
  setup: { latency_ms: 30000, rss_bytes: 12884901888 }
  fold_per_step: { latency_us: 1500000, rss_bytes: 12884901888 }
  compression_setup: { latency_ms: 10000, rss_bytes: 12884901888 }
  compression_prove: { latency_ms: 60000, rss_bytes: 12884901888 }
  verifier_cold_load: { latency_ms: 60000, rss_bytes: 4294967296 }
  verifier_check: { latency_ms: 15000, rss_bytes: 4294967296 }
  complete_prover: { latency_ms: 3600000, rss_bytes: 12884901888 }
  native_evaluator_rss_bytes: 1073741824
  artifact_bytes:
    pp: 1073741824
    pk: 1048576
    vk_decoded: 1073741824
    vk_encoded: 67108864
    verifier_bundle: 67108864
    proof: 131072
    envelope: 524288
    recovery: 1073741824
policy:
  cancellation_deadline_ms: 5000
  hard_kill_after_ms: 15000
  core_dump_policy: FORBIDDEN_RLIMIT_CORE_ZERO
  non_dumpable_policy: REQUIRED_FAIL_CLOSED
  private_directory_mode: "0700"
  private_file_mode: "0600"
  no_candidate: BLOCK_T2_REQUIRE_AMENDMENT
  f12_resident_envelope: ACCEPT_FINAL_MEASURED_STREAMING_BOUND
  f23_redb_boundary_equivalence: ACCEPT_OWNED_BOUNDARY_EQUIVALENCE_WITH_REDB_RESIDUAL
  f24_dependency_memory_residual: ACCEPT_DEPENDENCY_RESIDUAL
approval:
  decision: APPROVE_SELECTED_K1_STREAMING_BUDGET
  authority_identity: repository-authority/user-session
  decision_reference: phase-069-t2-interactive-authority-2026-07-20
  signature_or_attestation: explicit-interactive-authority-instruction
  decided_at: 2026-07-20T02:32:00+03:00
artifact_rotation:
  reason: format-4 deterministic-key-omitting verifier wire and final source identity
  selected_budget_changed: false
  active_bundle_bytes: 15372615
  active_bundle_raw_sha256: 86da72808877492cf73bb5ac3e0878abfd8c97ecbe9e91b2a0efb3d6d68fdf38
  active_bundle_role_framed_digest: d76c545b57d2cffec8e2e2564241858d5ab66d91ea537ed18306461eaab0e1ff
  active_prover_material_raw_sha256: c449daa46d2522acfa9456a02c37e341edd4fca53483da39b9dfafd831f298cb
  evidence: crates/z00z_storage/outputs/checkpoint/069-051/final/artifacts/source-1da-active-material
```

## Binding and interpretation

`in_flight_result_bytes` is exactly
`2 × hjmt_threads × segment_cap_bytes`. The 64 MiB input/snapshot reservation
is separate and cannot be relabelled as result capacity. The selected 1 MiB
segment, one HJMT worker, one Nova prover and `k=1` are compile-time/profile
authority; no runtime candidate selector is permitted.

The 1,555,200-block challenge window is exactly 90 days at a five-second block
interval. It does not select Nova proof retention: Plan 06 owns accumulator
recovery/compression/publication cadence, while Plan 09 owns proof-body
retention policy. `q_v_max_verifier_attempts` is operational verifier admission
and is separate from the theoretical uniqueness-analysis parameter `q_U`.

`artifact_bytes.recovery: 1 GiB` is a provisional single-object admission
ceiling, not an activated hot-recovery allocation. ConfigV3 intentionally keeps
`max_nova_hot_recovery_bytes: 0`; Plan 06 must measure the real snapshot codec
and activate a positive finite hot-set cap before production recovery can run.

The exact final measurements, artifact sizes, headroom arithmetic and accepted
clean-verifier report are retained in `069-051-BENCHMARKS.md` and under
`crates/z00z_storage/outputs/checkpoint/069-051/final/`. The active bundle is
`15,372,615 B`, leaving `51,736,249 B` below its unchanged 64-MiB limit.
The final-source cold verifier passed in `58.343 s` at
`3,348,504,576 B` kernel peak HWM, leaving `946,462,720 B` below 4 GiB.
Because this is one sample with only `1.657 s` latency margin, it satisfies the
current evidence gate but does not establish a production p95/p99 SLO.

Any change to an identity above, candidate space, selected tuple, resource
limit, residual disposition or lifecycle policy requires a new authority
generation and a complete release/review/doublecheck rerun. Measurements alone
cannot amend this decision.
