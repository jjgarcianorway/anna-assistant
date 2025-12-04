# Anna Assistant - Release Notes

---

## v0.0.82 - "Stop the Nonsense" UX Stabilization

**Release Date:** 2025-12-04

### Summary

Major UX stabilization release: Pre-router for deterministic routing without LLM, translator JSON schema with robust parsing, debug mode toggle via natural language, and comprehensive golden tests for clean output. Project completion: 28.9%.

### Features

**Pre-Router (Deterministic Routing):**
- Routes common queries directly without LLM: stats, memory, disk, CPU, kernel, network, editor, updates, capabilities, debug toggle
- Typo tolerance: "rum" interpreted as "RAM"
- Falls through to translator for complex/action requests
- Module: `pre_router.rs` with `PreRouterIntent` enum

**Translator JSON Schema:**
- Strict JSON schema for translator output
- Robust parser supports: direct JSON, markdown code blocks, embedded JSON, legacy text format
- Silent in normal mode (no parse warnings to stdout)
- Deterministic fallback when JSON parsing fails
- Module: `translator_v082.rs`

**Debug Mode Toggle:**
- Enable: "enable debug", "debug on", "turn debug on"
- Disable: "disable debug", "debug off", "turn debug off"
- Session-based via ANNA_DEBUG_TRANSCRIPT env var
- Persistent via config.toml update
- Module: `debug_toggle.rs`

**Direct Handlers (Skip Translator):**
- Stats: RPG stats display with level, XP, title
- Memory: Parse /proc/meminfo, show usage in GB
- Disk: Run df -h with human-readable output
- CPU: Parse /proc/cpuinfo for model, cores, speed
- Kernel: Read /proc/sys/kernel for release/version
- Network: Ping test + ip addr output
- Editor: Check $VISUAL, $EDITOR, installed editors
- Updates: checkupdates or pacman -Qu
- Capabilities: Help text with Anna's abilities
- Module: `pipeline_v082.rs`

**Golden Tests:**
- 52 tests for normal mode clean output
- Verify no evidence IDs, tool names, or debug info
- Test pre-router matching for all query variations
- Test handler output cleanliness
- Test reliability scores (100% for deterministic)
- Module: `golden_tests_v082.rs`

### Technical Details

**New Modules in anna_common:**
- `pre_router.rs` - Deterministic routing before translator
- `translator_v082.rs` - JSON schema + robust parser
- `debug_toggle.rs` - Natural language debug mode control
- `pipeline_v082.rs` - Direct handler execution
- `golden_tests_v082.rs` - Clean output verification tests

**New Exports:**
- `pre_route`, `PreRouterIntent`, `PreRouterResult`
- `parse_translator_json`, `TranslatorJsonOutput`, `classify_deterministic`
- `toggle_debug_session`, `toggle_debug_persistent`, `DebugMode`
- `try_direct_handle`, `DirectHandlerResult`

### Tests

- All 52 v0.0.82 tests pass
- Pre-router tests: 11 (query variations, no-match cases)
- Translator tests: 9 (JSON parsing, legacy format, deterministic)
- Debug toggle tests: 6 (enable/disable patterns, session toggle)
- Pipeline tests: 10 (direct handlers for all query types)
- Golden tests: 18 (output cleanliness, reliability)

---

## v0.0.81 - Mutation Engine Refinements

**Release Date:** 2025-12-04

### Summary

Refinements to the Mutation Engine: real verification, smart previews, reliability gating, unified transcript modes, enhanced status display, and reliable rollback execution. Project completion: 24.5%.

### Features

**A) Real Verification (not generic):**
- Service verification: Uses `systemctl is-active` and `is-enabled` with actual state comparison
- Package verification: Uses `pacman -Qi` to check installed version
- Config verification: Checks actual file content and backup existence
- Diagnostic output on verification failure

**B) Smart and Specific Previews:**
- Service unit resolution: "docker" → "docker.service", "networkmanager" → "NetworkManager.service"
- Package info from `pacman -Si`: version, size, dependencies (top 5)
- Warning if package not found in repositories

**C) Reliability Gating:**
- Mutations require 90% minimum reliability to proceed
- Read-only queries: 70% minimum
- Diagnostic flows: 80% minimum
- Clear feedback when blocked with improvement suggestions

**D) Transcript Mode Overhaul:**
- Unified `transcript_config` module
- Human mode: Professional IT department dialogue, no raw tool names/evidence IDs
- Debug mode: Full internals, timing, retries, parse warnings
- Mode switching via ANNA_DEBUG_TRANSCRIPT=1 or ANNA_UI_TRANSCRIPT_MODE=debug

**E) Case File Schema v0.0.81:**
- Unified schema at `/var/lib/anna/cases/<case_id>.json`
- Includes mutation artifacts: backups, rollback info, verification results
- Doctor data: findings, causes, suggested actions
- Methods for save/load/list_recent

**F) Reliable Rollback Executor:**
- File rollback: Restore from backup with content verification
- Service rollback: Restore previous active/enabled state
- Package rollback: Use cached `.pkg.tar.zst` from `/var/cache/pacman/pkg`
- Verification after every rollback

**G) Enhanced Status Display:**
- Debug mode indicator (source: env var, config, or default)
- Last 3 cases with reliability scores and mutation indicator
- Mutation stats: total, successful, failed, rollbacks, by-type breakdown
- Doctor stats: invocations, successes, by-domain breakdown

### New Modules

- `mutation_verification.rs`: Real verification for all mutation types
- `reliability_gate.rs`: Reliability-based action gating
- `transcript_config.rs`: Unified transcript mode configuration
- `case_file_v081.rs`: Case file schema with mutation artifacts
- `rollback_executor.rs`: Actual rollback execution
- `status_v081.rs`: Enhanced status information

### Tests

- All existing tests pass (1372 in anna_common, 41 in annactl)
- New tests for each new module

---

## v0.0.80 - Mutation Engine v1

**Release Date:** 2025-12-04

### Summary

First implementation of the Mutation Engine with safe, transparent system changes. Anna can now execute service control, package management, and config file edits with full plan/preview/confirm/execute/verify/rollback pipeline.

### Features

**Mutation Pipeline:**
- Plan → Preview → Confirm → Execute → Verify → Rollback (if needed)
- Unified diff previews for config file changes
- Risk-based confirmation phrases: "I CONFIRM (low risk)", "I CONFIRM (medium risk)", "I CONFIRM (high risk)"
- Automatic rollback on verification failure with "I CONFIRM ROLLBACK" confirmation

**Supported Mutations (v0.0.80 scope - deliberately limited):**
- **Service Control**: NetworkManager, sshd, docker, bluetooth (whitelist only)
  - Actions: start, stop, restart, enable, disable
- **Package Management**: pacman install/remove (no AUR support yet)
- **Config File Editing**: /etc/pacman.conf, /etc/ssh/sshd_config, /etc/NetworkManager/NetworkManager.conf
  - Operations: add line, replace line, comment, uncomment

**Privilege Handling:**
- Detects root or passwordless sudo without hanging on prompts
- Shows manual commands when privilege not available
- Graceful degradation - never blocks waiting for password

**Transcript Integration:**
- "Change Manager" dialogue layer in human mode
- Full plan/step IDs in debug mode
- Risk levels shown as colloquial phrases ("straightforward, low-risk", etc.)

**Recipe Learning:**
- Mutations can be learned as recipes if reliability >= 95%
- Requires 3+ evidence items (stricter than read-only recipes)
- Rollback blocks recipe extraction

**Status Display:**
- New [MUTATIONS] section in annactl status
- Shows capability (privilege status), successful/rollback/blocked counts
- Lists supported scope for v0.0.80

### Tests

- 23 integration tests for mutation engine
- Tests for privilege detection, allowlists, risk levels, confirmation validation
- Tests for plan creation, rollback creation, verification creation

---

## v0.0.77 - Version Detection Fix

**Release Date:** 2025-12-04

### Summary

Fixed version detection to filter for Anna releases (0.0.x) only, ignoring legacy installer releases (7.x.x) that were confusing semantic version sorting.

### Problem

After fixing v0.0.76 to use semver sorting, the install script found 7.42.5 as "latest" because old installer releases (7.x.x) were in the releases list, and 7.42.5 > 0.0.76 semantically.

### Fix

- Install script: Changed regex from `[0-9]+\.[0-9]+\.[0-9]+` to `0\.0\.[0-9]+`
- Update checker: Added `if !version_str.starts_with("0.0.") { continue; }` filter

---

## v0.0.76 - Semantic Version Fix

**Release Date:** 2025-12-04

### Summary

Fixed install script and daemon auto-updater to find the highest semantic version, not the most recently created release. This caused issues when backfilling old missing releases.

### Problem

The GitHub API endpoint `/releases/latest` returns the **most recently published** release, not the one with the highest version number. When backfilling missing releases (v0.0.60, v0.0.69, etc.), they were being created with recent timestamps, causing:

- Install script reported v0.0.60 as latest instead of v0.0.75
- Auto-updater showed v0.0.48 -> v0.0.70 instead of v0.0.75

### Solution

**Install Script (v7.43.0):**
- Changed from `/releases/latest` to `/releases?per_page=100`
- Extracts all version tags and uses `sort -V | tail -1` to find highest

**Update Checker (v7.43.0):**
- Fetches all releases instead of just latest
- Parses versions with `parse_version()` helper
- Finds highest (major, minor, patch) tuple

### Also Fixed

- Marked v0.0.75 as "Latest" release in GitHub using `gh release edit v0.0.75 --latest`

---

## v0.0.75 - Persistent Learning + RPG Stats + Transcript Polish

**Release Date:** 2025-12-04

### Summary

Recipes now actually change future behavior with configurable reliability thresholds. RPG stats block added to `annactl status`. Human vs debug transcript separation fully implemented. Expected project completion: 18.5%.

### Part A: Persistent Learning System

**New Module: recipe_engine.rs**
- Recipe matching using canonical translator fields (INTENT/TARGETS/RISK/TOOLS/DOCTOR/CONFIDENCE)
- Precondition validation before recipe use
- Automatic recipe demotion after 3 consecutive failures

**Reliability Thresholds (Configurable):**
- Read-only queries: 90% minimum reliability
- Doctor workflows: 80% minimum reliability
- Mutation plans: 95% minimum reliability

**Evidence Requirements:**
- Read-only: 1 evidence item minimum
- Doctor: 2 evidence items minimum
- Mutation: 3 evidence items minimum

**Recipe Lifecycle:**
- Active: Full reliability, auto-suggested
- Draft: Created when reliability below threshold
- Archived: Demoted after consecutive failures

**Coverage Tracking:**
- Rolling request/match counters
- Coverage percentage for status display
- By-domain statistics

### Part B: RPG Stats in annactl status

**New Module: rpg_stats.rs**
- XP and Level (0-100) with non-linear progression
- Titles: Intern→Apprentice→Technician→Analyst→Engineer→Senior Engineer→Architect→Wizard→Sage→Grandmaster

**Metrics Tracked:**
- Request counters: total, successes, failures
- Reliability: average (all-time), rolling last-50
- Escalation percentages: junior used, doctor used, recipe-solved
- Latency: median, p95 (total request)
- By-domain breakdown: count, success rate, avg reliability

