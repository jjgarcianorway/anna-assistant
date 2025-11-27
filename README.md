# Anna v0.4.0

**Your Intelligent Linux Assistant**

Anna is a two-LLM system that provides reliable, evidence-based answers about your Linux system. Zero hallucinations. Only facts from probes.

## What's New in v0.4.0

- 🔄  **Dev Auto-Update** - Automatic updates every 10 minutes in dev mode
- 📊  **Update Status in Version/Help** - Channel, mode, last check info
- ⚙️  **Config-Driven Updates** - No new CLI commands, all via config

## What's in v0.3.0

- 🛡️  **Strict Hallucination Guardrails** - Zero tolerance for unsupported claims
- 🔄  **Stable Repeated Answers** - Reconciliation when answers differ
- 📊  **70% Reliability Threshold** - Below 70% = insufficient evidence
- 🤖  **LLM-Orchestrated Help/Version** - Even help/version uses the evidence pipeline

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

# Show help (mentions auto-update config)
annactl -h
annactl --help
```

**That's it.** No other commands exist.

## Auto-Update (v0.4.0)

Anna can automatically update itself. Configuration is done via config file, not CLI:

### Config Location

- User config: `~/.config/anna/config.toml`
- System config: `/etc/anna/config.toml`

### Config Options

```toml
[update]
channel = "stable"        # stable, beta, or dev
auto = false              # Enable auto-updates
interval_seconds = 86400  # Check interval (optional)
```

### Channels

| Channel | Default Interval | Description |
|---------|-----------------|-------------|
| `stable` | 24 hours | Production releases only |
| `beta` | 12 hours | Pre-release versions |
| `dev` | 10 minutes | Development versions |

### Dev Mode Auto-Update

To enable automatic updates every 10 minutes:

```toml
[update]
channel = "dev"
auto = true
```

When enabled:
- Checks for new versions every 10 minutes
- Downloads and verifies binaries atomically
- Restarts daemon automatically
- Rolls back on failure

### Version Output

```
Anna Assistant v0.4.0
Channel: stable
Update mode: manual
Last update check: 2025-01-15 10:30:00 UTC
Last update result: ok
Daemon: running (v0.4.0, uptime: 3600s, 3 probes)
Model: llama3.2:3b
Tool catalog: 3 probes registered
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

## Domains WITHOUT Probes (Cannot Answer)

- GPU/Graphics - No gpu.info probe
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
