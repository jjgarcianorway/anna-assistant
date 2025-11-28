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

## v0.27.0 - Planned

[ ] Real-time auto-update integration testing
[ ] Watchdog runtime integration with annad
[ ] Live health monitoring dashboard
[ ] Enhanced error recovery strategies
[ ] Metrics export for monitoring systems
