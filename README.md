# Anna Assistant v0.0.76

**Local-first Virtual Senior Sysadmin for Arch Linux**

[![CI](https://github.com/jjgarcianorway/anna-assistant/actions/workflows/ci.yml/badge.svg)](https://github.com/jjgarcianorway/anna-assistant/actions/workflows/ci.yml)

Anna is a natural language assistant that answers questions, executes requests safely, monitors your system proactively, and continuously learns from interactions.

**v0.0.76**: Semantic Version Fix - Fixed install script and auto-updater to find highest version by semver, not by release creation date. Previously, backfilling old releases caused them to be picked as "latest". Now fetches all releases and sorts by semantic version. Installer v7.43.0.

> **Supported Platform: Arch Linux only.** Other distributions are unsupported and untested.

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

# Self-status (version, daemon, helpers, models, updates)
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

### Fly-on-the-Wall Transcripts (v0.0.56)

All internal dialogue is visible with consistent actor voices:

```
=== Case: cpu-query-001 ===

----- intake -----
[you] to [anna]: what cpu do i have

----- triage -----
[anna] to [translator]: What are we looking at?
[translator] to [anna]: Clear SYSTEM_QUERY request. 95% confidence.
[anna] to [translator]: Acknowledged, I agree.

----- evidence -----
[anna] to [annad]: Run the probes.
[annad]: [E1] hw_snapshot_cpu -> AMD Ryzen 9 5900X, 12 cores, 24 threads
[annad] to [anna]: Evidence collected: [E1]

----- verification -----
[anna] to [junior]: Check this before I ship it.
[junior] to [anna]: Reliability 94%. Solid evidence. Ship it.
[anna] to [junior]: Good. Shipping response.

----- response -----
[anna] to [you]:
  You have an AMD Ryzen 9 5900X [E1] with 12 cores (24 threads).

Reliability: 94% - Verified.
(42ms)
```

**Actor Voices:**
- `[anna]` - Calm senior admin, concise, honest, pushes for evidence
- `[translator]` - Service desk triage, brisk, checklist-driven
- `[junior]` - Skeptical QA, calls out missing evidence, disagrees when warranted
- `[annad]` - Robotic/operational, terse, structured

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
    +-- Helper auto-installation
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

Titles are nerdy, old-school, ASCII-friendly. No icons.

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

### Auto-Update

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
- Keeps previous binaries for rollback
- Atomic installation (no partial states)

### First-Run Setup

Anna automatically:
- Installs Ollama if needed (records as anna-installed)
- Starts Ollama service
- Selects models based on hardware tier
- Downloads models with progress display
- Proceeds without user intervention

### Helper Tools (v0.0.28)

Anna auto-installs tools she needs for telemetry and diagnostics:
- **System-aware**: Only installs helpers relevant to your hardware
  - No ethtool if you have no ethernet
  - No nvme-cli if you have no NVMe drives
  - No iw if you have no WiFi
- Listed in `annactl status`
- Tracked with provenance (anna-installed vs user-installed)
- Only anna-installed helpers are removable on uninstall

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

## Status Display (v0.0.28)

`annactl status` shows:

| Section | Content |
|---------|---------|
| VERSION | Anna version |
| DAEMON | Running/stopped, uptime, PID |
| SNAPSHOT | Last data snapshot age |
| HEALTH | Overall system health, alerts |
| DATA | Knowledge objects, scan times |
| UPDATES | Mode, channel, last check, available updates |
| ALERTS | Critical/warning/info counts, recent alerts |
| HELPERS | Relevant helpers only, presence, provenance |
| LEARNING | Recipes, sessions, memory |
| KNOWLEDGE | Knowledge packs, documents |
| Q&A TODAY | Answer count, reliability, citations |
| PERFORMANCE | Latencies, cache hit rates |
| RELIABILITY | Error budgets, success rates |
| POLICY | Safety policy status, protected paths |
| MODELS | Ollama status, translator/junior models |
| RECENT ACTIONS | Tool executions, mutations, blocks |
| STORAGE | Disk usage by category |

---

## File Paths

| Path | Content |
|------|---------|
| `/etc/anna/config.toml` | Configuration |
| `/var/lib/anna/` | Data directory |
| `/var/lib/anna/knowledge/` | Object inventory |
| `/var/lib/anna/telemetry.db` | SQLite telemetry |
| `/var/lib/anna/recipes/` | Learned recipes |
| `/var/lib/anna/helpers.json` | Helper tracking |
| `/var/lib/anna/internal/snapshots/` | Daemon snapshots |
| `/var/lib/anna/internal/update_state.json` | Update scheduler |
| `/var/lib/anna/internal/bootstrap_state.json` | LLM model state |

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

- **OS**: **Arch Linux only** (x86_64) - other distributions are unsupported
- **Rust**: 1.70+ (for building from source)
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
8. System-aware - only shows/installs what's relevant

---

## CI/CD (v0.0.32)

All builds and tests run on **Arch Linux only**.

**CI Pipeline:**
- Build (debug + release) in Arch Linux container
- Unit tests and integration tests
- Clippy lints (advisory)
- Rustfmt check (advisory)
- Security audit (advisory)
- Smoke tests (--version, --help, status, natural language)
- Repo hygiene checks (version consistency, no legacy commands)
- Policy and redaction security tests

**Release Pipeline:**
- Triggered on `v*.*.*` tags
- Validates version consistency across all docs
- Builds Arch-compatible binaries
- Generates SHA256 checksums
- Creates GitHub release with notes

**Rules:**
- No green CI, no merge
- No release without updated docs
- No regressions allowed

---

## Recent Changes

### v0.0.75
- **Persistent Learning System**: Recipes now actually change future behavior
  - Reliability thresholds: 90% read-only, 80% doctor, 95% mutation
  - Evidence requirements: 1 (read-only), 2 (doctor), 3 (mutation)
  - Automatic recipe demotion after consecutive failures
  - Recipe coverage tracking for status display
- **RPG Stats in Status**: XP, levels, and metrics displayed in `annactl status`
  - Level 0-100 with titles (Intern→Apprentice→Technician→...→Grandmaster)
  - Request counters (total, success, failed)
  - Reliability metrics (average, rolling last-50)
  - Escalation percentages (junior, doctor, recipe-solved)
  - Latency metrics (median, p95)
  - By-domain breakdown
  - Progress bars for XP, success rate, recipe coverage
- **Transcript Polish**: Human mode vs debug mode fully separated
  - Human mode: No evidence IDs, no tool names
  - Evidence as descriptions ("Hardware snapshot: Intel i9-14900HX")
  - Confidence-based prefixes ("It looks like..." for medium confidence)
  - Debug mode: Full internals (evidence IDs, tool names, timing)
- **Integration Tests**: 22+ tests for learning + stats system

### v0.0.74
- **Direct Answers**: System queries return answers, not action plans
- **Classification Fix**: SYSTEM_QUERY checked before ACTION_REQUEST
- **Direct Answer Module**: Topic-based answer generation
- **Structured Doctor Outputs**: For transcript rendering
- **26 Integration Tests**: For direct answer system

### v0.0.57
- **Evidence Coverage Scoring**: Deterministic `EvidenceCoverage` checks evidence types against query targets
- **Target Facet Map**: Defines required fields per target (cpu needs `cpu_model`, disk_free needs `root_fs_free`)
- **Junior Rubric v2**: Wrong evidence caps at 20%, missing evidence caps at 50%, uncited claims penalized
- **Tool Routing Fix**: disk -> `mount_usage`, memory -> `memory_info`, kernel -> `kernel_version`, network -> `network_status`
- **Coverage Retry**: Automatic retry with correct tools when initial evidence misses required facets
- **Transcript Integration**: Coverage shown in fly-on-wall log when low (`[junior]: Coverage 30%...`)
- **Integration Tests**: 20+ tests asserting correct routing and evidence validation

### v0.0.56
- **Fly-on-the-Wall Dialogue Layer**: Transcripts feel like a real IT department
- **Actor Voices**: `[anna]` calm/honest, `[translator]` brisk/checklist-driven, `[junior]` skeptical QA
- **Agree/Disagree Moments**: Explicit acknowledgment after Translator and Junior
- **Doctor Handoffs**: Doctors render as departments (`[networking-doctor]`)
- **Phase Separators**: `----- triage -----`, `----- evidence -----`, etc.
- **Evidence Summaries**: `Evidence collected: [E1, E2, E3]`
- **QA Signoff**: Reliability footer with verdict
- **Golden Tests**: Transcript stability tests

### v0.0.41
- **Arch Boot Doctor v1 (Slow Boot + Service Regressions)**: Diagnoses slow boot causes
- **Evidence Bundle**: systemd-analyze time/blame/critical-chain, enabled units, journal errors, recent changes
- **Deterministic Diagnosis Flow**: 5 steps (boot time, offenders, regression check, correlation, hypotheses)
- **"What Changed" Correlation**: Links slow units to package updates, service enables, config edits
- **Fix Playbooks**: Restart/disable services with policy gates, confirmation, post-checks
- **Recipe Capture**: Automatic recipe creation when fix works with reliability >= 80%
- **Case File**: `boot_doctor.json` for audit trail with verification pending notes

### v0.0.40
- **Arch Audio Doctor v1 (PipeWire Focus)**: Diagnoses common audio issues
- **Stack Detection**: PipeWire/WirePlumber (primary), PulseAudio (legacy), ALSA, Bluetooth
- **Deterministic Diagnosis Flow**: 6 steps (stack, services, devices, defaults, conflicts, bluetooth)
- **Fix Playbooks**: Restart services, unmute/volume, set default, stop conflicts, BT profile
- **Recipe Capture**: Automatic recipe creation when fix works with reliability >= 80%
- **Case File**: `audio_doctor.json` for audit trail

### v0.0.39
- **Arch Storage Doctor v1 (BTRFS Focus)**: Specialized storage diagnosis
- **Evidence Bundle**: Mount topology, BTRFS device stats/usage/scrub/balance, SMART data, I/O errors
- **BTRFS-Specific Diagnostics**: Metadata pressure detection, device error tracking, scrub/balance status
- **Deterministic Diagnosis Flow**: 5-step analysis with risk-rated findings
- **Hypotheses**: Up to 3 evidence-backed hypotheses with confirm/refute criteria
- **Safe Repair Plans**: Read-only (no confirmation) and mutations (policy-gated)
- **Policy Controls**: Balance blocked by default (can take hours), scrub allowed
- **Case File**: `storage_doctor.json` for audit trail

### v0.0.38
- **Arch Networking Doctor v1**: Specialized WiFi/Ethernet diagnosis
- **Network Manager Support**: NetworkManager, iwd, systemd-networkd, wpa_supplicant
- **Deterministic Diagnosis Flow**: Physical link, IP, route, DNS, manager health checks
- **Fix Playbooks**: Confirmation, post-checks, and rollback support

### v0.0.32
- **CI Hardening**: All builds/tests run in Arch Linux container
- **Release Guardrails**: Version consistency checks, doc validation
- **Repo Hygiene**: No legacy commands in docs, version sync enforcement
- **Arch-Only**: Explicit single-platform support statement
- **Smoke Tests**: CLI verification in CI (--version, --help, status, NL query)

### v0.0.31
- **Reliability Engineering**: Full metrics collection, error budgets, self-diagnostics
- **Metrics Collection**: Tracks request/tool/mutation/LLM success rates, latencies (p50/p95), cache hits
- **Error Budgets**: 1% request failures, 2% tool failures, 0.5% mutation rollbacks, 3% LLM timeouts
- **Self-Diagnostics**: Generate pasteable reports via "generate a self-diagnostics report"
- **Natural Language**: Ask "show me the error budgets" or "generate a bug report"
- New tools: `self_diagnostics`, `metrics_summary`, `error_budgets`

### v0.0.30
- Helper auto-installation on daemon startup
- Periodic helper health check (every 10 minutes)
- Auto-reinstall helpers if user removes them

### v0.0.29
- Fixed auto-update artifact name matching for architecture suffix
- Update now properly detects and installs new versions

### v0.0.28
- System-aware helper filtering (no ethtool if no ethernet, etc.)
- Improved Ollama detection (checks command, not just package)
- Removed confusing INSTALL REVIEW section from status
- Clearer helper display ("Anna will install when needed")
- Better policy display ("Protected: X paths")

### v0.0.27
- Fixed Ollama detection reliability
- Changed helpers text to "Anna will install when needed"
- Fixed stale update state on daemon start
- Improved policy display clarity

### v0.0.26
- Implemented actual auto-update download/install in daemon
- Automatic binary backup and atomic installation
- Self-restart via systemd after update

### v0.0.25
- Auto-create install state on daemon start
- Better helpers display messaging

### v0.0.24
- Fixed translator LLM parsing for different model outputs
- More robust intent detection

### v0.0.23
- Auto-install Ollama if missing
- Auto-pull models when needed
- Track installations as "anna-installed"
- Auto-create policy defaults on daemon start
