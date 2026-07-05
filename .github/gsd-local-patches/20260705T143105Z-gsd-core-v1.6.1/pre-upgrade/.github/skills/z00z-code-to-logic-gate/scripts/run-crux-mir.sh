#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L2_STRICT:-0}"
FEATURE_FLAG="${Z00Z_ALL_FEATURES_FLAG---all-features}"
PROFILE_ARGS_TEXT="${Z00Z_CARGO_PROFILE_ARGS:---release}"
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
TARGETS_PATH="$(resolve_repo_path "${Z00Z_CODE_TO_LOGIC_TARGETS:-$VERIFICATION_ROOT/code-to-logic/targets.yaml}")"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

log() {
  printf '[z00z-code-logic:crux] %s\n' "$1"
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

if ! cargo crux-test --help >/dev/null 2>&1; then
  unknown_or_fail "cargo-crux-test is not installed"
  exit 0
fi

if ! command -v crux-mir >/dev/null 2>&1 && [[ -z "${CRUX_MIR:-}" ]]; then
  unknown_or_fail "crux-mir runtime is not installed"
  exit 0
fi

if [[ ! -f "$TARGETS_PATH" ]]; then
  unknown_or_fail "targets file not found: $TARGETS_PATH"
  exit 0
fi

mapfile -t targets < <(python3 - "$TARGETS_PATH" <<'PY'
import pathlib
import sys

try:
    import yaml  # type: ignore
except ImportError:
    raise SystemExit(0)

path = pathlib.Path(sys.argv[1])
data = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
for target in data.get("targets", []):
    if not isinstance(target, dict):
        continue
    if target.get("tool") != "crux-mir":
        continue
    target_id = target.get("id")
    manifest = target.get("manifest_path")
    package = target.get("package", "")
    if target_id and (manifest or package):
        print("\t".join([str(target_id), str(manifest or ""), str(package)]))
PY
)

if [[ "${#targets[@]}" -eq 0 ]]; then
  unknown_or_fail "no Crux-MIR targets declared"
  exit 0
fi

feature_args=()
if [[ -n "$FEATURE_FLAG" ]]; then
  feature_args+=("$FEATURE_FLAG")
fi

profile_args=()
if cargo crux-test --help 2>/dev/null | grep -Eq -- '--release|--profile'; then
  if [[ -n "$PROFILE_ARGS_TEXT" ]]; then
    read -r -a profile_args <<<"$PROFILE_ARGS_TEXT"
  fi
fi

mkdir -p "$VERIFICATION_ROOT/code-to-logic/target"

for target in "${targets[@]}"; do
  IFS=$'\t' read -r target_id manifest package <<<"$target"
  target_dir="$VERIFICATION_ROOT/code-to-logic/target/$target_id"

  if [[ -n "$manifest" ]]; then
    manifest_abs="$ROOT_DIR/$manifest"
    crate_dir="$(cd "$(dirname "$manifest_abs")" && pwd)"
    log "cargo crux-test --manifest-path $manifest"
    z00z_profile_run_command command "crux:$target_id" \
      bash -lc "cd '$crate_dir' && export Z00Z_SIMULATOR_CACHE_ROOT='${Z00Z_SIMULATOR_CACHE_ROOT:-}' Z00Z_SIMULATOR_STORAGE_ROOT='${Z00Z_SIMULATOR_STORAGE_ROOT:-}' Z00Z_RUNTIME_CWD_ROOT='${Z00Z_RUNTIME_CWD_ROOT:-}' Z00Z_VERIFICATION_RUN_ROOT='${Z00Z_VERIFICATION_RUN_ROOT:-}' CARGO_TARGET_DIR='${CARGO_TARGET_DIR:-}' && cargo crux-test ${profile_args[*]} --manifest-path '$manifest_abs' --lib --target-dir '$target_dir' ${feature_args[*]}"
    continue
  fi

  log "cargo crux-test -p $package"
  z00z_profile_run_command command "crux:$target_id" \
    env \
    "Z00Z_SIMULATOR_CACHE_ROOT=${Z00Z_SIMULATOR_CACHE_ROOT:-}" \
    "Z00Z_SIMULATOR_STORAGE_ROOT=${Z00Z_SIMULATOR_STORAGE_ROOT:-}" \
    "Z00Z_RUNTIME_CWD_ROOT=${Z00Z_RUNTIME_CWD_ROOT:-}" \
    "Z00Z_VERIFICATION_RUN_ROOT=${Z00Z_VERIFICATION_RUN_ROOT:-}" \
    "CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-}" \
    cargo crux-test "${profile_args[@]}" -p "$package" --lib --target-dir "$target_dir" "${feature_args[@]}"
done

log "BOUNDED_VERIFIED: Crux-MIR targets completed successfully"
