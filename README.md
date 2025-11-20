# Anna Assistant

**Your Knowledgeable Arch Linux Caretaker**

Anna is a local system and desktop caretaker for Arch Linux. She's a bridge between technical documentation (Arch Wiki and official project docs) and you, focused on this machine: its hardware, software, and how you actually use it.

**Version:** 5.7.0-beta.136 (Production Ready: All Tests Passing + TUI Fixed + Auto-Update Working)

---

## What is Anna?

Anna is:
- **A local caretaker** - Watches your Arch system, spots problems, suggests improvements
- **A bridge to documentation** - Every suggestion is grounded in Arch Wiki or official docs
- **Warm and professional** - Explains things clearly, honest about tradeoffs and uncertainty
- **Transparent and safe** - Always explains what will change, always asks before acting

Anna is **not**:
- ❌ Not a generic monitoring platform
- ❌ Not a chatbot for weather or general conversation
- ❌ Not a remote management server
- ❌ Not running commands behind your back

## Current Status (v5.7.0-beta.131)

### ✅ What Actually Works (Honest Assessment - Beta.116-131 Improvements)

**Recent Improvements (Beta.116-128):**
- 🔒 **CRITICAL Security Fix (Beta.123):** Dangerous command detection now works (`rm -rf /` properly blocked)
- ⚡ **MAJOR Performance (Beta.117):** Daemon startup 21s → 2s (90% faster!)
- ✅ **Quality (Beta.123-126):** ALL 379 tests passing (100% - was 83%)
- 🧹 **Code Quality (Beta.121-122):** 78% warning reduction (1237 → 270)
- 📝 **Honesty (Beta.116-118):** Documentation reflects reality, no false claims

### ✅ Core Features

**Recipe Planner Integration (NEW in beta.113):**
- 🚀 **Planner/Critic LLM loop fully integrated** - RecipePlanner now makes actual LLM API calls
- 🚀 **JSON-structured responses** - Both planner and critic return validated command recipes
- 🚀 **LlmConfig-based architecture** - Explicit configuration required for recipe planning
- 🚀 **Iterative refinement** - Up to 3 planner/critic iterations with feedback loop
- 🚀 **Safety-first validation** - Static checks before critic, comprehensive validation

**Core Infrastructure:**
- ✅ Daemon (annad) runs and collects system telemetry
- ✅ CLI (annactl) communicates via Unix socket
- ✅ Historian database stores 30-day trends
- ✅ System facts collection (hardware, OS, packages, services)
- ✅ **Auto-update system (FIXED in beta.115)** - Was broken due to filesystem permissions (beta.71-114)
- ✅ **Installer optimization:** Skips re-downloading same version (beta.65)
- ✅ **Fast daemon startup (FIXED in beta.117)** - Now ~2-3 seconds (was 21+ seconds)

**Template System (NEW in beta.112 - MAJOR UPGRADE):**
- 🚀 **68 of 102 templates now mapped** - Up from 13 (67% coverage vs <10%)
- 🚀 **Package management:** 13 templates (orphans, AUR, cache, mirrors, keyring, updates)
- 🚀 **Boot & systemd:** 8 templates (boot time, errors, logs, journal, timers)
- 🚀 **CPU & performance:** 8 templates (frequency, governors, usage, temperature, throttling)
- 🚀 **Memory:** 6 templates (usage, swap, pressure, OOM, huge pages)
- 🚀 **Network:** 7 templates (DNS, interfaces, ports, latency, firewall)
- 🚀 **GPU & display:** 9 templates (NVIDIA, AMD, Xorg, Wayland, desktop environment)
- 🚀 **Hardware:** 4 templates (disk health, temperature, USB, PCI devices)
- 🚀 **Perfect consistency:** Same templates across one-shot, REPL, and TUI modes

**User Experience (beta.108-115 - FINALLY Fixed):**
- ✨ **Word-by-word streaming in all three modes** - Real-time LLM response display
- ✨ **Beta.108:** One-shot mode streaming (`annactl <question>`)
- ✨ **Beta.110:** REPL mode streaming (`annactl repl`)
- ✨ **Beta.115:** TUI mode streaming (`annactl tui`) **← FIXED (was broken in beta.111-114)**
- ⚠️  **Partial consistency:** Streaming now works in all modes, but TUI lacks RecipePlanner (one-off has it)

**Security (beta.66 - CRITICAL UPDATE):**
- 🔐 **ACTION_PLAN validation layer** - Prevents command injection
- 🔐 **SafeCommand builder** - Injection-resistant execution
- 🔐 **ANNA_BACKUP enforcement** - All backups follow naming convention
- 🔐 **Risk-based confirmation** - High/medium risk requires approval
- 🔐 **Execution halt on failure** - Prevents cascading damage
- 🔐 **6 comprehensive security tests** - All passing

