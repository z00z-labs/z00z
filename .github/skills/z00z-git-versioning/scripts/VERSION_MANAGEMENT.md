# Version Management System

> [!IMPORTANT]
> The canonical Z00Z versioning workflow now lives in `.github/skills/z00z-git-versioning/scripts/`.
> Use `.github/skills/z00z-git-versioning/scripts/version-manager.sh` from the repository root.
>
> [!IMPORTANT]
> For the combined Z00Z workflow of minor git commit plus GitHub sync, use the current branch only, force-push to the same branch only, do not open a PR, and do not restore deleted files from git.
>
> [!IMPORTANT]
> `versions.yaml` must stay internally consistent: if `total_version.version` is `X.Y.Z`, then `total_version.last_git_tag` must be exactly `vX.Y.Z`. A mismatch is a release-state error, not a cosmetic issue.
>
> [!IMPORTANT]
> Files larger than `50 MiB` must not enter git through this workflow by default. To permit a larger single file intentionally, the exact run must pass `--allow-large-files-up-to-mb <MB>` and state the maximum allowed size for that file.
>
> [!NOTE]
> Any historical references in this document to `release.sh` are legacy only. This repository's supported workflow is `version-manager.sh`.

This document describes the automated version management system for the Verkle Fractal project.

## Overview

The version management system provides:
- **Automated version updates** in `Cargo.toml`
- **Git commits** with conventional format: `feat(vX.Y.Z): description`
- **GitHub releases** with automatic changelog generation
- **Empty file filtering** to prevent committing empty files
- **Quality checks** and automated testing
- **versions.yaml synchronization** on each repo release commit (`total_version.version` and `total_version.last_git_tag` move together)
- **Specific crate version updates** with dedicated commands

## Components

### 1. Version Manager Script (`.github/skills/z00z-git-versioning/scripts/version-manager.sh`)

The main script that handles all version management operations.

#### Usage:
```bash
./.github/skills/z00z-git-versioning/scripts/version-manager.sh [OPTIONS] <COMMAND>
```

#### Commands:
- `patch` - Increment patch version (X.Y.Z → X.Y.Z+1)
- `minor` - Increment minor version (X.Y.Z → X.Y+1.0)  
- `major` - Increment major version (X.Y.Z → X+1.0.0)
- `crate <name> <version>` - Update specific crate version and commit
- `current` - Show current version
- `status` - Show git status and current version
- `sync` - Push current changes to GitHub

#### Options:
- `-m, --message <MSG>` - Commit message (default: "Automated version update")
- `-b, --branch <BRANCH>` - Target branch (default: main)
- `--allow-large-files-up-to-mb <MB>` - Explicitly allow a larger single git file for this run up to the stated MiB limit
- `-d, --dry-run` - Show what would be done without executing

#### Examples:
```bash
# Create patch release
./.github/skills/z00z-git-versioning/scripts/version-manager.sh patch -m "Fix memory leak in PTB engine"

# Create minor release
./.github/skills/z00z-git-versioning/scripts/version-manager.sh minor -m "Add new batch verification feature"

# Create major release
./.github/skills/z00z-git-versioning/scripts/version-manager.sh major -m "Breaking changes to API"

# Update specific crate version
./.github/skills/z00z-git-versioning/scripts/version-manager.sh crate bulletproofs 4.4.1 -m "Update bulletproofs to 4.4.1"

# Check current status
./.github/skills/z00z-git-versioning/scripts/version-manager.sh status

# Sync changes without version bump
./.github/skills/z00z-git-versioning/scripts/version-manager.sh sync

# Intentionally allow a single file up to 250 MiB for this run
./.github/skills/z00z-git-versioning/scripts/version-manager.sh minor --stage-all --allow-large-files-up-to-mb 250 -m "Intentional large artifact update"

#### Same-Branch Minor Commit And Sync

```bash
CURRENT_BRANCH="$(git branch --show-current)"
./.github/skills/z00z-git-versioning/scripts/version-manager.sh minor --stage-all -f -b "$CURRENT_BRANCH" -m "Description of changes"
```
```

### 2. Release Script (`scripts/release.sh`)

A simplified wrapper script for common release operations.

