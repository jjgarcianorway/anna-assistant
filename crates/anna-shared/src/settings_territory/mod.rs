// v0.0.745: Settings Territory (Phase 321)
// Controlled territory for settings administration

mod types;
mod config;
mod ordinance;
mod administrator;
mod stats;
mod territory;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use types::{TerritoryType, TerritoryStatus};
pub use config::TerritoryConfig;
pub use ordinance::TerritoryOrdinance;
pub use administrator::TerritoryAdministrator;
pub use stats::TerritoryStats;
pub use territory::SettingsTerritory;
pub use registry::TerritoryRegistry;
pub use utils::{format_territory_registry, is_territory_query, territory_fun_fact};
