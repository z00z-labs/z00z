#!/usr/bin/env bash

# unpack_z00z_project.sh
#
# Purpose:
#   Restore a portable Z00Z workspace created by pack_z00z_project.sh, rewrite
#   portable placeholders for the current machine, rebuild local environments,
#   and run the repository verification chain until the restored project is in
#   a working state.
#
# Interactive mode:
#   ./unpack_z00z_project.sh
#
#   Expected result:
#   - Prompts for the destination directory.
#   - Extracts the archive there, then runs the automated restore/install flow.
#
# Non-interactive mode:
#   ./unpack_z00z_project.sh --yes
#
#   Expected result:
#   - Uses ./z00z as the destination when --dest is not provided.
#   - Runs the full restore flow without further confirmation prompts.
#
# Docker sandbox mode:
#   ./unpack_z00z_project.sh --archive ./z00z-<pack-date>.tar.gz --docker-sandbox
#
#   Expected result:
#   - Launches a disposable Docker container and runs the full unpack/install/
#     verify flow inside that container instead of on the host.
#   - Leaves the host project tree and user home untouched apart from Docker's
#     own image/container cache.
#   - Treats --dest as a container-internal path. If omitted, /workspace/z00z
#     is used inside the sandbox container.
#   - Copies any generated reports/z00z-verification-orchestrator-* directories
#     back to the host after the inner run, even when the inner run exits with
#     a failure code, so diagnostics are not lost when the disposable container
#     is removed.
#
# Supported flags:
#   --archive <path>
#     Path to the portable archive. Relative paths are resolved against the
#     current working directory. If omitted, the script first tries
#     ./z00z-<today>.tar.gz and otherwise auto-selects the newest
#     ./z00z-*.tar.gz in the current working directory.
#
#   --dest <path>
#     Target directory for the restored project. The directory must not already
#     exist as a non-empty path.
#
#   --yes
#     Enable unattended mode. This suppresses the destination prompt and uses
#     defaults whenever the script can decide safely on its own.
#
#   --skip-formal-verification
#     Stop after extraction, placeholder replacement, planning-runtime restore,
#     symlink verification, and the optional penetration-tool check hook.
#
#   --docker-sandbox
#     Run the restore inside a disposable Docker container instead of directly
#     on the host. This is the safest mode when you want a real end-to-end
#     restore test without contaminating the host user environment. If Docker
#     is missing, the script attempts to install it first on supported hosts.
#
#   --docker-image <image>
#     Base image used for --docker-sandbox. Default: debian:12-slim
#     (Debian 12 / bookworm)
#
#   --sandbox-export-dir <path>
#     Host directory that receives exported verification report folders from
#     --docker-sandbox runs. Default: ./reports relative to the unpack script.
#
#   --keep-tmp
#     Keep the /tmp/z00z-unpack.* extraction directory after completion. Use
#     this only for debugging extraction or normalization issues.
#
#   -h, --help
#     Print the short CLI usage summary and exit.
#
# Example commands:
#   ./unpack_z00z_project.sh
#   ./unpack_z00z_project.sh --archive /tmp/z00z-<pack-date>.tar.gz --yes
#   ./unpack_z00z_project.sh --archive ./z00z-<pack-date>.tar.gz --dest ~/work/z00z --yes
#   ./unpack_z00z_project.sh --archive ./z00z-<pack-date>.tar.gz --docker-sandbox
#   ./unpack_z00z_project.sh --archive ./z00z-<pack-date>.tar.gz --docker-sandbox --docker-image debian:12-slim
#   ./unpack_z00z_project.sh --archive ./z00z-<pack-date>.tar.gz --docker-sandbox --sandbox-export-dir /tmp/z00z-sandbox-reports
#
# Docker sandbox cleanup commands:
#   The sandbox container is started with `--rm`, so on a normal exit Docker
#   removes the container automatically. If a run is interrupted, these
#   commands help clean only Z00Z sandbox leftovers without touching unrelated
#   Docker workloads:
#
#   List current Z00Z sandbox containers:
#     docker ps -a --filter 'name=z00z-unpack-sandbox-'
#
#   Stop all running Z00Z sandbox containers:
#     docker ps -q --filter 'name=z00z-unpack-sandbox-' | xargs -r docker stop
#
#   Remove all stopped or orphaned Z00Z sandbox containers:
#     docker ps -aq --filter 'name=z00z-unpack-sandbox-' | xargs -r docker rm -f
#
#   Show Docker disk usage before deciding on broader cleanup:
#     docker system df
#
#   Optionally remove the sandbox base image used by this script when it is no
#   longer needed by other workloads:
#     docker image rm debian:12-slim
#
#   Optionally clear only dangling build cache layers:
#     docker builder prune -f
#
#   Avoid broad host-wide cleanup commands such as `docker system prune -a`
#   unless you intentionally want to remove unrelated containers, images,
#   networks, and caches from the host.
#
# Restore actions performed by this script:
#   - Extract the packed project into the selected destination
#   - Replace portable placeholders with the actual unpack path and current
#     user home directory
#   - Restore the Python virtual environment with uv from packed metadata when present
#   - Restore a slim `.planning/` runtime tree rebuilt from the planning files
#     actually referenced by packed code/tests
#   - Reinstall or refresh repository-managed toolchains and surfaces
#   - Install VS Code from the official download service when missing, then
#     restore packed VS Code extensions
#   - Verify exact packed symlink targets before any long-running installation
#   - Rebuild generated indices and verify that all symlinks resolve
#   - Smoke-test writable cache directories
#   - Run repository cleanup and the full verification/report chain
#
# Main verification/install chain executed after extraction:
#   scripts/verification-tools/install-verification-tools.sh --install --profile research --strict
#   scripts/penetration/install_pentest_tools.sh
#   scripts/install_py_venv.sh
#   scripts/install_deep_wiki.sh
#   scripts/install_nvk_llm_wiki.sh
#   scripts/install_understand_anything.sh --install-pnpm
#   scripts/cargo_build.sh
#   scripts/z00z_cleanup.sh --yes
#   .github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run
#   .github/skills/z00z-verification-orchestrator/scripts/orchestrate.sh report project
#
# Optional pentest-only restore hook:
#   When --skip-formal-verification is used, the script skips the install and
#   verification chain and instead runs scripts/penetration/check_pentest_tools.sh
#   --json when that script exists, reporting missing tools without failing the
#   restore.
#
# Authentication behavior:
#   When sudo is available, the script opens one sudo keepalive session at the
#   start so package installation can reuse a single authentication window
#   instead of prompting repeatedly.
#
# Important notes:
#   - The packed archive excludes .git history by design.
#   - If codex is unavailable on the target machine, repo-local Deep Wiki
#     surfaces are still restored, while Codex-specific marketplace install
#     steps are skipped automatically.
#   - If uv is missing, the script installs it before Python environment
#     reconstruction so the restore path remains uv-first instead of pip-first.
#   - If git is missing, the script installs it before the heavier verifier and
#     repo-local tool bootstrap that depend on git clones/checkouts.
#   - If VS Code is missing, the script uses the official Visual Studio Code
#     download service for the current OS/architecture before restoring
#     extensions from the packed extension manifest.
#   - Docker sandbox mode still uses Docker's local image/layer storage on the
#     host, but it does not write restored project state into the host checkout
#     or host user home. If Docker is absent, the script attempts a host-level
#     Docker install before launching the sandbox.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERIFICATION_VERSIONS_FILE="$SCRIPT_DIR/scripts/verification-tools/versions.env"
DEFAULT_ARCHIVE_DATE="$(date +%F)"
DEFAULT_ARCHIVE_NAME="z00z-${DEFAULT_ARCHIVE_DATE}.tar.gz"
DEFAULT_DOCKER_IMAGE="debian:12-slim"
DEFAULT_SANDBOX_EXPORT_DIR="$SCRIPT_DIR/reports"
PORTABLE_ROOT_REL=".portable-transfer"
RUN_STARTED_AT="$(date +%s)"
if [[ -f "$VERIFICATION_VERSIONS_FILE" ]]; then
  # shellcheck disable=SC1090
  source "$VERIFICATION_VERSIONS_FILE"
