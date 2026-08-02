#!/usr/bin/env bash
# Phase-069 Tier-0 release-mode fail-fast gate.
#
# This default packet deliberately excludes fresh Nova proofs, heavyweight
# Plonky3 proving, wallets, benches, examples, and exact-epoch acceptance.
# Those remain mandatory in their named higher tiers.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT_DIR"

# All Phase-069 release tiers share one Cargo unit graph. Reject ambient
# overrides instead of silently compiling a second fingerprint inside the
# canonical cache.
[[ -z "${CARGO_BUILD_JOBS:-}" || "$CARGO_BUILD_JOBS" == "1" ]] || {
  echo "bootstrap requires CARGO_BUILD_JOBS=1" >&2
  exit 2
}
[[ -z "${CARGO_PROFILE_RELEASE_CODEGEN_UNITS:-}" ||
  "$CARGO_PROFILE_RELEASE_CODEGEN_UNITS" == "64" ]] || {
  echo "bootstrap requires CARGO_PROFILE_RELEASE_CODEGEN_UNITS=64" >&2
  exit 2
}
export CARGO_BUILD_JOBS=1
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=64

MODE="${1:-bootstrap}"
[[ "$#" -le 1 ]] || {
  echo "usage: ${0##*/} [bootstrap|prewarm]" >&2
  exit 2
}
case "$MODE" in
  bootstrap)
    WALL_BUDGET_SECONDS="${BOOTSTRAP_WALL_BUDGET_SECONDS:-60}"
    EXECUTION_BUDGET_SECONDS="${BOOTSTRAP_EXECUTION_BUDGET_SECONDS:-30}"
    WARM_TARGET_SECONDS="${BOOTSTRAP_WARM_TARGET_SECONDS:-12}"
    RUN_TIER="bootstrap-fast"
    RUN_SCOPE="test-pyramid/bootstrap"
    ;;
  prewarm)
    # The isolated single-job dual-root cold compile exceeded 885s. This
    # diagnostic-only ceiling leaves measured linking headroom without
    # weakening the 60s mandatory warm bootstrap gate.
    WALL_BUDGET_SECONDS="${BOOTSTRAP_PREWARM_WALL_BUDGET_SECONDS:-1200}"
    EXECUTION_BUDGET_SECONDS=0
    WARM_TARGET_SECONDS=0
    RUN_TIER="bootstrap-prewarm"
    RUN_SCOPE="diagnostics/bootstrap-prewarm"
    ;;
  *)
    echo "usage: ${0##*/} [bootstrap|prewarm]" >&2
    exit 2
    ;;
esac
readonly MODE WALL_BUDGET_SECONDS EXECUTION_BUDGET_SECONDS WARM_TARGET_SECONDS RUN_TIER RUN_SCOPE
readonly THREADS="${BOOTSTRAP_THREADS:-8}"
[[ "$THREADS" =~ ^[1-8]$ ]] || {
  echo "bootstrap requires BOOTSTRAP_THREADS in the inclusive range 1..8" >&2
  exit 2
}
readonly SOURCE_CHECK_RESERVE_SECONDS=5
readonly NOVA_VERIFICATION="./.github/skills/smart-tests-bootstrap/scripts/nova_milestone_tests.sh"
readonly SOURCE_AUTHORITY="${Z00Z_BOOTSTRAP_SOURCE_AUTHORITY:-./.github/skills/smart-tests-bootstrap/scripts/bootstrap_source_authority.sh}"
readonly CACHE_IDENTITY_HELPER="./.github/skills/smart-tests-bootstrap/scripts/bootstrap_cache_identity.sh"
readonly EXPECTED_SOURCE_MANIFEST="${Z00Z_BOOTSTRAP_EXPECTED_SOURCE_MANIFEST:-}"
readonly OUTPUT_ROOT="$ROOT_DIR/crates/z00z_storage/outputs/checkpoint"
readonly CACHE_AUTHORITY_ROOT="$ROOT_DIR/.cache"
readonly DEFAULT_CACHE_ROOT="$CACHE_AUTHORITY_ROOT/phase-069/plan-08/cargo-release"
RUN_ID="$(date -u +'%Y%m%dT%H%M%S%NZ')"
readonly RUN_ID
readonly RUN_DIR="$OUTPUT_ROOT/069-08/task-1/$RUN_SCOPE/$RUN_ID"
readonly STAGE_TIMINGS="$RUN_DIR/stage-timings.tsv"
readonly SOURCE_CHECKS="$RUN_DIR/source-checks.tsv"
readonly SOURCE_MANIFEST_BASELINE="$RUN_DIR/source-manifest-baseline.tsv"
readonly CACHE_IDENTITY_FILE="$RUN_DIR/cache-identity.json"
readonly CACHE_CONTEXT_FILE="$RUN_DIR/cache-context.json"
readonly NORMAL_MESSAGES="$RUN_DIR/cargo-normal-unit-graph.jsonl"
readonly LIB_TEST_MESSAGES="$RUN_DIR/cargo-lib-test-unit-graph.jsonl"

CACHE_ROOT="${Z00Z_BOOTSTRAP_CACHE_ROOT:-$DEFAULT_CACHE_ROOT}"
NORMAL_TARGET_DIR=""
LIB_TEST_TARGET_DIR=""
CACHE_PACKAGES_JSON="[]"
CACHE_CARGO_VERSION=""
CACHE_RUSTC_VERSION=""
CACHE_RUSTFLAGS_SHA=""
CACHE_WARM_AT_START=false
STORAGE_TEST_SELECTION_JSON=null
STORAGE_TIMING_CACHE=""
BOOTSTRAP_RUN_LOCK=""
BOOTSTRAP_RUN_LOCK_FD=""
BOOTSTRAP_RUN_LOCK_ACQUIRED=false

CURRENT_STAGE="initialization"
RESULT_WRITTEN=false
START_NS="$(date +%s%N)"
EXECUTION_MS=0
CURRENT_STAGE_START_NS=0
SOURCE_DIGEST=""
SOURCE_DIGEST_AFTER=""
SOURCE_STABILITY_STATUS="uninitialized"
SOURCE_DRIFT_STAGE=""
SOURCE_CHANGED_PATHS_JSON="[]"
CACHE_IDENTITY_JSON="null"
FAIL_REASON="stage failed"

if [[ "$MODE" == prewarm ]]; then
  SELECTED_GATES=(
    "z00z_storage normal release library compile"
    "z00z_storage release lib-test compile"
  )
else
  SELECTED_GATES=(
    "Nova source/owner/coverage guards"
    "z00z_storage non-Nova/non-Plonky3 release unit suite"
    "Nova curated source/dependency/R1CS smoke packet"
    "Plonky3 structural/source/security smoke packet"
  )
