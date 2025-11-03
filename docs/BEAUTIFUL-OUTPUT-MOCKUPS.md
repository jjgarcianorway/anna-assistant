# Anna — Beautiful CLI Output Mockups

This document showcases Anna's **living command-line aesthetic** — expressive, readable, delightful. Every log line sings, every progress bar hums.

---

## 🧬 Installation (already implemented: `scripts/install.sh`)

```
╔═══════════════════════════════════════════════════════════════════════╗
║                                                                       ║
║                        🤖  Anna Assistant                             ║
║                   Event-Driven Intelligence                           ║
║                                                                       ║
╚═══════════════════════════════════════════════════════════════════════╝

━━━ 🚀 Installation Starting

  📦  Checking dependencies
     ↳ curl: found
     ↳ jq: found
     ↳ systemctl: found

━━━ 📦 Fetching Latest Release

  🔍  Finding latest release
     ↳ Latest version: v1.0.0-rc.7

  ✓  Assets available for v1.0.0-rc.7

  📡  Downloading release v1.0.0-rc.7
     ↳ Downloading tarball...
  ✓  Downloading tarball

  🔐  Verifying integrity
  ✓  Checksum verified

  📦  Extracting binaries
  ✓  Binaries installed to /usr/local/bin

━━━ ⚙️  System Configuration

  ⚙️   Configuring system
     ↳ Creating anna user...
  ✓  User 'anna' created
     ↳ Setting up directories...
     ↳ Adding lhoqvso to 'anna' group...
  ⚠  You'll need to logout/login for group membership to take effect
     ↳ Installing systemd service...
     ↳ Starting daemon...
  ✓  Daemon configured and started

━━━ 🩺 Health Check & Auto-Repair

  ℹ  Anna will now check her own health and fix any issues...

  🔍  Running health checks...
     ✓ Binaries installed
     ✓ Directories created
     ✓ Permissions correct
     ✓ Daemon running
     ✓ Socket responding

  ✓  Health check complete

╔═══════════════════════════════════════════════════════════════════════╗
║                                                                       ║
║                   ✨  Installation Complete! ✨                       ║
║                                                                       ║
║              Anna is now running and ready to assist                  ║
║                                                                       ║
╚═══════════════════════════════════════════════════════════════════════╝

  Next Steps:
    annactl status   # Check Anna's current state
    annactl report   # Get system health report
    annactl --help   # See all available commands

  Documentation: docs/V1.0-QUICKSTART.md
```

---

## 💔 Crash Recovery — The Most Dramatic Scenario

### Scenario 1: Daemon Crash Detection

```bash
$ annactl status
```

```
╭─────────────────────────────────────────────────────────────╮
│  ⚠️  Anna System Status — Degraded                           │
╰─────────────────────────────────────────────────────────────╯

│
│  ❌ Daemon: inactive (crashed)
│  ⏱  Last seen: 2m 14s ago
│  📍 PID: 1234 (deceased)
│
│  📜 Last 5 log lines:
│     [ERROR] Panic: thread 'tokio-runtime-worker' panicked
│     [ERROR] at 'index out of bounds: src/collector.rs:142'
│     [ERROR] Stack backtrace captured
│     [INFO]  Writing crash report to /var/log/anna/crash-2025-11-03.log
│     [INFO]  Daemon terminated
│
│  💡 Recommended actions:
│     • Run: annactl doctor check --verbose
│     • Run: annactl doctor repair --yes
│     • View crash report: /var/log/anna/crash-2025-11-03.log
│
│  🩹 Auto-recovery available — run repair to restore
│
╰─────────────────────────────────────────────────────────────╯

⚠️  Exit code: 2 (service degraded)
```

### Scenario 2: Self-Healing Kickoff

```bash
$ annactl doctor repair --yes
```