fi
: "${Z00Z_VERUS_TOOLCHAIN:=1.96.1-x86_64-unknown-linux-gnu}"
export Z00Z_VERUS_TOOLCHAIN
readonly SCRIPT_DIR VERIFICATION_VERSIONS_FILE DEFAULT_ARCHIVE_DATE DEFAULT_ARCHIVE_NAME DEFAULT_DOCKER_IMAGE DEFAULT_SANDBOX_EXPORT_DIR
readonly PORTABLE_ROOT_REL RUN_STARTED_AT

ARCHIVE_PATH=""
ARCHIVE_PATH_SET=0
ARCHIVE_AUTO_SELECTED=0
DEST_PATH=""
AUTO_YES=0
DOCKER_SANDBOX=0
DOCKER_IMAGE="$DEFAULT_DOCKER_IMAGE"
SANDBOX_EXPORT_DIR="$DEFAULT_SANDBOX_EXPORT_DIR"
KEEP_TMP=0
SKIP_FORMAL_VERIFICATION=0
TMP_ROOT=""
SANDBOX_ASSET_ROOT=""
SUDO_KEEPALIVE_PID=""
PROJECT_ROOT=""
VSCODE_CLI=""
SANDBOX_MODE="${Z00Z_SANDBOX_MODE:-0}"
FULL_VERIFY_GATE_STARTED_AT=""

usage() {
  cat <<EOF
Usage:
  ./unpack_z00z_project.sh [--archive <archive.tar.gz>] [--dest <project-dir>] [--yes] [--skip-formal-verification] [--docker-sandbox] [--docker-image <image>] [--sandbox-export-dir <host-dir>] [--keep-tmp]

Unpacks a portable Z00Z archive, restores local toolchains, and runs the
repository verification chain.

Options:
  --archive <path>  Archive path. Default: ./$DEFAULT_ARCHIVE_NAME
                    If missing, auto-detect newest ./z00z-*.tar.gz in cwd.
  --dest <path>     Destination project directory. If omitted, prompt or use
                    ./z00z when --yes is set.
  --yes             Non-interactive mode.
  --skip-formal-verification
                    Stop after extraction, symlink verification, and the
                    optional penetration-tool check hook.
  --docker-sandbox  Run the full restore inside a disposable Docker container.
                    In this mode, --dest is interpreted inside the container.
                    If docker is missing, try to install it on the host first.
  --docker-image    Base image for --docker-sandbox. Default: debian:12-slim
                    (Debian 12 / bookworm)
  --sandbox-export-dir
                    Host directory that receives exported
                    reports/z00z-verification-orchestrator-* directories from
                    docker sandbox runs. Default: $DEFAULT_SANDBOX_EXPORT_DIR
  --keep-tmp        Keep the extraction temp directory for debugging.
  -h, --help        Show this help.
EOF
}

now_epoch() {
  date +%s
}

format_duration() {
  local total_seconds="${1:-0}"
  local hours=0
  local minutes=0
  local seconds=0

  (( total_seconds < 0 )) && total_seconds=0

  hours=$(( total_seconds / 3600 ))
  minutes=$(( (total_seconds % 3600) / 60 ))
  seconds=$(( total_seconds % 60 ))

  if (( hours > 0 )); then
    printf '%dh %02dm %02ds' "$hours" "$minutes" "$seconds"
    return 0
  fi

  if (( minutes > 0 )); then
    printf '%dm %02ds' "$minutes" "$seconds"
    return 0
  fi

  printf '%ds' "$seconds"
}

elapsed_seconds_since() {
  local started_at="$1"
  printf '%s' "$(( $(now_epoch) - started_at ))"
}

format_elapsed_since() {
  local started_at="$1"
  format_duration "$(elapsed_seconds_since "$started_at")"
}

is_sandbox_mode() {
  [[ "$DOCKER_SANDBOX" -eq 1 || "$SANDBOX_MODE" == "1" ]]
}

log() {
  printf '[unpack-z00z][+%s] %s\n' "$(format_elapsed_since "$RUN_STARTED_AT")" "$1"
}

warn() {
  printf '[unpack-z00z] WARNING: %s\n' "$1" >&2
}

die() {
  printf '[unpack-z00z] ERROR: %s\n' "$1" >&2
  exit 1
}

have() {
  command -v "$1" >/dev/null 2>&1
}

emit_duration_hint() {
  if is_sandbox_mode; then
    log "Duration hint: a cold Docker sandbox restore can take 30-120+ minutes. The longest wait is usually dependency and verifier-tool bootstrap before the full verify gate starts."
    return 0
  fi

  log "Duration hint: a cold host restore can take from several minutes to well over an hour, mostly during dependency and verifier-tool installation. Warm reruns are usually much faster."
}

cleanup() {
  if [[ -n "$SUDO_KEEPALIVE_PID" ]]; then
    kill "$SUDO_KEEPALIVE_PID" >/dev/null 2>&1 || true
  fi

  if [[ -n "$SANDBOX_ASSET_ROOT" && -d "$SANDBOX_ASSET_ROOT" ]]; then
    case "$SANDBOX_ASSET_ROOT" in
      /tmp/z00z-sandbox-assets.*)
        rm -rf -- "$SANDBOX_ASSET_ROOT"
        ;;
      *)
        warn "Refusing to remove unexpected sandbox asset directory: $SANDBOX_ASSET_ROOT"
        ;;
    esac
  fi

  if [[ "$KEEP_TMP" -eq 1 ]]; then
    if [[ -n "$TMP_ROOT" && -d "$TMP_ROOT" ]]; then
      log "Keeping temp directory: $TMP_ROOT"
    fi
    return 0
  fi

  if [[ -n "$TMP_ROOT" && -d "$TMP_ROOT" ]]; then
    case "$TMP_ROOT" in
      /tmp/z00z-unpack.*)
        rm -rf -- "$TMP_ROOT"
        ;;
      *)
        warn "Refusing to remove unexpected temp directory: $TMP_ROOT"
        ;;
    esac
  fi
}

trap cleanup EXIT

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --archive)
        [[ -n "${2:-}" ]] || die "--archive requires a value"
        ARCHIVE_PATH="$2"
        ARCHIVE_PATH_SET=1
        shift 2
        ;;
      --dest)
        [[ -n "${2:-}" ]] || die "--dest requires a value"
        DEST_PATH="$2"
        shift 2
        ;;
      --yes)
        AUTO_YES=1
        shift
        ;;
      --skip-formal-verification)
        SKIP_FORMAL_VERIFICATION=1
        shift
        ;;
      --docker-sandbox)
        DOCKER_SANDBOX=1
        shift
        ;;
      --docker-image)
        [[ -n "${2:-}" ]] || die "--docker-image requires a value"
        DOCKER_IMAGE="$2"
        shift 2
        ;;
      --sandbox-export-dir)
        [[ -n "${2:-}" ]] || die "--sandbox-export-dir requires a value"
        SANDBOX_EXPORT_DIR="$2"
        shift 2
        ;;
      --keep-tmp)
        KEEP_TMP=1
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        die "Unknown argument: $1"
        ;;
    esac
  done
}

