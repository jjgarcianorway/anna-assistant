# Version Management Guide

This guide explains how Anna Assistant ensures version consistency across all components.

## The Problem

In a project with multiple version touchpoints (source code, built binaries, installed system binaries, GitHub releases), it's easy for versions to become out of sync, leading to confusion and bugs.

## The Solution

Anna uses a multi-layered approach to prevent version mismatches:

### 1. Verification Script (`scripts/verify_versions.sh`)

A comprehensive standalone script that checks:
- ✓ Source version (Cargo.toml)
- ✓ Installed system version (/usr/local/bin/annactl)
- ✓ Local build version (./target/release/annactl)
- ✓ Latest GitHub tag
- ✓ Latest GitHub release with assets

**Usage:**
```bash
./scripts/verify_versions.sh
```

**Output:**
- Shows all versions side-by-side
- Reports mismatches with actionable recommendations
- Exits with code 1 if any mismatch detected

### 2. Built-in Version Check (`annactl version --check`)

The `annactl` command includes a `--check` flag that runs the verification script:

```bash
annactl version --check
```

This makes it easy to verify versions from anywhere on the system.

### 3. Post-Release Validation

The `release.sh` script now includes a post-release checklist:

```bash
./scripts/release.sh
```

After creating a release, it reminds you to:
1. Test the installer
2. Run version verification
3. Confirm installation

### 4. CI/CD Validation (`.github/workflows/version-check.yml`)

GitHub Actions automatically validates versions on:
- ✓ Every push to main/develop
- ✓ Every pull request
- ✓ Every release tag

The workflow:
- Runs `verify_versions.sh`
- Comments on PRs if versions mismatch
- Validates post-release that assets are uploaded
- Ensures tag version matches source version

### 5. Pre-Commit Hook (`.githooks/pre-commit`)

A Git hook that runs before each commit:
- Detects version changes in Cargo.toml
- Warns about source/build version mismatches
- Provides post-bump reminders

**Enable hooks:**
```bash
git config core.hooksPath .githooks
```

## Workflow: Making a New Release

### Step 1: Develop and Test
```bash
# Make your changes
# ...

# Build and test
cargo build --release
./target/release/annactl version
```

### Step 2: Create Release
```bash
./scripts/release.sh
```

This script:
1. Detects changes since last release
2. Computes next version (auto-increments RC number)
3. Updates Cargo.toml
4. Commits and tags
5. Pushes to GitHub
6. Waits for GitHub Actions to build
7. Shows post-release checklist

### Step 3: Install and Verify
```bash
# Install the new version
sudo ./scripts/install.sh

# Verify everything is in sync
./scripts/verify_versions.sh

# Or use the built-in check
annactl version --check
```

## Workflow: Daily Development

### Before Committing
```bash
# If you modified Cargo.toml, rebuild
cargo build --release

# Verify versions before commit (pre-commit hook does this automatically)
./scripts/verify_versions.sh
```

### After Pulling Changes
```bash
# Check if versions changed
./scripts/verify_versions.sh

# If mismatch, rebuild
cargo build --release
```

### Before Release
```bash
# Ensure everything is in sync
./scripts/verify_versions.sh

# If all green, proceed with release
./scripts/release.sh
```

## Troubleshooting

### "Local build version mismatch"

**Problem:** Your `./target/release/annactl` version doesn't match `Cargo.toml`

**Solution:**
```bash
cargo build --release
```

### "Installed version mismatch"

**Problem:** System installation (`/usr/local/bin/annactl`) is outdated

**Solution:**
```bash
sudo ./scripts/install.sh
```

### "Source version ahead of GitHub tag"

**Problem:** Your local changes aren't released yet

**Solution:**
```bash
# If ready to release
./scripts/release.sh

# If not ready, this is normal during development
```

### "GitHub tag has no assets"

**Problem:** GitHub Actions hasn't finished building yet

**Solution:**
1. Check https://github.com/jjgarcianorway/anna-assistant/actions
2. Wait for build to complete
3. Re-run installer: `sudo ./scripts/install.sh`

## Architecture

### Version Sources

```
┌─────────────────────────────────────────────────────────────┐
│                     Version Sources                         │
├─────────────────────────────────────────────────────────────┤
│  1. Source Code (Cargo.toml)         → Ground Truth        │
│  2. Local Build (target/release/)    → Development         │
│  3. Installed (/usr/local/bin/)      → System State        │
│  4. GitHub Tag (git)                 → Release Marker      │
│  5. GitHub Release (assets)          → Distribution        │
└─────────────────────────────────────────────────────────────┘
```

### Validation Flow

```
┌──────────────┐
│  Developer   │
│   Changes    │
└──────┬───────┘
       │
       ▼
┌──────────────┐      ┌─────────────────┐
│  Pre-commit  │─────▶│  Warn if build  │
│     Hook     │      │  version stale  │
└──────┬───────┘      └─────────────────┘
       │
       ▼
┌──────────────┐
│  Push to GH  │
└──────┬───────┘
       │
       ▼
┌──────────────┐      ┌─────────────────┐
│ CI: Version  │─────▶│  Fail PR if     │
│    Check     │      │  mismatch       │
└──────┬───────┘      └─────────────────┘
       │
       ▼
┌──────────────┐
│  Merge to    │
│    Main      │
└──────┬───────┘
       │
       ▼
┌──────────────┐      ┌─────────────────┐
│ release.sh   │─────▶│  Tag & Build    │
└──────┬───────┘      └─────────────────┘
       │
       ▼
┌──────────────┐      ┌─────────────────┐
│ CI: Post-    │─────▶│  Verify assets  │
│   Release    │      │  & tag match    │
└──────┬───────┘      └─────────────────┘
       │
       ▼
┌──────────────┐
│  install.sh  │
└──────┬───────┘
       │
       ▼
┌──────────────┐      ┌─────────────────┐
│ annactl      │─────▶│  Final check:   │
│ version      │      │  All in sync!   │
│  --check     │      └─────────────────┘
└──────────────┘
```

## Best Practices

### DO ✓
- Run `./scripts/verify_versions.sh` frequently
- Enable Git hooks: `git config core.hooksPath .githooks`
- Rebuild after pulling changes: `cargo build --release`
- Follow the post-release checklist
- Let `release.sh` handle version bumps (automatic)

### DON'T ✗
- Don't manually edit version numbers (release.sh does this)
- Don't skip version checks before releasing
- Don't force-push without verifying versions
- Don't bypass pre-commit hooks without good reason

## Summary

With these safeguards in place, version mismatches should **never happen again**:

1. ✓ **Automatic detection** - Multiple layers catch mismatches
2. ✓ **Clear remediation** - Scripts tell you exactly how to fix issues
3. ✓ **Proactive prevention** - Hooks warn before problems occur
4. ✓ **CI enforcement** - PRs can't merge with version issues
5. ✓ **Post-release validation** - Ensures releases are fully deployed

Run this command regularly:
```bash
./scripts/verify_versions.sh
```

If it's all green, you're good! 🎉
