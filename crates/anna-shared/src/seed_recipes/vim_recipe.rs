//! Recipe for enabling syntax highlighting in Vim.

use crate::recipe_schema::{
    ConfirmationPolicy, PlanStep, Precondition, Recipe, RecipeMatcher, RecipePattern, RecipeStatus,
    SuccessCriteria,
};
use std::collections::HashMap;

/// Recipe: Enable syntax highlighting in Vim.
pub fn create_vim_enable_syntax() -> Recipe {
    let mut recipe = Recipe::new(
        "vim_enable_syntax".into(),
        "desktop".into(),
        "configure_editor_feature".into(),
        RecipePattern {
            user_goal: "enable syntax highlighting in vim".into(),
            slots: HashMap::from([
                ("editor".into(), "vim".into()),
                ("feature".into(), "syntax_highlighting".into()),
            ]),
        },
        RecipeMatcher {
            required_keywords: vec!["vim".into(), "syntax".into()],
            optional_keywords: vec![
                "highlight".into(),
                "highlighting".into(),
                "color".into(),
                "colour".into(),
                "enable".into(),
            ],
            negative_keywords: vec!["neovim".into(), "nvim".into(), "emacs".into()],
            min_confidence: 0.75,
            exact_intent: Some("configure_editor_feature".into()),
        },
        vec![
            PlanStep::Explain {
                message: "Vim enables syntax highlighting using 'syntax enable' in ~/.vimrc. \
                          This makes code and config files easier to read with colors."
                    .into(),
            },
            PlanStep::BackupFile {
                path: "$HOME/.vimrc".into(),
            },
            PlanStep::EnsureLine {
                path: "$HOME/.vimrc".into(),
                line: "syntax enable".into(),
            },
            PlanStep::VerifyCommand {
                command: "grep -q 'syntax' ~/.vimrc".into(),
                expect_success: true,
            },
        ],
    );

    recipe.preconditions = vec![Precondition::ToolExists { tool: "vim".into() }];
    recipe.confirmation_policy = ConfirmationPolicy::MutatingOnly;
    recipe.success_criteria = SuccessCriteria {
        must_succeed: vec!["ensure_line".into()],
        rollback_on_failure: true,
        post_verification: Some("grep -q 'syntax' ~/.vimrc".into()),
    };
    recipe.citations = vec![
        "archwiki:Vim#Syntax_highlighting".into(),
        "man:vim(1)".into(),
    ];
    recipe.status = RecipeStatus::Active;

    recipe
}
