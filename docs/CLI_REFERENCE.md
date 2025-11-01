# Anna v0.12.0 - CLI Reference

Complete command-line interface reference for `annactl`.

---

## Global Options

All commands support:
- `--help` - Show command-specific help
- `--version` - Show Anna version (root level only)

---

## Commands

### `annactl version`

Show version information.

```bash
annactl version
```

**Output:**
```
Anna v0.12.0 - Event-Driven Intelligence
Build: annactl 0.12.0
```

**Exit Code:** `0`

---

### `annactl status`

Show daemon status and health.

**Syntax:**
```bash
annactl status [--json] [--verbose]
```

**Flags:**
- `--json` - Output as JSON
- `-v, --verbose` - Show detailed status (socket path, uptime)

**Examples:**
```bash
# Basic status
annactl status

# Verbose output
annactl status --verbose

# JSON output
annactl status --json
```

**Human Output:**
```
╭─ Anna Status ────────────────────────────────
│
│  Daemon:       running
│  DB Path:      /var/lib/anna/telemetry.db
│  Last Sample:  5 seconds ago
│  Sample Count: 1234
│  Loop Load:    2.3%
│
│  Process ID:   699
│
╰──────────────────────────────────────────────
```

**JSON Schema:**
```json
{
  "daemon_state": "running",
  "db_path": "/var/lib/anna/telemetry.db",
  "last_sample_age_s": 5,
  "sample_count": 1234,
  "loop_load_pct": 2.3,
  "annad_pid": 699,
  "socket_path": "/run/anna/annad.sock",
  "uptime_secs": 86400
}
```

**Exit Codes:**
- `0` - Daemon running
- `1` - Connection failed

---

### `annactl sensors`

Show CPU, memory, temperatures, and battery.

**Syntax:**
```bash
annactl sensors [--json] [--detail]
```

**Flags:**
- `--json` - Output as JSON
- `-d, --detail` - Show detailed sensor information

**Examples:**
```bash
# Basic sensors
annactl sensors

# Detailed view
annactl sensors --detail

# JSON output
annactl sensors --json
```

**Human Output:**
```
╭─ System Sensors ─────────────────────────────
│
│  CPU
│    Core 0: ██████░░░░░░░░░░░░░░   30.5% 68°C
│    Core 1: ████████░░░░░░░░░░░░   40.2% 70°C
│    Load: 0.85, 0.92, 1.05
│
│  Memory: ████████████░░░░░░░░  60.3%  (9.7/16.0 GB)
│  Swap:   ██░░░░░░░░░░░░░░░░░░  10.5%  (0.8 GB)
│
│  Battery: 🔋 85%  (Discharging)
│           12.3W
│
╰──────────────────────────────────────────────
```

**JSON Schema:**
```json
{
  "cpu": {
    "load_avg": [0.85, 0.92, 1.05],
    "cores": [
      {"core": 0, "util_pct": 30.5, "temp_c": 68.0}
    ]
  },
  "mem": {
    "total_mb": 16384,
    "used_mb": 9883,
    "free_mb": 6501,
    "swap_total_mb": 8192,
    "swap_used_mb": 860
  },
  "power": {
    "percent": 85,
    "status": "Discharging",
    "power_now_w": 12.3
  }
}
```

---

### `annactl net`

Show network interfaces and connectivity.

**Syntax:**
```bash
annactl net [--json] [--detail]
```

**Flags:**
- `--json` - Output as JSON
- `-d, --detail` - Show detailed network information (IP addresses, link speed)

**Examples:**
```bash
# Basic network status
annactl net

# Detailed view with IPs
annactl net --detail

# JSON output
annactl net --json
```

**Human Output:**
```
╭─ Network Interfaces ─────────────────────────
│
│  ● wlan0          up
│     ↓    1234.5 KB/s  ↑      89.2 KB/s
│     IPv4: 192.168.1.xxx
│
│  ○ eth0           down
│     ↓       0.0 KB/s  ↑       0.0 KB/s
│
│  Default Route: via 192.168.1.1 dev wlan0
│
╰──────────────────────────────────────────────
```