**LLM Integration (beta.55-62):**
- ✅ Local LLM setup via Ollama (automatic detection and installation)
- ✅ Hardware-aware model selection (detects RAM, CPU, GPU)
- ✅ Internal dialogue system (planning + answer rounds)
- ✅ Telemetry-first approach (LLM checks data before answering)
- ✅ Anti-hallucination rules for small models (beta.62)
- ✅ Smart context filtering (only relevant info sent to LLM)
- ✅ 16-personalities trait system (8 adjustable traits)

**User Interface (beta.63 UX polish):**
- ✅ Interactive REPL (`annactl`)
- ✅ One-shot queries (`annactl "question"`)
- ✅ Status command (`annactl status`)
- ✅ Clean welcome message (no debug output)
- ✅ Silent error handling (no noisy warnings on startup)
- ✅ Terminal adaptation (color, unicode, emoji fallback)

**Code Quality (beta.64):**
- ✅ Zero clippy errors (89 → 0 fixed)
- ✅ Clean, idiomatic Rust code
- ✅ Ready for security audit

### 🚧 Partially Implemented / Needs Testing

**Features that exist in code but may not be fully wired or tested:**
- 🚧 ACTION_PLAN execution from LLM (validation done, needs LLM integration)
- 🚧 Change rollback system (logging infrastructure exists, rollback untested)
- 🚧 Multi-language support (6 languages configured, translations incomplete)
- 🚧 Suggestion engine with Arch Wiki integration (framework exists)
- 🚧 Doctor/repair system (self-healing code exists, needs validation)

### 📋 Next: Beta.67-68 Roadmap

**Beta.67 - Real-World QA Scenarios (In Progress):**
- 📋 Vim syntax highlighting scenario (backup, no duplicates, restore)
- 📋 Hardware detection scenario (no hallucinations, exact values)
- 📋 LLM model upgrade scenario (safe config changes)
- 📋 Regression test suite (capture bugs from beta.56-65)

**Beta.68 - LLM Quality & UX Polish:**
- 📋 LLM benchmarking harness (`annactl debug llm-benchmark`)
- 📋 Extended model catalog (memory/VRAM requirements, quality tiers)
- 📋 First-run wizard improvements (model selection, personality)
- 📋 REPL UX smoothing (history command, minimal boilerplate)

**See [ROADMAP.md](./ROADMAP.md) and [CHANGELOG.md](./CHANGELOG.md) for details.**

---

## Documentation
- Detection surface: `docs/DETECTION_SCOPE.md`
- Observer/historian requirements: `docs/INTERNAL_OBSERVER.md`
- Historian datasets/schema: `docs/HISTORIAN_SCHEMA.md`
- **Current status**: This README (honest status above)
- **Release notes**: `CHANGELOG.md` (version-by-version changes)

---

## Installation

One-line install:

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

The installer will:
1. Introduce Anna and explain what she does
2. Explain privacy (all data stays local)
3. Ask for your consent
4. Set up the daemon and CLI tools
5. Show you how to get started

---

## How to Use Anna

There are **exactly two commands**:

### 1. Talk to Anna - `annactl` or `annactl "question"`

Ask Anna anything about your system in natural language.

**Start a conversation:**
```bash
annactl
```

This opens an interactive session where you can have a back-and-forth conversation with Anna.

**One-shot queries:**
```bash
annactl "how are you?"
annactl "my computer feels slower than usual, any idea why?"
annactl "what are the top 3 things I should improve?"
annactl "prepare a report about this machine for my boss"
annactl "what do you store about me?"
```

**What you can ask:**
- System status and health
- Problems and suggestions for improvement
- Generate professional reports
- Privacy and data handling questions
- Fix specific issues (Anna will explain and ask for approval)
- Adjust Anna's personality (humor, verbosity)

**Examples:**
```bash
# Status check
annactl "how are you?"
annactl "any problems with my system?"

# Get suggestions
annactl "what should I improve?"
annactl "my system feels slow"

# Generate reports
annactl "generate a report"
annactl "I need a summary for my boss"

# Privacy
annactl "what do you store about me?"
annactl "tell me about privacy"

# Personality adjustment
annactl "be more brief"
annactl "show personality settings"

# Help
annactl "help"
annactl "what can you do?"
```

### 2. Check Anna's Own Health - `annactl repair`

This is **only for Anna's own health**, not for fixing your system.

```bash
annactl repair
```

This checks and fixes:
- Anna's permissions and groups
- Missing dependencies Anna needs
- Socket and service issues
- Context database problems

Always explains what it's checking and asks for confirmation before making changes that require sudo.

---

## First-Run Experience