select_default_archive_path() {
  local dated_candidate="$PWD/$DEFAULT_ARCHIVE_NAME"
  local newest_path=""
  local newest_mtime=-1
  local path=""
  local mtime=0
  local matches=()

  if [[ -f "$dated_candidate" ]]; then
    ARCHIVE_PATH="$dated_candidate"
    return 0
  fi

  shopt -s nullglob
  matches=("$PWD"/z00z-*.tar.gz)
  shopt -u nullglob

  if [[ "${#matches[@]}" -eq 0 ]]; then
    ARCHIVE_PATH="$dated_candidate"
    return 0
  fi

  for path in "${matches[@]}"; do
    mtime="$(stat -c '%Y' "$path" 2>/dev/null || printf '0')"
    if (( mtime > newest_mtime )); then
      newest_path="$path"
      newest_mtime="$mtime"
    fi
  done

  ARCHIVE_PATH="$newest_path"
  ARCHIVE_AUTO_SELECTED=1
}

normalize_paths() {
  if [[ "$ARCHIVE_PATH_SET" -eq 1 ]]; then
    case "$ARCHIVE_PATH" in
      /*) ;;
      *)
        ARCHIVE_PATH="$PWD/$ARCHIVE_PATH"
        ;;
    esac
  else
    select_default_archive_path
  fi

  if [[ "$DOCKER_SANDBOX" -eq 0 && -n "$DEST_PATH" ]]; then
    case "$DEST_PATH" in
      /*) ;;
      *)
        DEST_PATH="$PWD/$DEST_PATH"
        ;;
    esac
  fi

  if [[ -n "$SANDBOX_EXPORT_DIR" ]]; then
    case "$SANDBOX_EXPORT_DIR" in
      /*) ;;
      *)
        SANDBOX_EXPORT_DIR="$PWD/$SANDBOX_EXPORT_DIR"
        ;;
    esac
  fi
}

ensure_tools() {
  local tool

  for tool in python3 tar mktemp; do
    have "$tool" || die "Required command not found: $tool"
  done
}

container_destination_path() {
  local candidate="$DEST_PATH"

  if [[ -z "$candidate" ]]; then
    printf '/workspace/z00z'
    return 0
  fi

  case "$candidate" in
    /*)
      printf '%s' "$candidate"
      ;;
    *)
      candidate="${candidate#./}"
      printf '/workspace/%s' "$candidate"
      ;;
  esac
}

prompt_destination() {
  local default_dest="$PWD/z00z"
  local reply=""

  if [[ -n "$DEST_PATH" ]]; then
    return 0
  fi

  if [[ "$AUTO_YES" -eq 1 ]]; then
    DEST_PATH="$default_dest"
    return 0
  fi

  printf 'Destination directory [%s]: ' "$default_dest"
  read -r reply
  DEST_PATH="${reply:-$default_dest}"
  case "$DEST_PATH" in
    /*) ;;
    *)
      DEST_PATH="$PWD/$DEST_PATH"
      ;;
  esac
}

validate_archive() {
  [[ -f "$ARCHIVE_PATH" ]] || die "Archive not found: $ARCHIVE_PATH"
}

run_docker_sandbox() {
  local started_at
  local inner_archive_path
  local inner_dest_path
  local inner_script_path="/runner/unpack_z00z_project.sh"
  local inner_export_root="/host-reports"
  local inner_saw_suite_path="/portable-assets/saw-suite"
  local container_name
  local saw_suite_image
  local sandbox_saw_suite_src
  local sandbox_status

  started_at="$(now_epoch)"
  ensure_host_docker
  saw_suite_image="$(resolve_saw_suite_image)"

  inner_archive_path="/input/$(basename "$ARCHIVE_PATH")"
  inner_dest_path="$(container_destination_path)"
  container_name="z00z-unpack-sandbox-$$-$(date -u +%Y%m%d%H%M%S)"
  mkdir -p "$SANDBOX_EXPORT_DIR"
  SANDBOX_ASSET_ROOT="$(mktemp -d /tmp/z00z-sandbox-assets.XXXXXX)"
  sandbox_saw_suite_src="$(prepare_sandbox_saw_suite_asset "$saw_suite_image" "$SANDBOX_ASSET_ROOT")"

  log "Running disposable Docker sandbox with image $DOCKER_IMAGE"
  log "Container destination path: $inner_dest_path"
  log "Sandbox report export root on host: $SANDBOX_EXPORT_DIR"
  log "Sandbox progress will stream live from inside the container. The first major milestone is the start of z00z-full-verify-gate.sh."
  log "Sandbox will mount a readonly pinned saw-suite asset prepared from $saw_suite_image"

  # shellcheck disable=SC2016
  set +e
  docker_host run --rm \
    --name "$container_name" \
    --hostname "$container_name" \
    --workdir /workspace \
    --tmpfs /tmp:rw,exec,nosuid,nodev,size=16g \
    --tmpfs /run:rw,nosuid,nodev,size=1g \
    -e DEBIAN_FRONTEND=noninteractive \
    -e Z00Z_SANDBOX_MODE=1 \
    -e Z00Z_SANDBOX_ARCHIVE="$inner_archive_path" \
    -e Z00Z_SANDBOX_DEST="$inner_dest_path" \
    -e Z00Z_SANDBOX_EXPORT_ROOT="$inner_export_root" \
    -e Z00Z_SANDBOX_SCRIPT="$inner_script_path" \
    -e Z00Z_PORTABLE_SAW_SUITE_SRC="$inner_saw_suite_path" \
    -e Z00Z_VERUS_TOOLCHAIN="$Z00Z_VERUS_TOOLCHAIN" \
    --mount "type=bind,src=$ARCHIVE_PATH,dst=$inner_archive_path,readonly" \
    --mount "type=bind,src=$SANDBOX_EXPORT_DIR,dst=$inner_export_root" \
    --mount "type=bind,src=$SCRIPT_DIR/unpack_z00z_project.sh,dst=$inner_script_path,readonly" \
    --mount "type=bind,src=$sandbox_saw_suite_src,dst=$inner_saw_suite_path,readonly" \
    "$DOCKER_IMAGE" \
    bash -lc '
      set -euo pipefail
      export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
      export RUNLEVEL=1
      inner_status=0

      apt_quiet() {
        "$@" \
          > >(sed \
            -e "s/\r$//" \
            -e "/^invoke-rc.d: could not determine current runlevel$/d" \
            -e "/^invoke-rc.d: policy-rc.d denied execution of .*$/d") \
          2> >(sed \
            -e "s/\r$//" \
            -e "/^invoke-rc.d: could not determine current runlevel$/d" \
            -e "/^invoke-rc.d: policy-rc.d denied execution of .*$/d" >&2)
      }

      if [[ -d /etc/dpkg/dpkg.cfg.d ]]; then
        while IFS= read -r -d "" cfg; do
          sed -i \
            -e "/^path-exclude=\\/usr\\/share\\/man\\/\\*/d" \
            -e "/^path-include=\\/usr\\/share\\/man\\//d" \
            "$cfg"
        done < <(find /etc/dpkg/dpkg.cfg.d -maxdepth 1 -type f -print0 2>/dev/null)
      fi

      if ! getent group systemd-journal >/dev/null 2>&1; then
        groupadd -r systemd-journal >/dev/null 2>&1 || true
      fi
      if ! getent group systemd-network >/dev/null 2>&1; then
        groupadd -r systemd-network >/dev/null 2>&1 || true
      fi
      if ! id -u systemd-network >/dev/null 2>&1; then
        useradd --system --no-create-home --gid systemd-network --shell /usr/sbin/nologin systemd-network >/dev/null 2>&1 || true
      fi

      if command -v apt-get >/dev/null 2>&1; then
        echo "[unpack-z00z][sandbox-bootstrap] Preparing base image packages with apt-get"
        apt_quiet apt-get update >/dev/null
        DEBCONF_NOWARNINGS=yes RUNLEVEL=1 apt_quiet apt-get install -y --no-install-recommends apt-utils >/dev/null
        DEBCONF_NOWARNINGS=yes RUNLEVEL=1 apt_quiet apt-get install -y --no-install-recommends bash ca-certificates coreutils findutils tar python3 curl sudo git gpg >/dev/null
      elif command -v pacman >/dev/null 2>&1; then
        echo "[unpack-z00z][sandbox-bootstrap] Preparing base image packages with pacman"
        pacman -Sy --needed --noconfirm bash ca-certificates coreutils findutils tar python curl sudo git >/dev/null
      else
        echo "[unpack-z00z] ERROR: docker sandbox bootstrap requires apt-get or pacman in the base image" >&2
        exit 1
      fi

      if ! id -u z00z >/dev/null 2>&1; then
        useradd --create-home --shell /bin/bash z00z >/dev/null 2>&1 || true
      fi
      mkdir -p /etc/sudoers.d /workspace
      printf "%s\n" "z00z ALL=(ALL) NOPASSWD:ALL" >/etc/sudoers.d/90-z00z
      chmod 440 /etc/sudoers.d/90-z00z
      chown -R z00z:z00z /workspace

      echo "[unpack-z00z][sandbox-bootstrap] Base image bootstrap finished; starting inner restore as non-root user z00z"
      sudo -H -u z00z env Z00Z_SANDBOX_MODE=1 Z00Z_PORTABLE_SAW_SUITE_SRC="$Z00Z_PORTABLE_SAW_SUITE_SRC" Z00Z_VERUS_TOOLCHAIN="$Z00Z_VERUS_TOOLCHAIN" PATH=/home/z00z/.local/bin:/home/z00z/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
        bash "$Z00Z_SANDBOX_SCRIPT" \
        --archive "$Z00Z_SANDBOX_ARCHIVE" \
        --dest "$Z00Z_SANDBOX_DEST" \
        --yes || inner_status=$?

      if [[ -d "$Z00Z_SANDBOX_DEST/reports" && -d "$Z00Z_SANDBOX_EXPORT_ROOT" ]]; then
        while IFS= read -r -d "" report_dir; do
          export_name="$(basename "$report_dir")"
          target_dir="$Z00Z_SANDBOX_EXPORT_ROOT/$export_name"
          if [[ -e "$target_dir" ]]; then
            target_dir="$Z00Z_SANDBOX_EXPORT_ROOT/${export_name}-sandbox-export-$(date -u +%Y%m%d%H%M%S)"
          fi
          cp -a "$report_dir" "$target_dir"
          echo "[unpack-z00z][sandbox-export] exported $(basename "$target_dir")"
        done < <(find "$Z00Z_SANDBOX_DEST/reports" -maxdepth 1 -mindepth 1 -type d -name "z00z-verification-orchestrator-*" -print0 2>/dev/null)
      fi

      exit "$inner_status"
    '
  sandbox_status=$?
  set -e

  log "Docker sandbox wrapper completed in $(format_elapsed_since "$started_at")"
  return "$sandbox_status"
}

