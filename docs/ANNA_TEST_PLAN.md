# Anna Test Plan v3.7.0 "Reliability Gauntlet"

This document describes the "Day in the Life" acceptance test scenario that validates
Anna's end-to-end behavior from fresh install through production use.

## Purpose

This test plan codifies exactly what Anna must do correctly. It is not aspirational -
every scenario described here is implemented as automated tests in the codebase.

## Test Categories

| Category | Purpose | Test Module |
|----------|---------|-------------|
| Fresh Install | Validate clean state | `system_acceptance.rs` |
| Autoprovision | LLM setup works | `system_acceptance.rs` |
| First Light | Self-test passes | `system_acceptance.rs` |
| Canonical Questions | Core functionality | `system_acceptance.rs` |
| Learning | Brain improves latency | `system_acceptance.rs` |
| Natural Language | Intent normalization | `system_acceptance.rs` |
| Benchmarks | Snow Leopard consistency | `system_acceptance.rs` |
| Resets | Soft/hard reset behavior | `system_acceptance.rs` |
| Debug Mode | Fly-on-the-wall output | `system_acceptance.rs` |
| Stats & Percentages | Correct display format | `system_acceptance.rs` |

---

## 1. Fresh Install Validation

### Scenario
A new Anna installation with no prior state.

### Expected State
- **XP Store**: Does not exist OR contains baseline values (level 1, trust 0.5, xp 0)
- **Telemetry**: Empty or does not exist
- **Knowledge**: Empty or does not exist
- **Recipe Cache**: Empty
- **Status Output**:
  - Shows "Level 1" or equivalent fresh state
  - Shows "Trust: 50%" (baseline)
  - Shows "No telemetry data" or equivalent
  - Autoprovision status shows either "pending" or "ready"

### Assertions
```
ASSERT xp_store.anna.level == 1
ASSERT xp_store.anna.xp == 0
ASSERT xp_store.anna.trust == 0.5
ASSERT telemetry.line_count == 0
ASSERT status.contains("Level 1") OR status.contains("Intern")
```

### Test Function
`test_fresh_install_state`

---

## 2. Autoprovision Flow

### Scenario
Anna is asked to set up models for daily use when no LLMs are configured.

### Trigger
User asks: "Set up your models and be ready for daily use."

### Expected Behavior
1. Anna detects no Ollama installed (or no models pulled)
2. Anna explains what will happen
3. Anna installs Ollama (if needed)
4. Anna pulls required models (Junior: small, Senior: larger)
5. Anna confirms completion with specific model names

### Expected Output
- "Installing Ollama..." (if not present)
- "Pulling model: qwen2.5:7b" (or similar)
- "Junior model: qwen2.5:7b"
- "Senior model: qwen2.5:14b" (or detected model)
- "Ready for daily use"

### Assertions
```
ASSERT config.junior_model IS SET
ASSERT config.senior_model IS SET
ASSERT ollama.is_available == true
ASSERT response.contains("Ready") OR response.contains("available")
```

### Test Function
`test_autoprovision_flow`

---

## 3. First Light Self-Test

### Scenario
Anna runs her internal self-test to validate all systems are working.

### Trigger
User asks: "Run your first light self test and summarize the result."

### Expected Behavior
1. Anna runs a series of internal checks:
   - LLM connectivity
   - Probe execution
   - Knowledge store access
   - XP system
2. Anna reports results with reliability scores

### Expected Output
- "Running First Light self-test..."
- "LLM connectivity: OK"
- "Probe execution: OK"
- "All systems operational" (or specific issues if any)
- "Overall reliability: XX%"

### Assertions
```
ASSERT response.contains("OK") OR response.contains("operational")
ASSERT reliability_percentage >= 70%
ASSERT no_errors_in_execution
```

### Test Function
`test_first_light_self_test`

---

## 4. Canonical Questions (First Pass)

### Scenario
Ask Anna 10 fundamental system questions that any sysadmin assistant must answer.

### Questions
1. "What is my CPU model and how many cores does it have?"
2. "How much RAM is installed and how much is available?"
3. "What is my root filesystem usage?"
4. "What is the system uptime?"
5. "What OS and kernel version am I running?"
6. "What is your self health status?"
7. "What GPU do I have?" (may report "none detected")
8. "What are my network interfaces and IP addresses?"
9. "Are there any pending system updates?"
10. "Show me recent system logs from the last hour."

### Expected Behavior
- Each question gets a meaningful answer
- Reliability is shown as percentage (e.g., "Reliability: 87%")
- Origin is tracked (Brain, Junior, or Senior)
- Latency is recorded

### Assertions (Per Question)
```
ASSERT answer IS NOT EMPTY
ASSERT answer.reliability IS PERCENTAGE (contains "%")
ASSERT answer.origin IN ["Brain", "Junior", "Senior"]
ASSERT latency_ms > 0
ASSERT telemetry.event_recorded == true
```

### Test Function
`test_canonical_questions_first_pass`

---

## 5. Learning Check (Second Pass)

### Scenario
Repeat the same 10 canonical questions to verify learning.

