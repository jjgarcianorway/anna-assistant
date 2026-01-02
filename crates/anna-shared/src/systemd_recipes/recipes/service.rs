//! Service-related systemd recipes.

use crate::systemd_recipes::types::{SystemdFeature, SystemdRecipe};

/// Recipe for creating a basic systemd service
pub fn create_service_recipe() -> SystemdRecipe {
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
    .with_note("Replace myservice, paths, and user with your values")
}

/// Recipe for enabling and starting a systemd service
pub fn enable_service_recipe() -> SystemdRecipe {
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
    )
}