```
╭─────────────────────────────────────────────────────────────╮
│  🩹  Anna Doctor — Initiating Self-Repair                    │
╰─────────────────────────────────────────────────────────────╯

│
│  🔍  Phase 1: Diagnostic Scan
│     ✓ Crash report found: /var/log/anna/crash-2025-11-03.log
│     ✓ Backtrace: collector.rs:142 (index bounds)
│     ✓ Root cause: malformed telemetry snapshot
│     ✓ DB integrity: OK
│
│  📦  Phase 2: Backup Current State
│     ⏳ Creating backup...
│     ✓ Backup saved: /var/lib/anna/backups/pre-repair-2025-11-03-183045.tar.gz
│
│  🔧  Phase 3: Repair Actions
│     ⏳ Clearing malformed snapshot (ID: 3847)
│     ✓ Snapshot removed
│     ⏳ Reindexing telemetry database
│     ✓ Database reindexed (2,341 samples intact)
│     ⏳ Restarting daemon with --safe-mode
│     ✓ Daemon restarted (PID: 5678)
│
│  🏥  Phase 4: Post-Repair Health Check
│     ✓ Daemon responding
│     ✓ Socket alive: /run/anna/annad.sock
│     ✓ RPC test: OK (12ms latency)
│     ✓ Memory: 18.3 MB (healthy)
│     ✓ Queue: 0 events (clear)
│
│  🎉  Repair Complete!
│     • Duration: 4.2s
│     • Actions taken: 3
│     • Success rate: 100%
│     • Anna is operational again
│
╰─────────────────────────────────────────────────────────────╯

✨ Anna says: "I've recovered. Thank you for your patience."
```

### Scenario 3: Catastrophic Failure (Rollback Required)

```bash
$ annactl doctor check --verbose
```

```
╭─────────────────────────────────────────────────────────────╮
│  🩺  Anna Doctor — Deep Health Scan                          │
╰─────────────────────────────────────────────────────────────╯

│
│  ❌ Critical Issues Detected
│
│  1. Database Corruption
│     ⨯ Integrity check failed
│     ⨯ 847 snapshots unreadable
│     ⨯ WAL journal missing
│
│  2. Configuration Drift
│     ⨯ /etc/anna/config.toml: parse error at line 23
│     ⨯ Unknown key: "telemetry.sample_rats" (typo?)
│
│  3. Daemon Cannot Start
│     ⨯ systemctl status: failed (exit code 1)
│     ⨯ Last error: "Failed to open database"
│
│  🆘 Repair Strategy: ROLLBACK TO LAST KNOWN GOOD STATE
│
│  Available backups:
│     1. 2025-11-03 12:30 (6h ago) — pre-upgrade-v1.0.0-rc.7
│     2. 2025-11-02 18:00 (1d ago) — pre-repair
│     3. 2025-11-01 08:15 (2d ago) — weekly-auto
│
│  Recommended: Rollback to backup #1
│
╰─────────────────────────────────────────────────────────────╯

  Run: annactl doctor rollback --backup 1 --verify
```

```bash
$ annactl doctor rollback --backup 1 --verify
```

```
╭─────────────────────────────────────────────────────────────╮
│  ⏪  Anna Doctor — Rollback to Last Known Good State         │
╰─────────────────────────────────────────────────────────────╯

│
│  📦  Target Backup: pre-upgrade-v1.0.0-rc.7
│     • Created: 2025-11-03 12:30 (6h ago)
│     • Size: 12.4 MB
│     • SHA-256: verified ✓
│
│  ⚠️  WARNING: This will restore Anna to v1.0.0-rc.6
│     • Current installation will be backed up first
│     • All data from last 6 hours will be preserved
│
│  ⏳  Phase 1: Pre-Rollback Backup
│     ⏳ Backing up current state...
│     ✓ Backup saved: /var/lib/anna/backups/pre-rollback-2025-11-03-183245.tar.gz
│
│  ⏳  Phase 2: Extract Backup
│     ⏳ Verifying manifest...
│     ✓ Manifest valid (23 files)
│     ⏳ Extracting...
│     [████████████████████████████████████████] 100%
│     ✓ Extracted to /var/lib/anna/restore-staging/
│
│  ⏳  Phase 3: Restore Components
│     ⏳ Restoring binaries...
│     ✓ annad v1.0.0-rc.6 → /usr/local/bin/annad
│     ✓ annactl v1.0.0-rc.6 → /usr/local/bin/annactl
│     ⏳ Restoring database...
│     ✓ Database restored (1,494 samples intact)
│     ⏳ Restoring configuration...
│     ✓ /etc/anna/config.toml restored
│
│  ⏳  Phase 4: Restart Daemon
│     ⏳ Starting daemon...
│     ✓ Daemon started (PID: 6789)
│
│  ⏳  Phase 5: Verification
│     ✓ Version check: v1.0.0-rc.6 (expected)
│     ✓ Database query: OK (1,494 samples)
│     ✓ RPC test: OK (9ms latency)
│     ✓ Health score: 9.2/10.0
│
│  🎉  Rollback Complete!
│     • Duration: 8.7s
│     • Restored to: v1.0.0-rc.6
│     • System health: Excellent
│
╰─────────────────────────────────────────────────────────────╯

✨ Anna says: "I'm back online. The issue has been isolated."
```

