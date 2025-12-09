# Anna

Your local IT department - a team of AI specialists running entirely on your machine.

Anna answers questions about your Linux system using real data, never inventing facts.

## Quick Start

```bash
# Install
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash

# Ask anything
annactl "is my system healthy?"
annactl "show disk usage"
annactl "what's using the most memory?"

# Interactive mode
annactl
```

## Requirements

- Linux with systemd
- 8GB+ RAM recommended

## Features

### Natural Language Interface
Just ask questions in plain English. Anna understands context and runs the right system commands.

```
annactl "how much RAM do I have?"
annactl "is docker running?"
annactl "show me failed services"
```

### Grounded Responses
Every answer comes from actual system data. Anna never guesses or invents facts.

### Service Desk Theatre Experience
Anna presents as a virtual IT department with named specialists who handle your requests:
- **Desktop Team** - System configuration, editors, preferences
- **Network Team** - Connectivity, DNS, routing issues
- **Storage Team** - Disk usage, mounts, file systems
- **Security Team** - Permissions, access, vulnerabilities
- **Services Team** - Package management, systemd services

Each request gets a case number and assigned staff member, creating a "fly-on-wall"
experience of a real IT department.

### Personalized Experience
Anna remembers your patterns and preferences:
- Tracks your preferred tools (editors, commands)
- Notes frequently asked topics
- Maintains usage streaks
- Shows personalized greetings

### Staff Performance Tracking
Track team performance with gamification:
```
annactl stats
```
Shows XP, levels, achievements, and staff leaderboards.

### Async Tickets
For complex issues, Anna creates tickets that persist across sessions:
```
annactl "show my tickets"
```

## Commands

```bash
annactl "question"     # One-shot query
annactl                # Interactive REPL (personalized prompt)
annactl status         # System status
annactl stats          # Usage statistics & achievements
annactl uninstall      # Remove Anna
```

## Architecture

- **annad**: Background service (manages AI models, runs probes)
- **annactl**: CLI client (talks to annad via Unix socket)

Data stored in:
- `~/.anna/` - User data (profiles, tickets, preferences)
- `/var/lib/anna/` - System data (snapshots, telemetry)
- `/etc/anna/` - Configuration and staff stats

## Recent Updates (v0.0.168)

- **Personalized REPL prompt**: Shows your username instead of generic "You:"
- **Full stage module integration**: RPC handler reduced by 45%
- **Fixed CI workflow**: Updated version checking and CLI surface tests
- **Ongoing modularization**: Code quality improvements

See [CHANGELOG.md](CHANGELOG.md) for full version history.

## Documentation

- [CHANGELOG.md](CHANGELOG.md) - Version history
- [SPEC.md](SPEC.md) - Technical specification

## License

GPL-3.0
