# Anna v0.9.6-alpha.7 - Ready for Testing

**Date**: October 30, 2025
**Status**: ✅ Complete - Ready for installation testing

---

## What This Release Fixes

### The Big Change: No More Root

Anna now runs as a dedicated `anna` system user instead of root. This is:
- **More secure** - limited privileges
- **More stable** - can't break the system
- **More correct** - follows Linux best practices

### What Works Right Now

**Build & Offline Commands** ✅ (100% Tested)
```bash
cargo build --release     # ✓ Clean build
annactl --version         # ✓ 0.9.6-alpha.7
annactl profile show      # ✓ Beautiful output
annactl profile checks    # ✓ 11 health checks
annactl persona list      # ✓ 4 personas
annactl doctor check      # ✓ 9 diagnostics
```

**Installer** ✅ (Syntax Validated)
```bash
./scripts/install.sh      # ✓ Syntax valid, ready to run
```

---

## How to Install

**Simple**: Just run the installer as your regular user:

```bash
./scripts/install.sh
```

It will:
1. Build the binaries
2. Ask permission to elevate
3. Create anna system user
4. Install everything
5. Start the daemon
6. Verify it's working

**Time**: ~2 minutes (depends on build speed)

---

## What the Installer Does

With your permission, it will:

### Creates
- ✓ System user `anna` (no home directory, no login)
- ✓ Directories: `/etc/anna`, `/run/anna`, `/var/lib/anna`, `/var/log/anna`
- ✓ Binaries: `/usr/local/bin/annad`, `/usr/local/bin/annactl`
- ✓ Service: `/etc/systemd/system/annad.service`
- ✓ Config: `/etc/anna/config.toml` (with friendly banner)
- ✓ Policies: `/etc/anna/policies.d/00-bootstrap.yaml`

### Sets
- ✓ Directories owned by `anna:anna`
- ✓ Permissions: 0750 on data directories
- ✓ Socket: 0660 (readable/writable by anna group)
- ✓ Current user added to `anna` group

---

## After Installation

### Verify It Worked

```bash
systemctl is-active annad
# Expected: active

ls -l /run/anna/annad.sock
# Expected: srw-rw---- anna anna

annactl status
# Expected: Shows version, socket, status

annactl profile show
# Expected: Beautiful system profile
```

### Note

You'll need to **log out and back in** for group membership to take effect.

Or use: `su - $USER` as a quick workaround.

---

## What's Different from alpha.6

| Feature | alpha.6 | alpha.7 |
|---------|---------|---------|
| Daemon user | root | anna (system user) |
| Directory creation | Manual in code | systemd RuntimeDirectory |
| Resource limits | MemoryMax only | CPU + Memory + Watchdog |
| Root check | Required | Removed |
| Security | Runs as root ⚠️ | Principle of least privilege ✓ |
| Stability | No CPU limits | CPUQuota=50%, WatchdogSec=60s |

---

## Technical Details

### Systemd Service

```ini
[Service]
User=anna
Group=anna
RuntimeDirectory=anna         # systemd creates /run/anna
StateDirectory=anna           # systemd creates /var/lib/anna
CPUQuota=50%                  # Max 50% CPU
MemoryMax=300M                # Max 300MB RAM
WatchdogSec=60s               # Auto-restart if hung
Restart=on-failure
RestartSec=3s
```

### Directories

```
/run/anna/              # Runtime (socket, PID)
  ├── annad.sock        # Unix socket (0660 anna:anna)

/var/lib/anna/          # Persistent data
  ├── telemetry.db      # Metrics database
  └── backups/          # Installation backups

/var/log/anna/          # Logs (future)

/etc/anna/              # Configuration
  ├── config.toml       # Main config
  ├── policies.d/       # Policy rules
  └── personas.d/       # Communication styles
```

### Config Governance Banner

All managed files now start with:
```
# Hi, I'm Anna! Please use 'annactl config set' to change settings.
# Don't edit me by hand - I track who changed what and why.
```

---

## Migration from alpha.6

If you have alpha.6 installed:

```bash
# Stop old daemon (runs as root)
sudo systemctl stop annad

# Run new installer
./scripts/install.sh

# It will:
# - Create anna user
# - Update service file
# - Fix ownership
# - Restart as anna user
```

---

## Files in This Release

**Modified (5)**:
- `Cargo.toml` - Version 0.9.6-alpha.7
- `etc/systemd/annad.service` - User=anna, resource limits
- `scripts/install.sh` - Creates anna user
- `src/annad/src/main.rs` - Removed root requirement
- `CHANGELOG.md` - Complete alpha.7 entry

**Created (2)**:
- `etc/tmpfiles.d/anna.conf` - Runtime directory
- `ALPHA7_SUMMARY.md` - This file

---

## Test Checklist

After running `./scripts/install.sh`:

- [ ] Installer completed without errors
- [ ] `systemctl is-active annad` returns "active"
- [ ] `/run/anna/annad.sock` exists with correct permissions
- [ ] `annactl ping` succeeds
- [ ] `annactl status` shows correct version
- [ ] `annactl profile show` displays system info
- [ ] Daemon uses < 2% CPU when idle
- [ ] No errors in `journalctl -u annad -n 50`

---

## Known Limitations

### Must Do Manually
- Log out/in after install (for group membership)

### Not Yet Implemented
- Firmware/driver deep diagnostics
- Log rotation
- Some advanced commands (ask with AI, explore)
- Automatic updates

### Works Offline
All these commands work without the daemon:
- `annactl profile show`
- `annactl profile checks`
- `annactl persona list|get`
- `annactl config list`
- `annactl doctor check`

---

## If Something Goes Wrong

### Daemon won't start

```bash
# Check logs
journalctl -u annad -n 50

# Common fixes:
sudo systemctl stop annad
sudo rm /run/anna/annad.sock
sudo systemctl start annad
```

### Can't connect to socket

```bash
# Check permissions
ls -l /run/anna/annad.sock

# Verify you're in anna group
groups | grep anna

# If not, add yourself:
sudo usermod -aG anna $USER
# Then log out and back in
```

### Want to uninstall

```bash
sudo systemctl stop annad
sudo systemctl disable annad
sudo rm /usr/local/bin/{annad,annactl}
sudo rm /etc/systemd/system/annad.service
sudo systemctl daemon-reload
sudo rm -rf /etc/anna /var/lib/anna /run/anna
sudo userdel anna
sudo groupdel anna
```

---

## Next Steps

1. **Run the installer**: `./scripts/install.sh`
2. **Test it works**: `annactl status`
3. **Try commands**: See above checklist
4. **Report issues**: Check logs if anything fails

---

## Commit

```
commit d13886d
Author: Anna Assistant Team
Date:   Thu Oct 30 2025

    v0.9.6-alpha.7: Run as anna user, not root
```

---

**Status**: ✅ Ready for installation and testing

**Recommendation**: Install on a test VM first, then on your main system.

**Safety**: All privileged operations ask permission first.

---

*Anna is ready to serve - as a proper system user!*