### Expected Behavior
- More questions answered by Brain (cached/learned)
- Lower average latency
- Same or higher reliability

### Assertions
```
ASSERT second_pass.brain_count >= first_pass.brain_count
ASSERT second_pass.avg_latency_ms <= first_pass.avg_latency_ms * 1.5
ASSERT second_pass.avg_reliability >= first_pass.avg_reliability * 0.95
```

### Metrics to Track
| Metric | First Pass | Second Pass | Expected |
|--------|------------|-------------|----------|
| Brain answers | X | Y | Y >= X |
| Avg latency | A ms | B ms | B <= A * 1.5 |
| Avg reliability | R1% | R2% | R2 >= R1 * 0.95 |

### Test Function
`test_learning_improves_performance`

---

## 6. Natural Language and Typos

### Scenario
Ask paraphrased and slightly broken questions to test intent normalization.

### Questions
1. "how much mem do I got?" -> RAM question
2. "what is ur cpu again?" -> CPU question
3. "what is the current uptime on this machine?" -> Uptime question
4. "show me disk space" -> Filesystem question
5. "whats my ip" -> Network question

### Expected Behavior
- Questions map to canonical intents
- Answers are semantically correct
- No crashes or "I don't understand" for reasonable typos

### Assertions
```
ASSERT "mem" question -> answer contains RAM/memory info
ASSERT "cpu" question -> answer contains processor info
ASSERT "uptime" question -> answer contains time/duration
ASSERT "disk" question -> answer contains filesystem/storage info
ASSERT "ip" question -> answer contains network/address info
```

### Test Function
`test_natural_language_intent_mapping`

---

## 7. Snow Leopard Benchmark

### Scenario
Run both full and quick benchmarks to validate consistency.

### Triggers
1. "Run the full snow leopard benchmark and show me the summary."
2. "Run a quick benchmark and summarize the key differences."

### Expected Behavior (Full Benchmark)
- Runs all benchmark phases:
  - Canonical questions
  - Paraphrased questions
  - Novel questions
  - Learning validation
- Reports per-phase results
- Updates status with benchmark summary

### Expected Behavior (Quick Benchmark)
- Runs subset of questions
- Completes faster
- Still validates core functionality

### Assertions
```
ASSERT benchmark.completed == true
ASSERT benchmark.phases_failed == 0
ASSERT benchmark.timeouts == 0
ASSERT status.last_benchmark IS SET
ASSERT all_questions_have_telemetry_entries
```

### Test Function
`test_snow_leopard_benchmark_consistency`

---

## 8. Reset Operations

### 8a. Soft Reset (Experience Reset)

### Trigger
"Soft reset your experience and stats but keep what you have learned about my system."

### What Gets Reset
- XP (back to level 1, trust 0.5)
- Telemetry (cleared)
- Stats counters (cleared)

### What Is Preserved
- Knowledge base
- Recipe cache (Brain learning)
- Config

### Assertions
```
AFTER soft_reset:
  ASSERT xp.level == 1
  ASSERT xp.trust == 0.5
  ASSERT telemetry.is_empty == true
  ASSERT recipe_cache.exists == true  # Preserved!
  ASSERT knowledge.exists == true     # Preserved!
```

### Test Function
`test_soft_reset_behavior`

---

### 8b. Hard Reset (Factory Reset)

### Trigger
"Hard reset yourself to factory settings."
(Requires confirmation: "I UNDERSTAND AND CONFIRM FACTORY RESET")

### What Gets Cleared
- Everything from soft reset, PLUS:
- Knowledge base
- Recipe cache
- Brain learning

### What Is Preserved
- Binaries
- Config structure (but may need re-provision)

### Assertions
```
AFTER hard_reset:
  ASSERT xp.level == 1
  ASSERT xp.trust == 0.5
  ASSERT telemetry.is_empty == true
  ASSERT recipe_cache.is_empty == true    # Cleared!
  ASSERT knowledge.is_empty == true       # Cleared!
  ASSERT anna_can_start == true
  ASSERT anna_can_autoprovision == true
```

### Test Function
`test_hard_reset_behavior`

---

## 9. Debug Mode

### 9a. Debug Mode ON

### Trigger
"Enable debug mode" or set ANNA_DEBUG=1

### Expected Behavior
- Every question shows detailed trace:
  - Router decision
  - Brain check
  - LLM calls (if any)
  - Probe executions
  - Timing per step
  - Final answer with origin

### Expected Output Structure
```
[HH:MM:SS.mmm] [ITERATION_STARTED] ...
[HH:MM:SS.mmm] [JUNIOR_PLAN_STARTED] ...
  Intent: hardware_info
  Probes: ["cpu.info"]
[HH:MM:SS.mmm] [ANNA_PROBE] ...
  [OK] cpu.info (23ms)
[HH:MM:SS.mmm] [SENIOR_REVIEW_DONE] ...
  Verdict: approve
  Confidence: 92%
[HH:MM:SS.mmm] [ANSWER_READY] ...
  Reliability: 92%
```

### Assertions
```
ASSERT output.contains("[ITERATION_STARTED]") OR output.contains("[BRAIN]")
ASSERT output.contains("ms")  # Timing present
ASSERT output.contains("%")   # Percentages present
```

