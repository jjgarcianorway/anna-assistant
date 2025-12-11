//! JSON-only specialist handler (v0.0.404).
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

use anna_shared::rpc::{ProbeResult, SpecialistDomain, TranslatorTicket, QueryIntent};
use anna_shared::specialist_contract::SpecialistResponse;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

use crate::evidence_integration::{
    build_evidence_for_specialist, build_enhanced_specialist_input, EvidenceIntegrationResult,
};
use crate::ollama;
use crate::response_renderer::{render_response, format_for_display, RenderedResponse};
use crate::specialist_prompt::{build_specialist_prompt, build_specialist_input};

/// Result of JSON specialist processing
#[derive(Debug, Clone)]
pub struct JsonSpecialistResult {
    /// The parsed specialist response (if successful)
    pub response: Option<SpecialistResponse>,
    /// Rendered output with personality
    pub rendered: RenderedResponse,
    /// Whether LLM was called or deterministic path used
    pub used_llm: bool,
    /// Raw LLM output (for debugging)
    pub raw_output: Option<String>,
}

/// Call specialist with evidence pipeline (v0.0.410)
/// This is the new preferred entry point - uses full evidence gathering
pub async fn call_json_specialist_with_evidence(
    ticket: &TranslatorTicket,
    ticket_id: &str,
    question: &str,
    probe_results: &[ProbeResult],
    model: &str,
    timeout_secs: u64,
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

    call_llm_with_prompt(&system_prompt, &user_message, ticket_id, domain, model, timeout_secs).await
}

/// Internal: Call LLM with prepared prompts
async fn call_llm_with_prompt(
    system_prompt: &str,
    user_message: &str,
    ticket_id: &str,
    domain: SpecialistDomain,
    model: &str,
    timeout_secs: u64,
) -> JsonSpecialistResult {
    // Full prompt for LLM
    let full_prompt = format!("{}\n\n{}", system_prompt, user_message);

    debug!(
        "LLM prompt length: {} chars",
        full_prompt.len()
    );

    // Call LLM with streaming
    let token_count = Arc::new(AtomicUsize::new(0));
    let token_count_clone = Arc::clone(&token_count);

    let result = timeout(
        Duration::from_secs(timeout_secs),
        ollama::chat_streaming_with_retry(
            model,
            &full_prompt,
            timeout_secs,
            move |_token| {
                token_count_clone.fetch_add(1, Ordering::Relaxed);
            },
        ),
    )
    .await;

    match result {
        Ok(Ok(raw_output)) => {
            let tokens = token_count.load(Ordering::Relaxed);
            info!("JSON specialist generated {} tokens", tokens);

            // Parse JSON from output
            match parse_specialist_json(&raw_output, ticket_id) {
                Ok(response) => {
                    info!(
                        "Parsed specialist response: status={:?}, confidence={}",
                        response.status, response.confidence
                    );

                    // Render with personality
                    let rendered = render_response(&response, domain_to_string(domain));

                    JsonSpecialistResult {
                        response: Some(response),
                        rendered,
                        used_llm: true,
                        raw_output: Some(raw_output),
                    }
                }
                Err(e) => {
                    warn!("Failed to parse specialist JSON: {}", e);

                    // Create error response
                    let error_response = SpecialistResponse::parse_error(ticket_id, &e);
                    let rendered = render_response(&error_response, domain_to_string(domain));

                    JsonSpecialistResult {
                        response: Some(error_response),
                        rendered,
                        used_llm: true,
                        raw_output: Some(raw_output),
                    }
                }
            }
        }
        Ok(Err(e)) => {
            error!("JSON specialist LLM error: {}", e);
            let error_response = SpecialistResponse::parse_error(ticket_id, &e.to_string());
            let rendered = render_response(&error_response, domain_to_string(domain));

            JsonSpecialistResult {
                response: Some(error_response),
                rendered,
                used_llm: true,
                raw_output: None,
            }
        }
        Err(_) => {
            warn!("JSON specialist timeout after {}s", timeout_secs);
            let error_response = SpecialistResponse::parse_error(ticket_id, "Timeout");
            let rendered = render_response(&error_response, domain_to_string(domain));

            JsonSpecialistResult {
                response: Some(error_response),
                rendered,
                used_llm: true,
                raw_output: None,
            }
        }
    }
}

