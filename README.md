# Anna Assistant - Next Generation

**Minimal, Contract-Driven Linux System Assistant**

Version: 0.9.0
Target: Arch Linux
Language: Rust + Bash

---

## Overview

Anna is a production-grade Linux assistant designed with strict architectural contracts:

- **annad**: Root daemon handling privileged operations via Unix socket (`/run/anna.sock`)
- **annactl**: Non-privileged CLI client for user interaction
- **Polkit integration**: All privilege elevation through standard system dialogs
- **Zero external dependencies**: No web frameworks, AI models, or cloud services
- **Test-driven**: Every feature validated through QA harness

---

## Architecture

```
┌─────────────────┐
│    annactl      │  (user space, no privileges)
│   (CLI client)  │
└────────┬────────┘
         │ Unix Socket
         │ /run/anna.sock
         ▼
┌─────────────────┐
│     annad       │  (root daemon)
│  (privileged)   │
└─────────────────┘
         │
         ▼
    /etc/anna/
  (config, policy)
```

---

## Quick Start

### Prerequisites

```bash
# Arch Linux
sudo pacman -S rust cargo systemd

# Verify
rustc --version
cargo --version
```

### Installation

```bash
# Clone and build
git clone <repository>
cd anna-assistant

# Run installer
./scripts/install.sh
```

The installer will:
1. ✓ Check requirements
2. ✓ Compile binaries
3. ✓ Install to /usr/local/bin
4. ✓ Create systemd service
5. ✓ Enable and start daemon
6. ✓ Run diagnostics

### Basic Usage

```bash
# Check daemon status
annactl status

# Run diagnostics
annactl doctor

# Ping daemon
annactl ping

# View configuration
annactl config

# Get help
annactl --help
```

---

## Development

### Build from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Binaries appear in:
#   target/debug/annad
#   target/debug/annactl
```

### Run Tests

```bash
# Full QA validation
./tests/qa_runner.sh

# Individual components
cargo test
cargo clippy
```

### Project Structure

```
anna-assistant/
├── Cargo.toml              # Workspace definition
├── src/
│   ├── annad/              # Daemon (root process)
│   │   ├── src/
│   │   │   ├── main.rs     # Entry point
│   │   │   ├── config.rs   # Configuration loading
│   │   │   ├── rpc.rs      # Unix socket RPC
│   │   │   └── diagnostics.rs # System checks
│   │   └── Cargo.toml
│   └── annactl/            # CLI client (user process)
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
├── config/
│   └── default.toml        # Default configuration
├── etc/
│   └── systemd/
│       └── annad.service   # Systemd unit
├── scripts/
│   ├── install.sh          # Installation script
│   └── uninstall.sh        # Safe uninstaller
├── tests/
│   └── qa_runner.sh        # QA test harness
└── docs/
    └── (documentation)
```

---

## Configuration

Location: `/etc/anna/config.toml`

```toml
[daemon]
socket_path = "/run/anna.sock"
pid_file = "/run/anna.pid"

[autonomy]
tier = 0        # 0=manual, 1=low-risk
enabled = false

[logging]
level = "info"
directory = "/var/log/anna"
```

---

## Service Management

```bash
# Status
sudo systemctl status annad

# Restart
sudo systemctl restart annad

# Stop
sudo systemctl stop annad

# View logs
sudo journalctl -u annad -f
```

---

## Uninstallation

```bash
./scripts/uninstall.sh
```

This will:
- Stop and disable service
- Back up config to `~/Documents/anna_backup_<timestamp>/`
- Remove all binaries and files
- Clean up socket

---

## Contract Guarantees

This implementation adheres to the following immutable contracts:

1. **Privilege Separation**: Daemon runs as root, CLI never requires sudo
2. **Communication**: Unix socket at `/run/anna.sock` with 0666 permissions
3. **Configuration**: Single source of truth at `/etc/anna/config.toml`
4. **Installation**: Reproducible on clean Arch Linux VM
5. **Testing**: All features validated through `qa_runner.sh`
6. **No Magic**: Zero dynamic code generation, no external APIs
7. **Modularity**: Each component is independently testable
8. **Safety**: Uninstaller creates timestamped backups

---

## Autonomy Tiers

- **Tier 0**: Manual only (default) - All actions require explicit user approval
- **Tier 1**: Low-risk automation - Read-only diagnostics, logging, telemetry
- **Tier 2+**: (Not implemented) - Reserved for future capability expansion

---

## Troubleshooting

### Daemon won't start

```bash
# Check logs
sudo journalctl -u annad -n 50

# Verify socket
ls -la /run/anna.sock

# Run diagnostics
annactl doctor
```

### Connection refused

```bash
# Ensure daemon is running
sudo systemctl status annad

# Check socket permissions
ls -la /run/anna.sock
# Should show: srw-rw-rw- 1 root root
```

### Configuration issues

```bash
# Validate config syntax
annactl config

# Reset to defaults
sudo rm /etc/anna/config.toml
sudo systemctl restart annad
```

---

## Contributing

This is a contract-driven project. Before contributing:

1. Review the architectural contracts above
2. Ensure all tests pass: `./tests/qa_runner.sh`
3. Follow the existing code structure
4. No external frameworks or dependencies without discussion
5. All features must be testable and documented

---

## License

MIT

---

## Version History

**0.9.0** (Current)
- Initial Next-Gen implementation
- Core daemon/client architecture
- Unix socket RPC
- Basic diagnostics
- Installation/uninstallation scripts
- QA test harness

---

**Built with discipline. Zero compromise on contracts.**
