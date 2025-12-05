# Changelog

All notable changes to Anna will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.36] - 2025-12-05

### Added
- **SystemSnapshot (Preventive Anna)** (`snapshot.rs`):
  - `SystemSnapshot` struct captures minimal deterministic system state
  - Tracks: disk usage per mount, failed services, memory (total/used)
  - `capture_snapshot()` parses df, free, and systemctl --failed output
  - `diff_snapshots()` detects meaningful changes with anti-spam thresholds
  - `DeltaItem` enum: DiskWarning, DiskCritical, NewFailedService, MemoryHigh, etc.
  - Thresholds: DISK_WARN=85%, DISK_CRITICAL=95%, MEMORY_HIGH=85%
  - Persistence: `save_snapshot()`, `load_last_snapshot()`, `clear_snapshots()`
  - `is_fresh()` checks if snapshot is within `snapshot_max_age_secs` (default 300s)

- **PendingClarification** (`pending.rs`):
  - `PendingClarification` struct for REPL session continuity
  - Persists pending questions to `~/.anna/pending.json`
  - `ParseResult` enum: Selected, Custom, Cancelled, Invalid
  - `VerifyResult` enum for answer verification (vim vs vi fallback)
  - `format_prompt()` generates numbered option list
  - `parse_input()` handles number, name, or custom input
  - Stale detection: clarifications expire after 1 hour

- **PacketPolicy per Team** (`ticket_packet.rs`):
  - `PacketPolicy` struct: max_summary_lines, allowed_facts, required_probes, max_probes
  - `for_team()` returns team-specific policy (Desktop, Storage, Network, etc.)
  - `truncate_summary()` for deterministic truncation with "(n more lines omitted)"
  - `is_fact_allowed()` validates fact access per team
  - Desktop: max 10 lines, PreferredEditor fact allowed
  - Storage: disk_usage + block_devices required
  - Performance: max 5 probes, memory_info + cpu_info + top_cpu required

- **ProbeBudget** (`budget.rs`):
  - New `ProbeBudget` struct for controlling probe resource usage
  - Methods: `fast_path()`, `standard()`, `extended()` presets
  - `max_probes`, `max_output_bytes`, `per_probe_cap_bytes` limits
  - `would_exceed()` and `cap_output()` for budget enforcement
  - `ProbeBudgetCheck` enum for budget validation results

- **Clarification Cancel option** (`clarify.rs`):
  - `CLARIFY_CANCEL_KEY` and `CLARIFY_OTHER_KEY` constants
  - `is_cancel_selection()` and `is_other_selection()` helpers
  - Editor options now always include Cancel and Other options
  - Cancel allows user to skip clarification without answering

- **Enhanced latency tracking** (`state.rs`, `status.rs`):
  - Added `p50_ms()` and `p90_ms()` percentile methods to LatencyStats
  - Added `min_ms()` and `max_ms()` methods
  - Updated `LatencyStatus` struct with p50, p90 fields for all stages
  - Helper `percentile_ms()` method for flexible percentile calculation

- **TicketPacket** (`ticket_packet.rs`):
  - `TicketPacket` struct for domain-relevant evidence collection
  - `PacketBudget` tracks probe execution stats
  - `TicketPacketBuilder` with fluent API for packet construction
  - `recommended_probes_for_domain()` returns domain-specific probes
  - `evidence_kinds_for_domain()` returns required evidence kinds
  - Methods: `find_probe()`, `successful_probes()`, `probe_success_rate()`

### Changed
- Latency status now reports p50, p90, p95 percentiles (was only p95)
- Editor clarification always shows installed editors + Other + Cancel
- `annactl reset` now clears snapshots and pending clarifications
- Config: Added `snapshot_max_age_secs` (default 300s = 5 minutes)

## [0.0.35] - 2025-12-05

### Added
- **SystemTriage patterns extended** (v0.0.35):
  - "health", "status" now route to SystemTriage (fast path), not full report
  - Added `boot_time` probe (systemd-analyze) to triage probes
  - Patterns: "how is my computer", "any errors", "any problems", "health", "status"

- **Journalctl parser module** (`parsers/journalctl.rs`):
  - `JournalSummary` with `count_total` and `top: Vec<JournalTopItem>`
  - Deterministic grouping by unit name (case-insensitive)
  - Stable ordering: count desc, then key asc
  - `BootTimeInfo` with millisecond precision (`total_ms`, `kernel_ms`, `userspace_ms`)
  - `parse_boot_time()` extracts timing from systemd-analyze output

- **ParsedProbeData variants** (parsers/mod.rs):
  - `JournalErrors(JournalSummary)` for journalctl -p 3
  - `JournalWarnings(JournalSummary)` for journalctl -p 4
  - `BootTime(BootTimeInfo)` for systemd-analyze

- **Editor clarification** (clarify.rs):
  - `KNOWN_EDITORS` constant with vim, vi, nvim, nano, emacs, code, micro, hx
  - `generate_editor_options_sync()` probes `which <editor>` for installed editors
  - `verify_editor_installed()` checks if user's choice is available
  - `generate_editor_clarification()` returns question + detected options

- **RAG-lite keyword search** (recipe.rs):
  - `RecipeMatch` struct with score and matched keywords
  - `search_recipes_by_keywords()` returns top N matches deterministically
  - Scoring: keyword matches + route_class/domain bonuses + reliability/maturity
  - `find_config_edit_recipes()` for use before junior escalation

