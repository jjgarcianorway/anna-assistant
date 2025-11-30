# Feature Verification Matrix (FVM) v3.5.0

**Complete Test Coverage Mapping for Anna Assistant**

This document maps every major feature to its implementation, tests, and success criteria.
Any feature without explicit tests documented here is a coverage gap that must be addressed.

---

## 1. BRAIN FAST PATHS

### Feature Description
Zero-LLM fast path for simple, pattern-matched questions. Returns answers from direct system probes without any LLM calls. Target latency: <150ms.

### Implementation
| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| Brain classifier | `anna_common/src/brain_fast.rs` | `FastQuestionType`, `try_fast_answer()` |
| Fast answers | `anna_common/src/brain_fast.rs` | `fast_ram_answer()`, `fast_cpu_answer()`, `fast_disk_answer()` |
| Micro-cache | `anna_common/src/brain_fast.rs` | `get_cached_cpu_info()`, `get_cached_ram_total()` |

### Test Coverage
| Test File | Test Name | What It Verifies |
|-----------|-----------|------------------|
| `annad/tests/integrity_suite.rs` | `test_inv_brain_*` | INV-BRAIN-001 through INV-BRAIN-005 |
| `annad/tests/baseline_tests.rs` | `test_brain_latency_*` | Brain answers complete in <150ms |
| `anna_common/src/brain_fast.rs` | `test_ram_classification` | RAM questions classified correctly |
| `anna_common/src/brain_fast.rs` | `test_cpu_classification` | CPU questions classified correctly |
| `anna_common/src/brain_fast.rs` | `test_disk_classification` | Disk questions classified correctly |
| `anna_common/src/brain_fast.rs` | `test_health_classification` | Health questions classified correctly |
| `anna_common/src/brain_fast.rs` | `test_uptime_classification` | Uptime questions classified correctly |
| `anna_common/src/brain_fast.rs` | `test_os_classification` | OS questions classified correctly |
| `anna_common/src/brain_fast.rs` | `test_logs_classification` | Logs questions classified correctly |

### Pass Criteria
- [x] Brain MUST NOT make any LLM calls
- [x] Brain MUST return `AnswerOrigin::Brain` for all answers
- [x] Brain reliability MUST be >= 0.90 (Green)
- [x] Brain MUST complete in <150ms
- [x] Repeated questions MUST hit Brain/Recipe, never LLM
- [x] Paraphrases of canonical questions MUST route to Brain

### Coverage Status: **COMPLETE**

---

## 2. ROUTER & RECIPE SELECTION

### Feature Description
Skill-based routing and learned recipe matching. Routes questions to appropriate handlers and reuses successful answer patterns.

### Implementation
| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| Skill router | `anna_common/src/skill_router.rs` | `SkillRouter`, `classify_question()` |
| Recipe store | `anna_common/src/recipe.rs` | `Recipe`, `RecipeStore`, `extract_recipe()` |
| Decision policy | `anna_common/src/decision_policy.rs` | `DecisionPolicy`, `route_question()` |

### Test Coverage
| Test File | Test Name | What It Verifies |
|-----------|-----------|------------------|
| `annad/tests/integrity_suite.rs` | `test_inv_recipe_*` | INV-RECIPE-001 through INV-RECIPE-005 |
| `anna_common/src/skill_router.rs` | 34 inline tests | Skill classification, intent detection |
| `anna_common/src/recipe.rs` | `test_recipe_extraction` | Recipes extracted from high-reliability answers |
| `anna_common/src/recipe.rs` | `test_recipe_matching` | Correct recipe chosen for variants |
| `anna_common/src/decision_policy.rs` | `test_brain_domain_*` | Brain domain routing |

### Pass Criteria
- [x] Correct recipe chosen for semantically equivalent variants
- [x] No recipe chosen when question is truly out-of-scope
- [x] Recipe reliability >= 0.85 (minimum extraction threshold)
- [x] Recipe path completes in <500ms

### Coverage Status: **COMPLETE**

---

## 3. LLM ORCHESTRATION (Junior/Senior)

### Feature Description
Dual-LLM orchestration with Junior (planner/drafter) and Senior (reviewer/auditor). Handles success, partial, timeout, and refusal scenarios.

