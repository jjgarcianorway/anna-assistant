# Anna Assistant - Beta.200 Architecture Specification

**Status:** Authoritative architectural mandate
**Version:** 5.7.0-beta.200
**Date:** 2025-11-20
**Supersedes:** All previous architectural documents

---

## I. EXECUTIVE SUMMARY

Beta.200 represents a complete architectural reset for Anna Assistant. This release removes all experimental features, consolidates the codebase around three simple commands, and enforces strict telemetry-first behavior with zero hallucinations.

**Core principle:** Anna is a simple, predictable, telemetry-driven Arch Linux assistant. Nothing more.

---

## II. FIXED ARCHITECTURE (NON-NEGOTIABLE)

### 2.1 Command Interface

Anna has **EXACTLY THREE** commands:

1. **`annactl`** - Full-screen TUI REPL (interactive mode)
2. **`annactl status`** - One-shot system summary
3. **`annactl "<question>"`** - One-shot natural-language question

**Removed commands:**
- All personality commands (chronos, collective, conscience, consensus, empathy)
- All admin commands (init, install, upgrade, suggest, discard, repair)
- All experimental commands (autonomy, learning, predictive, sentinel, steward, mirror)
- All hidden commands except `--version`, `--help`

**Hidden implementation details:**
- `ping` - Health checks (not user-facing)
- `--version` - Standard version flag
- `--help` - Standard help flag

### 2.2 Components

**annactl** (CLI client):
- Minimal entry point
- Dispatches to three modes: TUI, status, one-shot
- No command logic in main.rs
- All functionality in modules

**annad** (daemon):
- Root-privileged telemetry collector
- Persistent state storage
- Future: safe command executor
- No LLM logic (LLM runs in annactl)

**Local LLM** (Ollama):
- Model selection: best for hardware (Qwen2.5 7B/14B, Llama 3.2 8B, Mistral 7B, Gemma 9B)
- Must verify/restore on every run
- Must refuse to operate without valid model
- Anna NEVER runs without local LLM

---

## III. LLM ROLES (VERY STRICT)

The LLM does **ONLY THREE THINGS:**

### 3.1 Intent Interpretation

Parse natural language to understand what the user wants.

Examples:
- "how much RAM?" → query: system_memory
- "install docker" → action: install_package(docker)
- "why is my system slow?" → analysis: performance_investigation

### 3.2 Recipe Generation

Produce deterministic JSON recipes:

```json
{
  "recipe": {
    "intent": "user query summary",
    "steps": [
      {
        "id": "telemetry_check_1",
        "type": "telemetry",
        "description": "Check RAM usage",
        "commands": ["free -h"],
        "risk": "none"
      },
      {
        "id": "action_enable_swap",
        "type": "action",
        "description": "Enable swap file",
        "requires_confirmation": true,
        "risk": "medium",
        "backup": ["cp /etc/fstab /etc/fstab.backup"],
        "commands": ["fallocate -l 4G /swapfile", "chmod 600 /swapfile", ...],
        "restore": "restore from /etc/fstab.backup"
      }
    ],
    "citations": ["https://wiki.archlinux.org/title/Swap"]
  }
}
```

Recipe types:
- **telemetry**: Pure data collection (risk: none)
- **action**: System modification (risk: low/medium/high)

### 3.3 Answer Generation

Using ONLY the telemetry returned from annad, generate natural language answer.

**NEVER:**
- Hallucinate system state
- Guess configuration values
- Invent file paths
- Ignore backup rules
- Produce non-deterministic recipes

**ALWAYS:**
- Base answer on real telemetry
- Mark unknown values as "Unknown"
- Include citations to Arch Wiki
- Provide rollback instructions for actions

---

## IV. TELEMETRY-FIRST RULE

Every answer MUST be based on REAL telemetry data.

### 4.1 Workflow

```
User Query
    ↓
LLM: Parse Intent
    ↓
LLM: Generate Telemetry Plan (JSON recipe with telemetry steps)
    ↓
annad: Execute Telemetry Collection
    ↓
annad: Return Structured JSON Results
    ↓
LLM: Generate Final Answer (using ONLY real telemetry)
    ↓
annactl: Render Answer in TUI/CLI
```

### 4.2 Missing Telemetry

If required telemetry is unavailable:

```
User: "what GPU do I have?"
LLM: [generates telemetry plan]
annad: [fails to collect GPU info]
LLM: "I don't have GPU telemetry. Requesting: lspci, nvidia-smi, etc."
```

---

## V. RECIPE SYSTEM

### 5.1 Deterministic Recipes

Hard-coded recipes for common tasks (current: 77 recipes).

Examples:
- Package management (install, remove, update)
- Service management (enable, disable, start, stop)
- Network diagnostics
- GPU driver installation
- Development environment setup

### 5.2 Recipe Schema

