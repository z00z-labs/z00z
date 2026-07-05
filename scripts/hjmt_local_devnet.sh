#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

profile=""
timeout_secs="30"
smoke="0"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      profile="${2:-}"
      shift 2
      ;;
    --timeout)
      timeout_secs="${2:-}"
      shift 2
      ;;
    --smoke)
      smoke="1"
      shift
      ;;
    *)
      echo "unsupported argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ "$profile" != "sim_5a7s" ]]; then
  echo "--profile must stay sim_5a7s" >&2
  exit 2
fi

if [[ "$smoke" != "1" ]]; then
  echo "--smoke is required for the local devnet harness" >&2
  exit 2
fi

if ! [[ "$timeout_secs" =~ ^[0-9]+$ ]] || [[ "$timeout_secs" -le 0 ]]; then
  echo "--timeout must be a positive integer number of seconds" >&2
  exit 2
fi

run_id="${Z00Z_HJMT_LOCAL_DEVNET_RUN_ID:-sim-5a7s-$(date -u +%Y%m%dT%H%M%SZ)}"
artifact_root="${Z00Z_HJMT_LOCAL_DEVNET_ARTIFACT_ROOT:-$ROOT_DIR/reports/hjmt-local-devnet/$run_id}"
process_root="$artifact_root/process"
scenario11_root="$artifact_root/scenario11"

mkdir -p "$process_root" "$scenario11_root"

(
  cd "$ROOT_DIR"
  export RUST_TEST_THREADS=1
  export Z00Z_HJMT_DEVNET_ARTIFACT_ROOT="$process_root"
  export Z00Z_HJMT_DEVNET_TIMEOUT_SECS="$timeout_secs"
  cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_process_devnet sim_5a7s_process_devnet_smoke -- --exact --nocapture

  export Z00Z_HJMT_SCENARIO11_ARTIFACT_ROOT="$scenario11_root"
  cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 scenario11_process_devnet_fault_contract -- --exact --nocapture
)

smoke_report="$process_root/process-devnet-smoke.json"
fault_matrix="$scenario11_root/process-devnet-contract/scenario_11/quorum/fault_matrix.json"
honesty_report="$scenario11_root/process-devnet-contract/scenario_11/quorum/report_honesty.json"
summary_report="$artifact_root/process-devnet-evidence.json"

for required in "$smoke_report" "$fault_matrix" "$honesty_report"; do
  if [[ ! -f "$required" ]]; then
    echo "missing expected evidence file: $required" >&2
    exit 1
  fi
done

cat > "$summary_report" <<EOF
{
  "profile": "SIM-5A7S",
  "claim_level": "local simulated-full",
  "process_model": "os_process",
  "run_id": "$run_id",
  "smoke_report": "${smoke_report#$ROOT_DIR/}",
  "scenario11_fault_matrix": "${fault_matrix#$ROOT_DIR/}",
  "scenario11_honesty": "${honesty_report#$ROOT_DIR/}",
  "docker_compose": "docker/compose.hjmt-local.yaml",
  "note": "OS-process identity, restart, and cleanup are live through test_hjmt_process_devnet; quorum, partition, and post-quorum restart truth stay bound to scenario_11 deterministic evidence."
}
EOF

echo "$summary_report"
