//! Docker Compose recipes (v0.0.235).
//!
//! Recipes for creating and managing Docker Compose services.
//! Covers creating compose files, managing services, debugging, and cleanup.

mod matcher;
mod recipes;
mod recipes_compose;
mod recipes_images;
mod recipes_monitoring;
mod recipes_operations;
#[cfg(test)]
mod tests;
mod types;

// Re-export all types and functions
pub use matcher::{detect_feature, match_query};
pub use recipes::builtin_recipes;
pub use types::{DockerFeature, DockerRecipe};

// Optionally re-export individual recipe functions for advanced use
pub use recipes::{compose_recipes, image_recipes, monitoring_recipes, operation_recipes};
