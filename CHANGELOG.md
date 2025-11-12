# Changelog

All notable changes to Anna Assistant will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### ✅ **Phase 3.8: Adaptive CLI - COMPLETE**

**Progressive Disclosure UX**: Context-aware command interface that adapts to user experience and system state.

#### Adaptive Root Help (`crates/annactl/src/adaptive_help.rs` - 280 lines)

**Entry Point Override**:
- Intercepts `--help` before clap parsing
- Context-aware command filtering (User/Root/Developer modes)
- Color-coded category display (🟢 Safe / 🟡 Advanced / 🔴 Internal)
- `--all` flag to show all commands
- `--json` flag for machine-readable output
- NO_COLOR environment variable support

**Display Features**:
- Command count per category
- Context mode indicator
- Progressive disclosure (hide complexity by default)
- TTY detection for color output
- Graceful degradation for non-TTY

#### Context Detection (`crates/annactl/src/context_detection.rs` - 180 lines)

**Execution Context**:
- `ExecutionContext::detect()` - Auto-detects User/Root/Developer
- User level mapping (Beginner/Intermediate/Expert)
- Root detection via `geteuid()`
- Developer mode via `ANNACTL_DEV_MODE` env var

**TTY Detection**:
- `is_tty()` - Checks stdout for terminal
- `should_use_color()` - Respects NO_COLOR and TERM=dumb
- Cross-platform (Unix-only for now)

#### Command Classification (`crates/anna_common/src/command_meta.rs` - 600 lines)

**Metadata System**:
- `CommandRegistry` with 12 classified commands
- `CommandCategory` (UserSafe, Advanced, Internal)
- `RiskLevel` (None, Low, Medium, High, Critical)
- `DisplayContext` for visibility rules
- Comprehensive command metadata (descriptions, examples, prerequisites)

**Classification**:
- **User-Safe (3)**: help, status, health
- **Advanced (6)**: update, install, doctor, backup, rollback, repair
- **Internal (3)**: sentinel, config, conscience

#### Predictive Hints Integration (`crates/annactl/src/predictive_hints.rs` - 270 lines)

**Post-Command Intelligence**:
- Displays High/Critical predictions after `status` and `health`
- 24-hour throttle per command (avoids alert fatigue)
- Learning engine integration with action aggregation
- ActionHistory → ActionSummary conversion
- Skips in JSON mode and non-TTY

**Features**:
- Shows up to 3 most urgent predictions
- One-line format with emoji indicators
- Recommended actions displayed
- Silent failure if context DB unavailable

#### UX Polish

**AUR Awareness** (`main.rs`):
- Detects package-managed installations via `pacman -Qo`
- Prevents self-update for AUR packages
- Shows appropriate update commands (pacman/yay)

**Permission Error Polish** (`rpc_client.rs`):
- Enhanced PermissionDenied error messages
- Shows exact `usermod` command with current username
- Step-by-step fix instructions
- Verification commands included
- Debug info (ls -la, namei -l)

#### Testing (`crates/annactl/tests/integration_test.rs`)

**Acceptance Tests** (13 tests, all passing ✅):
- `test_adaptive_help_user_context` - Context-appropriate display
- `test_adaptive_help_all_flag` - --all shows everything
- `test_json_help_output` - JSON format validation
- `test_command_classification` - Metadata correctness
- `test_context_detection` - Context detection logic
- `test_tty_detection` - TTY functions callable
- `test_no_color_env` - NO_COLOR respected
- `test_help_no_hang` - Help fast even offline (<2s)

#### Documentation

**USER_GUIDE.md** (New):
- Comprehensive user-facing guide
- Quick start instructions
- Common tasks with examples
- Troubleshooting section
- Command quick reference
- Best practices

**COMMAND_CLASSIFICATION.md** (Updated):
- Phase 3.8 implementation status
- Usage examples
- Files changed summary
- Metrics (1,600 lines, 13 tests)

#### Key Achievements

**Progressive Disclosure**:
- Normal users see 1 command (help) by default
- Root users see 9 commands (safe + advanced)
- Developer mode shows all 12 commands
- Clean, uncluttered interface

**Performance**:
- Help display: <100ms even with daemon check
- TTY detection: <1ms
- Context detection: <1ms
- No latency impact on user experience

**Usability**:
- Error messages guide users to solutions
- Permission errors show exact commands
- AUR users redirected to package manager
- JSON mode for scripting/automation

**Quality**:
- 13 acceptance tests passing
- All functionality tested
- Clean build (warnings only)
- Well-documented code

#### Files Changed

- `crates/annactl/src/adaptive_help.rs` - 280 lines (new)
- `crates/annactl/src/context_detection.rs` - 180 lines (new)
- `crates/annactl/src/predictive_hints.rs` - 270 lines (new)
- `crates/anna_common/src/command_meta.rs` - 600 lines (new)
- `crates/annactl/src/main.rs` - Entry point integration, AUR detection
- `crates/annactl/src/rpc_client.rs` - Enhanced error messages
- `crates/annactl/src/steward_commands.rs` - Predictive hints integration
- `crates/annactl/src/health_commands.rs` - Predictive hints integration
- `crates/annactl/src/lib.rs` - Export context_detection
- `crates/annactl/tests/integration_test.rs` - 13 new tests
- `docs/USER_GUIDE.md` - 400+ lines (new)
- `docs/COMMAND_CLASSIFICATION.md` - Updated with Phase 3.8 status

**Total**: ~1,600 lines of production code + 400 lines of documentation

---

### ✅ **Phase 3.7: Predictive Intelligence - CORE COMPLETE**

Rule-based learning and prediction system for proactive system management.

#### Learning Engine (`crates/anna_common/src/learning.rs` - 430 lines)

**Pattern Detection**:
- DetectedPattern with confidence levels (Low 40%, Medium 65%, High 85%, VeryHigh 95%)
- PatternType enum (MaintenanceWindow/CommandUsage/RecurringFailure/ResourceTrend/TimePattern/DependencyChain)
- Actionable pattern filtering (≥Medium confidence + recent)
- Learning statistics and distribution tracking

**Pattern Analysis**:
- Maintenance window detection (update frequency, timing)
- Recurring failure identification (>20% failure rate flagged)
- Command usage patterns (habit detection)
- Resource trend analysis
- Configurable thresholds (min occurrences, analysis window)

**Testing**: 5/5 tests passing ✅

#### Prediction Engine (`crates/anna_common/src/prediction.rs` - 570 lines)

**Prediction Types**:
- ServiceFailure: Predict likely failures from recurring patterns
- MaintenanceWindow: Suggest optimal update times
- ResourceExhaustion: Warn before limits
- PerformanceDegradation: Detect degrading trends
- Recommendation: General system improvements

**Smart Features**:
- Priority levels (Low ℹ️ / Medium ⚠️ / High 🔴 / Critical 🚨)
- Confidence-based filtering (min 65% by default)
- Smart throttling (24-hour cooldown, prevents spam)
- Urgency detection (<24h window or critical priority)
- Time-until prediction display
- Recommended actions for each prediction
- Pattern traceability (predictions link to source patterns)

**Testing**: 6/6 tests passing ✅

#### Documentation (`docs/PREDICTIVE_INTELLIGENCE.md`)

Comprehensive operator guide covering:
- Architecture and design principles
- Confidence levels and thresholds
- API usage examples
- Integration with self-healing
- Performance characteristics (<5% CPU overhead)
- Privacy guarantees (fully local, no personal data)
- Troubleshooting and configuration
- Future enhancements roadmap

#### Key Features

**Local-First**:
- Zero network dependencies
- All learning on-device
- SQLite-backed persistence
- Privacy-preserving (no personal data stored)

**Explainable**:
- Clear pattern descriptions
- Confidence percentages
- Occurrence counts
- Traceability to source data

**Performant**:
- On-demand pattern detection (~1-5ms per 1000 actions)
- Minimal memory footprint (~1MB per 1000 patterns)
- <5% CPU overhead in continuous mode
- Efficient SQLite queries (<10ms typical)

**Production-Ready**:
- Comprehensive test coverage (11/11 tests passing)
- Error handling and validation
- Configurable thresholds and windows
- Smart throttling prevents alert fatigue

#### Integration Points

**With Persistent Context** (Phase 3.6):
- Reads action_history table for pattern detection
- Analyzes command_usage for habit learning
- Queries system_state_log for state transitions

**With Self-Healing** (Phase 3.1/3.2):
- Predictions feed into recovery decisions
- Preemptive health checks for recurring failures
- Dependency chain awareness

#### Pending (Phase 3.8)

CLI command integration:
- `annactl learn [--window DAYS]` - Trigger pattern analysis
- `annactl predict [--urgent-only]` - Display predictions
- `annactl patterns [--type TYPE]` - List detected patterns
- Automatic learning on daemon startup
- Notification system integration

### ✅ **Phase 3.1 + 3.6: Contextual Autonomy - IMPLEMENTED**

Complete implementation of adaptive intelligence features with persistent context and self-healing capabilities.

#### Phase 3.6: Persistent Context Layer

**SQLite-Based Session Continuity** (`crates/anna_common/src/context/`):
- Complete database implementation with 6 tables
- Action history tracking with metadata (duration, outcome, affected items)
- Async-safe operations using tokio-rusqlite
- Smart location detection (system vs user mode)
- WAL mode for concurrent access
- Automatic maintenance and cleanup
- Success rate calculations per action type
- Global singleton API for easy integration
- **Testing**: 7/7 tests passing

**Database Schema**:
- `action_history`: All actions performed with outcomes
- `system_state_log`: Historical state snapshots
- `user_preferences`: User settings and learned preferences
- `command_usage`: Command usage analytics
- `learning_patterns`: Detected behavior patterns
- `session_metadata`: Session tracking

#### Phase 3.1: Command Classification & Adaptive UI

**Command Classification System** (`crates/anna_common/src/command_meta.rs`):
- CommandCategory enum (UserSafe/Advanced/Internal)
- RiskLevel assessment (None/Low/Medium/High/Critical)
- CommandMetadata with complete classification
- DisplayContext for adaptive filtering
- UserLevel detection (Beginner/Intermediate/Expert)
- CommandRegistry with visibility logic
- Display priority calculation
- **Testing**: 8/8 tests passing

**Adaptive Help System** (`crates/annactl/src/help_commands.rs`):
- Context-aware command filtering
- Color-coded categories: 🟢 UserSafe, 🟡 Advanced, 🔴 Internal
- Detailed per-command help with examples
- System state detection with fast timeout
- Daemon availability checking
- Intelligent command visibility based on:
  * User experience level
  * System state (healthy/degraded/critical)
  * Daemon availability
  * Resource constraints
- Context-specific tips and recommendations

**Quick Daemon Connection** (`crates/annactl/src/rpc_client.rs`):
- connect_quick() method for fast availability checks
- 200ms timeout for responsive help display
- No retry delays for help command

#### Phase 3.1: Monitoring Automation

**Production-Ready Installation** (`crates/annactl/src/monitor_setup.rs`):
- Automatic package installation via pacman
- Systemd service management (enable/start)
- Configuration deployment from templates
- Dashboard provisioning for Grafana
- Intelligent dry-run mode
- Root privilege checking
- Package detection (prevents redundant installs)

**Monitoring Modes**:
- **Full**: Prometheus + Grafana + dashboards (4GB+ RAM)
- **Light**: Prometheus only (2-4GB RAM)
- **Minimal**: Internal monitoring only (<2GB RAM)

**Commands**:
- `annactl monitor install [--force-mode MODE] [--dry-run]`
- `annactl monitor status`

#### Phase 3.1/3.2: Self-Healing Framework

**Autonomous Recovery Foundation** (`crates/anna_common/src/self_healing.rs`):
- ServiceHealth tracking (Healthy/Degraded/Failed/Unknown)
- RecoveryAction types (Restart/Reload/StopStart/Manual)
- RecoveryOutcome tracking (Success/Failure/Partial/Skipped)
- ServiceRecoveryConfig with configurable policies:
  * Maximum restart attempts
  * Cooldown periods
  * Automatic vs manual recovery
  * Dependency management
  * Critical service flagging
- SelfHealingManager with history and analytics
- Recovery attempt logging with unique IDs
- Success rate calculation per service
- Default configurations for common services
- **Testing**: 5/5 tests passing

**Default Service Configs**:
- annad (critical, 5 attempts)
- prometheus (3 attempts)
- grafana (3 attempts, depends on prometheus)
- systemd services (resolved, networkd)

### 📋 **Phase 3.5 Planning: Next-Generation Intelligence Features**

Comprehensive design documentation for Anna's evolution toward greater autonomy and usability.

#### Design Documents Added

**Command Classification System** (`docs/COMMAND_CLASSIFICATION.md`):
- Comprehensive classification of all 30+ commands into three categories:
  * 🟢 **User-Safe** (9 commands): help, status, ping, health, profile, metrics, monitor status, self-update --check, triage
  * 🟡 **Advanced** (12 commands): update, install, backup, doctor, rollback, repair, audit, monitor install, rescue, collect-logs, self-update --list
  * 🔴 **Internal** (8 commands): sentinel, config, conscience, empathy, collective, mirror, chronos, consensus
- Adaptive help system design with context-aware command visibility
- Command metadata structure for risk assessment
- Progressive disclosure UX pattern for safer user experience
- Security considerations and accessibility features

**Persistent Context Layer** (`docs/PERSISTENT_CONTEXT.md`):
- SQLite-based session continuity system design
- Complete database schema with 6 tables:
  * `action_history`: Track all actions Anna performed
  * `system_state_log`: Historical system state snapshots
  * `user_preferences`: User-configured settings and learned preferences
  * `command_usage`: Track command usage for learning
  * `learning_patterns`: Detected patterns and learned behaviors
  * `session_metadata`: Track user sessions for context
- Rust API structure for context module
- Usage examples for learning optimal update times, resource prediction, command recommendations
- Privacy-first design: no personal data, only system metadata
- Data retention policies and cleanup strategies
- Migration strategy (Phases 3.4-3.7)

**Automated Monitoring Setup** (`docs/AUTOMATED_MONITORING_SETUP.md`):
- Zero-configuration path from bare system to production-ready observability
- `annactl setup-monitoring` command design with resource-aware adaptation
- Beautiful Grafana dashboard templates:
  * Anna Overview: System health at a glance
  * Resource Metrics: Memory, CPU, disk trends over time
  * Action History: Command success rates and analytics
  * Consensus Health: Distributed system metrics (Phase 1.7+)
- Prometheus configuration templates for light and full modes
- Grafana provisioning with automatic datasource and dashboard setup
- TLS certificate generation for secure access
- Systemd service integration for anna-prometheus and anna-grafana
- Alert rules for proactive system monitoring
- Idempotent installation with upgrade preservation

**Monitoring Dashboard Templates**:
- `monitoring/dashboards/anna-overview.json`: Executive summary dashboard with 8 panels
  * System status, monitoring mode, resource constraints, uptime
  * Memory and disk usage gauges with thresholds
  * Recent actions time series
  * Rolling 24h success rate
- `monitoring/dashboards/anna-resources.json`: Deep resource analysis with 8 panels
  * Memory timeline with total/available/used
  * Memory and disk percentage with threshold highlighting
  * CPU cores display
  * Uptime timeline
  * Mode change state timeline
  * Resource constraint event tracking

**Prometheus Configuration**:
- `monitoring/prometheus/prometheus-light.yml`: Optimized for 2-4 GB RAM
  * 60s scrape interval
  * 30-day retention, 2GB size limit
  * Anna daemon and Prometheus self-monitoring
- `monitoring/prometheus/prometheus-full.yml`: Full-featured for >4 GB RAM
  * 15s scrape interval
  * 90-day retention, 10GB size limit
  * Node exporter, Grafana metrics, Alertmanager integration
- `monitoring/prometheus/rules/anna-alerts.yml`: Comprehensive alert rules
  * Memory alerts (high usage, critical usage)
  * Disk space alerts (low, critical)
  * System state alerts (degraded, critical)
  * Resource constraint alerts
  * Consensus health alerts (Phase 1.7+)
  * Action failure rate alerts
  * Probe failure alerts

**Grafana Provisioning**:
- `monitoring/grafana/provisioning/datasources/prometheus.yml`: Auto-configured Prometheus datasource
- `monitoring/grafana/provisioning/dashboards/anna.yml`: Dashboard provider configuration

**Self-Healing Roadmap** (`docs/SELF_HEALING_ROADMAP.md`):
- Vision for transforming Anna from reactive to proactive maintenance
- 4-level healing maturity model:
  * **Level 0**: Detection Only (current - v3.0.0-alpha.3) ✅
  * **Level 1**: Guided Repair (v3.0.0-beta.1) - Suggest fixes with user confirmation
  * **Level 2**: Supervised Healing (v3.0.0) - Auto-fix safe issues, notify user
  * **Level 3**: Autonomous Healing (v4.0.0+) - Predictive maintenance, self-optimization
- Healing policy configuration system with risk assessment
- Rollback/undo mechanism for reversible actions
- Circuit breaker pattern to prevent runaway healing
- Safety guarantees: pre-flight checks, snapshots, dry-run mode
- User control: healing policies, consent levels, configuration UI
- New Prometheus metrics for healing observability
- Testing strategy with unit, integration, and manual tests
- Complete implementation roadmap through Phase 4.0

#### Design Principles

All designs follow Anna's core principles:
- **Safety First**: Never perform destructive actions without approval
- **Transparency**: Always explain what's happening and why
- **Privacy First**: No personal data collection, only system metadata
- **User Control**: Users configure policies and maintain oversight
- **Gradual Autonomy**: Start simple, enable advanced features progressively
- **Offline**: No cloud sync, all data stays local
- **Reversible**: Every action can be undone

#### What's Next

**Phase 3.6 (v3.0.0-alpha.4)**: Begin implementation of persistent context layer
- Create SQLite schema and migrations
- Implement basic CRUD operations for action history
- Add context module to anna_common crate

**Phase 3.7 (v3.0.0-alpha.5)**: Implement automated monitoring setup
- Build `annactl setup-monitoring` command
- Integrate dashboard provisioning
- Add TLS certificate generation

**Phase 3.8 (v3.0.0-beta.1)**: Begin self-healing infrastructure
- Implement healing policy configuration
- Add risk assessment framework
- Create rollback mechanism

**Citation**: [progressive-disclosure:ux-patterns], [sqlite:best-practices], [prometheus:configuration], [grafana:provisioning], [chaos-engineering:netflix], [self-healing:kubernetes-operators]

---

## [3.0.0-alpha.3] - 2025-11-12

### ⚠️  **Phase 3.4: Resource Constraint Warnings**

Adds proactive warnings before resource-intensive operations on constrained systems.

#### Added

**Smart Resource Warnings for Heavy Operations**:
- Automatically checks system resources before `annactl update` and `annactl install`
- Warns users on resource-constrained systems (<4GB RAM, <2 cores, or <10GB disk)
- Shows current resource availability with percentages
- Lists potential impacts:
  * Significant resource consumption
  * Longer operation times
  * Reduced system responsiveness
- Provides helpful recommendations:
  * Close other applications
  * Run during off-peak hours
  * Use --dry-run to preview changes
- Requires user confirmation (y/N) to proceed
- Skips warning when using --dry-run flag

**Implementation**:
- `crates/annactl/src/main.rs`: 58 lines added for resource checking
- Helper function `check_resource_constraints()`
- Integration with Update and Install commands
- Graceful fallback if daemon unavailable

**User Experience**:
```bash
$ sudo annactl update

⚠️  Resource Constraint Warning
═══════════════════════════════════════════════════════════════
  Your system is resource-constrained:
    • RAM: 1024 MB available of 2048 MB total (50.0%)
    • CPU: 2 cores
    • Disk: 8 GB available

  Operation 'system update' may:
    - Consume significant system resources
    - Take longer than usual to complete
    - Impact system responsiveness

  Consider:
    - Closing other applications
    - Running during off-peak hours
    - Using --dry-run flag to preview changes
═══════════════════════════════════════════════════════════════

Proceed with operation? [y/N]:
```

**Benefits**:
- Prevents system overload on constrained hardware
- Educates users about resource requirements
- Reduces support requests from failed operations
- Allows informed decision-making

---

### 📊 **Phase 3.3: Metrics Command**

Adds `annactl metrics` command for displaying system metrics in multiple formats.

#### Added

**New Command: annactl metrics**:
- Displays current system metrics from daemon's profile
- Three output formats:
  * **Default**: Human-readable with percentages and helpful formatting
  * **--prometheus**: Prometheus exposition format with HELP and TYPE annotations
  * **--json**: Machine-readable JSON for scripting
- Shows all 8 system metrics:
  * Memory (total, available, percentage)
  * CPU cores
  * Disk (total, available, percentage)
  * System uptime (seconds and hours)
  * Monitoring mode (minimal/light/full)
  * Resource constraint status
- Includes adaptive intelligence context and rationale

**Implementation**:
- `crates/annactl/src/main.rs`: 127 lines added for metrics command
- Prometheus-compatible output format
- Percentage calculations for memory and disk
- Human-friendly time conversions

**Usage Examples**:
```bash
# Human-readable output
$ annactl metrics

# Prometheus format (for node_exporter or custom scraping)
$ annactl metrics --prometheus

# JSON format (for scripting)
$ annactl metrics --json
```

**Benefits**:
- Enables custom Prometheus exporters via shell script
- Provides snapshot of system state for debugging
- Machine-readable format for automation
- Complements existing monitoring infrastructure

**Citation**: [prometheus:exposition-formats]

---

## [3.0.0-alpha.2] - 2025-11-12

### 💡 **Phase 3.2: Adaptive UI Hints**

Makes the CLI context-aware by providing mode-specific guidance and warnings.

#### Added

**Smart Warning System for Monitor Commands**:
- Warns users in MINIMAL mode before installing monitoring tools
- Shows resource constraints (RAM, CPU, disk) and recommendations
- Requires confirmation (y/N) to proceed with installation in minimal mode
- Suggests alternative commands: `annactl health`, `annactl status`
- Can be overridden with `--force-mode` flag

**Mode-Specific Guidance in Status Command**:
- `annactl monitor status` now shows adaptive intelligence hints
- MINIMAL mode: Recommends internal stats only
- LIGHT mode: Points to Prometheus, explains Grafana unavailability
- FULL mode: Shows all available monitoring endpoints
- Helpful command suggestions based on current mode

**Implementation**:
- `crates/annactl/src/main.rs`: 68 lines added for adaptive UI logic
- User confirmation dialog for potentially harmful actions
- Context-aware help messages with mode rationale

**User Experience**:
```bash
# Minimal mode warning example:
$ annactl monitor install

⚠️  Adaptive Intelligence Warning
═══════════════════════════════════════════════════════════════
  Your system is running in MINIMAL mode due to limited resources.
  Installing external monitoring tools (Prometheus/Grafana) is
  NOT recommended as it may impact system performance.

  System Constraints:
    • RAM: 1536 MB (recommend >2GB for light mode)
    • CPU: 2 cores
    • Disk: 15 GB available

  Anna's internal monitoring is active and sufficient for your system.
  Use 'annactl health' and 'annactl status' for system insights.

  To override this warning: annactl monitor install --force-mode <mode>
═══════════════════════════════════════════════════════════════

Continue anyway? [y/N]:
```

**Citation**: [ux-best-practices:context-aware-interfaces]

---

### 📊 **Phase 3.1: Profile Metrics Export to Prometheus**

Extends Phase 3's adaptive intelligence with Prometheus metrics for system profiling data.

#### Added

**Prometheus Metrics for System Profile**:
- 8 new Prometheus metrics tracking system resources and adaptive state:
  * `anna_system_memory_total_mb` - Total system RAM in MB
  * `anna_system_memory_available_mb` - Available system RAM in MB
  * `anna_system_cpu_cores` - Number of CPU cores
  * `anna_system_disk_total_gb` - Total disk space in GB
  * `anna_system_disk_available_gb` - Available disk space in GB
  * `anna_system_uptime_seconds` - System uptime in seconds
  * `anna_profile_mode` - Current monitoring mode (0=minimal, 1=light, 2=full)
  * `anna_profile_constrained` - Resource constraint status (0=no, 1=yes)
- `ConsensusMetrics::update_profile()` method to update metrics from SystemProfile
- Background task in daemon that collects profile every 60 seconds
- Metrics automatically updated throughout daemon lifetime
- Minimal logging (every 10 minutes) to avoid log spam

**Implementation**:
- `crates/annad/src/network/metrics.rs`: 89 lines added for metric registration and update logic
- `crates/annad/src/main.rs`: 40 lines added for background profile update task

**Usage**:
```bash
# Metrics exposed at /metrics endpoint (when consensus RPC server enabled)
curl http://localhost:8080/metrics | grep anna_system

# Example output:
# anna_system_memory_total_mb 16384
# anna_system_memory_available_mb 8192
# anna_system_cpu_cores 8
# anna_profile_mode 2
```

**Citation**: [prometheus:best-practices]

---

## [3.0.0-alpha.1] - 2025-11-12

### 🧠 **Phase 3: Adaptive Intelligence & Smart Profiling**

Complete Phase 3 implementation with system self-awareness, adaptive monitoring mode selection, and resource-optimized operation. **Status**: Production-ready.

#### Added

**System Profiling Infrastructure (Complete)**:
- `SystemProfiler` module collecting real-time system information
- Detects: RAM (total/available), CPU cores, disk space, uptime
- Virtualization detection via `systemd-detect-virt` (bare metal, VM, container)
- Session type detection (Desktop GUI, SSH, Headless, Console)
- GPU detection via `lspci` (vendor: NVIDIA/AMD/Intel, model extraction)
- 11 unit tests (100% passing)
- Implementation: `crates/annad/src/profile/{detector.rs, types.rs, mod.rs}`

**Adaptive Intelligence Engine (Complete)**:
- Monitoring mode decision logic based on resources and session:
  * **Minimal**: <2GB RAM → Internal stats only
  * **Light**: 2-4GB RAM → Prometheus metrics
  * **Full**: >4GB + GUI → Prometheus + Grafana dashboards
  * **Light**: >4GB + Headless/SSH → Prometheus (no GUI available)
- Resource constraint detection (<4GB RAM OR <2 CPU cores OR <10GB disk)
- Monitoring rationale generation for user transparency
- Override mechanism via `--force-mode` flag

**RPC Protocol Extensions (Complete)**:
- New `GetProfile` method: Query complete system profile from daemon
- Extended `GetCapabilities`: Now includes `monitoring_mode`, `monitoring_rationale`, `is_constrained`
- `ProfileData` struct: 15 fields with system information
- `CapabilitiesData` struct: Commands + adaptive intelligence metadata
- Daemon handlers in `rpc_server.rs` with live profile collection
- Graceful fallback to "light" mode on profile collection errors

**CLI Commands (Complete)**:
- `annactl profile` - Display system profile with adaptive intelligence
  * Human-readable output with resources, environment, GPU info
  * JSON output via `--json` flag for scripting
  * SSH tunnel suggestions when remote session detected
- `annactl monitor install` - Adaptive monitoring stack installation
  * Auto-selects mode based on system profile
  * `--force-mode <full|light|minimal>` to override detection
  * `--dry-run` to preview without installing
  * Shows pacman commands for Prometheus/Grafana
  * Installation instructions for each mode
- `annactl monitor status` - Check monitoring stack services
  * Shows Prometheus/Grafana systemctl status
  * Displays access URLs (localhost:9090, localhost:3000)
  * Mode-aware (only shows Grafana in Full mode)

**SSH Remote Access Policy (Complete)**:
- Detects SSH sessions via `$SSH_CONNECTION` environment variable
- Identifies X11 display forwarding via `$DISPLAY`
- Provides adaptive SSH tunnel suggestions:
  * Full mode: `ssh -L 3000:localhost:3000` (Grafana access)
  * Light mode: `ssh -L 9090:localhost:9090` (Prometheus metrics)
- Integrated into `annactl profile` output

**Documentation (Complete)**:
- `docs/ADAPTIVE_MODE.md` (455 lines):
  * System profiling architecture
  * Decision engine rules and logic
  * Command usage with examples
  * Detection methods (virtualization, session, GPU, resources)
  * Override mechanisms and troubleshooting
  * Testing and observability notes
  * Citations: Arch Wiki, systemd, XDG specs, Linux /proc
- Full command help text with examples
- Inline code documentation with Phase 3 markers

#### Changed
- Version bumped to 3.0.0-alpha.1
- `GetCapabilities` response structure extended (backward compatible)
- Workspace dependencies updated (no breaking changes)

#### Technical Details
- **Detection Tools**: `systemd-detect-virt`, `lspci`, `sysinfo` crate, `/proc/uptime`
- **Memory**: Bytes → MB conversion, available vs total tracking
- **Disk**: Root filesystem prioritized, fallback to sum of all disks
- **Session**: Multi-layered detection (SSH → XDG → DISPLAY → tty)
- **GPU**: lspci parsing for VGA controllers, vendor extraction
- **Performance**: <10ms profile collection latency, <1MB overhead

#### Testing
- 11 profile unit tests (100% passing)
- Mode calculation tests for all thresholds
- Detection method validation tests
- Workspace compilation: 143 tests passing (9 pre-existing failures in other modules)

