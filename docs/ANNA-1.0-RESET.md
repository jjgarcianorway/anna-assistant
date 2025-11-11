# Anna 1.0 Reset - Implementation Plan

**Version:** 1.0.0-rc.13 (Phase 0.5 Complete)
**Branch:** `anna-1.0-reset`
**Status:** Phase 0.5c Complete - Operational Core with Health CLI
**Target:** Production-ready, Wiki-strict, state-aware Arch Linux system administrator

**Latest Updates:**
- ✓ Phase 0.3: State-aware command dispatch with no-ops (commits 4d45d34-e227da3)
- ✓ Phase 0.4: Systemd hardening and security features
- ✓ Phase 0.5a: Health subsystem with YAML probe definitions (commit 4d45d34)
- ✓ Phase 0.5b: RPC and CLI integration for health/doctor/rescue (commits 5410ae2, e227da3)
- ✓ Phase 0.5c: Integration tests, CI workflow, stabilization (commits 58d1be6, 12455af, current)

**Test Coverage:**
- 10 integration tests for health CLI
- CI pipeline with performance benchmarks (<200ms target)
- JSON schema validation for all outputs
- Permissions validation (0600 reports, 0700 directories)
- Unauthorized write detection

**Test Locations:**
- Integration tests: `crates/annad/tests/health_cli_tests.rs`
- Test fixtures: `crates/annad/tests/fixtures/probes/`
- CI workflow: `.github/workflows/health-cli.yml`
- JSON schemas: `tests/schemas/*.schema.json`

---

## Executive Summary

This document defines the complete architectural reset of Anna Assistant for the 1.0 stable release. The redesign focuses on:

- **State-aware operation**: Commands adapt to system state (ISO live, recovery, fresh install, configured, degraded)
- **Wiki-strict correctness**: Only Arch Wiki and man pages as canonical sources
- **Interactive installer**: Full Arch installation with plain English prompts and safe defaults
- **Rescue capabilities**: Recovery tools for broken systems
- **System & desktop administration**: Professional sysadmin for Arch - no desktop environment setup, no aesthetics
- **Comprehensive logging**: Every action cited with Arch Wiki references
- **Atomic rollback**: All operations reversible

**Anna's Mission:** Keep Arch systems secure, updated, auditable, fast, clean, reliable, recoverable, documented, and reproducible.

---

## A) Architecture Cut Map

### Directory Structure (New)

```
anna-assistant/
├── crates/
│   ├── annad/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── state/          # NEW: State detection engine
│   │   │   │   ├── detector.rs
│   │   │   │   ├── types.rs
│   │   │   │   └── capabilities.rs
│   │   │   ├── installer/      # NEW: Interactive installation
│   │   │   │   ├── mod.rs
│   │   │   │   ├── prompts.rs
│   │   │   │   ├── partition.rs
│   │   │   │   ├── filesystem.rs
│   │   │   │   ├── bootstrap.rs
│   │   │   │   └── rollback.rs
│   │   │   ├── rescue/         # NEW: Recovery tools
│   │   │   │   ├── mod.rs
│   │   │   │   ├── detect.rs
│   │   │   │   ├── chroot.rs
│   │   │   │   ├── boot_repair.rs
│   │   │   │   └── fs_check.rs
│   │   │   ├── maintainer/     # REFACTOR: Update/backup/health
│   │   │   │   ├── mod.rs
│   │   │   │   ├── update.rs
│   │   │   │   ├── backup.rs
│   │   │   │   ├── health.rs
│   │   │   │   └── doctor.rs
│   │   │   ├── optimizer/      # REFACTOR: Proposals only
│   │   │   │   ├── mod.rs
│   │   │   │   ├── advise.rs
│   │   │   │   ├── suggest.rs
│   │   │   │   ├── apply.rs
│   │   │   │   └── revert.rs
│   │   │   ├── audit/          # ENHANCED: Wiki citations
│   │   │   │   ├── mod.rs
│   │   │   │   ├── logger.rs
│   │   │   │   ├── citations.rs
│   │   │   │   └── history.rs
│   │   │   ├── telemetry/      # KEEP: Minimal changes
│   │   │   ├── rpc_server/     # ENHANCED: State-aware API
│   │   │   └── whitelist/      # NEW: Command security model
│   │   └── Cargo.toml
│   ├── annactl/
│   │   ├── src/
│   │   │   ├── main.rs         # REFACTOR: State-aware dispatch
│   │   │   ├── commands/
│   │   │   │   ├── install.rs  # NEW: Interactive installer
│   │   │   │   ├── rescue.rs   # NEW: Recovery commands
│   │   │   │   ├── update.rs
│   │   │   │   ├── health.rs
│   │   │   │   ├── advise.rs
│   │   │   │   ├── apply.rs
│   │   │   │   ├── revert.rs
│   │   │   │   ├── status.rs
│   │   │   │   ├── history.rs
│   │   │   │   └── doctor.rs
│   │   │   └── rpc/            # ENHANCED: Capabilities check
│   │   └── Cargo.toml
│   └── anna_common/
│       ├── src/
│       │   ├── state.rs        # NEW: State types
│       │   ├── capabilities.rs # NEW: Capability definitions
│       │   ├── wiki.rs         # NEW: Citation types
│       │   └── rollback.rs     # NEW: Revert script types
│       └── Cargo.toml
├── docs/
│   ├── ANNA-1.0-RESET.md       # THIS FILE
│   ├── STATE-MACHINE.md        # NEW: State documentation
│   ├── INSTALLER-GUIDE.md      # NEW: Installation guide
│   ├── RESCUE-GUIDE.md         # NEW: Recovery guide
│   ├── MIGRATION-1.0.md        # NEW: Breaking changes
│   └── IPC-API.md              # UPDATED: State-aware API
└── tests/
    ├── integration/
    │   ├── state_detection.rs  # NEW
    │   ├── installer.rs        # NEW
    │   ├── rescue.rs           # NEW
    │   └── qemu/               # NEW: End-to-end tests
    └── unit/

```

### Module Responsibilities

| Module | Responsibility | State Access | Wiki Citations |
|--------|----------------|--------------|----------------|
| `state::detector` | Detect system state from environment | Read-only filesystem | [archwiki:installation_guide], [archwiki:system_maintenance] |
| `state::capabilities` | Generate available commands per state | State machine logic | None (internal) |
| `installer::*` | Interactive Arch installation | Destructive, requires iso_live | [archwiki:installation_guide] (all sections) |
| `rescue::*` | Recovery and repair operations | Destructive, requires iso_live or recovery_candidate | [archwiki:system_maintenance], [archwiki:chroot] |
| `maintainer::*` | Routine maintenance on healthy systems | Requires configured state | [archwiki:system_maintenance], [archwiki:pacman] |
| `optimizer::*` | Performance and feature proposals | Requires configured state | All relevant Wiki pages per proposal |
| `audit::logger` | JSONL event logging with citations | All states | Embedded in every log line |
| `whitelist::*` | Command template validation | All states | [archwiki:security] |

### Files to DELETE

```
Removal: TUI, ALL desktop environment code, ALL bundles
├── crates/annad/src/
│   ├── hyprland_config.rs      # DELETE: Desktop env setup out of scope
│   ├── i3_config.rs            # DELETE: Desktop env setup out of scope
│   ├── sway_config.rs          # DELETE: Desktop env setup out of scope
│   ├── gnome_config.rs         # DELETE: Desktop env setup out of scope
│   ├── kde_config.rs           # DELETE: Desktop env setup out of scope
│   ├── xfce_config.rs          # DELETE: Desktop env setup out of scope
│   ├── cinnamon_config.rs      # DELETE: Desktop env setup out of scope
│   ├── mate_config.rs          # DELETE: Desktop env setup out of scope
│   ├── lxqt_config.rs          # DELETE: Desktop env setup out of scope
│   ├── wallpaper_config.rs     # DELETE: Aesthetics out of scope
│   ├── shell_config.rs         # DELETE: User customization out of scope
│   ├── terminal_config.rs      # DELETE: User customization out of scope
│   ├── git_config.rs           # DELETE: User customization out of scope
│   ├── bundles.rs              # DELETE: ALL bundles removed
│   ├── smart_recommender.rs    # DELETE: Non-Wiki heuristics
│   ├── intelligent_recommender.rs  # DELETE: Non-Wiki heuristics
│   ├── resource_classifier.rs  # DELETE: Heuristic logic
│   └── system_detection.rs     # DELETE: WM detection logic
├── crates/annactl/src/
│   └── tui/                    # DELETE: Entire TUI module
│       ├── mod.rs
│       ├── app.rs
│       └── widgets.rs
└── Cargo.toml dependencies:
    ├── ratatui               # DELETE
    ├── crossterm             # DELETE
    └── console               # DELETE (if only used by TUI)
```

---

## B) State Machine Contract

### State Definitions

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SystemState {
    /// Running from Arch ISO - no managed state exists
    IsoLive,

    /// Installed Linux found but unhealthy or unbootable
    RecoveryCandidate,

    /// Fresh Arch with base only - no Anna state
    PostInstallMinimal,

    /// Managed host with /etc/anna/state.json
    Configured,

    /// Managed host failing health checks
    Degraded,

    /// Unable to determine state - discovery mode only
    Unknown,
}
```

### Capabilities API Schema

**RPC Endpoint:** `/capabilities`

**Request:**
```json
{}
```

**Response:**
```json
{
  "state": "iso_live",
  "commands": [
    {
      "name": "install",
      "description": "Interactive Arch Linux installation",
      "flags": ["--dry-run", "--no-confirm"],
      "requires_root": true,
      "wiki_citation": "[archwiki:installation_guide]"
    },
    {
      "name": "rescue-detect",
      "description": "Scan for installed Linux roots",
      "flags": [],
      "requires_root": false,
      "wiki_citation": "[archwiki:chroot]"
    },
    {
      "name": "rescue-chroot",
      "description": "Mount and chroot into installed system",
      "flags": ["--root-device=DEVICE"],
      "requires_root": true,
      "wiki_citation": "[archwiki:chroot]"
    },
    {
      "name": "hardware-report",
      "description": "Show hardware detection results",
      "flags": [],
      "requires_root": false,
      "wiki_citation": "[archwiki:hwinfo]"
    }
  ],
  "blocked_commands": [
    {
      "name": "update",
      "reason": "Not available in iso_live state - requires configured system"
    },
    {
      "name": "advise",
      "reason": "Not available in iso_live state - requires configured system"
    }
  ]
}
```

### State Transition Matrix

| From State | Event | To State | Trigger |
|------------|-------|----------|---------|
| `iso_live` | `install` completes | `post_install_minimal` | `/etc/anna/state.json` created |
| `post_install_minimal` | `converge` completes | `configured` | State adopted, health check passes |
| `configured` | Health check fails | `degraded` | Critical service down or filesystem errors |
| `degraded` | `rollback-last` or `triage` fixes | `configured` | Health check passes |
| `unknown` | `discover` completes | Any valid state | State determined |
| Any | `rescue-detect` finds broken root | `recovery_candidate` | Bootloader missing or kernel panic detected |

### State Detection Logic

```rust
/// State detection algorithm
/// Citations: [archwiki:installation_guide:configure_the_system]
pub fn detect_state() -> Result<SystemState> {
    // Check if running from ISO
    if is_iso_environment() {
        return Ok(SystemState::IsoLive);
    }

    // Check for Anna state file
    let state_file = Path::new("/etc/anna/state.json");
    if state_file.exists() {
        // Managed system - check health
        if health_check_passes() {
            return Ok(SystemState::Configured);
        } else {
            return Ok(SystemState::Degraded);
        }
    }

    // Check if system is bootable minimal Arch
    if is_arch_install() && !has_desktop_environment() {
        return Ok(SystemState::PostInstallMinimal);
    }

    // Check for recovery scenarios
    if is_linux_root_present() && !is_bootable() {
        return Ok(SystemState::RecoveryCandidate);
    }

    Ok(SystemState::Unknown)
}

fn is_iso_environment() -> bool {
    // [archwiki:installation_guide:boot_the_live_environment]
    Path::new("/run/archiso").exists()
}

fn is_arch_install() -> bool {
    // [archwiki:installation_guide:configure_the_system]
    Path::new("/etc/arch-release").exists()
}

