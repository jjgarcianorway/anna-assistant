# Anna v5.4.1 "Truthful Telemetry"

**System Intelligence Daemon for Linux**

> v5.4.1 critical fixes: Real-time package tracking via pacman.log, correct usage counting (no longer inflated), proper binary categorization, cursor-based log scanning.

---

## What Anna Does

Anna is a system intelligence daemon that:

- **Inventories** ALL commands on PATH, ALL packages, ALL systemd services
- **Monitors** process activity (CPU/memory) every 15 seconds
- **Indexes** errors and warnings from journalctl
- **Detects** intrusion patterns (SSH failures, sudo violations)
- **Tracks** service state changes and package updates

## What Anna Does NOT Do

- No LLM, no Q&A, no chat
- No arbitrary command execution
- No cloud connectivity
- No data leaves your machine

---

## Commands

```bash
# Daemon health and status
annactl status

# Daemon activity statistics
annactl stats

# Knowledge overview by category
annactl knowledge

# Coverage and quality statistics
annactl knowledge stats

# Full profile of a specific object
annactl knowledge <name>

# Clear all data and restart
annactl reset

# Version info
annactl version

# Help
annactl help
```

That's it. Seven commands. No flags, no options, no complexity.

---

## Example Output

### Status

```
  Anna Status
------------------------------------------------------------

[VERSION]
  annactl:  v5.4.0
  annad:    v5.4.0

[DAEMON]
  Status:   running (up 3h)
  Data:     R/W  /var/lib/anna

[INVENTORY]
  Commands:   2656/2685 (99%)
  Packages:   972
  Services:   260/260 (100%)
  Status:     complete

[HEALTH]                          # Only shown if issues exist!
  (last 24 hours)
  Errors:      54
  Warnings:    50

[SCANNER]
  Status:      running (last 6s ago)

------------------------------------------------------------
```

### Version

```
  Anna Version
------------------------------------------------------------

[VERSION]
  annactl:  v5.4.0
  annad:    v5.4.0

[INSTALL PATHS]
  Binaries:
    annactl:  ✓  /usr/local/bin/annactl
    annad:    ✓  /usr/local/bin/annad
  Config:
    ✓  /etc/anna/config.toml
  Data:
    ✓  /var/lib/anna
  Logs:
    ✓  journalctl -u annad

------------------------------------------------------------
```

### Knowledge Detail

```
  Anna Knowledge: bash
------------------------------------------------------------

[IDENTITY]
  Name:        bash
  Description: Bourne Again Shell
  Category:    shell
  Types:       command, package

[INSTALLATION]
  Installed:  yes
  Package:    bash (5.3.3-2)
  Binary:     /usr/bin/bash

[USAGE]
  (since daemon start)
  Runs:       2317 observed
  First seen: 3h ago
  Last seen:  15s ago
  CPU time:   40.9s (total)
  Peak memory: 9.9MB

------------------------------------------------------------
```

---

## Installation

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

### Build from Source

```bash
git clone https://github.com/jjgarcianorway/anna-assistant.git
cd anna-assistant
cargo build --release
sudo ./scripts/install.sh
```

### Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/uninstall.sh | bash
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
    +-- Process Monitor (15s interval, sysinfo)
    +-- Inventory Scanner (5min interval)
    +-- Log Scanner (60s interval, journalctl)
    +-- Service Indexer (2min interval, systemctl)
    |
    v
/var/lib/anna/
    +-- knowledge/    Knowledge store
    +-- telemetry/    Event logs
```

### Data Files

| Path | Content |
|------|---------|
| `/var/lib/anna/knowledge/` | Object inventory (JSON) |
| `/var/lib/anna/telemetry/` | Event logs (pipe-delimited) |
| `/etc/anna/config.toml` | Configuration |

### Event Log Format

```
timestamp|type|key|value1|value2|...
```

Example:
```
1733052000|process|firefox|1234|15.5|536870912
1733052000|command|git|status|0|100
1733052000|service|sshd|active|active
1733052000|package|linux|upgraded|6.6.1|6.6.2
```

---

## Configuration

Configuration lives at `/etc/anna/config.toml`:

```toml
[core]
mode = "normal"  # normal or dev

[telemetry]
sample_interval_secs = 15      # Process sampling interval
log_scan_interval_secs = 60    # Log scanning interval
max_events_per_log = 100000    # Max events per log file
retention_days = 30            # Days to keep event logs

[log]
level = "info"  # trace, debug, info, warn, error
```

---

## Requirements

- **OS**: Linux (x86_64)
- **Rust**: 1.70+ (for building)
- **Systemd**: For daemon management

No Ollama. No LLM. No cloud services.

---

## Version History

| Version | Milestone |
|---------|-----------|
| **v5.4.1** | **Truthful Telemetry** - Critical fixes: real-time package tracking, correct usage counting, expanded categorization, cursor-based log scanning. |
| v5.4.0 | Signal Not Noise - Enhanced version command with install paths, status with ETA, full descriptions, hide empty sections. |
| v5.3.0 | Telemetry Core - Complete reset. No LLM, no Q&A. Pure telemetry daemon. 7 commands only. |
| v5.2.6 | Meaningful Metrics - Every metric has explicit time window/units |
| v5.2.5 | Knowledge is the Machine - Installed-only views, relationships |
| v5.2.0 | Knowledge System - System profiler, full inventory |
| v5.1.0 | Knowledge Foundation - Initial inventory system |

---

## License

GPL-3.0-or-later

---

## Contributing

Issues and PRs welcome at: https://github.com/jjgarcianorway/anna-assistant

**Design Principles:**

1. Pure observation - no modification
2. Explicit time windows - every metric is scoped
3. Relevance only - hide empty sections
4. ASCII output - no Unicode dependencies
5. Local only - no cloud, no external calls
