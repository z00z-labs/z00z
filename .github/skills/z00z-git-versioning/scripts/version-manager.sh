#!/bin/bash

# Handles automated version updates, git commits, and GitHub synchronization

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if git_root="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null)"; then
    PROJECT_ROOT="$git_root"
else
    PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
fi
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"
VERSION_FILE="$PROJECT_ROOT/.version"
VERSIONS_YAML="$PROJECT_ROOT/versions.yaml"
DEFAULT_MAX_GIT_FILE_SIZE_MB=50
MAX_GIT_FILE_SIZE_MB=$DEFAULT_MAX_GIT_FILE_SIZE_MB
LARGE_FILE_OVERRIDE=false

# Colors for output
RED=$'\033[0;31m'
GREEN=$'\033[0;32m'
YELLOW=$'\033[1;33m'
BLUE=$'\033[0;34m'
NC=$'\033[0;0m' # No Color

# Check if colors are supported
if [[ -n "$TERM" && "$TERM" != "dumb" ]]; then
    # Colors are supported
    :
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# Logging functions
log_info() {
    echo "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo "${RED}[ERROR]${NC} $1"
}

format_size_mib() {
    local bytes="$1"
    awk -v value="$bytes" 'BEGIN { printf "%.2f MiB", value / 1048576 }'
}

validate_positive_integer() {
    local value="$1"
    local label="$2"

    if [[ ! "$value" =~ ^[0-9]+$ ]] || [[ "$value" -le 0 ]]; then
        log_error "$label must be a positive integer, got: $value"
        return 1
    fi

    return 0
}

# Get current version from versions.yaml
get_current_version() {
    awk '
        $0 == "total_version:" { in_total = 1; next }
        in_total && $0 == "crates:" { in_total = 0 }
        in_total && /^  version:/ {
            sub(/^  version: /, "", $0)
            gsub(/ /, "", $0)
            print
            exit
        }
    ' "$VERSIONS_YAML"
}

# Get last git tag recorded in versions.yaml
get_last_git_tag() {
    awk '
        $0 == "total_version:" { in_total = 1; next }
        in_total && $0 == "crates:" { in_total = 0 }
        in_total && /^  last_git_tag:/ {
            sub(/^  last_git_tag: /, "", $0)
            gsub(/ /, "", $0)
            print
            exit
        }
    ' "$VERSIONS_YAML"
}

expected_tag_for_version() {
    local version="$1"
    echo "v${version}"
}

print_large_file_limit_error() {
    local limit_mb="$1"
    local offenders_file="$2"

    if [[ "$LARGE_FILE_OVERRIDE" == "true" ]]; then
        log_error "Some files exceed the explicit per-file allowance of ${limit_mb} MiB for this run."
    else
        log_error "Git versioning blocks files larger than ${DEFAULT_MAX_GIT_FILE_SIZE_MB} MiB by default."
        log_error "To allow a large file intentionally, re-run with --allow-large-files-up-to-mb <MB>."
        log_error "That flag must explicitly state the maximum allowed size for a single file in this run."
    fi

    while IFS=$'\t' read -r size path; do
        log_error "Oversized git file: ${path} ($(format_size_mib "$size"))"
    done < "$offenders_file"
}

enforce_staged_file_size_limit() {
    local limit_mb="${1:-$MAX_GIT_FILE_SIZE_MB}"
    local limit_bytes=$((limit_mb * 1024 * 1024))
    local offenders_file
    local found="false"

    offenders_file="$(mktemp)"

    while IFS= read -r -d '' path; do
        [[ -n "$path" ]] || continue

        local size
        size="$(git cat-file -s ":$path" 2>/dev/null || true)"
        [[ -n "$size" ]] || continue

        if (( size > limit_bytes )); then
            printf '%s\t%s\n' "$size" "$path" >> "$offenders_file"
            found="true"
        fi
    done < <(git diff --cached --name-only --diff-filter=ACMR -z)

    if [[ "$found" == "true" ]]; then
        sort -u "$offenders_file" -o "$offenders_file"
        print_large_file_limit_error "$limit_mb" "$offenders_file"
        rm -f "$offenders_file"
        return 1
    fi

    rm -f "$offenders_file"
    return 0
}

