# One-Truth Versioning System

**Status**: ✅ Implemented
**Version**: v1.0.0-rc.16

## Overview

This document describes Anna's single-source-of-truth versioning system that eliminates version drift between releases, GitHub, and installations.

## The Problem (Before)

❌ **Version Drift Issues:**
- `Cargo.toml` had one version
- Git tags had another version
- Releases sometimes didn't have assets
- Installer grabbed "latest-ish" releases
- No idempotent releases (re-running created duplicates)
- No transactional guarantees

## The Solution (Now)

✅ **Single Source of Truth: `VERSION` file**

```
VERSION (at repo root)
  ↓
  ├─→ Cargo.toml (synced via make bump)
  ├─→ build.rs (embeds into binaries)
  ├─→ release.sh (creates exact tag)
  ├─→ CI workflow (validates consistency)
  └─→ install.sh (installs exact version)
```

## Components

### 1. VERSION File (Single Source of Truth)

**Location**: `/VERSION` (repo root)

**Format**:
```
v1.0.0-rc.16
```

**Rules**:
- Must start with `v`
- Semver format: `v<major>.<minor>.<patch>` or `v<major>.<minor>.<patch>-rc.<num>`
- Only modified via `make bump VERSION=vX.Y.Z`
- ALL other version sources derive from this

### 2. build.rs (Version Embedding)

**Files**:
- `src/annactl/build.rs`
- `src/annad/build.rs`

**Function**:
- Reads `VERSION` file at build time
- Sets `ANNA_VERSION` env var (with 'v' prefix)
- Overrides `CARGO_PKG_VERSION` (without 'v' prefix)
- Validates format
- Triggers rebuild if `VERSION` changes

**Result**: Binaries always embed the correct version from `VERSION` file.

### 3. release.sh (Transactional & Idempotent)

**Location**: `scripts/release.sh`

**Phases**:

#### Phase 0: Read Single Source of Truth
- Read `VERSION` file
- Validate format

#### Phase 1: Preflight Hard Checks
- ✅ Git tree clean
- ✅ Cargo.toml matches VERSION
- ✅ Binaries build successfully
- ✅ Binaries embed correct version
- ✅ CHANGELOG.md mentions version (optional)

#### Phase 2: Idempotent Tag Logic
- If tag exists remotely + assets complete → **exit 0** (idempotent!)
- If tag exists but assets incomplete → **resume** (retry-safe)
- Else → create tag

#### Phase 3: Create & Push Tag
- Create annotated tag
- Push to origin
- Cleanup on failure

#### Phase 4: Poll for CI Assets
- Wait up to 10 minutes for:
  - `anna-linux-x86_64.tar.gz`
  - `anna-linux-x86_64.tar.gz.sha256`
- Verify checksum file is non-empty
- Exit with success or diagnostic error

**Idempotent**: Running twice with same VERSION is a no-op
**Transactional**: Either complete release or fail cleanly with rollback

### 4. CI Workflow (Enforcement)

**File**: `.github/workflows/release.yml`

**Validation Steps**:

1. **VERSION File Consistency**
   ```bash
   - Tag name must exactly match VERSION file
   - Cargo.toml must match VERSION (without 'v')
   ```

2. **Build Binaries**
   ```bash
   cargo build --release --bin annad --bin annactl
   ```

3. **Verify Embedded Versions**
   ```bash
   - Run each binary with --version
   - Compare to VERSION file
   - Fail if mismatch
   ```

4. **Package & Upload**
   ```bash
   - Create tarball with VERSION file included
   - Generate SHA256 checksum
   - Upload: tarball, checksum, VERSION file
   - Mark as prerelease if contains '-rc.'
   ```

**Guardrails**: Builds fail fast if any version mismatch detected.

### 5. install.sh (Exact-Version Installer)

**Location**: `scripts/install.sh`

**Version Resolution Order**:

1. `--version X` → Use exactly X
2. `--stable` → Latest stable (non-RC) release from GitHub
3. Default → Fetch `VERSION` from main branch

**Features**:

- ✅ **No "latest-ish"**: Installs exact tag or nothing
- ✅ **Asset Polling**: Waits up to 5 minutes if assets not ready
- ✅ **Checksum Verification**: SHA256 before install
- ✅ **Post-Install Verification**: Runs binaries, checks versions
- ✅ **Automatic Revert**: Deletes binaries if version mismatch
- ✅ **RC Detection**: Warns if installing pre-release

