# Anna v0.10.1 - Quick Start Guide

## 60-Second Installation

```bash
# Build
cargo build --release

# Test
./tests/smoke_v101.sh

# Install
sudo ./scripts/install_v101.sh

# Verify
annactl status
```

## Essential Commands

```bash
# System Status
annactl status          # Daemon health
annactl capabilities    # Module status (Active/Degraded/Disabled)
annactl alerts          # System integrity alerts

# Telemetry
annactl sensors         # CPU, RAM, temp, battery
annactl net             # Network interfaces
annactl disk            # Disk usage + SMART
annactl radar           # Persona analysis

# Management
annactl module disable gpu --reason "No GPU"
annactl module enable gpu
annactl fix <issue-id>
```

## Troubleshooting

### Daemon won't start

```bash
# Check logs
journalctl -u annad -n 50

# Verify installation
annactl doctor post --verbose

# Check capabilities
annactl capabilities
```

### Module shows DEGRADED

```bash
# See what's missing
annactl capabilities

# Install missing packages (example for sensors)
sudo pacman -S lm_sensors
sudo sensors-detect --auto

# Restart daemon
sudo systemctl restart annad
```

### Fix an integrity issue

```bash
# List issues
annactl alerts

# Fix specific issue
annactl fix degraded_optional_sensors

# Or auto-accept
annactl fix degraded_optional_sensors --yes
```

## Uninstall

```bash
# Keep data
sudo ./scripts/uninstall_v101.sh

# Complete removal
sudo ./scripts/uninstall_v101.sh --purge
```

## Configuration

**Enable/Disable Modules:** `/etc/anna/modules.yaml`

```yaml
modules:
  sensors:
    state: disabled  # or enabled
    reason: "Using custom monitoring"
```

Then restart: `sudo systemctl restart annad`

## File Locations

```
Binaries:      /usr/local/bin/annad, annactl
Config:        /etc/anna/modules.yaml
Registry:      /usr/lib/anna/CAPABILITIES.toml
Data:          /var/lib/anna/telemetry.db
Logs:          /var/log/anna/annad.log
Socket:        /run/anna/annad.sock
```

## Exit Codes

```
0   Success
10  Preflight failed
11  Autofix/postflight failed
12  Postflight degraded (acceptable)
20  Permissions error
21  Disk space insufficient
30  Build failed
```

## Next Steps

1. Install optional dependencies for full capabilities:
   ```bash
   sudo pacman -S lm_sensors smartmontools ethtool
   ```

2. Configure module preferences in `/etc/anna/modules.yaml`

3. Monitor logs: `journalctl -u annad -f`

4. Export telemetry: `annactl export -o snapshot.json`

---

**Docs:** `V0.10.1_IMPLEMENTATION_SUMMARY.md`
**Tests:** `./tests/smoke_v101.sh`
**Version:** v0.10.1