See `docs/RECIPES_ARCHITECTURE.md` for full schema.

Key fields:
- `steps`: Array of command sequences
- `risk`: none/low/medium/high
- `backup`: Commands to run before action
- `restore`: How to undo changes
- `citations`: Arch Wiki links

### 5.3 Recipe Coverage

Current: 77 recipes
Goal (Beta.200): 100+ recipes

Priority areas:
- Core system management
- Hardware configuration
- Development environments
- Desktop environments
- Security tools

---

## VI. TUI REDESIGN (MANDATORY)

### 6.1 Modern Interface

Quality level: claude-cli, codex, modern ratatui apps

**Requirements:**

1. **Header bar:**
   - Version (v5.7.0-beta.200)
   - Username@hostname
   - LLM model (Qwen2.5:14B)
   - Health status (✓ Healthy / ⚠ Degraded / ✗ Broken)
   - Current time (HH:MM)
   - Theme colors (green/yellow/red/cyan/magenta)

2. **Body:**
   - Scrolling message history
   - Markdown-like rendering (bold, code blocks, lists)
   - NO backticks spam
   - Consistent spacing
   - Clear separation between user/assistant messages

3. **Thinking animation:**
   - Spinner or pulsing dots
   - Identical in TUI AND one-shot mode
   - Shows LLM is processing
   - NO fake delays

4. **Colors:**
   - Green: OK, healthy, success
   - Yellow: Warnings, medium risk
   - Red: Errors, high risk, critical
   - Cyan: Paths, commands, technical details
   - Magenta: Personality traits, special features

5. **Personality view (future):**
   - Horizontal bars 0-10
   - 16 traits (curiosity, caution, proactivity, etc.)
   - ASCII visualization (no complex graphics)

### 6.2 Consistency Requirement

**CRITICAL:** TUI mode and one-shot mode MUST produce:
- Same recipe for same query
- Same telemetry requests
- Same answer (word-for-word)
- Same thinking animation
- Same citations

**NO DIVERGENCE.**

---

## VII. CONSISTENCY: TUI vs ONE-SHOT

### 7.1 Shared Query Handler

Both modes must use the SAME query processing pipeline:

```
UnifiedQueryHandler::process(query)
    ↓
Same intent detection
    ↓
Same recipe generation
    ↓
Same telemetry collection
    ↓
Same answer generation
    ↓
Different rendering (TUI scrolling vs one-shot print)
```

### 7.2 Testing Strategy

Every query must be tested in BOTH modes:

```rust
#[test]
fn test_query_consistency() {
    let query = "how much RAM do I have?";
    let tui_result = tui::process_query(query);
    let oneshot_result = oneshot::process_query(query);
    assert_eq!(tui_result.answer, oneshot_result.answer);
    assert_eq!(tui_result.recipe, oneshot_result.recipe);
}
```

---

## VIII. WELCOME MESSAGES (DETERMINISTIC)

Every launch (TUI or one-shot) shows startup summary.

### 8.1 System Change Detection

Detect changes since last run:

```
$ annactl

Anna v5.7.0-beta.200 | @user@hostname | Qwen2.5:14B | ✓ Healthy | 14:23

System changes since last run (2 hours ago):
  • Storage: 78% → 82% (+4%, 15.2 GB used)
  • Boot time: 8.2s → 6.8s (improved by 1.4s)
  • Failed services: nginx.service (failed 3 times in 24h)
  • Security updates: 4 packages waiting

>
```

### 8.2 Deterministic Insights

Based on telemetry, NOT random comments:

**DO:**
- "Storage increased by 17% since last week"
- "Boot time improved by 1.4 seconds"
- "Network throughput decreased significantly"

**DON'T:**
- Random jokes
- Non-deterministic greetings
- Fake personality

**Exception:** One playful comment IF personality trait "humor" > 7.

---

## IX. LANGUAGE BEHAVIOR

### 9.1 Language Detection

Anna replies in the last language used by the user.

**Switching:**
- User: "speak English" → Anna switches to English
- User: "habla español" → Anna switches to Spanish
- User: "parle français" → Anna switches to French

**Storage:**
- Language preference stored in daemon state
- Persists across sessions

**Commands:**
- Command names remain English ("annactl status")
- Answer content uses active language

---

## X. EXIT BEHAVIOR

### 10.1 Natural Exit

REPL exits when user writes:

- exit
- quit
- bye
- adiós
- salir
- ciao
- au revoir
- good night (context-aware)

Case-insensitive, trim whitespace.

### 10.2 Graceful Shutdown

1. Save conversation history
2. Update language config
3. Flush state to disk
4. Clean exit (exit code 0)

---

## XI. MODULE BOUNDARIES

### 11.1 Current Structure