### Implementation
| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| Junior protocol | `anna_common/src/llm_protocol.rs` | `LlmAOutput`, `LlmAPlan` |
| Senior protocol | `anna_common/src/llm_protocol.rs` | `LlmBOutput`, `Verdict` |
| Orchestrator | `annad/src/orchestrator/` | `Orchestrator`, `run_pipeline()` |
| Fake LLM client | `annad/src/orchestrator/llm_trait.rs` | `FakeLlmClient` |

### Test Coverage
| Test File | Test Name | What It Verifies |
|-----------|-----------|------------------|
| `annad/tests/orchestration_tests.rs` | `test_flow_direct_answer` | Brain bypass, no LLM |
| `annad/tests/orchestration_tests.rs` | `test_flow_probe_then_answer` | Junior probe + draft |
| `annad/tests/orchestration_tests.rs` | `test_flow_senior_correction` | Senior fix_and_accept |
| `annad/tests/orchestration_tests.rs` | `test_flow_senior_refuses` | Senior refusal handling |
| `annad/tests/orchestration_tests.rs` | `test_llm_unavailable` | Fallback on LLM failure |
| `annad/tests/orchestration_tests.rs` | `test_probe_failures` | Continue with partial evidence |
| `annad/tests/integrity_suite.rs` | `test_inv_llm_*` | INV-LLM-001 through INV-LLM-005 |

### Pass Criteria
- [x] Junior-only success case works
- [x] Junior + Senior review and fix case works
- [x] Timeout at Junior → degraded answer respecting global budget
- [x] Timeout at Senior → Junior answer used with proper markings
- [x] Invalid JSON → fallback answer with Red reliability
- [x] LLM refusal → refusal answer with explanation

### Coverage Status: **COMPLETE**

---

## 4. GLOBAL LATENCY BUDGET

### Feature Description
Per-question time budget enforcement. Maximum 15 seconds per question with tiered timeouts for Junior (4s/6s) and Senior (5s/8s).

### Implementation
| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| Global budget | `anna_common/src/perf_timing.rs` | `GlobalBudget`, `DEFAULT_GLOBAL_BUDGET_MS` |
| Timeout constants | `anna_common/src/perf_timing.rs` | `JUNIOR_SOFT_TIMEOUT_MS`, `SENIOR_HARD_TIMEOUT_MS` |
| Timeout result | `anna_common/src/perf_timing.rs` | `LlmTimeoutResult` |
| PerfSpan | `anna_common/src/perf_timing.rs` | `PerfSpan`, `PipelineTimings` |

### Test Coverage
| Test File | Test Name | What It Verifies |
|-----------|-----------|------------------|
| `annad/tests/integrity_suite.rs` | `test_inv_perf_001_global_budget_reasonable` | Budget is 10-20s |
| `annad/tests/integrity_suite.rs` | `test_inv_perf_002_fast_path_budget_tiny` | Fast path <1s |
| `annad/tests/integrity_suite.rs` | `test_inv_perf_003_timeout_ordering` | soft < hard timeouts |
| `annad/tests/integrity_suite.rs` | `test_inv_perf_004_perf_span_accuracy` | PerfSpan timing accuracy |
| `annad/tests/integrity_suite.rs` | `test_inv_perf_005_global_budget_tracking` | Remaining time tracking |
| `annad/tests/integrity_suite.rs` | `test_inv_perf_006_budget_exhaustion` | Exhaustion detection |
| `anna_common/src/perf_timing.rs` | `test_global_budget_*` | Budget lifecycle |

### Pass Criteria
- [x] No test takes longer than GLOBAL_BUDGET_MS + 1s epsilon
- [x] Red answers returned quickly with honest message
- [x] Junior soft (4s) < Junior hard (6s) < Senior soft (5s) < Senior hard (8s)
- [x] Degraded answers generated within 2s

### Coverage Status: **COMPLETE**

---

## 5. XP TRACKING (Anna, Junior, Senior)

### Feature Description
RPG-style experience tracking for Anna (self-solve), Junior (planning), and Senior (review). Persistent storage with levels and trust.