---

## 🚀 Update Rollout — Beauty in Motion

### Scenario 1: Update Detection

```bash
$ annactl news
```

```
╭─────────────────────────────────────────────────────────────╮
│  📰  Anna News — Updates Available                           │
╰─────────────────────────────────────────────────────────────╯

│
│  🎉  New Version Available: v1.0.0 (stable)
│     • Released: 2025-11-03 (today)
│     • Current version: v1.0.0-rc.7
│
│  ✨ Highlights:
│     • Improved crash recovery with ML predictions
│     • 40% faster telemetry collection
│     • New storage advisor for Btrfs scrub scheduling
│     • 12 bug fixes, 3 security patches
│
│  📖 Changelog:
│     https://github.com/jjgarcianorway/anna-assistant/releases/tag/v1.0.0
│
│  🚀 To upgrade:
│     sudo bash -c "$(curl -fsSL https://install.anna.ai/upgrade.sh)"
│
│  💡 Backup recommendation: Automatic (included in upgrade process)
│
╰─────────────────────────────────────────────────────────────╯
```

### Scenario 2: Upgrade in Progress

```bash
$ sudo bash -c "$(curl -fsSL https://install.anna.ai/upgrade.sh)"
```

```
╔═══════════════════════════════════════════════════════════════════════╗
║                                                                       ║
║                     🚀  Anna Upgrade Wizard                           ║
║                   From v1.0.0-rc.7 → v1.0.0                           ║
║                                                                       ║
╚═══════════════════════════════════════════════════════════════════════╝

━━━ 🔍 Phase 1: Pre-Flight Checks

  ✓  Current installation detected
     • Version: v1.0.0-rc.7
     • Health: Excellent (9.8/10.0)
     • Uptime: 3d 12h

  ✓  Target version validated
     • Release: v1.0.0 (stable)
     • Signature: verified
     • Build: GitHub Actions #1247

  ✓  System requirements met
     • Disk space: 247 MB available (need 50 MB)
     • Memory: 3.2 GB free
     • Network: online

━━━ 📦 Phase 2: Backup Current State

  ⏳  Creating safety backup...
     [████████████████████████████████████████] 100%
  ✓  Backup saved: /var/lib/anna/backups/pre-upgrade-v1.0.0-2025-11-03.tar.gz
     • Size: 14.2 MB
     • Checksum: 7f3a9b...

━━━ 📥 Phase 3: Download New Release

  ⏳  Downloading v1.0.0 binaries...
     [████████████████████████████████████████] 100%  (2.1 MB/s)
  ✓  Tarball downloaded

  ⏳  Verifying integrity...
  ✓  SHA-256 checksum: PASS

  ⏳  Extracting binaries...
  ✓  Extracted: annad, annactl

━━━ 🔄 Phase 4: Graceful Daemon Shutdown

  ⏳  Sending SIGTERM to daemon (PID: 1234)...
  ✓  Daemon stopped gracefully (exit: 0)

  ⏳  Flushing telemetry queue...
  ✓  2,341 samples persisted to disk

━━━ ⚙️  Phase 5: Installation

  ⏳  Installing new binaries...
     ✓ /usr/local/bin/annad (v1.0.0)
     ✓ /usr/local/bin/annactl (v1.0.0)

  ⏳  Running database migrations...
     ✓ Migration 001: add_action_outcome_table
     ✓ Migration 002: add_learning_weights_table
     ✓ All migrations applied (2.3s)

  ⏳  Updating configuration...
     ✓ /etc/anna/config.toml (schema v3 → v4)

━━━ 🏥 Phase 6: Post-Install Verification

  ⏳  Starting daemon...
  ✓  Daemon started (PID: 5678)

  ⏳  Running health checks...
     ✓ Version: v1.0.0 (expected)
     ✓ Database: OK (2,341 samples)
     ✓ RPC latency: 8ms (excellent)
     ✓ Memory: 19.4 MB (healthy)

  ⏳  Running self-diagnostics...
     ✓ Telemetry collection: OK
     ✓ Event listeners: OK (3 active)
     ✓ Storage advisor: OK
     ✓ ML predictor: OK (baseline loaded)

╔═══════════════════════════════════════════════════════════════════════╗
║                                                                       ║
║                   ✨  Upgrade Successful! ✨                          ║
║                                                                       ║
║                  Anna v1.0.0 is now running                           ║
║                                                                       ║
╚═══════════════════════════════════════════════════════════════════════╝

  📊 Upgrade Statistics:
     • Duration: 23.4s
     • Downtime: 4.1s
     • Data preserved: 100%
     • New features: 7
     • Bug fixes: 12

  🎉 What's New:
     • annactl learn trend    — Behavioral trend analysis
     • annactl advisor btrfs  — Btrfs scrub scheduler
     • annactl autonomy check — Tier promotion eligibility

  📚 Documentation: docs/V1.0-WHATS-NEW.md

✨ Anna says: "I feel sharper already. Thank you for keeping me updated."
```

