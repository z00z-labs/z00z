#!/bin/bash

# Install and test the recommended Z00Z continuous verification toolchain.
# ./install-verification-tools.sh --self-test --profile all --strict

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
# shellcheck source=scripts/target-layout.sh
source "$ROOT_DIR/scripts/target-layout.sh"
TOOLS_DIR="${Z00Z_VERIFY_TOOLS_DIR:-$ROOT_DIR/tools/formal_verification}"
TOOLS_BIN_DIR="$TOOLS_DIR/bin"
LOCAL_CARGO_HOME="${Z00Z_VERIFY_CARGO_HOME:-$TOOLS_DIR/cargo}"
LOCAL_RUSTUP_HOME="${Z00Z_RUSTUP_HOME:-$TOOLS_DIR/rustup}"
LOCAL_PYTHON_PREFIX="${Z00Z_VERIFY_PYTHON_PREFIX:-$TOOLS_DIR/python}"
LOCAL_NODE_PREFIX="${Z00Z_VERIFY_NODE_PREFIX:-$TOOLS_DIR/node}"
NODE_DIST_CHANNEL="${Z00Z_NODE_DIST_CHANNEL:-latest-v22.x}"
LOCAL_OPAM_ROOT="${Z00Z_OPAM_ROOT:-$TOOLS_DIR/opam/root}"
LOCAL_OPAM_SWITCH="${Z00Z_VERIFY_OPAM_SWITCH:-z00z-verify}"
LOCAL_AENEAS_OPAM_SWITCH="${Z00Z_AENEAS_OPAM_SWITCH:-z00z-aeneas}"
LOCAL_KANI_HOME="${Z00Z_KANI_HOME:-$TOOLS_DIR/kani}"
LOCAL_CREUSOT_DATA_HOME="${Z00Z_CREUSOT_DATA_HOME:-$TOOLS_DIR/creusot/data}"
LOCAL_CREUSOT_CONFIG_HOME="${Z00Z_CREUSOT_CONFIG_HOME:-$TOOLS_DIR/creusot/config}"
LOCAL_CREUSOT_CACHE_HOME="${Z00Z_CREUSOT_CACHE_HOME:-$TOOLS_DIR/creusot/cache}"
LOCAL_RUFF_CACHE_DIR="${Z00Z_RUFF_CACHE_DIR:-$TOOLS_DIR/python/ruff-cache}"
LOCAL_MIRI_SYSROOT="${Z00Z_MIRI_SYSROOT:-$TOOLS_DIR/miri/sysroot}"
CREUSOT_TARGET_DIR="$(z00z_tool_target_dir "$ROOT_DIR" creusot)"
MIR_JSON_TARGET_DIR="$(z00z_tool_target_dir "$ROOT_DIR" mir-json)"
CHARON_TARGET_DIR="$(z00z_tool_target_dir "$ROOT_DIR" charon)"
HAX_TARGET_DIR="$(z00z_hax_target_dir "$ROOT_DIR")"
TARGET_ROOT_DIR="$(z00z_target_root "$ROOT_DIR")"
VERSIONS_ENV_SOURCE="$ROOT_DIR/scripts/verification-tools/versions.env"
SHA256SUMS_SOURCE="$ROOT_DIR/scripts/verification-tools/SHA256SUMS"
PROFILE="all"
ACTION="install"
SKIP_SYSTEM=0
SKIP_NODE=0
SKIP_OPAM=0
SKIP_HEAVY=0
FORCE_UPGRADE=0
STRICT=0
TLA_RELEASE="${Z00Z_TLA_RELEASE:-v1.7.4}"
ALLOY_RELEASE="${Z00Z_ALLOY_RELEASE:-v6.2.0}"
APALACHE_RELEASE="${Z00Z_APALACHE_RELEASE:-v0.58.0}"
VERUS_RELEASE="${Z00Z_VERUS_RELEASE:-release/0.2026.06.14.4ea7d0f}"
PRUSTI_RELEASE="${Z00Z_PRUSTI_RELEASE:-v-2023-08-22-1715}"
TAMARIN_RELEASE="${Z00Z_TAMARIN_RELEASE:-1.12.0}"
MAUDE_RELEASE="${Z00Z_MAUDE_RELEASE:-Maude3.5.1}"
SAW_RELEASE="${Z00Z_SAW_RELEASE:-v1.5.1}"
CRYPTOL_RELEASE="${Z00Z_CRYPTOL_RELEASE:-3.5.0}"
SAW_SUITE_IMAGE="${Z00Z_SAW_SUITE_IMAGE:-ghcr.io/galoisinc/saw-suite@sha256:aabdbf3442fffe35dc56cabf8ddd1d473d291df8226f5cf018009a94cfc4151f}"
RIPGREP_RELEASE="${Z00Z_RIPGREP_RELEASE:-15.1.0}"
CVC5_RELEASE="${Z00Z_CVC5_RELEASE:-cvc5-1.3.4}"
BITWUZLA_RELEASE="${Z00Z_BITWUZLA_RELEASE:-0.9.1}"
MIR_JSON_REF="${Z00Z_MIR_JSON_REF:-03bcbb1d07be8489e5148fe5eb58da8f9452a100}"
MIR_JSON_TOOLCHAIN="${Z00Z_MIR_JSON_TOOLCHAIN:-nightly-2025-09-14}"
MIR_JSON_SYSROOT_DUP_CRATES="${Z00Z_MIR_JSON_SYSROOT_DUP_CRATES:-getopts}"
CHARON_REF="${Z00Z_CHARON_REF:-e9b10cc37af3d1cd20a4f62cceca7331a70a7522}"
AENEAS_REF="${Z00Z_AENEAS_REF:-8dd8bfb3047ce9797fa08d8046d8410a3b6a21c4}"
PYTHON_TOOL_RUFF="${Z00Z_PYTHON_TOOL_RUFF:-ruff==0.15.17}"
PYTHON_TOOL_UV="${Z00Z_PYTHON_TOOL_UV:-uv}"
OPAM_COMPILER="${Z00Z_OPAM_COMPILER:-ocaml-base-compiler.5.1.1}"
RUSTUP_TOOLCHAIN_NAME="${Z00Z_RUSTUP_TOOLCHAIN:-stable}"

if [[ -f "$VERSIONS_ENV_SOURCE" ]]; then
  # shellcheck source=/dev/null
  source "$VERSIONS_ENV_SOURCE"
fi

export CARGO_HOME="$LOCAL_CARGO_HOME"
export RUSTUP_HOME="$LOCAL_RUSTUP_HOME"
export RUSTUP_TOOLCHAIN="$RUSTUP_TOOLCHAIN_NAME"
export RUFF_CACHE_DIR="$LOCAL_RUFF_CACHE_DIR"
export MIRI_SYSROOT="$LOCAL_MIRI_SYSROOT"
mkdir -p "$RUFF_CACHE_DIR"

usage() {
  cat <<'EOF'
Usage: install-verification-tools.sh [OPTIONS]

Options:
  --install          Install tools (default).
  --check            Only print installed/missing status.
  --self-test        Run version checks and tiny internal smoke tests.
  --profile <name>   core, recommended, all, research, or deep. Default: all.
  --skip-system      Skip apt/pacman system package installation.
  --skip-node        Skip npm markdownlint-cli2 installation.
  --skip-opam        Skip OPAM protocol/proof tools.
  --skip-heavy       Skip Tamarin, Verus, Prusti, Creusot, EasyCrypt, hax, dudect.
  --upgrade          Reinstall already-present cargo tools with cargo install --force.
  --strict           Make --check/--self-test fail when profile-required tools are missing.
  -h, --help         Show this help.

Profiles:
  core          Rust fast gate, fuzzing, audit, docs, TLA+/Apalache/Alloy jars.
  recommended  Core plus ProVerif, Why3, Tamarin, Verus, Prusti, dudect.
  all          Recommended plus EasyCrypt, Creusot, hax checkout.
  research     All plus SAW, Cryptol, mir-json/Crux-MIR, Charon, Aeneas, cvc5, Bitwuzla, rg, local ruff.
  deep         Alias for research.

Pin release downloads with env vars:
  scripts/verification-tools/versions.env is the repository source of truth.
  Override with environment variables only when intentionally testing another pinned version.

Install layout:
  Managed verifier tools are installed under:
    tools/formal_verification/
  including:
    cargo/    local cargo home and Rust tool binaries
    node/     local npm prefix for markdownlint-cli2
    opam/     local OPAM root and wrappers
    kani/     local Kani bundle home
    python/   local Python tool envs and binaries
    bin/      repository-local wrappers/symlinks used by Z00Z verifier gates

Uninstall:
  Review current inventory with:
    ./scripts/install-verification-tools.sh --check --profile all --strict
  Remove the repository-local verifier toolchain with:
    gio trash ./tools/formal_verification
  If research profile pulled pinned SAW container images, optionally remove them too with:
    docker image rm "$Z00Z_SAW_SUITE_IMAGE"
  Legacy cleanup for older global user-space installs created by previous revisions:
    cargo uninstall cargo-nextest cargo-audit cargo-deny cargo-vet cargo-fuzz cargo-geiger cargo-llvm-cov cargo-semver-checks just bacon watchexec-cli mdbook lychee taplo-cli kani-verifier cargo-creusot cargo-hax
    npm uninstall --global markdownlint-cli2
    opam switch remove z00z-verify
    gio trash ~/.kani
    gio trash ~/.local/bin/proverif ~/.local/bin/why3 ~/.local/bin/easycrypt ~/.local/bin/tamarin-prover ~/.local/bin/verus ~/.local/bin/cargo-verus ~/.local/bin/alloy-headless-z00z
  System packages like java, opam, z3, node, npm, jq, and shellcheck are prerequisites
  and remain system-managed unless you explicitly remove them with your package manager.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --install)
      ACTION="install"
      shift
      ;;
    --check)
      ACTION="check"
      shift
      ;;
    --self-test)
      ACTION="self-test"
      shift
      ;;
    --profile)
      PROFILE="${2:-}"
      shift 2
      ;;
    --skip-system)
      SKIP_SYSTEM=1
      shift
      ;;
    --skip-node)
      SKIP_NODE=1
      shift
      ;;
    --skip-opam)
      SKIP_OPAM=1
      shift
      ;;
    --skip-heavy)
      SKIP_HEAVY=1
      shift
      ;;
    --upgrade)
      FORCE_UPGRADE=1
      shift
      ;;
    --strict)
      STRICT=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

case "$PROFILE" in
  core|recommended|all|research|deep)
    if [[ "$PROFILE" == "deep" ]]; then
      PROFILE="research"
    fi
    ;;
  *)
    echo "ERROR: invalid profile: $PROFILE" >&2
    exit 1
    ;;
esac

log() {
  printf '[z00z-tools] %s\n' "$1" >&2
}

warn() {
  printf '[z00z-tools] WARNING: %s\n' "$1" >&2
}

have() {
  command -v "$1" >/dev/null 2>&1
}

profile_at_least_recommended() {
  [[ "$PROFILE" != "core" ]]
}

profile_includes_all() {
  [[ "$PROFILE" == "all" || "$PROFILE" == "research" ]]
}

profile_includes_research() {
  [[ "$PROFILE" == "research" ]]
}

