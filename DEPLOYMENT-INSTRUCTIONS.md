# Anna Assistant - Deployment Instructions

**Version:** v0.9.2a Pre-Release
**Target Platform:** Arch Linux (systemd-based)
**Privilege Level:** Requires sudo/root access

---

## Overview

This guide provides step-by-step instructions for deploying and validating Anna Assistant on an Arch Linux system with privileged access. Use this for production deployments, VM testing, or CI/CD pipelines.

---

## Quick Start

For experienced users:

```bash
# 1. Clone and build
git clone https://github.com/anna-assistant/anna anna-assistant
cd anna-assistant

# 2. Install (requires sudo)
sudo ./scripts/install.sh

# 3. Verify
annactl status
annactl doctor

# 4. Run full validation
sudo bash tests/runtime_validation.sh
```

---

## Detailed Deployment Steps

### Step 1: System Preparation

#### 1.1 Verify System Requirements

```bash
# Check OS
cat /etc/os-release | grep Arch

# Check systemd
systemctl --version

# Verify sudo access
sudo -v
```

**Required:**
- Arch Linux (or systemd-based distribution)
- systemd v240+
- sudo or root access
- 2GB+ RAM
- 500MB+ disk space

#### 1.2 Install Dependencies

```bash
# Install Rust toolchain (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify Rust installation
rustc --version
cargo --version

# Install system dependencies
sudo pacman -S base-devel systemd
```

#### 1.3 Prepare Working Directory

```bash
# Clone repository
cd ~
git clone https://github.com/anna-assistant/anna anna-assistant
cd anna-assistant

# Verify contents
ls -la
# Should see: Cargo.toml, src/, scripts/, tests/, docs/, packaging/
```

---

### Step 2: Installation

#### 2.1 Run Installer

```bash
# Run as sudo from project root
sudo ./scripts/install.sh
```

**What the installer does:**
1. Compiles binaries in release mode (~2-3 minutes)
2. Creates `anna` system group
3. Adds current user to `anna` group
4. Creates directories with proper permissions
5. Installs binaries to `/usr/local/bin`
6. Installs systemd service and tmpfiles
7. Enables and starts `annad.service`
8. Runs post-install validation

#### 2.2 Expected Output

```
╔═══════════════════════════════════════╗
║                                       ║
║        ANNA ASSISTANT v0.9.2          ║
║     Next-Gen Linux System Helper      ║
║   Sprint 3: Intelligence & Policies   ║
║                                       ║
╚═══════════════════════════════════════╝

[INFO] Checking system requirements...
[OK] All requirements satisfied
[INFO] Compiling Anna (this may take a few minutes)...
   Compiling annad v0.9.2 (/home/user/anna-assistant)
   Compiling annactl v0.9.2 (/home/user/anna-assistant)
    Finished release [optimized] target(s) in 2m 34s
[OK] Compilation complete
[INFO] Setting up anna group...
[OK] Group 'anna' created
[INFO] Adding user to anna group...
[OK] User 'lhoqvso' added to group 'anna'
[WARN] NOTE: Group membership requires logout/login to take effect
[INFO] Installing binaries to /usr/local/bin...
[OK] Binaries installed
[INFO] Installing systemd service...
[OK] Tmpfiles configuration installed
[OK] Systemd service installed
[INFO] Installing polkit policy...
[OK] Polkit policy installed
[INFO] Installing bash completion...
[OK] Bash completion installed
[INFO] Setting up directories with correct permissions...
[OK] Directories created with correct permissions
[INFO] Setting up configuration...
[OK] Default configuration created
[OK] Example policies installed
[INFO] Creating user paths...
[OK] User paths created for lhoqvso
[INFO] Enabling and starting annad service...
[INFO] Waiting for socket creation...
[OK] Service started successfully
[INFO] Running post-install validation...
[OK] Socket exists
[OK] Socket permissions correct (660)
[INFO] Testing annactl commands...
[OK] annactl ping: OK
[OK] annactl status: OK
[OK] All validation checks passed

╔═══════════════════════════════════════╗
║                                       ║
║   INSTALLATION COMPLETE!              ║
║                                       ║
╚═══════════════════════════════════════╝

Quick start:
  annactl status              - Check daemon status
  annactl doctor              - Run diagnostics
  annactl config list         - List configuration
  annactl policy list         - List policies
  annactl events show         - Show recent events
  annactl learning stats      - Learning statistics

Service management:
  sudo systemctl status annad
  sudo systemctl restart annad
  sudo journalctl -u annad -f

IMPORTANT: Group membership requires logout/login to take effect
Temporary workaround: Run 'newgrp anna' in your shell
```

