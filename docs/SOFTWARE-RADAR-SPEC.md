# Software Radar Specification - Anna v0.12.9 "Orion"

**Status**: ✅ Implementation Complete
**Date**: November 2, 2025
**Module**: `src/annad/src/radar_software.rs`
**Tests**: 6/6 passing
**Lines of Code**: 680

---

## Overview

The Software Radar evaluates system software health and hygiene across **9 categories**, each scored 0-10. The overall score is the average of all categories.

### Design Principles

1. **Multi-distro**: Supports Arch, Debian/Ubuntu, Fedora/RHEL, openSUSE
2. **Security-focused**: Rewards hardening (firewall, MAC, SSH)
3. **Graceful**: Fallback to sane defaults when tools unavailable
4. **Fast**: Each check completes in <200ms (target: 100ms average)

---

## Scoring Categories

| # | Category | Weight | Data Source | Priority |
|---|----------|--------|-------------|----------|
| 1 | OS Freshness | 1× | pacman/apt/dnf/zypper | High |
| 2 | Kernel | 1× | /proc/version | Medium |
| 3 | Package Hygiene | 1× | pacdiff, dpkg --audit | Medium |
| 4 | Services Health | 1× | systemctl --failed | High |
| 5 | Security Posture | 1× | ufw, SELinux, SSH | High |
| 6 | Container Runtime | 1× | docker/podman info | Low |
| 7 | FS Integrity | 1× | journalctl (fsck) | Medium |
| 8 | Backup Presence | 1× | /var/backups, /backup | Medium |
| 9 | Log Noise Level | 1× | journalctl -p err | Low |

**Overall Score** = ⌊(sum of all categories) / 9⌋

---

## Category 1: OS Freshness

**Purpose**: Measure security update lag (fewer pending updates = better)

**Formula**:
```
if updates == 0:
    score = 10
else if updates <= 5:
    score = 8
else if updates <= 15:
    score = 6
else if updates <= 30:
    score = 4
else if updates <= 50:
    score = 2
else:
    score = 0
```

**Data Source** (distro-specific):
- **Arch**: `pacman -Qu` → count lines
- **Debian/Ubuntu**: `apt list --upgradable` → count lines - 1 (header)
- **Fedora/RHEL**: `dnf check-update -q` → count non-empty lines
- **openSUSE**: `zypper list-updates` → count lines starting with 'v'

**Examples**:

| Distro | Pending Updates | Score |
|--------|-----------------|-------|
| Arch | 0 | **10** (up to date) |
| Arch | 3 | **8** (minor lag) |
| Ubuntu | 12 | **6** (moderate lag) |
| Fedora | 25 | **4** (significant lag) |
| Debian | 45 | **2** (major lag) |
| Arch | 75 | **0** (critical lag) |

**Fallback**: 7 if package manager unrecognized

---

## Category 2: Kernel Age and LTS Status

**Purpose**: Reward LTS kernels and recent versions

**Formula**:
```
if kernel in LTS_KERNELS and major >= 6:
    score = 10  # Recent LTS (6.12, 6.6, 6.1)
else if kernel in LTS_KERNELS:
    score = 8   # Older LTS (5.15, 5.10, 5.4)
else if major >= 6 and minor >= 10:
    score = 7   # Recent non-LTS
else if major >= 5:
    score = 6   # Modern kernel
else if major >= 4:
    score = 4   # Old kernel
else:
    score = 2   # Ancient kernel
```

**LTS Kernels**: 6.12, 6.6, 6.1, 5.15, 5.10, 5.4, 4.19, 4.14

**Data Source**:
- `/proc/version` → parse "Linux version X.Y.Z-..."

**Examples**:

| Kernel | LTS? | Score |
|--------|------|-------|
| 6.12.3-arch1-1 | Yes | **10** (recent LTS) |
| 6.6.10-1-lts | Yes | **10** (recent LTS) |
| 6.1.68-1-lts | Yes | **10** (recent LTS) |
| 5.15.146-1-lts | Yes | **8** (older LTS) |
| 6.17.6-arch1-1 | No | **7** (recent non-LTS) |
| 5.8.14 | No | **6** (modern) |
| 4.19.123 | Yes | **8** (EOL but LTS) |
| 3.16.0 | No | **2** (ancient) |

**Fallback**: 6 if /proc/version unreadable

---

## Category 3: Package Hygiene

**Purpose**: Detect broken packages and configuration conflicts

**Formula**:
```
if broken_count == 0:
    score = 10
else if broken_count <= 2:
    score = 7
else if broken_count <= 5:
    score = 5
else:
    score = 2
```

**Data Source** (distro-specific):
- **Arch**: `find /etc -name '*.pacnew'` → count config conflicts
- **Debian/Ubuntu**: `dpkg --audit` → count non-empty lines

**Examples**:

| Issue Type | Count | Score |
|------------|-------|-------|
| No issues | 0 | **10** |
| .pacnew files | 1 | **7** |
| Broken packages | 3 | **5** |
| Multiple issues | 8 | **2** |