---

### `annactl disk`

Show disk usage and SMART status.

**Syntax:**
```bash
annactl disk [--json] [--detail]
```

**Flags:**
- `--json` - Output as JSON
- `-d, --detail` - Show inode usage and SMART status

**Examples:**
```bash
# Basic disk usage
annactl disk

# Detailed view with SMART
annactl disk --detail

# JSON output
annactl disk --json
```

**Human Output:**
```
╭─ Disk Usage ─────────────────────────────────
│
│  /
│    ████████████████░░░░  75.2%  (451.2/600.0 GB)
│    Device: /dev/nvme0n1p2
│
│  /home
│    ███████░░░░░░░░░░░░░  35.8%  (358.4/1000.0 GB)
│    Device: /dev/sda1
│
╰──────────────────────────────────────────────
```

---

### `annactl top`

Show top processes by CPU and memory.

**Syntax:**
```bash
annactl top [--json] [--limit N]
```

**Flags:**
- `--json` - Output as JSON
- `-l, --limit N` - Number of processes to show (default: 10)

**Examples:**
```bash
# Top 10 processes
annactl top

# Top 5 processes
annactl top --limit 5

# JSON output
annactl top --json
```

**Human Output:**
```
╭─ Top Processes ──────────────────────────────
│
│  By CPU:
│    1.   45.2%  firefox (PID 1234)
│    2.   12.3%  annad (PID 699)
│    3.    8.1%  Xorg (PID 567)
│
│  By Memory:
│    1.  1234.5 MB  firefox (PID 1234)
│    2.   456.7 MB  gnome-shell (PID 890)
│    3.   234.1 MB  code (PID 2345)
│
╰──────────────────────────────────────────────
```

---

### `annactl events`

Show recent system events.

**Syntax:**
```bash
annactl events [--json] [--since DUR] [--limit N]
```

**Flags:**
- `--json` - Output as JSON
- `--since DUR` - Time window: `5m`, `1h`, `1d` (default: all)
- `-l, --limit N` - Number of events (default: 10)

**Examples:**
```bash
# Last 10 events
annactl events

# Events in last hour
annactl events --since 1h

# Last 50 events as JSON
annactl events --limit 50 --json
```

**Human Output:**
```
╭─ System Events ──────────────────────────────
│
│  Showing: 10 events    Pending: 2
│
│  📦 packages     5m ago       pacman -Syu
│     └─ 2 alerts, action: auto_repair (125ms)
│
│  ⚙ config        15m ago      /etc/resolv.conf
│     └─ no alerts, action: alert_only (8ms)
│
╰──────────────────────────────────────────────
```

---

### `annactl export`

Export telemetry data as JSON.

**Syntax:**
```bash
annactl export [--path PATH] [--since DUR] [--json]
```

**Flags:**
- `-p, --path PATH` - Output file (default: stdout)
- `--since DUR` - Time window: `5m`, `1h`, `1d` (default: all)
- `--json` - (Always JSON; included for consistency)

**Examples:**
```bash
# Export to stdout
annactl export

# Export to file
annactl export --path /tmp/telemetry.json

# Export last hour
annactl export --since 1h --path /tmp/hourly.json
```

**Output:**
```
✓ Exported to /tmp/telemetry.json
```

---

### `annactl doctor`

Run system health checks and repairs.

#### `annactl doctor pre`

Run preflight checks before installation.

**Syntax:**
```bash
annactl doctor pre [--json]
```

**Flags:**
- `--json` - Output as JSON

**Checks:**
- OS/architecture (Linux required)
- systemd presence
- Disk space (≥200 MB on `/` and `/var`)
- systemd unit directory writability

