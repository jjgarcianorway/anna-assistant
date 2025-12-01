# Anna v7.16.0 "Log History & Service Lifecycle"

**System Intelligence Daemon for Linux**

> v7.16.0: Multi-window log history (this boot, 24h, 7d) with severity breakdown, service lifecycle tracking (restarts, exit codes, activation failures), enhanced dependency linking.

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

## v7.16.0 Features

### Multi-Window Log History

Log patterns are now tracked across multiple time windows with severity breakdown:

```
[LOGS]

  This boot:
    Errors:   3
    Warnings: 12

  Top patterns:
    1) "connection to %IP% timed out"
       error (boot: 2, 24h: 5, 7d: 23)
    2) "failed to resolve %DOMAIN%"
       error (boot: 1, 7d: 8)
    3) "link state changed to down on %IFACE%"
       warning (boot: 12, 7d: 45)

  Recurring patterns (seen in previous boots):
    - "connection to %IP% timed out" (5 boots, 23 total in 7d)

  Source: journalctl -u service.service -p warning..alert
```

Each pattern shows:
- Count per window (this boot, 24h, 7d)
- Severity level (critical, error, warning)
- Recurrence across boots

### Service Lifecycle Tracking

Software profiles now show systemd unit lifecycle information:

```
[SERVICE LIFECYCLE]
  (source: systemctl show, journalctl)

  State:       active (running)
  Restarts:    0 this boot
  Last exit:   code=0 status=success
  Failures:
    last 24h:  0
    last 7d:   0
```

Hardware profiles show related service lifecycle:

```
[SERVICE LIFECYCLE]
  (source: systemctl show, journalctl)

  NetworkManager.service:
    State:       active (running)
    Restarts:    0 this boot

  wpa_supplicant.service:
    State:       active (running)
    Restarts:    0 this boot
```

### Enhanced Cross Notes

Cross notes now link log history patterns to other observations:

```
Cross notes:
  - Recurring issue seen in 5 boots - may need attention.
  - 2 critical error(s) this boot - requires attention.
```

---

## v7.15.0 Features

### Structured Hardware Overview

`annactl hw` now shows a complete hardware snapshot organized by category:

```
  Anna Hardware Inventory
------------------------------------------------------------

[CPU]
  Model:        Intel(R) Core(TM) i9-14900HX
  Sockets:      1
  Cores:        24 (32 threads)
  Microcode:    genuineintel (version 0x132)

[GPU]
  Discrete:   GeForce RTX 4060 Max-Q / Mobile (driver: nvidia)

[MEMORY]
  Installed:    32 GiB
  Layout:       2 slots (2 used)

[STORAGE]
  Devices:      1 NVMe, 1 SATA
  Root:         nvme0n1p2 (ext4, 512 GiB)

[NETWORK]
  WiFi:         Intel AX201 (driver: iwlwifi, firmware: loaded)
  Ethernet:     Realtek RTL8168 (driver: r8169)
  Bluetooth:    Intel AX201 Bluetooth (driver: btusb)

[AUDIO]
  Controller:   Intel Alder Lake P High Definition Audio
  Drivers:      sof-audio-pci-intel-tgl

[INPUT]
  Keyboard:     AT Translated Set 2 keyboard
  Touchpad:     ELAN1200 Touchpad

[SENSORS]
  Providers:    coretemp, nvme, battery

[POWER]
  Battery:      present (design 80 Wh)
  AC adapter:   connected
```

### Rich CPU Profiles with Firmware

`annactl hw cpu` now includes [FIRMWARE] section with microcode status:

```
[IDENTITY]
  Model:          Intel(R) Core(TM) i9-14900HX
  Sockets:        1
  Cores:          24 (32 threads)
  Architecture:   x86_64
  Flags:          aes, avx, avx2, fma, sse4_2, ...

[FIRMWARE]
  Microcode:      genuineintel (version 0x132)
  Source:         /sys/devices/system/cpu/microcode
  Loaded from:    intel-ucode [installed]
```

### Storage Health with SMART Data

`annactl hw <device>` shows consolidated health information:

```
[HEALTH]
  Overall:     SMART OK
  Temp:        43°C now
  Power on:    1534 hours
  Errors:      0 media errors, 0 reallocated sectors
  Status:      OK
```

### Battery Profile with Capacity and State

`annactl hw battery` shows detailed battery information:

