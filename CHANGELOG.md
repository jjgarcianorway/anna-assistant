# Changelog

All notable changes to Anna Assistant will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.9.1] - Sprint 2 - 2025-10-30

### Added

#### Autonomy Framework
- **Three-level autonomy system**:
  - `Off` - No automatic actions, all operations require explicit user commands
  - `Low` - Safe self-maintenance tasks only (diagnostics, cleanup)
  - `Safe` - Interactive recommendations that require user approval
- **Autonomous task types**:
  - `Doctor` - Run health checks automatically
  - `TelemetryCleanup` - Rotate old telemetry files
  - `ConfigSync` - Synchronize user and system configurations
- **Permission-based execution**: Tasks only run if allowed by current autonomy level
- **CLI commands**: `annactl autonomy status` and `annactl autonomy run <task>`
- **RPC operations**: AutonomyStatus, AutonomyRun
- **Telemetry logging**: All autonomous actions logged with task name and outcome

#### Persistence Layer
- **State management API**:
  - `save_state(component, data)` - Save component state as JSON
  - `load_state(component)` - Load component state with metadata
  - `list_states()` - List all saved states
- **Storage location**: `/var/lib/anna/state/` with weekly rotation
- **Metadata tracking**: Timestamp, version, component name for each state
- **Automatic cleanup**: States older than 7 days are rotated out
- **CLI commands**: `annactl state save/load/list`
- **RPC operations**: StateSave, StateLoad, StateList
- **Initialization**: Persistence layer initialized on daemon startup

#### Auto-Fix Mechanism
- **Enhanced doctor command**: `annactl doctor --autofix` flag for automatic repairs
- **Safe fix implementations**:
  - `autofix_socket_directory` - Create `/run/anna` if missing
  - `autofix_paths` - Create required directories (`/etc/anna`, `/var/lib/anna`, etc.)
  - `autofix_config_directory` - Create `/etc/anna` if missing
  - `autofix_polkit_notice` - Provide manual instructions for polkit policy installation
- **AutoFixResult tracking**: Each fix reports attempted/success/message
- **Telemetry integration**: All fix attempts logged as `autofix.<check_name>` events
- **RPC operation**: DoctorAutoFix
- **Deterministic behavior**: Only safe, idempotent operations are automated

#### Enhanced Telemetry Dashboard
- **New CLI commands**:
  - `annactl telemetry list` - Show recent telemetry events (with `--limit` or `-l` flag)
  - `annactl telemetry stats` - Display event type statistics
- **Multi-path reading**: Reads from both system (`/var/lib/anna/events`) and user (`~/.local/share/anna/events`) paths
- **JSON parsing**: Deserializes and pretty-prints event data
- **Event counting**: Aggregates events by type for statistics view
- **Default limit**: Shows 20 most recent events (configurable via --limit)

### Changed
- **Doctor diagnostics**: Extended with auto-fix capability via `run_autofix()` function
- **Telemetry module**: Added public `rotate_old_files_now()` function for manual rotation trigger
- **annactl**: Complete rewrite with all Sprint 2 commands and formatted output
- **RPC server**: Extended with Sprint 2 request handlers (autonomy, state, autofix)
- **Daemon initialization**: Now initializes persistence layer on startup

### Technical Details

#### New Modules
- `src/annad/src/autonomy.rs` (148 lines) - Autonomy framework implementation
- `src/annad/src/persistence.rs` (205 lines) - State management and storage

#### Modified Modules
- `src/annad/src/diagnostics.rs` - Added auto-fix functions and telemetry logging
- `src/annad/src/rpc.rs` - Added Sprint 2 request handlers
- `src/annad/src/main.rs` - Added module declarations and persistence initialization
- `src/annad/src/telemetry.rs` - Exposed rotation function for autonomy module
- `src/annactl/src/main.rs` (588 lines) - Complete rewrite with all Sprint 2 commands

#### New Dependencies
- `dirs = "5.0"` in annactl for home directory resolution

#### File Layout Updates
```
/var/lib/anna/state/              # Persistent state storage
/var/lib/anna/state/<component>.json  # Component state files
```

### QA Validation
- ✅ 89 tests passed (57 Sprint 1 + 32 Sprint 2)
- ✅ Compilation clean (14 harmless warnings)
- ✅ Contract compliance verified
- ✅ Full backward compatibility with Sprint 1
- ⏱️ Test runtime: 1 second

### Known Limitations
- Autonomy tasks are implemented but not scheduled (no timer triggers yet)
- State persistence is manual (no automatic snapshots)
- Auto-fix covers common issues but not all diagnostic failures
- Telemetry commands read from files directly (no daemon-side aggregation)

### Migration Notes
- No breaking changes - Sprint 2 is fully backward compatible with Sprint 1
- New CLI commands are opt-in - existing workflows unchanged
- Default autonomy level remains "off" - requires explicit configuration to enable
- Persistence layer creates directories automatically on first use

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
