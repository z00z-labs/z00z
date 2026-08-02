#!/usr/bin/env bash
# Focused guards for canonical bootstrap source closure and typed drift evidence.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT_DIR"

readonly AUTHORITY="./.github/skills/smart-tests-bootstrap/scripts/bootstrap_source_authority.sh"
readonly BOOTSTRAP="./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh"
readonly CACHE_HELPER="./.github/skills/smart-tests-bootstrap/scripts/bootstrap_cache_identity.sh"
readonly WORKER="./.github/skills/smart-tests-bootstrap/scripts/plonky3_resource_worker.sh"
readonly MILESTONE="./.github/skills/smart-tests-bootstrap/scripts/plonky3_milestone_tests.sh"
readonly DRIFT_FIXTURE="./.github/skills/smart-tests-bootstrap/scripts/fixtures/bootstrap_source_authority_drift_fixture.sh"
readonly CHECKPOINT_OUTPUT_ROOT="$ROOT_DIR/crates/z00z_storage/outputs/checkpoint"
readonly CACHE_AUTHORITY_ROOT="$ROOT_DIR/.cache"
readonly FOCUSED_TMP_ROOT="$CHECKPOINT_OUTPUT_ROOT/069-08/task-1/diagnostics/bootstrap-focused-tests"
readonly FOCUSED_CACHE_ROOT="$CACHE_AUTHORITY_ROOT/phase-069/plan-08/focused-tests"
HOST_TARGET="$(
    rustc -vV |
        awk -F ': ' '$1 == "host" { print $2 }'
)"
readonly HOST_TARGET
[[ "$HOST_TARGET" =~ ^[A-Za-z0-9_][A-Za-z0-9_.-]*$ ]] || {
    printf 'invalid rustc host target: %s\n' "$HOST_TARGET" >&2
    exit 1
}
command -v gio >/dev/null 2>&1 || {
    printf 'required command not found: gio\n' >&2
    exit 1
}
mkdir -p "$FOCUSED_TMP_ROOT" "$FOCUSED_CACHE_ROOT"
TMP_DIR="$(mktemp -d "$FOCUSED_TMP_ROOT/run.XXXXXX")"
readonly TMP_DIR
CACHE_TMP_DIR="$(mktemp -d "$FOCUSED_CACHE_ROOT/run.XXXXXX")"
readonly CACHE_TMP_DIR
cleanup_tmp() {
    local path root
    for path in "$TMP_DIR" "$CACHE_TMP_DIR"; do
        case "$path" in
            "$FOCUSED_TMP_ROOT"/run.*)
                root="$FOCUSED_TMP_ROOT"
                ;;
            "$FOCUSED_CACHE_ROOT"/run.*)
                root="$FOCUSED_CACHE_ROOT"
                ;;
            *)
                printf 'refusing to clean non-canonical focused test path: %s\n' \
                    "$path" >&2
                return 1
                ;;
        esac
        [[ "$path" == "$root"/run.* ]] || return 1
        if [[ -e "$path" ]] && ! gio trash -- "$path" 2>/dev/null; then
            printf 'focused test scratch retained for evidence: %s\n' \
                "$path" >&2
        fi
    done
}
trap cleanup_tmp EXIT

assert_absent() {
    local needle="$1" file="$2"
    if grep -Fq -- "$needle" "$file"; then
        printf 'unexpected text %q in %s\n' "$needle" "$file" >&2
        exit 1
    fi
}

for script in \
    "$AUTHORITY" "$BOOTSTRAP" "$CACHE_HELPER" "$WORKER" "$MILESTONE" "$DRIFT_FIXTURE"
do
    bash -n "$script"
done