fi
readonly -a SELECTED_GATES
readonly -a DEFERRED_GATES=(
  "z00z_crypto,z00z_core,z00z_utils full release unit suites"
  "Nova semantic/TestCS/fresh-proof/artifact milestones"
  "Plan-08 Plonky3 semantic integration tier"
  "real Plonky3 table, recursion, and exact-2000 prover tests"
  "Plan-08 schema/integration corpus"
  "wallet integration"
  "benches/examples compile checks"
  "workspace cargo test --release"
)

elapsed_ms() {
  local now_ns
  now_ns="$(date +%s%N)"
  printf '%s\n' "$(((now_ns - START_NS) / 1000000))"
}

json_lines() {
  jq -R . | jq -s .
}

check_root_test_output() {
  if [[ -e "$ROOT_DIR/test-results" ]]; then
    echo "forbidden repository-root test-results path exists; use crates/z00z_storage/outputs/checkpoint" >&2
    return 1
  fi
}

validate_no_symlinked_ancestor() {
  local candidate="$1" stop="$2" cursor
  cursor="$candidate"
  while [[ "$cursor" != "$stop" ]]; do
    if [[ -L "$cursor" ]]; then
      echo "bootstrap cache root contains a symlinked ancestor: $cursor" >&2
      return 88
    fi
    cursor="$(dirname "$cursor")"
    case "$cursor" in
      "$stop" | "$stop/"*) ;;
      *)
        echo "bootstrap cache root escaped repository .cache authority" >&2
        return 88
        ;;
    esac
  done
}

