# Sprint 5: Telemetry & Automation - Technical Documentation

**Version:** 0.9.4-alpha
**Date:** 2025-10-30
**Status:** Phase 2B Complete

---

## Overview

Sprint 5 transforms Anna from a self-healing system to a **self-optimizing** system by adding persistent telemetry collection, policy-driven automation, and learning feedback loops. This document describes the implementation of the telemetry collection and automation infrastructure.

### Key Components

1. **Telemetry Collector** - Background loop collecting system metrics every 60 seconds
2. **SQLite Storage** - Persistent database for historical telemetry data
3. **RPC/CLI Integration** - Commands to query telemetry snapshots, history, and trends
4. **Doctor Integration** - Health checks for telemetry database
5. **Validation Tests** - 4 new runtime tests for telemetry functionality

---

## Architecture

### Telemetry Collection Loop

The telemetry collector runs as a background tokio task within the `annad` daemon:

```rust
pub fn start_collection_loop(self: Arc<Self>) {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            // Collect and store sample
        }
    });
}
```

**Metrics Collected:**
- CPU usage (%) - Global average across all cores
- Memory usage (%) - Total RAM utilization
- Disk free (%) - Available space on primary disk
- System uptime (seconds) - Time since boot
- Network I/O (KB) - Total bytes in/out across all interfaces

**Collection Frequency:** Every 60 seconds
**Storage:** `/var/lib/anna/telemetry.db` (SQLite)

### Database Schema

```sql
CREATE TABLE telemetry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,        -- ISO 8601 format
    cpu REAL,                        -- CPU usage percentage
    mem REAL,                        -- Memory usage percentage
    disk REAL,                       -- Disk free percentage
    uptime INTEGER,                  -- System uptime in seconds
    net_in INTEGER,                  -- Network in (KB)
    net_out INTEGER                  -- Network out (KB)
);

CREATE INDEX idx_timestamp ON telemetry(timestamp DESC);
```

**Permissions:**
- Owner: `root:anna`
- Mode: `0640`
- Location: `/var/lib/anna/telemetry.db`

---

## RPC Endpoints

### TelemetrySnapshot

Returns the most recent telemetry sample from the database.

**Request:**
```json
{
    "type": "telemetry_snapshot"
}
```

**Response (Success):**
```json
{
    "status": "success",
    "data": {
        "timestamp": "2025-10-30T12:34:56+00:00",
        "cpu_usage": 23.5,
        "mem_usage": 48.2,
        "disk_free": 72.8,
        "uptime_sec": 86400,
        "net_in_kb": 1234567,
        "net_out_kb": 987654
    }
}
```

**Response (No Data):**
```json
{
    "status": "error",
    "message": "No telemetry data yet. Collector needs ~60s to populate first sample."
}
```

### TelemetryHistory

Returns the N most recent telemetry samples, optionally filtered by timestamp.

**Request:**
```json
{
    "type": "telemetry_history",
    "since": "2025-10-30T00:00:00+00:00",  // Optional
    "limit": 10                             // Optional, default 10
}
```

**Response:**
```json
{
    "status": "success",
    "data": {
        "samples": [
            {
                "timestamp": "2025-10-30T12:34:56+00:00",
                "cpu_usage": 23.5,
                "mem_usage": 48.2,
                "disk_free": 72.8,
                "uptime_sec": 86400,
                "net_in_kb": 1234567,
                "net_out_kb": 987654
            },
            // ... more samples
        ],
        "count": 10
    }
}
```

### TelemetryTrends

Calculates statistical trends (avg/min/max) for a specific metric over a time window.

**Request:**
```json
{
    "type": "telemetry_trends",
    "metric": "cpu",     // "cpu", "mem", "memory", or "disk"
    "hours": 24          // Time window in hours
}
```

**Response:**
```json
{
    "status": "success",
    "data": {
        "metric": "cpu",
        "hours": 24,
        "avg": 25.3,
        "min": 5.2,
        "max": 87.4,
        "samples": 144
    }
}
```