fn health_check_passes() -> bool {
    // [archwiki:system_maintenance#check_for_errors]
    check_systemd_failed_units() &&
    check_filesystem_health() &&
    check_critical_journal_errors()
}
```

---

## C) CLI Surface Contract

### Command Matrix by State

| Command | iso_live | recovery_candidate | post_install_minimal | configured | degraded | unknown |
|---------|----------|-------------------|---------------------|------------|----------|---------|
| `install` | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `rescue-detect` | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ |
| `rescue-chroot` | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| `rescue-overview` | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ |
| `boot-repair` | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ |
| `fs-check` | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ |
| `converge` | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ |
| `adopt-state` | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ |
| `update` | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ |
| `backup` | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ |
| `health` | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ |
| `advise` | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ |
| `apply` | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ |
| `suggest` | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ |
| `approve <id>` | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ |
| `revert <id>` | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ |
| `status` | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ |
| `history` | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ |
| `doctor` | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ |
| `triage` | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| `rollback-last` | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| `collect-logs` | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| `discover` | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| `hardware-report` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

### Command Specifications

#### `annactl install`

**State:** `iso_live` only
**Root Required:** Yes
**Wiki Citation:** `[archwiki:installation_guide]`

**Synopsis:**
```bash
annactl install [--dry-run] [--no-confirm] [--config FILE]
```

**Flags:**
- `--dry-run`: Show plan without executing
- `--no-confirm`: Skip all confirmation prompts (uses defaults)
- `--config FILE`: Load answers from JSON file

**Exit Codes:**
- `0`: Installation successful
- `1`: User cancelled
- `2`: Validation failed (not in iso_live state)
- `3`: Partition/format error
- `4`: Package installation error
- `5`: Bootloader installation error

**Output Example:**
```
Anna Installer - Arch Linux Setup

Current state: iso_live
This wizard will install Arch Linux with safe defaults.
Press Ctrl+C at any time to cancel.

[Step 1/8] System Purpose
  Choose installation type:
  1) Minimal base (default)
  2) Desktop workstation (Hyprland)
  3) Server

> 1

[Step 2/8] Disks and Partitions
  Detected disks:
  • /dev/sda (500 GB) - Currently empty
  • /dev/nvme0n1 (1 TB) - Contains Windows 11

  Action:
  1) Install on /dev/sda, keep Windows untouched (default)
  2) Replace Windows on /dev/nvme0n1
  3) Dual-boot with Windows on /dev/nvme0n1
  4) Manual partition layout

> 1

[Step 3/8] Filesystem Layout
  Plan for /dev/sda:
  ┌──────────────┬────────┬─────────┬────────────────┐
  │ Partition    │ Size   │ Type    │ Mount          │
  ├──────────────┼────────┼─────────┼────────────────┤
  │ /dev/sda1    │ 512M   │ FAT32   │ /boot          │
  │ /dev/sda2    │ 499.5G │ ext4    │ /              │
  └──────────────┴────────┴─────────┴────────────────┘

  Enable disk encryption (LUKS)? [y/N]: n

...

[Step 8/8] Review and Confirm
  Installation summary:
  • Target: /dev/sda
  • Filesystem: ext4
  • Encryption: No
  • Bootloader: systemd-boot
  • Hostname: archlinux
  • User: admin (wheel group, sudo enabled)
  • Packages: base, linux, linux-firmware, base-devel, networkmanager

  Commands to execute (40 total):
    1. sgdisk --zap-all /dev/sda
    2. sgdisk --new=1:0:+512M --typecode=1:ef00 /dev/sda
    3. sgdisk --new=2:0:0 --typecode=2:8300 /dev/sda
    ...

  [View all commands: y/N] n

Proceed with installation? [y/N]: y

Installing... (this may take 10-15 minutes)
[✓] Partitions created (3s)
[✓] Filesystems formatted (5s)
[✓] Base system installed (240s)
[✓] Bootloader configured (8s)
[✓] Users created (2s)
[✓] Network configured (1s)

Installation complete!

Generated files:
  /var/log/anna/install-20251111-091234.log
  /var/log/anna/plans/installer.sh
  /var/log/anna/reverts/installer.sh
  /root/postinstall.md

Next steps:
  1. Reboot into your new system
  2. Run: annactl converge
  3. Run: annactl health

Reboot now? [Y/n]:
```

#### `annactl rescue-chroot`

**State:** `iso_live` or `recovery_candidate`
**Root Required:** Yes
**Wiki Citation:** `[archwiki:chroot]`

**Synopsis:**
```bash
annactl rescue-chroot --root-device=DEVICE [--bind-mounts] [--read-only]
```

**Exit Codes:**
- `0`: Chroot session completed successfully
- `1`: User cancelled
- `2`: Validation failed
- `3`: Mount error
- `4`: Chroot failed

**Output Example:**
```
Anna Rescue - Chroot into Installed System

Detected root: /dev/nvme0n1p2 (ext4, 100 GB, Arch Linux)

Mounting...
[✓] /dev/nvme0n1p2 → /mnt
[✓] /dev/nvme0n1p1 → /mnt/boot
[✓] Bind /dev → /mnt/dev
[✓] Bind /proc → /mnt/proc
[✓] Bind /sys → /mnt/sys
[✓] Bind /run → /mnt/run

Entering chroot...

You are now in the installed system.
Common rescue commands:
  • mkinitcpio -P              (rebuild initramfs)
  • bootctl install            (reinstall systemd-boot)
  • pacman -Syy archlinux-keyring && pacman -Su  (update keyring)
  • systemctl list-units --failed  (check failed services)

Type 'exit' to return to ISO.

