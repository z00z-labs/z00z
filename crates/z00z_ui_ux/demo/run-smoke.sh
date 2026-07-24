#!/usr/bin/env bash
set -euo pipefail

demo_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(git -C "$demo_dir" rev-parse --show-toplevel)"
output_dir="$repo_root/crates/z00z_storage/outputs/checkpoint/phase-110/playwright"

mkdir -p "$output_dir"
cd "$demo_dir"
node scripts/check-locales.mjs
node scripts/test-help-sync.mjs
node scripts/compile-help.mjs
node scripts/check-help.mjs
node scripts/test-port-contracts.mjs
node scripts/check-port-readiness.mjs

server_port="$(python3 - <<'PY'
import socket

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as socket_:
    socket_.bind(("127.0.0.1", 0))
    print(socket_.getsockname()[1])
PY
)"
server_log="$output_dir/http-server.log"
python3 -m http.server "$server_port" --bind 127.0.0.1 --directory "$demo_dir" >"$server_log" 2>&1 &
server_pid="$!"

cleanup() {
  kill "$server_pid" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

server_ready=false
for _ in {1..50}; do
  if curl --fail --silent "http://127.0.0.1:$server_port/index.html" >/dev/null 2>&1; then
    server_ready=true
    break
  fi
  sleep 0.1
done

if [[ "$server_ready" != "true" ]]; then
  echo "Demo HTTP server did not start; see $server_log" >&2
  exit 1
fi

Z00Z_PLAYWRIGHT_OUTPUT_DIR="$output_dir" \
Z00Z_WALLET_DEMO_URL="http://127.0.0.1:$server_port/index.html" \
  npx --yes --package @playwright/test \
  -c 'export NODE_PATH="$(dirname "$(dirname "$(command -v playwright)")")"; playwright test smoke.spec.js --workers=1 --reporter=line --output="$Z00Z_PLAYWRIGHT_OUTPUT_DIR"'
