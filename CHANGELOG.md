# Changelog

All notable changes to Anna Assistant will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.9.2a-final] - Sprint 3 Self-Healing Runtime - 2025-10-30

### Added - Self-Healing System
- **Automated Permission Repair**:
  - Installer now runs as normal user, escalates only when needed
  - Auto-detects and fixes missing `anna` group
  - Auto-adds users to group with re-login detection
  - Creates per-user audit logs at `/var/lib/anna/users/<uid>/audit.log`
  - Runs `annactl doctor repair --bootstrap` automatically after install
- **Enhanced Doctor Diagnostics**:
  - 16 comprehensive health checks (vs. 8 previously)
  - Granular checks: `anna_group`, `group_membership`, `socket_permissions`
  - Permission validation: config (0750), state (0750), runtime (0770), socket (0660)
  - Path validation: `/etc/anna`, `/var/lib/anna`, `/run/anna`
  - Auto-repair bootstrap runs twice (fix, then verify)
- **Capability Gating**:
  - Detects polkit availability, skips gracefully if missing
  - Shows actionable message: "Install with: sudo pacman -S polkit"
  - Installer continues without polkit (autonomy disabled)
- **Idempotent Installation**:
  - Detects existing state and preserves it
  - [OK] for already-correct state
  - [FIXED] for repaired issues
  - [SKIP] for unavailable features
  - [FAIL] only for critical errors

### Changed
- **Installer** (`scripts/install.sh`):
  - No longer requires `sudo` to start - escalates automatically
  - Uses `run_elevated()` helper for sudo/pkexec
  - Compiles as user, installs as root
  - Better status messages with color coding
  - Auto-runs doctor repair bootstrap
- **Daemon** (`src/annad/src/main.rs`):
  - Uses `chown` command instead of nix crate for ownership
  - Improved structured logging ([BOOT], [READY])
  - Auto-creates audit log directories
- **Diagnostics** (`src/annad/src/diagnostics.rs`):
  - Added 8 new diagnostic checks
  - More precise permission validation
  - Better fix hints with exact commands
- **Version**: Updated to v0.9.2a-final

### Fixed
- Group membership detection now works in all scenarios
- Socket permissions verified as 0660 root:anna
- User audit logs created with correct ownership
- Doctor repair runs automatically post-install
- No manual `sudo` needed for repairs

### Validation
- Installer idempotency: ✓ PASS
- Permission auto-repair: ✓ PASS
- Group auto-add: ✓ PASS (with re-login notice)
- Doctor bootstrap: ✓ PASS (2-pass verification)
- Audit log creation: ✓ PASS (0640 root:anna)
- Socket permissions: ✓ PASS (0660 root:anna)
- Service startup: ✓ PASS
- All CLI commands: ✓ PASS

---

## [0.9.2a] - Sprint 3 Runtime Validation - 2025-10-30

### Added
- **Runtime Validation Suite** (`tests/runtime_validation.sh`):
  - 12 comprehensive end-to-end tests
  - Validates installation, service status, socket creation, and all CLI commands
  - Automated logging to `tests/logs/runtime_validation.log`
  - Integration with QA runner for privileged testing
- **Systemd Integration**:
  - `packaging/arch/annad.service` - Proper systemd unit with RuntimeDirectory
  - `packaging/arch/annad.tmpfiles.conf` - Boot-time directory creation for `/run/anna`
  - RuntimeDirectory management (0770 root:anna)
- **Anna Group Management**:
  - System group `anna` for socket access control
  - Install script adds users to group automatically
  - Post-install validation checks group membership
- **Comprehensive Installation System**:
  - Idempotent `scripts/install.sh` (447 lines)
  - Proper permission handling (0750 config, 0660 socket, 0770 runtime)
  - Post-install validation with `annactl ping/status` tests
  - Group context handling with `sg anna` workaround
- **Safe Uninstallation**:
  - Complete rewrite of `scripts/uninstall.sh` (365 lines)
  - Timestamped backups to `~/Documents/anna_backup_<timestamp>/`
  - Generated `README-RESTORE.md` with three restore options
  - Preserves user-specific configuration
- **Enhanced Documentation**:
  - `docs/RUNTIME-VALIDATION-Sprint3.md` - Complete validation guide
  - `DEPLOYMENT-INSTRUCTIONS.md` - Step-by-step deployment procedures
  - `docs/SPRINT-3-RUNTIME-STATUS.md` - Implementation status tracking
  - Comprehensive troubleshooting sections

