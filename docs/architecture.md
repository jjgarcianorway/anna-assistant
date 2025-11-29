# Anna Architecture v1.0.0

> Snow Leopard Stabilization Reference Document

This document describes **how Anna answers a question** - the single mental model
every developer should understand before modifying orchestration code.

## 1. High-Level Flow

```
User Question
     │
     ▼
┌─────────────────────────────────────────────────────────────────┐
│                     UnifiedEngine.process_question()            │
│                     (crates/annad/src/orchestrator/engine_v90.rs)│
└─────────────────────────────────────────────────────────────────┘
     │
     ├──► STEP 1: Brain Fast Path (NO LLM, <150ms)
     │         │
     │         ├── Match? → Return FastAnswer (origin: Brain)
     │         └── No match → Continue to Junior
     │
     ├──► STEP 2: Junior Planning (LLM call #1)
     │         │
     │         └── Request probes needed to answer
     │
     ├──► STEP 3: Execute Probes (safe commands only)
     │         │
     │         └── Collect evidence
     │
     ├──► STEP 4: Junior Draft (LLM call #2)
     │         │
     │         └── Draft answer using evidence
     │
     ├──► STEP 5: Senior Audit (LLM call #3, optional)
     │         │
     │         ├── Skip if: simple domain + confidence >= 80%
     │         └── Verdict: approve | fix_and_accept | refuse
     │
     ├──► STEP 6: Final Answer Assembly
     │         │
     │         └── Build FinalAnswer struct
     │
     └──► STEP 7: XP/Trust Updates
              │
              └── Record XpEvent for Anna/Junior/Senior
```

## 2. The Three Answer Origins

| Origin | When Used | Latency Target | LLM Calls |
|--------|-----------|----------------|-----------|
| **Brain** | Simple hardware questions (RAM, CPU, disk) | <150ms | 0 |
| **Junior** | Simple domain + high confidence (>=80%) | <15s | 2 |
| **Senior** | Complex questions or low confidence | <30s | 3 |

## 3. Brain Fast Path

**File:** `crates/anna_common/src/brain_fast.rs`

The Brain is a zero-LLM fast path for questions that can be answered by
running a single system command and parsing the output.

### Supported Question Types

| Type | Detection Pattern | Command Used |
|------|-------------------|--------------|
| RAM | "how much ram/memory" | `cat /proc/meminfo` |
| CPU Cores | "how many cores/threads" | `lscpu` |
| Disk Space | "free disk space" | `df -h /` |
| Anna Health | "are you ok/healthy" | `pgrep`, `curl /health` |
| Debug Mode | "enable/disable debug" | State file toggle |

### Brain Answer Structure

```rust
FastAnswer {
    text: String,        // Human-readable answer
    citations: Vec<String>, // Commands used
    reliability: f64,    // Always high (0.99) for Brain
    origin: "Brain",
    duration_ms: u64,    // Typically <50ms
}
```

## 4. Junior + Senior LLM Flow

**File:** `crates/annad/src/orchestrator/engine_v90.rs`

When Brain cannot answer, the orchestration falls through to the LLM path.

### Junior (2 calls max)

1. **Planning Call:** Given the question and available probes, decide which
   probes to run
2. **Draft Call:** Given probe evidence, draft an answer

Junior uses a smaller/faster model (configurable, default: qwen3).

### Senior (1 call max, optional)

Senior audits Junior's work and can:
- **approve:** Accept the answer as-is
- **fix_and_accept:** Minor corrections, still good
- **refuse:** Answer is unreliable, decline to answer

Senior uses a larger/smarter model for verification.

### Skip Senior Optimization

If the question is in a "simple domain" (hardware queries) and Junior's
confidence is >=80%, Senior is skipped entirely to save latency.

```rust
let skip_senior = is_simple_domain(question) && junior_confidence >= 0.80;
```

## 5. Time Budgets

| Stage | Budget | Hard Limit |
|-------|--------|------------|
| Brain | 150ms | - |
| Junior (per call) | 15s | - |
| Senior | 15s | - |
| **Total Orchestration** | 20s soft | **30s hard** |

