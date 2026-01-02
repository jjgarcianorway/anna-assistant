//! Socket activation systemd recipes.

use crate::systemd_recipes::types::{SystemdFeature, SystemdRecipe};

/// Recipe for creating a socket-activated service
pub fn socket_activation_recipe() -> SystemdRecipe {
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
    )
}
