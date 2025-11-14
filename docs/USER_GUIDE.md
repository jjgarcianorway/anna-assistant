# Anna Assistant User Guide

**Version**: 5.2.0-beta.1
**Focus**: Storage & Network Reliability (Don't Lose Data, Don't Break Network)
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

---

## Machine Profiles and Adaptive Behavior

### Profile Detection (Phase 4.6)

Anna automatically detects what kind of machine she's running on and adapts her behavior accordingly:

**Laptop Profile**
- Detected by: Battery presence, AC adapter, laptop-specific hardware
- Focus areas: Battery life, power management (TLP), suspend/resume, mobile security (firewall)
- Behavior: More insistent about TLP, watchful of battery-draining issues

**Desktop Profile**
- Detected by: No battery, desktop-class components, gaming peripherals
- Focus areas: GPU drivers, graphics performance, moderate desktop concerns
- Behavior: Balanced approach between server and laptop

**Server-Like Profile**
- Detected by: No display server, no graphical packages, server-oriented services
- Focus areas: Core system health, services, disk space, updates
- Behavior: Quiet about desktop-specific concerns (GPU, TLP, etc.)

You can see your detected profile in command output:
```bash
$ annactl daily
Daily System Check (Laptop) - 2025-01-15 09:30
...

$ annactl status
System Status (Server-Like) - healthy
...
```

### Noise Control (Phase 4.7)

Anna learns from your behavior and adapts over time to avoid nagging:

**How It Works**
- **First 2-3 times**: Low-priority suggestions (Info hints) are shown in `annactl daily`
- **After that**: If you ignore a suggestion, it fades into the background
- **Critical issues**: Always shown, never de-emphasized
- **Full visibility**: All issues remain visible in `annactl status`

**Example: Time Sync Suggestion**

```bash
# First time
$ annactl daily
Daily System Check (Desktop) - 2025-01-15 09:30
...
ℹ️  Time synchronization not enabled
    systemd-timesyncd is available but not enabled...
    💡 Run 'sudo systemctl enable --now systemd-timesyncd.service'

# After showing 2-3 times without being fixed
$ annactl daily
Daily System Check (Desktop) - 2025-01-15 09:35
...
✅ System is stable!
💡 1 low-priority hint hidden (see 'annactl status' for details)

# Still available in status
$ annactl status
System Status (Desktop) - healthy
...
ℹ️  Recommendations:
  • Time synchronization not enabled
    [full details shown]
```

**Benefits**
- Calm, predictable experience - no nagging
- Critical issues always get attention
- You stay aware of long-term recommendations
- Nothing is hidden - just de-emphasized in daily view

### User Control (Phase 4.9)

**The Decision Layer** - Beyond automatic noise control, you have explicit control over what Anna tells you about.

Anna's visibility system has three layers:
1. **Automatic noise control**: Low-priority hints naturally fade after 2-3 viewings
2. **User decisions**: You explicitly acknowledge or snooze specific issues
3. **Display filtering**: `daily` shows only what needs attention, `status` shows everything

**When to Use User Control**

- **Acknowledge**: "I know about this, stop showing it in daily" (e.g., firewall on trusted network)
- **Snooze**: "Hide this for 30 days, I'll deal with it later" (e.g., orphaned packages)
- **Reset**: "Return to normal behavior" (reverts acknowledge/snooze)

**The `annactl issues` Command**

List all issues with their decision status:

```bash
$ annactl issues

📋 Current Issues and Decisions:

Severity    Key                            Decision             Title
----------------------------------------------------------------------------------------------------
⚠️  WARNING firewall-inactive               acknowledged         No active firewall detected
ℹ️  INFO    orphaned-packages              snoozed until 2025-12-15  63 orphaned packages found
⚠️  WARNING tlp-not-enabled                 none                 Laptop detected but TLP not en...

💡 Tip: Use 'annactl issues --acknowledge <key>' to acknowledge an issue
   Or 'annactl issues --snooze <key> --days 30' to snooze it
```

**Acknowledge an Issue** - Keep in status, hide from daily:

```bash
$ annactl issues --acknowledge firewall-inactive

✅ Issue 'firewall-inactive' acknowledged
   It will no longer appear in daily, but remains visible in status.
```

**Behavior after acknowledging:**
- ❌ Won't appear in `annactl daily` output
- ✅ Remains visible in `annactl status` with `[acknowledged]` marker
- ✅ Persists across reboots and Anna updates
- ✅ Applies even if issue isn't currently present

**Example Use Case:** You decide not to enable a firewall on your trusted home network laptop. Acknowledging the firewall warning silences the daily nag while keeping you aware in detailed status.

**Snooze an Issue** - Temporarily hide for N days:

```bash
$ annactl issues --snooze orphaned-packages --days 30

⏰ Issue 'orphaned-packages' snoozed for 30 days (until 2025-12-15)
   It will not appear in daily until that date.
```

