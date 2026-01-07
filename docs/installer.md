# Anna Installer and Uninstaller Guide

This document explains how to install, configure, and uninstall Anna system assistant.

## Overview

Anna supports two installation modes:

- **System mode** (default): System-wide installation with multi-user support
- **User mode**: Single-user installation for development/testing

## Installation

### Quick Start

```bash
# Default system mode installation
./scripts/install_anna.sh

# Force user mode (dev/test)
./scripts/install_anna.sh --user

# Repair existing installation
./scripts/install_anna.sh --repair
```

### Installation Modes

#### System Mode (Default, Production)

System mode provides:
- Multi-user support with per-user data isolation
- Centralized daemon running as root
- Group-based permissions (anna group)
- RPC over Unix socket at `/run/anna/annad.sock`
- No sudo required for read operations

**Paths:**
- Binaries: `/usr/local/sbin/annad`, `/usr/local/bin/annactl`
- Service: `/etc/systemd/system/annad.service`
- Config: `/etc/anna/`
- Per-user data: `/var/lib/anna/users/<uid>/`
- Policy files: `/etc/anna/policy.d/<uid>.toml`
- Socket: `/run/anna/annad.sock` (0660 root:anna)

**Setup includes:**
- Creates `anna` group
- Adds installing user to `anna` group
- Creates per-user data directories with 2770 root:anna permissions
- Generates default policy file
- Enables and starts systemd service

#### User Mode (Development/Testing)

User mode provides:
- Single-user installation
- No system-wide changes
- All files under `~/.anna/`
- User systemd service

**Paths:**
- Binaries: `~/.local/bin/annad`, `~/.local/bin/annactl`
- Service: `~/.config/systemd/user/annad.service`
- Config: `~/.anna/config/`
- Data: `~/.anna/data/`
- Socket: `~/.anna/run/annad.sock`

### Installation Flags

```bash
./scripts/install_anna.sh [OPTIONS]

Options:
  --system        Force system mode (default)
  --user          Force user mode (dev/test only)
  --repair        Re-run setup without clobbering data
  --no-compile    Skip cargo build if binaries already present
  --help          Show help message
```

### Post-Install Verification

After installation completes, the installer automatically runs these checks:

1. `annactl status` - Verify daemon connection
2. `annactl doctor perms` - Check permissions and configuration
3. `annactl quickscan` - Test RPC access (may require re-login if just added to group)

**If quickscan fails:**
```bash
# Check if you were just added to anna group (requires re-login)
groups

# Or temporarily activate group
newgrp anna

# Check daemon status
systemctl status annad

# View recent logs
journalctl -u annad -n 200
```

## Default Policy

System mode installations create a default policy at `/etc/anna/policy.d/<uid>.toml`:

```toml
[level]
auto_apply = 1  # SafeMaintenance

[approval]
confirm_dangerous = true
prompt_style = "interactive"
```

**Policy levels:**
- 0 = Manual (no auto-apply)
- 1 = SafeMaintenance (cache cleanup, orphan packages)
- 2 = SafeModerate (+ cpu governor, ntp sync)
- 3 = FullAutonomy (all non-dangerous actions)

Edit this file to adjust your automation level.

## Uninstallation

### Quick Start

```bash
# Default: backup data, remove program files
./scripts/uninstall_anna.sh

# Keep data in place (remove program files only)
./scripts/uninstall_anna.sh --keep-data

# Remove everything (no backup)
./scripts/uninstall_anna.sh --purge
```

### Uninstaller Behavior

**Default behavior:**
1. Backs up data to `~/Documents/anna_backup/<timestamp>/`
2. Stops and disables service
3. Removes binaries and service files
4. Removes data and config directories
5. Preserves `anna` group (manual removal if desired)

**With --keep-data:**
- Removes program files only
- Leaves data/config in place

**With --purge:**
- Removes everything
- No backup created

### Uninstaller Flags

```bash
./scripts/uninstall_anna.sh [OPTIONS]

Options:
  --purge       Remove everything including data (no backup)
  --keep-data   Remove program files only, keep data in place
  --help        Show help message
```

## Troubleshooting

### Socket Not Found

```bash
# Check service status
systemctl status annad

# Start service
sudo systemctl start annad

# Enable at boot
sudo systemctl enable annad

# View logs
journalctl -u annad -n 50
```

### Permission Denied on Socket

```bash
# Check group membership
groups

# Add to anna group
sudo usermod -aG anna $USER

# Re-login or activate group
newgrp anna

# Verify socket permissions
ls -l /run/anna/annad.sock
# Should show: srw-rw---- 1 root anna
```

