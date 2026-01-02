//! Conversion and utility functions for recipe fast path.

use anna_shared::recipe::{Recipe, RecipeKind};
use anna_shared::rpc::{
    EvidenceBlock, QueryIntent, ReliabilitySignals, ServiceDeskResult, SpecialistDomain,
    TranslatorTicket,
};
use anna_shared::trace::{ExecutionTrace, ProbeStats};
use anna_shared::transcript::Transcript;

/// Map recipe team to specialist domain
pub fn team_to_domain(team: &anna_shared::teams::Team) -> SpecialistDomain {
    match team {
        anna_shared::teams::Team::Network => SpecialistDomain::Network,
        anna_shared::teams::Team::Storage => SpecialistDomain::Storage,
        anna_shared::teams::Team::Security => SpecialistDomain::Security,
        _ => SpecialistDomain::System,
    }
}

/// Create a TranslatorTicket from a recipe
pub fn ticket_from_recipe(recipe: &Recipe) -> TranslatorTicket {
    let intent = match recipe.kind {
        RecipeKind::Query => QueryIntent::Question,
        _ => QueryIntent::Request,
    };

    TranslatorTicket {
        intent,
        domain: team_to_domain(&recipe.team),
        entities: recipe.targets.clone(),
        needs_probes: recipe.probe_sequence.clone(),
        clarification_question: None,
        confidence: (recipe.reliability_score as f32) / 100.0,
        answer_contract: None,
    }
}

/// v0.0.102: Build a ServiceDeskResult directly from a recipe
/// v0.0.305: Added query parameter for negative feedback learning
pub fn build_recipe_result(
    request_id: String,
    recipe: &Recipe,
    matched_tokens: &[String],
    transcript: Transcript,
    query: &str,
) -> ServiceDeskResult {
    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: true,
        answer_grounded: true,
        no_invention: true,
        clarification_not_needed: true,
    };
    let trace = ExecutionTrace::deterministic_route(
        &format!("recipe:{}", recipe.id),
        ProbeStats::default(),
        vec![],
    );
    let answer = format!(
        "{}\n\n*Recipe: {} (matched: {})*",
        recipe.answer_template,
        recipe.id,
        matched_tokens.join(", ")
    );

    // v0.0.103: Ask for feedback if recipe confidence is borderline (60-75)
    // or if recipe is new (success_count < 3)
    // v0.0.305: Pass query for negative feedback learning
    let feedback_request = if recipe.reliability_score >= 60 && recipe.reliability_score <= 75 {
        Some(
            anna_shared::recipe_feedback::FeedbackRequest::borderline_confidence(
                &recipe.id,
                recipe.reliability_score,
                query,
            ),
        )
    } else if recipe.success_count < 3 {
        Some(anna_shared::recipe_feedback::FeedbackRequest::new_recipe(
            &recipe.id, query,
        ))
    } else {
        None
    };

    ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer,
        // v0.0.298: Recipe answers validated if high reliability_score
        validated: recipe.reliability_score >= 80,
        reliability_score: recipe.reliability_score,
        reliability_signals: signals,
        reliability_explanation: None,
        domain: team_to_domain(&recipe.team),
        evidence: EvidenceBlock::default(),
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript,
        execution_trace: Some(trace),
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request,
    }
}

/// v0.0.264: Get config hint for specialist context if this is a config query.
/// Returns Some(hint_text) if the query is about editor/app configuration.
pub fn get_config_hint_for_specialist(query: &str) -> Option<String> {
    use anna_shared::config_intent::ConfigHint;

    ConfigHint::from_query(query).map(|hint| hint.to_specialist_context())
}