**Behavior after snoozing:**
- ❌ Won't appear in `annactl daily` until snooze expires
- ❌ Won't appear in `annactl status` until snooze expires
- ✅ After expiration, returns to normal visibility
- ✅ Can be extended by snoozing again

**Example Use Case:** You have 63 orphaned packages but want to deal with them in one cleanup session next month. Snoozing for 30 days hides the issue until you're ready.

**Reset a Decision** - Return to normal behavior:

```bash
$ annactl issues --reset firewall-inactive

🔄 Decision reset for issue 'firewall-inactive'
   Normal noise control rules will apply again.
```

**After resetting:**
- Issue returns to normal visibility behavior
- Noise control rules apply (first 2-3 times shown, then de-emphasized if ignored)
- Can be acknowledged or snoozed again later

**Decision Persistence**

- Decisions are stored in `/var/lib/anna/context.db`
- Survive system reboots and Anna upgrades
- Apply by issue key, not by instance (works even if issue not currently present)
- Can be inspected with `annactl issues` list command

**Visibility Summary by Command**

| Command | Shows Acknowledged? | Shows Snoozed? | Shows Deemphasized? |
|---------|-------------------|----------------|---------------------|
| `daily` | ❌ No | ❌ No | ❌ No (says "X hidden") |
| `status` | ✅ Yes (marked) | ❌ No | ✅ Yes (full list) |
| `issues` | ✅ Yes | ✅ Yes | ✅ Yes |

**Real-World Scenario: Firewall on Trusted Network**

```bash
# First time - Anna suggests firewall
$ annactl daily
...
⚠️  No active firewall detected
    This machine appears to be online with no active firewall...

# You decide: trusted home network, don't need firewall
# Acknowledge the issue
$ annactl issues --acknowledge firewall-inactive

# Next time - no more daily nagging
$ annactl daily
✅ System is stable!

# But still visible in detailed status with marker
$ annactl status
...
⚠️  Warnings:
  • No active firewall detected [acknowledged]
    ...
```

**JSON Output for Scripts**

Get machine-readable output for automation:

```bash
# Daily with visible issues only (compact)
$ annactl daily --json
{
  "profile": "Laptop",
  "timestamp": "2025-11-13T09:30:00Z",
  "health": {
    "ok": 4,
    "warnings": 1,
    "failures": 0
  },
  "disk": {
    "used_percent": 87.2,
    "total_bytes": 512000000000,
    "available_bytes": 65536000000
  },
  "issues": [
    {
      "key": "disk-space-warning",
      "title": "Disk 87% full - action needed soon",
      "severity": "warning",
      "visibility": "normal",
      "category": "disk_space",
      "summary": "Your disk is getting full...",
      "recommended_action": "Run 'sudo annactl repair'",
      "repair_action_id": "cleanup-disk-space",
      "reference": "https://wiki.archlinux.org/title/System_maintenance",
      "impact": "Frees 30GB",
      "decision": {
        "kind": "none",
        "snoozed_until": null
      }
    }
  ],
  "deemphasized_issue_count": 2
}

# Status with ALL issues including deemphasized (comprehensive)
$ annactl status --json
{
  "profile": "Laptop",
  "timestamp": "2025-11-13T09:30:00Z",
  "health": { ... },
  "disk": { ... },
  "issues": [
    {
      "key": "firewall-inactive",
      "title": "No active firewall detected",
      "severity": "warning",
      "visibility": "normal",
      "decision": {
        "kind": "acknowledged",
        "snoozed_until": null
      }
      ...
    },
    {
      "key": "orphaned-packages",
      "severity": "info",
      "visibility": "deemphasized",
      "decision": {
        "kind": "snoozed",
        "snoozed_until": "2025-12-15"
      }
      ...
    }
  ]
}
```

**JSON Fields Explained:**
- `key`: Stable identifier for tracking issues
- `severity`: `"critical"`, `"warning"`, or `"info"`
- `visibility`: `"normal"`, `"low_priority"`, or `"deemphasized"`
- `category`: Issue category (e.g., `"disk_space"`, `"pacman_locks"`)
- `decision.kind`: `"none"`, `"acknowledged"`, or `"snoozed"`
- `decision.snoozed_until`: ISO 8601 date if snoozed, null otherwise

**Integration Examples:**

Monitor specific issues:
```bash
# Check if critical issues exist
critical_count=$(annactl daily --json | jq '[.issues[] | select(.severity=="critical")] | length')
if [ "$critical_count" -gt 0 ]; then
  notify-send "Anna Alert" "$critical_count critical issues detected"
fi
```

Track disk space trends:
```bash
# Log disk usage daily
annactl daily --json | jq -r '"\(.timestamp),\(.disk.used_percent)"' >> /var/log/disk-usage.csv
```

**Decision Layer Philosophy**

