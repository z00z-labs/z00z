#!/usr/bin/env bash

# pack_z00z_project.sh
#
# Purpose:
#   Build a portable workspace archive for this Z00Z checkout without changing
#   the source tree. The resulting tarball is meant to be restored later by
#   unpack_z00z_project.sh on another machine or under another user account.
#
# Default run mode:
#   ./pack_z00z_project.sh
#
#   Expected result:
#   - Creates ./z00z-<YYYY-MM-DD>.tar.gz in the project root.
#   - The date stamp is generated from the local system date at pack start.
#   - Uses a temporary staging directory under /tmp and removes it on exit.
#   - Does not mutate the original repository checkout, even when auxiliary
#     directories are excluded recursively from the archive.
#
# Supported flags:
#   --output <path>
#     Write the archive to a custom path. Relative paths are resolved against
#     the current working directory.
#
#   --without-git
#     Compatibility alias. Git history is always excluded.
#
#   --keep-tmp
#     Keep the /tmp/z00z-pack.* staging directory after completion. Use this
#     only for debugging archive contents or normalization results.
#
#   -h, --help
#     Print the short CLI usage summary and exit.
#
# Example commands:
#   ./pack_z00z_project.sh
#   ./pack_z00z_project.sh --output /tmp/z00z-portable.tar.gz
#   ./pack_z00z_project.sh --output ../exports/z00z-<YYYY-MM-DD>.tar.gz --keep-tmp
#
# What this script packs:
#   - The project working tree, excluding local-only runtime/output/cache areas.
#   - Portable metadata under .portable-transfer/, including:
#       * Python venv reconstruction metadata (pyvenv.cfg, uv freeze snapshot)
#       * VS Code extension list
#       * Toolchain/system snapshot for restore diagnostics
#       * Slim planning-runtime bundle rebuilt from live source references so
#         tests that require specific .planning documents can still run after
#         unpack, without shipping the full source .planning workspace
#       * Placeholder mapping for project-root and user-home path normalization
#       * Exact symlink manifest for restore-time integrity verification
#
# What this script does not pack:
#   - Runtime outputs and helper directories that must not be transferred,
#     including top-level logs/reports/target/.cache/.codeviz/.bg-shell/.venv/.temp/.planning,
#     the nested verification review store at scripts/verification-tools/.reviews,
#     and recursively excluded directories such as outputs/, .cache/, .planning/, logs/, reports/, target/, node_modules/,
#     tools/formal_verification/, heavy `tools/penetration/` payload caches
#     and local tool installs, __pycache__/, and other generated storage areas
#     such as Python bytecode
#
# Path portability:
#   Absolute paths that point at the current project root or current user home
#   are rewritten to placeholders during packing. The unpack script replaces
#   those placeholders with values valid on the target machine.
#
# Important note:
#   This archive is workspace-focused, not image-based. It preserves the files
#   needed to continue work, plus enough metadata for the restore script to
#   rebuild local environments with uv-driven Python restore, instead of
#   shipping raw cache/runtime state.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
PROJECT_NAME="$(basename "$PROJECT_ROOT")"
DEFAULT_ARCHIVE_DATE="$(date +%F)"
DEFAULT_OUTPUT_NAME="z00z-${DEFAULT_ARCHIVE_DATE}.tar.gz"
PORTABLE_ROOT_REL=".portable-transfer"
PLACEHOLDER_PROJECT_ROOT="__Z00Z_PROJECT_ROOT__"
PLACEHOLDER_USER_HOME="__Z00Z_USER_HOME__"
RUN_STARTED_AT="$(date +%s)"
readonly SCRIPT_DIR PROJECT_ROOT PROJECT_NAME DEFAULT_ARCHIVE_DATE DEFAULT_OUTPUT_NAME
readonly PORTABLE_ROOT_REL PLACEHOLDER_PROJECT_ROOT PLACEHOLDER_USER_HOME RUN_STARTED_AT