"$AUTHORITY" packages >"$TMP_DIR/actual-packages.tsv"
cargo metadata \
    --format-version 1 \
    --locked \
    --offline \
    --filter-platform "$HOST_TARGET" |
    jq -r \
      --arg root_name z00z_storage \
      --arg root "$ROOT_DIR/" \
      --arg app_root "$(dirname "$ROOT_DIR")/z00z-app/" '
        (.packages | map({key: .id, value: .}) | from_entries) as $packages
        | (
            .resolve.nodes
            | map({
                key: .id,
                value: {
                    all: [.deps[]?.pkg],
                    non_dev: [
                        .deps[]?
                        | select(any(
                            .dep_kinds[]?;
                            (.kind // "normal") != "dev"
                        ))
                        | .pkg
                    ]
                }
            })
            | from_entries
          ) as $edges
        | def closure($ids):
            (
                (
                    $ids
                    + [$ids[] as $id | $edges[$id].non_dev[]?]
                )
                | unique
            ) as $next
            | if $next == $ids then $ids else closure($next) end;
          [
            $packages
            | to_entries[]
            | select(.value.name == $root_name and .value.source == null)
            | .key
          ] as $roots
        | (
            (
                $roots
                + [$roots[] as $id | $edges[$id].all[]?]
            )
            | unique
            | closure(.)
          )[]
        | $packages[.]
        | select(.source == null)
        | [
            .name,
            (
                if (.manifest_path | startswith($root)) then
                    .manifest_path | ltrimstr($root)
                elif (.manifest_path | startswith($app_root)) then
                    "../z00z-app/" + (.manifest_path | ltrimstr($app_root))
                else
                    error("unexpected local dependency root: " + .manifest_path)
                end
            )
          ]
        | @tsv
    ' |
    LC_ALL=C sort -t $'\t' -k2,2 >"$TMP_DIR/expected-packages.tsv"
diff -u "$TMP_DIR/expected-packages.tsv" "$TMP_DIR/actual-packages.tsv"
[[ "$(wc -l <"$TMP_DIR/actual-packages.tsv")" == 12 ]]
assert_absent $'z00z_ui_ux\t' "$TMP_DIR/actual-packages.tsv"
assert_absent $'z00z-app-rpc\t' "$TMP_DIR/actual-packages.tsv"

"$AUTHORITY" manifest >"$TMP_DIR/manifest.tsv"
LC_ALL=C sort -c -u "$TMP_DIR/manifest.tsv"
cut -f1 "$TMP_DIR/manifest.tsv" >"$TMP_DIR/paths.txt"
for required_path in \
    ".cargo/config.toml" \
    "Cargo.lock" \
    "crates/z00z_core/src/actions/action_descriptor.rs" \
    "crates/z00z_wallets/src/rpc/object_rpc_impl.rs" \
    "crates/z00z_storage/tests/test_recursive_v2_nova_step.rs" \
    "crates/z00z_storage/tests/test_recursive_v2_nova_adversarial.rs" \
    "../z00z-app/Cargo.toml" \
    "../z00z-app/crates/z00z_app_api/Cargo.toml" \
    "../z00z-app/crates/z00z_app_api/src/action_basis_manifest.rs" \
    "../z00z-app/crates/z00z_app_api/outputs/action-basis-manifest-v1.bin" \
    "../z00z-app/crates/z00z_app_api/outputs/action-basis-manifest-v1.sha256" \
    "../z00z-app/crates/z00z_app_ext/src/lib.rs" \
    ".github/skills/smart-tests-bootstrap/scripts/bootstrap_cache_identity.sh" \
    ".github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh" \
    ".planning/phases/069-Recursive-Proof/069-COVERAGE-AUDIT.py"
do
    grep -Fqx "$required_path" "$TMP_DIR/paths.txt"
done
assert_absent 'crates/z00z_ui_ux/' "$TMP_DIR/paths.txt"
assert_absent 'crates/z00z_storage/outputs/' "$TMP_DIR/paths.txt"
assert_absent '../z00z-app/crates/z00z_app_rpc/' "$TMP_DIR/paths.txt"
while IFS=$'\t' read -r path digest; do
    [[ "$(sha256sum -- "$path" | awk '{print $1}')" == "$digest" ]]
done <"$TMP_DIR/manifest.tsv"
"$AUTHORITY" rehash "$TMP_DIR/manifest.tsv" >"$TMP_DIR/rehash.tsv"
cmp "$TMP_DIR/manifest.tsv" "$TMP_DIR/rehash.tsv"
printf '%s\n' \
    $'../z00z-app/.git/config\taaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' \
    >"$TMP_DIR/forbidden-external-baseline.tsv"
set +e
"$AUTHORITY" rehash "$TMP_DIR/forbidden-external-baseline.tsv" \
    >"$TMP_DIR/forbidden-external-rehash.tsv" \
    2>"$TMP_DIR/forbidden-external-rehash.log"
forbidden_external_status=$?
set -e
(( forbidden_external_status != 0 ))
grep -Fq 'escaped canonical source roots' \
    "$TMP_DIR/forbidden-external-rehash.log"

"$AUTHORITY" compare \
    "$TMP_DIR/manifest.tsv" "$TMP_DIR/manifest.tsv" >"$TMP_DIR/stable.json"
jq -e '
    .status == "stable"
    and .before_digest == .after_digest
    and (.changed_paths | length) == 0
' "$TMP_DIR/stable.json" >/dev/null

printf '%s\n' \
    $'alpha.rs\taaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' \
    $'beta.rs\tbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb' \
    $'removed.rs\tcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc' \
    >"$TMP_DIR/before.tsv"
printf '%s\n' \
    $'added.rs\tdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd' \
    $'alpha.rs\teeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee' \
    $'beta.rs\tbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb' \
    >"$TMP_DIR/after.tsv"
set +e
"$AUTHORITY" compare \
    "$TMP_DIR/before.tsv" "$TMP_DIR/after.tsv" >"$TMP_DIR/drift.json"
compare_status=$?
set -e
[[ "$compare_status" == 86 ]]
jq -e '
    .status == "source_drift"
    and .before_digest != .after_digest
    and (.changed_paths | length) == 3
    and any(.changed_paths[]; .change == "added" and .path == "added.rs")
    and any(.changed_paths[]; .change == "modified" and .path == "alpha.rs")
    and any(.changed_paths[]; .change == "removed" and .path == "removed.rs")
' "$TMP_DIR/drift.json" >/dev/null

mkdir -p \
    "$CACHE_TMP_DIR/cache-fixture/normal/release/deps" \
    "$CACHE_TMP_DIR/cache-fixture/test/release/deps" \
    "$TMP_DIR/no-toolchain-bin"
printf 'normal-cache\n' >"$CACHE_TMP_DIR/cache-fixture/normal/cache.bin"
printf 'test-cache\n' >"$CACHE_TMP_DIR/cache-fixture/test/cache.bin"
jq -n -c \
    --arg normal_root "$CACHE_TMP_DIR/cache-fixture/normal" \
    '{
        reason: "compiler-artifact",
        package_id: "path+file:///fixture#z00z_storage@0.2.0",
        target: {
            name: "z00z_storage",
            kind: ["lib"],
            crate_types: ["lib"]
        },
        profile: {
            opt_level: "3",
            debuginfo: 0,
            debug_assertions: false,
            overflow_checks: false,
            test: false
        },
        features: [],
        filenames: [
            ($normal_root
              + "/release/deps/libz00z_storage-aaaaaaaaaaaaaaaa.rlib"),
            ($normal_root
              + "/release/deps/libz00z_storage-aaaaaaaaaaaaaaaa.rmeta")
        ],
        executable: null
    }' >"$TMP_DIR/normal-messages.jsonl"