activate_local_tool_env() {
  mkdir -p "$TOOLS_BIN_DIR" "$LOCAL_CARGO_HOME/bin" "$LOCAL_RUSTUP_HOME" "$LOCAL_PYTHON_PREFIX/bin" "$LOCAL_NODE_PREFIX/bin" "$TOOLS_DIR/opam/bin"

  local path_dirs=(
    "$TOOLS_BIN_DIR"
    "$TOOLS_DIR/saw-suite/bin"
    "$LOCAL_CARGO_HOME/bin"
    "$LOCAL_PYTHON_PREFIX/bin"
    "$LOCAL_NODE_PREFIX/bin"
    "$TOOLS_DIR/opam/bin"
    "$TOOLS_DIR/prusti/bin"
    "$TOOLS_DIR/verus/bin"
    "$TOOLS_DIR/tamarin/bin"
    "$TOOLS_DIR/maude/bin"
    "$TOOLS_DIR/apalache/bin"
    "$TOOLS_DIR/alloy/bin"
    "$TOOLS_DIR/saw/bin"
    "$TOOLS_DIR/cryptol/bin"
    "$TOOLS_DIR/cvc5/bin"
    "$TOOLS_DIR/bitwuzla/bin"
    "$TOOLS_DIR/mir-json/bin"
    "$TOOLS_DIR/charon/bin"
    "$TOOLS_DIR/aeneas/bin"
    "$TOOLS_DIR/rg/bin"
  )

  local idx dir
  for ((idx=${#path_dirs[@]} - 1; idx >= 0; idx--)); do
    dir="${path_dirs[$idx]}"
    if [[ -d "$dir" ]]; then
      PATH="$dir:$PATH"
    fi
  done

  export PATH
  export KANI_HOME="$LOCAL_KANI_HOME"
  export NPM_CONFIG_PREFIX="$LOCAL_NODE_PREFIX"
  export MIRI_SYSROOT="$LOCAL_MIRI_SYSROOT"
  export CARGO_HTTP_MULTIPLEXING="${Z00Z_CARGO_HTTP_MULTIPLEXING:-false}"
  export CARGO_NET_RETRY="${Z00Z_CARGO_NET_RETRY:-10}"
  export CARGO_REGISTRIES_CRATES_IO_PROTOCOL="${Z00Z_CARGO_REGISTRIES_CRATES_IO_PROTOCOL:-sparse}"

  if [[ -d "$TOOLS_DIR/saw-suite/rlibs" ]]; then
    export CRUX_RUST_LIBRARY_PATH="${CRUX_RUST_LIBRARY_PATH:-$TOOLS_DIR/saw-suite/rlibs}"
    export SAW_RUST_LIBRARY_PATH="${SAW_RUST_LIBRARY_PATH:-$TOOLS_DIR/saw-suite/rlibs}"
  elif [[ -d "$TOOLS_DIR/mir-json/rlibs" ]]; then
    export CRUX_RUST_LIBRARY_PATH="${CRUX_RUST_LIBRARY_PATH:-$TOOLS_DIR/mir-json/rlibs}"
    export SAW_RUST_LIBRARY_PATH="${SAW_RUST_LIBRARY_PATH:-$TOOLS_DIR/mir-json/rlibs}"
  fi

  if [[ -x "$TOOLS_DIR/saw-suite/bin/crux-mir" ]]; then
    export CRUX_MIR="${CRUX_MIR:-$TOOLS_DIR/saw-suite/bin/crux-mir}"
  elif [[ -x "$TOOLS_DIR/saw-suite/bin/crux-mir-comp" ]]; then
    export CRUX_MIR="${CRUX_MIR:-$TOOLS_DIR/saw-suite/bin/crux-mir-comp}"
  fi
}

stage_repo_config_files() {
  mkdir -p "$TOOLS_DIR"
  if [[ -f "$VERSIONS_ENV_SOURCE" ]]; then
    cp "$VERSIONS_ENV_SOURCE" "$TOOLS_DIR/versions.env"
  fi
  if [[ -f "$SHA256SUMS_SOURCE" ]]; then
    cp "$SHA256SUMS_SOURCE" "$TOOLS_DIR/SHA256SUMS"
  fi
}

tool_path() {
  command -v "$1" 2>/dev/null || true
}

managed_tool_candidates() {
  local name="$1"

  cat <<EOF
$TOOLS_BIN_DIR/$name
$TOOLS_DIR/saw-suite/bin/$name
$TOOLS_DIR/saw/bin/$name
$TOOLS_DIR/cryptol/bin/$name
$TOOLS_DIR/cvc5/bin/$name
$TOOLS_DIR/bitwuzla/bin/$name
$TOOLS_DIR/rg/bin/$name
$LOCAL_CARGO_HOME/bin/$name
$LOCAL_PYTHON_PREFIX/bin/$name
$LOCAL_NODE_PREFIX/bin/$name
$TOOLS_DIR/opam/bin/$name
$TOOLS_DIR/prusti/bin/$name
$TOOLS_DIR/verus/bin/$name
$TOOLS_DIR/tamarin/bin/$name
$TOOLS_DIR/maude/bin/$name
$TOOLS_DIR/apalache/bin/$name
$TOOLS_DIR/alloy/bin/$name
$TOOLS_DIR/charon/bin/$name
$TOOLS_DIR/aeneas/bin/$name
EOF
}

managed_tool_exists() {
  local name="$1"
  local candidate=""

  while IFS= read -r candidate; do
    [[ -n "$candidate" ]] || continue
    if [[ -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done < <(managed_tool_candidates "$name")

  return 1
}

tool_path_is_local() {
  local resolved
  resolved="$(tool_path "$1")"
  if [[ -n "$resolved" && "$resolved" == "$TOOLS_DIR/"* ]]; then
    return 0
  fi
  managed_tool_exists "$1" >/dev/null
}

refresh_repo_bin_from_tool() {
  local name="$1"
  local resolved
  resolved="$(tool_path "$name")"
  if [[ -z "$resolved" ]]; then
    resolved="$(managed_tool_exists "$name" || true)"
  fi
  [[ -n "$resolved" ]] || return 0
  if [[ "$resolved" != "$TOOLS_BIN_DIR/$name" ]]; then
    link_repo_bin "$resolved" "$name"
  fi
}

link_repo_bin() {
  local target="$1"
  local name="$2"

  [[ -e "$target" ]] || return 0
  mkdir -p "$TOOLS_BIN_DIR"
  ln -sfn "$target" "$TOOLS_BIN_DIR/$name"
}

wrap_repo_bin() {
  local target="$1"
  local name="$2"
  local dest="$TOOLS_BIN_DIR/$name"

  [[ -e "$target" ]] || return 0
  mkdir -p "$TOOLS_BIN_DIR"
  if [[ -L "$dest" || -e "$dest" ]]; then
    safe_trash_path "$dest"
  fi
  cat >"$dest" <<EOF
#!/bin/bash
set -euo pipefail
exec "$target" "\$@"
EOF
  chmod +x "$dest"
}

write_exec_wrapper() {
  local dest="$1"
  local target="$2"

  mkdir -p "$(dirname "$dest")"
  cat >"$dest" <<EOF
#!/bin/bash
set -euo pipefail
exec "$target" "\$@"
EOF
  chmod +x "$dest"
}

write_env_wrapper() {
  local dest="$1"
  local target="$2"
  shift 2

  mkdir -p "$(dirname "$dest")"
  {
    echo '#!/bin/bash'
    echo 'set -euo pipefail'
    while [[ $# -ge 2 ]]; do
      local name="$1"
      local value="$2"
      shift 2
      printf 'export %s=%q\n' "$name" "$value"
    done
    printf 'exec %q "$@"\n' "$target"
  } >"$dest"
  chmod +x "$dest"
}

ensure_path_symlink() {
  local link_path="$1"
  local target_path="$2"

  mkdir -p "$(dirname "$link_path")"
  if [[ -L "$link_path" ]]; then
    local current_target
    current_target="$(readlink "$link_path" || true)"
    if [[ "$current_target" == "$target_path" ]]; then
      return 0
    fi
  fi
  if [[ -L "$link_path" || -e "$link_path" ]]; then
    safe_trash_path "$link_path"
  fi
  ln -sfn "$target_path" "$link_path"
}

wrap_repo_bin_env() {
  local target="$1"
  local name="$2"
  shift 2
  local dest="$TOOLS_BIN_DIR/$name"

  [[ -e "$target" ]] || return 0
  if [[ -L "$dest" || -e "$dest" ]]; then
    safe_trash_path "$dest"
  fi
  write_env_wrapper "$dest" "$target" "$@"
}

ensure_cargo_target_config() {
  local cargo_root="$1"
  local target_dir="$2"
  local config_dir="$cargo_root/.cargo"
  local config_path="$config_dir/config.toml"
  local escaped_target_dir
  escaped_target_dir="$(printf '%s\n' "$target_dir" | sed 's/[\\/&]/\\&/g')"

  mkdir -p "$config_dir"

  if [[ ! -f "$config_path" ]]; then
    cat >"$config_path" <<EOF
[build]
target-dir = "$target_dir"
EOF
    return 0
  fi

  if grep -Eq '^[[:space:]]*target-dir[[:space:]]*=' "$config_path"; then
    sed -i -E "s/^[[:space:]]*target-dir[[:space:]]*=.*/target-dir = \"$escaped_target_dir\"/" "$config_path"
    return 0
  fi

  if grep -Eq '^[[:space:]]*\[build\]([[:space:]]+.*)?$' "$config_path"; then
    local tmp
    tmp="$(mktemp)"
    awk -v td="$target_dir" '
      BEGIN { inserted = 0 }
      {
        if (!inserted && $0 ~ /^[[:space:]]*\[build\][[:space:]]*$/) {
          print
          print "target-dir = \"" td "\""
          inserted = 1
          next
        }
        if (!inserted && $0 ~ /^[[:space:]]*\[build\][[:space:]]+.+$/) {
          sub(/^[[:space:]]*\[build\][[:space:]]+/, "", $0)
          print "[build]"
          print "target-dir = \"" td "\""
          print
          inserted = 1
          next
        }
        print
      }
    ' "$config_path" >"$tmp"
    mv "$tmp" "$config_path"
    return 0
  fi

  cat >>"$config_path" <<EOF

[build]
target-dir = "$target_dir"
EOF
}

link_standalone_saw_bins() {
  link_first_executable saw "$TOOLS_DIR/saw" "$TOOLS_DIR/saw/bin/saw"
  if [[ -x "$TOOLS_DIR/saw/bin/saw" ]]; then
    link_repo_bin "$TOOLS_DIR/saw/bin/saw" saw
  fi

  local saw_crux_mir_comp=""
  local saw_crux_mir=""
  saw_crux_mir_comp="$(first_executable crux-mir-comp "$TOOLS_DIR/saw" || true)"
  saw_crux_mir="$(first_executable crux-mir "$TOOLS_DIR/saw" || true)"

  if [[ -n "$saw_crux_mir_comp" ]]; then
    ln -sfn "$saw_crux_mir_comp" "$TOOLS_DIR/saw/bin/crux-mir-comp"
    link_repo_bin "$TOOLS_DIR/saw/bin/crux-mir-comp" crux-mir-comp
  fi

  if [[ -n "$saw_crux_mir" ]]; then
    ln -sfn "$saw_crux_mir" "$TOOLS_DIR/saw/bin/crux-mir"
  elif [[ -n "$saw_crux_mir_comp" ]]; then
    # Standalone SAW bundles currently ship only crux-mir-comp; expose a
    # deterministic crux-mir shim so cargo-crux-test can execute in research
    # profile without requiring the Docker-only saw-suite image.
    write_exec_wrapper "$TOOLS_DIR/saw/bin/crux-mir" "$TOOLS_DIR/saw/bin/crux-mir-comp"
  fi

  if [[ -x "$TOOLS_DIR/saw/bin/crux-mir" ]]; then
    link_repo_bin "$TOOLS_DIR/saw/bin/crux-mir" crux-mir
  fi
}

saw_suite_bin_path() {
  local name="$1"
  printf '%s\n' "$TOOLS_DIR/saw-suite/bin/$name"
}

saw_suite_has_tool() {
  local name="$1"
  [[ -x "$(saw_suite_bin_path "$name")" ]]
}

link_saw_suite_bins() {
  local tool
  for tool in \
    saw cryptol cargo-saw-build cargo-crux-test cargo-mir-json mir-json mir-json-translate-libs \
    mir-json-rustc-wrapper crux-rustc saw-rustc crux-mir crux-mir-comp cvc5 bitwuzla abc \
    boolector yices yices-smt2 yices_sat yices_sat_new yices_smt yices_smt2 yices_smtcomp z3; do
    if saw_suite_has_tool "$tool"; then
      wrap_repo_bin "$(saw_suite_bin_path "$tool")" "$tool"
    fi
  done
}

local_opam_switch_exists() {
  have opam || return 1
  OPAMROOT="$LOCAL_OPAM_ROOT" opam switch list --root "$LOCAL_OPAM_ROOT" --short 2>/dev/null | grep -Fxq "$LOCAL_OPAM_SWITCH"
}

aeneas_opam_switch_exists() {
  have opam || return 1
  OPAMROOT="$LOCAL_OPAM_ROOT" opam switch list --root "$LOCAL_OPAM_ROOT" --short 2>/dev/null | grep -Fxq "$LOCAL_AENEAS_OPAM_SWITCH"
}

local_opam_switch_has_binary() {
  local tool="$1"
  [[ -x "$LOCAL_OPAM_ROOT/$LOCAL_OPAM_SWITCH/bin/$tool" ]]
}

aeneas_opam_switch_has_binary() {
  local tool="$1"
  [[ -x "$LOCAL_OPAM_ROOT/$LOCAL_AENEAS_OPAM_SWITCH/bin/$tool" ]]
}

repair_local_opam_root_paths() {
  [[ -d "$LOCAL_OPAM_ROOT" ]] || return 0

  local repair_result=""
  repair_result="$(
    python3 - "$ROOT_DIR" "$LOCAL_OPAM_ROOT" "${HOME:-}" "$(id -un 2>/dev/null || true)" "$(id -gn 2>/dev/null || true)" <<'PY'
import os
import re
import sys
from pathlib import Path

project_root = Path(sys.argv[1]).resolve()
opam_root = Path(sys.argv[2]).resolve()
home_dir_arg = sys.argv[3].strip()
user_name = sys.argv[4].strip()
group_name = sys.argv[5].strip()
home_dir = Path(home_dir_arg).resolve() if home_dir_arg else Path.home().resolve()
project_marker = "/tools/formal_verification/opam/root"

text_suffixes = {
    ".conf",
    ".config",
    ".cfg",
    ".json",
    ".lock",
    ".md",
    ".ml",
    ".mli",
    ".mll",
    ".mly",
    ".opam",
    ".sexp",
    ".sh",
    ".txt",
    ".yaml",
    ".yml",
}

text_names = {
    "Makefile",
    "META",
    "dune-package",
    "findlib.conf",
    "switch-config",
    "switch-state",
}

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
    return is_probably_utf8_text(path)

project_roots = set()
candidates = []

for path in sorted(opam_root.rglob("*")):
    if not path.is_file() or not is_text_candidate(path):
        continue
    try:
        content = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        continue
    candidates.append((path, content))
    if project_marker in content:
        for prefix in re.findall(r"(/[^\s\"'():<>\[\]]+?)(?=" + re.escape(project_marker) + r")", content):
            if prefix != str(project_root):
                project_roots.add(prefix)

rewritten = 0
for path, content in candidates:
    updated = content.replace("__Z00Z_PROJECT_ROOT__", str(project_root))
    updated = updated.replace("__Z00Z_USER_HOME__", str(home_dir))
    for prefix in sorted(project_roots, key=len, reverse=True):
        updated = updated.replace(prefix, str(project_root))
    if path.name == "switch-config":
        updated = re.sub(
            r'(^\s*opam-root:\s*)".*"$',
            rf'\1"{opam_root.as_posix()}"',
            updated,
            flags=re.MULTILINE,
        )
        if user_name:
            updated = re.sub(r'(^\s*user:\s*)".*"$', rf'\1"{user_name}"', updated, flags=re.MULTILINE)
        if group_name:
            updated = re.sub(r'(^\s*group:\s*)".*"$', rf'\1"{group_name}"', updated, flags=re.MULTILINE)
    if path.name == "config":
        updated = re.sub(r"(^\s*depext-run-installs:\s*)true$", r"\1false", updated, flags=re.MULTILINE)
    if updated != content:
        path.write_text(updated, encoding="utf-8")
        rewritten += 1

print(f"{rewritten} {len(project_roots)}")
PY
  )"

  if [[ -n "$repair_result" ]]; then
    local rewritten_count=0
    local root_count=0
    read -r rewritten_count root_count <<<"$repair_result"
    if (( rewritten_count > 0 || root_count > 0 )); then
      log "Repaired packaged OPAM metadata paths ($rewritten_count files updated, $root_count project-root variants normalized)"
    fi
  fi
}

opam_exec_local() {
  opam_filter_output env \
    OPAMROOT="$LOCAL_OPAM_ROOT" \
    OPAMYES=1 \
    OPAMCONFIRMLEVEL=unsafe-yes \
    OPAMASSUMEDEPEXTS=1 \
    OPAMNOSELFUPGRADE=1 \
    OPAMNOAUTOUPGRADE=1 \
    opam exec --root "$LOCAL_OPAM_ROOT" --switch "$LOCAL_OPAM_SWITCH" -- "$@"
}

opam_local_cmd() {
  opam_filter_output env \
    OPAMROOT="$LOCAL_OPAM_ROOT" \
    OPAMYES=1 \
    OPAMCONFIRMLEVEL=unsafe-yes \
    OPAMASSUMEDEPEXTS=1 \
    OPAMNOSELFUPGRADE=1 \
    OPAMNOAUTOUPGRADE=1 \
    opam "$@"
}

opam_exec_aeneas() {
  opam_filter_output env \
    OPAMROOT="$LOCAL_OPAM_ROOT" \
    OPAMYES=1 \
    OPAMCONFIRMLEVEL=unsafe-yes \
    OPAMASSUMEDEPEXTS=1 \
    OPAMNOSELFUPGRADE=1 \
    OPAMNOAUTOUPGRADE=1 \
    opam exec --root "$LOCAL_OPAM_ROOT" --switch "$LOCAL_AENEAS_OPAM_SWITCH" -- "$@"
}

opam_aeneas_cmd() {
  opam_filter_output env \
    OPAMROOT="$LOCAL_OPAM_ROOT" \
    OPAMYES=1 \
    OPAMCONFIRMLEVEL=unsafe-yes \
    OPAMASSUMEDEPEXTS=1 \
    OPAMNOSELFUPGRADE=1 \
    OPAMNOAUTOUPGRADE=1 \
    opam "$@"
}

activate_local_tool_env
stage_repo_config_files

nightly_has_component() {
  local component="$1"
  rustup +nightly component list --installed 2>/dev/null | grep -Eq "^${component}($|-)"
}

miri_sysroot_stamp_path() {
  printf '%s\n' "$LOCAL_MIRI_SYSROOT/.rustc-vv"
}

current_nightly_rustc_fingerprint() {
  rustc +nightly -Vv 2>/dev/null
}

miri_sysroot_matches_nightly() {
  local stamp_path current_fingerprint
  stamp_path="$(miri_sysroot_stamp_path)"
  current_fingerprint="$(current_nightly_rustc_fingerprint)"
  [[ -n "$current_fingerprint" ]] || return 1
  [[ -f "$stamp_path" ]] || return 1
  [[ "$(cat "$stamp_path" 2>/dev/null || true)" == "$current_fingerprint" ]]
}

miri_sysroot_ready() {
  local host target_lib
  host="$(rustc +nightly -vV 2>/dev/null | sed -n 's/^host: //p' | head -n 1)"
  [[ -n "$host" ]] || return 1
  target_lib="$LOCAL_MIRI_SYSROOT/lib/rustlib/$host/lib"
  [[ -d "$target_lib" ]] || return 1
  compgen -G "$target_lib/libcore-*.rlib" >/dev/null || return 1
  compgen -G "$target_lib/libstd-*.rlib" >/dev/null || return 1
  miri_sysroot_matches_nightly
}

as_root() {
  if [[ "${EUID:-$(id -u)}" -eq 0 ]]; then
    "$@"
  elif have sudo && sudo -n true >/dev/null 2>&1; then
    sudo "$@"
  elif have sudo && [[ -t 0 ]]; then
    sudo "$@"
  else
    warn "passwordless sudo is not available in this non-interactive session; cannot run: $*"
    return 1
  fi
}

install_system_deps() {
  [[ "$SKIP_SYSTEM" -eq 0 ]] || return 0

  if have apt-get; then
    log "Installing system packages with apt-get"
    as_root bash -lc '
      export DEBIAN_FRONTEND=noninteractive
      export DEBCONF_NOWARNINGS=yes
      export RUNLEVEL=1
      exec apt-get update \
        > >(sed \
          -e "s/\r$//" \
          -e "/^invoke-rc.d: could not determine current runlevel$/d" \
          -e "/^invoke-rc.d: policy-rc.d denied execution of .*$/d") \
        2> >(sed \
          -e "s/\r$//" \
          -e "/^invoke-rc.d: could not determine current runlevel$/d" \
          -e "/^invoke-rc.d: policy-rc.d denied execution of .*$/d" >&2)
    ' || {
      warn "system package update failed or requires interactive sudo; continuing with user-space tools"
      return 0
    }
    as_root env DEBIAN_FRONTEND=noninteractive DEBCONF_NOWARNINGS=yes RUNLEVEL=1 apt-get install -y --no-install-recommends \
      openjdk-17-jre-headless ca-certificates-java >/dev/null 2>&1 || true
    as_root bash -lc '
      export DEBIAN_FRONTEND=noninteractive
      export DEBCONF_NOWARNINGS=yes
      export RUNLEVEL=1
      exec apt-get install -y --no-install-recommends \
        apt-utils build-essential pkg-config libssl-dev clang lld cmake git curl mercurial rsync unzip zip \
        nodejs openjdk-17-jdk openjdk-17-jre-headless graphviz opam z3 jq shellcheck ripgrep python3 python3-yaml python3-venv pipx \
        autoconf ca-certificates coreutils libcairo2-dev libexpat1-dev libgmp-dev libgtk2.0-dev libgtk-3-dev \
        libgtksourceview-3.0-dev libmpfr-dev libpcre2-dev xz-utils zlib1g-dev ca-certificates-java \
        > >(sed \
          -e "s/\r$//" \
          -e "/^invoke-rc.d: could not determine current runlevel$/d" \
          -e "/^invoke-rc.d: policy-rc.d denied execution of .*$/d") \
        2> >(sed \
          -e "s/\r$//" \
          -e "/^invoke-rc.d: could not determine current runlevel$/d" \
          -e "/^invoke-rc.d: policy-rc.d denied execution of .*$/d" >&2)
    ' || warn "system package installation failed; continuing with user-space tools"
  elif have pacman; then
    log "Installing system packages with pacman"
    as_root pacman -Sy --needed --noconfirm \
      base-devel pkgconf openssl clang lld cmake git curl mercurial rsync unzip zip \
      jdk17-openjdk graphviz opam z3 nodejs npm jq shellcheck ripgrep python python-yaml python-pipx \
      autoconf ca-certificates cairo coreutils expat gmp gtk2 gtk3 gtksourceview3 mpfr pcre2 xz zlib \
      || warn "system package installation failed; continuing with user-space tools"
  else
    warn "unsupported package manager; install system deps manually"
  fi
}

ensure_rustup() {
  if ! have rustup; then
    local final_rustup_home="$RUSTUP_HOME"
    local final_cargo_home="$CARGO_HOME"
    local install_root
    local install_home
    local install_rustup_home
    local install_cargo_home

    safe_trash_path "$final_rustup_home"
    safe_trash_path "$final_cargo_home"
    install_root="$(mktemp -d "${TMPDIR:-/tmp}/z00z-rustup-install.XXXXXX")"
    install_home="$install_root/home"
    install_rustup_home="$install_root/rustup"
    install_cargo_home="$install_root/cargo"
    log "Installing rustup"
    # shellcheck disable=SC2016
    rustup_filter_output sh -lc '
      curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | \
        env HOME="$1" RUSTUP_HOME="$2" CARGO_HOME="$3" RUSTUP_INIT_SKIP_SUDO_CHECK=yes \
          sh -s -- -q -y --no-modify-path --profile minimal
    ' sh "$install_home" "$install_rustup_home" "$install_cargo_home"
    scrub_shell_startup_path "$install_root"
    mkdir -p "$(dirname "$final_rustup_home")" "$(dirname "$final_cargo_home")"
    mv "$install_rustup_home" "$final_rustup_home"
    mv "$install_cargo_home" "$final_cargo_home"
    write_local_cargo_env "$final_cargo_home"
    safe_trash_path "$install_root"
  fi

  log "Updating Rust toolchains"
  rustup update
  rustup component add rustfmt clippy rust-src llvm-tools-preview
  rustup toolchain install nightly
  rustup +nightly component add miri rust-src llvm-tools-preview
}

ensure_nightly_miri_component() {
  if nightly_has_component miri; then
    return 0
  fi
  log "Installing nightly Miri component"
  rustup toolchain install nightly
  rustup +nightly component add miri rust-src llvm-tools-preview
}

ensure_miri_sysroot() {
  nightly_has_component miri || return 0
  if miri_sysroot_ready && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already installed: Miri sysroot"
    return 0
  fi

  local setup_tmp
  setup_tmp="$(mktemp -d "${TMPDIR:-/tmp}/z00z-miri-setup.XXXXXX")"
  if [[ -d "$LOCAL_MIRI_SYSROOT" ]]; then
    safe_trash_path "$LOCAL_MIRI_SYSROOT"
  fi
  mkdir -p "$LOCAL_MIRI_SYSROOT"
  log "Prebuilding Miri sysroot -> ${LOCAL_MIRI_SYSROOT#"$ROOT_DIR"/}"
  (
    cd "$setup_tmp" &&
    TMPDIR="${TMPDIR:-/tmp}" MIRI_SYSROOT="$LOCAL_MIRI_SYSROOT" cargo +nightly miri setup >/dev/null
  )
  current_nightly_rustc_fingerprint >"$(miri_sysroot_stamp_path)"
  safe_trash_path "$setup_tmp"
  if ! miri_sysroot_ready; then
    warn "Miri sysroot bootstrap did not produce a complete host sysroot at ${LOCAL_MIRI_SYSROOT#"$ROOT_DIR"/}"
    return 1
  fi
}

binary_semver() {
  local bin="$1"
  "$bin" --version 2>/dev/null | sed -nE 's/.* ([0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z]+)?)$/\1/p' | head -n 1
}

version_ge() {
  local current="$1"
  local required="$2"
  [[ -n "$current" && -n "$required" ]] || return 1
  [[ "$(printf '%s\n%s\n' "$required" "$current" | sort -V | head -n 1)" == "$required" ]]
}

append_compiler_flag() {
  local current="${1:-}"
  local extra="$2"
  if [[ -n "$current" ]]; then
    printf '%s %s' "$current" "$extra"
    return 0
  fi
  printf '%s' "$extra"
}

external_rust_build_env() {
  env \
    CARGO_TERM_COLOR=never \
    RUSTFLAGS="$(append_compiler_flag "${RUSTFLAGS:-}" "-Awarnings")" \
    RUSTDOCFLAGS="$(append_compiler_flag "${RUSTDOCFLAGS:-}" "-Awarnings")" \
    "$@"
}

normalize_rust_sysroot_duplicate_metadata() {
  local toolchain="$1"
  local backup_root="$2"
  shift 2 || true

  local target_libdir=""
  target_libdir="$(rustc +"$toolchain" --print target-libdir 2>/dev/null || true)"
  if [[ -z "$target_libdir" || ! -d "$target_libdir" ]]; then
    warn "Could not resolve Rust sysroot target-libdir for $toolchain; skipping metadata normalization"
    return 0
  fi

  mkdir -p "$backup_root"

  local moved=0
  local crate_name=""
  for crate_name in "$@"; do
    [[ -n "$crate_name" ]] || continue
    if ! compgen -G "$target_libdir/lib${crate_name}-*.rlib" >/dev/null; then
      continue
    fi
    if ! compgen -G "$target_libdir/lib${crate_name}-*.rmeta" >/dev/null; then
      continue
    fi

    local crate_backup="$backup_root/$crate_name"
    mkdir -p "$crate_backup"

    local rmeta_path=""
    for rmeta_path in "$target_libdir"/lib"$crate_name"-*.rmeta; do
      [[ -e "$rmeta_path" ]] || continue
      mv -f "$rmeta_path" "$crate_backup/"
      log "Moved duplicate sysroot metadata for $crate_name -> ${crate_backup#"$ROOT_DIR"/}/$(basename "$rmeta_path")"
      moved=1
    done
  done

  if [[ "$moved" -eq 0 ]]; then
    log "No conflicting sysroot metadata found for $toolchain"
  fi
}

packaged_source_snapshot_available() {
  local repo_dir="$1"
  local marker_path="$2"
  [[ -d "$repo_dir" && ! -d "$repo_dir/.git" && -e "$repo_dir/$marker_path" ]]
}

existing_source_tree_available() {
  local repo_dir="$1"
  [[ -d "$repo_dir" ]] || return 1
  [[ -n "$(find "$repo_dir" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]]
}

remove_optional_generated_doc_symlink() {
  local link_path="$1"

  if [[ -L "$link_path" && ! -e "$link_path" ]]; then
    rm -f "$link_path"
    log "Removed optional generated-doc symlink ${link_path#"$ROOT_DIR"/}"
  fi
}

read_declared_rust_toolchain() {
  local repo_dir="$1"
  python3 - "$repo_dir" <<'PY'
import pathlib
import re
import sys

repo_dir = pathlib.Path(sys.argv[1])
for name in ("rust-toolchain.toml", "rust-toolchain"):
    path = repo_dir / name
    if not path.is_file():
        continue
    text = path.read_text(encoding="utf-8")
    match = re.search(r'^\s*channel\s*=\s*"([^"]+)"', text, re.MULTILINE)
    if match:
        print(match.group(1))
        raise SystemExit(0)
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or line.startswith("["):
            continue
        print(line.strip('"').strip("'"))
        raise SystemExit(0)
raise SystemExit(0)
PY
}

ensure_declared_rust_toolchain_components() {
  local repo_dir="$1"
  shift

  local toolchain=""
  toolchain="$(read_declared_rust_toolchain "$repo_dir")"
  [[ -n "$toolchain" ]] || return 0

  log "Ensuring Rust toolchain $toolchain components: $*"
  rustup toolchain install "$toolchain" --profile minimal
  rustup component add --toolchain "$toolchain" "$@"
}

cargo_install_locked() {
  local crate="$1"
  local bin="${2:-$1}"
  shift 2 || true

  if tool_path_is_local "$bin" && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already installed: $bin"
    return 0
  fi

  log "cargo install --locked $crate"
  if [[ "$FORCE_UPGRADE" -eq 1 ]]; then
    cargo install --locked --force --root "$LOCAL_CARGO_HOME" "$crate" "$@"
  else
    cargo install --locked --root "$LOCAL_CARGO_HOME" "$crate" "$@"
  fi

  if [[ -x "$LOCAL_CARGO_HOME/bin/$bin" ]]; then
    link_repo_bin "$LOCAL_CARGO_HOME/bin/$bin" "$bin"
  fi
}

cargo_install_resolved() {
  local crate="$1"
  local bin="${2:-$1}"
  shift 2 || true

  if tool_path_is_local "$bin" && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already installed: $bin"
    return 0
  fi

  log "cargo install $crate"
  if [[ "$FORCE_UPGRADE" -eq 1 ]]; then
    cargo install --force --root "$LOCAL_CARGO_HOME" "$crate" "$@"
  else
    cargo install --root "$LOCAL_CARGO_HOME" "$crate" "$@"
  fi

  if [[ -x "$LOCAL_CARGO_HOME/bin/$bin" ]]; then
    link_repo_bin "$LOCAL_CARGO_HOME/bin/$bin" "$bin"
  fi
}

install_cargo_tools() {
  cargo_install_locked cargo-nextest cargo-nextest
  cargo_install_locked cargo-audit cargo-audit
  if tool_path_is_local cargo-deny && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    cargo_deny_version="$(binary_semver cargo-deny || true)"
    if version_ge "${cargo_deny_version:-}" "0.19.0"; then
      log "Already installed: cargo-deny (${cargo_deny_version:-unknown})"
    else
      log "Upgrading cargo-deny for CVSS 4 RustSec compatibility (${cargo_deny_version:-unknown} -> >= 0.19.0)"
      cargo install --locked --force --root "$LOCAL_CARGO_HOME" cargo-deny
    fi
  else
    cargo_install_locked cargo-deny cargo-deny
  fi
  cargo_install_resolved cargo-vet cargo-vet
  cargo_install_locked cargo-fuzz cargo-fuzz
  cargo_install_locked cargo-llvm-cov cargo-llvm-cov
  cargo_install_locked cargo-semver-checks cargo-semver-checks
  cargo_install_locked just just
  cargo_install_locked bacon bacon
  cargo_install_locked watchexec-cli watchexec
  cargo_install_locked mdbook mdbook
  cargo_install_locked lychee lychee
  cargo_install_resolved taplo-cli taplo

  if ! tool_path_is_local cargo-geiger || [[ "$FORCE_UPGRADE" -eq 1 ]]; then
    log "cargo install cargo-geiger"
    if [[ "$FORCE_UPGRADE" -eq 1 ]]; then
      cargo install --force --root "$LOCAL_CARGO_HOME" cargo-geiger || cargo install --force --root "$LOCAL_CARGO_HOME" cargo-geiger --features vendored-openssl
    else
      cargo install --root "$LOCAL_CARGO_HOME" cargo-geiger || cargo install --root "$LOCAL_CARGO_HOME" cargo-geiger --features vendored-openssl
    fi
  else
    log "Already installed: cargo-geiger"
  fi
  link_repo_bin "$LOCAL_CARGO_HOME/bin/cargo-geiger" cargo-geiger
}

install_kani() {
  local bundle_ready=0

  if [[ -n "$(find "$LOCAL_KANI_HOME" -maxdepth 3 -path '*/bin/kani-driver' -type f 2>/dev/null | head -n 1 || true)" ]]; then
    bundle_ready=1
  fi

  if tool_path_is_local cargo-kani && [[ "$bundle_ready" -eq 1 && "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already installed: cargo-kani"
    return 0
  fi

  log "Installing Kani"
  if [[ "$FORCE_UPGRADE" -eq 1 ]]; then
    cargo install --locked --force --root "$LOCAL_CARGO_HOME" kani-verifier
  else
    cargo install --locked --root "$LOCAL_CARGO_HOME" kani-verifier
  fi

  mkdir -p "$TOOLS_BIN_DIR"
  cat >"$TOOLS_BIN_DIR/cargo-kani" <<EOF
#!/bin/bash
set -euo pipefail

export KANI_HOME="$LOCAL_KANI_HOME"
exec "$LOCAL_CARGO_HOME/bin/cargo-kani" "\$@"
EOF
  chmod +x "$TOOLS_BIN_DIR/cargo-kani"

  cat >"$TOOLS_BIN_DIR/kani" <<EOF
#!/bin/bash
set -euo pipefail

export KANI_HOME="$LOCAL_KANI_HOME"
exec "$LOCAL_CARGO_HOME/bin/kani" "\$@"
EOF
  chmod +x "$TOOLS_BIN_DIR/kani"
}

safe_trash_path() {
  local path="$1"
  [[ -e "$path" ]] || return 0

  if have trash-put; then
    if trash-put "$path" >/dev/null 2>&1; then
      return 0
    fi
  fi

  if have gio; then
    if gio trash "$path" >/dev/null 2>&1; then
      return 0
    fi
  fi

  if [[ -L "$path" || -f "$path" ]]; then
    rm -f "$path"
    return 0
  fi

  rm -rf "$path"
}

opam_filter_output() {
  local status
  set +e
  "$@" \
    > >(sed \
      -e 's/\r$//' \
      -e '/^[[:space:]]*\[WARNING\] Running as root is not recommended$/d' \
      -e '/^[[:space:]]*\[WARNING\] Shell not updated in non-interactive mode: use --shell-setup$/d' \
      -e '/^[[:space:]]*\[WARNING\] opam is out-of-date\./d' \
      -e '/^[[:space:]]*-[[:space:]]you won'\''t be able to use darcs repositories unless you install the darcs command on your system\.$/d') \
    2> >(sed \
      -e 's/\r$//' \
      -e '/^[[:space:]]*\[WARNING\] Running as root is not recommended$/d' \
      -e '/^[[:space:]]*\[WARNING\] Shell not updated in non-interactive mode: use --shell-setup$/d' \
      -e '/^[[:space:]]*\[WARNING\] opam is out-of-date\./d' \
      -e '/^[[:space:]]*-[[:space:]]you won'\''t be able to use darcs repositories unless you install the darcs command on your system\.$/d' >&2)
  status=$?
  set -e
  return "$status"
}

rustup_filter_output() {
  local status
  set +e
  "$@" \
    > >(sed \
      -e 's/\r$//' \
      -e '/^warn: It looks like you have an existing rustup settings file at:$/d' \
      -e '/^warn: .*settings\.toml$/d' \
      -e '/^warn: Rustup will install the default toolchain as specified in the settings file,$/d' \
      -e '/^warn: instead of the one inferred from the default host triple\.$/d' \
      -e '/^error: [$]HOME differs from euid-obtained home directory: you may be using sudo$/d' \
      -e '/^error: [$]HOME directory: .*$/d' \
      -e '/^error: euid-obtained home directory: .*$/d' \
      -e '/^Cargo'\''s bin directory \(.*\/cargo\/bin\)\.$/d' \
      -e '/^.*\/cargo\/bin.*$/d' \
      -e '/^This path needs to be in your PATH environment variable.*$/d' \
      -e '/^To configure your current shell, you need to source.*$/d' \
      -e '/^To get started you .*$/d' \
      -e '/^This would reload your .*$/d' \
      -e '/^Rust is installed now\. Great!$/d' \
      -e '/^environment variable\. This has not been done automatically\.$/d' \
      -e '/^the corresponding env file under .*\/cargo\.$/d' \
      -e '/^This is usually done by running one of the following.*$/d' \
      -e '/^Consider running the right command for your shell.*$/d' \
      -e '/^sh\/bash\/zsh\/ash\/dash\/pdksh$/d' \
      -e '/^\. \".*\/cargo\/env\".*$/d' \
      -e '/^source \".*\/cargo\/env\.fish\".*$/d' \
      -e '/^source \".*\/cargo\/env\.nu\".*$/d' \
      -e '/^source \".*\/cargo\/env\.tcsh\".*$/d' \
      -e '/^\. \".*\/cargo\/env\.ps1\".*$/d' \
      -e '/^source \".*\/cargo\/env\.xsh\".*$/d') \
    2> >(sed \
      -e 's/\r$//' \
      -e '/^warn: It looks like you have an existing rustup settings file at:$/d' \
      -e '/^warn: .*settings\.toml$/d' \
      -e '/^warn: Rustup will install the default toolchain as specified in the settings file,$/d' \
      -e '/^warn: instead of the one inferred from the default host triple\.$/d' \
      -e '/^error: [$]HOME differs from euid-obtained home directory: you may be using sudo$/d' \
      -e '/^error: [$]HOME directory: .*$/d' \
      -e '/^error: euid-obtained home directory: .*$/d' \
      -e '/^Cargo'\''s bin directory \(.*\/cargo\/bin\)\.$/d' \
      -e '/^.*\/cargo\/bin.*$/d' \
      -e '/^This path needs to be in your PATH environment variable.*$/d' \
      -e '/^To configure your current shell, you need to source.*$/d' \
      -e '/^To get started you .*$/d' \
      -e '/^This would reload your .*$/d' \
      -e '/^Rust is installed now\. Great!$/d' \
      -e '/^environment variable\. This has not been done automatically\.$/d' \
      -e '/^the corresponding env file under .*\/cargo\.$/d' \
      -e '/^This is usually done by running one of the following.*$/d' \
      -e '/^Consider running the right command for your shell.*$/d' \
      -e '/^sh\/bash\/zsh\/ash\/dash\/pdksh$/d' \
      -e '/^\. \".*\/cargo\/env\".*$/d' \
      -e '/^source \".*\/cargo\/env\.fish\".*$/d' \
      -e '/^source \".*\/cargo\/env\.nu\".*$/d' \
      -e '/^source \".*\/cargo\/env\.tcsh\".*$/d' \
      -e '/^\. \".*\/cargo\/env\.ps1\".*$/d' \
      -e '/^source \".*\/cargo\/env\.xsh\".*$/d' >&2)
  status=$?
  set -e
  return "$status"
}

easycrypt_filter_output() {
  local status
  set +e
  "$@" \
    > >(sed \
      -e 's/\r$//' \
      -e '/^warning: cannot read config file .*\/why3\.conf:$/d' \
      -e '/^[[:space:]]\+.*\/why3\.conf: No such file or directory$/d') \
    2> >(sed \
      -e 's/\r$//' \
      -e '/^warning: cannot read config file .*\/why3\.conf:$/d' \
      -e '/^[[:space:]]\+.*\/why3\.conf: No such file or directory$/d' >&2)
  status=$?
  set -e
  return "$status"
}

write_local_cargo_env() {
  local cargo_home="$1"

  mkdir -p "$cargo_home/bin"
  cat >"$cargo_home/env" <<EOF
#!/bin/sh
# rustup shell setup for the repo-local toolchain
case ":\${PATH}:" in
    *:"$cargo_home/bin":*)
        ;;
    *)
        export PATH="$cargo_home/bin:\$PATH"
        ;;
esac
EOF
  chmod +x "$cargo_home/env"
}

scrub_shell_startup_path() {
  local needle="$1"
  local shell_file
  local tmp_file
  local shell_files=(
    "$HOME/.profile"
    "$HOME/.bashrc"
    "$HOME/.zprofile"
    "$HOME/.zshrc"
    "$HOME/.zshenv"
    "$HOME/.config/fish/config.fish"
    "$HOME/.config/nushell/config.nu"
  )

  [[ -n "$needle" ]] || return 0

  for shell_file in "${shell_files[@]}"; do
    [[ -f "$shell_file" ]] || continue
    if ! grep -Fq "$needle" "$shell_file"; then
      continue
    fi

    tmp_file="$(mktemp "${TMPDIR:-/tmp}/z00z-shell-scrub.XXXXXX")"
    awk -v needle="$needle" 'index($0, needle) == 0 { print }' "$shell_file" >"$tmp_file"
    cat "$tmp_file" >"$shell_file"
    safe_trash_path "$tmp_file"
  done
}

host_os() {
  case "$(uname -s)" in
    Linux) printf '%s\n' "linux" ;;
    Darwin) printf '%s\n' "macos" ;;
    *) printf '%s\n' "unknown" ;;
  esac
}

host_arch() {
  case "$(uname -m)" in
    x86_64|amd64) printf '%s\n' "x86_64" ;;
    aarch64|arm64) printf '%s\n' "arm64" ;;
    *) printf '%s\n' "unknown" ;;
  esac
}

checksum_manifest_path() {
  if [[ -f "$TOOLS_DIR/SHA256SUMS" ]]; then
    printf '%s\n' "$TOOLS_DIR/SHA256SUMS"
    return 0
  fi
  printf '%s\n' "$SHA256SUMS_SOURCE"
}

verify_download_checksum() {
  local file_path="$1"
  local manifest
  manifest="$(checksum_manifest_path)"
  [[ -f "$manifest" ]] || return 0

  local expected
  expected="$(python3 - "$manifest" "$(basename "$file_path")" <<'PY'
import pathlib
import sys

manifest = pathlib.Path(sys.argv[1])
basename = sys.argv[2]
for raw in manifest.read_text(encoding="utf-8", errors="replace").splitlines():
    line = raw.strip()
    if not line or line.startswith("#"):
        continue
    parts = line.split()
    if len(parts) < 2:
        continue
    if pathlib.Path(parts[-1]).name == basename:
        print(parts[0])
        break
PY
)"

  if [[ -z "$expected" ]]; then
    warn "no pinned sha256 entry found for $(basename "$file_path"); skipping checksum verification"
    return 0
  fi

  local actual
  actual="$(sha256sum "$file_path" | awk '{print $1}')"
  if [[ "$actual" != "$expected" ]]; then
    echo "ERROR: sha256 mismatch for $file_path" >&2
    echo "ERROR: expected $expected" >&2
    echo "ERROR: actual   $actual" >&2
    return 1
  fi
}

github_asset_url() {
  local repo="$1"
  local release="$2"
  local pattern="$3"
  local endpoint

  if [[ "$release" == "latest" ]]; then
    endpoint="https://api.github.com/repos/$repo/releases/latest"
  else
    endpoint="https://api.github.com/repos/$repo/releases/tags/$release"
  fi

  curl -fsSL "$endpoint" \
    | jq -r --arg pattern "$pattern" '.assets[] | select(.name | test($pattern)) | .browser_download_url' \
    | head -n 1
}

download_asset() {
  local repo="$1"
  local release="$2"
  local pattern="$3"
  local dest="$4"
  local url

  if [[ -f "$dest" && "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already downloaded: $dest"
    verify_download_checksum "$dest" || true
    return 0
  fi

  if ! have jq; then
    warn "jq is required to discover GitHub release assets for $repo"
    return 1
  fi

  mkdir -p "$(dirname "$dest")"
  url="$(github_asset_url "$repo" "$release" "$pattern" || true)"
  if [[ -z "$url" || "$url" == "null" ]]; then
    warn "no matching asset for $repo release $release pattern $pattern"
    return 1
  fi

  log "Downloading $repo release $release asset to $dest"
  curl -fL "$url" -o "$dest"
  verify_download_checksum "$dest"
}

download_archive_asset() {
  local repo="$1"
  local release="$2"
  local pattern="$3"
  local dest_dir="$4"
  local url
  local archive

  if ! have jq; then
    warn "jq is required to discover GitHub release assets for $repo"
    return 1
  fi

  mkdir -p "$dest_dir"
  url="$(github_asset_url "$repo" "$release" "$pattern" || true)"
  if [[ -z "$url" || "$url" == "null" ]]; then
    warn "no matching asset for $repo release $release pattern $pattern"
    return 1
  fi

  archive="$dest_dir/${url##*/}"
  if [[ -f "$archive" && "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already downloaded: $archive"
    verify_download_checksum "$archive" || true
    printf '%s\n' "$archive"
    return 0
  fi
  log "Downloading $repo release $release asset to $archive"
  curl -fL "$url" -o "$archive"
  verify_download_checksum "$archive"
  printf '%s\n' "$archive"
}

link_first_executable() {
  local name="$1"
  local search_dir="$2"
  local link_path="$3"
  local executable

  executable="$(find "$search_dir" -type f -name "$name" -perm -u+x 2>/dev/null | sort | head -n 1 || true)"
  if [[ -n "$executable" ]]; then
    mkdir -p "$(dirname "$link_path")"
    ln -sf "$executable" "$link_path"
  fi
}

extract_archive() {
  local archive="$1"
  local dest_dir="$2"

  case "$archive" in
    *.zip)
      unzip -q -o "$archive" -d "$dest_dir"
      ;;
    *.tar.gz|*.tgz)
      tar -xzf "$archive" -C "$dest_dir"
      ;;
    *.tar.xz|*.txz)
      tar -xJf "$archive" -C "$dest_dir"
      ;;
    *)
      warn "unsupported archive format: $archive"
      return 1
      ;;
  esac
}

host_release_suffix() {
  local tool="$1"
  local os arch
  os="$(host_os)"
  arch="$(host_arch)"

  case "$tool:$os:$arch" in
    ripgrep:linux:x86_64) printf '%s\n' "x86_64-unknown-linux-musl.tar.gz" ;;
    ripgrep:linux:arm64) printf '%s\n' "aarch64-unknown-linux-gnu.tar.gz" ;;
    ripgrep:macos:x86_64) printf '%s\n' "x86_64-apple-darwin.tar.gz" ;;
    ripgrep:macos:arm64) printf '%s\n' "aarch64-apple-darwin.tar.gz" ;;
    saw:linux:x86_64) printf '%s\n' "ubuntu-24.04-X64.tar.gz" ;;
    saw:macos:x86_64) printf '%s\n' "macos-15-intel-X64.tar.gz" ;;
    saw:macos:arm64) printf '%s\n' "macos-15-ARM64.tar.gz" ;;
    cryptol:linux:x86_64) printf '%s\n' "ubuntu-24.04-X64.tar.gz" ;;
    cryptol:macos:x86_64) printf '%s\n' "macos-15-intel-X64.tar.gz" ;;
    cryptol:macos:arm64) printf '%s\n' "macos-15-ARM64.tar.gz" ;;
    cvc5:linux:x86_64) printf '%s\n' "Linux-x86_64-static.zip" ;;
    cvc5:linux:arm64) printf '%s\n' "Linux-arm64-static.zip" ;;
    cvc5:macos:x86_64) printf '%s\n' "macOS-x86_64-static.zip" ;;
    cvc5:macos:arm64) printf '%s\n' "macOS-arm64-static.zip" ;;
    bitwuzla:linux:x86_64) printf '%s\n' "Linux-x86_64-static.zip" ;;
    bitwuzla:linux:arm64) printf '%s\n' "Linux-arm64-static.zip" ;;
    bitwuzla:macos:arm64) printf '%s\n' "macOS-arm64-static.zip" ;;
    *)
      printf '%s\n' ""
      ;;
  esac
}

