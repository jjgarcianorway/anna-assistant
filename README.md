# Anna v7.12.0 "Config Intelligence and Log Literacy"

**System Intelligence Daemon for Linux**

> v7.12.0: Primary/Secondary config structure, full log deduplication with no truncation, telemetry State summaries, ops.log for internal operations.

---

## What Anna Does

Anna is a system intelligence daemon that:

- **Inventories** ALL commands on PATH, ALL packages, ALL systemd services
- **Monitors** process activity (CPU/memory) every 30 seconds
- **Tracks** hardware telemetry (temperature, utilization, I/O)
- **Indexes** errors and warnings from journalctl (per-service only)
- **Correlates** telemetry with logs for health insights
- **Records** Anna's own operations in ops.log

## What Anna Does NOT Do

- No LLM, no Q&A, no chat
- No arbitrary command execution
- No cloud connectivity
- No data leaves your machine

---

## Commands

```bash
# Show help
annactl

# Anna-only health and status
annactl status

# Software overview
annactl sw

# Object profile (package, command, or service)
annactl sw vim
annactl sw systemd-journald

# Category overview
annactl sw editors
annactl sw shells
annactl sw terminals
annactl sw browsers
annactl sw compositors
annactl sw tools
annactl sw services

# Hardware overview
annactl hw

# Hardware category details
annactl hw cpu
annactl hw gpu
annactl hw storage
annactl hw network
```

---

## v7.12.0 Features

### [CONFIG] Primary/Secondary/Notes Structure

```
[CONFIG]
  Primary:
    ~/.vimrc                                      [present]   (filesystem)
    ~/.vim/                                       [present]   (filesystem)
    ~/.config/vim/vimrc                           [not present] (man vim)
  Secondary:
    /usr/share/vim                                [present]   (filesystem)
  Notes:
    - User config is active.
    - XDG paths take precedence when documented.
    Source: filesystem, pacman -Ql, man pages, local Arch Wiki
```

### [LOGS] Full Deduplication

```
[LOGS]
  (source: journalctl -b -u vim.service -p warning..alert)

  Dec 01 09:15:23  Error loading spellfile: /usr/share/vim/...  (seen 3 times this boot)
  Dec 01 10:30:45  Warning: plugin deprecated
```

### [TELEMETRY] State Summary Line

```
[TELEMETRY]
  (source: Anna daemon, sampling every 30s)

  State (24h):     mostly active, moderate CPU, moderate RAM

  CPU avg (1h):    12.5 %    (max 45.2 %)
  RAM avg (1h):    1.2 GiB  (max 2.4 GiB)
```

### [HW TELEMETRY] State Summary

```
[HW TELEMETRY]
  (source: /sys/class/hwmon, /sys/class/thermal, /proc, sensors)

  State (now):    normal thermals, moderate utilization

  CPU:        52C, 3200 MHz, 12.5% util
  GPU:        45C, 8% util, 512/8192 MiB VRAM
```

### [PATHS] with ops.log

```
[PATHS]
  Config:     /etc/anna/config.toml
  Data:       /var/lib/anna
  Internal:   /var/lib/anna/internal
  Ops log:    /var/lib/anna/internal/ops.log (3 installs, 1 configs)
  Logs:       journalctl -u annad
  Docs:       arch-wiki-lite, man pages, /usr/share/doc
```

---

## Example Output

### Status

```
  Anna Status
------------------------------------------------------------

[VERSION]
  Anna:       v7.12.0

[DAEMON]
  Status:     running
  Uptime:     3h 12m
  PID:        1234
  Restarts:   0 (24h)

[HEALTH]
  Overall:    all systems nominal
  Daemon:     stable
  Telemetry:  collecting
  Sync:       current

[INVENTORY]
  Packages:   972
  Commands:   2656
  Services:   260
  Sync:       OK (last full scan 45s ago)

[TELEMETRY]
  Window: last 24h (sampling every 30s)

  Top CPU identities:
    firefox          avg 5.2 percent, peak 45.2 percent
    code             avg 3.1 percent, peak 28.4 percent

  Top memory identities:
    code             avg 1.2 GiB, peak 2.4 GiB
    firefox          avg 890 MiB, peak 1.8 GiB

[RESOURCE HOTSPOTS]
  (top resource consumers in last 24h)

  CPU:        firefox (45.2% peak)
  RAM:        code (2.4 GiB RSS peak)

[UPDATES]
  Mode:       auto
  Interval:   10m
  Last check: 5m ago
  Result:     ok
  Next check: 5m

[PATHS]
  Config:     /etc/anna/config.toml
  Data:       /var/lib/anna
  Internal:   /var/lib/anna/internal
  Ops log:    /var/lib/anna/internal/ops.log (no operations recorded)
  Logs:       journalctl -u annad
  Docs:       arch-wiki-lite, man pages, /usr/share/doc

[INTERNAL ERRORS]
  Crashes:          0
  Command failures: 0
  Parse errors:     0

[ALERTS]
  Critical:   0
  Warnings:   0

[ANNA NEEDS]
  All tools installed. Anna is fully functional.

------------------------------------------------------------
```

