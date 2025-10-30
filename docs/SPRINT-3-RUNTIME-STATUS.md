# Sprint 3 Runtime Validation - Implementation Status

**Date:** 2025-10-30
**Version:** v0.9.2a (in progress)
**Status:** Systemd integration partially complete

## ✅ Completed Components

### 1. Systemd Service Files
- ✅ `packaging/arch/annad.service` - Complete systemd unit with RuntimeDirectory
- ✅ `packaging/arch/annad.tmpfiles.conf` - Tmpfiles configuration for /run/anna

### 2. Updated Installation Script
- ✅ `scripts/install.sh` - Comprehensive installer with:
  - Anna group creation and management
  - User group membership handling
  - Proper directory permissions (0750 root:anna)
  - Systemd service and tmpfiles installation
  - Post-install validation (socket check, annactl ping/status)
  - Group context handling (sg anna workaround)
  - Example policy installation

### 3. Updated Uninstall Script
- ✅ `scripts/uninstall.sh` - Safe removal with:
  - Service stop and disable
  - Comprehensive backup to ~/Documents/anna_backup_<timestamp>/
  - README-RESTORE.md with restore instructions
  - Group cleanup (only if no members)
  - Tmpfiles cleanup

## ⚠️ Remaining Work

### 4. Daemon Initialization Hardening
**File:** `src/annad/src/main.rs`

**Required Changes:**
```rust
// Add after root check:
- Ensure /run/anna exists with 0770 root:anna if not created by systemd
- Set socket permissions to 0660 root:anna after creation
- Add startup sequence logging:
  - "Configuration loaded"
  - "Persistence initialized"
  - "RPC server online"
  - "Telemetry/Policy/Events ready"
- Exit with explicit error on any init failure
```

### 5. annactl UX Improvements
**File:** `src/annactl/src/main.rs`

**Required Changes:**
```rust
// In send_request() error handling:
if socket connection fails:
  println!("annad not running");
  println!("Check status: sudo systemctl status annad");
  println!("View logs: sudo journalctl -u annad --since -5m | tail -50");
```

### 6. Runtime Validation Test Script
**File:** `tests/runtime_validation.sh` (new)

**Required Content:**
```bash
#!/usr/bin/env bash
# Full end-to-end runtime validation
# - Builds and installs via install.sh
# - Verifies systemctl is-active annad
# - Verifies socket exists with correct permissions
# - Tests annactl ping, status, config list, telemetry stats
# - Tests policy/events/learning commands
```

### 7. QA Runner Integration
**File:** `tests/qa_runner.sh`

**Required Changes:**
```bash
# Add final stage:
if [[ $EUID -eq 0 ]] || command -v sudo &>/dev/null; then
  test_runtime_validation() {
    # Run runtime_validation.sh
  }
else
  test_skip "Runtime validation" "Requires sudo access"
fi
```

### 8. Documentation Updates
**Files to Update:**
- `docs/RUNTIME-VALIDATION-Sprint3.md` - Add actual test commands and expected outputs
- `docs/QA-RESULTS-Sprint3.md` - Add runtime validation section with:
  - systemctl status annad output
  - journalctl -u annad --since -5m output
  - Socket permissions verification
  - annactl command tests

### 9. CHANGELOG Update
**File:** `CHANGELOG.md`

**Add v0.9.2a section:**
```markdown
## [0.9.2a] - Sprint 3 Runtime Validation - 2025-10-30

### Added
- Systemd service with RuntimeDirectory management
- Tmpfiles configuration for /run/anna
- Anna group for socket access control
- Comprehensive install/uninstall scripts
- Runtime validation test suite

### Changed
- Socket permissions now 0660 root:anna
- Daemon logs startup sequence
- annactl provides troubleshooting hints on connection failure

### Fixed
- Socket creation and permissions handling
- Group membership validation
- Service startup reliability
```

## Testing Requirements

### Environment Needed
- Fresh Arch Linux VM or system
- sudo/root access
- Rust toolchain installed

### Test Sequence
```bash
# 1. Clone and navigate
cd anna-assistant

# 2. Run installer
sudo ./scripts/install.sh

# Expected: All steps pass, service starts, validation succeeds

# 3. Verify service
sudo systemctl status annad
# Expected: active (running)

# 4. Check socket
ls -lh /run/anna/annad.sock
# Expected: srwxrw---- 1 root anna 0 ... /run/anna/annad.sock

# 5. Test commands
annactl status
annactl ping
annactl config list
annactl policy list
annactl events show --limit 5
annactl learning stats

# Expected: All commands work without errors

# 6. Check logs
sudo journalctl -u annad --since -5m
# Expected: Clean startup logs, no errors

# 7. Run full QA
bash tests/qa_runner.sh
# Expected: 134+ tests pass (including runtime stage)
```

## Success Criteria

Sprint 3 can be sealed as complete when:

1. ✅ Fresh install on Arch → `annactl status` works immediately
2. ✅ `/run/anna/annad.sock` exists with 0660 root:anna
3. ✅ `systemctl is-active annad` returns active
4. ✅ Full QA including runtime stage passes
5. ✅ No regressions in unit tests (134/134 pass)
6. ✅ Documentation includes captured runtime output

## Current Blockers

**Environmental:** Current development environment lacks sudo/root access for testing.

**Workaround:** Code changes can be made now, but testing requires deployment to VM/system with root access.

## Estimated Remaining Work

- **Code Changes:** 1-2 hours
  - Daemon hardening: 30 min
  - annactl UX: 15 min
  - Runtime validation script: 30 min
  - QA integration: 15 min

- **Testing:** 1 hour
  - VM setup: 20 min
  - Install and validate: 20 min
  - Documentation: 20 min

- **Total:** 2-3 hours to completion

## Next Steps

1. Complete daemon hardening in src/annad/src/main.rs
2. Add annactl UX improvements
3. Create tests/runtime_validation.sh
4. Update tests/qa_runner.sh
5. Deploy to VM with sudo
6. Execute full test sequence
7. Capture outputs for documentation
8. Update CHANGELOG to v0.9.2a
9. Commit and seal Sprint 3

---

**Status:** Infrastructure complete, runtime testing pending privileged environment
