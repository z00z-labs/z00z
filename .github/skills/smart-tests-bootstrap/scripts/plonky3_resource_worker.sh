#!/usr/bin/env bash
# Fail-closed Plan-069 Plonky3 prover isolation and resource evidence.

set -euo pipefail

SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
readonly SCRIPT_PATH
ROOT_DIR="$(cd "$(dirname "$SCRIPT_PATH")/../../../.." && pwd)"
readonly ROOT_DIR
readonly EVIDENCE_ROOT="$ROOT_DIR/crates/z00z_storage/outputs/checkpoint/069-07/task-1/resource-worker"
readonly BLOCK_ROOT="$EVIDENCE_ROOT/blocked-command-digests"
readonly CHUNK_CACHE_ROOT="$EVIDENCE_ROOT/chunk-proof-cache-v2"
readonly TEST_TARGET="test_recursive_v2_plonky3_base"
readonly SOURCE_DIAGNOSTIC_TEST="test_real_source_batch_stark_roundtrip"
readonly SOURCE_DIAGNOSTIC_FILTER="checkpoint::plonky3::tests::$SOURCE_DIAGNOSTIC_TEST"
readonly HASH_DIAGNOSTIC_TEST="test_real_hash_chunk_batch_stark_roundtrip"
readonly HASH_DIAGNOSTIC_FILTER="checkpoint::plonky3::tests::$HASH_DIAGNOSTIC_TEST"
readonly AGGREGATION_DIAGNOSTIC_TEST="test_real_recursive_aggregation_wave_roundtrip"
readonly AGGREGATION_DIAGNOSTIC_FILTER="checkpoint::plonky3::tests::$AGGREGATION_DIAGNOSTIC_TEST"
readonly MEMORY_HIGH_BYTES=17179869184
readonly MEMORY_MAX_BYTES=25769803776
readonly MEMORY_TARGET_BYTES=17179869184
readonly SMOKE_MEMORY_HIGH_BYTES=5368709120
readonly SMOKE_MEMORY_MAX_BYTES=6442450944
readonly SMOKE_PROCESS_BYTES=4294967296
readonly RUNTIME_SECONDS=7200
readonly STAGE_STALL_SECONDS=900
readonly SMOKE_RUNTIME_SECONDS=3600
readonly STATUS_POLL_SECONDS=5
readonly SCHEMA="z00z.plonky3.resource-evidence.v1"
readonly ISOLATION_SCHEMA="z00z.plonky3.resource-isolation.v1"
readonly RUN_STATE_SCHEMA="z00z.plonky3.detached-run-state.v1"
readonly TEST_SOURCE="crates/z00z_storage/tests/test_recursive_v2_plonky3_base.rs"
readonly BACKEND_SOURCE="crates/z00z_storage/src/checkpoint/plonky3.rs"
readonly RECURSION_SOURCE="crates/z00z_storage/src/checkpoint/plonky3_recursion.rs"
readonly BINARY_MMCS_SOURCE="crates/z00z_storage/src/checkpoint/plonky3_binary_mmcs.rs"
readonly BINARY_HASH_SOURCE="crates/z00z_storage/src/checkpoint/plonky3_binary_hash.rs"
readonly U16_RANGE_SOURCE="crates/z00z_storage/src/checkpoint/plonky3_u16_range.rs"
readonly AUTHORITY_SOURCE="crates/z00z_storage/src/checkpoint/authority_artifacts.rs"
readonly NOVA_SOURCE="crates/z00z_storage/src/checkpoint/nova.rs"
readonly BOOTSTRAP_SCRIPT=".github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh"
readonly NOVA_GATE_SCRIPT=".github/skills/smart-tests-bootstrap/scripts/nova_milestone_tests.sh"
readonly NOVA_RSS_SCRIPT=".github/skills/smart-tests-bootstrap/scripts/nova_verifier_rss_measurement.sh"

STARTED_RUN_DIR=""
ACTIVE_INTERNAL_RUN_DIR=""
ACTIVE_INTERNAL_EXIT_REASON=""

usage() {
    printf 'usage: %s --preflight | --bootstrap | --start <exact-test-name> | --run <exact-test-name> | --status <run-dir> | --status-latest <exact-test-name>\n' "${0##*/}"
}

