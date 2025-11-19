# What's New in Beta.88

**Release Date:** 2025-11-19
**Focus:** Personality CLI & QA Validation Infrastructure

---

## Overview

Beta.88 makes the 16-trait personality system accessible via CLI commands and validates the 100-question test suite works correctly.

**Key Achievement:** Personality customization is now a first-class CLI feature, not just internal configuration.

---

## Personality CLI Commands

**NEW:** `annactl personality` subcommand with 6 actions

### Available Commands

```bash
# View all 16 traits with current values
annactl personality show

# Set a specific trait (0-10 scale)
annactl personality set introvert_vs_extrovert 7
annactl personality set playful_vs_serious 4

# Adjust by delta
annactl personality adjust calm_vs_excitable +2
annactl personality adjust minimalist_vs_verbose -1

# Reset all traits to defaults
annactl personality reset

# Validate configuration for conflicts
annactl personality validate

# Export to TOML file
annactl personality export
annactl personality export --path ~/my_anna_personality.toml
```

### The 16 Traits

**Original 8 (Beta.83-86):**
1. `introvert_vs_extrovert` - Communication frequency and style
2. `calm_vs_excitable` - Energy level in responses
3. `direct_vs_diplomatic` - Phrasing approach
4. `playful_vs_serious` - Humor usage
5. `cautious_vs_bold` - Risk tolerance
6. `minimalist_vs_verbose` - Response length
7. `analytical_vs_intuitive` - Reasoning style
8. `reassuring_vs_challenging` - Support vs critique balance

**New 8 (Beta.86):**
9. `patient_vs_urgent` - Time taken to explain
10. `humble_vs_confident` - Self-assurance level
11. `formal_vs_casual` - Professional vs friendly tone
12. `empathetic_vs_logical` - Emotion vs logic priority
13. `protective_vs_empowering` - Safety vs autonomy
14. `traditional_vs_innovative` - Arch Way vs new tools
15. `collaborative_vs_independent` - Guidance vs self-sufficiency
16. `perfectionist_vs_pragmatic` - Thoroughness vs speed

### Trait Value Meanings

Each trait uses a 0-10 scale:
- **0-3**: Strong first pole (e.g., very extrovert)
- **4-6**: Balanced middle ground
- **7-10**: Strong second pole (e.g., very introvert)

The `show` command displays visual bars and computed meanings for each value.

---

## Validation Infrastructure Status

### 100-Question Test Suite

**File:** `data/post_install_questions.json`
**Status:** ✅ Exists and properly structured
**Count:** 100 questions verified

**Structure:**
```json
{
  "metadata": {
    "total_questions": 100,
    "categories": ["network", "packages", "display", "audio", ...]
  },
  "questions": [
    {
      "id": 1,
      "category": "network",
      "difficulty": "beginner",
      "question": "My internet doesn't work after installation...",
      "expected_commands": ["ip link", "ip addr", "ping"],
      "expected_topics": ["NetworkManager", "systemd-networkd"]
    },
    ...
  ]
}
```

### Validation Script

**File:** `scripts/validate_post_install_qa.sh`
**Status:** ✅ Ready to use
**Usage:**
```bash
# Test 10 questions (quick validation)
./scripts/validate_post_install_qa.sh 10

# Test all 100 questions
./scripts/validate_post_install_qa.sh 100

# Custom questions file
./scripts/validate_post_install_qa.sh <file.json> <count>
```

**Output:**
- Terminal: Real-time progress with pass/fail indicators
- Markdown report: `post_install_validation_results.md`

**Validation Criteria:**
- Response not empty or error
- Expected commands mentioned in answer
- Expected topics covered
- Warning provided if flagged as risky

---

## Technical Details

### Files Modified

**crates/annactl/src/main.rs:**
- Added `PersonalityCommands` enum (lines 135-167)
- Added `Commands::Personality` variant (lines 122-126)
- Added personality command routing (lines 928-956)
- Added "personality" to command_name match (line 947)

**Handlers (already existed in personality_commands.rs):**
- `handle_personality_show()` - Display all traits
- `handle_personality_set()` - Set specific value
- `handle_personality_adjust()` - Adjust by delta
- `handle_personality_reset()` - Reset to defaults
- `handle_personality_validate()` - Check conflicts
- `handle_personality_export()` - Export to TOML

