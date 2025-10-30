# Sprint 3 Runtime Validation Guide

**Version:** v0.9.2a
**Status:** Pre-Release (Requires Privileged Testing)
**Target:** Arch Linux with sudo/root access

---

## Overview

This document describes the complete runtime validation process for Anna Assistant Sprint 3. Unlike unit tests which validate code logic, runtime validation tests the actual deployment, systemd integration, socket creation, and real-world daemon operation.

## Prerequisites

### System Requirements
- **OS:** Arch Linux (preferred) or systemd-based Linux
- **Access:** sudo or root privileges
- **Rust:** Latest stable toolchain
- **Tools:** systemctl, journalctl, cargo, bash

### Clean Environment
For best results, test on a clean system or VM:
```bash
# Check if anna is already installed
which annad annactl

# If installed, uninstall first
sudo ./scripts/uninstall.sh
```

---

## Runtime Validation Tests

The runtime validation suite (`tests/runtime_validation.sh`) performs **12 comprehensive tests**:

### Test Suite Breakdown

| # | Test | Description | Expected Result |
|---|------|-------------|-----------------|
| 1 | Installation | Runs `./scripts/install.sh` | Exit 0, all steps pass |
| 2 | Service Status | `systemctl is-active annad` | `active` |
| 3 | Socket Existence | Check `/run/anna/annad.sock` | Socket file exists |
| 4 | Socket Permissions | Verify permissions and ownership | `0660 root:anna` or `0666` |
| 5 | annactl ping | Test daemon connectivity | `pong` response |
| 6 | annactl status | Daemon status query | JSON response with version |
| 7 | annactl config list | Configuration listing | Config keys displayed |
| 8 | annactl telemetry stats | Telemetry statistics | Event count ≥ 0 |
| 9 | annactl policy list | Policy listing | Policies shown or empty |
| 10 | Journal Logs | Check for `[READY]` message | Found in recent logs |
| 11 | Directory Permissions | Verify `/etc/anna`, `/var/lib/anna`, `/run/anna` | Correct permissions set |
| 12 | Anna Group | Check group existence | Group `anna` exists |

### Expected Output Format

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

Log file: tests/logs/runtime_validation.log

✓ All runtime validation tests passed!
✓ Sprint 3 runtime validation: COMPLETE
```

---

## Running Runtime Validation

### Method 1: Standalone Script (Recommended)

```bash
# Navigate to project root
cd anna-assistant

# Run validation with sudo
sudo bash tests/runtime_validation.sh
```

**This will:**
1. Run the installer
2. Start the daemon
3. Test all functionality
4. Generate detailed logs in `tests/logs/runtime_validation.log`

### Method 2: Via QA Runner

```bash
# Run full QA suite including runtime validation
bash tests/qa_runner.sh
```

**Note:** QA runner will automatically:
- Run 134+ unit tests first
- Check for sudo availability
- Run runtime validation if sudo is accessible
- Skip gracefully with instructions if sudo unavailable

### Method 3: Manual Step-by-Step

For debugging or educational purposes:

```bash
# 1. Install
sudo ./scripts/install.sh

# 2. Verify service
sudo systemctl status annad

# 3. Check socket
ls -lh /run/anna/annad.sock

# 4. Test commands
annactl status
annactl ping
annactl config list
annactl telemetry stats
annactl policy list
annactl events show --limit 5