die() {
    printf 'plonky3 resource worker: %s\n' "$*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

sha256_text() {
    sha256sum | awk '{print $1}'
}

file_digest() {
    local file="$1"
    [[ -f "$file" ]] || die "required digest input missing: $file"
    sha256sum "$file" | awk '{print $1}'
}

checkpoint_authority_digests() {
    local file
    while IFS= read -r -d '' file; do
        sha256sum "$file"
    done < <(
        find crates/z00z_storage/src/checkpoint -maxdepth 1 -type f \
            \( -name '*.rs' -o -name '*.yaml' \) -print0 |
            sort -z
    )
}

source_digest() {
    {
        checkpoint_authority_digests
        sha256sum "$TEST_SOURCE"
        sha256sum Cargo.toml
        sha256sum Cargo.lock
        sha256sum crates/z00z_storage/Cargo.toml
    } | sha256_text
}

backend_digest() {
    {
        checkpoint_authority_digests
        sha256sum Cargo.toml
        sha256sum Cargo.lock
        sha256sum crates/z00z_storage/Cargo.toml
    } | sha256_text
}

bootstrap_source_digest() {
    {
        checkpoint_authority_digests
        sha256sum "$BOOTSTRAP_SCRIPT"
        sha256sum "$NOVA_GATE_SCRIPT"
        sha256sum "$NOVA_RSS_SCRIPT"
        sha256sum Cargo.toml
        sha256sum Cargo.lock
        sha256sum crates/z00z_storage/Cargo.toml
    } | sha256_text
}

previous_bootstrap_digest() {
    local record
    while IFS= read -r -d '' record; do
        jq -r '
            if (
                (.command.kind? == "bootstrap")
                or ((.command.text? // "") | contains("bootstrap"))
            ) then
                .command.digest? // empty
            else
                empty
            end
        ' "$record"
    done < <(find "$EVIDENCE_ROOT" -type f -name resource-evidence.json -print0 2>/dev/null |
        sort -z) | tail -n 1
}

previous_test_digest() {
    local test_name="$1" record
    while IFS= read -r -d '' record; do
        jq -r --arg test_name "$test_name" '
            if .command.test? == $test_name then
                .command.digest? // empty
            else
                empty
            end
        ' "$record"
    done < <(find "$EVIDENCE_ROOT" -type f -name resource-evidence.json -print0 2>/dev/null |
        sort -z) | tail -n 1
}

is_named_test() {
    case "$1" in
        predicate_differential | transcript_and_actual_proof_mutations_reject | \
            "$SOURCE_DIAGNOSTIC_TEST" | "$HASH_DIAGNOSTIC_TEST" | \
            "$AGGREGATION_DIAGNOSTIC_TEST")
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

is_source_diagnostic_test() {
    [[ "$1" == "$SOURCE_DIAGNOSTIC_TEST" ]]
}

is_hash_diagnostic_test() {
    [[ "$1" == "$HASH_DIAGNOSTIC_TEST" ]]
}

is_aggregation_diagnostic_test() {
    [[ "$1" == "$AGGREGATION_DIAGNOSTIC_TEST" ]]
}

is_lib_diagnostic_test() {
    is_source_diagnostic_test "$1" ||
        is_hash_diagnostic_test "$1" ||
        is_aggregation_diagnostic_test "$1"
}

lib_diagnostic_filter() {
    if is_source_diagnostic_test "$1"; then
        printf '%s\n' "$SOURCE_DIAGNOSTIC_FILTER"
    elif is_hash_diagnostic_test "$1"; then
        printf '%s\n' "$HASH_DIAGNOSTIC_FILTER"
    elif is_aggregation_diagnostic_test "$1"; then
        printf '%s\n' "$AGGREGATION_DIAGNOSTIC_FILTER"
    else
        return 1
    fi
}

named_test_target() {
    if is_lib_diagnostic_test "$1"; then
        printf 'lib\n'
    else
        printf '%s\n' "$TEST_TARGET"
    fi
}

current_cgroup() {
    sed -nE 's/^0::(.*)$/\1/p' /proc/self/cgroup
}

cgroup_file() {
    local cgroup="$1" name="$2"
    printf '/sys/fs/cgroup%s/%s\n' "$cgroup" "$name"
}

read_cgroup_value() {
    local cgroup="$1" name="$2" path
    path="$(cgroup_file "$cgroup" "$name")"
    [[ -r "$path" ]] || return 1
    tr -d '[:space:]' <"$path"
}

cgroup_max_rss_kib() {
    local cgroup="$1" pid rss max_rss=0
    while IFS= read -r pid; do
        [[ "$pid" =~ ^[0-9]+$ && -r "/proc/$pid/status" ]] || continue
        rss="$(awk '$1 == "VmRSS:" { print $2; exit }' "/proc/$pid/status" 2>/dev/null || true)"
        rss="${rss:-0}"
        if [[ "$rss" =~ ^[0-9]+$ ]] && (( rss > max_rss )); then
            max_rss="$rss"
        fi
    done <"$(cgroup_file "$cgroup" cgroup.procs)"
    printf '%s\n' "$max_rss"
}

events_to_json() {
    local source="$1"
    jq -Rn '
        reduce (
            inputs
            | capture("^(?<key>[a-z_]+) (?<value>[0-9]+)$")
        ) as $item (
            {};
            .[$item.key] = ($item.value | tonumber)
        )
    ' <"$source"
}

write_run_state() {
    local run_dir="$1" state="$2" unit="$3" test_name="$4" command_digest="$5"
    local previous_digest="$6" source_sha="$7" fixture_sha="$8" backend_sha="$9"
    local worker_digest="${10}" parent_cgroup="${11}" test_target tmp
    test_target="$(named_test_target "$test_name")"
    tmp="$run_dir/run-state.json.tmp.$$"
    jq -n -S \
        --arg schema "$RUN_STATE_SCHEMA" \
        --arg created_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
        --arg state "$state" \
        --arg run_dir "$run_dir" \
        --arg unit "$unit.service" \
        --arg test_name "$test_name" \
        --arg test_target "$test_target" \
        --arg command_digest "$command_digest" \
        --arg previous_digest "$previous_digest" \
        --arg source_digest "$source_sha" \
        --arg fixture_digest "$fixture_sha" \
        --arg backend_digest "$backend_sha" \
        --arg worker_digest "$worker_digest" \
        --arg parent_cgroup "$parent_cgroup" \
        --argjson memory_high "$MEMORY_HIGH_BYTES" \
        --argjson memory_max "$MEMORY_MAX_BYTES" \
        --argjson runtime_seconds "$RUNTIME_SECONDS" \
        '{
            schema: $schema,
            created_at: $created_at,
            updated_at: $created_at,
            state: $state,
            detail: "detached transient worker launch prepared",
            run_dir: $run_dir,
            unit: $unit,
            parent_cgroup: $parent_cgroup,
            command: {
                package: "z00z_storage",
                target: $test_target,
                test: $test_name,
                profile: "release",
                exact: true,
                test_threads: 1,
                digest: $command_digest,
                previous_command_digest: $previous_digest
            },
            identity: {
                source_digest: $source_digest,
                fixture_digest: $fixture_digest,
                backend_digest: $backend_digest,
                worker_digest: $worker_digest
            },
            isolation: {
                detached: true,
                oom_policy: "continue",
                kill_mode: "control-group",
                memory_high_bytes: $memory_high,
                memory_max_bytes: $memory_max,
                memory_swap_max_bytes: 0,
                runtime_seconds: $runtime_seconds
            },
            evidence_path: null
        }' >"$tmp"
    mv "$tmp" "$run_dir/run-state.json"
}

set_run_state() {
    local run_dir="$1" state="$2" detail="$3" evidence_path="${4:-}" tmp
    [[ -s "$run_dir/run-state.json" ]] || return 0
    tmp="$run_dir/run-state.json.tmp.$$"
    jq -S \
        --arg updated_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
        --arg state "$state" \
        --arg detail "$detail" \
        --arg evidence_path "$evidence_path" \
        '
        .updated_at = $updated_at
        | .state = $state
        | .detail = $detail
        | .evidence_path = (
            $evidence_path | if length == 0 then null else . end
        )
        ' "$run_dir/run-state.json" >"$tmp"
    mv "$tmp" "$run_dir/run-state.json"
}

write_detached_terminal_failure() {
    local run_dir="$1" exit_reason="$2" detail="$3"
    local exit_status="${4:-null}" signal="${5:-null}" state command_digest output tmp
    state="$run_dir/run-state.json"
    output="$run_dir/resource-evidence.json"
    [[ -s "$state" ]] || return 1
    command_digest="$(jq -r '.command.digest' "$state")"
    tmp="$output.tmp.$$"
    jq -n -S \
        --arg schema "$SCHEMA" \
        --arg recorded_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
        --arg exit_reason "$exit_reason" \
        --arg detail "$detail" \
        --argjson exit_status "$exit_status" \
        --argjson signal "$signal" \
        --argjson state "$(cat "$state")" \
        '{
            schema: $schema,
            evidence_kind: "resource_exhaustion",
            recorded_at: $recorded_at,
            exit_reason: $exit_reason,
            detail: $detail,
            is_retry_forbidden: true,
            command: (
                $state.command
                + {
                    exit_status: $exit_status,
                    signal: $signal
                }
            ),
            identity: ($state.identity + {parameter_digest: null}),
            trace_dimensions: null,
            isolation: (
                $state.isolation
                + {
                    unit: $state.unit,
                    parent_cgroup: $state.parent_cgroup
                }
            ),
            resources: {
                telemetry_complete: false,
                wall_time_ms: null,
                peak_rss_kib: null,
                memory_peak_bytes: null,
                memory_swap_current_bytes: null,
                canonical_proof_bytes: null,
                proof_size_status: null,
                memory_events_before: null,
                memory_events_after: null,
                memory_events_delta: null
            },
            terminal_flags: {
                detached_worker: true,
                normal_finalizer_completed: false,
                unchanged_rerun_forbidden: true
            }
        }' >"$tmp"
    mv "$tmp" "$output"
    cp "$output" "$BLOCK_ROOT/$command_digest.json"
    set_run_state "$run_dir" terminal "$detail" "$output"
}

internal_run_exit_guard() {
    local status=$?
    trap - EXIT TERM INT
    if [[ -n "$ACTIVE_INTERNAL_RUN_DIR" \
        && ! -s "$ACTIVE_INTERNAL_RUN_DIR/resource-evidence.json" ]]; then
        set +e
        write_detached_terminal_failure \
            "$ACTIVE_INTERNAL_RUN_DIR" \
            "${ACTIVE_INTERNAL_EXIT_REASON:-isolation_unavailable}" \
            "detached worker exited before its normal resource finalizer completed" \
            "$status" null
        set -e
    fi
    exit "$status"
}

write_isolation_failure() {
    local output="$1" reason="$2" worker_digest="$3" parent_cgroup="$4" unit="$5"
    jq -n -S \
        --arg schema "$ISOLATION_SCHEMA" \
        --arg recorded_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
        --arg exit_reason "isolation_unavailable" \
        --arg detail "$reason" \
        --arg worker_digest "$worker_digest" \
        --arg parent_cgroup "$parent_cgroup" \
        --arg unit "$unit" \
        --argjson memory_high "$MEMORY_HIGH_BYTES" \
        --argjson memory_max "$MEMORY_MAX_BYTES" \
        '{
            schema: $schema,
            recorded_at: $recorded_at,
            exit_reason: $exit_reason,
            detail: $detail,
            worker_digest: $worker_digest,
            parent_cgroup: $parent_cgroup,
            unit: $unit,
            controls: {
                oom_policy: "continue",
                memory_high_bytes: $memory_high,
                memory_max_bytes: $memory_max,
                memory_swap_max_bytes: 0
            }
        }' >"$output"
}

write_smoke_failure() {
    local output="$1" reason="$2" worker_digest="$3" parent_cgroup="$4"
    local unit="$5" command_digest="$6" source_sha="$7"
    jq -n -S \
        --arg schema "$SCHEMA" \
        --arg recorded_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
        --arg detail "$reason" \
        --arg worker_digest "$worker_digest" \
        --arg parent_cgroup "$parent_cgroup" \
        --arg unit "$unit" \
        --arg command_digest "$command_digest" \
        --arg source_digest "$source_sha" \
        --argjson memory_high "$SMOKE_MEMORY_HIGH_BYTES" \
        --argjson memory_max "$SMOKE_MEMORY_MAX_BYTES" \
        '{
            schema: $schema,
            evidence_kind: "resource_exhaustion",
            recorded_at: $recorded_at,
            exit_reason: "isolation_unavailable",
            detail: $detail,
            is_retry_forbidden: true,
            command: {
                kind: "bootstrap",
                profile: "release",
                test_threads: 1,
                digest: $command_digest
            },
            identity: {
                source_digest: $source_digest,
                worker_digest: $worker_digest
            },
            isolation: {
                unit: $unit,
                parent_cgroup: $parent_cgroup,
                oom_policy: "continue",
                memory_high_bytes: $memory_high,
                memory_max_bytes: $memory_max,
                memory_swap_max_bytes: 0
            }
        }' >"$output"
}

host_preflight() {
    local command
    [[ "$(uname -s)" == "Linux" ]] || die "Linux cgroup-v2 is required"
    [[ "$(stat -fc %T /sys/fs/cgroup)" == "cgroup2fs" ]] || die "unified cgroup-v2 is required"
    [[ -r /sys/fs/cgroup/cgroup.controllers ]] || die "cgroup controllers are unreadable"
    grep -qw memory /sys/fs/cgroup/cgroup.controllers || die "cgroup-v2 memory controller is unavailable"
    for command in awk cargo date find grep jq mv ps readlink sed sha256sum sleep sort stat systemctl systemd-run tail timeout tr uname wc; do
        require_command "$command"
    done
    [[ -x /usr/bin/time ]] || die "/usr/bin/time is required"
    [[ -f "$BACKEND_SOURCE" \
        && -f "$RECURSION_SOURCE" \
        && -f "$BINARY_MMCS_SOURCE" \
        && -f "$BINARY_HASH_SOURCE" \
        && -f "$U16_RANGE_SOURCE" \
        && -f "$AUTHORITY_SOURCE" \
        && -f "$TEST_SOURCE" \
        && -f "$NOVA_SOURCE" ]] ||
        die "recursive proof sources are missing"
    [[ -x "$BOOTSTRAP_SCRIPT" && -x "$NOVA_GATE_SCRIPT" && -x "$NOVA_RSS_SCRIPT" ]] ||
        die "bootstrap gate scripts are missing or not executable"
    systemctl --user show-environment >/dev/null 2>&1 || die "systemd user manager is unavailable"
}

new_run_dir() {
    local label="$1" digest="$2" timestamp
    timestamp="$(date -u +'%Y%m%dT%H%M%S%NZ')"
    printf '%s/%s-%s-%s\n' "$EVIDENCE_ROOT" "$timestamp" "$label" "${digest:0:12}"
}

unit_name() {
    local label="$1" digest="$2"
    printf 'z00z-p3-06907-%s-%s-%s\n' "$label" "$$" "${digest:0:8}"
}

systemd_heavy_start() {
    local unit="$1" service_log="$2"
    shift 2
    systemd-run --user \
        --unit="$unit" \
        --service-type=exec \
        --property=OOMPolicy=continue \
        --property=KillMode=control-group \
        --property=MemoryHigh="$MEMORY_HIGH_BYTES" \
        --property=MemoryMax="$MEMORY_MAX_BYTES" \
        --property=MemorySwapMax=0 \
        --property=RuntimeMaxSec="$((RUNTIME_SECONDS + 120))" \
        --property=TimeoutStopSec=45 \
        --property="StandardOutput=append:$service_log" \
        --property="StandardError=append:$service_log" \
        "$@"
}

