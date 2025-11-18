# Anna Assistant v0.9.2a - Runtime Validation Kit

**Version:** 0.9.2a Pre-Release
**Date:** 2025-10-30
**Status:** Ready for Privileged Testing
**SHA256:** e875c9304340e2f7059db7b66d036f49d146c8de6c8e919748d8939b04e61748

---

## What This Is

This tarball contains the **Sprint 3 Runtime Validation Kit** for Anna Assistant - a complete, self-contained package for deploying and validating Anna on a privileged Arch Linux environment.

Unlike the development codebase, this kit focuses solely on **runtime validation** - proving that Anna actually works when installed as a system service with proper permissions, sockets, and systemd integration.

---

## Contents

```
anna-v0.9.2a-runtime-validation.tar.gz (70KB)
├── src/                            # Source code (annad, annactl)
├── scripts/
│   ├── install.sh                  # Idempotent installer (447 lines)
│   └── uninstall.sh                # Safe uninstaller with backup
├── packaging/arch/
│   ├── annad.service               # Systemd unit
│   └── annad.tmpfiles.conf         # Runtime directory config
├── tests/
│   ├── runtime_validation.sh       # 12-test validation suite
│   └── qa_runner.sh                # Full QA harness (134+ tests)
├── docs/
│   ├── RUNTIME-VALIDATION-Sprint3.md   # Complete validation guide
│   └── policies.d/                     # Example policies
├── DEPLOYMENT-INSTRUCTIONS.md      # Step-by-step deployment guide
├── CHANGELOG.md                    # v0.9.2a changes
├── README.md                       # Project overview
├── Cargo.toml / Cargo.lock         # Build manifest
├── config/                         # Default configuration
└── polkit/                         # Polkit policy
```

---

## Quick Start

### Prerequisites

- **System:** Arch Linux (or systemd-based distro)
- **Access:** sudo or root privileges
- **Tools:** rust, cargo, systemd, bash

### 3-Step Validation

```bash
# 1. Extract
tar -xzf anna-v0.9.2a-runtime-validation.tar.gz
cd anna-assistant

# 2. Install (as sudo from project root)
sudo ./scripts/install.sh

# 3. Validate
sudo bash tests/runtime_validation.sh
```

**Expected Result:** All 12 tests pass with green `PASS` indicators.

---

## What Gets Tested

The runtime validation suite verifies:

1. **Installation** - Complete build and install process
2. **Service Status** - `systemctl is-active annad` returns `active`
3. **Socket Existence** - `/run/anna/annad.sock` is created
4. **Socket Permissions** - Correct ownership (root:anna) and mode (0660)
5. **Connectivity** - `annactl ping` succeeds
6. **Status Query** - `annactl status` returns JSON
7. **Configuration** - `annactl config list` works
8. **Telemetry** - `annactl telemetry stats` returns data
9. **Policies** - `annactl policy list` works
10. **Daemon Logs** - `[READY]` message appears in journal
11. **Directory Permissions** - All dirs have correct ownership/modes
12. **Group Management** - Anna group exists with members

---

## Expected Output

### Successful Validation

```
╔═══════════════════════════════════════════════════╗
║                                                   ║
║     ANNA ASSISTANT v0.9.2a                        ║
║     Sprint 3 Runtime Validation                   ║
║                                                   ║
╚═══════════════════════════════════════════════════╝

Running full end-to-end runtime validation...

[TEST 1] Running installation script... PASS (45.2s)
[TEST 2] Checking systemd service status... PASS (0.3s)
[TEST 3] Checking socket existence... PASS (0.1s)
[TEST 4] Verifying socket permissions... PASS (0.1s)
[TEST 5] Testing annactl ping... PASS (0.2s)
[TEST 6] Testing annactl status... PASS (0.2s)
[TEST 7] Testing annactl config list... PASS (0.2s)
[TEST 8] Testing annactl telemetry stats... PASS (0.2s)
[TEST 9] Testing annactl policy list... PASS (0.2s)
[TEST 10] Checking daemon logs... PASS (0.3s)
[TEST 11] Verifying directory permissions... PASS (0.1s)
[TEST 12] Checking anna group... PASS (0.1s)

╔═══════════════════════════════════════════════════╗
║                                                   ║
║     RUNTIME VALIDATION COMPLETE                   ║
║                                                   ║
╚═══════════════════════════════════════════════════╝

Results:
  Total tests:  12
  Passed:       12
  Failed:       0
  Duration:     47s

✓ All runtime validation tests passed!
✓ Sprint 3 runtime validation: COMPLETE
```

---

## Validation Procedure

### Step 1: Prepare System

```bash
# Verify Arch Linux
cat /etc/os-release | grep Arch

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify sudo
sudo -v
```

### Step 2: Extract and Install

```bash
# Extract tarball
tar -xzf anna-v0.9.2a-runtime-validation.tar.gz
cd anna-assistant

# Run installer (compiles and installs)
sudo ./scripts/install.sh

# Expected: Compilation succeeds, service starts, socket created
```

### Step 3: Run Validation

```bash
# Full runtime validation suite
sudo bash tests/runtime_validation.sh

# Check logs
cat tests/logs/runtime_validation.log

# View daemon logs
sudo journalctl -u annad --since -5m
```

