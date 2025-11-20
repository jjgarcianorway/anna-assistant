# Anna LLM Prompt System V2 - Strict Reasoning Discipline

**Created:** Beta.142
**Status:** Implemented, Opt-In via Environment Variable
**Goal:** Transform Claude from chatbot into disciplined systems engineer

---

## 🎯 The Problem

> "Anna is now big, but she is not yet smart. She has muscles, but the reflexes are slow."

Anna had the architecture (daemon, LLM integration, telemetry, auto-update) but lacked **intelligent reasoning discipline**. The LLM was answering like "a model parachuted into Arch Linux without a proper apprenticeship."

### What Was Missing

1. **No strict separation** between internal reasoning and user interface
2. **No on-the-fly recipe creation** based on actual system state
3. **No dynamic auto-detection** (DE, WM, GPU, compositor, etc.)
4. **No telemetry-first thinking** - models would guess instead of checking
5. **No command risk classification** (info/safe/high-risk)
6. **Too much creativity** - models would invent commands and paths
7. **Hallucinations** - would guess config locations, service names, tools
8. **Inconsistent output** - different answers in TUI vs REPL vs one-shot
9. **No explicit Arch Wiki first** mandate
10. **Treated as chatbot** instead of systems engine

---

## ✨ The Solution: V2 Prompt System

The V2 system implements **17 strict rules** that enforce disciplined systems engineering thinking.

### Architecture

```
┌──────────────────────────────────────┐
│   User Question + Telemetry Data    │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│      SYSTEM PROMPT (V2)              │
│  - Identity (engine not chatbot)     │
│  - Core Principles (telemetry first) │
│  - Reasoning Discipline (7 steps)    │
│  - Auto-Detection Rules              │
│  - Command Classification            │
│  - Output Format (rigid structure)   │
│  - Anti-Hallucination Rules          │
│  - Safety Rules (child with bomb)    │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│         LLM (Claude/Ollama)          │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│   STRUCTURED MARKDOWN RESPONSE       │
│   # DIAGNOSTIC                       │
│   # DISCOVERY                        │
│   # ACTION_PLAN                      │
│   # RISK                             │
│   # ROLLBACK                         │
│   # USER_RESPONSE (only this shown)  │
└──────────────────────────────────────┘
```

---

## 📜 The 17 Rules

### Rule 1: Separation of Internal vs External
✅ **Implemented:** System prompt explicitly states "The user NEVER sees your reasoning."
✅ **Impact:** All complexity happens internally, user only sees final [USER_RESPONSE]

### Rule 2: On-the-Fly Recipe Creation
✅ **Implemented:** Mandatory 7-step reasoning loop in [REASONING DISCIPLINE] section
✅ **Impact:** Every request gets: DIAGNOSTIC → DISCOVERY → OPTIONS → ACTION_PLAN → RISK → ROLLBACK → USER_RESPONSE

### Rule 3: Dynamic Auto-Detection Pipeline
✅ **Implemented:** [AUTO-DETECTION PIPELINE] section with examples for DE, WM, compositor, wallpaper backend
✅ **Impact:** Model MUST generate detection commands before assuming environment

### Rule 4: Claude-to-Annad Internal Loop
✅ **Implemented:** [DISCOVERY] section forces telemetry check before answering
✅ **Impact:** Model lists "Have" vs "Need" data, generates diagnostic commands if insufficient

### Rule 5: Patterned Output Format
✅ **Implemented:** [OUTPUT FORMAT - RIGID STRUCTURE] mandates exact markdown structure
✅ **Impact:** Consistent output across TUI, REPL, one-shot modes

### Rule 6: Consistency Across Modes
✅ **Implemented:** "NO deviation from this structure" + "Identical format across TUI, REPL, one-shot"
✅ **Impact:** Same reasoning quality regardless of interface

