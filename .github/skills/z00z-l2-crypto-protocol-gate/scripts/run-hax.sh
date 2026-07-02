#!/bin/bash

# Run HAX extraction targets against verifier-owned crypto surfaces.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L2_STRICT:-0}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
VERIFICATION_RUNTIME_ROOT="${Z00Z_VERIFICATION_RUNTIME_ROOT:-$RUN_ROOT/verification$REPORT_STAMP}"
DEFAULT_TARGETS_FILE="$VERIFICATION_RUNTIME_ROOT/hax/targets.json"
TARGETS_FILE="${Z00Z_HAX_TARGETS:-$DEFAULT_TARGETS_FILE}"
DEFAULT_HAX_OUTPUT_ROOT="$RUN_ROOT/verification$REPORT_STAMP/l2/hax"
DEFAULT_HAX_OUTPUT_ROOT="$VERIFICATION_RUNTIME_ROOT/l2/hax"
HAX_OUTPUT_ROOT="${Z00Z_HAX_OUTPUT_ROOT:-$DEFAULT_HAX_OUTPUT_ROOT}"
DEFAULT_HAX_TMPDIR="$VERIFICATION_RUNTIME_ROOT/hax/tmp"
HAX_TMPDIR="${Z00Z_HAX_TMPDIR:-$DEFAULT_HAX_TMPDIR}"
RUNTIME_CWD="${Z00Z_RUNTIME_CWD:-$ROOT_DIR}"
PROFILE_ARGS_TEXT="${Z00Z_CARGO_PROFILE_ARGS:---release}"
DISABLE_TIME_LIMITS="${Z00Z_DISABLE_TIME_LIMITS:-1}"
TOOLS_DIR="${Z00Z_VERIFY_TOOLS_DIR:-$ROOT_DIR/tools/formal_verification}"
LOCAL_OPAM_ROOT="${Z00Z_OPAM_ROOT:-$TOOLS_DIR/opam/root}"
LOCAL_OPAM_SWITCH="${Z00Z_VERIFY_OPAM_SWITCH:-z00z-verify}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

mkdir -p "$RUNTIME_CWD"
cd "$RUNTIME_CWD"

log() {
  printf '[z00z-l2:hax] %s\n' "$1"
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

ensure_hax_engine_path() {
  if have hax-engine; then
    return 0
  fi

  if [[ -x "$TOOLS_DIR/opam/bin/hax-engine" ]]; then
    PATH="$TOOLS_DIR/opam/bin:$PATH"
  fi

  if ! have hax-engine && [[ -x "$TOOLS_DIR/hax/target/release/hax-engine" ]]; then
    PATH="$TOOLS_DIR/hax/target/release:$PATH"
  fi

  if ! have hax-engine && have opam && OPAMROOT="$LOCAL_OPAM_ROOT" opam switch list --root "$LOCAL_OPAM_ROOT" --short 2>/dev/null | grep -Fxq "$LOCAL_OPAM_SWITCH"; then
    PATH="$TOOLS_DIR/opam/bin:$PATH"
  fi
}

resolve_repo_path() {
  local path="$1"
  case "$path" in
    /*) printf '%s\n' "$path" ;;
    *) printf '%s/%s\n' "$ROOT_DIR" "$path" ;;
  esac
}

resolve_output_abs() {
  local output_rel="$1"
  local trimmed="$output_rel"

  if [[ "$output_rel" == /* ]]; then
    printf '%s\n' "$output_rel"
    return 0
  fi

  trimmed="${trimmed#verification/hax/out/}"
  trimmed="${trimmed#out/}"
  trimmed="${trimmed#/}"
  if [[ -z "$trimmed" || "$trimmed" == "$output_rel" ]]; then
    trimmed="$(basename "$output_rel")"
  fi

  printf '%s/%s\n' "$HAX_OUTPUT_ROOT" "$trimmed"
}

TARGETS_FILE="$(resolve_repo_path "$TARGETS_FILE")"
HAX_TMPDIR="$(resolve_repo_path "$HAX_TMPDIR")"

profile_args=()
if [[ -n "$PROFILE_ARGS_TEXT" ]]; then
  read -r -a profile_args <<<"$PROFILE_ARGS_TEXT"
fi

if [[ ! -f "$TARGETS_FILE" ]]; then
  unknown_or_fail "HAX targets file not found: $TARGETS_FILE"
  exit 0
fi

if ! cargo hax --help >/dev/null 2>&1; then
  unknown_or_fail "cargo-hax is not installed"
  exit 0
fi

mkdir -p "$HAX_TMPDIR"
ensure_hax_engine_path

mapfile -t targets < <(
  python3 - "$TARGETS_FILE" <<'PY'
import json
import pathlib
import sys

data = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
for item in data.get("targets", []):
    print(
        "\t".join(
            [
                item["name"],
                item["manifest_path"],
                item["package"],
                item["backend"],
                item["output"],
                str(item.get("timeout_secs", 0)),
            ]
        )
    )
PY
)

if [[ "${#targets[@]}" -eq 0 ]]; then
  unknown_or_fail "no HAX targets configured"
  exit 0
fi

ran=0
run_hax_cmd() {
  local label="$1"
  local timeout_secs="$2"
  shift 2

  if [[ "$DISABLE_TIME_LIMITS" == "1" || "$timeout_secs" -le 0 ]]; then
    z00z_profile_run_command command "$label" env TMPDIR="$HAX_TMPDIR" TMP="$HAX_TMPDIR" TEMP="$HAX_TMPDIR" "$@"
    return "$?"
  fi

  z00z_profile_run_command command "$label" env TMPDIR="$HAX_TMPDIR" TMP="$HAX_TMPDIR" TEMP="$HAX_TMPDIR" timeout --foreground "${timeout_secs}s" "$@"
  return "$?"
}

for record in "${targets[@]}"; do
  IFS=$'\t' read -r name manifest_rel package backend output_rel timeout_secs <<<"$record"
  manifest_abs="$(resolve_repo_path "$manifest_rel")"
  output_abs="$(resolve_output_abs "$output_rel")"

  if [[ ! -f "$manifest_abs" ]]; then
    log "UNKNOWN: manifest not found for HAX target $name ($manifest_rel)"
    continue
  fi

  cmd=(cargo hax -C --manifest-path "$manifest_abs" -p "$package")
  if [[ "${#profile_args[@]}" -gt 0 ]]; then
    cmd+=("${profile_args[@]}")
  fi
  cmd+=(";")
  case "$backend" in
    json)
      mkdir -p "$(dirname "$output_abs")"
      cmd+=(json -o "$output_abs")
      ;;
    easycrypt)
      cmd+=(into easycrypt)
      ;;
    *)
      log "UNKNOWN: unsupported HAX backend $backend for target $name"
      continue
      ;;
  esac

  log "$name backend=$backend package=$package"
  set +e
  run_hax_cmd "hax:$name" "$timeout_secs" "${cmd[@]}"
  status=$?
  set -e
  if [[ "$status" -eq 0 ]]; then
    if [[ "$backend" == "easycrypt" && ! -d "$output_abs" ]]; then
      log "UNKNOWN: EasyCrypt extraction output missing at ${output_abs#"$ROOT_DIR"/}"
      continue
    fi
    ran=1
    continue
  fi
  if [[ "$status" -eq 124 ]]; then
    log "UNKNOWN: timeout after ${timeout_secs}s for HAX target $name"
    continue
  fi
  exit "$status"
done

if [[ "$ran" -eq 0 ]]; then
  unknown_or_fail "no HAX targets completed successfully"
fi

log "TESTED: HAX extraction targets completed successfully"
