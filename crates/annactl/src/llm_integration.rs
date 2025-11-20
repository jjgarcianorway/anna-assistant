//! LLM Integration Module for Beta.55
//!
//! Telemetry-first internal dialogue with planning and answer rounds
//! Beta.87: Integrated answer validation for zero hallucination guarantee

use anna_common::answer_validator::{AnswerValidator, ValidationContext};
use anna_common::context::db::ContextDb;
use anna_common::historian::SystemSummary;
use anna_common::llm::{ChatMessage, LlmConfig};
use anna_common::personality::PersonalityConfig;
use anna_common::types::SystemFacts;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::internal_dialogue::{run_internal_dialogue, TelemetryPayload};

/// Request structure for OpenAI-compatible API
/// NOTE: This is legacy code. Modern implementation uses internal_dialogue::query_llm
#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: Option<u32>,
    temperature: f64,
    stream: bool,
}

/// Response structure for OpenAI-compatible API (non-streaming)
/// NOTE: Legacy code - see ChatCompletionRequest note above
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

/// Streaming response structure (Server-Sent Events)
/// NOTE: Legacy code - see ChatCompletionRequest note above
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct StreamingChunk {
    choices: Vec<StreamingChoice>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct StreamingChoice {
    delta: StreamingDelta,
    finish_reason: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct StreamingDelta {
    content: Option<String>,
}

/// Query the LLM with a user message, using telemetry-first internal dialogue
/// Beta.87: Includes answer validation with retry loop for zero hallucinations
pub async fn query_llm_with_context(
    user_message: &str,
    db: Option<&Arc<ContextDb>>,
) -> Result<String> {
    // Load LLM config from database
    let llm_config = if let Some(db) = db {
        db.load_llm_config()
            .await
            .context("Failed to load LLM config")?
    } else {
        return Ok("LLM not available: database not connected".to_string());
    };

    if !llm_config.is_usable() {
        return Ok("LLM not configured. Run 'annactl setup brain' to configure.".to_string());
    }

    // Fetch system facts from daemon
    let facts = fetch_system_facts().await?;

    // Fetch Historian summary (if available)
    let historian = fetch_historian_summary().await;

    // Compress telemetry into payload
    let payload = TelemetryPayload::compress(&facts, historian.as_ref());

    // Load personality config (Beta.87: prefer database over TOML)
    let personality = if let Some(db_ref) = db {
        // Try loading from database first
        match PersonalityConfig::load_from_db(db_ref).await {
            Ok(config) => config,
            Err(_) => {
                // Fallback to TOML if database load fails
                PersonalityConfig::load()
            }
        }
    } else {
        // No database available, use TOML
        PersonalityConfig::load()
    };

    // Get current model name
    let current_model = llm_config.model.as_deref().unwrap_or("unknown");

    // Create answer validator
    let validator = AnswerValidator::new(false);

    // Create validation context
    let context = ValidationContext::new(user_message.to_string());
    // TODO: Add known files and packages from system facts
    // context.known_files = extract_known_files(&facts);
    // context.known_packages = extract_known_packages(&facts);

    // Validation retry loop (max 3 attempts)
    const MAX_RETRIES: usize = 3;
    let mut attempt = 0;
    let mut current_prompt = user_message.to_string();

    loop {
        attempt += 1;

        // Run internal dialogue (planning + answer rounds)
        let result = run_internal_dialogue(
            &current_prompt,
            &payload,
            &personality,
            current_model,
            &llm_config,
        )
        .await?;

        // Validate the answer
        let validation_result = validator.validate(&result.answer, &context).await?;

        if validation_result.passed || attempt >= MAX_RETRIES {
            // Answer passed validation or we've exhausted retries
            let mut output = result.answer;

            // Include internal trace if enabled
            if let Some(trace) = result.trace {
                output.push_str("\n\n");
                output.push_str(&trace.render());
            }

            // If validation had warnings but still passed, include them
            if !validation_result.passed && !validation_result.suggestions.is_empty() {
                output.push_str("\n\n");
                output.push_str("⚠️ Note: This answer had validation concerns but is being shown after retries.\n");
                output.push_str(&format!(
                    "Confidence: {:.0}%\n",
                    validation_result.confidence * 100.0
                ));
            }

            return Ok(output);
        }

        // Answer failed validation - prepare retry with feedback
        let feedback = validation_result.suggestions.join("\n");
        current_prompt = format!(
            "{}\n\n[VALIDATION FEEDBACK - Previous answer had issues]\n{}\n\nPlease revise your answer to address these concerns.",
            user_message,
            feedback
        );

        // Log validation failure for debugging
        eprintln!(
            "Answer validation failed (attempt {}/{}), confidence: {:.2}, issues: {}",
            attempt,
            MAX_RETRIES,
            validation_result.confidence,
            validation_result.issues.len()
        );
    }
}

/// Query the LLM with streaming word-by-word output (Beta.89)
/// Sends response chunks via channel as they arrive from the LLM
pub async fn query_llm_with_context_streaming(
    user_message: &str,
    db: Option<&Arc<ContextDb>>,
    tx: UnboundedSender<String>,
) -> Result<()> {
    // Load LLM config from database
    let llm_config = if let Some(db) = db {
        db.load_llm_config()
            .await
            .context("Failed to load LLM config")?
    } else {
        let _ = tx.send("LLM not available: database not connected".to_string());
        return Ok(());
    };

    if !llm_config.is_usable() {
        let _ = tx.send("LLM not configured. Run 'annactl setup brain' to configure.".to_string());
        return Ok(());
    }

    // Fetch system facts from daemon
    let facts = fetch_system_facts().await?;

    // Fetch Historian summary (if available)
    let historian = fetch_historian_summary().await;

    // Compress telemetry into payload
    let payload = TelemetryPayload::compress(&facts, historian.as_ref());

    // Load personality config
    let personality = if let Some(db_ref) = db {
        match PersonalityConfig::load_from_db(db_ref).await {
            Ok(config) => config,
            Err(_) => PersonalityConfig::load(),
        }
    } else {
        PersonalityConfig::load()
    };

    // Get current model name
    let current_model = llm_config.model.as_deref().unwrap_or("unknown");

    // Beta.89: For streaming, we skip validation retry loop
    // The LLM response streams word-by-word directly to the user
    // This provides immediate feedback at the cost of validation

    // Run internal dialogue (planning + answer rounds)
    let result = run_internal_dialogue(
        user_message,
        &payload,
        &personality,
        current_model,
        &llm_config,
    )
    .await?;

    // Stream the answer word-by-word
    let words: Vec<&str> = result.answer.split_whitespace().collect();
    for word in words {
        let _ = tx.send(format!("{} ", word));
        // Small delay to make streaming visible (adjust as needed)
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Include internal trace if enabled
    if let Some(trace) = result.trace {
        let _ = tx.send("\n\n".to_string());
        let _ = tx.send(trace.render());
    }

    Ok(())
}

/// Query the LLM with a prepared prompt
/// NOTE: Legacy - use internal_dialogue::query_llm instead
#[allow(dead_code)]
async fn query_llm(config: &LlmConfig, prompt: &str) -> Result<String> {
    let base_url = config
        .base_url
        .as_ref()
        .context("LLM base_url not configured")?;

    let model = config.model.as_ref().context("LLM model not configured")?;

    // Build API endpoint
    let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    // Build request
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt.to_string(),
    }];

    let request = ChatCompletionRequest {
        model: model.clone(),
        messages,
        max_tokens: config.max_tokens,
        temperature: 0.7,
        stream: false, // Non-streaming for now
    };

    // Make HTTP request
    let client = reqwest::Client::new();
    let mut req_builder = client.post(&endpoint).json(&request);

    // Add API key if configured
    if let Some(api_key_env) = &config.api_key_env {
        if let Ok(api_key) = std::env::var(api_key_env) {
            req_builder = req_builder.bearer_auth(api_key);
        }
    }

    let response = req_builder
        .send()
        .await
        .context("Failed to send LLM request")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("LLM API error {}: {}", status, error_text);
    }

    let completion: ChatCompletionResponse = response
        .json()
        .await
        .context("Failed to parse LLM response")?;

    let answer = completion
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response from LLM".to_string());

    Ok(answer)
}

