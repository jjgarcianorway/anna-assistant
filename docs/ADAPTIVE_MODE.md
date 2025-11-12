# Adaptive Intelligence & Smart Profiling

**Phase 3.0.0-alpha.1** - System self-awareness for resource-optimized operation

## Overview

Anna now possesses **adaptive intelligence**: the ability to detect her runtime environment and automatically adjust features, monitoring depth, and resource usage to match system capacity. This ensures Anna runs optimally on everything from resource-constrained VMs to high-performance workstations.

## How It Works

### System Profiling

On every request, Anna collects a comprehensive system profile:

- **Resources**: RAM, CPU cores, disk space, uptime
- **Environment**: Virtualization type (bare metal, VM, container)
- **Session**: Desktop GUI, headless, SSH (with display forwarding detection)
- **Hardware**: GPU presence and vendor (NVIDIA, AMD, Intel)

**Implementation**: `crates/annad/src/profile/detector.rs`

### Decision Engine

Based on the profile, Anna calculates a **monitoring mode** using these rules:

| Condition | Mode | Features |
|-----------|------|----------|
| RAM < 2GB | **Minimal** | Internal stats only (no Prometheus/Grafana) |
| 2-4GB RAM | **Light** | Prometheus metrics (no Grafana UI) |
| >4GB + GUI | **Full** | Prometheus + Grafana dashboards |
| >4GB + Headless/SSH | **Light** | Prometheus only (no GUI available) |

**Implementation**: `crates/annad/src/profile/types.rs::SystemProfile::calculate_monitoring_mode`

### Adaptive Behaviors

#### 1. Monitoring Stack Selection (`annactl monitor install`)

```bash
# Auto-selects appropriate monitoring based on system
$ annactl monitor install

# Override auto-detection if needed
$ annactl monitor install --force-mode full

# Preview without installing
$ annactl monitor install --dry-run
```

**Logic**:
- Minimal: Informs user no external monitoring is installed
- Light: Installs Prometheus only
- Full: Installs Prometheus + Grafana with dashboard setup

#### 2. SSH Tunnel Suggestions (`annactl profile`)

When Anna detects an SSH session, she provides tunnel instructions:

```bash
$ annactl profile
...
Remote Access:
  SSH session detected (no X11 forwarding)

  💡 To access Grafana dashboards, create an SSH tunnel:
     ssh -L 3000:localhost:3000 user@host
     Then browse to: http://localhost:3000
```

**Logic**:
- Checks `$SSH_CONNECTION` environment variable
- Detects X11 forwarding via `$DISPLAY`
- Shows relevant tunnel commands based on monitoring mode

#### 3. Capability Filtering (Future)

The `/capabilities` endpoint now includes monitoring mode:

```json
{
  "commands": [...],
  "monitoring_mode": "light",
  "monitoring_rationale": "3072 MB RAM detected → light mode (Prometheus only)",
  "is_constrained": false
}
```

**Future Use**: CLI can hide monitoring commands in minimal mode, show warnings when resources are constrained.

## Commands

### `annactl profile`

Display complete system profile with adaptive intelligence recommendations.

**Flags**:
- `--json`: Output JSON for scripting

**Example**:
```bash
$ annactl profile

System Profile (Phase 3.0: Adaptive Intelligence)
==================================================

Resources:
  Memory:  8192 MB total, 4096 MB available
  CPU:     4 cores
  Disk:    256 GB total, 128 GB available
  Uptime:  86400 seconds (24.0 hours)

Environment:
  Virtualization: none
  Session Type:   desktop:wayland
  GPU:            Intel (Intel Iris Xe Graphics)

Adaptive Intelligence:
  Monitoring Mode: FULL
  Rationale:       8192 MB RAM + wayland session detected → full mode (Grafana + Prometheus)
  Constrained:     No

Timestamp: 2025-01-12T10:30:00Z
```

### `annactl monitor install`

Install monitoring stack with adaptive selection.

**Flags**:
- `--force-mode <MODE>`: Override auto-detection (full/light/minimal)
- `--dry-run`: Preview installation without executing

