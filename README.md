# Anna Assistant

**Autonomous Arch Linux System Administrator**

Anna is a security-hardened system administration daemon for Arch Linux. She provides state-aware command dispatch, comprehensive health monitoring, and Arch Wiki-cited operations.

**Current Version:** 1.0.0-rc.1 (November 2025)

**Status:** Operational Core - Active Development

---

## What Anna Does

Anna is a **minimal, auditable sysadmin core** focused on:

- **State-Aware Operations**: Commands adapt to system state (ISO live, recovery, fresh install, configured, degraded)
- **Health Monitoring**: Proactive system checks with Arch Wiki citations
- **Diagnostic Tools**: System health analysis and recovery planning
- **Comprehensive Logging**: Every action logged with citations to `/var/log/anna/`
- **Security First**: Systemd sandbox, strict permissions, no privilege escalation

**What Anna Is NOT:**
- ❌ Desktop environment manager (removed in 1.0 reset)
- ❌ Hyprland/i3/sway bundle installer (removed)
- ❌ TUI application (removed, returns in 2.0)
- ❌ Application recommender system (removed)

Anna is a **system administrator, not a user assistant**. One daemon, one socket, one truth.

---

## Quick Start

### Installation

```bash
# One-line install (recommended)
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash

# Or clone and build from source
git clone https://github.com/jjgarcianorway/anna-assistant.git
cd anna-assistant
cargo build --release
sudo cp target/release/{annad,annactl} /usr/local/bin/
sudo systemctl enable --now annad
```

### Basic Usage

```bash
# Check system status
annactl status

# Run health checks
annactl health

# Get diagnostic report
annactl doctor

# List recovery plans
annactl rescue list

# Show available commands
annactl help
```

### Auto-Update

Anna updates herself automatically:
- Daemon checks GitHub releases every 2 hours
- Downloads and installs updates automatically (runs as root)
- Sends desktop notification when updated
- Systemd restarts daemon after update

No manual intervention required. Updates happen transparently in the background.

---

## Commands

### System Status
```bash
annactl status              # Show daemon health and system state
annactl help                # List available commands for current state
annactl help --json         # JSON output for scripting
```

### Health Monitoring (Phase 0.5)
```bash
annactl health              # Run all health probes
annactl health --json       # JSON output with full details
annactl doctor              # Diagnostic report with recommendations
annactl doctor --json       # JSON diagnostic output
```

**Health Probes:**
- `disk-space`: Filesystem usage monitoring
- `pacman-db`: Package database integrity
- `systemd-units`: Failed unit detection
- `journal-errors`: System log analysis
- `services-failed`: Service health
- `firmware-microcode`: Microcode status

**Exit Codes:**
- `0` - All checks passed
- `1` - One or more failures detected
- `2` - Warnings detected (no failures)
- `64` - Command not available in current state
- `65` - Invalid daemon response
- `70` - Daemon unavailable

### Installation (Phase 0.8)
```bash
annactl install                # Interactive Arch Linux installation (iso_live only)
annactl install --dry-run      # Simulate installation without executing
```

**Installation Steps:**
1. Disk setup (manual partitioning)
2. Base system installation (pacstrap)
3. System configuration (fstab, locale, timezone)
4. Bootloader (systemd-boot or GRUB)
5. User creation with sudo access

**Requirements:**
- Must run as root
- Only available in iso_live state (Arch ISO environment)
- Network connectivity required for pacstrap

**Output Example:**
```
[anna] disk_setup — Partition, format, and mount disks (OK)
  formatted sda2 as ext4; formatted sda1 as FAT32; mounted root to /mnt
  Citation: [archwiki:Installation_guide#Partition_the_disks]
[anna] base_system — Install base packages with pacstrap (OK)
  installed 7 packages successfully
  Citation: [archwiki:Installation_guide#Install_essential_packages]
[anna] Installation complete!
[anna] Next steps:
  1. Reboot: umount -R /mnt && reboot
  2. Log in with created user credentials
  3. Change default passwords immediately
```

**Installation Log:**
All steps logged to `/var/log/anna/install.jsonl` with timestamps and Arch Wiki citations.

### Lifecycle Management (Phase 0.9)
```bash
annactl status                 # Comprehensive system health report
annactl update                 # Orchestrated system update with pacman
annactl update --dry-run       # Simulate update without executing
annactl audit                  # Security and integrity audit
```

**System Health (`status`):**
- Service status monitoring (failed, active, enabled)
- Package update detection via checkupdates
- System log analysis (journalctl errors)
- Actionable recommendations

**Update Orchestration (`update`):**
- Package updates via `pacman -Syu`
- Automatic service restart detection
- Package change tracking (old → new versions)
- Dry-run mode for risk-free preview

