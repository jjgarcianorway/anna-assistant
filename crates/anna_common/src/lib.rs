//! Anna Common v0.0.60 - 3-Tier Transcript Rendering
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
pub mod intrusion;
pub mod knowledge_collector;
pub mod knowledge_core;
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
pub mod log_atlas;
pub mod golden_baseline;
// v7.21.0: Config atlas, topology maps, impact view
pub mod config_atlas;
pub mod topology_map;
pub mod impact_view;
// v7.22.0: Scenario lenses and self toolchain hygiene
pub mod scenario_lens;
pub mod sw_lens;
pub mod toolchain;
// v7.23.0: Time-anchored trends, boot snapshots, inventory drift, config provenance
pub mod timeline;
pub mod boot_snapshot;
pub mod inventory_drift;
pub mod config_hygiene;
// v7.24.0: Relationship store, hotspots, relationships
pub mod relationship_store;
pub mod hotspots;
pub mod relationships;
// v7.26.0: Instrumentation manifest and auto-install
pub mod instrumentation;
pub mod auto_install;
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
pub mod snapshots;
pub mod snapshot_builder;

// v7.42.0: Control socket for daemon/CLI contract
pub mod control_socket;

// v0.0.4: Ollama local LLM client for Junior verification
pub mod ollama;

// v0.0.5: Role-based model selection and benchmarking
pub mod model_selection;

// v0.0.7: Read-only tool catalog and executor
pub mod tools;
pub mod tool_executor;

// v0.0.8: Mutation tools, rollback, and executor
pub mod mutation_tools;
pub mod rollback;
pub mod mutation_executor;

// v0.0.47: Append line mutation with full rollback
pub mod append_line_mutation;

// v0.0.9: Helper tracking with provenance
pub mod helpers;

// v0.0.10: Install state and installer review
pub mod install_state;
pub mod installer_review;

// v0.0.11: Safe auto-update system
pub mod update_system;

// v0.0.12: Proactive anomaly detection
pub mod anomaly_engine;

// v0.0.13: Conversation memory and recipe system
pub mod memory;
pub mod recipes;
pub mod introspection;

// v0.0.14: Policy engine and security posture
pub mod policy;
pub mod audit_log;

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
pub mod systemd_probe;
pub mod systemd_apply;
pub mod systemd_rollback;

// v0.0.51: Systemd Tool Executors
pub mod systemd_tools;

// v0.0.52: System Query Router
pub mod system_query_router;

// v0.0.53: Doctor Flow (orchestrates diagnostic flows)
pub mod doctor_flow;

// v0.0.54: Action Engine (safe mutations with diffs, rollback, confirmations)
pub mod action_engine;
pub mod action_risk;
pub mod action_executor;

// v0.0.55: Deterministic Case Engine + Doctor-first routing
pub mod case_engine;
pub mod intent_taxonomy;
pub mod evidence_tools;
pub mod case_file_v1;
pub mod recipe_extractor;
pub mod transcript_render;

// v0.0.56: Fly-on-the-Wall Dialogue Layer
pub mod dialogue_renderer;
#[cfg(test)]
mod dialogue_golden_tests;

// v0.0.57: Evidence Coverage + Correct Tool Routing
pub mod evidence_coverage;
pub mod junior_rubric;
#[cfg(test)]
mod evidence_coverage_tests;

// v0.0.58: Proactive Monitoring Loop v1
pub mod proactive_alerts;
pub mod alert_detectors;
pub mod alert_probes;

// v0.0.59: Auto-Case Opening + Departmental IT Org
pub mod case_lifecycle;
pub mod service_desk;
pub mod transcript_v2;
#[cfg(test)]
mod case_lifecycle_tests;

