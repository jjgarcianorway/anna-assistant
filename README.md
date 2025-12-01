# Anna v7.11.0 "Honest Telemetry and Trends"

**System Intelligence Daemon for Linux**

> v7.11.0: Real hardware telemetry (CPU/GPU/Memory/Disk I/O), resource hotspots with health notes, telemetry-to-logs correlation.

---

## What Anna Does

Anna is a system intelligence daemon that:

- **Inventories** ALL commands on PATH, ALL packages, ALL systemd services
- **Monitors** process activity (CPU/memory) every 30 seconds
- **Tracks** hardware telemetry (temperature, utilization, I/O)
- **Indexes** errors and warnings from journalctl (per-service only)
- **Correlates** telemetry with logs for health insights

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

## v7.11.0 Features

### [RESOURCE HOTSPOTS] in Status

```
[RESOURCE HOTSPOTS]
  (top resource consumers in last 24h)

  CPU:        firefox (45.2% peak, web browsing)
              Note: has errors this boot - see `annactl sw firefox`
  RAM:        code (2.4 GiB RSS peak)
```

### [HW TELEMETRY] in Hardware Overview

```
[HW TELEMETRY]
  (source: /sys/class/hwmon, /sys/class/thermal, /proc, sensors)

  CPU:        52°C, 3200 MHz, 12.5% util
  GPU:        45°C, 8% util, 512/8192 MiB VRAM
  Memory:     8.2/32.0 GiB (26% used)
  Disk I/O:   25.3 MB/s read, 12.1 MB/s write
```

### Health Notes in [TELEMETRY]

```
[TELEMETRY]
  Activity windows:
    Last 1h:   120 samples active, avg CPU 12.5%, peak 89.3%
    Last 24h:  2880 samples active, avg CPU 8.2%, peak 95.1%

  Trend:
    CPU:    higher recently (24h vs 7d)
    Memory: stable (24h vs 7d)

  Notes:
    ⚠  High CPU usage detected (peak 95%) - check for runaway processes
    ⚠  3 error(s) in logs this boot - see [LOGS] section above
```

---

## Example Output

### Status

```
  Anna Status
------------------------------------------------------------

[VERSION]
  Anna:       v7.11.0

[DAEMON]
  Status:     running
  Uptime:     3h 12m
  PID:        1234
  Restarts:   0 (24h)

[HEALTH]
  Overall:    ✓ all systems nominal
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
  Logs:       journalctl -u annad

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

### Hardware Overview

```
  Anna Hardware
------------------------------------------------------------

[COMPONENTS]
  (source: lspci -k, lsmod, lscpu, ip link)

  CPU:        AMD Ryzen 9 5900X, driver: amd_pstate, microcode: amd-ucode [present]
  GPU:        NVIDIA GeForce RTX 3080, driver: nvidia, firmware: nvidia-utils [present]
  WiFi:       Intel Wi-Fi 6 AX200, driver: iwlwifi, firmware: linux-firmware [present]
  Audio:      Starship/Matisse HD Audio Controller, driver: snd_hda_intel

[HW TELEMETRY]
  (source: /sys/class/hwmon, /sys/class/thermal, /proc, sensors)

  CPU:        52°C, 3200 MHz, 12.5% util
  GPU:        45°C, 8% util, 512/8192 MiB VRAM
  Memory:     8.2/32.0 GiB (26% used)
  Disk I/O:   25.3 MB/s read, 12.1 MB/s write

[HEALTH HIGHLIGHTS]
  CPU:        normal (52°C)
  GPU:        normal (45°C)
  Disks:      normal (all SMART OK)
  Battery:    not present
  Network:    normal (2 interfaces up)

[CATEGORIES]
  CPU:        cpu0
  Memory:     mem0
  GPU:        gpu0
  Storage:    nvme0n1, sda
  Network:    enp5s0, wlp6s0
  Audio:      audio0
  Power:      (no batteries)

[DEPENDENCIES]
  (hardware monitoring tools)
  Hardware tools:
    smartctl:    installed (smartmontools) - disk SMART
    nvme:        installed (nvme-cli) - NVMe health
    sensors:     installed (lm_sensors) - temperature
    iw:          installed (iw) - wireless info
    ethtool:     installed (ethtool) - ethernet info

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
sudo curl -L https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.11.0/annad-7.11.0-x86_64-unknown-linux-gnu -o /usr/local/bin/annad
sudo curl -L https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.11.0/annactl-7.11.0-x86_64-unknown-linux-gnu -o /usr/local/bin/annactl
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
```

### Data Files

| Path | Content |
|------|---------|
| `/var/lib/anna/knowledge/` | Object inventory (JSON) |
| `/var/lib/anna/telemetry.db` | SQLite telemetry database |
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
| **v7.11.0** | **Honest Telemetry** - Real HW telemetry, resource hotspots, health notes |
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
