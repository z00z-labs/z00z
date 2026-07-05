#!/bin/bash

# Run Alloy model checks when a headless Alloy runner is configured.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L1_STRICT:-0}"
ALLOY_RUNNER="${Z00Z_ALLOY_RUNNER:-}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
SPECS_ROOT="${Z00Z_SPECS_ROOT:-$RUN_ROOT/specs$REPORT_STAMP}"
TOOLS_DIR="${Z00Z_VERIFY_TOOLS_DIR:-$ROOT_DIR/tools/formal_verification}"
ALLOY_JAR="${Z00Z_ALLOY_JAR:-$TOOLS_DIR/alloy/org.alloytools.alloy.dist.jar}"
LOCAL_ALLOY_RUNNER="$TOOLS_DIR/alloy/bin/alloy-headless-z00z"
RUNTIME_CWD="${Z00Z_RUNTIME_CWD:-$ROOT_DIR}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

mkdir -p "$RUNTIME_CWD"
cd "$RUNTIME_CWD"

log() {
  printf '[z00z-l1:alloy] %s\n' "$1"
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

mapfile -t alloy_files < <(find "$SPECS_ROOT/alloy" -type f -name '*.als' 2>/dev/null | sort)
if [[ "${#alloy_files[@]}" -eq 0 ]]; then
  unknown_or_fail "no Alloy models found under $SPECS_ROOT/alloy"
  exit 0
fi

if [[ -n "$ALLOY_RUNNER" ]]; then
  for model in "${alloy_files[@]}"; do
    log "Alloy runner $model"
    z00z_profile_run_command command "alloy:runner:$(basename "$model")" "$ALLOY_RUNNER" "$model"
  done
  log "MODEL_CHECKED: Alloy model checks completed successfully"
  exit 0
fi

if [[ -x "$LOCAL_ALLOY_RUNNER" ]]; then
  for model in "${alloy_files[@]}"; do
    log "Alloy headless $model"
    z00z_profile_run_command command "alloy:headless:$(basename "$model")" "$LOCAL_ALLOY_RUNNER" "$model"
  done
  log "MODEL_CHECKED: Alloy model checks completed successfully"
  exit 0
fi

if [[ -f "$ALLOY_JAR" ]]; then
  log "Alloy jar found at $ALLOY_JAR"
  unknown_or_fail "headless Alloy execution requires Z00Z_ALLOY_RUNNER; the stock jar is GUI/API oriented"
else
  unknown_or_fail "Alloy jar not found at $ALLOY_JAR"
fi
