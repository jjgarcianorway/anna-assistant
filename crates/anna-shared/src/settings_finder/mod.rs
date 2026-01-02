// v0.0.685: Settings Finder (Phase 261)
// Find settings by various criteria

mod types;
mod config;
mod stats;
mod finder;
mod registry;
mod utils;

// Re-export all public types and functions to preserve the original API
pub use types::{FindMode, FindLimit, FoundItem, FindResult};
pub use config::FinderConfig;
pub use stats::FinderStats;
pub use finder::SettingsFinder;
pub use registry::FinderRegistry;
pub use utils::{format_finder_registry, is_finder_query, finder_fun_fact};