enforce_push_file_size_limit() {
    local branch="$1"
    local limit_mb="${2:-$MAX_GIT_FILE_SIZE_MB}"
    local limit_bytes=$((limit_mb * 1024 * 1024))
    local offenders_file
    local found="false"
    local object_range="HEAD"

    if git show-ref --verify --quiet "refs/remotes/origin/${branch}"; then
        object_range="origin/${branch}..HEAD"
    fi

    offenders_file="$(mktemp)"

    while IFS= read -r line; do
        [[ -n "$line" ]] || continue
        [[ "$line" == *" "* ]] || continue

        local object_id="${line%% *}"
        local path="${line#* }"
        local object_type
        local size

        object_type="$(git cat-file -t "$object_id" 2>/dev/null || true)"
        [[ "$object_type" == "blob" ]] || continue

        size="$(git cat-file -s "$object_id" 2>/dev/null || true)"
        [[ -n "$size" ]] || continue

        if (( size > limit_bytes )); then
            printf '%s\t%s\n' "$size" "$path" >> "$offenders_file"
            found="true"
        fi
    done < <(git rev-list --objects "$object_range")

    if [[ "$found" == "true" ]]; then
        sort -u "$offenders_file" -o "$offenders_file"
        print_large_file_limit_error "$limit_mb" "$offenders_file"
        rm -f "$offenders_file"
        return 1
    fi

    rm -f "$offenders_file"
    return 0
}

# Parse version components
parse_version() {
    local version="$1"
    echo "${version}" | sed -E 's/^([0-9]+)\.([0-9]+)\.([0-9]+)$/\1 \2 \3/'
}

# Increment version based on change type
increment_version() {
    local current_version="$1"
    local change_type="${2:-patch}" # patch, minor, major
    
    local major minor patch
    read -r major minor patch <<< "$(parse_version "$current_version")"
    
    case "$change_type" in
        "major")
            ((major++))
            minor=0
            patch=0
            ;;
        "minor")
            ((minor++))
            patch=0
            ;;
        "patch")
            ((patch++))
            ;;
        *)
            log_error "Invalid change type: $change_type. Use: major, minor, patch"
            exit 1
            ;;
    esac
    
    echo "${major}.${minor}.${patch}"
}

# Update version in versions.yaml
update_versions_yaml() {
    local new_version="$1"
    local new_tag
    new_tag="$(expected_tag_for_version "$new_version")"
    log_info "Updating versions.yaml release metadata to $new_version / $new_tag"
    
    # Create backup
    cp "$VERSIONS_YAML" "$VERSIONS_YAML.bak"
    
    # Update release metadata together so versions.yaml cannot drift.
    sed -i.tmp \
        -e "0,/^  version: .*/s//  version: $new_version/" \
        -e "0,/^  last_git_tag: .*/s//  last_git_tag: $new_tag/" \
        "$VERSIONS_YAML"
    rm -f "$VERSIONS_YAML.tmp"

    validate_versions_yaml_sync "$new_version" "$new_tag"

    # Ensure the updated file is staged so the release commit matches the tag.
    # This script intentionally commits only already-staged changes unless --stage-all is used.
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        git add "$VERSIONS_YAML" || true
    fi
    
    log_success "versions.yaml updated to version $new_version with matching last_git_tag $new_tag"
}

# Create git commit with version tag
create_git_commit() {
    local old_version="$1"
    local new_version="$2"
    local change_type="${3:-patch}"
    local description="${4:-Automated version update}"
    local create_release_tag="${5:-true}"
    
    log_info "Creating git commit for version $new_version"

    # Detect empty tracked files (do NOT delete them automatically).
    while IFS= read -r -d '' file; do
        if [[ -f "$file" && ! -s "$file" ]]; then
            log_warning "Tracked empty file detected (not removed): $file"
        fi
    done < <(git ls-files -z)
    
    # Create commit with conventional commit format
    if [[ "$change_type" == crate-* ]]; then
        local crate_name="${change_type#crate-}"
        commit_message="feat(crate-${crate_name}): ${description}

Crate update: ${crate_name} → ${new_version}

[skip ci]"
    else
        commit_message="feat(v${new_version}): ${description}

Version bump: ${old_version} → ${new_version}
Change type: ${change_type}

[skip ci]"
    fi
    
    git commit -m "$commit_message"
    
    if [[ "$create_release_tag" == "true" ]]; then
        # Create annotated tag
        if [[ "$change_type" == crate-* ]]; then
            local crate_name="${change_type#crate-}"
            tag_message="Crate Update: ${crate_name} v${new_version}

${description}

Crate update: ${crate_name} → ${new_version}"
        else
            tag_message="Release v${new_version}

${description}

Version bump: ${old_version} → ${new_version}
Change type: ${change_type}"
        fi
        
        git tag -a "v${new_version}" -m "$tag_message"
        log_success "Git commit and tag v${new_version} created"
    else
        log_success "Git commit created without repository release tag"
    fi
}