**Security Audit (`audit`):**
- Package integrity checks (`pacman -Qkk`)
- GPG keyring verification
- File permission validation (`/etc/passwd`, `/etc/shadow`, `/etc/sudoers`)
- Security baseline checks (firewall, SSH hardening)
- Configuration compliance (fstab mount options)

**Output Example:**
```
$ annactl status
┌─────────────────────────────────────────────────────────
│ SYSTEM HEALTH REPORT
├─────────────────────────────────────────────────────────
│ Status:    Healthy
│ Timestamp: 2025-11-11T17:00:00Z
│ State:     configured
├─────────────────────────────────────────────────────────
│ All critical services: OK
│ UPDATES AVAILABLE: 5
│   • linux 6.6.1 → 6.6.2
│   • systemd 255.1 → 255.2
│   ... and 3 more
├─────────────────────────────────────────────────────────
│ RECOMMENDATIONS:
│   • Updates available - run 'annactl update'
└─────────────────────────────────────────────────────────

[archwiki:System_maintenance]
```

**Steward Log:**
All lifecycle operations logged to `/var/log/anna/steward.jsonl` with timestamps and Arch Wiki citations.

### Sentinel Framework (Phase 1.0)
```bash
annactl sentinel status        # Show sentinel daemon status
annactl sentinel metrics       # Detailed metrics and event counts
annactl config get             # View current configuration
annactl config set <key> <val> # Update configuration at runtime
```

**Autonomous Daemon:**
Anna runs as a persistent sentinel that continuously monitors and responds to system events:
- Periodic health checks (every 5 minutes)
- Update scans (every hour)
- Security audits (every 24 hours)
- Service failure detection and auto-restart (opt-in)
- Package drift notifications
- Log anomaly monitoring

**Configuration Keys:**
- `autonomous_mode` - Enable autonomous operations (default: false)
- `health_check_interval` - Seconds between checks (default: 300)
- `update_scan_interval` - Seconds between scans (default: 3600)
- `audit_interval` - Seconds between audits (default: 86400)
- `auto_repair_services` - Auto-restart failed services (default: false)
- `auto_update` - Auto-install updates (default: false)
- `auto_update_threshold` - Max packages for auto-update (default: 5)
- `adaptive_scheduling` - Adjust frequencies by stability (default: true)

**Observability:**
- Real-time metrics: uptime, event counts, error rates
- System drift index (0.0-1.0 scale)
- State persistence in `/var/lib/anna/state.json`
- Structured event logging to `/var/log/anna/sentinel.jsonl`

**Output Example:**
```
$ annactl sentinel status
┌─────────────────────────────────────────────────────────
│ SENTINEL STATUS
├─────────────────────────────────────────────────────────
│ Enabled:        ✓ Yes
│ Autonomous:     ✗ Inactive
│ Uptime:         3600 seconds
│ System State:   configured
├─────────────────────────────────────────────────────────
│ HEALTH
│ Status:         Healthy
│ Last Check:     2025-11-11T18:00:00Z
└─────────────────────────────────────────────────────────

$ annactl config set autonomous_mode true
[anna] Configuration updated: autonomous_mode = true
```

**Safety Guarantees:**
- All automated actions require explicit configuration
- Never modifies `/home` or `/data` directories
- Configuration changes logged with timestamps
- Dry-run validation for all mutations

### Repair Actions (Phase 0.7)
```bash
annactl repair                 # Repair all failed probes
annactl repair --dry-run       # Simulate repairs without executing
annactl repair disk-space      # Repair specific probe
```

**Automated Repairs:**
- `disk-space` → Clean systemd journal + pacman cache
- `pacman-db` → Synchronize package databases
- `services-failed` → Restart failed systemd units
- `firmware-microcode` → Install missing CPU microcode

**Output Example:**
```
[anna] repair: probe=disk-space
[anna] probe: disk-space — cleanup_disk_space (OK)
  journalctl --vacuum-size=100M (success); paccache -r -k 2 (success)
  Citation: [archwiki:System_maintenance#Clean_the_filesystem]
All repairs completed successfully
Citation: [archwiki:System_maintenance]
```

**Audit Trail:**
All repair actions logged to `/var/log/anna/audit.jsonl` with timestamps, commands, and results.

### Recovery (Phase 0.6 - Foundation)
```bash
annactl rescue list         # Show available recovery plans
```

