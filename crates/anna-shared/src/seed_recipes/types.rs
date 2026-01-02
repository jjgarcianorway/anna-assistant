//! Utility functions for seed recipes.

use crate::recipe_schema::Recipe;
use crate::recipe_storage::RecipeStorage;

/// Create and save all seed recipes.
pub fn install_seed_recipes(storage: &mut RecipeStorage) -> anyhow::Result<usize> {
    let recipes = create_seed_recipes();
    let mut installed = 0;

    for recipe in recipes {
        // Only install if not already present
        if !storage.exists(&recipe.id) {
            storage.save(&recipe)?;
            installed += 1;
        }
    }

    Ok(installed)
}

/// Create all seed recipes.
pub fn create_seed_recipes() -> Vec<Recipe> {
    vec![
        super::vim_recipe::create_vim_enable_syntax(),
        super::disk_recipe::create_check_disk_usage_root(),
        super::memory_recipes::create_check_free_ram(),
        super::memory_recipes::create_check_swap_status(),
        super::service_recipes::create_check_failed_services(),
        super::service_recipes::create_enable_sshd_service(),
    ]
}

/// Get a seed recipe by ID (for testing).
pub fn get_seed_recipe(id: &str) -> Option<Recipe> {
    create_seed_recipes().into_iter().find(|r| r.id == id)
}