# Push changes to GitHub
sync_to_github() {
    local new_version="$1"
    local branch="${2:-main}"
    local force_push="${3:-false}"
    
    log_info "Syncing changes to GitHub (branch: $branch)"
    
    # Push commits with force if needed
    if [[ "$force_push" == "true" ]]; then
        log_warning "Using force push to GitHub"
        git push --force origin "$branch"
    else
        git push origin "$branch"
    fi
    
    # Push tags if new_version provided
    if [[ -n "$new_version" ]]; then
        if [[ "$force_push" == "true" ]]; then
            git push --force origin "v${new_version}"
        else
            git push origin "v${new_version}"
        fi
    fi
    
    log_success "Changes synced to GitHub"
}

dispatch_wallet_demo_pages() {
    local branch="$1"
    local workflow_file="$PROJECT_ROOT/.github/workflows/wallet-demo-pages.yml"

    if [[ "$branch" != "main" || ! -f "$workflow_file" ]]; then
        return 0
    fi

    if ! command -v gh >/dev/null 2>&1; then
        log_warning "GitHub CLI is unavailable; wallet demo deployment was not dispatched."
        log_warning "Run publish-wallet-demo manually after installing and authenticating gh."
        return 0
    fi

    if ! gh auth status >/dev/null 2>&1; then
        log_warning "GitHub CLI is not authenticated; wallet demo deployment was not dispatched."
        log_warning "Run 'gh auth login', then dispatch publish-wallet-demo manually."
        return 0
    fi

    log_info "Dispatching publish-wallet-demo for branch $branch"
    if gh workflow run wallet-demo-pages.yml --ref "$branch"; then
        log_success "Wallet demo deployment dispatched"
    else
        log_warning "Wallet demo deployment dispatch failed; run publish-wallet-demo manually."
    fi
}

# Check if there are uncommitted changes
check_git_status() {
    if [[ -n $(git status --porcelain) ]]; then
        log_info "Uncommitted changes detected"
        return 0
    else
        log_info "No uncommitted changes"
        return 1
    fi
}

# Update specific crate version in its Cargo.toml
update_crate_version() {
    local crate_name="$1"
    local new_version="$2"
    
    # Get crate path from versions.yaml
    local crate_path
    crate_path=$(grep -A 3 "^  $crate_name:" "$VERSIONS_YAML" | grep "path:" | sed 's/.*path: //' | tr -d ' ')
    
    if [[ -z "$crate_path" ]]; then
        log_error "Crate '$crate_name' not found in versions.yaml"
        exit 1
    fi
    
    local crate_cargo_toml="$PROJECT_ROOT/$crate_path/Cargo.toml"
    
    if [[ ! -f "$crate_cargo_toml" ]]; then
        log_error "Cargo.toml not found for crate '$crate_name' at $crate_cargo_toml"
        exit 1
    fi
    
    log_info "Updating $crate_name Cargo.toml version to $new_version"
    
    # Backup
    cp "$crate_cargo_toml" "$crate_cargo_toml.bak"
    
    # Update version
    sed -i.tmp "s/^version = .*/version = \"$new_version\"/" "$crate_cargo_toml"
    rm -f "$crate_cargo_toml.tmp"
    
    # Update versions.yaml with new version
    sed -i.bak "/^  $crate_name:/ { n; n; n; s/version: .*/version: $new_version/; }" "$VERSIONS_YAML"
    rm -f "$VERSIONS_YAML.bak"
    
    log_success "$crate_name updated to version $new_version"
}

# Validate version format
validate_version() {
    local version="$1"
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        log_error "Invalid version format: $version. Expected format: X.Y.Z"
        return 1
    fi
    return 0
}

validate_versions_yaml_sync() {
    local version="${1:-$(get_current_version)}"
    local last_git_tag="${2:-$(get_last_git_tag)}"
    local expected_tag

    if [[ -z "$version" ]]; then
        log_error "versions.yaml is missing total_version.version"
        return 1
    fi

    if [[ -z "$last_git_tag" ]]; then
        log_error "versions.yaml is missing total_version.last_git_tag"
        return 1
    fi

    validate_version "$version"
    expected_tag="$(expected_tag_for_version "$version")"

    if [[ "$last_git_tag" != "$expected_tag" ]]; then
        log_error "versions.yaml invariant violated: total_version.version=$version but total_version.last_git_tag=$last_git_tag (expected $expected_tag)"
        return 1
    fi

    return 0
}

