//! UI Display for Change Recipes
//!
//! Human-readable formatting of change recipes with risk classification,
//! consequences, and rollback strategies clearly explained.

use crate::change_recipe::*;
use crate::display::UI;

/// Display a change recipe for user review before execution
pub fn display_recipe_for_approval(ui: &UI, recipe: &ChangeRecipe) {
    println!();
    ui.section_header("📋", &format!("Proposed Change: {}", recipe.title));
    println!();

    // Summary
    ui.info(&recipe.summary);
    println!();

    // Why it matters
    ui.section_header("💡", "Why This Matters");
    ui.info(&recipe.why_it_matters);
    println!();

    // Risk assessment
    display_risk_assessment(ui, recipe);
    println!();

    // Planned actions
    ui.section_header("🔧", "Planned Actions");
    for (idx, action) in recipe.actions.iter().enumerate() {
        display_action(ui, action, idx + 1);
    }
    println!();

    // Rollback strategy
    ui.section_header("🔄", "Rollback Strategy");
    ui.info(&recipe.rollback_notes);
    println!();

    // Sudo requirements
    if recipe.needs_sudo() {
        let sudo_actions = recipe.sudo_actions();
        ui.warning(&format!(
            "⚠️  {} out of {} actions require sudo privileges",
            sudo_actions.len(),
            recipe.actions.len()
        ));
        let sudo_descriptions: Vec<&str> = sudo_actions.iter().map(|a| a.description.as_str()).collect();
        ui.bullet_list(&sudo_descriptions);
        println!();
    }
}

/// Display risk assessment with clear visual indicators
fn display_risk_assessment(ui: &UI, recipe: &ChangeRecipe) {
    ui.section_header("⚠️", "Risk Assessment");

    // Overall risk
    let risk_color = match recipe.overall_risk {
        ChangeRisk::Low => "green",
        ChangeRisk::Medium => "yellow",
        ChangeRisk::High => "red",
        ChangeRisk::Forbidden => "red",
    };

    let risk_indicator = match recipe.overall_risk {
        ChangeRisk::Low => "✓ LOW RISK",
        ChangeRisk::Medium => "⚠ MEDIUM RISK",
        ChangeRisk::High => "⚠⚠ HIGH RISK",
        ChangeRisk::Forbidden => "🛑 FORBIDDEN",
    };

    // Display risk level (we'll use UI's existing methods)
    match recipe.overall_risk {
        ChangeRisk::Low => ui.success(&format!("{} - {}", risk_indicator, recipe.overall_risk.description())),
        ChangeRisk::Medium => ui.warning(&format!("{} - {}", risk_indicator, recipe.overall_risk.description())),
        ChangeRisk::High => ui.error(&format!("{} - {}", risk_indicator, recipe.overall_risk.description())),
        ChangeRisk::Forbidden => {
            ui.error(&format!("{} - {}", risk_indicator, recipe.overall_risk.description()));
            ui.error("This change is TOO DANGEROUS to automate. Do it manually.");
        }
    }

    println!();

    // Worst case scenario
    ui.info("Worst case scenario:");
    ui.info(&format!("  • {}", recipe.overall_risk.worst_case()));
}

/// Display a single action with its details
fn display_action(ui: &UI, action: &ChangeAction, number: usize) {
    let risk_badge = match action.risk {
        ChangeRisk::Low => "✓",
        ChangeRisk::Medium => "⚠",
        ChangeRisk::High => "⚠⚠",
        ChangeRisk::Forbidden => "🛑",
    };

    let sudo_badge = if action.kind.needs_sudo() { " [sudo]" } else { "" };

    ui.info(&format!(
        "{}. {} {} {}{}",
        number,
        risk_badge,
        action.description,
        action.risk.description(),
        sudo_badge
    ));

    ui.info(&format!("   Impact: {}", action.estimated_impact));

    // Show specific details based on action kind
    display_action_details(ui, &action.kind);

    println!();
}

/// Display specific details for different action kinds
fn display_action_details(ui: &UI, kind: &ChangeActionKind) {
    match kind {
        ChangeActionKind::EditFile { path, strategy } => {
            ui.info(&format!("   • File: {}", path.display()));
            match strategy {
                EditStrategy::AppendIfMissing { lines } => {
                    ui.info(&format!("   • Strategy: Append {} lines if missing", lines.len()));
                }
                EditStrategy::ReplaceSection { start_marker, end_marker, .. } => {
                    ui.info(&format!(
                        "   • Strategy: Replace section between '{}' and '{}'",
                        start_marker, end_marker
                    ));
                }
                EditStrategy::ReplaceEntire { .. } => {
                    ui.info("   • Strategy: Replace entire file (DESTRUCTIVE)");
                }
            }
        }
        ChangeActionKind::AppendToFile { path, content } => {
            ui.info(&format!("   • File: {}", path.display()));
            ui.info(&format!("   • Content: {} bytes", content.len()));
        }
        ChangeActionKind::InstallPackages { packages } => {
            ui.info(&format!("   • Packages: {}", packages.join(", ")));
        }
        ChangeActionKind::RemovePackages { packages } => {
            ui.info(&format!("   • Packages: {}", packages.join(", ")));
        }
        ChangeActionKind::EnableService { service_name, user_service } => {
            let scope = if *user_service { "user" } else { "system" };
            ui.info(&format!("   • Service: {} ({})", service_name, scope));
        }
        ChangeActionKind::DisableService { service_name, user_service } => {
            let scope = if *user_service { "user" } else { "system" };
            ui.info(&format!("   • Service: {} ({})", service_name, scope));
        }
        ChangeActionKind::SetWallpaper { image_path } => {
            ui.info(&format!("   • Image: {}", image_path.display()));
        }
        ChangeActionKind::RunReadOnlyCommand { command, args } => {
            ui.info(&format!("   • Command: {} {}", command, args.join(" ")));
        }
    }
}

/// Display a concise summary of a recipe (for lists)
pub fn display_recipe_summary(ui: &UI, recipe: &ChangeRecipe) {
    let risk_indicator = match recipe.overall_risk {
        ChangeRisk::Low => "✓",
        ChangeRisk::Medium => "⚠",
        ChangeRisk::High => "⚠⚠",
        ChangeRisk::Forbidden => "🛑",
    };

    ui.info(&format!(
        "{} {} - {} ({} actions, {} risk)",
        risk_indicator,
        recipe.title,
        recipe.summary,
        recipe.actions.len(),
        match recipe.overall_risk {
            ChangeRisk::Low => "low",
            ChangeRisk::Medium => "medium",
            ChangeRisk::High => "high",
            ChangeRisk::Forbidden => "FORBIDDEN",
        }
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_display_simple_recipe() {
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

        // This test just ensures the display functions don't panic
        let ui = UI::auto();
        display_recipe_for_approval(&ui, &recipe);
        display_recipe_summary(&ui, &recipe);
    }

    #[test]
    fn test_display_complex_recipe_with_sudo() {
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
}
