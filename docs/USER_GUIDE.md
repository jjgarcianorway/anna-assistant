# Anna Assistant User Guide

**Version**: 4.4.0-beta.1
**Focus**: System Caretaker - Real Problem Detection and Repair
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

## System Intelligence and Detection

Anna's caretaker brain continuously monitors your system across 9 major categories. This section explains what Anna detects, why it matters, and how to interpret the findings.

### Detection Categories

#### 1. Disk Space Analysis

Anna monitors root filesystem usage and identifies space consumers:

```bash
$ annactl status

📊 Detailed Analysis:

🔴 Critical Issues:

  • Disk 96% full - system at risk
    Your disk is critically full. This can cause system instability and data loss. Immediate action required.
    💡 Run 'sudo annactl repair' to clean up space
    📚 https://wiki.archlinux.org/title/System_maintenance#Clean_the_filesystem
```

**Severity Levels:**
- **Critical (>95% full)**: System at risk, operations may fail
- **Warning (>90% full)**: Action needed soon
- **Info (>80% full)**: Consider cleanup

**What Anna Checks:**
- Root filesystem usage via `df /`
- Package cache size (`/var/cache/pacman/pkg`)
- Log directory size (`/var/log`)
- Common large directories

**Repair Actions:**
- `paccache -rk1`: Keeps only latest package version (safest, most effective)
- `journalctl --vacuum-size=100M`: Reduces journal size
- Guidance on manual cleanup for user directories

#### 2. Failed Systemd Services

Detects services in failed or degraded state:

```bash
🔴 Critical Issues:

  • Failed systemd services detected
    Some system services are not running properly.
    💡 Run 'sudo annactl repair services-failed' to restart failed services
```

**What Anna Checks:**
- Runs `systemctl list-units --failed`
- Identifies which services failed and why
- Checks service dependencies

**Repair Actions:**
- Attempts `systemctl restart` for failed services
- Reports success/failure for each service
- Provides guidance if restart fails

#### 3. Pacman Database Health

Detects stale pacman lock files that prevent package operations:

```bash
⚠️ Warnings:

  • Stale pacman lock file detected
    Pacman database is locked but appears to be stale. This prevents package operations.
    💡 Run 'sudo rm /var/lib/pacman/db.lck' to remove the lock
    📚 https://wiki.archlinux.org/title/Pacman#Failed_to_init_transaction
```

**What Anna Checks:**
- Existence of `/var/lib/pacman/db.lck`
- Lock file age (stale if >1 hour old)
- No active pacman process

**Repair Actions:**
- Safely removes stale lock file after verification
- Allows package operations to proceed

#### 4. Laptop Power Management

Auto-detects laptops and checks power management configuration:

```bash
⚠️ Warnings:

  • Laptop detected but TLP not enabled
    TLP is installed but not enabled. Your battery life could be significantly better.
    💡 Run 'sudo systemctl enable --now tlp.service'
    📚 https://wiki.archlinux.org/title/TLP
```

**What Anna Checks:**
- Battery presence in `/sys/class/power_supply/BAT0` or `BAT1`
- TLP installation via `which tlp`
- TLP service status via `systemctl is-enabled tlp.service`

**Repair Actions:**
- Enables and starts TLP service
- Immediate battery life improvement

#### 5. GPU Driver Status

Detects GPUs and checks if proper drivers are loaded:

```bash
⚠️ Warnings:

  • NVIDIA GPU detected but driver not loaded
    You have an NVIDIA GPU but the proprietary driver is not loaded. GPU acceleration won't work.
    💡 Install NVIDIA driver: 'sudo pacman -S nvidia nvidia-utils'
    📚 https://wiki.archlinux.org/title/NVIDIA
```

**What Anna Checks:**
- NVIDIA GPU presence via `lspci`
- Driver module loaded via `lsmod | grep nvidia`

**Repair Actions:**
- Provides package installation guidance
- User confirms driver installation (requires reboot)

#### 6. Journal Error Volume (NEW in 4.4)

Monitors system journal for high error rates:

```bash
🔴 Critical Issues:

  • High journal error volume (237 errors)
    Your system journal has an unusually high number of errors this boot. This indicates serious system issues that need investigation.
    💡 Review errors with 'journalctl -p err -b' and investigate the most frequent issues
    📊 237 errors need investigation
    📚 https://wiki.archlinux.org/title/Systemd/Journal
```