# Show help
show_help() {
    cat << EOF
Version Management Script

USAGE:
    $0 [OPTIONS] <COMMAND>

COMMANDS:
    patch           Increment patch version (X.Y.Z → X.Y.Z+1)
    minor           Increment minor version (X.Y.Z → X.Y+1.0)
    major           Increment major version (X.Y.Z → X+1.0.0)
    crate <name> <ver> Update specific crate version and commit
    current         Show current version
    status          Show git status and current version
    branch          Show current git branch
    sync            Push current changes to GitHub

OPTIONS:
    -m, --message <MSG>    Commit message (default: "Automated version update")
    -b, --branch <BRANCH>  Target branch (default: current branch)
    -f, --force            Force push to GitHub
    --stage-all            Stage all changes before committing (unsafe; includes deletions)
    --allow-large-files-up-to-mb <MB> Explicitly allow a larger single git file for this run up to the stated MiB limit
    --allow-branch-mismatch Allow pushing when target branch differs from current branch
    -d, --dry-run          Show what would be done without executing
    -h, --help             Show this help message

EXAMPLES:
    $0 patch -m "Fix memory leak in PTB engine"
    $0 minor -m "Add new batch verification feature"
    $0 major -m "Breaking changes to API"
    $0 crate bulletproofs 4.4.1 -m "Update bulletproofs"
    $0 current
    $0 status
    $0 branch
    $0 sync
    $0 minor --stage-all --allow-large-files-up-to-mb 250 -m "Intentional large artifact update"

EOF
}