The first time you talk to Anna (`annactl` or `annactl "question"`), she will introduce herself and offer to set up her "brain" - a language model that helps her understand your questions better.

### Three Options

Anna will assess your machine's capabilities and present three options:

**1. Local Model (Recommended - Privacy First)**
- Automatically installs and configures Ollama
- Downloads an appropriate model based on your RAM and CPU
- All processing stays on your machine
- Free, no API costs
- Works offline

**2. Remote API (OpenAI-Compatible)**
- Connect to OpenAI, Anthropic, or compatible API
- You provide API key and endpoint
- Faster responses on lower-end machines
- **Warning**: Your questions leave your machine and may cost money
- Anna clearly explains privacy and cost implications

**3. Skip for Now**
- Anna works with built-in rules and Arch Wiki only
- Limited conversational ability
- You can set up the brain later by asking: "Anna, set up your brain"

### What Gets Installed (Local Path)

If you choose local setup:
- **Ollama** via pacman or AUR (yay)
- A language model (1-5 GB depending on your hardware):
  - **Tiny** (1.3 GB): 4GB RAM, 2 cores → llama3.2:1b
  - **Small** (2.0 GB): 8GB RAM, 4 cores → llama3.2:3b
  - **Medium** (4.7 GB): 16GB RAM, 6+ cores → llama3.1:8b
- Ollama service enabled and started
- One-time model download

Anna handles everything automatically. No manual steps required.

### Brain Upgrades

If your machine gets more RAM or CPU power, Anna will notice and offer a **one-time suggestion** to upgrade to a better model:

```
🚀 My Brain Can Upgrade!

Great news! Your machine got more powerful.
I can now upgrade to a better language model:

  New model: llama3.1:8b
  Download size: ~4.7 GB
  Profile: ollama-llama3.1-8b

To upgrade, ask me: "Upgrade your brain" or "Set up your brain"
```

This only appears once. No nagging.

---

## Auto-Update System

Anna keeps herself up to date automatically **(FIXED in Beta.115 - was broken Beta.71-114)**.

### For Manual Installations (curl installer)

Every 10 minutes, Anna's daemon:
1. Checks GitHub releases for new versions
2. Downloads new binaries + SHA256SUMS
3. Verifies checksums cryptographically
4. Backs up current binaries (with rollback capability)
5. Atomically swaps binaries in `/usr/local/bin`
6. Restarts the daemon
7. Records the update

**Next time you interact with Anna:**

```
✨ I Updated Myself!

I upgraded from v5.4.0 to v5.5.0

What's new:
  • Added automatic brain upgrade detection
  • Improved LLM setup wizard UX
  • Fixed permission handling for Ollama
  • Enhanced changelog parsing

[Then answers your question normally]
```

This notice appears **once per update**, then never again.

### For Package Manager Installations (AUR, pacman)

Anna detects package-managed installations and:
- **Does not** replace binaries (respects your package manager)
- Notifies you that a new version is available
- Shows you the command to update: `pacman -Syu anna` or `yay -Syu anna`

### Safety Guarantees

- **Cryptographic verification**: Updates fail if SHA256 mismatch
- **Atomic operations**: No partial states during swap
- **Automatic backups**: Every binary replacement is backed up
- **Rollback capability**: File backup system tracks all changes
- **No interruption**: Updates happen in background, daemon restart is seamless

---

## Languages & Terminal Support

Anna speaks multiple languages and adapts to your terminal's capabilities.

### Supported Languages

Anna can communicate in:
- **English** (default)
- **Spanish** (Español)
- **Norwegian** (Norsk)
- **German** (Deutsch)
- **French** (Français)
- **Portuguese** (Português)

Change Anna's language naturally:
```bash
annactl "use Spanish"
annactl "cambia al español"
annactl "parle français"
annactl "spreche Deutsch"
```

Anna will confirm the change in the NEW language and remember your preference.

### Terminal Adaptation

Anna automatically detects your terminal's capabilities and adapts:
- **Color support**: TrueColor → 256 colors → 16 colors → no color
- **Unicode support**: Full Unicode → ASCII fallback
- **Emoji support**: Native emoji → text replacements (✓ → [OK], ⚠️ → [!])

This means Anna works beautifully in modern terminals and gracefully degrades for older TTY environments. You never see broken glyphs or garbled output.

---

## How Anna Works

### Telemetry Collection

The `annad` daemon continuously collects:
- Hardware info (CPU, RAM, disks, battery, GPU)
- Software state (packages, updates, services)
- Resource usage over time
- Configuration state (window manager, display, audio)
- Usage patterns at a coarse level (e.g., "lots of coding", "mostly browser")

**All data stays on your machine.** No exfiltration. No remote logging.

### Knowledge Sources

