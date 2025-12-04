//! Anna Common v0.0.68 - Two-Layer Transcript Renderer
//!
//! v0.0.68: Human Mode vs Debug Mode with full separation
//! - Human Mode (default): Natural IT department conversation
//!   - No evidence IDs ([E1]), no tool names, no raw commands
//!   - Shows topic-based evidence summaries
//!   - Readable, professional dialogue
//! - Debug Mode: Full fidelity for troubleshooting
//!   - Exact prompts, tool names, evidence IDs
//!   - Raw refs, parse warnings, timings
//! - TranscriptEventV2: Structured events with dual summaries
//! - TranscriptRole: Visible roles (departments) vs internal (translator/junior)
//! - Case files store BOTH human.log and debug.log
//! - validate_human_output() ensures no forbidden terms leak
//!
//! v0.0.67: Evidence-based department playbooks for Networking and Storage
//!
//! v0.0.63: Fix "obviously wrong answers" - strict topic-to-tool routing
//! - Deterministic topic detection routes to correct tools FIRST
//! - Strict answer validation: caps reliability at 40% for mismatched answers
//! - Memory/disk/kernel/network/audio questions must get matching evidence
//! - Topic configs define required tools and answer validation rules
//! - cap_reliability() enforces max score for wrong answer content
//!
//! v0.0.62: Human Mode vs Debug Mode (First-Class UX)
//! - Narrator component for human-readable IT department dialogue
//! - Human Mode (default): No tool names, evidence IDs, or raw prompts
//! - Debug Mode (--debug or ANNA_DEBUG=1): Full internal details
//! - IT department tone: Anna calm/competent, Translator brisk, Junior skeptical
//! - Evidence displayed by topic, not by ID
//! - Honest narration of fallbacks and retries
//!
//! v0.0.61: Evidence Topics for deterministic routing
//! - EvidenceTopic enum: CpuInfo, MemoryInfo, KernelVersion, DiskFree, etc.
//! - Pre-LLM topic detection: detect_topic() runs before translator
//! - Topic configs: required tools, fields, human labels
//! - Answer templates: generate_answer() for structured responses
//! - Topic validation: validate_evidence() for Junior verification
//! - Human labels: working_message() for transcript
//!
//! v0.0.60: Human-readable transcript mode (default)
//! - TranscriptMode enum: Human (default), Debug, Test
//! - ANNA_UI_TRANSCRIPT_MODE env var for override
//! - Human mode shows professional IT department dialogue without internals
//! - Debug/Test modes show tool names, evidence IDs, prompts, timings
//! - Human labels registry maps tools to plain-language descriptions
//! - Events always saved to debug log, filtered for human log
//!
//! v0.0.59: Auto-Case Opening + Departmental IT Org
//!
//! v0.0.37: Recipe Engine v1 (Reusable Fixes)
//! - RecipeStatus enum: Active, Draft, Archived
//! - PostCheck and PostCheckType for verification after recipe execution
//! - Recipe creation rules: >= 80% reliability = Active, < 80% = Draft
//! - Draft recipes never auto-suggested, can be promoted after validated run
//! - Recipe matching via intent_tags + keywords, ranking by confidence and success_count
//! - Recipe events tracked in case files (matched, executed, succeeded, failed, created, promoted)
//! - promote(), is_usable(), can_auto_suggest() methods on Recipe
//! - Updated RecipeStats with active_count field
//!
//! v0.0.23: Self-Sufficiency
//! - Auto-install Ollama if missing (daemon installs on bootstrap)
//! - Auto-pull models when needed (with progress tracking)
//! - Track installations as "installed by Anna" for clean uninstall
//! - Helper install functions: install_package, install_ollama
//! - Automatic service start after Ollama install
//!
//! v0.0.16: Preflight checks, dry-run diffs, post-checks, automatic rollback
//! - MutationState enum: planned -> preflight_ok -> confirmed -> applied -> verified_ok | rolled_back
//! - PreflightResult with checks for path, permissions, size, hash, backup
//! - DiffPreview with line-based diff for file edits
//! - PostCheckResult for verification after mutations
//! - SafeMutationExecutor with full lifecycle management
//! - Automatic rollback on post-check failure
//!
//! v0.0.15: Debug levels, unified formatting, enhanced status display
//! - UiConfig with debug_level (0=minimal, 1=normal, 2=full)
//! - Unified formatting module (colors, SectionFormatter, DialogueFormatter)
//! - Enhanced annactl status with 10 sections
//!
//! v0.0.14 - Policy Engine + Security Posture
//!
//! v7.42.0: Fix daemon running vs snapshot available confusion
//! - Control socket for authoritative daemon health (/run/anna/annad.sock)
//! - Canonical paths in daemon_state.rs used by BOTH annad and annactl
//! - Status snapshot at /var/lib/anna/internal/snapshots/status.json
//! - annactl status shows DAEMON (live check) + SNAPSHOT (file status) separately
//! - annactl doctor for diagnostics
//! - Schema versioning for forward compatibility
//!
//! v7.41.0: Snapshot-based architecture (daemon writes, annactl reads only)
//! - snapshots.rs: SwSnapshot, HwSnapshot structs
//! - snapshot_builder.rs: Daemon-only build functions
//! - annactl NEVER does heavyweight scanning - reads snapshots only
//! - Delta detection via pacman.log fingerprint, PATH dir fingerprints, systemd hashes
//! - p95 < 1.0s for sw command (snapshot read only)
//! - Compact output by default (--full/--json/--section flags)
//!
//! v7.40.0: Cache-first architecture for fast sw command
//! - sw_cache.rs: Persistent cache with delta detection
//! - Package delta via pacman.log offset/mtime
//! - Command delta via PATH directory mtimes
//! - Service delta via unit file mtimes
//! - p95 < 1.0s for sw command when cache warm
//! - Compact output modes (--full, --json, --section)
//!
//! v7.39.0: Domain-based incremental refresh, adaptive rendering
//! - Domain-based incremental refresh (hw.static, sw.packages, etc.)
//! - On-demand refresh with "checking..." indicator
//! - Terminal-adaptive rendering (compact/standard/wide)
//! - Daemon self-observation (CPU/RAM thresholds)
//!
//! v7.38.0: Cache-only status, no live probing
//! - status_snapshot.json: Daemon writes, annactl reads (no live probing)
//! - last_crash.json: Written on panic/fatal for debugging without journalctl
//! - last_start.json: Written on every start attempt
//! - Hardened startup with explicit permission and DB checks
//! - --version outputs exactly "vX.Y.Z" (no banners, no ANSI)
//! - Update scheduler writes real timestamps (never "never" or "not scheduled")
//! - Installer smoke check verifies daemon actually starts
//!
//! v7.37.0: Functional auto-update and auto-install
//! - Auto-update scheduler that actually runs and persists state
//! - Instrumentation engine that installs scoped tools on-demand
//! - Idle-aware scanning and installation (respects CPU/pacman lock)
//! - Explicit clean statements in all logs sections
//! - Correct installer version detection (binary --version precedence)
//! - Internal paths created on daemon start
//!
//! v7.36.0: Bounded Knowledge & Chunked Storage
//! - MAX_CHUNK_BYTES = 16,384 (16 KiB) per chunk
//! - MAX_DOC_BYTES = 512,000 (500 KiB) total per document
//! - Chunked storage with index for all large content
//! - Deterministic fact extraction (config paths, units, modules, packages)
//! - Bounded rendering with page budgets per command
//!
//! v7.35.1: Version detection and platform discovery
//! - Installer version detection with strict precedence (annad/annactl --version)
//! - Update checks that actually run and persist state
//! - Telemetry coverage rules (80% threshold per window)
//! - annactl hw usb with controllers and connected devices
//! - AVAILABLE QUERIES section in hw overview
//! - Steam/platform detection with local manifest parsing
//! - Network stability metrics (WiFi signal trends, disconnects)
//! - Driver/firmware info per hardware component
//! - Scoped dependency auto-installs (visible, logged, safe)
//! - No truncation - all output wraps to terminal width
//! - No HTML/wiki markup in output - clean text only
//!
//! v7.34.0: Update scheduler that actually runs and records checks
//! - Consolidated UpdateState in config.rs
//! - Real timestamps in annactl status
//! - Ops.log audit trail for update checks
//!
//! v7.33.0: No truncation, real updates, peripheral inventory, sensors
//! - Working auto-update scheduler with real timestamps
//! - Concrete telemetry readiness (samples, windows, freshness)
//! - Process identity from exe/cmdline/cgroup (no "Bun Pool" nonsense)
//! - USB/Thunderbolt/SD/Bluetooth inventory from /sys
//! - Sensors category (thermal, fan, battery from hwmon)
//! - AVAILABLE NAMES in hw overview
//! - Deterministic ordering across runs
//! - No truncation anywhere - full text wrapping
//!
//! v7.32.0: Evidence-based categorization, Steam/platform detection, WiFi trends
//! - Software categorization with evidence trail (desktop, pacman, man)
//! - Steam game detection from local appmanifest files
//! - Platform detection (Heroic, Lutris, Bottles) when present
//! - WiFi signal/link quality trends (1h/24h/7d/30d)
//! - On-demand scoped scans with time budget
//! - Staleness model for datasets
//! v7.31.0: Telemetry correctness and trend windows
//! - Concrete readiness model (collecting/ready for 1h/24h/7d/30d)
//! - Trend windows with proper availability checks
//! - Global percent formatting with correct ranges
//! - Fixed process identity (no more "Bun Pool" nonsense)
//! - Truthful auto-update status
//! v7.1.0: Real telemetry with SQLite storage
//! v7.5.0: Enhanced telemetry with CPU time, exec counts, hotspots
//! v7.6.0: Config maps, trends, Anna needs model
//! v7.6.1: Config hygiene - identity filtering, lean output, vim/nvim separation
//! v7.7.0: Precise per-window aggregation and compact display format
//! v7.16.0: Log history with multi-window patterns, service lifecycle tracking
//! v7.17.0: Network topology, storage mapping, config graph
//! v7.18.0: Change journal, boot timeline, error focus with pattern IDs
//! v7.19.0: Service topology, signal quality, topology hints, cross-references
//! v7.20.0: Telemetry trends, log atlas with pattern IDs, golden baselines
//! v7.21.0: Config atlas, topology maps, impact view
//! v7.22.0: Scenario lenses, self toolchain hygiene
//! v7.23.0: Time-anchored trends, boot snapshots, inventory drift, config provenance
//! v7.24.0: Relationship store (links.db), hotspots, stack packages
//! - Every number has a verifiable source
//! - No invented descriptions
//! - No hallucinated metrics
//! - Per-process CPU/memory tracking in SQLite
//! - Per-execution JSONL logs with window aggregation
//! - Config discovery from man pages, pacman, Arch Wiki
//! - Anna needs tracking for missing tools
//! - Service lifecycle: state, restarts, exit codes, activation failures
//! - Network topology: routes, DNS, interface management
//! - Storage: SMART/NVMe health, filesystem mounts
//! - Config graph: ownership, consumers, includes
//! - Change journal: package, service, config, kernel changes
//! - Boot timeline: per-boot health, failed units, slow starts
//! - Log patterns: stable IDs, novelty detection
//! - Service topology: requires/wants/wanted-by relationships (v7.19.0)
//! - Signal quality: WiFi dBm, storage SMART/NVMe health (v7.19.0)
//! - Topology hints: high-impact services, driver stacks (v7.19.0)
//! - Telemetry trends: deterministic trend labels (stable/higher/lower) (v7.20.0)
//! - Log atlas: pattern IDs, cross-boot visibility (v7.20.0)
//! - Golden baselines: baseline selection, new-since-baseline tagging (v7.20.0)
//! - Config atlas: clean per-component config discovery (v7.21.0)
//! - Topology maps: software stacks and hardware components (v7.21.0)
//! - Impact view: top resource consumers from telemetry (v7.21.0)
//!
//! Modules:
//! - grounded: Real data from real system commands
//! - atomic_write: Atomic file write operations
//! - boot_timeline: Per-boot health summary (v7.18.0+)
//! - change_journal: System change tracking (v7.18.0+)
//! - config: System configuration
//! - display_format: Output formatting utilities
//! - error_index: Log scanning and error aggregation
//! - intrusion: Security event detection
//! - knowledge_core: Object inventory and classification
//! - knowledge_collector: System discovery
//! - log_patterns_enhanced: Pattern IDs and novelty (v7.18.0+)
//! - needs: Anna's tool and doc dependencies (v7.6.0+)
//! - object_metadata: Static descriptions and relationships
//! - ops_log: Anna's internal operations audit trail (v7.12.0+)
//! - service_lifecycle: Systemd unit lifecycle tracking (v7.16.0+)
//! - service_state: Systemd service tracking
//! - telemetry: Process monitoring and usage tracking (log files)
//! - telemetry_db: SQLite-based telemetry storage (v7.1.0+)
//! - telemetry_exec: Per-object, per-day JSONL execution logs (v8.0.0+)

