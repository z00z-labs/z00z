#!/usr/bin/env bash

# shellcheck disable=SC2086,SC2178 # Optional flags split intentionally; Bash namerefs target arrays.

# ./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run
# The optional max-safe stage uses prebuilt-artifact reuse when enabled.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
LONG_FILE="${Z00Z_FULL_VERIFY_REPORT:-$ROOT_DIR/reports/full_verify-report-long-running-tests.txt}"
LONG_TEST_SEC="${LONG_TEST_SEC:-20}"
ISOLATED_TEST_TIMEOUT_SEC="${ISOLATED_TEST_TIMEOUT_SEC:-120}"
RUST_TEST_WARN_MS="${RUST_TEST_WARN_MS:-$((LONG_TEST_SEC * 1000))}"
RUST_TEST_CRIT_MS="${RUST_TEST_CRIT_MS:-$RUST_TEST_WARN_MS}"
FEATURES="${Z00Z_VERIFY_FEATURES:-}"
ALL_FEATURES_FLAG="${Z00Z_ALL_FEATURES_FLAG:---all-features}"
DENY_WARNINGS="${Z00Z_DENY_WARNINGS:-1}"
RUN_TARGETS="${Z00Z_RUN_TARGETS:-1}"
TARGET_MANIFEST="${Z00Z_TARGET_MANIFEST:-$SCRIPT_DIR/runnable_targets.toml}"
MAX_SAFE_RUN="${Z00Z_MAX_SAFE_RUN:-0}"
START_TS="$(date +%s)"

print_elapsed() {
  local end_ts
  local elapsed
  local hh
  local mm
  local ss

  end_ts="$(date +%s)"
  elapsed=$((end_ts - START_TS))
  hh=$((elapsed / 3600))
  mm=$(((elapsed % 3600) / 60))
  ss=$((elapsed % 60))

  printf '[full-verify] elapsed: %02d:%02d:%02d (%ss)\n' "$hh" "$mm" "$ss" "$elapsed"
}

on_exit() {
  local code=$?

  set +e
  if [[ "$code" -ne 0 ]]; then
    mkdir -p "$(dirname "$LONG_FILE")"
    if [[ -f "$LONG_FILE" ]]; then
      if ! grep -Fqx 'IsolatedMeasurements:' "$LONG_FILE"; then
        printf '\nIsolatedMeasurements:\n' >> "$LONG_FILE"
      fi
      if ! grep -Fq 'skipped_due_to_early_failure' "$LONG_FILE"; then
        printf 'skipped_due_to_early_failure\n' >> "$LONG_FILE"
      fi
      if ! grep -Fq 'full_verify_exit_code=' "$LONG_FILE"; then
        printf 'script | fail | full_verify_exit_code=%s\n' "$code" >> "$LONG_FILE"
      fi
      printf '[full-verify] partial long-running report path: %s\n' "$LONG_FILE"
    fi
  fi
  set -e

  print_elapsed
}

trap on_exit EXIT

usage() {
  cat <<'EOF'
Usage:
  ./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh [--max-safe-run]

Note:
  --max-safe-run enables the optional max-safe stage.
  That stage runs via prebuilt-artifact reuse.

What it does:
  1. cargo fmt --check
  2. cargo clippy --workspace --release --all-targets --all-features -- -D warnings
  3. cargo test --workspace --release --lib --bins --tests --examples --all-features
  4. cargo test --workspace --release --all-features --doc
  5. cargo bench --workspace --all-features --no-run
  6. run whitelisted bins/examples from the skill-local runnable_targets.toml
  7. collect tests slower than LONG_TEST_SEC into Z00Z_FULL_VERIFY_REPORT
     (default: reports/full_verify-report-long-running-tests.txt)
  8. optionally run the heavy simulator replay-contract suite
  9. optionally run a max-safe target sweep across workspace crates
     using prebuilt-artifact reuse for that stage
     maximum wide run lib/bin/example/test/bench across the workspace

Environment:
  LONG_TEST_SEC=20           Slow-test threshold in seconds
  ISOLATED_TEST_TIMEOUT_SEC=120
                              Timeout for isolated single-test measurement runs
  RUST_TEST_WARN_MS=20000    Rust test harness warning threshold in ms
  RUST_TEST_CRIT_MS=20000    Rust test harness critical threshold in ms
  Z00Z_VERIFY_FEATURES=...   Optional comma-separated features when not using --all-features
  Z00Z_ALL_FEATURES_FLAG=... Usually --all-features; set empty to disable
  Z00Z_DENY_WARNINGS=1       Add -D warnings to the workspace clippy stage
  Z00Z_RUN_TARGETS=1         Run enabled whitelist entries from the skill-local manifest
  Z00Z_TARGET_MANIFEST=...   Override the runnable target manifest path
  Z00Z_FULL_VERIFY_REPORT=... Override the long-running test report path
  Z00Z_MAX_SAFE_RUN=1        Also run the optional max-safe target sweep
                              using prebuilt-artifact reuse for that stage

Options:
  --max-safe-run             Run the extra max-safe target sweep stage
                              using prebuilt-artifact reuse for that stage
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    --max-safe-run)
      MAX_SAFE_RUN=1
      shift
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

