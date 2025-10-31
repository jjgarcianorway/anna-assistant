# Anna v0.11.0 Troubleshooting Guide

Quick "if X then Y" reference for common issues.

## If: Socket missing (`/run/anna/annad.sock` not found)

```bash
# Check daemon is running
sudo systemctl status annad

# If not running, start it
sudo systemctl start annad

# Wait 5 seconds, then check again
sleep 5 && test -S /run/anna/annad.sock && echo "OK" || echo "STILL MISSING"

# If still missing, check logs
sudo journalctl -u annad -n 30

# Run repair
sudo bash scripts/fix_v011_installation.sh
```

## If: "attempt to write a readonly database" in logs

```bash
# Fix directory ownership
sudo chown -R anna:anna /var/lib/anna /var/log/anna
sudo chmod 0750 /var/lib/anna /var/log/anna

# Restart daemon
sudo systemctl restart annad

# Verify
sudo journalctl -u annad -n 10 | grep -i readonly
```

## If: `annactl status` fails with "Failed to connect"

```bash
# Check socket exists
ls -la /run/anna/annad.sock

# Check daemon is active
systemctl is-active annad

# Check you're in anna group
groups | grep anna

# If not in group, add yourself
sudo usermod -aG anna $USER
newgrp anna  # Or log out/in

# Run repair
annactl doctor repair --yes
```

## If: Service fails to start

```bash
# Check for detailed error
sudo systemctl status annad --no-pager -l

# Check journal
sudo journalctl -u annad -n 50 --no-pager

# Common fixes:
sudo install -d -m 0750 -o anna -g anna /var/lib/anna
sudo install -d -m 0750 -o anna -g anna /var/log/anna

# Restart
sudo systemctl daemon-reload
sudo systemctl restart annad
```

## If: CAPABILITIES.toml missing warning

```bash
# Install from repo
sudo install -m 0644 etc/CAPABILITIES.toml /usr/lib/anna/CAPABILITIES.toml

# Or use repair
sudo bash scripts/fix_v011_installation.sh
```

## If: Permission denied errors

```bash
# Check effective user
ps aux | grep annad | grep -v grep

# Should show "anna" as user, not your username

# Fix systemd unit if needed
grep "User=anna" /etc/systemd/system/annad.service

# If missing, reinstall
./scripts/install.sh
```

## If: Database grows too large

```bash
# Check size
du -h /var/lib/anna/telemetry.db

# Vacuum database (CAREFUL)
sudo systemctl stop annad
sudo -u anna sqlite3 /var/lib/anna/telemetry.db "VACUUM;"
sudo systemctl start annad
```

## If: Need to completely reset

```bash
# Stop and disable
sudo systemctl stop annad
sudo systemctl disable annad

# Remove all state
sudo rm -rf /var/lib/anna /var/log/anna /run/anna
sudo rm -rf /etc/anna /usr/lib/anna

# Reinstall
./scripts/install.sh
```

## If: Want to see what's wrong

```bash
# Run comprehensive diagnostics
bash scripts/anna-diagnostics.sh --output /tmp/diag.txt

# Or generate report tarball
annactl doctor report

# Check doctor
annactl doctor pre
annactl doctor post
```

## If: Tests failing

```bash
# Run smoke test
./tests/smoke_v011.sh

# Run annactl matrix
./tests/annactl_matrix.sh

# Run full CI suite
./scripts/ci_smoke.sh
```

## If: Socket appears then disappears

```bash
# Daemon is crashing during startup
# Check for panic or fatal error
sudo journalctl -u annad -n 100 | grep -E "panic|fatal|FATAL"

# Common cause: DB write failure
# Fix permissions
sudo chown -R anna:anna /var/lib/anna
sudo chmod 0750 /var/lib/anna

# Restart and watch logs
sudo systemctl restart annad
sudo journalctl -u annad -f
```

## If: Want detailed startup logs

```bash
# Watch journal in real-time
sudo journalctl -u annad -f

# Look for:
# - "Running as uid=XXX, gid=YYY" (should be anna user)
# - "RPC socket ready: /run/anna/annad.sock"
# - "Capabilities: X active, Y degraded"
```

## Quick Commands Reference

| Command | Purpose |
|---------|---------|
| `annactl doctor pre` | Check prerequisites |
| `annactl doctor post` | Verify installation |
| `annactl doctor repair --yes` | Fix installation |
| `annactl doctor report` | Generate diagnostic tarball |
| `sudo bash scripts/fix_v011_installation.sh` | Emergency repair |
| `bash scripts/anna-diagnostics.sh` | Full diagnostics |
| `./tests/smoke_v011.sh` | Basic functionality test |
| `sudo systemctl status annad` | Service status |
| `sudo journalctl -u annad -n 50` | Recent logs |
| `test -S /run/anna/annad.sock` | Check socket exists |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General failure |
| 10 | Preflight check failed |
| 11 | Postflight check failed |
| 12 | Postflight passed with warnings |

## Getting Help

1. Run diagnostics: `annactl doctor report`
2. Check: `docs/V0.11.0_INSTALLATION_FIXES.md`
3. Review: `QUICK_FIX_INSTRUCTIONS.md`
4. Report issue with diagnostic tarball
