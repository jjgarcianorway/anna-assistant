// v0.0.689: Settings Comparer Module (Phase 265)
// Compare two settings collections

mod types;
mod config;
mod result;
mod comparer;
mod registry;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{CompareMode, DiffType, DiffEntry};
pub use config::ComparerConfig;
pub use result::{CompareResult, ComparerStats};
pub use comparer::SettingsComparer;
pub use registry::{ComparerRegistry, format_comparer_registry, is_comparer_query, comparer_fun_fact};
