# Version Management Quick Reference

## Single Source of Truth

```
VERSION file → Everything else
```

## Common Tasks

### Bump Version

```bash
make bump VERSION=v1.0.0-rc.17
# Edit CHANGELOG.md
git add VERSION Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to v1.0.0-rc.17"
```

### Release

```bash
make release
# Or: ./scripts/release.sh
```

### Check Consistency

```bash
make check-version
# Or: ./scripts/check-version-consistency.sh
```

### Verify All Versions

```bash
./scripts/verify_versions.sh
```

### Install

```bash
# Default: VERSION from main branch
sudo ./scripts/install.sh

# Specific version
sudo ./scripts/install.sh --version v1.0.0-rc.16

# Latest stable (no RCs)
sudo ./scripts/install.sh --stable

# Dev: from local build
make install
```

## File Locations

| File | Purpose |
|------|---------|
| `VERSION` | Single source of truth |
| `src/*/build.rs` | Embeds VERSION into binaries |
| `scripts/release.sh` | Transactional release |
| `scripts/install.sh` | Exact-version installer |
| `scripts/check-version-consistency.sh` | Local checks |
| `scripts/verify_versions.sh` | GitHub checks |
| `.github/workflows/release.yml` | CI validation |
| `Makefile` | Developer shortcuts |

## Release Flow

```
1. make bump VERSION=vX.Y.Z
2. Update CHANGELOG.md
3. git commit
4. make release
   ├─ Phase 1: Preflight checks
   ├─ Phase 2: Create/check tag
   ├─ Phase 3: Push tag
   └─ Phase 4: Wait for CI
5. GitHub Actions builds & uploads
6. Users install with exact version
```

## Validation Checks

### Local Build
```bash
cargo build --release
./target/release/annactl --version
# Should match VERSION file (without 'v')
```

### Consistency
```bash
./scripts/check-version-consistency.sh
# ✓ VERSION file valid
# ✓ Cargo.toml matches
# ✓ Binaries match
```

### GitHub Status
```bash
./scripts/verify_versions.sh
# Shows: source, builds, installed, GitHub
```

## Troubleshooting

### Version Mismatch
```bash
cargo clean && cargo build --release
```

### Tag Already Exists
```bash
./scripts/release.sh  # Idempotent - safe to retry
```

### Assets Not Ready
```bash
# Check: https://github.com/jjgarcianorway/anna-assistant/actions
./scripts/release.sh  # Will poll and wait
```

### Install Timeout
```bash
# Wait for CI, then retry
sudo ./scripts/install.sh  # Will poll and wait
```

## Rules

✅ **DO**:
- Use `make bump` to change versions
- Update CHANGELOG.md
- Let CI build binaries
- Run `make check-version` before release

❌ **DON'T**:
- Edit Cargo.toml version manually
- Create tags manually
- Upload assets manually
- Skip CHANGELOG updates

## CI Guardrails

GitHub Actions will **fail** if:
- Tag name != VERSION file
- Cargo.toml != VERSION file
- Binary version != VERSION file

This ensures version consistency.

---

**See Also**: `docs/ONE-TRUTH-VERSIONING.md` for complete documentation
