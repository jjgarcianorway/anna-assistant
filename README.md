# Anna v0.12.1

**Your Intelligent Linux Assistant**

Anna is a two-LLM system that provides reliable, evidence-based answers about your Linux system. Zero hallucinations. Only facts from probes.

## What's New in v0.12.1

- **Flexible JSON Parsing** - Handles null draft_answer, null text, missing fields gracefully
- **Clearer LLM-A Prompts** - Explicit instruction to produce draft_answer when evidence exists
- **Evidence-Aware Prompting** - User prompt now explicitly tells LLM-A when evidence is available
- **Better Defaults** - Approves with low confidence on parse errors instead of refusing

## Previous in v0.12.0

- **Strict Probe Catalog** - Hard-frozen 14-probe catalog embedded in prompts, no invented probes
- **fix_and_accept Verdict** - LLM-B can fix minor issues without requesting new probes
- **Partial Answer Fallback** - Returns honest low-confidence answers instead of total refusal
- **Scoring Formula Update** - overall = min(evidence, reasoning, coverage) for stricter evaluation

## Previous in v0.11.0

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
- **Reliability Scoring** - Transparent scoring with confidence levels
- **Confidence Levels** - GREEN (>=0.90), YELLOW (0.70-0.90), RED (<0.70)

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
         |(14 tools)| |  Store   | | (Learn)  |
         +----------+ +----------+ +----------+
```

## v0.12.0 LLM Protocol

### Hard-Frozen Probe Catalog

| probe_id             | description                      | cost   |
|----------------------|----------------------------------|--------|
| cpu.info             | CPU info from lscpu              | cheap  |
| mem.info             | Memory from /proc/meminfo        | cheap  |
| disk.lsblk           | Block devices from lsblk         | cheap  |
| fs.usage_root        | Root filesystem usage (df /)     | cheap  |
| net.links            | Network link status (ip link)    | cheap  |
| net.addr             | Network addresses (ip addr)      | cheap  |
| net.routes           | Routing table (ip route)         | cheap  |
| dns.resolv           | DNS config (/etc/resolv.conf)    | cheap  |
| pkg.pacman_updates   | Available pacman updates         | medium |
| pkg.yay_updates      | Available AUR updates            | medium |
| pkg.games            | Game packages (steam/lutris/wine)| medium |
| system.kernel        | Kernel info (uname -a)           | cheap  |
| system.journal_slice | Recent journal entries           | medium |
| anna.self_health     | Anna daemon health check         | cheap  |

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
| **LLM-B** | Auditor/Skeptic. Verifies evidence grounding, can fix or request more probes. |

## Core Principles

1. **Zero hardcoded knowledge** - Only facts from probe catalog
2. **100% reliability** - No hallucinations, no guesses
3. **Evidence-based** - Every claim must have a citation
4. **Hard-frozen probe catalog** - Only 14 registered probes allowed
5. **Partial answers over refusal** - Honest confidence better than blocking
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

This is version 0.12.1 - Flexible JSON parsing, clearer prompts, better defaults.
