# Release & Install Guide

## TL;DR - Just Works™

```bash
# To release a new version
./scripts/release.sh

# To install latest version
sudo ./scripts/install.sh
```

That's it. Everything else is automatic.

---

## release.sh - Auto-Release

**What it does automatically:**

1. Reads `VERSION` file
2. Syncs `Cargo.toml` if needed
3. Builds binaries
4. Verifies versions match
5. **Auto-commits** any changes
6. Creates git tag
7. Pushes to GitHub
8. Waits for CI to build assets
9. Confirms release is complete

**Usage:**
```bash
./scripts/release.sh
```

**Idempotent:** Running twice does nothing (detects existing release)

**No interaction needed** - commits happen automatically

---

## install.sh - Auto-Installer

**What it does automatically:**

1. Finds latest GitHub release with assets
2. Downloads tarball + checksum
3. **Waits if assets not ready** (up to 5 minutes)
4. Verifies SHA256
5. Installs to `/usr/local/bin`
6. Creates directories
7. Sets up systemd service
8. Verifies installation

**Usage:**
```bash
sudo ./scripts/install.sh
```

**Waits for CI** - if release just happened, installer polls until assets are ready

**Always installs latest** - finds newest release with complete assets

---

## Typical Workflow

### 1. Bump version
```bash
# Edit VERSION file
echo "v1.0.0-rc.17" > VERSION

# Optional: edit CHANGELOG.md
```

### 2. Release
```bash
./scripts/release.sh
# Auto-commits
# Auto-tags
# Auto-pushes
# Waits for CI
# Shows release URL when done
```

### 3. Install (anywhere)
```bash
sudo ./scripts/install.sh
# Finds latest
# Waits if needed
# Installs & verifies
```

---

## Error Handling

Both scripts:
- ✅ Auto-retry downloads
- ✅ Wait for CI (with timeout)
- ✅ Verify versions match
- ✅ Show clear errors
- ✅ Safe to re-run

---

## Version Matching

**How versions stay in sync:**

```
VERSION file (v1.0.0-rc.16)
  ↓
release.sh syncs Cargo.toml
  ↓
build.rs embeds into binaries
  ↓
CI builds + uploads
  ↓
install.sh downloads latest
  ↓
Verification ensures match
```

Everything flows from `VERSION` file → always consistent

---

## CI Integration

`.github/workflows/release.yml` automatically:
- Validates VERSION matches tag
- Builds binaries
- Verifies embedded versions
- Uploads tarball + checksum
- Marks RCs as prerelease

**No manual uploads needed**

---

## Common Questions

**Q: What if I run release.sh twice?**
A: Second run detects existing release and exits immediately

**Q: What if CI is still building when I install?**
A: Installer waits up to 5 minutes, polling every 10 seconds

**Q: How do I bump the version?**
A: Edit `VERSION` file, then run `./scripts/release.sh`

**Q: Do I need to commit manually?**
A: No, `release.sh` auto-commits everything

**Q: What if versions mismatch?**
A: Install fails with clear error showing mismatch

**Q: Can I install specific version?**
A: Current version always installs latest. Edit script if you need specific version.

---

## Files

| File | Purpose |
|------|---------|
| `VERSION` | Single source of truth |
| `scripts/release.sh` | Auto-release script |
| `scripts/install.sh` | Auto-installer |
| `src/*/build.rs` | Embeds VERSION into binaries |
| `.github/workflows/release.yml` | CI automation |

---

**Everything just works. No babysitting required.**
