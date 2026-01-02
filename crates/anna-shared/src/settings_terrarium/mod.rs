// v0.0.777: Settings Terrarium (Phase 353)
// Enclosed terrarium for settings miniature ecosystem

mod types;
mod config;
mod plant;
mod stats;
mod terrarium;
mod registry;

// Re-export all public types and functions
pub use types::{TerrariumType, TerrariumStatus};
pub use config::TerrariumConfig;
pub use plant::{TerrariumPlant, TerrariumCreator};
pub use stats::TerrariumStats;
pub use terrarium::SettingsTerrarium;
pub use registry::{TerrariumRegistry, format_terrarium_registry, is_terrarium_query, terrarium_fun_fact};