### Step 4: Manual Verification (Optional)

```bash
# Test individual commands
annactl status
annactl ping
annactl config list
annactl telemetry stats
annactl policy list
annactl events show --limit 5

# Check socket
ls -lh /run/anna/annad.sock

# Verify service
sudo systemctl status annad
```

---

## Troubleshooting

### Issue: Compilation Fails

```bash
# Ensure rust is installed
rustup default stable
source ~/.cargo/env

# Retry install
sudo ./scripts/install.sh
```

### Issue: Service Won't Start

```bash
# Check logs
sudo journalctl -u annad --since -5m

# Common fixes:
sudo systemctl daemon-reload
sudo systemctl restart annad
```

### Issue: Socket Not Created

```bash
# Manually trigger tmpfiles
sudo systemd-tmpfiles --create /usr/lib/tmpfiles.d/annad.conf

# Restart service
sudo systemctl restart annad
```

### Issue: Permission Denied

```bash
# Add user to anna group
sudo usermod -aG anna $USER

# Apply immediately (or logout/login)
newgrp anna

# Test again
annactl status
```

---

## What Success Means

When all 12 tests pass, it proves:

✅ **Build System Works** - Cargo successfully compiles release binaries
✅ **Installation Succeeds** - All files placed in correct locations with proper permissions
✅ **Systemd Integration** - Service starts automatically, socket created via RuntimeDirectory
✅ **Permission Model** - Group-based access control functions correctly
✅ **Socket IPC** - Unix domain socket communication works
✅ **CLI Commands** - All annactl operations succeed
✅ **Daemon Stability** - Service runs without errors or crashes
✅ **Directory Structure** - All dirs exist with correct ownership and modes
✅ **Logging** - Structured startup sequence with [READY] message
✅ **Group Management** - Anna group created and users added
✅ **Telemetry** - Event system functional
✅ **Policy Engine** - Policy loading and evaluation works

---

## Next Steps After Validation

### If All Tests Pass

1. **Document Results**
   ```bash
   # Save validation log
   cp tests/logs/runtime_validation.log ~/validation-success-$(date +%Y%m%d).log

   # Capture daemon logs
   sudo journalctl -u annad --since -10m > ~/annad-logs-$(date +%Y%m%d).log
   ```

2. **Seal Sprint 3**
   - Update project status to "Runtime Validated"
   - Tag repository: `v0.9.2a-validated`
   - Mark Sprint 3 as complete

3. **Production Preparation**
   - Create release tarball with precompiled binaries
   - Write deployment playbooks (Ansible/Docker)
   - Prepare monitoring and alerting

### If Tests Fail

1. **Capture Diagnostics**
   ```bash
   # Save failure log
   cp tests/logs/runtime_validation.log ~/validation-failure-$(date +%Y%m%d).log

   # Gather system info
   sudo journalctl -u annad --since -10m > ~/annad-failure-$(date +%Y%m%d).log
   systemctl status annad --no-pager >> ~/annad-failure-$(date +%Y%m%d).log
   ```

2. **Review Documentation**
   - See `docs/RUNTIME-VALIDATION-Sprint3.md` for troubleshooting
   - Check `DEPLOYMENT-INSTRUCTIONS.md` for step-by-step guidance

3. **Report Issues**
   - File bug report with logs attached
   - Include system details (uname -a, systemctl --version)

---

## Files Modified in v0.9.2a

### New Files
- `tests/runtime_validation.sh` - 12-test validation suite
- `docs/RUNTIME-VALIDATION-Sprint3.md` - Complete validation guide
- `DEPLOYMENT-INSTRUCTIONS.md` - Deployment procedures
- `packaging/arch/annad.tmpfiles.conf` - Runtime dir config

### Modified Files
- `src/annad/src/main.rs` - Hardened initialization, structured logging
- `src/annactl/src/main.rs` - Enhanced error messages with troubleshooting
- `scripts/install.sh` - Anna group management, permission fixing, post-install validation
- `scripts/uninstall.sh` - Timestamped backups, restore instructions
- `packaging/arch/annad.service` - Relaxed security hardening (fixed read-only fs errors)
- `tests/qa_runner.sh` - Runtime validation stage integration
- `CHANGELOG.md` - v0.9.2a section with all changes

---

## Support

- **Validation Guide:** `docs/RUNTIME-VALIDATION-Sprint3.md`
- **Deployment Guide:** `DEPLOYMENT-INSTRUCTIONS.md`
- **Changelog:** `CHANGELOG.md` (v0.9.2a section)
- **Issues:** https://github.com/anna-assistant/anna/issues

---

## Checksum Verification

```bash
# Verify tarball integrity
echo "e875c9304340e2f7059db7b66d036f49d146c8de6c8e919748d8939b04e61748  anna-v0.9.2a-runtime-validation.tar.gz" | sha256sum -c

# Expected output:
# anna-v0.9.2a-runtime-validation.tar.gz: OK
```

---

**Prepared by:** Claude Code (Anthropic)
**Sprint:** 3 - Runtime Validation
**Status:** Ready for Privileged Testing
**Target:** Arch Linux with sudo/root access

---

## License

See LICENSE file in the repository.
