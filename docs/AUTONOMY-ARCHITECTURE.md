# Anna Autonomy & Self-Healing Architecture

**Version**: 0.9.3-alpha
**Sprint**: 4 - Autonomy and Self-Healing
**Status**: Active Development

---

## Executive Summary

Anna Assistant is designed as a **self-healing, autonomous system** that can detect, diagnose, and repair its own issues without manual intervention. This document defines the architecture, privilege boundaries, repair logic, and safety mechanisms that enable Anna to maintain itself while respecting user control and security boundaries.

---

## Core Principles

### 1. Self-Sufficiency
Anna must never depend on external fixes. Any missing dependency, permission error, or configuration drift is recoverable through `annactl doctor repair`.

### 2. Non-Destructive Operation
All operations are:
- **Idempotent**: Can be run multiple times safely
- **Reversible**: Backups created before changes
- **Auditable**: Full logging of all actions

### 3. Graduated Autonomy
Users control Anna's autonomy level:
- **Low (default)**: Self-repair, service restart, permission fixes
- **High**: Package installation, config updates, policy changes

### 4. Explicit Privilege Escalation
Anna runs as a normal user but can temporarily escalate privileges through:
- `sudo` for immediate operations
- `polkit` for policy-controlled actions
- All escalations are logged with `[ESCALATED]` markers

---

## Autonomy Tiers

### Low-Risk Autonomy (Default)

**Scope**: Self-maintenance operations that don't modify system packages or critical configs.

**Allowed Actions**:
- Fix directory permissions (`/run/anna`, `/etc/anna`, `/var/lib/anna`)
- Restart `annad` service
- Repair socket ownership
- Reload policy files
- Clear event history
- Run telemetry diagnostics
- Create/restore backups

**Privilege Required**: Temporary `sudo` for specific commands

**User Confirmation**: None (automated)

### High-Risk Autonomy

**Scope**: System-level changes that affect packages, configs, or security policies.

**Allowed Actions**:
- Install missing dependencies (`pacman`, `apt`, `dnf`)
- Modify system configuration files
- Update polkit policies
- Change autonomy level itself
- Modify systemd service files

**Privilege Required**: `polkit` authentication

**User Confirmation**: Explicit prompt with Y/N confirmation

**Audit**: Detailed logs in `/var/log/anna/audit.log`

---

## Version Management

### Version File: `/etc/anna/version`

```
0.9.3-alpha
```

Simple, single-line version identifier.

### Installer Logic

```bash
BUNDLE_VERSION="0.9.3-alpha"
INSTALLED_VERSION=$(cat /etc/anna/version 2>/dev/null || echo "none")

if [[ "$INSTALLED_VERSION" == "none" ]]; then
    # Fresh install
    perform_install
elif [[ "$INSTALLED_VERSION" < "$BUNDLE_VERSION" ]]; then
    # Upgrade available
    offer_upgrade
elif [[ "$INSTALLED_VERSION" == "$BUNDLE_VERSION" ]]; then
    # Already up to date
    exit_skip
else
    # Newer version installed (downgrade not supported)
    abort_safely
fi
```

### Upgrade Flow

1. Detect installed version
2. Offer upgrade (or auto-upgrade with `--yes`)
3. Create backup: `/var/lib/anna/backups/pre-upgrade-$(date +%Y%m%d-%H%M%S)/`
4. Deploy new binaries
5. Run `annactl doctor repair`
6. Update `/etc/anna/version`
7. Emit `[READY]` log entry

---

## Doctor System

The `doctor` subsystem is Anna's self-healing engine.

### Commands

#### `annactl doctor check`

**Purpose**: Read-only diagnostics
**Privilege**: None required
**Output**: Detailed health report

**Validates**:
- ✓ Directories exist: `/run/anna`, `/etc/anna`, `/var/lib/anna`, `/var/log/anna`
- ✓ Ownership: `root:anna`
- ✓ Permissions: `0750` (dirs), `0640` (configs), `0660` (logs)
- ✓ Dependencies installed: `sudo`, `polkit`, `systemd`, `tmpfiles.d`
- ✓ Service active: `systemctl is-active annad`
- ✓ Socket reachable: `/run/anna/annad.sock`
- ✓ Policies loaded: `≥ 2 policy rules`
- ✓ Events functional: `≥ 3 bootstrap events`

