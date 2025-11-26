# Anna v0.2.0

**Your Intelligent Linux Assistant**

Anna is a two-LLM system that provides reliable, evidence-based answers about your Linux system. Zero hallucinations. Only facts from probes.

## What's New in v0.2.0

- 🔒  **Strict Evidence Discipline** - Every claim must cite its source
- 🛡️  **Tool Catalog Enforcement** - No hallucinated probes allowed
- 📊  **Reliability Scoring** - Transparent confidence with breakdown
- 🔄  **Auto-Update** - Anna can update herself automatically

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

## Components

| Component | Role |
|-----------|------|
| **annad** | Evidence Oracle. Runs probes, provides raw JSON. Never interprets. |
| **annactl** | CLI wrapper. Talks to LLM-A only. Provides clean output. |
| **LLM-A** | Orchestrator. Parses intent, requests probes, builds responses. |
| **LLM-B** | Expert validator. Verifies reasoning, catches errors, computes confidence. |

## Core Principles

1. **Zero hardcoded knowledge** - Only facts from probes
2. **100% reliability** - No hallucinations, no guesses
3. **Evidence-based** - Every claim must have a source
4. **Transparent confidence** - Always show certainty level
5. **Tool catalog enforcement** - Only registered probes allowed

## Installation

### Quick Install (curl)

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | sudo bash
```

### Manual Install

```bash
# Download binaries
wget https://github.com/jjgarcianorway/anna-assistant/releases/download/v0.2.0/annad-0.2.0-x86_64-unknown-linux-gnu
wget https://github.com/jjgarcianorway/anna-assistant/releases/download/v0.2.0/annactl-0.2.0-x86_64-unknown-linux-gnu

# Install
sudo mv annad-0.2.0-x86_64-unknown-linux-gnu /usr/local/bin/annad
sudo mv annactl-0.2.0-x86_64-unknown-linux-gnu /usr/local/bin/annactl
sudo chmod +x /usr/local/bin/annad /usr/local/bin/annactl

# Initialize
annactl init
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
annactl status               # Verify
```

## Usage

```bash
# Ask a question
annactl "How many CPU cores do I have?"

# Or explicitly
annactl ask "What's my RAM usage?"

# Check status
annactl status

# List probes
annactl probes

# Run specific probe
annactl probe cpu.info

# Check for updates
annactl check-update

# Update Anna
annactl update

# Show version
annactl version
```

## Auto-Update

Anna can update herself automatically. Updates are checked in the background (once per hour).

```bash
# Check for updates manually
annactl check-update

# Update to latest version
annactl update

# Use beta channel
annactl check-update --beta
annactl update --beta

# Skip update check on startup
annactl --no-update-check "your question"
```

## Probes

| Probe | Description | Cache |
|-------|-------------|-------|
| `cpu.info` | CPU information from /proc/cpuinfo | STATIC |
| `mem.info` | Memory usage from /proc/meminfo | VOLATILE (5s) |
| `disk.lsblk` | Disk information from lsblk | SLOW (1h) |

## Evidence Discipline

Anna v0.2.0 enforces strict evidence discipline:

- **Every claim must cite its source** - `[source: cpu.info]`
- **Only registered probes allowed** - No hallucinated tools
- **Reliability scoring** - Breakdown of confidence
  - Evidence quality (40%)
  - Reasoning quality (30%)
  - Coverage (30%)
- **Hallucination detection** - LLM-B catches unsupported claims

## LLM Model Selection

Anna automatically selects models based on your hardware:

**LLM-A (Orchestrator):**
- Default: `llama3.2:3b`
- With GPU: `mistral-nemo-instruct`
- 8+ CPU cores: `qwen2.5:3b`

**LLM-B (Expert):**
- ≤16GB RAM: `qwen2.5:7b`
- 16-32GB RAM: `qwen2.5:14b`
- ≥32GB + GPU: `qwen2.5:32b`

## Requirements

- Linux (x86_64 or aarch64)
- Rust 1.70+
- [Ollama](https://ollama.ai) for LLM inference

## License

GPL-3.0-or-later

## Contributing

This is version 0.2.0 - Evidence discipline and auto-update release.