Anna's three-layer system creates calm, predictable behavior:

1. **First layer (automatic)**: Noise control learns what you ignore
2. **Second layer (explicit)**: You tell Anna what to hide
3. **Third layer (display)**: Commands show appropriate detail level

This means:
- **No nagging**: Low-priority items fade naturally
- **No surprises**: You control what you see
- **No hiding**: Everything remains accessible in `status`

### Phase 5.0: Storage & Network Reliability

**The "Don't Lose My Data, Don't Break My Network" Release**

Anna now monitors for early warning signs of disk failure and network issues - the kind of problems that escalate quickly if ignored.

**Three New Detector Categories (15→18 Total)**

**16. Disk SMART Health** (`check_disk_smart_health()` in `caretaker_brain.rs`):

Early warning system for failing disks using smartmontools.

**What Anna Checks:**
- Runs `smartctl -H /dev/sdX` on all non-removable disks and NVMe devices
- Detects SMART health status: PASSED, PREFAIL, FAILING, FAILED
- If smartmontools not installed: suggests installation (Info severity)

**Severity Levels:**
- **Critical**: SMART health is FAILING or FAILED - disk may fail imminently
- **Warning**: SMART health shows PREFAIL or warnings - early warning signs
- **Info**: smartmontools not installed but disks are present

**Repair Action:**
- **Guidance only** - No auto-repair (too dangerous)
- `disk-smart-guidance`: Provides structured backup and replacement guidance
- Explicitly warns against running fsck or repartition on failing disks
- Recommends: immediate backup, detailed SMART review, extended tests, disk replacement

**Why It Matters:**
Disk failure is often preceded by SMART warnings. Catching these early gives you time to back up data and plan replacement before catastrophic failure.

**Example:**
```bash
$ annactl daily
🔴 Disk SMART health failing (sda)
   SMART health check reports that /dev/sda is FAILING. This disk may fail soon and cause data loss.
   💡 Action: Back up important data immediately and plan disk replacement
   📊 Impact: Risk of data loss; immediate backup recommended

$ sudo annactl repair disk-smart-guidance
⚠️  SMART health issues detected:

1. Back up important data IMMEDIATELY
   - Your disk may fail at any time
   - Use rsync, borg, or similar tools

2. Review detailed SMART data:
   sudo smartctl -a /dev/sda

3. Run extended SMART test:
   sudo smartctl -t long /dev/sda

⚠️  DO NOT RUN fsck or repartition on a failing disk
```

**17. Filesystem / Kernel Storage Errors** (`check_filesystem_errors()` in `caretaker_brain.rs`):

Detects repeated filesystem errors in kernel logs that suggest disk or filesystem corruption.

**What Anna Checks:**
- Scans `journalctl -k -b` (kernel messages, this boot only)
- Looks for: "EXT4-fs error", "BTRFS error", "XFS error", "I/O error" on block devices
- Counts error frequency and collects sample messages

**Severity Levels:**
- **Critical**: 10+ serious filesystem errors this boot
- **Warning**: 3-9 filesystem errors this boot
- Silent if no significant errors (not Info-level)

**Repair Action:**
- **Guidance only** - Never runs fsck automatically (data loss risk)
- `filesystem-errors-guidance`: Provides filesystem-specific repair instructions
- Recommends: backup first, check SMART health, schedule fsck from live environment
- Different guidance for EXT4 (e2fsck), BTRFS (scrub), XFS (xfs_repair)

**Why It Matters:**
Filesystem errors in kernel logs often indicate failing disks or corruption. Early detection prevents data loss by prompting immediate backup and diagnostic actions.

**Example:**
```bash
$ annactl status
⚠️  Filesystem errors detected (12 errors)
   Kernel reported 12 filesystem or I/O errors this boot. This may indicate failing disk or filesystem corruption.
   💡 Action: Check backups, avoid heavy writes, and schedule a filesystem check from a live environment

$ sudo annactl repair filesystem-errors-guidance
⚠️  Filesystem errors detected in kernel log:

1. Back up important data IMMEDIATELY
2. Review kernel errors:
   journalctl -k -b | grep -i 'error\|fail'

3. Check disk SMART health:
   sudo smartctl -a /dev/sdX

4. Schedule filesystem check from live environment:
   For EXT4:
   - Boot from Arch ISO or live USB
   - sudo e2fsck -f /dev/sdX

⚠️  DO NOT run filesystem checks on mounted filesystems
   Always use a live environment or unmount first
```

**18. Network & DNS Health** (`check_network_health()` in `caretaker_brain.rs`):

Detects "desktop feels broken" network issues: no connectivity or broken DNS.

**What Anna Checks:**
- Checks if any network interface has an IP address (excluding loopback)
- Tests connectivity with `ping -c1 -W2 archlinux.org` (DNS + connectivity)
- If DNS fails: tests direct IP ping to `1.1.1.1` to differentiate DNS vs connectivity