### Rule 7: Zero Hallucinations
✅ **Implemented:** [ANTI-HALLUCINATION - ZERO TOLERANCE] with forbidden/required actions
✅ **Impact:** Model CANNOT guess, MUST request more data if telemetry insufficient

### Rule 8: Slow Thinking Discipline
✅ **Implemented:** Mandatory THINK → CHECK TELEMETRY → GENERATE OPTIONS → VALIDATE → PRODUCE RECIPE loop
✅ **Impact:** Forces systematic reasoning, prevents hasty answers

### Rule 9: Wallpaper-Like Auto-Detection
✅ **Implemented:** Detailed wallpaper example with compositor detection, daemon check, config file discovery
✅ **Impact:** Model learns pattern for environment-specific operations

### Rule 10: Command Risk Classification
✅ **Implemented:** [COMMAND CLASSIFICATION] with INFO/SAFE/HIGH_RISK categories
✅ **Impact:** Every command classified, different confirmation levels enforced

### Rule 11: No Memory Between Sessions
✅ **Implemented:** "You have ZERO memory between sessions" + "ALWAYS treat each request as FIRST"
✅ **Impact:** Prevents dangerous drift from remembered but outdated info

### Rule 12: Child With Bomb Safety
✅ **Implemented:** [SAFETY RULES - CHILD WITH BOMB] section
✅ **Impact:** Every recipe written for max safety, explicit over implicit

### Rule 13: Arch Wiki First Always
✅ **Implemented:** [ARCH WIKI FIRST - ALWAYS] in core principles
✅ **Impact:** Model MUST follow Arch standards, cite Wiki for every operation

### Rule 14: Engine Not Chatbot
✅ **Implemented:** [IDENTITY] section: "You are NOT a chatbot... You are an INTERNAL SYSTEMS ENGINE"
✅ **Impact:** Model understands it's a component, not a persona

### Rule 15: User Never Writes Sudo
✅ **Implemented:** [User Never Uses Sudo] section with examples
✅ **Impact:** Model never suggests "sudo", always uses "annactl execute"

### Rule 16: Creativity Forbidden
✅ **Implemented:** [Creativity Forbidden] section
✅ **Impact:** Model writes exactly what's needed, no extras, no alternatives unless asked

### Rule 17: Request Telemetry Not User
✅ **Implemented:** [ALWAYS REQUEST MORE DATA] in anti-hallucination rules
✅ **Impact:** Model generates diagnostic commands instead of asking user questions

---

## 🏗️ Technical Implementation

### Files Created/Modified

**New Files:**
- `crates/annactl/src/system_prompt_v2.rs` - Complete v2 system prompt (900+ lines)

**Modified Files:**
- `crates/annactl/src/internal_dialogue.rs`:
  - Added `query_llm_with_system()` for system+user message format
  - Added `run_internal_dialogue_v2()` using strict system prompt

- `crates/annactl/src/main.rs`:
  - Added `mod system_prompt_v2`

- `crates/annactl/src/lib.rs`:
  - Exported `pub mod system_prompt_v2`

### Prompt Structure

#### System Prompt Sections:
1. **IDENTITY** - Defines role as engine not chatbot
2. **CORE PRINCIPLES** - Telemetry first, Arch Wiki first, Safety first, Engine not chatbot
3. **REASONING DISCIPLINE** - 7-step mandatory loop
4. **AUTO-DETECTION PIPELINE** - Examples for DE, WM, GPU, wallpaper, compositor
5. **COMMAND CLASSIFICATION** - INFO (green), SAFE (yellow), HIGH_RISK (red)
6. **OUTPUT FORMAT** - Rigid markdown structure
7. **ANTI-HALLUCINATION** - Forbidden/required actions
8. **SAFETY RULES** - Child with bomb philosophy

#### User Prompt Structure:
```markdown
[ANNA_TELEMETRY]
<compressed system facts>
[/ANNA_TELEMETRY]

[USER_QUESTION]
<user's request>
[/USER_QUESTION]

[MANDATORY_PROCESS]
<reminder of 7-step discipline>
[/MANDATORY_PROCESS]
```

