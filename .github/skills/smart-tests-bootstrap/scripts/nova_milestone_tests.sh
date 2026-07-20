#!/usr/bin/env bash
# Sound release-only verification pyramid for the sole Phase 069 Nova owner.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT_DIR"

THREADS="${BOOTSTRAP_THREADS:-4}"
SOURCE="crates/z00z_storage/src/checkpoint/nova.rs"
OWNER="checkpoint::nova::tests::"
VERIFIER_RSS_HARNESS=".github/skills/smart-tests-bootstrap/scripts/nova_verifier_rss_measurement.sh"
COVERAGE_AUDIT=".planning/phases/069-Recursive-Proof/069-COVERAGE-AUDIT.py"

SEMANTIC_TESTS=(
  settlement_root_sha_jobs_bind_policy_layout_definition_and_finalize_post_root
  finalize_settlement_roots_are_decoded_from_the_canonical_r1cs_window
  jmt_header_and_promoted_root_are_bound_directly_in_r1cs
  jmt_new_root_machine_accepts_all_native_mutation_cases
  hierarchy_r1cs_consumes_canonical_roles_parent_values_and_definition_root
  typed_checkpoint_commitments_bind_x_h_fields_in_canonical_order
  jmt_new_root_machine_rejects_authenticated_transcript_mutations
  jmt_micro_operation_framing_is_ordered_and_counted_in_r1cs
  canonical_hash_controls_bind_the_fixed_fips_schedule
  replay_grammar_rejects_input_after_output_prefix
  test_op_kind_replay_independent
  test_output_put_trace
  replay_payload_terminal_is_bound_to_the_source_object_id_in_r1cs
  replay_output_switches_the_exact_canonical_replay_prefix
  precommit_allows_a_delete_only_replay_set
  uniqueness_precommit_payload_is_streamed_and_count_bound_in_r1cs
  uniqueness_sorted_row_version_is_constrained_from_the_same_memory_window
  uniqueness_sorted_rows_bind_order_and_precommit_cardinality_in_r1cs
  uniqueness_challenge_payload_binds_precommit_bytes_in_r1cs
  net_merge_payload_is_streamed_from_canonical_source_bytes_in_r1cs
  net_mutations_are_permuted_into_exact_terminal_jmt_operations
  canonical_hash_controls_reject_binding_and_order_mutations
  trace_chunk_payload_reaches_the_constrained_source_and_global_sha_contexts
  test_source_window_binding
  hash_control_shape_metrics_cover_the_canonical_schedule
  final_successor_erases_private_uniqueness_job_cursors
  final_successor_rejects_a_changed_declared_opcode_count
  nova_shape_profile_identifies_exact_top_level_resource_owners
  sha_lane_resource_preflight_uses_pinned_wire_and_pedersen_sizes
  non_boolean_done_cell_is_unsatisfied
  every_opcode_uses_one_fixed_shape
  source_record_rejects_a_second_record_before_hash_completion
  source_stage_cannot_masquerade_as_a_hash_control
  source_record_requires_a_live_global_trace_context
  final_source_record_requires_global_hash_closure
  schema_bound_trace_end_is_the_only_trace_closure_terminal_edge
)

ARTIFACT_TESTS=(
  prover_material_roundtrip_rejects_identity_length_payload_and_key_substitution
  real_nova_verifier_bundle_loads_and_verifies_compressed_proof
  real_nova_proof_binds_one_source_event_after_trace_begin
)

run_unit_exact() {
  local test_name="$1"
  shift
  cargo test --release -p z00z_storage --lib "${OWNER}${test_name}" -- \
    --exact --nocapture --test-threads 1 "$@"
}