// v0.0.60: 3-Tier Transcript Rendering
pub mod transcript_events;
pub mod human_labels;
pub mod transcript_renderer;
#[cfg(test)]
mod transcript_mode_tests;

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
    // Constants
    TELEMETRY_DIR, TELEMETRY_STATE_FILE,
    PROCESS_ACTIVITY_LOG, COMMAND_USAGE_LOG, SERVICE_CHANGES_LOG,
    PACKAGE_CHANGES_LOG, ERROR_EVENTS_LOG,
    // Event types
    ProcessSample, CommandEvent, ServiceChangeEvent,
    PackageChangeType, PackageChangeEvent,
    // State
    TelemetryState,
    // Writer/Reader
    TelemetryWriter, TelemetryReader,
    // Time helpers
    hours_ago, days_ago, now,
    // Stats
    CommandStats, command_stats, top_commands,
};
// v7.2.0: SQLite telemetry database exports (with aggregations)
// v7.5.0: Enhanced with CPU time, exec counts, hotspots
// v7.6.0: Added MaintenanceResult for pruning
// v7.7.0: Added compact per-window stats (AllWindowStats, WindowStats, TopCompactEntry)
// v7.9.0: Added trend classification (24h vs 7d), TopIdentityWithTrend, TrendWithStats
// v7.27.0: Added boot_id for "this boot" aggregations
pub use telemetry_db::{
    TelemetryDb, ProcessTelemetrySample, ObjectTelemetry, TelemetryStats,
    SampleCounts, UsageStats, GlobalPeak, DataStatus, MaintenanceResult,
    EnhancedUsageStats, EnhancedWindowedStats, TopProcessEntry,
    HealthHotspot, TelemetryHealth,
    WindowStats, AllWindowStats, TopCompactEntry, format_cpu_time_compact,
    // v7.7.0: Trend and window status types
    Trend, TrendData, WindowStatusInfo, TopHighlightEntry,
    // v7.9.0: Enhanced trend types
    TrendWithStats, TopIdentityWithTrend,
    // v7.27.0: Boot ID support
    get_current_boot_id,
    TELEMETRY_DB_PATH,
    WINDOW_1H, WINDOW_24H, WINDOW_7D, WINDOW_30D,
    format_cpu_time, format_bytes_human,
};
// v8.0.0: Execution telemetry with per-object, per-day JSONL storage
pub use telemetry_exec::{
    ExecutionRecord, ExecTelemetryWriter, ExecTelemetryReader,
    ObjectTelemetryResult, EXEC_TELEMETRY_DIR,
    WindowStats as ExecWindowStats,
};
// v7.6.0: Anna needs model for missing tools and docs
pub use needs::{
    AnnaNeeds, Need, NeedType, NeedStatus, NeedScope, NeedsSummary,
    HardwareDeps, is_smartctl_available, is_nvme_available, is_sensors_available,
    is_nvidia_smi_available, is_iw_available, is_ethtool_available, is_man_available,
    get_tool_status,
};
// v7.12.0: Operations log for Anna's internal tooling audit trail
pub use ops_log::{
    OpsAction, OpsEntry, OpsLogWriter, OpsLogReader,
    OpsActionCounts, OpsLogSummary,
    INTERNAL_DIR, OPS_LOG_FILE,
};
// v7.16.0: Service lifecycle tracking
pub use service_lifecycle::{
    ServiceLifecycle, ServiceLifecycleSummary,
    find_related_units, find_hardware_related_units,
};
// v7.18.0: Change journal for tracking system changes
pub use change_journal::{
    ChangeType, ChangeEvent, ChangeDetails,
    ChangeJournalWriter, ChangeJournalReader,
    get_package_history, get_config_history, get_recent_changes,
    scan_pacman_log, JOURNAL_DIR, JOURNAL_FILE,
};
// v7.18.0: Boot timeline for per-boot health view
pub use boot_timeline::{
    BootSummary, BootPhase, SlowUnit,
    get_current_boot_summary, get_previous_boot_summary, get_boot_summary,
    get_boot_list, get_service_log_patterns_by_boot, LogPatternEntry,
    BOOT_TIMELINE_DIR,
};
// v7.18.0: Enhanced log patterns with pattern IDs and novelty
pub use log_patterns_enhanced::{
    LogPattern, PatternOccurrence, ServicePatternSummary,
    LogPatternAnalyzer, get_service_log_counts,
    LOG_PATTERNS_DIR,
};
// v7.20.0: Telemetry trends with deterministic labels
pub use telemetry_trends::{
    TrendDirection, WindowStats as TrendWindowStats, ProcessTrends, HardwareTrends,
    SignalTrends, get_process_trends, format_bytes_short,
};
// v7.20.0: Log atlas with pattern IDs and cross-boot visibility
pub use log_atlas::{
    LogPattern as AtlasLogPattern, ComponentAtlas, BootLogEntry, CrossBootLogSummary,
    get_service_log_atlas, get_device_log_atlas, normalize_message, format_timestamp_short,
    JOURNAL_DIR as ATLAS_JOURNAL_DIR, BASELINE_DIR,
};
// v7.20.0: Golden baseline for pattern comparison
pub use golden_baseline::{
    GoldenBaseline, BaselineTag, MAX_BASELINE_WARNINGS,
    find_or_create_service_baseline, find_or_create_device_baseline,
    tag_pattern, get_components_with_new_patterns,
};
// v7.21.0: Config atlas for clean per-component config discovery
pub use config_atlas::{
    ConfigAtlas, ConfigEntry, ConfigCategory, ConfigStatus,
    PrecedenceEntry, build_config_atlas,
};
// v7.21.0: Topology maps for software and hardware stacks
pub use topology_map::{
    SoftwareTopology, HardwareTopology, StackRole, ServiceGroup,
    CpuInfo, MemoryInfo, GpuInfo, StorageInfo, NetworkInfo, AudioInfo,
    build_software_topology, build_hardware_topology,
};
// v7.21.0: Impact view for resource consumer rankings
pub use impact_view::{
    SoftwareImpact, HardwareImpact, ConsumerEntry, DiskPressure, NetworkUsage,
    get_software_impact, get_hardware_impact,
    format_bytes as impact_format_bytes, format_bytes_compact,
};
// v7.22.0: Scenario lenses for category-aware views
pub use scenario_lens::{
    NetworkLens, NetworkInterface, NetworkTelemetry, NetworkEvent,
    StorageLens, StorageDevice, StorageHealth, StorageTelemetry,
    GraphicsLens, GpuDevice, DisplayConnector,
    AudioLens, AudioDevice,
    format_bytes as lens_format_bytes,
};
// v7.22.0: Software lenses for category views
pub use sw_lens::{
    NetworkSwLens, DisplaySwLens, AudioSwLens, PowerSwLens,
    ServiceEntry, ConfigFileEntry, ServiceTelemetry,
    is_sw_category, get_sw_category,
};
// v7.22.0: Self toolchain hygiene
pub use toolchain::{
    ToolCategory, AnnaTool, ToolStatus, ToolchainStatus, ToolchainSummary,
    InstallResult,
    get_anna_tools, check_toolchain, install_tool, ensure_tool,
    format_toolchain_section, format_toolchain_status_section,
};
// v7.23.0: Time-anchored trends
pub use timeline::{
    UsageTrends, HwTelemetryTrends, TimeWindow, TrendLabel,
    get_usage_trends, get_hw_telemetry_trends,
    format_usage_section, format_hw_telemetry_section,
    format_cpu_percent_with_range, format_percent, format_fraction_as_percent,
    format_memory as timeline_format_memory, format_temperature, format_io_bytes,
    get_logical_cores,
};
// v7.23.0: Boot snapshots
pub use boot_snapshot::{
    BootSnapshot, IncidentPattern,
    format_boot_snapshot_section,
};
// v7.23.0: Inventory drift
pub use inventory_drift::{
    InventorySnapshot, DriftSummary,
};
// v7.23.0: Config hygiene with provenance
pub use config_hygiene::{
    ConfigSource, ValidatedConfigEntry, ValidatedConfig,
    ConfigGraph, ConfigPrecedenceEntry,
    format_config_section, format_config_graph_section,
};
// v7.24.0: Relationship store
pub use relationship_store::{
    LinkType, Link, RelationshipStore,
    discover_package_service_links, discover_service_process_links,
    discover_device_driver_links, discover_driver_firmware_links,
};
// v7.24.0: Hotspots (v7.28.0: added NetworkHotspot)
pub use hotspots::{
    CpuHotspot, MemoryHotspot, StartFrequencyHotspot,
    TempHotspot, IoHotspot, LoadHotspot, GpuHotspot,
    NetworkHotspot,  // v7.28.0
    SoftwareHotspots, HardwareHotspots,
    get_software_hotspots, get_hardware_hotspots,
    format_software_hotspots_section, format_hardware_hotspots_section,
    format_status_hotspots_section,
};
// v7.24.0: Relationships
pub use relationships::{
    ServiceRelation, ProcessRelation, HardwareRelation, StackPackage,
    SoftwareRelationships, DriverRelation, FirmwareRelation,
    ServiceUsingDevice, SoftwareUsingDevice, HardwareRelationships,
    get_software_relationships, get_hardware_relationships,
    format_software_relationships_section, format_hardware_relationships_section,
};
// v7.26.0: Instrumentation manifest and auto-install
pub use instrumentation::{
    InstalledTool, AvailableTool, InstallAttempt, InstrumentationManifest,
    INSTRUMENTATION_FILE,
    get_known_tools, get_missing_tools, is_package_installed, get_package_version,
};
pub use auto_install::{
    InstallResult as AutoInstallResult, InstrumentationStatus,
    try_install, try_install_known_tool, get_instrumentation_status,
    // v7.28.0: In-band disclosure for auto-install
    PendingInstall, InstallDisclosure, ensure_tool_for_command,
    is_package_installed as auto_is_package_installed, COMMON_TOOLS,
};
pub use local_docs::{
    LocalDocResult, LocalDocsSummary,
    has_man_page, get_man_path, get_man_description,
    get_doc_paths, get_config_paths_from_pacman, get_sample_configs_from_pacman,
    resolve_local_docs, get_local_docs_summary,
};
// v7.28.0: Text wrapping for zero truncation
pub use text_wrap::{
    get_terminal_width, wrap_text, wrap_with_prefix,
    format_kv, format_list_item,
};
// v7.31.0: Telemetry format and update state
pub use telemetry_format::{
    TelemetryReadiness, WindowAvailability, TrendDelta,
    TrendDirection as TelemetryTrendDirection,  // Alias to avoid conflict
    get_logical_cpu_count, format_cpu_percent, format_cpu_percent_short,
    format_cpu_avg_peak, format_memory as fmt_memory, format_memory_avg_peak,
    format_duration_short, format_cpu_time as fmt_cpu_time,
    MIN_SAMPLES_1H, MIN_SAMPLES_24H, MIN_SAMPLES_7D, MIN_SAMPLES_30D,
};
// v7.34.0: Update checking with consolidated state
pub use update_checker::{
    CheckResult, check_anna_updates, run_update_check,
    is_check_due, is_daemon_running,
};
pub use ops_log::OpsLog;
// v7.32.0: Network trends and scoped scans
pub use network_trends::{
    InterfaceType, WiFiSample, EthernetSample, NetworkTrendWindow, InterfaceTrends,
    collect_wifi_sample, collect_ethernet_sample, detect_interface_type,
    list_network_interfaces, is_iw_available as network_is_iw_available,
    is_ethtool_available as network_is_ethtool_available, is_nmcli_available,
    format_rssi, format_link_quality,
};
pub use scoped_scan::{
    ScanScope, StalenessInfo, ScanResult, ScanData, ScopedScanner,
    MountInfo, InterfaceInfo, TempSensor,
    DEFAULT_TIME_BUDGET_MS, MAX_TIME_BUDGET_MS,
};
// v7.36.0: Bounded knowledge storage with chunking
pub use chunk_store::{
    // Hard limits
    MAX_CHUNK_BYTES, MAX_DOC_BYTES, MAX_CHUNKS_PER_DOC, CHUNK_STORE_PATH,
    // Rendering budgets
    BUDGET_STATUS, BUDGET_OVERVIEW, BUDGET_DETAIL,
    // Types
    DocType, DocEntry, DocIndex, ExtractedFacts, FactWithSource, LocationHint, LocationScope,
    OverflowInfo,
    // Operations
    store_document, read_chunks, read_facts, delete_document,
    render_bounded, sanitize_to_plain_text,
};
// v7.32.0: Evidence-based categorization and game platform detection
pub use grounded::{
    // Category evidence
    Confidence, EvidenceSource, CategoryAssignment, classify_software,
    // Steam detection
    SteamGame, SteamLibrary, detect_steam_games, detect_steam_libraries,
    find_steam_game, is_steam_installed, get_steam_root, get_steam_games_count,
    format_game_size,
    // Game platforms
    Platform, PlatformGame, detect_heroic_games, detect_lutris_games,
    detect_bottles_games, detect_all_platform_games, get_platforms_summary,
    // v7.25.0+: Peripherals (USB, Bluetooth, Thunderbolt, SD, audio, input)
    PeripheralUsbDevice, UsbController, UsbSummary,
    BluetoothAdapter, BluetoothState, BluetoothSummary,
    ThunderboltController, ThunderboltDevice, ThunderboltSummary,
    SdCardReader, SdCardSummary,
    CameraDevice, CameraSummary,
    InputDevice, InputType, InputSummary,
    AudioCard, AudioSummary,
    HardwareOverview, FirewireSummary, FirewireController,
    get_usb_summary, get_bluetooth_summary, get_thunderbolt_summary,
    get_sdcard_summary, get_camera_summary, get_input_summary,
    get_audio_summary, get_hardware_overview,
};
// v7.39.0: Domain-based incremental refresh
pub use domain_state::{
    Domain, DomainRefreshState, RefreshResult, RefreshRequest, RefreshResponse, DomainSummary,
    DOMAIN_STATE_SCHEMA_VERSION, DOMAIN_STATE_DIR, REQUESTS_DIR, RESPONSES_DIR,
    cleanup_old_requests,
};
// v7.39.0: Terminal-adaptive rendering
pub use terminal::{
    DisplayMode, SimpleTable,
    get_terminal_size, truncate, wrap_text as terminal_wrap_text,
    format_with_overflow, format_compact_line,
    MIN_WIDTH, WIDE_WIDTH_THRESHOLD, COMPACT_HEIGHT_THRESHOLD, COMPACT_WIDTH_THRESHOLD,
};
// v7.39.0: Daemon self-observation
pub use self_observation::{
    SelfSample, SelfObservation, SelfWarning, WarningKind,
    DEFAULT_CPU_THRESHOLD, DEFAULT_RSS_THRESHOLD_BYTES, CPU_WINDOW_SECONDS, SELF_SAMPLE_INTERVAL_SECS,
};
// v0.0.4/v0.0.5: Ollama local LLM client
pub use ollama::{
    OllamaClient, OllamaStatus, OllamaModel, OllamaError,
    GenerateRequest, GenerateResponse, GenerateOptions,
    PullRequest, PullProgress,
    select_junior_model, is_ollama_installed, get_ollama_version,
    OLLAMA_DEFAULT_URL, HEALTH_CHECK_TIMEOUT_MS, GENERATE_TIMEOUT_MS,
};
// v0.0.5: Role-based model selection and benchmarking
pub use model_selection::{
    HardwareProfile, HardwareTier, LlmRole,
    ModelCandidate, ModelSelection,
    BenchmarkCase, CaseResult, BenchmarkResults,
    BootstrapPhase, BootstrapState, DownloadProgress,
    default_candidates, translator_benchmark_cases, junior_benchmark_cases,
    run_benchmark, select_model_for_role,
};
// v0.0.7: Read-only tool catalog and executor
pub use tools::{
    ToolCatalog, ToolDef, ToolSecurity, LatencyHint,
    ToolResult, ToolRequest, ToolPlan,
    EvidenceCollector,
    parse_tool_plan, unavailable_result, unknown_tool_result,
};
pub use tool_executor::{execute_tool, execute_tool_plan};
// v0.0.8: Mutation tools, rollback, and executor
pub use mutation_tools::{
    MutationToolCatalog, MutationToolDef, MutationRisk,
    MutationRequest, MutationResult, MutationPlan, MutationError,
    RollbackInfo, FileEditOp, ServiceState,
    is_path_allowed, validate_mutation_path, validate_confirmation,
    validate_mutation_request, get_service_state,
    MEDIUM_RISK_CONFIRMATION, MAX_EDIT_FILE_SIZE,
};
pub use rollback::{
    RollbackManager, MutationLogEntry, MutationType, MutationDetails,
    ROLLBACK_BASE_DIR, ROLLBACK_FILES_DIR, ROLLBACK_LOGS_DIR,
};
pub use mutation_executor::{
    execute_mutation, execute_mutation_plan,
    generate_request_id, create_file_edit_request, create_systemd_request,
    create_package_install_request, create_package_remove_request,
};
// v0.0.16: Mutation safety system exports
pub use mutation_safety::{
    MutationState, PreflightCheck, PreflightResult,
    DiffLine, DiffPreview, PostCheck, PostCheckResult,
    RollbackResult, SafeMutationExecutor,
    generate_request_id as safety_generate_request_id,
};