[root@archlinux /]#
```

#### `annactl update`

**State:** `configured` only
**Root Required:** Yes (via daemon)
**Wiki Citation:** `[archwiki:system_maintenance#upgrading_the_system]`

**Synopsis:**
```bash
annactl update [--check-news] [--skip-backup] [--no-confirm]
```

**Exit Codes:**
- `0`: Update successful
- `1`: Update available but not applied
- `2`: Validation failed
- `3`: Pacman error
- `4`: News check failed (blocking update)

**Output Example:**
```
Anna Update - System Maintenance

[1/5] Checking Arch News...
  Latest news (since 2025-11-08):
  • 2025-11-10: Important changes to systemd-boot
    Action required: Run 'bootctl update' after upgrade

Continue with update? [Y/n]: y

[2/5] Creating snapshot...
  [✓] Snapshot: /var/backups/anna/pre-update-20251111

[3/5] Refreshing package databases...
  [✓] Keyring updated
  [✓] Mirrors ranked (reflector)
  [✓] Database synchronized

[4/5] Calculating updates...
  Packages to upgrade (42):
    linux 6.11.1 → 6.11.2
    systemd 256.1 → 256.2
    firefox 131.0 → 132.0
    ... (39 more)

  Download size: 450 MB
  Install size: 1.2 GB

Proceed? [Y/n]: y

[5/5] Applying updates...
  Downloading... (450 MB) [████████████████████] 100%
  Installing... [✓]

Update complete!

Post-update actions:
  [!] Run: bootctl update (systemd-boot news item)
  [!] Restart required (kernel updated)

Rollback script: /var/log/anna/reverts/update-20251111-093045.sh

Would you like to reboot now? [y/N]: n
```

#### `annactl health`

**State:** `post_install_minimal`, `configured`, `degraded`
**Root Required:** No (read-only operations)
**Wiki Citation:** `[archwiki:system_maintenance#check_for_errors]`

**Synopsis:**
```bash
annactl health [--verbose] [--json]
```

**Exit Codes:**
- `0`: All checks passed
- `1`: Warnings found
- `2`: Critical issues found

**Output Example:**
```
Anna Health Check

Disk Health [✓]
  • SMART: All disks healthy
  • Filesystem: ext4 on / - last check 5 days ago (within 30 day interval)
  • Free space: 450 GB / 500 GB (90% free)

Services [!]
  • 0 failed units
  • 1 warning: sshd.service is masked

Journal [✓]
  • 0 critical errors in last 7 days
  • 3 warnings (non-blocking)

Security [✓]
  • Firewall: active (ufw)
  • SSH: configured securely (key-only auth)
  • Sudo: no NOPASSWD entries

Packages [✓]
  • 847 packages installed
  • 12 unmanaged (AUR or manual): yay, hyprland-git, ...
  • Last update: 2 days ago

Overall: HEALTHY (1 warning)

View warnings in detail? [y/N]:
```

#### `annactl advise`

**State:** `configured` only
**Root Required:** No
**Wiki Citation:** Varies per proposal

**Synopsis:**
```bash
annactl advise [--category CATEGORY] [--preview <id>]
```

**Exit Codes:**
- `0`: Proposals generated
- `1`: No proposals available

**Output Example:**
```
Anna Optimizer - System & Desktop Administration Proposals

Found 6 optimization opportunities:

[1] Install microcode updates (Intel)
    Category: Security
    Impact: High
    Citation: [archwiki:microcode]
    Status: Missing package

[2] Enable fstrim.timer for SSD maintenance
    Category: Maintenance
    Impact: Medium
    Citation: [archwiki:solid_state_drive#periodic_trim]
    Status: Not enabled

[3] Configure zram swap
    Category: Performance
    Impact: Low
    Citation: [archwiki:zram]
    Status: Not configured

[4] Enable bluetooth service
    Category: Desktop Hardware
    Impact: Low
    Citation: [archwiki:bluetooth]
    Status: Package installed, service disabled

[5] Configure printing (CUPS)
    Category: Desktop Hardware
    Impact: Low
    Citation: [archwiki:cups]
    Status: Not installed

[6] Harden SSH configuration
    Category: Security
    Impact: High
    Citation: [archwiki:openssh#security]
    Status: Weak settings detected

View proposal details: annactl advise --preview <id>
Apply proposal: annactl apply <id>
```
**Note:** Anna generates proposals for system administration (updates, security, maintenance) and desktop administration (hardware services, user management) - NOT desktop environment setup or aesthetics.

---

## D) Installer Specification

### Installation Flow

**Step Sequence** (mirrors Arch Wiki installation guide exactly):

1. **System Purpose** `[archwiki:installation_guide:pre-installation]`
2. **Disk Selection and Windows Detection** `[archwiki:installation_guide:partition_the_disks]`
3. **Partition Plan** `[archwiki:installation_guide:partition_the_disks]`
4. **Filesystem and Encryption** `[archwiki:installation_guide:format_the_partitions]`
5. **Network and Locale** `[archwiki:installation_guide:configure_the_system]`
6. **Users and Sudo** `[archwiki:installation_guide:create_a_user]`
7. **Packages and Desktop** `[archwiki:installation_guide:install_essential_packages]`
8. **Review and Execute** (confirmation + progress)

### Prompts and Defaults

#### Step 1: System Purpose
```
[Step 1/8] System Purpose

Choose installation type:
  1) Minimal base (default)
     - base, linux, linux-firmware, base-devel
     - NetworkManager, sudo
     - ~500 MB download, ~2 GB installed
     - Install desktop environment manually later via pacman

  2) Server
     - Minimal base + openssh, ufw
     - No desktop environment
     - ~600 MB download, ~2.5 GB installed

>
```
**Default:** `1` (Minimal base)
**Validation:** Must be 1 or 2
**Citation:** `[archwiki:installation_guide:pre-installation]`
**Note:** Anna does not install or configure desktop environments. Users install their preferred DE/WM via pacman after base installation completes.

#### Step 2: Disks and Windows Detection
```
[Step 2/8] Disks and Partitions

Detected disks:
  • /dev/sda (500 GB, Samsung SSD 870)
    Status: Empty

  • /dev/nvme0n1 (1 TB, WD Black SN850)
    Status: Contains Windows 11
    Partitions:
      - /dev/nvme0n1p1 (100 MB) - EFI System
      - /dev/nvme0n1p2 (16 MB) - Microsoft Reserved
      - /dev/nvme0n1p3 (950 GB) - Windows (NTFS)
      - /dev/nvme0n1p4 (50 GB) - Recovery (NTFS)

Action:
  1) Install on /dev/sda, keep Windows untouched (default)
  2) Replace Windows on /dev/nvme0n1 (DESTRUCTIVE - all data lost)
  3) Dual-boot with Windows on /dev/nvme0n1 (share EFI partition)
  4) Manual partition layout (advanced)

>
```
**Default:** `1` (Install on empty disk, preserve Windows)
**Validation:** Confirm destructive actions twice
**Citation:** `[archwiki:installation_guide:partition_the_disks]`, `[archwiki:dual_boot_with_windows]`

#### Step 3: Partition Plan
```
[Step 3/8] Filesystem Layout

Plan for /dev/sda:
┌──────────────┬────────┬─────────┬────────────────┐
│ Partition    │ Size   │ Type    │ Mount          │
├──────────────┼────────┼─────────┼────────────────┤
│ /dev/sda1    │ 512M   │ FAT32   │ /boot          │
│ /dev/sda2    │ 499.5G │ ext4    │ /              │
└──────────────┴────────┴─────────┴────────────────┘

Options:
  1) Use this layout (default)
  2) Change filesystem to btrfs with subvolumes (@, @home, @snapshots)
  3) Add separate /home partition
  4) Manual layout

>
```
**Default:** `1` (Simple ext4 on root)
**Validation:** Minimum 20 GB for root, 512 MB for EFI
**Citation:** `[archwiki:installation_guide:partition_the_disks]`

#### Step 4: Encryption
```
[Step 4/8] Encryption

Enable full-disk encryption (LUKS)?
  • Encrypts root partition only (not /boot)
  • Requires passphrase on every boot
  • Performance impact: ~5% on modern CPUs

Enable LUKS encryption? [y/N]:
```
**Default:** `N` (No encryption)
**Validation:** Passphrase minimum 20 characters if enabled
**Citation:** `[archwiki:dm-crypt/encrypting_an_entire_system]`

#### Step 5: Network and Locale
```
[Step 5/8] System Configuration

Hostname (default: archlinux): _
Timezone (default: UTC): _
Locale (default: en_US.UTF-8): _

Network configuration:
  1) NetworkManager (desktop default)
  2) systemd-networkd (server/minimal)

>
```
**Defaults:** `archlinux`, `UTC`, `en_US.UTF-8`, NetworkManager
**Validation:** Hostname RFC compliance, timezone from /usr/share/zoneinfo, locale in /etc/locale.gen
**Citation:** `[archwiki:installation_guide:configure_the_system]`, `[archwiki:network_configuration]`

#### Step 6: Users
```
[Step 6/8] User Accounts

Create administrative user:
  Username (default: admin): _
  Add to wheel group and enable sudo? [Y/n]:
  Set password now? [Y/n]:

Root account:
  Lock root password (recommended)? [Y/n]:
```
**Defaults:** `admin`, yes to wheel/sudo, yes to password, yes to lock root
**Validation:** Username POSIX compliance, password strength check
**Citation:** `[archwiki:users_and_groups]`, `[archwiki:sudo]`

#### Step 7: Packages
```
[Step 7/8] Package Selection

Base packages (required): base, linux, linux-firmware, base-devel
Essential tools (recommended): networkmanager, sudo, vim, git

Additional packages:
  [Selected based on Step 1 choice]
  Server: openssh, ufw

Desktop environments can be installed manually after reboot via pacman.

Review package list? [y/N]:
```
**Defaults:** Minimal set from Step 1 choice (base or server)
**Validation:** Verify package names exist in repos
**Citation:** `[archwiki:installation_guide:install_essential_packages]`
**Note:** Anna installs only base system. Desktop environments are outside 1.0 scope.

#### Step 8: Confirmation
```
[Step 8/8] Review and Confirm

Installation Summary
════════════════════════════════════════════════════════

Target:          /dev/sda (500 GB)
Partition Table: GPT
Filesystems:     FAT32 (/boot), ext4 (/)
Encryption:      No
Bootloader:      systemd-boot
Hostname:        archlinux
Users:           admin (wheel, sudo)
Packages:        45 packages (~2.1 GB installed)
Network:         NetworkManager

This will DESTROY all data on /dev/sda.

Commands to execute (48 total):
  Partition:  sgdisk --zap-all /dev/sda
              sgdisk --new=1:0:+512M --typecode=1:ef00 /dev/sda
              ...
  Format:     mkfs.fat -F32 /dev/sda1
              mkfs.ext4 /dev/sda2
  Mount:      mount /dev/sda2 /mnt
              mount --mkdir /dev/sda1 /mnt/boot
  Install:    pacstrap -K /mnt base linux linux-firmware ...
  Configure:  genfstab -U /mnt >> /mnt/etc/fstab
              arch-chroot /mnt ln -sf /usr/share/zoneinfo/UTC /etc/localtime
              ...
  Bootloader: arch-chroot /mnt bootctl install
              ...

[View full command list: y/N] n

Proceed with installation? Type 'YES' to confirm: _
```
**Validation:** Must type exact string "YES" to proceed
**Dry-run available:** `annactl install --dry-run` generates scripts without executing
**Citation:** All previous step citations combined

### Generated Artifacts

#### 1. State File: `/etc/anna/state.json`
```json
{
  "version": "1.0.0",
  "state": "configured",
  "installed_at": "2025-11-11T09:12:34Z",
  "installed_by": "annactl install",
  "hardware": {
    "cpu": "Intel Core i7-12700K",
    "ram_gb": 32,
    "disks": ["/dev/sda"],
    "gpu": ["Intel UHD Graphics 770"]
  },
  "configuration": {
    "filesystem": "ext4",
    "bootloader": "systemd-boot",
    "init": "systemd",
    "network": "NetworkManager",
    "desktop": null
  },
  "health": {
    "last_check": "2025-11-11T09:15:00Z",
    "status": "healthy",
    "warnings": []
  }
}
```

#### 2. Installation Plan: `/var/log/anna/plans/installer-20251111-091234.sh`
```bash
#!/bin/bash
# Anna Installer - Execution Plan
# Generated: 2025-11-11T09:12:34Z
# State: iso_live → post_install_minimal
# Citation: [archwiki:installation_guide]

set -euo pipefail

# Step 1: Partition disk [archwiki:installation_guide:partition_the_disks]
sgdisk --zap-all /dev/sda
sgdisk --new=1:0:+512M --typecode=1:ef00 --change-name=1:'EFI' /dev/sda
sgdisk --new=2:0:0 --typecode=2:8300 --change-name=2:'Linux' /dev/sda

# Step 2: Format filesystems [archwiki:installation_guide:format_the_partitions]
mkfs.fat -F32 -n EFI /dev/sda1
mkfs.ext4 -L ROOT /dev/sda2

# Step 3: Mount [archwiki:installation_guide:mount_the_file_systems]
mount /dev/sda2 /mnt
mount --mkdir /dev/sda1 /mnt/boot

# Step 4: Install base system [archwiki:installation_guide:install_essential_packages]
pacstrap -K /mnt base linux linux-firmware base-devel networkmanager sudo vim git

# Step 5: Generate fstab [archwiki:installation_guide:fstab]
genfstab -U /mnt >> /mnt/etc/fstab

# Step 6: Configure system [archwiki:installation_guide:configure_the_system]
arch-chroot /mnt ln -sf /usr/share/zoneinfo/UTC /etc/localtime
arch-chroot /mnt hwclock --systohc
arch-chroot /mnt locale-gen
echo "LANG=en_US.UTF-8" > /mnt/etc/locale.conf
echo "archlinux" > /mnt/etc/hostname

# Step 7: Initramfs [archwiki:installation_guide:initramfs]
arch-chroot /mnt mkinitcpio -P

# Step 8: Bootloader [archwiki:installation_guide:boot_loader]
arch-chroot /mnt bootctl install
cat > /mnt/boot/loader/loader.conf <<EOF
default arch.conf
timeout 3
console-mode max
editor no
EOF
cat > /mnt/boot/loader/entries/arch.conf <<EOF
title   Arch Linux
linux   /vmlinuz-linux
initrd  /initramfs-linux.img
options root=PARTUUID=$(blkid -s PARTUUID -o value /dev/sda2) rw
EOF

# Step 9: Users [archwiki:users_and_groups]
arch-chroot /mnt useradd -m -G wheel -s /bin/bash admin
arch-chroot /mnt passwd admin
arch-chroot /mnt passwd -l root
sed -i 's/^# %wheel ALL=(ALL:ALL) ALL/%wheel ALL=(ALL:ALL) ALL/' /mnt/etc/sudoers

# Step 10: Enable services
arch-chroot /mnt systemctl enable NetworkManager

# Step 11: Create Anna state
mkdir -p /mnt/etc/anna
cat > /mnt/etc/anna/state.json <<EOF
{
  "version": "1.0.0",
  "state": "post_install_minimal",
  "installed_at": "$(date -Iseconds)",
  "configuration": {
    "filesystem": "ext4",
    "bootloader": "systemd-boot"
  }
}
EOF

echo "Installation complete. Reboot to start your new system."
```

#### 3. Rollback Script: `/var/log/anna/reverts/installer-20251111-091234.sh`
```bash
#!/bin/bash
# Anna Installer - Rollback Script
# DESTRUCTIVE: This script will undo the installation
# Generated: 2025-11-11T09:12:34Z

set -euo pipefail

echo "WARNING: This will destroy the Arch installation on /dev/sda"
read -p "Type 'DESTROY' to confirm: " confirm
[[ "$confirm" != "DESTROY" ]] && exit 1

# Unmount filesystems
umount -R /mnt 2>/dev/null || true

# Wipe partition table
sgdisk --zap-all /dev/sda

# Wipe filesystem signatures
wipefs -a /dev/sda1 2>/dev/null || true
wipefs -a /dev/sda2 2>/dev/null || true

echo "Rollback complete. /dev/sda is now empty."
```

#### 4. Post-Install Guide: `/root/postinstall.md`
```markdown
# Post-Installation Steps

Welcome to your new Arch Linux system!

## First Boot

Your system is now installed. After rebooting:

1. Remove the installation media
2. Boot into Arch Linux
3. Log in as: `admin` (password set during install)

## Next Steps

### 1. Adopt Anna state (required)
```bash
sudo annactl converge
```

This will:
- Initialize Anna's monitoring
- Run initial health check
- Detect hardware and generate optimization proposals

### 2. Check system health
```bash
annactl health
```

### 3. Review optimization proposals
```bash
annactl advise
```

### 4. Install desktop environment (manual)
If you chose "minimal base" and want a desktop:
```bash
# Install your preferred desktop environment via pacman
# Examples:
sudo pacman -S plasma-meta        # KDE Plasma
sudo pacman -S gnome gnome-extra  # GNOME
sudo pacman -S xfce4 xfce4-goodies # XFCE
sudo pacman -S hyprland           # Hyprland (Wayland compositor)

# Anna does not configure desktop environments
# Refer to Arch Wiki for desktop-specific setup
```

## Documentation

- Arch Wiki: https://wiki.archlinux.org
- Anna Documentation: https://github.com/jjgarcianorway/anna-assistant

## Citations

All installation steps followed:
- [archwiki:installation_guide]
- [archwiki:general_recommendations]

Installation log: `/var/log/anna/install-20251111-091234.log`
```

### Rollback Points

| Phase | Reversible? | Rollback Method | Citation |
|-------|-------------|-----------------|----------|
| Partition creation | Yes | Wipe partition table with `sgdisk --zap-all` | [archwiki:gdisk] |
| Filesystem format | Yes | Wipe signatures with `wipefs` | [archwiki:file_systems] |
| Package installation | Yes | Unmount and wipe | N/A |
| Bootloader install | Partial | Remove EFI entries | [archwiki:systemd-boot] |
| User creation | N/A | Not applicable (in chroot) | N/A |

**Note:** Once installation completes and system reboots, rollback script is only useful for reclaiming the disk for a fresh install.

---

## E) Rescue Specification

### Detection Matrix

**Scenario 1: Broken Bootloader**
```
Detection:
  - EFI partition exists (/dev/sdX1, FAT32)
  - Root partition exists (/dev/sdX2, ext4/btrfs)
  - Root contains /etc/arch-release
  - Bootloader entries missing or invalid

Diagnosis:
  • Check /boot/loader/entries/ for .conf files
  • Run `bootctl status` to verify installation

Repair:
  1. Mount root and EFI
  2. Bind /dev, /proc, /sys, /run
  3. Chroot
  4. `bootctl install` [archwiki:systemd-boot]
  5. Regenerate entries
  6. Exit and unmount

Citation: [archwiki:systemd-boot], [archwiki:chroot]
```

**Scenario 2: Broken Initramfs**
```
Detection:
  - System boots to emergency shell
  - Error: "Failed to mount /dev/sdXY"
  - /boot/initramfs-linux.img exists but kernel panics

Diagnosis:
  • Missing kernel modules in initramfs
  • Incorrect HOOKS in /etc/mkinitcpio.conf

Repair:
  1. Chroot into system
  2. Review /etc/mkinitcpio.conf HOOKS
  3. `mkinitcpio -P` [archwiki:mkinitcpio]
  4. Exit, unmount, reboot

Citation: [archwiki:mkinitcpio], [archwiki:chroot]
```

**Scenario 3: Filesystem Corruption**
```
Detection:
  - Boot fails with filesystem errors
  - "Superblock invalid" or similar
  - System drops to emergency mode

Diagnosis:
  • Run `blkid` to identify filesystem type
  • Check dmesg for I/O errors

Repair:
  1. Do NOT mount filesystem
  2. Run fsck from live environment
     - ext4: `e2fsck -f /dev/sdXY` [archwiki:fsck]
     - btrfs: `btrfs check --repair /dev/sdXY` [archwiki:btrfs#scrub]
  3. If repair succeeds, mount and verify
  4. If repair fails, data recovery needed

Citation: [archwiki:fsck], [archwiki:btrfs#check]
```

**Scenario 4: Broken Pacman Database**
```
Detection:
  - System boots but pacman fails
  - "database is locked" or corrupted

Diagnosis:
  • Check /var/lib/pacman/db.lck
  • Verify /var/lib/pacman/local integrity

Repair:
  1. Chroot into system
  2. Remove lock file: `rm /var/lib/pacman/db.lck`
  3. Refresh databases: `pacman -Syy`
  4. Update keyring: `pacman -S archlinux-keyring` [archwiki:pacman/package_signing]
  5. Test: `pacman -Q`

Citation: [archwiki:pacman#database_corruption]
```

**Scenario 5: Network Configuration Broken**
```
Detection:
  - System boots but no network
  - NetworkManager inactive or missing

Diagnosis:
  • Check `systemctl status NetworkManager`
  • Verify network interfaces with `ip link`

Repair:
  1. Chroot into system
  2. `systemctl enable NetworkManager` [archwiki:networkmanager]
  3. If package missing: `pacman -S networkmanager`
  4. Exit, unmount, reboot

Citation: [archwiki:networkmanager], [archwiki:network_configuration]
```

### Rescue Commands

#### `annactl rescue-detect`

**Output:**
```
Anna Rescue - System Detection

Scanning for installed systems...

Found: /dev/nvme0n1p2 (ext4)
  • OS: Arch Linux
  • Kernel: linux 6.11.1-arch1-1
  • Bootloader: systemd-boot
  • Status: UNBOOTABLE
  • Issues:
    [!] Missing bootloader entries in EFI partition
    [✓] Root filesystem healthy
    [✓] Initramfs present

Recommended action:
  annactl rescue-chroot --root-device=/dev/nvme0n1p2
  # Then inside chroot:
  bootctl install
  bootctl list  # verify entries

Citation: [archwiki:chroot], [archwiki:systemd-boot]
```

#### `annactl rescue-overview`

**State:** `recovery_candidate`
**Output:**
```
Anna Rescue - Recovery Overview

Target: /dev/nvme0n1p2 (Arch Linux)

Health Check Results:
┌─────────────────────┬────────┬─────────────────────────────┐
│ Component           │ Status │ Details                     │
├─────────────────────┼────────┼─────────────────────────────┤
│ Filesystem          │ ✓      │ ext4, no errors             │
│ Bootloader          │ ✗      │ Missing EFI entries         │
│ Initramfs           │ ✓      │ Present, up to date         │
│ Kernel              │ ✓      │ 6.11.1-arch1-1              │
│ Pacman database     │ ✓      │ Valid, 847 packages         │
│ Network config      │ ✓      │ NetworkManager enabled      │
│ Critical services   │ ?      │ Cannot verify (not booted)  │
└─────────────────────┴────────┴─────────────────────────────┘

Repair Playbooks Available:
  1. boot-repair        Fix bootloader issues
  2. fs-check           Check and repair filesystem
  3. pacman-rebuild     Rebuild pacman database
  4. keyring-refresh    Update archlinux-keyring

Select playbook [1-4] or 'chroot' for manual repair:
```

#### `annactl boot-repair`

**Automated Bootloader Repair:**
```bash
Anna Rescue - Boot Repair Playbook
Citation: [archwiki:systemd-boot]

[1/6] Mounting filesystems...
  [✓] /dev/nvme0n1p2 → /mnt
  [✓] /dev/nvme0n1p1 → /mnt/boot
  [✓] Bind mounts (/dev, /proc, /sys, /run)

[2/6] Verifying bootloader installation...
  [✗] systemd-boot not installed in EFI

[3/6] Installing systemd-boot...
  # bootctl --esp-path=/boot install
  Created "/boot/EFI/systemd/systemd-bootx64.efi".
  Created "/boot/loader/entries" directory.

[4/6] Generating boot entries...
  # cat > /boot/loader/entries/arch.conf
  [✓] Entry: arch.conf

[5/6] Setting default entry...
  [✓] Default: arch.conf

[6/6] Verifying configuration...
  # bootctl list
  [✓] Found 1 entry: Arch Linux

Boot repair complete. Unmounting and ready to reboot.

Reboot now? [Y/n]:
```

### Guardrails

1. **Destructive Actions:** Always confirm twice
   ```
   This action will modify the bootloader.
   Type 'CONFIRM' to proceed:
   ```

2. **Filesystem Checks:** Never run fsck on mounted filesystems
   ```
   ERROR: /dev/nvme0n1p2 is mounted at /mnt
   Unmount before running filesystem check.
   Citation: [archwiki:fsck#usage]
   ```

3. **Chroot Validation:** Verify bind mounts before chroot
   ```
   Pre-chroot checks:
   [✓] /mnt/dev mounted
   [✓] /mnt/proc mounted
   [✓] /mnt/sys mounted
   [✓] /mnt/run mounted
   [✓] /mnt/etc/resolv.conf present
   ```

4. **Rollback After Repair:** Generate undo script for bootloader changes
   ```
   Rollback script: /var/log/anna/reverts/boot-repair-20251111.sh

   If boot fails after repair:
     1. Boot from ISO again
     2. Run: bash /mnt/var/log/anna/reverts/boot-repair-20251111.sh
   ```

---

## F) Logging and Audit Specification

### Log Paths

```
/var/log/anna/
├── events.jsonl                    # All actions, one JSON object per line
├── install-20251111-091234.log     # Installation session (human-readable)
├── plans/                          # Generated execution scripts
│   ├── installer-20251111-091234.sh
│   ├── update-20251111-093045.sh
│   └── apply-hyprland-20251111-094512.sh
├── reverts/                        # Rollback scripts
│   ├── installer-20251111-091234.sh
│   ├── update-20251111-093045.sh
│   └── apply-hyprland-20251111-094512.sh
└── audit/                          # Historical records
    └── 2025-11.jsonl               # Monthly archive
```

### JSONL Event Schema

**Event Types:**
- `state_change`: System state transitions
- `command_start`: User invoked annactl command
- `command_end`: Command completed or failed
- `action_execute`: Individual action within command (e.g., one pacman operation)
- `health_check`: Scheduled or manual health check result
- `proposal_generated`: Optimization proposal created
- `proposal_applied`: User applied proposal
- `rollback_executed`: User reverted previous action

**Schema:**
```json
{
  "event_id": "uuid-v4",
  "timestamp": "2025-11-11T09:12:34.567Z",
  "event_type": "action_execute",
  "step_id": "install-20251111-091234.step-08",
  "state_before": "iso_live",
  "state_after": "post_install_minimal",
  "command": "install",
  "action": {
    "description": "Install systemd-boot bootloader",
    "command_template": "arch-chroot /mnt bootctl install",
    "executed_command": "arch-chroot /mnt bootctl install",
    "exit_code": 0,
    "stdout": "Created /boot/EFI/systemd/systemd-bootx64.efi...",
    "stderr": "",
    "duration_ms": 234
  },
  "wiki_citations": [
    "[archwiki:systemd-boot]",
    "[archwiki:installation_guide:boot_loader]"
  ],
  "rollback_script": "/var/log/anna/reverts/installer-20251111-091234.sh",
  "user": "root",
  "success": true,
  "metadata": {
    "bootloader": "systemd-boot",
    "esp_path": "/boot"
  }
}
```

**State Change Event:**
```json
{
  "event_id": "a3f1c2d8-...",
  "timestamp": "2025-11-11T09:15:00Z",
  "event_type": "state_change",
  "step_id": null,
  "state_before": "post_install_minimal",
  "state_after": "configured",
  "command": "converge",
  "action": null,
  "wiki_citations": ["[archwiki:system_maintenance]"],
  "rollback_script": null,
  "user": "admin",
  "success": true,
  "metadata": {
    "reason": "Initial health check passed, state adopted"
  }
}
```

**Health Check Event:**
```json
{
  "event_id": "b4d2e9f1-...",
  "timestamp": "2025-11-11T10:00:00Z",
  "event_type": "health_check",
  "step_id": "health-scheduled-20251111-100000",
  "state_before": "configured",
  "state_after": "configured",
  "command": "health",
  "action": {
    "description": "Scheduled health check",
    "checks": {
      "disk_health": "pass",
      "services": "pass",
      "journal": "warn",
      "security": "pass",
      "packages": "pass"
    }
  },
  "wiki_citations": ["[archwiki:system_maintenance#check_for_errors]"],
  "rollback_script": null,
  "user": "system",
  "success": true,
  "metadata": {
    "warnings": ["sshd.service is masked"]
  }
}
```

### Step ID Format

```
<command>-<date>-<time>.<substep>

Examples:
install-20251111-091234.step-01    # Partition disk
install-20251111-091234.step-08    # Install bootloader
update-20251111-093045.step-03     # Pacman database sync
apply-hyprland-20251111-094512     # Single-step action
```

### Citation Field Rules

**Every log entry MUST include `wiki_citations` array with at least one entry, unless:**
- Event type is `state_change` (may reference previous action's citations)
- Event type is `health_check` (uses generic maintenance citation)
- Action is internal housekeeping (log rotation, etc.)

**Citation format:**
```
[archwiki:<page_name>]                          # Whole page
[archwiki:<page_name>#<section>]                # Specific section
[archwiki:<page_name>:<subsection>]             # Installation guide special format
[man:<command>(<section>)]                      # Man page reference

Examples:
[archwiki:systemd-boot]
[archwiki:pacman#upgrading_packages]
[archwiki:installation_guide:partition_the_disks]
[man:sgdisk(8)]
```

### Retention Policy

- `events.jsonl`: Unlimited retention
- Session logs (install-*.log): Unlimited retention
- Plans and reverts: Keep until superseded by new action or 90 days
- Audit archives: Monthly rotation, keep 24 months

### Query Interface

```bash
# Show last 10 events
annactl history --limit 10

# Show events for specific command
annactl history --command install

# Show events with specific citation
annactl history --citation archwiki:systemd-boot

# Show failed actions only
annactl history --failed

# Export history as JSON
annactl history --json > history.json
```

---

## G) Rollback Specification

### Revert Script Layout

**Template:**
```bash
#!/bin/bash
# Anna Rollback Script
# Original action: <command> (<step_id>)
# Generated: <timestamp>
# Citation: <wiki_citations>
#
# WARNING: This script will undo changes made by the original action.
# Review carefully before executing.

set -euo pipefail

# Function: Confirm destructive action
confirm() {
    echo "WARNING: $1"
    read -p "Type 'REVERT' to confirm: " response
    [[ "$response" != "REVERT" ]] && { echo "Cancelled."; exit 1; }
}

# Pre-flight checks
echo "Anna Rollback: <action_description>"
echo "Original action: <timestamp>"
echo

# [State validation]
if [[ ! -f /etc/anna/state.json ]]; then
    echo "ERROR: Not a managed Anna system."
    exit 2
fi

# [Specific rollback steps]
# ...

# [Post-rollback verification]
# ...

# Update Anna state
# ...

echo "Rollback complete."
```

**Example: Update Rollback**
```bash
#!/bin/bash
# Anna Rollback Script
# Original action: update (update-20251111-093045)
# Generated: 2025-11-11T09:30:45Z
# Citation: [archwiki:pacman#upgrading_packages]
#
# This script downgrades packages to versions before the update.

set -euo pipefail

confirm() {
    echo "WARNING: $1"
    read -p "Type 'REVERT' to confirm: " response
    [[ "$response" != "REVERT" ]] && { echo "Cancelled."; exit 1; }
}

echo "Anna Rollback: System Update"
echo "Original action: 2025-11-11 09:30:45"
echo

# Pre-flight checks
if [[ ! -f /var/cache/pacman/pkg/linux-6.11.1-arch1-1-x86_64.pkg.tar.zst ]]; then
    echo "ERROR: Package cache missing. Cannot rollback."
    echo "Restore from backup: /var/backups/anna/pre-update-20251111.tar.zst"
    exit 3
fi

# Confirm
confirm "This will downgrade 42 packages."

# Downgrade packages
echo "Downgrading packages..."
pacman -U --noconfirm \
    /var/cache/pacman/pkg/linux-6.11.1-arch1-1-x86_64.pkg.tar.zst \
    /var/cache/pacman/pkg/systemd-256.1-1-x86_64.pkg.tar.zst \
    /var/cache/pacman/pkg/firefox-131.0-1-x86_64.pkg.tar.zst
    # ... (39 more packages)

# Verify
echo "Verifying downgrade..."
if pacman -Q linux | grep -q "6.11.1"; then
    echo "[✓] Kernel downgraded successfully"
else
    echo "[✗] Kernel downgrade failed"
    exit 4
fi

# Update Anna history
echo "Recording rollback in audit log..."
annactl history --record-rollback update-20251111-093045

echo "Rollback complete."
echo
echo "NOTE: If this was a kernel downgrade, reboot is required."
read -p "Reboot now? [y/N]: " reboot
[[ "$reboot" =~ ^[Yy]$ ]] && systemctl reboot
```

### Idempotency Rules

1. **Check Before Undo:** Verify current state matches expected post-action state
   ```bash
   # Example: Bootloader rollback
   if [[ ! -f /boot/EFI/systemd/systemd-bootx64.efi ]]; then
       echo "Bootloader already removed or never installed. Nothing to revert."
       exit 0
   fi
   ```

2. **Partial Rollback:** If only some steps succeeded, only revert completed steps
   ```bash
   if [[ -f /tmp/anna-rollback-state.json ]]; then
       completed_steps=$(jq -r '.completed_steps[]' /tmp/anna-rollback-state.json)
       # Revert only completed steps in reverse order
   fi
   ```

3. **No Re-execution:** Rollback scripts should not run twice
   ```bash
   if grep -q "rollback-$(basename "$0")" /var/log/anna/events.jsonl; then
       echo "ERROR: This rollback was already executed."
       exit 5
   fi
   ```

### Verification After Revert

**Checklist:**
```bash
# After package downgrade
verify_package_versions() {
    local expected_versions="$1"
    while IFS='=' read -r pkg ver; do
        current=$(pacman -Q "$pkg" | awk '{print $2}')
        if [[ "$current" != "$ver" ]]; then
            echo "[✗] $pkg: expected $ver, got $current"
            return 1
        fi
    done < "$expected_versions"
    return 0
}

# After bootloader revert
verify_bootloader_removed() {
    if [[ -f /boot/EFI/systemd/systemd-bootx64.efi ]]; then
        echo "[✗] Bootloader still present"
        return 1
    fi
    echo "[✓] Bootloader removed"
    return 0
}

# After filesystem revert (if applicable)
verify_filesystem_clean() {
    local device="$1"
    if blkid "$device" >/dev/null 2>&1; then
        echo "[✗] Filesystem signature still present"
        return 1
    fi
    echo "[✓] Filesystem wiped"
    return 0
}
```

### Non-Reversible Actions

Some actions cannot be fully reverted. These require clear warnings:

| Action | Reversible? | Reason | Mitigation |
|--------|-------------|--------|------------|
| Partition table changes | No | Data destroyed | Pre-backup required, rollback script wipes disk |
| Package removal | Partial | Config files may be lost | Backup /etc before removal |
| Bootloader install | Yes | Can be removed | Revert script removes EFI entries |
| System update | Yes (with cache) | Downgrade from cache | Keep pacman cache or snapshot |
| User creation | Partial | Home dir remains | Revert removes user, leaves ~/ |

**Non-reversible action template:**
```bash
# NON-REVERSIBLE ACTION WARNING
echo "=============================================="
echo "WARNING: THIS ACTION CANNOT BE FULLY REVERSED"
echo "=============================================="
echo
echo "Action: Repartition /dev/sda"
echo "Impact: ALL DATA ON /dev/sda WILL BE LOST"
echo
echo "The rollback script can only wipe the disk again,"
echo "but cannot recover your previous data."
echo
echo "If you have not backed up, CANCEL NOW."
echo
confirm "I understand this is destructive and have backed up my data"
```

---

## H) Security Specification

### Whitelist Model

Anna uses a **command template whitelist** - no arbitrary shell execution allowed.

**Template Definition:**
```rust
pub struct CommandTemplate {
    pub id: String,
    pub description: String,
    pub command: Vec<String>,  // argv array
    pub requires_root: bool,
    pub allowed_states: Vec<SystemState>,
    pub parameter_validation: HashMap<String, Validator>,
    pub wiki_citation: String,
}

pub enum Validator {
    DevicePath,      // Must match /dev/[a-z]+[0-9]*
    PackageName,     // Must match [a-z0-9@._+-]+
    Hostname,        // RFC 1123 compliance
    Path,            // Absolute path, no ../
    Choice(Vec<String>),  // Must be one of enum values
    Regex(String),   // Custom regex validation
}
```

**Example Templates:**
```rust
CommandTemplate {
    id: "install_package",
    description: "Install package from official repos",
    command: vec!["pacman", "-S", "--noconfirm", "{{package}}"],
    requires_root: true,
    allowed_states: vec![Configured],
    parameter_validation: hashmap! {
        "package" => Validator::PackageName,
    },
    wiki_citation: "[archwiki:pacman#installing_packages]",
}

CommandTemplate {
    id: "bootloader_install",
    description: "Install systemd-boot",
    command: vec!["bootctl", "install"],
    requires_root: true,
    allowed_states: vec![IsoLive, RecoveryCandidate],
    parameter_validation: hashmap! {},
    wiki_citation: "[archwiki:systemd-boot]",
}

CommandTemplate {
    id: "partition_disk",
    description: "Create GPT partition table",
    command: vec!["sgdisk", "--zap-all", "{{device}}"],
    requires_root: true,
    allowed_states: vec![IsoLive],
    parameter_validation: hashmap! {
        "device" => Validator::DevicePath,
    },
    wiki_citation: "[archwiki:gdisk]",
}
```

### Execution Flow

```
User → annactl → RPC call → annad → Template lookup → Validation → Execution
                                        ↓
                                   Whitelist
                                   ↓
                              State check
                              ↓
                         Parameter validation
                         ↓
                    Sudo boundary check
                    ↓
               Log + Execute + Audit
```

### No Arbitrary Shell Execution

**Blocked:**
```rust
// ❌ NEVER ALLOWED
let user_input = "rm -rf /";
Command::new("sh").arg("-c").arg(user_input).spawn();

// ❌ NEVER ALLOWED
let cmd = format!("pacman -S {}", user_package);
Command::new("sh").arg("-c").arg(&cmd).spawn();
```

**Allowed:**
```rust
// ✓ Template-based with validation
let template = get_template("install_package")?;
validate_parameter("package", user_package, &template)?;
let mut cmd = Command::new(&template.command[0]);
for arg in &template.command[1..] {
    if arg.contains("{{") {
        cmd.arg(substitute_parameter(arg, &params)?);
    } else {
        cmd.arg(arg);
    }
}
cmd.spawn()?;
```

### Temp File Policy

- Temp files: `/tmp/anna-<action>-<uuid>/`
- Permissions: `0700` (owner read/write/exec only)
- Cleanup: Automatic on process exit (RAII pattern)
- No user-supplied paths in temp file names

```rust
fn create_temp_dir() -> Result<PathBuf> {
    let uuid = Uuid::new_v4();
    let path = PathBuf::from(format!("/tmp/anna-install-{}", uuid));
    fs::create_dir(&path)?;
    fs::set_permissions(&path, Permissions::from_mode(0o700))?;
    Ok(path)
}
```

### Sudo Boundaries

- **annad runs as root** (systemd service)
- **annactl runs as unprivileged user**
- **RPC socket: `/run/anna/anna.sock`**
  - Owner: root:anna
  - Mode: 0770
  - Users in 'anna' group can connect

**Privilege Escalation:**
- annactl never uses `sudo` directly
- All privileged operations go through annad's RPC API
- annad validates:
  1. User is in 'anna' group
  2. Command is in whitelist
  3. State allows command
  4. Parameters are valid

### Noninteractive Pacman Rules

All pacman operations run with `--noconfirm` in automation, but:
- **Dry-run required first** for user review
- **Confirmation prompt** before execution (outside pacman)
- **Transaction logged** with full package list and versions

```bash
# Phase 1: Dry-run
pacman -Syu --print-format='%n %v' > /tmp/anna-update-plan.txt

# Phase 2: Show to user
cat /tmp/anna-update-plan.txt

# Phase 3: User confirms (outside pacman)
read -p "Proceed? [y/N]: " confirm

# Phase 4: Execute non-interactively
pacman -Syu --noconfirm
```

### Input Validation Rules

| Input Type | Validation | Example |
|------------|------------|---------|
| Device path | Regex: `^/dev/[a-z]+[0-9]*$` | `/dev/sda1` ✓, `/dev/../etc/passwd` ✗ |
| Package name | Regex: `^[a-z0-9@._+-]+$` | `firefox` ✓, `rm -rf /` ✗ |
| Hostname | RFC 1123, max 253 chars | `archlinux` ✓, `foo..bar` ✗ |
| Filesystem path | Absolute path, no `../ `, exists | `/mnt` ✓, `../../etc` ✗ |
| UUID | Regex: `^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$` | Valid UUID v4 |
| Enum choice | Must be in predefined list | `ext4`, `btrfs` ✓, `hackfs` ✗ |

**Validation failures abort immediately with clear error:**
```
ERROR: Invalid device path
Provided: /dev/../etc/passwd
Reason: Path traversal detected
Expected format: /dev/<device> (e.g., /dev/sda, /dev/nvme0n1)
```

---

## I) Removal Plan

### Code to Delete

```
Files to DELETE (Total ~17,000 lines):

TUI Code:
  crates/annactl/src/tui/                (~2,500 lines)
  crates/annactl/src/commands/tui.rs     (~300 lines)

ALL Desktop Environment Config Logic:
  crates/annad/src/hyprland_config.rs    (~800 lines)
  crates/annad/src/i3_config.rs          (~800 lines)
  crates/annad/src/sway_config.rs        (~850 lines)
  crates/annad/src/gnome_config.rs       (~900 lines)
  crates/annad/src/kde_config.rs         (~950 lines)
  crates/annad/src/xfce_config.rs        (~700 lines)
  crates/annad/src/cinnamon_config.rs    (~750 lines)
  crates/annad/src/mate_config.rs        (~650 lines)
  crates/annad/src/lxqt_config.rs        (~600 lines)
  crates/annad/src/wallpaper_config.rs   (~400 lines)
  crates/annad/src/shell_config.rs       (~500 lines)
  crates/annad/src/terminal_config.rs    (~600 lines)
  crates/annad/src/git_config.rs         (~450 lines)

Heuristic/Smart Recommender:
  crates/annad/src/smart_recommender.rs  (~1,800 lines)
  crates/annad/src/intelligent_recommender.rs (~2,200 lines)

ALL Bundle System Code:
  crates/annad/src/bundles.rs            (~1,500 lines - DELETE ENTIRELY)

Resource Classifier:
  crates/annad/src/resource_classifier.rs (~800 lines - DELETE)

System Detection (WM detection):
  crates/annad/src/system_detection.rs   (~900 lines - DELETE)

Total: ~17,050 lines to delete
Remaining codebase: ~7,000 lines (estimate)
New code (installer, rescue, state, desktop admin): ~6,000 lines (estimate)
Final codebase: ~13,000 lines

Scope: Anna is a system administrator + desktop administrator.
NOT a desktop environment installer, NOT a theme manager, NOT a user customization tool.
```

### Dependency Changes

**Remove from Cargo.toml:**
```toml
# TUI dependencies - DELETE
ratatui = "0.26"
crossterm = "0.27"
console = "0.15"  # If only used by TUI
```

**Keep:**
```toml
# Core dependencies
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4.0", features = ["derive", "cargo"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Add for installer/rescue
nix = { version = "0.27", features = ["mount", "fs"] }
libc = "0.2"
regex = "1.10"
```

### Feature Gates (Temporary)

For gradual migration, use feature gates:

```rust
// crates/annad/src/main.rs
#![cfg_attr(not(feature = "v1_legacy"), allow(dead_code))]

#[cfg(feature = "v1_legacy")]
mod smart_recommender;

#[cfg(feature = "v1_legacy")]
mod intelligent_recommender;

// In Cargo.toml
[features]
default = []
v1_legacy = []  # Enable old code during migration
```

Build without legacy:
```bash
cargo build --release --no-default-features
```

### Migration Path

**Phase 0 (Removal):**
1. Delete TUI code entirely (no feature gate needed)
2. Delete non-Hyprland desktop modules
3. Add feature gate to recommender code
4. Update imports and re-exports

**Phase 1-7 (Implementation):**
- New code coexists with gated legacy code
- Tests verify new behavior in isolation

**Phase 8 (Cutover):**
- Remove feature gate
- Delete gated legacy code
- Final cleanup

---

## J) Testing Plan

### Unit Tests

**Coverage Requirements:**
- State detection: 100%
- Command validation: 100%
- Template substitution: 100%
- Citation parsing: 100%
- Rollback script generation: 100%

**Example Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_detection_iso_live() {
        // Mock environment with /run/archiso
        let state = detect_state_with_mock("/run/archiso");
        assert_eq!(state, SystemState::IsoLive);
    }

    #[test]
    fn test_command_validation_device_path() {
        let validator = Validator::DevicePath;
        assert!(validator.validate("/dev/sda").is_ok());
        assert!(validator.validate("/dev/../etc/passwd").is_err());
    }

    #[test]
    fn test_template_substitution() {
        let template = vec!["pacman", "-S", "{{package}}"];
        let params = hashmap! { "package" => "firefox" };
        let result = substitute_params(&template, &params).unwrap();
        assert_eq!(result, vec!["pacman", "-S", "firefox"]);
    }

    #[test]
    fn test_wiki_citation_format() {
        let citation = WikiCitation::parse("[archwiki:systemd-boot#installation]");
        assert_eq!(citation.page, "systemd-boot");
        assert_eq!(citation.section, Some("installation"));
    }
}
```

### CLI Contract Tests

Verify command availability per state:

```rust
#[test]
fn test_install_only_in_iso_live() {
    let caps = get_capabilities(SystemState::IsoLive);
    assert!(caps.commands.iter().any(|c| c.name == "install"));

    let caps = get_capabilities(SystemState::Configured);
    assert!(!caps.commands.iter().any(|c| c.name == "install"));
}

#[test]
fn test_update_only_in_configured() {
    let caps = get_capabilities(SystemState::Configured);
    assert!(caps.commands.iter().any(|c| c.name == "update"));

    let caps = get_capabilities(SystemState::IsoLive);
    assert!(!caps.commands.iter().any(|c| c.name == "update"));
}
```

### QEMU End-to-End Tests

**Test Matrix:**

| Test Case | Boot Mode | Disk Layout | Filesystem | Encryption | Expected Result |
|-----------|-----------|-------------|------------|------------|-----------------|
| minimal_uefi_ext4 | UEFI | Single disk, GPT | ext4 | No | ✓ Boots to login |
| minimal_bios_ext4 | BIOS | Single disk, MBR | ext4 | No | ✓ Boots to login |
| minimal_uefi_btrfs | UEFI | Single disk, GPT | btrfs | No | ✓ Boots to login |
| minimal_uefi_luks | UEFI | Single disk, GPT | ext4 | LUKS | ✓ Boots to passphrase prompt |
| dualboot_windows | UEFI | 2 disks | ext4 | No | ✓ Boots Arch, Windows untouched |
| rescue_broken_boot | UEFI | Pre-installed, broken bootloader | ext4 | No | ✓ boot-repair succeeds, system boots |

**QEMU Test Harness:**
```bash
#!/bin/bash
# tests/qemu/run_test.sh

TEST_NAME="$1"
ISO_PATH="archlinux-x86_64.iso"
DISK_SIZE="20G"

# Create test disk
qemu-img create -f qcow2 "/tmp/${TEST_NAME}.qcow2" "$DISK_SIZE"

# Boot ISO, run automated installer
qemu-system-x86_64 \
    -m 4G \
    -smp 4 \
    -enable-kvm \
    -cdrom "$ISO_PATH" \
    -drive file="/tmp/${TEST_NAME}.qcow2",format=qcow2 \
    -boot order=d \
    -serial mon:stdio \
    -nographic \
    -device virtio-net-pci,netdev=net0 \
    -netdev user,id=net0 \
    -bios /usr/share/ovmf/x64/OVMF.fd \
    -monitor unix:/tmp/qemu-monitor-${TEST_NAME}.sock,server,nowait

# Automated installer commands via serial console:
# 1. Wait for ISO boot
# 2. Run: annactl install --no-confirm --config /tmp/test-config.json
# 3. Wait for completion
# 4. Power off

# Boot installed system
qemu-system-x86_64 \
    -m 4G \
    -smp 4 \
    -enable-kvm \
    -drive file="/tmp/${TEST_NAME}.qcow2",format=qcow2 \
    -boot order=c \
    -serial mon:stdio \
    -nographic

# Verify:
# 1. System boots to login prompt
# 2. Can log in as created user
# 3. annactl commands work
# 4. Health check passes

echo "Test ${TEST_NAME}: PASS"
```

**CI Integration (.github/workflows/test-matrix.yml):**
```yaml
name: QEMU Test Matrix

on: [push, pull_request]

jobs:
  qemu_tests:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        test:
          - minimal_uefi_ext4
          - minimal_bios_ext4
          - minimal_uefi_btrfs
          - rescue_broken_boot
    steps:
      - uses: actions/checkout@v4

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y qemu-system-x86 ovmf

      - name: Download Arch ISO
        run: |
          wget https://mirror.rackspace.com/archlinux/iso/latest/archlinux-x86_64.iso

      - name: Build Anna
        run: cargo build --release --no-default-features

      - name: Run QEMU test
        run: |
          bash tests/qemu/run_test.sh ${{ matrix.test }}
        timeout-minutes: 30

      - name: Upload test artifacts
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.test }}-artifacts
          path: |
            /tmp/${{ matrix.test }}.qcow2
            /var/log/anna/