### Changed
- **SystemHealthSummary narrowed**: Only triggers on explicit "summary", "report", "overview"
- **Probe mappings** (translator.rs): Added journal_errors, journal_warnings, failed_units, boot_time
- **Triage answer format**: Shows boot time, evidence sources, top 3 error/warning keys

## [0.0.34] - 2025-12-05

### Added
- **FAST PATH routing (SystemTriage)**: Zero-timeout path for "how is my computer?" queries
  - New `QueryClass::SystemTriage` routes error-focused queries before SystemHealthSummary
  - Probes: `journal_errors`, `journal_warnings`, `failed_units` only (no disk/memory unless needed)
  - Matches: "any errors", "any problems", "is everything ok", "how is my computer"

- **Journalctl parser** (`parsers.rs`):
  - `parse_journalctl()`: Parses error/warning output with unit grouping
  - `parse_failed_units()`: Extracts failed systemd units
  - `JournalSummary` and `FailedUnit` structs for deterministic processing

- **Deterministic triage answer generator** (`triage_answer.rs`):
  - `generate_triage_answer()`: Produces deterministic answers from journal/systemctl evidence
  - Rules: "No critical issues" + warnings, or list errors/failed units
  - Always includes evidence summary for auditability

- **CLARIFY loop enhancements** (`clarify.rs`):
  - `ClarifyOption` with evidence strings (e.g., "installed: true")
  - `ClarifyAnswer` for structured user responses
  - Verification probes for all clarification types

- **Evidence kinds** (`trace.rs`):
  - New `EvidenceKind::Journal` and `EvidenceKind::FailedUnits`
  - `evidence_kinds_from_route("system_triage")` returns Journal + FailedUnits

- **REPL greeting UX** (`display.rs`):
  - Shows only relevant deltas on startup: failed units, journal errors, boot delta
  - No full report unless user asks `annactl report`

### Changed
- **RESCUE hardening (Phase D)**:
  - Global timeout responses no longer say "please rephrase"
  - Provides deterministic status answer with actionable suggestions
  - New `ExecutionTrace::global_timeout()` for tracing
  - All timeout responses set `needs_clarification: false`

## [0.0.33] - 2025-12-05

### Added
- **Knowledge Store (RAG-first)**: Local retrieval system for fast, deterministic answers
  - `knowledge/sources.rs`: KnowledgeDoc, KnowledgeSource enum (Recipe, SystemFact, PackageFact, ArchWiki, AUR, Journal, Usage)
  - `knowledge/index.rs`: BM25-lite keyword index for sub-50ms retrieval
  - `knowledge/retrieval.rs`: RetrievalQuery, RetrievalHit with source filtering
  - `knowledge/store.rs`: KnowledgeStore with JSONL persistence at ~/.anna/knowledge/
  - `knowledge/conversion.rs`: Recipe-to-KnowledgeDoc conversion for learning

- **System Collectors**: On-demand knowledge gathering from system state
  - `collectors.rs`: collect_boot_time() from systemd-analyze
  - `collectors.rs`: collect_packages() from pacman -Q or dpkg
  - `collectors.rs`: collect_journal_errors() from journalctl -p 3 -b
  - Full provenance tracking for auditability

- **RAG-first Query Classes**: Direct answers from knowledge store, skip LLM
  - `QueryClass::BootTimeStatus`: "boot time", "how long to boot"
  - `QueryClass::InstalledPackagesOverview`: "how many packages", "what's installed"
  - `QueryClass::AppAlternatives`: "alternative to vim", "instead of firefox"
  - `rag_answerer.rs`: try_rag_answer() routes queries through knowledge store

### Changed
- Knowledge answers use collectors on-demand if store is empty (collect-then-answer pattern)
- App alternatives suggest importing Arch Wiki/AUR data when knowledge is missing

### Fixed
- BriefSeverity now implements Default (required for HealthBrief)
- Integer overflow in health_brief_builder for terabyte sizes

## [0.0.32] - 2025-12-05

### Added
- **Humanized IT Department Roster**: Stable person profiles for service desk narration
  - `roster.rs` module with PersonProfile struct (person_id, display_name, role_title, team, tier)
  - Deterministic `person_for(team, tier)` mapping - same inputs always return same person
  - 16 named specialists: Alex, Morgan, Jordan, Taylor, Riley, Casey, Drew, Quinn, etc.

- **Fact Lifecycle Management**: Facts with TTL, staleness, and automatic expiration
  - `StalenessPolicy` enum: Never, TTLSeconds(u64), SessionOnly
  - `FactLifecycle` enum: Active, Stale, Archived
  - `apply_lifecycle()` transitions facts based on current time

- **Health Brief (NEW)**: Relevant-only health status for "how is my computer" queries
  - `health_brief.rs` module with BriefSeverity (Ok, Warning, Error) and BriefItem
  - Only shows actionable items: disk warnings (>85%), memory pressure (>90%), failed services
  - `HealthBrief.format_answer()` returns "Your system is healthy" when nothing needs attention
  - Replaces full system reports for health queries

- **Clarify Module (NEW)**: Clarification questions with verification probes
  - `clarify.rs` module with ClarifyKind enum (PreferredEditor, ServiceName, MountPoint, etc.)
  - `ClarifyQuestion` struct with verification probe template
  - `generate_question()` creates questions with defaults from facts
  - `needs_clarification()` checks if clarification is needed based on query

- **Per-Person Statistics**: Track individual specialist performance
  - `PersonStats` struct with tickets_closed, escalations_sent/received, avg_loops, avg_score
  - `PersonStatsTracker` tracks all 16 roster entries

