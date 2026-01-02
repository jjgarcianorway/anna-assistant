//! Debugging systemd services recipes.

use crate::systemd_recipes::types::{SystemdFeature, SystemdRecipe};

/// Recipe for debugging a failing systemd service
pub fn debug_service_recipe() -> SystemdRecipe {
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
    )
}
