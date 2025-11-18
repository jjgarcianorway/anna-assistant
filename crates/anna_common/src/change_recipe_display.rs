//! UI Display for Change Recipes
//!
//! Phase 8: Beautiful UX - Professional formatting with visual safety cues.
//!
//! Human-readable formatting of change recipes with risk classification,
//! consequences, and rollback strategies clearly explained.

use crate::change_recipe::*;
use crate::display::UI;
use crate::terminal_format as fmt;

/// Display a change recipe for user review before execution
///
/// Phase 8: Beautified with consistent formatting, color-coded risk badges,
/// clear visual hierarchy, and professional spacing.
pub fn display_recipe_for_approval(ui: &UI, recipe: &ChangeRecipe) {
    println!();

    // Title with icon
    println!(
        "{}",
        fmt::section_title("📋", &format!(" Proposed Change: {}", recipe.title))
    );
    println!();

    // Summary in a subtle box
    println!("{}", fmt::info(&recipe.summary));
    println!();

    // Why it matters section
    println!("{}", fmt::section_title("💡", " Why This Matters"));
    println!("{}", fmt::bullet(&recipe.why_it_matters));
    println!();

    // Risk assessment with visual emphasis
    display_risk_assessment_beautiful(ui, recipe);
    println!();

    // Planned actions with clean formatting
    println!("{}", fmt::section_title("🔧", " Planned Actions"));
    println!();
    for (idx, action) in recipe.actions.iter().enumerate() {
        display_action_beautiful(action, idx + 1);
    }
    println!();

    // Rollback strategy
    println!("{}", fmt::section_title("🔄", " Rollback Strategy"));
    println!("{}", fmt::bullet(&recipe.rollback_notes));
    println!();

    // Sudo requirements with strong visual indicator
    if recipe.needs_sudo() {
        let sudo_actions = recipe.sudo_actions();
        println!(
            "{}",
            fmt::warning(&format!(
                "{} out of {} actions require sudo privileges:",
                sudo_actions.len(),
                recipe.actions.len()
            ))
        );
        for action in sudo_actions {
            println!("  {} {}", fmt::sudo_badge(), action.description);
        }
        println!();
    }

    // Final separator
    println!("{}", fmt::separator(70));
}

/// Display risk assessment with beautiful color coding
fn display_risk_assessment_beautiful(_ui: &UI, recipe: &ChangeRecipe) {
    println!("{}", fmt::section_title("⚠️", "  Risk Assessment"));
    println!();

    // Overall risk with appropriate badge
    let risk_str = match recipe.overall_risk {
        ChangeRisk::Low => "low",
        ChangeRisk::Medium => "medium",
        ChangeRisk::High => "high",
        ChangeRisk::Forbidden => "forbidden",
    };

    println!(
        "  Overall Risk: {}  {}",
        fmt::risk_badge(risk_str),
        fmt::dimmed(&format!("({})", recipe.overall_risk.description()))
    );
    println!();

    // Worst case scenario
    println!(
        "{}",
        fmt::key_value("  Worst case:", recipe.overall_risk.worst_case())
    );
    println!();

    // Special handling for Forbidden
    if recipe.overall_risk == ChangeRisk::Forbidden {
        println!(
            "{}",
            fmt::error("This change is TOO DANGEROUS to automate.")
        );
        println!(
            "{}",
            fmt::error("Do it manually with proper backups and expertise.")
        );
        println!();
    }
}

/// Display a single action with beautiful formatting
fn display_action_beautiful(action: &ChangeAction, number: usize) {
    // Risk and sudo badges
    let risk_str = match action.risk {
        ChangeRisk::Low => "low",
        ChangeRisk::Medium => "medium",
        ChangeRisk::High => "high",
        ChangeRisk::Forbidden => "forbidden",
    };

    let category_str = match action.category {
        ChangeCategory::CosmeticUser => "cosmetic",
        ChangeCategory::UserConfig => "config",
        ChangeCategory::SystemService => "service",
        ChangeCategory::SystemPackage => "package",
        ChangeCategory::BootAndStorage => "boot",
    };

    let sudo = if action.kind.needs_sudo() {
        format!(" {}", fmt::sudo_badge())
    } else {
        String::new()
    };

    // Numbered item with risk badge
    println!(
        "{}  {} {}{}",
        fmt::numbered(number, &action.description),
        fmt::risk_badge(risk_str),
        fmt::category_badge(category_str),
        sudo
    );

    // Impact on indented line
    println!(
        "     {}",
        fmt::dimmed(&format!("Impact: {}", action.estimated_impact))
    );

    // Action-specific details
    display_action_details_beautiful(&action.kind);
}

