# Anna Assistant - Release Notes

---

## v0.0.4 - Real Junior Verifier

**Release Date:** 2024-12-03

### Summary

Junior becomes a real LLM-powered verifier via local Ollama. Translator remains deterministic. No Senior implementation yet - keeping complexity low while measuring real value.

### Key Features

**Junior LLM Integration:**
- Real verification via Ollama local LLM
- Auto-selects best model (prefers qwen2.5:1.5b, llama3.2:1b, etc.)
- Structured output parsing (SCORE, CRITIQUE, SUGGESTIONS, MUTATION_WARNING)
- Fallback to deterministic scoring when Ollama unavailable
- Spinner while Junior thinks

**Ollama Client (`ollama.rs`):**
- HTTP client for local Ollama API
- Health check, model listing, generation
- Timeout and retry handling
- Model auto-selection based on availability

**Junior Config:**
- `junior.enabled` (default: true)
- `junior.model` (default: auto-select)
- `junior.timeout_ms` (default: 60000)
- `junior.ollama_url` (default: http://127.0.0.1:11434)

### Pipeline Flow (with real LLM)

```
[you] to [anna]: what CPU do I have?
[anna] to [translator]: Please classify this request...
[translator] to [anna]: Intent: system_query, Targets: cpu, Risk: read-only, Confidence: 85%
[anna] to [annad]: Retrieve evidence for: cpu
[annad] to [anna]: snapshot:hw.cpu: [CPU data]
[anna] to [junior]: Please verify this draft response...
[junior thinking via qwen2.5:1.5b...]
[junior] to [anna]: Reliability: 80%
                    Critique: The response mentions evidence but doesn't parse it
                    Suggestions: Add specific CPU model and core count
[anna] to [you]: Based on system data from: snapshot:hw.cpu...
Reliability: 80%
```

### Junior System Prompt

Junior is instructed to:
- NEVER invent machine facts
- Downscore missing evidence
- Prefer "unknown" over guessing
- Keep output short and structured
- Warn about mutations for action requests

### Graceful Degradation

When Ollama is not available:
- REPL shows warning with install instructions
- Pipeline falls back to deterministic scoring (v0.0.3 logic)
- Exit code 0 - no crashes

### Tests

- 9 unit tests for pipeline (Translator, Junior parsing, fallback scoring)
- 20 CLI integration tests
- 4 new v0.0.4 tests (Critique, Suggestions, mutation warning, graceful degradation)

### Model Selection Order

1. qwen2.5:1.5b (fastest, good for verification)
2. qwen2.5:3b
3. llama3.2:1b
4. llama3.2:3b
5. phi3:mini
6. gemma2:2b
7. mistral:7b
8. First available model

---

## v0.0.3 - Request Pipeline Skeleton

**Release Date:** 2024-12-03

### Summary

Implements the full multi-party dialogue transcript with deterministic mocks for intent classification, evidence retrieval, and Junior scoring. No LLM integration yet - all behavior is keyword-based and deterministic.

### Pipeline Flow

```
[you] to [anna]: what CPU do I have?
[anna] to [translator]: Please classify this request...
[translator] to [anna]: Intent: system_query, Targets: cpu, Risk: read-only, Confidence: 85%
[anna] to [annad]: Retrieve evidence for: cpu
[annad] to [anna]: snapshot:hw.cpu: [CPU data would come from snapshot]
[anna] to [junior]: Please verify and score this response.
[junior] to [anna]: Reliability: 100%, Breakdown: +40 evidence, +30 confident, +20 observational+cited, +10 read-only
[anna] to [you]: Based on system data from: snapshot:hw.cpu...
Reliability: 100%
```

### Changes

**Pipeline Module (`pipeline.rs`):**
- DialogueActor enum: You, Anna, Translator, Junior, Annad
- `dialogue()` function with format: `[actor] to [target]: message`
- IntentType enum: question, system_query, action_request, unknown
- RiskLevel enum: read-only, low-risk, medium-risk, high-risk
- Intent struct with keywords, targets, risk, confidence
- Evidence struct with source, data, timestamp

**Translator Mock:**
- Keyword-based intent classification
- Target detection (cpu, memory, disk, network, docker, nginx, etc.)
- Action keyword detection (install, remove, restart, etc.)
- Confidence scoring based on keyword matches

**Evidence Retrieval Mock:**
- Maps targets to snapshot sources (hw.cpu, hw.memory, sw.services.*)
- Returns mock evidence with timestamps
- System queries trigger annad dialogue

**Junior Scoring:**
- +40: evidence exists
- +30: confident classification (>70%)
- +20: observational + cited (read-only with evidence)
- +10: read-only operation
- Breakdown shown in output

**Tests:**
- test_annactl_pipeline_shows_translator
- test_annactl_pipeline_shows_junior
- test_annactl_pipeline_shows_annad_for_system_query
- test_annactl_pipeline_intent_classification
- test_annactl_pipeline_target_detection
- test_annactl_pipeline_reliability_breakdown
- test_annactl_pipeline_action_risk_level

### Internal Notes

- All responses are mocked (no LLM integration)
- Evidence retrieval is simulated (no actual snapshot reads)
- Risk classification is keyword-based
- Pipeline is ready for LLM integration in 0.1.x

---

## v0.0.2 - Strict CLI Surface

**Release Date:** 2024-12-03

### Summary

Enforces the strict CLI surface. All legacy commands (sw, hw, JSON flags) are removed from public dispatch and now route through natural language processing.

### Supported Entrypoints

```bash
annactl                  # REPL mode (interactive)
annactl <request>        # One-shot natural language request
annactl status           # Self-status
annactl --version        # Version (also: -V)
annactl --help           # Help (also: -h)
```

**That's the entire public surface.**

### Changes

**CLI Surface:**
- Removed `sw` command from public surface
- Removed `hw` command from public surface
- Removed all JSON flags (--json, --full) from public surface
- Legacy commands now route as natural language requests (no custom error message)
- Added --help/-h flags for explicit help display

**REPL Mode:**
- Implemented basic REPL loop
- Exit commands: exit, quit, bye, q
- Help command shows REPL-specific help
- Status command works in REPL

**Dialogue Format:**
- Natural language requests show `[you] to [anna]:` format
- Responses show `[anna] to [you]:` format
- Reliability score displayed (stub: 0% until LLM integration)

**Tests:**
- Added test for --help showing strict surface only
- Added test for status command exit 0
- Added test for --version format
- Added test for legacy command routing (sw, hw)
- Added test for natural language request format

### Breaking Changes

- `annactl sw` no longer shows software overview (routes as request)
- `annactl hw` no longer shows hardware overview (routes as request)
- `annactl` (no args) now enters REPL instead of showing help
- Use `annactl --help` or `annactl -h` for help

### Internal

- Snapshot architecture preserved (internal capabilities only)
- Status command unchanged
- Version output format unchanged: `annactl vX.Y.Z`

---

## v0.0.1 - Specification Lock-In

**Release Date:** 2024-12-03

### Summary

Complete specification reset. Anna transitions from a "snapshot reader with fixed commands" to a "natural language virtual sysadmin" architecture.

### Changes

**Governance:**
- Established immutable operating contract (CLAUDE.md)
- Created implementation roadmap (TODO.md)
- Set up release notes workflow
- Version reset to 0.0.1 (staying in 0.x.x until production)

**Documentation:**
- README.md rewritten for natural language assistant vision
- CLAUDE.md created with full engineering contract
- TODO.md created with phased implementation roadmap
- RELEASE_NOTES.md created for change tracking

**Architecture Decision:**
- Preserve existing snapshot-based telemetry foundation
- Build natural language layer on top
- Strict CLI surface: `annactl`, `annactl <request>`, `annactl status`, `annactl --version`
- All old commands (sw, hw, JSON flags) become internal capabilities only

**Spec Highlights:**
- 4-player model: User, Anna, Translator, Junior, Senior
- Debug mode always on (visible dialogue)
- Reliability scores on all answers (0-100%)
- Safety classification: read-only, low-risk, medium-risk, high-risk
- Rollback mandate for all mutations
- Recipe learning system
- XP and gamification (levels 0-100, nerdy titles)
- Auto-update every 10 minutes
- Auto Ollama setup

### Breaking Changes

- Version number reset from 7.42.5 to 0.0.1
- Old CLI commands will be removed in 0.0.2
- New CLI surface is strict and minimal

### Migration Path

Existing snapshot infrastructure is preserved. Natural language capabilities will be added incrementally without breaking current performance.

---

## Previous Versions

Prior to v0.0.1, Anna was a snapshot-based telemetry daemon with fixed CLI commands. See git history for v7.x releases.