**Output Format**:
```
🏥 Anna System Health Check

[OK] Directories present and accessible
[OK] Ownership correct (root:anna)
[OK] Permissions correct (0750/0640/0660)
[OK] Dependencies installed (4/4)
[OK] Service active (annad)
[OK] Socket reachable (/run/anna/annad.sock)
[OK] Policies loaded (2 rules)
[OK] Events functional (3 bootstrap events)

✓ System healthy - no repairs needed
```

#### `annactl doctor repair`

**Purpose**: Self-healing with privilege escalation
**Privilege**: Temporary `sudo` or `polkit`
**Output**: Repair actions taken

**Repair Logic**:

1. **Missing Directories**: Create with correct ownership/perms
2. **Wrong Ownership**: `chown root:anna`
3. **Wrong Permissions**: `chmod 0750/0640/0660`
4. **Missing Dependencies**:
   - Low autonomy: Report error with install instructions
   - High autonomy: `sudo pacman -S --noconfirm <pkg>`
5. **Service Stopped**: `sudo systemctl restart annad`
6. **Socket Missing**: Wait 5s for service to create it
7. **Policies Not Loaded**: `annactl policy reload`

**Safety**:
- Backup created before each change: `/var/lib/anna/backups/repair-$(date +%Y%m%d-%H%M%S)/`
- All actions logged to `/var/log/anna/doctor.log`
- Failed repairs do not block subsequent checks
- Rollback available via `annactl doctor rollback <timestamp>`

---

## Logging Infrastructure

### Log Files

| Path | Purpose | Rotation |
|------|---------|----------|
| `/var/log/anna/install.log` | Installer actions | 10 files, 1MB each |
| `/var/log/anna/doctor.log` | Self-healing repairs | 10 files, 1MB each |
| `/var/log/anna/audit.log` | Privilege escalations | 50 files, 10MB each |
| `/var/log/anna/daemon.log` | Runtime events | Managed by systemd |

### Log Format

```
[2025-10-30 12:34:56] [INSTALL] Starting Anna v0.9.3-alpha installation
[2025-10-30 12:35:10] [ESCALATED] sudo chown root:anna /etc/anna
[2025-10-30 12:35:15] [HEAL] Fixed permissions: /etc/anna (0755 → 0750)
[2025-10-30 12:35:20] [READY] Anna Assistant operational
```

**Tags**:
- `[INSTALL]`: Installer actions
- `[HEAL]`: Doctor repairs
- `[ESCALATED]`: Privilege escalation
- `[READY]`: System operational
- `[ERROR]`: Failures requiring attention
- `[SKIP]`: Idempotent operation skipped

---

## Backup and Rollback System

### Backup Structure

```
/var/lib/anna/backups/
├── pre-upgrade-20251030-123456/
│   ├── manifest.json
│   ├── etc/anna/config.toml
│   ├── etc/anna/policies.d/
│   └── var/lib/anna/state/
├── repair-20251030-140000/
│   └── manifest.json
└── manual-20251030-150000/
```

### Manifest Format

```json
{
  "timestamp": "2025-10-30T12:34:56Z",
  "version_before": "0.9.2b-final",
  "version_after": "0.9.3-alpha",
  "trigger": "upgrade",
  "files": [
    "/etc/anna/config.toml",
    "/etc/anna/policies.d/error-rate.yml"
  ]
}
```

### Rollback

```bash
annactl doctor rollback 20251030-123456
```

Restores all files from the specified backup, updates version file, and restarts service.

---

## Privilege Boundaries

### No Privilege Required

- `annactl status`
- `annactl doctor check`
- `annactl policy list`
- `annactl events list`
- `annactl telemetry stats`
- `annactl learning stats`

### Temporary Sudo Required

- `annactl doctor repair` (low autonomy)
- Directory/permission fixes
- Service restart

### Polkit Authentication Required

- `annactl autonomy set high`
- `annactl doctor repair` (high autonomy with package install)
- Config file modifications
- Policy updates

---

## Unified Installation Flow

