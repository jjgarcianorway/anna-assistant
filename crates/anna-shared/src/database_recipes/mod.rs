//! Database recipes (v0.0.461).
//!
//! Recipes for database backup, restore, and management.
//! Covers PostgreSQL, MySQL/MariaDB, SQLite, MongoDB, and Redis.
//!
//! v0.0.461: Initial implementation per ROADMAP.md Future section.

mod matcher;
mod recipes;
#[cfg(test)]
mod tests;
mod types;

// Re-export all types and functions
pub use matcher::{detect_feature, match_query};
pub use recipes::builtin_recipes;
pub use types::{DatabaseFeature, DatabaseRecipe, DatabaseType};