run_guards() {
  local contract dollar='$'
  local -a verifier_rss_contract=(
    'readonly VERIFIER_MARKER="Z00Z_NOVA_VERIFIER_ONLY_V2=1"'
    'readonly EXPECTED_SOURCE_REVISION="e58e2f9a2f715a64b37dd464248b57601e7deda4254086c0b6598160cf30dbd6"'
    'readonly EXPECTED_WORKER_SOURCE="272379f7f47f735dc2536682c23e3e3d93e1434933f863f8e8841e89106d8ca0"'
    'readonly EXPECTED_NOVA_SHA256="1e39544c8c58f7d5a8117cdcdbf6ca0836e5e70e056d6c84f77e88fe1336c053"'
    'readonly EXPECTED_CARGO_LOCK_SHA256="23a86f3341579b25ad5be96080a642405633df5f8c6e99dd4c3329d7d51f2a11"'
    "for children_path in \"/proc/${dollar}pid/task/\"[0-9]*/children; do"
    "setsid env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=\"${dollar}ROOT_DIR/target/workspace\""
    "cargo test --release -p z00z_storage --lib \"${dollar}TEST_NAME\" --"
    '--exact --nocapture --test-threads 1 --ignored'
    "printf 'measurement_status=%s\\n' \"${dollar}status\""
    'printf '\''worker_rlimit_as_bytes=25769803776\n'\'''
    'printf '\''worker_timeout_seconds=3600\n'\'''
    'printf '\''process_group_cleanup=%s\n'\'''
    'printf '\''worker_lock_after=%s\n'\'''
  )

  echo "=== Nova source / owner / coverage guards ==="
  if [[ ! -x "$VERIFIER_RSS_HARNESS" ]]; then
    echo "missing executable verifier RSS harness: $VERIFIER_RSS_HARNESS" >&2
    return 1
  fi
  for contract in "${verifier_rss_contract[@]}"; do
    if ! grep -Fq -- "$contract" "$VERIFIER_RSS_HARNESS"; then
      echo "verifier RSS harness contract drifted: $contract" >&2
      return 1
    fi
  done
  "$VERIFIER_RSS_HARNESS" --check
  if [[ -f "$COVERAGE_AUDIT" ]]; then
    python3 "$COVERAGE_AUDIT"
  else
    echo "planning coverage audit unavailable in clean source distribution; source/test inventory guards remain active"
  fi
  python3 - "$SOURCE" "${SEMANTIC_TESTS[@]}" -- "${ARTIFACT_TESTS[@]}" <<'PY'
import re
import sys
from pathlib import Path

source_path = Path(sys.argv[1])
separator = sys.argv.index("--")
milestone_tests = sys.argv[2:separator] + sys.argv[separator + 1:]
if not source_path.is_file():
    raise SystemExit(f"missing canonical Nova owner: {source_path}")
source = source_path.read_text()
if (source_path.parent / "recursive_v2" / "nova.rs").exists():
    raise SystemExit("legacy recursive_v2/nova.rs owner still exists")

owner_literal = 'const NOVA_OWNER_PATH_V2: &[u8] = b"z00z_storage::checkpoint::nova";'
if source.count(owner_literal) != 1:
    raise SystemExit("canonical Nova owner literal must occur exactly once")

for test_name in milestone_tests:
    pattern = re.compile(
        rf'#\[test\]\s*#\[ignore = "[^"]*milestone-only[^"]*"\]\s*fn {re.escape(test_name)}\(',
        re.MULTILINE,
    )
    if not pattern.search(source):
        raise SystemExit(f"milestone test is absent or not unconditionally ignored: {test_name}")

ignored_tests = set(
    re.findall(
        r'#\[test\]\s*#\[ignore = "[^"]*milestone-only[^"]*"\]\s*fn ([A-Za-z0-9_]+)\(',
        source,
        re.MULTILINE,
    )
)

testcs = "complete_mixed_fixture_satisfies_every_test_cs_step"
proof = "real_nova_mixed_checkpoint_proves_the_complete_t2_relation"
for test_name in (testcs, proof):
    pattern = re.compile(
        rf'#\[test\]\s*#\[ignore = "[^"]*milestone-only[^"]*"\]\s*fn {test_name}\(',
        re.MULTILINE,
    )
    if not pattern.search(source):
        raise SystemExit(f"mandatory milestone gate is absent or not ignored: {test_name}")
expected_ignored = set(milestone_tests) | {testcs, proof}
if ignored_tests != expected_ignored:
    missing = sorted(expected_ignored - ignored_tests)
    extra = sorted(ignored_tests - expected_ignored)
    raise SystemExit(
        f"milestone ignore set drifted: missing={missing or 'none'}, extra={extra or 'none'}"
    )

smoke = re.search(
    r'(?P<attrs>(?:\s*#\[[^\n]+\]\n)+)\s*fn nova_r1cs_canonical_and_mutation_smoke\(',
    source,
)
if smoke is None or "ignore" in smoke.group("attrs"):
    raise SystemExit("canonical+mutation R1CS smoke must exist and remain unignored")
if source.count("fn nova_r1cs_canonical_and_mutation_smoke(") != 1:
    raise SystemExit("canonical+mutation R1CS smoke must have exactly one owner")

print(
    "Nova guard contract: canonical_owner=1, legacy_owner=0, "
    f"ordinary_r1cs_smoke=1, semantic_milestone={len(sys.argv[2:separator])}, "
    f"artifact_milestone={len(sys.argv[separator + 1:])}, full_testcs=1, proof_model_c=1"
)
PY

  mapfile -t nova_files < <(rg --files crates/z00z_storage/src | rg '(^|/)nova\.rs$' || true)
  if [[ "${#nova_files[@]}" -ne 1 || "${nova_files[0]}" != "$SOURCE" ]]; then
    printf 'expected exactly one Nova source owner, found: %s\n' "${nova_files[*]:-none}" >&2
    return 1
  fi
  if rg -F 'recursive_v2/nova.rs' crates/z00z_storage/src crates/z00z_storage/tests; then
    echo "legacy recursive_v2/nova.rs string remains in storage code/tests" >&2
    return 1
  fi
}

