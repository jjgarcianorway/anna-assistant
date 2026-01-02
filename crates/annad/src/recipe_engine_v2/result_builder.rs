//! Result builder module.
//!
//! Handles building ServiceDeskResult from recipe execution results.

use anna_shared::recipe_engine::Recipe as LearnedRecipe;
use anna_shared::recipe_executor::ExecutionResult;
use anna_shared::rpc::{
    EvidenceBlock, ReliabilitySignals, ServiceDeskResult, SpecialistDomain,
};
use anna_shared::trace::{ExecutionTrace, ProbeStats};
use anna_shared::transcript::Transcript;

/// Build ServiceDeskResult from recipe execution
pub fn build_learned_recipe_result(
    request_id: String,
    recipe: &LearnedRecipe,
    exec_result: ExecutionResult,
    _query: &str,
) -> ServiceDeskResult {
    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: !recipe.required_evidence.is_empty(),
        answer_grounded: true,
        no_invention: true,
        clarification_not_needed: true,
    };

    let domain = match recipe.domain.to_lowercase().as_str() {
        "services" | "system" => SpecialistDomain::System,
        "storage" => SpecialistDomain::Storage,
        "network" => SpecialistDomain::Network,
        "packages" => SpecialistDomain::Packages,
        "desktop" => SpecialistDomain::Desktop,
        _ => SpecialistDomain::System,
    };

    let trace = ExecutionTrace::deterministic_route(
        &format!("learned_recipe:{}", recipe.id),
        ProbeStats::default(),
        vec![],
    );

    // Build answer with sources
    let mut answer = exec_result.answer;
    if !recipe.doc_sources.is_empty() {
        answer.push_str("\n\n**Sources:**\n");
        for src in &recipe.doc_sources {
            answer.push_str(&format!("- {}\n", src));
        }
    }
    answer.push_str(&format!(
        "\n*Used learned recipe: {} (success rate: {:.0}%)*",
        recipe.name,
        recipe.success_rate() * 100.0
    ));

    ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: Some("Anna (Recipe Engine)".to_string()),
        staff_id: Some("recipe_engine".to_string()),
        answer,
        validated: exec_result.success,
        reliability_score: if exec_result.success { 90 } else { 40 },
        reliability_signals: signals,
        reliability_explanation: None,
        domain,
        evidence: EvidenceBlock::default(),
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript: Transcript::new(),
        execution_trace: Some(trace),
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    }
}