### Changed
- **Fast Translator Model**: Use smaller, faster model to eliminate timeouts
  - Changed default translator model from qwen2.5:1.5b-instruct to qwen2.5:0.5b-instruct
  - Changed default supervisor model from qwen2.5:1.5b-instruct to qwen2.5:0.5b-instruct
  - Reduced translator timeout from 4s to 2s
  - Reduced specialist timeout from 8s to 6s

- **Faster Budget Defaults**: Bias toward deterministic answers
  - Translator budget: 5s → 1.5s
  - Probes budget: 12s → 8s
  - Specialist budget: 15s → 6s
  - Supervisor budget: 8s → 4s
  - Total budget: 25s → 18s

- **Health Query Routing**: SystemHealthSummary now uses HealthBrief
  - Routes to health_brief_builder instead of full system summary
  - Uses disk_usage, memory_info, failed_services, top_cpu probes
  - Returns "healthy" status when no issues detected

- **Always-Answer Behavior**: Removed "Could you rephrase" failure mode
  - `create_no_data_response()` now builds best-effort answer from available probe data
  - Never asks for rephrase - always provides actionable information
  - Timeout responses no longer ask user to "try again"

### Technical
- Tests updated for new default values
- Golden tests for deterministic health brief output
- All files under 400 lines

## [0.0.31] - 2025-12-05

### Added
- **Facts Store (Phase 1)**: Persistent store for verified user/system facts
  - `facts.rs` module with FactKey enum for typed fact identification
  - `Fact` struct with key, value, verified flag, source, and timestamp
  - `FactsStore` with save/load, deterministic JSON serialization
  - Facts persisted to `~/.anna/facts.json` only when verified
  - `FactStatus` enum: Known, Unknown, Stale for fact querying

- **Intake with Verification Plans (Phase 2)**: Clarification questions with verification
  - `intake.rs` module for query analysis and clarification planning
  - `VerifyPlan` enum: BinaryExists, UnitExists, MountExists, InterfaceExists, etc.
  - `ClarificationQuestion` with question ID, prompt, choices, verify plan
  - `IntakeResult` for intake analysis with clarifications and facts used
  - `ClarificationSlot` enum: EditorName, ConfigPath, NetworkInterface, etc.
  - `analyze_intake()` checks known facts before asking clarifications

- **Verification Probes (Phase 3)**: Safe probes for clarification verification
  - `verify_probes.rs` module with safe read-only verification commands
  - `run_verify_probe()` executes verification based on VerifyPlan
  - `verify_and_store()` verifies and stores fact if valid
  - `VerificationResult` with verified flag, value, alternatives

- **Clarification Ticket States (Phase 4)**: Ticket pause/resume for clarification
  - `AwaitingClarification` and `VerifyingClarification` ticket statuses
  - Clarification fields in Ticket: pending_clarification_id, answer, rounds
  - `set_pending_clarification()`, `set_clarification_answer()`, `complete_clarification()`
  - Transcript events: ClarificationAsked, ClarificationAnswered, ClarificationVerified, FactStored

- **Clarification Templates (Phase 5)**: Learned clarification patterns in recipes
  - `RecipeKind::ClarificationTemplate` for storing learned patterns
  - `RecipeSlot` struct with name, question_id, required, verify_type
  - `Recipe::clarification_template()` constructor for template recipes
  - Templates store which clarifications to ask for an intent

### Changed
- Recipe now includes clarification_slots, default_question_id, populates_facts fields
- Transcript renderer handles new clarification events in debug mode
- Test coverage updated for new event types

### Technical
- All tests passing
- Files remain under 400 lines
- No breaking CLI changes

## [0.0.30] - 2025-12-05

### Fixed
- **Specialist Timeout Fallback (Phase 1-2)**: Fixed "specialist TIMEOUT → useless rephrase" failure mode
  - Health/status queries ("how is my computer", "any errors") now route deterministically before translator
  - Added `strip_greetings()` to ignore "hello", "hi anna", emoticons in query classification
  - Expanded `SystemHealthSummary` patterns to catch conversational health queries
  - `generate_best_effort_summary()` produces useful answers from any available probe evidence
  - When specialist times out but evidence exists, returns parsed summary instead of rephrase request

- **Translator Hardening (Phase 3)**: Prevent greeting + health query misrouting
  - Updated translator prompt with explicit instructions to ignore greetings
  - Added health query examples to guide correct classification (system domain, not network)
  - `translate_fallback()` now detects health queries before domain classification
  - Health fallback returns comprehensive probe set: memory_info, disk_usage, cpu_info, failed_services

### Added
- **Latency Guardrails (Phase 4)**: Protect against slow specialist responses
  - `max_specialist_prompt_bytes` config option (default 16KB) caps prompt size
  - Prompts exceeding cap skip to deterministic fallback immediately
  - Reduced default specialist timeout from 12s to 8s (deterministic fallback handles gaps)
  - Early budget enforcement prevents wasted time on oversized prompts

### Changed
- Default `specialist_timeout_secs` reduced from 12 to 8 (fallback covers timeouts reliably)
- New config option: `llm.max_specialist_prompt_bytes` (default 16384)
- Router patterns expanded for health/status queries
- Translator fallback now health-query-aware

### Technical
- All tests passing
- Files remain under 400 lines
- No breaking API changes

## [0.0.29] - 2025-12-05

