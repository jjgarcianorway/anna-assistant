// v0.0.776: Settings Vivarium (Phase 352)
// Living vivarium for settings animal habitat

mod types;
mod config;
mod creature;
mod keeper;
mod stats;
mod vivarium;
mod registry;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{VivariumType, VivariumStatus};
pub use config::VivariumConfig;
pub use creature::VivariumCreature;
pub use keeper::VivariumKeeper;
pub use stats::VivariumStats;
pub use vivarium::SettingsVivarium;
pub use registry::{VivariumRegistry, format_vivarium_registry, is_vivarium_query, vivarium_fun_fact};
