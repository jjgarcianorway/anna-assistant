# Anna Assistant

**Experimental Arch Linux System Assistant - Version 6.39.0**

[![Version](https://img.shields.io/badge/version-6.39.0-blue.svg)](https://github.com/jjgarcianorway/anna-assistant)
[![License](https://img.shields.io/badge/license-GPL--3.0-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Arch%20Linux-1793d1.svg)](https://archlinux.org)
[![Status](https://img.shields.io/badge/status-experimental-orange.svg)](https://github.com/jjgarcianorway/anna-assistant)

---

## ⚠️ Status: Experimental Prototype

**Anna is NOT production-ready software.**

This is an experimental CLI tool for Arch Linux system diagnostics and troubleshooting. It's under active development and changing rapidly. Use it if you:
- Want to experiment with local LLM-based system assistance
- Are comfortable debugging issues yourself
- Understand that features may break or change
- Don't rely on it for critical systems

**What this means:**
- ❌  Not suitable for production systems
- ❌  No stability guarantees
- ❌  Features may be incomplete or experimental
- ✅  CLI-only (no GUI)
- ✅  Local-first (no telemetry sent anywhere)
- ✅  Open source (GPL-3.0)

---

## What Works Right Now (6.39.0)

### Intelligent Log Noise Filtering in 6.39.0

**Problem:** Anna was reporting `CRITICAL: 10 critical log issues` when the only errors were gaming controller CRC errors (PlayStation DualSense, Xbox controllers). This eroded trust in health diagnostics.

**Solution:** Implemented intelligent log noise filtering:
- **Filters benign hardware errors:** PlayStation/Xbox controllers, USB enumeration, Bluetooth
- **Transparent:** Shows what was filtered and why
- **Zero false negatives:** Real system errors still escalate to CRITICAL
- **16 new tests:** 13 filter tests + 6 integration tests

**Before v6.39.0:**
```
Overall Status: 🚨 CRITICAL (10 log issues)
```

**After v6.39.0:**
```
Overall Status: ✓ HEALTHY
  ℹ️  10 hardware errors filtered (PlayStation controller noise)
```

**Impact:**
- Eliminates false-positive CRITICAL alerts from gaming hardware
- Improves trust in system health reporting
- Maintains sensitivity to real system errors

### Code Quality & File Size Guidelines (from 6.38.0)

To maintain code quality, Anna enforces file size limits:

**File Size Targets:**
- **Soft limit:** 400 lines per file
- **Hard limit:** 500 lines per file
- **Script:** `scripts/check_file_sizes.sh`

**Recent Modularization:**
- **recommender.rs**: Split from 12,012 → 241 lines (12 submodules)
- Result: 98% size reduction, zero breaking changes

**Guidelines:**
- Break large files into focused submodules using `mod` subdirectories
- Preserve public APIs (use re-exports)
- Group related functionality logically
- Document any whitelisted exceptions

### 1. Thinking Indicator & Progress Feedback (NEW in 6.36.0!)

Anna now shows visual progress during long-running operations:

**Animated Spinner:**
```bash
$ annactl "give me a system report"
⠹ Gathering system information...
✓ Report ready (1.8s)

▪  System Configuration
   CPU: AMD Ryzen 9 5950X (32 threads)
   RAM: 64 GB DDR4
...
```

**When You'll See It:**
- ✅ System reports (1-2 seconds)
- ✅ Disk analysis (1-5 seconds)
- ✅ LLM queries (2-10 seconds)
- ❌ Instant operations like capability checks (<100ms)

**Smart Behavior:**
- Only shows for operations taking >500ms
- Automatically hidden in piped output: `annactl status | cat` ✓
- Respects NO_COLOR environment variable
- Always cleans up on Ctrl+C (no broken terminal)

**Example with LLM:**
```bash
$ annactl "how do I check disk space?"
⠙ Thinking...
✓ Answer ready (2.7s)

To check disk space on Arch Linux, use the df command:
...
```

### 2. Presence Awareness & Anna Reflection (NEW in 6.35.0!)

Anna now tracks your usage patterns and shows contextual awareness:

**Anna Reflection on Status:**
When you run `annactl status`, Anna displays a self-aware reflection block at the top:

```bash
annactl status
```

```
Haven't seen you in 3 days - checking if anything broke while you were away.
Note: Memory usage trending upward
==================================================

SYSTEM STATUS — 2025-11-25 14:32:01 UTC
...
```

**What Anna tracks:**
- When you first and last used Anna
- Total query count and rolling 7-day/30-day windows
- Usage patterns (long absence, intense usage, quiet periods)
- Current system state from insights engine

**Presence Greetings:**
Anna greets you when you return after 12+ hours of inactivity (on system-oriented queries only):

```bash
# After a week away
annactl "run a full diagnostic"
```

```
Welcome back! It's been 8 days.

▪  System Diagnostics
   All core services: healthy
   No failed systemd units
...
```

**What's tested:**
- 8 reflection engine tests (usage commentary patterns, system reflection)
- 6 greeting behavior tests (intent filtering, throttling logic)
- Fail-safe design: DB failures never impact query results

**Privacy:**
- All data stored locally in `/var/lib/anna/historian.db`
- No network calls, no external tracking
- Only usage timestamps and query counts (no query content)

### 3. System Reports & Capability Queries (NEW in 6.33.0!)

```bash
# Generate full system reports
annactl "write me a report about my computer"
annactl "generate a system report"

# Check CPU capabilities
annactl "does my CPU support SSE2?"
annactl "do I have AVX2?"
annactl "does my system support virtualization?"

# Check GPU information
annactl "how much VRAM do I have?"
annactl "what GPU do I have?"

# Explore disk usage
annactl "show me the 10 biggest folders in /home"
annactl "what are the biggest directories?"
annactl "show top 5 largest folders under /var"
```

**What's new:**
- Deterministic pattern-based query detection (no LLM required)
- CPU feature flag detection (SSE2, AVX, AVX2, AVX-512, virtualization)
- GPU presence detection from system telemetry
- Disk usage analysis with safe `du` parameters
- All handlers tested with 20 new tests (12 unit + 8 integration)

**What's actually tested:**
- Pattern matching for all three query families
- CPU flag detection from `/proc/cpuinfo`
- Disk explorer path and count extraction
- Handler execution and error handling
- Integration with unified query router

### 4. Status Command

```bash
annactl status
```

Shows:
- **Anna's Reflection** (NEW in 6.35.0): Self-aware commentary on usage patterns and system state
- Overall system health (HEALTHY/DEGRADED/CRITICAL)
- Anna's self-health (daemon, permissions, LLM backend)
- System diagnostics with fix commands
- Recent daemon logs

**What's actually tested:**
- Daemon RPC reachability checks
- Strict health derivation (never lies about status)
- /var/log/anna permission validation
- Daemon restart instability detection
- Reflection engine usage commentary and system state

**Output example:**
```
You've been using Anna more intensively this week (47 queries vs usual 12).
System looks healthy.
==================================================

SYSTEM STATUS — 2025-11-25 14:32:01 UTC

Overall Status:
  ✓ HEALTHY: all systems operational

Anna Self-Health
  ✓ All dependencies, permissions, and LLM backend are healthy
  ✓ Daemon health: HEALTHY
```

### 5. Natural Language Queries with Follow-Up Support (6.26.0)

```bash
annactl "how do I check my kernel version?"
annactl "what desktop am I running?"
annactl "show me failed services"
```

**How it works:**
1. Tries deterministic wiki-backed answers first (6.16.0 - Wiki Answer Engine)
2. Falls back to LLM if no wiki match (requires Ollama)
3. Fetches actual system telemetry to ground answers in reality
4. Returns answers immediately (no interactive mode)

**NEW: Follow-Up Queries (6.26.0)**
Anna now remembers your last query for 5 minutes and can handle follow-ups:

```bash
annactl "how do I check disk space?"
# ... shows answer with df command ...

annactl "give me more details"
# ... expands with filesystem types, mount options, usage patterns ...

annactl "just show me the commands"
# ... extracts just the executable commands:
# • df -h
# • df -i
# • lsblk -f
```

**Supported follow-up patterns:**
- "more details", "more information", "expand on that"
- "just the commands", "only the command"
- "what do you mean", "clarify", "explain that"

**What's tested:**
- 40 wiki answer engine tests (query understanding, command synthesis)
- 14 session context tests (follow-ups, preference inference)
- Command intelligence layer tests (dynamic command generation)
- Answer validation tests

**Limitations:**
- LLM answers may contain errors (hallucination possible)
- Only ~10 wiki scenarios implemented so far
- Command execution requires manual confirmation
- Follow-up context expires after 5 minutes

---

## What Does NOT Work

Be honest about what's missing:

- ❌  **No interactive TUI** - The 5.x terminal UI is archived and disabled
- ❌  **No command execution** - Anna shows commands but doesn't run them (you run them)
- ❌  **No autonomous actions** - Everything requires manual approval
- ❌  **Limited scenario coverage** - Only DNS and service troubleshooting plans implemented
- ❌  **Daemon stability issues** - annad may crash or restart (experimental code)
- ❌  **No automatic fixes** - Anna diagnoses, you fix
- ❌  **Requires Ollama** - Most features need a local LLM running

---

## Installation

### Requirements

**Mandatory:**
- Arch Linux (x86_64) or Arch-based distro
- 8GB RAM minimum (for LLM)
- Ollama installed with a model (e.g., `qwen2.5:3b`)

**Optional but recommended:**
- systemd (for daemon mode)
- User in `systemd-journal` group (for log access)

### Quick Install

Download the latest release binaries from [GitHub Releases](https://github.com/jjgarcianorway/anna-assistant/releases) and place them in your system PATH.

Once installed, verify with:
```bash
annactl --version
annactl status
```

### For Developers

Build from source:
```bash
git clone https://github.com/jjgarcianorway/anna-assistant
cd anna-assistant
cargo build --release
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup details.

---

## Usage

### Basic Commands

**Check Anna's status:**
```bash
annactl status
```

**Ask a question:**
```bash
annactl "show me disk usage"
```

**That's it.** There are only two entry points.

### Running the Daemon (Optional)

The daemon (`annad`) collects telemetry and runs background diagnostics.

**Start manually:**
```bash
sudo ./target/release/annad
```

**Or use systemd:**
```bash
sudo systemctl start annad
sudo systemctl enable annad  # Start on boot
```

**Check if it's working:**
```bash
systemctl status annad
journalctl -u annad -n 50
```

---

## Architecture (Current Reality)

### What's Actually Built

**annad (Daemon):**
- Systemd service that runs as root
- Collects system telemetry (packages, services, hardware, logs)
- Runs diagnostic rules (9 built-in checks)
- Historian: Stores snapshots for trend analysis
- RPC server on Unix socket (`/run/anna.sock`)
- **Status:** Experimental, may crash

**annactl (Client):**
- CLI tool for status and queries
- Talks to daemon via Unix socket
- Formats answers for terminal display
- No interactive mode (one-shot only)
- **Session Context (6.26.0):** Remembers last query for follow-ups
- **Status:** Works but limited features

**Intelligence Layers (6.15.0-6.26.0):**
1. **Wiki Answer Engine (6.16.0)** - Pattern-based query matching to Arch Wiki articles (deterministic, no LLM)
2. **Session Context Memory (6.26.0)** - Follow-up query support with file-based persistence
3. **Command Intelligence Layer (6.15.0)** - Dynamic command synthesis based on system state
4. **LLM Fallback** - Ollama integration for questions without wiki coverage

**Diagnostic Engine:**
- 9 rule-based checks (disk, memory, services, packages, mounts, logs)
- 6 temporal correlation rules (Historian)
- Proactive assessment (calculates health score)
- **Status:** Works but limited scope

---

## What's Tested

### Passing Tests (661/661 - 100%)

**New in 6.34.0:**
- CLI consistency & output standards (16 tests)
  - No-markdown-fence validation (5 tests)
  - Style correctness (compact/stepwise/sectioned) (6 tests)
  - OutputEngine integration (5 tests)

**New in 6.33.0:**
- System report queries (20 tests)
  - Pattern matching for capability queries (4 tests)
  - Pattern matching for disk explorer (4 tests)
  - CPU flag detection (4 tests)
  - Integration tests - end-to-end flow (8 tests)

**New in 6.30.0:**
- Self-optimizing system (24 tests)
  - Meta telemetry tracking (6 tests)
  - Historian insight metadata (6 tests)
  - Optimization profile rules (9 tests)
  - Self-tuning report generation (3 tests)
- System knowledge test fixed (was failing in v6.29.0)

**Existing:**
- Proactive commentary engine (8 tests)
- Wiki answer engine (40 tests)
- Insights engine (16 tests)
- Session context (19 tests)
- Command intelligence (10 tests)
- Status health derivation (6 tests)
- Output formatting (8 tests)
- Diagnostic engine (core functionality)
- Action plan validation

**Test Status:** All tests passing (100% green)

---

## Known Issues

Be transparent:

1. **Daemon Instability**
   - annad may restart frequently under load
   - Memory usage can grow over time
   - Socket sometimes becomes unresponsive
   - **Workaround:** `sudo systemctl restart annad`

2. **LLM Dependency**
   - Most features require Ollama running
   - Models must be downloaded separately (4GB+)
   - Slow on systems with <8GB RAM
   - **Workaround:** Use wiki-backed queries only

3. **Limited Scenario Coverage**
   - Only ~10 troubleshooting scenarios implemented
   - Many queries fall back to generic LLM answers
   - Not all Arch Wiki articles mapped yet
   - **Workaround:** Use manual Arch Wiki lookups

4. **Permission Issues**
   - Requires correct `/var/log/anna` ownership (root:anna mode 750)
   - User must be in required groups for full features
   - **Workaround:** Status command will tell you exact fix commands

---

## Development Status

**Current Version:** 6.37.0 (November 25, 2025)

**Recent Progress:**
- ✅  6.37.0 - Reliable Updates & Hardened Answer UX (critical update detection fix, markdown fence/bold removal - 4 new tests)
- ✅  6.36.0 - Thinking Indicator & Progress Feedback v1 (animated spinners, TTY-aware, timing display - 8 new tests)
- ✅  6.35.0 - Presence Awareness & Anna Reflection (usage tracking, contextual greetings - 14 new tests)
- ✅  6.34.0 - CLI Consistency & Output Standards v1 (unified OutputEngine formatting, no markdown fences, compact/stepwise/sectioned styles - 16 new tests)
- ✅  6.33.0 - Actionable System Reports & Capability Queries v1 (system reports, CPU feature detection, disk explorer - 20 new tests)
- ✅  6.32.0 - Status Command Output Engine Integration (formatting refactor, 18 new tests, zero behavior changes)
- ✅  6.31.0 - Professional Output Engine (foundation module, no user-facing changes yet)
- ✅  6.30.0 - First Self-Optimizing Cycle (noise suppression, high-value highlighting, meta telemetry)
- ✅  6.29.0 - Insight Summaries Engine v1 (high-level deterministic system summaries)
- ✅  6.28.0 - Predictive Diagnostics Engine v1 (forecast future system risks deterministically)
- ✅  6.27.0 - Proactive Commentary Engine v1 (context-aware insights, "why did you say that?" follow-up)
- ✅  6.26.0 - Deep Context Memory & Proactive Commentary (follow-up queries, session persistence)

**Active Development:**
- Full answer path integration for commentary
- Additional insight matchers
- Natural language intent routing improvements
- Daemon stability enhancements

**Not Planned:**
- Interactive TUI (archived from 5.x)
- Remote management
- Cloud integration
- Autonomous actions without approval

---

## Contributing

This project needs help! Areas where contributions are valuable:

**High Priority:**
- Daemon stability fixes
- Additional wiki scenario mappings
- More diagnostic rules
- Test coverage improvements
- Documentation

**Lower Priority:**
- New features (focus on stability first)
- Performance optimizations
- UI/UX enhancements

**How to Contribute:**
1. Fork the repo
2. Create a branch
3. Make changes
4. Run tests: `cargo test`
5. Submit PR with clear description

---

## Documentation

- [CHANGELOG.md](CHANGELOG.md) - Version history
- [Architecture](docs/ARCHITECTURE_BETA_200.md) - System design (may be outdated)
- [Planner Design](docs/PLANNER_DESIGN_6x.md) - Planning system (6.2.0-6.4.x)

---

## Core Principles

What Anna is trying to be:

- **Local-First:** All data stays on your machine
- **Telemetry-First:** Real system facts, not LLM guesses
- **Transparent:** Every command shown before execution
- **Deterministic:** Prefer wiki-backed answers over LLM
- **Safe:** Never modify system without explicit approval
- **Honest:** Don't claim capabilities we don't have

---

## License

GPL-3.0-or-later - See [LICENSE](LICENSE)

---

## Credits

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Ollama](https://ollama.ai/) - Local LLM runtime
- [Arch Wiki](https://wiki.archlinux.org/) - Authoritative documentation
- Community testing and feedback

---

## Disclaimer

**This is experimental software. Use at your own risk.**

- No warranty of any kind
- May contain bugs or incomplete features
- Not suitable for production systems
- Commands may be incorrect (always verify before running)
- LLM answers may hallucinate (use wiki-backed answers when available)

**If you need production-ready system management, use:**
- Standard Arch Linux tools (pacman, systemctl, journalctl)
- Cockpit or similar web UIs
- Commercial monitoring solutions

Anna is for experimentation and learning, not production deployment.

---

**Made for Arch Linux enthusiasts who want to explore local LLM-based system assistance, understand the limitations, and enjoy the development process.**
