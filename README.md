# Anna v7.22.0 "Scenario Lenses & Self Toolchain Hygiene"

**System Intelligence Daemon for Linux**

> v7.22.0: Category-aware scenario lenses for hardware (network, storage) and software (network, display, audio, power) with curated views showing [IDENTITY], [TOPOLOGY], [TELEMETRY], [EVENTS], [LOGS]. Self toolchain hygiene tracks Anna's own diagnostic tools with [ANNA TOOLCHAIN] section in status.

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

## v7.22.0 Features

### Scenario Lenses for Hardware Categories

Hardware category commands (`annactl hw network`, `annactl hw storage`) now display curated scenario lenses with structured sections:

```
  Anna HW Lens: network
------------------------------------------------------------

[IDENTITY]
  Name:         wlp0s20f3
  Type:         wifi
  Driver:       iwlwifi
  Firmware:     iwlwifi-ty-a0-gf-a0-89.ucode

[TOPOLOGY]
  MAC:          f8:fe:5e:8d:a4:28
  State:        connected
  IP:           192.168.1.42/24
  Speed:        866 MBit/s

[TELEMETRY]
  RX:           2.4 GiB
  TX:           9.9 GiB
  Signal:       -52 dBm (excellent)

[EVENTS]
  (last 24h from journalctl)
  12:30  wlp0s20f3: connected to MyNetwork
  12:28  wlp0s20f3: link becomes ready

[LOGS]
  (warnings/errors this boot)
  NET001  connection timeout  (count: 2)
  NET002  DNS retry           (count: 5)
```

### Scenario Lenses for Software Categories

Software category commands (`annactl sw network`, `annactl sw display`, `annactl sw audio`, `annactl sw power`) now display curated scenario lenses:

```
  Anna SW Lens: network
------------------------------------------------------------

[SERVICES]
  NetworkManager.service        active (running)
  wpa_supplicant.service        active (running)
  systemd-resolved.service      active (running)

[CONFIG]
  /etc/NetworkManager/NetworkManager.conf   [present]
  /etc/resolv.conf                          [present]
  /etc/nsswitch.conf                        [present]

[LOGS]
  (from journalctl, last 24h)
  NET001  connection established  (count: 3)
  NET002  DHCP lease renewed      (count: 1)
```

### Self Toolchain Hygiene

Anna now tracks her own diagnostic tools with the [ANNA TOOLCHAIN] section in `annactl status`:

```
[ANNA TOOLCHAIN]
  Local wiki:     ready
  Storage tools:  ready
  Network tools:  ready
```

Allowed tools (diagnostic only):
- **Documentation**: arch-wiki-docs
- **Storage**: smartmontools, nvme-cli
- **Network**: ethtool, iw
- **Hardware**: pciutils, usbutils, lm_sensors

Operations are logged to `/var/lib/anna/internal/ops.log`.

---

## v7.21.0 Features

### Config Atlas with Precedence Order

Software profiles now show clean per-component config discovery with explicit precedence:

```
[CONFIG]
  (sources: man vim, Arch Wiki: Vim)
  Active:
    /etc/vimrc                               [present]  (system)
    ~/.vimrc                                 [present]  (user)
    ~/.vim                                   [present]  (user)
  Recommended:
    $XDG_CONFIG_HOME/vim/vimrc               [not present]
  Recently Modified:
    /etc/vimrc                               2w ago

[CONFIG GRAPH]
  Precedence (first match wins):
    1.  ~/.vimrc                             [present]
    2.  /etc/vimrc                           [present]
    3.  /usr/share/vim/vimfiles              [missing]
```

Config discovery is strictly scoped to the component - no cross-contamination between related packages.

### Software Topology and Impact

`annactl sw` now shows [TOPOLOGY] and [IMPACT] sections:

```
[TOPOLOGY]
  (from package descriptions and deps)
  Stacks:
    Display stack libxv, xdg-desktop-portal-hyprland, slurp
    Network stack networkmanager, wpa_supplicant
    Audio stack   portaudio, flac, libwireplumber
  Service Groups:
    Login        systemd-logind.service, getty@tty1.service
    Power        tlp.service
    Network      NetworkManager.service, wpa_supplicant.service

[IMPACT]
  (from telemetry, last 24h)
  CPU:
    1. steam              0.5% avg
    2. claude             19.3% avg
  Memory:
    1. JITWorker          2.3 GiB
    2. HeapHelper         2.3 GiB
```

### Hardware Topology and Impact

`annactl hw` now shows [TOPOLOGY] and [IMPACT] sections:

```
[TOPOLOGY]
  (hardware component summary)
  CPU:          Intel(R) Core(TM) i9-14900HX (24 cores, 32 threads)
  Memory:       31 GiB total
  GPU:          GeForce RTX 4060 (discrete, driver: nvidia)
  Storage:      2 devices [all OK]
  Network:      1 interfaces (wifi)

[IMPACT]
  (from /proc/diskstats, /sys/class/net)
  Disk I/O (since boot):
    nvme0n1 R: 123.0 GiB W: 116.3 GiB
  Network I/O (since boot):
    wlp0s20f3 RX: 2.4 GiB TX: 9.9 GiB
```

