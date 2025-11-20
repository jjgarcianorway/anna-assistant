# Anna Assistant Release Process

## Overview

This document describes the correct process for creating Anna releases to ensure the auto-update mechanism works properly.

---

## Critical Requirement: Asset Naming

**IMPORTANT**: The auto-updater expects specific asset names!

The daemon's auto-updater (`crates/annad/src/auto_updater.rs`) downloads assets with these exact names:
- `annactl` (simple name)
- `annad` (simple name)
- `SHA256SUMS` (checksums file)

### Why Both Versioned and Simple Names?

- **Simple names** (`annactl`, `annad`): Required for auto-updater to work
- **Versioned names** (`annactl-X.Y.Z-beta.N-...`): Helpful for manual downloads and archival

**Each release MUST include both naming conventions!**

---

## Release Checklist

### 1. Update Version Number

```bash
# Update Cargo.toml
vim Cargo.toml
# Change: version = "5.7.0-beta.XXX"
```

### 2. Build Release Binaries

```bash
cargo build --release
```

### 3. Create Release Directory

```bash
VERSION="5.7.0-beta.XXX"
mkdir -p release-v${VERSION}
cd release-v${VERSION}
```

### 4. Copy Binaries (Both Naming Conventions!)

```bash
# Versioned names (for clarity)
cp ../target/release/annactl annactl-${VERSION}-x86_64-unknown-linux-gnu
cp ../target/release/annad annad-${VERSION}-x86_64-unknown-linux-gnu

# Simple names (for auto-updater) ⭐ CRITICAL!
cp ../target/release/annactl annactl
cp ../target/release/annad annad
```

### 5. Generate Checksums

```bash
# Include ALL files in checksums
sha256sum annactl annad annactl-*.* annad-*.* > SHA256SUMS
```

Verify the checksums file contains entries for both naming conventions:
```
f8d960d4...  annactl
9a7e2bc3...  annad
f8d960d4...  annactl-5.7.0-beta.147-x86_64-unknown-linux-gnu
9a7e2bc3...  annad-5.7.0-beta.147-x86_64-unknown-linux-gnu
```

### 6. Create GitHub Release

```bash
cd ..
gh release create v${VERSION} \
  --title "Beta.XXX - Feature Description" \
  --notes "$(cat <<'EOF'
## Summary
...release notes...
EOF
)" \
  release-v${VERSION}/*
```

### 7. Verify Release Assets

```bash
gh release view v${VERSION} --json assets --jq '.assets[].name'
```

**Expected output:**
```
annactl
annactl-5.7.0-beta.XXX-x86_64-unknown-linux-gnu
annad
annad-5.7.0-beta.XXX-x86_64-unknown-linux-gnu
SHA256SUMS
```

✅ **MUST have both simple and versioned names!**

---

## Auto-Update Architecture

### How It Works

1. **Daemon checks every 10 minutes** (`CHECK_INTERVAL = 10 * 60` seconds)
2. **Fetches highest version release** from GitHub API (including prereleases)
3. **Compares versions** using semantic versioning with beta comparison
4. **Downloads binaries** if update available:
   ```rust
   let annactl_url = format!(
       "https://github.com/{}/{}/releases/download/{}/annactl",
       GITHUB_OWNER, GITHUB_REPO, release.tag_name
   );
   ```
5. **Verifies SHA256 checksums** from `SHA256SUMS` file
6. **Checks if annactl is running** (postpones if in use)
7. **Installs atomically** to `/usr/local/bin/`
8. **Restarts daemon** with new version

### Safety Features

- ✅ Checksum verification prevents corrupted downloads
- ✅ Doesn't replace binaries while annactl is in use
- ✅ Creates backups before updating
- ✅ Automatic rollback on failure
- ✅ Respects installation source (disabled for AUR)

---

## Troubleshooting

### Issue: Auto-Update Not Working

**Symptom**: Daemon stuck on old version

