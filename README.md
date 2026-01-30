# Anna

```
    _
   / \   _ __  _ __   __ _
  / _ \ | '_ \| '_ \ / _` |
 / ___ \| | | | | | | (_| |
/_/   \_\_| |_|_| |_|\__,_|

Your AI System Administrator
```

Anna is a **local AI assistant** that manages your Linux system. She runs diagnostics, installs packages, configures services, and keeps your system healthy - all through natural conversation via terminal or Telegram.

## Quick Start

```bash
# Install (Arch, Ubuntu, Fedora, Debian)
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash

# Ask anything
annactl "how much disk space do I have?"
annactl "install htop"
annactl "is my system healthy?"
```

## Telegram Access (Recommended)

Control your system from anywhere via Telegram.

### Setup

1. Create a bot with [@BotFather](https://t.me/botfather) on Telegram
2. Get your Telegram user ID from [@userinfobot](https://t.me/userinfobot)
3. Add to `/etc/anna/telegram.env`:

```bash
ANNA_TELEGRAM_TOKEN=your_bot_token_here
ANNA_TELEGRAM_USERS=your_telegram_id
```

4. Restart Anna: `sudo systemctl restart annad`

### What You Can Do

Just message your bot naturally:

- "how's the system?" - Quick status
- "any updates?" - Check for updates
- "health check" - Full health report
- "fix issues" - Auto-fix safe problems
- "clean up" - Clear caches and logs
- "remind me in 2 hours to check backups" - Set reminders
- "set up morning briefing at 8am" - Daily health reports

Or ask any question - Anna will figure it out.

## Features

### Proactive Monitoring

- **Morning briefing**: Daily natural-language health report at your chosen time
- **Critical alerts only**: Notifications only for emergencies (disk full, multiple service failures)
- **Self-healing**: Automatically restarts failed services, clears stale locks, frees disk space

### Smart Updates

- Categorizes updates (security, kernel, regular)
- Warns when reboot is needed
- Shows updates in morning briefing (no spam)

### Anomaly Detection

- Learns normal patterns for RAM, CPU, disk, network
- Detects deviations from baseline
- Reports anomalies in morning briefing

### Safe Changes

All system changes:
- Require confirmation
- Create backups first
- Can be rolled back
- Are logged for history

## Multi-User Scenarios

### Sharing Your Anna (Same Computer)

To let another person control your computer via their Telegram:

```bash
# Add their Telegram ID to allowed users
ANNA_TELEGRAM_USERS=your_id,their_id
```

They'll be able to run commands on your system. Only do this with people you trust.

### Multiple Computers

Each computer needs its own Anna installation:

1. Run the installer on each machine
2. Create a separate Telegram bot for each (or use the same bot with different notification channels)
3. Configure each machine's `/etc/anna/telegram.env`

### New User Setup

For someone setting up Anna on their own computer:

1. Run the installer:
   ```bash
   curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
   ```

2. Create a Telegram bot (optional but recommended):
   - Message @BotFather: `/newbot`
   - Choose a name and username
   - Copy the token

3. Get your Telegram ID:
   - Message @userinfobot
   - Copy your ID

4. Configure Telegram:
   ```bash
   sudo tee /etc/anna/telegram.env << EOF
   ANNA_TELEGRAM_TOKEN=your_token
   ANNA_TELEGRAM_USERS=your_id
   EOF
   sudo systemctl restart annad
   ```

5. Message your bot "hello" to test

## Commands

```bash
annactl "question"     # Ask anything
annactl                # Interactive mode
annactl status         # Daemon health
```

Everything else? Just ask naturally.

## Requirements

- Linux with systemd
- 8GB+ RAM (runs Qwen2.5 locally via Ollama)
- Ollama (installed automatically)

## Architecture

```
annactl ──> annad ──> Ollama (local LLM)
              │
              ├── Telegram bot (optional)
              ├── Scheduler (reminders, briefings)
              ├── Self-healing (auto-recovery)
              └── Anomaly detection (baselines)
```

All data stays local:
- `/etc/anna/` - Configuration
- `/var/lib/anna/` - State data
- `/run/anna/` - Runtime socket

## Privacy

Anna runs **100% locally**. No cloud APIs, no telemetry, no data leaving your machine. The only network requests are for auto-updates (from GitHub releases).

## FAQ

**Why is the first query slow?**
LLM needs to warm up. After that, responses are fast.

**Can Anna break my system?**
All changes require confirmation and can be undone. Critical system files are protected.

**Does it work offline?**
Yes, after initial install. LLM runs locally.

**Can I use a different model?**
Yes. Edit `/etc/anna/config.toml` or set `ANNA_OLLAMA_MODEL`.

## Version

Current: **v0.3.99**

Recent changes:
- Natural morning briefings (sounds human, varies daily)
- Critical-only notifications (no spam)
- Self-healing (auto-restart services, clear locks, free disk)
- Telegram conversation context (remembers recent questions)

## License

GPL-3.0