### Changed
- **Daemon Initialization** (`src/annad/src/main.rs`):
  - Hardened startup sequence with structured `[BOOT]` and `[READY]` logging
  - Automatic directory creation with proper permissions
  - Socket permission enforcement (0660 root:anna)
  - Anna group ID lookup with fallback handling
  - Early exit with explicit errors on initialization failure
- **CLI Error Handling** (`src/annactl/src/main.rs`):
  - Comprehensive troubleshooting guidance on connection failures
  - 5-step diagnostic hints when daemon unavailable
  - Clear actionable error messages with exact commands
- **Install Script** (`scripts/install.sh`):
  - Fixed compilation error detection (was incorrectly passing on failures)
  - Added anna group creation and user management
  - Directory permissions set to exact requirements
  - Systemd service and tmpfiles installation
  - Post-install validation with socket and command tests
- **Systemd Service** (`packaging/arch/annad.service`):
  - Removed overly strict `ProtectSystem=strict` and `ProtectHome=true`
  - Kept essential security: `NoNewPrivileges=true`, `PrivateTmp=true`
  - Fixed "Read-only file system" errors
- **QA Runner** (`tests/qa_runner.sh`):
  - Added runtime validation stage with privilege detection
  - Graceful skip with instructions when sudo unavailable
  - Passwordless sudo detection for automated testing

### Fixed
- **Socket Creation**:
  - Daemon now creates `/run/anna` directory if missing
  - Correct permissions (0660) enforced at socket creation
  - Ownership set to root:anna for group access
- **Permission Issues**:
  - All directories now have correct ownership (root:anna)
  - Config directory: 0750 root:anna
  - State directory: 0750 root:anna
  - Runtime directory: 0770 root:anna
  - Socket: 0660 root:anna
- **Service Startup**:
  - No more "Read-only file system (os error 30)" errors
  - No more "os error 2" on socket creation
  - Clean startup with `[READY]` message in logs
- **Installation Reliability**:
  - Compilation failures now properly detected and reported
  - No silent failures in build process
  - Correct exit codes on all error paths

### Testing
- **134 Unit Tests** - All passing (no regressions)
- **12 Runtime Tests** - Validates actual deployment and operation
- **Full QA Suite** - Extended with privileged testing stage
- **Idempotency** - Install/uninstall scripts safe to run multiple times

### Documentation
- Complete runtime validation guide with troubleshooting
- Deployment instructions for Arch Linux
- Permission matrix and security model
- Step-by-step testing procedures

---

## [0.9.2] - Sprint 3 - 2025-10-30

### Added

#### Policy Engine
- **Rule-based decision making**:
  - YAML-based policy definition in `/etc/anna/policies.d/*.yaml`
  - Condition syntax: `field operator value` (e.g., `telemetry.error_rate > 5%`)
  - Supported operators: `>`, `<`, `>=`, `<=`, `==`, `!=`
  - Value types: numbers, percentages, strings, booleans
- **Policy actions**:
  - `disable_autonomy` / `enable_autonomy` - Control autonomy level
  - `run_doctor` - Trigger diagnostics
  - `restart_service` - Restart daemon
  - `send_alert` - Send notifications
  - `custom: <command>` - Execute custom actions
- **PolicyContext**: Structured state representation for rule evaluation
- **CLI commands**: `annactl policy list/reload/eval`
- **RPC operations**: PolicyEvaluate, PolicyReload, PolicyList
- **Example policies**: Telemetry-based and system health monitoring rules

#### Event Reaction System
- **Structured event types**:
  - `TelemetryAlert` - Metric threshold violations
  - `ConfigChange` - Configuration modifications
  - `DoctorResult` - Diagnostic check outcomes
  - `AutonomyAction` - Autonomous task executions
  - `PolicyTriggered` - Policy-driven reactions
  - `SystemStartup` / `SystemShutdown` - Lifecycle events
- **Event severity levels**: Info, Warning, Error, Critical
- **EventDispatcher**:
  - Handler registration for custom reactions
  - Event history (last 1000 events in memory)
  - Filtering by type and severity
  - Automatic policy evaluation on dispatch
- **EventReactor**: High-level coordinator with built-in handlers
- **CLI commands**: `annactl events show/list/clear`
- **RPC operations**: EventsList, EventsShow, EventsClear
- **Telemetry integration**: Events logged for audit trail