OUTPUT_PATH="$PROJECT_ROOT/$DEFAULT_OUTPUT_NAME"
KEEP_TMP=0
INCLUDE_GIT=0
TMP_ROOT=""
STAGE_ROOT=""
SOURCE_HOME="${HOME:-$(dirname "$PROJECT_ROOT")}"

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

archive_size_human() {
  du -sh "$1" 2>/dev/null | awk '{print $1}'
}

usage() {
  cat <<EOF
Usage:
  ./pack_z00z_project.sh [--output <archive.tar.gz>] [--without-git] [--keep-tmp]

Creates a portable project archive without mutating the source tree.

Options:
  --output <path>  Archive path. Default: ./$DEFAULT_OUTPUT_NAME
  --without-git    Compatibility alias. Git history is always excluded.
  --keep-tmp       Keep the staging temp directory for debugging.
  -h, --help       Show this help.
EOF
}

log() {
  printf '[pack-z00z][+%s] %s\n' "$(format_elapsed_since "$RUN_STARTED_AT")" "$1"
}

warn() {
  printf '[pack-z00z] WARNING: %s\n' "$1" >&2
}

die() {
  printf '[pack-z00z] ERROR: %s\n' "$1" >&2
  exit 1
}

have() {
  command -v "$1" >/dev/null 2>&1
}

cleanup() {
  if [[ "$KEEP_TMP" -eq 1 ]]; then
    if [[ -n "$TMP_ROOT" && -d "$TMP_ROOT" ]]; then
      log "Keeping temp directory: $TMP_ROOT"
    fi
    return 0
  fi

  if [[ -n "$TMP_ROOT" && -d "$TMP_ROOT" ]]; then
    case "$TMP_ROOT" in
      /tmp/z00z-pack.*)
        rm -rf -- "$TMP_ROOT"
        ;;
      *)
        warn "Refusing to remove unexpected temp directory: $TMP_ROOT"
        ;;
    esac
  fi
}

trap cleanup EXIT

