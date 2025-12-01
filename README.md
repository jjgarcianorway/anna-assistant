# Anna v7.0.0 "Minimal Surface"

**System Intelligence Daemon for Linux**

> v7.0.0: Only 4 commands. Clean separation of Anna-internal metrics from host monitoring. Every number traceable to real system commands.

---

## What Anna Does

Anna is a system intelligence daemon that:

- **Inventories** ALL commands on PATH, ALL packages, ALL systemd services
- **Monitors** process activity (CPU/memory) every 15 seconds
- **Indexes** errors and warnings from journalctl (per-service only)
- **Tracks** service state changes and package updates

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

# Knowledge database overview
annactl kdb

# Object profile (package, command, or service)
annactl kdb vim
annactl kdb systemd-journald

# Category overview
annactl kdb editors
annactl kdb shells
annactl kdb terminals
annactl kdb browsers
annactl kdb compositors
annactl kdb tools
annactl kdb services
```

That's it. 4 commands. No flags, no complexity.

---

## Example Output

### Help

```
  Anna CLI
------------------------------------------------------------
  annactl           show this help
  annactl status    health and runtime of Anna
  annactl kdb       overview of knowledge database
  annactl kdb NAME  profile for a package, command or category
------------------------------------------------------------
```

### Status

```
  Anna Status
------------------------------------------------------------

[VERSION]
  Anna:       v7.0.0

[DAEMON]
  Status:     running
  Uptime:     3h 12m
  PID:        1234
  Restarts:   0 (24h)

[INVENTORY]
  Packages:   972
  Commands:   2656
  Services:   260
  Sync:       idle (scan 45s ago)

[UPDATES]
  Mode:       auto
  Interval:   10m
  Last check: 5m ago
  Result:     ok
  Next check: 5m

[PATHS]
  Config:     /etc/anna/config.toml
  Data:       /var/lib/anna
  Logs:       journalctl -u annad

[INTERNAL ERRORS]
  Crashes:         0
  Command errors:  0
  Parse errors:    0

------------------------------------------------------------
```

### KDB Overview

```
  Anna Knowledge Database
------------------------------------------------------------

[OVERVIEW]
  Packages known:   972
  Commands known:   2656
  Services known:   260

[CATEGORIES]
  Editors:          vim, nvim, code
  Terminals:        foot, alacritty
  Shells:           bash, zsh, fish
  Compositors:      hyprland, sway
  Browsers:         firefox, chromium
  Tools:            git, curl, wget, grep, awk, sed, ...

[USAGE HIGHLIGHTS]
  Usage telemetry: not collected yet

------------------------------------------------------------
```

### KDB Object (Package)

```
  Anna KDB: vim
------------------------------------------------------------

[IDENTITY]
  Name:        vim
  Description: Vi Improved, a highly configurable...
               (source: pacman -Qi)

[PACKAGE]
  (source: pacman -Qi)
  Version:     9.1.0-1
  Source:      official
  Installed:   explicit
  Size:        5.0 MiB
  Date:        Wed 12 Nov 2025 11:27:12 PM CET

[USAGE]
  (source: Anna telemetry, when available)
  Telemetry not collected yet

[COMMAND]
  (source: which)
  Path:        /usr/bin/vim
  Man:         Vi IMproved, a programmer's text editor

------------------------------------------------------------
```

### KDB Object (Service with Logs)

```
  Anna KDB: systemd-journald
------------------------------------------------------------

[IDENTITY]
  Name:        systemd-journald.service
  Description: Journal Service
               (source: systemctl show)

[SERVICE]
  (source: systemctl)
  Unit:        systemd-journald.service
  State:       running
  Enabled:     static

[LOGS]
  (journalctl -b -u systemd-journald.service -p warning..alert -n 10)

  (no warnings or errors this boot)

------------------------------------------------------------
```

### KDB Category

```
  Anna KDB: Editors
------------------------------------------------------------

  3 editors installed:

  vim         Vi Improved, a highly configurable... (9.1.0-1)
  nvim        Neovim text editor (0.10.1-1)
  code        Visual Studio Code (1.95.0-1)

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
sudo curl -L https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.0.0/annad-7.0.0-x86_64-unknown-linux-gnu -o /usr/local/bin/annad
sudo curl -L https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.0.0/annactl-7.0.0-x86_64-unknown-linux-gnu -o /usr/local/bin/annactl
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
| **v7.0.0** | **Minimal Surface** - Only 4 commands. Clean separation of Anna vs host. Per-service logs only. |
| v6.1.0 | Clean Separation - status/stats/knowledge split |
| v6.0.0 | Grounded System Intelligence - Complete rebuild |
| v5.5.0 | Telemetry Reset - No LLM, pure observation |

---

## License

GPL-3.0-or-later

---

## Contributing

Issues and PRs welcome at: https://github.com/jjgarcianorway/anna-assistant

**Design Principles:**

1. Pure observation - no modification
2. Explicit sources - every number traceable to a real command
3. Minimal surface - only 4 commands
4. ASCII output - no Unicode dependencies
5. Local only - no cloud, no external calls
6. Clean separation - Anna internals vs host monitoring