```
[CAPACITY]
  Design:        80 Wh
  Full now:      78 Wh (97% of design)
  Charge now:    72 Wh (92% of full)
  Cycles:        42

[STATE]
  Status:        Discharging
  AC adapter:    not connected

[HEALTH]
  Status:        OK
  Capacity:      97% remaining
```

---

## v7.14.0 Features

### Pattern-Based [LOGS] Section

Log messages are now normalized into patterns for grouping and counting:

```
[LOGS]
  Patterns (this boot):
    Total warnings/errors: 47 (3 patterns)

    Pattern 1: connection to %IP% timed out  (seen 23 times, last at 14:32)
    Pattern 2: failed to resolve %DOMAIN%  (seen 18 times, last at 14:30)
    Pattern 3: link down on interface %IFACE%  (seen 6 times, last at 12:15)
```

Variable parts like IPs, paths, PIDs, interfaces, and domain names are replaced with placeholders (%IP%, %PATH%, %IFACE%, etc.) to group similar messages.

### Config Sanity Checks

The [CONFIG] section now includes sanity notes:

```
[CONFIG]
  Primary:
    ~/.vimrc                                      [present]   (filesystem)
    ~/.vim/                                       [present]   (filesystem)
  Secondary:
    /usr/share/vim                                [present]   (filesystem)
  Sanity notes:
    - ~/.vimrc is a symlink (pointing to dotfiles/vim/vimrc).
    - /etc/vim/vimrc.local exists but is empty (0 bytes).
```

Sanity checks include:
- Empty files (0 bytes)
- Symlinks (with target)
- Readability by current user

### Cross Notes Section

When observations from different sections correlate, a cross notes section appears:

```
Cross notes:
  - Frequent log activity (47 warnings/errors this boot).
```

Cross notes link logs, telemetry, dependencies, and config observations when relevant.

---

## v7.13.0 Features

### [DEPENDENCIES] in Software Profiles

Software profiles now show dependency relationships:

```
[DEPENDENCIES]
  (sources: pacman -Qi, pactree, systemctl show)

  Package deps:
    glibc, openssl, curl, libssh2, gpgme, archlinux-keyring
  Service relations:
    Requires:   dbus.service
    Wants:      network-online.target
    WantedBy:   multi-user.target
```

### [DEPENDENCIES] in Hardware Profiles

Hardware profiles show kernel module dependencies and related services:

```
[DEPENDENCIES]
  (sources: lsmod, modinfo, systemctl)

  Driver module chain:
    iwlwifi  ->  cfg80211
  Used by:
    iwlmvm
  Related services:
    NetworkManager.service [active]
    wpa_supplicant.service [active]
```

### [INTERFACES] in Network Profiles

Network hardware profiles show interface details with traffic:

```
[INTERFACES]
  (sources: /sys/class/net, ip addr)

  wlp0s20f3:
    Type:       wifi
    Driver:     iwlwifi
    MAC:        f8:fe:5e:8d:a4:28
    State:      connected
    IP:         192.168.1.42/24
    Traffic:    RX 1.6 GiB / TX 7.7 GiB (since boot)
```

### Network Summary in Status

Status inventory now includes network interface summary:

```
[INVENTORY]
  Packages:   970  (from pacman -Q)
  Commands:   2654  (from $PATH)
  Services:   260  (from systemctl)
  Network:    1 interfaces (wifi: wlp0s20f3 [up])  (from /sys/class/net)
  Sync:       OK (last full scan 4m ago)
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
  Anna:       v7.16.0

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
  Network:    1 interfaces (wifi: wlp0s20f3 [up])
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
sudo curl -L https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.16.0/annad-7.16.0-x86_64-unknown-linux-gnu -o /usr/local/bin/annad
sudo curl -L https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.16.0/annactl-7.16.0-x86_64-unknown-linux-gnu -o /usr/local/bin/annactl
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
| **v7.16.0** | **Log History & Service Lifecycle** - Multi-window log history, service lifecycle tracking, enhanced cross notes |
| v7.15.0 | Deeper Hardware Insight - Structured hw overview, firmware/microcode, SMART health, battery profiles |
| v7.14.0 | Log Patterns and Config Sanity - Pattern-based log grouping, config sanity checks, cross notes |
| v7.13.0 | Dependency Graph and Network Awareness - deps for packages/services/drivers, network interfaces |
| v7.12.0 | Config Intelligence - Primary/Secondary config, log deduplication, State summaries, ops.log |
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
