//! Recipes for managing systemd services.

use crate::recipe_schema::{
    ConfirmationPolicy, PlanStep, Precondition, Recipe, RecipeMatcher, RecipePattern, RecipeStatus,
    SuccessCriteria,
};
use std::collections::HashMap;

/// Recipe: Check failed systemd services.
pub fn create_check_failed_services() -> Recipe {
    let mut recipe = Recipe::new(
        "check_failed_services".into(),
        "services".into(),
        "check_failed_services".into(),
        RecipePattern {
            user_goal: "check for failed systemd services".into(),
            slots: HashMap::new(),
        },
        RecipeMatcher {
            required_keywords: vec!["failed".into()],
            optional_keywords: vec![
                "services".into(),
                "systemd".into(),
                "units".into(),
                "check".into(),
                "any".into(),
            ],
            negative_keywords: vec!["fix".into(), "restart".into()],
            min_confidence: 0.7,
            exact_intent: Some("check_failed_services".into()),
        },
        vec![
            PlanStep::Explain {
                message: "Checking for failed systemd services.".into(),
            },
            PlanStep::VerifyCommand {
                command: "systemctl --failed --no-pager".into(),
                expect_success: true,
            },
        ],
    );

    recipe.preconditions = vec![Precondition::ToolExists {
        tool: "systemctl".into(),
    }];
    recipe.confirmation_policy = ConfirmationPolicy::Never;
    recipe.success_criteria = SuccessCriteria {
        must_succeed: vec![],
        rollback_on_failure: false,
        post_verification: None,
    };
    recipe.citations = vec!["man:systemctl(1)".into(), "archwiki:Systemd".into()];
    recipe.status = RecipeStatus::Active;

    recipe
}

/// Recipe: Enable sshd service.
pub fn create_enable_sshd_service() -> Recipe {
    let mut recipe = Recipe::new(
        "enable_sshd_service".into(),
        "services".into(),
        "enable_service".into(),
        RecipePattern {
            user_goal: "enable SSH daemon service".into(),
            slots: HashMap::from([("service".into(), "sshd".into())]),
        },
        RecipeMatcher {
            required_keywords: vec!["ssh".into(), "enable".into()],
            optional_keywords: vec![
                "sshd".into(),
                "daemon".into(),
                "service".into(),
                "start".into(),
            ],
            negative_keywords: vec!["disable".into(), "stop".into()],
            min_confidence: 0.8,
            exact_intent: Some("enable_service".into()),
        },
        vec![
            PlanStep::Explain {
                message: "Enabling sshd.service with systemctl. This allows SSH connections."
                    .into(),
            },
            PlanStep::EnableService {
                service: "sshd.service".into(),
                start: true,
            },
            PlanStep::VerifyCommand {
                command: "systemctl is-active sshd".into(),
                expect_success: true,
            },
        ],
    );

    recipe.preconditions = vec![
        Precondition::ToolExists {
            tool: "systemctl".into(),
        },
        Precondition::ServiceExists {
            service: "sshd.service".into(),
        },
    ];
    recipe.confirmation_policy = ConfirmationPolicy::Require;
    recipe.success_criteria = SuccessCriteria {
        must_succeed: vec!["enable_service".into()],
        rollback_on_failure: true,
        post_verification: Some("systemctl is-active sshd".into()),
    };
    recipe.citations = vec![
        "archwiki:OpenSSH#Server_usage".into(),
        "man:systemctl(1)".into(),
    ];
    recipe.status = RecipeStatus::Active;

    recipe
}