run_curated() {
  run_guards
  echo "=== curated Nova release packet: 7 source/dependency/R1CS units + 2 integration targets; features=production ==="
  for test_name in \
    verifier_source_identity_binds_nova_and_canonical_trace_owner \
    nova_backend_manifest_lock_and_private_owner_are_exact \
    nova_dependency_transcript_entropy_and_source_files_are_exact \
    nova_poseidon_constant_wires_are_pinned_for_both_cycle_fields \
    nova_pasta_identity_and_first_generator_wires_are_explicit \
    nova_pasta_keccak_transcript_is_non_evm_and_pinned \
    nova_r1cs_canonical_and_mutation_smoke
  do
    run_unit_exact "$test_name"
  done
  cargo test --release -p z00z_storage \
    --test test_recursive_v2_nova_step \
    --test test_recursive_v2_nova_adversarial \
    -- --nocapture --test-threads "$THREADS"
  echo "Nova curated release packet: PASS (7 unit gates, 2 integration targets)"
}

run_ignored_exact() {
  run_unit_exact "$1" --ignored
}

MODE="${1:-curated}"
case "$MODE" in
  guards)
    run_guards
    ;;
  curated)
    run_curated
    ;;
  semantic)
    run_guards
    echo "=== milestone semantic R1CS corpus: ${#SEMANTIC_TESTS[@]} exact ignored tests ==="
    for test_name in "${SEMANTIC_TESTS[@]}"; do
      run_ignored_exact "$test_name"
    done
    ;;
  testcs)
    run_guards
    echo "=== milestone full 1,727-step TestCS replay ==="
    run_ignored_exact complete_mixed_fixture_satisfies_every_test_cs_step
    ;;
  proof)
    run_guards
    echo "=== milestone fresh full proof + independently recomputed Model C ==="
    run_ignored_exact real_nova_mixed_checkpoint_proves_the_complete_t2_relation
    ;;
  artifacts)
    run_guards
    echo "=== milestone real-Nova artifact corpus: ${#ARTIFACT_TESTS[@]} exact ignored tests ==="
    for test_name in "${ARTIFACT_TESTS[@]}"; do
      run_ignored_exact "$test_name"
    done
    ;;
  verifier-rss)
    run_guards
    exec "$VERIFIER_RSS_HARNESS"
    ;;
  all)
    "$0" semantic
    "$0" testcs
    "$0" proof
    "$0" artifacts
    ;;
  *)
    echo "usage: $0 {guards|curated|semantic|testcs|proof|artifacts|verifier-rss|all}" >&2
    exit 64
    ;;
esac
