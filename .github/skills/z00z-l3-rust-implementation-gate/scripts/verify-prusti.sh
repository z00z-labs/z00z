#!/bin/bash

# Run Prusti targets when explicitly configured.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L3_STRICT:-0}"
PRUSTI_BIN="${Z00Z_PRUSTI_BIN:-}"
PRUSTI_RUSTC="${Z00Z_PRUSTI_RUSTC:-}"
PACKAGES="${Z00Z_PRUSTI_PACKAGES:-}"
PROFILE_ARGS_TEXT="${Z00Z_CARGO_PROFILE_ARGS:---release}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
VERIFICATION_RUNTIME_ROOT="${Z00Z_VERIFICATION_RUNTIME_ROOT:-$RUN_ROOT/verification$REPORT_STAMP}"
DEFAULT_PRUSTI_ROOT="$VERIFICATION_RUNTIME_ROOT/prusti"
PRUSTI_ROOT="${Z00Z_PRUSTI_ROOT:-$DEFAULT_PRUSTI_ROOT}"
FEATURE_FLAG="${Z00Z_ALL_FEATURES_FLAG-}"
RUNTIME_CWD="${Z00Z_RUNTIME_CWD:-$ROOT_DIR}"
TOOLS_DIR="${Z00Z_VERIFY_TOOLS_DIR:-$ROOT_DIR/tools/formal_verification}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

mkdir -p "$RUNTIME_CWD"
cd "$RUNTIME_CWD"

log() {
  printf '[z00z-l3:prusti] %s\n' "$1"
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

find_tool() {
  local name="$1"
  local configured="$2"
  if [[ -n "$configured" && -x "$configured" ]]; then
    printf '%s\n' "$configured"
  elif command -v "$name" >/dev/null 2>&1; then
    command -v "$name"
  else
    find "$TOOLS_DIR/prusti" -type f -name "$name" -perm -u+x 2>/dev/null | sort | head -n 1 || true
  fi
}

resolve_repo_path() {
  local path="$1"
  case "$path" in
    /*) printf '%s\n' "$path" ;;
    *) printf '%s/%s\n' "$ROOT_DIR" "$path" ;;
  esac
}

PRUSTI_BIN="$(find_tool cargo-prusti "$PRUSTI_BIN")"
PRUSTI_RUSTC="$(find_tool prusti-rustc "$PRUSTI_RUSTC")"
PRUSTI_ROOT="$(resolve_repo_path "$PRUSTI_ROOT")"

if [[ -z "$PRUSTI_BIN" && -z "$PRUSTI_RUSTC" ]]; then
  unknown_or_fail "Prusti is not installed"
  exit 0
fi

ran=0
profile_args=()
if [[ -n "$PROFILE_ARGS_TEXT" ]]; then
  read -r -a profile_args <<<"$PROFILE_ARGS_TEXT"
fi

feature_args=()
if [[ -n "$FEATURE_FLAG" ]]; then
  feature_args+=("$FEATURE_FLAG")
fi

mapfile -t files < <(find "$PRUSTI_ROOT" -type f -name '*.rs' 2>/dev/null | sort)
for file in "${files[@]}"; do
  if [[ -z "$PRUSTI_RUSTC" ]]; then
    unknown_or_fail "prusti-rustc is required for standalone target $file"
    continue
  fi
  log "prusti-rustc $file"
  z00z_profile_run_command command "prusti:file:$file" "$PRUSTI_RUSTC" --edition=2021 "$file"
  ran=1
done

for package in $PACKAGES; do
  if [[ -z "$PRUSTI_BIN" ]]; then
    unknown_or_fail "cargo-prusti is required for package target $package"
    continue
  fi
  log "cargo-prusti ${PROFILE_ARGS_TEXT:-} -p $package ${FEATURE_FLAG:-}"
  z00z_profile_run_command command "prusti:package:$package" "$PRUSTI_BIN" "${profile_args[@]}" -p "$package" "${feature_args[@]}"
  ran=1
done

if [[ "$ran" -eq 0 ]]; then
  unknown_or_fail "no Prusti targets found under $PRUSTI_ROOT and no Z00Z_PRUSTI_PACKAGES configured"
fi