# Main execution
main() {
    local command=""
    local message="Automated version update"
    local branch=""
    local dry_run=false
    local force_push=false
    local stage_all=false
    local allow_branch_mismatch=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -m|--message)
                message="$2"
                shift 2
                ;;
            -b|--branch)
                branch="$2"
                shift 2
                ;;
            -f|--force)
                force_push=true
                shift
                ;;
            --stage-all)
                stage_all=true
                shift
                ;;
            --allow-large-files-up-to-mb)
                MAX_GIT_FILE_SIZE_MB="$2"
                LARGE_FILE_OVERRIDE=true
                shift 2
                ;;
            --allow-branch-mismatch)
                allow_branch_mismatch=true
                shift
                ;;
            -d|--dry-run)
                dry_run=true
                shift
                ;;
            patch|minor|major|current|status|branch|sync)
                command="$1"
                shift
                ;;
            crate)
                command="$1"
                crate_name="$2"
                crate_version="$3"
                shift 3
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Validate command
    if [[ -z "$command" ]]; then
        log_error "No command specified"
        show_help
        exit 1
    fi
    
    # Change to project root
    cd "$PROJECT_ROOT"

    validate_positive_integer "$MAX_GIT_FILE_SIZE_MB" "--allow-large-files-up-to-mb"

    # Default branch to the current checked-out branch (safer than defaulting to main)
    if [[ -z "$branch" ]]; then
        branch="$(git branch --show-current)"
    fi

    local current_branch
    current_branch="$(git branch --show-current)"
    if [[ -z "$current_branch" ]]; then
        log_error "Unable to determine current git branch"
        exit 1
    fi

    # Refuse to push/commit for a different branch unless explicitly overridden.
    if [[ "$branch" != "$current_branch" && "$allow_branch_mismatch" != "true" ]]; then
        log_error "Target branch '$branch' does not match current branch '$current_branch'."
        log_error "Re-run with: -b '$current_branch' (recommended) or use --allow-branch-mismatch"
        exit 1
    fi
    
    # Get current version
    local current_version
    local current_last_git_tag
    current_version=$(get_current_version)
    current_last_git_tag=$(get_last_git_tag)
    validate_versions_yaml_sync "$current_version" "$current_last_git_tag"
    
    log_info "Current version: $current_version"
    
    case "$command" in
        "current")
            echo "$current_version"
            ;;
        "status")
            echo "Current version: $current_version"
            if check_git_status; then
                echo "Git status: Uncommitted changes"
            else
                echo "Git status: Clean"
            fi
            ;;
        "branch")
            echo "$(git branch --show-current)"
            ;;
        "sync")
            # The sync command should push committed changes too, not only a dirty working tree.
            # Determine whether local HEAD is ahead of origin/<branch> or the release tag is missing.
            local needs_sync="false"

            # Ensure remote refs are up-to-date for comparison.
            git fetch origin "$branch" --tags >/dev/null 2>&1 || true

            if git show-ref --verify --quiet "refs/remotes/origin/${branch}"; then
                local ahead_count
                ahead_count=$(git rev-list --count "origin/${branch}..HEAD" 2>/dev/null || echo 0)
                if [[ "$ahead_count" != "0" ]]; then
                    needs_sync="true"
                fi
            else
                # Remote branch does not exist yet.
                needs_sync="true"
            fi

            # Also ensure the tag for the current version exists on origin.
            if ! git ls-remote --tags origin "v${current_version}" | grep -q "v${current_version}"; then
                needs_sync="true"
            fi

            # Preserve old behavior: if there are uncommitted changes, we definitely need a sync.
            if check_git_status; then
                needs_sync="true"
            fi

            if [[ "$needs_sync" != "true" ]]; then
                log_warning "No changes to sync"
                exit 0
            fi

            enforce_push_file_size_limit "$branch" "$MAX_GIT_FILE_SIZE_MB"
            
            if [[ "$dry_run" == "true" ]]; then
                log_info "[DRY RUN] Would sync changes to GitHub"
                exit 0
            fi
            
            sync_to_github "$current_version" "$branch" "$force_push"
            ;;
        "patch"|"minor"|"major")
            # Safer default: require staged changes. Use --stage-all to stage everything.
            if [[ "$stage_all" != "true" ]]; then
                if git diff --cached --quiet; then
                    log_error "No staged changes to commit. Stage files first, or re-run with --stage-all."
                    exit 1
                fi
            else
                if ! check_git_status; then
                    log_warning "No changes to commit"
                    exit 0
                fi

                log_warning "Staging all changes (including untracked and deletions)"
                git add -A
            fi

            enforce_staged_file_size_limit "$MAX_GIT_FILE_SIZE_MB"
            
            local new_version
            new_version=$(increment_version "$current_version" "$command")
            validate_version "$new_version"
            
            log_info "New version: $new_version (change type: $command)"
            
            if [[ "$dry_run" == "true" ]]; then
                log_info "[DRY RUN] Would update version from $current_version to $new_version"
                log_info "[DRY RUN] Would create commit: feat(v${new_version}): $message"
                log_info "[DRY RUN] Would create tag: v${new_version}"
                log_info "[DRY RUN] Would sync to GitHub (branch: $branch)"
                exit 0
            fi
            
            # Update version
            update_versions_yaml "$new_version"
            
            # Create commit and tag
            create_git_commit "$current_version" "$new_version" "$command" "$message"
            
            # Sync to GitHub
            sync_to_github "$new_version" "$branch" "$force_push"
            dispatch_wallet_demo_pages "$branch"
            
            log_success "Version management completed: $current_version → $new_version"
            ;;
        "crate")
            if [[ -z "$crate_name" || -z "$crate_version" ]]; then
                log_error "Crate name and version required: crate <name> <version>"
                exit 1
            fi
            validate_version "$crate_version"
            
            if [[ "$dry_run" == "true" ]]; then
                log_info "[DRY RUN] Would update crate $crate_name to $crate_version"
                log_info "[DRY RUN] Would create commit: feat(crate-${crate_name}): Update $crate_name to $crate_version"
                log_info "[DRY RUN] Would not create a repository release tag or mutate total_version release metadata"
                exit 0
            fi

            if [[ "$stage_all" == "true" ]]; then
                log_warning "Staging all changes (including untracked and deletions)"
                git add -A
            fi

            enforce_staged_file_size_limit "$MAX_GIT_FILE_SIZE_MB"
            
            # Update crate version
            update_crate_version "$crate_name" "$crate_version"
            
            # Create commit only. Crate-specific updates must not create a repository release tag.
            create_git_commit "$current_version" "$crate_version" "crate-$crate_name" "$message" "false"
            
            # Sync branch only; no repository release tag is pushed for crate-only updates.
            sync_to_github "" "$branch" "$force_push"
            
            log_success "Crate $crate_name updated to $crate_version"
            ;;
        *)
            log_error "Unknown command: $command"
            show_help
            exit 1
            ;;
    esac
}

# Execute main function
main "$@"