// v0.0.47: Append line mutation exports
pub use append_line_mutation::{
    SandboxCheck, RiskLevel as AppendRiskLevel,
    AppendMutationEvidence, FileStatEvidence, FilePreviewEvidence,
    AppendDiffPreview, AppendMutationResult,
    RollbackResult as AppendRollbackResult,
    check_sandbox, collect_evidence as collect_append_evidence,
    generate_diff_preview, check_mutation_allowed,
    execute_append_line, execute_rollback as execute_append_rollback,
    generate_mutation_case_id,
    SANDBOX_CONFIRMATION, HOME_CONFIRMATION,
};

// v0.0.9: Helper tracking exports
// v0.0.23: Added install functions (install_package, install_ollama)
pub use helpers::{
    HelpersManifest, HelperState, HelperDefinition, InstalledBy,
    HelpersSummary, HelperStatusEntry, InstallResult as HelperInstallResult,
    get_helper_definitions, get_helper_status_list, get_helpers_summary,
    refresh_helper_states, is_package_present, get_package_version as helpers_get_package_version,
    install_package, install_ollama, is_command_available, get_ollama_version as helpers_get_ollama_version,
    // v0.0.30: Install all missing helpers on daemon start
    install_missing_helpers,
    HELPERS_STATE_FILE,
};

