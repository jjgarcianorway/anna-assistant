# Anna v5.2.5 "Knowledge is the Machine"

**Your Intelligent Linux Assistant - Evidence-Based, Never Hallucinating**

> v5.2.5 Knowledge refinement: Strict installed-only knowledge views. Command separation (status=Anna health, knowledge=installed objects, stats=daemon activity). Object relationships (aquamarine->hyprland). Description metadata from ArchWiki/man. Relevance filter hides zero metrics. Quality scoring for knowledge completeness.

Anna is a dual-LLM system that provides reliable, evidence-based answers about your Linux system. She uses a strict command whitelist—no arbitrary shell execution. Every answer is grounded in measured facts.

---

## 📋  What Anna Is

- **Evidence Oracle** - Answers based on command output from YOUR machine
- **Dual-LLM Architecture** - Junior (LLM-A) proposes, Senior (LLM-B) verifies
- **Zero Hallucinations** - If she can't measure it, she says so
- **Command Whitelist** - 40+ safe commands, no arbitrary shell
- **Learning System** - Remembers facts about your specific machine
- **Self-Updating** - Auto-updates enabled by default

## ❌  What Anna Is NOT

- Not a general chatbot (won't discuss weather, politics, recipes)
- Not a code generator (won't write your Python scripts)
- Not omniscient (won't guess what she can't measure)
- Not cloud-based (runs 100% locally via Ollama)
- Not destructive (high-risk commands require explicit approval)

---

## 🏗️  Architecture (v3.0.0)

```
┌─────────────────┐
│     annactl     │  User interface (CLI only)
│  (REPL / CLI)   │
└────────┬────────┘
         │ HTTP :7865
         ▼
┌─────────────────────────────────────────────────────┐
│                      annad                          │
│  ┌─────────────────────────────────────────────┐   │
│  │               Unified Pipeline               │   │
│  │  Question → Brain → Recipe → Junior → Senior │   │
│  └─────────────────────────────────────────────┘   │
└────────┬───────────────────────────────────────────┘
         │
    ┌────┴────┬───────────┬───────────┬────────────┐
    ▼         ▼           ▼           ▼            ▼
┌───────┐ ┌───────┐ ┌──────────┐ ┌──────────┐ ┌────────┐
│ Brain │ │Recipe │ │  Junior  │ │  Senior  │ │Ollama  │
│<150ms │ │<500ms │ │   <8s    │ │  <10s    │ │ LLM    │
│0 LLMs │ │0 LLMs │ │  2 LLMs  │ │  3 LLMs  │ │(Local) │
└───────┘ └───────┘ └──────────┘ └──────────┘ └────────┘
```

### Answer Origins (v3.0.0)

| Origin | Latency | LLM Calls | Description |
|--------|---------|-----------|-------------|
| **Brain** | <150ms | 0 | Fast pattern match for CPU, RAM, disk, health |
| **Recipe** | <500ms | 0 | Learned pattern + probe execution |
| **Junior** | <8s | 2 | Junior plan + draft (Senior skipped) |
| **Senior** | <10s | 3 | Full Junior + Senior audit |

### Components

| Component | Role |
|-----------|------|
| **annactl** | CLI wrapper. REPL mode or one-shot questions. |
| **annad** | Evidence Oracle. Daemon with unified pipeline. |
| **Brain** | Pattern matcher. <150ms, no LLM. Hardware/health/debug. |
| **Recipe** | Learned patterns. Extracts and reuses successful answers. |
| **Junior** | Researcher. Plans probes, drafts answers. |
| **Senior** | Auditor. Reviews Junior work, scores reliability. |
| **Ollama** | Local LLM backend. Auto-provisioned models. |

---

## 🔒  Security Model

### Command Whitelist

Anna cannot execute arbitrary shell commands. Only these 40+ whitelisted commands:

```
HARDWARE:     lscpu -J, cat /proc/meminfo, lspci, lsusb
STORAGE:      lsblk -J, df -P /, mount
NETWORK:      ip -j link show, ip -j addr, ip -j route, ss -tuln
PACKAGES:     pacman -Qi {pkg}, pacman -Q, pacman -Ss {pattern}
SERVICES:     systemctl status {svc}, systemctl list-units
FILES:        cat {file}, head/tail {file}, ls -la {path}, grep {pattern} {file}
PROCESS:      ps aux, uptime, who
CONFIG:       cat /etc/os-release, hostname, timedatectl, locale, env
```

### Risk Levels

| Risk Level | Auto-Approve | User Confirm | Examples |
|------------|--------------|--------------|----------|
| 🟢  **Low** | Normal mode | Never | `lscpu -J`, `pacman -Qi vim` |
| 🟡  **Medium** | Dev mode | Normal mode | `mkdir -p`, `cp backup` |
| 🔴  **High** | Never | Always | `pacman -S`, `systemctl start` |

### Injection Prevention (v1.0.0 Hardened)

- Parameters are validated against shell metacharacters
- No `|`, `;`, `&`, `` ` ``, `$`, `()`, `<>`, newlines
- Path traversal blocked (`..` sequences rejected)
- Null byte injection blocked
- Maximum parameter length enforced (4KB)
- Templates are compiled into binary—cannot be changed at runtime

---

## 🔄  Unified Pipeline (v3.0.0)

### Answer Flow

```
1. Question arrives
2. Brain fast path: Pattern match for hardware/health/debug (<150ms)
3. Recipe match: Check learned patterns from previous answers (<500ms)
4. Junior planning: Select probes to run (1st LLM call)
5. Probe execution: Run whitelisted commands
6. Junior draft: Generate answer with evidence (2nd LLM call)
7. Senior audit: Review and score (3rd LLM call, optional)
8. Recipe extraction: Learn from high-reliability answers
9. Final answer with origin, reliability, timing
```

### Senior Verdicts

| Verdict | Meaning |
|---------|---------|
| `approve` | Answer is properly grounded, deliver as-is |
| `fix_and_accept` | Minor corrections applied |
| `refuse` | Cannot answer safely (rare) |
| `skipped` | Senior skipped (Junior confidence >= 80%) |

### Reliability Scoring

```
>= 0.90: GREEN  (high confidence)
>= 0.70: YELLOW (medium confidence)
<  0.70: RED    (low confidence)
```

### Performance Budgets (v3.13.0)

| Budget | Time | Purpose |
|--------|------|---------|
| **Global** | 20s | Maximum time per question |
| **Fast Path** | 500ms | Brain + Recipe answers |
| **Junior Soft** | 8s | Trigger degradation warning |
| **Junior Hard** | 12s | Cancel and fall back |
| **Senior Soft** | 10s | Trigger degradation warning |
| **Senior Hard** | 15s | Cancel and produce RED answer |
| **Degraded** | 2s | Emergency RED answer generation |

If any timeout is hit, Anna produces an honest RED answer explaining what happened.

### Recipe Learning

When Junior/Senior produces a high-reliability answer (>=85%):
- Recipe extracted: question type + probes + answer template
- Stored for future matching
- Similar questions answered by Recipe (no LLM needed)

---

## 💬  CLI Surface (Locked)

```bash
# Interactive REPL mode
annactl

# One-shot question
annactl "How many CPU cores do I have?"

# System status (daemon, LLM, update, self-health)
annactl status

# Version info
annactl version

# Help
annactl -h
annactl --help
annactl help
```

**That's it.** No other flags or subcommands exist. The CLI surface is intentionally locked.

---

## ⚙️  Configuration

### Location

Configuration is stored at **`/etc/anna/config.toml`** (system-wide only).

This is intentional: the system administrator controls Anna's settings (LLM model, update policy), not individual users.

### Schema

```toml
[core]
mode = "normal"           # normal or dev

[llm]
preferred_model = "llama3.2:3b"
fallback_model = "llama3.2:3b"
selection_mode = "auto"   # auto or manual

[update]
enabled = true            # Auto-update enabled by default
interval_seconds = 600    # Check every 10 minutes (minimum allowed)
channel = "main"          # main, stable, beta, or dev

[log]
level = "info"            # trace, debug, info, warn, error
```

### Natural Language Configuration

You can change settings by asking Anna:

```bash
# Enable dev mode with frequent updates
annactl "enable dev auto-update every 10 minutes"

# Switch to a specific model
annactl "use qwen2.5:14b manually"

# Show current configuration
annactl "show your configuration"
```

---

## 🔄  Auto-Update Engine

- **Enabled by default** - Fresh installs auto-update daily
- **Tarball-based** - Both annad and annactl update together
- **Atomic** - Downloads to temp, verifies checksum, then replaces
- **Restart** - Daemon restarts automatically after update
- **Dev mode** - Can check every 10 minutes for rapid iteration

### Update Flow

1. Daemon checks GitHub releases at configured interval
2. If newer version found, downloads tarball
3. Verifies SHA256 checksum
4. Extracts and replaces binaries atomically
5. Daemon exits with code 42 to trigger systemd restart

---

## 🧠  Knowledge Store

Anna learns facts about YOUR specific machine and stores them in SQLite.

### Fact Sources

| Source | Trust Level | Example |
|--------|-------------|---------|
| **Measured** | 1.0 | "CPU has 8 cores" (from `lscpu`) |
| **User-asserted** | 0.7 | "I use Vim" (user said so) |
| **Inferred** | 0.5 | "Likely Arch-based" (from package manager) |

### What Gets Stored

- Hardware characteristics (static, cached)
- Package installation states (verified on demand)
- Config file locations discovered during research
- User preferences stated in conversation

---

## 🏥  Self-Health & Auto-Repair

Anna monitors her own health and can auto-repair common issues:

```bash
annactl status
```

### Health Checks

| Component | What's Checked |
|-----------|----------------|
| Daemon | Process running, responding to requests |
| Ollama | Service available, models loaded |
| Config | File exists, valid TOML syntax |
| Permissions | Log/state directories writable |
| Logging | Writers functional, rotation working |

### Auto-Repair Actions

- Restart daemon if unresponsive
- Regenerate default config if missing
- Create log directories if missing

---

## 📊  Telemetry & Logging

### JSONL Logs

All LLM interactions are logged for debugging:

```
/var/log/anna/daemon.jsonl     # Daemon operations
/var/log/anna/requests.jsonl   # User requests
/var/log/anna/llm.jsonl        # LLM-A/LLM-B exchanges
```

### Reasoning Traces

Each research loop produces a trace:

```json
{
  "trace_id": "uuid",
  "steps": [
    {"type": "user_request", "text": "..."},
    {"type": "llm_a_plan", "checks": [...]},
    {"type": "check_executed", "result": {...}},
    {"type": "llm_b_eval", "verdict": "accept"},
    {"type": "final_answer", "confidence": 0.92}
  ]
}
```

---

## 📦  Installation

### Quick Install (curl)

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

### What the Installer Does

1. Downloads latest release tarball
2. Verifies SHA256 checksum
3. Installs binaries to `/usr/local/bin`
4. Creates systemd service file
5. Creates config directory at `/etc/anna`
6. Creates log directory at `/var/log/anna`
7. Creates state directory at `/var/lib/anna`
8. Starts and enables the daemon

### Build from Source

```bash
git clone https://github.com/jjgarcianorway/anna-assistant.git
cd anna-assistant
cargo build --release
sudo ./scripts/install.sh
```

### Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/uninstall.sh | bash
```

---

## 📋  Requirements

- **OS**: Linux (x86_64 or aarch64)
- **LLM**: [Ollama](https://ollama.ai) with at least one model
- **Rust**: 1.70+ (for building from source)
- **Systemd**: For daemon management

### Recommended Models

| Model | Size | Speed | Quality |
|-------|------|-------|---------|
| `llama3.2:3b` | 2GB | Fast | Good for basic queries |
| `qwen2.5:7b` | 4GB | Medium | Better reasoning |
| `qwen2.5:14b` | 8GB | Slow | Best quality |

---

## 🏷️  Version History

| Version | Milestone |
|---------|-----------|
| **v5.2.5** | **Knowledge is the Machine** - Strict installed-only views, command separation (status/knowledge/stats), object relationships, description metadata, relevance filter, quality scoring |
| v5.2.4 | Knowledge Core - Error index, service state, intrusion detection, display formatting |
| v5.2.0 | Knowledge System - System profiler, knowledge engine, full inventory |
| v4.5.4 | LLM Routing Sanity - Predictable routing invariants, empty plan = failure, consistent ROUTE lines, 8 routing invariant tests, ASCII only |
| v4.5.3 | Learning & Fast Reuse - question_key normalization, answer cache (>=90% reliability), 5-min TTL, ROUTE: Cache debug line, instant reuse |
| v4.5.2 | Latency & Fallback - Timeout streak tracking, auto-switch after 2 timeouts, status/stats show timeout info, FALLBACK debug lines, ASCII-only |
| v4.5.1 | Debug Spine - Telemetry reads both primary/fallback paths, clear ROUTE lines (Brain/Orchestrator(Junior)/Orchestrator(Senior)), ASCII-only debug, status/stats coherence |
| v4.5.0 | Tiered Learning - 3-tier model architecture (Brain/Junior/Senior), per-class model selection, skip LLM when reliability ≥90%, enhanced debug (CLASSIFIED/CACHE/TIER lines) |
| v4.4.0 | Functional Learning - Semantic classification, paraphrase recognition, pattern caching, reset clears patterns |
| v4.3.2 | Telemetry Fallback - Fixed permission issue on telemetry reset fallback path |
| v4.3.1 | LLM Answer Counting - Correct answer count tracking, shared answer cache |
| v4.3.0 | Smart Recovery - Auto-downgrade, answer cache, daemon reset |
| v4.2.0 | Real Debug Mode - Live request/response tracing, failure analysis |
| v4.1.0 | Simplified Health - Detailed dependencies, no trust, success rate |
| v4.0.0 | Debug Tracing - Reset command, learning analytics |
| v3.13.1 | Lifecycle Integrity - Hard reset executes on confirmation, timeouts fixed everywhere (10s/12s/20s), systemd HOME fix for ollama, confirmation patterns |
| v3.13.0 | Lifecycle Integrity (broken) - Partial fixes, timeouts not updated in engine |
| v3.12.0 | Performance & Consistency - GPU/Network Brain handlers, First Light uses Brain, per-call LLM timeouts, hardware tier in status |
| v3.11.0 | Lifecycle Correctness - Benchmark triggers route to daemon, Brain telemetry recording, lifecycle tests |
| v3.10.0 | Correctness Patch - OS/kernel Brain fast path, honest self-health, percentage display, permission fixes |
| v3.9.0 | Consistency & Migration - Honest status output, reset history tracking, coherent messages |
| v3.8.0 | Preflight QA - Learning Contract, 43 learning tests, benchmark verification |
| v3.7.0 | Reliability Gauntlet - System acceptance tests, "Day in the Life" scenario, zero warnings |
| v3.5.0 | Verification & Guardrails - Property tests, dry-run checks, 1000+ tests, clean codebase |
| v3.4.0 | Performance & Degradation Guard - Time budgets, tiered timeouts, degraded answers |
| v3.3.0 | Integrity & Verification - Feature Integrity Matrix, 56+ integrity tests |
| v3.1.0 | Pipeline Purity - Remove legacy LLM orchestrator from annactl (1036 lines removed) |
| v3.0.0 | Brain First - Router LLM, Recipe learning, hardware-aware provisioning |
| v2.3.0 | Runtime Snow Leopard - Benchmark triggers, 10s latency guardrail, no empty answers |
| v2.2.0 | First Light - Post-reset self-test, XP/Telemetry sanity validation, daily check-in |
| v2.1.0 | Permissions Fix - XP/Telemetry persistence, installer permissions, reset pipeline |
| v2.0.0 | Autoprovision - Self-provisioning LLM models, auto-install, auto-benchmark |
| v1.1.0 | Adaptive LLM Provisioning - Model benchmarking, skill routing |
| v1.0.0 | Snow Leopard - Stabilization, deterministic tests, security hardening |
| v0.89.0 | Conversational Debug Mode - Natural language debug toggle |
| v0.88.0 | Dynamic Probe Catalog & XP Wiring - Single source of truth for probes, Junior/Senior XP events |
| v0.87.0 | Latency Cuts & Brain Fast Path - <3s simple questions, hard fallback, always visible answer |
| v0.86.0 | XP Reinforcement - Anna/Junior/Senior XP tracking, trust, ranks, behaviour bias |
| v0.85.0 | Architecture Optimisation - Brain layer, LLM reduction, self-sufficiency |
| v0.83.0 | Performance Focus - Compact prompts, 15s target latency |
| v0.80.0 | Razorback Fast Path - <5s response for simple questions |
| v0.70.0 | Evidence Oracle - Structured LLM protocol, difficulty routing, knowledge-first |
| v0.65.0 | Reliability Patch - Confidence gating (60% min), stats tracking, daemon robustness |
| v0.60.0 | Conversational UX - Live progress events, conversation logging, persona messaging |
| v0.50.0 | Brain Upgrade - 5-type classification, safe command policy, generic probes |
| v0.43.0 | Live Debug Streaming View |
| v0.42.0 | Negative Feedback, Skill Pain, Remediation Engine |
| v0.40.0 | Generic skills, parameterized commands, skill learning, no probe zoo |
| v0.30.0 | Journalctl probe, question classifier, auto-update fix |
| v0.26.0 | Auto-update Reliability, Self-Healing Watchdog, Structured Tracing |
| v0.15.0 | Research Loop Engine with command whitelist |
| v0.11.0 | Knowledge store, event-driven learning |
| v0.10.0 | LLM-A/LLM-B supervised audit loop |

---

## 📜  License

GPL-3.0-or-later

---

## 🤝  Contributing

Issues and PRs welcome at: https://github.com/jjgarcianorway/anna-assistant

**Core Design Principles:**

1. Zero hardcoded knowledge - only measured facts
2. Command whitelist - no arbitrary shell
3. Dual-LLM verification - Junior proposes, Senior approves
4. System-wide config - admin controls settings
5. Auto-update by default - always fresh
6. Honest about limitations - "no data" over hallucination
