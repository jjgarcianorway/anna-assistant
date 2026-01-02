//! Built-in recipe matchers for system services (systemd and cron).
//!
//! Extracted from recipe_builtins.rs for modularization.
//! v0.0.233: Added systemd unit file recipes.
//! v0.0.234: Added cron job recipes.

use anna_shared::cron_recipes;
use anna_shared::recipe::{Recipe, RecipeAction, RecipeKind};
use anna_shared::systemd_recipes;
use tracing::info;

use crate::recipe_fast_path::{ticket_from_recipe, RecipeFastPathResult};

/// Check query against built-in systemd recipes (v0.0.233)
pub fn check_systemd_recipes(query: &str) -> Option<RecipeFastPathResult> {
    // Use the systemd recipe matcher
    let systemd_recipe = systemd_recipes::match_query(query)?;

    // Build a synthetic Recipe from the systemd recipe
    let synthetic_recipe = Recipe {
        id: format!("systemd-{:?}", systemd_recipe.feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "system",
            "request",
            "systemd_unit",
            query,
        ),
        team: anna_shared::teams::Team::Services,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: systemd_recipe.answer_template.clone(),
        created_at: 0,
        success_count: 100, // Built-in = mature
        reliability_score: 95,
        kind: RecipeKind::SystemdUnit,
        target: None,
        action: RecipeAction::None,
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: systemd_recipe
            .feature
            .keywords()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        targets: vec!["systemd".to_string(), "service".to_string()],
        preconditions: vec![],
        clarify_prereqs: vec![],
        negative_match_patterns: vec![],
        citations: vec![],
    };

    info!(
        "Systemd recipe match: {}",
        systemd_recipe.feature.display_name()
    );

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec![
            "systemd".to_string(),
            systemd_recipe.feature.display_name().to_string(),
        ],
        skip_llm: true,
        learned_recipe_id: None,
    })
}

/// Check query against built-in cron recipes (v0.0.234)
pub fn check_cron_recipes(query: &str) -> Option<RecipeFastPathResult> {
    // Use the cron recipe matcher
    let cron_recipe = cron_recipes::match_query(query)?;

    // Build a synthetic Recipe from the cron recipe
    let synthetic_recipe = Recipe {
        id: format!("cron-{:?}", cron_recipe.feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "system", "request", "cron_job", query,
        ),
        team: anna_shared::teams::Team::Services,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: cron_recipe.answer_template.clone(),
        created_at: 0,
        success_count: 100, // Built-in = mature
        reliability_score: 95,
        kind: RecipeKind::CronJob,
        target: None,
        action: RecipeAction::None,
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: cron_recipe
            .feature
            .keywords()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        targets: vec!["cron".to_string(), "crontab".to_string()],
        preconditions: vec![],
        clarify_prereqs: vec![],
        negative_match_patterns: vec![],
        citations: vec![],
    };

    info!("Cron recipe match: {}", cron_recipe.feature.display_name());

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec![
            "cron".to_string(),
            cron_recipe.feature.display_name().to_string(),
        ],
        skip_llm: true,
        learned_recipe_id: None,
    })
}
