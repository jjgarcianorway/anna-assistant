//! Anna Common v8.0.0 - Grounded System Intelligence
//!
//! v7.1.0: Real telemetry with SQLite storage
//! v7.5.0: Enhanced telemetry with CPU time, exec counts, hotspots
//! v7.6.0: Config maps, trends, Anna needs model
//! v7.6.1: Config hygiene - identity filtering, lean output, vim/nvim separation
//! v7.7.0: Precise per-window aggregation and compact display format
//! v8.0.0: Execution telemetry with per-object, per-day JSONL storage
//! - Every number has a verifiable source
//! - No invented descriptions
//! - No hallucinated metrics
//! - Per-process CPU/memory tracking in SQLite
//! - Per-execution JSONL logs with window aggregation
//! - Config discovery from man pages, pacman, Arch Wiki
//! - Anna needs tracking for missing tools
//!
//! Modules:
//! - grounded: Real data from real system commands
//! - atomic_write: Atomic file write operations
//! - config: System configuration
//! - display_format: Output formatting utilities
//! - error_index: Log scanning and error aggregation
//! - intrusion: Security event detection
//! - knowledge_core: Object inventory and classification
//! - knowledge_collector: System discovery
//! - needs: Anna's tool and doc dependencies (v7.6.0+)
//! - object_metadata: Static descriptions and relationships
//! - service_state: Systemd service tracking
//! - telemetry: Process monitoring and usage tracking (log files)
//! - telemetry_db: SQLite-based telemetry storage (v7.1.0+)
//! - telemetry_exec: Per-object, per-day JSONL execution logs (v8.0.0+)

// v6.0.0: Grounded knowledge system - every fact has a source
pub mod grounded;

// Core modules
pub mod atomic_write;
pub mod config;
pub mod display_format;
pub mod error_index;
pub mod intrusion;
pub mod knowledge_collector;
pub mod knowledge_core;
pub mod needs;
pub mod object_metadata;
pub mod service_state;
pub mod telemetry;
pub mod telemetry_db;
pub mod telemetry_exec;

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
pub use telemetry_db::{
    TelemetryDb, ProcessTelemetrySample, ObjectTelemetry, TelemetryStats,
    SampleCounts, UsageStats, GlobalPeak, DataStatus, MaintenanceResult,
    EnhancedUsageStats, EnhancedWindowedStats, TopProcessEntry,
    HealthHotspot, TelemetryHealth,
    WindowStats, AllWindowStats, TopCompactEntry, format_cpu_time_compact,
    // v7.7.0: Trend and window status types
    Trend, TrendData, WindowStatusInfo, TopHighlightEntry,
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
