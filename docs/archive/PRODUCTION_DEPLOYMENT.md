# Anna Assistant - Production Deployment Guide

**Version**: 1.6.0-rc.1
**Phase**: 1.6 Mirror Audit and Temporal Self-Reflection
**Date**: 2025-11-12

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [First Start](#first-start)
4. [Verification](#verification)
5. [Monitoring](#monitoring)
6. [Troubleshooting](#troubleshooting)
7. [Security](#security)
8. [Backup and Recovery](#backup-and-recovery)

---

## Prerequisites

### System Requirements

- **OS**: Arch Linux (primary target) or compatible systemd-based distribution
- **Architecture**: x86_64 (amd64)
- **Rust**: 1.70+ (for building from source)
- **Runtime**: systemd, logrotate
- **Storage**: Minimum 500MB for binaries and state
- **Permissions**: Root access for initial setup

### Dependencies

```bash
# Arch Linux
sudo pacman -S systemd logrotate

# Debian/Ubuntu
sudo apt-get install systemd logrotate adduser

# RHEL/Fedora
sudo dnf install systemd logrotate
```

---

## Installation

### Option 1: From Pre-built Packages (Recommended)

**Debian/Ubuntu**:
```bash
# Download package
wget https://github.com/yourusername/anna-assistant/releases/download/v1.6.0-rc.1/anna-daemon_1.6.0-rc.1_amd64.deb

# Install
sudo dpkg -i anna-daemon_1.6.0-rc.1_amd64.deb
sudo apt-get install -f  # Fix dependencies if needed
```

**RHEL/Fedora**:
```bash
# Download package
wget https://github.com/yourusername/anna-assistant/releases/download/v1.6.0-rc.1/anna-daemon-1.6.0-0.1.rc1.x86_64.rpm

# Install
sudo dnf install anna-daemon-1.6.0-0.1.rc1.x86_64.rpm
```

### Option 2: From Source

```bash
# Clone repository
git clone https://github.com/yourusername/anna-assistant.git
cd anna-assistant
git checkout v1.6.0-rc.1

# Build binaries (release mode)
cargo build --release --bins

# Run system setup script
sudo bash scripts/setup-anna-system.sh

# Install binaries
sudo install -m 0755 target/release/annad /usr/bin/annad
sudo install -m 0755 target/release/annactl /usr/bin/annactl

# Install systemd service
sudo install -m 0644 systemd/anna-daemon.service /etc/systemd/system/anna-daemon.service

# Install logrotate config
sudo install -m 0644 logrotate/anna /etc/logrotate.d/anna

# Reload systemd
sudo systemctl daemon-reload
```

---

## First Start

### 1. Verify Installation

```bash
# Check binaries
annad --version
annactl --version

# Check user and group
getent passwd anna
getent group anna

# Check directories
ls -ld /var/lib/anna /var/log/anna
# Expected: drwxr-x--- anna anna
```

### 2. Review Configuration

Anna uses sensible defaults. Optional configuration can be placed in:
- `/etc/anna/mirror_audit.yml` - Mirror audit settings

Example `/etc/anna/mirror_audit.yml`:
```yaml
# Mirror Audit Configuration (Phase 1.6)
enable_bias_scan: true
min_sample_size: 5
min_confidence: 0.6
write_jsonl: true
audit_log_path: /var/log/anna/mirror-audit.jsonl
state_path: /var/lib/anna/mirror_audit/state.json
```

### 3. Enable and Start Service

**IMPORTANT**: The service is **NOT** enabled by default for safety.

```bash
# Enable service (start on boot)
sudo systemctl enable anna-daemon

# Start service
sudo systemctl start anna-daemon

# Check status
sudo systemctl status anna-daemon
```

Expected output:
```
● anna-daemon.service - Anna Assistant Daemon
     Loaded: loaded (/etc/systemd/system/anna-daemon.service; enabled)
     Active: active (running) since ...
```

---

## Verification

### 1. CLI Connectivity

```bash
# Check daemon is responding
annactl status

# Test mirror audit commands
annactl mirror audit-forecast
annactl mirror reflect-temporal
```

### 2. Check Logs

```bash
# Systemd journal
sudo journalctl -u anna-daemon -f

# Mirror audit log (JSONL)
sudo tail -f /var/log/anna/mirror-audit.jsonl
```

### 3. Verify State Persistence

```bash
# Check state file exists
sudo ls -lh /var/lib/anna/mirror_audit/state.json

# View state (pretty-printed)
sudo cat /var/lib/anna/mirror_audit/state.json | jq .
```

---

## Monitoring

### Expected Behavior

- **Audit log growth**: ~1KB per forecast audit
- **State file updates**: On every audit run
- **TIS values**: Range 0.0-1.0 (higher = better temporal integrity)
- **Bias flags**: Appear when confidence ≥ 0.6

### Key Metrics

**Average Temporal Integrity Score**:
```bash
annactl mirror audit-forecast --json | jq '.average_temporal_integrity'
```

**Recent Audit Entry**:
```bash
sudo tail -1 /var/log/anna/mirror-audit.jsonl | jq .
```

**Bias Detection Status**:
```bash
annactl mirror audit-forecast --json | jq '.active_biases'
```

**State Summary**:
```bash
sudo cat /var/lib/anna/mirror_audit/state.json | jq '{
  total_audits,
  last_audit_at,
  recent_integrity_scores: .recent_integrity_scores[-5:]
}'
```

### Real-time Monitoring

**Live audit trail**:
```bash
sudo tail -f /var/log/anna/mirror-audit.jsonl | jq .
```

**Watch TIS trends**:
```bash
watch -n 60 "annactl mirror audit-forecast --json | jq '.average_temporal_integrity'"
```

### Alerting Recommendations

Set up alerts for:
- TIS dropping below 0.6 (indicates systematic bias)
- Audit log not growing (forecasting may be stalled)
- Multiple high-confidence bias flags (>0.7)
- Daemon restart failures

---

## Troubleshooting

### Daemon Won't Start

**Symptom**: `systemctl status anna-daemon` shows failed state

**Checks**:
```bash
# Check journal for errors
sudo journalctl -u anna-daemon -n 50

# Verify permissions
sudo ls -ld /var/lib/anna /var/log/anna
# Must be anna:anna with 750 permissions

# Verify user exists
getent passwd anna

# Test manual start
sudo -u anna /usr/bin/annad
```

**Common issues**:
- Permission denied on `/var/lib/anna` or `/var/log/anna`
- Conflicting RPC socket at `/run/anna/annad.sock`
- Missing dependencies

### CLI Shows "Daemon Unavailable"

**Symptom**: `annactl` commands exit with code 70

**Checks**:
```bash
# Check daemon is running
systemctl is-active anna-daemon

# Check socket exists
ls -l /run/anna/annad.sock

# Test socket connection
sudo -u anna nc -U /run/anna/annad.sock
```

### Audit Log Not Growing

**Symptom**: No new entries in `mirror-audit.jsonl`

**Checks**:
```bash
# Verify forecasting is active
annactl chronos status

# Check mirror audit state
sudo cat /var/lib/anna/mirror_audit/state.json | jq .

# Verify write permissions
sudo -u anna touch /var/log/anna/test.log
```

---

## Security

### Advisory Mode Enforcement

**CRITICAL**: Anna operates in **advisory-only mode** by design. This cannot be disabled via configuration.

Enforcement layers:
1. **Code-level**: No auto-apply paths in RPC handlers (`rpc_server.rs`)
2. **CLI-level**: Warnings on all adjustment outputs (`mirror_commands.rs`)
3. **Documentation**: Security model in `CHANGELOG.md`

### AppArmor (Debian/Ubuntu)

```bash
# Install profile
sudo cp security/apparmor.anna.profile /etc/apparmor.d/usr.bin.annad

# Load profile
sudo apparmor_parser -r /etc/apparmor.d/usr.bin.annad

# Verify
sudo aa-status | grep annad
```

### SELinux (RHEL/Fedora)

```bash
# Build policy module
cd security
checkmodule -M -m -o anna.mod selinux.anna.te
semodule_package -o anna.pp -m anna.mod

# Install policy
sudo semodule -i anna.pp

# Verify
semodule -l | grep anna

# Label files
sudo restorecon -Rv /var/lib/anna /var/log/anna /usr/bin/annad
```

### File Permissions

All sensitive files should be owned by `anna:anna`:
```bash
sudo chown -R anna:anna /var/lib/anna /var/log/anna
sudo chmod 750 /var/lib/anna /var/log/anna
sudo chmod 640 /var/lib/anna/mirror_audit/state.json
```

### Network Security

Phase 1.6 uses only local Unix sockets. Future phases (1.7+) will add:
- Ed25519 cryptographic signing
- Peer authentication
- TLS for inter-node communication

---

## Backup and Recovery

### What to Back Up

**State directory** (critical):
```bash
sudo tar czf anna-state-$(date +%Y%m%d).tar.gz /var/lib/anna
```

**Audit logs** (important):
```bash
sudo tar czf anna-logs-$(date +%Y%m%d).tar.gz /var/log/anna
```

**Configuration** (if customized):
```bash
sudo tar czf anna-config-$(date +%Y%m%d).tar.gz /etc/anna
```

### Backup Schedule

Recommended:
- **State**: Daily incremental, weekly full
- **Logs**: Weekly (or after log rotation)
- **Config**: On change

### Recovery

**Restore state**:
```bash
sudo systemctl stop anna-daemon
sudo tar xzf anna-state-20251112.tar.gz -C /
sudo chown -R anna:anna /var/lib/anna
sudo systemctl start anna-daemon
```

**Verify integrity**:
```bash
annactl mirror audit-forecast --json | jq .
```

---

## Maintenance

### Log Rotation

Automated by logrotate (configured in `/etc/logrotate.d/anna`):
- Trigger: 10MB file size
- Retention: 14 rotations
- Compression: gzip
- Action: copytruncate (daemon-friendly)

Manual rotation:
```bash
sudo logrotate -f /etc/logrotate.d/anna
```

### Updates

```bash
# Stop daemon
sudo systemctl stop anna-daemon

# Install new version (package or binary)
# ... (same as installation steps)

# Start daemon
sudo systemctl start anna-daemon

# Verify
annactl --version
annactl status
```

### State Cleanup

**WARNING**: Do not delete state files while daemon is running.

To reset state (for testing):
```bash
sudo systemctl stop anna-daemon
sudo rm -rf /var/lib/anna/mirror_audit/state.json
sudo systemctl start anna-daemon
```

---

## References

- [Arch Wiki: System Maintenance](https://wiki.archlinux.org/title/System_maintenance)
- [Arch Wiki: systemd](https://wiki.archlinux.org/title/Systemd)
- [Arch Wiki: AppArmor](https://wiki.archlinux.org/title/AppArmor)
- [Arch Wiki: SELinux](https://wiki.archlinux.org/title/SELinux)
- Phase 1.6 validation report: `/tmp/FINAL_ACCEPTANCE_REPORT.md`

---

**Citation**: [archwiki:System_maintenance]
