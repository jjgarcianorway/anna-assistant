# Anna

```
    _
   / \   _ __  _ __   __ _
  / _ \ | '_ \| '_ \ / _` |
 / ___ \| | | | | | | (_| |
/_/   \_\_| |_|_| |_|\__,_|

Your Local IT Department (that actually works)
```

> *"Have you tried turning it off and on again?"* - Not Anna. She runs actual diagnostics.

Anna is a **local AI assistant** that knows your Linux system inside out. She answers questions using real data from your machine - never making things up, never hallucinating. Think of her as having a tiny, competent IT department living inside your terminal.

## What Makes Anna Different?

Most AI assistants will confidently tell you that your disk is 47% full when they have no idea. Anna actually checks.

```bash
$ annactl "how much disk space do I have?"

Checking your storage situation...

Your root partition (/) has 234GB free out of 500GB (47% used).
That swap partition? 16GB, barely touched.
That random USB drive you forgot about? Still mounted at /mnt/backup with 89GB available.

Evidence: df -h, lsblk
```

Every answer is **grounded in evidence**. Anna shows her work.

## Quick Start

```bash
# One-liner install (Arch, Ubuntu, Fedora, Debian supported)
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash

# Ask anything about your system
annactl "is my system healthy?"
annactl "what's eating my RAM?"
annactl "show me failed services"

# Or just start chatting
annactl
```

## The Hollywood IT Experience

When you enable internal communications (`show internal comms`), you get to be a fly on the wall in Anna's virtual IT department:

```
lhoqvso: why is my system slow?

  Marcus (team): I'll check the usual suspects.
  Anna: Running performance diagnostics...

  [probe] ps aux --sort=-%mem | head -10
  [probe] top -bn1 | head -20
  [probe] free -h

  Anna: Found the culprit. Chrome is using 4.2GB. Again.

Chrome has 47 tabs open (yes, I counted). That's eating 68% of your RAM.
Firefox in the background adds another 1.1GB. Slack's electron wrapper
is also being dramatic with 800MB.

Consider: closing some tabs, or accepting your fate as a Chrome farmer.
```

*It's like watching a TV show about a competent IT department. Except it's real and it's your computer.*

## Features That Actually Work

### Natural Language Everything

Just talk to Anna. No flags, no syntax, no man pages.

```bash
# System questions
"how much RAM do I have?"
"is docker running?"
"show me failed services"
"what's using port 3000?"

# Configuration (yes, really)
"enable syntax highlighting in vim"
"set my git email to me@example.com"
"add an alias ll='ls -la' to my bashrc"

# Package management
"install htop"
"remove that flatpak I never use"
"update all packages"

# Service management
"restart nginx"
"show docker logs"
"enable ssh at startup"
```

### Settings Through Conversation

Forget config files. Just tell Anna what you want:

```
> make Anna more casual
[config] Made my style more casual.
  Hey! I'll be more chill now.

> enable learning mode
[config] Enabled learning mode - I'll explain why commands work.
  I'll explain why commands work and what they do.

> my email is lhoqvso@company.com
[config] Got it! I'll notify you at lhoqvso@company.com for long-running tickets.
  When a request takes a long time, I'll email you the answer.

> be more playful
[config] Enabled playful humor mode.
  This is going to be fun! :)
```

Available settings:
- **Learning mode**: Get explanations for commands
- **Verbosity**: Minimal, normal, or detailed responses
- **Formality**: Casual, balanced, or formal tone
- **Humor**: None, subtle, or playful
- **Auto-confirm**: Skip confirmations for safe changes
- **Internal comms**: See the IT team discuss your request
- **Email notifications**: Get notified when long tasks complete

### Real-Time Streaming

Watch Anna think in real-time. No more staring at a spinner wondering if she crashed:

```
lhoqvso: explain my network configuration

Your network setup is actually quite... <streaming word by word>
interesting. You have three interfaces: enp0s31f6 (your main
ethernet, 192.168.1.42), wlp82s0 (wifi, currently down), and
docker0 (the bridge network for containers at 172.17.0.1)...
```

### Idle-Time Tips

Left the terminal open? Anna notices and shares helpful tips:

```
lhoqvso: <sits idle for 30 seconds>

[tip] By the way, if you want me to explain why commands work,
      just say "enable learning mode".

lhoqvso: _
```

*Max 3 tips per session because Anna knows when to stop.*

### Session Memory

Anna remembers what you talked about:

```
Good evening, lhoqvso!

Last time (2 hours ago): handled 5 queries, discussed vim (3 times),
learned about your nvim setup.

What can I help with?
```

### Recipe Learning

Ask the same question twice? Anna learns and responds instantly:

```
# First time: Anna runs probes, calls LLM, takes 3 seconds
$ annactl "how do I enable syntax highlighting in vim?"

# Second time: Instant response from learned recipe
$ annactl "enable vim syntax highlighting"
# Response in 50ms
```

Anna learns from successful interactions and builds a recipe book of your patterns.

### RPG-Style Stats

Because everything is better with XP:

```
$ annactl stats

╔════════════════════════════════════════════════╗
║           ANNA SERVICE DESK STATS              ║
╠════════════════════════════════════════════════╣
║  Staff: lhoqvso          Title: Senior Tech    ║
║  Level: 12               XP: 2,847 / 3,000     ║
║  ████████████████░░░░░░░░  (95%)              ║
╠════════════════════════════════════════════════╣
║  ACHIEVEMENTS                                  ║
║  [1] First Query      <3d> Three Day Streak   ║
║  (90+) Perfectionist  {*} Night Owl            ║
╠════════════════════════════════════════════════╣
║  TEAM LEADERBOARD                              ║
║  1. Desktop Team    - 847 tickets             ║
║  2. Storage Team    - 523 tickets             ║
║  3. Network Team    - 412 tickets             ║
╚════════════════════════════════════════════════╝
```

### Safe Change Engine

When Anna makes changes, she's careful:

```
> enable dark mode in vim