// v6.0.0: Grounded knowledge system - every fact has a source
pub mod grounded;

// Core modules
pub mod atomic_write;
pub mod boot_timeline;
pub mod change_journal;
pub mod config;
pub mod display_format;
pub mod error_index;
pub mod golden_baseline;
pub mod intrusion;
pub mod knowledge_collector;
pub mod knowledge_core;
pub mod log_atlas;
pub mod log_patterns_enhanced;
pub mod needs;
pub mod object_metadata;
pub mod ops_log;
pub mod service_lifecycle;
pub mod service_state;
pub mod telemetry;
pub mod telemetry_db;
pub mod telemetry_exec;
pub mod telemetry_trends;
// v7.21.0: Config atlas, topology maps, impact view
pub mod config_atlas;
pub mod impact_view;
pub mod topology_map;
// v7.22.0: Scenario lenses and self toolchain hygiene
pub mod scenario_lens;
pub mod sw_lens;
pub mod toolchain;
// v7.23.0: Time-anchored trends, boot snapshots, inventory drift, config provenance
pub mod boot_snapshot;
pub mod config_hygiene;
pub mod inventory_drift;
pub mod timeline;
// v7.24.0: Relationship store, hotspots, relationships
pub mod hotspots;
pub mod relationship_store;
pub mod relationships;
// v7.26.0: Instrumentation manifest and auto-install
pub mod auto_install;
pub mod instrumentation;
pub mod local_docs;
// v7.28.0: Text wrapping for zero truncation
pub mod text_wrap;
// v7.30.0: Evidence-based config locator
pub mod config_locator;
// v7.31.0: Telemetry format
pub mod telemetry_format;
// v7.32.0: Network trends and scoped scans
pub mod network_trends;
pub mod scoped_scan;
// v7.34.0: Update checking (uses config::UpdateState)
pub mod update_checker;
// v7.36.0: Bounded knowledge storage with chunking
pub mod chunk_store;
// v7.37.0: Idle detection and instrumentation state
pub mod idle;
pub mod instrumentation_state;
// v7.38.0: Daemon state (crash logging, status snapshots)
pub mod daemon_state;
// v7.39.0: Domain-based incremental refresh
pub mod domain_state;
// v7.39.0: Terminal-adaptive rendering
pub mod terminal;
// v7.39.0: Daemon self-observation
pub mod self_observation;

// v7.40.0: Cache-first software discovery
pub mod sw_cache;

// v7.41.0: Snapshot-based architecture (daemon writes, annactl reads only)
pub mod snapshot_builder;
pub mod snapshots;

// v7.42.0: Control socket for daemon/CLI contract
pub mod control_socket;

// v0.0.4: Ollama local LLM client for Junior verification
pub mod ollama;

// v0.0.5: Role-based model selection and benchmarking
pub mod model_selection;

// v0.0.7: Read-only tool catalog and executor
pub mod tool_executor;
pub mod tools;

// v0.0.8: Mutation tools, rollback, and executor
pub mod mutation_executor;
pub mod mutation_tools;
pub mod rollback;

// v0.0.47: Append line mutation with full rollback
pub mod append_line_mutation;

// v0.0.9: Helper tracking with provenance
pub mod helpers;

// v0.0.10: Install state and installer review
pub mod install_state;
pub mod installer_review;

// v0.0.11: Safe auto-update system
pub mod update_system;

// v0.0.73: Reliable auto-update with state machine, locking, and rollback
pub mod updater;

// v0.0.12: Proactive anomaly detection
pub mod anomaly_engine;

// v0.0.13: Conversation memory and recipe system
pub mod introspection;
pub mod memory;
pub mod recipes;

// v0.0.14: Policy engine and security posture
pub mod audit_log;
pub mod policy;

// v0.0.16: Mutation safety system
pub mod mutation_safety;

// v0.0.17: Target user and multi-user correctness
pub mod target_user;

// v0.0.18: Secrets hygiene and redaction
pub mod redaction;

// v0.0.19: Offline Documentation Engine (Knowledge Packs)
pub mod knowledge_packs;

// v0.0.20: Ask Me Anything Mode (Source-Labeled Answers)
pub mod source_labels;

// v0.0.21: Performance and Latency Sprint
pub mod performance;

// v0.0.22: Reliability Engineering
pub mod reliability;

// v0.0.33: Human-first transcript and case files
pub mod transcript;

// v0.0.34: Fix-It Mode for bounded troubleshooting loops
pub mod fixit;

// v0.0.35: Model policy and readiness
pub mod model_policy;

// v0.0.38: Arch Networking Doctor
pub mod networking_doctor;

// v0.0.39: Arch Storage Doctor (BTRFS Focus)
pub mod storage_doctor;

// v0.0.40: Arch Audio Doctor (PipeWire Focus)
pub mod audio_doctor;

// v0.0.41: Arch Boot Doctor (Slow Boot + Service Regressions)
pub mod boot_doctor;

// v0.0.42: Arch GPU/Graphics Doctor (Wayland/X11, Drivers, Compositor Health)
pub mod graphics_doctor;

// v0.0.43: Doctor Registry + Unified Entry Flow
pub mod doctor_registry;

// v0.0.48: Learning System (Knowledge Packs + XP)
pub mod learning;

// v0.0.49: Doctor Lifecycle System (unified interface + evidence-based diagnosis)
pub mod doctor_lifecycle;
pub mod doctor_network_tools;
pub mod networking_doctor_v2;

// v0.0.50: User File Mutation (append_line, set_key_value with rollback)
pub mod user_file_mutation;

// v0.0.50: File Edit Tool Executors
pub mod file_edit_tools;

// v0.0.51: Systemd Action Engine (modular)
pub mod systemd_action;
pub mod systemd_apply;
pub mod systemd_probe;
pub mod systemd_rollback;

// v0.0.51: Systemd Tool Executors
pub mod systemd_tools;

// v0.0.52: System Query Router
pub mod system_query_router;

// v0.0.53: Doctor Flow (orchestrates diagnostic flows)
pub mod doctor_flow;

// v0.0.54: Action Engine (safe mutations with diffs, rollback, confirmations)
pub mod action_engine;
pub mod action_executor;
pub mod action_risk;

// v0.0.55: Deterministic Case Engine + Doctor-first routing
pub mod case_engine;
pub mod case_file_v1;
pub mod evidence_tools;
pub mod intent_taxonomy;
pub mod recipe_extractor;
pub mod transcript_render;

// v0.0.56: Fly-on-the-Wall Dialogue Layer
#[cfg(test)]
mod dialogue_golden_tests;
pub mod dialogue_renderer;

// v0.0.57: Evidence Coverage + Correct Tool Routing
pub mod evidence_coverage;
#[cfg(test)]
mod evidence_coverage_tests;
pub mod junior_rubric;

// v0.0.58: Proactive Monitoring Loop v1
pub mod alert_detectors;
pub mod alert_probes;
pub mod proactive_alerts;

// v0.0.59: Auto-Case Opening + Departmental IT Org
pub mod case_lifecycle;
#[cfg(test)]
mod case_lifecycle_tests;
pub mod service_desk;
pub mod transcript_v2;

// v0.0.60: 3-Tier Transcript Rendering
pub mod human_labels;
pub mod transcript_events;
#[cfg(test)]
mod transcript_mode_tests;
pub mod transcript_renderer;

// v0.0.61: Evidence Topics (Targeted Answers)
pub mod evidence_topic;

// v0.0.62: Human Mode vs Debug Mode (Narrator)
pub mod narrator;

// v0.0.65: Typed Evidence System + Answer Shaping
pub mod answer_shaper;
pub mod evidence_record;
pub mod evidence_router;

// v0.0.74: Direct Answer Generator
pub mod direct_answer;
#[cfg(test)]
mod direct_answer_tests;
#[cfg(test)]
mod narrator_tests;

// v0.0.66: Service Desk Dispatcher + Department Protocol
pub mod department_protocol;

// v0.0.67: Department Evidence Playbooks v1
pub mod action_proposal;
pub mod evidence_playbook;
pub mod networking_playbook;
pub mod storage_playbook;

// v0.0.68: Two-Layer Transcript Renderer
pub mod transcript_v068;

// v0.0.69: Service Desk Case Coordinator
pub mod case_coordinator;

// v0.0.70: Dual Transcript Renderer
pub mod transcript_v070;

// v0.0.71: Humanizer Layer
pub mod humanizer;

// v0.0.72: Unified Dual Mode Transcript
pub mod transcript_v072;

// v0.0.75: Enhanced Recipe Engine + RPG Stats + Transcript v075
#[cfg(test)]
mod learning_stats_tests;
pub mod recipe_engine;
pub mod rpg_stats;
pub mod transcript_v075;

// v0.0.80: Mutation Engine v1 (safe system mutations)
pub mod config_mutation;
pub mod mutation_engine_v1;
pub mod mutation_orchestrator;
pub mod mutation_transcript;
#[cfg(test)]
mod mutation_tests_v080;
pub mod package_mutation;
pub mod privilege;
pub mod service_mutation;

// v0.0.81: Real verification for mutations
pub mod mutation_verification;

// v0.0.81: Reliability gate for mutation safety
pub mod reliability_gate;

