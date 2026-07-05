#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L2_STRICT:-0}"
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
SPECS_ROOT="$(resolve_repo_path "${Z00Z_SPECS_ROOT:-$RUN_ROOT/specs$REPORT_STAMP}")"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

log() {
  printf '[z00z-code-logic:cryptol] %s\n' "$1"
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

if ! command -v cryptol >/dev/null 2>&1; then
  unknown_or_fail "cryptol is not installed"
  exit 0
fi

mapfile -t specs < <(find "$VERIFICATION_ROOT/code-to-logic/cryptol" "$SPECS_ROOT/cryptol" -type f -name '*.cry' 2>/dev/null | sort)
if [[ "${#specs[@]}" -eq 0 ]]; then
  unknown_or_fail "no Cryptol specs found"
  exit 0
fi

for spec in "${specs[@]}"; do
  log "Cryptol $spec"
  z00z_profile_run_command command "cryptol:$(basename "$spec")" cryptol --ignore-cryptolrc --command ":load $spec" --command ":quit"
done

log "TESTED: Cryptol specs executed successfully"