/// Parse JSON from LLM output, handling common issues
/// v0.0.409: Now includes validation for forbidden patterns
fn parse_specialist_json(raw: &str, ticket_id: &str) -> Result<SpecialistResponse, String> {
    // Try to find JSON object in the output
    let json_str = extract_json_object(raw)?;

    // Parse JSON
    let mut response: SpecialistResponse = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    // v0.0.409: Validate for forbidden patterns
    let validation_errors = response.validate();
    if !validation_errors.is_empty() {
        warn!(
            "Specialist response validation failed for {}: {:?}",
            ticket_id, validation_errors
        );
        // Downgrade confidence and mark as error if validation fails
        response.confidence = response.confidence.min(0.3);
        if let Some(ref mut staff_view) = response.staff_view {
            staff_view.mood = anna_shared::specialist_contract::Mood::Blocked;
            staff_view.short_note = Some(format!("Validation failed: {}", validation_errors.join(", ")));
        }
        // If forbidden pattern detected, this is an error
        if validation_errors.iter().any(|e| e.contains("forbidden pattern")) {
            return Err(format!("Validation failed: {}", validation_errors.join(", ")));
        }
    }

    Ok(response)
}

/// Extract JSON object from LLM output
/// Handles cases where LLM adds prose before/after JSON
fn extract_json_object(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();

    // If it starts with {, try to find matching }
    if trimmed.starts_with('{') {
        // Find the last } which should close the object
        if let Some(end) = trimmed.rfind('}') {
            return Ok(trimmed[..=end].to_string());
        }
    }

    // Try to find JSON block in markdown
    if let Some(start) = raw.find("```json") {
        let after_marker = &raw[start + 7..];
        if let Some(end) = after_marker.find("```") {
            return Ok(after_marker[..end].trim().to_string());
        }
    }

    // Try to find bare code block
    if let Some(start) = raw.find("```") {
        let after_marker = &raw[start + 3..];
        if let Some(end) = after_marker.find("```") {
            let content = after_marker[..end].trim();
            if content.starts_with('{') {
                return Ok(content.to_string());
            }
        }
    }

    // Look for { anywhere in the output
    if let Some(start) = raw.find('{') {
        if let Some(end) = raw.rfind('}') {
            if end > start {
                return Ok(raw[start..=end].to_string());
            }
        }
    }

    Err("No JSON object found in output".to_string())
}

/// Convert domain enum to string
/// v0.0.405: Expanded for all domains
fn domain_to_string(domain: SpecialistDomain) -> &'static str {
    match domain {
        SpecialistDomain::System => "system",
        SpecialistDomain::Boot => "boot",
        SpecialistDomain::Services => "services",
        SpecialistDomain::Network => "network",
        SpecialistDomain::Storage => "storage",
        SpecialistDomain::Packages => "packages",
        SpecialistDomain::Audio => "audio",
        SpecialistDomain::Display => "display",
        SpecialistDomain::Desktop => "desktop",
        SpecialistDomain::Security => "security",
    }
}

/// Get the final answer text from a JSON specialist result
pub fn get_answer_text(result: &JsonSpecialistResult) -> String {
    result.rendered.answer.clone()
}

/// Get the formatted display output
pub fn get_display_output(result: &JsonSpecialistResult, ticket_id: &str) -> String {
    format_for_display(&result.rendered, ticket_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_object() {
        // Clean JSON
        let json = r#"{"ticket_id": "DSK-0101", "status": "ok"}"#;
        assert!(extract_json_object(json).is_ok());

        // JSON with surrounding prose
        let with_prose = r#"Here is the response:
{"ticket_id": "DSK-0101", "status": "ok"}
Done."#;
        let extracted = extract_json_object(with_prose).unwrap();
        assert!(extracted.contains("ticket_id"));

        // JSON in markdown block
        let markdown = r#"```json
{"ticket_id": "DSK-0101", "status": "ok"}
```"#;
        assert!(extract_json_object(markdown).is_ok());
    }

    #[test]
    fn test_parse_specialist_json() {
        let json = r#"{"ticket_id": "DSK-0101", "status": "ok", "answer": {"short": "Test"}, "evidence": [], "confidence": 0.9}"#;
        let result = parse_specialist_json(json, "DSK-0101");
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.answer.short, "Test");
    }

    #[test]
    fn test_validation_rejects_unknown_is_installed() {
        let json = r#"{"ticket_id": "DSK-0101", "status": "ok", "answer": {"short": "unknown is installed"}, "evidence": [], "confidence": 0.9}"#;
        let result = parse_specialist_json(json, "DSK-0101");
        // Should fail validation due to forbidden pattern
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("forbidden pattern"));
    }

    #[test]
    fn test_validation_rejects_number_as_package() {
        let json = r#"{"ticket_id": "DSK-0101", "status": "ok", "answer": {"short": "2 is installed on your system"}, "evidence": [], "confidence": 0.9}"#;
        let result = parse_specialist_json(json, "DSK-0101");
        assert!(result.is_err());
    }
}
