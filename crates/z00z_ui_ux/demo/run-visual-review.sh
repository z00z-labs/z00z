#!/usr/bin/env bash
set -euo pipefail

demo_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(git -C "$demo_dir" rev-parse --show-toplevel)"
output_dir="$repo_root/crates/z00z_storage/outputs/checkpoint/phase-110/ui-help-review"

cd "$demo_dir"
node scripts/check-locales.mjs
node scripts/check-help.mjs
mkdir -p "$output_dir"
find "$output_dir" -maxdepth 1 -type f -name '*.png' -delete

server_port="$(python3 - <<'PY'
import socket

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as socket_:
    socket_.bind(("127.0.0.1", 0))
    print(socket_.getsockname()[1])
PY
)"
python3 -m http.server "$server_port" --bind 127.0.0.1 --directory "$demo_dir" >"$output_dir/http-server.log" 2>&1 &
server_pid="$!"

cleanup() {
  kill "$server_pid" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

for _ in {1..50}; do
  if curl --fail --silent "http://127.0.0.1:$server_port/index.html" >/dev/null 2>&1; then
    Z00Z_WALLET_DEMO_URL="http://127.0.0.1:$server_port/index.html" \
      npx --yes --package @playwright/test \
      -c 'export NODE_PATH="$(dirname "$(dirname "$(command -v playwright)")")"; playwright test visual-review.spec.js --workers=1 --reporter=line'
    exit 0
  fi
  sleep 0.1
done

echo "Demo HTTP server did not start; see $output_dir/http-server.log" >&2
exit 1