**JSON Schema:**
```json
{
  "ok": true,
  "warnings": ["Not running as root"],
  "errors": [],
  "details": {
    "os": "linux",
    "systemd": true,
    "disk_root_mb": 45120,
    "disk_var_mb": 23456
  }
}
```

**Exit Codes:**
- `0` - All checks passed
- `2` - Warnings present
- `4` - Critical failures

#### `annactl doctor post`

Run postflight checks after installation.

**Syntax:**
```bash
annactl doctor post [--json]
```

**Checks:**
- Daemon service active
- Socket exists and has correct permissions
- Database writable
- journalctl shows "RPC socket ready" line

**Example:**
```bash
# Run post-install checks
annactl doctor post

# JSON output
annactl doctor post --json
```

**Human Output:**
```
╭─ Anna Postflight Checks ─────────────────────
│
│  ✓ Service: annad.service active
│  ✓ Socket: /run/anna/annad.sock exists (anna:anna 0770)
│  ✓ Database: /var/lib/anna/telemetry.db writable
│  ✓ Logs: Found "RPC socket ready" in journalctl
│
│  All checks passed!
│
╰──────────────────────────────────────────────
```

#### `annactl doctor repair`

Repair installation issues.

**Syntax:**
```bash
annactl doctor repair [--json] [--yes]
```

**Flags:**
- `--json` - Output as JSON
- `-y, --yes` - Skip confirmation prompt

**Actions:**
- Stop daemon
- Ensure `/run/anna` exists with correct ownership
- Unlink stale socket
- Fix `/var/lib/anna` and `/var/log/anna` permissions
- Restart daemon
- Poll for socket readiness

**Example:**
```bash
# Interactive repair
annactl doctor repair

# Auto-repair
annactl doctor repair --yes
```

**Exit Codes:**
- `0` - No repairs needed
- `3` - Repairs applied successfully
- `4` - Repair failed

---

### `annactl radar show`

Show radar scores for user classification.

**Syntax:**
```bash
annactl radar show [--json] [--user UID|NAME]
```

**Flags:**
- `--json` - Output as JSON
- `-u, --user UID|NAME` - Specify user (default: current user)

**Examples:**
```bash
# Current user's radars
annactl radar show

# Specific user
annactl radar show --user alice

# JSON output
annactl radar show --json
```

**Human Output:**
```
╭─ Persona Radar ──────────────────────────────
│
│  Developer           [▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░]  7.2
│    └─ High interactive time, regular work hours
│
│  Power User          [▓▓▓▓▓▓▓▓▓░░░░░░░░░░░]  4.5
│    └─ Moderate system load
│
╰──────────────────────────────────────────────
```

---

### `annactl classify run`

Run user classification and compute radar scores.

**Syntax:**
```bash
annactl classify run [--json] [--user UID|NAME]
```

**Flags:**
- `--json` - Output as JSON
- `-u, --user UID|NAME` - Specify user (default: current user)

**Examples:**
```bash
# Classify current user
annactl classify run

# Classify specific user
annactl classify run --user 1001

# JSON output
annactl classify run --json
```

**Human Output:**
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
│  Usage Habit Radar:
│    interactive_time     [▓▓▓▓▓▓▓▓▓▓▓▓░░░]  7.5/10
│    cpu_bursty           N/A
│    work_window_regular  N/A
│
│  Network Posture Radar:
│    latency              [▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓] 10.0/10
│    loss                 [▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓] 10.0/10
│    dns_reliability      [▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓] 10.0/10
│
╰──────────────────────────────────────────────
```

See [RADARS.md](RADARS.md) for scoring details.

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Warnings present |
| `3` | Repairs applied |
| `4` | Critical failure |

---

## Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `ANNA_SOCKET` | Override socket path | `/run/anna/annad.sock` |
| `RUST_LOG` | Set log level | `info` |

---

## See Also

- [RADARS.md](RADARS.md) - Radar scoring details
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Common issues
- [README.md](../README.md) - Quick start guide
