# Anna Learning Contract v3.8.0

This document specifies what Anna promises about learning, speed, and correctness.

---

## A) First Time Behavior

For canonical system questions, the first time Anna encounters a question type on a given machine:

### Canonical Question Categories

| Category | Examples | First-Time Path |
|----------|----------|-----------------|
| **CPU** | "What CPU do I have?", "How many cores?" | Brain (<150ms) or Junior+Senior |
| **RAM** | "How much RAM?", "Memory installed?" | Brain (<150ms) or Junior+Senior |
| **Disk** | "Disk space?", "Root filesystem usage?" | Brain (<150ms) or Junior+Senior |
| **Uptime** | "How long has the system been running?" | Brain or Junior+Senior |
| **OS** | "What OS am I running?", "Kernel version?" | Brain or Junior+Senior |
| **GPU** | "What GPU do I have?" | Brain or Junior+Senior |
| **Network** | "What's my IP address?" | Brain or Junior+Senior |
| **Self Health** | "Are you healthy?", "What's your status?" | Brain (<100ms) |
| **Updates** | "Any pending updates?" | Junior+Senior |
| **Logs** | "Show me recent logs" | Junior+Senior |

### First-Time Guarantees

1. **Correctness**: Answer is grounded in probe evidence from the local machine
2. **Honesty**: Reliability score reflects actual confidence; no guessing
3. **Reasonable Speed**:
   - Brain path: <150ms
   - Recipe path: <500ms
   - Junior path: <8s
   - Senior path: <10s
4. **Evidence**: Commands used are visible in debug mode
5. **Learning**: If reliability >= 85%, a recipe is extracted for future use

---

## B) Second and Later Behavior

For the same logical question (even if phrased differently):

### Speed Guarantee

| Question Type | First Time | Second Time | Improvement |
|---------------|------------|-------------|-------------|
| Brain-handled | <150ms | <150ms | Same (already fast) |
| Recipe-learned | 8-10s | <500ms | 16-20x faster |
| LLM-required | 8-10s | 8-10s | No change (needs reasoning) |

### Learning Mechanics

1. **Recipe Extraction**: When Junior/Senior produces a high-reliability answer (>=85%):
   - Question type and key tokens are captured
   - Probes used are recorded
   - Answer template is created
   - Stored in `/var/lib/anna/recipes/store.json`

2. **Recipe Matching**: On subsequent questions:
   - Key tokens are compared (>=50% match required)
   - Question type must match exactly
   - If confidence >=70%, recipe is used
   - Probes are executed, template is applied
   - Origin: "Recipe" (0 LLM calls)

3. **Brain Patterns**: For hardware questions:
   - Patterns are cached with 60-second TTL
   - Commands are reused without re-planning
   - Origin: "Brain" (0 LLM calls)

### What Learning Is NOT

- **Not guessing**: Brain only answers when it has solid evidence
- **Not inventing**: Recipes come from successful, verified answers
- **Not global**: Learning is per-machine, not shared
- **Not permanent**: Recipes can be cleared with hard reset

### Canonical Flows That Must Learn

| Flow | First Time | After Learning |
|------|------------|----------------|
| Hardware queries (CPU, RAM, disk) | Brain fast path | Brain (same) |
| Self diagnostics | Brain fast path | Brain (same) |
| Anna status | Brain fast path | Brain (same) |
| Filesystem usage | Brain or Recipe | Recipe (<500ms) |
| Update checks | Junior+Senior | Recipe (<500ms) |
| Log queries | Junior+Senior | Recipe (<500ms) |

---

## C) Reset Behavior

### Soft Reset (Experience Reset)

**Trigger**: "soft reset", "reset my stats", "clear my progress"

**What is cleared**:
- XP and level (returns to baseline: Level 1, 0 XP)
- Telemetry counters
- Stats history

**What is preserved**:
- Brain recipes and learned patterns
- Knowledge about the local system
- Configuration settings

**Result**: "You forget your streak and score, but keep your skills."

### Hard Reset (Factory Reset)

**Trigger**: "hard reset", "factory reset", "delete everything"

**What is cleared**:
- XP, stats, telemetry (same as soft reset)
- Brain recipes and patterns
- User-specific knowledge
- Learned preferences

**What is preserved**:
- System binaries (annad, annactl)
- Default configuration
- Autoprovision capability

**Result**: "You are a fresh intern again, but still functional."

### Post-Reset Validation

After hard reset, First Light self-test runs to verify:
1. Brain can answer CPU, RAM, disk questions
2. Junior and Senior LLMs are responsive
3. XP system is tracking correctly
4. Reliability scoring is accurate