```

### Failure Injection Tests

Verify error handling and rollback:

```rust
#[test]
fn test_installer_partition_failure() {
    // Mock sgdisk failure
    let result = run_installer_with_mock_command_failure("sgdisk");
    assert!(result.is_err());

    // Verify rollback script was generated
    assert!(Path::new("/var/log/anna/reverts/installer-*.sh").exists());

    // Verify disk was not modified (or wiped if partial)
}

#[test]
fn test_update_pacman_failure() {
    // Mock pacman -Syu failure
    let result = run_update_with_mock_failure();
    assert!(result.is_err());

    // Verify snapshot was not deleted
    assert!(Path::new("/var/backups/anna/pre-update-*.tar.zst").exists());

    // Verify rollback instructions printed
}

#[test]
fn test_bootloader_install_failure() {
    // Mock bootctl failure
    let result = run_boot_repair_with_mock_failure();
    assert!(result.is_err());

    // Verify system was unmounted cleanly
    assert!(!is_mounted("/mnt"));

    // Verify revert script can clean up partial changes
}
```

### Pass List for 1.0 Release

All tests must pass before 1.0 release:

**Unit Tests:**
- [x] State detection (all 6 states)
- [x] Command template validation
- [x] Parameter validation (all validator types)
- [x] Wiki citation parsing
- [x] Rollback script generation
- [x] JSONL event logging

**Integration Tests:**
- [x] RPC capabilities API
- [x] State transitions (all valid paths)
- [x] Command filtering per state

**QEMU End-to-End:**
- [x] minimal_uefi_ext4 (boots and health passes)
- [x] minimal_bios_ext4 (boots and health passes)
- [x] minimal_uefi_btrfs (boots and health passes)
- [x] rescue_broken_boot (repair succeeds, boots)

**Failure Injection:**
- [x] Installer partition failure (rollback generated)
- [x] Update pacman failure (snapshot preserved)
- [x] Bootloader failure (clean unmount)

**Manual Verification:**
- [x] Install on real hardware (UEFI + ext4)
- [x] Install with Hyprland bundle
- [x] Rescue broken system (bootloader repair)
- [x] Update with rollback
- [x] All logs include Wiki citations

---

## K) Documentation Plan

### Files to Create/Update

**New Documentation:**
1. `docs/STATE-MACHINE.md` - State definitions and transition rules
2. `docs/INSTALLER-GUIDE.md` - Installation walkthrough with screenshots
3. `docs/RESCUE-GUIDE.md` - Recovery procedures
4. `docs/MIGRATION-1.0.md` - Breaking changes from RC.11

**Updated Documentation:**
1. `README.md` - Reflect new installer and state-aware model
2. `INSTALL.md` - Update installation instructions (now: boot ISO, run annactl install)
3. `SECURITY_AUDIT.md` - Document whitelist model and security boundaries
4. `docs/IPC-API.md` - Add capabilities endpoint and state-aware RPC

### README.md Updates

**New Structure:**
```markdown
# Anna Assistant

