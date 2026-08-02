#!/usr/bin/env bash
# Canonical source authority for the Phase-069 bootstrap compile and test graph.

set -euo pipefail

SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
readonly SCRIPT_PATH
ROOT_DIR="$(cd "$(dirname "$SCRIPT_PATH")/../../../.." && pwd)"
readonly ROOT_DIR
PROJECTS_DIR="$(dirname "$ROOT_DIR")"
readonly PROJECTS_DIR
APP_REPO_DIR="$PROJECTS_DIR/z00z-app"
readonly APP_REPO_DIR
readonly ROOT_PACKAGE="z00z_storage"
readonly SOURCE_HASH_PROCESSES=8
readonly SOURCE_HASH_BATCH_SIZE=8
readonly SOURCE_AUTHORITY_TMP_ROOT="$ROOT_DIR/.cache/phase-069/plan-08/source-authority-tmp"

die() {
    printf 'bootstrap source authority: %s\n' "$*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 ||
        die "required command not found: $1"
}

canonical_temp_root() {
    local canonical path
    for path in \
        "$ROOT_DIR/.cache" \
        "$ROOT_DIR/.cache/phase-069" \
        "$ROOT_DIR/.cache/phase-069/plan-08" \
        "$SOURCE_AUTHORITY_TMP_ROOT"
    do
        [[ ! -L "$path" ]] ||
            die "source-authority cache ancestor is symlinked: $path"
        if [[ -e "$path" ]]; then
            [[ -d "$path" ]] ||
                die "source-authority cache ancestor is not a directory: $path"
        else
            mkdir -- "$path" ||
                die "failed to create source-authority cache ancestor: $path"
        fi
        canonical="$(realpath -e -- "$path")" ||
            die "source-authority cache ancestor does not resolve: $path"
        [[ "$canonical" == "$path" ]] ||
            die "source-authority cache ancestor escaped canonical path: $canonical"
    done
    printf '%s\n' "$SOURCE_AUTHORITY_TMP_ROOT"
}