systemd_smoke_launch() {
    local unit="$1"
    shift
    systemd-run --user --wait --pipe \
        --unit="$unit" \
        --service-type=exec \
        --property=OOMPolicy=continue \
        --property=KillMode=control-group \
        --property=MemoryHigh="$SMOKE_MEMORY_HIGH_BYTES" \
        --property=MemoryMax="$SMOKE_MEMORY_MAX_BYTES" \
        --property=MemorySwapMax=0 \
        --property=RuntimeMaxSec="$((SMOKE_RUNTIME_SECONDS + 120))" \
        "$@"
}

internal_preflight() {
    local run_dir="$1" parent_cgroup="$2" unit="$3" worker_digest="$4"
    local child_cgroup cgroup_root memory_high memory_max memory_swap_max memory_oom_group
    local oom_policy kill_mode
    local success=false detail="isolation controls did not match"

    child_cgroup="$(current_cgroup)"
    cgroup_root="/sys/fs/cgroup$child_cgroup"
    memory_high="$(read_cgroup_value "$child_cgroup" memory.high 2>/dev/null || true)"
    memory_max="$(read_cgroup_value "$child_cgroup" memory.max 2>/dev/null || true)"
    memory_swap_max="$(read_cgroup_value "$child_cgroup" memory.swap.max 2>/dev/null || true)"
    memory_oom_group="$(read_cgroup_value "$child_cgroup" memory.oom.group 2>/dev/null || true)"
    oom_policy="$(systemctl --user show "$unit.service" --property=OOMPolicy --value 2>/dev/null || true)"
    kill_mode="$(systemctl --user show "$unit.service" --property=KillMode --value 2>/dev/null || true)"

    if [[ -n "$child_cgroup" \
        && "$child_cgroup" != "$parent_cgroup" \
        && "$child_cgroup" != "$parent_cgroup/"* \
        && "$child_cgroup" != *app-code-*.scope* \
        && -r "$cgroup_root/memory.current" \
        && -r "$cgroup_root/memory.peak" \
        && -r "$cgroup_root/memory.events" \
        && -r "$cgroup_root/memory.swap.current" \
        && "$memory_high" == "$MEMORY_HIGH_BYTES" \
        && "$memory_max" == "$MEMORY_MAX_BYTES" \
        && "$memory_swap_max" == "0" \
        && "$memory_oom_group" == "0" \
        && "$oom_policy" == "continue" \
        && "$kill_mode" == "control-group" ]]; then
        success=true
        detail="separate transient service and all cgroup controls verified"
    fi

    jq -n -S \
        --arg schema "$ISOLATION_SCHEMA" \
        --arg recorded_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
        --arg exit_reason "$([[ "$success" == true ]] && printf success || printf isolation_unavailable)" \
        --arg detail "$detail" \
        --arg worker_digest "$worker_digest" \
        --arg parent_cgroup "$parent_cgroup" \
        --arg child_cgroup "$child_cgroup" \
        --arg unit "$unit.service" \
        --arg oom_policy "$oom_policy" \
        --arg kill_mode "$kill_mode" \
        --arg memory_high "$memory_high" \
        --arg memory_max "$memory_max" \
        --arg memory_swap_max "$memory_swap_max" \
        --arg memory_oom_group "$memory_oom_group" \
        '{
            schema: $schema,
            recorded_at: $recorded_at,
            exit_reason: $exit_reason,
            detail: $detail,
            worker_digest: $worker_digest,
            parent_cgroup: $parent_cgroup,
            child_cgroup: $child_cgroup,
            unit: $unit,
            controls: {
                oom_policy: $oom_policy,
                kill_mode: $kill_mode,
                memory_high_bytes: ($memory_high | tonumber?),
                memory_max_bytes: ($memory_max | tonumber?),
                memory_swap_max_bytes: ($memory_swap_max | tonumber?),
                memory_oom_group: ($memory_oom_group | tonumber?)
            }
        }' >"$run_dir/isolation-preflight.json"
    [[ "$success" == true ]]
}

run_preflight() {
    local worker_digest parent_cgroup run_dir unit output active_state
    local attempt=0
    host_preflight
    mkdir -p "$EVIDENCE_ROOT" "$BLOCK_ROOT"
    worker_digest="$(file_digest "$SCRIPT_PATH")"
    parent_cgroup="$(current_cgroup)"
    [[ -n "$parent_cgroup" ]] || die "caller cgroup is unavailable"
    run_dir="$(new_run_dir preflight "$worker_digest")"
    unit="$(unit_name preflight "$worker_digest")"
    mkdir -p "$run_dir"
    output="$run_dir/systemd-run.log"
    if ! systemd_heavy_start "$unit" "$run_dir/service.log" \
        "$SCRIPT_PATH" --internal-preflight \
        "$run_dir" "$parent_cgroup" "$unit" "$worker_digest" >"$output" 2>&1; then
        if [[ ! -s "$run_dir/isolation-preflight.json" ]]; then
            write_isolation_failure \
                "$run_dir/isolation-preflight.json" \
                "transient service launch or execution failed; see systemd-run.log" \
                "$worker_digest" "$parent_cgroup" "$unit.service"
        fi
        printf '%s\n' "$run_dir/isolation-preflight.json"
        return 125
    fi
    while [[ ! -s "$run_dir/isolation-preflight.json" ]] && (( attempt < 120 )); do
        active_state="$(
            systemctl --user show "$unit.service" --property=ActiveState --value \
                2>/dev/null || true
        )"
        if [[ "$active_state" == inactive || "$active_state" == failed || -z "$active_state" ]]; then
            break
        fi
        sleep 0.25
        attempt=$((attempt + 1))
    done
    if [[ ! -s "$run_dir/isolation-preflight.json" ]]; then
        write_isolation_failure \
            "$run_dir/isolation-preflight.json" \
            "detached transient preflight ended without typed isolation evidence" \
            "$worker_digest" "$parent_cgroup" "$unit.service"
        printf '%s\n' "$run_dir/isolation-preflight.json"
        return 125
    fi
    jq -e '.exit_reason == "success"' "$run_dir/isolation-preflight.json" >/dev/null ||
        return 125
    cp "$run_dir/isolation-preflight.json" "$EVIDENCE_ROOT/preflight-latest.json"
    printf '%s\n' "$run_dir/isolation-preflight.json"
}

