#!/bin/bash

# Run configured constant-time and timing-sensitive harnesses.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L4_STRICT:-0}"
BENCHES="${Z00Z_CONSTANT_TIME_BENCHES:-z00z_wallets:mac_timing}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
VERIFICATION_RUNTIME_ROOT="${Z00Z_VERIFICATION_RUNTIME_ROOT:-$RUN_ROOT/verification$REPORT_STAMP}"
DEFAULT_DUDECT_ROOT="$VERIFICATION_RUNTIME_ROOT/dudect"
DUDECT_ROOT="${Z00Z_DUDECT_ROOT:-$DEFAULT_DUDECT_ROOT}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

cd "$ROOT_DIR"

log() {
  printf '[z00z-l4:constant-time] %s\n' "$1"
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
ran=0

if [[ -d "$DUDECT_ROOT" ]]; then
  mapfile -t harnesses < <(find "$DUDECT_ROOT" -type f -name '*.sh' | sort)
  for harness in "${harnesses[@]}"; do
    log "dudect harness $harness"
    z00z_profile_run_command command "dudect:$(basename "$harness")" bash "$harness"
    ran=1
  done
fi

for item in $BENCHES; do
  package="${item%%:*}"
  bench="${item#*:}"
  if cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; pkg=sys.argv[1]; print(any(p["name"] == pkg for p in json.load(sys.stdin)["packages"]))' "$package" | grep -Fxq "True"; then
    if rg -q "name = \"$bench\"" "crates" -g Cargo.toml; then
      log "cargo bench -p $package --bench $bench"
      z00z_profile_run_command command "bench:$package:$bench" cargo bench -p "$package" --bench "$bench" -- --test
      ran=1
    fi
  fi
done

if [[ "$ran" -eq 0 && ! -d "$DUDECT_ROOT" ]]; then
  skip_or_fail "no configured constant-time harnesses ran"
fi

if [[ "$ran" -eq 1 ]]; then
  log "TESTED: constant-time harnesses completed successfully"
fi