authority_label_for_absolute() {
    local absolute="$1"
    case "$absolute" in
        "$ROOT_DIR"/*)
            printf '%s\n' "${absolute#"$ROOT_DIR"/}"
            ;;
        "$APP_REPO_DIR/Cargo.toml")
            printf '%s\n' "../z00z-app/Cargo.toml"
            ;;
        "$APP_REPO_DIR/crates/z00z_app_api"/* | \
        "$APP_REPO_DIR/crates/z00z_app_ext"/* | \
        "$APP_REPO_DIR/crates/z00z_app_rpc"/*)
            printf '%s\n' "../z00z-app/${absolute#"$APP_REPO_DIR"/}"
            ;;
        *)
            die "resolved local source escaped canonical source roots: $absolute"
            ;;
    esac
}

authority_relative_path() {
    local path="$1" absolute
    absolute="$(realpath -e -- "$path")" ||
        die "source input does not resolve: $path"
    authority_label_for_absolute "$absolute"
}

require_canonical_source_file() {
    local path="$1" canonical expected_label
    [[ "$path" != *$'\t'* && "$path" != *$'\n'* ]] ||
        die "source manifest path contains a forbidden control character"
    [[ -n "$path" ]] || die "source manifest path is empty"
    [[ ! -L "$path" ]] ||
        die "source manifest path is symlinked: $path"
    [[ -f "$path" ]] ||
        die "source manifest path is not a regular file: $path"
    canonical="$(realpath -e -- "$path")" ||
        die "source manifest path does not resolve: $path"
    expected_label="$(authority_label_for_absolute "$canonical")"
    [[ "$expected_label" == "$path" ]] ||
        die "source manifest path is noncanonical or has a symlinked ancestor: $path"
}

expected_absolute_for_label() {
    local path="$1" output_name="$2" relative resolved
    local -n output_ref="$output_name"
    [[ "$path" != *$'\t'* && "$path" != *$'\n'* ]] ||
        die "source manifest path contains a forbidden control character"
    [[ -n "$path" ]] || die "source manifest path is empty"
    case "$path" in
        ../z00z-app/Cargo.toml)
            relative="Cargo.toml"
            case "/$relative/" in
                *"//"* | *"/./"* | *"/../"*)
                    die "source manifest path is not lexically canonical: $path"
                    ;;
            esac
            resolved="$APP_REPO_DIR/Cargo.toml"
            ;;
        ../z00z-app/crates/z00z_app_api/* | \
        ../z00z-app/crates/z00z_app_ext/* | \
        ../z00z-app/crates/z00z_app_rpc/*)
            relative="${path#../z00z-app/}"
            case "/$relative/" in
                *"//"* | *"/./"* | *"/../"*)
                    die "source manifest path is not lexically canonical: $path"
                    ;;
            esac
            resolved="$APP_REPO_DIR/${path#../z00z-app/}"
            ;;
        /* | ../*)
            die "source manifest path escaped canonical source roots: $path"
            ;;
        *)
            case "/$path/" in
                *"//"* | *"/./"* | *"/../"*)
                    die "source manifest path is not lexically canonical: $path"
                    ;;
            esac
            resolved="$ROOT_DIR/$path"
            ;;
    esac
    output_ref="$resolved"
}

validate_canonical_source_files() (
    local canonical_file canonical expected path temp_root
    local -a paths=("$@")
    ((${#paths[@]} > 0)) || return 0
    temp_root="$(canonical_temp_root)"
    canonical_file="$(mktemp --tmpdir="$temp_root" source-realpaths.XXXXXXXX.bin)"
    trap 'rm -f -- "$canonical_file"' EXIT
    for path in "${paths[@]}"; do
        expected_absolute_for_label "$path" expected
        [[ ! -L "$path" ]] ||
            die "source manifest path is symlinked: $path"
        [[ -f "$path" ]] ||
            die "source manifest path is not a regular file: $path"
        [[ -n "$expected" ]] ||
            die "source manifest path has no canonical absolute target: $path"
    done
    realpath -e -z -- "${paths[@]}" >"$canonical_file" ||
        die "source manifest path does not resolve"
    exec 3<"$canonical_file"
    for path in "${paths[@]}"; do
        IFS= read -r -d '' canonical <&3 ||
            die "batched source canonicalization returned too few paths"
        expected_absolute_for_label "$path" expected
        [[ "$canonical" == "$expected" ]] ||
            die "source manifest path is noncanonical or has a symlinked ancestor: $path"
    done
    if IFS= read -r -d '' canonical <&3; then
        die "batched source canonicalization returned too many paths"
    fi
)

validate_local_package_root() {
    local package_name="$1" package_root="$2" expected_root
    case "$package_root" in
        "$ROOT_DIR"/*)
            return 0
            ;;
    esac
    case "$package_name" in
        z00z-app-api)
            expected_root="$APP_REPO_DIR/crates/z00z_app_api"
            ;;
        z00z-app-ext)
            expected_root="$APP_REPO_DIR/crates/z00z_app_ext"
            ;;
        z00z-app-rpc)
            expected_root="$APP_REPO_DIR/crates/z00z_app_rpc"
            ;;
        *)
            die "external local dependency package is not approved: $package_name at $package_root"
            ;;
    esac
    [[ -d "$APP_REPO_DIR" && ! -L "$APP_REPO_DIR" ]] ||
        die "canonical sibling app repository is unavailable or symlinked: $APP_REPO_DIR"
    [[ "$(realpath -e -- "$APP_REPO_DIR")" == "$APP_REPO_DIR" ]] ||
        die "canonical sibling app repository escaped its declared path"
    [[ "$package_root" == "$expected_root" && -d "$expected_root" && ! -L "$expected_root" ]] ||
        die "approved app package has a noncanonical root: $package_name at $package_root"
    [[ "$(realpath -e -- "$expected_root")" == "$expected_root" ]] ||
        die "approved app package root is symlinked: $expected_root"
}

host_target_triple() {
    local host
    host="$(
        rustc -vV |
            awk -F ': ' '$1 == "host" { print $2 }'
    )" || die "failed to resolve the rustc host target"
    [[ "$host" =~ ^[A-Za-z0-9_][A-Za-z0-9_.-]*$ ]] ||
        die "rustc returned an invalid host target: $host"
    printf '%s\n' "$host"
}

resolved_local_packages() {
    local host
    host="$(host_target_triple)"
    cargo metadata \
        --format-version 1 \
        --locked \
        --offline \
        --filter-platform "$host" |
        jq -r --arg root_name "$ROOT_PACKAGE" '
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
            | if ($roots | length) != 1 then
                error("expected exactly one local root package named " + $root_name)
              else
                (
                    (
                        $roots
                        + [$roots[] as $id | $edges[$id].all[]?]
                    )
                    | unique
                    | closure(.)
                )[]
                | $packages[.]
                | select(.source == null)
                | [.name, .manifest_path]
                | @tsv
              end
        ' |
        LC_ALL=C sort -t $'\t' -k2,2
}

emit_required_file() {
    local path="$1"
    [[ -f "$path" ]] || die "required source input missing: $path"
    authority_relative_path "$path"
}

emit_optional_file() {
    local path="$1"
    [[ ! -e "$path" || -f "$path" ]] ||
        die "optional source input is not a regular file: $path"
    if [[ -f "$path" ]]; then
        authority_relative_path "$path"
    fi
}

emit_tree_files() (
    local root="$1" file canonical_root temp_root tree_files
    [[ ! -e "$root" || -d "$root" ]] ||
        die "source input root is not a directory: $root"
    [[ -d "$root" ]] || return 0
    canonical_root="$(realpath -e -- "$root")" ||
        die "source input root does not resolve: $root"
    authority_label_for_absolute "$canonical_root" >/dev/null
    temp_root="$(canonical_temp_root)"
    tree_files="$(mktemp --tmpdir="$temp_root" source-tree.XXXXXXXX.bin)"
    trap 'rm -f -- "$tree_files"' EXIT
    find "$canonical_root" \( -type f -o -type l \) -print0 |
        sort -z >"$tree_files" ||
        die "failed to enumerate source tree: $canonical_root"
    while IFS= read -r -d '' file; do
        [[ ! -L "$file" ]] ||
            die "source tree contains a symlink: $file"
        authority_label_for_absolute "$file"
    done <"$tree_files"
)

source_paths() {
    local package_name manifest_path package_root packages
    local -a bootstrap_scripts=(
        ".github/skills/smart-tests-bootstrap/SKILL.md"
        ".github/skills/smart-tests-bootstrap/scripts/bootstrap_cache_identity.sh"
        ".github/skills/smart-tests-bootstrap/scripts/bootstrap_source_authority.sh"
        ".github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh"
        ".github/skills/smart-tests-bootstrap/scripts/fixtures/bootstrap_source_authority_drift_fixture.sh"
        ".github/skills/smart-tests-bootstrap/scripts/nova_measurement_worker_authority_v2.txt"
        ".github/skills/smart-tests-bootstrap/scripts/nova_milestone_tests.sh"
        ".github/skills/smart-tests-bootstrap/scripts/nova_verifier_rss_measurement.sh"
        ".github/skills/smart-tests-bootstrap/scripts/plonky3_milestone_tests.sh"
        ".github/skills/smart-tests-bootstrap/scripts/plonky3_resource_worker.sh"
        ".github/skills/smart-tests-bootstrap/scripts/test_bootstrap_source_authority.sh"
    )
    local -a planning_inputs=(
        ".planning/phases/069-Recursive-Proof/069-CONTEXT.md"
        ".planning/phases/069-Recursive-Proof/069-COVERAGE-AUDIT.py"
        ".planning/phases/069-Recursive-Proof/069-COVERAGE.md"
        ".planning/phases/069-Recursive-Proof/069-TEST-SPEC.md"
        ".planning/phases/069-Recursive-Proof/069-TESTS-TASKS.md"
        ".planning/phases/069-Recursive-Proof/069-TODO.md"
        ".planning/phases/069-Recursive-Proof/069-01-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-02-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-03-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-04-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-05-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-06-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-07-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-08-ARCHITECTURE-ROOT-CAUSE.md"
        ".planning/phases/069-Recursive-Proof/069-08-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-09-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-10-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-11-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-12-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-13-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-051-PLAN.md"
    )
    local -a plan08_tests=(
        "crates/z00z_storage/tests/test_recursive_epoch.rs"
        "crates/z00z_storage/tests/test_recursive_history.rs"
        "crates/z00z_storage/tests/test_recursive_v2_plonky3_base.rs"
        "crates/z00z_storage/tests/test_recursive_v2_plonky3_epoch.rs"
        "crates/z00z_storage/tests/test_recursive_v2_plonky3_history.rs"
    )

    emit_required_file "Cargo.toml"
    emit_required_file "Cargo.lock"
    emit_required_file ".cargo/config.toml"
    for manifest_path in \
        "${bootstrap_scripts[@]}" \
        "${planning_inputs[@]}" \
        "${plan08_tests[@]}"; do
        emit_required_file "$manifest_path"
    done

    packages="$(resolved_local_packages)" ||
        die "failed to resolve the local bootstrap dependency graph"
    [[ -n "$packages" ]] ||
        die "resolved local bootstrap dependency graph is empty"
    if grep -Fq "$APP_REPO_DIR/" <<<"$packages"; then
        emit_required_file "$APP_REPO_DIR/Cargo.toml"
    fi
    while IFS=$'\t' read -r package_name manifest_path; do
        [[ -n "$package_name" && -n "$manifest_path" ]] ||
            die "cargo metadata returned an incomplete local package record"
        manifest_path="$(realpath -e -- "$manifest_path")"
        package_root="$(dirname "$manifest_path")"
        validate_local_package_root "$package_name" "$package_root"

        emit_required_file "$manifest_path"
        emit_optional_file "$package_root/build.rs"
        emit_optional_file "$package_root/README.md"
        emit_tree_files "$package_root/build_support"
        emit_tree_files "$package_root/src"
        emit_tree_files "$package_root/config"
        emit_tree_files "$package_root/configs"
        emit_tree_files "$package_root/docs"
        emit_tree_files "$package_root/generated"
        emit_tree_files "$package_root/tests/fixtures"

        # `z00z-app-api` owns a checked build-time Action Basis projection in
        # `outputs/`.  It is source authority for its build script, not a test
        # artifact, so hash the complete bounded package-local tree explicitly.
        if [[ "$package_name" == "z00z-app-api" ]]; then
            emit_tree_files "$package_root/outputs"
        fi

        if [[ "$package_name" == "$ROOT_PACKAGE" ]]; then
            emit_required_file \
                "$package_root/tests/test_recursive_v2_nova_step.rs"
            emit_required_file \
                "$package_root/tests/test_recursive_v2_nova_adversarial.rs"
        fi
    done <<<"$packages"
}

emit_hashed_paths() (
    local record digest path hash_records parsed_records temp_root
    local -a paths=("$@")
    ((${#paths[@]} > 0)) || return 0
    validate_canonical_source_files "${paths[@]}"
    temp_root="$(canonical_temp_root)"
    hash_records="$(mktemp --tmpdir="$temp_root" source-hashes.XXXXXXXX.bin)"
    parsed_records="$(mktemp --tmpdir="$temp_root" source-hashes.XXXXXXXX.tsv)"
    trap 'rm -f -- "$hash_records" "$parsed_records"' EXIT
    printf '%s\0' "${paths[@]}" |
        xargs -0 -r -n "$SOURCE_HASH_BATCH_SIZE" \
            -P "$SOURCE_HASH_PROCESSES" sha256sum -z -- >"$hash_records" ||
        die "failed to hash the complete source authority"
    while IFS= read -r -d '' record; do
        digest="${record%% *}"
        path="${record#"$digest  "}"
        [[ "$digest" =~ ^[0-9a-f]{64}$ && -n "$path" ]] ||
            die "sha256sum returned an invalid source record"
        printf '%s\t%s\n' "$path" "$digest"
    done <"$hash_records" >"$parsed_records"
    LC_ALL=C sort -t $'\t' -k1,1 -- "$parsed_records"
)

write_manifest() (
    local path source_list temp_root
    local -a paths=()
    temp_root="$(canonical_temp_root)"
    source_list="$(mktemp --tmpdir="$temp_root" source-paths.XXXXXXXX)"
    trap 'rm -f -- "$source_list"' EXIT
    source_paths >"$source_list" ||
        die "failed to enumerate the complete source authority"
    LC_ALL=C sort -u -o "$source_list" -- "$source_list" ||
        die "failed to sort the complete source authority"
    mapfile -t paths <"$source_list"
    ((${#paths[@]} > 0)) || die "source authority returned no paths"
    for path in "${paths[@]}"; do
        [[ "$path" != *$'\t'* && "$path" != *$'\n'* ]] ||
            die "source path contains a forbidden control character"
    done
    emit_hashed_paths "${paths[@]}"
)

rehash_manifest() {
    local baseline="$1" path expected
    local -a paths=()
    [[ -f "$baseline" ]] || die "baseline manifest missing: $baseline"
    while IFS=$'\t' read -r path _; do
        [[ -n "$path" ]] || die "baseline manifest contains an empty path"
        [[ "$path" != *$'\t'* && "$path" != *$'\n'* ]] ||
            die "baseline manifest contains a forbidden control character"
        expected_absolute_for_label "$path" expected
        [[ -n "$expected" ]] ||
            die "baseline manifest path has no canonical absolute target: $path"
        if [[ -e "$path" || -L "$path" ]]; then
            paths+=("$path")
        fi
    done <"$baseline"
    emit_hashed_paths "${paths[@]}"
}

manifest_digest() {
    local manifest="$1"
    [[ -f "$manifest" ]] || die "manifest missing: $manifest"
    sha256sum -- "$manifest" | awk '{print $1}'
}

compare_manifests() (
    local before="$1" after="$2" before_digest after_digest changed_tsv changed_json_file temp_root
    [[ -f "$before" ]] || die "before manifest missing: $before"
    [[ -f "$after" ]] || die "after manifest missing: $after"
    before_digest="$(manifest_digest "$before")"
    after_digest="$(manifest_digest "$after")"
    temp_root="$(canonical_temp_root)"
    changed_tsv="$(mktemp --tmpdir="$temp_root" source-changes.XXXXXXXX.tsv)"
    changed_json_file="$(mktemp --tmpdir="$temp_root" source-changes.XXXXXXXX.json)"
    trap 'rm -f -- "$changed_tsv" "$changed_json_file"' EXIT

    awk -F '\t' '
        FNR == NR {
            before[$1] = $2
            next
        }
        {
            after[$1] = $2
        }
        END {
            for (path in before) {
                if (!(path in after)) {
                    printf "removed\t%s\t%s\t\n", path, before[path]
                } else if (before[path] != after[path]) {
                    printf "modified\t%s\t%s\t%s\n", path, before[path], after[path]
                }
            }
            for (path in after) {
                if (!(path in before)) {
                    printf "added\t%s\t\t%s\n", path, after[path]
                }
            }
        }
    ' "$before" "$after" |
        LC_ALL=C sort -t $'\t' -k2,2 >"$changed_tsv"

    jq -Rn '
            [
                inputs
                | split("\t")
                | {
                    change: .[0],
                    path: .[1],
                    before_sha256: (
                        if (.[2] // "") == "" then null else .[2] end
                    ),
                    after_sha256: (
                        if (.[3] // "") == "" then null else .[3] end
                    )
                }
            ]
        ' <"$changed_tsv" >"$changed_json_file" ||
        die "failed to encode changed source paths"
    jq -n -S \
        --arg status "$([[ "$before_digest" == "$after_digest" ]] &&
            printf stable || printf source_drift)" \
        --arg before_digest "$before_digest" \
        --arg after_digest "$after_digest" \
        --slurpfile changed_paths "$changed_json_file" \
        '{
            status: $status,
            before_digest: $before_digest,
            after_digest: $after_digest,
            changed_paths: $changed_paths[0]
        }'
    [[ "$before_digest" == "$after_digest" ]] || return 86
)

usage() {
    printf 'usage: %s packages | manifest | digest | rehash <baseline> | compare <before> <after>\n' \
        "${0##*/}"
}

main() {
    local command="${1:-}" command_name
    cd "$ROOT_DIR"
    for command_name in awk cargo find jq mktemp realpath rustc sha256sum sort xargs; do
        require_command "$command_name"
    done
    case "$command" in
        packages)
            [[ "$#" == 1 ]] || die "packages takes no arguments"
            resolved_local_packages |
                while IFS=$'\t' read -r package_name manifest_path; do
                    printf '%s\t%s\n' \
                        "$package_name" \
                        "$(authority_relative_path "$manifest_path")"
                done
            ;;
        manifest)
            [[ "$#" == 1 ]] || die "manifest takes no arguments"
            write_manifest
            ;;
        digest)
            [[ "$#" == 1 ]] || die "digest takes no arguments"
            write_manifest | sha256sum | awk '{print $1}'
            ;;
        rehash)
            [[ "$#" == 2 ]] || die "rehash requires a baseline manifest"
            rehash_manifest "$2"
            ;;
        compare)
            [[ "$#" == 3 ]] || die "compare requires before and after manifests"
            compare_manifests "$2" "$3"
            ;;
        *)
            usage >&2
            return 2
            ;;
    esac
}

main "$@"
