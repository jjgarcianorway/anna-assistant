# Anna v7.38.0 "Cache-Only Status & Hardened Daemon"

**System Intelligence Daemon for Linux - NO LLM, NO NATURAL LANGUAGE**

> v7.38.0: Cache-only status (no live probing), hardened daemon startup with crash logging, strict `--version` output for installer parsing.

---

## Public CLI Surface

```bash
# Show help
annactl

# Show version (exactly "vX.Y.Z")
annactl --version

# Anna-only health and status (cache-only, fast)
annactl status

# Software overview
annactl sw

# Software detail (package, command, service, or category)
annactl sw <name-or-category>

# Hardware overview
annactl hw

# Hardware detail (device or category)
annactl hw <name-or-category>
```

**That's it.** Exactly 7 commands. All arguments are case-insensitive.

---

## What Anna Does

Anna is a system intelligence daemon that:

- **Inventories** ALL commands on PATH, ALL packages, ALL systemd services
- **Monitors** process activity (CPU/memory) every 30 seconds
- **Tracks** hardware telemetry (temperature, utilization, I/O)
- **Indexes** errors and warnings from journalctl (per-service only)
- **Records** crash info for debugging without journalctl
- **Writes** status snapshots for fast cache-only status display
- **Checks** for Anna updates (auto-scheduled, configurable)

## What Anna Does NOT Do

- No LLM, no Q&A, no chat
- No arbitrary command execution
- No cloud connectivity
- No data leaves your machine

---

## v7.38.0 Features

### Cache-Only Status

`annactl status` now reads from `status_snapshot.json` only:

```
[VERSION]
  Anna:       v7.38.0

[DAEMON]
  Status:     running
  Uptime:     1d 3h 42m
  PID:        12345
  Snapshot:   32s ago

[HEALTH]
  Overall:    ✓ all systems nominal

[DATA]
  Knowledge:  1234 objects
  Last scan:  2m ago (took 150ms)

[TELEMETRY]
  Samples (24h): 5678

[UPDATES]
  Mode:       auto
  Interval:   10m
  Last check: 2025-12-02 12:34:56
  Result:     up to date
  Next check: in 8m

[ALERTS]
  Critical:   0
  Warnings:   0

[PATHS]
  Config:     /etc/anna/config.toml
  Data:       /var/lib/anna
  Internal:   /var/lib/anna/internal
  Logs:       journalctl -u annad
```

**No live probing** - no `pacman -Q`, `systemctl`, `journalctl`, or filesystem crawling. Status executes in < 10ms.

### Hardened Daemon Startup

- Writes `last_start.json` on every start attempt
- Verifies all directories are writable before starting
- Writes `last_crash.json` on panic/fatal for debugging
- Writes `status_snapshot.json` every 60 seconds

### Crash Logging

When daemon crashes, crash info is written to `/var/lib/anna/internal/last_crash.json`:

```json
{
  "crashed_at": "2025-12-02T12:34:56Z",
  "version": "7.38.0",
  "reason": "panic: index out of bounds at src/foo.rs:123",
  "component": "panic",
  "backtrace": "..."
}
```

`annactl status` shows the last crash when daemon is down - no need to dig through journalctl.

### Strict Version Output

`annactl --version` outputs exactly `vX.Y.Z`:

```bash
$ annactl --version
v7.38.0
```

No banners, no ANSI codes, nothing else. Reliable for installer parsing.

---

## File Paths

| Path | Content |
|------|---------|
| `/etc/anna/config.toml` | Configuration |
| `/var/lib/anna/` | Data directory |
| `/var/lib/anna/knowledge/` | Object inventory (JSON) |
| `/var/lib/anna/telemetry.db` | SQLite telemetry database |
| `/var/lib/anna/internal/` | Internal state |
| `/var/lib/anna/internal/status_snapshot.json` | Daemon status snapshot |
| `/var/lib/anna/internal/last_start.json` | Last start attempt |
| `/var/lib/anna/internal/last_crash.json` | Last crash info |
| `/var/lib/anna/internal/update_state.json` | Update scheduler state |
| `/var/lib/anna/internal/ops.log` | Operations audit trail |

---

## Installation

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

### Manual Install

```bash
# Download binaries
curl -LO https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.38.0/annad-7.38.0-x86_64-unknown-linux-gnu
curl -LO https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.38.0/annactl-7.38.0-x86_64-unknown-linux-gnu

# Install
sudo install -m 755 annad-7.38.0-x86_64-unknown-linux-gnu /usr/local/bin/annad
sudo install -m 755 annactl-7.38.0-x86_64-unknown-linux-gnu /usr/local/bin/annactl
```

### Build from Source

```bash
git clone https://github.com/jjgarcianorway/anna-assistant.git
cd anna-assistant
cargo build --release
sudo install -m 755 target/release/annad /usr/local/bin/annad
sudo install -m 755 target/release/annactl /usr/local/bin/annactl
```

---

## Configuration

Configuration lives at `/etc/anna/config.toml`:

```toml
[core]
mode = "normal"  # normal or dev

[telemetry]
enabled = true
sample_interval_secs = 30
retention_days = 30

[update]
mode = "auto"           # auto or manual
interval_seconds = 600  # 10 minutes
```

---

## Architecture

```
annactl (CLI)
    |
    | reads status_snapshot.json (cache-only)
    |
annad (daemon)
    |
    +-- Process Monitor (30s interval)
    +-- Inventory Scanner (5min interval)
    +-- Log Scanner (60s interval)
    +-- Service Indexer (2min interval)
    +-- Status Snapshot Writer (60s interval)
    +-- Update Scheduler (30s tick, configurable interval)
    |
    v
/var/lib/anna/
    +-- knowledge/               Object inventory
    +-- telemetry.db             SQLite telemetry
    +-- internal/
        +-- status_snapshot.json   Daemon status (for annactl status)
        +-- last_start.json        Last start attempt
        +-- last_crash.json        Last crash info
        +-- update_state.json      Update scheduler state
        +-- ops.log                Operations audit trail
```

---

## Requirements

- **OS**: Linux (x86_64)
- **Rust**: 1.70+ (for building)
- **Systemd**: For daemon management

No Ollama. No LLM. No cloud services.

---

## License

GPL-3.0-or-later

---

## Contributing

Issues and PRs welcome at: https://github.com/jjgarcianorway/anna-assistant

**Design Principles:**

1. Pure observation - no modification
2. Explicit sources - every number traceable to a real command
3. Minimal surface - only essential commands
4. Local only - no cloud, no external calls
5. Clean separation - Anna internals vs host monitoring
6. Honest telemetry - no invented numbers, real data only
7. NO LLM - this is a telemetry daemon, not an AI assistant
8. Cache-only status - no live probing, fast display
