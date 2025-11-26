# Anna v0.0.1

**Your Intelligent Linux Assistant**

Anna is a two-LLM system that provides reliable, evidence-based answers about your Linux system. Zero hallucinations. Only facts from probes.

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

## Installation

### Quick Install (curl)

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | sudo bash
```

### Manual Install

```bash
# Download binaries
wget https://github.com/jjgarcianorway/anna-assistant/releases/download/v0.0.1/annad-0.0.1-x86_64-unknown-linux-gnu
wget https://github.com/jjgarcianorway/anna-assistant/releases/download/v0.0.1/annactl-0.0.1-x86_64-unknown-linux-gnu

# Install
sudo mv annad-0.0.1-x86_64-unknown-linux-gnu /usr/local/bin/annad
sudo mv annactl-0.0.1-x86_64-unknown-linux-gnu /usr/local/bin/annactl
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
```

## Probes

| Probe | Description | Cache |
|-------|-------------|-------|
| `cpu.info` | CPU information from /proc/cpuinfo | STATIC |
| `mem.info` | Memory usage from /proc/meminfo | VOLATILE (5s) |
| `disk.lsblk` | Disk information from lsblk | SLOW (1h) |

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

This is version 0.0.1 - the clean room reboot. Architecture is intentionally minimal and strict.