**Usage**:
```bash
sudo ./scripts/install.sh                     # Install VERSION from main
sudo ./scripts/install.sh --version v1.2.3    # Install exact version
sudo ./scripts/install.sh --stable            # Latest stable only
sudo ./scripts/install.sh --from-local        # Dev: local build
```

### 6. Makefile (Developer Ergonomics)

**Key Targets**:

#### `make bump VERSION=vX.Y.Z`
- Validates semver format
- Updates `VERSION` file
- Updates `Cargo.toml`
- Reminds to update `CHANGELOG.md`
- Shows next steps

#### `make check-version`
- Runs `scripts/check-version-consistency.sh`
- Validates all version sources

#### `make release`
- Runs `scripts/release.sh`
- Transactional release process

#### `make build`
- Builds with VERSION embedding

#### `make install`
- Installs from local build

### 7. Verification Scripts

#### `scripts/check-version-consistency.sh`
- Checks VERSION file exists and is valid
- Verifies Cargo.toml matches
- Checks built binaries (if present)
- Checks installed binaries (if present)
- Checks latest git tag
- Reports errors/warnings

#### `scripts/verify_versions.sh`
- Enhanced version with GitHub API checks
- Shows full version matrix
- Checks for release assets
- Detects CI build status

## Workflows

### Developer Workflow: Bump Version

```bash
# 1. Bump version
make bump VERSION=v1.0.0-rc.17

# 2. Update CHANGELOG.md
vim CHANGELOG.md

# 3. Review changes
git diff

# 4. Commit
git add VERSION Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to v1.0.0-rc.17"

# 5. Release
make release
```

### Release Workflow: Create New Release

```bash
# 1. Release (automatically reads VERSION file)
./scripts/release.sh

# Output:
# Phase 0: Read VERSION ✓
# Phase 1: Preflight checks ✓
# Phase 2: Tag logic (create or skip) ✓
# Phase 3: Push tag ✓
# Phase 4: Wait for CI (polls for assets) ✓
```

### CI Workflow: Automated Build

```yaml
Trigger: Tag push (v*)
Steps:
  1. Validate VERSION file matches tag
  2. Build binaries
  3. Verify embedded versions
  4. Package tarball (with VERSION)
  5. Generate checksum
  6. Upload to release
```

### Install Workflow: Exact Version

```bash
# Default: Use VERSION from main
sudo ./scripts/install.sh

# Specific version
sudo ./scripts/install.sh --version v1.0.0-rc.16

# Latest stable (skip RCs)
sudo ./scripts/install.sh --stable

# Dev: from local build
make install
```

## Version Format Rules

### Stable Releases
```
v1.0.0
v1.2.3
```

### Release Candidates
```
v1.0.0-rc.1
v1.0.0-rc.16
```

### Invalid Formats
```
1.0.0          # Missing 'v' prefix
v1.0           # Incomplete semver
v1.0.0-beta.1  # Wrong suffix (must be -rc.N)
```

## Validation & Testing

### Pre-Release Checks

```bash
# Check version consistency
make check-version

# Should show:
# ✓ VERSION file exists and is valid
# ✓ Cargo.toml matches VERSION
# ✓ Built binaries match VERSION
# ⚠ Not installed (or version mismatch - normal during dev)
```

### Post-Release Verification

```bash
# Verify versions
./scripts/verify_versions.sh

# Should show:
# ✓ Source version matches VERSION file
# ✓ Local build matches
# ✓ GitHub tag matches
# ✓ GitHub release with assets exists
```

### CI Validation

GitHub Actions automatically:
- Validates VERSION consistency
- Builds binaries
- Verifies embedded versions
- Uploads assets
- Marks RC releases as prerelease

## Error Handling

### Build Errors

**Symptom**: `cargo build` fails
**Cause**: VERSION file missing or invalid
**Fix**:
```bash
echo "v1.0.0-rc.16" > VERSION
cargo clean
cargo build --release
```

### Version Mismatch

**Symptom**: Binary version != VERSION file
**Cause**: Stale build cache
**Fix**:
```bash
cargo clean
cargo build --release
./target/release/annactl --version  # Should match VERSION
```