resolve_tamarin_exec() {
  local executable

  if [[ -x "$TOOLS_DIR/tamarin/upstream/tamarin-prover" ]]; then
    printf '%s\n' "$TOOLS_DIR/tamarin/upstream/tamarin-prover"
    return 0
  fi

  executable="$(find "$TOOLS_DIR/tamarin" -type f -name tamarin-prover -perm -u+x ! -path "$TOOLS_DIR/tamarin/bin/*" ! -path "$TOOLS_DIR/tamarin/tamarin-prover-z00z" 2>/dev/null | sort | head -n 1 || true)"
  if [[ -n "$executable" ]]; then
    printf '%s\n' "$executable"
    return 0
  fi
}

resolve_tamarin_cmd() {
  if [[ -x "$TOOLS_DIR/tamarin/bin/tamarin-prover-z00z" ]]; then
    printf '%s\n' "$TOOLS_DIR/tamarin/bin/tamarin-prover-z00z"
    return 0
  fi
  if [[ -x "$TOOLS_DIR/tamarin/bin/tamarin-prover" ]]; then
    printf '%s\n' "$TOOLS_DIR/tamarin/bin/tamarin-prover"
    return 0
  fi
  resolve_tamarin_exec
}

write_opam_wrapper() {
  local tool="$1"
  local wrapper="$TOOLS_DIR/opam/bin/$tool"

  mkdir -p "$(dirname "$wrapper")"
  cat >"$wrapper" <<EOF
#!/bin/bash
set -euo pipefail

exec env OPAMROOT="$LOCAL_OPAM_ROOT" opam exec --root "$LOCAL_OPAM_ROOT" --switch "$LOCAL_OPAM_SWITCH" -- "$tool" "\$@"
EOF
  chmod +x "$wrapper"
}

write_system_opam_wrapper() {
  local wrapper="$TOOLS_DIR/opam/bin/opam"
  local system_opam

  system_opam="$(PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin" command -v opam || true)"
  [[ -n "$system_opam" ]] || return 0

  mkdir -p "$(dirname "$wrapper")"
  cat >"$wrapper" <<EOF
#!/bin/bash
set -euo pipefail

exec env OPAMNOSELFUPGRADE=1 OPAMNOAUTOUPGRADE=1 "$system_opam" "\$@" \\
  > >(sed \
    -e '/^[[:space:]]*\\[WARNING\\] Running as root is not recommended$/d' \
    -e '/^[[:space:]]*\\[WARNING\\] Shell not updated in non-interactive mode: use --shell-setup$/d' \
    -e '/^[[:space:]]*\\[WARNING\\] opam is out-of-date\\./d') \\
  2> >(sed \
    -e '/^[[:space:]]*\\[WARNING\\] Running as root is not recommended$/d' \
    -e '/^[[:space:]]*\\[WARNING\\] Shell not updated in non-interactive mode: use --shell-setup$/d' \
    -e '/^[[:space:]]*\\[WARNING\\] opam is out-of-date\\./d' >&2)
EOF
  chmod +x "$wrapper"
}

ensure_opam_wrappers() {
  [[ "$SKIP_OPAM" -eq 0 ]] || return 0
  if ! local_opam_switch_exists; then
    return 0
  fi
  write_system_opam_wrapper
  if opam_exec_local proverif -help >/dev/null 2>&1; then
    write_opam_wrapper proverif
    link_repo_bin "$TOOLS_DIR/opam/bin/proverif" proverif
  fi
  if opam_exec_local why3 --version >/dev/null 2>&1; then
    write_opam_wrapper why3
    link_repo_bin "$TOOLS_DIR/opam/bin/why3" why3
  fi
  if opam_exec_local easycrypt config >/dev/null 2>&1; then
    write_opam_wrapper easycrypt
    link_repo_bin "$TOOLS_DIR/opam/bin/easycrypt" easycrypt
  fi
  if opam_exec_local which hax-engine >/dev/null 2>&1; then
    write_opam_wrapper hax-engine
    link_repo_bin "$TOOLS_DIR/opam/bin/hax-engine" hax-engine
  fi
}

git_checkout_ref() {
  local repo_dir="$1"
  local ref="$2"

  if [[ ! -d "$repo_dir/.git" ]]; then
    return 1
  fi

  git -C "$repo_dir" fetch --quiet --tags --force origin
  git -c advice.detachedHead=false -C "$repo_dir" checkout --quiet --force "$ref"
}

ensure_standalone_cargo_root() {
  local manifest_path="$1"
  [[ -f "$manifest_path" ]] || return 0

  python3 - "$manifest_path" <<'PY'
import pathlib
import sys

marker = "# GENERATED BY z00z verification installer: detach from parent workspace"
path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
if "\n[workspace]\n" in text or text.startswith("[workspace]\n") or marker in text:
    raise SystemExit(0)

with path.open("a", encoding="utf-8") as handle:
    if not text.endswith("\n"):
        handle.write("\n")
    handle.write("\n")
    handle.write(marker)
    handle.write("\n[workspace]\n")
PY
}

ensure_standalone_top_level_cargo_crates() {
  local base_dir="$1"
  [[ -d "$base_dir" ]] || return 0

  local manifest
  while IFS= read -r manifest; do
    ensure_standalone_cargo_root "$manifest"
  done < <(find "$base_dir" -mindepth 2 -maxdepth 2 -type f -name Cargo.toml | sort)
}

install_alloy_headless_runner() {
  local alloy_jar="$TOOLS_DIR/alloy/org.alloytools.alloy.dist.jar"
  local source_file="$ROOT_DIR/scripts/Z00ZAlloyHeadless.java"
  local classes_dir="$TOOLS_DIR/alloy/classes"
  local bin_dir="$TOOLS_DIR/alloy/bin"
  local wrapper="$bin_dir/alloy-headless-z00z"

  [[ -f "$alloy_jar" ]] || return 0
  [[ -f "$source_file" ]] || {
    warn "Alloy headless runner source not found at $source_file"
    return 0
  }
  if ! have javac; then
    warn "javac is not installed; cannot build Alloy headless runner"
    return 0
  fi

  mkdir -p "$classes_dir" "$bin_dir"
  javac -cp "$alloy_jar" -d "$classes_dir" "$source_file"
  cat >"$wrapper" <<EOF
#!/bin/bash
set -euo pipefail

SCRIPT_DIR="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="\$(cd "\$SCRIPT_DIR/.." && pwd)"
ALLOY_JAR="\$ROOT_DIR/org.alloytools.alloy.dist.jar"
CLASSES_DIR="\$ROOT_DIR/classes"

exec java -cp "\$ALLOY_JAR:\$CLASSES_DIR" Z00ZAlloyHeadless "\$@"
EOF
  chmod +x "$wrapper"
  link_repo_bin "$wrapper" alloy-headless-z00z
}

resolve_local_maude_cmd() {
  if [[ -x "$TOOLS_DIR/maude/bin/maude" ]]; then
    printf '%s\n' "$TOOLS_DIR/maude/bin/maude"
  fi
}

resolve_maude_cmd() {
  resolve_local_maude_cmd
}

write_tamarin_wrapper() {
  local target="$1"
  local wrapper="$TOOLS_DIR/tamarin/bin/tamarin-prover-z00z"

  mkdir -p "$(dirname "$wrapper")"
  cat >"$wrapper" <<EOF
#!/bin/bash
set -euo pipefail

SCRIPT_DIR="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="\$(cd "\$SCRIPT_DIR/.." && pwd)"
MAUDE_BIN="\$ROOT_DIR/../maude/bin"

if [[ -d "\$MAUDE_BIN" ]]; then
  PATH="\$MAUDE_BIN:\$PATH"
fi

exec "$target" "\$@"
EOF
  chmod +x "$wrapper"
  ln -sfn "$wrapper" "$TOOLS_DIR/tamarin/bin/tamarin-prover"
}

tamarin_runtime_is_healthy() {
  local tamarin_cmd="$1"
  local maude_cmd="${2:-}"
  local status=0
  local log_file
  local path_dir=""

  log_file="$(mktemp "${TMPDIR:-/tmp}/z00z-tamarin-selftest.XXXXXX")"
  if [[ -n "$maude_cmd" ]]; then
    path_dir="$(dirname "$maude_cmd")"
    PATH="$path_dir:$PATH" timeout 30 "$tamarin_cmd" test >"$log_file" 2>&1 || status=$?
  else
    timeout 30 "$tamarin_cmd" test >"$log_file" 2>&1 || status=$?
  fi

  if grep -Eq "unsupported version|might NOT WORK AS INTENDED|WARNING: Some tests failed" "$log_file"; then
    status=1
  fi

  safe_trash_path "$log_file"
  [[ "$status" -eq 0 ]]
}

install_formal_jars() {
  mkdir -p "$TOOLS_DIR/tla" "$TOOLS_DIR/alloy" "$TOOLS_DIR/apalache"
  download_asset "tlaplus/tlaplus" "$TLA_RELEASE" '^tla2tools\.jar$' "$TOOLS_DIR/tla/tla2tools.jar" || true
  download_asset "AlloyTools/org.alloytools.alloy" "$ALLOY_RELEASE" '^org\.alloytools\.alloy\.dist\.jar$' "$TOOLS_DIR/alloy/org.alloytools.alloy.dist.jar" || true

  local apalache_archive
  if apalache_archive="$(download_archive_asset "apalache-mc/apalache" "$APALACHE_RELEASE" 'apalache.*\.(zip|tar\.gz|tgz)$' "$TOOLS_DIR/apalache")"; then
    case "$apalache_archive" in
      *.zip)
        unzip -q -o "$apalache_archive" -d "$TOOLS_DIR/apalache"
        ;;
      *.tar.gz|*.tgz)
        tar -xzf "$apalache_archive" -C "$TOOLS_DIR/apalache"
        ;;
    esac
    link_first_executable apalache-mc "$TOOLS_DIR/apalache" "$TOOLS_DIR/apalache/bin/apalache-mc"
    link_repo_bin "$TOOLS_DIR/apalache/bin/apalache-mc" apalache-mc
  fi

  install_alloy_headless_runner
}

install_node_tools() {
  local npm_cmd=""
  local node_cmd=""
  local node_major=0

  [[ "$SKIP_NODE" -eq 0 ]] || return 0
  ensure_modern_node_runtime

  if [[ -x "$LOCAL_NODE_PREFIX/bin/node" && -x "$LOCAL_NODE_PREFIX/bin/npm" ]]; then
    node_cmd="$LOCAL_NODE_PREFIX/bin/node"
    npm_cmd="$LOCAL_NODE_PREFIX/bin/npm"
  elif have node && have npm; then
    node_cmd="$(command -v node)"
    npm_cmd="$(command -v npm)"
  fi

  if [[ -z "$npm_cmd" || -z "$node_cmd" ]]; then
    warn "npm is not installed; skipping markdownlint-cli2"
    return 0
  fi

  node_major="$("$node_cmd" -p 'process.versions.node.split(".")[0]' 2>/dev/null || printf '0')"
  if [[ ! "$node_major" =~ ^[0-9]+$ ]] || (( node_major < 20 )); then
    warn "Skipping markdownlint-cli2 install because the available Node runtime is too old: $("$node_cmd" --version 2>/dev/null || printf 'unknown')"
    return 0
  fi

  if tool_path_is_local markdownlint-cli2 && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already installed: markdownlint-cli2"
    return 0
  fi

  log "Installing markdownlint-cli2 with $("$node_cmd" --version 2>/dev/null || printf 'node')"
  NPM_CONFIG_UPDATE_NOTIFIER=false NPM_CONFIG_FUND=false "$npm_cmd" --no-update-notifier --fund=false install --global --prefix "$LOCAL_NODE_PREFIX" markdownlint-cli2
  link_repo_bin "$LOCAL_NODE_PREFIX/bin/markdownlint-cli2" markdownlint-cli2
}

node_runtime_platform() {
  case "$(host_os):$(host_arch)" in
    linux:x86_64) printf '%s\n' "linux-x64" ;;
    linux:arm64) printf '%s\n' "linux-arm64" ;;
    macos:x86_64) printf '%s\n' "darwin-x64" ;;
    macos:arm64) printf '%s\n' "darwin-arm64" ;;
    *) printf '%s\n' "" ;;
  esac
}

current_node_major() {
  local version
  version="$(node --version 2>/dev/null || true)"
  version="${version#v}"
  printf '%s\n' "${version%%.*}"
}

ensure_modern_node_runtime() {
  local platform
  local manifest_url
  local manifest_path="$LOCAL_NODE_PREFIX/downloads/SHASUMS256.txt"
  local archive_name=""
  local archive_path=""
  local extract_dir="$LOCAL_NODE_PREFIX/extract"
  local runtime_root="$LOCAL_NODE_PREFIX/runtime"
  local extracted_root=""
  local current_major=0

  if have node; then
    current_major="$(current_node_major)"
    if [[ "$current_major" =~ ^[0-9]+$ ]] && (( current_major >= 20 )) && have npm; then
      return 0
    fi
  fi

  platform="$(node_runtime_platform)"
  if [[ -z "$platform" ]]; then
    warn "No official Node runtime mapping for $(host_os):$(host_arch); continuing with system node if present"
    return 0
  fi

  mkdir -p "$LOCAL_NODE_PREFIX/downloads" "$LOCAL_NODE_PREFIX/bin"
  manifest_url="https://nodejs.org/dist/${NODE_DIST_CHANNEL}/SHASUMS256.txt"
  log "Installing official Node runtime ${NODE_DIST_CHANNEL} for $platform"
  curl -fsSL "$manifest_url" -o "$manifest_path"

  archive_name="$(
    python3 - "$manifest_path" "$platform" <<'PY'
import pathlib
import re
import sys

manifest = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace").splitlines()
platform = sys.argv[2]
pattern = re.compile(rf"^(?:[0-9a-f]{{64}})\s+node-v[0-9.]+-{re.escape(platform)}\.tar\.(?:xz|gz)$")
for line in manifest:
    if pattern.match(line.strip()):
        print(line.split()[-1])
        break
PY
  )"

  if [[ -z "$archive_name" ]]; then
    warn "Could not resolve an official Node archive for platform $platform from $manifest_url"
    return 0
  fi

  archive_path="$LOCAL_NODE_PREFIX/downloads/$archive_name"
  if [[ ! -f "$archive_path" || "$FORCE_UPGRADE" -eq 1 ]]; then
    curl -fL "https://nodejs.org/dist/${NODE_DIST_CHANNEL}/${archive_name}" -o "$archive_path"
  fi

  mkdir -p "$extract_dir"
  mkdir -p "$runtime_root"
  extract_archive "$archive_path" "$extract_dir"
  extracted_root="$(find "$extract_dir" -mindepth 1 -maxdepth 1 -type d -name "node-v*-$platform" | sort | tail -n 1 || true)"
  if [[ -z "$extracted_root" ]]; then
    warn "Node archive extracted but no runtime directory was found for $platform"
    return 0
  fi

  if [[ -e "$runtime_root/current" ]]; then
    mv "$runtime_root/current" "$runtime_root/current.bak-$(date +%Y%m%d%H%M%S)"
  fi
  mv "$extracted_root" "$runtime_root/current"
  ln -sfn "$runtime_root/current/bin/node" "$LOCAL_NODE_PREFIX/bin/node"
  ln -sfn "$runtime_root/current/bin/npm" "$LOCAL_NODE_PREFIX/bin/npm"
  ln -sfn "$runtime_root/current/bin/npx" "$LOCAL_NODE_PREFIX/bin/npx"
  if [[ -x "$runtime_root/current/bin/corepack" ]]; then
    ln -sfn "$runtime_root/current/bin/corepack" "$LOCAL_NODE_PREFIX/bin/corepack"
  fi
  export PATH="$LOCAL_NODE_PREFIX/bin:$PATH"
}

install_python_tools() {
  local uv_tool_dir="$LOCAL_PYTHON_PREFIX/uv-tools"
  local uv_tool_bin="$LOCAL_PYTHON_PREFIX/bin"
  local pipx_home="$LOCAL_PYTHON_PREFIX/pipx"
  local pipx_bin="$LOCAL_PYTHON_PREFIX/bin"
  local venv_dir="$LOCAL_PYTHON_PREFIX/venvs/python-tools"

  mkdir -p "$uv_tool_dir" "$uv_tool_bin" "$pipx_home" "$pipx_bin"

  if have uv; then
    log "Installing local Python verifier tools with uv"
    UV_TOOL_DIR="$uv_tool_dir" UV_TOOL_BIN_DIR="$uv_tool_bin" uv tool install --force "$PYTHON_TOOL_RUFF"
    UV_TOOL_DIR="$uv_tool_dir" UV_TOOL_BIN_DIR="$uv_tool_bin" uv tool install --force "$PYTHON_TOOL_UV"
    link_repo_bin "$uv_tool_bin/ruff" ruff
    link_repo_bin "$uv_tool_bin/uv" uv
    return 0
  fi

  if have pipx; then
    log "Installing local Python verifier tools with pipx"
    PIPX_HOME="$pipx_home" PIPX_BIN_DIR="$pipx_bin" pipx install --force "$PYTHON_TOOL_RUFF"
    PIPX_HOME="$pipx_home" PIPX_BIN_DIR="$pipx_bin" pipx install --force "$PYTHON_TOOL_UV"
    link_repo_bin "$pipx_bin/ruff" ruff
    link_repo_bin "$pipx_bin/uv" uv
    return 0
  fi

  if ! have python3; then
    warn "python3 is not installed; skipping local Python verifier tools"
    return 0
  fi

  log "Installing local Python verifier tools with venv"
  python3 -m venv "$venv_dir"
  "$venv_dir/bin/pip" install --upgrade pip
  "$venv_dir/bin/pip" install "$PYTHON_TOOL_RUFF" "$PYTHON_TOOL_UV"
  link_repo_bin "$venv_dir/bin/ruff" ruff
  link_repo_bin "$venv_dir/bin/uv" uv
}

install_ripgrep() {
  local suffix archive
  suffix="$(host_release_suffix ripgrep)"
  if [[ -z "$suffix" ]]; then
    warn "no repo-local ripgrep asset mapping configured for $(host_os):$(host_arch)"
    return 0
  fi

  if [[ -x "$TOOLS_DIR/rg/bin/rg" && "$FORCE_UPGRADE" -eq 0 ]]; then
    link_repo_bin "$TOOLS_DIR/rg/bin/rg" rg
    log "Already installed: rg"
    return 0
  fi

  archive="$(download_archive_asset "BurntSushi/ripgrep" "$RIPGREP_RELEASE" "^ripgrep-${RIPGREP_RELEASE#v}-${suffix//./\\.}$" "$TOOLS_DIR/rg")" || return 0
  extract_archive "$archive" "$TOOLS_DIR/rg"
  link_first_executable rg "$TOOLS_DIR/rg" "$TOOLS_DIR/rg/bin/rg"
  link_repo_bin "$TOOLS_DIR/rg/bin/rg" rg
}

install_saw_suite() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_includes_research || return 0

  local portable_saw_suite_src="${Z00Z_PORTABLE_SAW_SUITE_SRC:-}"
  local tmp_dir="$TOOLS_DIR/saw-suite.tmp"

  if saw_suite_has_tool saw && saw_suite_has_tool cargo-saw-build && saw_suite_has_tool cargo-crux-test && saw_suite_has_tool crux-mir && [[ -d "$TOOLS_DIR/saw-suite/rlibs" ]] && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    link_saw_suite_bins
    log "Already installed: saw-suite"
    return 0
  fi

  if [[ -n "$portable_saw_suite_src" && -d "$portable_saw_suite_src/bin" && -d "$portable_saw_suite_src/rlibs" ]]; then
    if [[ -d "$tmp_dir" ]]; then
      safe_trash_path "$tmp_dir"
    fi
    mkdir -p "$tmp_dir"
    log "Installing saw-suite from portable sandbox asset $portable_saw_suite_src"
    cp -a "$portable_saw_suite_src/." "$tmp_dir/"
    if [[ -d "$TOOLS_DIR/saw-suite" ]]; then
      safe_trash_path "$TOOLS_DIR/saw-suite"
    fi
    mv "$tmp_dir" "$TOOLS_DIR/saw-suite"
    link_saw_suite_bins
    return 0
  fi

  if ! have docker; then
    log "docker is not available; falling back to standalone SAW/Cryptol/MIR-JSON research toolchain installation"
    return 0
  fi

  local container_id=""
  safe_trash_path "$tmp_dir"
  mkdir -p "$tmp_dir"

  log "Pulling pinned SAW suite image $SAW_SUITE_IMAGE"
  docker pull "$SAW_SUITE_IMAGE"

  container_id="$(docker create "$SAW_SUITE_IMAGE")"
  docker cp "$container_id:/opt/saw-suite/." "$tmp_dir/"
  docker rm -f "$container_id" >/dev/null 2>&1 || true

  if [[ -d "$TOOLS_DIR/saw-suite" ]]; then
    safe_trash_path "$TOOLS_DIR/saw-suite"
  fi
  mv "$tmp_dir" "$TOOLS_DIR/saw-suite"
  link_saw_suite_bins
}

install_saw() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_includes_research || return 0

  local suffix archive
  suffix="$(host_release_suffix saw)"
  if [[ -z "$suffix" ]]; then
    warn "no repo-local SAW asset mapping configured for $(host_os):$(host_arch)"
    return 0
  fi
  if saw_suite_has_tool saw && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    wrap_repo_bin "$(saw_suite_bin_path saw)" saw
    log "Already installed: saw via saw-suite"
    return 0
  fi
  if tool_path_is_local saw && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    link_standalone_saw_bins
    refresh_repo_bin_from_tool saw
    log "Already installed: saw"
    return 0
  fi

  archive="$(download_archive_asset "GaloisInc/saw-script" "$SAW_RELEASE" "^saw-${SAW_RELEASE#v}-${suffix//./\\.}$" "$TOOLS_DIR/saw")" || return 0
  extract_archive "$archive" "$TOOLS_DIR/saw"
  link_standalone_saw_bins
}

install_cryptol() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_includes_research || return 0

  local suffix archive
  suffix="$(host_release_suffix cryptol)"
  if [[ -z "$suffix" ]]; then
    warn "no repo-local Cryptol asset mapping configured for $(host_os):$(host_arch)"
    return 0
  fi
  if saw_suite_has_tool cryptol && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    wrap_repo_bin "$(saw_suite_bin_path cryptol)" cryptol
    log "Already installed: cryptol via saw-suite"
    return 0
  fi
  if tool_path_is_local cryptol && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    refresh_repo_bin_from_tool cryptol
    log "Already installed: cryptol"
    return 0
  fi

  archive="$(download_archive_asset "GaloisInc/cryptol" "$CRYPTOL_RELEASE" "^cryptol-${CRYPTOL_RELEASE}-${suffix//./\\.}$" "$TOOLS_DIR/cryptol")" || return 0
  extract_archive "$archive" "$TOOLS_DIR/cryptol"
  link_first_executable cryptol "$TOOLS_DIR/cryptol" "$TOOLS_DIR/cryptol/bin/cryptol"
  link_repo_bin "$TOOLS_DIR/cryptol/bin/cryptol" cryptol
}

install_cvc5() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_includes_research || return 0

  local suffix archive
  suffix="$(host_release_suffix cvc5)"
  if [[ -z "$suffix" ]]; then
    warn "no repo-local cvc5 asset mapping configured for $(host_os):$(host_arch)"
    return 0
  fi
  if saw_suite_has_tool cvc5 && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    wrap_repo_bin "$(saw_suite_bin_path cvc5)" cvc5
    log "Already installed: cvc5 via saw-suite"
    return 0
  fi
  if tool_path_is_local cvc5 && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    refresh_repo_bin_from_tool cvc5
    log "Already installed: cvc5"
    return 0
  fi

  archive="$(download_archive_asset "cvc5/cvc5" "$CVC5_RELEASE" "^cvc5-${suffix//./\\.}$" "$TOOLS_DIR/cvc5")" || return 0
  extract_archive "$archive" "$TOOLS_DIR/cvc5"
  link_first_executable cvc5 "$TOOLS_DIR/cvc5" "$TOOLS_DIR/cvc5/bin/cvc5"
  link_repo_bin "$TOOLS_DIR/cvc5/bin/cvc5" cvc5
}

install_bitwuzla() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_includes_research || return 0

  local suffix archive
  suffix="$(host_release_suffix bitwuzla)"
  if [[ -z "$suffix" ]]; then
    warn "no repo-local Bitwuzla asset mapping configured for $(host_os):$(host_arch)"
    return 0
  fi
  if saw_suite_has_tool bitwuzla && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    wrap_repo_bin "$(saw_suite_bin_path bitwuzla)" bitwuzla
    log "Already installed: bitwuzla via saw-suite"
    return 0
  fi
  if tool_path_is_local bitwuzla && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    refresh_repo_bin_from_tool bitwuzla
    log "Already installed: bitwuzla"
    return 0
  fi

  archive="$(download_archive_asset "bitwuzla/bitwuzla" "$BITWUZLA_RELEASE" "^Bitwuzla-${suffix//./\\.}$" "$TOOLS_DIR/bitwuzla")" || return 0
  extract_archive "$archive" "$TOOLS_DIR/bitwuzla"
  link_first_executable bitwuzla "$TOOLS_DIR/bitwuzla" "$TOOLS_DIR/bitwuzla/bin/bitwuzla"
  link_repo_bin "$TOOLS_DIR/bitwuzla/bin/bitwuzla" bitwuzla
}

ensure_opam_switch() {
  [[ "$SKIP_OPAM" -eq 0 ]] || return 1
  if ! have opam; then
    warn "opam is not installed"
    return 1
  fi

  repair_local_opam_root_paths

  if [[ ! -d "$LOCAL_OPAM_ROOT" ]]; then
    log "Initializing OPAM"
    opam_local_cmd init --root "$LOCAL_OPAM_ROOT" --bare --disable-sandboxing --no-setup --disable-shell-hook -y
  fi

  opam_local_cmd option --root "$LOCAL_OPAM_ROOT" --yes depext-run-installs=false || true

  if local_opam_switch_exists && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Reusing local OPAM switch $LOCAL_OPAM_SWITCH"
    return 0
  fi

  if ! local_opam_switch_exists; then
    log "Creating local OPAM switch $LOCAL_OPAM_SWITCH"
    opam_local_cmd switch create --root "$LOCAL_OPAM_ROOT" "$LOCAL_OPAM_SWITCH" "$OPAM_COMPILER" -y
  fi
  opam_local_cmd update --root "$LOCAL_OPAM_ROOT" --switch "$LOCAL_OPAM_SWITCH"
}

ensure_aeneas_opam_switch() {
  [[ "$SKIP_OPAM" -eq 0 ]] || return 1
  if ! have opam; then
    warn "opam is not installed"
    return 1
  fi

  repair_local_opam_root_paths

  if [[ ! -d "$LOCAL_OPAM_ROOT" ]]; then
    ensure_opam_switch || return 1
  fi

  opam_aeneas_cmd option --root "$LOCAL_OPAM_ROOT" --yes depext-run-installs=false || true

  if aeneas_opam_switch_exists && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Reusing local OPAM switch $LOCAL_AENEAS_OPAM_SWITCH"
    return 0
  fi

  if ! aeneas_opam_switch_exists; then
    log "Creating dedicated Aeneas OPAM switch $LOCAL_AENEAS_OPAM_SWITCH"
    opam_aeneas_cmd switch create --root "$LOCAL_OPAM_ROOT" "$LOCAL_AENEAS_OPAM_SWITCH" "$OPAM_COMPILER" -y
  fi
  opam_aeneas_cmd update --root "$LOCAL_OPAM_ROOT" --switch "$LOCAL_AENEAS_OPAM_SWITCH"
}

install_opam_core_tools() {
  repair_local_opam_root_paths
  if local_opam_switch_has_binary proverif && local_opam_switch_has_binary why3 && local_opam_switch_has_binary alt-ergo && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    ensure_opam_wrappers
    log "Already installed: ProVerif, Why3, Alt-Ergo"
    return 0
  fi
  ensure_opam_switch || return 0
  log "Installing ProVerif, Why3, and Alt-Ergo"
  opam_local_cmd install --root "$LOCAL_OPAM_ROOT" --switch "$LOCAL_OPAM_SWITCH" -y proverif why3 alt-ergo
}

install_easycrypt() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_includes_all || return 0
  repair_local_opam_root_paths
  if local_opam_switch_has_binary easycrypt && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    ensure_opam_wrappers
    log "Already installed: EasyCrypt"
    return 0
  fi
  ensure_opam_switch || return 0
  log "Installing EasyCrypt"
  (
    set -o pipefail
    opam_local_cmd pin --root "$LOCAL_OPAM_ROOT" --switch "$LOCAL_OPAM_SWITCH" -yn add easycrypt https://github.com/EasyCrypt/easycrypt.git 2>&1 | \
      sed -u -e '/^Package easycrypt does not exist, create as a NEW package? \[Y\/n\] y$/d'
  ) || true
  opam_local_cmd install --root "$LOCAL_OPAM_ROOT" --switch "$LOCAL_OPAM_SWITCH" -y --deps-only easycrypt
  opam_local_cmd install --root "$LOCAL_OPAM_ROOT" --switch "$LOCAL_OPAM_SWITCH" -y easycrypt
  # shellcheck disable=SC2016
  XDG_CONFIG_HOME="$TOOLS_DIR/easycrypt/config" PATH="/usr/bin:/bin" easycrypt_filter_output opam_exec_local sh -lc '
    mkdir -p "$XDG_CONFIG_HOME/easycrypt"
    : > "$XDG_CONFIG_HOME/easycrypt/why3.conf"
    exec easycrypt why3config
  ' || true
}

install_tamarin() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_at_least_recommended || return 0
  if tool_path_is_local tamarin-prover && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already installed: tamarin-prover"
    return 0
  fi

  local archive
  if archive="$(download_archive_asset "tamarin-prover/tamarin-prover" "$TAMARIN_RELEASE" 'tamarin-prover-.*linux64-ubuntu\.tar\.gz$|tamarin-prover-.*x86_64_linux\.bottle\.tar\.gz$|tamarin-prover-.*arm64_linux\.bottle\.tar\.gz$' "$TOOLS_DIR/tamarin")"; then
    mkdir -p "$TOOLS_DIR/tamarin/upstream"
    tar -xzf "$archive" -C "$TOOLS_DIR/tamarin/upstream"
    local tamarin_exec
    tamarin_exec="$(resolve_tamarin_exec || true)"
    if [[ -n "$tamarin_exec" ]]; then
      write_tamarin_wrapper "$tamarin_exec"
    fi
    if ! have dot; then
      warn "Graphviz 'dot' was not found; Tamarin runtime support may be incomplete"
    fi
  else
    warn "Tamarin binary release was not detected; install manually if upstream asset naming changed"
  fi

  local tamarin_exec
  tamarin_exec="$(resolve_tamarin_exec || true)"
  if [[ -n "$tamarin_exec" ]]; then
    write_tamarin_wrapper "$tamarin_exec"
    link_repo_bin "$TOOLS_DIR/tamarin/bin/tamarin-prover-z00z" tamarin-prover
  fi
}

install_maude() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_at_least_recommended || return 0
  if [[ -x "$TOOLS_DIR/maude/bin/maude" && "$FORCE_UPGRADE" -eq 0 ]]; then
    link_repo_bin "$TOOLS_DIR/maude/bin/maude" maude
    log "Already installed: local Maude wrapper"
    return 0
  fi

  local pattern=""
  case "$(uname -s):$(uname -m)" in
    Linux:x86_64)
      pattern='Maude-.*-linux-x86_64\.zip$'
      ;;
    Darwin:arm64)
      pattern='Maude-.*-macos-arm64\.zip$'
      ;;
    Darwin:x86_64)
      pattern='Maude-.*-macos-x86_64\.zip$'
      ;;
    *)
      warn "No official Maude asset pattern configured for $(uname -s):$(uname -m)"
      return 0
      ;;
  esac

  local archive
  if archive="$(download_archive_asset "SRI-CSL/Maude" "$MAUDE_RELEASE" "$pattern" "$TOOLS_DIR/maude")"; then
    unzip -q -o "$archive" -d "$TOOLS_DIR/maude"
    local maude_exec maude_dir wrapper
    maude_exec="$(find "$TOOLS_DIR/maude" -type f -name maude -perm -u+x ! -path "$TOOLS_DIR/maude/bin/*" 2>/dev/null | sort | head -n 1 || true)"
    if [[ -z "$maude_exec" ]]; then
      warn "Maude archive extracted but no executable was found"
      return 0
    fi
    maude_dir="$(dirname "$maude_exec")"
    wrapper="$TOOLS_DIR/maude/bin/maude"
    mkdir -p "$(dirname "$wrapper")"
    cat >"$wrapper" <<EOF
#!/bin/bash
set -euo pipefail

cd "$maude_dir"
exec "$maude_exec" "\$@"
EOF
    chmod +x "$wrapper"
    link_repo_bin "$wrapper" maude
  else
    warn "Maude binary asset not detected; install from https://maude.cs.illinois.edu/get-maude"
  fi
}

ensure_tamarin_runtime() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_at_least_recommended || return 0

  local tamarin_exec local_maude
  tamarin_exec="$(resolve_tamarin_exec || true)"
  [[ -n "$tamarin_exec" ]] || return 0

  local_maude="$(resolve_local_maude_cmd || true)"
  if [[ -n "$local_maude" ]] && tamarin_runtime_is_healthy "$tamarin_exec" "$local_maude"; then
    write_tamarin_wrapper "$tamarin_exec"
    link_repo_bin "$TOOLS_DIR/tamarin/bin/tamarin-prover-z00z" tamarin-prover
    return 0
  fi

  install_maude
  local_maude="$(resolve_local_maude_cmd || true)"
  if [[ -n "$local_maude" ]]; then
    write_tamarin_wrapper "$tamarin_exec"
    link_repo_bin "$TOOLS_DIR/tamarin/bin/tamarin-prover-z00z" tamarin-prover
    if ! tamarin_runtime_is_healthy "$TOOLS_DIR/tamarin/bin/tamarin-prover" "$local_maude"; then
      warn "Tamarin still reports Maude runtime problems after installing local Maude"
    fi
  else
    warn "Tamarin has no compatible Maude runtime; install Maude manually"
  fi
}

install_verus() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_at_least_recommended || return 0
  local existing_verus existing_cargo_verus
  existing_verus="$(first_executable verus "$TOOLS_DIR/verus" || true)"
  existing_cargo_verus="$(first_executable cargo-verus "$TOOLS_DIR/verus" || true)"

  if tool_path_is_local verus && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    ensure_verus_toolchain "$(command -v verus)"
    log "Already installed: verus"
    return 0
  fi
  if [[ -n "$existing_verus" ]]; then
    mkdir -p "$TOOLS_DIR/verus/bin"
    ln -sf "$existing_verus" "$TOOLS_DIR/verus/bin/verus"
    link_repo_bin "$TOOLS_DIR/verus/bin/verus" verus
    if [[ -n "$existing_cargo_verus" ]]; then
      ln -sf "$existing_cargo_verus" "$TOOLS_DIR/verus/bin/cargo-verus"
      link_repo_bin "$TOOLS_DIR/verus/bin/cargo-verus" cargo-verus
    fi
    ensure_verus_toolchain "$TOOLS_DIR/verus/bin/verus"
    log "Re-linked existing Verus checkout"
    return 0
  fi

  local archive
  if archive="$(download_archive_asset "verus-lang/verus" "$VERUS_RELEASE" '.*(linux|x86).*\.(zip|tar\.gz|tgz)$' "$TOOLS_DIR/verus")"; then
    case "$archive" in
      *.zip)
        unzip -q -o "$archive" -d "$TOOLS_DIR/verus"
        ;;
      *.tar.gz|*.tgz)
        tar -xzf "$archive" -C "$TOOLS_DIR/verus"
        ;;
    esac
    link_first_executable verus "$TOOLS_DIR/verus" "$TOOLS_DIR/verus/bin/verus"
    link_first_executable cargo-verus "$TOOLS_DIR/verus" "$TOOLS_DIR/verus/bin/cargo-verus"
    if [[ -x "$TOOLS_DIR/verus/bin/verus" ]]; then
      link_repo_bin "$TOOLS_DIR/verus/bin/verus" verus
      ensure_verus_toolchain "$TOOLS_DIR/verus/bin/verus"
    fi
    if [[ -x "$TOOLS_DIR/verus/bin/cargo-verus" ]]; then
      link_repo_bin "$TOOLS_DIR/verus/bin/cargo-verus" cargo-verus
    fi
    if [[ ! -x "$TOOLS_DIR/verus/bin/verus" ]]; then
      warn "Add the extracted Verus binary directory under $TOOLS_DIR/verus to PATH if it was not linked by the release package"
    fi
  else
    warn "Verus binary asset not detected; follow upstream INSTALL.md"
  fi
}

install_prusti() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_at_least_recommended || return 0
  if tool_path_is_local cargo-prusti && tool_path_is_local prusti-rustc && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already installed: Prusti"
    return 0
  fi

  local archive
  if archive="$(download_archive_asset "viperproject/prusti-dev" "$PRUSTI_RELEASE" 'prusti-release-(ubuntu|linux).*\.(zip|tar\.gz|tgz)$|prusti-release-ubuntu\.zip$' "$TOOLS_DIR/prusti")"; then
    case "$archive" in
      *.zip)
        unzip -q -o "$archive" -d "$TOOLS_DIR/prusti"
        ;;
      *.tar.gz|*.tgz)
        tar -xzf "$archive" -C "$TOOLS_DIR/prusti"
        ;;
    esac
    mkdir -p "$TOOLS_DIR/prusti/bin"
    chmod +x \
      "$TOOLS_DIR/prusti/cargo-prusti" \
      "$TOOLS_DIR/prusti/prusti-rustc" \
      "$TOOLS_DIR/prusti/prusti-driver" \
      "$TOOLS_DIR/prusti/prusti-server" \
      "$TOOLS_DIR/prusti/prusti-server-driver" 2>/dev/null || true
    if [[ -f "$TOOLS_DIR/prusti/cargo-prusti" ]]; then
      ln -sf "$TOOLS_DIR/prusti/cargo-prusti" "$TOOLS_DIR/prusti/bin/cargo-prusti"
    else
      link_first_executable cargo-prusti "$TOOLS_DIR/prusti" "$TOOLS_DIR/prusti/bin/cargo-prusti"
    fi
    if [[ -f "$TOOLS_DIR/prusti/prusti-rustc" ]]; then
      ln -sf "$TOOLS_DIR/prusti/prusti-rustc" "$TOOLS_DIR/prusti/bin/prusti-rustc"
    else
      link_first_executable prusti-rustc "$TOOLS_DIR/prusti" "$TOOLS_DIR/prusti/bin/prusti-rustc"
    fi
    link_repo_bin "$TOOLS_DIR/prusti/bin/cargo-prusti" cargo-prusti
    link_repo_bin "$TOOLS_DIR/prusti/bin/prusti-rustc" prusti-rustc
    log "Installed Prusti wrappers into $TOOLS_DIR/prusti/bin"
  else
    warn "Prusti binary asset not detected; follow upstream command-line setup or VS Code Prusti Assistant"
  fi
}

install_creusot() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_includes_all || return 0
  mkdir -p "$TOOLS_DIR"
  if [[ -d "$TOOLS_DIR/creusot/.git" ]]; then
    log "Updating Creusot checkout"
    git -C "$TOOLS_DIR/creusot" pull --ff-only
  elif packaged_source_snapshot_available "$TOOLS_DIR/creusot" "Cargo.toml"; then
    log "Using packaged Creusot source snapshot"
  else
    log "Cloning Creusot"
    git clone https://github.com/creusot-rs/creusot "$TOOLS_DIR/creusot"
  fi
  mkdir -p "$TOOLS_DIR/creusot/target"
  ensure_path_symlink "$TOOLS_DIR/creusot/target/creusot" "$CREUSOT_TARGET_DIR"
  ensure_path_symlink "$TOOLS_DIR/creusot/target/debug" "$CREUSOT_TARGET_DIR/debug"
  ensure_cargo_target_config "$TOOLS_DIR/creusot" "$CREUSOT_TARGET_DIR"
  if [[ -d "$LOCAL_CREUSOT_DATA_HOME/_opam" ]] && ! OPAMROOT="$LOCAL_OPAM_ROOT" opam switch list --root "$LOCAL_OPAM_ROOT" --short 2>/dev/null | grep -Fxq "$LOCAL_CREUSOT_DATA_HOME"; then
    log "Removing incomplete Creusot local switch"
    safe_trash_path "$LOCAL_CREUSOT_DATA_HOME/_opam"
  fi
  log "Installing Creusot"
  (
    cd "$TOOLS_DIR/creusot" &&
    set -o pipefail
    # creusot-install still surfaces two opam prompts on some fresh switches
    # even with OPAMYES/unsafe-yes. Feed deterministic approvals and strip
    # the echoed prompt lines so unpack logs stay unattended and stable.
    python3 - <<'PY' | \
    CREUSOT_DATA_HOME="$LOCAL_CREUSOT_DATA_HOME" \
    XDG_CONFIG_HOME="$LOCAL_CREUSOT_CONFIG_HOME" \
    XDG_CACHE_HOME="$LOCAL_CREUSOT_CACHE_HOME" \
    CARGO_TARGET_DIR="$CREUSOT_TARGET_DIR" \
    OPAMROOT="$LOCAL_OPAM_ROOT" \
    OPAMYES=1 \
    OPAMCONFIRMLEVEL=unsafe-yes \
    OPAMASSUMEDEPEXTS=1 \
    OPAMNOSELFUPGRADE=1 \
    OPAMNOAUTOUPGRADE=1 \
    external_rust_build_env cargo run --bin creusot-install -- \
      why3 build-prelude prelude provers why3-conf creusot-rustc 2>&1 | \
    sed -u \
      -e '/^Package creusot-deps does not exist, create as a NEW package? \[Y\/n\] y$/d' \
      -e '/^Pin and install them? \[Y\/n\] y$/d'
import sys
for _ in range(64):
    sys.stdout.write("y\n")
PY
  )

  CARGO_TARGET_DIR="$CREUSOT_TARGET_DIR" \
    external_rust_build_env cargo install --locked --root "$LOCAL_CARGO_HOME" --path "$TOOLS_DIR/creusot/cargo-creusot"

  if [[ -x "$LOCAL_CARGO_HOME/bin/cargo-creusot" ]]; then
    wrap_repo_bin_env \
      "$LOCAL_CARGO_HOME/bin/cargo-creusot" \
      cargo-creusot \
      CREUSOT_DATA_HOME "$LOCAL_CREUSOT_DATA_HOME" \
      XDG_CONFIG_HOME "$LOCAL_CREUSOT_CONFIG_HOME" \
      XDG_CACHE_HOME "$LOCAL_CREUSOT_CACHE_HOME" \
      CARGO_TARGET_DIR "$CREUSOT_TARGET_DIR"
  fi

  local creusot_rustc
  creusot_rustc="$(find "$LOCAL_CREUSOT_DATA_HOME/toolchains" -type f -name creusot-rustc -perm -u+x 2>/dev/null | sort | head -n 1 || true)"
  if [[ -n "$creusot_rustc" ]]; then
    wrap_repo_bin_env \
      "$creusot_rustc" \
      creusot-rustc \
      CREUSOT_DATA_HOME "$LOCAL_CREUSOT_DATA_HOME" \
      XDG_CONFIG_HOME "$LOCAL_CREUSOT_CONFIG_HOME" \
      XDG_CACHE_HOME "$LOCAL_CREUSOT_CACHE_HOME" \
      CARGO_TARGET_DIR "$CREUSOT_TARGET_DIR"
  fi
}

install_dudect() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_at_least_recommended || return 0
  mkdir -p "$TOOLS_DIR"
  if [[ -d "$TOOLS_DIR/dudect/.git" ]]; then
    log "Updating dudect checkout"
    git -C "$TOOLS_DIR/dudect" pull --ff-only
  elif packaged_source_snapshot_available "$TOOLS_DIR/dudect" "README.md"; then
    log "Using packaged dudect source snapshot"
  else
    log "Cloning dudect"
    git clone https://github.com/oreparaz/dudect "$TOOLS_DIR/dudect"
  fi
}

install_hax() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_includes_all || return 0

  if tool_path_is_local cargo-hax && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already installed: cargo-hax"
    return 0
  fi

  mkdir -p "$TOOLS_DIR"
  if [[ -d "$TOOLS_DIR/hax/.git" ]]; then
    log "Updating hax checkout"
    git -C "$TOOLS_DIR/hax" pull --ff-only
  elif packaged_source_snapshot_available "$TOOLS_DIR/hax" "setup.sh"; then
    log "Using packaged hax source snapshot"
  else
    log "Cloning hax"
    git clone https://github.com/hacspec/hax "$TOOLS_DIR/hax" || {
      warn "hax checkout failed; install when a concrete hax target is added"
      return 0
    }
  fi

  ensure_cargo_target_config "$TOOLS_DIR/hax" "$HAX_TARGET_DIR"

  if [[ -x "$TOOLS_DIR/hax/setup.sh" ]]; then
    log "Installing hax Rust frontends from source checkout"
    (
      cd "$TOOLS_DIR/hax" &&
      # The OCaml engine bootstrap currently depends on upstream generators that
      # emit unstable build failures and warning storms in portable restore
      # environments. The unpack verification chain only requires cargo-hax plus
      # the preserved source snapshot, so restore those deterministic pieces here.
      for path in driver subcommands ../rust-engine; do
        CARGO_TARGET_DIR="$HAX_TARGET_DIR" \
          external_rust_build_env cargo install --locked --force --root "$LOCAL_CARGO_HOME" --path "cli/$path"
      done
    )
    wrap_repo_bin_env \
      "$LOCAL_CARGO_HOME/bin/cargo-hax" \
      cargo-hax \
      CARGO_TARGET_DIR "$TARGET_ROOT_DIR"
    if [[ -x "$LOCAL_CARGO_HOME/bin/hax-rust-engine" ]]; then
      link_repo_bin "$LOCAL_CARGO_HOME/bin/hax-rust-engine" hax-rust-engine
    fi
    log "Portable restore skips the optional hax-engine OCaml bootstrap; cargo-hax and the preserved hax source tree are ready for later manual engine builds if a concrete HAX extraction target is introduced."
  else
    warn "hax checkout does not expose setup.sh; follow upstream installation notes"
  fi
}

install_mir_json() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_includes_research || return 0

  if tool_path_is_local mir-json && tool_path_is_local cargo-crux-test && tool_path_is_local cargo-saw-build && tool_path_is_local crux-mir && path_available mir_json_rlibs && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already installed: mir-json / Crux-MIR toolchain"
    return 0
  fi

  if saw_suite_has_tool mir-json && saw_suite_has_tool cargo-crux-test && saw_suite_has_tool cargo-saw-build && saw_suite_has_tool crux-mir && [[ -d "$TOOLS_DIR/saw-suite/rlibs" ]] && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    link_saw_suite_bins
    log "Already installed: mir-json / Crux-MIR toolchain"
    return 0
  fi

  local checkout="$TOOLS_DIR/mir-json/src"
  local mir_json_target_dir="$MIR_JSON_TARGET_DIR"
  mkdir -p "$TOOLS_DIR/mir-json" "$TOOLS_DIR/mir-json/bin"

  if [[ -d "$checkout/.git" ]]; then
    log "Updating mir-json checkout"
    git_checkout_ref "$checkout" "$MIR_JSON_REF"
  elif packaged_source_snapshot_available "$checkout" "Cargo.toml"; then
    log "Using packaged mir-json source snapshot"
  else
    log "Cloning mir-json"
    git clone https://github.com/GaloisInc/mir-json.git "$checkout"
    git_checkout_ref "$checkout" "$MIR_JSON_REF"
  fi
  ensure_path_symlink "$TOOLS_DIR/mir-json/target" "$MIR_JSON_TARGET_DIR"
  ensure_cargo_target_config "$TOOLS_DIR/mir-json" "$MIR_JSON_TARGET_DIR"
  ensure_path_symlink "$checkout/target" "$MIR_JSON_TARGET_DIR"
  ensure_path_symlink "$checkout/libs/crucible_proc_macros/target" "$MIR_JSON_TARGET_DIR/extra-libs/crucible_proc_macros"
  ensure_cargo_target_config "$checkout" "$MIR_JSON_TARGET_DIR"

  ensure_standalone_cargo_root "$checkout/Cargo.toml"
  ensure_standalone_top_level_cargo_crates "$checkout/libs"
  rustup toolchain install "$MIR_JSON_TOOLCHAIN" --force --component rustc-dev,rust-src,rustfmt
  local sysroot_backup_dir="$TOOLS_DIR/mir-json/sysroot-rmeta-backup/$MIR_JSON_TOOLCHAIN"
  local -a sysroot_duplicate_crates=()
  read -r -a sysroot_duplicate_crates <<<"$MIR_JSON_SYSROOT_DUP_CRATES"
  normalize_rust_sysroot_duplicate_metadata "$MIR_JSON_TOOLCHAIN" "$sysroot_backup_dir" "${sysroot_duplicate_crates[@]}"
  log "Installing mir-json cargo tools"
  (
    cd "$checkout" &&
    CARGO_TARGET_DIR="$mir_json_target_dir" \
      external_rust_build_env cargo +"$MIR_JSON_TOOLCHAIN" install --locked --force --root "$LOCAL_CARGO_HOME" --path .
  )

  local tool
  for tool in mir-json mir-json-translate-libs cargo-crux-test cargo-saw-build crux-rustc saw-rustc; do
    if [[ -x "$LOCAL_CARGO_HOME/bin/$tool" ]]; then
      link_repo_bin "$LOCAL_CARGO_HOME/bin/$tool" "$tool"
    fi
  done

  if [[ -x "$LOCAL_CARGO_HOME/bin/mir-json-translate-libs" ]]; then
    log "Translating mir-json Rust libraries"
    (
      cd "$checkout" &&
      CARGO_TARGET_DIR="$mir_json_target_dir" \
      PATH="$LOCAL_CARGO_HOME/bin:$PATH" \
        external_rust_build_env "$LOCAL_CARGO_HOME/bin/mir-json-translate-libs"
    )
    if [[ -e "$checkout/rlibs" ]]; then
      ln -sfn "$checkout/rlibs" "$TOOLS_DIR/mir-json/rlibs"
    fi
    if [[ -e "$checkout/rlibs_real" ]]; then
      ln -sfn "$checkout/rlibs_real" "$TOOLS_DIR/mir-json/rlibs_real"
    fi
  fi
}

install_charon() {
  [[ "$SKIP_HEAVY" -eq 0 ]] || return 0
  profile_includes_research || return 0

  if tool_path_is_local charon && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already installed: charon"
    return 0
  fi

  local checkout="$TOOLS_DIR/charon/src"
  local charon_source_bin_dir="$checkout/bin"
  local charon_target_dir="$CHARON_TARGET_DIR"
  local charon_bin="$charon_target_dir/release/charon"
  local charon_driver_bin="$charon_target_dir/release/charon-driver"
  mkdir -p "$TOOLS_DIR/charon" "$TOOLS_DIR/charon/bin"

  if [[ -d "$checkout/.git" ]]; then
    log "Updating Charon checkout"
    git_checkout_ref "$checkout" "$CHARON_REF"
  elif packaged_source_snapshot_available "$checkout" "Makefile" || packaged_source_snapshot_available "$checkout" "charon/Cargo.toml"; then
    log "Using packaged Charon source snapshot"
  elif existing_source_tree_available "$checkout"; then
    log "Using existing Charon source tree"
  else
    log "Cloning Charon"
    git clone https://github.com/AeneasVerif/charon.git "$checkout"
    git_checkout_ref "$checkout" "$CHARON_REF"
  fi
  ensure_path_symlink "$TOOLS_DIR/charon/target" "$CHARON_TARGET_DIR"
  ensure_cargo_target_config "$TOOLS_DIR/charon" "$CHARON_TARGET_DIR"
  ensure_cargo_target_config "$checkout/charon" "$CHARON_TARGET_DIR"

  mkdir -p "$charon_source_bin_dir"
  remove_optional_generated_doc_symlink "$checkout/doc-rust.html"
  remove_optional_generated_doc_symlink "$checkout/doc-ml.html"

  ensure_standalone_cargo_root "$checkout/charon/Cargo.toml"
  ensure_declared_rust_toolchain_components "$checkout" rustfmt rust-src
  log "Building Charon"
  (
    cd "$checkout/charon" &&
    CARGO_TARGET_DIR="$charon_target_dir" \
      external_rust_build_env cargo build --release --bins
  )

  if [[ -x "$charon_driver_bin" ]]; then
    ln -sfn "$charon_driver_bin" "$charon_source_bin_dir/charon-driver"
    ln -sfn "$charon_driver_bin" "$TOOLS_DIR/charon/bin/charon-driver"
  fi

  if [[ -x "$charon_bin" ]]; then
    ln -sfn "$charon_bin" "$charon_source_bin_dir/charon"
    ln -sfn "$charon_bin" "$TOOLS_DIR/charon/bin/charon"
    link_repo_bin "$TOOLS_DIR/charon/bin/charon" charon
    return 0
  fi

  warn "Charon build completed but $charon_bin was not produced"
}

install_aeneas() {
  [[ "$SKIP_HEAVY" -eq 0 && "$SKIP_OPAM" -eq 0 ]] || return 0
  profile_includes_research || return 0
  ensure_aeneas_opam_switch || return 0

  if tool_path_is_local aeneas && [[ "$FORCE_UPGRADE" -eq 0 ]]; then
    log "Already installed: aeneas"
    return 0
  fi

  local checkout="$TOOLS_DIR/aeneas/src"
  local aeneas_charon_pin=""
  mkdir -p "$TOOLS_DIR/aeneas" "$TOOLS_DIR/aeneas/bin"

  if [[ -d "$checkout/.git" ]]; then
    log "Updating Aeneas checkout"
    git_checkout_ref "$checkout" "$AENEAS_REF"
  elif packaged_source_snapshot_available "$checkout" "Makefile"; then
    log "Using packaged Aeneas source snapshot"
  elif existing_source_tree_available "$checkout"; then
    log "Using existing Aeneas source tree"
  else
    log "Cloning Aeneas"
    git clone https://github.com/AeneasVerif/aeneas.git "$checkout"
    git_checkout_ref "$checkout" "$AENEAS_REF"
  fi

  if [[ -f "$checkout/charon-pin" ]]; then
    aeneas_charon_pin="$(sed -e '/^[[:space:]]*#/d' -e '/^[[:space:]]*$/d' "$checkout/charon-pin" | tail -n 1 | tr -d '[:space:]')"
  fi

  # Aeneas warns and skips commit-hash verification when ./charon is a symlink.
  # Materialize a local copy and align it to Aeneas' pinned Charon commit.
  if [[ -e "$checkout/charon" || -L "$checkout/charon" ]]; then
    safe_trash_path "$checkout/charon"
  fi
  mkdir -p "$checkout/charon"
  cp -a "$TOOLS_DIR/charon/src/." "$checkout/charon/"
  if [[ -n "$aeneas_charon_pin" && -d "$checkout/charon/.git" ]]; then
    if ! git -C "$checkout/charon" rev-parse --verify "${aeneas_charon_pin}^{commit}" >/dev/null 2>&1; then
      git -C "$checkout/charon" fetch origin "$aeneas_charon_pin" --quiet
    fi
    git -C "$checkout/charon" checkout --quiet "$aeneas_charon_pin"
  fi
  remove_optional_generated_doc_symlink "$checkout/doc.html"
  ensure_standalone_cargo_root "$checkout/Cargo.toml"

  log "Installing Aeneas OPAM dependencies"
  opam_aeneas_cmd install --root "$LOCAL_OPAM_ROOT" --switch "$LOCAL_AENEAS_OPAM_SWITCH" -y \
    ppx_deriving visitors easy_logging zarith yojson core_unix odoc ocamlgraph menhir ocamlformat.0.27.0 unionFind progress domainslib

  log "Building Aeneas"
  # shellcheck disable=SC2016
  OPAMROOT="$LOCAL_OPAM_ROOT" OPAMYES=1 OPAMCONFIRMLEVEL=unsafe-yes OPAMASSUMEDEPEXTS=1 OPAMNOSELFUPGRADE=1 OPAMNOAUTOUPGRADE=1 LOCAL_CARGO_HOME="$LOCAL_CARGO_HOME" \
    opam exec --root "$LOCAL_OPAM_ROOT" --switch "$LOCAL_AENEAS_OPAM_SWITCH" -- \
    sh -lc 'export PATH="$LOCAL_CARGO_HOME/bin:$PATH"; export CARGO_TERM_COLOR=never; export RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }-Awarnings"; export RUSTDOCFLAGS="${RUSTDOCFLAGS:+$RUSTDOCFLAGS }-Awarnings"; make -C "$1"' sh "$checkout"

  if [[ -x "$checkout/bin/aeneas" ]]; then
    ln -sfn "$checkout/bin/aeneas" "$TOOLS_DIR/aeneas/bin/aeneas"
    link_repo_bin "$TOOLS_DIR/aeneas/bin/aeneas" aeneas
  fi
}

opam_tool_available() {
  local tool="$1"
  shift
  tool_path_is_local "$tool" && return 0
  local_opam_switch_exists || return 1
  opam_exec_local "$tool" "$@" >/dev/null 2>&1
}

first_executable() {
  local name="$1"
  local search_dir="$2"
  find "$search_dir" -type f -name "$name" -perm -u+x 2>/dev/null | sort | head -n 1
}

resolve_verus_toolchain_name() {
  local verus_bin_path="${1:-}"
  local toolchain="${Z00Z_VERUS_TOOLCHAIN:-}"
  local resolved_verus version_json

  if [[ -n "$toolchain" ]]; then
    printf '%s\n' "$toolchain"
    return 0
  fi

  [[ -n "$verus_bin_path" ]] || return 0
  resolved_verus="$(readlink -f "$verus_bin_path" 2>/dev/null || printf '%s\n' "$verus_bin_path")"
  version_json="$(dirname "$resolved_verus")/version.json"
  if [[ -f "$version_json" ]] && have jq; then
    toolchain="$(jq -r '.verus.toolchain // empty' "$version_json" 2>/dev/null || true)"
  fi
  if [[ -z "$toolchain" ]]; then
    toolchain="$("$resolved_verus" --version 2>/dev/null | awk -F': ' '/Toolchain:/ {print $2; exit}' || true)"
  fi
  [[ -n "$toolchain" ]] && printf '%s\n' "$toolchain"
}

ensure_verus_toolchain() {
  local verus_bin_path="${1:-}"
  local required_toolchain
  local install_status=0

  required_toolchain="$(resolve_verus_toolchain_name "$verus_bin_path")"
  [[ -n "$required_toolchain" ]] || return 0

  if rustup toolchain list 2>/dev/null | awk '{print $1}' | grep -Fxq "$required_toolchain"; then
    return 0
  fi

  log "Installing Verus Rust toolchain $required_toolchain into $RUSTUP_HOME"
  set +e
  rustup toolchain install --profile minimal "$required_toolchain"
  install_status=$?
  set -e
  if rustup toolchain list 2>/dev/null | awk '{print $1}' | grep -Fxq "$required_toolchain"; then
    return 0
  fi
  if [[ "$install_status" -ne 0 ]]; then
    warn "could not install required Verus Rust toolchain $required_toolchain"
  fi
}

tool_available() {
  local tool="$1"
  case "$tool" in
    rustup|cargo|rustc|java|javac|jq|node|npm|opam|z3|shellcheck|python3|pipx|uv|make|g++)
      have "$tool"
      ;;
    rg|ruff|markdownlint-cli2|cargo-nextest|cargo-audit|cargo-deny|cargo-vet|cargo-fuzz|cargo-geiger|cargo-kani|kani|cargo-llvm-cov|cargo-semver-checks|just|bacon|watchexec|mdbook|lychee|taplo|maude|tamarin-prover|proverif|why3|easycrypt|verus|cargo-prusti|prusti-rustc|cargo-creusot|cargo-hax|saw|cryptol|mir-json|cargo-saw-build|cargo-crux-test|crux-mir|crux-mir-comp|charon|aeneas|cvc5|bitwuzla)
      tool_path_is_local "$tool"
      ;;
    *)
      have "$tool"
      ;;
  esac
}

path_available() {
  local item="$1"
  case "$item" in
    tla2tools)
      [[ -f "$TOOLS_DIR/tla/tla2tools.jar" ]]
      ;;
    alloy)
      [[ -f "$TOOLS_DIR/alloy/org.alloytools.alloy.dist.jar" ]]
      ;;
    apalache)
      [[ -x "$TOOLS_DIR/apalache/bin/apalache-mc" ]]
      ;;
    dudect)
      [[ -d "$TOOLS_DIR/dudect" ]]
      ;;
    hax)
      [[ -d "$TOOLS_DIR/hax" ]] || tool_available cargo-hax
      ;;
    mir_json_rlibs)
      [[ -d "$TOOLS_DIR/saw-suite/rlibs" || -d "$TOOLS_DIR/mir-json/rlibs" || -d "$TOOLS_DIR/mir-json/src/rlibs" || -d "$TOOLS_DIR/mir-json/src/rlibs_real" ]]
      ;;
    miri_sysroot)
      miri_sysroot_ready
      ;;
    *)
      return 1
      ;;
  esac
}

required_tools() {
  local tools=(
    rustup cargo rustc java javac jq shellcheck make g++ python3
    rg ruff
    cargo-nextest cargo-audit cargo-deny cargo-vet cargo-fuzz cargo-geiger cargo-kani
    cargo-llvm-cov cargo-semver-checks just bacon watchexec mdbook lychee taplo
  )
  if [[ "$SKIP_NODE" -eq 0 ]]; then
    tools+=(node npm markdownlint-cli2)
  fi
  if profile_at_least_recommended && [[ "$SKIP_OPAM" -eq 0 ]]; then
    tools+=(opam z3 proverif why3)
  fi
  if profile_at_least_recommended && [[ "$SKIP_HEAVY" -eq 0 ]]; then
    tools+=(maude tamarin-prover verus cargo-prusti prusti-rustc)
  fi
  if profile_includes_all && [[ "$SKIP_HEAVY" -eq 0 && "$SKIP_OPAM" -eq 0 ]]; then
    tools+=(easycrypt cargo-creusot cargo-hax)
  fi
  if profile_includes_research && [[ "$SKIP_HEAVY" -eq 0 ]]; then
    tools+=(saw cryptol mir-json cargo-saw-build cargo-crux-test crux-mir charon cvc5 bitwuzla)
  fi
  if profile_includes_research && [[ "$SKIP_HEAVY" -eq 0 && "$SKIP_OPAM" -eq 0 ]]; then
    tools+=(aeneas)
  fi
  printf '%s\n' "${tools[@]}"
}

required_paths() {
  local paths=(tla2tools alloy apalache)
  if profile_at_least_recommended && [[ "$SKIP_HEAVY" -eq 0 ]]; then
    paths+=(miri_sysroot)
  fi
  if profile_at_least_recommended && [[ "$SKIP_HEAVY" -eq 0 ]]; then
    paths+=(dudect)
  fi
  if profile_includes_all && [[ "$SKIP_HEAVY" -eq 0 ]]; then
    paths+=(hax)
  fi
  if profile_includes_research && [[ "$SKIP_HEAVY" -eq 0 ]]; then
    paths+=(mir_json_rlibs)
  fi
  printf '%s\n' "${paths[@]}"
}

check_status() {
  local tools=(
    rustup cargo rustc java javac jq node npm opam z3 shellcheck python3 pipx uv make g++ rg ruff markdownlint-cli2
    cargo-nextest cargo-audit cargo-deny cargo-vet cargo-fuzz cargo-geiger cargo-kani
    cargo-llvm-cov cargo-semver-checks just bacon watchexec mdbook lychee taplo
    maude tamarin-prover proverif why3 easycrypt verus cargo-prusti prusti-rustc cargo-creusot cargo-hax
    saw cryptol mir-json cargo-saw-build cargo-crux-test crux-mir crux-mir-comp charon aeneas cvc5 bitwuzla
  )
  local missing_required=0

  for tool in "${tools[@]}"; do
    if tool_available "$tool"; then
      printf 'OK      %s\n' "$tool"
    else
      printf 'MISSING %s\n' "$tool"
    fi
  done
  if nightly_has_component miri; then
    printf 'OK      %s\n' 'rustup+nightly:miri'
  else
    printf 'MISSING %s\n' 'rustup+nightly:miri'
  fi
  path_available miri_sysroot && printf 'OK      %s\n' "$LOCAL_MIRI_SYSROOT" || printf 'MISSING %s\n' "$LOCAL_MIRI_SYSROOT"

  path_available tla2tools && printf 'OK      %s\n' "$TOOLS_DIR/tla/tla2tools.jar" || printf 'MISSING %s\n' "$TOOLS_DIR/tla/tla2tools.jar"
  path_available alloy && printf 'OK      %s\n' "$TOOLS_DIR/alloy/org.alloytools.alloy.dist.jar" || printf 'MISSING %s\n' "$TOOLS_DIR/alloy/org.alloytools.alloy.dist.jar"
  path_available apalache && printf 'OK      %s\n' "$TOOLS_DIR/apalache/bin/apalache-mc" || printf 'MISSING %s\n' "$TOOLS_DIR/apalache/bin/apalache-mc"
  if tool_available maude; then
    printf 'OK      %s\n' "$(resolve_maude_cmd)"
  else
    printf 'MISSING %s\n' "$TOOLS_DIR/maude/bin/maude"
  fi
  path_available dudect && printf 'OK      %s\n' "$TOOLS_DIR/dudect" || printf 'MISSING %s\n' "$TOOLS_DIR/dudect"
  path_available hax && printf 'OK      %s\n' "$TOOLS_DIR/hax" || printf 'MISSING %s\n' "$TOOLS_DIR/hax"
  path_available mir_json_rlibs && printf 'OK      %s\n' "$TOOLS_DIR/mir-json/rlibs" || printf 'MISSING %s\n' "$TOOLS_DIR/mir-json/rlibs"

  if [[ "$STRICT" == "1" ]]; then
    if ! nightly_has_component miri; then
      printf 'ERROR   profile %s requires missing nightly component: miri\n' "$PROFILE" >&2
      missing_required=1
    fi

    while IFS= read -r tool; do
      [[ -n "$tool" ]] || continue
      if ! tool_available "$tool"; then
        printf 'ERROR   profile %s requires missing tool: %s\n' "$PROFILE" "$tool" >&2
        missing_required=1
      fi
    done < <(required_tools)

    while IFS= read -r path_id; do
      [[ -n "$path_id" ]] || continue
      if ! path_available "$path_id"; then
        printf 'ERROR   profile %s requires missing tool asset: %s\n' "$PROFILE" "$path_id" >&2
        missing_required=1
      fi
    done < <(required_paths)
  fi

  return "$missing_required"
}

self_test_tla() {
  local jar="$TOOLS_DIR/tla/tla2tools.jar"
  [[ -f "$jar" ]] || return 0

  local tmp_dir
  tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/z00z-tla-selftest.XXXXXX")"
  cat >"$tmp_dir/Tiny.tla" <<'EOF'
---- MODULE Tiny ----
EXTENDS Naturals
VARIABLE x
Init == x = 0
Next == /\ x < 2
        /\ x' = x + 1
Spec == Init /\ [][Next]_x
Inv == x <= 2
====
EOF
  cat >"$tmp_dir/Tiny.cfg" <<'EOF'
SPECIFICATION Spec
INVARIANT Inv
EOF
  log "TLA+ TLC tiny model"
  local status=0
  java -XX:+UseParallelGC -cp "$jar" tlc2.TLC -deadlock -config "$tmp_dir/Tiny.cfg" "$tmp_dir/Tiny.tla" || status=$?
  safe_trash_path "$tmp_dir"
  return "$status"
}

self_test_warn_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    return 1
  fi
  warn "$message"
  return 0
}

self_test_versions() {
  local first_line
  if [[ -f "$ROOT_DIR/scripts/verify-env.sh" ]]; then
    bash -lc "source '$ROOT_DIR/scripts/verify-env.sh' && command -v rg >/dev/null && command -v cargo >/dev/null"
  fi
  if have rustc; then
    rustc --version
  fi
  if have cargo; then
    cargo --version
  fi
  if have java; then
    java -version
  fi
  if have javac; then
    javac -version
  fi
  if have rg; then
    first_line="$(rg --version)"
    printf '%s\n' "${first_line%%$'\n'*}"
  fi
  if have ruff; then
    ruff --version
  fi
  if have uv; then
    uv --version
  fi
  if have make; then
    first_line="$(make --version)"
    printf '%s\n' "${first_line%%$'\n'*}"
  fi
  if have g++; then
    first_line="$(g++ --version)"
    printf '%s\n' "${first_line%%$'\n'*}"
  fi
  if have cargo-nextest; then
    cargo-nextest --version
  fi
  if have cargo-audit; then
    cargo audit --version
  fi
  if have cargo-deny; then
    cargo deny --version
  fi
  if have cargo-vet; then
    cargo vet --version
  fi
  if have cargo-fuzz; then
    cargo fuzz --help >/dev/null 2>&1 || true
  fi
  if have cargo-geiger; then
    cargo geiger --version
  fi
  if have cargo-kani; then
    cargo-kani --version
  fi
  if have kani; then
    kani --version
  fi
  if nightly_has_component miri; then
    cargo +nightly miri --version
    if miri_sysroot_ready; then
      printf 'Miri sysroot: %s\n' "$LOCAL_MIRI_SYSROOT"
    fi
  fi
  if have markdownlint-cli2; then
    markdownlint-cli2 --version
  fi
  if local_opam_switch_exists; then
    opam_exec_local proverif -help >/dev/null || true
    opam_exec_local why3 --version || true
    opam_exec_local easycrypt config || true
  fi
  if [[ -x "$TOOLS_DIR/opam/bin/proverif" ]]; then
    "$TOOLS_DIR/opam/bin/proverif" -help >/dev/null || true
  fi
  local maude_cmd=""
  maude_cmd="$(resolve_maude_cmd || true)"
  if [[ -n "$maude_cmd" ]]; then
    "$maude_cmd" --version || true
  fi
  if have tamarin-prover; then
    tamarin-prover --version
  elif [[ -n "$(resolve_tamarin_cmd || true)" ]]; then
    "$(resolve_tamarin_cmd)" --version || true
  fi
  if have verus; then
    ensure_verus_toolchain "$(command -v verus)"
    verus --version
  elif [[ -n "$(first_executable verus "$TOOLS_DIR/verus")" ]]; then
    ensure_verus_toolchain "$(first_executable verus "$TOOLS_DIR/verus")"
    "$(first_executable verus "$TOOLS_DIR/verus")" --version || true
  fi
  if tool_available cargo-prusti; then
    if tool_path_is_local cargo-prusti; then
      cargo-prusti --help >/dev/null || true
    else
      "$(first_executable cargo-prusti "$TOOLS_DIR/prusti")" --help >/dev/null || true
    fi
  fi
  if tool_available prusti-rustc; then
    if tool_path_is_local prusti-rustc; then
      prusti-rustc --help >/dev/null || true
    else
      "$(first_executable prusti-rustc "$TOOLS_DIR/prusti")" --help >/dev/null || true
    fi
  fi
  if tool_available cargo-hax; then
    cargo-hax --version || true
  fi
  if tool_available saw; then
    saw --version
  fi
  if tool_available cryptol; then
    cryptol --version
  fi
  if tool_available mir-json; then
    mir-json --version
  fi
  if tool_available cargo-saw-build; then
    cargo saw-build --help >/dev/null 2>&1 || true
  fi
  if tool_available cargo-crux-test; then
    cargo crux-test --help >/dev/null 2>&1 || true
  fi
  if tool_available crux-mir; then
    crux-mir --help >/dev/null 2>&1 || true
  fi
  if tool_available charon; then
    charon --help >/dev/null || true
  fi
  if tool_available aeneas; then
    aeneas --help >/dev/null || true
  fi
  if tool_available cvc5; then
    cvc5 --version
  fi
  if tool_available bitwuzla; then
    bitwuzla --version || true
  fi
}

self_test_repo_scripts() {
  if have ruff; then
    log "ruff fatal-error sanity for verifier scripts"
    ruff check --select E9,F63,F7,F82 \
      "$ROOT_DIR/scripts" \
      "$ROOT_DIR/.github/skills/z00z-"*"/scripts" \
      "$ROOT_DIR/.github/skills/skill-selector/scripts" >/dev/null
  fi

  if have shellcheck; then
    log "shellcheck sanity for verifier shell scripts"
    shellcheck -x -e SC1090,SC2016 \
      "$ROOT_DIR/scripts/install-verification-tools.sh" \
      "$ROOT_DIR/scripts/verify-env.sh" \
      "$ROOT_DIR/.github/skills/z00z-code-to-logic-gate/scripts/"*.sh \
      "$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/"*.sh \
      "$ROOT_DIR/.github/skills/z00z-l1-protocol-model-gate/scripts/"*.sh \
      "$ROOT_DIR/.github/skills/z00z-l2-crypto-protocol-gate/scripts/"*.sh \
      "$ROOT_DIR/.github/skills/z00z-l3-rust-implementation-gate/scripts/"*.sh \
      "$ROOT_DIR/.github/skills/z00z-l4-security-engineering-gate/scripts/"*.sh >/dev/null
  fi

  if have python3; then
    local venv_tmp
    venv_tmp="$(mktemp -d "${TMPDIR:-/tmp}/z00z-venv-selftest.XXXXXX")"
    python3 -m venv "$venv_tmp"
    safe_trash_path "$venv_tmp"
  fi
}

self_test_checksums() {
  local manifest asset_file status=0
  manifest="$(checksum_manifest_path)"
  [[ -f "$manifest" ]] || return 0

  while IFS= read -r asset_file; do
    [[ -n "$asset_file" ]] || continue
    verify_download_checksum "$asset_file" || status=1
  done < <(find "$TOOLS_DIR" -maxdepth 2 -type f \( -name '*.zip' -o -name '*.tar.gz' -o -name '*.tgz' -o -name '*.jar' \) | sort)

  return "$status"
}

self_test_code_to_logic_suite() {
  if ! tool_available saw || ! tool_available cargo-saw-build || ! tool_available cargo-crux-test || ! tool_available crux-mir || ! path_available mir_json_rlibs; then
    return 0
  fi

  local tmp_dir saw_log crux_log
  tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/z00z-code-logic-selftest.XXXXXX")"
  saw_log="$(mktemp "${TMPDIR:-/tmp}/z00z-code-logic-saw.XXXXXX")"
  crux_log="$(mktemp "${TMPDIR:-/tmp}/z00z-code-logic-crux.XXXXXX")"

  mkdir -p "$tmp_dir/src"
  cat >"$tmp_dir/Cargo.toml" <<'EOF'
[package]
name = "z00z_code_logic_selftest"
version = "0.1.0"
edition = "2021"

[workspace]

[lints.rust]
unexpected_cfgs = { level = "allow", check-cfg = ['cfg(crux)'] }
EOF

  cat >"$tmp_dir/src/lib.rs" <<'EOF'
pub fn serial_roundtrip(value: u32) -> u32 {
    u32::from_le_bytes(value.to_le_bytes())
}

#[cfg(crux)]
#[allow(dead_code)]
mod crux_test {
    extern crate crucible;

    use super::*;
    use crucible::*;

    #[crux::test]
    fn serial_roundtrip_holds() {
        let value = u32::symbolic("value");
        crucible_assert!(serial_roundtrip(value) == value);
    }
}
EOF

  cat >"$tmp_dir/selftest.saw" <<'EOF'
m <- mir_load_module "z00z_code_logic_selftest.linked-mir.json";

let serial_roundtrip_spec = do {
  x <- mir_fresh_var "x" mir_u32;
  mir_execute_func [mir_term x];
  mir_return (mir_term x);
};

mir_verify m "z00z_code_logic_selftest::serial_roundtrip" [] false serial_roundtrip_spec z3;
EOF

  log "SAW/Crux-MIR integrated code-to-logic smoke"
  (
    cd "$tmp_dir" &&
    cargo saw-build --release --lib >"$saw_log" 2>&1
  ) || {
    cat "$saw_log" >&2
    safe_trash_path "$tmp_dir"
    safe_trash_path "$saw_log"
    safe_trash_path "$crux_log"
    self_test_warn_or_fail "cargo saw-build smoke failed"
    return 0
  }
  if grep -qi "warning:" "$saw_log"; then
    cat "$saw_log" >&2
    safe_trash_path "$tmp_dir"
    safe_trash_path "$saw_log"
    safe_trash_path "$crux_log"
    self_test_warn_or_fail "cargo saw-build smoke emitted warnings"
    return 0
  fi

  local linked_json
  linked_json="$(find "$tmp_dir/target" -type f -name '*.linked-mir.json' | sort | head -n 1 || true)"
  if [[ -z "$linked_json" ]]; then
    safe_trash_path "$tmp_dir"
    safe_trash_path "$saw_log"
    safe_trash_path "$crux_log"
    self_test_warn_or_fail "cargo saw-build smoke did not emit linked MIR JSON"
    return 0
  fi
  cp "$linked_json" "$tmp_dir/z00z_code_logic_selftest.linked-mir.json"

  (
    cd "$tmp_dir" &&
    saw selftest.saw >>"$saw_log" 2>&1
  ) || {
    cat "$saw_log" >&2
    safe_trash_path "$tmp_dir"
    safe_trash_path "$saw_log"
    safe_trash_path "$crux_log"
    self_test_warn_or_fail "saw smoke proof failed"
    return 0
  }

  (
    cd "$tmp_dir" &&
    cargo crux-test --release --lib >"$crux_log" 2>&1
  ) || {
    cat "$crux_log" >&2
    safe_trash_path "$tmp_dir"
    safe_trash_path "$saw_log"
    safe_trash_path "$crux_log"
    self_test_warn_or_fail "cargo crux-test smoke failed"
    return 0
  }
  if grep -qi "warning:" "$crux_log"; then
    cat "$crux_log" >&2
    safe_trash_path "$tmp_dir"
    safe_trash_path "$saw_log"
    safe_trash_path "$crux_log"
    self_test_warn_or_fail "cargo crux-test smoke emitted warnings"
    return 0
  fi

  safe_trash_path "$tmp_dir"
  safe_trash_path "$saw_log"
  safe_trash_path "$crux_log"
}

self_test_formal_tools() {
  self_test_tla
  if tool_available kani; then
    local kani_tmp
    kani_tmp="$(mktemp -d "${TMPDIR:-/tmp}/z00z-kani-selftest.XXXXXX")"
    cat >"$kani_tmp/tiny.rs" <<'EOF'
#[kani::proof]
fn main() {
    assert!(1 + 1 == 2);
}
EOF
    log "Kani single-file smoke"
    kani --output-format terse "$kani_tmp/tiny.rs"
    safe_trash_path "$kani_tmp"
  fi
  if nightly_has_component miri; then
    ensure_miri_sysroot
    local miri_tmp
    miri_tmp="$(mktemp -d "${TMPDIR:-/tmp}/z00z-miri-selftest.XXXXXX")"
    cat >"$miri_tmp/Cargo.toml" <<'EOF'
[package]
name = "z00z_miri_smoke"
version = "0.1.0"
edition = "2021"

[workspace]
members = []
EOF
    mkdir -p "$miri_tmp/src"
    cat >"$miri_tmp/src/lib.rs" <<'EOF'
#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let bytes = [1_u8, 2, 3];
        assert_eq!(bytes.iter().copied().sum::<u8>(), 6);
    }
}
EOF
    log "Miri smoke test"
    (
      cd "$miri_tmp" &&
      TMPDIR="${TMPDIR:-/tmp}" MIRIFLAGS="-Zmiri-disable-isolation" MIRI_SYSROOT="$LOCAL_MIRI_SYSROOT" \
        cargo +nightly miri test --manifest-path "$miri_tmp/Cargo.toml" --lib
    )
    safe_trash_path "$miri_tmp"
  fi
  if [[ -x "$TOOLS_DIR/alloy/bin/alloy-headless-z00z" ]]; then
    log "Alloy headless runner smoke check"
    "$TOOLS_DIR/alloy/bin/alloy-headless-z00z" --help >/dev/null
  elif [[ -f "$TOOLS_DIR/alloy/org.alloytools.alloy.dist.jar" ]]; then
    log "Alloy jar smoke check"
    timeout 10 java -jar "$TOOLS_DIR/alloy/org.alloytools.alloy.dist.jar" --help >/dev/null 2>&1 || warn "Alloy jar help check is GUI-dependent or timed out"
  fi
  if [[ -x "$TOOLS_DIR/apalache/bin/apalache-mc" ]]; then
    "$TOOLS_DIR/apalache/bin/apalache-mc" version || true
  elif [[ -n "$(first_executable apalache-mc "$TOOLS_DIR/apalache")" ]]; then
    "$(first_executable apalache-mc "$TOOLS_DIR/apalache")" version || true
  elif have apalache-mc; then
    apalache-mc version || true
  fi
  local tamarin_cmd=""
  local maude_cmd=""
  tamarin_cmd="$(resolve_tamarin_cmd || true)"
  maude_cmd="$(resolve_maude_cmd || true)"
  if [[ -n "$tamarin_cmd" ]]; then
    if [[ -z "$maude_cmd" ]]; then
      self_test_warn_or_fail "maude is not installed; tamarin runtime test cannot run"
    else
      PATH="$(dirname "$maude_cmd"):$PATH" "$tamarin_cmd" test
    fi
  fi
  if [[ -d "$TOOLS_DIR/dudect" && -f "$TOOLS_DIR/dudect/Makefile" ]]; then
    local build_log leak_log ct_log leak_status=0 ct_status=0
    build_log="$(mktemp "${TMPDIR:-/tmp}/z00z-dudect-build.XXXXXX")"
    leak_log="$(mktemp "${TMPDIR:-/tmp}/z00z-dudect-leak.XXXXXX")"
    ct_log="$(mktemp "${TMPDIR:-/tmp}/z00z-dudect-ct.XXXXXX")"
    log "dudect build + smoke"
    if ! make -C "$TOOLS_DIR/dudect" all >"$build_log" 2>&1; then
      cat "$build_log" >&2
      safe_trash_path "$build_log"
      safe_trash_path "$leak_log"
      safe_trash_path "$ct_log"
      self_test_warn_or_fail "dudect build failed"
      return 0
    fi
    if grep -qi "warning:" "$build_log"; then
      cat "$build_log" >&2
      safe_trash_path "$build_log"
      safe_trash_path "$leak_log"
      safe_trash_path "$ct_log"
      self_test_warn_or_fail "dudect build emitted warnings"
      return 0
    fi
    (cd "$TOOLS_DIR/dudect" && timeout 20 ./dudect_aes32_O2 >"$leak_log" 2>&1) || leak_status=$?
    (cd "$TOOLS_DIR/dudect" && timeout 10 ./dudect_simple_O2 >"$ct_log" 2>&1) || ct_status=$?
    if [[ "$leak_status" -ne 0 ]]; then
      if [[ "$leak_status" -eq 124 ]] && grep -Eq 'Probably not constant time|Definitely not constant time' "$leak_log"; then
        leak_status=0
      fi
    fi
    if [[ "$leak_status" -ne 0 ]]; then
      tail -n 20 "$leak_log" >&2 || true
      safe_trash_path "$build_log"
      safe_trash_path "$leak_log"
      safe_trash_path "$ct_log"
      self_test_warn_or_fail "dudect leaky sample did not terminate as expected"
      return 0
    fi
    if [[ "$ct_status" -ne 124 ]]; then
      tail -n 20 "$ct_log" >&2 || true
      safe_trash_path "$build_log"
      safe_trash_path "$leak_log"
      safe_trash_path "$ct_log"
      self_test_warn_or_fail "dudect constant-time sample did not survive timeout as expected"
      return 0
    fi
    safe_trash_path "$build_log"
    safe_trash_path "$leak_log"
    safe_trash_path "$ct_log"
  fi
  self_test_code_to_logic_suite
  self_test_repo_scripts
  self_test_checksums
}

run_install() {
  stage_repo_config_files
  install_system_deps
  ensure_rustup
  ensure_miri_sysroot
  install_ripgrep
  install_cargo_tools
  install_kani
  install_formal_jars
  install_python_tools
  install_node_tools

  if profile_at_least_recommended; then
    install_opam_core_tools
    ensure_opam_wrappers
    install_tamarin
    ensure_tamarin_runtime
    install_verus
    install_prusti
    install_dudect
  fi

  install_easycrypt
  install_creusot
  install_hax
  install_saw_suite
  install_saw
  install_cryptol
  install_cvc5
  install_bitwuzla
  install_mir_json
  install_charon
  install_aeneas
  link_saw_suite_bins
  ensure_opam_wrappers
}

case "$ACTION" in
  install)
    run_install
    check_status
    ;;
  check)
    check_status
    ;;
  self-test)
    if have rustup; then
      ensure_nightly_miri_component
      ensure_miri_sysroot
    fi
    check_status
    self_test_versions
    self_test_formal_tools
    ;;
esac
