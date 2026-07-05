#!/bin/bash

# Run Tamarin models when present.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L2_STRICT:-0}"
TAMARIN_CMD_OVERRIDE="${Z00Z_TAMARIN_CMD:-}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
VERIFICATION_RUNTIME_ROOT="${Z00Z_VERIFICATION_RUNTIME_ROOT:-}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
SPECS_ROOT="${Z00Z_SPECS_ROOT:-$RUN_ROOT/specs$REPORT_STAMP}"
DEFAULT_TAMARIN_TMPDIR="$RUN_ROOT/tmp$REPORT_STAMP/tamarin"
if [[ -n "$VERIFICATION_RUNTIME_ROOT" ]]; then
  DEFAULT_TAMARIN_TMPDIR="$VERIFICATION_RUNTIME_ROOT/l2/tamarin/tmp"
fi
TAMARIN_TMPDIR="${Z00Z_TAMARIN_TMPDIR:-$DEFAULT_TAMARIN_TMPDIR}"
RUNTIME_CWD="${Z00Z_RUNTIME_CWD:-$ROOT_DIR}"
TOOLS_DIR="${Z00Z_VERIFY_TOOLS_DIR:-$ROOT_DIR/tools/formal_verification}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

mkdir -p "$RUNTIME_CWD"
cd "$RUNTIME_CWD"

log() {
  printf '[z00z-l2:tamarin] %s\n' "$1"
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

have() {
  command -v "$1" >/dev/null 2>&1
}

declare -a TAMARIN_CMD=()

resolve_repo_path() {
  local path="$1"
  case "$path" in
    /*) printf '%s\n' "$path" ;;
    *) printf '%s/%s\n' "$ROOT_DIR" "$path" ;;
  esac
}

SPECS_ROOT="$(resolve_repo_path "$SPECS_ROOT")"

tamarin_available() {
  [[ -n "$TAMARIN_CMD_OVERRIDE" ]] && return 0
  [[ -x "$TOOLS_DIR/tamarin/bin/tamarin-prover-z00z" ]] && return 0
  [[ -x "$TOOLS_DIR/tamarin/bin/tamarin-prover" ]] && return 0
  have tamarin-prover
}

resolve_tamarin_cmd() {
  TAMARIN_CMD=()
  if [[ -n "$TAMARIN_CMD_OVERRIDE" ]]; then
    TAMARIN_CMD=("$TAMARIN_CMD_OVERRIDE")
  elif [[ -x "$TOOLS_DIR/tamarin/bin/tamarin-prover-z00z" ]]; then
    TAMARIN_CMD=("$TOOLS_DIR/tamarin/bin/tamarin-prover-z00z")
  elif [[ -x "$TOOLS_DIR/tamarin/bin/tamarin-prover" ]]; then
    TAMARIN_CMD=("$TOOLS_DIR/tamarin/bin/tamarin-prover")
  else
    TAMARIN_CMD=("tamarin-prover")
  fi
}

mapfile -t models < <(find "$SPECS_ROOT/tamarin" -type f -name '*.spthy' 2>/dev/null | sort)
if [[ "${#models[@]}" -eq 0 ]]; then
  unknown_or_fail "no Tamarin models found under $SPECS_ROOT/tamarin"
  exit 0
fi

if ! tamarin_available; then
  unknown_or_fail "tamarin-prover is not installed"
  exit 0
fi

mkdir -p "$TAMARIN_TMPDIR"
resolve_tamarin_cmd

for model in "${models[@]}"; do
  log "Tamarin $model"
  log_file="$(mktemp "$TAMARIN_TMPDIR/z00z-tamarin-run.XXXXXX")"
  set +e
  z00z_profile_run_command command "tamarin:$(basename "$model")" "${TAMARIN_CMD[@]}" --prove "$model" >"$log_file" 2>&1
  status=$?
  set -e
  cat "$log_file"
  if [[ "$status" -ne 0 ]]; then
    rm -f "$log_file"
    exit "$status"
  fi
  if grep -q "WARNING: the following wellformedness checks failed" "$log_file"; then
    rm -f "$log_file"
    unknown_or_fail "tamarin wellformedness warning in $model"
    continue
  fi
  rm -f "$log_file"
done

log "SECURITY_PROTOCOL_PROVED: Tamarin models completed successfully"
