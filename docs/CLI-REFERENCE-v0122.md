# Anna v0.12.2 CLI Reference

## New Commands

### `annactl collect`

Collect and display telemetry snapshots.

**Usage:**
```bash
annactl collect [OPTIONS]
```

**Options:**
- `--limit <N>`: Number of snapshots to retrieve (default: 1)
- `--json`: Output as JSON

**Examples:**
```bash
# Human-readable output
annactl collect --limit 1

# JSON output
annactl collect --limit 1 --json

# Multiple snapshots
annactl collect --limit 5 --json | jq '.snapshots | length'
```

**JSON Schema:**
```json
{
  "snapshots": [
    {
      "ts": 1234567890,
      "sensors": {
        "cpu": {
          "load_avg": [1.2, 1.5, 1.8],
          "cores": [...]
        },
        "mem": {
          "total_mb": 16384,
          "used_mb": 8192,
          ...
        },
        "power": { ... }
      },
      "net": {
        "interfaces": [...],
        "default_route": "...",
        "dns_latency_ms": 12.3
      },
      "disk": {
        "disks": [...]
      },
      "top": {
        "by_cpu": [...],
        "by_mem": [...]
      }
    }
  ],
  "count": 1,
  "limit": 1
}
```

---

### `annactl classify`

Classify the system persona (laptop, workstation, server, vm).

**Usage:**
```bash
annactl classify [OPTIONS]
```

**Options:**
- `--json`: Output as JSON

**Examples:**
```bash
# Human-readable output
annactl classify

# JSON output
annactl classify --json
```

**JSON Schema:**
```json
{
  "persona": "laptop",
  "confidence": 0.9,
  "evidence": [
    "Battery detected",
    "4 CPU cores",
    "2 network interfaces"
  ],
  "radars": {
    "system_health": {
      "name": "System Health",
      "categories": {
        "cpu_load": {
          "category": "cpu_load",
          "score": 8.5,
          "max": 10.0,
          "description": "Load 0.53 per core (4 cores)"
        },
        ...
      },
      "overall_score": 8.2
    },
    "network_posture": {
      ...
    }
  }
}
```

---

### `annactl radar show`

Display radar scores for system health and network posture.

**Usage:**
```bash
annactl radar show [OPTIONS]
```

**Options:**
- `--json`: Output as JSON

**Examples:**
```bash
# Human-readable output
annactl radar show

# JSON output
annactl radar show --json

# Extract specific score
annactl radar show --json | jq '.health.overall_score'
```

**JSON Schema:**
```json
{
  "health": {
    "name": "System Health",
    "categories": {
      "cpu_load": {
        "category": "cpu_load",
        "score": 8.5,
        "max": 10.0,
        "description": "Load 0.53 per core (4 cores)"
      },
      "memory_pressure": {
        "category": "memory_pressure",
        "score": 7.2,
        "max": 10.0,
        "description": "28.0% memory free"
      },
      "disk_headroom": {
        "category": "disk_headroom",
        "score": 6.8,
        "max": 10.0,
        "description": "32.0% disk free on /"
      },
      "thermal_ok": {
        "category": "thermal_ok",
        "score": 9.0,
        "max": 10.0,
        "description": "CPU temp 62°C"
      }
    },
    "overall_score": 7.9
  },
  "network": {
    "name": "Network Posture",
    "categories": {
      "latency": {
        "category": "latency",
        "score": 9.5,
        "max": 10.0,
        "description": "12.3ms ping latency"
      },
      "loss": null,
      "dns_reliability": {
        "category": "dns_reliability",
        "score": 10.0,
        "max": 10.0,
        "description": "DNS: OK"
      }
    },
    "overall_score": 9.7
  },
  "overall": {
    "health_score": 7.9,
    "network_score": 9.7,
    "combined": 8.8
  }
}
```

---

## Existing Commands (Unchanged)

All v0.12.1 commands remain functional:

- `annactl version`
- `annactl status [--json] [--verbose]`
- `annactl sensors [--json] [--detail]`
- `annactl net [--json] [--detail]`
- `annactl disk [--json] [--detail]`
- `annactl top [--json] [--limit N]`
- `annactl events [--json] [--since WINDOW] [--limit N]`
- `annactl export [--path FILE] [--since WINDOW]`
- `annactl doctor {pre|post|repair} [--json] [--yes]`

---

## Exit Codes

- `0`: Success
- `1`: General error
- `7`: Timeout (daemon not responding)

---

## Tips

### Piping to jq

```bash
# Pretty-print JSON
annactl collect --json | jq '.'

# Extract specific fields
annactl classify --json | jq '.persona'

# Filter arrays
annactl collect --json | jq '.snapshots[0].sensors.cpu.load_avg'
```

### Watch Mode (using `watch`)

```bash
# Monitor radar scores every 5 seconds
watch -n 5 'annactl radar show'

# Monitor collection
watch -n 10 'annactl collect --limit 1'
```

### Logging

Combine with system logs for debugging:

```bash
annactl collect --json 2>&1 | tee collect.log
```

---

## Troubleshooting

### "Failed to connect to annad"

```bash
# Check if daemon is running
sudo systemctl status annad

# Check socket
ls -la /run/anna/annad.sock

# View logs
sudo journalctl -u annad -n 50
```

### "timeout (connect/write/read)"

Daemon is hung or overloaded. Check logs:

```bash
sudo journalctl -u annad --since "5 minutes ago"
```

### "Method not found: collect"

Daemon is running old version. Restart after deploying new binary:

```bash
sudo systemctl restart annad
```
