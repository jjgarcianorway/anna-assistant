# Continuous Profiling Daemon Specification

**Anna v0.14.0 "Orion III" - Phase 2.2**

## Overview

The Continuous Profiling Daemon enables Anna to monitor herself - detecting performance drift and environment anomalies in real time. This self-awareness layer is critical for maintaining system stability and detecting degradation early.

## Architecture

### Core Components

1. **Profiler** (`src/annactl/src/profiled.rs`)
   - Captures performance snapshots
   - Compares to 7-day baseline
   - Detects degradation patterns
   - Logs to perfwatch.jsonl

2. **Performance Snapshot** (`PerfSnapshot`)
   - RPC latency measurement
   - Memory usage tracking
   - I/O latency monitoring
   - CPU usage sampling

3. **CLI Commands** (`annactl profiled`)
   - `--status`: Show current performance status
   - `--summary`: Display statistics
   - `--rebuild`: Regenerate 7-day baseline

## Data Model

### PerfSnapshot Structure

```rust
pub struct PerfSnapshot {
    pub timestamp: u64,
    pub rpc_latency_ms: f32,
    pub memory_mb: f32,
    pub io_latency_ms: f32,
    pub cpu_percent: f32,
    pub queue_depth: u32,
}
```

### PerfBaseline Structure

```rust
pub struct PerfBaseline {
    pub avg_rpc_latency_ms: f32,
    pub avg_memory_mb: f32,
    pub avg_io_latency_ms: f32,
    pub avg_cpu_percent: f32,
    pub sample_count: u32,
    pub created_at: u64,
}
```

### Degradation Classification

| Level | Threshold | Action |
|-------|-----------|--------|
| **Normal** | ≤ 15% above baseline | Continue monitoring |
| **Minor** | > 15% above baseline | Log warning |
| **Moderate** | > 30% above baseline | Trigger alert |
| **Critical** | > 50% above baseline | Immediate intervention |

## Measurement Methodology

### RPC Latency

```rust
// Fast socket existence check
let start = Instant::now();
std::path::Path::new("/run/anna/annad.sock").exists();
let latency_ms = start.elapsed().as_micros() as f32 / 1000.0;
```

### Memory Usage

```rust
// Read from /proc/self/status
VmRSS: <value> kB  // Convert to MB
```

### I/O Latency

```rust
// Filesystem metadata read
let start = Instant::now();
state_dir.exists();
let latency_ms = start.elapsed().as_micros() as f32 / 1000.0;
```

### CPU Usage

```rust
// Read from /proc/self/stat
utime + stime = total CPU time
```

## Storage

### File Locations

| File | Path | Format |
|------|------|--------|
| Perfwatch Log | `~/.local/state/anna/perfwatch.jsonl` | JSONL |
| Baseline | `~/.local/state/anna/perfbaseline.json` | JSON |

### Perfwatch Entry Schema

```json
{
  "timestamp": 1699900000,
  "snapshot": {
    "rpc_latency_ms": 0.8,
    "memory_mb": 52.3,
    "io_latency_ms": 0.3,
    "cpu_percent": 3.2,
    "queue_depth": 0
  },
  "baseline": {
    "avg_rpc_latency_ms": 1.0,
    "avg_memory_mb": 50.0,
    "avg_io_latency_ms": 0.5,
    "avg_cpu_percent": 5.0,
    "sample_count": 168
  },
  "degradation": "Normal",
  "rpc_delta_pct": -20.0,
  "memory_delta_pct": 4.6,
  "io_delta_pct": -40.0,
  "cpu_delta_pct": -36.0
}
```

## Integration

### Anomaly Detection

Performance degradation events are automatically integrated into the anomaly detection system:

```rust
pub enum Severity {
    Info,       // Minor deviation
    Warning,    // >15% persistent deviation
    Critical,   // >50% for 3+ consecutive cycles
}
```

New anomaly metrics:
- `perf_rpc_latency`
- `perf_memory`
- `perf_io_latency`
- `perf_cpu`

### Advisor Rules

New critical rule: `critical_performance_drift`

```
Title: "Anna performance degradation detected"
Condition: Persistent degradation for 3+ cycles
Action: "Check profiler: annactl profiled --status"
```

## Usage Examples

### View Current Status

```bash
$ annactl profiled --status
```