**Profile Behavior:**
- **Laptop/Desktop**: Full checks with Critical/Warning severity
- **Server-Like**: Skipped (servers typically have dedicated network monitoring)

**Severity Levels:**
- **Critical**: No IP addresses assigned OR no external connectivity to IP
- **Warning**: IP connectivity works but DNS resolution fails

**Repair Action:**
- **Conservative service restart only** - Never edits config files
- `network-health-repair`: Detects NetworkManager vs systemd-networkd, restarts if appropriate
- If no recognized network manager: prints guidance for manual troubleshooting
- Dry-run mode safe to test

**Why It Matters:**
Network issues on desktops/laptops often manifest as "nothing works" - catching DNS vs connectivity problems helps users troubleshoot effectively.

**Example:**
```bash
$ annactl daily
⚠️  DNS resolution failing
   Network connectivity works but DNS resolution is broken. Most Internet services will fail.
   💡 Action: Check /etc/resolv.conf and restart network services: 'sudo systemctl restart NetworkManager'

$ sudo annactl repair network-health-repair
✅ network-health-repair: Restarted NetworkManager

# Now DNS should work again
$ ping archlinux.org
PING archlinux.org (95.217.163.246) 56(84) bytes of data.
```

**Phase 5.0 Integration**

All three detectors integrate seamlessly with existing Anna systems:

**Profile-Aware:**
- SMART & filesystem errors: All profiles
- Network health: Desktop/Laptop only (Server-Like skipped)

**Noise Control:**
- SMART Info (install suggestion): Can be de-emphasized after 2-3 viewings
- SMART Critical/Warning: Always visible (too important to hide)
- Filesystem Critical: Always visible
- Network Critical/Warning: Always visible

**User Decisions:**
- Can acknowledge SMART Info to hide daily install nagging
- **Cannot** suppress Critical issues (SMART failing, filesystem errors, network down)
- Snooze not recommended for Critical issues

**JSON Output:**
All issues appear in `annactl daily --json` and `annactl status --json` with proper:
- `category`: "disk_smart_health", "filesystem_errors", "network_health"
- `severity`: "critical", "warning", or "info"
- `visibility`: Always "normal" for Critical issues
- `repair_action_id`: "disk-smart-guidance", "filesystem-errors-guidance", "network-health-repair"

**Safety Philosophy**

Phase 5.0 maintains Anna's "don't make things worse" principle:

**Storage Detectors (SMART, Filesystem):**
- **Guidance only** - Never run destructive operations
- No auto-fsck, no repartitioning, no filesystem modifications
- Clear warnings about when operations are safe vs dangerous
- Emphasis on backup first, then diagnostics

**Network Detector:**
- **Service restart only** - Never edit config files
- Detects active network manager before acting
- Dry-run mode available for safety
- Falls back to manual guidance if uncertain

**Real-World Scenario: Failing Disk**

```bash
# Morning: Anna detects early SMART warning
$ annactl daily
⚠️  Disk SMART health warning (sda)
   SMART health check reports warnings for /dev/sda. The disk may be developing problems.

# User acknowledges but starts backup immediately
$ rsync -av /home/user/ /mnt/backup/

# Days later: Disk condition worsens
$ annactl daily
🔴 Disk SMART health failing (sda)
🔴 Filesystem errors detected (15 errors)

   Critical issues detected - run 'sudo annactl repair' now

# User gets detailed guidance
$ sudo annactl repair disk-smart-guidance
$ sudo annactl repair filesystem-errors-guidance

# Result: User backs up data, orders replacement disk, avoids catastrophic data loss
```

**Detector Count: 18 Categories Total**

Anna now monitors:
1. Disk space (15 detectors from Phases 0-4.8)
2. Failed services
3. Pacman locks
4. Laptop power (TLP)
5. GPU drivers
6. Journal errors
7. Zombie processes
8. Orphaned packages
9. Core dumps
10. Time sync
11. Firewall status
12. Backup awareness
13. User services (Desktop/Laptop)
14. Broken autostart (Desktop/Laptop)
15. Heavy cache/trash
16. **Disk SMART health** (Phase 5.0)
17. **Filesystem errors** (Phase 5.0)
18. **Network & DNS health** (Phase 5.0, Desktop/Laptop)

---

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

### Repairing System Issues

Anna can automatically fix many common system issues, with strong safety guarantees:

#### Preview Before Acting

Always preview repairs before executing them:

