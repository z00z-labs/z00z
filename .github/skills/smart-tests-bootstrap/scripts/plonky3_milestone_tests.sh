#!/usr/bin/env bash
# Phase-069 Plan-08 release-only verification pyramid.
#
# Semantic tests stay in-process and bounded. Every mode that performs real
# Plonky3 proving delegates exactly one named test to the isolated resource
# worker; there is intentionally no mode that chains heavyweight prover tests.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT_DIR"

readonly THREADS="${PLAN08_TEST_THREADS:-8}"
readonly SEMANTIC_TARGET_SECONDS="${PLONKY3_SEMANTIC_TARGET_SECONDS:-10}"
readonly SEMANTIC_BUDGET_SECONDS="${PLONKY3_SEMANTIC_BUDGET_SECONDS:-30}"
readonly DIRECT_TABLE_TARGET_SECONDS=10
readonly DIRECT_TABLE_BUDGET_SECONDS=120
readonly BOUNDED_EPOCH_BUDGET_SECONDS=900
readonly EXACT_EPOCH_BUDGET_SECONDS=7200
[[ "$THREADS" =~ ^[1-8]$ ]] || {
  printf 'PLAN08_TEST_THREADS must be in the inclusive range 1..8\n' >&2
  exit 2
}
readonly OUTPUT_ROOT="$ROOT_DIR/crates/z00z_storage/outputs/checkpoint"
readonly CACHE_ROOT="$ROOT_DIR/.cache/phase-069/plan-08/cargo-release"
readonly WORKER="./.github/skills/smart-tests-bootstrap/scripts/plonky3_resource_worker.sh"
readonly SOURCE_AUTHORITY="./.github/skills/smart-tests-bootstrap/scripts/bootstrap_source_authority.sh"
readonly PLAN=".planning/phases/069-Recursive-Proof/069-08-PLAN.md"
RUN_ID="$(date -u +'%Y%m%dT%H%M%S%NZ')"
readonly RUN_ID

export CARGO_TARGET_DIR="$CACHE_ROOT/library-test"
export CARGO_BUILD_JOBS=1
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=64
export Z00Z_PLONKY3_RESOURCE_PHASE=069-08

readonly -a SEMANTIC_TARGETS=(
  test_recursive_epoch
  test_recursive_history
  test_recursive_v2_plonky3_epoch
  test_recursive_v2_plonky3_history
)

usage() {
  printf 'usage: %s {guards|selection|semantic|preflight|trace-table|packed-table|typed-table|transition-batch|transition-batch-chunk|sha-table|jmt-table|bounded-epoch|cache-authority|exact-2000}\n' \
    "${0##*/}"
}

check_output_scope() {
  if [[ -e "$ROOT_DIR/test-results" ]]; then
    printf '%s\n' \
      "forbidden repository-root test-results path exists; use crates/z00z_storage/outputs/checkpoint" >&2
    return 1
  fi
  case "$CARGO_TARGET_DIR" in
    "$CACHE_ROOT/"*) ;;
    *)
      printf 'Cargo cache escaped canonical repository .cache authority: %s\n' \
        "$CARGO_TARGET_DIR" >&2
      return 1
      ;;
  esac
}

source_digest() {
  {
    find crates/z00z_storage/src/checkpoint -maxdepth 1 -type f \
      \( -name '*.rs' -o -name '*.yaml' -o -name '*.txt' \) -print0 |
      sort -z |
      xargs -0 sha256sum
    for target in "${SEMANTIC_TARGETS[@]}"; do
      sha256sum "crates/z00z_storage/tests/$target.rs"
    done
    sha256sum "$WORKER" "$PLAN" "${BASH_SOURCE[0]}"
    sha256sum Cargo.toml Cargo.lock crates/z00z_storage/Cargo.toml
  } | sha256sum | awk '{print $1}'
}

append_unique() {
  local value="$1" item
  shift
  for item in "$@"; do
    [[ "$item" != "$value" ]] || return 0
  done
  printf '%s\n' "$value"
}

