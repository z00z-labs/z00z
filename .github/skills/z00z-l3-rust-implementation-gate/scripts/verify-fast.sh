#!/bin/bash

# Run the fast Rust implementation gate.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
FEATURE_FLAG="${Z00Z_ALL_FEATURES_FLAG-}"
PROFILE_ARGS_TEXT="${Z00Z_CARGO_PROFILE_ARGS:---release}"
VENDOR_ROOT="${Z00Z_VENDOR_ROOT:-$ROOT_DIR/crates/z00z_crypto/tari}"
NEXTEST_MODE="${Z00Z_L3_USE_NEXTEST:-auto}"
DRY_RUN=0

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

usage() {
  cat <<'EOF'
Usage: verify-fast.sh [OPTIONS]

Options:
  --dry-run          Print commands without running them.
  -h, --help         Show this help.

Environment:
  Z00Z_ALL_FEATURES_FLAG   Cargo feature flag. Default: disabled; set to --all-features only for
                           explicit non-production feature sweeps.
  Z00Z_CARGO_PROFILE_ARGS  Cargo profile args for heavy runs. Default: --release.
  Z00Z_VENDOR_ROOT         Vendor root excluded from workspace package selection.
  Z00Z_INCLUDE_VENDOR=1    Do not exclude vendored workspace packages.
  Z00Z_L3_USE_NEXTEST      auto|0|1. Default: auto. Under orchestrator, auto falls back to
                           cargo test so test binaries inherit run-root env and keep artifacts
                           inside reports/.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

cd "$ROOT_DIR"

gate_failed=0

log() {
  printf '[z00z-l3:fast] %s\n' "$1"
}

build_runtime_env() {
  local -a env_cmd=(env)
  local key
  local keys=(
    CARGO_TARGET_DIR
    TMPDIR
    TMP
    TEMP
    XDG_CACHE_HOME
    XDG_STATE_HOME
    PYTHONPYCACHEPREFIX
    PIP_CACHE_DIR
    NPM_CONFIG_CACHE
    MYPY_CACHE_DIR
    RUFF_CACHE_DIR
    UV_CACHE_DIR
    Z00Z_VERIFICATION_RUN_ROOT
    Z00Z_VERIFICATION_TMPDIR
    Z00Z_RUNTIME_CWD_ROOT
    Z00Z_RUN_CACHE_ROOT
    Z00Z_SYSTEM_TMPDIR
    Z00Z_SIMULATOR_CACHE_ROOT
    Z00Z_SIMULATOR_STORAGE_ROOT
  )

  for key in "${keys[@]}"; do
    if [[ -v "$key" ]]; then
      env_cmd+=("${key}=${!key}")
    fi
  done

  printf '%s\0' "${env_cmd[@]}"
}

run_cmd() {
  local label="$1"
  shift
  local -a runtime_env=()
  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf 'DRY'
    printf ' %q' "$@"
    printf '\n'
    return 0
  fi
  mapfile -d '' -t runtime_env < <(build_runtime_env)
  z00z_profile_run_command command "$label" "${runtime_env[@]}" "$@"
}

feature_args=()
if [[ -n "$FEATURE_FLAG" ]]; then
  feature_args+=("$FEATURE_FLAG")
fi

profile_args=()
if [[ -n "$PROFILE_ARGS_TEXT" ]]; then
  read -r -a profile_args <<<"$PROFILE_ARGS_TEXT"
fi

vendor_excludes=()
fmt_packages=()
collect_vendor_excludes() {
  if ! command -v python3 >/dev/null 2>&1; then
    log "SKIPPED: python3 unavailable; cannot derive workspace package selection"
    return 0
  fi

  while IFS=$'\t' read -r kind package; do
    [[ -n "$package" ]] || continue
    case "$kind" in
      fmt)
        fmt_packages+=(-p "$package")
        ;;
      exclude)
        vendor_excludes+=(--exclude "$package")
        ;;
    esac
  done < <(cargo metadata --no-deps --format-version 1 \
      | python3 -c '
import json
import pathlib
import sys

vendor = pathlib.Path(sys.argv[1]).resolve()
include_vendor = sys.argv[2] == "1"
data = json.load(sys.stdin)
workspace_members = set(data.get("workspace_members", []))
for package in data.get("packages", []):
    if package["id"] not in workspace_members:
        continue
    manifest = pathlib.Path(package["manifest_path"]).resolve()
    try:
        manifest.relative_to(vendor)
        in_vendor = True
    except ValueError:
        in_vendor = False

    if include_vendor or not in_vendor:
        print("fmt\t%s" % package["name"])
    if not include_vendor and in_vendor:
        print("exclude\t%s" % package["name"])
' "$VENDOR_ROOT" "${Z00Z_INCLUDE_VENDOR:-0}")

  if [[ "${#vendor_excludes[@]}" -gt 0 ]]; then
    log "excluding vendored tari packages from workspace gates"
  fi
}