### KDB Section in Status

`annactl status` now shows [KDB] section showing knowledge database readiness:

```
[KDB]
  Software:   970 packages, 2654 commands, 260 services
  Hardware:   4 devices (1 GPU, 2 storage, 1 network)
```

---

## v7.20.0 Features

### Deterministic Telemetry Trends

Telemetry sections now show deterministic trend labels comparing 24h vs 7d averages:

```
[TELEMETRY]
  (source: Anna daemon, sampling every 30s)

  State (24h):     mostly active, moderate CPU

  Trend (24h vs 7d):
    CPU:    stable
    Memory: slightly higher
```

Trend labels are mechanically defined:
- **stable**: 24h avg within ±10% of 7d avg
- **slightly higher/lower**: between ±10% and ±30%
- **higher/lower**: between ±30% and ±50%
- **much higher/lower**: more than ±50%

### Log Atlas with Baseline Tags

Log patterns now include baseline tags showing whether patterns are known or new:

```
[LOGS]

  Boot 0 (current):
    Warnings: 3

  New patterns (first seen this boot):
    [S01] "connection timeout to server" [new since baseline]
           error (count: 2)

  Known patterns:
    [S02] "retrying DNS lookup" [known, baseline W01]
           warning (boot: 5, 7d: 23, 3 boots)

  Baseline:
    Boot: -2, 1 known warning patterns
```

Golden baseline selection is deterministic:
- First boot with no error/critical messages
- And no more than 3 warning patterns

### Status Summaries

`annactl status` now shows [TELEMETRY SUMMARY] and [LOG SUMMARY] sections:

```
[TELEMETRY SUMMARY]
  (services with increasing resource usage 24h vs 7d)

  firefox          CPU much higher
  code             memory slightly higher

[LOG SUMMARY]
  (components with new patterns since baseline)

  NetworkManager.service   3 new patterns
  systemd-resolved.service 1 new patterns
```

These appear only when there are notable trends or new patterns to report.

---

## v7.19.0 Features

### Driver Overview and Hot Signals in hw

`annactl hw` now shows loaded kernel modules and signal quality warnings:

```
[DRIVERS]
  (source: lsmod, modinfo)
  GPU:          nvidia + nvidia_modeset, nvidia_drm [loaded]
  WiFi:         iwlwifi [loaded]
  Bluetooth:    btusb [loaded]

[HOT SIGNALS]
  (source: iw, smartctl, nvme)
  🟡 WiFi wlan0 signal weak -76dBm
  🔴 NVMe nvme0n1 health critical
```

### Signal Quality in WiFi and Storage Profiles

`annactl hw wifi` now shows [SIGNAL] section with detailed metrics:

```
[SIGNAL]
  (source: iw, /proc/net/wireless)

  Signal:       -52 dBm ▂▄▆█ (excellent)
  SSID:         MyNetwork
  TX bitrate:   866.7 MBit/s
  RX bitrate:   780.0 MBit/s
  Assessment:   🟢 Good
```

`annactl hw nvme0n1` shows storage signal quality:

```
[SIGNAL]
  (source: nvme smart-log)

  Model:        Samsung SSD 980 PRO 1TB
  Temperature:  35°C
  Wear:         2% used
  Power on:     1847 hours
  Assessment:   🟢 Good
```

### Topology Hints in Status

`annactl status` shows high-impact services and driver stacks:

```
[TOPOLOGY HINTS]
  (source: systemctl, lsmod)

  High-impact services:
    dbus.service (23 wanted by it)
    polkit.service (8 wanted by it)

  Driver stacks:
    GPU [multi-module] (nvidia + nvidia_modeset, nvidia_drm)
```

### Cross-References Between sw and hw

Service profiles now link to related hardware:

```
[DEPENDENCIES]
  Service relations:
    Requires:  dbus.socket
    WantedBy:  multi-user.target

  Related hardware:
    → See: annactl hw wifi
    → See: annactl hw ethernet
```

---

## v7.18.0 Features

### Boot Timeline and Recent Changes in Status

`annactl status` now shows boot health and recent system changes:

```
[LAST BOOT]
  Started:    2025-11-30 20:50
  Kernel:     6.17.9-arch1-1
  Duration:   22s to graphical.target
  Health:     OK (0 failed units, 1 warnings)

[RECENT CHANGES]
  (source: pacman.log)
    2025-12-01 14:23  pkg_upgrade  linux        6.17.8.arch1-1 -> 6.17.9.arch1-1
    2025-12-01 14:23  pkg_upgrade  nvidia       570.133.07-4 -> 570.133.07-5
    2025-11-30 18:04  pkg_upgrade  firefox      132.0.2-1 -> 133.0-1
```

