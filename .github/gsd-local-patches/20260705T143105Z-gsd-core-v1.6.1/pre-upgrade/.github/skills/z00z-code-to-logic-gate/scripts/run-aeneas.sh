#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L2_STRICT:-0}"
BACKEND="${Z00Z_AENEAS_BACKEND:-lean}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"

resolve_repo_path() {
  local path="$1"
  if [[ "$path" == /* ]]; then
    printf '%s\n' "$path"
  else
    printf '%s\n' "$ROOT_DIR/$path"
  fi
}

VERIFICATION_ROOT="$(resolve_repo_path "${Z00Z_VERIFICATION_RUNTIME_ROOT:-$RUN_ROOT/verification$REPORT_STAMP}")"
LLBC_DIR="$(resolve_repo_path "${Z00Z_CODE_TO_LOGIC_CHARON_OUT:-$VERIFICATION_ROOT/code-to-logic/llbc}")"
OUT_DIR="$(resolve_repo_path "${Z00Z_CODE_TO_LOGIC_AENEAS_OUT:-$VERIFICATION_ROOT/code-to-logic/aeneas}")"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

log() {
  printf '[z00z-code-logic:aeneas] %s\n' "$1"
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

if ! command -v aeneas >/dev/null 2>&1; then
  unknown_or_fail "aeneas is not installed"
  exit 0
fi

mapfile -t llbc_files < <(find "$LLBC_DIR" -type f -name '*.llbc' 2>/dev/null | sort)
if [[ "${#llbc_files[@]}" -eq 0 ]]; then
  unknown_or_fail "no LLBC files found for Aeneas translation"
  exit 0
fi

mkdir -p "$OUT_DIR"

for llbc_file in "${llbc_files[@]}"; do
  stem="$(basename "$llbc_file" .llbc)"
  out_file="$OUT_DIR/${stem}.${BACKEND}.out"
  log "aeneas -backend $BACKEND $llbc_file"
  z00z_profile_run_command command "aeneas:$stem" bash -lc "aeneas -backend '$BACKEND' '$llbc_file' > '$out_file'"
done

log "TESTED: Aeneas translation artifacts generated successfully"
