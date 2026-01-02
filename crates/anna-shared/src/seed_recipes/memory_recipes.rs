//! Recipes for checking memory and swap status.

use crate::recipe_schema::{
    ConfirmationPolicy, PlanStep, Precondition, Recipe, RecipeMatcher, RecipePattern, RecipeStatus,
    SuccessCriteria,
};
use std::collections::HashMap;

/// Recipe: Check free RAM.
pub fn create_check_free_ram() -> Recipe {
    let mut recipe = Recipe::new(
        "check_free_ram".into(),
        "performance".into(),
        "check_free_ram".into(),
        RecipePattern {
            user_goal: "check how much free RAM is available".into(),
            slots: HashMap::new(),
        },
        RecipeMatcher {
            required_keywords: vec!["ram".into()],
            optional_keywords: vec![
                "memory".into(),
                "free".into(),
                "available".into(),
                "how much".into(),
                "check".into(),
            ],
            negative_keywords: vec!["swap".into()],
            min_confidence: 0.7,
            exact_intent: Some("check_free_ram".into()),
        },
        vec![
            PlanStep::Explain {
                message: "Checking memory usage from /proc/meminfo.".into(),
            },
            PlanStep::VerifyCommand {
                command: "free -h".into(),
                expect_success: true,
            },
        ],
    );

    recipe.preconditions = vec![Precondition::FileExists {
        path: "/proc/meminfo".into(),
    }];
    recipe.confirmation_policy = ConfirmationPolicy::Never; // Read-only
    recipe.success_criteria = SuccessCriteria {
        must_succeed: vec!["verify_command".into()],
        rollback_on_failure: false,
        post_verification: None,
    };
    recipe.citations = vec!["man:free(1)".into(), "man:proc(5)".into()];
    recipe.status = RecipeStatus::Active;

    recipe
}

/// Recipe: Check swap status.
pub fn create_check_swap_status() -> Recipe {
    let mut recipe = Recipe::new(
        "check_swap_status".into(),
        "performance".into(),
        "check_swap".into(),
        RecipePattern {
            user_goal: "check if swap is enabled and how much".into(),
            slots: HashMap::new(),
        },
        RecipeMatcher {
            required_keywords: vec!["swap".into()],
            optional_keywords: vec![
                "enabled".into(),
                "active".into(),
                "status".into(),
                "check".into(),
                "how much".into(),
            ],
            negative_keywords: vec!["disable".into(), "off".into()],
            min_confidence: 0.7,
            exact_intent: Some("check_swap".into()),
        },
        vec![
            PlanStep::Explain {
                message: "Checking swap status using swapon command.".into(),
            },
            PlanStep::VerifyCommand {
                command: "swapon --show".into(),
                expect_success: true,
            },
        ],
    );

    recipe.preconditions = vec![];
    recipe.confirmation_policy = ConfirmationPolicy::Never;
    recipe.success_criteria = SuccessCriteria {
        must_succeed: vec![],
        rollback_on_failure: false,
        post_verification: None,
    };
    recipe.citations = vec!["man:swapon(8)".into(), "archwiki:Swap".into()];
    recipe.status = RecipeStatus::Active;

    recipe
}
