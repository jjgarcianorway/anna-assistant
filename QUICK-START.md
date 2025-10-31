# Anna Quick Start

## For Your Current Installation (Fix the Daemon)

Your Anna installation has a missing file. Run this to fix it:

```bash
cd /home/lhoqvso/anna-assistant
./scripts/fix-capabilities.sh
```

Then verify it's working:

```bash
./tests/diagnostics.sh
```

You should see the socket at `/run/anna/annad.sock` and `annactl status` should work!

---

## Understanding the Problem

The daemon was crashing on startup:
- **Missing:** `/usr/lib/anna/CAPABILITIES.toml`
- **Result:** Daemon failed to start, no socket created
- **Fixed in:** Installer now includes this file (will be in v0.13.7)

---

## What Anna Can Do Now

Once the fix is applied, test these commands:

### System Health:
```bash
annactl status          # Daemon status and health
annactl sensors         # CPU, memory, temperatures, battery
annactl disk            # Disk usage and SMART status
annactl net             # Network interfaces and connectivity
annactl top             # Top processes by CPU and memory
```

### System Events:
```bash
annactl events          # Recent system events
annactl watch           # Watch live events (Ctrl+C to exit)
```

### Diagnostics:
```bash
annactl doctor pre      # Preflight checks (works offline)
annactl doctor post     # Postflight checks
annactl doctor repair   # Repair installation issues
annactl doctor report   # Generate diagnostic tarball
```

### Capabilities:
```bash
annactl capabilities    # Show module capabilities
annactl alerts          # Show integrity alerts
```

---

## Development Workflow (Creating v0.13.7)

Once you confirm the fix works, create the next release:

```bash
# The fix is already committed
# Create release with the fix
./scripts/release.sh -t patch --yes

# Wait ~3 minutes for GitHub Actions to build

# Users can then install with:
./scripts/install.sh  # Will get working v0.13.7
```

---

## Files Changed

### Added:
- `scripts/fix-capabilities.sh` - Quick fix for existing installations
- `tests/diagnostics.sh` - Modern health check tool
- `FIX-DAEMON-SOCKET.md` - Detailed troubleshooting guide

### Modified:
- `scripts/install.sh` - Now installs CAPABILITIES.toml

### Removed (11 outdated test scripts, 3,923 lines):
- All version-specific test scripts from v0.9-v0.11
- All cargo-based test scripts
- Replaced with one modern diagnostics tool

---

## Next Steps

1. **Fix your installation:**
   ```bash
   ./scripts/fix-capabilities.sh
   ```

2. **Verify it works:**
   ```bash
   ./tests/diagnostics.sh
   annactl status
   annactl sensors
   ```

3. **Create v0.13.7 release:**
   ```bash
   ./scripts/release.sh -t patch --yes
   ```

4. **Test fresh installation** (after release builds):
   ```bash
   # In a clean directory
   git clone https://github.com/jjgarcianorway/anna-assistant.git
   cd anna-assistant
   ./scripts/install.sh
   # Should work perfectly!
   ```

---

## Expected Output from diagnostics.sh

After running the fix, you should see:

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

## Summary

- ✅ Made repository public (fixed binary downloads)
- ✅ Created releases v0.13.0 through v0.13.6
- ✅ Simplified workflow to 3 essential scripts
- ✅ Diagnosed daemon crash (missing CAPABILITIES.toml)
- ✅ Fixed installer to include the file
- ✅ Created fix script for existing installations
- ✅ Cleaned up 11 outdated test scripts
- ✅ Created modern diagnostics tool
- 🔜 Ready to release v0.13.7 (first fully working release!)

**Run `./scripts/fix-capabilities.sh` now to get Anna running!** 🚀