I can add these settings to your .vimrc:
  set background=dark
  colorscheme slate

Apply this change? [y/N] y

Creating backup... /home/lhoqvso/.vimrc.anna.1702147523
Applying changes... done!

Done! Applied 1 change.
(You can undo with: annactl undo vim-dark-1702147523)
```

Every change:
- Creates a backup first
- Can be undone
- Requires confirmation (unless you enable auto-confirm)
- Is logged for history

## IT Department Teams

Anna's virtual department has specialized teams:

| Team | Handles | Example Queries |
|------|---------|-----------------|
| **Desktop** | Editors, terminals, preferences | "configure vim", "customize my prompt" |
| **Network** | Connectivity, DNS, routing | "why can't I reach google?", "show open ports" |
| **Storage** | Disks, mounts, filesystems | "disk usage", "mount this drive" |
| **Security** | Permissions, access, vulnerabilities | "who can read this file?", "check for updates" |
| **Services** | Systemd, packages, daemons | "restart nginx", "install docker" |
| **Performance** | CPU, memory, processes | "what's slow?", "top processes" |
| **Hardware** | Devices, drivers, peripherals | "is my audio working?", "show GPU info" |

Each team has named staff members who handle your requests. Enable internal comms to meet them!

## Supported Recipes

Anna comes with built-in recipes for common tasks:

### Shell Configuration
- bash/zsh/fish aliases and functions
- Environment variables
- Prompt customization
- PATH management

### Git Configuration
- User name and email setup
- Default branch settings
- Credential helpers
- Aliases (co, br, ci, st)

### SSH Key Management
- Key generation (ed25519, RSA)
- SSH agent setup
- Config file management
- GitHub/GitLab integration

### Systemd Services
- Create custom service units
- Timer configurations
- Service enable/disable/restart
- Log viewing with journalctl

### Cron Jobs
- Add/remove scheduled tasks
- Common presets (hourly, daily, weekly)
- Crontab syntax help
- Job listing and editing

### Docker Compose
- Project scaffolding
- Service management
- Log viewing
- Network inspection

### Package Management
- Cross-distro support (apt, pacman, dnf, flatpak, snap)
- Automatic package name mapping
- Safe removal with dependency checking

## Commands

Anna keeps it simple. Four commands, that's it:

```bash
annactl "question"     # Ask anything
annactl                # Interactive mode (the good stuff)
annactl status         # System health check
annactl stats          # Your RPG stats
annactl uninstall      # Remove Anna (but why would you?)
```

Everything else? Just ask in natural language.

## Requirements

- **Linux** with systemd
- **8GB+ RAM** recommended (Anna runs Qwen3 locally)
- **Ollama** (installed automatically if missing)

Works on: Arch, Ubuntu, Debian, Fedora, and most systemd-based distros.

## Architecture

```
┌─────────────────┐     ┌─────────────────┐
│    annactl      │────▶│     annad       │
│  (CLI client)   │◀────│   (daemon)      │
└─────────────────┘     └────────┬────────┘
                                 │
                    ┌────────────┼────────────┐
                    │            │            │
              ┌─────▼─────┐ ┌────▼────┐ ┌─────▼─────┐
              │  Probes   │ │ Ollama  │ │  Recipes  │
              │ (shell)   │ │ (LLM)   │ │ (learned) │
              └───────────┘ └─────────┘ └───────────┘
```

- **annad**: Background daemon that manages AI models and runs system probes
- **annactl**: Your CLI interface, talks to annad via Unix socket
- **Probes**: Shell commands that gather real system data
- **Recipes**: Learned patterns for instant responses

Data storage:
- `~/.anna/` - Your profile, tickets, learned recipes
- `/var/lib/anna/` - System snapshots, telemetry
- `/etc/anna/` - Global configuration

## Privacy

Anna runs **100% locally**. Your data never leaves your machine:

- No cloud APIs
- No telemetry phoning home
- No "anonymous" usage tracking
- The LLM runs on your hardware

The only network requests are for auto-updates (which you can disable).

## FAQ

**Q: Why is the first query slow?**
A: Anna needs to warm up the LLM (Qwen3). After that, responses are fast. Learned recipes respond in <100ms.

**Q: Can Anna break my system?**
A: Anna creates backups before any changes and requires confirmation. You can always `undo`. She also refuses to touch critical system files.

**Q: Does Anna work offline?**
A: Yes! The LLM runs locally. You only need internet for the initial install and auto-updates.

**Q: Why "Anna"?**
A: She's named after... actually, we're not sure. She just showed up one day and started being helpful.

**Q: Can I customize the AI model?**
A: The model selection is automatic based on your hardware. Anna prefers Qwen3-VL for vision support but falls back to smaller models on constrained systems.

**Q: How do I report bugs?**
A: Open an issue at https://github.com/jjgarcianorway/anna-assistant/issues

## Version

Current: **v0.0.351**

Recent highlights:
- **Centralized UI system** with consistent formatting across all displays
- **Zero compiler warnings** for cleaner, more reliable code
- **Enhanced greeting system** with personalized tips and status
- **Improved error display** with actionable recovery hints
- Real-time streaming output (watch Anna think!)
- Idle-time tips and suggestions
- Session history ("since last time")
- Pattern tracking and trend detection

See [CHANGELOG.md](CHANGELOG.md) for the full story.

## License

GPL-3.0

---

*"The AI that reads your system, not your mind."*

```
$ annactl "thanks anna"

You're welcome! That's what I'm here for.
(And unlike cloud AI, I'm not judging your 47 Chrome tabs.)
```