**What Anna Checks:**
- Error-level entries in current boot via `journalctl -p err -b`
- Counts non-empty lines

**Severity Levels:**
- **Critical (>200 errors)**: Serious system issues
- **Warning (>50 errors)**: Configuration or hardware problems

**Repair Actions:**
- Vacuums old journal entries (last 7 days) with `journalctl --vacuum-time=7d`
- Preserves recent logs for debugging
- Reduces journal disk usage

**Troubleshooting Tips:**
- Review errors: `journalctl -p err -b | less`
- Find most common: `journalctl -p err -b | awk '{print $5}' | sort | uniq -c | sort -rn | head`
- Address root cause before vacuuming

#### 7. Zombie Process Detection (NEW in 4.4)

Scans for defunct processes accumulating on the system:

```bash
ℹ️ Recommendations:

  • 3 zombie process(es) detected (bash, python, systemd)
    Zombie processes are harmless but may indicate improper process management.
    💡 Use 'ps aux | grep Z' to identify zombies and check their parent processes
    📚 https://wiki.archlinux.org/title/Core_utilities#Process_management
```

**What Anna Checks:**
- Scans `/proc/*/status` for `State: Z`
- Extracts process names when available
- Counts total zombie processes

**Severity Levels:**
- **Warning (>10 zombies)**: Parent process issue
- **Info (>0 zombies)**: Minor issue, usually harmless

**Important Notes:**
- Zombies can't be killed directly
- Parent process must reap them
- Usually indicates application bug
- Zero resource consumption

**Troubleshooting:**
```bash
# Find zombies and their parents
$ ps -eo pid,ppid,stat,comm | grep Z

# Check parent process
$ ps -p <PPID> -o pid,comm,cmd

# If parent is alive, it should reap the zombie
# If parent is dead, init should reap it
```

#### 8. Orphaned Package Detection (NEW in 4.4)

Finds packages no longer required by any installed package:

```bash
⚠️ Warnings:

  • 63 orphaned packages found
    Many packages are installed as dependencies but no longer required by any package. These consume disk space unnecessarily.
    💡 Remove with 'sudo pacman -Rns $(pacman -Qtdq)' after reviewing the list
    📚 https://wiki.archlinux.org/title/Pacman/Tips_and_tricks#Removing_unused_packages_(orphans)
```

**What Anna Checks:**
- Runs `pacman -Qtdq` to list orphaned packages
- Counts total orphans

**Severity Levels:**
- **Warning (>50 orphans)**: Significant disk waste
- **Info (>10 orphans)**: Cleanup recommended

**Repair Actions:**
- Lists orphaned packages
- Removes with `pacman -Rns` after user confirmation
- Frees disk space from unused dependencies

**Safe Review Process:**
```bash
# List orphaned packages
$ pacman -Qtd

# See what would be removed
$ pacman -Rns $(pacman -Qtdq) --print

# Actually remove (via Anna)
$ sudo annactl repair orphaned-packages
```

#### 9. Core Dump Accumulation (NEW in 4.4)

Monitors crash dumps consuming disk space:

```bash
ℹ️ Recommendations:

  • 15 core dumps found (342 MB)
    8 dumps are older than 30 days and can likely be removed.
    💡 Review with 'coredumpctl list' and clean old dumps with 'sudo coredumpctl vacuum --keep-free=1G'
    📚 https://wiki.archlinux.org/title/Core_dump
```

**What Anna Checks:**
- Scans `/var/lib/systemd/coredump` for dump files
- Calculates total size in MB
- Identifies dumps older than 30 days

**Severity Levels:**
- **Warning (>1GB)**: Significant space consumed
- **Info (>10 files, >5 old)**: Cleanup recommended

**Repair Actions:**
- Uses `coredumpctl vacuum --keep-free=1G` to clean old dumps
- Preserves recent dumps for debugging
- Gracefully handles missing coredumpctl

**Troubleshooting:**
```bash
# List all core dumps
$ coredumpctl list

# Show info about specific dump
$ coredumpctl info <PID>

# Extract specific dump
$ coredumpctl dump <PID> -o core.dump

# Clean old dumps (via Anna)
$ sudo annactl repair core-dump-cleanup
```

### Real-World Scenarios

#### Scenario 1: Disk Nearly Full