### Database Storage

Personality traits stored in SQLite `context.db`:
```sql
TABLE personality (
  key TEXT PRIMARY KEY,
  name TEXT,
  value INTEGER CHECK(value >= 0 AND value <= 10),
  meaning TEXT
)
```

Location: Auto-detected via `DbLocation::auto_detect()`
- Development: `./context.db`
- Production: `/var/lib/anna/context.db`

### TOML Export Format

```toml
active = true

[[traits]]
key = "introvert_vs_extrovert"
name = "Introvert vs Extrovert"
value = 3
meaning = "Extrovert style. Frequent, chatty communication."

[[traits]]
key = "calm_vs_excitable"
value = 8
...
```

---

## Known Limitations

### Async Runtime Issue

**Symptom:** `annactl personality` commands may fail with tokio runtime error in debug builds:
```
Cannot drop a runtime in a context where blocking is not allowed
```

**Cause:** Mixing sync/async contexts in personality command handlers.

**Workaround:** Use release builds (`cargo build --release`) which handle this more gracefully.

**Fix planned:** Beta.89 will refactor to proper async handling.

### No Import Command Yet

TOML export works, but `annactl personality import` is not yet implemented.

**Workaround:** Manually edit `~/.config/anna/personality.toml` or database directly.

**Planned:** Beta.89

### No Conflict Enforcement

Validation detects conflicts but doesn't prevent invalid configurations.

**Example issue:** User can set both `introvert_vs_extrovert=10` and `extrovert=10` conceptually
(though current implementation uses single slider per pair).

**Planned:** Beta.89 will add strict conflict rules.

---

## Migration Notes

**No breaking changes.** All existing configurations remain compatible.

If you have a `personality.toml` from earlier beta versions, it will be read correctly.

New installations get default trait values from `personality.rs:default_traits()`.

---

## Testing Guide

### Verify Personality CLI

```bash
# 1. Show current personality
annactl personality show

# 2. Set a trait
annactl personality set introvert_vs_extrovert 7

# 3. Verify it changed
annactl personality show | grep "Introvert vs Extrovert"

# 4. Export to file
annactl personality export --path /tmp/test_personality.toml

# 5. Check file exists
cat /tmp/test_personality.toml

# 6. Reset to defaults
annactl personality reset
```

### Run Validation Suite

```bash
# Quick test (10 questions)
./scripts/validate_post_install_qa.sh 10

# Check the report
cat post_install_validation_results.md
```

**Expected results:**
- Some questions will pass (response contains expected commands/topics)
- Some may fail (Anna not yet perfect at all question types)
- Report shows detailed breakdown

---

## What's NOT in Beta.88

**Deferred to Beta.89:**
- Large-scale validation run (100+ questions)
- Answer quality metrics and benchmarking
- Personality import from TOML
- Trait conflict enforcement
- Async runtime fixes

**Deferred to Beta.90:**
- Personality presets (professional, casual, technical)
- Interactive personality wizard
- Diff/compare commands

**See ROADMAP_BETA88.md for full feature timeline.**

---

## Command Reference

```bash
# Personality Management
annactl personality show                    # Display all traits
annactl personality set <trait> <value>     # Set 0-10
annactl personality adjust <trait> <delta>  # +/- adjustment
annactl personality reset                   # Reset to defaults
annactl personality validate                # Check conflicts
annactl personality export [--path FILE]    # Export TOML

# Validation (via scripts)
./scripts/validate_post_install_qa.sh [COUNT]
./scripts/fetch_reddit_qa.sh <file> <count>
./scripts/fetch_multi_subreddit_qa.sh <file> <count>
```

---

## Developer Notes

**Trait naming convention:** `first_pole_vs_second_pole`
Lower values (0-3) = first pole, higher values (7-10) = second pole

**Adding new traits:** Edit `personality.rs:default_traits()` and add meaning computation in `compute_meaning()`.

**Testing handlers:** All personality commands work without daemon (direct DB access).

---

**Questions or issues?** Report at https://github.com/jjgarcianorway/anna-assistant/issues