// v0.0.10: Install state and installer review exports
// v0.0.25: Added InstallState::ensure_initialized for auto-creation on daemon start
pub use install_state::{
    InstallState, BinaryInfo, UnitInfo, DirectoryInfo, ReviewResult, LastReview,
    discover_install_state, INSTALL_STATE_PATH, INSTALL_STATE_SCHEMA,
};
pub use installer_review::{
    CheckResult as InstallerCheckResult, InstallerReviewReport,
    run_installer_review, run_and_record_review,
};

// v0.0.11: Update system exports
// v0.0.26: Added perform_auto_update for full auto-update in daemon
pub use update_system::{
    UpdateManager, UpdateChannel as UpdateSystemChannel, UpdatePhase,
    UpdateMarker, BackupEntry, ReleaseInfo, ReleaseArtifact,
    IntegrityStatus, GuardrailResult,
    is_newer_version, generate_update_evidence_id, handle_post_restart,
    perform_auto_update,
    UPDATE_STAGE_DIR, UPDATE_BACKUP_DIR, UPDATE_MARKER_FILE, MIN_DISK_SPACE_BYTES,
};

// v0.0.12: Anomaly detection exports
pub use anomaly_engine::{
    AnomalySeverity, AnomalySignal, TimeWindow as AnomalyTimeWindow, Anomaly,
    AlertQueue, AnomalyThresholds, AnomalyEngine,
    ALERTS_FILE, ALERTS_SCHEMA_VERSION,
    // What changed tool
    WhatChangedResult, PackageChange, ConfigChange as AnomalyConfigChange, what_changed,
    // Slowness analysis tool
    SlownessHypothesis, SlownessAnalysisResult, analyze_slowness,
};

// v0.0.13: Memory and recipe exports
pub use memory::{
    SessionRecord, TranslatorSummary, ToolUsage, RecipeAction, SessionType,
    MemoryManager, MemoryStats, MemoryIndex,
    MEMORY_DIR, SESSIONS_FILE, MEMORY_INDEX_FILE, MEMORY_ARCHIVE_DIR,
    MEMORY_SCHEMA_VERSION, MEMORY_EVIDENCE_PREFIX,
    generate_memory_evidence_id,
};
pub use recipes::{
    Recipe, IntentPattern, RecipeToolPlan, RecipeToolStep,
    RecipeSafety, RecipeRiskLevel, Precondition, PreconditionCheck,
    RollbackTemplate, RecipeCreator, RecipeManager, RecipeStats, RecipeIndex,
    ArchivedRecipe,
    // v0.0.37: New recipe types
    RecipeStatus, PostCheck as RecipePostCheck, PostCheckType as RecipePostCheckType,
    RECIPES_DIR, RECIPE_INDEX_FILE, RECIPE_ARCHIVE_DIR,
    RECIPE_SCHEMA_VERSION, RECIPE_EVIDENCE_PREFIX, MIN_RELIABILITY_FOR_RECIPE,
    generate_recipe_id,
};
pub use introspection::{
    IntrospectionIntent, IntrospectionResult, IntrospectionItem, IntrospectionItemType,
    FORGET_CONFIRMATION,
    detect_introspection_intent, execute_introspection, execute_forget,
};

// v0.0.14: Policy engine exports
// v0.0.23: Added ensure_policy_defaults for auto-creation on first run
pub use policy::{
    Policy, PolicyError, PolicyCheckResult, PolicyValidation,
    CapabilitiesPolicy, RiskPolicy, BlockedPolicy, HelpersPolicy,
    FileEditPolicy, SystemdPolicy, PackagePolicy,
    POLICY_DIR, CAPABILITIES_FILE, RISK_FILE, BLOCKED_FILE, HELPERS_FILE,
    POLICY_SCHEMA_VERSION, POLICY_EVIDENCE_PREFIX,
    generate_policy_evidence_id, get_policy, reload_policy, clear_policy_cache,
    ensure_policy_defaults,
};
pub use audit_log::{
    AuditLogger, AuditEntry, AuditEntryType, AuditResult,
    sanitize_for_audit, redact_env_secrets,
    AUDIT_DIR, AUDIT_LOG_FILE, AUDIT_ARCHIVE_DIR,
};