---

## 🔍 Self-Healing Diagnostics — Intelligent & Transparent

### Scenario 1: Routine Health Check

```bash
$ annactl doctor check
```

```
╭─────────────────────────────────────────────────────────────╮
│  🩺  Anna Doctor — Health Scan                               │
╰─────────────────────────────────────────────────────────────╯

│
│  Running 8 health checks...
│
│  ✓  [1/8] Binaries installed
│  ✓  [2/8] Directories exist
│  ✓  [3/8] Permissions correct
│  ✓  [4/8] Daemon running
│  ✓  [5/8] Socket responding
│  ✓  [6/8] Database integrity
│  ✓  [7/8] Configuration valid
│  ⚠  [8/8] Telemetry queue depth: 87 events (threshold: 50)
│
│  📊 System Health: 8.5/10.0
│
│  ⚠️  Issues Found: 1 (non-critical)
│
│  📋 Recommendations:
│     1. Queue depth elevated
│        • Detected: 87 events pending (normally <10)
│        • Impact: Telemetry processing delayed by ~3s
│        • Fix: annactl doctor repair --yes
│        • Auto-fix: Process backlog and adjust collection rate
│
╰─────────────────────────────────────────────────────────────╯

💡 Tip: Run with --verbose for detailed diagnostics
```

### Scenario 2: Verbose Diagnostic Output

```bash
$ annactl doctor check --verbose
```

