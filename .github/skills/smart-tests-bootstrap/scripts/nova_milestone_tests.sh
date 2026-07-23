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
PHASE069_OUTPUT_ROOT="$ROOT_DIR/crates/z00z_storage/outputs/checkpoint"
T3_ARTIFACT_DIR="$(realpath -m -- "${Z00Z_NOVA_T3_ARTIFACT_DIR_V2:-$PHASE069_OUTPUT_ROOT/recursive-v2-current}")"
case "$T3_ARTIFACT_DIR" in
  "$PHASE069_OUTPUT_ROOT" | "$PHASE069_OUTPUT_ROOT"/*) ;;
  *)
    printf 'Phase 069 output path must stay under %s: %s\n' \
      "$PHASE069_OUTPUT_ROOT" "$T3_ARTIFACT_DIR" >&2
    exit 1
    ;;
esac

SEMANTIC_TESTS=(
  test_settlement_sha_jobs_bind
  test_finalize_roots_decode_window
  test_jmt_header_binds_root
  test_jmt_machine_accepts_mutations
  test_hierarchy_r1cs_definition
  test_checkpoint_commitments_bind_fields
  test_jmt_machine_rejects_mutations
  test_jmt_framing_ordered_counted
  test_hash_controls_bind_fips
  test_replay_grammar_rejects_prefix
  test_op_kind_replay_independent
  test_output_put_trace
  test_replay_terminal_binds_object
  test_replay_output_switches_prefix
  test_precommit_allows_delete_replay
  test_uniqueness_precommit_binds_count
  test_uniqueness_row_version_bound
  test_uniqueness_rows_bind_cardinality
  test_uniqueness_challenge_binds_precommit
  test_net_merge_streams_source
  test_net_mutations_map_jmt
  test_hash_controls_reject_mutations
  test_trace_chunk_binds_contexts
  test_source_window_binding
  test_hash_shape_matches_schedule
  test_successor_erases_cursors
  test_successor_rejects_opcode_change
  test_nova_profile_identifies_owners
  test_sha_preflight_uses_sizes
  test_done_cell_rejects_nonboolean
  test_opcodes_use_fixed_shape
  test_source_record_rejects_second
  test_source_stage_rejects_control
  test_source_record_requires_context
  test_final_source_requires_closure
  test_trace_end_is_terminal
)

ARTIFACT_TESTS=(
  test_prover_material_rejects_substitution
  test_nova_bundle_verifies_proof
  test_nova_proof_binds_event
)

run_unit_exact() {
  local test_name="$1"
  shift
  cargo test --release -p z00z_storage --lib "${OWNER}${test_name}" -- \
    --exact --nocapture --test-threads 1 "$@"
}

run_storage_exact() {
  local test_name="$1"
  shift
  cargo test --release -p z00z_storage --lib "$test_name" -- \
    --exact --nocapture --test-threads 1 "$@"
}

run_guards() {
  local contract dollar='$'
  local -a verifier_rss_contract=(
    'readonly VERIFIER_MARKER="Z00Z_NOVA_VERIFIER_ONLY_V2=1"'
    'readonly EXPECTED_SOURCE_REVISION="0aaf33303b5e7a9aef957ccdb855525fdb6d8ce82c0f5101828580f5edd55332"'
    'readonly EXPECTED_WORKER_SOURCE="a0fd346405c1f3d103d62b7d7b886574ad50d58dd749fcea22f8bf22960ade69"'
    'readonly EXPECTED_NOVA_SHA256="bc6f2482f4d66fc2806da9416448fe61fd0d94d686d10624f95098ef116b8073"'
    'readonly EXPECTED_CARGO_LOCK_SHA256="dc39936ae850926a973d884ba4571eefb4be4f56e68ba459b914ec633b7f85ca"'
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

testcs = "test_mixed_fixture_satisfies_testcs"
proof = "test_nova_checkpoint_proves_relation"
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
    r'(?P<attrs>(?:\s*#\[[^\n]+\]\n)+)\s*fn test_nova_mutation_smoke\(',
    source,
)
if smoke is None or "ignore" in smoke.group("attrs"):
    raise SystemExit("canonical+mutation R1CS smoke must exist and remain unignored")
if source.count("fn test_nova_mutation_smoke(") != 1:
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
    test_verifier_identity_binds_path \
    test_nova_backend_owner_locked \
    test_nova_dependency_transcript_pinned \
    test_nova_poseidon_wires_pinned \
    test_nova_pasta_identity_pinned \
    test_nova_keccak_transcript_pinned \
    test_nova_mutation_smoke
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
    run_ignored_exact test_mixed_fixture_satisfies_testcs
    ;;
  proof)
    run_guards
    echo "=== milestone fresh full proof + independently recomputed Model C ==="
    run_ignored_exact test_nova_checkpoint_proves_relation
    ;;
  artifacts)
    run_guards
    echo "=== milestone real-Nova artifact corpus: ${#ARTIFACT_TESTS[@]} exact ignored tests ==="
    mkdir -p "$T3_ARTIFACT_DIR"
    for artifact in prover-material.bin verifier-bundle.bin; do
      if [[ -e "$T3_ARTIFACT_DIR/$artifact" ]]; then
        command -v gio >/dev/null 2>&1 || {
          echo "gio is required to retire an existing artifact safely" >&2
          exit 1
        }
        gio trash "$T3_ARTIFACT_DIR/$artifact"
      fi
    done
    export Z00Z_NOVA_T3_ARTIFACT_DIR_V2="$T3_ARTIFACT_DIR"
    for test_name in "${ARTIFACT_TESTS[@]}"; do
      run_ignored_exact "$test_name"
    done
    ;;
  t3-chain)
    run_guards
    for artifact in prover-material.bin verifier-bundle.bin; do
      if [[ ! -f "$T3_ARTIFACT_DIR/$artifact" ]]; then
        echo "missing retained T3 artifact: $T3_ARTIFACT_DIR/$artifact; run '$0 artifacts' first" >&2
        exit 1
      fi
    done
    export Z00Z_NOVA_T3_ARTIFACT_DIR_V2="$T3_ARTIFACT_DIR"
    echo "=== milestone real continuous same-z0 1/3/5 proof + public receipt ==="
    run_storage_exact \
      checkpoint::adapter::tests::test_real_chain_public_receipt \
      --ignored
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
    "$0" t3-chain
    ;;
  *)
    echo "usage: $0 {guards|curated|semantic|testcs|proof|artifacts|t3-chain|verifier-rss|all}" >&2
    exit 64
    ;;
esac
