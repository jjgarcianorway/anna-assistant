# Changelog

All notable changes to Anna are documented in this file.

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
