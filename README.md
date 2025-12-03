# Anna Assistant v0.0.11

**Local-first Virtual Senior Sysadmin for Arch Linux**

Anna is a natural language assistant that answers questions, executes requests safely, monitors your system proactively, and continuously learns from interactions.

**v0.0.11**: Safe Auto-Update System. Update channels (stable/canary), safe download with integrity verification, atomic installation with rollback, zero-downtime restart via systemd, complete state visibility in status.

---

## Mission

Anna does exactly three things, and does them extremely well:

1. **Answers questions** about your machine, OS, and computing topics with reliability scores and evidence citations
2. **Monitors proactively** via telemetry, reporting issues and anomalies before you notice them
3. **Learns continuously** by creating and evolving recipes from solved problems

---

## CLI Surface (Strict)

```bash
# One-shot natural language request
annactl "what CPU do I have?"
annactl "is nginx running?"
annactl "why is my system slow?"

# Interactive REPL mode
annactl

# Self-status (version, permissions, update state, helpers)
annactl status

# Version
annactl --version
annactl -V
```

**That's the entire public surface.** No other commands are exposed.

---

## The IT Department (4-Player Model)

Every interaction involves four participants:

| Role | Description |
|------|-------------|
| **User** | You. Asks questions in natural language |
| **Anna** | Primary assistant and orchestrator. Starts as "intern", becomes elite over time |
| **Translator** | Converts user intent to structured internal request plans |
| **Junior** | Verifies Anna's answers, attempts improvements, produces reliability score |
| **Senior** | Slower, wiser. Junior escalates after unsuccessful improvement rounds |

### Debug Mode (Always On)

All internal dialogue is visible:

```
[you] to [anna]: what CPU do I have?
[anna] to [translator]: parse user intent
[translator] to [anna]: query type: hardware.cpu, detail: model
[anna] to [annad]: retrieve CPU info from snapshot
[annad] to [anna]: AMD Ryzen 9 5900X (12 cores, 24 threads)
[anna] to [junior]: validate answer: "AMD Ryzen 9 5900X, 12 cores"
[junior] to [anna]: confidence 94% - answer verified against /proc/cpuinfo
[anna] to [you]: You have an AMD Ryzen 9 5900X with 12 cores (24 threads).
                 Reliability: 94%
```

### Reliability Scores

Every answer includes a reliability percentage (0-100%) based on:
- Evidence quality
- Repeatability
- Risk assessment

---

## Architecture

```
annactl (CLI)
    |
    +-- Natural language interface
    +-- REPL mode
    +-- Status display
    |
annad (root daemon)
    |
    +-- Telemetry gathering and snapshots
    +-- Safe execution with rollback
    +-- Self-update (every 10 minutes)
    +-- Local LLM orchestration (Ollama)
    +-- Recipe storage and learning
```

---

## Safety Policy

### Action Classification

| Category | Description | Confirmation |
|----------|-------------|--------------|
| **Read-only** | Safe observation commands | None required |
| **Low-risk** | Reversible, local changes | Simple y/n |
| **Medium-risk** | Config edits, service restarts, installs | Explicit confirmation |
| **High-risk** | Destructive, irreversible operations | "I assume the risk" + rollback plan |

### Evidence Requirement

No invented outputs. Every claim is backed by:
- Stored snapshots
- Command outputs
- Log excerpts
- Measured telemetry
- Clearly labeled inferences

### Rollback Mandate

Every mutation has rollback support via:
- Timestamped file backups
- btrfs snapshots (when available)
- Action logs
- Explicit rollback instructions

---

## Learning System

### Recipes

Anna creates recipes when:
- She needed help from Junior or Senior
- A new fix path was discovered
- A repeated question type is solved

Recipes are:
- Versioned
- Testable (dry-run when possible)
- Annotated with risk level
- Linked to evidence patterns

### XP and Levels

All players (Anna, Junior, Senior) have:
- Level 0-100
- Non-linear XP progression
- XP gained from correct answers and new recipes
- No XP loss (poor outcomes earn nothing)

Titles are nerdy, old-school, ASCII-friendly. No emojis or icons.

---

## Proactive Monitoring

Anna detects and reports:
- Boot regressions (slower boot times)
- System degradation correlated with recent changes
- Recurring journal warnings or crashes
- Thermal and power anomalies
- Network instability
- Disk I/O regressions
- Service failures

Example: "What have I installed in the last two weeks that might explain my machine feeling slower?"

---

## Self-Sufficiency

### Auto-Update (v0.0.11)

annad checks for updates every 10 minutes:
- Pings GitHub releases API
- Downloads artifacts with integrity verification
- Atomic installation with staging
- Zero-downtime restart via systemd
- Automatic rollback on failure
- Records state in `annactl status`

**Update Channels:**
- `stable` (default): Only stable tagged releases
- `canary`: Include pre-releases (alpha, beta, rc, canary)

**Configuration** (`/etc/anna/config.toml`):
```toml
[update]
mode = "auto"           # auto or manual
channel = "stable"      # stable or canary
interval_seconds = 600  # check every 10 minutes
```

**Safety Guarantees:**
- Never updates during active mutations
- Checks disk space before download
- Verifies installer review health before update
- Keeps previous binaries for rollback
- Atomic installation (no partial states)

### First-Run Setup

Anna automatically:
- Installs Ollama if needed
- Selects models based on hardware
- Downloads models with progress display
- Proceeds without user intervention

### Helper Tools

Anna may install tools for her operation (ethtool, lm_sensors, etc.):
- Listed in `annactl status`
- Tracked as "Anna-installed helpers"
- Removable on uninstall

### Clean Uninstall

```bash
annactl uninstall
```

- Shows list of Anna-installed helpers
- Asks whether to remove them
- Removes services, data, models
- Never leaves broken permissions

### Factory Reset

```bash
annactl reset
```

- Deletes learned recipes
- Removes Anna-installed helpers
- Resets internal DBs and state
- Keeps base binaries and service

---

## File Paths

| Path | Content |
|------|---------|
| `/etc/anna/config.toml` | Configuration |
| `/var/lib/anna/` | Data directory |
| `/var/lib/anna/knowledge/` | Object inventory |
| `/var/lib/anna/telemetry.db` | SQLite telemetry |
| `/var/lib/anna/recipes/` | Learned recipes |
| `/var/lib/anna/internal/snapshots/` | Daemon snapshots |
| `/var/lib/anna/internal/update_state.json` | Update scheduler |

---

## Installation

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

### Build from Source

```bash
git clone https://github.com/jjgarcianorway/anna-assistant.git
cd anna-assistant
cargo build --release
sudo install -m 755 target/release/annad /usr/local/bin/annad
sudo install -m 755 target/release/annactl /usr/local/bin/annactl
```

---

## Requirements

- **OS**: Arch Linux (x86_64)
- **Rust**: 1.70+ (for building)
- **Systemd**: For daemon management
- **Ollama**: Auto-installed on first run

---

## License

GPL-3.0-or-later

---

## Contributing

Issues and PRs welcome at: https://github.com/jjgarcianorway/anna-assistant

**Design Principles:**

1. Natural language first - no memorizing commands
2. Evidence-based - every claim traceable to real data
3. Safe by default - explicit confirmation for changes
4. Local only - no cloud, no external calls
5. Learns over time - recipes and XP progression
6. Transparent - debug mode shows all internal dialogue
7. Self-sufficient - auto-updates, auto-provisions models
