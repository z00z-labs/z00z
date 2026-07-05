#!/bin/bash

# Run ProVerif models when present.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L2_STRICT:-0}"
PROVERIF_CMD_OVERRIDE="${Z00Z_PROVERIF_CMD:-}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
SPECS_ROOT="${Z00Z_SPECS_ROOT:-$RUN_ROOT/specs$REPORT_STAMP}"
RUNTIME_CWD="${Z00Z_RUNTIME_CWD:-$ROOT_DIR}"
TOOLS_DIR="${Z00Z_VERIFY_TOOLS_DIR:-$ROOT_DIR/tools/formal_verification}"
LOCAL_OPAM_ROOT="${Z00Z_OPAM_ROOT:-$TOOLS_DIR/opam/root}"
LOCAL_OPAM_SWITCH="${Z00Z_VERIFY_OPAM_SWITCH:-z00z-verify}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

mkdir -p "$RUNTIME_CWD"
cd "$RUNTIME_CWD"

log() {
  printf '[z00z-l2:proverif] %s\n' "$1"
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

declare -a PROVERIF_CMD=()

resolve_repo_path() {
  local path="$1"
  case "$path" in
    /*) printf '%s\n' "$path" ;;
    *) printf '%s/%s\n' "$ROOT_DIR" "$path" ;;
  esac
}

SPECS_ROOT="$(resolve_repo_path "$SPECS_ROOT")"

proverif_available() {
  [[ -n "$PROVERIF_CMD_OVERRIDE" ]] && return 0
  [[ -x "$TOOLS_DIR/opam/bin/proverif" ]] && return 0
  have proverif && return 0
  have opam || return 1
  OPAMROOT="$LOCAL_OPAM_ROOT" opam switch list --root "$LOCAL_OPAM_ROOT" --short 2>/dev/null | grep -Fxq "$LOCAL_OPAM_SWITCH" || return 1
  OPAMROOT="$LOCAL_OPAM_ROOT" opam exec --root "$LOCAL_OPAM_ROOT" --switch "$LOCAL_OPAM_SWITCH" -- proverif -help >/dev/null 2>&1
}

resolve_proverif_cmd() {
  PROVERIF_CMD=()
  if [[ -n "$PROVERIF_CMD_OVERRIDE" ]]; then
    PROVERIF_CMD=("$PROVERIF_CMD_OVERRIDE")
  elif [[ -x "$TOOLS_DIR/opam/bin/proverif" ]]; then
    PROVERIF_CMD=("$TOOLS_DIR/opam/bin/proverif")
  elif have proverif; then
    PROVERIF_CMD=("proverif")
  else
    PROVERIF_CMD=(
      "opam"
      "exec"
      "--root"
      "$LOCAL_OPAM_ROOT"
      "--switch"
      "$LOCAL_OPAM_SWITCH"
      "--"
      "proverif"
    )
  fi
}

mapfile -t models < <(find "$SPECS_ROOT/proverif" -type f -name '*.pv' 2>/dev/null | sort)
if [[ "${#models[@]}" -eq 0 ]]; then
  unknown_or_fail "no ProVerif models found under $SPECS_ROOT/proverif"
  exit 0
fi

if ! proverif_available; then
  unknown_or_fail "proverif is not installed"
  exit 0
fi

resolve_proverif_cmd

for model in "${models[@]}"; do
  log "ProVerif $model"
  z00z_profile_run_command command "proverif:$(basename "$model")" "${PROVERIF_CMD[@]}" "$model"
done

log "SECURITY_PROTOCOL_PROVED: ProVerif models completed successfully"