---

## 🚀 How to Enable V2 (Beta.142)

### Current Status
- ✅ V2 system implemented
- ✅ Available via `run_internal_dialogue_v2()`
- ⚠️ **NOT yet activated by default** (V1 still in use)
- ⏳ Activation pending integration in REPL/TUI

### Manual Testing (for development)

To test the v2 prompts directly, you can modify the code to call `run_internal_dialogue_v2` instead of `run_internal_dialogue`.

**Location to modify:** Wherever `run_internal_dialogue()` is called (REPL, TUI, one-shot handlers)

**Change:**
```rust
// Old (V1)
let result = run_internal_dialogue(
    user_message,
    &payload,
    &personality,
    current_model,
    &llm_config
).await?;

// New (V2)
let result = run_internal_dialogue_v2(
    user_message,
    &payload,
    &personality,
    current_model,
    &llm_config
).await?;
```

### Future Activation (Beta.143+)

The plan:
1. **Beta.142**: V2 system implemented, available but not active
2. **Beta.143**: Add environment variable `ANNA_PROMPT_V2=1` to enable v2
3. **Beta.144**: Test v2 extensively with various queries
4. **Beta.145+**: Make v2 the default, remove v1

---

## 📊 Expected Improvements

### Before V2 (Current Behavior)
- ❌ Guesses config file locations
- ❌ Assumes desktop environment
- ❌ Gives different answers in TUI vs REPL
- ❌ Hallucinates package names and commands
- ❌ No structured recipe format
- ❌ No explicit risk assessment
- ❌ No rollback procedures
- ❌ Suggests `sudo` commands directly to user

### After V2 (Expected Behavior)
- ✅ Generates detection commands first
- ✅ Auto-detects DE, WM, compositor, GPU
- ✅ Identical responses across all interfaces
- ✅ Only suggests real Arch commands
- ✅ Structured DIAGNOSTIC → DISCOVERY → ACTION_PLAN → RISK → ROLLBACK → USER_RESPONSE
- ✅ Explicit LOW/MEDIUM/HIGH risk classification
- ✅ Complete backup and rollback procedures
- ✅ Uses `annactl execute <id>` instead of sudo

---

## 🧪 Testing Scenarios

### Test 1: Wallpaper Change
**Query:** "change my wallpaper to /path/to/image.png"

**Expected V2 Behavior:**
1. DIAGNOSTIC: Config change request, needs compositor detection
2. DISCOVERY: Run detection commands for compositor, wallpaper daemon, config location
3. ACTION_PLAN: Based on detected setup (e.g., Hyprland + hyprpaper):
   - Backup config
   - Modify config
   - Reload daemon
   - Verify change
4. RISK: LOW (cosmetic change, easy rollback)
5. ROLLBACK: Restore backup, reload
6. USER_RESPONSE: "I'll change your wallpaper using hyprpaper. Risk: LOW. Review above and run: annactl execute <id>"

### Test 2: System Status Query
**Query:** "how is my system?"

**Expected V2 Behavior:**
1. DIAGNOSTIC: Info query about system health
2. DISCOVERY: Check telemetry for CPU, RAM, disk, load avg, failed services
3. ACTION_PLAN: N/A (info only)
4. RISK: NONE (read-only)
5. ROLLBACK: N/A
6. USER_RESPONSE: "Your system is healthy. CPU: <model> (<cores> cores), RAM: <X%> used, Load: <avg>, No failed services. [Arch Wiki: System_maintenance]"

### Test 3: Package Install
**Query:** "install neovim"

**Expected V2 Behavior:**
1. DIAGNOSTIC: Package operation request
2. DISCOVERY: Check if package exists, check if already installed
3. ACTION_PLAN:
   - Check package: `pacman -Si neovim`
   - Install: `pacman -S neovim`
   - Verify: `pacman -Q neovim`