/// Display specific details for different action kinds with beautiful formatting
fn display_action_details_beautiful(kind: &ChangeActionKind) {
    match kind {
        ChangeActionKind::EditFile { path, strategy } => {
            println!(
                "     {}",
                fmt::key_value("File:", &path.display().to_string())
            );
            match strategy {
                EditStrategy::AppendIfMissing { lines } => {
                    println!(
                        "     {}",
                        fmt::key_value(
                            "Strategy:",
                            &format!("Append {} lines if missing", lines.len())
                        )
                    );
                }
                EditStrategy::ReplaceSection {
                    start_marker,
                    end_marker,
                    ..
                } => {
                    println!(
                        "     {}",
                        fmt::key_value(
                            "Strategy:",
                            &format!("Replace section '{}' to '{}'", start_marker, end_marker)
                        )
                    );
                }
                EditStrategy::ReplaceEntire { .. } => {
                    println!(
                        "     {}",
                        fmt::warning("Strategy: Replace entire file (DESTRUCTIVE)")
                    );
                }
            }
        }
        ChangeActionKind::AppendToFile { path, content } => {
            println!(
                "     {}",
                fmt::key_value("File:", &path.display().to_string())
            );
            println!(
                "     {}",
                fmt::key_value("Content:", &format!("{} bytes", content.len()))
            );
        }
        ChangeActionKind::InstallPackages { packages } => {
            println!("     {}", fmt::key_value("Packages:", &packages.join(", ")));
        }
        ChangeActionKind::RemovePackages { packages } => {
            println!("     {}", fmt::key_value("Packages:", &packages.join(", ")));
        }
        ChangeActionKind::EnableService {
            service_name,
            user_service,
        } => {
            let scope = if *user_service { "user" } else { "system" };
            println!(
                "     {}",
                fmt::key_value("Service:", &format!("{} ({})", service_name, scope))
            );
        }
        ChangeActionKind::DisableService {
            service_name,
            user_service,
        } => {
            let scope = if *user_service { "user" } else { "system" };
            println!(
                "     {}",
                fmt::key_value("Service:", &format!("{} ({})", service_name, scope))
            );
        }
        ChangeActionKind::SetWallpaper { image_path } => {
            println!(
                "     {}",
                fmt::key_value("Image:", &image_path.display().to_string())
            );
        }
        ChangeActionKind::RunReadOnlyCommand { command, args } => {
            println!(
                "     {}",
                fmt::key_value("Command:", &format!("{} {}", command, args.join(" ")))
            );
        }
    }

    println!(); // Space after action details
}

/// Display a concise summary of a recipe (for lists) with beautiful formatting
pub fn display_recipe_summary(_ui: &UI, recipe: &ChangeRecipe) {
    let risk_str = match recipe.overall_risk {
        ChangeRisk::Low => "low",
        ChangeRisk::Medium => "medium",
        ChangeRisk::High => "high",
        ChangeRisk::Forbidden => "forbidden",
    };

    println!(
        "{}  {} - {} ({} actions)",
        fmt::risk_badge(risk_str),
        fmt::bold(&recipe.title),
        recipe.summary,
        recipe.actions.len()
    );
}

/// Display multiple recipes in a table format
pub fn display_recipe_table(recipes: &[&ChangeRecipe]) {
    if recipes.is_empty() {
        println!("{}", fmt::info("No recipes to display"));
        return;
    }

    println!("{}", fmt::section_title("📋", " Available Recipes"));
    println!();

    // Table header
    println!(
        "{}",
        fmt::table_header(&[("ID", 8), ("Title", 30), ("Risk", 12), ("Actions", 8)])
    );
    println!();

    // Table rows
    for (idx, recipe) in recipes.iter().enumerate() {
        let risk_str = match recipe.overall_risk {
            ChangeRisk::Low => "low",
            ChangeRisk::Medium => "medium",
            ChangeRisk::High => "high",
            ChangeRisk::Forbidden => "forbidden",
        };

        println!(
            "{}",
            fmt::table_row(&[
                (&format!("{}", idx + 1), 8),
                (&recipe.title, 30),
                (risk_str, 12),
                (&format!("{}", recipe.actions.len()), 8),
            ])
        );
    }

    println!();
}

