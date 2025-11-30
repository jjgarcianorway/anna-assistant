# Feature Integrity Matrix (FIM) v3.4.0

**Contract of Correctness for Anna Assistant**

This document defines the exact behavior, invariants, and boundaries for every subsystem.
Any deviation from this contract constitutes a bug.

---

## 1. BRAIN FAST PATH

### Intended Behavior
Zero-LLM fast path for simple, pattern-matched questions. Returns answers from direct system probes or micro-cached facts without any LLM calls.

### Required Invariants
- **INV-BRAIN-001**: Brain MUST NOT make any LLM calls
- **INV-BRAIN-002**: Brain MUST return `AnswerOrigin::Brain` for all answers
- **INV-BRAIN-003**: Brain reliability MUST be >= 0.90 (Green)
- **INV-BRAIN-004**: Brain MUST use micro-cache (60s TTL) for CPU model and RAM total
- **INV-BRAIN-005**: Brain MUST NOT return empty answer text

### Dependency Constraints
- Depends on: `/proc/meminfo`, `lscpu`, `df`, system commands
- No dependency on: Ollama, network, daemon (for direct annactl calls)

### Latency Boundaries
| Metric | Target | Hard Limit |
|--------|--------|------------|
| Cached facts (CPU, RAM) | <50ms | 150ms |
| Fresh probes (disk, health) | <100ms | 150ms |
| Pattern matching | <10ms | 50ms |

### Reliability Boundaries
| Scenario | Reliability |
|----------|-------------|
| Cached fact hit | 0.99 (Green) |
| Fresh probe success | 0.95 (Green) |
| Partial answer | 0.80 (Yellow) |

### Success Definition
- Question pattern matched
- Answer text is non-empty
- Reliability >= 0.90
- Latency < 150ms
- Origin = "Brain"

### Failure Definition
- Return `None` (not an error - routes to next layer)
- NEVER return partial or empty answer
- NEVER exceed 150ms timeout

### Supported Patterns
```
RAM:        "how much ram", "memory", "how many gb ram"
CPU Cores:  "how many cpu", "cores", "threads"
CPU Model:  "what cpu", "processor", "cpu model"
Disk:       "disk space", "root filesystem", "free storage"
Health:     "are you healthy", "health status"
Debug:      "enable debug", "disable debug", "debug status"
Reset:      "soft reset", "hard reset", "factory reset"
Benchmark:  "run benchmark", "snow leopard"
```

---

## 2. ROUTER LLM

### Intended Behavior
Optional fast classification layer (500ms budget) that categorizes questions for intelligent routing.

### Required Invariants
- **INV-ROUTER-001**: Router is OPTIONAL - Brain patterns work without it
- **INV-ROUTER-002**: Router MUST return valid `QuestionType` enum
- **INV-ROUTER-003**: Router MUST respect 1000ms hard timeout
- **INV-ROUTER-004**: Router MUST NOT block Brain fast path

### Dependency Constraints
- Depends on: Ollama (qwen2.5:0.5b or fallback)
- Fallback: Brain heuristics if router unavailable

### Latency Boundaries
| Metric | Target | Hard Limit |
|--------|--------|------------|
| Classification | 500ms | 1000ms |

### Reliability Boundaries
| Scenario | Reliability |
|----------|-------------|
| Correct classification | 0.95 |
| Fallback to heuristics | 0.80 |

### Success Definition
- Valid QuestionType returned within timeout
- Classification matches question intent

### Failure Definition
- Timeout (>1000ms) → fallback to Brain heuristics
- Parse error → fallback to Brain heuristics
- NEVER block the pipeline

---

## 3. RECIPE STORE

### Intended Behavior
Learn from successful answers (reliability >= 0.85) and reuse patterns for similar future questions.

### Required Invariants
- **INV-RECIPE-001**: Recipes MUST only be extracted from answers with reliability >= 0.85
- **INV-RECIPE-002**: Recipe match threshold MUST be >= 0.70
- **INV-RECIPE-003**: Recipe MUST contain valid probe IDs from catalog
- **INV-RECIPE-004**: Recipe store MUST persist to `/var/lib/anna/recipes/store.json`
- **INV-RECIPE-005**: MAX 20 recipes per QuestionType
- **INV-RECIPE-006**: Recipe answer MUST have `AnswerOrigin::Recipe`
- **INV-RECIPE-007**: Recipe MUST NOT match different intents