prepare_destination() {
  if [[ -e "$DEST_PATH" ]]; then
    if [[ -d "$DEST_PATH" ]] && [[ -z "$(find "$DEST_PATH" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]]; then
      rmdir "$DEST_PATH"
      return 0
    fi
    die "Destination already exists and is not empty: $DEST_PATH"
  fi

  mkdir -p "$(dirname "$DEST_PATH")"
}

start_sudo_session() {
  if [[ "$EUID" -eq 0 ]]; then
    return 0
  fi

  if ! have sudo; then
    warn "sudo is not available; installation will continue in user-space where possible"
    return 0
  fi

  if ! sudo -v; then
    warn "sudo authentication failed; continuing without a cached sudo session"
    return 0
  fi

  while true; do
    sudo -n true >/dev/null 2>&1 || exit
    sleep 45
  done &
  SUDO_KEEPALIVE_PID=$!
}

as_root() {
  if [[ "$EUID" -eq 0 ]]; then
    "$@"
    return $?
  fi

  if have sudo; then
    sudo "$@"
    return $?
  fi

  return 1
}

docker_host() {
  if have docker && docker info >/dev/null 2>&1; then
    docker "$@"
    return $?
  fi

  if as_root docker info >/dev/null 2>&1; then
    as_root docker "$@"
    return $?
  fi

  return 1
}

resolve_saw_suite_image() {
  local versions_file="$SCRIPT_DIR/scripts/verification-tools/versions.env"
  local saw_suite_image_default="ghcr.io/galoisinc/saw-suite@sha256:aabdbf3442fffe35dc56cabf8ddd1d473d291df8226f5cf018009a94cfc4151f"

  if [[ -n "${Z00Z_SAW_SUITE_IMAGE:-}" ]]; then
    printf '%s\n' "$Z00Z_SAW_SUITE_IMAGE"
    return 0
  fi

  if [[ -f "$versions_file" ]]; then
    # shellcheck disable=SC1090
    source "$versions_file"
    if [[ -n "${Z00Z_SAW_SUITE_IMAGE:-}" ]]; then
      printf '%s\n' "$Z00Z_SAW_SUITE_IMAGE"
      return 0
    fi
  fi

  printf '%s\n' "$saw_suite_image_default"
}

prepare_sandbox_saw_suite_asset() {
  local image_ref="$1"
  local asset_root="$2"
  local saw_suite_dir="$asset_root/saw-suite"
  local container_id=""

  mkdir -p "$saw_suite_dir"

  log "Preparing readonly saw-suite sandbox asset from $image_ref" >&2
  docker_host pull "$image_ref" >/dev/null
  container_id="$(docker_host create "$image_ref")"
  if ! docker_host cp "$container_id:/opt/saw-suite/." "$saw_suite_dir/"; then
    docker_host rm -f "$container_id" >/dev/null 2>&1 || true
    die "Failed to extract /opt/saw-suite from $image_ref for docker sandbox"
  fi
  docker_host rm -f "$container_id" >/dev/null 2>&1 || true

  printf '%s\n' "$saw_suite_dir"
}

ensure_host_docker() {
  if docker_host info >/dev/null 2>&1; then
    return 0
  fi

  log "docker was not found or is not ready; attempting host install for sandbox mode"
  start_sudo_session

  if ! have docker; then
    if have apt-get; then
      as_root env DEBIAN_FRONTEND=noninteractive apt-get update
      as_root env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends docker.io
    elif have pacman; then
      as_root pacman -Sy --needed --noconfirm docker
    else
      die "Could not auto-install docker: no supported package manager (apt-get or pacman) was found"
    fi
  fi

  if have systemctl; then
    as_root systemctl enable --now docker >/dev/null 2>&1 || true
  elif have service; then
    as_root service docker start >/dev/null 2>&1 || true
  fi

  docker_host info >/dev/null 2>&1 \
    || die "docker CLI is present, but the docker daemon is not ready for --docker-sandbox"
}

extract_archive() {
  local started_at
  local extract_root
  local extracted_dir

  started_at="$(now_epoch)"
  TMP_ROOT="$(mktemp -d /tmp/z00z-unpack.XXXXXX)"
  extract_root="$TMP_ROOT/extracted"
  mkdir -p "$extract_root"

  log "Extracting archive"
  tar -C "$extract_root" -xzf "$ARCHIVE_PATH"

  extracted_dir="$(find "$extract_root" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
  [[ -n "$extracted_dir" ]] || die "Archive does not contain a top-level project directory"

  mv "$extracted_dir" "$DEST_PATH"
  PROJECT_ROOT="$DEST_PATH"
  log "Project extracted to $PROJECT_ROOT in $(format_elapsed_since "$started_at")"
}

restore_planning_runtime_bundle() {
  local started_at
  local bundle_root
  local bundle_manifest
  local copied_count=0
  local missing_count=0

  bundle_root="$PROJECT_ROOT/$PORTABLE_ROOT_REL/planning-runtime"
  bundle_manifest="$bundle_root/manifest.json"

  if [[ ! -d "$bundle_root" || ! -f "$bundle_manifest" ]]; then
    log "No slim planning runtime bundle was packed; continuing without .planning restore"
    return 0
  fi

  started_at="$(now_epoch)"
  log "Restoring slim .planning runtime bundle"

  python3 - "$PROJECT_ROOT" "$bundle_root" <<'PY'
import json
import shutil
import sys
from pathlib import Path

project_root = Path(sys.argv[1]).resolve()
bundle_root = Path(sys.argv[2]).resolve()
manifest = json.loads((bundle_root / "manifest.json").read_text(encoding="utf-8"))

for rel in manifest.get("copied_files", []):
    src = bundle_root / rel
    dst = project_root / rel
    if not src.is_file():
        continue
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)
PY
  read -r copied_count missing_count < <(
    python3 - "$bundle_manifest" <<'PY'
import json
import sys
from pathlib import Path

manifest = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(len(manifest.get("copied_files", [])), len(manifest.get("missing_files", [])))
PY
  )

  log "Restored slim .planning runtime bundle in $(format_elapsed_since "$started_at"): ${copied_count:-0} files copied, ${missing_count:-0} unresolved references remained excluded"
}

replace_placeholders() {
  local started_at

  started_at="$(now_epoch)"
  log "Replacing portable placeholders"

  python3 - "$PROJECT_ROOT" <<'PY'
import json
import os
import sys
from pathlib import Path

project_root = Path(sys.argv[1]).resolve()
home_dir = Path.home().resolve()
state_path = project_root / ".portable-transfer" / "normalization-state.json"

text_suffixes = {
    ".md",
    ".txt",
    ".json",
    ".yaml",
    ".yml",
    ".toml",
    ".py",
    ".sh",
    ".cjs",
    ".mjs",
    ".js",
    ".ts",
    ".tsx",
    ".jsx",
    ".rs",
    ".html",
    ".css",
    ".env",
    ".cfg",
    ".conf",
    ".config",
    ".lock",
    ".ml",
    ".mli",
    ".mll",
    ".mly",
    ".opam",
    ".install",
}

text_names = {
    "Cargo.toml",
    "Cargo.lock",
    "Makefile",
    "VERSION",
    "README",
    "README.md",
    "pyvenv.cfg",
}

portable_text_roots = (
    project_root / "tools/formal_verification/opam/root",
)

def is_probably_utf8_text(path: Path) -> bool:
    try:
        with path.open("rb") as handle:
            sample = handle.read(4096)
    except OSError:
        return False
    if sample.startswith(b"#!"):
        return True
    if b"\x00" in sample:
        return False
    try:
        sample.decode("utf-8")
    except UnicodeDecodeError:
        return False
    return True

def is_text_candidate(path: Path) -> bool:
    if path.name in text_names:
        return True
    if path.suffix.lower() in text_suffixes:
        return True
    for root in portable_text_roots:
        if path.is_relative_to(root):
            return is_probably_utf8_text(path)
    if path.suffix:
        return False
    return is_probably_utf8_text(path)

candidate_paths = None
if state_path.is_file():
    try:
        state = json.loads(state_path.read_text(encoding="utf-8"))
        rewritten_files = state.get("rewritten_files", [])
        if isinstance(rewritten_files, list):
            candidate_paths = [project_root / rel for rel in rewritten_files]
    except json.JSONDecodeError:
        candidate_paths = None

if candidate_paths is None:
    candidate_paths = list(project_root.rglob("*"))

for path in candidate_paths:
    if not path.is_file():
        continue
    if not is_text_candidate(path):
        continue
    try:
        content = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        continue
    updated = content.replace("__Z00Z_PROJECT_ROOT__", str(project_root))
    updated = updated.replace("__Z00Z_USER_HOME__", str(home_dir))
    if updated != content:
        path.write_text(updated, encoding="utf-8")
PY
  log "Finished replacing portable placeholders in $(format_elapsed_since "$started_at")"
}

install_minimal_packages() {
  local missing=()

  have curl || missing+=(curl)
  have python3 || missing+=(python3)
  have git || missing+=(git)

  if [[ "${#missing[@]}" -eq 0 ]]; then
    return 0
  fi

  if have apt-get; then
    log "Installing minimal prerequisites with apt-get"
    as_root env DEBIAN_FRONTEND=noninteractive apt-get update
    as_root env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends "${missing[@]}"
    return 0
  fi

  if have pacman; then
    log "Installing minimal prerequisites with pacman"
    as_root pacman -Sy --needed --noconfirm "${missing[@]}"
    return 0
  fi

  warn "Could not install missing prerequisites automatically: ${missing[*]}"
}

ensure_uv_available() {
  local started_at

  started_at="$(now_epoch)"

  export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

  if have uv; then
    log "Using uv $(uv --version | awk '{print $2}') at $(command -v uv)"
    return 0
  fi

  have curl || die "curl is required to install uv automatically"
  log "uv was not found; installing it for portable restore"
  curl -LsSf https://astral.sh/uv/install.sh | sh
  export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

  have uv || die "uv installation finished, but the uv binary is still not available in PATH"
  log "Installed uv $(uv --version | awk '{print $2}') at $(command -v uv) in $(format_elapsed_since "$started_at")"
}

read_manifest_value() {
  local expression="$1"
  local manifest_path="$PROJECT_ROOT/$PORTABLE_ROOT_REL/manifest.json"

  python3 - "$manifest_path" "$expression" <<'PY'
import json
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
expression = sys.argv[2]
data = json.loads(manifest_path.read_text(encoding="utf-8"))

current = data
for part in expression.split("."):
    if isinstance(current, dict):
        current = current.get(part, "")
    else:
        current = ""
        break

if current is None:
    current = ""
if isinstance(current, bool):
    print("true" if current else "false")
else:
    print(current)
PY
}

verify_packed_symlink_manifest() {
  local started_at
  local manifest_path="$PROJECT_ROOT/$PORTABLE_ROOT_REL/symlink-manifest.json"
  local result=""
  local total_symlinks=0
  local codex_symlinks=0

  if [[ ! -f "$manifest_path" ]]; then
    warn "Packed symlink manifest is missing; falling back to broken-link verification only"
    return 0
  fi

  started_at="$(now_epoch)"
  result="$(
    python3 - "$PROJECT_ROOT" "$manifest_path" <<'PY'
import json
import os
import sys
from pathlib import Path

project_root = Path(sys.argv[1]).resolve()
manifest_path = Path(sys.argv[2])
data = json.loads(manifest_path.read_text(encoding="utf-8"))
expected_entries = data.get("entries", [])
expected = {}
for entry in expected_entries:
    path = entry.get("path")
    target = entry.get("target")
    if path is None or target is None:
        continue
    expected[path] = target

actual = {}
broken = []
for path in project_root.rglob("*"):
    if not path.is_symlink():
        continue
    rel = path.relative_to(project_root).as_posix()
    actual[rel] = os.readlink(path)
    if not path.exists():
        broken.append(rel)

missing = sorted(set(expected) - set(actual))
unexpected = sorted(set(actual) - set(expected))
mismatched = sorted(path for path in set(expected) & set(actual) if expected[path] != actual[path])

if missing:
    for path in missing[:20]:
        print(f"[unpack-z00z] MISSING symlink: {path} -> {expected[path]}", file=sys.stderr)
if unexpected:
    for path in unexpected[:20]:
        print(f"[unpack-z00z] UNEXPECTED symlink: {path} -> {actual[path]}", file=sys.stderr)
if mismatched:
    for path in mismatched[:20]:
        print(
            f"[unpack-z00z] MISMATCHED symlink: {path} expected {expected[path]!r} got {actual[path]!r}",
            file=sys.stderr,
        )
if broken:
    for path in broken[:20]:
        print(f"[unpack-z00z] BROKEN symlink after extract: {path} -> {actual[path]}", file=sys.stderr)

if missing or unexpected or mismatched or broken:
    sys.exit(1)

codex_count = sum(1 for path in expected if path.startswith(".codex/"))
print(f"{len(expected)} {codex_count}")
PY
  )" || die "Restored symlinks do not match the packed symlink manifest"

  read -r total_symlinks codex_symlinks <<<"$result"
  log "Verified exact symlink targets from manifest in $(format_elapsed_since "$started_at"): ${total_symlinks} entries, ${codex_symlinks} under .codex/"
}

