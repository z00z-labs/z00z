#!/usr/bin/env bash
# Release-only, cache-bypassing RSS measurement for the clean Nova verifier.
# The proof-bound Rust owner is intentionally not instrumented: this harness
# discovers the clean child from its fail-closed environment marker and reads
# Linux /proc accounting while the complete production-parameter proof runs.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT_DIR"

readonly TEST_NAME="checkpoint::nova::tests::test_nova_checkpoint_proves_relation"
readonly VERIFIER_TEST_NAME="checkpoint::nova::tests::test_nova_clean_verifier_process"
readonly VERIFIER_MARKER="Z00Z_NOVA_VERIFIER_ONLY_V2=1"
readonly VERIFIER_BUNDLE_ENV="Z00Z_NOVA_VERIFIER_ONLY_BUNDLE_V2"
readonly EXPECTED_SOURCE_REVISION="2d4a6312028d3987520d10e53f376dd22b40e303fd0e7d1b122c900f0d9e55d8"
readonly EXPECTED_WORKER_SOURCE="a0fd346405c1f3d103d62b7d7b886574ad50d58dd749fcea22f8bf22960ade69"
readonly EXPECTED_NOVA_SHA256="ef88f863c74806b667858ab571f22772d40aadb7010d56231bb9d68020a7eb88"
readonly EXPECTED_CARGO_LOCK_SHA256="0abd89e2d1fdf007051e592db3e2f764262206dfc7050c8006bc8a33540eae8d"
readonly NOVA_SOURCE="crates/z00z_storage/src/checkpoint/nova.rs"
readonly WORKER_LOCK="target/workspace/z00z-nova-worker-v2.lock"

usage() {
    printf 'usage: %s [--check|--self-test]\n' "${0##*/}"
    printf '  --check  validate the harness and pinned inputs without running the proof\n'
    printf '  --self-test  prove discovery of a child spawned by a non-leader thread\n'
}

