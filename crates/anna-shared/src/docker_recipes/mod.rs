//! Docker Compose recipes (v0.0.235).
//!
//! Recipes for creating and managing Docker Compose services.
//! Covers creating compose files, managing services, debugging, and cleanup.

mod matcher;
mod recipes;
#[cfg(test)]
mod tests;
mod types;

// Re-export all types and functions
pub use matcher::{detect_feature, match_query};
pub use recipes::builtin_recipes;
pub use types::{DockerFeature, DockerRecipe};