Boot timeline shows:
- Boot timestamp and kernel version
- Duration to reach graphical.target
- Failed units count and warning count
- Overall health status (OK/Warning/Critical)

Recent changes tracks:
- Package installs, upgrades, and removals from pacman.log
- Last 5 events shown in status

### History Section in Profiles

Software and hardware profiles now show [HISTORY] sections:

```
[HISTORY]
  (source: pacman.log, change journal)
  Package:
    2025-03-15 18:59  pkg_install  vim  9.1.1198-1
    2025-03-25 09:51  pkg_upgrade  vim  9.1.1198-1 -> 9.1.1236-1
    2025-04-30 01:01  pkg_upgrade  vim  9.1.1236-1 -> 9.1.1337-1
    ... (7 more events)
```

Hardware profiles (cpu, gpu0) show driver package history:
- CPU shows linux kernel package upgrades
- GPU shows nvidia/mesa driver updates

### Boot-Anchored Log Patterns with Pattern IDs

Service logs now use boot-anchored view with pattern IDs and novelty detection:

```
[LOGS]

  Boot 0 (current):
    Warnings: 1

  New patterns (first seen this boot):
    [c135] "NetworkManager[1163]: <warn>  [1764532230.9456]..."
           error (count: 1)

  Known patterns:
    [a2b7] "connection to %IP% timed out"
           error (boot: 2, 7d: 15, 5 boots)

  Boot -1 (previous):
    3 patterns, 8 total events
```

Each pattern has:
- Stable pattern ID (hash of service+priority+normalized message)
- Novelty indicator (new this boot vs known from previous boots)
- Count per boot and historical context (7d, boots seen)

---

## v7.17.0 Features

### Enhanced Network Topology

`annactl hw` [NETWORK] section now shows complete network topology:

```
[NETWORK]
  Interfaces:
    wlp0s20f3 wifi     up, NetworkManager, 192.168.1.42/24
    enp3s0    ethernet down, NetworkManager
    lo        loopback up
    hci0      bluetooth up (driver: btusb)

  Default route:
    via 192.168.1.1 dev wlp0s20f3

  DNS:
    1.1.1.1, 9.9.9.9 (source: NetworkManager)
```

Each interface shows:
- State (up/down)
- Manager (NetworkManager, systemd-networkd, manual)
- IP address when up

### Storage with Device Health and Filesystems

`annactl hw` [STORAGE] section now shows devices and filesystems:

```
[STORAGE]
  Devices:
    nvme0n1 [OK]  NVMe, Samsung 980 PRO, 1.0 TB
    sda     [?]   SATA, WDC WD10EZEX, 1.0 TB

  Filesystems:
    /            btrfs     5%  nvme0n1p2 [@]
    /home        btrfs    42%  nvme0n1p2 [@home]
    /boot/efi    vfat      1%  nvme0n1p1
```

Each device shows:
- Health status (OK, warning, unknown)
- Type, model, size

Each filesystem shows:
- Mount point
- Type
- Usage percentage (color-coded: green <75%, yellow 75-90%, red >90%)
- Device and subvolume

### Config Graph in Software Profiles

`annactl sw NAME` now shows [CONFIG GRAPH] for services:

```
[CONFIG GRAPH]
  (source: systemctl show, man pages, pacman -Ql)

  Reads:
    /etc/NetworkManager/NetworkManager.conf  [present]  (common path)

  Shared:
    /etc/nsswitch.conf                       [present]  (NSS name resolution)
```

This maps which config files a software component reads, distinguishing:
- Direct configs read by this component
- Shared configs (PAM, NSS) used by multiple components

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
sudo curl -L https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.22.0/annad-7.22.0-x86_64-unknown-linux-gnu -o /usr/local/bin/annad
sudo curl -L https://github.com/jjgarcianorway/anna-assistant/releases/download/v7.22.0/annactl-7.22.0-x86_64-unknown-linux-gnu -o /usr/local/bin/annactl
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
| **v7.22.0** | **Scenario Lenses & Self Toolchain Hygiene** - Category-aware scenario lenses for hw/sw, self toolchain hygiene, [ANNA TOOLCHAIN] section |
| v7.21.0 | Config Atlas, Topology Maps & Impact View - Per-component config discovery, topology maps, resource impact |
| v7.20.0 | Telemetry Trends & Golden Baselines - Deterministic trend labels, log atlas with pattern IDs, golden baselines for pattern comparison |
| v7.19.0 | Topology, Dependencies & Signal Quality - Driver graphs, topology hints, WiFi/storage signal metrics |
| v7.18.0 | Boot Timeline & History - Boot-anchored logs, system change tracking, pattern IDs with novelty |
| v7.17.0 | Network & Storage Topology - Network routes/DNS, storage health, config graphs |
| v7.16.0 | Log History & Service Lifecycle - Multi-window log history, service lifecycle tracking |
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
