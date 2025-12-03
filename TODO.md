# Anna Assistant - Implementation Roadmap

**Current Version: 0.0.48**

This roadmap migrates from the v7.42.5 snapshot-based architecture to the full natural language assistant while preserving performance.

---

## Phase 1: CLI Surface Lockdown (0.0.x)

### 0.0.48 - Learning System (COMPLETED)
- [x] Knowledge Pack v1 format with strict limits
  - [x] Local JSON at /var/lib/anna/knowledge_packs/installed/*.json
  - [x] Max 50 packs, 500 total recipes, 24KB per recipe
  - [x] Schema with pack_id, name, version, source, tags, entries
  - [x] LearnedRecipe structure with intent, targets, triggers, actions, rollback
- [x] Knowledge search tool with local retrieval
  - [x] learned_recipe_search(query, limit) tool
  - [x] Token-based scoring (no embeddings for v1)
  - [x] Returns SearchHit with recipe_id, title, score, evidence_id
  - [x] Evidence IDs: K1, K2 prefix for knowledge
- [x] Learning pipeline (case -> recipe conversion)
  - [x] LearningManager with storage/retrieval
  - [x] Auto-create monthly packs (learned-pack-YYYYMM)
  - [x] Recipe deduplication (increment wins instead of duplicate)
  - [x] Minimum reliability 90%, minimum evidence count 1
- [x] XP system with deterministic progression
  - [x] Non-linear XP curve (100→500→1200→2000→...)
  - [x] Level 0-100 with title progression
  - [x] Titles: Intern, Apprentice, Junior, Competent, Senior, Expert, Wizard, Grandmaster
  - [x] XP gains: +2 (85% reliability), +5 (90%), +10 (recipe created)
  - [x] learning_stats tool for XP/level display
- [x] Transcript and case file updates
  - [x] LearningRecord structure in transcript.rs
  - [x] knowledge_searched, knowledge_query, recipes_matched fields
  - [x] recipe_written, recipe_id, xp_gained, level_after fields
- [x] Integration tests (run_learning_tests())
  - [x] Learning stats retrieval
  - [x] Query timing comparison
  - [x] Knowledge search capability
  - [x] XP directory structure check
- [x] Updated version to 0.0.48

### 0.0.47 - First Mutation Flow (COMPLETED)
- [x] Append line mutation with full lifecycle
  - [x] append_line_mutation.rs module
  - [x] SandboxCheck for dev-safe paths (cwd, /tmp, $HOME)
  - [x] AppendMutationEvidence collection (stat, preview, hash, policy)
  - [x] AppendDiffPreview for showing changes before execution
  - [x] Risk levels: Sandbox (low/yes), Home (medium/I CONFIRM), System (blocked)
  - [x] execute_append_line with backup, verification, and rollback info
  - [x] execute_rollback by case_id
- [x] File evidence tools (4 new tools)
  - [x] file_stat - uid/gid, mode, size, mtime, exists
  - [x] file_preview - first N bytes with secrets redacted
  - [x] file_hash - SHA256 for before/after verification
  - [x] path_policy_check - allowed/blocked decision with evidence ID
- [x] Case files for every request
  - [x] User-readable copies in $HOME/.local/share/anna/cases/
  - [x] save_user_copy() method on CaseFile
- [x] Deep test mutation tests (run_mutation_tests())
  - [x] Test diff preview and confirmation requirement
  - [x] Test file unchanged without confirmation
  - [x] Test sandbox path policy
  - [x] Test blocked path policy
- [x] Updated version to 0.0.47

### 0.0.46 - Evidence Quality Release (COMPLETED)
- [x] Domain-specific evidence tools (10 new tools)
  - [x] uname_summary - kernel version and architecture
  - [x] mem_summary - memory total/available from /proc/meminfo
  - [x] mount_usage - disk space with root free/used
  - [x] nm_summary - NetworkManager status and connections
  - [x] ip_route_summary - routing table and default gateway
  - [x] link_state_summary - interface link states
  - [x] audio_services_summary - pipewire/wireplumber/pulseaudio status
  - [x] pactl_summary - PulseAudio/PipeWire sinks and sources
  - [x] boot_time_summary - uptime and boot timestamp
  - [x] recent_errors_summary - journal errors filtered by keyword
- [x] Domain routing in translator (route_to_domain_evidence())
- [x] Tool sanity gate (apply_tool_sanity_gate())
  - [x] Prevents generic hw_snapshot for domain-specific queries
  - [x] Auto-replaces with correct domain tools
- [x] Deep test evidence validation (run_evidence_tool_validation())
- [x] Updated deep test to v0.0.46

### 0.0.45 - Deep Test Harness + Correctness Fixes (COMPLETED)
- [x] Deep test harness (scripts/anna_deep_test.sh)
  - [x] Environment capture
  - [x] Translator stability tests (50 queries)
  - [x] Read-only correctness tests
  - [x] Doctor auto-trigger tests
  - [x] Policy gating tests
  - [x] Case file verification
  - [x] REPORT.md and report.json outputs
- [x] New evidence tools for correctness
  - [x] kernel_version - direct uname
  - [x] memory_info - direct /proc/meminfo
  - [x] network_status - interfaces, routes, NM status
  - [x] audio_status - pipewire/wireplumber
- [x] Enhanced disk_usage with explicit free space values
- [x] Version mismatch display in status (CLI vs daemon)
- [x] Table-driven doctor selection tests (25 phrases)
- [x] docs/TESTING.md documentation

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

### 0.0.17 - Multi-User Correctness (COMPLETED)
- [x] Target user selection with strict precedence: REPL session > SUDO_USER > invoking user > primary interactive
- [x] Safe home directory detection via /etc/passwd (never guess /home/<name>)
- [x] User-scoped file operations: write_file_as_user, backup_file_as_user, fix_file_ownership
- [x] UserHomePolicy in capabilities.toml for allowed/blocked subpaths
- [x] Default allowed: .config/**, .bashrc, .zshrc, .vimrc, .gitconfig, etc.
- [x] Default blocked: .ssh/**, .gnupg/**, .password-store/**, browser credentials
- [x] Clarification prompt for ambiguous user selection (multiple candidates)
- [x] Evidence ID citations for user selection (E-user-##### format)
- [x] Target user transcript message in pipeline
- [x] 15 unit tests for target user system
- [x] 10 unit tests for user home policy

### 0.0.18 - Secrets Hygiene (COMPLETED)
- [x] Centralized redaction module with 22 secret types (Password, ApiKey, BearerToken, PrivateKey, etc.)
- [x] Compiled regex patterns via LazyLock for performance
- [x] Pattern matching for: passwords, tokens, API keys, bearer tokens, private keys, PEM blocks, SSH keys, cookies, AWS/Azure/GCP credentials, git credentials, database URLs, connection strings
- [x] Evidence restriction policy for sensitive paths (~/.ssh/**, ~/.gnupg/**, /etc/shadow, /proc/*/environ, etc.)
- [x] Redaction format: [REDACTED:TYPE] with type-specific placeholders
- [x] Junior leak detection enforcement (rules 20-24, penalties for secret leaks)
- [x] Redaction integration in dialogue output, evidence summaries, and LLM prompts
- [x] check_for_leaks() function with penalty calculation
- [x] 22 unit tests for redaction patterns and path restrictions