```
╭─────────────────────────────────────────────────────────────╮
│  🩺  Anna Doctor — Deep Health Scan                          │
╰─────────────────────────────────────────────────────────────╯

│
│  🔍 Diagnostic Suite: 8 checks, 47 sub-checks
│
│  ✓  [1/8] Binaries Installed
│     ✓ /usr/local/bin/annad (v1.0.0, 3.2 MB)
│     ✓ /usr/local/bin/annactl (v1.0.0, 2.1 MB)
│     ✓ Binary permissions: 0755 (correct)
│     ✓ Binary ownership: root:root (correct)
│     ✓ Binary signatures: valid
│
│  ✓  [2/8] Directories Exist
│     ✓ /var/lib/anna (owner: anna:anna, mode: 0770)
│     ✓ /var/log/anna (owner: anna:anna, mode: 0770)
│     ✓ /run/anna (owner: anna:anna, mode: 0770)
│     ✓ /etc/anna (owner: root:root, mode: 0755)
│     ✓ Disk usage: /var/lib/anna: 14.2 MB (healthy)
│
│  ✓  [3/8] Permissions Correct
│     ✓ User 'anna' exists (UID: 987)
│     ✓ Group 'anna' exists (GID: 987)
│     ✓ lhoqvso is member of 'anna' group
│     ✓ File permissions: all correct
│
│  ✓  [4/8] Daemon Running
│     ✓ systemctl status: active (running)
│     ✓ Process ID: 5678
│     ✓ Uptime: 3d 12h 47m
│     ✓ Memory usage: 19.4 MB (well within 100 MB limit)
│     ✓ CPU usage: 0.3% (idle)
│     ✓ Thread count: 8 (optimal)
│
│  ✓  [5/8] Socket Responding
│     ✓ Socket exists: /run/anna/annad.sock
│     ✓ Socket permissions: srwxrwx--- (correct)
│     ✓ Connection test: success (3ms)
│     ✓ RPC test: {"method":"ping"} → "pong" (8ms)
│
│  ✓  [6/8] Database Integrity
│     ✓ Database file: /var/lib/anna/telemetry.db (14.1 MB)
│     ✓ SQLite version: 3.43.0
│     ✓ PRAGMA integrity_check: ok
│     ✓ PRAGMA foreign_key_check: ok
│     ✓ Sample count: 2,341 (healthy)
│     ✓ Oldest sample: 7d ago (within retention: 90d)
│     ✓ Index health: optimal
│     ✓ WAL journal: 234 KB (normal)
│
│  ✓  [7/8] Configuration Valid
│     ✓ Config file: /etc/anna/config.toml
│     ✓ Syntax: valid TOML
│     ✓ Schema version: 4 (current)
│     ✓ Required keys: all present
│     ✓ Telemetry sample_rate: 30s (recommended: 15-60s)
│     ✓ Autonomy tier: observer (safe default)
│
│  ⚠  [8/8] Telemetry Queue Depth
│     ⚠ Queue depth: 87 events (threshold: 50)
│     ⚠ Processing rate: 12 events/sec (normally 30/sec)
│     ✓ Oldest event: 8s ago (not stale)
│     ✓ Memory pressure: low (19.4 MB / 100 MB limit)
│     ℹ Root cause: Temporary CPU spike detected
│     ℹ Self-healing: Queue will drain in ~7s
│
│  📊 Overall Health: 8.5/10.0
│
│  🎯 Health Breakdown:
│     • Core Infrastructure: 10.0/10.0 (perfect)
│     • Data Layer: 10.0/10.0 (perfect)
│     • Runtime Performance: 7.0/10.0 (queue backlog)
│
│  📈 Performance Metrics:
│     • RPC p50: 7ms, p95: 14ms, p99: 23ms
│     • Database queries: 234 total, 12ms avg
│     • Memory efficiency: 19.4 MB (excellent)
│     • CPU time: 3.2 hours (over 3d uptime = 0.3% avg)
│
│  ⚠️  Issues: 1 non-critical
│
│  📋 Repair Plan:
│     1. Process telemetry queue backlog
│        • Action: Spawn additional worker threads
│        • Expected duration: 7-10s
│        • Risk: None (memory sufficient)
│     2. Adjust collection rate temporarily
│        • Action: Slow sample rate to 45s for 5 minutes
│        • Expected impact: Reduced queue pressure
│        • Auto-restore: Yes (after queue stabilizes)
│
│  💡 Run: annactl doctor repair --yes
│
╰─────────────────────────────────────────────────────────────╯

🔬 Full diagnostic log: /var/log/anna/doctor-2025-11-03-183400.log
```

### Scenario 3: Auto-Repair with Detailed Progress

```bash
$ annactl doctor repair --yes
```

```
╭─────────────────────────────────────────────────────────────╮
│  🩹  Anna Doctor — Automated Repair                          │
╰─────────────────────────────────────────────────────────────╯

│
│  📋 Repair Plan: 2 actions
│     1. Process telemetry queue backlog (est. 7-10s)
│     2. Adjust collection rate temporarily (est. 2s)
│
│  ⏳  [Action 1/2] Process Telemetry Queue Backlog
│     ⏳ Spawning 2 additional worker threads...
│     ✓ Workers started (thread IDs: 42, 43)
│     ⏳ Processing backlog...
│        Queue: 87 → 72 → 58 → 41 → 28 → 12 → 4 → 0 events
│        [████████████████████████████████████████] 100%
│     ✓ Backlog cleared (8.3s)
│     ✓ Workers terminated gracefully
│     ⏳ Analyzing root cause...
│     ℹ Detection: Temporary CPU spike (core 3: 98% for 12s)
│     ℹ Trigger: User process 'ffmpeg' (PID: 8234)
│     ℹ Resolution: Spike ended, queue now stable
│
│  ⏳  [Action 2/2] Adjust Collection Rate
│     ⏳ Temporarily slowing sample rate: 30s → 45s
│     ✓ Config updated: /etc/anna/config.toml
│     ⏳ Sending SIGHUP to daemon...
│     ✓ Daemon reloaded configuration (no restart needed)
│     ℹ Auto-restore scheduled: 5 minutes
│     ℹ Watchdog active: Will restore to 30s if queue remains stable
│
│  🏥  Post-Repair Health Check
│     ✓ Queue depth: 0 events (excellent)
│     ✓ Processing rate: 30 events/sec (restored)
│     ✓ RPC latency: 7ms (optimal)
│     ✓ Memory: 21.1 MB (slight increase, normal after backlog)
│     ✓ Overall health: 9.8/10.0 (excellent)
│
│  🎉  Repair Complete!
│     • Duration: 10.4s
│     • Actions taken: 2/2 (100% success)
│     • System health: Restored to optimal
│     • Downtime: 0s (live repair)
│
│  📊 Before → After:
│     • Queue depth: 87 → 0 events
│     • Health score: 8.5 → 9.8 / 10.0
│     • RPC latency: 23ms → 7ms (p99)
│
│  🔍 Root Cause Summary:
│     • Trigger: Temporary CPU spike (user process)
│     • Detection: Telemetry queue threshold (50 events)
│     • Resolution: Backlog processed + rate adjusted
│     • Prevention: Watchdog monitoring for 5 minutes
│
╰─────────────────────────────────────────────────────────────╯

✨ Anna says: "All systems nominal. The hiccup has been resolved."

📄 Full repair log: /var/log/anna/repair-2025-11-03-183410.log
```