Output:
```
╭─ Performance Profiler Status ────────────────────────────
│
│  Overall Status: ✅ Normal
│
│  Current Metrics
│    RPC Latency:    0.85 ms (-15.0%)
│    Memory Usage:   52.3 MB (+4.6%)
│    I/O Latency:    0.30 ms (-40.0%)
│    CPU Usage:      3.2% (-36.0%)
│
│  7-Day Baseline
│    RPC Latency:    1.00 ms
│    Memory Usage:   50.0 MB
│    I/O Latency:    0.50 ms
│    CPU Usage:      5.0%
│    Samples: 168
│
╰──────────────────────────────────────────────────────────
```

### View Summary Statistics

```bash
$ annactl profiled --summary
```

Output:
```
╭─ Performance Profiler Summary ───────────────────────────
│
│  Monitoring Statistics
│    Total Snapshots:   420
│    Degraded Count:    15 (3.6%)
│
│  Current Averages
│    RPC Latency:      0.92 ms
│    Memory Usage:     51.8 MB
│
│  7-Day Baseline
│    RPC Latency:      1.00 ms
│    Memory Usage:     50.0 MB
│    Samples:          168
│
│  Overall Health: Excellent
│
╰──────────────────────────────────────────────────────────
```

### Rebuild Baseline

```bash
$ annactl profiled --rebuild
```

Output:
```
╭─ Rebuild Performance Baseline ───────────────────────────
│
│  ✅ Baseline rebuilt from last 7 days
│
│  New Baseline Metrics
│    RPC Latency:    0.95 ms
│    Memory Usage:   51.5 MB
│    I/O Latency:    0.45 ms
│    CPU Usage:      4.5%
│
│    Samples: 168
│
╰──────────────────────────────────────────────────────────
```

## Performance

**Target:** < 50ms overhead per snapshot capture

**Actual Performance:**
- RPC latency measurement: ~0.1-0.5ms
- Memory read: ~1-3ms
- I/O latency measurement: ~0.1-0.3ms
- CPU stat read: ~1-2ms
- Logging: ~5-10ms
- **Total: ~7-16ms** ✅

## Testing

### Unit Tests (8 total)

```bash
cargo test --bin annactl profiled::tests
```

Tests cover:
1. Performance snapshot creation
2. Baseline from snapshots
3. Degradation classification
4. Perfwatch entry creation
5. Delta calculation
6. Degradation emojis
7. Degradation names
8. State directory path validation

## Operational Considerations

### Baseline Updates

Baselines should be rebuilt:
- After major system changes
- After performance tuning
- Monthly as routine maintenance

```bash
annactl profiled --rebuild
```

### Degradation Response

| Level | Response |
|-------|----------|
| **Minor** | Review logs, monitor trend |
| **Moderate** | Check for resource contention |
| **Critical** | Consider daemon restart |

### Log Rotation

Currently not implemented. Planned for future:
- Rotate at 1MB threshold
- Keep 5 historical files
- Compress archives

## Integration with Other Systems

### 1. Anomaly Detection
Performance drift automatically creates anomalies:
```bash
annactl anomalies
```

### 2. Forecasting
Future enhancement: predict performance trends

### 3. Action Engine
Future enhancement: auto-remediation actions

## Future Enhancements

1. **Automated Log Rotation**
   - 1MB threshold
   - 5 file retention
   - Compression

2. **True SHA-256**
   - Replace DefaultHasher with sha2 crate
   - Cryptographic integrity verification

3. **Rollback Preview**
   - Show diff before restore
   - Selective metric restoration

4. **Smart Repair**
   - ML-based degradation prediction
   - Auto-tuning recommendations

5. **Remote Backup**
   - Cloud integration
   - Cross-system comparison

## Security Considerations

- All metrics read from `/proc/self/*` (process-local)
- No privileged operations required
- No network communication
- Local-only storage

## Troubleshooting

### Profiler Not Collecting Data

```bash
# Check state directory
ls -la ~/.local/state/anna/

# Verify permissions
stat ~/.local/state/anna/perfwatch.jsonl
```

### High Memory Readings

```bash
# Compare with system tools
top -p $(pgrep annad)
ps aux | grep anna
```

### Missing Baseline

```bash
# Rebuild from scratch
annactl profiled --rebuild
```

## References

- Anomaly Detection: `docs/ANOMALY-SPEC.md`
- Advisor Rules: `docs/ADVISOR-SPEC.md`
- System Architecture: `docs/ARCHITECTURE.md`