// v0.0.17: Target user and multi-user correctness exports
pub use target_user::{
    // User info
    UserInfo, UserSelectionSource, TargetUserSelection, AmbiguousUserSelection,
    SelectionResult, TargetUserSelector,
    // Home directory functions
    get_user_home, is_path_in_user_home, get_path_relative_to_home,
    expand_home_path, contract_home_path,
    // User-scoped operations
    UserScopeError, write_file_as_user, create_dir_as_user,
    backup_file_as_user, check_file_ownership, fix_file_ownership,
    // Policy helpers
    is_home_path_allowed, DEFAULT_ALLOWED_HOME_PATHS, DEFAULT_BLOCKED_HOME_PATHS,
    // Evidence
    USER_EVIDENCE_PREFIX, generate_user_evidence_id,
};
pub use policy::UserHomePolicy;

// v0.0.18: Secrets hygiene and redaction exports
pub use redaction::{
    // Main redaction functions
    redact, redact_secrets, contains_secrets, detect_secret_types,
    // Context-specific redaction
    redact_transcript, redact_evidence, redact_audit_details, redact_memory_content,
    // Environment variable redaction
    redact_env_value, redact_env_map,
    // Path restriction
    is_path_restricted, get_restriction_message, RESTRICTED_EVIDENCE_PATHS,
    // Junior verification
    check_for_leaks, LeakCheckResult,
    // Types
    SecretType, RedactionResult,
    // Evidence
    REDACTION_EVIDENCE_PREFIX, generate_redaction_id,
};

// v0.0.19: Knowledge Packs exports
pub use knowledge_packs::{
    // Storage
    KnowledgeIndex, KnowledgePack, KnowledgeDocument, KnowledgeStats,
    // Types
    PackSource, TrustLevel, RetentionPolicy, SearchResult,
    // Ingestion
    ingest_manpages, ingest_package_docs, ingest_project_docs, ingest_user_note,
    // Constants
    KNOWLEDGE_PACKS_DIR, KNOWLEDGE_INDEX_PATH, KNOWLEDGE_EVIDENCE_PREFIX,
    MAX_EXCERPT_LENGTH, DEFAULT_TOP_K,
    // Evidence
    generate_knowledge_evidence_id,
};

// v0.0.20: Source labeling for Ask Me Anything mode
pub use source_labels::{
    // Types
    SourceType, QuestionType, SourcePlan, AnswerContext, MissingEvidenceReport, QaStats,
    // Functions
    classify_question_type, detect_source_types, has_proper_source_labels, count_citations,
    // Constants
    QA_STATS_DIR, QA_STATS_FILE,
};

// v0.0.21: Performance and Latency Sprint
pub use performance::{
    // Token budgets
    TokenBudget, BudgetSettings, BudgetViolation,
    // Tool caching
    ToolCacheKey, ToolCacheEntry, ToolCache,
    // LLM caching
    LlmCacheKey, LlmCacheEntry, LlmCache,
    // Statistics
    CacheStats, LlmCacheStats, LatencySample, PerfStats,
    // Helpers
    get_snapshot_hash, get_policy_version,
    // Constants
    CACHE_DIR, TOOL_CACHE_DIR, LLM_CACHE_DIR, PERF_STATS_FILE,
    TOOL_CACHE_TTL_SECS, LLM_CACHE_TTL_SECS, MAX_CACHE_ENTRIES,
};

// v0.0.22: Reliability Engineering
pub use reliability::{
    // Metrics
    MetricType, DailyMetrics, MetricsStore, LatencyRecord,
    // Error budgets
    ErrorBudgets, BudgetStatus, BudgetState, BudgetAlert, AlertSeverity,
    calculate_budget_status, check_budget_alerts,
    // Operations log
    OpsLogEntry, load_recent_ops_log, load_recent_errors,
    // Self-diagnostics
    DiagnosticsReport, DiagnosticsSection, DiagStatus,
    // Constants
    METRICS_FILE,
    DEFAULT_REQUEST_FAILURE_BUDGET, DEFAULT_TOOL_FAILURE_BUDGET,
    DEFAULT_MUTATION_ROLLBACK_BUDGET, DEFAULT_LLM_TIMEOUT_BUDGET,
};

// v0.0.33: Human-first transcript and case files
// Note: Some names conflict with other modules, so we use explicit prefixes
pub use transcript::{
    // Actors
    Actor as TranscriptActor,
    // Transcript building
    TranscriptMessage, TranscriptBuilder,
    // Case file structures (renamed to avoid conflicts)
    CaseSummary, CaseOutcome, CaseFile,
    EvidenceEntry as CaseEvidenceEntry,
    PolicyRef as CasePolicyRef,
    CaseTiming,
    CaseResult as TranscriptCaseResult,
    // v0.0.35: Model info for case files
    CaseModelInfo,
    // v0.0.36: Knowledge refs for case files
    KnowledgeRef as CaseKnowledgeRef,
    // v0.0.37: Recipe events for case files
    RecipeEvent, RecipeEventType,
    // v0.0.48: Learning records for case files
    LearningRecord,
    // Case retrieval
    load_case_summary, list_recent_cases, list_today_cases, find_last_failure,
    get_cases_storage_size, prune_cases,
    // Utilities (renamed to avoid conflict with mutation_executor)
    generate_request_id as generate_case_id,
    // Constants
    CASES_DIR, DEFAULT_RETENTION_DAYS, DEFAULT_MAX_SIZE_BYTES,
};

// v0.0.34: Fix-It Mode for bounded troubleshooting loops
pub use fixit::{
    // State machine
    FixItState, FixItSession, ProblemCategory,
    // Hypotheses
    Hypothesis, HypothesisTestResult,
    // Change sets (mutation batches)
    ChangeSet, ChangeItem, ChangeResult, FixItRiskLevel,
    // Timeline tracking
    StateTransition, FixTimeline,
    // Detection
    is_fixit_request,
    // Constants
    MAX_HYPOTHESIS_CYCLES, MAX_TOOLS_PER_PHASE, MAX_MUTATIONS_PER_BATCH, FIX_CONFIRMATION,
};