```bash
# See exactly what Anna would do, without making any changes
$ sudo annactl repair --dry-run

════════════════════════════════════════════════════════════
🔧 SYSTEM REPAIR (DRY RUN)
════════════════════════════════════════════════════════════

Anna would attempt to fix detected issues automatically.
Only low-risk actions would be performed.

⚠️  This is a DRY RUN - no changes will be made!

📋 PLANNED REPAIRS

✅ disk-space
  Action: cleanup_disk_space
  Would run: paccache -rk1
  Impact: Would free ~12.4GB from package cache
  Risk: ✅ Safe (keeps latest package version)

✅ failed-service: bluetooth.service
  Action: restart_service
  Would run: systemctl restart bluetooth.service
  Impact: Would restart bluetooth service
  Risk: ⚠️  Low (service restart only)

────────────────────────────────────────────────────────────
Summary: 2 actions planned, 0 blocked
```

#### Execute Repairs

After reviewing the dry-run output:

```bash
# Actually perform the repairs
$ sudo annactl repair

════════════════════════════════════════════════════════════
🔧 SYSTEM REPAIR
════════════════════════════════════════════════════════════

Anna will attempt to fix detected issues automatically.
Only low-risk actions will be performed.

⚠️  Actions may modify system state!

Proceed with repair? [y/N]: y

🔧 EXECUTING REPAIRS

✅ disk-space
  Action: cleanup_disk_space
  Details: Installed pacman-contrib; paccache -rk1: removed 847 packages (12.4GB)
  Source: [archwiki:System_maintenance#Clean_the_filesystem]

✅ failed-service: bluetooth.service
  Action: restart_service
  Details: systemctl restart bluetooth.service: succeeded
  Source: [archwiki:Systemd#Using_units]

────────────────────────────────────────────────────────────
Summary: 2 succeeded, 0 failed

All repairs logged to context database.
Run 'annactl repairs' to view full history.
```

#### View Repair History

Anna maintains a complete history of all repair actions for transparency:

```bash
# Show recent repairs (requires --help --all to discover)
$ annactl repairs

════════════════════════════════════════════════════════════
🔧 REPAIR HISTORY
════════════════════════════════════════════════════════════

✅ disk-space
  Time:   2025-11-13T10:30:00Z
  Action: cleanup_disk_space
  Result: success
  Installed pacman-contrib; paccache -rk1: removed 847 packages (12.4GB)

✅ failed-service
  Time:   2025-11-13T10:30:05Z
  Action: restart_service
  Result: success
  systemctl restart bluetooth.service: succeeded

────────────────────────────────────────────────────────────
Showing 2 most recent repairs
Use --json for machine-readable output

# JSON output for scripting
$ annactl repairs --json
{
  "schema_version": "v1",
  "repairs": [...]
}
```

#### Safety Principles

Anna follows strict safety principles - she never makes things worse:

**Storage Safety:**
- Disk SMART and filesystem errors: **guidance only**
- Never runs fsck, repartition, or destructive operations automatically
- Clear warnings about safe vs dangerous operations
- Backup first, then diagnose

**Network Safety:**
- Only restarts services (NetworkManager/systemd-networkd)
- Never edits configuration files
- Falls back to manual guidance if uncertain

**Transparency:**
- All actions are logged to the context database
- Repair history always available via `annactl repairs`
- Dry-run mode shows exact commands before execution
- JSON output available for automation (`--json`)

**Reversibility:**
- Package operations use pacman (can be rolled back)
- Service restarts are non-destructive
- Configuration never modified automatically
- Manual rollback guidance provided when needed

### Weekly System Summary

Anna can provide a 7-day overview combining behavioral patterns with repair activity:

```bash
# Human-readable weekly summary
$ annactl weekly

════════════════════════════════════════════════════════════
🗓️  WEEKLY SYSTEM SUMMARY - Laptop Profile (Last 7 Days)
════════════════════════════════════════════════════════════

📅 Period: 2025-11-06 → 2025-11-13

📊 Recurring Issues

   • orphaned-packages flapped 3 times (Appeared/disappeared repeatedly)
     💡 Consider addressing this more permanently.

   • disk-space escalated (Severity increased from Info to Warning)
     ⚠️  This issue is getting worse over time.

🔧 Repairs Executed

   • cleanup_disk_space - Ran 2 times (last: 2025-11-13 10:30)
   • orphaned-packages - Ran 3 times (last: 2025-11-12 15:20)

💡 Suggested Habits

   • You ran 'orphaned-packages' 3 times this week. Maybe add a monthly cleanup to your routine.

────────────────────────────────────────────────────────────
💡 For detailed patterns, run 'annactl insights'
   For current status, run 'annactl status'

# JSON output for monitoring/scripts
$ annactl weekly --json
{
  "schema_version": "v1",
  "generated_at": "2025-11-13T10:45:00Z",
  "profile": "Laptop",
  "window_start": "2025-11-06T10:45:00Z",
  "window_end": "2025-11-13T10:45:00Z",
  "total_observations": 42,
  "recurring_issues": [...],
  "escalating_issues": [...],
  "long_term_issues": [...],
  "repairs": [...],
  "suggestions": [...]
}
```

**What the Weekly Report Shows:**