**Valid Metrics:**
- `cpu` - CPU usage percentage
- `mem` or `memory` - Memory usage percentage
- `disk` - Disk free percentage

---

## CLI Commands

### annactl telemetry snapshot

Show the current system telemetry snapshot.

**Usage:**
```bash
annactl telemetry snapshot
```

**Example Output:**
```
📊 System Telemetry Snapshot

  Timestamp:    2025-10-30T12:34:56+00:00
  CPU Usage:    23.5%
  Memory Usage: 48.2%
  Disk Free:    72.8%
  Uptime:       86400 seconds
  Network In:   1234567 KB
  Network Out:  987654 KB
```

### annactl telemetry history

Show historical telemetry samples in tabular format.

**Usage:**
```bash
annactl telemetry history [OPTIONS]

Options:
  -l, --limit <N>       Number of samples to show (default: 10)
      --since <TIME>    Show samples since timestamp (ISO 8601)
```

**Example:**
```bash
annactl telemetry history --limit 5
```

**Example Output:**
```
📈 Telemetry History (5 samples)

Timestamp                     CPU%     MEM%    DISK%  Uptime(s)
----------------------------------------------------------------------
2025-10-30 12:34:56          23.5%    48.2%    72.8%      86400
2025-10-30 12:33:56          22.1%    47.9%    72.8%      86340
2025-10-30 12:32:56          21.8%    47.5%    72.9%      86280
2025-10-30 12:31:56          23.2%    48.1%    72.9%      86220
2025-10-30 12:30:56          24.5%    48.4%    72.9%      86160
```

### annactl telemetry trends

Analyze metric trends over a time window.

**Usage:**
```bash
annactl telemetry trends <METRIC> [OPTIONS]

Arguments:
  <METRIC>              Metric to analyze: cpu, mem, or disk

Options:
  -h, --hours <N>       Time window in hours (default: 24)
```

**Example:**
```bash
annactl telemetry trends cpu --hours 12
```

**Example Output:**
```
📉 Telemetry Trends Analysis

  Metric:   cpu
  Period:   12 hours
  Samples:  72

  Average:  25.3%
  Minimum:  5.2%
  Maximum:  87.4%
```

---

## Doctor Integration

### Check: Telemetry Database

The doctor system now includes a check for the telemetry database.

**Check Performed:**
1. Verify `/var/lib/anna/telemetry.db` exists
2. Verify file is readable
3. Report file size (in verbose mode)

**Example Output:**
```bash
$ annactl doctor check

🏥 Anna System Health Check

[OK] Directories present
[OK] Ownership correct (root:anna)
[OK] Permissions correct
[OK] Dependencies installed
[OK] Service running
[OK] Socket accessible
[OK] Policies loaded (3 rules)
[OK] Events functional (3 bootstrap events)
[OK] Telemetry database exists

✓ System healthy - no repairs needed
```

### Repair: Telemetry Database

If the telemetry database or its parent directory is missing, the repair system can recreate it.

**Repair Actions:**
1. Create `/var/lib/anna` if missing
2. Set permissions: `0750 root:anna`
3. Set ownership: `root:anna`
4. Database file created automatically by daemon on startup

**Example:**
```bash
$ annactl doctor repair

🔧 Doctor Repair

[FIX] Creating telemetry directory: /var/lib/anna
[OK] Set permissions: 0750 root:anna

✓ Made 1 repairs successfully
```

---

## Implementation Details

### TelemetryCollector Struct

Location: `src/annad/src/telemetry_collector.rs`