// v0.0.35: Model policy and readiness
pub use model_policy::{
    // Policy types
    ModelsPolicy, RolePolicy, GlobalPolicy, ScoringWeights,
    // Download progress (renamed to avoid conflict with model_selection::DownloadProgress)
    DownloadProgress as ModelDownloadProgress,
    DownloadStatus as ModelDownloadStatus,
    // Readiness state
    ModelReadinessState,
    // Constants
    MODELS_POLICY_FILE, MODELS_POLICY_DIR, DEFAULT_MODELS_POLICY,
};

// v0.0.38: Arch Networking Doctor
pub use networking_doctor::{
    // Network manager detection
    NetworkManager, NetworkManagerStatus,
    detect_network_manager, detect_manager_conflicts,
    // Diagnosis flow
    DiagnosisStep, DiagnosisStepResult, DiagnosisResult, DiagnosisStatus,
    // Evidence collection
    NetworkEvidence, InterfaceEvidence,
    collect_network_evidence, run_diagnosis,
    // Hypotheses
    NetworkHypothesis,
    // Fix playbooks
    FixPlaybook, FixStep, FixRiskLevel, FixResult,
    get_fix_playbooks,
    // Case file
    NetworkingDoctorCase,
    // Constants
    PING_TIMEOUT_SECS, PING_COUNT, DNS_TEST_DOMAINS, RAW_IP_TEST,
    FIX_CONFIRMATION as NET_FIX_CONFIRMATION,
};

// v0.0.39: Arch Storage Doctor (BTRFS Focus)
pub use storage_doctor::{
    // Health status
    StorageHealth as StorageDoctorHealth, RiskLevel as StorageRiskLevel, FilesystemType,
    // Mount and device info
    MountInfo as StorageMountInfo, BlockDevice,
    // BTRFS specific
    BtrfsDeviceStats, BtrfsUsage, BtrfsInfo,
    ScrubStatus, BalanceStatus,
    // SMART health
    SmartHealth, IoErrorLog, IoErrorType,
    // Evidence collection
    StorageEvidence,
    // Diagnosis flow
    DiagnosisStep as StorageDiagnosisStep, Finding, StorageHypothesis,
    DiagnosisResult as StorageDiagnosisResult,
    // Repair plans
    RepairPlanType, RepairPlan, RepairCommand,
    PreflightCheck as StoragePreflightCheck, PostCheck as StoragePostCheck,
    RollbackPlan as StorageRollbackPlan, RepairResult,
    CommandResult, CheckResult as StorageCheckResult,
    // Case file
    StorageDoctorCase, CaseStatus as StorageCaseStatus, CaseNote as StorageCaseNote,
    // Engine
    StorageDoctor,
};

// v0.0.40: Arch Audio Doctor (PipeWire Focus)
pub use audio_doctor::{
    // Audio stack
    AudioStack, AudioHealth, RiskLevel as AudioRiskLevel,
    // Service state
    ServiceState as AudioServiceState,
    // Devices
    AlsaDevice, AudioNode,
    // Bluetooth (aliased - base types from peripherals)
    BluetoothAdapter as AudioBluetoothAdapter, BluetoothAudioDevice, BluetoothProfile,
    BluetoothState as AudioBluetoothState,
    // Permissions
    AudioPermissions,
    // Evidence
    AudioEvidence,
    // Diagnosis
    StepResult as AudioStepResult, DiagnosisStep as AudioDiagnosisStep,
    Finding as AudioFinding, AudioHypothesis,
    DiagnosisResult as AudioDiagnosisResult,
    // Playbooks
    PlaybookType, PlaybookCommand as AudioPlaybookCommand,
    PreflightCheck as AudioPreflightCheck, PostCheck as AudioPostCheck,
    FixPlaybook as AudioFixPlaybook, PlaybookResult,
    CommandResult as AudioCommandResult, CheckResult as AudioCheckResult,
    // Recipe capture
    RecipeCaptureRequest,
    // Case file
    AudioDoctorCase, CaseStatus as AudioCaseStatus, CaseNote as AudioCaseNote,
    // Engine
    AudioDoctor,
    // Constants
    FIX_CONFIRMATION as AUDIO_FIX_CONFIRMATION,
};

// v0.0.41: Arch Boot Doctor (Slow Boot + Service Regressions)
pub use boot_doctor::{
    // Health types
    BootHealth, RiskLevel as BootRiskLevel,
    // Timing types
    BootTiming, BootOffender, BootBaseline, BootTrend,
    TrendDirection as BootTrendDirection,
    // Change tracking (aliased - base types in telemetry_db)
    ChangeEvent as BootChangeEvent, ChangeType as BootChangeType,
    // Evidence
    BootEvidence, JournalEntry as BootJournalEntry,
    // Diagnosis
    StepResult as BootStepResult, DiagnosisStep as BootDiagnosisStep,
    Finding as BootFinding, BootHypothesis,
    DiagnosisResult as BootDiagnosisResult,
    // Playbooks
    PlaybookType as BootPlaybookType, PlaybookCommand as BootPlaybookCommand,
    PreflightCheck as BootPreflightCheck, PostCheck as BootPostCheck,
    FixPlaybook as BootFixPlaybook, PlaybookResult as BootPlaybookResult,
    CommandResult as BootCommandResult, CheckResult as BootCheckResult,
    // Recipe capture
    RecipeCaptureRequest as BootRecipeCaptureRequest,
    // Case file
    BootDoctorCase, CaseStatus as BootCaseStatus, CaseNote as BootCaseNote,
    // Engine
    BootDoctor,
    // Constants
    FIX_CONFIRMATION as BOOT_FIX_CONFIRMATION,
};

// v0.0.42: Arch GPU/Graphics Doctor (Wayland/X11, Drivers, Compositor Health)
pub use graphics_doctor::{
    // Session types (aliased - SessionType already in memory module)
    SessionType as GraphicsSessionType, Compositor, SessionInfo,
    // GPU/Driver types
    GpuVendor, GpuInfo as GraphicsGpuInfo, DriverStack, DriverPackages,
    // Portal types
    PortalBackend, PortalState,
    // Monitor types
    MonitorInfo, LogEntry as GraphicsLogEntry,
    // Health types
    GraphicsHealth, RiskLevel as GraphicsRiskLevel,
    // Evidence
    GraphicsEvidence,
    // Diagnosis
    StepResult as GraphicsStepResult, DiagnosisStep as GraphicsDiagnosisStep,
    Finding as GraphicsFinding, GraphicsHypothesis,
    DiagnosisResult as GraphicsDiagnosisResult,
    // Playbooks
    PlaybookType as GraphicsPlaybookType, PlaybookCommand as GraphicsPlaybookCommand,
    PreflightCheck as GraphicsPreflightCheck, PostCheck as GraphicsPostCheck,
    FixPlaybook as GraphicsFixPlaybook, PlaybookResult as GraphicsPlaybookResult,
    CommandResult as GraphicsCommandResult, CheckResult as GraphicsCheckResult,
    // Recipe capture
    RecipeCaptureRequest as GraphicsRecipeCaptureRequest,
    // Case file
    GraphicsDoctorCase, CaseStatus as GraphicsCaseStatus, CaseNote as GraphicsCaseNote,
    // Engine
    GraphicsDoctor,
    // Constants
    FIX_CONFIRMATION as GRAPHICS_FIX_CONFIRMATION,
};

