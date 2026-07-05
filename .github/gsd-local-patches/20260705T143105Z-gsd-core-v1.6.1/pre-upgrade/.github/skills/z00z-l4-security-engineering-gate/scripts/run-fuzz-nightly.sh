#!/bin/bash

# Run longer fuzz sessions for generated Z00Z fuzz targets.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
TIME_SECS="${Z00Z_FUZZ_NIGHTLY_SECS:-300}"

cd "$ROOT_DIR"

env Z00Z_FUZZ_TIME_SECS="$TIME_SECS" \
  "$SCRIPT_DIR/run-fuzz-short.sh"