jq -n -c \
    --arg test_root "$CACHE_TMP_DIR/cache-fixture/test" \
    '{
        reason: "compiler-artifact",
        package_id: "path+file:///fixture#z00z_storage@0.2.0",
        target: {
            name: "z00z_storage",
            kind: ["lib"],
            crate_types: ["lib"]
        },
        profile: {
            opt_level: "3",
            debuginfo: 0,
            debug_assertions: false,
            overflow_checks: false,
            test: false
        },
        features: [],
        filenames: [
            ($test_root
              + "/release/deps/libz00z_storage-bbbbbbbbbbbbbbbb.rlib")
        ],
        executable: null
    },
    {
        reason: "compiler-artifact",
        package_id: "path+file:///fixture#z00z_storage@0.2.0",
        target: {
            name: "z00z_storage",
            kind: ["lib"],
            crate_types: ["lib"]
        },
        profile: {
            opt_level: "3",
            debuginfo: 0,
            debug_assertions: false,
            overflow_checks: false,
            test: true
        },
        features: [],
        filenames: [
            ($test_root + "/release/deps/z00z_storage-cccccccccccccccc")
        ],
        executable: (
            $test_root + "/release/deps/z00z_storage-cccccccccccccccc"
        )
    }' >"$TMP_DIR/test-messages.jsonl"