collect_vendor_excludes

run_workspace_test_lane() {
  local workspace_label="$1"
  local wallet_label="$2"
  local simulator_label="$3"
  shift 3
  local -a test_args=("$@")
  local -a serial_test_args=()

  if [[ "${#test_args[@]}" -gt 0 ]]; then
    serial_test_args=("${test_args[@]}")
    if [[ "${serial_test_args[0]}" != "--" ]]; then
      serial_test_args+=(--)
    fi
  else
    serial_test_args=(--)
  fi
  serial_test_args+=(--test-threads 1)

  if ! run_cmd "$workspace_label" cargo test --workspace --exclude z00z_simulator --exclude z00z_wallets \
    "${vendor_excludes[@]}" "${profile_args[@]}" "${feature_args[@]}" "${test_args[@]}"; then
    gate_failed=1
  fi

  if ! run_cmd "$wallet_label" cargo test -p z00z_wallets \
    "${profile_args[@]}" "${feature_args[@]}" "${serial_test_args[@]}"; then
    gate_failed=1
  fi

  if ! run_cmd "$simulator_label" cargo test -p z00z_simulator \
    "${profile_args[@]}" "${feature_args[@]}" "${serial_test_args[@]}"; then
    gate_failed=1
  fi
}

should_use_nextest() {
  case "$NEXTEST_MODE" in
    1|true|TRUE|yes|YES|always|ALWAYS|force|FORCE)
      return 0
      ;;
    0|false|FALSE|no|NO|never|NEVER)
      return 1
      ;;
    auto|AUTO)
      if [[ -n "${Z00Z_VERIFICATION_RUN_ROOT:-}" ]]; then
        return 1
      fi
      if command -v cargo-nextest >/dev/null 2>&1 || cargo nextest --version >/dev/null 2>&1; then
        return 0
      fi
      return 1
      ;;
    *)
      log "UNKNOWN: invalid Z00Z_L3_USE_NEXTEST value '$NEXTEST_MODE'; using cargo test fallback"
      return 1
      ;;
  esac
}

log "cargo fmt"
if [[ "${Z00Z_INCLUDE_VENDOR:-0}" == "1" || "${#fmt_packages[@]}" -eq 0 ]]; then
  if ! run_cmd "fmt:workspace" cargo fmt --all --check; then
    gate_failed=1
  fi
elif ! run_cmd "fmt:workspace-packages" cargo fmt --check "${fmt_packages[@]}"; then
  gate_failed=1
fi

log "cargo clippy"
if ! run_cmd "clippy:workspace" cargo clippy --workspace "${vendor_excludes[@]}" "${profile_args[@]}" --all-targets "${feature_args[@]}" -- -D warnings; then
  gate_failed=1
fi

if should_use_nextest; then
  log "cargo nextest"
  if ! run_cmd "nextest:workspace" cargo nextest run --workspace "${vendor_excludes[@]}" "${profile_args[@]}" "${feature_args[@]}"; then
    gate_failed=1
  fi
  log "cargo nextest ignored-only"
  if ! run_cmd "nextest:workspace:ignored" cargo nextest run --workspace "${vendor_excludes[@]}" "${profile_args[@]}" "${feature_args[@]}" --run-ignored ignored-only; then
    gate_failed=1
  fi
else
  if [[ "${NEXTEST_MODE,,}" == "auto" && -n "${Z00Z_VERIFICATION_RUN_ROOT:-}" ]]; then
    log "cargo test fallback (nextest disabled under orchestrator to preserve run-root env in test binaries)"
  else
    log "cargo test fallback"
  fi
  run_workspace_test_lane \
    "test:workspace:no-wallet-sim" \
    "test:z00z_wallets:serial" \
    "test:z00z_simulator:serial"
  log "cargo test ignored-only"
  run_workspace_test_lane \
    "test:workspace:ignored:no-wallet-sim" \
    "test:z00z_wallets:ignored:serial" \
    "test:z00z_simulator:ignored:serial" \
    -- --ignored
fi

log "cargo doc tests"
if ! run_cmd "doc-tests:workspace" cargo test --workspace "${vendor_excludes[@]}" "${profile_args[@]}" "${feature_args[@]}" --doc; then
  gate_failed=1
fi

exit "$gate_failed"