### Added
- **StatusSnapshot (Phase 1)**: Comprehensive system state snapshot
  - `status_snapshot.rs` module with complete system state capture
  - `StatusSnapshot` struct: versions, daemon, permissions, update, helpers, models, config
  - `VersionInfo`, `DaemonInfo`, `PermissionsInfo` for granular health data
  - `UpdateInfo`, `UpdateResult` for update subsystem tracking
  - `HelpersInfo`, `ModelsInfo`, `ConfigInfo` for component state
  - `StatusSnapshot` RPC method for detailed status queries
  - `health_status()` returns "OK", "DAEMON_DOWN", "OLLAMA_MISSING", etc.

- **Update Ledger (Phase 2)**: Auto-update transparency and auditability
  - `update_ledger.rs` module for tracking update checks
  - `UpdateCheckEntry`: timestamp, local_version, remote_tag, result, duration
  - `UpdateCheckResult` enum: UpToDate, UpdateAvailable, Downloaded, Installed, Failed
  - `UpdateLedger` with max 20 entries, persisted to `~/.anna/update_ledger.json`
  - Daemon update loop now records all check results with timing

- **Model Registry (Phase 3)**: Hardware-aware model selection
  - `model_registry.rs` module for role-model bindings
  - `ModelSpec`: name, size_hint_gb, quantization
  - `RoleBinding`: team + role to model mapping
  - `HardwareTier` enum: Low/Medium/High/VeryHigh based on RAM/CPU/GPU
  - `recommended_model_for_tier()` returns appropriate model spec
  - `ModelRegistry` tracks bindings and model states
  - `parse_ollama_list()` for model state detection

- **Telemetry Snapshots (Phase 4)**: Measured system deltas
  - `telemetry/mod.rs`, `telemetry/boot.rs`, `telemetry/pacman.rs` modules
  - `BootSnapshot`: tracks boot time changes via systemd-analyze
  - `parse_systemd_analyze_time()` parses various boot time formats
  - `PacmanSnapshot`: tracks package events from /var/log/pacman.log
  - `PackageEvent`: timestamp, kind (installed/upgraded/removed), package, version
  - Checkpoint-based incremental log reading for efficiency
  - REPL greeting now shows measured telemetry when available
  - Shows "boot X.Xs faster/slower" and "N pkg changes" in greeting

### Changed
- `update_check_loop` now records all results to UpdateLedger
- `DaemonStateInner.to_status_snapshot()` builds comprehensive snapshot
- `print_repl_header()` shows telemetry data when available

### Technical
- All new modules under 400 lines per project guidelines
- All tests passing (257+ tests)
- StatusSnapshot RPC wired into daemon handlers

## [0.0.28] - 2025-12-05

### Added
- **Team-Specialized Junior/Senior Execution (Phase 1)**
  - Extended `SpecialistsRegistry` with prompt accessors
  - `SpecialistProfile.prompt()` returns team-specific prompt
  - `SpecialistsRegistry.junior_prompt(team)` and `senior_prompt(team)`
  - `SpecialistsRegistry.junior_model(team)` and `senior_model(team)`
  - `SpecialistsRegistry.escalation_threshold(team)`

- **Helpers Management (Phase 2)**: Track external dependencies
  - `helpers.rs` module for helper package tracking
  - `HelperPackage` struct: id, name, version, install_source, available
  - `InstallSource` enum: Anna, User, Bundled, Unknown
  - `HelpersRegistry` for managing tracked packages
  - `known_helpers()` returns default helper definitions (ollama)
  - `detect_helper()` for system package detection
  - Persistence to `~/.anna/helpers.json`

- **True Reset (Phase 3)**: `annactl reset` now wipes all learned data
  - Clears ledger (existing behavior)
  - Clears recipes (`~/.anna/recipes/`)
  - Clears helpers store (`~/.anna/helpers.json`)
  - Enhanced reset confirmation dialog showing what will be cleared
  - Returns list of cleared stores in response

- **IT Department Dialog Style (Phase 4)**: Polish for non-debug mode
  - `it_greeting(domain)` - contextual greeting based on query type
  - `it_confidence(score)` - reliability as IT confidence statement
  - `it_domain_context(domain)` - domain as IT department context
  - Clean mode output now uses IT department style formatting
  - Footer shows: Domain Context | Confidence Note | Score

### Changed
- Moved specialists tests to `tests/specialists_tests.rs` (file now 232 lines)
- Enhanced `handle_reset` handler to clear recipes and helpers
- Updated `handle_reset` command with better user feedback

### Tests
- 6 new specialists registry tests for v0.0.28 features
- 10 new helpers module tests
- 3 new narrator IT department style tests

## [0.0.27] - 2025-12-05

### Added
- **Recipe Learning Loop**: Team-tagged recipes for learning from successful patterns
  - `RecipeKind` enum: Query, ConfigEnsureLine, ConfigEditLineAppend
  - `RecipeTarget` struct with app_id and config_path_template
  - `RecipeAction` enum: EnsureLine, AppendLine, None
  - `RollbackInfo` struct for reversible changes
  - Recipe persistence to `~/.anna/recipes/` with deterministic naming
  - Only persists when: Ticket status = Verified, reliability score >= 80

- **Safe Change Engine**: Backup-first, idempotent config edits
  - `ChangePlan` struct describing what will change before execution
  - `ChangeResult` struct with applied, was_noop, backup_path, diagnostics
  - `ChangeOperation` enum: EnsureLine, AppendLine
  - `ChangeRisk` levels: Low, Medium, High
  - `plan_ensure_line()` function for planning changes
  - `apply_change()` function with automatic backup
  - `rollback()` function to restore from backup
  - Deterministic backup naming using path hash

