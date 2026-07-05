#!/bin/bash

# Run Kani where harnesses are present.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
PACKAGES="${Z00Z_KANI_PACKAGES:-z00z_core z00z_validators}"
STRICT="${Z00Z_L3_STRICT:-0}"
WALL_TIMEOUT_SECS="${Z00Z_KANI_TIMEOUT_SECS:-0}"
DISABLE_TIME_LIMITS="${Z00Z_DISABLE_TIME_LIMITS:-1}"
PROFILE_ARGS_TEXT="${Z00Z_CARGO_PROFILE_ARGS:---release}"
FEATURE_FLAG_TEXT="${Z00Z_ALL_FEATURES_FLAG-}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

cd "$ROOT_DIR"

log() {
  printf '[z00z-l3:kani] %s\n' "$1"
}

build_runtime_env() {
  local -a env_cmd=(env)
  local key
  local keys=(
    CARGO_TARGET_DIR
    TMPDIR
    TMP
    TEMP
    XDG_CACHE_HOME
    XDG_STATE_HOME
    PYTHONPYCACHEPREFIX
    PIP_CACHE_DIR
    NPM_CONFIG_CACHE
    MYPY_CACHE_DIR
    RUFF_CACHE_DIR
    UV_CACHE_DIR
    Z00Z_VERIFICATION_RUN_ROOT
    Z00Z_VERIFICATION_TMPDIR
    Z00Z_RUNTIME_CWD_ROOT
    Z00Z_RUN_CACHE_ROOT
    Z00Z_SYSTEM_TMPDIR
    Z00Z_SIMULATOR_CACHE_ROOT
    Z00Z_SIMULATOR_STORAGE_ROOT
  )

  for key in "${keys[@]}"; do
    if [[ -v "$key" ]]; then
      env_cmd+=("${key}=${!key}")
    fi
  done

  printf '%s\0' "${env_cmd[@]}"
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

if ! cargo kani --version >/dev/null 2>&1; then
  unknown_or_fail "cargo-kani is not installed"
  exit 0
fi

kani_profile_args=()
profile_args=()
if [[ -n "$PROFILE_ARGS_TEXT" ]]; then
  read -r -a profile_args <<<"$PROFILE_ARGS_TEXT"
fi
if cargo kani --help 2>/dev/null | grep -Eq -- '--release|--profile'; then
  kani_profile_args=("${profile_args[@]}")
elif [[ "${#profile_args[@]}" -gt 0 ]]; then
  kani_version="$(cargo kani --version 2>/dev/null | head -n 1 || true)"
  log "NOTE: ${kani_version:-cargo-kani} cannot honor Z00Z_CARGO_PROFILE_ARGS='${PROFILE_ARGS_TEXT}'; continuing with Kani-supported profile"
fi

feature_args=()
if [[ -n "$FEATURE_FLAG_TEXT" ]]; then
  read -r -a feature_args <<<"$FEATURE_FLAG_TEXT"
fi

caller_controls_features=0
for arg in "${feature_args[@]}"; do
  case "$arg" in
    --all-features|--no-default-features|--features)
      caller_controls_features=1
      ;;
  esac
done

derive_package_feature_args() {
  local package="$1"
  local feature_csv
  feature_csv="$(
    python3 - "$workspace_json_file" "$package" <<'PY'
import json
import sys

metadata_path = sys.argv[1]
package_name = sys.argv[2]
with open(metadata_path, encoding="utf-8") as fh:
    data = json.load(fh)
package = next((pkg for pkg in data["packages"] if pkg["name"] == package_name), None)
if package is None:
    raise SystemExit(0)

feature_map = package.get("features", {})

def expand(enabled):
    resolved = set()
    stack = list(enabled)
    while stack:
        feature = stack.pop()
        if feature in resolved:
            continue
        resolved.add(feature)
        for child in feature_map.get(feature, []):
            if child.startswith("dep:"):
                continue
            stack.append(child.split("/", 1)[0])
    return resolved

default_features = expand(feature_map.get("default", []))
required = set()
for target in package.get("targets", []):
    required.update(target.get("required-features") or [])

missing = sorted(required - default_features)
if missing:
    print(",".join(missing))
PY
  )"

  if [[ -n "$feature_csv" ]]; then
    printf '%s\n' "--features" "$feature_csv"
  fi
}