initialize_cache_root() {
  local cache_authority_real candidate_real marker
  [[ ! -L "$CACHE_AUTHORITY_ROOT" ]] || {
    echo "repository cache authority must not be a symlink" >&2
    return 88
  }
  mkdir -p -- "$CACHE_AUTHORITY_ROOT"
  cache_authority_real="$(realpath -e -- "$CACHE_AUTHORITY_ROOT")" || {
    echo "repository cache authority is unavailable" >&2
    return 88
  }
  [[ "$cache_authority_real" == "$CACHE_AUTHORITY_ROOT" && ! -L "$CACHE_AUTHORITY_ROOT" ]] || {
    echo "repository cache authority must be a canonical physical directory" >&2
    return 88
  }
  [[ "$CACHE_ROOT" == /* ]] || {
    echo "bootstrap cache root must be an absolute canonical path" >&2
    return 88
  }
  candidate_real="$(realpath -m -- "$CACHE_ROOT")"
  case "$candidate_real" in
    "$cache_authority_real/"*) ;;
    *)
      echo "bootstrap cache root escaped repository .cache authority" >&2
      return 88
      ;;
  esac
  if [[ "$candidate_real" != "$CACHE_ROOT" ]]; then
    validate_no_symlinked_ancestor "$CACHE_ROOT" "$cache_authority_real" || return
    echo "bootstrap cache root must not contain dot segments or path aliases" >&2
    return 88
  fi
  validate_no_symlinked_ancestor "$candidate_real" "$cache_authority_real" || return
  [[ ! -e "$candidate_real" || -d "$candidate_real" ]] || {
    echo "bootstrap cache root is not a directory" >&2
    return 88
  }
  mkdir -p -- "$candidate_real"
  [[ ! -L "$candidate_real" &&
    "$(realpath -e -- "$candidate_real")" == "$candidate_real" ]] || {
    echo "bootstrap cache root contains a symlinked ancestor" >&2
    return 88
  }
  CACHE_ROOT="$candidate_real"
  marker="$CACHE_ROOT/.z00z-bootstrap-cache-v3"
  [[ ! -L "$marker" ]] || {
    echo "bootstrap cache marker must not be a symlink" >&2
    return 88
  }
  if [[ -e "$marker" ]]; then
    [[ -f "$marker" &&
      "$(<"$marker")" == "z00z.phase069.bootstrap-cache.v3" ]] || {
      echo "bootstrap cache marker is invalid" >&2
      return 88
    }
  else
    printf 'z00z.phase069.bootstrap-cache.v3\n' >"$marker"
  fi
}

initialize_cache_layout() {
  local target_dir
  [[ "$SOURCE_DIGEST" =~ ^[0-9a-f]{64}$ ]] || {
    echo "bootstrap source digest is unavailable for cache identity" >&2
    return 88
  }
  NORMAL_TARGET_DIR="$CACHE_ROOT/normal-library"
  LIB_TEST_TARGET_DIR="$CACHE_ROOT/library-test"
  if [[ -d "$NORMAL_TARGET_DIR/release/.fingerprint" &&
    -d "$LIB_TEST_TARGET_DIR/release/.fingerprint" &&
    -n "$(find "$NORMAL_TARGET_DIR/release/.fingerprint" \
      -mindepth 1 -maxdepth 1 -type d -print -quit)" &&
    -n "$(find "$LIB_TEST_TARGET_DIR/release/.fingerprint" \
      -mindepth 1 -maxdepth 1 -type d -print -quit)" ]]; then
    CACHE_WARM_AT_START=true
  fi
  for target_dir in "$NORMAL_TARGET_DIR" "$LIB_TEST_TARGET_DIR"; do
    [[ ! -L "$target_dir" ]] || {
      echo "bootstrap target root must not be a symlink: $target_dir" >&2
      return 88
    }
    mkdir -p -- "$target_dir"
    [[ "$(realpath -e -- "$target_dir")" == "$target_dir" ]] || {
      echo "bootstrap target root is not canonical: $target_dir" >&2
      return 88
    }
  done
  STORAGE_TIMING_CACHE="$CACHE_ROOT/bootstrap-exact-test-timings-v1.tsv"
  [[ ! -L "$STORAGE_TIMING_CACHE" ]] || {
    echo "bootstrap timing cache must not be a symlink" >&2
    return 88
  }
  if [[ ! -e "$STORAGE_TIMING_CACHE" ]]; then
    : >"$STORAGE_TIMING_CACHE"
  fi
  [[ -f "$STORAGE_TIMING_CACHE" ]] || {
    echo "bootstrap timing cache is not a regular file" >&2
    return 88
  }
  if ! awk -F '\t' '
    NF != 2 || $1 == "" || $2 !~ /^[0-9]+$/ {
      exit 1
    }
  ' "$STORAGE_TIMING_CACHE"; then
    echo "bootstrap timing cache is malformed" >&2
    return 88
  fi
  if ! LC_ALL=C sort -c -u -t $'\t' -k1,1 "$STORAGE_TIMING_CACHE"; then
    echo "bootstrap timing cache is not canonically ordered" >&2
    return 88
  fi
}

acquire_bootstrap_run_lock() {
  command -v flock >/dev/null 2>&1 || {
    echo "bootstrap requires flock for canonical cache ownership" >&2
    return 88
  }
  BOOTSTRAP_RUN_LOCK="$CACHE_ROOT/.bootstrap-run.lock"
  [[ ! -L "$BOOTSTRAP_RUN_LOCK" ]] || {
    echo "bootstrap run lock must not be a symlink" >&2
    return 88
  }
  exec {BOOTSTRAP_RUN_LOCK_FD}<>"$BOOTSTRAP_RUN_LOCK"
  if ! flock -n "$BOOTSTRAP_RUN_LOCK_FD"; then
    FAIL_REASON="concurrent_bootstrap_runner"
    echo "canonical bootstrap cache is owned by another active runner" >&2
    return 76
  fi
  BOOTSTRAP_RUN_LOCK_ACQUIRED=true
  : >"$BOOTSTRAP_RUN_LOCK"
  printf 'pid=%s run_id=%s mode=%s\n' "$$" "$RUN_ID" "$MODE" \
    >&"$BOOTSTRAP_RUN_LOCK_FD"
}

initialize_cache_context_inputs() {
  CACHE_PACKAGES_JSON="$(
    "$SOURCE_AUTHORITY" packages |
      jq -Rn '
        [
          inputs
          | split("\t")
          | {name: .[0], manifest: .[1]}
        ]
      '
  )"
  CACHE_RUSTFLAGS_SHA="$(sha256sum .cargo/config.toml | awk '{print $1}')"
  CACHE_CARGO_VERSION="$(cargo --version --verbose)"
  CACHE_RUSTC_VERSION="$(rustc --version --verbose)"
}

write_cache_context() {
  jq -n -S \
    --arg execution_scope "${Z00Z_BOOTSTRAP_EXECUTION_SCOPE:-direct}" \
    --arg repo_root "$ROOT_DIR" \
    --arg checkpoint_output_root "$OUTPUT_ROOT" \
    --arg cache_authority_root "$CACHE_AUTHORITY_ROOT" \
    --arg normal_target "$NORMAL_TARGET_DIR" \
    --arg test_target "$LIB_TEST_TARGET_DIR" \
    --arg rustflags_sha "$CACHE_RUSTFLAGS_SHA" \
    --arg cargo_version "$CACHE_CARGO_VERSION" \
    --arg rustc_version "$CACHE_RUSTC_VERSION" \
    --arg bootstrap_threads "$THREADS" \
    --arg cargo_build_jobs "${CARGO_BUILD_JOBS:-}" \
    --arg release_codegen_units "${CARGO_PROFILE_RELEASE_CODEGEN_UNITS:-}" \
    --arg source_authority_digest "$SOURCE_DIGEST" \
    --argjson packages "$CACHE_PACKAGES_JSON" \
    '{
      schema: "z00z.phase069.bootstrap-cache-context.v3",
      execution_scope: $execution_scope,
      repo_root: $repo_root,
      checkpoint_output_root: $checkpoint_output_root,
      cache_authority_root: $cache_authority_root,
      source_authority_digest: $source_authority_digest,
      root_package_name: "z00z_storage",
      target_dirs: {
        normal_library: $normal_target,
        library_test: $test_target
      },
      profile: "release",
      rustflags: {
        authority: ".cargo/config.toml",
        authority_sha256: $rustflags_sha,
        required_cfg: "test_fast"
      },
      toolchain: {
        cargo: $cargo_version,
        rustc: $rustc_version
      },
      compile_environment: {
        bootstrap_threads: $bootstrap_threads,
        cargo_build_jobs: $cargo_build_jobs,
        release_codegen_units: $release_codegen_units
      },
      resolved_local_packages: $packages,
      retention: {
        schema: "z00z.phase069.bootstrap-cache-retention.v3",
        strategy: "fixed-cargo-targets",
        max_target_roots: 2,
        automatic_deletion: false
      },
      compile_contract: {
        normal_library: (
          "CARGO_TARGET_DIR=" + $normal_target
          + " cargo build --release --locked --offline -p z00z_storage"
        ),
        library_test: (
          "CARGO_TARGET_DIR=" + $test_target
          + " cargo test --release --locked --offline --lib"
          + " -p z00z_storage --no-run"
        )
      }
    }' >"$CACHE_CONTEXT_FILE"
}

initialize_source_authority() {
  local current_digest comparison compare_status expected_real
  [[ -x "$SOURCE_AUTHORITY" ]] || {
    echo "bootstrap source authority is missing or not executable: $SOURCE_AUTHORITY" >&2
    return 1
  }
  "$SOURCE_AUTHORITY" manifest >"$SOURCE_MANIFEST_BASELINE"
  current_digest="$(sha256sum "$SOURCE_MANIFEST_BASELINE" | awk '{print $1}')"
  if [[ -n "$EXPECTED_SOURCE_MANIFEST" ]]; then
    expected_real="$(realpath -e -- "$EXPECTED_SOURCE_MANIFEST")" || {
      echo "expected bootstrap source manifest is unavailable" >&2
      return 86
    }
    case "$expected_real" in
      "$OUTPUT_ROOT/069-08/task-1/resource-worker/"*/source-manifest-launch.tsv) ;;
      *)
        echo "expected bootstrap source manifest escaped the worker evidence root" >&2
        return 86
        ;;
    esac
    SOURCE_DIGEST="$(sha256sum "$expected_real" | awk '{print $1}')"
    SOURCE_DIGEST_AFTER="$current_digest"
    printf 'worker-launch\t%s\n' "$SOURCE_DIGEST" >>"$SOURCE_CHECKS"
    printf 'initialization\t%s\n' "$current_digest" >>"$SOURCE_CHECKS"
    comparison="$RUN_DIR/source-comparison-initialization.json"
    set +e
    "$SOURCE_AUTHORITY" compare \
      "$expected_real" "$SOURCE_MANIFEST_BASELINE" >"$comparison"
    compare_status=$?
    set -e
    if (( compare_status != 0 )); then
      SOURCE_STABILITY_STATUS="source_drift"
      SOURCE_DRIFT_STAGE="initialization"
      SOURCE_CHANGED_PATHS_JSON="$(jq -c '.changed_paths' "$comparison")"
      FAIL_REASON="source_drift"
      echo "bootstrap source drift detected before initialization" >&2
      return 86
    fi
  else
    SOURCE_DIGEST="$current_digest"
    SOURCE_DIGEST_AFTER="$current_digest"
    printf 'initialization\t%s\n' "$current_digest" >>"$SOURCE_CHECKS"
  fi
  SOURCE_STABILITY_STATUS="stable"
}

