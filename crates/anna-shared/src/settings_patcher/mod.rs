// v0.0.662: Settings Patcher Module (Phase 238)
// Patcher for applying incremental changes to settings

pub mod config;
pub mod entry;
pub mod patcher;
pub mod registry;
pub mod result;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export all public items for backward compatibility
pub use config::PatcherConfig;
pub use entry::PatchEntry;
pub use patcher::SettingsPatcher;
pub use registry::{format_patcher_registry, is_patcher_query, patcher_fun_fact, SettingsPatcherRegistry};
pub use result::{PatchResult, PatcherStats};
pub use types::{PatchMode, PatchOperation};