source_verify_env() {
  # shellcheck disable=SC1091
  source "$PROJECT_ROOT/scripts/verify-env.sh"
}

run_repo_script() {
  local description="$1"
  local started_at
  shift

  started_at="$(now_epoch)"
  log "START: $description"
  (
    cd "$PROJECT_ROOT"
    "$@"
  )
  log "DONE: $description (step $(format_elapsed_since "$started_at"))"
}

run_repo_shell() {
  local description="$1"
  local started_at
  shift

  started_at="$(now_epoch)"
  log "START: $description"
  (
    cd "$PROJECT_ROOT"
    bash -lc "$*"
  )
  log "DONE: $description (step $(format_elapsed_since "$started_at"))"
}

latest_orchestrator_report_since() {
  local min_epoch="${1:-0}"
  local newest_epoch=-1
  local newest_path=""
  local path=""
  local mtime_epoch=0

  shopt -s nullglob
  for path in "$PROJECT_ROOT"/reports/z00z-verification-orchestrator-*/z00z-verification-report.md; do
    [[ -f "$path" ]] || continue
    mtime_epoch="$(stat -c '%Y' "$path" 2>/dev/null || printf '0')"
    if (( mtime_epoch < min_epoch )); then
      continue
    fi
    if (( mtime_epoch > newest_epoch )); then
      newest_epoch="$mtime_epoch"
      newest_path="$path"
    fi
  done
  shopt -u nullglob

  [[ -n "$newest_path" ]] || return 1
  printf '%s\n' "$newest_path"
}