- **Recurring Issues** - Problems that appeared/disappeared multiple times (flapping)
- **Escalating Issues** - Problems that got worse over time
- **Long-term Issues** - Items visible for weeks without resolution
- **Repairs Executed** - What Anna fixed and how often
- **Suggested Habits** - Rule-based recommendations for preventive maintenance

**When to Use:**

- Weekly system review (e.g., every Monday morning)
- Understanding repair patterns over time
- Planning preventive maintenance
- Monitoring scripts via `--json` output

**Discovery Hint:**

Anna may hint once per week if patterns exist:
```
💡 Weekly snapshot available. For a 7-day overview run: 'annactl weekly'.
```

### Behavioral Insights (Advanced)

Anna silently observes your system over time and can detect patterns that aren't obvious from a single snapshot. This is completely optional and requires no configuration.

```bash
# View behavioral patterns (hidden command - requires --help --all)
$ annactl insights

════════════════════════════════════════════════════════════
📊 INSIGHTS REPORT (Last 30 Days) - Laptop Profile
════════════════════════════════════════════════════════════

📈 Analyzed 127 observations

🔄 Flapping Issues
   Issues that appear and disappear repeatedly (last 14 days)

   • orphaned-packages
     Appeared 6 times over 14 days without permanent resolution
     Confidence: 85%

📈 Escalating Issues
   Issues that increased in severity over time

   • disk-space
     Severity increased from Info → Warning → Critical
     Details: Severity increased by 2 levels
     Time span: 21 days
     Confidence: 100%

⏳ Long-term Unaddressed Issues
   Issues visible for more than 14 days without resolution

   • time-sync-disabled
     Visible for 30 days across 18 observations
     Confidence: 90%

🔝 Top Recurring Issues

   1. orphaned-packages (15 appearances)
   2. disk-space (12 appearances)
   3. failed-autostart (8 appearances)

────────────────────────────────────────────────────────────
💡 These patterns are based on your system's behavior over time.
   For current status, run 'annactl status'
   For quick health check, run 'annactl daily'

# JSON output for automation
$ annactl insights --json
{
  "schema_version": "v1",
  "generated_at": "2025-11-13T10:30:00Z",
  "profile": "Laptop",
  "analysis_window_days": 30,
  "total_observations": 127,
  "flapping": [...],
  "escalating": [...],
  "long_term": [...],
  "profile_transitions": [...],
  "top_recurring_issues": [...]
}
```

**Pattern Types:**

- **Flapping** - Issues that appear/disappear >5 times in 2 weeks
- **Escalation** - Severity increases over time (e.g., Info → Warning → Critical)
- **Long-term Unaddressed** - Visible for >14 days without action
- **Profile Transitions** - Machine profile changes (e.g., Laptop → Desktop in VMs)

**How It Works:**

1. Anna records system state after every `daily` and `status` command
2. Observations stored in context database (`/var/lib/anna/context.db`)
3. Pattern detection runs on demand when you call `insights` or `weekly`
4. All patterns calculated using rule-based algorithms (no AI/ML)

**Discovery Hint:**

Anna may hint once per day if patterns exist:
```
💡 Insight: Recurring patterns detected. For details run 'annactl insights'.
```

**When to Use:**

- Understanding why issues keep coming back
- Planning permanent fixes for recurring problems
- Spotting trends before they become critical
- Monitoring long-term system behavior

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

#### 10. Time Synchronization (NEW in 4.5)

Ensures your system clock is synchronized with network time:

```bash
ℹ️ Recommendations:

  • Time synchronization not enabled
    systemd-timesyncd is available but not enabled. Your system clock may drift over time.
    💡 Run 'sudo systemctl enable --now systemd-timesyncd.service'
    📚 https://wiki.archlinux.org/title/Systemd-timesyncd
```

**What Anna Checks:**
- Active NTP services: systemd-timesyncd, chronyd, ntpd
- Service availability and enabled status

**Severity Levels:**
- **Warning**: No network time synchronization active
- **Info**: Service available but not enabled

**Repair Actions:**
- Enables systemd-timesyncd via `sudo annactl repair time-sync-enable`
- Safe automatic action, checks for conflicting NTP services first
- Verifies synchronization after enabling

**Why It Matters:**
Clock drift causes TLS certificate validation failures, incorrect log timestamps, and issues with time-sensitive applications.

#### 11. Firewall Status (NEW in 4.5)

Detects networked machines without firewall protection:

```bash
⚠️ Warnings:

  • No active firewall detected
    This machine appears to be online with no active firewall. Incoming connections are not filtered.
    💡 Install ufw: 'sudo pacman -S ufw', then configure: 'sudo ufw allow ssh && sudo ufw enable'
    📚 https://wiki.archlinux.org/title/Uncomplicated_Firewall
```

**What Anna Checks:**
- Network interfaces (skips if only loopback)
- Active firewall solutions: ufw, firewalld, nftables, iptables
- Installed but inactive firewall packages

