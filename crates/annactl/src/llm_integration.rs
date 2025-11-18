//! LLM Integration Module for Beta.53
//!
//! Provides query functionality with streaming support and Historian context

use anna_common::context::db::ContextDb;
use anna_common::historian::SystemSummary;
use anna_common::llm::{ChatMessage, LlmConfig};
use anna_common::types::SystemFacts;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Request structure for OpenAI-compatible API
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: Option<u32>,
    temperature: f64,
    stream: bool,
}

/// Response structure for OpenAI-compatible API (non-streaming)
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

/// Query the LLM with a user message, including full system context
pub async fn query_llm_with_context(
    user_message: &str,
    db: Option<&Arc<ContextDb>>,
) -> Result<String> {
    // Load LLM config from database
    let llm_config = if let Some(db) = db {
        db.load_llm_config().await.context("Failed to load LLM config")?
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

    // Build runtime prompt with all context
    let prompt = crate::runtime_prompt::build_runtime_prompt(
        user_message,
        &facts,
        historian.as_ref(),
        llm_config.model.as_deref().unwrap_or("unknown"),
    );

    // Query LLM
    query_llm(&llm_config, &prompt).await
}

/// Query the LLM with a prepared prompt
async fn query_llm(config: &LlmConfig, prompt: &str) -> Result<String> {
    let base_url = config.base_url.as_ref()
        .context("LLM base_url not configured")?;

    let model = config.model.as_ref()
        .context("LLM model not configured")?;

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

    let mut rpc = RpcClient::connect().await
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
