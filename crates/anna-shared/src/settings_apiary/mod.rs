// v0.0.779: Settings Apiary (Phase 355)
// Bee apiary for settings apiculture

mod types;
mod config;
mod hive;
mod beekeeper;
mod stats;
mod apiary;
mod registry;
mod utils;

// Re-export all public types
pub use types::{ApiaryType, ApiaryStatus};
pub use config::ApiaryConfig;
pub use hive::ApiaryHive;
pub use beekeeper::ApiaryBeekeeper;
pub use stats::ApiaryStats;
pub use apiary::SettingsApiary;
pub use registry::ApiaryRegistry;
pub use utils::{format_apiary_registry, is_apiary_query, apiary_fun_fact};
