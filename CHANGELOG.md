# Changelog

All notable changes to Anna are documented in this file.

## [7.37.0] - 2025-12-02

### Added

- **Auto-update scheduler** - Functional update checks with proper state persistence:
  - State file created on daemon start at `/var/lib/anna/internal/update_state.json`
  - `Last check` shows real timestamps (never shows "never" after first check)
  - `Next check` shows scheduled time with countdown
  - Exponential backoff on failures (capped at 1 hour)
- **Instrumentation state tracking** - New module for tool install tracking:
  - Records tool_id, package, install_time, reason, scope, trigger, result
  - Persists to `/var/lib/anna/internal/instrumentation_state.json`
  - Explicit clean statement when no tools installed: "none (clean)"
- **Idle detection module** - System load and pacman lock awareness:
  - CPU load threshold (1-minute average < 2.0)
  - Pacman lock detection (`/var/lib/pacman/db.lck`)
  - Safe for background operations
- **Explicit clean statements** - All logs sections show clean status when empty:
  - `Installed: none (clean)` for instrumentation
  - `(no ops recorded yet, clean)` for ops log
  - `(ready)` or `(will create on daemon start)` for internal dir

### Changed

- Daemon creates all internal paths on startup (including kdb/chunks, kdb/facts)
- Status [UPDATES] shows "not yet (first check pending)" instead of "never"
- Status [INSTRUMENTATION] shows auto-install status, AUR gate, rate limit
- Status [PATHS] shows explicit ready/missing status for internal dir
- Installer version detection uses target path binary first

### Fixed

- **Update scheduler actually runs** - Proper initialization on daemon start
- **Installer version detection** - Uses `/usr/local/bin/annad --version` first
- **Internal paths created** - All directories created on daemon startup
- **Clean statements** - No empty sections, always explicit status

---

## [7.36.0] - 2025-12-02

### Added

- **Chunk storage module** - New bounded knowledge storage with hard limits:
  - `MAX_CHUNK_BYTES = 16,384` (16 KiB per chunk)
  - `MAX_DOC_BYTES = 512,000` (500 KiB total per document)
  - Automatic chunking with index for all large content
  - Documents exceeding limits are truncated at ingest with metadata tracking
- **Deterministic fact extraction** - Extract structured facts without LLM:
  - Config paths (e.g., `/etc/pacman.conf`)
  - Service units (e.g., `systemd-networkd.service`)
  - Kernel modules (from modprobe commands)
  - Package names (from install commands)
  - Environment variables
- **Bounded rendering** - Page budgets per command:
  - `BUDGET_STATUS = 4,000` bytes for status command
  - `BUDGET_OVERVIEW = 16,000` bytes for sw/hw overview
  - `BUDGET_DETAIL = 32,000` bytes for sw/hw detail
- **Content sanitization** - Strip HTML and wiki markup to plain text before storage
- **Table of contents extraction** - Store heading outline for large documents
- **Overflow information** - When content exceeds budget, show "More available in knowledge store"

### Changed

- Knowledge store modularized to: `kdb/index.json` + `kdb/chunks/` + `kdb/facts/`
- No single stored field can exceed `MAX_CHUNK_BYTES`
- All module headers updated to v7.36.0

### Fixed

- Prevents token overflow by enforcing hard limits at storage time
- Truncation happens at ingest, not at display (no ellipsis in output)

---

## [7.35.1] - 2025-12-02

### Added

- **--version flag** - Both `annad --version` and `annactl --version` now output version string for installer detection
- **[AVAILABLE QUERIES] section** - `annactl hw` now shows valid categories and detected devices for quick navigation
- **[PLATFORMS] section** - `annactl sw` shows Steam games with size info from local manifests
- **Enhanced USB display** - `annactl hw usb` now shows power (mA), speed, and driver per device
- **Network interface names** - Hardware overview includes interface names for device queries
- **Installer version detection** - Strict 4-step precedence: annad --version → annactl --version → version.json → not installed
- **Version stamp file** - Installer writes `/var/lib/anna/internal/version.json` after successful install
- **Installation verification** - Installer verifies annad/annactl --version output and systemd ExecStart path

### Changed

- HW command header updated to v7.35.1
- SW command header updated to v7.35.1
- USB category now groups devices by type with full details
- Categories section only shows detected hardware
- All sections show "only what exists" - no placeholders

### Fixed

- Network stability metrics with WiFi signal trends and disconnect counts were already present
- Sensors category fully functional with hwmon enumeration
- Update checks continue to run and persist state correctly (from v7.34.0)

---

## [7.34.0] - 2025-12-02

### Fixed

- **Update scheduler actually runs** - The daemon now properly initializes and executes scheduled update checks
- **Real timestamps in status** - `annactl status` shows actual Last check and Next check timestamps instead of "never"
- **Consolidated update state** - Removed redundant `update_state.rs` module, unified on `config::UpdateState`

### Added

- **Ops.log audit trail** - Update checks logged to `/var/lib/anna/internal/ops.log`
- **Daemon down detection** - Status shows clear warning when daemon is not running
- **First check on start** - If never checked, schedules first check within 60 seconds of daemon start
- **Error display** - Status shows last error message if update check failed
- **CONTRIBUTING.md** - Repository maintenance policy documentation
- **CHANGELOG.md** - This file

### Changed

- Update state file moved to `/var/lib/anna/internal/update_state.json`
- UpdateState struct now includes: mode, interval_seconds, last_check_at, last_result, last_error, next_check_at
- Scheduler tick interval reduced from 60s to 30s for more responsive checks
- README.md completely rewritten to match current product truth

### Removed

- Deleted `update_state.rs` module (consolidated into `config.rs`)
- Removed legacy README content describing old features

---

## [7.33.0] - 2025-12-02

### Added

- Real update checking implementation via GitHub API
- checkupdates/pacman for system package updates
- Semantic version comparison
- Peripheral inventory (USB, Thunderbolt, SD, Bluetooth, sensors)

---

## [7.32.0] - 2025-12-01

### Added

- Evidence-based software categorization
- Steam game detection from local appmanifest files
- WiFi signal/link quality trends
- On-demand scoped scans with time budget

---

## [7.31.0] - 2025-12-01

### Added

- Concrete telemetry readiness model
- Trend windows with availability checks
- Global percent formatting with ranges
- Truthful auto-update status

### Fixed

- Process identity (no more "Bun Pool" nonsense)

---

## Previous Versions

See git history for detailed changes in earlier versions.
