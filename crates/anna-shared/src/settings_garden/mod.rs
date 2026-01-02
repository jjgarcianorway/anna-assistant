// v0.0.768: Settings Garden (Phase 344)
// Cultivated garden for settings horticulture

mod types;
mod models;
mod garden;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve API
pub use types::{GardenType, GardenStatus};
pub use models::{GardenConfig, GardenPlant, GardenGardener, GardenStats};
pub use garden::SettingsGarden;
pub use registry::GardenRegistry;
pub use utils::{format_garden_registry, is_garden_query, garden_fun_fact};
