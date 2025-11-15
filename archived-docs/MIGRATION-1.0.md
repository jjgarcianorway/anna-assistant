# Migration Guide: Anna 1.0 Reset

**From Beta/RC.11 to v1.0.0-rc.13.2**

This document outlines breaking changes and migration steps for upgrading to Anna 1.0.

---

## Overview

Anna 1.0 represents a complete architectural reset. The project has transitioned from a desktop environment manager to a **minimal, auditable system administration core**.

**Timeline:**
- **Before (Beta/RC.11)**: Desktop bundles, TUI, application recommendations
- **After (RC.13)**: State-aware daemon, health monitoring, recovery framework

---

## Breaking Changes

### 1. Removed Features

The following features have been **permanently removed** in 1.0:

#### Desktop Environment Management
- ❌ Hyprland bundle installation
- ❌ i3 bundle installation
- ❌ sway bundle installation
- ❌ Window manager configuration
- ❌ Dotfile management
- ❌ Pywal integration
- ❌ Hardware detection for DEs

**Rationale**: Anna is now a sysadmin core, not a desktop setup tool.

#### Commands Removed
```bash
# These commands no longer exist:
annactl setup hyprland     # Removed
annactl setup i3           # Removed
annactl setup sway         # Removed
annactl apply <bundle>     # Removed (use rescue plans)
annactl advise             # Removed
annactl revert <id>        # Replaced by rollback <plan>
```

#### TUI (Terminal User Interface)
- ❌ Interactive terminal UI
- ❌ `annactl tui` command
- ❌ Progress bars and spinners

**Status**: TUI will return in 2.0 as an optional interface.

#### Recommendation Engine
- ❌ Application suggestions
- ❌ Resource-aware filtering
- ❌ Context-aware recommendations
- ❌ Advice catalog system

**Rationale**: Replaced by health monitoring and recovery plans.

### 2. Changed Behavior

#### Command Structure
**Before (RC.11)**:
```bash
annactl advise             # Get recommendations
annactl apply vulkan-intel # Apply recommendation
annactl revert last        # Rollback last action
```

**After (RC.13)**:
```bash
annactl health             # Run health checks
annactl doctor             # Get diagnostics
annactl rescue list        # Show recovery plans
# (rescue run and rollback coming in 0.6b)
```

#### State Detection
**Before**: Simple installed/not-installed checks
**After**: Six-state machine (iso_live, recovery_candidate, post_install_minimal, configured, degraded, unknown)

Commands now adapt to system state automatically.

#### Logging
**Before**: Human-readable logs in ~/.local/share/anna/
**After**: JSONL format in /var/log/anna/ with citations

All logs include:
- ISO 8601 timestamps
- UUID request IDs
- Arch Wiki citations
- State at execution time

#### Permissions
**Before**: User-level operation
**After**: System-level with strict permissions

- Socket: `root:anna` 0660
- Reports: `root:root` 0600
- Directories: `root:root` 0700
- Users must be in `anna` group

---

## Migration Steps

### Step 1: Backup Your Data

```bash
# Backup old Anna configs
mkdir -p ~/anna-backup
cp -r ~/.config/anna ~/anna-backup/config
cp -r ~/.local/share/anna ~/anna-backup/data

# Backup any custom configurations
tar -czf ~/anna-backup/dotfiles.tar.gz ~/.config/{hyprland,i3,sway} 2>/dev/null || true
```

### Step 2: Uninstall Old Version

```bash
# Stop old daemon
sudo systemctl stop annad
sudo systemctl disable annad

# Run uninstall script (if available)
sudo ./scripts/uninstall.sh

# Or manual removal
sudo rm -f /usr/local/bin/{annad,annactl}
sudo rm -rf /usr/local/lib/anna
sudo rm -f /etc/systemd/system/annad.{service,socket}
sudo systemctl daemon-reload
```

### Step 3: Clean Old Data

```bash
# Remove old user configs
rm -rf ~/.config/anna
rm -rf ~/.local/share/anna

# Remove old system data
sudo rm -rf /var/lib/anna
sudo rm -rf /var/log/anna
```

### Step 4: Install RC.13

```bash
# From release
curl -sSL https://raw.githubusercontent.com/YOUR_ORG/anna-assistant/main/scripts/install.sh | sh

# Or from source
git clone https://github.com/YOUR_ORG/anna-assistant.git
cd anna-assistant
git checkout v1.0.0-rc.13
cargo build --release
sudo ./scripts/install.sh --local
```

