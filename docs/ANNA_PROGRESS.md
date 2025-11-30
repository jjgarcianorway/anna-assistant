# Anna Progress

## Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` done

---

## v0.10.0 - LLM-A/LLM-B supervised audit loop

[x] Two-level LLM orchestration (Junior/Senior)
[x] Basic evidence discipline

## v0.11.0 - Knowledge store, event-driven learning

[x] SQLite-backed knowledge store
[x] Fact learning from probes

## v0.12.0 - Iteration-aware prompts, fix_and_accept

[x] Senior can fix answers inline
[x] Iteration-aware context

## v0.13.0 - Strict evidence discipline

[x] No guessing - only measured facts
[x] Confidence scoring

## v0.14.0 - Aligned to reality with 6 real probes

[x] CPU probe (lscpu)
[x] Memory probe (cat /proc/meminfo)
[x] Storage probe (lsblk)
[x] Network probe (ip)
[x] Process probe (ps)
[x] System probe (os-release)

## v0.15.0 - Research Loop Engine

[x] Command whitelist implementation
[x] Risk classification (low/medium/high)
[x] User confirmation for high-risk commands

## v0.16.x - Enhanced status and debug output

[x] Human-readable uptime display
[x] Probe names in output
[x] Detailed health information
[x] Debug trace output with [JUNIOR] [SENIOR] labels

## v0.17.0 - Senior answer synthesis

[x] Use Senior's synthesized answer instead of Junior's draft

## v0.18.0 - Step-by-step orchestration

[x] One action per Junior iteration
[x] Clear Junior/Senior role separation
[x] Max 6 iterations per question

## v0.19.0 - Subproblem decomposition

[x] Break complex questions into subproblems
[x] Fact-aware planning
[x] Senior as mentor with feedback

## v0.20.0 - Background telemetry

[x] Warm-up learning on startup
[x] Fact store integration
[x] Background telemetry collection

## v0.21.0 - Hybrid answer pipeline

[x] Fast-first from cached facts
[x] Selective probing only when needed
[x] No iteration loops for cached answers

## v0.22.0 - Fact Brain & Question Decomposition

[x] TTL-based fact expiration
[x] Validated facts with confidence
[x] Semantic linking between facts
[x] Question decomposition strategy

## v0.23.0 - System Brain, User Brain & Idle Learning

[x] Separate system and user knowledge stores
[x] User identity tracking
[x] Idle learning during low CPU periods
[x] Safe file scanning with whitelist

## v0.24.0 - App Awareness, Stats & Faster Answers

[x] Window manager detection
[x] Desktop environment awareness
[x] Default apps registry (MIME types)
[x] Stats engine for telemetry
[x] Answer caching for speed

## v0.25.0 - Relevance First, Usage Tracking, Session Awareness

[x] Relevance engine with scoring
[x] Recency/frequency-based ranking
[x] Usage tracking and pattern detection
[x] Session awareness for active apps
[x] Ambiguity resolver with remembered resolutions

## v0.26.0 - Auto-update Reliability, Self-Healing, Logging

[x] Auto-update manager with retry logic
[x] SHA256 checksum verification
[x] Daemon watchdog for self-healing
[x] Health check system with targets
[x] Rate-limited restart logic
[x] Healing event tracing
[x] Structured tracing for debugging

**Tests**: 75 passed, 0 failed (annad + annactl + anna_common)
**Release**: https://github.com/jjgarcianorway/anna-assistant/releases/tag/v0.26.0

---

## v0.27.0 - SSH-Friendly Spinner

[x] TTY detection for spinner (skip animation for piped output)
[x] Slower spinner update interval (80ms → 200ms)
[x] Non-TTY mode prints static messages without escape codes

**Release**: https://github.com/jjgarcianorway/anna-assistant/releases/tag/v0.27.0

## v0.27.1 - SSH Stability Hardening

[x] Even slower spinner (200ms → 500ms, 6x slower than original)
[x] ANNA_NO_SPINNER environment variable to completely disable animation
[x] Batch-run friendly for test scripts

**Usage for batch runs**:
```bash
ANNA_NO_SPINNER=1 ./test_script.sh
```

**Release**: https://github.com/jjgarcianorway/anna-assistant/releases/tag/v0.27.1

---

## v0.28.0 - Cross-Device Auto-Update Fix & ASCII Aesthetic

