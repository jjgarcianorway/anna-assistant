//! Built-in recipe matchers for SSH configurations.
//!
//! Extracted from recipe_builtins.rs for modularization.
//! v0.0.104: Added SSH recipe support.

use anna_shared::recipe::{Recipe, RecipeAction, RecipeKind};
use anna_shared::ssh_recipes;
use tracing::info;

use crate::recipe_fast_path::{ticket_from_recipe, RecipeFastPathResult};

/// Check query against built-in SSH recipes (v0.0.104)
pub fn check_ssh_recipes(query: &str) -> Option<RecipeFastPathResult> {
    // Use the SSH recipe matcher
    let ssh_recipe = ssh_recipes::match_query(query)?;

    // Build a synthetic Recipe from the SSH recipe
    let synthetic_recipe = Recipe {
        id: format!("ssh-{:?}", ssh_recipe.feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "system",
            "request",
            "ssh_config",
            query,
        ),
        team: anna_shared::teams::Team::Security,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: ssh_recipe.answer_template.clone(),
        created_at: 0,
        success_count: 100, // Built-in = mature
        reliability_score: 95,
        kind: RecipeKind::SshConfig,
        target: None,
        action: RecipeAction::None,
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: ssh_recipe
            .feature
            .keywords()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        targets: vec!["ssh".to_string()],
        preconditions: vec![],
        clarify_prereqs: vec![],
        negative_match_patterns: vec![],
        citations: vec![],
    };

    info!("SSH recipe match: {}", ssh_recipe.feature.display_name());

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec![
            "ssh".to_string(),
            ssh_recipe.feature.display_name().to_string(),
        ],
        skip_llm: true,
        learned_recipe_id: None,
    })
}
