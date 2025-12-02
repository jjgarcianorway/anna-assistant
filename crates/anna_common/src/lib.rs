//! Anna Common v7.23.0 - Timelines, Drift & Incidents
//!
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