**Example**:
```bash
$ annactl monitor install --dry-run

Monitoring Stack Installation (Phase 3.0: Adaptive Intelligence)
====================================================================

System Profile:
  Memory: 3072 MB  |  CPU: 2 cores  |  Constrained: No
  Recommended Mode: LIGHT

Installation Plan [LIGHT MODE]:
  ✓ Prometheus (metrics collector)
  ✗ Grafana (skipped - insufficient RAM or no GUI)

Requirements: 2-4GB RAM

[DRY RUN] Would install Prometheus only
```

### `annactl monitor status`

Check monitoring stack installation and service status.

**Example**:
```bash
$ annactl monitor status

Monitoring Stack Status (Phase 3.0)
====================================

System Mode: FULL
Rationale:   8192 MB RAM + wayland session detected → full mode (Grafana + Prometheus)

Prometheus:
  Status: ✓ Running
  Access: http://localhost:9090

Grafana:
  Status: ✓ Running
  Access: http://localhost:3000

Internal Stats: ✓ Available (via daemon)
  Commands: annactl status, annactl health
```

## Override Mechanisms

### Force Monitoring Mode

Users can override Anna's recommendations:

```bash
# Force full mode on a low-RAM system (not recommended)
$ annactl monitor install --force-mode full
⚠️  Using FORCED mode: FULL
   (System recommendation: MINIMAL)
...
```

**Use Case**: Testing, development, or when user knows system load will be low.

### Future: Config File Override

Planned: `config.toml` setting to persist mode override:

```toml
[adaptive]
# Override: full, light, minimal, auto (default)
monitoring_mode = "light"
```

## RPC Protocol

### `GetProfile` Method

Returns complete system profile:

```json
{
  "total_memory_mb": 8192,
  "available_memory_mb": 4096,
  "cpu_cores": 4,
  "total_disk_gb": 256,
  "available_disk_gb": 128,
  "uptime_seconds": 86400,
  "virtualization": "none",
  "session_type": "desktop:wayland",
  "gpu_present": true,
  "gpu_vendor": "Intel",
  "gpu_model": "Intel Iris Xe Graphics",
  "recommended_monitoring_mode": "full",
  "monitoring_rationale": "8192 MB RAM + wayland session detected → full mode",
  "is_constrained": false,
  "timestamp": "2025-01-12T10:30:00Z"
}
```

### Extended `GetCapabilities`

Now includes adaptive intelligence:

```json
{
  "commands": [
    {"name": "update", "description": "...", ...},
    ...
  ],
  "monitoring_mode": "full",
  "monitoring_rationale": "8192 MB RAM + wayland session detected → full mode",
  "is_constrained": false
}
```

## Detection Methods

### Virtualization

**Tool**: `systemd-detect-virt`

**Detected Types**:
- Bare metal: `none`
- VMs: `kvm`, `qemu`, `vmware`, `virtualbox`, `xen`
- Containers: `docker`, `podman`, `lxc`, `systemd-nspawn`

**Fallback**: `unknown` if tool unavailable

