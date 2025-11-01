# Anna v0.12.0 - Radar Scoring System

## Overview

Anna v0.12.0 introduces **radar-based user classification** through three independent scoring systems:

1. **System Health Radar** - Hardware and resource health
2. **Usage Habit Radar** - User behavior patterns
3. **Network Posture Radar** - Network connectivity quality

Each radar scores multiple categories on a 0-10 scale. Missing metrics are reported as `null`/N/A rather than failing.

---

## 1. System Health Radar

Assesses the overall health of system resources.

### Categories

| Category | Score Formula | Data Source |
|----------|--------------|-------------|
| **cpu_load** | 10 at `<0.5 per core`<br>Linear to 0 at `≥2.0 per core` | `/proc/loadavg` |
| **memory_pressure** | 10 at `≥40% free`<br>Linear to 0 at `≤5% free` | `/proc/meminfo` (MemAvailable) |
| **disk_headroom** | 10 at `≥30% free on /`<br>Linear to 0 at `≤5% free` | `df` command |
| **thermal_ok** | 10 at `≤70°C`<br>Linear to 0 at `≥90°C` | `sensors` or `/sys/class/thermal` |

### Examples

```bash
# View current system health radar
annactl radar show

# JSON output
annactl radar show --json
```

**Output (human-readable):**
```
╭─ User Classification ────────────────────────
│
│  User:        lhoqvso
│  UID:         1000
│
│  System Health Radar:
│    cpu_load             [▓▓▓▓▓▓▓▓▓▓▓▓▓▓░]  9.2/10
│    memory_pressure      [▓▓▓▓▓▓▓▓░░░░░░░]  5.8/10
│    disk_headroom        [▓▓▓▓▓▓▓▓▓▓▓▓▓░░]  8.5/10
│    thermal_ok           [▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓] 10.0/10
│
╰──────────────────────────────────────────────
```

---

## 2. Usage Habit Radar

Measures user interaction patterns and work habits.

### Categories

| Category | Score Formula | Data Source |
|----------|--------------|-------------|
| **interactive_time** | 10 at `≥2h in last 24h`<br>Linear to 0 at `0h` | User login sessions |
| **cpu_bursty** | 10 at variance `0.0`<br>Linear to 0 at variance `≥1.0` | Per-minute CPU share over 2h |
| **work_window_regular** | 10 at stddev `0h`<br>Linear to 0 at stddev `≥4h` | Login times over last 7d |

### Examples

```bash
# Classify current user
annactl classify run

# Classify specific user
annactl classify run --user alice

# JSON output
annactl classify run --json
```

**JSON Schema:**
```json
{
  "uid": 1000,
  "username": "lhoqvso",
  "radars": {
    "usage_habit": {
      "name": "Usage Habit",
      "categories": {
        "interactive_time": {
          "category": "interactive_time",
          "score": 8.5,
          "max": 10.0,
          "description": "1.7h interactive in last 24h"
        },
        "cpu_bursty": {
          "category": "cpu_bursty",
          "score": null,
          "max": 10.0,
          "description": "Insufficient data"
        }
      },
      "overall_score": 8.5
    }
  },
  "timestamp": 1730428800
}
```

---

## 3. Network Posture Radar

Evaluates network connectivity quality.

### Categories

| Category | Score Formula | Data Source |
|----------|--------------|-------------|
| **latency** | 10 at `≤20ms`<br>Linear to 0 at `≥250ms` | `ping -c1 8.8.8.8` |
| **loss** | 10 at `0% loss`<br>Linear to 0 at `≥10% loss` | Ping packet loss |
| **dns_reliability** | 10 if successful<br>0 if failed | `getent hosts` / `dig` |

### Examples

```bash
# Full classification including network
annactl classify run

# View all radars with JSON
annactl classify run --json | jq '.radars.network_posture'
```

**Output:**
```json
{
  "name": "Network Posture",
  "categories": {
    "latency": {
      "category": "latency",
      "score": 9.8,
      "max": 10.0,
      "description": "12.3ms ping latency"
    },
    "loss": {
      "category": "loss",
      "score": 10.0,
      "max": 10.0,
      "description": "0.0% packet loss"
    },
    "dns_reliability": {
      "category": "dns_reliability",
      "score": 10.0,
      "max": 10.0,
      "description": "DNS: OK"
    }
  },
  "overall_score": 9.9
}
```

---

## Storage

Radar scores are persisted in `/var/lib/anna/telemetry.db`:

```sql
-- Table: radar_scores
CREATE TABLE radar_scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    uid INTEGER NOT NULL,
    radar TEXT NOT NULL,           -- 'system_health', 'usage_habit', 'network_posture'
    category TEXT NOT NULL,         -- e.g. 'cpu_load', 'latency'
    score REAL,                     -- NULL if metric unavailable
    max REAL NOT NULL DEFAULT 10.0,
    FOREIGN KEY (uid) REFERENCES users(uid) ON DELETE CASCADE
);
```

### Query Latest Scores

```sql
SELECT radar, category, score, max, ts
FROM radar_scores
WHERE uid = 1000
  AND ts = (SELECT MAX(ts) FROM radar_scores WHERE uid = 1000)
ORDER BY radar, category;
```

---

## Implementation Notes

### Missing Metrics

When a metric is unavailable (e.g., no battery, no temperature sensor), the category score is set to `null`:

- **Human output**: Displays as `N/A`
- **JSON output**: `"score": null`
- **Overall score**: Computed only from available categories

### Pure Functions

All scoring functions are deterministic and testable:

```rust
use anna::radars_v12::score_system_health;

let result = score_system_health(
    0.8,    // load_avg_1m
    8,      // num_cores
    45.0,   // mem_free_pct
    35.0,   // root_free_pct
    Some(68.0) // cpu_temp_c
);

assert!(result.overall_score > 9.0);
```

### Update Frequency

Radars are computed on-demand:

- **Manual**: `annactl classify run`
- **Scheduled**: Via cron or systemd timer (not included in v0.12.0)
- **Event-driven**: After significant system changes (future)

---

## Integration

### RPC API

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "classify",
  "params": { "user": "alice" },
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "uid": 1001,
    "username": "alice",
    "radars": { ... },
    "timestamp": 1730428800
  },
  "id": 1
}
```

---

## Future Enhancements

1. **Historical Trending**: Track radar changes over time
2. **Anomaly Detection**: Alert on sudden radar score drops
3. **Persona Inference**: Map radar patterns to user personas
4. **Adaptive Thresholds**: Adjust scoring based on system baseline

---

## See Also

- [CLI Reference](CLI_REFERENCE.md) - Complete command documentation
- [Troubleshooting](TROUBLESHOOTING.md) - Common issues and fixes
- [Architecture](AUTONOMY-ARCHITECTURE.md) - System design overview
