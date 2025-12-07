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

### Service Desk Experience
Anna presents as a virtual IT department with named specialists:
- Storage team for disk questions
- Network team for connectivity
- Performance team for CPU/memory
- Desktop team for configuration

### Async Tickets
For complex issues, Anna creates tickets that persist across sessions:
```
annactl "show my tickets"
annactl reply CN-1234 "here's the info you requested"
```

### Stats & Achievements
Track your usage with XP, levels, and achievement badges:
```
annactl stats
```

## Commands

```bash
annactl "question"     # One-shot query
annactl                # Interactive REPL
annactl status         # System status
annactl stats          # Usage statistics
annactl -d "question"  # Debug mode (show pipeline)
annactl -i "question"  # Internal mode (show IT dialogue)
annactl reset          # Clear learned data
annactl uninstall      # Remove Anna
```

## Architecture

- **annad**: Background service (manages AI models, runs probes)
- **annactl**: CLI client (talks to annad via Unix socket)

Data stored in `~/.anna/` (user) and `/var/lib/anna/` (system).

## Documentation

- [CHANGELOG.md](CHANGELOG.md) - Version history
- [SPEC.md](SPEC.md) - Technical specification

## Version

v0.0.123

## License

GPL-3.0