If the hard limit is hit, a timeout answer is returned with reduced confidence.

## 6. XP and Trust System

**Files:**
- `crates/anna_common/src/xp_events.rs` - Event types and base XP values
- `crates/anna_common/src/xp_log.rs` - Persistent event log
- `crates/anna_common/src/xp_track.rs` - Actor stats (Anna/Junior/Senior)

### XP Events

| Event | XP | Trigger |
|-------|-----|---------|
| `BrainSelfSolve` | +15 | Brain answered without LLM |
| `JuniorCleanProposal` | +10 | Junior draft approved |
| `SeniorGreenApproval` | +12 | High-quality answer |
| `JuniorBadCommand` | -8 | Junior proposed failing command |
| `SeniorRepeatedFix` | -10 | Senior keeps fixing same error |
| `LlmTimeoutFallback` | -5 | Timeout forced fallback |

### Trust Routing (v0.92.0)

The `DecisionPolicy` uses accumulated XP to make routing decisions:
- High Anna XP → Prefer Brain path
- Low Junior XP → Force Senior review
- Circuit breaker → Disable failing actors temporarily

## 7. Debug Mode

**File:** `crates/anna_common/src/debug_state.rs`

Debug mode is a persistent toggle that shows orchestration internals.

### Enable/Disable

```
> enable debug mode
> turn off debug
> is debug enabled?
```

These are handled by Brain fast path (no LLM needed).

### What Debug Mode Shows

When enabled, answers include an `OrchestrationTrace`:
- Which path was taken (Brain/Junior/Senior)
- Probe executions and outputs
- Junior/Senior prompts and responses
- Timing breakdown
- XP events generated

## 8. Key Files Reference

| File | Purpose |
|------|---------|
| `crates/annad/src/orchestrator/engine_v90.rs` | Main orchestration logic |
| `crates/annad/src/orchestrator/llm_client.rs` | Ollama client wrapper |
| `crates/annad/src/orchestrator/probe_executor.rs` | Safe command execution |
| `crates/anna_common/src/brain_fast.rs` | Zero-LLM fast path |
| `crates/anna_common/src/xp_events.rs` | XP event definitions |
| `crates/anna_common/src/decision_policy.rs` | Routing and circuit breaker |
| `crates/anna_common/src/conversation_trace.rs` | Debug trace structures |

## 9. Legacy Code (Deprecated)

These files are no longer used but remain in the codebase:

| File | Status | Reason |
|------|--------|--------|
| `engine.rs` | Deprecated | Pre-v0.90 orchestration |
| `engine_v18.rs` | Deprecated | Legacy step-by-step flow |
| `engine_v19.rs` | Deprecated | Legacy subproblem decomposition |
| `engine_v80.rs` | Deprecated | Legacy Razorback flow |
| `research_engine.rs` | Deprecated | Unused research loop |

**Note:** These should be removed in a future cleanup pass.

## 10. Invariants

The following must always be true:

1. **Max LLM calls:** 2 Junior + 1 Senior = 3 total
2. **Hard timeout:** 30 seconds, enforced at each step
3. **No loops:** Each step executes exactly once
4. **Safe commands only:** Probes use whitelist (no rm, mv, etc.)
5. **XP recorded:** Every answer generates at least one XP event
6. **Origin tracked:** Every answer has model_used set

## 11. Adding New Fast Path Questions

To add a new Brain-handled question type:

1. Add variant to `FastQuestionType` enum in `brain_fast.rs`
2. Add classification patterns in `FastQuestionType::classify()`
3. Implement `fast_*_answer()` function
4. Add case to `try_fast_answer()` match
5. Add tests for classification and answer

## 12. Testing Strategy

| Layer | Test Type | Where |
|-------|-----------|-------|
| Brain | Unit tests | `brain_fast.rs` |
| XP Events | Unit tests | `xp_events.rs` |
| Orchestration | Integration | `annad/tests/` |
| End-to-end | QA harness | `ANNA_QA_MODE=1` |

For deterministic orchestration tests, use the LLM and Probe fakes
(see Phase 2 of stabilization plan).
