#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/penetration/common.sh
source "$ROOT/scripts/penetration/common.sh"

MODE="standard"
SCOPE_PATH=".security/scope.yaml"
NO_DAST=0
STATIC_ONLY=0
CHECK_ONLY=0
PROFILE="generic"
ARTIFACT_DIR=""
REPORT_DIR=""
DOCKER_SANDBOX=0
ARCHIVE_PATH=""
FORCE_PACK=0
DOCKER_IMAGE="${Z00Z_PENTEST_DOCKER_IMAGE:-python:3.12-slim}"

usage() {
  cat <<'EOF'
Usage: ./z00z_penetration_tests.sh [--mode quick|standard|deep] [--scope <path>] [--static-only] [--no-dast] [--profile generic|z00z] [--artifact-dir <path>] [--check-only] [--docker-sandbox [--archive <path>] [--pack] [--report-dir <path>] [--docker-image <image>]]

Modes:
  Local mode runs scripts/penetration/run_local_pentest.sh directly.
  Docker mode packs or reuses an archive, extracts it inside a container, and
  runs the same pentest entrypoint against the extracted workspace only. The
  live checkout is never scanned directly through the Docker path.
EOF
}

resolve_repo_path() {
  local raw_path="$1"
  if [[ "$raw_path" = /* ]]; then
    printf '%s\n' "$raw_path"
    return
  fi
  printf '%s\n' "$ROOT/$raw_path"
}

resolve_any_path() {
  local raw_path="$1"
  if [[ "$raw_path" = /* ]]; then
    printf '%s\n' "$raw_path"
    return
  fi
  printf '%s\n' "$PWD/$raw_path"
}

scope_path_for_archive() {
  python3 - "$ROOT" "$SCOPE_PATH" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1]).resolve()
scope = Path(sys.argv[2])

if not scope.is_absolute():
    print(scope.as_posix())
    raise SystemExit(0)

scope = scope.resolve()
try:
    print(scope.relative_to(root).as_posix())
except ValueError as exc:
    raise SystemExit(
        f"ERROR: docker pentest scope must stay inside the repository root: {scope}"
    ) from exc
PY
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      MODE="$2"
      shift 2
      ;;
    --scope)
      SCOPE_PATH="$(resolve_repo_path "$2")"
      shift 2
      ;;
    --static-only)
      STATIC_ONLY=1
      shift
      ;;
    --no-dast)
      NO_DAST=1
      shift
      ;;
    --profile)
      PROFILE="$2"
      shift 2
      ;;
    --artifact-dir)
      ARTIFACT_DIR="$(resolve_repo_path "$2")"
      shift 2
      ;;
    --report-dir)
      REPORT_DIR="$(resolve_repo_path "$2")"
      shift 2
      ;;
    --check-only)
      CHECK_ONLY=1
      shift
      ;;
    --docker-sandbox)
      DOCKER_SANDBOX=1
      shift
      ;;
    --archive)
      ARCHIVE_PATH="$(resolve_any_path "$2")"
      shift 2
      ;;
    --pack)
      FORCE_PACK=1
      shift
      ;;
    --docker-image)
      DOCKER_IMAGE="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

case "$MODE" in
  quick|standard|deep) ;;
  *)
    echo "ERROR: unsupported mode: $MODE" >&2
    exit 1
    ;;
esac

case "$PROFILE" in
  generic|z00z) ;;
  *)
    echo "ERROR: unsupported profile: $PROFILE" >&2
    exit 1
    ;;
esac

if [[ "$DOCKER_SANDBOX" -eq 0 ]]; then
  if [[ -n "$ARCHIVE_PATH" || "$FORCE_PACK" -eq 1 || -n "$REPORT_DIR" ]]; then
    echo "ERROR: --archive, --pack, and --report-dir require --docker-sandbox" >&2
    exit 1
  fi

  local_args=(--mode "$MODE" --scope "$SCOPE_PATH" --profile "$PROFILE")
  if [[ -n "$ARTIFACT_DIR" ]]; then
    local_args+=(--artifact-dir "$ARTIFACT_DIR")
  fi
  if [[ "$STATIC_ONLY" -eq 1 ]]; then
    local_args+=(--static-only)
  fi
  if [[ "$NO_DAST" -eq 1 ]]; then
    local_args+=(--no-dast)
  fi
  if [[ "$CHECK_ONLY" -eq 1 ]]; then
    local_args+=(--check-only)
  fi

  exec bash "$ROOT/scripts/penetration/run_local_pentest.sh" "${local_args[@]}"
fi

if [[ -n "$ARCHIVE_PATH" && "$FORCE_PACK" -eq 1 ]]; then
  echo "ERROR: --archive and --pack cannot be used together" >&2
  exit 1
fi

if [[ -z "$ARCHIVE_PATH" || "$FORCE_PACK" -eq 1 ]]; then
  archive_run_id="$(pen_now_run_id)"
  ARCHIVE_PATH="/tmp/z00z-pentest-${archive_run_id}.tar.gz"
  bash "$ROOT/pack_z00z_project.sh" --output "$ARCHIVE_PATH"
fi

docker_args=(
  --archive "$ARCHIVE_PATH"
  --mode "$MODE"
  --scope "$(scope_path_for_archive)"
  --profile "$PROFILE"
  --docker-image "$DOCKER_IMAGE"
)
if [[ -n "$ARTIFACT_DIR" ]]; then
  docker_args+=(--artifact-dir "$ARTIFACT_DIR")
fi
if [[ -n "$REPORT_DIR" ]]; then
  docker_args+=(--report-dir "$REPORT_DIR")
fi
if [[ "$STATIC_ONLY" -eq 1 ]]; then
  docker_args+=(--static-only)
fi
if [[ "$NO_DAST" -eq 1 ]]; then
  docker_args+=(--no-dast)
fi
if [[ "$CHECK_ONLY" -eq 1 ]]; then
  docker_args+=(--check-only)
fi

exec bash "$ROOT/tools/penetration/docker/run_pentest_container.sh" "${docker_args[@]}"