### Release Timeout

**Symptom**: release.sh times out waiting for assets
**Cause**: CI build failed or still running
**Fix**:
1. Check: https://github.com/jjgarcianorway/anna-assistant/actions
2. Fix build errors
3. Re-run: `./scripts/release.sh` (idempotent - safe to retry)

### Install Polling Timeout

**Symptom**: install.sh times out waiting for assets
**Cause**: Release not complete
**Fix**:
1. Check release: https://github.com/jjgarcianorway/anna-assistant/releases
2. Wait for CI to complete
3. Re-run: `sudo ./scripts/install.sh` (will retry)

## Migration from Old System

### What Changed

**Before**:
- ❌ Multiple version sources
- ❌ Manual version bumps in Cargo.toml
- ❌ release.sh calculated next version
- ❌ Could create duplicate tags
- ❌ Installer used "latest" release

**After**:
- ✅ Single VERSION file
- ✅ `make bump` manages all versions
- ✅ release.sh reads VERSION file
- ✅ Idempotent (running twice is safe)
- ✅ Installer uses exact versions

### Migration Steps

1. ✅ Created `VERSION` file
2. ✅ Added build.rs to both binaries
3. ✅ Rewrote release.sh (transactional)
4. ✅ Updated CI workflow (validation)
5. ✅ Rewrote install.sh (exact-version)
6. ✅ Added Makefile targets
7. ✅ Updated verification scripts

## Best Practices

### DO ✅

- Use `make bump VERSION=vX.Y.Z` to change versions
- Update CHANGELOG.md for every version
- Run `make check-version` before releasing
- Let CI build binaries (don't upload manually)
- Use `--from-local` only for dev testing

### DON'T ❌

- Don't edit Cargo.toml version directly
- Don't create tags manually (use release.sh)
- Don't skip CHANGELOG updates
- Don't force-push tags
- Don't upload release assets manually

## Future Enhancements

### Planned Features

- [ ] Log rotation (1MB threshold, 5 files)
- [ ] Rollback preview with diff
- [ ] Remote backup integration
- [ ] `annactl release doctor` command (version diagnostics)

### Under Consideration

- [ ] Homebrew formula (auto-update VERSION)
- [ ] AUR package (auto-update VERSION)
- [ ] Docker images (tagged with VERSION)
- [ ] Nix flake (pinned to VERSION)

## Troubleshooting

### "VERSION file not found"

```bash
# Create it
echo "v1.0.0-rc.16" > VERSION
```

### "Cargo.toml version mismatch"

```bash
# Fix with make bump
make bump VERSION=v1.0.0-rc.16
```

### "Binary version mismatch"

```bash
# Rebuild from scratch
cargo clean
cargo build --release
./target/release/annactl --version
```

### "Tag already exists"

```bash
# Idempotent - run release.sh again
./scripts/release.sh
# Will detect existing tag and either:
# - Exit 0 if complete
# - Resume if incomplete
```

### "Assets not ready"

```bash
# Check CI
open https://github.com/jjgarcianorway/anna-assistant/actions

# Wait and retry
./scripts/release.sh  # Safe to retry
```

## Summary

### North Star Achieved ✅

- ✅ **One-Truth**: VERSION file is sole authority
- ✅ **Transactions**: Releases are atomic operations
- ✅ **Deterministic**: Installer is exact-version, verified checksum
- ✅ **Idempotent**: Safe to retry all operations
- ✅ **Validated**: CI enforces consistency

### Key Files

```
VERSION                                  # Single source of truth
src/annactl/build.rs                     # Version embedding
src/annad/build.rs                       # Version embedding
scripts/release.sh                       # Transactional release
scripts/install.sh                       # Exact-version installer
scripts/check-version-consistency.sh     # Local validation
scripts/verify_versions.sh               # GitHub validation
.github/workflows/release.yml            # CI enforcement
Makefile                                 # Developer ergonomics
```

### Success Metrics

- ✅ VERSION file is authoritative
- ✅ Releases are transactional (all-or-nothing)
- ✅ Idempotent operations (safe retries)
- ✅ No version drift possible
- ✅ CI validates everything
- ✅ Installer is deterministic

---

**Last Updated**: 2025-11-04
**System Version**: v1.0.0-rc.16
**Status**: Production Ready
