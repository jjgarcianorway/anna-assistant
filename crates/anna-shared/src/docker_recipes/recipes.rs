//! Built-in Docker Compose recipes (v0.0.235).

use super::types::DockerRecipe;

// Re-export recipe functions from sibling modules
pub use super::recipes_compose::compose_recipes;
pub use super::recipes_images::image_recipes;
pub use super::recipes_monitoring::monitoring_recipes;
pub use super::recipes_operations::operation_recipes;

/// Get built-in Docker Compose recipes
pub fn builtin_recipes() -> Vec<DockerRecipe> {
    let mut recipes = Vec::new();

    // Add compose management recipes
    recipes.extend(compose_recipes());

    // Add monitoring recipes
    recipes.extend(monitoring_recipes());

    // Add image management recipes
    recipes.extend(image_recipes());

    // Add operation recipes
    recipes.extend(operation_recipes());

    recipes
}