assert_source_stable() {
  local label="$1" discovery="${2:-tracked}"
  local safe_label current_manifest comparison
  local manifest_status compare_status
  safe_label="${label//[^A-Za-z0-9._-]/-}"
  current_manifest="$RUN_DIR/source-manifest-$safe_label.tsv"
  comparison="$RUN_DIR/source-comparison-$safe_label.json"

  set +e
  if [[ "$discovery" == full ]]; then
    "$SOURCE_AUTHORITY" manifest >"$current_manifest"
  else
    "$SOURCE_AUTHORITY" rehash \
      "$SOURCE_MANIFEST_BASELINE" >"$current_manifest"
  fi
  manifest_status=$?
  set -e
  if (( manifest_status != 0 )) && [[ "$discovery" == full ]]; then
    "$SOURCE_AUTHORITY" rehash \
      "$SOURCE_MANIFEST_BASELINE" >"$current_manifest"
  fi

  set +e
  "$SOURCE_AUTHORITY" compare \
    "$SOURCE_MANIFEST_BASELINE" "$current_manifest" >"$comparison"
  compare_status=$?
  set -e
  if (( compare_status != 0 && compare_status != 86 )); then
    SOURCE_STABILITY_STATUS="source_drift"
    SOURCE_DRIFT_STAGE="$label"
    SOURCE_DIGEST_AFTER="unavailable"
    SOURCE_CHANGED_PATHS_JSON="$(
      jq -n \
        --arg detail "authority comparison failed with status $compare_status" \
        '[{
          change: "authority_error",
          path: "<source-authority>",
          before_sha256: null,
          after_sha256: null,
          detail: $detail
        }]'
    )"
    FAIL_REASON="source_drift"
    echo "bootstrap source authority comparison failed at $label" >&2
    return 86
  fi

  SOURCE_DIGEST_AFTER="$(jq -r '.after_digest' "$comparison")"
  printf '%s\t%s\n' "$label" "$SOURCE_DIGEST_AFTER" >>"$SOURCE_CHECKS"
  if (( manifest_status != 0 || compare_status == 86 )); then
    SOURCE_STABILITY_STATUS="source_drift"
    SOURCE_DRIFT_STAGE="$label"
    SOURCE_CHANGED_PATHS_JSON="$(jq -c '.changed_paths' "$comparison")"
    if (( manifest_status != 0 )); then
      SOURCE_CHANGED_PATHS_JSON="$(
        jq -c \
          --argjson paths "$SOURCE_CHANGED_PATHS_JSON" \
          '$paths + [{
            change: "authority_error",
            path: "<source-authority>",
            before_sha256: null,
            after_sha256: null,
            detail: "current source authority could not be resolved"
          }]'
      )"
    fi
    FAIL_REASON="source_drift"
    echo "bootstrap source drift detected at $label" >&2
    return 86
  fi
}

write_result() {
  local status="$1" reason="$2" total_ms selected_json deferred_json stages_json
  local source_checks_json drift_stage_json
  total_ms="$(elapsed_ms)"
  selected_json="$(printf '%s\n' "${SELECTED_GATES[@]}" | json_lines)"
  deferred_json="$(printf '%s\n' "${DEFERRED_GATES[@]}" | json_lines)"
  stages_json="$(
    jq -Rn '
      [
        inputs
        | split("\t")
        | select(length == 2)
        | {name: .[0], wall_ms: (.[1] | tonumber)}
      ]
    ' <"$STAGE_TIMINGS"
  )"
  source_checks_json="$(
    jq -Rn '
      [
        inputs
        | split("\t")
        | select(length == 2)
        | {checkpoint: .[0], digest: .[1]}
      ]
    ' <"$SOURCE_CHECKS"
  )"
  drift_stage_json="$(
    if [[ -n "$SOURCE_DRIFT_STAGE" ]]; then
      jq -n --arg value "$SOURCE_DRIFT_STAGE" '$value'
    else
      printf 'null\n'
    fi
  )"
  jq -n -S \
    --arg schema "z00z.phase069.test-pyramid.v1" \
    --arg recorded_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
    --arg status "$status" \
    --arg reason "$reason" \
    --arg mode "$MODE" \
    --arg tier "$RUN_TIER" \
    --arg current_stage "$CURRENT_STAGE" \
    --arg source_digest "$SOURCE_DIGEST" \
    --arg source_digest_after "$SOURCE_DIGEST_AFTER" \
    --arg source_stability_status "$SOURCE_STABILITY_STATUS" \
    --arg normal_target_dir "$NORMAL_TARGET_DIR" \
    --arg lib_test_target_dir "$LIB_TEST_TARGET_DIR" \
    --arg bootstrap_run_lock "$BOOTSTRAP_RUN_LOCK" \
    --arg profile "release" \
    --argjson wall_budget_seconds "$WALL_BUDGET_SECONDS" \
    --argjson execution_budget_seconds "$EXECUTION_BUDGET_SECONDS" \
    --argjson warm_target_seconds "$WARM_TARGET_SECONDS" \
    --argjson total_wall_ms "$total_ms" \
    --argjson test_execution_ms "$EXECUTION_MS" \
    --argjson stages "$stages_json" \
    --argjson source_checks "$source_checks_json" \
    --argjson source_drift_stage "$drift_stage_json" \
    --argjson source_changed_paths "$SOURCE_CHANGED_PATHS_JSON" \
    --argjson cache_identity "$CACHE_IDENTITY_JSON" \
    --argjson cache_warm_at_start "$CACHE_WARM_AT_START" \
    --argjson bootstrap_run_lock_acquired "$BOOTSTRAP_RUN_LOCK_ACQUIRED" \
    --argjson storage_test_selection "$STORAGE_TEST_SELECTION_JSON" \
    --argjson selected "$selected_json" \
    --argjson deferred "$deferred_json" \
    '{
      schema: $schema,
      recorded_at: $recorded_at,
      tier: $tier,
      mode: $mode,
      status: $status,
      reason: $reason,
      current_stage: $current_stage,
      identity: {
        source_digest: $source_digest,
        source_digest_before: $source_digest,
        source_digest_after: $source_digest_after
      },
      source_stability: {
        status: $source_stability_status,
        before_digest: $source_digest,
        after_digest: $source_digest_after,
        drift_stage: $source_drift_stage,
        checks: $source_checks,
        changed_paths: $source_changed_paths
      },
      cache_identity: $cache_identity,
      concurrency: {
        canonical_run_lock: $bootstrap_run_lock,
        acquired: $bootstrap_run_lock_acquired,
        policy: "fail-fast-single-runner"
      },
      profile: $profile,
      cargo_target_dirs: {
        normal_library: $normal_target_dir,
        library_test: $lib_test_target_dir
      },
      compile_contract: [
        "cargo build --release -p z00z_storage",
        "cargo test --release --lib -p z00z_storage --no-run"
      ],
      cache_semantics: {
        compile_only_prewarm_is_acceptance: false,
        bootstrap_requires_prior_prewarm: false,
        prewarm_policy: "on_demand_after_typed_cold_compile_timeout",
        incompatible_unit_fingerprints_use_separate_target_roots: true,
        stable_target_roots_use_cargo_fingerprints: true,
        cache_identity_binds_source_authority: true,
        automatic_cache_deletion: false,
        cold_compile_is_bounded_inside_gate: true,
        compile_stage_is_in_total_wall: true,
        compile_stage_is_in_post_compile_execution: false
      },
      budgets: {
        total_wall_seconds: $wall_budget_seconds,
        post_compile_execution_seconds: $execution_budget_seconds,
        warm_target_seconds: $warm_target_seconds
      },
      resources: {
        total_wall_ms: $total_wall_ms,
        post_compile_execution_ms: $test_execution_ms,
        cache_warm_at_start: $cache_warm_at_start,
        warm_target_met: (
          $warm_target_seconds == 0
          or ($cache_warm_at_start | not)
          or $total_wall_ms <= ($warm_target_seconds * 1000)
        )
      },
      stages: $stages,
      storage_test_selection: $storage_test_selection,
      selected_gates: $selected,
      explicitly_deferred_gates: $deferred,
      acceptance_authority: ($mode == "bootstrap")
    }' >"$RUN_DIR/result.json"
  RESULT_WRITTEN=true
}

