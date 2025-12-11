//! Seed recipes for Anna's learning system.
//! v0.0.418: Initial recipes that demonstrate the recipe system.
//!
//! These are 3 concrete recipes:
//! 1. vim_enable_syntax - Config change recipe
//! 2. check_disk_usage_root - Repeatable diagnostic recipe
//! 3. check_free_ram - Repeatable diagnostic recipe

use crate::recipe_schema::{
    ConfirmationPolicy, PlanStep, Precondition, Recipe, RecipeMatcher, RecipePattern,
    RecipeStatus, SuccessCriteria,
};
use crate::recipe_storage::RecipeStorage;
use std::collections::HashMap;

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
        create_vim_enable_syntax(),
        create_check_disk_usage_root(),
        create_check_free_ram(),
        create_check_swap_status(),
        create_check_failed_services(),
        create_enable_sshd_service(),
    ]
}

/// Recipe: Enable syntax highlighting in Vim.
fn create_vim_enable_syntax() -> Recipe {
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

    recipe.preconditions = vec![
        Precondition::ToolExists { tool: "vim".into() },
    ];
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

/// Recipe: Check disk usage on root partition.
fn create_check_disk_usage_root() -> Recipe {
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

/// Recipe: Check free RAM.
fn create_check_free_ram() -> Recipe {
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
fn create_check_swap_status() -> Recipe {
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

/// Recipe: Check failed systemd services.
fn create_check_failed_services() -> Recipe {
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
fn create_enable_sshd_service() -> Recipe {
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

/// Get a seed recipe by ID (for testing).
pub fn get_seed_recipe(id: &str) -> Option<Recipe> {
    create_seed_recipes().into_iter().find(|r| r.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let mut storage = RecipeStorage::with_dirs(
            dir.path().join("user"),
            dir.path().join("sys"),
        );

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
        assert!(recipe.matcher.required_keywords.contains(&"vim".to_string()));
        assert!(recipe.matcher.required_keywords.contains(&"syntax".to_string()));
        assert!(recipe.matcher.negative_keywords.contains(&"neovim".to_string()));
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
