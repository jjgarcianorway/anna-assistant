//! Log viewing systemd recipes.

use crate::systemd_recipes::types::{SystemdFeature, SystemdRecipe};

/// Recipe for viewing systemd service logs
pub fn view_logs_recipe() -> SystemdRecipe {
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
        )
}