jq -n -S \
    --arg repo_root "$ROOT_DIR" \
    --arg checkpoint_output_root "$CHECKPOINT_OUTPUT_ROOT" \
    --arg cache_authority_root "$CACHE_AUTHORITY_ROOT" \
    --arg normal_target "$CACHE_TMP_DIR/cache-fixture/normal" \
    --arg test_target "$CACHE_TMP_DIR/cache-fixture/test" \
    --arg source_authority_digest \
      "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee" \
    '{
        schema: "z00z.phase069.bootstrap-cache-context.v3",
        execution_scope: "focused-test",
        repo_root: $repo_root,
        checkpoint_output_root: $checkpoint_output_root,
        cache_authority_root: $cache_authority_root,
        source_authority_digest: $source_authority_digest,
        root_package_name: "z00z_storage",
        target_dirs: {
            normal_library: $normal_target,
            library_test: $test_target
        },
        profile: "release",
        rustflags: {
            authority: ".cargo/config.toml",
            authority_sha256: (
              "dddddddddddddddddddddddddddddddd"
              + "dddddddddddddddddddddddddddddddd"
            ),
            required_cfg: "test_fast"
        },
        toolchain: {
            cargo: "cargo fixture",
            rustc: "rustc fixture"
        },
        compile_environment: {
            bootstrap_threads: "1",
            cargo_build_jobs: "1",
            release_codegen_units: "64"
        },
        resolved_local_packages: [{
            name: "z00z_storage",
            manifest: "crates/z00z_storage/Cargo.toml"
        }],
        retention: {
            schema: "z00z.phase069.bootstrap-cache-retention.v3",
            strategy: "fixed-cargo-targets",
            max_target_roots: 2,
            automatic_deletion: false
        },
        compile_contract: {
            normal_library: "fixture normal",
            library_test: "fixture test"
        }
    }' >"$TMP_DIR/cache-context.json"
for forbidden_tool in cargo rustc; do
    printf '%s\n' \
        '#!/usr/bin/env bash' \
        "printf \"%s\\n\" \"\${0##*/}\" >>\"\$Z00Z_FORBIDDEN_TOOL_MARKER\"" \
        'exit 99' >"$TMP_DIR/no-toolchain-bin/$forbidden_tool"
    chmod +x "$TMP_DIR/no-toolchain-bin/$forbidden_tool"
done
PATH="$TMP_DIR/no-toolchain-bin:$PATH" \
Z00Z_FORBIDDEN_TOOL_MARKER="$TMP_DIR/forbidden-tool-called" \
"$CACHE_HELPER" capture \
    "$TMP_DIR/normal-messages.jsonl" \
    "$TMP_DIR/test-messages.jsonl" \
    "$TMP_DIR/cache-context.json" \
    "$TMP_DIR/cache-identity.json"