```
crates/annactl/src/
├── main.rs              # Entry point only
├── cli.rs               # Argument parsing
├── runtime.rs           # Dispatch logic
├── tui/                 # TUI implementation (KEEP, ENHANCE)
│   ├── mod.rs
│   ├── event_loop.rs
│   ├── render.rs
│   ├── input.rs
│   ├── llm.rs
│   ├── action_plan.rs
│   ├── state.rs
│   └── utils.rs
├── llm/                 # LLM integration (CREATE)
│   ├── mod.rs
│   ├── intent.rs        # Intent detection
│   ├── recipe.rs        # Recipe generation
│   ├── answer.rs        # Answer generation
│   └── runtime.rs       # Runtime prompt construction
├── telemetry/           # Telemetry collection (CREATE)
│   ├── mod.rs
│   ├── collector.rs     # Telemetry gathering
│   └── parser.rs        # Parse telemetry results
├── personality/         # Personality system (SIMPLIFY)
│   ├── mod.rs
│   ├── traits.rs        # 16 core traits
│   ├── store.rs         # Persistence
│   └── view.rs          # ASCII visualization
├── state/               # State management (CREATE)
│   ├── mod.rs
│   ├── history.rs       # Conversation history
│   └── startup_summary.rs # System change detection
├── recipes.rs           # Recipe library (KEEP)
├── status_command.rs    # Status implementation (ENHANCE)
├── one_shot.rs          # One-shot query handler (CREATE)
├── unified_query.rs     # Shared query processing (ENHANCE)
└── health.rs            # Health checks (KEEP)
```

### 11.2 Files to REMOVE

```
chronos_commands.rs
collective_commands.rs
conscience_commands.rs
consensus_commands.rs
empathy_commands.rs
mirror_commands.rs
autonomy_command.rs
learning_commands.rs
predictive_hints.rs
sentinel_cli.rs
steward_commands.rs
suggest_command.rs
discard_command.rs
adaptive_help.rs (simplify to basic help)
context_detection.rs (merge into telemetry)
monitor_setup.rs
init_command.rs
install_command.rs
upgrade_command.rs
report_command.rs (merge into status)
historian_cli.rs (simplify)
```

**Total removal:** ~15-20 files, ~3000-5000 lines

---

## XII. QA EXPECTATIONS

### 12.1 Test Categories

1. **Unit tests:**
   - Recipe validation
   - Intent detection
   - Telemetry parsing
   - Personality trait calculations

2. **Integration tests:**
   - TUI/one-shot consistency
   - End-to-end query processing
   - Language switching
   - Exit behavior

3. **Snapshot tests:**
   - TUI output formatting
   - Status command output
   - Recipe generation

4. **LLM pipeline tests:**
   - Intent → Recipe → Answer flow
   - Telemetry-first enforcement
   - Error handling

### 12.2 Test Goals

- **Current:** 320 tests passing
- **Goal:** 400+ tests passing
- **Coverage:** >80% for core modules

---

## XIII. RELEASE WORKFLOW

### 13.1 Deliverables

1. **Code:**
   - Remove non-mandated modules
   - Implement new architecture
   - Modularize large files
   - Fix all clippy warnings

2. **Tests:**
   - Write new tests for all modules
   - Ensure TUI/one-shot consistency
   - Snapshot tests for UI

3. **Documentation:**
   - README.md (user-facing)
   - ARCHITECTURE.md (technical)
   - ROADMAP.md (future plans)
   - INTERNAL_PROMPT.md (LLM contract)
   - CHANGELOG.md (version history)

4. **Build:**
   - Compile release binaries
   - Generate checksums
   - Validate auto-updater compatibility

5. **Release:**
   - Commit with clear message
   - Tag: v5.7.0-beta.200
   - Push to GitHub
   - Create GitHub release
   - Test auto-updater

### 13.2 Commit Message Format

```
🏗️ ARCHITECTURE: Complete rewrite for Beta.200

This is a complete architectural reset:

REMOVED (15-20 files, ~4000 lines):
- All personality commands (chronos, collective, conscience, consensus, empathy, mirror)
- All admin commands (init, install, upgrade, suggest, discard, repair)
- All experimental commands (autonomy, learning, predictive, sentinel, steward)

SIMPLIFIED (3 commands only):
- annactl (TUI REPL)
- annactl status (one-shot summary)
- annactl "<question>" (one-shot query)

NEW FEATURES:
- Modern TUI with header, thinking animation, color scheme
- Telemetry-first LLM workflow (zero hallucinations)
- Deterministic startup summary (system changes since last run)
- TUI/one-shot consistency (identical answers)
- Multi-language support (natural switching)
- Natural exit commands (exit, quit, bye, etc.)

MODULARIZATION:
- Created llm/ module (intent, recipe, answer)
- Created telemetry/ module (collector, parser)
- Simplified personality/ module
- Created state/ module (history, startup summary)

TESTS:
- 400+ tests passing (was 320)
- TUI/one-shot consistency tests
- Snapshot tests for UI
- LLM pipeline tests

DOCUMENTATION:
- Updated README.md
- New ARCHITECTURE_BETA_200.md
- Updated ROADMAP.md
- Updated INTERNAL_PROMPT.md

This release makes Anna simple, predictable, and deterministic.

Closes #XXX
```

