# Sysadmin Recipes Inventory: Services, Disk, Logs, CPU, Memory, Processes, Network

**Purpose**: Internal developer mapping of existing templates, modules, and diagnostic rules for system troubleshooting.

**Last Updated**: Beta.264

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

## CPU Load

### Existing Infrastructure

**Module**: `crates/anna_common/src/telemetry.rs`
- `CpuInfo` struct with usage percentage
- Load average tracking
- CPU count available

**Commands Used**:
```bash
uptime
ps -eo pid,comm,%cpu --sort=-%cpu | head
top
htop
```

**Current Usage**:
- CPU metrics in telemetry snapshots
- Load average monitoring
- Available via RPC

**What's Missing** (Beta.264):
- No focused "is my CPU overloaded" handler
- No process-level CPU consumer analysis
- Generic LLM answers instead of deterministic patterns

## Memory & Swap

### Existing Infrastructure

**Module**: `crates/anna_common/src/telemetry.rs`
- `MemoryInfo` struct with total/used/available/swap
- Memory usage percentage
- Swap usage tracking

**Commands Used**:
```bash
free -h
cat /proc/meminfo | head
ps -eo pid,comm,%mem --sort=-%mem | head
```

**Current Usage**:
- Memory metrics in telemetry
- Swap pressure detection
- Available via diagnostic engine

**What's Missing** (Beta.264):
- No focused "am I low on memory" handler
- No swap pressure explanation
- Generic responses for memory questions

## Processes

### Existing Infrastructure

**Module**: System telemetry and diagnostic rules
- Process scanning capabilities
- CPU/memory per-process tracking possible

**Commands Used**:
```bash
ps aux
ps -eo pid,comm,%cpu,%mem --sort=-%cpu | head
pgrep
top
```

**Current Usage**:
- Brain can detect misbehaving processes
- Process info available in telemetry

**What's Missing** (Beta.264):
- No focused "what is using resources" handler
- No top process consumer reporting
- Generic answers for process queries

## Network

### Existing Infrastructure

**Module**: `crates/anna_common/src/network_config.rs`
- Network interface detection
- Basic connectivity checks possible

**Commands Used**:
```bash
ip addr show
ip route
ping -c 4 1.1.1.1
ping -c 4 example.com
ss
netstat
systemctl status NetworkManager
```

**Current Usage**:
- Network status monitoring
- Interface configuration detection

**What's Missing** (Beta.264):
- No focused "is my network ok" handler
- No basic connectivity health check
- Generic responses for network questions

## Action Items

### Beta.263 (Completed)
- ✅ Services: `compose_service_health_answer()`
- ✅ Disk: `compose_disk_health_answer()`
- ✅ Logs: `compose_log_health_answer()`

### Beta.264 (Current)
- CPU: `compose_cpu_health_answer()`
- Memory: `compose_memory_health_answer()`
- Processes: `compose_process_health_answer()`
- Network: `compose_network_health_answer()`

## Notes

- All infrastructure already exists in diagnostic engine and telemetry
- Main gap is NL query → deterministic answer wiring
- Should reuse telemetry and Brain data instead of re-querying system
- Answer patterns should match canonical format from ANSWER_FORMAT.md
- All composers follow [SUMMARY] + [DETAILS] + [COMMANDS] structure
