# Anna v0.12.0 - Troubleshooting Guide

Common issues and solutions for Anna Assistant.

---

## Quick Diagnosis

Run the doctor commands first:

```bash
# Check if installation is healthy
annactl doctor post

# Attempt automatic repair
sudo annactl doctor repair --yes
```

---

## Issue: `annactl` commands hang or timeout

**Symptoms:**
- Commands like `annactl status` hang indefinitely
- Error: "Connection timeout (2s) - is the daemon running?"

**Root Cause:**
Daemon version mismatch or RPC server not accepting connections.

**Solution:**

1. **Check versions:**
```bash
annactl --version
systemctl status annad | head -n 20 | grep "daemon starting"
```

If versions don't match (e.g., annactl v0.13.6 but daemon v0.11.0), you have a version mismatch.

2. **Check socket existence:**
```bash
ls -la /run/anna/annad.sock
stat -c 'Owner: %U:%G  Mode: %a' /run/anna/annad.sock
```

Socket should exist as `anna:anna 0770`.

3. **Check daemon logs:**
```bash
sudo journalctl -u annad --since "10 minutes ago" --no-pager
```

Look for "RPC socket ready: /run/anna/annad.sock" line. If missing, daemon failed to start RPC server.

4. **Restart daemon:**
```bash
sudo systemctl restart annad
sleep 2
sudo journalctl -u annad -n 50 --no-pager | grep "RPC socket ready"
```

5. **If restart fails, check permissions:**
```bash
sudo ls -lZa /run/anna
sudo ls -lZa /var/lib/anna
```

Directories should be owned by `anna:anna` with mode `0750`.

6. **Emergency fix:**
```bash
sudo systemctl stop annad
sudo rm -f /run/anna/annad.sock
sudo install -d -m 0750 -o anna -g anna /run/anna
sudo systemctl start annad
```

---

## See full CLI Reference and RADARS documentation

- [CLI_REFERENCE.md](CLI_REFERENCE.md) - Complete command documentation
- [RADARS.md](RADARS.md) - Radar scoring details
- [README.md](../README.md) - Installation and quick start