normalize_output_path() {
  case "$OUTPUT_PATH" in
    /*) ;;
    *)
      OUTPUT_PATH="$PWD/$OUTPUT_PATH"
      ;;
  esac
}

ensure_tools() {
  local tool

  for tool in python3 tar mktemp; do
    have "$tool" || die "Required command not found: $tool"
  done
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --output)
        [[ -n "${2:-}" ]] || die "--output requires a value"
        OUTPUT_PATH="$2"
        shift 2
        ;;
      --keep-tmp)
        KEEP_TMP=1
        shift
        ;;
      --with-git)
        die "--with-git is not supported; portable Z00Z archives always exclude .git"
        ;;
      --without-git)
        INCLUDE_GIT=0
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

create_staging_root() {
  local started_at

  started_at="$(now_epoch)"
  TMP_ROOT="$(mktemp -d /tmp/z00z-pack.XXXXXX)"
  STAGE_ROOT="$TMP_ROOT/$PROJECT_NAME"
  mkdir -p "$STAGE_ROOT"
  log "Created staging root at $TMP_ROOT in $(format_elapsed_since "$started_at")"
}

copy_project_tree() {
  local started_at

  started_at="$(now_epoch)"
  log "Copying project tree into staging"

  python3 - "$PROJECT_ROOT" "$STAGE_ROOT" "$OUTPUT_PATH" "$INCLUDE_GIT" <<'PY'
import os
import shutil
import stat
import sys
from pathlib import Path

source_root = Path(sys.argv[1]).resolve()
stage_root = Path(sys.argv[2]).resolve()
output_path = Path(sys.argv[3]).resolve()
include_git = sys.argv[4] == "1"

TOP_LEVEL_EXCLUDES = {
    "logs",
    "reports",
    "target",
    ".cache",
    ".bg-shell",
    ".codeviz",
    ".venv",
    ".temp",
    ".planning",
}

EXACT_DIR_EXCLUDES = {
    "scripts/verification-tools/.reviews",
    "tools/formal_verification",
    "tools/penetration/cache",
    "tools/penetration/cargo",
    "tools/penetration/go",
    "tools/penetration/python/bin",
    "tools/penetration/python/pipx",
    "tools/penetration/python/uv-tools",
    ".agents/.install-backups",
    "tools/formal_verification/.probe-saw-suite",
    "tools/formal_verification/creusot/cache",
    "tools/formal_verification/creusot/config",
    "tools/formal_verification/creusot/data",
    "tools/formal_verification/opam/root/log",
    "tools/formal_verification/opam/root/z00z-verify/.opam-switch/build",
}

RECURSIVE_DIR_NAME_EXCLUDES = {
    "__pycache__",
    "outputs",
    ".cache",
    ".bg-shell",
    ".planning",
    "_build",
    "target",
    "fuzz_target",
    "target_fuzz",
    ".codeviz",
    ".reviews",
    ".venv",
    ".temp",
    ".z00z-storage-redb",
    "logs",
    "reports",
    "node_modules",
}

EXACT_FILE_EXCLUDES = {
    "crates/.understand-anything/fingerprints.error.log",
    "tools/formal_verification/aeneas/src/doc.html",
    "tools/formal_verification/charon/src/doc-ml.html",
    "tools/formal_verification/charon/src/doc-rust.html",
}

DIR_EXCLUDE_EXCEPTIONS = (
    "tools/formal_verification/node/lib/node_modules",
)

def is_relative_to(path: Path, base: Path) -> bool:
    try:
        path.relative_to(base)
    except ValueError:
        return False
    return True

relative_output = None
if is_relative_to(output_path, source_root):
    relative_output = output_path.relative_to(source_root).as_posix()

def should_exclude_dir(rel_dir: Path) -> bool:
    rel_posix = rel_dir.as_posix()
    parts = rel_dir.parts
    if not parts:
        return False
    for exception in DIR_EXCLUDE_EXCEPTIONS:
        if rel_posix == exception or rel_posix.startswith(f"{exception}/"):
            return False
    if not include_git and len(parts) == 1 and parts[0] == ".git":
        return True
    if len(parts) == 1 and parts[0] in TOP_LEVEL_EXCLUDES:
        return True
    if rel_posix in EXACT_DIR_EXCLUDES:
        return True
    if not include_git and parts[-1] == ".git":
        return True
    if parts[-1] in RECURSIVE_DIR_NAME_EXCLUDES:
        return True
    return False

def should_exclude_file(rel_file: Path) -> bool:
    rel_posix = rel_file.as_posix()
    if rel_posix in EXACT_FILE_EXCLUDES:
        return True
    if rel_file.suffix.lower() in {".pyc", ".pyo"}:
        return True
    if relative_output and rel_posix == relative_output:
        return True
    if len(rel_file.parts) == 1 and rel_file.name.startswith("z00z-") and rel_file.name.endswith(".tar.gz"):
        return True
    return False

for root, dirs, files in os.walk(source_root, topdown=True, followlinks=False):
    root_path = Path(root)
    rel_root = root_path.relative_to(source_root)
    if rel_root == Path("."):
        rel_root = Path()

    kept_dirs = []
    for name in dirs:
        rel_dir = rel_root / name if rel_root.parts else Path(name)
        if should_exclude_dir(rel_dir):
            continue
        src_dir = source_root / rel_dir
        dst_dir = stage_root / rel_dir
        if src_dir.is_symlink():
            dst_dir.parent.mkdir(parents=True, exist_ok=True)
            os.symlink(os.readlink(src_dir), dst_dir)
            continue
        kept_dirs.append(name)
    dirs[:] = kept_dirs

    dest_root = stage_root / rel_root if rel_root.parts else stage_root
    dest_root.mkdir(parents=True, exist_ok=True)

    for name in files:
        rel_file = rel_root / name if rel_root.parts else Path(name)
        if should_exclude_file(rel_file):
            continue

        src_path = source_root / rel_file
        dst_path = stage_root / rel_file
        dst_path.parent.mkdir(parents=True, exist_ok=True)

        st = os.lstat(src_path)
        if stat.S_ISLNK(st.st_mode):
            os.symlink(os.readlink(src_path), dst_path)
            continue

        shutil.copy2(src_path, dst_path)
PY
  log "Finished copying project tree in $(format_elapsed_since "$started_at")"
}

collect_planning_runtime_bundle() {
  local started_at
  local portable_dir

  started_at="$(now_epoch)"
  log "Collecting slim planning runtime bundle"

  portable_dir="$STAGE_ROOT/$PORTABLE_ROOT_REL"
  mkdir -p "$portable_dir"

  python3 - "$STAGE_ROOT" "$PROJECT_ROOT" "$portable_dir/planning-runtime" <<'PY'
import json
import re
import shutil
import sys
from pathlib import Path

stage_root = Path(sys.argv[1]).resolve()
source_root = Path(sys.argv[2]).resolve()
bundle_root = Path(sys.argv[3]).resolve()

text_suffixes = {
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
    ".cfg",
}

text_names = {
    "Cargo.toml",
}

pattern = re.compile(r"\.planning/[A-Za-z0-9_./-]+")
referenced = set()
sources_by_ref = {}

def is_text_candidate(path: Path) -> bool:
    if path.name in text_names:
        return True
    if path.suffix.lower() in text_suffixes:
        return True
    if path.suffix:
        return False
    try:
        with path.open("rb") as handle:
            return handle.read(2) == b"#!"
    except OSError:
        return False
    return False

for path in stage_root.rglob("*"):
    if not path.is_file() or not is_text_candidate(path):
        continue
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        continue
    rel_source = path.relative_to(stage_root).as_posix()
    for match in pattern.findall(text):
        rel = match.strip("./")
        if not rel.startswith(".planning/"):
            rel = f".{rel}" if rel.startswith("planning/") else rel
        referenced.add(rel)
        sources_by_ref.setdefault(rel, []).append(rel_source)

copied = []
missing = []
for rel in sorted(referenced):
    src = source_root / rel
    if not src.is_file():
        missing.append({"path": rel, "referenced_from": sorted(sources_by_ref.get(rel, []))})
        continue
    dst = bundle_root / rel
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)
    copied.append(rel)

manifest = {
    "schema_version": 1,
    "copied_files": copied,
    "missing_files": missing,
}
manifest_path = bundle_root / "manifest.json"
manifest_path.parent.mkdir(parents=True, exist_ok=True)
manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
  read -r planning_copied planning_missing < <(
    python3 - "$portable_dir/planning-runtime/manifest.json" <<'PY'
import json
import sys
from pathlib import Path

manifest = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(len(manifest.get("copied_files", [])), len(manifest.get("missing_files", [])))
PY
  )

  log "Collected planning runtime bundle in $(format_elapsed_since "$started_at"): ${planning_copied:-0} files copied, ${planning_missing:-0} references missing from source"
}

collect_portable_metadata() {
  local started_at
  local portable_dir
  local system_info_path
  local vscode_dir
  local venv_dir
  local active_toolchain
  local cargo_version
  local rustc_version
  local node_version
  local npm_version
  local python_version
  local code_version
  local vscode_cli
  local freeze_path
  local pyvenv_path
  local extensions_path

  started_at="$(now_epoch)"
  log "Collecting portable metadata"

  portable_dir="$STAGE_ROOT/$PORTABLE_ROOT_REL"
  system_info_path="$portable_dir/system-info.txt"
  vscode_dir="$portable_dir/vscode"
  venv_dir="$portable_dir/python-venv"
  freeze_path="$venv_dir/pip-freeze.txt"
  pyvenv_path="$venv_dir/pyvenv.cfg"
  extensions_path="$vscode_dir/extensions.txt"

  mkdir -p "$portable_dir" "$vscode_dir" "$venv_dir"

  active_toolchain="$(rustup show active-toolchain 2>/dev/null || true)"
  cargo_version="$(cargo --version 2>/dev/null || true)"
  rustc_version="$(rustc --version 2>/dev/null || true)"
  node_version="$(node --version 2>/dev/null || true)"
  npm_version="$(npm --version 2>/dev/null || true)"
  python_version="$(python3 --version 2>/dev/null || true)"
  vscode_cli=""
  if have code; then
    vscode_cli="code"
  elif have codium; then
    vscode_cli="codium"
  fi
  code_version=""
  if [[ -n "$vscode_cli" ]]; then
    code_version="$("$vscode_cli" --version 2>/dev/null | head -n 1 || true)"
  fi

  {
    printf 'packed_at_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf 'project_name=%s\n' "$PROJECT_NAME"
    printf 'archive_name=%s\n' "$(basename "$OUTPUT_PATH")"
    printf 'active_toolchain=%s\n' "$active_toolchain"
    printf 'cargo_version=%s\n' "$cargo_version"
    printf 'rustc_version=%s\n' "$rustc_version"
    printf 'node_version=%s\n' "$node_version"
    printf 'npm_version=%s\n' "$npm_version"
    printf 'python3_version=%s\n' "$python_version"
    printf 'code_version=%s\n' "$code_version"
    printf 'codex_version=%s\n' "$(codex --version 2>/dev/null || true)"
  } >"$system_info_path"

  if [[ -n "$vscode_cli" ]]; then
    "$vscode_cli" --list-extensions --show-versions >"$extensions_path" 2>/dev/null || true
  else
    : >"$extensions_path"
  fi

  if [[ -f "$PROJECT_ROOT/.venv/pyvenv.cfg" ]]; then
    cp "$PROJECT_ROOT/.venv/pyvenv.cfg" "$pyvenv_path"
  else
    : >"$pyvenv_path"
  fi

  if [[ -x "$PROJECT_ROOT/.venv/bin/python" ]]; then
    if have uv; then
      uv pip freeze --python "$PROJECT_ROOT/.venv/bin/python" >"$freeze_path" 2>/dev/null \
        || "$PROJECT_ROOT/.venv/bin/python" -m pip freeze --all >"$freeze_path" 2>/dev/null \
        || true
    else
      "$PROJECT_ROOT/.venv/bin/python" -m pip freeze --all >"$freeze_path" 2>/dev/null || true
    fi
  else
    : >"$freeze_path"
  fi

  python3 - "$portable_dir/manifest.json" <<'PY'
import json
import os
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
portable_dir = manifest_path.parent
pyvenv_cfg = portable_dir / "python-venv" / "pyvenv.cfg"
freeze_file = portable_dir / "python-venv" / "pip-freeze.txt"

def read_pyvenv_value(key: str) -> str:
    if not pyvenv_cfg.exists():
        return ""
    for line in pyvenv_cfg.read_text(encoding="utf-8").splitlines():
        if line.startswith(f"{key} = "):
            return line.split(" = ", 1)[1].strip()
    return ""

full_version = read_pyvenv_value("version_info")
minor_version = ".".join(full_version.split(".")[:2]) if full_version else ""

manifest = {
    "schema_version": 1,
    "project_name": os.environ["PROJECT_NAME"],
    "archive_name": os.environ["ARCHIVE_NAME"],
    "packed_at_utc": os.environ["PACKED_AT_UTC"],
    "include_git": os.environ["INCLUDE_GIT"] == "1",
    "placeholders": {
        "project_root": os.environ["PLACEHOLDER_PROJECT_ROOT"],
        "user_home": os.environ["PLACEHOLDER_USER_HOME"],
    },
    "excluded_paths": {
        "top_level": [name for name in [
            ".git" if os.environ["INCLUDE_GIT"] != "1" else "",
            "logs",
            "reports",
            "target",
            ".cache",
            ".bg-shell",
            ".codeviz",
            ".venv",
            ".temp",
            ".planning",
        ] if name],
        "recursive_dir_names": [name for name in [
            ".git" if os.environ["INCLUDE_GIT"] != "1" else "",
            "__pycache__",
            "outputs",
            ".cache",
            ".bg-shell",
            ".planning",
            "target",
            "fuzz_target",
            "target_fuzz",
            ".codeviz",
            ".reviews",
            ".venv",
            ".temp",
            ".z00z-storage-redb",
            "logs",
            "reports",
            "node_modules",
        ] if name],
        "generated_file_suffixes": [
            ".pyc",
            ".pyo",
        ],
        "exact_dirs": [
            "scripts/verification-tools/.reviews",
            "tools/formal_verification",
            "tools/penetration/cache",
            "tools/penetration/cargo",
            "tools/penetration/go",
            "tools/penetration/python/bin",
            "tools/penetration/python/pipx",
            "tools/penetration/python/uv-tools",
            ".agents/.install-backups",
            "tools/formal_verification/.probe-saw-suite",
            "tools/formal_verification/creusot/cache",
            "tools/formal_verification/creusot/config",
            "tools/formal_verification/creusot/data",
        ],
    },
    "restore_steps": [
        "scripts/verification-tools/install-verification-tools.sh --profile research --strict",
        "scripts/install_py_venv.sh",
        "scripts/install_deep_wiki.sh",
        "scripts/install_nvk_llm_wiki.sh",
        "scripts/install_understand_anything.sh --install-pnpm",
        "scripts/cargo_build.sh",
        "source scripts/verify-env.sh",
        "scripts/z00z_cleanup.sh --yes",
        ".github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run",
        ".github/skills/z00z-verification-orchestrator/scripts/orchestrate.sh report project",
    ],
    "python_venv": {
        "present": pyvenv_cfg.exists(),
        "minor_version": minor_version,
        "full_version": full_version,
        "pyvenv_cfg": "python-venv/pyvenv.cfg",
        "pip_freeze": "python-venv/pip-freeze.txt",
        "freeze_nonempty": freeze_file.exists() and bool(freeze_file.read_text(encoding="utf-8").strip()),
    },
    "vscode": {
        "extensions_file": "vscode/extensions.txt",
    },
    "planning_runtime": {
        "bundle_root": "planning-runtime",
        "manifest": "planning-runtime/manifest.json",
    },
}

manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
  log "Portable metadata collected in $(format_elapsed_since "$started_at")"
}

normalize_staging_tree() {
  local started_at

  started_at="$(now_epoch)"
  log "Normalizing path-sensitive content in staging"

  python3 - "$STAGE_ROOT" "$PROJECT_ROOT" "$SOURCE_HOME" "$PORTABLE_ROOT_REL" <<'PY'
import json
import os
import re
import shutil
import sys
from pathlib import Path

stage_root = Path(sys.argv[1]).resolve()
source_root = Path(sys.argv[2]).resolve()
source_home = Path(sys.argv[3]).resolve()
portable_root_rel = sys.argv[4]

placeholder_project_root = os.environ["PLACEHOLDER_PROJECT_ROOT"]
placeholder_user_home = os.environ["PLACEHOLDER_USER_HOME"]

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

rewritten_files = []
rewritten_symlinks = []
external_symlinks = []
portable_text_roots = (
    stage_root / "tools/formal_verification/opam/root",
)
# Portable archives must not vendor user-home-managed toolchains such as
# ~/.cargo or ~/.rustup; the restore flow recreates them from install scripts.
home_symlink_mappings = {}

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

def rewrite_text_file(path: Path) -> None:
    if not is_text_candidate(path):
        return
    try:
        content = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return

    updated = content.replace(str(source_root), placeholder_project_root)
    updated = updated.replace(str(source_home), placeholder_user_home)
    if updated != content:
        path.write_text(updated, encoding="utf-8")
        rewritten_files.append(path.relative_to(stage_root).as_posix())

def stage_external_dependency(source_target: Path, staged_target: Path) -> bool:
    if staged_target.exists() or staged_target.is_symlink():
        return True
    if not source_target.exists() and not source_target.is_symlink():
        return False

    staged_target.parent.mkdir(parents=True, exist_ok=True)
    if source_target.is_symlink():
        os.symlink(os.readlink(source_target), staged_target)
        return True
    if source_target.is_dir():
        shutil.copytree(source_target, staged_target, symlinks=True, dirs_exist_ok=True)
        return True

    shutil.copy2(source_target, staged_target)
    return True

for path in stage_root.rglob("*"):
    if path.is_symlink():
        target = os.readlink(path)
        rewritten_target = None

        if target.startswith(str(source_root)):
            rel_target_path = stage_root / Path(target).relative_to(source_root)
            rewritten_target = os.path.relpath(rel_target_path, path.parent)
        elif target.startswith(str(source_home)):
            for source_prefix, staged_prefix in home_symlink_mappings.items():
                if not target.startswith(source_prefix):
                    continue
                staged_target = staged_prefix / Path(target).relative_to(source_prefix)
                if not staged_target.exists() and not staged_target.is_symlink():
                    if not stage_external_dependency(Path(target), staged_target):
                        continue
                if not staged_target.exists() and not staged_target.is_symlink():
                    continue
                rewritten_target = os.path.relpath(staged_target, path.parent)
                break

            if rewritten_target is None:
                external_symlinks.append({
                    "path": path.relative_to(stage_root).as_posix(),
                    "target": target,
                })

        if rewritten_target is not None:
            path.unlink()
            os.symlink(rewritten_target, path)
            rewritten_symlinks.append({
                "path": path.relative_to(stage_root).as_posix(),
                "target": rewritten_target,
            })
        continue

    if path.is_file():
        rewrite_text_file(path)

state_path = stage_root / portable_root_rel / "normalization-state.json"
state = {
    "schema_version": 1,
    "rewritten_files": rewritten_files,
    "rewritten_symlinks": rewritten_symlinks,
    "external_symlinks": external_symlinks,
}
state_path.write_text(json.dumps(state, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
  log "Finished normalizing staging tree in $(format_elapsed_since "$started_at")"
}

write_symlink_manifest() {
  local started_at

  started_at="$(now_epoch)"
  log "Recording exact symlink targets for restore verification"

  python3 - "$STAGE_ROOT" "$PORTABLE_ROOT_REL/symlink-manifest.json" <<'PY'
import json
import os
import sys
from pathlib import Path

stage_root = Path(sys.argv[1]).resolve()
manifest_rel = Path(sys.argv[2])
manifest_path = stage_root / manifest_rel
entries = []

for path in sorted(stage_root.rglob("*")):
    if not path.is_symlink():
        continue
    entries.append(
        {
            "path": path.relative_to(stage_root).as_posix(),
            "target": os.readlink(path),
        }
    )

manifest = {
    "schema_version": 1,
    "entries": entries,
}
manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

  log "Recorded symlink manifest in $(format_elapsed_since "$started_at")"
}

create_archive() {
  local started_at
  local output_dir

  started_at="$(now_epoch)"
  output_dir="$(dirname "$OUTPUT_PATH")"
  mkdir -p "$output_dir"

  log "Creating archive $OUTPUT_PATH"
  tar -C "$TMP_ROOT" -czf "$OUTPUT_PATH" "$PROJECT_NAME"
  log "Archive ready: $OUTPUT_PATH (size $(archive_size_human "$OUTPUT_PATH"), step $(format_elapsed_since "$started_at"))"
}

main() {
  parse_args "$@"
  normalize_output_path
  ensure_tools
  log "Pack started"
  log "Duration hint: large workspaces spend most of the time in file copy and archive creation. Total elapsed time is printed at the end."
  log "Git history packing is disabled"
  create_staging_root

  copy_project_tree

  export PROJECT_NAME
  ARCHIVE_NAME="$(basename "$OUTPUT_PATH")"
  PACKED_AT_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  export ARCHIVE_NAME PACKED_AT_UTC INCLUDE_GIT
  export PLACEHOLDER_PROJECT_ROOT
  export PLACEHOLDER_USER_HOME

  collect_planning_runtime_bundle
  collect_portable_metadata
  normalize_staging_tree
  write_symlink_manifest
  create_archive
  log "Pack completed; total elapsed $(format_elapsed_since "$RUN_STARTED_AT")"
}

main "$@"
