# Anna Assistant

**Intelligent Arch Linux System Assistant with Deterministic Diagnostics**

[![Version](https://img.shields.io/badge/version-5.7.0--beta.222-blue.svg)](https://github.com/jjgarcianorway/anna-assistant)
[![License](https://img.shields.io/badge/license-GPL--3.0-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Arch%20Linux-1793d1.svg)](https://archlinux.org)

Anna is a local system assistant for Arch Linux that combines deterministic diagnostics with optional LLM-powered insights to help you understand and manage your system.

---

## Features

- **🧠 Sysadmin Brain** - 9 diagnostic rules analyzing services, disk, memory, CPU, packages, and mounts
- **🔍 Telemetry-First** - Real system data, zero hallucinations
- **💬 Interactive TUI** - Natural language queries with structured action plans
- **📊 System Status** - Comprehensive health reporting
- **🤖 Local LLM** - Optional Ollama integration, all data stays local
- **📝 77+ Recipes** - Deterministic system management actions
- **🔒 Transparent** - Shows exact commands before execution
- **✅ Approval-Based** - Requires confirmation before changes

---

## Quick Start

**Install:**
```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

**Usage:**
```bash
# Interactive assistant
annactl

# System health check
annactl status

# Deep diagnostics
annactl brain

# One-shot query
annactl "what's using disk space?"
```

---

## Three Interfaces

### 1. Interactive TUI - `annactl`
Launch the interactive assistant for natural language queries and system management.

### 2. Status Check - `annactl status`
Quick health overview showing:
- Daemon and LLM status
- Top 3 system issues
- Recent logs

### 3. Brain Analysis - `annactl brain`
Deep diagnostic analysis with all 9 rules:
- Failed/degraded services
- Disk space issues
- Memory pressure
- CPU load
- Orphaned packages
- Failed mounts
- Critical log issues

**Options:**
- `--verbose` - Show evidence and citations
- `--json` - Machine-readable output for automation

---

## Architecture

**Daemon (`annad`):**
- Runs as systemd service
- Collects telemetry
- Performs health checks
- Executes approved actions
- RPC server (Unix socket)

**Client (`annactl`):**
- Three-command CLI
- Interactive TUI
- RPC client

**Intelligence:**
- Sysadmin Brain (deterministic rules)
- 77+ deterministic recipes
- Optional LLM integration (Ollama)

---

## Requirements

- **OS:** Arch Linux (native) or Arch-based distros
- **CPU:** x86_64
- **RAM:** 4GB minimum (8GB+ for LLM)
- **Disk:** 2GB for Anna + 4GB for LLM models (optional)
- **Optional:** Ollama for LLM features

---

## Documentation

- **[User Guide](docs/USER_GUIDE.md)** - Complete usage guide
- **[Architecture](docs/ARCHITECTURE_BETA_200.md)** - System design
- **[Beta.217 Release](docs/BETA_217_COMPLETE.md)** - Sysadmin Brain details
- **[Debugging Guide](docs/DEBUGGING_GUIDE.md)** - Troubleshooting
- **[Changelog](CHANGELOG.md)** - Version history

---

## Core Principles

- **Local-First:** All data stays on your machine, no cloud services
- **Telemetry-First:** Real system facts, not LLM guesses
- **Transparent:** Every command shown before execution
- **Deterministic:** Predictable, reproducible actions
- **Safe:** Approval required for system changes
- **Arch-Focused:** Deep Arch Linux integration

---

## What Anna is NOT

- ❌ Not a remote management tool
- ❌ Not a fully autonomous agent
- ❌ Not production-ready (beta software)
- ❌ Not a generic chatbot
- ❌ Not a replacement for system knowledge

---

## Security

Anna operates with appropriate privileges:
- Daemon runs as root (for system management)
- Client runs as user
- Actions require explicit approval
- All commands logged with citations
- No network access except localhost LLM

See [SECURITY.md](SECURITY.md) for details.

---

## Development

**Build:**
```bash
cargo build --release
```

**Test:**
```bash
cargo test
```

**Run locally:**
```bash
# Daemon (requires root)
sudo ./target/release/annad

# Client
./target/release/annactl
```

---

## Project Status

**Current:** Beta.222 - Complete Truecolor TUI Polish + Greetings

**Recent milestones:**
- ✅ Beta.222 - Complete truecolor palette migration + Beta.221 greetings
- ✅ Beta.221 - Smart greetings with context-aware welcome
- ✅ Beta.220 - Production-quality TUI with truecolor visuals
- ✅ Beta.218 - Brain diagnostics integrated into TUI
- ✅ Beta.217 - Sysadmin Brain (9 diagnostic rules)
- ✅ Beta.200 - Three-command architecture
- ✅ 77+ deterministic recipes
- ✅ Full RPC integration
- ✅ JSON output modes

**Next:**
- Network diagnostics
- Configuration validation
- Hardware monitoring
- Recipe expansion

---

## Contributing

Anna is currently in active development. Contributions welcome!

**Areas of interest:**
- Additional diagnostic rules
- Recipe improvements
- Documentation
- Testing
- Bug reports

---

## License

GPL-3.0 - See [LICENSE](LICENSE) for details.

---

## Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming
- [Ollama](https://ollama.ai/) - Local LLM runtime
- [Arch Wiki](https://wiki.archlinux.org/) - Documentation source
- Community feedback and testing

---

**Made for Arch Linux enthusiasts who want intelligent system management without sacrificing control or privacy.**