// v0.0.81: Unified transcript configuration
pub mod transcript_config;

// v0.0.81: Case file schema v0.0.81
pub mod case_file_v081;

// v0.0.81: Rollback executor
pub mod rollback_executor;

// v0.0.81: Status enhancements
pub mod status_v081;

// v0.0.82: Pre-router for deterministic routing before translator
pub mod pre_router;

// v0.0.82: Translator JSON schema + robust parser
pub mod translator_v082;

// v0.0.82: Debug mode toggle via natural language
pub mod debug_toggle;

// v0.0.82: Pipeline integration with pre-router + direct handlers
pub mod pipeline_v082;

// v0.0.82: Golden tests for normal mode clean output
#[cfg(test)]
mod golden_tests_v082;

// Re-exports for convenience
pub use atomic_write::{atomic_write, atomic_write_bytes};
pub use config::*;
pub use display_format::*;
pub use error_index::*;
pub use intrusion::*;
pub use knowledge_collector::*;
pub use knowledge_core::*;
// Alias for backward compatibility
pub use knowledge_core::Category as KnowledgeCategory;
pub use object_metadata::*;
pub use service_state::*;
// Explicit telemetry exports to avoid conflicts with knowledge_core::TelemetryAggregates
pub use telemetry::{
    command_stats,
    days_ago,
    // Time helpers
    hours_ago,
    now,
    top_commands,
    CommandEvent,
    // Stats
    CommandStats,
    PackageChangeEvent,
    PackageChangeType,
    // Event types
    ProcessSample,
    ServiceChangeEvent,
    TelemetryReader,
    // State
    TelemetryState,
    // Writer/Reader
    TelemetryWriter,
    COMMAND_USAGE_LOG,
    ERROR_EVENTS_LOG,
    PACKAGE_CHANGES_LOG,
    PROCESS_ACTIVITY_LOG,
    SERVICE_CHANGES_LOG,
    // Constants
    TELEMETRY_DIR,
    TELEMETRY_STATE_FILE,
};
// v7.2.0: SQLite telemetry database exports (with aggregations)
// v7.5.0: Enhanced with CPU time, exec counts, hotspots
// v7.6.0: Added MaintenanceResult for pruning
// v7.7.0: Added compact per-window stats (AllWindowStats, WindowStats, TopCompactEntry)
// v7.9.0: Added trend classification (24h vs 7d), TopIdentityWithTrend, TrendWithStats
// v7.27.0: Added boot_id for "this boot" aggregations
pub use telemetry_db::{
    format_bytes_human,
    format_cpu_time,
    format_cpu_time_compact,
    // v7.27.0: Boot ID support
    get_current_boot_id,
    AllWindowStats,
    DataStatus,
    EnhancedUsageStats,
    EnhancedWindowedStats,
    GlobalPeak,
    HealthHotspot,
    MaintenanceResult,
    ObjectTelemetry,
    ProcessTelemetrySample,
    SampleCounts,
    TelemetryDb,
    TelemetryHealth,
    TelemetryStats,
    TopCompactEntry,
    TopHighlightEntry,
    TopIdentityWithTrend,
    TopProcessEntry,
    // v7.7.0: Trend and window status types
    Trend,
    TrendData,
    // v7.9.0: Enhanced trend types
    TrendWithStats,
    UsageStats,
    WindowStats,
    WindowStatusInfo,
    TELEMETRY_DB_PATH,
    WINDOW_1H,
    WINDOW_24H,
    WINDOW_30D,
    WINDOW_7D,
};
// v8.0.0: Execution telemetry with per-object, per-day JSONL storage
pub use telemetry_exec::{
    ExecTelemetryReader, ExecTelemetryWriter, ExecutionRecord, ObjectTelemetryResult,
    WindowStats as ExecWindowStats, EXEC_TELEMETRY_DIR,
};
// v7.6.0: Anna needs model for missing tools and docs
pub use needs::{
    get_tool_status, is_ethtool_available, is_iw_available, is_man_available,
    is_nvidia_smi_available, is_nvme_available, is_sensors_available, is_smartctl_available,
    AnnaNeeds, HardwareDeps, Need, NeedScope, NeedStatus, NeedType, NeedsSummary,
};
// v7.12.0: Operations log for Anna's internal tooling audit trail
pub use ops_log::{
    OpsAction, OpsActionCounts, OpsEntry, OpsLogReader, OpsLogSummary, OpsLogWriter, INTERNAL_DIR,
    OPS_LOG_FILE,
};
// v7.16.0: Service lifecycle tracking
pub use service_lifecycle::{
    find_hardware_related_units, find_related_units, ServiceLifecycle, ServiceLifecycleSummary,
};
// v7.18.0: Change journal for tracking system changes
pub use change_journal::{
    get_config_history, get_package_history, get_recent_changes, scan_pacman_log, ChangeDetails,
    ChangeEvent, ChangeJournalReader, ChangeJournalWriter, ChangeType, JOURNAL_DIR, JOURNAL_FILE,
};
// v7.18.0: Boot timeline for per-boot health view
pub use boot_timeline::{
    get_boot_list, get_boot_summary, get_current_boot_summary, get_previous_boot_summary,
    get_service_log_patterns_by_boot, BootPhase, BootSummary, LogPatternEntry, SlowUnit,
    BOOT_TIMELINE_DIR,
};
// v7.18.0: Enhanced log patterns with pattern IDs and novelty
pub use log_patterns_enhanced::{
    get_service_log_counts, LogPattern, LogPatternAnalyzer, PatternOccurrence,
    ServicePatternSummary, LOG_PATTERNS_DIR,
};
// v7.20.0: Telemetry trends with deterministic labels
pub use telemetry_trends::{
    format_bytes_short, get_process_trends, HardwareTrends, ProcessTrends, SignalTrends,
    TrendDirection, WindowStats as TrendWindowStats,
};
// v7.20.0: Log atlas with pattern IDs and cross-boot visibility
pub use log_atlas::{
    format_timestamp_short, get_device_log_atlas, get_service_log_atlas, normalize_message,
    BootLogEntry, ComponentAtlas, CrossBootLogSummary, LogPattern as AtlasLogPattern, BASELINE_DIR,
    JOURNAL_DIR as ATLAS_JOURNAL_DIR,
};
// v7.20.0: Golden baseline for pattern comparison
pub use golden_baseline::{
    find_or_create_device_baseline, find_or_create_service_baseline,
    get_components_with_new_patterns, tag_pattern, BaselineTag, GoldenBaseline,
    MAX_BASELINE_WARNINGS,
};
// v7.21.0: Config atlas for clean per-component config discovery
pub use config_atlas::{
    build_config_atlas, ConfigAtlas, ConfigCategory, ConfigEntry, ConfigStatus, PrecedenceEntry,
};
// v7.21.0: Topology maps for software and hardware stacks
pub use topology_map::{
    build_hardware_topology, build_software_topology, AudioInfo, CpuInfo, GpuInfo,
    HardwareTopology, MemoryInfo, NetworkInfo, ServiceGroup, SoftwareTopology, StackRole,
    StorageInfo,
};
// v7.21.0: Impact view for resource consumer rankings
pub use impact_view::{
    format_bytes as impact_format_bytes, format_bytes_compact, get_hardware_impact,
    get_software_impact, ConsumerEntry, DiskPressure, HardwareImpact, NetworkUsage, SoftwareImpact,
};
// v7.22.0: Scenario lenses for category-aware views
pub use scenario_lens::{
    format_bytes as lens_format_bytes, AudioDevice, AudioLens, DisplayConnector, GpuDevice,
    GraphicsLens, NetworkEvent, NetworkInterface, NetworkLens, NetworkTelemetry, StorageDevice,
    StorageHealth, StorageLens, StorageTelemetry,
};
// v7.22.0: Software lenses for category views
pub use sw_lens::{
    get_sw_category, is_sw_category, AudioSwLens, ConfigFileEntry, DisplaySwLens, NetworkSwLens,
    PowerSwLens, ServiceEntry, ServiceTelemetry,
};
// v7.22.0: Self toolchain hygiene
pub use toolchain::{
    check_toolchain, ensure_tool, format_toolchain_section, format_toolchain_status_section,
    get_anna_tools, install_tool, AnnaTool, InstallResult, ToolCategory, ToolStatus,
    ToolchainStatus, ToolchainSummary,
};
// v7.23.0: Time-anchored trends
pub use timeline::{
    format_cpu_percent_with_range, format_fraction_as_percent, format_hw_telemetry_section,
    format_io_bytes, format_memory as timeline_format_memory, format_percent, format_temperature,
    format_usage_section, get_hw_telemetry_trends, get_logical_cores, get_usage_trends,
    HwTelemetryTrends, TimeWindow, TrendLabel, UsageTrends,
};
// v7.23.0: Boot snapshots
pub use boot_snapshot::{format_boot_snapshot_section, BootSnapshot, IncidentPattern};
// v7.23.0: Inventory drift
pub use inventory_drift::{DriftSummary, InventorySnapshot};
// v7.23.0: Config hygiene with provenance
pub use config_hygiene::{
    format_config_graph_section, format_config_section, ConfigGraph, ConfigPrecedenceEntry,
    ConfigSource, ValidatedConfig, ValidatedConfigEntry,
};
// v7.24.0: Relationship store
pub use relationship_store::{
    discover_device_driver_links, discover_driver_firmware_links, discover_package_service_links,
    discover_service_process_links, Link, LinkType, RelationshipStore,
};
// v7.24.0: Hotspots (v7.28.0: added NetworkHotspot)
pub use hotspots::{
    format_hardware_hotspots_section,
    format_software_hotspots_section,
    format_status_hotspots_section,
    get_hardware_hotspots,
    get_software_hotspots,
    CpuHotspot,
    GpuHotspot,
    HardwareHotspots,
    IoHotspot,
    LoadHotspot,
    MemoryHotspot,
    NetworkHotspot, // v7.28.0
    SoftwareHotspots,
    StartFrequencyHotspot,
    TempHotspot,
};
// v7.24.0: Relationships
pub use relationships::{
    format_hardware_relationships_section, format_software_relationships_section,
    get_hardware_relationships, get_software_relationships, DriverRelation, FirmwareRelation,
    HardwareRelation, HardwareRelationships, ProcessRelation, ServiceRelation, ServiceUsingDevice,
    SoftwareRelationships, SoftwareUsingDevice, StackPackage,
};
// v7.26.0: Instrumentation manifest and auto-install
pub use auto_install::{
    ensure_tool_for_command,
    get_instrumentation_status,
    is_package_installed as auto_is_package_installed,
    try_install,
    try_install_known_tool,
    InstallDisclosure,
    InstallResult as AutoInstallResult,
    InstrumentationStatus,
    // v7.28.0: In-band disclosure for auto-install
    PendingInstall,
    COMMON_TOOLS,
};
pub use instrumentation::{
    get_known_tools, get_missing_tools, get_package_version, is_package_installed, AvailableTool,
    InstallAttempt, InstalledTool, InstrumentationManifest, INSTRUMENTATION_FILE,
};
pub use local_docs::{
    get_config_paths_from_pacman, get_doc_paths, get_local_docs_summary, get_man_description,
    get_man_path, get_sample_configs_from_pacman, has_man_page, resolve_local_docs, LocalDocResult,
    LocalDocsSummary,
};
// v7.28.0: Text wrapping for zero truncation
pub use text_wrap::{format_kv, format_list_item, get_terminal_width, wrap_text, wrap_with_prefix};
// v7.31.0: Telemetry format and update state
pub use telemetry_format::{
    format_cpu_avg_peak,
    format_cpu_percent,
    format_cpu_percent_short,
    format_cpu_time as fmt_cpu_time,
    format_duration_short,
    format_memory as fmt_memory,
    format_memory_avg_peak,
    get_logical_cpu_count,
    TelemetryReadiness,
    TrendDelta,
    TrendDirection as TelemetryTrendDirection, // Alias to avoid conflict
    WindowAvailability,
    MIN_SAMPLES_1H,
    MIN_SAMPLES_24H,
    MIN_SAMPLES_30D,
    MIN_SAMPLES_7D,
};
// v7.34.0: Update checking with consolidated state
pub use ops_log::OpsLog;
pub use update_checker::{
    check_anna_updates, is_check_due, is_daemon_running, run_update_check, CheckResult,
};
// v7.32.0: Network trends and scoped scans
pub use network_trends::{
    collect_ethernet_sample, collect_wifi_sample, detect_interface_type, format_link_quality,
    format_rssi, is_ethtool_available as network_is_ethtool_available,
    is_iw_available as network_is_iw_available, is_nmcli_available, list_network_interfaces,
    EthernetSample, InterfaceTrends, InterfaceType, NetworkTrendWindow, WiFiSample,
};
pub use scoped_scan::{
    InterfaceInfo, MountInfo, ScanData, ScanResult, ScanScope, ScopedScanner, StalenessInfo,
    TempSensor, DEFAULT_TIME_BUDGET_MS, MAX_TIME_BUDGET_MS,
};
// v7.36.0: Bounded knowledge storage with chunking
pub use chunk_store::{
    delete_document,
    read_chunks,
    read_facts,
    render_bounded,
    sanitize_to_plain_text,
    // Operations
    store_document,
    DocEntry,
    DocIndex,
    // Types
    DocType,
    ExtractedFacts,
    FactWithSource,
    LocationHint,
    LocationScope,
    OverflowInfo,
    BUDGET_DETAIL,
    BUDGET_OVERVIEW,
    // Rendering budgets
    BUDGET_STATUS,
    CHUNK_STORE_PATH,
    MAX_CHUNKS_PER_DOC,
    // Hard limits
    MAX_CHUNK_BYTES,
    MAX_DOC_BYTES,
};
// v7.32.0: Evidence-based categorization and game platform detection
pub use grounded::{
    classify_software,
    detect_all_platform_games,
    detect_bottles_games,
    detect_heroic_games,
    detect_lutris_games,
    detect_steam_games,
    detect_steam_libraries,
    find_steam_game,
    format_game_size,
    get_audio_summary,
    get_bluetooth_summary,
    get_camera_summary,
    get_hardware_overview,
    get_input_summary,
    get_platforms_summary,
    get_sdcard_summary,
    get_steam_games_count,
    get_steam_root,
    get_thunderbolt_summary,
    get_usb_summary,
    is_steam_installed,
    AudioCard,
    AudioSummary,
    BluetoothAdapter,
    BluetoothState,
    BluetoothSummary,
    CameraDevice,
    CameraSummary,
    CategoryAssignment,
    // Category evidence
    Confidence,
    EvidenceSource,
    FirewireController,
    FirewireSummary,
    HardwareOverview,
    InputDevice,
    InputSummary,
    InputType,
    // v7.25.0+: Peripherals (USB, Bluetooth, Thunderbolt, SD, audio, input)
    PeripheralUsbDevice,
    // Game platforms
    Platform,
    PlatformGame,
    SdCardReader,
    SdCardSummary,
    // Steam detection
    SteamGame,
    SteamLibrary,
    ThunderboltController,
    ThunderboltDevice,
    ThunderboltSummary,
    UsbController,
    UsbSummary,
};
// v7.39.0: Domain-based incremental refresh
pub use domain_state::{
    cleanup_old_requests, Domain, DomainRefreshState, DomainSummary, RefreshRequest,
    RefreshResponse, RefreshResult, DOMAIN_STATE_DIR, DOMAIN_STATE_SCHEMA_VERSION, REQUESTS_DIR,
    RESPONSES_DIR,
};
// v7.39.0: Terminal-adaptive rendering
pub use terminal::{
    format_compact_line, format_with_overflow, get_terminal_size, truncate,
    wrap_text as terminal_wrap_text, DisplayMode, SimpleTable, COMPACT_HEIGHT_THRESHOLD,
    COMPACT_WIDTH_THRESHOLD, MIN_WIDTH, WIDE_WIDTH_THRESHOLD,
};
// v7.39.0: Daemon self-observation
pub use self_observation::{
    SelfObservation, SelfSample, SelfWarning, WarningKind, CPU_WINDOW_SECONDS,
    DEFAULT_CPU_THRESHOLD, DEFAULT_RSS_THRESHOLD_BYTES, SELF_SAMPLE_INTERVAL_SECS,
};
// v0.0.4/v0.0.5: Ollama local LLM client
pub use ollama::{
    get_ollama_version, is_ollama_installed, select_junior_model, GenerateOptions, GenerateRequest,
    GenerateResponse, OllamaClient, OllamaError, OllamaModel, OllamaStatus, PullProgress,
    PullRequest, GENERATE_TIMEOUT_MS, HEALTH_CHECK_TIMEOUT_MS, OLLAMA_DEFAULT_URL,
};
// v0.0.5: Role-based model selection and benchmarking
pub use model_selection::{
    default_candidates, junior_benchmark_cases, run_benchmark, select_model_for_role,
    translator_benchmark_cases, BenchmarkCase, BenchmarkResults, BootstrapPhase, BootstrapState,
    CaseResult, DownloadProgress, HardwareProfile, HardwareTier, LlmRole, ModelCandidate,
    ModelSelection,
};
// v0.0.7: Read-only tool catalog and executor
pub use tool_executor::{execute_tool, execute_tool_plan};
pub use tools::{
    parse_tool_plan, unavailable_result, unknown_tool_result, EvidenceCollector, LatencyHint,
    ToolCatalog, ToolDef, ToolPlan, ToolRequest, ToolResult, ToolSecurity,
};
// v0.0.8: Mutation tools, rollback, and executor
pub use mutation_executor::{
    create_file_edit_request, create_package_install_request, create_package_remove_request,
    create_systemd_request, execute_mutation, execute_mutation_plan, generate_request_id,
};
pub use mutation_tools::{
    get_service_state, is_path_allowed, validate_confirmation, validate_mutation_path,
    validate_mutation_request, FileEditOp, MutationError, MutationPlan, MutationRequest,
    MutationResult, MutationRisk, MutationToolCatalog, MutationToolDef, RollbackInfo, ServiceState,
    MAX_EDIT_FILE_SIZE, MEDIUM_RISK_CONFIRMATION,
};
pub use rollback::{
    MutationDetails, MutationLogEntry, MutationType, RollbackManager, ROLLBACK_BASE_DIR,
    ROLLBACK_FILES_DIR, ROLLBACK_LOGS_DIR,
};
// v0.0.16: Mutation safety system exports
pub use mutation_safety::{
    generate_request_id as safety_generate_request_id, DiffLine, DiffPreview, MutationState,
    PostCheck, PostCheckResult, PreflightCheck, PreflightResult, RollbackResult,
    SafeMutationExecutor,
};