### Dependency Constraints
- Depends on: File system (writable /var/lib/anna)
- Depends on: Valid QuestionType classification
- Depends on: Probe catalog for validation

### Latency Boundaries
| Metric | Target | Hard Limit |
|--------|--------|------------|
| Recipe lookup | <100ms | 200ms |
| Probe execution (from recipe) | <300ms | 500ms |
| Total recipe path | <400ms | 500ms |

### Reliability Boundaries
| Scenario | Reliability |
|----------|-------------|
| Recipe match + fresh probes | 0.90+ (Green) |
| Recipe match + stale probes | 0.80+ (Yellow) |

### Success Definition
- Recipe matched on question type + key tokens
- Probes executed successfully
- Answer template filled correctly
- Reliability >= 0.85
- Latency < 500ms

### Failure Definition
- No matching recipe → route to Junior
- Probe execution fails → route to Junior
- Template fill fails → route to Junior
- NEVER return incomplete recipe answer

### Data Structure
```rust
Recipe {
    id: String,                    // UUID
    intent: String,                // e.g., "ram_total"
    question_type: QuestionType,
    parameters: HashMap<String, String>,
    key_tokens: Vec<String>,       // Match tokens
    probes: Vec<String>,           // Probe IDs to run
    answer_template: String,       // {placeholder} format
    last_success_score: f64,       // >= 0.85
    usage_count: u32,
    last_used: u64,                // Unix timestamp
}
```

---

## 4. PROBE SYSTEM

### Intended Behavior
Execute whitelisted system commands with structured output parsing. Provides evidence for LLM answers.

### Required Invariants
- **INV-PROBE-001**: Probes MUST only execute whitelisted commands
- **INV-PROBE-002**: Probes MUST validate parameters against injection patterns
- **INV-PROBE-003**: Probes MUST block path traversal (`..`)
- **INV-PROBE-004**: Probes MUST block null bytes (`\0`)
- **INV-PROBE-005**: Parameters MUST NOT exceed 4KB
- **INV-PROBE-006**: Probes MUST return structured `ProbeResult`
- **INV-PROBE-007**: Failed probes MUST have `success: false` and error message