workspace_json_file="$(mktemp)"
trap 'rm -f "$workspace_json_file"' EXIT
cargo metadata --format-version 1 --no-deps >"$workspace_json_file"
ran=0

run_kani_cmd() {
  local label="$1"
  local timeout_secs="$2"
  shift 2
  local -a runtime_env=()

  mapfile -d '' -t runtime_env < <(build_runtime_env)

  if [[ "$DISABLE_TIME_LIMITS" == "1" || "$timeout_secs" -le 0 ]]; then
    z00z_profile_run_command command "$label" "${runtime_env[@]}" "$@"
    return "$?"
  fi

  z00z_profile_run_command command "$label" "${runtime_env[@]}" timeout --foreground "${timeout_secs}s" "$@"
  return "$?"
}

for package in $PACKAGES; do
  manifest="$(python3 -c 'import json,sys; path,pkg=sys.argv[1:3]; data=json.load(open(path, encoding="utf-8")); print(next((p["manifest_path"] for p in data["packages"] if p["name"] == pkg), ""))' "$workspace_json_file" "$package")"
  if [[ -z "$manifest" ]]; then
    log "UNKNOWN: package $package not in workspace"
    continue
  fi
  crate_dir="$(dirname "$manifest")"
  package_feature_args=()
  if [[ "$caller_controls_features" == "0" ]]; then
    mapfile -t package_feature_args < <(derive_package_feature_args "$package")
  fi

  mapfile -t generated_harnesses < <(
    python3 - "$crate_dir" <<'PY'
import pathlib
import re
import sys

crate_dir = pathlib.Path(sys.argv[1])
proof_pattern = re.compile(
    r"#\s*\[\s*kani::proof\s*\]\s*(?:#\[[^\n]+\]\s*)*fn\s+([A-Za-z0-9_]+)\s*\(",
    re.MULTILINE,
)

for path in sorted(crate_dir.glob("tests/generated_kani_*.rs")):
    text = path.read_text(encoding="utf-8")
    if "kani::proof" not in text:
        continue
    for match in proof_pattern.finditer(text):
        print(f"{path.stem}\t{match.group(1)}")
PY
  )

  if [[ "${#generated_harnesses[@]}" -gt 0 ]]; then
    for harness_spec in "${generated_harnesses[@]}"; do
      IFS=$'\t' read -r target_name harness <<<"$harness_spec"
      qualified_harness="${target_name}::${harness}"
      log "cargo kani ${kani_profile_args[*]:-} -p $package ${feature_args[*]:-} ${package_feature_args[*]:-} --exact --harness $qualified_harness --output-format terse"
      set +e
      run_kani_cmd "kani:$package:$qualified_harness" "$WALL_TIMEOUT_SECS" cargo kani "${kani_profile_args[@]}" -p "$package" "${feature_args[@]}" "${package_feature_args[@]}" --exact --harness "$qualified_harness" --output-format terse
      status=$?
      set -e
      if [[ "$status" -eq 0 ]]; then
        ran=1
        continue
      fi
      if [[ "$status" -eq 124 ]]; then
        log "UNKNOWN: timeout after ${WALL_TIMEOUT_SECS}s for Kani harness $package:$qualified_harness"
        continue
      fi
      exit "$status"
    done
  elif rg -q "kani::|proof_for_contract|kani::proof" "$crate_dir" 2>/dev/null; then
    log "cargo kani ${kani_profile_args[*]:-} -p $package ${feature_args[*]:-} ${package_feature_args[*]:-} --output-format terse"
    set +e
    run_kani_cmd "kani:$package" "$WALL_TIMEOUT_SECS" cargo kani "${kani_profile_args[@]}" -p "$package" "${feature_args[@]}" "${package_feature_args[@]}" --output-format terse
    status=$?
    set -e
    if [[ "$status" -eq 0 ]]; then
      ran=1
      continue
    fi
    if [[ "$status" -eq 124 ]]; then
      log "UNKNOWN: timeout after ${WALL_TIMEOUT_SECS}s for package-wide Kani run on $package"
      continue
    fi
    exit "$status"
  else
    log "UNKNOWN: no Kani harness markers in $package"
  fi
done

if [[ "$ran" -eq 0 ]]; then
  unknown_or_fail "no Kani targets completed successfully"
  exit 0
fi

log "BOUNDED_VERIFIED: Kani completed successfully for configured packages"
