# Anna Assistant Scripts

This directory contains scripts for installation, maintenance, and development of Anna Assistant.

## Installation & Release

### `install.sh` - Smart Installer
**Primary installation script with intelligent binary handling.**

**Features:**
- Downloads pre-compiled binaries from GitHub releases (no Rust needed)
- Falls back to building from source if downloads fail
- Supports offline installation via `./bin/` directory
- Architecture detection (x86_64, aarch64)

**Usage:**
```bash
# Standard installation (tries download first)
./scripts/install.sh

# Force build from source
./scripts/install.sh --build

# Show help
./scripts/install.sh --help
```

**Requirements:**
- For binary download: curl or wget
- For source build: Rust/Cargo (`sudo pacman -S rust`)

---

### `release.sh` - Automated Release
**Automates version bumping and release creation.**

**Features:**
- Auto-increments version (major/minor/patch)
- Updates version in all files (Cargo.toml, install.sh, PKGBUILDs)
- Creates git commit and tag
- Pushes to GitHub (triggers CI build)

**Usage:**
```bash
# Patch release (0.11.1 -> 0.11.2)
./scripts/release.sh -t patch -m "Fix installation bug"

# Minor release (0.11.1 -> 0.12.0)
./scripts/release.sh -t minor -m "Add new features"

# Major release (0.11.1 -> 1.0.0)
./scripts/release.sh -t major -m "Breaking changes"

# Explicit version
./scripts/release.sh -v 0.11.2 -m "Hotfix"

# Dry run (preview)
./scripts/release.sh -t patch -m "Test" --dry-run
```

---

### `uninstall.sh` - Uninstaller
**Removes Anna Assistant from the system.**

**Usage:**
```bash
./scripts/uninstall.sh
```

**What it removes:**
- Binaries from /usr/local/bin
- Systemd services
- System user and group (optional)
- Data directories (optional, prompted)

---

## Diagnostics & Utilities

### `anna-diagnostics.sh` - System Diagnostics
Comprehensive system health check and diagnostic tool.

**Usage:**
```bash
./scripts/anna-diagnostics.sh
```

---

### `verify_installation.sh` - Installation Verification
Verifies that Anna is correctly installed and operational.

**Usage:**
```bash
./scripts/verify_installation.sh
```

---

### `verify_socket_persistence.sh` - Socket Persistence Test
Tests RPC socket persistence and permissions across daemon restarts.

**Usage:**
```bash
sudo ./scripts/verify_socket_persistence.sh
```

---

### `collect_debug.sh` - Debug Information Collector
Collects system information and logs for debugging issues.

**Usage:**
```bash
./scripts/collect_debug.sh
```

**Output:** Creates a tarball with logs, config, and system info.

---

### `fix_v011_installation.sh` - Emergency Repair
Emergency repair script for v0.11.0 installations with issues.

**Usage:**
```bash
sudo ./scripts/fix_v011_installation.sh
```

**Fixes:**
- Directory ownership and permissions
- Missing CAPABILITIES.toml
- Socket permissions
- Service configuration

---

## Hardware-Specific

### `anna_fans_asus.sh` - ASUS Thermal Management
Thermal management script for ASUS laptops.

**Features:**
- Monitors CPU temperatures
- Adjusts fan profiles based on load
- Integrates with asusctl

**Usage:**
```bash
# Installed automatically by install.sh on ASUS hardware
# Runs as systemd service: anna-fans.service
systemctl status anna-fans
```

---

## CI/Testing

### `ci_smoke.sh` - CI Smoke Tests
Quick smoke tests for continuous integration.

**Usage:**
```bash
./scripts/ci_smoke.sh
```

---

## Directory Structure

```
scripts/
├── README.md                      # This file
├── install.sh                     # Smart installer ⭐
├── release.sh                     # Release automation ⭐
├── uninstall.sh                   # Uninstaller
├── anna-diagnostics.sh            # System diagnostics
├── verify_installation.sh         # Installation verification
├── verify_socket_persistence.sh   # Socket testing
├── collect_debug.sh               # Debug collector
├── fix_v011_installation.sh       # Emergency repair
├── anna_fans_asus.sh              # ASUS thermal management
├── ci_smoke.sh                    # CI smoke tests
└── archive/                       # Old/deprecated scripts
    └── README.md                  # Archive documentation
```

---

## Quick Reference

### First-Time Installation
```bash
./scripts/install.sh
```

### Create a Release
```bash
./scripts/release.sh -t patch -m "Your commit message"
```

### Verify Installation
```bash
./scripts/verify_installation.sh
annactl doctor check
```

### Troubleshooting
```bash
./scripts/anna-diagnostics.sh
./scripts/collect_debug.sh
```

### Uninstall
```bash
./scripts/uninstall.sh
```

---

## For Developers

### Testing Installation Locally
```bash
# Build binaries
cargo build --release

# Test installer
./scripts/install.sh --build
```

### Creating a Test Release
```bash
# Dry run to preview changes
./scripts/release.sh -t patch -m "Test release" --dry-run

# Actually create the release
./scripts/release.sh -t patch -m "Test release"
```

### Debugging Installation Issues
```bash
# Collect debug info
./scripts/collect_debug.sh

# Run diagnostics
./scripts/anna-diagnostics.sh

# Check specific components
./scripts/verify_installation.sh
```

---

## See Also

- [INSTALLATION.md](../INSTALLATION.md) - Complete installation guide
- [docs/RELEASE-CHECKLIST.md](../docs/RELEASE-CHECKLIST.md) - Release process
- [docs/TROUBLESHOOTING.md](../docs/TROUBLESHOOTING.md) - Common issues
- [README.md](../README.md) - Project overview

---

## Archive

Old/deprecated scripts are moved to `archive/` directory for historical reference. **Do not use archived scripts for new installations.**