The installer will:
1. Create `anna` system group
2. Install binaries to `/usr/local/bin/`
3. Copy assets to `/usr/local/lib/anna/`
4. Create directories with correct permissions
5. Install systemd units
6. Enable and start `annad.service`

### Step 5: Add Your User to anna Group

```bash
# Add yourself to anna group
sudo usermod -a -G anna $USER

# Log out and back in for group to take effect
# Or use newgrp for current session
newgrp anna
```

### Step 6: Verify Installation

```bash
# Check daemon status
sudo systemctl status annad

# Check socket permissions
ls -l /run/anna/anna.sock
# Should show: srw-rw---- 1 root anna 0 Nov 11 13:00 /run/anna/anna.sock

# Test commands
annactl status
annactl health
annactl doctor
annactl help
```

### Step 7: Verify Directories and Permissions

```bash
# Check directory structure
ls -la /var/lib/anna
ls -la /var/log/anna
ls -la /usr/local/lib/anna

# Permissions should be:
# /var/lib/anna/reports/     drwx------ root root
# /var/lib/anna/alerts/      drwx------ root root
# /var/log/anna/             drwx------ root root
# /run/anna/anna.sock        srw-rw---- root anna
```

---

## Feature Mapping

### Desktop Environment Setup

**Before (RC.11)**:
```bash
annactl setup hyprland
```

**After (RC.13)**:
❌ **Not available** - Anna no longer manages desktop environments.

**Alternative**: Use official Arch installation guides
- [archwiki:Hyprland]
- [archwiki:i3]
- [archwiki:Sway]

### Application Installation

**Before (RC.11)**:
```bash
annactl apply vulkan-intel
annactl apply docker-setup
```

**After (RC.13)**:
❌ **Not available** - Use pacman directly or recovery plans.

**Alternative**:
```bash
sudo pacman -S vulkan-intel
sudo pacman -S docker docker-compose
```

### System Recommendations

**Before (RC.11)**:
```bash
annactl advise
annactl advise --category=security
```

**After (RC.13)**:
Use health monitoring and diagnostics:
```bash
annactl health        # System health checks
annactl doctor        # Diagnostic report
annactl rescue list   # Available recovery plans
```

### Rollback Actions

**Before (RC.11)**:
```bash
annactl revert last
annactl revert vulkan-intel
```

**After (RC.13)**:
```bash
# Not yet available in rc.13
# Coming in Phase 0.6b:
annactl rollback <plan>
annactl rollback bootloader
```

### System Health

**Before (RC.11)**:
Limited health checking through recommendations.

**After (RC.13)**:
Comprehensive health monitoring:
```bash
annactl health              # Run all probes
annactl health --json       # Machine-readable output
annactl doctor              # Diagnostic synthesis
```

**Probes**:
- disk-space
- pacman-db
- systemd-units
- journal-errors
- services-failed
- firmware-microcode

---

## Configuration Changes

### File Locations

**Before (RC.11)**:
```
~/.config/anna/config.toml
~/.local/share/anna/advice_cache.json
~/.local/share/anna/history.jsonl
```

**After (RC.13)**:
```
/var/lib/anna/reports/health-*.json
/var/lib/anna/alerts/*.json
/var/log/anna/ctl.jsonl
/var/log/anna/health.jsonl
```

### Systemd Units

**Before (RC.11)**:
```ini
[Service]
User=root
# Basic permissions
```

**After (RC.13)**:
```ini
[Service]
User=root
# Strict sandboxing
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=yes
# ... (see README.md for full hardening)
```

### Socket Permissions

**Before (RC.11)**:
```
/run/anna/anna.sock  (0666 or 0600)
```

**After (RC.13)**:
```
/run/anna/anna.sock  (root:anna 0660)
```

Requires users to be in `anna` group.

---

## API Changes

### RPC Methods

**Removed Methods**:
- `GetAdvice` → Use `HealthRun`
- `ApplyAction` → Use recovery plans (Phase 0.6)
- `GetConfig` → Configuration now system-level

**New Methods**:
- `GetState` - System state detection
- `GetCapabilities` - Available commands for state
- `HealthRun` - Execute health probes
- `HealthSummary` - Last health check results
- `RecoveryPlans` - List recovery plans

### Response Format

**Before (RC.11)**:
```json
{
  "advice": [
    {
      "id": "vulkan-intel",
      "title": "Install Vulkan for Intel",
      "command": "pacman -S vulkan-intel"
    }
  ]
}
```