#### Usage:
```bash
./scripts/release.sh [OPTIONS] <COMMAND>
```

#### Examples:
```bash
# Quick patch release
./scripts/release.sh patch -m "Fix critical bug"

# Quick minor release
./scripts/release.sh minor -m "Add new feature"

# Check status
./scripts/release.sh status
```

### 3. Pre-commit Hook (`.git/hooks/pre-commit`)

Automatically runs before each commit to:
- **Remove empty files** from commits
- **Run cargo check** for Rust files
- **Check for trailing whitespace**
- **Validate file formats**

### 5. Versions.yaml Synchronization

The script automatically updates `versions.yaml` on each repository release commit:
- **total_version.version** becomes the new repository release version (`X.Y.Z`)
- **total_version.last_git_tag** becomes the matching created tag (`vX.Y.Z`)
- These two fields must always describe the same release

This ensures the versions file stays in sync with git releases.

For `crate` updates:
- update only the targeted crate version metadata
- do not create a repository release tag
- do not mutate `total_version.version`
- do not mutate `total_version.last_git_tag`

### 6. Large File Gate

The workflow enforces a default per-file git size limit:
- default maximum single-file size entering git: `50 MiB`
- applies to versioned commit flows and sync flows
- blocks staged files above the limit before commit
- blocks outgoing blob objects above the limit before push

To override intentionally for a single run:
- pass `--allow-large-files-up-to-mb <MB>`
- the `<MB>` value is the maximum allowed size for one file in that run
- if any file still exceeds that explicit limit, the workflow fails

## Workflow

### Manual Release Process

1. **Make your changes** to the codebase
2. **Stage your changes** with `git add`
3. **Create a release**:
   ```bash
   # For bug fixes
   ./scripts/release.sh patch -m "Description of changes"
   
   # For new features
   ./scripts/release.sh minor -m "Description of new features"
   
   # For breaking changes
   ./scripts/release.sh major -m "Description of breaking changes"
   ```
4. **Push to GitHub** (automatically done by the script)

### Automated Process (GitHub Actions)

When you push to the `main` branch:
1. **Version detection** - Checks if version was manually updated
2. **If version changed** → Creates GitHub release
3. **If version not changed** → Auto-increments patch version and creates release
4. **Quality checks** - Runs tests, formatting, and linting
5. **Release artifacts** - Creates and uploads release archive

## Commit Message Format

All commits created by the version management system follow the conventional format:

```
feat(vX.Y.Z): Description

Version bump: vA.B.C → vX.Y.Z
Change type: patch|minor|major

[skip ci]
```

After a repository release is pushed to `main`, the version manager dispatches
the `publish-wallet-demo` workflow when that workflow exists and GitHub CLI is
installed and authenticated. This keeps the `[skip ci]` release policy while
publishing the current wallet demo immediately. If dispatch is unavailable, the
script prints a warning and the workflow can still be started manually.

For crate updates:
```
feat(crate-name): Description

Crate update: crate-name → vX.Y.Z

[skip ci]
```

Examples:
```
feat(v7.2.0): Add new batch verification feature

Version bump: v7.1.9 → v7.2.0
Change type: minor

[skip ci]
```

```
feat(crate-bulletproofs): Update bulletproofs to 4.4.1

Crate update: bulletproofs → 4.4.1

[skip ci]
```

## Empty File Handling

The system automatically prevents committing empty files:

### Pre-commit Hook
- Detects empty files in the commit
- Removes them from the staging area
- Logs warnings about removed files

### Version Manager Script
- Checks for empty files before committing
- Excludes them from git operations
- Provides clear feedback about excluded files

## Quality Assurance

### Pre-commit Checks
- **Cargo check** for Rust files
- **Trailing whitespace** detection
- **File format** validation

### GitHub Actions Checks
- **Cargo fmt** - Code formatting
- **Cargo clippy** - Linting with warnings as errors (first-party crates only; vendored deps are not linted)
- **Cargo test** - Full test suite
- **Cargo doc** - Documentation generation

## Configuration

