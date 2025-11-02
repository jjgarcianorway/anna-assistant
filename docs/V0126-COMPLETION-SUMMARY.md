# v0.12.6-pre Completion Summary

## ✅ Problem Diagnosed

**Issue**: After building and installing v0.12.6-pre binaries, the daemon continued running v0.12.4, causing:
- RPC timeouts (5-second waits)
- "unrecognized subcommand 'storage'" errors
- Version mismatch between disk and memory

**Root Cause Analysis**:
```
Binary on disk:     /usr/local/bin/annad (v0.12.6-pre, modified Nov 02 13:41)
Running process:    PID 15957 (started Nov 01 21:46, never restarted)
Installer behavior: systemctl enable --now annad (starts but doesn't restart)
```

The installer used `systemctl enable --now`, which only starts services that aren't running. During upgrades, the old daemon kept running with the old binary.

## ✅ Fix Implemented

Updated `scripts/install.sh` with three key improvements:

### 1. Upgrade Detection (lines 207-214)
Detects if daemon is already running before install:
```bash
DAEMON_WAS_RUNNING=false
if systemctl is-active --quiet annad 2>/dev/null; then
    DAEMON_WAS_RUNNING=true
    OLD_VERSION=$(annactl --version | awk '{print $2}')
    echo "→ Detected running daemon (v${OLD_VERSION})"
    echo "→ Will restart after installing new binaries"
fi
```

### 2. Smart Restart Logic (lines 299-317)
Branches on upgrade vs fresh install:
```bash
if [ "$DAEMON_WAS_RUNNING" = true ]; then
    # UPGRADE: Force restart to load new binaries
    sudo systemctl restart annad
else
    # FRESH INSTALL: Start for first time
    sudo systemctl enable --now annad
fi
```

### 3. Version Validation (lines 349-357)
Confirms running version matches installed version:
```bash
if [ "$RUNNING_VERSION" = "$INSTALLED_VERSION" ]; then
    echo "✓ Version verified: $RUNNING_VERSION"
else
    echo "⚠ Version mismatch: running=$RUNNING_VERSION, installed=$INSTALLED_VERSION"
fi
```

## ✅ Documentation

Created/updated:
- `docs/DAEMON-RESTART-FIX.md` - Detailed technical analysis
- `CHANGELOG.md` - Added v0.12.6-pre entry with fix details
- This summary document

## 🔧 Manual Recovery Steps (User Action Required)

Since I don't have sudo access to restart the daemon, you'll need to run:

```bash
# 1. Restart daemon to load v0.12.6-pre binaries
sudo systemctl restart annad

# 2. Wait for initialization
sleep 3

# 3. Verify versions align
echo "Installed binary:"
/usr/local/bin/annactl --version

echo "Running daemon:"
annactl --version

# 4. Test RPC works
timeout 5 annactl status

# 5. Test storage command (new in v0.12.6-pre)
annactl storage --help
```

### Expected Output
```
Installed binary:
annactl 0.12.6-pre

Running daemon:
annactl 0.12.6-pre

annactl status
System Status: Operational
Version: 0.12.6-pre
...

annactl storage --help
Show storage profile and health status

Usage: annactl storage [OPTIONS] [COMMAND]
...
```

## 🧪 Validation After Manual Restart

Run these to confirm everything works:

```bash
# 1. Comprehensive smoke test
./tests/verify_v0122.sh

# 2. Btrfs-specific tests
./tests/arch_btrfs_smoke.sh

# 3. Quick RPC validation
annactl status
annactl profile show
annactl storage btrfs show
```

All should return data without timeouts or "unrecognized subcommand" errors.

## 🎯 Impact Assessment

| Scenario | Before Fix | After Fix |
|----------|-----------|-----------|
| Fresh install | ✅ Works (enable --now starts) | ✅ Works (same behavior) |
| Upgrade | ❌ Old daemon keeps running | ✅ Daemon restarted |
| Version check | ❌ No validation | ✅ Explicit verification |
| Error clarity | ❌ Silent mismatch | ✅ Clear warning if mismatch |

## 🚀 Roadmap: v0.12.6 → v0.12.7

With daemon restart reliability fixed, proceed with:

### 1. Enhanced Health Checks (Priority: High)
- **Goal**: Comprehensive daemon self-diagnostics
- **Tasks**:
  - RPC latency monitoring
  - Socket file validation
  - Memory/CPU usage tracking
  - Event queue depth metrics
- **Deliverable**: `annactl doctor check` shows daemon health

### 2. Dynamic Reload (Priority: Medium)
- **Goal**: Reload config without full restart
- **Tasks**:
  - SIGHUP handler in daemon
  - Config file change detection
  - Graceful module enable/disable
- **Deliverable**: `annactl reload` without downtime

### 3. Storage Profile Enhancements (Priority: Medium)
- **Goal**: Richer Btrfs insights
- **Tasks**:
  - Subvolume tree visualization
  - Snapshot diff preview
  - Balance status tracking
- **Deliverable**: `annactl storage btrfs tree`

### 4. RPC Error Handling (Priority: Low)
- **Goal**: Better error messages
- **Tasks**:
  - Structured error codes
  - Timeout categorization (socket missing vs slow response)
  - Retry logic with exponential backoff
- **Deliverable**: Clearer error messages in CLI

## 📋 Pre-Commit Checklist for v0.12.6

Before tagging v0.12.6 (after manual daemon restart):

- [ ] Daemon restarted successfully
- [ ] Versions aligned (annactl --version == annactl status output)
- [ ] RPC tests pass (`./tests/verify_v0122.sh`)
- [ ] Storage commands work (`annactl storage --help`, `btrfs show`)
- [ ] No timeout errors in normal operations
- [ ] Smoke tests pass (10/10 checks)

## 📊 Metrics

**Code Changes**:
- Files modified: 1 (`scripts/install.sh`)
- Lines added: ~40
- Lines removed: ~10
- Net change: +30 lines

**Testing**:
- Manual validation required (sudo restart)
- Automated tests ready (smoke tests)
- Regression risk: Low (only affects installer, not runtime)

**Documentation**:
- New docs: 2 (`DAEMON-RESTART-FIX.md`, `V0126-COMPLETION-SUMMARY.md`)
- Updated docs: 1 (`CHANGELOG.md`)
- Total doc lines: ~300

## 🔗 Related Files

- **Fixed**: `scripts/install.sh`
- **Docs**: `docs/DAEMON-RESTART-FIX.md`, `CHANGELOG.md`
- **Tests**: `tests/verify_v0122.sh`, `tests/arch_btrfs_smoke.sh`
- **Config**: `/etc/systemd/system/annad.service` (no changes needed)

## 🎓 Lessons Learned

1. **systemctl quirks**: `enable --now` ≠ restart
2. **Upgrade detection**: Always check if service is already running
3. **Version validation**: Don't assume binaries match running process
4. **Explicit feedback**: Show old → new version during upgrades

---

**Status**: Fix implemented, pending manual daemon restart for validation
**Next Action**: User runs `sudo systemctl restart annad` and validates
**Then**: Proceed to v0.12.7 milestone (health checks, dynamic reload)
**Timeline**: v0.12.6 release candidate ready after validation

**Completed**: 2025-11-02
**Author**: Claude Code (via jjgarcianorway)
