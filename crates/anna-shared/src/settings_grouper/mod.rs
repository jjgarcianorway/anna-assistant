// v0.0.676: Settings Grouper (Phase 252)
// Group settings by various criteria

mod types;
mod config;
mod group;
mod stats;
mod grouper;
mod registry;
mod utils;

// Re-export public API
pub use types::{GroupByField, ValueTypeClass, classify_value};
pub use config::GrouperConfig;
pub use group::{SettingsGroup, GroupResult};
pub use stats::GrouperStats;
pub use grouper::SettingsGrouper;
pub use registry::{GrouperRegistry, format_grouper_registry};
pub use utils::{is_grouper_query, grouper_fun_fact};
