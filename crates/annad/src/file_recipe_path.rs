//! FileRecipe-based fast path integration (v0.0.406).
//!
//! Handles TOML-based authored recipes from /etc/anna/recipes and ~/.anna/recipes/authored.
//! These take priority over learned recipes for known query patterns.

use anna_shared::recipe_file::{
    execute_recipe, find_matching_recipe, render_answer, ExecutionResult, FileRecipe, RecipeContext,
};
use anna_shared::rpc::{
    EvidenceBlock, QueryIntent, ReliabilitySignals, ServiceDeskResult, SpecialistDomain,
    TranslatorTicket,
};
use anna_shared::trace::{ExecutionTrace, ProbeStats};
use anna_shared::transcript::Transcript;
use std::collections::HashMap;
use tracing::info;

use crate::recipe_fast_path::RecipeFastPathResult;

/// v0.0.406: Check TOML-based authored recipes (highest priority)
pub fn check_file_recipes(
    query: &str,
    domain: SpecialistDomain,
    intent: &str,
    params: &HashMap<String, String>,
) -> Option<RecipeFastPathResult> {
    let match_result = find_matching_recipe(domain, intent, params, query)?;

    // Only use if confidence >= 70
    if match_result.confidence < 70 {
        return None;
    }

    info!(
        "FileRecipe match: {} (confidence={}%, criteria={:?})",
        match_result.recipe.full_id(),
        match_result.confidence,
        match_result.matched_criteria
    );

    // Create a ticket from the file recipe
    let ticket = ticket_from_file_recipe(&match_result.recipe);

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket),
        recipe: None, // This is a FileRecipe, not the old Recipe type
        score: match_result.confidence as u32,
        matched_tokens: match_result.matched_criteria,
        skip_llm: true,
        learned_recipe_id: None,
    })
}

/// v0.0.406: Create a TranslatorTicket from a FileRecipe
fn ticket_from_file_recipe(recipe: &FileRecipe) -> TranslatorTicket {
    let domain = domain_from_str(&recipe.id.domain);

    // Extract probe IDs from steps
    let probes: Vec<String> = recipe
        .plan
        .steps
        .iter()
        .filter_map(|s| s.probe.clone())
        .collect();

    TranslatorTicket {
        intent: QueryIntent::Request,
        domain,
        entities: vec![],
        needs_probes: probes,
        clarification_question: None,
        confidence: 0.9, // FileRecipes are authored and trusted
        answer_contract: None,
    }
}

/// Convert domain string to SpecialistDomain enum
fn domain_from_str(s: &str) -> SpecialistDomain {
    match s {
        "system" => SpecialistDomain::System,
        "storage" => SpecialistDomain::Storage,
        "network" => SpecialistDomain::Network,
        "services" => SpecialistDomain::Services,
        "packages" => SpecialistDomain::Packages,
        "security" => SpecialistDomain::Security,
        "desktop" => SpecialistDomain::Desktop,
        "boot" => SpecialistDomain::Boot,
        "audio" => SpecialistDomain::Audio,
        "display" => SpecialistDomain::Display,
        _ => SpecialistDomain::System,
    }
}

/// v0.0.406: Build a ServiceDeskResult directly from a FileRecipe
pub fn build_file_recipe_result(
    request_id: String,
    recipe: &FileRecipe,
    exec_result: &ExecutionResult,
    transcript: Transcript,
    _query: &str,
) -> ServiceDeskResult {
    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: true,
        answer_grounded: true,
        no_invention: true,
        clarification_not_needed: true,
    };

    let trace = ExecutionTrace::deterministic_route(
        &format!("recipe:{}", recipe.full_id()),
        ProbeStats::default(),
        vec![],
    );

    let answer = render_answer(recipe, exec_result);
    let domain = domain_from_str(&recipe.id.domain);

    ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer,
        validated: true,       // FileRecipes are authored and trusted
        reliability_score: 90, // High confidence for authored recipes
        reliability_signals: signals,
        reliability_explanation: None,
        domain,
        evidence: EvidenceBlock::default(),
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript,
        execution_trace: Some(trace),
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    }
}

/// v0.0.406: Execute a FileRecipe and return the result
pub fn execute_file_recipe(
    recipe: &FileRecipe,
    probe_outputs: HashMap<String, String>,
) -> ExecutionResult {
    let context = RecipeContext {
        probe_outputs,
        execute: true,
        ..Default::default()
    };

    // Probe lookup function (just returns the stored output)
    let probe_lookup = |_id: &str| -> Option<String> { None };

    execute_recipe(recipe, &context, probe_lookup)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_from_str() {
        assert!(matches!(
            domain_from_str("system"),
            SpecialistDomain::System
        ));
        assert!(matches!(
            domain_from_str("storage"),
            SpecialistDomain::Storage
        ));
        assert!(matches!(
            domain_from_str("network"),
            SpecialistDomain::Network
        ));
        assert!(matches!(
            domain_from_str("unknown"),
            SpecialistDomain::System
        ));
    }
}
