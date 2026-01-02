// v0.0.775: Settings Aquarium (Phase 351)
// Aquatic aquarium for settings marine life

mod types;
mod config;
mod inhabitant;
mod stats;
mod aquarium;
mod registry;

// Re-export all public types to preserve API
pub use types::{AquariumType, AquariumStatus};
pub use config::AquariumConfig;
pub use inhabitant::{AquariumInhabitant, AquariumAquarist};
pub use stats::AquariumStats;
pub use aquarium::SettingsAquarium;
pub use registry::{AquariumRegistry, format_aquarium_registry, is_aquarium_query, aquarium_fun_fact};