**After (RC.13)**:
```json
{
  "type": "HealthRun",
  "data": {
    "state": "configured",
    "summary": {"ok": 5, "warn": 1, "fail": 0},
    "results": [...],
    "citation": "[archwiki:System_maintenance]"
  }
}
```

All responses now include:
- State at execution time
- Arch Wiki citations
- ISO 8601 timestamps
- Duration in milliseconds

---

## Exit Code Changes

### New Exit Codes

Anna now uses standard exit codes:

| Code | Meaning | Example |
|------|---------|---------|
| 0 | Success | Health check passed |
| 1 | Failure | One or more probes failed |
| 2 | Warning | Warnings but no failures |
| 64 | Command not available | Command invalid for current state |
| 65 | Invalid response | Daemon returned bad data |
| 70 | Daemon unavailable | Cannot connect to socket |

**Scripts should check specific exit codes** rather than just success/fail.

---

## Troubleshooting

### "Permission denied" on /run/anna/anna.sock

**Solution**: Add user to anna group
```bash
sudo usermod -a -G anna $USER
newgrp anna  # Or log out and back in
```

### "Command not available in current state"

**Solution**: Check current state and available commands
```bash
annactl status
annactl help
```

Commands are state-aware. Some are only available in specific states.

### "Daemon unavailable" (exit 70)

**Solutions**:
```bash
# Check daemon status
sudo systemctl status annad

# Check socket exists
ls -l /run/anna/anna.sock

# Restart daemon
sudo systemctl restart annad

# Check logs
sudo journalctl -u annad -n 50
```

### Missing reports directory

**Solution**: Directories are created automatically, but permissions must be correct
```bash
sudo mkdir -p /var/lib/anna/{reports,alerts}
sudo chmod 700 /var/lib/anna/{reports,alerts}
sudo mkdir -p /var/log/anna
sudo chmod 700 /var/log/anna
```

### Old desktop configs not working

**Expected**: Desktop environment management was removed.

**Solution**: Manually manage your desktop environment using Arch Wiki guides:
- [archwiki:Hyprland]
- [archwiki:i3]
- [archwiki:Sway]

---

## What to Expect Next

### Phase 0.6 (In Progress)
- Executable recovery plans
- `annactl rescue run <plan>`
- `annactl rollback <plan>`
- Interactive rescue mode
- Rollback script generation

### Phase 0.7-0.9 (Future)
- State-aware update system
- Backup automation
- Installation wizard

### Version 2.0 (Future)
- TUI returns as optional interface
- Additional recovery plans
- Advanced diagnostics

---

## FAQ

### Q: Will desktop environment setup return?
**A**: No. Anna 1.0+ focuses on system administration, not desktop management. Use official Arch guides for DE setup.

### Q: Can I still use the TUI?
**A**: The TUI was removed in 1.0. It will return as an optional interface in 2.0.

### Q: How do I get application recommendations now?
**A**: Anna no longer provides application recommendations. Use the Arch Wiki and pacman directly.

### Q: What happened to my old advice history?
**A**: Old data is incompatible with 1.0. Back it up before migrating if needed, but it cannot be imported.

### Q: Is rc.13.2 stable enough for production?
**A**: rc.13.2 has comprehensive test coverage, CI validation, and security hardening. It's suitable for testing and early adoption. Version 1.0 stable is planned for production use.

### Q: Can I downgrade to RC.11?
**A**: Yes, but you must manually uninstall rc.13.2 first and reinstall from the old branch. Note that RC.11 is no longer maintained.

### Q: What happened to rc.13 and rc.13.1?
**A**: rc.13 and rc.13.1 had daemon startup issues (socket access and systemd unit problems). Always use rc.13.2 or later.

---

## Support

If you encounter issues during migration:

1. **Check logs**:
   ```bash
   sudo journalctl -u annad -n 100
   cat /var/log/anna/ctl.jsonl | tail -10
   ```

2. **Verify permissions**:
   ```bash
   ls -la /var/lib/anna
   ls -la /var/log/anna
   ls -l /run/anna/anna.sock
   groups $USER  # Should include 'anna'
   ```

3. **Report issues**: https://github.com/YOUR_ORG/anna-assistant/issues

---

## Conclusion

Anna 1.0 is a complete reset focused on what matters: **reliable, auditable system administration**.

While this means removing features that don't align with this mission, it results in a more focused, secure, and maintainable codebase.

Thank you for your patience during this transition.

---

**Anna Assistant v1.0.0-rc.13.2**

*One daemon, one socket, one truth*