```bash
$ annactl daily

🔴 First System Scan - 2025-11-13 13:58

2 critical, 2 warnings detected

Health: 4 ok, 1 warnings, 1 failures
Disk: 96.5% used (24.7GB / 802.1GB total)

🔍 Issues Detected (prioritized):

1. 🔴 Disk 96% full - system at risk
2. 🔴 disk-space issue
3. ⚠️ TLP not properly configured
4. ⚠️ 63 orphaned packages found

# Fix it all at once
$ sudo annactl repair

# Or fix specific issues
$ sudo annactl repair disk-space
$ sudo annactl repair orphaned-packages
```

#### Scenario 2: System Logging Errors

```bash
$ annactl status

🔴 Critical Issues:

  • High journal error volume (237 errors)

# Investigate first
$ journalctl -p err -b | less

# Find patterns
$ journalctl -p err -b | grep -oP '(?<=: ).*' | sort | uniq -c | sort -rn | head -10

# Clean up after fixing root cause
$ sudo annactl repair journal-cleanup
```

#### Scenario 3: First Run on New Machine

```bash
$ annactl daily

👋 Welcome to Anna!

Looks like this is the first time I see this machine.
I will run a deeper scan once and then remember the results.

Running first system scan...

# Anna checks ALL 9 categories and prioritizes findings
# You get immediate visibility into system health
# Fix critical issues first, then warnings, then info
```

### Detection Summary

| Category | Check Frequency | Severity Levels | Auto-Repair |
|----------|----------------|-----------------|-------------|
| Disk Space | Every run | Critical/Warning/Info | Yes |
| Failed Services | Every run | Critical | Yes |
| Pacman Locks | Every run | Warning | Yes |
| Laptop Power | First run + daily | Warning/Info | Yes |
| GPU Drivers | First run + daily | Warning | No (guidance) |
| Journal Errors | Every run | Critical/Warning | Yes |
| Zombie Processes | Every run | Warning/Info | No (guidance) |
| Orphaned Packages | Every run | Warning/Info | Yes |
| Core Dumps | Every run | Warning/Info | Yes |

**Key Principles:**
- All detectors fail gracefully if commands unavailable
- All issues include Arch Wiki references
- Repair actions require explicit user confirmation
- Critical issues are always prioritized
- First run performs deep scan, subsequent runs are fast (~2s)

---

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

## Upgrading Anna

Anna supports automatic upgrades that respect your package manager.

### AUR Installations (Recommended)

If installed via AUR (yay/paru):

```bash
# Update Anna along with your system
yay -Syu

# Or update Anna specifically
yay -S anna-assistant-bin
```

**Why AUR?** Package managers track dependencies, maintain file ownership, and provide rollback via pacman.

### Manual Installations

If installed manually (GitHub release or curl):

```bash
# Check for updates
annactl upgrade --check

# Interactive upgrade (with confirmation)
sudo annactl upgrade

# Automated upgrade (no confirmation)
sudo annactl upgrade --yes
```

**What happens during upgrade:**
1. ✅ Detects installation source (blocks if AUR-managed)
2. 🌐 Checks GitHub for latest release
3. 📥 Downloads binaries and SHA256SUMS
4. 🔐 Verifies checksums before installation
5. 💾 Backs up current version to `/var/lib/anna/backup/`
6. 📦 Replaces binaries with new version
7. 🔄 Restarts daemon automatically

### Rollback

If an upgrade fails:

```bash
sudo annactl rollback
# Restores from /var/lib/anna/backup/
```

### Automatic Background Checks

The daemon checks for updates every 24 hours (manual installations only):

```bash
# Check last update check time
cat /var/lib/anna/last_update_check

# View update check logs
journalctl -u annad | grep -i update
```

**Note**: Background checks only log availability. You must run `sudo annactl upgrade` to install updates.

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

## A Typical Day with Anna

This is how Anna works in real daily use - a simple, reliable workflow for keeping your Arch system healthy.

### Morning Routine (2 minutes)

Start your day with a quick system check:

```bash
$ annactl daily
═══════════════════════════════════════════════════════════
📋 DAILY CHECKUP
═══════════════════════════════════════════════════════════
Health: ✅ (5 ok, 0 warn, 0 fail)

Report: /var/lib/anna/reports/daily-20251113-080000.json
Next: annactl status (for detailed view)
```

**What this does:**
- Checks disk space, pacman database, systemd services
- Scans journal for errors in the last 24 hours
- Shows top 3 predictions if any issues are brewing
- Saves a report you can track over time

**Time cost**: ~2 seconds

### When Issues Appear

If `annactl daily` shows warnings or failures:

