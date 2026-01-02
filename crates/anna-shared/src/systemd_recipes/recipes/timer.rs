//! Timer-related systemd recipes.

use crate::systemd_recipes::types::{SystemdFeature, SystemdRecipe};

/// Recipe for creating a systemd timer (cron replacement)
pub fn create_timer_recipe() -> SystemdRecipe {
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
    )
}
