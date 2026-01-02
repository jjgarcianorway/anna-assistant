// v0.0.661: Settings Differ Module (Phase 237)
// Differ for comparing settings configurations

mod types;
mod config;
mod entry;
mod result;
mod stats;
mod differ;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to maintain API compatibility
pub use types::{DiffType, DiffMode};
pub use config::DifferConfig;
pub use entry::DiffEntry;
pub use result::DiffResult;
pub use stats::DifferStats;
pub use differ::SettingsDiffer;
pub use registry::{SettingsDifferRegistry, format_differ_registry};
pub use utils::{is_differ_query, differ_fun_fact};
