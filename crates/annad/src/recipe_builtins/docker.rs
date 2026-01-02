//! Built-in recipe matchers for Docker configurations.
//!
//! Extracted from recipe_builtins.rs for modularization.
//! v0.0.235: Added Docker Compose recipes.

use anna_shared::docker_recipes;
use anna_shared::recipe::{Recipe, RecipeAction, RecipeKind};
use tracing::info;

use crate::recipe_fast_path::{ticket_from_recipe, RecipeFastPathResult};

/// Check query against built-in Docker recipes (v0.0.235)
pub fn check_docker_recipes(query: &str) -> Option<RecipeFastPathResult> {
    // Use the Docker recipe matcher
    let docker_recipe = docker_recipes::match_query(query)?;

    // Build a synthetic Recipe from the Docker recipe
    let synthetic_recipe = Recipe {
        id: format!("docker-{:?}", docker_recipe.feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "system",
            "request",
            "docker_compose",
            query,
        ),
        team: anna_shared::teams::Team::Services,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: docker_recipe.answer_template.clone(),
        created_at: 0,
        success_count: 100, // Built-in = mature
        reliability_score: 95,
        kind: RecipeKind::DockerCompose,
        target: None,
        action: RecipeAction::None,
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: docker_recipe
            .feature
            .keywords()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        targets: vec!["docker".to_string(), "compose".to_string()],
        preconditions: vec![],
        clarify_prereqs: vec![],
        negative_match_patterns: vec![],
        citations: vec![],
    };

    info!(
        "Docker recipe match: {}",
        docker_recipe.feature.display_name()
    );

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec![
            "docker".to_string(),
            docker_recipe.feature.display_name().to_string(),
        ],
        skip_llm: true,
        learned_recipe_id: None,
    })
}
