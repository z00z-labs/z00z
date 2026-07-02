#!/bin/bash

# Shared profiling helpers for Z00Z verification scripts.

z00z_require_canonical_run_root() {
  local root_dir="$1"
  local run_root="$2"
  local reports_dir="$root_dir/reports"
  local parent_dir base_name

  [[ -n "$run_root" ]] || return 0

  parent_dir="$(dirname "$run_root")"
  base_name="$(basename "$run_root")"

  if [[ "$parent_dir" != "$reports_dir" || ! "$base_name" =~ ^z00z-verification-orchestrator-[0-9]{8}-[0-9]{6}$ ]]; then
    echo "ERROR: verifier outputs must use reports/z00z-verification-orchestrator-<YYYYMMDD-HHMMSS>; got $run_root" >&2
    return 1
  fi
}

z00z_profile_prepare_tmp_workspace_sentinel() {
  local tmp_root="$1"
  local workspace_file

  [[ -n "$tmp_root" ]] || return 0
  mkdir -p "$tmp_root"
  workspace_file="$tmp_root/Cargo.toml"
  if [[ ! -f "$workspace_file" ]]; then
    cat >"$workspace_file" <<'EOF'
[workspace]
members = []
exclude = [".tmp*"]
EOF
  fi
}

z00z_profile_run_root() {
  local run_root="${Z00Z_VERIFICATION_RUN_ROOT:-${RUN_ROOT:-}}"
  local verification_root="${Z00Z_VERIFICATION_RUNTIME_ROOT:-${VERIFICATION_RUNTIME_ROOT:-}}"

  if [[ -z "$run_root" && -n "$verification_root" ]]; then
    run_root="$(dirname "$verification_root")"
  fi

  printf '%s\n' "$run_root"
}

z00z_profile_safe_slug() {
  printf '%s\n' "$1" | tr '/ :=' '____' | tr -cd '[:alnum:]_.-'
}

z00z_profile_res_dir() {
  local run_root

  run_root="$(z00z_profile_run_root)"
  [[ -n "$run_root" ]] || return 1
  printf '%s\n' "$run_root/profiling/resources"
}

z00z_profile_meta_dir() {
  local run_root

  run_root="$(z00z_profile_run_root)"
  [[ -n "$run_root" ]] || return 1
  printf '%s\n' "$run_root/profiling/resource-meta"
}

z00z_profile_cache_tsv() {
  local run_root

  run_root="$(z00z_profile_run_root)"
  [[ -n "$run_root" ]] || return 1
  printf '%s\n' "$run_root/profiling/cache-maintenance.tsv"
}

z00z_profile_owner_file() {
  local run_root="${1:-}"

  if [[ -z "$run_root" ]]; then
    run_root="$(z00z_profile_run_root)"
  fi
  [[ -n "$run_root" ]] || return 1
  printf '%s\n' "$run_root/profiling/run-owner.tsv"
}

z00z_profile_proc_start_ticks() {
  local pid="${1:-}"
  local stat_file="/proc/$pid/stat"

  [[ -n "$pid" ]] || return 0
  [[ -r "$stat_file" ]] || return 0
  awk '{print $22}' "$stat_file" 2>/dev/null | head -n 1
}

