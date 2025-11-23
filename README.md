# Anna Assistant

**Intelligent Arch Linux System Assistant with Deterministic Diagnostics**

[![Version](https://img.shields.io/badge/version-5.7.0--beta.276-blue.svg)](https://github.com/jjgarcianorway/anna-assistant)
[![License](https://img.shields.io/badge/license-GPL--3.0-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Arch%20Linux-1793d1.svg)](https://archlinux.org)

Anna is a local system assistant for Arch Linux that combines deterministic diagnostics with LLM-powered insights to help you understand and manage your system. Think of her as your caffeinated sysadmin buddy who actually reads the logs.

---

## Features

- **🧠 Internal Diagnostic Engine** - 9 rules analyzing services, disk, memory, CPU, packages, and mounts
- **🔍 Telemetry-First** - Real system data, zero hallucinations
- **💬 Interactive TUI** - Natural language queries with structured action plans
- **📊 System Status** - Comprehensive health reporting
- **🤖 Local LLM** - Requires Ollama (all data stays local, no cloud nonsense)
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

# One-shot natural language
annactl "what's using disk space?"
annactl "run a full diagnostic"
annactl "check my system health"
```

---

## Three Interfaces

### 1. Interactive TUI - `annactl`
Launch the interactive assistant for natural language queries and system management.

### 2. Status Check - `annactl status`
Quick health overview showing:
- Daemon and LLM status
- Top 3 system issues (from diagnostic engine)
- Recent logs

**Options:**
- `--json` - Machine-readable output for automation

### 3. Deep System Analysis (Natural Language)
Invoke comprehensive diagnostic analysis through natural language:
- **In TUI:** "run a full diagnostic", "check my system health", "show any problems"
- **One-shot:** `annactl "run a full diagnostic"`

The internal diagnostic engine analyzes 9 critical system areas:
- Failed/degraded services
- Disk space issues
- Memory pressure
- CPU load
- Orphaned packages
- Failed mounts
- Critical log issues

Results are presented conversationally with evidence, citations, and actionable commands.

---

## Architecture

**Daemon (`annad`):**
- Runs as systemd service
- Collects telemetry
- Performs health checks
- Executes approved actions
- RPC server (Unix socket)

**Client (`annactl`):**
- Simple CLI (`status`, natural language)
- Interactive TUI
- RPC client

**Intelligence:**
- Internal diagnostic engine (9 deterministic rules)
- 77+ deterministic recipes
- Local LLM via Ollama (required for conversational features)

---

## Requirements

- **OS:** Arch Linux (native) or Arch-based distros
- **CPU:** x86_64
- **RAM:** 8GB recommended (4GB minimum for basic features)
- **Disk:** 2GB for Anna + 4GB for LLM models
- **Required:** Ollama with a model (qwen2.5:3b or similar)

---

## Documentation

- **[User Guide](docs/USER_GUIDE.md)** - Complete usage guide
- **[Architecture](docs/ARCHITECTURE_BETA_200.md)** - System design
- **[Beta.217 Release](docs/BETA_217_COMPLETE.md)** - Diagnostic engine details
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

**Current:** Beta.258 - "How is my system today?" Daily Snapshot v1

**Recent milestones:**
- ✅ Beta.258 - Daily snapshot for "today" queries (sysadmin briefing format, session delta integration, 8 new tests, 186/250 maintained)
- ✅ Beta.257 - Unified health & status experience (health consistency, temporal wording, 13 new tests, 186/250 maintained)
- ✅ Beta.256 - Routing consolidation and expectation cleanup (+11 tests, 186/250 big suite passing, 74.4%)
- ✅ Beta.255 - Temporal and recency routing patterns (+2 tests, 175/250 big suite passing, 70.0%)
- ✅ Beta.254 - Punctuation and noise normalization (+10 tests, 173/250 big suite passing, 69.2%)
- ✅ Beta.253 - Strategic routing improvements and expectation corrections (+13 tests, 163/250 big suite passing, 65.2%)
- ✅ Beta.252 - Comprehensive failure taxonomy, metadata-driven tests (+3 tests, 150/250 big suite passing, 60.0%)
- ✅ Beta.251 - Conservative routing improvements (+9 tests, 147/250 big suite passing, 58.8%)
- ✅ Beta.250 - Canonical diagnostic formatter, consistent health answers across all surfaces
- ✅ Beta.249 - Router alignment with high-value fixes (55.2% pass rate on big suite, +7.6pp)
- ✅ Beta.248 - NL QA Marathon v1 (250-test big suite, measurement baseline)
- ✅ Beta.243 - First routing improvement pass (whitespace, punctuation, phrase variations)
- ✅ Beta.242 - Regression suite expansion (115 tests, 100% pass rate)
- ✅ Beta.241 - Regression test infrastructure foundation
- ✅ Beta.238 - Full diagnostic routing via natural language
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
