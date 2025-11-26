//! Query Handler v8 - Pure LLM-Driven Architecture
//!
//! The LLM is the brain.
//! annad is only: memory (telemetry), hands (tools), message relay.
//! Single prompt. Single loop. No planner/interpreter split.

use anna_common::brain_v8::BrainOrchestrator;
use anna_common::llm_client::LlmConfig;
use anna_common::telemetry::SystemTelemetry;
use anyhow::Result;

/// Handle a query using the v8 pure LLM architecture
pub async fn handle_query_v8(
    query: &str,
    telemetry: &SystemTelemetry,
    llm_config: Option<&LlmConfig>,
) -> Result<String> {
    // Check if this is a meta query about Anna
    if is_meta_query(query) {
        return Ok(handle_meta_query(query));
    }

    // Get LLM config
    let config = match llm_config {
        Some(c) if c.enabled => c.clone(),
        _ => {
            return Ok(
                "LLM is not configured. Anna requires a local LLM (Ollama) to answer questions.\n\n\
                 To enable:\n\
                 1. Install Ollama: curl -fsSL https://ollama.com/install.sh | sh\n\
                 2. Pull a model: ollama pull llama3.2:3b\n\
                 3. Restart Anna".to_string()
            );
        }
    };

    // Create orchestrator and process
    let orchestrator = BrainOrchestrator::new(config)?;
    orchestrator.process(query, telemetry)
}

/// Check if this is a meta query about Anna itself
fn is_meta_query(query: &str) -> bool {
    let q = query.to_lowercase();
    q.contains("anna version")
        || q.contains("who are you")
        || q.contains("about anna")
        || q.contains("upgrade your brain")
        || q.contains("upgrade brain")
        || q.contains("change model")
        || q.contains("what version")
}

/// Handle meta queries about Anna
fn handle_meta_query(query: &str) -> String {
    let q = query.to_lowercase();

    if q.contains("upgrade") || q.contains("brain") || q.contains("model") {
        return r#"To upgrade Anna's LLM:

1. List available models:
   ollama list

2. Pull a new model:
   ollama pull llama3.2:3b    # Fast, good for simple queries
   ollama pull llama3.1:8b    # Balanced
   ollama pull mistral:7b     # Good reasoning

3. Edit ~/.config/anna/config.toml:
   [llm]
   model = "llama3.2:3b"

4. Restart Anna:
   systemctl --user restart annad"#.to_string();
    }

    if q.contains("version") {
        return "Anna Assistant v8.0.0 - Pure LLM-Driven Architecture".to_string();
    }

    if q.contains("who are you") || q.contains("about anna") {
        return r#"I am Anna, a local Arch Linux system assistant.

My architecture (v8.0.0):
- The LLM is the brain - it controls planning, tool selection, and answers
- annad provides: telemetry (memory), tools (hands), message relay
- I never invent data - only use what I observe from your system
- I never hardcode recipes - the LLM reasons from evidence

If I don't know something, I'll say so honestly."#.to_string();
    }

    "I'm Anna, your local Arch Linux assistant. Ask me about your system!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_query_detection() {
        assert!(is_meta_query("what is Anna version"));
        assert!(is_meta_query("upgrade your brain"));
        assert!(is_meta_query("who are you"));
        assert!(!is_meta_query("how much RAM"));
    }

    #[test]
    fn test_meta_query_responses() {
        let upgrade = handle_meta_query("upgrade brain");
        assert!(upgrade.contains("ollama"));

        let version = handle_meta_query("anna version");
        assert!(version.contains("8.0.0"));
    }
}