internal_run() {
    local run_dir="$1" parent_cgroup="$2" unit="$3" test_name="$4"
    local command_digest="$5" source_sha="$6" fixture_sha="$7" backend_sha="$8"
    local previous_digest="$9"
    local child_cgroup cgroup_root high max swap_max oom_group oom_policy start_ns end_ns wall_ms
    local command_status=0 phase="launch" observed_phase="" current peak swap_current time_peak=0
    local proc_rss_kib=0 proc_rss_peak_kib=0 command_pgid
    local max_before=0 oom_before=0 event_max=0 event_oom=0 abort_reason=""
    local last_log_mtime=0 current_log_mtime=0 last_stage_progress_epoch=0 now_epoch=0
    local stage_watchdog_armed=false
    local events_before events_after events_before_json events_after_json events_delta_json
    local oom_delta max_delta exit_reason detail retry_forbidden=false test_count_ok=false
    local telemetry_json=null parameter_digest=null trace_dimensions=null canonical_proof_bytes=null
    local size_status=null last_chunk_json=null chunk_candidate trace_candidate
    local cache_files=0 cache_bytes=0 worker_digest test_target diagnostic_filter=""
    declare -A phase_high_water=(
        [launch]=0
        [fixture_ready]=0
        [proving]=0
        [chunk_structural]=0
        [chunk_hash]=0
        [chunk_source]=0
        [chunk_lists]=0
        [chunk_uniqueness]=0
        [chunk_trace]=0
        [chunk_transition]=0
        [aggregation]=0
        [proof_ready]=0
        [verifying]=0
        [verify_complete]=0
    )

    ACTIVE_INTERNAL_RUN_DIR="$run_dir"
    ACTIVE_INTERNAL_EXIT_REASON=""
    trap internal_run_exit_guard EXIT
    trap 'ACTIVE_INTERNAL_EXIT_REASON=resource_timeout; exit 143' TERM
    trap 'ACTIVE_INTERNAL_EXIT_REASON=isolation_unavailable; exit 130' INT
    worker_digest="$(file_digest "$SCRIPT_PATH")"
    test_target="$(named_test_target "$test_name")"

    child_cgroup="$(current_cgroup)"
    cgroup_root="/sys/fs/cgroup$child_cgroup"
    high="$(read_cgroup_value "$child_cgroup" memory.high 2>/dev/null || true)"
    max="$(read_cgroup_value "$child_cgroup" memory.max 2>/dev/null || true)"
    swap_max="$(read_cgroup_value "$child_cgroup" memory.swap.max 2>/dev/null || true)"
    oom_group="$(read_cgroup_value "$child_cgroup" memory.oom.group 2>/dev/null || true)"
    oom_policy="$(systemctl --user show "$unit.service" --property=OOMPolicy --value 2>/dev/null || true)"
    if [[ -z "$child_cgroup" \
        || "$child_cgroup" == "$parent_cgroup" \
        || "$child_cgroup" == "$parent_cgroup/"* \
        || "$child_cgroup" == *app-code-*.scope* \
        || "$high" != "$MEMORY_HIGH_BYTES" \
        || "$max" != "$MEMORY_MAX_BYTES" \
        || "$swap_max" != "0" \
        || "$oom_group" != "0" \
        || "$oom_policy" != "continue" \
        || ! -r "$cgroup_root/memory.events" ]]; then
        printf 'isolation unavailable before cargo launch\n' >"$run_dir/inner-error.log"
        write_detached_terminal_failure \
            "$run_dir" isolation_unavailable \
            "detached worker isolation controls were unavailable before cargo launch" \
            125 null
        return 125
    fi

    set_run_state \
        "$run_dir" running \
        "detached transient worker verified its cgroup controls and started" ""

    events_before="$run_dir/memory-events-before.txt"
    events_after="$run_dir/memory-events-after.txt"
    cp "$cgroup_root/memory.events" "$events_before"
    max_before="$(awk '$1 == "max" { print $2; exit }' "$events_before")"
    oom_before="$(awk '
        $1 == "oom_kill" || $1 == "oom_group_kill" { total += $2 }
        END { print total + 0 }
    ' "$events_before")"
    read_cgroup_value "$child_cgroup" memory.current >"$run_dir/memory-current-before.txt"
    read_cgroup_value "$child_cgroup" memory.peak >"$run_dir/memory-peak-before.txt"
    read_cgroup_value "$child_cgroup" memory.swap.current >"$run_dir/memory-swap-before.txt"

    start_ns="$(date +%s%N)"
    set +e
    mkdir -p "$CHUNK_CACHE_ROOT"
    if is_lib_diagnostic_test "$test_name"; then
        diagnostic_filter="$(lib_diagnostic_filter "$test_name")"
        /usr/bin/timeout --signal=TERM --kill-after=30s "$RUNTIME_SECONDS" \
            /usr/bin/time -v -o "$run_dir/time-v.txt" \
            env Z00Z_PLONKY3_RESOURCE_TELEMETRY=1 \
            cargo test --release --lib -p z00z_storage "$diagnostic_filter" -- \
            --ignored --exact --nocapture --test-threads=1 >"$run_dir/test.log" 2>&1 &
    else
        /usr/bin/timeout --signal=TERM --kill-after=30s "$RUNTIME_SECONDS" \
            /usr/bin/time -v -o "$run_dir/time-v.txt" \
            env Z00Z_PLONKY3_RESOURCE_TELEMETRY=1 \
            Z00Z_PLONKY3_CHUNK_CACHE_DIR="$CHUNK_CACHE_ROOT" \
            cargo test --release -p z00z_storage --test "$TEST_TARGET" "$test_name" -- \
            --ignored --exact --nocapture --test-threads=1 >"$run_dir/test.log" 2>&1 &
    fi
    local command_pid=$!
    command_pgid="$(ps -o pgid= -p "$command_pid" | tr -d '[:space:]')"
    command_pgid="${command_pgid:-$command_pid}"
    last_log_mtime="$(stat -c %Y "$run_dir/test.log" 2>/dev/null || printf 0)"
    last_stage_progress_epoch="$(date +%s)"
    while kill -0 "$command_pid" 2>/dev/null; do
        current="$(read_cgroup_value "$child_cgroup" memory.current 2>/dev/null || printf 0)"
        proc_rss_kib="$(cgroup_max_rss_kib "$child_cgroup")"
        if (( proc_rss_kib > proc_rss_peak_kib )); then
            proc_rss_peak_kib="$proc_rss_kib"
        fi
        event_max="$(awk '$1 == "max" { print $2; exit }' "$cgroup_root/memory.events")"
        event_oom="$(awk '
            $1 == "oom_kill" || $1 == "oom_group_kill" { total += $2 }
            END { print total + 0 }
        ' "$cgroup_root/memory.events")"
        if (( event_oom > oom_before )); then
            abort_reason=oom
        elif (( event_max > max_before )); then
            abort_reason=cgroup_max
        elif (( proc_rss_kib * 1024 > MEMORY_TARGET_BYTES )); then
            abort_reason=process_rss
        elif [[ "$current" =~ ^[0-9]+$ ]] && (( current > MEMORY_TARGET_BYTES )); then
            abort_reason=target_high
        fi
        if [[ -n "$abort_reason" ]]; then
            kill -TERM -- "-$command_pgid" 2>/dev/null ||
                kill -TERM "$command_pid" 2>/dev/null || true
            break
        fi
        observed_phase="$(
            sed -n 's/^Z00Z_PLONKY3_PHASE_V1 //p' "$run_dir/test.log" |
                tail -n 1
        )"
        case "$observed_phase" in
            fixture_ready | proving | chunk_structural | chunk_hash | chunk_source | \
                chunk_lists | chunk_uniqueness | chunk_trace | chunk_transition | \
                aggregation | proof_ready | verifying | verify_complete)
                phase="$observed_phase"
                ;;
        esac
        current_log_mtime="$(stat -c %Y "$run_dir/test.log" 2>/dev/null || printf 0)"
        now_epoch="$(date +%s)"
        if [[ "$current_log_mtime" =~ ^[0-9]+$ ]] &&
            (( current_log_mtime != last_log_mtime )); then
            last_log_mtime="$current_log_mtime"
            last_stage_progress_epoch="$now_epoch"
        fi
        case "$phase" in
            chunk_structural | chunk_hash | chunk_source | chunk_lists | \
                chunk_uniqueness | chunk_trace | chunk_transition | aggregation | \
                proof_ready | verifying)
                stage_watchdog_armed=true
                ;;
        esac
        if [[ "$stage_watchdog_armed" == true ]] &&
            (( now_epoch - last_stage_progress_epoch >= STAGE_STALL_SECONDS )); then
            abort_reason=phase_timeout
            kill -TERM -- "-$command_pgid" 2>/dev/null ||
                kill -TERM "$command_pid" 2>/dev/null || true
            break
        fi
        if [[ "$current" =~ ^[0-9]+$ ]] && (( current > phase_high_water[$phase] )); then
            phase_high_water[$phase]="$current"
        fi
        sleep 0.25
    done
    wait "$command_pid"
    command_status=$?
    set -e
    end_ns="$(date +%s%N)"
    wall_ms=$(((end_ns - start_ns) / 1000000))

    cp "$cgroup_root/memory.events" "$events_after"
    current="$(read_cgroup_value "$child_cgroup" memory.current 2>/dev/null || printf 0)"
    peak="$(read_cgroup_value "$child_cgroup" memory.peak 2>/dev/null || printf 0)"
    swap_current="$(read_cgroup_value "$child_cgroup" memory.swap.current 2>/dev/null || printf 0)"
    printf '%s\n' "$current" >"$run_dir/memory-current-after.txt"
    printf '%s\n' "$peak" >"$run_dir/memory-peak-after.txt"
    printf '%s\n' "$swap_current" >"$run_dir/memory-swap-after.txt"
    time_peak="$(sed -nE 's/^[[:space:]]*Maximum resident set size \\(kbytes\\):[[:space:]]*([0-9]+)$/\\1/p' "$run_dir/time-v.txt" | tail -n 1)"
    time_peak="${time_peak:-0}"
    cache_files="$(find "$CHUNK_CACHE_ROOT" -maxdepth 1 -type f -name '*.postcard' | wc -l)"
    cache_bytes="$(find "$CHUNK_CACHE_ROOT" -maxdepth 1 -type f -name '*.postcard' -printf '%s\n' |
        awk '{ total += $1 } END { print total + 0 }')"

    events_before_json="$(events_to_json "$events_before")"
    events_after_json="$(events_to_json "$events_after")"
    events_delta_json="$(jq -n \
        --argjson before "$events_before_json" \
        --argjson after "$events_after_json" \
        '$after | with_entries(.value = (.value - ($before[.key] // 0)))')"
    oom_delta="$(jq -r '(.oom_kill // 0) + (.oom_group_kill // 0)' <<<"$events_delta_json")"
    max_delta="$(jq -r '.max // 0' <<<"$events_delta_json")"

    if grep -Fq 'running 1 test' "$run_dir/test.log" &&
        grep -Fq 'test result: ok. 1 passed; 0 failed;' "$run_dir/test.log"; then
        if is_lib_diagnostic_test "$test_name"; then
            diagnostic_filter="$(lib_diagnostic_filter "$test_name")"
            grep -Fq "test $diagnostic_filter ..." "$run_dir/test.log" &&
                test_count_ok=true
        elif grep -Fq "test $test_name ..." "$run_dir/test.log"; then
            test_count_ok=true
        fi
    fi
    if grep -Fq 'Z00Z_PLONKY3_TELEMETRY_V1 ' "$run_dir/test.log"; then
        telemetry_json="$(sed -n 's/^Z00Z_PLONKY3_TELEMETRY_V1 //p' "$run_dir/test.log" |
            tail -n 1)"
        if jq -e 'type == "object"' <<<"$telemetry_json" >/dev/null 2>&1; then
            parameter_digest="$(jq -c '.parameter_digest // null' <<<"$telemetry_json")"
            trace_dimensions="$(jq -c '.trace_dimensions // null' <<<"$telemetry_json")"
            canonical_proof_bytes="$(jq -c '.canonical_proof_bytes // null' <<<"$telemetry_json")"
            size_status="$(jq -c '.size_status // null' <<<"$telemetry_json")"
        else
            telemetry_json=null
        fi
    fi
    if grep -Fq 'Z00Z_PLONKY3_CHUNK_V1 ' "$run_dir/test.log"; then
        chunk_candidate="$(
            sed -n 's/^Z00Z_PLONKY3_CHUNK_V1 //p' "$run_dir/test.log" |
                tail -n 1
        )"
        if jq -e '
            type == "object"
            and (.stage | type == "string")
            and (.domain | type == "string")
            and (.replica | type == "number")
            and (.index | type == "number")
            and (.count | type == "number")
            and (.elapsed_ms | type == "number")
        ' <<<"$chunk_candidate" >/dev/null 2>&1; then
            last_chunk_json="$chunk_candidate"
        fi
    fi
    if [[ "$trace_dimensions" == null ]] &&
        grep -Fq 'Z00Z_PLONKY3_TRACE_DIMENSIONS_V1 ' "$run_dir/test.log"; then
        trace_candidate="$(
            sed -n 's/^Z00Z_PLONKY3_TRACE_DIMENSIONS_V1 //p' "$run_dir/test.log" |
                tail -n 1
        )"
        if jq -e '
            type == "object"
            and (.domain | type == "string")
            and (.replica | type == "number")
            and (.index | type == "number")
            and (.count | type == "number")
            and (.dimensions | type == "object")
        ' <<<"$trace_candidate" >/dev/null 2>&1; then
            trace_dimensions="$(jq -c '.dimensions' <<<"$trace_candidate")"
        fi
    fi

    if [[ "$abort_reason" == process_rss || "$abort_reason" == target_high ]]; then
        exit_reason=resource_memory_max
        detail="real proof exceeded the 16 GiB acceptance target"
    elif [[ "$abort_reason" == phase_timeout ]]; then
        exit_reason=resource_timeout
        detail="named proof stage made no progress for the bounded 900-second limit"
    elif (( oom_delta > 0 )); then
        exit_reason=resource_oom
        detail="cgroup memory.events recorded an OOM kill"
    elif (( max_delta > 0 )); then
        exit_reason=resource_memory_max
        detail="cgroup memory.events recorded MemoryMax pressure"
    elif (( command_status == 124 )); then
        exit_reason=resource_timeout
        detail="bounded resource timeout expired"
    elif (( command_status == 137 )); then
        exit_reason=resource_sigkill
        detail="isolated command exited after SIGKILL"
    elif [[ ! "$peak" =~ ^[0-9]+$ || ! "$swap_current" =~ ^[0-9]+$ ]]; then
        exit_reason=isolation_unavailable
        detail="required cgroup resource metadata is missing"
    elif (( peak > MEMORY_TARGET_BYTES )); then
        exit_reason=resource_memory_max
        detail="peak cgroup memory exceeded the 16 GiB acceptance target"
    elif (( swap_current > 0 )); then
        exit_reason=resource_memory_max
        detail="nonzero cgroup swap use violates acceptance"
    elif (( command_status == 0 )) && [[ "$test_count_ok" == true ]] &&
        is_source_diagnostic_test "$test_name"; then
        exit_reason=success
        detail="isolated bounded Source AIR diagnostic passed"
    elif (( command_status == 0 )) && [[ "$test_count_ok" == true ]] &&
        is_hash_diagnostic_test "$test_name" && [[ "$trace_dimensions" != null ]]; then
        exit_reason=success
        detail="isolated bounded hash-leaf prove/actual-verify diagnostic passed"
    elif (( command_status == 0 )) && [[ "$test_count_ok" == true ]] &&
        is_aggregation_diagnostic_test "$test_name"; then
        exit_reason=success
        detail="isolated bounded concurrent aggregation wave prove/actual-verify diagnostic passed"
    elif (( command_status == 0 )) && [[ "$test_count_ok" == true ]] &&
        [[ "$telemetry_json" != null && "$parameter_digest" != null &&
            "$trace_dimensions" != null && "$canonical_proof_bytes" != null &&
            "$size_status" != null ]]; then
        exit_reason=success
        detail="exact named proof test passed with complete telemetry"
    elif (( command_status == 0 )); then
        exit_reason=isolation_unavailable
        detail="exact-test or Plonky3 telemetry metadata is missing"
    else
        exit_reason=test_failure
        detail="exact named proof test failed without a resource terminal"
    fi
    case "$exit_reason" in
        resource_oom | resource_sigkill | resource_memory_max | resource_timeout | isolation_unavailable)
            retry_forbidden=true
            ;;
    esac

    jq -n -S \
        --arg schema "$SCHEMA" \
        --arg recorded_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
        --arg exit_reason "$exit_reason" \
        --arg detail "$detail" \
        --arg test_name "$test_name" \
        --arg test_target "$test_target" \
        --arg command_digest "$command_digest" \
        --arg previous_command_digest "$previous_digest" \
        --arg source_digest "$source_sha" \
        --arg fixture_digest "$fixture_sha" \
        --arg backend_digest "$backend_sha" \
        --arg worker_digest "$worker_digest" \
        --arg child_cgroup "$child_cgroup" \
        --arg unit "$unit.service" \
        --arg oom_policy "$oom_policy" \
        --argjson parameter_digest "$parameter_digest" \
        --argjson trace_dimensions "$trace_dimensions" \
        --argjson canonical_proof_bytes "$canonical_proof_bytes" \
        --argjson size_status "$size_status" \
        --argjson last_chunk "$last_chunk_json" \
        --argjson events_before "$events_before_json" \
        --argjson events_after "$events_after_json" \
        --argjson events_delta "$events_delta_json" \
        --argjson command_status "$command_status" \
        --argjson wall_ms "$wall_ms" \
        --argjson peak "$peak" \
        --argjson current "$current" \
        --argjson swap_current "$swap_current" \
        --argjson time_peak_kib "$time_peak" \
        --argjson process_peak_kib "$proc_rss_peak_kib" \
        --argjson cache_files "$cache_files" \
        --argjson cache_bytes "$cache_bytes" \
        --argjson retry_forbidden "$retry_forbidden" \
        --argjson launch_hwm "${phase_high_water[launch]}" \
        --argjson fixture_hwm "${phase_high_water[fixture_ready]}" \
        --argjson proving_hwm "${phase_high_water[proving]}" \
        --argjson structural_hwm "${phase_high_water[chunk_structural]}" \
        --argjson hash_hwm "${phase_high_water[chunk_hash]}" \
        --argjson source_hwm "${phase_high_water[chunk_source]}" \
        --argjson lists_hwm "${phase_high_water[chunk_lists]}" \
        --argjson uniqueness_hwm "${phase_high_water[chunk_uniqueness]}" \
        --argjson trace_hwm "${phase_high_water[chunk_trace]}" \
        --argjson transition_hwm "${phase_high_water[chunk_transition]}" \
        --argjson aggregation_hwm "${phase_high_water[aggregation]}" \
        --argjson proof_hwm "${phase_high_water[proof_ready]}" \
        --argjson verifying_hwm "${phase_high_water[verifying]}" \
        --argjson verify_hwm "${phase_high_water[verify_complete]}" \
        --argjson memory_high "$MEMORY_HIGH_BYTES" \
        --argjson memory_max "$MEMORY_MAX_BYTES" \
        --argjson stage_stall_seconds "$STAGE_STALL_SECONDS" \
        '{
            schema: $schema,
            evidence_kind: (if $retry_forbidden then "resource_exhaustion" else "proof_run" end),
            recorded_at: $recorded_at,
            exit_reason: $exit_reason,
            detail: $detail,
            is_retry_forbidden: $retry_forbidden,
            command: {
                package: "z00z_storage",
                target: $test_target,
                test: $test_name,
                profile: "release",
                exact: true,
                test_threads: 1,
                exit_status: $command_status,
                digest: $command_digest,
                previous_command_digest: $previous_command_digest
            },
            identity: {
                source_digest: $source_digest,
                fixture_digest: $fixture_digest,
                backend_digest: $backend_digest,
                worker_digest: $worker_digest,
                parameter_digest: $parameter_digest
            },
            trace_dimensions: $trace_dimensions,
            isolation: {
                unit: $unit,
                cgroup: $child_cgroup,
                oom_policy: $oom_policy,
                memory_high_bytes: $memory_high,
                memory_max_bytes: $memory_max,
                memory_swap_max_bytes: 0,
                stage_stall_seconds: $stage_stall_seconds
            },
            resources: {
                wall_time_ms: $wall_ms,
                peak_rss_kib: $time_peak_kib,
                observed_process_peak_rss_kib: $process_peak_kib,
                memory_current_bytes: $current,
                memory_peak_bytes: $peak,
                memory_swap_current_bytes: $swap_current,
                chunk_cache_files: $cache_files,
                chunk_cache_bytes: $cache_bytes,
                canonical_proof_bytes: $canonical_proof_bytes,
                proof_size_status: $size_status,
                last_chunk_progress: $last_chunk,
                stage_stall_seconds: $stage_stall_seconds,
                memory_events_before: $events_before,
                memory_events_after: $events_after,
                memory_events_delta: $events_delta,
                phase_high_water_bytes: {
                    launch: $launch_hwm,
                    fixture_ready: $fixture_hwm,
                    proving: $proving_hwm,
                    chunk_structural: $structural_hwm,
                    chunk_hash: $hash_hwm,
                    chunk_source: $source_hwm,
                    chunk_lists: $lists_hwm,
                    chunk_uniqueness: $uniqueness_hwm,
                    chunk_trace: $trace_hwm,
                    chunk_transition: $transition_hwm,
                    aggregation: $aggregation_hwm,
                    proof_ready: $proof_hwm,
                    verifying: $verifying_hwm,
                    verify_complete: $verify_hwm
                }
            }
        }' >"$run_dir/resource-evidence.json"

    set_run_state \
        "$run_dir" terminal \
        "detached worker completed with exit_reason=$exit_reason" \
        "$run_dir/resource-evidence.json"
    trap - EXIT TERM INT
    ACTIVE_INTERNAL_RUN_DIR=""
    ACTIVE_INTERNAL_EXIT_REASON=""
    if [[ "$retry_forbidden" == true ]]; then
        cp "$run_dir/resource-evidence.json" "$BLOCK_ROOT/$command_digest.json"
        return 125
    fi
    [[ "$exit_reason" == success ]]
}

