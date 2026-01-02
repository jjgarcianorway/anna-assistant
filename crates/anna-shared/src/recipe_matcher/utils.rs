//! Utility functions for recipe matching.

use crate::recipe::recipe_dir;
use crate::recipe_index::RecipeIndex;

/// v0.0.104: Try to match an SSH-related query to builtin SSH recipes
pub fn match_ssh_recipe(query: &str) -> Option<&'static crate::ssh_recipes::SshRecipe> {
    crate::ssh_recipes::match_query(query)
}

/// Load recipe index from disk
pub fn load_recipe_index() -> RecipeIndex {
    RecipeIndex::build_from_disk()
}

/// Get recipes count
pub fn recipe_count() -> usize {
    let dir = recipe_dir();
    if !dir.exists() {
        return 0;
    }
    std::fs::read_dir(&dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0)
}
