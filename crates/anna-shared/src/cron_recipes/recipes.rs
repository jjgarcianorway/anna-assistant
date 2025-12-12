//! Built-in cron recipes (v0.0.234).

use super::types::{CronFeature, CronRecipe};

/// Get built-in cron recipes
pub fn builtin_recipes() -> Vec<CronRecipe> {
    vec![
        // Add a cron job
        CronRecipe::new(CronFeature::AddJob, "Add a new cron job")
            .with_command("crontab -e")
            .with_answer(
                r#"To add a cron job:

1. **Open your crontab:**
```bash
crontab -e
```

2. **Add a line with the schedule and command:**
```
# Format: minute hour day month weekday command
#    *      *     *    *      *
#    |      |     |    |      |
#    |      |     |    |      +-- Day of week (0-7, 0 or 7 = Sunday)
#    |      |     |    +--------- Month (1-12)
#    |      |     +-------------- Day of month (1-31)
#    |      +-------------------- Hour (0-23)
#    +--------------------------- Minute (0-59)

# Examples:
0 * * * * /path/to/script.sh        # Every hour
0 0 * * * /path/to/backup.sh        # Daily at midnight
30 4 * * 0 /path/to/weekly.sh       # Sunday 4:30 AM
0 9 1 * * /path/to/monthly.sh       # First of month, 9 AM
*/5 * * * * /path/to/frequent.sh    # Every 5 minutes
```

3. **Save and exit** (the cron daemon will pick up changes automatically)

**Useful shortcuts:**
- `@reboot` - Run once at startup
- `@hourly` - Run every hour (same as `0 * * * *`)
- `@daily` - Run daily at midnight
- `@weekly` - Run weekly on Sunday
- `@monthly` - Run on first day of month"#,
            ),
        // List cron jobs
        CronRecipe::new(CronFeature::ListJobs, "List current cron jobs")
            .with_command("crontab -l")
            .with_answer(
                r#"To list your cron jobs:

**Your user's crontab:**
```bash
crontab -l
```

**System-wide cron jobs:**
```bash
# System crontab
cat /etc/crontab

# Hourly, daily, weekly, monthly directories
ls -la /etc/cron.d/
ls -la /etc/cron.hourly/
ls -la /etc/cron.daily/
ls -la /etc/cron.weekly/
ls -la /etc/cron.monthly/
```

**All users' crontabs (root only):**
```bash
for user in $(cut -f1 -d: /etc/passwd); do
    echo "=== $user ==="
    crontab -u $user -l 2>/dev/null
done
```

**Specific user's crontab:**
```bash
sudo crontab -u username -l
```"#,
            ),
        // Edit crontab
        CronRecipe::new(CronFeature::EditCrontab, "Edit crontab")
            .with_command("crontab -e")
            .with_answer(
                r#"To edit your crontab:

```bash
crontab -e
```

**Change the editor** (if you prefer vim/nano):
```bash
# For current session
export EDITOR=nano && crontab -e

# Permanently (add to ~/.bashrc)
export EDITOR=nano
```

**Edit another user's crontab (root only):**
```bash
sudo crontab -u username -e
```

**Import from a file:**
```bash
crontab mycrontab.txt
```

**Tips:**
- Changes take effect immediately after saving
- Always test your script manually first
- Use absolute paths for commands
- Redirect output to a log file"#,
            ),
        // Remove cron job
        CronRecipe::new(CronFeature::RemoveJob, "Remove a cron job").with_answer(
            r#"To remove cron jobs:

**Remove specific job:**
1. Edit your crontab: `crontab -e`
2. Delete or comment out the line (prefix with #)
3. Save and exit

**Remove ALL your cron jobs:**
```bash
crontab -r
```
*Warning: This removes your entire crontab!*

**Safer: Remove with confirmation:**
```bash
crontab -i -r
```

**Backup before removing:**
```bash
crontab -l > ~/crontab.backup
crontab -r
# To restore:
crontab ~/crontab.backup
```"#,
        ),
        // View logs
        CronRecipe::new(CronFeature::ViewLogs, "View cron job logs").with_answer(
            r#"To view cron logs:

**System cron log (varies by distro):**
```bash
# Arch/systemd
journalctl -u cronie

# Debian/Ubuntu
grep CRON /var/log/syslog

# RedHat/CentOS
cat /var/log/cron
```

**Redirect job output to a log file:**
```
# In your crontab:
0 * * * * /script.sh >> /var/log/myjob.log 2>&1

# With timestamp:
0 * * * * /script.sh >> /var/log/myjob.log 2>&1 && echo "---$(date)---" >> /var/log/myjob.log
```

**Email output (if mail is configured):**
```
# Add at top of crontab:
MAILTO=your@email.com

# Or suppress output:
0 * * * * /script.sh > /dev/null 2>&1
```"#,
        ),
        // Syntax help
        CronRecipe::new(CronFeature::SyntaxHelp, "Cron syntax explanation").with_answer(
            r#"Cron expression format:

```
 ┌───────────── minute (0-59)
 │ ┌───────────── hour (0-23)
 │ │ ┌───────────── day of month (1-31)
 │ │ │ ┌───────────── month (1-12)
 │ │ │ │ ┌───────────── day of week (0-7, Sun=0 or 7)
 │ │ │ │ │
 * * * * * command
```

**Special characters:**
- `*` = any value
- `,` = list (1,3,5 = 1, 3, and 5)
- `-` = range (1-5 = 1 through 5)
- `/` = step (*/15 = every 15)

**Examples:**
```
*/15 * * * *     # Every 15 minutes
0 */2 * * *      # Every 2 hours
0 9-17 * * 1-5   # Hourly, 9-5, Mon-Fri
0 0 1,15 * *     # 1st and 15th of month
30 4 * * 0       # Sunday 4:30 AM
```

**Special strings:**
```
@reboot          # Run once at startup
@hourly          # 0 * * * *
@daily           # 0 0 * * *
@weekly          # 0 0 * * 0
@monthly         # 0 0 1 * *
@yearly          # 0 0 1 1 *
```"#,
        ),
        // Environment variables
        CronRecipe::new(CronFeature::Environment, "Cron environment variables").with_answer(
            r#"Cron runs with a minimal environment. Common issues:

**Set variables in crontab:**
```
# At top of crontab:
PATH=/usr/local/bin:/usr/bin:/bin
SHELL=/bin/bash
MAILTO=your@email.com

# Then your jobs:
0 * * * * /script.sh
```

**Or in your script:**
```bash
#!/bin/bash
export PATH=/usr/local/bin:/usr/bin:/bin
export HOME=/home/user
# ... rest of script
```

**Source your profile:**
```
0 * * * * source ~/.bashrc && /path/to/script.sh
```

**Common variables:**
```
HOME     # User's home directory
LOGNAME  # User running the job
PATH     # Command search path (very limited!)
SHELL    # Shell to use (default: /bin/sh)
MAILTO   # Email output recipient
```

**Debug environment:**
```
# Add this job to see what cron sees:
* * * * * env > /tmp/cron_env.txt
```"#,
        ),
        // Debug cron job
        CronRecipe::new(CronFeature::DebugJob, "Debug a failing cron job").with_answer(
            r#"To debug a failing cron job:

1. **Check if cron is running:**
```bash
systemctl status cronie    # Arch
systemctl status cron      # Debian/Ubuntu
```

2. **Verify your job is installed:**
```bash
crontab -l | grep your-script
```

3. **Check cron logs:**
```bash
journalctl -u cronie --since "1 hour ago"
```

4. **Test manually with cron's environment:**
```bash
env -i SHELL=/bin/bash HOME=$HOME /path/to/script.sh
```

5. **Common issues:**
   - **Wrong PATH**: Use absolute paths or set PATH
   - **Permissions**: Script must be executable (`chmod +x`)
   - **No output capture**: Add `>> /tmp/job.log 2>&1`
   - **GUI/display**: X apps need DISPLAY variable
   - **User context**: Check which user owns the crontab

6. **Verbose logging:**
```
# In your crontab:
* * * * * /bin/bash -x /path/to/script.sh > /tmp/debug.log 2>&1
```"#,
        ),
    ]
}
