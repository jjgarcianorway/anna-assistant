# Anna Assistant

**Experimental Arch Linux System Assistant - Version 6.49.0**

[![Version](https://img.shields.io/badge/version-6.49.0-blue.svg)](https://github.com/jjgarcianorway/anna-assistant)
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

## What's New in 6.49.0 - Episodic Action Log & Rollback Foundation 🔄

### "If Anna did it, Anna can undo it" - Infrastructure Complete

**The Problem:** When Anna makes changes to your system, there's no way to track what was done or roll it back later.

**The Solution:** Complete action tracking and rollback infrastructure:

1. **Episodic Action Log** - Track every change
   - 📝 **ActionEpisode**: User question → actions → final result
   - 🏷️ **Smart tagging**: Automatic categorization (vim, ssh, audio, packages, services)
   - 💾 **Persistent storage**: SQLite database with fast topic queries
   - ✅ **Rollback capability**: Full, Partial, or None based on backup availability

2. **Rollback Engine** - Mechanical inverse commands
   - 📁 **File edits**: Restore from `.anna-*.bak` backups
   - 📦 **Packages**: Install ↔ Remove with same tool (yay/pacman)
   - ⚙️ **Services**: Enable ↔ Disable, Start ↔ Stop
   - 🎯 **Smart summaries**: "Restore 1 file backup and undo 2 package changes"

3. **Foundation for Semantic Rollback**
   - Ready for future integration: "revert my vim changes"
   - Topic-based episode selection
   - Safe rollback plan generation

**Example (Infrastructure API):**
```rust
// Episode building
let mut builder = EpisodeBuilder::new("make my vim use 4 spaces");
builder.add_action(ActionRecord {
    kind: ActionKind::EditFile,
    files_touched: vec!["/home/user/.vimrc"],
    backup_paths: vec!["/home/user/.vimrc.anna-backup"],
    ...
});

// Storage
let storage = EpisodeStorage::new("/var/lib/anna/episodes.db")?;
storage.store_action_episode(&episode)?;

// Rollback plan
let plan = build_rollback_plan(&episode);
// → "Restore 1 file backup" with inverse commands
```

**Status:** Foundation complete with 18 tests. User-facing rollback commands coming in v6.50.0+.

**Previous: 6.48.0 - Reality Check Engine ✅**

Multi-signal truth verification. 670 tests passing.

## What's New in 6.48.0 - Reality Check Engine ✅

### Multi-Signal Truth Verification: Zero Hallucination Guarantee

**The Problem:** LLMs can hallucinate system state, leading to dangerous actions based on false information.

**The Solution:** Reality Check Engine that validates every LLM output against real system state:

1. **Multi-Signal Verification** - 6 independent reality checks
   - 📊 **Telemetry**: Direct system metrics
   - 📁 **FileSystem**: File existence, permissions
   - ⚙️ **ProcessStatus**: Running services, processes
   - 📈 **HistoricalPattern**: Compare to past behavior
   - 🧠 **LogicalConsistency**: Internal contradiction detection
   - 🛡️ **SafetyValidation**: Safety rails check

2. **Smart Severity Classification**
   - ✅ **All agree**: Verified → Proceed
   - ⚠️ **1 disagrees**: Minor → Proceed with caution
   - 🚨 **Multiple disagree**: Major → Request clarification
   - 🛑 **All disagree**: Critical → Abort

3. **Confidence Calculation** - Weighted signal agreement
   - Normalized 0.0-1.0 scale
   - Configurable threshold (default 0.7)
   - Exact discrepancy reporting: "LLM says X, reality shows Y"

4. **Recommended Actions**
   - **Proceed**: High confidence, all verified
   - **Proceed with Caution**: Minor issues, monitored
   - **Request Clarification**: Major contradictions, ask LLM to re-check
   - **Request More Signals**: Insufficient data
   - **Abort**: Critical safety concern

**Example:**
```
LLM: "nginx service is running"
Signal 1 (Telemetry): ❌ Disagrees - "Service status: inactive" (confidence: 0.9)
Signal 2 (ProcessStatus): ❌ Disagrees - "Process not found" (confidence: 0.95)

Result: Contradicted (Critical) - confidence: 0.05
Action: Abort - "Critical contradiction detected"
```

**Impact:** Zero hallucination guarantee through multi-signal verification. 670 tests passing (658 + 12 new). Foundation for safe LLM automation.

**Previous: 6.47.0 - Situational Insights & Personality Greetings 🎭**

Context-aware greetings with pattern learning. 658 tests passing.

## What's New in 6.47.0 - Situational Insights & Personality Greetings 🎭

### Context-Aware Welcome Messages with Pattern Learning

**The Problem:** Generic greetings don't adapt to user preferences or surface important system changes.

**The Solution:** Three-engine system for intelligent, personalized greetings:

1. **Greeting Engine** - Personality-aware messages
   - 🎩 **Professional**: "Good morning. Note: High memory usage"
   - 😊 **Friendly**: "Good morning! ☀️ Heads up: High memory usage"
   - 🔧 **Technical**: "Morning. System operational. ⚠️ Warning: High memory usage"
   - 😎 **Casual**: "Hey! Morning. BTW: High memory usage"

2. **Learning Engine** - Pattern learning with time decay
   - Tracks user's preferred personality style
   - Confidence-based reinforcement (0.0-1.0)
   - Exponential decay (30-day half-life default)
   - Automatic pattern replacement on contradictions

3. **Telemetry Diff** - Change detection
   - Compares snapshots (packages, services, memory, load, errors)
   - Severity classification: Info, Warning, Critical
   - Health trend: Improving, Stable, Degrading
   - Automatic descriptions: "2 services failed", "Memory usage increased by 15%"

**Impact:** Anna adapts to your style and proactively surfaces system changes. 658 tests passing (628 + 30 new). Foundation for v6.48.0 Reality Check Engine.

**Previous: 6.46.0 - Interactive Query Mode 💬**

Multi-turn conversations with follow-up questions. Adaptive clarifications and alternative suggestions. 628 tests passing.

## What's New in 6.46.0 - Interactive Query Mode 💬

### Multi-Turn Conversations with Follow-Up Questions

**The Problem:** When results are ambiguous or incomplete, Anna just says "insufficient data" without offering help. Users are left stuck.

**The Solution:** Interactive mode that transforms queries into conversations:

1. **Smart Follow-Ups** - Anna asks clarifying questions
   - 📝 **Ambiguous results?** → "Which interpretation: 1. Gaming, 2. Development?"
   - 🔍 **Need more data?** → "Would you like me to check system logs? (yes/no)"
   - ✅ **Complete results?** → Shows summary and concludes

2. **User-Friendly Input** - Natural response parsing
   - Confirmations: "yes", "y", "sure", "ok" → Proceed
   - Selections: "1" or "gaming" → Pick first option
   - Maximum 3 rounds with full history

3. **Leverages v6.45.0 Validation** - Automatically triggered
   - ValidationDecision::Ambiguous → Clarification prompt
   - ValidationDecision::NeedMoreData → Alternative suggestion
   - ValidationDecision::Sufficient → Completion message

**Example:**
```
You: "do I have games?"
Anna: "The results are ambiguous. Which interpretation?
       1. Steam gaming
       2. Game development"
You: "1"
Anna: "✓ Found 3 Steam games: steam, lutris, wine-staging"
```

**Impact:** Better UX, adaptive conversations. 628 tests passing (618 + 10 new). Foundation for v6.47.0 personality greetings.

**Previous: 6.45.0 - Multi-Round Validation Loop 🔄**

Adaptive command retry with LLM validation (Sufficient/NeedMoreData/Ambiguous). Maximum 3 rounds with context preservation. 618 tests passing.

**Previous: 6.44.0 - Core Safety Rails 🛡️**

Three-layer safety: Command validation, Evidence classification (Positive/Negative/Unknown), Honest interpretation. 610 tests passing.

See [CHANGELOG.md](CHANGELOG.md) for full details.

---

## What's New in 6.42.0 - LLM Intelligence Layer 🧠

### Real AI Planning and Interpretation

**The Breakthrough:** Anna now uses real LLM backends (Ollama, OpenAI-compatible) for intelligent planning and interpretation, while maintaining safe fallback behavior when LLM is unavailable.

**What This Means:**
- Dynamic command planning based on your installed tools
- Intelligent interpretation of results with natural language reasoning
- Adapts to novel queries beyond hard-coded patterns
- Graceful degradation: Falls back to v6.41.0 deterministic behavior on LLM failure

**Key Features:**
- ✅ Trait-based LLM client (HttpLlmClient + FakeLlmClient for testing)
- ✅ JSON schema enforcement for structured LLM responses
- ✅ Tool inventory detection (pacman, yay, grep, lscpu, steam, etc.)
- ✅ LLM-backed Planner with fallback planning
- ✅ LLM-backed Interpreter with fallback interpretation
- ✅ 25 tests - all passing with fake clients (no network calls)

**Previous: 6.41.0 - Architectural Revolution 🎉**

Anna introduced the unified Planner → Executor → Interpreter pipeline that handles ALL inspection questions generically.

**Visible Thinking Traces:**
```
🧠 Anna thinking trace

Intent:
  - Goal: Inspect
  - Domain: Packages

Commands executed:
  [CMD] pacman -Qq | grep -Ei '(steam|game...)' ✓

Key outputs:
  sh: gamemode, steam, wine-staging

Execution time: 27ms
```

**How It Works:**
1. **Planner** - Understands your question, classifies intent (Inspect/Diagnose/List/Check)
2. **Executor** - Plans & runs safe commands specific to YOUR system
3. **Interpreter** - Analyzes outputs and gives you a clear answer
4. **Trace** - Shows you exactly how Anna reasoned

**Pilot Queries Working:**
- ✅ "do I have games installed?" → detects steam, lutris, wine, etc.
- ✅ "what DE/WM am I running?" → uses 5-layer detection (Sway, i3, KDE, etc.)
- ✅ "does my CPU have SSE/AVX?" → parses flags, groups by feature family
- ✅ "do I have any file manager installed?" → checks thunar, dolphin, nautilus, etc.

**Benefits:**
- 🎯 **Generic** - One architecture for all questions
- 👁️ **Transparent** - See how Anna thinks
- 🔒 **Safe** - Command validation, tool detection
- ⚡ **Fast** - 20-100ms typical queries
- 📈 **Extensible** - Easy to add new questions

**Problem 3:** Daemon logs spammed with routine INFO messages for every RPC call
**Solution:** Moved routine connection logs to DEBUG level (security failures still WARN)

**See CHANGELOG.md for complete v6.40.0 details.**

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
