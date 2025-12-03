# Anna Assistant - Release Notes

---

## v0.0.16 - Better Mutation Safety

**Release Date:** 2025-12-03

### Summary

Senior-engineer-level safety for all mutations: preflight checks verify preconditions, dry-run diffs preview changes, post-checks verify expected state, and automatic rollback restores system on failure. Mutation state machine tracks lifecycle from planned through verified_ok or rolled_back.

### Key Features

**Mutation State Machine:**
- `MutationState` enum: `Planned` -> `PreflightOk` -> `Confirmed` -> `Applied` -> `VerifiedOk` | `RolledBack` | `Failed`
- Complete lifecycle tracking for audit trail
- State transitions logged with evidence IDs

**Preflight Checks (`PreflightResult`):**
- File edits: path allowed, file exists/creatable, is text, size under limit, permissions OK, hash recorded, backup available
- Systemd ops: unit exists, current state captured, operation allowed by policy
- Package ops: Arch Linux check, packages exist and not blocked, disk space check
- All checks generate evidence IDs for traceability

**Dry-Run Diff Preview (`DiffPreview`):**
- Line-based diff with context, additions, removals, modifications
- `DiffLine` enum: `Context`, `Added`, `Removed`, `Modified`
- Truncated output (max 20 lines) with overflow indicator
- Shows backup path and rollback command
- Human-readable format: `+1 added, -0 removed, ~2 modified`

**Post-Check Verification (`PostCheckResult`):**
- File edits: verify file exists, readable, contains expected content, hash changed
- Systemd ops: verify active/enabled state matches expectation, no immediate failure (500ms check)
- Package ops: verify package installed/removed
- Evidence IDs for post-state documentation

**Automatic Rollback:**
- `RollbackResult` with success, message, evidence_id, restored_state
- File edits: restore from backup
- Systemd ops: restore prior active/enabled state
- Logged and cited in audit trail
- Reliability score downgrade on rollback

