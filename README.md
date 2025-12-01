# Anna v5.7.2 "Real-time Package Detection"

**System Intelligence Daemon for Linux**

> v5.7.2: Fixed package install/removal detection. Fixed stats uptime display. Both stats and status commands show proper daemon uptime from API.

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

# List objects by category
annactl knowledge editors
annactl knowledge shells
annactl knowledge terminals
annactl knowledge browsers
annactl knowledge services

# System information (no LLM needed)
annactl cpu
annactl ram
annactl disk

# Clear all data and restart
annactl reset

# Version info
annactl version

# Help
annactl help
```

That's it. Clean commands. No flags, no complexity.

---

## Example Output

### Status

```
  Anna Status
------------------------------------------------------------

[VERSION]
  annactl:  v5.5.0
  annad:    v5.5.0

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

### Knowledge Category

```
  Anna Knowledge: Editors
------------------------------------------------------------

  5 editors installed:

  helix                Helix text editor (0.24.0)
  nano                 GNU Nano text editor (8.0-1)
  nvim                 Neovim text editor (0.10.1-1)
  vim                  Vi Improved text editor (9.1.0-1)
  code                 Visual Studio Code (1.95.0-1)

------------------------------------------------------------

  'annactl knowledge <name>' for full profile.
```

### Knowledge Detail (Package)

```
  Anna Knowledge: nano
------------------------------------------------------------

[IDENTITY]
  Name:        nano
  Description: GNU Nano text editor
  Category:    editor
  Types:       command, package

[INSTALLATION]
  Installed:  yes
  Package:    nano (8.0-1)
  Binary:     /usr/bin/nano

[USAGE]
  (since daemon start)
  Runs:       12 observed
  First seen: 3h ago
  Last seen:  5m ago
  CPU time:   1.2s (total)
  Peak memory: 8.5MB

------------------------------------------------------------
```

### Knowledge Detail (Service)

```
  Anna Knowledge: systemd-journald
------------------------------------------------------------

[IDENTITY]
  Name:        systemd-journald
  Description: System service (systemd service)
  Category:    service
  Types:       service

[SERVICE]
  Unit:       systemd-journald.service
  State:      running
  Enabled:    static

[USAGE]
  (since daemon start)
  Type:       daemon (long-running process)
  First seen: 3h ago
  Last seen:  5s ago
  CPU time:   15.4s (total)
  Peak memory: 42.1MB

------------------------------------------------------------
```

### CPU Info

```
  CPU Information
------------------------------------------------------------

[MODEL]
  AMD Ryzen 9 7950X 16-Core Processor

[CORES]
  32 logical cores

[LOAD AVERAGE]
  1m:  0.52
  5m:  0.61
  15m: 0.55

[TOP PROCESSES BY CPU]
  PID %CPU %MEM COMMAND
  1234 12.5  2.1 firefox
  5678  8.2  1.5 code
  9012  3.1  0.8 hyprland

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
|------|---------
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
| **v5.5.0** | **Telemetry Reset** - No LLM. Fixed service display, category queries, proper reset, cpu/ram/disk commands, error pipeline with top offenders. |
| v5.4.1 | Truthful Telemetry - Real-time package tracking, correct usage counting, expanded categorization. |
| v5.4.0 | Signal Not Noise - Enhanced version command, status with ETA, full descriptions. |
| v5.3.0 | Telemetry Core - Complete reset. No LLM, no Q&A. Pure telemetry daemon. |
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
6. Services are services - not "installed: no"
7. Real telemetry only - no fabricated metrics