[[ ! -e "$TMP_DIR/forbidden-tool-called" ]]
cache_digest="$(jq -r '.digest' "$TMP_DIR/cache-identity.json")"
PATH="$TMP_DIR/no-toolchain-bin:$PATH" \
Z00Z_FORBIDDEN_TOOL_MARKER="$TMP_DIR/forbidden-tool-called" \
"$CACHE_HELPER" capture \
    "$TMP_DIR/normal-messages.jsonl" \
    "$TMP_DIR/test-messages.jsonl" \
    "$TMP_DIR/cache-context.json" \
    "$TMP_DIR/cache-identity-expected.json" \
    "$cache_digest"
[[ ! -e "$TMP_DIR/forbidden-tool-called" ]]
jq -e \
    --arg digest "$cache_digest" \
    --arg cache_authority_root "$CACHE_AUTHORITY_ROOT" \
    '
        .schema == "z00z.phase069.bootstrap-cache-identity.v3"
        and .digest == $digest
        and .expected_digest == $digest
        and .matches_expected == true
        and .profile == "release"
        and .cache_authority_root == $cache_authority_root
        and .rustflags.required_cfg == "test_fast"
        and .unit_hashes.normal_library == "aaaaaaaaaaaaaaaa"
        and .unit_hashes.library_test_dependency_library
            == "bbbbbbbbbbbbbbbb"
        and .unit_hashes.library_test_executable == "cccccccccccccccc"
        and .target_dirs.normal_library.observed_size_bytes > 0
        and .target_dirs.library_test.observed_size_bytes > 0
        and .source_authority_digest
            == "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
        and .retention.max_target_roots == 2
        and .retention.automatic_deletion == false
    ' \
    "$TMP_DIR/cache-identity-expected.json" >/dev/null

printf '%s\n' \
    '#!/usr/bin/env bash' \
    'if [[ "${1:-}" == "--version" ]]; then' \
    '  printf "cargo 1.0.0 (focused-drift-test)\n"' \
    '  exit 0' \
    'fi' \
    'printf "compile\n" >>"$Z00Z_FAKE_CARGO_MARKER"' \
    'exit 99' >"$TMP_DIR/no-toolchain-bin/cargo"
printf '%s\n' \
    '#!/usr/bin/env bash' \
    'if [[ "${1:-}" == "--version" ]]; then' \
    '  printf "rustc 1.0.0 (focused-drift-test)\n"' \
    '  exit 0' \
    'fi' \
    'exit 99' >"$TMP_DIR/no-toolchain-bin/rustc"
chmod +x \
    "$TMP_DIR/no-toolchain-bin/cargo" \
    "$TMP_DIR/no-toolchain-bin/rustc"
chmod +x "$DRIFT_FIXTURE"
set +e
PATH="$TMP_DIR/no-toolchain-bin:$PATH" \
Z00Z_FAKE_CARGO_MARKER="$TMP_DIR/drift-fake-cargo" \
Z00Z_BOOTSTRAP_SOURCE_AUTHORITY="$DRIFT_FIXTURE" \
Z00Z_BOOTSTRAP_AUTHORITY_FIXTURE_STATE="$TMP_DIR/fixture-state" \
Z00Z_BOOTSTRAP_CACHE_ROOT="$CACHE_TMP_DIR/bootstrap-drift-cache" \
Z00Z_BOOTSTRAP_TEST_MODE=true \
BOOTSTRAP_THREADS=1 \
CARGO_BUILD_JOBS=1 \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=64 \
"$BOOTSTRAP" >"$TMP_DIR/bootstrap-drift.log" 2>&1
bootstrap_status=$?
set -e
[[ "$bootstrap_status" == 86 ]]
result_path="$(
    sed -n 's/^Z00Z_BOOTSTRAP_EVIDENCE_V1 //p' \
        "$TMP_DIR/bootstrap-drift.log" |
        tail -n 1
)"
case "$result_path" in
    "$ROOT_DIR/crates/z00z_storage/outputs/checkpoint/069-08/task-1/test-pyramid/bootstrap/"*/result.json) ;;
    *)
        printf 'unexpected bootstrap drift evidence path: %s\n' "$result_path" >&2
        exit 1
        ;;
