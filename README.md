# Anna v7.34.0 "Update Scheduler Fix"

**System Intelligence Daemon for Linux - NO LLM, NO NATURAL LANGUAGE**

> v7.34.0: Fixed update scheduler that actually runs and records checks. Real timestamps in status. Consolidated state management. Ops.log audit trail.

---

## Public CLI Surface

```bash
# Show help
annactl

# Anna-only health and status
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

**That's it.** No other public commands. All arguments are case-insensitive.

---

## What Anna Does

Anna is a system intelligence daemon that:

- **Inventories** ALL commands on PATH, ALL packages, ALL systemd services
- **Monitors** process activity (CPU/memory) every 30 seconds
- **Tracks** hardware telemetry (temperature, utilization, I/O)
- **Indexes** errors and warnings from journalctl (per-service only)
- **Records** Anna's own operations in ops.log
- **Checks** for Anna updates (auto-scheduled, configurable)

## What Anna Does NOT Do

- No LLM, no Q&A, no chat
- No arbitrary command execution
- No cloud connectivity
- No data leaves your machine

---

## v7.34.0 Features

### Working Update Scheduler

The update scheduler now actually runs and records checks:

```
[UPDATES]
  Mode:       auto
  Target:     Anna releases (GitHub)
  Interval:   10m
  Last check: 2025-12-02 12:34:56
  Result:     up to date
  Next check: in 8m
```

- **First check**: Within 60 seconds of daemon start if never checked
- **Scheduled checks**: Every `interval_seconds` (default 600 = 10 minutes)
- **State persistence**: `/var/lib/anna/internal/update_state.json`
- **Audit trail**: All checks logged to `/var/lib/anna/internal/ops.log`

### Status Shows Real Timestamps

- **Last check**: Real timestamp or "never"
- **Next check**: Time until next check, or "n/a" if mode=manual
- **Error**: Shows last error if check failed
- **Available**: Shows version upgrade if update available

### Daemon Status Awareness

If the daemon is not running:
- Status shows "never (daemon not running)" for last check
- Status shows "not running (daemon down)" for next check

---

## File Paths

| Path | Content |
|------|---------|
| `/etc/anna/config.toml` | Configuration |
| `/var/lib/anna/` | Data directory |
| `/var/lib/anna/knowledge/` | Object inventory (JSON) |
| `/var/lib/anna/telemetry.db` | SQLite telemetry database |
| `/var/lib/anna/internal/` | Internal state |
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
curl -LO https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.34.0/annad-7.34.0-x86_64-unknown-linux-gnu
curl -LO https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.34.0/annactl-7.34.0-x86_64-unknown-linux-gnu

# Install
sudo install -m 755 annad-7.34.0-x86_64-unknown-linux-gnu /usr/local/bin/annad
sudo install -m 755 annactl-7.34.0-x86_64-unknown-linux-gnu /usr/local/bin/annactl
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
    | HTTP :7865
    v
annad (daemon)
    |
    +-- Process Monitor (30s interval)
    +-- Inventory Scanner (5min interval)
    +-- Log Scanner (60s interval)
    +-- Service Indexer (2min interval)
    +-- Update Scheduler (30s tick, configurable interval)
    |
    v
/var/lib/anna/
    +-- knowledge/           Object inventory
    +-- telemetry.db         SQLite telemetry
    +-- internal/
        +-- update_state.json  Update scheduler state
        +-- ops.log            Operations audit trail
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