### 0.0.19 - Offline Documentation Engine (COMPLETED)
- [x] Knowledge packs stored under /var/lib/anna/knowledge_packs/
- [x] Pack schema: id, name, source type, trust level, retention policy, timestamps
- [x] SQLite FTS5 index for fast full-text search
- [x] Evidence IDs for citations (K1, K2, K3...)
- [x] Default pack ingestion: man pages, /usr/share/doc, Anna project docs
- [x] knowledge_search(query, top_k) tool with excerpts and citations
- [x] knowledge_stats() tool for index information
- [x] [KNOWLEDGE] section in annactl status
- [x] Secrets hygiene applied to all excerpts
- [x] 10+ unit tests for knowledge pack system

### 0.0.20 - Ask Me Anything Mode (COMPLETED)
- [x] Source labeling for answers: [E#] for system evidence, [K#] for knowledge, (Reasoning) for inference
- [x] New tools: answer_context(), source_plan(), qa_stats()
- [x] Translator plans source mix by question type (how-to vs system status vs mixed)
- [x] Junior enforcement: penalize unlabeled factual claims
- [x] "I don't know" behavior: report missing evidence, suggest read-only tools
- [x] [Q&A TODAY] section in annactl status: answers count, avg reliability, top sources
- [x] QuestionType classification: HowTo, SystemStatus, Mixed, General
- [x] SourcePlan with primary_sources, knowledge_query, system_tools
- [x] 8+ unit tests for source labeling

### 0.0.21 - Performance and Latency Sprint (COMPLETED)
- [x] TTFO (Time to First Output) < 150ms with header and working indicator
- [x] Token budgets per role: translator.max_tokens=256, translator.max_ms=1500
- [x] Token budgets per role: junior.max_tokens=384, junior.max_ms=2500
- [x] Read-only tool result caching with TTL policy (5 min default)
- [x] LLM response caching keyed by request, evidence, policy, model versions
- [x] Cache storage in /var/lib/anna/internal/cache/
- [x] Performance statistics tracking (samples, latencies, hit rates)
- [x] [PERFORMANCE] section in annactl status (avg latency, cache hit rate, top tools)
- [x] BudgetSettings, PerformanceConfig in config.toml
- [x] 10+ unit tests for cache key determinism and budget validation

### 0.0.22 - Reliability Engineering (COMPLETED)
- [x] Metrics collection module with local JSON storage (metrics.json)
- [x] Track: request success/failure, tool success/failure, mutation rollbacks, LLM timeouts
- [x] Latency tracking with p50/p95 percentile calculations
- [x] Error budgets with configurable thresholds (1% request, 2% tool, 0.5% rollback, 3% timeout)
- [x] Budget burn rate calculation with warning (50%) and critical (80%) thresholds
- [x] BudgetState enum: Ok, Warning, Critical, Exhausted
- [x] self_diagnostics() read-only tool for comprehensive system health report
- [x] metrics_summary() tool for reliability metrics display
- [x] error_budgets() tool for budget status
- [x] DiagnosticsReport with evidence IDs per section
- [x] Sections: Version, Install Review, Update State, Model Readiness, Policy, Storage, Error Budgets, Recent Errors, Active Alerts
- [x] Redaction integration for error logs in diagnostics
- [x] [RELIABILITY] section in annactl status with budget alerts
- [x] ReliabilityConfig in config.toml
- [x] 14 unit tests for metrics, budgets, and diagnostics

### 0.0.23 - REPL Enhancement (Planned)
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