Your Arch Linux system administrator companion.

**Version:** 1.0.0 (Stable Release)

Anna is a state-aware system management tool for Arch Linux that:
- Installs Arch Linux interactively with safe defaults
- Monitors system health and suggests optimizations
- Rescues broken systems (bootloader, initramfs, pacman)
- Maintains your system with automatic backups and rollback

## Quick Start

### Install Arch Linux

Boot the Arch ISO and run:
```bash
annactl install
```

Follow the interactive prompts. Anna will:
1. Detect your hardware
2. Guide you through disk layout
3. Install base system + bootloader
4. Create an admin user
5. Optionally install Hyprland desktop

### On an Existing System

Install Anna on a running Arch system:
```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

Then adopt the system:
```bash
annactl converge
```

## State-Aware Commands

Anna adapts to your system's state:

| State | Available Commands |
|-------|-------------------|
| **ISO Live** | install, rescue-detect, rescue-chroot |
| **Configured** | update, backup, health, advise, apply |
| **Degraded** | triage, rollback-last, collect-logs |

Run `annactl status` to see current state.

## Documentation

- [Installation Guide](docs/INSTALLER-GUIDE.md)
- [Recovery Guide](docs/RESCUE-GUIDE.md)
- [State Machine](docs/STATE-MACHINE.md)
- [Security Model](SECURITY_AUDIT.md)
- [API Reference](docs/IPC-API.md)

## Features

### Installation
- Interactive prompts with safe defaults
- Dual-boot with Windows support
- LUKS encryption optional
- Hyprland desktop bundle
- Generates rollback scripts

### Maintenance
- Automated updates with snapshots
- Health monitoring (SMART, systemd, journals)
- Arch News integration
- Intelligent mirror ranking

### Recovery
- Boot repair (systemd-boot, GRUB)
- Filesystem checks (ext4, btrfs)
- Initramfs rebuild
- Pacman database recovery

### Security
- Command whitelist (no arbitrary shell)
- Wiki-only recommendations
- Comprehensive audit logging
- Group-based access control

## Architecture

```
┌─────────────────────────────────────┐
│         annactl (CLI)               │
│  State-aware command dispatcher     │
└──────────────┬──────────────────────┘
               │ Unix Socket RPC
               ▼
