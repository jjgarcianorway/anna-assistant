# Anna Assistant Examples

This directory contains practical examples of using Anna Assistant.

## Basic Usage Examples

### Check System Status

```bash
# View daemon status and system info
annactl status
```

**Expected Output:**
```
╭───────────────╮
│ Anna Status │
╰───────────────╯

→ System
  Hostname: archlinux
  Kernel: 6.5.9-arch1-1

→ Daemon
  ✓ Running
  Version: v1.0.0-beta.17
  Uptime: 3600s

ℹ 5 recommendations pending
```

### Get Recommendations

```bash
# See all personalized recommendations
annactl advise
```

**Expected Output:**
```
╭──────────────────────────────╮
│ System Recommendations │
╰──────────────────────────────╯

ℹ Taking a look at your system...

→ Critical

1. 🔒 Install CPU microcode updates
   Your AMD processor needs microcode updates to protect against security
   vulnerabilities like Spectre and Meltdown. Think of it like a security
   patch for your CPU itself.

   → Run: pacman -S --noconfirm amd-ucode

   [ID: amd-microcode]

→ Recommended

2. 🧹 Clean up orphaned packages
   You have 12 orphaned packages that were installed as dependencies but are
   no longer needed. Removing them frees up disk space.

   → Run: pacman -Rns $(pacman -Qtdq)

   [ID: orphan-packages]
```

### Filter by Risk Level

```bash
# Show only critical recommendations
annactl advise --risk high

# Show only optional improvements  
annactl advise --risk low
```

### Apply Recommendations

```bash
# Apply a single recommendation by number
annactl apply --nums 1

# Apply by ID
annactl apply --id amd-microcode

# Dry run to see what would happen
annactl apply --nums 1 --dry-run

# Apply multiple recommendations
annactl apply --nums 1,2,5

# Apply a range
annactl apply --nums 1-5

# Apply complex selections
annactl apply --nums 1,3,5-7,10
```

### System Health Report

```bash
# Get a plain English system assessment
annactl report
```

**Expected Output:**
```
╭────────────────────────────────╮
│ 📊 System Health Report │
╰────────────────────────────────╯

→ 💭 What I think about your system

   I found 2 critical issues that need your attention right away.
   These affect your system's security or stability.

→ 📋 System Overview

   You're running Arch Linux with 1,523 packages installed.
   Your kernel is version 6.5.9-arch1-1 on AMD Ryzen 7 5800X.
   You're using Btrfs - great choice for modern features!
   You have plenty of disk space (45% used).

→ 🎯 Recommendations Summary

   🚨 2 Critical - These need immediate attention
   🔧 3 Recommended - Would improve your system
   ✨ 5 Optional - Nice to have enhancements

   By category:
     • 2 suggestions about Security
     • 3 suggestions about Performance
     • 5 suggestions about Desktop

→ 🚀 Next Steps

   Run annactl advise to see the critical issues that need fixing.
```

### Run Diagnostics

```bash
# Check if everything is working
annactl doctor
```

### View Configuration

```bash
# See current settings
annactl config

# Set autonomy tier (0-3)
annactl config --set autonomy_tier=1

# Enable/disable auto-update checking
annactl config --set auto_update_check=true
```

## Advanced Examples

### Systemd Service Management

```bash
# Check daemon status
sudo systemctl status annad

# View daemon logs
journalctl -u annad -f

# Restart daemon
sudo systemctl restart annad

# Stop daemon temporarily
sudo systemctl stop annad

# Enable autostart on boot
sudo systemctl enable annad
```

### Manual Installation

```bash
# If you didn't use the installer script

# 1. Build from source
cargo build --release

# 2. Install binaries
sudo cp target/release/annad /usr/local/bin/
sudo cp target/release/annactl /usr/local/bin/

# 3. Install systemd service
sudo cp annad.service /etc/systemd/system/
sudo systemctl daemon-reload

# 4. Start the service
sudo systemctl enable --now annad
```

### Integration Examples

#### Use in Shell Scripts

```bash
#!/bin/bash
# Check if Anna has critical recommendations

annactl advise --risk high > /tmp/anna_critical.txt

if [ -s /tmp/anna_critical.txt ]; then
    echo "⚠️  Critical system issues detected!"
    cat /tmp/anna_critical.txt
    exit 1
else
    echo "✓ System healthy"
    exit 0
fi
```

#### Cron Job for Periodic Checks

```bash
# Add to crontab
# Run Anna check every 6 hours and email if critical issues found

0 */6 * * * /usr/local/bin/annactl advise --risk high | mail -s "Anna Critical Alerts" admin@example.com
```

#### Git Hook Integration

```bash
# .git/hooks/pre-push
#!/bin/bash
# Ensure system is healthy before pushing

echo "Running Anna system check..."
CRITICAL=$(annactl advise --risk high 2>/dev/null | wc -l)

if [ "$CRITICAL" -gt 10 ]; then
    echo "⚠️  Critical system issues detected. Fix them first!"
    annactl advise --risk high
    exit 1
fi

echo "✓ System healthy, proceeding with push"
```

## Multi-User System Examples

### User-Specific Recommendations

Anna automatically detects user context (desktop environment, shell, display server) and personalizes recommendations.

```bash
# As user 'alice' using Hyprland + Wayland + zsh
$ annactl advise

# Shows:
# - Waybar (Wayland status bar)
# - Wofi (Wayland launcher)
# - Mako (Wayland notifications)
# - zsh-autosuggestions
# + System-wide advice (security, updates)

# As user 'bob' using i3 + X11 + bash
$ annactl advise  

# Shows:
# - i3blocks (X11 status bar)
# - Rofi (X11 launcher)
# - Dunst (X11 notifications)
# + System-wide advice (security, updates)
```

## Troubleshooting Examples

### Daemon Not Running

```bash
# Check if daemon is running
sudo systemctl status annad

# View logs for errors
journalctl -u annad --since "10 minutes ago"

# Try starting manually to see errors
sudo /usr/local/bin/annad
```

### Permission Issues

```bash
# Check socket permissions
ls -la /run/anna/anna.sock

# Should be: srw-rw-rw- (0666)

# If wrong, restart daemon
sudo systemctl restart annad
```

### Stale Recommendations

```bash
# Anna auto-refreshes, but you can force it via RPC
# (Refresh is internal-only now)

# Restart daemon to refresh
sudo systemctl restart annad
```

## Development Examples

### Testing New Detection Rules

```bash
# 1. Add your rule to recommender.rs or intelligent_recommender.rs

# 2. Build
cargo build

# 3. Stop production daemon
sudo systemctl stop annad

# 4. Run test daemon in foreground
sudo ./target/debug/annad

# 5. In another terminal, test
./target/debug/annactl advise | grep "your-new-rule"

# 6. Clean up
# Ctrl+C to stop test daemon
sudo systemctl start annad
```

### Custom RPC Client

```rust
// examples/custom_client.rs
use tokio::net::UnixStream;
use anna_common::ipc::{Request, Response, Method};

#[tokio::main]
async fn main() {
    let stream = UnixStream::connect("/run/anna/anna.sock")
        .await
        .expect("Failed to connect");

    let request = Request {
        id: 1,
        method: Method::Status,
    };

    // Send request, receive response
    // (See docs/IPC_API.md for full protocol)
}
```

## See Also

- [README.md](../README.md) - Project overview
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contributing guide
- [docs/IPC_API.md](../docs/IPC_API.md) - IPC protocol documentation
- [CHANGELOG.md](../CHANGELOG.md) - Version history