### Blocked Patterns (Security)
```
Shell metacharacters: | ; & ` $ ( ) < > \n
Path traversal: ..
Null bytes: \0
Max length: 4096 bytes
```

### Dependency Constraints
- Depends on: System commands (lscpu, df, ip, etc.)
- No network dependency for local probes

### Latency Boundaries
| Probe Type | Target | Hard Limit |
|------------|--------|------------|
| CPU info | <50ms | 200ms |
| Memory info | <30ms | 100ms |
| Disk info | <100ms | 300ms |
| Network info | <100ms | 300ms |
| Service status | <200ms | 500ms |

### Reliability Boundaries
| Scenario | Reliability |
|----------|-------------|
| Probe success | 1.0 |
| Probe cached hit | 1.0 |
| Probe timeout | 0.0 (error) |
| Probe command fail | 0.0 (error) |

### Success Definition
- Command executed within timeout
- Output parsed successfully
- Structured data returned
- `success: true` in result

### Failure Definition
- `success: false` with error message
- Timeout after hard limit
- Parse failure returns raw output
- NEVER execute non-whitelisted commands

### Cache Policies
```rust
Static:   Never expires (hardware info)
Slow:     1 hour TTL (configs, packages)
Volatile: 5 seconds TTL (memory, CPU load)
```

---

## 5. JUNIOR/SENIOR LLM PIPELINE

### Intended Behavior
Dual-LLM orchestration: Junior plans/drafts, Senior audits/verifies. Provides high-quality answers for complex questions.

### Required Invariants
- **INV-LLM-001**: Junior MUST return valid JSON with probe_requests or draft_answer
- **INV-LLM-002**: Senior MUST return verdict in ["approve", "fix_and_accept", "needs_more_checks", "refuse"]
- **INV-LLM-003**: Senior scores MUST be 0.0-1.0 for evidence, reasoning, coverage
- **INV-LLM-004**: Overall reliability = min(evidence, reasoning, coverage)
- **INV-LLM-005**: Max 6 iterations per question (needs_more_checks loop)
- **INV-LLM-006**: Senior MUST be skipped if Junior confidence >= 0.80
- **INV-LLM-007**: Fallback MUST activate on LLM timeout/error
- **INV-LLM-008**: NEVER return empty answer text

### Dependency Constraints
- Depends on: Ollama service running
- Depends on: Valid models pulled (Junior, Senior)
- Depends on: Probe system for evidence

### Latency Boundaries
| Stage | Target | Hard Limit |
|-------|--------|------------|
| Junior planning | 2s | 4s |
| Probe execution | 1s | 2s |
| Junior draft | 2s | 4s |
| Senior audit | 3s | 5s |
| **Total pipeline** | **6s** | **10s** |

### Reliability Boundaries
| Scenario | Reliability |
|----------|-------------|
| Senior approve | 0.90+ (Green) |
| Senior fix_and_accept | 0.80+ (Yellow) |
| Junior only (high confidence) | 0.80+ (Yellow) |
| Fallback answer | 0.40-0.50 (Red) |
| Timeout/error | 0.30 (Red) |

### Success Definition
- Answer text is non-empty
- Reliability >= 0.50
- Latency < 10s
- Origin correctly set (Junior or Senior)

### Failure Definition
- LLM timeout → Fallback answer (Red)
- LLM parse error → Fallback answer (Red)
- Senior refuse → Refusal answer (Red)
- NEVER hang > 10s
- NEVER return empty answer

### Verdict Handling
```rust
"approve"          → Use answer as-is, Green/Yellow reliability
"fix_and_accept"   → Use corrected_answer, Yellow reliability
"needs_more_checks"→ Loop (max 6 iterations)
"refuse"           → Return refusal answer, Red reliability
```

---

## 6. XP & TITLES SYSTEM

### Intended Behavior
Track performance of Anna, Junior, and Senior agents. Affects trust-based routing and displays RPG-style progression.

### Required Invariants
- **INV-XP-001**: XP MUST only increase on positive events (never decrease)
- **INV-XP-002**: Trust MUST be clamped to 0.0-1.0
- **INV-XP-003**: Level MUST be clamped to 1-99
- **INV-XP-004**: Streak counters MUST reset on opposite event type
- **INV-XP-005**: XP store MUST persist atomically (temp file + rename)
- **INV-XP-006**: XP store path: `/var/lib/anna/xp/xp_store.json`

### XP Event Values
| Event | XP | Trust Delta |
|-------|----|-----------:|
| BrainSelfSolve | +15 | +0.02 |
| JuniorCleanProposal | +10 | +0.02 |
| SeniorGreenApproval | +12 | +0.02 |
| StablePatternDetected | +20 | +0.03 |
| JuniorBadCommand | 0 | -0.05 |
| LlmTimeoutFallback | 0 | -0.03 |
| LowReliabilityRefusal | 0 | -0.02 |

### Dependency Constraints
- Depends on: File system (writable /var/lib/anna/xp)
- No network dependency

### Latency Boundaries
| Operation | Target | Hard Limit |
|-----------|--------|------------|
| Load store | <50ms | 100ms |
| Save store | <50ms | 100ms |
| Record event | <10ms | 50ms |

### Success Definition
- Event recorded correctly
- Store saved atomically
- Level/title updated if threshold crossed

### Failure Definition
- File I/O error → log warning, continue (non-fatal)
- Corrupt store → reset to defaults
- NEVER crash on XP errors

### Title Progression
```
Level 1-4:   Intern
Level 5-9:   Junior Specialist
Level 10-19: Specialist
Level 20-34: Senior Specialist
Level 35-49: Lead
Level 50-69: Principal
Level 70-89: Archon
Level 90-99: Mythic
```

---

## 7. TELEMETRY SYSTEM

### Intended Behavior
Local-only diagnostic telemetry for understanding real-world performance. Privacy-preserving (questions hashed).

### Required Invariants
- **INV-TEL-001**: Questions MUST be hashed (SHA256, first 16 chars)
- **INV-TEL-002**: Telemetry MUST NOT include raw question text
- **INV-TEL-003**: Events MUST include: timestamp, correlation_id, outcome, origin, latency_ms, reliability
- **INV-TEL-004**: Telemetry MUST persist to JSONL file
- **INV-TEL-005**: Telemetry path: `/var/log/anna/telemetry.jsonl`
- **INV-TEL-006**: Fallback path if unwritable: `/tmp/anna-telemetry.jsonl`

### Event Schema
```rust
TelemetryEvent {
    timestamp: String,        // ISO 8601
    correlation_id: String,
    question_hash: String,    // SHA256 first 16 chars
    outcome: Outcome,         // Success, Failure, Timeout, Refusal
    origin: Origin,           // Brain, Junior, Senior, Fallback
    latency_ms: u64,
    reliability: f64,
    xp_awarded: i32,
    model_used: Option<String>,
    error_type: Option<String>,
}
```

### Dependency Constraints
- Depends on: File system (writable /var/log/anna or /tmp)
- No network dependency (local-only)

### Latency Boundaries
| Operation | Target | Hard Limit |
|-----------|--------|------------|
| Record event | <10ms | 50ms |
| Load summary | <100ms | 500ms |

### Success Definition
- Event appended to JSONL file
- All required fields populated
- Question hash (not plaintext) stored

### Failure Definition
- File I/O error → use fallback path or skip (non-fatal)
- NEVER block answer pipeline for telemetry
- NEVER store plaintext questions

---

## 8. BENCHMARK SYSTEM (Snow Leopard)

### Intended Behavior
Comprehensive runtime benchmark measuring reliability, UX, learning, and reset integrity across 6 phases.

### Required Invariants
- **INV-BENCH-001**: Benchmark MUST use the SAME pipeline as runtime (no bypass)
- **INV-BENCH-002**: Benchmark MUST test all answer origins (Brain, Recipe, Junior, Senior)
- **INV-BENCH-003**: Phase 3 (SoftReset) MUST clear XP/telemetry but preserve knowledge
- **INV-BENCH-004**: Phase 1 (HardReset) MUST clear everything
- **INV-BENCH-005**: Benchmark results MUST include per-phase metrics
- **INV-BENCH-006**: Learning test (Phase 6) MUST show improvement on repeated questions

### Phases
| Phase | Name | Purpose |
|-------|------|---------|
| 1 | HardReset | Factory reset + canonical questions |
| 2 | WarmState | No reset + canonical questions |
| 3 | SoftReset | Experience reset + canonical questions |
| 4 | NLStress | Paraphrased variants |
| 5 | NovelQuestions | Never-before-seen questions |
| 6 | LearningTest | Repeat learning test |

### Canonical Questions
```
1. "How many CPU cores and threads do I have?"
2. "How much RAM is installed and available?"
3. "How much space is free on root filesystem?"
4. "What is your health status?"
5. "Are Junior and Senior LLM models working?"
```

### Dependency Constraints
- Depends on: Full Anna system (Brain, Recipe, LLM, Probes)
- Depends on: Reset system for Phase 1, 3

### Latency Boundaries
| Mode | Target | Hard Limit |
|------|--------|------------|
| Quick (3 phases) | 2 minutes | 5 minutes |
| Full (6 phases) | 5 minutes | 15 minutes |

### Success Definition
- All phases complete
- Success rate >= 80%
- Latency within targets
- Reset integrity verified

### Failure Definition
- Phase fails if success rate < 50%
- Abort on critical error (LLM unavailable)
- Results saved even on partial failure

---

## 9. DEBUG MODE

### Intended Behavior
Persistent debug mode toggled via natural language. Streams detailed transcripts when enabled.

### Required Invariants
- **INV-DEBUG-001**: Debug state MUST persist across daemon restarts
- **INV-DEBUG-002**: Debug mode MUST NOT change answer content
- **INV-DEBUG-003**: Debug mode MUST NOT significantly change latency (±5%)
- **INV-DEBUG-004**: Debug path: `~/.config/anna/debug_state.json`
- **INV-DEBUG-005**: Debug output MUST include Junior/Senior prompts and responses

### Toggle Patterns
```
Enable:  "enable debug mode", "turn on debugging", "debug please"
Disable: "disable debug mode", "turn off debugging"
Status:  "is debug mode enabled?", "debug status"
```

### Dependency Constraints
- Depends on: File system (writable ~/.config/anna)
- No network dependency

### Success Definition
- State persists correctly
- Toggle patterns recognized
- Debug output shown when enabled
- No debug output in normal mode

### Failure Definition
- File I/O error → default to disabled
- NEVER pollute normal output with debug
- NEVER change answer content based on debug mode

---

## 10. RESET SYSTEM

### Intended Behavior
Two-mode reset: Experience (soft) resets XP/telemetry only; Factory (hard) deletes everything.

### Required Invariants
- **INV-RESET-001**: Soft reset MUST clear XP, telemetry, stats
- **INV-RESET-002**: Soft reset MUST preserve knowledge, config, recipes
- **INV-RESET-003**: Hard reset MUST clear everything from soft reset + knowledge
- **INV-RESET-004**: Hard reset MUST preserve config and models
- **INV-RESET-005**: Hard reset MUST require strong confirmation phrase
- **INV-RESET-006**: After hard reset, Anna MUST boot into First Light

### Soft Reset Clears
```
/var/lib/anna/xp/xp_store.json     → Fresh XpStore (level 1, trust 0.5)
/var/log/anna/telemetry.jsonl      → Truncated
/var/lib/anna/knowledge/stats/*    → Deleted
```

### Hard Reset Clears (additional)
```
/var/lib/anna/knowledge/*          → Deleted entirely
/var/lib/anna/recipes/store.json   → Deleted
```

### Preserved (both modes)
```
/etc/anna/*                        → Config preserved
Ollama models                      → Not touched
```

### Confirmation Requirements
| Reset Type | Confirmation |
|------------|--------------|
| Soft | "yes", "y", "confirm", "ok" |
| Hard | "I UNDERSTAND AND CONFIRM FACTORY RESET" |

### Dependency Constraints
- Depends on: File system (writable /var/lib/anna, /var/log/anna)

### Success Definition
- Correct files cleared/preserved per mode
- Confirmation validated
- Anna operational after reset

### Failure Definition
- Partial reset → log error, report to user
- Permission error → report to user
- NEVER silently fail

---

## 11. AUTOPROVISION SYSTEM

### Intended Behavior
Automatic LLM model detection, installation, benchmarking, and selection at startup.

### Required Invariants
- **INV-AUTO-001**: Hardware tier detection MUST be deterministic
- **INV-AUTO-002**: Model selection MUST be based on benchmark scores
- **INV-AUTO-003**: Junior JSON compliance MUST be >= 0.95
- **INV-AUTO-004**: Fallback timeout: 2 seconds for Junior
- **INV-AUTO-005**: Selection MUST persist to `/var/lib/anna/llm/selection.json`
- **INV-AUTO-006**: Auto-download MUST be triggered if selected model missing

### Hardware Tiers
| Tier | Cores | RAM | GPU |
|------|-------|-----|-----|
| Minimal | <4 | <4GB | No |
| Basic | 4-7 | 4-8GB | No |
| Standard | 8-15 | 8-16GB | No |
| Power | 16+ | 16GB+ | Yes (NVIDIA) |

### Model Selection Criteria
```rust
Junior Score = (json_compliance * 0.5) + (latency_factor * 0.3) + (determinism * 0.2)
Senior Score = (reasoning_score * 0.5) + (latency_factor * 0.3) + (determinism * 0.2)
```

### Dependency Constraints
- Depends on: Ollama service
- Depends on: Network (for model download)
- Depends on: File system (for selection persistence)

### Success Definition
- Hardware tier detected correctly
- Valid models selected for Junior/Senior
- Benchmark scores recorded
- Selection persisted

### Failure Definition
- No models available → Brain-only mode
- Download fails → retry with fallback model
- NEVER block startup indefinitely

---

## 12. ANSWER CONSTRUCTION

### Intended Behavior
Format final answers with consistent structure: text, reliability, origin, duration. Every answer path MUST use unified formatting.

### Required Invariants
- **INV-ANS-001**: Answer text MUST NOT be empty
- **INV-ANS-002**: Reliability MUST be 0.0-1.0
- **INV-ANS-003**: Reliability label MUST match thresholds (Green >= 0.90, Yellow >= 0.70, Red >= 0.50)
- **INV-ANS-004**: Origin MUST be one of: Brain, Recipe, Junior, Senior, Fallback
- **INV-ANS-005**: Duration MUST be actual elapsed milliseconds
- **INV-ANS-006**: Every answer MUST go through unified formatting function
- **INV-ANS-007**: Refuse if reliability < 0.50 (with explanation)

### Reliability Thresholds
| Label | Threshold | Meaning |
|-------|-----------|---------|
| Green | >= 0.90 | High confidence, well-grounded |
| Yellow | >= 0.70 | Medium confidence, may have gaps |
| Red | >= 0.50 | Low confidence, use with caution |
| Refuse | < 0.50 | Cannot answer reliably |

### Answer Structure
```rust
FinalAnswer {
    text: String,              // Non-empty answer
    reliability: f64,          // 0.0-1.0
    reliability_label: String, // "Green", "Yellow", "Red"
    origin: String,            // "Brain", "Recipe", etc.
    duration_ms: u64,          // Actual elapsed time
    xp_awarded: i32,           // XP for this answer
}
```

### Dependency Constraints
- All answer paths MUST call unified formatting
- No dependency on specific answer source

### Success Definition
- All fields populated correctly
- Label matches reliability threshold
- Origin reflects actual answer source
- Duration is accurate

### Failure Definition
- Empty text → return error, not empty answer
- Invalid reliability → clamp to 0.0-1.0
- NEVER return malformed answer

---

## 13. ERROR HANDLING PIPELINE

### Intended Behavior
Graceful degradation with informative error messages. Never crash, never hang, never return gibberish.

### Required Invariants
- **INV-ERR-001**: LLM timeout → Fallback answer with Red reliability
- **INV-ERR-002**: LLM parse error → Fallback answer with Red reliability
- **INV-ERR-003**: Probe failure → Continue with available evidence
- **INV-ERR-004**: File I/O error → Log and continue (non-fatal)
- **INV-ERR-005**: NEVER return empty answer
- **INV-ERR-006**: NEVER hang > 10s total
- **INV-ERR-007**: ALWAYS include error explanation in failure answers

### Error Categories
| Category | Response | Reliability |
|----------|----------|-------------|
| LLM Timeout | Fallback from evidence | 0.40 |
| LLM Parse Error | Fallback from evidence | 0.30 |
| Probe Failure | Continue without probe | N/A |
| Senior Refuse | Refusal answer | 0.50 |
| No Evidence | "Cannot answer" | 0.0 |
| System Error | Error message | 0.0 |

### Fallback Answer Format
```
[Partial Answer]
Based on available system information:
{extracted_facts}

Note: This answer has limited reliability ({reliability}%) due to {error_reason}.
```

---

## VERIFICATION CHECKLIST

For each subsystem, the following MUST be tested:

- [ ] Positive case (happy path)
- [ ] Negative case (expected failure)
- [ ] Timeout/slow-path handling
- [ ] Bad input handling
- [ ] State reset behavior (where applicable)
- [ ] Debug mode parity (same behavior ±5% latency)

---

## INVARIANT VIOLATION PROTOCOL

When any invariant is violated:

1. **Log**: Structured "INVARIANT_VIOLATION" event with:
   - Invariant ID (e.g., INV-BRAIN-001)
   - Expected value
   - Actual value
   - Stack context

2. **Recover**: Return safe fallback:
   - For answers: Red reliability fallback
   - For XP: Skip recording (non-fatal)
   - For telemetry: Skip recording (non-fatal)

3. **Alert**: In debug mode, print violation details

4. **Test**: Integrity suite MUST fail on any violation

---

## 14. PERFORMANCE BUDGETS (v3.4.0)

### Intended Behavior
All pipeline operations complete within defined time budgets. When budgets are exceeded, produce fast degraded answers with honest reliability scores.

### Required Invariants
- **INV-PERF-001**: Global budget MUST be 10-20 seconds (currently 15s)
- **INV-PERF-002**: Fast path MUST complete in <1 second
- **INV-PERF-003**: Junior soft timeout < Junior hard timeout
- **INV-PERF-004**: Senior soft timeout < Senior hard timeout
- **INV-PERF-005**: Degraded answer MUST generate in <2 seconds
- **INV-PERF-006**: GlobalBudget MUST track remaining time accurately
- **INV-PERF-007**: Performance hints MUST be computed from rolling stats

### Timeout Hierarchy
| Tier | Timeout | Response |
|------|---------|----------|
| Junior Soft | 4s | Warning logged, degradation hint |
| Junior Hard | 6s | Cancel LLM, use partial evidence |
| Senior Soft | 5s | Warning logged, degradation hint |
| Senior Hard | 8s | Cancel LLM, produce RED answer |
| Global | 15s | Emergency fallback |
| Degraded | 2s | Fast RED answer generation |

### Anti-Hardcoding Policy
- **INV-PERF-015**: Brain MUST use generic pattern matching
- **INV-PERF-015**: NO question-specific string matching
- **INV-PERF-015**: Case-insensitive patterns only
- Variations of same question MUST route to same handler

---

## REVISION HISTORY

| Version | Date | Changes |
|---------|------|---------|
| 3.4.0 | 2025-11-30 | Added Performance Budgets section, anti-hardcoding policy |
| 3.3.0 | 2025-11-30 | Initial FIM creation |
