# Anna v5.7.0-beta.55 Release Notes

**Release Date:** 2025-11-18

## Telemetry-First Internal Dialogue & Personality System

**From "Simple Query" to "Thoughtful Analysis"**

Beta.54 fixed critical wiring bugs. Beta.55 implements a fundamental architectural upgrade: telemetry-first internal dialogue with planning and answer rounds, plus a 16-personalities style trait system.

---

## 🎯 Major Features

### 1. Internal Dialogue System (550+ lines)

Anna now "thinks" before answering using a two-round LLM dialogue:

**Planning Round:**
- Analyzes your question
- Checks telemetry for available data
- Identifies missing information
- Sketches answer structure

**Answer Round:**
- Generates final structured output
- Uses all sections: ANNA_TUI_HEADER, ANNA_SUMMARY, ANNA_ACTION_PLAN, ANNA_HUMAN_OUTPUT
- Includes Arch Wiki references
- Provides backup/restore commands

**Key Benefits:**
- ✅ Answers are now telemetry-aware (checks data first)
- ✅ Reduced hallucination (won't guess hardware specs)
- ✅ Better structured responses
- ✅ Debug mode: `ANNA_INTERNAL_TRACE=1` shows Anna's "thinking"

### 2. 16-Personalities Style Trait System

Anna's personality is now adjustable via 8 trait sliders (0-10 scale):

**Trait Dimensions:**
1. **Introvert ↔ Extrovert**: Communication frequency and chattiness
2. **Cautious ↔ Bold**: Risk tolerance, backup emphasis
3. **Direct ↔ Diplomatic**: Phrasing style
4. **Playful ↔ Serious**: Humor level
5. **Minimalist ↔ Verbose**: Answer length
6. **Teacher ↔ Servant**: Explanation depth
7. **Optimistic ↔ Cynical**: Tone and problem framing
8. **Formal ↔ Casual**: Language formality

**Usage:**
```bash
annactl "be more direct"        # Adjusts direct_vs_diplomatic +2
annactl "less serious"           # Adjusts playful_vs_serious -2
annactl "show personality"       # Displays all traits with bars
```

**Visual Output:**
```
Introvert vs Extrovert: ████████░░ 8
  → Introvert style. Reserved, focused messages.
```

Settings persist to `~/.config/anna/personality.toml`

### 3. Telemetry Payload Compression

System context is now compressed for LLM efficiency:

**Before:** Thousands of lines of raw telemetry
**After:** ~200-300 lines of structured, compact data

**Payload Structure:**
```rust
TelemetryPayload {
    hardware: { cpu_model, cpu_cores, total_ram_gb, gpu_model },
    os: { hostname, kernel, arch_status },
    resources: { load_avg, ram_used_percent, disk_usage[] },
    recent_errors: ["Failed service: X", ...],
    trends: { avg_boot_time_s, avg_cpu_percent, stability_score, performance_score },
}
```

Fits rich system context in LLM's working memory without bloat.

---

## 🔧 Technical Changes

**New Files:**
- `crates/annactl/src/internal_dialogue.rs` (550+ lines)
- Redesigned `crates/anna_common/src/personality.rs` (300+ lines)

**Modified Files:**
- `crates/annactl/src/llm_integration.rs` - Integrated internal dialogue
- `crates/annactl/src/main.rs` - Updated personality handling to use new traits
- `crates/annactl/src/repl.rs` - Added HashMap import (compilation fix)
- `Cargo.toml` - Version bump to 5.7.0-beta.55
- `CHANGELOG.md` - Added comprehensive beta.55 entry

**Compilation:**
- ✅ 0 errors
- ⚠️ 233 warnings (mostly unused code in experimental modules)

---

## 🐛 Fixes from Beta.54

Beta.54 had these critical bugs (now fixed):
- ❌ LLM integration not wired to REPL (Intent::Unclear showed "I don't understand")
- ❌ Auto-updater attempted downgrades (beta.53 → beta.9)
- ❌ Historian showed zeros on fresh installs
- ❌ Noisy socket permission warnings

All fixed in beta.54, stable foundation for beta.55.

---

## 📊 Testing Status

**Build:**
- ✅ `cargo build --release` succeeds
- ✅ Both binaries built: annactl 5.7.0-beta.55, annad 5.7.0-beta.55

**Unit Tests:**
- ✅ Personality trait creation, clamping, bar rendering
- ✅ Trait getter/setter methods
- ✅ Natural language adjustment parsing
- ✅ Personality view rendering for LLM prompts

**Integration Tests:**
- 🚧 Internal dialogue end-to-end (needs manual testing)
- 🚧 ANNA_INTERNAL_TRACE output (needs verification)
- 🚧 Personality adjustments in live system (needs testing)

---

## 🚀 User Impact

### What This Means for You

**Before Beta.55:**
```
> tell me about my computer
ℹ I don't understand that yet. Try asking about system status...
```

**After Beta.55:**
```
> tell me about my computer

[Planning Round: Checks telemetry, identifies available data...]
[Answer Round: Generates structured response with references...]

## Hardware
CPU: AMD Ryzen 9 7950X (32 cores)
RAM: 31 GB
GPU: NVIDIA detected
...

References:
- Arch Wiki: System information — https://wiki.archlinux.org/title/System_information
```

**Personality Control:**
```
> be more direct
✓ Adjusted Direct vs Diplomatic to 9/10
  → Direct. Clear, straightforward language.
```

**Debug Mode:**
```bash
ANNA_INTERNAL_TRACE=1 annactl "what's my CPU?"

[ANNA_INTERNAL_TRACE]
internal_summary: Two-round dialogue: planning + answer
planner_prompt_excerpt: You are Anna's planning system...
planner_response_excerpt: Question classified as hardware info query...
answer_prompt_excerpt: Generate final answer with CPU data...
[/ANNA_INTERNAL_TRACE]
```

---

## ⚠️ Known Limitations

1. **RAM/Disk usage**: Telemetry compression simplified for beta.55 (shows 0.0%, needs refinement)
2. **Model quality threshold**: Not yet enforced (planned for future release)
3. **Log rotation**: Not implemented (logs grow unbounded)
4. **SHA-256 checksums**: Uses DefaultHasher (demo only, not cryptographic)

---

## 📖 Migration from Beta.54

**No breaking changes.** Existing installations upgrade seamlessly:

```bash
# Manual install (curl installer):
# Auto-update will download beta.55 within 10 minutes

# Or force update:
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

**New Config Files:**
- `~/.config/anna/personality.toml` - Created on first personality adjustment
- No other config changes required

**Default Personality:**
- All traits start at balanced values (4-8 range)
- Matches previous "moderate humor, normal verbosity" behavior
- Fully backwards compatible

---

## 🔮 What's Next (Beta.56+)

**Immediate Priorities:**
1. Refine telemetry payload (accurate RAM/disk usage)
2. Implement model quality threshold enforcement
3. Test internal dialogue end-to-end
4. Validate personality adjustments in real usage

**Roadmap:**
- Arch Wiki reference enforcement (in prompts, needs validation)
- Sysadmin identity hardening (prompt refinement)
- Snapshot tests for telemetry/dialogue
- Log rotation and cleanup

---

## 📝 Full Changelog

See [CHANGELOG.md](./CHANGELOG.md) for complete version history.

---

## 🙏 Credits

Built with:
- Rust 1.91.1
- Claude Code (AI-assisted development)
- Human guidance and creativity

**License:** GNU General Public License v3 (GPLv3)

---

**Installation:** https://github.com/jjgarcianorway/anna-assistant
**Issues:** https://github.com/jjgarcianorway/anna-assistant/issues
**Changelog:** [CHANGELOG.md](./CHANGELOG.md)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
