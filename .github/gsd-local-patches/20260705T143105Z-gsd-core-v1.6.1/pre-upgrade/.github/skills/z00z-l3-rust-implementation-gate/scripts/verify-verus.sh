#!/bin/bash

# Run Verus targets when explicitly present.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L3_STRICT:-0}"
VERUS_BIN="${Z00Z_VERUS_BIN:-verus}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
VERIFICATION_RUNTIME_ROOT="${Z00Z_VERIFICATION_RUNTIME_ROOT:-$RUN_ROOT/verification$REPORT_STAMP}"
DEFAULT_VERUS_ROOT="$VERIFICATION_RUNTIME_ROOT/verus"
VERUS_ROOT="${Z00Z_VERUS_ROOT:-$DEFAULT_VERUS_ROOT}"
RUNTIME_CWD="${Z00Z_RUNTIME_CWD:-$ROOT_DIR}"
TOOLS_DIR="${Z00Z_VERIFY_TOOLS_DIR:-$ROOT_DIR/tools/formal_verification}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

mkdir -p "$RUNTIME_CWD"
cd "$RUNTIME_CWD"

log() {
  printf '[z00z-l3:verus] %s\n' "$1"
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

resolve_verus_bin() {
  if [[ -n "$VERUS_BIN" && "$VERUS_BIN" != "verus" && -x "$VERUS_BIN" ]]; then
    printf '%s\n' "$VERUS_BIN"
    return 0
  fi
  if command -v "$VERUS_BIN" >/dev/null 2>&1; then
    command -v "$VERUS_BIN"
    return 0
  fi
  find "$TOOLS_DIR/verus" -type f -name verus -perm -u+x 2>/dev/null | sort | head -n 1 || true
}

resolve_repo_path() {
  local path="$1"
  case "$path" in
    /*) printf '%s\n' "$path" ;;
    *) printf '%s/%s\n' "$ROOT_DIR" "$path" ;;
  esac
}

VERUS_ROOT="$(resolve_repo_path "$VERUS_ROOT")"

mapfile -t verus_files < <(find "$VERUS_ROOT" -type f -name '*.rs' 2>/dev/null | sort)
if [[ "${#verus_files[@]}" -eq 0 ]]; then
  unknown_or_fail "no Verus targets found under $VERUS_ROOT"
  exit 0
fi

VERUS_BIN="$(resolve_verus_bin)"

if [[ -z "$VERUS_BIN" ]]; then
  unknown_or_fail "verus is not installed"
  exit 0
fi

for file in "${verus_files[@]}"; do
  log "Verus $file"
  z00z_profile_run_command command "verus:$file" "$VERUS_BIN" "$file"
done

log "FORMALLY_PROVED: Verus targets completed successfully"