latest_accepted_bootstrap_manifest() {
  local result candidate
  while IFS= read -r result; do
    jq -e \
      '.status == "pass"
      and .tier == "bootstrap-fast"
      and .source_stability.status == "stable"' \
      "$result" >/dev/null 2>&1 || continue
    candidate="$(dirname "$result")/source-manifest-baseline.tsv"
    [[ -s "$candidate" ]] || continue
    printf '%s\n' "$candidate"
    return 0
  done < <(
    find "$OUTPUT_ROOT/069-08/task-1/test-pyramid/bootstrap" \
      -mindepth 2 -maxdepth 2 -type f -name result.json 2>/dev/null |
      LC_ALL=C sort -r
  )
  return 1
}

run_selection() {
  local run_dir current_manifest previous_manifest comparison
  local current_digest previous_digest=null compare_status=0 routing_rule=stable
  local changed_json='[]' path cache_marker_digest=null preflight_digest=null
  local host_boot_id command_digest
  local select_trace=false select_packed=false select_typed=false
  local select_transition_batch=false
  local select_sha=false select_jmt=false select_bounded=false select_all=false
  local -a selected=(bootstrap-fast plonky3-semantic)
  local -a deferred=()

  check_output_scope
  [[ -x "$SOURCE_AUTHORITY" ]] || {
    printf 'missing executable source authority: %s\n' "$SOURCE_AUTHORITY" >&2
    return 1
  }
  run_dir="$OUTPUT_ROOT/069-08/task-1/test-pyramid/selection/$RUN_ID"
  mkdir -p "$run_dir"
  current_manifest="$run_dir/source-manifest-current.tsv"
  comparison="$run_dir/source-comparison.json"
  "$SOURCE_AUTHORITY" manifest >"$current_manifest"
  current_digest="$(sha256sum "$current_manifest" | awk '{print $1}')"

  if previous_manifest="$(latest_accepted_bootstrap_manifest)"; then
    previous_digest="$(
      sha256sum "$previous_manifest" |
        awk '{print $1}' |
        jq -R .
    )"
    set +e
    "$SOURCE_AUTHORITY" compare \
      "$previous_manifest" "$current_manifest" >"$comparison"
    compare_status=$?
    set -e
    if (( compare_status != 0 && compare_status != 86 )); then
      printf 'source comparison failed with status %s\n' "$compare_status" >&2
      return "$compare_status"
    fi
    changed_json="$(jq -c '.changed_paths' "$comparison")"
  else
    routing_rule=initial_wide
    select_all=true
    select_bounded=true
    jq -n -S \
      --arg current_digest "$current_digest" \
      '{
        status: "no_accepted_baseline",
        before_digest: null,
        after_digest: $current_digest,
        changed_paths: [{
          change: "unknown",
          path: "<no-accepted-bootstrap-baseline>",
          before_sha256: null,
          after_sha256: null
        }]
      }' >"$comparison"
    changed_json="$(jq -c '.changed_paths' "$comparison")"
  fi

  while IFS= read -r path; do
    case "$path" in
      "<no-accepted-bootstrap-baseline>")
        select_all=true
        select_bounded=true
        routing_rule=initial_wide
        ;;
      Cargo.toml | Cargo.lock | .cargo/config.toml | \
        .github/skills/smart-tests-bootstrap/scripts/plonky3_resource_worker.sh | \
        crates/z00z_storage/Cargo.toml | \
        crates/z00z_storage/src/checkpoint/plonky3.rs | \
        crates/z00z_storage/src/checkpoint/plonky3_binary_hash.rs | \
        crates/z00z_storage/src/checkpoint/plonky3_binary_mmcs.rs | \
        crates/z00z_storage/src/checkpoint/plonky3_recursion.rs | \
        crates/z00z_storage/src/checkpoint/version_registry.rs | \
        crates/z00z_storage/src/checkpoint/authority_artifacts.rs | \
        crates/z00z_storage/src/checkpoint/contract_config*.rs | \
        crates/z00z_storage/src/checkpoint/checkpoint_contract*.yaml)
        select_all=true
        select_bounded=true
        routing_rule=shared_wide
        ;;
      *plonky3_epoch_trace_framing* | *recursive_trace*)
        select_trace=true
        [[ "$routing_rule" != stable ]] || routing_rule=affected_tables
        ;;
      *plonky3_epoch_packed_range* | *plonky3_u16_range*)
        select_packed=true
        [[ "$routing_rule" != stable ]] || routing_rule=affected_tables
        ;;
      *plonky3_epoch_typed_commitment*)
        select_typed=true
        select_transition_batch=true
        [[ "$routing_rule" != stable ]] || routing_rule=affected_tables
        ;;
      *plonky3_epoch_transition_* | *plonky3_epoch_uniqueness_*)
        select_transition_batch=true
        [[ "$routing_rule" != stable ]] || routing_rule=affected_tables
        ;;
      *plonky3_epoch_sha256* | *sha256_hash* | *hash/domains*)
        select_sha=true
        [[ "$routing_rule" != stable ]] || routing_rule=affected_tables
        ;;
      *plonky3_epoch_jmt* | *hjmt*)
        select_jmt=true
        [[ "$routing_rule" != stable ]] || routing_rule=affected_tables
        ;;
      .planning/phases/069-Recursive-Proof/* | \
        .github/skills/smart-tests-bootstrap/SKILL.md | \
        .github/skills/smart-tests-bootstrap/scripts/bootstrap_*.sh | \
        .github/skills/smart-tests-bootstrap/scripts/nova_*.sh | \
        .github/skills/smart-tests-bootstrap/scripts/plonky3_milestone_tests.sh)
        [[ "$routing_rule" != stable ]] || routing_rule=runner_or_authority_only
        ;;
      crates/z00z_storage/src/checkpoint/epoch_* | \
        crates/z00z_storage/src/checkpoint/history_* | \
        crates/z00z_storage/src/checkpoint/recursive_v2.rs | \
        crates/z00z_storage/src/checkpoint/receipt.rs | \
        crates/z00z_storage/tests/test_recursive_epoch.rs | \
        crates/z00z_storage/tests/test_recursive_history.rs | \
        crates/z00z_storage/tests/test_recursive_v2_plonky3_epoch.rs | \
        crates/z00z_storage/tests/test_recursive_v2_plonky3_history.rs)
        select_all=true
        select_bounded=true
        routing_rule=recursive_relation_wide
        ;;
      *)
        select_all=true
        select_bounded=true
        routing_rule=ambiguous_wide
        ;;
    esac
  done < <(jq -r '.[].path' <<<"$changed_json")

  if [[ "$select_all" == true ]]; then
    select_trace=true
    select_packed=true
    select_typed=true
    select_transition_batch=true
    select_sha=true
    select_jmt=true
  fi
  [[ "$select_trace" == false ]] ||
    selected+=("$(append_unique trace-table "${selected[@]}")")
  [[ "$select_packed" == false ]] ||
    selected+=("$(append_unique packed-table "${selected[@]}")")
  [[ "$select_typed" == false ]] ||
    selected+=("$(append_unique typed-table "${selected[@]}")")
  [[ "$select_transition_batch" == false ]] ||
    selected+=("$(append_unique transition-batch "${selected[@]}")")
  [[ "$select_transition_batch" == false ]] ||
    selected+=("$(append_unique transition-batch-chunk "${selected[@]}")")
  [[ "$select_sha" == false ]] ||
    selected+=("$(append_unique sha-table "${selected[@]}")")
  [[ "$select_jmt" == false ]] ||
    selected+=("$(append_unique jmt-table "${selected[@]}")")
  [[ "$select_bounded" == false ]] ||
    selected+=("$(append_unique bounded-epoch "${selected[@]}")")

  for path in trace-table packed-table typed-table transition-batch transition-batch-chunk sha-table jmt-table bounded-epoch; do
    if ! printf '%s\n' "${selected[@]}" | grep -Fqx "$path"; then
      deferred+=("$path:changed-surface-real")
    fi
  done
  deferred+=(
    "exact-2000:final-acceptance"
    "cargo-test-release:release-closure"
    "three-yolo-reviews:release-closure"
    "two-doublechecks:release-closure"
  )

  if [[ -s "$CACHE_ROOT/.z00z-bootstrap-cache-v3" ]]; then
    cache_marker_digest="$(
      sha256sum "$CACHE_ROOT/.z00z-bootstrap-cache-v3" |
        awk '{print $1}' |
        jq -R .
    )"
  fi
  if [[ -s "$OUTPUT_ROOT/069-08/task-1/resource-worker/preflight-latest.json" ]]; then
    preflight_digest="$(
      sha256sum \
        "$OUTPUT_ROOT/069-08/task-1/resource-worker/preflight-latest.json" |
        awk '{print $1}' |
        jq -R .
    )"
  fi
  host_boot_id="$(</proc/sys/kernel/random/boot_id)"
  command_digest="$({
    printf '%s\n' \
      "$SOURCE_AUTHORITY manifest" \
      "$current_digest" \
      "$(sha256sum "${BASH_SOURCE[0]}" "$PLAN" | sha256sum)"
  } | sha256sum | awk '{print $1}')"

  jq -n -S \
    --arg schema "z00z.phase069.test-pyramid.selection.v1" \
    --arg recorded_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
    --arg routing_rule "$routing_rule" \
    --arg current_digest "$current_digest" \
    --argjson previous_digest "$previous_digest" \
    --arg current_manifest "$current_manifest" \
    --arg previous_manifest "${previous_manifest:-}" \
    --arg cache_root "$CACHE_ROOT" \
    --argjson cache_marker_digest "$cache_marker_digest" \
    --arg host_boot_id "$host_boot_id" \
    --argjson preflight_digest "$preflight_digest" \
    --arg command_digest "$command_digest" \
    --argjson changed_paths "$changed_json" \
    --argjson selected "$(
      printf '%s\n' "${selected[@]}" | jq -R . | jq -s .
    )" \
    --argjson deferred "$(
      printf '%s\n' "${deferred[@]}" |
        jq -R 'split(":") | {target: .[0], mandatory_tier: .[1]}' |
        jq -s .
    )" \
    '{
      schema: $schema,
      recorded_at: $recorded_at,
      status: "selected",
      routing_rule: $routing_rule,
      identity: {
        previous_source_digest: $previous_digest,
        current_source_digest: $current_digest,
        previous_source_manifest: (
          if $previous_manifest == "" then null else $previous_manifest end
        ),
        current_source_manifest: $current_manifest,
        command_digest: $command_digest
      },
      changed_paths: $changed_paths,
      selected_targets: $selected,
      deferred_targets: $deferred,
      cache_identity: {
        root: $cache_root,
        marker_digest: $cache_marker_digest
      },
      preflight_identity: {
        host_boot_id: $host_boot_id,
        evidence_digest: $preflight_digest
      },
      budgets: {
        bootstrap_warm_seconds: 12,
        bootstrap_hard_seconds: 60,
        semantic_warm_seconds: 10,
        semantic_hard_seconds: 30,
        direct_table_warm_seconds: 10,
        direct_table_hard_seconds: 120,
        bounded_epoch_hard_seconds: 900,
        exact_2000_hard_seconds: 7200
      },
      promotion_authority: false
    }' >"$run_dir/selection.json"
  printf 'selection evidence: %s/selection.json\n' "$run_dir"
  jq -r \
    '"routing=" + .routing_rule
    + " selected=" + (.selected_targets | join(","))' \
    "$run_dir/selection.json"
}

check_worker_test() {
  local source="$1" test_name="$2"
  if ! rg -Uq \
    "#\\[ignore = \"[^\"]*plonky3_resource_worker\\.sh\"\\][[:space:]]*fn ${test_name}\\(" \
    "$source"; then
    printf 'real prover test is absent or not unconditionally worker-only: %s\n' \
      "$test_name" >&2
    return 1
  fi
}

run_guards() {
  local cargo_pattern cargo_violations contract
  local -a contracts=(
    "crates/z00z_storage/src/checkpoint/epoch_prover.rs:test_direct_trace_framing_actual_roundtrip"
    "crates/z00z_storage/src/checkpoint/epoch_prover.rs:test_direct_packed_range_actual_roundtrip"
    "crates/z00z_storage/src/checkpoint/epoch_prover.rs:test_direct_sha256_actual_roundtrip"
    "crates/z00z_storage/src/checkpoint/epoch_prover.rs:test_direct_jmt_actual_roundtrip"
    "crates/z00z_storage/tests/test_recursive_v2_plonky3_base.rs:test_direct_typed_commitment_actual_roundtrip"
    "crates/z00z_storage/tests/test_recursive_v2_plonky3_base.rs:test_direct_transition_batch_actual_roundtrip"
    "crates/z00z_storage/tests/test_recursive_v2_plonky3_base.rs:test_direct_transition_batch_actual_eight_transition_roundtrip"
    "crates/z00z_storage/tests/test_recursive_v2_plonky3_base.rs:test_production_epoch_2000_actual_recursion_step"
  )

  check_output_scope
  [[ -x "$WORKER" ]] || {
    printf 'missing executable resource worker: %s\n' "$WORKER" >&2
    return 1
  }
  for contract in "${contracts[@]}"; do
    local source="${contract%%:*}" test_name="${contract#*:}"
    check_worker_test "$source" "$test_name"
  done
  cargo_pattern="cargo"" test "
  cargo_violations="$(
    rg -nF "$cargo_pattern" "${BASH_SOURCE[0]}" "$WORKER" |
      grep -Fv 'cargo test --release ' || true
  )"
  if [[ -n "$cargo_violations" ]]; then
    printf '%s\n' "$cargo_violations" >&2
    printf '%s\n' "Plan-08 pyramid contains a non-release Cargo invocation" >&2
    return 1
  fi
  printf '%s\n' \
    "Plan-08 pyramid guards: release-only=1 worker-only-real-prover=8 cache=.cache root-test-results=absent"
}

write_semantic_result() {
  local run_dir="$1" status="$2" reason="$3" wall_ms="$4"
  local digest_before="$5" digest_after="$6"
  jq -n -S \
    --arg schema "z00z.phase069.test-pyramid.v1" \
    --arg recorded_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
    --arg status "$status" \
    --arg reason "$reason" \
    --arg source_digest_before "$digest_before" \
    --arg source_digest_after "$digest_after" \
    --arg cargo_target_dir "$CARGO_TARGET_DIR" \
    --argjson wall_ms "$wall_ms" \
    --argjson target_seconds "$SEMANTIC_TARGET_SECONDS" \
    --argjson budget_seconds "$SEMANTIC_BUDGET_SECONDS" \
    --argjson targets "$(
      printf '%s\n' "${SEMANTIC_TARGETS[@]}" | jq -R . | jq -s .
    )" \
    '{
      schema: $schema,
      recorded_at: $recorded_at,
      tier: "plonky3-semantic",
      status: $status,
      reason: $reason,
      identity: {
        source_digest: $source_digest_before,
        source_digest_before: $source_digest_before,
        source_digest_after: $source_digest_after
      },
      source_stability: {
        status: (
          if $source_digest_before == $source_digest_after
          then "stable"
          else "source_drift"
          end
        )
      },
      profile: "release",
      cargo_target_dir: $cargo_target_dir,
      budget: {
        warm_target_seconds: $target_seconds,
        hard_wall_seconds: $budget_seconds
      },
      resources: {
        wall_ms: $wall_ms,
        warm_target_met: ($wall_ms <= ($target_seconds * 1000))
      },
      selected_targets: $targets,
      explicitly_deferred: [
        "real trace-framing table proof",
        "real packed-range table proof",
        "real typed-commitment table proof",
        "real SHA-256 table proof",
        "real JMT update table proof",
        "real bounded epoch recursion",
        "real exact-2000 epoch/history acceptance",
        "broad workspace release suite"
      ],
      acceptance_authority: false
    }' >"$run_dir/result.json"
}

run_semantic() {
  local run_dir start_ns end_ns wall_ms status digest_before digest_after reason
  run_dir="$OUTPUT_ROOT/069-08/task-1/test-pyramid/semantic/$RUN_ID"
  mkdir -p "$run_dir"
  digest_before="$(source_digest)"
  start_ns="$(date +%s%N)"
  set +e
  timeout --signal=TERM --kill-after=5s "$SEMANTIC_BUDGET_SECONDS" \
    cargo test --release --locked --offline -p z00z_storage \
    --test test_recursive_epoch \
    --test test_recursive_history \
    --test test_recursive_v2_plonky3_epoch \
    --test test_recursive_v2_plonky3_history \
    -- --nocapture --test-threads "$THREADS" \
    > >(tee "$run_dir/test.log") 2>&1
  status=$?
  set -e
  end_ns="$(date +%s%N)"
  wall_ms=$(((end_ns - start_ns) / 1000000))
  digest_after="$(source_digest)"
  reason="success"
  if (( status != 0 )); then
    if (( status == 124 )); then
      reason="semantic hard wall budget exceeded"
    else
      reason="semantic test failure"
    fi
  elif [[ "$digest_before" != "$digest_after" ]]; then
    status=86
    reason="source_drift"
  elif (( wall_ms > SEMANTIC_TARGET_SECONDS * 1000 )); then
    status=75
    reason="semantic warm promotion budget exceeded"
  fi
  check_output_scope || {
    status=1
    reason="forbidden root test-results path"
  }
  if (( status == 0 )); then
    write_semantic_result \
      "$run_dir" pass "$reason" "$wall_ms" "$digest_before" "$digest_after"
  else
    write_semantic_result \
      "$run_dir" fail "$reason" "$wall_ms" "$digest_before" "$digest_after"
  fi
  printf 'evidence: %s/result.json\n' "$run_dir"
  return "$status"
}

ensure_preflight() {
  local evidence_root latest worker_digest host_boot_id
  evidence_root="$OUTPUT_ROOT/069-08/task-1/resource-worker"
  latest="$evidence_root/preflight-latest.json"
  worker_digest="$(sha256sum "$WORKER" | awk '{print $1}')"
  host_boot_id="$(</proc/sys/kernel/random/boot_id)"

  if [[ -s "$latest" ]] &&
    jq -e \
      --arg worker_digest "$worker_digest" \
      --arg host_boot_id "$host_boot_id" \
      '
        .exit_reason == "success"
        and .worker_digest == $worker_digest
        and .host_boot_id == $host_boot_id
        and .controls.oom_policy == "continue"
        and .controls.kill_mode == "control-group"
        and .controls.memory_high_bytes == 17179869184
        and .controls.memory_max_bytes == 25769803776
        and .controls.memory_swap_max_bytes == 0
        and .controls.memory_oom_group == 0
      ' "$latest" >/dev/null; then
    printf 'reused current-boot isolation preflight: %s\n' "$latest"
    return 0
  fi
  "$WORKER" --preflight
}

write_real_promotion_result() {
  local run_dir="$1" test_name="$2" status="$3" reason="$4"
  local target_seconds="$5" budget_seconds="$6" evidence_path="$7"
  local evidence_json=null wall_ms=0 tier=changed-table-real
  case "$test_name" in
    test_bounded_epoch_two_leaf_actual_recursion)
      tier=bounded-epoch-real
      ;;
    test_production_epoch_2000_actual_recursion_step)
      tier=exact-2000
      ;;
    test_recursive_cache_authority_inventory)
      tier=cache-authority-real
      ;;
  esac
  if [[ -s "$evidence_path" ]]; then
    evidence_json="$(<"$evidence_path")"
    wall_ms="$(jq -r '.resources.wall_time_ms // 0' "$evidence_path")"
  fi
  jq -n -S \
    --arg schema "z00z.phase069.test-pyramid.real-promotion.v1" \
    --arg recorded_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
    --arg status "$status" \
    --arg reason "$reason" \
    --arg tier "$tier" \
    --arg test_name "$test_name" \
    --arg source_digest "$(source_digest)" \
    --arg evidence_path "$evidence_path" \
    --argjson target_seconds "$target_seconds" \
    --argjson budget_seconds "$budget_seconds" \
    --argjson wall_ms "$wall_ms" \
    --argjson resource_evidence "$evidence_json" \
    '{
      schema: $schema,
      recorded_at: $recorded_at,
      tier: $tier,
      status: $status,
      reason: $reason,
      profile: "release",
      source_digest: $source_digest,
      selected_target: $test_name,
      budget: {
        warm_target_seconds: (
          if $target_seconds == 0 then null else $target_seconds end
        ),
        hard_wall_seconds: $budget_seconds
      },
      resources: {
        wall_ms: $wall_ms,
        warm_target_met: (
          $target_seconds == 0 or $wall_ms <= ($target_seconds * 1000)
        )
      },
      resource_evidence_path: $evidence_path,
      resource_evidence: $resource_evidence,
      acceptance_authority: false
    }' >"$run_dir/promotion.json"
}

run_isolated() {
  local test_name="$1" target_seconds="$2" budget_seconds="$3"
  local run_dir worker_status evidence_path reason status
  check_output_scope
  ensure_preflight
  run_dir="$OUTPUT_ROOT/069-08/task-1/test-pyramid/real/$RUN_ID/$test_name"
  mkdir -p "$run_dir"
  set +e
  "$WORKER" --run "$test_name" > >(tee "$run_dir/worker.log") 2>&1
  worker_status=$?
  set -e
  evidence_path="$(
    sed -n \
      "\|^$OUTPUT_ROOT/069-08/task-1/resource-worker/|p" \
      "$run_dir/worker.log" |
      tail -n 1
  )"
  if [[ -d "$evidence_path" ]]; then
    evidence_path="$evidence_path/resource-evidence.json"
  fi
  status="$worker_status"
  reason="worker failure"
  if (( worker_status == 0 )) &&
    [[ -s "$evidence_path" ]] &&
    jq -e \
      --arg test_name "$test_name" \
      '.exit_reason == "success" and .command.test == $test_name' \
      "$evidence_path" >/dev/null; then
    status=0
    reason="success"
    if (( target_seconds > 0 )) &&
      (( $(jq -r '.resources.wall_time_ms' "$evidence_path") >
        target_seconds * 1000 )); then
      status=75
      reason="real-table warm promotion budget exceeded"
    fi
  elif (( worker_status == 0 )); then
    status=1
    reason="typed real-prover evidence missing or inconsistent"
  fi
  write_real_promotion_result \
    "$run_dir" "$test_name" \
    "$([[ "$status" == 0 ]] && printf pass || printf fail)" \
    "$reason" "$target_seconds" "$budget_seconds" "$evidence_path"
  printf 'promotion evidence: %s/promotion.json\n' "$run_dir"
  return "$status"
}

MODE="${1:-}"
case "$MODE" in
  guards)
    run_guards
    ;;
  selection)
    run_guards
    run_selection
    ;;
  semantic)
    run_guards
    run_semantic
    ;;
  preflight)
    run_guards
    ensure_preflight
    ;;
  trace-table)
    run_isolated \
      test_direct_trace_framing_actual_roundtrip \
      "$DIRECT_TABLE_TARGET_SECONDS" "$DIRECT_TABLE_BUDGET_SECONDS"
    ;;
  packed-table)
    run_isolated \
      test_direct_packed_range_actual_roundtrip \
      "$DIRECT_TABLE_TARGET_SECONDS" "$DIRECT_TABLE_BUDGET_SECONDS"
    ;;
  typed-table)
    run_isolated \
      test_direct_typed_commitment_actual_roundtrip \
      "$DIRECT_TABLE_TARGET_SECONDS" "$DIRECT_TABLE_BUDGET_SECONDS"
    ;;
  transition-batch)
    run_isolated \
      test_direct_transition_batch_actual_roundtrip \
      "$DIRECT_TABLE_TARGET_SECONDS" "$DIRECT_TABLE_BUDGET_SECONDS"
    ;;
  transition-batch-chunk)
    run_isolated \
      test_direct_transition_batch_actual_eight_transition_roundtrip \
      0 "$DIRECT_TABLE_BUDGET_SECONDS"
    ;;
  sha-table)
    run_isolated \
      test_direct_sha256_actual_roundtrip \
      "$DIRECT_TABLE_TARGET_SECONDS" "$DIRECT_TABLE_BUDGET_SECONDS"
    ;;
  jmt-table)
    run_isolated \
      test_direct_jmt_actual_roundtrip \
      "$DIRECT_TABLE_TARGET_SECONDS" "$DIRECT_TABLE_BUDGET_SECONDS"
    ;;
  bounded-epoch)
    check_worker_test \
      crates/z00z_storage/src/checkpoint/plonky3.rs \
      test_bounded_epoch_two_leaf_actual_recursion
    run_isolated \
      test_bounded_epoch_two_leaf_actual_recursion \
      0 "$BOUNDED_EPOCH_BUDGET_SECONDS"
    ;;
  cache-authority)
    run_isolated \
      test_recursive_cache_authority_inventory \
      0 "$EXACT_EPOCH_BUDGET_SECONDS"
    ;;
  exact-2000)
    run_isolated \
      test_production_epoch_2000_actual_recursion_step \
      0 "$EXACT_EPOCH_BUDGET_SECONDS"
    ;;
  *)
    usage >&2
    exit 64
    ;;
esac
