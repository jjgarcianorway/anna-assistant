//! Built-in systemd recipes (v0.0.233).

use super::types::{SystemdFeature, SystemdRecipe};

/// Get built-in systemd recipes
pub fn builtin_recipes() -> Vec<SystemdRecipe> {
    vec![
        // Create a basic service
        SystemdRecipe::new(
            SystemdFeature::CreateService,
            "Create a systemd service unit",
        )
        .with_template(
            r#"[Unit]
Description=My Service
After=network.target

[Service]
Type=simple
ExecStart=/path/to/your/program
Restart=on-failure
RestartSec=5
User=myuser

[Install]
WantedBy=multi-user.target"#,
        )
        .with_command("sudo systemctl daemon-reload")
        .with_command("sudo systemctl enable myservice.service")
        .with_command("sudo systemctl start myservice.service")
        .with_answer(
            r#"To create a systemd service:

1. **Create the unit file** at `/etc/systemd/system/myservice.service`:

```ini
[Unit]
Description=My Service
After=network.target

[Service]
Type=simple
ExecStart=/path/to/your/program
Restart=on-failure
RestartSec=5
User=myuser
WorkingDirectory=/path/to/working/dir

[Install]
WantedBy=multi-user.target
```

2. **Reload systemd and enable:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable myservice.service
sudo systemctl start myservice.service
```

3. **Check status:**
```bash
sudo systemctl status myservice.service
```

Common `Type` values:
- `simple` - default, process stays in foreground
- `forking` - forks to background (like traditional daemons)
- `oneshot` - runs once and exits
- `notify` - sends notification when ready"#,
        )
        .with_note("Replace myservice, paths, and user with your values"),
        // Create a timer (cron replacement)
        SystemdRecipe::new(
            SystemdFeature::CreateTimer,
            "Create a systemd timer for scheduled tasks",
        )
        .with_template(
            r#"# mytimer.timer
[Unit]
Description=Run my task periodically

[Timer]
OnCalendar=*-*-* 00:00:00
Persistent=true

[Install]
WantedBy=timers.target"#,
        )
        .with_answer(
            r#"To create a systemd timer (modern cron replacement):

1. **Create the service** at `/etc/systemd/system/mytask.service`:
```ini
[Unit]
Description=My scheduled task

[Service]
Type=oneshot
ExecStart=/path/to/your/script.sh
```

2. **Create the timer** at `/etc/systemd/system/mytask.timer`:
```ini
[Unit]
Description=Run my task periodically

[Timer]
OnCalendar=*-*-* 00:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

3. **Enable the timer:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable mytask.timer
sudo systemctl start mytask.timer
```

**OnCalendar examples:**
- `hourly` / `daily` / `weekly` / `monthly`
- `*-*-* 04:00:00` - daily at 4 AM
- `Mon *-*-* 09:00:00` - every Monday at 9 AM
- `*-*-* *:00:00` - every hour
- `*-*-* *:*:00` - every minute

**Check timers:**
```bash
systemctl list-timers
```"#,
        ),
        // Create user service
        SystemdRecipe::new(
            SystemdFeature::CreateUserService,
            "Create a user-level systemd service",
        )
        .with_template(
            r#"[Unit]
Description=My User Service

[Service]
Type=simple
ExecStart=/home/user/bin/myapp

[Install]
WantedBy=default.target"#,
        )
        .with_answer(
            r#"To create a user-level service (no root required):

1. **Create the directory:**
```bash
mkdir -p ~/.config/systemd/user
```

2. **Create the unit file** at `~/.config/systemd/user/myservice.service`:
```ini
[Unit]
Description=My User Service

[Service]
Type=simple
ExecStart=%h/bin/myapp
Restart=on-failure

[Install]
WantedBy=default.target
```

3. **Enable and start:**
```bash
systemctl --user daemon-reload
systemctl --user enable myservice.service
systemctl --user start myservice.service
```

4. **Enable lingering** (to run without login):
```bash
loginctl enable-linger $USER
```

**Notes:**
- `%h` expands to home directory
- User services run as your user, no sudo needed
- Located in `~/.config/systemd/user/`"#,
        ),
        // Enable/start service
        SystemdRecipe::new(
            SystemdFeature::EnableService,
            "Enable and start a systemd service",
        )
        .with_command("sudo systemctl enable servicename")
        .with_command("sudo systemctl start servicename")
        .with_answer(
            r#"To enable and start a service:

**Enable (start at boot) and start now:**
```bash
sudo systemctl enable --now servicename
```

**Or separately:**
```bash
sudo systemctl enable servicename   # Start at boot
sudo systemctl start servicename    # Start now
```

**Other useful commands:**
```bash
sudo systemctl status servicename   # Check status
sudo systemctl restart servicename  # Restart
sudo systemctl stop servicename     # Stop
sudo systemctl disable servicename  # Don't start at boot
```

**For user services:**
```bash
systemctl --user enable --now servicename
```"#,
        ),
        // View logs
        SystemdRecipe::new(SystemdFeature::ViewLogs, "View systemd service logs")
            .with_command("journalctl -u servicename")
            .with_answer(
                r#"To view service logs with journalctl:

**Basic log viewing:**
```bash
journalctl -u servicename           # All logs for service
journalctl -u servicename -f        # Follow (like tail -f)
journalctl -u servicename -n 100    # Last 100 lines
```

**Time-based filtering:**
```bash
journalctl -u servicename --since today
journalctl -u servicename --since "2024-01-01"
journalctl -u servicename --since "1 hour ago"
```

**Output formats:**
```bash
journalctl -u servicename -o json   # JSON output
journalctl -u servicename -o cat    # Just messages, no metadata
```

**Filter by priority:**
```bash
journalctl -u servicename -p err    # Errors only
journalctl -u servicename -p warning
```

**Boot-related:**
```bash
journalctl -u servicename -b        # Current boot only
journalctl -u servicename -b -1     # Previous boot
```"#,
            ),
        // Debug failing service
        SystemdRecipe::new(
            SystemdFeature::DebugService,
            "Debug a failing systemd service",
        )
        .with_answer(
            r#"To debug a failing service:

1. **Check status:**
```bash
systemctl status servicename
```

2. **View detailed logs:**
```bash
journalctl -u servicename -n 50 --no-pager
```

3. **Check for configuration errors:**
```bash
systemd-analyze verify /etc/systemd/system/servicename.service
```

4. **Try running manually:**
```bash
# Get the ExecStart command from:
systemctl cat servicename
# Then run it directly to see errors
```

5. **Common issues:**
- **Permission denied**: Check User= and file permissions
- **Path not found**: Verify ExecStart path exists
- **Dependencies**: Check After= and Wants= are correct
- **Environment**: May need Environment= or EnvironmentFile=

6. **Reset failed state:**
```bash
sudo systemctl reset-failed servicename
```

7. **Enable debug output:**
```bash
# Add to [Service] section:
Environment="DEBUG=1"
StandardOutput=journal
StandardError=journal
```"#,
        ),
        // Socket activation
        SystemdRecipe::new(
            SystemdFeature::SocketActivation,
            "Create a socket-activated service",
        )
        .with_answer(
            r#"To create a socket-activated service:

1. **Create the socket** at `/etc/systemd/system/myapp.socket`:
```ini
[Unit]
Description=My App Socket

[Socket]
ListenStream=8080
Accept=no

[Install]
WantedBy=sockets.target
```

2. **Create the service** at `/etc/systemd/system/myapp.service`:
```ini
[Unit]
Description=My App
Requires=myapp.socket

[Service]
Type=simple
ExecStart=/path/to/myapp
StandardInput=socket
```

3. **Enable the socket (not the service):**
```bash
sudo systemctl daemon-reload
sudo systemctl enable myapp.socket
sudo systemctl start myapp.socket
```

**Benefits:**
- Service starts on-demand when connection arrives
- Systemd handles socket, service can restart without dropping connections
- Better resource usage for infrequent services"#,
        ),
        // Service hardening
        SystemdRecipe::new(
            SystemdFeature::HardenService,
            "Harden a systemd service with security options",
        )
        .with_answer(
            r#"To harden a systemd service, add these to `[Service]`:

**Basic hardening:**
```ini
[Service]
# Run as non-root
User=myuser
Group=mygroup

# Filesystem restrictions
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
ReadOnlyPaths=/
ReadWritePaths=/var/lib/myapp

# Network restrictions
PrivateNetwork=true  # or use RestrictAddressFamilies=

# Capability restrictions
NoNewPrivileges=true
CapabilityBoundingSet=

# System call filtering
SystemCallFilter=@system-service
SystemCallArchitectures=native
```

**Check security score:**
```bash
systemd-analyze security servicename
```

**Common options:**
- `ProtectSystem=strict` - mount /usr, /boot, /etc read-only
- `ProtectHome=true` - make /home inaccessible
- `PrivateTmp=true` - private /tmp namespace
- `NoNewPrivileges=true` - prevent privilege escalation
- `PrivateDevices=true` - no access to /dev

**Allow specific capabilities:**
```ini
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE
```"#,
        ),
    ]
}