# 5. View logs
sudo journalctl -u annad --since -5m
```

---

## Expected System State After Validation

### Files Installed
```
/usr/local/bin/annad           (755 root:root)
/usr/local/bin/annactl         (755 root:root)
/etc/systemd/system/annad.service
/usr/lib/tmpfiles.d/annad.conf
/etc/anna/config.toml          (640 root:anna)
/etc/anna/policies.d/*.yaml    (640 root:anna)
/usr/share/polkit-1/actions/com.anna.policy
/usr/share/bash-completion/completions/annactl
```

### Directories Created
```
/etc/anna/              (0750 root:anna)
/etc/anna/policies.d/   (0750 root:anna)
/var/lib/anna/          (0750 root:anna)
/var/lib/anna/state/    (0750 root:anna)
/var/lib/anna/events/   (0750 root:anna)
/run/anna/              (0770 root:anna)
```

### Socket
```
/run/anna/annad.sock    (0660 root:anna)
```

### Service Status
```bash
$ sudo systemctl is-enabled annad
enabled

$ sudo systemctl is-active annad
active
```

### Group Membership
```bash
$ getent group anna
anna:x:989:lhoqvso

$ groups
lhoqvso wheel anna docker ...
```

---

## Troubleshooting

### Issue: Service fails to start

**Symptom:**
```
● annad.service - Anna Assistant Daemon
     Active: failed (Result: exit-code)
```

**Diagnosis:**
```bash
# Check logs
sudo journalctl -u annad --since -5m

# Common errors:
# - "Read-only file system" → Check systemd service hardening
# - "Permission denied" → Check socket directory permissions
# - "os error 2" → Binary not found or missing dependencies
```

**Solutions:**
1. Verify binary exists: `ls -lh /usr/local/bin/annad`
2. Check permissions: `stat /run/anna`
3. Disable strict systemd hardening temporarily (see service file)
4. Reinstall: `sudo ./scripts/uninstall.sh && sudo ./scripts/install.sh`

### Issue: Socket not created

**Symptom:**
```
ls: cannot access '/run/anna/annad.sock': No such file or directory
```

**Diagnosis:**
```bash
# Check runtime directory
ls -lh /run/anna/

# Check if systemd created it
systemctl show annad -p RuntimeDirectory
```

**Solutions:**
1. Ensure tmpfiles is configured: `ls -lh /usr/lib/tmpfiles.d/annad.conf`
2. Manually create runtime: `sudo systemd-tmpfiles --create /usr/lib/tmpfiles.d/annad.conf`
3. Check service logs for early failures

### Issue: annactl can't connect

**Symptom:**
```
❌ annad not running or socket unavailable
Error: Connection refused
```

**Diagnosis:**
```bash
# Check service
sudo systemctl status annad

# Check socket
ls -lh /run/anna/annad.sock

# Check group membership
groups | grep anna
```

**Solutions:**
1. Ensure service is running: `sudo systemctl start annad`
2. Verify group membership: `sudo usermod -aG anna $USER`
3. Refresh group membership: `newgrp anna` (or logout/login)
4. Check socket permissions: `sudo chmod 0660 /run/anna/annad.sock`

### Issue: Permission denied on config

**Symptom:**
```
Error: Permission denied (os error 13)
```

**Diagnosis:**
```bash
# Check directory ownership
stat /etc/anna /var/lib/anna

# Check group membership
id -nG
```

**Solutions:**
1. Fix ownership: `sudo chown -R root:anna /etc/anna /var/lib/anna`
2. Fix permissions: `sudo chmod 0750 /etc/anna /var/lib/anna`
3. Add user to group: `sudo usermod -aG anna $USER`

### Issue: Compilation fails

**Symptom:**
```
error: rustup could not choose a version of cargo to run
```

**Solutions:**
1. Install rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Set default toolchain: `rustup default stable`
3. Source environment: `source ~/.cargo/env`

---

## Success Criteria

Sprint 3 runtime validation is **COMPLETE** when:

- ✅ All 12 runtime tests pass
- ✅ Service starts cleanly with `[READY]` message in logs
- ✅ Socket exists with correct permissions (0660 root:anna)
- ✅ All `annactl` commands work without errors
- ✅ No regressions in unit tests (134/134 pass)
- ✅ Daemon survives restart: `sudo systemctl restart annad`
- ✅ Clean uninstall with backup: `sudo ./scripts/uninstall.sh`

---

## Validation Checklist

Use this checklist when performing manual validation:

```
□ Fresh system or VM with Arch Linux
□ Rust toolchain installed and configured
□ Cloned repository to /home/user/anna-assistant
□ No previous Anna installation

Installation Phase:
□ sudo ./scripts/install.sh runs without errors
□ Compilation completes successfully
□ Anna group created
□ Current user added to anna group
□ Binaries installed to /usr/local/bin
□ Systemd service enabled and started
□ Socket created at /run/anna/annad.sock

Service Validation:
□ systemctl is-active annad returns "active"
□ systemctl is-enabled annad returns "enabled"
□ journalctl -u annad shows [READY] message
□ No error messages in daemon logs

Socket Validation:
□ Socket exists at /run/anna/annad.sock
□ Socket permissions are 0660 or 0666
□ Socket ownership is root:anna
□ Socket is writable by anna group members

Command Validation:
□ annactl ping succeeds
□ annactl status returns JSON
□ annactl config list shows configuration
□ annactl telemetry stats returns event count
□ annactl policy list shows policies or empty
□ annactl events show works
□ annactl learning stats works
□ annactl doctor runs without errors

Permission Validation:
□ /etc/anna permissions are 0750 root:anna
□ /var/lib/anna permissions are 0750 root:anna
□ /run/anna permissions are 0770 root:anna
□ User is in anna group: groups | grep anna

Stress Testing:
□ Service survives restart: sudo systemctl restart annad
□ Socket recreated after removal: sudo rm /run/anna/annad.sock && sudo systemctl restart annad
□ Multiple concurrent annactl commands work

Cleanup:
□ sudo ./scripts/uninstall.sh runs cleanly
□ Backup created in ~/Documents/anna_backup_<timestamp>
□ All binaries removed
□ Service disabled and stopped
□ Directories removed
□ Anna group handled appropriately
```

---

## Next Steps

After successful runtime validation:

1. **Document Results**
   - Save logs from `tests/logs/runtime_validation.log`
   - Capture `journalctl -u annad` output
   - Screenshot successful test output

2. **Update Changelog**
   - Mark v0.9.2a as validated
   - Note any issues encountered and resolved

3. **Seal Sprint 3**
   - Commit with message: "Sprint 3 Runtime Validation Complete (v0.9.2a)"
   - Tag: `v0.9.2a-validated`
   - Update project status documents

4. **Prepare for Production**
   - Create release tarball
   - Generate installation guide
   - Document deployment procedures

---

## Log Analysis

Key log messages to look for:

### Successful Startup
```
[BOOT] Anna Assistant Daemon v0.9.2 starting...
[BOOT] Directories initialized
[BOOT] Config loaded
[BOOT] Persistence ready
[BOOT] RPC online (/run/anna/annad.sock)
[BOOT] Policy/Event/Learning subsystems active
[READY] anna-assistant operational
```

### Successful Command
```
INFO annad::rpc: [RPC] Received request: ping
INFO annad::rpc: [RPC] Handled ping in 0.01ms
```

### Error Patterns to Watch For
```
ERROR [FATAL] annad must run as root
Error: Read-only file system (os error 30)
Error: Permission denied (os error 13)
Error: Connection refused (os error 111)
```

---

## Contact & Support

For issues during runtime validation:

1. **Check Logs:** `tests/logs/runtime_validation.log`
2. **Check Daemon Logs:** `sudo journalctl -u annad --since -10m`
3. **Run Doctor:** `annactl doctor` (if daemon is running)
4. **GitHub Issues:** https://github.com/anna-assistant/anna/issues

---

**Document Version:** 1.0
**Last Updated:** 2025-10-30
**Sprint:** 3 - Runtime Validation
**Status:** Ready for Privileged Testing