**Severity Levels:**
- **Warning**: Online machine with no firewall
- **Info**: Firewall installed but not active

**Important:** Firewall configuration is **guidance only**. Anna will never automatically enable or configure firewall rules for safety reasons. You must review and enable manually.

**Safe Firewall Setup:**
```bash
# Install ufw
$ sudo pacman -S ufw

# Allow SSH first (important!)
$ sudo ufw allow ssh

# Enable firewall
$ sudo ufw enable

# Check status
$ sudo ufw status
```

#### 12. Backup Awareness (NEW in 4.5)

Reminds you to configure backups if none are detected:

```bash
ℹ️ Recommendations:

  • No backup or snapshot tools detected
    No common backup tools (timeshift, borg, restic) detected. If this machine holds important data, consider configuring backups.
    💡 Options: Install timeshift ('pacman -S timeshift'), borg ('pacman -S borg'), or restic ('pacman -S restic')
    📚 https://wiki.archlinux.org/title/Backup_programs
```

**What Anna Checks:**
- Common backup tools: timeshift, borg, restic, rsnapshot
- btrfs systems with snapshot capability

**Severity Level:**
- **Info only**: Non-intrusive reminder

**No Automatic Action:** Backup configuration is complex and personal. Anna provides suggestions only.

**Backup Options:**
- **Timeshift**: GUI-friendly, great for btrfs/ext4 snapshots
- **Borg**: Deduplicating encrypted backups to local/remote storage
- **Restic**: Fast, efficient backups with multiple backends
- **btrfs snapshots**: Built-in if using btrfs filesystem

#### 13. User Services Failed (NEW in 4.8)

Detects failing user-level systemd services (desktop/laptop only):

```bash
⚠️ Warnings:

  • 2 user services failing
    User-level systemd services are failing: pipewire.service, wireplumber.service. This may cause desktop features to not work properly.
    💡 Check with 'systemctl --user status' and run 'sudo annactl repair user-services-failed'
    📚 https://wiki.archlinux.org/title/Systemd/User
```

**What Anna Checks:**
- Runs `systemctl --user list-units --failed`
- Identifies which user services are in failed state
- Only runs on Desktop and Laptop profiles (skipped on Server-like)

**Severity Levels:**
- **Critical**: Core desktop services failing (plasma-, gnome-, pipewire, wireplumber)
- **Warning**: Other user services failing

**Repair Actions:**
- Auto-restarts safe services: pipewire, wireplumber, pipewire-pulse
- Provides guidance for other services (manual investigation recommended)
- Reports restart success/failure

**Why It Matters:**
User services power your desktop audio, session management, and many desktop features. When they fail, you might not hear audio, see notifications, or use certain desktop features properly.

**Troubleshooting:**
```bash
# Check all user services
$ systemctl --user status

# Check specific failed service
$ systemctl --user status pipewire.service

# View service logs
$ journalctl --user -xeu pipewire.service

# Restart via Anna (safest)
$ sudo annactl repair user-services-failed
```

#### 14. Broken Autostart Entries (NEW in 4.8)

Detects .desktop autostart files pointing to missing programs (desktop/laptop only):

```bash
⚠️ Warnings:

  • 3 broken autostart entries
    Desktop autostart entries point to missing programs: old-app.desktop (old-app), removed-tool.desktop (removed-tool), uninstalled.desktop (uninstalled). These will fail silently on login.
    💡 Review with 'ls ~/.config/autostart/' and run 'sudo annactl repair broken-autostart'
    📚 https://wiki.archlinux.org/title/XDG_Autostart
```

**What Anna Checks:**
- Scans `~/.config/autostart` and `/etc/xdg/autostart` for .desktop files
- Parses Exec= lines to extract command names
- Checks if commands exist in PATH via `which`
- Only runs on Desktop and Laptop profiles (skipped on Server-like)

**Severity Levels:**
- **Warning (>3 broken)**: Many broken entries cluttering autostart
- **Info (1-3 broken)**: Minor cleanup recommended

**Repair Actions:**
- Moves broken user entries to `~/.config/autostart/disabled/`
- Provides guidance for system entries (requires manual intervention)
- Safe: doesn't delete, only disables

**Why It Matters:**
Broken autostart entries accumulate when you uninstall applications. They fail silently every login, slowing down your session startup and cluttering your autostart directory.

**Troubleshooting:**
```bash
# List all autostart entries
$ ls -la ~/.config/autostart/

# Check what each entry does
$ cat ~/.config/autostart/some-app.desktop

# Disable via Anna (moves to disabled/)
$ sudo annactl repair broken-autostart

# Re-enable if needed
$ mv ~/.config/autostart/disabled/some-app.desktop ~/.config/autostart/
```

#### 15. Heavy User Cache & Trash (NEW in 4.8)

