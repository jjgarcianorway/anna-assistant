// v0.0.774: Settings Herbarium (Phase 350)
// Plant herbarium for settings taxonomy

mod types;
mod config;
mod specimen;
mod taxonomist;
mod stats;
mod herbarium;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{HerbariumType, HerbariumStatus};
pub use config::HerbariumConfig;
pub use specimen::HerbariumSpecimen;
pub use taxonomist::HerbariumTaxonomist;
pub use stats::HerbariumStats;
pub use herbarium::SettingsHerbarium;
pub use registry::HerbariumRegistry;
pub use utils::{format_herbarium_registry, is_herbarium_query, herbarium_fun_fact};