**Citation**: [systemd:detect-virt](https://www.freedesktop.org/software/systemd/man/systemd-detect-virt.html)

### Session Type

**Methods**:
1. Check `$SSH_CONNECTION` → SSH session
2. Check `$XDG_SESSION_TYPE` → Desktop type (wayland/x11/tty)
3. Check `$DISPLAY` → X11 session
4. Check `$WAYLAND_DISPLAY` → Wayland session
5. Run `tty` → Console (TTY) vs pseudo-terminal (PTS)
6. Default → Headless

**Citation**: [xdg:basedir](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)

### GPU Detection

**Tool**: `lspci`

**Method**:
1. Parse `lspci` output for "VGA compatible controller"
2. Extract vendor (NVIDIA, AMD, Intel) from device string
3. Store model name

**Fallback**: `present: false` if no GPU detected

**Citation**: [pciutils:lspci](https://mj.ucw.cz/sw/pciutils/)

### System Resources

**Library**: `sysinfo` crate

**Collected**:
- Total/available memory (bytes → MB)
- CPU core count
- Disk space (root filesystem)
- System uptime (via `/proc/uptime`)

**Citation**: [linux:proc](https://www.kernel.org/doc/html/latest/filesystems/proc.html)

## Resource Constraint Detection

Anna considers a system **constrained** if:

```rust
pub fn is_constrained(&self) -> bool {
    self.total_memory_mb < 4096 ||
    self.cpu_cores < 2 ||
    self.available_disk_gb < 10
}
```

**Use Cases**:
- Warning users before resource-heavy operations
- Automatic throttling of background tasks
- Disabling non-essential features

## Future Enhancements

### Phase 3.1: Dynamic Adaptation
- Runtime mode switching based on memory pressure
- Automatic disabling of Grafana if RAM drops below threshold
- Graceful degradation under high load

### Phase 3.2: Machine Learning
- Learn user's resource usage patterns
- Predict optimal monitoring mode based on time of day
- Adaptive sampling rates for metrics

### Phase 3.3: Multi-Node Awareness
- Detect cluster configuration
- Centralized monitoring for distributed Anna instances
- Resource pooling across nodes

## Testing

### Unit Tests

Located in `crates/annad/src/profile/`:

```bash
# Run all profile tests
$ cargo test --package annad --bin annad profile

running 11 tests
test profile::types::tests::test_monitoring_mode_minimal ... ok
test profile::types::tests::test_monitoring_mode_light ... ok
test profile::types::tests::test_monitoring_mode_full_desktop ... ok
test profile::detector::tests::test_collect_profile ... ok
...
test result: ok. 11 passed; 0 failed
```

### Integration Tests

Test adaptive behavior end-to-end:

```bash
# Simulate low-memory system
$ ANNA_TEST_RAM=1024 cargo test test_minimal_mode

# Simulate SSH session
$ SSH_CONNECTION="1.2.3.4 1234 5.6.7.8 22" cargo test test_ssh_detection
```

## Observability

Profile collection is lightweight:
- **Latency**: <10ms per profile collection
- **Memory**: <1MB overhead for sysinfo cache
- **CPU**: Negligible (command execution only)

Profile metrics exposed via Prometheus (future):
```
anna_profile_mode{mode="full"} 1
anna_system_memory_used_percent 45.2
anna_system_uptime_seconds 86400
```

## Configuration

### Monitoring Mode Thresholds

Defined in `crates/annad/src/profile/types.rs`:

```rust
pub fn calculate_monitoring_mode(
    total_memory_mb: u64,
    session_type: &SessionType,
) -> MonitoringMode {
    // Rule 1: RAM < 2 GB → minimal
    if total_memory_mb < 2048 {
        return MonitoringMode::Minimal;
    }

    // Rule 2: 2-4 GB → light
    if total_memory_mb < 4096 {
        return MonitoringMode::Light;
    }

    // Rule 3: > 4 GB + GUI → full
    if matches!(session_type, SessionType::Desktop(_)) {
        return MonitoringMode::Full;
    }

    // Default: Light for headless/SSH with sufficient RAM
    MonitoringMode::Light
}
```

**Customization**: Edit thresholds and rebuild, or use `--force-mode` flag.

## Troubleshooting

### Profile Detection Issues

**Problem**: "Unable to detect system profile"

**Causes**:
- `systemd-detect-virt` not installed
- `/proc/uptime` unavailable (non-Linux)
- `lspci` missing (minimal installs)

**Solution**: Install missing tools or ignore detection failures (daemon uses defaults).

### Forced Mode Not Working

**Problem**: `--force-mode` ignored

**Cause**: Typo in mode name

**Solution**: Use exact names: `full`, `light`, `minimal` (case-insensitive)

### SSH Tunnel Not Showing

**Problem**: No tunnel suggestions despite SSH session

**Cause**: `$SSH_CONNECTION` not set (screen/tmux, or manual SSH env cleanup)

**Solution**: Manually set: `export SSH_CONNECTION="1.2.3.4 1234 5.6.7.8 22"`

## Citations & References

- [Arch Wiki: System Maintenance](https://wiki.archlinux.org/title/System_maintenance)
- [Arch Wiki: Prometheus](https://wiki.archlinux.org/title/Prometheus)
- [Arch Wiki: Grafana](https://wiki.archlinux.org/title/Grafana)
- [systemd: detect-virt](https://www.freedesktop.org/software/systemd/man/systemd-detect-virt.html)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [Linux /proc filesystem](https://www.kernel.org/doc/html/latest/filesystems/proc.html)
- [Observability Best Practices](https://sre.google/sre-book/monitoring-distributed-systems/)

---

**Implemented**: Phase 3.0.0-alpha.1
**Next**: Adaptive UI hints, profile metrics export
**Author**: Anna (with human guidance)
**License**: Custom (see LICENSE file)