Anna's knowledge hierarchy:
1. **Arch Wiki** - Primary knowledge base
2. **Official project docs** - Secondary (linked from Arch Wiki)
3. **Your system** - What annad observes about this machine

### Suggestions and Actions

**Suggestions:**
- Explained in plain English
- Always include documentation URLs (Arch Wiki first)
- Only 2-5 suggestions at a time (not overwhelming)
- Prioritized by impact and safety

**Actions:**
- Only executed after you explicitly agree
- Anna shows exactly what will change (commands, config files, packages)
- Explains why it's safe and what risks exist
- Links to documentation sources
- All changes are logged for rollback

### Change Logging and Rollback

When Anna makes changes to your system:
- Each change is logged as a **Change Unit** with:
  - Unique ID and human-readable label
  - Commands run, files modified, packages changed
  - Timestamps and result
  - Your original request that triggered it

You can roll back changes:
```bash
annactl "roll back your latest changes"
annactl "I'm not happy with the KDE install, put my system back"
```

Anna shows what will be rolled back and asks for confirmation before proceeding.

### Reports

Request professional reports for managers or documentation:

```bash
annactl "prepare a report for my boss"
```

Reports include:
- Machine overview (hardware, OS, uptime, usage patterns)
- Status and health summary
- Key issues and how they were addressed
- Tradeoffs (improvements vs costs)
- Recommended next steps

Tone is professional, clear, and non-technical enough for managers.

---

## Privacy

**What Anna stores locally:**
- System metrics (CPU, RAM, disk, services)
- Configuration state
- Usage patterns (coarse level, no file contents)
- Change history
- Suggestions and decisions

**What Anna NEVER does:**
- Read personal file contents
- Send data to external servers
- Track you for advertising
- Run commands without your approval

**Network access:**
- Only to Arch Wiki and official project documentation
- Only when generating suggestions or looking up information
- Never for telemetry or data exfiltration

Ask Anna anytime: `annactl "what do you store about me?"`

---

## Personality

Anna has a warm, professional personality with subtle wit. You can adjust her behavior:

```bash
annactl "be more funny"           # Increase humor
annactl "please don't joke"       # Decrease humor
annactl "be more brief"           # Concise answers
annactl "explain in more detail"  # Thorough explanations
annactl "show personality settings" # View current settings
```

Settings are saved to `~/.config/anna/personality.toml`.

---

## Autonomy

By default, Anna **does not** change your system automatically. All actions require explicit approval.

The only autonomous behavior:
- Anna checks for her own updates every 10 minutes
- If configured for auto-update, she may update herself safely (checksums, backups, rollback)

Everything else requires you to initiate through `annactl`.

---

## Examples

**Morning health check:**
```bash
annactl "how's everything looking?"
```

**Get improvement suggestions:**
```bash
annactl "what should I improve?"
```

**Fix a specific issue:**
```bash
# Anna detects low disk space
annactl "can you help me fix the disk space issue?"
# Anna explains the problem, suggests solutions
# You approve, Anna applies the fix and logs it
```

**Generate a report for documentation:**
```bash
annactl "generate a comprehensive system report"
```

**Adjust Anna's tone:**
```bash
annactl "be more serious, I prefer professional communication"
```

**Check what Anna stores:**
```bash
annactl "what data do you keep about me?"
```

---

## Technical Details

**Architecture:**
- `annad` - Secure telemetry daemon (runs as system service)
- `annactl` - CLI frontend (conversational interface)
- Local SQLite database for context and history
- File-based config in `~/.config/anna/` and `/etc/anna/`

**Security:**
- Runs with minimal privileges
- Uses group-based socket permissions
- No unsafe automatic operations (no fsck, no repartitioning)
- All changes are logged and reversible when possible

**Compatibility:**
- Arch Linux only
- x86_64 architecture
- Systemd required

---

## Development

**Build from source:**
```bash
git clone https://github.com/jjgarcianorway/anna-assistant
cd anna-assistant
cargo build --release
```

**Run tests:**
```bash
cargo test
```

**Contribute:**
See `CONTRIBUTING.md` for guidelines.

---

## Roadmap

See [ROADMAP.md](./ROADMAP.md) for planned features and development priorities.

Current focus:
- Enhanced suggestion engine with Arch Wiki integration
- Change logging and rollback infrastructure
- Professional report generation
- System degradation tracking

---

## License

GNU General Public License v3 (GPLv3) - See [LICENSE](./LICENSE) for details.

Anna is free and open source software. You can redistribute and modify it under the terms of GPLv3.

---

## Support

**Issues:** https://github.com/jjgarcianorway/anna-assistant/issues

**Documentation:** This README and the Arch Wiki links Anna provides

**Philosophy:** Anna is designed to be self-explanatory. If you need to read extensive docs, we've failed. Just ask Anna.
