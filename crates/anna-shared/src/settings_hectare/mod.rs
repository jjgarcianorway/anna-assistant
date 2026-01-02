// v0.0.761: Settings Hectare (Phase 337)
// Land hectare for settings metric area

mod types;
mod config;
mod record;
mod stats;
mod hectare;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{HectareType, HectareStatus};
pub use config::HectareConfig;
pub use record::{HectareRecord, HectareInspector};
pub use stats::HectareStats;
pub use hectare::SettingsHectare;
pub use registry::HectareRegistry;
pub use utils::{format_hectare_registry, is_hectare_query, hectare_fun_fact};
