//! Anna Common v5.5.2 - Telemetry Core
//!
//! Pure system intelligence: inventory, telemetry, correlation.
//! No LLM, no Q&A, no conversational features.
//!
//! v5.5.2: Added atomic file writes for data integrity
//!
//! Modules:
//! - atomic_write: Atomic file write operations (v5.5.2)
//! - config: System configuration
//! - display_format: Output formatting utilities
//! - error_index: Log scanning and error aggregation
//! - intrusion: Security event detection
//! - knowledge_core: Object inventory and classification
//! - knowledge_collector: System discovery (packages, binaries, services)
//! - object_metadata: Static descriptions and relationships
//! - service_state: Systemd service tracking
//! - telemetry: Process monitoring and usage tracking

// Core modules
pub mod atomic_write;
pub mod config;
pub mod display_format;
pub mod error_index;
pub mod intrusion;
pub mod knowledge_collector;
pub mod knowledge_core;
pub mod object_metadata;
pub mod service_state;
pub mod telemetry;

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