// v0.0.47: Append line mutation exports
pub use append_line_mutation::{
    check_mutation_allowed, check_sandbox, collect_evidence as collect_append_evidence,
    execute_append_line, execute_rollback as execute_append_rollback, generate_diff_preview,
    generate_mutation_case_id, AppendDiffPreview, AppendMutationEvidence, AppendMutationResult,
    FilePreviewEvidence, FileStatEvidence, RiskLevel as AppendRiskLevel,
    RollbackResult as AppendRollbackResult, SandboxCheck, HOME_CONFIRMATION, SANDBOX_CONFIRMATION,
};

// v0.0.9: Helper tracking exports
// v0.0.23: Added install functions (install_package, install_ollama)
pub use helpers::{
    get_helper_definitions,
    get_helper_status_list,
    get_helpers_summary,
    get_ollama_version as helpers_get_ollama_version,
    get_package_version as helpers_get_package_version,
    // v0.0.30: Install all missing helpers on daemon start
    install_missing_helpers,
    install_ollama,
    install_package,
    is_command_available,
    is_package_present,
    refresh_helper_states,
    HelperDefinition,
    HelperState,
    HelperStatusEntry,
    HelpersManifest,
    HelpersSummary,
    InstallResult as HelperInstallResult,
    InstalledBy,
    HELPERS_STATE_FILE,
};

// v0.0.10: Install state and installer review exports
// v0.0.25: Added InstallState::ensure_initialized for auto-creation on daemon start
pub use install_state::{
    discover_install_state, BinaryInfo, DirectoryInfo, InstallState, LastReview, ReviewResult,
    UnitInfo, INSTALL_STATE_PATH, INSTALL_STATE_SCHEMA,
};
pub use installer_review::{
    run_and_record_review, run_installer_review, CheckResult as InstallerCheckResult,
    InstallerReviewReport,
};

// v0.0.11: Update system exports
// v0.0.26: Added perform_auto_update for full auto-update in daemon
pub use update_system::{
    generate_update_evidence_id, handle_post_restart, is_newer_version, perform_auto_update,
    BackupEntry, GuardrailResult, IntegrityStatus, ReleaseArtifact, ReleaseInfo,
    UpdateChannel as UpdateSystemChannel, UpdateManager, UpdateMarker, UpdatePhase,
    MIN_DISK_SPACE_BYTES, UPDATE_BACKUP_DIR, UPDATE_MARKER_FILE, UPDATE_STAGE_DIR,
};

// v0.0.12: Anomaly detection exports
pub use anomaly_engine::{
    analyze_slowness,
    what_changed,
    AlertQueue,
    Anomaly,
    AnomalyEngine,
    AnomalySeverity,
    AnomalySignal,
    AnomalyThresholds,
    ConfigChange as AnomalyConfigChange,
    PackageChange,
    SlownessAnalysisResult,
    // Slowness analysis tool
    SlownessHypothesis,
    TimeWindow as AnomalyTimeWindow,
    // What changed tool
    WhatChangedResult,
    ALERTS_FILE,
    ALERTS_SCHEMA_VERSION,
};

