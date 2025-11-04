# Simple Workflow - Just 2 Commands

## For LOCAL Development/Testing

```bash
# 1. Build (handles git, versioning, everything)
./scripts/release.sh

# 2. Install from local build
sudo ./scripts/install_local.sh
```

## For END USERS (Download from GitHub)

```bash
# Just one command - downloads latest release
sudo ./scripts/install.sh
```

## What Each Script Does

### `release.sh` (for developers)
- Reads VERSION file
- Builds binaries
- Auto-commits changes
- Creates git tag
- Pushes to GitHub (triggers CI to build release)

### `install_local.sh` (for developers)
- Installs from `target/release/` binaries
- Stops daemon, installs, starts daemon

### `install.sh` (for end users)
- Downloads latest release from GitHub
- Verifies checksum
- Installs binaries
- Sets up systemd service

## Summary

**Developers**: `release.sh` → `install_local.sh`
**End Users**: `install.sh` (that's it!)
