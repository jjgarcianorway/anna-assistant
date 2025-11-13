# Anna Assistant User Guide

**Version**: 3.8.0-alpha.1
**Phase**: Adaptive CLI with Progressive Disclosure
**Audience**: End users, system administrators

---

## Welcome to Anna

Anna is an intelligent Arch Linux administration assistant that helps you maintain, update, and monitor your system with confidence. This guide covers everything you need to know to use Anna effectively.

## 🚀 3-Minute Quickstart

New to Anna? Get up and running in 3 minutes:

### Step 1: Install (30 seconds)

```bash
# Via AUR (recommended for Arch Linux)
yay -S anna-assistant-bin

# Or download binary from GitHub releases
# https://github.com/jjgarcianorway/anna-assistant/releases/latest
```

### Step 2: Initialize (1 minute)

```bash
# Initialize configuration (creates /etc/anna)
sudo annactl init

# Enable and start the daemon
sudo systemctl enable --now annad

# Add your user to anna group
sudo usermod -aG anna $USER
newgrp anna
```

### Step 3: Your First Commands (1.5 minutes)

```bash
# Check system status
annactl status

# Run health diagnostics
annactl health

# See what Anna learned about your system
annactl profile

# Get help anytime
annactl help
```

**That's it!** 🎉 Anna is now monitoring your system and ready to help.

### What's Next?

- **Learn the safest commands**: `help`, `status`, `health`, `profile` (read-only, always safe)
- **Explore predictive intelligence**: `annactl learn` and `annactl predict`
- **Install monitoring** (optional): `annactl monitor install`
- **Read the full guide below** for advanced features

---

## Full Quick Start Guide

### Installation

```bash
# Via AUR (recommended)
yay -S anna-assistant-bin

# Or manual installation
# Download from GitHub releases
wget https://github.com/jjgarcianorway/anna-assistant/releases/latest/download/annactl
sudo mv annactl /usr/local/bin/
sudo chmod +x /usr/local/bin/annactl
```

### First Steps

```bash
# 1. Start the Anna daemon
sudo systemctl start annad

# 2. Add your user to the 'anna' group
sudo usermod -aG anna $USER
newgrp anna

# 3. Check system status
annactl status

# 4. Run health check
annactl health

# 5. Get help
annactl --help
```

## Understanding Anna's Interface

### Adaptive Help System

Anna shows you only the commands that are relevant to your current situation:

```bash
# Default help - shows commands for your context
$ annactl --help

Anna Assistant - Adaptive Arch Linux Administration
============================================================

Mode: Normal User

Safe Commands (1 available)
  help            Show available commands and help

Run with --all to see all commands
```

**Want to see everything?**

```bash
# Show all available commands
$ annactl --help --all

Safe Commands (3 available)
  help            Show available commands
  status          Show system status
  health          Check system health

Advanced Commands (6 available)
  update          Update system packages
  install         Install Arch Linux
  doctor          Diagnose and fix issues
  ...
```

### Understanding Command Categories

#### 🟢 Safe Commands (Green)
- **What**: Read-only information commands
- **When**: Use anytime to check system state
- **Risk**: None - these commands never modify your system
- **Examples**: `status`, `health`, `ping`

#### 🟡 Advanced Commands (Yellow)
- **What**: System administration operations
- **When**: Use when you need to make changes
- **Risk**: Medium - requires understanding of system administration
- **Examples**: `update`, `doctor`, `rollback`

#### 🔴 Internal Commands (Red)
- **What**: Developer and diagnostic tools
- **When**: Use only if you know Anna's internals
- **Risk**: Low but requires technical knowledge
- **Examples**: `sentinel`, `config`, `conscience`

## Common Tasks

### Checking System Status

```bash
# Quick status overview
$ annactl status

┌─────────────────────────────────────────────────────────
│ SYSTEM HEALTH REPORT
├─────────────────────────────────────────────────────────
│ Status:    HEALTHY
│ Timestamp: 2025-11-12T10:30:00Z
│ State:     configured
├─────────────────────────────────────────────────────────
│ All critical services: OK
│ System is up to date
└─────────────────────────────────────────────────────────
```

### Running Health Checks