internal_bootstrap() {
    local run_dir="$1" parent_cgroup="$2" unit="$3" command_digest="$4"
    local previous_digest="$5" source_sha="$6" fixture_sha="$7" backend_sha="$8"
    local worker_digest="$9" child_cgroup cgroup_root high max swap_max oom_group
    local oom_policy kill_mode start_ns end_ns wall_ms command_status=0 phase=launch
    local current peak swap_current time_peak=0 time_peak_bytes=0 signal=null
    local proc_rss_kib=0 proc_rss_peak_kib=0 command_pgid
    local max_before=0 oom_before=0 event_max=0 event_oom=0 abort_reason=""
    local events_before events_after events_before_json events_after_json events_delta_json
    local oom_delta max_delta exit_reason detail retry_forbidden=false bootstrap_complete=false
    local host_swap_kib=0
    declare -A phase_hwm=(
        [launch]=0
        [compile]=0
        [foundational]=0
        [storage]=0
        [nova]=0
        [plonky3]=0
        [wallet]=0
        [complete]=0
    )

    child_cgroup="$(current_cgroup)"
    cgroup_root="/sys/fs/cgroup$child_cgroup"
    high="$(read_cgroup_value "$child_cgroup" memory.high 2>/dev/null || true)"
    max="$(read_cgroup_value "$child_cgroup" memory.max 2>/dev/null || true)"
    swap_max="$(read_cgroup_value "$child_cgroup" memory.swap.max 2>/dev/null || true)"
    oom_group="$(read_cgroup_value "$child_cgroup" memory.oom.group 2>/dev/null || true)"
    oom_policy="$(systemctl --user show "$unit.service" --property=OOMPolicy --value 2>/dev/null || true)"
    kill_mode="$(systemctl --user show "$unit.service" --property=KillMode --value 2>/dev/null || true)"
    if [[ -z "$child_cgroup" \
        || "$child_cgroup" == "$parent_cgroup" \
        || "$child_cgroup" == "$parent_cgroup/"* \
        || "$child_cgroup" == *app-code-*.scope* \
        || "$high" != "$SMOKE_MEMORY_HIGH_BYTES" \
        || "$max" != "$SMOKE_MEMORY_MAX_BYTES" \
        || "$swap_max" != "0" \
        || "$oom_group" != "0" \
        || "$oom_policy" != "continue" \
        || "$kill_mode" != "control-group" \
        || ! -r "$cgroup_root/memory.events" ]]; then
        write_smoke_failure \
            "$run_dir/resource-evidence.json" \
            "smoke isolation controls were unavailable before bootstrap launch" \
            "$worker_digest" "$parent_cgroup" "$unit.service" \
            "$command_digest" "$source_sha"
        return 125
    fi

    events_before="$run_dir/memory-events-before.txt"
    events_after="$run_dir/memory-events-after.txt"
    cp "$cgroup_root/memory.events" "$events_before"
    max_before="$(awk '$1 == "max" { print $2; exit }' "$events_before")"
    oom_before="$(awk '
        $1 == "oom_kill" || $1 == "oom_group_kill" { total += $2 }
        END { print total + 0 }
    ' "$events_before")"
    read_cgroup_value "$child_cgroup" memory.current >"$run_dir/memory-current-before.txt"
    read_cgroup_value "$child_cgroup" memory.peak >"$run_dir/memory-peak-before.txt"
    read_cgroup_value "$child_cgroup" memory.swap.current >"$run_dir/memory-swap-before.txt"
    host_swap_kib="$(awk '$1 == "SwapTotal:" { print $2; exit }' /proc/meminfo)"
    host_swap_kib="${host_swap_kib:-0}"

    start_ns="$(date +%s%N)"
    set +e
    /usr/bin/timeout --signal=TERM --kill-after=30s "$SMOKE_RUNTIME_SECONDS" \
        /usr/bin/time -v -o "$run_dir/time-v.txt" \
        env \
        BOOTSTRAP_THREADS=1 \
        MALLOC_ARENA_MAX=1 \
        MALLOC_MMAP_THRESHOLD_=131072 \
        MALLOC_TRIM_THRESHOLD_=131072 \
        MALLOC_TOP_PAD_=0 \
        "$BOOTSTRAP_SCRIPT" >"$run_dir/bootstrap.log" 2>&1 &
    local command_pid=$!
    command_pgid="$(ps -o pgid= -p "$command_pid" | tr -d '[:space:]')"
    command_pgid="${command_pgid:-$command_pid}"
    while kill -0 "$command_pid" 2>/dev/null; do
        current="$(read_cgroup_value "$child_cgroup" memory.current 2>/dev/null || printf 0)"
        proc_rss_kib="$(cgroup_max_rss_kib "$child_cgroup")"
        if (( proc_rss_kib > proc_rss_peak_kib )); then
            proc_rss_peak_kib="$proc_rss_kib"
        fi
        event_max="$(awk '$1 == "max" { print $2; exit }' "$cgroup_root/memory.events")"
        event_oom="$(awk '
            $1 == "oom_kill" || $1 == "oom_group_kill" { total += $2 }
            END { print total + 0 }
        ' "$cgroup_root/memory.events")"
        if (( event_oom > oom_before )); then
            abort_reason=oom
        elif (( event_max > max_before )); then
            abort_reason=cgroup_max
        elif (( proc_rss_kib * 1024 > SMOKE_PROCESS_BYTES )); then
            abort_reason=process_rss
        fi
        if [[ -n "$abort_reason" ]]; then
            kill -TERM -- "-$command_pgid" 2>/dev/null ||
                kill -TERM "$command_pid" 2>/dev/null || true
            break
        fi
        if grep -Fq '=== BOOTSTRAP COMPLETE ===' "$run_dir/bootstrap.log"; then
            phase=complete
        elif grep -Fq '=== wallet integration ===' "$run_dir/bootstrap.log"; then
            phase=wallet
        elif grep -Fq '=== storage units: Plonky3 owner ===' "$run_dir/bootstrap.log"; then
            phase=plonky3
        elif grep -Fq '=== storage units: Nova owner ===' "$run_dir/bootstrap.log"; then
            phase=nova
        elif grep -Fq '=== storage units: non-Nova/non-Plonky3 ===' "$run_dir/bootstrap.log"; then
            phase=storage
        elif grep -Fq '=== foundational units ===' "$run_dir/bootstrap.log"; then
            phase=foundational
        else
            phase=compile
        fi
        if [[ "$current" =~ ^[0-9]+$ ]] && (( current > phase_hwm[$phase] )); then
            phase_hwm[$phase]="$current"
        fi
        sleep 0.25
    done
    wait "$command_pid"
    command_status=$?
    set -e
    end_ns="$(date +%s%N)"
    wall_ms=$(((end_ns - start_ns) / 1000000))

    cp "$cgroup_root/memory.events" "$events_after"
    current="$(read_cgroup_value "$child_cgroup" memory.current 2>/dev/null || printf 0)"
    peak="$(read_cgroup_value "$child_cgroup" memory.peak 2>/dev/null || printf 0)"
    swap_current="$(read_cgroup_value "$child_cgroup" memory.swap.current 2>/dev/null || printf 0)"
    printf '%s\n' "$current" >"$run_dir/memory-current-after.txt"
    printf '%s\n' "$peak" >"$run_dir/memory-peak-after.txt"
    printf '%s\n' "$swap_current" >"$run_dir/memory-swap-after.txt"
    time_peak="$(sed -nE 's/^[[:space:]]*Maximum resident set size \\(kbytes\\):[[:space:]]*([0-9]+)$/\\1/p' "$run_dir/time-v.txt" | tail -n 1)"
    time_peak="${time_peak:-0}"
    time_peak_bytes=$((time_peak * 1024))

    events_before_json="$(events_to_json "$events_before")"
    events_after_json="$(events_to_json "$events_after")"
    events_delta_json="$(jq -n \
        --argjson before "$events_before_json" \
        --argjson after "$events_after_json" \
        '$after | with_entries(.value = (.value - ($before[.key] // 0)))')"
    oom_delta="$(jq -r '(.oom_kill // 0) + (.oom_group_kill // 0)' <<<"$events_delta_json")"
    max_delta="$(jq -r '.max // 0' <<<"$events_delta_json")"
    if (( command_status == 137 )) ||
        grep -Eq 'signal: 9, SIGKILL|Command terminated by signal 9' \
            "$run_dir/bootstrap.log" "$run_dir/time-v.txt" 2>/dev/null; then
        signal=9
    fi
    if (( command_status == 0 )) &&
        grep -Fq '=== BOOTSTRAP COMPLETE ===' "$run_dir/bootstrap.log"; then
        bootstrap_complete=true
    fi

    if [[ "$abort_reason" == process_rss ]]; then
        exit_reason=resource_memory_max
        detail="one bootstrap process exceeded the 4 GiB RSS ceiling"
        signal=15
    elif (( oom_delta > 0 )); then
        exit_reason=resource_oom
        detail="bootstrap cgroup recorded an OOM kill"
    elif (( max_delta > 0 )); then
        exit_reason=resource_memory_max
        detail="bootstrap cgroup recorded MemoryMax pressure"
    elif (( command_status == 124 )); then
        exit_reason=resource_timeout
        detail="bootstrap resource timeout expired"
    elif [[ "$signal" == 9 ]]; then
        exit_reason=resource_sigkill
        detail="bootstrap command or child received SIGKILL"
    elif [[ ! "$peak" =~ ^[0-9]+$ || ! "$swap_current" =~ ^[0-9]+$ ]]; then
        exit_reason=isolation_unavailable
        detail="required bootstrap cgroup telemetry is missing"
    elif (( proc_rss_peak_kib * 1024 > SMOKE_PROCESS_BYTES ||
        time_peak_bytes > SMOKE_PROCESS_BYTES )); then
        exit_reason=resource_memory_max
        detail="one bootstrap process exceeded the 4 GiB RSS ceiling"
    elif (( swap_current > 0 )); then
        exit_reason=resource_memory_max
        detail="bootstrap used swap despite the zero-swap contract"
    elif [[ "$bootstrap_complete" == true ]]; then
        exit_reason=success
        detail="isolated release bootstrap passed within the smoke budget"
    else
        exit_reason=test_failure
        detail="isolated release bootstrap failed without a resource terminal"
    fi
    case "$exit_reason" in
        resource_oom | resource_sigkill | resource_memory_max | resource_timeout | isolation_unavailable)
            retry_forbidden=true
            ;;
    esac

    jq -n -S \
        --arg schema "$SCHEMA" \
        --arg recorded_at "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
        --arg exit_reason "$exit_reason" \
        --arg detail "$detail" \
        --arg command_digest "$command_digest" \
        --arg previous_digest "$previous_digest" \
        --arg source_digest "$source_sha" \
        --arg fixture_digest "$fixture_sha" \
        --arg backend_digest "$backend_sha" \
        --arg worker_digest "$worker_digest" \
        --arg child_cgroup "$child_cgroup" \
        --arg parent_cgroup "$parent_cgroup" \
        --arg unit "$unit.service" \
        --arg oom_policy "$oom_policy" \
        --arg kill_mode "$kill_mode" \
        --arg log_path "$run_dir/bootstrap.log" \
        --arg time_path "$run_dir/time-v.txt" \
        --argjson events_before "$events_before_json" \
        --argjson events_after "$events_after_json" \
        --argjson events_delta "$events_delta_json" \
        --argjson command_status "$command_status" \
        --argjson signal "$signal" \
        --argjson wall_ms "$wall_ms" \
        --argjson peak "$peak" \
        --argjson current "$current" \
        --argjson swap_current "$swap_current" \
        --argjson time_peak_kib "$time_peak" \
        --argjson process_peak_kib "$proc_rss_peak_kib" \
        --argjson host_swap_bytes "$((host_swap_kib * 1024))" \
        --argjson retry_forbidden "$retry_forbidden" \
        --argjson bootstrap_complete "$bootstrap_complete" \
        --argjson launch_hwm "${phase_hwm[launch]}" \
        --argjson compile_hwm "${phase_hwm[compile]}" \
        --argjson foundational_hwm "${phase_hwm[foundational]}" \
        --argjson storage_hwm "${phase_hwm[storage]}" \
        --argjson nova_hwm "${phase_hwm[nova]}" \
        --argjson plonky3_hwm "${phase_hwm[plonky3]}" \
        --argjson wallet_hwm "${phase_hwm[wallet]}" \
        --argjson complete_hwm "${phase_hwm[complete]}" \
        --argjson memory_high "$SMOKE_MEMORY_HIGH_BYTES" \
        --argjson memory_max "$SMOKE_MEMORY_MAX_BYTES" \
        --argjson process_limit "$SMOKE_PROCESS_BYTES" \
        '{
            schema: $schema,
            evidence_kind: (if $retry_forbidden then "resource_exhaustion" else "bootstrap_run" end),
            recorded_at: $recorded_at,
            exit_reason: $exit_reason,
            detail: $detail,
            is_retry_forbidden: $retry_forbidden,
            command: {
                kind: "bootstrap",
                profile: "release",
                test_threads: 1,
                exit_status: $command_status,
                signal: $signal,
                digest: $command_digest,
                previous_command_digest: (
                    $previous_digest | if length == 0 then null else . end
                )
            },
            identity: {
                source_digest: $source_digest,
                fixture_digest: $fixture_digest,
                backend_digest: $backend_digest,
                worker_digest: $worker_digest
            },
            isolation: {
                unit: $unit,
                cgroup: $child_cgroup,
                parent_cgroup: $parent_cgroup,
                outside_parent_scope: true,
                oom_policy: $oom_policy,
                kill_mode: $kill_mode,
                memory_high_bytes: $memory_high,
                memory_max_bytes: $memory_max,
                memory_swap_max_bytes: 0
            },
            resources: {
                wall_time_ms: $wall_ms,
                peak_rss_kib: $time_peak_kib,
                observed_process_peak_rss_kib: $process_peak_kib,
                process_rss_limit_bytes: $process_limit,
                memory_current_bytes: $current,
                memory_peak_bytes: $peak,
                memory_swap_current_bytes: $swap_current,
                host_swap_total_bytes: $host_swap_bytes,
                memory_events_before: $events_before,
                memory_events_after: $events_after,
                memory_events_delta: $events_delta,
                phase_high_water_bytes: {
                    launch: $launch_hwm,
                    release_compile: $compile_hwm,
                    foundational_tests: $foundational_hwm,
                    storage_tests: $storage_hwm,
                    nova_tests: $nova_hwm,
                    plonky3_tests: $plonky3_hwm,
                    wallet_tests: $wallet_hwm,
                    bootstrap_complete: $complete_hwm
                }
            },
            terminal_flags: {
                bootstrap_complete: $bootstrap_complete,
                named_plonky3_prover_started: false
            },
            artifacts: {
                stdout_stderr_log: $log_path,
                time_v_log: $time_path
            }
        }' >"$run_dir/resource-evidence.json"

    if [[ "$retry_forbidden" == true ]]; then
        cp "$run_dir/resource-evidence.json" "$BLOCK_ROOT/$command_digest.json"
        return 125
    fi
    [[ "$exit_reason" == success ]]
}