on_exit() {
  local status=$? reason="$FAIL_REASON"
  trap - EXIT
  set +e
  if ! check_root_test_output; then
    status=1
    reason="forbidden root test-results path"
  elif (( status == 0 )); then
    reason="success"
  fi
  if [[ "$RESULT_WRITTEN" != true ]]; then
    write_result "$([[ "$status" == 0 ]] && printf pass || printf fail)" "$reason"
  fi
  printf 'Z00Z_BOOTSTRAP_EVIDENCE_V1 %s\n' "$RUN_DIR/result.json"
  exit "$status"
}
trap on_exit EXIT

run_budgeted() {
  local kind="$1" wall_remaining_ms execution_remaining_ms limit_ms
  local limit_reason status now_ns stage_elapsed_ms
  shift

  wall_remaining_ms=$((
    (WALL_BUDGET_SECONDS - SOURCE_CHECK_RESERVE_SECONDS) * 1000 -
      $(elapsed_ms)
  ))
  limit_ms=$wall_remaining_ms
  limit_reason="total wall budget exceeded"
  if [[ "$kind" == execution ]]; then
    now_ns="$(date +%s%N)"
    stage_elapsed_ms=$(((now_ns - CURRENT_STAGE_START_NS) / 1000000))
    execution_remaining_ms=$((
      EXECUTION_BUDGET_SECONDS * 1000 -
        EXECUTION_MS -
        stage_elapsed_ms -
        5000
    ))
    if (( execution_remaining_ms < limit_ms )); then
      limit_ms=$execution_remaining_ms
      limit_reason="post-compile execution budget exceeded"
    fi
  fi
  if (( limit_ms <= 0 )); then
    FAIL_REASON="$limit_reason"
    echo "$RUN_TIER has no remaining bounded time for $CURRENT_STAGE" >&2
    return 124
  fi

  if timeout --signal=TERM --kill-after=5s \
    "$(((limit_ms + 999) / 1000))" "$@"; then
    status=0
  else
    status=$?
  fi
  if (( status == 124 )); then
    if [[ "$MODE" == bootstrap && "$CURRENT_STAGE" == storage_compile ]]; then
      FAIL_REASON="prewarm_required"
      echo "$RUN_TIER stopped a cold compile at its bounded deadline; run the isolated diagnostic prewarm once, then rerun bootstrap" >&2
    else
      FAIL_REASON="$limit_reason"
      echo "$RUN_TIER stopped $CURRENT_STAGE at its bounded deadline" >&2
    fi
  fi
  return "$status"
}

run_stage() {
  local name="$1" kind="$2" stage_start_ns stage_end_ns stage_ms status
  shift 2
  CURRENT_STAGE="$name"
  stage_start_ns="$(date +%s%N)"
  CURRENT_STAGE_START_NS="$stage_start_ns"
  set +e
  "$@" > >(tee "$RUN_DIR/$name.log") 2>&1
  status=$?
  set -e
  stage_end_ns="$(date +%s%N)"
  stage_ms=$(((stage_end_ns - stage_start_ns) / 1000000))
  printf '%s\t%s\n' "$name" "$stage_ms" >>"$STAGE_TIMINGS"
  if [[ "$kind" == execution ]]; then
    EXECUTION_MS=$((EXECUTION_MS + stage_ms))
  fi
  assert_source_stable "$name:after" || return 86
  (( status == 0 )) || return "$status"
  if (( $(elapsed_ms) > WALL_BUDGET_SECONDS * 1000 )); then
    FAIL_REASON="total wall budget exceeded"
    echo "bootstrap-fast exceeded ${WALL_BUDGET_SECONDS}s total wall budget" >&2
    return 124
  fi
  if (( EXECUTION_MS > EXECUTION_BUDGET_SECONDS * 1000 )); then
    FAIL_REASON="post-compile execution budget exceeded"
    echo "bootstrap-fast exceeded ${EXECUTION_BUDGET_SECONDS}s post-compile execution budget" >&2
    return 124
  fi
}

capture_cache_identity() {
  "$CACHE_IDENTITY_HELPER" capture \
    "$NORMAL_MESSAGES" \
    "$LIB_TEST_MESSAGES" \
    "$CACHE_CONTEXT_FILE" \
    "$CACHE_IDENTITY_FILE"
  CACHE_IDENTITY_JSON="$(<"$CACHE_IDENTITY_FILE")"
}

