//! Recipe store tests (v0.0.232).

#[cfg(test)]
mod tests {
    use crate::recipe_store::{should_learn_recipe, Recipe, RecipeRisk, RecipeStore};

    #[test]
    fn test_recipe_builder() {
        let recipe = Recipe::new(
            "vim-syntax",
            "editor_config",
            "Enable Vim Syntax Highlighting",
        )
        .with_trigger("configure_editor")
        .with_evidence("tool_exists")
        .with_risk(RecipeRisk::ConfigChange);

        assert_eq!(recipe.triggers, vec!["configure_editor"]);
        assert_eq!(recipe.risk, RecipeRisk::ConfigChange);
    }

    #[test]
    fn test_recipe_matches() {
        let recipe = Recipe::new("test", "test", "Test")
            .with_trigger("memory_usage")
            .with_evidence("memory");

        assert!(recipe.matches("memory_usage", &["memory".to_string()]));
        assert!(!recipe.matches("disk_usage", &["memory".to_string()]));
        assert!(!recipe.matches("memory_usage", &["disk".to_string()]));
    }

    #[test]
    fn test_recipe_store_operations() {
        let mut store = RecipeStore::new();

        let recipe = Recipe::new("test-1", "test", "Test Recipe").with_trigger("test_query");

        store.add(recipe);

        assert_eq!(store.len(), 1);
        assert!(store.get("test-1").is_some());
        assert_eq!(store.find_matches("test_query", &[]).len(), 1);
        assert_eq!(store.find_matches("other_query", &[]).len(), 0);
    }

    #[test]
    fn test_should_learn_recipe() {
        assert!(should_learn_recipe(90, true, false));
        assert!(!should_learn_recipe(80, true, false)); // Too low
        assert!(!should_learn_recipe(90, false, false)); // Not deterministic
        assert!(!should_learn_recipe(90, true, true)); // Already exists
    }

    #[test]
    fn test_risk_requires_confirmation() {
        assert!(!RecipeRisk::ReadOnly.requires_confirmation());
        assert!(!RecipeRisk::ConfigChange.requires_confirmation());
        assert!(RecipeRisk::SystemChange.requires_confirmation());
        assert!(RecipeRisk::Destructive.requires_confirmation());
    }
}
