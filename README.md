# Anna Assistant

**Intelligent Arch Linux System Assistant - Version 6.5.0 (Experimental Prototype)**

[![Version](https://img.shields.io/badge/version-6.5.0-blue.svg)](https://github.com/jjgarcianorway/anna-assistant)
[![License](https://img.shields.io/badge/license-GPL--3.0-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Arch%20Linux-1793d1.svg)](https://archlinux.org)

---

## ⚠️ Important: 6.0.0 is a Prototype Reset

**Anna 6.0.0 is an experimental release focused on stabilizing the core architecture.**

- **CLI-only interface**: The interactive TUI from 5.x has been disabled
- **Daemon stability**: Focus on proven daemon features (Historian, ProactiveAssessment, health monitoring)
- **Clean foundation**: Removed unstable code to build a solid base for future UI work

This is **not production software**. It is a working prototype for Arch Linux power users who want local system intelligence without the instability of the 5.x TUI.

---

## What Actually Works in 6.0.0

### ✅ Stable Features

#### 1. System Health Monitoring (`annad` daemon)
- **Runs as systemd service** with root privileges
- **9 diagnostic rules** analyzing:
  - Failed/degraded systemd services
  - Disk space issues
  - Memory pressure
  - CPU load
  - Orphaned packages
  - Failed mounts
  - Critical log errors
- **RPC server** (Unix socket communication)

#### 2. Historian (Beta.279)
- **JSONL-based storage** of system health snapshots
- **6 temporal correlation rules**:
  - Service flapping detection
  - Disk growth trends
  - Sustained resource pressure
  - Network degradation
  - Kernel regression detection
- **Automatic pruning** and efficient lookups

#### 3. Adaptive Planner (6.2.0-6.3.1)
- **Arch Wiki-only planning** - No random blog posts or StackOverflow
- **Two scenarios implemented**:
  - DNS resolution troubleshooting (systemd-resolved)
  - Failed systemd service troubleshooting
- **Safety guarantees**:
  - Inspect steps before Change steps
  - All changes require confirmation
  - Rollback commands provided
  - Knowledge sources traceable to Arch Wiki
- **Deterministic** - No LLM needed for plan generation
- **Tested** - ACTS v1 test suite + selftest command

#### 4. Proactive Assessment (Beta.271-279)
- **Correlated issue detection** across time
- **Health score calculation** (0-100)
- **Integration** with diagnostic engine

#### 5. CLI Interface (`annactl`)

**Two Commands Only:**

Anna has exactly two CLI entry points:

**1. Status Command:**
```bash
annactl status
```
Shows:
- Daemon and LLM status
- Top 3 system issues (from diagnostic engine)
- Recent logs
- JSON output available with `--json` flag

**2. Ask Anna (One-Shot Queries):**
```bash
annactl "my DNS is broken"
annactl "nginx keeps crashing"
annactl "what's using disk space?"
```

**How Anna Responds:**

When you ask a question, Anna will:
1. **Analyze your system** - Fetch telemetry from the daemon
2. **Consult Arch Wiki** - Find authoritative guidance for detected issues
3. **Propose a plan** - Show step-by-step commands with explanations
4. **Present commands clearly** - List all commands that would be run
5. **Ask for confirmation** - End with: "Do you want me to run it for you?? y/N"

Example response structure:
```
You requested help fixing DNS resolution on your Arch system: "my DNS is broken"

Anna detected:
- Network is reachable
- DNS resolution suspected broken

Based on Arch Wiki guidance:
- https://wiki.archlinux.org/title/Systemd-resolved

This plan follows the safe pattern: inspect first (4 steps), then propose changes (1 steps).
All changes require confirmation and have rollback commands if needed.

This is what we need to run:
systemctl status systemd-resolved.service
journalctl -u systemd-resolved.service -n 50
cat /etc/resolv.conf
resolvectl query archlinux.org
sudo systemctl restart systemd-resolved.service

Do you want me to run it for you?? y/N
```

**What Anna Can Plan (6.4.x):**
- DNS resolution troubleshooting (systemd-resolved)
- Failed systemd service troubleshooting
- More scenarios coming in future releases

For other questions, Anna falls back to LLM-generated answers (requires Ollama).

---

## ❌ What Does NOT Work in 6.0.0

- **No interactive TUI** - The 5.x terminal UI is disabled
- **No panels** - Right panel, brain panel, etc. are archived
- **No streaming UI** - CLI queries return complete answers only
- **No fancy workflows** - Focus is on working core features

---

## Quick Start

### Requirements

- **OS:** Arch Linux (native) or Arch-based distros
- **CPU:** x86_64
- **RAM:** 8GB recommended (4GB minimum)
- **Disk:** 2GB for Anna + 4GB for LLM models
- **LLM:** Ollama with a model (qwen2.5:3b or similar)

### Installation

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

### Usage

**Start the daemon:**
```bash
sudo systemctl start annad
sudo systemctl enable annad  # Optional: start on boot
```

**Check system health:**
```bash
annactl status
```

Example output:
```
[DAEMON STATUS]
✓ annad running (PID 1234)
✓ LLM available (ollama/qwen2.5:3b)

[TOP ISSUES]
1. ⚠ Disk usage at 85% on /
2. ⚠ Service NetworkManager failed
3. ℹ 12 orphaned packages detected

[RECENT LOGS]
2025-11-23 14:32:01 - Health check completed
2025-11-23 14:30:00 - Historian snapshot saved
```

**Ask questions:**
```bash
annactl "why did NetworkManager fail?"
annactl "show disk usage breakdown"
```

---

## Architecture

Anna 6.0.0 consists of:

**Daemon (`annad`):**
- Runs as systemd service
- Collects telemetry
- Performs health checks via diagnostic engine
- Stores history in Historian
- Executes approved actions
- RPC server (Unix socket)

**Client (`annactl`):**
- Simple CLI for `status` command
- One-shot natural language queries
- RPC client to daemon

**Intelligence:**
- Internal diagnostic engine (9 deterministic rules)
- Historian (temporal correlation, 6 rules)
- Proactive Assessment (issue correlation)
- 77+ deterministic recipes
- Local LLM via Ollama (for natural language)

---

## Documentation

- **[Architecture](docs/ARCHITECTURE_BETA_200.md)** - System design overview
- **[Beta.279 Notes](docs/BETA_279_NOTES.md)** - Historian implementation
- **[Changelog](CHANGELOG.md)** - Version history
- **[Release Notes 6.0.0](RELEASE_NOTES_6.0.0.md)** - What changed in 6.0.0

---

## Core Principles

- **Local-First:** All data stays on your machine
- **Telemetry-First:** Real system facts, not LLM guesses
- **Transparent:** Every command shown before execution
- **Deterministic:** Predictable, reproducible actions
- **Safe:** Approval required for system changes
- **Arch-Focused:** Deep Arch Linux integration

---

## What Anna is NOT

- ❌ Not production-ready (experimental prototype)
- ❌ Not a remote management tool
- ❌ Not a fully autonomous agent
- ❌ Not a generic chatbot
- ❌ Not a replacement for system knowledge

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
./target/release/annactl status
```

---

## Project Status

**Current:** 6.0.0 - Prototype Reset (CLI-only, stable daemon)

**Recent milestones:**
- ✅ 6.0.0 - Disabled TUI, cleaned repository, focused on stable CLI
- ✅ Beta.279 - Historian v1 with JSONL storage and 6 temporal rules
- ✅ Beta.271 - Proactive Assessment integration
- ✅ Beta.250 - Canonical diagnostic formatting
- ✅ 77+ deterministic recipes
- ✅ Full RPC integration

**Future roadmap:**
- Rebuild TUI as a stable feature
- Network diagnostics expansion
- Configuration validation
- Hardware monitoring
- Recipe expansion

---

## Contributing

Anna is in active development. Contributions welcome!

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
