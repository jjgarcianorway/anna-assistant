//! V3 JSON Dialogue Runner - Beta.145
//!
//! Implements JSON runtime contract from Beta.143.
//! Uses structured ActionPlan output instead of freeform markdown.
//!
//! Architecture:
//! 1. Build system prompt with strict JSON output requirements
//! 2. Build user prompt with telemetry JSON
//! 3. Call LLM (Ollama)
//! 4. Parse JSON response
//! 5. Validate against ActionPlan schema
//! 6. Return structured plan

use anna_common::action_plan_v3::ActionPlan;
use anna_common::llm::{ChatMessage, LlmConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::system_prompt_v3_json;
use crate::internal_dialogue::TelemetryPayload;

/// V3 dialogue result with structured action plan
#[derive(Debug, Clone)]
pub struct DialogueV3Result {
    /// Parsed action plan from LLM
    pub action_plan: ActionPlan,

    /// Raw JSON response (for debugging)
    pub raw_json: String,
}

/// Run V3 JSON dialogue with LLM
///
/// This is the production implementation that replaces v1/v2 markdown dialogue.
/// Returns structured ActionPlan validated against JSON schema.
pub async fn run_dialogue_v3_json(
    user_request: &str,
    telemetry: &TelemetryPayload,
    llm_config: &LlmConfig,
) -> Result<DialogueV3Result> {
    // Build system prompt with JSON contract
    let system_prompt = system_prompt_v3_json::build_runtime_system_prompt();

    // Serialize telemetry to JSON
    let telemetry_json = serde_json::to_string_pretty(telemetry)
        .context("Failed to serialize telemetry")?;

    // Build user prompt with request + telemetry
    let user_prompt = system_prompt_v3_json::build_user_prompt(
        user_request,
        &telemetry_json,
        "cli", // interaction mode (could be "cli" or "tui")
    );

    // Call LLM with system + user prompts
    let raw_json = query_llm_json(llm_config, &system_prompt, &user_prompt).await?;

    // Parse JSON response
    let action_plan: ActionPlan = serde_json::from_str(&raw_json)
        .context("Failed to parse LLM response as ActionPlan JSON")?;

    // Validate action plan
    action_plan.validate()
        .map_err(|e| anyhow::anyhow!("Action plan validation failed: {}", e))?;

    Ok(DialogueV3Result {
        action_plan,
        raw_json,
    })
}

/// Query LLM with system + user prompts, expect JSON response
async fn query_llm_json(
    config: &LlmConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    let base_url = config.base_url.as_ref()
        .context("LLM base_url not configured")?;

    let model = config.model.as_ref()
        .context("LLM model not configured")?;

    // Build API endpoint
    let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    // Build messages with system and user prompts
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_prompt.to_string(),
        },
    ];

    let request = ChatCompletionRequest {
        model: model.clone(),
        messages,
        max_tokens: Some(4000), // Generous limit for JSON plans
        temperature: 0.3, // Lower temperature for structured output
        stream: false,
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

    let content = completion
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "{}".to_string());

    // Clean up JSON if LLM wrapped it in markdown code blocks
    let cleaned = clean_json_response(&content);

    Ok(cleaned)
}

/// Clean up JSON response - remove markdown code blocks if present
fn clean_json_response(raw: &str) -> String {
    let trimmed = raw.trim();

    // Check if wrapped in ```json ... ``` or ``` ... ```
    if trimmed.starts_with("```") {
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.len() >= 3 {
            // Remove first line (```json or ```) and last line (```)
            let json_lines = &lines[1..lines.len() - 1];
            return json_lines.join("\n");
        }
    }

    // Return as-is if no markdown wrapping
    trimmed.to_string()
}

/// Request structure for OpenAI-compatible API
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: Option<u32>,
    temperature: f64,
    stream: bool,
}

/// Response structure for OpenAI-compatible API
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_json_response() {
        // Test markdown-wrapped JSON
        let input = r#"```json
{
  "test": "value"
}
```"#;
        let cleaned = clean_json_response(input);
        assert_eq!(cleaned, r#"{
  "test": "value"
}"#);

        // Test plain JSON
        let input = r#"{"test": "value"}"#;
        let cleaned = clean_json_response(input);
        assert_eq!(cleaned, r#"{"test": "value"}"#);

        // Test code block without json label
        let input = r#"```
{"test": "value"}
```"#;
        let cleaned = clean_json_response(input);
        assert_eq!(cleaned, r#"{"test": "value"}"#);
    }
}
