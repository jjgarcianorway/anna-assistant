# Anna Installation Troubleshooting

Quick fixes for common installation issues.

---

## Issue: "group anna exists" error

**Symptom:**
```
useradd: group 'anna' exists
```

**Cause:** The anna group was created in a previous installation attempt.

**Fix (v0.9.8-alpha.1+):**
This is now handled automatically by the installer. Just run:
```bash
./scripts/install.sh
```

**Manual fix (if needed):**
```bash
# Remove existing user/group
sudo userdel anna
sudo groupdel anna

# Run installer again
./scripts/install.sh
```

---

## Issue: Permission denied on /run/anna

**Symptom:**
```
mkdir: cannot create directory '/run/anna': Permission denied
```

**Fix:**
```bash
# Create directory with sudo
sudo mkdir -p /run/anna
sudo chown anna:anna /run/anna
sudo chmod 0750 /run/anna

# Restart daemon
sudo systemctl restart annad
```

---

## Issue: Socket not found

**Symptom:**
```
annactl: Error: Failed to connect to /run/anna/annad.sock
```

**Diagnosis:**
```bash
# Check if daemon is running
sudo systemctl status annad

# Check socket exists
ls -l /run/anna/annad.sock

# Check permissions
groups | grep anna
```

**Fixes:**

**1. Daemon not running:**
```bash
sudo systemctl start annad
sudo systemctl status annad
```

**2. Socket has wrong permissions:**
```bash
sudo systemctl restart annad
```

**3. User not in anna group:**
```bash
# Add user to group
sudo usermod -aG anna $USER

# Log out and back in (or use newgrp)
newgrp anna
```

---

## Issue: Build fails with "rand not found"

**Symptom:**
```
error: package `rand` not found
```

**Fix:**
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

If still failing:
```bash
# Update dependencies
cargo update
cargo build --release
```

---

## Issue: ASUS commands fail

**Symptom:**
```
[AUTONOMIC] Action IncreaseFan failed: "asusctl: command not found"
```

**Fix:**
```bash
# Install ASUS tools
sudo pacman -S asusctl supergfxctl

# Enable services
sudo systemctl enable --now asusd supergfxd

# Restart Anna
sudo systemctl restart annad
```

---

## Issue: Temperature sensors not working

**Symptom:**
```
cpu_temp=50.0°C (always)
```

**Diagnosis:**
```bash
# Check if sensors work
sensors

# Check if lm-sensors is installed
which sensors
```

**Fix:**
```bash
# Install lm-sensors
sudo pacman -S lm_sensors

# Detect sensors
sudo sensors-detect --auto

# Restart daemon
sudo systemctl restart annad
```

---

## Issue: Fan control not working

**Symptom:**
Fans don't change speed despite temperature changes.

**Diagnosis:**
```bash
# Check adaptive log
tail -f /var/log/anna/adaptive.log

# Check if actions are being executed
journalctl -u annad | grep AUTONOMIC

# Check current profile
asusctl profile -p
```

**Fixes:**

**1. ASUS laptop - daemon not running:**
```bash
sudo systemctl start asusd
sudo systemctl restart annad
```

**2. Generic laptop - fancontrol not configured:**
```bash
sudo sensors-detect --auto
sudo pwmconfig
sudo systemctl enable --now fancontrol
```

**3. Actions being debounced:**
```
This is normal! Actions won't repeat within 60 seconds.
Wait a minute and check again.
```

---

## Issue: High CPU usage

**Symptom:**
annad using > 3% CPU constantly.

**Diagnosis:**
```bash
# Check what's running
top | grep anna

# Check logs for errors
journalctl -u annad -n 100 | grep -i error

# Check sensor collection rate
journalctl -u annad | grep SENSORS
```

**Fix:**
```bash
# Restart daemon
sudo systemctl restart annad

# If persists, check for sensor issues
sensors
```

---

## Issue: Installer hangs

**Symptom:**
Installer stops at "Building binaries..."

**Cause:** Large Rust compile taking time.

**Fix:**
```bash
# Wait (first build can take 5-10 minutes)
# Or cancel and build manually:
Ctrl+C

# Build manually with progress
cargo build --release
# Then run installer
./scripts/install.sh
```

---

## Issue: Doctor validate fails

**Symptom:**
```
annactl doctor validate
✗ Service Status  │ active  │ inactive
```

**Fix:**
```bash
# Check why service failed
sudo systemctl status annad
journalctl -u annad -n 50

# Common fixes:
sudo systemctl restart annad
sudo systemctl enable annad
```

---

## Issue: Adaptive log not being written

**Symptom:**
```
/var/log/anna/adaptive.log: No such file
```

**Causes:**
1. Daemon hasn't made first decision yet (wait 2 minutes)
2. No autonomic actions needed yet (temperature optimal)
3. Permission issue

**Fix:**
```bash
# Create log directory manually
sudo mkdir -p /var/log/anna
sudo chown anna:anna /var/log/anna
sudo chmod 0750 /var/log/anna

# Restart daemon
sudo systemctl restart annad

# Wait 2 minutes, then check
tail /var/log/anna/adaptive.log
```

---

## Issue: Wrong fan profile applied

**Symptom:**
Fans loud even at low temperatures.

**Diagnosis:**
```bash
# Check current state
cat /run/anna/state.json

# Check recent actions
tail -20 /var/log/anna/adaptive.log

# Check ASUS profile
asusctl profile -p
```

**Fix:**
```bash
# Manually set quiet mode
asusctl profile -P quiet

# Check if autonomic loop is working
journalctl -u annad | grep AUTONOMIC

# If loop is working, wait for next iteration (60s)
```

---

## Complete Reinstall

If all else fails:

```bash
# Stop and disable services
sudo systemctl stop annad anna-fans
sudo systemctl disable annad anna-fans

# Remove binaries
sudo rm /usr/local/bin/{annad,annactl}

# Remove config
sudo rm -rf /etc/anna

# Remove runtime data
sudo rm -rf /run/anna

# Remove persistent data
sudo rm -rf /var/lib/anna

# Remove user
sudo userdel anna
sudo groupdel anna 2>/dev/null || true

# Clean build
cd anna-assistant
cargo clean

# Reinstall
cargo build --release
./scripts/install.sh
```

---

## Getting Help

**Check logs:**
```bash
# Daemon logs
journalctl -u annad -n 100

# Adaptive actions
tail -f /var/log/anna/adaptive.log

# System logs
dmesg | grep -i anna
```

**Validate system:**
```bash
annactl doctor validate
annactl doctor check
annactl profile checks
```

**File an issue:**
Include:
- Output of `annactl --version`
- Output of `journalctl -u annad -n 50`
- Content of `/var/log/anna/adaptive.log`
- Output of `sensors`
- Hardware info (ASUS model or generic)

---

**Most issues are fixed by:**
1. `sudo systemctl restart annad`
2. Waiting 2 minutes for autonomous actions
3. Checking you're in the anna group: `groups | grep anna`
