#!/bin/bash

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SKILL_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

CONFIG_PATH="${SKILL_DIR}/promptfooconfig.yaml"
PROVIDER="${TOKEN_DISCIPLINE_PROVIDER:-file://${SCRIPT_DIR}/promptfoo_local_provider.mjs}"
PROMPTFOO_BIN="${PROMPTFOO_BIN:-$(command -v promptfoo || true)}"
NODE_RUNNER=(npx -y node@22.22.0 --no-warnings)

if [[ -z "${PROMPTFOO_BIN}" ]]; then
  echo 'Error: promptfoo is required' >&2
  exit 1
fi

if [[ ! -f "${CONFIG_PATH}" ]]; then
  echo "Error: missing config ${CONFIG_PATH}" >&2
  exit 1
fi

echo 'Running token-discipline promptfoo eval'
"${NODE_RUNNER[@]}" "${PROMPTFOO_BIN}" eval -c "${CONFIG_PATH}" -r "${PROVIDER}" --no-table "$@"