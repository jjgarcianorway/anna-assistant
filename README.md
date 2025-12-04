# Anna

A local AI assistant for Linux systems.

## Requirements

- Linux with systemd
- 8GB+ RAM recommended
- Internet connection for initial setup

## Installation

```bash
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | sudo bash
```

## Usage

```bash
# Send a request
annactl "what processes are using the most memory?"

# Interactive mode
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

## Architecture

Anna consists of two components:

- **annad**: Root-level systemd service that manages Ollama, models, and system state
- **annactl**: User CLI that communicates with annad over a Unix socket

## Documentation

- [SPEC.md](SPEC.md) - Authoritative specification
- [TRACKER.md](TRACKER.md) - Implementation tracker and release notes

## Version

v0.0.1 - Initial release

## License

Apache-2.0