validate_orchestrator_report_contract() {
  local report_path="$1"
  local run_root=""
  local summary_out=""
  local tmp_parent=""

  [[ -f "$report_path" ]] || return 1

  run_root="$(cd "$(dirname "$report_path")" && pwd)"

  if [[ -n "$TMP_ROOT" ]]; then
    mkdir -p "$TMP_ROOT" >/dev/null 2>&1 || true
    if [[ -d "$TMP_ROOT" ]]; then
      tmp_parent="$TMP_ROOT"
    fi
  fi

  if [[ -z "$tmp_parent" ]]; then
    tmp_parent="/tmp"
  fi

  summary_out="$(mktemp "${tmp_parent%/}/z00z-orchestrator-report-validation.XXXXXX.json")" \
    || return 1

  if python3 "$PROJECT_ROOT/.github/skills/z00z-verification-orchestrator/scripts/validate-report-format.py" \
    --report "$report_path" \
    --run-root "$run_root" \
    --root "$PROJECT_ROOT" \
    --scope-kind project \
    --format-path ".github/skills/z00z-verification-orchestrator/FORMAT.md" \
    --summary-out "$summary_out" >/dev/null; then
    rm -f -- "$summary_out"
    return 0
  fi

  rm -f -- "$summary_out"
  return 1
}

run_orchestrator_report_project() {
  local description="Running verification orchestrator report project"
  local started_at

  started_at="$(now_epoch)"
  log "START: $description"
  (
    local command_status=0
    local report_path=""

    cd "$PROJECT_ROOT"

    set +e
    ./.github/skills/z00z-verification-orchestrator/scripts/orchestrate.sh report project
    command_status=$?
    set -e

    if [[ "$command_status" -eq 0 ]]; then
      exit 0
    fi

    report_path="$(latest_orchestrator_report_since "$started_at" || true)"
    if [[ -n "$report_path" ]] && validate_orchestrator_report_contract "$report_path"; then
      printf '[unpack-z00z] NOTE: verification orchestrator returned %s because the generated report status is non-pass; report generation and contract validation still succeeded at %s\n' \
        "$command_status" "${report_path#"$PROJECT_ROOT"/}"
      exit 0
    fi

    exit "$command_status"
  )
  log "DONE: $description (step $(format_elapsed_since "$started_at"))"
}

restore_python_venv() {
  local venv_present
  local python_minor
  local freeze_file="$PROJECT_ROOT/$PORTABLE_ROOT_REL/python-venv/pip-freeze.txt"

  venv_present="$(read_manifest_value "python_venv.present")"
  python_minor="$(read_manifest_value "python_venv.minor_version")"

  if [[ "$venv_present" != "true" ]]; then
    log "No packed Python venv metadata found; skipping venv restore"
    return 0
  fi

  if [[ -n "$python_minor" ]]; then
    run_repo_script "Creating Python venv" ./scripts/install_py_venv.sh --python "$python_minor"
  else
    run_repo_script "Creating Python venv" ./scripts/install_py_venv.sh
  fi

  if [[ -s "$freeze_file" ]]; then
    run_repo_shell \
      "Restoring Python packages from uv freeze snapshot" \
      "set -euo pipefail; uv pip sync --python ./.venv/bin/python '$freeze_file'"
  fi
}

detect_vscode_cli() {
  export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
  if have code; then
    VSCODE_CLI="code"
  elif have codium; then
    VSCODE_CLI="codium"
  elif [[ -x "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code" ]]; then
    VSCODE_CLI="/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code"
  elif [[ -x "$HOME/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code" ]]; then
    VSCODE_CLI="$HOME/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code"
  elif [[ -x "$HOME/.local/opt/vscode/bin/code" ]]; then
    VSCODE_CLI="$HOME/.local/opt/vscode/bin/code"
  elif [[ -x "$HOME/.local/bin/code" ]]; then
    VSCODE_CLI="$HOME/.local/bin/code"
  else
    VSCODE_CLI=""
  fi
}

detect_vscode_platform_arch() {
  local machine_arch

  machine_arch="$(uname -m)"
  case "$machine_arch" in
    x86_64|amd64)
      printf 'x64'
      ;;
    aarch64|arm64)
      printf 'arm64'
      ;;
    armv7l|armv7*|armhf)
      printf 'armhf'
      ;;
    i386|i686)
      printf 'ia32'
      ;;
    *)
      printf ''
      ;;
  esac
}

download_vscode_installer() {
  local url="$1"
  local dest_path="$2"

  mkdir -p "$(dirname "$dest_path")"
  curl -fL "$url" -o "$dest_path"
}

