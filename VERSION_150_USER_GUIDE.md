# Anna Assistant v150 - User Guide

## What's New in Version 150

Version 150 brings **unified, deterministic responses** across all modes and introduces **contextual awareness** that makes Anna behave like a professional system administrator.

---

## Key Features

### 1. **Consistent Behavior Everywhere**

**Before v150**: Asking the same question in CLI vs TUI could give different answers.

**Now**: CLI and TUI use the same query pipeline → **identical responses always**.

**Example:**
```bash
# CLI one-shot
$ annactl "what is my CPU?"
✓ Your CPU is a Intel(R) Core(TM) i9-14900HX with 32 cores.

# TUI (type the same question)
✓ Your CPU is a Intel(R) Core(TM) i9-14900HX with 32 cores.
```

---

### 2. **Contextual Greetings**

Anna now remembers your sessions and greets you contextually:

**Time-Aware:**
- "Good morning" (5am-12pm)
- "Good afternoon" (12pm-6pm)
- "Good evening" (6pm-9pm)
- "Hello" (night hours)

**Session-Aware:**
- Remembers last login time
- Notices version upgrades
- Reports time since last session

**Health-Aware:**
- Surfaces critical issues immediately
- Warns about low disk space
- Alerts about failed services

**Example TUI Startup:**
```
Good morning! Welcome back.
Last session was 2 days ago.

⚠️  System Alerts:
  • 1 critical issue(s)
  • 2 warning(s)
Type 'alerts' to view details.

🖥️  System Status:
• CPU: Intel(R) Core(TM) i9-14900HX (2% load) - ✅ running smoothly
• RAM: 18.5GB / 62.5GB (30% used) - ✅ plenty available
• Disk: 277.0GB free

🤖 AI Assistant: ✅ llama3.1:8b ready
```

---

### 3. **User Profile Queries**

Anna can now describe your usage profile based on actual interaction patterns:

**Query Examples:**
```bash
$ annactl "what are my personality traits?"
$ annactl "describe me as a user"
$ annactl "what kind of user am I?"
```

**Response:**
```
Based on your usage patterns: Regular user - building familiarity with system administration

Most frequent commands:
  • system status (15 times)
  • disk space (8 times)
  • docker ps (5 times)

Total interactions: 35

🔍 Confidence: High | Sources: system telemetry
```

**What This Means:**
- ✅ Uses **only local data** (your command history)
- ✅ **No personal analysis** - just usage statistics
- ✅ **No dangerous commands** - pure telemetry
- ✅ Safe, respectful, professional

---

### 4. **Structured JSON Responses**

**All responses are now structured** - no more freeform markdown rambling.

**Response Types:**

| Type | Description | Latency | Confidence |
|------|-------------|---------|------------|
| **Deterministic Recipe** | Hard-coded, tested plans | <1ms | 100% |
| **Template** | Instant shell commands | <10ms | 100% |
| **Action Plan** | LLM-generated structured plan | ~1-3s | High |
| **Conversational Answer** | Telemetry or LLM answer | <1ms or ~1-3s | High/Medium |

**All include:**
- Confidence level (High/Medium/Low)
- Data sources (system telemetry, LLM, etc.)
- Professional formatting

---

### 5. **Accurate System Reporting**

**Storage Queries:**
```bash
$ annactl "how much free disk space do I have?"
$ annactl "show me disk usage"
```

**Response:**
```
Running: $ df -h /

Filesystem      Size  Used Avail Use% Mounted on
/dev/nvme0n1p6  803G  519G  277G  66% /
```

**CPU Queries:**
```bash
$ annactl "what is my CPU?"
```

**Response:**
```
Your CPU is a Intel(R) Core(TM) i9-14900HX with 32 cores.
Current load: 2.21 (1-min avg).

🔍 Confidence: High | Sources: system telemetry
```

**RAM Queries:**
```bash
$ annactl "how much RAM do I have?"
```

**Response:**
```
You have 62.5 GB of RAM total.
Currently using 18.5 GB (29.6%).

🔍 Confidence: High | Sources: system telemetry
```

---

## How to Use Anna v150

