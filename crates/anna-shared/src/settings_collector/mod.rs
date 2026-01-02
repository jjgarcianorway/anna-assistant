// v0.0.682: Settings Collector Module (Phase 258)
// Collect settings from multiple sources

mod types;
mod config;
mod source;
mod result;
mod collector;
mod registry;
mod utils;

// Re-export all public types to preserve the API
pub use types::{CollectMode, SourcePriority};
pub use config::CollectorConfig;
pub use source::SettingsSource;
pub use result::{CollectResult, CollectorStats};
pub use collector::SettingsCollector;
pub use registry::CollectorRegistry;
pub use utils::{format_collector_registry, is_collector_query, collector_fun_fact};