**Recovery Plans:**
- `bootloader`: GRUB/systemd-boot repair ([archwiki:GRUB#Installation])
- `initramfs`: Rebuild initramfs images ([archwiki:Mkinitcpio])
- `pacman-db`: Database repair ([archwiki:Pacman/Tips_and_tricks])
- `fstab`: Filesystem table validation ([archwiki:Fstab])
- `systemd`: Unit restoration ([archwiki:Systemd])

---

## Architecture

### System Overview

```
┌─────────────────────────────────────┐
│        annad (Daemon)               │
│                                     │
│  ┌──────────┐    ┌──────────────┐ │
│  │  State   │    │    Health    │ │
│  │ Machine  │    │  Subsystem   │ │
│  └────┬─────┘    └──────┬───────┘ │
│       │                 │          │
│       ▼                 ▼          │
│  ┌─────────────────────────────┐  │
│  │      RPC Server             │  │
│  │    (Unix Socket)            │  │
│  └──────────┬──────────────────┘  │
└─────────────┼─────────────────────┘
              │
              ▼
      /run/anna/anna.sock
       (root:anna 0660)
              │
              ▼
       ┌──────────┐
       │ annactl  │
       └──────────┘
```

### State Machine

Anna detects and adapts to six system states:

1. **iso_live**: Running from Arch ISO
2. **recovery_candidate**: Chroot-ready environment
3. **post_install_minimal**: Fresh Arch install
4. **configured**: Fully configured system
5. **degraded**: System with detected issues
6. **unknown**: Unable to determine state

Commands are only available in states where they're safe to execute.

### File Structure

```
/usr/local/bin/
├── annad                   # Daemon binary
└── annactl                 # CLI client

/var/lib/anna/
├── reports/                # Health and doctor reports (0700)
│   ├── health-*.json       # Health check results (0600)
│   └── doctor-*.json       # Diagnostic reports (0600)
└── alerts/                 # Failed probe alerts (0700)
    └── *.json              # Per-probe alert files (0600)

/var/log/anna/
├── ctl.jsonl               # Command execution log
└── health.jsonl            # Health check history

/run/anna/
└── anna.sock               # IPC socket (root:anna 0660)

/etc/systemd/system/
├── annad.service           # Daemon service unit
└── annad.socket            # Socket activation unit

/usr/local/lib/anna/
├── health/                 # Health probe definitions (YAML)
└── recovery/               # Recovery plan definitions (YAML)
```

---

## Security

### Systemd Hardening

Anna runs with strict systemd sandboxing:

```ini
[Service]
# Security
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=yes
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
RestrictNamespaces=true
RestrictRealtime=true
RestrictSUIDSGID=true
SystemCallArchitectures=native

# File Access
ReadWritePaths=/var/lib/anna /var/log/anna
```

### Permissions

- **Socket**: `root:anna` with mode `0660`
- **System group**: Users must be in `anna` group
- **Reports**: Mode `0600` (root-only read)
- **Directories**: Mode `0700` (root-only access)
- **Logs**: Append-only JSONL format

### Audit Trail

Every command execution is logged with:
- ISO 8601 timestamp
- UUID request ID
- System state at execution time
- Exit code and duration
- Arch Wiki citation
- Success/failure status

Example log entry:
```json
{
  "ts": "2025-11-11T13:00:00Z",
  "req_id": "550e8400-e29b-41d4-a716-446655440000",
  "state": "configured",
  "command": "health",
  "allowed": true,
  "args": [],
  "exit_code": 0,
  "citation": "[archwiki:System_maintenance]",
  "duration_ms": 45,
  "ok": true
}
```

---

## Health Monitoring (Phase 0.5)

### Probes

Each health probe:
- Executes read-only system checks
- Reports status: `ok`, `warn`, or `fail`
- Includes Arch Wiki citation
- Logs execution time
- Creates alerts for failures

### Report Generation

```bash
annactl health
# Output:
# Health summary: ok=5 warn=1 fail=0
# warn: disk-space  [archwiki:System_maintenance#Check_for_errors]
# Details saved: /var/lib/anna/reports/health-2025-11-11T13:00:00Z.json
```

### Doctor Diagnostics

```bash
annactl doctor
# Output:
# Doctor report for state: configured
# Failed probes: none
# Degraded units: 0
# Top journal errors: (see details)
# Citations: [archwiki:System_maintenance] ...
# Report saved: /var/lib/anna/reports/doctor-2025-11-11T13:00:00Z.json
```

---

## Testing

### Integration Tests

```bash
# Run health CLI tests
cargo test --package annad --test health_cli_tests

# Run all tests
cargo test --workspace
```

**Test Coverage:**
- 10 integration tests for health CLI
- Exit code validation (0, 1, 2, 64, 65, 70)
- Report generation and permissions
- JSON schema validation
- Control log verification

### CI Pipeline

GitHub Actions workflow validates:
- Code formatting (`cargo fmt --check`)
- Linting (`cargo clippy`)
- Performance benchmarks (<200ms health command)
- Unauthorized write detection
- JSON schema compliance
- File permissions (0600/0700)

---

## Development

### Building from Source

```bash
git clone https://github.com/YOUR_ORG/anna-assistant.git
cd anna-assistant
git checkout anna-1.0-reset

# Build release binaries
cargo build --release

# Install locally
sudo ./scripts/install.sh --local

# Start daemon
sudo systemctl start annad

# Run commands
annactl status
annactl health
```

### Running Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Health CLI tests specifically
cargo test --package annad --test health_cli_tests

# With output
cargo test -- --nocapture
```

### Project Structure

```
anna-assistant/
├── crates/
│   ├── annad/              # Daemon
│   │   └── src/
│   │       ├── health/     # Health subsystem (Phase 0.5)
│   │       ├── recovery/   # Recovery framework (Phase 0.6)
│   │       ├── state/      # State detection (Phase 0.3)
│   │       └── rpc_server.rs
│   ├── annactl/            # CLI client
│   │   └── src/
│   │       ├── health_commands.rs
│   │       └── main.rs
│   └── anna_common/        # Shared types
├── assets/
│   ├── health/             # Health probe YAML definitions
│   └── recovery/           # Recovery plan YAML definitions
├── scripts/
│   ├── install.sh          # Installation script
│   └── uninstall.sh        # Uninstallation script
├── tests/
│   └── schemas/            # JSON schemas for validation
└── docs/
    └── ANNA-1.0-RESET.md   # Architecture documentation
```

---

## Migration from Beta/RC.11

**⚠️ BREAKING CHANGES - See MIGRATION-1.0.md**

Anna 1.0 removed several features present in earlier versions:

### Removed Features
- Desktop environment bundles (Hyprland, i3, sway)
- Application installation system
- TUI (terminal user interface)
- Recommendation engine
- Pywal integration
- Hardware detection for DEs
- `annactl setup` command
- `annactl apply` command (replaced with recovery plans)
- `annactl advise` command

### What Remains
- ✅ Core daemon (`annad`)
- ✅ CLI client (`annactl`)
- ✅ State detection
- ✅ Health monitoring
- ✅ System diagnostics
- ✅ Recovery framework (foundation)
- ✅ Comprehensive logging
- ✅ Security hardening

### Migration Path
1. Uninstall old version: `sudo ./scripts/uninstall.sh`
2. Remove old configs: `rm -rf ~/.config/anna`
3. Install rc.13: `curl -sSL .../scripts/install.sh | sh`
4. Verify: `annactl health`

---

## Documentation

- **ANNA-1.0-RESET.md**: Architecture and design decisions
- **MIGRATION-1.0.md**: Breaking changes and migration guide
- **SECURITY_AUDIT.md**: Security model and hardening
- **CHANGELOG.md**: Version history
- **docs/IPC_API.md**: RPC protocol documentation

### Man Pages

```bash
man annactl        # CLI usage
man annad          # Daemon configuration
```

---

## Development Status

### ✅ Shipped (rc.13.2)
- **Phase 0.3**: State-aware command dispatch with 6-state detection
- **Phase 0.4**: Systemd hardening and security sandbox
- **Phase 0.5**: Health monitoring (6 probes) + doctor diagnostics
- **Auto-update**: Daemon self-updates every 2 hours from GitHub releases
- **Audit logging**: JSONL logs with UUIDs and Arch Wiki citations

### 🚧 In Progress
- **Phase 0.6**: Recovery framework foundation (types, parser, chroot)
  - Foundation complete, execution pending

### 📋 Planned
- **v1.0 Stable**: Production-ready release
  - Executable recovery plans (`annactl rescue run <plan>`)
  - Rollback script generation (`annactl rollback <plan>`)
  - Manual update trigger (`annactl update`)
- **v1.1**: Enhanced diagnostics
  - User-mode operation (no sudo required)
  - `annactl diag` diagnostic command
  - Dual-socket support (user/system)
- **v2.0**: Optional TUI
  - Terminal user interface returns as optional component
  - Advanced recovery workflows

---

## Contributing

See `CONTRIBUTING.md` for:
- Code style guidelines
- Testing requirements
- Pull request process
- Security disclosure policy

---

## License

[Your License Here]

---

## Support

- **Issues**: https://github.com/YOUR_ORG/anna-assistant/issues
- **Documentation**: https://docs.annaassistant.dev
- **Wiki**: https://wiki.archlinux.org

---

## Credits

Anna Assistant is built on the foundation of the Arch Linux community and adheres strictly to Arch Wiki standards.

**Citations:**
- [archwiki:System_maintenance]
- [archwiki:System_maintenance#Troubleshooting]
- [archwiki:Chroot#Using_arch-chroot]
- [archwiki:GRUB#Installation]
- [archwiki:Mkinitcpio]
- [archwiki:Pacman]
- [archwiki:Systemd]

---

**Anna Assistant v1.0.0-rc.13 - Operational Core**

*Security-hardened • State-aware • Wiki-strict • Production-ready*