// v0.0.43: Doctor Registry + Unified Entry Flow
pub use doctor_registry::{
    // Registry config types
    DoctorRegistryConfig, DoctorEntry, DoctorDomain,
    // Selection types
    DoctorSelection, SelectedDoctor,
    // Run lifecycle types
    DoctorRunStage, StageTiming, StageStatus, DoctorRunResult,
    KeyFinding, FindingSeverity,
    // Run output schema
    DoctorRun, PlaybookRunResult, VerificationStatus, JuniorVerification,
    DOCTOR_RUN_SCHEMA_VERSION,
    // Registry
    DoctorRegistry,
    // Status integration
    LastDoctorRunSummary, DoctorRunStats, get_doctor_run_stats,
    // Config generation
    generate_default_config,
    // Constants
    REGISTRY_CONFIG_PATH, REGISTRY_CONFIG_PATH_USER, DOCTOR_RUNS_DIR,
};

// v0.0.48: Learning System (Knowledge Packs + XP)
// Note: Some types renamed to avoid conflicts with knowledge_packs and transcript modules
pub use learning::{
    // XP System
    XpState, XpSummary,
    XP_GAIN_SUCCESS_85, XP_GAIN_SUCCESS_90, XP_GAIN_RECIPE_CREATED,
    // Knowledge Packs (renamed to avoid conflicts)
    KnowledgePack as LearnedKnowledgePack,
    PackSource as LearnedPackSource,
    LearnedRecipe,
    RecipeIntent as LearnedRecipeIntent,
    RecipeAction as LearnedRecipeAction,
    // Learning Manager
    LearningManager, SearchHit, LearningResult, LearningStats,
    // Search (renamed to avoid conflict)
    generate_knowledge_evidence_id as generate_learned_evidence_id,
    // Constants (renamed to avoid conflicts)
    LEARNING_PACKS_DIR, XP_STATE_FILE,
    MAX_PACKS as MAX_LEARNED_PACKS,
    MAX_RECIPES_TOTAL, MAX_RECIPE_SIZE_BYTES,
    MIN_RELIABILITY_FOR_LEARNING, MIN_EVIDENCE_FOR_LEARNING,
    KNOWLEDGE_EVIDENCE_PREFIX as LEARNED_EVIDENCE_PREFIX,
    LEARNING_PACK_SCHEMA_VERSION,
};

// v0.0.49: Doctor Lifecycle System
pub use doctor_lifecycle::{
    // Core trait
    Doctor,
    // Diagnostic planning
    DiagnosticCheck,
    // Evidence collection
    CollectedEvidence,
    // Diagnosis output (DiagnosisResult renamed to avoid conflict with networking_doctor)
    DiagnosisFinding, DiagnosisResult as DoctorDiagnosis,
    SafeNextStep, ProposedAction, ActionRisk,
    // Report
    DoctorReport,
    // Runner
    DoctorRunner,
    // Network evidence helpers
    NetInterfaceSummary, RouteSummary, DnsSummary,
    NmSummary, WifiSummary, NetworkErrorsSummary,
};

// v0.0.49: NetworkingDoctor v2 (Doctor trait implementation)
pub use networking_doctor_v2::NetworkingDoctorV2;

// v0.0.50: User File Mutation
pub use user_file_mutation::{
    // Edit action and mode
    UserFileEditAction, EditMode, VerifyStrategy,
    // Path policy
    PathPolicyResult, check_path_policy,
    // Preview
    EditPreview, FileStat, generate_edit_preview,
    // Apply
    ApplyResult, apply_edit,
    // Rollback
    RollbackResult as UserFileRollbackResult, execute_rollback as execute_user_file_rollback,
    // Helpers
    generate_mutation_case_id as generate_user_file_case_id, USER_FILE_CONFIRMATION,
};

// v0.0.50: File Edit Tool Executors
pub use file_edit_tools::{
    execute_file_edit_preview_v1,
    execute_file_edit_apply_v1,
    execute_file_edit_rollback_v1,
};

// v0.0.51: Systemd Action Engine (modular)
pub use systemd_action::{
    // Operations
    ServiceOperation, ServiceAction, RiskLevel,
    // Confirmation phrases (MEDIUM_RISK_CONFIRMATION is already exported from mutation_tools)
    LOW_RISK_CONFIRMATION, HIGH_RISK_CONFIRMATION,
    // Risk assessment
    assess_risk, normalize_service_name,
};
pub use systemd_probe::{ServiceProbe, probe_service, MAX_STATUS_LINES};
pub use systemd_apply::{
    ServicePreview, preview_service_action,
    ServiceApplyResult, ServiceStateSnapshot, apply_service_action,
};
pub use systemd_rollback::{
    ServiceRollbackResult, rollback_service_action,
    generate_service_case_id, ROLLBACK_BASE,
};

// v0.0.52: System Query Router
pub use system_query_router::{
    QueryTarget, ToolRouting,
    detect_target, get_tool_routing, validate_answer_for_target,
    map_translator_targets, get_required_tools,
};

// v0.0.51: Systemd Tool Executors
pub use systemd_tools::{
    execute_systemd_service_probe_v1,
    execute_systemd_service_preview_v1,
    execute_systemd_service_apply_v1,
    execute_systemd_service_rollback_v1,
};

// v0.0.53: Doctor Flow
pub use doctor_flow::{
    detect_problem_phrase,
    DoctorFlowStep, StepStatus,
    DoctorFlowResult, DoctorFlowExecutor,
    DoctorCaseFile, CaseStep, CaseSuggestedAction,
};

