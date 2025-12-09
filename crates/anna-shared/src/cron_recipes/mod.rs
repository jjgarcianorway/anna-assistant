//! Cron job recipes (v0.0.234).
//!
//! Recipes for creating and managing cron jobs.
//! Covers scheduling, syntax, debugging, and common tasks.

mod matcher;
mod recipes;
#[cfg(test)]
mod tests;
mod types;

// Re-export all types and functions
pub use matcher::{detect_feature, match_query};
pub use recipes::builtin_recipes;
pub use types::{CronFeature, CronPreset, CronRecipe};