### Implementation
| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| XP tracker | `anna_common/src/xp_track.rs` | `XpTracker`, `XpGain`, `XpPenalty` |
| XP events | `anna_common/src/xp_events.rs` | `XpEvent`, `XpEventType` |
| XP log | `anna_common/src/xp_log.rs` | `XpLog`, `StoredXpEvent` |
| Progression | `anna_common/src/progression/` | `AnnaProgression`, `Level`, `Title` |

### Test Coverage
| Test File | Test Name | What It Verifies |
|-----------|-----------|------------------|
| `annad/tests/integrity_suite.rs` | `test_inv_xp_*` | INV-XP-001 through INV-XP-005 |
| `anna_common/src/xp_track.rs` | `test_xp_increment_*` | XP increments correctly |
| `anna_common/src/xp_track.rs` | `test_level_calculation` | Levels computed correctly |
| `anna_common/src/xp_track.rs` | `test_trust_bounds` | Trust stays in [0.0, 1.0] |
| `anna_common/src/progression/xp.rs` | `test_xp_*` | 9 progression tests |

### Pass Criteria
- [x] XP increments for: Brain self_solve, Junior good_plan/bad_plan, Senior approve/fix_and_accept
- [x] XP persisted correctly after restart
- [x] Hard reset clears XP
- [x] Soft reset clears XP but keeps knowledge
- [x] XP and levels are monotonic (no negative XP)
- [x] Trust values stay in [0.0, 1.0]

### Coverage Status: **COMPLETE**

---

## 6. TELEMETRY & PERFORMANCE HINTS

### Feature Description
Local telemetry for performance diagnostics. Rolling averages, success rates, timeout tracking, and performance hints.

### Implementation
| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| Telemetry | `anna_common/src/telemetry.rs` | `TelemetryEvent`, `TelemetrySummary` |
| Rolling stats | `anna_common/src/telemetry.rs` | `RollingStats`, `get_rolling_stats()` |
| Performance hints | `anna_common/src/perf_timing.rs` | `PerformanceHint`, `get_performance_hint()` |

### Test Coverage
| Test File | Test Name | What It Verifies |
|-----------|-----------|------------------|
| `annad/tests/integrity_suite.rs` | `test_inv_tel_*` | INV-TEL-001 through INV-TEL-005 |
| `annad/tests/integrity_suite.rs` | `test_inv_perf_007_performance_hints` | Hint transitions |
| `anna_common/src/telemetry.rs` | `test_telemetry_*` | 12 inline tests |
| `anna_common/src/telemetry.rs` | `test_rolling_stats_*` | Rolling average accuracy |

### Pass Criteria
- [x] Summary metrics correct (rolling averages, success, timeout)
- [x] PerformanceHint transitions: Good → Degraded → Critical
- [x] success_rate always in [0.0, 1.0]
- [x] Latency aggregates never go negative

### Coverage Status: **COMPLETE**

---

## 7. RESET PIPELINE (Hard & Soft)

### Feature Description
Experience and factory reset capabilities. Soft reset preserves knowledge, hard reset clears everything.

### Implementation
| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| Reset engine | `anna_common/src/experience_reset.rs` | `reset_experience_default()`, `reset_factory_default()` |
| Reset paths | `anna_common/src/experience_reset.rs` | `ExperiencePaths`, `ExperienceSnapshot` |
| Natural language | `anna_common/src/brain_fast.rs` | `fast_reset_experience_confirm()`, `fast_reset_factory_confirm()` |

### Test Coverage
| Test File | Test Name | What It Verifies |
|-----------|-----------|------------------|
| `annad/tests/integrity_suite.rs` | `test_inv_reset_*` | INV-RESET-001 through INV-RESET-005 |
| `anna_common/src/experience_reset.rs` | `test_soft_reset_*` | Soft reset behavior |
| `anna_common/src/experience_reset.rs` | `test_hard_reset_*` | Hard reset behavior |
| `anna_common/src/experience_reset.rs` | `test_snapshot_*` | Snapshot capture |

### Pass Criteria
- [x] Hard reset: Clears XP, telemetry, stats, knowledge
- [x] Hard reset: Leaves configuration and binaries intact
- [x] Soft reset: Clears XP and stats, preserves knowledge and recipes
- [x] "factory reset" → hard reset (with confirmation)
- [x] "soft reset" → soft reset (with confirmation)

### Coverage Status: **COMPLETE**

---

## 8. SNOW LEOPARD BENCHMARK