### Software Object Detail

```
  Anna SW: vim
------------------------------------------------------------

[IDENTITY]
  Name:        vim
  Type:        package + command
  Description: Vi Improved, a highly configurable, improved version of the vi text editor
               (source: pacman -Qi)
  Category:    TextEditor
               (source: /usr/share/applications/vim.desktop -> Categories)

[PACKAGE]
  (source: pacman -Qi)
  Version:     9.1.1908-1
  Source:      official
  Installed:   explicit
  Size:        5.0 MiB
  Date:        Wed 12 Nov 2025 11:27:12 PM CET

[CONFIG]
  Primary:
    ~/.vim                                        [present]   (filesystem)
    ~/.vimrc                                      [present]   (filesystem)
    ~/.config/vim/vimrc                           [not present] (man vim)
  Secondary:
    /usr/share/vim                                [present]   (filesystem)
  Notes:
    - User config is active.
    - XDG paths take precedence when documented.
    Source: filesystem, pacman -Ql, man pages, local Arch Wiki

[TELEMETRY]
  (source: Anna daemon, sampling every 30s)

  State (24h):     mostly idle, light CPU, low RAM

  CPU avg (1h):    0.5 %    (max 2.1 %)
  RAM avg (1h):    45 MiB  (max 120 MiB)

  Activity windows:
    Last 1h:   120 samples, avg CPU 0.5%, peak 2.1%, avg RSS 45 MiB, peak 120 MiB
    Last 24h:  2880 samples, avg CPU 0.3%, peak 5.2%, avg RSS 42 MiB, peak 150 MiB

[COMMAND]
  (source: which)
  Path:        /usr/bin/vim
  Man:         Vi IMproved, a programmer's text editor (source: man -f)

------------------------------------------------------------
```

---

## Installation

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

### Manual Install

```bash
sudo curl -L https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.12.0/annad-7.12.0-x86_64-unknown-linux-gnu -o /usr/local/bin/annad
sudo curl -L https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.12.0/annactl-7.12.0-x86_64-unknown-linux-gnu -o /usr/local/bin/annactl
sudo chmod +x /usr/local/bin/annad /usr/local/bin/annactl
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
    +-- Process Monitor (30s interval, sysinfo)
    +-- Inventory Scanner (5min interval)
    +-- Log Scanner (60s interval, journalctl)
    +-- Service Indexer (2min interval, systemctl)
    |
    v
/var/lib/anna/
    +-- knowledge/    Knowledge store
    +-- telemetry/    SQLite telemetry database
    +-- internal/     Anna's internal operations
        +-- ops.log   Operations audit trail
```

### Data Files

| Path | Content |
|------|---------|
| `/var/lib/anna/knowledge/` | Object inventory (JSON) |
| `/var/lib/anna/telemetry.db` | SQLite telemetry database |
| `/var/lib/anna/internal/ops.log` | Anna operations log |
| `/etc/anna/config.toml` | Configuration |

---

## Configuration

Configuration lives at `/etc/anna/config.toml`:

```toml
[core]
mode = "normal"  # normal or dev

[telemetry]
enabled = true
sample_interval_secs = 30      # Process sampling interval
log_scan_interval_secs = 60    # Log scanning interval
retention_days = 30            # Days to keep telemetry

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
| **v7.12.0** | **Config Intelligence** - Primary/Secondary config, log deduplication, State summaries, ops.log |
| v7.11.0 | Honest Telemetry - Real HW telemetry, resource hotspots, health notes |
| v7.10.0 | Arch Wiki configs, hardware drivers and firmware visibility |
| v7.9.0 | Real trends (24h vs 7d), unified telemetry section |
| v7.8.0 | CONFIG hygiene, no ecosystem pollution |
| v7.7.0 | Snow Leopard - Per-window telemetry, auto-install docs |
| v7.6.0 | Telemetry Stability - Configurable retention and max_keys |
| v7.5.0 | Enhanced Telemetry - CPU time tracking, exec counts, hotspots |
| v7.4.0 | Config Discovery - Multi-source config file detection |
| v7.3.0 | Health Signals - Overall health with warnings/criticals |
| v7.2.0 | Telemetry Windows - 1h/24h/7d/30d stats |
| v7.1.0 | SQLite Telemetry - Real process metrics in database |
| v7.0.0 | Minimal Surface - Only 4 commands, clean separation |
| v6.0.0 | Grounded System Intelligence - Complete rebuild |

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
7. Config intelligence - accurate primary/secondary separation
