//! LLM calling logic for specialist processing

use anna_shared::rpc::SpecialistDomain;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

use crate::ollama;
use crate::response_renderer::render_response;
use super::parsing::parse_specialist_json;
use super::types::JsonSpecialistResult;
use super::utils::domain_to_string;

/// Internal: Call LLM with prepared prompts
pub async fn call_llm_with_prompt(
    system_prompt: &str,
    user_message: &str,
    ticket_id: &str,
    domain: SpecialistDomain,
    model: &str,
    timeout_secs: u64,
) -> JsonSpecialistResult {
    // Full prompt for LLM
    let full_prompt = format!("{}\n\n{}", system_prompt, user_message);

    // v0.0.433: Log prompt size at INFO level for debugging token issues
    info!("LLM prompt size: {} chars, system={} chars, user={} chars",
        full_prompt.len(), system_prompt.len(), user_message.len());

    // Call LLM with streaming
    let token_count = Arc::new(AtomicUsize::new(0));
    let token_count_clone = Arc::clone(&token_count);

    let result = timeout(
        Duration::from_secs(timeout_secs),
        ollama::chat_streaming_with_retry(model, &full_prompt, timeout_secs, move |_token| {
            token_count_clone.fetch_add(1, Ordering::Relaxed);
        }),
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
                    let error_response = anna_shared::specialist_contract::SpecialistResponse::parse_error(ticket_id, &e);
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
            let error_response = anna_shared::specialist_contract::SpecialistResponse::parse_error(ticket_id, &e.to_string());
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
            let error_response = anna_shared::specialist_contract::SpecialistResponse::parse_error(ticket_id, "Timeout");
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
