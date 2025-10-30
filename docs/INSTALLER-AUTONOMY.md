# Installer & Autonomy System Guide

**Version**: 0.9.3-beta
**Sprint**: 4 - Autonomy & Self-Healing Architecture
**Last Updated**: 2025-01-30

## Overview

This document describes Anna's intelligent installer, version management, autonomy system, and self-healing architecture. These systems enable Anna to maintain herself without external dependencies.

## Table of Contents

1. [Installation System](#installation-system)
2. [Version Management](#version-management)
3. [Autonomy System](#autonomy-system)
4. [Doctor System](#doctor-system)
5. [Backup & Rollback](#backup--rollback)
6. [Privilege Model](#privilege-model)
7. [Logging Infrastructure](#logging-infrastructure)
8. [Safety Mechanisms](#safety-mechanisms)

---

## Installation System

### Intelligent Installer

The installer (`scripts/install.sh`) detects existing installations and handles upgrades intelligently:

```bash
./scripts/install.sh              # Interactive install/upgrade
./scripts/install.sh --yes        # Auto-approve upgrade prompts
```

### Installation Modes

The installer operates in three modes:

1. **Fresh Install** - No previous version detected
   - Creates all directories and files
   - Initializes config with defaults
   - Sets up systemd service
   - Runs initial doctor check

2. **Upgrade** - Older version detected
   - Preserves existing config and state
   - Creates backup before upgrade
   - Updates binaries and policies
   - Prompts for confirmation (unless `--yes`)

3. **Skip** - Same or newer version detected
   - Reports current version
   - Exits without changes
   - Safe to re-run anytime

### Version Detection Flow

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Check /etc/anna/version                                  │
├─────────────────────────────────────────────────────────────┤
│ Not found → FRESH INSTALL                                   │
│ Found → Compare versions                                    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Semantic Version Comparison                              │
├─────────────────────────────────────────────────────────────┤
│ Strip -alpha/-beta suffixes                                 │
│ Split into major.minor.patch                                │
│ Compare numerically                                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────┬───────────────────┬────────────────────────┐
│ Installed < New │ Installed == New  │ Installed > New        │
├─────────────────┼───────────────────┼────────────────────────┤
│ UPGRADE MODE    │ SKIP MODE         │ ERROR (no downgrade)   │
│ Prompt user     │ Exit cleanly      │ Abort installation     │
└─────────────────┴───────────────────┴────────────────────────┘
```

### Directory Structure

The installer creates this hierarchy:

```
/etc/anna/
├── config.toml           # Main configuration (0640 root:anna)
├── autonomy.conf         # Autonomy level (0644 root:anna)
├── version               # Installed version (0644 root:anna)
└── policies.d/           # Policy files (0750 root:anna)
    ├── 00-core.yml
    └── 01-dangerous.yml

/var/lib/anna/
├── state/                # Persistent state (0750 root:anna)
│   └── state.json
└── backups/              # Backup snapshots (0750 root:anna)
    └── repair-20250130-143022/
        ├── manifest.json
        ├── config.toml
        ├── autonomy.conf
        └── version

/var/log/anna/            # Log files (0750 root:anna)
├── install.log           # Installation history (0660 root:anna)
├── doctor.log            # Repair operations (0660 root:anna)
└── autonomy.log          # Autonomy changes (0660 root:anna)

/run/anna/                # Runtime files (0755 root:anna)
└── annad.sock            # RPC socket (0770 root:anna)
```

### Permissions Model

| Path                  | Owner      | Mode  | Reason                          |
|-----------------------|------------|-------|---------------------------------|
| `/etc/anna/`          | root:anna  | 0750  | Config directory                |
| `config.toml`         | root:anna  | 0640  | Sensitive settings              |
| `autonomy.conf`       | root:anna  | 0644  | User-readable autonomy level    |
| `policies.d/`         | root:anna  | 0750  | Policy directory                |
| `*.yml`               | root:anna  | 0640  | Policy files                    |
| `/var/lib/anna/`      | root:anna  | 0750  | State directory                 |
| `state/state.json`    | root:anna  | 0660  | Persistent state                |
| `backups/`            | root:anna  | 0750  | Backup directory                |
| `/var/log/anna/`      | root:anna  | 0750  | Log directory                   |
| `*.log`               | root:anna  | 0660  | Log files (group writable)      |
| `/run/anna/`          | root:anna  | 0755  | Runtime directory               |
| `annad.sock`          | root:anna  | 0770  | RPC socket                      |

---

## Version Management

### Version File Format

`/etc/anna/version` contains a single line:

```
0.9.3-beta
```

This file is:
- Written by installer on fresh install
- Updated by installer on upgrade
- Read by `annactl version` and doctor system
- Used for upgrade/skip decisions

### Version Comparison Algorithm

The installer uses semantic versioning comparison:

```bash
compare_versions() {
    local v1="$1"  # e.g., "0.9.2"
    local v2="$2"  # e.g., "0.9.3-beta"

    # Strip suffixes: "0.9.3-beta" → "0.9.3"
    local v1_base=$(echo "$v1" | sed 's/-.*$//')
    local v2_base=$(echo "$v2" | sed 's/-.*$//')

    # Split into arrays: [0, 9, 3]
    IFS='.' read -ra V1 <<< "$v1_base"
    IFS='.' read -ra V2 <<< "$v2_base"

    # Compare major
    if [[ ${V1[0]} -lt ${V2[0]} ]]; then
        return 0  # v1 < v2 (upgrade available)
    elif [[ ${V1[0]} -gt ${V2[0]} ]]; then
        return 2  # v1 > v2 (downgrade not allowed)
    fi

    # Compare minor, then patch...
    # Returns: 0=upgrade, 1=equal, 2=downgrade
}
```

### Upgrade Workflow

```
User runs: ./scripts/install.sh
                │
                ▼
┌───────────────────────────────────────────────┐
│ detect_version() reads /etc/anna/version     │
│ Installed: 0.9.2                              │
│ Bundle:    0.9.3-beta                         │
└───────────────────────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────┐
│ compare_versions "0.9.2" "0.9.3-beta"        │
│ Result: 0 (upgrade available)                │
└───────────────────────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────┐
│ Prompt user:                                  │
│ "Upgrade available: 0.9.2 → 0.9.3-beta"      │
│ "Would you like to upgrade? [Y/n]"           │
└───────────────────────────────────────────────┘
                │ (user confirms or --yes flag)
                ▼
┌───────────────────────────────────────────────┐
│ create_backup("upgrade")                      │
│ → /var/lib/anna/backups/upgrade-20250130...  │
└───────────────────────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────┐
│ Install new binaries                          │
│ Update policies if needed                     │
│ Preserve config.toml                          │
│ Write new version file                        │
└───────────────────────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────┐
│ Restart service: systemctl restart annad     │
│ Run doctor check                              │
│ Log completion                                │
└───────────────────────────────────────────────┘
```

---

## Autonomy System

### Overview

Anna's autonomy system controls what operations she can perform without explicit user approval. This is critical for self-healing while maintaining safety.

### Autonomy Levels

#### **Low Autonomy** (Default)

Safe, low-risk operations:
- ✓ Fix directory permissions
- ✓ Restart annad service
- ✓ Repair socket ownership
- ✓ Reload policy files
- ✓ Clear event history
- ✓ Create/restore backups
- ✗ Install packages (requires High)
- ✗ Modify system configs (requires High)
- ✗ Update policies automatically (requires High)

#### **High Autonomy** (Explicit Opt-In)

All low-risk operations plus:
- ✓ Install missing dependencies (sudo pacman/apt)
- ✓ Modify system configuration files
- ✓ Update polkit policies automatically
- ✓ Change autonomy level
- ⚠ All actions logged to audit.log

### Autonomy Configuration

File: `/etc/anna/autonomy.conf`

```ini
autonomy_level=low
last_changed=2025-01-30T14:30:22-05:00
changed_by=lhoqvso
```

### Commands

```bash
# Check current autonomy level
annactl autonomy get

# Change autonomy level (with confirmation)
annactl autonomy set high

# Change autonomy level (skip confirmation)
annactl autonomy set low --yes
```

### Example Output

```
$ annactl autonomy get

🔐 Anna Autonomy Status

Current Level: LOW
Description:   Low-risk autonomy: self-repair, permission fixes, service restarts

Capabilities:
  ✓ Fix directory permissions
  ✓ Restart annad service
  ✓ Repair socket ownership
  ✓ Reload policy files
  ✓ Clear event history
  ✓ Create/restore backups
  ✗ Install packages (requires High)
  ✗ Modify system configs (requires High)
  ✗ Update policies automatically (requires High)

Last changed:  2025-01-30T14:30:22-05:00 by lhoqvso
```

### Changing Autonomy Level

```
$ annactl autonomy set high

⚠️  Changing Autonomy Level

Current: low → New: high

High-risk autonomy: package installation, config updates, policy changes

⚠️  HIGH-RISK AUTONOMY WARNING ⚠️

This allows Anna to:
  • Install system packages automatically
  • Modify configuration files
  • Update security policies

Do you want to continue? [y/N]: y

✓ Autonomy level changed to: high
```

### Autonomy Decision Tree

When Anna encounters an operation:

```
Operation requested (e.g., "restart service")
        │
        ▼
┌─────────────────────────────────┐
│ Check operation risk level      │
├─────────────────────────────────┤
│ Low-risk:  permission fixes     │
│ High-risk: package installs     │
└─────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────┐
│ Read /etc/anna/autonomy.conf    │
│ Current level: low or high?     │
└─────────────────────────────────┘
        │
        ├─ Low autonomy ──┐
        │                 ▼
        │     ┌───────────────────────────┐
        │     │ Operation is low-risk?    │
        │     ├───────────────────────────┤
        │     │ Yes → Execute             │
        │     │ No  → Skip with warning   │
        │     └───────────────────────────┘
        │
        └─ High autonomy ─┐
                          ▼
              ┌───────────────────────────┐
              │ Execute operation         │
              │ Log to autonomy.log       │
              └───────────────────────────┘
```

---

## Doctor System

### Overview

The doctor system provides standalone diagnostics and self-healing that works even when the daemon is broken.

### Commands

```bash
# Read-only health check
annactl doctor check
annactl doctor check --verbose

# Perform repairs
annactl doctor repair
annactl doctor repair --dry-run

# List available backups
annactl doctor rollback list

# Restore from backup
annactl doctor rollback 20250130-143022

# Verify backup integrity without restoring
annactl doctor rollback --verify 20250130-143022
```

### Health Checks

The doctor performs these checks:

1. **Directories** - Verify all required directories exist
2. **Ownership** - Check `root:anna` ownership
3. **Permissions** - Verify 0750/0640/0660 modes
4. **Dependencies** - Check sudo, systemctl, journalctl
5. **Service** - Verify annad is active
6. **Socket** - Check `/run/anna/annad.sock` exists
7. **Policies** - Verify ≥2 policy files loaded
8. **Events** - Check bootstrap events recorded

### Example Check Output

```
$ annactl doctor check

🏥 Anna System Health Check

[OK] Directories present and accessible
[OK] Ownership correct (root:anna)
[OK] Permissions correct (0750/0640/0660)
[OK] Dependencies installed (3/3)
[OK] Service active (annad)
[OK] Socket reachable (/run/anna/annad.sock)
[OK] Policies loaded (2 rules)
[OK] Events functional (3 bootstrap events)

✓ System healthy - no repairs needed
```

### Repair Operations

The doctor can automatically fix:

1. **Missing Directories**
   ```
   [HEAL] Creating directory: /var/lib/anna/state
   ```

2. **Wrong Ownership**
   ```
   [HEAL] Fixing ownership: /etc/anna → root:anna
   ```

3. **Wrong Permissions**
   ```
   [HEAL] Fixing permissions: /etc/anna → 0750
   ```

4. **Inactive Service**
   ```
   [HEAL] Restarting service: annad
   ```

### Example Repair Output

```
$ annactl doctor repair

🔧 Doctor Repair

[BACKUP] Creating backup: /var/lib/anna/backups/repair-20250130-143022
[BACKUP] Created manifest with 3 files
[HEAL] Creating directory: /var/lib/anna/state
[HEAL] Fixing ownership: /var/lib/anna/state
[HEAL] Restarting service: annad

✓ Made 3 repairs successfully
```

### Dry-Run Mode

Preview repairs without making changes:

```
$ annactl doctor repair --dry-run

🔍 Doctor Repair (Dry-Run Mode)

[DRY-RUN] Would create: /var/lib/anna/state
[DRY-RUN] Would fix ownership: /etc/anna
[DRY-RUN] Would restart service: annad

Would make 3 repairs
```

---

## Backup & Rollback

### Backup System

Anna creates automatic backups before risky operations:
- Before upgrades
- Before repairs
- Before policy changes (High autonomy)

### Backup Structure

Each backup is a timestamped directory:

```
/var/lib/anna/backups/repair-20250130-143022/
├── manifest.json       # Metadata and checksums
├── config.toml         # Backed up config
├── autonomy.conf       # Backed up autonomy settings
└── version             # Backed up version file
```

### Manifest Format

`manifest.json` contains:

```json
{
  "version": "0.9.3-beta",
  "created": "2025-01-30T14:30:22-05:00",
  "trigger": "repair",
  "files": [
    {
      "path": "/etc/anna/config.toml",
      "sha256": "a3f8c9d2e1b4f6a7...",
      "size": 1024
    },
    {
      "path": "/etc/anna/autonomy.conf",
      "sha256": "7b2e1d4c9a8f3e6b...",
      "size": 128
    },
    {
      "path": "/etc/anna/version",
      "sha256": "2d9f7a3c8e1b5d4a...",
      "size": 12
    }
  ]
}
```

### Listing Backups

```
$ annactl doctor rollback list

📦 Available Backups

  repair-20250130-143022
  upgrade-20250129-091530
  repair-20250128-163045
```

### Verifying Backups

Check integrity without restoring:

```
$ annactl doctor rollback --verify 20250130-143022

🔍 Verifying backup: 20250130-143022

[VERIFY] OK: /etc/anna/config.toml
[VERIFY] OK: /etc/anna/autonomy.conf
[VERIFY] OK: /etc/anna/version

✓ Backup integrity verified
```

### Verification Process

```
For each file in manifest:
    1. Check file exists in backup
    2. Read file content
    3. Calculate SHA-256 hash
    4. Compare hash with manifest
    5. Compare size with manifest
    6. Report: OK, Missing, or Mismatch
```

### Rollback Workflow

```
$ annactl doctor rollback 20250130-143022

⏮  Rolling back to backup: 20250130-143022

[VERIFY] Checking backup integrity...
[VERIFY] OK: /etc/anna/config.toml
[VERIFY] OK: /etc/anna/autonomy.conf
[VERIFY] OK: /etc/anna/version
[ROLLBACK] Restoring: /etc/anna/config.toml
[ROLLBACK] Restoring: /etc/anna/autonomy.conf
[ROLLBACK] Restoring: /etc/anna/version

✓ Rollback complete - 3 files restored
```

### Safety Features

1. **Always verify before restore** - Prevents corrupted rollbacks
2. **Manifest checksums** - Detects tampering or corruption
3. **Size validation** - Additional integrity check
4. **Read-only verify mode** - Test without changing system
5. **Detailed logging** - Audit trail of all operations

---

## Privilege Model

### Sudo Usage

Anna uses `sudo` for privileged operations with these principles:

1. **Minimal Scope** - Only elevate when necessary
2. **Explicit Commands** - Never `sudo bash -c "..."`
3. **User Confirmation** - Prompt for risky operations
4. **Audit Logging** - Log all elevated operations

### Privileged Operations

| Operation                  | Requires Sudo | Autonomy Level |
|----------------------------|---------------|----------------|
| Read config files          | No            | N/A            |
| Write to `/tmp`            | No            | N/A            |
| Create `/etc/anna/` files  | Yes           | Low            |
| Modify permissions         | Yes           | Low            |
| Restart service            | Yes           | Low            |
| Install packages           | Yes           | High           |
| Update policies            | Yes           | High           |

### Privilege Escalation Pattern

Anna uses this pattern for privileged writes:

```rust
fn write_autonomy_level(level: &AutonomyLevel) -> Result<()> {
    let content = format!("autonomy_level={}\n", level.as_str());

    // 1. Write to temp file (no sudo)
    let temp_file = "/tmp/anna-autonomy.conf";
    fs::write(temp_file, &content)?;

    // 2. Copy with sudo
    Command::new("sudo")
        .args(&["cp", temp_file, "/etc/anna/autonomy.conf"])
        .status()?;

    // 3. Set ownership with sudo
    Command::new("sudo")
        .args(&["chown", "root:anna", "/etc/anna/autonomy.conf"])
        .status()?;

    // 4. Clean up temp file
    fs::remove_file(temp_file)?;

    Ok(())
}
```

This pattern:
- Minimizes privileged code
- Allows content verification before elevation
- Cleans up temporary files
- Maintains proper ownership/permissions

### Polkit Integration

Anna's polkit policy (`/etc/polkit-1/rules.d/50-anna.rules`) allows:

```javascript
polkit.addRule(function(action, subject) {
    if (action.id == "org.freedesktop.systemd1.manage-units" &&
        action.lookup("unit") == "annad.service" &&
        subject.isInGroup("anna")) {
        return polkit.Result.YES;
    }
});
```

This allows `anna` group members to restart the service without password.

---

## Logging Infrastructure

### Log Files

Anna maintains three log files:

1. **`/var/log/anna/install.log`** - Installation and upgrade history
2. **`/var/log/anna/doctor.log`** - Repair operations
3. **`/var/log/anna/autonomy.log`** - Autonomy level changes

### Log Format

All logs use this format:

```
[YYYY-MM-DD HH:MM:SS] [LEVEL] Message
```

Example entries:

```
[2025-01-30 14:30:22] [INSTALL] Starting installation: v0.9.3-beta
[2025-01-30 14:30:45] [UPDATE] Upgrading from 0.9.2 to 0.9.3-beta
[2025-01-30 14:31:10] [HEAL] Restarted service: annad
[2025-01-30 14:31:25] [ESCALATED] Autonomy level changed to 'high' by user lhoqvso
```

### Log Levels

- **`[INSTALL]`** - Fresh installation operations
- **`[UPDATE]`** - Upgrade operations
- **`[HEAL]`** - Repair operations
- **`[ESCALATED]`** - High-risk autonomy changes
- **`[VERIFY]`** - Backup integrity checks
- **`[ROLLBACK]`** - Restore operations
- **`[ERROR]`** - Failures and errors
- **`[WARN]`** - Non-fatal warnings

### Logging Functions

The installer uses these helper functions:

```bash
log_install() {
    echo -e "${CYAN}[INSTALL]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INSTALL] $1" >> "$LOG_DIR/install.log"
}

log_update() {
    echo -e "${YELLOW}[UPDATE]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [UPDATE] $1" >> "$LOG_DIR/install.log"
}

log_heal() {
    echo -e "${GREEN}[HEAL]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [HEAL] $1" >> "$LOG_DIR/doctor.log"
}
```

### Log Rotation

Currently, logs grow unbounded. Future implementation will:
- Rotate when log files exceed 1MB
- Keep up to 5 rotated logs (`install.log.1` through `.5`)
- Compress old logs with gzip

Planned rotation logic:

```bash
rotate_log() {
    local log_file="$1"
    local max_size=$((1024 * 1024))  # 1MB

    if [[ -f "$log_file" ]] && [[ $(stat -f%z "$log_file") -gt $max_size ]]; then
        # Rotate: .4→.5, .3→.4, .2→.3, .1→.2, current→.1
        for i in 4 3 2 1; do
            [[ -f "$log_file.$i" ]] && mv "$log_file.$i" "$log_file.$((i+1))"
        done
        mv "$log_file" "$log_file.1"
        touch "$log_file"
        chown root:anna "$log_file"
        chmod 0660 "$log_file"
    fi
}
```

---

## Safety Mechanisms

### Idempotency

All Anna operations are idempotent - safe to run multiple times:

- **Installer**: Detects existing installation and skips/upgrades
- **Doctor**: Checks before repairing, only fixes what's broken
- **Autonomy**: Allows setting same level without errors
- **Policies**: Reloading policies is safe

### Backup Before Risky Operations

Anna creates backups before:
- Upgrades (installer)
- Repairs (doctor)
- Policy updates (High autonomy)
- Configuration changes (future)

### Verification Before Rollback

Rollback always verifies backup integrity:
1. Read manifest.json
2. Check all files exist
3. Verify SHA-256 checksums
4. Verify file sizes
5. Only restore if all checks pass

This prevents restoring corrupted backups.

### Confirmation Prompts

Risky operations prompt for confirmation:

- **Autonomy escalation to High**: "Do you want to continue? [y/N]"
- **Upgrade installation**: "Would you like to upgrade? [Y/n]"
- **Rollback restore**: (Future) "This will overwrite current config. Continue? [y/N]"

Use `--yes` flag to skip prompts for automation.

### Audit Logging

All privileged operations are logged:
- Autonomy changes → `/var/log/anna/autonomy.log`
- Repairs → `/var/log/anna/doctor.log`
- Upgrades → `/var/log/anna/install.log`

Logs include:
- Timestamp
- Operation type
- User who triggered it
- Success/failure status

### No Downgrade Support

The installer prevents downgrades:

```
$ ./scripts/install.sh  # Bundle: 0.9.2, Installed: 0.9.3-beta

✗ Cannot downgrade: installed version (0.9.3-beta) is newer than bundle (0.9.2)
```

This prevents accidental version rollbacks that could break state.

### Dry-Run Mode

Doctor repairs support `--dry-run`:

```bash
annactl doctor repair --dry-run
```

Shows what would be repaired without making changes.

---

## Troubleshooting

### Installer Issues

**Problem**: Installer reports "Cannot downgrade"
- **Solution**: You're trying to install an older version. Use rollback instead:
  ```bash
  annactl doctor rollback list
  annactl doctor rollback <timestamp>
  ```

**Problem**: Installer hangs at "Would you like to upgrade?"
- **Solution**: Answer `y` or `n`, or use `--yes` flag for automation

**Problem**: Permission denied errors during install
- **Solution**: Installer uses sudo internally. Ensure your user can sudo

### Doctor Issues

**Problem**: Doctor check reports failures but repair does nothing
- **Solution**: Some issues (like missing dependencies) can't be auto-repaired.
  Install manually: `sudo pacman -S <package>`

**Problem**: Rollback fails with "Manifest not found"
- **Solution**: Backup is corrupted or incomplete. List backups and try another:
  ```bash
  annactl doctor rollback list
  ```

**Problem**: Verification fails with checksum mismatch
- **Solution**: Backup is corrupted. Use `--verify` on other backups to find a good one

### Autonomy Issues

**Problem**: "Permission denied" when setting autonomy level
- **Solution**: Changing autonomy requires sudo. Command will prompt for password

**Problem**: Operations still blocked despite High autonomy
- **Solution**: Some operations always require explicit confirmation.
  Use `--yes` flags or check daemon logs

---

## Examples

### Fresh Installation

```bash
$ ./scripts/install.sh

╔═══════════════════════════════════════════════════════╗
║                                                       ║
║   ANNA ASSISTANT v0.9.3-beta                          ║
║   Autonomous Neural Network Agent                     ║
║                                                       ║
║   Sprint 4: Autonomy & Self-Healing Architecture      ║
║                                                       ║
╚═══════════════════════════════════════════════════════╝

[INSTALL] Starting installation: v0.9.3-beta
[INSTALL] Checking installed version...
[INSTALL] Fresh installation mode
[INSTALL] Creating directories...
[INSTALL] Installing binaries...
[INSTALL] Configuring system...
[INSTALL] Starting service...
[INSTALL] Running health check...

✓ Installation complete: v0.9.3-beta
```

### Upgrade Installation

```bash
$ ./scripts/install.sh

[INSTALL] Checking installed version...
[UPDATE] Installed: v0.9.2
[UPDATE] Bundle:    v0.9.3-beta

Upgrade available: 0.9.2 → 0.9.3-beta
Would you like to upgrade? [Y/n]: y

[BACKUP] Creating backup: /var/lib/anna/backups/upgrade-20250130-143022
[UPDATE] Upgrading binaries...
[UPDATE] Updating policies...
[UPDATE] Restarting service...

✓ Upgrade complete: v0.9.3-beta
```

### Health Check and Repair

```bash
$ annactl doctor check --verbose

🏥 Anna System Health Check

✓ Directory: /run/anna
✓ Directory: /etc/anna
✓ Directory: /etc/anna/policies.d
✗ Directory: /var/lib/anna/state
✓ Dependency: sudo
✓ Dependency: systemctl
✓ Dependency: journalctl
[FAIL] Service inactive (annad)

⚠ Some checks failed - run `annactl doctor repair` to fix

$ annactl doctor repair

🔧 Doctor Repair

[BACKUP] Creating backup: /var/lib/anna/backups/repair-20250130-143022
[HEAL] Creating directory: /var/lib/anna/state
[HEAL] Restarting service: annad

✓ Made 2 repairs successfully
```

### Autonomy Management

```bash
$ annactl autonomy get

🔐 Anna Autonomy Status

Current Level: LOW
Description:   Low-risk autonomy: self-repair, permission fixes, service restarts

Capabilities:
  ✓ Fix directory permissions
  ✓ Restart annad service
  ✓ Repair socket ownership
  ✗ Install packages (requires High)

$ annactl autonomy set high --yes

✓ Autonomy level changed to: high
```

### Backup and Rollback

```bash
$ annactl doctor rollback list

📦 Available Backups

  repair-20250130-143022
  upgrade-20250129-091530

$ annactl doctor rollback --verify repair-20250130-143022

🔍 Verifying backup: repair-20250130-143022

[VERIFY] OK: /etc/anna/config.toml
[VERIFY] OK: /etc/anna/autonomy.conf
[VERIFY] OK: /etc/anna/version

✓ Backup integrity verified

$ annactl doctor rollback repair-20250130-143022

⏮  Rolling back to backup: repair-20250130-143022

[VERIFY] Checking backup integrity...
[ROLLBACK] Restoring: /etc/anna/config.toml
[ROLLBACK] Restoring: /etc/anna/autonomy.conf
[ROLLBACK] Restoring: /etc/anna/version

✓ Rollback complete - 3 files restored
```

---

## Future Enhancements

### Planned for Sprint 5+

1. **Log Rotation**
   - Automatic rotation at 1MB threshold
   - Keep 5 historical logs
   - Gzip compression for old logs

2. **Smart Repair**
   - Machine learning to predict failures
   - Proactive repairs before issues manifest
   - Trend analysis from telemetry

3. **Remote Backup**
   - Optional cloud backup integration
   - Encrypted backup storage
   - Multi-machine restore

4. **Rollback Previews**
   - Show diff of what will change
   - Interactive file-by-file restore
   - Selective restore (choose specific files)

5. **Autonomy Tiers**
   - Medium tier between Low/High
   - Fine-grained capability control
   - Time-limited escalation

6. **Self-Update**
   - Anna checks for new versions
   - Auto-upgrade with High autonomy
   - Staged rollout support

---

## References

- **Architecture**: `docs/AUTONOMY-ARCHITECTURE.md` - Detailed design document
- **Installation**: `scripts/install.sh` - Installer source code
- **Doctor**: `src/annactl/src/doctor.rs` - Doctor system implementation
- **Autonomy**: `src/annactl/src/autonomy.rs` - Autonomy management
- **Changelog**: `CHANGELOG.md` - Version history

---

**Document Version**: 1.0
**Last Updated**: 2025-01-30
**Maintained By**: Anna Development Team
