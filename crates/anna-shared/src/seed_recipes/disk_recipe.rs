//! Recipe for checking disk usage.

use crate::recipe_schema::{
    ConfirmationPolicy, PlanStep, Precondition, Recipe, RecipeMatcher, RecipePattern, RecipeStatus,
    SuccessCriteria,
};
use std::collections::HashMap;

/// Recipe: Check disk usage on root partition.
pub fn create_check_disk_usage_root() -> Recipe {
    let mut recipe = Recipe::new(
        "check_disk_usage_root".into(),
        "storage".into(),
        "check_disk_usage".into(),
        RecipePattern {
            user_goal: "check disk usage on root partition".into(),
            slots: HashMap::from([("partition".into(), "/".into())]),
        },
        RecipeMatcher {
            required_keywords: vec!["disk".into()],
            optional_keywords: vec![
                "usage".into(),
                "space".into(),
                "root".into(),
                "full".into(),
                "free".into(),
                "how much".into(),
            ],
            negative_keywords: vec![],
            min_confidence: 0.7,
            exact_intent: Some("check_disk_usage".into()),
        },
        vec![
            PlanStep::Explain {
                message: "Checking disk usage using df command.".into(),
            },
            PlanStep::VerifyCommand {
                command: "df -h /".into(),
                expect_success: true,
            },
        ],
    );

    recipe.preconditions = vec![Precondition::ToolExists { tool: "df".into() }];
    recipe.confirmation_policy = ConfirmationPolicy::Never; // Read-only
    recipe.success_criteria = SuccessCriteria {
        must_succeed: vec!["verify_command".into()],
        rollback_on_failure: false,
        post_verification: None,
    };
    recipe.citations = vec!["man:df(1)".into(), "archwiki:File_systems".into()];
    recipe.status = RecipeStatus::Active;

    recipe
}
