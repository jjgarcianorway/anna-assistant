// v0.0.767: Settings Vineyard (Phase 343)
// Grape vineyard for settings viticulture

mod types;
mod config;
mod vine;
mod vintner;
mod stats;
mod vineyard;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{VineyardType, VineyardStatus};
pub use config::VineyardConfig;
pub use vine::VineyardVine;
pub use vintner::VineyardVintner;
pub use stats::VineyardStats;
pub use vineyard::SettingsVineyard;
pub use registry::VineyardRegistry;
pub use utils::{format_vineyard_registry, is_vineyard_query, vineyard_fun_fact};