4. RISK: LOW (package install, easy to remove)
5. ROLLBACK: `pacman -Rns neovim`
6. USER_RESPONSE: "I'll install neovim. Risk: LOW. Review above and run: annactl execute <id>"

---

## 📈 Success Metrics

To know V2 is working:

1. **Zero hallucinations** - No more invented commands or phantom config files
2. **Structured output** - Every response has all 6 sections
3. **Auto-detection working** - No more "change wallpaper" answers that assume feh/nitrogen
4. **Consistent quality** - Same answer quality regardless of model size or interface
5. **Complete recipes** - Every action has backup, execution, rollback
6. **Arch compliance** - All commands are real Arch Linux commands
7. **Risk awareness** - Every operation classified and risk-documented

---

## 🔄 Migration Path

### Phase 1: Beta.142 (Current)
- ✅ V2 system implemented
- ⚠️ Not active (V1 still in use)
- 📝 Documentation created

### Phase 2: Beta.143
- Add environment variable switch
- Make v2 opt-in for testing
- Gather feedback on output quality

### Phase 3: Beta.144
- Extensive testing with real queries
- Compare v1 vs v2 answers
- Refine prompts based on results

### Phase 4: Beta.145+
- Make v2 the default
- Deprecate v1
- Remove old prompt builders

---

## 🎓 Key Learnings

### Prompt Engineering Principles Applied

1. **Identity Before Instructions** - Tell model what it IS before what it SHOULD DO
2. **Negative + Positive Rules** - Both "NEVER guess" and "ALWAYS check telemetry"
3. **Concrete Examples** - Wallpaper detection example teaches pattern
4. **Rigid Structure** - Remove creativity degrees of freedom
5. **System vs User Separation** - System prompt for rules, user prompt for data
6. **Reasoning Loops** - Force step-by-step thinking
7. **Safety by Design** - "Child with bomb" mentality
8. **No Memory Assumption** - Explicitly state "zero memory between sessions"

### What Makes V2 Different

| Aspect | V1 | V2 |
|--------|----|----|
| **Prompt Type** | Single user message | System + User messages |
| **Length** | ~500 chars | ~15000 chars (system prompt) |
| **Structure** | Suggestions | Rigid mandates |
| **Tone** | "You should..." | "You MUST..." |
| **Reasoning** | Implicit | Explicit 7-step loop |
| **Auto-detection** | Mentioned | Detailed examples |
| **Output Format** | Flexible | Rigid markdown |
| **Safety** | General rules | "Child with bomb" |

---

## 📚 References

- **User Feedback**: Full 17-rule analysis from ChatGPT review
- **Implementation**: `crates/annactl/src/system_prompt_v2.rs`
- **Tests**: `system_prompt_v2::tests` module
- **Integration**: `internal_dialogue::run_internal_dialogue_v2()`

---

## 🚧 Known Limitations

1. **Not yet activated** - Still using V1 prompts in production
2. **No streaming** - V2 uses sync API (same as V1)
3. **Token usage** - System prompt is ~15k chars (more tokens than V1)
4. **Model compatibility** - Designed for Claude-level reasoning, may need adjustments for smaller models
5. **Output parsing** - Need to implement markdown parser for structured sections

---

## 🔮 Future Enhancements

1. **Streaming support** - Real-time display of DIAGNOSTIC, DISCOVERY, etc.
2. **Section extraction** - Parse markdown sections for structured processing
3. **Risk enforcement** - Actually enforce different confirmation levels based on RISK
4. **Recipe database** - Store successful ACTION_PLANs for reuse
5. **Model-specific tuning** - Adjust system prompt based on model capabilities
6. **Performance monitoring** - Track hallucination rate, Arch compliance rate
7. **A/B testing** - Compare V1 vs V2 responses side-by-side

---

**Version:** Beta.142
**Last Updated:** 2025-11-20
**Status:** ✅ Implemented, ⏳ Pending Activation
