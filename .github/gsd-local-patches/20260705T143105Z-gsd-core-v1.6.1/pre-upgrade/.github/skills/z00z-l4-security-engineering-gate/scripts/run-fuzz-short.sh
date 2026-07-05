#!/bin/bash

# Run short fuzz sessions for generated Z00Z fuzz targets.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L4_STRICT:-0}"
FUZZ_DIR="${Z00Z_FUZZ_DIR:-}"
TIME_SECS="${Z00Z_FUZZ_TIME_SECS:-60}"
WALL_TIMEOUT_SECS="${Z00Z_FUZZ_WALL_TIMEOUT_SECS:-0}"
DISABLE_TIME_LIMITS="${Z00Z_DISABLE_TIME_LIMITS:-1}"
INCLUDE_VENDOR="${Z00Z_INCLUDE_VENDOR:-0}"
FUZZ_EXTRA_ARGS="${Z00Z_FUZZ_EXTRA_ARGS:--print_final_stats=1 -verbosity=0}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
FUZZ_RUNTIME_ROOT="${Z00Z_FUZZ_RUNTIME_ROOT:-$RUN_ROOT/fuzz$REPORT_STAMP}"
FUZZ_TARGET_DIR="${Z00Z_FUZZ_TARGET_DIR:-$FUZZ_RUNTIME_ROOT/target}"
FUZZ_CORPUS_ROOT="${Z00Z_FUZZ_CORPUS_ROOT:-$FUZZ_RUNTIME_ROOT/corpus}"
FUZZ_ARTIFACT_ROOT="${Z00Z_FUZZ_ARTIFACT_ROOT:-$FUZZ_RUNTIME_ROOT/artifacts}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

cd "$ROOT_DIR"