### Quickscan Requires Sudo

This indicates you're not in the `anna` group or haven't re-logged since being added.

```bash
# Check group membership
id -nG | tr ' ' '\n' | grep anna

# If missing, add yourself
sudo usermod -aG anna $USER

# Re-login (or use newgrp)
newgrp anna

# Test
annactl quickscan
```

### Doctor Reports Issues

```bash
# Run full diagnostics
annactl doctor perms

# Common fixes:
# 1. Restart daemon
sudo systemctl restart annad

# 2. Check per-user directory
ls -la /var/lib/anna/users/$(id -u)

# 3. Verify policy file
cat /etc/anna/policy.d/$(id -u).toml

# 4. Re-run installer in repair mode
./scripts/install_anna.sh --repair
```

### Mode Confusion

If Anna is using the wrong mode:

```bash
# Check current mode
annactl status

# Force system mode
export ANNA_MODE=system

# Force user mode
export ANNA_MODE=user

# Or reinstall with explicit mode
./scripts/install_anna.sh --system
# or
./scripts/install_anna.sh --user
```

### Daemon Fails to Start

```bash
# View detailed logs
journalctl -u annad -b -n 200

# Check for port/socket conflicts
sudo lsof /run/anna/annad.sock

# Verify binary exists and is executable
ls -la /usr/local/sbin/annad

# Test daemon manually
sudo /usr/local/sbin/annad
```

## Environment Variables

- `ANNA_MODE`: Override mode detection (`system` or `user`)
- `NO_COLOR`: Disable colored output
- `ANNA_HEARTBEAT_SECS`: Daemon heartbeat interval (default: 60)

## Systemd Commands

```bash
# System mode
systemctl status annad
systemctl start annad
systemctl stop annad
systemctl restart annad
systemctl enable annad
systemctl disable annad
journalctl -u annad -f

# User mode
systemctl --user status annad
systemctl --user start annad
systemctl --user stop annad
systemctl --user restart annad
systemctl --user enable annad
systemctl --user disable annad
journalctl --user -u annad -f
```

## File Permissions Reference

### System Mode

| Path | Owner | Group | Mode | Purpose |
|------|-------|-------|------|---------|
| `/etc/anna` | root | root | 0755 | System config |
| `/etc/anna/policy.d` | root | root | 0755 | Policy directory |
| `/etc/anna/policy.d/<uid>.toml` | root | root | 0644 | User policy |
| `/var/lib/anna` | root | root | 0755 | Data root |
| `/var/lib/anna/users` | root | root | 0755 | User data root |
| `/var/lib/anna/users/<uid>` | root | anna | 2770 | Per-user data |
| `/run/anna` | root | root | 0755 | Runtime directory |
| `/run/anna/annad.sock` | root | anna | 0660 | Unix socket |

### User Mode

| Path | Owner | Group | Mode | Purpose |
|------|-------|-------|------|---------|
| `~/.anna/config` | user | user | 0755 | User config |
| `~/.anna/data` | user | user | 0755 | User data |
| `~/.anna/run` | user | user | 0755 | Runtime directory |
| `~/.anna/run/annad.sock` | user | user | 0660 | Unix socket |

## Migration

### From User Mode to System Mode

```bash
# 1. Backup user data (optional)
cp -r ~/.anna ~/anna_backup

# 2. Uninstall user mode
./scripts/uninstall_anna.sh --keep-data

# 3. Install system mode
./scripts/install_anna.sh --system

# 4. Verify
annactl status
```

### From System Mode to User Mode

```bash
# 1. Backup system data (optional)
sudo cp -r /var/lib/anna ~/anna_backup

# 2. Uninstall system mode
sudo ./scripts/uninstall_anna.sh

# 3. Install user mode
./scripts/install_anna.sh --user

# 4. Verify
annactl status
```

## Security Considerations

### System Mode

- Daemon runs as root to manage per-user directories
- Socket permissions (0660 root:anna) control access
- Users must be in `anna` group for read access
- Write operations (apply) flow through RPC with policy validation
- Dangerous operations always require confirmation

### User Mode

- No elevated privileges required
- Single-user only
- Suitable for development and testing
- Not recommended for production

## See Also

- [Architecture Documentation](../ARCH-SYSTEM-MODE.md) - Detailed technical design
- [CHANGES.txt](../CHANGES.txt) - Version history
- `annactl --help` - Command reference
- `annactl doctor perms` - Permission diagnostics
