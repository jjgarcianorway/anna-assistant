// v0.0.780: Settings Butterfly (Phase 356)
// Butterfly house for settings lepidopterology

mod types;
mod config;
mod specimen;
mod stats;
mod butterfly;
mod registry;
mod utils;

// Re-export public API
pub use types::{ButterflyType, ButterflyStatus};
pub use config::ButterflyConfig;
pub use specimen::{ButterflySpecimen, ButterflyCurator};
pub use stats::ButterflyStats;
pub use butterfly::SettingsButterfly;
pub use registry::ButterflyRegistry;
pub use utils::{format_butterfly_registry, is_butterfly_query, butterfly_fun_fact};