run_bootstrap() {
    local worker_digest parent_cgroup run_dir unit source_sha fixture_sha backend_sha
    local command_text command_digest previous_digest output launch_status reason
    host_preflight
    mkdir -p "$EVIDENCE_ROOT" "$BLOCK_ROOT"
    worker_digest="$(file_digest "$SCRIPT_PATH")"
    if [[ ! -s "$EVIDENCE_ROOT/preflight-latest.json" ]] ||
        ! jq -e \
            --arg worker_digest "$worker_digest" \
            '.exit_reason == "success" and .worker_digest == $worker_digest' \
            "$EVIDENCE_ROOT/preflight-latest.json" >/dev/null; then
        die "current-worker isolation preflight is required before bootstrap"
    fi

    source_sha="$(bootstrap_source_digest)"
    fixture_sha="$(file_digest "$BOOTSTRAP_SCRIPT")"
    backend_sha="$(backend_digest)"
    previous_digest="$(previous_bootstrap_digest)"
    command_text="BOOTSTRAP_THREADS=1 $BOOTSTRAP_SCRIPT"
    command_digest="$({
        printf '%s\n' "$command_text"
        printf '%s\n' "$source_sha" "$worker_digest"
        printf '%s\n' \
            "$SMOKE_MEMORY_HIGH_BYTES" "$SMOKE_MEMORY_MAX_BYTES" \
            "$SMOKE_PROCESS_BYTES" 0 "$SMOKE_RUNTIME_SECONDS"
    } | sha256_text)"
    if [[ -s "$BLOCK_ROOT/$command_digest.json" ]]; then
        printf 'unchanged bootstrap command is retry-forbidden: %s\n' "$command_digest" >&2
        printf '%s\n' "$BLOCK_ROOT/$command_digest.json"
        return 125
    fi

    parent_cgroup="$(current_cgroup)"
    run_dir="$(new_run_dir bootstrap "$command_digest")"
    unit="$(unit_name bootstrap "$command_digest")"
    mkdir -p "$run_dir"
    output="$run_dir/systemd-run.log"
    set +e
    systemd_smoke_launch "$unit" "$SCRIPT_PATH" --internal-bootstrap \
        "$run_dir" "$parent_cgroup" "$unit" "$command_digest" "$previous_digest" \
        "$source_sha" "$fixture_sha" "$backend_sha" "$worker_digest" >"$output" 2>&1
    launch_status=$?
    set -e
    if [[ ! -s "$run_dir/resource-evidence.json" ]]; then
        write_smoke_failure \
            "$run_dir/resource-evidence.json" \
            "bootstrap service ended before typed evidence was written; see systemd-run.log" \
            "$worker_digest" "$parent_cgroup" "$unit.service" \
            "$command_digest" "$source_sha"
        cp "$run_dir/resource-evidence.json" "$BLOCK_ROOT/$command_digest.json"
        printf '%s\n' "$run_dir/resource-evidence.json"
        return 125
    fi
    reason="$(jq -r '.exit_reason' "$run_dir/resource-evidence.json")"
    printf '%s\n' "$run_dir/resource-evidence.json"
    if [[ "$reason" == success && "$launch_status" == 0 ]]; then
        return 0
    fi
    if jq -e '.is_retry_forbidden == true' "$run_dir/resource-evidence.json" >/dev/null; then
        return 125
    fi
    return 1
}