---

## XIV. RATIONALE

### 14.1 Why This Redesign?

**Problem:** Anna had too many experimental features that confused users and diluted focus.

**Solution:** Remove everything except the core value proposition: telemetry-driven system assistance.

**Benefits:**
1. **Simplicity:** 3 commands vs 20+
2. **Consistency:** TUI and one-shot produce identical results
3. **Reliability:** Telemetry-first = zero hallucinations
4. **Maintainability:** Smaller codebase, clearer architecture
5. **Quality:** Modern TUI matches industry standards

### 14.2 What We're NOT Building

- ❌ Generic chatbot
- ❌ Autonomous agent with complex decision-making
- ❌ Cloud-connected service
- ❌ Multi-agent collective intelligence
- ❌ Philosophical AI experiments

### 14.3 What We ARE Building

- ✅ Local Arch Linux assistant
- ✅ Telemetry-driven answers
- ✅ Transparent command execution
- ✅ Deterministic recipes
- ✅ Simple, predictable UX

---

## XV. MIGRATION GUIDE

### 15.1 For Users

**Old commands that no longer work:**

| Old Command | New Equivalent |
|------------|----------------|
| `annactl suggest` | `annactl "suggest improvements"` |
| `annactl repair` | `annactl "check system health"` |
| `annactl chronos inspect` | Removed (no equivalent) |
| `annactl collective sync` | Removed (no equivalent) |
| `annactl conscience inspect` | Removed (no equivalent) |
| `annactl init` | Installer handles this |
| `annactl upgrade` | Auto-updater handles this |

**What works:**
- `annactl` - Interactive TUI (same as before)
- `annactl status` - System health (enhanced)
- `annactl "question"` - Ask anything (new!)

### 15.2 For Developers

**Breaking changes:**
- Removed 15-20 command modules
- Simplified personality system
- Removed complex historian features
- No more hidden commands (except ping for health)

**New modules:**
- `llm/` - LLM integration
- `telemetry/` - Telemetry collection
- `state/` - State management

**Testing:**
- Must test TUI/one-shot consistency
- Must validate telemetry-first behavior

---

## XVI. APPENDIX: FILE CLEANUP CHECKLIST

### Files to DELETE:

```
✗ adaptive_help.rs
✗ autonomy_command.rs
✗ chronos_commands.rs
✗ collective_commands.rs
✗ conscience_commands.rs
✗ consensus_commands.rs
✗ context_detection.rs
✗ discard_command.rs
✗ empathy_commands.rs
✗ init_command.rs
✗ install_command.rs
✗ learning_commands.rs
✗ mirror_commands.rs
✗ monitor_setup.rs
✗ predictive_hints.rs
✗ report_command.rs
✗ sentinel_cli.rs
✗ steward_commands.rs
✗ suggest_command.rs
✗ upgrade_command.rs
```

### Files to KEEP and ENHANCE:

```
✓ main.rs (minimal entry point)
✓ cli.rs (3 commands only)
✓ runtime.rs (dispatch logic)
✓ tui/ (enhance with new design)
✓ recipes.rs (expand recipe library)
✓ status_command.rs (enhance output)
✓ health.rs (health checks)
✓ unified_query.rs (TUI/one-shot consistency)
✓ errors.rs (error handling)
✓ logging.rs (telemetry logging)
✓ version_banner.rs (startup banner)
```

### Files to CREATE:

```
+ llm/mod.rs
+ llm/intent.rs
+ llm/recipe.rs
+ llm/answer.rs
+ llm/runtime.rs
+ telemetry/mod.rs
+ telemetry/collector.rs
+ telemetry/parser.rs
+ state/mod.rs
+ state/history.rs
+ state/startup_summary.rs
+ one_shot.rs
```

---

## XVII. SUCCESS CRITERIA

Beta.200 is successful when:

1. ✅ ONLY 3 user-facing commands exist
2. ✅ TUI and one-shot produce IDENTICAL answers
3. ✅ Modern TUI matches quality of claude-cli/codex
4. ✅ Zero hallucinations (telemetry-first enforced)
5. ✅ 400+ tests passing
6. ✅ Deterministic startup summary works
7. ✅ Multi-language support works
8. ✅ Natural exit commands work
9. ✅ All documentation updated
10. ✅ Auto-updater compatible
11. ✅ Build succeeds with zero warnings
12. ✅ Codebase reduced by 3000-5000 lines

---

**END OF SPECIFICATION**