canonical_lib_test_binary() {
  local executable_relative test_binary
  executable_relative="$(
    jq -er '
      [
        .unit_graph.library_test[]
        | select(.profile.test == true)
        | .executable
      ]
      | if length == 1 then .[0]
        else error("expected exactly one release lib-test executable")
        end
    ' "$CACHE_IDENTITY_FILE"
  )" || return
  test_binary="$(realpath -e -- "$ROOT_DIR/$executable_relative")" || return
  case "$test_binary" in
    "$LIB_TEST_TARGET_DIR"/release/deps/*) ;;
    *)
      echo "release lib-test executable escaped the canonical cache root" >&2
      return 88
      ;;
  esac
  [[ -f "$test_binary" && -x "$test_binary" && ! -L "$test_binary" ]] || {
    echo "release lib-test executable is unavailable or symlinked" >&2
    return 88
  }
  printf '%s\n' "$test_binary"
}

compile_storage_tests() {
  local status
  # Keep incompatible normal and lib-test fingerprints in distinct stable roots.
  # The JSON streams from these exact commands are the sole identity inputs.
  if CARGO_TARGET_DIR="$NORMAL_TARGET_DIR" \
    run_budgeted compile \
      cargo build --release --locked --offline -p z00z_storage \
      --message-format=json-render-diagnostics >"$NORMAL_MESSAGES"; then
    status=0
  else
    status=$?
  fi
  if (( status != 0 )); then
    [[ "$FAIL_REASON" != "stage failed" ]] ||
      FAIL_REASON="normal_release_compile_failed"
    return "$status"
  fi

  if CARGO_TARGET_DIR="$LIB_TEST_TARGET_DIR" \
    run_budgeted compile \
      cargo test --release --locked --offline --lib -p z00z_storage --no-run \
      --message-format=json-render-diagnostics >"$LIB_TEST_MESSAGES"; then
    status=0
  else
    status=$?
  fi
  if (( status != 0 )); then
    [[ "$FAIL_REASON" != "stage failed" ]] ||
      FAIL_REASON="library_test_compile_failed"
    return "$status"
  fi

  write_cache_context || return
  capture_cache_identity
}

update_storage_timing_cache() {
  local selected_manifest="$1" log_dir="$2" expected_count="$3"
  local next_cache staged_cache observed_count
  next_cache="$RUN_DIR/storage-exact-test-timings-next.tsv"
  staged_cache="$STORAGE_TIMING_CACHE.next"
  [[ ! -L "$staged_cache" ]] || {
    echo "bootstrap staged timing cache must not be a symlink" >&2
    return 88
  }
  awk -F '\t' '
    FNR == NR {
      names[$1] = $2
      next
    }
    /^test result: ok\. 1 passed; 0 failed;/ &&
      /finished in [0-9]+([.][0-9]+)?s/ {
      id = FILENAME
      sub(/^.*\//, "", id)
      sub(/[.]log$/, "", id)
      seconds = $0
      sub(/^.*finished in /, "", seconds)
      sub(/s.*$/, "", seconds)
      if (!(id in names) || seconds !~ /^[0-9]+([.][0-9]+)?$/) {
        exit 2
      }
      printf "%s\t%.0f\n", names[id], seconds * 1000
      seen[id]++
    }
    END {
      for (id in names) {
        if (seen[id] != 1) {
          exit 3
        }
      }
    }
  ' "$selected_manifest" "$log_dir"/*.log |
    LC_ALL=C sort -t $'\t' -k1,1 >"$next_cache"
  observed_count="$(wc -l <"$next_cache")"
  (( observed_count == expected_count )) || {
    echo "bootstrap timing cache did not observe every exact test" >&2
    return 1
  }
  cp -- "$next_cache" "$staged_cache"
  chmod 600 "$staged_cache"
  mv -f -- "$staged_cache" "$STORAGE_TIMING_CACHE"
  sha256sum "$STORAGE_TIMING_CACHE" | awk '{print $1}'
}

run_non_recursive_storage() {
  local executable_relative test_binary all_tests ignored_tests runnable_tests
  local selected_tests deferred_tests selected_manifest recursive_smoke_tests log_dir
  local weighted_manifest regular_manifest weighted_test weighted_entry weighted_id
  local weighted_name weighted_log weighted_count regular_count weighted_crash_workers
  local regular_schedule_manifest timing_cache_digest timing_cache_updated_digest
  local maximum_simultaneous_test_processes
  local all_count ignored_count runnable_count selected_count deferred_count smoke_count
  local schedule_status=0 id test_name test_log

  executable_relative="$(
    jq -er '
      [
        .unit_graph.library_test[]
        | select(.profile.test == true)
        | .executable
      ]
      | if length == 1 then .[0]
        else error("expected exactly one release lib-test executable")
        end
    ' "$CACHE_IDENTITY_FILE"
  )"
  test_binary="$ROOT_DIR/$executable_relative"
  case "$test_binary" in
    "$LIB_TEST_TARGET_DIR/"*) ;;
    *)
      echo "release lib-test executable escaped the canonical cache root" >&2
      return 88
      ;;
  esac
  [[ -x "$test_binary" && ! -L "$test_binary" ]] || {
    echo "release lib-test executable is unavailable or symlinked" >&2
    return 88
  }

  all_tests="$RUN_DIR/storage-all-tests.txt"
  ignored_tests="$RUN_DIR/storage-ignored-tests.txt"
  runnable_tests="$RUN_DIR/storage-runnable-tests.txt"
  selected_tests="$RUN_DIR/storage-selected-tests.txt"
  deferred_tests="$RUN_DIR/storage-scope-deferred-tests.txt"
  selected_manifest="$RUN_DIR/storage-selected-tests.tsv"
  weighted_manifest="$RUN_DIR/storage-weighted-tests.tsv"
  regular_manifest="$RUN_DIR/storage-regular-tests.tsv"
  recursive_smoke_tests="$RUN_DIR/recursive-smoke-unit-tests.txt"
  log_dir="$RUN_DIR/storage-exact-tests"
  weighted_test="backend::redb::helpers::recursive_v2_cutover_crash_tests::recursive_v2_cutover_owned_boundary_crash_corpus"
  mkdir -p "$log_dir"
  printf '%s\n' \
    checkpoint::nova::tests::test_recursive_source_manifest_covers_explicit_path_modules \
    checkpoint::nova::tests::test_verifier_identity_binds_path \
    checkpoint::nova::tests::test_nova_backend_owner_locked \
    checkpoint::nova::tests::test_nova_dependency_transcript_pinned \
    checkpoint::nova::tests::test_nova_poseidon_wires_pinned \
    checkpoint::nova::tests::test_nova_pasta_identity_pinned \
    checkpoint::nova::tests::test_nova_keccak_transcript_pinned \
    checkpoint::nova::tests::test_nova_mutation_smoke \
    checkpoint::plonky3::tests::test_source_sha_binding \
    checkpoint::plonky3::tests::test_complete_air_enables_evidence \
    checkpoint::plonky3::tests::test_poseidon_hash_binds_shape \
    checkpoint::plonky3::tests::test_recursive_event_source_framing_rejects_public_aliases \
    checkpoint::plonky3::tests::test_root_authority_fails_closed \
    checkpoint::plonky3::tests::test_security_derivation_rejects_drift \
    checkpoint::plonky3::tests::test_security_budget_rounding |
    LC_ALL=C sort -u >"$recursive_smoke_tests"

  "$test_binary" --list --format terse |
    sed -n 's/: test$//p' |
    LC_ALL=C sort -u >"$all_tests"
  "$test_binary" --list --ignored --format terse |
    sed -n 's/: test$//p' |
    LC_ALL=C sort -u >"$ignored_tests"
  awk '
    NR == FNR {
      ignored[$0] = 1
      next
    }
    !($0 in ignored)
  ' "$ignored_tests" "$all_tests" >"$runnable_tests"
  if [[ -n "$(LC_ALL=C comm -23 "$recursive_smoke_tests" "$runnable_tests")" ]]; then
    echo "recursive smoke allowlist contains a missing or ignored test" >&2
    LC_ALL=C comm -23 "$recursive_smoke_tests" "$runnable_tests" >&2
    return 1
  fi
  awk '
    NR == FNR {
      smoke[$0] = 1
      next
    }
    (/^checkpoint::nova::tests::/ ||
      /^checkpoint::plonky3::tests::/) &&
      !($0 in smoke) {
      print
    }
  ' "$recursive_smoke_tests" "$runnable_tests" >"$deferred_tests"
  awk '
    NR == FNR {
      smoke[$0] = 1
      next
    }
    !(/^checkpoint::nova::tests::/ ||
      /^checkpoint::plonky3::tests::/) ||
      ($0 in smoke) {
      print
    }
  ' "$recursive_smoke_tests" "$runnable_tests" >"$selected_tests"
  awk '{ printf "%06d\t%s\n", NR, $0 }' \
    "$selected_tests" >"$selected_manifest"
  awk -F '\t' -v weighted_test="$weighted_test" \
    '$2 == weighted_test' "$selected_manifest" >"$weighted_manifest"
  awk -F '\t' -v weighted_test="$weighted_test" \
    '$2 != weighted_test' "$selected_manifest" >"$regular_manifest"

  all_count="$(wc -l <"$all_tests")"
  ignored_count="$(wc -l <"$ignored_tests")"
  runnable_count="$(wc -l <"$runnable_tests")"
  selected_count="$(wc -l <"$selected_tests")"
  deferred_count="$(wc -l <"$deferred_tests")"
  smoke_count="$(wc -l <"$recursive_smoke_tests")"
  weighted_count="$(wc -l <"$weighted_manifest")"
  regular_count="$(wc -l <"$regular_manifest")"
  weighted_crash_workers=$((THREADS - 1))
  (( weighted_crash_workers >= 1 )) || weighted_crash_workers=1
  (( weighted_crash_workers <= 5 )) || weighted_crash_workers=5
  maximum_simultaneous_test_processes=$((weighted_crash_workers + 1))
  if (( THREADS > maximum_simultaneous_test_processes )); then
    maximum_simultaneous_test_processes="$THREADS"
  fi
  (( all_count > 0 && selected_count > 0 )) || {
    echo "storage exact-test scheduler selected an empty suite" >&2
    return 1
  }
  (( runnable_count == selected_count + deferred_count )) || {
    echo "storage exact-test scheduler lost or duplicated runnable tests" >&2
    return 1
  }
  (( weighted_count == 1 && regular_count + weighted_count == selected_count )) || {
    echo "storage weighted scheduler lost or duplicated selected tests" >&2
    return 1
  }
  regular_schedule_manifest="$RUN_DIR/storage-regular-schedule.tsv"
  timing_cache_digest="$(sha256sum "$STORAGE_TIMING_CACHE" | awk '{print $1}')"
  awk -F '\t' '
    FILENAME == ARGV[1] {
      timing_ms[$1] = $2 + 0
      next
    }
    {
      observed_ms = (($2 in timing_ms) ? timing_ms[$2] : 0)
      printf "%012.0f\t%s\n", observed_ms, $0
    }
  ' "$STORAGE_TIMING_CACHE" "$regular_manifest" |
    LC_ALL=C sort -t $'\t' -k1,1nr -k3,3 |
    cut -f2- >"$regular_schedule_manifest"
  [[ "$(wc -l <"$regular_schedule_manifest")" == "$regular_count" ]] || {
    echo "storage timing scheduler lost or duplicated regular tests" >&2
    return 1
  }

  STORAGE_TEST_SELECTION_JSON="$(
    jq -n -S \
      --arg executable "$executable_relative" \
      --arg selected_manifest "$selected_manifest" \
      --arg selected_digest "$(sha256sum "$selected_manifest" | awk '{print $1}')" \
      --arg recursive_smoke_manifest "$recursive_smoke_tests" \
      --arg weighted_manifest "$weighted_manifest" \
      --arg regular_schedule_manifest "$regular_schedule_manifest" \
      --arg regular_schedule_digest "$(sha256sum "$regular_schedule_manifest" | awk '{print $1}')" \
      --arg timing_cache "$STORAGE_TIMING_CACHE" \
      --arg timing_cache_digest "$timing_cache_digest" \
      --arg weighted_test "$weighted_test" \
      --argjson all_count "$all_count" \
      --argjson ignored_count "$ignored_count" \
      --argjson runnable_count "$runnable_count" \
      --argjson selected_count "$selected_count" \
      --argjson deferred_count "$deferred_count" \
      --argjson recursive_smoke_count "$smoke_count" \
      --argjson weighted_count "$weighted_count" \
      --argjson regular_count "$regular_count" \
      --argjson process_parallelism "$THREADS" \
      --argjson weighted_crash_workers "$weighted_crash_workers" \
      --argjson maximum_simultaneous_test_processes "$maximum_simultaneous_test_processes" \
      --argjson weighted_verify_workers 5 \
      '{
        executable: $executable,
        selected_manifest: $selected_manifest,
        selected_manifest_sha256: $selected_digest,
        recursive_smoke_manifest: $recursive_smoke_manifest,
        weighted_manifest: $weighted_manifest,
        regular_schedule_manifest: $regular_schedule_manifest,
        regular_schedule_manifest_sha256: $regular_schedule_digest,
        timing_cache: {
          path: $timing_cache,
          used_sha256: $timing_cache_digest,
          updated_sha256: null,
          affects_selection: false
        },
        recursive_smoke_unit_tests: $recursive_smoke_count,
        discovered_tests: $all_count,
        ignored_tests: $ignored_count,
        runnable_tests: $runnable_count,
        selected_tests: $selected_count,
        weighted_tests: $weighted_count,
        regular_tests: $regular_count,
        explicitly_deferred_scope_tests: $deferred_count,
        scheduler: {
          kind: "bounded-weighted-exact-test-process-pool",
          process_parallelism: $process_parallelism,
          test_threads_per_process: 1,
          weighted_test: $weighted_test,
          weighted_crash_workers: $weighted_crash_workers,
          weighted_verify_workers: $weighted_verify_workers,
          weighted_and_regular_overlap: false,
          maximum_simultaneous_test_processes: $maximum_simultaneous_test_processes,
          exactly_once_required: true
        }
      }'
  )"

  weighted_entry="$(<"$weighted_manifest")"
  weighted_id="${weighted_entry%%	*}"
  weighted_name="${weighted_entry#*	}"
  weighted_log="$log_dir/$weighted_id.log"
  if run_budgeted execution \
    env Z00Z_RECURSIVE_V2_CUTOVER_CRASH_WORKERS="$weighted_crash_workers" \
    "$test_binary" "$weighted_name" \
    --exact --nocapture --test-threads=1 >"$weighted_log" 2>&1; then
    schedule_status=0
  else
    schedule_status=$?
  fi
  if (( schedule_status != 0 )); then
    echo "storage weighted exact test failed with status $schedule_status" >&2
    [[ ! -s "$weighted_log" ]] || tail -n 40 "$weighted_log" >&2
    return "$schedule_status"
  fi

  export Z00Z_BOOTSTRAP_TEST_BINARY="$test_binary"
  export Z00Z_BOOTSTRAP_TEST_LOG_DIR="$log_dir"
  # The child bash process expands the single-quoted scheduler body.
  # shellcheck disable=SC2016
  if run_budgeted execution \
    xargs -d '\n' -P "$THREADS" -I '{}' \
      bash -c '
        entry="$1"
        id="${entry%%	*}"
        test_name="${entry#*	}"
        test_log="$Z00Z_BOOTSTRAP_TEST_LOG_DIR/$id.log"
        "$Z00Z_BOOTSTRAP_TEST_BINARY" "$test_name" \
          --exact --nocapture --test-threads=1 >"$test_log" 2>&1
      ' _ '{}' <"$regular_schedule_manifest"; then
    schedule_status=0
  else
    schedule_status=$?
  fi
  if (( schedule_status != 0 )); then
    echo "storage regular exact-test process pool failed with status $schedule_status" >&2
  fi

  while IFS=$'\t' read -r id test_name; do
    test_log="$log_dir/$id.log"
    if [[ ! -s "$test_log" ]] ||
      ! grep -Fq "test $test_name ..." "$test_log" ||
      ! grep -Fq "test result: ok. 1 passed; 0 failed;" "$test_log"; then
      if (( schedule_status == 0 )); then
        schedule_status=1
      fi
      printf 'storage exact test did not pass exactly once: %s\n' \
        "$test_name" >&2
      [[ ! -s "$test_log" ]] || tail -n 40 "$test_log" >&2
      break
    fi
  done <"$selected_manifest"
  (( schedule_status == 0 )) || return "$schedule_status"
  timing_cache_updated_digest="$(
    update_storage_timing_cache "$selected_manifest" "$log_dir" "$selected_count"
  )"
  STORAGE_TEST_SELECTION_JSON="$(
    jq -c --arg digest "$timing_cache_updated_digest" \
      '.timing_cache.updated_sha256 = $digest' \
      <<<"$STORAGE_TEST_SELECTION_JSON"
  )"

  printf 'storage exact-test pool: selected=%s weighted=%s regular=%s deferred=%s ignored=%s parallelism=%s\n' \
    "$selected_count" "$weighted_count" "$regular_count" "$deferred_count" \
    "$ignored_count" "$THREADS"
}

run_nova_curated() {
  local test_binary
  test_binary="$(canonical_lib_test_binary)" || return
  Z00Z_BOOTSTRAP_LIB_TEST_BINARY="$test_binary" \
    Z00Z_BOOTSTRAP_SELECTED_TESTS_MANIFEST="$RUN_DIR/storage-selected-tests.tsv" \
    Z00Z_BOOTSTRAP_SELECTED_TEST_LOG_DIR="$RUN_DIR/storage-exact-tests" \
    run_budgeted execution "$NOVA_VERIFICATION" curated
}

run_plonky3_smoke() {
  local exact_name id packet_status=0 test_name transcript
  local -a tests=(
    test_source_sha_binding
    test_complete_air_enables_evidence
    test_poseidon_hash_binds_shape
    test_recursive_event_source_framing_rejects_public_aliases
    test_root_authority_fails_closed
    test_security_derivation_rejects_drift
    test_security_budget_rounding
  )
  for test_name in "${tests[@]}"; do
    exact_name="checkpoint::plonky3::tests::$test_name"
    id="$(
      awk -F '\t' -v test_name="$exact_name" '
        $2 == test_name {
          print $1
        }
      ' "$RUN_DIR/storage-selected-tests.tsv"
    )"
    transcript="$RUN_DIR/storage-exact-tests/$id.log"
    if [[ ! "$id" =~ ^[0-9]{6}$ ]] ||
      [[ ! -s "$transcript" ]] ||
      [[ "$(grep -Fc "test $exact_name ..." "$transcript")" != "1" ]] ||
      ! grep -Fq "test result: ok. 1 passed; 0 failed;" "$transcript"; then
      packet_status=1
      echo "preselected Plonky3 smoke evidence is invalid: $exact_name" >&2
    fi
  done
  (( packet_status == 0 )) || {
    echo "Plonky3 bounded smoke packet failed selection or execution" >&2
    return 1
  }
}

run_recursive_smoke() {
  local nova_pid plonky3_pid nova_status plonky3_status
  (
    run_nova_curated
  ) >"$RUN_DIR/nova-curated.log" 2>&1 &
  nova_pid=$!
  (
    run_plonky3_smoke
  ) >"$RUN_DIR/plonky3-smoke.log" 2>&1 &
  plonky3_pid=$!

  if wait "$nova_pid"; then
    nova_status=0
  else
    nova_status=$?
  fi
  if wait "$plonky3_pid"; then
    plonky3_status=0
  else
    plonky3_status=$?
  fi
  cat "$RUN_DIR/nova-curated.log"
  cat "$RUN_DIR/plonky3-smoke.log"
  if (( nova_status != 0 || plonky3_status != 0 )); then
    echo "bounded recursive smoke packet failed" >&2
    return 1
  fi
}

mkdir -p "$RUN_DIR"
: >"$STAGE_TIMINGS"
: >"$SOURCE_CHECKS"
check_root_test_output
initialize_source_authority
initialize_cache_root || {
  status=$?
  FAIL_REASON="cache_path_invalid"
  exit "$status"
}
CURRENT_STAGE="bootstrap_run_lock"
acquire_bootstrap_run_lock || {
  status=$?
  if [[ "$FAIL_REASON" == "stage failed" ]]; then
    FAIL_REASON="bootstrap_run_lock_failed"
  fi
  exit "$status"
}
initialize_cache_layout || {
  status=$?
  FAIL_REASON="cache_path_invalid"
  exit "$status"
}
initialize_cache_context_inputs

echo "=== $RUN_TIER: release-mode fail-fast tier ==="
run_stage storage_compile compile compile_storage_tests
if [[ "$MODE" == bootstrap ]]; then
  run_stage storage_non_recursive execution run_non_recursive_storage
  run_stage recursive_smoke execution run_recursive_smoke
fi

assert_source_stable "complete:full" full
CURRENT_STAGE="complete"
if [[ "$MODE" == bootstrap && "$CACHE_WARM_AT_START" == true ]] &&
  (( $(elapsed_ms) > WARM_TARGET_SECONDS * 1000 )); then
  CURRENT_STAGE="promotion_budget"
  FAIL_REASON="warm promotion budget exceeded"
  write_result fail "$FAIL_REASON"
  echo "bootstrap-fast exceeded the ${WARM_TARGET_SECONDS}s cache-warm promotion budget" >&2
  exit 75
fi
write_result pass success
if [[ "$MODE" == prewarm ]]; then
  echo "diagnostic only: compile cache warmed; no acceptance authority"
  echo "=== BOOTSTRAP PREWARM COMPLETE ==="
else
  echo "deferred by contract: real prover, exact-2000, wallet, benches/examples, broad workspace"
  echo "=== BOOTSTRAP COMPLETE ==="
fi
echo "evidence: $RUN_DIR/result.json"
