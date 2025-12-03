# Anna Assistant - Implementation Roadmap

**Current Version: 0.0.16**

This roadmap migrates from the v7.42.5 snapshot-based architecture to the full natural language assistant while preserving performance.

---

## Phase 1: CLI Surface Lockdown (0.0.x)

### 0.0.2 - Strict CLI Surface (COMPLETED)
- [x] Remove `sw` command from public surface
- [x] Remove `hw` command from public surface
- [x] Remove all JSON flags from public surface
- [x] Keep only: `annactl`, `annactl <request>`, `annactl status`, `annactl --version`
- [x] Legacy commands route as natural language requests (no custom error)
- [x] REPL mode basic implementation (exit, quit, help, status)
- [x] CLI tests for new surface

### 0.0.3 - Request Pipeline Skeleton (COMPLETED)
- [x] Create DialogueActor enum (You, Anna, Translator, Junior, Annad)
- [x] Full multi-party dialogue transcript
- [x] Deterministic Translator mock (intent classification: question, system_query, action_request, unknown)
- [x] Target detection (cpu, memory, disk, docker, etc.)
- [x] Risk classification (read-only, low-risk, medium-risk, high-risk)
- [x] Evidence retrieval mock from snapshots
- [x] Junior scoring rubric (+40 evidence, +30 confident, +20 observational+cited, +10 read-only)
- [x] CLI tests for pipeline behavior

### 0.0.4 - Real Junior Verifier (COMPLETED)
- [x] Ollama HTTP client in anna_common
- [x] Junior config keys (junior.enabled, junior.model, junior.timeout_ms, junior.ollama_url)
- [x] Real Junior LLM verification via Ollama
- [x] Junior system prompt with scoring rubric
- [x] Junior output parsing (SCORE, CRITIQUE, SUGGESTIONS, MUTATION_WARNING)
- [x] Fallback to deterministic scoring when Ollama unavailable
- [x] Spinner while Junior thinks
- [x] Graceful handling when Ollama not available
- [x] Model auto-selection (prefers qwen2.5:1.5b, llama3.2:1b, etc.)
- [x] CLI tests for Junior LLM behavior

### 0.0.5 - Role-Based Model Selection + Benchmarking (COMPLETED)
- [x] Hardware detection module (CPU/RAM/GPU profiling)
- [x] Hardware tier classification (Low: <8GB, Medium: 8-16GB, High: >16GB)
- [x] LlmRole enum (Translator, Junior)
- [x] Role-based model candidate pools with priority
- [x] Model selection based on hardware tier and availability
- [x] Translator benchmark suite (30 prompts for intent classification)
- [x] Junior benchmark suite (15 cases for verification quality)
- [x] Ollama pull with progress streaming and ETA
- [x] BootstrapPhase state machine (detecting_ollama, installing_ollama, pulling_models, benchmarking, ready, error)
- [x] Bootstrap state in status snapshot
- [x] annactl progress display when models not ready
- [x] Graceful degradation with reduced reliability score when LLM unavailable
- [x] Unit tests for hardware bucketing and model selection

### 0.0.6 - Real Translator LLM (COMPLETED)
- [x] Real Translator LLM integration replacing deterministic translator
- [x] Translator structured output parsing (intent, targets, risk, evidence_needs, clarification)
- [x] Clarify-or-proceed loop with multiple-choice prompts
- [x] Evidence-first pipeline with real snapshot integration
- [x] 8KB evidence excerpt cap with truncation indication
- [x] Action plan generation for action requests (steps, affected resources, rollback outline)
- [x] Confirmation-gated action plan display (no execution)
- [x] Deterministic fallback when Translator LLM unavailable
- [x] 15 unit tests for Translator parsing, clarification, evidence, action plans
- [x] CLI tests updated for v0.0.6

### 0.0.7 - Read-Only Tooling & Evidence Citations (COMPLETED)
- [x] Read-only tool catalog (tools.rs) with allowlist enforcement
- [x] 10 read-only tools: status_snapshot, sw_snapshot_summary, hw_snapshot_summary, recent_installs, journal_warnings, boot_time_trend, top_resource_processes, package_info, service_status, disk_usage
- [x] Tool executor with structured outputs and human summaries
- [x] Evidence IDs (E1, E2, ...) assigned by EvidenceCollector
- [x] Translator outputs tool plans in TOOLS field
- [x] Tool plan parsing from Translator LLM output
- [x] Deterministic fallback generates tool plans from evidence_needs
- [x] Junior no-guessing enforcement (require citations, UNCITED_CLAIMS output)
- [x] Natural language dialogue transcripts for tool execution
- [x] Evidence ID citations in final responses
- [x] 7 new unit tests for tool catalog, evidence collector, uncited claims
- [x] Updated main.rs and pipeline.rs to v0.0.7