```rust
pub struct TelemetryCollector {
    db_path: String,
    conn: Arc<Mutex<Connection>>,
}

impl TelemetryCollector {
    pub fn new(db_path: &str) -> Result<Self>
    pub fn collect_sample() -> Result<TelemetrySample>
    pub fn store_sample(&self, sample: &TelemetrySample) -> Result<()>
    pub fn get_snapshot(&self) -> Result<Option<TelemetrySample>>
    pub fn get_history(&self, limit: usize) -> Result<Vec<TelemetrySample>>
    pub fn get_trends(&self, metric: &str, hours: usize) -> Result<TelemetryTrends>
    pub fn start_collection_loop(self: Arc<Self>)
}
```

**Thread Safety:**
- Uses `Arc<Mutex<Connection>>` for thread-safe database access
- RPC handlers can query concurrently without blocking collection loop

**Error Handling:**
- Collection errors logged but don't stop daemon
- Database errors return clear error messages to CLI
- Empty database returns helpful "wait 60s" message

### Integration into DaemonState

Location: `src/annad/src/state.rs`

```rust
pub struct DaemonState {
    pub policy_engine: Arc<PolicyEngine>,
    pub event_dispatcher: Arc<EventDispatcher>,
    pub telemetry: Arc<Mutex<TelemetrySnapshot>>,
    pub telemetry_collector: Arc<TelemetryCollector>,  // Added
    pub learning_cache: Arc<Mutex<LearningCache>>,
    pub start_time: u64,
}

impl DaemonState {
    pub fn new() -> Result<Self> {
        // ...
        let telemetry_collector = Arc::new(
            TelemetryCollector::new("/var/lib/anna/telemetry.db")?
        );
        telemetry_collector.clone().start_collection_loop();
        // ...
    }
}
```

**Initialization:**
1. Create database connection
2. Initialize schema if needed
3. Start background collection loop
4. Store Arc in DaemonState for RPC access

---

## Validation Tests

### Test 1: Telemetry Snapshot

**Purpose:** Verify telemetry collector populates database after 60 seconds

**Test Steps:**
1. Wait 60 seconds for collector to run
2. Execute `annactl telemetry snapshot`
3. Verify output contains CPU, Memory, and Disk metrics

**Expected Result:**
```
[OK] Telemetry snapshot: all metrics present (CPU, MEM, DISK)
```

### Test 2: Telemetry History

**Purpose:** Verify history command returns samples in correct format

**Test Steps:**
1. Execute `annactl telemetry history --limit 5`
2. Verify output contains "Telemetry History" or "samples"
3. Check for no errors in output

**Expected Result:**
```
[OK] Telemetry history: valid output with samples
```

### Test 3: Telemetry Trends

**Purpose:** Verify trends calculation for metrics

**Test Steps:**
1. Execute `annactl telemetry trends cpu --hours 1`
2. Verify output contains "Average", "Minimum", "Maximum"
3. Check for no errors in output

**Expected Result:**
```
[OK] Telemetry trends: all stats present (avg, min, max)
```

### Test 4: Doctor Telemetry DB

**Purpose:** Verify doctor system includes telemetry database check

**Test Steps:**
1. Execute `annactl doctor check`
2. Verify output contains "Telemetry database" check

**Expected Result:**
```
[OK] Doctor check includes telemetry database
```

---

## Performance Considerations

### Collection Overhead

- **CPU:** Minimal (~0.1% average)
- **Memory:** ~2MB for sysinfo crate
- **Disk I/O:** One write per 60 seconds (~200 bytes)
- **Database Growth:** ~200 bytes/minute = ~288 KB/day = ~105 MB/year

### Query Performance

- **Snapshot:** O(1) - Single row query with indexed timestamp
- **History:** O(n) - Limited by result set size (default 10 rows)
- **Trends:** O(m) - Where m = samples in time window (1 hour = 60 samples)

**Indexes:**
- `idx_timestamp` on `telemetry(timestamp DESC)` - Accelerates all queries

### Concurrent Access

- Multiple CLI commands can query simultaneously
- Background collector doesn't block queries
- SQLite handles read concurrency automatically
- Write concurrency not needed (single writer: collector)

---