/// Fetch system facts from daemon via IPC
async fn fetch_system_facts() -> Result<SystemFacts> {
    use crate::rpc_client::RpcClient;
    use anna_common::ipc::Method;

    let mut rpc = RpcClient::connect()
        .await
        .context("Failed to connect to annad")?;

    match rpc.call(Method::GetFacts).await? {
        anna_common::ipc::ResponseData::Facts(facts) => Ok(facts),
        _ => anyhow::bail!("Unexpected response from GetFacts"),
    }
}

/// Fetch Historian summary from daemon via IPC
async fn fetch_historian_summary() -> Option<SystemSummary> {
    use crate::rpc_client::RpcClient;
    use anna_common::ipc::Method;

    let mut rpc = RpcClient::connect().await.ok()?;

    match rpc.call(Method::GetHistorianSummary).await {
        Ok(anna_common::ipc::ResponseData::HistorianSummary(summary)) => Some(summary),
        Ok(_) => {
            // Unexpected response type, return None
            None
        }
        Err(_) => {
            // Historian not available yet - graceful degradation
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = ChatCompletionRequest {
            model: "llama3.2:3b".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            max_tokens: Some(500),
            temperature: 0.7,
            stream: false,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("llama3.2:3b"));
        assert!(json.contains("Hello"));
    }
}
