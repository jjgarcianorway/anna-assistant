//! Service hardening systemd recipes.

use crate::systemd_recipes::types::{SystemdFeature, SystemdRecipe};

/// Recipe for hardening a systemd service with security options
pub fn harden_service_recipe() -> SystemdRecipe {
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
    )
}