esac
jq -e '
    .status == "fail"
    and .reason == "source_drift"
    and .current_stage == "storage_compile"
    and .identity.source_digest_before != .identity.source_digest_after
    and .source_stability.status == "source_drift"
    and .source_stability.before_digest == .identity.source_digest_before
    and .source_stability.after_digest == .identity.source_digest_after
    and .source_stability.drift_stage == "storage_compile:after"
    and (.source_stability.checks | length) == 2
    and .acceptance_authority == true
    and .budgets.total_wall_seconds == 60
    and .budgets.post_compile_execution_seconds == 30
    and .budgets.warm_target_seconds == 12
    and .cache_semantics.bootstrap_requires_prior_prewarm == false
    and .cache_semantics.prewarm_policy
        == "on_demand_after_typed_cold_compile_timeout"
    and .cache_semantics.cold_compile_is_bounded_inside_gate == true
    and any(
        .source_stability.changed_paths[];
        .change == "modified" and .path == "fixture/source.rs"
    )
' "$result_path" >/dev/null
assert_absent 'Compiling ' "$TMP_DIR/bootstrap-drift.log"
[[ -s "$TMP_DIR/drift-fake-cargo" ]]

set +e
PATH="$TMP_DIR/no-toolchain-bin:$PATH" \
Z00Z_FAKE_CARGO_MARKER="$TMP_DIR/prewarm-drift-fake-cargo" \
Z00Z_BOOTSTRAP_SOURCE_AUTHORITY="$DRIFT_FIXTURE" \
Z00Z_BOOTSTRAP_AUTHORITY_FIXTURE_STATE="$TMP_DIR/prewarm-fixture-state" \
Z00Z_BOOTSTRAP_CACHE_ROOT="$CACHE_TMP_DIR/prewarm-drift-cache" \
Z00Z_BOOTSTRAP_TEST_MODE=true \
BOOTSTRAP_THREADS=1 \
CARGO_BUILD_JOBS=1 \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=64 \
"$BOOTSTRAP" prewarm >"$TMP_DIR/prewarm-drift.log" 2>&1
prewarm_status=$?
set -e
[[ "$prewarm_status" == 86 ]]
prewarm_result_path="$(
    sed -n 's/^Z00Z_BOOTSTRAP_EVIDENCE_V1 //p' \
        "$TMP_DIR/prewarm-drift.log" |
        tail -n 1
)"
case "$prewarm_result_path" in
    "$ROOT_DIR/crates/z00z_storage/outputs/checkpoint/069-08/task-1/diagnostics/bootstrap-prewarm/"*/result.json) ;;
    *)
        printf 'unexpected bootstrap prewarm evidence path: %s\n' \
            "$prewarm_result_path" >&2
        exit 1
        ;;
esac
jq -e '
    .status == "fail"
    and .reason == "source_drift"
    and .tier == "bootstrap-prewarm"
    and .mode == "prewarm"
    and .acceptance_authority == false
    and .budgets.total_wall_seconds == 1200
    and .budgets.post_compile_execution_seconds == 0
    and (.selected_gates | length) == 2
' "$prewarm_result_path" >/dev/null
assert_absent 'Compiling ' "$TMP_DIR/prewarm-drift.log"
[[ -s "$TMP_DIR/prewarm-drift-fake-cargo" ]]

printf '%s\n' \
    '#!/usr/bin/env bash' \
    "if [[ \"\${1:-}\" == \"--version\" ]]; then" \
    '  printf "cargo 1.0.0 (focused-test)\n"' \
    '  exit 0' \
    'fi' \
    "printf \"compile\\n\" >\"\$Z00Z_FAKE_CARGO_MARKER\"" \
    'exec sleep 30' >"$TMP_DIR/no-toolchain-bin/cargo"
