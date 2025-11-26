//! Query Handler v7 - Clean Brain Architecture
//!
//! Uses the brain_v7 module for strict three-phase pipeline:
//! PLAN → EXECUTE → INTERPRET
//!
//! This is the replacement for planner_query_handler.rs

use anna_common::{
    brain_v7::BrainOrchestrator,
    llm_client::LlmConfig,
    telemetry::SystemTelemetry,
};
use anyhow::Result;

/// Handle a query using the v7 brain architecture
pub async fn handle_query_v7(
    query: &str,
    _telemetry: &SystemTelemetry,
    llm_config: Option<&LlmConfig>,
) -> Result<String> {
    // Check if LLM is available
    let config = match llm_config {
        Some(c) if c.enabled => c.clone(),
        _ => {
            return Ok(llm_not_configured_message(query));
        }
    };

    // Create the v7 brain orchestrator
    let brain = match BrainOrchestrator::new(config) {
        Ok(b) => b,
        Err(e) => {
            return Ok(format!(
                "❌  Failed to connect to LLM: {}\n\n\
                 Make sure Ollama is running: systemctl start ollama\n\
                 Or check if the endpoint is correct.",
                e
            ));
        }
    };

    // Process the query through the v7 pipeline
    let result = brain.process(query);

    Ok(result.answer)
}

/// Get v7 tool catalog for display
pub fn get_v7_tools() -> Vec<String> {
    use anna_common::brain_v7::ToolCatalog;
    let catalog = ToolCatalog::new();
    catalog
        .get_descriptors()
        .iter()
        .map(|d| format!("{}: {}", d.name, d.description))
        .collect()
}

/// Message shown when LLM is not configured
fn llm_not_configured_message(query: &str) -> String {
    format!(
        "❌  LLM is not configured or disabled.\n\n\
         Anna v7 requires a local LLM (e.g., Ollama) to process queries.\n\n\
         To enable:\n\
         1. Install Ollama: curl -fsSL https://ollama.com/install.sh | sh\n\
         2. Pull a model: ollama pull qwen2.5:14b\n\
         3. Edit ~/.config/anna/config.toml:\n\
            [llm]\n\
            enabled = true\n\
            model = \"qwen2.5:14b\"\n\
         4. Restart Anna: systemctl --user restart annad\n\n\
         Query received: \"{}\"",
        query
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_v7_tools() {
        let tools = get_v7_tools();

        // Check key tools exist
        let tool_str = tools.join("\n");
        assert!(tool_str.contains("mem_info"), "Missing mem_info tool");
        assert!(tool_str.contains("cpu_info"), "Missing cpu_info tool");
        assert!(tool_str.contains("gpu_pci"), "Missing gpu_pci tool");
        assert!(tool_str.contains("pacman_search"), "Missing pacman_search tool");
    }
}
