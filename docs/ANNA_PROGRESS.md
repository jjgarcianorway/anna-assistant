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