---

## 🌈 Bonus: Live Monitoring with `annactl watch`

### Real-Time Status Dashboard

```bash
$ annactl status --watch
```

```
╭───────────────────────────────────────────────────────────────────────╮
│  🌊  Anna System Status — Live Watch Mode                             │
│  Update: 2s   •   Iteration: 47   •   Elapsed: 1m 34s                 │
╰───────────────────────────────────────────────────────────────────────╯

│
│  ✅ Daemon: active (running)
│  PID: 5678    Uptime: 3d 12h 51m
│  RPC p99: 8 ms   Memory: 21.1 MB   Queue: 0 events
│  Journal: 0 errors, 2 warnings
│
│  System seems calm and healthy.
│
╰───────────────────────────────────────────────────────────────────────╯

  Press Ctrl+C to exit watch mode
```

---

## 🎨 Design Principles

### Visual Language

1. **Box Drawing Characters**
   - Rounded boxes (`╭─╮ ╰─╯`) for major sections
   - Tree borders (`┌─ │ └─`) for phase progression
   - Double lines (`╔═╗ ╚═╝`) for ceremonial moments (install complete, upgrade success)

2. **Unicode Symbols with Fallbacks**
   - `✓ ✗ ⚠ → ⏳` for status (checkmark, cross, warning, arrow, hourglass)
   - `🤖 🔧 📦 🩺 🔍` for context (robot, wrench, package, stethoscope, magnifying glass)
   - ASCII fallback: `[OK] [FAIL] [WARN] -> ...`

3. **Color Palette (Pastel for Dark Terminals)**
   - Green: Success, healthy states
   - Yellow: Warnings, non-critical issues
   - Red: Errors, critical issues
   - Cyan: Headers, section titles
   - Dim gray: Metadata, timestamps
   - Bold white: Key values

4. **Progress Indicators**
   - Animated spinners: `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`
   - Progress bars: `[████████░░░░░░░░░░] 40%`
   - Live counters: `Queue: 87 → 72 → 58...`

### Personality

- **Calm Competence**: Never chatty, always professional
- **Transparency**: Show what's happening, why, and how long
- **Empathy**: "Thank you for your patience" during repairs
- **Celebration**: "🎉 Repair Complete!" for successes
- **Reassurance**: "All systems nominal" after recovery

### Information Density

- **Scannable**: Eye can quickly jump to status symbols
- **Hierarchical**: Sections > subsections > details
- **Contextual**: Show metadata only when useful (timestamps, PIDs, sizes)
- **Actionable**: Always suggest next steps

---

## 🚀 Implementation Status

| Scenario | Status | File |
|----------|--------|------|
| Installation Wizard | ✅ Implemented | `scripts/install.sh` |
| Crash Recovery | ⏳ Planned | TBD |
| Update Rollout | ⏳ Planned | TBD |
| Self-Healing Diagnostics | ⏳ Planned | `src/annactl/src/doctor_cmd.rs` |
| Live Watch Mode | ✅ Implemented | `src/annactl/src/watch_mode.rs` |

---

**End of mockups.** This is Anna's visual language — **beauty in every byte**.
