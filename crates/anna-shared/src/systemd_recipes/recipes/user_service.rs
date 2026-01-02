//! User-level systemd service recipes.

use crate::systemd_recipes::types::{SystemdFeature, SystemdRecipe};

/// Recipe for creating a user-level systemd service
pub fn create_user_service_recipe() -> SystemdRecipe {
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
    )
}