```bash
# Comprehensive health check
$ annactl health

Health Status: PASS (6/6 probes passed)
Total Runtime: 1.2s

✅ disk-space: PASS (5.2s)
   Root partition: 45% used (120GB free)

✅ pacman-db: PASS (0.3s)
   Package database is valid

✅ systemd-units: PASS (0.8s)
   All critical units active

Details saved: /var/log/anna/health/2025-11-12-103000.json
```

### Updating Your System

```bash
# Check what updates are available (safe, no changes)
$ annactl update --dry-run

System Update (Dry Run)
─────────────────────────────────────────────────────
Total packages: 23 (156.4 MB download)

Packages to upgrade:
  linux            → 6.7.2-1 (124 MB)
  systemd          → 255.1-1 (12 MB)
  firefox          → 122.0-1 (8 MB)
  ...

# Actually perform the update (requires root)
$ sudo annactl update
```

### Getting Help for Specific Commands

```bash
# Detailed help for any command
$ annactl help status

Command: status
Category: User-Safe (🟢)
Risk Level: None

Description:
  Show comprehensive system health report including service
  status, available updates, and recent log issues.

Prerequisites:
  • Anna daemon must be running
  • User must be in 'anna' group

Examples:
  annactl status              # Show current status

Usage:
  annactl status
```

## Advanced Features

### JSON Output Mode

For scripting and automation:

```bash
# Get help in JSON format
$ annactl --help --json
{
  "context": "User",
  "commands": [
    {
      "name": "help",
      "category": "UserSafe",
      "risk": "None",
      "description": "Show available commands and help",
      "requires_root": false,
      "requires_daemon": false
    }
  ],
  "total": 1
}

# JSON output for health
$ annactl health --json > health-report.json
```

### Predictive Intelligence

Anna learns from your system's behavior and provides proactive suggestions:

```bash
$ annactl status

[... status output ...]

┌─────────────────────────────────────────────────────────
│ PREDICTIVE INTELLIGENCE
├─────────────────────────────────────────────────────────
│ 🔴 ServiceFailure · postgresql service may fail soon
│   → Run 'annactl doctor' to diagnose
│ ⚠️  MaintenanceWindow · System updates recommended
│   → Run 'annactl update --dry-run' to preview
└─────────────────────────────────────────────────────────
```

**Note**: Predictions appear only once per 24 hours to avoid alert fatigue.

### Monitoring Stack

Install Grafana and Prometheus for advanced monitoring:

```bash
# Check if your system can run monitoring
$ annactl profile

System Profile: Workstation (High-End)
─────────────────────────────────────────────────────
CPU: 8 cores
RAM: 16 GB
Recommended monitoring mode: Full

# Install monitoring stack (requires root)
$ sudo annactl monitor install

🚀 Installing Full Monitoring Stack
─────────────────────────────────────────────────────
[✓] Prometheus installed
[✓] Grafana installed
[✓] Node Exporter installed
[✓] Services started

Grafana: http://localhost:3000
  Username: admin
  Password: (check /etc/grafana/grafana.ini)
```

## Troubleshooting

### Permission Denied Errors

If you see:

```
❌ Permission denied accessing Anna daemon socket.

Socket path: /run/annad/control.sock

Your user account needs to be added to the 'anna' group.

Fix (run these commands):

1. Add your user to the 'anna' group:
   sudo usermod -aG anna YOUR_USERNAME

2. Apply the group change (choose one):
   newgrp anna              # Apply immediately (current shell)
   # OR logout and login     # Apply permanently

3. Verify the fix:
   groups | grep anna       # Should show 'anna' in output
   annactl status           # Should work now
```

### Daemon Not Running

```bash
# Check daemon status
$ sudo systemctl status annad

# Start daemon
$ sudo systemctl start annad

# Enable on boot
$ sudo systemctl enable annad

# View daemon logs
$ sudo journalctl -u annad -f
```

### Health/Doctor Commands Fail (v3.9.0-alpha.1 Known Issue)

**Fixed in v3.9.1-alpha.1**: Report directory permissions

If `annactl health` or `annactl doctor` fail with "Permission denied" on v3.9.0-alpha.1:

```bash
# Quick fix: Create reports directory with correct permissions
sudo install -d -o root -g anna -m 0770 /var/lib/anna/reports
sudo chmod 0750 /var/lib/anna /var/log/anna
sudo setfacl -d -m g:anna:rwx /var/lib/anna/reports

# Verify
annactl health
# Should now work and show: "Details saved: /var/lib/anna/reports/health-*.json"
```

