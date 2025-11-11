# Anna Assistant

**Autonomous Arch Linux System Administrator**

Anna is a production-ready, security-hardened system administration daemon for Arch Linux. She provides state-aware command dispatch, comprehensive health monitoring, and Arch Wiki-cited operations.

**Current Version:** 1.0.0-rc.13 (Release Candidate - November 2025)

**Branch:** `anna-1.0-reset` → `main`

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

Anna graduated from prototype to operational core in v1.0.0-rc.13.

---

## Quick Start

### Installation

```bash
# Install from release
curl -sSL https://raw.githubusercontent.com/YOUR_ORG/anna-assistant/main/scripts/install.sh | sh

# Or clone and build
git clone https://github.com/YOUR_ORG/anna-assistant.git
cd anna-assistant
cargo build --release
sudo ./scripts/install.sh --local
```

### Basic Usage

```bash
# Check system status
annactl status

# Run health checks
annactl health

# Get diagnostic report
annactl doctor

# Show available commands
annactl help

# List recovery plans
annactl rescue list
```

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

## Roadmap

### Phase 0.3 ✅ Complete
- State-aware command dispatch
- No-op handlers with logging

### Phase 0.4 ✅ Complete
- Systemd hardening
- Security audit and permissions

### Phase 0.5 ✅ Complete
- Health subsystem with 6 probes
- Doctor diagnostics
- Recovery plan scaffolds
- Integration tests and CI

### Phase 0.6 🚧 In Progress
- Executable recovery plans
- Rollback script generation
- Interactive rescue mode
- `annactl rescue run <plan>`
- `annactl rollback <plan>`

### Future Phases
- Phase 0.7: State-aware update system
- Phase 0.8: Backup automation
- Phase 0.9: Installation wizard
- Phase 1.0: Stable release

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