detached_unit_active() {
    local unit="$1" active_state
    active_state="$(
        systemctl --user show "$unit" --property=ActiveState --value 2>/dev/null || true
    )"
    [[ "$active_state" == active || "$active_state" == activating ||
        "$active_state" == reloading || "$active_state" == deactivating ]]
}

recover_detached_terminal() {
    local run_dir="$1" unit result exec_status exec_code exit_reason detail signal=null
    [[ -s "$run_dir/run-state.json" ]] || return 1
    [[ ! -s "$run_dir/resource-evidence.json" ]] || return 0
    unit="$(jq -r '.unit' "$run_dir/run-state.json")"
    detached_unit_active "$unit" && return 75
    result="$(systemctl --user show "$unit" --property=Result --value 2>/dev/null || true)"
    exec_status="$(
        systemctl --user show "$unit" --property=ExecMainStatus --value \
            2>/dev/null || true
    )"
    exec_code="$(
        systemctl --user show "$unit" --property=ExecMainCode --value \
            2>/dev/null || true
    )"
    exec_status="${exec_status:-null}"
    case "$result" in
        oom-kill)
            exit_reason=resource_oom
            detail="detached systemd worker ended with Result=oom-kill before its finalizer"
            ;;
        timeout)
            exit_reason=resource_timeout
            detail="detached systemd worker reached RuntimeMaxSec before its finalizer"
            ;;
        signal)
            if [[ "$exec_status" == 9 ]]; then
                exit_reason=resource_sigkill
                signal=9
            else
                exit_reason=isolation_unavailable
                [[ "$exec_status" =~ ^[0-9]+$ ]] && signal="$exec_status"
            fi
            detail="detached systemd worker ended by signal before typed telemetry was finalized"
            ;;
        *)
            exit_reason=isolation_unavailable
            detail="detached systemd worker became inactive without typed terminal evidence"
            ;;
    esac
    write_detached_terminal_failure \
        "$run_dir" "$exit_reason" \
        "$detail (Result=${result:-unavailable}, ExecMainCode=${exec_code:-unavailable})" \
        "$exec_status" "$signal"
}