### 0.0.8 - First Safe Mutations (COMPLETED)
- [x] Mutation tool catalog with allowlist enforcement (6 tools)
- [x] Config file edits: /etc/**, $HOME/** (< 1 MiB, text only)
- [x] Systemd operations: restart, reload, enable --now, disable --now, daemon-reload
- [x] Automatic rollback: timestamped backups in /var/lib/anna/rollback/
- [x] Structured mutation logs (JSON per-request + JSONL append log)
- [x] Confirmation gate: exact phrase "I CONFIRM (medium risk)"
- [x] Junior verification threshold: >= 70% reliability required
- [x] MutationPlan, MutationRequest, MutationResult types
- [x] RollbackManager with file hashing and diff summaries
- [x] ActionPlan extended with mutation_plan and is_medium_risk_executable
- [x] handle_mutation_execution() in pipeline
- [x] Unit tests for path validation, confirmation, backup, rollback

### 0.0.9 - Package Management + Helper Tracking (COMPLETED)
- [x] Helper tracking system with provenance (helpers.rs)
- [x] HelperDefinition, HelperState, HelpersManifest types
- [x] InstalledBy enum: anna | user | unknown
- [x] Two dimensions tracked: present/missing + installed_by
- [x] get_helper_status_list(), refresh_helper_states()
- [x] Package management mutation tools: package_install, package_remove (8 tools total)
- [x] Only anna-installed packages removable via package_remove
- [x] Package transaction logging (MutationType::PackageInstall/Remove)
- [x] [HELPERS] section in annactl status
- [x] StatusSnapshot extended with helpers_total, helpers_present, helpers_missing, helpers_anna_installed
- [x] Unit tests for helper provenance tracking

### 0.0.10 - Reset + Uninstall + Installer Review (COMPLETED)
- [x] Add `annactl reset` command with factory reset
- [x] Add `annactl uninstall` command with provenance-aware helper removal
- [x] Install state tracking (install_state.json)
- [x] Installer review with auto-repair capabilities
- [x] Confirmation phrases: "I CONFIRM (reset)" and "I CONFIRM (uninstall)"
- [x] [INSTALL REVIEW] section in annactl status
- [x] Reset runs installer review at end and reports health

### 0.0.11 - Safe Auto-Update System (COMPLETED)
- [x] Update channels: stable (default) and canary
- [x] UpdateConfig with channel and min_disk_space
- [x] UpdateState enhanced with phase, progress, ETA tracking
- [x] UpdateManager for complete update lifecycle
- [x] Safe download with integrity verification (SHA256)
- [x] Staging directory for atomic updates
- [x] Backup of current binaries for rollback
- [x] Atomic installation via rename
- [x] Zero-downtime restart via systemd
- [x] Automatic rollback on failure
- [x] Guardrails: disk space, mutation lock, installer review
- [x] Update progress display in annactl status
- [x] Unit tests for version comparison and channel matching

### 0.0.12 - Proactive Anomaly Detection (COMPLETED)
- [x] AnomalyEngine with periodic detection (anomaly_engine.rs)
- [x] Anomaly signals: boot time regression, CPU load, memory pressure, disk space, crashes, services
- [x] Alert queue with deduplication and severity (Info, Warning, Critical)
- [x] Evidence IDs for all anomalies (ANO##### format)
- [x] what_changed(days) correlation tool
- [x] slowness_hypotheses(days) analysis tool
- [x] Alert surfacing in REPL welcome
- [x] Alert footer in one-shot mode
- [x] [ALERTS] section enhanced in status display
- [x] 16 unit tests for anomaly detection

### 0.0.13 - Conversation Memory + Recipe Evolution (COMPLETED)
- [x] Session memory with local storage (memory.rs)
- [x] Privacy default: summaries only, not raw transcripts
- [x] Recipe system with named intent patterns (recipes.rs)
- [x] Recipe creation threshold: Junior reliability >= 80%
- [x] Recipe matching (keyword-based/BM25 style)
- [x] User introspection via natural language (introspection.rs)
- [x] Forget/delete requires "I CONFIRM (forget)" confirmation
- [x] Status display with [LEARNING] section
- [x] Junior enforcement for learning claims (MEM/RCP citations required)
- [x] 28 unit tests for memory, recipes, introspection

### 0.0.14 - Policy Engine + Security Posture (COMPLETED)
- [x] Policy engine with TOML configuration files (policy.rs)
- [x] Four policy files: capabilities.toml, risk.toml, blocked.toml, helpers.toml
- [x] Policy-driven allowlists (no hardcoded deny rules)
- [x] Policy evidence IDs (POL##### format) in transcript
- [x] Audit logging with secret redaction (audit_log.rs)
- [x] Installer review policy sanity checks
- [x] Junior policy enforcement rules
- [x] 24 unit tests for policy and audit systems

### 0.0.15 - Governance UX Polish (COMPLETED)
- [x] Debug levels configuration in config.toml (ui.debug_level = 0|1|2)
- [x] Level 0 (minimal): only [you]->[anna] and final [anna]->[you], plus confirmations
- [x] Level 1 (normal/default): dialogues condensed, tool calls summarized, evidence IDs
- [x] Level 2 (full): full dialogues, tool execution summaries, Junior critique
- [x] Unified formatting module (display_format.rs) with colors, SectionFormatter, DialogueFormatter
- [x] Enhanced annactl status with sections: VERSION, INSTALLER REVIEW, UPDATES, MODELS, POLICY, HELPERS, ALERTS, LEARNING, RECENT ACTIONS, STORAGE
- [x] Format helpers: format_bytes, format_timestamp, wrap_text, indent
- [x] UiConfig struct with colors_enabled and max_width
- [x] 19 unit tests for display formatting
- [x] 5 unit tests for UI config

### 0.0.16 - Better Mutation Safety (COMPLETED)
- [x] Mutation state machine: planned -> preflight_ok -> confirmed -> applied -> verified_ok | rolled_back
- [x] Preflight checks for file edits (path, permissions, size, hash, backup)
- [x] Preflight checks for systemd ops (unit exists, state captured, policy)
- [x] Preflight checks for package ops (distro, packages, disk space)
- [x] Dry-run diff preview for file edits (line-based diff, truncated output)
- [x] DiffPreview struct with additions/deletions/modifications counts
- [x] Post-check verification for all mutation types
- [x] Automatic rollback on post-check failure
- [x] SafeMutationExecutor with full lifecycle management
- [x] Junior enforcement rules for mutation safety (penalties for missing preflight/diff/post-check)
- [x] 21 unit tests for mutation safety system

### 0.0.17 - REPL Enhancement (Planned)
- [ ] Improve REPL welcome message with level/XP display
- [ ] Add history support for REPL

---

## Phase 2: Full LLM Integration (0.1.x)

### 0.1.0 - Anna Response Generation
- [ ] Connect Anna response generation to Ollama
- [ ] Stream output per participant
- [ ] Real evidence parsing and answer formulation

### 0.1.1 - Senior Escalation
- [ ] Create escalation criteria (confidence < threshold, needs_senior flag)
- [ ] Create Senior LLM prompt template
- [ ] Implement query_senior() function
- [ ] Multi-round improvement loop

---

## Phase 3: Evidence System (0.2.x)

### 0.2.0 - Snapshot Integration
- [ ] Route hardware queries to hw.json snapshot
- [ ] Route software queries to sw.json snapshot
- [ ] Route status queries to status_snapshot.json
- [ ] Add snapshot source citations to answers

### 0.2.1 - Command Execution
- [ ] Define safe command whitelist (read-only)
- [ ] Implement safe command runner
- [ ] Capture command output as evidence
- [ ] Add command output citations

### 0.2.2 - Log Evidence
- [ ] Query journalctl for relevant logs
- [ ] Extract error/warning patterns
- [ ] Add log excerpt citations

---

## Phase 4: Safety Gates (0.3.x)

### 0.3.0 - Action Classification
- [ ] Define ActionRisk enum (ReadOnly, LowRisk, MediumRisk, HighRisk)
- [ ] Create action classifier
- [ ] Implement confirmation prompts per risk level
- [ ] "I assume the risk" for high-risk actions

### 0.3.1 - Rollback Foundation
- [ ] Create backup before file modifications
- [ ] Store timestamped backups
- [ ] Store patch diffs
- [ ] Create rollback instruction set

### 0.3.2 - btrfs Integration
- [ ] Detect btrfs filesystem
- [ ] Create pre-action snapshots when available
- [ ] Expose snapshot in rollback plan

---

## Phase 5: Learning System (0.4.x) - COMPLETED in 0.0.13

### 0.4.0 - Recipe Storage (COMPLETED)
- [x] Define Recipe struct (trigger, steps, verification, rollback, risk)
- [x] Create recipe store (JSON files)
- [x] Implement recipe save/load
- [x] Recipe versioning

### 0.4.1 - Recipe Matching (COMPLETED)
- [x] Match user intent to existing recipes
- [x] Execute recipe steps
- [ ] Skip Junior/Senior when recipe exists (future optimization)
- [x] Track recipe usage stats

### 0.4.2 - Recipe Learning (COMPLETED)
- [x] Detect recipe-worthy interactions (reliability >= 80%)
- [x] Extract recipe from successful answer
- [x] Create new recipe
- [x] Update existing recipes

---

## Phase 6: XP and Gamification (0.5.x)

### 0.5.0 - XP System
- [ ] Define XP curve (non-linear)
- [ ] Track XP for Anna, Junior, Senior
- [ ] Award XP for correct answers
- [ ] Award XP for new recipes

### 0.5.1 - Level and Titles
- [ ] Define level 0-100 progression
- [ ] Create title list (nerdy, ASCII-friendly)
- [ ] Display level/title in status
- [ ] Display level/title in REPL welcome

---

## Phase 7: Self-Sufficiency (0.6.x)

### 0.6.0 - Ollama Auto-Setup
- [ ] Detect Ollama installation
- [ ] Install Ollama if missing
- [ ] Detect hardware capabilities
- [ ] Select appropriate models
- [ ] Download models with progress

### 0.6.1 - Auto-Update
- [ ] Check GitHub releases every 10 minutes
- [ ] Download new version
- [ ] Verify checksum
- [ ] Restart annad safely
- [ ] Record update state

### 0.6.2 - Helper Tracking
- [ ] Track Anna-installed packages
- [ ] Display helpers in status
- [ ] Remove helpers on uninstall

### 0.6.3 - Clean Uninstall
- [ ] Implement `annactl uninstall`
- [ ] List helpers for removal choice
- [ ] Remove services, data, models
- [ ] Clean permissions

### 0.6.4 - Factory Reset
- [ ] Implement `annactl reset`
- [ ] Delete recipes
- [ ] Remove helpers
- [ ] Reset DBs
- [ ] Keep binaries

---

## Phase 8: Proactive Monitoring (0.7.x)

### 0.7.0 - Trend Detection
- [ ] Track boot time trends
- [ ] Track performance metrics over time
- [ ] Detect regressions

### 0.7.1 - Correlation Engine
- [ ] Correlate degradation with recent changes
- [ ] Package install timeline
- [ ] Service state changes

### 0.7.2 - Anomaly Alerts
- [ ] Thermal anomalies
- [ ] Network instability
- [ ] Disk I/O regressions
- [ ] Service failures

---

## Phase 9: Production Polish (0.8.x - 0.9.x)

### 0.8.0 - UI Polish
- [ ] ASCII borders and formatting
- [ ] True color support
- [ ] Spinner indicators
- [ ] Streaming output

### 0.8.1 - Performance Optimization
- [ ] Minimize LLM prompt sizes
- [ ] Cache frequent queries
- [ ] Optimize snapshot reads

### 0.9.0 - Testing and Hardening
- [ ] Integration tests for all flows
- [ ] Error handling coverage
- [ ] Edge case testing

### 0.9.1 - Documentation
- [ ] Complete README
- [ ] Architecture docs
- [ ] User guide

---

## Milestone: v1.0.0 - Production Ready

All phases complete, tested, documented. Ready for production use.

---

## Notes

- Each task should be small and verifiable
- Preserve snapshot performance throughout
- Debug mode always on for now
- No emojis or icons in output
- Every completed task moves to RELEASE_NOTES.md