**Diagnosis**:
```bash
# Check what assets are in the release
gh release view v5.7.0-beta.XXX --json assets --jq '.assets[].name'
```

**Common Cause**: Missing simple-named assets (`annactl`, `annad`)

**Fix**:
```bash
# Add simple-named copies to existing release
cd release-v${VERSION}
cp annactl-*-x86_64-unknown-linux-gnu annactl
cp annad-*-x86_64-unknown-linux-gnu annad

# Regenerate checksums
sha256sum annactl annad annactl-*.* annad-*.* > SHA256SUMS

# Upload to release
gh release upload v${VERSION} annactl annad SHA256SUMS --clobber
```

### Issue: Checksum Verification Failed

**Cause**: SHA256SUMS doesn't match downloaded files

**Fix**: Regenerate SHA256SUMS with correct file names and re-upload

### Issue: "annactl is currently in use"

**Behavior**: Update postponed until annactl is idle

**Resolution**: This is normal! Update will retry in 10 minutes when annactl is not running.

---

## Version Comparison Logic

The auto-updater uses smart version comparison:

```rust
// Examples:
5.7.0-beta.9 < 5.7.0-beta.53    ✅ Correct (numeric comparison)
5.7.0-beta.53 < 5.7.0-beta.138  ✅ Correct
5.7.0-beta.147 > 5.7.0-beta.138 ✅ Correct
5.7.0-beta.1 < 5.7.0            ✅ Beta < Stable
```

**Key Point**: The `.53` vs `.138` comparison is **numeric**, not lexicographic!

---

## Manual Testing

### Test Auto-Update Mechanism

```bash
# Check daemon logs
sudo journalctl -u annad -f

# Look for these messages:
# "🎯 Update available: vX.Y.Z → vA.B.C"
# "✓ Update successfully installed: vA.B.C"
# "Restarting daemon to apply update..."
```

### Force Immediate Check

The daemon checks every 10 minutes automatically. To test immediately:

```bash
# Restart daemon (triggers 5-minute initial delay)
sudo systemctl restart annad

# Watch logs
sudo journalctl -u annad -f
```

### Verify Update Applied

```bash
annactl --version
# Should show: annactl 5.7.0-beta.XXX
```

---

## Historical Note: Beta.138 → Beta.147 Issue

**Issue**: Auto-updater stopped at beta.138, didn't update to beta.139-147

**Root Cause**: Releases beta.139-147 only had versioned asset names:
- ❌ `annactl-5.7.0-beta.147-x86_64-unknown-linux-gnu` (updater can't find this)
- ❌ Missing: `annactl` (updater expects this!)

**Fix**: Added simple-named copies to all releases going forward

**Lesson**: Always include BOTH naming conventions in releases!

---

## Future Improvements

### Potential Enhancements

1. **Asset name flexibility**: Update auto-updater to search for versioned names as fallback
2. **Explicit update command**: `annactl update now` to force immediate check
3. **Update notifications**: Show update banner in TUI when new version available
4. **Rollback command**: `annactl rollback` to revert to previous version
5. **Release channels**: Stable vs Beta update tracks

### GitHub Actions

Consider automating the release process with GitHub Actions:
- Build binaries on tag push
- Create both naming conventions automatically
- Generate checksums
- Upload to GitHub Releases

---

## Summary

**Golden Rule**: Every release MUST have these 5 assets:

1. ✅ `annactl` (simple name for auto-updater)
2. ✅ `annad` (simple name for auto-updater)
3. ✅ `annactl-X.Y.Z-beta.N-x86_64-unknown-linux-gnu` (versioned name)
4. ✅ `annad-X.Y.Z-beta.N-x86_64-unknown-linux-gnu` (versioned name)
5. ✅ `SHA256SUMS` (checksums for all 4 binaries)

Follow this process and auto-updates will work flawlessly! 🚀

---

**Last Updated**: Beta.147 (2025-11-20)
**Maintainer**: Anna Assistant Development Team
