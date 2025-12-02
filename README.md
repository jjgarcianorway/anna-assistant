# Anna v7.41.0 "Snapshot-Only Display"

**System Intelligence Daemon for Linux - NO LLM, NO NATURAL LANGUAGE**

> v7.41.0: annactl is a pure snapshot reader - all heavyweight scanning is done by the daemon. `annactl sw` now executes in < 1s (warm).

---

## Public CLI Surface

```bash
# Show help
annactl

# Show version (exactly "annactl vX.Y.Z")
annactl --version

# Anna-only health and status (cache-only, fast)
annactl status

# Software overview (compact)
annactl sw

# Software overview (detailed)
annactl sw --full

# Software data (JSON)
annactl sw --json

# Software detail (package, command, service, or category)
annactl sw <name-or-category>

# Hardware overview
annactl hw

# Hardware detail (device or category)
annactl hw <name-or-category>
```

**That's it.** Exactly 10 commands. All arguments are case-insensitive.

---

## What Anna Does

Anna is a system intelligence daemon that:

- **Inventories** ALL commands on PATH, ALL packages, ALL systemd services
- **Monitors** process activity (CPU/memory) every 30 seconds
- **Tracks** hardware telemetry (temperature, utilization, I/O)
- **Indexes** errors and warnings from journalctl (per-service only)
- **Builds** snapshots for fast annactl display
- **Records** crash info for debugging without journalctl
- **Writes** status snapshots for fast cache-only status display
- **Checks** for Anna updates (auto-scheduled, configurable)

## What Anna Does NOT Do

- No LLM, no Q&A, no chat
- No arbitrary command execution
- No cloud connectivity
- No data leaves your machine

---

## v7.41.0 Features

### Snapshot-Only Architecture

`annactl sw` now reads from daemon-generated snapshots:

```
[OVERVIEW]
  Packages:  1234 (126 explicit, 1108 deps, 15 AUR)
  Commands:  2345
  Services:  67 (52 running, 0 failed)

[CATEGORIES]
  (from package descriptions)
  Editors:       vim, neovim, helix
  Terminals:     alacritty, foot
  Browsers:      firefox, chromium
  Development:   git, rust, python
  ...

[PLATFORMS]
  Steam:  42 games (156.3 GiB)
    Elden Ring (45.2 GiB)
    Cyberpunk 2077 (38.7 GiB)
    ...

[TOPOLOGY]
  Compositor:   hyprland, xdg-portal
  Audio:        pipewire, wireplumber
  Network:      networkmanager, iwd
```

**Architecture Rule**: annactl NEVER does heavyweight scanning. All data comes from snapshots written by the daemon.

### Performance

| Command | v7.40.0 (cache) | v7.41.0 (snapshot) |
|---------|-----------------|-------------------|
| `annactl sw` | 6-7s | < 1s |
| `annactl status` | < 350ms | < 350ms |
| `annactl hw` | < 1.2s | < 1.2s |

### Delta Detection

The daemon only rebuilds snapshots when changes are detected:

- **Packages**: pacman.log fingerprint (inode, size, mtime, offset, last line hash)
- **Commands**: PATH directory fingerprints (inode, mtime, file count, names hash)
- **Services**: systemd unit files hash and mtimes

### Snapshot Files

| Path | Content |
|------|---------|
| `/var/lib/anna/internal/snapshots/sw.json` | Software snapshot |
| `/var/lib/anna/internal/snapshots/hw.json` | Hardware snapshot |
| `/var/lib/anna/internal/meta/sw_meta.json` | Delta detection metadata |

---

## File Paths

| Path | Content |
|------|---------|
| `/etc/anna/config.toml` | Configuration |
| `/var/lib/anna/` | Data directory |
| `/var/lib/anna/knowledge/` | Object inventory (JSON) |
| `/var/lib/anna/telemetry.db` | SQLite telemetry database |
| `/var/lib/anna/internal/` | Internal state |
| `/var/lib/anna/internal/snapshots/` | Daemon-written snapshots |
| `/var/lib/anna/internal/meta/` | Delta detection metadata |
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
curl -LO https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.41.0/annad-7.41.0-x86_64-unknown-linux-gnu
curl -LO https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.41.0/annactl-7.41.0-x86_64-unknown-linux-gnu

# Install
sudo install -m 755 annad-7.41.0-x86_64-unknown-linux-gnu /usr/local/bin/annad
sudo install -m 755 annactl-7.41.0-x86_64-unknown-linux-gnu /usr/local/bin/annactl
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
annactl (CLI) - Pure snapshot reader
    |
    | reads snapshots/ (sw.json, hw.json)
    | reads status_snapshot.json
    |
annad (daemon) - Owns all scanning
    |
    +-- Snapshot Builder (60s delta check)
    |     +-- sw.json (software snapshot)
    |     +-- hw.json (hardware snapshot)
    |     +-- sw_meta.json (delta fingerprints)
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
        +-- snapshots/
        |   +-- sw.json          Software snapshot
        |   +-- hw.json          Hardware snapshot
        +-- meta/
        |   +-- sw_meta.json     Delta detection
        +-- status_snapshot.json Daemon status
        +-- last_start.json      Last start attempt
        +-- last_crash.json      Last crash info
        +-- update_state.json    Update scheduler state
        +-- ops.log              Operations audit trail
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
9. Snapshot-only sw - daemon scans, annactl reads