### Version Format
The project uses [Semantic Versioning](https://semver.org/):
- **MAJOR.MINOR.PATCH** (e.g., 7.1.9)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Git Configuration
- **Main branch**: `main`
- **Development branch**: `develop`
- **Release tags**: `vX.Y.Z`

### GitHub Integration
- **Automatic releases** for version changes
- **Release artifacts** as tar.gz archives
- **Changelog generation** from commit history

## Troubleshooting

### Common Issues

#### 1. Pre-commit hook fails
```bash
# Check for issues
cargo check
./scripts/fmt_z00z.sh --check

# Lint first-party crates with warnings-as-errors.
# NOTE: Vendored Tari crates are intentionally read-only and may have clippy warnings.
./scripts/clippy_first_party.sh

# Fix formatting
./scripts/fmt_z00z.sh

# Commit again
git commit -m "Your message"
```

#### 2. Version conflict
```bash
# Check current status
./.github/skills/z00z-git-versioning/scripts/version-manager.sh status

# Sync changes first
./.github/skills/z00z-git-versioning/scripts/version-manager.sh sync

# Then create release
./scripts/release.sh patch -m "Your message"
```

#### 3. GitHub Actions fails
- Check the Actions tab in GitHub
- Review error logs
- Fix issues locally and push again

### Manual Recovery

If the automated system fails, you can manually:

1. **Update version** in `Cargo.toml`
2. **Commit manually**:
   ```bash
   git add Cargo.toml
   git commit -m "feat(vX.Y.Z): Your message"
   git tag -a "vX.Y.Z" -m "Release vX.Y.Z"
   git push origin main
   git push origin vX.Y.Z
   ```
3. **Create GitHub release** manually in the web interface

## Best Practices

1. **Use descriptive commit messages** that explain what changed
2. **Choose the right version type**:
   - `patch` for bug fixes
   - `minor` for new features
   - `major` for breaking changes
3. **Test before releasing** - the system runs tests automatically
4. **Check status** before operations to understand current state
5. **Use dry-run** to preview actions before executing

## Integration with Development Workflow

### Daily Development
```bash
# Make changes
git add .
git commit -m "feat: Add new feature"
git push origin main
# GitHub Actions will auto-increment patch version
```

### Feature Releases
```bash
# Complete feature development
git add .
./scripts/release.sh minor -m "Add comprehensive batch verification"
# Automatically creates release and pushes to GitHub
```

### Breaking Changes
```bash
# Major API changes
git add .
./scripts/release.sh major -m "Redesign public API for better performance"
# Creates major release with proper version bump
```

This system ensures consistent, automated version management while maintaining high code quality and providing clear visibility into all changes.

---

# Version Management Implementation Summary

## ✅ Successfully Implemented

I have successfully implemented a comprehensive version management system for the Verkle Fractal project that meets all your requirements:

### ✅ Requirements Fulfilled

1. **✅ Создавать новый git commit при каждом изменении (новая версия)**
   - Automated git commits with every version change
   - Conventional commit format: `feat(vX.Y.Z): description`

2. **✅ Update version in Cargo**
   - Automatic version updates in `Cargo.toml`
   - Semantic versioning: MAJOR.MINOR.PATCH

3. **✅ Обновлять теги релиза в формате: feat(v8.0.0): ...**
   - Git tags created automatically: `vX.Y.Z`
   - Proper commit message format with version info

4. **✅ Синхронизировать изменения с GitHub**
   - Automatic push to GitHub
   - Tags pushed to remote repository
   - GitHub Actions integration for CI/CD

5. **✅ Не включать в git пустые файлы**
   - Pre-commit hook removes empty files
   - Empty files excluded from commits automatically
   - Clear warnings about excluded files

## 🚀 System Components

### 1. Version Manager Script (`scripts/version-manager.sh`)

- **Full-featured version management**
- **Commands**: `patch`, `minor`, `major`, `current`, `status`, `sync`
- **Options**: `-m` (message), `-b` (branch), `-d` (dry-run)
- **Automated**: Cargo.toml updates, git commits, GitHub sync

### 2. Release Script (`scripts/release.sh`)

- **Simplified wrapper** for common operations
- **User-friendly interface** with emoji indicators
- **Quick commands** for daily use

### 3. Pre-commit Hook (`.git/hooks/pre-commit`)

- **Empty file detection** and removal
- **Cargo check** for Rust files
- **Quality checks** (whitespace, formatting)

### 4. GitHub Actions Workflow (`.github/workflows/version-management.yml`)

- **Automated CI/CD pipeline**
- **Version detection** and auto-releases
- **Quality assurance** (fmt, clippy, tests)
- **Release artifacts** generation

### 5. Documentation (`docs/VERSION_MANAGEMENT.md`)

- **Comprehensive guide** with examples
- **Troubleshooting section**
- **Best practices** and workflows

## 📊 Demonstration Results

### ✅ Successfully Tested

- **Version update**: 7.1.9 → 7.1.10
- **Git commit**: `feat(v7.1.10): Add comprehensive version management system...`
- **Empty file removal**: 9 empty files automatically excluded
- **GitHub sync**: Changes and tags pushed successfully
- **Quality checks**: All passed (cargo check, formatting, etc.)

### 📈 Commit Statistics

- **Files changed**: 311 files
- **Insertions**: 11,587 lines
- **Deletions**: 1,354 lines
- **Empty files removed**: 9 files
- **New files created**: Version management system files

## 🎯 Usage Examples

### Daily Development

```bash
# Quick status check
./scripts/release.sh status

# Patch release for bug fixes
./scripts/release.sh patch -m "Fix critical memory leak"

# Minor release for new features
./scripts/release.sh minor -m "Add batch verification"
```

### Advanced Usage

```bash
# Full control with version manager
./.github/skills/z00z-git-versioning/scripts/version-manager.sh minor -m "Breaking API changes" -b "$(git branch --show-current)"

# Dry run to preview changes
./scripts/release.sh major -d -m "Major redesign"

# Sync changes without version bump
./.github/skills/z00z-git-versioning/scripts/version-manager.sh sync
```

## 🔧 Technical Features

### Version Management

- **Semantic Versioning** (MAJOR.MINOR.PATCH)
- **Automatic version detection** from Cargo.toml
- **Change type detection** (major/minor/patch)
- **Version validation** and format checking

### Git Integration

- **Conventional commits** with proper format
- **Annotated tags** with release notes
- **Branch management** (main/develop support)
- **Remote synchronization** with GitHub

### Quality Assurance

- **Pre-commit hooks** for code quality
- **Automated testing** in CI/CD
- **Empty file filtering** to keep repository clean
- **Formatting and linting** checks

### GitHub Integration

- **Automatic releases** with changelog
- **Release artifacts** (tar.gz archives)
- **CI/CD pipeline** with quality gates
- **Issue tracking** integration ready

## 📁 File Structure

```
verkle-fractal/
├── scripts/
│   ├── version-manager.sh    # Main version management script
│   └── release.sh           # Simplified wrapper script
├── .git/hooks/
│   └── pre-commit          # Pre-commit quality checks
├── .github/workflows/
│   └── version-management.yml  # GitHub Actions CI/CD
├── docs/
│   └── VERSION_MANAGEMENT.md   # Comprehensive documentation
└── Cargo.toml              # Version automatically updated
```

## 🎉 Benefits Achieved

### ✅ Automation

- **Zero manual version management**
- **Automated git operations**
- **GitHub integration** without manual steps

### ✅ Quality

- **Consistent commit messages**
- **No empty files** in repository
- **Automated testing** and validation

### ✅ Flexibility

- **Multiple command interfaces**
- **Dry-run capability** for testing
- **Branch support** for different workflows

### ✅ Reliability

- **Error handling** and validation
- **Rollback capability** if needed
- **Clear logging** and feedback

## 🚀 Next Steps

The system is **production-ready** and can be used immediately:

1. **Start using** `./scripts/release.sh` for daily version management
2. **Configure GitHub Actions** secrets if needed
3. **Customize commit messages** and workflows as desired
4. **Monitor GitHub releases** for automated deployments

## 📞 Support

- **Documentation**: `docs/VERSION_MANAGEMENT.md`
- **Help commands**: `./.github/skills/z00z-git-versioning/scripts/version-manager.sh --help`
- **Status checking**: `./scripts/release.sh status`

---
