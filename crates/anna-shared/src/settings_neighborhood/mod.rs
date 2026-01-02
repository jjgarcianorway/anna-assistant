// v0.0.754: Settings Neighborhood (Phase 330)
// Residential neighborhood for settings community

mod types;
mod config;
mod initiative;
mod stats;
mod neighborhood;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to maintain API compatibility
pub use types::{NeighborhoodType, NeighborhoodStatus};
pub use config::NeighborhoodConfig;
pub use initiative::{NeighborhoodInitiative, NeighborhoodOrganizer};
pub use stats::NeighborhoodStats;
pub use neighborhood::SettingsNeighborhood;
pub use registry::NeighborhoodRegistry;
pub use utils::{format_neighborhood_registry, is_neighborhood_query, neighborhood_fun_fact};