### Feature Description
6-phase benchmark suite measuring real performance, learning curve, and reliability. Used as regression gate.

### Implementation
| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| Benchmark engine | `anna_common/src/bench_snow_leopard.rs` | `SnowLeopardBenchmark`, `run_benchmark()` |
| Benchmark config | `anna_common/src/bench_snow_leopard.rs` | `SnowLeopardConfig`, `BenchmarkMode` |
| Phases | `anna_common/src/bench_snow_leopard.rs` | `PhaseId`, `PhaseResult` |
| History | `anna_common/src/bench_snow_leopard.rs` | `BenchmarkHistoryEntry` |

### Test Coverage
| Test File | Test Name | What It Verifies |
|-----------|-----------|------------------|
| `annad/tests/benchmark_snow_leopard.rs` | 15 tests | Full/Quick/Smoke modes |
| `annad/tests/integrity_suite.rs` | `test_inv_bench_*` | INV-BENCH-001 through INV-BENCH-005 |
| `anna_common/src/bench_snow_leopard.rs` | `test_benchmark_mode_parsing` | Mode detection |
| `anna_common/src/bench_snow_leopard.rs` | `test_phase_ids` | All 6 phases |
| `anna_common/src/bench_snow_leopard.rs` | `test_compare_benchmarks_*` | Delta analysis |

### Pass Criteria
- [x] Max latency thresholds per phase enforced
- [x] Phase 2+ latency <= Phase 1 for canonical questions
- [x] Brain/Recipe proportion increases across phases
- [x] Reliability does not drop between phases
- [x] Fails if latency envelope exceeded

### Coverage Status: **NEEDS HARDENING** (Task 4)

---

## 9. AUTOPROVISION & MODEL SELECTION

### Feature Description
Automatic LLM model selection based on hardware tier. Provisions best available models for Junior and Senior.

### Implementation
| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| LLM provision | `anna_common/src/llm_provision.rs` | `LlmSelection`, `HardwareTier` |
| Model benchmark | `anna_common/src/llm_provision.rs` | `ModelBenchmark`, `benchmark_model()` |
| Auto-tune | `anna_common/src/auto_tune.rs` | `auto_tune_from_benchmark()` |

### Test Coverage
| Test File | Test Name | What It Verifies |
|-----------|-----------|------------------|
| `annad/tests/integrity_suite.rs` | `test_inv_provision_*` | INV-PROVISION-001 through INV-PROVISION-005 |
| `anna_common/src/llm_provision.rs` | 13 inline tests | Scoring, selection, fallback |
| `anna_common/src/auto_tune.rs` | 6 inline tests | Tuning decisions |

### Pass Criteria
- [x] Autoprovision chooses expected models for hardware tier
- [x] Status shows "not yet run" when untouched
- [x] Status shows proper models and scores once run
- [x] Fallback model used when preferred unavailable

### Coverage Status: **COMPLETE**

---

## 10. DEBUG MODE TOGGLE

### Feature Description
Natural language debug mode control. Enables/disables detailed debug output in status and streaming.

### Implementation
| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| Debug state | `anna_common/src/debug_state.rs` | `DebugState`, `debug_set_enabled()` |
| Fast handlers | `anna_common/src/brain_fast.rs` | `fast_debug_enable()`, `fast_debug_disable()`, `fast_debug_status()` |

### Test Coverage
| Test File | Test Name | What It Verifies |
|-----------|-----------|------------------|
| `annad/tests/integrity_suite.rs` | `test_inv_debug_*` | INV-DEBUG-001 through INV-DEBUG-003 |
| `anna_common/src/debug_state.rs` | `test_debug_toggle` | Enable/disable cycle |
| `anna_common/src/brain_fast.rs` | `test_debug_enable` | Natural language enable |
| `anna_common/src/brain_fast.rs` | `test_debug_disable` | Natural language disable |

### Pass Criteria
- [x] "enable debug" / "disable debug" / "turn debug on/off" work
- [x] Debug section appears in status only when enabled
- [x] Live debug streaming respects the flag
- [x] Debug state persists across restarts

### Coverage Status: **COMPLETE**

---

## 11. PERMISSIONS & PERSISTENCE

### Feature Description
File-based persistence for XP store, telemetry, and knowledge. Safe file operations with proper permissions.