### Test Function
`test_debug_mode_output_structure`

---

### 9b. Debug Mode OFF

### Trigger
"Disable debug mode" or unset ANNA_DEBUG

### Expected Behavior
- Questions show only:
  - Final answer
  - Reliability percentage
  - Optional compact summary

### Assertions
```
ASSERT NOT output.contains("[ITERATION_STARTED]")
ASSERT NOT output.contains("[JUNIOR_PLAN")
ASSERT output.contains("Reliability:")
```

### Test Function
`test_debug_mode_disabled`

---

## 10. Stats and Percentages Verification

### Rule
ALL values conceptually in [0, 1] MUST be displayed as percentages [0%, 100%].

### Checked Locations
1. `annactl status` output
2. Answer reliability display
3. Benchmark results
4. Debug mode output
5. Trust and success rate displays

### Assertions
```
# In ANY user-visible output:
ASSERT NOT contains("0.87")     # Raw float
ASSERT NOT contains("reliability: 0")  # Raw float
ASSERT contains("87%") OR contains("Reliability: 87%")

# Stats verification:
ASSERT best_reliability IS PERCENTAGE
ASSERT worst_reliability IS PERCENTAGE
ASSERT median_reliability IS PERCENTAGE
ASSERT avg_reliability IS PERCENTAGE
```

### Test Function
`test_percentage_formatting_everywhere`

---

## 11. Stats Correctness

### Scenario
After running questions, verify stats match actual events.

### Metrics to Validate
- Total questions == telemetry event count
- Brain count == events with origin "Brain"
- Junior count == events with origin "Junior"
- Senior count == events with origin "Senior"
- Success rate == successes / total
- Avg latency == mean of all latencies
- Best/worst/median correctly computed

### Assertions
```
ASSERT stats.total == telemetry.events.count()
ASSERT stats.brain_count == telemetry.events.filter(origin=Brain).count()
ASSERT stats.success_rate == stats.successes / stats.total
ASSERT stats.min_latency <= stats.avg_latency <= stats.max_latency
ASSERT stats.min_reliability <= stats.median_reliability <= stats.max_reliability
```

### Test Function
`test_stats_match_telemetry`

---

## Execution Order

The "Day in the Life" scenario runs in this order:

1. **Fresh Install** - Validate clean state
2. **Autoprovision** - Set up LLMs
3. **First Light** - Self-test
4. **Canonical Questions (1st)** - Baseline performance
5. **Canonical Questions (2nd)** - Verify learning
6. **Natural Language** - Intent normalization
7. **Snow Leopard Full** - Comprehensive benchmark
8. **Snow Leopard Quick** - Fast validation
9. **Soft Reset** - Partial reset
10. **Verify Post-Soft-Reset** - Brain cache preserved
11. **Hard Reset** - Complete reset
12. **Verify Post-Hard-Reset** - Everything cleared
13. **Re-Autoprovision** - Can start fresh
14. **Debug Mode Tests** - Output validation

---

## Test Harness Requirements

### Fake Backends
To run these tests without real external dependencies:

- **FakeOllama**: Simulates Ollama installation and model pulls
- **FakeLlm**: Returns deterministic responses for test questions
- **FakeProbeExecutor**: Returns known outputs for system probes
- **TempFilesystem**: Isolated directories for XP, telemetry, knowledge

### Performance Budgets (Test Environment)
- Brain path: < 50ms
- LLM fake responses: < 100ms
- Probe execution: < 20ms
- Full benchmark: < 30s

### Determinism
Tests must be deterministic:
- Fixed timestamps where needed
- Seeded random if any
- Known question/answer mappings

---

## Success Criteria

v3.7.0 is complete when:

1. All test functions in `system_acceptance.rs` pass
2. No warnings in `cargo test --workspace`
3. `cargo clippy` clean
4. All percentage displays verified
5. Learning improvement demonstrated
6. Reset behavior exact as specified
7. Debug output structure validated

---

## Test Commands

```bash
# Full workspace test
cargo test --workspace

# Run only acceptance tests
cargo test -p annad system_acceptance

# Run specific acceptance test
cargo test -p annad test_fresh_install_state

# Run with output
cargo test -p annad system_acceptance -- --nocapture

# Individual crate tests
cargo test -p anna_common
cargo test -p annad
cargo test -p annactl
```

---

## Related Files

| File | Purpose |
|------|---------|
| `crates/annad/tests/system_acceptance.rs` | Main acceptance test module |
| `crates/anna_common/src/experience_reset.rs` | Reset implementation |
| `crates/anna_common/src/telemetry.rs` | Stats and metrics |
| `crates/anna_common/src/ui_colors.rs` | Percentage formatting |
| `crates/anna_common/src/bench_snow_leopard.rs` | Benchmark implementation |
| `crates/anna_common/src/first_light.rs` | Self-test implementation |

---

## Changelog

- v3.7.0: Complete rewrite as "Day in the Life" acceptance test plan
- v0.26.0: Original test plan with basic coverage tracking