log() {
  printf '[z00z-l4:fuzz] %s\n' "$1"
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

if ! cargo fuzz --help >/dev/null 2>&1; then
  unknown_or_fail "cargo-fuzz is not installed"
  exit 0
fi

resolve_root_path() {
  local path="$1"
  case "$path" in
    /*) printf '%s\n' "$path" ;;
    *) printf '%s/%s\n' "$ROOT_DIR" "$path" ;;
  esac
}

FUZZ_TARGET_DIR="$(resolve_root_path "$FUZZ_TARGET_DIR")"
FUZZ_CORPUS_ROOT="$(resolve_root_path "$FUZZ_CORPUS_ROOT")"
FUZZ_ARTIFACT_ROOT="$(resolve_root_path "$FUZZ_ARTIFACT_ROOT")"

discover_manifests() {
  if [[ -n "$FUZZ_DIR" ]]; then
    if [[ -f "$FUZZ_DIR" ]]; then
      printf '%s\n' "$FUZZ_DIR"
    elif [[ -f "$FUZZ_DIR/Cargo.toml" ]]; then
      printf '%s\n' "$FUZZ_DIR/Cargo.toml"
    fi
    return 0
  fi

  if [[ -f "$FUZZ_RUNTIME_ROOT/Cargo.toml" ]]; then
    printf '%s\n' "$FUZZ_RUNTIME_ROOT/Cargo.toml"
    return 0
  fi

  find "$ROOT_DIR/crates" -mindepth 3 -maxdepth 3 -type f -path '*/fuzz/Cargo.toml' 2>/dev/null \
    | while IFS= read -r manifest; do
        case "$manifest" in
          "$ROOT_DIR"/crates/z00z_crypto/tari/*)
            [[ "$INCLUDE_VENDOR" == "1" ]] || continue
            ;;
        esac
        printf '%s\n' "$manifest"
      done \
    | sort
}

targets_for_manifest() {
  local manifest="$1"
  python3 - "$manifest" <<'PY'
import pathlib
import sys
import tomllib

data = tomllib.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
for entry in data.get("bin", []):
    name = entry.get("name")
    if name:
        print(name)
PY
}

manifest_slug() {
  local manifest="$1"
  manifest="${manifest#"$ROOT_DIR"/}"
  printf '%s\n' "$manifest" | tr '/[:space:]' '__' | tr -cd '[:alnum:]_.-'
}

relocate_local_output() {
  local from_path="$1"
  local to_path="$2"

  [[ -e "$from_path" ]] || return 0
  [[ "$from_path" != "$to_path" ]] || return 0
  case "$to_path" in
    "$from_path"/*)
      return 0
      ;;
  esac

  mkdir -p "$(dirname "$to_path")"
  if [[ -e "$to_path" ]]; then
    to_path="${to_path}-$(date -u +%Y%m%dT%H%M%SZ)"
  fi
  mv "$from_path" "$to_path"
}

relocate_unexpected_local_outputs() {
  local fuzz_dir="$1"
  local manifest_key="$2"

  relocate_local_output "$fuzz_dir/artifacts" "$FUZZ_ARTIFACT_ROOT/$manifest_key/local-artifacts"
  relocate_local_output "$fuzz_dir/corpus" "$FUZZ_CORPUS_ROOT/$manifest_key/local-corpus"
  relocate_local_output "$fuzz_dir/coverage" "$FUZZ_ARTIFACT_ROOT/$manifest_key/local-coverage"
  relocate_local_output "$fuzz_dir/target" "$FUZZ_TARGET_DIR/leaked-$manifest_key"
}

mapfile -t manifests < <(discover_manifests)
if [[ "${#manifests[@]}" -eq 0 ]]; then
  unknown_or_fail "no fuzz/Cargo.toml manifests found"
  exit 0
fi

read -r -a fuzz_extra_args <<<"$FUZZ_EXTRA_ARGS"
ran=0

run_fuzz_cmd() {
  local label="$1"
  local timeout_secs="$2"
  shift 2

  if [[ "$DISABLE_TIME_LIMITS" == "1" || "$timeout_secs" -le 0 ]]; then
    z00z_profile_run_command command "$label" "$@"
    return "$?"
  fi

  z00z_profile_run_command command "$label" timeout --foreground "${timeout_secs}s" "$@"
  return "$?"
}

for manifest in "${manifests[@]}"; do
  fuzz_dir="$(cd "$(dirname "$manifest")" && pwd)"
  manifest_key="$(manifest_slug "$manifest")"
  mapfile -t targets < <(targets_for_manifest "$manifest")
  if [[ "${#targets[@]}" -eq 0 ]]; then
    log "UNKNOWN: no fuzz targets declared in ${manifest#"$ROOT_DIR"/}"
    continue
  fi

  cd "$fuzz_dir"
  for target in "${targets[@]}"; do
    corpus_dir="$FUZZ_CORPUS_ROOT/$manifest_key/$target"
    artifact_dir="$FUZZ_ARTIFACT_ROOT/$manifest_key/$target"
    mkdir -p "$corpus_dir" "$artifact_dir" "$FUZZ_TARGET_DIR"
    log "cargo +nightly fuzz run $target -- -max_total_time=$TIME_SECS $FUZZ_EXTRA_ARGS (${manifest#"$ROOT_DIR"/})"
    set +e
    run_fuzz_cmd "fuzz:$manifest_key:$target" "$WALL_TIMEOUT_SECS" cargo +nightly fuzz run \
      --release \
      --target-dir "$FUZZ_TARGET_DIR" \
      --fuzz-dir "$fuzz_dir" \
      "$target" \
      "$corpus_dir" \
      -- \
      -max_total_time="$TIME_SECS" \
      -artifact_prefix="${artifact_dir}/" \
      "${fuzz_extra_args[@]}"
    status=$?
    set -e
    if [[ "$status" -eq 0 ]]; then
      relocate_unexpected_local_outputs "$fuzz_dir" "$manifest_key"
      ran=1
      continue
    fi
    if [[ "$status" -eq 124 ]]; then
      relocate_unexpected_local_outputs "$fuzz_dir" "$manifest_key"
      log "UNKNOWN: timeout after ${WALL_TIMEOUT_SECS}s for fuzz target $target"
      continue
    fi
    relocate_unexpected_local_outputs "$fuzz_dir" "$manifest_key"
    exit "$status"
  done
done

if [[ "$ran" -eq 0 ]]; then
  unknown_or_fail "no fuzz targets completed successfully"
  exit 0
fi

log "TESTED: short fuzz sessions completed successfully"
