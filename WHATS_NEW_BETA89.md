# What's New in Beta.89

**Release Date:** 2025-11-19
**Focus:** Tiny CLI Surface & Natural Language Control

---

## Overview

Beta.89 is a fundamental refactoring that transforms Anna from a complex CLI tool into a pure natural language assistant.

**Key Achievement:** The entire user interface now consists of exactly **3 commands** - everything else happens through conversation.

---

## The Tiny CLI Revolution

### ONLY 3 Commands Exist

```bash
# 1. Interactive REPL (conversational mode)
annactl

# 2. One-shot status check
annactl status

# 3. One-shot question (any language)
annactl "how do I install vim?"
annactl "cómo configuro la red wifi?"
annactl "comment installer firefox?"
```

**That's it. Nothing else.**

### What's Gone

All subcommands from Beta.88 are **removed**:

- ❌ `annactl personality show/set/adjust/reset/validate/export`
- ❌ `annactl help`
- ❌ `annactl version`
- ❌ Any other subcommands

These capabilities now exist **only through natural language** inside the REPL.

---

## Natural Language Personality Control

The 16-trait personality system remains, but control is now entirely conversational.

### Inside the REPL

```
You: show your personality

Anna: My current personality configuration:

1. introvert_vs_extrovert: 3/10  ██████░░░░
   Extrovert style. Frequent, chatty communication.

2. formal_vs_casual: 5/10  ██████████
   Balanced between formal and casual tone.

[... all 16 traits ...]

You: be more concise

Anna: Adjusted concise_vs_verbose from 6 to 4.
I'll now give shorter, more direct answers.

You: set your warm_vs_cold to 8

Anna: Updated warm_vs_cold to 8/10.
I'll respond with more warmth and empathy.

You: reset your personality

Anna: Reset all 16 traits to defaults.
Personality configuration restored.

You: validate your personality

Anna: Personality validation complete.
Warning: You have both minimalist (2/10) and detailed (9/10) traits high.
This may cause inconsistent response styles.
```

### The 16 Traits

Each trait ranges from 0 (left descriptor) to 10 (right descriptor):

1. `introvert_vs_extrovert` - Communication frequency
2. `formal_vs_casual` - Professional vs friendly tone
3. `strict_vs_lenient` - Rule adherence
4. `concise_vs_verbose` - Response length
5. `optimistic_vs_realistic` - Outlook style
6. `emotional_vs_neutral` - Affect level
7. `serious_vs_playful` - Humor usage
8. `technical_vs_plain` - Jargon vs simplification
9. `cautious_vs_bold` - Risk tolerance
10. `warm_vs_cold` - Empathy level
11. `proactive_vs_passive` - Initiative taking
12. `minimalist_vs_detailed` - Information density
13. `dry_vs_humorous` - Wit and jokes
14. `adaptive_vs_fixed` - Flexibility
15. `directive_vs_socratic` - Teaching style
16. `empathetic_vs_direct` - Support vs efficiency

---

## Multilingual Support

### Language Detection and Switching

Anna starts in your system locale language (English for `en_US`, etc.).

To change language, just ask naturally:

```
You: please speak French

Anna: J'ai changé ma langue en français. Comment puis-je vous aider?

You: spreek Nederlands

Anna: Ik spreek nu Nederlands. Hoe kan ik u helpen?

You: habla español

Anna: Ahora hablo español. ¿Cómo puedo ayudarte?
```

Language preference is saved and persists across sessions.

### Exit Words (Multilingual)

Anna recognizes exit intent in any active language:

- **English:** exit, quit, bye, goodbye
- **Spanish:** salir, adiós, chao
- **French:** sortir, au revoir, quitter
- **Dutch:** uitgang, tot ziens, stoppen
- **German:** ausgang, tschüss, beenden
- **Italian:** uscita, ciao, arrivederci

Just type the word and Anna exits cleanly, no confirmation needed.

---

## LLM Purpose and Boundaries

### What the LLM Does

The local LLM behind Anna has **exactly 3 roles**:

1. **Interpret Intent**
   - Map natural language to required telemetry queries
   - Identify risk level (safe, medium, risky)
   - Determine what checks are needed

2. **Transform Results**
   - Take raw telemetry data
   - Format into clear, structured answers
   - Add backup/restore instructions
   - Cite Arch Wiki references

3. **Mark Risk**
   - Detect dangerous operations
   - Demand backup steps for file changes
   - Warn about system modifications

### What the LLM Never Does

- ❌ Guess system state
- ❌ Invent hardware details
- ❌ Fabricate log contents
- ❌ Claim to have run commands
- ❌ Execute anything directly

**Everything comes from real telemetry or explicit commands proposed to the user.**

---

## Telemetry-First Principle

Every system-related answer is driven by actual measurements from `annad`.

### Answer Workflow

1. User asks a question in natural language
2. LLM generates a telemetry plan (what to check)
3. `annad` executes telemetry and returns JSON
4. LLM uses that data to format the answer
5. `annactl` displays the structured response

### Example

