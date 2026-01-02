//! Seed recipes for Anna's learning system.
//! v0.0.418: Initial recipes that demonstrate the recipe system.
//!
//! These are 6 concrete recipes:
//! 1. vim_enable_syntax - Config change recipe
//! 2. check_disk_usage_root - Repeatable diagnostic recipe
//! 3. check_free_ram - Repeatable diagnostic recipe
//! 4. check_swap_status - Repeatable diagnostic recipe
//! 5. check_failed_services - Repeatable diagnostic recipe
//! 6. enable_sshd_service - Service management recipe

mod disk_recipe;
mod memory_recipes;
mod service_recipes;
mod types;
mod vim_recipe;

// Re-export public API
pub use types::{create_seed_recipes, get_seed_recipe, install_seed_recipes};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_schema::ConfirmationPolicy;
    use crate::recipe_storage::RecipeStorage;
    use tempfile::tempdir;

    #[test]
    fn test_seed_recipes_valid() {
        let recipes = create_seed_recipes();
        assert!(recipes.len() >= 3);

        for recipe in recipes {
            assert!(!recipe.id.is_empty());
            assert!(!recipe.plan.is_empty());
            assert!(!recipe.matcher.required_keywords.is_empty());
            assert!(recipe.matcher.min_confidence > 0.0);
        }
    }

    #[test]
    fn test_install_seed_recipes() {
        let dir = tempdir().unwrap();
        let mut storage = RecipeStorage::with_dirs(dir.path().join("user"), dir.path().join("sys"));

        let installed = install_seed_recipes(&mut storage).unwrap();
        assert!(installed >= 3);

        // Installing again should install 0 (already present)
        let installed2 = install_seed_recipes(&mut storage).unwrap();
        assert_eq!(installed2, 0);
    }

    #[test]
    fn test_vim_recipe_structure() {
        let recipe = get_seed_recipe("vim_enable_syntax").unwrap();

        assert_eq!(recipe.domain, "desktop");
        assert!(recipe
            .matcher
            .required_keywords
            .contains(&"vim".to_string()));
        assert!(recipe
            .matcher
            .required_keywords
            .contains(&"syntax".to_string()));
        assert!(recipe
            .matcher
            .negative_keywords
            .contains(&"neovim".to_string()));
        assert!(!recipe.citations.is_empty());
    }

    #[test]
    fn test_diagnostic_recipes_read_only() {
        let disk_recipe = get_seed_recipe("check_disk_usage_root").unwrap();
        let ram_recipe = get_seed_recipe("check_free_ram").unwrap();

        // Diagnostic recipes should not require confirmation
        assert_eq!(disk_recipe.confirmation_policy, ConfirmationPolicy::Never);
        assert_eq!(ram_recipe.confirmation_policy, ConfirmationPolicy::Never);

        // Should not have mutating steps
        assert!(!disk_recipe.has_mutating_steps());
        assert!(!ram_recipe.has_mutating_steps());
    }
}