### CLI Mode (One-Shot Queries)

```bash
# System information
$ annactl "what is my CPU?"
$ annactl "how much RAM do I have?"
$ annactl "show me disk usage"

# System health
$ annactl "show failed services"
$ annactl "how is my system?"

# User profile
$ annactl "what are my personality traits?"

# Installation tasks (generates ActionPlan)
$ annactl "install docker"
$ annactl "setup neovim"
```

### TUI Mode (Interactive)

```bash
# Start TUI
$ annactl tui

# Or just:
$ annactl
```

**Features:**
- Contextual greeting on startup
- System status overview
- Interactive Q&A
- Same responses as CLI mode

---

## Understanding Confidence Levels

Every answer includes a confidence indicator:

**✅ High Confidence**
- Direct from system telemetry
- Zero hallucination risk
- Instant (<1ms latency)
- Examples: CPU model, RAM total, disk space

**🟡 Medium Confidence**
- LLM-generated with system context
- Validated responses
- ~1-3s latency
- Examples: Complex explanations, troubleshooting

**⚠️ Low Confidence** (reserved, not currently used)
- LLM without validation
- Used only for general questions

---

## Health Monitoring

Anna proactively monitors your system and surfaces issues:

**Critical Alerts:**
- Disk space < 5GB free
- Failed systemd services
- System load > 2x CPU cores

**Warnings:**
- Disk space < 10GB free
- Memory usage > 90%
- Load average elevated

**Healthy:**
- All systems nominal
- Brief, reassuring status

---

## Context Engine Data Storage

Anna stores minimal context in:
```
~/.local/share/anna/context.json
```

**What's Stored:**
- Last session timestamp
- Anna version at last run
- Command frequency (for usage profiles)
- Recent command history (last 50)
- Known system alerts

**What's NOT Stored:**
- Personal information
- Command arguments or data
- File contents
- Passwords or secrets

**Privacy:**
- All data stays local
- No telemetry sent externally
- You can delete `context.json` anytime

---

## Troubleshooting

### "LLM not available"

Install Ollama for full AI capabilities:
```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model
ollama pull llama3.1:8b

# Verify
ollama list
```

### "Context Engine error"

Context Engine is optional. If it fails:
- Check permissions on `~/.local/share/anna/`
- Delete `context.json` to reset
- Anna will work fine without it

### Different answers in CLI vs TUI

This should not happen in v150. If you see this:
1. Verify both are using v150: `annactl --version`
2. Report the issue with both commands
3. Include exact phrasing of query

---

## Command Reference

### Quick Actions

| Query | Result |
|-------|--------|
| `what is my CPU?` | CPU model and load |
| `how much RAM?` | RAM usage statistics |
| `disk space?` | Disk usage for root |
| `show failed services` | Failed systemd units |
| `what are my personality traits?` | Usage profile |
| `how is my system?` | Overall health check |

### Installation Recipes (Tier 1)

These generate instant, deterministic ActionPlans:
- `install docker`
- `install neovim` / `setup neovim`
- `change wallpaper`
- `update system`

### Action Keywords (Tier 3)

These trigger LLM-generated ActionPlans:
- install, setup, configure
- fix, repair
- update, upgrade
- enable, disable
- start, stop, restart

---

## What Makes v150 Different

**Professional Sysadmin Behavior:**
- ✅ Proactive health monitoring
- ✅ Contextual awareness
- ✅ Deterministic responses
- ✅ Structured outputs
- ✅ Confidence transparency

**Not a Chatbot:**
- ❌ No rambling explanations
- ❌ No freeform streaming
- ❌ No inconsistent responses
- ❌ No dangerous commands for personality queries

**Technical Excellence:**
- 4-tier architecture
- JSON determinism enforced
- Unified CLI/TUI pipeline
- Context Engine integration

---

## Feedback and Issues

Report issues or suggest features:
- GitHub: https://github.com/yourusername/anna-assistant/issues
- Include: Anna version, OS, exact query, unexpected behavior

---

**Version**: 150 Beta
**Release Date**: 2025-11-20
**Status**: Production Ready
