#!/bin/bash

# Minimize a cargo-fuzz crash for an existing target.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"

usage() {
  cat <<'EOF'
Usage: minimize-crash.sh <target> <crash-file>

Finds the fuzz directory containing <target> and runs cargo fuzz tmin.
EOF
}

if [[ $# -ne 2 ]]; then
  usage
  exit 1
fi

TARGET="$1"
CRASH_FILE="$2"

cd "$ROOT_DIR"

if [[ ! -f "$CRASH_FILE" ]]; then
  echo "ERROR: crash file not found: $CRASH_FILE" >&2
  exit 1
fi

if ! cargo +nightly fuzz --help >/dev/null 2>&1; then
  echo "ERROR: cargo-fuzz or nightly toolchain is not installed" >&2
  exit 1
fi

mapfile -t matches < <(find . -path './target' -prune -o -path './crates/*/target' -prune -o -path "*/fuzz/fuzz_targets/$TARGET.rs" -print | sort)
if [[ "${#matches[@]}" -eq 0 ]]; then
  echo "ERROR: fuzz target not found: $TARGET" >&2
  exit 1
fi

fuzz_dir="$(dirname "$(dirname "${matches[0]}")")"
(
  cd "$fuzz_dir"
  cargo +nightly fuzz tmin "$TARGET" "$ROOT_DIR/$CRASH_FILE"
)
