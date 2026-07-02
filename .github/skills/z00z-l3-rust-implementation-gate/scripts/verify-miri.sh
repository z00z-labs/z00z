#!/bin/bash

# Run Miri for configured packages.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
MANIFEST_PATH="$ROOT_DIR/Cargo.toml"
PACKAGES="${Z00Z_MIRI_PACKAGES:-z00z_utils}"
STRICT="${Z00Z_L3_STRICT:-0}"
PROFILE_ARGS_TEXT="${Z00Z_CARGO_PROFILE_ARGS:---release}"
FEATURE_FLAG="${Z00Z_ALL_FEATURES_FLAG-}"
TOOLS_TMPDIR="${Z00Z_MIRI_TMPDIR:-}"
TOOLS_CWD="${Z00Z_MIRI_CWD:-}"
MIRI_FLAGS_TEXT="${Z00Z_MIRIFLAGS:--Zmiri-disable-isolation}"
MIRI_SYSROOT_PATH="${Z00Z_MIRI_SYSROOT:-$ROOT_DIR/tools/formal_verification/miri/sysroot}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

log() {
  printf '[z00z-l3:miri] %s\n' "$1"
}

nightly_has_miri() {
  rustup +nightly component list --installed 2>/dev/null | grep -Eq '^miri($|-)'
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

if ! nightly_has_miri; then
  unknown_or_fail "Miri is not installed"
  exit 0
fi

if [[ ! -d "$MIRI_SYSROOT_PATH/lib/rustlib" ]]; then
  unknown_or_fail "prebuilt Miri sysroot is missing at ${MIRI_SYSROOT_PATH#"$ROOT_DIR"/}; run scripts/install-verification-tools.sh --install --profile all"
  exit 0
fi

profile_args=()
if [[ -n "$PROFILE_ARGS_TEXT" ]]; then
  read -r -a profile_args <<<"$PROFILE_ARGS_TEXT"
fi

feature_args=()
if [[ -n "$FEATURE_FLAG" ]]; then
  feature_args+=("$FEATURE_FLAG")
fi

cleanup_tmpdir() {
  if [[ -n "${TOOLS_TMPDIR:-}" && -d "${TOOLS_TMPDIR:-}" ]]; then
    rmdir "$TOOLS_TMPDIR" 2>/dev/null || true
  fi
  if [[ -n "${TOOLS_CWD_CLEANUP:-}" && "${TOOLS_CWD:-}" == "${TOOLS_CWD_CLEANUP:-}" && -d "${TOOLS_CWD:-}" ]]; then
    rmdir "$TOOLS_CWD" 2>/dev/null || true
  fi
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

if [[ -z "$TOOLS_TMPDIR" ]]; then
  if [[ -n "${Z00Z_VERIFICATION_TMPDIR:-}" ]]; then
    TOOLS_TMPDIR="$Z00Z_VERIFICATION_TMPDIR/miri-tmp"
    mkdir -p "$TOOLS_TMPDIR"
  else
    TOOLS_TMPDIR="${TMPDIR:-}"
  fi
fi

if [[ -z "$TOOLS_CWD" ]]; then
  if [[ -n "${Z00Z_SYSTEM_TMPDIR:-}" ]]; then
    TOOLS_CWD="$Z00Z_SYSTEM_TMPDIR/miri-cwd"
  else
    TOOLS_CWD="${TMPDIR:-$ROOT_DIR}/z00z-miri-cwd"
    TOOLS_CWD_CLEANUP="$TOOLS_CWD"
    trap cleanup_tmpdir EXIT
  fi
fi
mkdir -p "$TOOLS_CWD"

is_miri_tool_limit() {
  local log_path="$1"
  rg -q \
    "unsupported operation occurred here|this is likely not a bug in the program; it indicates that the program performed an operation that Miri does not support" \
    "$log_path"
}

miri_env=()
mapfile -d '' -t miri_env < <(build_runtime_env)
if [[ -n "$TOOLS_TMPDIR" ]]; then
  miri_env+=(TMPDIR="$TOOLS_TMPDIR")
  miri_env+=(TMP="$TOOLS_TMPDIR")
  miri_env+=(TEMP="$TOOLS_TMPDIR")
fi
if [[ -n "$MIRI_FLAGS_TEXT" ]]; then
  miri_env+=(MIRIFLAGS="$MIRI_FLAGS_TEXT")
fi
miri_env+=(MIRI_SYSROOT="$MIRI_SYSROOT_PATH")

workspace_packages="$(cargo metadata --manifest-path "$MANIFEST_PATH" --format-version 1 --no-deps | python3 -c 'import json,sys; print("\n".join(p["name"] for p in json.load(sys.stdin)["packages"]))')"
ran=0

package_target_specs() {
  local package="$1"
  case "$package" in
    z00z_utils)
      printf 'lib-os-hardening\t--lib\t\tos_hardening::\n'
      printf 'itest-os-hardening\t--test\ttest_os_hardening_integration\t\n'
      ;;
    *)
      printf 'lib\t--lib\t\t\n'
      ;;
  esac
}

for package in $PACKAGES; do
  if printf '%s\n' "$workspace_packages" | grep -Fxq "$package"; then
    mapfile -t target_specs < <(package_target_specs "$package")
    package_passed=0
    for spec in "${target_specs[@]}"; do
      IFS=$'\t' read -r target_label target_flag target_name target_filter <<<"$spec"
      cmd=(cargo +nightly miri test --manifest-path "$MANIFEST_PATH" "${profile_args[@]}" -p "$package" "$target_flag")
      if [[ -n "$target_name" ]]; then
        cmd+=("$target_name")
      fi
      if [[ "${#feature_args[@]}" -gt 0 ]]; then
        cmd+=("${feature_args[@]}")
      fi
      if [[ -n "$target_filter" ]]; then
        cmd+=("$target_filter")
      fi

      log "$(printf '%s' "$(z00z_profile_join_command "${cmd[@]}")") (MIRIFLAGS=${MIRI_FLAGS_TEXT:-unset}, MIRI_SYSROOT=${MIRI_SYSROOT_PATH#"$ROOT_DIR"/})"
      miri_log="$(mktemp "${TOOLS_TMPDIR:-${TMPDIR:-/tmp}}/z00z-miri-${package}-${target_label}.XXXXXX")"
      set +e
      z00z_profile_run_command command "miri:$package:$target_label" \
        "${miri_env[@]}" \
        bash -c 'cd "$1" && shift && "$@"' _ "$TOOLS_CWD" "${cmd[@]}" >"$miri_log" 2>&1
      status=$?
      set -e
      cat "$miri_log"
      if [[ "$status" -eq 0 ]]; then
        rm -f "$miri_log"
        ran=1
        package_passed=1
        continue
      fi
      if is_miri_tool_limit "$miri_log"; then
        rm -f "$miri_log"
        unknown_or_fail "Miri hit an unsupported operation while analyzing $package target $target_label"
        continue
      fi
      rm -f "$miri_log"
      exit "$status"
    done
    if [[ "$package_passed" -eq 0 ]]; then
      log "UNKNOWN: no configured Miri targets completed successfully for $package"
    fi
  else
    log "UNKNOWN: package $package not in workspace"
  fi
done

if [[ "$ran" -eq 0 ]]; then
  unknown_or_fail "no Miri targets completed successfully"
  exit 0
fi

log "TESTED: Miri completed successfully for configured packages"