```
You: why is my system slow?

[Anna internally requests telemetry:]
- CPU usage and load averages
- Memory usage and swap
- Disk I/O statistics
- Process list sorted by resources
- Recent systemd failures

[annad provides real data]

Anna: I checked your system and found:

**What I checked:**
- CPU load: 0.42, 0.38, 0.31 (5min, 10min, 15min)
- Memory: 6.2 GB used / 16 GB total (38%)
- Swap: 0 GB used
- Disk I/O: Normal activity
- Top processes: firefox (892 MB), code (654 MB)

**Summary:**
Your system is not slow. Load averages are low and you have 9.8 GB free RAM.
The perception of slowness may be application-specific rather than system-wide.

**Explanation:**
[detailed analysis based on real telemetry]

**References:**
- https://wiki.archlinux.org/title/System_monitoring
```

---

## Structured Answer Format

Every Anna answer follows this 7-section structure:

### 1. Header Line (Status Bar)
- Anna version
- Active LLM model
- System status (OK / WARN / CRIT)
- Topic focus

### 2. Short Summary
- What you asked
- What Anna checked
- Main conclusion (1-3 lines)

### 3. "What I checked" Section
- Bullet list of telemetry sources
- Example: "Checked vim package installation"
- Example: "Located config files: ~/.vimrc, /etc/vimrc"

### 4. "Suggested commands" Section
- Numbered list of copy-paste ready commands
- Includes backup commands where applicable
- Uses correct backup naming format

### 5. "How to undo" Section
- Explicit restore commands
- Or states "No persistent changes, nothing to undo"

### 6. "Explanation" Section
- Clear technical explanation in active language
- Short paragraphs and bullet lists
- No fluff or unnecessary prose

### 7. "References" Section
- Arch Wiki URLs (required for command answers)
- Official upstream documentation
- One URL per line

---

## Backup and File Marking Rules

### Backup Naming Convention

When Anna suggests modifying a file, backups use this exact format:

```
<original_filename>.ANNA_BACKUP.YYYYMMDD-HHMMSS
```

Examples:
```bash
/etc/pacman.conf.ANNA_BACKUP.20251119-143022
~/.vimrc.ANNA_BACKUP.20251119-143530
/etc/systemd/system/something.service.ANNA_BACKUP.20251119-144105
```

### File Change Markers

Every Anna-modified config gets a comment marker:

```bash
# For shell configs and many others:
# Modified by Anna on 2025-11-19 14:30:22

# For INI-style configs:
; Modified by Anna on 2025-11-19 14:30:22

# For other syntaxes: appropriate comment character
```

### Restore Instructions

Every answer that modifies files includes explicit restore steps:

```bash
# To undo and restore previous version:
cp /etc/pacman.conf.ANNA_BACKUP.20251119-143022 /etc/pacman.conf
```

---

## Model Selection and Installation

### Hardware Detection

Anna detects your hardware capabilities:
- CPU cores and threads
- Total RAM
- GPU presence and VRAM (if available)

### Model Catalog

Anna maintains a catalog of LLM models:
- **Families:** Llama 3.2, Qwen 2.5, Mistral, Gemma
- **Sizes:** 3B, 7B, 8B, 14B, 70B parameters
- **Requirements:** RAM/VRAM needs, latency estimates

### First-Run Setup

On first run, Anna:
1. Checks your hardware
2. Recommends 1-2 suitable models:
   - "Fast but good enough" (e.g., llama3.2:3b)
   - "Slower but higher quality" (e.g., qwen2.5:14b)
3. Uses Ollama to download and install your choice
4. Verifies installation

### Quality Standards

Anna **refuses to use obviously underpowered models**.

If your hardware cannot support a viable model, Anna will:
- Explain the limitation
- Suggest hardware upgrades
- Refuse to start with inadequate LLM

### Model Upgrades

If Anna detects hardware improvements (new RAM, new GPU):
- Suggests upgrading to a stronger model
- Explains the quality benefits
- Handles installation if approved

---

## Arch Wiki as Authority

### Primary Reference Source

For all Linux and Arch-related answers:
- **Arch Wiki is the canonical source**
- Official upstream docs (linked from Arch Wiki) are also valid
- Random blogs and forums are NOT used as references

### Citation Requirements

Every answer that proposes commands **MUST include**:
- One or more Arch Wiki URLs, or
- Official documentation URLs referenced by Arch Wiki

### Consistency Enforcement

Commands, flags, and config examples must match Arch Wiki content.

**If there is a conflict between the LLM's training data and Arch Wiki, the Wiki wins.**

---

## Quality Assurance and Validation

### Extended Question Dataset

Beta.89 includes **1000+ validated questions** covering:

- **Network:** WiFi, Ethernet, VPN, DNS, routing
- **Packages:** pacman, AUR, dependencies, conflicts
- **Display:** X11, Wayland, drivers, resolution
- **Audio:** PipeWire, ALSA, PulseAudio
- **Users:** permissions, groups, sudo
- **Services:** systemd units, enabling, masking
- **Troubleshooting:** logs, debugging, recovery
- **Optimization:** performance tuning, monitoring
- **Edge cases:** weird hardware, exotic configs