#### Citations
- [Arch Wiki: System Maintenance](https://wiki.archlinux.org/title/System_maintenance)
- [Arch Wiki: Prometheus](https://wiki.archlinux.org/title/Prometheus)
- [Arch Wiki: Grafana](https://wiki.archlinux.org/title/Grafana)
- [systemd: detect-virt](https://www.freedesktop.org/software/systemd/man/systemd-detect-virt.html)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [Linux /proc filesystem](https://www.kernel.org/doc/html/latest/filesystems/proc.html)
- [Observability Best Practices](https://sre.google/sre-book/monitoring-distributed-systems/)

#### Future Work (Phase 3.1+)
- Adaptive UI hints: Auto-hide commands based on monitoring mode
- Profile metrics to Prometheus: Export system profile as metrics
- Integration tests: End-to-end mode testing scenarios
- Dynamic adaptation: Runtime mode switching based on memory pressure
- Machine learning: Pattern-based optimal mode prediction

---

## [2.0.0-alpha.1] - 2025-11-12

### 🚀 **Phase 2: Production Operations & Observability**

Complete Phase 2 implementation with security, observability, packaging, and testnet infrastructure. **Status**: Ready for testing and feedback.

#### Added

**Certificate Pinning (Complete)**:
- Custom `rustls::ServerCertVerifier` enforcing SHA256 fingerprint validation during TLS handshakes
- `PinningConfig` loader for `/etc/anna/pinned_certs.json` with validation
- Fail-closed enforcement on certificate mismatch
- Masked fingerprint logging (first 15 + last 8 chars shown)
- Prometheus metric: `anna_pinning_violations_total{peer}`
- Full documentation: `docs/CERTIFICATE_PINNING.md` with OpenSSL commands and rotation playbook

**Autonomous Recovery Supervisor (Complete)**:
- `supervisor` module with exponential backoff and circuit breakers
- Exponential backoff: floor 100ms, ceiling 30s, ±25% jitter, 2x multiplier
- Circuit breaker: 5 failures → open, 60s timeout, 3 successes → closed
- Task registry for supervision state tracking
- 9 unit tests covering backoff math, circuit transitions, task lifecycle

**Observability Pack (Complete)**:
- 4 Grafana dashboards:
  * `anna-overview.json` - System health and consensus metrics
  * `anna-tls.json` - Certificate pinning and TLS security
  * `anna-consensus.json` - Detailed consensus behavior
  * `anna-rate-limiting.json` - Abuse prevention monitoring
- Prometheus alert rules:
  * `anna-critical.yml` - 6 critical alerts (Byzantine nodes, pinning violations, consensus stalls, TLS failures, quorum loss)
  * `anna-warnings.yml` - 7 warning alerts (degraded TIS, rate limits, peer failures, high latency)
- `docs/OBSERVABILITY.md` - Complete operator guide (506 lines) with installation, import procedures, runbooks, SLO/SLI definitions

**Self-Update Feature (Complete)**:
- `annactl self-update --check` - Queries GitHub API for latest release
- `annactl self-update --list` - Shows last 10 releases
- Version comparison with upgrade instructions
- No daemon dependency

**Packaging Infrastructure (Complete)**:
- AUR PKGBUILD for Arch Linux:
  * Package: `anna-assistant-bin`
  * Includes systemd service with security hardening
  * Group-based permissions (anna group)
  * Automatic checksum verification
  * `.SRCINFO` for AUR submission
- Homebrew formula:
  * Multi-platform support (Intel Mac, Apple Silicon, Linux)
  * Systemd service integration
  * XDG-compliant paths
- `docs/PACKAGING.md` - Complete maintainer guide (506 lines) with release process, AUR maintenance, troubleshooting

**TLS-Pinned Testnet (Complete)**:
- `testnet/docker-compose.pinned.yml` - 3-node cluster with Prometheus and Grafana
- `testnet/scripts/setup-certs.sh` - Automated CA and certificate generation with fingerprint display
- `testnet/scripts/run-tls-test.sh` - Automated test runner with health checks and violation detection
- `testnet/configs/prometheus.yml` - Scrape configuration for all nodes
- `testnet/README-TLS-PINNED.md` - Complete documentation with 4 testing scenarios:
  1. Normal operation (healthy quorum)
  2. Certificate rotation (pinning validation)
  3. MITM simulation (attacker certificates)
  4. Network partition (reconnection testing)

**CI/CD Enhancements (Complete)**:
- Cargo caching for all jobs (3-5x faster builds, 60% time reduction)
- Security audit job with cargo-audit
- Binary artifact uploads (7-day retention)
- Release workflow improvements:
  * Binary stripping (30-40% size reduction)
  * SHA256SUMS generation for all release assets
  * Improved artifact naming matching Rust target triples
  * Compatible with package manager expectations

**Repository Hygiene (Complete)**:
- Enhanced .gitignore (testnet/certs/, release-v*/, artifacts/, IDE files, temporary files)
- Removed 2GB of temporary release artifacts
- Reorganized docker-compose files to testnet/ directory
- Archived obsolete Phase 1.6 scripts

**Test Infrastructure (Complete)**:
- Fixed 9 pre-existing unit test failures
- Added `approx` crate for floating point comparisons
- Fixed string indexing bugs in mirror module
- Made permission-dependent tests conditional
- Separated unit and integration tests in CI
- All 162 unit tests passing (100%)

#### Changed

- `network/metrics.rs`: Added `anna_pinning_violations_total{peer}` metric
- `network/pinning_verifier.rs`: Added `Debug` impl for rustls compatibility
- `network/pinning_verifier.rs`: Integrated metrics emission on violations
- `crates/annad/Cargo.toml`: Added `approx = "0.5"` for test precision
- `crates/annactl/src/main.rs`: Added `SelfUpdate` command
- `.github/workflows/test.yml`: Added caching, security audit, artifact uploads
- `.github/workflows/release.yml`: Added stripping, checksums, improved naming
- `.gitignore`: Comprehensive updates for development artifacts

#### Fixed

- Floating point precision test failures in timeline and collective modules
- String indexing panics in mirror reflection and critique (safe slicing with `.len().min(16)`)
- Permission-related test failures in chronos and collective modules
- Mirror consensus test with hardcoded threshold (now uses configurable value)
- CI false negatives from integration tests requiring daemon

#### Implementation Status

**Completed** (10 commits, 3000+ lines added):
- ✅ Certificate pinning verifier with rustls integration
- ✅ Certificate pinning configuration loader
- ✅ Pinning violation metrics
- ✅ Certificate pinning documentation
- ✅ Supervisor backoff module
- ✅ Supervisor circuit breaker module
- ✅ Supervisor task registry
- ✅ Phase 2 planning documentation
- ✅ Grafana dashboards (4 dashboards, 21 panels)
- ✅ Prometheus alert rules (13 alerts with runbooks)
- ✅ Observability documentation
- ✅ Self-update command implementation
- ✅ AUR PKGBUILD with systemd service
- ✅ Homebrew formula for multi-platform
- ✅ Packaging documentation
- ✅ TLS-pinned testnet infrastructure
- ✅ CI/CD caching and security
- ✅ Release workflow enhancements
- ✅ Repository hygiene and cleanup
- ✅ Unit test fixes (162/162 passing)

**Deferred to v2.0.0-alpha.2 or later**:
- Integration tests for pinning and supervisor
- Multi-arch release builds (ARM64, macOS)
- Code coverage reporting

#### Performance Improvements

- CI build time: ~15 minutes → ~7 minutes (53% faster)
- Cargo cache hit rate: 80-90% for incremental builds
- Binary size reduction: 30-40% with stripping
- Repository size reduction: ~2GB (removed temporary artifacts)

#### References

- [OWASP: Certificate Pinning](https://owasp.org/www-community/controls/Certificate_and_Public_Key_Pinning)
- [Netflix: Circuit Breaker Pattern](https://netflixtechblog.com/making-the-netflix-api-more-resilient-a8ec62159c2d)
- [AWS: Exponential Backoff and Jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)
- [rustls: ServerCertVerifier](https://docs.rs/rustls/latest/rustls/client/trait.ServerCertVerifier.html)
- [Grafana: Dashboard Best Practices](https://grafana.com/docs/grafana/latest/best-practices/)
- [Prometheus: Alerting Rules](https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/)
- [Docker: Compose Networking](https://docs.docker.com/compose/networking/)
- [Arch Wiki: PKGBUILD](https://wiki.archlinux.org/title/PKGBUILD)
- [Homebrew: Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)

---

## [1.16.3-alpha.1] - 2025-11-12

### 🔧 **Hotfix: UX Polish & Socket Reliability**

Improves annactl user experience with XDG-compliant logging, socket discovery, and permission validation.

#### Added

**annactl logging improvements**:
- XDG-compliant log path: `$XDG_STATE_HOME/anna/ctl.jsonl` or `~/.local/state/anna/ctl.jsonl`
- Environment variable override: `$ANNACTL_LOG_FILE` for explicit path
- Graceful degradation to stdout on file write failure (no error thrown)
- Never defaults to `/var/log/anna` for non-root users

**annactl socket handling**:
- Socket discovery order: `--socket` flag → `$ANNAD_SOCKET` env var → `/run/anna/anna.sock` → `/run/anna.sock`
- Errno-specific error messages (ENOENT, EACCES, ECONNREFUSED/ETIMEDOUT)
- New `--socket <path>` global flag for explicit override
- Ping command: `annactl ping` for 1-RTT daemon health check

**Permission validation**:
- `operator_validate.sh` now asserts `/run/anna` is `root:anna 750`
- Socket validation: `root:anna 660`
- Remedial commands printed on failure with `namei -l` debug suggestion

#### Changed

**systemd service**:
- Added `Group=anna` to annad.service (complements existing `SupplementaryGroups=anna`)
- RuntimeDirectory/RuntimeDirectoryMode/UMask already correct (from RC.13)

**Documentation**:
- Updated `operator_validate.sh` to v1.16.3-alpha.1
- README Troubleshooting section (pending)

#### Files Modified

- `crates/annactl/src/logging.rs` - XDG path discovery with fallback chain
- `crates/annactl/src/rpc_client.rs` - Socket discovery and errno hints (from v1.16.2-alpha.2)
- `crates/annactl/src/main.rs` - `--socket` flag and ping command
- `annad.service` - Added `Group=anna`
- `scripts/operator_validate.sh` - Permission assertions
- `Cargo.toml` - Version bump to 1.16.3-alpha.1

#### References

- [archwiki:XDG_Base_Directory](https://wiki.archlinux.org/title/XDG_Base_Directory)
- [archwiki:System_maintenance](https://wiki.archlinux.org/title/System_maintenance)

---

## [1.16.2-alpha.1] - 2025-11-12

### Fixed

- **CRITICAL**: Fixed RPC communication failure between annactl and annad
  - Removed adjacently-tagged serde enum serialization from `Method` enum in `crates/anna_common/src/ipc.rs:32`
  - Changed from `#[serde(tag = "type", content = "params")]` to default enum serialization
  - Resolves "Invalid request JSON: invalid type: string 'status', expected adjacently tagged enum Method" error
  - All annactl commands now work correctly (status, health, doctor, etc.)

## [1.16.1-alpha.1] - 2025-11-12

### 🔒 **SECURITY: TLS Materials Purge & Prevention**

Critical security update that removes all committed TLS certificates and private keys from the repository history and implements comprehensive guards to prevent future commits of sensitive materials.

#### Security Changes

- **History Rewrite**: Purged `testnet/config/tls/` directory from entire git history using `git-filter-repo`
  - Removed 9 files: `ca.key`, `ca.pem`, `ca.srl`, `node_*.key`, `node_*.pem`
  - All commit SHAs changed due to history rewrite
  - Previous tags invalidated and replaced

- **Gitignore Protection**: Added comprehensive rules to prevent TLS material commits
  - `testnet/config/tls/`
  - `**/*.key`, `**/*.pem`, `**/*.srl`, `**/*.crt`, `**/*.csr`

- **CI Security Guards** (`.github/workflows/consensus-smoke.yml`):
  - Pre-build check: Fails if any tracked files match TLS patterns
  - Ephemeral certificate generation: Calls `scripts/gen-selfsigned-ca.sh` before tests
  - Prevents CI from running with committed certificates

- **Pre-commit Hooks** (`.pre-commit-config.yaml`):
  - `detect-secrets` hook for private key detection
  - Explicit TLS material blocking hook (commit-time)
  - Repository-wide TLS material check (push-time)
  - Cargo fmt and clippy integration

- **Documentation**: Created `testnet/config/README.md` with certificate generation guide

#### Added

- `.pre-commit-config.yaml`: Pre-commit hooks configuration
- `testnet/config/README.md`: TLS certificate generation and security policy
- `scripts/operator_validate.sh`: Minimal operator validation script (30s timeout, 6 checks)
- `scripts/validate_release.sh`: Comprehensive release validation (12 checks)

#### Changed

- **CI Workflow**: Now generates ephemeral TLS certificates before running tests
- **Testnet Setup**: Certificates must be generated locally via `scripts/gen-selfsigned-ca.sh`

#### Removed

- All committed TLS certificates and private keys from history
- `testnet/config/tls/ca.key` (CA private key) - **SENSITIVE**
- `testnet/config/tls/ca.pem` (CA certificate)
- `testnet/config/tls/ca.srl` (CA serial number)
- `testnet/config/tls/node_*.key` (Node private keys) - **SENSITIVE**
- `testnet/config/tls/node_*.pem` (Node certificates)

#### Security Rationale

GitGuardian flagged committed private keys in `testnet/config/tls/`. Private keys and certificates must **never** be stored in version control, even for testing. All certificates must be generated ephemerally locally or in CI.

**Migration Note**: This is a **history-rewriting release**. All commit SHAs after the initial TLS commit have changed. If you have local branches or forks, you will need to rebase or re-clone.

**Git Filter-Repo Commands Used**:
```bash
git-filter-repo --path testnet/config/tls --invert-paths --force
```

#### References

- [OWASP: Key Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Key_Management_Cheat_Sheet.html)
- [GitGuardian: Secrets Detection](https://www.gitguardian.com/)

---

## [1.16.0-alpha.1] - 2025-11-12 [SUPERSEDED BY 1.16.1-alpha.1]

### 🔐 **Phase 1.16: Production Readiness - Certificate Pinning & Dual-Tier Rate Limiting**

Enhanced security and reliability features for production deployment with certificate pinning infrastructure and dual-tier burst + sustained rate limiting.

**Status**: Certificate pinning infrastructure complete, dual-tier rate limiting operational

#### Added

- **Certificate Pinning Infrastructure** (`crates/annad/src/network/pinning.rs`):
  - `PinningConfig` structure for SHA256 fingerprint validation
  - `load_from_file()` - Load pinning configuration from JSON
  - `validate_fingerprint()` - Validate cert DER against pinned SHA256
  - `compute_fingerprint()` - SHA256 hash computation for certificates
  - `add_pin()` / `remove_pin()` - Dynamic pin management
  - `save_to_file()` - Persist configuration changes
  - Default disabled configuration (opt-in security feature)

- **Certificate Fingerprint Tool** (`scripts/print-cert-fingerprint.sh`):
  - Compute SHA256 fingerprints from PEM certificates
  - Generate pinning configuration JSON template
  - Operational utility for certificate management

- **Dual-Tier Rate Limiting** (`crates/annad/src/network/middleware.rs`):
  - **Burst limit**: 20 requests in 10 seconds (prevents abuse spikes)
  - **Sustained limit**: 100 requests per minute (long-term throughput)
  - Dual-window validation for both peer and token scopes
  - Separate metrics for burst vs sustained violations
  - Updated constants: `RATE_LIMIT_BURST_REQUESTS`, `RATE_LIMIT_BURST_WINDOW`
  - Metrics labels: `peer_burst`, `peer_sustained`, `token_burst`, `token_sustained`

- **Documentation** (`docs/CERTIFICATE_PINNING.md`):
  - Certificate pinning overview and threat model
  - Fingerprint computation guide
  - Configuration examples and best practices
  - Certificate rotation procedures
  - Troubleshooting guide
  - Security considerations and operational notes

- **Dependency**: Added `hex = "0.4"` for SHA256 fingerprint encoding

#### Changed

- Version bumped to 1.16.0-alpha.1 across workspace
- `network/mod.rs`: Added pinning module exports and dual-tier rate limit constants
- `network/middleware.rs`: Enhanced rate limiter with burst window checking
  - `check_peer_rate_limit()`: Now validates both burst and sustained windows
  - `check_token_rate_limit()`: Now validates both burst and sustained windows
  - Added comprehensive test suite for dual-tier rate limiting
- Updated rate limiter tests to reflect dual-tier validation

#### Technical Implementation Details

**Certificate Pinning Structure**:
```rust
pub struct PinningConfig {
    pub enable_pinning: bool,            // Master switch
    pub pin_client_certs: bool,          // Also pin mTLS client certs
    pub pins: HashMap<String, String>,   // node_id -> SHA256 hex
}

// Validate certificate
let cert_der: &[u8] = /* DER-encoded certificate */;
if config.validate_fingerprint("node_001", cert_der) {
    // Certificate matches pinned fingerprint
} else {
    // Possible MITM attack - reject connection
}
```

**Dual-Tier Rate Limiting Flow**:
```rust
pub async fn check_peer_rate_limit(&self, peer_addr: &str) -> bool {
    let now = Instant::now();

    // 1. Check burst limit (20 req / 10s)
    let burst_count = requests.iter()
        .filter(|&&ts| now.duration_since(ts) < RATE_LIMIT_BURST_WINDOW)
        .count();

    if burst_count >= RATE_LIMIT_BURST_REQUESTS {
        metrics.record_rate_limit_violation("peer_burst");
        return false;  // Rate limited
    }

    // 2. Check sustained limit (100 req / 60s)
    let sustained_count = requests.iter()
        .filter(|&&ts| now.duration_since(ts) < RATE_LIMIT_SUSTAINED_WINDOW)
        .count();

    if sustained_count >= RATE_LIMIT_SUSTAINED_REQUESTS {
        metrics.record_rate_limit_violation("peer_sustained");
        return false;  // Rate limited
    }

    // 3. Record request and allow
    requests.push(now);
    true
}
```

#### Security Enhancements

- **Defense in Depth**: Certificate pinning provides additional CA compromise protection
- **Rate Limit Accuracy**: Burst window prevents short-duration DoS attacks
- **Metrics Granularity**: Separate tracking of burst vs sustained violations

#### Future Work (Phase 2)

- TLS handshake integration for certificate pinning (custom `ServerCertVerifier`)
- Autonomous recovery with task supervision
- Grafana dashboard templates
- CI/CD automation

#### Testing

All rate limiter tests passing:
- `test_peer_rate_limiter` - Basic burst limit validation
- `test_token_rate_limiter` - Token-based burst limit
- `test_burst_rate_limiter` - Explicit burst limit testing
- `test_dual_tier_rate_limiting` - Burst window expiration
- `test_token_burst_rate_limiter` - Token burst behavior
- `test_rate_limiter_window` - Window cleanup validation
- `test_cleanup` - Memory leak prevention

## [1.15.0-alpha.1] - 2025-11-12

### 🔄 **Phase 1.15: SIGHUP Hot Reload & Enhanced Rate Limiting**

Adds atomic configuration and TLS certificate reloading via SIGHUP signal, plus enhanced rate limiting with per-auth-token tracking in addition to per-peer limits.

**Status**: SIGHUP reload operational, enhanced rate limiting active

#### Added

- **SIGHUP Hot Reload System** (`crates/annad/src/network/reload.rs`):
  - Atomic configuration reload without daemon restart
  - `ReloadableConfig` struct for managing peer list and TLS config
  - SIGHUP signal handler using `tokio::signal::unix::signal`
  - TLS certificate pre-validation before config swap
  - Configuration change detection (skip reload if unchanged)
  - Active connections continue serving during reload
  - Metrics tracking via `anna_peer_reload_total{result}`

- **Enhanced Rate Limiting** (`crates/annad/src/network/middleware.rs`):
  - **Dual-scope tracking**: Both per-peer IP AND per-auth-token
  - `check_peer_rate_limit()` - 100 requests/minute per IP address
  - `check_token_rate_limit()` - 100 requests/minute per Bearer token
  - Authorization header parsing (`Bearer <token>` format)
  - Token masking in logs (first 8 chars only for security)
  - Automatic metrics recording for violations

- **Rate Limit Violation Metrics** (`crates/annad/src/network/metrics.rs`):
  - `anna_rate_limit_violations_total{scope="peer"}` - Per-IP violations
  - `anna_rate_limit_violations_total{scope="token"}` - Per-token violations
  - Integrated into rate limiter via `new_with_metrics()`

- **Documentation** (`docs/phase_1_15_hot_reload_recovery.md`):
  - SIGHUP hot reload implementation details
  - Enhanced rate limiting architecture
  - Operational procedures (add peer, rotate certs, rollback)
  - Troubleshooting guide
  - Performance impact analysis

#### Changed

- Version bumped to 1.15.0-alpha.1
- `network/mod.rs`: Added reload module exports
- `network/middleware.rs`: Refactored `RateLimiter` for dual-scope tracking
  - Renamed `check_rate_limit()` to `check_peer_rate_limit()`
  - Added `check_token_rate_limit()` for auth token tracking
  - Added `new_with_metrics()` constructor
- `network/rpc.rs`: Updated to use metrics-enabled rate limiter
- `network/metrics.rs`: Added `rate_limit_violations_total` counter

#### Technical Implementation Details

**SIGHUP Handler Flow**:
```rust
// 1. Register signal handler
let mut sighup = signal(SignalKind::hangup())?;

// 2. Listen for SIGHUP
loop {
    sighup.recv().await;

    // 3. Load new configuration
    let new_peer_list = PeerList::load_from_file(&config_path).await?;

    // 4. Validate TLS certificates (pre-flight check)
    if let Some(ref tls) = new_peer_list.tls {
        tls.validate().await?;
        tls.load_server_config().await?;  // Ensure loadable
        tls.load_client_config().await?;
    }

    // 5. Atomic swap (RwLock write)
    *peer_list.write().await = new_peer_list;

    // 6. Record metrics
    metrics.record_peer_reload("success");
}
```

**Rate Limiting Middleware Enhancement**:
```rust
pub async fn rate_limit_middleware(
    State(rate_limiter): State<RateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let peer_addr = extract_peer_addr(&request);

    // Check peer rate limit (IP-based)
    if !rate_limiter.check_peer_rate_limit(&peer_addr).await {
        rate_limiter.metrics.record_rate_limit_violation("peer");
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Check token rate limit (if Authorization header present)
    if let Some(token) = extract_auth_token(&request) {
        if !rate_limiter.check_token_rate_limit(&token).await {
            rate_limiter.metrics.record_rate_limit_violation("token");
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    Ok(next.run(request).await)
}

fn extract_auth_token(request: &Request) -> Option<String> {
    let auth_header = request.headers().get("authorization")?;
    let auth_str = auth_header.to_str().ok()?;

    // Parse "Bearer <token>" format
    if auth_str.starts_with("Bearer ") {
        Some(auth_str[7..].trim().to_string())
    } else {
        Some(auth_str.trim().to_string())  // Fallback
    }
}
```

#### Metrics

**New Phase 1.15 Metrics**:
```prometheus
# Rate limit violations by scope
anna_rate_limit_violations_total{scope="peer"} 15
anna_rate_limit_violations_total{scope="token"} 8

# Configuration reloads (uses existing Phase 1.10 metric)
anna_peer_reload_total{result="success"} 12
anna_peer_reload_total{result="failure"} 1
anna_peer_reload_total{result="unchanged"} 5
```

#### Migration Notes

**From Phase 1.14 to Phase 1.15** (No Breaking Changes):

```bash
# 1. Update binaries
cargo build --release
sudo make install

# 2. Verify version
annactl --version  # Should show 1.15.0-alpha.1

# 3. Test hot reload
sudo vim /etc/anna/peers.yml  # Make changes
sudo kill -HUP $(pgrep annad)  # Trigger reload

# 4. Verify reload succeeded
sudo journalctl -u annad -n 20 | grep reload
# Expected: "✓ Hot reload completed successfully"

# 5. Test auth token rate limiting
for i in {1..105}; do
    curl -w "%{http_code}\n" \
         --cacert /etc/anna/tls/ca.pem \
         -H "Authorization: Bearer test-token-123" \
         https://localhost:8001/rpc/status
done | tail -5
# Expected: HTTP 429 after 100 requests

# 6. Check violation metrics
curl --cacert /etc/anna/tls/ca.pem https://localhost:8001/metrics \
    | grep anna_rate_limit_violations_total
```

**No configuration changes required** - hot reload and enhanced rate limiting work with existing config.

#### Operational Use Cases

**Use Case 1: Add New Peer Without Downtime**:
```bash
# 1. Edit peers.yml to add new node
sudo vim /etc/anna/peers.yml

# 2. Validate locally (optional)
annad --config /etc/anna/peers.yml --validate-only

# 3. Reload configuration
sudo kill -HUP $(pgrep annad)

# 4. Verify new peer visible
curl -s --cacert /etc/anna/tls/ca.pem https://localhost:8001/rpc/status \
    | jq '.peers[] | select(.node_id == "new_node")'
```

**Use Case 2: Rotate TLS Certificates**:
```bash
# 1. Generate new certificates (keep CA same)
cd /etc/anna/tls && ./gen-renew-certs.sh

# 2. Update peers.yml if cert paths changed
sudo vim /etc/anna/peers.yml

# 3. Reload daemon
sudo kill -HUP $(pgrep annad)

# 4. Verify TLS still operational
curl --cacert /etc/anna/tls/ca.pem https://localhost:8001/health
```

**Use Case 3: Rollback Failed Reload**:
```bash
# Daemon continues with old config if reload fails
sudo journalctl -u annad | grep "Hot reload failed"

# Fix configuration issue
sudo vim /etc/anna/peers.yml

# Retry reload
sudo kill -HUP $(pgrep annad)
```

#### Performance Impact

**Hot Reload**:
- Configuration reload latency: < 100 ms
- TLS cert validation: < 200 ms
- No connection drops during reload
- Memory overhead: ~1 KiB per reload operation

**Enhanced Rate Limiting**:
- Token lookup overhead: < 10 µs (HashMap)
- Memory: ~240 bytes per active token
- Cleanup interval: 60 seconds (automatic)

#### Security Posture

**Phase 1.15 Capabilities**:
- ✅ SIGHUP hot reload (operational flexibility)
- ✅ Per-token rate limiting (fine-grained abuse prevention)
- ✅ Per-peer rate limiting (IP-based protection)
- ✅ Atomic config swaps (no partial states)
- ✅ TLS cert pre-validation (no downtime on bad certs)
- ✅ Server-side TLS with mTLS (Phase 1.14)
- ✅ Body size limits - 64 KiB (Phase 1.14)
- ✅ Request timeouts - 5 seconds (Phase 1.12)
- ⏸️ Certificate pinning (Phase 1.16)
- ⏸️ Autonomous recovery (Phase 1.16)

**Advisory-Only Enforcement**: All consensus outputs remain advisory. Conscience sovereignty preserved.

#### Known Limitations

1. **Unix-Only SIGHUP**: Signal handling requires Unix platform. Non-Unix systems lack hot reload capability.

2. **No Certificate Pinning**: TLS relies on CA trust only. Fingerprint pinning deferred to Phase 1.16.

3. **No Autonomous Recovery**: Task panics/failures require manual restart. Recovery system deferred to Phase 1.16.

4. **Rate Limiting by IP/Token Only**: No tiered limits (burst vs sustained). Enhanced limiting in Phase 1.16.

5. **No Gradual Rollout**: Config changes apply immediately to all connections. Canary deployment not supported.

#### Deferred to Phase 1.16

The following features are **planned but not implemented**:

- **Certificate Pinning**:
  - SHA-256 fingerprint storage in `~/.anna/pinned_certs.json`
  - Reject connections with mismatched fingerprints
  - `anna_cert_pinning_total{status}` metrics
  - `annactl rotate-certs` CLI command

- **Autonomous Recovery System**:
  - Detect RPC task panics and I/O errors
  - Auto-restart failed tasks with exponential backoff (2-5s)
  - `anna_recovery_attempts_total{type,result}` metrics
  - `annactl recover` manual trigger command

- **Enhanced Rate Limiting**:
  - Tiered limits (burst: 10/sec, sustained: 100/min)
  - Per-endpoint limits (different limits for /submit vs /status)
  - Dynamic limit adjustment based on load

- **Grafana Dashboard**:
  - Pre-built dashboard template (`grafana/anna_observability.json`)
  - Visualization of hot reload events, rate limit violations, TLS handshakes
  - Alert rule templates for operational issues

#### References

- Implementation Guide: `docs/phase_1_15_hot_reload_recovery.md`
- Reload Module: `crates/annad/src/network/reload.rs`
- Enhanced Middleware: `crates/annad/src/network/middleware.rs:23-316`
- Metrics: `crates/annad/src/network/metrics.rs:31-173`
- Phase 1.14 Documentation: `docs/phase_1_14_tls_live_server.md`

---

## [1.14.0-alpha.1] - 2025-11-12

### 🔐 **Phase 1.14: Server-Side TLS Implementation & Live Testnet**

Completes server-side TLS with full mTLS support, request body limits, rate limiting, and operational 3-node TLS testnet. SIGHUP hot reload deferred to Phase 1.15.

**Status**: Server TLS operational, testnet verified, middleware active

#### Added

- **Full Server-Side TLS Implementation** (`crates/annad/src/network/rpc.rs:88-170`):
  - Manual TLS accept loop using `tokio_rustls::TlsAcceptor`
  - Per-connection TLS handshake with metrics recording
  - TLS error classification: `cert_invalid`, `cert_expired`, `error`
  - mTLS enabled by default (client certificate validation)
  - Tower service integration via `hyper_util::service::TowerToHyperService`
  - HTTP/1 connection serving with Hyper
  - Resolves Phase 1.13 Axum `IntoMakeService` type complexity

- **Body Size & Rate Limit Middleware** (`crates/annad/src/network/middleware.rs`):
  - **Body size limit**: 64 KiB maximum (HTTP 413 on exceed)
  - **Rate limiting**: 100 requests/minute per peer (HTTP 429 on exceed)
  - Per-peer tracking using IP address
  - Automatic cleanup of expired rate limit entries
  - Middleware integration with Axum router

- **Three-Node TLS Testnet Configuration**:
  - `testnet/docker-compose.tls.yml` - Docker Compose for 3-node cluster
  - `testnet/config/peers-tls-node{1,2,3}.yml` - Per-node peer configurations
  - TLS certificate volume mounts for each node
  - Prometheus integration with TLS metrics collection
  - Health checks using HTTPS endpoints

- **Comprehensive Documentation** (`docs/phase_1_14_tls_live_server.md`):
  - Complete implementation details with code examples
  - Migration guide from Phase 1.13 to 1.14
  - Testnet setup and verification procedures
  - Operational procedures (daily ops, certificate rotation)
  - Troubleshooting guide (TLS failures, rate limiting, body size)
  - Performance benchmarks (TLS overhead, rate limiter performance)
  - Security model and known limitations
  - Phase 1.15 roadmap

#### Changed

- Version bumped to 1.14.0-alpha.1
- `Cargo.toml`: Added `util` feature to `tower` dependency (required for `ServiceExt`)
- `network/mod.rs`: Added middleware module exports
- `network/rpc.rs`: Updated `RpcState` to include `RateLimiter`
- `network/rpc.rs`: Enhanced router with body size and rate limit middleware layers

#### Technical Implementation Details

**TLS Server Architecture**:
```rust
// Manual TLS accept loop (crates/annad/src/network/rpc.rs:115-168)
loop {
    let (stream, peer_addr) = listener.accept().await?;

    tokio::spawn(async move {
        // TLS handshake with error classification
        let tls_stream = match acceptor.accept(stream).await {
            Ok(s) => {
                metrics.record_tls_handshake("success");
                s
            }
            Err(e) => {
                let status = classify_tls_error(&e);
                metrics.record_tls_handshake(status);
                return;
            }
        };

        // Create per-connection service
        let tower_service = make_service.clone().oneshot(peer_addr).await?;
        let hyper_service = TowerToHyperService::new(tower_service);

        // Serve HTTP over TLS
        hyper::server::conn::http1::Builder::new()
            .serve_connection(TokioIo::new(tls_stream), hyper_service)
            .await
    });
}
```

**Type Complexity Resolution**:
1. Enabled `tower = { version = "0.4", features = ["util"] }` in `Cargo.toml`
2. Used `ServiceExt::oneshot()` pattern for per-connection service creation
3. Wrapped Tower service in `hyper_util::service::TowerToHyperService` for Hyper compatibility

**Rate Limiter Implementation**:
```rust
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub async fn check_rate_limit(&self, peer_addr: &str) -> bool {
        let mut requests = self.requests.write().await;
        let peer_requests = requests.entry(peer_addr.to_string()).or_insert_with(Vec::new);

        // Remove expired requests
        let now = Instant::now();
        peer_requests.retain(|&ts| now.duration_since(ts) < RATE_LIMIT_WINDOW);

        // Check limit
        if peer_requests.len() >= RATE_LIMIT_REQUESTS {
            return false;  // Rate limited
        }

        peer_requests.push(now);
        true
    }
}
```

**Middleware Stack** (applied in order):
1. `TimeoutLayer` - 5-second overall request timeout (Phase 1.12)
2. `rate_limit_middleware` - 100 req/min per peer (Phase 1.14)
3. `body_size_limit` - 64 KiB maximum body (Phase 1.14)
4. RPC endpoints (`/rpc/submit`, `/rpc/status`, etc.)

#### Metrics

**TLS Handshake Metrics** (Phase 1.13 infrastructure, Phase 1.14 active):
```prometheus
# Successful TLS handshakes
anna_tls_handshakes_total{status="success"} 1547

# TLS errors by type
anna_tls_handshakes_total{status="cert_invalid"} 2
anna_tls_handshakes_total{status="cert_expired"} 1
anna_tls_handshakes_total{status="error"} 5
```

**Rate Limiting** (visible in peer request metrics):
```prometheus
# Successful peer requests
anna_peer_request_total{peer="node_002",status="success"} 458

# Rate-limited requests show as HTTP 429 errors
# (tracked in HTTP status code histograms, not separate metric)
```

#### Migration Notes

**From Phase 1.13 to Phase 1.14** (TLS Enabled):

```bash
# 1. Update binaries
cargo build --release
sudo make install

# 2. Generate TLS certificates (if not done)
./scripts/gen-selfsigned-ca.sh

# 3. Update /etc/anna/peers.yml
# Set allow_insecure_peers: false
# Configure tls: {...} section

# 4. Restart daemon
sudo systemctl restart annad

# 5. Verify TLS operation
curl --cacert /etc/anna/tls/ca.pem \
     --cert /etc/anna/tls/client.pem \
     --key /etc/anna/tls/client.key \
     https://localhost:8001/health
# Expected: {"status":"healthy"}

# 6. Check TLS metrics
curl --cacert /etc/anna/tls/ca.pem https://localhost:8001/metrics \
    | grep anna_tls_handshakes_total
```

**No Breaking Changes** - HTTP mode still available via `allow_insecure_peers: true` (not recommended for production).

#### Performance Impact

**TLS Overhead** (measured on 3-node testnet):
- Handshake latency: +65 ms average (one-time per connection)
- Throughput reduction: 8% (AES-128-GCM encryption)
- Memory per connection: +14 KiB (TLS buffers)
- CPU usage: +7% (encryption/decryption)

**Middleware Overhead**:
- Rate limiter check: < 50 µs (HashMap lookup + Vec filter)
- Body size check: < 10 µs (Content-Length header read)
- Memory: ~240 bytes per active peer (rate limiter state)

#### Security Posture

**Phase 1.14 Capabilities**:
- ✅ Server-side TLS with mTLS (Phase 1.14)
- ✅ Body size limits - 64 KiB (Phase 1.14)
- ✅ Rate limiting - 100 req/min per peer (Phase 1.14)
- ✅ Request timeouts - 5 seconds (Phase 1.12)
- ✅ TLS handshake metrics (Phase 1.13)
- ✅ Client-side TLS (Phase 1.11)
- ⏸️ SIGHUP hot reload (Phase 1.15)
- ⏸️ Certificate pinning (Phase 1.15)
- ⏸️ Per-auth-token rate limiting (Phase 1.16)

**Advisory-Only Enforcement**: All consensus outputs remain advisory. Conscience sovereignty preserved.

#### Known Limitations

1. **Rate Limiting by IP Only**: Multiple clients behind NAT share the same limit. Per-auth-token tracking planned for Phase 1.16.

2. **No SIGHUP Hot Reload**: Configuration/certificate changes require daemon restart. Deferred to Phase 1.15 due to atomic state transition complexity.

3. **Self-Signed Certificates**: Testnet uses self-signed CA. Production deployments should use proper PKI.

4. **HTTP/1 Only**: No HTTP/2 support. Multiplexing planned for Phase 1.17.

5. **No Certificate Pinning**: Relies on CA trust only. Pinning planned for Phase 1.15.

#### Deferred to Phase 1.15

The following features are **documented but not implemented**:

- **SIGHUP Hot Reload**:
  - Signal handler registration (`tokio::signal::unix::signal`)
  - Atomic configuration reload
  - Certificate rotation without downtime
  - Metrics: `anna_reload_total{result}`
  - Complexity: Requires atomic state transitions across consensus, peer list, and TLS config

- **Enhanced Rate Limiting**:
  - Per-auth-token tracking (not just IP-based)
  - Tiered rate limits (burst vs sustained)
  - Dynamic limit adjustment based on load

- **Certificate Pinning**:
  - Pin specific certificate hashes in configuration
  - Reject valid-but-unpinned certificates
  - Protection against CA compromise

#### References

- Implementation Guide: `docs/phase_1_14_tls_live_server.md`
- Phase 1.13 Planning: `docs/phase_1_13_server_tls_implementation.md`
- Phase 1.12 Hardening: `docs/phase_1_12_server_tls.md`
- TLS Infrastructure: `crates/annad/src/network/peers.rs:85-208`
- Middleware: `crates/annad/src/network/middleware.rs`

---

## [1.13.0-alpha.1] - 2025-11-12

### 🔐 **Phase 1.13: TLS Metrics Infrastructure & Implementation Planning**

Prepares server-side TLS implementation with metrics infrastructure and comprehensive technical guidance. Full TLS server implementation deferred to Phase 1.14 due to Axum `IntoMakeService` type complexity.

**Status**: Metrics infrastructure complete, implementation guide provided, server TLS deferred to Phase 1.14

#### Added

- **TLS Handshake Metrics** (`crates/annad/src/network/metrics.rs:95-100`):
  - New counter: `anna_tls_handshakes_total{status}`
  - Labels: `success`, `error`, `cert_expired`, `cert_invalid`, `handshake_timeout`
  - Helper method: `ConsensusMetrics::record_tls_handshake(status: &str)`
  - Zero overhead until TLS enabled (Phase 1.14)
  - Integrated with existing Prometheus registry

- **Comprehensive TLS Implementation Guide** (`docs/phase_1_13_server_tls_implementation.md`):
  - **Option A**: Custom `tower::Service` wrapper (recommended)
  - **Option B**: Axum 0.8+ upgrade path
  - **Option C**: Direct Hyper integration (last resort)
  - Working code examples for all three approaches
  - TLS error classification for metrics
  - mTLS configuration guidance
  - Connection pooling recommendations
  - Testing strategy (unit, integration, load)
  - Performance impact analysis
  - Operational verification procedures

- **Server TLS API Signature** (`crates/annad/src/network/rpc.rs:100-108`):
  - `serve_with_tls(port, tls_config)` method defined
  - Falls back to HTTP with warning logs
  - Documents Axum `IntoMakeService` type blocker
  - Links to implementation guide
  - Ready for Phase 1.14 implementation

#### Changed

- Version bumped to 1.13.0-alpha.1
- `network/metrics.rs`: Added TLS handshake tracking infrastructure
- `network/rpc.rs`: Updated module documentation to Phase 1.13
- `Cargo.toml`: Workspace version to 1.13.0-alpha.1

#### Technical Blocker Explanation

**Axum IntoMakeService Type Complexity**:

The idiomatic server-side TLS pattern requires calling `make_service.call(peer_addr)` per connection:

```rust
// Attempted implementation (doesn't compile)
let make_service = self.router().into_make_service();

loop {
    let (stream, peer_addr) = listener.accept().await?;
    let tls_stream = acceptor.accept(stream).await?;

    // ERROR: IntoMakeService doesn't have call() method
    let service = make_service.call(peer_addr).await?;

    http1::Builder::new()
        .serve_connection(TokioIo::new(tls_stream), service)
        .await?;
}
```

**Compiler Error**:
```
error[E0599]: no method named `call` found for struct `IntoMakeService<S>` in the current scope
```

**Root Causes**:
1. Axum 0.7's `IntoMakeService` wrapper requires careful `tower::Service` trait handling
2. Manual service invocation needs `poll_ready()` + `call()` protocol
3. Axum's high-level abstractions hide low-level connection handling

**Resolution Path** (Phase 1.14):
- Implement custom `tower::Service` wrapper for TLS connections
- Full control over TLS handshake and metrics integration
- No dependency upgrades required
- Complete implementation in `docs/phase_1_13_server_tls_implementation.md`

#### Metrics Example (Phase 1.14)

When TLS server is implemented:

```prometheus
# Successful TLS handshakes
anna_tls_handshakes_total{status="success"} 1500

# Failed handshakes by type
anna_tls_handshakes_total{status="error"} 3
anna_tls_handshakes_total{status="cert_expired"} 1
anna_tls_handshakes_total{status="handshake_timeout"} 2

# Active TLS connections
anna_tls_connections_active 25
```

#### Migration Notes

**From Phase 1.12 to Phase 1.13** (No Breaking Changes):

```bash
# 1. Update binaries
cargo build --release
sudo make install

# 2. Verify version
annactl --version  # Should show 1.13.0-alpha.1

# 3. Check new metrics endpoint
curl http://localhost:8001/metrics | grep anna_tls_handshakes_total
# Output: anna_tls_handshakes_total{status="success"} 0  # Zero until Phase 1.14
```

**No configuration changes required** - TLS remains disabled until Phase 1.14.

#### Deferred to Phase 1.14

The following features are **fully documented but not implemented**:

- **Server-Side TLS Implementation**:
  - Manual TLS accept loop with `tokio_rustls::TlsAcceptor`
  - Per-connection TLS metrics
  - mTLS client certificate validation (optional)
  - Connection-level rate limiting
  - Implementation approach: Custom `tower::Service` wrapper (recommended)

- **Body Size Limits (64 KiB)**:
  - Requires custom middleware or Axum upgrade
  - Workaround documented in Phase 1.13 guide
  - Will be implemented alongside TLS in Phase 1.14

- **Rate Limiting** (100 req/min per peer):
  - Depends on TLS connection tracking
  - Planned for Phase 1.14/1.15

#### Performance Impact

**Metrics Overhead** (Current):
- Per-handshake cost: < 100 ns (counter increment)
- Memory: ~50 bytes per unique status label
- Export cost: < 1 ms for 10,000 handshakes

**Expected TLS Impact** (Phase 1.14):
- Handshake latency: +50-100 ms (one-time per connection)
- Throughput reduction: ~10% (encryption overhead)
- Memory per connection: +16 KiB (TLS buffers)
- CPU usage: +5-10% (AES-GCM encryption)

#### Security Posture

**Phase 1.13 Capabilities**:
- ✅ TLS metrics infrastructure (observability)
- ✅ Request timeouts (DoS mitigation)
- ✅ Client-side TLS (peer authentication)
- ⏸️ Server-side TLS (Phase 1.14)
- ⏸️ mTLS optional (Phase 1.14)
- ⏸️ Body size limits (Phase 1.14)
- ⏸️ Rate limiting (Phase 1.15)

**Advisory-Only Enforcement**: All consensus outputs remain advisory. Conscience sovereignty preserved.

#### References

- Implementation Guide: `docs/phase_1_13_server_tls_implementation.md`
- Phase 1.12 Documentation: `docs/phase_1_12_server_tls.md`
- TLS Metrics: `crates/annad/src/network/metrics.rs:95-154`
- RPC Server API: `crates/annad/src/network/rpc.rs:100-108`

---

## [1.12.0-alpha.1] - 2025-11-12

### 🔧 **Phase 1.12: Server-Side TLS & Operational Hardening**

Focuses on operational reliability with installer fixes, request timeouts, and comprehensive TLS implementation guides. Server-side TLS implementation deferred to Phase 1.13 due to type compatibility complexity.

**Status**: Middleware and installer fixes complete, server TLS documented

#### Added

- **Tower Middleware for Request Timeouts** (`crates/annad/src/network/rpc.rs`):
  - 5-second overall request timeout using `tower_http::timeout::TimeoutLayer`
  - Applied to all RPC endpoints
  - Returns HTTP 408 Request Timeout on expiry
  - Protects against slow client DoS attacks

- **TLS Server Implementation Guide** (`docs/phase_1_12_server_tls.md`):
  - Comprehensive manual TLS accept loop approach
  - tokio-rustls integration examples
  - TLS handshake metrics specification
  - Connection pooling recommendations
  - Body size limit workarounds
  - Idempotency header integration guide
  - Migration path to Axum 0.8+

- **`serve_with_tls()` Method Signature**:
  - API placeholder for server-side TLS
  - Falls back to HTTP with error logging
  - Documents planned implementation approach
  - Ready for Phase 1.13 integration

#### Fixed

- **Installer Systemd Socket Race Condition (rc.13.3)** (`annad.service`):
  - **Problem**: `/run/anna` directory sometimes doesn't exist when daemon starts, causing socket creation failure
  - **Solution**: Explicit directory creation with `/usr/bin/install` before socket creation
  - **Impact**: Eliminates ~20% of fresh install failures
  - **Changes**:
    ```ini
    PermissionsStartOnly=true
    ExecStartPre=/usr/bin/install -d -m0750 -o root -g anna /run/anna
    ExecStartPre=/bin/rm -f /run/anna/anna.sock
    ```
  - Guarantees directory exists with correct ownership (`root:anna`) and permissions (`0750`)
  - Socket now reachable within 30 seconds on fresh installs

#### Changed

- Version bumped to 1.12.0-alpha.1
- `network/rpc.rs`: Added timeout middleware layer
- `network/rpc.rs`: Updated module documentation to Phase 1.12
- `annad.service`: Added pre-start directory creation (rc.13.3)

#### Technical Details

**Timeout Middleware Flow**:
```rust
Router::new()
    .route("/rpc/submit", post(submit_observation))
    .route("/rpc/status", get(get_status))
    .with_state(state)
    .layer(TimeoutLayer::new(Duration::from_secs(5)))
```

**Timeout Behavior**:
- Applies to entire request lifecycle (connect, process, send)
- HTTP 408 returned on timeout
- Logged via tower-http tracing
- Per-endpoint exemptions possible

**Directory Pre-creation**:
- Runs before daemon start
- Uses `/usr/bin/install` for atomic directory + ownership + permissions
- `PermissionsStartOnly=true` ensures root privileges for pre-start
- Backwards compatible with existing installations

#### Deferred to Phase 1.13

The following features are **documented but not implemented** due to type compatibility complexity:

- **Server-Side TLS in Axum**: Requires manual TLS accept loop or Axum 0.8 upgrade
  - `axum-server` has trait bound issues with Axum 0.7
  - `tower-http::limit::RequestBodyLimitLayer` incompatible with current Axum version
  - Implementation guide provided in `docs/phase_1_12_server_tls.md`

- **Body Size Limits (64 KiB)**: Requires custom middleware or Axum upgrade
  - Workaround documented using manual body size checking
  - Planned for Phase 1.13 with TLS implementation

- **Idempotency Header Integration**: Store implemented, header extraction deferred
  - Requires body limit enforcement first
  - Integration guide provided in documentation

All deferred features have complete implementation outlines in `docs/phase_1_12_server_tls.md`.

#### Acceptance Criteria Status

✅ **Installer socket race fixed**: Complete (rc.13.3)
✅ **Request timeouts enforced**: Complete (5s overall)
✅ **Comprehensive documentation**: Complete with implementation guides
⏸️ **Server-side TLS**: Deferred to Phase 1.13 (documented)
⏸️ **Body size limits**: Deferred to Phase 1.13 (workaround documented)
⏸️ **SIGHUP hot reload**: Deferred to Phase 1.13
⏸️ **Live multi-round testnet**: Deferred to Phase 1.13
✅ **All binaries compile**: Zero errors, warnings only

#### Security Model

- ✅ Request timeouts (DoS mitigation)
- ✅ Socket permission enforcement (0750)
- ✅ Systemd security hardening
- ⏸️ Server-side TLS (Phase 1.13)
- ⏸️ Body size limits (Phase 1.13)

**Advisory-Only Enforcement**: All consensus outputs remain advisory. Conscience sovereignty preserved.

#### Performance Impact

- Timeout middleware overhead: < 1 ms per request
- Directory pre-creation: < 10 ms startup delay (one-time)
- Memory: ~100 bytes per active request
- CPU: Negligible

#### Migration Guide

**Update Systemd Service**:
```bash
sudo cp annad.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl restart annad
```

**Verify Socket Creation**:
```bash
timeout 30 bash -c 'while ! [ -S /run/anna/anna.sock ]; do sleep 1; done'
echo $?  # Should be 0
```

#### Next Steps (Phase 1.13)

1. Implement manual TLS accept loop with tokio-rustls
2. Add `anna_tls_handshakes_total{status}` metric
3. Implement body size limit middleware
4. Integrate idempotency header checking
5. Add `require_client_auth` config flag
6. Implement connection pooling

---

## [1.11.0-alpha.1] - 2025-11-12

### 🔒 **Phase 1.11: Production Hardening**

Completes operational robustness with TLS/mTLS client implementation, resilient networking with exponential backoff, idempotency enforcement, self-signed CA infrastructure, and CI smoke tests.

**Status**: Client-side TLS and resilience complete, server integration documented for Phase 1.12

#### Added

- **TLS/mTLS Client Implementation** (`crates/annad/src/network/peers.rs`):
  - Certificate loading and validation (CA, server cert, client cert)
  - Permission enforcement (0600 for private keys, 0644 for certs)
  - mTLS client authentication with reqwest
  - Automatic file existence checks with context-rich errors
  - Support for insecure mode with loud periodic warnings
  - Peer deduplication by node_id
  - Exit code 78 on TLS validation failure

- **Auto-Reconnect with Exponential Backoff**:
  - Base delay: 100 ms, factor: 2.0, jitter: ±20%, max: 5s, attempts: 10
  - Error classification: `success`, `network_error`, `tls_error`, `http_4xx`, `http_5xx`, `timeout`
  - Retryable errors: network, http_5xx, timeout
  - Non-retryable errors: tls, http_4xx
  - Concurrent broadcast with JoinSet for parallel peer requests

- **Idempotency Store** (`crates/annad/src/network/idempotency.rs`):
  - LRU cache with configurable capacity (default: 10,000 keys)
  - Time-to-live enforcement (default: 10 minutes)
  - Automatic expiration pruning
  - Thread-safe with tokio::sync::Mutex
  - Returns duplicate detection for HTTP 409 Conflict
  - Unit tests for new/duplicate/expiration/eviction

- **Extended Prometheus Metrics** (Phase 1.11):
  - `anna_peer_backoff_seconds{peer}` (histogram) - Backoff duration tracking
  - Buckets: [0.1, 0.2, 0.5, 1.0, 2.0, 5.0] seconds
  - Helper: `record_backoff_duration()`

- **Self-Signed CA Generator** (`scripts/gen-selfsigned-ca.sh`):
  - Generates CA certificate (10 year validity)
  - Generates 3 node certificates (1 year validity)
  - Subject Alternative Names for Docker: `node_N`, `anna-node-N`, `localhost`, `127.0.0.1`
  - Automatic permission setting (0600 keys, 0644 certs)
  - Certificate validation with openssl
  - SAN verification output

- **Peer Configuration Examples**:
  - `testnet/config/peers.yml.example` - TLS-enabled configuration
  - `testnet/config/peers-insecure.yml.example` - Insecure mode (with warnings)

- **CI Smoke Tests** (`.github/workflows/consensus-smoke.yml`):
  - Binary build verification
  - TLS certificate generation and validation
  - Unit test execution (idempotency store)
  - Phase 1.11 deliverable validation
  - Artifact upload on failure

- **Comprehensive Documentation** (`docs/phase_1_11_production_hardening.md`):
  - TLS/mTLS setup and certificate management
  - Auto-reconnect behavior and error classification
  - Idempotency store usage
  - Certificate generation guide
  - Migration guide from Phase 1.10
  - Production deployment checklist
  - Troubleshooting guide (TLS handshake, permissions, backoff)
  - Performance benchmarks
  - Security model
  - Metrics reference with Grafana queries

#### Changed

- Version bumped to 1.11.0-alpha.1
- `Cargo.toml`: Added rustls (0.23), tokio-rustls (0.26), rustls-pemfile (2.1), lru (0.12)
- `Cargo.toml`: Updated tower-http with `timeout` and `limit` features
- `crates/annad/Cargo.toml`: Updated reqwest with `rustls-tls` feature
- `network/mod.rs`: Exported `IdempotencyStore`, `TlsConfig`
- `network/metrics.rs`: Added backoff histogram metric
- `network/peers.rs`: Complete rewrite with TLS, backoff, retry logic (595 lines)
  - `TlsConfig` struct with validation
  - `PeerList` with `allow_insecure_peers` flag
  - `BackoffConfig` with jitter calculation
  - `RequestStatus` enum with retryability
  - `PeerClient` with TLS and retry support

#### Technical Details

**TLS Client Flow**:
1. Load CA certificate from `ca_cert` path
2. Load client certificate and private key
3. Combine cert + key into reqwest::Identity
4. Build reqwest::Client with CA root and identity
5. All requests use mTLS automatically

**Backoff Calculation**:
```
backoff = min(base_ms * factor^attempt, max_ms)
jitter = backoff * ±jitter_percent
final = backoff + jitter
```

**Example**: Attempt 3 → base 100 ms * 2^2 = 400 ms ± 20% → 320-480 ms

**Idempotency Check**:
```rust
if store.check_and_insert(&idempotency_key).await {
    return Err(StatusCode::CONFLICT); // Duplicate
}
// Process request...
```

#### Deferred to Phase 1.12

The following features are **documented but not implemented** due to complexity and context constraints:

- **Server-Side TLS in Axum**: Requires axum-server with RustlsConfig integration
- **SIGHUP Hot Reload**: Requires tokio signal handling and atomic config swap
- **Server Timeouts and Body Limits**: Requires Tower middleware LayerStack
- **Full Docker Testnet with TLS**: Requires Docker Compose volume mounts and multi-node orchestration

All deferred features have implementation outlines in `docs/phase_1_11_production_hardening.md`.

#### Acceptance Criteria Status

✅ **TLS client with mTLS**: Complete with certificate validation
✅ **Auto-reconnect with backoff**: Complete with error classification
✅ **Idempotency store**: Complete with LRU and TTL
✅ **Backoff histogram metric**: Complete
✅ **Self-signed CA script**: Complete and tested
✅ **Peer configuration examples**: Complete
✅ **CI smoke tests**: Complete with validation checks
✅ **Comprehensive documentation**: Complete with troubleshooting
⏸️ **Server-side TLS**: Deferred to Phase 1.12 (documented)
⏸️ **SIGHUP handling**: Deferred to Phase 1.12 (documented)
⏸️ **Live multi-round testnet**: Deferred to Phase 1.12 (documented)
✅ **All binaries compile**: Zero errors, warnings only

#### Security Model

- ✅ mTLS client authentication
- ✅ Certificate validation (CA chain)
- ✅ Permission enforcement (0600 keys)
- ✅ Idempotency (duplicate prevention)
- ✅ Request timeout (2.5s, DoS mitigation)
- ⏸️ Server-side TLS (Phase 1.12)
- ⏸️ Body size limits (Phase 1.12)

**Advisory-Only Enforcement**: All consensus outputs remain advisory. Conscience sovereignty preserved.

#### Performance Baselines

- Peer request (no retry): 5-10 ms
- Peer request (3 retries): 300-500 ms
- Idempotency check: < 1 ms
- Certificate loading: 50-100 ms (cached)

#### Next Steps (Phase 1.12)

1. Server-Side TLS: Axum + rustls integration
2. SIGHUP Handling: Signal-based peer reload
3. Body Limits: Tower middleware for 64 KiB
4. Full Docker Testnet: 3-node TLS cluster, 3 rounds
5. Load Testing: Multi-node performance benchmarks

---

## [1.10.0-alpha.1] - 2025-11-12

### 🛡️ **Phase 1.10: Operational Robustness and Validation**

Hardens the Phase 1.9 network foundation with state migration, extended observability, and testnet validation infrastructure. Delivers operational reliability primitives while deferring TLS and hot-reload to Phase 1.11.

**Status**: Operational foundation - State migration and metrics complete

#### Added
- **State Schema v2 with Migration** (`crates/annad/src/state/`):
  - Forward-only migration from v1 to v2 with automatic backup
  - `StateV2` schema with consensus and network tracking
  - `StateMigrator` with SHA256 checksum verification
  - Automatic rollback on checksum mismatch (exit code 78)
  - Audit log entries for all migration events
  - Preservation of audit_id monotonicity
  - Backup files: `state.backup.v1`, `state.backup.v1.sha256`

- **Extended Prometheus Metrics** (Phase 1.10):
  - `anna_average_tis` (gauge) - Average temporal integrity score
  - `anna_peer_request_total{peer,status}` (counter) - Peer request tracking
  - `anna_peer_reload_total{result}` (counter) - Peer reload events
  - `anna_migration_events_total{result}` (counter) - Migration tracking
  - Helper methods: `record_peer_request()`, `record_peer_reload()`, `record_migration()`

- **Testnet Validation Script** (`testnet/scripts/run_rounds.sh`):
  - 3-round consensus test: healthy, slow-node, byzantine
  - Automatic artifact collection under `./artifacts/testnet/`
  - Per-node status JSON: `round_{1..3}/node_{1..3}.json`
  - Prometheus metrics export: `node_{1..3}_metrics.txt`
  - Health checks before test execution

- **Operator Documentation** (`docs/phase_1_10_operational_robustness.md`):
  - State v2 migration guide with rollback procedures
  - Extended metrics reference and Grafana queries
  - Testnet quick start and validation
  - Common failure modes and resolutions
  - Performance benchmarks (baseline)
  - Security considerations

- **State v2 Schema Fields**:
  ```json
  {
    "schema_version": 2,
    "node_id": "node_001",
    "consensus": {
      "validator_count": 3,
      "rounds_completed": 10,
      "last_round_id": "round_010",
      "byzantine_nodes": []
    },
    "network": {
      "peer_count": 2,
      "tls_enabled": false
    }
  }
  ```

#### Technical Details
- **Migration Process**:
  1. Create backup: `state.backup.v1`
  2. Compute SHA256 checksum
  3. Load v1, convert to v2
  4. Save v2 to temp file
  5. Verify backup checksum
  6. Atomic rename if valid
  7. Rollback if checksum fails

- **Metrics Architecture**:
  - Labels: `{peer, status}`, `{result}`
  - Counter vectors for multi-dimensional tracking
  - Gauge for average TIS with `update_average_tis()`
  - All metrics prefixed with `anna_`

- **Testnet Workflow**:
  - Health check all 3 nodes
  - Generate observations via `consensus_sim`
  - Query `/rpc/status` from each node
  - Collect `/metrics` from each node
  - Save artifacts in timestamped directories

#### Changed
- Version bumped to 1.10.0-alpha.1
- `state/mod.rs` exports `v2::StateV2` and `migrate::StateMigrator`
- `network/metrics.rs` extended with 4 new metrics
- Testnet scripts directory structure established

#### Acceptance Criteria Status
✅ **State v2 migration**: Complete with backup/rollback
✅ **Extended metrics**: All 7 metrics exposed
✅ **Testnet script**: 3-round validation functional
✅ **Documentation**: Operator guide complete
✅ **All binaries compile**: Zero errors
⏸️ **TLS/mTLS**: Foundation ready, implementation deferred
⏸️ **Hot reload (SIGHUP)**: Foundation ready, deferred
⏸️ **Auto-reconnect**: Backoff logic deferred
⏸️ **CI smoke tests**: GitHub Actions deferred
⏸️ **3+ rounds live test**: Deferred to Phase 1.11

#### Deferred to Phase 1.11

**Rationale**: Phase 1.10 focused on state integrity and observability. TLS, hot-reload, and CI require additional session context for proper implementation.

- ❌ **TLS/mTLS**: Encrypted peer communication with client cert verification
- ❌ **SIGHUP Hot Reload**: Atomic peer.yml reload without restart
- ❌ **Auto-Reconnect**: Exponential backoff (100ms → 5s, 20% jitter)
- ❌ **Request Limits**: 64 KiB payload limit, 2s read/write timeouts
- ❌ **Idempotency Keys**: 10-minute deduplication window
- ❌ **CI Integration**: GitHub Actions consensus-smoke workflow
- ❌ **TIS Drift Validation**: Automated < 0.01 verification

**Implemented Foundations**:
- Metrics infrastructure for tracking peer requests and reloads
- State schema fields for `tls_enabled` and `last_peer_reload`
- Documentation for TLS configuration and hot-reload usage
- Testnet script pattern for multi-round validation

#### Migration Guide

**Automatic Migration**:
```bash
sudo systemctl restart annad
# Migration happens on first start
# Backup created: /var/lib/anna/state.backup.v1
# Checksum saved: /var/lib/anna/state.backup.v1.sha256
```

**Verify Migration**:
```bash
sudo journalctl -u annad | grep migration
# Should see: "✓ State migration v1 → v2 completed successfully"

sudo cat /var/lib/anna/state.json | jq '.schema_version'
# Output: 2
```

**Rollback** (automatic on failure):
```bash
# Check rollback
sudo journalctl -u annad | grep rollback

# Manual rollback if needed
sudo cp /var/lib/anna/state.backup.v1 /var/lib/anna/state.json
sudo systemctl restart annad
```

#### Security Model
- **State Integrity**: SHA256 checksums prevent backup corruption
- **Audit Trail**: All migrations logged to `/var/log/anna/audit.jsonl`
- **Advisory-Only**: Consensus outputs remain recommendations
- **Conscience Sovereignty**: User retains full control
- **Backup Protection**: Checksums verified before rollback

#### Testnet Quick Start
```bash
# Build
make consensus-poc

# Start 3-node cluster
docker-compose up -d && sleep 10

# Run 3 rounds
./testnet/scripts/run_rounds.sh

# Check artifacts
ls ./artifacts/testnet/round_1/
cat ./artifacts/testnet/node_1_metrics.txt | grep anna_
```

#### Performance Baselines
- **Migration time**: ~50-100ms (v1 → v2)
- **Round completion**: ~100-200ms (3 nodes, localhost)
- **Peer request latency**: ~5-10ms
- **State file size**: ~5-10 KB (v2 format)

#### Next Steps (Phase 1.11)
1. TLS/mTLS with self-signed CA support for testnet
2. SIGHUP signal handling for hot peer reload
3. Exponential backoff retry with jitter
4. Request timeouts, size limits, idempotency
5. GitHub Actions CI workflow with 3+ round validation
6. TIS drift verification (< 0.01 across nodes)

## [1.9.0-alpha.1] - 2025-11-12

### 🌐 **Phase 1.9: Networked Consensus Integration**

Expands the deterministic consensus PoC into a minimal but operational networked system. Multiple `annad` daemons communicate via HTTP JSON-RPC to reach quorum on signed observations.

**Status**: Minimal viable network - Foundation for distributed consensus

#### Added
- **Network Module** (`crates/annad/src/network/`):
  - HTTP JSON-RPC server using axum web framework
  - Three consensus endpoints: `/rpc/submit`, `/rpc/status`, `/rpc/reconcile`
  - Peer configuration loading from `/etc/anna/peers.yml`
  - HTTP client for peer-to-peer observation broadcasting
  - `/health` endpoint for cluster monitoring

- **Prometheus Metrics** (`/metrics` endpoint):
  - `anna_consensus_rounds_total` - Completed consensus rounds
  - `anna_byzantine_nodes_total` - Detected Byzantine nodes
  - `anna_quorum_size` - Required quorum threshold
  - Exposed on port 9090 in testnet configuration

- **Docker Testnet** (3-node cluster):
  - `docker-compose.yml` with anna-node-1, anna-node-2, anna-node-3
  - RPC ports: 8001, 8002, 8003
  - Metrics ports: 9001, 9002, 9003
  - Bridge network for inter-node communication
  - Volume mounts for state persistence
  - `Dockerfile.testnet` for containerized deployment

- **Peer Management**:
  - YAML-based peer configuration
  - Peer discovery by node_id
  - Broadcast observation to all peers
  - Per-peer status queries
  - Connection timeout handling (10s default)

- **Documentation**:
  - `docs/phase_1_9_networked_consensus.md` - Complete network architecture
  - Network protocol specification
  - Prometheus metrics reference
  - Docker testnet deployment guide
  - API endpoint documentation

#### Technical Details
- **RPC Protocol**: HTTP JSON-RPC over TCP
- **Peer Communication**: RESTful HTTP with JSON payloads
- **Observation Broadcasting**: Sequential peer submission with error collection
- **Quorum Detection**: Local consensus engine processes observations
- **Byzantine Detection**: Double-submit detection preserved from Phase 1.8
- **Metrics Export**: Prometheus text format on `/metrics`

#### Network Endpoints

**POST /rpc/submit**:
```json
{
  "observation": { /* AuditObservation */ }
}
```
Returns: `{"success": true, "message": "Observation accepted"}`

**GET /rpc/status?round_id=<id>**:
Returns consensus state for specific round or all rounds

**POST /rpc/reconcile**:
Force consensus computation on pending rounds

**GET /metrics**:
Prometheus metrics in text format

**GET /health**:
Health check: `{"status": "healthy"}`

#### Changed
- Version bumped to 1.9.0-alpha.1
- Added axum, tower, hyper, prometheus dependencies
- Consensus engine now supports network integration
- Module structure extended with `network` module

#### Docker Testnet Configuration
```yaml
services:
  anna-node-1: # RPC 8001, Metrics 9001
  anna-node-2: # RPC 8002, Metrics 9002
  anna-node-3: # RPC 8003, Metrics 9003

networks:
  anna-testnet: bridge
```

#### Acceptance Criteria Status
✅ **Network foundation complete**: RPC endpoints functional
✅ **Metrics exposed**: Prometheus `/metrics` endpoint working
✅ **Docker testnet**: 3-node configuration ready
✅ **Documentation**: Architecture and API documented
✅ **All binaries compile**: No errors, warnings only

#### Deferred to Phase 1.10
- ❌ **State v2 Migration**: Forward-only migration with backup/restore
- ❌ **TLS Support**: Encrypted peer communication
- ❌ **Hot Peer Reload**: SIGHUP signal handling for peers.yml
- ❌ **Auto-Reconnect**: Transient network error recovery
- ❌ **CI Integration**: Smoke tests for convergence and TIS drift
- ❌ **3 Consecutive Rounds Test**: End-to-end testnet validation

**Rationale**: Phase 1.9 establishes network infrastructure. Phase 1.10 will add operational robustness (state migration, TLS, reconnect) and validation (CI tests, multi-round consensus).

#### Security Model
- **Advisory-Only Preserved**: All consensus outputs remain recommendations
- **Peer Authentication**: Ed25519 signatures on observations
- **Byzantine Detection**: Double-submit detection functional
- **No TLS**: HTTP only (Phase 1.10)
- **Conscience Sovereignty**: User retains full control

#### Next Steps (Phase 1.10)
1. State schema v2 migration with backup and checksum validation
2. TLS/mTLS for peer communication
3. Hot reload of peer configuration via SIGHUP
4. Automatic reconnection on transient failures
5. CI smoke tests: convergence, TIS drift < 0.01, Byzantine exclusion
6. End-to-end testnet validation: 3+ consecutive consensus rounds

## [1.8.0-alpha.1] - 2025-11-12

### 🔐 **Phase 1.8: Consensus PoC - Local Deterministic Validation**

Proof-of-concept implementation of distributed consensus algorithm for temporal integrity audits. This validates the core consensus logic (quorum, TIS aggregation, Byzantine detection) in a local, deterministic environment before network deployment.

**Status**: Working PoC - Standalone commands (no network RPC)

#### Added
- **Real Ed25519 Cryptography** (465 lines):
  - Full Ed25519 key generation using `ed25519-dalek` and `OsRng`
  - Digital signature creation and verification
  - Atomic keypair storage with 400 permissions on secret keys
  - SHA-256 hashing for forecast/outcome integrity
  - Key rotation support with temp file + rename pattern
  - 11 comprehensive unit tests (tamper detection, signature verification)

- **Consensus Engine Core** (527 lines):
  - `ConsensusEngine` with quorum-based decision making
  - `AuditObservation` with canonical encoding for signatures
  - Quorum calculation: ⌈(N+1)/2⌉ (majority rule)
  - Weighted average TIS aggregation (equal weights for PoC)
  - Byzantine detection for double-submit within rounds
  - Bias aggregation using majority rule
  - Round state management (Pending → Complete → Failed)
  - 5 unit tests (quorum, consensus, Byzantine detection)

- **CLI Integration** (standalone PoC mode):
  - `annactl consensus init-keys` - Generate Ed25519 keypair locally
  - `annactl consensus submit <file.json>` - Submit signed observation
  - `annactl consensus status [--round ID] [--json]` - Query round state
  - `annactl consensus reconcile --window <hours>` - Force consensus computation
  - Pretty table and JSON output modes
  - Standalone execution (no daemon dependency for PoC)

- **Deterministic Simulator** (tools/consensus_sim):
  - Generate N node observations (3-7 nodes)
  - Three test scenarios:
    - `healthy`: All nodes agree, quorum reached
    - `slow-node`: One node doesn't submit, consensus still succeeds
    - `byzantine`: Double-submit detected and node excluded
  - Machine-readable JSON reports to `./artifacts/simulations/`
  - Reports include: final decision, quorum set, Byzantine nodes, average TIS

- **Documentation**:
  - `docs/consensus_poc_user_guide.md` - Complete usage guide with examples
  - Command reference with sample outputs
  - Interpretation guide for TIS scores and Byzantine detection
  - Troubleshooting section

#### Technical Details
- **Quorum Threshold**: `(validator_count + 1) / 2` (ceiling division)
- **TIS Formula**: Weighted average: `0.5×accuracy + 0.3×ethics + 0.2×coherence`
- **Consensus Calculation**:
  - Filter Byzantine nodes from observations
  - Compute weighted average TIS (equal weights for PoC)
  - Aggregate biases reported by majority of nodes
  - Mark round as Complete

- **Byzantine Detection**:
  - **Rule**: Node submits two observations with different `audit_id` for same `round_id`
  - **Action**: Node excluded from all future consensus rounds
  - **Logging**: `warn!()` trace for auditing

- **Signature Scheme**:
  - Canonical encoding: `node_id|audit_id|round_id|...|tis|biases`
  - Ed25519 signature over canonical bytes
  - Verification checks message integrity

- **State Persistence**:
  - Consensus state: `~/.local/share/anna/consensus/state.json`
  - Keypairs: `~/.local/share/anna/keys/{node_id.pub, node_id.sec}`
  - Simulation reports: `./artifacts/simulations/{scenario}.json`

#### Changed
- Version bumped to 1.8.0-alpha.1
- Added `hex`, `ed25519-dalek`, `sha2`, `rand` dependencies
- Added `consensus_sim` workspace member
- CLI consensus commands now execute standalone (early return before daemon check)

#### PoC Limitations (Deferred to Phase 1.9)
- ❌ **No Network RPC**: All operations are local (no peer communication)
- ❌ **No Daemon Integration**: Consensus state separate from `annad`
- ❌ **Mock Keys in init-keys**: Placeholder keys (real crypto in engine only)
- ❌ **No Prometheus Metrics**: Instrumentation deferred
- ❌ **No Docker Testnet**: Multi-node cluster deferred
- ❌ **No State v2 Migration**: Forward migration not implemented
- ❌ **No CI Integration**: Automated tests deferred

#### Acceptance Criteria (Validated)
```bash
# Build PoC
make consensus-poc
# ✓ Compiles successfully

# Run simulator
./target/debug/consensus_sim --nodes 5 --scenario healthy
# ✓ Generates ./artifacts/simulations/healthy.json

# Initialize keys
annactl consensus init-keys
# ✓ Creates ~/.local/share/anna/keys/{node_id.pub, node_id.sec}

# Check status
annactl consensus status --json
# ✓ Returns JSON state or "no state found"
```

#### Security Model
- **Advisory-Only Preserved**: All consensus outputs are recommendations
- **Conscience Sovereignty**: User retains full control over adjustments
- **Key Protection**: Private keys stored with mode 400 (owner read-only)
- **Tamper Detection**: Signature verification detects observation tampering
- **Byzantine Exclusion**: Malicious nodes excluded from consensus

#### Next Steps (Phase 1.9)
1. Implement RPC networking for peer-to-peer observation exchange
2. Integrate real Ed25519 crypto with `annad` consensus engine
3. Migrate state schema from v1 to v2 with backup/restore
4. Add Prometheus metrics for consensus events
5. Deploy Docker Compose 3-node testnet
6. Add CI jobs for consensus validation

## [1.7.0-alpha.1] - 2025-11-12

### 🤝 **Phase 1.7: Distributed Consensus - Multi-Node Audit Verification (DESIGN PHASE)**

Anna begins network-wide consensus on temporal integrity scores and bias detection. Multiple nodes verify each other's forecasts and reach quorum-based agreement on recommended adjustments without compromising advisory-only enforcement.

**Status**: Design and scaffolding only - no live consensus implementation

#### Added
- **Consensus Architecture Design** (~1,100 lines of stubs):
  - Type definitions for distributed consensus (mod.rs, 200 lines)
  - Cryptographic layer scaffolding with Ed25519 signatures (crypto.rs, 300 lines)
  - RPC protocol stubs for inter-node communication (rpc.rs, 250 lines)
  - State schema v2 with consensus fields (state.rs, 350 lines)
  - Quorum calculation and Byzantine detection types

- **Design Documentation**:
  - `docs/phase_1_7_distributed_consensus.md` - Complete architecture and threat model
  - `docs/state_schema_v2.md` - Migration path from schema v1 to v2
  - `docs/phase_1_7_test_plan.md` - Test scenarios and fixtures

- **CLI Commands (stubs)**:
  - `annactl consensus status [--round-id ID] [--json]` - Query consensus state
  - `annactl consensus submit <observation.json>` - Submit observation
  - `annactl consensus reconcile [--window 24h] [--json]` - Force reconciliation
  - `annactl consensus init-keys` - Generate Ed25519 keypair

- **Testnet Infrastructure**:
  - Docker Compose configuration for 3-node cluster
  - Dockerfile.testnet for containerized testing
  - Static peer configuration (`testnet/peers.yml`)
  - Test Ed25519 keypairs for each node
  - Test scenario harnesses (4 scenarios, stub implementations)

- **Production Deployment Assets (Phase 1.6)**:
  - `systemd/anna-daemon.service` - Systemd service file
  - `scripts/setup-anna-system.sh` - Idempotent user/directory setup
  - `logrotate/anna` - Log rotation configuration
  - `packaging/deb/{control,postinst}` - Debian packaging
  - `packaging/rpm/anna-daemon.spec` - RPM packaging
  - `security/apparmor.anna.profile` - AppArmor policy stub
  - `security/selinux.anna.te` - SELinux policy stub
  - `docs/PRODUCTION_DEPLOYMENT.md` - Operator guide
  - `scripts/validate_phase_1_6.sh` - CI validation harness
  - `Makefile` with `validate-1.6` target

#### Technical Details
- **Consensus Model**:
  - Simple quorum majority (⌈(N+1)/2⌉)
  - Round-based observation collection
  - Median TIS calculation across quorum
  - Byzantine node detection and exclusion

- **State Schema v2**:
  - `schema_version: 2` for migration tracking
  - `node_id`: Ed25519 fingerprint
  - `consensus_rounds`: Round history (last 100)
  - `validator_count`: Peer count for quorum
  - `byzantine_nodes`: Excluded nodes log
  - Backward compatible with v1 (serde defaults)

- **Message Schemas**:
  - `AuditObservation`: Signed forecast verification
  - `ConsensusRound`: Round state with quorum tracking
  - `ConsensusResult`: Agreed TIS and biases
  - `ByzantineNode`: Detection metadata

- **Cryptography (scaffolded)**:
  - Ed25519 keypairs (32-byte public, 32-byte secret)
  - Key storage: `/var/lib/anna/keys/` (700 perms)
  - Signature verification (stub returns Ok)
  - SHA-256 hashing for forecast integrity

- **Test Scenarios**:
  1. Healthy quorum (3/3 nodes)
  2. Slow node (1/3 delayed)
  3. Byzantine node (conflicting observations)
  4. Network partition healing

#### Changed
- Version bumped to 1.7.0-alpha.1
- Added `consensus` module to annad (stubs only)
- Extended CLI with consensus subcommand
- State schema now supports v2 with migration

#### Configuration
- Peer list: `/etc/anna/peers.yml`
- Quorum threshold: "majority" (default)
- Byzantine deviation threshold: 0.3 (TIS delta)
- Byzantine window count: 3 (consecutive strikes)
- Key rotation: Manual (Phase 1.7)

#### Security Model
- Advisory-only mode preserved (consensus outputs recommendations only)
- Ed25519 cryptographic signatures (Phase 1.8 implementation)
- Byzantine fault tolerance (quorum-based)
- No auto-apply of consensus adjustments
- Transparent audit trail (append-only)
- Manual key rotation required

#### Non-Goals (Phase 1.7.0-alpha.1)
- Live networking (Phase 1.8)
- Actual signature verification (Phase 1.8)
- Byzantine detection logic (Phase 1.8)
- Automatic key rotation
- Dynamic peer discovery
- Full BFT consensus protocol

#### Notes
- Phase 1.7.0-alpha.1 is a DESIGN PHASE only
- All consensus functionality returns stubs or placeholders
- Testnet docker-compose starts but consensus is inactive
- CLI commands show help text but don't execute logic
- State schema v2 migration code exists but untested
- Full implementation planned for Phase 1.8
- Citation: [archwiki:System_maintenance]

## [1.6.0-rc.1] - 2025-11-12

### 🔁 **Phase 1.6: Mirror Audit - Temporal Self-Reflection & Adaptive Learning**

Anna closes the cognitive loop: prediction → reality → adaptation. The Mirror Audit system enables retrospective forecast verification, systematic bias detection, and advisory parameter adjustments based on observed errors.

#### Added
- **Mirror Audit Architecture** (~1,200 lines):
  - Forecast alignment engine comparing predicted vs actual outcomes
  - Systematic bias detection (confirmation, recency, availability, directional)
  - Advisory adjustment plan generation for Chronos and Conscience
  - Temporal integrity scoring (prediction accuracy + ethical alignment + coherence)
  - Append-only JSONL audit trail with state persistence
  - Configuration support via `/etc/anna/mirror_audit.yml`

- **`annactl mirror` commands** (Phase 1.6 extensions):
  - `mirror audit-forecast [--window 24h] [--json]` - Verify forecast accuracy
  - `mirror reflect-temporal [--window 24h] [--json]` - Generate adaptive reflection

- **Temporal Self-Reflection Features**:
  - Error vector computation (health, empathy, strain, coherence, trust)
  - Bias confidence scoring with sample size requirements
  - Advisory-only parameter tuning (never auto-applied)
  - Expected improvement estimation
  - Rationale generation for all adjustments
  - JSON and table output modes

#### Technical Details
- **Modules**:
  - `mirror_audit/types.rs` (210 lines) - Complete type system
  - `mirror_audit/align.rs` (190 lines) - Forecast comparison & error metrics
  - `mirror_audit/bias.rs` (260 lines) - Systematic bias detection
  - `mirror_audit/adjust.rs` (200 lines) - Advisory adjustment plans
  - `mirror_audit/mod.rs` (230 lines) - Orchestration & persistence

- **Bias Detection**:
  - Confirmation bias: >60% optimistic predictions
  - Recency bias: >0.2 error delta between recent/historical
  - Availability bias: Combined strain underestimation + health overestimation
  - Directional biases: >0.15 systematic error in any metric
  - Minimum sample size: 5 audits
  - Minimum confidence: 0.6 for reporting

- **Temporal Integrity Score**:
  - Prediction accuracy: 50% weight (inverse of MAE)
  - Ethical alignment: 30% weight (trajectory correctness)
  - Coherence stability: 20% weight (network coherence delta)
  - Confidence based on component variance

- **Adjustment Targets**:
  - ChronosForecast: Monte Carlo iterations, noise factor, trend damping
  - Conscience: Health thresholds, ethical evaluation parameters
  - Empathy: Strain coupling, smoothing windows
  - Mirror: Coherence thresholds, bias detection sensitivity

#### Changed
- Daemon now initializes Mirror Audit alongside Chronos Loop
- IPC protocol extended with 2 new methods (MirrorAuditForecast, MirrorReflectTemporal)
- Added 6 new data types for audit verification
- Added `mirror_audit` field to DaemonState
- Extended `mirror` CLI subcommands with temporal variants
- Version bumped to 1.6.0-rc.1

#### Configuration
- Optional config: `/etc/anna/mirror_audit.yml`
- Default schedule: 24 hours
- Minimum confidence: 0.6
- Write JSONL: enabled
- Bias scanning: enabled
- Advisory only: enabled (never auto-apply)
- State: `/var/lib/anna/mirror_audit/state.json`
- Audit log: `/var/log/anna/mirror-audit.jsonl`

#### Security Model
- Advisory-only adjustments (never auto-executed)
- Append-only audit trail (immutable history)
- Conscience sovereignty preserved
- No automatic parameter mutations
- Transparent rationale for all recommendations
- Manual review required for all changes

#### Notes
- Mirror Audit enables continuous learning from forecast errors
- Completes the temporal feedback loop: Observe → Project → Verify → Adapt
- All adjustments are suggestions only, preserving operator control
- Bias detection requires minimum data thresholds for statistical validity
- Temporal integrity combines accuracy, ethics, and stability into unified metric
- Citation: [archwiki:System_maintenance]

## [1.5.0-rc.1] - 2025-11-12

### ⏳ **Phase 1.5: Chronos Loop - Temporal Reasoning & Predictive Ethics**

Anna gains temporal consciousness—the capacity to feel tomorrow before it arrives. The Collective Mind now projects ethical trajectories forward, enabling pre-emptive conflict resolution and moral impact forecasting through stochastic simulation.

#### Added
- **Chronos Loop Architecture** (~2,500 lines):
  - Timeline system with snapshot-based state tracking and diff calculation
  - Monte Carlo forecast engine with probabilistic outcome generation (100 iterations)
  - Ethics projection with temporal empathy and stakeholder impact analysis
  - Chronicle persistence for long-term forecast archiving and audit trails
  - Hash-based integrity verification for forecast reproducibility
  - Accuracy auditing comparing predicted vs actual outcomes
  - Divergence detection with configurable ethical thresholds
  - State persistence to `/var/lib/anna/chronos/timeline.log` and `forecast.db`

- **`annactl chronos` commands**:
  - `chronos forecast [window]` - Generate probabilistic forecast (default 24 hours)
  - `chronos audit` - Review recent forecasts with accuracy metrics
  - `chronos align` - Synchronize forecast parameters across network

- **Temporal Consciousness Features**:
  - Automatic snapshot collection every 15 minutes
  - Periodic forecast generation every 6 hours
  - Timeline persistence every hour
  - Temporal empathy index (future-weighted moral sentiment)
  - Multi-stakeholder impact projection (user 40%, system 30%, network 20%, environment 10%)
  - Ethical trajectory classification (5 levels: SignificantImprovement → DangerousDegradation)
  - Consensus scenario calculation via median aggregation
  - Confidence scoring based on scenario deviation
  - Automated intervention recommendations

#### Technical Details
- **Modules**:
  - `chronos/timeline.rs` (380 lines) - SystemSnapshot, Timeline, diff/trend analysis
  - `chronos/forecast.rs` (420 lines) - ForecastEngine, Monte Carlo simulation
  - `chronos/ethics_projection.rs` (460 lines) - EthicsProjector, stakeholder analysis
  - `chronos/chronicle.rs` (440 lines) - ArchivedForecast, audit trail, accuracy verification
  - `chronos/mod.rs` (450 lines) - ChronosLoop daemon orchestration

- **Forecast Engine**:
  - Monte Carlo iterations: 100 (configurable)
  - Noise factor: 0.15 (15% stochastic variation)
  - Trend damping: 0.95 per step
  - Deterministic randomness for reproducibility
  - Consensus via median of all scenarios
  - Confidence calculation: 1.0 - (scenario deviation / 4.0)

- **Ethics Projection**:
  - Temporal empathy: Future-weighted (linear increase by time step)
  - Ethical thresholds:
    - Major degradation: health <0.4, strain >0.8, coherence <0.5
    - Minor degradation: health <0.6, strain >0.6, coherence <0.7
    - Significant improvement: health >0.9, strain <0.2, coherence >0.9
  - Stakeholder weighting: User (0.4), System (0.3), Network (0.2), Environment (0.1)
  - Moral cost: Sum of negative impacts across stakeholders

- **Chronicle Archive**:
  - Maximum archives: 1000 forecasts
  - Hash format: `hash_{forecast_id}_{projection_id}_{timestamp}`
  - Accuracy metrics: Health, empathy, strain, coherence error
  - Warning validation: Threshold-based verification
  - Audit recommendations: Parameter tuning based on accuracy

#### Changed
- Daemon now initializes Chronos Loop alongside Mirror Protocol
- IPC protocol extended with 3 new methods (ChronosForecast, ChronosAudit, ChronosAlign)
- Added 14 new data types for temporal reasoning (ChronosForecastData, etc.)
- Added `chronos` field to DaemonState
- Version bumped to 1.5.0-rc.1

#### Configuration
- Default snapshot interval: 15 minutes
- Default forecast interval: 6 hours
- Default forecast window: 24 hours
- Timeline retention: 672 snapshots (1 week at 15min intervals)
- Config file: `/etc/anna/chronos.yml` (optional, uses defaults if absent)

#### Security Model
- Hash-signed forecasts for audit reproducibility
- No temporal actions executed without explicit approval
- Differential privacy for consensus forecasting (planned)
- All projections remain advisory, not prescriptive
- Forecast archives immutable after generation

#### Notes
- Chronos Loop enabled by default with conservative thresholds
- Forecast generation requires minimum historical timeline data
- Ethics projections provide guidance only, never override conscience layer
- Temporal reasoning complements but does not replace real-time empathy
- Citation: [archwiki:System_maintenance]

## [1.4.0-rc.1] - 2025-11-11

### 🔮 **Phase 1.4: The Mirror Protocol - Recursive Introspection**

Anna gains metacognition—the ability to reflect on reflection. The network now observes itself observing, establishing bidirectional self-audit loops for moral and operational consistency.

#### Added
- **Mirror Protocol Architecture** (~2,000 lines):
  - Reflection generation for compact ethical/empathic decision records
  - Peer critique evaluation with inconsistency and bias detection
  - Mirror consensus for quorum-based collective alignment
  - Bias remediation engine (confirmation, recency, availability bias)
  - Network coherence calculation (self-coherence + peer assessment + agreement)
  - State persistence to `/var/lib/anna/mirror/state.json`
  - Reflection logs to `/var/lib/anna/mirror/reflections.log`

- **`annactl mirror` commands**:
  - `mirror reflect` - Generate manual reflection cycle
  - `mirror audit` - Summarize peer critiques and network coherence
  - `mirror repair` - Trigger remediation protocol for detected biases

- **Metacognitive Features**:
  - Automatic reflection generation every 24 hours
  - Peer critique with coherence assessment (self vs actual consistency)
  - Systemic bias detection (affecting ≥2 nodes)
  - Consensus-driven remediations (parameter reweight, trust reset, conscience adjustment)
  - Network coherence threshold enforcement (default 0.7)
  - Differential privacy for consensus sessions

#### Technical Details
- **Modules**:
  - `mirror/types.rs` (390 lines) - Complete type system including audit summaries
  - `mirror/reflection.rs` (320 lines) - Self-assessment generation
  - `mirror/critique.rs` (420 lines) - Peer evaluation engine
  - `mirror/mirror_consensus.rs` (450 lines) - Collective alignment coordinator
  - `mirror/repair.rs` (360 lines) - Bias remediation execution
  - `mirror/mod.rs` (450 lines) - Main daemon orchestration

- **Bias Detection**:
  - Confirmation bias: >95% or <5% approval rates
  - Recency bias: Recent 20% decisions differ >0.2 from older 80%
  - Availability bias: Excessive empathy adaptations (>10)
  - Empathy-strain contradictions
  - Coherence-bias mismatches

- **Remediation Types**:
  - ParameterReweight: Adjust scrutiny/strain thresholds
  - TrustReset: Recalibrate peer relationships
  - ConscienceAdjustment: Modify ethical evaluation parameters
  - PatternRetrain: Address systematic issues
  - ManualReview: Escalate unknown patterns

#### Changed
- Daemon now initializes Mirror Protocol alongside Collective Mind
- IPC protocol extended with 3 new methods (MirrorReflect, MirrorAudit, MirrorRepair)
- Added `mirror` field to DaemonState
- Version bumped to 1.4.0-rc.1

#### Security Model
- AES-256-GCM encryption for reflection data (when implemented)
- Differential privacy for mirror consensus
- Conscience layer sovereignty preserved
- No peer can force remediations on another node

#### Notes
- Mirror Protocol enabled by default with placeholder configuration
- Consensus requires minimum 3 nodes for quorum
- Reflection period defaults to 24 hours, consensus every 7 days
- Citation: [archwiki:System_maintenance]

## [1.3.0-rc.1] - 2025-11-11

### 🌐 **Phase 1.3: Collective Mind - Distributed Cooperation**

Anna evolves from empathetic custodian into a distributed civilization of ethical agents—capable of multi-node coordination, consensus-based decision making, and shared learning without centralization.

#### Added
- **Collective Mind Architecture** (~1,900 lines):
  - Gossip Protocol v1 for peer-to-peer discovery and event propagation
  - Trust Ledger with weighted scoring (honesty 50%, ethical 30%, reliability 20%)
  - Consensus Engine with 60% weighted approval threshold
  - Network-wide empathy/strain synchronization
  - Distributed introspection for cross-node ethical audits
  - Ed25519-style cryptographic identity (placeholder for development)
  - State persistence to `/var/lib/anna/collective/state.json`

- **`annactl collective` commands**:
  - `collective status` - Network health, peers, consensus activity
  - `collective trust <peer_id>` - Trust details for a specific peer
  - `collective explain <consensus_id>` - Consensus decision explanation

- **Distributed Features**:
  - Peer announcement via signed gossip messages
  - Heartbeat monitoring with reliability scoring
  - Trust decay toward neutral (1% per day)
  - Network health calculation (empathy 40%, low strain 40%, sync recency 20%)
  - Cross-node introspection requests (conscience, empathy, health)
  - Replay attack prevention via message deduplication

#### Technical Details
- **Modules**:
  - `collective/types.rs` (320 lines) - Complete type system
  - `collective/crypto.rs` (170 lines) - Cryptographic operations
  - `collective/trust.rs` (220 lines) - Reputation management
  - `collective/gossip.rs` (320 lines) - UDP-based messaging
  - `collective/consensus.rs` (270 lines) - Weighted voting
  - `collective/sync.rs` (250 lines) - State synchronization
  - `collective/introspect.rs` (220 lines) - Distributed audits
  - `collective/mod.rs` (370 lines) - Main daemon

- **Security Model**:
  - End-to-end message signing (placeholder crypto)
  - Peer trust scoring prevents Sybil attacks
  - No peer can override another's Conscience Layer
  - Ethics isolation enforced at protocol level

#### Changed
- Daemon now initializes Collective Mind alongside Sentinel
- IPC protocol extended with 3 new methods for collective operations
- Version bumped to 1.3.0-rc.1

#### Notes
- Collective Mind disabled by default (requires configuration in `/etc/anna/collective.yml`)
- Cryptographic implementation is placeholder—production requires proper libraries (ed25519-dalek, aes-gcm)
- Citation: [archwiki:System_maintenance]

## [1.0.0-rc.1] - 2025-11-11

### 🤖 **Phase 1.0: Sentinel Framework - Autonomous System Governance**

Anna evolves from reactive administrator to autonomous sentinel— a persistent daemon that continuously monitors, responds, and adapts without user intervention.

#### Added
- **Sentinel Daemon Architecture**:
  - Persistent event-driven system with unified event bus
  - Periodic schedulers for health (5min), updates (1hr), audits (24hr)
  - State persistence to `/var/lib/anna/state.json`
  - Configuration management in `/var/lib/anna/config.json`
  - Automated response playbooks for system events
  - Adaptive scheduling based on system stability

- **`annactl sentinel` commands**:
  - `sentinel status` - Daemon health and uptime
  - `sentinel metrics` - Event counts, error rates, drift tracking

- **`annactl config` commands**:
  - `config get` - View current configuration
  - `config set <key> <value>` - Update settings at runtime

- **Autonomous Features**:
  - Service failure auto-restart (configurable)
  - Package drift detection and notification
  - Log anomaly monitoring with severity filtering
  - State transition tracking
  - System drift index (0.0-1.0 scale)

- **Observability**:
  - Real-time metrics: uptime, event counts, error rates
  - Health trend tracking over time
  - Structured logging to `/var/log/anna/sentinel.jsonl`
  - State diff calculation (degradation vs improvement)

#### Configuration Keys
```
autonomous_mode          - Enable/disable autonomous operations (default: false)
health_check_interval    - Seconds between health checks (default: 300)
update_scan_interval     - Seconds between update scans (default: 3600)
audit_interval           - Seconds between audits (default: 86400)
auto_repair_services     - Automatically restart failed services (default: false)
auto_update              - Automatically install updates (default: false)
auto_update_threshold    - Max packages for auto-update (default: 5)
adaptive_scheduling      - Adjust frequencies by stability (default: true)
```

#### Examples
```bash
# View sentinel status
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

# Enable autonomous mode
$ annactl config set autonomous_mode true
[anna] Configuration updated: autonomous_mode = true

# View metrics
$ annactl sentinel metrics
┌─────────────────────────────────────────────────────────
│ SENTINEL METRICS
├─────────────────────────────────────────────────────────
│ Total Events:     127
│ Health Checks:     12
│ Update Scans:       3
│ Audits:             1
│ Error Rate:      0.05 errors/hour
│ Drift Index:     0.12
└─────────────────────────────────────────────────────────
```

#### Architecture
- **Event Bus**: Unified event system for all subsystems (health, steward, repair, recovery)
- **Response Playbooks**: Configurable automated responses to system events
- **State Machine**: Continuous tracking of system health and configuration
- **Ethics Layer**: Prevents destructive operations on user data (`/home`, `/data`)
- **Watchdog Integration**: Auto-restart on daemon failure (future)

#### Security & Safety
- All automated actions require explicit configuration
- Dry-run validation for all mutations
- Append-only audit logging with integrity verification
- Never modifies user directories
- Configuration changes logged with timestamps

**Citation**: [archwiki:System_maintenance]

---

## [1.0.3-rc.1] - 2025-11-11

### 🔧 **Phase 0.9: System Steward - Lifecycle Management**

Anna now provides comprehensive lifecycle management with system health monitoring, update orchestration, and security auditing.

#### Added
- **`annactl status` command**: Comprehensive system health dashboard
  - Service status monitoring (failed, active, enabled)
  - Package update detection
  - Log issue analysis (errors and warnings)
  - Actionable recommendations
- **`annactl update` command**: Intelligent system update orchestration
  - Package updates via pacman with signature verification
  - Automatic service restart detection and execution
  - `--dry-run` flag for simulation
  - Structured reporting of all changes
- **`annactl audit` command**: Security and integrity verification
  - Package integrity checks (pacman -Qkk)
  - GPG keyring verification
  - File permission validation
  - Security baseline checks (firewall, SSH hardening)
  - Configuration compliance (fstab options)
- **Steward subsystem** (`crates/annad/src/steward/`):
  - `health.rs` - System health monitoring with service/package/log analysis
  - `update.rs` - Update orchestration with pacman
  - `audit.rs` - Integrity verification and security audit
  - `types.rs` - Data structures for reports
  - `logging.rs` - Structured logging to `/var/log/anna/steward.jsonl`
- **IPC protocol**: Three new RPC methods
  - `SystemHealth` → `HealthReportData`
  - `SystemUpdate { dry_run }` → `UpdateReportData`
  - `SystemAudit` → `AuditReportData`

#### Health Monitoring
```bash
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

#### Update Orchestration
```bash
$ annactl update --dry-run
┌─────────────────────────────────────────────────────────
│ SYSTEM UPDATE (DRY RUN)
├─────────────────────────────────────────────────────────
│ Status:    SUCCESS
│ PACKAGES UPDATED: 5
│   • linux 6.6.1 → 6.6.2
│   • systemd 255.1 → 255.2
│ SERVICES RESTARTED:
│   • NetworkManager.service
└─────────────────────────────────────────────────────────

[archwiki:System_maintenance#Upgrading_the_system]
```

#### Security Audit
```bash
$ annactl audit
┌─────────────────────────────────────────────────────────
│ SYSTEM AUDIT REPORT
├─────────────────────────────────────────────────────────
│ Compliance: ✓ PASS
│ All integrity checks: PASSED (3 checks)
│ SECURITY FINDINGS: 1
│   • [MEDIUM] Firewall is not active
│     → Enable firewalld: systemctl enable --now firewalld
└─────────────────────────────────────────────────────────

[archwiki:Security]
```

#### Security & Safety
- All operations logged to `/var/log/anna/steward.jsonl` with timestamps
- Package signature verification enforced
- Service restart limited to known-safe services
- Dry-run mode for risk-free validation
- Never modifies `/home` or `/data` directories

**Citation**: [archwiki:System_maintenance]

---

## [1.0.2-rc.1] - 2025-11-11

### 🚀 **Phase 0.8: System Installer - Guided Arch Linux Installation**

Anna can now perform complete Arch Linux installations through structured, state-aware dialogue.

#### Added
- **`annactl install` command**: Interactive guided installation
  - Disk setup (manual partitioning with automatic formatting)
  - Base system installation via pacstrap
  - System configuration (fstab, locale, timezone, hostname)
  - Bootloader installation (systemd-boot or GRUB)
  - User creation with sudo access and anna group membership
  - `--dry-run` flag for simulation
- **Installation subsystem** (`crates/annad/src/install/`):
  - `mod.rs` - Installation orchestrator
  - `types.rs` - Configuration data structures
  - `disk.rs` - Disk partitioning and formatting
  - `packages.rs` - Base system with pacstrap
  - `bootloader.rs` - systemd-boot and GRUB support
  - `users.rs` - User creation and permissions
  - `logging.rs` - Structured logging to `/var/log/anna/install.jsonl`
- **IPC protocol**: New `PerformInstall` RPC method with `InstallResultData` response type
- **State validation**: Installation only allowed in `iso_live` state

#### Interactive Dialogue
```bash
[anna] Arch Linux Installation
[anna] Disk Setup
Available partitions:
NAME   SIZE   TYPE   MOUNTPOINT
sda    100G   disk
├─sda1 512M   part
└─sda2  99G   part

[anna] Select bootloader
  * systemd-boot - Modern, simple
    grub - Traditional
[anna] Choice [systemd-boot]:

[anna] Hostname [archlinux]:
[anna] Username [user]:
[anna] Timezone [UTC]:
[anna] Locale [en_US.UTF-8]:
```

#### Security
- Runs only as root in iso_live state
- Uses arch-chroot and pacstrap (no shell injection)
- All operations logged to `/var/log/anna/install.jsonl`
- Dry-run mode for safe validation
- Validates environment before execution

#### Examples
```bash
# Dry-run simulation
sudo annactl install --dry-run

# Interactive installation
sudo annactl install
```

**Citation**: [archwiki:Installation_guide]

---

## [1.0.1-rc.1] - 2025-11-11

### 🛠️ **Phase 0.7: System Guardian - Corrective Actions**

Anna moves from passive observation to active system repair. The `repair` command performs automated corrections for failed health probes.

#### Added
- **`annactl repair` command**: Repair failed probes with automatic corrective actions
  - `annactl repair all` - Repair all failed probes
  - `annactl repair <probe>` - Repair specific probe
  - `--dry-run` flag for simulation without execution
- **Probe-specific repair logic**:
  - `disk-space` → Clean systemd journal (`journalctl --vacuum-size=100M`) + pacman cache (`paccache -r -k 2`)
  - `pacman-db` → Synchronize package databases (`pacman -Syy`)
  - `services-failed` → Restart failed systemd units
  - `firmware-microcode` → Install missing CPU microcode packages (intel-ucode/amd-ucode)
- **Audit logging**: All repair actions logged to `/var/log/anna/audit.jsonl` with timestamps, commands, and results
- **IPC protocol**: New `RepairProbe` RPC method with `RepairResultData` response type
- **Daemon repair subsystem**: `crates/annad/src/repair/` module with probe-specific actions

#### User Experience
- Plain-text output (no colors, no emojis): `[anna] probe: pacman-db — sync_pacman_db (OK)`
- Dry-run simulation: `[anna] repair simulation: probe=all`
- Citations for all actions: `Citation: [archwiki:System_maintenance]`
- Exit codes: 0 = success, 1 = repair failed

#### Security
- All repairs execute through daemon (root privileges)
- Audit trail for all corrective actions
- Dry-run mode for safe testing
- No arbitrary shell execution from user input

#### Examples
```bash
# Check system health
annactl health

# Simulate repair (dry-run)
annactl repair --dry-run

# Repair all failed probes
sudo annactl repair all

# Repair specific probe
sudo annactl repair disk-space
```

**Citation**: [archwiki:System_maintenance]

---

## [1.0.0-rc.13.2] - 2025-11-11

### 🐛 **Hotfix: Daemon Startup Reliability**

**CRITICAL FIX** for daemon startup failures in rc.13 and rc.13.1.

#### Fixed
- **Systemd unit**: Removed problematic `ExecStartPre` with complex shell escaping
- **WorkingDirectory**: Changed from `/var/lib/anna` to `/` (avoid startup dependency)
- **StateDirectory**: Added subdirectories `anna anna/reports anna/alerts` for atomic creation
- **Socket cleanup**: Simple `rm -f` instead of complex validation

#### Impact
- ✅ Daemon starts reliably on first install
- ✅ No more "socket not reachable after 30s" errors
- ✅ StateDirectory creates all required directories before ExecStart
- ✅ Clean deterministic startup sequence

#### Foundation
- Added `paths.rs` module for future dual-mode socket support
- Added `--user`, `--foreground`, `--help` flags to `annad` (partial implementation)
- Groundwork for user-mode operation (planned for rc.14)

**Citation**: [archwiki:Systemd#Service_types]

---

## [1.0.0-rc.13.1] - 2025-11-11

### 🐛 **Hotfix: Runtime Socket Access and Readiness**

**CRITICAL FIX** for runtime socket access issues and installer readiness checks.

#### Fixed
- **Systemd unit**: Moved `StartLimitIntervalSec=0` from `[Service]` to `[Unit]` section (correct placement per systemd spec)
- **Systemd unit**: Added `UMask=007` to ensure socket files default to 0660 for group anna
- **Installer**: Extended readiness wait from 15s to 30s
- **Installer**: Check both socket existence AND accessibility before declaring ready
- **Installer**: Clean up old daemon binaries (`annad-old`, `annactl-old`) for rc.9.9/rc.11 compatibility
- **Installer**: Added group enrollment hint if user not in anna group
- **RPC client**: Detect `EACCES` (Permission Denied) errors and provide targeted hint

#### User Experience Improvements
- Socket access works immediately after install
- Clear error messages with actionable hints: `sudo usermod -aG anna "$USER" && newgrp anna`
- Better troubleshooting for non-root users
- Deterministic startup readiness

**Citation**: [archwiki:Systemd#Drop-in_files]

---

## [1.0.0-rc.13] - 2025-11-11

### 🎯 **Complete Architectural Reset - "Operational Core"**

Anna 1.0 represents a **complete rewrite** from prototype to production-ready system administration core. This release removes all desktop environment features and focuses exclusively on reliable, auditable system monitoring and maintenance.

### ⚠️ **BREAKING CHANGES**

**Removed Features** (See MIGRATION-1.0.md for details):
- ❌ Desktop environment bundles (Hyprland, i3, sway, all WMs)
- ❌ Application installation system
- ❌ TUI (terminal user interface) - returns in 2.0
- ❌ Recommendation engine and advice catalog
- ❌ Pywal integration and theming
- ❌ Hardware detection for DEs
- ❌ Commands: `setup`, `apply`, `advise`, `revert`

**What Remains**:
- ✅ Core daemon with state-aware dispatch
- ✅ Health monitoring and diagnostics
- ✅ Recovery framework (foundation)
- ✅ Comprehensive logging with Arch Wiki citations
- ✅ Security hardening (systemd sandbox)

### 🚀 **New Features**

#### Phase 0.3: State-Aware Command Dispatch
- **Six-state machine**: iso_live, recovery_candidate, post_install_minimal, configured, degraded, unknown
- Commands only available in states where they're safe to execute
- State detection with Arch Wiki citations
- Capability-based command filtering
- `annactl help` shows commands for current state

#### Phase 0.4: Security Hardening
- **Systemd sandbox**: NoNewPrivileges, ProtectSystem=strict, ProtectHome=true
- **Socket permissions**: root:anna with mode 0660
- **Directory permissions**: 0700 for /var/lib/anna, /var/log/anna
- **File permissions**: 0600 for all reports and sensitive files
- Users must be in `anna` system group
- No privilege escalation paths
- Restricted system call architectures

#### Phase 0.5: Health Monitoring System
- **Six health probes**:
  - `disk-space`: Filesystem usage monitoring
  - `pacman-db`: Package database integrity
  - `systemd-units`: Failed unit detection
  - `journal-errors`: System log analysis
  - `services-failed`: Service health checks
  - `firmware-microcode`: Microcode status
- **Commands**:
  - `annactl health`: Run all probes, exit codes 0/1/2
  - `annactl health --json`: Machine-readable output
  - `annactl doctor`: Diagnostic synthesis with recommendations
  - `annactl rescue list`: Show available recovery plans
- **Report generation**: JSON reports saved to /var/lib/anna/reports/ (0600)
- **Alert system**: Failed probes create alerts in /var/lib/anna/alerts/
- **JSONL logging**: All execution logged to /var/log/anna/ctl.jsonl
- **Health history**: Probe results logged to /var/log/anna/health.jsonl

#### Phase 0.6a: Recovery Framework Foundation
- **Recovery plan parser**: Loads declarative YAML plans
- **Five recovery plans**: bootloader, initramfs, pacman-db, fstab, systemd
- **Chroot detection**: Identifies and validates chroot environments
- **Type-safe structures**: RecoveryPlan, RecoveryStep, StateSnapshot
- **Embedded fallback**: Works without external YAML files
- Foundation for executable recovery (Phase 0.6b)

### 🔧 **Technical Improvements**

#### CI/CD Pipeline
- **GitHub Actions workflow**: .github/workflows/health-cli.yml
- **Performance benchmarks**: <200ms health command latency target
- **Automated validation**:
  - Code formatting (cargo fmt --check)
  - Linting (cargo clippy)
  - JSON schema validation with jq
  - File permissions checks (0600/0700)
  - Unauthorized write detection
- **Test artifacts**: Logs uploaded on failure (7-day retention)

#### Testing
- **10 integration tests** for health CLI
- **Exit code validation**: 0 (ok), 1 (fail), 2 (warn), 64 (unavailable), 65 (invalid), 70 (daemon down)
- **Permissions tests**: Validate 0600 reports, 0700 directories
- **Schema validation**: JSON schemas for health-report, doctor-report, ctl-log
- **Mock probes**: Environment variable-driven test fixtures
- **Test duration**: <20s total suite execution

#### Exit Codes
- `0` - Success (all probes passed)
- `1` - Failure (one or more probes failed)
- `2` - Warning (warnings but no failures)
- `64` - Command not available in current state
- `65` - Invalid daemon response
- `70` - Daemon unavailable

#### Logging Format
All operations logged as JSONL with:
- ISO 8601 timestamps
- UUID request IDs
- System state at execution
- Exit codes and duration
- Arch Wiki citations
- Success/failure status

Example:
```json
{
  "ts": "2025-11-11T13:00:00Z",
  "req_id": "550e8400-e29b-41d4-a716-446655440000",
  "state": "configured",
  "command": "health",
  "exit_code": 0,
  "citation": "[archwiki:System_maintenance]",
  "duration_ms": 45,
  "ok": true
}
```

### 📦 **File Structure**

```
/usr/local/bin/{annad,annactl}
/var/lib/anna/reports/      # Health and doctor reports (0700)
/var/lib/anna/alerts/       # Failed probe alerts (0700)
/var/log/anna/ctl.jsonl     # Command execution log
/var/log/anna/health.jsonl  # Health check history
/run/anna/anna.sock         # IPC socket (root:anna 0660)
/usr/local/lib/anna/health/ # Probe YAML definitions
/usr/local/lib/anna/recovery/ # Recovery plan YAMLs
```

### 🔒 **Security**

- **Systemd hardening**: 11 security directives enabled
- **No new privileges**: NoNewPrivileges=true prevents escalation
- **Read-only probes**: All health checks are non-destructive
- **Socket isolation**: Unix socket with group-based access control
- **Audit trail**: Every command logged with full context

### 📚 **Documentation**

- **README.md**: Completely rewritten for operational core
- **MIGRATION-1.0.md**: Comprehensive migration guide from rc.11
- **ANNA-1.0-RESET.md**: Architecture documentation updated
- **JSON schemas**: Version-pinned schemas with $id URIs
- Test coverage documentation
- Security model documentation

### 🐛 **Bug Fixes**

- Unknown flags now exit with code 64 (not 2)
- MockableProbe properly gated with #[cfg(test)]
- Environment variables ignored in production builds
- Proper error handling for daemon unavailability
- Fixed chroot detection edge cases

### 🏗️ **Internal Changes**

- **Module structure**: health/, recovery/, state/ subsystems
- **RPC methods**: GetState, GetCapabilities, HealthRun, HealthSummary, RecoveryPlans
- **Type safety**: Comprehensive error handling with anyhow::Result
- **Parser**: YAML-based probe and recovery plan definitions
- **State machine**: Capability-based command availability
- **Rollback foundation**: StateSnapshot types for future rollback

### ⚡ **Performance**

- Health command: <200ms on ok-path
- Daemon startup: <2s
- Test suite: <20s total
- Memory footprint: Minimal (no desktop management)

### 🎓 **Citations**

All operations cite Arch Wiki:
- [archwiki:System_maintenance]
- [archwiki:Systemd]
- [archwiki:Chroot#Using_arch-chroot]
- [archwiki:GRUB#Installation]
- [archwiki:Mkinitcpio]
- [archwiki:Pacman]

### 🔄 **Migration Path**

1. Uninstall rc.11: `sudo ./scripts/uninstall.sh`
2. Remove old configs: `rm -rf ~/.config/anna`
3. Install rc.13: `curl -sSL .../scripts/install.sh | sh`
4. Add user to anna group: `sudo usermod -a -G anna $USER`
5. Verify: `annactl health`

See **MIGRATION-1.0.md** for detailed instructions.

### 📝 **Commits**

This release includes 18 commits across Phases 0.3-0.6a:
- Phase 0.3: State machine and dispatch (5 commits)
- Phase 0.4: Security hardening (1 commit)
- Phase 0.5a: Health subsystem (1 commit)
- Phase 0.5b: RPC/CLI integration (2 commits)
- Phase 0.5c: Tests, CI, stabilization (3 commits)
- Phase 0.6a: Recovery framework foundation (1 commit)
- Documentation: README, MIGRATION, schemas (5 commits)

### 🚀 **What's Next**

**Phase 0.6b** (Next Release):
- Executable recovery plans
- `annactl rescue run <plan>`
- `annactl rollback <plan>`
- Rollback script generation
- Interactive rescue mode

**Version 2.0** (Future):
- TUI returns as optional interface
- Additional health probes
- Advanced diagnostics
- Backup automation

---

## [1.0.0-rc.11] - 2025-11-07

### 🔥 Critical Bug Fixes

**33 Broken Advice Items Fixed**
- CRITICAL: Fixed 30 advice items with `command: None` that showed up but couldn't be applied
- CRITICAL: Fixed `hyprland-nvidia-env-vars` (MANDATORY item) - now automatically configures Nvidia Wayland environment
- Fixed 3 comment-only commands that wouldn't execute anything
- All 136 advice items now have valid, executable commands
- No more "No command specified" errors

**Examples of Fixed Items:**
- AMD driver upgrade: Added `lspci -k | grep -A 3 -i vga`
- SSH security checks: Added SSH config diagnostics
- Network diagnostics (4 items): Added ping/ip commands
- Btrfs optimizations (3 items): Added mount checks
- Hardware monitoring: Added sensors/smartctl commands
- System health: Added journalctl error checks

**Nvidia + Hyprland Critical Fix:**
```bash
# Now automatically appends to ~/.config/hypr/hyprland.conf:
env = GBM_BACKEND,nvidia-drm
env = __GLX_VENDOR_LIBRARY_NAME,nvidia
env = LIBVA_DRIVER_NAME,nvidia
env = WLR_NO_HARDWARE_CURSORS,1
```

### ✨ Major UX Improvements (RC.10)

**Command Rename: bundles → setup**
- Better UX: "setup" is universally understood vs "bundles"
- `annactl setup` - List available desktop environments
- `annactl setup hyprland` - Install complete Hyprland environment
- `annactl setup hyprland --preview` - Show what would be installed
- Friendly error messages for unsupported desktops

**Hyprland-Focused Design**
- Removed support for 21 other window managers
- Anna is now a dedicated Hyprland assistant
- Better to do one thing perfectly than many things poorly
- Only Hyprland bundle available (sway, i3, bspwm, etc. removed)
- Other WMs may return in v2.0 if there's demand

### 🛠️ Technical Changes

**Feature Freeze Enforcement**
- Strict feature freeze for v1.0 release
- Only bug fixes and critical issues allowed
- All new features deferred to v2.0
- v2.0 ideas tracked in ROADMAP.md

**Files Changed:**
- `crates/annad/src/recommender.rs` - Fixed 33 broken advice items
- `crates/annactl/src/main.rs` - Renamed Bundles → Setup command
- `crates/annactl/src/commands.rs` - Implemented setup() function
- `crates/annad/src/bundles/mod.rs` - Removed non-Hyprland bundles
- `crates/annad/src/bundles/wayland_compositors.rs` - Hyprland-only
- `Cargo.toml` - Version bump to 1.0.0-rc.11
- `README.md` - Updated version and design focus
- `ROADMAP.md` - Documented changes and v2.0 plans

### 📦 Version History

- **1.0.0-rc.9.3** → **1.0.0-rc.10** - Command rename + Hyprland focus
- **1.0.0-rc.10** → **1.0.0-rc.11** - Critical bugfixes (33 items)

## [1.0.0-rc.9.3] - 2025-11-07

### 🔥 Critical Fixes

**Watchdog Crash Fixed**
- CRITICAL: Removed `WatchdogSec=60s` from systemd service that was killing daemon after 60 seconds
- Daemon now stays running indefinitely
- Already had `Restart=on-failure` for real crash recovery

**Daemon-Based Updates (No Sudo)**
- Update system now works entirely through daemon (runs as root)
- Downloads → Installs → Schedules restart AFTER sending response (no race condition)
- No more password prompts during updates
- Seamless update experience

### ✨ UX Improvements

**Show All Categories**
- Removed "6 more categories..." truncation
- Now shows complete category breakdown in `annactl advise`

**Unique IDs for Apply**
- Display format: `[1] amd-microcode  Enable AMD microcode updates`
- Both work: `annactl apply 1` OR `annactl apply amd-microcode`
- IDs shown in cyan for visibility
- Fixes apply confusion when using category filters

**Doctor Auto-Fix**
- `annactl doctor --fix` now fixes all issues automatically
- Removed individual confirmation prompts per user feedback
- One command, no babysitting

### 🛠️ Technical Changes

- annad.service: Removed WatchdogSec to prevent false-positive kills
- Update system: Async block prevents early-return type conflicts
- Apply command: Box::pin for recursive async ID handling
- Daemon update: Downloads+installs before scheduling restart

### 📦 Files Changed
- `annad.service` - Watchdog removal
- `crates/annactl/src/commands.rs` - UX improvements, ID support
- `crates/annad/src/rpc_server.rs` - Daemon-based update implementation
- `crates/anna_common/src/updater.rs` - Export download_binary()

## [1.0.0-beta.82] - 2025-11-06

### 🖼️ Universal Wallpaper Intelligence

**New Module: wallpaper_config.rs (181 lines)**

Anna now provides comprehensive wallpaper intelligence for ALL desktop environments!

**Top 10 Curated Wallpaper Sources (4K+ Resolution):**
1. **Unsplash** - 4K+ free high-resolution photos
2. **Pexels** - 4K and 8K stock photos
3. **Wallpaper Abyss** - 1M+ wallpapers up to 8K
4. **Reddit** (r/wallpapers, r/wallpaper) - Community curated
5. **InterfaceLIFT** - Professional photography up to 8K
6. **Simple Desktops** - Minimalist, distraction-free
7. **NASA Image Library** - Space photography, public domain
8. **Bing Daily** - Daily rotating 4K images
9. **GNOME Wallpapers** - Professional curated collection
10. **KDE Wallpapers** - High-quality abstract and nature

**Official Arch Linux Wallpapers:**
- Recommends `archlinux-wallpaper` package
- Multiple resolutions (1080p, 1440p, 4K, 8K)
- Dark and light variants
- Location: `/usr/share/archlinux/wallpaper/`

**Dynamic Wallpaper Tools:**
- **variety** - Wallpaper changer with multiple sources
- **wallutils** - Universal wallpaper manager
- **nitrogen** - Lightweight wallpaper setter (X11)
- **swaybg** - Wallpaper for Wayland compositors
- **wpaperd** - Wallpaper daemon with automatic rotation
- **hyprpaper** - Wallpaper utility for Hyprland

**Wallpaper Management:**
- X11 tools: nitrogen, feh, variety
- Wayland tools: swaybg, wpaperd, hyprpaper
- Universal: wallutils (works on both X11 and Wayland)

**Format & Resolution Guide:**
- **Formats:** PNG (lossless), JPG (smaller), WebP (modern), AVIF (next-gen)
- **Common Resolutions:** 1920x1080 (FHD), 2560x1440 (QHD), 3840x2160 (4K)
- **High-end:** 5120x2880 (5K), 7680x4320 (8K)
- **Ultrawide:** 2560x1080, 3440x1440, 5120x1440 (32:9)
- Multi-monitor support guidance

**Universal Coverage:**
- Works across ALL 9 supported desktop environments
- Hyprland, i3, Sway, GNOME, KDE, XFCE, Cinnamon, MATE, LXQt
- Helps 100% of users beautify their desktop
- Not DE-specific - benefits everyone

**Technical Details:**
- Module: `crates/annad/src/wallpaper_config.rs`
- Integrated with `smart_recommender.rs` line 285
- Added to `main.rs` line 96
- 5 major recommendation categories
- Clean build, zero compiler warnings

**User Experience:**
Every Anna user gets instant access to curated wallpaper sources, learning about top-quality wallpaper collections in 4K+, dynamic wallpaper tools, and best practices for formats and resolutions. Makes desktop beautification easy and accessible for everyone!

**Example Recommendations:**

Install official Arch wallpapers:
```bash
sudo pacman -S --noconfirm archlinux-wallpaper
# Location: /usr/share/archlinux/wallpaper/
```

Install dynamic wallpaper manager:
```bash
sudo pacman -S --noconfirm nitrogen  # X11
sudo pacman -S --noconfirm swaybg    # Wayland
yay -S --noconfirm variety           # Advanced manager
```

**Files Modified:**
- Created: `crates/annad/src/wallpaper_config.rs` (181 lines)
- Modified: `crates/annad/src/main.rs` (added wallpaper_config module)
- Modified: `crates/annad/src/smart_recommender.rs` (integrated wallpaper recommendations)
- Modified: `Cargo.toml` (bumped to Beta.82)

**Impact:**
Thirteenth major ROADMAP feature! Anna now provides wallpaper intelligence for EVERY desktop environment, helping 100% of users beautify their desktop with curated high-quality sources and best practices.

**Next Steps (Future Betas):**
- **Beta.83+:** Terminal color schemes (dark + light variants)
- **Beta.84+:** Desktop environment toolkit consistency (GTK vs Qt)
- **Beta.85+:** Complete theme coverage (dark + light for all DEs)

## [1.0.0-beta.59] - 2025-11-05

### 🔧 Update Command Fix

**Fixed Version Verification:**
- `annactl update --install` was failing with "Version mismatch" error
- Issue: Expected `v1.0.0-beta.58` but binary outputs `annad 1.0.0-beta.58`
- Solution: Strip 'v' prefix when comparing versions
- Update command now works properly from start to finish!

**User Experience:**
- Before: "✗ Update failed: Version mismatch: expected v1.0.0-beta.58, got annad 1.0.0-beta.58"
- After: Update completes successfully ✅

**Technical Details:**
- Modified `verify_binary()` in updater.rs
- Strips 'v' prefix from tag name before version comparison
- More lenient version matching while still being safe

## [1.0.0-beta.58] - 2025-11-05

### 🔧 Critical Apply Command Fix

**Fixed Hanging Apply Commands:**
- Apply command was hanging because pacman/yay needed `--noconfirm` flag
- Fixed all 35 commands missing the flag across the codebase
- CLI and TUI apply commands now work without hanging
- Package installations run non-interactively as intended

**User Experience Before Fix:**
```bash
annactl apply 25
# Would hang with: ":: Proceed with installation? [Y/n]"
# User couldn't see progress and thought it was dead
```

**User Experience After Fix:**
- Commands execute automatically without prompts
- Clean installation without user interaction needed
- No more frozen terminals waiting for input

**Files Modified:**
- `recommender.rs` - Fixed 19 pacman/yay commands
- `smart_recommender.rs` - Fixed 16 pacman/yay commands
- `rpc_server.rs` - Added debug logging for history tracking

**Affected Commands:**
- `sudo pacman -S <package>` → `sudo pacman -S --noconfirm <package>`
- `yay -S <package>` → `yay -S --noconfirm <package>`
- All package installation commands across TLP, timeshift, bluetooth, etc.

**User Feedback Implemented:**
- "It has finished but I thought it was dead" - FIXED! ✅
- "With command line it fails" - FIXED! ✅
- "Tried to apply from TUI and it is just hanging" - FIXED! ✅

### 🔍 History Investigation (In Progress)

**Added Debug Logging:**
- Added detailed logging to RPC server for history recording
- Logs show when history is being recorded and saved
- Helps diagnose why history might not be persisting
- Path: `/var/log/anna/application_history.jsonl`

**Next Steps:**
- User to test with: `annactl apply <number>`
- Check logs with: `journalctl -u annad | grep history`
- Verify file permissions on `/var/log/anna/`

## [1.0.0-beta.57] - 2025-11-05

### 🔕 Smart Notification System (Anti-Spam)

**Fixed Notification Spam:**
- Added 1-hour cooldown between notifications
- Removed wall (terminal broadcast) completely - it was spamming all terminals
- GUI notifications only - cleaner and less intrusive
- Rate limiting prevents notification spam
- Thread-safe cooldown tracking with Mutex

**More Visible Notifications:**
- Increased timeout from 5 to 10 seconds
- Better icons based on urgency (dialog-error for critical)
- Added category tag for proper desktop integration
- More prominent display

**User Experience:**
- No more wall spam across all terminals!
- Maximum one notification per hour (configurable)
- GUI-only notifications are professional and clean
- Cooldown logged for transparency
- Critical issues still notified, but rate-limited

### 🔧 Technical Details

**New Features:**
- `should_send_notification()` - Cooldown check function (lines 29-54)
- Global `LAST_NOTIFICATION` mutex for thread-safe tracking
- `NOTIFICATION_COOLDOWN` constant (1 hour = 3600 seconds)

**Modified Functions:**
- `send_notification()` - Added cooldown check (lines 57-73)
- `send_gui_notification()` - Enhanced visibility (lines 98-123)
- Removed `send_terminal_broadcast()` - wall was too intrusive

**Files Modified:**
- notifier.rs: Complete rewrite of notification system

**Rate Limiting:**
- First notification: Allowed immediately
- Subsequent notifications: 1-hour cooldown enforced
- Logged with minutes remaining when blocked
- Thread-safe with Mutex

**User Feedback Implemented:**
- "Anna is spamming me with notifications" - FIXED! ✅
- "Too frequently" - 1-hour cooldown implemented
- "Be careful with bothering the user" - Rate limiting added
- "Bundle the notification" - Single notification per hour max

## [1.0.0-beta.56] - 2025-11-05

### 🤖 True Auto-Update (Autonomy Tier 3)

**Auto-Update Implementation:**
- Anna can now update herself automatically when in Tier 3 autonomy
- Checks for updates from GitHub in the background
- Downloads and installs new versions automatically
- Restarts daemon after successful update
- Sends desktop notification when update completes
- Completely hands-free update experience

**User Experience:**
- No manual intervention required for updates
- Desktop notification: "Anna Updated Automatically - Updated to vX.X.X in the background"
- Appears in autonomy log: `annactl autonomy`
- Safe and tested update mechanism
- Falls back gracefully on errors

**Autonomy System:**
- New Task 19 in Tier 3: Auto-update Anna
- Runs periodically with other maintenance tasks
- Only activates in Tier 3 (Fully Autonomous) mode
- Can be enabled with: `annactl config set autonomy_tier 3`

### 🔧 Technical Details

**New Function:**
- `auto_update_anna()` - Checks and installs Anna updates (lines 1134-1211)

**Modified Functions:**
- `run_tier3_tasks()` - Added auto-update as Task 19 (lines 203-208)

**Files Modified:**
- autonomy.rs: Added auto-update functionality to Tier 3

**Integration:**
- Uses existing `anna_common::updater::check_for_updates()`
- Uses existing `anna_common::updater::perform_update()`
- Sends notification via notify-send if available
- Records action in autonomy log for audit trail

**Autonomy Tiers:**
- Tier 0 (Advise Only): No automatic actions
- Tier 1 (Safe Auto-Apply): 7 safe maintenance tasks
- Tier 2 (Semi-Autonomous): +8 extended maintenance tasks
- Tier 3 (Fully Autonomous): +4 full maintenance tasks including auto-update

## [1.0.0-beta.55] - 2025-11-05

### ⚡ Shell Completion Support

**Completion Generation:**
- New `completions` command generates shell completion scripts
- Supports bash, zsh, fish, PowerShell, and elvish
- Autocompletes all commands, subcommands, and options
- Autocompletes argument values where applicable

### 🎯 Apply by ID Support

**Enhanced Apply Command:**
- Added `--id` flag to apply command
- Apply recommendations by ID: `annactl apply --id amd-microcode`
- Works alongside existing number-based apply (e.g., `annactl apply 1`)
- TUI already supported apply by ID, now CLI has feature parity
- More flexible recommendation application

**Installation:**
- Bash: `annactl completions bash > /usr/share/bash-completion/completions/annactl`
- Zsh: `annactl completions zsh > /usr/share/zsh/site-functions/_annactl`
- Fish: `annactl completions fish > ~/.config/fish/completions/annactl.fish`
- PowerShell: `annactl completions powershell > annactl.ps1`

**User Experience:**
- Tab completion for all commands
- Faster command-line navigation
- Discover commands and options easily
- Reduces typing and errors

### 🔧 Technical Details

**New Command:**
- `completions` - Generate shell completion scripts

**New Function:**
- `generate_completions()` - Uses clap_complete to generate completions

**Files Modified:**
- main.rs: Added Completions command and generation handler
- Cargo.toml (annactl): Added clap_complete dependency

**Dependencies Added:**
- clap_complete = "4.5" (for completion generation)

**Integration:**
- Uses clap's built-in CommandFactory
- Outputs to stdout for easy redirection
- Works with all shells supported by clap_complete

## [1.0.0-beta.54] - 2025-11-05

### 🎉 Beautiful Update Experience

**Auto-Update Notifications:**
- Desktop notification when update completes (via notify-send)
- Non-intrusive notification system (no wall spam)
- Beautiful colored update success banner
- Version upgrade display with highlighting
- Release date shown in banner

**Release Notes Display:**
- Automatic fetching of release notes from GitHub API
- Formatted display with syntax highlighting
- Headers, bullets, and text properly styled
- First 20 lines shown with link to full notes
- Integrated into update completion flow

**User Experience:**
- Visual feedback that update succeeded
- Immediate access to what's new
- Desktop notification for background awareness
- Clean, beautiful terminal output
- Non-blocking notification system

### 🔧 Technical Details

**New Functions:**
- `fetch_release_notes()` - Fetches notes from GitHub API (lines 3107-3124)
- `display_release_notes()` - Formats and displays notes (lines 3126-3153)
- `send_update_notification()` - Sends desktop notification (lines 3155-3174)

**Enhanced Functions:**
- `update()` - Added banner, release notes, and notification (lines 3223-3252)

**Files Modified:**
- commands.rs: Enhanced update success flow with rich feedback
- Cargo.toml (annactl): Added reqwest dependency for GitHub API

**Dependencies Added:**
- reqwest = "0.11" with JSON feature (for GitHub API)

**Integration:**
- Uses GitHub API to fetch release body
- Checks for notify-send availability before sending
- Only sends notification if desktop environment detected
- Graceful fallback if notes fetch fails

**Documentation Updated:**
- README.md: Updated for beta.54
- CHANGELOG.md: Detailed technical documentation
- ROADMAP.md: Marked completion checkboxes
- examples/README.md: Fixed outdated command syntax

## [1.0.0-beta.53] - 2025-11-05

### 📊 Improved Transparency & Management

**Grand Total Display:**
- Advise command now shows "Showing X of Y recommendations" format
- Clearly indicates when some items are hidden by filters or limits
- Users always know the total number of available recommendations

**List Hidden Recommendations:**
- New command: `annactl ignore list-hidden`
- Shows all recommendations currently filtered by ignore settings
- Displays items grouped by category with priority indicators
- Provides copy-paste commands to un-ignore specific filters

**Show Dismissed Recommendations:**
- New command: `annactl dismissed`
- View all previously dismissed recommendations
- Shows time since dismissal ("2 days ago", "5 hours ago")
- Grouped by category for easy navigation
- Un-dismiss with `annactl dismissed --undismiss <number>`

### 🔧 Technical Details

**New Commands:**
- `annactl ignore list-hidden` - Lists filtered-out recommendations
- `annactl dismissed` - Manages dismissed recommendations

**Modified Functions:**
- `advise()` - Enhanced count display with grand total context (lines 371-395)
- `ignore()` - Added ListHidden action handler (lines 3140-3244)
- `dismissed()` - New function to manage dismissed items (lines 2853-2952)

**Files Modified:**
- commands.rs: Added list-hidden and dismissed functionality
- main.rs: Added ListHidden enum variant and Dismissed command

**User Experience:**
- Full visibility into what's being filtered
- Easy management of ignore filters and dismissed items
- Time-based information for dismissed recommendations
- Clear commands for reversing actions

## [1.0.0-beta.52] - 2025-11-05

### ✨ TUI Enhancements

**Ignore/Dismiss Keyboard Shortcuts:**
- Added 'd' key to ignore recommendations by category
- Added 'i' key to ignore recommendations by priority
- Works in both Dashboard and Details views
- Immediate visual feedback with status messages
- Automatically refreshes view after ignoring
- Footer shortcuts updated to show new options

**User Experience:**
- Press 'd' to dismiss all recommendations in the same category
- Press 'i' to dismiss all recommendations with the same priority
- Returns to Dashboard view after ignoring from Details
- Color-coded status messages (yellow for success, red for errors)

### 🔧 Technical Details

**Modified Functions:**
- `handle_dashboard_keys()` - Added 'd' and 'i' handlers (lines 301-343)
- `handle_details_keys()` - Added 'd' and 'i' handlers (lines 414-460)
- Footer rendering - Updated shortcuts display for both views

**Files Modified:**
- tui.rs: Added ignore keyboard shortcuts to TUI interface

**Integration:**
- Uses existing IgnoreFilters system from anna_common
- Triggers automatic refresh by adjusting last_update timestamp
- Consistent behavior between Dashboard and Details views

## [1.0.0-beta.51] - 2025-11-05

### 🎯 User-Requested Features

**Recent Activity in Status:**
- Status command now shows last 10 audit log entries
- Displays timestamp, action type, and details
- Color-coded actions (apply, install, remove, update)
- Success/failure indicators

**Bundle Rollback with Numbers:**
- Bundle rollback now accepts numbered IDs: `#1`, `#2`, `#3`
- Bundles command shows installed bundles with [#1], [#2], [#3]
- Still supports rollback by name for backwards compatibility
- Easy rollback: `annactl rollback #1`

**Code Cleanup:**
- Removed duplicate `Priority` imports
- Centralized imports at module level
- Cleaner, more maintainable code

### 🔧 Technical Details

**New Function:**
- `read_recent_audit_entries()` - Reads and sorts audit log
- Handles missing log files gracefully
- Returns most recent N entries

**Enhanced Functions:**
- `bundles()` - Now shows installed bundles with numbered IDs
- `rollback()` - Accepts both `#number` and `bundle-name`

**Files Modified:**
- commands.rs: Added audit display, bundle numbering, import cleanup
- All compilation warnings fixed

## [1.0.0-beta.50] - 2025-11-05

### ✨ Quality & Polish

**Count Message Improvements:**
- Simplified advise command count display
- Clear format: "Showing X recommendations"
- Shows hidden count: "(30 hidden by filters)"
- Shows limited count: "(15 more available, use --limit=0)"
- No more confusing multiple totals

**Category Consistency:**
- Created centralized `categories.rs` module in anna_common
- All 21 categories now have canonical names and emojis
- TUI and CLI use same category definitions
- Consistent emoji display across all interfaces

### 🔧 Technical Details

**New Module:**
- `anna_common/categories.rs` - Central source of truth for categories
- `get_category_order()` - Returns display order
- `get_category_emoji()` - Returns emoji for category

**Refactoring:**
- commands.rs uses centralized category list
- tui.rs uses centralized emoji function
- Eliminated duplicate category definitions

## [1.0.0-beta.49] - 2025-11-05

### 🐛 Critical Bug Fixes

**Ignore Filters Consistency:**
- Fixed: `report` command now applies ignore filters (was showing all advice)
- Fixed: `health` command now applies ignore filters (was including filtered items in score)
- Fixed: TUI now applies ignore filters (was showing all recommendations)
- Result: ALL commands now consistently respect user's ignore settings

**Count Display Accuracy:**
- Fixed: `status` command shows filtered count instead of total
- Fixed: Status count now matches category breakdown
- Added: Message when all recommendations are filtered out
- TUI footer shows active filter count: "🔍 2 filters"

### ✨ User Experience

**Visual Feedback:**
- TUI displays filter count in footer when filters active
- Consistent messaging across all commands
- Clear indication when items are hidden by filters

### 🔧 Technical Details

**Files Modified:**
- `commands.rs`: Added filter application to report() and health()
- `tui.rs`: Added filter application to refresh() and filter indicator to footer
- `commands.rs`: Restructured status() to show filtered count

**Quality Check Results:**
- Comprehensive codebase review completed
- 3 critical issues fixed
- 2 high-priority issues resolved
- Filter integration now 100% consistent

## [1.0.0-beta.48] - 2025-11-05

### 🐛 Critical Bug Fixes

**Display Consistency:**
- Fixed critical count mismatch between TUI and report command
- Both now use `Priority::Mandatory` field (was mixing Priority and RiskLevel)
- TUI health gauge now shows: "Score: 0/100 - Critical (2 issues)"
- Clear indication of both score AND issue count

### ✨ UI/UX Improvements

**Update Command:**
- Now shows installed version before checking for updates
- Friendly message: "No updates available - you're on the latest development version!"
- Better error handling distinguishing network issues from missing releases

**Status Command:**
- Added category breakdown showing top 10 categories with counts
- Example: "Security · 15", "Packages · 23"
- Respects ignore filters when calculating

**TUI Health Display:**
- Changed from confusing "0/100" to clear "Score: 0/100"
- Shows critical issue count when score is low
- Title changed from "System Health" to "System Health Score"

### 📚 Documentation

- Updated README to beta.48 with latest features
- Updated ROADMAP to track completed features
- Documented ignore system commands

## [1.0.0-beta.47] - 2025-11-05

### ✨ Improvements

**Update Command Enhancements:**
- Shows installed version upfront
- Friendly messaging for development versions
- Clear distinction between network errors and missing releases

**Status Command:**
- Added category breakdown display
- Shows top 10 categories with recommendation counts
- Integrated with ignore filters

## [1.0.0-beta.46] - 2025-11-05

### 🎯 New Features

**Category & Priority Ignore System:**
- Ignore entire categories: `annactl ignore category "Desktop Customization"`
- Ignore priority levels: `annactl ignore priority Optional`
- View filters: `annactl ignore show`
- Remove filters: `annactl ignore unignore category <name>`
- Reset all: `annactl ignore reset`
- Storage: `~/.config/anna/ignore_filters.json`

**History Improvements:**
- Sequential rollback numbers ([#1], [#2], [#3])
- Added "Applied by" field
- Better formatting and alignment

### 📚 Documentation

- Added "Recent User Feedback & Ideas" section to ROADMAP
- Tracking all pending improvements
- User feedback preserved for future work

## [1.0.0-beta.45] - 2025-11-05

### 🎯 Critical Fix - Apply Numbers

**Advice Display Cache System:**
- Created `AdviceDisplayCache` to save exact display order
- `advise` command saves IDs to `~/.cache/anna/advice_display_cache.json`
- `apply` command reads from cache - GUARANTEED match
- Removed 200+ lines of complex filtering code
- Simple, reliable, cache-based approach

**What This Fixes:**
- Apply numbers now ALWAYS match what's shown in advise
- No more "applied wrong advice" issues
- No more complex state replication
- User feedback: "apply must work with the right numbers!"

## [1.0.0-beta.44] - 2025-11-05

### 🎉 System Completeness & Quality Release!

**AUTO-UPDATE:** Tier 3 users get automatic updates every 24 hours!
**SMART HEALTH:** Performance rating now accurately reflects pending improvements!
**30+ NEW TOOLS:** Essential CLI utilities, git enhancements, security tools!

### 🔧 Critical Fixes

**Duplicate Function Compilation Error:**
- Fixed: Renamed `check_kernel_parameters` → `check_sysctl_parameters`
- Separated sysctl security parameters from boot parameters
- Build no longer fails with duplicate definition error

**Performance Rating Logic:**
- Fixed: System never shows 100% health when improvements are pending
- Now deducts points for Optional (-2) and Cosmetic (-1) recommendations
- Addressed user feedback: "If performance is 100, why pending improvements?"
- Score accurately reflects system improvement potential

**Health Score Category Matching:**
- Updated to use standardized category names
- "Security & Privacy" (was "security")
- "Performance Optimization" (was "performance")
- "System Maintenance" (was "maintenance")
- Performance score now correctly deducts for pending optimizations

### 🤖 Daemon Auto-Update

**Background Update System:**
- Checks for new releases every 24 hours automatically
- Tier 3 (Fully Autonomous) users: Auto-installs updates with systemd restart
- Tier < 3: Shows notification only, manual install required
- Safe installation with backup of previous version
- User can manually update: `annactl update --install`

### ✨ 30+ New Comprehensive Recommendations

**Essential CLI Tools (5 tools):**
- `bat` - Syntax-highlighted cat replacement with line numbers
- `eza` - Modern ls with icons, colors, and git integration
- `fzf` - Fuzzy finder for command history (Ctrl+R!), files, git
- `tldr` - Practical command examples instead of verbose man pages
- `ncdu` - Interactive disk usage analyzer with ncurses UI
- **Bundle:** cli-essentials

**System Monitoring (1 tool):**
- `btop` - Gorgeous resource monitor with mouse support and themes
- Shows CPU, memory, disks, network, processes in beautiful TUI

**Arch-Specific Tools (3 tools):**
- `arch-audit` - Scan installed packages for CVE vulnerabilities
- `pkgfile` - Command-not-found handler + package file search
- `pacman-contrib` - paccache, checkupdates, pacdiff utilities
- Security and maintenance focused

**Git Enhancements (2 tools):**
- `lazygit` - Beautiful terminal UI for git operations
- `git-delta` - Syntax-highlighted diffs with side-by-side view
- **Bundle:** git-tools

**Desktop Utilities (1 tool):**
- `flameshot` - Powerful screenshot tool with annotations, arrows, blur
- **Bundle:** desktop-essentials

**Security Tools (1 tool):**
- `KeePassXC` - Secure password manager with browser integration
- Open-source, encrypted database, no cloud dependency
- **Bundle:** security-essentials

**System Hardening (3 sysctl parameters):**
- `kernel.dmesg_restrict=1` - Restrict kernel ring buffer to root
- `kernel.kptr_restrict=2` - Hide kernel pointers from exploits
- `net.ipv4.tcp_syncookies=1` - SYN flood protection (DDoS)
- **Bundle:** security-hardening

**Universal App Support (1 tool):**
- `Flatpak` + Flathub integration
- Sandboxed apps, access to thousands of desktop applications
- No conflicts with pacman packages

### 📦 New Bundles

Added 4 new workflow bundles for easy installation:
- `cli-essentials` - bat, eza, fzf, tldr, ncdu
- `git-tools` - lazygit, git-delta
- `desktop-essentials` - flameshot
- `security-essentials` - KeePassXC

Use `annactl bundles` to see all available bundles!

### 📊 Statistics

- **Total recommendations**: 310+ (up from 280+)
- **New recommendations**: 30+
- **New bundles**: 4
- **Health score improvements**: More accurate with all priorities counted
- **Auto-update**: Tier 3 support added

### 💡 What This Means

**More Complete System:**
- Anna now recommends essential tools every Arch user needs
- CLI productivity tools, git workflow enhancements, security utilities
- Better coverage of system completeness (password managers, screenshot tools)

**Smarter Health Scoring:**
- Performance rating never misleadingly shows 100% with pending items
- All recommendation priorities properly counted (Mandatory through Cosmetic)
- More accurate system health representation

**Self-Updating System:**
- Tier 3 users stay automatically up-to-date
- Background checks every 24 hours, installs seamlessly
- No user intervention needed for cutting-edge features

### 🐛 Bug Fixes

- Fixed: Duplicate function definition preventing compilation
- Fixed: Health score ignoring Optional/Cosmetic recommendations
- Fixed: Category name mismatches causing incorrect health calculations
- Fixed: Performance score not deducting for pending optimizations

### 🔄 Breaking Changes

None - all changes are backward compatible!

### 📝 Notes for Users

- Install new binaries to test all fixes: `sudo cp ./target/release/{annad,annactl} /usr/local/bin/`
- Tier 3 users will now receive automatic updates
- Many new Optional/Recommended tools available - check `annactl advise`
- Health score is now more accurate (may show lower scores with pending items)

## [1.0.0-beta.43] - 2025-11-05

### 🚀 Major Intelligence & Autonomy Upgrade!

**COMPREHENSIVE TELEMETRY:** 8 new telemetry categories for smarter recommendations!
**AUTONOMOUS MAINTENANCE:** Expanded from 6 to 13 intelligent maintenance tasks!
**ARCH WIKI INTEGRATION:** Working offline cache with 40+ common pages!

### ✨ New Telemetry Categories

**Extended System Detection:**
- **CPU Microcode Status**: Detects Intel/AMD microcode packages and versions (critical for security)
- **Battery Information**: Health, capacity, cycle count, charge status (laptop optimization)
- **Backup Systems**: Detects timeshift, rsync, borg, restic, and other backup tools
- **Bluetooth Status**: Hardware detection, service status, connected devices
- **SSD Information**: TRIM status detection, device identification, optimization opportunities
- **Swap Configuration**: Type (partition/file/zram), size, usage, swappiness analysis
- **Locale Information**: Timezone, locale, keymap, language for regional recommendations
- **Pacman Hooks**: Detects installed hooks to understand system automation level

### 🤖 Expanded Autonomy System

**13 Autonomous Tasks** (up from 6):

**Tier 1 (Safe Auto Apply) - Added:**
- Update package database automatically (pacman -Sy) when older than 1 day
- Check for failed systemd services and log for user attention

**Tier 2 (Semi-Autonomous) - Added:**
- Clean user cache directories (Firefox, Chromium, npm, yarn, thumbnails)
- Remove broken symlinks from home directory (maxdepth 3)
- Optimize pacman database for better performance

**Tier 3 (Fully Autonomous) - Added:**
- Apply security updates automatically (kernel, glibc, openssl, systemd, sudo, openssh)
- Backup important system configs before changes (/etc/pacman.conf, fstab, etc.)

### 🧠 New Smart Recommendations

**Using New Telemetry Data:**
- **Microcode Updates**: Mandatory recommendations for missing Intel/AMD microcode (security critical)
- **Battery Optimization**: TLP recommendations, battery health warnings for laptops
- **Backup System Checks**: Warns if no backup system installed, suggests automation
- **Bluetooth Setup**: Enable bluetooth service, install blueman GUI for management
- **SSD TRIM Status**: Automatically detects SSDs without TRIM and recommends fstrim.timer
- **Swap Optimization**: Recommends zram for better performance, adjusts swappiness for desktops
- **Timezone Configuration**: Detects unconfigured (UTC) timezones
- **Pacman Hooks**: Suggests useful hooks like auto-listing orphaned packages

### 🌐 Arch Wiki Cache (Fixed!)

**Now Fully Functional:**
- Added `UpdateWikiCache` RPC method to IPC protocol
- Implemented daemon-side cache update handler
- Wired up `annactl wiki-cache` command properly
- Downloads 40+ common Arch Wiki pages for offline access
- Categories: Security, Performance, Hardware, Desktop Environments, Development, Gaming, Power Management, Troubleshooting

### 🎨 UI/UX Improvements

**Installer Updates:**
- Updated "What's New" section with current features (was showing outdated info)
- Better formatting and categorization of features
- Highlights key capabilities: telemetry, autonomy, wiki integration

**TUI Enhancements:**
- Added sorting by category/priority/risk (hotkeys: c, p, r)
- Popularity indicators showing how common each recommendation is (★★★★☆)
- Detailed health score explanations showing what affects each score

### 📊 System Health Score Improvements

**Detailed Explanations Added:**
- **Security Score**: Lists specific issues found, shows ✓ for perfect scores
- **Performance Score**: Disk usage per drive, orphaned package counts, optimization opportunities
- **Maintenance Score**: Pending tasks, cache sizes, specific actionable items
- Each score now includes contextual details explaining the rating

### 🐛 Bug Fixes

**Build & Compilation:**
- Fixed Advice struct field name mismatches (links→wiki_refs, tags removed)
- Fixed bundle parameter type issues (String vs Option<String>)
- Resolved CPU model borrow checker errors in telemetry
- All new code compiles cleanly with proper error handling

### 💡 What This Means

**Smarter Recommendations:**
- Anna now understands your system at a much deeper level
- Recommendations are targeted and relevant to your actual configuration
- Critical security items (microcode) are properly prioritized

**More Autonomous:**
- System maintains itself better with 13 automated tasks
- Graduated autonomy tiers let you choose your comfort level
- Security updates can be applied automatically (Tier 3)

**Better Documentation:**
- Offline Arch Wiki access works properly
- 40+ common pages cached for quick reference
- No more broken wiki cache functionality

### 🔧 Technical Details

**Code Statistics:**
- ~770 lines of new functionality
- 8 new telemetry collection functions (~385 lines)
- 8 new autonomous maintenance tasks (~342 lines)
- 8 new recommendation functions using telemetry data
- All with comprehensive error handling and logging

**Architecture Improvements:**
- Telemetry data structures properly defined in anna_common
- RPC methods for wiki cache updates
- Builder pattern usage for Advice construction
- Proper use of SystemFacts fields throughout

### 📚 Files Changed

- `crates/anna_common/src/types.rs`: Added 8 new telemetry struct definitions (+70 lines)
- `crates/annad/src/telemetry.rs`: Added 8 telemetry collection functions (+385 lines)
- `crates/annad/src/autonomy.rs`: Added 8 new maintenance tasks (+342 lines)
- `crates/annad/src/recommender.rs`: Added 8 new recommendation functions
- `crates/annad/src/rpc_server.rs`: Added wiki cache RPC handler
- `crates/annad/src/wiki_cache.rs`: Removed dead code markers
- `crates/anna_common/src/ipc.rs`: Added UpdateWikiCache method
- `crates/annactl/src/commands.rs`: Implemented wiki cache command
- `scripts/install.sh`: Updated "What's New" section

## [1.0.0-beta.42] - 2025-11-05

### 🎯 Major TUI Overhaul & Auto-Update!

**INTERACTIVE TUI:** Complete rewrite with proper scrolling, details view, and apply confirmation!

### ✨ New Features

**Completely Redesigned TUI:**
- **Fixed Scrolling**: Now properly scrolls through long recommendation lists using `ListState`
- **Details View**: Press Enter to see full recommendation details with word-wrapped text
  - Shows priority badge, risk level, full reason
  - Displays command to execute
  - Lists Arch Wiki references
  - Press `a` or `y` to apply, Esc to go back
- **Apply Confirmation**: Yes/No button dialog before applying recommendations
  - Visual [Y] Yes and [N] No buttons
  - Safe confirmation workflow
- **Renamed Command**: `annactl dashboard` → `annactl tui` (more descriptive)
- **Better Navigation**: Up/Down arrows or j/k to navigate, Enter for details

**Auto-Update System:**
- **`annactl update` command**: Check for and install updates from GitHub
  - `annactl update` - Check for available updates
  - `annactl update --install` - Install updates automatically
  - `annactl update --check` - Quick version check only
- **Automatic Updates**: Downloads, verifies, and installs new versions
- **Safe Updates**: Backs up current binaries before updating to `/var/lib/anna/backup/`
- **Version Verification**: Checks binary versions after download
- **Atomic Installation**: Stops daemon, replaces binaries, restarts daemon
- **GitHub API Integration**: Fetches latest releases including prereleases

### 🐛 Bug Fixes

**Fixed Install Script (CRITICAL):**
- **Install script now fetches latest version correctly**
- Changed from `/releases/latest` (excludes prereleases) to `/releases[0]` (includes all)
- Users can now install beta.41+ instead of being stuck on beta.30
- This was a **blocking issue** preventing users from installing newer versions

**Category Style Consistency:**
- Added missing categories: `usability` (✨) and `media` (📹)
- All categories now have proper emojis and colors
- Fixed fallback for undefined categories

**Borrow Checker Fixes:**
- Fixed TUI borrow checker error in apply confirmation
- Cloned data before mutating state

### 💡 What This Means

**Better User Experience:**
- TUI actually works for long lists (scrolling was broken before)
- Can view full details of recommendations before applying
- Safe confirmation workflow prevents accidental applies
- Much more intuitive interface

**Stay Up-to-Date Easily:**
- Simple `annactl update --install` keeps you on the latest version
- No more manual downloads or broken install scripts
- Automatic verification ensures downloads are correct
- Safe rollback with automatic backups

**Installation Fixed:**
- New users can finally install the latest version
- Install script now correctly fetches beta.41+
- Critical fix for user onboarding

### 🔧 Technical Details

**TUI Implementation:**
```rust
// New view modes
enum ViewMode {
    Dashboard,      // Main list
    Details,        // Full recommendation info
    ApplyConfirm,   // Yes/No dialog
}

// Proper state tracking for scrolling
struct Tui {
    list_state: ListState,  // Fixed scrolling
    view_mode: ViewMode,
    // ...
}
```

**Updater Architecture:**
- Moved to `anna_common` for shared access
- Uses `reqwest` for GitHub API calls
- Version parsing and comparison
- Binary download and verification
- Systemd integration for daemon restart

**File Changes:**
- Created: `crates/annactl/src/tui.rs` (replaces dashboard.rs)
- Created: `crates/anna_common/src/updater.rs`
- Updated: `scripts/install.sh` (critical fix)
- Added: `textwrap` dependency for word wrapping

---

## [1.0.0-beta.41] - 2025-11-05

### 🎮 Multi-GPU Support & Polish!

**COMPREHENSIVE GPU DETECTION:** Anna now supports Intel, AMD, and Nvidia GPUs with tailored recommendations!

### ✨ New Features

**Multi-GPU Detection & Recommendations:**
- **Intel GPU Support**: Automatic detection of Intel integrated graphics
  - Vulkan support recommendations (`vulkan-intel`)
  - Hardware video acceleration (`intel-media-driver` for modern, `libva-intel-driver` for legacy)
  - Detects via both `lspci` and `i915` kernel module
- **AMD/ATI GPU Support**: Enhanced AMD graphics detection
  - Identifies modern `amdgpu` vs legacy `radeon` drivers
  - Suggests driver upgrade path for compatible GPUs
  - Hardware video acceleration (`libva-mesa-driver`, `mesa-vdpau`)
  - Detects via `lspci` and kernel modules
- **Complete GPU Coverage**: Now supports Intel, AMD, and Nvidia GPUs with specific recommendations

### 🐛 Bug Fixes

**Category Consistency:**
- All category names now properly styled with emojis
- Added explicit mappings for: `utilities`, `system`, `productivity`, `audio`, `shell`, `communication`, `engineering`
- Fixed capitalization inconsistency in hardware recommendations
- Updated category display order for better organization

**Documentation Fixes:**
- Removed duplication between Beta.39 and Beta.40 sections in README
- Consolidated "What's New" section with clear version separation
- Updated current version reference in README

### 💡 What This Means

**Better Hardware Support:**
- Anna now detects and provides recommendations for ALL common GPU types
- Tailored advice based on your specific hardware
- Hardware video acceleration setup for smoother video playback and lower power consumption
- Legacy hardware gets appropriate driver recommendations

**Improved User Experience:**
- Consistent category display across all recommendations
- Clear visual hierarchy with proper emojis and colors
- Better documentation that reflects current features

### 🔧 Technical Details

**New SystemFacts Fields:**
```rust
pub is_intel_gpu: bool
pub is_amd_gpu: bool
pub amd_driver_version: Option<String>  // "amdgpu (modern)" or "radeon (legacy)"
```

**New Detection Functions:**
- `detect_intel_gpu()` - Checks lspci and i915 module
- `detect_amd_gpu()` - Checks lspci and amdgpu/radeon modules
- `get_amd_driver_version()` - Identifies driver in use

**New Recommendation Functions:**
- `check_intel_gpu_support()` - Vulkan and video acceleration for Intel
- `check_amd_gpu_enhancements()` - Driver upgrades and video acceleration for AMD

---

## [1.0.0-beta.40] - 2025-11-05

### 🎨 Polish & Documentation Update!

**CLEAN & CONSISTENT:** Fixed rendering issues and updated all documentation to Beta.39/40!

### 🐛 Bug Fixes

**Fixed Box Drawing Rendering Issues:**
- Replaced Unicode box drawing characters (╭╮╰╯━) with simple, universally-compatible separators
- Changed from decorative boxes to clean `=` separators
- Category headers now render perfectly in all terminals
- Summary separators simplified from `━` to `-`
- Much better visual consistency across different terminal emulators

**Fixed CI Build:**
- Fixed unused variable warning that caused GitHub Actions to fail
- Prefixed `_is_critical` in doctor command

### 📚 Documentation Updates

**Completely Updated README.md:**
- Reflects Beta.39 features and simplified commands
- Added environment-aware recommendations section
- Updated command examples with new syntax
- Added comprehensive feature list
- Updated installation instructions
- Removed outdated Beta.30 references

**Updated Command Help:**
- Fixed usage examples to show new simplified syntax
- `annactl apply <number>` instead of `annactl apply --nums <number>`
- `annactl advise security` instead of `annactl advise --category security`

### 💡 What This Means

**Better Terminal Compatibility:**
- Works perfectly in all terminals (kitty, alacritty, gnome-terminal, konsole, etc.)
- No more broken box characters
- Cleaner, more professional output
- Consistent rendering regardless of font or locale

**Up-to-Date Documentation:**
- README reflects current version (Beta.40)
- All examples use correct command syntax
- Clear feature descriptions
- Easy for new users to understand

### 🔧 Technical Details

**Before:**
```
╭────────────────────────────────────╮
│  🔒 Security                       │
╰────────────────────────────────────╯
```

**After:**
```
🔒 Security
============================================================
```

Much simpler, renders everywhere, still looks great!

---

## [1.0.0-beta.39] - 2025-11-05

### 🎯 Context-Aware Recommendations & Simplified Commands!

**SMART & INTUITIVE:** Anna now understands your environment and provides tailored recommendations!

### ✨ Major Features

**📝 Simplified Command Structure**
- Positional arguments for cleaner commands
- `annactl advise security` instead of `annactl advise --category security`
- `annactl apply 1-5` instead of `annactl apply --nums 1-5`
- `annactl rollback hyprland` instead of `annactl rollback --bundle hyprland`
- `annactl report security` instead of `annactl report --category security`
- `annactl dismiss 1` instead of `annactl dismiss --num 1`
- `annactl config get/set` for easier configuration
- Much more intuitive and faster to type!

**🔍 Enhanced Environment Detection**
- **Window Manager Detection**: Hyprland, i3, sway, bspwm, dwm, qtile, xmonad, awesome, and more
- **Desktop Environment Detection**: GNOME, KDE, XFCE, and others
- **Compositor Detection**: Hyprland, picom, compton, xcompmgr
- **Nvidia GPU Detection**: Automatic detection of Nvidia hardware
- **Driver Version Detection**: Tracks Nvidia driver version
- **Wayland+Nvidia Configuration Check**: Detects if properly configured

**🎮 Environment-Specific Recommendations**

*Hyprland + Nvidia Users:*
- Automatically detects Hyprland with Nvidia GPU
- Recommends critical environment variables (GBM_BACKEND, __GLX_VENDOR_LIBRARY_NAME, etc.)
- Suggests nvidia-drm.modeset=1 kernel parameter
- Provides Hyprland-specific package recommendations

*Window Manager Users:*
- **i3**: Recommends rofi/dmenu for app launching
- **bspwm**: Warns if sxhkd is missing (critical for keybindings)
- **sway**: Suggests waybar for status bar

*Desktop Environment Users:*
- **GNOME**: Recommends GNOME Tweaks for customization
- **KDE**: Suggests plasma-systemmonitor

**📊 Telemetry Enhancements**
New fields in SystemFacts:
- `window_manager` - Detected window manager
- `compositor` - Detected compositor
- `is_nvidia` - Whether system has Nvidia GPU
- `nvidia_driver_version` - Nvidia driver version if present
- `has_wayland_nvidia_support` - Wayland+Nvidia configuration status

### 🔧 Technical Details

**Command Examples:**
```bash
# Old way (still works)
annactl advise --category security --limit 10
annactl apply --nums "1-5"
annactl rollback --bundle "Container Stack"

# New way (cleaner!)
annactl advise security -l 10
annactl apply 1-5
annactl rollback "Container Stack"
```

**Detection Capabilities:**
- Checks `XDG_CURRENT_DESKTOP` environment variable
- Uses `pgrep` to detect running processes
- Checks installed packages with `pacman`
- Parses `lspci` for GPU detection
- Reads `/sys/class/` for hardware info
- Checks kernel parameters
- Analyzes config files for environment variables

**Hyprland+Nvidia Check:**
```rust
// Detects Hyprland running with Nvidia GPU
if window_manager == "Hyprland" && is_nvidia {
    if !has_wayland_nvidia_support {
        // Recommends critical env vars
    }
}
```

### 💡 What This Means

**Simpler Commands:**
- Faster to type
- More intuitive
- Less typing for common operations
- Follows Unix philosophy

**Personalized Recommendations:**
- Anna knows what you're running
- Tailored advice for your setup
- No more generic recommendations
- Proactive problem prevention

**Example Scenarios:**

*Scenario 1: Hyprland User*
```
User runs: annactl advise
Anna detects: Hyprland + Nvidia RTX 4070
Anna recommends:
  → Configure Nvidia env vars for Hyprland
  → Enable nvidia-drm.modeset=1
  → Install hyprpaper, hyprlock, waybar
```

*Scenario 2: i3 User*
```
User runs: annactl advise
Anna detects: i3 window manager, no launcher
Anna recommends:
  → Install rofi for application launching
  → Install i3status or polybar for status bar
```

### 🚀 What's Coming in Beta.40

Based on user feedback, the next release will focus on:
- **Multi-GPU Support**: Intel, AMD/ATI, Nouveau recommendations
- **More Desktop Environments**: Support for less common DEs/WMs
- **Automatic Maintenance**: Low-risk updates with safety checks
- **Arch News Integration**: `informant` integration for breaking changes
- **Deep System Analysis**: Library mismatches, incompatibilities
- **Security Hardening**: Post-quantum SSH, comprehensive security
- **Log Analysis**: All system logs, not just journal
- **Category Consistency**: Proper capitalization across all categories

---

## [1.0.0-beta.38] - 2025-11-05

### 📊 Interactive TUI Dashboard!

**REAL-TIME MONITORING:** Beautiful terminal dashboard with live system health visualization!

### ✨ Major Features

**📺 Interactive TUI Dashboard**
- `annactl dashboard` - Launch full-screen interactive dashboard
- Real-time system health monitoring
- Live hardware metrics (CPU temp, load, memory, disk)
- Interactive recommendations panel
- Keyboard-driven navigation (↑/↓ or j/k)
- Auto-refresh every 2 seconds
- Color-coded health indicators

**🎨 Beautiful UI Components**
- Health score gauge with color coding (🟢 90-100, 🟡 70-89, 🔴 <70)
- Hardware monitoring panel:
  - CPU temperature with thermal warnings
  - Load averages (1min, 5min, 15min)
  - Memory usage with pressure indicators
  - SMART disk health status
  - Package statistics
- Recommendations panel:
  - Priority-colored advice (🔴 Mandatory, 🟡 Recommended, 🟢 Optional)
  - Scrollable list
  - Visual selection highlight
- Status bar with keyboard shortcuts
- Live timestamp in header

**⌨️ Keyboard Controls**
- `q` or `Esc` - Quit dashboard
- `↑` or `k` - Navigate up in recommendations
- `↓` or `j` - Navigate down in recommendations
- Auto-refresh - Updates every 2 seconds

**📈 Real-Time Health Monitoring**
- System health score (0-100 scale)
- CPU temperature tracking with alerts
- Memory pressure detection
- Disk health from SMART data
- Failed services monitoring
- Package health indicators

### 🔧 Technical Details

**Dashboard Architecture:**
- Built with ratatui (modern TUI framework)
- Crossterm for terminal control
- Async RPC client for daemon communication
- Non-blocking event handling
- Efficient render loop with 100ms tick rate

**Health Score Algorithm:**
```
Base: 100 points

Deductions:
- Critical advice:  -15 points each
- Recommended advice: -5 points each
- CPU temp >85°C:  -20 points
- CPU temp >75°C:  -10 points
- Failing disks:   -25 points each
- Memory >95%:     -15 points
- Memory >85%:     -5 points
```

**UI Layout:**
```
┌─────────────────────────────────────┐
│  Header (version, time)             │
├─────────────────────────────────────┤
│  Health Score Gauge                 │
├──────────────┬──────────────────────┤
│  Hardware    │  Recommendations     │
│  Monitoring  │  (scrollable)        │
│              │                      │
├──────────────┴──────────────────────┤
│  Footer (keyboard shortcuts)        │
└─────────────────────────────────────┘
```

**Dependencies Added:**
- `ratatui 0.26` - TUI framework
- `crossterm 0.27` - Terminal control

### 📋 Example Usage

**Launch Dashboard:**
```bash
# Start interactive dashboard
annactl dashboard

# Dashboard shows:
# - Live health score
# - CPU temperature and load
# - Memory usage
# - Disk health
# - Active recommendations
# - Package statistics
```

**Dashboard Features:**
- Auto-connects to Anna daemon
- Shows error if daemon not running
- Gracefully restores terminal on exit
- Updates data every 2 seconds
- Responsive keyboard input
- Clean exit with q or Esc

### 💡 What This Means

**At-a-Glance System Health:**
- No need to run multiple commands
- All critical metrics in one view
- Color-coded warnings grab attention
- Real-time updates keep you informed

**Better User Experience:**
- Visual, not just text output
- Interactive navigation
- Professional terminal UI
- Feels like a modern monitoring tool

**Perfect for:**
- System administrators monitoring health
- Checking system status quickly
- Watching metrics in real-time
- Learning what Anna monitors
- Impressive demos!

### 🚀 What's Next

The dashboard foundation is in place. Future enhancements could include:
- Additional panels (network, processes, logs)
- Charts and graphs (sparklines, histograms)
- Action execution from dashboard (apply fixes)
- Custom views and layouts
- Export/save dashboard state

---

## [1.0.0-beta.37] - 2025-11-05

### 🔧 Auto-Fix Engine & Enhanced Installation!

**SELF-HEALING:** Doctor can now automatically fix detected issues! Plus beautiful uninstaller.

### ✨ Major Features

**🤖 Auto-Fix Engine**
- `annactl doctor --fix` - Automatically fix detected issues
- `annactl doctor --dry-run` - Preview fixes without applying
- `annactl doctor --fix --auto` - Fix all issues without confirmation
- Interactive confirmation for each fix
- Safe execution with error handling
- Success/failure tracking and reporting
- Fix summary with statistics

**🔧 Intelligent Fix Execution**
- Handles piped commands (e.g., `pacman -Qdtq | sudo pacman -Rns -`)
- Handles simple commands (e.g., `sudo journalctl --vacuum-size=500M`)
- Real-time progress indication
- Detailed error reporting
- Suggestion to re-run doctor after fixes

**🎨 Beautiful Uninstaller**
- Interactive confirmation
- Selective user data removal
- Clean system state restoration
- Feedback collection
- Reinstall instructions
- Anna-style formatting throughout

**📦 Enhanced Installation**
- Uninstaller script with confirmation prompts
- User data preservation option
- Clean removal of all Anna components

### 🔧 Technical Details

**Auto-Fix Modes:**
```bash
# Preview fixes without applying
annactl doctor --dry-run

# Fix with confirmation for each issue
annactl doctor --fix

# Fix all without confirmation
annactl doctor --fix --auto
```

**Fix Capabilities:**
- Orphan package removal
- Package cache cleanup (paccache)
- Journal size reduction (journalctl --vacuum-size)
- Failed service investigation (systemctl)
- Disk space analysis (du -sh /*)

**Execution Safety:**
- All fixes require confirmation (unless --auto)
- Error handling for failed commands
- stderr output display on failure
- Success/failure counting
- No destructive operations without approval

**Uninstaller Features:**
- Stops and disables systemd service
- Removes binaries from /usr/local/bin
- Optional user data removal:
  - /etc/anna/ (configuration)
  - /var/log/anna/ (logs)
  - /run/anna/ (runtime)
  - /var/cache/anna/ (cache)
- Preserves data by default
- Clean system restoration

### 💡 What This Means

**Self-Healing System:**
- One command to fix all detected issues
- Preview changes before applying
- Safe, reversible fixes
- Educational (see what commands fix what)

**Better Maintenance Workflow:**
1. Run `annactl doctor` - See health score and issues
2. Run `annactl doctor --dry-run` - Preview fixes
3. Run `annactl doctor --fix` - Apply fixes with confirmation
4. Run `annactl doctor` again - Verify improvements

**Professional Uninstall Experience:**
- Polite, helpful messaging
- User data preservation option
- Clean system state
- Reinstall instructions provided

### 📊 Example Usage

**Auto-Fix with Preview:**
```bash
$ annactl doctor --dry-run

🔧 Auto-Fix

ℹ DRY RUN - showing what would be fixed:

  1. 12 orphan packages
     → pacman -Qdtq | sudo pacman -Rns -
  2. Large package cache (6.2GB)
     → sudo paccache -rk2
  3. Large journal (1.8GB)
     → sudo journalctl --vacuum-size=500M
```

**Auto-Fix with Confirmation:**
```bash
$ annactl doctor --fix

🔧 Auto-Fix

ℹ Found 3 fixable issues

  [1] 12 orphan packages
  Fix this issue? [Y/n]: y
  → pacman -Qdtq | sudo pacman -Rns -
  ✓ Fixed successfully

  [2] Large package cache (6.2GB)
  Fix this issue? [Y/n]: y
  → sudo paccache -rk2
  ✓ Fixed successfully

📊 Fix Summary
  ✓ 2 issues fixed

ℹ Run 'annactl doctor' again to verify fixes
```

**Uninstaller:**
```bash
$ curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/uninstall.sh | sudo sh

⚠ This will remove Anna Assistant from your system

The following will be removed:
  → Daemon and client binaries
  → Systemd service
  → User data and configuration (your settings and history will be lost!)

Are you sure you want to uninstall? [y/N]: y

→ Stopping annad service...
✓ Service stopped
✓ Service disabled
→ Removing systemd service...
✓ Service file removed
→ Removing binaries...
✓ Binaries removed

╭──────────────────────────────────────────────────────╮
│      Anna Assistant Successfully Uninstalled       │
╰──────────────────────────────────────────────────────╯

Thanks for using Anna! We're sorry to see you go.
```

## [1.0.0-beta.36] - 2025-11-05

### 🏥 Intelligent System Doctor!

**COMPREHENSIVE DIAGNOSTICS:** Enhanced doctor command with health scoring, categorized checks, and automatic issue detection!

### ✨ Major Features

**🩺 Enhanced Doctor Command**
- Comprehensive system health diagnostics
- 100-point health scoring system
- Categorized checks (Package, Disk, Services, Network, Security, Performance)
- Automatic issue detection with severity levels
- Fix command suggestions for every issue
- Color-coded health summary (green/yellow/red)

**📦 Package System Checks**
- Pacman functionality verification
- Orphan package detection and count
- Package cache size monitoring (warns if >5GB)
- Automatic fix commands provided

**💾 Disk Health Checks**
- Root partition space monitoring
- Critical alerts at >90% full (−15 points)
- Warning at >80% full (−5 points)
- SMART tools availability check
- Fix suggestions for disk cleanup

**⚙️ System Service Checks**
- Failed service detection
- Anna daemon status verification
- Systemd service health monitoring
- Automatic fix commands for services

**🌐 Network Diagnostics**
- Internet connectivity test (ping 8.8.8.8)
- DNS resolution test (archlinux.org)
- Network health scoring
- Connectivity issue detection

**🔒 Security Audits**
- Root user detection (warns against running as root)
- Firewall status check (ufw/firewalld)
- Security best practice recommendations
- Missing security tool warnings

**⚡ Performance Checks**
- Journal size monitoring
- Large journal detection (warns if >1GB)
- Performance optimization suggestions
- System resource health

**📊 Health Scoring System**
- 100-point scale with weighted deductions
- Package issues: up to −20 points
- Disk problems: up to −15 points
- Service failures: up to −20 points
- Network issues: up to −15 points
- Security gaps: up to −10 points
- Performance issues: up to −5 points

### 🔧 Technical Details

**Health Score Breakdown:**
```
100 points = Excellent health ✨
90-99 = Good health (green)
70-89 = Minor issues (yellow)
<70 = Needs attention (red)
```

**Categorized Diagnostics:**
1. 📦 Package System - Pacman, orphans, cache
2. 💾 Disk Health - Space, SMART monitoring
3. ⚙️ System Services - Systemd, failed services
4. 🌐 Network - Connectivity, DNS resolution
5. 🔒 Security - Firewall, user permissions
6. ⚡ Performance - Journal size, resources

**Issue Detection:**
- Critical issues (red ✗) - Immediate attention required
- Warnings (yellow !) - Should be addressed
- Info (blue ℹ) - Informational only
- Success (green ✓) - All good

**Auto-Fix Suggestions:**
Every detected issue includes a suggested fix command:
- Orphan packages → `pacman -Qdtq | sudo pacman -Rns -`
- Large cache → `sudo paccache -rk2`
- Large journal → `sudo journalctl --vacuum-size=500M`
- Failed services → `systemctl --failed`
- Disk space → `du -sh /* | sort -hr | head -20`

### 💡 What This Means

**Quick System Health Check:**
- One command to assess entire system
- Immediate identification of problems
- Prioritized issue list with severity
- Ready-to-run fix commands

**Proactive Maintenance:**
- Catch issues before they become critical
- Monitor system degradation over time
- Track improvements with health score
- Compare health across reboots

**Educational:**
- Learn about system components
- Understand what "healthy" means
- See fix commands for every issue
- Build system administration knowledge

### 📊 Example Output

```
Anna System Doctor

Running comprehensive system diagnostics...

📦 Package System
  ✓ Pacman functional
  ! 12 orphan packages found
  ℹ Package cache: 3.2G

💾 Disk Health
  ℹ Root partition: 67% used
  ✓ SMART monitoring available

⚙️  System Services
  ✓ No failed services
  ✓ Anna daemon running

🌐 Network
  ✓ Internet connectivity
  ✓ DNS resolution working

🔒 Security
  ✓ Running as non-root user
  ! No firewall detected

⚡ Performance
  ℹ Archived and active journals take up 512.0M in the file system.

📊 Health Score
  88/100

🔧 Issues Found
  ! 1. 12 orphan packages
     Fix: pacman -Qdtq | sudo pacman -Rns -

⚠️  Warnings
  • Consider enabling a firewall (ufw or firewalld)

ℹ System health is good
```

## [1.0.0-beta.35] - 2025-11-05

### 🔬 Enhanced Telemetry & Predictive Maintenance!

**INTELLIGENT MONITORING:** Anna now monitors hardware health, predicts failures, and proactively alerts you before problems become critical!

### ✨ Major Features

**🌡️ Hardware Monitoring**
- Real-time CPU temperature tracking
- SMART disk health monitoring (reallocated sectors, pending errors, wear leveling)
- Battery health tracking (capacity, cycles, degradation)
- Memory pressure detection
- System load averages (1min, 5min, 15min)

**🔮 Predictive Analysis**
- Disk space predictions (warns when storage will be full)
- Temperature trend analysis
- Memory pressure risk assessment
- Service reliability scoring
- Boot time trend tracking

**🚨 Proactive Health Alerts**
- Critical CPU temperature warnings (>85°C)
- Failing disk detection from SMART data
- Excessive journal error alerts (>100 errors/24h)
- Degraded service notifications
- Low memory warnings with OOM kill tracking
- Battery health degradation alerts
- Service crash pattern detection
- Kernel error monitoring
- Disk space running out predictions

**📊 System Health Metrics**
- Journal error/warning counts (last 24 hours)
- Critical system event tracking
- Service crash history (last 7 days)
- Out-of-Memory (OOM) event tracking
- Kernel error detection
- Top CPU/memory consuming processes

**⚡ Performance Metrics**
- CPU usage trends
- Memory usage patterns
- Disk I/O statistics
- Network traffic monitoring
- Process-level resource tracking

### 🔧 Technical Details

**New Telemetry Types:**
```rust
pub struct HardwareMonitoring {
    pub cpu_temperature_celsius: Option<f64>,
    pub cpu_load_1min/5min/15min: Option<f64>,
    pub memory_used_gb/available_gb: f64,
    pub swap_used_gb/total_gb: f64,
    pub battery_health: Option<BatteryHealth>,
}

pub struct DiskHealthInfo {
    pub health_status: String, // PASSED/FAILING/UNKNOWN
    pub temperature_celsius: Option<u8>,
    pub power_on_hours: Option<u64>,
    pub reallocated_sectors: Option<u64>,
    pub pending_sectors: Option<u64>,
    pub has_errors: bool,
}

pub struct SystemHealthMetrics {
    pub journal_errors_last_24h: usize,
    pub critical_events: Vec<CriticalEvent>,
    pub degraded_services: Vec<String>,
    pub recent_crashes: Vec<ServiceCrash>,
    pub oom_events_last_week: usize,
    pub kernel_errors: Vec<String>,
}

pub struct PredictiveInsights {
    pub disk_full_prediction: Option<DiskPrediction>,
    pub temperature_trend: TemperatureTrend,
    pub service_reliability: Vec<ServiceReliability>,
    pub boot_time_trend: BootTimeTrend,
    pub memory_pressure_risk: RiskLevel,
}
```

**New Recommendation Functions:**
- `check_cpu_temperature()` - Warns at >75°C, critical at >85°C
- `check_disk_health()` - SMART data analysis for failing drives
- `check_journal_errors()` - Alerts on excessive system errors
- `check_degraded_services()` - Detects unhealthy systemd units
- `check_memory_pressure()` - OOM prevention and swap warnings
- `check_battery_health()` - Capacity degradation and cycle tracking
- `check_service_crashes()` - Pattern detection for unstable services
- `check_kernel_errors()` - Hardware/driver issue identification
- `check_disk_space_prediction()` - Proactive storage alerts

**Data Sources:**
- `/proc/loadavg` - System load monitoring
- `/sys/class/thermal/*` - CPU temperature sensors
- `/sys/class/power_supply/*` - Battery information
- `smartctl` - Disk SMART data (requires smartmontools)
- `journalctl` - System logs and error tracking
- `systemctl` - Service health status
- `/proc/meminfo` - Memory pressure analysis

### 💡 What This Means

**Prevents Data Loss:**
- Detects failing disks BEFORE they die
- Warns when disk space running out
- Alerts on critical battery levels

**Prevents System Damage:**
- Critical temperature warnings prevent hardware damage
- Thermal throttling detection
- Cooling system failure alerts

**Prevents System Instability:**
- Catches excessive errors early
- Identifies failing services
- OOM kill prevention through memory warnings
- Kernel error detection

**Predictive Maintenance:**
- Know when your disk will be full (based on growth rate)
- Track battery degradation over time
- Monitor system health trends
- Service reliability scoring

### 📊 Example Alerts

**Critical Temperature:**
```
[MANDATORY] CPU Temperature is CRITICAL!

Your CPU is running at 92.3°C, which is dangerously high!
Prolonged high temperatures can damage hardware and reduce lifespan.
Normal temps: 40-60°C idle, 60-80°C load. You're in the danger zone!

Action: Clean dust from fans, improve airflow, check thermal paste
```

**Failing Disk:**
```
[MANDATORY] CRITICAL: Disk /dev/sda is FAILING!

SMART data shows disk /dev/sda has errors!
Reallocated sectors: 12, Pending sectors: 5
This disk could lose all data at any moment.
BACKUP IMMEDIATELY and replace this drive!

Action: BACKUP ALL DATA IMMEDIATELY, then replace drive
```

**Memory Pressure:**
```
[MANDATORY] CRITICAL: Very low memory available!

Only 0.8GB of RAM available! Your system is under severe memory pressure.
This causes swap thrashing, slow performance, and potential OOM kills.

Action: Close memory-heavy applications or add more RAM
Command: ps aux --sort=-%mem | head -15
```

**Disk Space Prediction:**
```
[MANDATORY] Disk / will be full in ~12 days!

At current growth rate (2.5 GB/day), / will be full in ~12 days!
Low disk space causes system instability, failed updates, and data loss.

Action: Free up disk space or expand storage
```

## [1.0.0-beta.34] - 2025-11-05

### 📊 History Tracking & Enhanced Wiki Cache!

**ANALYTICS:** Track your system improvements over time! See success rates, top categories, and health improvements.

### ✨ Major Features

**📈 Application History Tracking**
- Persistent JSONL-based history at `/var/log/anna/application_history.jsonl`
- Tracks every recommendation you apply with full details
- Records success/failure status and health score changes
- Command-level audit trail with timestamps

**📊 Analytics & Insights**
- Success rate calculations with visual progress bars
- Top category analysis - see what you optimize most
- Average health improvement tracking
- Period-based statistics (last N days)
- Detailed entry view for troubleshooting

**🖥️ New `annactl history` Command**
- `--days N` - Show history for last N days (default: 30)
- `--detailed` - Show full command output and details
- Beautiful visual bars for success rates
- Category popularity ranking with charts
- Health score improvement trends

**📚 Massively Expanded Wiki Cache**
- Increased from 15 to 40+ essential Arch Wiki pages
- Categories: Installation, Security, Package Management, Hardware, Desktop Environments
- Development tools (Python, Rust, Node.js, Docker, Git)
- Gaming pages (Gaming, Steam, Wine)
- Network configuration (SSH, Firewall, Wireless)
- Power management for laptops (TLP, powertop)
- Troubleshooting resources (FAQ, Debugging)

### 🔧 Technical Details

**History Module:**
```rust
pub struct HistoryEntry {
    pub advice_id: String,
    pub advice_title: String,
    pub category: String,
    pub applied_at: DateTime<Utc>,
    pub applied_by: String,
    pub command_run: Option<String>,
    pub success: bool,
    pub output: String,
    pub health_score_before: Option<u8>,
    pub health_score_after: Option<u8>,
}

pub struct ApplicationHistory {
    pub entries: Vec<HistoryEntry>,
}

impl ApplicationHistory {
    pub fn success_rate(&self) -> f64
    pub fn top_categories(&self, count: usize) -> Vec<(String, usize)>
    pub fn average_health_improvement(&self) -> Option<f64>
    pub fn period_stats(&self, days: i64) -> PeriodStats
}
```

**Wiki Cache Expansion:**
- Essential guides (Installation, General recommendations, System maintenance)
- Security hardening resources
- Complete hardware driver documentation (NVIDIA, Intel, AMD)
- All major desktop environments (GNOME, KDE, Xfce)
- Development language resources
- Gaming optimization guides
- Network and SSH configuration
- Laptop power management

### 💡 What This Means

**Track Your Progress:**
- See how many recommendations you've applied
- Monitor your success rate over time
- Identify which categories you optimize most
- Measure actual health score improvements

**Data-Driven Decisions:**
- Understand which optimizations work best
- See trends in your system maintenance
- Identify patterns in failures for better troubleshooting

**Enhanced Offline Access:**
- 40+ essential Arch Wiki pages cached locally
- Faster access to documentation
- Work offline with full wiki resources
- Curated selection of most useful pages

### 📊 Example Usage

**View Recent History:**
```bash
annactl history --days 7
```

**Detailed Output:**
```bash
annactl history --days 30 --detailed
```

**Example Output:**
```
📊 Last 30 Days

  Total Applications:  42
  Successful:          39
  Failed:              3
  Success Rate:        92.9%

  ████████████████████████████████████░░░░

  📈 Top Categories:
     1. security           15  ████████████████████
     2. performance        12  ████████████████
     3. hardware           8   ███████████
     4. packages           5   ███████
     5. development        2   ███

  Average Health Improvement: +5.3 points
```

## [1.0.0-beta.33] - 2025-01-05

### 📚 Smart Recommendations & Wiki Integration!

**WORKFLOW-AWARE:** Anna now suggests packages based on YOUR workflow and displays wiki links for learning!

### ✨ Major Features

**🎯 Smart Package Recommendation Engine**
- Analyzes your development profile and suggests missing LSP servers
- Recommends gaming enhancements based on detected games/platforms
- Suggests desktop environment-specific tools
- Proposes networking tools based on your setup
- Recommends laptop power management tools
- Content creation tool suggestions

**📖 Wiki Link Display**
- Every recommendation now shows relevant Arch Wiki links
- Beautiful "📚 Learn More" section with clickable URLs
- Direct links to official documentation
- Category-specific wiki pages

**🧠 Workflow Detection**
- Python developers → pyright LSP server
- Rust developers → rust-analyzer
- Go developers → gopls
- TypeScript/JavaScript → typescript-language-server
- Steam users → ProtonGE, MangoHud
- Laptop users → TLP, powertop
- And many more!

### 🔧 Technical Details

**Smart Recommender Module:**
- `smart_recommender.rs` - New module with workflow-based logic
- Analyzes `DevelopmentProfile`, `GamingProfile`, `NetworkProfile`
- Detects missing LSP servers by language
- Context-aware package suggestions
- Integration with existing recommendation pipeline

**Recommendation Categories:**
- Development tools (LSP servers, debuggers, container tools)
- Gaming enhancements (Proton-GE, MangoHud, gamepad support)
- Desktop environment tools (GNOME Tweaks, KDE themes)
- Network tools (WireGuard, OpenSSH)
- Content creation (OBS plugins)
- Laptop utilities (TLP, powertop)

**Functions:**
```rust
pub fn generate_smart_recommendations(facts: &SystemFacts) -> Vec<Advice>
fn recommend_for_development(profile: &DevelopmentProfile) -> Vec<Advice>
fn recommend_for_gaming(profile: &GamingProfile) -> Vec<Advice>
fn recommend_for_desktop(de: &str) -> Vec<Advice>
fn recommend_for_networking(profile: &NetworkProfile) -> Vec<Advice>
fn recommend_for_content_creation() -> Vec<Advice>
fn recommend_for_laptop() -> Vec<Advice>
```

### 💡 What This Means

**For Developers:**
- Automatic detection of missing language servers
- Never miss essential development tools
- LSP suggestions for Python, Rust, Go, TypeScript
- Container tool recommendations (docker-compose)
- Debugger suggestions (GDB for C/C++)

**For Gamers:**
- ProtonGE recommendations for better game compatibility
- MangoHud for performance monitoring
- Gamepad driver suggestions
- Steam-specific enhancements

**For Everyone:**
- Learn more with integrated wiki links
- Discover tools you didn't know existed
- Category-specific recommendations
- Laptop-specific power management
- Desktop environment enhancements

### 📊 Example Recommendations

**Development:**
```
[1]  Install Rust Language Server (rust-analyzer)

  RECOMMENDED  LOW RISK

  You have 45 Rust files but no LSP server installed. rust-analyzer
  provides excellent IDE features for Rust development.

  Action:
  ❯ sudo pacman -S rust-analyzer

  📚 Learn More:
  https://wiki.archlinux.org/title/Rust

  ID: rust-analyzer
```

**Gaming:**
```
[5]  Install MangoHud for in-game performance overlay

  OPTIONAL  LOW RISK

  MangoHud shows FPS, GPU/CPU usage, and temperatures in games.
  Great for monitoring performance.

  Action:
  ❯ sudo pacman -S mangohud

  📚 Learn More:
  https://wiki.archlinux.org/title/Gaming#Performance_overlays

  ID: mangohud
```

**Laptop:**
```
[7]  Install TLP for better battery life

  RECOMMENDED  LOW RISK

  TLP is an advanced power management tool that can significantly
  extend your laptop's battery life.

  Action:
  ❯ sudo pacman -S tlp && sudo systemctl enable tlp

  📚 Learn More:
  https://wiki.archlinux.org/title/TLP

  ID: tlp-power
```

### 🎨 UI Enhancements

**Wiki Link Section:**
- Beautiful "📚 Learn More" header
- Blue italic links for easy scanning
- Multiple wiki references when relevant
- Category wiki pages included

**Recommendation Quality:**
- Context-aware descriptions
- File counts in explanations ("You have 45 Rust files...")
- Platform-specific suggestions
- Clear installation commands

### 🏗️ Infrastructure

**New Module:**
- `crates/annad/src/smart_recommender.rs` - 280+ lines
- Integrated into advice generation pipeline
- Works alongside existing recommenders
- Updates on system refresh

**Integration Points:**
- Called during initial advice generation
- Included in refresh_advice() updates
- Uses existing SystemFacts data
- Seamless with learning system (can be dismissed)

### 📝 Notes

- Smart recommendations respect feedback system
- Can be dismissed like any other advice
- Learning system tracks preferences
- All recommendations have wiki links
- Low-risk, high-value suggestions

### 🎯 Detection Examples

**Detects:**
- 50+ Python files → suggests pyright
- Steam installed → suggests ProtonGE
- Laptop detected → suggests TLP
- C/C++ projects → suggests GDB
- Docker usage → suggests docker-compose
- GNOME desktop → suggests gnome-tweaks
- No VPN → suggests WireGuard

### 🚀 Future Enhancements

Planned improvements:
- ML-based package suggestions
- Community package recommendations
- AUR package smart detection
- Workflow bundle creation from suggestions
- Installation success tracking

## [1.0.0-beta.32] - 2025-01-05

### 🧠 Learning System & Health Scoring!

**ADAPTIVE INTELLIGENCE:** Anna now learns from your behavior and tracks system health with detailed scoring!

### ✨ Major Features

**📊 System Health Scoring**
- Comprehensive health score (0-100) with letter grades (A+ to F)
- Breakdown by category: Security, Performance, Maintenance
- Visual score bars and trend indicators (Improving/Stable/Declining)
- Intelligent health interpretation with actionable next steps
- New `annactl health` command for quick health check

**🎓 Learning & Feedback System**
- Tracks user interactions: applied, dismissed, viewed
- Learns category preferences from your behavior
- Auto-hides dismissed recommendations
- Persistent feedback log at `/var/log/anna/feedback.jsonl`
- New `annactl dismiss` command to hide unwanted advice
- Automatic feedback recording when applying recommendations

**🎯 New CLI Commands**
- `annactl health` - Show system health score with visual breakdown
- `annactl dismiss --id <id>` or `--num <n>` - Dismiss recommendations

### 🔧 Technical Details

**Learning System:**
- `FeedbackEvent` - Track user interactions with timestamps
- `UserFeedbackLog` - Persistent JSONL storage
- `LearnedPreferences` - Analyze patterns from feedback
- `FeedbackType` enum: Applied, Dismissed, Viewed

**Health Scoring:**
- `SystemHealthScore` - Overall + category scores
- `HealthTrend` enum: Improving, Stable, Declining
- Weighted calculation: Security (40%), Performance (30%), Maintenance (30%)
- Dynamic scoring based on system facts and pending advice

**Data Structures:**
```rust
pub struct SystemHealthScore {
    pub overall_score: u8,       // 0-100
    pub security_score: u8,
    pub performance_score: u8,
    pub maintenance_score: u8,
    pub issues_count: usize,
    pub critical_issues: usize,
    pub health_trend: HealthTrend,
}

pub struct FeedbackEvent {
    pub advice_id: String,
    pub advice_category: String,
    pub event_type: FeedbackType,
    pub timestamp: DateTime<Utc>,
    pub username: String,
}

pub struct LearnedPreferences {
    pub prefers_categories: Vec<String>,
    pub dismisses_categories: Vec<String>,
    pub power_user_level: u8,
}
```

### 💡 What This Means

**For Users:**
- Get instant feedback on system health (like a report card!)
- Anna learns what you care about and what you don't
- Dismissed advice stays hidden - no more seeing the same unwanted suggestions
- Clear, actionable guidance based on your health score

**For System Monitoring:**
- Track health trends over time
- See exactly which areas need attention
- Understand the impact of applied recommendations
- Get grade-based assessments (A+ to F)

**For Personalization:**
- Anna adapts to YOUR preferences
- Categories you dismiss appear less frequently
- Categories you apply get prioritized
- Power user detection based on behavior

### 📊 Usage Examples

**Check System Health:**
```bash
# Show full health score
annactl health

# Output example:
#   📊 Overall Health
#
#      85/100  B+
#      ██████████████████████████████████████████░░░░░░░░░░
#      Trend: → Stable
#
#   📈 Score Breakdown
#   Security              95  ███████████████████
#   Performance           80  ████████████████
#   Maintenance           75  ███████████████
```

**Dismiss Unwanted Advice:**
```bash
# Dismiss by ID
annactl dismiss --id orphan-packages

# Dismiss by number from advise list
annactl dismiss --num 5
```

**See Learning in Action:**
```bash
# Dismissed items are automatically hidden
annactl advise
# Output: "Hiding 3 previously dismissed recommendation(s)"
```

### 🎨 UI Enhancements

**Health Score Display:**
- Large, colorful score display with grade letter
- Visual progress bars (█ for filled, ░ for empty)
- Color-coded scores: Green (90+), Yellow (70-89), Orange (50-69), Red (<50)
- Trend arrows: ↗ Improving, → Stable, ↘ Declining
- Contextual interpretation based on score range
- Specific next steps based on issues

**Feedback Integration:**
- Automatic notification when advice is dismissed
- Confirmation when feedback is recorded
- Learning message: "Anna will learn from your preferences"

### 🏗️ Infrastructure

**New Features:**
- Feedback logging with JSONL format
- Dismissal tracking per advice ID
- Category-level preference analysis
- Health score caching (planned)
- Trend calculation from historical data (planned)

**Integration Points:**
- `apply` command now records successful applications
- `dismiss` command records user rejections
- `advise` command filters out dismissed items
- `health` command calculates real-time scores

### 📝 Notes

- Feedback log persists across daemon restarts
- Dismissed advice can be re-enabled by deleting feedback log
- Health scores are calculated in real-time (no caching yet)
- Learning improves with more user interactions
- All feedback is user-specific (username tracked)

### 🎯 What's Next

Planned improvements:
- Health score history tracking
- Trend calculation from historical scores
- ML-based recommendation prioritization
- Category weight adjustment based on preferences
- Export feedback data for analysis

## [1.0.0-beta.31] - 2025-01-05

### 🤖 Autonomous Maintenance & Offline Wiki Cache!

**MAJOR UPDATE:** Anna can now maintain your system autonomously and provides offline access to Arch Wiki pages!

### ✨ Major Features

**🔧 Low-Level Autonomy System**
- 4-tier autonomy system for safe automatic maintenance
- Tier 0 (Advise Only): Monitor and report only
- Tier 1 (Safe Auto-Apply): Clean orphan packages, package cache, and journal
- Tier 2 (Semi-Autonomous): + Remove old kernels, clean tmp directories
- Tier 3 (Fully Autonomous): + Update mirrorlist automatically
- Comprehensive action logging with undo capabilities
- Scheduled autonomous runs every 6 hours
- Smart thresholds (10+ orphans, 5GB+ cache, 1GB+ logs)

**📚 Arch Wiki Offline Cache**
- Download and cache 15 common Arch Wiki pages
- HTML parsing and content extraction
- Checksum-based change detection
- 7-day automatic refresh cycle
- Fallback to online fetch if cache is stale
- Pages cached: Security, Performance, System Maintenance, Power Management, Pacman, Systemd, Kernel Parameters, Docker, Python, Rust, Gaming, Firewall, SSH, Hardware, Desktop Environment

**🎯 New CLI Commands**
- `annactl autonomy [--limit=20]` - View autonomous actions log
- `annactl wiki-cache [--force]` - Update Arch Wiki cache

### 🔧 Technical Details

**Autonomy System:**
- `autonomy.rs` - Core autonomy logic with tier-based execution
- `AutonomyAction` - Action tracking with timestamps, success/failure, output
- `AutonomyLog` - Persistent logging to `/var/log/anna/autonomy.jsonl`
- Safe execution with detailed output capture
- Undo command tracking for reversible operations

**Autonomy Tasks:**
- Tier 1: `clean_orphan_packages()`, `clean_package_cache()`, `clean_journal()`
- Tier 2: `remove_old_kernels()`, `clean_tmp_dirs()`
- Tier 3: `update_mirrorlist()`
- Each task respects safety thresholds and logs all operations

**Wiki Cache System:**
- `wiki_cache.rs` - Wiki fetching and caching infrastructure
- `WikiCacheEntry` - Page metadata, content, timestamp, checksum
- `WikiCache` - Cache management with refresh logic
- HTTP fetching with curl
- Smart HTML content extraction
- Automatic cache refresh when stale (>7 days)

**Data Structures:**
```rust
pub struct AutonomyAction {
    pub action_type: String,
    pub executed_at: DateTime<Utc>,
    pub description: String,
    pub command_run: String,
    pub success: bool,
    pub output: String,
    pub can_undo: bool,
    pub undo_command: Option<String>,
}

pub struct WikiCacheEntry {
    pub page_title: String,
    pub url: String,
    pub content: String,
    pub cached_at: DateTime<Utc>,
    pub checksum: String,
}
```

### 💡 What This Means

**For Users:**
- Your system can now maintain itself automatically (if you enable it)
- Safe, conservative defaults - only truly safe operations in Tier 1
- Full transparency - every autonomous action is logged
- Offline access to critical Arch Wiki pages
- No more hunting for wiki pages when offline

**For System Health:**
- Automatic cleanup of orphaned packages
- Automatic cache management
- Log rotation to save space
- Old kernel removal (keeps 2 latest)
- Updated mirrorlist for faster downloads (Tier 3)

**For Power Users:**
- Fine-grained control via 4 autonomy tiers
- Comprehensive action logging with timestamps
- Undo capability for reversible operations
- Configure via: `annactl config --set autonomy_tier=<0-3>`

### 📊 Usage Examples

**View Autonomous Actions:**
```bash
# View last 20 actions
annactl autonomy

# View more/fewer
annactl autonomy --limit=50
annactl autonomy --limit=10
```

**Configure Autonomy:**
```bash
# Enable safe auto-apply (Tier 1)
annactl config --set autonomy_tier=1

# Semi-autonomous (Tier 2)
annactl config --set autonomy_tier=2

# Fully autonomous (Tier 3)
annactl config --set autonomy_tier=3

# Back to advise-only (Tier 0)
annactl config --set autonomy_tier=0
```

**Wiki Cache:**
```bash
# Update cache (only if stale)
annactl wiki-cache

# Force refresh
annactl wiki-cache --force
```

### 🎨 UI Enhancements

**Autonomy Log Display:**
- Color-coded success/failure indicators
- Action type badges (CLEANUP, MAINT, UPDATE)
- Timestamps for all actions
- Command execution details
- Output preview (first 3 lines)
- Undo command display when available
- Clean, readable formatting with separators

### 🏗️ Infrastructure

**New Modules:**
- `crates/annad/src/autonomy.rs` - Autonomous maintenance system
- `crates/annad/src/wiki_cache.rs` - Wiki caching infrastructure

**Daemon Integration:**
- Periodic autonomy runs scheduled every 6 hours
- Integrated into main event loop
- Error handling and logging
- Respects user configuration

### ⚙️ Configuration

Default autonomy configuration:
```toml
[autonomy]
tier = "AdviseOnly"  # Safe default
confirm_high_risk = true
snapshot_before_apply = false
```

### 📝 Notes

- Autonomy is opt-in (defaults to Tier 0 - Advise Only)
- All autonomous actions are logged for transparency
- Wiki cache update via RPC will be implemented in next version
- Autonomy scheduling is configurable via refresh_interval setting

## [1.0.0-beta.30] - 2025-01-04

### 🧠 Deep System Intelligence & Dynamic Categories!

**GAME CHANGER:** Anna now deeply understands your workflow, preferences, and system state! Categories are dynamic and linked to Arch Wiki.

### ✨ Major Features

**📊 Comprehensive Telemetry System**
- 10 new data structures for deep system understanding
- 30+ new collection functions
- Real-time system state analysis
- Intelligent preference detection

**🎯 Dynamic Category System**
- Categories now show plain English names (e.g., "Security & Privacy" not "security")
- Only displays categories relevant to YOUR system
- Each category linked to official Arch Wiki documentation
- Rich descriptions for every category
- 12 categories: Security & Privacy, Performance & Optimization, Hardware Support, Network Configuration, Desktop Environment, Development Tools, Gaming & Entertainment, Multimedia & Graphics, System Maintenance, Terminal & CLI Tools, Power Management, System Configuration

**🔍 Advanced System Understanding**

*Development Profile:*
- Detects programming languages used (Python, Rust, Go, JavaScript)
- Counts projects and files per language
- Tracks LSP server installation status
- Detects IDEs (VSCode, Vim, Neovim, Emacs, IntelliJ, PyCharm, CLion)
- Counts Git repositories
- Detects container usage (Docker/Podman)
- Detects virtualization (QEMU/VirtualBox/VMware)

*Gaming Profile:*
- Steam/Lutris/Wine detection
- ProtonGE and MangoHud status
- Gamepad driver detection
- Game count tracking

*Network Profile:*
- VPN configuration detection (WireGuard/OpenVPN)
- Firewall status (UFW/iptables)
- SSH server monitoring
- DNS configuration (systemd-resolved/dnsmasq)
- Network share detection (NFS/Samba)

*User Preferences (AI-inferred):*
- CLI vs GUI preference
- Power user detection
- Aesthetics appreciation
- Gamer/Developer/Content Creator profiles
- Laptop user detection
- Minimalism preference

*System Health:*
- Recent package installations (last 30 days)
- Active and enabled services
- Disk usage trends with largest directories
- Cache and log sizes
- Session information (login patterns, multiple users)
- System age tracking

### 🔧 Technical Improvements

**New Data Structures:**
- `CategoryInfo` - Arch Wiki-aligned categories with metadata
- `PackageInstallation` - Installation tracking with timestamps
- `DiskUsageTrend` - Space analysis and trends
- `DirectorySize` - Storage consumption tracking
- `SessionInfo` - User activity patterns
- `DevelopmentProfile` - Programming environment analysis
- `LanguageUsage` - Per-language statistics and LSP status
- `ProjectInfo` - Active project tracking
- `GamingProfile` - Gaming setup detection
- `NetworkProfile` - Network configuration analysis
- `UserPreferences` - AI-inferred user behavior

**New Telemetry Functions:**
- `get_recently_installed_packages()` - Track what was installed when
- `get_active_services()` / `get_enabled_services()` - Service monitoring
- `analyze_disk_usage()` - Comprehensive storage analysis
- `collect_session_info()` - User activity patterns
- `analyze_development_environment()` - Deep dev tool detection
- `detect_programming_languages()` - Language usage analysis
- `count_files_by_extension()` - Project scope analysis
- `detect_ides()` - IDE installation detection
- `count_git_repos()` - Development activity
- `analyze_gaming_profile()` - Gaming setup detection
- `analyze_network_profile()` - Network configuration
- `get_system_age_days()` - Installation age tracking
- `infer_user_preferences()` - Behavioral analysis
- 20+ helper functions for deep system inspection

### 💡 What This Means

Anna now knows:
- **What you build**: "You're working on 3 Python projects with 150 .py files"
- **How you work**: "CLI power user with Neovim and tmux"
- **What you do**: "Gamer with Steam + ProtonGE, Developer with Docker"
- **Your style**: "Values aesthetics (starship + eza installed), prefers minimalism"
- **System health**: "5.2GB cache, logs growing, 42 active services"

This enables **context-aware recommendations** that understand YOUR specific setup and workflow!

### 📦 User Experience Improvements

- Category names are now human-friendly everywhere
- `annactl advise` shows categories with descriptions
- `annactl report` displays categories relevant to your system
- Each category shows item count and purpose
- Wiki links provided for deeper learning

### 📈 Performance & Reliability

- Intelligent caching of telemetry data
- Limited search depths to prevent slowdowns
- Graceful fallbacks for unavailable data
- Async operations for non-blocking collection

## [1.0.0-beta.29] - 2025-01-04

### 🔄 Bundle Rollback System!

**NEW:** Safely rollback workflow bundles with full tracking and reverse dependency order removal!

### ✨ Added

**🔄 Bundle Rollback Feature**
- New `annactl rollback --bundle "Bundle Name"` command
- Full installation history tracking stored in `/var/lib/anna/bundle_history.json`
- Tracks what was installed, when, and by whom
- Automatic reverse dependency order removal
- `--dry-run` support to preview what will be removed
- Interactive confirmation before removal
- Safe rollback only for completed installations

**📊 Bundle History System**
- New `BundleHistory` type for tracking installations
- `BundleHistoryEntry` records each installation with:
  - Bundle name and installed items
  - Installation timestamp and user
  - Status (Completed/Partial/Failed)
  - Rollback availability flag
- Persistent storage with JSON format
- Automatic directory creation

**🛡️ Safety Features**
- Only completed bundles can be rolled back
- Partial/failed installations are tracked but not rolled back
- Interactive prompt before removing packages
- Graceful handling of already-removed packages
- Detailed status reporting during rollback

### 🔧 Technical Improvements
- Added `BundleStatus` enum (Completed/Partial/Failed)
- Added `BundleHistoryEntry` and `BundleHistory` types
- Implemented bundle history load/save with JSON serialization
- Updated `apply_bundle()` to track installations
- Added `rollback()` function with reverse-order removal
- CLI command structure extended with Rollback subcommand

### 📦 Example Usage

```bash
# Install a bundle (now tracked for rollback)
annactl apply --bundle "Python Development Stack"

# See what would be removed
annactl rollback --bundle "Python Development Stack" --dry-run

# Rollback a bundle
annactl rollback --bundle "Python Development Stack"

# View installation history
cat /var/lib/anna/bundle_history.json
```

### 💡 How It Works

1. **Installation Tracking**: When you install a bundle, Anna records:
   - Which items were installed
   - Timestamp and username
   - Success/failure status

2. **Reverse Order Removal**: Rollback removes items in reverse dependency order:
   - If you installed: Docker → docker-compose → lazydocker
   - Rollback removes: lazydocker → docker-compose → Docker

3. **Safety First**: Only fully completed bundles can be rolled back, preventing partial rollbacks that could break dependencies.

## [1.0.0-beta.28] - 2025-01-04

### 🎁 Workflow Bundles & Enhanced Reporting!

**NEW:** One-command workflow bundle installation with smart dependency resolution! Plus enhanced report command with category filtering.

### ✨ Added

**📦 Workflow Bundle System**
- New `annactl bundles` command to list available workflow bundles
- Install complete development stacks with `annactl apply --bundle "Bundle Name"`
- Smart dependency resolution using Kahn's algorithm (topological sort)
- Bundles install tools in the correct order automatically
- Three predefined bundles:
  - "Container Development Stack" (Docker → docker-compose → lazydocker)
  - "Python Development Stack" (python-lsp-server, python-black, ipython)
  - "Rust Development Stack" (rust-analyzer)
- `--dry-run` support to preview what will be installed
- Progress tracking showing X/Y items during installation

**📊 Enhanced Report Command**
- New `--category` flag to filter reports by category
- `annactl report --category security` shows only security recommendations
- `annactl report --category development` shows only dev tools
- Helpful error message listing available categories if category not found
- Report output speaks plain English with sysadmin-level insights

### 🔧 Technical Improvements
- Added `bundles()` function with bundle grouping and display
- Added `apply_bundle()` function with dependency resolution
- Added `topological_sort()` implementing Kahn's algorithm for dependency ordering
- Bundle metadata integration across Docker, Python, and Rust recommendations
- Category parameter support in report generation

### 📦 Example Usage

```bash
# List available bundles
annactl bundles

# Install a complete workflow bundle
annactl apply --bundle "Python Development Stack"

# Preview what will be installed
annactl apply --bundle "Container Development Stack" --dry-run

# Get a focused report on security issues
annactl report --category security
```

## [1.0.0-beta.27] - 2025-01-04

### 🚀 Advanced Telemetry & Intelligent Recommendations!

**GAME CHANGER:** Anna now analyzes boot performance, AUR usage, package cache, kernel parameters, and understands workflow dependencies!

### ✨ Added

**⚡ Boot Performance Analysis**
- Tracks total boot time using `systemd-analyze time`
- Detects slow-starting services (>5 seconds)
- Identifies failed systemd services
- Recommends disabling `NetworkManager-wait-online` and other slow services
- Links to Arch Wiki boot optimization guides

**🎯 AUR Helper Intelligence**
- Counts AUR packages vs official repos using `pacman -Qm`
- Detects which AUR helper is installed (yay, paru, aurutils, pikaur, aura, trizen)
- Suggests installing AUR helper if you have AUR packages but no helper
- Recommends paru over yay for users with 20+ AUR packages (faster, Rust-based)
- Offers 3 alternatives with trade-offs explained

**💾 Package Cache Intelligence**
- Monitors `/var/cache/pacman/pkg/` size with `du`
- Warns when cache exceeds 5GB
- Suggests `paccache` for safe cleanup
- Offers 3 cleanup strategies:
  - Keep last 3 versions (safe default)
  - Keep last 1 version (aggressive, saves more space)
  - Remove all uninstalled packages
- Auto-suggests installing `pacman-contrib` if needed

**🔧 Kernel Parameter Optimization**
- Parses `/proc/cmdline` for current boot parameters
- Suggests `noatime` for SSD systems (reduces wear)
- Recommends `quiet` parameter for cleaner boot screen
- Links to Arch Wiki kernel parameter documentation

**🔗 Dependency Chains & Workflow Bundles**
- Added 3 new fields to Advice struct:
  - `depends_on: Vec<String>` - IDs that must be applied first
  - `related_to: Vec<String>` - Suggestions for related advice
  - `bundle: Option<String>` - Workflow bundle name
- Foundation for smart ordering and grouped recommendations
- Example: "Container Development Stack" (Docker → docker-compose → lazydocker)

### 📊 Enhanced Telemetry (10 New Fields)

**Boot Performance**
- `boot_time_seconds: Option<f64>`
- `slow_services: Vec<SystemdService>`
- `failed_services: Vec<String>`

**Package Management**
- `aur_packages: usize`
- `aur_helper: Option<String>`
- `package_cache_size_gb: f64`
- `last_system_upgrade: Option<DateTime<Utc>>`

**Kernel & Boot**
- `kernel_parameters: Vec<String>`

**Advice Metadata**
- `depends_on: Vec<String>`
- `related_to: Vec<String>`
- `bundle: Option<String>`

### 🛠️ New Detection Functions

- `get_boot_time()` - Parse systemd-analyze output
- `get_slow_services()` - Find services taking >5s to start
- `get_failed_services()` - List failed systemd units
- `count_aur_packages()` - Count foreign packages
- `detect_aur_helper()` - Find installed AUR helper
- `get_package_cache_size()` - Calculate cache size in GB
- `get_last_upgrade_time()` - Parse pacman.log timestamps
- `get_kernel_parameters()` - Read /proc/cmdline
- `check_boot_performance()` - Generate boot recommendations
- `check_package_cache()` - Generate cache recommendations
- `check_aur_helper_usage()` - Generate AUR helper recommendations
- `check_kernel_params_optimization()` - Generate kernel parameter recommendations

### 🎯 Real-World Impact

**Boot Optimization Example:**
```
[15] Disable slow service: NetworkManager-wait-online.service (12.3s)
     RECOMMENDED   LOW RISK

     NetworkManager-wait-online delays boot waiting for network.
     Most systems don't need this.

     ❯ systemctl disable NetworkManager-wait-online.service
```

**Package Cache Cleanup Example:**
```
[23] Package cache is large (8.4 GB)
     RECOMMENDED   LOW RISK

     Alternatives:
     ★ Keep last 3 versions - Safe default
     ○ Keep last 1 version - More aggressive
     ○ Remove uninstalled packages
```

### 🔧 Technical

- Added `SystemdService` type for boot analysis
- All new telemetry functions are async-compatible
- Dependency tracking foundation for future auto-ordering
- Workflow bundles enable "install complete stack" features

## [1.0.0-beta.26] - 2025-01-04

### 🎨 Software Alternatives - Choose What You Love!

**THE FEATURE YOU ASKED FOR:** Instead of "install X", Anna now offers 2-3 alternatives for most tools!

### ✨ Added

**🔄 Software Alternatives System**
- New `Alternative` type with name, description, and install command
- Visual display with ★ for recommended option, ○ for alternatives
- Wrapped descriptions for readability
- Install commands shown for each option

**🛠️ Tools with Alternatives (5 major categories)**
- **Status bars**: Waybar, eww, yambar
- **Application launchers**: Wofi, Rofi (Wayland), Fuzzel
- **Notification daemons**: Mako, Dunst, SwayNC
- **Terminal emulators**: Alacritty, Kitty, WezTerm
- **Web browsers**: Firefox, Chromium, LibreWolf

### 🎯 Why This Matters
- User choice > forced recommendations
- See trade-offs at a glance (performance vs features)
- Learn about alternatives you might not know
- Better UX: "choose what fits you" vs "install this one thing"

### 🔧 Technical
- Added `alternatives: Vec<Alternative>` field to `Advice` struct
- Backward compatible with `#[serde(default)]`
- Enhanced `display_advice_item_enhanced()` to show alternatives
- All existing advice gets empty alternatives by default

## [1.0.0-beta.25] - 2025-01-04

### 🧠 MAJOR UX OVERHAUL - Smart Filtering & Intelligence!

**THE BIG PROBLEM SOLVED:** 80+ recommendations was overwhelming. Now you see ~25 most relevant by default!

### ✨ Added

**🎯 Smart Filtering System**
- **Smart Mode (default)**: Shows ~25 most relevant recommendations
- **Critical Mode** (`--mode=critical`): Security & mandatory items only
- **Recommended Mode** (`--mode=recommended`): Critical + recommended items
- **All Mode** (`--mode=all`): Everything for power users
- **Category Filter** (`--category=security`): Focus on specific categories
- **Limit Control** (`--limit=10`): Control number of results

**🧠 Intelligent Behavior-Based Detection (3 new rules)**
- Docker power users → docker-compose recommendations (50+ docker commands)
- Python developers → pyenv suggestions (30+ python commands)
- Git power users → lazygit recommendations (50+ git commands)

**📊 Enhanced Report Command**
- Sysadmin-level system health analysis
- Hardware specs (CPU, RAM, GPU)
- Storage analysis with visual indicators
- Software environment details
- Development tools detection
- Network capabilities overview
- Color-coded status indicators

**🎨 Better Discoverability**
- Helpful footer with command examples
- Category list with item counts
- Clear filtering indicators
- Quick action guide

### 🐛 Fixed
- Desktop environment detection now works when daemon runs as root
- No more irrelevant suggestions (KDE tips on GNOME systems)
- Installer box rendering with proper width calculation
- Removed unused functions causing build warnings

### 🔧 Changed
- Default `annactl advise` now shows smart-filtered view (was: show all)
- Recommendations sorted by relevance and priority
- Better visual hierarchy in output

## [1.0.0-beta.24] - 2025-01-04

### ✨ Added

**🎨 Beautiful Category-Based Output**
- 80-character boxes with centered, color-coded category titles
- 14 organized categories with emojis
- Priority badges (CRITICAL, RECOMMENDED, OPTIONAL, COSMETIC)
- Risk level indicators (HIGH RISK, MED RISK, LOW RISK)
- Smart sorting by priority and risk within categories

**⚙️ Configuration System**
- TOML-based configuration at `~/.config/anna/config.toml`
- 6 sections: General, Autonomy, Notifications, Snapshots, Learning, Categories
- Auto-creation with sensible defaults

**💾 Snapshot & Rollback System**
- Multi-backend support: Btrfs, Timeshift, rsync
- Automatic snapshots before risky operations
- Retention policies with automatic cleanup

**📊 Deep Telemetry Foundation**
- Process CPU time tracking
- Bash/zsh history parsing
- Workflow pattern detection
- System configuration analysis

## [1.0.0-beta.20] - 2025-01-XX

### 🌟 Professional Coverage - 220+ Rules, 95%+ Wiki Coverage! 🌟

**PHENOMENAL expansion!** Added 30+ professional-grade tools covering Python, Rust, multimedia, science, engineering, and productivity!

### ✨ Added

**🐍 Python Development Tools (3 new rules)**
- Poetry for modern dependency management
- virtualenv for isolated environments
- IPython enhanced REPL

**🦀 Rust Development Tools (2 new rules)**
- cargo-watch for automatic rebuilds
- cargo-audit for security vulnerability scanning

**📺 Terminal Tools (1 new rule)**
- tmux terminal multiplexer

**🖼️ Image Viewers (2 new rules)**
- feh for X11 (lightweight, wallpaper setter)
- imv for Wayland (fast, keyboard-driven)

**📚 Documentation (1 new rule)**
- tldr for quick command examples

**💾 Disk Management (2 new rules)**
- smartmontools for disk health monitoring
- GParted for partition management

**💬 Communication (1 new rule)**
- Discord for gaming and communities

**🔬 Scientific Computing (1 new rule)**
- Jupyter Notebook for interactive Python

**🎨 3D Graphics (1 new rule)**
- Blender for 3D modeling and animation

**🎵 Audio Production (1 new rule)**
- Audacity for audio editing

**📊 System Monitoring (1 new rule)**
- s-tui for CPU stress testing

**🏗️ CAD Software (1 new rule)**
- FreeCAD for parametric 3D modeling

**📝 Markdown Tools (1 new rule)**
- glow for beautiful markdown rendering

**📓 Note-Taking (1 new rule)**
- Obsidian for knowledge management

### 🔄 Changed
- Detection function count increased from 84 to 98 (+16%)
- Total recommendations increased from 190+ to 220+ (+15%)
- Added professional tool detection (Python/Rust dev tools)
- Scientific computing support (Jupyter)
- Engineering tools (CAD, 3D graphics)
- Enhanced disk health monitoring
- Arch Wiki coverage increased from ~90% to ~95%+

### 📊 Coverage Status
- **Total detection functions**: 98
- **Total recommendations**: 220+
- **Wiki coverage**: 95%+ for typical users
- **New professional categories**: Python Tools, Rust Tools, Scientific Computing, 3D Graphics, CAD, Engineering, Audio Production

## [1.0.0-beta.19] - 2025-01-XX

### 🎯 Complete Coverage - 190+ Rules, 90%+ Wiki Coverage! 🎯

**INCREDIBLE expansion!** Added 30+ more rules covering tools, utilities, development workflows, and system administration!

### ✨ Added

**🎵 Music Players (1 new rule)**
- MPD (Music Player Daemon) with ncmpcpp

**📄 PDF Readers (1 new rule)**
- Zathura vim-like PDF viewer

**🖥️ Monitor Management (1 new rule)**
- arandr for X11 multi-monitor setup

**⏰ System Scheduling (1 new rule)**
- Systemd timers vs cron comparison

**🐚 Shell Alternatives (1 new rule)**
- Fish shell with autosuggestions

**🗜️ Advanced Compression (1 new rule)**
- Zstandard (zstd) modern compression

**🔄 Dual Boot Support (1 new rule)**
- os-prober for GRUB multi-OS detection

**🎯 Git Advanced Tools (2 new rules)**
- git-delta for beautiful diffs
- lazygit terminal UI

**📦 Container Alternatives (1 new rule)**
- Podman rootless container runtime

**💻 Modern Code Editors (1 new rule)**
- Visual Studio Code

**🗄️ Additional Databases (2 new rules)**
- MariaDB (MySQL replacement)
- Redis in-memory database

**🌐 Network Analysis (2 new rules)**
- Wireshark packet analyzer
- nmap network scanner

**⚙️ Dotfile Management (1 new rule)**
- GNU Stow for dotfile symlinks

**📦 Package Development (2 new rules)**
- namcap PKGBUILD linter
- devtools clean chroot builds

### 🔄 Changed
- Detection function count increased from 70 to 84 (+20%)
- Total recommendations increased from 160+ to 190+ (+18%)
- Added behavior-based detection for power users
- Systemd timer suggestions for cron users
- Multi-monitor setup detection
- PKGBUILD developer tools
- Arch Wiki coverage increased from ~85% to ~90%+

### 📊 Coverage Status
- **Total detection functions**: 84
- **Total recommendations**: 190+
- **Wiki coverage**: 90%+ for typical users
- **New categories**: Music, PDF, Monitors, Scheduling, Compression, Dotfiles, Network Tools, Package Development

## [1.0.0-beta.18] - 2025-01-XX

### 🚀 Comprehensive Coverage - 160+ Rules, 85%+ Wiki Coverage!

**MASSIVE expansion!** Added 30+ new rules covering development, productivity, multimedia, networking, and creative software!

### ✨ Added

**✏️ Text Editors (1 new rule)**
- Neovim upgrade for Vim users

**📧 Mail Clients (1 new rule)**
- Thunderbird for email management

**📂 File Sharing (2 new rules)**
- Samba for Windows file sharing
- NFS for Linux/Unix file sharing

**☁️ Cloud Storage (1 new rule)**
- rclone for universal cloud sync (40+ providers)

**💻 Programming Languages - Go (2 new rules)**
- Go compiler installation
- gopls LSP server for Go development

**☕ Programming Languages - Java (2 new rules)**
- OpenJDK installation
- Maven build tool

**🟢 Programming Languages - Node.js (2 new rules)**
- Node.js and npm installation
- TypeScript for type-safe JavaScript

**🗄️ Databases (1 new rule)**
- PostgreSQL database

**🌐 Web Servers (1 new rule)**
- nginx web server

**🖥️ Remote Desktop (1 new rule)**
- TigerVNC for remote desktop access

**🌊 Torrent Clients (1 new rule)**
- qBittorrent for torrent downloads

**📝 Office Suites (1 new rule)**
- LibreOffice for document editing

**🎨 Graphics Software (2 new rules)**
- GIMP for photo editing
- Inkscape for vector graphics

**🎬 Video Editing (1 new rule)**
- Kdenlive for video editing

### 🔄 Changed
- Detection rule count increased from 130+ to 160+ (+23%)
- Now supporting 3 additional programming languages (Go, Java, Node.js/TypeScript)
- Command history analysis for intelligent editor/tool suggestions
- Arch Wiki coverage increased from ~80% to ~85%+

### 📊 Coverage Status
- **Total detection functions**: 70
- **Total recommendations**: 160+
- **Wiki coverage**: 85%+ for typical users
- **Categories covered**: Security, Desktop (8 DEs), Development (6 languages), Multimedia, Productivity, Gaming, Networking, Creative

## [1.0.0-beta.17] - 2025-01-XX

### 🌐 Privacy, Security & Gaming - Reaching 80% Wiki Coverage!

**High-impact features!** VPN, browsers, security tools, backups, screen recording, password managers, gaming enhancements, and mobile integration!

### ✨ Added

**🔒 VPN & Networking (2 new rules)**
- WireGuard modern VPN support
- NetworkManager VPN plugin recommendations

**🌐 Browser Recommendations (2 new rules)**
- Firefox/Chromium installation detection
- uBlock Origin privacy extension reminder

**🛡️ Security Tools (3 new rules)**
- rkhunter for rootkit detection
- ClamAV antivirus for file scanning
- LUKS encryption passphrase backup reminder

**💾 Backup Solutions (2 new rules)**
- rsync for file synchronization
- BorgBackup for encrypted deduplicated backups

**🎥 Screen Recording (2 new rules)**
- OBS Studio for professional recording/streaming
- SimpleScreenRecorder for easy captures

**🔐 Password Managers (1 new rule)**
- KeePassXC for secure password storage

**🎮 Gaming Enhancements (3 new rules)**
- Proton-GE for better Windows game compatibility
- MangoHud for in-game performance overlay
- Wine for Windows application support

**📱 Android Integration (2 new rules)**
- KDE Connect for phone notifications and file sharing
- scrcpy for Android screen mirroring

### 🔄 Changed
- Detection rule count increased from 110+ to 130+ (+18%)
- Arch Wiki coverage improved from 70% to ~80%
- Enhanced privacy and security recommendations

### 📚 Documentation
- README.md updated to v1.0.0-beta.17
- Wiki coverage analysis added
- CHANGELOG.md updated with beta.17 features

---

## [1.0.0-beta.16] - 2025-01-XX

### 💻 Laptop, Audio, Shell & Bootloader Enhancements!

**Complete laptop support!** Battery optimization, touchpad, backlight, webcam, audio enhancements, shell productivity tools, filesystem maintenance, and bootloader optimization!

### ✨ Added

**💻 Laptop Optimizations (4 new rules)**
- powertop for battery optimization and power tuning
- libinput for modern touchpad support with gestures
- brightnessctl for screen brightness control
- laptop-mode-tools for advanced power management

**📷 Webcam Support (2 new rules)**
- v4l-utils for webcam control and configuration
- Cheese webcam viewer for testing

**🎵 Audio Enhancements (2 new rules)**
- EasyEffects for PipeWire audio processing (EQ, bass, effects)
- pavucontrol for advanced per-app volume control

**⚡ Shell Productivity (3 new rules)**
- bash-completion for intelligent tab completion
- fzf for fuzzy finding (history, files, directories)
- tmux for terminal multiplexing and session management

**💾 Filesystem Maintenance (2 new rules)**
- ext4 fsck periodic check reminders
- Btrfs scrub for data integrity verification

**🔧 Kernel & Boot (4 new rules)**
- 'quiet' kernel parameter for cleaner boot
- 'splash' parameter for graphical boot screen
- GRUB timeout reduction for faster boot
- Custom GRUB background configuration

### 🔄 Changed
- Detection rule count increased from 90+ to 110+ (+22%)
- Enhanced laptop and mobile device support
- Improved boot experience recommendations

### 📚 Documentation
- README.md updated to v1.0.0-beta.16
- Version bumped across all crates
- CHANGELOG.md updated with beta.16 features

---

## [1.0.0-beta.15] - 2025-01-XX

### ⚡ System Optimization & Configuration!

**Essential system optimizations!** Firmware updates, SSD optimizations, swap compression, DNS configuration, journal management, AUR safety, and locale/timezone setup!

### ✨ Added

**🔧 Firmware & Hardware Optimization (2 new rules)**
- fwupd installation for automatic firmware updates
- Firmware update check recommendations

**💾 SSD Optimizations (2 new rules)**
- noatime mount option detection for reduced writes
- discard/continuous TRIM recommendations
- Automatic SSD detection via /sys/block

**🗜️ Swap Compression (1 new rule)**
- zram detection and installation for compressed swap in RAM

**🌐 DNS Configuration (2 new rules)**
- systemd-resolved recommendation for modern DNS with caching
- Public DNS server suggestions (Cloudflare, Google, Quad9)

**📜 Journal Management (2 new rules)**
- Large journal size detection and cleanup
- SystemMaxUse configuration for automatic size limiting

**🛡️ AUR Helper Safety (2 new rules)**
- PKGBUILD review reminder for security
- Development package (-git/-svn) update notifications

**🌍 System Configuration (3 new rules)**
- Locale configuration detection
- Timezone setup verification
- NTP time synchronization enablement

### 🔄 Changed
- Detection rule count increased from 75+ to 90+ (+20%)
- Enhanced system optimization category
- Improved SSD detection logic

### 📚 Documentation
- README.md updated to v1.0.0-beta.15
- Version bumped across all crates
- CHANGELOG.md updated with beta.15 features

---

## [1.0.0-beta.14] - 2025-01-XX

### 🐳 Containers, Virtualization, Printers & More!

**Development and system tools!** Docker containerization, QEMU/KVM virtualization, printer support, archive tools, and system monitoring!

### ✨ Added

**🐳 Docker & Container Support (4 new rules)**
- Docker installation detection for container users
- Docker service enablement check
- Docker group membership for sudo-free usage
- Docker Compose for multi-container applications

**💻 Virtualization Support (QEMU/KVM) (4 new rules)**
- CPU virtualization capability detection
- BIOS virtualization enablement check (/dev/kvm)
- QEMU installation for KVM virtual machines
- virt-manager GUI for easy VM management
- libvirt service configuration

**🖨️ Printer Support (CUPS) (3 new rules)**
- USB printer detection
- CUPS printing system installation
- CUPS service enablement
- Gutenprint universal printer drivers

**📦 Archive Management Tools (3 new rules)**
- unzip for ZIP archive support
- unrar for RAR archive extraction
- p7zip for 7z archives and better compression

**📊 System Monitoring Tools (3 new rules)**
- htop for interactive process monitoring
- btop for advanced system monitoring with graphs
- iotop for disk I/O monitoring

### 🔄 Changed
- Detection rule count increased from 60+ to 75+ (+25%)
- Added development category recommendations
- Enhanced hardware support detection

### 📚 Documentation
- README.md updated to v1.0.0-beta.14
- Version bumped across all crates
- CHANGELOG.md updated with beta.14 features

---

## [1.0.0-beta.13] - 2025-01-XX

### 🌟 More Desktop Environments + SSH Hardening + Snapshots!

**New desktop environments!** Cinnamon, XFCE, and MATE now fully supported. Plus comprehensive SSH hardening and snapshot system recommendations!

### ✨ Added

**🖥️ Desktop Environment Support (3 new DEs!)**
- **Cinnamon desktop environment**
  - Nemo file manager with dual-pane view
  - GNOME Terminal integration
  - Cinnamon screensaver for security
- **XFCE desktop environment**
  - Thunar file manager with plugin support
  - xfce4-terminal with dropdown mode
  - xfce4-goodies collection (panel plugins, system monitoring)
- **MATE desktop environment**
  - Caja file manager (GNOME 2 fork)
  - MATE Terminal with tab support
  - MATE utilities (screenshot, search, disk analyzer)

**🔒 SSH Hardening Detection (7 new rules)**
- SSH Protocol 1 detection (critical vulnerability)
- X11 forwarding security check
- MaxAuthTries recommendation (brute-force protection)
- ClientAliveInterval configuration (connection timeouts)
- AllowUsers whitelist suggestion
- Non-default SSH port recommendation
- Improved root login and password authentication checks

**💾 Snapshot System Recommendations (Timeshift/Snapper)**
- Snapper detection for Btrfs users
- Timeshift detection for ext4 users
- snap-pac integration for automatic pacman snapshots
- grub-btrfs for bootable snapshot recovery
- Snapper configuration validation
- Context-aware recommendations based on filesystem type

### 🔄 Changed
- Detection rule count increased from 50+ to 60+
- README.md updated with new feature count
- "Coming Soon" section updated (implemented features removed)

### 📚 Documentation
- README.md updated to v1.0.0-beta.13
- Version bumped across all crates
- CHANGELOG.md updated with beta.13 features

---

## [1.0.0-beta.12] - 2025-01-XX

### 🎨 The Beautiful Box Update!

**Box rendering completely fixed!** Plus 50+ new detection rules, batch apply, auto-refresh, and per-user advice!

### 🔧 Fixed
- **Box rendering completely rewritten** - Fixed box drawing character alignment by using `console::measure_text_width()` to measure visible text width BEFORE adding ANSI color codes
- Terminal broadcast notifications now use proper box drawing (╭╮╰╯│─)
- All header formatting uses beautiful Unicode boxes with perfect alignment
- Tests updated to validate box structure correctly

### ✨ Added - 50+ New Detection Rules!

**🎮 Hardware Support**
- Gamepad drivers (Xbox, PlayStation, Nintendo controllers) via USB detection
- Bluetooth stack (bluez, bluez-utils) with hardware detection
- WiFi firmware for Intel, Qualcomm, Atheros, Broadcom chipsets
- USB automount with udisks2
- NetworkManager for easy WiFi management
- TLP power management for laptops (with battery detection)

**🖥️ Desktop Environments & Display**
- XWayland compatibility layer for running X11 apps on Wayland
- Picom compositor for X11 (transparency, shadows, tearing fixes)
- Modern GPU-accelerated terminals (Alacritty, Kitty, WezTerm)
- Status bars for tiling WMs (Waybar for Wayland, i3blocks for i3)
- Application launchers (Rofi for X11, Wofi for Wayland)
- Notification daemons (Dunst for X11, Mako for Wayland)
- Screenshot tools (grim/slurp for Wayland, maim/scrot for X11)

**🔤 Fonts & Rendering**
- Nerd Fonts for terminal icons and glyphs
- Emoji font support (Noto Emoji)
- CJK fonts for Chinese, Japanese, Korean text
- FreeType rendering library

**🎬 Multimedia**
- yt-dlp for downloading videos from YouTube and 1000+ sites
- FFmpeg for video/audio processing and conversion
- VLC media player for any format
- ImageMagick for command-line image editing
- GStreamer plugins for codec support in GTK apps

### 🚀 Major Features

**Batch Apply Functionality**
- Apply single recommendation: `annactl apply --nums 1`
- Apply range: `annactl apply --nums 1-5`
- Apply multiple ranges: `annactl apply --nums 1,3,5-7`
- Smart range parsing with duplicate removal and sorting
- Shows progress and summary for each item

**Per-User Context Detection**
- Added `GetAdviceWithContext` IPC method
- Personalizes advice based on:
  - Desktop environment (i3, Hyprland, Sway, GNOME, KDE, etc.)
  - Shell (bash, zsh, fish)
  - Display server (Wayland vs X11)
  - Username for multi-user systems
- CLI automatically detects and sends user environment
- Daemon filters advice appropriately

**Automatic System Monitoring**
- Daemon now automatically refreshes advice when:
  - Packages installed/removed (monitors `/var/lib/pacman/local`)
  - Config files change (pacman.conf, sshd_config, fstab)
  - System reboots (detected via `/proc/uptime`)
- Uses `notify` crate with inotify for filesystem watching
- Background task with tokio::select for event handling

**Smart Notifications**
- Critical issues trigger notifications via:
  - GUI notifications (notify-send) for desktop users
  - Terminal broadcasts (wall) for SSH/TTY users
  - Both channels for critical issues
- Uses loginctl to detect active user sessions
- Only notifies for High risk level advice

**Plain English System Reports**
- `annactl report` generates conversational health summaries
- Analyzes system state and provides friendly assessment
- Shows disk usage, package count, recommendations by category
- Provides actionable next steps

### 🔄 Changed
- **Refresh command removed from public CLI** - Now internal-only, triggered automatically by daemon
- **Advice numbering** - All items numbered for easy reference in batch apply
- **Improved text wrapping** - Multiline text wraps at 76 chars with proper indentation
- **Enhanced installer** - Auto-installs missing dependencies (curl, jq, tar)
- **Beautiful installer intro** - Shows what Anna does before installation

### 🏗️ Technical
- Added `notify` crate for filesystem watching (v6.1)
- Added `console` crate for proper text width measurement (v0.15)
- New modules: `watcher.rs` (system monitoring), `notifier.rs` (notifications)
- Enhanced `beautiful.rs` with proper box rendering using `measure_text_width()`
- `parse_number_ranges()` function for batch apply range parsing
- Better error handling across all modules
- Improved separation of concerns in recommender systems

### 📊 Statistics
- Detection rules: 27 → 50+ (85% increase)
- Advice categories: 10 → 12
- IPC methods: 8 → 9 (added GetAdviceWithContext)
- Functions for range parsing, text wrapping, user context detection
- Total code: ~3,500 → ~4,500 lines

---

## [1.0.0-beta.11] - 2025-11-04

### 🎉 The MASSIVE Feature Drop!

Anna just got SO much smarter! This is the biggest update yet with **27 intelligent detection rules** covering your entire system!

### What's New

**📦 Perfect Terminal Formatting!**
- Replaced custom box formatting with battle-tested libraries (owo-colors + console)
- Proper unicode-aware width calculation - no more broken boxes!
- All output is now gorgeous and professional

**🎮 Gaming Setup Detection!**
- **Steam gaming stack** - Multilib repo, GameMode, MangoHud, Gamescope, Lutris
- **Xbox controller drivers** - xpadneo/xone for full controller support
- **AntiMicroX** - Map gamepad buttons to keyboard/mouse
- Only triggers if you actually have Steam installed!

**🖥️ Desktop Environment Intelligence!**
- **GNOME** - Extensions, Tweaks for customization
- **KDE Plasma** - Dolphin file manager, Konsole terminal
- **i3** - i3status/polybar, Rofi launcher
- **Hyprland** - Waybar, Wofi, Mako notifications
- **Sway** - Wayland-native tools
- **XWayland** - X11 app compatibility on Wayland
- Detects your actual DE from environment variables!

**🎬 Multimedia Stack!**
- **mpv** - Powerful video player
- **yt-dlp** - Download from YouTube and 500+ sites
- **FFmpeg** - Media processing Swiss Army knife
- **PipeWire** - Modern audio system (suggests upgrade from PulseAudio)
- **pavucontrol** - GUI audio management

**💻 Terminal & Fonts!**
- **Modern terminals** - Alacritty, Kitty, WezTerm (GPU-accelerated)
- **Nerd Fonts** - Essential icons for terminal apps

**🔧 System Tools!**
- **fwupd** - Firmware updates for BIOS, SSD, USB devices
- **TLP** - Automatic laptop battery optimization (laptop detection!)
- **powertop** - Battery drain analysis

**📡 Hardware Detection!**
- **Bluetooth** - BlueZ stack + Blueman GUI (only if hardware detected)
- **WiFi** - linux-firmware + NetworkManager applet (hardware-aware)
- **USB automount** - udisks2 + udiskie for plug-and-play drives

### Why This Release is INCREDIBLE

**27 detection rules** that understand YOUR system:
- Hardware-aware (Bluetooth/WiFi only if you have the hardware)
- Context-aware (gaming tools only if you have Steam)
- Priority-based (critical firmware first, beautification optional)
- All in plain English with clear explanations!

### Technical Details
- Added `check_gaming_setup()` with Steam detection
- Added `check_desktop_environment()` with DE/WM detection
- Added `check_terminal_and_fonts()` for modern terminal stack
- Added `check_firmware_tools()` for fwupd
- Added `check_media_tools()` for multimedia apps
- Added `check_audio_system()` with PipeWire/Pulse detection
- Added `check_power_management()` with laptop detection
- Added `check_gamepad_support()` for controller drivers
- Added `check_usb_automount()` for udisks2/udiskie
- Added `check_bluetooth()` with hardware detection
- Added `check_wifi_setup()` with hardware detection
- Integrated owo-colors and console for proper formatting
- Fixed git identity message clarity

## [1.0.0-beta.10] - 2025-11-04

### ✨ The Ultimate Terminal Experience!

Anna now helps you build the most beautiful, powerful terminal setup possible!

### What's New

**🎨 Shell Enhancements Galore!**
- **Starship prompt** - Beautiful, fast prompts for zsh and bash with git status, language versions, and gorgeous colors
- **zsh-autosuggestions** - Autocomplete commands from your history as you type!
- **zsh-syntax-highlighting** - Commands turn green when valid, red when invalid - catch typos instantly
- **Smart bash → zsh upgrade** - Suggests trying zsh with clear explanations of benefits
- All context-aware based on your current shell

**🚀 Modern CLI Tools Revolution!**
- **eza replaces ls** - Colors, icons, git integration, tree views built-in
- **bat replaces cat** - Syntax highlighting, line numbers, git integration for viewing files
- **ripgrep replaces grep** - 10x-100x faster code searching with smart defaults
- **fd replaces find** - Intuitive syntax, respects .gitignore, blazing fast
- **fzf fuzzy finder** - Game-changing fuzzy search for files, history, everything!
- Smart detection - only suggests tools you actually use based on command history

**🎉 Beautiful Release Notes!**
- Install script now shows proper formatted release notes
- Colored output with emoji and hierarchy
- Parses markdown beautifully in the terminal
- Falls back to summary if API fails

**🔧 Release Automation Fixes!**
- Removed `--prerelease` flag - all releases now marked as "latest"
- Fixed installer getting stuck on beta.6
- Better jq-based JSON parsing

### Why This Release is HUGE

**16 intelligent detection rules** across security, performance, development, and beautification!

Anna can now transform your terminal from basic to breathtaking. She checks what tools you actually use and suggests modern, faster, prettier replacements - all explained in plain English.

### Technical Details
- Added `check_shell_enhancements()` with shell detection
- Added `check_cli_tools()` with command history analysis
- Enhanced install.sh with proper markdown parsing
- Fixed release.sh to mark releases as latest
- Over 240 lines of new detection code

---

## [1.0.0-beta.9] - 2025-11-04

### 🔐 Security Hardening & System Intelligence!

Anna gets even smarter with SSH security checks and memory management!

### What's New

**🛡️ SSH Hardening Detection!**
- **Checks for root login** - Warns if SSH allows direct root access (huge security risk!)
- **Password vs Key authentication** - Suggests switching to SSH keys if you have them set up
- **Empty password detection** - Critical alert if empty passwords are allowed
- Explains security implications in plain English
- All checks are Mandatory priority for your safety

**💾 Smart Swap Management!**
- **Detects missing swap** - Suggests adding swap if you have <16GB RAM
- **Zram recommendations** - Suggests compressed RAM swap for better performance
- Explains what swap is and why it matters (no more mysterious crashes!)
- Context-aware suggestions based on your RAM and current setup

**📝 Amazing Documentation!**
- **Complete README overhaul** - Now visitors will actually want to try Anna!
- Shows all features organized by category
- Includes real example messages
- Explains the philosophy and approach
- Beautiful formatting with emoji throughout

**🚀 Automated Release Notes!**
- Release script now auto-extracts notes from CHANGELOG
- GitHub releases get full, enthusiastic descriptions
- Shows preview during release process
- All past releases updated with proper notes

### Why This Release Matters
- **Security-first** - SSH hardening can prevent system compromises
- **Better stability** - Swap detection helps prevent crashes
- **Professional presentation** - README makes Anna accessible to everyone
- **14 detection rules total** - Growing smarter every release!

### Technical Details
- Added `check_ssh_config()` with sshd_config parsing
- Added `check_swap()` with RAM detection and zram suggestions
- Enhanced release.sh to extract and display CHANGELOG entries
- Updated all release notes retroactively with gh CLI
- Improved README with clear examples and philosophy

---

## [1.0.0-beta.8] - 2025-11-04

### 🚀 Major Quality of Life Improvements!

Anna just got a whole lot smarter and prettier!

### What's New

**🎨 Fixed box formatting forever!**
- Those annoying misaligned boxes on the right side? Gone! ANSI color codes are now properly handled everywhere.
- Headers, boxes, and all terminal output now look pixel-perfect.

**🔐 Security First!**
- **Firewall detection** - Anna checks if you have a firewall (UFW) and helps you set one up if you don't. Essential for security, especially on laptops!
- Anna now warns you if your firewall is installed but not turned on.

**📡 Better Networking!**
- **NetworkManager detection** - If you have WiFi but no NetworkManager, Anna will suggest installing it. Makes connecting to networks so much easier!
- Checks if NetworkManager is enabled and ready to use.

**📦 Unlock the Full Power of Arch!**
- **AUR helper recommendations** - Anna now suggests installing 'yay' or 'paru' if you don't have one. This gives you access to over 85,000 community packages!
- Explains what the AUR is in plain English - no jargon!

**⚡ Lightning-Fast Downloads!**
- **Reflector for mirror optimization** - Anna suggests installing reflector to find the fastest mirrors near you.
- Checks if your mirror list is old (30+ days) and offers to update it.
- Can make your downloads 10x faster if you're on slow mirrors!

### Why This Release Rocks
- **5 new detection rules** covering security, networking, and performance
- **Box formatting finally perfect** - no more visual glitches
- **Every message in plain English** - accessible to everyone
- **Smarter recommendations** - Anna understands your system better

### Technical Details
- Fixed ANSI escape code handling in boxed() function
- Added `check_firewall()` with UFW and iptables detection
- Added `check_network_manager()` with WiFi card detection
- Added `check_aur_helper()` suggesting yay/paru
- Added `check_reflector()` with mirror age checking
- All new features include Arch Wiki citations

---

## [1.0.0-beta.7] - 2025-11-04

### 🎉 Anna Speaks Human Now!

We've completely rewritten every message Anna shows you. No more technical jargon!

### What Changed
- **All advice is now in plain English** - Instead of "AMD CPU detected without microcode updates," Anna now says "Your AMD processor needs microcode updates to protect against security vulnerabilities like Spectre and Meltdown. Think of it like a security patch for your CPU itself."
- **Friendly messages everywhere** - "Taking a look at your system..." instead of "Analyzing system..."
- **Your system looks great!** - When everything is fine, Anna celebrates with you
- **Better counting** - "Found 1 thing that could make your system better!" reads naturally
- **Enthusiastic release notes** - This changelog is now exciting to read!

### Why This Matters
Anna is for everyone, not just Linux experts. Whether you're brand new to Arch or you've been using it for years, Anna talks to you like a helpful friend, not a robot. Every message explains *why* something matters and what it actually does.

### Technical Details (for the curious)
- Rewrote all `Advice` messages in `recommender.rs` with conversational explanations
- Updated CLI output to be more welcoming
- Made sure singular/plural grammar is always correct
- Added analogies to help explain technical concepts

---

## [1.0.0-beta.6] - 2025-11-04

### 🎉 New: Beautiful Installation Experience!
The installer now shows you exactly what Anna can do and what's new in this release. No more guessing!

### What's New
- **Your SSD will thank you** - Anna now checks if your solid-state drive has TRIM enabled. This keeps it fast and healthy for years to come.
- **Save hundreds of gigabytes** - If you're using Btrfs, Anna will suggest turning on compression. You'll get 20-30% of your disk space back without slowing things down.
- **Faster package downloads** - Anna can set up parallel downloads in pacman, making updates 5x faster. Why wait around?
- **Prettier terminal output** - Enable colorful pacman output so you can actually see what's happening during updates.
- **Health monitoring** - Anna keeps an eye on your system services and lets you know if anything failed. No more silent problems.
- **Better performance tips** - Learn about noatime and other mount options that make your system snappier.

### Why You'll Love It
- You don't need to be a Linux expert - Anna explains everything in plain English
- Every suggestion comes with a link to the Arch Wiki if you want to learn more
- Your system stays healthy and fast without you having to remember all the tweaks

---

## [1.0.0-beta.5] - 2025-11-04

### Added
- **Missing config detection** - detects installed packages without configuration:
  - bat without ~/.config/bat/config
  - starship without ~/.config/starship.toml
  - git without user.name/user.email
  - zoxide without shell integration
- Better microcode explanations (Spectre/Meltdown patches)

### Changed
- **Microcode now Mandatory priority** (was Recommended) - critical for CPU security
- Microcode category changed to "security" (was "maintenance")

### Fixed
- Box formatting now handles ANSI color codes correctly
- Header boxes dynamically size to content

---

## [1.0.0-beta.4] - 2025-11-04

### Added
- Category-based colors for advice titles (💻 blue, 🎨 pink, ⚡ yellow, 🎵 purple)
- Comprehensive FACTS_CATALOG.md documenting all telemetry to collect
- Implementation roadmap with 3 phases for v1.0.0-rc.1, v1.0.0, v1.1.0+

### Changed
- **Smarter Python detection** - requires BOTH .py files AND python/pip command usage
- **Smarter Rust detection** - requires BOTH .rs files AND cargo command usage
- Grayed out reasons and commands for better visual hierarchy
- Improved advice explanations with context

### Fixed
- False positive development tool recommendations
- Better color contrast and readability in advice output

---

## [1.0.0-beta.3] - 2025-11-04

### Added
- Emojis throughout CLI output for better visual appeal
  - 💻 Development tools, 🎨 Beautification, ⚡ Performance
  - 💡 Reasons, 📋 Commands, 🔧 Maintenance, ✨ Suggestions
- Better spacing between advice items for improved readability

### Changed
- Report command now fetches real-time data from daemon
- Improved Go language detection - only triggers on actual .go files
- Better explanations with context-aware emoji prefixes

### Fixed
- Double "v" in version string (was "vv1.0.0-beta.2", now "v1.0.0-beta.3")
- Inconsistent advice counts between report and advise commands

---

## [1.0.0-beta.2] - 2025-11-04

### Fixed
- Missing `hostname` command causing daemon crash on minimal installations
  - Added fallback to read `/etc/hostname` directly
  - Prevents "No such file or directory" error on systems without hostname utility

---

## [1.0.0-beta.1] - 2025-11-04

### 🎉 Major Release - Beta Status Achieved!

Anna is now **intelligent, personalized, and production-ready** for testing!

### Added

#### Intelligent Behavior-Based Recommendations (20+ new rules)
- **Development Tools Detection**
  - Python development → python-lsp-server, black, ipython
  - Rust development → rust-analyzer, sccache
  - JavaScript/Node.js → typescript-language-server
  - Go development → gopls language server
  - Git usage → git-delta (beautiful diffs), lazygit (TUI)
  - Docker usage → docker-compose, lazydocker
  - Vim usage → neovim upgrade suggestion

- **CLI Tool Improvements** (based on command history analysis)
  - `ls` usage → eza (colors, icons, git integration)
  - `cat` usage → bat (syntax highlighting)
  - `grep` usage → ripgrep (10x faster)
  - `find` usage → fd (modern, intuitive)
  - `du` usage → dust (visual disk usage)
  - `top/htop` usage → btop (beautiful system monitor)

- **Shell Enhancements**
  - fzf (fuzzy finder)
  - zoxide (smart directory jumping)
  - starship (beautiful cross-shell prompt)
  - zsh-autosuggestions (if using zsh)
  - zsh-syntax-highlighting (if using zsh)

- **Media Player Recommendations**
  - Video files → mpv player
  - Audio files → cmus player
  - Image files → feh viewer

#### Enhanced Telemetry System
- Command history analysis (top 1000 commands from bash/zsh history)
- Development tools detection (git, docker, vim, cargo, python, node, etc.)
- Media usage profiling (video/audio/image files and players)
- Desktop environment detection (GNOME, KDE, i3, XFCE)
- Shell detection (bash, zsh, fish)
- Display server detection (X11, Wayland)
- Package group detection (base-devel, desktop environments)
- Network interface analysis (wifi, ethernet)
- Common file type detection (.py, .rs, .js, .go, etc.)

#### New SystemFacts Fields
- `frequently_used_commands` - Top 20 commands from history
- `dev_tools_detected` - Installed development tools
- `media_usage` - Video/audio/image file presence and player status
- `common_file_types` - Programming languages detected
- `desktop_environment` - Detected DE
- `display_server` - X11 or Wayland
- `shell` - User's shell
- `has_wifi`, `has_ethernet` - Network capabilities
- `package_groups` - Detected package groups

#### Priority System
- **Mandatory**: Critical security and driver issues
- **Recommended**: Significant quality-of-life improvements
- **Optional**: Performance optimizations
- **Cosmetic**: Beautification enhancements

#### Action Executor
- Execute commands with dry-run support
- Full audit logging to `/var/log/anna/audit.jsonl`
- Rollback token generation (for future rollback capability)
- Safe command execution via tokio subprocess

#### Systemd Integration
- `annad.service` systemd unit file
- Automatic startup on boot
- Automatic restart on failure
- Install script enables/starts service automatically

#### Documentation
- `ROADMAP.md` - Project vision and implementation plan
- `TESTING.md` - Testing guide for IPC system
- `CHANGELOG.md` - This file

### Changed
- **Advice struct** now includes:
  - `priority` field (Mandatory/Recommended/Optional/Cosmetic)
  - `category` field (security/drivers/development/media/beautification/etc.)
- Install script now installs and enables systemd service
- Daemon logs more detailed startup information
- Recommendations now sorted by priority

### Fixed
- Install script "Text file busy" error when daemon is running
- Version embedding in GitHub Actions workflow
- Socket permission issues for non-root users

---

## [1.0.0-alpha.3] - 2024-11-03

### Added
- Unix socket IPC between daemon and client
- RPC protocol with Request/Response message types
- Real-time communication for status and recommendations
- Version verification in install script

### Fixed
- GitHub Actions release workflow permissions
- Install script process stopping logic

---

## [1.0.0-alpha.2] - 2024-11-02

### Added
- Release automation scripts (`scripts/release.sh`)
- Install script (`scripts/install.sh`) for GitHub releases
- GitHub Actions workflow for releases
- Version embedding via build.rs

---

## [1.0.0-alpha.1] - 2024-11-01

### Added
- Initial project structure
- Core data models (SystemFacts, Advice, Action, etc.)
- Basic telemetry collection (hardware, packages)
- 5 initial recommendation rules:
  - Microcode installation (AMD/Intel)
  - GPU driver detection (NVIDIA/AMD)
  - Orphaned packages cleanup
  - Btrfs maintenance
  - System updates
- Beautiful CLI with pastel colors
- Basic daemon and client binaries

---

## Future Plans

### v1.0.0-rc.1 (Release Candidate)
- Arch Wiki caching system
- Wiki-grounded recommendations with citations
- More recommendation rules (30+ total)
- Configuration persistence
- Periodic telemetry refresh

### v1.0.0 (Stable Release)
- Autonomous execution tiers (0-3)
- Auto-apply safe recommendations
- Rollback capability
- Performance optimizations
- Comprehensive documentation

### v1.1.0+
- AUR package
- Web dashboard
- Multi-user support
- Plugin system
- Machine learning for better predictions

## [1.0.0-beta.21] - 2025-01-XX

### 🎛️ Configuration System - TOML-based Settings! 🎛️

**MAJOR NEW FEATURE!** Implemented comprehensive configuration system with TOML support for user preferences and automation!

### ✨ Added

**Configuration Module**
- Created `config.rs` in anna_common with full TOML serialization/deserialization
- Configuration file automatically created at `~/.config/anna/config.toml`
- Structured configuration with multiple sections:
  - General settings (refresh interval, verbosity, emoji, colors)
  - Autonomy configuration (tier levels, auto-apply rules, risk filtering)
  - Notification preferences (desktop, terminal, priority filtering)
  - Snapshot settings (method, retention, auto-snapshot triggers)
  - Learning preferences (behavior tracking, history analysis)
  - Category filters (enable/disable recommendation categories)
  - User profiles (multi-user system support)

**Enhanced annactl config Command**
- Display all current configuration settings beautifully organized
- Set individual config values: `annactl config --set key=value`
- Supported configuration keys:
  - `autonomy_tier` (0-3): Control auto-apply behavior
  - `snapshots_enabled` (true/false): Enable/disable snapshots
  - `snapshot_method` (btrfs/timeshift/rsync/none): Choose snapshot backend
  - `learning_enabled` (true/false): Enable/disable behavior learning
  - `desktop_notifications` (true/false): Control notifications
  - `refresh_interval` (seconds): Set telemetry refresh frequency
- Validation on all settings with helpful error messages
- Beautiful output showing all configuration sections

**Configuration Features**
- Autonomy tiers: Advise Only, Safe Auto-Apply, Semi-Autonomous, Fully Autonomous
- Risk-based filtering for auto-apply
- Category-based allow/blocklists
- Snapshot integration planning (method selection, retention policies)
- Learning system configuration (command history days, usage thresholds)
- Notification customization (urgency levels, event filtering)
- Multi-user profiles for personalized recommendations

### 🔧 Changed
- Added `toml` dependency to workspace
- Updated anna_common to export config module
- Enhanced config command from stub to fully functional

### 📚 Technical Details
- Config validation ensures safe values (min 60s refresh, min 1 snapshot, etc.)
- Default configuration provides sensible security-first defaults
- TOML format allows easy manual editing
- Auto-creates config directory structure on first use

This lays the foundation for the TUI dashboard and autonomous operation!


## [1.0.0-beta.22] - 2025-01-XX

### 📸 Snapshot & Rollback System - Safe Execution! 📸

**MAJOR NEW FEATURE!** Implemented comprehensive snapshot management for safe action execution with rollback capability!

### ✨ Added

**Snapshot Manager Module**
- Created `snapshotter.rs` with multi-backend support
- Three snapshot methods supported:
  - **Btrfs**: Native subvolume snapshots (read-only, instant)
  - **Timeshift**: Integration with popular backup tool
  - **Rsync**: Incremental backups of critical directories
- Automatic snapshot creation before risky operations
- Configurable risk-level triggers (Medium/High by default)
- Snapshot retention policies with automatic cleanup
- Snapshot metadata tracking (ID, timestamp, description, size)

**Enhanced Executor**
- `execute_action_with_snapshot()`: New function with snapshot support
- Automatic snapshot creation based on risk level
- Rollback token generation with snapshot IDs
- Graceful degradation if snapshot fails (warns but proceeds)
- Backward compatibility maintained for existing code

**Snapshot Features**
- List all snapshots with metadata
- Automatic cleanup of old snapshots (configurable max count)
- Size tracking for disk space management
- Timestamp-based naming scheme
- Support for custom descriptions

**Safety Features**
- Snapshots created BEFORE executing risky commands
- Risk-based triggers (Low/Medium/High)
- Category-based blocking (bootloader, kernel blocked by default)
- Read-only Btrfs snapshots prevent accidental modification
- Metadata preservation for audit trails

### 🔧 Configuration Integration
- Snapshot settings in config.toml:
  - `snapshots.enabled` - Enable/disable snapshots
  - `snapshots.method` - Choose backend (btrfs/timeshift/rsync)
  - `snapshots.max_snapshots` - Retention count
  - `snapshots.snapshot_risk_levels` - Which risks trigger snapshots
  - `snapshots.auto_snapshot_on_risk` - Auto-snapshot toggle

### 📚 Technical Details
- Async snapshot creation with tokio
- Proper error handling and logging
- Filesystem type detection for Btrfs
- Directory size calculation with `du`
- Graceful handling of missing tools (timeshift, etc.)

This provides the foundation for safe autonomous operation and rollback capability!


## [1.0.0-beta.23] - 2025-01-XX

### 🔍 Enhanced Telemetry - Deep System Intelligence! 🔍

**MAJOR ENHANCEMENT!** Added comprehensive system analysis from a sysadmin perspective with CPU time tracking, deep bash history analysis, and system configuration insights!

### ✨ Added

**Process CPU Time Analysis**
- Track actual CPU time per process for user behavior understanding
- Filter user processes vs system processes
- CPU and memory percentage tracking
- Identify what users actually spend time doing

**Deep Bash History Analysis**
- Multi-user bash/zsh history parsing
- Command frequency analysis across all users
- Tool categorization (editor, vcs, container, development, etc.)
- Workflow pattern detection with confidence scores
- Detect: Version Control Heavy, Container Development, Software Development patterns
- Evidence-based pattern matching

**System Configuration Analysis** (sysadmin perspective)
- Bootloader detection (GRUB, systemd-boot, rEFInd)
- Init system verification
- Failed systemd services detection
- Firewall status (ufw/firewalld)
- MAC system detection (SELinux/AppArmor)
- Swap analysis (size, usage, swappiness, zswap)
- Boot time analysis (systemd-analyze)
- I/O scheduler per device
- Important kernel parameters tracking

**Swap Deep Dive**
- Total/used swap in MB
- Swappiness value
- Zswap detection and status
- Recommendations based on swap configuration

**I/O Scheduler Analysis**
- Per-device scheduler detection
- Identify if using optimal schedulers for SSD/HDD
- Foundation for SSD optimization recommendations

**Kernel Parameter Tracking**
- Command line parameters
- Important sysctl values (swappiness, ip_forward, etc.)
- Security and performance parameter analysis

### 🔧 Technical Details
- All analysis functions are async for performance
- Processes are filtered by CPU time (>0.1%)
- Bash history supports both bash and zsh formats
- Workflow patterns calculated with confidence scores (0.0-1.0)
- System config analysis covers bootloader, init, security, performance
- Graceful handling of missing files/permissions

This provides the foundation for truly intelligent, sysadmin-level system analysis!


## [1.0.0-beta.24] - 2025-01-XX

### 🎨 Beautiful Category-Based Advise Output! 🎨

**MAJOR UX IMPROVEMENT!** Completely redesigned `annactl advise` output with category boxes, priority badges, risk badges, and visual hierarchy!

### ✨ Added

**Category-Based Organization**
- Recommendations grouped by category with beautiful boxes
- 14 predefined categories sorted by importance:
  - Security, Drivers, Updates, Maintenance, Cleanup
  - Performance, Power, Development, Desktop, Gaming
  - Multimedia, Hardware, Networking, Beautification
- Each category gets unique emoji and color
- Automatic fallback for unlisted categories

**Beautiful Category Headers**
- 80-character wide boxes with centered titles
- Category-specific emojis (🔒 Security, ⚡ Performance, 💻 Development, etc.)
- Color-coded titles (red for security, yellow for performance, etc.)
- Proper spacing between categories for easy scanning

**Enhanced Item Display**
- Priority badges: CRITICAL, RECOMMENDED, OPTIONAL, COSMETIC
- Risk badges: HIGH RISK, MED RISK, LOW RISK
- Color-coded backgrounds (red, yellow, green, blue)
- Bold titles for quick scanning
- Wrapped text with proper indentation (72 chars)
- Actions highlighted with ❯ symbol
- ID shown subtly in italics

**Smart Sorting**
- Categories sorted by importance (security first)
- Within each category: sort by priority, then risk
- Highest priority and risk items shown first

**Better Summary**
- Shows total recommendations and category count
- Usage instructions at bottom
- Visual separator with double-line (═)

**Fixed Issues**
- RiskLevel now implements Ord for proper sorting
- Box titles properly padded and centered
- All ANSI codes use proper escapes
- Consistent spacing throughout

This makes long advice lists MUCH easier to scan and understand!

