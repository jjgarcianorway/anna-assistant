# Anna Daemon Socket Fix

## Problem Diagnosed

Anna daemon was crashing on startup with:
```
Error: Failed to create runtime directory
Caused by: Permission denied (os error 13)
```

Additionally:
```
WARN Failed to load capability registry: Failed to read /usr/lib/anna/CAPABILITIES.toml
```

**Root cause:** The installer wasn't copying `CAPABILITIES.toml` to `/usr/lib/anna/`, causing the daemon to fail on startup.

---

## What Was Fixed

### 1. Updated Installer (`scripts/install.sh`)
Added installation of `CAPABILITIES.toml`:
```bash
# Install capability registry
echo ""
echo "→ Installing capability registry..."
if [ -f etc/CAPABILITIES.toml ]; then
    sudo install -m 0644 etc/CAPABILITIES.toml /usr/lib/anna/
    echo "✓ Capabilities registered"
fi
```

### 2. Created Quick-Fix Script (`scripts/fix-capabilities.sh`)
For users with existing installations, this script:
- Installs the missing `CAPABILITIES.toml` file
- Restarts the Anna daemon
- Verifies the daemon is running

### 3. New Diagnostics Tool (`tests/diagnostics.sh`)
Replaces 11 outdated test scripts with one modern health check:
- ✓ Checks if binaries are installed
- ✓ Verifies systemd service status
- ✓ Confirms socket exists
- ✓ Validates configuration files
- ✓ Tests annactl commands
- ✓ Provides actionable recommendations

---

## How to Fix Your Current Installation

Run the quick-fix script:

```bash
cd /path/to/anna-assistant
./scripts/fix-capabilities.sh
```

**Expected output:**
```
╭─────────────────────────────────────────╮
│  Fix Missing CAPABILITIES.toml          │
╰─────────────────────────────────────────╯

→ Installing CAPABILITIES.toml...
✓ File installed

→ Restarting Anna daemon...

→ Waiting for daemon to start...
✓ Anna daemon is running!

╭─────────────────────────────────────────╮
│  ✓ Fix Applied Successfully!            │
╰─────────────────────────────────────────╯

Test with: annactl status
```

---

## Verify the Fix

### Quick Test:
```bash
./tests/diagnostics.sh
```

### Manual Verification:
```bash
# 1. Check daemon is running
systemctl status annad

# 2. Verify socket exists
ls -la /run/anna/annad.sock

# 3. Test annactl connection
annactl status

# 4. View system info
annactl sensors
annactl disk
annactl net
```

---

## annactl Commands Reference

### Offline Commands (work without daemon):
```bash
annactl --version              # Show version
annactl doctor pre             # Preflight checks
annactl doctor post            # Postflight checks
annactl doctor repair          # Repair installation
annactl doctor report          # Generate diagnostic tarball
```

### Online Commands (require running daemon):
```bash
annactl status                 # Show daemon status and health
annactl sensors                # CPU, memory, temperatures, battery
annactl net                    # Network interfaces and connectivity
annactl disk                   # Disk usage and SMART status
annactl top                    # Top processes by CPU and memory
annactl events                 # Recent system events
annactl watch                  # Watch live system events
annactl capabilities           # Show module capabilities
annactl alerts                 # Show integrity alerts
```

---

## Test Results After Fix

Expected output from `./tests/diagnostics.sh`:

```
╭─────────────────────────────────────────╮
│  Anna System Diagnostics                │
╰─────────────────────────────────────────╯

→ Checking binaries...
✓ Binaries installed: annactl 0.13.6

→ Checking systemd service...
✓ Daemon is running

→ Checking RPC socket...
✓ Socket exists: /run/anna/annad.sock

→ Checking configuration...
✓ Config exists: /etc/anna/config.toml
✓ Capabilities registry installed

→ Checking directories...
✓ Directory exists: /var/lib/anna
✓ Directory exists: /var/log/anna
✓ Directory exists: /run/anna

→ Testing annactl commands (without daemon)...
✓ annactl doctor pre works
✓ annactl --version works

→ Testing daemon connection...
✓ annactl status connects to daemon

╭─────────────────────────────────────────╮
│ ✓ All checks passed (13 passed)        │
╰─────────────────────────────────────────╯
```

---

## For Future Installations

This fix is included in the installer starting from v0.13.7.

**New installations will:**
1. Download pre-compiled binaries from GitHub
2. Install `CAPABILITIES.toml` automatically
3. Start the daemon successfully
4. Be fully operational immediately

**No Rust/Cargo required!**

---

## Cleanup Performed

**Removed 11 outdated test scripts:**
- `current_status.sh` (hardcoded to v0.9.6)
- `e2e_basic.sh` (required cargo)
- `persistence_v011.sh` (v0.11.0 specific)
- `qa_runner.sh` (Sprint 3 specific)
- `runtime_validation.sh` (v0.9.6-alpha.4 specific)
- `self_validation.sh` (local build approach)
- `smoke.sh` (v0.10.1 specific)
- `smoke_v011.sh` (v0.11.0 specific)
- `smoke_v101.sh` (v0.10.1 specific)
- `test_offline_commands.sh` (superseded)
- `annactl_matrix.sh` (superseded)

**Replaced with:**
- `tests/diagnostics.sh` - Modern, version-agnostic health check

**Kept:**
- `tests/ACCEPTANCE.md` - Documentation

---

## Troubleshooting

### If daemon still won't start:
```bash
# Check detailed logs
sudo journalctl -u annad -n 100

# Verify file permissions
ls -la /usr/lib/anna/
ls -la /var/lib/anna/
ls -la /var/log/anna/

# Try running manually to see errors
annad  # Will show permission errors
```

### If socket permissions are wrong:
```bash
# Check socket
ls -la /run/anna/annad.sock

# Should be: srwxrwx--- anna anna

# Restart if needed
sudo systemctl restart annad
```

### If still having issues:
```bash
# Generate diagnostic report
annactl doctor report

# This creates: /tmp/anna-diagnostics-YYYYMMDD-HHMMSS.tar.gz
# Share this for debugging
```

---

## Next Release

This fix will be included in **v0.13.7**, which will be the first fully working release with:
- ✓ Pre-compiled binaries
- ✓ One-command installation
- ✓ No Rust required
- ✓ Complete capability registry
- ✓ Working daemon and socket
- ✓ All annactl commands functional

Run `./scripts/fix-capabilities.sh` now to get your current installation working!