// v0.0.13: Memory and recipe exports
pub use introspection::{
    detect_introspection_intent, execute_forget, execute_introspection, IntrospectionIntent,
    IntrospectionItem, IntrospectionItemType, IntrospectionResult, FORGET_CONFIRMATION,
};
pub use memory::{
    generate_memory_evidence_id, MemoryIndex, MemoryManager, MemoryStats, RecipeAction,
    SessionRecord, SessionType, ToolUsage, TranslatorSummary, MEMORY_ARCHIVE_DIR, MEMORY_DIR,
    MEMORY_EVIDENCE_PREFIX, MEMORY_INDEX_FILE, MEMORY_SCHEMA_VERSION, SESSIONS_FILE,
};
pub use recipes::{
    generate_recipe_id,
    ArchivedRecipe,
    IntentPattern,
    PostCheck as RecipePostCheck,
    PostCheckType as RecipePostCheckType,
    Precondition,
    PreconditionCheck,
    Recipe,
    RecipeCreator,
    RecipeIndex,
    RecipeManager,
    RecipeRiskLevel,
    RecipeSafety,
    RecipeStats,
    // v0.0.37: New recipe types
    RecipeStatus,
    RecipeToolPlan,
    RecipeToolStep,
    RollbackTemplate,
    MIN_RELIABILITY_FOR_RECIPE,
    RECIPES_DIR,
    RECIPE_ARCHIVE_DIR,
    RECIPE_EVIDENCE_PREFIX,
    RECIPE_INDEX_FILE,
    RECIPE_SCHEMA_VERSION,
};

// v0.0.14: Policy engine exports
// v0.0.23: Added ensure_policy_defaults for auto-creation on first run
pub use audit_log::{
    redact_env_secrets, sanitize_for_audit, AuditEntry, AuditEntryType, AuditLogger, AuditResult,
    AUDIT_ARCHIVE_DIR, AUDIT_DIR, AUDIT_LOG_FILE,
};
pub use policy::{
    clear_policy_cache, ensure_policy_defaults, generate_policy_evidence_id, get_policy,
    reload_policy, BlockedPolicy, CapabilitiesPolicy, FileEditPolicy, HelpersPolicy, PackagePolicy,
    Policy, PolicyCheckResult, PolicyError, PolicyValidation, RiskPolicy, SystemdPolicy,
    BLOCKED_FILE, CAPABILITIES_FILE, HELPERS_FILE, POLICY_DIR, POLICY_EVIDENCE_PREFIX,
    POLICY_SCHEMA_VERSION, RISK_FILE,
};

// v0.0.17: Target user and multi-user correctness exports
pub use policy::UserHomePolicy;
pub use target_user::{
    backup_file_as_user,
    check_file_ownership,
    contract_home_path,
    create_dir_as_user,
    expand_home_path,
    fix_file_ownership,
    generate_user_evidence_id,
    get_path_relative_to_home,
    // Home directory functions
    get_user_home,
    // Policy helpers
    is_home_path_allowed,
    is_path_in_user_home,
    write_file_as_user,
    AmbiguousUserSelection,
    SelectionResult,
    TargetUserSelection,
    TargetUserSelector,
    // User info
    UserInfo,
    // User-scoped operations
    UserScopeError,
    UserSelectionSource,
    DEFAULT_ALLOWED_HOME_PATHS,
    DEFAULT_BLOCKED_HOME_PATHS,
    // Evidence
    USER_EVIDENCE_PREFIX,
};

// v0.0.18: Secrets hygiene and redaction exports
pub use redaction::{
    // Junior verification
    check_for_leaks,
    contains_secrets,
    detect_secret_types,
    generate_redaction_id,
    get_restriction_message,
    // Path restriction
    is_path_restricted,
    // Main redaction functions
    redact,
    redact_audit_details,
    redact_env_map,
    // Environment variable redaction
    redact_env_value,
    redact_evidence,
    redact_memory_content,
    redact_secrets,
    // Context-specific redaction
    redact_transcript,
    LeakCheckResult,
    RedactionResult,
    // Types
    SecretType,
    // Evidence
    REDACTION_EVIDENCE_PREFIX,
    RESTRICTED_EVIDENCE_PATHS,
};

// v0.0.19: Knowledge Packs exports
pub use knowledge_packs::{
    // Evidence
    generate_knowledge_evidence_id,
    // Ingestion
    ingest_manpages,
    ingest_package_docs,
    ingest_project_docs,
    ingest_user_note,
    KnowledgeDocument,
    // Storage
    KnowledgeIndex,
    KnowledgePack,
    KnowledgeStats,
    // Types
    PackSource,
    RetentionPolicy,
    SearchResult,
    TrustLevel,
    DEFAULT_TOP_K,
    KNOWLEDGE_EVIDENCE_PREFIX,
    KNOWLEDGE_INDEX_PATH,
    // Constants
    KNOWLEDGE_PACKS_DIR,
    MAX_EXCERPT_LENGTH,
};

// v0.0.20: Source labeling for Ask Me Anything mode
pub use source_labels::{
    // Functions
    classify_question_type,
    count_citations,
    detect_source_types,
    has_proper_source_labels,
    AnswerContext,
    MissingEvidenceReport,
    QaStats,
    QuestionType,
    SourcePlan,
    // Types
    SourceType,
    // Constants
    QA_STATS_DIR,
    QA_STATS_FILE,
};

// v0.0.21: Performance and Latency Sprint
pub use performance::{
    get_policy_version,
    // Helpers
    get_snapshot_hash,
    BudgetSettings,
    BudgetViolation,
    // Statistics
    CacheStats,
    LatencySample,
    LlmCache,
    LlmCacheEntry,
    // LLM caching
    LlmCacheKey,
    LlmCacheStats,
    PerfStats,
    // Token budgets
    TokenBudget,
    ToolCache,
    ToolCacheEntry,
    // Tool caching
    ToolCacheKey,
    // Constants
    CACHE_DIR,
    LLM_CACHE_DIR,
    LLM_CACHE_TTL_SECS,
    MAX_CACHE_ENTRIES,
    PERF_STATS_FILE,
    TOOL_CACHE_DIR,
    TOOL_CACHE_TTL_SECS,
};

// v0.0.22: Reliability Engineering
pub use reliability::{
    calculate_budget_status,
    check_budget_alerts,
    load_recent_errors,
    load_recent_ops_log,
    AlertSeverity,
    BudgetAlert,
    BudgetState,
    BudgetStatus,
    DailyMetrics,
    DiagStatus,
    // Self-diagnostics
    DiagnosticsReport,
    DiagnosticsSection,
    // Error budgets
    ErrorBudgets,
    LatencyRecord,
    // Metrics
    MetricType,
    MetricsStore,
    // Operations log
    OpsLogEntry,
    DEFAULT_LLM_TIMEOUT_BUDGET,
    DEFAULT_MUTATION_ROLLBACK_BUDGET,
    DEFAULT_REQUEST_FAILURE_BUDGET,
    DEFAULT_TOOL_FAILURE_BUDGET,
    // Constants
    METRICS_FILE,
};

// v0.0.33: Human-first transcript and case files
// Note: Some names conflict with other modules, so we use explicit prefixes
pub use transcript::{
    find_last_failure,
    // Utilities (renamed to avoid conflict with mutation_executor)
    generate_request_id as generate_case_id,
    get_cases_storage_size,
    list_recent_cases,
    list_today_cases,
    // Case retrieval
    load_case_summary,
    prune_cases,
    // Actors
    Actor as TranscriptActor,
    CaseFile,
    // v0.0.35: Model info for case files
    CaseModelInfo,
    CaseOutcome,
    CaseResult as TranscriptCaseResult,
    // Case file structures (renamed to avoid conflicts)
    CaseSummary,
    CaseTiming,
    EvidenceEntry as CaseEvidenceEntry,
    // v0.0.36: Knowledge refs for case files
    KnowledgeRef as CaseKnowledgeRef,
    // v0.0.48: Learning records for case files
    LearningRecord,
    PolicyRef as CasePolicyRef,
    // v0.0.37: Recipe events for case files
    RecipeEvent,
    RecipeEventType,
    TranscriptBuilder,
    // Transcript building
    TranscriptMessage,
    // Constants
    CASES_DIR,
    DEFAULT_MAX_SIZE_BYTES,
    DEFAULT_RETENTION_DAYS,
};

// v0.0.34: Fix-It Mode for bounded troubleshooting loops
pub use fixit::{
    // Detection
    is_fixit_request,
    ChangeItem,
    ChangeResult,
    // Change sets (mutation batches)
    ChangeSet,
    FixItRiskLevel,
    FixItSession,
    // State machine
    FixItState,
    FixTimeline,
    // Hypotheses
    Hypothesis,
    HypothesisTestResult,
    ProblemCategory,
    // Timeline tracking
    StateTransition,
    FIX_CONFIRMATION,
    // Constants
    MAX_HYPOTHESIS_CYCLES,
    MAX_MUTATIONS_PER_BATCH,
    MAX_TOOLS_PER_PHASE,
};

// v0.0.35: Model policy and readiness
pub use model_policy::{
    // Download progress (renamed to avoid conflict with model_selection::DownloadProgress)
    DownloadProgress as ModelDownloadProgress,
    DownloadStatus as ModelDownloadStatus,
    GlobalPolicy,
    // Readiness state
    ModelReadinessState,
    // Policy types
    ModelsPolicy,
    RolePolicy,
    ScoringWeights,
    DEFAULT_MODELS_POLICY,
    MODELS_POLICY_DIR,
    // Constants
    MODELS_POLICY_FILE,
};

// v0.0.38: Arch Networking Doctor
pub use networking_doctor::{
    collect_network_evidence,
    detect_manager_conflicts,
    detect_network_manager,
    get_fix_playbooks,
    run_diagnosis,
    DiagnosisResult,
    DiagnosisStatus,
    // Diagnosis flow
    DiagnosisStep,
    DiagnosisStepResult,
    // Fix playbooks
    FixPlaybook,
    FixResult,
    FixRiskLevel,
    FixStep,
    InterfaceEvidence,
    // Evidence collection
    NetworkEvidence,
    // Hypotheses
    NetworkHypothesis,
    // Network manager detection
    NetworkManager,
    NetworkManagerStatus,
    // Case file
    NetworkingDoctorCase,
    DNS_TEST_DOMAINS,
    FIX_CONFIRMATION as NET_FIX_CONFIRMATION,
    PING_COUNT,
    // Constants
    PING_TIMEOUT_SECS,
    RAW_IP_TEST,
};

// v0.0.39: Arch Storage Doctor (BTRFS Focus)
pub use storage_doctor::{
    BalanceStatus,
    BlockDevice,
    // BTRFS specific
    BtrfsDeviceStats,
    BtrfsInfo,
    BtrfsUsage,
    CaseNote as StorageCaseNote,
    CaseStatus as StorageCaseStatus,
    CheckResult as StorageCheckResult,
    CommandResult,
    DiagnosisResult as StorageDiagnosisResult,
    // Diagnosis flow
    DiagnosisStep as StorageDiagnosisStep,
    FilesystemType,
    Finding,
    IoErrorLog,
    IoErrorType,
    // Mount and device info
    MountInfo as StorageMountInfo,
    PostCheck as StoragePostCheck,
    PreflightCheck as StoragePreflightCheck,
    RepairCommand,
    RepairPlan,
    // Repair plans
    RepairPlanType,
    RepairResult,
    RiskLevel as StorageRiskLevel,
    RollbackPlan as StorageRollbackPlan,
    ScrubStatus,
    // SMART health
    SmartHealth,
    // Engine
    StorageDoctor,
    // Case file
    StorageDoctorCase,
    // Evidence collection
    StorageEvidence,
    // Health status
    StorageHealth as StorageDoctorHealth,
    StorageHypothesis,
};

