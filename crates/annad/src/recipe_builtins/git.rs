//! Built-in recipe matchers for git configurations.
//!
//! Extracted from recipe_builtins.rs for modularization.

use anna_shared::git_recipes;
use anna_shared::recipe::{Recipe, RecipeAction, RecipeKind};
use tracing::info;

use crate::recipe_fast_path::{ticket_from_recipe, RecipeFastPathResult};

/// Check query against built-in git recipes
pub fn check_git_recipes(query: &str) -> Option<RecipeFastPathResult> {
    let q = query.to_lowercase();

    // Must mention "git" to match git recipes
    if !q.contains("git") {
        return None;
    }

    // Detect feature from query
    let feature = git_recipes::detect_feature(&q)?;

    // Find matching recipes
    let recipes = git_recipes::find_recipe(feature);
    if recipes.is_empty() {
        return None;
    }

    let recipe = &recipes[0];

    // Build answer from recipe
    let answer = if recipe.needs_parameters() {
        format!(
            "To configure {}:\n\nCommands:\n{}\n\nNote: Replace {{name}} and {{email}} with your values.\n\n{}",
            feature.display_name(),
            recipe.commands.iter().map(|c| format!("  {}", c)).collect::<Vec<_>>().join("\n"),
            recipe.rollback_hint.as_deref().unwrap_or("")
        )
    } else {
        format!(
            "To configure {}:\n\nRun:\n{}\n\n{}",
            feature.display_name(),
            recipe
                .commands
                .iter()
                .map(|c| format!("  {}", c))
                .collect::<Vec<_>>()
                .join("\n"),
            recipe.rollback_hint.as_deref().unwrap_or("")
        )
    };

    // Build a synthetic Recipe
    let synthetic_recipe = Recipe {
        id: format!("git-{:?}", feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "system",
            "request",
            "git_config",
            query,
        ),
        team: anna_shared::teams::Team::General,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: answer,
        created_at: 0,
        success_count: 100,
        reliability_score: 95,
        kind: RecipeKind::GitConfig,
        target: None,
        action: RecipeAction::None,
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: feature.keywords().iter().map(|s| s.to_string()).collect(),
        targets: vec!["git".to_string()],
        preconditions: vec![],
        clarify_prereqs: vec![],
        negative_match_patterns: vec![],
        citations: vec![],
    };

    info!("Git recipe match: {}", feature.display_name());

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec!["git".to_string(), feature.display_name().to_string()],
        skip_llm: true,
        learned_recipe_id: None,
    })
}
