# Anna v0.3.0

**Your Intelligent Linux Assistant**

Anna is a two-LLM system that provides reliable, evidence-based answers about your Linux system. Zero hallucinations. Only facts from probes.

## What's New in v0.3.0

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

# Show version (via LLM pipeline)
annactl -V
annactl --version

# Show help (via LLM pipeline)
annactl -h
annactl --help
```

**That's it.** No other commands exist.

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

## Hallucination Guardrails (v0.3.0)

If Anna does not have a probe for a domain:

- Immediate return: "Insufficient evidence"
- Reliability < 70% = red warning
- Explicit list of missing probes:
  - "No gpu.info probe available"
  - "No network.info probe available"
  - "No package.info probe available"

## Stable Repeated Answers (v0.3.0)

Every question runs twice through the LLM pipeline:

- If answers match: +10% stability bonus
- If answers differ: reconciliation → +5% stability bonus
- Stability status shown in output

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

This is version 0.3.0 - Strict hallucination guardrails release.