**SafeMutationExecutor:**
- `preflight_file_edit()`, `preflight_systemd()`, `preflight_package()`
- `dry_run_file_edit()` for diff preview without modification
- `postcheck_file_edit()`, `postcheck_systemd()`, `postcheck_package()`
- `rollback_file_edit()`, `rollback_systemd()`
- Evidence ID generation (PRE#####, POST#####, RB#####)

**Junior Enforcement (v0.0.16):**
- New rules 14-19 for mutation safety
- Penalties: -25 for missing preflight, -20 for missing diff, -25 for missing post-check, -30 for missing rollback

### Tests Added

- 21 unit tests in `mutation_safety.rs`:
  - State display, preflight checks, diff generation
  - Edit operations, rollback results
  - Helper functions (arch detection, disk space)

### Files Changed

- `crates/anna_common/src/mutation_safety.rs`: New module (~1300 lines)
- `crates/anna_common/src/lib.rs`: Added module export
- `crates/annactl/src/pipeline.rs`: Updated Junior prompt with mutation safety rules

### Example Transcript: Config Change with Diff Preview

```
[you] to [anna]:
  Add log_level=debug to /etc/myapp/config.toml

[anna] to [translator]:
  Classify this request...

[translator] to [anna]:
  Intent: action_request
  Targets: /etc/myapp/config.toml
  Risk: medium

[anna] to [annad]:
  Run preflight checks for file edit

[annad] to [anna]:
  Preflight checks for edit_file_lines:
    [OK] path_allowed: Path /etc/myapp/config.toml is allowed [POL00012]
    [OK] file_exists: File exists
    [OK] is_text_file: File is text (no binary content)
    [OK] file_size: 256 bytes (limit: 1048576 bytes)
    [OK] permissions: File is readable and writable
    [OK] hash_recorded: Current hash: a1b2c3d4e5f6...
    [OK] backup_available: Backup directory writable
    [OK] operations_valid: 1 edit operation(s) validated

[anna] to [you]:
  Preflight passed. Here's what will change:

  Changes to /etc/myapp/config.toml:
    +1 added, -0 removed, ~0 modified

  +   3 | log_level=debug

  Backup: /var/lib/anna/rollback/config.toml_REQ...
  Rollback: cp /var/lib/anna/rollback/config.toml_REQ... /etc/myapp/config.toml

  To proceed, type exactly: I CONFIRM (medium risk)
```

### Example Transcript: Post-Check Failure with Rollback

```
[anna] to [annad]:
  Execute file edit (confirmed)

[annad] to [anna]:
  File edited. Running post-checks...

  Post-checks for edit_file_lines:
    [OK] file_exists: exists
    [OK] file_readable: readable
    [FAIL] content_check: expected 'log_level=debug', got 'not found'

  Post-check FAILED. Initiating automatic rollback...

[annad] to [anna]:
  Rollback completed [RB00001]:
  Restored /etc/myapp/config.toml from backup
  Reason: post-check failed - expected content not found

[anna] to [you]:
  The change was applied but post-verification failed.
  I've automatically rolled back to the previous state.

  Reliability: 45% (downgraded due to rollback)
```

---

## v0.0.15 - Governance UX Polish

**Release Date:** 2025-12-03

### Summary

Debug level configuration for controlling output verbosity. Unified formatting module for consistent terminal output. Enhanced `annactl status` as single source of truth with comprehensive sections. No regressions - all 500+ tests pass.

### Key Features

**Debug Levels (`config.toml`):**
- Configure in `/etc/anna/config.toml` with `ui.debug_level = 0 | 1 | 2`
- Level 0 (minimal): Only [you]->[anna] and final [anna]->[you], plus confirmations
- Level 1 (normal/default): Dialogues condensed, tool calls summarized, evidence IDs included
- Level 2 (full): Full dialogues between all players, tool execution summaries, Junior critique in full

**UI Configuration (`UiConfig`):**
- `debug_level`: Output verbosity (0=minimal, 1=normal, 2=full)
- `colors_enabled`: Terminal color output (default: true)
- `max_width`: Text wrapping width (0=auto-detect terminal width)
- Helper methods: `is_minimal()`, `is_normal_debug()`, `is_full_debug()`, `effective_width()`

**Unified Formatting (`display_format.rs`):**
- `colors` module: `section_header()`, `label()`, `success()`, `warning()`, `error()`, `evidence_id()`, `reliability()`
- `SectionFormatter`: Consistent status section headers and key-value formatting
- `DialogueFormatter`: Debug level filtering for pipeline output
- Format helpers: `format_bytes()`, `format_timestamp()`, `format_duration_ms()`, `format_percent()`, `format_eta()`
- Text helpers: `wrap_text()`, `indent()`, `format_list()`, `format_summary()`

**Enhanced Status Display (`annactl status`):**
- [VERSION]: Current version and build info
- [INSTALLER REVIEW]: Installation health and component status
- [UPDATES]: Update channel and status
- [MODELS]: LLM model status (Translator, Junior)
- [POLICY]: Policy status, schema version, blocked counts
- [HELPERS]: Installed helper packages
- [ALERTS]: Active alerts by severity
- [LEARNING]: Session memory and recipe counts
- [RECENT ACTIONS]: Audit log summary with last 3 actions
- [STORAGE]: Disk usage for Anna directories

**Pipeline Updates:**
- `dialogue()` function respects debug level configuration
- `dialogue_always()` for confirmations that always show
- Condensed message display at debug level 0

### Tests Added

- 19 tests in `display_format.rs` for formatting utilities
- 5 tests in `config.rs` for UiConfig
- All existing tests pass (500+ total)

### Files Changed

- `crates/anna_common/src/config.rs`: Added UiConfig struct
- `crates/anna_common/src/display_format.rs`: Enhanced formatting module
- `crates/annactl/src/commands/status.rs`: Added new sections
- `crates/annactl/src/pipeline.rs`: Debug level filtering

---

## v0.0.14 - Policy Engine + Security Posture

**Release Date:** 2025-12-03

### Summary

Policy-driven allowlists with no hardcoded deny rules. All major allow/deny decisions flow from editable TOML files in `/etc/anna/policy/`. Structured audit logging with secret redaction. Junior enforcement for policy compliance. Installer review validates policy sanity.

### Key Features

**Policy Engine (`policy.rs` - ~1400 lines):**
- Four policy files in `/etc/anna/policy/`:
  - `capabilities.toml`: Read-only and mutation tool settings
  - `risk.toml`: Risk thresholds, confirmations, reliability requirements
  - `blocked.toml`: Blocked packages, services, paths, commands
  - `helpers.toml`: Helper dependency management and policy
- Hot-reload support via `reload_policy()`
- Global policy cache with RwLock for thread safety
- Policy evidence IDs (POL##### format)

**Policy Checks:**
- `is_path_allowed()`: Check if path can be edited
- `is_package_allowed()`: Check if package can be installed/removed
- `is_service_allowed()`: Check if service can be modified
- `is_systemd_operation_allowed()`: Check systemd operations
- `PolicyCheckResult` with allowed, reason, evidence_id, policy_rule fields

**Blocked Categories (Default):**
- Kernel packages: `linux`, `linux-*`, `kernel*`
- Bootloader packages: `grub`, `systemd-boot`, `refind`, `syslinux`
- Init packages: `systemd`, `openrc`, `runit`
- Critical services: `systemd-*`, `dbus.service`, `NetworkManager.service`
- Protected paths: `/boot/*`, `/etc/shadow`, `/etc/passwd`, `/etc/sudoers`

**Audit Logging (`audit_log.rs` - ~400 lines):**
- Structured JSONL audit trail at `/var/lib/anna/audit/audit.jsonl`
- Entry types: ReadOnlyTool, MutationTool, PolicyCheck, Confirmation, ActionBlocked, Rollback, SecurityEvent
- Secret sanitization: passwords, API keys, tokens, Bearer headers
- Environment variable redaction for sensitive keys
- Log rotation at 10MB with archive directory
- Evidence ID linkage in all entries

**Mutation Tools Integration:**
- `validate_mutation_path()` uses policy for path validation
- `validate_package_policy()` checks blocked packages
- `validate_service_policy()` checks critical services
- `PolicyBlocked` error variant with evidence_id and policy_rule
- Symlink traversal protection (follows symlinks for policy checks)

**Junior Enforcement (v0.0.14):**
- Policy citation requirement for risky operations
- New rules 9-13 in Junior system prompt:
  - Risky operations MUST cite policy evidence [POL#####]
  - Refusals MUST explain which policy rule applied
  - Policy bypass suggestions = DANGEROUS = max penalty
- Penalties: -20 for risky operation without policy citation, -50 for policy bypass

**Installer Review (`installer_review.rs`):**
- `check_policy_sanity()` validates policy files
- Auto-repair creates default policy files if missing
- Policy file parsing and validation
- Evidence IDs for repair tracking

**Configuration Defaults:**
```toml
# /etc/anna/policy/capabilities.toml
[read_only_tools]
enabled = true
max_evidence_bytes = 1048576

[mutation_tools]
enabled = true

[mutation_tools.file_edit]
enabled = true
allowed_paths = ["/etc/", "/home/", "/root/", "/var/lib/anna/", "/tmp/"]
blocked_paths = ["/etc/shadow", "/etc/passwd", "/etc/sudoers"]
max_file_size_bytes = 1048576
text_only = true

[mutation_tools.systemd]
enabled = true
allowed_operations = ["status", "restart", "reload", "enable", "disable", "start", "stop"]
blocked_units = []
protected_units = ["sshd.service", "networkd.service", "systemd-resolved.service"]

[mutation_tools.packages]
enabled = true
max_packages_per_operation = 10
blocked_categories = ["kernel", "bootloader", "init"]
```

### Files Changed

- `crates/anna_common/src/policy.rs` (NEW - ~1400 lines)
- `crates/anna_common/src/audit_log.rs` (NEW - ~400 lines)
- `crates/anna_common/src/mutation_tools.rs` - Policy integration, PolicyBlocked error
- `crates/anna_common/src/installer_review.rs` - Policy sanity checks
- `crates/anna_common/src/lib.rs` - Module declarations and exports
- `crates/annactl/src/pipeline.rs` - Junior policy enforcement rules

### Tests

24 new unit tests covering:
- Policy evidence ID generation
- Pattern matching for packages/services
- Path policy checks (allowed, blocked, boot)
- Package policy checks (allowed, blocked categories, patterns)
- Service policy checks (critical services)
- Systemd operation validation
- Policy validation and defaults
- Confirmation phrase parsing
- Audit entry creation and serialization
- Secret sanitization (passwords, API keys, Bearer tokens)
- Environment variable redaction

### Bug Fixes

- Fixed `parse_tool_plan()` to handle commas inside parentheses correctly
- Fixed `execute_tool_plan()` double-counting evidence IDs
- Updated version test to use pattern matching instead of hardcoded version

---

## v0.0.13 - Conversation Memory + Recipe Evolution

**Release Date:** 2025-12-03

### Summary

Local-first conversation memory and recipe evolution system. Anna remembers past sessions, creates recipes for successful patterns, and allows user introspection via natural language. Privacy-first with summaries by default (no raw transcripts unless configured).

### Key Features

**Session Memory (`/var/lib/anna/memory/`):**
- Compact session records for every REPL/one-shot request
- Fields: request_id, request_text, translator_plan_summary, tools_used, evidence_ids, final_answer_summary, reliability_score, recipe_actions, timestamp
- Privacy default: summaries only (`store_raw` config option for full transcripts)
- Keyword-based search indexing
- Append-only JSONL storage with atomic writes
- Archive/tombstone pattern for "forget" operations
- Evidence IDs: MEM##### format

**Recipe System (`/var/lib/anna/recipes/`):**
- Named intent patterns with conditions
- Tool plan templates (read-only and/or mutation)
- Safety classification and required confirmations
- Precondition validation checks
- Rollback templates for mutations
- Provenance tracking (creator, timestamps)
- Confidence scoring and success/failure counters
- Evidence IDs: RCP##### format

**Recipe Creation Rules:**
- Created when: successful request AND Junior reliability >= 80% AND tools used repeatably
- Below 80% reliability creates "experimental" draft
- Configurable via `memory.min_reliability_for_recipe`

**Recipe Matching:**
- Keyword-based scoring (BM25-style for now)
- Intent type and target matching
- Negative keyword exclusion
- Top matching recipes provided to Translator

**User Introspection (Natural Language):**
- "What have you learned recently?"
- "List recipes" / "Show all recipes"
- "Show recipe for X" / "How do you handle X?"
- "Forget what you learned about X" (requires confirmation)
- "Delete recipe X" (requires confirmation)
- "Show my recent questions"
- "Search memory for X"

**Forget/Delete Operations:**
- Medium risk classification
- Requires explicit confirmation: "I CONFIRM (forget)"
- Reversible via archive (not hard delete)
- All operations logged

**Status Display (`annactl status`):**
- New [LEARNING] section
- Shows: recipe count, last learned time, sessions count
- Top 3 most used recipes with confidence scores
- Memory settings display (store_raw, max_sessions, min_reliability)

**Junior Enforcement (v0.0.13):**
- New learning claim detection in Junior system prompt
- Claims about "learned", "remembered", "knows", "has recipes for" must cite MEM/RCP IDs
- Automatic penalty (-25 per uncited learning claim) in fallback scoring
- Detection of fabricated learning claims in final response

**Configuration (`/etc/anna/config.toml`):**
```toml
[memory]
enabled = true              # Enable memory/learning
store_raw = false          # Store raw transcripts (privacy)
max_sessions = 10000       # Max sessions in index
min_reliability_for_recipe = 80  # Minimum % to create recipe
max_recipes = 500          # Max recipes to store
```

### Files Changed

- `crates/anna_common/src/memory.rs` (NEW - ~650 lines)
- `crates/anna_common/src/recipes.rs` (NEW - ~850 lines)
- `crates/anna_common/src/introspection.rs` (NEW - ~630 lines)
- `crates/anna_common/src/config.rs` - Added MemoryConfig
- `crates/anna_common/src/lib.rs` - Module declarations and exports
- `crates/annactl/src/commands/status.rs` - [LEARNING] section
- `crates/annactl/src/pipeline.rs` - Junior learning claim enforcement

### Tests

28 new unit tests covering:
- Session record creation and serialization
- Memory index management
- Keyword extraction
- Recipe creation and matching
- Recipe score calculation
- Introspection intent detection
- Evidence ID generation
- Learning claim detection

---

## v0.0.12 - Proactive Anomaly Detection

**Release Date:** 2025-12-03

### Summary

Proactive anomaly detection engine with alert queue, what_changed correlation tool, slowness_hypotheses analysis, and alert surfacing in REPL and one-shot modes. Complete with evidence IDs for traceability.

### Key Features

**Anomaly Detection Engine (`anomaly_engine.rs`):**
- Periodic anomaly detection (every 5 minutes when integrated with daemon)
- Baseline window (14 days) vs recent window (2 days) comparison
- Configurable thresholds for all metrics
- Evidence ID generation (ANO##### format)

**Supported Anomaly Signals:**
- `BootTimeRegression` - Boot time increased significantly
- `CpuLoadIncrease` - CPU load trend increasing
- `MemoryPressure` - High memory usage or swap activity
- `DiskSpaceLow` - Disk space below threshold
- `SystemCrash` - System crash detected
- `ServiceCrash` - Individual service crash
- `ServiceFailed` - Service in failed state
- `JournalWarningsIncrease` - Warning rate increase
- `JournalErrorsIncrease` - Error rate increase
- `DiskIoLatency` - Disk I/O latency increasing

**Alert Queue (`/var/lib/anna/internal/alerts.json`):**
- Deduplication by signal type
- Severity levels: Info, Warning, Critical
- Severity upgrades on repeated occurrences
- Acknowledgment support
- Persistence across restarts

**New Read-Only Tools:**
- `active_alerts` - Returns current alerts with evidence IDs
- `what_changed(days)` - Packages installed/removed, services enabled/disabled, config changes
- `slowness_hypotheses(days)` - Ranked hypotheses combining changes, anomalies, and resource usage

**Alert Surfacing:**
- REPL welcome: Shows active alerts on startup
- One-shot: Footer with alert summary
- `annactl status [ALERTS]` section: Detailed alert display with evidence IDs

**Evidence Integration:**
- All anomalies have unique evidence IDs
- Hypotheses cite supporting evidence
- Junior enforces citation requirements

### Files Changed

- `crates/anna_common/src/anomaly_engine.rs` (NEW - 1600+ lines)
- `crates/anna_common/src/tools.rs` - Added 3 new tools
- `crates/anna_common/src/tool_executor.rs` - Tool implementations
- `crates/anna_common/src/lib.rs` - Module exports
- `crates/annactl/src/main.rs` - Alert surfacing
- `crates/annactl/src/commands/status.rs` - Enhanced [ALERTS] section
- `crates/annactl/src/commands/mod.rs` - Version update

### Tests

16 new unit tests covering:
- Severity ordering and comparison
- Signal deduplication keys
- Alert queue operations (add, acknowledge, dedup)
- What_changed result formatting
- Slowness hypothesis builders
- Anomaly engine defaults
- Time window calculations

---

## v0.0.11 - Safe Auto-Update System

**Release Date:** 2025-12-03

### Summary

Complete safe auto-update system with update channels (stable/canary), integrity verification, atomic installation, zero-downtime restart, and automatic rollback on failure. Full state visibility in status display.

### Key Features

**Update Channels:**
- `stable` (default): Only stable tagged releases (no -alpha, -beta, -rc, -canary suffixes)
- `canary`: Accept all releases including pre-releases

**Configuration (`/etc/anna/config.toml`):**
```toml
[update]
mode = "auto"           # auto or manual
channel = "stable"      # stable or canary
interval_seconds = 600  # check every 10 minutes
min_disk_space_bytes = 104857600  # 100 MB minimum
```

**Safe Update Workflow:**
1. Check GitHub releases API for new version matching channel
2. Download artifacts to staging directory (`/var/lib/anna/internal/update_stage/<version>/`)
3. Verify integrity (SHA256 checksums)
4. Backup current binaries (`/var/lib/anna/internal/update_backup/`)
5. Atomic installation via rename
6. Signal systemd restart
7. Post-restart validation
8. Cleanup staging and old backups

**Rollback Support:**
- Previous binaries kept in backup directory
- Automatic rollback on restart failure
- Manual rollback possible via backup files
- Rollback state shown in `annactl status`

**Guardrails:**
- Never updates during active mutation operations
- Checks disk space before download (100 MB minimum)
- Verifies installer review health before update
- Rate-limits on consecutive failures with exponential backoff
- No partial installations (atomic or nothing)

**Status Display ([UPDATES] section):**
```
[UPDATES]
  Mode:       auto (stable)
  Interval:   10m
  Last check: 2025-12-03 14:30:00
  Result:     up to date
  Next check: in 8m
```

During update:
```
[UPDATES]
  Mode:       auto (stable)
  Interval:   10m
  Progress:   downloading (45%, ETA: 30s)
  Updating:   0.0.10 -> 0.0.11
```

After update:
```
[UPDATES]
  Mode:       auto (stable)
  Interval:   10m
  Last update: 0.0.10 -> 0.0.11 (5m ago)
```

### Technical Details

**UpdatePhase enum:**
- `Idle`, `Checking`, `Downloading`, `Verifying`, `Staging`, `Installing`, `Restarting`, `Validating`, `Completed`, `Failed`, `RollingBack`

**UpdateState fields (v0.0.11):**
- `channel`: Update channel (stable/canary)
- `update_phase`: Current phase name
- `update_progress_percent`: Progress 0-100
- `update_eta_seconds`: Estimated time remaining
- `updating_to_version`: Target version
- `last_update_at`: Timestamp of last successful update
- `previous_version`: For rollback display

**IntegrityStatus:**
- `StrongVerified`: Verified against release checksum
- `WeakComputed`: Self-computed checksum (no release checksum available)
- `Failed`: Checksum mismatch
- `NotVerified`: Skipped verification

### Files Changed

- `crates/anna_common/src/update_system.rs` (new - 600+ lines)
- `crates/anna_common/src/config.rs` (enhanced UpdateConfig, UpdateState, UpdateResult)
- `crates/anna_common/src/lib.rs` (exports)
- `crates/annactl/src/commands/status.rs` (update progress display)
- `crates/annactl/src/main.rs` (version update)

### Tests

```rust
// Version comparison
assert!(is_newer_version("0.0.11", "0.0.10"));
assert!(!is_newer_version("0.0.9", "0.0.10"));

// Channel matching
assert!(stable.matches_tag("v0.0.10"));
assert!(!stable.matches_tag("v0.0.11-alpha"));
assert!(canary.matches_tag("v0.0.11-alpha"));

// Update phase display
assert_eq!(UpdatePhase::Downloading { progress_percent: 50, eta_seconds: Some(30) }
    .format_display(), "downloading... 50% (ETA: 30s)");
```

---

## v0.0.10 - Reset + Uninstall + Installer Review

**Release Date:** 2025-12-03

### Summary

Factory reset and clean uninstall commands with provenance-aware helper removal. Installer review system verifies installation health and auto-repairs common issues. Confirmation phrases required for destructive operations.

### Key Features

**Reset Command (`annactl reset`):**
- Factory reset returns Anna to fresh install state
- Confirmation phrase: "I CONFIRM (reset)"
- --dry-run flag shows what would be deleted
- --force flag skips confirmation prompt
- Clears all data directories, config, and helper tracking
- Recreates directory structure with correct permissions
- Runs installer review at end and reports health status

**Uninstall Command (`annactl uninstall`):**
- Complete Anna removal from system
- Confirmation phrase: "I CONFIRM (uninstall)"
- Provenance-aware helper removal (only anna-installed)
- Asks user about helper removal unless --keep-helpers
- Removes: systemd unit, binaries, data, config
- --dry-run and --force flags supported

**Install State (`install_state.rs`):**
- Tracks installation artifacts for accurate uninstall
- BinaryInfo: path, checksum, version, last_verified
- UnitInfo: path, exec_start, enabled state
- DirectoryInfo: path, expected permissions, ownership
- Review history with last 10 reviews
- Stored at `/var/lib/anna/install_state.json`

**Installer Review (`installer_review.rs`):**
- Verifies installation correctness
- Checks: binary presence, systemd correctness, directories/permissions, config, update scheduler, Ollama health, helper inventory
- Auto-repair for common issues (without user confirmation):
  - Recreate missing internal directories
  - Fix Anna-owned permissions
  - Re-enable annad service if misconfigured
- Evidence IDs for repair tracking (format: IRxxxx)
- ReviewResult: Healthy, Repaired, NeedsAttention, Failed

**Status Display ([INSTALL REVIEW] section):**
```
[INSTALL REVIEW]
  Status:     healthy
  Last run:   5 minute(s) ago
  Duration:   42 ms
```

### Technical Details

- Install state schema version: 1
- Auto-repair rules: read-only or low-risk internal fixes only
- Review checks ordered: critical (binaries) to informational (helpers)
- Confirmation gates use exact phrase matching

### CLI Changes

```bash
annactl reset              # Factory reset (requires root)
annactl reset --dry-run    # Show what would be deleted
annactl reset --force      # Skip confirmation

annactl uninstall          # Complete removal (requires root)
annactl uninstall --dry-run
annactl uninstall --force
annactl uninstall --keep-helpers
```

### Files Changed

- `crates/anna_common/src/install_state.rs` (new)
- `crates/anna_common/src/installer_review.rs` (new)
- `crates/annactl/src/commands/reset.rs` (updated)
- `crates/annactl/src/commands/uninstall.rs` (new)
- `crates/annactl/src/commands/status.rs` (updated)
- `crates/annactl/src/main.rs` (updated)

---

## v0.0.9 - Package Management + Helper Tracking

**Release Date:** 2025-12-03

### Summary

Package management (controlled) with helper tracking and provenance. Tracks all packages Anna relies on, distinguishing between anna-installed and user-installed packages. Only anna-installed packages can be removed via Anna.

### Key Features

**Helper Tracking System (`helpers.rs`):**
- Tracks ALL helpers Anna relies on for telemetry/diagnostics/execution
- Two dimensions: present/missing + installed_by (anna/user/unknown)
- HelperDefinition, HelperState, HelpersManifest types
- InstalledBy enum with Display implementation
- Persistent storage in `/var/lib/anna/helpers.json`
- First-seen and anna-install timestamps
- Provenance tracked per-machine, not globally

**Helper Definitions (9 helpers):**
- `lm_sensors`: Temperature and fan monitoring
- `smartmontools`: SATA/SAS disk health (SMART)
- `nvme-cli`: NVMe SSD health monitoring
- `ethtool`: Network interface diagnostics
- `iw`: WiFi signal and stats
- `usbutils`: USB device enumeration
- `pciutils`: PCI device enumeration
- `hdparm`: SATA disk parameters
- `ollama`: Local LLM inference

**Package Management Mutation Tools:**
- `package_install`: Install package via pacman (tracks as anna-installed)
- `package_remove`: Remove package (only anna-installed packages)
- 8 mutation tools total (6 from v0.0.8 + 2 new)

**Status Display ([HELPERS] section):**
```
[HELPERS]
  Summary:    7 present, 2 missing (1 by Anna)

  ethtool (present, installed by user)
  lm_sensors (present, installed by Anna)
  ollama (present, installed by Anna)
  smartmontools (missing, unknown origin)
```

**StatusSnapshot Extensions:**
- `helpers_total`: Total helpers tracked
- `helpers_present`: Helpers currently installed
- `helpers_missing`: Helpers not installed
- `helpers_anna_installed`: Helpers installed by Anna
- `helpers`: Vec<HelperStatusEntry> for detailed display

**Transaction Logging:**
- MutationType::PackageInstall, MutationType::PackageRemove
- MutationDetails::Package with package, version, reason, operation
- log_package_operation() in RollbackManager

**Provenance Rules:**
- If helper present before Anna tracked it → installed_by=user
- If Anna installs helper → installed_by=anna
- Only helpers with installed_by=anna are removal_eligible
- package_remove rejects non-anna packages with clear error

### New Error Types

- `PackageAlreadyInstalled(String)`
- `PackageNotInstalled(String)`
- `PackageNotRemovable { package, reason }`
- `PackageInstallFailed { package, reason }`
- `PackageRemoveFailed { package, reason }`

### New Files

- `crates/anna_common/src/helpers.rs` - Helper tracking system

### Modified Files

- `crates/anna_common/src/mutation_tools.rs` - Added package_install, package_remove tools
- `crates/anna_common/src/mutation_executor.rs` - Added package execution functions
- `crates/anna_common/src/rollback.rs` - Added PackageInstall/Remove types and logging
- `crates/anna_common/src/daemon_state.rs` - Added helpers fields to StatusSnapshot
- `crates/anna_common/src/lib.rs` - Added helpers module and exports
- `crates/annactl/src/commands/status.rs` - Added [HELPERS] section

### New Tests

**helpers.rs:**
- test_helper_state_new_missing
- test_helper_state_new_user_installed
- test_helper_state_mark_anna_installed
- test_helper_state_already_present_not_anna
- test_helper_definitions
- test_manifest_removal_eligible
- test_installed_by_display
- test_format_status

### Breaking Changes

- None - backward compatible with v0.0.8
- New fields in StatusSnapshot are optional with `#[serde(default)]`

---

## v0.0.8 - First Safe Mutations

**Release Date:** 2025-12-03

### Summary

First safe mutations (medium-risk only) with automatic rollback and confirmation gates. Introduces mutation tool catalog with allowlist enforcement, file backup system, and Junior reliability threshold for execution approval.

### Key Features

**Mutation Tool Catalog (`mutation_tools.rs`):**
- Allowlist-enforced mutation tools (6 tools total)
- `edit_file_lines`: Text file edits under /etc/** and $HOME/**
- `systemd_restart`: Service restart
- `systemd_reload`: Service configuration reload
- `systemd_enable_now`: Enable and start service
- `systemd_disable_now`: Disable and stop service
- `systemd_daemon_reload`: Reload systemd daemon
- MutationRisk enum (Medium, High)
- File size limit: MAX_EDIT_FILE_SIZE = 1 MiB
- Path validation: is_path_allowed(), validate_mutation_path()

**Rollback System (`rollback.rs`):**
- RollbackManager with timestamped backups
- Backup location: /var/lib/anna/rollback/files/
- Logs location: /var/lib/anna/rollback/logs/
- File hashing (SHA256) for integrity verification
- Diff summary generation for file edits
- Structured JSON logs per mutation request
- JSONL append log (mutations.jsonl) for audit trail
- Rollback instructions with exact restore commands

**Confirmation Gate:**
- Exact phrase requirement: "I CONFIRM (medium risk)"
- Validation via validate_confirmation()
- User confirmation displayed in dialogue transcript
- Action NOT executed if phrase doesn't match exactly

**Junior Verification Threshold:**
- Minimum 70% reliability required for execution
- MutationPlan.junior_approved flag
- MutationPlan.junior_reliability score
- is_approved_for_execution() check

**Mutation Executor (`mutation_executor.rs`):**
- execute_mutation(): Single mutation execution
- execute_mutation_plan(): Batch execution
- Request ID generation (UUID v4)
- Evidence ID propagation from tool results
- Structured MutationResult with human_summary and rollback_info

**Pipeline Updates (`pipeline.rs`):**
- ActionPlan extended with mutation_plan and is_medium_risk_executable
- generate_action_plan() creates MutationPlan for service operations
- handle_mutation_execution() for confirmation and execution flow
- ensure_service_suffix() helper for .service extension

### Not Allowed in v0.0.8

The following are explicitly NOT executable:
- Package installs/removals (pacman, yay)
- Filesystem operations (mount, umount, mkfs)
- Destructive operations (rm, rmdir)
- Permission changes (chmod, chown)
- User management (useradd, userdel, usermod)
- Firewall rules (iptables, ufw)
- Kernel parameters (sysctl)

### Mutation Execution Flow

```
[you] to [anna]: restart nginx
[anna] to [translator]: Classify this request...
[translator] to [anna]: Intent: action_request, Targets: nginx
                        Risk: medium
[anna] generates action plan with MutationPlan:
  What will change: Services to restart: nginx.service
  Why: Service restart requested
  Risk: medium
  Rollback: Service state can be restored by restarting again
[anna] to [junior]: Verify this mutation plan...
[junior] to [anna]: Reliability: 85%
[anna] to [you]: I can execute this action for you.

                 What will change:
                   Services to restart: nginx.service

                 Risk level: medium

                 To proceed, type exactly: I CONFIRM (medium risk)
[you] to [anna]: I CONFIRM (medium risk)
[anna] to [annad]: User confirmed. Please execute:
                   - systemd_restart (nginx.service)
[annad] executes: systemctl restart nginx.service
SUCCESS systemd_restart: Restarted nginx.service
  Rollback: sudo systemctl restart nginx.service
[annad] to [anna]: All operations completed successfully.
                   1 mutation(s) executed.
                   Logs saved to: /var/lib/anna/rollback/logs/
```

### Rollback Log Example

```json
{
  "request_id": "a1b2c3d4-...",
  "timestamp": 1701619200,
  "tool_name": "systemd_restart",
  "evidence_ids": ["E1", "E2"],
  "mutation_type": "systemd",
  "target": "nginx.service",
  "details": {
    "Systemd": {
      "operation": "restart",
      "service": "nginx.service",
      "previous_state": { "active": true, "enabled": true }
    }
  },
  "success": true,
  "error": null
}
```

### New Tests

**mutation_tools.rs:**
- test_path_allowed_etc
- test_path_allowed_home
- test_path_not_allowed
- test_confirmation_valid
- test_confirmation_missing
- test_confirmation_wrong
- test_mutation_catalog_has_expected_tools
- test_mutation_catalog_rejects_unknown
- test_mutation_plan_approval
- test_risk_display

**rollback.rs:**
- test_diff_summary_generation
- test_backup_file
- test_file_hash
- test_rollback_info_generation

### Breaking Changes

- None - backward compatible with v0.0.7
- New fields in ActionPlan are optional (mutation_plan, is_medium_risk_executable)

---

## v0.0.7 - Read-Only Tooling & Evidence Citations

**Release Date:** 2025-12-03

### Summary

Read-only tool catalog with allowlist enforcement, Evidence IDs for citations, and human-readable natural language transcripts. Junior now enforces no-guessing with uncited claim detection. Translator outputs tool plans for evidence gathering.

### Key Features

**Read-Only Tool Catalog (`tools.rs`):**
- Internal tool allowlist with security classification
- 10 read-only tools: status_snapshot, sw_snapshot_summary, hw_snapshot_summary, recent_installs, journal_warnings, boot_time_trend, top_resource_processes, package_info, service_status, disk_usage
- ToolDef with human-readable descriptions and latency hints
- ToolSecurity enum (ReadOnly, LowRisk, MediumRisk, HighRisk)
- parse_tool_plan() for parsing Translator output

**Tool Executor (`tool_executor.rs`):**
- execute_tool() for individual tool execution
- execute_tool_plan() for batch execution with EvidenceCollector
- Structured ToolResult with human_summary
- Unknown tool handling with graceful errors

**Evidence IDs and Citations:**
- EvidenceCollector assigns sequential IDs (E1, E2, ...)
- Evidence IDs in all tool results and dialogue
- Citations expected in final responses: [E1], [E2]
- Evidence legend in final response

**Junior No-Guessing Enforcement:**
- UNCITED_CLAIMS output field for speculation detection
- Strict scoring: uncited claims = -15 per claim
- "Unknown" preferred over guessing
- Label inference explicitly: "Based on [E2], likely..."

**Translator Tool Plans:**
- TOOLS output field in Translator response
- Tool plan parsing from comma-separated calls
- RATIONALE field for tool selection reasoning
- Deterministic fallback generates tool plans from evidence_needs

**Natural Language Transcripts:**
```
[anna] to [annad]: Please gather evidence using: hw_snapshot_summary
[annad] to [anna]: [E1] hw_snapshot_summary: CPU: AMD Ryzen 7 5800X (8 cores)
                                            Memory: 32GB total, 16GB available (found)
```

### Pipeline Flow (v0.0.7)

```
[you] to [anna]: what CPU do I have?
[anna] to [translator]: Classify this request...
[translator] to [anna]: Intent: system_query, Targets: cpu
                        Risk: read_only
                        Tools: hw_snapshot_summary
[anna] to [annad]: Please gather evidence using: hw_snapshot_summary
[annad] to [anna]: [E1] hw_snapshot_summary: CPU: AMD Ryzen 7 5800X (found)
[anna] to [junior]: Verify this draft response:
                    Based on gathered evidence:
                    [E1] CPU: AMD Ryzen 7 5800X
[junior] to [anna]: Reliability: 95%, Critique: All claims cite evidence
                    Uncited claims: none
[anna] to [you]: Based on gathered evidence:
                 [E1] CPU: AMD Ryzen 7 5800X
                 ---
                 Evidence sources:
                   [E1]: hw_snapshot_summary (OK)
Reliability: 95%
```

### New Tests (7 tests added)

- test_deterministic_generates_tool_plan
- test_deterministic_service_query_generates_service_tool
- test_parse_junior_response_with_uncited_claims
- test_parse_junior_response_no_uncited_claims
- test_tool_catalog_creation
- test_evidence_collector
- test_fallback_scoring_v2_with_tool_results

### Breaking Changes

- None - backward compatible with v0.0.6
- Legacy evidence retrieval still works when tool_plan is None

---

## v0.0.6 - Real Translator LLM

**Release Date:** 2025-12-03

### Summary

Real LLM-powered Translator for intent classification with clarification loop support. Evidence-first pipeline with real snapshot integration and 8KB excerpt cap. Action plan generation for mutation requests (no execution yet).

### Key Features

**Real Translator LLM (`pipeline.rs`):**
- Real LLM-backed intent classification replacing deterministic mock
- Structured output parsing: intent, targets, risk, evidence_needs, clarification
- System prompt with strict output format
- Fallback to deterministic classification when LLM unavailable

**Clarify-or-Proceed Loop:**
- Multiple-choice clarification prompts
- Default option selection
- Single-turn clarification (no infinite loops)
- CLARIFICATION field in Translator output: `question|option1|option2|option3|default:N`

**Evidence-First Pipeline:**
- Real snapshot integration from annad
- Evidence excerpting with 8KB hard cap
- `[EXCERPT truncated]` indication when data exceeds limit
- Evidence sources: hw_snapshot, sw_snapshot, status, journalctl

**Action Plan Generation:**
- Action plans for action_request intent
- Steps, affected files/services/packages
- Risk classification propagation
- Rollback outline
- Confirmation phrase (no execution - confirmation-gated)

**Translator System Prompt:**
```
OUTPUT FORMAT (follow exactly, one field per line):
INTENT: [question|system_query|action_request|unknown]
TARGETS: [comma-separated list or "none"]
RISK: [read_only|low|medium|high]
EVIDENCE_NEEDS: [hw_snapshot, sw_snapshot, status, journalctl, or "none"]
CLARIFICATION: [empty OR "question|opt1|opt2|opt3|default:N"]
```

### Pipeline Flow (v0.0.6)

```
[you] to [anna]: install nginx
[anna] to [translator]: Please classify this request...
[translator thinking via qwen2.5:0.5b...]
[translator] to [anna]: Intent: action_request, Targets: nginx, Risk: medium
                        Evidence: sw_snapshot
                        Clarification: none
[anna] to [annad]: Retrieve evidence for: nginx
[annad] to [anna]: snapshot:sw: [package data, 8KB max excerpt]
[anna] generates action plan:
  Steps: 1. Run pacman -S nginx  2. Enable nginx.service  3. Start nginx.service
  Affected: packages: nginx, services: nginx.service
  Rollback: pacman -Rns nginx
  Confirmation: Type "I understand and accept the risk" to proceed
[anna] to [junior]: Please verify this action plan...
[junior] to [anna]: Reliability: 75%, Critique: Plan looks correct...
                    MUTATION_WARNING: This will install a package.
[anna] to [you]: Action plan ready. Type confirmation phrase to execute.
Reliability: 75%
```

### Tests

- 15 pipeline unit tests (parsing, clarification, evidence, action plans)
- 20 CLI integration tests (all passing)
- Test coverage for deterministic fallback
- Evidence excerpting edge cases

### Breaking Changes

- None - backward compatible with v0.0.5

---

## v0.0.5 - Role-Based Model Selection + Benchmarking

**Release Date:** 2025-12-03

### Summary

Role-based LLM model selection with hardware-aware configuration and built-in benchmarking. Translator (fast) and Junior (reliable) now have distinct model pools selected based on system capabilities. Bootstrap process with progress tracking.

### Key Features

**Hardware Detection (`model_selection.rs`):**
- CPU detection (cores, model name)
- RAM detection (total, available)
- GPU/VRAM detection via nvidia-smi
- Hardware tier classification:
  - Low: <8GB RAM
  - Medium: 8-16GB RAM
  - High: >16GB RAM

**Role-Based Model Selection:**
- LlmRole enum: Translator, Junior
- Translator candidates (smallest/fastest first): qwen2.5:0.5b, qwen2.5:1.5b, phi3:mini, gemma2:2b, llama3.2:1b
- Junior candidates (most reliable first): llama3.2:3b-instruct, qwen2.5:3b-instruct, mistral:7b-instruct, gemma2:9b
- Priority-based selection respecting hardware tier

**Benchmark Suites:**
- Translator: 30 prompts testing intent classification (read-only vs mutation, targets, etc.)
- Junior: 15 cases testing verification quality (evidence handling, honesty scoring, etc.)
- Per-case latency and pattern matching evaluation

**Ollama Model Pull with Progress:**
- Streaming pull progress via `/api/pull`
- Real-time progress percentage
- Download speed (MB/s)
- ETA calculation
- Progress exposed in status snapshot

**Bootstrap State Machine:**
- `detecting_ollama`: Checking Ollama availability
- `installing_ollama`: Installing Ollama (future)
- `pulling_models`: Downloading required models
- `benchmarking`: Running model benchmarks (future)
- `ready`: Models ready for use
- `error`: Bootstrap failed

**Status Snapshot Updates (schema v3):**
- `llm_bootstrap_phase`: Current phase
- `llm_translator_model`: Selected translator
- `llm_junior_model`: Selected junior
- `llm_downloading_model`: Model being pulled
- `llm_download_percent`: Progress percentage
- `llm_download_speed`: Download speed
- `llm_download_eta_secs`: ETA in seconds
- `llm_error`: Error message if any
- `llm_hardware_tier`: Detected tier

**annactl Progress Display:**
- Shows bootstrap progress when models not ready
- Progress bar for model downloads
- ETA display
- Graceful fallback with reduced reliability score (-10 points)
- Explicit reason shown when LLM unavailable

### Configuration

New LLM settings in `/etc/anna/config.toml`:
```toml
[llm]
enabled = true
ollama_url = "http://127.0.0.1:11434"

[llm.translator]
model = ""  # Empty for auto-select
timeout_ms = 30000
enabled = true

[llm.junior]
model = ""  # Empty for auto-select
timeout_ms = 60000
enabled = true

# Custom candidate pools (optional)
translator_candidates = ["qwen2.5:0.5b", "qwen2.5:1.5b"]
junior_candidates = ["llama3.2:3b-instruct", "mistral:7b-instruct"]
```

### Tests

- 21 model_selection unit tests
- Hardware tier boundary tests
- Model priority selection tests
- Fallback behavior tests
- Bootstrap state tests

### Breaking Changes

- Config structure changed: `junior.*` deprecated, use `llm.junior.*`
- Status snapshot schema bumped to v3

---

## v0.0.4 - Real Junior Verifier

**Release Date:** 2024-12-03

### Summary

Junior becomes a real LLM-powered verifier via local Ollama. Translator remains deterministic. No Senior implementation yet - keeping complexity low while measuring real value.

### Key Features

**Junior LLM Integration:**
- Real verification via Ollama local LLM
- Auto-selects best model (prefers qwen2.5:1.5b, llama3.2:1b, etc.)
- Structured output parsing (SCORE, CRITIQUE, SUGGESTIONS, MUTATION_WARNING)
- Fallback to deterministic scoring when Ollama unavailable
- Spinner while Junior thinks

**Ollama Client (`ollama.rs`):**
- HTTP client for local Ollama API
- Health check, model listing, generation
- Timeout and retry handling
- Model auto-selection based on availability

**Junior Config:**
- `junior.enabled` (default: true)
- `junior.model` (default: auto-select)
- `junior.timeout_ms` (default: 60000)
- `junior.ollama_url` (default: http://127.0.0.1:11434)

### Pipeline Flow (with real LLM)

```
[you] to [anna]: what CPU do I have?
[anna] to [translator]: Please classify this request...
[translator] to [anna]: Intent: system_query, Targets: cpu, Risk: read-only, Confidence: 85%
[anna] to [annad]: Retrieve evidence for: cpu
[annad] to [anna]: snapshot:hw.cpu: [CPU data]
[anna] to [junior]: Please verify this draft response...
[junior thinking via qwen2.5:1.5b...]
[junior] to [anna]: Reliability: 80%
                    Critique: The response mentions evidence but doesn't parse it
                    Suggestions: Add specific CPU model and core count
[anna] to [you]: Based on system data from: snapshot:hw.cpu...
Reliability: 80%
```

### Junior System Prompt

Junior is instructed to:
- NEVER invent machine facts
- Downscore missing evidence
- Prefer "unknown" over guessing
- Keep output short and structured
- Warn about mutations for action requests

### Graceful Degradation

When Ollama is not available:
- REPL shows warning with install instructions
- Pipeline falls back to deterministic scoring (v0.0.3 logic)
- Exit code 0 - no crashes

### Tests

- 9 unit tests for pipeline (Translator, Junior parsing, fallback scoring)
- 20 CLI integration tests
- 4 new v0.0.4 tests (Critique, Suggestions, mutation warning, graceful degradation)

### Model Selection Order

1. qwen2.5:1.5b (fastest, good for verification)
2. qwen2.5:3b
3. llama3.2:1b
4. llama3.2:3b
5. phi3:mini
6. gemma2:2b
7. mistral:7b
8. First available model

---

## v0.0.3 - Request Pipeline Skeleton

**Release Date:** 2024-12-03

### Summary

Implements the full multi-party dialogue transcript with deterministic mocks for intent classification, evidence retrieval, and Junior scoring. No LLM integration yet - all behavior is keyword-based and deterministic.

### Pipeline Flow

```
[you] to [anna]: what CPU do I have?
[anna] to [translator]: Please classify this request...
[translator] to [anna]: Intent: system_query, Targets: cpu, Risk: read-only, Confidence: 85%
[anna] to [annad]: Retrieve evidence for: cpu
[annad] to [anna]: snapshot:hw.cpu: [CPU data would come from snapshot]
[anna] to [junior]: Please verify and score this response.
[junior] to [anna]: Reliability: 100%, Breakdown: +40 evidence, +30 confident, +20 observational+cited, +10 read-only
[anna] to [you]: Based on system data from: snapshot:hw.cpu...
Reliability: 100%
```

### Changes

**Pipeline Module (`pipeline.rs`):**
- DialogueActor enum: You, Anna, Translator, Junior, Annad
- `dialogue()` function with format: `[actor] to [target]: message`
- IntentType enum: question, system_query, action_request, unknown
- RiskLevel enum: read-only, low-risk, medium-risk, high-risk
- Intent struct with keywords, targets, risk, confidence
- Evidence struct with source, data, timestamp

**Translator Mock:**
- Keyword-based intent classification
- Target detection (cpu, memory, disk, network, docker, nginx, etc.)
- Action keyword detection (install, remove, restart, etc.)
- Confidence scoring based on keyword matches

**Evidence Retrieval Mock:**
- Maps targets to snapshot sources (hw.cpu, hw.memory, sw.services.*)
- Returns mock evidence with timestamps
- System queries trigger annad dialogue

**Junior Scoring:**
- +40: evidence exists
- +30: confident classification (>70%)
- +20: observational + cited (read-only with evidence)
- +10: read-only operation
- Breakdown shown in output

**Tests:**
- test_annactl_pipeline_shows_translator
- test_annactl_pipeline_shows_junior
- test_annactl_pipeline_shows_annad_for_system_query
- test_annactl_pipeline_intent_classification
- test_annactl_pipeline_target_detection
- test_annactl_pipeline_reliability_breakdown
- test_annactl_pipeline_action_risk_level

### Internal Notes

- All responses are mocked (no LLM integration)
- Evidence retrieval is simulated (no actual snapshot reads)
- Risk classification is keyword-based
- Pipeline is ready for LLM integration in 0.1.x

---

## v0.0.2 - Strict CLI Surface

**Release Date:** 2024-12-03

### Summary

Enforces the strict CLI surface. All legacy commands (sw, hw, JSON flags) are removed from public dispatch and now route through natural language processing.

### Supported Entrypoints

```bash
annactl                  # REPL mode (interactive)
annactl <request>        # One-shot natural language request
annactl status           # Self-status
annactl --version        # Version (also: -V)
annactl --help           # Help (also: -h)
```

**That's the entire public surface.**

### Changes

**CLI Surface:**
- Removed `sw` command from public surface
- Removed `hw` command from public surface
- Removed all JSON flags (--json, --full) from public surface
- Legacy commands now route as natural language requests (no custom error message)
- Added --help/-h flags for explicit help display

**REPL Mode:**
- Implemented basic REPL loop
- Exit commands: exit, quit, bye, q
- Help command shows REPL-specific help
- Status command works in REPL

**Dialogue Format:**
- Natural language requests show `[you] to [anna]:` format
- Responses show `[anna] to [you]:` format
- Reliability score displayed (stub: 0% until LLM integration)

**Tests:**
- Added test for --help showing strict surface only
- Added test for status command exit 0
- Added test for --version format
- Added test for legacy command routing (sw, hw)
- Added test for natural language request format

### Breaking Changes

- `annactl sw` no longer shows software overview (routes as request)
- `annactl hw` no longer shows hardware overview (routes as request)
- `annactl` (no args) now enters REPL instead of showing help
- Use `annactl --help` or `annactl -h` for help

### Internal

- Snapshot architecture preserved (internal capabilities only)
- Status command unchanged
- Version output format unchanged: `annactl vX.Y.Z`

---

## v0.0.1 - Specification Lock-In

**Release Date:** 2024-12-03

### Summary

Complete specification reset. Anna transitions from a "snapshot reader with fixed commands" to a "natural language virtual sysadmin" architecture.

### Changes

**Governance:**
- Established immutable operating contract (CLAUDE.md)
- Created implementation roadmap (TODO.md)
- Set up release notes workflow
- Version reset to 0.0.1 (staying in 0.x.x until production)

**Documentation:**
- README.md rewritten for natural language assistant vision
- CLAUDE.md created with full engineering contract
- TODO.md created with phased implementation roadmap
- RELEASE_NOTES.md created for change tracking

**Architecture Decision:**
- Preserve existing snapshot-based telemetry foundation
- Build natural language layer on top
- Strict CLI surface: `annactl`, `annactl <request>`, `annactl status`, `annactl --version`
- All old commands (sw, hw, JSON flags) become internal capabilities only

**Spec Highlights:**
- 4-player model: User, Anna, Translator, Junior, Senior
- Debug mode always on (visible dialogue)
- Reliability scores on all answers (0-100%)
- Safety classification: read-only, low-risk, medium-risk, high-risk
- Rollback mandate for all mutations
- Recipe learning system
- XP and gamification (levels 0-100, nerdy titles)
- Auto-update every 10 minutes
- Auto Ollama setup

### Breaking Changes

- Version number reset from 7.42.5 to 0.0.1
- Old CLI commands will be removed in 0.0.2
- New CLI surface is strict and minimal

### Migration Path

Existing snapshot infrastructure is preserved. Natural language capabilities will be added incrementally without breaking current performance.

---

## Previous Versions

Prior to v0.0.1, Anna was a snapshot-based telemetry daemon with fixed CLI commands. See git history for v7.x releases.