die() {
    printf 'nova verifier RSS measurement: %s\n' "$*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

preflight() {
    local nova_sha cargo_lock_sha

    [[ "$(uname -s)" == "Linux" ]] || die "Linux /proc is required"
    [[ -r /proc/self/status && -r /proc/self/environ ]] || die "required /proc process fields are unreadable"
    [[ -f "$NOVA_SOURCE" ]] || die "missing proof-bound source: $NOVA_SOURCE"
    [[ -f Cargo.lock ]] || die "missing Cargo.lock"
    grep -Fq \
        'const NOVA_VERIFIER_BUNDLE_PATH_V2: &str = "Z00Z_NOVA_VERIFIER_ONLY_BUNDLE_V2";' \
        "$NOVA_SOURCE" || die "clean-verifier bundle environment name drifted"
    for command in awk cargo cat chmod date dd flock grep head mkdir mv od pgrep ps readlink sed setsid sha256sum sleep tail tr uname; do
        require_command "$command"
    done

    nova_sha="$(sha256sum "$NOVA_SOURCE" | awk '{print $1}')"
    cargo_lock_sha="$(sha256sum Cargo.lock | awk '{print $1}')"

    [[ "$nova_sha" == "$EXPECTED_NOVA_SHA256" ]] || \
        die "proof-bound nova.rs identity drifted: expected $EXPECTED_NOVA_SHA256, got $nova_sha"
    [[ "$cargo_lock_sha" == "$EXPECTED_CARGO_LOCK_SHA256" ]] || \
        die "Cargo.lock identity drifted: expected $EXPECTED_CARGO_LOCK_SHA256, got $cargo_lock_sha"
}

proc_start_ticks() {
    local pid="$1" stat_line rest
    local -a fields
    if ! stat_line="$(cat "/proc/$pid/stat" 2>/dev/null)"; then
        return 1
    fi
    rest="${stat_line##*) }"
    read -r -a fields <<<"$rest"
    printf '%s\n' "${fields[19]:-unknown}"
}

proc_ppid() {
    local pid="$1" key value
    if [[ -r "/proc/$pid/status" ]]; then
        while IFS=: read -r key value; do
            if [[ "$key" == "PPid" ]]; then
                value="${value//[[:space:]]/}"
                printf '%s\n' "$value"
                return 0
            fi
        done <"/proc/$pid/status" 2>/dev/null || true
    fi
    return 0
}

proc_exe() {
    readlink -f "/proc/$1/exe" 2>/dev/null || printf 'unavailable'
}

proc_cmdline() {
    if [[ -r "/proc/$1/cmdline" ]]; then
        { tr '\0' ' ' <"/proc/$1/cmdline" | tr '\n\r\t' '   '; } 2>/dev/null || true
    fi
    return 0
}

proc_env_value() {
    local pid="$1" name="$2"
    if [[ -r "/proc/$pid/environ" ]]; then
        { tr '\0' '\n' <"/proc/$pid/environ" |
            sed -nE "s/^${name}=(.*)$/\\1/p" | head -n 1; } 2>/dev/null || true
    fi
    return 0
}

collect_tree() {
    local root="$1" pid child children children_path
    local -a queue=("$root")
    local index=0
    declare -A emitted=()

    while (( index < ${#queue[@]} )); do
        pid="${queue[$index]}"
        ((index += 1))
        [[ "$pid" =~ ^[0-9]+$ && -d "/proc/$pid" ]] || continue
        [[ -z "${emitted[$pid]:-}" ]] || continue
        emitted[$pid]=1
        printf '%s\n' "$pid"
        # A multithreaded parent records a child under the exact thread that
        # called clone/fork, not necessarily under the thread-group leader.
        # Scan every task so a Rust test thread cannot hide its worker tree.
        for children_path in "/proc/$pid/task/"[0-9]*/children; do
            [[ -r "$children_path" ]] || continue
            children="$(cat "$children_path" 2>/dev/null)" || children=""
            for child in $children; do
                queue+=("$child")
            done
        done
    done
}

self_test_thread_child_discovery() {
    local output bundle root child leader_children found pid parsed_source churn_root cleanup_root cleanup_pgid
    local -a churn_pids
    require_command python3
    require_command mktemp
    output="$(mktemp)"
    python3 -c 'import subprocess, threading
def spawn_from_thread():
    child = subprocess.Popen(["sleep", "2"])
    print(child.pid, flush=True)
    child.wait()
thread = threading.Thread(target=spawn_from_thread)
thread.start()
thread.join()' >"$output" &
    root=$!

    child=""
    for _ in {1..50}; do
        child="$(sed -nE '1s/^([0-9]+)$/\1/p' "$output")"
        [[ -n "$child" ]] && break
        sleep 0.02
    done
    if [[ -z "$child" ]]; then
        wait "$root" || true
        unlink "$output"
        die "self-test child PID was not emitted"
    fi

    leader_children="$(<"/proc/$root/task/$root/children")" 2>/dev/null || leader_children=""
    [[ " $leader_children " != *" $child "* ]] || {
        wait "$root" || true
        unlink "$output"
        die "self-test did not exercise a non-leader thread child"
    }

    found=false
    while IFS= read -r pid; do
        [[ "$pid" == "$child" ]] && found=true
    done < <(collect_tree "$root")
    wait "$root"
    unlink "$output"
    [[ "$found" == true ]] || die "all-thread traversal missed the synthetic child"

    # Exercise the exact verifier-bundle source-identity offset independently
    # of the long proof so a field-name/offset regression fails before launch.
    require_command xxd
    bundle="$(mktemp)"
    # Format-4 source_digest starts at byte 322 (after six prior digests).
    dd if=/dev/zero of="$bundle" bs=1 count=322 status=none
    printf '%s' "$EXPECTED_SOURCE_REVISION" | xxd -r -p >>"$bundle"
    parsed_source="$(extract_bundle_source_revision "$bundle")"
    unlink "$bundle"
    [[ "$parsed_source" == "$EXPECTED_SOURCE_REVISION" ]] || \
        die "verifier-bundle source identity parser self-test failed"

    # Repeatedly sample a multithreaded parent creating very short-lived
    # children. Every /proc helper must tolerate a PID vanishing after tree
    # enumeration without leaking ENOENT or terminating the monitor.
    python3 -c 'import subprocess, threading
def churn():
    for _ in range(100):
        subprocess.run(["sleep", "0.002"], check=True)
thread = threading.Thread(target=churn)
thread.start()
thread.join()' >/dev/null 2>&1 &
    churn_root=$!
    for _ in {1..100}; do
        mapfile -t churn_pids < <(collect_tree "$churn_root")
        for pid in "${churn_pids[@]}"; do
            proc_start_ticks "$pid" >/dev/null || true
            proc_ppid "$pid" >/dev/null
            proc_exe "$pid" >/dev/null
            proc_cmdline "$pid" >/dev/null
            proc_env_value "$pid" PATH >/dev/null
            read_rss_fields "$pid" >/dev/null
            read_wait_status_if_zombie "$pid" >/dev/null || true
        done
        sleep 0.002
    done
    wait "$churn_root"
    for _ in {1..100}; do
        proc_start_ticks 999999999 >/dev/null || true
        proc_ppid 999999999 >/dev/null
        proc_exe 999999999 >/dev/null
        proc_cmdline 999999999 >/dev/null
        proc_env_value 999999999 PATH >/dev/null
        read_rss_fields 999999999 >/dev/null
        read_wait_status_if_zombie 999999999 >/dev/null || true
    done

    setsid bash -c 'sleep 30 & wait' >/dev/null 2>&1 &
    cleanup_root=$!
    cleanup_pgid=""
    for _ in {1..50}; do
        cleanup_pgid="$(ps -o pgid= -p "$cleanup_root" 2>/dev/null | tr -d '[:space:]')" || cleanup_pgid=""
        [[ "$cleanup_pgid" == "$cleanup_root" ]] && break
        sleep 0.02
    done
    [[ "$cleanup_pgid" == "$cleanup_root" ]] || die "process-group cleanup self-test was not isolated"
    terminate_owned_process_group "$cleanup_root"
    wait "$cleanup_root" 2>/dev/null || true
    ! pgrep -g "$cleanup_root" >/dev/null 2>&1 || die "process-group cleanup self-test left descendants"
    printf 'nova verifier RSS harness all-thread traversal self-test: PASS\n'
    printf 'nova verifier RSS harness bundle identity parser self-test: PASS\n'
    printf 'nova verifier RSS harness vanished-PID stress self-test: PASS (100 churn + 100 absent)\n'
    printf 'nova verifier RSS harness isolated process-group cleanup self-test: PASS\n'
}

read_rss_fields() {
    local pid="$1"
    sed -nE \
        -e 's/^VmRSS:[[:space:]]*([0-9]+)[[:space:]]+kB.*$/rss=\1/p' \
        -e 's/^VmHWM:[[:space:]]*([0-9]+)[[:space:]]+kB.*$/hwm=\1/p' \
        "/proc/$pid/status" 2>/dev/null || true
}

read_wait_status_if_zombie() {
    local pid="$1" stat_line rest state
    local -a fields
    if ! stat_line="$(cat "/proc/$pid/stat" 2>/dev/null)"; then
        return 1
    fi
    rest="${stat_line##*) }"
    read -r -a fields <<<"$rest"
    state="${fields[0]:-}"
    [[ "$state" == "Z" ]] || return 1
    # /proc/PID/stat field 52 is the raw wait status; fields[] starts at field 3.
    printf '%s\n' "${fields[49]:-unknown}"
}

record_lineage() {
    local pid="$1" root="$2" current ppid exe cmd
    local depth=0
    current="$pid"
    : >"$LINEAGE"
    while [[ "$current" =~ ^[0-9]+$ ]] && (( current > 0 )) && (( depth < 32 )); do
        ppid="$(proc_ppid "$current")"
        exe="$(proc_exe "$current")"
        cmd="$(proc_cmdline "$current")"
        printf 'depth=%s pid=%s ppid=%s exe=%q cmd=%q\n' \
            "$depth" "$current" "${ppid:-unknown}" "$exe" "$cmd" >>"$LINEAGE"
        [[ "$current" == "$root" ]] && return 0
        [[ "$ppid" =~ ^[0-9]+$ ]] || break
        current="$ppid"
        ((depth += 1))
    done
    return 1
}

extract_bundle_source_revision() {
    local bundle="$1" suite_id feature_id prefix_bytes source_offset
    suite_id='nova-snark/0.73.0;pallas-vesta;spartan-snark-ipa'
    feature_id='io'
    # Header before its digest array: magic/version bytes, two length-prefixed
    # fixed IDs, width/arity, three activation fields, and four shape fields.
    prefix_bytes=$((8 + 4 + 1 + ${#suite_id} + 1 + ${#feature_id} + 2 + 8 + 3 * 8 + 4 * 8))
    source_offset=$((prefix_bytes + 6 * 32))
    dd if="$bundle" bs=1 skip="$source_offset" count=32 status=none 2>/dev/null |
        od -An -tx1 | tr -d ' \n'
}

proc_state() {
    local pid="$1" key value
    if [[ -r "/proc/$pid/status" ]]; then
        while IFS=: read -r key value; do
            if [[ "$key" == "State" ]]; then
                value="${value//[[:space:]]/}"
                printf '%s\n' "${value:0:1}"
                return 0
            fi
        done <"/proc/$pid/status" 2>/dev/null || true
    fi
    return 0
}

persist_partial_observation() {
    local status="$1" temp_file="${PARTIAL_REPORT}.tmp"
    local rss_kib="${verifier_peak_rss_kib:-0}" hwm_kib="${verifier_peak_hwm_kib:-0}"
    local marker_count=0
    if declare -p marker_process_ids >/dev/null 2>&1; then
        marker_count="${#marker_process_ids[@]}"
    fi
    {
        printf 'measurement_status=%s\n' "$status"
        printf 'measurement_schema=z00z.recursive.v2.clean-verifier-rss.procfs.v1\n'
        printf 'measurement_start_utc=%s\n' "${measurement_start_utc:-missing}"
        printf 'updated_utc=%s\n' "$(date -u +'%Y-%m-%dT%H:%M:%S.%3NZ')"
        printf 'cargo_pid=%s\n' "${cargo_pid:-missing}"
        printf 'proof_parent_pid=%s\n' "${proof_parent_pid:-missing}"
        printf 'verifier_pid=%s\n' "${verifier_pid:-missing}"
        printf 'verifier_ppid=%s\n' "${verifier_ppid:-missing}"
        printf 'verifier_process_start_ticks=%s\n' "${verifier_start_ticks:-missing}"
        printf 'verifier_observed_start_utc=%s\n' "${verifier_start_utc:-missing}"
        printf 'verifier_observed_end_utc=%s\n' "${verifier_end_utc:-missing}"
        printf 'verifier_marker_processes=%s\n' "$marker_count"
        printf 'verifier_peak_vmrss_kib=%s\n' "$rss_kib"
        printf 'verifier_peak_vmrss_bytes=%s\n' "$((rss_kib * 1024))"
        printf 'verifier_peak_vmhwm_kib=%s\n' "$hwm_kib"
        printf 'verifier_peak_vmhwm_bytes=%s\n' "$((hwm_kib * 1024))"
        printf 'source_revision_digest=%s\n' "${measured_source_revision:-missing}"
        printf 'worker_source_digest=%s\n' "${measured_worker_source:-missing}"
        printf 'measurement_bundle_sha256=%s\n' "${measurement_bundle_sha256:-missing}"
        printf 'failure_reason=%s\n' "${failure_reason:-none}"
        printf 'process_group_cleanup=%s\n' "${process_group_cleanup:-pending}"
    } >"$temp_file"
    chmod 0600 "$temp_file"
    mv "$temp_file" "$PARTIAL_REPORT"
}

terminate_owned_process_group() {
    local group_id="$1" member
    kill -TERM -- "-$group_id" 2>/dev/null || true
    for _ in {1..150}; do
        member="$(pgrep -g "$group_id" 2>/dev/null | head -n 1)" || member=""
        [[ -z "$member" ]] && break
        sleep 0.1
    done
    if pgrep -g "$group_id" >/dev/null 2>&1; then
        kill -KILL -- "-$group_id" 2>/dev/null || true
    fi
}

cleanup_process_group_on_exit() {
    local exit_status=$?
    trap - EXIT INT TERM HUP
    set +e
    if [[ "${cleanup_enabled:-false}" == "true" && "${cargo_pid:-}" =~ ^[0-9]+$ ]]; then
        terminate_owned_process_group "$cargo_pid"
        wait "$cargo_pid" 2>/dev/null || true
        process_group_cleanup="forced_by_exit_trap"
    fi
    if [[ -n "${PARTIAL_REPORT:-}" && ! -f "${REPORT:-/nonexistent}" ]]; then
        failure_reason="${failure_reason:-monitor exited unexpectedly with status $exit_status}"
        persist_partial_observation ABORTED
    fi
    exit "$exit_status"
}

write_report() {
    local status="$1" end_utc="$2" end_ms="$3" elapsed_ms="$4"
    local rss_bytes=$((verifier_peak_rss_kib * 1024))
    local hwm_bytes=$((verifier_peak_hwm_kib * 1024))
    local temp_file="${REPORT}.tmp"

    {
        printf 'measurement_status=%s\n' "$status"
        printf 'measurement_schema=z00z.recursive.v2.clean-verifier-rss.procfs.v1\n'
        printf 'measurement_start_utc=%s\n' "$measurement_start_utc"
        printf 'measurement_end_utc=%s\n' "$end_utc"
        printf 'measurement_end_epoch_ms=%s\n' "$end_ms"
        printf 'measurement_elapsed_ms=%s\n' "$elapsed_ms"
        printf 'cargo_pid=%s\n' "$cargo_pid"
        printf 'cargo_terminal_exit_code=%s\n' "$cargo_exit"
        printf 'proof_parent_pid=%s\n' "${proof_parent_pid:-missing}"
        printf 'proof_parent_terminal_exit_code=%s\n' "${proof_parent_exit:-missing}"
        printf 'proof_parent_exit_evidence=%s\n' "${proof_parent_exit_evidence:-missing}"
        printf 'verifier_pid=%s\n' "${verifier_pid:-missing}"
        printf 'verifier_ppid=%s\n' "${verifier_ppid:-missing}"
        printf 'verifier_process_start_ticks=%s\n' "${verifier_start_ticks:-missing}"
        printf 'verifier_observed_start_utc=%s\n' "${verifier_start_utc:-missing}"
        printf 'verifier_observed_end_utc=%s\n' "${verifier_end_utc:-missing}"
        printf 'verifier_observed_elapsed_ms=%s\n' "${verifier_observed_elapsed_ms:-missing}"
        printf 'verifier_terminal_exit_code=%s\n' "${verifier_exit:-missing}"
        printf 'verifier_exit_evidence=%s\n' "${verifier_exit_evidence:-missing}"
        printf 'verifier_zombie_wait_status_raw=%s\n' "${verifier_zombie_wait_status:-not_observed}"
        printf 'verifier_marker_processes=%s\n' "$marker_processes"
        printf 'verifier_peak_vmrss_kib=%s\n' "$verifier_peak_rss_kib"
        printf 'verifier_peak_vmrss_bytes=%s\n' "$rss_bytes"
        printf 'verifier_peak_vmhwm_kib=%s\n' "$verifier_peak_hwm_kib"
        printf 'verifier_peak_vmhwm_bytes=%s\n' "$hwm_bytes"
        printf 'clean_verify_ms=%s\n' "${clean_verify_ms:-missing}"
        printf 'source_revision_digest=%s\n' "${measured_source_revision:-missing}"
        printf 'worker_source_digest=%s\n' "${measured_worker_source:-missing}"
        printf 'nova_source_sha256=%s\n' "$EXPECTED_NOVA_SHA256"
        printf 'cargo_lock_sha256=%s\n' "$EXPECTED_CARGO_LOCK_SHA256"
        printf 'measurement_bundle_sha256=%s\n' "${measurement_bundle_sha256:-missing}"
        printf 'test_name=%s\n' "$TEST_NAME"
        printf 'release_profile=true\n'
        printf 'test_threads=1\n'
        printf 'nova_runtime_cache=none\n'
        printf 'worker_rlimit_as_bytes=25769803776\n'
        printf 'worker_timeout_seconds=3600\n'
        printf 'process_group_id=%s\n' "$cargo_pid"
        printf 'process_group_cleanup=%s\n' "${process_group_cleanup:-unknown}"
        printf 'worker_lock_after=%s\n' "${worker_lock_after:-unknown}"
        printf 'partial_report=%s\n' "$PARTIAL_REPORT"
        printf 'lineage_file=%s\n' "$LINEAGE"
        printf 'process_log=%s\n' "$PROCESS_LOG"
        printf 'transcript_file=%s\n' "$TRANSCRIPT"
        printf 'failure_reason=%s\n' "${failure_reason:-none}"
    } >"$temp_file"
    chmod 0600 "$temp_file"
    mv "$temp_file" "$REPORT"
}

mode="measure"
if (( $# > 1 )); then
    usage >&2
    exit 2
elif (( $# == 1 )); then
    case "$1" in
        --check) mode="check" ;;
        --self-test) mode="self-test" ;;
        -h|--help) usage; exit 0 ;;
        *) usage >&2; exit 2 ;;
    esac
fi

preflight
if [[ "$mode" == "check" ]]; then
    printf 'nova verifier RSS harness preflight: PASS\n'
    exit 0
fi
if [[ "$mode" == "self-test" ]]; then
    self_test_thread_child_discovery
    exit 0
fi

umask 077
run_stamp="$(date -u +'%Y%m%dT%H%M%SZ')"
PHASE069_OUTPUT_ROOT="$ROOT_DIR/crates/z00z_storage/outputs/checkpoint"
RUN_DIR="$(realpath -m -- "${NOVA_VERIFIER_RSS_OUTPUT_DIR:-$PHASE069_OUTPUT_ROOT/nova-verifier-rss/$run_stamp-$$}")"
case "$RUN_DIR" in
  "$PHASE069_OUTPUT_ROOT" | "$PHASE069_OUTPUT_ROOT"/*) ;;
  *) die "output path must stay under $PHASE069_OUTPUT_ROOT: $RUN_DIR" ;;
esac
mkdir -p "$RUN_DIR"
chmod 0700 "$RUN_DIR"
TRANSCRIPT="$RUN_DIR/proof-transcript.log"
LINEAGE="$RUN_DIR/verifier-lineage.log"
PROCESS_LOG="$RUN_DIR/process-tree.log"
REPORT="$RUN_DIR/measurement.env"
PARTIAL_REPORT="$RUN_DIR/measurement.partial.env"
: >"$TRANSCRIPT"
: >"$LINEAGE"
: >"$PROCESS_LOG"
chmod 0600 "$TRANSCRIPT" "$LINEAGE" "$PROCESS_LOG"

measurement_start_utc="$(date -u +'%Y-%m-%dT%H:%M:%S.%3NZ')"
measurement_start_ms="$(date -u +%s%3N)"
printf 'Nova verifier RSS measurement started at %s\n' "$measurement_start_utc"
printf 'Artifacts: %s\n' "$RUN_DIR"

declare -A observed_processes=()
declare -A marker_process_ids=()
cargo_pid=""
cleanup_enabled=false
process_group_cleanup="pending"
worker_lock_after="unknown"
verifier_pid=""
verifier_ppid=""
verifier_start_ticks=""
verifier_start_utc=""
verifier_start_ms=""
verifier_end_utc=""
verifier_end_ms=""
verifier_observed_elapsed_ms=""
verifier_peak_rss_kib=0
verifier_peak_hwm_kib=0
persisted_hwm_kib=0
verifier_zombie_wait_status=""
proof_parent_pid=""
proof_parent_exit=""
proof_parent_exit_evidence=""
measured_worker_source=""
measured_source_revision=""
measurement_bundle_sha256=""
failure_reason=""
marker_processes=0

trap cleanup_process_group_on_exit EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

# This cache-bypassing measurement never activates the private PP/PK cache;
# proof and verdict caching have no API. CARGO_INCREMENTAL=0 additionally keeps
# compiler incremental state from obscuring the release invocation identity.
setsid env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$ROOT_DIR/target/workspace" \
    cargo test --release -p z00z_storage --lib "$TEST_NAME" -- \
        --exact --nocapture --test-threads 1 --ignored >"$TRANSCRIPT" 2>&1 &
cargo_pid=$!
cleanup_enabled=true
cargo_pgid=""
for _ in {1..50}; do
    cargo_pgid="$(ps -o pgid= -p "$cargo_pid" 2>/dev/null | tr -d '[:space:]')" || cargo_pgid=""
    [[ -n "$cargo_pgid" ]] && break
    sleep 0.02
done
[[ "$cargo_pgid" == "$cargo_pid" ]] || \
    die "cargo process group isolation failed: pid=$cargo_pid pgid=${cargo_pgid:-missing}"
persist_partial_observation RUNNING

while [[ -d "/proc/$cargo_pid" ]]; do
    mapfile -t tree_pids < <(collect_tree "$cargo_pid")
    declare -A current_tree=()
    for pid in "${tree_pids[@]}"; do
        current_tree[$pid]=1
        start_ticks="$(proc_start_ticks "$pid")" || start_ticks="unknown"
        process_key="$pid:$start_ticks"
        if [[ -z "${observed_processes[$process_key]:-}" ]]; then
            observed_processes[$process_key]=1
            exe="$(proc_exe "$pid")"
            cmd="$(proc_cmdline "$pid")"
            ppid="$(proc_ppid "$pid")"
            printf 'observed_utc=%s pid=%s ppid=%s start_ticks=%s exe=%q cmd=%q\n' \
                "$(date -u +'%Y-%m-%dT%H:%M:%S.%3NZ')" "$pid" "${ppid:-unknown}" \
                "$start_ticks" "$exe" "$cmd" >>"$PROCESS_LOG"

            if grep -azFxq "$VERIFIER_MARKER" "/proc/$pid/environ" 2>/dev/null; then
                marker_process_ids[$process_key]=1
                if [[ "$cmd" == *"$VERIFIER_TEST_NAME"* && "$exe" != */timeout ]]; then
                    if [[ -n "$verifier_pid" && "$verifier_pid" != "$pid" ]]; then
                        failure_reason="multiple clean verifier process identities observed: $verifier_pid and $pid"
                    elif [[ -z "$verifier_pid" ]]; then
                        verifier_pid="$pid"
                        verifier_ppid="${ppid:-unknown}"
                        verifier_start_ticks="$start_ticks"
                        verifier_start_utc="$(date -u +'%Y-%m-%dT%H:%M:%S.%3NZ')"
                        verifier_start_ms="$(date -u +%s%3N)"
                        record_lineage "$verifier_pid" "$cargo_pid" || \
                            failure_reason="clean verifier lineage did not reach cargo root"

                        bundle_path="$(proc_env_value "$pid" "$VERIFIER_BUNDLE_ENV")"
                        if [[ -r "$bundle_path" ]]; then
                            measured_source_revision="$(extract_bundle_source_revision "$bundle_path")" || measured_source_revision=""
                            measurement_bundle_sha256="$(sha256sum "$bundle_path" | awk '{print $1}')" || measurement_bundle_sha256=""
                        else
                            failure_reason="clean verifier bundle path was absent or unreadable"
                        fi
                        persist_partial_observation VERIFIER_IDENTIFIED
                    fi
                fi
            fi
        fi
    done

    if [[ -z "$proof_parent_pid" && -r "$WORKER_LOCK" ]]; then
        lock_pid="$(sed -nE 's/^pid=([0-9]+)$/\1/p' "$WORKER_LOCK" | head -n 1)" || lock_pid=""
        if [[ -n "$lock_pid" && -n "${current_tree[$lock_pid]:-}" ]]; then
            proof_parent_pid="$lock_pid"
            measured_worker_source="$(sed -nE 's/^source_digest=([0-9a-f]{64})$/\1/p' "$WORKER_LOCK" | head -n 1)" || measured_worker_source=""
        fi
    fi

    if [[ -n "$verifier_pid" && -z "$verifier_end_ms" && -r "/proc/$verifier_pid/status" ]]; then
        current_start_ticks="$(proc_start_ticks "$verifier_pid")" || current_start_ticks=""
        if [[ "$current_start_ticks" != "$verifier_start_ticks" ]]; then
            failure_reason="clean verifier PID identity changed: pid=$verifier_pid expected_start=$verifier_start_ticks observed_start=${current_start_ticks:-missing}"
            verifier_end_utc="$(date -u +'%Y-%m-%dT%H:%M:%S.%3NZ')"
            verifier_end_ms="$(date -u +%s%3N)"
            verifier_observed_elapsed_ms=$((verifier_end_ms - verifier_start_ms))
            persist_partial_observation VERIFIER_EXITED
        else
            rss_kib=0
            hwm_kib=0
            while IFS='=' read -r key value; do
                case "$key" in
                    rss) rss_kib="$value" ;;
                    hwm) hwm_kib="$value" ;;
                esac
            done < <(read_rss_fields "$verifier_pid")
            (( rss_kib > verifier_peak_rss_kib )) && verifier_peak_rss_kib="$rss_kib"
            (( hwm_kib > verifier_peak_hwm_kib )) && verifier_peak_hwm_kib="$hwm_kib"
            if (( verifier_peak_hwm_kib >= persisted_hwm_kib + 16384 )); then
                persisted_hwm_kib="$verifier_peak_hwm_kib"
                persist_partial_observation VERIFIER_RUNNING
            fi
            zombie_status="$(read_wait_status_if_zombie "$verifier_pid")" || zombie_status=""
            [[ -n "$zombie_status" ]] && verifier_zombie_wait_status="$zombie_status"
        fi
    elif [[ -n "$verifier_pid" && -z "$verifier_end_ms" ]]; then
        verifier_end_utc="$(date -u +'%Y-%m-%dT%H:%M:%S.%3NZ')"
        verifier_end_ms="$(date -u +%s%3N)"
        verifier_observed_elapsed_ms=$((verifier_end_ms - verifier_start_ms))
        persist_partial_observation VERIFIER_EXITED
    fi

    root_state="$(proc_state "$cargo_pid")"
    [[ "$root_state" == "Z" ]] && break
    if [[ -n "$verifier_pid" && -z "$verifier_end_ms" ]]; then
        sleep 0.02
    else
        sleep 0.10
    fi
done

if wait "$cargo_pid"; then
    cargo_exit=0
else
    cargo_exit=$?
fi

remaining_group_members="$(pgrep -g "$cargo_pid" 2>/dev/null | tr '\n' ' ')" || remaining_group_members=""
if [[ -n "$remaining_group_members" ]]; then
    failure_reason="process group retained members after cargo exit: $remaining_group_members"
    process_group_cleanup="pending_failure_cleanup"
else
    cleanup_enabled=false
    process_group_cleanup="clean"
fi
if flock -n "$WORKER_LOCK" -c true; then
    worker_lock_after="free"
else
    worker_lock_after="held"
    failure_reason="Nova worker lock remained held after cargo exit${failure_reason:+; $failure_reason}"
fi

measurement_end_utc="$(date -u +'%Y-%m-%dT%H:%M:%S.%3NZ')"
measurement_end_ms="$(date -u +%s%3N)"
measurement_elapsed_ms=$((measurement_end_ms - measurement_start_ms))
marker_processes="${#marker_process_ids[@]}"
clean_verify_ms="$(sed -nE 's/.*mixed Nova worker stage=clean-verifier-only .*clean_verify_ms=([0-9]+).*/\1/p' "$TRANSCRIPT" | tail -n 1)"

if [[ -n "$verifier_pid" && -z "$verifier_end_ms" ]]; then
    verifier_end_utc="$measurement_end_utc"
    verifier_end_ms="$measurement_end_ms"
    verifier_observed_elapsed_ms=$((verifier_end_ms - verifier_start_ms))
fi

if (( cargo_exit == 0 )) && \
    grep -Fq 'clean verifier-only process: steps=1727' "$TRANSCRIPT" && \
    grep -Fq 'mixed Nova worker stage=clean-verifier-only' "$TRANSCRIPT"; then
    verifier_exit=0
    verifier_exit_evidence="rust_parent_asserted_success_then_full_cargo_test_exited_zero"
else
    verifier_exit="$cargo_exit"
    verifier_exit_evidence="clean_verifier_success_stage_or_parent_success_missing"
fi

if (( cargo_exit == 0 )) && \
    grep -Fq 'worker-confirmed mixed Nova proof:' "$TRANSCRIPT" && \
    grep -Fq 'test result: ok. 1 passed;' "$TRANSCRIPT"; then
    proof_parent_exit=0
    proof_parent_exit_evidence="bounded_worker_asserted_success_then_cargo_test_exited_zero"
else
    proof_parent_exit="$cargo_exit"
    proof_parent_exit_evidence="bounded_worker_success_or_cargo_success_missing"
fi

if [[ -n "$failure_reason" ]]; then
    :
elif (( cargo_exit != 0 )); then
    failure_reason="cargo test exited $cargo_exit${failure_reason:+; $failure_reason}"
elif [[ -z "$verifier_pid" ]]; then
    failure_reason="clean verifier executable with exact environment marker was never observed"
elif [[ "$marker_processes" -lt 2 ]]; then
    failure_reason="expected marked timeout wrapper and clean verifier child were not both observed"
elif [[ "$verifier_peak_rss_kib" -le 0 || "$verifier_peak_hwm_kib" -le 0 ]]; then
    failure_reason="clean verifier VmRSS/VmHWM was not observed"
elif [[ "$measured_source_revision" != "$EXPECTED_SOURCE_REVISION" ]]; then
    failure_reason="bundle source revision mismatch: expected $EXPECTED_SOURCE_REVISION, got ${measured_source_revision:-missing}"
elif [[ "$measured_worker_source" != "$EXPECTED_WORKER_SOURCE" ]]; then
    failure_reason="worker source mismatch: expected $EXPECTED_WORKER_SOURCE, got ${measured_worker_source:-missing}"
elif [[ -z "$clean_verify_ms" ]]; then
    failure_reason="clean verifier terminal success stage was not observed"
elif [[ "$verifier_exit" != "0" || "$proof_parent_exit" != "0" ]]; then
    failure_reason="clean verifier or proof parent did not terminate successfully"
fi

if [[ -n "$failure_reason" ]]; then
    write_report FAIL "$measurement_end_utc" "$measurement_end_ms" "$measurement_elapsed_ms"
    tail -n 200 "$TRANSCRIPT" >&2
    printf 'Measurement failed: %s\nReport: %s\n' "$failure_reason" "$REPORT" >&2
    exit 1
fi

write_report PASS "$measurement_end_utc" "$measurement_end_ms" "$measurement_elapsed_ms"
printf 'Nova verifier RSS measurement: PASS\n'
printf 'VmRSS peak: %s KiB (%s bytes)\n' "$verifier_peak_rss_kib" "$((verifier_peak_rss_kib * 1024))"
printf 'VmHWM peak: %s KiB (%s bytes)\n' "$verifier_peak_hwm_kib" "$((verifier_peak_hwm_kib * 1024))"
printf 'Clean verifier: %s ms; complete invocation: %s ms\n' "$clean_verify_ms" "$measurement_elapsed_ms"
printf 'Report: %s\n' "$REPORT"