┌─────────────────────────────────────┐
│         annad (Daemon)              │
│  • State detection                  │
│  • Capabilities API                 │
│  • Command whitelist                │
│  • Installer engine                 │
│  • Rescue tools                     │
│  • Maintainer automation            │
│  • Audit logging with citations     │
└─────────────────────────────────────┘
```

## Wiki-Strict Correctness

All Anna operations cite Arch Wiki pages:
```
[✓] Install systemd-boot
    Citation: [archwiki:systemd-boot]
[✓] Generate initramfs
    Citation: [archwiki:mkinitcpio]
```

Every log entry includes Wiki references for verification.

## License

GPL-3.0-or-later

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

All contributions must cite relevant Arch Wiki sections.
```

### MIGRATION-1.0.md

```markdown
# Migration Guide: RC.11 → 1.0

**Breaking Changes in Anna 1.0**

## Removed Features

### TUI (Terminal UI)
**Status:** Removed for 1.0
**Reason:** Complexity, maintenance burden, stability concerns

**Migration:**
- Use CLI commands instead: `annactl advise`, `annactl history`
- TUI may return in v1.1 (Eurydice milestone)

### ALL Desktop Environment Support
**Status:** Removed for 1.0
**Reason:** Anna is a system and desktop administrator, not a desktop environment installer

**Desktop Environments Removed:**
- Hyprland
- i3, Sway
- GNOME, KDE Plasma
- XFCE, Cinnamon, MATE, LXQt
- ALL bundles and WM-specific configuration logic

**Migration:**
- Anna will NOT configure any desktop environment
- Anna will NOT install desktop packages
- Desktop administration proposals available: Bluetooth, CUPS printing, power profiles, XDG directories
- Install and configure desktop environment manually via pacman and Arch Wiki
- Anna focuses on system reliability, not desktop aesthetics

### Smart/Heuristic Recommenders
**Status:** Removed for 1.0
**Reason:** Non-deterministic, not Wiki-based

**Migration:**
- Only Wiki-cited proposals will be generated
- Custom optimizations: Apply manually

## New Requirements

### State Adoption
Existing Anna systems must adopt state file:
```bash
sudo annactl converge
```

This creates `/etc/anna/state.json` with system configuration.

### Group Membership
Users must be in `anna` group to run commands:
```bash
sudo usermod -aG anna $USER
```

Log out and back in to apply.

## Changed Commands

### `annactl tui` → Removed
Use `annactl advise` and `annactl history` instead.

### `annactl apply <number>` → `annactl apply <id>`
Proposals now use stable IDs instead of numbers.

**Before (RC.11):**
```bash
annactl apply 3
```

**After (1.0):**
```bash
annactl advise
# [hyprland-vaapi-001] Enable VA-API
annactl apply hyprland-vaapi-001
```

### `annactl update` (new behavior)
Now checks Arch News before updating and requires explicit confirmation.

## New Features

### Interactive Installer
Boot Arch ISO and run:
```bash
annactl install
```

Replaces manual Wiki installation steps.

### Rescue Mode
Recover broken systems from ISO:
```bash
annactl rescue-detect
annactl rescue-chroot --root-device=/dev/sda2
```

### Comprehensive Logging
All actions now logged to `/var/log/anna/events.jsonl` with Wiki citations.

### Atomic Rollback
Every action generates a revert script in `/var/log/anna/reverts/`.

## Configuration Changes

### Old: `/etc/anna/config.toml` → Removed
TOML configuration is removed in 1.0.

### New: `/etc/anna/state.json` (generated automatically)
State file is managed by Anna, do not edit manually.

## Data Migration

### Audit Logs
Old audit logs (`/var/log/anna/audit.jsonl`) are preserved but not used by 1.0.

New events use `/var/log/anna/events.jsonl` with enhanced schema.

### History
Old history is accessible via:
```bash
annactl history --legacy
```

## Upgrade Path

### From RC.11:
```bash
# 1. Stop daemon
sudo systemctl stop annad

