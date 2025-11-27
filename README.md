# Anna v0.14.0

**Your Intelligent Linux Assistant**

Anna is a two-LLM system that provides reliable, evidence-based answers about your Linux system. Zero hallucinations. Only facts from probes.

## What's New in v0.14.0

- **Aligned to Reality** - Probe catalog shrunk from 14 to 6 actual working probes
- **Explicit Unsupported Domains** - Honest refusal for network, packages, kernel (no probes yet)
- **Stronger Evidence Discipline** - "If no probe, you do not know" enforced in prompts
- **Cleaner Heuristics Separation** - Heuristics clearly marked, evidence <= 0.4 when used
- **Auto-Update Enabled by Default** - Fresh installs have auto-update on (v10.4.1)
- **Fixed Installer** - Proper architecture detection, tarball-based installation

## v0.14.0 Probe Catalog (6 Real Probes)

| probe_id      | description                                    | cache  |
|---------------|------------------------------------------------|--------|
| cpu.info      | CPU info (model, threads, flags) from lscpu    | STATIC |
| mem.info      | Memory from /proc/meminfo (RAM in kB)          | STATIC |
| disk.lsblk    | Block devices from lsblk -J                    | STATIC |
| hardware.gpu  | GPU presence and basic model/vendor            | STATIC |
| drivers.gpu   | GPU driver stack summary                       | STATIC |
| hardware.ram  | High level RAM summary (total, slots)          | STATIC |

### Unsupported Domains (No Probes Yet)

- Network status, WiFi, DNS
- Package installation state, updates
- Desktop environment, window manager
- Config file locations (Hyprland, VS Code, etc.)
- Per-folder/file disk usage
- System logs, kernel version

Questions in these areas get honest "no probe for this" responses with optional heuristics.

## Previous Versions

<details>
<summary>v0.13.0 - Strict Evidence Discipline</summary>

- No hardcoded knowledge - evidence only
- Intent mapping for common questions
- Explicit "no probe → you do not know" rule

</details>

<details>
<summary>v0.12.x - Iteration-Aware Prompts</summary>

- LLM-A must answer on iteration 2+
- Fallback answer extraction from evidence
- fix_and_accept verdict

</details>

<details>
<summary>v0.11.0 - Knowledge Store</summary>

- SQLite-backed fact storage
- Event-driven learning framework
- System mapping phases

</details>

<details>
<summary>v0.10.0 - Evidence-Based Engine</summary>

- LLM-A/LLM-B supervised audit loop
- Reliability scoring (GREEN/YELLOW/RED)
- Probe catalog system

</details>

## Architecture

```
+----------------+     +----------------+     +----------------+
|   annactl      |---->|    LLM-A       |---->|    LLM-B       |
|  (CLI UI)      |     |  Planner/      |     |   Auditor/     |
|                |     |  Answerer      |     |   Skeptic      |
+----------------+     +----------------+     +----------------+
                             |                      |
                             v                      |
                      +----------------+            |
                      |    annad       |<-----------+
                      |   (Daemon)     |
                      +----------------+
                             |
                +------------+------------+
                v            v            v
         +----------+ +----------+ +----------+
         |  Probes  | |Knowledge | |  Brain   |
         | (6 tools)| |  Store   | | (Learn)  |
         +----------+ +----------+ +----------+
```

## LLM Protocol

### LLM-B Verdicts

| Verdict          | Meaning                                           |
|------------------|---------------------------------------------------|
| approve          | Answer is adequately grounded, deliver as-is      |
| fix_and_accept   | Minor issues fixed, use fixed_answer              |
| needs_more_probes| Specific catalog probes would improve answer      |
| refuse           | No catalog probes can help (very rare)            |

### Scoring

```
overall = min(evidence, reasoning, coverage)

>= 0.90: GREEN  (high confidence) - approve
>= 0.70: YELLOW (medium)          - approve or fix_and_accept
<  0.70: RED    (low)             - partial answer with disclaimer
```

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
enabled = true            # Auto-update enabled by default (v0.14.0)
interval_seconds = 86400  # Minimum 600 (10 minutes)
channel = "main"          # main, stable, beta, or dev
```

## Components

| Component | Role |
|-----------|------|
| **annad** | Evidence Oracle. Executes probes, orchestrates LLM-A/LLM-B loop, manages knowledge store. |
| **annactl** | CLI wrapper. Clean output with citations and confidence. |
| **LLM-A** | Planner/Answerer. Plans probes, produces draft answers, self-scores. |
| **LLM-B** | Auditor/Skeptic. Verifies evidence grounding, can fix or request more probes. |

## Core Principles

1. **Zero hardcoded knowledge** - Only facts from the 6 real probes
2. **100% reliability** - No hallucinations, no guesses
3. **Evidence-based** - Every claim must have a citation
4. **Aligned to reality** - Only 6 probes that actually exist
5. **Honest about limitations** - Clear "no probe for this" when unsupported
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

This is version 0.14.0 - Aligned to reality with 6 real probes, honest unsupported domain handling.
