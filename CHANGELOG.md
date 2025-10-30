# Changelog

All notable changes to Anna Assistant will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.9.0] - Sprint 1 - 2025-10-30

### Added

#### Privilege Model & Security
- **Privilege separation architecture**: annad runs as root daemon, annactl runs unprivileged
- **Polkit integration**: System-wide operations authorized via polkit
- **Policy file**: `com.anna.policy` with actions for config.write and maintenance.execute
- **Design document**: `DESIGN-NOTE-privilege-model.md` explaining architecture and future options

#### Configuration Service
- **Multi-scope configuration**: System config (`/etc/anna/config.toml`) and user config (`~/.config/anna/config.toml`)
- **Merge strategy**: User settings override system settings
- **Configuration keys**:
  - `autonomy.level` - Automation level (off/low/safe)
  - `telemetry.local_store` - Local telemetry storage (on/off)
  - `shell.integrations.autocomplete` - Bash completion (on/off)
- **RPC operations**: Config.Get, Config.Set, Config.List
- **CLI commands**: `annactl config get/set/list`

#### Doctor Diagnostics
- **Comprehensive health checks**:
  - `daemon_active` - Systemd service status
  - `socket_ready` - Unix socket availability
  - `polkit_policies_present` - Policy installation
  - `paths_writable` - Required directories accessible
  - `autocomplete_installed` - Bash completion present
- **Fix hints**: Each failing check includes remediation command
- **Non-zero exit**: `annactl doctor` exits 1 on any failure

#### Telemetry
- **Local-only event logging**: No network, no PII
- **Event types**:
  - `daemon_started` - Daemon initialization
  - `rpc_call` - RPC invocations with status
  - `config_changed` - Configuration modifications
- **Storage locations**:
  - System: `/var/lib/anna/events/`
  - User: `~/.local/share/anna/events/`
- **Rotation**: Daily log files, maximum 5 kept

#### Installation & Deployment
- **Idempotent installer** (`scripts/install.sh`):
  - Compiles release binaries
  - Installs to `/usr/local/bin`
  - Installs systemd service
  - Installs polkit policy
  - Installs bash completion
  - Creates required paths
  - Enables and starts daemon
  - Runs diagnostics
- **Safe uninstaller** (`scripts/uninstall.sh`):
  - Stops and disables service
  - Creates timestamped backup in `~/Documents/anna_backup_<timestamp>/`
  - Includes `README-RESTORE.md` with restore instructions
  - Removes all binaries and configuration

#### Developer Experience
- **Bash completion**: Full command and argument completion for annactl
- **QA test harness**: Comprehensive automated testing suite (57 tests)
- **Documentation**:
  - 60-second quickstart in README
  - Architecture diagrams
  - Design notes
  - QA results report

### Changed
- **Socket path**: Updated from `/run/anna.sock` to `/run/anna/annad.sock`
- **Config structure**: Expanded with Sprint 1 keys and sections
- **Systemd service**: Enhanced with security hardening options

### Technical Details

#### Architecture
- **Language**: Rust (stable)
- **IPC**: Unix socket at `/run/anna/annad.sock` with 0666 permissions
- **Daemon**: tokio-based async RPC server
- **CLI**: clap-based command parser with subcommands

#### Dependencies
- Core: tokio, serde, anyhow, clap
- System: nix (Unix APIs), which (executable lookup)
- Time: chrono (telemetry timestamps)
- Config: toml (configuration parsing)

#### File Layout
```
/etc/anna/                           # System configuration
/etc/anna/config.toml               # System config file
/run/anna/annad.sock                # Daemon socket
/var/lib/anna/events/               # System telemetry
~/.config/anna/config.toml          # User config (optional)
~/.local/share/anna/events/         # User telemetry
/usr/local/bin/annad                # Daemon binary
/usr/local/bin/annactl              # CLI binary
/usr/share/polkit-1/actions/com.anna.policy  # Polkit policy
/usr/share/bash-completion/completions/annactl  # Bash completion
```

### QA Validation
- ✅ 57 tests passed
- ✅ Compilation clean (warnings only)
- ✅ Contract compliance verified
- ✅ All deliverables present
- ⏱️ Test runtime: 1 second

### Known Limitations
- Polkit integration is preparatory; actual D-Bus integration deferred to future sprint
- Doctor checks validate installation but don't auto-repair
- Telemetry events are logged but not yet used for insights
- Autonomy levels defined but not yet enforced

### Migration Notes
No migration needed - this is the initial Sprint 1 release.

---

## [0.9.0-genesis] - 2025-10-30

### Added
- Initial project scaffolding
- Basic daemon/client architecture
- Simple RPC over Unix socket
- Minimal diagnostics
- Installation scripts

---

**Legend:**
- Added: New features
- Changed: Changes to existing functionality
- Deprecated: Soon-to-be removed features
- Removed: Now removed features
- Fixed: Bug fixes
- Security: Vulnerability fixes
