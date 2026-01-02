// v0.0.657: Settings Cloner Module (Phase 233)
// Modular organization for settings cloning functionality

mod cloner;
mod registry;
mod result;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve API compatibility
pub use cloner::SettingsCloner;
pub use registry::{format_cloner_registry, SettingsClonerRegistry};
pub use result::{CloneResult, ClonerStats};
pub use types::{CloneDepth, CloneMod, CloneMode, ClonerConfig};
pub use utils::{cloner_fun_fact, is_cloner_query};