install_vscode_linux_from_download() {
  local arch="$1"
  local download_dir="$TMP_ROOT/vscode-installer"
  local deb_path="$download_dir/vscode.deb"
  local rpm_path="$download_dir/vscode.rpm"
  local tarball_path="$download_dir/vscode.tar.gz"
  local tar_extract_dir="$download_dir/tarball"
  local tar_root=""
  local target_root="$HOME/.local/opt/vscode"
  local user_bin_dir="$HOME/.local/bin"

  case "$arch" in
    x64|arm64|armhf) ;;
    *)
      warn "Unsupported Linux architecture for official VS Code install: $(uname -m)"
      return 1
      ;;
  esac

  if have apt-get; then
    log "Installing VS Code from the official Linux .deb package"
    if ! have gpg; then
      log "Installing gpg because the official VS Code .deb post-install script requires it"
      as_root env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends gpg
    fi
    if have debconf-set-selections; then
      printf '%s\n' "code code/add-microsoft-repo boolean true" | as_root debconf-set-selections || true
    fi
    download_vscode_installer \
      "https://update.code.visualstudio.com/latest/linux-deb-${arch}/stable" \
      "$deb_path"
    as_root env DEBIAN_FRONTEND=noninteractive apt-get install -y "$deb_path"
    return 0
  fi

  if have dnf || have yum || have zypper || have rpm; then
    log "Installing VS Code from the official Linux .rpm package"
    download_vscode_installer \
      "https://update.code.visualstudio.com/latest/linux-rpm-${arch}/stable" \
      "$rpm_path"
    if have dnf; then
      as_root dnf install -y "$rpm_path"
    elif have yum; then
      as_root yum install -y "$rpm_path"
    elif have zypper; then
      as_root zypper --non-interactive install "$rpm_path"
    else
      as_root rpm -Uvh --replacepkgs "$rpm_path"
    fi
    return 0
  fi

  log "Installing VS Code from the official Linux tarball"
  download_vscode_installer \
    "https://update.code.visualstudio.com/latest/linux-${arch}/stable" \
    "$tarball_path"
  mkdir -p "$tar_extract_dir" "$HOME/.local/opt" "$user_bin_dir"
  tar -xzf "$tarball_path" -C "$tar_extract_dir"
  tar_root="$(find "$tar_extract_dir" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
  [[ -n "$tar_root" ]] || {
    warn "Could not find extracted VS Code directory inside the Linux tarball"
    return 1
  }
  if [[ -e "$target_root" && ! -L "$target_root" ]]; then
    mv "$target_root" "${target_root}.bak-$(date +%Y%m%d%H%M%S)"
  fi
  mv "$tar_root" "$target_root"
  ln -sfn "$target_root/bin/code" "$user_bin_dir/code"
  export PATH="$user_bin_dir:$PATH"
}

install_vscode_macos_from_download() {
  local download_dir="$TMP_ROOT/vscode-installer"
  local dmg_path="$download_dir/VSCode.dmg"
  local mount_point="$download_dir/mount"
  local source_app=""
  local target_app=""
  local symlink_target=""

  log "Installing VS Code from the official macOS download"
  download_vscode_installer \
    "https://update.code.visualstudio.com/latest/darwin-universal/stable" \
    "$dmg_path"
  mkdir -p "$mount_point"
  hdiutil attach -nobrowse -mountpoint "$mount_point" "$dmg_path" >/dev/null
  source_app="$(find "$mount_point" -mindepth 1 -maxdepth 1 -type d -name 'Visual Studio Code.app' | head -n 1)"
  [[ -n "$source_app" ]] || {
    hdiutil detach "$mount_point" >/dev/null 2>&1 || true
    warn "Could not find Visual Studio Code.app inside the mounted DMG"
    return 1
  }

  if as_root test -d /Applications >/dev/null 2>&1; then
    target_app="/Applications/Visual Studio Code.app"
    if [[ -e "$target_app" ]]; then
      as_root mv "$target_app" "${target_app}.bak-$(date +%Y%m%d%H%M%S)"
    fi
    as_root ditto "$source_app" "$target_app"
    as_root mkdir -p /usr/local/bin
    as_root ln -sfn "$target_app/Contents/Resources/app/bin/code" /usr/local/bin/code
    symlink_target="/usr/local/bin/code"
  else
    mkdir -p "$HOME/Applications" "$HOME/.local/bin"
    target_app="$HOME/Applications/Visual Studio Code.app"
    if [[ -e "$target_app" ]]; then
      mv "$target_app" "${target_app}.bak-$(date +%Y%m%d%H%M%S)"
    fi
    ditto "$source_app" "$target_app"
    ln -sfn "$target_app/Contents/Resources/app/bin/code" "$HOME/.local/bin/code"
    export PATH="$HOME/.local/bin:$PATH"
    symlink_target="$HOME/.local/bin/code"
  fi

  hdiutil detach "$mount_point" >/dev/null
  [[ -x "$symlink_target" ]] || warn "VS Code CLI symlink was created, but it is not executable yet: $symlink_target"
}

install_vscode_windows_from_download() {
  local arch="$1"
  local download_dir="$TMP_ROOT/vscode-installer"
  local installer_path="$download_dir/VSCodeSetup.exe"
  local win_installer_path=""
  local platform=""

  case "$arch" in
    x64)
      platform="win32-x64-user"
      ;;
    arm64)
      platform="win32-arm64-user"
      ;;
    ia32)
      platform="win32-user"
      ;;
    *)
      warn "Unsupported Windows architecture for official VS Code install: $(uname -m)"
      return 1
      ;;
  esac

  if ! have powershell.exe; then
    warn "powershell.exe is required for automated VS Code install on Windows-like shells"
    return 1
  fi

  log "Installing VS Code from the official Windows user installer"
  download_vscode_installer \
    "https://update.code.visualstudio.com/latest/${platform}/stable" \
    "$installer_path"
  if have cygpath; then
    win_installer_path="$(cygpath -w "$installer_path")"
  else
    win_installer_path="$installer_path"
  fi
  powershell.exe -NoProfile -ExecutionPolicy Bypass -Command \
    "Start-Process -FilePath '$win_installer_path' -ArgumentList '/VERYSILENT','/NORESTART','/MERGETASKS=!runcode' -Wait"
}

install_vscode_from_official_download() {
  local started_at
  local os_name
  local arch

  started_at="$(now_epoch)"
  os_name="$(uname -s)"
  arch="$(detect_vscode_platform_arch)"

  case "$os_name" in
    Linux)
      install_vscode_linux_from_download "$arch"
      ;;
    Darwin)
      have hdiutil || {
        warn "hdiutil is required to install VS Code automatically on macOS"
        return 1
      }
      install_vscode_macos_from_download
      ;;
    CYGWIN*|MINGW*|MSYS*)
      install_vscode_windows_from_download "$arch"
      ;;
    *)
      warn "Automatic official VS Code install is not implemented for OS: $os_name"
      return 1
      ;;
  esac

  log "Official VS Code install attempt finished in $(format_elapsed_since "$started_at")"
}

ensure_vscode_cli() {
  detect_vscode_cli
  if [[ -n "$VSCODE_CLI" ]]; then
    return 0
  fi

  install_vscode_from_official_download || true

  detect_vscode_cli
  if [[ -z "$VSCODE_CLI" ]]; then
    warn "VS Code CLI could not be installed automatically from the official Visual Studio Code download service; extension restore will be skipped"
  fi
}