```bash
$ annactl daily
═══════════════════════════════════════════════════════════
📋 DAILY CHECKUP
═══════════════════════════════════════════════════════════
Health: ⚠️  (4 ok, 1 warn, 0 fail)
  ⚠️  pacman-db: Database lock file exists

Next: annactl repair (to fix issues)
```

Run repair to fix safe issues automatically:

```bash
$ annactl repair
═══════════════════════════════════════════════════════════
🔧 SYSTEM REPAIR
═══════════════════════════════════════════════════════════

Anna will attempt to fix detected issues automatically.
Only low-risk actions will be performed.

⚠️  Actions may modify system state!

Proceed with repair? [y/N]: y

🔧 EXECUTING REPAIRS

✅ pacman-db
  Action: Remove stale lock file
  Details: Removed /var/lib/pacman/db.lck
  Source: [archwiki:pacman#Database_locked]

────────────────────────────────────────────────────────────
Summary: 1 succeeded, 0 failed
Reference: [archwiki:System_maintenance]
```

**What repair does:**
- Only fixes low-risk issues (no package updates, no config changes)
- Always asks for confirmation
- Shows what it's doing and why (with Arch Wiki citations)
- Reports success/failure clearly

**Time cost**: ~5 seconds + your confirmation

### Weekly Maintenance (5 minutes)

Once a week, run a deeper check:

```bash
# 1. Full health check
$ annactl health

# 2. Check for system updates (safe - no changes)
$ annactl update --dry-run

# 3. Review what Anna learned
$ annactl profile
```

**That's it.** Three commands, 5 minutes, once a week.

### When You're Away

Anna runs in the background (as `annad` daemon) and:
- Monitors system metrics continuously
- Builds predictions about potential issues
- Checks for Anna updates once per day (manual installations only)
- **Never makes changes without your explicit permission**

You can check what it's been doing:

```bash
# View recent command history
$ tail -20 /var/log/anna/ctl.jsonl | jq -r '.command'

# Check daemon status
$ annactl status
```

### The Core Philosophy

Anna follows a simple rule: **Observe by default, act only when you ask.**

- **Read-only commands** (`daily`, `status`, `health`): Run anytime, zero risk
- **Safe repairs** (`repair`): Always ask first, only low-risk actions
- **System changes** (`update`): You must explicitly run these

This means you're always in control. Anna won't surprise you with automatic updates, reboots, or configuration changes.

### Real-World Example

Here's what a typical Monday morning looks like:

```bash
# Monday, 8:00 AM - Check system health
$ annactl daily
Health: ✅ All good

# Wednesday, 8:00 AM - Notice a warning
$ annactl daily
Health: ⚠️  (4 ok, 1 warn, 0 fail)
  ⚠️  disk-space: /var partition 85% full

# Run repair - but disk space needs manual attention
$ annactl repair
No automatic repairs available for disk-space issues.
Recommendation: Review /var/log and /var/cache for cleanup

# Manual cleanup
$ sudo journalctl --vacuum-time=7d
$ sudo paccache -r

# Verify fixed
$ annactl daily
Health: ✅ All good
```

### For Advanced Users

Once comfortable with the basics:

- **Monitoring stack**: `annactl monitor install` (Grafana + Prometheus)
- **Auto-upgrades**: `sudo annactl upgrade` (manual installations only)
- **Predictions**: `annactl predict` (see what Anna thinks might happen)
- **Learning**: `annactl learn` (view Anna's system knowledge)

But you don't need any of this for daily use. The core workflow - `daily`, `repair`, and occasional `health` - is all most users ever need.

---

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

1. **Practice the daily routine**: Make `annactl daily` part of your morning
2. **Review health reports**: Study `annactl health` output when warnings appear
3. **Understand your system**: Read `annactl profile` results
4. **Explore monitoring** (optional): `annactl monitor install` for Grafana dashboards

## Version History

- **4.0.0-beta.1**: Core Caretaker Workflows - First beta release
- **3.10.0-alpha.1**: AUR-Aware Auto-Upgrade System
- **3.8.0-alpha.1**: Adaptive CLI with progressive disclosure
- **3.7.0**: Predictive intelligence and learning
- **3.6.0**: Persistent context layer
- **3.1.0**: Command classification system

---

**Need Help?**
Run `annactl help` or open an issue on GitHub.

**License**: Custom (see LICENSE file)
**Citation**: [archwiki:system_maintenance]