// v0.0.40: Arch Audio Doctor (PipeWire Focus)
pub use audio_doctor::{
    // Devices
    AlsaDevice,
    // Engine
    AudioDoctor,
    // Case file
    AudioDoctorCase,
    // Evidence
    AudioEvidence,
    AudioHealth,
    AudioHypothesis,
    AudioNode,
    // Permissions
    AudioPermissions,
    // Audio stack
    AudioStack,
    // Bluetooth (aliased - base types from peripherals)
    BluetoothAdapter as AudioBluetoothAdapter,
    BluetoothAudioDevice,
    BluetoothProfile,
    BluetoothState as AudioBluetoothState,
    CaseNote as AudioCaseNote,
    CaseStatus as AudioCaseStatus,
    CheckResult as AudioCheckResult,
    CommandResult as AudioCommandResult,
    DiagnosisResult as AudioDiagnosisResult,
    DiagnosisStep as AudioDiagnosisStep,
    Finding as AudioFinding,
    FixPlaybook as AudioFixPlaybook,
    PlaybookCommand as AudioPlaybookCommand,
    PlaybookResult,
    // Playbooks
    PlaybookType,
    PostCheck as AudioPostCheck,
    PreflightCheck as AudioPreflightCheck,
    // Recipe capture
    RecipeCaptureRequest,
    RiskLevel as AudioRiskLevel,
    // Service state
    ServiceState as AudioServiceState,
    // Diagnosis
    StepResult as AudioStepResult,
    // Constants
    FIX_CONFIRMATION as AUDIO_FIX_CONFIRMATION,
};

// v0.0.41: Arch Boot Doctor (Slow Boot + Service Regressions)
pub use boot_doctor::{
    BootBaseline,
    // Engine
    BootDoctor,
    // Case file
    BootDoctorCase,
    // Evidence
    BootEvidence,
    // Health types
    BootHealth,
    BootHypothesis,
    BootOffender,
    // Timing types
    BootTiming,
    BootTrend,
    CaseNote as BootCaseNote,
    CaseStatus as BootCaseStatus,
    // Change tracking (aliased - base types in telemetry_db)
    ChangeEvent as BootChangeEvent,
    ChangeType as BootChangeType,
    CheckResult as BootCheckResult,
    CommandResult as BootCommandResult,
    DiagnosisResult as BootDiagnosisResult,
    DiagnosisStep as BootDiagnosisStep,
    Finding as BootFinding,
    FixPlaybook as BootFixPlaybook,
    JournalEntry as BootJournalEntry,
    PlaybookCommand as BootPlaybookCommand,
    PlaybookResult as BootPlaybookResult,
    // Playbooks
    PlaybookType as BootPlaybookType,
    PostCheck as BootPostCheck,
    PreflightCheck as BootPreflightCheck,
    // Recipe capture
    RecipeCaptureRequest as BootRecipeCaptureRequest,
    RiskLevel as BootRiskLevel,
    // Diagnosis
    StepResult as BootStepResult,
    TrendDirection as BootTrendDirection,
    // Constants
    FIX_CONFIRMATION as BOOT_FIX_CONFIRMATION,
};

// v0.0.42: Arch GPU/Graphics Doctor (Wayland/X11, Drivers, Compositor Health)
pub use graphics_doctor::{
    CaseNote as GraphicsCaseNote,
    CaseStatus as GraphicsCaseStatus,
    CheckResult as GraphicsCheckResult,
    CommandResult as GraphicsCommandResult,
    Compositor,
    DiagnosisResult as GraphicsDiagnosisResult,
    DiagnosisStep as GraphicsDiagnosisStep,
    DriverPackages,
    DriverStack,
    Finding as GraphicsFinding,
    FixPlaybook as GraphicsFixPlaybook,
    GpuInfo as GraphicsGpuInfo,
    // GPU/Driver types
    GpuVendor,
    // Engine
    GraphicsDoctor,
    // Case file
    GraphicsDoctorCase,
    // Evidence
    GraphicsEvidence,
    // Health types
    GraphicsHealth,
    GraphicsHypothesis,
    LogEntry as GraphicsLogEntry,
    // Monitor types
    MonitorInfo,
    PlaybookCommand as GraphicsPlaybookCommand,
    PlaybookResult as GraphicsPlaybookResult,
    // Playbooks
    PlaybookType as GraphicsPlaybookType,
    // Portal types
    PortalBackend,
    PortalState,
    PostCheck as GraphicsPostCheck,
    PreflightCheck as GraphicsPreflightCheck,
    // Recipe capture
    RecipeCaptureRequest as GraphicsRecipeCaptureRequest,
    RiskLevel as GraphicsRiskLevel,
    SessionInfo,
    // Session types (aliased - SessionType already in memory module)
    SessionType as GraphicsSessionType,
    // Diagnosis
    StepResult as GraphicsStepResult,
    // Constants
    FIX_CONFIRMATION as GRAPHICS_FIX_CONFIRMATION,
};

// v0.0.43: Doctor Registry + Unified Entry Flow
pub use doctor_registry::{
    // Config generation
    generate_default_config,
    get_doctor_run_stats,
    DoctorDomain,
    DoctorEntry,
    // Registry
    DoctorRegistry,
    // Registry config types
    DoctorRegistryConfig,
    // Run output schema
    DoctorRun,
    DoctorRunResult,
    // Run lifecycle types
    DoctorRunStage,
    DoctorRunStats,
    // Selection types
    DoctorSelection,
    FindingSeverity,
    JuniorVerification,
    KeyFinding,
    // Status integration
    LastDoctorRunSummary,
    PlaybookRunResult,
    SelectedDoctor,
    StageStatus,
    StageTiming,
    VerificationStatus,
    DOCTOR_RUNS_DIR,
    DOCTOR_RUN_SCHEMA_VERSION,
    // Constants
    REGISTRY_CONFIG_PATH,
    REGISTRY_CONFIG_PATH_USER,
};

// v0.0.48: Learning System (Knowledge Packs + XP)
// Note: Some types renamed to avoid conflicts with knowledge_packs and transcript modules
pub use learning::{
    // Search (renamed to avoid conflict)
    generate_knowledge_evidence_id as generate_learned_evidence_id,
    // Knowledge Packs (renamed to avoid conflicts)
    KnowledgePack as LearnedKnowledgePack,
    LearnedRecipe,
    // Learning Manager
    LearningManager,
    LearningResult,
    LearningStats,
    PackSource as LearnedPackSource,
    RecipeAction as LearnedRecipeAction,
    RecipeIntent as LearnedRecipeIntent,
    SearchHit,
    // XP System
    XpState,
    XpSummary,
    KNOWLEDGE_EVIDENCE_PREFIX as LEARNED_EVIDENCE_PREFIX,
    // Constants (renamed to avoid conflicts)
    LEARNING_PACKS_DIR,
    LEARNING_PACK_SCHEMA_VERSION,
    MAX_PACKS as MAX_LEARNED_PACKS,
    MAX_RECIPES_TOTAL,
    MAX_RECIPE_SIZE_BYTES,
    MIN_EVIDENCE_FOR_LEARNING,
    MIN_RELIABILITY_FOR_LEARNING,
    XP_GAIN_RECIPE_CREATED,
    XP_GAIN_SUCCESS_85,
    XP_GAIN_SUCCESS_90,
    XP_STATE_FILE,
};

// v0.0.49: Doctor Lifecycle System
// v0.0.64: Added DoctorLifecycleStage, IntakeResult, DoctorLifecycleState
pub use doctor_lifecycle::{
    ActionRisk,
    // Evidence collection
    CollectedEvidence,
    // Diagnosis output (DiagnosisResult renamed to avoid conflict with networking_doctor)
    DiagnosisFinding,
    DiagnosisResult as DoctorDiagnosis,
    // Diagnostic planning
    DiagnosticCheck,
    DnsSummary,
    // Core trait
    Doctor,
    // v0.0.64: Lifecycle stages
    DoctorLifecycleStage,
    DoctorLifecycleState,
    // Report
    DoctorReport,
    // Runner
    DoctorRunner,
    IntakeResult,
    // Network evidence helpers
    NetInterfaceSummary,
    NetworkErrorsSummary,
    NmSummary,
    ProposedAction,
    RouteSummary,
    SafeNextStep,
    WifiSummary,
};

// v0.0.49: NetworkingDoctor v2 (Doctor trait implementation)
pub use networking_doctor_v2::NetworkingDoctorV2;

// v0.0.50: User File Mutation
pub use user_file_mutation::{
    apply_edit,
    check_path_policy,
    execute_rollback as execute_user_file_rollback,
    generate_edit_preview,
    // Helpers
    generate_mutation_case_id as generate_user_file_case_id,
    // Apply
    ApplyResult,
    EditMode,
    // Preview
    EditPreview,
    FileStat,
    // Path policy
    PathPolicyResult,
    // Rollback
    RollbackResult as UserFileRollbackResult,
    // Edit action and mode
    UserFileEditAction,
    VerifyStrategy,
    USER_FILE_CONFIRMATION,
};

// v0.0.50: File Edit Tool Executors
pub use file_edit_tools::{
    execute_file_edit_apply_v1, execute_file_edit_preview_v1, execute_file_edit_rollback_v1,
};

// v0.0.51: Systemd Action Engine (modular)
pub use systemd_action::{
    // Risk assessment
    assess_risk,
    normalize_service_name,
    RiskLevel,
    ServiceAction,
    // Operations
    ServiceOperation,
    HIGH_RISK_CONFIRMATION,
    // Confirmation phrases (MEDIUM_RISK_CONFIRMATION is already exported from mutation_tools)
    LOW_RISK_CONFIRMATION,
};
pub use systemd_apply::{
    apply_service_action, preview_service_action, ServiceApplyResult, ServicePreview,
    ServiceStateSnapshot,
};
pub use systemd_probe::{probe_service, ServiceProbe, MAX_STATUS_LINES};
pub use systemd_rollback::{
    generate_service_case_id, rollback_service_action, ServiceRollbackResult, ROLLBACK_BASE,
};

// v0.0.52: System Query Router
pub use system_query_router::{
    detect_target, get_required_tools, get_tool_routing, map_translator_targets,
    validate_answer_for_target, QueryTarget, ToolRouting,
};

// v0.0.51: Systemd Tool Executors
pub use systemd_tools::{
    execute_systemd_service_apply_v1, execute_systemd_service_preview_v1,
    execute_systemd_service_probe_v1, execute_systemd_service_rollback_v1,
};

// v0.0.53: Doctor Flow
pub use doctor_flow::{
    detect_problem_phrase, CaseStep, CaseSuggestedAction, DoctorCaseFile, DoctorFlowExecutor,
    DoctorFlowResult, DoctorFlowStep, StepStatus,
};