z00z_profile_run_tag() {
  local run_root="$1"
  local pid="$2"

  if command -v sha256sum >/dev/null 2>&1; then
    printf '%s' "$run_root:$pid" | sha256sum | cut -c1-16
    return 0
  fi

  printf '%s\n' "$(z00z_profile_safe_slug "${run_root##*/}-$pid")"
}

z00z_profile_mark_run_root() {
  local run_root="$1"
  local owner_file run_tag start_ticks

  [[ -n "$run_root" ]] || return 0
  owner_file="$(z00z_profile_owner_file "$run_root")" || return 0
  mkdir -p "$(dirname "$owner_file")"
  run_tag="$(z00z_profile_run_tag "$run_root" "${BASHPID:-$$}")"
  start_ticks="$(z00z_profile_proc_start_ticks "${BASHPID:-$$}")"

  {
    printf 'pid\t%s\n' "${BASHPID:-$$}"
    printf 'proc_start_ticks\t%s\n' "${start_ticks:-}"
    printf 'run_root\t%s\n' "$run_root"
    printf 'run_tag\t%s\n' "$run_tag"
    printf 'started_at\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  } >"$owner_file"
}

z00z_profile_owner_pid() {
  local owner_file

  owner_file="$(z00z_profile_owner_file "$1")" || return 0
  [[ -f "$owner_file" ]] || return 0
  awk -F '\t' '$1 == "pid" { print $2; exit }' "$owner_file"
}

z00z_profile_owner_start_ticks() {
  local owner_file

  owner_file="$(z00z_profile_owner_file "$1")" || return 0
  [[ -f "$owner_file" ]] || return 0
  awk -F '\t' '$1 == "proc_start_ticks" { print $2; exit }' "$owner_file"
}

z00z_profile_owner_run_root() {
  local owner_file

  owner_file="$(z00z_profile_owner_file "$1")" || return 0
  [[ -f "$owner_file" ]] || return 0
  awk -F '\t' '$1 == "run_root" { print $2; exit }' "$owner_file"
}

z00z_profile_live_root_owner() {
  local run_root="$1"
  local owner_pid owner_start_ticks current_start_ticks owner_run_root

  [[ -d "$run_root" ]] || return 1

  owner_pid="$(z00z_profile_owner_pid "$run_root")"
  [[ -n "$owner_pid" ]] || return 1
  kill -0 "$owner_pid" 2>/dev/null || return 1

  owner_start_ticks="$(z00z_profile_owner_start_ticks "$run_root")"
  current_start_ticks="$(z00z_profile_proc_start_ticks "$owner_pid")"
  [[ -n "$owner_start_ticks" && -n "$current_start_ticks" ]] || return 1
  [[ "$owner_start_ticks" == "$current_start_ticks" ]] || return 1

  owner_run_root="$(z00z_profile_owner_run_root "$run_root")"
  [[ -z "$owner_run_root" || "$owner_run_root" == "$run_root" ]] || return 1

  return 0
}

z00z_profile_bytes() {
  local path="$1"
  local size

  [[ -e "$path" ]] || {
    printf '0\n'
    return 0
  }

  size="$(du -sb "$path" 2>/dev/null | awk '{print $1}' | head -n 1)"
  printf '%s\n' "${size:-0}"
}

z00z_profile_trim_root_data() {
  local run_root="$1"
  local path name bytes trimmed_count=0 reclaimed=0
  local -a exact_paths=()
  local -a trimmed=()

  [[ -d "$run_root" ]] || return 1

  exact_paths=(
    "$run_root/target"
    "$run_root/.cache"
    "$run_root/cache"
    "$run_root/geiger"
    "$run_root/workdir"
    "$run_root/tmp"
    "$run_root/specs"
    "$run_root/verification"
    "$run_root/fuzz"
  )

  for path in "${exact_paths[@]}"; do
    [[ -e "$path" ]] || continue
    name="${path#"$run_root"/}"
    bytes="$(z00z_profile_bytes "$path")"
    rm -rf -- "$path"
    reclaimed=$((reclaimed + bytes))
    trimmed_count=$((trimmed_count + 1))
    trimmed+=("$name")
  done

  shopt -s nullglob
  for path in \
    "$run_root"/tmp[0-9]* \
    "$run_root"/specs[0-9]* \
    "$run_root"/verification[0-9]* \
    "$run_root"/fuzz[0-9]*; do
    [[ -e "$path" ]] || continue
    name="${path#"$run_root"/}"
    bytes="$(z00z_profile_bytes "$path")"
    rm -rf -- "$path"
    reclaimed=$((reclaimed + bytes))
    trimmed_count=$((trimmed_count + 1))
    trimmed+=("$name")
  done
  shopt -u nullglob

  [[ "$trimmed_count" -gt 0 ]] || return 1

  local IFS=';'
  printf '%s\t%s\t%s\n' "$trimmed_count" "$reclaimed" "${trimmed[*]}"
}

z00z_profile_trash_path() {
  local path="$1"

  [[ -e "$path" ]] || return 1

  if command -v gio >/dev/null 2>&1; then
    gio trash "$path"
    return 0
  fi

  if command -v trash-put >/dev/null 2>&1; then
    trash-put "$path"
    return 0
  fi

  echo "ERROR: neither gio nor trash-put is available for safe verifier cleanup" >&2
  return 1
}

z00z_profile_trash_run_root() {
  local run_root="$1"
  local reclaimed base_name

  [[ -d "$run_root" ]] || return 1

  reclaimed="$(z00z_profile_bytes "$run_root")"
  base_name="$(basename "$run_root")"
  z00z_profile_trash_path "$run_root"

  printf '1\t%s\t%s\n' "${reclaimed:-0}" "$base_name"
}

z00z_profile_init_cache_tsv() {
  local path

  path="$(z00z_profile_cache_tsv)" || return 0
  mkdir -p "$(dirname "$path")"
  if [[ ! -f "$path" || ! -s "$path" ]]; then
    printf 'label\telapsed_ms\tscanned_roots\ttrimmed_roots\ttrimmed_paths\treclaimed_bytes\n' >"$path"
  fi
}

z00z_profile_append_cache_tsv() {
  local label="$1"
  local elapsed_ms="$2"
  local scanned_roots="$3"
  local trimmed_roots="$4"
  local trimmed_paths="$5"
  local reclaimed_bytes="$6"
  local path

  path="$(z00z_profile_cache_tsv)" || return 0
  z00z_profile_init_cache_tsv
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$(z00z_profile_escape_field "$label")" \
    "$elapsed_ms" \
    "$scanned_roots" \
    "$trimmed_roots" \
    "$trimmed_paths" \
    "$reclaimed_bytes" \
    >>"$path"
}

z00z_profile_prune_stale_roots() {
  local label="$1"
  local run_root reports_dir path info
  local start_ns end_ns elapsed_ms
  local scanned_roots=0 trimmed_roots=0 trimmed_paths=0 reclaimed_bytes=0
  local trimmed_count trimmed_bytes

  run_root="$(z00z_profile_run_root)"
  [[ -n "$run_root" ]] || return 0
  [[ "${Z00Z_KEEP_PREVIOUS_RUNS:-0}" != "1" ]] || return 0
  reports_dir="$(dirname "$run_root")"
  [[ -d "$reports_dir" ]] || return 0

  start_ns="$(date -u +%s%N)"
  shopt -s nullglob
  for path in "$reports_dir"/z00z-verification-orchestrator-*; do
    [[ -d "$path" ]] || continue
    [[ "$path" != "$run_root" ]] || continue
    [[ "$(basename "$path")" =~ ^z00z-verification-orchestrator-[0-9]{8}-[0-9]{6}$ ]] || continue
    scanned_roots=$((scanned_roots + 1))
    if z00z_profile_live_root_owner "$path"; then
      continue
    fi
    if info="$(z00z_profile_trash_run_root "$path" 2>/dev/null)"; then
      IFS=$'\t' read -r trimmed_count trimmed_bytes _ <<<"$info"
      trimmed_roots=$((trimmed_roots + 1))
      trimmed_paths=$((trimmed_paths + trimmed_count))
      reclaimed_bytes=$((reclaimed_bytes + trimmed_bytes))
    fi
  done
  shopt -u nullglob
  end_ns="$(date -u +%s%N)"
  elapsed_ms=$(((end_ns - start_ns) / 1000000))
  z00z_profile_append_cache_tsv \
    "$label" \
    "$elapsed_ms" \
    "$scanned_roots" \
    "$trimmed_roots" \
    "$trimmed_paths" \
    "$reclaimed_bytes"
  printf '%s\t%s\t%s\t%s\t%s\n' \
    "$elapsed_ms" \
    "$scanned_roots" \
    "$trimmed_roots" \
    "$trimmed_paths" \
    "$reclaimed_bytes"
}

z00z_profile_cmd_bin() {
  if [[ "${1:-}" == "env" ]]; then
    shift
    while [[ $# -gt 0 ]]; do
      [[ "$1" == *=* ]] || break
      shift
    done
  fi
  printf '%s\n' "${1:-}"
}

z00z_profile_cargo_sub() {
  if [[ "${1:-}" == "env" ]]; then
    shift
    while [[ $# -gt 0 ]]; do
      [[ "$1" == *=* ]] || break
      shift
    done
  fi
  [[ "${1:-}" == "cargo" ]] || return 0
  shift
  while [[ $# -gt 0 ]]; do
    if [[ "$1" == +* ]]; then
      shift
      continue
    fi
    printf '%s\n' "${1:-}"
    return 0
  done
}

z00z_profile_is_cargo_cmd() {
  [[ "$(z00z_profile_cmd_bin "$@")" == "cargo" ]]
}

z00z_profile_cmd_env() {
  local wanted="$1"
  shift

  if [[ "${1:-}" == "env" ]]; then
    shift
    while [[ $# -gt 0 ]]; do
      case "$1" in
        "$wanted"=*)
          printf '%s\n' "${1#*=}"
          return 0
          ;;
        *=*)
          shift
          ;;
        *)
          break
          ;;
      esac
    done
  fi

  if [[ -n "${!wanted+x}" ]]; then
    printf '%s\n' "${!wanted}"
  fi
}

z00z_profile_cmd_arg() {
  local wanted="$1"
  shift

  if [[ "${1:-}" == "env" ]]; then
    shift
    while [[ $# -gt 0 ]]; do
      [[ "$1" == *=* ]] || break
      shift
    done
  fi

  while [[ $# -gt 0 ]]; do
    if [[ "$1" == "$wanted" ]]; then
      shift
      printf '%s\n' "${1:-}"
      return 0
    fi
    shift
  done
}

z00z_profile_join_unique() {
  local item joined=""

  for item in "$@"; do
    [[ -n "$item" ]] || continue
    case ";$joined;" in
      *";$item;"*) ;;
      *)
        joined="${joined:+$joined;}$item"
        ;;
    esac
  done

  printf '%s\n' "$joined"
}

z00z_profile_cmd_mode() {
  local bin cargo_sub

  bin="$(z00z_profile_cmd_bin "$@")"
  cargo_sub="$(z00z_profile_cargo_sub "$@")"

  case "$bin:$cargo_sub" in
    cargo:fmt)
      printf 'format-only\n'
      ;;
    cargo:clippy|cargo:geiger)
      printf 'compile+analyze\n'
      ;;
    cargo:test|cargo:nextest|cargo:bench|cargo:fuzz)
      printf 'compile+execute\n'
      ;;
    cargo:audit|cargo:deny|cargo:vet|cargo:tree|cargo:metadata|cargo:semver-checks)
      printf 'analyze-only\n'
      ;;
    python3:*|python:*)
      printf 'analyze-only\n'
      ;;
    bash:*)
      printf 'execute-only\n'
      ;;
    *)
      printf 'unknown\n'
      ;;
  esac
}

z00z_profile_cmd_targets() {
  local cargo_target fuzz_target geiger_root cli_target

  cargo_target="$(z00z_profile_cmd_env CARGO_TARGET_DIR "$@")"
  fuzz_target="$(z00z_profile_cmd_env Z00Z_FUZZ_TARGET_DIR "$@")"
  geiger_root="$(z00z_profile_cmd_env Z00Z_GEIGER_TARGET_ROOT "$@")"
  cli_target="$(z00z_profile_cmd_arg --target-dir "$@")"

  z00z_profile_join_unique "$cargo_target" "$fuzz_target" "$geiger_root" "$cli_target"
}

z00z_profile_cmd_caches() {
  local run_cache sim_cache cargo_home xdg_cache pip_cache npm_cache mypy_cache ruff_cache uv_cache

  run_cache="$(z00z_profile_cmd_env Z00Z_RUN_CACHE_ROOT "$@")"
  sim_cache="$(z00z_profile_cmd_env Z00Z_SIMULATOR_CACHE_ROOT "$@")"
  cargo_home="$(z00z_profile_cmd_env CARGO_HOME "$@")"
  xdg_cache="$(z00z_profile_cmd_env XDG_CACHE_HOME "$@")"
  pip_cache="$(z00z_profile_cmd_env PIP_CACHE_DIR "$@")"
  npm_cache="$(z00z_profile_cmd_env NPM_CONFIG_CACHE "$@")"
  mypy_cache="$(z00z_profile_cmd_env MYPY_CACHE_DIR "$@")"
  ruff_cache="$(z00z_profile_cmd_env RUFF_CACHE_DIR "$@")"
  uv_cache="$(z00z_profile_cmd_env UV_CACHE_DIR "$@")"

  z00z_profile_join_unique \
    "$run_cache" \
    "$sim_cache" \
    "$cargo_home" \
    "$xdg_cache" \
    "$pip_cache" \
    "$npm_cache" \
    "$mypy_cache" \
    "$ruff_cache" \
    "$uv_cache"
}

z00z_profile_write_meta() {
  local resource_path="$1"
  local kind="$2"
  local label="$3"
  local command_text="$4"
  local cleanup_ms="$5"
  local cleanup_scanned="$6"
  local cleanup_roots="$7"
  local cleanup_paths="$8"
  local cleanup_bytes="$9"
  shift 9
  local meta_dir meta_path stem

  meta_dir="$(z00z_profile_meta_dir)" || return 0
  mkdir -p "$meta_dir"
  stem="$(basename "${resource_path%.time}")"
  meta_path="$meta_dir/$stem.tsv"

  {
    printf 'kind\t%s\n' "$kind"
    printf 'label\t%s\n' "$(z00z_profile_escape_field "$label")"
    printf 'execution_mode\t%s\n' "$(z00z_profile_cmd_mode "$@")"
    printf 'target_roots\t%s\n' "$(z00z_profile_escape_field "$(z00z_profile_cmd_targets "$@")")"
    printf 'cache_roots\t%s\n' "$(z00z_profile_escape_field "$(z00z_profile_cmd_caches "$@")")"
    printf 'cleanup_elapsed_ms\t%s\n' "$cleanup_ms"
    printf 'cleanup_scanned_roots\t%s\n' "$cleanup_scanned"
    printf 'cleanup_trimmed_roots\t%s\n' "$cleanup_roots"
    printf 'cleanup_trimmed_paths\t%s\n' "$cleanup_paths"
    printf 'cleanup_reclaimed_bytes\t%s\n' "$cleanup_bytes"
    printf 'command\t%s\n' "$(z00z_profile_escape_field "$command_text")"
  } >"$meta_path"
}

z00z_profile_activate_tool_env() {
  local root_dir="$1"
  local tools_dir="${Z00Z_VERIFY_TOOLS_DIR:-$root_dir/tools/formal_verification}"
  local run_root="${Z00Z_VERIFICATION_RUN_ROOT:-${RUN_ROOT:-}}"
  local verification_root="${Z00Z_VERIFICATION_RUNTIME_ROOT:-${VERIFICATION_RUNTIME_ROOT:-}}"
  local verify_env_script="$root_dir/scripts/verify-env.sh"
  local saved_script_dir="${SCRIPT_DIR:-}"
  local saved_root_dir="${ROOT_DIR:-}"
  local cache_root xdg_cache_dir xdg_state_dir pycache_dir pip_cache_dir npm_cache_dir mypy_cache_dir
  local path_dirs=(
    "$tools_dir/bin"
    "$tools_dir/saw-suite/bin"
    "$tools_dir/cargo/bin"
    "$tools_dir/python/bin"
    "$tools_dir/node/bin"
    "$tools_dir/opam/bin"
    "$tools_dir/prusti/bin"
    "$tools_dir/verus/bin"
    "$tools_dir/tamarin/bin"
    "$tools_dir/maude/bin"
    "$tools_dir/apalache/bin"
    "$tools_dir/alloy/bin"
    "$tools_dir/saw/bin"
    "$tools_dir/cryptol/bin"
    "$tools_dir/cvc5/bin"
    "$tools_dir/bitwuzla/bin"
    "$tools_dir/mir-json/bin"
    "$tools_dir/charon/bin"
    "$tools_dir/aeneas/bin"
    "$tools_dir/rg/bin"
  )
  local idx dir

  if [[ -f "$verify_env_script" ]]; then
    # shellcheck source=/dev/null
    source "$verify_env_script"
    if [[ -n "$saved_script_dir" ]]; then
      SCRIPT_DIR="$saved_script_dir"
    fi
    if [[ -n "$saved_root_dir" ]]; then
      ROOT_DIR="$saved_root_dir"
    fi
  fi

  for ((idx=${#path_dirs[@]} - 1; idx >= 0; idx--)); do
    dir="${path_dirs[$idx]}"
    if [[ -d "$dir" ]]; then
      PATH="$dir:$PATH"
    fi
  done

  export PATH
  export KANI_HOME="${Z00Z_KANI_HOME:-$tools_dir/kani}"
  export NPM_CONFIG_PREFIX="${Z00Z_VERIFY_NODE_PREFIX:-$tools_dir/node}"
  export Z00Z_RELEASE_BLOCK_UNKNOWN="${Z00Z_RELEASE_BLOCK_UNKNOWN:-1}"
  export Z00Z_L0_STRICT="${Z00Z_L0_STRICT:-$Z00Z_RELEASE_BLOCK_UNKNOWN}"
  export Z00Z_L1_STRICT="${Z00Z_L1_STRICT:-$Z00Z_RELEASE_BLOCK_UNKNOWN}"
  export Z00Z_L2_STRICT="${Z00Z_L2_STRICT:-$Z00Z_RELEASE_BLOCK_UNKNOWN}"
  export Z00Z_L3_STRICT="${Z00Z_L3_STRICT:-$Z00Z_RELEASE_BLOCK_UNKNOWN}"
  export Z00Z_L4_STRICT="${Z00Z_L4_STRICT:-$Z00Z_RELEASE_BLOCK_UNKNOWN}"
  export Z00Z_OPAM_ROOT="${Z00Z_OPAM_ROOT:-$tools_dir/opam/root}"
  export Z00Z_VERIFY_OPAM_SWITCH="${Z00Z_VERIFY_OPAM_SWITCH:-z00z-verify}"
  export CREUSOT_DATA_HOME="${Z00Z_CREUSOT_DATA_HOME:-$tools_dir/creusot/data}"
  export XDG_CONFIG_HOME="${Z00Z_CREUSOT_CONFIG_HOME:-$tools_dir/creusot/config}"
  export XDG_CACHE_HOME="${Z00Z_CREUSOT_CACHE_HOME:-$tools_dir/creusot/cache}"
  export MIRI_SYSROOT="${Z00Z_MIRI_SYSROOT:-$tools_dir/miri/sysroot}"
  export CHARON_MIRI_SYSROOTS="${CHARON_MIRI_SYSROOTS:-$MIRI_SYSROOT}"

  if [[ -z "$run_root" && -n "$verification_root" ]]; then
    run_root="$(dirname "$verification_root")"
  fi
  if [[ -n "$run_root" ]]; then
    z00z_require_canonical_run_root "$root_dir" "$run_root" || return 1
    cache_root="${Z00Z_RUN_CACHE_ROOT:-$run_root/.cache}"
    xdg_cache_dir="$cache_root/xdg"
    xdg_state_dir="$cache_root/xdg-state"
    pycache_dir="$cache_root/python/pycache"
    pip_cache_dir="$cache_root/pip"
    npm_cache_dir="$cache_root/npm"
    mypy_cache_dir="$cache_root/mypy"
    export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$run_root/target}"
    export Z00Z_RUN_CACHE_ROOT="$cache_root"
    export RUFF_CACHE_DIR="$cache_root/ruff"
    export UV_CACHE_DIR="$cache_root/uv"
    export TMP="${TMP:-${Z00Z_SYSTEM_TMPDIR:-${TMPDIR:-$run_root/tmp}}}"
    export TEMP="${TEMP:-$TMP}"
    export TMPDIR="${TMPDIR:-$TMP}"
    export XDG_CACHE_HOME="$xdg_cache_dir"
    export XDG_STATE_HOME="$xdg_state_dir"
    export PYTHONPYCACHEPREFIX="$pycache_dir"
    export PIP_CACHE_DIR="$pip_cache_dir"
    export NPM_CONFIG_CACHE="$npm_cache_dir"
    export MYPY_CACHE_DIR="$mypy_cache_dir"
    mkdir -p "$cache_root" "$CARGO_TARGET_DIR" "$RUFF_CACHE_DIR" "$UV_CACHE_DIR"
    mkdir -p "$XDG_CACHE_HOME" "$XDG_STATE_HOME" "$PYTHONPYCACHEPREFIX"
    mkdir -p "$PIP_CACHE_DIR" "$NPM_CONFIG_CACHE" "$MYPY_CACHE_DIR"
    if ! z00z_profile_live_root_owner "$run_root"; then
      z00z_profile_mark_run_root "$run_root"
    fi
  fi

  if [[ -n "${Z00Z_SYSTEM_TMPDIR:-}" ]]; then
    z00z_profile_prepare_tmp_workspace_sentinel "$Z00Z_SYSTEM_TMPDIR"
  elif [[ -n "${TMPDIR:-}" ]]; then
    z00z_profile_prepare_tmp_workspace_sentinel "$TMPDIR"
  fi

  if [[ -d "$tools_dir/saw-suite/rlibs" ]]; then
    export CRUX_RUST_LIBRARY_PATH="${CRUX_RUST_LIBRARY_PATH:-$tools_dir/saw-suite/rlibs}"
    export SAW_RUST_LIBRARY_PATH="${SAW_RUST_LIBRARY_PATH:-$tools_dir/saw-suite/rlibs}"
  elif [[ -d "$tools_dir/mir-json/rlibs" ]]; then
    export CRUX_RUST_LIBRARY_PATH="${CRUX_RUST_LIBRARY_PATH:-$tools_dir/mir-json/rlibs}"
    export SAW_RUST_LIBRARY_PATH="${SAW_RUST_LIBRARY_PATH:-$tools_dir/mir-json/rlibs}"
  fi

  if [[ -x "$tools_dir/saw-suite/bin/crux-mir" ]]; then
    export CRUX_MIR="${CRUX_MIR:-$tools_dir/saw-suite/bin/crux-mir}"
  elif [[ -x "$tools_dir/saw-suite/bin/crux-mir-comp" ]]; then
    export CRUX_MIR="${CRUX_MIR:-$tools_dir/saw-suite/bin/crux-mir-comp}"
  fi
}

z00z_profile_events_file() {
  printf '%s\n' "${Z00Z_PROFILE_EVENTS_FILE:-}"
}

z00z_profile_enabled() {
  [[ -n "${Z00Z_PROFILE_EVENTS_FILE:-}" ]]
}

z00z_profile_now_ns() {
  date -u +%s%N
}

z00z_profile_now_iso() {
  date -u +%Y-%m-%dT%H:%M:%SZ
}

z00z_profile_escape_field() {
  local value="$1"
  value="${value//$'\t'/ }"
  value="${value//$'\n'/ }"
  value="${value//$'\r'/ }"
  printf '%s' "$value"
}

z00z_profile_join_command() {
  local rendered=""
  local arg
  for arg in "$@"; do
    if [[ -n "$rendered" ]]; then
      rendered+=" "
    fi
    rendered+="$(printf '%q' "$arg")"
  done
  printf '%s' "$rendered"
}

z00z_profile_init_file() {
  local file
  file="$(z00z_profile_events_file)"
  [[ -n "$file" ]] || return 0
  mkdir -p "$(dirname "$file")"
  if [[ ! -f "$file" || ! -s "$file" ]]; then
    printf 'kind\tlabel\tstatus\telapsed_ms\telapsed_secs\tstarted_at\tended_at\tcommand\n' >"$file"
  fi
}

z00z_profile_record_event() {
  local kind="$1"
  local label="$2"
  local status="$3"
  local start_ns="$4"
  local end_ns="$5"
  local started_at="$6"
  local ended_at="$7"
  local command_text="$8"
  local file elapsed_ns elapsed_ms elapsed_secs

  file="$(z00z_profile_events_file)"
  [[ -n "$file" ]] || return 0

  z00z_profile_init_file
  elapsed_ns=$((end_ns - start_ns))
  if (( elapsed_ns < 0 )); then
    elapsed_ns=0
  fi
  elapsed_ms=$((elapsed_ns / 1000000))
  elapsed_secs="$(printf '%d.%03d' "$((elapsed_ms / 1000))" "$((elapsed_ms % 1000))")"

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$(z00z_profile_escape_field "$kind")" \
    "$(z00z_profile_escape_field "$label")" \
    "$(z00z_profile_escape_field "$status")" \
    "$elapsed_ms" \
    "$elapsed_secs" \
    "$(z00z_profile_escape_field "$started_at")" \
    "$(z00z_profile_escape_field "$ended_at")" \
    "$(z00z_profile_escape_field "$command_text")" \
    >>"$file"
}

z00z_profile_run_command() {
  local kind="$1"
  local label="$2"
  shift 2

  local start_ns end_ns started_at ended_at status had_errexit=0
  local command_text resource_time_bin resource_path safe_id run_root
  local cleanup_info cleanup_ms=0 cleanup_scanned=0 cleanup_roots=0 cleanup_paths=0 cleanup_bytes=0
  case $- in
    *e*) had_errexit=1 ;;
  esac

  command_text="$(z00z_profile_join_command "$@")"
  if z00z_profile_is_cargo_cmd "$@"; then
    cleanup_info="$(z00z_profile_prune_stale_roots "$label" || true)"
    if [[ -n "$cleanup_info" ]]; then
      IFS=$'\t' read -r cleanup_ms cleanup_scanned cleanup_roots cleanup_paths cleanup_bytes <<<"$cleanup_info"
    fi
  fi

  start_ns="$(z00z_profile_now_ns)"
  started_at="$(z00z_profile_now_iso)"
  resource_time_bin="${Z00Z_RESOURCE_TIME_BIN:-/usr/bin/time}"
  resource_path=""
  run_root="$(z00z_profile_run_root)"
  if [[ -n "$run_root" && -x "$resource_time_bin" && "${Z00Z_PROFILE_COMMAND_RESOURCES:-1}" == "1" ]]; then
    safe_id="$(z00z_profile_safe_slug "$kind-$label-$start_ns")"
    resource_path="$(z00z_profile_res_dir)/$safe_id.time"
    mkdir -p "$(dirname "$resource_path")"
    z00z_profile_write_meta \
      "$resource_path" \
      "$kind" \
      "$label" \
      "$command_text" \
      "$cleanup_ms" \
      "$cleanup_scanned" \
      "$cleanup_roots" \
      "$cleanup_paths" \
      "$cleanup_bytes" \
      "$@"
  fi
  set +e
  if [[ -n "$resource_path" ]]; then
    "$resource_time_bin" -v -o "$resource_path" "$@"
    status=$?
  else
    "$@"
    status=$?
  fi
  if [[ "$had_errexit" -eq 1 ]]; then
    set -e
  fi
  end_ns="$(z00z_profile_now_ns)"
  ended_at="$(z00z_profile_now_iso)"

  z00z_profile_record_event \
    "$kind" \
    "$label" \
    "exit:$status" \
    "$start_ns" \
    "$end_ns" \
    "$started_at" \
    "$ended_at" \
    "$command_text"

  return "$status"
}