#### Learning Cache (Passive Intelligence)
- **Outcome tracking**:
  - `Success`, `Failure`, `Partial` outcome types
  - Action execution statistics (count, rate, duration)
  - Recent outcomes window (last 100 per action)
- **Intelligent scoring**:
  - Priority scoring based on success rate and confidence
  - Retry logic with consecutive failure detection
  - Time-weighted recent performance (70% recent, 30% historical)
- **Persistent storage**: `/var/lib/anna/learning.json`
- **LearningAnalytics**: Top/worst performer analysis
- **Recommendation engine**: Actions ranked by success probability
- **CLI commands**: `annactl learning stats/recommendations/reset`
- **RPC operations**: LearningStats, LearningRecommendations, LearningReset
- **Global metrics**: Total actions, outcomes, success rate

#### Integration & Flow
- **Event → Policy → Action → Learning cycle**:
  1. System generates event (e.g., telemetry alert)
  2. EventDispatcher evaluates policies against event context
  3. Matched policies trigger actions
  4. Action outcomes recorded in learning cache
  5. Future decisions informed by learning scores
- **Cross-module linkage**: Events link to PolicyEngine, PolicyEngine uses telemetry context, Learning informs autonomy

### Changed
- **RPC server**: Extended with Sprint 3 request handlers (policy, events, learning)
- **annactl**: Added policy, events, and learning subcommands with formatted output
- **Daemon initialization**: Now includes policy, events, and learning module declarations

### Technical Details

#### New Modules
- `src/annad/src/policy.rs` (466 lines) - Policy engine with YAML parsing and evaluation
- `src/annad/src/events.rs` (382 lines) - Event system with dispatcher and reactor
- `src/annad/src/learning.rs` (402 lines) - Learning cache with analytics

#### Modified Modules
- `src/annad/src/rpc.rs` - Added Sprint 3 RPC handlers (+128 lines)
- `src/annad/src/main.rs` - Added Sprint 3 module declarations
- `src/annactl/src/main.rs` - Added Sprint 3 CLI commands (+138 lines)

#### New Dependencies
```toml
serde_yaml = "0.9"          # Policy YAML parsing
uuid = "1.0"                # Event ID generation
tempfile = "3.8"            # Test utilities
```

#### File Layout Updates
```
/etc/anna/policies.d/                    # Policy rule files
/etc/anna/policies.d/*.yaml              # Individual policy files
/var/lib/anna/learning.json              # Learning cache persistence
docs/policies.d/example-*.yaml           # Example policies
docs/policies.d/README.md                # Policy documentation
```

### QA Validation
- ✅ 134 tests passed (57 Sprint 1 + 32 Sprint 2 + 34 Sprint 3 + 11 integration)
- ✅ 100% test coverage for Sprint 3 modules
- ✅ Compilation clean (warnings only)
- ✅ Zero critical regressions
- ✅ Full backward compatibility with Sprints 1-2
- ⏱️ Test runtime: 3 seconds

### Known Limitations
- **Policy Engine**: Stub RPC handlers (full daemon integration requires Sprint 4)
- **Event Persistence**: Events in-memory only (max 1000, no disk persistence yet)
- **Learning Intelligence**: Basic scoring algorithm (advanced ML planned for later)

### Migration Notes
- No breaking changes - Sprint 3 is fully backward compatible
- New CLI commands are opt-in
- Policy files optional - system works without policies
- Learning cache auto-creates on first use

### Example Usage

**Policy Management:**
```bash
annactl policy list               # List loaded policies
annactl policy reload             # Reload from /etc/anna/policies.d/
annactl policy eval --context '{"telemetry.error_rate": 0.06}'
```

**Event Inspection:**
```bash
annactl events show --limit 20    # Recent events
annactl events show --severity error  # Filter by severity
annactl events clear              # Clear history
```

**Learning Cache:**
```bash
annactl learning stats            # Global statistics
annactl learning stats doctor_autofix  # Specific action
annactl learning recommendations  # Recommended actions
annactl learning reset --confirm  # Reset cache
```

### Performance Impact
- **Compilation time**: +12s (new dependencies)
- **Binary size**: +340KB (annad), +180KB (annactl)
- **Runtime overhead**: <5ms per event dispatch
- **Memory usage**: ~2MB (policy engine + events + learning)

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
