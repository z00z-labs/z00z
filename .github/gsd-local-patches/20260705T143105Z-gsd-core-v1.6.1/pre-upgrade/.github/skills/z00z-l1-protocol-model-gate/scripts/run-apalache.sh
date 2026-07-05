#!/bin/bash

# Run Apalache over Z00Z TLA+ models when present.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L1_STRICT:-0}"
APALACHE_BIN="${Z00Z_APALACHE_BIN:-}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
VERIFICATION_RUNTIME_ROOT="${Z00Z_VERIFICATION_RUNTIME_ROOT:-}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
SPECS_ROOT="${Z00Z_SPECS_ROOT:-$RUN_ROOT/specs$REPORT_STAMP}"
DEFAULT_APALACHE_OUT_DIR="$RUN_ROOT/verification$REPORT_STAMP/l1/apalache"
if [[ -n "$VERIFICATION_RUNTIME_ROOT" ]]; then
  DEFAULT_APALACHE_OUT_DIR="$VERIFICATION_RUNTIME_ROOT/l1/apalache"
fi
APALACHE_OUT_DIR="${Z00Z_APALACHE_OUT_DIR:-$DEFAULT_APALACHE_OUT_DIR}"
APALACHE_RUN_DIR="${Z00Z_APALACHE_RUN_DIR:-$APALACHE_OUT_DIR/runs}"
RUNTIME_CWD="${Z00Z_RUNTIME_CWD:-$ROOT_DIR}"
TOOLS_DIR="${Z00Z_VERIFY_TOOLS_DIR:-$ROOT_DIR/tools/formal_verification}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

mkdir -p "$RUNTIME_CWD"
cd "$RUNTIME_CWD"

log() {
  printf '[z00z-l1:apalache] %s\n' "$1"
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

resolve_repo_path() {
  local path="$1"
  case "$path" in
    /*) printf '%s\n' "$path" ;;
    *) printf '%s/%s\n' "$ROOT_DIR" "$path" ;;
  esac
}

SPECS_ROOT="$(resolve_repo_path "$SPECS_ROOT")"

if [[ -z "$APALACHE_BIN" ]]; then
  if [[ -x "$TOOLS_DIR/apalache/bin/apalache-mc" ]]; then
    APALACHE_BIN="$TOOLS_DIR/apalache/bin/apalache-mc"
  elif [[ -n "$(find "$TOOLS_DIR/apalache" -type f -name apalache-mc -perm -u+x 2>/dev/null | sort | head -n 1)" ]]; then
    APALACHE_BIN="$(find "$TOOLS_DIR/apalache" -type f -name apalache-mc -perm -u+x 2>/dev/null | sort | head -n 1)"
  elif command -v apalache-mc >/dev/null 2>&1; then
    APALACHE_BIN="$(command -v apalache-mc)"
  fi
fi

mapfile -t tla_files < <(find "$SPECS_ROOT/tla" -type f -name '*.tla' 2>/dev/null | sort)
if [[ "${#tla_files[@]}" -eq 0 ]]; then
  unknown_or_fail "no TLA models found under $SPECS_ROOT/tla"
  exit 0
fi

if [[ -z "$APALACHE_BIN" ]]; then
  unknown_or_fail "apalache-mc is not installed"
  exit 0
fi

mkdir -p "$APALACHE_OUT_DIR" "$APALACHE_RUN_DIR"

for model in "${tla_files[@]}"; do
  cfg="${model%.tla}.cfg"
  model_name="$(basename "${model%.tla}")"
  model_run_dir="$APALACHE_RUN_DIR/$model_name"
  mkdir -p "$model_run_dir"
  log "Apalache check $model"
  if [[ -f "$cfg" ]]; then
    z00z_profile_run_command command "apalache:$model_name" "$APALACHE_BIN" --out-dir="$APALACHE_OUT_DIR" --run-dir="$model_run_dir" check --config="$cfg" "$model"
  else
    z00z_profile_run_command command "apalache:$model_name" "$APALACHE_BIN" --out-dir="$APALACHE_OUT_DIR" --run-dir="$model_run_dir" check "$model"
  fi
done

log "MODEL_CHECKED: Apalache model checks completed successfully"