[x] Fixed EXDEV error 18 (cross-device link) in auto-update
[x] Copy fallback when /tmp and /usr/local/bin on different filesystems
[x] Replaced all emojis with ASCII indicators for hacker aesthetic
[x] Log output now uses [*], [+], [-], [!], [>], [#] instead of emojis

**Tests**: 75 passed, 0 failed
**Release**: https://github.com/jjgarcianorway/anna-assistant/releases/tag/v0.28.0

---

## v0.70.0 - Evidence Oracle - Structured LLM Protocol, Difficulty Routing

[x] 5-Type question classification
[x] Difficulty-based routing (Easy/Normal/Hard)
[x] Structured LLM protocol
[x] Confidence gating and stats tracking

---

## v0.71.0 - Performance Patch - Fast Path, Stats Fix, Timeout Fix

[x] Fixed debug stream header (was hardcoded v0.43.0, now uses package version)
[x] Increased question display from 70 to 512 characters
[x] Added fast path for simple hardware questions (CPU, RAM, disk)
    - Bypasses LLM orchestration entirely for ~1s responses
    - Directly parses probe output instead of using Junior/Senior loop
[x] Added fast path for annad logs (logs.annad probe) and system updates (updates.pending probe)
[x] Added fast path for self-diagnosis using real health checks
    - Uses self_health::run_all_probes() instead of LLM guessing
    - Provides actual component status, not hallucinated responses
[x] Reduced LLM timeout from 120s to 30s for better responsiveness
[x] Fixed stats persistence - now fetches from daemon API instead of local file
    - Solves permission issues when daemon runs as different user
    - Stats now correctly update after each answer
[x] New probes: logs.annad, updates.pending
[x] 90 tests passing

**Key Changes**:
- CPU/RAM questions now complete in <1 second (was ~90 seconds)
- Logs/Updates questions now complete in <2 seconds (was failing or slow)
- Self-diagnosis now runs real health checks (was LLM guessing)
- Stats/XP actually increment after answering questions
- Debug stream shows correct version number

**Tests**: 90 passed, 0 failed

---

## v0.72.0 - Answer & Debug Transcript Patch

[x] Clear answer output block with Anna header, Evidence, Reliability sections
[x] Upgraded debug stream with detailed transcript (Junior plan, Anna probe, Senior verdict)
[x] Stats formatting: percentages, human-friendly latency (ms/s)
[x] Live stats update after each answer (streaming route fix)
[x] First-run detection with marker file
[x] Protocol version updated to 0.72.0
[x] 90 tests passing

**Key Changes**:
- Answer output now shows clear blocks: header, evidence with commands, reliability with %
- Debug stream uses conversational labels: [JUNIOR: PLAN], [ANNA: PROBE], [SENIOR: VERDICT]
- Stats show "92%" instead of "0.92", latency shows "1.2s" instead of "1200ms"
- Streaming answers now update stats (was missing in v0.71.0)
- Marker file /var/lib/anna/.initialized tracks first-run state

**Tests**: 90 passed, 0 failed

---

## v0.73.0 - Rubber-Stamping Fix

[x] Fix Senior parse failures that silently approved with 70/70/70
[x] Reject 0-score answers instead of delivering unverified content
[x] Require probe evidence before delivering answers

---

## v0.74.0 - Structured Trace Pipeline

[x] JSON traces with correlation IDs
[x] Debug output system
[x] Canonical questions

---

## v0.76.0-v0.80.0 - LLM Prompt Optimizations

[x] Minimal Junior Planner prompt (v0.76.0)
[x] Dialog View events for streaming (v0.76.2, v0.77.0)
[x] Minimal Senior Auditor prompt (v0.78.0)
[x] CPU semantics and scoring fix (v0.79.0)
[x] Razorback Fast Path <5s response (v0.80.0)

---

## v0.81.0-v0.82.0 - QA Harness

[x] Structured answer format (headline/details/evidence)
[x] QaOutput JSON schema
[x] Benchmark script with canonical questions
[x] Acceptance thresholds

---

## v0.83.0 - Performance Focus

[x] Explicit time budgets (15s target)
[x] Compact Junior/Senior prompts (<600 chars)

---

## v0.84.0 - Hard Test Harness

[x] TEST_PLAN_v0.84.md formal test plan
[x] Benchmark script with timing/JSON
[x] FailureCause classification
[x] XpEvent types

---

## v0.85.0 - Architecture Optimisation

[x] Brain layer (COMMAND_LIBRARY, OUTPUT_PARSERS, FAILURE_MEMORY)
[x] Performance budget enforcement (12s total)
[x] Ultra-compact prompts (<2KB Junior, <4KB Senior)
[x] LLM output validation
[x] Extended XP events

---

## v0.85.1 - XP Log Command

[x] `annactl xp-log [N]` command (removed in v0.88.0 per CLI surface policy)
[x] 24h XP metrics in status command
[x] JSONL-based XP event storage

**Tests**: 740 passed, 0 failed
**Release**: https://github.com/jjgarcianorway/anna-assistant/releases/tag/v0.85.1

---

## v0.86.0 - XP Reinforcement

[x] Anna/Junior/Senior XP tracking
[x] Trust levels and ranks
[x] Behaviour bias based on XP performance

**Release**: https://github.com/jjgarcianorway/anna-assistant/releases/tag/v0.86.0

---

## v0.87.0 - Latency Cuts & Brain Fast Path

[x] Brain fast path for simple questions (<3s response)
[x] Hard fallback when LLM fails
[x] Always visible answer block
[x] Anna XP events for self_solve

**Key Changes**:
- CPU/RAM questions complete in <9ms via Brain fast path
- 99% reliability (Green) for simple hardware questions
- No LLM required for cached knowledge

**Release**: https://github.com/jjgarcianorway/anna-assistant/releases/tag/v0.87.0

---

## v0.88.0 - Dynamic Probe Catalog & XP Wiring

[x] Dynamic probe catalog - single source of truth for probe lists
[x] probe_ids_string() and probe_ids() methods in ProbeCatalog
[x] Removed hardcoded probe lists from LLM prompts
[x] Wire Junior/Senior XP events via process_llm_xp_events()
[x] XP events now append to XpLog for 24-hour metrics
[x] Removed xp-log command (CLI surface: help, version, status only)
[x] Export XpEvent and XpEventType from anna_common root

**Fixes**:
- Junior no longer loops on logs.annad (probes now passed dynamically)
- 24-hour XP metrics now correctly display events from LLM answers
- XP events properly recorded for Junior/Senior verdicts

**Tests**: 736 passed, 0 failed
**Release**: https://github.com/jjgarcianorway/anna-assistant/releases/tag/v0.88.0

---

## v0.89.0 - Conversational Debug Mode

[x] Persistent debug mode toggle via natural language
[x] DebugState struct with enabled, last_changed_at, last_changed_reason
[x] Brain fast path for debug intents (no LLM required)
[x] DebugIntent enum with Enable/Disable/Status/None variants
[x] annactl streams live debug when persistent debug mode is ON
[x] DEBUG MODE section in status command (only shown when enabled)
[x] Pattern matching fixes for deactivate/enabled containing substrings

**Key Features**:
- "enable debug mode" / "disable debug mode" / "is debug mode enabled?" handled instantly via Brain
- Persistent state survives daemon restarts and reboots
- Stored in /var/lib/anna/knowledge/stats/debug_state.json
- No environment variable or config file edit required
- 99% reliability (Green) for debug toggle operations

**Tests**: 771 passed, 0 failed
**Release**: https://github.com/jjgarcianorway/anna-assistant/releases/tag/v0.89.0

---

## v2.1.0 "Permissions Fix" - XP/Telemetry Persistence & Reset Pipeline

**Theme**: Bulletproof persistence for XP and telemetry, proper permissions, comprehensive reset system.

### Permissions Health Check (v2.1.0)
[x] New `permissions.rs` module with health check and auto-fix
[x] `PermissionsHealthCheck::run()` - checks all Anna directories
[x] `auto_fix_permissions()` - attempts to fix permission issues
[x] `atomic_write()` - safe persistence with temp file + rename
[x] `safe_append()` - safe append for telemetry JSONL
[x] `ensure_writable_dir()` - creates directories if missing

### Installer Permissions Fix (v2.1.0)
[x] Installer v3.1.0 - detects runtime user (SUDO_USER or root)
[x] Creates all directories with correct ownership
[x] `/var/lib/anna/` owned by runtime user (for annactl XP tracking)
[x] `/var/log/anna/` owned by runtime user (for telemetry)
[x] No manual chown ever needed

### Daemon Startup Health Check (v2.1.0)
[x] annad runs `PermissionsHealthCheck::run()` at startup
[x] Auto-fix attempted if issues detected
[x] Warning logged with manual fix instructions if auto-fix fails

### XP Persistence Fix (v2.1.0)
[x] `XpStore::save()` now uses atomic write (temp file + rename)
[x] Auto-creates XP directory if missing
[x] Sets 777 permissions on new directories

### Reset Pipeline (v2.1.0)
[x] True Hard Reset (factory reset) - wipes XP, telemetry, stats, knowledge
[x] True Soft Reset (experience reset) - preserves knowledge
[x] Natural language triggers: "hard reset", "soft reset", "reset everything"
[x] Natural language triggers: "clear memory", "factory reset anna"
[x] Confirmation required for both reset types

### Autoprovision Display Fix (v2.1.0)
[x] `annactl status` shows "not yet run" when autoprovision hasn't run
[x] Helpful message: "Run annactl and ask a question to trigger"
[x] Scores show "not benchmarked" instead of 0.00 when unprovisioned

### Tests (v2.1.0)
[x] 8 permissions tests (atomic write, safe append, health check)
[x] 13 experience_reset tests (hard reset, soft reset, confirmation)
[x] Reset trigger tests for new natural language patterns
[x] All workspace tests passing

**Key Files**:
- crates/anna_common/src/permissions.rs - New permissions module
- crates/anna_common/src/xp_track.rs - Atomic save
- crates/anna_common/src/brain_fast.rs - Reset triggers
- crates/annad/src/main.rs - Startup health check
- scripts/install.sh v3.1.0 - Permission fixes

---

## v3.3.0 "Integrity & Verification" - Comprehensive Subsystem Verification

**Theme**: Verify every subsystem works exactly as intended. Feature Integrity Matrix, invariant guards, 56+ tests.

### Feature Integrity Matrix (v3.3.0)
[x] docs/FEATURE_INTEGRITY_MATRIX.md - contract of correctness for all 13 subsystems
[x] Defined invariants for: Brain, Recipe, Probe, LLM, XP, Telemetry, Answer, Reset, Benchmark, Autoprovision
[x] Each subsystem has: intended behavior, required invariants, latency/reliability boundaries

### Integrity Test Suite (v3.3.0)
[x] crates/annad/tests/integrity_suite.rs - 56 comprehensive tests
[x] Brain integrity tests (INV-BRAIN-001 to INV-BRAIN-005)
[x] Recipe integrity tests (INV-RECIPE-001, INV-RECIPE-002, INV-RECIPE-007)
[x] XP integrity tests (INV-XP-002, INV-XP-003)
[x] Answer integrity tests (INV-ANS-001 to INV-ANS-004)
[x] Autoprovision integrity tests (INV-PROV-001 to INV-PROV-006)
[x] Reset system integrity tests (INV-RESET-001 to INV-RESET-007)
[x] Benchmark integrity tests (INV-BENCH-001 to INV-BENCH-007)
[x] Debug parity tests, regression detection tests

### Centralized Invariant Guards (v3.3.0)
[x] crates/anna_common/src/invariants.rs - canonical guard functions
[x] guard_reliability() - clamps to 0.0-1.0, logs violations
[x] guard_answer_text() - ensures non-empty text
[x] guard_trust() - clamps trust to 0.0-1.0
[x] guard_level() - clamps level to 1-99
[x] guard_verdict() - validates Senior verdicts
[x] validate_answer() - composite validation

### Validation Coverage (v3.3.0)
[x] Debug mode parity - answers identical with/without debug (±5% latency)
[x] Recipe learning - extraction threshold >= 0.85, match threshold >= 0.70
[x] Autoprovision - hardware tier detection, model viability checks
[x] Reset system - soft/hard reset separation, XP baseline restoration
[x] Benchmark system - uses same pipeline as runtime, question sets validation

### Tests (v3.3.0)
[x] 56 integrity tests in integrity_suite.rs
[x] All workspace tests passing
[x] Release build successful

**Key Files**:
- docs/FEATURE_INTEGRITY_MATRIX.md - Contract of correctness
- crates/annad/tests/integrity_suite.rs - 56 integrity tests
- crates/anna_common/src/invariants.rs - Centralized guard functions

---

## v3.1.0 "Pipeline Purity" - Remove Legacy Orchestration from annactl

**Theme**: Make annactl a pure thin client. All LLM calls route through daemon.

### Legacy Code Removal (v3.1.0)
[x] Removed crates/annactl/src/orchestrator.rs (902 lines) - legacy LLM-A/B pipeline
[x] Removed crates/annactl/src/llm_client.rs (134 lines) - duplicate Ollama client
[x] Total: 1036 lines of legacy code removed from annactl

### Architecture Purity (v3.1.0)
[x] annactl is now a THIN CLIENT - no LLM calls happen locally
[x] All questions route through daemon API (/v1/answer)
[x] Brain fast path handles simple questions locally (no LLM)
[x] Help/version are now static (no LLM calls needed)

### Documentation (v3.1.0)
[x] Updated annactl/main.rs module docs with v3.1.0 architecture
[x] Static help display in run_help_via_llm() - cleaner, faster
[x] Updated README.md version history
[x] Updated ANNA_PROGRESS.md

### Tests (v3.1.0)
[x] All workspace tests passing (same test count - no functionality removed)
[x] Build successful

**Key Files Modified**:
- crates/annactl/src/main.rs - Removed orchestrator/llm_client imports, static help

**Key Files Deleted**:
- crates/annactl/src/orchestrator.rs (902 lines)
- crates/annactl/src/llm_client.rs (134 lines)

---

## v3.0.0 "Brain First" - Unified Pipeline, Recipe Learning, Legacy Cleanup

**Theme**: Single unified pipeline. Brain → Recipe → Junior → Senior. No legacy code paths.

### Unified Pipeline (v3.0.0)
[x] Single orchestration entry point: UnifiedEngine in engine_v90.rs
[x] Removed legacy engines: engine.rs, engine_v18.rs, engine_v19.rs, engine_v80.rs, research_engine.rs
[x] Removed legacy LLM clients: llm_client_v18.rs, llm_client_v19.rs
[x] Answer origin enum: Brain, Recipe, Junior, Senior
[x] Flow: Brain fast path → Recipe match → Junior plan/draft → Senior audit

### Recipe Learning Integration (v3.0.0)
[x] RecipeStore integrated into UnifiedEngine
[x] Recipe check after Brain fast path, before Junior
[x] Recipe extraction after successful Senior answers (reliability >=85%)
[x] Recipe answer building with origin tracking
[x] classify_question() for simple question type detection

### Architecture Cleanup (v3.0.0)
[x] Removed 7 legacy engine/client files (~5000 lines)
[x] Updated orchestrator/mod.rs exports
[x] Single pipeline documentation in engine_v90.rs header
[x] Updated README.md with v3.0.0 architecture diagram

### LLM Provisioning (v3.0.0)
[x] HardwareTier enum (Minimal, Basic, Standard, Power)
[x] Hardware detection (CPU cores, RAM GB, NVIDIA GPU)
[x] Router model candidates (qwen2.5:0.5b, qwen2.5:1.5b, phi3:mini, etc.)
[x] Tier-specific model selection

### Tests (v3.0.0)
[x] All orchestration tests passing (24 tests)
[x] Recipe system tests (10 tests)
[x] router_llm tests (13 tests)
[x] Full workspace: 827+ tests passing

**Key Files**:
- crates/annad/src/orchestrator/mod.rs - Unified orchestration module
- crates/annad/src/orchestrator/engine_v90.rs - Canonical engine with Recipe integration
- crates/anna_common/src/recipe.rs - Recipe system
- crates/anna_common/src/router_llm.rs - QuestionType classification

---

## v2.3.0 "Runtime Snow Leopard" - Benchmark Triggers & Latency Guardrails

**Theme**: Make Anna feel like a serious tool - runtime benchmarks, no hanging, no empty answers.

### Runtime Snow Leopard Benchmark (v2.3.0)
[x] Enhanced benchmark triggers in Brain fast path
[x] Natural language: "run the full snow leopard benchmark", "benchmark anna", "quick benchmark"
[x] Natural language: "full benchmark", "complete benchmark", just "benchmark"
[x] Runtime benchmark execution with phase-by-phase progress output
[x] Error handling for benchmark failures (JSON parse, timeout)

### Global Runtime Latency Guardrail (v2.3.0)
[x] Hard 10-second wall-clock limit (reduced from 30s)
[x] LLM budgets tuned for small autoprovision models:
    - Junior: 4s (was 15s)
    - Senior: 5s (was 15s)
    - Global soft limit: 8s (was 20s)
    - Global hard limit: 10s (was 30s)

### No More Silent Red 0.00 Answers (v2.3.0)
[x] Timeout fallback: partial answer with 40% reliability (was 0%)
[x] Error fallback: error explanation with 30% reliability (was 0%)
[x] Refusal fallback: safety explanation with 50% reliability (was empty)
[x] All fallbacks include human-readable context

### Daily Check-In Enhancements (v2.3.0)
[x] First Light status in Daily Check-In (last result or "pending")
[x] Snow Leopard benchmark status in Daily Check-In
[x] "EVALUATION TOOLS" section with First Light and Snow Leopard status
[x] `get_first_light_status()` helper function

### Tests (v2.3.0)
[x] `test_benchmark_triggers_v230` - enhanced trigger patterns
[x] `test_time_budgets_v230` - reduced timeout values
[x] All workspace tests passing

**Key Files**:
- crates/anna_common/src/brain_fast.rs - Enhanced triggers, reduced budgets
- crates/annad/src/orchestrator/engine_v90.rs - 10s timeout, improved fallbacks
- crates/anna_common/src/first_light.rs - Daily Check-In enhancements

---

## v2.2.0 "First Light" - Post-Reset Validation & Daily Check-In

**Theme**: Predictable, validated, measurable post-reset state. Ready for Snow Leopard benchmark cycles.

### First Light Self-Test (v2.2.0)
[x] New `first_light.rs` module with First Light framework
[x] `FIRST_LIGHT_QUESTIONS` - 5 canonical questions (CPU, RAM, Disk, Health, LLM)
[x] `FirstLightQuestion` struct - tracks success/failure/reliability/latency/xp
[x] `FirstLightResult` struct - aggregated test results with stats
[x] `FirstLightResult::new()` - calculates all_passed, avg_reliability, avg_latency, total_xp
[x] `FirstLightResult::format_display()` - TRUE COLOR formatted output
[x] `FirstLightQuestion::success/failure()` - factory methods for results

### XP/Telemetry Sanity Validation (v2.2.0)
[x] `SanityCheckResult` struct - comprehensive validation results
[x] `run_sanity_checks()` - validates XP file, Telemetry file, Stats directory
[x] `SanityCheckResult::format_display()` - colored status output
[x] Checks: file existence, writability, JSON parsability, reasonable values

### Daily Check-In Command (v2.2.0)
[x] `DailyCheckIn` struct - generates daily status report
[x] `is_daily_checkin_question()` - pattern matching for triggers
[x] Natural language triggers: "daily check in", "show today's check in"
[x] Natural language triggers: "how are you today", "daily status"
[x] `DailyCheckIn::generate()` - creates check-in from current state
[x] `DailyCheckIn::format_display()` - TRUE COLOR formatted output
[x] `FastQuestionType::DailyCheckIn` - Brain fast path classification
[x] `fast_daily_checkin()` - handles check-in without LLM

### Reset Confirmation UX (v2.2.0)
[x] Improved confirmation prompts with clearer instructions
[x] Soft reset accepts: "yes", "y", "confirm", "ok", "yes, soft reset"
[x] Hard reset accepts: "I UNDERSTAND AND CONFIRM FACTORY RESET", "yes, hard reset"
[x] Confirmation messages show exact required input

### Tests (v2.2.0)
[x] `test_first_light_question_success` - success factory method
[x] `test_first_light_question_failure` - failure factory method
[x] `test_first_light_result_stats` - aggregation calculations
[x] `test_sanity_checks_empty_dir` - sanity on missing files
[x] `test_daily_checkin_triggers` - pattern matching
[x] `test_daily_checkin_generate` - generation and display
[x] All workspace tests passing (827+)

**Key Files**:
- crates/anna_common/src/first_light.rs - New First Light module
- crates/anna_common/src/brain_fast.rs - DailyCheckIn classification, reset UX
- crates/anna_common/src/lib.rs - first_light export

---

## v2.0.0 "Autoprovision" - Fully Self-Provisioning LLM Models

**Theme**: Anna now manages her own LLM models - detects, installs, benchmarks, and switches automatically.

### Runtime Autoprovision
[x] Automatic Ollama detection and installation on first startup
[x] `is_ollama_installed()`, `is_ollama_running()`, `install_ollama()`, `start_ollama_service()`
[x] `ensure_ollama_ready()` - handles full Ollama setup
[x] `run_full_autoprovision<F>(on_progress)` - main entry point with progress callback
[x] `needs_autoprovision()` - quick check if provisioning is needed
[x] Wired into annad startup at `is_first_run()` check

### Model Benchmarking at Startup
[x] Dynamic model candidate lists (8 Junior, 6 Senior candidates)
[x] JSON compliance scoring with structured test prompts
[x] Reasoning/logic scoring for Senior models
[x] Determinism scoring across 3 runs
[x] Latency thresholds (Junior: 8s max, Senior: 15s max)
[x] Composite score calculation for role suitability

### Automatic Model Selection
[x] `select_best_junior()` - picks fastest JSON-compliant model
[x] `select_best_senior()` - picks best reasoning model
[x] `AutoprovisionResult` struct with installation/benchmark details
[x] Selection persisted to `/var/lib/anna/llm/selection.json`
[x] Benchmark results saved to `/var/lib/anna/llm/benchmarks/`

### Runtime Model Switching
[x] `ModelSwitchResult` struct for tracking switches
[x] `try_switch_junior_model()` - switch to faster model on timeouts
[x] `try_switch_senior_model()` - switch on poor performance
[x] `handle_model_failure()` - main entry for runtime adaptation
[x] Automatic switch after 3 consecutive failures
[x] Suggestions added to status output

### Integration
[x] Installer v3.0.0 - mentions autoprovision in post-install message
[x] `annactl status` shows LLM AUTOPROVISION section with model scores
[x] Junior fallback timeout (2 seconds) integrated

### Tests
[x] 15 llm_provision tests (scoring, selection, fallback, switching)
[x] All workspace tests passing

**Key Files**:
- crates/anna_common/src/llm_provision.rs - Full autoprovision module
- crates/annad/src/main.rs - Startup autoprovision hook
- scripts/install.sh v3.0.0 - Updated installer

---

## v1.1.0 - Adaptive LLM Provisioning & Skill Router

**Theme**: Self-provisioning LLM models, deterministic skill routing, no LLM for system queries.

### Adaptive LLM Provisioning
[x] Model candidate lists for Junior (9 models) and Senior (7 models)
[x] Model benchmarking with JSON compliance, latency, determinism scoring
[x] `select_best_junior/senior()` selection functions with criteria
[x] Automatic model installation via `ollama pull`
[x] Junior fallback timeout (2 seconds) - auto-fallback if slow
[x] `LlmSelection` struct for persisting model choices
[x] Integration with `annactl status` command

### Skill-Based Routing System
[x] Generic `SkillType` enum with 21 skill types
[x] Pattern-based classifier (no hardcoded phrases)
[x] `SkillAnswer` with strict invariants (never empty, reliability 0.1-1.0)
[x] `SkillContext` for time budgets and metadata

### Brain-First Deterministic Categories
[x] ALL system queries now skip LLM (Brain-only):
    - CPU, RAM, Disk, Uptime, Network, GPU, OS info
    - Service health, logs, updates
    - Debug enable/disable/status
[x] New skills: `GpuInfo`, `OsInfo`
[x] Fast handlers: `fast_gpu_answer()`, `fast_os_answer()`, `fast_network_answer()`

### Tests
[x] 13 llm_provision tests (scoring, selection, fallback policy)
[x] 34 skill_router tests (classification, brain-first validation)
[x] 10 skill_handlers tests (time budgets, failure policy)
[x] 813+ workspace tests passing

**Key Files**:
- crates/anna_common/src/llm_provision.rs - LLM autoprovision module
- crates/anna_common/src/skill_router.rs - Skill classification
- crates/anna_common/src/skill_handlers.rs - Skill execution

---

## v1.0.0 "Snow Leopard" - Stabilization Release

**Theme**: Robustness, predictability, testing. No new user features.

### Phase 1: Architecture Freeze
[x] Created docs/architecture.md - canonical reference for all code
[x] Documented Brain/Junior/Senior orchestration paths
[x] Defined latency targets and reliability thresholds

### Phase 2: Deterministic Tests
[x] Created LlmClient trait abstraction for testing
[x] Created FakeLlmClient with canned responses
[x] Created FakeProbeEngine for deterministic probe results
[x] Added orchestration tests for Brain, Junior+Senior paths

### Phase 3: XP Consolidation
[x] Single source of truth for XP events in xp_events.rs
[x] Fixed annactl status consistency

### Phase 4: Performance Baseline Tests
[x] Created baseline_tests.rs with latency assertions
[x] Brain path: <150ms latency, Green reliability
[x] Reliability scaling tests (Green/Yellow/Orange/Red)

### Phase 5: CLI Formatting Centralization
[x] Created ui_colors.rs as CANONICAL source for all colors
[x] Centralized reliability thresholds: GREEN >= 90%, YELLOW >= 70%, RED >= 50%
[x] Centralized actor colors (Anna, Junior, Senior, System)
[x] Updated progress_display.rs to use centralized colors

### Phase 6: Security Hardening
[x] Path traversal protection (`..` blocked)
[x] Null byte injection protection
[x] Parameter length limit (4KB max)
[x] Security test suite (29 command_whitelist tests)

**Tests**: 948 passed, 0 failed
**Key Files**:
- docs/architecture.md - Architecture reference
- crates/anna_common/src/ui_colors.rs - Canonical colors/thresholds
- crates/annad/tests/baseline_tests.rs - Performance baselines
- crates/annad/tests/orchestration_tests.rs - Deterministic tests

---

## v3.8.0 "Preflight QA" - Learning Contract & Speed Verification

**Theme**: Prove Anna learns correctly before public release. Comprehensive learning tests, speed verification, contract documentation.

### Learning Contract Documentation (v3.8.0)
[x] docs/ANNA_LEARNING_CONTRACT.md - Formal learning guarantees specification
[x] First Time Behavior section - correctness, honesty, speed guarantees
[x] Second Time Behavior section - learning mechanics, recipe extraction, speed improvement
[x] Reset Behavior section - soft reset (XP) vs hard reset (factory)
[x] Debug Mode section - output structure guarantees
[x] Reliability & Honesty Guarantees section - scoring rules, no fake Green
[x] Telemetry Consistency section - origin tracking, stats matching
[x] Implementation Invariants table - threshold values with code enforcement

### Learning Speed Test Harness (v3.8.0)
[x] crates/annad/tests/learning_speed_canonical.rs - 13 learning speed tests
[x] Round 1→2→3 pattern for canonical questions
[x] Brain fast path latency tests (<200ms target)
[x] No latency regression on repeat questions
[x] Recipe extraction and matching tests
[x] RecipeStore operations tests (add, find_match)
[x] Telemetry shows learning trend (Brain count increases)
[x] No silent failures test
[x] Percentage formatting (no raw floats) test
[x] Paraphrased intent mapping tests

### Snow Leopard Benchmark Learning Integration (v3.8.0)
[x] crates/annad/tests/benchmark_snow_leopard.rs - 8 new learning verification tests
[x] test_learning_phase_reports_brain_trend - Phase 6 learning verification
[x] test_benchmark_reports_origin_distribution - Brain vs LLM usage percentages
[x] test_no_silent_benchmark_failures - every question has answer, reliability, origin
[x] test_reliability_percentages_no_raw_floats - ASCII summary uses %
[x] test_warm_state_vs_hard_reset_comparison - Phase 1 vs Phase 2 comparison
[x] test_skill_summary_populated - skill tracking validation
[x] test_ux_consistency_rules - answer quality checks

### Learning Contract Test Suite (v3.8.0)
[x] crates/annad/tests/learning_contract_tests.rs - 22 contract validation tests
[x] Debug intent classification tests (enable/disable/status/none patterns)
[x] Debug state serialization tests
[x] Soft reset preserves knowledge test
[x] Hard reset clears knowledge test
[x] Reset confirmation strings test
[x] Experience snapshot capture test
[x] Telemetry tracks failures test
[x] Reliability bounds test (MIN_RECIPE_RELIABILITY, RECIPE_MATCH_THRESHOLD)
[x] Origin tracking consistency test
[x] Percentage formatting tests
[x] Brain handles hardware questions test
[x] Brain handles health questions test
[x] FastQuestionType classification test
[x] RecipeStore operations test
[x] Recipe matching threshold test
[x] Recipe reliability minimum test
[x] Learning contract compliance summary test

### Constraints (v3.8.0)
[x] NO new user-visible features
[x] NO new commands
[x] NO new RPCs
[x] ONLY: Tests, Fixes, Performance improvements, Cleanup

### Tests (v3.8.0)
[x] 13 learning speed canonical tests (all passing)
[x] 17 benchmark snow leopard tests (+8 new, all passing)
[x] 22 learning contract tests (all passing)
[x] All workspace tests passing (1299+)
[x] Zero compiler warnings

**Key Files**:
- docs/ANNA_LEARNING_CONTRACT.md - Formal learning guarantees
- crates/annad/tests/learning_speed_canonical.rs - Learning speed tests
- crates/annad/tests/benchmark_snow_leopard.rs - Benchmark learning integration
- crates/annad/tests/learning_contract_tests.rs - Contract validation tests

---

## v3.7.0 "Reliability Gauntlet" - System Acceptance & Production Validation

**Theme**: Comprehensive test and validation layer. NO new features, ONLY fixes, tests, cleanup.

### System Acceptance Test Harness (v3.7.0)
[x] crates/annad/tests/system_acceptance.rs - 19 comprehensive acceptance tests
[x] "Day in the Life" scenario - full journey from fresh install to power user
[x] Fresh install state validation (XP baseline, trust levels)
[x] First Light self-test execution and validation
[x] Canonical questions coverage (CPU, RAM, disk, health, LLM)
[x] Learning improvement verification (repeated questions faster)
[x] Natural language intent mapping tests
[x] Soft reset behavior validation (preserves knowledge, resets XP)
[x] Hard reset behavior validation (factory reset to fresh state)
[x] Debug mode output structure verification
[x] Percentage formatting validation (no raw floats in output)
[x] Stats/telemetry consistency verification

### Test Plan Documentation (v3.7.0)
[x] docs/ANNA_TEST_PLAN.md - Comprehensive test plan document
[x] 11 test scenarios with detailed assertions
[x] "Day in the Life" journey definition
[x] Performance assertions and acceptance criteria
[x] Snapshot/golden output testing patterns

### Warning Fixes & Code Cleanup (v3.7.0)
[x] Fixed unused variable warnings in integrity_suite.rs
[x] Fixed "comparison is useless" warnings for u64 types
[x] Fixed unused import warnings (chrono::Utc)
[x] Fixed unused field warnings (latency_ms)
[x] Fixed skill_router.rs test using unused variables
[x] Fixed auto_tune.rs unused decision variable
[x] Replaced SkillType::Unknown with SkillType::Unsupported

### Constraints (v3.7.0)
[x] NO new user-visible features
[x] NO new command line flags
[x] NO new RPCs
[x] ONLY: Fixes, Tests, Performance/reliability improvements, Cleanup/hardening

### Tests (v3.7.0)
[x] 19 system acceptance tests (all passing)
[x] 102 integrity suite tests (all passing)
[x] 15 property-based tests (all passing)
[x] 15 dry-run checker tests (all passing)
[x] All workspace tests passing (898+)
[x] Zero compiler warnings

**Key Files**:
- docs/ANNA_TEST_PLAN.md - Test plan document
- crates/annad/tests/system_acceptance.rs - Main acceptance harness
- crates/annad/tests/integrity_suite.rs - Warning fixes
- crates/annad/tests/property_tests.rs - Warning fixes
- crates/annad/tests/dry_run_checker.rs - Warning fixes
- crates/anna_common/src/skill_router.rs - Warning fixes
- crates/anna_common/src/auto_tune.rs - Warning fixes

---

## v3.5.0 "Verification & Guardrails"

**Theme**: Comprehensive verification infrastructure. Property tests, dry-run checks, clean codebase.

### Feature Verification Matrix Update (v3.5.0)
[x] docs/FEATURE_VERIFICATION_MATRIX.md - Updated with all v3.5.0 test coverage
[x] Maps all 13 subsystems to their verification tests
[x] Each feature has: implementation file, test file, test count, invariants checked

### Property-Based Tests (v3.5.0)
[x] crates/annad/tests/property_tests.rs - 15 property-based tests
[x] PROP-TEL-001/002: Telemetry counter monotonicity
[x] PROP-XP-001/002/003: XP invariants (level bounds 0-99, total XP monotonic)
[x] PROP-REL-001/002: Reliability always in [0.0, 1.0]
[x] PROP-RECIPE-001/002: Threshold bounds validation
[x] PROP-BUDGET-001/002/003: Time budget invariants
[x] PROP-BRAIN-001/002: Brain answer consistency

### Dry-Run Integrity Checker (v3.5.0)
[x] crates/annad/tests/dry_run_checker.rs - 15 dry-run tests
[x] DRY-RUN-001: Brain fast path coverage (canonical questions)
[x] DRY-RUN-002: Recipe store operations (safe load/stats)
[x] DRY-RUN-003: XP system operations (progression add/from_xp)
[x] DRY-RUN-004: Telemetry counter safety
[x] DRY-RUN-005: Benchmark system validation (configs, question sets, phases)
[x] DRY-RUN-006: Performance budget creation
[x] DRY-RUN-007: Config system validation
[x] DRY-RUN-008: Integration mock pipeline

### Integrity Suite Expansion (v3.5.0)
[x] crates/annad/tests/integrity_suite.rs - Expanded to 102 tests
[x] Fixed type assertions (Level newtype, progression starting at 0)
[x] Added snapshot tests for status output formatting
[x] Added degradation harness tests (slow LLM, hard timeout, probe failures)
[x] Added regression detection tests

### Code Quality (v3.5.0)
[x] Fixed clippy warnings (unused parentheses, unused variables)
[x] Added `#![allow(ambiguous_glob_reexports)]` with TODO for v3.6.0
[x] All workspace tests passing (1000+)

### Tests (v3.5.0)
[x] 102 integrity suite tests
[x] 15 property-based tests
[x] 15 dry-run checker tests
[x] 24 orchestration tests
[x] All workspace tests passing

**Key Files**:
- docs/FEATURE_VERIFICATION_MATRIX.md - Updated verification matrix
- crates/annad/tests/property_tests.rs - Property-based tests
- crates/annad/tests/dry_run_checker.rs - Dry-run integrity checker
- crates/annad/tests/integrity_suite.rs - Expanded integrity tests

---

## v3.4.0 "Performance & Degradation Guard"

**Theme**: Performance infrastructure, time budgets, graceful degradation.

### Phase 1: PerfSpan Timing Helpers
[x] Created perf_timing.rs module with timing infrastructure
[x] PerfSpan RAII helper for measuring execution time
[x] PipelineTimings struct for aggregating pipeline metrics
[x] Tracing integration for automatic logging

### Phase 2: Global Time Budget
[x] DEFAULT_GLOBAL_BUDGET_MS = 15,000ms (15 seconds max per question)
[x] FAST_PATH_BUDGET_MS = 500ms (Brain + Recipe target)
[x] GlobalBudget struct with exhaustion detection
[x] Degraded mode detection when budget exceeded

### Phase 3: Tiered LLM Timeouts
[x] JUNIOR_SOFT_TIMEOUT_MS = 4,000ms (trigger degradation warning)
[x] JUNIOR_HARD_TIMEOUT_MS = 6,000ms (cancel and fallback)
[x] SENIOR_SOFT_TIMEOUT_MS = 5,000ms (trigger degradation warning)
[x] SENIOR_HARD_TIMEOUT_MS = 8,000ms (cancel and produce RED answer)
[x] LlmTimeoutResult enum: Success, SoftTimeout, HardTimeout

### Phase 4: Fail-Fast for Unsupported Questions
[x] UnsupportedResult and UnsupportedReason types
[x] classify_unsupported() function (<500ms classification)
[x] Detection for: EmptyInput, Gibberish, Conversational, BeyondCapability, Greeting
[x] Fast honest explanations for unsupported questions

### Phase 5: Fast Degraded Answers
[x] DegradedAnswer and DegradationReason types
[x] DEGRADED_ANSWER_BUDGET_MS = 2,000ms (emergency answer budget)
[x] Reasons: LlmTimeout, BudgetExhausted, LlmInvalid, ProbesFailed, BackendUnavailable
[x] Emergency fallback path (instantaneous)
[x] Partial evidence preservation in degraded answers

### Phase 6: Telemetry-Driven Performance Hints
[x] RollingStats struct (avg_latency, failure_rate, timeout_rate)
[x] get_performance_hint() and get_rolling_stats() functions
[x] PerformanceHint enum: Good, Degraded, Critical
[x] Extended TelemetrySummaryComplete with rolling metrics

### Phase 7: Integrity Tests
[x] 19 performance integrity tests (INV-PERF-001 through INV-PERF-019)
[x] Tests for: global budget, fast path, timeout ordering, PerfSpan accuracy
[x] Tests for: budget tracking, exhaustion, performance hints, LLM timeout evaluation
[x] Tests for: unsupported detection, degraded answer speed, partial evidence

**Tests**: 75+ integrity tests, all passing
**Key Files**:
- crates/anna_common/src/perf_timing.rs - Performance timing infrastructure
- crates/anna_common/src/telemetry.rs - Extended with rolling stats
- crates/annad/tests/integrity_suite.rs - Performance integrity tests