Monitors user cache and trash directories consuming disk space:

```bash
⚠️ Warnings:

  • Large user cache and trash (12 GB)
    User cache (8,456 MB) and trash (3,821 MB) are consuming 12 GB. This is safe to clean.
    💡 Run 'rm -rf ~/.cache/* ~/.local/share/Trash/*' or use 'sudo annactl repair heavy-user-cache'
    💾 Impact: Frees 12 GB
    📚 https://wiki.archlinux.org/title/System_maintenance#Clean_the_filesystem
```

**What Anna Checks:**
- Calculates size of `~/.cache` directory
- Calculates size of `~/.local/share/Trash` directory
- Runs on all profiles (Desktop, Laptop, and Server-like)

**Severity Levels:**
- **Warning (>10GB total)**: Significant disk space consumed
- **Info (single dir >2GB)**: Cleanup recommended

**Repair Actions:**
- Cleans `~/.cache/*` (application temporary files)
- Empties `~/.local/share/Trash/*` (desktop trash)
- Reports MB/GB freed
- Safe: cache and trash are meant to be clearable

**Why It Matters:**
Cache directories grow over time as applications store temporary data. The desktop trash holds deleted files until manually emptied. Both are safe to clean and can free significant disk space.

**Troubleshooting:**
```bash
# Check cache size
$ du -sh ~/.cache

# Check trash size
$ du -sh ~/.local/share/Trash

# Clean via Anna (safest)
$ sudo annactl repair heavy-user-cache

# Manual clean (advanced)
$ rm -rf ~/.cache/*
$ rm -rf ~/.local/share/Trash/*
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

# Anna checks ALL 15 categories and prioritizes findings
# You get immediate visibility into system health
# Fix critical issues first, then warnings, then info
```

#### Scenario 4: Desktop Hygiene - User Services and Cache (NEW in 4.8)

This scenario shows Anna detecting desktop-level issues on a laptop:

```bash
$ annactl daily

╔══════════════════════════════════════════════════════════╗
║ 🔴 Daily System Check - 2025-11-13 14:22 ║
╚══════════════════════════════════════════════════════════╝

Health: 5 ok, 1 warnings, 2 failures

Disk: 87.2% used (210.5GB / 512.0GB total)

📊 Issues Detected:

  🔴 2 user services failing (pipewire.service, wireplumber.service)
  ⚠️ Large user cache and trash (12 GB)
  ℹ️ 2 broken autostart entries

💡 Run 'sudo annactl repair' to fix these issues

# Fix all desktop hygiene issues
$ sudo annactl repair

════════════════════════════════════════════════════════════
🔧 SYSTEM REPAIR
════════════════════════════════════════════════════════════

Anna will attempt to fix detected issues automatically.
Only low-risk actions will be performed.

⚠️  Actions may modify system state!

Proceed with repair? [y/N]: y

🔧 EXECUTING REPAIRS

✅ user-services-failed
  Action: restart_user_services
  Details: Restarted pipewire.service; Restarted wireplumber.service. Total freed: ~10,234MB
  Source: [archwiki:Systemd/User]

✅ heavy-user-cache
  Action: clean_user_cache
  Details: Cleaned cache (~8,456MB freed); Cleaned trash (~3,778MB freed). Total freed: ~12,234MB
  Source: [archwiki:System_maintenance#Clean_the_filesystem]

✅ broken-autostart
  Action: disable_broken_entries
  Details: Disabled old-app.desktop; Disabled removed-tool.desktop
  Source: [archwiki:XDG_Autostart]

────────────────────────────────────────────────────────────
Summary: 3 succeeded, 0 failed

# Check disk space improvement
$ annactl status

Disk: 84.8% used (198.3GB / 512.0GB total)
# Freed 12.2GB from cache and trash cleanup!

Health: All checks passed ✅
```

**What happened:**
1. Anna detected failing audio services (pipewire, wireplumber) - common after system updates
2. Found 12GB of accumulated cache and trash files
3. Identified 2 broken autostart entries pointing to uninstalled programs
4. Repair automatically:
   - Restarted the safe audio services (audio works again!)
   - Cleaned cache and trash (12GB freed)
   - Moved broken autostart entries to disabled/ folder
5. Result: Working audio, faster login, 12GB disk space freed

**Desktop/Laptop Focus:** These checks only run on Desktop and Laptop profiles. Server-like systems skip user services and autostart checks to avoid noise.

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
| Time Sync ✨ | Every run | Warning/Info | Yes |
| Firewall ✨ | Every run | Warning/Info | No (guidance) |
| Backups ✨ | Every run | Info only | No (guidance) |
| User Services 🖥️ | Every run | Critical/Warning | Yes (safe only) |
| Broken Autostart 🖥️ | Every run | Warning/Info | Yes (user only) |
| Heavy Cache/Trash 🖥️ | Every run | Warning/Info | Yes |

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
