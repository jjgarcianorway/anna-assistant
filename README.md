# Anna

A local AI assistant for Linux systems with grounded, accurate responses.

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

# Check status
annactl status

# Reset learned data
annactl reset

# Uninstall
annactl uninstall

# Version
annactl --version
```

## Output Modes

Anna has two display modes:

**Normal mode** (default): Clean, IT department style output
- Shows what was checked (e.g., "Checking audio hardware...")
- Clean answer with reliability indicator
- Evidence source in footer when grounded

**Debug mode** (`-d` or `--debug`): Full pipeline visibility
- Shows translator intent and domain
- Probe execution with commands, exit codes, timing
- Evidence kinds and deterministic routing
- Full reliability signals breakdown

## Features

- **Grounded responses**: Anna answers from actual system data, never invents facts
- **Auto-probes**: Automatically runs system queries for memory/CPU/disk questions
- **Hardware-aware**: Selects optimal model based on your CPU, RAM, and GPU
- **Self-healing**: Auto-repairs Ollama and model issues
- **Auto-update**: Checks for updates every 60 seconds
- **Team-scoped specialists** (v0.0.26): Domain-specialized teams (Desktop, Storage, Network, Performance, Services, Security, Hardware) with junior/senior reviewer roles
- **Deterministic review gate** (v0.0.26): Hybrid review system that uses deterministic logic first, only escalating to LLM when signals are unclear
- **Execution traces** (v0.0.23): Full audit trail of pipeline execution for debugging

## Architecture

Anna consists of two components:

- **annad**: Root-level systemd service that manages Ollama, models, and system state
- **annactl**: User CLI that communicates with annad over a Unix socket

## Documentation

- [SPEC.md](SPEC.md) - Authoritative specification
- [CHANGELOG.md](CHANGELOG.md) - Version history

## Version

v0.0.63

## License

Apache-2.0
