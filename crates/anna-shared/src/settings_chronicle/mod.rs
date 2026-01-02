// v0.0.692: Settings Chronicle Module (Phase 268)
// Track settings changes over time

mod types;
mod config;
mod record;
mod history;
mod stats;
mod chronicle;
mod registry;
mod helpers;

// Re-export all public types to maintain the same API
pub use types::{ChronicleEvent, ChronicleMode};
pub use config::ChronicleConfig;
pub use record::ChronicleRecord;
pub use history::ChronicleHistory;
pub use stats::ChronicleStats;
pub use chronicle::SettingsChronicle;
pub use registry::ChronicleRegistry;
pub use helpers::{format_chronicle_registry, is_chronicle_query, chronicle_fun_fact};
