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
use anna_common::telemetry::SystemTelemetry;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 6.3.1: recipes removed - planner is the only planning path
// use crate::recipes;
use crate::system_prompt_v3_json;

/// V3 dialogue result with structured action plan
#[derive(Debug, Clone)]
pub struct DialogueV3Result {
    /// Parsed action plan from LLM
    pub action_plan: ActionPlan,

    /// Raw JSON response (for debugging, empty string if from recipe)
    pub raw_json: String,
}

/// Run V3 JSON dialogue with LLM
///
/// This is the production implementation that replaces v1/v2 markdown dialogue.
/// Returns structured ActionPlan validated against JSON schema.
///
/// Beta.151: Now tries deterministic recipe matching before LLM fallback
/// Version 150: Updated to use SystemTelemetry directly from unified_query_handler
pub async fn run_dialogue_v3_json(
    user_request: &str,
    telemetry: &SystemTelemetry,
    llm_config: &LlmConfig,
) -> Result<DialogueV3Result> {
    // 6.3.1: Recipe system removed - LLM generation only
    // Old recipe matching logic disabled:
    // let telemetry_map = convert_telemetry_to_hashmap(telemetry);
    // if let Some(recipe_result) = recipes::try_recipe_match(user_request, &telemetry_map) {
    //     return Ok(DialogueV3Result { action_plan, raw_json: String::new() });
    // }

    // LLM generation

    // Build system prompt with JSON contract
    let system_prompt = system_prompt_v3_json::build_runtime_system_prompt();

    // Serialize telemetry to JSON
    let telemetry_json =
        serde_json::to_string_pretty(telemetry).context("Failed to serialize telemetry")?;

    // Build user prompt with request + telemetry
    let user_prompt = system_prompt_v3_json::build_user_prompt(
        user_request,
        &telemetry_json,
        "cli", // interaction mode (could be "cli" or "tui")
    );

    // Call LLM with system + user prompts
    let raw_json = query_llm_json(llm_config, &system_prompt, &user_prompt).await?;

    // Parse JSON response
    let action_plan: ActionPlan = serde_json::from_str(&raw_json).map_err(|parse_err| {
        // Beta.151: Log failed JSON responses for debugging
        log_failed_json_response(user_request, &raw_json, &parse_err);
        anyhow::anyhow!("Failed to parse LLM response as ActionPlan JSON: {}", parse_err)
    })?;

    // Validate action plan
    action_plan
        .validate()
        .map_err(|e| anyhow::anyhow!("Action plan validation failed: {}", e))?;

    Ok(DialogueV3Result {
        action_plan,
        raw_json,
    })
}

/// Beta.151: Log failed JSON responses for debugging
fn log_failed_json_response(user_request: &str, raw_response: &str, error: &serde_json::Error) {
    use std::fs;
    use std::io::Write;

    // Beta.151: Use user-accessible location first, fallback to /var/log/anna
    let log_dir = if let Ok(home) = std::env::var("HOME") {
        format!("{}/.local/share/anna/logs", home)
    } else {
        "/tmp/anna_logs".to_string()
    };

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let log_file = format!("{}/failed_json_{}.log", log_dir, timestamp);

    // Try to create log directory if it doesn't exist
    let _ = fs::create_dir_all(&log_dir);

    // Write log entry
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
    {
        let _ = writeln!(file, "=== Failed JSON Response ===");
        let _ = writeln!(file, "Timestamp: {}", chrono::Local::now());
        let _ = writeln!(file, "User Request: {}", user_request);
        let _ = writeln!(file, "Parse Error: {}", error);
        let _ = writeln!(file, "\nRaw LLM Response:");
        let _ = writeln!(file, "{}", raw_response);
        let _ = writeln!(file, "\n=========================\n");
    }

    // Also log to stderr for immediate visibility
    eprintln!("⚠️ JSON parse failed. Log saved to: {}", log_file);
}

