//! Tests for recipe runtime.

#[cfg(test)]
mod tests {
    use crate::recipe_schema::{PlanStep, Recipe, RecipeMatcher, RecipePattern, Precondition};
    use crate::recipe_runtime::{
        check_preconditions, generate_confirmation_prompt, ExecutionContext,
    };
    use std::collections::HashMap;

    fn make_test_recipe() -> Recipe {
        let mut recipe = Recipe::new(
            "test".into(),
            "desktop".into(),
            "configure_editor".into(),
            RecipePattern {
                user_goal: "enable syntax highlighting".into(),
                slots: HashMap::new(),
            },
            RecipeMatcher {
                required_keywords: vec!["vim".into()],
                optional_keywords: vec![],
                negative_keywords: vec![],
                min_confidence: 0.8,
                exact_intent: None,
            },
            vec![
                PlanStep::BackupFile {
                    path: "$HOME/.vimrc".into(),
                },
                PlanStep::AppendLine {
                    path: "$HOME/.vimrc".into(),
                    line: "syntax enable".into(),
                },
            ],
        );
        recipe.preconditions = vec![Precondition::FileExists {
            path: "$HOME/.vimrc".into(),
        }];
        recipe
    }

    #[test]
    fn test_expand_path() {
        let ctx = ExecutionContext::default();
        let expanded = super::super::utils::expand_path("$HOME/.vimrc", &ctx);
        assert!(!expanded.contains("$HOME"));
    }

    #[test]
    fn test_precondition_check() {
        let recipe = make_test_recipe();
        let ctx = ExecutionContext::default();

        // File may or may not exist, but function should work
        let result = check_preconditions(&recipe, &ctx);
        assert!(!result.passed.is_empty() || !result.failed.is_empty());
    }

    #[test]
    fn test_confirmation_prompt() {
        let recipe = make_test_recipe();
        let ctx = ExecutionContext::default();

        let prompt = generate_confirmation_prompt(&recipe, &ctx);
        assert!(prompt.contains("syntax enable"));
        assert!(prompt.contains("Apply this change"));
    }
}