- **Config Intent Detection**: Pattern-based detection for config edit requests
  - `detect_vim_config_intent()` for vim config patterns
  - `detect_config_intent()` for general config detection
  - Supports: syntax on, line numbers, autoindent, mouse, tabs
  - Bridges query classification to change engine

- **Stats Command**: Per-team statistics via `annactl stats`
  - Total requests, success rate, avg reliability
  - Per-team breakdown: total, success, failed, avg rounds, avg score
  - Most consulted team indicator

- **Enhanced Team Routing**: Desktop team routes for editor configs
  - Added vim, nano, emacs, syntax, config_edit route classes

### Changed
- Extracted change.rs tests to tests/change_tests.rs (under 400 lines)
- Added `Stats` RPC method for statistics retrieval

### Tests
- 8 new change engine tests (tempdir-based)
- 9 new config intent detection tests
- 18 recipe tests including v0.0.27 config edit recipes

## [0.0.26] - 2025-12-05

### Added
- **SPECIALISTS Registry**: Team-scoped specialist system
  - `SpecialistRole` enum: Translator, Junior, Senior
  - `SpecialistProfile` struct with team, role, model_id, max_rounds, escalation_threshold
  - `SpecialistsRegistry` with 24 default profiles (8 teams × 3 roles)
  - Teams: Desktop, Storage, Network, Performance, Services, Security, Hardware, General

- **Deterministic Review Gate**: Hybrid review that minimizes LLM calls
  - `ReviewContext` struct with all deterministic signals
  - `GateOutcome` with decision, reasons, requires_llm_review, confidence
  - Pure `deterministic_review_gate()` function - no I/O
  - Rules: Invention → Escalate, No claims → Revise, Low grounding → Revise, High score → Accept
  - Medium scores (50-79) trigger LLM review only when needed

- **Team-Specific Review Prompts**: Customized junior/senior prompts per team
  - Each team has domain-specific verification rules
  - Storage: verify df/lsblk output exactly
  - Network: verify ip/ss output
  - Performance: verify free/top output
  - Security: flag risky operations

- **Review Gate Transcript Events**:
  - `ReviewGateDecision { decision, score, requires_llm }`
  - `TeamReview { team, reviewer, decision, issues_count }`
  - Full visibility into review decisions

- **Trace Enhancements**:
  - `ReviewerOutcome` enum for audit trail
  - `FallbackUsed::Timeout` variant for timeout fallback tracking

- **Ticket Service Integration**:
  - `run_review_gate()` function wired into ticket verification
  - Transcript events emitted for all gate decisions

### Changed
- Refactored `transcript.rs` (495→368 lines) with `transcript_ext.rs` extension module
- Split `review_prompts.rs` into modular directory structure
- All files now under 400 line limit per project standards

### Tests
- 550+ tests passing
- Golden tests for specialists registry serialization
- Golden tests for review gate decisions
- Tests for transcript event creation

## [0.0.23] - 2025-12-05

### Added
- **TRACE Phase**: Structured execution trace for debugging degraded paths
  - `ExecutionTrace` in ServiceDeskResult (wire-compatible, optional)
  - `SpecialistOutcome` enum: Ok | Timeout | BudgetExceeded | Skipped | Error
  - `FallbackUsed` enum: None | Deterministic { route_class }
  - `ProbeStats`: planned/succeeded/failed/timed_out counts
  - `EvidenceKind` enum: Memory | Disk | BlockDevices | Cpu | Services
  - Trace rendering in `annactl` debug mode
  - 12 golden tests for trace scenarios

- **TRUST+ Phase**: Enhanced reliability explanations with fallback context
  - `ReasonContext` extended with fallback fields
  - ProbeTimeout explanation now mentions fallback source and evidence kinds
  - Example: "2 of 3 probes timed out; used deterministic fallback from memory evidence"

- **RESCUE Hardening**: Explicit threshold constants for reliability scoring
  - `INVENTION_CEILING = 40`
  - `PENALTY_NOT_GROUNDED = -30`
  - `PENALTY_BUDGET_EXCEEDED = -15`
  - `PENALTY_PROBE_TIMEOUT = -10`
  - `PENALTY_EVIDENCE_MISSING = -25`
  - `MAX_PROBE_COVERAGE_PENALTY = 30.0`
  - Confidence thresholds: `CONFIDENCE_LOW_THRESHOLD = 0.70`, `CONFIDENCE_MEDIUM_THRESHOLD = 0.85`
  - All magic numbers in `compute_reliability()` replaced with named constants

- **New Parsers**: `lsblk.rs` and `lscpu.rs` for block device and CPU info
- **Probe Answers Module**: Centralized deterministic answer generation

### Changed
- `DeterministicResult` now includes `route_class` field for trace auditing
- All deterministic answers include route classification

## [0.0.18] - 2025-12-05

### Fixed
- **Duplicate `[anna]` block**: Debug mode no longer prints final answer twice
  - Transcript renderer tracks if Anna's answer was already printed in event stream
  - Only prints fallback `[anna]` block if no Anna message was rendered
- **CLI `help` command conflict**: `annactl help` now sends "help" as a request to Anna
  - Disabled clap's implicit help subcommand (`disable_help_subcommand = true`)
  - `annactl --help` still shows CLI usage
