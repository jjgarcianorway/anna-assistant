# Sysadmin Recipes Inventory: Services, Disk, Logs

**Purpose**: Internal developer mapping of existing templates, modules, and diagnostic rules for services, disk, and logs.

## Services (Systemd)

### Existing Infrastructure

**Module**: `crates/anna_common/src/systemd_health.rs`
- `SystemdHealth::detect()` - Main detection function
- `detect_failed_units()` - Lists failed systemd units
- `detect_essential_timers()` - Checks timer status
- `FailedUnit` struct with name, type, load/active/sub states

**Commands Used**:
```bash
systemctl list-units --state=failed --no-pager --no-legend
systemctl list-timers --all --no-pager
```

**Current Usage**:
- Used by diagnostic engine in daemon
- Available via RPC `BrainAnalysis`
- Surfaced in TUI brain panel and CLI status

**What's Missing**:
- No focused NL handler for "why is X service failing"
- No journalctl integration for service-specific errors
- Generic LLM answers instead of deterministic service health templates

## Disk Space

### Existing Infrastructure

**Module**: `crates/anna_common/src/disk_analysis.rs`
- Disk usage detection and analysis
- Mount point monitoring
- Inode usage tracking

**Module**: `crates/anna_common/src/filesystem.rs`
- Filesystem health checks
- Mount status verification

**Commands Used**:
```bash
df -h
df -i
du -sh /var/*
lsblk
findmnt
```

**Current Usage**:
- Diagnostic engine checks disk space thresholds
- Brain analysis includes disk warnings
- Available in telemetry snapshots

**What's Missing**:
- No focused "what is using disk space" handler
- No du-based top consumers analysis in answers
- Generic responses instead of specific mount point + usage data

## Logs (Journalctl)

### Existing Infrastructure

**Module**: `crates/anna_common/src/systemd_health.rs`
- `detect_journal_disk_usage()` - Checks journal size
- `check_journal_rotation_config()` - Validates rotation

**Diagnostic Rules**:
- Brain checks for critical systemd errors
- Log analysis for failed service correlation

**Commands Used**:
```bash
journalctl -p err -n 50
journalctl -p crit -b
journalctl -u <service> --since "1 hour ago"
journalctl --disk-usage
```

**Current Usage**:
- Diagnostic engine scans for critical log entries
- Used to correlate service failures
- Journal disk usage monitoring

**What's Missing**:
- No "show me errors in my logs" focused handler
- No structured error summarization (currently raw log spam risk)
- No temporal scoping for recent errors (last hour, today, etc.)

## Action Items for Beta.263

### Services
1. Create `compose_service_health_answer()` function
2. Wire to NL queries: "service health", "failed services", "why is X failing"
3. Format: [SUMMARY] + [DETAILS] + [COMMANDS] using existing `SystemdHealth` data

### Disk
1. Create `compose_disk_health_answer()` function
2. Wire to NL queries: "disk space", "what's using disk", "disk full"
3. Format: [SUMMARY] + [DETAILS] + [COMMANDS] using existing disk analysis

### Logs
1. Create `compose_log_health_answer()` function
2. Wire to NL queries: "log errors", "critical errors", "check logs"
3. Format: [SUMMARY] + [DETAILS] + [COMMANDS] using journal analysis

## Notes

- All infrastructure already exists in diagnostic engine
- Main gap is NL query → deterministic answer wiring
- Should reuse Brain analysis data instead of re-querying system
- Answer patterns should match canonical format from ANSWER_FORMAT.md
