# Anna

A local AI assistant for Linux systems with grounded, accurate responses.
Hollywood IT department aesthetic - classic ASCII terminal style.

## Requirements

- Linux with systemd
- 8GB+ RAM recommended
- Internet connection for initial setup

## Installation

```bash
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

## Usage

```bash
# Send a request
annactl "what processes are using the most memory?"

# Interactive mode (exit with: quit, bye, q, :q)
annactl

# Debug mode - show full pipeline (translator, probes, evidence, traces)
annactl -d "what is my sound card?"

# Show internal IT communications (team dialogue)
annactl -i "check disk space"

# Check status
annactl status

# View RPG-style stats with achievements
annactl stats

# Reset learned data
annactl reset

# Uninstall
annactl uninstall

# Version
annactl --version
```

## Output Modes

Anna has two display modes:

**Normal mode** (default): Cinematic IT department experience
```
[you] how much disk space?

Checking disk space...

[anna]
Your root partition is 45% full (50GB used of 110GB).

Storage Support | Confident answer | 90% | Verified from disk
```

**Debug mode** (`-d` or `--debug`): Full pipeline visibility
- Shows translator intent and domain
- Probe execution with commands, exit codes, timing
- Evidence kinds and deterministic routing
- Full reliability signals breakdown

## Features

### Core
- **Grounded responses**: Anna answers from actual system data, never invents facts
- **Auto-probes**: Automatically runs system queries for memory/CPU/disk questions
- **Hardware-aware**: Selects optimal model based on your CPU, RAM, and GPU
- **Self-healing**: Auto-repairs Ollama and model issues
- **Auto-update**: Checks for updates, never downgrades

### Service Desk Theatre (v0.0.81+)
- **Named personas**: IT team members with distinct roles (Michael from Network, Sofia from Desktop, etc.)
- **Cinematic narrative**: "Checking disk space..." instead of raw probe output
- **Varied dialogue**: Different greetings, approvals, and escalation phrases
- **Time-aware greetings**: "Good morning!", "Good evening!" based on time of day

### RPG Stats System (v0.0.75+)
- **XP and Levels**: Earn XP for queries, level up over time
- **Titles**: Progress from "Trainee" to "Principal Engineer"
- **Streaks**: Track consecutive days of usage
- **Achievement Badges**: ASCII-style badges like `[100]` `<7d>` `{*}`

### Achievement Categories
- **Milestones**: `[1]` `[10]` `[50]` `[100]` `[500]`
- **Streaks**: `<3d>` `<7d>` `<30d>`
- **Quality**: `(90+)` `(ok)` `(<<)`
- **Teams**: `{*}` `{df}` `{ip}` `{top}`
- **Special**: `~00~` `~05~` `[rx]` `[!!]`

### Team System (v0.0.26+)
- **Domain specialists**: Desktop, Storage, Network, Performance, Services, Security, Hardware, Logs
- **Junior/Senior reviewers**: Escalation path for complex queries
- **Deterministic review gate**: Uses logic first, LLM only when needed

## Architecture

Anna consists of two components:

- **annad**: Root-level systemd service that manages Ollama, models, and system state
- **annactl**: User CLI that communicates with annad over a Unix socket

## Documentation

- [SPEC.md](SPEC.md) - Authoritative specification
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [FEATURES.md](FEATURES.md) - Feature details
- [ROADMAP.md](ROADMAP.md) - Development roadmap

## Version

v0.0.97

## License

Apache-2.0
