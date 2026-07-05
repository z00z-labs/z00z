#!/bin/bash

# Run TLC/SANY over Z00Z TLA+ models when present.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L1_STRICT:-0}"
TOOLS_DIR="${Z00Z_VERIFY_TOOLS_DIR:-$ROOT_DIR/tools/formal_verification}"
TLA_JAR="${Z00Z_TLA2TOOLS_JAR:-$TOOLS_DIR/tla/tla2tools.jar}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
VERIFICATION_RUNTIME_ROOT="${Z00Z_VERIFICATION_RUNTIME_ROOT:-}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
SPECS_ROOT="${Z00Z_SPECS_ROOT:-$RUN_ROOT/specs$REPORT_STAMP}"
DEFAULT_TLA_STATE_ROOT="$RUN_ROOT/verification$REPORT_STAMP/l1/tla-states"
if [[ -n "$VERIFICATION_RUNTIME_ROOT" ]]; then
  DEFAULT_TLA_STATE_ROOT="$VERIFICATION_RUNTIME_ROOT/l1/tla-states"
fi
TLA_STATE_ROOT="${Z00Z_TLA_STATE_ROOT:-$DEFAULT_TLA_STATE_ROOT}"
TLA_USER_OUTPUT_ROOT="${Z00Z_TLA_USER_OUTPUT_ROOT:-$TLA_STATE_ROOT/user}"
RUNTIME_CWD="${Z00Z_RUNTIME_CWD:-$ROOT_DIR}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

mkdir -p "$RUNTIME_CWD"
cd "$RUNTIME_CWD"

log() {
  printf '[z00z-l1:tla] %s\n' "$1"
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

mapfile -t tla_files < <(find "$SPECS_ROOT/tla" -type f -name '*.tla' 2>/dev/null | sort)
if [[ "${#tla_files[@]}" -eq 0 ]]; then
  unknown_or_fail "no TLA models found under $SPECS_ROOT/tla"
  exit 0
fi

if [[ ! -f "$TLA_JAR" ]]; then
  unknown_or_fail "tla2tools.jar not found at $TLA_JAR"
  exit 0
fi

mkdir -p "$TLA_STATE_ROOT" "$TLA_USER_OUTPUT_ROOT"

for model in "${tla_files[@]}"; do
  cfg="${model%.tla}.cfg"
  model_name="$(basename "${model%.tla}")"
  meta_dir="$TLA_STATE_ROOT/$model_name"
  user_file="$TLA_USER_OUTPUT_ROOT/$model_name.log"
  mkdir -p "$meta_dir"
  if [[ -f "$cfg" ]]; then
    log "TLC $model"
    z00z_profile_run_command command "tla:tlc:$model_name" java -cp "$TLA_JAR" tlc2.TLC -workers auto -metadir "$meta_dir" -userFile "$user_file" -config "$cfg" "$model"
  else
    log "SANY syntax check $model"
    z00z_profile_run_command command "tla:sany:$model_name" java -cp "$TLA_JAR" tla2sany.SANY "$model"
  fi
done

log "MODEL_CHECKED: TLA+ models completed successfully"
