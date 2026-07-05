#!/bin/bash

# Run Loom-instrumented tests when the repository contains them.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L3_STRICT:-0}"
VENDOR_ROOT="${Z00Z_VENDOR_ROOT:-$ROOT_DIR/crates/z00z_crypto/tari}"
PROFILE_ARGS_TEXT="${Z00Z_CARGO_PROFILE_ARGS:---release}"
FEATURE_FLAG="${Z00Z_ALL_FEATURES_FLAG-}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

cd "$ROOT_DIR"

log() {
  printf '[z00z-l3:loom] %s\n' "$1"
}

if ! rg -q "loom::|cfg\\(loom\\)|cfg\\(.*loom" crates tests 2>/dev/null; then
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: no Loom instrumentation found" >&2
    exit 1
  fi
  log "UNKNOWN: no Loom instrumentation found"
  exit 0
fi

vendor_excludes=()
profile_args=()
if [[ -n "$PROFILE_ARGS_TEXT" ]]; then
  read -r -a profile_args <<<"$PROFILE_ARGS_TEXT"
fi

feature_args=()
if [[ -n "$FEATURE_FLAG" ]]; then
  feature_args+=("$FEATURE_FLAG")
fi

if [[ "${Z00Z_INCLUDE_VENDOR:-0}" != "1" && -d "$VENDOR_ROOT" ]]; then
  while IFS= read -r package; do
    [[ -n "$package" ]] || continue
    vendor_excludes+=(--exclude "$package")
  done < <(
    cargo metadata --no-deps --format-version 1 \
      | python3 -c '
import json
import pathlib
import sys

vendor = pathlib.Path(sys.argv[1]).resolve()
data = json.load(sys.stdin)
for package in data.get("packages", []):
    manifest = pathlib.Path(package["manifest_path"]).resolve()
    try:
        manifest.relative_to(vendor)
    except ValueError:
        continue
    print(package["name"])
' "$VENDOR_ROOT"
  )
fi

log "RUSTFLAGS=--cfg loom cargo test ${PROFILE_ARGS_TEXT:-} --workspace ${FEATURE_FLAG:-}"
z00z_profile_run_command command "loom:workspace" env RUSTFLAGS="--cfg loom" cargo test "${profile_args[@]}" --workspace "${vendor_excludes[@]}" "${feature_args[@]}" loom