- **Misleading specialist output**: Deterministic path shows correct stage status
  - New `StageOutcome::Deterministic` variant
  - Shows `[specialist] skipped (deterministic)` instead of `ok`

### Added
- CLI integration tests for argument parsing regressions
- `ProgressTracker::skip_stage_deterministic()` for cleaner stage handling

## [0.0.17] - 2025-12-05

### Added
- **docs/VERIFICATION.md**: Comprehensive verification guide with exact commands
  - Binary verification, release asset checks, smoke tests
  - Per-feature validation commands for deterministic outputs

### Changed
- **SPEC.md updated to v0.0.16**: Full specification refresh
  - Documents all features from v0.0.13-v0.0.16
  - Pipeline flow diagram, configuration reference
  - Latency stats, timeout handling, probe allowlist

### Fixed
- Cleaned up dead code warnings with `#[allow(dead_code)]` annotations
- Removed unused imports in test files and commands.rs

## [0.0.16] - 2025-12-05

### Added
- **Global Request Timeout**: Configurable `request_timeout_secs` in config.toml (default 20s)
  - Entire pipeline wrapped in global timeout
  - Graceful timeout response with clarification message
- **Per-Stage Latency Stats**: Track avg and p95 latency for last 20 requests
  - Exposed via `annactl status --debug` flag
  - Tracks translator, probes, specialist, and total latency
- **`annactl status --debug`**: Extended status output showing latency statistics
- **v0.0.16 Golden Tests**: Tests for PID column, CRITICAL warnings, state display

### Changed
- **Deterministic Outputs Improved**:
  - top_memory: Shows 10 processes with PID, COMMAND, %MEM, RSS, USER
  - network_addrs: Shows active connection at top ("Active: Wi-Fi (wlan0)...")
  - RSS values formatted human-readable (12M, 1.2G)
- **Translator JSON Parser**: Fully tolerant of malformed JSON
  - Parse errors fallback to defaults instead of failing
  - Missing confidence defaults to 0.0
  - Null arrays become empty Vec
- **Strict Translator Prompt**: Forces exact enum values (intent, domain)
- **Parser Struct Updates**: ProcessInfo now includes `pid` and `rss` fields

### Fixed
- All source files kept under 400 line limit
- Removed unused `extract_pid_from_process` function

## [0.0.15] - 2025-12-05

### Added
- **Triage Router**: Handles ambiguous queries with LLM translator and confidence thresholds
  - Confidence < 0.7 triggers immediate clarification (reliability capped at 40%)
  - Probe cap at 3 maximum per query, warning in evidence if exceeded
  - Deterministic clarification generator fallback if translator fails
- **Probe Summarizer**: Compacts probe outputs to <= 15 lines for specialist
  - Raw output only sent in debug mode with explicit "show raw" request
- **Evidence Redaction**: Automatic removal of sensitive patterns
  - Private keys, password hashes, AWS keys, API tokens
  - Applied even in debug mode for security
- **Two Display Modes**:
  - debug OFF: Clean fly-on-the-wall format (`anna vX.Y.Z`, `[you]`, `[anna]`, reliability/domain footer)
  - debug ON: Full stages with consistent speaker tags on separate lines
- **REPL Polish**:
  - Spinner only in debug OFF mode while waiting
  - Stage transitions shown in debug ON mode
  - Improved help command with examples
- **Config-based debug mode**: `daemon.debug_mode` in config.toml

### Changed
- **Specialist receives summarized probes**: Never raw stdout unless debug + "show raw"
- **Scoring refinement**: Triage path grounded=true only if answer references probe/snapshot facts
- **Clarification max reliability**: Capped at 40% when clarification returned
- **Transcript format**: Content starts on line after speaker tag, no inline arrows

### Fixed
- Display consistency between REPL and one-shot modes
- Redundant separators and spacing in output
- Final [anna] block always present with answer (never empty)

## [0.0.14] - 2025-12-04

### Added
- **Deterministic Router**: Overrides LLM translator for known query classes
  - CPU/RAM/GPU queries: Use hardware snapshot, no probes needed
  - Memory processes: Automatically runs `top_memory` probe
  - CPU processes: Automatically runs `top_cpu` probe
  - Disk queries: Routes to Storage domain with `disk_usage` probe
  - Network queries: Routes to Network domain with `network_addrs` probe
  - "help": Returns deterministic help response
  - "slow/sluggish": Runs multi-probe diagnostic (CPU, memory, disk)
- **Help command**: "help" now returns comprehensive usage guide
- **Interface type detection**: WiFi vs Ethernet heuristics (wlan*/wlp* = WiFi)
- **Golden tests**: Router, translator robustness, scoring validation

### Changed
- **Translator JSON parsing tolerant**: Missing fields use sensible defaults
  - Missing `confidence` → 0.0
  - Null arrays → empty Vec
  - Missing `intent`/`domain` → fallback to deterministic router
- **Specialist skipped for known classes**: Deterministic answers bypass LLM
- **Scoring reflects reality**:
  - `grounded=true` only if parsed data count > 0
  - Empty parser result = clarification needed, not 100% score
  - Coverage based on actual probe success, not request count
- **Improved deterministic outputs**:
  - Process tables include PID column
  - Disk usage shows critical (>=95%) and warning (>=85%) status
  - Network interfaces show type (WiFi/Ethernet/Loopback)

### Fixed
- Known query classes can't be misrouted by LLM translator
- Translator errors don't block deterministic answering
- Empty parser results don't claim 100% reliability