/// Display execution progress for multi-step recipes
pub fn display_execution_progress(current: usize, total: usize, current_action: &str) {
    println!(
        "{}",
        fmt::progress(current, total, &format!("Executing: {}", current_action))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_display_simple_recipe_beautiful() {
        let action = ChangeAction::new(
            ChangeActionKind::SetWallpaper {
                image_path: PathBuf::from("/home/user/Pictures/wallpaper.jpg"),
            },
            "Set desktop wallpaper".to_string(),
            "Changes background image".to_string(),
        );

        let recipe = ChangeRecipe::new(
            "Change Wallpaper".to_string(),
            "Sets a random wallpaper from your Pictures directory".to_string(),
            "Personalizes your desktop and improves your workspace mood".to_string(),
            vec![action],
            "Just change the wallpaper back using your system settings".to_string(),
            ChangeRecipeSource::Manual,
        );

        // This test ensures the display functions don't panic
        let ui = UI::auto();
        display_recipe_for_approval(&ui, &recipe);
        display_recipe_summary(&ui, &recipe);
    }

    #[test]
    fn test_display_complex_recipe_with_sudo_beautiful() {
        let actions = vec![
            ChangeAction::new(
                ChangeActionKind::EditFile {
                    path: PathBuf::from("/home/user/.vimrc"),
                    strategy: EditStrategy::AppendIfMissing {
                        lines: vec!["syntax on".to_string(), "set number".to_string()],
                    },
                },
                "Enable syntax highlighting and line numbers".to_string(),
                "Makes code more readable in vim".to_string(),
            ),
            ChangeAction::new(
                ChangeActionKind::InstallPackages {
                    packages: vec!["vim-runtime".to_string()],
                },
                "Install vim color schemes".to_string(),
                "Adds color scheme support".to_string(),
            ),
        ];

        let recipe = ChangeRecipe::new(
            "Configure Vim".to_string(),
            "Sets up vim with syntax highlighting and line numbers".to_string(),
            "Makes coding in vim much easier and reduces mistakes".to_string(),
            actions,
            "Restore .vimrc from backup, uninstall vim-runtime if needed".to_string(),
            ChangeRecipeSource::LlmPlanned {
                model_profile_id: "ollama-llama3.1-8b".to_string(),
                user_query: "enable syntax highlighting in vim".to_string(),
            },
        );

        // This test ensures display works for recipes with sudo requirements
        let ui = UI::auto();
        display_recipe_for_approval(&ui, &recipe);

        assert!(recipe.needs_sudo());
        assert_eq!(recipe.sudo_actions().len(), 1);
    }

    #[test]
    fn test_display_recipe_table_beautiful() {
        let recipe1 = ChangeRecipe::new(
            "Test Recipe 1".to_string(),
            "First test".to_string(),
            "Testing".to_string(),
            vec![ChangeAction::new(
                ChangeActionKind::SetWallpaper {
                    image_path: PathBuf::from("/test.jpg"),
                },
                "Test".to_string(),
                "Test".to_string(),
            )],
            "Rollback".to_string(),
            ChangeRecipeSource::Manual,
        );

        let recipe2 = ChangeRecipe::new(
            "Test Recipe 2".to_string(),
            "Second test".to_string(),
            "Testing".to_string(),
            vec![ChangeAction::new(
                ChangeActionKind::EditFile {
                    path: PathBuf::from("/etc/test"),
                    strategy: EditStrategy::AppendIfMissing {
                        lines: vec!["test".to_string()],
                    },
                },
                "Test".to_string(),
                "Test".to_string(),
            )],
            "Rollback".to_string(),
            ChangeRecipeSource::Manual,
        );

        let recipes = vec![&recipe1, &recipe2];
        display_recipe_table(&recipes);
    }

    #[test]
    fn test_display_execution_progress_beautiful() {
        display_execution_progress(3, 5, "Installing package");
    }
}
