//! LLM integration for Wiki Reasoning Engine
//!
//! Calls Ollama with structured prompts to generate WikiAdvice from wiki content + telemetry

use crate::wiki_reasoner::{WikiAdvice, WikiError};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

const OLLAMA_URL: &str = "http://127.0.0.1:11434/v1/chat/completions";
const DEFAULT_MODEL: &str = "llama3.2:3b";
const TIMEOUT_SECS: u64 = 60;

/// LLM response from Ollama
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    choices: Vec<OllamaChoice>,
}

#[derive(Debug, Deserialize)]
struct OllamaChoice {
    message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}

/// Call LLM with wiki reasoning prompt
pub async fn reason_with_llm(
    question: &str,
    system_context: &str,
    wiki_snippet: &str,
) -> Result<WikiAdvice, WikiError> {
    debug!("Calling LLM for wiki reasoning");

    // Build structured prompt
    let prompt = build_wiki_reasoning_prompt(question, system_context, wiki_snippet);

    // Call Ollama
    let response_text = call_ollama(&prompt)
        .await
        .map_err(|e| WikiError::LlmFailure(e.to_string()))?;

    // Parse WikiAdvice from response
    parse_wiki_advice(&response_text)
}

/// Build the wiki reasoning prompt
fn build_wiki_reasoning_prompt(question: &str, system_context: &str, wiki_snippet: &str) -> String {
    format!(
        r#"You are Anna, an Arch Linux system administrator assistant. Your role is to provide accurate, actionable guidance based on the Arch Wiki.

User Question: {}

System Context:
{}

Relevant Arch Wiki Content:
{}

Your task:
1. Provide a 2-5 line summary explaining the situation and approach
2. List concrete, ordered steps to address the question
3. For each step, include safe diagnostic commands (prefer read-only operations)
4. Add warnings for any potentially risky operations
5. Include helpful notes from the wiki
6. Cite the wiki pages you used

Output ONLY valid JSON matching this exact schema:
{{
  "summary": "Brief explanation of the situation and recommended approach",
  "steps": [
    {{
      "title": "Brief step title",
      "description": "Detailed explanation of what and why",
      "commands": ["command1", "command2"],
      "caution": "Optional warning if operation has risks"
    }}
  ],
  "notes": [
    "Additional helpful tips from the wiki"
  ],
  "citations": [
    {{
      "url": "https://wiki.archlinux.org/title/PageName",
      "section": "Section name",
      "note": "Brief description"
    }}
  ]
}}

Important guidelines:
- Prioritize read-only diagnostic commands
- Add cautions for any commands that modify the system
- Base all advice on the wiki content provided
- Keep commands practical and safe
- Always include at least one wiki citation"#,
        question, system_context, wiki_snippet
    )
}

/// Call Ollama API
async fn call_ollama(prompt: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "model": DEFAULT_MODEL,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "stream": false,
        "temperature": 0.3,  // Lower temperature for more consistent JSON
    });

    let response = client
        .post(OLLAMA_URL)
        .json(&payload)
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .send()
        .await
        .context("Failed to send request to Ollama")?;

    if !response.status().is_success() {
        anyhow::bail!("Ollama returned error status: {}", response.status());
    }

    let ollama_response: OllamaResponse = response
        .json()
        .await
        .context("Failed to parse Ollama response")?;

    let content = ollama_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow::anyhow!("No response from LLM"))?;

    Ok(content)
}

/// Parse WikiAdvice from LLM response
fn parse_wiki_advice(response: &str) -> Result<WikiAdvice, WikiError> {
    // Try to extract JSON from response (LLM might wrap it in markdown)
    let json_str = extract_json(response);

    // Parse JSON
    let advice: WikiAdvice = serde_json::from_str(&json_str).map_err(|e| {
        warn!("Failed to parse WikiAdvice JSON: {}", e);
        warn!("Response was: {}", response);
        WikiError::ParsingFailure(format!("Invalid JSON: {}", e))
    })?;

    // Validate
    if advice.summary.is_empty() {
        return Err(WikiError::ParsingFailure("Empty summary".to_string()));
    }
    if advice.steps.is_empty() {
        return Err(WikiError::ParsingFailure("No steps provided".to_string()));
    }
    if advice.citations.is_empty() {
        return Err(WikiError::ParsingFailure(
            "No wiki citations provided".to_string(),
        ));
    }

    Ok(advice)
}

/// Extract JSON from LLM response (handle markdown code blocks)
fn extract_json(response: &str) -> String {
    let trimmed = response.trim();

    // If starts with ```json or ```, extract the content
    if trimmed.starts_with("```") {
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.len() >= 3 {
            // Skip first and last line (the ``` markers)
            let json_lines = &lines[1..lines.len() - 1];
            return json_lines.join("\n");
        }
    }

    // Otherwise return as-is
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_plain() {
        let response = r#"{"summary": "test"}"#;
        let extracted = extract_json(response);
        assert_eq!(extracted, r#"{"summary": "test"}"#);
    }

    #[test]
    fn test_extract_json_markdown() {
        let response = r#"```json
{"summary": "test"}
```"#;
        let extracted = extract_json(response);
        assert_eq!(extracted, r#"{"summary": "test"}"#);
    }

    #[test]
    fn test_build_prompt_contains_key_elements() {
        let prompt = build_wiki_reasoning_prompt("test question", "test context", "test wiki");
        assert!(prompt.contains("test question"));
        assert!(prompt.contains("test context"));
        assert!(prompt.contains("test wiki"));
        assert!(prompt.contains("JSON"));
        assert!(prompt.contains("citations"));
    }
}
