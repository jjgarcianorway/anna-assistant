// v0.0.782: Settings Reserve (Phase 358)
// Nature reserve for settings preservation

mod types;
mod config;
mod species;
mod ranger;
mod stats;
mod reserve;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{ReserveType, ReserveStatus};
pub use config::ReserveConfig;
pub use species::ReserveSpecies;
pub use ranger::ReserveRanger;
pub use stats::ReserveStats;
pub use reserve::SettingsReserve;
pub use registry::ReserveRegistry;
pub use utils::{format_reserve_registry, is_reserve_query, reserve_fun_fact};