**Visual Progress Bars:**
- XP progress bar (= characters)
- Success rate bar (# characters)
- Recipe coverage bar (* characters)

**Storage:** `/var/lib/anna/internal/stats.json`

### Part C: Transcript Polish

**New Module: transcript_v075.rs**
- Complete human/debug mode separation
- Evidence as descriptions, not IDs

**Human Mode:**
- No evidence IDs ([E1], [E2])
- No tool names (hw_snapshot_summary, memory_info)
- Topics as descriptions: "Hardware snapshot: Intel i9-14900HX"
- Confidence-based prefixes:
  - 90%+: Direct statement
  - 75-90%: "It looks like..."
  - 60-75%: "I think..."
  - <60%: "I'm not certain, but..."

**Debug Mode:**
- Full evidence IDs and tool names
- Timing information
- Parse warnings and retries

**Validation:**
- `validate_human_output()` checks for forbidden terms
- Forbidden: [E#], _summary, _probe, _v1, tool names, etc.

### Part D: Integration Tests

**22+ New Tests in learning_stats_tests.rs:**
- Recipe creation gate tests (thresholds by risk level)
- RPG stats recording and calculation tests
- Rolling reliability window tests
- Latency percentile calculation tests
- Level title mapping tests
- Human transcript validation tests
- Evidence description format tests
- Confidence prefix tests
- CLI surface verification (no new commands)

### Files Changed

**New Files:**
- `crates/anna_common/src/recipe_engine.rs` - Enhanced recipe matching and lifecycle
- `crates/anna_common/src/rpg_stats.rs` - RPG stats and metrics system
- `crates/anna_common/src/transcript_v075.rs` - Transcript rendering enhancements
- `crates/anna_common/src/learning_stats_tests.rs` - Integration tests

**Modified Files:**
- `crates/anna_common/src/lib.rs` - New module exports
- `README.md` - v0.0.75 description
- `CLAUDE.md` - Version bump
- `TODO.md` - v0.0.75 completed, v0.0.76 planned

### Storage Locations

- Recipe engine state: `/var/lib/anna/internal/recipe_engine_state.json`
- RPG stats: `/var/lib/anna/internal/stats.json`
- Recipes: `/var/lib/anna/recipes/` (unchanged)

---

## v0.0.74 - Direct Answers + Classification Fix

**Release Date:** 2025-12-04

### Summary

System queries (RAM, kernel, disk, network, service status) now return direct answers instead of "Proposed action plan" scaffolding. Fixed classification order to ensure read-only queries never become action requests.

### Part A: Request Classification Fix

**Classification Reordering:**
- SYSTEM_QUERY patterns now checked BEFORE ACTION_REQUEST patterns
- Added `is_read_only_query()` function to guard against false action classification
- Service verbs (stop/start/restart) detected at request start for proper action classification

**New Query Patterns:**
- Memory: "how much ram do i have", "memory do i have"
- Kernel: "kernel am i using", "running kernel"
- Disk: "space free", "space left"
- Network: "am i connected", "am i online"
- Service: "is X running", "is X enabled"

### Part B: Direct Answer Generation

**New Module: direct_answer.rs**
- Topic-based answer routing using evidence detection
- Generates human-readable answers from tool results
- No "Proposed action plan" scaffolding for read-only queries

**Answer Formats:**
- Memory: "You have 32 GiB of RAM total, 16 GiB available."
- Kernel: "You are running Linux kernel 6.17.9-arch1-1."
- Disk: "Your root filesystem has 100 GiB free (40% available)."
- Network: "Network is connected, primary interface: enp5s0."
- Service: "docker is running."

### Part C: Structured Doctor Outputs

**New Types in doctor_registry.rs:**
- `StructuredDoctorOutput`: title, findings, evidence_topics, recommendations, risk, reliability_hint
- `HumanFinding`: description, severity, source
- `ReliabilityHint`: evidence_count, confidence, coverage

**Human/Debug Mode Formatting:**
- Human mode: No IDs, no tool names, conversational output
- Debug mode: Full IDs and evidence references

### Part D: Transcript Improvements

**New Humanizer Functions (humanizer/transform.rs):**
- `humanize_direct_answer()`: Format direct answers for Human mode
- `humanize_system_query_answer()`: Format with evidence validation

### Part E: Integration Tests

**26 New Tests in direct_answer_tests.rs:**
- Classification tests (memory/kernel/disk/network/service → SystemQuery)
- Topic detection tests (correct EvidenceTopic for each query type)
- Query target tests (correct tool routing)
- Direct answer generation tests
- Regression tests (no action plan for system queries)
- Human mode verification (no leaked IDs or tool names)

### Files Changed

**New Files:**
- `crates/anna_common/src/direct_answer.rs`
- `crates/anna_common/src/direct_answer_tests.rs`

**Modified Files:**
- `crates/anna_common/src/intent_taxonomy.rs` - Classification reordering, read-only detection
- `crates/anna_common/src/system_query_router.rs` - Mutation verb exclusion for service detection
- `crates/anna_common/src/doctor_registry.rs` - Structured output types
- `crates/anna_common/src/humanizer/transform.rs` - Direct answer formatting
- `crates/anna_common/src/lib.rs` - New module exports

---

## v0.0.73 - Human Transcript Realism + Auto-Update Rewrite

**Release Date:** 2025-12-04

### Summary

Two major deliverables: (1) Human transcript now reads like a real IT department with role-appropriate phrasing, and (2) auto-update has been completely rewritten for reliability with proper locking, atomic installs, and automatic rollback.

### Part 1: Human Transcript Realism

**Role-Based Phrasing (humanizer/phrases.rs):**
- Service Desk: "I'm triaging the request and deciding who should handle it."
- Network: "Looking at link state and active connections."
- Storage: "Checking disk and filesystem status."
- Performance: "Checking the latest hardware and load snapshot."
- Audio: "Checking audio server and device status."

**Doctor Selection UX:**
```
[network] Network team is taking this case.
  [network] We'll start by checking interface state and connectivity.
```

**Evidence Labels with Source Context:**
- "CPU model and core count (from latest hardware snapshot)"
- "Disk usage summary (from storage snapshot)"
- "Running services summary (from service snapshot)"

No evidence IDs or tool names exposed in Human mode.

### Part 2: Auto-Update Rewrite

**State Machine (updater/steps.rs):**
11-step update flow with persistence after each step:
1. acquire_lock
2. check_remote_release
3. compare_versions
4. download_assets
5. verify_assets
6. install_cli (optional)
7. install_daemon (optional)
8. restart_daemon (optional)
9. healthcheck
10. release_lock
11. rollback (on failure)

**Hard Locking (updater/lock.rs):**
- Filesystem lock file at `/var/lib/anna/internal/update.lock`
- Contains PID, timestamp, hostname, current step
- Stale lock detection (>5 minutes) with automatic recovery
- Dead process detection via /proc check

**Atomic Installs:**
- Download to staging directory
- Verify SHA256 checksums
- Verify ELF executable format
- Rename into place atomically
- Backup previous binaries to `/var/lib/anna/internal/backups/`

**Rollback Guarantees:**
- Automatic rollback if daemon restart fails
- Automatic rollback if healthcheck fails
- Checksums verified on restore
- Daemon restarted after rollback

**Version Mismatch Detection:**
- `annactl status` shows WARNING if CLI and daemon versions differ
- Auto-update attempts to resolve mismatches

### Key Files

**Part 1 (Humanizer):**
- `crates/anna_common/src/humanizer/phrases.rs` - Role-based phrasing (NEW)
- `crates/anna_common/src/humanizer/transform.rs` - Updated with doctor selection
- `crates/anna_common/src/humanizer/mod.rs` - Re-exports and tests

**Part 2 (Updater):**
- `crates/anna_common/src/updater/mod.rs` - Module entry (NEW)
- `crates/anna_common/src/updater/lock.rs` - Hard locking (NEW)
- `crates/anna_common/src/updater/state.rs` - State machine (NEW)
- `crates/anna_common/src/updater/steps.rs` - Execution flow (NEW)
- `crates/annactl/src/commands/status.rs` - Version mismatch display

### Tests

- 11 updater tests (lock, state, steps)
- 38 humanizer tests (phrases, transform, workflows)
- 49 new tests total

---

## v0.0.72 - Dual Transcript Mode

**Release Date:** 2025-12-04

### Summary

Both Human and Debug transcript modes are now generated from a SINGLE event stream, ensuring they cannot diverge. Human mode shows clean IT department dialogue without tool names, evidence IDs, or raw commands. Debug mode shows full internals for troubleshooting.

### Key Concepts

**Single Event Stream, Dual Renderers:**
- All transcript events stored in `TranscriptStreamV72`
- `render_human_v72()` produces clean output
- `render_debug_v72()` produces full internal details
- Both modes are faithful representations of the same events

**Enable Debug Mode:**
- `ANNA_DEBUG_TRANSCRIPT=1` env var (shorthand for tests)
- `ANNA_UI_TRANSCRIPT_MODE=debug` env var
- `/etc/anna/config.toml` with `transcript_mode = "debug"`
- No new public CLI commands or flags

### Key Features

**1. Unified Event Types**
```rust
pub enum EventDataV72 {
    UserMessage { text: String },
    StaffMessage { role: RoleV72, tone: ToneV72, content_human: String, content_debug: Option<String> },
    Evidence { evidence_id: String, tool_name: String, human_label: String, summary_human: String, summary_debug: Option<String>, duration_ms: u64 },
    ToolCall { tool_name: String, action_human: String, args: Option<String> },
    Classification { understood_human: String, canonical_lines: Option<Vec<String>>, parse_attempts: Option<u8>, fallback_used: bool },
    Reliability { score: u8, rationale_short: String, rationale_full: Option<String>, uncited_claims: Option<Vec<String>> },
    Confirmation { change_description: String, risk_level: RiskLevelV72, confirm_phrase: String, rollback_summary: String, rollback_details: Option<String> },
    Warning { message_human: String, details_debug: Option<String>, category: WarningCategoryV72 },
    // ...
}
```

**2. Humanized Equivalents**
- "Translator struggled to classify this; we used house rules." (instead of "deterministic fallback")
- "Hardware: Pulled inventory from the latest hardware snapshot." (instead of "[E1] hw_snapshot_summary")
- Parse warnings hidden in human mode, visible in debug

**3. Confirmation Safety Preserved**
- Confirmation phrase unchanged in both modes
- Risk level and rollback info always shown
- Only the description is humanized

**4. Forbidden Pattern Validation**
```rust
pub static FORBIDDEN_HUMAN_PATTERNS: &[&str] = &[
    r"\[E\d+\]",           // Evidence IDs
    r"hw_snapshot_",       // Tool names
    r"sw_snapshot_",
    r"journalctl",         // Raw commands
    r"systemctl\s",
    // ...
];
```

### Files

- `crates/anna_common/src/transcript_v072/events.rs` - Unified event types
- `crates/anna_common/src/transcript_v072/render.rs` - Dual renderers
- `crates/anna_common/src/transcript_v072/validation.rs` - Forbidden pattern validation
- `crates/anna_common/src/transcript_v072/output.rs` - File output utilities
- `crates/anna_common/src/transcript_v072/mod.rs` - Module re-exports
- `scripts/anna_deep_test.sh` - Updated with `run_dual_mode_tests()`

### Tests

- 26 tests in `transcript_v072` module
- Integration tests: `test_complete_workflow_human_clean`, `test_complete_workflow_debug_has_internals`
- Validation tests: `validate_human_output()`, `validate_debug_has_internals()`
- Deep test harness: `run_dual_mode_tests()` with `transcripts-human/` and `transcripts-debug/` directories

---

## v0.0.71 - Real IT Department Humanizer

**Release Date:** 2025-12-04

### Summary

Makes the Human transcript read like a competent IT org operating in real time, while remaining a faithful rendering of internal events. Adds role/tone metadata, natural language phrasing, micro-threads for complex cases, shorter human labels, and validation to prevent misclassification.

### Key Concepts

**Role/Tone Metadata:**
- StaffRole: ServiceDesk, Department, Anna, Translator, Junior, Senior
- MessageTone: Neutral, Brisk, Skeptical, Helpful, Cautious, Urgent
- ConfidenceHint: Low, Medium, High (derived from scores)
- DepartmentTag: Network, Storage, Performance, Audio, Graphics, Boot, Security, InfoDesk

**Humanizer Layer:**
- Transforms internal events to natural language
- HumanizedMessage with tag, text, tone, is_side_thread
- HumanizerContext tracks confidence, evidence_missing, complexity

**Micro-Threads:**
- ThreadBuilder creates multi-threaded transcripts
- Side threads for evidence gathering (indented with 2 spaces)
- Main thread for case open, triage, findings, final answer

**Human Labels (v0.0.71):**
- Shorter labels without "snapshot" suffix
- "hardware inventory" instead of "hardware inventory snapshot"
- "network link and routing signals" instead of "network status snapshot"

### Key Features

**1. Role and Tone System**
```rust
pub enum StaffRole { ServiceDesk, Department, Anna, Translator, Junior, Senior }
pub enum MessageTone { Neutral, Brisk, Skeptical, Helpful, Cautious, Urgent }

impl MessageTone {
    pub fn from_confidence(confidence: u8) -> Self {
        if confidence >= 85 { Self::Brisk }
        else if confidence >= 70 { Self::Neutral }
        else if confidence >= 50 { Self::Helpful }
        else { Self::Skeptical }
    }
}
```

**2. Humanized Messages**
```rust
pub struct HumanizedMessage {
    pub tag: String,           // "service desk", "network", etc.
    pub text: String,          // Natural language text
    pub tone: MessageTone,
    pub is_side_thread: bool,  // Indented evidence gathering
}

// Example: humanize_case_open("what is my cpu", &ctx)
// → HumanizedMessage { tag: "service desk", text: "Opening case and reviewing request.", ... }
```

**3. Micro-Thread Rendering**
```rust
let mut builder = ThreadBuilder::new();
builder.case_open(humanize_case_open("memory check", &ctx));
builder.start_evidence(DepartmentTag::Performance);
builder.evidence(humanize_evidence_gather(...));  // Side thread (indented)
builder.end_evidence();
builder.finding(humanize_finding(...));           // Main thread

// Output:
// [service desk] Opening case and reviewing request.
//   [performance] hardware inventory: 32 GiB total, 45% used
// [performance] Your system has 32 GiB of RAM.
```

**4. Answer Validation**
```rust
pub fn validate_answer_relevance(query: &str, answer: &str) -> AnswerValidation {
    // Memory query should not get CPU answer
    if query.contains("memory") && !answer.contains("memory") && !answer.contains("ram") {
        return AnswerValidation::WrongTopic { expected: "memory", got_hint: "cpu" };
    }
    // Disk query should not get CPU answer
    // Systemd query should have specific answer
    ...
}
```

**5. Human Labels**
```rust
pub enum HumanLabel {
    HardwareInventory,   // "hardware inventory"
    SoftwareServices,    // "software/services inventory"
    NetworkSignals,      // "network link and routing signals"
    StorageStatus,       // "storage status"
    ...
}
```

### Module Structure

```
anna_common/src/humanizer/
├── mod.rs         # Re-exports, standard tags, integration tests
├── roles.rs       # StaffRole, MessageTone, ConfidenceHint, DepartmentTag
├── labels.rs      # HumanLabel, EvidenceSummary, humanize_evidence
├── transform.rs   # HumanizedMessage, HumanizerContext, humanize_*, validate_*
└── threads.rs     # ThreadBuilder, ThreadSegment, ThreadedTranscript
```

### Tests

- 29 humanizer tests covering workflow, validation, threading
- Misclassification guardrail tests in case_coordinator
- Integration tests for complete workflows

### What's Allowed vs. NOT Allowed

**Allowed Realism (reflects real uncertainty):**
- "I can't confirm link state from snapshots alone; I need live signals."
- "Let's keep it read-only for now and gather more evidence."
- "Your answer is okay, but you didn't ground it in the hardware inventory."

**NOT Allowed (fabrication):**
- Pretending a command ran if it did not
- Claiming a file was read when snapshots didn't include it
- Inventing device names or numbers

---

## v0.0.70 - Dual Transcript Renderer

**Release Date:** 2025-12-04

### Summary

Complete separation between Human Mode and Debug Mode transcript rendering. Human Mode presents a "fly on the wall" IT department conversation without internal details, while Debug Mode provides full transparency for developers and testing.

### Key Concepts

**Human Mode (default):**
- No tool names (hw_snapshot_summary, sw_snapshot_summary, etc.)
- No evidence IDs ([E1], [E2], etc.)
- No raw commands (journalctl, systemctl, etc.)
- No parse errors or fallback indicators
- Uses topic abstractions: "hardware inventory snapshot", "network status snapshot"

**Debug Mode:**
- Canonical translator output (6-line format)
- Tool names, evidence IDs, timing
- Parse warnings, retries, fallbacks
- Raw evidence payloads
- Reliability scoring inputs

### Key Features

**1. Evidence Topic Abstraction**
```rust
tool_to_evidence_topic("hw_snapshot_summary") → EvidenceTopicV70::HardwareInventory
tool_to_evidence_topic("network_status") → EvidenceTopicV70::NetworkSnapshot

// Human-readable descriptions
EvidenceTopicV70::HardwareInventory.human_description() → "hardware inventory snapshot"
EvidenceTopicV70::NetworkSnapshot.human_description() → "network status snapshot"
```

**2. Actor Visibility**
```rust
// Visible in human mode:
ActorV70::You, ServiceDesk, Networking, Storage, Boot, Audio, Graphics, Security, Performance, InfoDesk

// Hidden in human mode (internal):
ActorV70::Translator, Junior, Senior, Annad
```

**3. Event Types**
```rust
EventV70::UserToAnna { text }                    // User's request
EventV70::StaffMessage { from, to, ... }         // Staff dialogue
EventV70::Evidence { actor, topic, ... }         // Evidence with dual summaries
EventV70::ToolCall { tool_name, duration_ms }    // Debug only
EventV70::ParseWarning { subsystem, details }    // Debug only
EventV70::TranslatorCanonical { ... }            // Debug only (6-line format)
EventV70::Reliability { score, rationale_* }     // Score with dual rationales
EventV70::FinalAnswer { text, reliability }      // Final response
```

**4. Validation**
```rust
// Forbidden terms in human mode output
FORBIDDEN_HUMAN: &[&str] = &[
    "[E1]", "[E2]", "_snapshot", "_summary",
    "journalctl", "systemctl", "Parse error",
    "deterministic fallback", "tool=", "evidence_id",
];

let violations = validate_human_output(&human_lines);
assert!(violations.is_empty());
```

**5. Toggle Mode**
```bash
# Environment variable
export ANNA_UI_TRANSCRIPT_MODE=debug
annactl "what is my wifi status"

# Or flag
annactl --debug "what is my wifi status"
```

### Human Mode Example Output
```
[you] what is my wifi status
[networking] Evidence from network status snapshot: WiFi connected to "MyNetwork"
[service-desk] Your WiFi is connected to the network "MyNetwork" with good signal strength.

Reliability: 85% (direct evidence)
```

### Debug Mode Example Output
```
=== Case: case_abc123 ===
Stats: 3 tools (42ms), 0 parse warnings, 0 retries, 0 fallbacks

14:32:15.123 [you] what is my wifi status
14:32:15.125 [translator] CANONICAL OUTPUT:
    INTENT: question
    TARGET: networking
    DEPTH: quick
    TOPICS: wifi, network_status
    ACTIONS: none
    SAFETY: read-only
14:32:15.130 [networking] EVIDENCE [E1] tool=network_status topic=NetworkSnapshot (42ms)
    wlan0: UP, carrier=true, ssid="MyNetwork", signal=-45dBm
14:32:15.150 [FINAL] reliability=85% (direct evidence)
    Your WiFi is connected to the network "MyNetwork" with good signal strength.
```

### Status Command Integration

The `annactl status` command now shows transcript mode in the [CASES] section:
```
[CASES]
  Transcript: Human (IT dialogue, no tool names/evidence IDs)
              Enable debug: ANNA_UI_TRANSCRIPT_MODE=debug or --debug
```

### Module Structure

All files under 400 lines per CLAUDE.md requirement:
```
transcript_v070/
├── mod.rs       (174 lines) - Module exports and integration tests
├── events.rs    (272 lines) - ActorV70, EventV70, TranscriptStreamV70
├── topics.rs    (165 lines) - EvidenceTopicV70, tool_to_evidence_topic()
├── render.rs    (330 lines) - render_human(), render_debug(), write_transcripts()
├── colored.rs   (321 lines) - print_human_colored(), print_debug_colored()
└── validation.rs (60 lines) - FORBIDDEN_HUMAN, validate_human_output()
```

### Tests

11 unit tests covering:
- Topic mapping (tool names → topic categories)
- Actor visibility (internal actors hidden in human mode)
- Event stats tracking (parse warnings, retries, fallbacks)
- Human/debug separation (no tool names in human, full detail in debug)
- Validation (forbidden terms detected)

---

## v0.0.69 - Service Desk Case Coordinator

**Release Date:** 2025-12-04

### Summary

Introduces the CaseCoordinator layer that runs for every request, orchestrating the full case lifecycle from opening through final answer composition. Fixes the "wrong targeting" bugs where memory queries became action requests or disk queries got CPU info. Adds structured DepartmentReport output with dual human/debug summaries and evidence topic mapping.

### The Problem This Solves

Before v0.0.69:
```
User: "how much memory do I have"
→ Intent: ActionRequest (WRONG!)
→ Topics: [CpuInfo] (WRONG!)
→ Department: Performance (WRONG!)
```

After v0.0.69:
```
User: "how much memory do I have"
→ Intent: SystemQuery (CORRECT)
→ Topics: [MemoryInfo] (CORRECT)
→ Department: Performance (CORRECT, but now with proper routing)
```

### Key Features

**1. CaseCoordinator Orchestration**
```rust
let mut coord = CaseCoordinator::open_case("how much disk space is free");
let triage = coord.triage(request);  // primary + supporting depts
// ... dispatch to departments ...
coord.add_report(networking_report);
coord.merge_reports();
let answer = coord.compose_user_answer();
```

**2. Hard Rules in classify_intent()**
- Queries starting with "how much", "what is", "show me" = SystemQuery
- Keywords "install", "remove", "enable" = ActionRequest
- Keywords "broken", "disconnecting", "not working" = ProblemReport

**3. Evidence Topic Mapping Table**
| Query Contains | Evidence Topics |
|----------------|-----------------|
| memory, ram | MemoryInfo |
| disk, space, storage | DiskFree |
| network, wifi, internet | NetworkStatus |
| audio, sound | AudioStatus |
| boot, startup | BootTime, ServiceState |

**4. DepartmentReport Structure**
```rust
DepartmentReport {
    department: DepartmentName::Networking,
    summary_human: "Network is stable, all interfaces up.",
    findings: vec![...],
    evidence_topics: vec![EvidenceTopicSummary { ... }],
    confidence: 90,
    recommended_next_steps: vec![...],
    action_plan: Some(ActionPlan { ... }),
    policy_notes: vec![],
}
```

**5. Multi-Department Triage**
```rust
TriageDecision {
    primary_dept: Networking,
    supporting_depts: vec![Audio],  // "wifi and sound problems"
    intent: ProblemReport,
    matched_keywords: vec!["wifi", "sound"],
    confidence: 85,
}
```

### Unit Tests (9 tests)
1. `test_classify_intent_system_query` - Queries are SystemQuery
2. `test_classify_intent_action_request` - Install/remove are ActionRequest
3. `test_classify_intent_problem_report` - "disconnecting" is ProblemReport
4. `test_evidence_topics_for_memory` - Memory query → MemoryInfo
5. `test_evidence_topics_for_disk` - Disk query → DiskFree
6. `test_case_coordinator_triage` - Full triage flow
7. `test_multi_dept_detection` - Compound queries route to multiple depts
8. `test_department_report_merge` - Reports merge with weighted confidence
9. `test_human_transcript_no_internals` - Human mode has no debug info

### Human Mode Output Example

```
[service-desk] Opening case and reviewing request.
[service-desk] I'll have Performance look into this.
[performance] Memory usage is normal at 45%.

Reliability: 90% (good evidence coverage)
```

### Files Changed
- **NEW**: `crates/anna_common/src/case_coordinator.rs` (832 lines)
- `crates/anna_common/src/lib.rs` - Added case_coordinator exports

---

## v0.0.68 - Two-Layer Transcript Renderer

**Release Date:** 2025-12-04

### Summary

Implements a complete separation between Human Mode and Debug Mode for the transcript system. Human Mode (default) shows a natural IT department conversation without any internal details. Debug Mode provides full fidelity for troubleshooting with exact prompts, evidence IDs, tool names, and timings. Case files now store both human.log and debug.log.

### The Problem This Solves

Before v0.0.68:
```
[annad] tool=hw_snapshot_summary [E1] (42ms)
[junior] Evidence coverage 85%, [E1] matches cpu query
```

After v0.0.68 (Human Mode):
```
[service-desk] I'll check your system information.
[networking] Link State: WiFi is connected and interface is up.
[service-desk] Your CPU is an AMD Ryzen 9 with 12 cores.

Reliability: 94% (good evidence coverage)
```

After v0.0.68 (Debug Mode):
```
12:34:56.789 [tool_call] [annad] tool=hw_snapshot_cpu [E1] (42ms)
12:34:56.831 [decision] [junior] Evidence coverage 100%, reliability=94%
```

### Key Features

**1. TranscriptEventV2 with Dual Summaries**
```rust
TranscriptEventV2::EvidenceTopicSummary {
    topic_title: "Link State",
    summary_human: "WiFi is connected",  // No tool names!
    summary_debug: "wlan0: UP (carrier=true)",
    evidence_ids: vec!["E1"],  // Hidden in human mode
    tool_names: vec!["network_status"],  // Hidden in human mode
}
```

**2. TranscriptRole with Visibility Control**
- Visible in Human Mode: service-desk, networking, storage, boot, audio, graphics
- Hidden in Human Mode: translator, junior, annad (internal roles)

**3. Humanized Reliability Line**
- 80%+: "Reliability: X% (good evidence coverage)"
- 60-79%: "Reliability: X% (some evidence gaps)"
- <60%: "Reliability: X% (limited evidence, some assumptions)"

**4. validate_human_output() Contract**
Rejects output containing:
- Evidence IDs: `[E1]`, `[E2]`, etc.
- Tool name patterns: `_summary`, `_snapshot`, `_probe`
- Command names: `journalctl`, `systemctl`, `nmcli`, `ip `, `btrfs `

**5. Case Files Store Both Transcripts**
```
/var/lib/anna/cases/<case_id>/
├── case.json      # Full case state
├── human.log      # What the user saw (no internals)
└── debug.log      # Full trace for debugging
```

### Files Added

- `transcript_v068.rs` (NEW): TranscriptEventV2, TranscriptRole, TranscriptStreamV2, render_human(), render_debug(), write_transcripts(), validate_human_output()

### Files Changed

- `lib.rs`: Added v0.0.68 module and exports

### Unit Tests Added

- `test_human_mode_no_evidence_ids` - Evidence IDs hidden
- `test_debug_mode_has_evidence_ids` - Evidence IDs visible
- `test_internal_roles_hidden_in_human_mode` - Translator/Junior/Annad hidden
- `test_reliability_line_human_mode` - Humanized reliability display
- `test_validate_human_output` - Forbidden term detection

---

## v0.0.67 - Department Evidence Playbooks v1

**Release Date:** 2025-12-04

### Summary

Introduces evidence-based playbooks for Networking and Storage departments. Each playbook defines topics to investigate, collects structured evidence, and produces diagnoses with cause classification. Human Mode shows findings without tool names or IDs; Debug Mode shows commands and technical details. ActionProposal provides safe, rollbackable action templates with tiered confirmation gates.

### The Problem This Solves

Before v0.0.67:
```
[networking] Looking at network...
[networking] IP: 192.168.1.100, DNS: 8.8.8.8
```

After v0.0.67:
```
[networking] Checking link state, IP configuration, DNS, and network manager status.
[networking] Findings:
  - WiFi connection is active
  - IP address configured, internet route present
  - 2 DNS server(s) configured
  - NetworkManager active (connected)
[networking] Cause: Unknown (all checks passed). Confidence: 30%
```

### Key Features

**1. PlaybookTopic Framework**
```rust
PlaybookTopic {
    id: "net_link",
    title: "Link State",
    why_it_matters: "Physical or wireless link must be UP",
    tool_steps: ["ip link show", "/sys/class/net"],
    required: true,
}
```

**2. PlaybookEvidence with Dual Summaries**
```rust
PlaybookEvidence {
    topic_id: "net_link",
    summary_human: "WiFi connection is active",  // No tool names!
    summary_debug: "wlan0: UP (operstate=up, carrier=true)",
    raw_refs: ["ip link show", "/sys/class/net/wlan0/operstate"],
}
```

**3. PlaybookBundle with Coverage Scoring**
- Tracks which required topics have evidence
- Calculates coverage_score (0-100%)
- Lists missing_topics for retry logic

**4. Cause Classification**
- NetworkCauseCategory: Link, Dhcp, Dns, ManagerConflict, DriverFirmware, Unknown
- StorageRiskLevel: None, Low, Medium, High
- Confidence score based on evidence quality

**5. ActionProposal with Confirmation Gates**
| Risk Level | Confirmation Phrase | Example |
|------------|---------------------|---------|
| read_only  | (none)              | flush DNS cache |
| low        | "y"                 | restart NetworkManager |
| medium     | "confirm"           | install package |
| high       | "I assume the risk" | switch WiFi backend |

**6. Pre-built Action Templates**
- `networking_actions::restart_networkmanager()`
- `networking_actions::switch_to_iwd()`
- `storage_actions::btrfs_scrub_start(mount)`
- `storage_actions::dangerous_btrfs_check_repair(mount)`

### Files Added

- `evidence_playbook.rs` (NEW): PlaybookTopic, PlaybookEvidence, PlaybookBundle, NetworkCauseCategory, StorageRiskLevel
- `action_proposal.rs` (NEW): ActionProposal with builders, networking_actions, storage_actions
- `networking_playbook.rs` (NEW): LinkEvidence, AddrRouteEvidence, DnsEvidence, ManagerEvidence, collect_* functions, run_networking_playbook()
- `storage_playbook.rs` (NEW): MountEvidence, BtrfsEvidence, SmartEvidence, IoErrorsEvidence, FstabEvidence, collect_* functions, run_storage_playbook()

### Files Changed

- `lib.rs`: Added v0.0.67 modules and exports

### Unit Tests Added

- `test_playbook_topic` - topic construction
- `test_playbook_evidence` - evidence creation
- `test_playbook_bundle_coverage` - coverage calculation
- `test_human_summary_no_tool_names` - human mode contract
- `test_network_cause_category` - cause labels
- `test_storage_risk_level` - risk labels
- `test_action_proposal_read_only` - no confirmation needed
- `test_action_proposal_low_risk` - "y" confirmation
- `test_action_proposal_high_risk` - "I assume the risk" confirmation
- `test_networking_actions` - action templates
- `test_storage_actions` - action templates
- `test_human_description_no_commands` - human mode
- `test_debug_description_has_commands` - debug mode
- `test_networking_topics` - topic definitions
- `test_classify_link_down` - cause classification
- `test_classify_no_ip` - DHCP detection
- `test_storage_topics` - storage topics
- `test_fstab_parsing` - fstab evidence
- `test_btrfs_device_error` - BTRFS error detection

---

## v0.0.66 - Service Desk Dispatcher + Department Protocol

**Release Date:** 2025-12-04

### Summary

Makes the "IT department" structure real in the architecture. Service Desk receives requests and routes to specialized departments (Networking, Storage, Audio, Boot, Graphics) with a shared case file. Each request creates a case folder with case.json, human.log, and debug.log. Department trait enables structured investigations with findings, evidence, and reliability hints. Human Mode shows fly-on-the-wall dialogue; Debug Mode shows full traces.

### The Problem This Solves

Before v0.0.66:
```
[you] wifi keeps disconnecting
[anna] *runs random tools*
[anna] Here's some network info...
```

After v0.0.66:
```
[you -> anna] wifi keeps disconnecting
[service desk] Opening ticket #A-20251204-1234. Routing to Networking.
[networking] Checking link state, IP, route, and DNS configuration.
[networking] Link is up, IP assigned, route exists, DNS configured.
[service desk] Summary: Network stack appears healthy. Reliability: 88%
```

### Key Features

**1. Enhanced CaseFile Schema**
- `TicketType`: Question, Incident, ChangeRequest
- `CaseOutcome`: Pending, Answered, NeedsConfirmation, BlockedByPolicy, InsufficientEvidence, Abandoned
- `human_transcript[]`: Human Mode lines (no tool names, no evidence IDs)
- `evidence_summaries[]`: Human-readable evidence descriptions
- `debug_trace_path`: Path to debug.log file

**2. Case Folder Structure**
```
/var/lib/anna/cases/<case_id>/
├── case.json      # Full case state
├── human.log      # Fly-on-the-wall dialogue
└── debug.log      # Full trace with evidence IDs
```

**3. Department Protocol**
- `DepartmentTrait`: name(), can_handle(), required_topics(), investigate()
- `DepartmentName`: ServiceDesk, Networking, Storage, Audio, Boot, Graphics, InfoDesk
- `DepartmentResult`: findings, evidence_bundle, recommended_actions, reliability_hint
- `WorkOrder`: department, goals, required_topics, max_escalations

**4. Service Desk Enhancements**
- `detect_ticket_type()`: Question vs Incident vs ChangeRequest
- `create_work_order()`: Goals and required evidence topics
- `create_routing_decision()`: Department, confidence, reason

**5. Department Tone Tags**
Each department has personality in Human Mode:
- Service Desk: calm, structured, slightly terse
- Networking: investigative, skeptical, "prove it"
- Storage: cautious, rollback-minded
- Audio: practical, checks stack/services/devices
- Graphics: driver-aware, configuration-sensitive
- Boot: timeline-focused, service-aware

**6. Multi-Department Escalation (v1)**
- `WorkOrder.can_escalate()` and `record_escalation()`
- Max 1 additional department consultation
- `DepartmentResult.needs_escalation` and `escalation_target`

### Files Changed

- `case_lifecycle.rs`: TicketType, CaseOutcome, save_human_log(), save_debug_log(), save_all()
- `department_protocol.rs` (NEW): DepartmentTrait, DepartmentName, RoutingDecision, WorkOrder, DepartmentResult
- `service_desk.rs`: detect_ticket_type(), create_work_order(), create_routing_decision(), DispatchResult extended
- `narrator.rs`: ActorVoice extended with department voices, get_investigation_message(), format_department_message()
- `config.rs`: UiConfig.show_spinner
- `evidence_record.rs`: EvidenceBundle::empty()
- `lib.rs`: New exports for department protocol

### Integration Tests Added

- `test_annactl_network_routes_to_networking_department`
- `test_annactl_disk_routes_to_storage_department`
- `test_annactl_human_mode_no_evidence_ids`
- `test_annactl_debug_mode_has_details`
- `test_annactl_department_tone_visible`

---

## v0.0.65 - Evidence Topics v1 + Answer Shaping

**Release Date:** 2025-12-04

### Summary

Fixes "obviously wrong answers" where disk queries got CPU info. Introduces typed evidence system (EvidenceRecord, EvidenceBundle) that forces relevance. Evidence Router prevents hw_snapshot_summary from satisfying specific queries. AnswerShaper generates topic-appropriate responses. Junior verification now penalizes irrelevant tools (-40%). Audio/network queries don't overclaim "working" status.

### The Problem This Solves

Before v0.0.65:
```
[you] how much disk space do I have?
[anna] You have an AMD Ryzen 9... (CPU info)  # WRONG - generic hw_snapshot!
```

After v0.0.65:
```
[you] how much disk space do I have?
[anna] Root filesystem: 234 GiB total, 156 GiB free (67%)  # CORRECT - mount_usage!
```

### Key Features

**1. Typed Evidence System (EvidenceRecord, EvidenceBundle)**
- `ProbeKind`: Passive (read-only) vs Active (traffic-generating)
- `EvidenceRecord`: topic, human_summary, debug_source, evidence_id, raw_data, timestamp, success, probe_kind, duration_ms
- `EvidenceBundle`: records, primary_topic, complete, missing_topics
- `EvidenceSchema`: defines required/optional fields per topic

**2. Evidence Router**
- `route_evidence()`: Determines topics and tools from request text
- `tool_satisfies_topic()`: **Key gatekeeper** - blacklists generic tools from specific queries
- `get_tool_for_topic()`: Returns correct tool for each evidence topic
- Generic tools (hw_snapshot_summary, sw_snapshot_summary) can ONLY satisfy GraphicsStatus/Unknown

**3. AnswerShaper**
- `shape_answer()`: Generates topic-specific response text
- `ShapedAnswer`: text, topic, complete, confidence_notes, evidence_summary, debug_evidence
- Audio/network confidence_notes prevent overclaiming "working" status
- `format_human_answer()` / `format_debug_answer()` for output modes

**4. Junior Verification Upgrade**
- `check_tool_relevance()`: Validates evidence matches query topic
- `IRRELEVANT_TOOL_PENALTY`: -40% (caps answer at 45%)
- `verify_answer_with_topic()`: Topic-aware verification
- Wrong evidence type now properly penalized

**5. Human Mode Evidence Descriptions**
- Evidence shown as human-readable summaries, not tool names/IDs
- Example: "Disk usage from mount points" not "[mount_usage]"

### Evidence Topic Routing

| Query | Topic | Tool | NOT Satisfied By |
|-------|-------|------|------------------|
| "disk space" | DiskFree | mount_usage | hw_snapshot_summary |
| "kernel version" | KernelVersion | kernel_version | sw_snapshot_summary |
| "memory" | MemoryUsage | memory_info | hw_snapshot_summary |
| "audio working" | AudioStatus | audio_status | hw_snapshot_summary |
| "network" | NetworkStatus | network_status | hw_snapshot_summary |

### Confidence Notes (No Overclaiming)

Audio and network queries now include confidence notes:
```
AudioStatus: "Audio device detection doesn't confirm playback is working"
NetworkStatus: "Interface status doesn't confirm end-to-end connectivity"
```

### Files Changed

- `evidence_record.rs` (NEW): ProbeKind, EvidenceRecord, EvidenceBundle, EvidenceSchema
- `evidence_router.rs` (NEW): route_evidence(), tool_satisfies_topic(), get_tool_for_topic()
- `answer_shaper.rs` (NEW): ShapedAnswer, shape_answer(), format_human_answer(), format_debug_answer()
- `junior_rubric.rs`: check_tool_relevance(), verify_answer_with_topic(), IRRELEVANT_TOOL_PENALTY
- `lib.rs`: New module declarations and exports
- `cli_tests.rs`: v0.0.65 regression tests (disk, kernel, network, audio queries)

### Regression Tests Added

- `test_annactl_disk_query_not_cpu_info`: Disk queries must not return CPU info
- `test_annactl_kernel_query_not_hw_summary`: Kernel queries must not return hw blob
- `test_annactl_network_query_mentions_details`: Network queries must mention interface details
- `test_annactl_audio_query_no_overclaim`: Audio queries must not overclaim "working"

---

## v0.0.64 - Service Desk Dispatcher + Doctor Lifecycle

**Release Date:** 2025-12-04

### Summary

Every request now flows through a Service Desk dispatcher that creates a Ticket (A-YYYYMMDD-XXXX) with category and severity. Problem reports ("wifi down", "no sound") route to specialist Doctors, while informational queries use the Evidence Topic Router. Human Mode narration describes tickets, routing, and doctor actions like an IT department conversation.

### The IT Department Model

```
[anna] Opening ticket #A-20251204-1234. Triage: Networking. Severity: high.
[anna] → Routing this to Network Doctor team.
[network_doctor] Recording symptoms: connectivity issues, wifi keeps disconnecting
[network_doctor] Checking: interface state, wifi signal, DNS resolution
[network_doctor] Diagnosis: Possible interference or driver issue on wlan0
```

### Key Features

**1. Service Desk Dispatcher**
- `dispatch_request()` runs FIRST, creates Ticket and RoutingPlan
- Ticket ID format: A-YYYYMMDD-XXXX (e.g., A-20251204-1234)
- TicketSeverity: Low, Medium, High, Critical
- TicketCategory: Networking, Storage, Audio, Boot, Graphics, Security, Performance, Services, Packages, General

**2. Problem Detection**
- `is_problem_report()` detects problem vs informational queries
- Problem keywords: down, broken, not working, failed, error, slow, dead, no sound, no internet, etc.
- Informational prefixes: "what is", "how much", "show me", "list", etc.

**3. Severity Heuristic**
- Critical: "completely", "total", "nothing works"
- High: "down", "dead", "no internet", "no sound", "not booting"
- Medium: Problem report without outage keywords
- Low: Informational query

**4. Doctor Lifecycle Stages**
- Intake: Record symptoms and checking points
- EvidenceGathering: Collect diagnostic data
- Diagnosis: Analyze findings
- Planning: Propose next steps
- Verification: Confirm diagnosis
- HandOff: Return to user

**5. Human Mode Narration**
- TicketOpened: "Opening ticket #A-20251204-1234. Triage: Networking. Severity: high."
- RoutingDecision: "Routing this to Network Doctor team."
- DoctorStage: "[network_doctor] Recording symptoms: connectivity issues"

### Routing Logic

| Request Type | Route To | Example |
|-------------|----------|---------|
| Problem report + network keywords | Network Doctor | "wifi keeps disconnecting" |
| Problem report + audio keywords | Audio Doctor | "no sound from speakers" |
| Informational query | Evidence Topic Router | "how much memory do I have" |
| General query | ServiceDesk (no doctor) | "what's the status" |

### Files Changed

- `service_desk.rs`: Ticket, TicketSeverity, TicketCategory, RoutingPlan, HumanNarrationPlan, dispatch_request()
- `doctor_lifecycle.rs`: DoctorLifecycleStage, IntakeResult, DoctorLifecycleState, enhanced Doctor trait
- `case_lifecycle.rs`: ticket_id, ticket_category, ticket_severity, doctor_stages_completed fields
- `narrator.rs`: TicketOpened, RoutingDecision, DoctorStage events in Human and Debug modes
- `pipeline.rs`: Service Desk dispatcher integration before translator
- `lib.rs`: New exports for dispatch types

---

## v0.0.63 - Deterministic Evidence Router + Strict Validation

**Release Date:** 2025-12-04

### Summary

Fixes "obviously wrong answers" where questions like "how much memory do I have?" would get CPU/GPU blob summaries instead of memory information. The deterministic topic router now runs BEFORE the LLM and ensures the correct domain tools are selected.

### The Problem This Solves

Before v0.0.63:
```
[you] how much memory do I have?
[anna] (returns hw_snapshot_summary with CPU/GPU info)  # WRONG!
```

After v0.0.63:
```
[you] how much memory do I have?
[anna] You have 32 GiB total memory, 12.5 GiB available.  # CORRECT!
```

### Key Features

**1. Deterministic Topic Router**
- `detect_topic()` runs BEFORE any LLM call
- Routes common questions to the correct domain tools:
  - "how much memory" → `memory_info` tool
  - "disk space free" → `mount_usage` tool
  - "kernel version" → `kernel_version` tool
  - "network working" → `network_status` tool
  - "audio working" → `audio_status` tool

**2. Strict Answer Validation (40% Cap)**
- If answer doesn't contain expected data, reliability is CAPPED at 40%
- A memory question with GPU-only answer can never exceed 40%
- Uses `cap_reliability()` function for enforcement

**3. Evidence Freshness Tracking**
- Evidence older than 5 minutes gets freshness penalty
- 5-30 min: -5%, 30-60 min: -10%, 1-2 hours: -15%, >2 hours: -20%
- `with_evidence_freshness()` updates validation with age penalty

**4. Human Mode Narration**
- `topic_evidence_narration()` describes collected evidence without tool names
- Example: "To answer your memory question, I gathered: memory usage report."

### Acceptance Criteria (5 Common Questions)

| Question | Expected Topic | Required Content |
|----------|---------------|------------------|
| "how much memory do I have" | MemoryInfo | GiB/GB, total/available |
| "how much disk is free" | DiskFree | GiB/GB, %, mount points |
| "what kernel am I running" | KernelVersion | 6.x.x-arch, linux |
| "is my network working" | NetworkStatus | connected/interface/wifi |
| "is my audio working" | AudioStatus | pipewire/pulseaudio/alsa |

### Files Changed

- `evidence_topic.rs`: TopicValidation extended, cap_reliability, freshness tracking
- `pipeline.rs`: Topic-based routing in classify_intent_deterministic
- `human_labels.rs`: topic_evidence_narration, audio_status label
- `lib.rs`: New exports for freshness functions
- `cli_tests.rs`: Integration tests for 5 common questions

---

## v0.0.62 - Human Mode vs Debug Mode

**Release Date:** 2025-12-04

### Summary

Two distinct output modes for different needs. Human Mode (default) shows a professional IT department dialogue without exposing internal implementation details. Debug Mode shows everything for troubleshooting.

### Human Mode (Default)

What you see:
```
[you] to [anna]: how much disk space is free

[translator] I've reviewed your request. SystemQuery query, ReadOnly risk. (92% confident)

[anna] Let me gather the relevant information.
[annad] Evidence: disk usage for mounted filesystems

[junior] Reliability 94%. Verified.

[anna] to [you]: Free space on /: 50 GiB (40% free).

Reliability: 94%
Based on: disk usage snapshot
```

What you DON'T see:
- Tool names (no `mount_usage`, `hw_snapshot_summary`)
- Evidence IDs (no `[E1]`, `[E2]`)
- Raw JSON or prompts
- Parse errors or internal warnings
- Internal struct dumps

### Debug Mode (--debug)

Enable with:
- `annactl --debug "your request"`
- `ANNA_DEBUG=1 annactl "your request"`
- `ANNA_UI_TRANSCRIPT_MODE=debug`

Shows everything:
```
10:24:15.123 [classification] [translator]
    INTENT: SystemQuery
    TARGETS: disk, storage
    RISK: ReadOnly
    CONFIDENCE: 92%
    LLM_BACKED: true

10:24:15.145 [tool_result] tool=mount_usage OK [E1] (22ms)
10:24:15.200 [verification] [junior]
    SCORE: 94%
    CRITIQUE: (none)
```

### CLI Changes

```bash
annactl "request"           # Human Mode (default)
annactl --debug "request"   # Debug Mode
annactl --debug             # Debug Mode REPL
annactl status              # Shows current mode
```

### Status Page

`annactl status` now shows:
```
[CASES]
  Mode:       Human (professional IT dialogue, no internals)
              To enable debug: annactl --debug or ANNA_DEBUG=1
```

### New Files

- `narrator.rs` - Narrator component with IT department voices

### Technical Details

New types:
- `NarratorEvent` - All events the narrator can describe
- `ActorVoice` - IT department role voices (Anna, Translator, Junior, etc.)

Functions:
- `get_output_mode()` - Returns current TranscriptMode
- `is_debug_mode()` - Quick check for debug mode
- `narrate(event)` - Renders event based on current mode
- `topic_evidence_description(topic)` - Human-readable evidence description

### Tests

25 new tests ensuring:
- Human Mode hides evidence IDs and tool names
- Debug Mode exposes all internal details
- Mode switching via env vars works correctly
- All actor voices produce valid output

---

## v0.0.61 - Evidence Topics (Targeted Answers)

**Release Date:** 2025-12-04

### Summary

Answers now match the question. Pre-LLM topic detection ensures disk queries return disk info, kernel queries return kernel versions, and CPU queries return CPU info. No more nonsense answers mixing unrelated data.

### What This Means

**Before v0.0.61:**
```
User: "how much disk space is free"
Anna: [Evidence about CPU, GPU, or random system info]
```

**After v0.0.61:**
```
User: "how much disk space is free"
Anna: Free space on /: 50 GiB (40% free).
```

### How It Works

1. **detect_topic()** runs BEFORE the LLM translator
2. Deterministic pattern matching identifies 13 distinct topics:
   - CpuInfo, MemoryInfo, KernelVersion, DiskFree
   - NetworkStatus, AudioStatus, ServiceState, RecentErrors
   - BootTime, PackagesChanged, GraphicsStatus, Alerts, Unknown
3. Each topic has:
   - Required tools (e.g., DiskFree requires `mount_usage`)
   - Required fields (e.g., DiskFree requires `root`)
   - Answer template (e.g., "Free space on /: X GiB (Y% free)")
4. Junior verification penalizes mismatched answers

### Technical Details

New types in `anna_common`:
- `EvidenceTopic` enum - closed set of 13 topics
- `TopicConfig` - required_tools, required_fields, evidence_description
- `TopicDetection` - topic, confidence, secondary, service_name, is_diagnostic
- `TopicValidation` - topic_match, missing_fields, answer_contains_value, penalty

Functions:
- `detect_topic(request)` - returns TopicDetection with confidence 0-100
- `get_topic_config(topic)` - returns TopicConfig for routing
- `generate_answer(topic, data)` - returns templated answer string
- `validate_evidence(topic, data, answer)` - returns TopicValidation

### New File

- `evidence_topic.rs` - Core topic detection, configs, and validation

---

## v0.0.60 - 3-Tier Transcript Rendering

**Release Date:** 2025-12-04

### Summary

Transcript output is now human-readable by default. The system supports three rendering modes: `human` (default), `debug`, and `test`. Human mode shows a professional IT department conversation without exposing internal tool names, evidence IDs, or raw prompts. Debug and test modes show full internal details for troubleshooting.

### What This Means

In Human mode (default):
```
----- investigation -----
[annad] I'm checking hardware inventory.
[annad] Evidence: hardware inventory snapshot
[anna] Your CPU is an AMD Ryzen 9 5900X with 12 cores.
```

In Debug mode:
```
10:24:15.123 [tool_call] [annad] tool=hw_snapshot_summary
10:24:15.145 [tool_result] [annad] tool=hw_snapshot_summary [E1] (22ms)
10:24:15.200 [final] [anna] Your CPU is an AMD Ryzen 9 5900X with 12 cores.
```

### Configuration

Set transcript mode via:
1. Environment variable (highest priority): `ANNA_UI_TRANSCRIPT_MODE=debug`
2. Config file `/etc/anna/config.toml`:
   ```toml
   [ui]
   transcript_mode = "human"  # human|debug|test
   ```
3. Default: `human`

### New Files

- `transcript_events.rs` - Event bus for all transcript steps
- `human_labels.rs` - Registry mapping tools to human descriptions
- `transcript_renderer.rs` - Mode-aware rendering system
- `transcript_mode_tests.rs` - Integration tests

### Status Display

The `[CASES]` section in `annactl status` now shows:
```
[CASES]
  Transcript: human
  Active:     1 case(s) in progress
```

---

## v0.0.59 - Auto-Case Opening + Departmental IT Org

**Release Date:** 2025-12-04

### Summary

Anna now operates like a structured IT department. Every request creates a case with explicit lifecycle stages, and the Service Desk routes to specialist departments based on keyword analysis and alert context. The fly-on-the-wall transcript shows professional departmental handoffs, Junior QA verification, and structured findings with evidence - no theater, just professional diagnostic flow.

### What This Means

Before v0.0.59:
```
[anna] Checking network...
[anna] Here's what I found...
```

After v0.0.59:
```
=== Case net-001 ===

----- intake -----
[you] my wifi keeps disconnecting

----- triage -----
[translator] Intent: diagnose. Targets: [network_status].
[anna] Assigning to [networking] due to targets: network_status

----- investigation -----
[annad] Collected 3 evidence item(s): [E1, E2, E3]
[junior] Coverage 85%. Below threshold. May need additional evidence.

----- diagnosis -----
[networking] Findings:
  - WiFi signal strength: -72 dBm (weak)
  - Frequent reconnects in journal
[networking] Hypotheses:
  - [H1] Distance from AP causing instability (75% confidence)

----- verification -----
[junior] Reliability 78%. Coverage 85%. Acceptable. Ship it.

Case: net-001 | Status: plan_ready | Coverage: 85% | Reliability: 78%
```

### Case Lifecycle Stages

| Stage | Description |
|-------|-------------|
| `new` | Just created, not yet triaged |
| `triaged` | Intent classified, department assigned |
| `investigating` | Gathering evidence, running diagnostics |
| `plan_ready` | Diagnosis complete, action plan ready |
| `awaiting_confirmation` | Mutations proposed, waiting for user |
| `executing` | User confirmed, executing mutations |
| `verifying` | Checking results after mutation |
| `resolved` | Successfully completed |
| `abandoned` | User stopped or cannot proceed |

### Departments

| Department | Keywords | Doctor Domain |
|------------|----------|---------------|
| `networking` | wifi, dns, ethernet, route | Network |
| `storage` | disk, space, mount, btrfs | Storage |
| `boot` | boot, startup, systemd, slow | Boot |
| `audio` | sound, pipewire, speaker | Audio |
| `graphics` | gpu, wayland, screen, display | Graphics |
| `security` | permission, sudo, ssh, key | System |
| `performance` | slow, cpu, memory, thermal | System |
| `service_desk` | General fallback | - |

### Key Features

**Case Lifecycle (`case_lifecycle.rs`):**
- `CaseFileV2` schema with full lifecycle tracking
- `CaseStatus` enum with 9 explicit states
- `Department` enum with 8 IT departments
- `Participant` tracking (you, anna, translator, junior, annad, specialists)
- `TimelineEvent` append-only audit trail
- `ProposedAction` for mutation planning
- Alert linking via `linked_alert_ids`
- `count_active_cases()` for status display

**Service Desk (`service_desk.rs`):**
- `triage_request()` with keyword-based department scoring
- Confidence scoring (0-100) for routing decisions
- `find_linked_alerts()` matches queries to active alerts
- `dispatch_to_specialist()` maps department to DoctorRegistry
- `open_case_for_alert()` for alert-triggered cases
- `progress_case_*` functions for lifecycle transitions

**Transcript v2 (`transcript_v2.rs`):**
- `TranscriptBuilder` for structured output
- `DepartmentOutput` with:
  - Findings (bullets)
  - Evidence IDs
  - Hypotheses (labeled, with confidence, evidence-backed)
  - Next checks (read-only, auto-run)
  - Action plan (mutations gated)
- `render_case_transcript()` with phase separators
- `render_handoff()` for departmental assignments
- `render_junior_disagreement()` when evidence insufficient
- `render_collaboration()` for multi-doctor consultation

### Transcript Participants

| Actor | Role | Style |
|-------|------|-------|
| `[you]` | User | Literal input |
| `[anna]` | Service Desk lead | Calm, structured |
| `[translator]` | Intake analyst | Terse, checklist |
| `[junior]` | QA/reliability | Skeptical, calls out gaps |
| `[annad]` | Operator | Robotic, facts only |
| `[networking]` | Specialist | Concise findings |
| `[storage]` | Specialist | Concise findings |
| `[boot]` | Specialist | Concise findings |
| `[audio]` | Specialist | Concise findings |
| `[graphics]` | Specialist | Concise findings |

### Design Principles

1. **Every request = case** - Full audit trail for every interaction
2. **Deterministic routing** - Keywords + targets + alerts → department
3. **Evidence-first** - All findings backed by [E#] references
4. **Professional tone** - No jokes, no emojis, no theater
5. **Structured output** - Findings, hypotheses, actions in clear format
6. **Junior verification** - QA checkpoint with coverage/reliability gates

### Files Added

- `crates/anna_common/src/case_lifecycle.rs` - Case schema and lifecycle
- `crates/anna_common/src/service_desk.rs` - Triage and routing
- `crates/anna_common/src/transcript_v2.rs` - Departmental transcript
- `crates/anna_common/src/case_lifecycle_tests.rs` - Integration tests

### Files Modified

- `crates/anna_common/src/lib.rs` - Module exports
- `crates/annactl/src/commands/status.rs` - Active cases count

---

## v0.0.58 - Proactive Monitoring Loop v1

**Release Date:** 2025-12-04

### Summary

Anna now notices problems without being asked - like a real sysadmin who spots issues before they become crises. This release introduces a daemon-owned **Proactive Alerts** system that detects 5 high-signal issue types with evidence trails. Alerts show in `annactl status`, and "show alerts" / "why are you warning me?" queries route to alert evidence.

### What This Means

Before v0.0.58:
```
User: annactl status
[ALERTS]
  Critical: 0
  Warnings: 0
```

After v0.0.58:
```
User: annactl status
[ALERTS]
  Critical:   1
  Warnings:   2
  Info:       0

  Active alerts:
    [CRITICAL] Service failed: nginx.service [E1] (3h)
    [WARNING] Disk pressure on /: 8.2 GiB free (6%) [E2] (1d)
    [WARNING] Boot regression: 45s (baseline 28s) [E3] (2h)

  Snapshot:   just now
```

### Alert Types

| Type | Trigger | Severity |
|------|---------|----------|
| **BOOT_REGRESSION** | Boot time > baseline + 2 stddev, delta >= 5s | Warning |
| **DISK_PRESSURE** | / free < 10% or < 15 GiB | Warning |
| **DISK_PRESSURE** | / free < 5% or < 5 GiB | Critical |
| **JOURNAL_ERROR_BURST** | >= 20 errors in 10 min per unit | Warning |
| **JOURNAL_ERROR_BURST** | >= 50 errors in 10 min per unit | Critical |
| **SERVICE_FAILED** | Any systemd unit in failed state | Critical |
| **THERMAL_THROTTLING** | CPU temp > 85C | Warning |
| **THERMAL_THROTTLING** | CPU temp > 95C | Critical |

### Key Features

**Alerts Subsystem (`proactive_alerts.rs`):**
- `ProactiveAlert` struct with stable ID (hash of type + dedupe_key)
- `AlertType` enum: BOOT_REGRESSION, DISK_PRESSURE, JOURNAL_ERROR_BURST, SERVICE_FAILED, THERMAL_THROTTLING
- `AlertSeverity`: Critical, Warning, Info
- `AlertStatus`: Active, Resolved
- `ProactiveAlertsState` with upsert/resolve/get_active/count_by_severity
- State persisted to `/var/lib/anna/internal/alerts.json`

**Alert Detectors (`alert_detectors.rs`):**
- `detect_boot_regression()` - compares to rolling baseline
- `detect_disk_pressure()` - checks / filesystem
- `detect_journal_error_burst()` - parses `journalctl -p err` for 10 min window
- `detect_service_failed()` - runs `systemctl --failed`
- `detect_thermal_throttling()` - reads hwmon sensors (coretemp/k10temp)
- `run_all_detectors()` - executes all 5 and returns alerts

**Daemon Probes (`alert_probes.rs`):**
- `probe_alerts_summary()` - for "show alerts" queries
- `probe_boot_time_summary()`, `probe_disk_pressure_summary()`
- `probe_journal_error_burst_summary()`, `probe_failed_units_summary()`
- `probe_thermal_summary()`

**Tool Routing (`system_query_router.rs`):**
- `QueryTarget::Alerts` routes to `proactive_alerts_summary`
- Patterns: "show alerts", "any warnings", "why are you warning me?"
- `validate_answer_for_target()` validates alert-related answers

**Status Integration (`status.rs`):**
- `[ALERTS]` section shows counts by severity
- Top 3 active alerts with evidence IDs and age
- Snapshot age indicator

### Design Principles

1. **No auto-fixing** - Only detection + reporting with evidence trail
2. **High-signal only** - 5 carefully chosen alert types, not noisy
3. **Daemon-owned** - State written by annad, read by annactl
4. **Evidence-linked** - Every alert has [E#] references
5. **Deduplicated** - Same alert type + key = upsert, not duplicate

### Files Added

- `crates/anna_common/src/proactive_alerts.rs` - Alert schema and state
- `crates/anna_common/src/alert_detectors.rs` - 5 detector implementations
- `crates/anna_common/src/alert_probes.rs` - Daemon probes for detection

### Files Modified

- `crates/anna_common/src/lib.rs` - Module exports
- `crates/anna_common/src/tools.rs` - Tool catalog entries
- `crates/anna_common/src/tool_executor.rs` - Tool executors
- `crates/anna_common/src/system_query_router.rs` - Alert routing
- `crates/anna_common/src/evidence_coverage.rs` - Alerts target facets
- `crates/anna_common/src/evidence_tools.rs` - Alerts evidence planning
- `crates/annactl/src/commands/status.rs` - Status display

---

## v0.0.57 - Evidence Coverage + Correct Tool Routing

**Release Date:** 2025-12-04

### Summary

This release fixes a critical bug where Anna would answer disk/memory/network queries with unrelated evidence (e.g., answering "how much disk space" with CPU info). A new **Evidence Coverage** system ensures the RIGHT evidence backs each claim, and Junior's rubric now aggressively penalizes wrong or missing evidence.

### The Bug We Fixed

Before v0.0.57:
```
User: how much disk space is free?
Anna: You have an AMD Ryzen 7 5800X [E1] with 8 cores.
Junior: Reliability 85%. Ship it.  <-- WRONG!
```

After v0.0.57:
```
User: how much disk space is free?
[junior]: Coverage 0%. Evidence doesn't include disk_free. Missing: root_fs_free
[anna]: Coverage too low. Pulling mount_usage for disk_free evidence.
Anna: You have 433 GiB free on / [E1].
Junior: Reliability 92%. Ship it.
```

### Key Features

**Evidence Coverage (`evidence_coverage.rs`):**
- `TargetFacets` struct defines required/optional fields per query target
- `analyze_coverage()` computes coverage % by checking if evidence contains required fields
- `check_evidence_mismatch()` detects when evidence is for the wrong target entirely
- `get_gap_filling_tools()` suggests which tools to run to fill coverage gaps
- `COVERAGE_SUFFICIENT_THRESHOLD = 90` - below this, retry is triggered
- `COVERAGE_PENALTY_THRESHOLD = 50` - below this, Junior caps score at 50%

**Target Facet Map:**
| Target | Required Fields | Providing Tools |
|--------|----------------|-----------------|
| cpu | cpu_model, cores | hw_snapshot_cpu, hw_snapshot_summary |
| memory | mem_total | memory_info, mem_summary |
| disk_free | root_fs_free, root_fs_total | mount_usage, disk_usage |
| kernel_version | kernel_release | kernel_version, uname_summary |
| network_status | link_state, has_ip | network_status, nm_summary |

**Junior Rubric v2 (`junior_rubric.rs`):**
- `verify_answer()` produces `VerificationResult` with coverage analysis
- Wrong evidence type → reliability capped at **20%**
- Missing required fields → reliability capped at **50%**
- Answer doesn't match question → reliability capped at **60%**
- Uncited claims → **-10 points** per claim
- High coverage (≥95%) → **+10 bonus** points

**Transcript Integration:**
- `render_junior_coverage_check()` shows coverage in fly-on-wall log
- `render_anna_coverage_retry()` shows retry dialogue when fetching more evidence
- Example: `[junior] to [anna]: Coverage 30%. Evidence doesn't include disk_free.`

**Case File Extensions:**
- `evidence_coverage_percent: u8` - coverage score stored in case
- `missing_evidence_fields: Vec<String>` - list of missing fields
- `evidence_retry_triggered: bool` - whether retry was needed

### Files Added

- `crates/anna_common/src/evidence_coverage.rs` - Coverage scoring and facet map
- `crates/anna_common/src/junior_rubric.rs` - Evidence-based verification rubric
- `crates/anna_common/src/evidence_coverage_tests.rs` - 20+ integration tests

### Files Modified

- `crates/anna_common/src/case_file_v1.rs` - Added coverage fields
- `crates/anna_common/src/dialogue_renderer.rs` - Added coverage display functions
- `crates/anna_common/src/lib.rs` - Added module exports
- `README.md` - Updated version and description
- `CLAUDE.md` - Updated version
- `TODO.md` - Added v0.0.57 section, next step for 0.0.58

### Integration Tests

Tests assert:
1. Disk query with CPU evidence → fails (reliability ≤ 20%)
2. Memory query with CPU evidence → fails
3. Kernel query with CPU evidence → fails
4. Network query with CPU evidence → fails
5. Correct evidence for each target → passes (reliability ≥ 85%)
6. Gap filling suggests correct tools
7. Empty evidence → fails
8. Multiple evidence sources → improves coverage

### Upgrade Notes

No breaking changes. The coverage system is additive - existing cases will have `evidence_coverage_percent: 0` which is harmless. The new Junior rubric only affects new verifications.

---

## v0.0.56 - Fly-on-the-Wall Dialogue Layer v1

**Release Date:** 2025-12-04

### Summary

This release introduces the **Fly-on-the-Wall Dialogue Layer** - transcripts now feel like eavesdropping on a real IT department. Each actor has a consistent voice: Anna is calm and evidence-focused, Translator is brisk and checklist-driven, Junior is skeptical QA, and Doctors feel like specialized departments with proper handoffs.

### Key Features

**DialogueRenderer (`dialogue_renderer.rs`):**
- Actor voices applied at render-time only (logic layer stays deterministic)
- `[anna]` - calm senior admin, concise, honest, dry humor, pushes for evidence
- `[translator]` - service desk triage, brisk, checklist-driven
- `[junior]` - skeptical QA, calls out missing evidence, disagrees when warranted
- `[annad]` - robotic/operational, terse, structured
- `DialogueContext` for tracking intent confidence, reliability, doctor domain

**Explicit Agree/Disagree Moments:**
- Translator classification with acknowledgment: "Acknowledged, I agree" vs "proceed carefully"
- Junior verification with QA signoff: "Ship it" vs "Not good enough"
- Anna responds to Junior's verdict appropriately

**Doctors as Departments:**
- Doctor actor names: `[networking-doctor]`, `[audio-doctor]`, `[storage-doctor]`, etc.
- Proper handoff dialogue: "You're up. Collect link state, routes, DNS..."
- Doctors initiate probe requests when present (not anna)

**Transcript Ergonomics:**
- Phase separators: `----- triage -----`, `----- evidence -----`, `----- verification -----`
- Evidence summaries: "Evidence collected: [E1, E2, E3]"
- QA-style reliability footer with verdict

**Uncertainty Handling:**
- `CONFIDENCE_CERTAIN_THRESHOLD = 80` - Below this, translator says "Not fully certain"
- `RELIABILITY_SHIP_THRESHOLD = 75` - Below this, junior says "Not good enough"
- Fallback handling: "Taking a conservative route"

**Golden Tests (`dialogue_golden_tests.rs`):**
- SYSTEM_QUERY tests: actors, evidence IDs, agree/disagree, no raw command spam
- DIAGNOSE tests: doctor handoff, multiple evidence, correct actors
- ACTION_REQUEST tests: evidence, confirmation mentions
- Low confidence tests: uncertainty expressions
- Low reliability tests: junior disagreement

### Sample Transcript

```
=== Case: cpu-query-001 ===

----- intake -----
[you] to [anna]: what cpu do i have

----- triage -----
[anna] to [translator]: What are we looking at?
[translator] to [anna]: Clear SYSTEM_QUERY request. 95% confidence.
[anna] to [translator]: Acknowledged, I agree.

----- evidence -----
[anna] to [annad]: Run the probes.
[annad]: [E1] hw_snapshot_cpu -> AMD Ryzen 7 5800X, 8 cores, 16 threads, 3.8GHz
[annad] to [anna]: Evidence collected: [E1]

----- verification -----
[anna] to [junior]: Check this before I ship it.
[junior] to [anna]: Reliability 92%. Solid evidence. Ship it.
[anna] to [junior]: Good. Shipping response.

----- response -----
[anna] to [you]:
  You have an AMD Ryzen 7 5800X [E1]. It has 8 cores and 16 threads.

Reliability: 92% - Verified.
(45ms)
```

### Files Added

- `crates/anna_common/src/dialogue_renderer.rs` - Dialogue rendering with actor voices
- `crates/anna_common/src/dialogue_golden_tests.rs` - Golden tests for transcript stability

### Files Modified

- `crates/anna_common/src/lib.rs` - Added module exports
- `README.md` - Updated version and description
- `CLAUDE.md` - Updated version
- `TODO.md` - Added v0.0.56 section
- `RELEASE_NOTES.md` - This file

### Upgrade Notes

No breaking changes. The Dialogue Renderer provides tone at render-time only - the Case Engine and Doctors remain deterministic. All existing functionality continues to work unchanged.

---

## v0.0.55 - Deterministic Case Engine + Doctor-first Routing

**Release Date:** 2025-12-04

### Summary

This release introduces the **Deterministic Case Engine** - a 10-phase state machine that orchestrates all request processing with complete audit trails. Every request flows through explicit phases from Intake to LearnRecipe, with deterministic intent classification and doctor routing.

### Key Features

**Case Engine State Machine (`case_engine.rs`):**
- `CasePhase` enum with 10 phases:
  1. Intake - Receive and validate request
  2. Triage - Classify intent (SYSTEM_QUERY, DIAGNOSE, ACTION_REQUEST, HOWTO, META)
  3. DoctorSelect - Select doctor for DIAGNOSE intents
  4. EvidencePlan - Plan which evidence to collect
  5. EvidenceGather - Execute tool calls
  6. SynthesisDraft - Generate answer draft
  7. JuniorVerify - Verify with Junior model
  8. Respond - Send response to user
  9. RecordCase - Persist case file
  10. LearnRecipe - Extract recipe if reliability >= 80%
- `CaseState` tracks phase, events, timings, evidence IDs
- `IntentType` enum for canonical intent classification

**Intent Taxonomy (`intent_taxonomy.rs`):**
- `classify_intent()` with pattern-based detection
- SYSTEM_QUERY: "what cpu", "disk space", "kernel version"
- DIAGNOSE: "wifi not working", "no sound", "slow boot"
- ACTION_REQUEST: "install nginx", "restart sshd"
- HOWTO: "how do I configure", "how can I"
- META: "status", "what can you do"
- Query target detection for correct tool routing
- Problem domain detection for doctor selection

**Evidence Tools (`evidence_tools.rs`):**
- `EvidencePlan` with list of `PlannedTool` entries
- `plan_evidence()` creates plan based on intent classification
- Correct tool routing:
  - CPU queries → hw_snapshot_cpu
  - Memory queries → memory_info
  - Disk queries → mount_usage
  - Kernel queries → kernel_version
- `validate_evidence_for_query()` ensures evidence matches target

**Case File Schema v1 (`case_file_v1.rs`):**
- `CaseFileV1` with complete audit fields:
  - Intent classification with confidence
  - Doctor selection (for DIAGNOSE)
  - Evidence records with tool names and summaries
  - Phase timings
  - XP gained
- Atomic save to `/var/lib/anna/cases/<case_id>/`:
  - `case.json` - Full case data
  - `summary.txt` - Human-readable summary
  - `transcript.log` - Readable transcript

**Recipe Extractor (`recipe_extractor.rs`):**
- Gate rules: reliability >= 80%, >= 2 evidence items, success
- `check_recipe_gate()` validates case eligibility
- `extract_recipe()` creates recipe from successful case
- `calculate_case_xp()` determines XP based on outcome

**Transcript Renderer (`transcript_render.rs`):**
- `[actor] to [actor]: message` format
- Phase separators: `--- Phase ---`
- Evidence citations: `[E#]` inline
- `render_compact_summary()` for status display
- `render_recent_cases()` for case list

### Files Added

- `crates/anna_common/src/case_engine.rs` - Case Engine state machine
- `crates/anna_common/src/intent_taxonomy.rs` - Intent classification
- `crates/anna_common/src/evidence_tools.rs` - Evidence planning
- `crates/anna_common/src/case_file_v1.rs` - Case file schema v1
- `crates/anna_common/src/recipe_extractor.rs` - Recipe extraction
- `crates/anna_common/src/transcript_render.rs` - Transcript rendering

### Files Modified

- `crates/anna_common/src/lib.rs` - Added new module exports
- `README.md` - Updated version and description
- `CLAUDE.md` - Updated version
- `TODO.md` - Added v0.0.55 section
- `RELEASE_NOTES.md` - This file

### Upgrade Notes

No breaking changes. The Case Engine provides the foundation for deterministic request processing but does not change existing behavior. Integration with the main pipeline will be completed in future versions.

---

## v0.0.54 - Action Engine v1 (Safe Mutations)

**Release Date:** 2025-12-04

### Summary

This release introduces **Action Engine v1** for safe, auditable mutations on Arch Linux. The engine executes mutation steps only after user confirmation, shows diff previews for file edits, and creates rollback records for every executed mutation.

### Key Features

**Action Engine Contract (`action_engine.rs`):**
- `ActionPlan` with risk, summary, steps, and confirmation phrase
- `ActionStep` with precheck_probes, verify_probes, rollback_hint, and evidence_ids
- `ActionType` enum: EditFile, WriteFile, DeleteFile, Systemd, Pacman
- `MutationRiskLevel`: Low, Medium, High, Destructive, Denied

**Risk Scoring (`action_risk.rs`):**
- `score_path_risk()`: /etc → medium, /etc/fstab → high, /proc → denied
- `score_systemd_risk()`: NetworkManager → medium, sshd → high, systemd-journald → denied
- `score_package_risk()`: install → low, remove → medium, kernel remove → denied
- `score_delete_risk()`: always high or destructive

**Confirmation Phrases (hardcoded safety contract):**
- Low: `yes`
- Medium: `I CONFIRM (medium risk)`
- High: `I CONFIRM (high risk)`
- Destructive: `I ACCEPT DATA LOSS RISK`

**Diff Preview Pipeline (`action_executor.rs`):**
- `generate_action_diff_preview()` with unified diff format
- `ActionDiffPreview` shows additions, deletions, truncation indicator
- Backup path displayed before execution
- Maximum 50 lines with "(truncated)" indicator

**Rollback Scaffolding:**
- `RollbackRecord` with steps, backups, restore_instructions, verifications
- Saved to `/var/lib/anna/cases/<case_id>/rollback.json`
- `BackupRecord` with original_path, backup_path, SHA256 hash
- Human-readable restore instructions generated automatically

**Step Execution:**
- Atomic writes: write to temp file, fsync, rename
- `execute_action_step()` for individual steps
- `execute_action_plan()` with confirmation validation
- Systemd operations via `systemctl`
- Pacman operations via `pacman -S --noconfirm --needed` / `pacman -Rs --noconfirm`

### Files Added/Modified

**New Files:**
- `crates/anna_common/src/action_engine.rs` - Action engine types and contracts
- `crates/anna_common/src/action_risk.rs` - Deterministic risk scoring
- `crates/anna_common/src/action_executor.rs` - Diff preview and step execution

**Modified Files:**
- `crates/anna_common/src/lib.rs` - Added action engine module exports
- `crates/anna_common/Cargo.toml` - Added sha2 dependency
- `crates/annactl/tests/cli_tests.rs` - Added action engine integration tests

### Risk Scoring Examples

| Path/Service/Package | Operation | Risk Level |
|---------------------|-----------|------------|
| /home/user/.bashrc | Edit | Low |
| /etc/hosts | Edit | Medium |
| /etc/fstab | Edit | High |
| /proc/any | Any | Denied |
| nginx.service | Restart | Low |
| NetworkManager | Restart | Medium |
| sshd | Stop | High |
| systemd-journald | Any | Denied |
| htop | Install | Low |
| htop | Remove | Medium |
| linux | Remove | Denied |

### Upgrade Notes

No breaking changes. The Action Engine provides infrastructure for safe mutations but does not change existing behavior. Integration with the pipeline for automatic action execution will be completed in future versions.

---

## v0.0.53 - Doctor Flow v1 (Interactive Diagnostics)

**Release Date:** 2025-12-04

### Summary

This release introduces **Doctor Flow v1** for interactive diagnostic flows. When users report problems ("wifi keeps disconnecting", "no sound", "slow boot"), Anna automatically triggers specialized diagnostic doctors that collect evidence, analyze findings, and produce actionable conclusions with full audit trails.

### Key Features

**Doctor Flow Orchestration (`doctor_flow.rs`):**
- `DoctorFlowExecutor` orchestrates diagnostic flows with evidence collection
- `DoctorFlowStep` tracks each step: Pending, Running, Success, Failed, Skipped
- `DoctorFlowResult` bundles evidence, diagnosis, and human-readable report
- `DoctorCaseFile` persists full diagnostic session for audit trails

**Problem Phrase Detection:**
- `detect_problem_phrase()` with confidence scoring (0-100)
- Multi-word phrases: "not working", "keeps disconnecting", "no sound", etc.
- Single-word indicators: "broken", "crashed", "slow", "stuck", etc.
- Triggers FixIt intent when confidence >= 25

**Interactive Diagnostic Flow:**
- Problem queries route to specialized doctors (Network, Audio, Storage, Boot, Graphics)
- Diagnostic plan shows steps before execution
- Evidence collected with unique IDs (N1, N2 for network, A1, A2 for audio, etc.)
- Human-readable transcript (no raw command spam)

**Case File Persistence:**
- Stored at `/var/lib/anna/cases/<case_id>/doctor.json`
- Steps executed with tool names and durations
- Findings with severity (Critical, Error, Warning, Info)
- Most likely cause and suggested next steps
- Reliability score for diagnosis confidence

**Doctor Report Format:**
- Diagnosis Confidence percentage
- Findings list with evidence references
- Most Likely Cause summary
- Suggested Next Steps
- Optional Fixes (with risk level and confirmation phrases)

### Files Added/Modified

**New Files:**
- `crates/anna_common/src/doctor_flow.rs` - Doctor flow orchestration (398 lines)

**Modified Files:**
- `crates/anna_common/src/lib.rs` - Added doctor_flow module and exports
- `crates/annactl/src/pipeline.rs` - Integrated doctor flow into pipeline routing
- `crates/annactl/tests/cli_tests.rs` - Added doctor flow integration tests

### Example Interaction

```
[you] to [anna]: wifi keeps disconnecting

[anna] to [annad]: Running diagnostic plan with 7 checks:
  1. Check network interfaces (net_interfaces_summary)
  2. Check routing table (net_routes_summary)
  3. Check DNS configuration (dns_summary)
  ...

[annad] to [anna]: [N1] net_interfaces_summary: Found 3 interfaces (2 up, 1 down)
[annad] to [anna]: [N2] net_routes_summary: Default route via 192.168.1.1
...

[anna] to [you]:
Doctor: NetworkingDoctor v2
Diagnosis Confidence: 75%

Findings:
  [WARNING] Wireless interface shows intermittent carrier drops [N1]
  [INFO] DNS configured via NetworkManager [N3]

Most Likely Cause: Wireless driver or access point instability

Suggested Next Steps:
  - Check dmesg for wireless driver errors
  - Try connecting to a different access point
```

### Upgrade Notes

No breaking changes. Problem queries now trigger diagnostic flows automatically.

---

## v0.0.52 - System Query Router v1 (Quality Sprint)

**Release Date:** 2025-12-04

### Summary

This release fixes critical bugs where system queries returned wrong data. Disk space queries were returning CPU info, kernel version queries were returning CPU info, and memory questions were being misclassified. The new System Query Router ensures each query type uses the correct evidence tool.

### Key Features

**System Query Router (`system_query_router.rs`):**
- `QueryTarget` enum: Cpu, Memory, DiskFree, KernelVersion, NetworkStatus, AudioStatus, ServicesStatus
- `ToolRouting` struct with required_tools, optional_tools, output_description
- `detect_target()` with confidence scoring (0-100) for pattern matching
- `get_tool_routing()` maps targets to domain-specific tools
- `validate_answer_for_target()` verifies answer content matches query

**Fixed Query-Tool Mappings:**
- Disk space queries → `mount_usage` (was `hw_snapshot_summary`)
- Kernel version queries → `kernel_version` (was `hw_snapshot_summary`)
- Memory queries → `memory_info` (was `hw_snapshot_summary`)
- Network queries → `network_status`
- Audio queries → `audio_status`
- CPU queries → `hw_snapshot_summary` (correct)

**Translator Contract Update:**
- Domain-specific tools listed in system prompt with examples
- Examples for memory, disk, kernel, network, audio queries
- `hw_snapshot_summary` reserved for general CPU/GPU queries only

**Junior Verification Upgrade:**
- `enforce_answer_target_correctness()` validates answer matches query target
- 50-point penalty for wrong-target answers
- Critique includes "WRONG TARGET" message with details
- Ensures disk queries don't return CPU info (and vice versa)

**Pattern Matching Examples:**
- "how much disk space" → DiskFree (95% confidence)
- "what kernel version" → KernelVersion (95% confidence)
- "how much memory" → Memory (95% confidence)
- "am I connected" → NetworkStatus (90% confidence)
- "is audio working" → AudioStatus (90% confidence)
- "is docker running" → ServicesStatus (85% confidence)

### Files Added/Modified

**New Files:**
- `crates/anna_common/src/system_query_router.rs` - Query routing logic (392 lines)

**Modified Files:**
- `crates/anna_common/src/lib.rs` - Added module and exports
- `crates/annactl/src/pipeline.rs` - Updated Translator prompt, classify_intent_deterministic(), added enforce_answer_target_correctness()

### Upgrade Notes

No breaking changes. Existing queries will now return more accurate results.

---

## v0.0.51 - Action Engine v1 (Systemd Service Operations)

**Release Date:** 2025-12-04

### Summary

This release introduces the **Action Engine v1** for safe systemd service operations. The engine provides probe, preview, apply, and rollback functionality for service management with risk-based confirmation requirements and automatic state verification.

### Key Features

**Systemd Action Engine (`systemd_action.rs`):**
- `ServiceOperation` enum: Start, Stop, Restart, Enable, Disable
- `RiskLevel` enum: Low, Medium, High, Denied
- `ServiceAction` struct with service, operation, reason
- `assess_risk()` for automatic risk classification based on service type
- `normalize_service_name()` adds .service suffix if needed

**Risk-Based Service Classification:**
- Low risk: Normal services (nginx, docker, etc.)
- Medium risk: Network services (NetworkManager, iwd, systemd-resolved, etc.)
- High risk: Critical services (sshd, display-manager, gdm, sddm, etc.)
- Denied: Core systemd services (systemd-journald, dbus, systemd-logind, etc.)

**Confirmation Phrases:**
- `LOW_RISK_CONFIRMATION = "I CONFIRM (low risk)"`
- `MEDIUM_RISK_CONFIRMATION = "I CONFIRM (medium risk)"` (shared with mutation_tools)
- `HIGH_RISK_CONFIRMATION = "I ASSUME THE RISK"`

**4 New Systemd Service Tools:**
- `systemd_service_probe_v1` - Probe current state (active, enabled, description, PID)
- `systemd_service_preview_v1` - Preview action with risk assessment and expected changes
- `systemd_service_apply_v1` - Apply action with pre/post state capture and verification
- `systemd_service_rollback_v1` - Rollback to pre-action state by case_id

**Evidence-Driven Execution:**
- Pre-state capture before action
- Post-state verification after action
- verify_message with success/warning status
- Rollback command included in apply result

**Rollback Infrastructure:**
- Rollback metadata at `/var/lib/anna/rollback/<case_id>/service_rollback.json`
- Records: case_id, service, operation, pre_state, post_state, timestamp
- Automatic state restoration on rollback

**Modular File Structure (< 400 lines each):**
- `systemd_action.rs` - Core types, operations, risk assessment (308 lines)
- `systemd_probe.rs` - Service probing functionality (169 lines)
- `systemd_apply.rs` - Preview and apply functions (355 lines)
- `systemd_rollback.rs` - Rollback functionality (150 lines)
- `systemd_tools.rs` - Tool executors (249 lines)

### Files Added/Modified

**New Files:**
- `crates/anna_common/src/systemd_action.rs`
- `crates/anna_common/src/systemd_probe.rs`
- `crates/anna_common/src/systemd_apply.rs`
- `crates/anna_common/src/systemd_rollback.rs`
- `crates/anna_common/src/systemd_tools.rs`

**Modified Files:**
- `crates/anna_common/src/lib.rs` - Module and export declarations
- `crates/anna_common/src/tools.rs` - Tool definitions for 4 new tools
- `crates/anna_common/src/tool_executor.rs` - Tool dispatch

### Tests

Unit tests in `systemd_action.rs`:
- `test_normalize_service_name` - Service name normalization
- `test_risk_assessment_network` - Network service risk (medium)
- `test_risk_assessment_critical` - Critical service risk (high)
- `test_risk_assessment_core` - Core systemd service risk (denied)
- `test_risk_assessment_normal` - Normal service risk (low)
- `test_confirmation_phrases` - Confirmation phrase mapping
- `test_operation_inverse` - Operation inverse mapping

---

## v0.0.50 - User File Mutations

**Release Date:** 2025-12-04

### Summary

This release introduces the **User File Mutations** system: the first real mutation execution capability for user-scope files with safe diff preview, confirmation flow, and rollback support. Only paths under $HOME are allowed, with symlink escape detection to prevent accessing system files via symlinks.

### Key Features

**User File Edit Primitive (`user_file_mutation.rs`):**
- `UserFileEditAction` struct with path, mode, line, key, value, separator
- `EditMode` enum: `AppendLine` (append a line) and `SetKeyValue` (add/update key=value)
- `VerifyStrategy` enum: `FileContains`, `HashChanged`, `None`
- Idempotent operations (skip if line/key already exists with same value)
- `check_path_policy()` for HOME-only enforcement

**Path Policy Enforcement:**
- Only paths under $HOME are allowed in v0.0.50
- Blocked prefixes: /etc, /usr, /var, /boot, /root, /proc, /sys, /dev, /run, /lib, /lib64, /bin, /sbin, /opt
- Symlink escape detection (symlinks pointing outside $HOME are blocked)
- `PathPolicyResult` with allowed, reason, evidence_id, resolved_path, is_symlink

**3 New File Edit Tools (`tools.rs` + `file_edit_tools.rs`):**
- `file_edit_preview_v1` - Read-only preview with unified diff, would_change flag, policy check
- `file_edit_apply_v1` - Apply changes with backup, verification, and rollback info
- `file_edit_rollback_v1` - Restore from backup by case_id

**Backup and Rollback Infrastructure:**
- Backups stored at `/var/lib/anna/rollback/<case_id>/backup/<sanitized_path>`
- `apply_result.json` records: success, path, mode, hashes, backup_path, verified
- Operations logged to `/var/lib/anna/internal/ops.log`
- Rollback restores original file and verifies hash match

**Confirmation Flow:**
- `USER_FILE_CONFIRMATION = "I CONFIRM (medium risk)"` required for apply
- `preview_id` must be provided (Junior will verify preview exists)
- Verification after apply (FileContains or HashChanged)
- Rollback command included in apply result

**Integration Tests:**
- Path policy tests (blocked paths, home paths, symlink escape)
- Preview tests (append, set_key_value, idempotent skip)
- Action validation tests

### Files Added/Modified

**New Files:**
- `crates/anna_common/src/user_file_mutation.rs` (~700 lines)
- `crates/anna_common/src/file_edit_tools.rs` (~450 lines)

**Modified Files:**
- `crates/anna_common/src/lib.rs` - Module and export declarations
- `crates/anna_common/src/tools.rs` - Tool definitions
- `crates/anna_common/src/tool_executor.rs` - Tool dispatch

---

## v0.0.49 - Doctor Lifecycle System

**Release Date:** 2025-12-03

### Summary

This release introduces the **Doctor Lifecycle System**: a unified interface for all diagnostic doctors with structured diagnostic plans, evidence-based diagnosis, and knowledge learning integration. The new NetworkingDoctorV2 demonstrates the full lifecycle with ordered checks, confidence-scored findings, and proposed actions with risk levels. Also fixes case file permissions for non-root users.

### Key Features

**Doctor Trait (`doctor_lifecycle.rs`):**
- `Doctor` trait defining lifecycle contract with id(), domains(), matches(), plan(), diagnose()
- `DiagnosticCheck` for ordered evidence collection with tool_name and params
- `CollectedEvidence` linking checks to gathered data with evidence IDs
- `DiagnosisFinding` with severity, evidence_ids, confidence scores, and tags
- `DiagnosisResult` with summary, most_likely_cause, findings, next_steps
- `ProposedAction` with risk levels (Low/Medium/High) and rollback commands
- `SafeNextStep` for read-only suggestions
- `DoctorRunner` for orchestrating evidence collection
- `DoctorReport` with render() for human-readable output

**NetworkingDoctorV2 (`networking_doctor_v2.rs`):**
- Implements Doctor trait for network diagnosis
- Ordered diagnostic plan: interfaces → routes → DNS → NM status → wireless → connectivity → errors
- Layer-by-layer analysis (link, routes, DNS, connectivity, wireless)
- Confidence-scored findings with severity levels
- Manager conflict detection
- Proposed actions with confirmation phrases and rollback

**6 New Network Evidence Tools (`doctor_network_tools.rs`):**
- `net_interfaces_summary` - Detailed interface info (state, carrier, MAC, IPs, wireless)
- `net_routes_summary` - Routing table (default gateway, route count)
- `dns_summary` - DNS config (servers, source, stub resolver status)
- `iw_summary` - Wireless status (SSID, signal dBm, quality, frequency)
- `recent_network_errors` - Network-specific journal errors and warnings
- `ping_check` - Single-packet connectivity test with latency

**Case File Permission Fix (`transcript.rs`):**
- Case files now save to user directory first (`~/.local/share/anna/cases/`)
- Falls back to system directory (`/var/lib/anna/cases/`) for daemon use
- Fixes "Permission denied" error for non-root users running annactl

**Knowledge Learning Integration:**
- `qualifies_for_learning()` method on DoctorReport
- Requires: reliability >= 90%, confidence >= 80%, evidence >= 3
- `tools_used()` and `targets()` methods for recipe creation

### Technical Details

**Doctor Lifecycle Flow:**
```
User Request → Translator → Doctor Selection → Plan → Evidence Collection → Diagnose → Report
```

**Finding Severity Levels:**
- Critical: Service-breaking issues requiring immediate attention
- Error: Significant problems affecting functionality
- Warning: Potential issues that should be monitored
- Info: Informational findings

**Action Risk Levels:**
- Low: Read-only or easily reversible (e.g., scan networks)
- Medium: Service restart or config change (e.g., restart NetworkManager)
- High: Destructive or hard to undo

### Files Changed

- **NEW** `doctor_lifecycle.rs` - Doctor trait and lifecycle types
- **NEW** `doctor_network_tools.rs` - Network evidence tool implementations
- **NEW** `networking_doctor_v2.rs` - Doctor trait implementation for network
- **MODIFIED** `transcript.rs` - Case file permission fix
- **MODIFIED** `tools.rs` - Added 6 new network tools
- **MODIFIED** `tool_executor.rs` - Dispatch for new tools
- **MODIFIED** `lib.rs` - Module exports
- **MODIFIED** `learning.rs` - Fixed test (XP title assertion)

---

## v0.0.48 - Learning System

**Release Date:** 2025-12-03

### Summary

This release introduces Anna's **learning system**: the ability to store learned recipes from successful interactions, search existing knowledge, track XP progression, and evolve from "Intern" to "Grandmaster" over time. The system is local-only, bounded (50 packs, 500 recipes max), and uses lightweight token-based matching instead of embeddings.

### Key Features

**Knowledge Pack v1 Format (`learning.rs`):**
- `KnowledgePack` struct with pack_id, name, version, source, tags, entries
- `LearnedRecipe` struct with intent, targets, triggers, actions, rollback, confidence rules
- `PackSource` enum: Learned (from cases), UserCreated, Imported
- `RecipeIntent` and `RecipeAction` for structured recipe definitions
- Storage at `/var/lib/anna/knowledge_packs/installed/*.json`
- Strict limits: MAX_PACKS=50, MAX_RECIPES_TOTAL=500, MAX_RECIPE_SIZE_BYTES=24KB

**Knowledge Search Tool:**
- `learned_recipe_search(query, limit)` - Search local recipes by token matching
- Lightweight BM25-style scoring without embeddings
- Returns `SearchHit` with recipe_id, title, score, pack_id, evidence_id
- Evidence ID format: K1, K2, etc. for knowledge citations

**Learning Pipeline:**
- `LearningManager` coordinates storage, retrieval, and learning
- Auto-creates monthly packs (`learned-pack-YYYYMM`)
- Recipe deduplication: if same intent+targets+tools exists, increment wins counter
- Minimum thresholds: reliability >= 90%, evidence_count >= 1
- Converts successful cases into reusable recipes

**XP System:**
- Non-linear XP curve: Level 1=100, 2=500, 3=1200, 4=2000, etc.
- Level 0-100 progression
- Title progression: Intern → Apprentice → Junior → Competent → Senior → Expert → Wizard → Grandmaster
- XP gains: +2 (85% reliability), +5 (90% reliability), +10 (recipe created)
- No XP loss (poor outcomes earn nothing)
- `learning_stats` tool for XP/level display

**2 New Tools:**
- `learned_recipe_search(query, limit)` - Search learned recipes
- `learning_stats()` - Get XP level, title, recipe count, pack count

**Transcript Updates:**
- `LearningRecord` struct with knowledge_searched, knowledge_query, recipes_matched
- Fields for recipe_written, recipe_id, xp_gained, level_after, title_after
- Integration with CaseFile for learning provenance

### Technical Details

**XP Curve Formula:**
```rust
// Level 1-20: exponential curve
match level {
    0 => 0,
    1 => 100,
    2 => 500,
    3 => 1200,
    4 => 2000,
    5 => 3000,
    6 => 4500,
    7 => 6500,
    8 => 9000,
    9 => 12000,
    10 => 16000,
    ...
    20 => 100_000,
    _ => 100_000 + (level - 20) * 1000,  // Linear after 20
}
```

**Token Matching Algorithm:**
1. Normalize query: lowercase, split on whitespace/punctuation
2. For each recipe: score = matches / max(query_tokens, recipe_tokens)
3. Boost for exact substring matches in title/intent
4. Return top K results above threshold (0.3)

**Storage Paths:**
- XP state: `/var/lib/anna/internal/xp_state.json`
- Knowledge packs: `/var/lib/anna/knowledge_packs/installed/*.json`

### Files Added/Modified

- **Added:** `crates/anna_common/src/learning.rs` - Core learning module (~350 lines)
- **Modified:** `crates/anna_common/src/tools.rs` - Added 2 learning tools
- **Modified:** `crates/anna_common/src/tool_executor.rs` - Added 2 tool implementations
- **Modified:** `crates/anna_common/src/transcript.rs` - Added LearningRecord
- **Modified:** `crates/anna_common/src/lib.rs` - Added learning module exports
- **Modified:** `scripts/anna_deep_test.sh` - Added run_learning_tests()
- **Modified:** Version files (Cargo.toml, CLAUDE.md, README.md, TODO.md)

### Tests

- Deep test `run_learning_tests()` covering:
  - Learning stats retrieval
  - Query timing comparison (first vs second run)
  - Knowledge search capability
  - XP directory structure check

### Breaking Changes

None.

---

## v0.0.47 - First Mutation Flow

**Release Date:** 2025-12-03

### Summary

This release introduces the **first real mutation capability**: appending a line to a text file with full evidence collection, diff preview, confirmation gates, execution via annad, verification, and rollback. The mutation flow is sandbox-safe by default, protecting system files while allowing edits in the current directory, /tmp, and $HOME.

### Key Features

**Append Line Mutation Module (`append_line_mutation.rs`):**
- `SandboxCheck` - Classifies paths as Sandbox (cwd, /tmp), Home ($HOME), or System (blocked)
- `AppendMutationEvidence` - Collects file stat, preview, hash, and policy evidence before mutation
- `AppendDiffPreview` - Shows last 20 lines and the line to be appended
- `execute_append_line()` - Full mutation with backup, ownership preservation, and verification
- `execute_rollback()` - Restore file from backup by case_id

**Risk Levels and Confirmation:**
- Sandbox (cwd, /tmp): Low risk, confirmation phrase "yes"
- Home ($HOME): Medium risk, confirmation phrase "I CONFIRM (medium risk)"
- System paths: Blocked in v0.0.47

**4 New File Evidence Tools:**
- `file_stat` - Returns uid/gid, mode, size, mtime, exists flag
- `file_preview` - Returns first N bytes with secrets redacted, last 20 lines
- `file_hash` - Returns hash for before/after integrity verification
- `path_policy_check` - Returns policy decision with evidence ID and rule

**Case Files for Every Request:**
- Automatic user-readable copies in `$HOME/.local/share/anna/cases/<case_id>/`
- Contains summary.txt, transcript.log, evidence.json
- Resolves the "can't access cases without sudo" friction

**Rollback by Natural Language:**
- "To rollback, run: annactl 'rollback <case_id>'"
- Restores file bytes, ownership, and permissions
- Verifies restored hash matches pre-mutation hash
- Results written back to original case file

### Technical Details

**Sandbox Allowlist (Dev Mode):**
- Current working directory (detected via `std::env::current_dir()`)
- `/tmp/` and subdirectories
- `$HOME/` with policy checks

**Mutation Flow:**
1. Collect evidence (file_stat, file_preview, file_hash, path_policy_check)
2. Generate diff preview
3. Show preview and require confirmation phrase
4. Execute via annad (preserves ownership/mode)
5. Verify with after-hash
6. Store rollback artifact with backup path
7. Create case file with user copy

### Files Added/Modified

- **Added:** `crates/anna_common/src/append_line_mutation.rs` - New mutation module
- **Modified:** `crates/anna_common/src/tools.rs` - Added 4 file evidence tools
- **Modified:** `crates/anna_common/src/tool_executor.rs` - Implemented 4 tool executors
- **Modified:** `crates/anna_common/src/transcript.rs` - Added save_user_copy()
- **Modified:** `crates/anna_common/src/lib.rs` - Added module and exports
- **Modified:** `scripts/anna_deep_test.sh` - Added run_mutation_tests()
- **Modified:** Version files (Cargo.toml, CLAUDE.md, README.md, TODO.md)

### Tests

- Deep test `run_mutation_tests()` covering:
  - Diff preview display
  - Confirmation requirement
  - File unchanged without confirmation
  - Sandbox path policy recognition
  - Blocked path detection

### Breaking Changes

None.

---

## v0.0.46 - Evidence Quality Release

**Release Date:** 2025-12-03

### Summary

This release solves the **generic snapshot problem**: different domain questions were being answered with the same generic hw_snapshot_summary. Now, domain-specific questions (disk, kernel, memory, network, audio, boot, errors) are routed to specialized evidence tools that return focused, accurate data.

### Key Features

**10 New Domain-Specific Evidence Tools:**
- `uname_summary` - Kernel version and architecture (uname -r, -m)
- `mem_summary` - Memory info from /proc/meminfo (MemTotal, MemAvailable)
- `mount_usage` - Disk space per filesystem with root free/used highlighted
- `nm_summary` - NetworkManager status, active connections
- `ip_route_summary` - Routing table, default gateway
- `link_state_summary` - Interface link states (up/down, carrier)
- `audio_services_summary` - pipewire, wireplumber, pulseaudio service status
- `pactl_summary` - PulseAudio/PipeWire sinks, sources, default devices
- `boot_time_summary` - Uptime, boot timestamp
- `recent_errors_summary` - Journal errors filtered by optional keyword

**Domain Routing (route_to_domain_evidence):**
- Pattern matching for domain keywords (disk space, kernel version, memory, etc.)
- Maps domains to correct tools automatically
- Falls back to hw_snapshot_summary only for generic hardware queries

**Tool Sanity Gate (apply_tool_sanity_gate):**
- Runtime enforcement that generic snapshot cannot satisfy domain queries
- Automatically replaces hw_snapshot_summary with correct domain tools
- Prints "[v0.0.46] Tool sanity gate applied" when activated

**Deep Test Evidence Validation (run_evidence_tool_validation):**
- Tests that domain queries use correct tools
- Fails if generic hw_snapshot used for disk/kernel/memory/network/audio queries
- Table-driven test cases with expected/forbidden tool patterns

### Technical Details

**Domain Patterns:**
- Disk: "disk space", "disk free", "free space", "storage", "space left"
- Kernel: "kernel version", "linux version", "uname"
- Memory: "memory", "ram", "meminfo"
- Network: "NetworkManager", "nmcli", "network manager"
- Audio: "pipewire", "wireplumber", "audio service"
- Boot: "boot time", "uptime", "how long running"

**Sanity Gate Flow:**
1. Translator returns tool plan (possibly with hw_snapshot_summary)
2. Sanity gate checks if domain routing would return different tools
3. If so, replaces the tool plan with domain-specific tools
4. Pipeline executes corrected plan

### Files Modified

- `crates/anna_common/src/tools.rs` - 10 new tool definitions
- `crates/anna_common/src/tool_executor.rs` - 10 new tool implementations
- `crates/annactl/src/pipeline.rs` - route_to_domain_evidence(), apply_tool_sanity_gate()
- `scripts/anna_deep_test.sh` - run_evidence_tool_validation(), v0.0.46 header
- `Cargo.toml` - Version 0.0.46
- `CLAUDE.md` - Version 0.0.46
- `README.md` - Updated version and description
- `TODO.md` - Added 0.0.46 section, moved completed items
- `RELEASE_NOTES.md` - This file

### Tests

- Evidence tool validation in deep test (5 domain test cases)
- Each case asserts correct tool AND asserts forbidden tool NOT used

### Breaking Changes

None.

---

## v0.0.45 - Deep Test Harness + Correctness Fixes

**Release Date:** 2025-12-03

### Summary

This release focuses on **verification and correctness**. A comprehensive deep test harness has been added to prove fixes are real and reproducible. Key evidence tools have been added to ensure queries return concrete, cited values rather than generic summaries.

### Key Features

**Deep Test Harness (scripts/anna_deep_test.sh):**
- Single-command test runner producing timestamped artifact directories
- Environment capture (uname, CPU, RAM, GPU, disk layout)
- Translator stability tests (50 queries, fallback rate tracking)
- Read-only correctness tests (kernel, memory, disk, network, audio)
- Doctor auto-trigger tests (wifi, audio, boot)
- Policy gating tests (mutations blocked without confirmation)
- Case file verification
- REPORT.md and report.json outputs

**New Evidence Tools (v0.0.45):**
- `kernel_version` - Direct uname -r output, not from hw snapshot
- `memory_info` - Direct /proc/meminfo with total/free/available
- `network_status` - Interface states, default route, DNS, NetworkManager status
- `audio_status` - pipewire/wireplumber status, sinks, sources

**Enhanced disk_usage Tool:**
- Now returns explicit free space values for / (and all filesystems)
- Structured JSON with per-filesystem breakdown
- Root filesystem highlighted in summary

**Version Mismatch Display (annactl status):**
- CLI version vs daemon version comparison
- Warning when versions don't match
- Explanation of automatic daemon restart on update

**Doctor Selection Unit Tests:**
- Table-driven tests mapping 25 phrases to expected doctors
- Covers: networking, audio, boot, storage, graphics
- Tests for non-doctor queries to ensure they don't trigger incorrectly

### Documentation

- Added `docs/TESTING.md` - Complete testing guide
- Running unit tests, deep test harness, CI/CD integration
- Interpreting test results and troubleshooting

### Files Modified

- **NEW:** `scripts/anna_deep_test.sh` - Deep test harness
- **NEW:** `docs/TESTING.md` - Testing documentation
- `crates/anna_common/src/tools.rs` - 4 new tool definitions
- `crates/anna_common/src/tool_executor.rs` - 4 new tool implementations, disk_usage fix
- `crates/anna_common/src/doctor_registry.rs` - Table-driven selection tests
- `crates/annactl/src/commands/status.rs` - Version mismatch display
- `Cargo.toml` - Version 0.0.45
- `CLAUDE.md` - Version 0.0.45
- `README.md` - Updated version
- `TODO.md` - Updated version, moved completed items
- `RELEASE_NOTES.md` - This file

### Tests

8 new tests:
- `test_doctor_selection_table_driven` - 25 phrases map to correct doctors
- `test_no_doctor_for_unrelated_queries` - Non-diagnostic queries don't trigger
- New tool execution tests via deep test harness

### Breaking Changes

None.

---

## v0.0.44 - Translator Stabilization + Doctor Integration

**Release Date:** 2025-12-03

### Summary

This release stabilizes the Translator LLM integration with a simplified canonical format designed for small language models. The Doctor Registry from v0.0.43 is now integrated into the main pipeline, and all requests are logged to case files for debugging.

### Key Features

**Simplified Translator Format:**
- New canonical 6-line format: INTENT, TARGETS, RISK, TOOLS, DOCTOR, CONFIDENCE
- Designed for small models (qwen2.5:0.5b-instruct)
- Removed complex RATIONALE and CLARIFICATION fields
- Added retry logic (2 attempts) before deterministic fallback
- More robust parser with case-insensitive matching

**Doctor Integration in Pipeline:**
- FixIt intent now triggers doctor selection from registry
- Transcript shows doctor routing: doctor_name (doctor_id)
- Fallback to general troubleshooting if no doctor matches

**Case File Logging:**
- All requests now create case files (not just mutations)
- Includes: intent, reliability score, evidence count, timing, outcome
- Stored in `/var/lib/anna/cases/` with unique request IDs
- Enables debugging and replay of past requests

**Code Cleanup:**
- Removed unused Intent struct and legacy code
- Updated module header to v0.0.44
- Cleaned up test for new format

### Technical Details

**Translator Prompt (v0.0.44):**
```
Classify the user request. Output EXACTLY 6 lines:

INTENT: system_query OR action_request OR knowledge_query OR doctor_query
TARGETS: word1,word2 OR none
RISK: read_only OR low OR medium OR high
TOOLS: tool1,tool2 OR none
DOCTOR: networking OR graphics OR audio OR storage OR boot OR none
CONFIDENCE: 0 to 100
```

**Example Translator Output:**
```
INTENT: system_query
TARGETS: cpu
RISK: read_only
TOOLS: hw_snapshot_summary
DOCTOR: none
CONFIDENCE: 95
```

**Doctor Query Example:**
```
INTENT: doctor_query
TARGETS: wifi,network
RISK: read_only
TOOLS: hw_snapshot_summary,journal_warnings
DOCTOR: networking
CONFIDENCE: 85
```

### Files Modified

- `crates/annactl/src/pipeline.rs` - Simplified translator, doctor integration, case logging
- `Cargo.toml` - Version 0.0.44
- `README.md` - Updated version and description
- `CLAUDE.md` - Version 0.0.44
- `TODO.md` - Version 0.0.44

### Breaking Changes

- Translator CLARIFICATION field removed (was rarely used)
- Old translator output format no longer supported by parser

---

## v0.0.43 - Doctor Registry + Unified Entry Flow (Auto-Detect Domain and Run the Right Doctor)

**Release Date:** 2025-12-03

### Summary

Anna now has a **Doctor Registry** that automatically selects the right diagnostic doctor based on your request. Instead of manually specifying which doctor to use, Anna analyzes keywords, intent tags, and symptoms to pick the best match. All doctors now follow a unified lifecycle with consistent output schemas.

### Key Features

**Doctor Registry (Data-Driven):**
- Registry loaded from `/etc/anna/policy/doctors.toml` (or `~/.config/anna/doctors.toml`)
- 5 doctors registered: Network, Storage, Audio, Boot, Graphics
- Each doctor entry includes:
  - Keywords (e.g., "wifi", "audio", "boot")
  - Intent tags (e.g., "network_diagnosis", "audio_problem")
  - Symptom patterns (e.g., "no sound", "slow boot")
  - Required/optional evidence bundles
  - Allowed playbooks
  - Case file name

**Doctor Selection Algorithm:**
- Keyword matching (10 points per match)
- Intent tag matching (15 points per tag)
- Symptom matching (20 points per symptom)
- Priority weighting for tie-breaking
- Max 2 doctors per request (1 primary + 1 optional secondary from different domain)
- Selection includes reasoning explaining why

**Unified Doctor Run Lifecycle:**
1. `select_doctor` - Pick the right doctor(s)
2. `collect_evidence` - Gather required evidence bundle
3. `diagnosis_flow` - Run deterministic diagnosis steps
4. `playbook_offer` - Suggest fix if applicable
5. `apply_fix` - Execute playbook if confirmed
6. `verify` - Check fix worked (or mark pending)
7. `close` - Complete run, capture recipe if eligible

**Doctor Run Output Schema (doctor_run.json):**
```json
{
  "schema_version": 1,
  "run_id": "dr-20231203-143022-123",
  "user_request": "my sound is broken",
  "selection": {
    "primary": {
      "doctor_id": "audio_doctor",
      "doctor_name": "Audio Doctor",
      "match_score": 85,
      "match_reason": "Matched keywords: sound, audio"
    },
    "reasoning": "Selected Audio Doctor based on keywords and symptoms"
  },
  "current_stage": "close",
  "stage_timings": [...],
  "key_findings": [...],
  "chosen_playbook": "restart_pipewire",
  "verification_status": { "verified": true },
  "result": "success",
  "reliability": 90,
  "junior_verification": {
    "doctor_choice_approved": true,
    "diagnosis_steps_followed": true,
    "fix_policy_compliant": true,
    "score": 92
  }
}
```

**Junior Verification Enforcement:**
- Verifies doctor choice makes sense for request
- Confirms diagnosis steps were followed
- Checks fix is policy-compliant and minimal
- Ensures final claim is evidence-backed or marked "verification pending"

**Status Integration:**
`annactl status` now shows:
- Last doctor run (ID, result, reliability)
- Doctor runs today count
- Success rate today

### doctors.toml Schema

```toml
schema_version = 1

[[doctors]]
id = "network_doctor"
name = "Network Doctor"
description = "Diagnoses network connectivity, WiFi, DNS, and routing issues"
domain = "network"
keywords = ["network", "wifi", "internet", "connection", "dns", "ping"]
intent_tags = ["network_diagnosis", "connectivity_issue"]
symptoms = ["no internet", "wifi disconnecting", "can't connect"]
required_evidence = ["interface_status", "ip_addresses", "routes", "dns_config"]
optional_evidence = ["wifi_signal", "network_manager_status"]
required_tools = ["ip", "ping"]
allowed_playbooks = ["restart_networkmanager", "renew_dhcp", "flush_dns"]
case_file_name = "networking_doctor.json"
priority = 80
enabled = true

[[doctors]]
id = "audio_doctor"
name = "Audio Doctor"
# ... similar structure
```

### Files Changed

- **NEW:** `crates/anna_common/src/doctor_registry.rs` - Full registry implementation (~1100 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added module and exports
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.43
- **MODIFIED:** `CLAUDE.md` - Version bump
- **MODIFIED:** `README.md` - Version bump and feature description
- **MODIFIED:** `TODO.md` - Version bump
- **MODIFIED:** `RELEASE_NOTES.md` - This file

### Tests

20 new tests covering:
- Default registry creation
- Select network doctor ("wifi disconnecting")
- Select audio doctor ("no sound")
- Select boot doctor ("boot is slow")
- Select graphics doctor ("screen share broken")
- Select storage doctor ("disk full")
- Ambiguous request selects one and explains why
- Selection with intent tags
- No match returns None
- Doctor run lifecycle stages
- Doctor domain display
- Stage status display
- Doctor run result display
- Run ID generation
- Junior verification
- Verification pending
- Mark verified
- Config serialization
- Secondary doctor different domain
- Doctors by domain
- Selection explains why

### Example Transcript: "sound broken" - Doctor Selection and Flow

```
[you] to [anna]: my sound is broken

[anna] to [translator]: parse user intent
[translator] to [anna]: intent=fix_problem, targets=[audio], tags=[audio_problem]

[anna] to [doctor_registry]: select doctor for "my sound is broken"
[doctor_registry] to [anna]:
  Selection complete:
    Primary: Audio Doctor (score: 65)
    Matched keywords: sound
    Matched symptoms: no sound
    Reasoning: Selected Audio Doctor (score: 65) as primary based on
               keywords: sound and symptoms: no sound

[anna] to [you]:
  Selected: Audio Doctor
  Why: Your request mentions "sound" and matches the symptom "no sound".
  Evidence to collect: pipewire_status, audio_devices, default_sink

[anna] to [audio_doctor]: collect evidence
[audio_doctor] to [anna]:
  Evidence collected:
  - pipewire_status: NOT RUNNING
  - audio_devices: 2 detected (speakers, headphones)
  - default_sink: not set
  Run ID: dr-20231203-143022-123

[anna] to [audio_doctor]: run diagnosis
[audio_doctor] to [anna]:
  Step 1: Service Status - FAIL
    PipeWire service is not running
    Implication: Audio subsystem is down

  Step 2: Device Detection - PASS
    Found 2 audio devices
    Implication: Hardware detected correctly

  Step 3: Configuration - FAIL
    No default sink configured
    Implication: Even if service starts, no output selected

  Finding: pipewire-not-running (Error)
  Hypothesis: "PipeWire crashed or was never started" (90% confidence)
  Suggested playbook: restart_pipewire

[anna] to [you]:
  Diagnosis Complete

  Audio Health: BROKEN

  Key Findings:
  1. [Error] PipeWire service is not running
  2. [Warning] No default audio sink configured

  Top Hypothesis (90% confidence):
    PipeWire service has stopped. Restarting should restore audio.

  Recommended Fix:
    Restart PipeWire (Low risk)

  Type "I CONFIRM" to apply fix

[you] to [anna]: I CONFIRM

[anna] to [audio_doctor]: execute playbook restart_pipewire
[audio_doctor] to [anna]:
  Playbook executed:
  - Command: systemctl --user restart pipewire pipewire-pulse wireplumber
  - Exit code: 0
  - Post-check: PipeWire running = PASS

[anna] to [junior]: verify doctor run dr-20231203-143022-123
[junior] to [anna]:
  VERIFICATION:
  - Doctor choice approved: YES (audio keywords matched)
  - Diagnosis steps followed: YES (3 steps completed)
  - Fix policy-compliant: YES (restart_pipewire in allowed_playbooks)
  - Fix minimal: YES (single service restart)
  - Claim evidence-backed: YES (post-check confirms service running)
  SCORE: 92

[anna] to [you]:
  Audio Restored!

  PipeWire is now running. Try playing audio.

  Run ID: dr-20231203-143022-123
  Reliability: 92%
  Recipe captured: Yes (restart_pipewire for no-sound)

  Case file: audio_doctor.json
  Run file: doctor_run.json
```

### Example: Ambiguous Request Handled

```
[you] to [anna]: my system is slow

[anna] to [doctor_registry]: select doctor for "my system is slow"
[doctor_registry] to [anna]:
  Selection complete:
    Primary: Boot Doctor (score: 25)
    Matched keywords: slow
    Reasoning: Selected Boot Doctor (score: 25) as primary.
               "slow" matches boot diagnosis for startup slowness.
               Note: Score is low. Request is ambiguous.

[anna] to [you]:
  I'll use the Boot Doctor to check startup performance.

  Note: Your request "my system is slow" could mean several things:
  - Slow boot time → Boot Doctor (selected)
  - Slow disk access → Storage Doctor
  - Slow network → Network Doctor

  If this doesn't help, please clarify what feels slow
  (e.g., "boot takes forever" or "disk is slow").

  Collecting boot evidence...
```

---

## v0.0.42 - Arch GPU/Graphics Doctor v1 (Wayland/X11, Drivers, Compositor Health)

**Release Date:** 2025-12-03

### Summary

Anna can now diagnose common graphics issues on Arch Linux systems. The Graphics Doctor detects session type (Wayland/X11), compositor (Hyprland focus), GPU inventory, driver stack (NVIDIA/AMD/Intel), XDG portal health for screen sharing, and PipeWire status. It produces evidence-backed hypotheses for issues like black screen, stutter, tearing, and broken screen sharing.

### Key Features

**Evidence Bundle (Read-Only):**
- Session type detection (Wayland vs X11 vs TTY)
- Compositor detection (Hyprland, Sway, KDE Plasma, GNOME/Mutter, wlroots-based, X11 WMs)
- GPU inventory via lspci with vendor detection (NVIDIA, AMD, Intel)
- Kernel module detection (nvidia, nvidia-open, nouveau, amdgpu, radeon, i915, xe)
- Driver packages (nvidia, mesa, vulkan, libva, vdpau)
- XDG Portal stack health (xdg-desktop-portal, xdg-desktop-portal-hyprland, -gtk, -kde, etc.)
- PipeWire and WirePlumber status (required for screen sharing)
- Portal backend matching (correct backend for compositor)
- Monitor information (from hyprctl/swaymsg/xrandr)
- Compositor and portal logs from journalctl
- Graphics-relevant environment variables

**Deterministic Diagnosis Flow (6 steps):**
1. Session Detection - Identify Wayland/X11/TTY and compositor
2. GPU Detection - Inventory GPUs, detect vendor/driver
3. Package Verification - Confirm required packages for stack
4. Portal Health - Check XDG portal services, backend matching
5. Log Analysis - Scan for crash indicators, errors
6. Hypothesis Generation - Produce max 3 evidence-backed theories

Each step produces:
- Result: PASS/FAIL/PARTIAL/SKIPPED
- Details with evidence citations
- Implication explaining what it means

**Portal Backend Matching:**
| Compositor | Expected Portal Backend |
|------------|------------------------|
| Hyprland | xdg-desktop-portal-hyprland or -wlr |
| Sway | xdg-desktop-portal-wlr |
| KDE Plasma | xdg-desktop-portal-kde |
| GNOME | xdg-desktop-portal-gnome |
| wlroots-based | xdg-desktop-portal-wlr |
| X11 WMs | xdg-desktop-portal-gtk |

**Hypothesis Examples:**
- "Wrong portal backend for compositor" (90% confidence)
- "Portal service not running" (85%)
- "PipeWire not running for screen sharing" (85%)
- "NVIDIA driver not loaded" (80%)
- "Crash indicators in compositor logs" (75%)

**Fix Playbooks (with confirmation):**

| Playbook | Risk | Description |
|----------|------|-------------|
| `restart_portals` | Low | Restart xdg-desktop-portal services |
| `restart_pipewire_portals` | Low | Restart PipeWire, WirePlumber, and portals |
| `collect_crash_report` | Info | Collect crash logs for debugging |
| `restart_display_manager` | High | BLOCKED - Restart display manager (gdm, sddm, etc.) |

Each playbook has:
- Preflight checks (service exists, etc.)
- Commands with timeout
- Post-checks with wait times
- Rollback commands
- Policy blocking for high-risk actions
- Confirmation phrase: `I CONFIRM`

**Recipe Capture:**
When a fix playbook succeeds with reliability >= 80%:
- Automatic recipe creation request
- Problem description from diagnosis
- Solution from playbook
- Preconditions (session type, compositor)
- Evidence patterns for matching

**Case File Integration:**
- `graphics_doctor.json` in case files
- Evidence bundle stored
- Diagnosis result tracked
- Playbook execution recorded
- Notes and timestamps

### Files Changed

- **NEW:** `crates/anna_common/src/graphics_doctor.rs` - Full graphics doctor implementation (~1800 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added module and exports
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.42
- **MODIFIED:** `CLAUDE.md` - Version bump
- **MODIFIED:** `README.md` - Version bump and feature description
- **MODIFIED:** `TODO.md` - Version bump
- **MODIFIED:** `RELEASE_NOTES.md` - This file

### Tests

18 new tests covering:
- Session type display
- Compositor display
- GPU vendor display
- Driver stack display
- Portal backend display
- Graphics health display
- Risk level display
- Session info collection
- GPU info collection
- Evidence health determination (broken/degraded/healthy)
- Portal backend correctness
- Diagnosis healthy system
- Diagnosis broken portal
- Wrong portal backend detection
- Restart portals playbook
- Display manager restart blocked by policy
- Case file workflow
- Recipe capture on success

### Example Transcript: "Screen Share Not Working"

```
[you] to [anna]: my screen share is not working in discord

[anna] to [translator]: parse user intent
[translator] to [anna]: graphics diagnosis request (screen sharing issue)

[anna] to [graphics_doctor]: collect evidence
[graphics_doctor] to [anna]:
  Evidence collected:
  - Session: Wayland (Hyprland)
  - GPU: AMD Radeon RX 7900 XTX (amdgpu driver)
  - Portals: xdg-desktop-portal running, but using GTK backend
  - PipeWire: running
  - WirePlumber: running

[anna] to [graphics_doctor]: run diagnosis
[graphics_doctor] to [anna]:
  Step 1: Session Detection - PASS
    Session type: Wayland
    Compositor: Hyprland
    Implication: Wayland compositor detected. Portal stack required for screen sharing.

  Step 2: GPU Detection - PASS
    Found 1 GPU(s):
    - AMD Radeon RX 7900 XTX (amdgpu driver loaded)
    Implication: GPU and driver correctly detected.

  Step 3: Package Verification - PASS
    Required packages present for AMD/Mesa stack.

  Step 4: Portal Health - FAIL
    Portal service running but wrong backend.
    Active backend: GTK
    Expected for Hyprland: Hyprland or wlr
    Implication: Screen sharing will likely fail or show black screen.

  Step 5: Log Analysis - PASS
    No crash indicators found in compositor logs.

  Step 6: Hypothesis Generation - PASS
    Generated 2 hypothesis(es):
    1. "Wrong portal backend for Hyprland" (95% confidence)
       GTK portal cannot capture Hyprland screens.
       Install xdg-desktop-portal-hyprland and restart portals.
       Suggested playbook: restart_portals
    2. "Portal may need restart after package change" (70% confidence)
       If portal was recently installed, restart may help.

[anna] to [you]:
  Graphics Health: DEGRADED

  Your screen sharing is likely failing because the wrong portal
  backend is active.

  Top hypothesis (95% confidence):
    Wrong portal backend for Hyprland compositor.
    You have GTK portal but Hyprland needs xdg-desktop-portal-hyprland
    or xdg-desktop-portal-wlr.

  Recommended fix:
    1. Install xdg-desktop-portal-hyprland (if not installed)
    2. Restart portal services
    Risk: Low - just restarts user services

  To apply restart playbook: type "I CONFIRM" when prompted

  Evidence: [ev-graphics-1733234567-session] [ev-graphics-1733234567-portals]

  Reliability: 90%
```

### Example Transcript: Portal Restart Fix

```
[anna] to [you]:
  Playbook: restart_portals
  Action: Restart XDG portal services for screen sharing

  This will:
  1. Preflight: Check portal services exist
  2. Execute: systemctl --user restart xdg-desktop-portal*
  3. Post-check: Verify portals running, wait 3s

  Rollback: Services will restart automatically on next login

  This is a Low-risk operation.
  Type "I CONFIRM" to proceed:

[you] to [anna]: I CONFIRM

[anna] to [annad]: execute playbook restart_portals
[annad] to [anna]:
  Playbook executed:
  - Commands: 1 executed
  - Post-checks: 1/1 passed
  - Result: SUCCESS
  - Reliability: 85%

[anna] to [you]:
  Portal services restarted successfully!

  Please try screen sharing again in Discord.

  If this fixes the issue, a recipe will be created for future use.

  Case file: graphics-20231203-143022.json
```

### Example: High-Risk Action Blocked

```
[you] to [anna]: restart my display manager

[anna] to [graphics_doctor]: generate playbook restart_display_manager
[graphics_doctor] to [anna]:
  Playbook: restart_display_manager
  POLICY BLOCKED: Restarting display manager will kill your session.
  This action requires manual intervention.

[anna] to [you]:
  Cannot restart display manager automatically.

  This is a HIGH-RISK action that would immediately terminate your
  graphical session, potentially causing data loss.

  If you need to restart the display manager, do it manually:
    sudo systemctl restart gdm  (or sddm, lightdm)

  Make sure to save all work first.

  Reliability: N/A (blocked by policy)
```

---

## v0.0.41 - Arch Boot Doctor v1 (Slow Boot + Service Regressions + "What Changed" Correlation)

**Release Date:** 2025-12-03

### Summary

Anna can now diagnose slow boot causes on Arch Linux systems. The Boot Doctor uses systemd tooling to identify offenders, detect regressions against baseline, and correlate issues with recent changes (package updates, service enables, config edits). Successful fixes with high reliability automatically create recipes for future reuse.

### Key Features

**Evidence Bundle (Read-Only):**
- `systemd-analyze time` - Boot timing breakdown (firmware, loader, kernel, initrd, userspace)
- `systemd-analyze blame` - Top N slowest units (default 20)
- `systemd-analyze critical-chain` - Critical path units (top 10)
- Enabled units snapshot (delta-friendly)
- Recent journal warnings/errors during boot
- "What changed" in last N days (default 14) - packages, services, configs
- Anna telemetry boot time trend (if tracked)
- Baseline for comparison (if available)

**Deterministic Diagnosis Flow (5 steps):**
1. Boot Time Summary - Analyze systemd-analyze output
2. Top Offenders - Identify units taking > 5s to start
3. Regression Check - Compare with baseline (new/regressed offenders)
4. Correlation Analysis - Link slow units to recent changes
5. Hypothesis Generation - Produce max 3 evidence-backed theories

Each step produces:
- Result: PASS/FAIL/PARTIAL/SKIPPED
- Details with evidence citations
- Implication explaining what it means

**"What Changed" Correlation Engine:**
- Parses `/var/log/pacman.log` for package installs/updates/removes
- Tracks kernel updates separately
- Detects service enable/disable events
- Ranks potential causes by evidence strength
- Every ranked item cites evidence IDs

**Hypothesis Generation (max 3):**
- "NetworkManager-wait-online is delaying boot" (90% confidence)
- "New service X is slowing boot" (85%)
- "Package update correlates with slow unit" (75%)
- "Service Y may be stuck or timing out" (70%)

**Fix Playbooks (with confirmation):**

| Playbook | Risk | Description |
|----------|------|-------------|
| `disable_wait_online` | Medium | Disable NetworkManager-wait-online.service |
| `restart_<unit>` | Low | Restart a stuck or slow service |
| `disable_<unit>` | Medium | Disable a non-essential slow service |

Each playbook has:
- Preflight checks (unit exists, etc.)
- Commands with timeout
- Post-checks with wait times
- Rollback commands
- Policy blocking for critical services (systemd-*, dbus, login, udev)
- Confirmation phrase: `I CONFIRM`
- Verification pending note for post-reboot checks

**Recipe Capture:**
When a fix playbook succeeds with reliability >= 80%:
- Automatic recipe creation request
- Problem description from diagnosis
- Solution from playbook
- Preconditions (boot time, target unit)
- Evidence patterns for matching

**Case File Integration:**
- `boot_doctor.json` in case files
- Evidence bundle stored
- Diagnosis result tracked
- Playbook execution recorded
- Verification pending items for post-reboot confirmation
- Notes and timestamps

### Files Changed

- **NEW:** `crates/anna_common/src/boot_doctor.rs` - Full boot doctor implementation (~1600 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added module and exports
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.41
- **MODIFIED:** `CLAUDE.md` - Version bump
- **MODIFIED:** `README.md` - Version bump and feature description
- **MODIFIED:** `TODO.md` - Version bump
- **MODIFIED:** `RELEASE_NOTES.md` - This file

### Tests

18 new tests covering:
- Boot health display
- Risk level display
- Timing is_slow/is_very_slow checks
- Evidence health determination
- Healthy system diagnosis
- Slow boot diagnosis
- New offender detection
- Regression detection
- Correlation engine
- Wait-online playbook generation
- Restart service playbook
- Disable critical service blocked by policy
- Max 3 hypotheses limit
- Case file workflow
- Recipe capture on success
- Regression percent calculation
- Timing summary formatting

### Example Transcript: "Boot Got Slower" with Offender Ranking

```
[you] to [anna]: my boot got slower recently

[anna] to [translator]: parse user intent
[translator] to [anna]: boot diagnosis request

[anna] to [boot_doctor]: collect evidence (lookback: 14 days)
[boot_doctor] to [anna]:
  Evidence collected:
  - Boot timing: 45000ms total (userspace: 38000ms)
  - Top offenders: 3 units > 5s
  - Baseline available from 7 days ago (userspace: 15000ms)
  - Recent changes: 5 packages, 1 service enable

[anna] to [boot_doctor]: run diagnosis
[boot_doctor] to [anna]:
  Step 1: Boot Time Summary - PARTIAL
    Boot time: 45000ms (userspace 38000ms > 30s threshold)
    Implication: Boot is slower than ideal. Optimization possible.

  Step 2: Top Offenders - PARTIAL
    Found 3 slow units:
    - NetworkManager-wait-online.service (28000ms)
    - docker.service (6500ms) [NEW]
    - systemd-journal-flush.service (3200ms)

  Step 3: Regression Check - FAIL
    2 new slow units, 1 regressed unit
    - NEW: docker.service (6500ms) not in baseline
    - REGRESSED: NetworkManager-wait-online.service 28000ms (was 8000ms, +250%)

  Step 4: Correlation Analysis - PASS
    Found 2 correlation(s):
    - docker.service slow after Package Install docker
    - NetworkManager-wait-online regressed after networkmanager 1.46.0-1 update

  Step 5: Hypothesis Generation - PASS
    Generated 3 hypothesis(es):
    1. "NetworkManager-wait-online.service is delaying boot" (90% confidence)
       Suggested playbook: disable_wait_online
    2. "New service docker.service is slowing boot" (85% confidence)
       Correlated with: docker package install 2 days ago
    3. "NetworkManager update may have caused regression" (75% confidence)
       Evidence: pacman log shows upgrade on 2025-12-01

[anna] to [you]:
  Boot Health: DEGRADED

  Your boot time has regressed from ~15s to ~38s (userspace).

  Top hypothesis (90% confidence):
    NetworkManager-wait-online.service is delaying boot by 28s.
    This service waits for full network connectivity before proceeding.

  Correlated changes:
    - docker installed 2 days ago (new 6.5s offender)
    - networkmanager updated (wait-online regressed from 8s to 28s)

  Recommended fix:
    Disable NetworkManager-wait-online.service
    Risk: Medium - network-dependent services may start before network ready
    To apply: type "I CONFIRM" when prompted

  Evidence: [ev-boot-1733234567-timing] [ev-boot-1733234567-blame]
            [ev-pacman-1] [ev-pacman-2]

  Reliability: 85%
```

### Example Change Set: Disabling Non-Essential Slow Service (Policy-Gated)

```
[anna] to [you]:
  Playbook: disable_wait_online
  Action: Disable NetworkManager-wait-online.service

  This will:
  1. Run preflight: Verify unit exists
  2. Execute: systemctl disable --now NetworkManager-wait-online.service
  3. Post-check: Verify unit is disabled

  Rollback available:
    systemctl enable --now NetworkManager-wait-online.service

  WARNING: This is a Medium-risk operation.
  Network will still work, but network-dependent services may start
  before network is fully ready.

  Verification pending: Boot time improvement will be verified on next boot

  To proceed, type exactly: I CONFIRM

[you] to [anna]: I CONFIRM

[anna] to [boot_doctor]: execute playbook disable_wait_online
[boot_doctor] to [anna]:
  Preflight: PASS - Unit exists
  Executed: systemctl disable --now NetworkManager-wait-online.service
  Post-check: PASS - Unit disabled

  Result: SUCCESS
  Reliability: 75% (capped until verified on reboot)

  Next steps:
  - Reboot to verify boot time improvement
  - Run "systemd-analyze time" after reboot
  - Expected improvement: ~25-28 seconds

[anna] to [you]:
  Fix applied successfully.

  The service has been disabled. On next reboot:
  - Boot should be ~25-28 seconds faster
  - Run me again to verify and capture recipe if successful

  Verification pending item added to case file.
```

---

## v0.0.40 - Arch Audio Doctor v1 (PipeWire Focus + Fix-It Playbooks + Recipe Capture)

**Release Date:** 2025-12-03

### Summary

Anna can now diagnose and fix common audio issues on Arch Linux desktops. The Audio Doctor focuses on PipeWire (the modern audio stack) while providing legacy PulseAudio conflict detection. Successful fixes with high reliability automatically create recipes for future reuse.

### Key Features

**Supported Audio Stacks:**
- **PipeWire + WirePlumber** (primary, recommended)
- **PulseAudio** (legacy detection, conflict resolution)
- **ALSA** (basic hardware detection)
- **Bluetooth** (bluez integration)

**Common Issues Diagnosed:**
- "No sound" - Service not running, muted, wrong output
- "Mic not working" - Source issues, permissions
- "Bluetooth audio broken" - Service, adapter, connection, profile issues
- "Wrong output" - Default sink misconfigured
- "Crackling" - Service restart often helps

**Evidence Bundle:**
- Audio stack detection (pipewire vs pulseaudio vs alsa-only)
- Service states (pipewire, wireplumber, pulseaudio - user services)
- ALSA devices (aplay/arecord -l)
- PipeWire/WirePlumber nodes (wpctl status)
- Default sink/source with volume/mute state
- Bluetooth adapter and device states
- User permissions (audio/video/bluetooth groups, /dev/snd access)
- Recent journal logs

**Deterministic Diagnosis Flow (6 steps):**
1. Identify Audio Stack - Detect PipeWire/PulseAudio/ALSA
2. Verify Services - Check pipewire/wireplumber running
3. Confirm Devices - ALSA hardware + PipeWire nodes
4. Check Default Output - Sink selection, volume, mute
5. Check Conflicts - PulseAudio vs PipeWire
6. Check Bluetooth - Service, adapter, device, profile

Each step produces:
- Result: PASS/FAIL/PARTIAL/SKIPPED
- Details with evidence
- Implication explaining what it means

**Hypothesis Generation (max 3):**
- "PipeWire user service not running" (95% confidence)
- "WirePlumber session manager not running" (90%)
- "Default output is muted" (95%)
- "Volume too low" (85%)
- "PulseAudio running alongside PipeWire" (90%)
- "Bluetooth using low-quality profile" (80%)
- "Audio device permission issue" (60%)

**Fix Playbooks (with confirmation):**

| Playbook | Risk | Description |
|----------|------|-------------|
| `restart_pipewire` | Low | Restart PipeWire + WirePlumber user services |
| `restart_wireplumber` | Low | Restart WirePlumber only |
| `unmute_volume` | Low | Unmute + set volume to 50% |
| `set_default_sink` | Low | Set first available sink as default |
| `stop_pulseaudio` | Medium | Stop and mask PulseAudio (BLOCKED by default) |
| `set_bt_a2dp` | Low | Switch Bluetooth to high-quality A2DP profile |

Each playbook has:
- Preflight checks
- Commands (run as target user for user services)
- Post-checks with wait times
- Rollback commands
- Policy blocking for higher-risk operations
- Confirmation phrase: `I CONFIRM`

**Recipe Capture:**
When a fix playbook succeeds with reliability >= 80%:
- Automatic recipe creation request
- Problem description from diagnosis
- Solution from playbook
- Preconditions (audio stack, target user)
- Evidence patterns for matching

**Target User Correctness:**
All user service operations (pipewire, wireplumber, pulseaudio) are executed for the target user, not root.

**Case File Integration:**
- `audio_doctor.json` in case files
- Evidence bundle stored
- Diagnosis result tracked
- Playbook execution recorded
- Recipe capture requests saved

### Files Changed

- **NEW:** `crates/anna_common/src/audio_doctor.rs` - Full audio doctor implementation (~1400 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added module and exports
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.40
- **MODIFIED:** `CLAUDE.md` - Version bump
- **MODIFIED:** `README.md` - Version bump and feature description
- **MODIFIED:** `TODO.md` - Version bump
- **MODIFIED:** `RELEASE_NOTES.md` - This file

### Tests

16 new tests covering:
- Audio stack display
- Audio health display
- Service state healthy check
- Audio node issue detection
- PulseAudio conflict detection
- Evidence health determination
- Healthy system diagnosis
- PipeWire stopped diagnosis
- Muted output diagnosis
- PulseAudio conflict diagnosis
- Playbook generation
- PulseAudio stop blocked by policy
- Case file workflow
- Recipe capture on success
- Bluetooth profile hypothesis
- Max 3 hypotheses limit

### Example Transcript: "No Sound" Resolved

```
[you] to [anna]: I have no sound

[anna] to [translator]: parse user intent
[translator] to [anna]: audio diagnosis request

[anna] to [audio_doctor]: collect evidence for testuser
[audio_doctor] to [anna]: Evidence collected. Stack: PipeWire

[anna] to [audio_doctor]: run diagnosis
[audio_doctor] to [anna]:
  Step 1: Identify Audio Stack - PASS
    PipeWire detected as active audio stack
  Step 2: Verify Services - FAIL
    PipeWire running but WirePlumber stopped
    Implication: Session/policy management missing
  Step 3: Confirm Devices - PASS
    Found 2 sink(s), 1 source(s)
  Step 4: Check Default Output - PASS
    Default sink 'Built-in Audio' at 50% volume, not muted
  Step 5: Check Conflicts - PASS
    No conflicts detected
  Step 6: Check Bluetooth - SKIPPED

  Hypothesis: WirePlumber session manager not running (90% confidence)
  Suggested fix: restart_wireplumber

[anna] to [you]: Your audio is not working because WirePlumber
                 (the session manager) is stopped. PipeWire itself
                 is running but cannot route audio without WirePlumber.

                 I can restart WirePlumber for you. Type:
                 I CONFIRM (apply fix)

[you] to [anna]: I CONFIRM

[anna] to [audio_doctor]: execute restart_wireplumber for testuser
[audio_doctor] to [anna]:
  Command: systemctl --user restart wireplumber
  Result: exit code 0
  Post-check: WirePlumber running - PASS
  Reliability: 92%

[anna] to [junior]: validate fix result
[junior] to [anna]: SCORE 92, fix verified, recipe creation triggered

[anna] to [you]: Fixed! WirePlumber is now running.
                 Reliability: 92%

                 Created recipe: "Fix: No sound because WirePlumber stopped"
```

### Example Transcript: Bluetooth Audio Profile Issue

```
[you] to [anna]: my bluetooth headphones sound terrible

[anna] to [audio_doctor]: collect evidence including bluetooth
[audio_doctor] to [anna]:
  Bluetooth: service active, adapter powered
  Device 'Sony WH-1000XM4' connected via HSP (low quality)
  A2DP profile available but not active

[anna] to [audio_doctor]: run diagnosis
[audio_doctor] to [anna]:
  Step 6: Check Bluetooth - PARTIAL
    Bluetooth audio device 'Sony WH-1000XM4' connected, profile: HSP
    Implication: Using low-quality profile

  Hypothesis: Bluetooth using low-quality profile (80% confidence)
    Device is using HSP (voice) instead of A2DP (high quality music)
  Suggested fix: set_bt_a2dp

[anna] to [you]: Your Bluetooth headphones are connected using the
                 HSP profile (meant for calls, low quality).
                 For music, you want A2DP (high fidelity).

                 I can switch to A2DP for you. Type:
                 I CONFIRM (apply fix)

[you] to [anna]: I CONFIRM

[anna] to [audio_doctor]: execute set_bt_a2dp for testuser
[audio_doctor] to [anna]:
  Command: wpctl set-profile ... 1
  Post-check: Profile set - PASS
  Reliability: 85%

[anna] to [you]: Switched to A2DP profile. Your headphones should
                 now sound much better for music playback.
                 Reliability: 85%
```

---

## v0.0.39 - Arch Storage Doctor v1 (BTRFS Focus + Safe Repair Plans)

**Release Date:** 2025-12-03

### Summary

Anna can now diagnose storage issues with a focus on BTRFS filesystems. The Storage Doctor collects comprehensive evidence, follows a deterministic diagnosis flow, generates risk-rated hypotheses, and offers safe repair plans with policy controls.

### Key Features

**Evidence Bundle:**
- Mount topology (lsblk, findmnt)
- Filesystem types (BTRFS, EXT4, XFS)
- Free space and metadata space (BTRFS-specific)
- Device errors (btrfs device stats)
- SMART health data (smartctl)
- Scrub and balance status (BTRFS)
- I/O errors from kernel log (journalctl)

**BTRFS-Specific Diagnostics:**
- Metadata pressure detection (critical when >90%)
- Device error tracking (corruption_errs, generation_errs = critical)
- Scrub status with uncorrected error detection
- Balance status monitoring
- RAID profile awareness

**Deterministic Diagnosis Flow (5 steps):**
1. Identify Filesystem Types - Detect BTRFS, EXT4, XFS
2. Check Space Usage - Free space and BTRFS metadata space
3. Check Device Health - BTRFS device stats and SMART data
4. Check BTRFS Maintenance - Scrub and balance status
5. Check I/O Error Logs - Kernel log analysis

Each step produces:
- Pass/Fail status
- Findings with risk levels (Info/Warning/Critical)
- Evidence IDs for traceability

**Hypothesis Generation (max 3):**
Each hypothesis includes:
- Summary and explanation
- Confidence percentage (0-100)
- Supporting evidence IDs
- Confirm/refute criteria
- Suggested repair plan

Example hypotheses:
- "Failing storage device" (SMART failure + I/O errors)
- "BTRFS metadata space exhaustion" (metadata >90%)
- "Data corruption detected" (scrub uncorrected errors)

**Safe Repair Plans:**

*Read-Only Plans (no confirmation needed):*
- SMART Extended Test - Run self-test on drive
- BTRFS Device Stats - View current error statistics

*Mutation Plans (confirmation required):*
- Start Scrub - Verify data integrity (allowed by default)
- Balance Metadata - Redistribute metadata chunks (BLOCKED by default)
- Clear Device Stats - Reset error counters after investigation

Each mutation plan has:
- Risk level
- Preflight checks
- Post-execution checks
- Rollback instructions
- Policy block status
- Confirmation phrase

**Policy Controls:**
- `allow_scrub: true` - Scrub operations allowed by default
- `allow_balance: false` - Balance operations BLOCKED by default (can take hours)

**Case File Integration:**
- `storage_doctor.json` in case files
- Full evidence bundle stored
- Diagnosis result tracked
- Repair history maintained
- Notes and status updates

### Storage Health Status

- **Healthy**: All checks pass, no warnings
- **Degraded**: Non-critical issues (I/O errors, SMART warnings, device errors)
- **Critical**: Data at risk (SMART failed, corruption, metadata exhausted)
- **Unknown**: Evidence collection failed

### Files Changed

- **NEW:** `crates/anna_common/src/storage_doctor.rs` - Full storage doctor implementation (~2100 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added module and exports
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.39
- **MODIFIED:** `CLAUDE.md` - Version bump
- **MODIFIED:** `README.md` - Version bump and feature description
- **MODIFIED:** `RELEASE_NOTES.md` - This file

### Tests

17 new tests covering:
- Storage health display
- Filesystem type parsing
- Mount space checks
- BTRFS device stats error detection
- BTRFS metadata pressure detection
- SMART health warnings
- Diagnosis flow (healthy, degraded, critical)
- Hypothesis generation (max 3)
- Repair plan generation and policy blocking
- Case file workflow
- Scrub status with uncorrected errors

All 746 tests pass with zero regressions.

---

## v0.0.38 - Arch Networking Doctor v1 (WiFi/Ethernet Diagnosis + Fix-It Playbooks)

**Release Date:** 2025-12-03

### Summary

Anna can now diagnose and fix common Arch Linux networking issues. The Networking Doctor follows a deterministic diagnosis flow, generates evidence-backed hypotheses, and offers fix playbooks with confirmation, post-checks, and rollback support.

### Key Features

**Supported Network Managers:**
- NetworkManager (most common)
- iwd (Intel wireless daemon)
- systemd-networkd
- wpa_supplicant

**Deterministic Diagnosis Flow (6 steps):**
1. Physical Link - Check carrier/WiFi association
2. IP Address - Check for IPv4/IPv6 assignment
3. Default Route - Check for gateway configuration
4. IP Connectivity - Ping gateway and 1.1.1.1
5. DNS - Test name resolution
6. Manager Health - Check service status and conflicts

Each step produces:
- Result: PASS/FAIL/PARTIAL/SKIPPED
- Details with evidence
- Implication explaining what it means

**Evidence Collection Bundle:**
- Interface inventory (ip link, ip addr)
- Routes + default gateway (ip route)
- DNS config (resolvectl or resolv.conf)
- Link state + speed (ethtool)
- WiFi status (iw dev, signal strength)
- Manager state (systemctl status)
- Recent logs (journalctl filtered)

**Hypothesis Generation (1-3 max):**
Each hypothesis includes:
- Description
- Confidence percentage
- Supporting evidence IDs
- Next test to confirm/refute
- Suggested fix playbook

**Fix Playbooks (with confirmation):**
- `restart_manager` - Restart the active network manager
- `renew_dhcp` - Request new IP via DHCP
- `restart_resolver` - Restart systemd-resolved
- `start_manager` - Start stopped network manager
- `disable_conflicting` - Disable conflicting service (high risk)
- `toggle_wifi_powersave` - Disable WiFi power saving

Each playbook has:
- Risk level (Low/Medium/High)
- Confirmation phrase requirement
- Post-checks to verify fix
- Rollback steps if verification fails

**Case File Integration:**
- `networking_doctor.json` in case files
- Full diagnosis result stored
- Fix applied and result tracked
- Recipe creation on successful fix

### Files Changed

- **NEW:** `crates/anna_common/src/networking_doctor.rs` - Full networking doctor implementation (~1800 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added module and exports
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.38
- **MODIFIED:** `CLAUDE.md` - Version bump
- **MODIFIED:** `README.md` - Version bump and feature description
- **MODIFIED:** `TODO.md` - Version bump

### Example Transcript: WiFi Disconnect Diagnosis

```
=== NETWORKING DOCTOR DIAGNOSIS ===

Network Manager: NetworkManager (running)

Interfaces:
  wlan0 (wifi) - UP - 192.168.1.100/24 [WiFi: HomeNetwork]
  enp0s3 (ethernet) - DOWN - no IP

Diagnosis Flow:
  [OK] physical_link
    1 interface(s) up: wlan0 (WiFi: HomeNetwork)
    -> Physical layer is working
  [OK] ip_address
    wlan0: 192.168.1.100/24
    -> IP address acquired - DHCP or static config working
  [OK] default_route
    Default gateway: 192.168.1.1
    -> Routing table has a default route
  [FAIL] ip_connectivity
    Cannot ping gateway 192.168.1.1
    -> Gateway unreachable - check cable/WiFi or gateway device
  [SKIP] dns
    Skipped - no IP connectivity
    -> Cannot test DNS without IP connectivity
  [OK] manager_health
    NetworkManager running and healthy
    -> Network manager is functioning normally

Hypotheses:
  1. Gateway unreachable - network path issue (70% confidence)
     Fix available: restart_manager

Recommended Fix:
  Restart Network Manager - Restart the active network manager service
  Risk: low
  To apply, confirm with: I CONFIRM (apply fix)

Overall Status: DEGRADED
```

### Example Transcript: DNS Failure Diagnosis

```
=== NETWORKING DOCTOR DIAGNOSIS ===

Network Manager: NetworkManager (running)

Interfaces:
  enp0s3 (ethernet) - UP - 192.168.1.50/24

Diagnosis Flow:
  [OK] physical_link
    1 interface(s) up: enp0s3 (ethernet)
  [OK] ip_address
    enp0s3: 192.168.1.50/24
  [OK] default_route
    Default gateway: 192.168.1.1
  [OK] ip_connectivity
    Gateway 192.168.1.1 and 1.1.1.1 reachable
  [FAIL] dns
    DNS resolution failed (servers: 127.0.0.53, tests: archlinux.org: FAIL)
    -> Cannot resolve domain names - check DNS config
  [OK] manager_health
    NetworkManager running and healthy

Hypotheses:
  1. DNS resolution failure - resolver misconfigured (85% confidence)
     Fix available: restart_resolver
  2. systemd-resolved may need restart (75% confidence)
     Fix available: restart_resolver

Recommended Fix:
  Restart DNS Resolver - Restart systemd-resolved to fix DNS issues
  Risk: low
  To apply, confirm with: I CONFIRM (apply fix)

Overall Status: DEGRADED
```

### Tests Added

- `test_network_manager_detection` - Manager enum and service names
- `test_diagnosis_step_result` - Step result symbols
- `test_diagnosis_step_creation` - Step creation and pass/fail
- `test_network_evidence_creation` - Evidence struct initialization
- `test_fix_playbooks_exist` - All playbooks are defined
- `test_fix_playbook_confirmation` - Confirmation phrases required
- `test_diagnosis_status` - Status enum strings
- `test_hypothesis_generation_empty` - No hypotheses when all pass
- `test_parse_interface_line` - Ethernet interface parsing
- `test_parse_interface_line_wifi` - WiFi interface parsing
- `test_networking_doctor_case` - Case file JSON serialization
- `test_fix_risk_levels` - Risk level assignments
- `test_determine_status` - Status determination logic

---

## v0.0.37 - Recipe Engine v1 (Reusable Fixes + Safe Auto-Drafts)

**Release Date:** 2025-12-03

### Summary

Recipes now have full lifecycle management: Active, Draft, and Archived states. Recipes created from sessions with < 80% reliability are automatically saved as drafts (never auto-suggested). Drafts can be promoted to active after a successful validated run. Recipe events are tracked in case files for full audit trail.

### Key Features

**Recipe Status Lifecycle:**
- `RecipeStatus` enum: `Active`, `Draft`, `Archived`
- Active: Can be auto-suggested and executed
- Draft: Usable but never auto-suggested, awaits promotion
- Archived: Tombstone state for deleted recipes, not usable

**Creation Rules (>= 80% reliability threshold):**
- Reliability >= 80%: Recipe created as Active
- Reliability < 80%: Recipe created as Draft
- Drafts can be promoted after successful validated run
- `promote()` method on Recipe struct

**Recipe Schema Enhancements:**
- `intent_tags: Vec<String>` - Semantic tags for better matching
- `evidence_required: Vec<String>` - Evidence to collect before execution
- `post_checks: Vec<PostCheck>` - Verification after execution
- `origin_case_id: Option<String>` - Links to creating case
- `notes: String` - Free-form notes
- `confirmation_phrase: Option<String>` - Replaces `confirmation_required`

**Post-Check Types:**
- `ServiceRunning { name }` - Check if service is running
- `FileExists { path }` - Check if file exists
- `CommandSucceeds { command }` - Check if command exits 0
- `OutputContains { command, expected }` - Check command output

**Recipe Matching Improvements:**
- Match scoring uses `intent_tags` for better relevance
- Draft recipes return 0.0 match score (excluded from auto-suggest)
- Active recipes ranked by confidence and success_count

**Case File Recipe Events:**
- `RecipeEvent` struct with event types:
  - `Matched` - Recipe matched for this case
  - `Executed` - Recipe was applied
  - `Succeeded` - Recipe execution succeeded
  - `Failed` - Recipe execution failed
  - `Created` - Recipe was created from this case
  - `Promoted` - Recipe was promoted from draft
- Case files now have `recipe_events: Vec<RecipeEvent>`

**Recipe Introspection Methods:**
- `is_usable()` - Active or Draft (not Archived)
- `can_auto_suggest()` - Active with confidence >= 0.5
- `promote()` - Promote Draft to Active

### Files Changed

- **MODIFIED:** `crates/anna_common/src/recipes.rs` - RecipeStatus enum, PostCheck types, new fields, lifecycle methods
- **MODIFIED:** `crates/anna_common/src/transcript.rs` - RecipeEvent, RecipeEventType, case file integration
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Exported new types (RecipeStatus, RecipePostCheck, RecipePostCheckType, RecipeEvent, RecipeEventType)
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.37
- **MODIFIED:** `CLAUDE.md` - Version bump to 0.0.37

### Tests Added

**Recipe Tests:**
- `test_recipe_status_default` - Active by default
- `test_recipe_status_draft` - Draft behavior
- `test_recipe_status_archived` - Archived behavior
- `test_recipe_promote` - Promote draft to active
- `test_recipe_promote_already_active` - No-op for active
- `test_recipe_promote_archived_fails` - Cannot promote archived
- `test_recipe_draft_no_match_score` - Drafts don't match
- `test_post_check_types` - All PostCheckType variants
- `test_recipe_intent_tags` - Intent tags in matching
- `test_recipe_status_from_reliability_high` - >= 80% = Active
- `test_recipe_status_from_reliability_low` - < 80% = Draft
- `test_recipe_format_detail_with_status` - Status in output

**Recipe Event Tests:**
- `test_recipe_event_matched` - Matched event creation
- `test_recipe_event_executed` - Executed event creation
- `test_recipe_event_succeeded` - Succeeded event creation
- `test_recipe_event_failed` - Failed event with error message
- `test_recipe_event_created` - Created event with notes
- `test_recipe_event_promoted` - Promoted event creation
- `test_case_file_with_recipe_events` - Case file integration
- `test_case_file_recipe_created` - Recipe creation tracking
- `test_recipe_event_types_serialization` - JSON roundtrip

---

## v0.0.36 - Knowledge Packs v1 (Offline Q&A with Strict Citations)

**Release Date:** 2025-12-03

### Summary

Anna can now answer "how do I..." questions using locally indexed documentation - man pages and package docs from `/usr/share/doc`. All factual claims require K-citations (K1, K2, K3...) that reference specific documentation excerpts. No network access, no hallucination.

### Key Features

**Knowledge Packs Storage:**
- Location: `/var/lib/anna/knowledge_packs/`
- SQLite FTS5 index for fast full-text search
- Pack types: `manpages`, `package_docs`
- Trust level tracking: `official` (system docs), `local`, `user`

**Citation System:**
- K-citations: [K1], [K2], [K3]... for knowledge references
- Each citation links to: title, pack, source path, trust level, excerpt
- Junior rejects uncited factual claims
- "How do I..." questions search knowledge first

**Status Visibility:**
- `annactl status` [KNOWLEDGE] section shows:
  - Pack count and document count
  - Index size and last indexed time
  - Breakdown by source type
  - Top packs by query count

**Auto-Build on Daemon Start:**
- Knowledge packs build automatically on first run
- Limited to 500 manpages and 500 package docs for fast startup
- Full index available for subsequent queries

**Case File Integration:**
- New `knowledge_refs` field in case files
- Records: evidence_id, title, pack_id, pack_name, source_path, trust, excerpt
- Full audit trail of which docs were used for each answer

### Files Changed

- **MODIFIED:** `crates/anna_common/src/knowledge_packs.rs` - Updated header, added citation tests
- **MODIFIED:** `crates/anna_common/src/transcript.rs` - Added KnowledgeRef struct, knowledge_refs field
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Exported CaseKnowledgeRef
- **MODIFIED:** `crates/annactl/src/commands/status.rs` - Re-enabled [KNOWLEDGE] section
- **MODIFIED:** `crates/annad/src/main.rs` - Auto-build knowledge packs on start

### Example Q&A with Citations

```
[you] to [anna]: how do I connect via SSH?

[anna] to [junior]: Let me search the knowledge packs...

[junior] to [anna]: Found relevant documentation:
  [K1] ssh - OpenSSH remote login client (man:ssh, official)
  [K2] openssh - README (/usr/share/doc/openssh/README, official)

[anna] to [you]:
To connect via SSH, use:

    ssh user@hostname [K1]

Configuration files are in /etc/ssh/ for system-wide settings
and ~/.ssh/ for user settings [K2].

Reliability: 85% (cited from official documentation)
```

### Tests Added

- `test_case_file_with_knowledge_refs` - Single knowledge reference
- `test_case_file_multiple_knowledge_refs` - Multiple references
- `test_citation_format_for_qa` - Citation format verification
- `test_search_how_to_question` - "How do I..." query handling

Tests: 644 passed

### Migration Notes

- Existing case files remain compatible (knowledge_refs defaults to empty)
- Knowledge packs build automatically on first daemon start
- No configuration changes required

---

## v0.0.35 - Ollama Role Selection (Policy-Driven Model Management)

**Release Date:** 2025-12-03

### Summary

Anna now independently selects models for the Translator and Junior roles based on hardware capabilities and a policy-driven configuration. The system detects hardware (CPU, RAM, GPU, VRAM), benchmarks available models, and selects the best fit for each role's requirements.

### Key Features

**Policy-Driven Model Selection:**
- Configuration file: `/etc/anna/policy/models.toml`
- Separate requirements for Translator (fast, low latency) and Junior (higher reasoning)
- Scoring weights: latency (0.3), throughput (0.2), quality (0.4), memory_fit (0.1)
- Candidate models in preference order with automatic fallback

**Hardware Detection:**
- Detects: CPU cores, total RAM, GPU vendor (nvidia/amd/intel), VRAM
- Hardware tier classification: Low (<8GB RAM), Medium (8-16GB), High (>16GB or dedicated GPU)
- Hardware hash for change detection and rebenchmarking triggers

**Model Readiness UX:**
- `annactl status` [MODELS] section shows:
  - Hardware tier (Low/Medium/High)
  - Translator model and status
  - Junior model and status (or fallback mode)
  - Download progress with ETA when pulling models
  - Readiness summary (full capability / partial / not ready)

**Fallback Behavior:**
- Translator fallback: deterministic classifier when no model available
- Junior fallback: skip mode with reliability capped at 60%
- Clear visibility in status output: "reliability capped at 60%"

**Case Files with Model Info:**
- New `models` field in case files tracks which models were used
- Records: translator model, junior model (if any), fallback states, hardware tier
- Supports debugging and learning from model performance

### Files Changed

- **NEW:** `crates/anna_common/src/model_policy.rs` - Complete policy system (500+ lines)
  - `ModelsPolicy` with role-specific policies
  - `RolePolicy` for Translator and Junior requirements
  - `ScoringWeights` for model selection scoring
  - `DownloadProgress` and `DownloadStatus` for tracking
  - `ModelReadinessState` for case file serialization
  - Default policy with candidate models for each role
- **MODIFIED:** `crates/anna_common/src/model_selection.rs` - Added role field to DownloadProgress
- **MODIFIED:** `crates/anna_common/src/transcript.rs` - Added CaseModelInfo struct and models field
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Exported model_policy and CaseModelInfo
- **MODIFIED:** `crates/annactl/src/commands/status.rs` - Enhanced [MODELS] section
- **MODIFIED:** `crates/annad/src/llm_bootstrap.rs` - Track role for each model download

### Default Policy

```toml
[translator]
max_latency_ms = 2000
min_quality_tier = "low"
max_tokens = 512
candidates = ["qwen2.5:0.5b", "qwen2.5:1.5b", "llama3.2:1b", "phi3:mini", "gemma2:2b"]
fallback = "deterministic"

[junior]
max_latency_ms = 5000
min_quality_tier = "medium"
max_tokens = 1024
candidates = ["qwen2.5:1.5b-instruct", "qwen2.5:3b-instruct", "llama3.2:3b-instruct"]
fallback = "skip"
no_junior_max_reliability = 60
```

### Migration Notes

- Existing case files remain compatible (models field is optional)
- Policy file is auto-generated on first run if missing
- No configuration changes required for default behavior

---

## v0.0.34 - Fix-It Mode (Bounded Troubleshooting Loops)

**Release Date:** 2025-12-03

### Summary

Anna can now actively troubleshoot problems using a bounded state machine approach. When you report an issue like "WiFi keeps disconnecting" or "my service won't start", Anna enters Fix-It mode: collecting evidence, forming hypotheses, testing them, and proposing fixes with full rollback support.

### Key Features

**Fix-It Mode State Machine:**
- States: `understand` → `evidence` → `hypothesize` → `test` → `plan_fix` → `apply_fix` → `verify` → `close`
- Maximum 2 hypothesis cycles before declaring "stuck" with evidence of what was tried
- Every state transition is logged in the transcript and case file
- Stuck state reports: "what I tried" + "what evidence is missing"

**Problem Category Detection:**
- Automatic detection: Networking, Audio, Performance, SystemdService, Storage, Graphics, Boot
- Each category has a predefined tool bundle for baseline evidence collection
- Example: Networking bundle includes hw_snapshot, NetworkManager status, journal warnings

**Tool Bundles (Policy-Driven):**
```
Networking: hw_snapshot_summary, service_status(NetworkManager), journal_warnings(NetworkManager, wpa_supplicant)
Audio: hw_snapshot_summary, service_status(pipewire/pulseaudio), journal_warnings(pipewire)
Performance: hw_snapshot_summary, top_resource_processes, slowness_hypotheses, what_changed
SystemdService: sw_snapshot_summary, journal_warnings
Storage: disk_usage, hw_snapshot_summary
```

**Change Sets (Mutation Batches):**
- Group 1-5 mutations into a single confirmation
- Each change specifies: what, why, risk, rollback action, post-check
- Single confirmation phrase: `I CONFIRM (apply fix)`
- Automatic rollback in reverse order if any post-check fails

**Fix Timeline Tracking:**
- New `fix_timeline.json` in case files for troubleshooting sessions
- Records: problem statement, category, all hypotheses, selected fix, state transitions
- Full audit trail for debugging and learning

**Intent Classification:**
- New `IntentType::FixIt` for troubleshooting requests
- Detection patterns: "fix my X", "X is broken", "keeps disconnecting", "won't start", etc.
- Automatic routing to Fix-It mode state machine

### Files Changed

- **NEW:** `crates/anna_common/src/fixit.rs` - Complete Fix-It mode system (400 lines)
  - `FixItState` enum with state machine
  - `FixItSession` for tracking troubleshooting progress
  - `ProblemCategory` with tool bundle mapping
  - `Hypothesis` and `HypothesisTestResult` for evidence-based reasoning
  - `ChangeSet` and `ChangeItem` for mutation batches
  - `FixTimeline` for audit trail
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added fixit module exports
- **MODIFIED:** `crates/anna_common/src/transcript.rs` - Added fix_timeline field and save
- **MODIFIED:** `crates/annactl/src/pipeline.rs` - Added IntentType::FixIt and detection
- **MODIFIED:** `Cargo.toml` - Version 0.0.34
- **MODIFIED:** `CLAUDE.md` - Version 0.0.34
- **MODIFIED:** `TODO.md` - Version 0.0.34
- **MODIFIED:** `README.md` - Version 0.0.34 with Fix-It description

### Tests Added

- `test_is_fixit_request` - Detection of troubleshooting requests
- `test_problem_category_detection` - Category classification from problem descriptions
- `test_fixit_session_creation` - Session initialization with category
- `test_fixit_session_transitions` - State machine transitions
- `test_hypothesis_cycle_limit` - Max 2 cycles enforcement
- `test_change_set_limit` - Max 5 mutations per batch
- `test_tool_bundles` - Correct tools for each category
- `test_change_set_confirmation_format` - User-facing confirmation display

### Example Fix-It Transcript

```
[you] to [anna]:
  WiFi keeps disconnecting

[anna] to [translator]:
  Classify: "WiFi keeps disconnecting"

[translator] to [anna]:
  INTENT: fix_it
  CATEGORY: networking
  TOOLS: hw_snapshot_summary, service_status(NetworkManager), journal_warnings(NetworkManager, 30)

[state] understand -> evidence
  reason: Starting evidence collection for networking issue

[anna] to [annad]:
  Execute tool bundle: networking

[annad] to [anna]:
  [E1] hw_snapshot_summary: Intel WiFi 6 AX200, driver iwlwifi
  [E2] service_status(NetworkManager): active (running)
  [E3] journal_warnings: "wlp0s20f3: deauthenticating by local choice (Reason: 3=DEAUTH_LEAVING)"

[state] evidence -> hypothesize
  reason: Evidence collected, forming hypotheses

[anna] to [junior]:
  Hypotheses based on [E1], [E2], [E3]:
  H1: Power management causing disconnects (confidence: 75%, evidence: [E3])
  H2: Driver bug in iwlwifi (confidence: 40%, evidence: [E1])

[junior] to [anna]:
  SCORE: 72
  CRITIQUE: H1 is well-supported by deauth reason. Suggest testing power management first.

[state] hypothesize -> plan_fix
  reason: H1 selected, planning fix

[anna] to [you]:
  ╭─────────────────────────────────────────────────────────────────╮
  │ Change Set: 18e4a7f2-3c1d (1 changes)
  ├─────────────────────────────────────────────────────────────────┤
  │ 1. Disable WiFi power management
  │    Why: Prevent power-save triggered disconnects
  │    Risk: low
  │    Rollback: Re-enable power management
  ├─────────────────────────────────────────────────────────────────┤
  │ To apply, type: I CONFIRM (apply fix)
  ╰─────────────────────────────────────────────────────────────────╯
```

### Rollback Behavior Example

```
[state] apply_fix -> verify
  reason: Change applied, verifying

[annad] to [anna]:
  Post-check FAILED: Network still disconnecting after 30 seconds

[state] verify -> apply_fix
  reason: Post-check failed, rolling back

[anna] to [annad]:
  Rollback: Re-enable power management

[anna] to [you]:
  Fix attempt failed. Rolled back change.
  Hypothesis H1 (power management) was not the cause.
  Cycle 1/2 complete. Trying next hypothesis...

[state] apply_fix -> hypothesize
  reason: Rollback complete, trying H2
```

---

## v0.0.33 - Human-First Transcript + Local Case Files

**Release Date:** 2025-12-03

### Summary

Anna now includes a human-first debug transcript system and local case file storage for troubleshooting. Debug levels now control verbosity, not readability - transcripts are always human-readable. Development mode defaults to maximum verbosity. Case files are stored locally per request and can be queried via natural language.

### Key Features

**Human-First Transcript Renderer:**
- `TranscriptBuilder` produces human-readable output at all debug levels
- Format: `[you] to [anna]:` style with proper text wrapping
- Debug level 0: User-facing messages only
- Debug level 1: Includes internal Anna/Translator dialogue
- Debug level 2: Full verbosity with Junior/Senior dialogue
- Evidence citations displayed inline with messages
- Automatic text wrapping to terminal width

**Local Case Files:**
- Case files stored per request at `/var/lib/anna/cases/YYYY/MM/DD/<request_id>/`
- Each case contains:
  - `summary.txt` - Human-readable case summary
  - `transcript.log` - Full debug transcript
  - `evidence.json` - Evidence entries with tool calls
  - `policy_refs.json` - Policy references cited
  - `timing.json` - Timing breakdown per phase
  - `result.json` - Outcome and reliability score
- Atomic file writes with secret redaction
- Automatic pruning (30 days retention, 1GB max)

**Case Retrieval via Natural Language:**
- "Show me the last case summary" - Displays most recent case
- "Show me what happened in the last failure" - Shows last failed case
- "List today's cases" - Shows today's cases
- "List recent cases" - Shows recent case summaries
- New tools: `last_case_summary`, `last_failure_summary`, `list_today_cases`, `list_recent_cases`
- Intent classifier detects case-related queries

**Dev Mode Defaults:**
- `dev_mode = true` in config during development
- `debug_level = 2` default for maximum verbosity
- `UiConfig.is_dev_mode()` and `effective_debug_level()` helpers
- [CASES] section in status shows dev mode indicator

**Status Additions:**
- New [CASES] section showing:
  - Dev mode status
  - Recent case count with success/failure breakdown
  - Case storage usage
  - Last failure summary
  - Latest case list (in non-compact mode)

**Helper Improvements:**
- `ethtool` now always installed (useful for USB/Thunderbolt adapters, wifi stats)
- Relevance check removed - network diagnostics always available

### Files Changed

- **NEW:** `crates/anna_common/src/transcript.rs` - Complete transcript and case file system (680+ lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added transcript module exports
- **MODIFIED:** `crates/anna_common/src/config.rs` - Added dev_mode, updated debug_level default to 2
- **MODIFIED:** `crates/anna_common/src/tools.rs` - Added case retrieval tools
- **MODIFIED:** `crates/anna_common/src/tool_executor.rs` - Added case tool executors
- **MODIFIED:** `crates/anna_common/src/helpers.rs` - Made ethtool always relevant
- **MODIFIED:** `crates/annactl/src/pipeline.rs` - Added case query detection
- **MODIFIED:** `crates/annactl/src/commands/status.rs` - Added [CASES] section
- **MODIFIED:** `Cargo.toml` - Version 0.0.33
- **MODIFIED:** `CLAUDE.md` - Version 0.0.33
- **MODIFIED:** `TODO.md` - Version 0.0.33
- **MODIFIED:** `README.md` - Version 0.0.33

### Tests Added

- `test_transcript_builder` - TranscriptBuilder message handling
- `test_transcript_debug_levels` - Debug level filtering
- `test_case_file_evidence` - Evidence entry management
- `test_case_file_outcome` - Outcome and error handling
- `test_case_file_timing` - Timing information
- `test_redact_transcript` - Secret redaction in case files
- `test_case_outcome_partial` - Partial completion handling

---

## v0.0.32 - CI Hardening + "No Regressions" Gate

**Release Date:** 2025-12-03

### Summary

Anna now has comprehensive CI/CD infrastructure that enforces quality gates on Arch Linux, the only supported platform. All builds, tests, and releases run in Arch Linux containers with strict validation of version consistency, documentation, and code quality.

### Key Features

**GitHub Actions CI (Arch-Only):**
- Build matrix: debug + release profiles in `archlinux:latest` container
- Unit tests and integration tests
- Clippy lints (advisory)
- Rustfmt check (advisory)
- Security audit via cargo-audit (advisory)
- Smoke tests for CLI verification
- Repo hygiene checks
- Policy/redaction security tests
- Final "CI Pass" gate requiring all critical jobs

**Smoke Tests:**
- `annactl --version` must succeed and show valid version
- `annactl --help` must not show legacy commands (sw, hw)
- `annactl status` must handle missing daemon gracefully
- `annactl <request>` must produce Reliability score even in fallback mode

**Release Pipeline Guardrails:**
- Triggered on `v*.*.*` tags only
- Validates version consistency:
  - Cargo.toml version matches tag
  - README.md updated for version
  - CLAUDE.md updated for version
  - RELEASE_NOTES.md has entry for version
  - TODO.md version matches
- Builds in Arch Linux container
- Runs full test suite
- Generates SHA256 checksums
- Extracts release notes automatically
- Creates GitHub release

**Repo Hygiene Checks:**
- No legacy commands (annactl sw, annactl hw, --json) in docs
- Version consistency across all documentation files
- Root-level file allowlist enforcement
- RELEASE_NOTES.md must have current version entry

**Documentation Updates:**
- Explicit "Arch Linux only" support statement
- CI badge in README
- CI/CD section explaining pipeline
- CLAUDE.md updated with CI rules:
  - No green, no merge
  - No release without updated docs
  - No regressions allowed

### Files Changed

- **NEW:** `.github/workflows/ci.yml` - Comprehensive CI workflow
- **MODIFIED:** `.github/workflows/release.yml` - Added guardrails
- **MODIFIED:** `README.md` - v0.0.32, CI badge, Arch-only statement, CI/CD section
- **MODIFIED:** `CLAUDE.md` - v0.0.32, CI rules, platform support
- **MODIFIED:** `Cargo.toml` - version 0.0.32
- **MODIFIED:** `TODO.md` - version 0.0.32

### CI Jobs Summary

| Job | Purpose | Blocking? |
|-----|---------|-----------|
| build (debug) | Compile in debug mode | Yes |
| build (release) | Compile in release mode | Yes |
| test | Unit + integration tests | Yes |
| clippy | Lint warnings | No (advisory) |
| fmt | Format check | No (advisory) |
| audit | Security vulnerabilities | No (advisory) |
| smoke | CLI verification | Yes |
| hygiene | Repo cleanliness | Yes |
| security-tests | Redaction/policy tests | Yes |
| ci-pass | Final gate | Yes |

### Release Validation Checks

1. Tag version matches Cargo.toml
2. README.md mentions version
3. CLAUDE.md mentions version
4. RELEASE_NOTES.md has version entry
5. TODO.md has current version
6. All tests pass
7. Binaries build successfully

---

## v0.0.31 - Reliability Engineering Integration

**Release Date:** 2025-12-03

### Summary

Anna now tracks her own reliability metrics in real-time with proper integration into the request pipeline. Metrics (request success/failure, tool success/failure, LLM timeouts, cache hits) are recorded during every request. Error budgets trigger alerts when thresholds are exceeded. Self-diagnostics reports can be generated via natural language requests.

### Key Features

**Pipeline Metrics Integration:**
- Metrics recording wired into `process()` function in pipeline.rs
- RequestStart/Success recorded at beginning/end of each request
- TranslatorStart/Success/Timeout recorded for Translator LLM calls
- ToolStart/Success/Failure recorded for each tool execution
- JuniorStart/Success/Timeout recorded for Junior LLM calls
- Latency samples recorded for translator, tools, junior, and e2e
- CacheHit/CacheMiss recorded based on cache status
- Metrics automatically pruned and saved after each request

**Natural Language Support for Reliability Tools:**
- Translator system prompt updated with reliability tools (v0.0.31)
- New tools available: self_diagnostics, metrics_summary, error_budgets
- Source planning rules for "diagnostics report", "metrics", "error budget"
- Deterministic fallback handles reliability keywords
- Tool plan generator creates plans for reliability evidence needs

**Tool Format Examples:**
```
TOOLS: self_diagnostics
TOOLS: metrics_summary(days=7), error_budgets
```

**User Queries Supported:**
- "Generate a self-diagnostics report"
- "Show me the error budgets"
- "What are my reliability metrics?"
- "Generate a bug report"

### Files Changed

- **MODIFIED:** `crates/annactl/src/pipeline.rs`
  - Added `MetricsStore, MetricType` imports
  - Added metrics recording in `process()` function
  - Updated TRANSLATOR_SYSTEM_PROMPT with reliability tools
  - Added reliability keywords to `classify_intent_deterministic()`
  - Added reliability tools to `generate_tool_plan_from_evidence_needs()`

- **MODIFIED:** `Cargo.toml` - version 0.0.31
- **MODIFIED:** `CLAUDE.md` - version 0.0.31
- **MODIFIED:** `README.md` - v0.0.31 documentation
- **MODIFIED:** `TODO.md` - version 0.0.31

### Metrics Recording Points

```rust
// Start of request
metrics.record(MetricType::RequestStart);

// Translator LLM
metrics.record(MetricType::TranslatorStart);
metrics.record(MetricType::TranslatorSuccess); // or TranslatorTimeout

// Tools
for each tool:
  metrics.record(MetricType::ToolStart);
  metrics.record(MetricType::ToolSuccess); // or ToolFailure

// Junior LLM
metrics.record(MetricType::JuniorStart);
metrics.record(MetricType::JuniorSuccess); // or JuniorTimeout

// End of request
metrics.record(MetricType::RequestSuccess);
metrics.record_latency("e2e", total_ms);
metrics.record(MetricType::CacheHit); // or CacheMiss
metrics.save();
```

### Tests

- All 45 existing tests pass (25 pipeline unit tests + 20 CLI tests)
- 14 reliability module tests pass
- No regressions in existing functionality

---

## v0.0.30 - Helper Auto-Installation on Daemon Start

**Release Date:** 2025-12-03

### Summary

Anna now automatically installs missing helpers on daemon startup and maintains them via periodic health checks. If a user removes an Anna-installed helper, it will be reinstalled on the next health check (every 10 minutes).

### Key Features

- `install_missing_helpers()` called on daemon startup
- Periodic helper health check every 10 minutes (HELPER_CHECK_INTERVAL_SECS = 600)
- Auto-reinstall of missing helpers during health checks
- Helper tracking maintained in manifest

---

## v0.0.29 - Auto-Update Artifact Name Fix

**Release Date:** 2025-12-03

### Summary

Fixed auto-update artifact name matching to handle architecture suffixes. Updates now properly detect and install new versions from GitHub releases with names like `annad-0.0.29-x86_64-unknown-linux-gnu`.

### Key Features

- `fetch_github_releases()` uses `starts_with("annad-")` pattern
- `atomic_install()` extracts base name from versioned artifacts

---

## v0.0.28 - System-Aware Helper Filtering & Reliability Improvements

**Release Date:** 2025-12-03

### Summary

Anna now shows only helpers that are relevant to your specific hardware. No more seeing ethtool if you have no ethernet, no nvme-cli if you have no NVMe drives. Improved Ollama detection reliability and cleaner status display.

### Key Features

**System-Aware Helper Filtering (v0.0.28):**
- `RelevanceCheck` enum for hardware-based filtering
- Checks: `HasEthernet`, `HasWiFi`, `HasNvme`, `HasSata`, `Always`
- `get_relevant_helper_definitions()` filters helpers by system hardware
- Hardware detection via `/sys/class/net`, `/sys/class/nvme`, `/sys/block`, `/sys/class/ata_device`
- Only shows helpers that are useful for YOUR specific machine

**Helper Relevance Mapping:**
- `ethtool` - Only shown if ethernet interfaces exist
- `iw` - Only shown if WiFi interfaces exist
- `nvme-cli` - Only shown if NVMe devices exist
- `smartmontools` - Only shown if SATA devices exist
- `hdparm` - Only shown if SATA devices exist
- `lm_sensors`, `usbutils`, `pciutils`, `ollama` - Always shown

**Improved Ollama Detection:**
- Fixed `get_helper_status_list()` to use `check_helper_presence()` with `provides_command` check
- Ollama correctly detected via `which ollama`, not just `pacman -Qi`
- Unified detection logic across all helper functions

**Cleaner Status Display:**
- Removed confusing INSTALL REVIEW section from `annactl status`
- Helpers display shows only relevant helpers
- Install script output updated to remove legacy sw/hw commands
- Now shows: `annactl status`, `annactl "question"`, `annactl` (REPL)

**README.md Updated:**
- Comprehensive documentation for v0.0.28
- CLI surface documentation
- Helper auto-installation explained
- Recent changes section

### Files Changed

- **MODIFIED:** `crates/anna_common/src/helpers.rs`
  - Added `RelevanceCheck` enum
  - Added `is_helper_relevant()` function
  - Added hardware detection: `has_ethernet_interfaces()`, `has_wifi_interfaces()`, `has_nvme_devices()`, `has_sata_devices()`
  - Added `get_relevant_helper_definitions()`
  - Updated `get_helper_status_list()` to use relevant helpers only
  - Updated `refresh_helper_states()` to use relevant helpers only
  - Added `relevance_check` field to `HelperDefinition`

- **MODIFIED:** `crates/annactl/src/commands/status.rs`
  - Removed `print_installer_review_section()` function
  - Removed `InstallState` import

- **MODIFIED:** `scripts/install.sh`
  - Updated completion message to show correct CLI surface
  - Removed legacy sw/hw command references

- **MODIFIED:** `README.md`
  - Complete rewrite for v0.0.28

### Hardware Detection Logic

```rust
// Ethernet: Check /sys/class/net for eth*, en*, em* interfaces
// WiFi: Check for /sys/class/net/<name>/wireless or wlan*/wlp* names
// NVMe: Check /sys/class/nvme has entries
// SATA: Check /sys/class/ata_device or sd* devices with "ata" in path
```

### Example Status (System with WiFi only, NVMe only)

Before v0.0.28:
```
[HELPERS]
  ethtool       missing (Anna will install when needed)
  smartmontools missing (Anna will install when needed)
  nvme-cli      missing (Anna will install when needed)
  ...8 more helpers...
```

After v0.0.28:
```
[HELPERS]
  iw            present (user-installed)
  nvme-cli      present (Anna-installed)
  lm_sensors    present (user-installed)
  ...only relevant helpers shown...
```

---

## v0.0.22 - Reliability Engineering (Metrics, Error Budgets, and Self-Diagnostics)

**Release Date:** 2025-12-03

### Summary

Anna now tracks her own reliability like an SRE-managed service. Request success rates, tool failures, mutation rollbacks, and LLM timeouts are continuously recorded with configurable error budgets that trigger alerts when burned. A new `self_diagnostics()` tool generates comprehensive health reports with evidence IDs for every claim.

### Key Features

**Metrics Collection:**
- Structured metrics stored locally in `/var/lib/anna/internal/metrics.json`
- Tracks 16 metric types: requests, tools, mutations, LLM timeouts, cache hits/misses
- Rolling 7-day retention with daily aggregation
- Latency recording with p50/p95 percentile calculation

**Error Budgets (SRE-Style):**
- Configurable thresholds in `config.toml`:
  - Request failures: max 1% per day (default)
  - Tool failures: max 2% per day (default)
  - Mutation rollbacks: max 0.5% per day (default)
  - LLM timeouts: max 3% per day (default)
- Budget states: Ok, Warning (50% burned), Critical (80% burned), Exhausted (100%+)
- Automatic alerts generated when budgets are burned

**Self-Diagnostics Tool:**
- `self_diagnostics()` - read-only tool generating comprehensive health reports
- Sections with individual evidence IDs:
  - Version information
  - Install review status
  - Update state (channels, phases)
  - Model readiness (Ollama, selected models)
  - Policy status (capabilities, blocked, risk files)
  - Storage usage
  - Error budget consumption
  - Recent errors (redacted)
  - Active alerts
- Overall status derived from worst section status
- Redaction applied to error messages (secrets never exposed)

**Additional Tools:**
- `metrics_summary(days)` - reliability metrics for specified period
- `error_budgets()` - current budget consumption status

**Status Display ([RELIABILITY] section):**
```
[RELIABILITY]
  Status:           healthy

  Error Budgets (today):
    request_failures:  0.0% / 1.0% [OK]
    tool_failures:     0.0% / 2.0% [OK]
    mutation_rollback: 0.0% / 0.5% [OK]
    llm_timeouts:      0.0% / 3.0% [OK]

  Request Success Rate (7d): 100.0% (42/42)
  Latency (7d): p50=1234ms, p95=2456ms
```

### Configuration (config.toml)

```toml
[reliability]
# Error budget thresholds (percentages)
[reliability.error_budgets]
request_failure_percent = 1.0
tool_failure_percent = 2.0
mutation_rollback_percent = 0.5
llm_timeout_percent = 3.0

# Metrics settings
[reliability.metrics]
retention_days = 7
```

### New Types

- `MetricType` - 16 metric categories (RequestSuccess, ToolFailure, etc.)
- `DailyMetrics` - per-day aggregated counts and latencies
- `MetricsStore` - persistent metrics storage with schema versioning
- `ErrorBudgets` - configurable budget thresholds
- `BudgetStatus` - current consumption state per category
- `BudgetState` - Ok, Warning, Critical, Exhausted
- `BudgetAlert` - generated alert with severity and message
- `DiagnosticsReport` - full health report with sections
- `DiagnosticsSection` - individual section with evidence ID

### Files Changed

- **NEW:** `crates/anna_common/src/reliability.rs` (~1300 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - module + exports
- **MODIFIED:** `crates/anna_common/src/config.rs` - ReliabilityConfig
- **MODIFIED:** `crates/anna_common/src/tools.rs` - 3 new tools
- **MODIFIED:** `crates/anna_common/src/tool_executor.rs` - tool implementations
- **MODIFIED:** `crates/annactl/src/commands/status.rs` - [RELIABILITY] section

### Tests

14 unit tests covering:
- Metric type string conversions
- Daily metrics increment/get
- Latency percentile calculations (p50, p95)
- Budget state calculation (Ok, Warning, Critical, Exhausted)
- Budget status calculation from metrics
- Budget alert generation on threshold breach
- Warning threshold detection
- Error budgets default values
- Ops log entry creation
- Diagnostics status strings
- Metrics store defaults
- Average latency calculation

---

## v0.0.21 - Performance and Latency Sprint

**Release Date:** 2025-12-03

### Summary

Anna now feels snappier with time-to-first-output (TTFO) optimizations, intelligent caching for repeated queries, and strict token budgets per role. The goal: always on, always fast, always transparent.

### Key Features

**Time-to-First-Output (TTFO) < 150ms:**
- Immediate header line and request display
- "I'm starting analysis and gathering evidence..." indicator
- No waiting for LLM before showing progress
- Streaming updates as work progresses

**Token Budgets Per Role:**
- `translator.max_tokens = 256` with `translator.max_ms = 1500`
- `junior.max_tokens = 384` with `junior.max_ms = 2500`
- Graceful degradation if exceeded (recorded, reliability adjusted)
- Configurable via `config.toml [performance]` section

**Read-Only Tool Result Caching:**
- Cache keyed by: tool name + args + snapshot version hash
- Default TTL: 5 minutes (configurable)
- Storage: `/var/lib/anna/internal/cache/tools/`
- Repeated queries like "what changed last 14 days" are instant

**LLM Response Caching (Safe):**
- Cache Translator plans and Junior critiques when:
  - Same request text (hashed)
  - Same evidence hashes
  - Same policy version
  - Same model version
- Redaction occurs before caching (secrets never stored)
- Default TTL: 10 minutes (configurable)
- Storage: `/var/lib/anna/internal/cache/llm/`

**Performance Statistics:**
- Latency samples tracked per request (total, translator, tools, junior)
- Cache hit rate calculated and displayed
- Top cached tools tracked
- Budget violations recorded

**Status Display ([PERFORMANCE] section):**
```
[PERFORMANCE]
  Samples:    42 (last 24h)
  Avg total:  1234ms
  Translator: 456ms avg
  Junior:     678ms avg
  Cache hit:  73% (31 hits, 11 misses)

  Top cached tools:
    hw_snapshot_summary (12 hits)
    sw_snapshot_summary (8 hits)
    recent_installs (6 hits)

  Cache storage:
    Tool cache:  15 entries (24 KB)
    LLM cache:   8 translator, 6 junior (48 KB)
```

### Configuration (config.toml)

```toml
[performance]
# Token budgets
[performance.budgets]
translator_max_tokens = 256
translator_max_ms = 1500
junior_max_tokens = 384
junior_max_ms = 2500
log_overruns = true

# Cache settings
[performance.cache]
tool_cache_enabled = true
tool_cache_ttl_secs = 300
llm_cache_enabled = true
llm_cache_ttl_secs = 600
max_entries = 1000
```

### New Types

- `TokenBudget`: Max tokens and time per role
- `BudgetSettings`: Configuration for all role budgets
- `BudgetViolation`: Recorded when budgets exceeded
- `ToolCacheKey`, `ToolCacheEntry`, `ToolCache`: Tool result caching
- `LlmCacheKey`, `LlmCacheEntry`, `LlmCache`: LLM response caching
- `PerfStats`, `LatencySample`: Performance tracking
- `PerformanceConfig`, `PerformanceBudgets`, `PerformanceCacheConfig`: Config types

### Cache Invalidation

Caches are automatically invalidated when:
- Snapshot data changes (tool cache)
- Policy files change (LLM cache)
- Model version changes (LLM cache)
- TTL expires (both caches)

### Files Changed

- NEW: `crates/anna_common/src/performance.rs` (~600 lines)
- MODIFIED: `crates/anna_common/src/lib.rs` (module + exports)
- MODIFIED: `crates/anna_common/src/config.rs` (PerformanceConfig)
- MODIFIED: `crates/annactl/src/pipeline.rs` (TTFO, latency tracking)
- MODIFIED: `crates/annactl/src/commands/status.rs` ([PERFORMANCE] section)

### Tests

- 10 unit tests for token budgets and cache key determinism
- Cache key determinism tests (args order, snapshot hash, policy version)
- Budget exceeded detection
- Performance stats calculations

---

## v0.0.20 - Ask Me Anything Mode (Source-Labeled Answers)

**Release Date:** 2025-12-03

### Summary

Anna now answers questions like a real senior Linux admin with clear source attribution. Every factual claim in responses is labeled with its source: system evidence [E#], knowledge documentation [K#], or explicit reasoning (Reasoning). The Translator plans the right source mix based on question type, and Junior penalizes unlabeled claims.

### Key Features

**Source Labeling System:**
- `[E#]` for system evidence (measurements, snapshots, tool output)
- `[K#]` for knowledge pack documentation (man pages, package docs)
- `(Reasoning)` for general inference not from direct evidence
- All factual claims must be labeled - unlabeled claims reduce reliability score

**Question Type Classification:**
- `HowTo`: "How do I...?" → knowledge packs first, then system tools
- `SystemStatus`: "What is happening on my machine?" → system evidence first
- `Mixed`: Both how-to and system state → query both sources
- `General`: Documentation or reasoning

**New Read-Only Tools:**
- `answer_context()`: Target user, distro, kernel, available knowledge packs
- `source_plan(request)`: Plans which sources to query with rationale
- `qa_stats()`: Today's Q&A statistics

**Junior Source Label Enforcement:**
- New penalty: `-10` for factual claim without [E#], [K#], or (Reasoning) label
- UNCITED_CLAIMS output now checks for missing source labels
- Examples in prompt show good vs bad source labeling

**Status Display ([Q&A TODAY] section):**
```
[Q&A TODAY]
  Answers:    5
  Avg reliability: 82%
  Citations:  K:8 E:12 R:3
  Top sources: system_evidence: 12, knowledge_docs: 8
```

**MissingEvidenceReport for "I don't know":**
- Explicit list of what information is missing
- Suggested read-only tools to gather evidence
- Suggested knowledge queries
- No guessing - propose what could be checked

### Example Transcripts

**How-to Question (Knowledge Citations):**
```
[you] to [anna]: How do I enable syntax highlighting in Vim?

[translator] to [anna]:
  INTENT: question
  TOOLS: knowledge_search(query=vim syntax highlighting)
  RATIONALE: How-to question: searching knowledge packs for documentation

[anna] to [you]:
  To enable syntax highlighting in Vim [K1]:
  1. Add `syntax on` to your ~/.vimrc file [K1]
  2. For specific filetypes, use `filetype plugin on` [K1]

  (Reasoning) This is the standard approach for most Vim installations.

  Reliability: 88%
```

**System Question (Evidence Citations):**
```
[you] to [anna]: What did I install in the last 2 weeks that might slow my machine?

[translator] to [anna]:
  INTENT: system_query
  TOOLS: recent_installs(days=14), what_changed(days=14), slowness_hypotheses(days=14)
  RATIONALE: System status question: gathering package changes and performance data

[anna] to [you]:
  In the last 14 days, you installed 8 packages [E1]:
  - electron28 (large runtime) [E1]
  - docker-compose (background service) [E2]

  Slowness analysis [E3] suggests docker may be consuming resources.
  Current CPU: 45% [E4], Memory: 68% [E4]

  Reliability: 85%
```

### Files Changed

- **NEW:** `crates/anna_common/src/source_labels.rs` (~500 lines)
- `crates/anna_common/src/lib.rs` - Module export and version bump
- `crates/anna_common/src/tools.rs` - Added answer_context, source_plan, qa_stats tools
- `crates/anna_common/src/tool_executor.rs` - Tool execution handlers
- `crates/annactl/src/pipeline.rs` - Updated Translator and Junior prompts
- `crates/annactl/src/commands/status.rs` - [Q&A TODAY] section
- `Cargo.toml` - Version 0.0.20

### Tests

- 8 unit tests in source_labels.rs
- Question type classification
- Source plan creation
- Citation detection and counting
- Q&A statistics tracking

---

## v0.0.19 - Offline Documentation Engine (Knowledge Packs)

**Release Date:** 2025-12-03

### Summary

Anna now has a local knowledge base for answering general questions without network access. Knowledge packs index man pages, package documentation, project docs, and user notes with SQLite FTS5 for fast full-text search. Answers include evidence citations (K1, K2...) linking to source documents.

### Key Features

**Knowledge Pack System (`/var/lib/anna/knowledge_packs/`):**
- Pack sources: manpages, package_docs, project_docs, user_notes, archwiki_cache, local_markdown
- Trust levels: Official (system docs), Local (project docs), User (user notes)
- Retention policies: Permanent, RefreshOnUpdate, Manual
- Per-pack metadata: document count, index size, timestamps

**SQLite FTS5 Index:**
- Full-text search with relevance ranking
- Automatic triggers to keep FTS index synchronized
- Evidence ID generation (K1, K2, K3...) for citation
- Excerpt extraction with keyword highlighting
- Secrets hygiene applied to all excerpts

**Knowledge Pack Schema:**
```rust
pub struct KnowledgePack {
    pub id: String,
    pub name: String,
    pub source: PackSource,
    pub trust: TrustLevel,
    pub retention: RetentionPolicy,
    pub source_paths: Vec<String>,
    pub created_at: u64,
    pub last_indexed_at: u64,
    pub document_count: usize,
    pub index_size_bytes: u64,
    pub description: String,
    pub enabled: bool,
}

pub struct SearchResult {
    pub doc_id: i64,
    pub evidence_id: String,  // K1, K2, etc.
    pub title: String,
    pub pack_id: String,
    pub pack_name: String,
    pub source_path: String,
    pub trust: TrustLevel,
    pub excerpt: String,  // redacted
    pub score: f64,
    pub matched_keywords: Vec<String>,
}
```

**Default Pack Ingestion:**
- `ingest_manpages()`: Index man pages via apropos/man commands
- `ingest_package_docs()`: Index /usr/share/doc/* markdown and text files
- `ingest_project_docs()`: Index Anna's own documentation (README, CLAUDE.md, etc.)
- `ingest_user_note()`: Add user-provided documentation

**New Read-Only Tools:**
- `knowledge_search(query, top_k)`: Search indexed documentation, returns excerpts with evidence IDs
- `knowledge_stats()`: Get pack counts, document counts, index size, last indexed time

**Status Display ([KNOWLEDGE] section):**
```
[KNOWLEDGE]
  Packs:      3
  Documents:  1247
  Index size: 2.4 MB
  Last index: 5m ago

  By source:
    manpages: 1
    package_docs: 1
    project_docs: 1

  Top packs:
    System Man Pages (42 queries)
    Package Documentation (18 queries)
```

**Security:**
- Secrets hygiene (redact_evidence) applied to all excerpts
- Restricted paths blocked from indexing
- Trust level tracking for provenance

### Files Changed

- **NEW:** `crates/anna_common/src/knowledge_packs.rs` (~900 lines)
- `crates/anna_common/src/lib.rs` - Module export and version bump
- `crates/anna_common/src/tools.rs` - Added knowledge_search, knowledge_stats tools
- `crates/anna_common/src/tool_executor.rs` - Tool execution handlers
- `crates/annactl/src/commands/status.rs` - [KNOWLEDGE] section
- `Cargo.toml` - Version 0.0.19

### Tests

- 10+ unit tests in knowledge_packs.rs
- Pack creation and document indexing
- FTS search with relevance scoring
- Evidence ID generation
- Excerpt extraction and redaction

---

## v0.0.18 - Secrets Hygiene

**Release Date:** 2025-12-03

### Summary

Anna now automatically redacts secrets from all output to prevent credential leaks. Passwords, API keys, tokens, private keys, and other sensitive data are replaced with `[REDACTED:TYPE]` placeholders throughout the transcript, evidence, and LLM prompts. Evidence from restricted paths (like ~/.ssh) is blocked entirely.

### Key Features

**Centralized Redaction Module (redaction.rs):**
- 22 secret types with compiled regex patterns
- Type-specific placeholders: `[REDACTED:PASSWORD]`, `[REDACTED:API_KEY]`, `[REDACTED:PRIVATE_KEY]`, etc.
- Lazy static compilation for performance

**Patterns Detected:**
- Passwords: `password=`, `--password`, `pwd=`
- API Keys: `api_key=`, `x-api-key:`, etc.
- Bearer tokens: `Bearer xxx`, `Authorization: Bearer`
- JWT tokens: `eyJ...` format
- Private keys: `-----BEGIN PRIVATE KEY-----`
- SSH keys: `-----BEGIN OPENSSH PRIVATE KEY-----`
- PEM blocks: RSA, EC, PGP private keys
- AWS credentials: `AKIA...`, `aws_secret_access_key`
- Azure/GCP credentials
- Git credentials: `https://user:pass@...`
- Database URLs: `postgres://user:pass@...`
- Cookies: `Set-Cookie:`, `session_id=`
- Generic secrets: `_token=`, `_secret=`, `_key=`

**Evidence Restriction Policy:**
Paths that are NEVER excerpted (content replaced with policy message):
- `~/.ssh/**`
- `~/.gnupg/**`
- `/etc/shadow`, `/etc/gshadow`
- `/proc/*/environ`
- `~/.password-store/**`
- Browser credential files (key*.db, Login Data)
- Keyrings and credential managers

**Junior Leak Detection (Rules 20-24):**
```
SECRETS LEAK PREVENTION (v0.0.18):
20. Responses MUST NOT reveal: passwords, tokens, API keys, etc.
21. Evidence from restricted paths MUST show [REDACTED:TYPE]
22. If secrets detected: force redaction, cite redaction ID, downscore
23. Restricted paths NEVER excerpted
24. Examples: BAD vs GOOD patterns

PENALTIES:
- Secret revealed in response: -50 (SECURITY LEAK)
- Unredacted restricted path: -40 (RESTRICTED PATH VIOLATION)
- Missing redaction for secret: -30 (INCOMPLETE REDACTION)
```

**Redaction in All Outputs:**
- `dialogue()` and `dialogue_always()` apply redaction
- Evidence summaries redacted before LLM prompt
- Draft answers redacted before Junior verification
- Final responses redacted for display

### New Types

```rust
/// Types of secrets that can be redacted
pub enum SecretType {
    Password, ApiKey, BearerToken, AuthHeader, PrivateKey, PemBlock,
    SshKey, Cookie, AwsCredential, AzureCredential, GcpCredential,
    GitCredential, NetrcEntry, EnvSecret, JwtToken, DatabaseUrl,
    ConnectionString, OAuthToken, WebhookSecret, EncryptionKey,
    Certificate, GenericSecret,
}

/// Result of a redaction operation
pub struct RedactionResult {
    pub text: String,
    pub redaction_count: usize,
    pub secret_types_found: Vec<SecretType>,
    pub was_redacted: bool,
}

/// Result of checking for leaks
pub struct LeakCheckResult {
    pub has_leaks: bool,
    pub leaked_types: Vec<SecretType>,
    pub penalty: i32,
    pub suggestions: Vec<String>,
}
```

### New Functions

```rust
// Main redaction
pub fn redact(text: &str) -> String;
pub fn redact_secrets(text: &str) -> RedactionResult;
pub fn contains_secrets(text: &str) -> bool;

// Context-specific redaction
pub fn redact_transcript(text: &str) -> String;
pub fn redact_evidence(content: &str, path: Option<&str>) -> Result<String, String>;
pub fn redact_audit_details(details: &serde_json::Value) -> serde_json::Value;

// Environment variable handling
pub fn redact_env_value(name: &str, value: &str) -> Cow<str>;
pub fn redact_env_map(vars: &[(String, String)]) -> Vec<(String, String)>;

// Path restriction
pub fn is_path_restricted(path: &str) -> bool;
pub fn get_restriction_message(path: &str) -> String;

// Leak detection
pub fn check_for_leaks(text: &str) -> LeakCheckResult;
pub fn calculate_leak_penalty(types: &[SecretType]) -> i32;
```

### Tests Added

- 22 unit tests for redaction patterns
- Path restriction tests with wildcard matching
- Leak detection and penalty calculation tests
- Environment variable redaction tests
- Audit details redaction tests

### Files Changed

- **NEW:** `crates/anna_common/src/redaction.rs` - Centralized redaction module
- `crates/anna_common/src/lib.rs` - Module export and version bump
- `crates/annactl/src/pipeline.rs` - Redaction integration in dialogue/evidence
- `Cargo.toml` - Version 0.0.18
- `CLAUDE.md` - Version 0.0.18
- `TODO.md` - Mark 0.0.18 completed

---

## v0.0.17 - Multi-User Correctness

**Release Date:** 2025-12-03

### Summary

Stop pretending there is only one user on the machine. Anna now correctly identifies and operates on the target user for user-scoped tasks while keeping root-only operations in annad. This prevents root-owned files appearing in user home directories and ensures correct ownership for backups.

### Key Features

**Target User Selection (Strict Precedence):**
1. REPL session chosen user (if set)
2. `SUDO_USER` environment variable (for sudo invocations)
3. Non-root invoking user (getuid)
4. Primary interactive user (most recent login, or clarification prompt if ambiguous)

**Transcript Message:**
```
[anna] to [you]:
  I will treat Barbara as the target user for user-scoped changes,
  because you invoked annactl via sudo. [E-user-12345]
```

**Safe Home Directory Detection:**
- Canonical home from /etc/passwd lookup
- NEVER guess `/home/<username>`
- Evidence ID for home directory determination
- Functions: `get_user_home()`, `is_path_in_user_home()`, `expand_home_path()`

**User-Scoped File Operations:**
- `write_file_as_user()`: Creates files owned by target user
- `backup_file_as_user()`: Backups owned by target user
- `create_dir_as_user()`: Directories with correct ownership
- `fix_file_ownership()`: Repair incorrect ownership
- Uses `install` command with `-o` and `-g` flags for atomic ownership

**UserHomePolicy in capabilities.toml:**
```toml
[mutation_tools.file_edit.user_home]
enabled = true
max_file_size_bytes = 1048576

allowed_subpaths = [
    ".config/**",
    ".local/share/**",
    ".bashrc",
    ".zshrc",
    ".vimrc",
    ".gitconfig",
    # ... more dotfiles
]

blocked_subpaths = [
    ".ssh/**",
    ".gnupg/**",
    ".password-store/**",
    ".mozilla/**/key*.db",
    ".mozilla/**/logins.json",
    # ... browser credentials
]
```

**Clarification Prompt for Ambiguous Users:**
```
[anna] to [you]:
  Which user should I target?
    1) alice (Alice Smith)
    2) bob
  Select [1-2]:
```

### New Types

```rust
/// Information about a Unix user
pub struct UserInfo {
    pub username: String,
    pub uid: u32,
    pub gid: u32,
    pub home: PathBuf,
    pub shell: String,
    pub gecos: String,
}

/// How the target user was determined
pub enum UserSelectionSource {
    ReplSession,
    SudoUser,
    InvokingUser,
    PrimaryInteractive,
    UserChoice,
    FallbackRoot,
}

/// Result of target user selection
pub struct TargetUserSelection {
    pub user: UserInfo,
    pub source: UserSelectionSource,
    pub evidence_id: String,
    pub explanation: String,
    pub required_clarification: bool,
    pub other_candidates: Vec<UserInfo>,
}

/// Selection result: determined or needs clarification
pub enum SelectionResult {
    Determined(TargetUserSelection),
    NeedsClarification(AmbiguousUserSelection),
}

/// Policy for user home directory edits
pub struct UserHomePolicy {
    pub enabled: bool,
    pub allowed_subpaths: Vec<String>,
    pub blocked_subpaths: Vec<String>,
    pub max_file_size_bytes: u64,
}
```

### Example Transcript: "Enable syntax highlighting"

```
[anna] to [you]:
  I will treat barbara as the target user for user-scoped changes,
  because you invoked annactl via sudo. [E-user-34521]

[you] to [anna]:
  Enable syntax highlighting

[anna] to [translator]:
  Classify this request:
  "Enable syntax highlighting"

[translator] to [anna]:
  Intent: action_request
  Targets: .vimrc, vim, syntax
  Risk: medium
  Tools: user_config_edit
  Confidence: 82%

[anna] to [annad]:
  Please prepare a file edit for /home/barbara/.vimrc to enable syntax highlighting.
  Target user: barbara (uid=1000, gid=1000)

[annad] to [anna]:
  Preflight checks passed: [E-pre-99821]
  - Path /home/barbara/.vimrc is allowed by user_home policy
  - File is text, size 423 bytes
  - Backup will be created at /var/lib/anna/rollback/files/... (owned by barbara:barbara)

  Dry-run diff preview: [E-diff-99822]
  --- /home/barbara/.vimrc (current)
  +++ /home/barbara/.vimrc (proposed)
  @@ lines 1-5 @@
  + syntax on
  + set hlsearch
    " existing content...
  Changes: +2 added, -0 removed

[anna] to [you]:
  I can enable syntax highlighting by adding to ~/.vimrc:
    syntax on
    set hlsearch

  This is a medium-risk change to your shell configuration.
  Type "I CONFIRM (medium risk)" to proceed:

[you] to [anna]:
  I CONFIRM (medium risk)

[anna] to [annad]:
  Execute the file edit for barbara.

[annad] to [anna]:
  Edit applied successfully. [E-mut-99823]
  - File written as barbara:barbara (uid=1000, gid=1000)
  - Backup at /var/lib/anna/rollback/files/1701612345_mut_123/vimrc
  - Post-check: syntax on present, file readable

[anna] to [you]:
  Done! Syntax highlighting is now enabled in vim.
  The change was made to /home/barbara/.vimrc
  Backup available at: /var/lib/anna/rollback/files/1701612345_mut_123/vimrc

  Reliability: 94% [E-user-34521, E-pre-99821, E-diff-99822, E-mut-99823]
```

### Testing

- 15 unit tests for target user system (selection, precedence, clarification)
- 10 unit tests for user home policy (allowed/blocked paths, glob matching)
- Integration with existing mutation safety tests

### Files Changed

- `crates/anna_common/src/target_user.rs` (NEW - ~900 lines)
- `crates/anna_common/src/policy.rs` (UserHomePolicy added)
- `crates/anna_common/src/lib.rs` (exports)
- `crates/annactl/src/pipeline.rs` (target user selection integration)
- `Cargo.toml` (version bump, libc dependency)

---

## v0.0.16 - Better Mutation Safety

**Release Date:** 2025-12-03

### Summary

Senior-engineer-level safety for all mutations: preflight checks verify preconditions, dry-run diffs preview changes, post-checks verify expected state, and automatic rollback restores system on failure. Mutation state machine tracks lifecycle from planned through verified_ok or rolled_back.

### Key Features

**Mutation State Machine:**
- `MutationState` enum: `Planned` -> `PreflightOk` -> `Confirmed` -> `Applied` -> `VerifiedOk` | `RolledBack` | `Failed`
- Complete lifecycle tracking for audit trail
- State transitions logged with evidence IDs

**Preflight Checks (`PreflightResult`):**
- File edits: path allowed, file exists/creatable, is text, size under limit, permissions OK, hash recorded, backup available
- Systemd ops: unit exists, current state captured, operation allowed by policy
- Package ops: Arch Linux check, packages exist and not blocked, disk space check
- All checks generate evidence IDs for traceability

**Dry-Run Diff Preview (`DiffPreview`):**
- Line-based diff with context, additions, removals, modifications
- `DiffLine` enum: `Context`, `Added`, `Removed`, `Modified`
- Truncated output (max 20 lines) with overflow indicator
- Shows backup path and rollback command
- Human-readable format: `+1 added, -0 removed, ~2 modified`

**Post-Check Verification (`PostCheckResult`):**
- File edits: verify file exists, readable, contains expected content, hash changed
- Systemd ops: verify active/enabled state matches expectation, no immediate failure (500ms check)
- Package ops: verify package installed/removed
- Evidence IDs for post-state documentation

**Automatic Rollback:**
- `RollbackResult` with success, message, evidence_id, restored_state
- File edits: restore from backup
- Systemd ops: restore prior active/enabled state
- Logged and cited in audit trail
- Reliability score downgrade on rollback

**SafeMutationExecutor:**
- `preflight_file_edit()`, `preflight_systemd()`, `preflight_package()`
- `dry_run_file_edit()` for diff preview without modification
- `postcheck_file_edit()`, `postcheck_systemd()`, `postcheck_package()`
- `rollback_file_edit()`, `rollback_systemd()`
- Evidence ID generation (PRE#####, POST#####, RB#####)

**Junior Enforcement (v0.0.16):**
- New rules 14-19 for mutation safety
- Penalties: -25 for missing preflight, -20 for missing diff, -25 for missing post-check, -30 for missing rollback

### Tests Added

- 21 unit tests in `mutation_safety.rs`:
  - State display, preflight checks, diff generation
  - Edit operations, rollback results
  - Helper functions (arch detection, disk space)

### Files Changed

- `crates/anna_common/src/mutation_safety.rs`: New module (~1300 lines)
- `crates/anna_common/src/lib.rs`: Added module export
- `crates/annactl/src/pipeline.rs`: Updated Junior prompt with mutation safety rules

### Example Transcript: Config Change with Diff Preview

```
[you] to [anna]:
  Add log_level=debug to /etc/myapp/config.toml

[anna] to [translator]:
  Classify this request...

[translator] to [anna]:
  Intent: action_request
  Targets: /etc/myapp/config.toml
  Risk: medium

[anna] to [annad]:
  Run preflight checks for file edit

[annad] to [anna]:
  Preflight checks for edit_file_lines:
    [OK] path_allowed: Path /etc/myapp/config.toml is allowed [POL00012]
    [OK] file_exists: File exists
    [OK] is_text_file: File is text (no binary content)
    [OK] file_size: 256 bytes (limit: 1048576 bytes)
    [OK] permissions: File is readable and writable
    [OK] hash_recorded: Current hash: a1b2c3d4e5f6...
    [OK] backup_available: Backup directory writable
    [OK] operations_valid: 1 edit operation(s) validated

[anna] to [you]:
  Preflight passed. Here's what will change:

  Changes to /etc/myapp/config.toml:
    +1 added, -0 removed, ~0 modified

  +   3 | log_level=debug

  Backup: /var/lib/anna/rollback/config.toml_REQ...
  Rollback: cp /var/lib/anna/rollback/config.toml_REQ... /etc/myapp/config.toml

  To proceed, type exactly: I CONFIRM (medium risk)
```

### Example Transcript: Post-Check Failure with Rollback

```
[anna] to [annad]:
  Execute file edit (confirmed)

[annad] to [anna]:
  File edited. Running post-checks...

  Post-checks for edit_file_lines:
    [OK] file_exists: exists
    [OK] file_readable: readable
    [FAIL] content_check: expected 'log_level=debug', got 'not found'

  Post-check FAILED. Initiating automatic rollback...

[annad] to [anna]:
  Rollback completed [RB00001]:
  Restored /etc/myapp/config.toml from backup
  Reason: post-check failed - expected content not found

[anna] to [you]:
  The change was applied but post-verification failed.
  I've automatically rolled back to the previous state.

  Reliability: 45% (downgraded due to rollback)
```

---

## v0.0.15 - Governance UX Polish

**Release Date:** 2025-12-03

### Summary

Debug level configuration for controlling output verbosity. Unified formatting module for consistent terminal output. Enhanced `annactl status` as single source of truth with comprehensive sections. No regressions - all 500+ tests pass.

### Key Features

**Debug Levels (`config.toml`):**
- Configure in `/etc/anna/config.toml` with `ui.debug_level = 0 | 1 | 2`
- Level 0 (minimal): Only [you]->[anna] and final [anna]->[you], plus confirmations
- Level 1 (normal/default): Dialogues condensed, tool calls summarized, evidence IDs included
- Level 2 (full): Full dialogues between all players, tool execution summaries, Junior critique in full

**UI Configuration (`UiConfig`):**
- `debug_level`: Output verbosity (0=minimal, 1=normal, 2=full)
- `colors_enabled`: Terminal color output (default: true)
- `max_width`: Text wrapping width (0=auto-detect terminal width)
- Helper methods: `is_minimal()`, `is_normal_debug()`, `is_full_debug()`, `effective_width()`

**Unified Formatting (`display_format.rs`):**
- `colors` module: `section_header()`, `label()`, `success()`, `warning()`, `error()`, `evidence_id()`, `reliability()`
- `SectionFormatter`: Consistent status section headers and key-value formatting
- `DialogueFormatter`: Debug level filtering for pipeline output
- Format helpers: `format_bytes()`, `format_timestamp()`, `format_duration_ms()`, `format_percent()`, `format_eta()`
- Text helpers: `wrap_text()`, `indent()`, `format_list()`, `format_summary()`

**Enhanced Status Display (`annactl status`):**
- [VERSION]: Current version and build info
- [INSTALLER REVIEW]: Installation health and component status
- [UPDATES]: Update channel and status
- [MODELS]: LLM model status (Translator, Junior)
- [POLICY]: Policy status, schema version, blocked counts
- [HELPERS]: Installed helper packages
- [ALERTS]: Active alerts by severity
- [LEARNING]: Session memory and recipe counts
- [RECENT ACTIONS]: Audit log summary with last 3 actions
- [STORAGE]: Disk usage for Anna directories

**Pipeline Updates:**
- `dialogue()` function respects debug level configuration
- `dialogue_always()` for confirmations that always show
- Condensed message display at debug level 0

### Tests Added

- 19 tests in `display_format.rs` for formatting utilities
- 5 tests in `config.rs` for UiConfig
- All existing tests pass (500+ total)

### Files Changed

- `crates/anna_common/src/config.rs`: Added UiConfig struct
- `crates/anna_common/src/display_format.rs`: Enhanced formatting module
- `crates/annactl/src/commands/status.rs`: Added new sections
- `crates/annactl/src/pipeline.rs`: Debug level filtering

---

## v0.0.14 - Policy Engine + Security Posture

**Release Date:** 2025-12-03

### Summary

Policy-driven allowlists with no hardcoded deny rules. All major allow/deny decisions flow from editable TOML files in `/etc/anna/policy/`. Structured audit logging with secret redaction. Junior enforcement for policy compliance. Installer review validates policy sanity.

### Key Features

**Policy Engine (`policy.rs` - ~1400 lines):**
- Four policy files in `/etc/anna/policy/`:
  - `capabilities.toml`: Read-only and mutation tool settings
  - `risk.toml`: Risk thresholds, confirmations, reliability requirements
  - `blocked.toml`: Blocked packages, services, paths, commands
  - `helpers.toml`: Helper dependency management and policy
- Hot-reload support via `reload_policy()`
- Global policy cache with RwLock for thread safety
- Policy evidence IDs (POL##### format)

**Policy Checks:**
- `is_path_allowed()`: Check if path can be edited
- `is_package_allowed()`: Check if package can be installed/removed
- `is_service_allowed()`: Check if service can be modified
- `is_systemd_operation_allowed()`: Check systemd operations
- `PolicyCheckResult` with allowed, reason, evidence_id, policy_rule fields

**Blocked Categories (Default):**
- Kernel packages: `linux`, `linux-*`, `kernel*`
- Bootloader packages: `grub`, `systemd-boot`, `refind`, `syslinux`
- Init packages: `systemd`, `openrc`, `runit`
- Critical services: `systemd-*`, `dbus.service`, `NetworkManager.service`
- Protected paths: `/boot/*`, `/etc/shadow`, `/etc/passwd`, `/etc/sudoers`

**Audit Logging (`audit_log.rs` - ~400 lines):**
- Structured JSONL audit trail at `/var/lib/anna/audit/audit.jsonl`
- Entry types: ReadOnlyTool, MutationTool, PolicyCheck, Confirmation, ActionBlocked, Rollback, SecurityEvent
- Secret sanitization: passwords, API keys, tokens, Bearer headers
- Environment variable redaction for sensitive keys
- Log rotation at 10MB with archive directory
- Evidence ID linkage in all entries

**Mutation Tools Integration:**
- `validate_mutation_path()` uses policy for path validation
- `validate_package_policy()` checks blocked packages
- `validate_service_policy()` checks critical services
- `PolicyBlocked` error variant with evidence_id and policy_rule
- Symlink traversal protection (follows symlinks for policy checks)

**Junior Enforcement (v0.0.14):**
- Policy citation requirement for risky operations
- New rules 9-13 in Junior system prompt:
  - Risky operations MUST cite policy evidence [POL#####]
  - Refusals MUST explain which policy rule applied
  - Policy bypass suggestions = DANGEROUS = max penalty
- Penalties: -20 for risky operation without policy citation, -50 for policy bypass

**Installer Review (`installer_review.rs`):**
- `check_policy_sanity()` validates policy files
- Auto-repair creates default policy files if missing
- Policy file parsing and validation
- Evidence IDs for repair tracking

**Configuration Defaults:**
```toml
# /etc/anna/policy/capabilities.toml
[read_only_tools]
enabled = true
max_evidence_bytes = 1048576

[mutation_tools]
enabled = true

[mutation_tools.file_edit]
enabled = true
allowed_paths = ["/etc/", "/home/", "/root/", "/var/lib/anna/", "/tmp/"]
blocked_paths = ["/etc/shadow", "/etc/passwd", "/etc/sudoers"]
max_file_size_bytes = 1048576
text_only = true

[mutation_tools.systemd]
enabled = true
allowed_operations = ["status", "restart", "reload", "enable", "disable", "start", "stop"]
blocked_units = []
protected_units = ["sshd.service", "networkd.service", "systemd-resolved.service"]

[mutation_tools.packages]
enabled = true
max_packages_per_operation = 10
blocked_categories = ["kernel", "bootloader", "init"]
```

### Files Changed

- `crates/anna_common/src/policy.rs` (NEW - ~1400 lines)
- `crates/anna_common/src/audit_log.rs` (NEW - ~400 lines)
- `crates/anna_common/src/mutation_tools.rs` - Policy integration, PolicyBlocked error
- `crates/anna_common/src/installer_review.rs` - Policy sanity checks
- `crates/anna_common/src/lib.rs` - Module declarations and exports
- `crates/annactl/src/pipeline.rs` - Junior policy enforcement rules

### Tests

24 new unit tests covering:
- Policy evidence ID generation
- Pattern matching for packages/services
- Path policy checks (allowed, blocked, boot)
- Package policy checks (allowed, blocked categories, patterns)
- Service policy checks (critical services)
- Systemd operation validation
- Policy validation and defaults
- Confirmation phrase parsing
- Audit entry creation and serialization
- Secret sanitization (passwords, API keys, Bearer tokens)
- Environment variable redaction

### Bug Fixes

- Fixed `parse_tool_plan()` to handle commas inside parentheses correctly
- Fixed `execute_tool_plan()` double-counting evidence IDs
- Updated version test to use pattern matching instead of hardcoded version

---

## v0.0.13 - Conversation Memory + Recipe Evolution

**Release Date:** 2025-12-03

### Summary

Local-first conversation memory and recipe evolution system. Anna remembers past sessions, creates recipes for successful patterns, and allows user introspection via natural language. Privacy-first with summaries by default (no raw transcripts unless configured).

### Key Features

**Session Memory (`/var/lib/anna/memory/`):**
- Compact session records for every REPL/one-shot request
- Fields: request_id, request_text, translator_plan_summary, tools_used, evidence_ids, final_answer_summary, reliability_score, recipe_actions, timestamp
- Privacy default: summaries only (`store_raw` config option for full transcripts)
- Keyword-based search indexing
- Append-only JSONL storage with atomic writes
- Archive/tombstone pattern for "forget" operations
- Evidence IDs: MEM##### format

**Recipe System (`/var/lib/anna/recipes/`):**
- Named intent patterns with conditions
- Tool plan templates (read-only and/or mutation)
- Safety classification and required confirmations
- Precondition validation checks
- Rollback templates for mutations
- Provenance tracking (creator, timestamps)
- Confidence scoring and success/failure counters
- Evidence IDs: RCP##### format

**Recipe Creation Rules:**
- Created when: successful request AND Junior reliability >= 80% AND tools used repeatably
- Below 80% reliability creates "experimental" draft
- Configurable via `memory.min_reliability_for_recipe`

**Recipe Matching:**
- Keyword-based scoring (BM25-style for now)
- Intent type and target matching
- Negative keyword exclusion
- Top matching recipes provided to Translator

**User Introspection (Natural Language):**
- "What have you learned recently?"
- "List recipes" / "Show all recipes"
- "Show recipe for X" / "How do you handle X?"
- "Forget what you learned about X" (requires confirmation)
- "Delete recipe X" (requires confirmation)
- "Show my recent questions"
- "Search memory for X"

**Forget/Delete Operations:**
- Medium risk classification
- Requires explicit confirmation: "I CONFIRM (forget)"
- Reversible via archive (not hard delete)
- All operations logged

**Status Display (`annactl status`):**
- New [LEARNING] section
- Shows: recipe count, last learned time, sessions count
- Top 3 most used recipes with confidence scores
- Memory settings display (store_raw, max_sessions, min_reliability)

**Junior Enforcement (v0.0.13):**
- New learning claim detection in Junior system prompt
- Claims about "learned", "remembered", "knows", "has recipes for" must cite MEM/RCP IDs
- Automatic penalty (-25 per uncited learning claim) in fallback scoring
- Detection of fabricated learning claims in final response

**Configuration (`/etc/anna/config.toml`):**
```toml
[memory]
enabled = true              # Enable memory/learning
store_raw = false          # Store raw transcripts (privacy)
max_sessions = 10000       # Max sessions in index
min_reliability_for_recipe = 80  # Minimum % to create recipe
max_recipes = 500          # Max recipes to store
```

### Files Changed

- `crates/anna_common/src/memory.rs` (NEW - ~650 lines)
- `crates/anna_common/src/recipes.rs` (NEW - ~850 lines)
- `crates/anna_common/src/introspection.rs` (NEW - ~630 lines)
- `crates/anna_common/src/config.rs` - Added MemoryConfig
- `crates/anna_common/src/lib.rs` - Module declarations and exports
- `crates/annactl/src/commands/status.rs` - [LEARNING] section
- `crates/annactl/src/pipeline.rs` - Junior learning claim enforcement

### Tests

28 new unit tests covering:
- Session record creation and serialization
- Memory index management
- Keyword extraction
- Recipe creation and matching
- Recipe score calculation
- Introspection intent detection
- Evidence ID generation
- Learning claim detection

---

## v0.0.12 - Proactive Anomaly Detection

**Release Date:** 2025-12-03

### Summary

Proactive anomaly detection engine with alert queue, what_changed correlation tool, slowness_hypotheses analysis, and alert surfacing in REPL and one-shot modes. Complete with evidence IDs for traceability.

### Key Features

**Anomaly Detection Engine (`anomaly_engine.rs`):**
- Periodic anomaly detection (every 5 minutes when integrated with daemon)
- Baseline window (14 days) vs recent window (2 days) comparison
- Configurable thresholds for all metrics
- Evidence ID generation (ANO##### format)

**Supported Anomaly Signals:**
- `BootTimeRegression` - Boot time increased significantly
- `CpuLoadIncrease` - CPU load trend increasing
- `MemoryPressure` - High memory usage or swap activity
- `DiskSpaceLow` - Disk space below threshold
- `SystemCrash` - System crash detected
- `ServiceCrash` - Individual service crash
- `ServiceFailed` - Service in failed state
- `JournalWarningsIncrease` - Warning rate increase
- `JournalErrorsIncrease` - Error rate increase
- `DiskIoLatency` - Disk I/O latency increasing

**Alert Queue (`/var/lib/anna/internal/alerts.json`):**
- Deduplication by signal type
- Severity levels: Info, Warning, Critical
- Severity upgrades on repeated occurrences
- Acknowledgment support
- Persistence across restarts

**New Read-Only Tools:**
- `active_alerts` - Returns current alerts with evidence IDs
- `what_changed(days)` - Packages installed/removed, services enabled/disabled, config changes
- `slowness_hypotheses(days)` - Ranked hypotheses combining changes, anomalies, and resource usage

**Alert Surfacing:**
- REPL welcome: Shows active alerts on startup
- One-shot: Footer with alert summary
- `annactl status [ALERTS]` section: Detailed alert display with evidence IDs

**Evidence Integration:**
- All anomalies have unique evidence IDs
- Hypotheses cite supporting evidence
- Junior enforces citation requirements

### Files Changed

- `crates/anna_common/src/anomaly_engine.rs` (NEW - 1600+ lines)
- `crates/anna_common/src/tools.rs` - Added 3 new tools
- `crates/anna_common/src/tool_executor.rs` - Tool implementations
- `crates/anna_common/src/lib.rs` - Module exports
- `crates/annactl/src/main.rs` - Alert surfacing
- `crates/annactl/src/commands/status.rs` - Enhanced [ALERTS] section
- `crates/annactl/src/commands/mod.rs` - Version update

### Tests

16 new unit tests covering:
- Severity ordering and comparison
- Signal deduplication keys
- Alert queue operations (add, acknowledge, dedup)
- What_changed result formatting
- Slowness hypothesis builders
- Anomaly engine defaults
- Time window calculations

---

## v0.0.11 - Safe Auto-Update System

**Release Date:** 2025-12-03

### Summary

Complete safe auto-update system with update channels (stable/canary), integrity verification, atomic installation, zero-downtime restart, and automatic rollback on failure. Full state visibility in status display.

### Key Features

**Update Channels:**
- `stable` (default): Only stable tagged releases (no -alpha, -beta, -rc, -canary suffixes)
- `canary`: Accept all releases including pre-releases

**Configuration (`/etc/anna/config.toml`):**
```toml
[update]
mode = "auto"           # auto or manual
channel = "stable"      # stable or canary
interval_seconds = 600  # check every 10 minutes
min_disk_space_bytes = 104857600  # 100 MB minimum
```

**Safe Update Workflow:**
1. Check GitHub releases API for new version matching channel
2. Download artifacts to staging directory (`/var/lib/anna/internal/update_stage/<version>/`)
3. Verify integrity (SHA256 checksums)
4. Backup current binaries (`/var/lib/anna/internal/update_backup/`)
5. Atomic installation via rename
6. Signal systemd restart
7. Post-restart validation
8. Cleanup staging and old backups

**Rollback Support:**
- Previous binaries kept in backup directory
- Automatic rollback on restart failure
- Manual rollback possible via backup files
- Rollback state shown in `annactl status`

**Guardrails:**
- Never updates during active mutation operations
- Checks disk space before download (100 MB minimum)
- Verifies installer review health before update
- Rate-limits on consecutive failures with exponential backoff
- No partial installations (atomic or nothing)

**Status Display ([UPDATES] section):**
```
[UPDATES]
  Mode:       auto (stable)
  Interval:   10m
  Last check: 2025-12-03 14:30:00
  Result:     up to date
  Next check: in 8m
```

During update:
```
[UPDATES]
  Mode:       auto (stable)
  Interval:   10m
  Progress:   downloading (45%, ETA: 30s)
  Updating:   0.0.10 -> 0.0.11
```

After update:
```
[UPDATES]
  Mode:       auto (stable)
  Interval:   10m
  Last update: 0.0.10 -> 0.0.11 (5m ago)
```

### Technical Details

**UpdatePhase enum:**
- `Idle`, `Checking`, `Downloading`, `Verifying`, `Staging`, `Installing`, `Restarting`, `Validating`, `Completed`, `Failed`, `RollingBack`

**UpdateState fields (v0.0.11):**
- `channel`: Update channel (stable/canary)
- `update_phase`: Current phase name
- `update_progress_percent`: Progress 0-100
- `update_eta_seconds`: Estimated time remaining
- `updating_to_version`: Target version
- `last_update_at`: Timestamp of last successful update
- `previous_version`: For rollback display

**IntegrityStatus:**
- `StrongVerified`: Verified against release checksum
- `WeakComputed`: Self-computed checksum (no release checksum available)
- `Failed`: Checksum mismatch
- `NotVerified`: Skipped verification

### Files Changed

- `crates/anna_common/src/update_system.rs` (new - 600+ lines)
- `crates/anna_common/src/config.rs` (enhanced UpdateConfig, UpdateState, UpdateResult)
- `crates/anna_common/src/lib.rs` (exports)
- `crates/annactl/src/commands/status.rs` (update progress display)
- `crates/annactl/src/main.rs` (version update)

### Tests

```rust
// Version comparison
assert!(is_newer_version("0.0.11", "0.0.10"));
assert!(!is_newer_version("0.0.9", "0.0.10"));

// Channel matching
assert!(stable.matches_tag("v0.0.10"));
assert!(!stable.matches_tag("v0.0.11-alpha"));
assert!(canary.matches_tag("v0.0.11-alpha"));

// Update phase display
assert_eq!(UpdatePhase::Downloading { progress_percent: 50, eta_seconds: Some(30) }
    .format_display(), "downloading... 50% (ETA: 30s)");
```

---

## v0.0.10 - Reset + Uninstall + Installer Review

**Release Date:** 2025-12-03

### Summary

Factory reset and clean uninstall commands with provenance-aware helper removal. Installer review system verifies installation health and auto-repairs common issues. Confirmation phrases required for destructive operations.

### Key Features

**Reset Command (`annactl reset`):**
- Factory reset returns Anna to fresh install state
- Confirmation phrase: "I CONFIRM (reset)"
- --dry-run flag shows what would be deleted
- --force flag skips confirmation prompt
- Clears all data directories, config, and helper tracking
- Recreates directory structure with correct permissions
- Runs installer review at end and reports health status

**Uninstall Command (`annactl uninstall`):**
- Complete Anna removal from system
- Confirmation phrase: "I CONFIRM (uninstall)"
- Provenance-aware helper removal (only anna-installed)
- Asks user about helper removal unless --keep-helpers
- Removes: systemd unit, binaries, data, config
- --dry-run and --force flags supported

**Install State (`install_state.rs`):**
- Tracks installation artifacts for accurate uninstall
- BinaryInfo: path, checksum, version, last_verified
- UnitInfo: path, exec_start, enabled state
- DirectoryInfo: path, expected permissions, ownership
- Review history with last 10 reviews
- Stored at `/var/lib/anna/install_state.json`

**Installer Review (`installer_review.rs`):**
- Verifies installation correctness
- Checks: binary presence, systemd correctness, directories/permissions, config, update scheduler, Ollama health, helper inventory
- Auto-repair for common issues (without user confirmation):
  - Recreate missing internal directories
  - Fix Anna-owned permissions
  - Re-enable annad service if misconfigured
- Evidence IDs for repair tracking (format: IRxxxx)
- ReviewResult: Healthy, Repaired, NeedsAttention, Failed

**Status Display ([INSTALL REVIEW] section):**
```
[INSTALL REVIEW]
  Status:     healthy
  Last run:   5 minute(s) ago
  Duration:   42 ms
```

### Technical Details

- Install state schema version: 1
- Auto-repair rules: read-only or low-risk internal fixes only
- Review checks ordered: critical (binaries) to informational (helpers)
- Confirmation gates use exact phrase matching

### CLI Changes

```bash
annactl reset              # Factory reset (requires root)
annactl reset --dry-run    # Show what would be deleted
annactl reset --force      # Skip confirmation

annactl uninstall          # Complete removal (requires root)
annactl uninstall --dry-run
annactl uninstall --force
annactl uninstall --keep-helpers
```

### Files Changed

- `crates/anna_common/src/install_state.rs` (new)
- `crates/anna_common/src/installer_review.rs` (new)
- `crates/annactl/src/commands/reset.rs` (updated)
- `crates/annactl/src/commands/uninstall.rs` (new)
- `crates/annactl/src/commands/status.rs` (updated)
- `crates/annactl/src/main.rs` (updated)

---

## v0.0.9 - Package Management + Helper Tracking

**Release Date:** 2025-12-03

### Summary

Package management (controlled) with helper tracking and provenance. Tracks all packages Anna relies on, distinguishing between anna-installed and user-installed packages. Only anna-installed packages can be removed via Anna.

### Key Features

**Helper Tracking System (`helpers.rs`):**
- Tracks ALL helpers Anna relies on for telemetry/diagnostics/execution
- Two dimensions: present/missing + installed_by (anna/user/unknown)
- HelperDefinition, HelperState, HelpersManifest types
- InstalledBy enum with Display implementation
- Persistent storage in `/var/lib/anna/helpers.json`
- First-seen and anna-install timestamps
- Provenance tracked per-machine, not globally

**Helper Definitions (9 helpers):**
- `lm_sensors`: Temperature and fan monitoring
- `smartmontools`: SATA/SAS disk health (SMART)
- `nvme-cli`: NVMe SSD health monitoring
- `ethtool`: Network interface diagnostics
- `iw`: WiFi signal and stats
- `usbutils`: USB device enumeration
- `pciutils`: PCI device enumeration
- `hdparm`: SATA disk parameters
- `ollama`: Local LLM inference

**Package Management Mutation Tools:**
- `package_install`: Install package via pacman (tracks as anna-installed)
- `package_remove`: Remove package (only anna-installed packages)
- 8 mutation tools total (6 from v0.0.8 + 2 new)

**Status Display ([HELPERS] section):**
```
[HELPERS]
  Summary:    7 present, 2 missing (1 by Anna)

  ethtool (present, installed by user)
  lm_sensors (present, installed by Anna)
  ollama (present, installed by Anna)
  smartmontools (missing, unknown origin)
```

**StatusSnapshot Extensions:**
- `helpers_total`: Total helpers tracked
- `helpers_present`: Helpers currently installed
- `helpers_missing`: Helpers not installed
- `helpers_anna_installed`: Helpers installed by Anna
- `helpers`: Vec<HelperStatusEntry> for detailed display

**Transaction Logging:**
- MutationType::PackageInstall, MutationType::PackageRemove
- MutationDetails::Package with package, version, reason, operation
- log_package_operation() in RollbackManager

**Provenance Rules:**
- If helper present before Anna tracked it → installed_by=user
- If Anna installs helper → installed_by=anna
- Only helpers with installed_by=anna are removal_eligible
- package_remove rejects non-anna packages with clear error

### New Error Types

- `PackageAlreadyInstalled(String)`
- `PackageNotInstalled(String)`
- `PackageNotRemovable { package, reason }`
- `PackageInstallFailed { package, reason }`
- `PackageRemoveFailed { package, reason }`

### New Files

- `crates/anna_common/src/helpers.rs` - Helper tracking system

### Modified Files

- `crates/anna_common/src/mutation_tools.rs` - Added package_install, package_remove tools
- `crates/anna_common/src/mutation_executor.rs` - Added package execution functions
- `crates/anna_common/src/rollback.rs` - Added PackageInstall/Remove types and logging
- `crates/anna_common/src/daemon_state.rs` - Added helpers fields to StatusSnapshot
- `crates/anna_common/src/lib.rs` - Added helpers module and exports
- `crates/annactl/src/commands/status.rs` - Added [HELPERS] section

### New Tests

**helpers.rs:**
- test_helper_state_new_missing
- test_helper_state_new_user_installed
- test_helper_state_mark_anna_installed
- test_helper_state_already_present_not_anna
- test_helper_definitions
- test_manifest_removal_eligible
- test_installed_by_display
- test_format_status

### Breaking Changes

- None - backward compatible with v0.0.8
- New fields in StatusSnapshot are optional with `#[serde(default)]`

---

## v0.0.8 - First Safe Mutations

**Release Date:** 2025-12-03

### Summary

First safe mutations (medium-risk only) with automatic rollback and confirmation gates. Introduces mutation tool catalog with allowlist enforcement, file backup system, and Junior reliability threshold for execution approval.

### Key Features

**Mutation Tool Catalog (`mutation_tools.rs`):**
- Allowlist-enforced mutation tools (6 tools total)
- `edit_file_lines`: Text file edits under /etc/** and $HOME/**
- `systemd_restart`: Service restart
- `systemd_reload`: Service configuration reload
- `systemd_enable_now`: Enable and start service
- `systemd_disable_now`: Disable and stop service
- `systemd_daemon_reload`: Reload systemd daemon
- MutationRisk enum (Medium, High)
- File size limit: MAX_EDIT_FILE_SIZE = 1 MiB
- Path validation: is_path_allowed(), validate_mutation_path()

**Rollback System (`rollback.rs`):**
- RollbackManager with timestamped backups
- Backup location: /var/lib/anna/rollback/files/
- Logs location: /var/lib/anna/rollback/logs/
- File hashing (SHA256) for integrity verification
- Diff summary generation for file edits
- Structured JSON logs per mutation request
- JSONL append log (mutations.jsonl) for audit trail
- Rollback instructions with exact restore commands

**Confirmation Gate:**
- Exact phrase requirement: "I CONFIRM (medium risk)"
- Validation via validate_confirmation()
- User confirmation displayed in dialogue transcript
- Action NOT executed if phrase doesn't match exactly

**Junior Verification Threshold:**
- Minimum 70% reliability required for execution
- MutationPlan.junior_approved flag
- MutationPlan.junior_reliability score
- is_approved_for_execution() check

**Mutation Executor (`mutation_executor.rs`):**
- execute_mutation(): Single mutation execution
- execute_mutation_plan(): Batch execution
- Request ID generation (UUID v4)
- Evidence ID propagation from tool results
- Structured MutationResult with human_summary and rollback_info

**Pipeline Updates (`pipeline.rs`):**
- ActionPlan extended with mutation_plan and is_medium_risk_executable
- generate_action_plan() creates MutationPlan for service operations
- handle_mutation_execution() for confirmation and execution flow
- ensure_service_suffix() helper for .service extension

### Not Allowed in v0.0.8

The following are explicitly NOT executable:
- Package installs/removals (pacman, yay)
- Filesystem operations (mount, umount, mkfs)
- Destructive operations (rm, rmdir)
- Permission changes (chmod, chown)
- User management (useradd, userdel, usermod)
- Firewall rules (iptables, ufw)
- Kernel parameters (sysctl)

### Mutation Execution Flow

```
[you] to [anna]: restart nginx
[anna] to [translator]: Classify this request...
[translator] to [anna]: Intent: action_request, Targets: nginx
                        Risk: medium
[anna] generates action plan with MutationPlan:
  What will change: Services to restart: nginx.service
  Why: Service restart requested
  Risk: medium
  Rollback: Service state can be restored by restarting again
[anna] to [junior]: Verify this mutation plan...
[junior] to [anna]: Reliability: 85%
[anna] to [you]: I can execute this action for you.

                 What will change:
                   Services to restart: nginx.service

                 Risk level: medium

                 To proceed, type exactly: I CONFIRM (medium risk)
[you] to [anna]: I CONFIRM (medium risk)
[anna] to [annad]: User confirmed. Please execute:
                   - systemd_restart (nginx.service)
[annad] executes: systemctl restart nginx.service
SUCCESS systemd_restart: Restarted nginx.service
  Rollback: sudo systemctl restart nginx.service
[annad] to [anna]: All operations completed successfully.
                   1 mutation(s) executed.
                   Logs saved to: /var/lib/anna/rollback/logs/
```

### Rollback Log Example

```json
{
  "request_id": "a1b2c3d4-...",
  "timestamp": 1701619200,
  "tool_name": "systemd_restart",
  "evidence_ids": ["E1", "E2"],
  "mutation_type": "systemd",
  "target": "nginx.service",
  "details": {
    "Systemd": {
      "operation": "restart",
      "service": "nginx.service",
      "previous_state": { "active": true, "enabled": true }
    }
  },
  "success": true,
  "error": null
}
```

### New Tests

**mutation_tools.rs:**
- test_path_allowed_etc
- test_path_allowed_home
- test_path_not_allowed
- test_confirmation_valid
- test_confirmation_missing
- test_confirmation_wrong
- test_mutation_catalog_has_expected_tools
- test_mutation_catalog_rejects_unknown
- test_mutation_plan_approval
- test_risk_display

**rollback.rs:**
- test_diff_summary_generation
- test_backup_file
- test_file_hash
- test_rollback_info_generation

### Breaking Changes

- None - backward compatible with v0.0.7
- New fields in ActionPlan are optional (mutation_plan, is_medium_risk_executable)

---

## v0.0.7 - Read-Only Tooling & Evidence Citations

**Release Date:** 2025-12-03

### Summary

Read-only tool catalog with allowlist enforcement, Evidence IDs for citations, and human-readable natural language transcripts. Junior now enforces no-guessing with uncited claim detection. Translator outputs tool plans for evidence gathering.

### Key Features

**Read-Only Tool Catalog (`tools.rs`):**
- Internal tool allowlist with security classification
- 10 read-only tools: status_snapshot, sw_snapshot_summary, hw_snapshot_summary, recent_installs, journal_warnings, boot_time_trend, top_resource_processes, package_info, service_status, disk_usage
- ToolDef with human-readable descriptions and latency hints
- ToolSecurity enum (ReadOnly, LowRisk, MediumRisk, HighRisk)
- parse_tool_plan() for parsing Translator output

**Tool Executor (`tool_executor.rs`):**
- execute_tool() for individual tool execution
- execute_tool_plan() for batch execution with EvidenceCollector
- Structured ToolResult with human_summary
- Unknown tool handling with graceful errors

**Evidence IDs and Citations:**
- EvidenceCollector assigns sequential IDs (E1, E2, ...)
- Evidence IDs in all tool results and dialogue
- Citations expected in final responses: [E1], [E2]
- Evidence legend in final response

**Junior No-Guessing Enforcement:**
- UNCITED_CLAIMS output field for speculation detection
- Strict scoring: uncited claims = -15 per claim
- "Unknown" preferred over guessing
- Label inference explicitly: "Based on [E2], likely..."

**Translator Tool Plans:**
- TOOLS output field in Translator response
- Tool plan parsing from comma-separated calls
- RATIONALE field for tool selection reasoning
- Deterministic fallback generates tool plans from evidence_needs

**Natural Language Transcripts:**
```
[anna] to [annad]: Please gather evidence using: hw_snapshot_summary
[annad] to [anna]: [E1] hw_snapshot_summary: CPU: AMD Ryzen 7 5800X (8 cores)
                                            Memory: 32GB total, 16GB available (found)
```

### Pipeline Flow (v0.0.7)

```
[you] to [anna]: what CPU do I have?
[anna] to [translator]: Classify this request...
[translator] to [anna]: Intent: system_query, Targets: cpu
                        Risk: read_only
                        Tools: hw_snapshot_summary
[anna] to [annad]: Please gather evidence using: hw_snapshot_summary
[annad] to [anna]: [E1] hw_snapshot_summary: CPU: AMD Ryzen 7 5800X (found)
[anna] to [junior]: Verify this draft response:
                    Based on gathered evidence:
                    [E1] CPU: AMD Ryzen 7 5800X
[junior] to [anna]: Reliability: 95%, Critique: All claims cite evidence
                    Uncited claims: none
[anna] to [you]: Based on gathered evidence:
                 [E1] CPU: AMD Ryzen 7 5800X
                 ---
                 Evidence sources:
                   [E1]: hw_snapshot_summary (OK)
Reliability: 95%
```

### New Tests (7 tests added)

- test_deterministic_generates_tool_plan
- test_deterministic_service_query_generates_service_tool
- test_parse_junior_response_with_uncited_claims
- test_parse_junior_response_no_uncited_claims
- test_tool_catalog_creation
- test_evidence_collector
- test_fallback_scoring_v2_with_tool_results

### Breaking Changes

- None - backward compatible with v0.0.6
- Legacy evidence retrieval still works when tool_plan is None

---

## v0.0.6 - Real Translator LLM

**Release Date:** 2025-12-03

### Summary

Real LLM-powered Translator for intent classification with clarification loop support. Evidence-first pipeline with real snapshot integration and 8KB excerpt cap. Action plan generation for mutation requests (no execution yet).

### Key Features

**Real Translator LLM (`pipeline.rs`):**
- Real LLM-backed intent classification replacing deterministic mock
- Structured output parsing: intent, targets, risk, evidence_needs, clarification
- System prompt with strict output format
- Fallback to deterministic classification when LLM unavailable

**Clarify-or-Proceed Loop:**
- Multiple-choice clarification prompts
- Default option selection
- Single-turn clarification (no infinite loops)
- CLARIFICATION field in Translator output: `question|option1|option2|option3|default:N`

**Evidence-First Pipeline:**
- Real snapshot integration from annad
- Evidence excerpting with 8KB hard cap
- `[EXCERPT truncated]` indication when data exceeds limit
- Evidence sources: hw_snapshot, sw_snapshot, status, journalctl

**Action Plan Generation:**
- Action plans for action_request intent
- Steps, affected files/services/packages
- Risk classification propagation
- Rollback outline
- Confirmation phrase (no execution - confirmation-gated)

**Translator System Prompt:**
```
OUTPUT FORMAT (follow exactly, one field per line):
INTENT: [question|system_query|action_request|unknown]
TARGETS: [comma-separated list or "none"]
RISK: [read_only|low|medium|high]
EVIDENCE_NEEDS: [hw_snapshot, sw_snapshot, status, journalctl, or "none"]
CLARIFICATION: [empty OR "question|opt1|opt2|opt3|default:N"]
```

### Pipeline Flow (v0.0.6)

```
[you] to [anna]: install nginx
[anna] to [translator]: Please classify this request...
[translator thinking via qwen2.5:0.5b...]
[translator] to [anna]: Intent: action_request, Targets: nginx, Risk: medium
                        Evidence: sw_snapshot
                        Clarification: none
[anna] to [annad]: Retrieve evidence for: nginx
[annad] to [anna]: snapshot:sw: [package data, 8KB max excerpt]
[anna] generates action plan:
  Steps: 1. Run pacman -S nginx  2. Enable nginx.service  3. Start nginx.service
  Affected: packages: nginx, services: nginx.service
  Rollback: pacman -Rns nginx
  Confirmation: Type "I understand and accept the risk" to proceed
[anna] to [junior]: Please verify this action plan...
[junior] to [anna]: Reliability: 75%, Critique: Plan looks correct...
                    MUTATION_WARNING: This will install a package.
[anna] to [you]: Action plan ready. Type confirmation phrase to execute.
Reliability: 75%
```

### Tests

- 15 pipeline unit tests (parsing, clarification, evidence, action plans)
- 20 CLI integration tests (all passing)
- Test coverage for deterministic fallback
- Evidence excerpting edge cases

### Breaking Changes

- None - backward compatible with v0.0.5

---

## v0.0.5 - Role-Based Model Selection + Benchmarking

**Release Date:** 2025-12-03

### Summary

Role-based LLM model selection with hardware-aware configuration and built-in benchmarking. Translator (fast) and Junior (reliable) now have distinct model pools selected based on system capabilities. Bootstrap process with progress tracking.

### Key Features

**Hardware Detection (`model_selection.rs`):**
- CPU detection (cores, model name)
- RAM detection (total, available)
- GPU/VRAM detection via nvidia-smi
- Hardware tier classification:
  - Low: <8GB RAM
  - Medium: 8-16GB RAM
  - High: >16GB RAM

**Role-Based Model Selection:**
- LlmRole enum: Translator, Junior
- Translator candidates (smallest/fastest first): qwen2.5:0.5b, qwen2.5:1.5b, phi3:mini, gemma2:2b, llama3.2:1b
- Junior candidates (most reliable first): llama3.2:3b-instruct, qwen2.5:3b-instruct, mistral:7b-instruct, gemma2:9b
- Priority-based selection respecting hardware tier

**Benchmark Suites:**
- Translator: 30 prompts testing intent classification (read-only vs mutation, targets, etc.)
- Junior: 15 cases testing verification quality (evidence handling, honesty scoring, etc.)
- Per-case latency and pattern matching evaluation

**Ollama Model Pull with Progress:**
- Streaming pull progress via `/api/pull`
- Real-time progress percentage
- Download speed (MB/s)
- ETA calculation
- Progress exposed in status snapshot

**Bootstrap State Machine:**
- `detecting_ollama`: Checking Ollama availability
- `installing_ollama`: Installing Ollama (future)
- `pulling_models`: Downloading required models
- `benchmarking`: Running model benchmarks (future)
- `ready`: Models ready for use
- `error`: Bootstrap failed

**Status Snapshot Updates (schema v3):**
- `llm_bootstrap_phase`: Current phase
- `llm_translator_model`: Selected translator
- `llm_junior_model`: Selected junior
- `llm_downloading_model`: Model being pulled
- `llm_download_percent`: Progress percentage
- `llm_download_speed`: Download speed
- `llm_download_eta_secs`: ETA in seconds
- `llm_error`: Error message if any
- `llm_hardware_tier`: Detected tier

**annactl Progress Display:**
- Shows bootstrap progress when models not ready
- Progress bar for model downloads
- ETA display
- Graceful fallback with reduced reliability score (-10 points)
- Explicit reason shown when LLM unavailable

### Configuration

New LLM settings in `/etc/anna/config.toml`:
```toml
[llm]
enabled = true
ollama_url = "http://127.0.0.1:11434"

[llm.translator]
model = ""  # Empty for auto-select
timeout_ms = 30000
enabled = true

[llm.junior]
model = ""  # Empty for auto-select
timeout_ms = 60000
enabled = true

# Custom candidate pools (optional)
translator_candidates = ["qwen2.5:0.5b", "qwen2.5:1.5b"]
junior_candidates = ["llama3.2:3b-instruct", "mistral:7b-instruct"]
```

### Tests

- 21 model_selection unit tests
- Hardware tier boundary tests
- Model priority selection tests
- Fallback behavior tests
- Bootstrap state tests

### Breaking Changes

- Config structure changed: `junior.*` deprecated, use `llm.junior.*`
- Status snapshot schema bumped to v3

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