#### 2.3 Handle Group Membership

**Important:** After installation, you need to refresh group membership.

**Option A: Logout/Login (Recommended)**
```bash
# Log out and log back in
exit
# Or reboot
sudo reboot
```

**Option B: Temporary Workaround**
```bash
# Start a new shell with anna group
newgrp anna

# Test commands in this shell
annactl status
```

---

### Step 3: Verification

#### 3.1 Check Service Status

```bash
# Verify service is running
sudo systemctl status annad

# Expected output:
# ● annad.service - Anna Assistant Daemon
#      Loaded: loaded (/etc/systemd/system/annad.service; enabled; preset: disabled)
#      Active: active (running) since Thu 2025-10-30 10:00:00 CET; 1min ago
#    Main PID: 12345 (annad)
#       Tasks: 4 (limit: 38400)
#      Memory: 12.5M
#         CPU: 45ms
#      CGroup: /system.slice/annad.service
#              └─12345 /usr/local/bin/annad
```

#### 3.2 Check Socket

```bash
# Verify socket exists with correct permissions
ls -lh /run/anna/annad.sock

# Expected output:
# srw-rw---- 1 root anna 0 Oct 30 10:00 /run/anna/annad.sock
```

#### 3.3 Test Commands

```bash
# Test connectivity
annactl ping
# Expected: pong

# Check daemon status
annactl status
# Expected: JSON with version, uptime, etc.

# List configuration
annactl config list
# Expected: Config keys and values

# Run diagnostics
annactl doctor
# Expected: System health report
```

#### 3.4 Check Logs

```bash
# View daemon logs
sudo journalctl -u annad --since -5m

# Look for successful startup sequence:
# [BOOT] Anna Assistant Daemon v0.9.2 starting...
# [BOOT] Directories initialized
# [BOOT] Config loaded
# [BOOT] Persistence ready
# [BOOT] RPC online (/run/anna/annad.sock)
# [BOOT] Policy/Event/Learning subsystems active
# [READY] anna-assistant operational
```

---

### Step 4: Runtime Validation

Run the comprehensive validation suite:

```bash
# Run full runtime validation
sudo bash tests/runtime_validation.sh
```

**Expected:** All 12 tests pass with green PASS indicators.

See `docs/RUNTIME-VALIDATION-Sprint3.md` for detailed test descriptions and troubleshooting.

---

## Directory Structure

After installation, the following structure is created:

```
/usr/local/
├── bin/
│   ├── annad              # Daemon (755 root:root)
│   └── annactl            # CLI (755 root:root)

/etc/
├── anna/
│   ├── config.toml        # Main config (640 root:anna)
│   └── policies.d/        # Policy directory (750 root:anna)
│       └── *.yaml         # Example policies (640 root:anna)
└── systemd/
    └── system/
        └── annad.service  # Systemd unit

/usr/lib/
└── tmpfiles.d/
    └── annad.conf         # Runtime dir config

/var/lib/
└── anna/                  # State directory (750 root:anna)
    ├── state/             # Persistent state
    └── events/            # Event logs

/run/
└── anna/                  # Runtime directory (770 root:anna)
    └── annad.sock         # Unix socket (660 root:anna)

/usr/share/
├── polkit-1/
│   └── actions/
│       └── com.anna.policy
└── bash-completion/
    └── completions/
        └── annactl

~/.config/
└── anna/                  # User configuration

~/.local/share/
└── anna/
    └── events/            # User telemetry
```

---

## Common Issues & Solutions

### Issue 1: "annad not running"

**Symptom:**
```
❌ annad not running or socket unavailable
```

**Solution:**
```bash
# Check service status
sudo systemctl status annad

# If not running, start it
sudo systemctl start annad

# Check for errors in logs
sudo journalctl -u annad --since -5m
```

### Issue 2: Permission denied

**Symptom:**
```
Error: Permission denied (os error 13)
```

**Solution:**
```bash
# Verify group membership
groups | grep anna

# If not in anna group, add yourself
sudo usermod -aG anna $USER

# Logout and login, or use newgrp
newgrp anna
```

### Issue 3: Socket has wrong permissions

**Symptom:**
```
Socket exists but connection fails
```

**Solution:**
```bash
# Check socket permissions
stat /run/anna/annad.sock

# Fix if needed
sudo chmod 0660 /run/anna/annad.sock
sudo chown root:anna /run/anna/annad.sock

# Restart service
sudo systemctl restart annad
```