## [0.0.13] - 2025-12-04

### Added
- **Per-stage model selection**: Configure different models for each pipeline stage
  - `translator_model`: Fast small model for query classification (default: qwen2.5:1.5b-instruct)
  - `specialist_model`: Capable model for domain expert answers (default: qwen2.5:7b-instruct)
  - `supervisor_model`: Validation model (default: qwen2.5:1.5b-instruct)
- **Config file support**: `/etc/anna/config.toml` with LLM section
- **Configurable timeouts**: Per-stage timeouts in config file
  - `translator_timeout_secs`: 4s (default)
  - `specialist_timeout_secs`: 12s (default)
  - `supervisor_timeout_secs`: 6s (default)
  - `probe_timeout_secs`: 4s (default)

### Changed
- **Translator payload minimized**: < 2KB for typical requests
  - Inputs: user query, one-line hardware summary, probe ID list
  - NO probe stdout/stderr, NO evidence blocks, NO long policy text
- **Daemon pulls all required models on startup/healthcheck**
- **Status shows all models with roles** (translator, specialist, supervisor)
- **Models pulled based on config**, not hardware detection

### Fixed
- Translator no longer receives large probe outputs
- Consistent timeout values across pipeline stages

## [0.0.12] - 2025-12-04

### Added
- **Deterministic Answerer**: Fallback module that answers common queries without LLM
  - CPU info: From hardware snapshot or lscpu probe
  - RAM info: From hardware snapshot or free -h probe
  - GPU info: From hardware snapshot
  - Top memory processes: Parsed from ps aux --sort=-%mem
  - Disk space: Parsed from df -h with critical/warning flags
  - Network interfaces: Parsed from ip addr show
  - Rules: Never invents facts, always produces grounded answers

### Changed
- **Specialist timeout behavior**: Now tries deterministic answerer instead of asking for clarification
- **Scoring improvements**:
  - Deterministic answers get `answer_grounded=true` and `no_invention=true` automatically
  - `translator_confident` is false if translator timed out
  - Score no longer capped at 20 when probes succeed with deterministic answer
- **Domain consistency**: ServiceDeskResult.domain now matches the classified domain
- **Update check**: Verifies release assets exist before showing update available

### Fixed
- Anna now produces answers even when specialist LLM times out (reliability > 20)
- Domain in summary now matches dispatcher routing
- Clarification no longer shown when probe data is available

## [0.0.11] - 2024-12-04

### Added
- **Transcript event model**
  - Single `TranscriptEvent` type for pipeline visibility
  - Events: Message, StageStart, StageEnd, ProbeStart, ProbeEnd, Note
  - Actors: You, Anna, Translator, Dispatcher, Probe, Specialist, Supervisor, System
  - Full request tracing with elapsed timestamps

- **Two render modes**
  - debug OFF: Human-readable fly-on-the-wall format
  - debug ON: Full troubleshooting view with stage timings

- **REPL improvements**
  - Prompt changed to `anna> `
  - Ctrl-D (EOF) now exits cleanly
  - Empty lines after answers for readability

- **CI improvements**
  - Release artifact naming check
  - Test files excluded from 400-line limit

### Changed
- ServiceDeskResult now includes `request_id` and `transcript`
- Transcript events generated during pipeline execution
- Refactored rpc_handler.rs to stay under 400 lines
  - Extracted utility handlers to handlers.rs
  - Extracted ProgressTracker to progress_tracker.rs

### Fixed
- Release script already had correct artifact naming (annad-linux-x86_64, annactl-linux-x86_64)
- CI now verifies release script uses correct names

## [0.0.7] - 2024-12-04

### Added
- **Service desk architecture**
  - Internal roles: translator, dispatcher, specialist, supervisor
  - Specialist domains: system, network, storage, security, packages
  - Automatic domain classification from query
- **Reliability scores**
  - Every response includes 0-100 reliability score
  - Score increases with successful probes
  - Color-coded display (green >80%, yellow 50-80%, red <50%)
- **Unified output format**
  - One-shot and REPL use identical formatting
  - Shows version, specialist domain, reliability, probes used
  - Consistent `[you]`/`[anna]` transcript blocks
- **Probe allowlist**
  - Only 11 read-only commands allowed
  - Dangerous commands are explicitly denied
  - Security tests verify allowlist safety
- **Clarification rules**
  - Short/ambiguous queries ask for more details
  - "help" without context triggers clarification
- **Golden tests**
  - 16 new tests for service desk behavior
  - Domain routing tests
  - Probe security tests
  - Output format consistency tests

### Changed
- **Request pipeline now uses service desk**
  - translate → dispatch → specialist → supervisor
  - All responses include ServiceDeskResult metadata
- **Response format includes domain and reliability**
  - No longer just raw text response
  - Full metadata for transparency

### Fixed
- REPL and one-shot now produce identical output format
- Commands.rs uses single send_request function for both modes

## [0.0.6] - 2024-12-04

### Added
- **Grounded LLM responses**
  - RuntimeContext injected into every LLM request
  - Hardware snapshot (CPU, RAM, GPU) always available to LLM
  - Capability flags prevent claiming abilities Anna doesn't have
- **Auto-probes for queries**
  - Memory/process queries auto-run `ps aux --sort=-%mem`
  - Disk queries auto-run `df -h`
  - Network queries auto-run `ip addr show`
