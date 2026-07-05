#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L2_STRICT:-0}"
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
  printf '[z00z-code-logic:saw] %s\n' "$1"
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

if ! command -v saw >/dev/null 2>&1; then
  unknown_or_fail "saw is not installed"
  exit 0
fi

if [[ -z "${SAW_RUST_LIBRARY_PATH:-}" || ! -d "${SAW_RUST_LIBRARY_PATH:-}" ]]; then
  unknown_or_fail "SAW_RUST_LIBRARY_PATH is not configured"
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
    if target.get("tool") != "saw":
        continue
    target_id = target.get("id")
    manifest = target.get("manifest_path")
    script = target.get("saw_script")
    if target_id and manifest and script:
        print("\t".join([str(target_id), str(manifest), str(script)]))
PY
)

if [[ "${#targets[@]}" -eq 0 ]]; then
  unknown_or_fail "no SAW proof targets declared"
  exit 0
fi

profile_args=()
if [[ -n "$PROFILE_ARGS_TEXT" ]]; then
  read -r -a profile_args <<<"$PROFILE_ARGS_TEXT"
fi

mkdir -p "$VERIFICATION_ROOT/code-to-logic/build" "$VERIFICATION_ROOT/code-to-logic/target"

for target in "${targets[@]}"; do
  IFS=$'\t' read -r target_id manifest script <<<"$target"
  manifest_abs="$ROOT_DIR/$manifest"
  script_abs="$ROOT_DIR/$script"
  crate_dir="$(cd "$(dirname "$manifest_abs")" && pwd)"
  target_dir="$VERIFICATION_ROOT/code-to-logic/target/$target_id"
  json_out="$VERIFICATION_ROOT/code-to-logic/build/$target_id.linked-mir.json"

  log "cargo saw-build --manifest-path $manifest"
  z00z_profile_run_command command "saw-build:$target_id" \
    bash -lc "cd '$crate_dir' && cargo saw-build ${profile_args[*]} --manifest-path '$manifest_abs' --lib --target-dir '$target_dir'"

  linked_json="$(find "$target_dir" -type f -name '*.linked-mir.json' | sort | head -n 1 || true)"
  if [[ -z "$linked_json" ]]; then
    unknown_or_fail "cargo saw-build did not emit linked MIR JSON for $target_id"
    exit 0
  fi

  cp "$linked_json" "$json_out"

  log "SAW $script"
  z00z_profile_run_command command "saw:$target_id" saw "$script_abs"
done

log "FORMALLY_PROVED: SAW proof scripts completed successfully"