cd "$ROOT_DIR"

# Keep libtest long-running diagnostics aligned with LONG_TEST_SEC.
export RUST_TEST_TIME_INTEGRATION="${RUST_TEST_WARN_MS},${RUST_TEST_CRIT_MS}"
export RUST_TEST_TIME_UNIT="${RUST_TEST_WARN_MS},${RUST_TEST_CRIT_MS}"
export RUST_TEST_TIME_DOCTEST="${RUST_TEST_WARN_MS},${RUST_TEST_CRIT_MS}"

log() {
  printf '[full-verify] %s\n' "$1"
}

append_cmd() {
  local cmd="$1"
  local part="$2"
  if [[ -n "$part" ]]; then
    printf -v "$cmd" '%s %s' "${!cmd}" "$part"
  fi
}

feature_args() {
  if [[ -n "$FEATURES" ]]; then
    printf -- '--features %s' "$FEATURES"
  fi
}

append_feature_args() {
  local -n cmd_ref="$1"
  if [[ -n "$FEATURES" ]]; then
    cmd_ref+=(--features "$FEATURES")
  fi
}

append_vendor_clippy_excludes() {
  local -n cmd_ref="$1"
  local workspace_names
  local cargo_toml
  local package_name

  workspace_names="$(cargo metadata --no-deps --format-version 1 2>/dev/null | jq -r '.packages[].name' 2>/dev/null || true)"
  [[ -n "$workspace_names" ]] || return 0

  while IFS= read -r cargo_toml; do
    package_name="$(sed -n 's/^name = "\([^"]*\)"/\1/p' "$cargo_toml" | head -n 1)"
    if [[ -n "$package_name" ]] && printf '%s\n' "$workspace_names" | grep -Fxq "$package_name"; then
      cmd_ref+=(--exclude "$package_name")
    fi
  done < <(find "$ROOT_DIR/crates/z00z_crypto/tari" -name Cargo.toml | sort)
}

init_long_report() {
  mkdir -p "$(dirname "$LONG_FILE")"
  cat >"$LONG_FILE" <<EOF
Generated: $(date -Iseconds)
ThresholdSeconds: $LONG_TEST_SEC
Workspace: $ROOT_DIR
MeasurementMode: live-harness-plus-isolated-single-test
Note: cargo test may print "running for over N seconds" during a parallel suite run; this report keeps only harness signals where N >= LONG_TEST_SEC, and that signal can differ from isolated single-test timing.

HarnessSignals:

TaskFailures:
EOF
}

append_report_line() {
  printf '%s\n' "$1" >> "$LONG_FILE"
}

start_isolated_section() {
  if ! grep -Fqx 'IsolatedMeasurements:' "$LONG_FILE"; then
    printf '\nIsolatedMeasurements:\n' >> "$LONG_FILE"
  fi
}

run_and_watch() {
  local label="$1"
  shift

  set +e
  "$@" 2>&1 | tee >(python3 "$SCRIPT_DIR/live_report_filter.py" --report "$LONG_FILE" --label "$label" --threshold "$LONG_TEST_SEC")
  local status=${PIPESTATUS[0]}
  set -e
  return "$status"
}

run_fmt_check() {
  set +e
  cargo fmt --check 2> >(grep -Fv 'unstable features are only available in nightly channel.' >&2)
  local status=$?
  set -e
  return "$status"
}

collect_test_bins() {
  local bins_file="$1"
  local extra_features
  extra_features="$(feature_args)"
  cargo test --workspace --release --all-targets $ALL_FEATURES_FLAG $extra_features \
    --no-run --message-format=json-render-diagnostics \
    | sed -n 's/.*"executable":"\([^"]*\)".*/\1/p' \
    | sed 's/\\"/"/g' \
    | sort -u > "$bins_file"
}

measure_long_tests() {
  local bins_file
  local candidates_file
  local list_file
  local time_file
  local list_status
  local found=0

  bins_file="$(mktemp)"
  candidates_file="$(mktemp)"
  list_file="$(mktemp)"
  time_file="$(mktemp)"
  trap 'rm -f "$bins_file" "$candidates_file" "$list_file" "$time_file"' RETURN

  start_isolated_section
  sed -n 's/^.* harness: test \(.*\) has been running for over .*$/\1/p' "$LONG_FILE" \
    | sort -u > "$candidates_file"
  if [[ ! -s "$candidates_file" ]]; then
    append_report_line 'none'
    return 0
  fi

  collect_test_bins "$bins_file"

  while IFS= read -r exe; do
    [[ -n "$exe" && -x "$exe" ]] || continue

    set +e
    timeout 5 "$exe" --list >"$list_file" 2>/dev/null
    list_status=$?
    set -e
    if [[ "$list_status" -ne 0 ]] || ! grep -q ': test$' "$list_file"; then
      continue
    fi

    while IFS= read -r line; do
      [[ "$line" == *": test" ]] || continue
      local test_name
      local elapsed
      local run_status
      test_name="${line%%: test}"
      if ! grep -Fqx "$test_name" "$candidates_file"; then
        continue
      fi

      set +e
      timeout --signal=TERM --kill-after=10 "$ISOLATED_TEST_TIMEOUT_SEC" \
        /usr/bin/time -f '%e' -o "$time_file" "$exe" --exact "$test_name" \
        >/dev/null 2>&1
      run_status=$?
      set -e
      if [[ "$run_status" -eq 124 ]]; then
        printf '%s | timeout>%ss | %s\n' \
          "$(basename "$exe")" "$ISOLATED_TEST_TIMEOUT_SEC" "$test_name" >> "$LONG_FILE"
        found=1
        continue
      fi
      if [[ "$run_status" -ne 0 ]]; then
        continue
      fi

      elapsed="$(cat "$time_file")"

      if awk -v elapsed="$elapsed" -v threshold="$LONG_TEST_SEC" 'BEGIN { exit !(elapsed > threshold) }'; then
        printf '%s | %ss | %s\n' "$(basename "$exe")" "$elapsed" "$test_name" >> "$LONG_FILE"
        found=1
      fi
    done < "$list_file"
  done < "$bins_file"

  if [[ "$found" == "0" ]]; then
    append_report_line 'none'
  fi
}

EXTRA_FEATURES="$(feature_args)"

init_long_report
log "long-running report will be updated live at $LONG_FILE"

# Disabled for incremental local verification runs.
# Re-enable explicitly when a cold-build check is required.
# log 'cargo clean'
# cargo clean

log 'cargo fmt --check'
run_fmt_check

log 'cargo clippy --workspace --release --all-targets'
clippy_cmd=(cargo clippy --workspace --release --all-targets)
if [[ -n "$ALL_FEATURES_FLAG" ]]; then
  clippy_cmd+=("$ALL_FEATURES_FLAG")
fi
append_feature_args clippy_cmd
append_vendor_clippy_excludes clippy_cmd
if [[ "$DENY_WARNINGS" == "1" ]]; then
  clippy_cmd+=(-- -D warnings)
fi
"${clippy_cmd[@]}"

log 'cargo test --workspace --release --lib --bins --tests --examples'
# Keep the generic gate focused on lib/bin/example/test surfaces.
# Bench targets are still compiled explicitly in the dedicated cargo bench --no-run stage.
Z00Z_SETTLEMENT_PROOF_NOTE_SCOPE="${Z00Z_SETTLEMENT_PROOF_NOTE_SCOPE:-skip}" \
  run_and_watch "workspace-test" cargo test --workspace --release --lib --bins --tests --examples $ALL_FEATURES_FLAG $EXTRA_FEATURES

log 'cargo test --workspace --release --doc'
cargo test --workspace --release $ALL_FEATURES_FLAG $EXTRA_FEATURES --doc

log 'cargo bench --workspace --no-run'
cargo bench --workspace $ALL_FEATURES_FLAG $EXTRA_FEATURES --no-run

if [[ "$RUN_TARGETS" == "1" ]]; then
  log 'run whitelisted bins/examples'
  if [[ -n "$FEATURES" ]]; then
    python3 "$SCRIPT_DIR/run_runnable_targets.py" \
      --manifest "$TARGET_MANIFEST" \
      --reuse-build \
      --features "$FEATURES"
  else
    python3 "$SCRIPT_DIR/run_runnable_targets.py" \
      --manifest "$TARGET_MANIFEST" \
      --reuse-build
  fi
fi

if [[ "$MAX_SAFE_RUN" == "1" ]]; then
  log 'run heavy simulator replay-contract suite'
  run_and_watch "sim-heavy-replay" \
    cargo test -p z00z_simulator --release $ALL_FEATURES_FLAG $EXTRA_FEATURES \
      --test scenario_1 test_stage4_digest_replay_heavy -- --ignored

  log 'run max-safe target sweep'
  max_cmd=(python3 "$SCRIPT_DIR/run_max_safe_targets.py" --manifest "$TARGET_MANIFEST" --keep-going --reuse-build --prebuilt-only)
  if [[ -n "$FEATURES" ]]; then
    max_cmd+=(--features "$FEATURES")
  fi
  if [[ -n "$ALL_FEATURES_FLAG" ]]; then
    max_cmd+=(--all-features)
  fi
  run_and_watch "max-safe" "${max_cmd[@]}"
fi

log 'collect long-running tests'
measure_long_tests

log "wrote slow test inventory to $LONG_FILE"
printf '[full-verify] long-running report path for user: %s\n' "$LONG_FILE"