/// Query LLM with system + user prompts, expect JSON response
async fn query_llm_json(
    config: &LlmConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    let base_url = config
        .base_url
        .as_ref()
        .context("LLM base_url not configured")?;

    let model = config.model.as_ref().context("LLM model not configured")?;

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

    // Beta.151: Detect if Ollama or OpenAI-compatible API
    let is_ollama = base_url.contains("11434") || base_url.to_lowercase().contains("ollama");

    let request = ChatCompletionRequest {
        model: model.clone(),
        messages,
        max_tokens: Some(4000), // Generous limit for JSON plans
        temperature: 0.1,       // Beta.151: Very low temperature for strict JSON adherence
        stream: false,
        // Beta.151: Force JSON mode
        format: if is_ollama {
            Some("json".to_string()) // Ollama: use format parameter
        } else {
            None
        },
        response_format: if !is_ollama {
            Some(ResponseFormat {
                format_type: "json_object".to_string(), // OpenAI-compatible: use response_format
            })
        } else {
            None
        },
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
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    temperature: f64,
    stream: bool,
    /// Beta.151: Force JSON output for Ollama
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    /// Beta.151: Force JSON output for OpenAI-compatible APIs
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

/// Response format specification for OpenAI-compatible APIs
#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
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

/// Convert SystemTelemetry to HashMap for recipe matching and LLM prompts
/// Version 150: Enables V3 dialogue to work with unified_query_handler's telemetry
fn convert_telemetry_to_hashmap(telemetry: &SystemTelemetry) -> HashMap<String, String> {
    let mut map = HashMap::new();

    // Hardware
    map.insert("cpu_model".to_string(), telemetry.hardware.cpu_model.clone());
    map.insert("cpu_cores".to_string(), telemetry.cpu.cores.to_string());
    map.insert("cpu_load".to_string(), telemetry.cpu.load_avg_1min.to_string());
    map.insert("total_ram_gb".to_string(), (telemetry.hardware.total_ram_mb as f64 / 1024.0).to_string());
    if let Some(ref gpu) = telemetry.hardware.gpu_info {
        map.insert("gpu_model".to_string(), gpu.clone());
    }

    // Memory
    map.insert("ram_used_gb".to_string(), (telemetry.memory.used_mb as f64 / 1024.0).to_string());
    map.insert("ram_total_gb".to_string(), (telemetry.memory.total_mb as f64 / 1024.0).to_string());

    // Disk
    if let Some(root_disk) = telemetry.disks.iter().find(|d| d.mount_point == "/") {
        let free_gb = (root_disk.total_mb - root_disk.used_mb) as f64 / 1024.0;
        map.insert("disk_free_gb".to_string(), free_gb.to_string());
        map.insert("disk_used_percent".to_string(), root_disk.usage_percent.to_string());
    }

    // Network
    map.insert("internet_connected".to_string(), telemetry.network.is_connected.to_string());

    // Desktop environment
    if let Some(ref desktop) = telemetry.desktop {
        if let Some(ref de) = desktop.de_name {
            map.insert("desktop_environment".to_string(), de.clone());
        }
        if let Some(ref wm) = desktop.wm_name {
            map.insert("window_manager".to_string(), wm.clone());
        }
        if let Some(ref display) = desktop.display_server {
            map.insert("display_protocol".to_string(), display.clone());
        }
    }

    // Get hostname from system
    if let Ok(hostname) = std::fs::read_to_string("/proc/sys/kernel/hostname") {
        map.insert("hostname".to_string(), hostname.trim().to_string());
    }

    // Get kernel
    if let Ok(output) = std::process::Command::new("uname").arg("-r").output() {
        if output.status.success() {
            map.insert("kernel".to_string(), String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    // User and home
    map.insert("user".to_string(), std::env::var("USER").unwrap_or_else(|_| "user".to_string()));
    map.insert("home".to_string(), std::env::var("HOME").unwrap_or_else(|_| "~".to_string()));

    map
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
        assert_eq!(
            cleaned,
            r#"{
  "test": "value"
}"#
        );

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