install_vscode_extensions() {
  local extensions_file="$PROJECT_ROOT/$PORTABLE_ROOT_REL/vscode/extensions.txt"
  local extension=""
  local name=""
  local version=""

  ensure_vscode_cli
  if [[ -z "$VSCODE_CLI" ]]; then
    warn "VS Code CLI was not found; skipping extension restore"
    return 0
  fi

  [[ -f "$extensions_file" ]] || return 0

  while IFS= read -r extension; do
    [[ -n "$extension" ]] || continue
    name="${extension%@*}"
    version="${extension##*@}"
    if [[ "$name" == "$version" ]]; then
      "$VSCODE_CLI" --install-extension "$name" --force >/dev/null 2>&1 || warn "Could not install extension: $name"
      continue
    fi
    "$VSCODE_CLI" --install-extension "${name}@${version}" --force >/dev/null 2>&1 \
      || "$VSCODE_CLI" --install-extension "$name" --force >/dev/null 2>&1 \
      || warn "Could not install extension: ${name}@${version}"
  done <"$extensions_file"
}

verify_symlinks() {
  local broken=0
  local link=""
  local rel=""

  while IFS= read -r -d '' link; do
    rel="${link#"$PROJECT_ROOT"/}"
    case "$rel" in
      tools/formal_verification/charon/src/doc-rust.html|\
      tools/formal_verification/charon/src/doc-ml.html|\
      tools/formal_verification/aeneas/src/doc.html)
        continue
        ;;
    esac
    if [[ ! -e "$link" ]]; then
      warn "Broken symlink: $rel -> $(readlink "$link")"
      broken=1
    fi
  done < <(find "$PROJECT_ROOT" -type l -print0)

  if [[ "$broken" -ne 0 ]]; then
    die "Broken symlinks remain after restore"
  fi
}

ensure_gsd_entrypoints_executable() {
  local rel=""
  local abs=""

  for rel in \
    ".github/gsd-core/bin/gsd-tools.cjs" \
    ".github/gsd-core/bin/verify-reapply-patches.cjs" \
    ".github/gsd-core/bin/check-latest-version.cjs"; do
    abs="$PROJECT_ROOT/$rel"
    [[ -f "$abs" ]] || continue
    if [[ -x "$abs" ]]; then
      continue
    fi
    chmod 755 "$abs"
    log "Restored executable bit for ${rel}"
  done
}

rebuild_generated_indices() {
  run_repo_shell \
    "Rebuilding skill-selector index" \
    "set -euo pipefail; python3 .github/skills/skill-selector/scripts/build_skill_index.py --rebuild-index >/dev/null"
}

verify_cache_directories() {
  source_verify_env

  mkdir -p "$PROJECT_ROOT/.cache/portable-transfer-smoke"
  : >"$PROJECT_ROOT/.cache/portable-transfer-smoke/write-test.txt"

  if [[ -n "${UV_CACHE_DIR:-}" ]]; then
    mkdir -p "$UV_CACHE_DIR"
    : >"$UV_CACHE_DIR/.portable-write-test"
  fi

  if [[ -n "${RUFF_CACHE_DIR:-}" ]]; then
    mkdir -p "$RUFF_CACHE_DIR"
    : >"$RUFF_CACHE_DIR/.portable-write-test"
  fi
}

run_install_chain() {
  local started_at

  started_at="$(now_epoch)"
  log "Milestone 1/2: restore and dependency setup phase started"

  install_minimal_packages
  ensure_uv_available

  run_repo_script \
    "Installing verification toolchain" \
    ./scripts/verification-tools/install-verification-tools.sh --install --profile research --strict

  run_repo_script \
    "Installing pentest tool root" \
    ./scripts/penetration/install_pentest_tools.sh

  source_verify_env

  restore_python_venv

  if have codex; then
    run_repo_script "Installing Deep Wiki surfaces" ./scripts/install_deep_wiki.sh
  else
    run_repo_script \
      "Installing Deep Wiki repo-local surfaces without Codex registration" \
      ./scripts/install_deep_wiki.sh --skip-codex-install
  fi

  if have codex; then
    run_repo_script "Installing llm-wiki Codex marketplace" ./scripts/install_nvk_llm_wiki.sh
  else
    log "codex CLI not found; skipping optional llm-wiki Codex marketplace install"
  fi

  run_repo_script \
    "Installing Understand Anything surfaces" \
    ./scripts/install_understand_anything.sh --install-pnpm

  run_repo_script "Building workspace crates" ./scripts/cargo_build.sh

  install_vscode_extensions
  rebuild_generated_indices
  verify_symlinks
  ensure_gsd_entrypoints_executable
  verify_cache_directories

  run_repo_script "Running repository cleanup" ./scripts/z00z_cleanup.sh --yes

  source_verify_env
  log "Milestone 1/2 completed in $(format_elapsed_since "$started_at")"
}

run_pentest_restore_hook() {
  local started_at

  started_at="$(now_epoch)"
  if [[ ! -x "$PROJECT_ROOT/scripts/penetration/check_pentest_tools.sh" ]]; then
    warn "Pentest tool check hook skipped because scripts/penetration/check_pentest_tools.sh is missing"
    return 0
  fi

  run_repo_shell \
    "Running pentest tool check hook" \
    "set -euo pipefail; ./scripts/penetration/check_pentest_tools.sh --json >/tmp/z00z-pentest-tool-status.json; cat /tmp/z00z-pentest-tool-status.json"
  log "Pentest tool check hook completed in $(format_elapsed_since "$started_at")"
}

run_final_verification() {
  FULL_VERIFY_GATE_STARTED_AT="$(now_epoch)"
  log "Milestone 2/2: end-to-end verification is starting after $(format_elapsed_since "$RUN_STARTED_AT") of setup"
  log "Duration hint: after the full verify gate starts, the remaining tail includes report generation and GSD runtime checks."

  run_repo_script \
    "Running full verify gate" \
    ./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run

  log "Checkpoint: z00z-full-verify-gate.sh completed in $(format_elapsed_since "$FULL_VERIFY_GATE_STARTED_AT")"

  run_orchestrator_report_project

  run_repo_shell \
    "Checking GSD runtime" \
    "set -euo pipefail; ./.github/gsd-core/bin/gsd-tools.cjs --help >/dev/null; node .github/gsd-core/bin/gsd-tools.cjs --help >/dev/null"

  log "Milestone 2/2 completed in $(format_elapsed_since "$FULL_VERIFY_GATE_STARTED_AT") after the full verify gate began"
}

main() {
  parse_args "$@"
  normalize_paths
  validate_archive
  log "Unpack started"
  if [[ "$ARCHIVE_AUTO_SELECTED" -eq 1 ]]; then
    log "Archive auto-selected from current directory: $ARCHIVE_PATH"
  fi
  emit_duration_hint

  if [[ "$DOCKER_SANDBOX" -eq 1 ]]; then
    run_docker_sandbox
    log "Docker sandbox restore completed; total elapsed $(format_elapsed_since "$RUN_STARTED_AT")"
    return 0
  fi

  ensure_tools
  prompt_destination
  prepare_destination
  start_sudo_session
  extract_archive
  replace_placeholders
  restore_planning_runtime_bundle
  verify_packed_symlink_manifest
  verify_symlinks
  if [[ "$SKIP_FORMAL_VERIFICATION" -eq 1 ]]; then
    run_pentest_restore_hook
    log "Project restore completed without formal verification; total elapsed $(format_elapsed_since "$RUN_STARTED_AT")"
    return 0
  fi
  run_install_chain
  run_final_verification
  log "Project restore completed; total elapsed $(format_elapsed_since "$RUN_STARTED_AT")"
}

main "$@"
