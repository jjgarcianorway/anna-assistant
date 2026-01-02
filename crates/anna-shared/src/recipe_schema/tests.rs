//! Tests for recipe schema.

use std::collections::HashMap;

use super::{PlanStep, Recipe, RecipeMatcher, RecipePattern, RecipeStatus};

#[test]
fn test_recipe_creation() {
    let recipe = Recipe::new(
        "test_recipe".into(),
        "desktop".into(),
        "configure_editor".into(),
        RecipePattern {
            user_goal: "enable syntax highlighting".into(),
            slots: HashMap::new(),
        },
        RecipeMatcher {
            required_keywords: vec!["vim".into(), "syntax".into()],
            optional_keywords: vec![],
            negative_keywords: vec![],
            min_confidence: 0.8,
            exact_intent: None,
        },
        vec![PlanStep::Explain {
            message: "Test".into(),
        }],
    );
    assert!(recipe.is_usable());
    assert_eq!(recipe.version, 1);
}

#[test]
fn test_auto_disable() {
    let mut recipe = Recipe::new(
        "test".into(),
        "test".into(),
        "test".into(),
        RecipePattern {
            user_goal: "test".into(),
            slots: HashMap::new(),
        },
        RecipeMatcher {
            required_keywords: vec![],
            optional_keywords: vec![],
            negative_keywords: vec![],
            min_confidence: 0.8,
            exact_intent: None,
        },
        vec![],
    );
    // Simulate 10 uses with 4 failures
    for _ in 0..6 {
        recipe.record_success();
    }
    for _ in 0..4 {
        recipe.record_failure();
    }
    assert_eq!(recipe.status, RecipeStatus::Disabled);
}
