//! Public API functions for JSON specialist handling
//!
//! This module implements the new architecture where:
//! - Specialists ONLY output JSON (no prose, no personality)
//! - JSON is parsed into SpecialistResponse struct
//! - Personality is added by the renderer layer
//!
//! Key benefits:
//! - Deterministic parsing (JSON is parseable)
//! - LLM can't hallucinate prose or wrong formats
//! - Personality is template-based, not LLM-generated
//! - Learning hooks in discovery field
//!
//! v0.0.410: Evidence pipeline integration - specialists now receive structured evidence

use anna_shared::rpc::{ProbeResult, SpecialistDomain, TranslatorTicket};
use anna_shared::specialist_contract::SpecialistResponse;
use std::collections::HashMap;
use tracing::info;

use crate::evidence_integration::{
    build_enhanced_specialist_input, build_evidence_for_specialist,
};
use crate::response_renderer::render_response;
use crate::specialist_prompt::{build_specialist_input, build_specialist_prompt};
use super::llm::call_llm_with_prompt;
use super::types::JsonSpecialistResult;
use super::utils::domain_to_string;

/// Call specialist with evidence pipeline (v0.0.410)
/// This is the new preferred entry point - uses full evidence gathering
pub async fn call_json_specialist_with_evidence(
    ticket: &TranslatorTicket,
    ticket_id: &str,
    question: &str,
    probe_results: &[ProbeResult],
    model: &str,
    timeout_secs: u64,
    context_claims: Option<String>, // Added context_claims parameter
) -> JsonSpecialistResult {
    // Build evidence using the pipeline
    let evidence_result = build_evidence_for_specialist(ticket, ticket_id, question, probe_results);

    // Check for instant answer (skip LLM entirely)
    if let Some(instant_answer) = evidence_result.instant_answer {
        info!(
            "v0.0.410: Instant answer from knowledge, bypassing LLM: {}",
            &instant_answer[..instant_answer.len().min(50)]
        );

        // Build a synthetic response for instant answers
        let response = SpecialistResponse::instant_answer(ticket_id, &instant_answer);
        let rendered = render_response(&response, domain_to_string(ticket.domain));

        return JsonSpecialistResult {
            response: Some(response),
            rendered,
            used_llm: false,
            raw_output: None,
        };
    }

    // Build enhanced prompt with evidence bundle
    let system_prompt = build_specialist_prompt(ticket.domain);
    let user_message = build_enhanced_specialist_input(
        ticket_id,
        domain_to_string(ticket.domain),
        &ticket.intent.to_string().to_lowercase(),
        question,
        &evidence_result.bundle,
        context_claims, // Pass context claims
    );

    info!(
        "v0.0.410: JSON specialist with evidence: domain={:?}, probes={}, docs={}, recipes={}",
        ticket.domain,
        evidence_result.bundle.probes.len(),
        evidence_result.bundle.docs.len(),
        evidence_result.bundle.recipes.len()
    );

    // Call LLM with evidence-enriched prompt
    call_llm_with_prompt(
        &system_prompt,
        &user_message,
        ticket_id,
        ticket.domain,
        model,
        timeout_secs,
    )
    .await
}

/// Call specialist with JSON-only contract (legacy - no evidence pipeline)
pub async fn call_json_specialist(
    domain: SpecialistDomain,
    ticket_id: &str,
    question: &str,
    probe_results: &[ProbeResult],
    model: &str,
    timeout_secs: u64,
) -> JsonSpecialistResult {
    // Build the JSON-only system prompt
    let system_prompt = build_specialist_prompt(domain);

    // Build probe map from results
    let probes: HashMap<String, String> = probe_results
        .iter()
        .filter(|p| p.exit_code == 0)
        .map(|p| (p.command.clone(), p.stdout.clone()))
        .collect();

    // Build the user message with ticket data
    let user_message = build_specialist_input(
        ticket_id,
        domain_to_string(domain),
        "question",
        question,
        &probes,
    );

    info!(
        "Calling JSON specialist for domain={:?}, ticket={}, probes={}",
        domain,
        ticket_id,
        probes.len()
    );

    call_llm_with_prompt(
        &system_prompt,
        &user_message,
        ticket_id,
        domain,
        model,
        timeout_secs,
    )
    .await
}

/// Get the final answer text from a JSON specialist result
pub fn get_answer_text(result: &JsonSpecialistResult) -> String {
    result.rendered.answer.clone()
}

/// Get the formatted display output
pub fn get_display_output(result: &JsonSpecialistResult, ticket_id: &str) -> String {
    crate::response_renderer::format_for_display(&result.rendered, ticket_id)
}
