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

# Check status
annactl status

# Reset learned data
annactl reset

# Uninstall
annactl uninstall

# Version
annactl --version
```

## Features

- **Grounded responses**: Anna answers from actual system data, never invents facts
- **Auto-probes**: Automatically runs system queries for memory/CPU/disk questions
- **Hardware-aware**: Selects optimal model based on your CPU, RAM, and GPU
- **Self-healing**: Auto-repairs Ollama and model issues
- **Auto-update**: Checks for updates every 60 seconds

## Architecture

Anna consists of two components:

- **annad**: Root-level systemd service that manages Ollama, models, and system state
- **annactl**: User CLI that communicates with annad over a Unix socket

## Documentation

- [SPEC.md](SPEC.md) - Authoritative specification
- [CHANGELOG.md](CHANGELOG.md) - Version history

## Version

v0.0.6

## License

Apache-2.0