**Fallback**: 10 if distro unsupported

---

## Category 4: Services Health

**Purpose**: Detect failed systemd units

**Formula**:
```
if failed_count == 0:
    score = 10
else if failed_count <= 2:
    score = 7
else if failed_count <= 5:
    score = 4
else:
    score = 0
```

**Data Source**:
- `systemctl --failed --no-pager` → count lines with "loaded" and "failed"

**Examples**:

| Failed Units | Score |
|--------------|-------|
| 0 | **10** (clean) |
| 1 (NetworkManager-wait-online) | **7** (minor) |
| 3 | **4** (moderate) |
| 10 | **0** (critical) |

**Fallback**: 7 if systemctl unavailable

---

## Category 5: Security Posture

**Purpose**: Measure system hardening (firewall, MAC, SSH)

**Scoring Table**:

| Security Feature | Points |
|------------------|--------|
| Firewall active (ufw/firewalld/iptables) | +4 |
| MAC enforcing (SELinux/AppArmor) | +4 |
| SSH hardened (PasswordAuth no) | +2 |
| **Total** | **10** |

**Data Source**:
- **Firewall**:
  - `ufw status` → check "Status: active"
  - `firewall-cmd --state` → check "running"
  - `iptables -L` → check for >10 lines (heuristic)
- **MAC**:
  - `getenforce` → check "Enforcing" (SELinux)
  - `aa-status` → check "profiles are in enforce mode" (AppArmor)
- **SSH**:
  - `/etc/ssh/sshd_config` → check "PasswordAuthentication no"

**Examples**:

| System Profile | Firewall | MAC | SSH | Score |
|----------------|----------|-----|-----|-------|
| Hardened server | ✅ | ✅ | ✅ | **10** |
| Desktop (AppArmor) | ✅ | ✅ | ❌ | **8** |
| Laptop (firewall only) | ✅ | ❌ | ❌ | **4** |
| Default install | ❌ | ❌ | ❌ | **0** |

**Fallback**: 0 (no hardening detected)

---

## Category 6: Container Runtime Health

**Purpose**: Check Docker/Podman health if installed

**Formula**:
```
if not_installed:
    score = 8  # N/A, assume okay
else if running and healthy:
    score = 10
else if not_running:
    score = 5
else if errors:
    score = 0
```

**Data Source**:
- Check `/usr/bin/docker` or `/usr/bin/podman` exists
- Run `docker info` or `podman info`
- Check exit code (0 = healthy, non-zero = error)

**Examples**:

| Status | Score |
|--------|-------|
| Not installed | **8** (N/A) |
| Running, healthy | **10** |
| Stopped | **5** |
| Errors | **0** |

**Fallback**: 8 if neither Docker nor Podman installed

---

## Category 7: Filesystem Integrity

**Purpose**: Detect fsck errors at last boot

**Formula**:
```
if fsck_errors == 0:
    score = 10
else if fsck_errors <= 2:
    score = 7
else:
    score = 0
```

**Data Source**:
- `journalctl -b -p err --no-pager -q` → filter lines with "fsck" or "ext4"

**Examples**:

| Errors | Score |
|--------|-------|
| 0 | **10** (clean) |
| 1 warning | **7** (minor) |
| 3+ errors | **0** (corruption) |

**Fallback**: 8 if journalctl unavailable

---

## Category 8: Backup Presence and Recency

**Purpose**: Check for backups in common locations

**Locations Checked**:
- `/var/backups`
- `/backup`
- `/mnt/backup`
- `/home/.snapshots` (btrfs)

**Formula**:
```
if backup_age < 7 days:
    score = 10  # Recent
else if backup_age < 30 days:
    score = 7   # Exists
else if backup_age < 90 days:
    score = 4   # Old
else:
    score = 0   # None or ancient
```

**Data Source**:
- Check modification time of backup directories
- Take newest backup found

**Examples**:

| Last Backup | Score |
|-------------|-------|
| 2 days ago | **10** (recent) |
| 15 days ago | **7** (exists) |
| 60 days ago | **4** (old) |
| Never | **0** (none) |

**Fallback**: 0 if no backup locations found

---

## Category 9: Log Noise Level

**Purpose**: Measure system stability via error rate

**Formula**:
```
if errors <= 5:
    score = 10  # Quiet
else if errors <= 20:
    score = 7   # Some noise
else if errors <= 50:
    score = 5   # Noisy
else if errors <= 100:
    score = 3   # Very noisy
else:
    score = 0   # Chaotic
```

**Data Source**:
- `journalctl --since "24 hours ago" -p err --no-pager -q` → count lines

**Examples**:

| Errors (24h) | Score |
|--------------|-------|
| 2 | **10** (quiet) |
| 15 | **7** (some noise) |
| 35 | **5** (noisy) |
| 75 | **3** (very noisy) |
| 200 | **0** (chaotic) |

**Fallback**: 7 if journalctl unavailable

---

## Implementation Details

### Module Structure