printf '%s\n' \
    '#!/usr/bin/env bash' \
    "if [[ \"\${1:-}\" == \"--version\" ]]; then" \
    '  printf "rustc 1.0.0 (focused-test)\n"' \
    '  exit 0' \
    'fi' \
    'exit 99' >"$TMP_DIR/no-toolchain-bin/rustc"
chmod +x \
    "$TMP_DIR/no-toolchain-bin/cargo" \
    "$TMP_DIR/no-toolchain-bin/rustc"
set +e
PATH="$TMP_DIR/no-toolchain-bin:$PATH" \
Z00Z_FAKE_CARGO_MARKER="$TMP_DIR/fake-cargo-compile" \
Z00Z_BOOTSTRAP_SOURCE_AUTHORITY="$DRIFT_FIXTURE" \
Z00Z_BOOTSTRAP_AUTHORITY_FIXTURE_STATE="$TMP_DIR/timeout-fixture-state" \
Z00Z_BOOTSTRAP_AUTHORITY_FIXTURE_STABLE=true \
Z00Z_BOOTSTRAP_CACHE_ROOT="$CACHE_TMP_DIR/timeout-cache" \
Z00Z_BOOTSTRAP_TEST_MODE=true \
BOOTSTRAP_PREWARM_WALL_BUDGET_SECONDS=12 \
BOOTSTRAP_THREADS=1 \
CARGO_BUILD_JOBS=1 \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=64 \
"$BOOTSTRAP" prewarm >"$TMP_DIR/prewarm-timeout.log" 2>&1
timeout_status=$?
set -e
[[ "$timeout_status" == 124 ]]
[[ -s "$TMP_DIR/fake-cargo-compile" ]]
timeout_result_path="$(
    sed -n 's/^Z00Z_BOOTSTRAP_EVIDENCE_V1 //p' \
        "$TMP_DIR/prewarm-timeout.log" |
        tail -n 1
)"
jq -e '
    .status == "fail"
    and .reason == "total wall budget exceeded"
    and .current_stage == "storage_compile"
    and .tier == "bootstrap-prewarm"
    and .acceptance_authority == false
    and (.stages | length) == 1
' "$timeout_result_path" >/dev/null
grep -Fq 'stopped storage_compile at its bounded deadline' \
    "$TMP_DIR/prewarm-timeout.log"

assert_cache_root_rejected() {
    local candidate="$1" label="$2" expected="$3" log_path status
    log_path="$TMP_DIR/cache-root-reject-$label.log"
    set +e
    Z00Z_BOOTSTRAP_SOURCE_AUTHORITY="$DRIFT_FIXTURE" \
    Z00Z_BOOTSTRAP_AUTHORITY_FIXTURE_STATE="$TMP_DIR/reject-$label-state" \
    Z00Z_BOOTSTRAP_AUTHORITY_FIXTURE_STABLE=true \
    Z00Z_BOOTSTRAP_CACHE_ROOT="$candidate" \
    Z00Z_BOOTSTRAP_TEST_MODE=true \
    "$BOOTSTRAP" prewarm >"$log_path" 2>&1
    status=$?
    set -e
    (( status != 0 ))
    grep -Fq -- "$expected" "$log_path"
}

target_candidate="$ROOT_DIR/target/z00z-phase069-forbidden-cache"
third_party_candidate="$ROOT_DIR/third_party/z00z-phase069-forbidden-cache"
[[ ! -e "$target_candidate" && ! -e "$third_party_candidate" ]]
assert_cache_root_rejected \
    "$target_candidate" target \
    'bootstrap cache root escaped repository .cache authority'
assert_cache_root_rejected \
    "$third_party_candidate" third-party \
    'bootstrap cache root escaped repository .cache authority'
[[ ! -e "$target_candidate" && ! -e "$third_party_candidate" ]]
ln -s "$ROOT_DIR/target" "$CACHE_TMP_DIR/symlink-cache-root"
assert_cache_root_rejected \
    "$CACHE_TMP_DIR/symlink-cache-root" symlink \
    'bootstrap cache root escaped repository .cache authority'

