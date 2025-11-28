# Anna Technical Specification

**Version**: 0.26.0
**Last Updated**: 2025-11-28

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

## 🏗️  Architecture

```
┌─────────────────┐
│     annactl     │  User interface (CLI only)
│  (REPL / CLI)   │
└────────┬────────┘
         │ HTTP :7865
         ▼
┌─────────────────┐     ┌─────────────────┐
│      annad      │────▶│   Ollama LLM    │
│    (Daemon)     │     │  (Local Only)   │
└────────┬────────┘     └─────────────────┘
         │
    ┌────┴────┬───────────┬────────────┐
    ▼         ▼           ▼            ▼
┌───────┐ ┌───────┐ ┌──────────┐ ┌──────────┐
│LLM-A  │ │LLM-B  │ │ Command  │ │Knowledge │
│Junior │ │Senior │ │Whitelist │ │  Store   │
└───────┘ └───────┘ └──────────┘ └──────────┘
```

### Components

| Component | Role |
|-----------|------|
| **annactl** | CLI wrapper. REPL mode or one-shot questions. Locked surface. |
| **annad** | Evidence Oracle. Daemon that orchestrates everything. |
| **LLM-A** | Junior Researcher. Plans checks, drafts answers, asks questions. |
| **LLM-B** | Senior Verifier. Reviews plans, approves commands, scores answers. |
| **Command Whitelist** | Rust-compiled list of allowed commands. No exceptions. |
| **Knowledge Store** | SQLite-backed facts learned from YOUR machine. |

---

## 🔒  Security Model

### Command Whitelist

Anna cannot execute arbitrary shell commands. Only whitelisted commands:

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
| 🟢 **Low** | Normal mode | Never | `lscpu -J`, `pacman -Qi vim` |
| 🟡 **Medium** | Dev mode | Normal mode | `mkdir -p`, `cp backup` |
| 🔴 **High** | Never | Always | `pacman -S`, `systemctl start` |

### Injection Prevention

- Parameters validated against shell metacharacters
- No `|`, `;`, `&`, `` ` ``, `$`, `()`, `<>`, newlines
- Templates compiled into binary - cannot be changed at runtime

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
annactl -V
annactl --version

# Help
annactl -h
annactl --help
annactl help
```

**That's it.** No other flags or subcommands exist. The CLI surface is intentionally locked.

---

## 🔄  Dual-LLM Protocol

### Research Loop (max 6 iterations)

```
1. User asks question
2. LLM-A (Junior) proposes checks from whitelist
3. LLM-B (Senior) reviews and approves/denies each check
4. Engine runs approved checks
5. LLM-A drafts answer with evidence
6. LLM-B verifies answer grounding
7. If more evidence needed, loop (max 6 times)
8. Final answer delivered with confidence score
```

### LLM-B Verdicts

| Verdict | Meaning |
|---------|---------|
| `accept` | Answer is properly grounded, deliver as-is |
| `fix_and_accept` | Minor corrections, use LLM-B's fixed answer |
| `needs_more_checks` | Run more whitelisted commands before answering |
| `mentor_retry` | LLM-A should try again with specific feedback |
| `refuse` | Cannot answer safely (very rare) |

### Confidence Scoring

```
overall = min(evidence, reasoning, coverage)

>= 0.90: GREEN  (high confidence)
>= 0.70: YELLOW (medium confidence)
<  0.70: RED    (low confidence, includes disclaimer)
```

---

## ⚙️  Configuration

### Location

Configuration stored at **`/etc/anna/config.toml`** (system-wide only).

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

---

## 🔄  Auto-Update Engine

- **Enabled by default** - Fresh installs auto-update daily
- **Tarball-based** - Both annad and annactl update together
- **Atomic** - Downloads to temp, verifies checksum, then replaces
- **Restart** - Daemon restarts automatically after update

### Update Flow

1. Daemon checks GitHub releases at configured interval
2. If newer version found, downloads tarball
3. Verifies SHA256 checksum
4. Extracts and replaces binaries atomically
5. Daemon exits with code 42 to trigger systemd restart

---

## 🏥  Self-Health & Auto-Repair (v0.26.0)

Anna monitors her own health and can auto-repair common issues:

### Health Checks

| Component | What's Checked |
|-----------|----------------|
| Daemon | Process running, responding to requests |
| Ollama | Service available, models loaded |
| Config | File exists, valid TOML syntax |
| Permissions | Log/state directories writable |
| Logging | Writers functional, rotation working |

### Auto-Repair Actions

- Restart daemon if unresponsive (rate-limited)
- Regenerate default config if missing
- Create log directories if missing

---

## 🧠  Knowledge Store

Anna learns facts about YOUR specific machine and stores them in SQLite.

### Fact Sources

| Source | Trust Level | Example |
|--------|-------------|---------|
| **Measured** | 1.0 | "CPU has 8 cores" (from `lscpu`) |
| **User-asserted** | 0.7 | "I use Vim" (user said so) |
| **Inferred** | 0.5 | "Likely Arch-based" (from package manager) |

---

## 📋  Requirements

- **OS**: Linux (x86_64 or aarch64)
- **LLM**: Ollama with at least one model
- **Rust**: 1.70+ (for building from source)
- **Systemd**: For daemon management

### Recommended Models

| Model | Size | Speed | Quality |
|-------|------|-------|---------|
| `llama3.2:3b` | 2GB | Fast | Good for basic queries |
| `qwen2.5:7b` | 4GB | Medium | Better reasoning |
| `qwen2.5:14b` | 8GB | Slow | Best quality |