### Issue 4: Compilation fails

**Symptom:**
```
error: rustup could not choose a version of cargo to run
```

**Solution:**
```bash
# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Set default toolchain
rustup default stable

# Retry installation
sudo ./scripts/install.sh
```

---

## Uninstallation

To remove Anna Assistant:

```bash
# Run uninstaller (creates backup)
sudo ./scripts/uninstall.sh
```

**This will:**
1. Stop and disable the service
2. Create timestamped backup in `~/Documents/anna_backup_<timestamp>/`
3. Remove binaries, configs, and state
4. Provide restore instructions

**Backup location:**
```
~/Documents/anna_backup_20251030_100000/
├── etc_anna/              # Config backup
├── var_lib_anna/          # State backup
├── annad                  # Binary backup
├── annactl                # Binary backup
└── README-RESTORE.md      # Restore instructions
```

---

## Advanced Configuration

### Service Management

```bash
# Start/stop/restart
sudo systemctl start annad
sudo systemctl stop annad
sudo systemctl restart annad

# Enable/disable autostart
sudo systemctl enable annad
sudo systemctl disable annad

# View logs in real-time
sudo journalctl -u annad -f

# View last 100 lines
sudo journalctl -u annad -n 100
```

### Configuration

```bash
# Edit main config
sudo nano /etc/anna/config.toml

# List current config
annactl config list

# Get specific value
annactl config get autonomy_level

# Set value
annactl config set autonomy_level 1
```

### Policies

```bash
# List policies
annactl policy list

# View policy details
annactl policy show <policy-name>

# Add custom policy
sudo nano /etc/anna/policies.d/custom.yaml
```

---

## CI/CD Integration

### Automated Testing

```bash
#!/bin/bash
# Deploy and validate Anna in CI environment

set -euo pipefail

# Clone
git clone https://github.com/anna-assistant/anna anna-assistant
cd anna-assistant

# Install
sudo ./scripts/install.sh

# Run validation
sudo bash tests/runtime_validation.sh

# Check exit code
if [ $? -eq 0 ]; then
    echo "✓ Deployment validated successfully"
    exit 0
else
    echo "✗ Deployment validation failed"
    sudo journalctl -u annad --since -5m
    exit 1
fi
```

### Docker/VM Testing

```dockerfile
FROM archlinux:latest

# Install dependencies
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm base-devel rust systemd

# Copy source
COPY . /anna-assistant
WORKDIR /anna-assistant

# Run tests
RUN bash tests/qa_runner.sh

# Install
RUN ./scripts/install.sh

# Validate
RUN bash tests/runtime_validation.sh
```

---

## Support & Documentation

- **Runtime Validation:** `docs/RUNTIME-VALIDATION-Sprint3.md`
- **QA Results:** `docs/QA-RESULTS-Sprint3.md`
- **Architecture:** `docs/ARCHITECTURE.md`
- **Contributing:** `CONTRIBUTING.md`
- **Issues:** https://github.com/anna-assistant/anna/issues

---

## Checklist for New Deployment

Use this checklist to verify a successful deployment:

```
□ System Requirements
  □ Arch Linux or systemd-based distro
  □ Rust toolchain installed
  □ Sudo access verified
  □ 2GB+ RAM available

□ Installation
  □ Cloned repository
  □ sudo ./scripts/install.sh completed without errors
  □ All compilation steps succeeded
  □ Anna group created
  □ User added to anna group
  □ Binaries installed

□ Service Verification
  □ systemctl is-active annad → active
  □ systemctl is-enabled annad → enabled
  □ journalctl shows [READY] message
  □ No errors in daemon logs

□ Socket Verification
  □ /run/anna/annad.sock exists
  □ Socket permissions: 0660 root:anna
  □ Socket accessible by anna group

□ Command Verification
  □ annactl ping succeeds
  □ annactl status returns JSON
  □ annactl config list works
  □ annactl doctor passes

□ Runtime Validation
  □ sudo bash tests/runtime_validation.sh
  □ All 12 tests pass
  □ Log file created in tests/logs/

□ Production Readiness
  □ Service survives restart
  □ Socket recreated after removal
  □ Multiple commands work concurrently
  □ Uninstall creates backup successfully
```

---

**Document Version:** 1.0
**Last Updated:** 2025-10-30
**Status:** Ready for Privileged Testing
**Sprint:** 3 - Runtime Validation