**Why this happened**: v3.9.0-alpha.1 created directories as 0700 (root-only), preventing anna group members from writing reports.

**Permanent fix**: Upgrade to v3.9.1-alpha.1 or later, which includes:
- Systemd unit with correct directory modes
- CLI fallback to `~/.local/state/anna/reports` if primary path not writable
- tmpfiles.d configuration for permission self-healing

### Self-Update Issues

If Anna was installed via package manager:

```bash
$ annactl self-update --check
⚠️  Anna was installed via package manager: anna-assistant-bin

Please use your package manager to update:
  pacman -Syu              # System update (includes Anna)
  yay -Sua                 # AUR update only
```

### Getting Detailed Help

```bash
# Show all commands including internal
$ annactl --help --all

# Disable color output (for scripts or accessibility)
$ NO_COLOR=1 annactl --help

# Check Anna version
$ annactl --version
```

## Best Practices

### Regular Maintenance

```bash
# Weekly routine (5 minutes)
1. annactl health          # Check system health
2. annactl update --dry-run # Preview updates
3. annactl status          # Verify everything is OK

# Before important work
1. annactl health          # Ensure system is healthy
2. annactl backup          # Create backup (if available)
```

### Safe Experimentation

```bash
# Always use --dry-run first
$ annactl update --dry-run       # See what would happen
$ annactl update                 # Actually do it

# Check help before running unfamiliar commands
$ annactl help doctor            # Understand what it does
$ annactl doctor                 # Run it
```

### Understanding System State

Anna is aware of your system's current state:

- **Healthy**: Everything is working normally
- **Degraded**: Some issues detected, but system functional
- **Critical**: Major issues requiring immediate attention
- **ISO Live**: Running from installation media
- **Unknown**: Daemon unavailable or state unclear

Commands adapt based on state to show most relevant options.

## Environment Variables

```bash
# Disable color output
export NO_COLOR=1

# Use custom socket path
export ANNACTL_SOCKET=/custom/path/control.sock

# Force developer mode (shows all commands)
export ANNACTL_DEV_MODE=1
```

## Getting More Help

- **Documentation**: `/usr/share/doc/anna-assistant/`
- **GitHub Issues**: https://github.com/jjgarcianorway/anna-assistant/issues
- **Command Help**: `annactl help <command>`
- **Logs**: `/var/log/anna/`

## Command Quick Reference

### Information Commands (Always Safe)
```bash
annactl status              # System overview
annactl health              # Health check
annactl ping                # Test daemon
annactl profile             # System profile
annactl metrics             # System metrics
annactl --help              # Show available commands
annactl --version           # Show version
```

### Administration Commands (Require Root)
```bash
sudo annactl update              # Update system
sudo annactl doctor              # Diagnose and fix
sudo annactl monitor install     # Install monitoring
sudo annactl backup              # Create backup
sudo annactl rollback            # Revert changes
```

### Diagnostic Commands
```bash
annactl triage                   # Get recommendations
annactl collect-logs            # Gather diagnostic info
annactl audit                   # Show audit log
```

## Tips & Tricks

1. **Tab Completion**: If available, use shell completion for commands
2. **JSON Mode**: Use `--json` flags for scripting and automation
3. **Dry Run**: Always test with `--dry-run` before system changes
4. **Help Text**: Read command help before first use
5. **Logs**: Check `/var/log/anna/ctl.jsonl` for command history

## Keyboard Shortcuts

When viewing long output:
- `Ctrl+C`: Stop current command
- `Ctrl+Z`: Suspend command (use `fg` to resume)
- `q`: Quit pager (if output is piped to `less`)

---

## What's Next?

After getting comfortable with basic commands:

1. **Explore monitoring**: `annactl monitor install`
2. **Review health reports**: Study `annactl health` output
3. **Understand your system**: Read `annactl profile` results
4. **Learn admin commands**: Use `annactl help --all` to discover more

## Version History

- **3.8.0-alpha.1**: Adaptive CLI with progressive disclosure
- **3.7.0**: Predictive intelligence and learning
- **3.6.0**: Persistent context layer
- **3.1.0**: Command classification system

---

**Need Help?**
Run `annactl help` or open an issue on GitHub.

**License**: Custom (see LICENSE file)
**Citation**: [archwiki:system_maintenance]