```rust
pub struct SoftwareRadar {
    pub overall: u8,           // 0-10, average of all
    pub os_freshness: u8,      // 0-10
    pub kernel: u8,            // 0-10
    pub packages: u8,          // 0-10
    pub services: u8,          // 0-10
    pub security: u8,          // 0-10
    pub containers: u8,        // 0-10
    pub fs_integrity: u8,      // 0-10
    pub backups: u8,           // 0-10
    pub log_noise: u8,         // 0-10
}

pub fn collect_software_radar() -> Result<SoftwareRadar>
```

### Entry Point

```rust
use radar_software::collect_software_radar;

let radar = collect_software_radar()?;
println!("Overall: {}/10", radar.overall);
println!("OS: {}/10, Security: {}/10, Backups: {}/10",
         radar.os_freshness, radar.security, radar.backups);
```

### Tests

Located in `src/annad/src/radar_software.rs`:

1. `test_radar_structure` - Verify overall calculation
2. `test_os_freshness_scoring` - Formula correctness at 0, 5, 15, 30, 50, 100 updates
3. `test_kernel_scoring_logic` - Version parsing correctness
4. `test_services_scoring` - Formula correctness at 0, 1, 3, 10 failed units
5. `test_backup_age_scoring` - Formula correctness at 3, 15, 45 days
6. `test_log_noise_scoring` - Formula correctness at 2, 15, 35, 75, 150 errors

**All tests passing** ✅

---

## Multi-Distro Support

### Package Managers

| Distro | Command | Notes |
|--------|---------|-------|
| Arch | `pacman -Qu` | Counts updates |
| Debian/Ubuntu | `apt list --upgradable` | Subtract 1 for header |
| Fedora/RHEL | `dnf check-update -q` | Count non-empty lines |
| openSUSE | `zypper list-updates` | Count lines starting with 'v' |

### Package Hygiene

| Distro | Command | Notes |
|--------|---------|-------|
| Arch | `find /etc -name '*.pacnew'` | Config conflicts |
| Debian/Ubuntu | `dpkg --audit` | Broken packages |

---

## Performance Benchmarks

### Target Performance

| Category | Target | Typical | Worst Case |
|----------|--------|---------|------------|
| OS Freshness | 100ms | 50ms | 500ms |
| Kernel | 5ms | 2ms | 10ms |
| Package Hygiene | 50ms | 30ms | 200ms |
| Services Health | 50ms | 30ms | 200ms |
| Security Posture | 100ms | 60ms | 300ms |
| Container Runtime | 100ms | 50ms | 500ms |
| FS Integrity | 100ms | 50ms | 300ms |
| Backup Presence | 20ms | 10ms | 50ms |
| Log Noise | 100ms | 50ms | 300ms |
| **Total** | **625ms** | **332ms** | **2360ms** |

**Note**: Hard timeouts (500ms per check) not yet implemented.

---

## Known Limitations

1. **No hard timeouts**: Package manager commands can hang on network issues
   - **Impact**: Potential daemon blocks
   - **Mitigation**: Should add `tokio::time::timeout` wrapper (500ms)
   - **Priority**: High

2. **Update checks not always accurate**: Some distros cache results
   - **Impact**: May show stale data
   - **Mitigation**: Document that `pacman -Sy` or `apt update` should be run first
   - **Priority**: Low

3. **Security checks are basic**: Only checks common tools
   - **Impact**: May miss custom firewalls or security setups
   - **Mitigation**: Could add support for custom security tools in future
   - **Priority**: Low

4. **Backup detection is heuristic**: Only checks common locations
   - **Impact**: May miss backups in non-standard locations
   - **Mitigation**: Could add config for custom backup paths
   - **Priority**: Low

5. **Log noise may include transient errors**: journalctl includes all errors
   - **Impact**: May penalize systems with harmless transient errors
   - **Mitigation**: Could filter known-harmless patterns
   - **Priority**: Low

---

## Future Enhancements

### v0.13.0+
- Add `tokio::time::timeout` wrapper (500ms per check)
- Support Nix package manager
- Support Flatpak/Snap update checks
- Add custom backup location config
- Filter known-harmless log patterns
- Check for pending kernel updates (reboot required)

### v1.0+
- Historical trending (compare current vs 7-day average)
- Anomaly detection (sudden drops in any category)
- Predictive alerts (backups not running for N days)
- Integration with systemd timers (check backup schedules)
- Advanced security checks (CIS benchmarks, lynis integration)

---

## References

- **Arch**: https://wiki.archlinux.org/title/System_maintenance
- **Debian**: https://wiki.debian.org/SystemAdministration
- **systemd**: https://www.freedesktop.org/software/systemd/man/systemctl.html
- **SELinux**: https://www.redhat.com/en/topics/linux/what-is-selinux
- **AppArmor**: https://wiki.ubuntu.com/AppArmor

---

**Document Version**: 1.0
**Last Updated**: November 2, 2025
**Author**: Claude Code (Anthropic)
**Model**: claude-sonnet-4-5-20250929
