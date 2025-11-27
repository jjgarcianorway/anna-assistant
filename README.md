# Anna v0.5.0

**Your Intelligent Linux Assistant**

Anna is a two-LLM system that provides reliable, evidence-based answers about your Linux system. Zero hallucinations. Only facts from probes.

## What's New in v0.5.0

- 🗣️  **Natural Language Configuration** - Configure Anna by talking to her
- 🧠  **Hardware-Aware Model Selection** - Automatically picks the right model for your hardware
- 🔒  **GPU Driver Detection** - Safe fallback when GPU exists but drivers are missing
- 🔄  **Dev Auto-Update** - 600 second minimum interval, config-driven

## What's in v0.4.0

- 🔄  **Dev Auto-Update** - Automatic updates every 10 minutes in dev mode
- 📊  **Update Status in Version/Help** - Channel, mode, last check info
- ⚙️  **Config-Driven Updates** - No new CLI commands, all via config

## Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   annactl    │────▶│    LLM-A     │────▶│    LLM-B     │
│  (CLI UI)    │     │ Orchestrator │     │   Expert     │
└──────────────┘     └──────────────┘     └──────────────┘
                           │                    │
                           ▼                    │
                    ┌──────────────┐            │
                    │    annad     │◀───────────┘
                    │   (Daemon)   │
                    └──────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │    Probes    │
                    │ (Evidence)   │
                    └──────────────┘
```

## Usage

```bash
# Ask a question
annactl "How many CPU cores do I have?"

# Start interactive REPL
annactl

# Show version (includes update status)
annactl -V
annactl --version

# Show help
annactl -h
annactl --help
```

**That's it.** No other commands exist.

## Natural Language Configuration (v0.5.0)

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

Under the hood, configuration is stored in `~/.config/anna/config.toml`:

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

### Hardware-Aware Model Selection

Anna automatically selects the appropriate model based on your hardware:

| Condition | Model Selected |
|-----------|---------------|
| GPU with drivers | `qwen2.5:14b` or `qwen2.5:32b` |
| GPU without drivers | `llama3.2:3b` (safe fallback) |
| High-performance CPU | `qwen2.5:7b` |
| Standard CPU | `llama3.2:3b` |

When GPU drivers become available, Anna can automatically upgrade to a larger model (in dev mode) or recommend an upgrade (in normal mode).

### Version Output

```
Anna Assistant v0.5.0
Mode: normal [source: config.core]
Update: manual (main, every 86400s) [source: config.update]
LLM:
  selection_mode: auto [source: config.llm]
  active_model: llama3.2:3b [source: config.llm]
  fallback_model: llama3.2:3b [source: config.llm]
  hardware_recommendation: Standard CPU system [source: hardware.profile]
Daemon: running (v0.5.0, uptime: 3600s, 6 probes) [source: system.version]
Tool catalog: 6 probes registered [source: system.version]
```

## Components

| Component | Role |
|-----------|------|
| **annad** | Evidence Oracle. Runs probes, provides raw JSON. Never interprets. |
| **annactl** | CLI wrapper. Talks to LLM-A only. Provides clean output. |
| **LLM-A** | Orchestrator. Parses intent, requests probes, builds responses. |
| **LLM-B** | Expert validator. Verifies reasoning, catches hallucinations, computes reliability. |

## Core Principles

1. **Zero hardcoded knowledge** - Only facts from probes
2. **100% reliability** - No hallucinations, no guesses
3. **Evidence-based** - Every claim must have a source
4. **70% threshold** - Below 70% reliability = insufficient evidence
5. **Tool catalog enforcement** - Only registered probes allowed
6. **Stability check** - Run twice, reconcile if different

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

## Probes

| Probe | Description | Cache |
|-------|-------------|-------|
| `cpu.info` | CPU information from /proc/cpuinfo | STATIC |
| `mem.info` | Memory usage from /proc/meminfo | VOLATILE (5s) |
| `disk.lsblk` | Disk information from lsblk | SLOW (1h) |
| `hardware.gpu` | GPU hardware detection via lspci | SLOW (1h) |
| `drivers.gpu` | GPU driver status from kernel modules | SLOW (1h) |
| `hardware.ram` | RAM information | SLOW (1h) |

## Domains WITHOUT Probes (Cannot Answer)

- Network/WiFi/IP - No network.info probe
- Packages/Software - No package.info probe
- Processes/Services - No process.info probe
- Users/Accounts - No user.info probe

## Requirements

- Linux (x86_64 or aarch64)
- Rust 1.70+
- [Ollama](https://ollama.ai) for LLM inference

## License

GPL-3.0-or-later

## Contributing

This is version 0.4.0 - Dev auto-update release.