find_active_heavy_run() {
    local state_file run_dir unit
    while IFS= read -r -d '' state_file; do
        jq -e '.state == "launching" or .state == "running"' \
            "$state_file" >/dev/null 2>&1 || continue
        run_dir="${state_file%/run-state.json}"
        unit="$(jq -r '.unit' "$state_file")"
        if detached_unit_active "$unit"; then
            printf '%s\n' "$run_dir"
            return 0
        fi
        recover_detached_terminal "$run_dir" >/dev/null 2>&1 || true
    done < <(
        find "$EVIDENCE_ROOT" -mindepth 2 -maxdepth 2 \
            -type f -name run-state.json -print0 2>/dev/null | sort -z
    )
    return 1
}

latest_run_for_test() {
    local test_name="$1" state_file latest=""
    while IFS= read -r -d '' state_file; do
        if jq -e --arg test_name "$test_name" \
            '.command.test == $test_name' "$state_file" >/dev/null 2>&1; then
            latest="${state_file%/run-state.json}"
        fi
    done < <(
        find "$EVIDENCE_ROOT" -mindepth 2 -maxdepth 2 \
            -type f -name run-state.json -print0 2>/dev/null | sort -z
    )
    [[ -n "$latest" ]] || die "no detached run found for exact test: $test_name"
    printf '%s\n' "$latest"
}

normalized_run_dir() {
    local requested="$1" resolved
    resolved="$(readlink -f "$requested" 2>/dev/null || true)"
    [[ -n "$resolved" ]] || die "run path does not exist: $requested"
    [[ -f "$resolved" ]] && resolved="${resolved%/*}"
    case "$resolved" in
        "$EVIDENCE_ROOT"/*) ;;
        *) die "run path is outside the canonical Phase 069 output root" ;;
    esac
    [[ -s "$resolved/run-state.json" ]] ||
        die "detached run state is missing: $resolved/run-state.json"
    printf '%s\n' "$resolved"
}

evidence_exit_status() {
    local evidence="$1" reason
    reason="$(jq -r '.exit_reason' "$evidence")"
    printf '%s\n' "$evidence"
    if [[ "$reason" == success ]]; then
        return 0
    fi
    if jq -e '.is_retry_forbidden == true' "$evidence" >/dev/null; then
        return 125
    fi
    return 1
}

status_detached_run() {
    local requested="$1" run_dir unit
    run_dir="$(normalized_run_dir "$requested")"
    if [[ -s "$run_dir/resource-evidence.json" ]]; then
        evidence_exit_status "$run_dir/resource-evidence.json"
        return
    fi
    unit="$(jq -r '.unit' "$run_dir/run-state.json")"
    if detached_unit_active "$unit"; then
        set_run_state \
            "$run_dir" running \
            "detached transient worker is active independently of its launcher" ""
        printf '%s\n' "$run_dir/run-state.json"
        return 0
    fi
    recover_detached_terminal "$run_dir" || true
    evidence_exit_status "$run_dir/resource-evidence.json"
}

wait_for_detached_run() {
    local run_dir="$1" unit
    unit="$(jq -r '.unit' "$run_dir/run-state.json")"
    while [[ ! -s "$run_dir/resource-evidence.json" ]]; do
        if ! detached_unit_active "$unit"; then
            recover_detached_terminal "$run_dir" || true
            break
        fi
        sleep "$STATUS_POLL_SECONDS"
    done
    evidence_exit_status "$run_dir/resource-evidence.json"
}

start_named_test() {
    local test_name="$1" worker_digest parent_cgroup run_dir unit source_sha fixture_sha backend_sha
    local command_text command_digest previous_digest output latest launch_status active_run
    host_preflight
    is_named_test "$test_name" || die "unapproved or non-exact heavy test: $test_name"
    mkdir -p "$EVIDENCE_ROOT" "$BLOCK_ROOT"
    if active_run="$(find_active_heavy_run)"; then
        die "another named heavy proof is already active: $active_run"
    fi
    worker_digest="$(file_digest "$SCRIPT_PATH")"
    latest="$EVIDENCE_ROOT/preflight-latest.json"
    [[ -s "$latest" ]] || die "positive isolation preflight is required before a real prover"
    jq -e \
        --arg worker_digest "$worker_digest" \
        '.exit_reason == "success" and .worker_digest == $worker_digest' \
        "$latest" >/dev/null || die "isolation preflight is stale or unsuccessful"

    source_sha="$(source_digest)"
    if is_source_diagnostic_test "$test_name" ||
        is_hash_diagnostic_test "$test_name" ||
        is_aggregation_diagnostic_test "$test_name"; then
        fixture_sha="$(file_digest "$BACKEND_SOURCE")"
    else
        fixture_sha="$(file_digest "$TEST_SOURCE")"
    fi
    backend_sha="$(backend_digest)"
    previous_digest="$(previous_test_digest "$test_name")"
    if is_lib_diagnostic_test "$test_name"; then
        command_text="cargo test --release --lib -p z00z_storage $(lib_diagnostic_filter "$test_name") -- --ignored --exact --nocapture --test-threads=1"
    else
        command_text="Z00Z_PLONKY3_RESOURCE_TELEMETRY=1 cargo test --release -p z00z_storage --test $TEST_TARGET $test_name -- --ignored --exact --nocapture --test-threads=1"
    fi
    command_digest="$({
        printf '%s\n' "$command_text"
        printf '%s\n' "$source_sha" "$worker_digest"
        printf '%s\n' "$MEMORY_HIGH_BYTES" "$MEMORY_MAX_BYTES" 0 "$RUNTIME_SECONDS"
    } | sha256_text)"
    if [[ -s "$BLOCK_ROOT/$command_digest.json" ]]; then
        printf 'unchanged terminal command is retry-forbidden: %s\n' "$command_digest" >&2
        printf '%s\n' "$BLOCK_ROOT/$command_digest.json"
        return 125
    fi

    parent_cgroup="$(current_cgroup)"
    run_dir="$(new_run_dir "$test_name" "$command_digest")"
    unit="$(unit_name run "$command_digest")"
    mkdir -p "$run_dir"
    output="$run_dir/systemd-run.log"
    write_run_state \
        "$run_dir" launching "$unit" "$test_name" "$command_digest" \
        "$previous_digest" "$source_sha" "$fixture_sha" "$backend_sha" \
        "$worker_digest" "$parent_cgroup"
    set +e
    systemd_heavy_start "$unit" "$run_dir/service.log" \
        "$SCRIPT_PATH" --internal-run \
        "$run_dir" "$parent_cgroup" "$unit" "$test_name" "$command_digest" \
        "$source_sha" "$fixture_sha" "$backend_sha" "$previous_digest" >"$output" 2>&1
    launch_status=$?
    set -e
    if (( launch_status != 0 )); then
        write_detached_terminal_failure \
            "$run_dir" isolation_unavailable \
            "detached transient service launch failed; see systemd-run.log" \
            "$launch_status" null
        STARTED_RUN_DIR="$run_dir"
        return 125
    fi
    set_run_state \
        "$run_dir" launching \
        "systemd accepted the detached transient worker; awaiting inner isolation verification" ""
    STARTED_RUN_DIR="$run_dir"
}

run_named_test() {
    local test_name="$1" start_status=0
    set +e
    start_named_test "$test_name"
    start_status=$?
    set -e
    [[ -n "$STARTED_RUN_DIR" ]] && printf '%s\n' "$STARTED_RUN_DIR"
    (( start_status == 0 )) || return "$start_status"
    wait_for_detached_run "$STARTED_RUN_DIR"
}

main() {
    cd "$ROOT_DIR"
    case "${1:-}" in
        --preflight)
            [[ "$#" == 1 ]] || die "--preflight accepts no arguments"
            run_preflight
            ;;
        --bootstrap)
            [[ "$#" == 1 ]] || die "--bootstrap accepts no arguments"
            run_bootstrap
            ;;
        --run)
            [[ "$#" == 2 ]] || die "--run requires one exact test name"
            run_named_test "$2"
            ;;
        --start)
            [[ "$#" == 2 ]] || die "--start requires one exact test name"
            local start_status=0
            set +e
            start_named_test "$2"
            start_status=$?
            set -e
            [[ -n "$STARTED_RUN_DIR" ]] && printf '%s\n' "$STARTED_RUN_DIR"
            return "$start_status"
            ;;
        --status)
            [[ "$#" == 2 ]] || die "--status requires one detached run path"
            status_detached_run "$2"
            ;;
        --status-latest)
            [[ "$#" == 2 ]] || die "--status-latest requires one exact test name"
            is_named_test "$2" || die "unapproved or non-exact heavy test: $2"
            status_detached_run "$(latest_run_for_test "$2")"
            ;;
        --internal-preflight)
            [[ "$#" == 5 ]] || die "invalid internal preflight arguments"
            internal_preflight "$2" "$3" "$4" "$5"
            ;;
        --internal-run)
            [[ "$#" == 10 ]] || die "invalid internal run arguments"
            internal_run "$2" "$3" "$4" "$5" "$6" "$7" "$8" "$9" "${10}"
            ;;
        --internal-bootstrap)
            [[ "$#" == 10 ]] || die "invalid internal bootstrap arguments"
            internal_bootstrap "$2" "$3" "$4" "$5" "$6" "$7" "$8" "$9" "${10}"
            ;;
        -h | --help)
            usage
            ;;
        *)
            usage >&2
            exit 2
            ;;
    esac
}

main "$@"