- **Probe RPC method**
  - `top_memory` - Top processes by memory
  - `top_cpu` - Top processes by CPU
  - `disk_usage` - Filesystem usage
  - `network_interfaces` - Network info
- **Integration tests for grounding**
  - Version consistency tests
  - Hardware context tests
  - Capability safety tests

### Changed
- **System prompt completely rewritten**
  - Strict grounding rules enforced
  - Never invents facts not in context
  - Answers hardware questions from snapshot
  - Never suggests manual commands when data available

### Fixed
- Anna no longer claims to be "v0.0.1" or wrong versions
- Anna no longer suggests `lscpu` when CPU info is in context
- Anna answers memory questions with actual process data

### Documentation
- SPEC.md updated to v0.0.6 with grounding policy
- README.md updated with features
- TRUTH_REPORT.md documents what was broken and how it was fixed

## [0.0.5] - 2024-12-04

### Added
- **Enhanced status display**
  - CPU model and core count
  - RAM total in GB
  - GPU model and VRAM
- **Improved REPL exit commands**
  - Added: bye, q, :q, :wq (for vim users!)

### Changed
- **Smarter model selection**
  - With 8GB VRAM: llama3.1:8b (was llama3.2:3b)
  - With 12GB+ VRAM: qwen2.5:14b
  - Better tiered selection based on GPU/RAM

### Fixed
- Friendlier goodbye message

## [0.0.4] - 2024-12-04

### Added
- **Auto-update system**
  - GitHub release version checking every 60 seconds
  - Automatic download and verification of new releases
  - Zero-downtime updates via atomic binary replacement
  - SHA256 checksum verification for security
- **Enhanced status display**
  - Current version and available version from GitHub
  - Update check pace (every 60s)
  - Countdown to next update check
  - Auto-update enabled/disabled status
  - "update available" indicator when new version exists
- **Security and permissions**
  - Dedicated `anna` group for socket access
  - Installer automatically creates group and adds user
  - Health check auto-adds new users to anna group
  - No reboot needed - `newgrp anna` activates immediately
  - Fallback to permissive mode if group unavailable

### Changed
- Update check interval reduced from 600s to 60s
- Status output now shows comprehensive version/update information
- Socket permissions now use group-based access (more secure)

## [0.0.3] - 2024-12-04

### Added
- **Self-healing health checks**
  - Periodic health check loop (every 30 seconds)
  - Automatic detection of missing Ollama or models
  - Auto-repair sequence when issues detected
- **Package manager support**
  - Ollama installation via pacman on Arch Linux
  - Fallback to official installer for other distros
- **Friendly bootstrap UI**
  - Live progress display when environment not ready
  - "Hello! I'm setting up my environment. Come back soon! ;)"
  - Spinner with phase and progress bar
  - Auto-continues when ready

### Changed
- annactl now waits and shows progress if LLM not ready
- REPL shows bootstrap progress before accepting input
- Requests wait for bootstrap completion automatically
- Split display code into separate module for maintainability

### Fixed
- Socket permissions allow regular users to connect
- Installer stops existing service before upgrade

## [0.0.2] - 2024-12-04

### Added
- **Beautiful terminal UI**
  - Colored output with ANSI true color (24-bit)
  - Progress bars for downloads
  - Formatted byte sizes (1.2 GB, 45 MB, etc.)
  - Formatted durations (2h 30m 15s)
  - Consistent styling across all commands
- **Enhanced status display**
  - LLM state indicators (Bootstrapping, Ready, Error)
  - Benchmark results display (CPU, RAM, GPU status)
  - Model information with roles
  - Download progress with ETA
  - Uptime and update check timing
- **Improved installer**
  - Beautiful step-by-step output
  - Clear sudo explanations
  - Checksum verification display

### Changed
- Refactored status types for richer UI
- Moved UI helpers to anna-shared for consistency

## [0.0.1] - 2024-12-04

### Added
- Initial release with complete repository rebuild
- **annad**: Root-level systemd daemon
  - Automatic Ollama installation and management
  - Hardware probing (CPU, RAM, GPU detection)
  - Model selection based on system resources
  - Installation ledger for safe uninstall
  - Update check ticker (every 600 seconds)
  - Unix socket RPC server (JSON-RPC 2.0)
- **annactl**: User CLI
  - `annactl <request>` - Send natural language request
  - `annactl` - Interactive REPL mode
  - `annactl status` - Show system status
  - `annactl reset` - Reset learned data
  - `annactl uninstall` - Safe uninstall via ledger
  - `annactl -V/--version` - Show version
- Installer script (`scripts/install.sh`)
- Uninstaller script (`scripts/uninstall.sh`)
- CI workflow with enforcement checks:
  - 400-line file limit
  - CLI surface verification
  - Build and test verification

### Security
- annad runs as root systemd service
- annactl communicates via Unix socket
- No remote network access except for Ollama API and model downloads

### Known Limitations
- v0.0.1 supports read-only operations only
- Full LLM pipeline planned for future versions
- Single model support only

[Unreleased]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.18...HEAD
[0.0.18]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.17...v0.0.18
[0.0.17]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.16...v0.0.17
[0.0.16]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.15...v0.0.16
[0.0.15]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.14...v0.0.15
[0.0.14]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.13...v0.0.14
[0.0.13]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.12...v0.0.13
[0.0.12]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.11...v0.0.12
[0.0.11]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.7...v0.0.11
[0.0.7]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/jjgarcianorway/anna-assistant/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/jjgarcianorway/anna-assistant/releases/tag/v0.0.1