### Implementation
| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| XP store | `anna_common/src/xp_track.rs` | File persistence in `~/.local/share/anna/` |
| Telemetry store | `anna_common/src/telemetry.rs` | Append-only log file |
| Knowledge store | `anna_common/src/knowledge/` | SQLite database |
| Permissions | `anna_common/src/permissions.rs` | Permission checking |

### Test Coverage
| Test File | Test Name | What It Verifies |
|-----------|-----------|------------------|
| `annad/tests/integrity_suite.rs` | `test_inv_persist_*` | Persistence invariants |
| `anna_common/src/xp_track.rs` | `test_persistence_*` | XP save/load |
| `anna_common/src/telemetry.rs` | `test_file_operations_*` | Safe file handling |

### Pass Criteria
- [x] XP persists correctly across restarts
- [x] Telemetry log append-only
- [x] Knowledge store survives restart
- [x] Corrupted files handled gracefully

### Coverage Status: **NEEDS VERIFICATION** (Task 2)

---

## 12. DEGRADATION BEHAVIOR

### Feature Description
Graceful degradation when LLM or probes fail. Always produces an answer within budget, clearly indicating degraded mode.

### Implementation
| Component | File | Key Types/Functions |
|-----------|------|---------------------|
| Degraded answer | `anna_common/src/perf_timing.rs` | `DegradedAnswer`, `DegradationReason` |
| Fallback | `anna_common/src/brain_fast.rs` | `create_fallback_answer()` |
| Emergency | `anna_common/src/perf_timing.rs` | `DegradedAnswer::emergency()` |

### Test Coverage
| Test File | Test Name | What It Verifies |
|-----------|-----------|------------------|
| `annad/tests/integrity_suite.rs` | `test_inv_perf_016_degraded_answer_fast` | <2s generation |
| `annad/tests/integrity_suite.rs` | `test_inv_perf_017_degradation_reliability_levels` | Correct reliability |
| `annad/tests/integrity_suite.rs` | `test_inv_perf_018_emergency_instant` | Emergency path |
| `annad/tests/integrity_suite.rs` | `test_inv_perf_019_partial_evidence_included` | Evidence preservation |

### Pass Criteria
- [x] User always gets response within global budget
- [x] Response clearly indicates degraded mode
- [x] No panics or hangs
- [x] Telemetry records timeout/degraded events
- [ ] Degraded modes do not permanently alter configuration (needs test)
- [ ] Recovery when environment improves (needs test)

### Coverage Status: **NEEDS HARNESS** (Task 5)

---

## VERIFICATION CHECKLIST

For each feature above, the following MUST be verified:

| Check | Brain | Router | LLM | Budget | XP | Telemetry | Reset | Bench | Provision | Debug | Persist | Degrade |
|-------|-------|--------|-----|--------|-----|-----------|-------|-------|-----------|-------|---------|---------|
| Positive case | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Negative case | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ◐ |
| Timeout handling | ✓ | N/A | ✓ | ✓ | N/A | N/A | N/A | ✓ | ✓ | N/A | N/A | ✓ |
| Bad input | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ◐ |
| State reset | N/A | N/A | N/A | N/A | ✓ | ✓ | ✓ | ✓ | N/A | ✓ | ✓ | ◐ |

Legend: ✓ = Tested, ◐ = Partial, ✗ = Missing

---

## TEST EXECUTION COMMANDS

```bash
# Full test suite
cargo test --workspace

# Feature-specific tests
cargo test --test integrity_suite brain          # Brain tests
cargo test --test integrity_suite recipe         # Recipe tests
cargo test --test integrity_suite llm            # LLM tests
cargo test --test integrity_suite perf           # Performance tests
cargo test --test integrity_suite xp             # XP tests
cargo test --test integrity_suite reset          # Reset tests
cargo test --test integrity_suite provision      # Provision tests

# Baseline performance tests
cargo test --test baseline_tests

# Acceptance scenarios
cargo test --test acceptance_scenarios

# Benchmark (slow, requires --ignored)
cargo test --test benchmark_snow_leopard -- --ignored
```

---

## REVISION HISTORY

| Version | Date | Changes |
|---------|------|---------|
| 3.5.0 | 2025-11-30 | Initial FVM creation |