// v0.0.54: Action Engine
pub use action_engine::{
    ActionDiffLine, ActionDiffPreview, ActionPlan, ActionResult, ActionStep, ActionType,
    BackupRecord, DeleteFileAction, DiffLineType as ActionDiffLineType, EditFileAction, EditIntent,
    MutationRiskLevel, PackageReason, PacmanAction, PacmanOperation, PlanStatus,
    RollbackHint as ActionRollbackHint, RollbackRecord, RollbackStepRecord,
    StepResult as ActionStepResult, StepStatus as ActionStepStatus,
    SystemdAction as ActionSystemdAction, SystemdOperation as ActionSystemdOp, VerificationRecord,
    WriteFileAction, CONFIRM_DESTRUCTIVE, CONFIRM_HIGH, CONFIRM_LOW, CONFIRM_MEDIUM,
};
pub use action_executor::{
    backup_file as action_backup_file, compute_hash as action_compute_hash,
    execute_plan as execute_action_plan, execute_step as execute_action_step,
    generate_action_diff_preview, get_backup_path as action_get_backup_path,
    validate_confirmation as validate_action_confirmation,
};
pub use action_risk::{
    describe_risk, score_action_risk, score_delete_risk, score_package_risk, score_path_risk,
    score_systemd_risk,
};

// v0.0.55: Case Engine + Intent Taxonomy
pub use case_engine::{
    CaseActor, CaseEvent, CaseEventType, CasePhase, CaseState, IntentType, PhaseTiming,
};
pub use case_file_v1::{
    list_recent_case_ids, load_case, CaseFileV1, EvidenceRecordV1, PhaseTimingRecord,
    CASE_FILES_DIR, CASE_SCHEMA_VERSION,
};
pub use evidence_tools::{plan_evidence, validate_evidence_for_query, EvidencePlan, PlannedTool};
pub use intent_taxonomy::{classify_intent, domain_to_doctor, IntentClassification};
pub use recipe_extractor::{
    calculate_case_xp, check_mutation_recipe_gate, check_recipe_gate, check_state_recipe_gate,
    extract_mutation_recipe, extract_recipe, MutationRecipeExtractionResult, RecipeExtractionResult,
    MIN_EVIDENCE_FOR_MUTATION_RECIPE, MIN_EVIDENCE_FOR_RECIPE as RECIPE_MIN_EVIDENCE,
    MIN_RELIABILITY_FOR_MUTATION_RECIPE, MIN_RELIABILITY_FOR_RECIPE as RECIPE_MIN_RELIABILITY,
};
pub use transcript_render::{
    render_compact_summary, render_recent_cases, render_transcript_from_file,
    render_transcript_from_state, truncate as transcript_truncate,
    wrap_text as transcript_wrap_text,
};

// v0.0.56: Fly-on-the-Wall Dialogue Layer
pub use dialogue_renderer::{
    doctor_actor_name,
    phase_separator,
    render_anna_coverage_retry,
    render_anna_junior_response,
    render_anna_translator_response,
    render_dialogue_transcript,
    render_doctor_handoff,
    render_evidence_item,
    render_evidence_summary,
    render_final_response,
    // v0.0.57: Coverage display in transcript
    render_junior_coverage_check,
    render_junior_verification,
    render_reliability_footer,
    render_translator_classification,
    DialogueContext,
    CONFIDENCE_CERTAIN_THRESHOLD,
    RELIABILITY_SHIP_THRESHOLD,
};

// v0.0.57: Evidence Coverage + Junior Rubric
pub use evidence_coverage::{
    analyze_coverage, calculate_coverage_penalty, check_evidence_mismatch, get_gap_filling_tools,
    get_max_score_for_coverage, get_target_facets, EvidenceCoverage, TargetFacets,
    COVERAGE_PENALTY_THRESHOLD, COVERAGE_SUFFICIENT_THRESHOLD,
};
pub use junior_rubric::{
    check_tool_relevance,
    get_evidence_suggestions,
    is_clearly_wrong_evidence,
    verify_answer,
    // v0.0.65: Topic-based relevance verification
    verify_answer_with_topic,
    Penalty,
    VerificationResult,
    ANSWER_MISMATCH_PENALTY,
    BASE_SCORE,
    HIGH_COVERAGE_BONUS,
    IRRELEVANT_TOOL_PENALTY,
    MISSING_EVIDENCE_PENALTY,
    SHIP_IT_THRESHOLD,
    UNCITED_CLAIM_PENALTY,
    WRONG_EVIDENCE_PENALTY,
};

// v0.0.58: Proactive Monitoring Loop v1
pub use alert_detectors::{
    // Detectors
    detect_boot_regression,
    detect_disk_pressure,
    detect_journal_error_burst,
    detect_service_failed,
    detect_thermal_throttling,
    run_all_detectors,
    // Evidence types
    BootRegressionEvidence,
    DiskPressureEvidence,
    JournalErrorBurstEvidence,
    ServiceFailedEvidence,
    ThermalThrottlingEvidence,
    BOOT_REGRESSION_MIN_DELTA_SECS,
    // Thresholds
    BOOT_REGRESSION_STDDEV_FACTOR,
    DISK_PRESSURE_CRITICAL_GIB,
    DISK_PRESSURE_CRITICAL_PERCENT,
    DISK_PRESSURE_WARNING_GIB,
    DISK_PRESSURE_WARNING_PERCENT,
    JOURNAL_ERROR_BURST_CRITICAL,
    JOURNAL_ERROR_BURST_WARNING,
    JOURNAL_ERROR_BURST_WINDOW_MINS,
    THERMAL_CRITICAL_TEMP,
    THERMAL_WARNING_TEMP,
};
pub use alert_probes::{
    probe_alerts_summary,
    // Probes
    probe_boot_time_summary,
    probe_disk_pressure_summary,
    probe_failed_units_summary,
    probe_journal_error_burst_summary,
    probe_thermal_summary,
    AlertCountsData,
    AlertSummaryEntry,
    AlertsSummary,
    // Probe results
    BootTimeSummary,
    DiskPressureSummary,
    FailedUnitsSummary,
    JournalErrorBurstSummary,
    ThermalSummary,
};
pub use proactive_alerts::{
    AlertCounts,
    AlertSeverity as ProactiveAlertSeverity, // Renamed to avoid conflict with reliability::AlertSeverity
    AlertStatus,
    AlertType,
    ProactiveAlert,
    ProactiveAlertsState,
    PROACTIVE_ALERTS_FILE,
    PROACTIVE_ALERTS_SCHEMA,
};

// v0.0.59: Auto-Case Opening + Departmental IT Org
// v0.0.66: Added TicketType, CaseOutcome for enhanced case tracking
pub use case_lifecycle::{
    count_active_cases,
    list_active_cases,
    load_case_v2,
    ActionRisk as CaseActionRisk, // Renamed to avoid conflict
    CaseFileV2,
    CaseOutcome as CaseLifecycleOutcome, // Renamed to avoid conflict with case_engine
    CaseStatus,
    Department,
    Participant,
    ProposedAction as CaseProposedAction, // Renamed to avoid conflict
    // v0.0.66: Ticket type and outcome
    TicketType,
    TimelineEvent,
    TimelineEventType,
    CASE_SCHEMA_VERSION_V2,
};
// v0.0.64: Service Desk Dispatcher with Ticket and RoutingPlan
// v0.0.66: Added detect_ticket_type, create_work_order, create_routing_decision
pub use service_desk::{
    create_routing_decision,
    create_work_order,
    // v0.0.66: Ticket type detection and work order
    detect_ticket_type,
    dispatch_request,
    dispatch_to_specialist,
    find_case_for_alert,
    // v0.0.64: Problem detection
    is_problem_report,
    open_case_for_alert,
    progress_case_investigating,
    progress_case_plan_ready,
    progress_case_triage,
    should_auto_open_case,
    triage_request,
    // v0.0.64: Dispatch result
    DispatchResult,
    HumanNarrationPlan,
    RoutingPlan,
    // v0.0.64: Ticket types
    Ticket,
    TicketCategory,
    TicketSeverity,
    // Legacy triage
    TriageResult,
};
// v0.0.66: Department Protocol exports
pub use department_protocol::{
    DepartmentFinding,
    DepartmentName,
    DepartmentResult,
    DepartmentTrait,
    FindingSeverity as DepartmentFindingSeverity, // Renamed to avoid conflict with doctor_registry
    InvestigateCtx,
    RecommendedAction,
    RoutingDecision,
    WorkOrder,
};
pub use transcript_v2::{
    render_active_cases_status,
    render_case_transcript,
    render_collaboration,
    render_handoff,
    render_junior_disagreement,
    DepartmentOutput,
    Hypothesis as DepartmentHypothesis, // Renamed to avoid conflict
    TranscriptBuilder as TranscriptBuilderV2, // Renamed to avoid conflict
    TranscriptLine,
};

// v0.0.61: Evidence Topics (Targeted Answers)
// v0.0.63: cap_reliability for strict answer validation + freshness tracking
pub use evidence_topic::{
    calculate_freshness_penalty, cap_reliability, detect_topic, generate_answer, get_topic_config,
    validate_evidence, with_evidence_freshness, EvidenceTopic, TopicConfig, TopicDetection,
    TopicValidation,
};

// v0.0.65: Typed Evidence System + Answer Shaping
pub use answer_shaper::{format_debug_answer, format_human_answer, shape_answer, ShapedAnswer};
pub use evidence_record::{
    get_evidence_schema, validate_evidence_data, EvidenceBundle, EvidenceRecord, EvidenceSchema,
    ProbeKind,
};
pub use evidence_router::{
    get_tool_for_topic, route_evidence, tool_satisfies_topic, EvidenceRouting,
};

// v0.0.62: Human Mode vs Debug Mode (Narrator)
pub use narrator::{
    clear_working, get_output_mode, is_debug_mode, narrate, phase as narrator_phase,
    topic_evidence_description, working as narrator_working, ActorVoice, NarratorEvent,
};

// v0.0.67: Department Evidence Playbooks v1
pub use action_proposal::{networking_actions, storage_actions, ActionProposal};
pub use evidence_playbook::{
    NetworkCauseCategory, NetworkingDiagnosis, PlaybookBundle, PlaybookEvidence, PlaybookTopic,
    StorageDiagnosis, StorageRiskLevel as PlaybookStorageRiskLevel,
};
pub use networking_playbook::{
    collect_addr_route_evidence, collect_dns_evidence,
    collect_errors_evidence as collect_network_errors_evidence, collect_link_evidence,
    collect_manager_evidence, networking_topics, run_networking_playbook, AddrRouteEvidence,
    DnsEvidence, LinkEvidence, ManagerEvidence, NetworkErrorsEvidence,
};
pub use storage_playbook::{
    collect_btrfs_evidence, collect_fstab_evidence, collect_io_errors_evidence,
    collect_mount_evidence, collect_smart_evidence, run_storage_playbook, storage_topics,
    BtrfsDeviceError, BtrfsEvidence, FstabEntry, FstabEvidence, IoErrorsEvidence, MountEvidence,
    SmartEvidence,
};