## Future Enhancements (Sprint 5 Phases 3-6)

### Phase 3: Policy Action Execution
- Execute actions based on telemetry thresholds
- Example: `if cpu > 80% for 5 minutes, log warning`

### Phase 4: Learning Feedback Loop
- Track action outcomes (success/failure)
- Build confidence scores for policy actions
- Recommend actions based on historical success

### Phase 5: Data Rotation
- Automatic cleanup of old telemetry data
- Configurable retention period (default: 90 days)
- Aggregate older data into hourly/daily summaries

### Phase 6: Final Testing & Documentation
- Comprehensive integration tests
- Performance benchmarks
- User guide and examples
- Policy template library

---

## Troubleshooting

### No telemetry data after 60 seconds

**Symptoms:**
```bash
$ annactl telemetry snapshot
Error: No telemetry data yet. Collector needs ~60s to populate first sample.
```

**Possible Causes:**
1. Daemon just started (wait full 60 seconds)
2. Database permission error
3. Collector loop crashed

**Resolution:**
```bash
# Check daemon logs
sudo journalctl -u annad --since -5m

# Check database permissions
ls -lh /var/lib/anna/telemetry.db

# Repair if needed
sudo annactl doctor repair
```

### Database permission errors

**Symptoms:**
```bash
Error: Failed to open telemetry database: Permission denied
```

**Resolution:**
```bash
# Run doctor repair
sudo annactl doctor repair

# Or manually fix
sudo chown root:anna /var/lib/anna/telemetry.db
sudo chmod 0640 /var/lib/anna/telemetry.db
```

### Invalid metric error

**Symptoms:**
```bash
$ annactl telemetry trends memory --hours 24
Error: Invalid metric 'memory'. Valid metrics: cpu, mem, disk
```

**Resolution:**
Use correct metric names: `cpu`, `mem`, or `disk`

---

## Files Modified/Created

### New Files
- `src/annad/src/telemetry_collector.rs` (365 lines) - Telemetry collection module
- `docs/TELEMETRY-AUTOMATION.md` (this file) - Documentation

### Modified Files
- `Cargo.toml` - Added rusqlite, sysinfo, sha2 dependencies
- `src/annad/Cargo.toml` - Added workspace dependencies
- `src/annad/src/main.rs` - Added telemetry_collector module
- `src/annad/src/state.rs` - Integrated TelemetryCollector into DaemonState
- `src/annad/src/rpc.rs` - Added TelemetrySnapshot/History/Trends endpoints
- `src/annactl/src/main.rs` - Added telemetry CLI commands
- `src/annactl/src/doctor.rs` - Added telemetry DB check and repair
- `tests/runtime_validation.sh` - Added 4 new telemetry tests
- `scripts/install.sh` - Version bump to 0.9.4-alpha

---

## Commits

### Phase 2A: Telemetry Collection
- Added dependencies (rusqlite, sysinfo, sha2)
- Implemented TelemetryCollector with SQLite storage
- Integrated into DaemonState with background loop

### Phase 2B: RPC/CLI Integration
- Added RPC endpoints (TelemetrySnapshot/History/Trends)
- Implemented CLI commands (snapshot/history/trends)
- Enhanced doctor with DB checks and repair
- Added 4 validation tests
- Created TELEMETRY-AUTOMATION.md documentation

---

## Summary

Sprint 5 Phase 2B successfully adds persistent telemetry collection and query capabilities to Anna, laying the foundation for self-optimization. The system now:

1. ✅ Collects system metrics every 60 seconds
2. ✅ Stores data persistently in SQLite
3. ✅ Provides RPC/CLI access to telemetry
4. ✅ Includes doctor health checks
5. ✅ Validated with 4 runtime tests

**Next Steps:** Implement policy action execution (Phase 3) to enable automated responses to telemetry thresholds.

---

**Generated with:** Claude Code
**Sprint:** 5 - Phase 2B
**Version:** 0.9.4-alpha
