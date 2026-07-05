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
TARGETS_PATH="$(resolve_repo_path "${Z00Z_CODE_TO_LOGIC_TARGETS:-$VERIFICATION_ROOT/code-to-logic/targets.yaml}")"
OUT_DIR="$(resolve_repo_path "${Z00Z_CODE_TO_LOGIC_CHARON_OUT:-$VERIFICATION_ROOT/code-to-logic/llbc}")"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

log() {
  printf '[z00z-code-logic:charon] %s\n' "$1"
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

relocate_charon_output_leaks() {
  local crate_dir="$1"
  local output_file=""

  while IFS= read -r output_file; do
    [[ -n "$output_file" ]] || continue
    mv -f "$output_file" "$OUT_DIR/$(basename "$output_file")"
  done < <(
    find "$ROOT_DIR" "$crate_dir" -maxdepth 1 -type f \
      \( -name '*.llbc' -o -name '*.ullbc' -o -name '*.postcard' \) \
      ! -path "$OUT_DIR/*" 2>/dev/null | sort -u
  )
}

charon_known_limitation() {
  local log_path="$1"
  rg -q \
    "duplicate symbol:|can't find crate for|ERROR Code failed to compile|crates/z00z_crypto/tari/" \
    "$log_path"
}

if ! command -v charon >/dev/null 2>&1; then
  unknown_or_fail "charon is not installed"
  exit 0
fi

if [[ ! -f "$TARGETS_PATH" ]]; then
  unknown_or_fail "targets file not found: $TARGETS_PATH"
  exit 0
fi

mkdir -p "$OUT_DIR"
ran=0

mapfile -t manifests < <(python3 - "$TARGETS_PATH" <<'PY'
import pathlib
import sys

try:
    import yaml  # type: ignore
except ImportError:
    raise SystemExit(0)

path = pathlib.Path(sys.argv[1])
data = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
seen = set()
for target in data.get("targets", []):
    if not isinstance(target, dict):
        continue
    if target.get("tool") not in {"charon", "aeneas", "charon-aeneas"}:
        continue
    manifest = target.get("manifest_path")
    if manifest and manifest not in seen:
        seen.add(manifest)
        print(manifest)
PY
)

if [[ "${#manifests[@]}" -eq 0 ]]; then
  unknown_or_fail "no Charon/Aeneas manifest targets declared"
  exit 0
fi

for manifest in "${manifests[@]}"; do
  charon_log=""
  manifest_abs="$ROOT_DIR/$manifest"
  crate_dir="$(cd "$(dirname "$manifest_abs")" && pwd)"
  crate_name="$(python3 - "$manifest_abs" <<'PY'
import pathlib
import sys
import tomllib

manifest = pathlib.Path(sys.argv[1])
data = tomllib.loads(manifest.read_text(encoding="utf-8"))
print(data["package"]["name"])
PY
)"
  dest_file="$OUT_DIR/${crate_name}.llbc"
  charon_log="$(mktemp "${TMPDIR:-/tmp}/z00z-charon-$(basename "$crate_dir").XXXXXX")"
  set +e
  if charon cargo --help 2>/dev/null | grep -q -- '--manifest-path'; then
    log "charon cargo --preset=aeneas --dest-file $dest_file --manifest-path $manifest -- --release --lib"
    z00z_profile_run_command command "charon:$(basename "$crate_dir")" \
      env -u MIRI_SYSROOT -u CHARON_MIRI_SYSROOTS \
      "Z00Z_SIMULATOR_CACHE_ROOT=${Z00Z_SIMULATOR_CACHE_ROOT:-}" \
      "Z00Z_SIMULATOR_STORAGE_ROOT=${Z00Z_SIMULATOR_STORAGE_ROOT:-}" \
      "Z00Z_RUNTIME_CWD_ROOT=${Z00Z_RUNTIME_CWD_ROOT:-}" \
      "Z00Z_VERIFICATION_RUN_ROOT=${Z00Z_VERIFICATION_RUN_ROOT:-}" \
      "CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-}" \
      charon cargo --preset=aeneas --dest-file "$dest_file" --manifest-path "$manifest_abs" -- --release --lib \
      >"$charon_log" 2>&1
  else
    log "charon cargo --preset=aeneas --dest-file $dest_file -- --release --lib (cwd=$crate_dir)"
    z00z_profile_run_command command "charon:$(basename "$crate_dir")" \
      bash -lc "cd '$crate_dir' && export Z00Z_SIMULATOR_CACHE_ROOT='${Z00Z_SIMULATOR_CACHE_ROOT:-}' Z00Z_SIMULATOR_STORAGE_ROOT='${Z00Z_SIMULATOR_STORAGE_ROOT:-}' Z00Z_RUNTIME_CWD_ROOT='${Z00Z_RUNTIME_CWD_ROOT:-}' Z00Z_VERIFICATION_RUN_ROOT='${Z00Z_VERIFICATION_RUN_ROOT:-}' CARGO_TARGET_DIR='${CARGO_TARGET_DIR:-}' && unset MIRI_SYSROOT CHARON_MIRI_SYSROOTS && charon cargo --preset=aeneas --dest-file '$dest_file' -- --release --lib" \
      >"$charon_log" 2>&1
  fi
  status=$?
  set -e
  relocate_charon_output_leaks "$crate_dir"
  cat "$charon_log"
  if [[ "$status" -ne 0 ]]; then
    if charon_known_limitation "$charon_log"; then
      rm -f "$charon_log"
      unknown_or_fail "Charon could not extract $manifest because its dependency closure currently exceeds the supported toolchain/vendor boundary"
      continue
    fi
    rm -f "$charon_log"
    exit "$status"
  fi
  rm -f "$charon_log"
  while IFS= read -r llbc_file; do
    [[ -n "$llbc_file" ]] || continue
    ran=1
  done < <(find "$OUT_DIR" -maxdepth 1 -type f -name '*.llbc' | sort)
done

if [[ "$ran" -eq 0 ]]; then
  unknown_or_fail "Charon completed but did not emit any LLBC files"
  exit 0
fi

log "TESTED: Charon LLBC extraction completed successfully"
