//! Public API functions for the translator module.

use anna_shared::answer_contract::AnswerContract;
use anna_shared::rpc::TranslatorTicket;
use std::time::Instant;
use tracing::info;

use crate::ollama;

use super::parsers::parse_translator_response;
use super::prompt::build_translator_request;
use super::types::{TranslatorInput, TranslatorResult};

/// Translate user query to structured ticket using LLM (with minimal input)
/// v0.0.74: Now generates AnswerContract from query for answer shaping
pub async fn translate_with_context(
    model: &str,
    input: &TranslatorInput,
    timeout_secs: u64,
) -> Result<TranslatorTicket, String> {
    let result = translate_with_debug(model, input, timeout_secs).await?;
    Ok(result.ticket)
}

/// v0.0.318: Translate with full debug info (prompt, response, timing)
pub async fn translate_with_debug(
    model: &str,
    input: &TranslatorInput,
    timeout_secs: u64,
) -> Result<TranslatorResult, String> {
    let full_prompt = build_translator_request(input);

    info!(
        "Translator: processing query (payload {} bytes)",
        full_prompt.len()
    );

    let start = Instant::now();
    let response = ollama::chat_with_timeout(model, &full_prompt, timeout_secs)
        .await
        .map_err(|e| format!("LLM error: {}", e))?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let mut ticket = parse_translator_response(&response, &input.query)?;

    // v0.0.74: Generate answer contract from original query
    ticket.answer_contract = Some(AnswerContract::from_query(&input.query));

    Ok(TranslatorResult {
        ticket,
        prompt: full_prompt,
        response,
        duration_ms,
    })
}

/// Legacy translate function (for compatibility/tests)
#[allow(dead_code)]
pub async fn translate(model: &str, query: &str) -> Result<TranslatorTicket, String> {
    // Use default hardware values for legacy calls
    let input = TranslatorInput::new(query, 4, 8.0, false);
    let full_prompt = build_translator_request(&input);

    info!("Translator: processing query");

    let response = ollama::chat(model, &full_prompt)
        .await
        .map_err(|e| format!("LLM error: {}", e))?;

    parse_translator_response(&response, query)
}