// v0.0.68: Two-Layer Transcript Renderer
pub use transcript_v068::{
    print_human_colored, render as render_transcript, render_debug as render_transcript_debug,
    render_human as render_transcript_human, render_to_string as render_transcript_to_string,
    validate_human_output, write_transcripts, TimestampedEvent, TranscriptEventV2, TranscriptRole,
    TranscriptStreamV2,
};

// v0.0.69: Service Desk Case Coordinator
pub use case_coordinator::{
    classify_intent as classify_request_intent,
    // Evidence topic mapping
    get_evidence_topics_for_target,
    ActionPlan as CaseActionPlan,
    // Main coordinator
    CaseCoordinator,
    // Consolidated assessment
    ConsolidatedAssessment,
    // Department reports
    DepartmentReport,
    EvidenceTopicSummary,
    // Intent classification
    RequestIntent,
    // Triage
    TriageDecision,
};

// v0.0.70: Dual Transcript Renderer
pub use transcript_v070::{
    print_colored as print_colored_v70,
    print_debug_colored as print_debug_colored_v70,
    print_human_colored as print_human_colored_v70,
    render as render_v70,
    render_debug as render_debug_v70,
    // Rendering
    render_human as render_human_v70,
    render_to_string as render_to_string_v70,
    tool_to_evidence_topic,
    validate_debug_has_internals,
    // Validation
    validate_human_output as validate_human_output_v70,
    // File I/O
    write_transcripts as write_transcripts_v70,
    // Actors
    ActorV70,
    // Events
    EventV70,
    // Evidence topic abstraction
    EvidenceTopicV70,
    TimestampedEventV70,
    TranscriptStatsV70,
    TranscriptStreamV70,
    FORBIDDEN_HUMAN,
};

// v0.0.71: Humanizer Layer
pub use humanizer::{
    humanize_case_open,
    humanize_caution,
    humanize_evidence,
    humanize_evidence_gather,
    humanize_final_answer,
    humanize_finding,
    humanize_junior_critique,
    humanize_missing_evidence,
    humanize_triage,
    // Standard tags
    tags as humanizer_tags,
    // Validation
    validate_answer_relevance,
    AnswerValidation,
    ConfidenceHint,
    DepartmentTag,
    EvidenceSummary,
    // Labels
    HumanLabel,
    // Transform functions
    HumanizedMessage,
    HumanizerContext,
    MessageTone,
    // Roles and tones
    StaffRole,
    // Threads
    ThreadBuilder,
    ThreadSegment,
    ThreadType,
    ThreadedTranscript,
};

// v0.0.72: Unified Dual Mode Transcript
pub use transcript_v072::{
    format_transcript_v72,
    is_line_clean_for_human,
    print_transcript_v72,
    // Rendering
    render_event_v72,
    render_stream_v72,
    render_to_string_v72,
    strip_ansi as strip_ansi_v72,
    validate_debug_has_internals as validate_debug_has_internals_v72,
    // Validation
    validate_human_output as validate_human_output_v72,
    // Output
    write_case_logs_v72,
    write_debug_log_v72,
    write_human_log_v72,
    write_line_v72,
    DebugValidation,
    // Events
    EventDataV72,
    PerfBreakdownV72,
    RenderedLineV72,
    RiskLevelV72,
    RoleV72,
    ToneV72,
    TranscriptEventV72,
    TranscriptStreamV72,
    WarningCategoryV72,
    FORBIDDEN_HUMAN_LITERALS,
    FORBIDDEN_HUMAN_PATTERNS,
};

// v0.0.75: Recipe Engine exports
pub use recipe_engine::{
    DomainRecipeStats, RecipeEngine, RecipeEngineState, RecipeEngineStats, RecipeGate, RecipeMatch,
    RecipeUseRecord, DEMOTION_FAILURE_THRESHOLD, MIN_EVIDENCE_DOCTOR, MIN_EVIDENCE_MUTATION,
    MIN_EVIDENCE_READ_ONLY, MIN_MATCH_SCORE, MIN_RELIABILITY_DOCTOR, MIN_RELIABILITY_MUTATION,
    MIN_RELIABILITY_READ_ONLY, RECIPE_ENGINE_STATE_FILE,
};

// v0.0.75: RPG Stats exports
pub use rpg_stats::{
    title_for_level, DomainStats, EscalationMetrics, LatencyMetrics, ReliabilityMetrics,
    RequestCounters, RpgStats, RpgStatsManager, XpData, ROLLING_WINDOW_SIZE, STATS_FILE,
};

// v0.0.75: Transcript v075 exports
pub use transcript_v075::{
    get_transcript_mode as get_transcript_mode_v75, human_case_open as human_case_open_v75,
    human_finding as human_finding_v75, human_missing_evidence as human_missing_evidence_v75,
    human_reliability_footer, human_timing_summary, human_triage as human_triage_v75,
    humanize_evidence as humanize_evidence_v75, is_debug_mode as is_debug_mode_v75,
    render_debug_evidence, render_human_evidence,
    validate_human_output as validate_human_output_v75, HumanEvidence, HumanStaffMessage,
};

// v0.0.80: Mutation Engine v1 exports
pub use config_mutation::{
    create_config_rollback, create_config_verification, execute_config_edit,
    generate_config_manual_commands, get_config_edit_risk, preview_config_edit,
    rollback_config_edit,
};
pub use mutation_engine_v1::{
    is_config_allowed, is_service_allowed, ConfigEditOp, MutationCategory, MutationDetail,
    MutationExecutionResult, MutationPlanState, MutationPlanV1, MutationPreview,
    MutationRiskLevel as MutationRiskLevelV1, MutationStats, MutationStep, PackageAction,
    RollbackResult as MutationRollbackResult, RollbackStep, ServiceAction as ServiceActionV1,
    StepExecutionResult, StepPreview, VerificationCheck, VerificationResult as MutationVerifyResult,
    ALLOWED_CONFIG_FILES, ALLOWED_SERVICES, MUTATION_STATS_FILE,
};
pub use mutation_orchestrator::{
    execute_plan as execute_mutation_plan_v1, execute_rollback as execute_mutation_rollback,
    format_preview_human, generate_preview, plan_config_mutation, plan_package_mutation,
    plan_service_mutation, update_stats, validate_confirmation as validate_mutation_confirmation_v1,
    verify_plan,
};
pub use package_mutation::{
    create_install_rollback, create_install_verification, create_remove_rollback,
    create_remove_verification, execute_package_install, execute_package_remove,
    generate_package_manual_commands, get_package_action_risk, get_package_info,
    is_package_installed as is_pkg_installed_v1, preview_package_install, preview_package_remove,
    PackageInfo,
};
pub use privilege::{
    check_privilege, format_privilege_blocked, generate_manual_commands as generate_priv_commands,
    has_passwordless_sudo, is_root, run_privileged, PrivilegeMethod, PrivilegeStatus,
};
pub use service_mutation::{
    create_service_rollback, create_service_verification, execute_service_action,
    generate_service_manual_commands, get_service_action_risk, get_service_state as get_svc_state_v1,
    preview_service_action as preview_svc_action_v1, ServiceStateInfo,
};
pub use mutation_transcript::{
    debug_execution_step, debug_mutation_preview, debug_plan_header, debug_rollback_result,
    debug_state_transition, debug_step_preview, debug_verification_result,
    get_transcript_mode as get_mutation_transcript_mode, human_confirmation_received,
    human_execution_step, human_mutation_preview, human_plan_header, human_privilege_blocked,
    human_rollback_result, human_step_preview, human_verification_result,
    is_debug_mode as is_mutation_debug_mode, render_execution_step, render_mutation_preview,
    render_plan_header, render_rollback, render_verification,
};

// v0.0.81: Real mutation verification exports
pub use mutation_verification::{
    get_package_info_detailed, resolve_service_unit, verify_backup_exists, verify_config_edit,
    verify_mutation_plan, verify_mutation_plan_detailed, verify_package_install,
    verify_package_remove, verify_service_action, DetailedVerificationResult, PackageInfoDetailed,
    ServiceUnitResolution,
};

// v0.0.81: Reliability gate exports
pub use reliability_gate::{
    check_reliability_gate, format_gate_blocked, format_gate_for_transcript,
    ActionType as ReliabilityActionType, ReliabilityGateResult, MIN_RELIABILITY_FOR_DIAGNOSIS,
    MIN_RELIABILITY_FOR_MUTATION, MIN_RELIABILITY_FOR_READ_ONLY,
};

// v0.0.81: Transcript config exports
pub use transcript_config::{
    format_timing, get_transcript_mode as get_transcript_mode_v81,
    humanize_evidence_ref, humanize_fallback, humanize_llm_error, humanize_parse_retry,
    humanize_tool_name, is_debug_mode as is_debug_mode_v81, render_if_debug, render_mode_aware,
    strip_evidence_ids, TranscriptMode as TranscriptModeV81,
};

// v0.0.81: Case file v0.0.81 exports (with v81 suffix to avoid conflicts)
pub use case_file_v081::{
    BackupRecord as BackupRecordV81, CaseDoctorData, CaseEvidence as CaseEvidenceV81,
    CaseFileV081, CaseMutationData, CaseStatus as CaseStatusV81, CaseSummaryV081,
    CaseTiming as CaseTimingV081, CaseVerification as CaseVerificationV81, DoctorFinding,
    MutationExecutionRecord, RollbackRecord as RollbackRecordV81, StepRecord,
    VerificationRecord as VerificationRecordV81, CASES_DIR as CASES_DIR_V81,
    CASE_SCHEMA_VERSION as CASE_SCHEMA_VERSION_V81,
};

// v0.0.81: Rollback executor exports
pub use rollback_executor::{
    find_cached_package, rollback_file, rollback_mutation, rollback_package_install,
    rollback_package_remove, rollback_service, RollbackResult as RollbackResultV81,
    PACMAN_CACHE_DIR,
};

// v0.0.81: Status enhancement exports (with V81 suffix for conflicting names)
pub use status_v081::{
    format_enhanced_status, get_debug_mode_status, get_enhanced_status, get_recent_cases,
    DebugModeStatus, DoctorStats, MutationStats as MutationStatsV81, StatusEnhancedV081,
    DOCTOR_STATS_FILE, MUTATION_STATS_FILE as MUTATION_STATS_FILE_V81,
};

// v0.0.82: Pre-router exports
pub use pre_router::{pre_route, PreRouterIntent, PreRouterResult};

// v0.0.82: Translator JSON schema exports
pub use translator_v082::{
    classify_deterministic, parse_translator_json, TranslatorIntent, TranslatorJsonOutput,
    TranslatorParseResult, TranslatorRisk, TRANSLATOR_JSON_SCHEMA, TRANSLATOR_JSON_SYSTEM_PROMPT,
};

// v0.0.82: Debug toggle exports
pub use debug_toggle::{
    generate_toggle_response, get_debug_status_message, parse_debug_toggle_request,
    toggle_debug_persistent, toggle_debug_session, DebugMode, ToggleResult,
};

// v0.0.82: Pipeline integration exports
pub use pipeline_v082::{try_direct_handle, DirectHandlerResult};