```
1. ./scripts/install.sh
   ├─ Detect installed version
   ├─ Compare with bundle version
   └─ Offer upgrade if needed

2. Deploy Binaries
   ├─ Compile: cargo build --release
   ├─ Install: /usr/local/bin/{annad,annactl}
   └─ Set permissions: 0755 root:root

3. Run Doctor Repair
   ├─ annactl doctor repair
   ├─ Fix directories, perms, ownership
   └─ Restart service if needed

4. Verify Autonomy
   ├─ Check autonomy level in config
   ├─ Default to 'low' if not set
   └─ Log current autonomy level

5. Sanity Tests
   ├─ annactl policy reload (expect ≥2)
   ├─ annactl events list (expect ≥3)
   └─ annactl telemetry stats (expect 3 fields)

6. Mark Ready
   ├─ Write /etc/anna/version
   ├─ Log [READY] entry
   └─ Display success banner
```

---

## Safety Mechanisms

### 1. Dry-Run Mode

```bash
annactl doctor repair --dry-run
```

Shows what would be fixed without making changes.

### 2. Backup Before Repair

Every repair operation creates a timestamped backup before making changes.

### 3. Audit Trail

All privilege escalations logged to `/var/log/anna/audit.log`:

```
[2025-10-30 12:35:10] [ESCALATED] Command: sudo chown root:anna /etc/anna
[2025-10-30 12:35:10] [ESCALATED] User: lhoqvso
[2025-10-30 12:35:10] [ESCALATED] Autonomy: low
[2025-10-30 12:35:10] [ESCALATED] Result: success
```

### 4. Autonomy Change Confirmation

```bash
$ annactl autonomy set high

⚠️  You are about to enable HIGH-RISK autonomy.

This allows Anna to:
  • Install system packages automatically
  • Modify configuration files
  • Update security policies

Do you want to continue? [y/N]: _
```

### 5. Rollback Always Available

```bash
annactl doctor rollback --list
annactl doctor rollback 20251030-123456
```

---

## Error Handling

### Missing Dependency (Low Autonomy)

```
[ERROR] Missing dependency: polkit

Anna cannot install packages in low-autonomy mode.

To fix this issue:
  1. Install manually: sudo pacman -S polkit
  2. Or enable high autonomy: annactl autonomy set high
  3. Then run: annactl doctor repair
```

### Permission Denied

```
[ERROR] Permission denied: /etc/anna/config.toml

Anna needs temporary privilege escalation to fix this.

Run with sudo: sudo annactl doctor repair
```

### Service Start Failure

```
[ERROR] Failed to start annad service

Diagnosis:
  • Socket already in use by another process
  • Run: lsof /run/anna/annad.sock
  • Kill conflicting process
  • Retry: annactl doctor repair
```

---

## Testing Strategy

### 10 New Validation Tests

1. **Version Detection**: Verify installer detects installed version correctly
2. **Fresh Install**: Test clean install on system without Anna
3. **Upgrade Path**: Test 0.9.2 → 0.9.3-alpha upgrade
4. **Skip Path**: Verify installer exits cleanly when already up-to-date
5. **Doctor Check**: Validate read-only diagnostic output
6. **Doctor Repair**: Test self-healing of broken permissions
7. **Missing Dependency**: Simulate missing `sudo` and verify error message
8. **Autonomy Change**: Test low → high autonomy upgrade flow
9. **Backup Creation**: Verify backups are created before repairs
10. **Rollback**: Test restoration from backup

### Test Harness

```bash
tests/runtime_validation.sh --sprint4
```

Runs all Sprint 4 tests in isolated environment.

---

## Future Enhancements (Sprint 5+)

1. **Distributed Healing**: Multiple Anna instances can coordinate repairs
2. **ML-Based Diagnostics**: Learn common failure patterns and predict issues
3. **Remote Management**: Web UI for monitoring Anna across multiple machines
4. **Policy-Driven Healing**: Custom repair policies defined in YAML
5. **Telemetry Integration**: Send health metrics to centralized monitoring

---

## Conclusion

Anna's autonomy architecture balances **self-sufficiency** with **user control**. By default, Anna can heal herself without user intervention, but respects privilege boundaries and requires explicit permission for high-risk operations. The result is a system that "just works" while remaining transparent and auditable.

**Key Takeaway**: Anna should never break, and when something does go wrong, she fixes herself.

---

**Document Version**: 1.0
**Last Updated**: 2025-10-30
**Author**: Anna Assistant Team