---

## D) Debug Mode and Honesty

### Debug Mode Behavior

When debug mode is enabled ("enable debug mode"):

**Visible information**:
1. **Start Banner**: Question received, timestamp
2. **Router Decision**: Which path was chosen (Brain/Recipe/Junior/Senior)
3. **Brain/LLM Steps**:
   - Brain: pattern matched, probe executed
   - Junior: plan proposed, draft created
   - Senior: audit performed, verdict given
4. **Probes Executed**: Commands run with timing
5. **Final Summary**: Reliability, origin, latency

**Guarantees**:
- No hidden steps
- All probes are shown
- Timing is accurate
- Origin matches actual path taken

### Normal Mode Behavior

When debug mode is disabled:

**Visible information**:
1. Answer text
2. Reliability percentage (not raw float)
3. Origin indicator (Brain/Recipe/Junior/Senior)
4. Brief timing summary

**Guarantees**:
- No internal error messages leak
- Compact, user-friendly output
- Same answer as debug mode (just less verbose)

---

## E) Reliability and Honesty Guarantees

### Reliability Scoring

| Score | Label | Meaning |
|-------|-------|---------|
| >= 90% | GREEN | High confidence, probe-backed |
| >= 70% | YELLOW | Medium confidence, some uncertainty |
| >= 50% | ORANGE | Low confidence, partial evidence |
| < 50% | RED | Very low confidence, may be degraded |

### Honesty Rules

1. **No fake Green**:
   - Only answers with real probe evidence can be Green
   - If a fallback is used, reliability reflects that

2. **No silent failures**:
   - If a probe fails, the answer acknowledges it
   - If LLM times out, a degraded answer is given with low reliability

3. **No hallucinations**:
   - If Anna cannot measure something, she says so
   - Unknown questions get honest "I don't know" responses

4. **Percentage format**:
   - All reliability in user-facing output is in percentage form (e.g., "92%")
   - No raw floats (e.g., "0.92") in status, answers, or benchmarks

---

## F) Telemetry Consistency

### Origin Tracking

Every answer has an origin field:
- `Brain`: Fast path, 0 LLM calls
- `Recipe`: Learned pattern, 0 LLM calls
- `Junior`: LLM planning + draft, 2 LLM calls
- `Senior`: Full audit, 3 LLM calls

### Telemetry Must Reflect Learning

After repeated canonical questions:
- Brain/Recipe count should increase
- Junior/Senior count should stay stable or decrease
- Average latency for Brain/Recipe should be lower than LLM paths
- Reliability should remain high (>=90% for canonical domains)

### Stats Consistency

- `annactl status` stats must match telemetry file
- Origin distribution must match actual paths taken
- No silent counter resets or gaps

---

## G) Implementation Invariants

These are the hard rules enforced by code:

| Invariant | Value | Enforced By |
|-----------|-------|-------------|
| MIN_RECIPE_RELIABILITY | 0.85 | recipe.rs |
| RECIPE_MATCH_THRESHOLD | 0.70 | recipe.rs |
| BRAIN_TIME_BUDGET | 150ms | brain_fast.rs |
| RECIPE_TIME_BUDGET | 500ms | perf_timing.rs |
| JUNIOR_HARD_TIMEOUT | 6s | perf_timing.rs |
| SENIOR_HARD_TIMEOUT | 8s | perf_timing.rs |
| GLOBAL_BUDGET | 15s | perf_timing.rs |
| MAX_RECIPES_PER_TYPE | 20 | recipe.rs |
| FAILURE_MEMORY_TTL | 24h | brain.rs |

---

## H) Testing Contract

The following behaviors are verified by automated tests:

### Learning Speed Tests (learning_speed_canonical.rs)

1. **Round 1**: First-time canonical questions record origin and latency
2. **Round 2**: Same questions show more Brain usage, equal or better latency
3. **Round 3**: Paraphrased variants map to same intents, use Brain/Recipe

### Telemetry Tests

1. Brain count increases after learning
2. Average Brain latency < average LLM latency
3. Reliability stays >=90% for canonical questions

### Reset Tests

1. Soft reset preserves Brain recipes
2. Hard reset clears Brain recipes
3. Both leave system in healthy state

### Debug Mode Tests

1. Debug output has correct structure
2. All expected elements appear in order
3. Normal mode shows only compact answer

---

## Version History

| Version | Changes |
|---------|---------|
| v3.8.0 | Initial contract definition |
