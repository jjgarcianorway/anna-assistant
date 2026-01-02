// v0.0.719: Settings Edict (Phase 295)
// Formal edicts for settings enforcement

mod types;
mod config;
mod proclamation;
mod stats;
mod edict;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{EdictType, EdictStatus};
pub use config::EdictConfig;
pub use proclamation::{EdictProclamation, EdictAnnotation};
pub use stats::EdictStats;
pub use edict::SettingsEdict;
pub use registry::EdictRegistry;
pub use utils::{format_edict_registry, is_edict_query, edict_fun_fact};
