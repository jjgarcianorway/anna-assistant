// v0.0.561: Settings Diff (Phase 137)
// Compares two settings objects and reports differences

mod types;
mod differ;
mod formatting;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{DiffType, DiffEntry, SettingsDiff};
pub use differ::{SettingsDiffer, diff_settings};
pub use formatting::{format_diff, settings_diff_fun_fact};
