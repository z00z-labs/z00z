#!/bin/bash

# Run configured dudect or timing-leakage harnesses.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
STRICT="${Z00Z_L2_STRICT:-0}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
VERIFICATION_RUNTIME_ROOT="${Z00Z_VERIFICATION_RUNTIME_ROOT:-$RUN_ROOT/verification$REPORT_STAMP}"
DEFAULT_DUDECT_ROOT="$VERIFICATION_RUNTIME_ROOT/dudect"
DUDECT_ROOT="${Z00Z_DUDECT_ROOT:-$DEFAULT_DUDECT_ROOT}"

cd "$ROOT_DIR"

log() {
  printf '[z00z-l2:dudect] %s\n' "$1"
}

skip_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "SKIPPED: $message"
}

resolve_repo_path() {
  local path="$1"
  case "$path" in
    /*) printf '%s\n' "$path" ;;
    *) printf '%s/%s\n' "$ROOT_DIR" "$path" ;;
  esac
}

DUDECT_ROOT="$(resolve_repo_path "$DUDECT_ROOT")"

mapfile -t harnesses < <(find "$DUDECT_ROOT" -type f -name '*.sh' 2>/dev/null | sort)
if [[ "${#harnesses[@]}" -gt 0 ]]; then
  for harness in "${harnesses[@]}"; do
    log "dudect harness $harness"
    bash "$harness"
  done
  exit 0
fi

mapfile -t timing_benches < <(find crates -path '*/benches/*.rs' -type f \( -name '*timing*.rs' -o -name '*constant*time*.rs' \) 2>/dev/null | sort)
if [[ "${#timing_benches[@]}" -eq 0 ]]; then
  skip_or_fail "no dudect harnesses under $DUDECT_ROOT or timing benches found"
  exit 0
fi

log "Timing benches exist; run explicit benches through L4 run-constant-time.sh"
printf '%s\n' "${timing_benches[@]}"