// v0.0.54: Action Engine
pub use action_engine::{
    MutationRiskLevel, ActionPlan, ActionStep, ActionType, ActionResult,
    StepResult as ActionStepResult, PlanStatus,
    StepStatus as ActionStepStatus,
    EditFileAction, EditIntent, WriteFileAction, DeleteFileAction,
    SystemdAction as ActionSystemdAction, SystemdOperation as ActionSystemdOp,
    PacmanAction, PacmanOperation, PackageReason,
    ActionDiffPreview, ActionDiffLine, DiffLineType as ActionDiffLineType,
    RollbackHint as ActionRollbackHint, RollbackRecord, RollbackStepRecord, BackupRecord, VerificationRecord,
    CONFIRM_LOW, CONFIRM_MEDIUM, CONFIRM_HIGH, CONFIRM_DESTRUCTIVE,
};
pub use action_risk::{
    score_action_risk, score_path_risk, score_systemd_risk, score_package_risk,
    score_delete_risk, describe_risk,
};
pub use action_executor::{
    generate_action_diff_preview, validate_confirmation as validate_action_confirmation,
    execute_step as execute_action_step, execute_plan as execute_action_plan,
    compute_hash as action_compute_hash, get_backup_path as action_get_backup_path,
    backup_file as action_backup_file,
};

// v0.0.55: Case Engine + Intent Taxonomy
pub use case_engine::{
    CasePhase, IntentType, CaseEvent, CaseActor, CaseEventType,
    CaseState, PhaseTiming,
};
pub use intent_taxonomy::{
    IntentClassification, classify_intent, domain_to_doctor,
};
pub use evidence_tools::{
    EvidencePlan, PlannedTool, plan_evidence, validate_evidence_for_query,
};
pub use case_file_v1::{
    CaseFileV1, EvidenceRecordV1, PhaseTimingRecord,
    load_case, list_recent_case_ids,
    CASE_SCHEMA_VERSION, CASE_FILES_DIR,
};
pub use recipe_extractor::{
    RecipeExtractionResult, check_recipe_gate, check_state_recipe_gate,
    extract_recipe, calculate_case_xp,
    MIN_RELIABILITY_FOR_RECIPE as RECIPE_MIN_RELIABILITY,
    MIN_EVIDENCE_FOR_RECIPE as RECIPE_MIN_EVIDENCE,
};
pub use transcript_render::{
    render_transcript_from_state, render_transcript_from_file,
    render_compact_summary, render_recent_cases,
    wrap_text as transcript_wrap_text, truncate as transcript_truncate,
};

// v0.0.56: Fly-on-the-Wall Dialogue Layer
pub use dialogue_renderer::{
    DialogueContext, render_dialogue_transcript,
    phase_separator, doctor_actor_name,
    render_translator_classification, render_anna_translator_response,
    render_doctor_handoff, render_evidence_summary, render_evidence_item,
    render_junior_verification, render_anna_junior_response,
    render_final_response, render_reliability_footer,
    // v0.0.57: Coverage display in transcript
    render_junior_coverage_check, render_anna_coverage_retry,
    RELIABILITY_SHIP_THRESHOLD, CONFIDENCE_CERTAIN_THRESHOLD,
};

// v0.0.57: Evidence Coverage + Junior Rubric
pub use evidence_coverage::{
    TargetFacets, EvidenceCoverage,
    get_target_facets, analyze_coverage, check_evidence_mismatch,
    get_gap_filling_tools, calculate_coverage_penalty, get_max_score_for_coverage,
    COVERAGE_SUFFICIENT_THRESHOLD, COVERAGE_PENALTY_THRESHOLD,
};
pub use junior_rubric::{
    VerificationResult, Penalty,
    verify_answer, is_clearly_wrong_evidence, get_evidence_suggestions,
    SHIP_IT_THRESHOLD, BASE_SCORE,
    WRONG_EVIDENCE_PENALTY, MISSING_EVIDENCE_PENALTY, ANSWER_MISMATCH_PENALTY,
    UNCITED_CLAIM_PENALTY, HIGH_COVERAGE_BONUS,
};

// v0.0.58: Proactive Monitoring Loop v1
pub use proactive_alerts::{
    AlertType,
    AlertSeverity as ProactiveAlertSeverity,  // Renamed to avoid conflict with reliability::AlertSeverity
    AlertStatus,
    ProactiveAlert, ProactiveAlertsState, AlertCounts,
    PROACTIVE_ALERTS_SCHEMA, PROACTIVE_ALERTS_FILE,
};
pub use alert_detectors::{
    // Evidence types
    BootRegressionEvidence, DiskPressureEvidence, JournalErrorBurstEvidence,
    ServiceFailedEvidence, ThermalThrottlingEvidence,
    // Thresholds
    BOOT_REGRESSION_STDDEV_FACTOR, BOOT_REGRESSION_MIN_DELTA_SECS,
    DISK_PRESSURE_WARNING_PERCENT, DISK_PRESSURE_WARNING_GIB,
    DISK_PRESSURE_CRITICAL_PERCENT, DISK_PRESSURE_CRITICAL_GIB,
    JOURNAL_ERROR_BURST_WARNING, JOURNAL_ERROR_BURST_CRITICAL,
    JOURNAL_ERROR_BURST_WINDOW_MINS,
    THERMAL_WARNING_TEMP, THERMAL_CRITICAL_TEMP,
    // Detectors
    detect_boot_regression, detect_disk_pressure, detect_journal_error_burst,
    detect_service_failed, detect_thermal_throttling, run_all_detectors,
};
pub use alert_probes::{
    // Probe results
    BootTimeSummary, DiskPressureSummary, JournalErrorBurstSummary,
    FailedUnitsSummary, ThermalSummary, AlertsSummary,
    AlertCountsData, AlertSummaryEntry,
    // Probes
    probe_boot_time_summary, probe_disk_pressure_summary,
    probe_journal_error_burst_summary, probe_failed_units_summary,
    probe_thermal_summary, probe_alerts_summary,
};

// v0.0.59: Auto-Case Opening + Departmental IT Org
pub use case_lifecycle::{
    CaseStatus, CaseFileV2, Department, Participant,
    ActionRisk as CaseActionRisk,  // Renamed to avoid conflict
    ProposedAction as CaseProposedAction,  // Renamed to avoid conflict
    TimelineEvent, TimelineEventType,
    CASE_SCHEMA_VERSION_V2,
    load_case_v2, list_active_cases, count_active_cases,
};
pub use service_desk::{
    TriageResult, triage_request, dispatch_to_specialist,
    should_auto_open_case, find_case_for_alert, open_case_for_alert,
    progress_case_triage, progress_case_investigating, progress_case_plan_ready,
};
pub use transcript_v2::{
    TranscriptLine,
    TranscriptBuilder as TranscriptBuilderV2,  // Renamed to avoid conflict
    DepartmentOutput,
    Hypothesis as DepartmentHypothesis,  // Renamed to avoid conflict
    render_case_transcript, render_handoff, render_junior_disagreement,
    render_collaboration, render_active_cases_status,
};
