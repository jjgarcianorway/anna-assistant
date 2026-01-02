// v0.0.601: Settings Comparator Module (Phase 177)
// Compare settings between snapshots, profiles, or versions

mod comparator;
mod types;
mod utils;

// Re-export all public types and functions to preserve API
pub use comparator::SettingsComparator;
pub use types::{CompareMode, CompareOptions, CompareResult, DiffEntry, DiffType};
pub use utils::{comparator_fun_fact, format_compare_result, is_comparator_query};