### Validation Criteria

Automated validation scripts check every answer for:

1. **Structure completeness:**
   - All 7 sections present
   - References section non-empty (for command answers)

2. **Backup rules followed:**
   - Correct `.ANNA_BACKUP.YYYYMMDD-HHMMSS` naming
   - Restore commands provided

3. **Forbidden phrases absent:**
   - No "I ran this command"
   - No claims of execution
   - No invented telemetry

### Success Thresholds

- **90%+ pass rate:** Excellent, ship immediately
- **80%+ pass rate:** Acceptable for beta release
- **<80% pass rate:** Must fix before releasing

---

## Developer Notes

### Files Modified

**crates/annactl/src/main.rs:**
- Removed all subcommands except `status`
- Simplified CLI to 3 modes (REPL, status, one-shot question)
- Removed personality CLI routing

**crates/annactl/src/repl.rs:**
- Added natural language personality control
- Added multilingual exit word detection
- Added language switching logic

**crates/anna_common/src/answer_formatter.rs** (new):
- 7-section structured answer formatting
- Arch Wiki reference enforcement
- Backup naming validation

**crates/anna_common/src/model_selection.rs** (enhanced):
- Hardware capability detection
- Model catalog with requirements
- First-run setup wizard
- Model upgrade recommendations

**scripts/validate_large_scale.sh** (enhanced):
- Validates 1000+ questions
- Checks answer structure
- Verifies backup rules
- Reports success rate

**data/all_questions.json** (extended):
- Expanded to 1000+ questions
- Categorized and tagged
- Real-world scenarios included

### Testing Commands

```bash
# Run validation suite (1000 questions)
./scripts/validate_large_scale.sh data/all_questions.json 1000

# Check success rate
cat validation_results.md | grep "Success Rate"

# Test personality control
annactl
> show your personality
> be more concise
> set technical_vs_plain to 8
> exit

# Test multilingual
annactl "comment installer vim?"
annactl "cómo funciona pacman?"

# Test status
annactl status
annactl status --json
```

---

## Migration from Beta.88

### Breaking Changes

**CLI subcommands removed:**
- `annactl personality <action>` → Natural language in REPL
- `annactl help` → Use `annactl` and ask "help"
- `annactl version` → Use `annactl --version` (standard flag)

**Adaptation required:**
- Scripts that used `annactl personality show` must be rewritten to use natural language via REPL
- Consider this a simplification, not a regression

### Backwards Compatibility

- **Personality database format:** Unchanged
- **Configuration files:** Compatible
- **Daemon protocol:** Compatible
- **Existing personality.toml files:** Still work

### Auto-Update

Razorback and other systems on older betas will auto-update to Beta.89 within 10 minutes of release.

---

## Known Limitations

### Deferred to Beta.90+

- **Large-scale validation report:** 1000-question run produces comprehensive benchmark, but analysis dashboard not yet built
- **Multi-model ensemble:** Anna uses one model at a time, no fallback chain yet
- **Personality import from TOML:** Export works, import must be done by editing database directly
- **Interactive personality wizard:** Coming in Beta.90

### Current Limitations

- **Async runtime issue from Beta.88:** Still present in some edge cases, use release builds
- **Model quantization:** Not yet automatic, manual setup required for speed gains
- **GPU detection:** Basic, may not recognize all hardware correctly

---

## What's NOT in Beta.89

**Features explicitly deferred to later releases:**

- **Telemetry execution:** LLM plans telemetry, but `annad` doesn't execute all checks yet (Beta.90)
- **Command execution:** Anna only proposes commands, doesn't run them (Phase 6+)
- **RAG / Knowledge base:** Reddit QA dataset exists but not yet indexed for retrieval (Beta.93)
- **Conversation context across sessions:** Each REPL session is independent (Beta.95+)
- **Web search integration:** Not implemented (Beta.95+)

---

## Command Reference

### The Only 3 Commands

```bash
# 1. Interactive REPL
annactl

# 2. One-shot status
annactl status
annactl status --json

# 3. One-shot question
annactl "your question here"
annactl "any language works"
```

### Inside the REPL (Natural Language)

```
# Personality management
show your personality
set <trait> to <0-10>
be more <adjective>  (e.g., "be more concise")
reset your personality
validate your personality

# Language switching
please speak <language>
habla español
parla italiano
spreek Nederlands

# System queries
why is my system slow?
how do I install vim?
my WiFi doesn't work
check my disk space

# Exit
exit, quit, bye, goodbye, salir, adiós, etc.
```

---

## Philosophy

Beta.89 represents Anna's evolution from a **CLI tool with many commands** to a **conversational assistant with one interface: natural language**.

This design acknowledges that:
- Humans think in sentences, not subcommands
- Context and conversation are more natural than `--flags`
- A tiny CLI surface is easier to learn and remember
- Everything complex should be discoverable through asking

---

**Questions or issues?** Report at https://github.com/jjgarcianorway/anna-assistant/issues
