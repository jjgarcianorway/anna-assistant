//! Built-in recipe matchers for shell configurations.
//!
//! Extracted from recipe_builtins.rs for modularization.

use anna_shared::recipe::{Recipe, RecipeAction, RecipeKind};
use anna_shared::shell_recipes;
use tracing::info;

use crate::recipe_fast_path::{ticket_from_recipe, RecipeFastPathResult};

/// Check query against built-in shell recipes
pub fn check_shell_recipes(query: &str) -> Option<RecipeFastPathResult> {
    let q = query.to_lowercase();

    // Detect shell from query or environment
    let shell = if q.contains("bash") || q.contains("bashrc") {
        Some(shell_recipes::Shell::Bash)
    } else if q.contains("zsh") || q.contains("zshrc") {
        Some(shell_recipes::Shell::Zsh)
    } else if q.contains("fish") {
        Some(shell_recipes::Shell::Fish)
    } else {
        shell_recipes::Shell::detect()
    };

    // Detect feature from query
    let feature = shell_recipes::detect_feature(&q)?;
    let shell = shell?;

    // Find matching recipe
    let recipe = shell_recipes::find_recipe(shell, feature)?;

    // Build a synthetic Recipe for the result
    let synthetic_recipe = Recipe {
        id: format!("shell-{}-{:?}", shell, feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "desktop",
            "request",
            "shell_config",
            query,
        ),
        team: anna_shared::teams::Team::Desktop,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: format!(
            "To {} in {}:\n\nAdd to ~/{}\n```\n{}\n```\n\n{}",
            feature.display_name(),
            shell.display_name(),
            shell.config_path().display(),
            recipe.lines.join("\n"),
            recipe
                .rollback_hint
                .as_deref()
                .unwrap_or("To undo: remove the added lines")
        ),
        created_at: 0,
        success_count: 100, // Built-in = mature
        reliability_score: 95,
        kind: RecipeKind::ShellConfig,
        target: None,
        action: RecipeAction::EnsureLine {
            line: recipe.lines.join("\n"),
        },
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: feature.keywords().iter().map(|s| s.to_string()).collect(),
        targets: vec![shell.display_name().to_lowercase()],
        preconditions: vec![],
        clarify_prereqs: vec![],
        negative_match_patterns: vec![],
        citations: vec![],
    };

    info!(
        "Shell recipe match: {} in {}",
        feature.display_name(),
        shell.display_name()
    );

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec![
            shell.display_name().to_lowercase(),
            feature.display_name().to_string(),
        ],
        skip_llm: true,
        learned_recipe_id: None,
    })
}
