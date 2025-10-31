# Anna Socket Issue - Root Cause Analysis & Fix

## Problem Summary
After reboot, `annactl status` fails with:
```
Error: Failed to connect to annad (socket: /run/anna/annad.sock)
Caused by: No such file or directory (os error 2)
```

## Root Causes Found

### 1. ~~Missing RuntimeDirectory (FIXED in v0.13.6)~~
- ✓ Service file had conflicting `ExecStartPre` that prevented `RuntimeDirectory=anna` from working
- ✓ Fixed in `etc/systemd/annad.service` - removed redundant `ExecStartPre`

### 2. **Missing policy.toml (CRITICAL - Fixed in this release)**
- ❌ Installer doesn't install `etc/policy.toml` to `/etc/anna/`
- ❌ Daemon crashes on startup: "Cannot initialize policy engine, using fallback"
- ❌ Socket briefly created, then removed when daemon crashes
- ❌ Daemon stuck in crash-restart loop (79+ restarts)

### 3. Weak Verification
- ❌ Installer only checks `systemctl is-active`, not actual socket connectivity
- ❌ Reports "Anna is running" even when socket doesn't exist
- ❌ Doesn't use `annactl` for verification

## Fixes Applied

### `scripts/install.sh`
1. **Added policy.toml installation** (line 249-257)
   ```bash
   if [ ! -f /etc/anna/policy.toml ] && [ -f etc/policy.toml ]; then
       sudo install -m 0644 etc/policy.toml /etc/anna/
       echo "✓ Policy config installed"
   fi
   ```

2. **Improved verification** (line 287-297)
   - Now uses `annactl status` to verify socket connectivity
   - Shows diagnostics if verification fails
   - Reports daemon state AND socket existence

### `scripts/release.sh`
3. **Reduced SSH passphrase prompts** (line 335)
   - Combined two separate git pushes into one
   - Now: `git push --atomic origin main "v$VERSION"`
   - Prompts reduced from 2-3 times to 1 time

### `etc/systemd/annad.service`
4. **Already fixed** - Removed conflicting ExecStartPre

## Testing

### Quick Test (Get Running Now)
```bash
./scripts/quick-fix-policy.sh
```
This installs just policy.toml and restarts the daemon.

### Full Test (After Release)
```bash
./scripts/release.sh -t patch -m "fix: install policy.toml, improve verification"
./scripts/install.sh
annactl status
```

## Expected Behavior After Fix

1. Daemon starts successfully (no crash loop)
2. Socket created at `/run/anna/annad.sock`
3. `annactl status` works immediately
4. Installer reports "Anna is running and responding"

## Files Changed
- `scripts/install.sh` - Added policy.toml + better verification
- `scripts/release.sh` - Atomic git push
- `etc/systemd/annad.service` - Already fixed (no ExecStartPre)
- `scripts/quick-fix-policy.sh` - New quick-fix script
