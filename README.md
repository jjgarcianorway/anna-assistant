# Anna v0.11.0

**Your Intelligent Linux Assistant**

Anna is a two-LLM system that provides reliable, evidence-based answers about your Linux system. Zero hallucinations. Only facts from probes.

## What's New in v0.11.0

- **Knowledge Store** - SQLite-backed persistent fact storage with entity/attribute/value model
- **Event-Driven Learning** - Watches pacman.log for package changes, triggers learning jobs
- **System Mapping** - Initial hardware/software discovery on first install
- **User Telemetry** - Tracks query topics to prioritize learning (local only, private)
- **Knowledge Hygiene** - Detects stale/conflicting facts, schedules revalidation
- **Knowledge API** - New `/v1/knowledge/query` and `/v1/knowledge/stats` endpoints

## Previous in v0.10.0

- **Evidence-Based Answer Engine** - LLM-A/LLM-B supervised audit loop with strict JSON protocol
- **Probe Catalog** - 14 registered probes with cost estimation (cheap/medium/expensive)
- **Strict Evidence Discipline** - Every answer must cite probe evidence
- **Reliability Scoring** - overall = 0.4×evidence + 0.3×reasoning + 0.3×coverage
- **Confidence Levels** - GREEN (≥0.90), YELLOW (0.70-0.90), RED (<0.70 = refuse)

## Previous in v0.9.0

- **Locked CLI Surface** - Only 5 commands: REPL, request, status, version, help
- **Status Command** - `annactl status` shows daemon, LLM, update state, and self-health
- **Case-Insensitive Commands** - version/VERSION/Version all work, same for help/status

## Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   annactl    │────▶│    LLM-A     │────▶│    LLM-B     │
│  (CLI UI)    │     │  Planner/    │     │   Auditor/   │
│              │     │  Answerer    │     │   Skeptic    │
└──────────────┘     └──────────────┘     └──────────────┘
                           │                    │
                           ▼                    │
                    ┌──────────────┐            │
                    │    annad     │◀───────────┘
                    │   (Daemon)   │
                    └──────────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
       ┌──────────┐ ┌──────────┐ ┌──────────┐
       │  Probes  │ │Knowledge │ │  Brain   │
       │(14 tools)│ │  Store   │ │ (Learn)  │
       └──────────┘ └──────────┘ └──────────┘
```

## v0.11.0 Knowledge Store

Anna now learns about your system and remembers facts persistently:

```
Knowledge Store Schema:
┌───────────────────────────────────────────────────┐
│ Fact                                              │
├───────────────────────────────────────────────────┤
│ entity: "cpu:0" / "pkg:vim" / "net:wlp0s20f3"     │
│ attribute: "cores" / "version" / "state"          │
│ value: "8" / "9.1.0" / "UP"                       │
│ source: "probe:cpu.info:2025-11-27T..."           │
│ confidence: 0.95                                   │
│ status: Active / Stale / Deprecated / Conflicted  │
└───────────────────────────────────────────────────┘
```

### Entity Types

| Prefix | Description |
|--------|-------------|
| `cpu:` | CPU information (cores, model, architecture) |
| `gpu:` | GPU information (vendor, description) |
| `disk:` | Block devices (size, type) |
| `fs:` | Filesystems (device, mountpoint) |
| `pkg:` | Packages (installed, version) |
| `svc:` | Services (state) |
| `net:` | Network interfaces (state) |
| `system:` | System info (kernel, distro, memory) |
| `user:` | User context (shell, editor, home) |
| `desktop:` | Desktop environment (name, session_type) |

## Usage

```bash
# Start interactive REPL
annactl

# Ask a question (one-shot)
annactl "How many CPU cores do I have?"

# Show status (daemon, LLM, update state, self-health)
annactl status

# Show version (includes update status)
annactl -V
annactl --version
annactl version       # Case-insensitive

# Show help
annactl -h
annactl --help
annactl help          # Case-insensitive
```

**That's it.** The CLI surface is locked. No other commands exist.

## Probe Catalog (v0.10.0)

| Probe ID | Description | Cost |
|----------|-------------|------|
| `cpu.info` | CPU information from lscpu | cheap |
| `mem.info` | Memory usage from /proc/meminfo | cheap |
| `disk.lsblk` | Block devices from lsblk | cheap |
| `fs.usage_root` | Root filesystem usage | cheap |
| `net.links` | Network interface link status | cheap |
| `net.addr` | Network interface addresses | cheap |
| `net.routes` | Network routing table | cheap |
| `dns.resolv` | DNS resolver configuration | cheap |
| `pkg.pacman_updates` | Available pacman updates | medium |
| `pkg.yay_updates` | Available AUR updates | medium |
| `pkg.games` | Installed game packages | medium |
| `system.kernel` | Kernel and system info | cheap |
| `system.journal_slice` | Recent journal entries | medium |
| `anna.self_health` | Anna self-health check | cheap |

## Natural Language Configuration

Configure Anna by talking to her - no manual config editing needed:

```bash
# Enable dev auto-update every 10 minutes
annactl "enable dev auto-update every 10 minutes"

# Switch to a specific model
annactl "switch to manual model selection and use qwen2.5:14b"

# Go back to automatic model selection
annactl "go back to automatic model selection"

# Disable auto-update
annactl "turn off auto update"

# Show current configuration
annactl "show me your current configuration"
```

### Config Schema

Configuration is stored in `~/.config/anna/config.toml`:

```toml
[core]
mode = "normal"           # normal or dev

[llm]
preferred_model = "llama3.2:3b"
fallback_model = "llama3.2:3b"
selection_mode = "auto"   # auto or manual

[update]
enabled = false
interval_seconds = 86400  # Minimum 600 (10 minutes)
channel = "main"          # main, stable, beta, or dev
```

## Components

| Component | Role |
|-----------|------|
| **annad** | Evidence Oracle. Executes probes, orchestrates LLM-A/LLM-B loop, manages knowledge store. |
| **annactl** | CLI wrapper. Clean output with citations and confidence. |
| **LLM-A** | Planner/Answerer. Plans probes, produces draft answers, self-scores. |
| **LLM-B** | Auditor/Skeptic. Verifies evidence grounding, approves/refuses/requests more. |

## Core Principles

1. **Zero hardcoded knowledge** - Only facts from probe catalog
2. **100% reliability** - No hallucinations, no guesses
3. **Evidence-based** - Every claim must have a citation
4. **70% threshold** - Below 70% overall score = refuse to answer
5. **Tool catalog enforcement** - Only registered probes allowed
6. **Supervised audit loop** - LLM-B validates LLM-A's work
7. **Learn from THIS machine** - Knowledge store captures facts from your system

## Installation

### Quick Install (curl)

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

### Build from Source

```bash
cargo build --release
sudo ./scripts/install.sh
```

### Start the Daemon

```bash
sudo systemctl start annad
sudo systemctl enable annad  # Enable at boot
annactl -V                   # Verify
```

## Requirements

- Linux (x86_64 or aarch64)
- Rust 1.70+
- [Ollama](https://ollama.ai) for LLM inference

## License

GPL-3.0-or-later

## Contributing

This is version 0.11.0 - Knowledge store, event-driven learning, and user telemetry.