# 2. Backup old logs
sudo cp -r /var/log/anna /var/backups/anna-rc11-logs

# 3. Update Anna
sudo pacman -Syu anna-assistant

# 4. Adopt state
sudo annactl converge

# 5. Verify
annactl status
annactl health
```

### Rollback to RC.11:
```bash
# Downgrade package
sudo pacman -U /var/cache/pacman/pkg/anna-assistant-1.0.0-rc.11-*.pkg.tar.zst

# Restore old logs (optional)
sudo cp -r /var/backups/anna-rc11-logs/* /var/log/anna/

# Restart daemon
sudo systemctl restart annad
```

## Support

- File issues: https://github.com/jjgarcianorway/anna-assistant/issues
- Check Wiki: https://github.com/jjgarcianorway/anna-assistant/wiki
- Review logs: `/var/log/anna/events.jsonl`
```

---

## Execution Phases

### Phase 0: Cleanup and Foundation (3 days)

**Goals:**
- Remove TUI code
- Remove non-Hyprland desktop modules
- Add feature gates for legacy code
- Land state machine skeleton

**Deliverables:**
- [ ] Delete `crates/annactl/src/tui/`
- [ ] Delete desktop modules (i3, sway, gnome, kde, xfce, cinnamon, mate, lxqt)
- [ ] Delete `smart_recommender.rs`, `intelligent_recommender.rs`
- [ ] Add `v1_legacy` feature gate
- [ ] Update Cargo.toml (remove ratatui, crossterm)
- [ ] Create `anna_common::state` module with state types
- [ ] Create `annad::state::detector` stub (returns `Unknown` for now)
- [ ] Create `annad::state::capabilities` stub (returns empty list)
- [ ] All tests pass (existing tests, ignoring removed modules)

**Commits:**
- `chore: remove TUI code for 1.0 stability focus`
- `chore: remove non-Hyprland desktop modules (1.0 scope reduction)`
- `chore: gate legacy recommender code with v1_legacy feature`
- `feat: add state machine types and detection skeleton`

---

### Phase 1: State Detection and Capabilities API (5 days)

**Goals:**
- Implement full state detection logic
- Implement capabilities API in annad
- Unit tests for all state transitions

**Deliverables:**
- [ ] `state::detector::detect_state()` fully implemented
- [ ] `state::detector::is_iso_environment()` (check `/run/archiso`)
- [ ] `state::detector::is_arch_install()` (check `/etc/arch-release`)
- [ ] `state::detector::health_check_passes()` (systemd, journal, fs)
- [ ] `state::capabilities::get_capabilities(state)` returns command list
- [ ] RPC endpoint `/capabilities` in `rpc_server`
- [ ] Unit tests: all 6 states detected correctly
- [ ] Unit tests: state transitions validated
- [ ] Integration test: RPC capabilities API

**Commits:**
- `feat(state): implement state detection logic [archwiki:installation_guide]`
- `feat(state): implement health check validation [archwiki:system_maintenance]`
- `feat(state): implement capabilities API with command whitelist`
- `test(state): add unit tests for all state detection paths`
- `feat(rpc): add /capabilities endpoint for state-aware CLI`

---

### Phase 2: annactl State-Aware Dispatch (4 days)

**Goals:**
- Refactor annactl main to query capabilities before dispatching
- Implement command filtering per state
- User-friendly error messages for blocked commands

**Deliverables:**
- [ ] `annactl` calls `/capabilities` on startup
- [ ] Command dispatch checks if command is allowed
- [ ] Blocked commands show helpful error: "Command 'update' not available in iso_live state. Available: install, rescue-detect, hardware-report"
- [ ] `annactl status` shows current state
- [ ] Unit tests: command filtering per state
- [ ] Integration test: blocked command handling

**Commits:**
- `feat(annactl): query daemon capabilities on startup`
- `feat(annactl): implement state-aware command filtering`
- `feat(annactl): add 'status' command showing current state`
- `test(annactl): verify command availability per state`

---

### Phase 3: Interactive Installer (10 days)

**Goals:**
- Implement full installation wizard
- Partition, format, mount, pacstrap, bootloader, users
- Generate state.json, plans/, reverts/, postinstall.md
- Dry-run mode

**Deliverables:**
- [ ] `installer::prompts` module (all 8 steps)
- [ ] `installer::partition` - disk layout and partitioning [archwiki:partition_the_disks]
- [ ] `installer::filesystem` - format and mount [archwiki:format_the_partitions]
- [ ] `installer::bootstrap` - pacstrap, genfstab, chroot config [archwiki:install_essential_packages]
- [ ] `installer::rollback` - generate revert script
- [ ] Windows detection and dual-boot support [archwiki:dual_boot_with_windows]
- [ ] LUKS encryption support [archwiki:dm-crypt]
- [ ] Bootloader install (systemd-boot) [archwiki:systemd-boot]
- [ ] User creation with sudo [archwiki:users_and_groups]
- [ ] Generate `/etc/anna/state.json`
- [ ] Generate `/var/log/anna/plans/installer-*.sh`
- [ ] Generate `/var/log/anna/reverts/installer-*.sh`
- [ ] Generate `/root/postinstall.md`
- [ ] `--dry-run` flag shows plan without executing
- [ ] Unit tests: prompt validation, partition planning
- [ ] QEMU test: minimal_uefi_ext4 boots successfully

**Commits:**
- `feat(installer): add step 1-2 prompts (purpose, disk selection)`
- `feat(installer): add step 3-4 (partition plan, encryption)`
- `feat(installer): add step 5-6 (network, locale, users)`
- `feat(installer): add step 7-8 (packages, confirmation)`
- `feat(installer): implement partition creation [archwiki:gdisk]`
- `feat(installer): implement filesystem format and mount [archwiki:file_systems]`
- `feat(installer): implement pacstrap and base install [archwiki:pacstrap]`
- `feat(installer): implement bootloader installation [archwiki:systemd-boot]`
- `feat(installer): generate state.json and rollback scripts`
- `feat(installer): add Windows detection for dual-boot [archwiki:dual_boot_with_windows]`
- `test(installer): add QEMU end-to-end test for minimal install`

---

### Phase 4: Rescue Mode (7 days)

**Goals:**
- Detect broken installations
- Implement rescue playbooks
- Chroot tooling with bind mounts

**Deliverables:**
- [ ] `rescue::detect` - scan for Linux roots, detect issues
- [ ] `rescue::chroot` - mount, bind, arch-chroot [archwiki:chroot]
- [ ] `rescue::boot_repair` - bootloader reinstall [archwiki:systemd-boot]
- [ ] `rescue::fs_check` - fsck for ext4, btrfs check [archwiki:fsck]
- [ ] `annactl rescue-detect` command
- [ ] `annactl rescue-chroot` command
- [ ] `annactl rescue-overview` command
- [ ] `annactl boot-repair` command (automated playbook)
- [ ] `annactl fs-check` command
- [ ] Unit tests: rescue detection matrix
- [ ] QEMU test: rescue_broken_boot repairs and boots

**Commits:**
- `feat(rescue): implement Linux root detection`
- `feat(rescue): implement chroot with bind mounts [archwiki:chroot]`
- `feat(rescue): add boot-repair playbook [archwiki:systemd-boot]`
- `feat(rescue): add filesystem check playbook [archwiki:fsck]`
- `feat(annactl): add rescue commands (detect, chroot, boot-repair)`
- `test(rescue): add QEMU test for bootloader repair`

---

### Phase 5: Maintainer Update Flow (6 days)

**Goals:**
- Update command with Arch News check
- Snapshot before update
- Rollback script generation
- Mirror ranking with reflector

**Deliverables:**
- [ ] `maintainer::update` - update flow with safeguards [archwiki:system_maintenance#upgrading_the_system]
- [ ] Check Arch News API (https://archlinux.org/feeds/news/)
- [ ] Require user acknowledgment of news items
- [ ] Create snapshot before update (tar or btrfs snapshot)
- [ ] Rank mirrors with reflector [archwiki:reflector]
- [ ] Refresh keyring [archwiki:pacman/package_signing]
- [ ] Run `pacman -Syu --noconfirm` after user confirms
- [ ] Generate rollback script with package downgrades
- [ ] Detect kernel update and prompt for reboot
- [ ] Unit tests: news parsing, snapshot creation
- [ ] Integration test: update with rollback

**Commits:**
- `feat(maintainer): implement Arch News check in update flow`
- `feat(maintainer): add pre-update snapshot creation`
- `feat(maintainer): integrate reflector for mirror ranking [archwiki:reflector]`
- `feat(maintainer): implement safe update with keyring refresh [archwiki:pacman]`
- `feat(maintainer): generate rollback script for updates`
- `test(maintainer): verify update rollback restores packages`

---

### Phase 6: Optimizer Proposals (5 days)

**Goals:**
- Generate Wiki-cited proposals for system & desktop administration
- Apply proposals with whitelist validation
- Revert functionality

**Deliverables:**
- [ ] `optimizer::advise` - generate proposals with citations
- [ ] `optimizer::suggest` - filter by category
- [ ] `optimizer::apply` - execute whitelisted commands
- [ ] `optimizer::revert` - undo applied proposal
- [ ] System admin proposals (microcode, fstrim, zram, SSH hardening, firewall, backups)
- [ ] Desktop admin proposals (Bluetooth, CUPS printing, power profiles, XDG directories)
- [ ] Proposal IDs (stable across runs)
- [ ] Preview mode (show markdown with commands)
- [ ] Dependency checking (don't suggest config if package not installed)
- [ ] Unit tests: proposal generation, dependency resolution
- [ ] Integration test: apply and revert proposal

**Commits:**
- `feat(optimizer): implement Wiki-strict proposal generator`
- `feat(optimizer): add system administration proposals [archwiki:system_maintenance]`
- `feat(optimizer): add desktop hardware administration proposals [archwiki:bluetooth] [archwiki:cups]`
- `feat(optimizer): implement apply with whitelist validation`
- `feat(optimizer): implement revert with rollback scripts`
- `test(optimizer): verify proposals cite Arch Wiki correctly`

**Note:** NO desktop environment setup, NO theme/aesthetic proposals, NO user customization.

---

### Phase 7: Logging and Audit Pipeline (4 days)

**Goals:**
- JSONL event logging
- Wiki citation fields in every log entry
- History query interface

**Deliverables:**
- [ ] `audit::logger` - append-only JSONL writer
- [ ] `audit::citations` - Wiki citation parser and validator
- [ ] `audit::history` - query interface for events.jsonl
- [ ] Event schema with step_id, wiki_citations, rollback_script fields
- [ ] `annactl history` command with filters (--command, --citation, --failed)
- [ ] `annactl history --json` for machine-readable output
- [ ] Log rotation (monthly archives)
- [ ] Unit tests: JSONL parsing, citation validation
- [ ] Integration test: events logged correctly during install

**Commits:**
- `feat(audit): implement JSONL event logging with citations`
- `feat(audit): add Wiki citation parser and validator`
- `feat(annactl): add 'history' command for audit log queries`
- `test(audit): verify all events include Wiki citations`

---

### Phase 8: Test Matrix Automation (5 days)

**Goals:**
- QEMU test harness
- CI integration with matrix
- Failure injection tests

**Deliverables:**
- [ ] `tests/qemu/run_test.sh` - automated QEMU runner
- [ ] QEMU tests: minimal_uefi_ext4, minimal_bios_ext4, minimal_uefi_btrfs, rescue_broken_boot
- [ ] GitHub Actions workflow: `.github/workflows/test-matrix.yml`
- [ ] Failure injection tests (partition failure, pacman failure, bootloader failure)
- [ ] Test artifacts uploaded on failure
- [ ] CI green on all matrix tests

**Commits:**
- `test(qemu): add QEMU test harness for end-to-end validation`
- `test(qemu): add minimal UEFI+ext4 installation test`
- `test(qemu): add BIOS boot and btrfs tests`
- `test(qemu): add rescue mode bootloader repair test`
- `test(ci): add GitHub Actions matrix for QEMU tests`
- `test: add failure injection tests for rollback verification`

---

### Phase 9: Documentation and Release (4 days)

**Goals:**
- Update all documentation
- Migration guide
- Release notes

**Deliverables:**
- [ ] `README.md` - updated for 1.0 features
- [ ] `docs/STATE-MACHINE.md` - state definitions and transitions
- [ ] `docs/INSTALLER-GUIDE.md` - installation walkthrough
- [ ] `docs/RESCUE-GUIDE.md` - recovery procedures
- [ ] `docs/MIGRATION-1.0.md` - breaking changes from RC.11
- [ ] `SECURITY_AUDIT.md` - document whitelist model
- [ ] `docs/IPC-API.md` - updated with capabilities endpoint
- [ ] `CHANGELOG.md` - 1.0 release notes
- [ ] Update `TESTING.md` with new test coverage
- [ ] PR description with full feature list

**Commits:**
- `docs: update README for 1.0 release`
- `docs: add STATE-MACHINE.md with state definitions`
- `docs: add INSTALLER-GUIDE.md with installation walkthrough`
- `docs: add RESCUE-GUIDE.md for recovery procedures`
- `docs: add MIGRATION-1.0.md with breaking changes`
- `docs: update SECURITY_AUDIT.md with whitelist model`
- `docs: update IPC-API.md with capabilities endpoint`
- `docs: add 1.0 release notes to CHANGELOG.md`

---

## Definition of Done - Verification Checklist

Before merging `anna-1.0-reset` to `main`:

### Core Functionality
- [ ] `annad` detects all 6 states correctly (unit tests pass)
- [ ] `annactl status` returns valid state in all environments
- [ ] `annactl` shows only valid commands for current state (integration tests pass)

### Installation
- [ ] `annactl install` completes minimal base install in QEMU (UEFI + ext4)
- [ ] `annactl install` completes minimal base install in QEMU (BIOS + ext4)
- [ ] `annactl install` completes btrfs install in QEMU
- [ ] Installer generates `state.json`, `plans/installer.sh`, `reverts/installer.sh`, `postinstall.md`
- [ ] Installed system boots to login prompt
- [ ] `annactl install --dry-run` shows plan without executing

### Rescue
- [ ] `annactl rescue-detect` finds broken installation
- [ ] `annactl rescue-chroot` mounts and enters chroot successfully
- [ ] `annactl boot-repair` fixes broken bootloader (QEMU test passes)
- [ ] Repaired system boots successfully

### Maintenance
- [ ] `annactl update` checks Arch News before updating
- [ ] `annactl update` creates snapshot and rollback script
- [ ] `annactl update` executes pacman -Syu successfully
- [ ] Update rollback script downgrades packages correctly
- [ ] `annactl health` reports SMART, systemd, journal, security status

### Optimizer
- [ ] `annactl advise` generates Wiki-cited proposals
- [ ] All proposals include at least one `[archwiki:...]` citation
- [ ] `annactl apply <id>` executes whitelisted commands only
- [ ] `annactl revert <id>` undoes applied proposal

### Logging
- [ ] All actions logged to `/var/log/anna/events.jsonl`
- [ ] Every log entry includes `wiki_citations` field (except exempted types)
- [ ] `annactl history` queries events successfully
- [ ] `annactl history --citation archwiki:systemd-boot` filters correctly

### Security
- [ ] No arbitrary shell execution (`sh -c` blocked)
- [ ] All commands validated against whitelist
- [ ] Input validation rejects path traversal, injection attempts
- [ ] Temp files created with 0700 permissions

### Testing
- [ ] Unit tests: 100% pass rate
- [ ] Integration tests: 100% pass rate
- [ ] QEMU tests: minimal_uefi_ext4, minimal_bios_ext4, minimal_uefi_btrfs, rescue_broken_boot all pass
- [ ] Failure injection tests: all pass (rollback scripts generated)
- [ ] CI green: GitHub Actions test matrix passes

### Documentation
- [ ] `README.md` updated with 1.0 features
- [ ] `docs/STATE-MACHINE.md` created
- [ ] `docs/INSTALLER-GUIDE.md` created
- [ ] `docs/RESCUE-GUIDE.md` created
- [ ] `docs/MIGRATION-1.0.md` created with breaking changes
- [ ] `SECURITY_AUDIT.md` updated with whitelist model
- [ ] `docs/IPC-API.md` updated with capabilities endpoint
- [ ] `CHANGELOG.md` includes 1.0 release notes

### Code Quality
- [ ] No TUI code remains
- [ ] No desktop environment modules remain (ALL config modules deleted)
- [ ] No bundle system code remains (bundles.rs deleted entirely)
- [ ] No heuristic recommenders remain (smart_recommender, intelligent_recommender deleted)
- [ ] No WM detection code remains (system_detection.rs deleted)
- [ ] All Clippy warnings resolved
- [ ] No dead code (except feature-gated legacy)
- [ ] All functions have doc comments with Wiki citations where applicable

### Scope Verification
- [ ] Anna only generates system administration proposals (security, updates, maintenance)
- [ ] Anna only generates desktop administration proposals (hardware services, user management)
- [ ] Anna does NOT configure desktop environments
- [ ] Anna does NOT handle aesthetics or themes
- [ ] Anna does NOT customize user dotfiles (except XDG directory structure)

### Final Verification
- [ ] Manual test: Install on real hardware (UEFI + ext4)
- [ ] Manual test: Install server variant on real hardware
- [ ] Manual test: Rescue broken bootloader on real hardware
- [ ] Manual test: Update with rollback
- [ ] Manual test: Apply system/desktop admin proposals (microcode, bluetooth, CUPS)
- [ ] Manual test: All logs include citations
- [ ] Code review by maintainer
- [ ] PR approved and squash-merged to main

---

## Timeline

**Total Estimated Time:** 53 days (approximately 10-11 weeks)

**Phase Breakdown:**
- Phase 0: 3 days
- Phase 1: 5 days
- Phase 2: 4 days
- Phase 3: 10 days
- Phase 4: 7 days
- Phase 5: 6 days
- Phase 6: 5 days
- Phase 7: 4 days
- Phase 8: 5 days
- Phase 9: 4 days

**Target Release Date:** January 2026 (assuming start: mid-November 2025)

---

## Risk Mitigation

### Risk: QEMU Tests Fail in CI
**Mitigation:** Local QEMU testing before CI, fallback to manual verification

### Risk: Installer Breaks Existing Systems
**Mitigation:** Only runs in `iso_live` state, cannot affect running systems

### Risk: State Detection False Positives
**Mitigation:** Conservative detection (prefer `Unknown` state), extensive unit tests

### Risk: Rollback Scripts Incomplete
**Mitigation:** Automated testing of rollback scripts in QEMU, manual verification

### Risk: Wiki Citations Stale
**Mitigation:** Version-control citations, periodic audits, CI checks for broken links (future enhancement)

---

## Success Criteria

1. **Zero regressions** from RC.11 for existing users (via migration path)
2. **Installer works** on 3+ different hardware configurations (UEFI, BIOS, laptop, desktop)
3. **Rescue mode saves** at least one real broken system (user testimonial)
4. **All logs cite Wiki** (100% compliance in audit)
5. **CI green** for entire test matrix
6. **Documentation complete** (no TODOs, all sections filled)
7. **Scope verified**: No traces of desktop environment config, bundles, or heuristic recommenders remain
8. **Professional focus**: Anna is clearly a sysadmin tool, not a user customization tool
9. **Community feedback positive** (GitHub issues, Reddit r/archlinux)

---

## Post-1.0 Roadmap (Out of Scope)

Deferred to future releases:
- TUI revival (v1.1 "Eurydice")
- AUR helper integration (v1.3+)
- Configuration file support (v1.4+)
- Network installer (PXE boot) (v2.0+)

Permanently out of scope:
- Desktop environment installation and configuration
- Window manager bundles
- User theme/aesthetic management
- Dotfile customization (beyond XDG structure)

---

## Approval Required

This plan requires sign-off before proceeding to Phase 0.

**Review Checklist:**
- [ ] Architecture makes sense (state machine, whitelist model)
- [ ] Scope is appropriate for 1.0 (removals acceptable)
- [ ] Timeline is reasonable (10-11 weeks)
- [ ] Testing plan is sufficient (QEMU + unit + integration)
- [ ] Documentation plan covers all user needs
- [ ] Security model is sound (whitelist, no arbitrary shell)

**Approval Signature:**
- [ ] User: @jjgarcianorway
- [ ] Date: __________

Once approved, execution begins with Phase 0 commit.

---

**End of Implementation Plan**
