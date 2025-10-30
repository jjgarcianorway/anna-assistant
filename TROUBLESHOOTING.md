# Troubleshooting Anna v0.9.6-alpha.6

## Current System State

This system has Anna v0.9.6-alpha.6 installed, but the daemon is not running correctly.

### Diagnosis

```bash
systemctl is-active annad
# Output: activating
```

The daemon is stuck in "activating" state, repeatedly trying to start and failing.

### Root Cause

The systemd service file at `/etc/systemd/system/annad.service` is missing two critical settings:

1. **Missing `User=root`** - The daemon requires root permissions but the service doesn't specify to run as root
2. **Missing `/var/lib/anna` in `ReadWritePaths`** - The telemetry database needs write access to `/var/lib/anna`

### Verification

Check the installed service file:
```bash
cat /etc/systemd/system/annad.service
```

You should see `User=root` and `Group=anna` in the `[Service]` section.
You should see `/var/lib/anna` in the `ReadWritePaths=` line.

If those are missing, the service file needs to be updated.

## Solution 1: Update Service File (Quick Fix)

Run the provided helper script:

```bash
sudo ./scripts/update_service_file.sh
```

This will:
1. Stop the daemon
2. Copy the corrected service file
3. Reload systemd
4. Start the daemon
5. Verify it's running

## Solution 2: Re-run Installer (Recommended)

The full installer will update everything:

```bash
./scripts/install.sh
```

This will:
- Detect the existing installation
- Prompt for upgrade confirmation
- Create a backup
- Update all files including the service file
- Restart the daemon
- Verify the installation

## Solution 3: Manual Fix

If you prefer to fix it manually:

1. Stop the daemon:
```bash
sudo systemctl stop annad
```

2. Edit the service file:
```bash
sudo nano /etc/systemd/system/annad.service
```

3. In the `[Service]` section, add after `Type=simple`:
```
User=root
Group=anna
```

4. In the `ReadWritePaths=` line, change:
```
ReadWritePaths=/etc/anna /run /var/log/anna
```
to:
```
ReadWritePaths=/etc/anna /run /var/lib/anna /var/log/anna
```

5. Reload and restart:
```bash
sudo systemctl daemon-reload
sudo systemctl start annad
```

6. Verify:
```bash
systemctl is-active annad  # Should show: active
annactl status             # Should connect successfully
```

## Verification After Fix

Once the daemon is running, run:

```bash
./scripts/verify_installation.sh
```

This will check:
- All files are in place
- Service is running
- Socket exists with correct permissions
- Commands work

## Common Issues

### Issue: "annad must run as root"

**Symptom**: Logs show `[FATAL] annad must run as root`

**Solution**: The service file needs `User=root` in the `[Service]` section.

### Issue: "Failed to bind socket"

**Symptom**: Logs show socket binding error

**Solution**:
- Check `/run/anna` exists and has correct permissions (0755 root:anna)
- Old socket file may exist: `sudo rm /run/anna/annad.sock`
- Restart: `sudo systemctl restart annad`

### Issue: "Permission denied" writing to /var/lib/anna

**Symptom**: Logs show permission errors for telemetry database

**Solution**: Add `/var/lib/anna` to `ReadWritePaths` in service file

### Issue: Socket exists but annactl can't connect

**Symptom**: `annactl ping` times out or fails

**Solution**:
- Check socket permissions: `ls -l /run/anna/annad.sock`
- Should be: `srw-rw---- root anna`
- Verify you're in the anna group: `groups | grep anna`
- If not: `sudo usermod -aG anna $USER` then log out/in

### Issue: Group membership doesn't work

**Symptom**: Can't access socket even after being added to anna group

**Solution**: Group changes require logging out and back in. Run:
```bash
su - $USER  # Quick way to get new group membership
# OR
logout/login
```

## Checking Daemon Logs

View recent logs:
```bash
journalctl -u annad -n 50
```

Follow logs in real-time:
```bash
journalctl -u annad -f
```

Look for:
- `[BOOT] Anna Assistant Daemon v0.9.6-alpha.6 starting...`
- `[READY] anna-assistant operational`

If you see:
- `[FATAL] annad must run as root` - Service file needs `User=root`
- `Failed to bind socket` - Socket/directory permissions issue
- `Permission denied` - ReadWritePaths issue

## Working Configuration

Here's what a working service file should look like:

```ini
[Unit]
Description=Anna Assistant Daemon
Documentation=https://github.com/anna-assistant/anna
After=network.target

[Service]
Type=simple
User=root
Group=anna
ExecStart=/usr/local/bin/annad
Restart=on-failure
RestartSec=5s

# Security hardening
PrivateTmp=yes
NoNewPrivileges=false
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/etc/anna /run /var/lib/anna /var/log/anna

# Resource limits
MemoryMax=512M
TasksMax=100

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=annad

[Install]
WantedBy=multi-user.target
```

## Expected Directory Structure

After successful installation:

```
/usr/local/bin/
├── annad               (755 root:root)
└── annactl             (755 root:root)

/etc/anna/
├── config.toml         (644 root:anna)
├── version             (644 root:anna)
├── policies.d/
│   └── 00-bootstrap.yaml
└── personas.d/

/var/lib/anna/          (750 root:anna)
├── telemetry.db        (created by daemon)
└── backups/

/var/log/anna/          (750 root:anna)

/run/anna/              (755 root:anna)
└── annad.sock          (660 root:anna)
```

## CPU Usage Issues

If the daemon is using high CPU:

1. Check the logs for watchdog warnings:
```bash
journalctl -u annad | grep WATCHDOG
```

2. The daemon should use < 1% CPU when idle. If higher:
   - Telemetry collector runs every 60s (configurable via env)
   - CPU watchdog checks every 5 minutes
   - Policy engine evaluates on events only

3. Reduce telemetry frequency temporarily:
```bash
sudo systemctl stop annad
sudo systemctl edit annad  # Add: Environment="ANNA_TELEM_INTERVAL=120"
sudo systemctl start annad
```

## Getting Help

1. Run diagnostics:
```bash
annactl doctor check
```

2. Try automatic repair:
```bash
annactl doctor repair
```

3. Check this file for common issues (you're reading it!)

4. Check logs:
```bash
journalctl -u annad -n 100 --no-pager
```

5. Verify files and permissions:
```bash
./scripts/verify_installation.sh
```

## Uninstalling (if needed)

To completely remove Anna:

```bash
sudo systemctl stop annad
sudo systemctl disable annad
sudo rm /usr/local/bin/{annad,annactl}
sudo rm /etc/systemd/system/annad.service
sudo systemctl daemon-reload
sudo rm -rf /etc/anna /var/lib/anna /var/log/anna /run/anna
```

To remove the group:
```bash
sudo gpasswd -d $USER anna
sudo groupdel anna
```

Then reinstall from scratch if desired.