for contract in \
    'exit_reason=source_drift' \
    'evidence_kind: (' \
    "source_stability: \$source_stability" \
    "source_digest_before: \$source_digest_before" \
    "source_digest_after: \$source_digest_after" \
    'readonly CHECKPOINT_OUTPUT_ROOT=' \
    "\"\$CHECKPOINT_OUTPUT_ROOT/069-08/task-1/\$result_scope/\"*/result.json)" \
    '--bootstrap-prewarm)' \
    'prewarm_required' \
    'BOOTSTRAP_THREADS=8' \
    "readonly CHUNK_CACHE_ROOT=\"\$ROOT_DIR/.cache/phase-069/plan-08/proof-restart-v2\"" \
    '[[ "$#" == 11 ]] || die "invalid internal bootstrap arguments"'
do
    grep -Fq -- "$contract" "$WORKER"
done
assert_absent "\"\$OUTPUT_ROOT/069-08/task-1/\$result_scope/\"" "$WORKER"

for contract in \
    'timeout --signal=TERM --kill-after=5s' \
    "\"\$@\" > >(tee \"\$RUN_DIR/\$name.log\") 2>&1" \
    "acceptance_authority: (\$mode == \"bootstrap\")" \
    'compile_only_prewarm_is_acceptance: false' \
    'bootstrap_requires_prior_prewarm: false' \
    'prewarm_policy: "on_demand_after_typed_cold_compile_timeout"' \
    'incompatible_unit_fingerprints_use_separate_target_roots: true' \
    'stable_target_roots_use_cargo_fingerprints: true' \
    'cache_identity_binds_source_authority: true' \
    'automatic_cache_deletion: false' \
    'cold_compile_is_bounded_inside_gate: true' \
    'CACHE_IDENTITY_HELPER' \
    "DEFAULT_CACHE_ROOT=\"\$CACHE_AUTHORITY_ROOT/phase-069/plan-08/cargo-release\"" \
    'bootstrap cache root escaped repository .cache authority' \
    'bootstrap cache root contains a symlinked ancestor' \
    'max_target_roots: 2' \
    "CARGO_TARGET_DIR=\"\$NORMAL_TARGET_DIR\"" \
    "CARGO_TARGET_DIR=\"\$LIB_TEST_TARGET_DIR\""
do
    grep -Fq -- "$contract" "$BOOTSTRAP"
done

for contract in \
    'z00z.phase069.bootstrap-cache-identity.v3' \
    'checkpoint_output_root' \
    'cache_authority_root' \
    'cache target escaped repository .cache authority' \
    'unit_hashes: {' \
    'observed_size_bytes' \
    'source_authority_digest' \
    'max_target_roots' \
    'digest_scope: ['
do
    grep -Fq -- "$contract" "$CACHE_HELPER"
done

for contract in \
    "readonly CACHE_ROOT=\"\$ROOT_DIR/.cache/phase-069/plan-08/cargo-release\"" \
    'readonly SEMANTIC_TARGET_SECONDS=' \
    'readonly SEMANTIC_BUDGET_SECONDS=' \
    "timeout --signal=TERM --kill-after=5s \"\$SEMANTIC_BUDGET_SECONDS\"" \
    'source_digest_before' \
    'source_digest_after' \
    'ensure_preflight()' \
    ".host_boot_id == \$host_boot_id" \
    'reused current-boot isolation preflight' \
    'typed-table)' \
    'test_direct_typed_commitment_actual_roundtrip'
do
    grep -Fq -- "$contract" "$MILESTONE"
done
assert_absent 'bootstrap-cache-v2' "$MILESTONE"
assert_absent '/target/' "$MILESTONE"
assert_absent '/third_party/' "$MILESTONE"
"$MILESTONE" guards

printf 'bootstrap source authority guards: PASS\n'
